use axum::Json;
use axum::routing::get;
use serde_json::{json, Value};

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/", get(get_verify))
}

async fn get_verify() -> Json<Value> {
    Json(json!({ "msg": "I am GET /verify" }))
}