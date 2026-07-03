use crate::AppState;
use crate::discord::{add_guild_member_role, get_current_user_guild, remove_guild_member_role};
use crate::guilds::models::Guild;
use crate::users::models::{Link, LinkGuild, User};
use crate::users::utils::link_arr_match;
use axum::extract::{Path, State};
use axum::routing::put;
use axum::{Extension, Json};
use http::StatusCode;
use serde_json::{Value, json};
use std::sync::Arc;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::user::CurrentUser;
use common::audit::AuditMessage;
use crate::audit::audit;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            "/{guild_id}",
            put(put_link_guilds_id).delete(delete_link_guilds_id),
        )
}

async fn put_link_guilds_id(
    Path((user_id, guild_id)): Path<(Id<UserMarker>, Id<GuildMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if current_user.id != user_id || !is_client_guild_member(guild_id, &discord_user).await? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let new_link_guild = LinkGuild {
        guild_id,
        enabled: true,
    };

    let mut user = User::from_db(user_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| User {
            user_id,
            ..Default::default()
        });
    let audit_old_data = user.link_guilds.clone();
    user.link_guilds.retain(|g| g.guild_id != guild_id);
    user.link_guilds.push(new_link_guild.clone());
    let audit_new_data = user.link_guilds.clone();
    user.save(&app_state.pg_pool).await?;

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });

    // Capture the previously stored links (if any) so we only assign roles
    // that don't already match, instead of skipping role sync entirely
    // when the user has linked this guild before.
    let previous_links = guild.verify.user_links.get(&user.user_id).cloned();
    guild.verify.user_links.insert(user_id, user.links.clone());

    for verify_role in &guild.verify.roles {
        if role_newly_qualifies(&user.links, previous_links.as_deref(), &verify_role.pattern) {
            add_guild_member_role(
                guild_id,
                user_id,
                verify_role.role_id,
                &app_state.discord_bot,
            )
            .await?;
        }
    }
    // Derive `members` from `user_links` (now updated above) rather than
    // hand-incrementing it, so this stays consistent with every other
    // mutation site (link add/remove, role add/remove, recon).
    guild.verify.recompute_role_members();

    guild.save(&app_state.pg_pool).await?;

    // write audit
    audit(AuditMessage::new("update_link_guilds".to_string(), user_id, Some(guild_id),
                             Some(audit_old_data), Some(audit_new_data)), &app_state.sqs).await;
    
    Ok(Json(json!(new_link_guild)))
}

/// Regression helper for koalabotuk/kb2#55: decides whether a verify role
/// needs to be (re)assigned when a user links/re-enables a guild.
///
/// A role only "newly qualifies" if the user's *current* links match its
/// pattern but their *previously stored* links (if any) did not. This is
/// what makes re-enabling an already-linked guild idempotent — unchanged
/// links don't get re-added or double-counted — while still correctly
/// assigning roles the user newly qualifies for (which the old
/// `contains_key` early return skipped entirely).
fn role_newly_qualifies(
    current_links: &[Link],
    previous_links: Option<&[Link]>,
    pattern: &str,
) -> bool {
    let matches = link_arr_match(current_links, pattern);
    let previously_matched =
        previous_links.is_some_and(|links| link_arr_match(links, pattern));
    matches && !previously_matched
}

async fn delete_link_guilds_id(
    Path((user_id, guild_id)): Path<(Id<UserMarker>, Id<GuildMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(discord_user): Extension<Arc<twilight_http::Client>>,
    State(app_state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    if current_user.id != user_id || !is_client_guild_member(guild_id, &discord_user).await? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut user = User::from_db(user_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| User {
            user_id,
            ..Default::default()
        });

    user.link_guilds.retain(|g| g.guild_id != guild_id);

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild {
            guild_id,
            ..Default::default()
        });

    for role in &guild.verify.roles {
        if link_arr_match(&user.links, &role.pattern) {
            remove_guild_member_role(guild_id, user_id, role.role_id, &app_state.discord_bot)
                .await?;
        }
    }
    guild.verify.user_links.remove(&user_id);
    // Recompute instead of the old `if role.members > 0 { role.members -= 1 }`
    // guard, which was only needed because the counter could otherwise drift.
    guild.verify.recompute_role_members();

    user.save(&app_state.pg_pool).await?;
    guild.save(&app_state.pg_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn is_client_guild_member(
    guild_id: Id<GuildMarker>,
    client: &twilight_http::Client,
) -> Result<bool, StatusCode> {
    Ok(get_current_user_guild(guild_id, client).await?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_link(address: &str) -> Link {
        Link {
            link_address: address.to_string(),
            linked_at: 0,
            active: true,
        }
    }

    /// Regression test for koalabotuk/kb2#55: re-enabling a guild link for a
    /// user whose links haven't changed since they last linked must not
    /// re-assign a role they already hold. Before the fix, `put_link_guilds_id`
    /// returned early whenever `user_links` already contained the user, which
    /// incidentally avoided double-adds here — but also skipped role sync
    /// entirely (see the other tests below).
    #[test]
    fn role_does_not_newly_qualify_when_links_are_unchanged() {
        let previous = vec![active_link("a@example.com")];
        let current = vec![active_link("a@example.com")];

        assert!(!role_newly_qualifies(
            &current,
            Some(&previous),
            r"@example\.com$"
        ));
    }

    /// The core bug: a user re-enabling an already-linked guild (so
    /// `user_links` already has an entry for them) must still be assigned
    /// any role their current links newly qualify for. The old code's
    /// `contains_key` early return made this a no-op — the user's row
    /// updated, but Discord roles were never (re)assigned.
    #[test]
    fn role_newly_qualifies_when_previous_links_did_not_match() {
        let previous = vec![active_link("a@other.com")];
        let current = vec![active_link("a@other.com"), active_link("b@example.com")];

        assert!(role_newly_qualifies(
            &current,
            Some(&previous),
            r"@example\.com$"
        ));
    }

    /// A first-time link (no previously stored links at all) must qualify
    /// for every role the current links match — the `None` case must not be
    /// treated as "already matched".
    #[test]
    fn role_newly_qualifies_when_there_are_no_previous_links() {
        let current = vec![active_link("a@example.com")];

        assert!(role_newly_qualifies(&current, None, r"@example\.com$"));
    }

    /// If the user's current links no longer match a role's pattern, that
    /// role must never be reported as newly qualifying, regardless of what
    /// they previously had linked.
    #[test]
    fn role_does_not_newly_qualify_when_current_links_do_not_match() {
        let previous = vec![active_link("a@other.com")];
        let current = vec![active_link("a@other.com")];

        assert!(!role_newly_qualifies(
            &current,
            Some(&previous),
            r"@example\.com$"
        ));
    }
}
