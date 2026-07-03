use crate::discord::{get_current_user_guilds, get_guild, get_guild_member};
use http::StatusCode;
use subtle::ConstantTimeEq;
use twilight_http::Client;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::user::{CurrentUser, CurrentUserGuild};

/// Compare two secrets (e.g. bot tokens) in constant time.
///
/// Using `==` on strings/bytes short-circuits on the first mismatched byte,
/// which leaks timing information that can be used to recover a secret
/// byte-by-byte. This compares the full contents in constant time instead.
pub fn secure_compare(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

pub async fn member_guilds(
    discord_user: &Client,
    discord_bot: &Client,
) -> Result<Vec<CurrentUserGuild>, StatusCode> {
    let mut bot_guilds = get_current_user_guilds(discord_bot).await?;
    bot_guilds.retain(is_admin);
    let user_guilds = get_current_user_guilds(discord_user).await?;
    let mut guilds = vec![];
    for u_guild in &user_guilds {
        for b_guild in &bot_guilds {
            if u_guild.id == b_guild.id {
                guilds.push(u_guild.clone());
                continue;
            }
        }
    }
    Ok(guilds)
}

pub async fn is_client_admin_guild(
    guild_id: Id<GuildMarker>,
    current_user: &CurrentUser,
    discord_bot: &Client,
) -> Result<bool, StatusCode> {
    let guild = get_guild(guild_id, discord_bot).await?;
    let member = get_guild_member(guild_id, current_user.id, discord_bot).await?;
    Ok(current_user.id == guild.owner_id
        || guild.roles.iter().any(|r| {
            member.roles.contains(&r.id)
                && r.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR
        }))
}

pub async fn admin_guilds(
    discord_user: &Client,
    discord_bot: &Client,
) -> Result<Vec<CurrentUserGuild>, StatusCode> {
    let bot_guilds = get_current_user_guilds(discord_bot).await?;
    let user_guilds = get_current_user_guilds(discord_user).await?;
    let mut guilds = vec![];
    for u_guild in &user_guilds {
        for b_guild in &bot_guilds {
            if u_guild.id == b_guild.id {
                if b_guild.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR
                    && (u_guild.owner
                        || u_guild.permissions & Permissions::ADMINISTRATOR
                            == Permissions::ADMINISTRATOR)
                {
                    guilds.push(u_guild.clone());
                }
                continue;
            }
        }
    }
    Ok(guilds)
}

pub fn is_admin(guild: &CurrentUserGuild) -> bool {
    guild.owner || guild.permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR
}

#[cfg(test)]
mod secure_compare_tests {
    use super::secure_compare;

    #[test]
    fn matching_secrets_are_accepted() {
        assert!(secure_compare("super-secret-token", "super-secret-token"));
    }

    #[test]
    fn non_matching_secrets_are_rejected() {
        assert!(!secure_compare("super-secret-token", "not-the-token"));
    }

    #[test]
    fn secrets_of_different_length_are_rejected() {
        assert!(!secure_compare("short", "a-much-longer-secret"));
    }

    #[test]
    fn empty_strings_are_equal() {
        assert!(secure_compare("", ""));
    }

    #[test]
    fn case_sensitive_comparison() {
        assert!(!secure_compare("Token", "token"));
    }
}
