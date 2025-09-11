use std::{env, error::Error};
use twilight_gateway::{EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_BOT_TOKEN")?;
    let intents = Intents::GUILD_MESSAGES;
    let mut shard = Shard::new(ShardId::ONE, token, intents);
    tracing::info!("created shard");

    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = item else {
            tracing::warn!(source = ?item.unwrap_err(), "error receiving event");

            continue;
        };

        tracing::debug!(?event, "event");
    }

    Ok(())
}