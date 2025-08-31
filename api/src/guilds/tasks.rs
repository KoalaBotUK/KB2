use std::collections::HashMap;
use crate::guilds::models::Guild;
use crate::utils::retry_on_rl;
use lambda_http::tracing::info;
use regex::Regex;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};

pub async fn update_guilds(bot: &twilight_http::Client, dynamo: &aws_sdk_dynamodb::Client) {
    info!("Updating guilds...");
    let d_guilds = retry_on_rl(|| async { bot.current_user_guilds().await })
        .await
        .unwrap()
        .models()
        .await
        .unwrap();

    let k_guilds =
        Guild::vec_from_db(d_guilds.iter().map(|g| g.id).collect(), dynamo).await;

    for d_guild in d_guilds {
        let mut found = false;
        for k_guild in &k_guilds {
            if d_guild.id == k_guild.guild_id {
                info!("Guild {} found in DB, checking for updates...", d_guild.id);
                let mut updated_guild = k_guild.clone();
                updated_guild.name = d_guild.name.clone();
                updated_guild.icon = d_guild.icon;
                if updated_guild != *k_guild {
                    updated_guild.save(dynamo).await;
                    info!("Guild {} updated.", d_guild.id);
                }
                found = true;
                continue;
            }
        }
        if !found {
            let new_guild = Guild {
                guild_id: d_guild.id,
                verify: crate::guilds::models::Verify { roles: vec![], user_links: vec![] },
                name: d_guild.name.clone(),
                icon: d_guild.icon,
                user_links: HashMap::new(),
            };
            new_guild.save(dynamo).await;
        }
    }
    info!("Guilds updated.");
}

async fn assign_role_user(
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
    bot: &twilight_http::Client,
) {
    let _ =
        retry_on_rl(|| async { bot.add_guild_member_role(guild_id, user_id, role_id).await }).await;
}

async fn remove_role_user(
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    role_id: Id<RoleMarker>,
    bot: &twilight_http::Client,
) {
    let _ = retry_on_rl(|| async { bot.remove_guild_member_role(guild_id, user_id, role_id).await })
        .await;
}

pub async fn assign_roles_guild_role(
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
    bot: &twilight_http::Client,
    dynamo: &aws_sdk_dynamodb::Client,
) {
    let guild = Guild::from_db(guild_id, dynamo).await.unwrap();
    let user_links = &guild.verify.user_links;
    let roles = &guild.verify.roles;
    let role = roles.iter().find(|r| r.role_id == role_id);
    if role.is_none() {
        return;
    }
    for user_link in user_links {
        if Regex::new(&role.unwrap().pattern)
            .unwrap()
            .is_match(&user_link.link_address)
        {
            assign_role_user(guild_id, user_link.user_id, role.unwrap().role_id, bot).await;
        }
    }
}

pub async fn assign_roles_guild_user_link(
    link_address: &str,
    user_id: Id<UserMarker>,
    guild: &Guild,
    bot: &twilight_http::Client,
) {
    let roles = &guild.verify.roles;
    for role in roles {
        if Regex::new(&role.pattern)
            .unwrap()
            .is_match(link_address)
        {
            assign_role_user(guild.guild_id, user_id, role.role_id, bot).await;
        } else {
            remove_role_user(guild.guild_id, user_id, role.role_id, bot).await;
        }
    }
}

pub async fn assign_all_roles_user_guild(
    user_id: Id<UserMarker>,
    guild: &Guild,
    bot: &twilight_http::Client,
) {
    let user_links = &guild.verify.user_links;
    for user_link in user_links {
        if user_link.user_id == user_id {
            assign_roles_guild_user_link(&user_link.link_address, user_id, guild, bot).await;
        }
    }
}
