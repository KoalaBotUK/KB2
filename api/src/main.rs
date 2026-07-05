use common::dsql::establish_connection;
use aws_config::BehaviorVersion;
use axum::body::Body;
use axum::{http::StatusCode, routing::get, Json, Router};
use http::{Method, Response};
use lambda_http::{run, tracing, Error};
use serde_json::{json, Value};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod discord;
mod guilds;
mod interactions;
mod meta;
mod middleware;
mod users;
mod utils;
mod audit;

#[derive(Clone)]
pub struct AppState {
    scheduler: aws_sdk_scheduler::Client,
    ses: aws_sdk_sesv2::Client,
    sqs: aws_sdk_sqs::Client,
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
        sqs: aws_sdk_sqs::Client::new(&config),
        pg_pool: pool,
        discord_bot,
        reqwest: Arc::new(reqwest::Client::new()),
    };

    // Only allow requests from the first-party UI origin(s). This API trusts the
    // `Authorization` header for auth, so a permissive `*` origin would let any
    // website script requests against it using a token it has obtained.
    let cors = CorsLayer::new()
        .allow_origin(
            parse_cors_allowed_origin(std::env::var("CORS_ALLOWED_ORIGIN").ok().as_deref())
                .expect("invalid CORS configuration"),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([http::header::AUTHORIZATION, http::header::CONTENT_TYPE]);

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
        .layer(cors);

    if std::env::var("RUN_LOCAL").is_err() {
        run(app).await
    } else {
        run_local(app).await
    }
}

/// Parses the `CORS_ALLOWED_ORIGIN` environment variable into a header value
/// suitable for `CorsLayer::allow_origin`.
///
/// This is extracted out of `main()` so the parsing/validation logic can be
/// unit tested in isolation. Whether the resulting `CorsLayer` actually
/// enforces the restriction on real requests (i.e. that browsers sending an
/// `Origin` header other than the configured one are rejected) is not
/// unit-testable here since it depends on `tower_http`'s CORS middleware
/// behaviour and axum's request handling; that should be covered by an
/// integration/CI-level test that exercises the running service.
fn parse_cors_allowed_origin(origin: Option<&str>) -> Result<http::HeaderValue, String> {
    let origin = origin.ok_or_else(|| "env variable `CORS_ALLOWED_ORIGIN` should be set".to_string())?;

    origin.parse::<http::HeaderValue>().map_err(|_| {
        "CORS_ALLOWED_ORIGIN must be a valid header value (e.g. https://koalabot.uk)".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cors_allowed_origin_accepts_valid_origin() {
        let result = parse_cors_allowed_origin(Some("https://koalabot.uk"));

        assert_eq!(result.expect("should parse").to_str().unwrap(), "https://koalabot.uk");
    }

    #[test]
    fn parse_cors_allowed_origin_rejects_missing_value() {
        let result = parse_cors_allowed_origin(None);

        assert_eq!(
            result.unwrap_err(),
            "env variable `CORS_ALLOWED_ORIGIN` should be set"
        );
    }

    #[test]
    fn parse_cors_allowed_origin_rejects_invalid_header_value() {
        // Carriage-return/newline characters are not permitted in header
        // values (they would allow header/response splitting), so this must
        // be rejected rather than silently accepted.
        let result = parse_cors_allowed_origin(Some("https://koalabot.uk\r\nEvil: 1"));

        assert_eq!(
            result.unwrap_err(),
            "CORS_ALLOWED_ORIGIN must be a valid header value (e.g. https://koalabot.uk)"
        );
    }
}
