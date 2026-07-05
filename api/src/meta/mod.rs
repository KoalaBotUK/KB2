use crate::AppState;
use axum::Router;

mod guilds;
mod users;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/guilds", guilds::router())
        .nest("/users", users::router())
}
