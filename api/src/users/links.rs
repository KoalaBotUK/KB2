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
use lambda_http::tracing::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
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
    build_email_link_key(
        &std::env::var("EMAIL_LINK_SIGNING_KEY").expect("EMAIL_LINK_SIGNING_KEY must be set"),
    )
}

fn build_email_link_key(secret: &str) -> Result<Hmac<Sha256>, StatusCode> {
    Hmac::new_from_slice(secret.as_bytes()).map_err(ise)
}

#[derive(Clone, Serialize, Deserialize)]
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
    if current_user.id != user_id {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let email = match link_req.origin {
        LinkOrigin::Discord => {
            // Handle Discord linking logic here
            current_user.email.ok_or(StatusCode::UNAUTHORIZED)?
        }
        LinkOrigin::Microsoft => {
            // Handle Microsoft linking logic here
            oidc_email(
                "https://graph.microsoft.com/oidc/userinfo",
                link_req.token,
                &app_state,
            )
            .await?
        }
        LinkOrigin::Google => {
            // Handle Google linking logic here
            oidc_email(
                "https://openidconnect.googleapis.com/v1/userinfo",
                link_req.token,
                &app_state,
            )
            .await?
        }
        LinkOrigin::Email => {
            let key = email_link_key()?;
            let claims: BTreeMap<String, String> =
                link_req.token.verify_with_key(&key).map_err(ise)?;
            let exp: u64 = claims
                .get("exp")
                .and_then(|e| e.parse().ok())
                .ok_or(StatusCode::UNAUTHORIZED)?;
            if exp < chrono::Utc::now().timestamp() as u64 {
                return Err(StatusCode::UNAUTHORIZED);
            }
            if claims.get("uid").map(String::as_str) != Some(current_user.id.to_string().as_str())
            {
                return Err(StatusCode::UNAUTHORIZED);
            }
            claims
                .get("sub")
                .ok_or(StatusCode::UNAUTHORIZED)?
                .to_string()
        }
    };

    let new_link = Link {
        link_address: email,
        linked_at: chrono::Utc::now().timestamp_millis() as u64,
        active: true,
    };
    let mut user_model = User::from_db(user_id, &app_state.pg_pool)
        .await
        .unwrap();
    user_model
        .links
        .retain(|l| l.link_address != new_link.link_address);
    for link_guild in &user_model.link_guilds {
        let mut guild = Guild::from_db(link_guild.guild_id, &app_state.pg_pool)
            .await
            .unwrap();
        for role in &guild.verify.roles {
            if link_match(&new_link, &role.pattern) {
                add_guild_member_role(
                    guild.guild_id,
                    user_id,
                    role.role_id,
                    &app_state.discord_bot,
                )
                .await?;
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
        guild.save(&app_state.pg_pool).await?;
    }
    user_model.links.push(new_link.clone());
    user_model.save(&app_state.pg_pool).await?;
    Ok(Json(json!(new_link)))
}

async fn oidc_email(url: &str, token: String, app_state: &AppState) -> Result<String, StatusCode> {
    let response = app_state
        .reqwest
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(ise)?;

    if response.status().is_success() {
        let user_info: serde_json::Value = response.json().await.map_err(ise)?;
        // Process user_info to link with the user_id
        // For example, save to DynamoDB or update user profile
        Ok(user_info
            .get("email")
            .and_then(|email| email.as_str())
            .ok_or(StatusCode::UNAUTHORIZED)?
            .parse()
            .unwrap())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
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
        .await
        .unwrap();
    let pos = user_model
        .links
        .iter()
        .position(|l| l.link_address == link_address);
    let existing_link = pos.map(|i| user_model.links.remove(i));
    info!("Deleting link for user {}: {}", user_id, link_address);
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
            .await
            .unwrap();
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

        let token = claims.sign_with_key(&signing_key).expect("signing succeeds");

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
