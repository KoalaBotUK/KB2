# Verify Role Reconciliation — Low Level Design

Status: **Proposed** · Companion to
[`high-level-architecture/verify-role-reconciliation.md`](../high-level-architecture/verify-role-reconciliation.md)
(high-level design & decision record). This document specifies the implementation: schemas,
module layout, code, message contracts, infra diffs, tuning constants, and the failure matrix.

Code snippets are written against the current codebase conventions (`sqlx` + `BigDecimal`
ids, `ise()` error mapping, `kind` message-attribute dispatch in `consumer`) and are
illustrative — final signatures may shift slightly during implementation.

---

## 1. Change map

| Area | File(s) | Change |
|---|---|---|
| Migrations | `api/migrations/2026xxxx_verify_recon.{up,down}.sql` | New `guild_user_links`, `verify_jobs` tables |
| Shared types | `common/src/verify.rs` *(new)*, `common/src/lib.rs` | `ReconMessage`, `ReconScope`, `VerifyJob`, `GuildUserLink` + their SQL |
| Shared Discord | `common/src/discord.rs` *(new)* | `retry_on_rl`-style helpers needed by the worker (moved/split from `api/src/discord.rs`) |
| API handlers | `api/src/guilds/verify/controllers.rs` | `put_roles_id` / `delete_roles_id` / `post_recon` become enqueue-only; new `get_job` |
| API handlers | `api/src/users/link_guilds.rs`, `api/src/users/links.rs` | Write `guild_user_links` rows instead of the guild blob's `user_links` |
| API enqueue | `api/src/recon.rs` *(new)* | `enqueue_recon()` — sibling of `api/src/audit.rs::audit()` |
| Consumer | `consumer/src/main.rs`, `consumer/src/verify/` *(new)* | `kind = "verify_recon"` dispatch arm + batch worker |
| OpenAPI | `api/openapi/openapi.yaml` | `VerifyJob` schema, `GET /guilds/{guild_id}/verify/job`, 202 responses |
| UI | `ui/src/helpers/verify.js`, `ui/src/components/dashboard/VerifyComponent.vue` | Job polling + progress bar |
| Infra | `infra/modules/compute/lambda/main.tf`, `infra/modules/data/sqs/main.tf` | Consumer: `sqs:SendMessage`, `SQS_URL`, `DISCORD_BOT_TOKEN`, timeout 120s; queue visibility timeout |

> **Crate layout note:** the worker runs in the `consumer` binary crate, which cannot
> depend on `api` (also a binary). Everything the worker shares with the API — models,
> matching, job SQL, Discord role calls — therefore lives in `common`. `Link` and
> `link_arr_match` move from `api::users` to `common::verify` with re-exports left behind
> so the `api` diff stays mechanical.

---

## 2. Database

### 2.1 Migration `up`

```sql
-- no-transaction
CREATE TABLE IF NOT EXISTS guild_user_links(
    guild_id   NUMERIC(20, 0) NOT NULL, -- u64
    user_id    NUMERIC(20, 0) NOT NULL, -- u64
    links      TEXT NOT NULL,           -- JSON Vec<Link>, same shape as users.links
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (guild_id, user_id)     -- also serves keyset pagination
);
```

```sql
-- no-transaction
CREATE TABLE IF NOT EXISTS verify_jobs(
    guild_id    NUMERIC(20, 0) NOT NULL PRIMARY KEY, -- one job row per guild, ever
    generation  BIGINT  NOT NULL DEFAULT 1,
    status      TEXT    NOT NULL,                    -- pending | running | succeeded | failed
    scope       TEXT    NOT NULL,                    -- JSON ReconScope (see §3.1)
    cursor      NUMERIC(20, 0),                      -- last processed user_id, NULL = start
    total       INTEGER NOT NULL DEFAULT 0,
    processed   INTEGER NOT NULL DEFAULT 0,
    errors      INTEGER NOT NULL DEFAULT 0,
    counts      TEXT    NOT NULL DEFAULT '{}',       -- JSON {role_id: matched_count} accumulator
    lease_until TIMESTAMP,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

`down` drops both tables. Two migration files if DSQL's one-DDL-per-migration constraint
applies (as with the audit index migrations).

### 2.2 Backfill strategy (from `guilds.verify.user_links`)

No SQL backfill (JSON explosion of a TEXT column is not portable to DSQL). Instead the
transition is done in code, in this order:

1. **Dual-write:** link handlers write both the blob and `guild_user_links`.
2. **Seed-on-first-job:** when the worker starts a job (`cursor IS NULL`) and
   `SELECT count(*) FROM guild_user_links WHERE guild_id = $1` is `0` but the blob has
   `user_links`, it inserts one row per entry before scanning. Idempotent
   (`ON CONFLICT DO NOTHING`), bounded by guild size, runs once per guild.
3. **Cut-over:** readers switch to the table; blob writes stop; `user_links` is dropped
   from serialized `Verify` (kept as `#[serde(default, skip_serializing)]` for one release
   so old blobs still deserialize).

