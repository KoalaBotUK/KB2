use axum::extract::{Path, State};
use axum::{Extension, Json, Router};
use axum::routing::get;
use http::StatusCode;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::user::CurrentUser;
use crate::AppState;
use crate::discord::get_user;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/@me", get(get_meta_users_me))
        .route("/{user_id}", get(get_meta_users_id))
        .layer(CorsLayer::permissive())
}

pub async fn get_meta_users_me(Extension(current_user): Extension<CurrentUser>,
                               State(state): State<AppState>)
                               -> Result<Json<Value>, StatusCode> {
    get_meta_users_id(Path(current_user.id), Extension(current_user), State(state)).await
}

pub async fn get_meta_users_id(Path(user_id): Path<Id<UserMarker>>, 
                               Extension(current_user): Extension<CurrentUser>,
                               State(state): State<AppState>) 
    -> Result<Json<Value>, StatusCode> {
    if current_user.id != user_id {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    Ok(Json(json!(get_user(user_id, &state.discord_bot).await?)))
}