use crate::discord::ise;
use http::StatusCode;
use lambda_http::tracing::error;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use sqlx::types::BigDecimal;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub link_address: String,
    pub linked_at: u64,
    pub active: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LinkGuild {
    pub guild_id: Id<GuildMarker>,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: Id<UserMarker>,
    pub links: Vec<Link>,
    pub link_guilds: Vec<LinkGuild>,
}

impl Default for User {
    fn default() -> Self {
        User {
            user_id: Id::new(1),
            links: vec![],
            link_guilds: vec![],
        }
    }
}

/// Parses the `links` and `link_guilds` JSON columns of a `users` row.
///
/// Extracted from the row-hydration code so it can be exercised in unit
/// tests without a live database connection. Malformed/legacy JSON returns
/// a `500` instead of panicking the handler (see issue #20).
fn parse_links(
    links_json: &str,
    link_guilds_json: &str,
) -> Result<(Vec<Link>, Vec<LinkGuild>), StatusCode> {
    let links = serde_json::from_str(links_json).map_err(ise)?;
    let link_guilds = serde_json::from_str(link_guilds_json).map_err(ise)?;
    Ok((links, link_guilds))
}

impl User {
    /// Fetches a user from the DB.
    ///
    /// Returns `Ok(None)` if the user simply doesn't have a row yet, and
    /// `Err(StatusCode)` only on an actual DB/deserialization failure, so
    /// callers can tell "not found" apart from "the DB blipped" instead of
    /// unwrapping both cases into a panic.
    pub async fn from_db(
        user_id: Id<UserMarker>,
        pg_pool: &Pool<Postgres>,
    ) -> Result<Option<User>, StatusCode> {
        let row = sqlx::query("SELECT id, links, link_guilds FROM users WHERE id = $1")
            .bind(BigDecimal::from(user_id.into_nonzero().get()))
            .fetch_optional(pg_pool)
            .await
            .map_err(ise)?;

        row.map(|row| {
            let (links, link_guilds) =
                parse_links(row.get::<&str, _>("links"), row.get::<&str, _>("link_guilds"))?;
            Ok(User {
                user_id,
                links,
                link_guilds,
            })
        })
        .transpose()
    }

    pub async fn save(&self, pg_pool: &Pool<Postgres>) {
        match sqlx::query("INSERT INTO users (id, links, link_guilds) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET links = $2, link_guilds = $3, updated_at = CURRENT_TIMESTAMP")
        .bind(BigDecimal::from(self.user_id.into_nonzero().get()))
        .bind(serde_json::to_string(&self.links).unwrap())
        .bind(serde_json::to_string(&self.link_guilds).unwrap())
        .execute(pg_pool)
        .await {
            Ok(_) => (),
            Err(e) => {
                error!("Error saving user to DB: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_links_json() -> String {
        serde_json::to_string(&Vec::<Link>::new()).unwrap()
    }

    fn valid_link_guilds_json() -> String {
        serde_json::to_string(&Vec::<LinkGuild>::new()).unwrap()
    }

    #[test]
    fn parse_links_succeeds_on_well_formed_json() {
        let result = parse_links(&valid_links_json(), &valid_link_guilds_json());
        assert!(result.is_ok());
    }

    #[test]
    fn parse_links_returns_500_instead_of_panicking_on_malformed_links_json() {
        // Regression test for issue #20: previously this hydration path used
        // `.unwrap()`, so a single malformed/legacy row would panic every
        // handler that touched the user, forever. It must now return a
        // clean 500 instead.
        let result = parse_links("not-json", &valid_link_guilds_json());
        assert!(matches!(result, Err(StatusCode::INTERNAL_SERVER_ERROR)));
    }

    #[test]
    fn parse_links_returns_500_instead_of_panicking_on_malformed_link_guilds_json() {
        let result = parse_links(&valid_links_json(), "not-json");
        assert!(matches!(result, Err(StatusCode::INTERNAL_SERVER_ERROR)));
    }

    #[test]
    fn row_present_maps_to_some_row_missing_maps_to_none() {
        // Mirrors the `row.map(..).transpose()` control flow used in
        // `User::from_db`: a present row must produce `Ok(Some(_))`, and a
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
