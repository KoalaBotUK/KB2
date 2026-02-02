use aws_lambda_events::sqs::SqsMessage;
use common::audit::AuditMessage;
use crate::AppState;
use crate::audit::models::Audit;

pub async fn consume(message: SqsMessage, state: &AppState) {
    let audit: Audit = serde_json::from_str::<AuditMessage<String>>(message.body.unwrap().as_str()).unwrap().into();
    audit.save(&state.pg_pool).await;
}