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

impl User {
    pub async fn from_db(user_id: Id<UserMarker>, pg_pool: &Pool<Postgres>) -> Option<User> {
        match sqlx::query("SELECT id, links, link_guilds FROM users WHERE id = $1")
        .bind(BigDecimal::from(user_id.into_nonzero().get()))
        .fetch_optional(pg_pool)
        .await {
            Ok(Some(row)) => {
                Some(User{
                    user_id,
                    links: serde_json::from_str(row.get::<&str, _>("links")).unwrap(),
                    link_guilds: serde_json::from_str(row.get::<&str, _>("link_guilds")).unwrap(),
                })
            },
            Ok(None) => Some(User{
                user_id,
                ..Default::default()
            }),
            Err(e) => {
                error!("Error fetching user from DB: {}", e);
                None
            }
        }
    }

    pub async fn save(&self, pg_pool: &Pool<Postgres>) -> Result<(), StatusCode> {
        sqlx::query("INSERT INTO users (id, links, link_guilds) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET links = $2, link_guilds = $3, updated_at = CURRENT_TIMESTAMP")
        .bind(BigDecimal::from(self.user_id.into_nonzero().get()))
        .bind(serde_json::to_string(&self.links).map_err(ise)?)
        .bind(serde_json::to_string(&self.link_guilds).map_err(ise)?)
        .execute(pg_pool)
        .await
        .map_err(ise)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for issue #21: `User::save` must return a `Result`
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

        let user = User::default();
        let result = user.save(&pool).await;

        assert_eq!(
            result,
            Err(StatusCode::INTERNAL_SERVER_ERROR),
            "save() must return an Err(StatusCode) when the underlying DB write fails, \
             not silently succeed"
        );
    }
}
