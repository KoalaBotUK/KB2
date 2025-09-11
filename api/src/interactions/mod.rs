use crate::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::post;
use axum::{Json, middleware};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, Verifier, VerifyingKey};
use hex::FromHex;
use http::{HeaderMap, StatusCode};
use http_body_util::BodyExt;
use lambda_http::tracing::error;
use once_cell::sync::Lazy;
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use twilight_model::application::command::CommandType;
use twilight_model::application::interaction::{
    Interaction, InteractionContextType, InteractionData, InteractionType,
};
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::Id;

static PUB_KEY: Lazy<VerifyingKey> = Lazy::new(|| {
    VerifyingKey::from_bytes(
        &<[u8; PUBLIC_KEY_LENGTH] as FromHex>::from_hex(
            std::env::var("DISCORD_PUBLIC_KEY").expect("DISCORD_PUBLIC_KEY must be set"),
        )
        .unwrap(),
    )
    .unwrap()
});

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(post_interactions))
        .route_layer(middleware::from_fn(pubkey_middleware))
        .route_layer(middleware::from_fn(user_agent_response_middleware))
        .route("/register", post(register_commands))
        .layer(CorsLayer::permissive())
}

pub async fn pubkey_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
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
        hex_sig.parse().map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let (parts, body) = request.into_parts();
    let body = body
        .collect()
        .await
        .map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_bytes();

    if PUB_KEY
        .verify([timestamp.as_bytes(), &body].concat().as_ref(), &signature)
        .is_err()
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let new_request = Request::from_parts(parts, axum::body::Body::from(body));
    Ok(next.run(new_request).await)
}

pub async fn user_agent_response_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;

    if response.headers().get("User-Agent").is_none() {
        let full_version = env!("CARGO_PKG_VERSION");
        response.headers_mut().insert(
            "User-Agent",
            format!("KoalaBot/{} (+{})", full_version, get_url())
                .parse()
                .unwrap(),
        );
    }

    Ok(response)
}

async fn post_interactions(
    State(_app_state): State<AppState>,
    Json(interaction): Json<Interaction>,
) -> Result<Json<Value>, StatusCode> {
    match interaction.kind {
        InteractionType::Ping => Ok(Json(json!(InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None
        }))),
        InteractionType::ApplicationCommand => {
            handle_command_interaction(_app_state, interaction).await
        }
        // InteractionType::ApplicationCommand => Err(StatusCode::ACCEPTED),
        InteractionType::MessageComponent => Err(StatusCode::NOT_IMPLEMENTED),
        InteractionType::ApplicationCommandAutocomplete => Err(StatusCode::NOT_IMPLEMENTED),
        InteractionType::ModalSubmit => Err(StatusCode::NOT_IMPLEMENTED),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

async fn handle_command_interaction(
    _app_state: AppState,
    interaction: Interaction,
) -> Result<Json<Value>, StatusCode> {
    let data = match interaction.data {
        Some(InteractionData::ApplicationCommand(data)) => Ok(data),
        _ => Err(StatusCode::BAD_REQUEST),
    }?;

    match data.name.as_ref() {
        "support" => support().await,
        "verify" => verify().await,
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

async fn support() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some("Join our support server for more help!".into()),
            components: Some(vec![Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some("Koala Support".to_owned()),
                    style: ButtonStyle::Link,
                    url: Some("https://discord.gg/5etEjVd".to_owned()),
                    sku_id: None,
                })],
            })]),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    })))
}

fn get_url() -> String {
    match std::env::var("DEPLOYMENT_ENV")
        .unwrap_or("prod".into())
        .as_str()
    {
        "prod" => "https://koalabot.uk".to_owned(),
        env => format!("https://{env}.koalabot.uk").to_owned(),
    }
}

async fn verify() -> Result<Json<Value>, StatusCode> {
    let url = get_url();
    Ok(Json(json!(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some("Verify yourself on our site!".into()),
            components: Some(vec![Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some("Koala Verify".to_owned()),
                    style: ButtonStyle::Link,
                    url: Some(format!("{url}/verify").to_owned()),
                    sku_id: None,
                })],
            })]),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    })))
}

async fn register_commands(
    header_map: HeaderMap,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let auth_token = header_map
        .get("Authorization")
        .unwrap()
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if auth_token != app_state.discord_bot.token().unwrap() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let application_id = app_state
        .discord_bot
        .current_user_application()
        .await
        .map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .model()
        .await
        .map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .id;
    let resp = app_state
        .discord_bot
        .interaction(application_id)
        .set_global_commands(&[
            twilight_model::application::command::Command {
                id: None,
                integration_types: None,
                application_id: Some(application_id),
                name: "support".to_owned(),
                name_localizations: None,
                description: "Get support for Koala Bot".to_owned(),
                description_localizations: None,
                options: vec![],
                default_member_permissions: None,
                version: Id::new(1),
                contexts: Some(vec![
                    InteractionContextType::Guild,
                    InteractionContextType::BotDm,
                ]),
                guild_id: None,
                kind: CommandType::ChatInput,
                nsfw: None,
                #[allow(deprecated)]
                dm_permission: None,
            },
            twilight_model::application::command::Command {
                id: None,
                integration_types: None,
                application_id: Some(application_id),
                name: "verify".to_owned(),
                name_localizations: None,
                description: "Verify yourself on the Koala Bot site".to_owned(),
                description_localizations: None,
                options: vec![],
                default_member_permissions: None,
                version: Id::new(1),
                contexts: Some(vec![
                    InteractionContextType::Guild,
                    InteractionContextType::BotDm,
                ]),
                guild_id: None,
                kind: CommandType::ChatInput,
                nsfw: None,
                #[allow(deprecated)]
                dm_permission: None,
            },
        ])
        .await
        .map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .model()
        .await
        .map_err(|e| {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(json!(resp)))
}
