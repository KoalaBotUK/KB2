use std::default::Default;
use axum::Json;
use http::StatusCode;
use serde_json::{json, Value};
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::guild::PartialMember;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use crate::AppState;
use crate::guilds::models::Guild;
use crate::guilds::votes::models::{RoleListType, VoteOption, VoteVote};
use crate::guilds::votes::utils::VoteOptionComponent;

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

    let mut guild = Guild::from_db(guild_id.unwrap(), &app_state.dynamo).await.unwrap();
    let VoteVote { options, role_list, role_list_type, .. }= guild.vote.votes.iter_mut().find(|v| v.message_id == message_id).unwrap();
    
    let role_in_role_list = roles.iter().any(|r| role_list.contains(r));
    let allowed = match role_list_type {
        RoleListType::BLACKLIST => {
            !role_in_role_list
        }
        RoleListType::WHITELIST => {
            role_in_role_list
        }
    };
    if !allowed {
        return Ok(Json(json!(
            InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(format!("You do not have permission to vote due to the {role_list_type}").to_string()),
                    flags: Some(MessageFlags::EPHEMERAL),
                    ..Default::default()
                }),
            }
        )));
    }
    
    let VoteOption { users, label, .. } = options.iter_mut().find(|o| o.custom_id() == data.custom_id).unwrap();
    let exists = users.iter().any(|&u| u == user_id);
    if exists {
        // Already voted, need to remove.
        users.retain(|&u| u != user_id);

    } else {
        users.insert(user_id);
    }
    let label = label.as_ref().unwrap();
    let content = Some(if exists {
        format!("You have removed your vote for {:?}.", label).to_string()
    } else {
        format!("You have voted for {:?}.", label).to_string()
    });
    guild.save(&app_state.dynamo).await;
    
    Ok(Json(json!(
        InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content,
                flags: Some(MessageFlags::EPHEMERAL),
                ..Default::default()
            }),
        }
    )))
}