use std::sync::Arc;
use axum::extract::{Path, State};
use crate::AppState;
use axum::{Extension, Json};
use axum::routing::{post, put};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker};
use twilight_model::user::CurrentUser;
use crate::discord::{add_guild_member_role, remove_guild_member_role};
use crate::guilds::models::Guild;
use crate::guilds::verify::models::{Verify, VerifyRole};
use crate::users::utils::link_arr_match;
use crate::utils::is_client_admin_guild;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/recon", post(post_recon))
        .route("/roles/{role_id}", put(put_roles_id).delete(delete_roles_id))
        .layer(CorsLayer::permissive())
}

#[derive(Serialize, Deserialize)]
struct PutRoleRequest {
    pub pattern: String,
}

async fn put_roles_id(
    Path((guild_id,role_id)): Path<(Id<GuildMarker>,Id<RoleMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(put_role_request): Json<PutRoleRequest>
) -> Result<Json<Value>, StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap();
    
    if guild.verify.roles.iter().any(|r| r.role_id == role_id) {
        remove_existing_role(&mut guild, role_id, &app_state).await?;
    }
    
    let mut new_role = VerifyRole {
        role_id,
        pattern: put_role_request.pattern,
        ..Default::default()
    };

    for (user_id, user_links) in &guild.verify.user_links {
        if link_arr_match(user_links, &new_role.pattern) {
            add_guild_member_role(guild.guild_id, *user_id, role_id, &app_state.discord_bot).await?;
            new_role.members += 1;
        }
    }
    guild.verify.roles.push(new_role);
    guild.save(&app_state.dynamo).await;
    
    Ok(Json(json!(guild.verify.roles.iter().find(|r| r.role_id == role_id).unwrap())))
}


async fn delete_roles_id(
    Path((guild_id,role_id)): Path<(Id<GuildMarker>,Id<RoleMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap();

    remove_existing_role(&mut guild, role_id, &app_state).await?;

    guild.save(&app_state.dynamo).await;

    Ok(StatusCode::NO_CONTENT)
}

async fn remove_existing_role(
    guild: &mut Guild,
    role_id: Id<RoleMarker>,
    app_state: &AppState
) -> Result<(), StatusCode> {
    let existing_role_opt = guild.verify.roles.iter().find(|r| r.role_id == role_id);

    if existing_role_opt.is_none() {
        // No role found
        return Ok(())
    }
    let existing_role = existing_role_opt.unwrap();

    for (user_id, user_links) in &guild.verify.user_links {
        if link_arr_match(user_links, &existing_role.pattern) {
            remove_guild_member_role(guild.guild_id, *user_id, role_id, &app_state.discord_bot).await?
        }
    }

    guild.verify.roles.retain(|r| r.role_id != role_id);
    Ok(())
}

async fn post_recon(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    if discord_user.token() != app_state.discord_bot.token() {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap();
    let Verify{roles, user_links} = &mut guild.verify;

    for role in roles {
        role.members = 0;
        for (user_id, links) in &*user_links {
            if link_arr_match(links, &role.pattern) {
                add_guild_member_role(guild.guild_id, *user_id, role.role_id, &app_state.discord_bot).await?;
                role.members += 1;
            } else {
                remove_guild_member_role(guild.guild_id, *user_id, role.role_id, &app_state.discord_bot).await?;
            }
        }
    }
    guild.save(&app_state.dynamo).await;
    
    Ok(StatusCode::NO_CONTENT)
}