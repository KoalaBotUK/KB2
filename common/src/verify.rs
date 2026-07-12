//! Shared verify-domain types and persistence used by both the `api` crate
//! (HTTP handlers) and the `consumer` crate (the async reconciliation
//! worker). See `docs/low-level-architecture/verify-role-reconciliation.md`.
//!
//! Lives in `common` because `consumer` (a binary crate) cannot depend on
//! `api` (also a binary crate); `api` re-exports these types from their old
//! paths so its handler diffs stay mechanical.

use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use sqlx::{Pool, Postgres, QueryBuilder, Row};
use tracing::error;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};

// ---------------------------------------------------------------------------
// Links & matching (moved from api::users)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub link_address: String,
    pub linked_at: u64,
    pub active: bool,
}

/// Never panics: a pattern that fails to compile (e.g. corrupted/legacy data
/// that slipped past validation at write time) is treated as "matches
/// nothing" rather than bringing down the whole guild's verify flow.
pub fn link_arr_match(links: &[Link], pattern: &str) -> bool {
    match regex::Regex::new(pattern) {
        Ok(regex) => links
            .iter()
            .any(|l| l.active && regex.is_match(&l.link_address)),
        Err(e) => {
            error!("Invalid verify pattern {pattern:?}: {e}");
            false
        }
    }
}

