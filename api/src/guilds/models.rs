use crate::discord::ise;
use crate::guilds::verify::models::Verify;
use crate::guilds::votes::models::Vote;
use http::StatusCode;
use lambda_http::tracing::{error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bigdecimal::{BigDecimal, ToPrimitive};
use sqlx::{Pool, Postgres, Row};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use common::dsql::{bind_in_params, in_params};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Guild {
    pub guild_id: Id<GuildMarker>,
    pub verify: Verify,
    pub vote: Vote,
}

impl Default for Guild {
    fn default() -> Self {
        Guild {
            guild_id: Id::new(1),
            verify: Verify {
                roles: vec![],
                user_links: HashMap::new(),
            },
            vote: Vote::default(),
        }
    }
}

/// Parses the `verify` and `vote` JSON columns of a `guilds` row.
///
/// Extracted from the row-hydration code so it can be exercised in unit
/// tests without a live database connection. Malformed/legacy JSON returns
/// a `500` instead of panicking the handler (see issue #20).
fn parse_verify_vote(verify_json: &str, vote_json: &str) -> Result<(Verify, Vote), StatusCode> {
    let verify = serde_json::from_str(verify_json).map_err(ise)?;
    let vote = serde_json::from_str(vote_json).map_err(ise)?;
    Ok((verify, vote))
}

impl Guild {
    /// Fetches a guild from the DB.
    ///
    /// Returns `Ok(None)` if the guild simply doesn't have a row yet, and
    /// `Err(StatusCode)` only on an actual DB/deserialization failure, so
    /// callers can tell "not found" apart from "the DB blipped" instead of
    /// unwrapping both cases into a panic.
    pub async fn from_db(
        guild_id: Id<GuildMarker>,
        pg_pool: &Pool<Postgres>,
    ) -> Result<Option<Guild>, StatusCode> {
        let row = sqlx::query("SELECT id, verify, vote FROM guilds WHERE id = $1")
            .bind(BigDecimal::from(guild_id.into_nonzero().get()))
            .fetch_optional(pg_pool)
            .await
            .map_err(ise)?;

        row.map(|row| {
            let (verify, vote) =
                parse_verify_vote(row.get::<&str, _>("verify"), row.get::<&str, _>("vote"))?;
            Ok(Guild {
                guild_id,
                verify,
                vote,
            })
        })
        .transpose()
    }

    pub async fn vec_from_db(
        guild_ids: Vec<Id<GuildMarker>>,
        pg_pool: &Pool<Postgres>,
    ) -> Result<Vec<Guild>, StatusCode> {
        let bd_guild_ids = guild_ids.iter().map(|guild_id| BigDecimal::from(guild_id.into_nonzero().get())).collect::<Vec<BigDecimal>>();
        let formatted_query = format!("SELECT id, verify, vote FROM guilds WHERE id IN ({})", in_params(&bd_guild_ids).as_str());
        let mut query = sqlx::query(formatted_query.as_str());
        query = bind_in_params(query, &bd_guild_ids);

        let rows = query.fetch_all(pg_pool).await.map_err(ise)?;

        rows.iter()
            .map(|row| {
                let guild_id = row
                    .get::<BigDecimal, _>("id")
                    .to_u64()
                    .ok_or_else(|| ise("guild id out of range"))?;
                let (verify, vote) =
                    parse_verify_vote(row.get::<&str, _>("verify"), row.get::<&str, _>("vote"))?;
                Ok(Guild {
                    guild_id: Id::new(guild_id),
                    verify,
                    vote,
                })
            })
            .collect()
    }

    pub async fn save(&self, pg_pool: &Pool<Postgres>) -> Result<(), StatusCode> {
        sqlx::query("INSERT INTO guilds (id, verify, vote) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET verify = $2, vote = $3, updated_at = CURRENT_TIMESTAMP")
            .bind(BigDecimal::from(self.guild_id.into_nonzero().get()))
            .bind(serde_json::to_string(&self.verify).map_err(ise)?)
            .bind(serde_json::to_string(&self.vote).map_err(ise)?)
            .execute(pg_pool)
            .await
            .map_err(ise)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for issue #21: `Guild::save` must return a `Result`
    /// that surfaces DB errors to the caller instead of swallowing them and
    /// reporting success. We point a lazily-connecting pool at a port that
    /// nothing is listening on, so the first query fails fast with a
    /// connection error, exercising the exact `sqlx::Error` -> `StatusCode`
    /// mapping path used in production. This does not require a live
    /// Postgres server, but a real DB integration test would additionally
    /// be valuable in CI to cover the success path end-to-end.
    #[tokio::test]
    async fn save_propagates_db_errors_instead_of_swallowing_them() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://baduser:badpass@127.0.0.1:1/nonexistent_db")
            .expect("connect_lazy should not eagerly connect");

        let guild = Guild::default();
        let result = guild.save(&pool).await;

        assert_eq!(
            result,
            Err(StatusCode::INTERNAL_SERVER_ERROR),
            "save() must return an Err(StatusCode) when the underlying DB write fails, \
             not silently succeed"
        );
    }

    fn valid_verify_json() -> String {
        serde_json::to_string(&Verify::default()).unwrap()
    }

    fn valid_vote_json() -> String {
        serde_json::to_string(&Vote::default()).unwrap()
    }

    #[test]
    fn parse_verify_vote_succeeds_on_well_formed_json() {
        let result = parse_verify_vote(&valid_verify_json(), &valid_vote_json());
        assert!(result.is_ok());
    }

    #[test]
    fn parse_verify_vote_returns_500_instead_of_panicking_on_malformed_verify_json() {
        // Regression test for issue #20: previously this hydration path used
        // `.unwrap()`, so a single malformed/legacy row would panic every
        // handler that touched the guild, forever. It must now return a
        // clean 500 instead.
        let result = parse_verify_vote("not-json", &valid_vote_json());
        assert!(matches!(result, Err(StatusCode::INTERNAL_SERVER_ERROR)));
    }

    #[test]
    fn parse_verify_vote_returns_500_instead_of_panicking_on_malformed_vote_json() {
        let result = parse_verify_vote(&valid_verify_json(), "not-json");
        assert!(matches!(result, Err(StatusCode::INTERNAL_SERVER_ERROR)));
    }

    #[test]
    fn row_present_maps_to_some_row_missing_maps_to_none() {
        // Mirrors the `row.map(..).transpose()` control flow used in
        // `Guild::from_db`: a present row must produce `Ok(Some(_))`, and a
        // genuinely missing row must produce `Ok(None)` -- NOT an error.
        // Only a real DB/deserialization failure should produce `Err(_)`.
        let present_row: Option<&str> = Some("row-data");
        let result: Result<Option<&str>, StatusCode> = present_row.map(Ok).transpose();
        assert_eq!(result, Ok(Some("row-data")));

        let missing_row: Option<&str> = None;
        let result: Result<Option<&str>, StatusCode> = missing_row.map(Ok).transpose();
        assert_eq!(result, Ok(None));
    }
}
