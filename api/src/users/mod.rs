mod models;
mod links;

use crate::users::models::User;
use crate::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Extension, Json};
use http::StatusCode;
use serde_json::{json, Value};
use std::sync::Arc;
use serde::Deserialize;
use twilight_http::Client as DiscordClient;
use twilight_model::id::marker::UserMarker;
use twilight_model::id::Id;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(get_users))
        .route("/{user_id}", get(get_users_id).put(put_users_id))
        .nest("/{user_id}/links", links::router())
}

async fn get_users(
) -> Json<Value> {
    todo!()
}

async fn get_users_id(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(discord_user): Extension<Arc<DiscordClient>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // Authorize
    let logged_in_user = discord_user.current_user().await.unwrap().model().await.unwrap();
    if logged_in_user.id.ne(&user_id) {
        return Err(StatusCode::NOT_FOUND)
    }

    // Fetch user from DynamoDB
    let result = User::from_db(&user_id.to_string(), &app_state.dynamo).await
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(json!(result)))
}

#[derive(Deserialize)]
struct PutUserRequest {
    linked_guild_ids: Vec<String>,
}

async fn put_users_id(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(discord_user): Extension<Arc<DiscordClient>>,
    State(app_state): State<AppState>,
    Json(user_req): Json<PutUserRequest>
) -> Result<Json<Value>, StatusCode> {
    let logged_in_user = discord_user.current_user().await.unwrap().model().await.unwrap();
    if logged_in_user.id.ne(&user_id) {
        return Err(StatusCode::NOT_FOUND)
    }
    // Write user to DynamoDB
    let mut new_user = User::from_db(&user_id.to_string(), &app_state.dynamo).await
        .unwrap_or_else(|| User {
            user_id: user_id.to_string(),
            links: vec![],
            linked_guild_ids: vec![],
        });

    new_user.linked_guild_ids = user_req.linked_guild_ids;

    new_user.save(&app_state.dynamo).await;

    Ok(Json(json!(new_user)))

}