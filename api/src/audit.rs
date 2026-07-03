use aws_sdk_sqs::types::MessageAttributeValue;
use lambda_http::tracing::error;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
pub use common::audit::AuditMessage;

const MAX_ATTEMPTS: u32 = 3;

pub async fn audit<T>(audit: AuditMessage<T>, sqs: &aws_sdk_sqs::Client)
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let queue_url = std::env::var("SQS_URL").expect("SQS_URL must be set");

    let old_data = match audit.data.old_data.map(|v: T| serde_json::to_string(&v)) {
        Some(Ok(s)) => Some(s),
        Some(Err(e)) => {
            error!("Failed to serialize audit old_data, dropping audit: {}", e);
            return;
        }
        None => None,
    };
    let new_data = match audit.data.new_data.map(|v: T| serde_json::to_string(&v)) {
        Some(Ok(s)) => Some(s),
        Some(Err(e)) => {
            error!("Failed to serialize audit new_data, dropping audit: {}", e);
            return;
        }
        None => None,
    };

    let new_audit = AuditMessage::new(audit.event, audit.user_id, audit.guild_id, old_data, new_data);

    let body = match serde_json::to_string(&new_audit) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to serialize audit message, dropping audit: {}", e);
            return;
        }
    };

    let kind_attribute = match MessageAttributeValue::builder()
        .data_type("String")
        .string_value("audit")
        .build()
    {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to build audit message attribute, dropping audit: {}", e);
            return;
        }
    };

    for attempt in 1..=MAX_ATTEMPTS {
        match sqs
            .send_message()
            .queue_url(&queue_url)
            .message_attributes("kind", kind_attribute.clone())
            .message_body(&body)
            .send()
            .await
        {
            Ok(_) => return,
            Err(e) => {
                if attempt >= MAX_ATTEMPTS {
                    error!(
                        "Failed to send audit to SQS after {} attempts, dropping audit: {} (message: {})",
                        MAX_ATTEMPTS, e, &new_audit
                    );
                } else {
                    error!("Failed to send audit to SQS (attempt {}/{}), retrying... {}", attempt, MAX_ATTEMPTS, e);
                    sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
                }
            }
        }
    }
}