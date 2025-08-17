use http_body_util::BodyExt;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use lambda_http::tracing::{error, info};
use twilight_http::Client;

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
    let mut auth_split = auth_header.splitn(2, ' ');
    let scheme = auth_split.next().unwrap();
    let credentials = auth_split.next().unwrap();
    println!("credentials: {}", credentials);
    if scheme == "Discord" {
        request.extensions_mut().insert(std::sync::Arc::new(Client::new(format!("Bearer {}", credentials))));
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn log_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (parts, body) = request.into_parts();
    let body = body
        .collect()
        .await
        .map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_bytes();

    info!("Received request: {:?} {:?}",parts, body);

    // Call the next middleware or handler
    let response = next.run(Request::from_parts(parts,axum::body::Body::from(body))).await;


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
    info!("Response: {:?} {:?}",parts, body);

    Ok(Response::from_parts(parts,axum::body::Body::from(body)))
}