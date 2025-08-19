use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use axum::body::Body;
use axum::{Json, Router, http::StatusCode, routing::get};
use http::Response;
use lambda_http::{Error, run, tracing};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod dynamo;
mod guilds;
mod interactions;
mod middleware;
mod users;

#[derive(Clone)]
pub struct AppState {
    dynamo: Client,
    discord_bot: Arc<twilight_http::Client>,
    reqwest: Arc<reqwest::Client>,
}

#[derive(Deserialize, Serialize)]
struct Params {
    first: Option<String>,
    second: Option<String>,
}

async fn health_check() -> (StatusCode, Json<Value>) {
    let health = true;
    match health {
        true => (StatusCode::OK, Json(json!({ "status": "OK" }))),
        false => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "ERROR" })),
        ),
    }
}

async fn get_bot_redirect() -> Response<Body> {
    let redirect_url = "https://discord.com/oauth2/authorize?client_id=1014995724888444998&permissions=0&integration_type=0&scope=bot";
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", redirect_url)
        .body(Body::empty())
        .unwrap()
}

async fn create_dynamodb_client() -> Client {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    Client::new(&config)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let dynamo = create_dynamodb_client().await;
    let discord_bot = Arc::new(twilight_http::Client::new(
        std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set"),
    ));

    let app_state = AppState {
        dynamo,
        discord_bot,
        reqwest: Arc::new(reqwest::Client::new()),
    };

    let app = Router::new()
        .nest("/users", users::router())
        .nest("/guilds", guilds::router())
        .layer(CorsLayer::permissive())
        .route_layer(axum::middleware::from_fn(middleware::auth_middleware))
        .nest("/interactions", interactions::router())
        .route_layer(axum::middleware::from_fn(middleware::log_middleware))
        .with_state(app_state)
        .route("/health", get(health_check))
        .route("/bot", get(get_bot_redirect));

    run(app).await
}
