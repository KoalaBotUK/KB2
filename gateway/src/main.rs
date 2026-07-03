use std::{env, error::Error};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use twilight_gateway::{EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_model::http::interaction::InteractionResponse;

/// Parse the body of the local API's interaction response into an
/// `InteractionResponse` we can relay back to Discord.
///
/// This is split out from the event loop (see issue #53) so that a
/// malformed/unexpected body (bad JSON, an error payload, an empty body,
/// etc.) is reported as an `Err` instead of panicking via `.unwrap()`,
/// which used to take down the entire shard loop on a single bad
/// interaction.
fn parse_interaction_response(body: &[u8]) -> Result<InteractionResponse, serde_json::Error> {
    serde_json::from_slice(body)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize the tracing subscriber.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_BOT_TOKEN")?;
    let intents = Intents::GUILD_MESSAGES;
    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);
    let client = reqwest::Client::new();
    tracing::info!("created shard");

    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = item else {
            tracing::warn!(source = ?item.unwrap_err(), "error receiving event");

            continue;
        };

        tracing::debug!(?event, "event");

        if let twilight_gateway::Event::InteractionCreate(interaction) = event {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
            headers.insert(AUTHORIZATION, format!("Bot {token}").parse().unwrap());
            headers.insert("x-signature-timestamp", "mock".parse().unwrap());
            headers.insert("x-signature-ed25519", "mock".parse().unwrap());

            let r = client.post("http://localhost:9000/lambda-url/api/interactions").json(&interaction).headers(headers).send().await;
            match r {
                Ok(r) => {
                    match r.status() {
                        reqwest::StatusCode::OK => {
                            let headers = r.headers().clone();
                            match r.bytes().await {
                                Ok(bytes) => match parse_interaction_response(&bytes) {
                                    Ok(body) => {
                                        let callback_url = format!("https://discord.com/api/v10/interactions/{}/{}/callback", interaction.id, interaction.token);
                                        if let Err(e) = client.post(callback_url).json(&body).headers(headers).send().await {
                                            tracing::error!(?e, "failed to post interaction callback");
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(?e, "invalid interaction response body");
                                    }
                                },
                                Err(e) => {
                                    tracing::error!(?e, "failed to read interaction response body");
                                }
                            }
                        },
                        _ => {
                            tracing::error!(?r, "error sending interaction");
                            continue;
                        }
                    }
                },
                Err(e) => {
                    tracing::error!(?e, "error sending interaction");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for issue #53: previously this path was
    /// `r.json::<InteractionResponse>().await.unwrap()`, which would panic
    /// (and take down the whole shard loop) whenever the local API returned
    /// a body that wasn't a valid `InteractionResponse` — e.g. an error
    /// payload. The fix replaced the unwrap with proper error handling, and
    /// `parse_interaction_response` is the extracted piece of that logic.
    #[test]
    fn parse_interaction_response_rejects_malformed_body_instead_of_panicking() {
        // Something that is valid JSON but not a valid InteractionResponse
        // (e.g. an error object the local API might return on failure).
        let bad_body = br#"{"error": "internal server error"}"#;

        let result = parse_interaction_response(bad_body);

        assert!(
            result.is_err(),
            "malformed interaction response body should be rejected via Err, not panic"
        );
    }

    /// Also cover a completely empty body (e.g. from a 200 with no content),
    /// which is another way the old `.unwrap()` used to panic.
    #[test]
    fn parse_interaction_response_rejects_empty_body_instead_of_panicking() {
        let result = parse_interaction_response(b"");

        assert!(
            result.is_err(),
            "empty interaction response body should be rejected via Err, not panic"
        );
    }

    /// A well-formed InteractionResponse body should still parse
    /// successfully, so the fix doesn't reject valid responses along with
    /// the invalid ones.
    #[test]
    fn parse_interaction_response_accepts_valid_body() {
        let good_body = br#"{"type": 4, "data": {"content": "pong"}}"#;

        let result = parse_interaction_response(good_body);

        assert!(
            result.is_ok(),
            "valid interaction response body should parse successfully, got: {result:?}"
        );
    }
}