use crate::guilds::votes::utils::VoteOptionComponent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet};
use std::fmt::{Display, Formatter};
use twilight_model::channel::message::EmojiReactionType;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, MessageMarker, RoleMarker, UserMarker};

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VoteOption {
    pub emoji: Option<EmojiReactionType>,
    pub label: Option<String>,
    pub users: HashSet<Id<UserMarker>>,
}

impl Display for VoteOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label.as_ref().unwrap())
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
    #[serde(rename = "BLACKLIST")]
    Blacklist,
    #[serde(rename = "WHITELIST")]
    Whitelist,
}

impl Display for RoleListType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

fn default_multi_select() -> bool {
    true
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
    pub role_list_type: RoleListType,
    #[serde(default = "default_multi_select")]
    pub is_multi_select: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Vote {
    pub votes: Vec<VoteVote>,
}

