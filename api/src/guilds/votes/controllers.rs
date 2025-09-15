use crate::discord::{create_message, ise, update_message};
use crate::guilds::models::Guild;
use crate::guilds::votes::models::{RoleListType, VoteOption, VoteVote};
use crate::guilds::votes::utils::{group_to_rows, VoteOptionComponent};
use crate::{utils, AppState};
use aws_sdk_scheduler::types::{FlexibleTimeWindow, FlexibleTimeWindowMode, Target};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::{HeaderMap, Method, StatusCode};
use lambda_http::aws_lambda_events::apigw::ApiGatewayProxyRequest;
use lambda_http::Context;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use tower_http::cors::CorsLayer;
use twilight_model::channel::message::{Component, EmojiReactionType};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker};
use twilight_model::id::Id;
use twilight_model::user::CurrentUser;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(post_votes))
        .route("/{message_id}", get(get_votes_id))
        .route("/{message_id}/close", post(post_votes_id_close))
        .layer(CorsLayer::permissive())
}

#[derive(Serialize, Deserialize)]
struct VoteOptionDto {
    emoji: Option<EmojiReactionType>,
    label: Option<String>,
}

impl VoteOptionComponent for VoteOptionDto {
    fn emoji(&self) -> &Option<EmojiReactionType> {
        &self.emoji
    }

    fn label(&self) -> &Option<String> {
        &self.label
    }
}

#[derive(Serialize, Deserialize)]
struct CreateVoteDto {
    title: String,
    description: String,
    options: Vec<VoteOptionDto>,
    channel_id: Id<ChannelMarker>,
    close_at: Option<DateTime<Utc>>,
    role_list: HashSet<Id<RoleMarker>>,
    role_list_type: RoleListType
}

async fn post_votes(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(context): Extension<Context>,
    State(app_state): State<AppState>,
    Json(vote_req): Json<CreateVoteDto>,
) -> Result<Json<Value>, StatusCode> {
    if !utils::is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let message = create_message(
        vote_req.channel_id,
        Some(&format!("# {}\n{}\n", vote_req.title, vote_req.description)),
        Some(group_to_rows(vote_req.options.iter().map(|o| o.to_component()).collect())?.as_slice()),
        &app_state.discord_bot,
    ).await?;

    let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap();
    let new_vote = VoteVote {
        message_id: message.id,
        title: vote_req.title.clone(),
        description: vote_req.description.clone(),
        options: vote_req.options.iter().map(|o| VoteOption { emoji: o.emoji.clone(), label: o.label.clone(), ..Default::default() }).collect(),
        channel_id: vote_req.channel_id,
        close_at: vote_req.close_at,
        open: true,
        role_list: vote_req.role_list,
        role_list_type: vote_req.role_list_type
    };
    guild.vote.votes.push(new_vote.clone());
    guild.save(&app_state.dynamo).await;

    if vote_req.close_at.is_some() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(AUTHORIZATION, format!("Discord {}", app_state.discord_bot.token().unwrap()).parse().unwrap());

        let payload = ApiGatewayProxyRequest {
            resource: Some("/{proxy+}".parse().unwrap()),
            path: Some(format!("/guilds/{}/votes/{}/close", guild_id, message.id).parse().unwrap()),
            http_method: Method::POST,
            headers,
            path_parameters: HashMap::from_iter(vec![("proxy".parse().unwrap(), format!("/guilds/{}/votes/{}", guild_id, message.id).parse().unwrap())]),
            ..Default::default()
        };

        let target = Target::builder()
            .arn(context.invoked_function_arn)
            .role_arn(context.identity.unwrap().identity_id)
            .input(json!({"payload": payload, "context": Context::default()}).to_string())
            .build()
            .map_err(ise)?;

        let _result = app_state
            .scheduler
            .create_schedule()
            .name(format!("KB2 Vote Close {}", message.id))
            .schedule_expression(format!("at({})", vote_req.close_at.unwrap()))
            .target(target)
            .flexible_time_window(
                FlexibleTimeWindow::builder()
                    .mode(FlexibleTimeWindowMode::Off)
                    .build()
                    .map_err(ise)?,
            )
            .send()
            .await
            .map_err(ise)?;
    }

    Ok(Json(json!(new_vote)))
}

async fn get_votes_id(
    Path((guild_id, message_id)): Path<(Id<GuildMarker>, Id<MessageMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !utils::is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap();
    let vote: &VoteVote = guild.vote.votes.iter().find(|v| v.message_id == message_id).unwrap();
    Ok(Json(json!(vote)))
}


async fn post_votes_id_close(
    Path((guild_id, message_id)): Path<(Id<GuildMarker>, Id<MessageMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !utils::is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut guild = Guild::from_db(guild_id, &app_state.dynamo).await.unwrap();
    let vote: &mut VoteVote = match guild.vote.votes.iter_mut().find(|v| v.message_id == message_id) {
        Some(v) => v,
        None => return Err(StatusCode::NOT_FOUND),
    };
    vote.open = false;
    guild.save(&app_state.dynamo).await;

    let vote: &VoteVote = guild.vote.votes.iter().find(|v| v.message_id == message_id).unwrap();
    let _message = update_message(vote.channel_id, vote.message_id, None, Some(Some(group_to_rows(vote.options.iter().map(|o| {
        let mut c = o.to_component();
        match c {
            Component::Button(ref mut b) => b.disabled = true,
            _ => (),
        }
        c
    }).collect())?.as_slice())), &app_state.discord_bot).await?;

    Ok(Json(json!(vote)))
}