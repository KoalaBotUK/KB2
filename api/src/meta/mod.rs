use std::ops::{Add, Sub};
use std::sync::Arc;
use std::time::Duration;
use axum::{Extension, Json, Router};
use axum::extract::State;
use axum::routing::get;
use http::StatusCode;
use lambda_http::tracing::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::time::{sleep, Instant};
use tower_http::cors::CorsLayer;
use twilight_http::Client;
use twilight_model::guild::{Guild, Role};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUserGuild;
use twilight_model::util::ImageHash;
use crate::AppState;
use crate::discord::{get_current_user_guilds_prime_cache, get_guild, get_guild_prime_cache};
use crate::utils::member_guilds;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/guilds", get(get_guilds_meta))
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
}
impl From<CurrentUserGuild> for PartialGuildMeta {
    fn from(guild: CurrentUserGuild) -> Self {
        PartialGuildMeta {
            id: guild.id,
            name: guild.name,
            icon: guild.icon,
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct GuildMeta {
    id: Id<GuildMarker>,
    name: String,
    icon: Option<ImageHash>,
    roles: Vec<Role>,
}

impl From<Guild> for GuildMeta {
    fn from(guild: Guild) -> Self {
        GuildMeta {
            id: guild.id,
            name: guild.name,
            icon: guild.icon,
            roles: guild.roles,
        }
    }
}

async fn get_guilds_meta(
    Extension(discord_user): Extension<Arc<Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!(
        member_guilds(&discord_user, &app_state.discord_bot).await?
        .into_iter().map(PartialGuildMeta::from).collect::<Vec<PartialGuildMeta>>()
    )))

}

async fn refresh_meta_cache(discord_bot: Arc<Client>) {
    loop {
        info!("Refreshing meta cache");
        let time = Instant::now();
        match get_current_user_guilds_prime_cache(&*discord_bot).await {
            Ok(guilds) => {
                info!("Refreshing meta cache: {}", guilds.len());
                for guild in guilds {
                    let _ = get_guild_prime_cache(guild.id, &*discord_bot).await;
                }
            }
            Err(e) => {
                error!("refresh_meta_cache error: {:#?}", e);
            }
        }
        info!("{:?} {:?} Waiting {} seconds",time, Instant::now(), (time.sub(Instant::now()).add(Duration::from_secs(50))).as_secs());
        sleep((time.sub(Instant::now())).add(Duration::from_secs(50))).await;
    }
}