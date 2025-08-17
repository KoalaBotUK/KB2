use crate::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::post;
use axum::{middleware, Json};
use ed25519_dalek::{Verifier, VerifyingKey, PUBLIC_KEY_LENGTH};
use hex::FromHex;
use http::StatusCode;
use http_body_util::BodyExt;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use twilight_model::application::interaction::{Interaction, InteractionType};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};

static PUB_KEY: Lazy<VerifyingKey> = Lazy::new(|| {
    VerifyingKey::from_bytes(&<[u8; PUBLIC_KEY_LENGTH] as FromHex>::from_hex(std::env::var("DISCORD_PUBLIC_KEY").expect("DISCORD_PUBLIC_KEY must be set")).unwrap())
        .unwrap()
});

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(post_interactions))
        .layer(middleware::from_fn(pubkey_middleware))
}

pub async fn pubkey_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let timestamp = if let Some(ts) = request.headers().get("x-signature-timestamp") {
        ts.to_owned()
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };
    // Extract the signature to check against.
    let signature = if let Some(hex_sig) = request
        .headers()
        .get("x-signature-ed25519")
        .and_then(|v| v.to_str().ok())
    {
        hex_sig.parse().unwrap()
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let (parts, body) = request.into_parts();
    let body = body
        .collect()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_bytes();

    if PUB_KEY
        .verify(
            [timestamp.as_bytes(), &body].concat().as_ref(),
            &signature,
        )
        .is_err()
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let new_request = Request::from_parts(parts, axum::body::Body::from(body));
    Ok(next.run(new_request).await)
}

async fn post_interactions(
    State(_app_state): State<AppState>,
    Json(interaction): Json<Interaction>,
) -> Result<Json<Value>, StatusCode> {
    match interaction.kind {
        InteractionType::Ping => {
            Ok(Json(json!(InteractionResponse {kind: InteractionResponseType::Pong, data: None})))
        }
        InteractionType::ApplicationCommand => {
            Err(StatusCode::NOT_IMPLEMENTED)
        }
        InteractionType::MessageComponent => {
            Err(StatusCode::NOT_IMPLEMENTED)
        }
        InteractionType::ApplicationCommandAutocomplete => {
            Err(StatusCode::NOT_IMPLEMENTED)
        }
        InteractionType::ModalSubmit => {
            Err(StatusCode::NOT_IMPLEMENTED)
        }
        _ => Err(StatusCode::BAD_REQUEST)
    }
}