---

## 3. Shared types — `common/src/verify.rs`

### 3.1 Scope and message

The scope is a **merge-able set of pending operations**, not a single enum value. This is
a correctness point: a `RoleRemove` must survive being merged with `All`, because the
removed role no longer exists in the guild config — a plain "widen to All" would forget
to strip it.

```rust
use serde::{Deserialize, Serialize};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker};

#[derive(Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RoleRemoval {
    pub role_id: Id<RoleMarker>,
    /// Pattern captured at removal time — the role is gone from config,
    /// so the worker needs it to know which users to strip (parity with
    /// the current `remove_existing_role` behaviour).
    pub pattern: String,
}

#[derive(Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ReconScope {
    /// Full reconcile: every config role, add-or-remove per user (recon).
    pub all: bool,
    /// Newly added/changed roles: add-only, for users whose links match.
    pub add_role_ids: Vec<Id<RoleMarker>>,
    /// Roles deleted from config: remove from users whose links match(ed).
    pub removals: Vec<RoleRemoval>,
}

impl ReconScope {
    pub fn all() -> Self { ReconScope { all: true, ..Default::default() } }

    pub fn role_add(role_id: Id<RoleMarker>) -> Self {
        ReconScope { add_role_ids: vec![role_id], ..Default::default() }
    }

    pub fn role_remove(role_id: Id<RoleMarker>, pattern: String) -> Self {
        ReconScope { removals: vec![RoleRemoval { role_id, pattern }], ..Default::default() }
    }

    /// Fold `other` (the newest change) into `self` (the in-flight scope).
    /// Union semantics; a role moving between add/remove keeps only its
    /// latest side. `all` is sticky.
    pub fn merge(&mut self, other: ReconScope) {
        self.all |= other.all;
        for r in other.removals {
            self.add_role_ids.retain(|id| *id != r.role_id);
            self.removals.retain(|x| x.role_id != r.role_id);
            self.removals.push(r);
        }
        for id in other.add_role_ids {
            self.removals.retain(|x| x.role_id != id);
            if !self.add_role_ids.contains(&id) {
                self.add_role_ids.push(id);
            }
        }
    }
}

/// SQS body for message attribute kind = "verify_recon".
/// Deliberately minimal: verify_jobs is the source of truth; the message is
/// only a wake-up token. Stale tokens (generation < row generation) are dropped.
#[derive(Serialize, Deserialize)]
pub struct ReconMessage {
    pub guild_id: Id<GuildMarker>,
    pub generation: i64,
}
```

