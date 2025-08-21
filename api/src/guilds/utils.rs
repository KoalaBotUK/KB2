use http::StatusCode;
use lambda_http::tracing::error;
use twilight_http::Error;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::CurrentUserGuild;

pub fn ise(e: Error) -> StatusCode {
    error!("Internal Server Error: {:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn client_admin_guilds(client: &twilight_http::Client) -> Vec<CurrentUserGuild> {
    client.current_user_guilds().await.unwrap().models().await.unwrap()
        .into_iter()
        .filter(|g| g.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR)
        .collect()
}

pub async fn intersect_admin_guilds(
    client_1: &twilight_http::Client,
    client_2: &twilight_http::Client
) -> Vec<CurrentUserGuild> {
    let client_1_guilds = client_admin_guilds(client_1).await;
    let client_2_guilds = client_admin_guilds(client_2).await;

    client_1_guilds
        .into_iter()
        .filter(|g| client_2_guilds.iter().any(|g2| g.id == g2.id))
        .collect()
}

async fn is_client_admin_guild(guild_id: Id<GuildMarker>, client: &twilight_http::Client) -> Result<bool, Error> {
    let guilds = client.current_user_guilds().after(Id::new(guild_id.get()-1)).limit(1).await?.models().await.unwrap();
    let admin_guilds: Vec<CurrentUserGuild> = guilds
        .into_iter()
        .filter(|g| g.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR)
        .collect();
    Ok(admin_guilds.len() == 1)
}

pub async fn is_intersect_admin_guild(
    guild_id: Id<GuildMarker>,
    client_1: &twilight_http::Client,
    client_2: &twilight_http::Client
) -> Result<bool, Error> {
    Ok(
        is_client_admin_guild(guild_id, client_1).await? && is_client_admin_guild(guild_id, client_2).await?)
}