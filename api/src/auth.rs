use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
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