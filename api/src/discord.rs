use cached::proc_macro::cached;
use http::StatusCode;
use lambda_http::tracing::{error, warn};
use std::time::Duration;
use tokio::time::sleep;
use twilight_http::api_error::ApiError;
use twilight_http::error::ErrorType;
use twilight_http::{Client, Error, Response};
use twilight_model::channel::message::Component;
use twilight_model::channel::{Channel, Message};
use twilight_model::guild::{Guild, Member, Role};
use twilight_model::id::Id;
use twilight_model::id::marker::{
    ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker,
};
use twilight_model::user::{CurrentUser, CurrentUserGuild, User};

pub fn ise<T: std::fmt::Debug>(e: T) -> StatusCode {
    error!("Internal Server Error: {:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn retry_on_rl<T, Fut, R>(mut fut: T) -> Result<Response<R>, Error>
where
    T: FnMut() -> Fut,
    Fut: Future<Output = Result<Response<R>, Error>>,
{
    let mut attempts = 0;
    loop {
        match fut().await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                if attempts > 3 {
                    return Err(e);
                }

                match e.kind() {
                    ErrorType::Response {
                        error: ApiError::Ratelimited(ratelimited),
                        ..
                    } => {
                        attempts += 1;
                        warn!(
                            "Rate limited, retrying in {} seconds",
                            ratelimited.retry_after
                        );
                        sleep(Duration::from_secs_f64(ratelimited.retry_after)).await;
                        continue;
                    }
                    _ => return Err(e),
                }
            }
        }
    }
}

pub fn as_http_err(e: Error) -> StatusCode {
    match e.kind() {
        ErrorType::Response { status, error, .. } => {
            error!("Discord Api Error: {} {:?}", status, error);
            StatusCode::from_u16(status.get()).map_err(ise).unwrap()
        }
        _ => ise(e),
    }
}

#[cached(
    time = 180,
    key = "String",
    convert = r##"{ format!("{:?}", client.token().unwrap()) }"##
)]
pub async fn get_current_user_guilds(client: &Client) -> Result<Vec<CurrentUserGuild>, StatusCode> {
    retry_on_rl(|| async { client.current_user_guilds().await })
        .await
        .map_err(as_http_err)?
        .models()
        .await
        .map_err(ise)
}

/// Selects the guild matching `guild_id` out of a batch of the current
/// user's guilds returned by Discord.
///
/// Discord's `after`/`limit` pagination on `Get Current User Guilds` only
/// narrows the search space to "the next guild after this ID" — it does not
/// guarantee that guild is actually the one requested (the user may not be a
/// member of `guild_id` at all, in which case Discord returns whichever
/// guild the client *is* in with the next-highest snowflake). Callers must
/// therefore verify the ID before trusting any fields (owner/permissions)
/// on the returned guild, otherwise an authorization check can silently
/// evaluate a different guild than the one requested.
///
/// Returns `None` — never panics — if none of the supplied guilds match.
fn select_current_user_guild(
    guilds: Vec<CurrentUserGuild>,
    guild_id: Id<GuildMarker>,
) -> Option<CurrentUserGuild> {
    guilds.into_iter().find(|g| g.id == guild_id)
}

#[cached(
    time = 180,
    key = "String",
    convert = r##"{ format!("{guild_id}{:?}", client.token().unwrap()) }"##
)]
pub async fn get_current_user_guild(
    guild_id: Id<GuildMarker>,
    client: &Client,
) -> Result<Option<CurrentUserGuild>, StatusCode> {
    let guilds = retry_on_rl(|| async {
        client
            .current_user_guilds()
            .after(Id::new(guild_id.get() - 1))
            .limit(1)
            .await
    })
    .await
    .map_err(as_http_err)?
    .models()
    .await
    .map_err(ise)?;

    Ok(select_current_user_guild(guilds, guild_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use twilight_model::guild::Permissions;

    fn guild(id: u64, owner: bool) -> CurrentUserGuild {
        CurrentUserGuild {
            id: Id::new(id),
            name: format!("guild-{id}"),
            icon: None,
            owner,
            permissions: if owner {
                Permissions::ADMINISTRATOR
            } else {
                Permissions::empty()
            },
            features: vec![],
        }
    }

    #[test]
    fn returns_the_matching_guild_when_user_has_multiple_guilds() {
        let guilds = vec![guild(100, false), guild(200, true), guild(300, false)];

        let found = select_current_user_guild(guilds, Id::new(200));

        assert_eq!(found.map(|g| g.id), Some(Id::new(200)));
    }

    #[test]
    fn returns_none_instead_of_panicking_when_guild_not_found() {
        let guilds = vec![guild(100, false), guild(300, false)];

        // Regression: previously `.pop().unwrap()` on an unmatched batch
        // would panic (HTTP 500) instead of yielding a clean "not a member"
        // result.
        let found = select_current_user_guild(guilds, Id::new(200));

        assert!(found.is_none());
    }

    #[test]
    fn returns_none_when_no_guilds_at_all() {
        let found = select_current_user_guild(vec![], Id::new(200));

        assert!(found.is_none());
    }

    #[test]
    fn does_not_leak_data_from_a_different_guild() {
        // Regression: `.after(guild_id - 1).limit(1)` can return the next
        // guild by snowflake order when the user isn't a member of the
        // requested guild. The selector must not treat that guild as a
        // match, or an admin of guild `999` would be authorized for the
        // unrelated, higher-privileged guild `1000`.
        let other_guild = guild(1000, true);
        let guilds = vec![other_guild.clone()];

        let found = select_current_user_guild(guilds, Id::new(999));

        assert!(found.is_none());
        assert_ne!(found, Some(other_guild));
    }
}

pub async fn add_guild_member_role(
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
    client: &Client,
) -> Result<(), StatusCode> {
    retry_on_rl(|| async {
        client
            .add_guild_member_role(guild_id, user_id, role_id)
            .await
    })
    .await
    .map_err(as_http_err)?;
    Ok(())
}

pub async fn remove_guild_member_role(
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
    client: &Client,
) -> Result<(), StatusCode> {
    retry_on_rl(|| async {
        client
            .remove_guild_member_role(guild_id, user_id, role_id)
            .await
    })
    .await
    .map_err(as_http_err)?;
    Ok(())
}

#[cached(
    time = 3600,
    key = "String",
    convert = r##"{ format!("{:?}", client.token().unwrap()) }"##
)]
pub async fn get_current_user(client: &Client) -> Result<CurrentUser, StatusCode> {
    retry_on_rl(|| async { client.current_user().await })
        .await
        .map_err(as_http_err)?
        .model()
        .await
        .map_err(ise)
}

