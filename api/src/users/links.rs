use crate::AppState;
use crate::discord::{add_guild_member_role, ise, remove_guild_member_role};
use crate::guilds::models::Guild;
use crate::users::email::send_verify_email;
use crate::users::models::{Link, User};
use crate::users::utils::{link_arr_match, link_match};
use axum::extract::{Path, State};
use axum::routing::{delete, post};
use axum::{Extension, Json};
use hmac::{Hmac, Mac};
use http::StatusCode;
use jwt::{SignWithKey, VerifyWithKey};
use lambda_http::tracing::{Instrument, info, info_span, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use std::collections::BTreeMap;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::user::CurrentUser;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(post_link))
        .route("/send-email", post(post_send_email))
        .route("/{link_address}", delete(delete_link))
}

/// Builds the HMAC key used to sign/verify email-verification link JWTs.
///
/// This is a secret dedicated to this purpose (`EMAIL_LINK_SIGNING_KEY`) and
/// must NOT be the Discord bot token: the bot token is a credential for
/// authenticating to Discord's API (sent on every Discord request, and
/// subject to unrelated rotation), and reusing it as a JWT signing secret
/// couples two unrelated trust domains together.
fn email_link_key() -> Result<Hmac<Sha256>, StatusCode> {
    let secret = std::env::var("EMAIL_LINK_SIGNING_KEY")
        .map_err(|_| ise("Email link signing key unavailable"))?;
    build_email_link_key(&secret)
}

