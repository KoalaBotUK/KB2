use aws_lambda_events::sqs::SqsMessage;
use common::audit::AuditMessage;
use crate::AppState;
use crate::audit::models::Audit;

pub async fn consume(message: SqsMessage, state: &AppState) {
    let audit: Audit = serde_json::from_str::<AuditMessage<String>>(message.body.unwrap().as_str()).unwrap().into();
    audit.save(&state.pg_pool).await;
}

#[cfg(test)]
mod tests {
    use serde_json::Error;
    use tracing::{debug};
    use super::*;

    #[test]
    fn test_decode() -> Result<(), Error> {
        let mut example_sqs_message = SqsMessage::default();
        example_sqs_message.body = Some(r#"{"event":"update_link_guilds","user_id":"228541431483072513","guild_id":"590643624358969350","data":{"old_data":"[{\"guild_id\":\"863362407183286302\",\"enabled\":true}]","new_data":"[{\"guild_id\":\"863362407183286302\",\"enabled\":true},{\"guild_id\":\"590643624358969350\",\"enabled\":true}]"}}"#.to_string());
        let audit_msg = serde_json::from_str::<AuditMessage<String>>(example_sqs_message.body.unwrap().as_str())?;
        debug!("audit_msg: {:?}", audit_msg);
        let _audit: Audit = audit_msg.into();

        assert_eq!(1, 1);
        Ok(())
    }

}
