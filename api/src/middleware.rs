use crate::discord::get_current_user;
use crate::utils::secure_compare;
use axum::extract::{MatchedPath, Request};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use lambda_http::tracing::{Instrument, info, info_span, warn};
use std::time::Instant;
use twilight_http::Client;

pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-kb-request-id");

/// Parses the `Authorization` header into a `(scheme, credentials)` pair.
///
/// Returns `Err(StatusCode::UNAUTHORIZED)` (instead of panicking) if the
/// header is missing, contains non-UTF8/non-visible-ASCII bytes, or does not
/// contain the expected `<scheme> <credentials>` format.
fn parse_auth_header(headers: &HeaderMap) -> Result<(&str, &str), StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    auth_header.split_once(' ').ok_or(StatusCode::UNAUTHORIZED)
}

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (scheme, credentials) = parse_auth_header(&headers).inspect_err(|_| {
        warn!(
            stage = "authenticate",
            reason = "missing_or_invalid_authorization",
            "Authentication failed"
        );
    })?;
    match scheme {
        "Discord" => {
            let client = Client::new(format!("Bearer {credentials}"));
            info!(stage = "authenticate", "Checking Discord session");
            let current_user = get_current_user(&client).await.inspect_err(|status| {
                warn!(
                    stage = "authenticate",
                    reason = "discord_auth_failed",
                    status = status.as_u16(),
                    "Authentication failed"
                );
            })?;
            if current_user.bot {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let ext_mut = request.extensions_mut();
            ext_mut.insert(std::sync::Arc::new(client));
            ext_mut.insert(current_user);
            Ok(next.run(request).await)
        }
        "Bot" => {
            let expected_token =
                std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set");
            if !secure_compare(credentials, &expected_token) {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let client = Client::new(format!("Bot {credentials}"));
            let ext_mut = request.extensions_mut();
            ext_mut.insert(std::sync::Arc::new(client));
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn log_middleware(request: Request, next: Next) -> Response {
    let request_id = request
        .extensions()
        .get::<lambda_http::Context>()
        .map(|context| context.request_id.clone())
        .unwrap_or_default();
    // MatchedPath is the route template, so path parameters and query strings stay out of logs.
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or("unmatched");
    let span = info_span!("http_request", %request_id, method = %request.method(), route);

    async move {
        let start = Instant::now();
        info!("Request started");
        let mut response = next.run(request).await;
        info!(
            status = response.status().as_u16(),
            elapsed_ms = start.elapsed().as_millis() as u64,
            "Request completed"
        );
        if !request_id.is_empty()
            && let Ok(value) = HeaderValue::from_str(&request_id)
        {
            response.headers_mut().insert(REQUEST_ID_HEADER, value);
        }
        response
    }
    .instrument(span)
    .await
}

pub async fn mock_ctx_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    request
        .extensions_mut()
        .insert(lambda_http::Context::default());
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn parse_auth_header_missing_scheme_prefix_returns_unauthorized() {
        // No space in the header value, so there is no `<scheme> <credentials>` split point.
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("x"));

        let result = parse_auth_header(&headers);

        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn parse_auth_header_non_utf8_returns_unauthorized() {
        // Non-visible-ASCII / non-UTF8 bytes make `HeaderValue::to_str()` fail.
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_bytes(&[0xC0, 0xAF]).expect("invalid header value bytes"),
        );

        let result = parse_auth_header(&headers);

        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn parse_auth_header_missing_header_returns_unauthorized() {
        let headers = HeaderMap::new();

        let result = parse_auth_header(&headers);

        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn parse_auth_header_well_formed_header_parses_successfully() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Discord some.jwt.token"),
        );

        let (scheme, credentials) = parse_auth_header(&headers).expect("should parse header");

        assert_eq!(scheme, "Discord");
        assert_eq!(credentials, "some.jwt.token");
    }

    #[test]
    fn parse_auth_header_well_formed_bot_header_parses_successfully() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bot secret-token"),
        );

        let (scheme, credentials) = parse_auth_header(&headers).expect("should parse header");

        assert_eq!(scheme, "Bot");
        assert_eq!(credentials, "secret-token");
    }

    #[tokio::test]
    async fn request_logger_returns_the_lambda_request_id_and_preserves_the_response() {
        use axum::Router;
        use axum::body::{Body, to_bytes};
        use axum::routing::post;
        use lambda_http::tower::ServiceExt;

        let app = Router::new()
            .route(
                "/users/{user_id}/links",
                post(|body: String| async { body }),
            )
            .route_layer(axum::middleware::from_fn(log_middleware));
        let mut request = Request::builder()
            .method("POST")
            .uri("/users/123/links")
            .body(Body::from("fixture"))
            .unwrap();
        let mut context = lambda_http::Context::default();
        context.request_id = "b37656aa-a9f4-46e8-9022-928711feec0b".into();
        request.extensions_mut().insert(context);

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[REQUEST_ID_HEADER],
            "b37656aa-a9f4-46e8-9022-928711feec0b"
        );
        assert_eq!(to_bytes(response.into_body(), 32).await.unwrap(), "fixture");
    }
}
