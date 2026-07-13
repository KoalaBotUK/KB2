use crate::AppState;
use crate::discord::ise;
use crate::guilds::models::Guild;
use crate::guilds::verify::models::VerifyRole;
use crate::utils::{is_client_admin_guild, secure_compare};
use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Extension, Json};
use common::verify::{ReconScope, VerifyJob, enqueue_recon};
use http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker};
use twilight_model::user::CurrentUser;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/recon", post(post_recon))
        .route("/job", get(get_job))
        .route(
            "/roles/{role_id}",
            put(put_roles_id).delete(delete_roles_id),
        )
}

#[derive(Serialize, Deserialize)]
struct PutRoleRequest {
    pub pattern: String,
}

/// Validates that a verify-role pattern compiles as a regex. Invalid
/// patterns must never be persisted: once stored, every code path that
/// matches links against the guild's roles would otherwise need to handle
/// (or, before this fix, panic on) an uncompilable pattern.
fn validate_verify_pattern(pattern: &str) -> Result<(), StatusCode> {
    if Regex::new(pattern).is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

/// Persists a job-generation bump and sends its SQS wake-up token.
///
/// Unlike the fire-and-forget audit publisher, enqueue failures here are
/// *propagated*: desired state was already saved, and swallowing the failure
/// would silently strand it un-reconciled. The admin retries and `supersede`
/// simply bumps another generation.
async fn start_recon_job(
    guild_id: Id<GuildMarker>,
    scope: ReconScope,
    app_state: &AppState,
) -> Result<VerifyJob, StatusCode> {
    let generation = VerifyJob::supersede(guild_id, scope, &app_state.pg_pool)
        .await
        .map_err(ise)?;
    let queue_url = std::env::var("SQS_URL").expect("SQS_URL must be set");
    enqueue_recon(
        guild_id,
        generation,
        Duration::ZERO,
        &app_state.sqs,
        &queue_url,
    )
    .await
    .map_err(ise)?;
    VerifyJob::from_db(guild_id, &app_state.pg_pool)
        .await
        .map_err(ise)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Computes the recon scope for a role add/replace.
///
/// A brand-new role is add-only (nobody can hold it yet, so the worker skips
/// remove calls entirely). Replacing an existing role's pattern needs a full
/// per-user sync of that role: users matching the new pattern gain it, users
/// matching only the old pattern lose it — without having to track the old
/// pattern at all.
fn put_role_scope(
    existing: Option<&VerifyRole>,
    role_id: Id<RoleMarker>,
    new_pattern: &str,
) -> ReconScope {
    match existing {
        Some(old) if old.pattern != new_pattern => ReconScope::role_sync(role_id),
        _ => ReconScope::role_add(role_id),
    }
}

async fn put_roles_id(
    Path((guild_id, role_id)): Path<(Id<GuildMarker>, Id<RoleMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(put_role_request): Json<PutRoleRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    validate_verify_pattern(&put_role_request.pattern)?;

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });

    let scope = put_role_scope(
        find_role(&guild.verify.roles, role_id),
        role_id,
        &put_role_request.pattern,
    );

    // 1. Persist desired state. The Discord fan-out happens asynchronously in
    //    the consumer's reconciliation worker — never inline in a request.
    guild.verify.roles.retain(|r| r.role_id != role_id);
    let new_role = VerifyRole {
        role_id,
        pattern: put_role_request.pattern,
        members: 0,
    };
    guild.verify.roles.push(new_role.clone());
    guild.save(&app_state.pg_pool).await?;

    // 2. Bump the job generation and wake the worker.
    let job = start_recon_job(guild_id, scope, &app_state).await?;

    // 3. 202: roles will propagate; the job snapshot lets the UI show progress.
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "role": new_role, "job": job })),
    ))
}

/// Locates a verify role by role id, without panicking when the role id
/// isn't present.
fn find_role(roles: &[VerifyRole], role_id: Id<RoleMarker>) -> Option<&VerifyRole> {
    roles.iter().find(|r| r.role_id == role_id)
}