### 3.2 `VerifyJob` — lease / supersede / checkpoint SQL

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct VerifyJob {
    pub guild_id: Id<GuildMarker>,
    pub generation: i64,
    pub status: String,
    pub scope: ReconScope,
    pub cursor: Option<Id<UserMarker>>,
    pub total: i32,
    pub processed: i32,
    pub errors: i32,
    pub counts: HashMap<Id<RoleMarker>, u32>,
}
```

**Supersede (API side).** Called by every handler that changes verify desired state.
Read-merge-upsert; the `ON CONFLICT` increment is atomic, so concurrent bumps both land —
the message chain that observes the *highest* generation wins, and both enqueued tokens
point at a row whose scope contains both changes (a lost scope-merge race between two
simultaneous admin edits degrades to `all: true`, never to a dropped operation, because
the loser's handler re-reads and re-merges before its upsert):

```rust
impl VerifyJob {
    /// Bump the guild's job to a new generation with `scope` folded in.
    /// Returns the new generation to embed in the SQS message.
    pub async fn supersede(
        guild_id: Id<GuildMarker>,
        scope: ReconScope,
        pg_pool: &Pool<Postgres>,
    ) -> Result<i64, StatusCode> {
        // Merge with the existing scope only if the job hasn't finished —
        // a terminal job's scope is history, not pending work.
        let mut merged = scope;
        if let Some(existing) = Self::from_db(guild_id, pg_pool).await? {
            if existing.status == "pending" || existing.status == "running" {
                let mut base = existing.scope;
                base.merge(merged);
                merged = base;
            }
        }

        let row = sqlx::query(
            "INSERT INTO verify_jobs (guild_id, generation, status, scope, total)
             VALUES ($1, 1, 'pending', $2,
                     (SELECT count(*) FROM guild_user_links WHERE guild_id = $1))
             ON CONFLICT (guild_id) DO UPDATE SET
                 generation = verify_jobs.generation + 1,
                 status     = 'pending',
                 scope      = $2,
                 cursor     = NULL,
                 processed  = 0,
                 errors     = 0,
                 counts     = '{}',
                 total      = EXCLUDED.total,
                 updated_at = CURRENT_TIMESTAMP
             RETURNING generation",
        )
        .bind(BigDecimal::from(guild_id.into_nonzero().get()))
        .bind(serde_json::to_string(&merged).map_err(ise)?)
        .fetch_one(pg_pool)
        .await
        .map_err(ise)?;

        Ok(row.get::<i64, _>("generation"))
    }
}
```

**Lease acquisition (worker side).** A conditional `UPDATE` is the mutex; zero rows means
someone else holds it (or the token is stale) and the message is dropped as a success:

```rust
/// Returns the job iff this invocation now exclusively owns it.
pub async fn acquire_lease(
    guild_id: Id<GuildMarker>,
    generation: i64,
    lease_secs: i64,
    pg_pool: &Pool<Postgres>,
) -> Result<Option<VerifyJob>, Error> {
    let res = sqlx::query(
        "UPDATE verify_jobs SET
             status = 'running',
             lease_until = CURRENT_TIMESTAMP + make_interval(secs => $3),
             updated_at  = CURRENT_TIMESTAMP
         WHERE guild_id = $1
           AND generation = $2                       -- stale token ⇒ no-op
           AND status IN ('pending', 'running')      -- terminal ⇒ no-op
           AND (lease_until IS NULL OR lease_until < CURRENT_TIMESTAMP)",
    )
    .bind(BigDecimal::from(guild_id.into_nonzero().get()))
    .bind(generation)
    .bind(lease_secs as f64)
    .execute(pg_pool)
    .await?;

    if res.rows_affected() == 0 {
        return Ok(None);
    }
    VerifyJob::from_db_common(guild_id, pg_pool).await
}
```

**Checkpoint (worker side).** Guarded by `generation` so a supersede mid-flight is
detected at the next checkpoint; the final checkpoint of an invocation also releases the
lease so the continuation message (delivered seconds later) can acquire it:

```rust
/// Persist progress. `rows_affected == 0` ⇒ superseded ⇒ caller must stop.
pub async fn checkpoint(
    &self,
    cursor: Id<UserMarker>,
    processed_delta: i32,
    errors_delta: i32,
    release_lease: bool,
    pg_pool: &Pool<Postgres>,
) -> Result<bool, Error> {
    let res = sqlx::query(
        "UPDATE verify_jobs SET
             cursor      = $3,
             processed   = processed + $4,
             errors      = errors + $5,
             counts      = $6,
             lease_until = CASE WHEN $7 THEN NULL ELSE lease_until END,
             updated_at  = CURRENT_TIMESTAMP
         WHERE guild_id = $1 AND generation = $2",
    )
    /* bindings elided */
    .execute(pg_pool)
    .await?;
    Ok(res.rows_affected() == 1)
}
```

### 3.3 `GuildUserLink` — keyset pagination

```rust
pub struct GuildUserLink {
    pub user_id: Id<UserMarker>,
    pub links: Vec<Link>,
}

