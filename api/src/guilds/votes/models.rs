use crate::dynamo::{as_bool, as_map_vec, as_string, as_string_opt, as_u64, as_u64_set};
use crate::guilds::votes::utils::VoteOptionComponent;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use twilight_model::channel::message::EmojiReactionType;
use twilight_model::id::marker::{ChannelMarker, MessageMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VoteOption {
    pub emoji: Option<EmojiReactionType>,
    pub label: Option<String>,
    pub users: HashSet<Id<UserMarker>>
}

impl From<&HashMap<String, AttributeValue>> for VoteOption {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        VoteOption {
            emoji: as_string_opt(item.get("emoji")).filter(|s| !s.is_empty()).and_then(|v| serde_json::from_str(&*v).unwrap()),
            label: as_string_opt(item.get("label")),
            users: as_u64_set(item.get("label")).into_iter().map(|s| Id::new(s)).collect(),
        }
    }
}

impl From<VoteOption> for HashMap<String, AttributeValue> {
    fn from(vote_option: VoteOption) -> Self {
        let mut item = HashMap::new();
        item.insert("emoji".to_string(), AttributeValue::S(serde_json::to_string(&vote_option.emoji).unwrap()));
        item.insert("label".to_string(), AttributeValue::S(vote_option.label.unwrap()));
        item.insert("users".to_string(), AttributeValue::L(vote_option.users.iter().map(|u| AttributeValue::S(u.to_string())).collect()));
        item
    }
}

impl VoteOptionComponent for VoteOption {
    fn emoji(&self) -> &Option<EmojiReactionType> {
        &self.emoji
    }

    fn label(&self) -> &Option<String> {
        &self.label
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum RoleListType {
    #[default]
    BLACKLIST,
    WHITELIST
}

impl Display for RoleListType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct VoteVote {
    pub message_id: Id<MessageMarker>,
    pub title: String,
    pub description: String,
    pub options: Vec<VoteOption>,
    pub channel_id: Id<ChannelMarker>,
    pub close_at: Option<DateTime<Utc>>,
    pub open: bool,
    pub role_list: HashSet<Id<RoleMarker>>,
    pub role_list_type: RoleListType
}

impl From<&HashMap<String, AttributeValue>> for VoteVote {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        VoteVote {
            message_id: Id::new(as_u64(item.get("message_id"), 0)),
            title: as_string(item.get("title"), &"".to_string()),
            description: as_string(item.get("description"), &"".to_string()),
            options: as_map_vec(item.get("options")).iter().map(|&v| VoteOption::from(v)).collect(),
            channel_id: Id::new(as_u64(item.get("channel_id"), 0)),
            close_at: as_string_opt(item.get("close_at")).filter(|v| !v.is_empty()).and_then(|v| serde_json::from_str(&*v).unwrap()),
            open: as_bool(item.get("open"), false),
            role_list: as_u64_set(item.get("role_list")).into_iter().map(|s| Id::new(s)).collect(),
            role_list_type: as_string_opt(item.get("role_list_type")).filter(|v| !v.is_empty()).and_then(|v| serde_json::from_str(&*v).unwrap()).unwrap_or_default(),
        }
    }
}

impl From<VoteVote> for HashMap<String, AttributeValue> {
    fn from(vote: VoteVote) -> Self {
        let mut item = HashMap::new();
        item.insert("message_id".to_string(), AttributeValue::N(vote.message_id.to_string()));
        item.insert("title".to_string(), AttributeValue::S(vote.title));
        item.insert("description".to_string(), AttributeValue::S(vote.description));
        item.insert("options".to_string(), AttributeValue::L(vote.options.iter().map(|v| AttributeValue::M(v.clone().into())).collect()));
        item.insert("channel_id".to_string(), AttributeValue::N(vote.channel_id.to_string()));
        item.insert("close_at".to_string(), AttributeValue::S(serde_json::to_string(&vote.close_at).unwrap()));
        item.insert("open".to_string(), AttributeValue::Bool(vote.open));
        item.insert("role_list".to_string(), AttributeValue::L(vote.role_list.iter().map(|u| AttributeValue::N(u.to_string())).collect()));
        item.insert("role_list_type".to_string(), AttributeValue::S(serde_json::to_string(&vote.role_list_type).unwrap()));
        item
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Vote {
    pub votes: Vec<VoteVote>
}

impl From<&HashMap<String, AttributeValue>> for Vote {
    fn from(item: &HashMap<String, AttributeValue>) -> Self {
        Vote {
            votes: as_map_vec(item.get("votes")).iter().map(|&v| VoteVote::from(v)).collect(),
        }
    }
}

impl From<Vote> for HashMap<String, AttributeValue> {
    fn from(vote: Vote) -> Self {
        let mut item = HashMap::new();
        item.insert("votes".to_string(), AttributeValue::L(vote.votes.iter().map(|v| AttributeValue::M(v.clone().into())).collect()));
        item
    }
}