use aws_lambda_events::sqs::SqsMessage;
use common::audit::AuditMessage;
use lambda_runtime::Error;
use crate::AppState;
use crate::audit::models::Audit;

/// Parse an SQS message body into an `Audit`, without performing any I/O.
///
/// This is split out from [`consume`] so a malformed/missing body results in
/// an `Err` that the caller can report as a single failed batch item, rather
/// than panicking and taking down the whole Lambda invocation (and with it,
/// every other message in the batch).
///
/// The SQS message id is threaded through into the resulting `Audit` so that
/// `Audit::save` can dedupe on it (redeliveries of the same message keep the
/// same message id), rather than on the DB-generated `id` column.
fn parse_audit(message: &SqsMessage) -> Result<Audit, Error> {
    let message_id = message.message_id.clone();
    let body = message.body.as_deref().ok_or("missing SQS message body")?;
    let audit_message = serde_json::from_str::<AuditMessage<String>>(body)?;
    Ok((message_id, audit_message).into())
}

pub async fn consume(message: SqsMessage, state: &AppState) -> Result<(), Error> {
    let audit = parse_audit(&message)?;
    audit.save(&state.pg_pool).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::Error as JsonError;
    use tracing::{debug};
    use super::*;

    #[test]
    fn test_decode() -> Result<(), JsonError> {
        let mut example_sqs_message = SqsMessage::default();
        example_sqs_message.message_id = Some("11111111-1111-1111-1111-111111111111".to_string());
        example_sqs_message.body = Some(r#"{"event":"update_link_guilds","user_id":"228541431483072513","guild_id":"590643624358969350","data":{"old_data":"[{\"guild_id\":\"863362407183286302\",\"enabled\":true}]","new_data":"[{\"guild_id\":\"863362407183286302\",\"enabled\":true},{\"guild_id\":\"590643624358969350\",\"enabled\":true}]"}}"#.to_string());
        let audit_msg = serde_json::from_str::<AuditMessage<String>>(example_sqs_message.body.clone().unwrap().as_str())?;
        debug!("audit_msg: {:?}", audit_msg);
        let _audit: Audit = (example_sqs_message.message_id.clone(), audit_msg).into();

        assert_eq!(1, 1);
        Ok(())
    }

    /// Regression test for a single malformed message stalling/re-delivering
    /// the whole SQS batch forever: `parse_audit` must return an `Err`
    /// instead of panicking when the body isn't valid JSON for `AuditMessage`.
    #[test]
    fn test_parse_audit_malformed_json_returns_error_not_panic() {
        let mut malformed_message = SqsMessage::default();
        malformed_message.body = Some("this is not valid json at all".to_string());

        let result = parse_audit(&malformed_message);

        assert!(result.is_err(), "malformed message body should be reported as an error, not panic");
    }

    /// Regression test: a message with no body at all must also be handled
    /// gracefully rather than panicking on `.unwrap()`.
    #[test]
    fn test_parse_audit_missing_body_returns_error_not_panic() {
        let missing_body_message = SqsMessage::default();

        let result = parse_audit(&missing_body_message);

        assert!(result.is_err(), "missing message body should be reported as an error, not panic");
    }

    /// A well-formed message in the same batch as a malformed one must still
    /// be parsed successfully - partial batch failures should not affect it.
    #[test]
    fn test_parse_audit_well_formed_message_succeeds() {
        let mut well_formed_message = SqsMessage::default();
        well_formed_message.body = Some(r#"{"event":"update_link_guilds","user_id":"228541431483072513","guild_id":"590643624358969350","data":{"old_data":"[{\"guild_id\":\"863362407183286302\",\"enabled\":true}]","new_data":"[{\"guild_id\":\"863362407183286302\",\"enabled\":true},{\"guild_id\":\"590643624358969350\",\"enabled\":true}]"}}"#.to_string());

        let result = parse_audit(&well_formed_message);

        assert!(result.is_ok(), "well-formed message should still parse successfully");
    }
}
