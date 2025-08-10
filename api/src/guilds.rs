use axum::extract::Path;
use axum::Json;
use axum::routing::get;
use serde_json::{json, Value};

pub mod verify;

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/", get(get_guilds))
        .route("/{guild_id}", get(get_guilds_id))
        .nest("/{guild_id}/verify", verify::router())
}

async fn get_guilds() -> Json<Value> {
    Json(json!([{ "guild_id": "1" }, { "guild_id": "2" }]))
}

async fn get_guilds_id(Path(guild_id): Path<String>) -> Json<Value> {
    Json(json!({ "guild_id": guild_id }))
}
