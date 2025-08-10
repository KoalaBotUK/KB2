use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use crate::users::User;

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // do something with `request`...

    if headers.get("Authorization").is_some() {
        request.extensions_mut().insert(User{user_id: "1".to_string(), emails: vec![], username: "".to_string(), first_name: "".to_string(), last_name: "".to_string() });
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}