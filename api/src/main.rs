//! This is an example function that leverages the Lambda Rust runtime HTTP support
//! and the [axum](https://docs.rs/axum/latest/axum/index.html) web framework.  The
//! runtime HTTP support is backed by the [tower::Service](https://docs.rs/tower-service/0.3.2/tower_service/trait.Service.html)
//! trait.  Axum's applications are also backed by the `tower::Service` trait.  That means
//! that it is fairly easy to build an Axum application and pass the resulting `Service`
//! implementation to the Lambda runtime to run as a Lambda function.  By using Axum instead
//! of a basic `tower::Service` you get web framework niceties like routing, request component
//! extraction, validation, etc.
use axum::{http::StatusCode, middleware, routing::get, Json, Router};
use lambda_http::{run, tracing, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env::set_var;
use std::sync::Arc;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use axum::body::Body;
use http::Response;
use tower_http::cors::CorsLayer;

mod auth;
mod dynamo;
mod users;
mod guilds;


#[derive(Clone)]
pub struct AppState {
    dynamo: Client,
    discord_bot: Arc<twilight_http::Client>,
}

#[derive(Deserialize, Serialize)]
struct Params {
    first: Option<String>,
    second: Option<String>,
}

/// Example on how to return status codes and data from an Axum function
async fn health_check() -> (StatusCode, Json<Value>) {
    let health = true;
    match health {
        true => (StatusCode::OK, Json(json!({ "status": "OK" }))),
        false => (StatusCode::INTERNAL_SERVER_ERROR,Json(json!({ "status": "ERROR" }))),
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
    };

    let app = Router::new()
        .nest("/users", users::router())
        .nest("/guilds", guilds::router())
        .layer(CorsLayer::permissive())
        .route_layer(middleware::from_fn(auth::auth_middleware))
        .with_state(app_state)
        .route("/health", get(health_check))
        .route("/bot", get(get_bot_redirect));

    run(app).await
}