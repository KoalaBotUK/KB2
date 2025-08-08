use lambda_http::{
    http::Method, service_fn, tower::ServiceBuilder, tracing, Body, Error, Request, RequestExt, Response,
};
use tower_http::cors::{Any, CorsLayer};

mod verify;
use crate::verify::controller as verify_controller;


#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    // Define a layer to inject CORS headers
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any);

    let handler = ServiceBuilder::new()
        // Add the CORS layer to the service
        .layer(cors_layer)
        .service(service_fn(controller));

    lambda_http::run(handler).await?;
    Ok(())
}

async fn controller(event: Request) -> Result<Response<Body>, Error> {
    match event.uri().path().split('/').nth(1) {
        Some("verify") => verify_controller::controller(event).await,
        _ => Ok(Response::builder()
            .status(404)
            .body("not found".into())
            .expect("failed to render response")),
    }
}