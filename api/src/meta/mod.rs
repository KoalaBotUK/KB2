use std::sync::Arc;
use axum::Router;
use lambda_http::tracing::info;
use tower_http::cors::CorsLayer;
use twilight_http::Client;
use crate::AppState;

mod guilds;
mod users;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/guilds", guilds::router())
        .nest("/users", users::router())
        .layer(CorsLayer::permissive())
}

pub fn setup(discord_bot: Arc<Client>) {
    info!("Spawning meta cache refresh task");
    guilds::setup(discord_bot);
}