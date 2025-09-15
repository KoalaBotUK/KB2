use crate::AppState;
use axum::Json;
use axum::routing::get;
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new().route("/", get(get_verify))
        .layer(CorsLayer::permissive())
}

async fn get_verify() -> Json<Value> {
    Json(json!({ "msg": "I am GET /verify" }))
}
