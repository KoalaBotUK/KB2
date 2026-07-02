use crate::discord::{create_message, get_guild_channels, ise, update_message};
use crate::guilds::models::Guild;
use crate::guilds::votes::models::{RoleListType, VoteOption, VoteVote};
use crate::guilds::votes::utils::{VoteOptionComponent, channel_belongs_to_guild, group_to_rows};
use crate::{AppState, utils};
use aws_sdk_scheduler::types::{FlexibleTimeWindow, FlexibleTimeWindowMode, Target};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::{HeaderMap, Method, StatusCode};
use lambda_http::Context;
use lambda_http::aws_lambda_events::apigw::ApiGatewayProxyRequest;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use twilight_model::channel::message::{Component, EmojiReactionType};
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker};
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
    role_list_type: RoleListType,
    is_multi_select: bool,
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

    let guild_channels = get_guild_channels(guild_id, &app_state.discord_bot).await?;
    let guild_channel_ids: Vec<_> = guild_channels.iter().map(|c| c.id).collect();
    if !channel_belongs_to_guild(vote_req.channel_id, &guild_channel_ids) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let message = create_message(
        vote_req.channel_id,
        Some(&format!("# {}\n{}\n", vote_req.title, vote_req.description)),
        Some(
            group_to_rows(vote_req.options.iter().map(|o| o.to_component()).collect())?.as_slice(),
        ),
        &app_state.discord_bot,
    )
    .await?;

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });
    let new_vote = VoteVote {
        message_id: message.id,
        title: vote_req.title.clone(),
        description: vote_req.description.clone(),
        options: vote_req
            .options
            .iter()
            .map(|o| VoteOption {
                emoji: o.emoji.clone(),
                label: o.label.clone(),
                ..Default::default()
            })
            .collect(),
        channel_id: vote_req.channel_id,
        close_at: vote_req.close_at,
        open: true,
        role_list: vote_req.role_list,
        role_list_type: vote_req.role_list_type,
        is_multi_select: vote_req.is_multi_select,
    };
    guild.vote.votes.push(new_vote.clone());
    guild.save(&app_state.pg_pool).await;

    if let Some(close_at) = vote_req.close_at {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            // The scheduled callback must authenticate as the bot: the `Discord`
            // auth scheme builds a `Bearer` client and explicitly rejects bots,
            // so it can never succeed here.
            format!(
                "Bot {}",
                std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set")
            )
            .parse()
            .unwrap(),
        );

        let payload = ApiGatewayProxyRequest {
            resource: Some("/{proxy+}".parse().unwrap()),
            path: Some(
                format!("/guilds/{}/votes/{}/close", guild_id, message.id)
                    .parse()
                    .unwrap(),
            ),
            http_method: Method::POST,
            headers,
            path_parameters: HashMap::from_iter(vec![(
                "proxy".parse().unwrap(),
                format!("/guilds/{}/votes/{}", guild_id, message.id)
                    .parse()
                    .unwrap(),
            )]),
            ..Default::default()
        };

        // `context.identity` is only populated for Cognito-identity invocations
        // (None behind API Gateway/function URLs) and, even when present, is not
        // an IAM role ARN. The EventBridge Scheduler execution role is instead
        // supplied via configuration.
        let scheduler_role_arn =
            std::env::var("SCHEDULER_ROLE_ARN").expect("SCHEDULER_ROLE_ARN must be set");

        let target = Target::builder()
            .arn(context.invoked_function_arn)
            .role_arn(scheduler_role_arn)
            .input(json!({"payload": payload, "context": Context::default()}).to_string())
            .build()
            .map_err(ise)?;

        let _result = app_state
            .scheduler
            .create_schedule()
            .name(vote_close_schedule_name(message.id))
            .schedule_expression(vote_close_schedule_expression(close_at))
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

/// EventBridge Scheduler schedule names must match `[0-9a-zA-Z\-_.]{1,64}`.
/// Spaces (as in the original `"KB2 Vote Close {id}"`) are invalid.
fn vote_close_schedule_name(message_id: Id<MessageMarker>) -> String {
    format!("kb2-vote-close-{}", message_id)
}

/// EventBridge Scheduler's `at()` expression requires `yyyy-mm-ddThh:mm:ss`.
/// `DateTime`'s `Display` impl (e.g. `2026-07-02 12:00:00 UTC`) is not valid.
fn vote_close_schedule_expression(close_at: DateTime<Utc>) -> String {
    format!("at({})", close_at.format("%Y-%m-%dT%H:%M:%S"))
}

