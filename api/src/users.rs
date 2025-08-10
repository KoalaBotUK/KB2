use axum::extract::Path;
use axum::{Extension, Json};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub emails: Vec<String>,
    pub username: String,
    pub first_name: String,
    pub last_name: String
}


pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/", get(get_users))
        .route("/{user_id}", get(get_users_id))
}

async fn get_users(
    Extension(auth_user): Extension<User>
) -> Json<Value> {
    Json(json!(auth_user))
}

async fn get_users_id(
    Path(user_id): Path<String>,
    Extension(auth_user): Extension<User>
) -> Json<Value> {
    let mut new_user = auth_user.clone();
    new_user.user_id = user_id;
    Json(json!(new_user))
}