use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Guild {
    guild_id: String,
    verify: String
}