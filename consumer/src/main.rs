mod audit;

use aws_lambda_events::event::sqs::SqsEvent;
use aws_sdk_dsql::config::BehaviorVersion;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use sqlx::{Pool, Postgres};
use tracing::{error, info};
use common::dsql::establish_connection;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: Pool<Postgres>,
}

async fn function_handler(event: LambdaEvent<SqsEvent>, state: &AppState) -> Result<(), Error> {
    for message in event.payload.records {
        info!("Message body: {:?}", message.body);
        match message.message_attributes.get("kind") {
            Some(attr) => {
                match attr.string_value.as_deref() {
                    Some("audit") => audit::consume(message.clone(), state).await,
                    _ => error!("Unknown message type: {}; body: {}", attr.string_value.as_deref().unwrap_or_default(), message.body.as_deref().unwrap_or_default())
                }

            }
            _ => error!("Unknown message kind; body: {}", message.body.as_deref().unwrap_or_default())
        }
    }

    Ok(())
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

    let state = AppState {
        pg_pool: pool,
    };

    run(service_fn(|event: LambdaEvent<SqsEvent>| {
        function_handler(event, &state)
    })).await
}
