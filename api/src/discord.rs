use std::time::Duration;
use cached::proc_macro::cached;
use http::StatusCode;
use lambda_http::tracing::{error, warn};
use tokio::time::sleep;
use twilight_http::{Client, Error, Response};
use twilight_http::api_error::ApiError;
use twilight_http::error::ErrorType;
use twilight_model::channel::Message;
use twilight_model::channel::message::Component;
use twilight_model::guild::{Guild, Member, Role};
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker};
use twilight_model::user::{CurrentUser, CurrentUserGuild};

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
                    ErrorType::Response { error: ApiError::Ratelimited(ratelimited), ..} => {
                            attempts += 1;
                            warn!("Rate limited, retrying in {} seconds", ratelimited.retry_after);
                            sleep(Duration::from_secs_f64(ratelimited.retry_after)).await;
                            continue;
                        }
                    _ => return Err(e)
                }
            }
        }
    }
}

pub fn as_http_err(e: Error) -> StatusCode {
    match e.kind() {
        ErrorType::Response{ status, error, .. } => {
            error!("Discord Api Error: {} {:?}", status, error);
            StatusCode::from_u16(status.get()).map_err(ise).unwrap()
        },
        _ => {
            error!("Internal Server Error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[cached(time = 180, key = "String", convert = r##"{ format!("{:?}", client.token().unwrap()) }"##)]
pub async fn get_current_user_guilds(client: &Client) -> Result<Vec<CurrentUserGuild>,StatusCode> {
    retry_on_rl(|| async { client.current_user_guilds().await}).await.map_err(as_http_err)?.models().await.map_err(ise)
}

#[cached(time = 180, key = "String", convert = r##"{ format!("{guild_id}{:?}", client.token().unwrap()) }"##)]
pub async fn get_current_user_guild(guild_id: Id<GuildMarker>, client: &Client) -> Result<CurrentUserGuild,StatusCode> {
    Ok(retry_on_rl(|| async { client.current_user_guilds().after(Id::new(guild_id.get()-1)).limit(1).await}).await.map_err(as_http_err)?.models().await.map_err(ise)?.pop().unwrap())
}

#[cached(time = 60, key = "String", convert = r##"{ format!("{guild_id}{user_id}{role_id}{:?}", client.token().unwrap()) }"##)]
pub async fn add_guild_member_role(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>, role_id: Id<RoleMarker>, client: &Client) -> Result<(),StatusCode> {
    retry_on_rl(|| async { client.add_guild_member_role(guild_id, user_id, role_id).await}).await.map_err(as_http_err)?;
    Ok(())
}

#[cached(time = 60, key = "String", convert = r##"{ format!("{guild_id}{user_id}{role_id}{:?}", client.token().unwrap()) }"##)]
pub async fn remove_guild_member_role(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>, role_id: Id<RoleMarker>, client: &Client) -> Result<(),StatusCode> {
    retry_on_rl(|| async { client.remove_guild_member_role(guild_id, user_id, role_id).await}).await.map_err(as_http_err)?;
    Ok(())
}

#[cached(time = 3600, key = "String", convert = r##"{ format!("{:?}", client.token().unwrap()) }"##)]
pub async fn get_current_user(client: &Client) -> Result<CurrentUser,StatusCode> {
    retry_on_rl(|| async { client.current_user().await}).await.map_err(as_http_err)?.model().await.map_err(ise)
}

#[cached(time = 60, key = "String", convert = r##"{ format!("{guild_id}{:?}", client.token().unwrap()) }"##)]
pub async fn get_guild(guild_id: Id<GuildMarker>, client: &Client) -> Result<Guild,StatusCode> {
    retry_on_rl(|| async { client.guild(guild_id).await}).await.map_err(as_http_err)?.model().await.map_err(ise)
}

#[cached(time = 60, key = "String", convert = r##"{ format!("{guild_id}{user_id}{:?}", client.token().unwrap()) }"##)]
pub async fn get_guild_member(guild_id: Id<GuildMarker>, user_id: Id<UserMarker>, client: &Client) -> Result<Member,StatusCode> {
    retry_on_rl(|| async { client.guild_member(guild_id, user_id).await}).await.map_err(as_http_err)?.model().await.map_err(ise)
}

#[cached(time = 60, key = "String", convert = r##"{ format!("{guild_id}{role_id}{:?}", client.token().unwrap()) }"##)]
pub async fn get_guild_role(guild_id: Id<GuildMarker>, role_id: Id<RoleMarker>, client: &Client) -> Result<Role,StatusCode> {
    retry_on_rl(|| async { client.role(guild_id, role_id).await}).await.map_err(as_http_err)?.model().await.map_err(ise)
}

pub async fn create_message(channel_id: Id<ChannelMarker>, 
                            content: Option<&str>, 
                            components: Option<&[Component]>,
                            client: &Client) -> Result<Message,StatusCode> {
    retry_on_rl(|| async {
        let mut msg_builder = client.create_message(channel_id);
        if let Some(content) = content {
            msg_builder = msg_builder.content(content);
        }
        if let Some(components) = components {
            msg_builder = msg_builder.components(components);
        }
        msg_builder.await
    
    }).await.map_err(as_http_err)?.model().await.map_err(ise)
}

pub async fn update_message(channel_id: Id<ChannelMarker>,
                            message_id: Id<MessageMarker>,
                          content: Option<Option<&str>>,
                          components: Option<Option<&[Component]>>,
                          client: &Client) -> Result<Message,StatusCode> {
                          retry_on_rl(|| async {
                              let mut msg_builder = client.update_message(channel_id, message_id);
                              if let Some(content) = content {
                                  msg_builder = msg_builder.content(content);
                              }
                              if let Some(components) = components {
                                  msg_builder = msg_builder.components(components);
                              }
                              msg_builder.await
                          }).await.map_err(as_http_err)?.model().await.map_err(ise)
      }