/// Next batch after `cursor`, O(batch) regardless of guild size.
pub async fn next_batch(
    guild_id: Id<GuildMarker>,
    cursor: Option<Id<UserMarker>>,
    limit: i64,
    pg_pool: &Pool<Postgres>,
) -> Result<Vec<GuildUserLink>, Error> {
    sqlx::query(
        "SELECT user_id, links FROM guild_user_links
         WHERE guild_id = $1 AND user_id > $2
         ORDER BY user_id
         LIMIT $3",
    )
    .bind(BigDecimal::from(guild_id.into_nonzero().get()))
    .bind(BigDecimal::from(cursor.map(|c| c.get()).unwrap_or(0)))
    .bind(limit)
    .fetch_all(pg_pool)
    .await?
    /* hydrate rows: parse links JSON with the parse-don't-panic pattern */
}

/// Single-row upsert used by the (still synchronous) per-user link handlers.
pub async fn upsert(
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    links: &[Link],
    pg_pool: &Pool<Postgres>,
) -> Result<(), StatusCode> {
    sqlx::query(
        "INSERT INTO guild_user_links (guild_id, user_id, links) VALUES ($1, $2, $3)
         ON CONFLICT (guild_id, user_id)
         DO UPDATE SET links = $3, updated_at = CURRENT_TIMESTAMP",
    )
    /* bindings elided */
    .execute(pg_pool).await.map_err(ise)?;
    Ok(())
}
```

---

## 4. API changes

### 4.1 `api/src/recon.rs` — enqueue helper (sibling of `audit.rs`)

Unlike `audit()` (fire-and-forget, drops on failure), enqueue failures here must surface:
a saved role with no job message would strand desired state. The handler therefore returns
`Err(500)` if the send ultimately fails — the admin retries, and `supersede` is idempotent
in effect (another generation bump).

```rust
pub async fn enqueue_recon(
    guild_id: Id<GuildMarker>,
    generation: i64,
    delay_seconds: i32,
    sqs: &aws_sdk_sqs::Client,
) -> Result<(), StatusCode> {
    let queue_url = std::env::var("SQS_URL").expect("SQS_URL must be set");
    let body = serde_json::to_string(&ReconMessage { guild_id, generation }).map_err(ise)?;
    let kind = MessageAttributeValue::builder()
        .data_type("String")
        .string_value("verify_recon")
        .build()
        .map_err(ise)?;

    // Same 3-attempt exponential backoff as audit(), but propagate the error.
    send_with_retry(sqs, &queue_url, &body, kind, delay_seconds)
        .await
        .map_err(ise)
}
```

### 4.2 `put_roles_id` — before/after

The ~40-line fan-out loop is replaced by persist-and-enqueue. Full handler:

```rust
async fn put_roles_id(
    Path((guild_id, role_id)): Path<(Id<GuildMarker>, Id<RoleMarker>)>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
    Json(put_role_request): Json<PutRoleRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }
    validate_verify_pattern(&put_role_request.pattern)?;

    let mut guild = Guild::from_db(guild_id, &app_state.pg_pool)
        .await?
        .unwrap_or_else(|| Guild { guild_id, ..Default::default() });

    // Replacing an existing role's pattern must strip users who matched the
    // OLD pattern but not the new one ⇒ a removal op for the old pattern is
    // folded into the same job as the add.
    let mut scope = ReconScope::role_add(role_id);
    if let Some(old) = guild.verify.roles.iter().find(|r| r.role_id == role_id) {
        scope.merge(ReconScope::role_remove(role_id, old.pattern.clone()));
    }

    guild.verify.roles.retain(|r| r.role_id != role_id);
    let new_role = VerifyRole { role_id, pattern: put_role_request.pattern, members: 0 };
    guild.verify.roles.push(new_role.clone());
    guild.save(&app_state.pg_pool).await?;                       // 1. desired state

    let generation = VerifyJob::supersede(guild_id, scope, &app_state.pg_pool).await?; // 2. job
    enqueue_recon(guild_id, generation, 0, &app_state.sqs).await?;                     // 3. wake worker

    let job = VerifyJob::from_db(guild_id, &app_state.pg_pool).await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "role": new_role, "job": job }))))
}
```

`delete_roles_id` is symmetric: capture the pattern, `retain` the role out of config,
`save`, `supersede(role_remove)`, enqueue, `202` with the job. `post_recon` keeps its
bot-token auth check and becomes three lines: `supersede(ReconScope::all())`, enqueue,
`202`.

Ordering is deliberate: desired state commits **before** the job bump, and the job bump
**before** the message, so a crash between any two steps leaves a state that the next
retry or the next admin action repairs (see §8).

### 4.3 New endpoint — job status

```rust
// GET /guilds/{guild_id}/verify/job  (admin-gated like the other verify routes)
async fn get_job(
    Path(guild_id): Path<Id<GuildMarker>>,
    Extension(current_user): Extension<CurrentUser>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    if !is_client_admin_guild(guild_id, &current_user, &app_state.discord_bot).await? {
        return Err(StatusCode::FORBIDDEN);
    }
    let job = VerifyJob::from_db(guild_id, &app_state.pg_pool)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(json!(job)))
}
```

OpenAPI addition (`api/openapi/openapi.yaml`):

```yaml
VerifyJob:
  type: object
  properties:
    status: { type: string, enum: [pending, running, succeeded, failed] }
    total: { type: integer }
    processed: { type: integer }
    errors: { type: integer }
    updatedAt: { type: string, format: date-time }
