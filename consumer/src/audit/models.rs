use sqlx::{Pool, Postgres};
use sqlx::types::BigDecimal;
use tracing::error;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use uuid::Uuid;
use common::audit::AuditMessage;

pub struct Audit {
    id: Option<Uuid>, // Optional for when we want to insert a new audit
    action: String,
    user_id: Id<UserMarker>,
    guild_id: Option<Id<GuildMarker>>,
    old_data: Option<String>,
    new_data: Option<String>
}

impl Audit {
    pub async fn save(&self, pg_pool: &Pool<Postgres>) {
        match sqlx::query("INSERT INTO audit (id, action, user_id, guild_id, old_data, new_data) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO UPDATE SET event = $2, user_id = $3, guild_id = $4, old_data = $5, new_data = $6, updated_at = CURRENT_TIMESTAMP")
            .bind(self.id)
            .bind(self.action.as_str())
            .bind(BigDecimal::from(self.user_id.into_nonzero().get()))
            .bind(self.guild_id.map(|id| BigDecimal::from(id.into_nonzero().get())))
            .bind(&self.old_data)
            .bind(&self.new_data)
            .execute(pg_pool)
            .await {
            Ok(_) => (),
            Err(e) => {
                error!("Error saving audit to DB: {}", e);
            }
        }
    }
}

impl From<AuditMessage<String>> for Audit {
    fn from(value: AuditMessage<String>) -> Self {
        Audit{
            id: None,
            action: value.action,
            user_id: value.user_id,
            guild_id: value.guild_id,
            old_data: value.data.old_data,
            new_data: value.data.new_data
        }
    }
}