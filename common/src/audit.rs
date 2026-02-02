use std::fmt::Display;
use serde::{Deserialize, Serialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

#[derive(Serialize, Deserialize)]
#[serde(bound = "T: Serialize + for<'a> Deserialize<'a>")]
pub struct AuditData<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    pub old_data: Option<T>,
    pub new_data: Option<T>
}

#[derive(Serialize, Deserialize)]
#[serde(bound = "T: Serialize + for<'a> Deserialize<'a>")]
pub struct AuditMessage<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    pub event: String,
    pub user_id: Id<UserMarker>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub data: AuditData<T>
}

impl<T> AuditMessage<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    pub fn new(event: String, user_id: Id<UserMarker>, guild_id: Option<Id<GuildMarker>>, old_data: Option<T>, new_data: Option<T>) -> AuditMessage<T> {
        AuditMessage {
            event,
            user_id,
            guild_id,
            data: AuditData{old_data,new_data}
        }
    }
}

impl<T> Display for AuditMessage<T>
where
    T: Serialize + for<'a> Deserialize<'a>, {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}