#[cached(
    time = 3600,
    key = "String",
    convert = r##"{ format!("{user_id}{:?}", client.token().unwrap()) }"##
)]
pub async fn get_user(user_id: Id<UserMarker>, client: &Client) -> Result<User, StatusCode> {
    retry_on_rl(|| async { client.user(user_id).await })
        .await
        .map_err(as_http_err)?
        .model()
        .await
        .map_err(ise)
}

#[cached(
    time = 60,
    key = "String",
    convert = r##"{ format!("{guild_id}{:?}", client.token().unwrap()) }"##
)]
pub async fn get_guild(guild_id: Id<GuildMarker>, client: &Client) -> Result<Guild, StatusCode> {
    retry_on_rl(|| async { client.guild(guild_id).await })
        .await
        .map_err(as_http_err)?
        .model()
        .await
        .map_err(ise)
}

#[cached(
    time = 60,
    key = "String",
    convert = r##"{ format!("{guild_id}{:?}", client.token().unwrap()) }"##
)]
pub async fn get_guild_channels(
    guild_id: Id<GuildMarker>,
    client: &Client,
) -> Result<Vec<Channel>, StatusCode> {
    retry_on_rl(|| async { client.guild_channels(guild_id).await })
        .await
        .map_err(as_http_err)?
        .model()
        .await
        .map_err(ise)
}

#[cached(
    time = 60,
    key = "String",
    convert = r##"{ format!("{guild_id}{user_id}{:?}", client.token().unwrap()) }"##
)]
pub async fn get_guild_member(
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    client: &Client,
) -> Result<Member, StatusCode> {
    retry_on_rl(|| async { client.guild_member(guild_id, user_id).await })
        .await
        .map_err(as_http_err)?
        .model()
        .await
        .map_err(ise)
}

#[cached(
    time = 60,
    key = "String",
    convert = r##"{ format!("{guild_id}{role_id}{:?}", client.token().unwrap()) }"##
)]
pub async fn get_guild_role(
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
    client: &Client,
) -> Result<Role, StatusCode> {
    retry_on_rl(|| async { client.role(guild_id, role_id).await })
        .await
        .map_err(as_http_err)?
        .model()
        .await
        .map_err(ise)
}

pub async fn create_message(
    channel_id: Id<ChannelMarker>,
    content: Option<&str>,
    components: Option<&[Component]>,
    client: &Client,
) -> Result<Message, StatusCode> {
    retry_on_rl(|| async {
        let mut msg_builder = client.create_message(channel_id);
        if let Some(content) = content {
            msg_builder = msg_builder.content(content);
        }
        if let Some(components) = components {
            msg_builder = msg_builder.components(components);
        }
        msg_builder.await
    })
    .await
    .map_err(as_http_err)?
    .model()
    .await
    .map_err(ise)
}

pub async fn update_message(
    channel_id: Id<ChannelMarker>,
    message_id: Id<MessageMarker>,
    content: Option<Option<&str>>,
    components: Option<Option<&[Component]>>,
    client: &Client,
) -> Result<Message, StatusCode> {
    retry_on_rl(|| async {
        let mut msg_builder = client.update_message(channel_id, message_id);
        if let Some(content) = content {
            msg_builder = msg_builder.content(content);
        }
        if let Some(components) = components {
            msg_builder = msg_builder.components(components);
        }
        msg_builder.await
    })
    .await
    .map_err(as_http_err)?
    .model()
    .await
    .map_err(ise)
}
