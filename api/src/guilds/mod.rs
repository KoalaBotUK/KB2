use crate::AppState;
use crate::guilds::models::{Guild, Verify};
use axum::extract::State;
use axum::{Extension, Json, Router, extract::Path, routing::get};
use http::StatusCode;
use lambda_http::tracing::warn;
use serde_json::{Value, json};
use std::sync::Arc;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;

mod models;
pub mod verify;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_guilds))
        .route("/{guild_id}", get(get_guilds_id))
        .nest("/{guild_id}/verify", verify::router())
}

async fn get_guilds(
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let user_guilds = discord_user
        .current_user_guilds()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .models()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_guilds_filtered = user_guilds
        .iter()
        .filter(|g| g.permissions.contains(Permissions::ADMINISTRATOR))
        .collect::<Vec<_>>();
    let guilds = Guild::vec_from_db(
        user_guilds_filtered
            .iter()
            .map(|g| g.id.to_string())
            .collect(),
        &app_state.dynamo,
    )
    .await;
    Ok(Json(json!(guilds)))
}

async fn verify_user_admin(
    guild_id: &Id<GuildMarker>,
    discord_user: &Arc<twilight_http::Client>,
    app_state: &AppState,
) -> Result<(), StatusCode> {
    let logged_in_user = discord_user
        .current_user()
        .await
        .unwrap()
        .model()
        .await
        .unwrap();

    let guild = match app_state.discord_bot.guild(*guild_id).await {
        Ok(g) => g.model().await.unwrap(),
        Err(e) => {
            warn!("Error fetching guild: {:?}", e);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let user_is_owner = guild.owner_id == logged_in_user.id;

    if !user_is_owner {
        let member = match app_state
            .discord_bot
            .guild_member(*guild_id, logged_in_user.id)
            .await
        {
            Ok(gm) => gm.model().await.unwrap(),
            Err(e) => {
                warn!("Error fetching guild member: {:?}", e);
                return Err(StatusCode::NOT_FOUND);
            }
        };

        let user_has_admin = guild.roles.iter().any(|r| {
            (r.permissions & Permissions::ADMINISTRATOR) == Permissions::ADMINISTRATOR
                && member.roles.contains(&r.id)
        });
        if !user_has_admin {
            warn!("User does not have admin permissions in guild {}", guild_id);
            return Err(StatusCode::NOT_FOUND);
        }
    }
    Ok(())
}

async fn get_guilds_id(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    verify_user_admin(&guild_id, &discord_user, &app_state).await?;

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