```

### 4.4 Per-user link handlers (stay synchronous)

`put_link_guilds_id` / `delete_link_guilds_id` / `post_link` keep their inline per-user
role calls (a handful of requests) but swap blob mutation for the single-row upsert:

```rust
// was: guild.verify.user_links.insert(user_id, user.links.clone()); guild.save(...)
let previous_links = GuildUserLink::get(guild_id, user_id, &app_state.pg_pool).await?;
GuildUserLink::upsert(guild_id, user_id, &user.links, &app_state.pg_pool).await?;

for verify_role in &guild.verify.roles {
    if role_newly_qualifies(&user.links, previous_links.as_deref(), &verify_role.pattern) {
        add_guild_member_role(guild_id, user_id, verify_role.role_id, &app_state.discord_bot).await?;
        bump_member_count(&mut guild, verify_role.role_id, 1);   // ±1 delta, no full recompute
    }
}
guild.save(&app_state.pg_pool).await?;   // roles config incl. members counts only
```

`recompute_role_members()` (O(all users) in memory) is deleted; counts are maintained by
±1 deltas on the per-user paths and set exactly from the job's `counts` accumulator when a
reconciliation completes.

---

## 5. Consumer changes

### 5.1 Dispatch — `consumer/src/main.rs`

```rust
// AppState gains the bot client + sqs client (for continuations):
pub struct AppState {
    pub pg_pool: Pool<Postgres>,
    pub sqs: aws_sdk_sqs::Client,
    pub discord_bot: Arc<twilight_http::Client>,   // from DISCORD_BOT_TOKEN
}

