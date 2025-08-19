use crate::dynamo::{as_map, as_map_vec, as_string};
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes};
use http::StatusCode;
use lambda_http::tracing::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct VerifyRole {
    pub role_id: String,
    pub role_name: String,
    pub pattern: String,
    pub member_count: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Verify {
    pub roles: Vec<VerifyRole>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Guild {
    pub guild_id: String,
    pub verify: Verify,
}

impl From<&HashMap<String, AttributeValue>> for Guild {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        Guild {
            guild_id: as_string(item.get("guild_id"), &"".to_string()),
            verify: as_map(item.get("verify"))
                .map(|m| Verify {
                    roles: as_map_vec(m.get("roles"))
                        .into_iter()
                        .map(|role_map| VerifyRole {
                            role_id: as_string(role_map.get("role_id"), &"".to_string()),
                            role_name: as_string(role_map.get("role_name"), &"".to_string()),
                            pattern: as_string(role_map.get("pattern"), &"".to_string()),
                            member_count: role_map
                                .get("member_count")
                                .and_then(|v| v.as_n().ok())
                                .and_then(|n| n.parse::<u32>().ok())
                                .unwrap_or(0),
                        })
                        .collect(),
                })
                .unwrap_or(Verify { roles: vec![] }),
        }
    }
}

impl From<Guild> for HashMap<String, AttributeValue> {
    fn from(guild: Guild) -> Self {
        let mut item = HashMap::new();
        item.insert("guild_id".to_string(), AttributeValue::S(guild.guild_id));

        let roles: Vec<AttributeValue> = guild
            .verify
            .roles
            .into_iter()
            .map(|role| {
                let mut role_map = HashMap::new();
                role_map.insert("role_id".to_string(), AttributeValue::S(role.role_id));
                role_map.insert("role_name".to_string(), AttributeValue::S(role.role_name));
                role_map.insert("pattern".to_string(), AttributeValue::S(role.pattern));
                role_map.insert(
                    "member_count".to_string(),
                    AttributeValue::N(role.member_count.to_string()),
                );
                AttributeValue::M(role_map)
            })
            .collect();

        item.insert(
            "verify".to_string(),
            AttributeValue::M(HashMap::from([(
                "roles".to_string(),
                AttributeValue::L(roles),
            )])),
        );

        item
    }
}

impl Guild {
    fn table_name() -> String {
        format!(
            "kb2_guilds_{}",
            std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set")
        )
    }

    pub async fn from_db(guild_id: &str, dynamo: &aws_sdk_dynamodb::Client) -> Option<Guild> {
        let mut key_attributes = HashMap::new();
        key_attributes.insert(
            "guild_id".to_string(),
            AttributeValue::S(guild_id.to_string()),
        );
        
        match dynamo
            .get_item()
            .table_name(format!(
                "kb2_guilds_{}",
                std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set"),
            ))
            .set_key(Some(key_attributes))
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            .and_then(|resp| {
                let item = resp.item.unwrap_or_default();
                let guild: Guild = (&item).into();
                Ok(guild)
            }) {
            Ok(guild) => Some(guild),
            Err(e) => {
                error!("Error fetching guild from DynamoDB: {}", e);
                None
            }
        }
    }

    pub async fn vec_from_db(
        guild_ids: Vec<String>,
        dynamo: &aws_sdk_dynamodb::Client,
    ) -> Vec<Guild> {
        let mut keys: Vec<HashMap<String, AttributeValue>> = vec![];
        for id in &guild_ids {
            let mut key = HashMap::new();
            key.insert("guild_id".to_string(), AttributeValue::S(id.clone()));
            keys.push(key);
        }

        let keys_and_attributes = KeysAndAttributes::builder()
            .set_keys(Some(keys))
            .build()
            .unwrap();

        let mut request_items = HashMap::new();
        request_items.insert(Guild::table_name(), keys_and_attributes);

        dynamo
            .batch_get_item()
            .set_request_items(Some(request_items))
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            .and_then(|resp| {
                let items = resp.responses.unwrap_or_default();
                if items.is_empty() {
                    return Err(StatusCode::NOT_FOUND);
                }
                let guilds: Vec<Guild> = items
                    .into_iter()
                    .flat_map(|(_, v)| v.into_iter().map(|item| Guild::from(&item)))
                    .collect();
                Ok(guilds)
            })
            .unwrap_or_else(|e| {
                error!("Error fetching guilds from DynamoDB: {}", e);
                vec![]
            })
    }

    pub async fn save(&self, dynamo: &aws_sdk_dynamodb::Client) {
        match dynamo
            .put_item()
            .table_name(format!(
                "kb2_guilds_{}",
                std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set")
            ))
            .set_item(Some(self.clone().into()))
            .send()
            .await
        {
            Ok(_) => (),
            Err(e) => {
                error!("DynamoDB write error: {}", e);
                panic!("Failed to save guild to DynamoDB");
            }
        }
    }
}
