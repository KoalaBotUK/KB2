use crate::AppState;
use crate::guilds::models::Guild;
use crate::guilds::votes::models::{RoleListType, VoteVote};
use crate::guilds::votes::utils::VoteOptionComponent;
use axum::Json;
use http::StatusCode;
use serde_json::{Value, json};
use std::default::Default;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::guild::PartialMember;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};

pub(crate) async fn handle_component_interaction(
    app_state: AppState,
    interaction: Interaction,
) -> Result<Json<Value>, StatusCode> {
    let data = match interaction.data {
        Some(InteractionData::MessageComponent(data)) => Ok(data),
        _ => Err(StatusCode::BAD_REQUEST),
    }?;
    let Interaction {
        guild_id,
        member,
        message,
        ..
    } = interaction;
    let message_id = message.unwrap().id;
    let PartialMember { user, roles, .. } = member.unwrap();
    let user_id = user.unwrap().id;

    let mut guild = Guild::from_db(guild_id.unwrap(), &app_state.pg_pool)
        .await
        .unwrap();

    let VoteVote {
        options,
        role_list,
        role_list_type,
        is_multi_select,
        ..
    } = match guild
        .vote
        .votes
        .iter_mut()
        .find(|v| v.message_id == message_id)
    {
        Some(v) => v,
        None => return Err(StatusCode::NOT_FOUND),
    };

    let role_in_role_list = roles.iter().any(|r| role_list.contains(r));

    let allowed = match role_list_type {
        RoleListType::Blacklist => !role_in_role_list,
        RoleListType::Whitelist => role_in_role_list,
    };
    if !allowed {
        return Ok(Json(json!(InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(
                    format!(
                        "You do not have permission to vote due to the {}",
                        role_list_type.to_string().to_lowercase()
                    )
                    .to_string()
                ),
                flags: Some(MessageFlags::EPHEMERAL),
                ..Default::default()
            }),
        })));
    }
    let mut responses = vec![];

    for o in options.iter_mut() {
        if o.users.contains(&user_id) && (data.custom_id == o.custom_id() || !*is_multi_select) {
            o.users.remove(&user_id);
            responses.push(format!("You have removed your vote for {o}."));
        } else if !o.users.contains(&user_id) && data.custom_id == o.custom_id() {
            o.users.insert(user_id);
            responses.push(format!("You have added a vote for {o}."));
        }
    }
    guild.save(&app_state.pg_pool).await;

    Ok(Json(json!(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(responses.join("\n")),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    })))
}
