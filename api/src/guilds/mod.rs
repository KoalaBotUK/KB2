use std::collections::HashMap;
use crate::AppState;
use crate::guilds::models::{Guild, Verify};
use axum::extract::State;
use axum::{Extension, Json, Router, extract::Path, routing::get};
use http::StatusCode;
use lambda_http::tracing::{info, warn};
use serde_json::{Value, json};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use twilight_http::Client;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUser;
use crate::utils::{admin_guilds, is_client_admin_guild, ise};

pub mod models;
pub mod verify;
mod utils;
pub mod tasks;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_guilds).post(post_guilds))
        .route("/{guild_id}", get(get_guilds_id).put(put_guilds_id).post(post_guilds_id))
        .nest("/{guild_id}/verify", verify::router())
        .layer(CorsLayer::permissive())
}

#[derive(Clone, Deserialize, Serialize)]
struct GuildRequest {
    guild_id: Id<GuildMarker>,
}

async fn verify_admin(
    guild_id: Id<GuildMarker>,
    current_user: &CurrentUser,
    discord_bot: &Client,
) -> Result<(), StatusCode> {
    let member = discord_bot.guild_member(guild_id, current_user.id).await.map_err(ise)?.model().await.map_err(ise)?;
    let guild = discord_bot.guild(guild_id).await.map_err(ise)?.model().await.map_err(ise)?;
    if guild.owner_id == current_user.id {
        return Ok(());
    }
    for role in guild.roles {
        if member.roles.contains(&role.id) && role.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR {
            return Ok(());
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

async fn post_guilds(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("post_guilds");
    let admin_guilds = admin_guilds(&current_user, &app_state.discord_bot).await.map_err(ise)?;
    let mut guilds = Guild::vec_from_db(admin_guilds.iter().map(|g| g.id).collect(), &app_state.dynamo).await;
    let missing_guilds: Vec<&twilight_model::guild::Guild> =  admin_guilds.iter().filter(|a| !guilds.iter().any(|g| g.guild_id == a.id)).collect();
    for admin_guild in missing_guilds {
        guilds.push(Guild {
            guild_id: admin_guild.id,
            ..Default::default()
        })
    }
    
    for i in 0..guilds.len() {
        let guild_dsc = app_state.discord_bot.guild(guilds[i].guild_id).await.map_err(ise)?.model().await.map_err(ise)?;
        guilds[i].name = guild_dsc.name;
        guilds[i].icon = guild_dsc.icon;
        guilds[i].save(&app_state.dynamo).await;
    }
    
    Ok(Json(json!(guilds)))
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
            .map(|g| g.id)
            .collect(),
        &app_state.dynamo,
    )
    .await;
    Ok(Json(json!(guilds)))
}


async fn post_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await.map_err(ise)?;
    let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap_or(Guild {
        guild_id,
        ..Default::default()
    });
    let guild_dsc = app_state.discord_bot.guild(guild.guild_id).await.map_err(ise)?.model().await.map_err(ise)?;
    guild.name = guild_dsc.name;
    guild.icon = guild_dsc.icon;
    guild.save(&app_state.dynamo).await;

    Ok(Json(json!(guild)))
}


async fn get_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !utils::is_intersect_admin_guild(guild_id, &discord_user, &app_state.discord_bot).await.map_err(crate::utils::ise)? {
        warn!("User is not an admin in guild {}", guild_id);
        return Err(StatusCode::NOT_FOUND);
    }

    let guild = Guild::from_db(guild_id, &app_state.dynamo).await;

    Ok(Json(json!(match guild {
        Some(g) => g,
        None => {
            let guild = app_state.discord_bot.guild(guild_id).await.unwrap().model().await.unwrap();
            let new_guild = Guild {
                guild_id,
                verify: Verify { roles: vec![], user_links: vec![] },
                name: guild.name,
                icon: guild.icon,
                user_links: HashMap::new()
            };
            new_guild.save(&app_state.dynamo).await;
            new_guild
        }
    })))
}

async fn put_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
    Json(guild_req): Json<models::Guild>,
) -> Result<Json<Value>, StatusCode> {
    if !utils::is_intersect_admin_guild(guild_id, &discord_user, &app_state.discord_bot).await.map_err(crate::utils::ise)? {
        warn!("User is not an admin in guild {}", guild_id);
        return Err(StatusCode::NOT_FOUND);
    }

    guild_req.save(&app_state.dynamo).await;
    Ok(Json(json!(guild_req)))
}