async fn get_votes_id(
    Path((guild_id, message_id)): Path<(Id<GuildMarker>, Id<MessageMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !utils::is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }

    let guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });
    let vote: &VoteVote = guild
        .vote
        .votes
        .iter()
        .find(|v| v.message_id == message_id)
        .unwrap();
    Ok(Json(json!(vote)))
}

/// Closes a vote and updates the Discord message. Shared by both the
/// admin-initiated close route and the EventBridge Scheduler callback.
async fn close_vote(
    guild_id: Id<GuildMarker>,
    message_id: Id<MessageMarker>,
    app_state: &AppState,
) -> Result<VoteVote, StatusCode> {
    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });
    let vote: &mut VoteVote = match guild
        .vote
        .votes
        .iter_mut()
        .find(|v| v.message_id == message_id)
    {
        Some(v) => v,
        None => return Err(StatusCode::NOT_FOUND),
    };
    vote.open = false;
    guild.save(&app_state.pg_pool).await;

    let vote: &VoteVote = guild
        .vote
        .votes
        .iter()
        .find(|v| v.message_id == message_id)
        .unwrap();
    let _message = update_message(
        vote.channel_id,
        vote.message_id,
        Some(Some(&format!(
            "# [CLOSED] {}\n{}\n",
            vote.title, vote.description
        ))),
        Some(Some(
            group_to_rows(
                vote.options
                    .iter()
                    .map(|o| {
                        let mut c = o.to_component();
                        if let Component::Button(ref mut b) = c {
                            b.disabled = true
                        }
                        c
                    })
                    .collect(),
            )?
            .as_slice(),
        )),
        &app_state.discord_bot,
    )
    .await?;

    Ok(vote.clone())
}

async fn post_votes_id_close(
    Path((guild_id, message_id)): Path<(Id<GuildMarker>, Id<MessageMarker>)>,
    Extension(discord_client): Extension<Arc<twilight_http::Client>>,
    current_user: Option<Extension<CurrentUser>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    // This route is hit two ways: by an admin closing a vote early (`Discord`
    // auth scheme, with a `CurrentUser` extension inserted by auth_middleware),
    // and by the EventBridge Scheduler callback closing it automatically
    // (`Bot` auth scheme, which never inserts a `CurrentUser`). Follow the
    // `post_recon` pattern for the latter: compare the authenticated client's
    // token to the bot's own token rather than requiring `Extension<CurrentUser>`.
    let is_bot_callback = discord_client.token() == app_state.discord_bot.token();
    if !is_bot_callback {
        let Extension(current_user) = current_user.ok_or(StatusCode::UNAUTHORIZED)?;
        if !utils::is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let vote = close_vote(guild_id, message_id, &app_state).await?;

    Ok(Json(json!(vote)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // EventBridge Scheduler schedule names must match `[0-9a-zA-Z\-_.]{1,64}`.
    fn is_valid_schedule_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 64
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }

    #[test]
    fn schedule_name_contains_no_spaces_and_uses_message_id() {
        let message_id: Id<MessageMarker> = Id::new(1234567890123456789);
        let name = vote_close_schedule_name(message_id);

        assert_eq!(name, "kb2-vote-close-1234567890123456789");
        assert!(
            is_valid_schedule_name(&name),
            "schedule name {name:?} does not match EventBridge Scheduler's allowed pattern"
        );
    }

    #[test]
    fn schedule_expression_uses_at_format_without_timezone_suffix() {
        let close_at = Utc.with_ymd_and_hms(2026, 7, 2, 12, 0, 0).unwrap();

        let expr = vote_close_schedule_expression(close_at);

        // Must be `at(yyyy-mm-ddThh:mm:ss)`, not `DateTime`'s
        // `Display` output (`2026-07-02 12:00:00 UTC`).
        assert_eq!(expr, "at(2026-07-02T12:00:00)");
    }

    #[test]
    fn schedule_expression_zero_pads_single_digit_components() {
        let close_at = Utc.with_ymd_and_hms(2026, 1, 5, 3, 4, 5).unwrap();

        let expr = vote_close_schedule_expression(close_at);

        assert_eq!(expr, "at(2026-01-05T03:04:05)");
    }
}