pub fn link_match(link: &Link, pattern: &str) -> bool {
    match regex::Regex::new(pattern) {
        Ok(regex) => link.active && regex.is_match(&link.link_address),
        Err(e) => {
            error!("Invalid verify pattern {pattern:?}: {e}");
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Guild verify config (moved from api::guilds::verify::models)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyRole {
    pub role_id: Id<RoleMarker>,
    pub pattern: String,
    pub members: u32,
}

impl Default for VerifyRole {
    fn default() -> Self {
        VerifyRole {
            role_id: Id::new(1),
            pattern: "".to_string(),
            members: 0,
        }
    }
}

impl std::hash::Hash for VerifyRole {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.role_id.hash(state);
        self.pattern.hash(state);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Verify {
    #[serde(default)]
    pub roles: Vec<VerifyRole>,
    /// Legacy field: per-guild user links used to live inside the guild JSON
    /// blob. It is still *deserialized* so pre-migration rows can be seeded
    /// into the `guild_user_links` table (see
    /// [`GuildUserLink::seed_from_blob`]), but it is never serialized back —
    /// the table is the sole post-migration source of truth.
    #[serde(default, skip_serializing)]
    pub user_links: HashMap<Id<UserMarker>, Vec<Link>>,
}

impl Verify {
    /// Adjusts a role's `members` count by `delta`, saturating at zero.
    ///
    /// The per-user link handlers apply ±1 deltas at the moment they issue a
    /// role change (they know exactly whether the user newly qualifies or
    /// newly disqualifies), and a completed reconciliation job overwrites the
    /// counts wholesale via [`fold_counts_into_guild`] — so unlike the old
    /// `recompute_role_members`, no code path ever needs every link in memory.
    pub fn bump_members(&mut self, role_id: Id<RoleMarker>, delta: i32) {
        if let Some(role) = self.roles.iter_mut().find(|r| r.role_id == role_id) {
            role.members = role.members.saturating_add_signed(delta);
        }
    }
}

fn bd(id: u64) -> BigDecimal {
    BigDecimal::from(id)
}

/// Reads just the `verify` column of a guild row. This is what the worker
/// uses: it must not deserialize (or even fetch) the unrelated `vote` blob.
///
/// Returns `Ok(None)` when the guild has no row yet.
pub async fn guild_verify_from_db(
    guild_id: Id<GuildMarker>,
    pg_pool: &Pool<Postgres>,
) -> anyhow::Result<Option<Verify>> {
    let row = sqlx::query("SELECT verify FROM guilds WHERE id = $1")
        .bind(bd(guild_id.get()))
        .fetch_optional(pg_pool)
        .await?;
    row.map(|row| Ok(serde_json::from_str(row.get::<&str, _>("verify"))?))
        .transpose()
}

/// Writes back a guild's verify config blob (used by
/// [`fold_counts_into_guild`]). Deliberately does NOT insert: a completed
/// job for a guild whose row vanished has nothing to fold into.
async fn guild_verify_save(
    guild_id: Id<GuildMarker>,
    verify: &Verify,
    pg_pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE guilds SET verify = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(bd(guild_id.get()))
        .bind(serde_json::to_string(verify)?)
        .execute(pg_pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Recon scope & SQS message
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RoleRemoval {
    pub role_id: Id<RoleMarker>,
    /// Pattern captured at removal time — the role is gone from config, so
    /// the worker needs it to know which users to strip (parity with the
    /// legacy synchronous `remove_existing_role` behaviour).
    pub pattern: String,
}

/// A merge-able *set* of pending reconciliation operations.
///
/// This is deliberately not a single enum value: a `RoleRemoval` must survive
/// being merged with `all` — the removed role no longer exists in the guild
/// config, so a plain "widen to All" would forget to strip it.
///
/// Three per-role operation kinds, in increasing strength:
///
/// * **add** — brand-new role: issue role adds for users whose links match.
///   No remove calls (nobody can hold a role that just got created), so a
///   50k-member guild pays Discord calls only for actual matchers.
/// * **sync** — changed role (pattern replaced): per user, add when links
///   match the *current* pattern, remove when they don't. Correct for any
///   chain of pattern replacements without tracking old patterns.
/// * **removal** — role deleted from config: strip users matching the
///   pattern captured at deletion time.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct ReconScope {
    /// Full reconcile: every config role, add-or-remove per user (recon).
    #[serde(default)]
    pub all: bool,
    /// Newly added roles: add-only, for users whose links match.
    #[serde(default)]
    pub add_role_ids: Vec<Id<RoleMarker>>,
    /// Changed roles: full add-or-remove per user by current pattern.
    #[serde(default)]
    pub sync_role_ids: Vec<Id<RoleMarker>>,
    /// Roles deleted from config: remove from users whose links match(ed).
    #[serde(default)]
    pub removals: Vec<RoleRemoval>,
}

impl ReconScope {
    pub fn all() -> Self {
        ReconScope {
            all: true,
            ..Default::default()
        }
    }

    pub fn role_add(role_id: Id<RoleMarker>) -> Self {
        ReconScope {
            add_role_ids: vec![role_id],
            ..Default::default()
        }
    }

    pub fn role_sync(role_id: Id<RoleMarker>) -> Self {
        ReconScope {
            sync_role_ids: vec![role_id],
            ..Default::default()
        }
    }

    pub fn role_remove(role_id: Id<RoleMarker>, pattern: String) -> Self {
        ReconScope {
            removals: vec![RoleRemoval { role_id, pattern }],
            ..Default::default()
        }
    }

    /// Folds `other` (the newest change) into `self` (the in-flight scope).
    /// Union semantics; per role, the latest operation kind wins — except
    /// that an add landing on a pending removal escalates to sync (the role
    /// was deleted and re-created, so stale holders of the old pattern must
    /// be stripped, which add-only would never do). `all` is sticky.
    pub fn merge(&mut self, other: ReconScope) {
        self.all |= other.all;
        for r in other.removals {
            self.add_role_ids.retain(|id| *id != r.role_id);
            self.sync_role_ids.retain(|id| *id != r.role_id);
            self.removals.retain(|x| x.role_id != r.role_id);
            self.removals.push(r);
        }
        for id in other.sync_role_ids {
            self.add_role_ids.retain(|x| *x != id);
            self.removals.retain(|x| x.role_id != id);
            if !self.sync_role_ids.contains(&id) {
                self.sync_role_ids.push(id);
            }
        }
        for id in other.add_role_ids {
            if self.removals.iter().any(|x| x.role_id == id) {
                // Deleted then re-added: escalate to sync so users holding
                // the role from the deleted incarnation get reconciled.
                self.removals.retain(|x| x.role_id != id);
                if !self.sync_role_ids.contains(&id) {
                    self.sync_role_ids.push(id);
                }
            } else if !self.sync_role_ids.contains(&id) && !self.add_role_ids.contains(&id) {
                self.add_role_ids.push(id);
            }
        }
    }

    /// Whether `role_id` (a role currently in config) is covered by this scope.
    pub fn covers_config_role(&self, role_id: Id<RoleMarker>) -> bool {
        self.all || self.add_role_ids.contains(&role_id) || self.sync_role_ids.contains(&role_id)
    }

    /// Whether a covered config role gets the full add-or-remove treatment
    /// (as opposed to add-only for brand-new roles).
    pub fn full_sync_for(&self, role_id: Id<RoleMarker>) -> bool {
        self.all || self.sync_role_ids.contains(&role_id)
    }
}

/// SQS body for message attribute `kind = "verify_recon"`.
///
/// Deliberately minimal: the `verify_jobs` row is the source of truth; the
/// message is only a wake-up token. Stale tokens (generation < row
/// generation) are dropped by the lease check.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReconMessage {
    pub guild_id: Id<GuildMarker>,
    pub generation: i64,
}

pub const RECON_MESSAGE_KIND: &str = "verify_recon";

/// Sends a recon wake-up token, with the same 3-attempt exponential backoff
/// as the audit publisher — but unlike audit, failures are *propagated*: a
/// saved role with no job token would strand desired state, so the caller
/// (an API handler) must surface the failure and let the admin retry.
pub async fn enqueue_recon(
    guild_id: Id<GuildMarker>,
    generation: i64,
    delay: Duration,
    sqs: &aws_sdk_sqs::Client,
    queue_url: &str,
) -> anyhow::Result<()> {
    const MAX_ATTEMPTS: u32 = 3;
    /// SQS's per-message DelaySeconds ceiling. Longer waits simply mature at
    /// 15 min and the worker re-checks; the lease/generation checks make an
    /// early wake harmless.
    const MAX_SQS_DELAY_SECS: u64 = 900;

    let body = serde_json::to_string(&ReconMessage {
        guild_id,
        generation,
    })?;
    let kind_attribute = aws_sdk_sqs::types::MessageAttributeValue::builder()
        .data_type("String")
        .string_value(RECON_MESSAGE_KIND)
        .build()?;
    let delay_seconds = delay.as_secs().min(MAX_SQS_DELAY_SECS) as i32;

    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=MAX_ATTEMPTS {
        match sqs
            .send_message()
            .queue_url(queue_url)
            .message_attributes("kind", kind_attribute.clone())
            .delay_seconds(delay_seconds)
            .message_body(&body)
            .send()
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                error!(
                    "Failed to send verify_recon to SQS (attempt {}/{}): {}",
                    attempt, MAX_ATTEMPTS, e
                );
                last_err = Some(e.into());
                if attempt < MAX_ATTEMPTS {
                    tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
                }
            }
        }
    }
    Err(last_err.expect("loop ran at least once"))
}

// ---------------------------------------------------------------------------
// verify_jobs — lease / supersede / checkpoint
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Succeeded => "succeeded",
            JobStatus::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<JobStatus> {
        match s {
            "pending" => Some(JobStatus::Pending),
            "running" => Some(JobStatus::Running),
            "succeeded" => Some(JobStatus::Succeeded),
            "failed" => Some(JobStatus::Failed),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, JobStatus::Succeeded | JobStatus::Failed)
    }
}

/// One row per guild: lease + cursor + progress. `guild_id` being the primary
/// key enforces the single-active-job invariant by construction.
#[derive(Clone, Debug, Serialize)]
pub struct VerifyJob {
    pub guild_id: Id<GuildMarker>,
    pub generation: i64,
    pub status: JobStatus,
    pub scope: ReconScope,
    pub cursor: Option<Id<UserMarker>>,
    pub total: i32,
    pub processed: i32,
    pub errors: i32,
    /// Per-role matched-member counts accumulated during the scan; folded
    /// into the guild's `VerifyRole::members` at completion.
    pub counts: HashMap<Id<RoleMarker>, u32>,
}

fn job_from_row(row: &sqlx::postgres::PgRow) -> anyhow::Result<VerifyJob> {
    let guild_id = row
        .get::<BigDecimal, _>("guild_id")
        .to_u64()
        .ok_or_else(|| anyhow::anyhow!("guild id out of range"))?;
    let status_str: String = row.get("status");
    let status = JobStatus::parse(&status_str)
        .ok_or_else(|| anyhow::anyhow!("unknown job status {status_str:?}"))?;
    let cursor = row
        .get::<Option<BigDecimal>, _>("cursor_user")
        .map(|c| {
            c.to_u64()
                .filter(|v| *v != 0)
                .ok_or_else(|| anyhow::anyhow!("cursor out of range"))
        })
        .transpose()?
        .map(Id::new);
    Ok(VerifyJob {
        guild_id: Id::new(guild_id),
        generation: row.get("generation"),
        status,
        scope: serde_json::from_str(row.get::<&str, _>("scope"))?,
        cursor,
        total: row.get("total"),
        processed: row.get("processed"),
        errors: row.get("errors"),
        counts: serde_json::from_str(row.get::<&str, _>("counts"))?,
    })
}

const JOB_COLUMNS: &str =
    "guild_id, generation, status, scope, cursor_user, total, processed, errors, counts";

impl VerifyJob {
    pub async fn from_db(
        guild_id: Id<GuildMarker>,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<Option<VerifyJob>> {
        let row = sqlx::query(&format!(
            "SELECT {JOB_COLUMNS} FROM verify_jobs WHERE guild_id = $1"
        ))
        .bind(bd(guild_id.get()))
        .fetch_optional(pg_pool)
        .await?;
        row.as_ref().map(job_from_row).transpose()
    }

    /// Bumps the guild's job to a new generation with `scope` folded in, and
    /// returns the new generation to embed in the SQS token.
    ///
    /// The `ON CONFLICT` increment is atomic, so concurrent bumps both land:
    /// the token carrying the highest generation wins the lease check, and a
    /// lost read-merge race between two simultaneous admin edits degrades to
    /// the *loser's* scope only — which is why each caller re-reads and
    /// re-merges immediately before its own upsert, bounding the window to
    /// the gap between two statements.
    pub async fn supersede(
        guild_id: Id<GuildMarker>,
        scope: ReconScope,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<i64> {
        let mut merged = scope;
        if let Some(existing) = Self::from_db(guild_id, pg_pool).await?
            && !existing.status.is_terminal()
        {
            let mut base = existing.scope;
            base.merge(merged);
            merged = base;
        }

        let row = sqlx::query(
            "INSERT INTO verify_jobs (guild_id, generation, status, scope, total)
             VALUES ($1, 1, 'pending', $2,
                     (SELECT count(*) FROM guild_user_links WHERE guild_id = $1))
             ON CONFLICT (guild_id) DO UPDATE SET
                 generation  = verify_jobs.generation + 1,
                 status      = 'pending',
                 scope       = EXCLUDED.scope,
                 cursor_user = NULL,
                 processed   = 0,
                 errors      = 0,
                 counts      = '{}',
                 total       = EXCLUDED.total,
                 updated_at  = CURRENT_TIMESTAMP
             RETURNING generation",
        )
        .bind(bd(guild_id.get()))
        .bind(serde_json::to_string(&merged)?)
        .fetch_one(pg_pool)
        .await?;

        Ok(row.get::<i64, _>("generation"))
    }

    /// Conditional-UPDATE mutex: returns the job iff this invocation now
    /// exclusively owns it. Zero rows updated means the token is stale, the
    /// job is terminal, or another invocation holds a live lease — all of
    /// which the caller treats as "drop the message successfully".
    pub async fn acquire_lease(
        guild_id: Id<GuildMarker>,
        generation: i64,
        lease: Duration,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<Option<VerifyJob>> {
        let res = sqlx::query(
            "UPDATE verify_jobs SET
                 status      = 'running',
                 lease_until = CURRENT_TIMESTAMP + ($3 * interval '1 second'),
                 updated_at  = CURRENT_TIMESTAMP
             WHERE guild_id = $1
               AND generation = $2
               AND status IN ('pending', 'running')
               AND (lease_until IS NULL OR lease_until < CURRENT_TIMESTAMP)",
        )
        .bind(bd(guild_id.get()))
        .bind(generation)
        .bind(lease.as_secs_f64())
        .execute(pg_pool)
        .await?;

        if res.rows_affected() == 0 {
            return Ok(None);
        }
        Self::from_db(guild_id, pg_pool).await
    }

    /// Refreshes `total` after the seed-on-first-job backfill may have grown
    /// the links table (the supersede computed it before seeding).
    pub async fn refresh_total(&mut self, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
        let row = sqlx::query(
            "UPDATE verify_jobs SET
                 total = (SELECT count(*) FROM guild_user_links WHERE guild_id = $1),
                 updated_at = CURRENT_TIMESTAMP
             WHERE guild_id = $1 AND generation = $2
             RETURNING total",
        )
        .bind(bd(self.guild_id.get()))
        .bind(self.generation)
        .fetch_optional(pg_pool)
        .await?;
        if let Some(row) = row {
            self.total = row.get("total");
        }
        Ok(())
    }

    /// Persists progress, guarded by `generation` so a supersede mid-flight
    /// is detected here. Returns `false` when superseded (caller must stop —
    /// the new generation's token owns the work now).
    ///
    /// `release_lease` is set on the invocation's *final* checkpoint so the
    /// continuation token (delivered seconds later) can acquire the lease.
    pub async fn checkpoint(
        &self,
        cursor: Option<Id<UserMarker>>,
        processed_delta: i32,
        errors_delta: i32,
        release_lease: bool,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<bool> {
        let res = sqlx::query(
            "UPDATE verify_jobs SET
                 cursor_user = COALESCE($3, cursor_user),
                 processed   = processed + $4,
                 errors      = errors + $5,
                 counts      = $6,
                 lease_until = CASE WHEN $7 THEN NULL ELSE lease_until END,
                 updated_at  = CURRENT_TIMESTAMP
             WHERE guild_id = $1 AND generation = $2 AND status = 'running'",
        )
        .bind(bd(self.guild_id.get()))
        .bind(self.generation)
        .bind(cursor.map(|c| bd(c.get())))
        .bind(processed_delta)
        .bind(errors_delta)
        .bind(serde_json::to_string(&self.counts)?)
        .bind(release_lease)
        .execute(pg_pool)
        .await?;
        Ok(res.rows_affected() == 1)
    }

    /// Marks the job succeeded and folds the accumulated per-role counts into
    /// the guild's `VerifyRole::members`. Returns `false` when superseded.
    pub async fn complete(&self, pg_pool: &Pool<Postgres>) -> anyhow::Result<bool> {
        let res = sqlx::query(
            "UPDATE verify_jobs SET
                 status      = 'succeeded',
                 lease_until = NULL,
                 processed   = total,
                 counts      = $3,
                 updated_at  = CURRENT_TIMESTAMP
             WHERE guild_id = $1 AND generation = $2 AND status = 'running'",
        )
        .bind(bd(self.guild_id.get()))
        .bind(self.generation)
        .bind(serde_json::to_string(&self.counts)?)
        .execute(pg_pool)
        .await?;
        if res.rows_affected() == 0 {
            return Ok(false);
        }

        fold_counts_into_guild(self.guild_id, &self.scope, &self.counts, pg_pool).await?;
        Ok(true)
    }
}

/// Overwrites `members` for every config role covered by `scope` with the
/// count the completed scan derived. Roles the scan did not cover keep their
/// delta-maintained counts; roles covered but matched by nobody get 0 (they
/// have no entry in `counts`).
pub async fn fold_counts_into_guild(
    guild_id: Id<GuildMarker>,
    scope: &ReconScope,
    counts: &HashMap<Id<RoleMarker>, u32>,
    pg_pool: &Pool<Postgres>,
) -> anyhow::Result<()> {
    let Some(mut verify) = guild_verify_from_db(guild_id, pg_pool).await? else {
        return Ok(()); // guild row vanished — nothing to fold into
    };
    let mut changed = false;
    for role in &mut verify.roles {
        if scope.covers_config_role(role.role_id) {
            let count = counts.get(&role.role_id).copied().unwrap_or(0);
            if role.members != count {
                role.members = count;
                changed = true;
            }
        }
    }
    if changed {
        guild_verify_save(guild_id, &verify, pg_pool).await?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// guild_user_links — normalized per-guild link rows
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct GuildUserLink {
    pub user_id: Id<UserMarker>,
    pub links: Vec<Link>,
}

impl GuildUserLink {
    /// Next batch after `cursor` in `user_id` order — O(batch) regardless of
    /// guild size, thanks to the (guild_id, user_id) primary key.
    pub async fn next_batch(
        guild_id: Id<GuildMarker>,
        cursor: Option<Id<UserMarker>>,
        limit: i64,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<Vec<GuildUserLink>> {
        let rows = sqlx::query(
            "SELECT user_id, links FROM guild_user_links
             WHERE guild_id = $1 AND user_id > $2
             ORDER BY user_id
             LIMIT $3",
        )
        .bind(bd(guild_id.get()))
        .bind(BigDecimal::from(cursor.map(|c| c.get()).unwrap_or(0)))
        .bind(limit)
        .fetch_all(pg_pool)
        .await?;

        rows.iter()
            .map(|row| {
                let user_id = row
                    .get::<BigDecimal, _>("user_id")
                    .to_u64()
                    .ok_or_else(|| anyhow::anyhow!("user id out of range"))?;
                Ok(GuildUserLink {
                    user_id: Id::new(user_id),
                    links: serde_json::from_str(row.get::<&str, _>("links"))?,
                })
            })
            .collect()
    }

    pub async fn get(
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<Option<Vec<Link>>> {
        let row = sqlx::query(
            "SELECT links FROM guild_user_links WHERE guild_id = $1 AND user_id = $2",
        )
        .bind(bd(guild_id.get()))
        .bind(bd(user_id.get()))
        .fetch_optional(pg_pool)
        .await?;
        row.map(|row| Ok(serde_json::from_str(row.get::<&str, _>("links"))?))
            .transpose()
    }

    /// Single-row upsert used by the (still synchronous) per-user link
    /// handlers — replaces the old whole-blob read-modify-write.
    pub async fn upsert(
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        links: &[Link],
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO guild_user_links (guild_id, user_id, links) VALUES ($1, $2, $3)
             ON CONFLICT (guild_id, user_id)
             DO UPDATE SET links = $3, updated_at = CURRENT_TIMESTAMP",
        )
        .bind(bd(guild_id.get()))
        .bind(bd(user_id.get()))
        .bind(serde_json::to_string(links)?)
        .execute(pg_pool)
        .await?;
        Ok(())
    }

    pub async fn remove(
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM guild_user_links WHERE guild_id = $1 AND user_id = $2")
            .bind(bd(guild_id.get()))
            .bind(bd(user_id.get()))
            .execute(pg_pool)
            .await?;
        Ok(())
    }

    pub async fn count(
        guild_id: Id<GuildMarker>,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<i64> {
        let row = sqlx::query("SELECT count(*) AS c FROM guild_user_links WHERE guild_id = $1")
            .bind(bd(guild_id.get()))
            .fetch_one(pg_pool)
            .await?;
        Ok(row.get::<i64, _>("c"))
    }

    /// Seeds the table from a legacy blob's `user_links` map.
    ///
    /// `ON CONFLICT DO NOTHING`: rows written by the new single-row handlers
    /// are always fresher than the blob, so the blob must never overwrite
    /// them. Idempotent — safe to run on every legacy `Guild::save` and on
    /// every job start until the blob stops carrying links.
    pub async fn seed_from_blob(
        guild_id: Id<GuildMarker>,
        user_links: &HashMap<Id<UserMarker>, Vec<Link>>,
        pg_pool: &Pool<Postgres>,
    ) -> anyhow::Result<()> {
        const CHUNK: usize = 500;
        let entries: Vec<(&Id<UserMarker>, &Vec<Link>)> = user_links.iter().collect();
        for chunk in entries.chunks(CHUNK) {
            let mut qb: QueryBuilder<Postgres> =
                QueryBuilder::new("INSERT INTO guild_user_links (guild_id, user_id, links) ");
            qb.push_values(chunk, |mut b, (user_id, links)| {
                b.push_bind(bd(guild_id.get()))
                    .push_bind(bd(user_id.get()))
                    .push_bind(serde_json::to_string(links).unwrap_or_else(|_| "[]".to_string()));
            });
            qb.push(" ON CONFLICT (guild_id, user_id) DO NOTHING");
            qb.build().execute(pg_pool).await?;
        }
        Ok(())
    }
}

/// The links a user currently has in a guild: the normalized row when
/// present, falling back to the legacy blob entry for guilds whose links
/// haven't been seeded into the table yet.
pub async fn effective_links(
    verify: &Verify,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    pg_pool: &Pool<Postgres>,
) -> anyhow::Result<Option<Vec<Link>>> {
    if let Some(links) = GuildUserLink::get(guild_id, user_id, pg_pool).await? {
        return Ok(Some(links));
    }
    Ok(verify.user_links.get(&user_id).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rid(id: u64) -> Id<RoleMarker> {
        Id::new(id)
    }

    fn active_link(address: &str) -> Link {
        Link {
            link_address: address.to_string(),
            linked_at: 0,
            active: true,
        }
    }

    fn inactive_link(address: &str) -> Link {
        Link {
            link_address: address.to_string(),
            linked_at: 0,
            active: false,
        }
    }

    // -- link matching (moved with the code from api::users::utils) --------

    #[test]
    fn link_match_returns_false_for_invalid_pattern_instead_of_panicking() {
        let l = active_link("https://example.com/user");
        assert!(!link_match(&l, "("));
    }

    #[test]
    fn link_arr_match_returns_false_for_invalid_pattern_instead_of_panicking() {
        let links = vec![active_link("https://example.com/user")];
        assert!(!link_arr_match(&links, "("));
    }

    #[test]
    fn link_arr_match_ignores_inactive_links() {
        let links = vec![inactive_link("a@example.com")];
        assert!(!link_arr_match(&links, r"@example\.com$"));
    }

    #[test]
    fn link_match_still_matches_valid_pattern() {
        let l = active_link("https://example.com/user");
        assert!(link_match(&l, r"^https://example\.com/.*$"));
    }

    // -- scope merge algebra ------------------------------------------------

    #[test]
    fn merge_all_is_sticky() {
        let mut scope = ReconScope::all();
        scope.merge(ReconScope::role_add(rid(1)));
        assert!(scope.all);
        scope.merge(ReconScope::all());
        assert!(scope.all);
    }

    #[test]
    fn merge_removal_survives_all() {
        // The correctness point the docs call out: a deleted role is no
        // longer in config, so `all` alone can never strip it. The removal
        // op must survive any merge.
        let mut scope = ReconScope::role_remove(rid(7), "@old$".into());
        scope.merge(ReconScope::all());
        assert!(scope.all);
        assert_eq!(scope.removals.len(), 1);
        assert_eq!(scope.removals[0].role_id, rid(7));

        // ...and merging removal INTO an all-scope keeps it too.
        let mut scope = ReconScope::all();
        scope.merge(ReconScope::role_remove(rid(7), "@old$".into()));
        assert_eq!(scope.removals.len(), 1);
    }

    #[test]
    fn merge_role_moving_between_sides_keeps_only_latest_side() {
        // add → remove: removal displaces the pending add.
        let mut scope = ReconScope::role_add(rid(3));
        scope.merge(ReconScope::role_remove(rid(3), "@x$".into()));
        assert!(scope.add_role_ids.is_empty());
        assert_eq!(scope.removals.len(), 1);

        // sync → remove: removal displaces the pending sync too.
        let mut scope = ReconScope::role_sync(rid(3));
        scope.merge(ReconScope::role_remove(rid(3), "@x$".into()));
        assert!(scope.sync_role_ids.is_empty());
        assert_eq!(scope.removals.len(), 1);
    }

    #[test]
    fn merge_add_after_removal_escalates_to_sync() {
        // Role deleted then re-created: users still holding the deleted
        // incarnation must be reconciled, which add-only would never do —
        // the merge must escalate to a full per-user sync of that role.
        let mut scope = ReconScope::role_remove(rid(3), "@x$".into());
        scope.merge(ReconScope::role_add(rid(3)));
        assert!(scope.removals.is_empty());
        assert!(scope.add_role_ids.is_empty());
        assert_eq!(scope.sync_role_ids, vec![rid(3)]);
    }

    #[test]
    fn merge_sync_absorbs_add_for_the_same_role() {
        // sync ⊇ add: whichever order they merge in, sync wins.
        let mut scope = ReconScope::role_sync(rid(3));
        scope.merge(ReconScope::role_add(rid(3)));
        assert!(scope.add_role_ids.is_empty());
        assert_eq!(scope.sync_role_ids, vec![rid(3)]);

        let mut scope = ReconScope::role_add(rid(3));
        scope.merge(ReconScope::role_sync(rid(3)));
        assert!(scope.add_role_ids.is_empty());
        assert_eq!(scope.sync_role_ids, vec![rid(3)]);
    }

    #[test]
    fn merge_dedupes_repeated_adds_and_unions_distinct_ops() {
        let mut scope = ReconScope::role_add(rid(1));
        scope.merge(ReconScope::role_add(rid(1)));
        scope.merge(ReconScope::role_add(rid(2)));
        scope.merge(ReconScope::role_sync(rid(4)));
        scope.merge(ReconScope::role_remove(rid(9), "@gone$".into()));
        assert_eq!(scope.add_role_ids, vec![rid(1), rid(2)]);
        assert_eq!(scope.sync_role_ids, vec![rid(4)]);
        assert_eq!(scope.removals.len(), 1);
        assert!(!scope.all);
    }

    #[test]
    fn full_sync_for_distinguishes_add_only_from_sync() {
        let mut scope = ReconScope::role_add(rid(1));
        scope.merge(ReconScope::role_sync(rid(2)));
        // add-only role: covered but NOT full-sync — the worker must not
        // issue remove calls for a brand-new role nobody can hold yet.
        assert!(scope.covers_config_role(rid(1)));
        assert!(!scope.full_sync_for(rid(1)));
        // sync role: covered AND full-sync.
        assert!(scope.covers_config_role(rid(2)));
        assert!(scope.full_sync_for(rid(2)));
        // all implies full sync of everything.
        assert!(ReconScope::all().full_sync_for(rid(3)));
    }

    #[test]
    fn merge_replacing_a_pattern_updates_the_pending_removal() {
        // Same role removed twice with different captured patterns (pattern
        // replaced, then role deleted): latest removal wins.
        let mut scope = ReconScope::role_remove(rid(3), "@first$".into());
        scope.merge(ReconScope::role_remove(rid(3), "@second$".into()));
        assert_eq!(scope.removals.len(), 1);
        assert_eq!(scope.removals[0].pattern, "@second$");
    }

    #[test]
    fn covers_config_role_respects_all_and_add_list() {
        assert!(ReconScope::all().covers_config_role(rid(1)));
        let scoped = ReconScope::role_add(rid(2));
        assert!(scoped.covers_config_role(rid(2)));
        assert!(!scoped.covers_config_role(rid(3)));
    }

    // -- message & scope serialization ---------------------------------------

    #[test]
    fn recon_message_round_trips() {
        let msg = ReconMessage {
            guild_id: Id::new(590643624358969350),
            generation: 42,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ReconMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn recon_scope_round_trips_and_tolerates_missing_fields() {
        let mut scope = ReconScope::role_add(rid(5));
        scope.merge(ReconScope::role_remove(rid(6), "@x$".into()));
        let json = serde_json::to_string(&scope).unwrap();
        let parsed: ReconScope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, scope);

        // Forward-compat: a scope stored before a field existed must parse.
        let parsed: ReconScope = serde_json::from_str(r#"{"all":true}"#).unwrap();
        assert!(parsed.all);
        assert!(parsed.add_role_ids.is_empty());
    }

    // -- verify blob shape ----------------------------------------------------

    #[test]
    fn verify_deserializes_legacy_blob_but_never_serializes_user_links() {
        // Legacy blob with user_links must load (for backfill seeding)...
        let legacy = r#"{"roles":[{"role_id":"1","pattern":"@x$","members":1}],
                         "user_links":{"2":[{"link_address":"a@x","linked_at":0,"active":true}]}}"#;
        let verify: Verify = serde_json::from_str(legacy).unwrap();
        assert_eq!(verify.user_links.len(), 1);

        // ...but must never be written back out.
        let out = serde_json::to_string(&verify).unwrap();
        assert!(!out.contains("user_links"));

        // And a post-migration blob without the field still parses.
        let modern: Verify = serde_json::from_str(r#"{"roles":[]}"#).unwrap();
        assert!(modern.user_links.is_empty());
    }

    // -- member count deltas ----------------------------------------------------

    #[test]
    fn bump_members_applies_deltas_and_saturates_at_zero() {
        let mut verify = Verify {
            roles: vec![VerifyRole {
                role_id: rid(1),
                pattern: "@x$".into(),
                members: 0,
            }],
            user_links: HashMap::new(),
        };
        verify.bump_members(rid(1), 1);
        assert_eq!(verify.roles[0].members, 1);
        verify.bump_members(rid(1), -1);
        assert_eq!(verify.roles[0].members, 0);
        // The old hand-rolled counters could underflow; saturating math can't.
        verify.bump_members(rid(1), -1);
        assert_eq!(verify.roles[0].members, 0);
        // Unknown role: silently ignored, no panic.
        verify.bump_members(rid(99), 1);
    }

    #[test]
    fn job_status_parse_round_trips_and_flags_terminal() {
        for s in [
            JobStatus::Pending,
            JobStatus::Running,
            JobStatus::Succeeded,
            JobStatus::Failed,
        ] {
            assert_eq!(JobStatus::parse(s.as_str()), Some(s));
        }
        assert_eq!(JobStatus::parse("bogus"), None);
        assert!(JobStatus::Succeeded.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(!JobStatus::Pending.is_terminal());
    }
}
