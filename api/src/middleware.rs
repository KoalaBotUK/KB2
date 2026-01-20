use crate::discord::{get_current_user, ise};
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;
use lambda_http::tracing::info;
use twilight_http::Client;

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
