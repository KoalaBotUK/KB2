use crate::AppState;
use crate::users::models::{Link, User};
use axum::extract::{Path, State};
use axum::routing::{delete, post};
use axum::{Extension, Json};
use lambda_http::tracing::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::user::CurrentUser;
use crate::guilds::models::Guild;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(post_link))
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
) -> Result<Json<serde_json::Value>, http::StatusCode> {
    if current_user.id != user_id {
        return Err(http::StatusCode::UNAUTHORIZED);
    }
    
    let email = match link_req.origin {
        LinkOrigin::Discord => {
            // Handle Discord linking logic here
            current_user.email.ok_or(http::StatusCode::UNAUTHORIZED)?
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
            todo!();
        }
    };

    let new_link = Link {
        link_address: email,
        linked_at: chrono::Utc::now().timestamp_millis() as u64,
        active: true,
    };
    let json_resp = Json(json!(new_link));
    let mut user_model = User::from_db(&user_id.to_string(), &app_state.dynamo)
        .await
        .unwrap();
    user_model
        .links
        .retain(|l| l.link_address != new_link.link_address);
    for link_guild in &user_model.link_guilds {
        if !link_guild.enabled {
            continue;
        }
        let guild = Guild::from_db(link_guild.guild_id, &app_state.dynamo).await.unwrap();
        crate::guilds::tasks::assign_roles_guild_user_link(
            &new_link.link_address,
            user_id,
            &guild,
            &app_state.discord_bot,
        )
        .await;
    }
    user_model.links.push(new_link);
    user_model.save(&app_state.dynamo).await;
    Ok(json_resp)
}

async fn oidc_email(
    url: &str,
    token: String,
    app_state: &AppState,
) -> Result<String, http::StatusCode> {
    let resp = app_state
        .reqwest
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await;
    match resp {
        Ok(response) => {
            if response.status().is_success() {
                let user_info: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;
                // Process user_info to link with the user_id
                // For example, save to DynamoDB or update user profile
                Ok(user_info
                    .get("email")
                    .and_then(|email| email.as_str())
                    .ok_or(http::StatusCode::UNAUTHORIZED)?
                    .parse()
                    .unwrap())
            } else {
                Err(http::StatusCode::UNAUTHORIZED)
            }
        }
        Err(_) => Err(http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_link(
    Path((user_id, link_address)): Path<(Id<UserMarker>, String)>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, http::StatusCode> {
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
        return Err(http::StatusCode::NOT_FOUND);
    }

    let mut user_model = User::from_db(&user_id.to_string(), &app_state.dynamo)
        .await
        .unwrap();
    let existing_link = user_model.links.pop_if(|l| l.link_address == link_address);
    info!("Deleting link for user {}: {}", user_id, link_address);
    if existing_link.is_none() {
        return Err(http::StatusCode::NOT_FOUND);
    }
    let mut existing_link = existing_link.unwrap();
    existing_link.active = false;
    user_model.links.push(existing_link);
    user_model.save(&app_state.dynamo).await;

    Ok(Json(
        json!({"status": "success", "message": "Link deleted"}),
    ))
}
