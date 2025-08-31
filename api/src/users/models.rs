use crate::dynamo::{as_bool, as_map_vec, as_string, as_string_opt, as_u64};
use aws_sdk_dynamodb::types::AttributeValue;
use http::StatusCode;
use lambda_http::tracing::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::util::ImageHash;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub link_address: String,
    pub linked_at: u64,
    pub active: bool,
}

impl From<&HashMap<String, AttributeValue>> for Link {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        Link {
            link_address: as_string(item.get("link_address"), &"".to_string()),
            linked_at: as_u64(item.get("linked_at"), 0),
            active: as_bool(item.get("active"), false),
        }
    }
}

impl From<Link> for HashMap<String, AttributeValue> {
    fn from(link: Link) -> Self {
        let mut link_map = HashMap::new();
        link_map.insert("link_address".to_string(), AttributeValue::S(link.link_address));
        link_map.insert("linked_at".to_string(), AttributeValue::N(link.linked_at.to_string()));
        link_map.insert("active".to_string(), AttributeValue::Bool(link.active));
        link_map
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LinkGuild {
    pub guild_id: Id<GuildMarker>,
    pub name: String,
    pub icon: Option<ImageHash>,
    pub enabled: bool,
}


impl From<&HashMap<String, AttributeValue>> for LinkGuild {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        LinkGuild {
            guild_id: Id::new(
                as_string(item.get("guild_id"), &"0".to_string())
                    .parse::<u64>()
                    .unwrap_or(0),
            ),
            name: as_string(item.get("name"), &"".to_string()),
            icon: as_string_opt(item.get("icon")).and_then(|s| ImageHash::parse(s.as_bytes()).ok()),
            enabled: as_bool(item.get("enabled"), false),
        }
    }
}

impl From<LinkGuild> for HashMap<String, AttributeValue> {
    fn from(link_guild: LinkGuild) -> Self {
        let mut lg_map = HashMap::new();
        lg_map.insert("guild_id".to_string(), AttributeValue::S(link_guild.guild_id.to_string()));
        lg_map.insert("name".to_string(), AttributeValue::S(link_guild.name));
        if let Some(icon) = link_guild.icon {
            lg_map.insert("icon".to_string(), AttributeValue::S(icon.to_string()));
        }
        lg_map.insert("enabled".to_string(), AttributeValue::Bool(link_guild.enabled));
        lg_map
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: Id<UserMarker>,
    pub global_name: String,
    pub avatar: Option<ImageHash>,
    pub links: Vec<Link>,
    pub link_guilds: Vec<LinkGuild>,
}

impl Default for User {
    fn default() -> Self {
        User {
            user_id: Id::new(1),
            global_name: "".to_string(),
            avatar: None,
            links: vec![],
            link_guilds: vec![],
        }
    }
}

impl From<&HashMap<String, AttributeValue>> for User {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        User {
            user_id: Id::new(
                as_string(item.get("user_id"), &"0".to_string())
                    .parse::<u64>()
                    .unwrap_or(0),
            ),
            global_name: as_string(item.get("global_name"), &"".to_string()),
            avatar: as_string_opt(item.get("avatar")).and_then(|s| ImageHash::parse(s.as_bytes()).ok()),
            links: as_map_vec(item.get("links"))
                .into_iter()
                .map(|m| m.into())
                .collect(),
            link_guilds: as_map_vec(item.get("link_guilds"))
                .into_iter()
                .map(|m| m.into())
                .collect(),
        }
    }
}

impl From<User> for HashMap<String, AttributeValue> {
    fn from(user: User) -> Self {
        let mut user_map = HashMap::new();
        user_map.insert("user_id".to_string(), AttributeValue::S(user.user_id.to_string()));
        user_map.insert("global_name".to_string(), AttributeValue::S(user.global_name));
        if let Some(avatar) = user.avatar {
            user_map.insert("avatar".to_string(), AttributeValue::S(avatar.to_string()));
        }
        let links: Vec<AttributeValue> = user.links.into_iter().map(|l| AttributeValue::M(l.into())).collect();
        user_map.insert("links".to_string(), AttributeValue::L(links));
        let link_guilds: Vec<AttributeValue> = user.link_guilds.into_iter().map(|lg| AttributeValue::M(lg.into())).collect();
        user_map.insert("link_guilds".to_string(), AttributeValue::L(link_guilds));
        user_map
    }
}

impl User {
    pub async fn from_db(user_id: &str, dynamo: &aws_sdk_dynamodb::Client) -> Option<User> {
        match dynamo
            .query()
            .table_name(format!(
                "kb2_users_{}",
                std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set"),
            ))
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
        match dynamo
            .put_item()
            .table_name(format!(
                "kb2_users_{}",
                std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set")
            ))
            .set_item(Some(self.clone().into()))
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
