use crate::discord::{get_current_user, ise};
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;
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
    let (parts, body) = request.into_parts();
    let body = body.collect().await.map_err(ise)?.to_bytes();

    info!("Received request: {:?} {:?}", parts, body);

    // Call the next middleware or handler
    let response = next
        .run(Request::from_parts(parts, axum::body::Body::from(body)))
        .await;

    let (parts, body) = response.into_parts();
    let body = body.collect().await.map_err(ise)?.to_bytes();

    // Log the response
    info!("Response: {:?} {:?}", parts, body);

    Ok(Response::from_parts(parts, axum::body::Body::from(body)))
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
}
