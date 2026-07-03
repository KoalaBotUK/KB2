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
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, MessageMarker, RoleMarker, UserMarker};
use twilight_model::user::User;

/// Fields pulled out of an [`Interaction`] that are required to process a
/// vote component interaction, but which Discord does not guarantee will be
/// present on every interaction payload.
#[derive(Debug, PartialEq)]
struct VoteInteractionFields {
    guild_id: Id<GuildMarker>,
    message_id: Id<MessageMarker>,
    user_id: Id<UserMarker>,
    roles: Vec<Id<RoleMarker>>,
}

/// Extracts the fields needed to process a vote component interaction.
///
/// Discord does not guarantee `guild_id`, `member`, or `message` will be set
/// on every interaction, and `member.user` is itself optional (falling back
/// to the top-level `user` field for DM-context interactions). Previously
/// these were all `.unwrap()`ed directly, which meant a malformed or
/// unexpected interaction payload from Discord would panic the process.
/// Each is now validated and turned into a `BAD_REQUEST` instead.
fn extract_vote_interaction_fields(
    guild_id: Option<Id<GuildMarker>>,
    member: Option<PartialMember>,
    message_id: Option<Id<MessageMarker>>,
    user: Option<User>,
) -> Result<VoteInteractionFields, StatusCode> {
    let message_id = message_id.ok_or(StatusCode::BAD_REQUEST)?;
    let PartialMember {
        user: member_user,
        roles,
        ..
    } = member.ok_or(StatusCode::BAD_REQUEST)?;
    let user_id = member_user.or(user).ok_or(StatusCode::BAD_REQUEST)?.id;
    let guild_id = guild_id.ok_or(StatusCode::BAD_REQUEST)?;

    Ok(VoteInteractionFields {
        guild_id,
        message_id,
        user_id,
        roles,
    })
}

pub(crate) async fn handle_component_interaction(
    app_state: AppState,
    interaction: Interaction,
) -> Result<Json<Value>, StatusCode> {
    let Interaction {
        data,
        guild_id,
        member,
        message,
        user,
        ..
    } = interaction;
    let data = match data {
        Some(InteractionData::MessageComponent(data)) => Ok(data),
        _ => Err(StatusCode::BAD_REQUEST),
    }?;
    let VoteInteractionFields {
        guild_id,
        message_id,
        user_id,
        roles,
    } = extract_vote_interaction_fields(guild_id, member, message.map(|m| m.id), user)?;

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
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
    guild.save(&app_state.pg_pool).await?;

    Ok(Json(json!(InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(responses.join("\n")),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use twilight_model::guild::MemberFlags;

    /// Builds a minimal, otherwise-empty Discord user with the given id.
    fn test_user(id: u64) -> User {
        User {
            accent_color: None,
            avatar: None,
            avatar_decoration: None,
            avatar_decoration_data: None,
            banner: None,
            bot: false,
            discriminator: 0,
            email: None,
            flags: None,
            global_name: None,
            id: Id::new(id),
            locale: None,
            mfa_enabled: None,
            name: "test-user".to_string(),
            premium_type: None,
            public_flags: None,
            system: None,
            verified: None,
        }
    }

    /// Builds a minimal, otherwise-empty partial member with the given user
    /// and roles.
    fn test_member(user: Option<User>, roles: Vec<Id<RoleMarker>>) -> PartialMember {
        PartialMember {
            avatar: None,
            communication_disabled_until: None,
            deaf: false,
            flags: MemberFlags::empty(),
            joined_at: None,
            mute: false,
            nick: None,
            permissions: None,
            premium_since: None,
            roles,
            user,
        }
    }

    #[test]
    fn missing_guild_id_returns_bad_request() {
        let result = extract_vote_interaction_fields(
            None,
            Some(test_member(Some(test_user(1)), vec![])),
            Some(Id::new(1)),
            None,
        );

        assert_eq!(result, Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn missing_member_returns_bad_request() {
        let result = extract_vote_interaction_fields(
            Some(Id::new(1)),
            None,
            Some(Id::new(1)),
            Some(test_user(2)),
        );

        assert_eq!(result, Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn missing_message_returns_bad_request() {
        let result = extract_vote_interaction_fields(
            Some(Id::new(1)),
            Some(test_member(Some(test_user(1)), vec![])),
            None,
            None,
        );

        assert_eq!(result, Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn missing_member_user_and_interaction_user_returns_bad_request() {
        let result = extract_vote_interaction_fields(
            Some(Id::new(1)),
            Some(test_member(None, vec![])),
            Some(Id::new(1)),
            None,
        );

        assert_eq!(result, Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn falls_back_to_interaction_user_when_member_user_missing() {
        // Discord omits `member.user` in some contexts; the top-level
        // `interaction.user` field should be used instead in that case.
        let fields = extract_vote_interaction_fields(
            Some(Id::new(10)),
            Some(test_member(None, vec![Id::new(5)])),
            Some(Id::new(20)),
            Some(test_user(99)),
        )
        .unwrap();

        assert_eq!(
            fields,
            VoteInteractionFields {
                guild_id: Id::new(10),
                message_id: Id::new(20),
                user_id: Id::new(99),
                roles: vec![Id::new(5)],
            }
        );
    }

    #[test]
    fn well_formed_payload_extracts_all_fields() {
        // When `member.user` is present it takes precedence over the
        // top-level `user` fallback.
        let fields = extract_vote_interaction_fields(
            Some(Id::new(10)),
            Some(test_member(Some(test_user(42)), vec![Id::new(7)])),
            Some(Id::new(20)),
            Some(test_user(999)),
        )
        .unwrap();

        assert_eq!(
            fields,
            VoteInteractionFields {
                guild_id: Id::new(10),
                message_id: Id::new(20),
                user_id: Id::new(42),
                roles: vec![Id::new(7)],
            }
        );
    }
}
