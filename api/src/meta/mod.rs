use crate::AppState;
use axum::Router;
use lambda_http::tracing::info;
use std::sync::Arc;
use twilight_http::Client;

mod guilds;
mod users;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/guilds", guilds::router())
        .nest("/users", users::router())
}

pub fn setup(discord_bot: Arc<Client>) {
    info!("Spawning meta cache refresh task");
    guilds::setup(discord_bot);
}
