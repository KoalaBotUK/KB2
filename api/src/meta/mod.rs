use std::ops::{Add, Sub};
use std::sync::Arc;
use std::time::Duration;
use axum::{Extension, Json, Router};
use axum::extract::{Path, State};
use axum::routing::get;
use http::StatusCode;
use lambda_http::tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::time::{sleep, Instant};
use tower_http::cors::CorsLayer;
use twilight_http::Client;
use twilight_model::channel::Channel;
use twilight_model::guild::{Permissions, Role};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUserGuild;
use twilight_model::util::ImageHash;
use crate::AppState;
use crate::discord::{get_current_user_guild, get_current_user_guilds_prime_cache, get_guild, get_guild_channels, get_guild_prime_cache};
use crate::utils::member_guilds;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/guilds", get(get_meta_guilds))
        .route("/guilds/{guild_id}", get(get_meta_guilds_id))
        .layer(CorsLayer::permissive())
}

pub fn setup(discord_bot: Arc<Client>) {
    info!("Spawning meta cache refresh task");
    tokio::spawn(refresh_meta_cache(discord_bot));
}


#[derive(Debug, Serialize, Deserialize)]
struct PartialGuildMeta {
    id: Id<GuildMarker>,
    name: String,
    icon: Option<ImageHash>,
    is_admin: bool,
}
impl From<CurrentUserGuild> for PartialGuildMeta {
    fn from(guild: CurrentUserGuild) -> Self {
        PartialGuildMeta {
            id: guild.id,
            name: guild.name,
            icon: guild.icon,
            is_admin: guild.owner || guild.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct GuildMeta {
    id: Id<GuildMarker>,
    name: String,
    icon: Option<ImageHash>,
    is_admin: bool,
    roles: Vec<Role>,
    channels: Vec<Channel>,
}

async fn get_meta_guilds(
    Extension(discord_user): Extension<Arc<Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!(
        member_guilds(&discord_user, &app_state.discord_bot).await?
        .into_iter().map(PartialGuildMeta::from).collect::<Vec<PartialGuildMeta>>()
    )))
}

async fn get_meta_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !crate::guilds::utils::is_intersect_admin_guild(guild_id, &discord_user, &app_state.discord_bot).await? {
        warn!("User is not an admin in guild {}", guild_id);
        return Err(StatusCode::NOT_FOUND);
    }
    
    let u_guild = get_current_user_guild(guild_id, &discord_user).await?;
    let guild = get_guild(guild_id, &app_state.discord_bot).await?;
    let channels = get_guild_channels(guild_id, &app_state.discord_bot).await?;
    
    Ok(Json(json!(
        GuildMeta{
            id: guild_id,
            name: guild.name,
            icon: guild.icon,
            is_admin: u_guild.owner || u_guild.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR,
            roles: guild.roles,
            channels: channels
        }
    )))
}



async fn refresh_meta_cache(discord_bot: Arc<Client>) {
    loop {
        debug!("Refreshing meta cache");
        let time = Instant::now();
        match get_current_user_guilds_prime_cache(&*discord_bot).await {
            Ok(guilds) => {
                debug!("Refreshing meta cache: {}", guilds.len());
                for guild in guilds {
                    let _ = get_guild_prime_cache(guild.id, &*discord_bot).await;
                }
            }
            Err(e) => {
                error!("refresh_meta_cache error: {:#?}", e);
            }
        }
        debug!("{:?} {:?} Waiting {} seconds",time, Instant::now(), (time.sub(Instant::now()).add(Duration::from_secs(50))).as_secs());
        sleep((time.sub(Instant::now())).add(Duration::from_secs(50))).await;
    }
}