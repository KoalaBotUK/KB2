use http::StatusCode;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUserGuild;
use crate::discord::{get_current_user_guild, get_current_user_guilds};

async fn client_admin_guilds(client: &twilight_http::Client) -> Result<Vec<CurrentUserGuild>, StatusCode> {
    let guilds = get_current_user_guilds(client).await?;
    Ok(guilds
        .into_iter()
        .filter(|g| g.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR)
        .collect())
}

pub async fn intersect_admin_guilds(
    client_1: &twilight_http::Client,
    client_2: &twilight_http::Client,
) -> Result<Vec<CurrentUserGuild>, StatusCode> {
    let client_1_guilds = client_admin_guilds(client_1).await?;
    let client_2_guilds = client_admin_guilds(client_2).await?;

    Ok(client_1_guilds
        .into_iter()
        .filter(|g| client_2_guilds.iter().any(|g2| g.id == g2.id))
        .collect())
}

async fn is_client_admin_guild(
    guild_id: Id<GuildMarker>,
    client: &twilight_http::Client,
) -> Result<bool, StatusCode> {
    let guild = get_current_user_guild(guild_id, client).await?;
    if guild.owner || guild.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR {
        return Ok(true);
    }
    Ok(false)
}

pub async fn is_intersect_admin_guild(
    guild_id: Id<GuildMarker>,
    client_1: &twilight_http::Client,
    client_2: &twilight_http::Client,
) -> Result<bool, StatusCode> {
    Ok(is_client_admin_guild(guild_id, client_1).await?
        && is_client_admin_guild(guild_id, client_2).await?)
}
