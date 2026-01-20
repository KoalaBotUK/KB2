pub mod email;
mod link_guilds;
mod links;
pub mod models;
pub mod utils;

use crate::AppState;
use crate::users::models::User;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Extension, Json};
use http::StatusCode;
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::user::CurrentUser;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(get_users))
        .route("/@me", get(get_users_me))
        .route("/{user_id}", get(get_users_id))
        .nest("/{user_id}/links", links::router())
        .nest("/{user_id}/link_guilds", link_guilds::router())
        .layer(CorsLayer::permissive())
}

async fn get_users() -> Json<Value> {
    todo!()
}

async fn get_users_me(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    get_users_id(
        Path(current_user.id),
        Extension(current_user),
        State(app_state),
    )
    .await
}

async fn get_users_id(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // Authorize
    if current_user.id != user_id {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Fetch user from DynamoDB
    let result = match User::from_db(user_id, &app_state.pg_pool).await {
        Some(user) => user,
        None => {
            let u = User {
                user_id,
                ..Default::default()
            };
            u.save(&app_state.pg_pool).await;
            u
        }
    };
    Ok(Json(json!(result)))
}
