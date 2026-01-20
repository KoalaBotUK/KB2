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
