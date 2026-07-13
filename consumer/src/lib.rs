//! Consumer crate library: the SQS message handlers (audit + verify
//! reconciliation worker). Split out of `main.rs` so integration and
//! performance tests (`consumer/tests/`) can drive the worker directly —
//! binary-only crates cannot be linked from integration tests.

pub mod audit;
pub mod verify;

use std::sync::Arc;

use aws_lambda_events::event::sqs::{SqsBatchResponse, SqsEvent};
use lambda_runtime::{Error, LambdaEvent};
use sqlx::{Pool, Postgres};
use tracing::{error, info};

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: Pool<Postgres>,
    pub sqs: aws_sdk_sqs::Client,
    pub discord_bot: Arc<twilight_http::Client>,
}

pub async fn function_handler(
    event: LambdaEvent<SqsEvent>,
    state: &AppState,
) -> Result<SqsBatchResponse, Error> {
    let mut response = SqsBatchResponse::default();

    for message in event.payload.records {
        let message_id = message.message_id.clone().unwrap_or_default();
        info!("Message body: {:?}", message.body);
        match message.message_attributes.get("kind") {
            Some(attr) => {
                match attr.string_value.as_deref() {
                    Some("audit") => {
                        if let Err(e) = audit::consume(message.clone(), state).await {
                            // Report only this message as failed so the rest of the
                            // batch can still succeed, instead of panicking and
                            // causing the whole batch to be redelivered forever.
                            error!("Failed to process audit message {}: {}", message_id, e);
                            response.add_failure(message_id);
                        }
                    }
                    Some(common::verify::RECON_MESSAGE_KIND) => {
                        if let Err(e) = verify::consume(message.clone(), state).await {
                            // Redelivery resumes from the job's checkpoint; the
                            // lease will have expired by the time it arrives.
                            error!(
                                "Failed to process verify_recon message {}: {}",
                                message_id, e
                            );
                            response.add_failure(message_id);
                        }
                    }
                    _ => error!(
                        "Unknown message type: {}; body: {}",
                        attr.string_value.as_deref().unwrap_or_default(),
                        message.body.as_deref().unwrap_or_default()
                    ),
                }
            }
            _ => error!(
                "Unknown message kind; body: {}",
                message.body.as_deref().unwrap_or_default()
            ),
        }
    }

    Ok(response)
}
