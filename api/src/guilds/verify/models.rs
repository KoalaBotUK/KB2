use crate::users::models::Link;
use crate::users::utils::link_arr_match;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use twilight_model::id::Id;
use twilight_model::id::marker::{RoleMarker, UserMarker};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyRole {
    pub role_id: Id<RoleMarker>,
    pub pattern: String,
    pub members: u32,
}

impl Default for VerifyRole {
    fn default() -> Self {
        VerifyRole {
            role_id: Id::new(1),
            pattern: "".to_string(),
            members: 0,
        }
    }
}

impl Hash for VerifyRole {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.role_id.hash(state);
        self.pattern.hash(state);
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Verify {
    pub roles: Vec<VerifyRole>,
    pub user_links: HashMap<Id<UserMarker>, Vec<Link>>,
}

impl Verify {
    /// Recomputes `members` for every role from `user_links`, which is the
    /// single source of truth for who currently holds a verify role.
    ///
    /// This must be called after any mutation of `user_links` (link add,
    /// link remove, guild link/unlink, role add, recon, ...) instead of
    /// hand-incrementing/decrementing `role.members`, so all call sites stay
    /// consistent and counts can never drift. A member is counted at most
    /// once per role even if they have multiple active links that match the
    /// role's pattern.
    pub fn recompute_role_members(&mut self) {
        let user_links = &self.user_links;
        for role in &mut self.roles {
            role.members = user_links
                .values()
                .filter(|links| link_arr_match(links, &role.pattern))
                .count() as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::users::models::Link;

    fn active_link(address: &str) -> Link {
        Link {
            link_address: address.to_string(),
            linked_at: 0,
            active: true,
        }
    }

    fn inactive_link(address: &str) -> Link {
        Link {
            link_address: address.to_string(),
            linked_at: 0,
            active: false,
        }
    }

    fn role(pattern: &str) -> VerifyRole {
        VerifyRole {
            role_id: Id::new(1),
            pattern: pattern.to_string(),
            members: 0,
        }
    }

    /// Regression test for koalabotuk/kb2#28: re-adding a link a user
    /// previously had (remove-then-re-add, as `post_link` does) must not
    /// inflate the role's member count. Recomputing from `user_links` after
    /// every mutation keeps the count at 1, not 2.
    #[test]
    fn recompute_role_members_does_not_double_count_on_relink() {
        let mut verify = Verify {
            roles: vec![role(r"@example\.com$")],
            user_links: HashMap::new(),
        };
        let user_id: Id<UserMarker> = Id::new(1);

        // First link is added.
        verify
            .user_links
            .insert(user_id, vec![active_link("a@example.com")]);
        verify.recompute_role_members();
        assert_eq!(verify.roles[0].members, 1);

        // Simulate `post_link`'s remove-then-re-add of the *same* address
        // (e.g. the user re-links an email they already had).
        let links = verify.user_links.get_mut(&user_id).unwrap();
        links.retain(|l| l.link_address != "a@example.com");
        links.push(active_link("a@example.com"));
        verify.recompute_role_members();

        // Previously this would have been 2 because callers incremented
        // `role.members` on every add without checking prior state.
        assert_eq!(verify.roles[0].members, 1);
    }

    /// A member with several active links that all match the same role
    /// pattern must still only count once — one member, one seat.
    #[test]
    fn recompute_role_members_dedupes_multiple_matching_links_for_one_user() {
        let mut verify = Verify {
            roles: vec![role(r"@example\.com$")],
            user_links: HashMap::new(),
        };
        let user_id: Id<UserMarker> = Id::new(1);
        verify.user_links.insert(
            user_id,
            vec![
                active_link("a@example.com"),
                active_link("b@example.com"),
            ],
        );

        verify.recompute_role_members();

        assert_eq!(verify.roles[0].members, 1);
    }

    /// Inactive (superseded) links must not count towards membership, and
    /// removing a user's only active link drops the count back to 0 —
    /// mirroring what `delete_link`/`delete_link_guilds_id` need.
    #[test]
    fn recompute_role_members_ignores_inactive_links_and_tracks_removal() {
        let mut verify = Verify {
            roles: vec![role(r"@example\.com$")],
            user_links: HashMap::new(),
        };
        let user_id: Id<UserMarker> = Id::new(1);
        verify
            .user_links
            .insert(user_id, vec![active_link("a@example.com")]);
        verify.recompute_role_members();
        assert_eq!(verify.roles[0].members, 1);

        // Link gets deactivated (soft removal), as `delete_link` does.
        verify
            .user_links
            .insert(user_id, vec![inactive_link("a@example.com")]);
        verify.recompute_role_members();
        assert_eq!(verify.roles[0].members, 0);
    }

    /// Two different roles derive independent, correct counts from the same
    /// shared `user_links` map — this is the "recon vs put_roles_id vs
    /// add/remove" consistency the issue calls out: every code path now
    /// goes through the same recompute logic instead of hand-rolled
    /// increment/decrement guards.
    #[test]
    fn recompute_role_members_keeps_multiple_roles_consistent() {
        let mut verify = Verify {
            roles: vec![role(r"@example\.com$"), role(r"@other\.com$")],
            user_links: HashMap::new(),
        };
        let user_a: Id<UserMarker> = Id::new(1);
        let user_b: Id<UserMarker> = Id::new(2);
        verify
            .user_links
            .insert(user_a, vec![active_link("a@example.com")]);
        verify
            .user_links
            .insert(user_b, vec![active_link("b@other.com")]);

        verify.recompute_role_members();

        assert_eq!(verify.roles[0].members, 1);
        assert_eq!(verify.roles[1].members, 1);

        // Remove user_a entirely (guild unlink) and re-run — no stale
        // decrement bookkeeping needed, and it can never go negative/underflow.
        verify.user_links.remove(&user_a);
        verify.recompute_role_members();
        assert_eq!(verify.roles[0].members, 0);
        assert_eq!(verify.roles[1].members, 1);
    }
}

