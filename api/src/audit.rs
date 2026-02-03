use aws_sdk_sqs::types::MessageAttributeValue;
use lambda_http::tracing::error;
use serde::{Deserialize, Serialize};
pub use common::audit::AuditMessage;

pub async fn audit<T>(audit: &AuditMessage<T>, sqs: &aws_sdk_sqs::Client)
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let queue_url = std::env::var("SQS_URL").expect("SQS_URL must be set");

    let mut attempts = 0;
    
    loop {
        attempts += 1;

        match sqs
            .send_message()
            .queue_url(&queue_url)
            .message_attributes("kind", MessageAttributeValue::builder().string_value("audit").build().unwrap())
            .message_body(serde_json::to_string(audit).unwrap())
            .send()
            .await {
            Ok(_) => break,
            Err(e) => {
                if attempts >= 3 {
                    error!("Failed to send audit to SQS after 3 attempts {}", audit);
                    break;
                } else {
                    error!("Failed to send audit to SQS, retrying... {}", e);
                }
            }
        }
    }
}