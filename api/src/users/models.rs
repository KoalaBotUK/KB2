use std::collections::HashMap;
use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use crate::dynamo::{as_string, as_string_vec};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub emails: Vec<String>,
    pub username: String,
    pub first_name: String,
    pub last_name: String
}

impl From<&HashMap<String, AttributeValue>> for User {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        User {
            user_id: as_string(item.get("user_id"), &"".to_string()),
            emails: as_string_vec(item.get("emails")),
            username: as_string(item.get("username"), &"".to_string()),
            first_name: as_string(item.get("first_name"), &"".to_string()),
            last_name: as_string(item.get("last_name"), &"".to_string()),
        }
    }
}