match attr.string_value.as_deref() {
    Some("audit") => { /* unchanged */ }
    Some("verify_recon") => {
        if let Err(e) = verify::consume(message.clone(), state).await {
            error!("Failed to process verify_recon {}: {}", message_id, e);
            response.add_failure(message_id);      // redeliver ⇒ resume from checkpoint
        }
    }
    _ => /* unchanged */
}
```

### 5.2 Worker — `consumer/src/verify/consumer.rs`

Constants (see §7 for rationale):

```rust
const BATCH_SIZE: i64 = 100;          // guild_user_links rows per DB page
const WORK_BUDGET: Duration = Duration::from_secs(60);   // per invocation
const LEASE_SECS: i64 = 240;          // 4 min: > worst-case invocation incl. timeout
const SHORT_WAIT: Duration = Duration::from_secs(2);     // 429s absorbed inline below this
const MAX_INLINE_WAIT: Duration = Duration::from_secs(10); // cumulative inline-sleep budget
```

Full control flow:

```rust
pub async fn consume(message: SqsMessage, state: &AppState) -> Result<(), Error> {
    let msg: ReconMessage = serde_json::from_str(message.body.as_deref().ok_or("no body")?)?;

    // 1. Own the job — or discover we shouldn't be running.
    let Some(mut job) =
        VerifyJob::acquire_lease(msg.guild_id, msg.generation, LEASE_SECS, &state.pg_pool).await?
    else {
        return Ok(()); // stale generation, terminal job, or live lease elsewhere: drop
    };

    // 2. Load desired state once per invocation.
    let guild = Guild::from_db_common(msg.guild_id, &state.pg_pool).await?
        .unwrap_or_default();
    seed_links_from_blob_if_needed(&guild, &job, &state.pg_pool).await?; // §2.2 backfill

    let deadline = Instant::now() + WORK_BUDGET;
    let mut inline_wait_spent = Duration::ZERO;
    let mut backoff_secs: i32 = 0;

    // 3. Batched scan from the checkpoint.
    'work: while Instant::now() < deadline {
        let batch = GuildUserLink::next_batch(
            msg.guild_id, job.cursor, BATCH_SIZE, &state.pg_pool,
        ).await?;

        if batch.is_empty() {
            job.complete(&guild, &state.pg_pool).await?; // status ⇒ succeeded, fold counts
            return Ok(());                               // into guilds.verify role members
        }

        let (mut processed, mut errors) = (0, 0);
        for row in &batch {
            match reconcile_user(&guild, &job.scope, row, state, &mut job.counts).await {
                Ok(()) => processed += 1,
                Err(UserError::RateLimited { retry_after }) if retry_after <= SHORT_WAIT
                    && inline_wait_spent + retry_after <= MAX_INLINE_WAIT =>
                {
                    inline_wait_spent += retry_after;
                    sleep(retry_after).await;            // cheap sub-second absorb
                    // re-run this user next invocation: don't advance past them
                    break;
                }
                Err(UserError::RateLimited { retry_after }) => {
                    backoff_secs = retry_after.as_secs() as i32; // long 429 ⇒ free wait via SQS
                    break;
                }
                Err(UserError::Skip) => { errors += 1; processed += 1; } // 403/404: count & move on
                Err(UserError::Fatal(e)) => return Err(e),               // DB/5xx: redeliver message
            }
        }

        // 4. Checkpoint up to the last fully-processed user.
        let cursor = batch[processed.saturating_sub(1).min(batch.len() - 1)].user_id;
        let releasing = backoff_secs > 0; // ending the invocation early
        if !job.checkpoint(cursor, processed as i32, errors as i32, releasing, &state.pg_pool).await? {
            return Ok(()); // generation bumped mid-flight: the new chain owns the work
        }
        job.cursor = Some(cursor);

        if backoff_secs > 0 { break 'work; }
    }

    // 5. Hand off: release lease (if not already), then continuation token.
    job.release_lease(&state.pg_pool).await?;
    enqueue_recon_common(msg.guild_id, msg.generation, backoff_secs, &state.sqs).await?;
    Ok(())
}
```

Per-user reconciliation — the one place the three legacy loops collapse into:

```rust
async fn reconcile_user(
    guild: &Guild,
    scope: &ReconScope,
    row: &GuildUserLink,
    state: &AppState,
    counts: &mut HashMap<Id<RoleMarker>, u32>,
) -> Result<(), UserError> {
    // Explicit removals first (roles no longer in config).
    for removal in &scope.removals {
        if link_arr_match(&row.links, &removal.pattern) {
            remove_role(guild.guild_id, row.user_id, removal.role_id, state).await?;
        }
    }

    for role in &guild.verify.roles {
        let in_scope = scope.all || scope.add_role_ids.contains(&role.role_id);
        if !in_scope { continue; }

        if link_arr_match(&row.links, &role.pattern) {
            add_role(guild.guild_id, row.user_id, role.role_id, state).await?; // idempotent PUT
            *counts.entry(role.role_id).or_default() += 1;
        } else if scope.all {
            // Parity with legacy post_recon: full recon also strips non-matchers.
            remove_role(guild.guild_id, row.user_id, role.role_id, state).await?; // idempotent DELETE
        }
        // scoped role_add + no match ⇒ zero Discord calls for this user
    }
    Ok(())
}
```

`add_role`/`remove_role` wrap the twilight calls **without** the in-process
`retry_on_rl` sleep loop — a 429 is surfaced as `UserError::RateLimited` so the
orchestration layer (inline short sleep, else `DelaySeconds`) owns all waiting. 404
(member left) and 403 (role hierarchy) map to `UserError::Skip`, mirroring the legacy
handlers' NOT_FOUND tolerance.

`job.complete()` finalizes:

```sql
UPDATE verify_jobs SET status = 'succeeded', lease_until = NULL,
                       processed = total, updated_at = CURRENT_TIMESTAMP
