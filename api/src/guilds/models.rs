use crate::dynamo::{as_string, as_string_opt};
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes};
use http::StatusCode;
use lambda_http::tracing::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use crate::discord::ise;
use crate::guilds::verify::models::Verify;
use crate::guilds::votes::models::Vote;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Guild {
    pub guild_id: Id<GuildMarker>,
    pub verify: Verify,
    pub vote: Vote,
}

impl Default for Guild {
    fn default() -> Self {
        Guild {
            guild_id: Id::new(1),
            verify: Verify { roles: vec![], user_links: HashMap::new() },
            vote: Vote::default(),
        }
    }
}

impl From<&HashMap<String, AttributeValue>> for Guild {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        Guild {
            guild_id: as_string(item.get("guild_id"), &"".to_string())
                .parse::<u64>()
                .map(Id::new)
                .unwrap(),
            verify: as_string_opt(item.get("verify")).map(|v| serde_json::from_str(&v).unwrap_or_default()).unwrap_or_default(),
            vote: as_string_opt(item.get("vote")).map(|v| serde_json::from_str(&v).unwrap_or_default()).unwrap_or_default(),
        }
    }
}

impl From<Guild> for HashMap<String, AttributeValue> {
    fn from(guild: Guild) -> Self {
        let mut item = HashMap::new();
        item.insert("guild_id".to_string(), AttributeValue::S(guild.guild_id.to_string()));
        item.insert("verify".to_string(), AttributeValue::S(serde_json::to_string(&guild.verify).unwrap()));
        item.insert("vote".to_string(), AttributeValue::S(serde_json::to_string(&guild.vote).unwrap()));

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

    pub async fn from_db(guild_id: Id<GuildMarker>, dynamo: &aws_sdk_dynamodb::Client) -> Option<Guild> {
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
            .map_err(ise)
            .map(|resp|
                match resp.item {
                    Some(item) => Guild::from(&item),
                    None => Guild{guild_id, ..Default::default()}
                }
            ) {
            Ok(guild) => Some(guild),
            Err(e) => {
                error!("Error fetching guild from DynamoDB: {}", e);
                None
            }
        }
    }

    pub async fn vec_from_db(
        guild_ids: Vec<Id<GuildMarker>>,
        dynamo: &aws_sdk_dynamodb::Client,
    ) -> Vec<Guild> {
        let mut keys: Vec<HashMap<String, AttributeValue>> = vec![];
        for id in &guild_ids {
            let mut key = HashMap::new();
            key.insert("guild_id".to_string(), AttributeValue::S(id.to_string()));
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
            .map_err(ise)
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
        info!("Before savbe guild to DynamoDB");
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
            Ok(_) => {
                info!("Saved guild to DynamoDB");
                
            },
            Err(e) => {
                error!("DynamoDB write error: {}", e);
                panic!("Failed to save guild to DynamoDB");
            }
        }
    }
}