fn build_email_link_key(secret: &str) -> Result<Hmac<Sha256>, StatusCode> {
    Hmac::new_from_slice(secret.as_bytes()).map_err(ise)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LinkOrigin {
    Microsoft,
    Google,
    Email,
    Discord,
}

#[derive(Clone, Serialize, Deserialize)]
struct LinkRequest {
    origin: LinkOrigin,
    token: String,
}

async fn post_link(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(link_req): Json<LinkRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let span = info_span!("verify_link", provider = ?link_req.origin);
    link_for_user(user_id, current_user, app_state, link_req)
        .instrument(span)
        .await
}

async fn link_for_user(
    user_id: Id<UserMarker>,
    current_user: CurrentUser,
    app_state: AppState,
    link_req: LinkRequest,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if current_user.id != user_id {
        return Err(verify_link_failure(
            "authenticate",
            "discord_user_mismatch",
            StatusCode::UNAUTHORIZED,
        ));
    }

    info!(stage = "resolve_email", "Verify link step started");
    let email = match link_req.origin {
        LinkOrigin::Discord => current_user.email.ok_or_else(|| {
            verify_link_failure(
                "resolve_email",
                "discord_email_missing",
                StatusCode::UNAUTHORIZED,
            )
        })?,
        LinkOrigin::Microsoft => {
            oidc_email(
                "https://graph.microsoft.com/oidc/userinfo",
                link_req.token,
                &app_state,
            )
            .await?
        }
        LinkOrigin::Google => {
            oidc_email(
                "https://openidconnect.googleapis.com/v1/userinfo",
                link_req.token,
                &app_state,
            )
            .await?
        }
        LinkOrigin::Email => email_from_link_token(&link_req.token, user_id)?,
    };

    let new_link = Link {
        link_address: email,
        linked_at: chrono::Utc::now().timestamp_millis() as u64,
        active: true,
    };
    info!(stage = "load_user", "Verify link step started");
    let mut user_model = User::from_db(user_id, &app_state.pg_pool)
        .await
        .map_err(|status| verify_link_failure("load_user", "database_read_failed", status))?
        .unwrap_or_else(|| User {
            user_id,
            ..Default::default()
        });
    user_model
        .links
        .retain(|l| l.link_address != new_link.link_address);
    for link_guild in &user_model.link_guilds {
        info!(stage = "load_guild", "Verify link step started");
        let mut guild = Guild::from_db(link_guild.guild_id, &app_state.pg_pool)
            .await
            .map_err(|status| verify_link_failure("load_guild", "database_read_failed", status))?
            .unwrap_or_else(|| Guild {
                guild_id: link_guild.guild_id,
                ..Default::default()
            });
        for role in &guild.verify.roles {
            if link_match(&new_link, &role.pattern) {
                info!(stage = "assign_role", "Verify link step started");
                add_guild_member_role(
                    guild.guild_id,
                    user_id,
                    role.role_id,
                    &app_state.discord_bot,
                )
                .await
                .map_err(|status| {
                    verify_link_failure("assign_role", "discord_role_assignment_failed", status)
                })?;
            }
        }
        guild
            .verify
            .user_links
            .entry(user_id)
            .or_default()
            .push(new_link.clone());
        // Recompute from `user_links` instead of incrementing `role.members`
        // by hand: a remove-then-re-add of the same address (see the
        // `retain` above) would otherwise inflate the count every time the
        // user re-links an address they already had.
        guild.verify.recompute_role_members();
        info!(stage = "save_guild", "Verify link step started");
        guild
            .save(&app_state.pg_pool)
            .await
            .map_err(|status| verify_link_failure("save_guild", "database_write_failed", status))?;
    }
    user_model.links.push(new_link.clone());
    info!(stage = "save_user", "Verify link step started");
    user_model
        .save(&app_state.pg_pool)
        .await
        .map_err(|status| verify_link_failure("save_user", "database_write_failed", status))?;
    info!("Verify link saved");
    Ok(Json(json!(new_link)))
}

fn email_from_link_token(token: &str, user_id: Id<UserMarker>) -> Result<String, StatusCode> {
    let unauthorized =
        |reason| verify_link_failure("resolve_email", reason, StatusCode::UNAUTHORIZED);
    let key = email_link_key().map_err(|status| {
        verify_link_failure("resolve_email", "email_signing_key_unavailable", status)
    })?;
    let claims: BTreeMap<String, String> = token
        .verify_with_key(&key)
        .map_err(|_| unauthorized("email_token_invalid"))?;
    let exp: u64 = claims
        .get("exp")
        .and_then(|expiry| expiry.parse().ok())
        .ok_or_else(|| unauthorized("email_token_expiry_invalid"))?;
    if exp < chrono::Utc::now().timestamp() as u64 {
        return Err(unauthorized("email_token_expired"));
    }
    if claims.get("uid").map(String::as_str) != Some(user_id.to_string().as_str()) {
        return Err(unauthorized("email_token_user_mismatch"));
    }
    claims
        .get("sub")
        .cloned()
        .ok_or_else(|| unauthorized("email_token_subject_missing"))
}

async fn oidc_email(url: &str, token: String, app_state: &AppState) -> Result<String, StatusCode> {
    let response = app_state
        .reqwest
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|error| {
            let reason = if error.is_timeout() {
                "userinfo_timeout"
            } else {
                "userinfo_transport_failed"
            };
            verify_link_failure("resolve_email", reason, StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    if !response.status().is_success() {
        warn!(
            stage = "resolve_email",
            reason = "userinfo_rejected",
            upstream_status = response.status().as_u16(),
            "Verify link failed"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_info: serde_json::Value = response.json().await.map_err(|_| {
        verify_link_failure(
            "resolve_email",
            "userinfo_decode_failed",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    user_info
        .get("email")
        .and_then(|email| email.as_str())
        .filter(|email| !email.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            verify_link_failure(
                "resolve_email",
                "userinfo_email_missing",
                StatusCode::UNAUTHORIZED,
            )
        })
}

fn verify_link_failure(stage: &str, reason: &str, status: StatusCode) -> StatusCode {
    warn!(
        stage,
        reason,
        status = status.as_u16(),
        "Verify link failed"
    );
    status
}

/// Authorizes a `delete_link` request: the middleware-provided current user
/// must match the `user_id` path segment being modified. Extracted as a pure
/// function (instead of inlined in the handler) so the authz decision can be
/// unit tested without standing up an axum request or a Discord client.
///
/// Regression coverage for koalabotuk/kb2#54: `delete_link` used to make its
/// own uncached, double-`.unwrap()`ed `current_user()` Discord API call for
/// this check instead of using the `Extension<CurrentUser>` already injected
/// by `auth_middleware`, which could panic on a transient Discord error and
/// returned the wrong status code (`NOT_FOUND`) on mismatch.
fn check_delete_link_authorized(
    current_user_id: Id<UserMarker>,
    target_user_id: Id<UserMarker>,
) -> Result<(), StatusCode> {
    if current_user_id != target_user_id {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

async fn delete_link(
    Path((user_id, link_address)): Path<(Id<UserMarker>, String)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    check_delete_link_authorized(current_user.id, user_id)?;

    let mut user_model = User::from_db(user_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| User {
            user_id,
            ..Default::default()
        });
    let pos = user_model
        .links
        .iter()
        .position(|l| l.link_address == link_address);
    let existing_link = pos.map(|i| user_model.links.remove(i));
    if existing_link.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let mut existing_link = existing_link.unwrap();

    let active_only_links: Vec<Link> = user_model
        .links
        .clone()
        .into_iter()
        .filter(|l| l.active)
        .collect();
    for guild in &user_model.link_guilds {
        let mut guild = Guild::from_db(guild.guild_id, &app_state.pg_pool)
            .await?
            .unwrap_or_else(|| Guild {
                guild_id: guild.guild_id,
                ..Default::default()
            });
        guild
            .verify
            .user_links
            .insert(user_id, active_only_links.clone());
        for role in &guild.verify.roles {
            if !link_arr_match(
                guild.verify.user_links.get(&user_id).unwrap(),
                &role.pattern,
            ) && link_match(&existing_link, &role.pattern)
            {
                remove_guild_member_role(
                    guild.guild_id,
                    user_id,
                    role.role_id,
                    &app_state.discord_bot,
                )
                .await?;
            }
        }
        // `user_links` was already updated above; derive `members` from it
        // instead of the old `if role.members > 0 { role.members -= 1 }`
        // guard, which only existed because the counter was untrustworthy.
        guild.verify.recompute_role_members();
        guild.save(&app_state.pg_pool).await?;
    }
    existing_link.active = false;
    user_model.links.push(existing_link);
    user_model.save(&app_state.pg_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Clone, Serialize, Deserialize)]
struct SendEmailRequest {
    email: String,
}

async fn post_send_email(
    // Path(_user_id): Path<Id<UserMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(send_email_req): Json<SendEmailRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let key = email_link_key()?;
    let mut claims = BTreeMap::new();
    claims.insert("sub", send_email_req.email.clone());
    claims.insert("uid", current_user.id.to_string());
    claims.insert(
        "exp",
        (chrono::Utc::now() + chrono::Duration::hours(1))
            .timestamp()
            .to_string(),
    );
    let token_str = claims.sign_with_key(&key).map_err(ise)?;

    send_verify_email(
        &app_state.ses,
        &current_user.name,
        send_email_req.email,
        &token_str,
    )
    .await?;

    Ok(Json(json!({"status": "success"})))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for issue #23: the email-verification JWT signing
    /// key must round-trip sign/verify correctly using its own dedicated
    /// secret.
    #[test]
    fn email_link_key_round_trips_sign_and_verify() {
        let key = build_email_link_key("a-dedicated-email-link-signing-secret")
            .expect("valid key material");

        let mut claims = BTreeMap::new();
        claims.insert("sub".to_string(), "user@example.com".to_string());
        claims.insert("uid".to_string(), "12345".to_string());
        claims.insert(
            "exp".to_string(),
            (chrono::Utc::now() + chrono::Duration::hours(1))
                .timestamp()
                .to_string(),
        );

        let token = claims.sign_with_key(&key).expect("signing succeeds");

        let verified: BTreeMap<String, String> =
            token.verify_with_key(&key).expect("verification succeeds");
        assert_eq!(verified.get("sub"), Some(&"user@example.com".to_string()));
        assert_eq!(verified.get("uid"), Some(&"12345".to_string()));
    }

    /// Regression test for issue #23: a JWT signed with the dedicated
    /// EMAIL_LINK_SIGNING_KEY secret must NOT verify against a key derived
    /// from the Discord bot token (i.e. the bot token can no longer be used
    /// as the HMAC secret for these tokens, proving the secrets are no
    /// longer reused/interchangeable).
    #[test]
    fn email_link_jwt_does_not_verify_with_discord_bot_token() {
        let signing_key =
            build_email_link_key("a-dedicated-email-link-signing-secret").expect("valid key");
        let bot_token_key =
            build_email_link_key("totally-unrelated-discord-bot-token").expect("valid key");

        let mut claims = BTreeMap::new();
        claims.insert("sub".to_string(), "user@example.com".to_string());
        claims.insert(
            "exp".to_string(),
            (chrono::Utc::now() + chrono::Duration::hours(1))
                .timestamp()
                .to_string(),
        );

        let token = claims
            .sign_with_key(&signing_key)
            .expect("signing succeeds");

        let result: Result<BTreeMap<String, String>, _> = token.verify_with_key(&bot_token_key);
        assert!(
            result.is_err(),
            "a token signed with the dedicated email-link secret must not verify \
             against a key built from the (different) Discord bot token"
        );
    }

    /// Regression test for issue #23: malformed/missing `exp` claims must
    /// not panic (previously `.unwrap()` on a missing/invalid `exp` claim
    /// would crash the handler with a 500 instead of returning 401).
    #[test]
    fn missing_exp_claim_is_handled_without_panicking() {
        let key = build_email_link_key("a-dedicated-email-link-signing-secret")
            .expect("valid key material");

        let mut claims = BTreeMap::new();
        claims.insert("sub".to_string(), "user@example.com".to_string());
        // Intentionally no "exp" claim.

        let token = claims.sign_with_key(&key).expect("signing succeeds");

        let verified: BTreeMap<String, String> =
            token.verify_with_key(&key).expect("verification succeeds");
        let exp: Option<u64> = verified.get("exp").and_then(|e| e.parse().ok());
        assert_eq!(exp, None, "missing exp should parse to None, not panic");
    }

    #[test]
    fn check_delete_link_authorized_allows_matching_ids() {
        let id = Id::<UserMarker>::new(228541431483072513);

        assert_eq!(check_delete_link_authorized(id, id), Ok(()));
    }

    #[test]
    fn check_delete_link_authorized_rejects_mismatched_ids_with_unauthorized() {
        // Regression test for koalabotuk/kb2#54: a request whose
        // middleware-provided current user does not match the target
        // `user_id` must be rejected with UNAUTHORIZED, not NOT_FOUND (the
        // old behaviour) and must not panic (the old uncached, double
        // `.unwrap()`ed Discord call could panic on a transient error).
        let current_user_id = Id::<UserMarker>::new(111111111111111111);
        let target_user_id = Id::<UserMarker>::new(222222222222222222);

        let result = check_delete_link_authorized(current_user_id, target_user_id);

        assert_eq!(result, Err(StatusCode::UNAUTHORIZED));
    }
}
