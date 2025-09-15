use std::{env, error::Error};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use twilight_gateway::{EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_model::http::interaction::InteractionResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_BOT_TOKEN")?;
    let intents = Intents::GUILD_MESSAGES;
    let mut shard = Shard::new(ShardId::ONE, token, intents);
    let token = env::var("DISCORD_BOT_TOKEN")?;
    let client = reqwest::Client::new();
    tracing::info!("created shard");

    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = item else {
            tracing::warn!(source = ?item.unwrap_err(), "error receiving event");

            continue;
        };

        tracing::debug!(?event, "event");

        match event {
            twilight_gateway::Event::InteractionCreate(interaction) => {
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
                headers.insert(AUTHORIZATION, format!("Bot {token}").parse().unwrap());
                headers.insert("x-signature-timestamp", "mock".parse().unwrap());
                headers.insert("x-signature-ed25519", "mock".parse().unwrap());

                let r = client.post("http://localhost:9000/lambda-url/api/interactions").json(&interaction).headers(headers).send().await;
                match r {
                    Ok(r) => {
                        let headers = r.headers().clone();
                        client.post(format!("https://discord.com/api/v10/interactions/{}/{}/callback",interaction.id, interaction.token))
                            .json(&r.json::<InteractionResponse>().await.unwrap())
                            .headers(headers).send().await.unwrap();
                    },
                    Err(e) => {
                        tracing::error!(?e, "error sending interaction");
                    }
                }
            },
            _ => (),
        }
    }

    Ok(())
}