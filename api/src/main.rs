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
use tower_http::cors::CorsLayer;

mod users;
mod guilds;
mod auth;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    unsafe { set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true"); }

    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let app = Router::new()
        .nest("/users", users::router())
        .nest("/guilds", guilds::router())
        .layer(CorsLayer::permissive())
        .route_layer(middleware::from_fn(auth::auth_middleware))
        .route("/health", get(health_check));

    run(app).await
}