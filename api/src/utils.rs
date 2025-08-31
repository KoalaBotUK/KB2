use twilight_http::{Client, Error, Response};
use tokio::time::sleep;
use std::time::Duration;
use http::StatusCode;
use lambda_http::tracing::error;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUser;

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
                let err_str = format!("{e}");
                if err_str.contains("429") && attempts < 3 {
                    attempts += 1;
                    sleep(Duration::from_secs(1)).await; // Default 1s
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

pub fn ise<T: std::fmt::Debug>(e: T) -> StatusCode {
    error!("Internal Server Error: {:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn member_guilds(
    current_user: &CurrentUser,
    discord_bot: &Client
) -> Result<Vec<twilight_model::guild::Guild>, StatusCode> {
    let mut member_guilds = vec![];
    let bot_guilds = discord_bot.current_user_guilds().await.map_err(ise)?.models().await.map_err(ise)?;
    for partial_guild in bot_guilds {
        let guild = discord_bot.guild(partial_guild.id).await.map_err(ise)?.model().await.map_err(ise)?;
        let member_result = discord_bot.guild_member(guild.id, current_user.id).await;
        if member_result.is_ok() {
            member_guilds.push(guild);
        }
    }
    Ok(member_guilds)
}

pub async fn is_client_admin_guild(guild_id: Id<GuildMarker>, current_user: &CurrentUser, discord_bot: &Client) -> Result<bool, StatusCode> {
    let guild = discord_bot.guild(guild_id).await.map_err(ise)?.model().await.map_err(ise)?;
    let member = discord_bot.guild_member(guild_id, current_user.id).await.map_err(ise)?.model().await.map_err(ise)?;
    Ok(current_user.id == guild.owner_id || guild.roles.iter().any(|r| member.roles.contains(&r.id) && r.permissions & twilight_model::guild::Permissions::ADMINISTRATOR == twilight_model::guild::Permissions::ADMINISTRATOR))
} 

pub async fn admin_guilds(
    current_user: &CurrentUser,
    discord_bot: &Client
) -> Result<Vec<twilight_model::guild::Guild>, StatusCode> {
    let mut admin_guilds = vec![];
    let bot_guilds = discord_bot.current_user_guilds().await.map_err(ise)?.models().await.map_err(ise)?;
    for partial_guild in bot_guilds {
        let guild = discord_bot.guild(partial_guild.id).await.map_err(ise)?.model().await.map_err(ise)?;
        if guild.owner_id == current_user.id {
            admin_guilds.push(guild);
            continue;
        }
        let member_result = discord_bot.guild_member(guild.id, current_user.id).await;
        if member_result.is_err() {
            // Not a member
            continue;
        }
        let admin_role_ids = guild.roles.iter().filter(|r| r.permissions & twilight_model::guild::Permissions::ADMINISTRATOR == twilight_model::guild::Permissions::ADMINISTRATOR).map(|r| r.id).collect::<Vec<_>>();
        if member_result.unwrap().model().await.unwrap().roles.iter().any(|r| admin_role_ids.contains(r)) {
            admin_guilds.push(guild);
            continue;
        }
    }
    Ok(admin_guilds)
}