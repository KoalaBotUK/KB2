use crate::users::models::Link;
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

