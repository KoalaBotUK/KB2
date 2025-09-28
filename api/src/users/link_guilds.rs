use crate::discord::get_current_user_guild;
use crate::guilds::models::Guild;
use crate::guilds::tasks::assign_roles_guild_user_link;
use crate::users::models::{LinkGuild, User};
use crate::AppState;
use axum::extract::{Path, State};
use axum::routing::put;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::user::CurrentUser;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/{guild_id}", put(put_link_guilds_id))
        .layer(CorsLayer::permissive())
}

#[derive(Clone, Serialize, Deserialize)]
struct LinkGuildsRequest {
    enabled: bool,
}

async fn put_link_guilds_id(
    Path((user_id, guild_id)): Path<(Id<UserMarker>,Id<GuildMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
    Json(link_guilds_req): Json<LinkGuildsRequest>,
) -> Result<Json<serde_json::Value>, http::StatusCode> {
    if current_user.id != user_id || !is_client_guild_member(guild_id, &discord_user).await {
        return Err(http::StatusCode::UNAUTHORIZED);
    }
    
    let new_link_guild = LinkGuild {
        guild_id,
        enabled: link_guilds_req.enabled,
    };
    
    let mut user = User::from_db(&user_id.to_string(), &app_state.dynamo).await.unwrap();
    match user.link_guilds.iter_mut().find(|g| g.guild_id == guild_id) {
        Some(link_guild) => {
            if link_guild.enabled == link_guilds_req.enabled {
                return Ok(Json(json!(new_link_guild)));
            }
            user.link_guilds.retain(|g| g.guild_id != guild_id);
            if new_link_guild.enabled {
                user.link_guilds.push(new_link_guild.clone());
            }
        },
        None => {
            if !new_link_guild.enabled {
                return Ok(Json(json!(new_link_guild)));
            }
            user.link_guilds.push(new_link_guild.clone());
        }
    }
    user.save(&app_state.dynamo).await;
    
    let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap();
    
    match guild.verify.user_links.get(&user.user_id) {
        Some(_links) => {
            if !new_link_guild.enabled {
                guild.verify.user_links.remove(&user.user_id);
                for user_link in &user.links {
                    assign_roles_guild_user_link(
                        false,
                        &user_link.link_address,
                        user_id,
                        &mut guild,
                        &app_state.discord_bot,
                    ).await?;
                }
            } else {
                // Already verified on this server
                return Ok(Json(json!(new_link_guild)));
            }
            
        },
        None => {
            if new_link_guild.enabled {
                guild.verify.user_links.insert(user.user_id, user.links.clone());
                for user_link in &user.links {
                    assign_roles_guild_user_link(
                        true,
                        &user_link.link_address,
                        user_id,
                        &mut guild,
                        &app_state.discord_bot,
                    ).await?;
                }
            } else {
                // Not on this server
                return Ok(Json(json!(new_link_guild)));
            }
        }
    }
    guild.save(&app_state.dynamo).await;
    Ok(Json(json!(new_link_guild)))
}

async fn is_client_guild_member(guild_id: Id<GuildMarker>,
                                client: &twilight_http::Client) -> bool {
    get_current_user_guild(guild_id, client).await.is_ok()
}