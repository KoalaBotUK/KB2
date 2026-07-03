use sqlx::{Pool, Postgres};
use sqlx::types::BigDecimal;
use tracing::error;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use common::audit::AuditMessage;

pub struct Audit {
    sqs_message_id: Option<String>, // Stable dedupe key from the SQS message, used to ignore at-least-once redeliveries
    event: String,
    user_id: Id<UserMarker>,
    guild_id: Option<Id<GuildMarker>>,
    old_data: Option<String>,
    new_data: Option<String>
}

impl Audit {
    pub async fn save(&self, pg_pool: &Pool<Postgres>) {
        // `id` is DB-generated (gen_random_uuid()) and never bound here, so it can never
        // collide and can't be used to dedupe retried SQS deliveries. Instead we dedupe on
        // the SQS message id, which stays stable across redeliveries of the same message.
        match sqlx::query("INSERT INTO audit (event, user_id, guild_id, old_data, new_data, sqs_message_id) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (sqs_message_id) DO NOTHING")
            .bind(self.event.as_str())
            .bind(BigDecimal::from(self.user_id.into_nonzero().get()))
            .bind(self.guild_id.map(|id| BigDecimal::from(id.into_nonzero().get())))
            .bind(&self.old_data)
            .bind(&self.new_data)
            .bind(&self.sqs_message_id)
            .execute(pg_pool)
            .await {
            Ok(_) => (),
            Err(e) => {
                error!("Error saving audit to DB: {}", e);
            }
        }
    }
}

impl From<(Option<String>, AuditMessage<String>)> for Audit {
    fn from((sqs_message_id, value): (Option<String>, AuditMessage<String>)) -> Self {
        Audit{
            sqs_message_id,
            event: value.event,
            user_id: value.user_id,
            guild_id: value.guild_id,
            old_data: value.data.old_data,
            new_data: value.data.new_data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression test for https://github.com/KoalaBotUK/KB2/issues/31
    //
    // `Audit::save` dedupes on `sqs_message_id` via `ON CONFLICT (sqs_message_id) DO NOTHING`,
    // which only works if the SQS message id is actually threaded from the incoming
    // `SqsMessage` through to the `Audit` value that gets inserted. This test pins down that
    // data-plumbing: a `message_id` provided to the `From<(Option<String>, AuditMessage<_>)>>`
    // conversion must land unchanged in `Audit.sqs_message_id`.
    #[test]
    fn from_threads_sqs_message_id_into_audit() {
        let message_id = Some("11111111-1111-1111-1111-111111111111".to_string());
        let audit_message = AuditMessage::new(
            "update_link_guilds".to_string(),
            Id::new(228541431483072513),
            Some(Id::new(590643624358969350)),
            Some("old".to_string()),
            Some("new".to_string()),
        );

        let audit: Audit = (message_id.clone(), audit_message).into();

        assert_eq!(audit.sqs_message_id, message_id);
    }

    // Redeliveries with no SQS message id (shouldn't normally happen, but the field is
    // `Option`) must not silently invent a value - `None` in means `None` out.
    #[test]
    fn from_threads_absent_sqs_message_id_into_audit() {
        let audit_message = AuditMessage::new(
            "update_link_guilds".to_string(),
            Id::new(228541431483072513),
            None,
            None,
            None,
        );

        let audit: Audit = (None, audit_message).into();

        assert_eq!(audit.sqs_message_id, None);
    }
}