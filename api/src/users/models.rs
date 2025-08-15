use std::collections::HashMap;
use aws_sdk_dynamodb::types::AttributeValue;
use http::StatusCode;
use lambda_http::tracing::error;
use serde::{Deserialize, Serialize};
use crate::dynamo::{as_u64, as_string, as_string_vec, as_map_vec};

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
            links: as_map_vec(item.get("links"))
                .into_iter()
                .map(|m| Link {
                    link_address: as_string(m.get("link_address"), &"".to_string()),
                    linked_at: as_u64(m.get("linked_at"), 0),
                })
                .collect(),
            linked_guild_ids: as_string_vec(item.get("linked_guild_ids")),
        }
    }
}

impl User {
    pub async fn from_db(user_id: &str, dynamo: &aws_sdk_dynamodb::Client) -> Option<User> {
        match dynamo.query().table_name(format!("kb2_users_{}",std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set"),))
            .key_condition_expression("#uid = :uid")
            .expression_attribute_names("#uid", "user_id")
            .expression_attribute_values(":uid", AttributeValue::S(user_id.to_string()))
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            .and_then(|resp| {
                let items = resp.items.unwrap_or_default();
                if items.is_empty() {
                    return Err(StatusCode::NOT_FOUND);
                }
                let user: User = (&items[0]).into();
                Ok(user)
            }) {
            Ok(user) => Some(user),
            Err(e) => {
                error!("Error fetching user from DynamoDB: {}", e);
                None
            }
        }

    }

    pub async fn save(&self, dynamo: &aws_sdk_dynamodb::Client) {
        let mut item = HashMap::new();
        item.insert("user_id".to_string(), AttributeValue::S(self.user_id.clone()));
        item.insert(
            "links".to_string(),
            AttributeValue::L(
                self.links
                    .iter()
                    .map(|l| {
                        let mut map = HashMap::new();
                        map.insert("link_address".to_string(), AttributeValue::S(l.link_address.clone()));
                        map.insert("linked_at".to_string(), AttributeValue::N(l.linked_at.to_string()));
                        AttributeValue::M(map)
                    })
                    .collect(),
            ),
        );
        item.insert(
            "linked_guild_ids".to_string(),
            AttributeValue::L(
                self.linked_guild_ids
                    .iter()
                    .map(|id| AttributeValue::S(id.clone()))
                    .collect(),
            ),
        );

        match dynamo
            .put_item()
            .table_name(format!("kb2_users_{}", std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set")))
            .set_item(Some(item))
            .send()
            .await
        {
            Ok(_) => (),
            Err(e) => {
                error!("DynamoDB write error: {}", e);
                panic!("Failed to save user to DynamoDB");
            }
        }
    }
}