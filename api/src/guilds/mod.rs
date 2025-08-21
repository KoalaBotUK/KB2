use crate::AppState;
use crate::guilds::models::{Guild, Verify};
use axum::extract::State;
use axum::{Extension, Json, Router, extract::Path, routing::get};
use http::StatusCode;
use lambda_http::tracing::warn;
use serde_json::{Value, json};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;

mod models;
pub mod verify;
mod utils;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_guilds))
        .route("/{guild_id}", get(get_guilds_id))
        .nest("/{guild_id}/verify", verify::router())
        .layer(CorsLayer::permissive())
}

async fn get_guilds(
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guilds = Guild::vec_from_db(
        utils::intersect_admin_guilds(
            &discord_user,
            &app_state.discord_bot,
        ).await
            .iter()
            .map(|g| g.id.to_string())
            .collect(),
        &app_state.dynamo,
    )
    .await;
    Ok(Json(json!(guilds)))
}

async fn get_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !utils::is_intersect_admin_guild(guild_id, &discord_user, &app_state.discord_bot).await.map_err(utils::ise)? {
        warn!("User is not an admin in guild {}", guild_id);
        return Err(StatusCode::NOT_FOUND);
    }

    let guild = Guild::from_db(&guild_id.to_string(), &app_state.dynamo).await;

    Ok(Json(json!(match guild {
        Some(g) => g,
        None => {
            let new_guild = Guild {
                guild_id: guild_id.to_string(),
                verify: Verify { roles: vec![] },
            };
            new_guild.save(&app_state.dynamo).await;
            new_guild
        }
    })))
}
