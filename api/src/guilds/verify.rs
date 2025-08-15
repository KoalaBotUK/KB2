use axum::Json;
use axum::routing::get;
use serde_json::{json, Value};
use crate::AppState;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(get_verify))
}

async fn get_verify() -> Json<Value> {
    Json(json!({ "msg": "I am GET /verify" }))
}