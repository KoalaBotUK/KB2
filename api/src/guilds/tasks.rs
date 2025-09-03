use http::StatusCode;
use crate::guilds::models::{Guild, VerifyRole};
use crate::utils::{ise, retry_on_rl};
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
                name: d_guild.name.clone(),
                icon: d_guild.icon,
                ..Default::default()
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

pub async fn remove_role_from_guild(
    guild: &mut Guild,
    role_id: Id<RoleMarker>,
    bot: &twilight_http::Client,
) {
    if !guild.verify.roles.iter().any(|r| r.role_id == role_id) {
        return;
    }
    for (&user_id, user_links) in &mut guild.verify.user_links {
        remove_role_user(guild.guild_id, user_id, role_id, bot).await;
    }
    guild.verify.roles.retain(|r| r.role_id != role_id);
}

pub async fn add_role_to_guild(
    guild: &mut Guild,
    mut role: VerifyRole,
    bot: &twilight_http::Client
){
    remove_role_from_guild(guild, role.role_id, bot).await;
    for (&user_id, user_links) in &mut guild.verify.user_links {
        for user_link in user_links {
            if Regex::new(&role.pattern)
                .unwrap()
                .is_match(&*user_link.link_address)
            {
                assign_role_user(guild.guild_id, user_id, role.role_id, bot).await;
                role.members += 1;
                break;
            }
        }
    }
    guild.verify.roles.push(role);
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
        let has_dsc_role = retry_on_rl(|| async { bot.guild_member(guild.guild_id, user_id).await}).await.map_err(ise)?.model().await.map_err(ise)?.roles.contains(&role.role_id);
        if enabled && Regex::new(&role.pattern)
            .unwrap()
            .is_match(link_address)
        {
            info!("Assigning role {} to user {}", role.role_id, user_id);
            if !has_dsc_role {
                assign_role_user(guild.guild_id, user_id, role.role_id, bot).await;
                role.members += 1;
            }
            } else {
            info!("Removing role {} from user {}", role.role_id, user_id);
            if has_dsc_role {
                remove_role_user(guild.guild_id, user_id, role.role_id, bot).await;
                role.members -= 1;
            }
        }
    }
    Ok(())
}

