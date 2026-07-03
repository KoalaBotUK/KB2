use crate::AppState;
use crate::discord::{add_guild_member_role, remove_guild_member_role};
use crate::guilds::models::Guild;
use crate::guilds::verify::models::{Verify, VerifyRole};
use crate::users::utils::link_arr_match;
use crate::utils::{is_client_admin_guild, secure_compare};
use axum::extract::{Path, State};
use axum::routing::{post, put};
use axum::{Extension, Json};
use http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker};
use twilight_model::user::CurrentUser;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/recon", post(post_recon))
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

async fn put_roles_id(
    Path((guild_id, role_id)): Path<(Id<GuildMarker>, Id<RoleMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(put_role_request): Json<PutRoleRequest>,
) -> Result<Json<Value>, StatusCode> {
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

    if guild.verify.roles.iter().any(|r| r.role_id == role_id) {
        remove_existing_role(&mut guild, role_id, &app_state).await?;
    }

    let new_role = VerifyRole {
        role_id,
        pattern: put_role_request.pattern,
        ..Default::default()
    };

    for (user_id, user_links) in &guild.verify.user_links {
        if link_arr_match(user_links, &new_role.pattern) {
            let r =
                add_guild_member_role(guild.guild_id, *user_id, role_id, &app_state.discord_bot)
                    .await;
            if !(r.is_err() && r.err().unwrap().eq(&StatusCode::NOT_FOUND)) {
                r?;
            }
        }
    }
    guild.verify.roles.push(new_role);
    // `members` is derived from `user_links`, not hand-incremented, so it
    // can never drift out of sync with the other role/link handlers.
    guild.verify.recompute_role_members();
    guild.save(&app_state.pg_pool).await?;

    Ok(Json(json!(
        find_role(&guild.verify.roles, role_id).ok_or(StatusCode::NOT_FOUND)?
    )))
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
) -> Result<StatusCode, StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });

    remove_existing_role(&mut guild, role_id, &app_state).await?;

    guild.save(&app_state.pg_pool).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn remove_existing_role(
    guild: &mut Guild,
    role_id: Id<RoleMarker>,
    app_state: &AppState,
) -> Result<(), StatusCode> {
    let existing_role_opt = guild.verify.roles.iter().find(|r| r.role_id == role_id);

    if existing_role_opt.is_none() {
        // No role found
        return Ok(());
    }
    let existing_role = existing_role_opt.unwrap();

    for (user_id, user_links) in &guild.verify.user_links {
        if link_arr_match(user_links, &existing_role.pattern) {
            let r =
                remove_guild_member_role(guild.guild_id, *user_id, role_id, &app_state.discord_bot)
                    .await;
            if !(r.is_err() && r.err().unwrap().eq(&StatusCode::NOT_FOUND)) {
                r?;
            }
        }
    }

    guild.verify.roles.retain(|r| r.role_id != role_id);
    Ok(())
}

async fn post_recon(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let tokens_match = match (discord_user.token(), app_state.discord_bot.token()) {
        (Some(a), Some(b)) => secure_compare(a, b),
        _ => false,
    };
    if !tokens_match {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });
    let Verify { roles, user_links } = &mut guild.verify;

    for role in roles.iter() {
        for (user_id, links) in &*user_links {
            if link_arr_match(links, &role.pattern) {
                let r = add_guild_member_role(
                    guild_id,
                    *user_id,
                    role.role_id,
                    &app_state.discord_bot,
                )
                .await;
                if !(r.is_err() && r.err().unwrap().eq(&StatusCode::NOT_FOUND)) {
                    r?;
                }
            } else {
                let r = remove_guild_member_role(
                    guild_id,
                    *user_id,
                    role.role_id,
                    &app_state.discord_bot,
                )
                .await;
                if !(r.is_err() && r.err().unwrap().eq(&StatusCode::NOT_FOUND)) {
                    r?;
                }
            }
        }
    }
    // Recompute every role's `members` from `user_links` in one place so
    // recon uses exactly the same counting logic as put_roles_id/add/remove,
    // instead of a manual counter that can disagree with them.
    guild.verify.recompute_role_members();
    guild.save(&app_state.pg_pool).await?;

    Ok(StatusCode::NO_CONTENT)
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

    /// Regression test for issue #26: put_roles_id used to `.unwrap()` the
    /// result of `find` when re-reading back the just-pushed role, which
    /// panics (and surfaces as a 500) if the role isn't present. This
    /// asserts the handler's lookup path now converts a missing role into
    /// a NOT_FOUND result instead of panicking.
    #[test]
    fn missing_role_maps_to_404_instead_of_panicking() {
        let roles = vec![sample_role(1)];

        let result: Result<&VerifyRole, StatusCode> =
            find_role(&roles, Id::new(404)).ok_or(StatusCode::NOT_FOUND);

        assert_eq!(result.err(), Some(StatusCode::NOT_FOUND));
    }

    #[test]
    fn invalid_pattern_is_rejected_at_creation_time() {
        // `(` is not a valid regex — this must be rejected up front with a
        // clean 400 instead of being persisted and panicking later.
        assert_eq!(
            validate_verify_pattern("("),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn valid_pattern_is_accepted() {
        assert_eq!(validate_verify_pattern(r"^https://example\.com/.*$"), Ok(()));
    }
}
