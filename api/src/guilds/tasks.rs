use http::StatusCode;
use crate::guilds::models::Guild;
use lambda_http::tracing::info;
use regex::Regex;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};
use crate::discord::{add_guild_member_role, get_guild_member, remove_guild_member_role};
use crate::guilds::verify::models::VerifyRole;

async fn assign_role_user(
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
    bot: &twilight_http::Client,
) -> Result<(), StatusCode> {
    add_guild_member_role(guild_id, user_id, role_id, bot).await
}

async fn remove_role_user(
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
    bot: &twilight_http::Client,
) -> Result<(), StatusCode> {
    remove_guild_member_role(guild_id, user_id, role_id, bot).await
}

pub async fn remove_role_from_guild(
    guild: &mut Guild,
    role_id: Id<RoleMarker>,
    bot: &twilight_http::Client,
) -> Result<(), StatusCode> {
    if !guild.verify.roles.iter().any(|r| r.role_id == role_id) {
        return Ok(());
    }
    for &user_id in &mut guild.verify.user_links.keys() {
        remove_role_user(guild.guild_id, user_id, role_id, bot).await?;
    }
    guild.verify.roles.retain(|r| r.role_id != role_id);
    Ok(())
}

pub async fn add_role_to_guild(
    guild: &mut Guild,
    mut role: VerifyRole,
    bot: &twilight_http::Client
) -> Result<(), StatusCode> {
    remove_role_from_guild(guild, role.role_id, bot).await?;
    for (&user_id, user_links) in &mut guild.verify.user_links {
        for user_link in user_links {
            if Regex::new(&role.pattern)
                .unwrap()
                .is_match(&user_link.link_address)
            {
                assign_role_user(guild.guild_id, user_id, role.role_id, bot).await?;
                role.members += 1;
                break;
            }
        }
    }
    guild.verify.roles.push(role);
    Ok(())
}

pub async fn assign_roles_guild_user_link(
    enabled: bool,
    link_address: &str,
    user_id: Id<UserMarker>,
    guild: &mut Guild,
    bot: &twilight_http::Client,
) -> Result<(), StatusCode> {
    let roles = &mut guild.verify.roles;
    for i in 0..roles.len() {
        let role = roles.get_mut(i).unwrap();
        let has_dsc_role = get_guild_member(guild.guild_id, user_id, bot).await?.roles.contains(&role.role_id);
        if enabled && Regex::new(&role.pattern)
            .unwrap()
            .is_match(link_address)
        {
            info!("Assigning role {} to user {}", role.role_id, user_id);
            if !has_dsc_role {
                assign_role_user(guild.guild_id, user_id, role.role_id, bot).await?;
                role.members += 1;
            }
            } else {
            info!("Removing role {} from user {}", role.role_id, user_id);
            if has_dsc_role {
                remove_role_user(guild.guild_id, user_id, role.role_id, bot).await?;
                role.members -= 1;
            }
        }
    }
    Ok(())
}

