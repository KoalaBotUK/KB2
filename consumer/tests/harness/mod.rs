//! Shared harness for the consumer's integration and performance tests.
//!
//! Requires a reachable Postgres set via `KB2_TEST_DATABASE_URL`
//! (see `scripts/verify-recon-tests.sh`, which provisions a disposable
//! cluster and runs the suites). Tests are `#[ignore]`d so a plain
//! `cargo test` stays green without a database.
//!
//! (dead_code allowed: each tests/*.rs binary compiles this module
//! separately and none of them uses every helper.)
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use consumer::verify::discord::{DiscordRoles, RoleCallError};
use common::verify::{GuildUserLink, Link, Verify, VerifyRole};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};

pub const DB_URL_ENV: &str = "KB2_TEST_DATABASE_URL";

/// The subset of production migrations these tests need. The audit
/// migrations are skipped: `CREATE INDEX ASYNC` is Aurora DSQL syntax that
/// vanilla Postgres rejects, and the worker never touches audit tables.
const MIGRATIONS: &[&str] = &[
    include_str!("../../../api/migrations/20260119010332_init.up.sql"),
    include_str!("../../../api/migrations/20260119050545_init.up.sql"),
    include_str!("../../../api/migrations/20260712120000_guild_user_links.up.sql"),
    include_str!("../../../api/migrations/20260712120001_verify_jobs.up.sql"),
];

pub async fn test_pool() -> Pool<Postgres> {
    let url = std::env::var(DB_URL_ENV).unwrap_or_else(|_| {
        panic!("{DB_URL_ENV} must point at a Postgres for integration/perf tests")
    });
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .expect("connect to test database");

    // Tests within one binary run in parallel and both test binaries may run
    // concurrently — serialize DDL behind an advisory lock (session-scoped,
    // so lock and unlock must use the same connection).
    let mut conn = pool.acquire().await.expect("acquire setup connection");
    sqlx::query("SELECT pg_advisory_lock(421337)")
        .execute(&mut *conn)
        .await
        .expect("advisory lock");
    for sql in MIGRATIONS {
        sqlx::raw_sql(sql)
            .execute(&mut *conn)
            .await
            .expect("apply migration");
    }
    sqlx::query("SELECT pg_advisory_unlock(421337)")
        .execute(&mut *conn)
        .await
        .expect("advisory unlock");

    pool
}

/// Deletes every row belonging to a test guild so tests are re-runnable
/// against a persistent cluster (guild ids are fixed per test; a previous
/// run's job row / links / lease would otherwise leak into this one).
pub async fn reset_guild(pool: &Pool<Postgres>, guild_id: u64) {
    let id = sqlx::types::BigDecimal::from(guild_id);
    for sql in [
        "DELETE FROM verify_jobs WHERE guild_id = $1",
        "DELETE FROM guild_user_links WHERE guild_id = $1",
        "DELETE FROM guilds WHERE id = $1",
    ] {
        sqlx::query(sql)
            .bind(id.clone())
            .execute(pool)
            .await
            .expect("reset guild");
    }
}

pub fn gid(id: u64) -> Id<GuildMarker> {
    Id::new(id)
}

pub fn rid(id: u64) -> Id<RoleMarker> {
    Id::new(id)
}

pub fn active_link(address: &str) -> Link {
    Link {
        link_address: address.to_string(),
        linked_at: 0,
        active: true,
    }
}

pub fn verify_role(role_id: u64, pattern: &str) -> VerifyRole {
    VerifyRole {
        role_id: rid(role_id),
        pattern: pattern.to_string(),
        members: 0,
    }
}

/// Writes a guild row whose `verify` blob carries the given config (and,
/// for backfill tests, optionally legacy in-blob user_links — which
/// `Verify` itself can no longer serialize, by design, so legacy JSON is
/// assembled manually here).
pub async fn seed_guild(
    pool: &Pool<Postgres>,
    guild_id: u64,
    roles: Vec<VerifyRole>,
    legacy_user_links: Option<HashMap<Id<UserMarker>, Vec<Link>>>,
) {
    let verify_json = match legacy_user_links {
        None => serde_json::to_string(&Verify {
            roles,
            user_links: HashMap::new(),
        })
        .unwrap(),
        Some(user_links) => {
            // Legacy on-disk shape: user_links inside the blob.
            let mut value = serde_json::to_value(&Verify {
                roles,
                user_links: HashMap::new(),
            })
            .unwrap();
            value["user_links"] = serde_json::to_value(&user_links).unwrap();
            value.to_string()
        }
    };
    sqlx::query(
        "INSERT INTO guilds (id, verify, vote) VALUES ($1, $2, '{\"votes\":[]}')
         ON CONFLICT (id) DO UPDATE SET verify = $2, updated_at = CURRENT_TIMESTAMP",
    )
    .bind(sqlx::types::BigDecimal::from(guild_id))
    .bind(verify_json)
    .execute(pool)
    .await
    .expect("seed guild");
}

/// Seeds `n` guild_user_links rows with user ids `1..=n`; `links_for`
/// decides each user's addresses (empty vec ⇒ a row with no active links).
pub async fn seed_links(
    pool: &Pool<Postgres>,
    guild_id: u64,
    n: u64,
    links_for: impl Fn(u64) -> Vec<Link>,
) {
    let map: HashMap<Id<UserMarker>, Vec<Link>> = (1..=n)
        .map(|user_id| (Id::new(user_id), links_for(user_id)))
        .collect();
    GuildUserLink::seed_from_blob(gid(guild_id), &map, pool)
        .await
        .expect("seed links");
}

/// In-process gateway stub: counts calls, optionally injects latency per
/// call and a single scripted 429.
#[derive(Default)]
pub struct StubGateway {
    pub adds: AtomicUsize,
    pub removes: AtomicUsize,
    pub latency: Option<Duration>,
    rate_limit_once: Mutex<Option<Duration>>,
}

impl StubGateway {
    pub fn with_latency(latency: Duration) -> Self {
        StubGateway {
            latency: Some(latency),
            ..Default::default()
        }
    }

    /// The next role call (only) fails with a 429 carrying `retry_after`.
    pub fn rate_limit_next_call(&self, retry_after: Duration) {
        *self.rate_limit_once.lock().unwrap() = Some(retry_after);
    }

    pub fn adds(&self) -> usize {
        self.adds.load(Ordering::SeqCst)
    }

    pub fn removes(&self) -> usize {
        self.removes.load(Ordering::SeqCst)
    }

    async fn call(&self, counter: &AtomicUsize) -> Result<(), RoleCallError> {
        if let Some(retry_after) = self.rate_limit_once.lock().unwrap().take() {
            return Err(RoleCallError::RateLimited { retry_after });
        }
        if let Some(latency) = self.latency {
            tokio::time::sleep(latency).await;
        }
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

impl DiscordRoles for StubGateway {
    async fn add_role(
        &self,
        _guild_id: Id<GuildMarker>,
        _user_id: Id<UserMarker>,
        _role_id: Id<RoleMarker>,
    ) -> Result<(), RoleCallError> {
        self.call(&self.adds).await
    }

    async fn remove_role(
        &self,
        _guild_id: Id<GuildMarker>,
        _user_id: Id<UserMarker>,
        _role_id: Id<RoleMarker>,
    ) -> Result<(), RoleCallError> {
        self.call(&self.removes).await
    }
}