async fn delete_roles_id(
    Path((guild_id, role_id)): Path<(Id<GuildMarker>, Id<RoleMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });

    let Some(existing) = find_role(&guild.verify.roles, role_id) else {
        // Nothing to remove, nothing to reconcile.
        return Ok((StatusCode::NO_CONTENT, Json(Value::Null)));
    };
    // Capture the pattern before the role vanishes from config: the worker
    // needs it to know which users to strip.
    let scope = ReconScope::role_remove(role_id, existing.pattern.clone());

    guild.verify.roles.retain(|r| r.role_id != role_id);
    guild.save(&app_state.pg_pool).await?;

    let job = start_recon_job(guild_id, scope, &app_state).await?;

    Ok((StatusCode::ACCEPTED, Json(json!({ "job": job }))))
}

async fn post_recon(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let tokens_match = match (discord_user.token(), app_state.discord_bot.token()) {
        (Some(a), Some(b)) => secure_compare(a, b),
        _ => false,
    };
    if !tokens_match {
        return Err(StatusCode::FORBIDDEN);
    }

    let job = start_recon_job(guild_id, ReconScope::all(), &app_state).await?;

    Ok((StatusCode::ACCEPTED, Json(json!({ "job": job }))))
}

/// Dashboard polling endpoint: the `verify_jobs` row (status, processed /
/// total, errors) for the guild's current/most-recent reconciliation job.
async fn get_job(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }
    let job = VerifyJob::from_db(guild_id, &app_state.pg_pool)
        .await
        .map_err(ise)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(json!(job)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_role(role_id: u64) -> VerifyRole {
        VerifyRole {
            role_id: Id::new(role_id),
            pattern: "pattern".to_string(),
            members: 0,
        }
    }

    #[test]
    fn find_role_returns_none_for_missing_role_id() {
        let roles = vec![sample_role(1), sample_role(2)];

        assert!(find_role(&roles, Id::new(999)).is_none());
    }

    #[test]
    fn find_role_returns_some_for_existing_role_id() {
        let roles = vec![sample_role(1), sample_role(2)];

        let found = find_role(&roles, Id::new(2)).expect("role should be found");
        assert_eq!(found.role_id, Id::new(2));
    }

    #[test]
    fn invalid_pattern_is_rejected_at_creation_time() {
        // `(` is not a valid regex — this must be rejected up front with a
        // clean 400 instead of being persisted and panicking later.
        assert_eq!(validate_verify_pattern("("), Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn valid_pattern_is_accepted() {
        assert_eq!(
            validate_verify_pattern(r"^https://example\.com/.*$"),
            Ok(())
        );
    }

    #[test]
    fn put_role_scope_for_a_new_role_is_add_only() {
        let scope = put_role_scope(None, Id::new(5), "@new$");

        assert_eq!(scope.add_role_ids, vec![Id::new(5)]);
        assert!(scope.removals.is_empty());
        assert!(!scope.all);
    }

    #[test]
    fn put_role_scope_replacing_a_pattern_requests_a_full_role_sync() {
        // Replacing an existing role's pattern must trigger a full per-user
        // sync of that role, otherwise users who matched the old pattern
        // (but not the new one) keep the role forever.
        let existing = VerifyRole {
            role_id: Id::new(5),
            pattern: "@old$".to_string(),
            members: 3,
        };

        let scope = put_role_scope(Some(&existing), Id::new(5), "@new$");

        assert_eq!(scope.sync_role_ids, vec![Id::new(5)]);
        assert!(scope.add_role_ids.is_empty());
        assert!(scope.removals.is_empty());
    }

    #[test]
    fn put_role_scope_with_unchanged_pattern_has_no_removal() {
        // Re-PUTting the same pattern (idempotent retry) must not schedule a
        // strip of the pattern it is simultaneously adding.
        let existing = VerifyRole {
            role_id: Id::new(5),
            pattern: "@same$".to_string(),
            members: 3,
        };

        let scope = put_role_scope(Some(&existing), Id::new(5), "@same$");

        assert_eq!(scope.add_role_ids, vec![Id::new(5)]);
        assert!(scope.removals.is_empty());
    }
}
