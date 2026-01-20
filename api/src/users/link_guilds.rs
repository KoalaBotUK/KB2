use crate::AppState;
use crate::discord::{add_guild_member_role, get_current_user_guild, remove_guild_member_role};
use crate::guilds::models::Guild;
use crate::users::models::{LinkGuild, User};
use crate::users::utils::link_arr_match;
use axum::extract::{Path, State};
use axum::routing::put;
use axum::{Extension, Json};
use http::StatusCode;
use serde_json::{Value, json};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::user::CurrentUser;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            "/{guild_id}",
            put(put_link_guilds_id).delete(delete_link_guilds_id),
        )
        .layer(CorsLayer::permissive())
}

async fn put_link_guilds_id(
    Path((user_id, guild_id)): Path<(Id<UserMarker>, Id<GuildMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if current_user.id != user_id || !is_client_guild_member(guild_id, &discord_user).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let new_link_guild = LinkGuild {
        guild_id,
        enabled: true,
    };

    let mut user = User::from_db(user_id, &app_state.pg_pool)
        .await
        .unwrap();
    if user
        .link_guilds
        .iter()
        .any(|g| g.guild_id == guild_id && !g.enabled)
    {
        // Already exists
        return Ok(Json(json!(new_link_guild)));
    }
    user.link_guilds.retain(|g| g.guild_id != guild_id);
    user.link_guilds.push(new_link_guild.clone());
    user.save(&app_state.pg_pool).await;

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool).await.unwrap();

    if guild.verify.user_links.contains_key(&user.user_id) {
        // Already exists
        return Ok(Json(json!(new_link_guild)));
    }

    guild.verify.user_links.insert(user_id, user.links.clone());

    for verify_role in &mut guild.verify.roles {
        if link_arr_match(&user.links, &verify_role.pattern) {
            add_guild_member_role(
                guild_id,
                user_id,
                verify_role.role_id,
                &app_state.discord_bot,
            )
            .await?;
            verify_role.members += 1;
        }
    }

    guild.save(&app_state.pg_pool).await;
    Ok(Json(json!(new_link_guild)))
}

async fn delete_link_guilds_id(
    Path((user_id, guild_id)): Path<(Id<UserMarker>, Id<GuildMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    if current_user.id != user_id || !is_client_guild_member(guild_id, &discord_user).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut user = User::from_db(user_id, &app_state.pg_pool)
        .await
        .unwrap();

    user.link_guilds.retain(|g| g.guild_id != guild_id);

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool).await.unwrap();

    for role in &mut guild.verify.roles {
        if link_arr_match(&user.links, &role.pattern) {
            remove_guild_member_role(guild_id, user_id, role.role_id, &app_state.discord_bot)
                .await?;
            if role.members > 0 {
                role.members -= 1;
            }
        }
    }
    guild.verify.user_links.remove(&user_id);

    user.save(&app_state.pg_pool).await;
    guild.save(&app_state.pg_pool).await;
    Ok(StatusCode::NO_CONTENT)
}
async fn is_client_guild_member(guild_id: Id<GuildMarker>, client: &twilight_http::Client) -> bool {
    get_current_user_guild(guild_id, client).await.is_ok()
}
