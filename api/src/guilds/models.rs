use crate::dynamo::{as_map, as_map_vec, as_string, as_string_opt};
use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes};
use http::StatusCode;
use lambda_http::tracing::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};
use twilight_model::util::ImageHash;
use crate::users::models::Link;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct VerifyRole {
    pub role_id: Id<RoleMarker>,
    pub role_name: Option<String>,
    pub pattern: String,
    pub members: u32,
}

impl From<&HashMap<String, AttributeValue>> for VerifyRole {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        VerifyRole {
            role_id: Id::new(
                as_string(item.get("role_id"), &"0".to_string())
                    .parse::<u64>()
                    .unwrap_or(0),
            ),
            role_name: as_string_opt(item.get("role_name")),
            pattern: as_string(item.get("pattern"), &"".to_string()),
            members: as_string(item.get("members"), &"0".to_string())
                .parse::<u32>()
                .unwrap_or(0),
        }
    }
}

impl From<VerifyRole> for HashMap<String, AttributeValue> {
    fn from(role: VerifyRole) -> Self {
        let mut role_map = HashMap::new();
        role_map.insert("role_id".to_string(), AttributeValue::S(role.role_id.to_string()));
        if let Some(role_name) = role.role_name {
            role_map.insert("role_name".to_string(), AttributeValue::S(role_name));
        }
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

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Verify {
    pub roles: Vec<VerifyRole>,
    pub user_links: Vec<UserLink>
}

impl From<&HashMap<String, AttributeValue>> for Verify {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        let roles = as_map_vec(item.get("roles"))
            .into_iter()
            .map(|m| m.into())
            .collect();
        let user_links = as_map_vec(item.get("user_links"))
            .into_iter()
            .map(|m| m.into())
            .collect();
        Verify { roles, user_links }
    }
}

impl From<Verify> for HashMap<String, AttributeValue> {
    fn from(verify: Verify) -> Self {
        let mut verify_map = HashMap::new();
        let roles: Vec<AttributeValue> = verify.roles.into_iter().map(|r| AttributeValue::M(r.into())).collect();
        verify_map.insert("roles".to_string(), AttributeValue::L(roles));
        let user_links: Vec<AttributeValue> = verify.user_links.into_iter().map(|ul| AttributeValue::M(ul.into())).collect();
        verify_map.insert("user_links".to_string(), AttributeValue::L(user_links));
        verify_map
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Guild {
    pub guild_id: Id<GuildMarker>,
    pub verify: Verify,
    pub name: String,
    pub icon: Option<ImageHash>,
    pub user_links: HashMap<Id<UserMarker>, Vec<Link>>,
}

impl Default for Guild {
    fn default() -> Self {
        Guild {
            guild_id: Id::new(1),
            verify: Verify { roles: vec![], user_links: vec![] },
            name: "".to_string(),
            icon: None,
            user_links: HashMap::new(),
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
            verify: as_map(item.get("verify")).unwrap().into(),
            name: as_string(item.get("name"), &"".to_string()),
            icon: as_string_opt(item.get("icon")).and_then(|s| ImageHash::parse(s.as_bytes()).ok()),
            user_links: as_map(item.get("user_links")).unwrap_or(&HashMap::new()).iter().map(
                |(k,v)| (Id::new(k.parse::<u64>().unwrap_or(0)), as_map_vec(Some(v)).iter().map(|&l| l.into()).collect()))
                .collect(),
        }
    }
}

impl From<Guild> for HashMap<String, AttributeValue> {
    fn from(guild: Guild) -> Self {
        let mut item = HashMap::new();
        item.insert("guild_id".to_string(), AttributeValue::S(guild.guild_id.to_string()));

        item.insert(
            "verify".to_string(), AttributeValue::M(guild.verify.into())
        );
        
        item.insert("name".to_string(), AttributeValue::S(guild.name));
        if let Some(icon) = guild.icon {
            item.insert("icon".to_string(), AttributeValue::S(icon.to_string()));
        }

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
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            .map(|resp| {
                let item = resp.item.unwrap_or_default();
                let guild: Guild = (&item).into();
                guild
            }) {
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
            .map_err(|e| {
                error!("Error fetching guilds from DynamoDB: {}", e); 
                StatusCode::INTERNAL_SERVER_ERROR
            })
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
