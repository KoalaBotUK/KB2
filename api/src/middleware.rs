use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;
use lambda_http::tracing::{error, info};
use twilight_http::Client;
use crate::utils::ise;

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // do something with `request`...
    println!("Attempting to authenticate user");

    if headers.get("Authorization").is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth_header = headers.get("Authorization").unwrap().to_str().unwrap();
    let (scheme, credentials) = auth_header.split_once(' ').unwrap();
    println!("credentials: {credentials}");
    if scheme == "Discord" {
        let client = Client::new(format!("Bearer {credentials}"));
        let current_user = client.current_user().await.map_err(ise)?.model().await.map_err(ise)?;
        let ext_mut = request.extensions_mut();
        ext_mut.insert(std::sync::Arc::new(client));
        ext_mut.insert(current_user);
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn log_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let (parts, body) = request.into_parts();
    let body = body
        .collect()
        .await
        .map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_bytes();

    info!("Received request: {:?} {:?}", parts, body);

    // Call the next middleware or handler
    let response = next
        .run(Request::from_parts(parts, axum::body::Body::from(body)))
        .await;

    let (parts, body) = response.into_parts();
    let body = body
        .collect()
        .await
        .map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_bytes();

    // Log the response
    info!("Response: {:?} {:?}", parts, body);

    Ok(Response::from_parts(parts, axum::body::Body::from(body)))
}
