mod links;
pub mod models;

use crate::AppState;
use crate::users::models::{LinkGuild, User};
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Extension, Json};
use http::StatusCode;
use lambda_http::tracing::info;
use serde::Deserialize;
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;
use twilight_model::id::Id;
use twilight_model::id::marker::{UserMarker};
use twilight_model::user::CurrentUser;
use crate::guilds::models::Guild;
use crate::guilds::tasks::{assign_roles_guild_user_link};
use crate::utils::member_guilds;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(get_users))
        .route("/@me", get(get_users_me).put(put_users_me).post(post_users_me))
        .route("/{user_id}", get(get_users_id).put(put_users_id).post(post_users_id))
        .nest("/{user_id}/links", links::router())
        .layer(CorsLayer::permissive())
}

async fn get_users() -> Json<Value> {
    todo!()
}

async fn get_users_me(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    get_users_id(Path(current_user.id), Extension(current_user), State(app_state)).await
}

async fn put_users_me(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(user_req): Json<PutUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    put_users_id(Path(current_user.id), Extension(current_user), State(app_state), Json(user_req)).await
}

async fn get_users_id(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // Authorize
    if current_user.id.ne(&user_id) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Fetch user from DynamoDB
    let result = User::from_db(&user_id.to_string(), &app_state.dynamo)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(json!(result)))
}

#[derive(Deserialize)]
struct PutUserRequest {
    link_guilds: Vec<LinkGuild>,
}

async fn put_users_id(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(user_req): Json<PutUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    if current_user.id.ne(&user_id) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    // Write user to DynamoDB
    let mut user = User::from_db(&user_id.to_string(), &app_state.dynamo)
        .await
        .unwrap_or_else(|| User {
            user_id,
            ..Default::default()
        });
    
    let link_guilds_req_map = user_req.link_guilds.iter().map(|lg| (&lg.guild_id, lg)).collect::<std::collections::HashMap<_, _>>();

    let mut changed_guilds = vec![];

    for i in 0..user.link_guilds.len() {
        let link_guild_req = link_guilds_req_map.get(&user.link_guilds[i].guild_id);
        if user.link_guilds[i].enabled != link_guild_req.unwrap().enabled {
            user.link_guilds[i].enabled = link_guild_req.unwrap().enabled;
            changed_guilds.push(user.link_guilds[i].guild_id);
        }
    }

    info!("Changed guilds: {changed_guilds:?}");
    for guild_id in changed_guilds {
        let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap();
        guild.verify.user_links.retain(|u, _| u != &user.user_id);
        let mut link_enabled = false;
        if user.link_guilds.iter().find(|lg| lg.guild_id == guild_id).unwrap().enabled {
            link_enabled = true;
            guild.verify.user_links.insert(user.user_id, user.links.clone());
        }

        info!("Saving guild {} {:?}", guild.guild_id.to_string(), guild.verify.user_links.len());
        for user_link in &user.links {
            assign_roles_guild_user_link(
                link_enabled,
                &user_link.link_address,
                user_id,
                &mut guild,
                &app_state.discord_bot,
            ).await?;

        }
        guild.save(&app_state.dynamo).await;
    }
    
    user.save(&app_state.dynamo).await;

    Ok(Json(json!(user)))
}

async fn post_users_me(
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    post_users_id(Path(current_user.id), Extension(current_user), State(app_state)).await
}

async fn post_users_id(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if current_user.id.ne(&user_id) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut member_guilds = member_guilds(&current_user, &app_state.discord_bot).await?;
    let mut user = User::from_db(&user_id.to_string(), &app_state.dynamo).await.unwrap_or_else(|| User {
        user_id,
        ..Default::default()
    });
    // Set Discord Values
    user.global_name = current_user.name;
    user.avatar = current_user.avatar;

    // Set Link Guilds
    for i in 0..user.link_guilds.len() {
        if member_guilds.iter().any(|g| g.id == user.link_guilds[i].guild_id) {
            member_guilds.retain(|g| g.id != user.link_guilds[i].guild_id);
        } else {
            let member_guild = member_guilds.iter().find(|g| g.id == user.link_guilds[i].guild_id);
            if member_guild.is_some() {
                user.link_guilds[i].name = member_guild.unwrap().name.clone();
                user.link_guilds[i].icon = member_guild.unwrap().icon;
            }
        }
    }
    let mut additional_link_guilds = member_guilds.iter().map(|g| LinkGuild {
        guild_id: g.id,
        name: g.name.clone(),
        icon: g.icon,
        enabled: true,
    }).collect::<Vec<LinkGuild>>();
    user.link_guilds.append(additional_link_guilds.as_mut());
    user.save(&app_state.dynamo).await;
    Ok(Json(json!(user)))
}