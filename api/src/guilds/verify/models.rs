use std::collections::HashMap;
use std::hash::Hash;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{RoleMarker, UserMarker};
use crate::dynamo::{as_map, as_map_vec, as_string, as_u32};
use crate::users::models::Link;

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

impl From<&HashMap<String, AttributeValue>> for VerifyRole {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        VerifyRole {
            role_id: Id::new(
                as_string(item.get("role_id"), &"0".to_string())
                    .parse::<u64>()
                    .unwrap_or(0),
            ),
            pattern: as_string(item.get("pattern"), &"".to_string()),
            members: as_u32(item.get("members"), 0),
        }
    }
}

impl From<VerifyRole> for HashMap<String, AttributeValue> {
    fn from(role: VerifyRole) -> Self {
        let mut role_map = HashMap::new();
        role_map.insert("role_id".to_string(), AttributeValue::S(role.role_id.to_string()));
        role_map.insert("pattern".to_string(), AttributeValue::S(role.pattern));
        role_map.insert("members".to_string(), AttributeValue::N(role.members.to_string()));
        role_map
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct UserLink {
    pub user_id: Id<UserMarker>,
    pub link_address: String,
}

impl From<&HashMap<String, AttributeValue>> for UserLink {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        UserLink {
            user_id: Id::new(
                as_string(item.get("user_id"), &"0".to_string())
                    .parse::<u64>()
                    .unwrap_or(0),
            ),
            link_address: as_string(item.get("link_address"), &"".to_string()),
        }
    }
}

impl From<UserLink> for HashMap<String, AttributeValue> {
    fn from(user_link: UserLink) -> Self {
        let mut link_map = HashMap::new();
        link_map.insert("user_id".to_string(), AttributeValue::S(user_link.user_id.to_string()));
        link_map.insert("link_address".to_string(), AttributeValue::S(user_link.link_address));
        link_map
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Verify {
    pub roles: Vec<VerifyRole>,
    pub user_links: HashMap<Id<UserMarker>, Vec<Link>>,
}

impl From<&HashMap<String, AttributeValue>> for Verify {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        Verify {
            roles: as_map_vec(item.get("roles"))
                .into_iter()
                .map(|m| m.into())
                .collect(),
            user_links: as_map(item.get("user_links")).unwrap_or(&HashMap::new()).iter().map(
                |(k,v)| (Id::new(k.parse::<u64>().unwrap_or(0)), as_map_vec(Some(v)).iter().map(|&l| l.into()).collect()))
                .collect()
        }
    }
}

impl From<Verify> for HashMap<String, AttributeValue> {
    fn from(verify: Verify) -> Self {
        let mut verify_map = HashMap::new();
        let roles: Vec<AttributeValue> = verify.roles.into_iter().map(|r| AttributeValue::M(r.into())).collect();
        verify_map.insert("roles".to_string(), AttributeValue::L(roles));

        verify_map.insert("user_links".to_string(), AttributeValue::M(verify.user_links.iter().map(
            |(k,v)| (k.to_string(), AttributeValue::L(v.iter().map(|l| AttributeValue::M(l.clone().into())).collect()))
        ).collect()));

        verify_map
    }
}