use crate::guilds::verify::models::Verify;
use crate::guilds::votes::models::Vote;
use lambda_http::tracing::{error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bigdecimal::{BigDecimal, ToPrimitive};
use sqlx::{Pool, Postgres, Row};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use crate::dsql::{bind_in_params, in_params};

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

impl Guild {
    pub async fn from_db(
        guild_id: Id<GuildMarker>,
        pg_pool: &Pool<Postgres>,
    ) -> Option<Guild> {
        match sqlx::query("SELECT id, verify, vote FROM guilds WHERE id = $1")
            .bind(BigDecimal::from(guild_id.into_nonzero().get()))
            .fetch_optional(pg_pool)
            .await {
            Ok(Some(row)) => {
                Some(Guild{
                    guild_id,
                    verify: serde_json::from_str(row.get::<&str, _>("verify")).unwrap(),
                    vote: serde_json::from_str(row.get::<&str, _>("vote")).unwrap(),
                })
            },
            Ok(None) => Some(Guild{
                guild_id,
                ..Default::default()
            }),
            Err(e) => {
                error!("Error fetching user from DB: {}", e);
                None
            }
        }
    }

    pub async fn vec_from_db(
        guild_ids: Vec<Id<GuildMarker>>,
        pg_pool: &Pool<Postgres>,
    ) -> Vec<Guild> {
        let bd_guild_ids = guild_ids.iter().map(|guild_id| BigDecimal::from(guild_id.into_nonzero().get())).collect::<Vec<BigDecimal>>();
        let formatted_query = format!("SELECT id, verify, vote FROM guilds WHERE id IN ({})", in_params(&bd_guild_ids).as_str());
        let mut query = sqlx::query(formatted_query.as_str());
        query = bind_in_params(query, &bd_guild_ids);

        match query
            .fetch_all(pg_pool)
            .await {
            Ok(rows) => rows.iter().map(|row|
                Guild{
                    guild_id: Id::new(row.get::<BigDecimal, _>("id").to_u64().unwrap()),
                    verify: serde_json::from_str(row.get::<&str, _>("verify")).unwrap(),
                    vote: serde_json::from_str(row.get::<&str, _>("vote")).unwrap(),
                }
            ).collect(),
            Err(e) => {
                error!("Error fetching user from DB: {}", e);
                vec![]
            }
        }
    }

    pub async fn save(&self, pg_pool: &Pool<Postgres>) {
        match sqlx::query("INSERT INTO guilds (id, verify, vote) VALUES ($1, $2, $3) ON CONFLICT (id) DO UPDATE SET verify = $2, vote = $3, updated_at = CURRENT_TIMESTAMP")
            .bind(BigDecimal::from(self.guild_id.into_nonzero().get()))
            .bind(serde_json::to_string(&self.verify).unwrap())
            .bind(serde_json::to_string(&self.vote).unwrap())
            .execute(pg_pool)
            .await {
            Ok(_) => (),
            Err(e) => {
                error!("Error saving user to DB: {}", e);
            }
        }
    }
}
