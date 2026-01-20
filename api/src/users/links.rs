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
        .layer(CorsLayer::permissive())
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
            let key: Hmac<Sha256> = Hmac::new_from_slice(
                std::env::var("DISCORD_BOT_TOKEN")
                    .expect("DISCORD_BOT_TOKEN must be set")
                    .into_bytes()
                    .as_ref(),
            )
            .map_err(ise)?;
            let claims: BTreeMap<String, String> =
                link_req.token.verify_with_key(&key).map_err(ise)?;
            if claims.get("exp").unwrap().parse::<u64>().unwrap()
                < chrono::Utc::now().timestamp() as u64
            {
                return Err(StatusCode::UNAUTHORIZED);
            }
            claims.get("sub").unwrap().to_string()
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
        for role in &mut guild.verify.roles {
            if link_match(&new_link, &role.pattern) {
                add_guild_member_role(
                    guild.guild_id,
                    user_id,
                    role.role_id,
                    &app_state.discord_bot,
                )
                .await?;
                role.members += 1;
            }
        }
        guild
            .verify
            .user_links
            .get_mut(&user_id)
            .unwrap()
            .push(new_link.clone());
        guild.save(&app_state.pg_pool).await;
    }
    user_model.links.push(new_link.clone());
    user_model.save(&app_state.pg_pool).await;
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

async fn delete_link(
    Path((user_id, link_address)): Path<(Id<UserMarker>, String)>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    if user_id
        != discord_user
            .current_user()
            .await
            .unwrap()
            .model()
            .await
            .unwrap()
            .id
    {
        return Err(StatusCode::NOT_FOUND);
    }

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
        for role in &mut guild.verify.roles {
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
                if role.members > 0 {
                    role.members -= 1;
                }
            }
        }
        guild.save(&app_state.pg_pool).await;
    }
    existing_link.active = false;
    user_model.links.push(existing_link);
    user_model.save(&app_state.pg_pool).await;
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
    let key: Hmac<Sha256> = Hmac::new_from_slice(
        std::env::var("DISCORD_BOT_TOKEN")
            .expect("DISCORD_BOT_TOKEN must be set")
            .into_bytes()
            .as_ref(),
    )
    .map_err(ise)?;
    let mut claims = BTreeMap::new();
    claims.insert("sub", send_email_req.email.clone());
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
