use crate::AppState;
use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use serde_json::json;
use twilight_model::id::marker::UserMarker;
use twilight_model::id::Id;
use crate::users::models::{Link, User};

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(post_link))
}

#[derive(Clone, Serialize, Deserialize)]
enum LinkOrigin {
    MICROSOFT,
    GOOGLE,
    EMAIL,
    DISCORD,
}

#[derive(Clone, Serialize, Deserialize)]
struct LinkRequest {
    origin: LinkOrigin,
    token: String,
}

async fn post_link(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
    Json(link_req): Json<LinkRequest>
) -> Result<Json<serde_json::Value>, http::StatusCode> {
    if user_id != discord_user.current_user().await.unwrap().model().await.unwrap().id {
        return Err(http::StatusCode::NOT_FOUND);
    }
    let email;
    match link_req.origin {
        LinkOrigin::DISCORD => {
            // Handle Discord linking logic here
            email = discord_user.current_user().await.map_err(|_| http::StatusCode::UNAUTHORIZED)?
                .model().await.map_err(|_| http::StatusCode::UNAUTHORIZED)?.email.unwrap();
        },
        LinkOrigin::MICROSOFT => {
            // Handle Microsoft linking logic here
            email = oidc_email("https://graph.microsoft.com/oidc/userinfo", link_req.token, &app_state).await?;
        },
        LinkOrigin::GOOGLE => {
            // Handle Google linking logic here
            email = oidc_email("https://openidconnect.googleapis.com/v1/userinfo", link_req.token, &app_state).await?;
        },
        LinkOrigin::EMAIL => {
            todo!();
        },
    }

    let new_link = Link{link_address: email, linked_at: chrono::Utc::now().timestamp_millis() as u64};
    let json_resp = Json(json!(new_link));
    let mut user_model = User::from_db(&user_id.to_string(), &app_state.dynamo).await.unwrap();
    user_model.links.retain(|l| l.link_address != new_link.link_address);
    user_model.links.push(new_link);
    user_model.save(&app_state.dynamo).await;
    Ok(json_resp)
}

async fn oidc_email(
    url: &str,
    token: String,
    app_state: &AppState,
) -> Result<String, http::StatusCode> {
    let resp = app_state.reqwest.get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;
    match resp {
        Ok(response) => {
            if response.status().is_success() {
                let user_info: serde_json::Value = response.json().await.map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;
                // Process user_info to link with the user_id
                // For example, save to DynamoDB or update user profile
                Ok(user_info.get("email").and_then(|email| email.as_str()).ok_or(http::StatusCode::UNAUTHORIZED)?.parse().unwrap())
            } else {
                Err(http::StatusCode::UNAUTHORIZED)
            }
        },
        Err(_) => Err(http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}
