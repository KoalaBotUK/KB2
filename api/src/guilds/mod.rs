use crate::AppState;
use axum::{extract::Path, routing::get, Extension, Json, Router};
use http::StatusCode;
use serde_json::{json, Value};
use std::sync::Arc;
use axum::extract::State;
use lambda_http::tracing::warn;
use twilight_model::guild::Permissions;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

pub mod verify;
mod models;



pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_guilds))
        .route("/{guild_id}", get(get_guilds_id))
        .nest("/{guild_id}/verify", verify::router())
}

async fn get_guilds(
) -> Json<Value> {
    todo!()
}


async fn get_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>
) -> Result<Json<Value>, StatusCode> {
    let logged_in_user = discord_user.current_user().await.unwrap().model().await.unwrap();

    let guild = match app_state.discord_bot.guild(guild_id).await {
        Ok(g) => g.model().await.unwrap(),
        Err(e) => {
            warn!("Error fetching guild: {:?}", e);
            return Err(StatusCode::NOT_FOUND)
        },
    };

    let user_is_owner = guild.owner_id == logged_in_user.id;

    if !user_is_owner {
        let member = match app_state.discord_bot.guild_member(guild_id, logged_in_user.id).await {
            Ok(gm) => gm.model().await.unwrap(),
            Err(e) => {
                warn!("Error fetching guild member: {:?}", e);
                return Err(StatusCode::NOT_FOUND)
            },
        };

        let user_has_admin = guild.roles.iter()
            .any(|r| (r.permissions & Permissions::ADMINISTRATOR) == Permissions::ADMINISTRATOR && member.roles.contains(&r.id));
        if !user_has_admin {
            warn!("User does not have admin permissions in guild {}", guild_id);
            return Err(StatusCode::NOT_FOUND);
        }
    }

    Ok(Json(json!(guild)))
}
