use std::sync::Arc;

use aws_lambda_events::event::sqs::SqsEvent;
use aws_sdk_dsql::config::BehaviorVersion;
use common::dsql::establish_connection;
use consumer::{AppState, function_handler};
use lambda_runtime::{Error, LambdaEvent, run, service_fn};

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
