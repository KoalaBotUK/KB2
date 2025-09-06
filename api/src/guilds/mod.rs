use std::collections::{HashSet};
use crate::AppState;
use crate::guilds::models::{Guild, VerifyRole};
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
use twilight_model::user::{CurrentUser, CurrentUserGuild};
use crate::discord::{get_guild, get_guild_member};
use crate::guilds::tasks::{add_role_to_guild, remove_role_from_guild};
use crate::utils::{admin_guilds, is_client_admin_guild};

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
        if member.roles.contains(&role.id) && role.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR {
            return Ok(());
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

async fn post_guilds(
    Extension(discord_user): Extension<Arc<Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("post_guilds");
    let admin_guilds = admin_guilds(&discord_user, &app_state.discord_bot).await?;
    let mut guilds = Guild::vec_from_db(admin_guilds.iter().map(|g| g.id).collect(), &app_state.dynamo).await;
    for guild in &mut guilds {
        for g in &admin_guilds {
            if g.id == guild.guild_id && (g.name != guild.name || g.icon != guild.icon) {
                guild.name = g.name.clone();
                guild.icon = g.icon.clone();
                guild.save(&app_state.dynamo).await;
            }
        }
    }
    
    let missing_guilds: Vec<&CurrentUserGuild> =  admin_guilds.iter().filter(|a| !guilds.iter().any(|g| g.guild_id == a.id)).collect();
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
        utils::intersect_admin_guilds(
            &discord_user,
            &app_state.discord_bot,
        ).await?
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
    is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await?;
    let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap_or(Guild {
        guild_id,
        ..Default::default()
    });
    let guild_dsc = get_guild(guild.guild_id, &app_state.discord_bot).await?;
    guild.name = guild_dsc.name;
    guild.icon = guild_dsc.icon;
    guild.save(&app_state.dynamo).await;

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

    let guild = Guild::from_db(guild_id, &app_state.dynamo).await;

    Ok(Json(json!(match guild {
        Some(g) => g,
        None => {
            let new_guild = Guild {
                guild_id,
                ..Default::default()
            };
            new_guild.save(&app_state.dynamo).await;
            new_guild
        }
    })))
}

async fn put_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(guild_req): Json<Guild>,
) -> Result<Json<Value>, StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut old_guild = match Guild::from_db(guild_id, &app_state.dynamo).await {
        Some(g) => g,
        None => return Err(StatusCode::NOT_FOUND),
    };

    let old_set: HashSet<VerifyRole> = HashSet::from_iter(old_guild.verify.roles.clone());
    let new_set: HashSet<VerifyRole> = HashSet::from_iter(guild_req.verify.roles);

    let add_roles: HashSet<VerifyRole> = new_set.difference(&old_set).cloned().collect();
    let remove_roles: HashSet<VerifyRole> = old_set.difference(&new_set).cloned().collect();

    for role in add_roles {
        add_role_to_guild(&mut old_guild, role, &app_state.discord_bot).await?;
    }

    for role in remove_roles {
        remove_role_from_guild(&mut old_guild, role.role_id, &app_state.discord_bot).await?;
    }

    old_guild.save(&app_state.dynamo).await;
    Ok(Json(json!(old_guild)))
}
