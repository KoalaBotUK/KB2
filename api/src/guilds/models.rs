use std::collections::HashMap;
use aws_sdk_dynamodb::types::AttributeValue;
use http::StatusCode;
use lambda_http::tracing::{error, info};
use serde::{Deserialize, Serialize};
use crate::dynamo::{as_map, as_map_vec, as_string};

#[derive(Clone, Serialize, Deserialize)]
pub struct VerifyRole {
    pub role_id: String,
    pub role_name: String,
    pub pattern: String,
    pub member_count: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Verify {
    pub roles: Vec<VerifyRole>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Guild {
    pub guild_id: String,
    pub verify: Verify
}

impl From<&HashMap<String, AttributeValue>> for Guild {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        Guild {
            guild_id: as_string(item.get("guild_id"), &"".to_string()),
            verify: as_map(item.get("verify")).map(
                |m| Verify {
                    roles: as_map_vec(m.get("roles"))
                        .into_iter()
                        .map(|role_map| VerifyRole {
                            role_id: as_string(role_map.get("role_id"), &"".to_string()),
                            role_name: as_string(role_map.get("role_name"), &"".to_string()),
                            pattern: as_string(role_map.get("pattern"), &"".to_string()),
                            member_count: role_map.get("member_count")
                                .and_then(|v| v.as_n().ok())
                                .and_then(|n| n.parse::<u32>().ok())
                                .unwrap_or(0),
                        })
                        .collect(),
                }).unwrap_or(Verify { roles: vec![] }),
        }
    }
}

impl Into<HashMap<String, AttributeValue>> for Guild {
    fn into(self) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        item.insert("guild_id".to_string(), AttributeValue::S(self.guild_id));

        let roles: Vec<AttributeValue> = self.verify.roles.into_iter().map(|role| {
            let mut role_map = HashMap::new();
            role_map.insert("role_id".to_string(), AttributeValue::S(role.role_id));
            role_map.insert("role_name".to_string(), AttributeValue::S(role.role_name));
            role_map.insert("pattern".to_string(), AttributeValue::S(role.pattern));
            role_map.insert("member_count".to_string(), AttributeValue::N(role.member_count.to_string()));
            AttributeValue::M(role_map)
        }).collect();

        item.insert("verify".to_string(), AttributeValue::M(HashMap::from([("roles".to_string(), AttributeValue::L(roles))])));

        item
    }
}

impl Guild {
    pub async fn from_db(guild_id: &str, dynamo: &aws_sdk_dynamodb::Client) -> Option<Guild> {
        info!("a");
        let a = dynamo.query().table_name(format!("kb2_guilds_{}",std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set"),));
        info!("b");
        let b = a.key_condition_expression("#uid = :uid");
        info!("c");
        let c = b.expression_attribute_names("#uid", "guild_id");
        info!("d");
        let d = c.expression_attribute_values(":uid", AttributeValue::S(guild_id.to_string()));
        info!("e");
        let e = d.send();
        info!("f");
        let f = e.await;
        info!("g");

        match f
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            .and_then(|resp| {
                info!("100");
                let items = resp.items.unwrap_or_default();
                info!("200");
                if items.is_empty() {
                    return Err(StatusCode::NOT_FOUND);
                }
                let guild: Guild = (&items[0]).into();
                Ok(guild)
            }) {
            Ok(guild) => Some(guild),
            Err(e) => {
                error!("Error fetching guild from DynamoDB: {}", e);
                None
            }
        }

    }

    pub async fn save(&self, dynamo: &aws_sdk_dynamodb::Client) {

        match dynamo
            .put_item()
            .table_name(format!("kb2_guilds_{}", std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set")))
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