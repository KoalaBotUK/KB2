use aws_config::BehaviorVersion;
use axum::body::Body;
use axum::{http::StatusCode, routing::get, Json, Router};
use http::Response;
use lambda_http::{run, tracing, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use lambda_http::tracing::info;
use tower_http::cors::CorsLayer;

mod dynamo;
mod guilds;
mod interactions;
mod middleware;
mod users;
mod utils;
mod discord;
mod meta;

#[derive(Clone)]
pub struct AppState {
    dynamo: aws_sdk_dynamodb::Client,
    scheduler: aws_sdk_scheduler::Client,
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

async fn run_local(app: Router) -> Result<(), Error> {
    let router = Router::new().nest("/lambda-url/api", app)
        .route_layer(axum::middleware::from_fn(middleware::mock_ctx_middleware));
    Ok(axum::serve(tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap(), router).await.unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let discord_bot = Arc::new(
        twilight_http::Client::builder()
            .token(std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set"))
        .build()
    );

    let app_state = AppState {
        dynamo: aws_sdk_dynamodb::Client::new(&config),
        scheduler: aws_sdk_scheduler::Client::new(&config),
        discord_bot,
        reqwest: Arc::new(reqwest::Client::new()),
    };
    
    // guilds::tasks::update_guilds(&app_state.discord_bot, &app_state.dynamo).await;
    setup(app_state.discord_bot.clone());
    let app = Router::new()
        .nest("/users", users::router())
        .nest("/guilds", guilds::router())
        .nest("/meta", meta::router())
        .route_layer(axum::middleware::from_fn(middleware::auth_middleware))
        .nest("/interactions", interactions::router())
        .route_layer(axum::middleware::from_fn(middleware::log_middleware))
        .with_state(app_state)
        .route("/health", get(health_check))
        .route("/bot", get(get_bot_redirect))
        .layer(CorsLayer::permissive());

    if std::env::var("RUN_LOCAL").is_err() {
        run(app).await
    } else {
        run_local(app).await
    }
}


fn setup(discord_bot: Arc<twilight_http::Client>) {
    meta::setup(discord_bot);
}