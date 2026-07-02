use crate::discord::get_current_user;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use lambda_http::tracing::info;
use twilight_http::Client;

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
    if headers.get("Authorization").is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth_header = headers.get("Authorization").unwrap().to_str().unwrap();
    let (scheme, credentials) = auth_header.split_once(' ').unwrap();
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