WHERE guild_id = $1 AND generation = $2;
-- plus: fold `counts` into guilds.verify roles' `members` (read-modify-write of
-- the roles config guarded by re-reading; counts for roles in scope only)
```

---

## 6. UI changes

`ui/src/helpers/verify.js`:

```js
export async function getVerifyJob(guildId, accessToken) {
  return axios.get(`${KB_API_URL}/guilds/${guildId}/verify/job`,
    { headers: { Authorization: `Bearer ${accessToken}` } });
}
```

`VerifyComponent.vue` — `addVerifyRole()` now receives `{role, job}` from the 202 and
starts polling; poll cadence backs off for long jobs:

```js
const jobRef = ref(null);
let pollTimer = null;

function trackJob(job) {
  jobRef.value = job;
  clearTimeout(pollTimer);
  if (job && (job.status === 'pending' || job.status === 'running')) {
    // 2s while small/fast, 15s once it's clearly a long-running big-guild job
    const delay = job.total > 2000 ? 15000 : 2000;
    pollTimer = setTimeout(async () => {
      const resp = await getVerifyJob(props.guild.guildId, userRef.value.token.accessToken);
      trackJob(resp.data);
      if (resp.data.status === 'succeeded') emits('update'); // refresh member counts
    }, delay);
  }
}
```

```html
<progress v-if="jobRef && jobRef.status !== 'succeeded'"
          class="progress progress-primary"
          :value="jobRef.processed" :max="jobRef.total"/>
<span v-if="jobRef?.status === 'running'">
  Syncing roles… {{ jobRef.processed.toLocaleString() }} / {{ jobRef.total.toLocaleString() }}
  <span v-if="jobRef.errors" class="text-warning">({{ jobRef.errors }} skipped)</span>
</span>
```

The recon button disables while a job with `scope.all` is non-terminal; role add/remove
stay enabled (supersede handles overlap).

---

## 7. Infra diff & tuning

`infra/modules/compute/lambda/main.tf`:

```hcl
resource "aws_lambda_function" "lambda_consumer" {
  # ...
  timeout = 120        # was 10 — must exceed WORK_BUDGET (60s) + batch tail

  environment {
    variables = {
      # existing vars...
      SQS_URL           = var.sqs_url             # NEW: continuation sends
      DISCORD_BOT_TOKEN = var.discord_bot_token   # NEW: role calls from the worker
    }
  }
}

