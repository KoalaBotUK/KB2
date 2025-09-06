use twilight_http::{Client};
use http::StatusCode;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUser;
use crate::discord::{get_current_user_guilds, get_guild, get_guild_member};

pub async fn member_guilds(
    current_user: &CurrentUser,
    discord_bot: &Client
) -> Result<Vec<twilight_model::guild::Guild>, StatusCode> {
    let mut member_guilds = vec![];
    let bot_guilds = get_current_user_guilds(discord_bot).await?;
    for partial_guild in bot_guilds {
        let guild = get_guild(partial_guild.id, discord_bot).await?;
        let member_result = get_guild_member(guild.id, current_user.id, discord_bot).await;
        if member_result.is_ok() {
            member_guilds.push(guild);
        }
    }
    Ok(member_guilds)
}

pub async fn is_client_admin_guild(guild_id: Id<GuildMarker>, current_user: &CurrentUser, discord_bot: &Client) -> Result<bool, StatusCode> {
    let guild = get_guild(guild_id, discord_bot).await?;
    let member = get_guild_member(guild_id, current_user.id, discord_bot).await?;
    Ok(current_user.id == guild.owner_id || guild.roles.iter().any(|r| member.roles.contains(&r.id) && r.permissions & twilight_model::guild::Permissions::ADMINISTRATOR == twilight_model::guild::Permissions::ADMINISTRATOR))
} 

pub async fn admin_guilds(
    current_user: &CurrentUser,
    discord_bot: &Client
) -> Result<Vec<twilight_model::guild::Guild>, StatusCode> {
    let mut admin_guilds = vec![];
    let bot_guilds = get_current_user_guilds(discord_bot).await?;
    for partial_guild in bot_guilds {
        let guild = get_guild(partial_guild.id, discord_bot).await?;
        if guild.owner_id == current_user.id {
            admin_guilds.push(guild);
            continue;
        }
        let member_result = get_guild_member(guild.id, current_user.id, discord_bot).await;
        if member_result.is_err() {
            // Not a member
            continue;
        }
        let admin_role_ids = guild.roles.iter().filter(|r| r.permissions & twilight_model::guild::Permissions::ADMINISTRATOR == twilight_model::guild::Permissions::ADMINISTRATOR).map(|r| r.id).collect::<Vec<_>>();
        if member_result?.roles.iter().any(|r| admin_role_ids.contains(r)) {
            admin_guilds.push(guild);
            continue;
        }
    }
    Ok(admin_guilds)
}