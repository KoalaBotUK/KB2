use crate::discord::get_current_user;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use lambda_http::tracing::info;
use twilight_http::Client;

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

/// Value substituted for sensitive header values when logging.
const REDACTED_PLACEHOLDER: &str = "***REDACTED***";

/// Headers whose values must never be written to logs verbatim.
///
/// Comparison is case-insensitive (matching `http::HeaderName` semantics).
const SENSITIVE_HEADERS: &[&str] = &["authorization", "cookie", "set-cookie"];

/// Build a loggable representation of a header map with sensitive values
/// (e.g. `Authorization` tokens, cookies) redacted.
///
/// This exists so request/response headers can still be logged for
/// debugging purposes without leaking credentials (Discord OAuth access
/// tokens, the bot token, session cookies, etc.) into CloudWatch.
fn redact_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            let name = name.as_str().to_string();
            if SENSITIVE_HEADERS.contains(&name.to_lowercase().as_str()) {
                (name, REDACTED_PLACEHOLDER.to_string())
            } else {
                let value = value
                    .to_str()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "<invalid utf-8>".to_string());
                (name, value)
            }
        })
        .collect()
}

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // do something with `request`...
    let (scheme, credentials) = parse_auth_header(&headers)?;
    match scheme {
        "Discord" => {
            let client = Client::new(format!("Bearer {credentials}"));
            let current_user = get_current_user(&client).await?;
            if current_user.bot {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let ext_mut = request.extensions_mut();
            ext_mut.insert(std::sync::Arc::new(client));
            ext_mut.insert(current_user);
            Ok(next.run(request).await)
        }
        "Bot" => {
            if credentials
                != std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set")
            {
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

pub async fn log_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Only capture method/URI/headers metadata - request and response bodies
    // are never logged, as they may contain secrets (OIDC tokens) or PII
    // (email addresses). Sensitive headers such as `Authorization` are
    // redacted before logging.
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_headers = redact_headers(request.headers());

    info!(
        "Received request: {} {} headers={:?}",
        method, uri, request_headers
    );

    let response = next.run(request).await;

    let response_headers = redact_headers(response.headers());
    info!(
        "Response: {} {} -> {} headers={:?}",
        method,
        uri,
        response.status(),
        response_headers
    );

    Ok(response)
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
        headers.insert("Authorization", HeaderValue::from_static("Bot secret-token"));

        let (scheme, credentials) = parse_auth_header(&headers).expect("should parse header");

        assert_eq!(scheme, "Bot");
        assert_eq!(credentials, "secret-token");
    }

    fn header_value(name: &str, redacted: &[(String, String)]) -> Option<String> {
        redacted
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    }

    #[test]
    fn redact_headers_hides_authorization_value() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Discord super-secret-oauth-token"),
        );
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let redacted = redact_headers(&headers);
        let rendered = format!("{:?}", redacted);

        // The raw secret must never appear anywhere in the loggable output.
        assert!(!rendered.contains("super-secret-oauth-token"));
        assert_eq!(
            header_value("Authorization", &redacted).as_deref(),
            Some(REDACTED_PLACEHOLDER)
        );
        // Non-sensitive headers should pass through unchanged.
        assert_eq!(
            header_value("Content-Type", &redacted).as_deref(),
            Some("application/json")
        );
    }

    #[test]
    fn redact_headers_is_case_insensitive() {
        let mut headers = HeaderMap::new();
        // HeaderName normalizes case, but exercise via the lowercase form
        // that http::HeaderName always reports from `.as_str()`.
        headers.insert("authorization", HeaderValue::from_static("Bot abc123"));

        let redacted = redact_headers(&headers);
        let rendered = format!("{:?}", redacted);

        assert!(!rendered.contains("abc123"));
        assert_eq!(
            header_value("authorization", &redacted).as_deref(),
            Some(REDACTED_PLACEHOLDER)
        );
    }

    #[test]
    fn redact_headers_hides_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Cookie",
            HeaderValue::from_static("session=top-secret-session-id"),
        );
        headers.insert(
            "Set-Cookie",
            HeaderValue::from_static("session=another-secret; HttpOnly"),
        );

        let redacted = redact_headers(&headers);
        let rendered = format!("{:?}", redacted);

        assert!(!rendered.contains("top-secret-session-id"));
        assert!(!rendered.contains("another-secret"));
        assert_eq!(
            header_value("Cookie", &redacted).as_deref(),
            Some(REDACTED_PLACEHOLDER)
        );
        assert_eq!(
            header_value("Set-Cookie", &redacted).as_deref(),
            Some(REDACTED_PLACEHOLDER)
        );
    }

    #[test]
    fn redact_headers_preserves_non_sensitive_headers_and_count() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Request-Id", HeaderValue::from_static("req-42"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        let redacted = redact_headers(&headers);

        assert_eq!(redacted.len(), 2);
        assert_eq!(
            header_value("X-Request-Id", &redacted).as_deref(),
            Some("req-42")
        );
        assert_eq!(
            header_value("Accept", &redacted).as_deref(),
            Some("application/json")
        );
    }
}
