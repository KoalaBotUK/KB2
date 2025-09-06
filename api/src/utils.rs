use std::collections::HashSet;
use twilight_http::{Client};
use http::StatusCode;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::{CurrentUser, CurrentUserGuild};
use crate::discord::{get_current_user_guilds, get_guild, get_guild_member};

pub async fn member_guilds(
    discord_user: &Client,
    discord_bot: &Client
) -> Result<Vec<CurrentUserGuild>, StatusCode> {
    let bot_guilds = get_current_user_guilds(discord_bot).await?;
    let mut user_guilds = get_current_user_guilds(discord_user).await?;
    let bot_guild_set = bot_guilds.iter().map(|g| g.id).collect::<HashSet<Id<GuildMarker>>>();
    let user_guild_set = user_guilds.iter().map(|g| g.id).collect::<HashSet<Id<GuildMarker>>>();
    let mut intersection_set = bot_guild_set.intersection(&user_guild_set);
    user_guilds.retain(|g| intersection_set.any(|ig| ig == &g.id));
    Ok(user_guilds)
}

pub async fn is_client_admin_guild(guild_id: Id<GuildMarker>, current_user: &CurrentUser, discord_bot: &Client) -> Result<bool, StatusCode> {
    let guild = get_guild(guild_id, discord_bot).await?;
    let member = get_guild_member(guild_id, current_user.id, discord_bot).await?;
    Ok(current_user.id == guild.owner_id || guild.roles.iter().any(|r| member.roles.contains(&r.id) && r.permissions & twilight_model::guild::Permissions::ADMINISTRATOR == twilight_model::guild::Permissions::ADMINISTRATOR))
}

pub async fn admin_guilds(
    discord_user: &Client,
    discord_bot: &Client
) -> Result<Vec<CurrentUserGuild>, StatusCode> {
    let bot_guilds = get_current_user_guilds(discord_bot).await?;
    let mut user_guilds = get_current_user_guilds(discord_user).await?;
    let bot_guild_set = bot_guilds.iter().map(|g| g.id).collect::<HashSet<Id<GuildMarker>>>();
    let user_guild_set = user_guilds.iter().map(|g| g.id).collect::<HashSet<Id<GuildMarker>>>();
    let mut intersection_set = bot_guild_set.intersection(&user_guild_set);
    user_guilds.retain(|g| intersection_set.any(|ig| ig == &g.id));

    let mut admin_guilds = vec![];
    for partial_guild in user_guilds {
        if partial_guild.owner || partial_guild.permissions & twilight_model::guild::Permissions::ADMINISTRATOR == twilight_model::guild::Permissions::ADMINISTRATOR {
            admin_guilds.push(partial_guild);
        }
    }
    Ok(admin_guilds)
}