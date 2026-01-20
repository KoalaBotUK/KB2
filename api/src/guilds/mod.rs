use crate::AppState;
use crate::discord::{get_guild, get_guild_member};
use crate::guilds::models::Guild;
use crate::utils::{admin_guilds, is_client_admin_guild};
use axum::extract::State;
use axum::{Extension, Json, Router, extract::Path, routing::get};
use http::StatusCode;
use lambda_http::tracing::warn;
use serde_json::{Value, json};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use twilight_http::Client;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::{CurrentUser, CurrentUserGuild};

pub mod models;
pub(crate) mod utils;
pub mod verify;
pub(crate) mod votes;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_guilds).post(post_guilds))
        .route("/{guild_id}", get(get_guilds_id).post(post_guilds_id))
        .nest("/{guild_id}/verify", verify::controllers::router())
        .nest("/{guild_id}/votes", votes::controllers::router())
        .layer(CorsLayer::permissive())
}

async fn _verify_admin(
    guild_id: Id<GuildMarker>,
    current_user: &CurrentUser,
    discord_bot: &Client,
) -> Result<(), StatusCode> {
    let member = get_guild_member(guild_id, current_user.id, discord_bot).await?;
    let guild = get_guild(guild_id, discord_bot).await?;
    if guild.owner_id == current_user.id {
        return Ok(());
    }
    for role in guild.roles {
        if member.roles.contains(&role.id)
            && role.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR
        {
            return Ok(());
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

async fn post_guilds(
    Extension(discord_user): Extension<Arc<Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let admin_guilds = admin_guilds(&discord_user, &app_state.discord_bot).await?;
    let mut guilds = Guild::vec_from_db(
        admin_guilds.iter().map(|g| g.id).collect(),
        &app_state.pg_pool,
    )
    .await;
    let missing_guilds: Vec<&CurrentUserGuild> = admin_guilds
        .iter()
        .filter(|a| !guilds.iter().any(|g| g.guild_id == a.id))
        .collect();
    for admin_guild in missing_guilds {
        guilds.push(Guild {
            guild_id: admin_guild.id,
            ..Default::default()
        })
    }

    Ok(Json(json!(guilds)))
}

async fn get_guilds(
    Extension(discord_user): Extension<Arc<Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let guilds = Guild::vec_from_db(
        utils::intersect_admin_guilds(&discord_user, &app_state.discord_bot)
            .await?
            .iter()
            .map(|g| g.id)
            .collect(),
        &app_state.pg_pool,
    )
    .await;
    Ok(Json(json!(guilds)))
}

async fn post_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await?;
    let guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await
        .unwrap_or(Guild {
            guild_id,
            ..Default::default()
        });
    Ok(Json(json!(guild)))
}

async fn get_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !utils::is_intersect_admin_guild(guild_id, &discord_user, &app_state.discord_bot).await? {
        warn!("User is not an admin in guild {}", guild_id);
        return Err(StatusCode::NOT_FOUND);
    }

    let guild = Guild::from_db(guild_id, &app_state.pg_pool).await;

    Ok(Json(json!(match guild {
        Some(g) => g,
        None => {
            let new_guild = Guild {
                guild_id,
                ..Default::default()
            };
            new_guild.save(&app_state.pg_pool).await;
            new_guild
        }
    })))
}
