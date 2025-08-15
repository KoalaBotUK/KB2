use std::collections::HashMap;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use crate::dynamo::{as_u64, as_string, as_string_vec};

#[derive(Clone, Serialize, Deserialize)]
pub struct Link {
    pub link_address: String,
    pub linked_at: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub links: Vec<Link>,
    pub linked_guild_ids: Vec<String>,
}

impl From<&HashMap<String, AttributeValue>> for User {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        User {
            user_id: as_string(item.get("user_id"), &"".to_string()),
            links: as_string_vec(item.get("links"))
                .into_iter()
                .map(|link| Link {
                    link_address: link,
                    linked_at: as_u64(item.get("linked_at"), 0),
                })
                .collect(),
            linked_guild_ids: as_string_vec(item.get("linked_guild_ids")),
        }
    }
}