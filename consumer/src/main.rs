mod audit;
mod verify;

use std::sync::Arc;

use aws_lambda_events::event::sqs::{SqsBatchResponse, SqsEvent};
use aws_sdk_dsql::config::BehaviorVersion;
use common::dsql::establish_connection;
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use sqlx::{Pool, Postgres};
use tracing::{error, info};

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: Pool<Postgres>,
    pub sqs: aws_sdk_sqs::Client,
    pub discord_bot: Arc<twilight_http::Client>,
}

async fn function_handler(event: LambdaEvent<SqsEvent>, state: &AppState) -> Result<SqsBatchResponse, Error> {
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
                            error!("Failed to process verify_recon message {}: {}", message_id, e);
                            response.add_failure(message_id);
                        }
                    }
                    _ => error!("Unknown message type: {}; body: {}", attr.string_value.as_deref().unwrap_or_default(), message.body.as_deref().unwrap_or_default())
                }

            }
            _ => error!("Unknown message kind; body: {}", message.body.as_deref().unwrap_or_default())
        }
    }

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().json()
        .with_max_level(tracing::Level::INFO)
        // this needs to be set to remove duplicated information in the log.
        .with_current_span(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        // remove the name of the function from every log entry
        .with_target(false)
        .init();
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    let pool = establish_connection(
        std::env::var("DSQL_USER").expect("env variable `DSQL_USER` should be set"),
        std::env::var("DSQL_ENDPOINT").expect("env variable `DSQL_ENDPOINT` should be set"),
        config.region().unwrap(),
    )
        .await?;

    let discord_bot = Arc::new(
        twilight_http::Client::builder()
            .token(std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN must be set"))
            .build(),
    );

    let state = AppState {
        pg_pool: pool,
        sqs: aws_sdk_sqs::Client::new(&config),
        discord_bot,
    };

    run(service_fn(|event: LambdaEvent<SqsEvent>| {
        function_handler(event, &state)
    })).await
}