data "aws_iam_policy_document" "lambda_consumer_sqs_policy" {
  statement {
    effect = "Allow"
    actions = [
      "sqs:ReceiveMessage", "sqs:DeleteMessage", "sqs:GetQueueAttributes",
      "sqs:SendMessage",                          # NEW: self-requeue
    ]
    resources = [var.sqs_arn]
  }
}
```

`infra/modules/data/sqs/main.tf`:

```hcl
resource "aws_sqs_queue" "default" {
  # ...
  visibility_timeout_seconds = 720   # ≥ 6× consumer timeout (AWS ESM guidance)
}
```

Consequence for `audit` messages on the shared queue: a *failed* audit message now waits
up to 12 min before redelivery (successes are unaffected). Acceptable for audit's
fire-and-forget semantics; if it ever isn't, audit moves to its own queue — the consumer
already dispatches on `kind`.

### Tuning constants

| Constant | Value | Rationale |
|---|---|---|
| `BATCH_SIZE` | 100 | ~100 users × 1–2 calls ≈ 10–40 s at the role bucket's pace; several checkpoints per invocation |
| `WORK_BUDGET` | 60 s | Big enough to amortize cold start + guild-config load; small enough that a crash loses ≤ 1 min of work |
| Lambda timeout | 120 s | WORK_BUDGET + one full batch overrun + margin |
| `LEASE_SECS` | 240 s | > Lambda timeout ⇒ a crashed invocation's lease always expires before SQS redelivery can be blocked by it |
| Queue visibility timeout | 720 s | 6 × Lambda timeout (AWS event-source-mapping guidance); also > `LEASE_SECS` so redelivered messages find the lease expired |
| `SHORT_WAIT` / `MAX_INLINE_WAIT` | 2 s / 10 s | Sub-second-to-2s 429s are cheaper to sleep through (~$0.000002) than to round-trip via SQS; anything longer is a free `DelaySeconds` wait |
| `maxReceiveCount` (existing) | 4 | A poisoned recon token lands in the existing DLQ after 4 attempts; the job row shows `running` with an expired lease and is resumed by the next admin action (supersede) |

Throughput check at the design target: 50k rows / 100-row batches = 500 invocations of
≤ 60 s ⇒ ≈ 2 h Discord-rate-limited wall time, ~$0.01–0.02 Lambda + ~1,000 SQS messages
(~$0.0004) per full job. Idle cost: zero.

---

## 8. Failure matrix

| Failure point | What happens | Why it's safe |
|---|---|---|
| API crashes after `guild.save`, before `supersede` | Desired state saved, no job | Next admin action or manual recon reconciles; UI shows no job (visible gap, not silent corruption) |
| API crashes after `supersede`, before send (or send fails 3×) | Job row `pending`, no token | Handler returns 500 ⇒ admin retries ⇒ new generation + token. Old row harmlessly superseded |
| Worker crashes mid-batch (no checkpoint) | Message not deleted; lease expires at ≤ 240 s | SQS redelivers at 720 s; lease is expired; resume from last checkpoint; Discord calls idempotent ⇒ double-apply is a no-op |
| Worker crashes after checkpoint, before continuation send | Same as above | Redelivered original token resumes from the *newer* checkpoint |
| Duplicate SQS delivery (at-least-once) | Second delivery races the first | Lease conditional-update admits exactly one; loser drops as `Ok` |
| Supersede lands mid-invocation | Running worker's next `checkpoint` matches 0 rows | Worker exits `Ok`; new-generation token restarts cursor against merged scope |
| Two admins edit simultaneously | Two supersedes, two tokens | Generation increment is atomic; lower token drops at lease check; scope merge union (worst case degrades to `all`) |
| Discord 403/404 on a user | Counted in `errors`, cursor advances | Parity with legacy NOT_FOUND tolerance; surfaced in UI as "skipped" |
| Discord long 429 / global limit | `DelaySeconds` continuation | Zero compute during the wait |
| Poisoned token (e.g. corrupt job row) | 4 receives ⇒ existing DLQ | Job resumable via any later supersede; DLQ alarm covers observability |

---

## 9. Testing plan

- **Unit (common):** `ReconScope::merge` algebra (all-sticky, add/remove displacement,
  removal survives `all`); `ReconMessage` round-trip; keyset pagination SQL shape;
  checkpoint-returns-false-on-generation-bump (via the same conditional-update pattern
  already unit-tested for `Guild::save` error propagation).
- **Unit (consumer):** `reconcile_user` against fixture links/roles — matrix of
  {scope.all, scoped add, removal} × {match, no-match, multi-link dedupe}; 429
  classification (inline vs backoff); 403/404 ⇒ Skip.
- **Unit (api):** handlers return 202 + job snapshot; pattern-replace folds a removal op
  into scope; enqueue failure ⇒ 500 (not silent).
- **Integration (local, `RUN_LOCAL`):** end-to-end add-role on a seeded 1k-row
  `guild_user_links` against a mocked Discord client; assert call count, final `counts`,
  `succeeded` status, and that a mid-run supersede restarts the cursor.
- **Load sanity (dev env):** synthetic 50k-row guild with a stubbed Discord base URL;
  verify chain completes, one lease holder at a time (assert via `lease_until` sampling),
  progress monotonic.

## 10. Implementation order

1. Migrations + `common::verify` (types, SQL, tests) — no behaviour change.
2. Dual-write in link handlers + seed-on-first-job backfill path.
3. Consumer worker + infra diff (IAM, env, timeouts, visibility timeout) — deployable
   dark, since nothing enqueues `verify_recon` yet.
4. Flip the three API endpoints to enqueue-only + job endpoint + OpenAPI.
5. UI polling.
6. Cleanup release: drop `user_links` from `Verify` serialization and delete
   `recompute_role_members`.

Each step ships independently and is revertible; the legacy synchronous path remains
intact until step 4.
