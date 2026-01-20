use crate::dsql::establish_connection;
use aws_config::BehaviorVersion;
use axum::body::Body;
use axum::{Json, Router, http::StatusCode, routing::get};
use http::Response;
use lambda_http::{Error, run, tracing};
use serde_json::{Value, json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod discord;
mod dsql;
mod guilds;
mod interactions;
mod meta;
mod middleware;
mod users;
mod utils;

#[derive(Clone)]
pub struct AppState {
    scheduler: aws_sdk_scheduler::Client,
    ses: aws_sdk_sesv2::Client,
    pg_pool: Pool<Postgres>,
    discord_bot: Arc<twilight_http::Client>,
    reqwest: Arc<reqwest::Client>,
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
    let router = Router::new()
        .nest("/lambda-url/api", app)
        .route_layer(axum::middleware::from_fn(middleware::mock_ctx_middleware));
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap(),
        router,
    )
    .await
    .unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    let pool = establish_connection(
        std::env::var("DSQL_USER").expect("env variable `DSQL_USER` should be set"),
        std::env::var("DSQL_ENDPOINT").expect("env variable `DSQL_ENDPOINT` should be set"),
        config.region().unwrap(),
    )
    .await?;

    let discord_bot = Arc::new(
        twilight_http::Client::builder()
            .token(std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set"))
            .build(),
    );

    let app_state = AppState {
        scheduler: aws_sdk_scheduler::Client::new(&config),
        ses: aws_sdk_sesv2::Client::new(&config),
        pg_pool: pool,
        discord_bot,
        reqwest: Arc::new(reqwest::Client::new()),
    };

    // guilds::tasks::update_guilds(&app_state.discord_bot, &app_state.pg_pool).await;
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
