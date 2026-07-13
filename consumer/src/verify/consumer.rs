//! The verify reconciliation worker: processes one bounded slice of a
//! guild's reconciliation job per SQS delivery, then hands off to a
//! continuation message. See
//! `docs/low-level-architecture/verify-role-reconciliation.md`.
//!
//! Invariants this module maintains:
//!
//! * **One owner.** A conditional-UPDATE lease admits exactly one invocation
//!   per guild; duplicate/stale SQS deliveries drop as successes.
//! * **Crash-safe.** The cursor is checkpointed before the continuation is
//!   sent, Discord role calls are idempotent, and SQS redelivery resumes
//!   from the last checkpoint — safe under every interleaving.
//! * **Supersede-aware.** Every checkpoint is generation-guarded; a config
//!   change mid-flight makes the next checkpoint match zero rows, and this
//!   slice yields to the new generation's token.
//! * **No long sleeps in compute.** Sub-`short_wait` 429s are absorbed
//!   inline under a cumulative budget; anything longer is returned as the
//!   continuation's `DelaySeconds`, where the wait costs nothing.

use std::time::Duration;

use aws_lambda_events::sqs::SqsMessage;
use common::verify::{
    GuildUserLink, ReconMessage, ReconScope, Verify, VerifyJob, VerifyRole, enqueue_recon,
    guild_verify_from_db, link_arr_match,
};
use lambda_runtime::Error;
use sqlx::{Pool, Postgres};
use tokio::time::Instant;
use tracing::{info, warn};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker};

use crate::AppState;
use crate::verify::discord::{DiscordRoles, RoleCallError, TwilightRoles};

#[derive(Clone, Debug)]
pub struct SliceConfig {
    /// `guild_user_links` rows fetched per DB page.
    pub batch_size: i64,
    /// Wall-clock budget for one invocation's useful work. Must sit well
    /// under the Lambda timeout so the final checkpoint + continuation send
    /// always fit.
    pub work_budget: Duration,
    /// Lease duration; must exceed the Lambda timeout so a crashed
    /// invocation's lease always expires before SQS redelivery retries it.
    pub lease: Duration,
    /// 429s with retry_after at or under this are slept through inline —
    /// cheaper than an SQS round-trip.
    pub short_wait: Duration,
    /// Cumulative inline-sleep budget per invocation; beyond it, even short
    /// waits are handed to SQS.
    pub max_inline_wait: Duration,
}

impl Default for SliceConfig {
    fn default() -> Self {
        SliceConfig {
            batch_size: 100,
            work_budget: Duration::from_secs(60),
            lease: Duration::from_secs(240),
            short_wait: Duration::from_secs(2),
            max_inline_wait: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SliceOutcome {
    /// Stale token, terminal job, or a live lease elsewhere — drop the
    /// message as a success; whoever owns the job carries it forward.
    NotAcquired,
    /// A newer generation superseded this job mid-slice; its token owns the
    /// work now.
    Superseded,
    /// The scan finished and the job is marked succeeded.
    Completed,
    /// More work remains: the caller must send a continuation token with
    /// this delay.
    Continue { delay: Duration },
}

/// SQS entry point: runs one slice, then sends the continuation if needed.
pub async fn consume(message: SqsMessage, state: &AppState) -> Result<(), Error> {
    let body = message.body.as_deref().ok_or("missing SQS message body")?;
    let msg: ReconMessage = serde_json::from_str(body)?;

    let gateway = TwilightRoles::new(state.discord_bot.clone());
    let outcome = run_job_slice(&msg, &state.pg_pool, &gateway, &SliceConfig::default()).await?;

    match outcome {
        SliceOutcome::Continue { delay } => {
            let queue_url = std::env::var("SQS_URL").expect("SQS_URL must be set");
            enqueue_recon(msg.guild_id, msg.generation, delay, &state.sqs, &queue_url).await?;
        }
        SliceOutcome::Completed => {
            info!("verify recon completed for guild {}", msg.guild_id);
        }
        SliceOutcome::NotAcquired | SliceOutcome::Superseded => {}
    }
    Ok(())
}

/// Runs one bounded slice of the guild's job. Separated from [`consume`] so
/// integration/perf tests can drive the whole chain in-process without SQS:
/// a `while let Continue { .. }` loop is exactly what the queue does.
pub async fn run_job_slice<G: DiscordRoles>(
    msg: &ReconMessage,
    pg_pool: &Pool<Postgres>,
    gateway: &G,
    cfg: &SliceConfig,
) -> anyhow::Result<SliceOutcome> {
    // 1. Own the job — or discover we shouldn't be running.
    let Some(mut job) =
        VerifyJob::acquire_lease(msg.guild_id, msg.generation, cfg.lease, pg_pool).await?
    else {
        return Ok(SliceOutcome::NotAcquired);
    };

    // 2. Desired state, loaded once per slice.
    let verify_cfg = guild_verify_from_db(msg.guild_id, pg_pool)
        .await?
        .unwrap_or_default();

    // 3. Seed-on-first-job backfill: a legacy guild whose links still live in
    //    the blob gets its rows created here, once, idempotently.
    if job.cursor.is_none() {
        if !verify_cfg.user_links.is_empty()
            && GuildUserLink::count(msg.guild_id, pg_pool).await? == 0
        {
            info!(
                "seeding {} legacy blob link entries for guild {}",
                verify_cfg.user_links.len(),
                msg.guild_id
            );
            GuildUserLink::seed_from_blob(msg.guild_id, &verify_cfg.user_links, pg_pool).await?;
        }
        // The supersede computed `total` before any seeding could run.
        job.refresh_total(pg_pool).await?;
    }

    // 4. Batched scan from the checkpoint.
    let deadline = Instant::now() + cfg.work_budget;
    let mut inline_wait_spent = Duration::ZERO;

    loop {
        let batch =
            GuildUserLink::next_batch(msg.guild_id, job.cursor, cfg.batch_size, pg_pool).await?;

        if batch.is_empty() {
            return Ok(if job.complete(pg_pool).await? {
                SliceOutcome::Completed
            } else {
                SliceOutcome::Superseded
            });
        }

        let mut processed: i32 = 0;
        let mut errors: i32 = 0;
        let mut new_cursor = None;
        let mut backoff = Duration::ZERO;

        for row in &batch {
            if Instant::now() >= deadline {
                break;
            }
            match reconcile_user(&verify_cfg, &job.scope, msg.guild_id, row, gateway).await {
                Ok(outcome) => {
                    for role_id in outcome.matched_roles {
                        *job.counts.entry(role_id).or_default() += 1;
                    }
                    errors += outcome.errors;
                    processed += 1;
                    new_cursor = Some(row.user_id);
                }
                Err(WorkerInterrupt::RateLimited(retry_after))
                    if retry_after <= cfg.short_wait
                        && inline_wait_spent + retry_after <= cfg.max_inline_wait =>
                {
                    // Cheap sub-second absorb; the user is NOT advanced past,
                    // so the next batch fetch retries them (idempotently).
                    tokio::time::sleep(retry_after).await;
                    inline_wait_spent += retry_after;
                    break;
                }
                Err(WorkerInterrupt::RateLimited(retry_after)) => {
                    // Long 429: hand the wait to SQS where it costs nothing.
                    warn!(
                        "rate limited for {:?} reconciling guild {}, deferring",
                        retry_after, msg.guild_id
                    );
                    backoff = retry_after;
                    break;
                }
                Err(WorkerInterrupt::Fatal(e)) => return Err(e),
            }
        }

        let stopping = backoff > Duration::ZERO || Instant::now() >= deadline;

        if processed > 0 || stopping {
            // The final checkpoint of a slice also releases the lease so the
            // continuation token (delivered seconds later) can acquire it.
            if !job
                .checkpoint(new_cursor, processed, errors, stopping, pg_pool)
                .await?
            {
                return Ok(SliceOutcome::Superseded);
            }
            if let Some(cursor) = new_cursor {
                job.cursor = Some(cursor);
            }
        }

        if stopping {
            return Ok(SliceOutcome::Continue { delay: backoff });
        }
    }
}

/// Aggregate result of reconciling one user.
#[derive(Debug, Default, PartialEq)]
pub struct UserReconOutcome {
    /// Config roles whose pattern the user's links matched — the job's
    /// per-role member-count accumulator increments once per (user, role).
    pub matched_roles: Vec<Id<RoleMarker>>,
    /// Per-call 403/404 skips (member left, role deleted, hierarchy).
    pub errors: i32,
}

/// Why reconciliation of a user could not finish.
#[derive(Debug)]
pub enum WorkerInterrupt {
    /// The whole user is retried after the wait (calls are idempotent, so
    /// re-running their earlier successful calls is a no-op).
    RateLimited(Duration),
    Fatal(anyhow::Error),
}

/// Reconciles a single user against the desired state — the one code path
/// that replaces the three legacy synchronous loops (`put_roles_id`,
/// `remove_existing_role`, `post_recon`):
///
/// 1. explicit removals first (roles deleted from config, matched by their
///    captured pattern);
/// 2. then every in-scope config role: matching links ⇒ idempotent add;
///    non-matching links ⇒ idempotent remove, but only under full-sync
///    semantics (`all`/pattern-replace) — a brand-new add-only role skips
///    remove calls entirely, so non-matching users cost zero Discord calls.
pub async fn reconcile_user<G: DiscordRoles>(
    verify_cfg: &Verify,
    scope: &ReconScope,
    guild_id: Id<GuildMarker>,
    row: &GuildUserLink,
    gateway: &G,
) -> Result<UserReconOutcome, WorkerInterrupt> {
    let mut outcome = UserReconOutcome::default();

    for removal in &scope.removals {
        if link_arr_match(&row.links, &removal.pattern) {
            apply_call(
                gateway.remove_role(guild_id, row.user_id, removal.role_id),
                &mut outcome,
            )
            .await?;
        }
    }

    for role in &verify_cfg.roles {
        if !scope.covers_config_role(role.role_id) {
            continue;
        }
        if link_arr_match(&row.links, &role.pattern) {
            apply_call(
                gateway.add_role(guild_id, row.user_id, role.role_id),
                &mut outcome,
            )
            .await?;
            outcome.matched_roles.push(role.role_id);
        } else if scope.full_sync_for(role.role_id) {
            apply_call(
                gateway.remove_role(guild_id, row.user_id, role.role_id),
                &mut outcome,
            )
            .await?;
        }
    }

    Ok(outcome)
}

async fn apply_call(
    call: impl Future<Output = Result<(), RoleCallError>>,
    outcome: &mut UserReconOutcome,
) -> Result<(), WorkerInterrupt> {
    match call.await {
        Ok(()) => Ok(()),
        Err(RoleCallError::Skip) => {
            outcome.errors += 1;
            Ok(())
        }
        Err(RoleCallError::RateLimited { retry_after }) => {
            Err(WorkerInterrupt::RateLimited(retry_after))
        }
        Err(RoleCallError::Fatal(e)) => Err(WorkerInterrupt::Fatal(e)),
    }
}

/// Roles used by tests and the perf harness to build a `Verify` config.
pub fn verify_role(role_id: u64, pattern: &str) -> VerifyRole {
    VerifyRole {
        role_id: Id::new(role_id),
        pattern: pattern.to_string(),
        members: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::verify::Link;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use twilight_model::id::marker::UserMarker;

    // -- mock gateway ---------------------------------------------------------

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Call {
        action: &'static str, // "add" | "remove"
        user_id: u64,
        role_id: u64,
    }

    /// Records every call; a per-(action,user,role) script queue injects
    /// failures, each consumed once (so a retried call succeeds — mirroring
    /// a transient 429).
    #[derive(Default)]
    struct MockRoles {
        calls: Mutex<Vec<Call>>,
        script: Mutex<HashMap<Call, Vec<MockFailure>>>,
    }

    #[derive(Clone, Copy, Debug)]
    enum MockFailure {
        RateLimited(u64), // retry_after millis
        Skip,
            Fatal,
    }

    impl MockRoles {
        fn fail_next(&self, action: &'static str, user_id: u64, role_id: u64, f: MockFailure) {
            self.script
                .lock()
                .unwrap()
                .entry(Call {
                    action,
                    user_id,
                    role_id,
                })
                .or_default()
                .push(f);
        }

        fn calls(&self) -> Vec<Call> {
            self.calls.lock().unwrap().clone()
        }

        fn respond(&self, call: Call) -> Result<(), RoleCallError> {
            self.calls.lock().unwrap().push(call.clone());
            let failure = {
                let mut script = self.script.lock().unwrap();
                script.get_mut(&call).and_then(|queue| {
                    if queue.is_empty() {
                        None
                    } else {
                        Some(queue.remove(0))
                    }
                })
            };
            match failure {
                None => Ok(()),
                Some(MockFailure::RateLimited(ms)) => Err(RoleCallError::RateLimited {
                    retry_after: Duration::from_millis(ms),
                }),
                Some(MockFailure::Skip) => Err(RoleCallError::Skip),
                Some(MockFailure::Fatal) => {
                    Err(RoleCallError::Fatal(anyhow::anyhow!("mock fatal")))
                }
            }
        }
    }

    impl DiscordRoles for MockRoles {
        async fn add_role(
            &self,
            _guild_id: Id<GuildMarker>,
            user_id: Id<UserMarker>,
            role_id: Id<RoleMarker>,
        ) -> Result<(), RoleCallError> {
            self.respond(Call {
                action: "add",
                user_id: user_id.get(),
                role_id: role_id.get(),
            })
        }

        async fn remove_role(
            &self,
            _guild_id: Id<GuildMarker>,
            user_id: Id<UserMarker>,
            role_id: Id<RoleMarker>,
        ) -> Result<(), RoleCallError> {
            self.respond(Call {
                action: "remove",
                user_id: user_id.get(),
                role_id: role_id.get(),
            })
        }
    }

    // -- fixtures -------------------------------------------------------------

    const GUILD: u64 = 1000;

    fn active_link(address: &str) -> Link {
        Link {
            link_address: address.to_string(),
            linked_at: 0,
            active: true,
        }
    }

    fn row(user_id: u64, addresses: &[&str]) -> GuildUserLink {
        GuildUserLink {
            user_id: Id::new(user_id),
            links: addresses.iter().map(|a| active_link(a)).collect(),
        }
    }

    fn config(roles: Vec<VerifyRole>) -> Verify {
        Verify {
            roles,
            user_links: HashMap::new(),
        }
    }

    async fn run(
        cfg: &Verify,
        scope: &ReconScope,
        row: &GuildUserLink,
        gw: &MockRoles,
    ) -> Result<UserReconOutcome, WorkerInterrupt> {
        reconcile_user(cfg, scope, Id::new(GUILD), row, gw).await
    }

    // -- scope × match matrix ---------------------------------------------------

    #[tokio::test]
    async fn full_recon_adds_matchers_and_strips_non_matchers() {
        let cfg = config(vec![
            verify_role(1, r"@example\.com$"),
            verify_role(2, r"@other\.com$"),
        ]);
        let user = row(10, &["a@example.com"]);
        let gw = MockRoles::default();

        let outcome = run(&cfg, &ReconScope::all(), &user, &gw).await.unwrap();

        // Parity with legacy post_recon: matching role added, non-matching
        // role removed, one call each.
        assert_eq!(
            gw.calls(),
            vec![
                Call { action: "add", user_id: 10, role_id: 1 },
                Call { action: "remove", user_id: 10, role_id: 2 },
            ]
        );
        assert_eq!(outcome.matched_roles, vec![Id::new(1)]);
        assert_eq!(outcome.errors, 0);
    }

    #[tokio::test]
    async fn add_only_scope_never_issues_remove_calls() {
        // A brand-new role: non-matching users must cost ZERO Discord calls
        // — this is what keeps a scoped role-add on a 50k guild at
        // O(matchers), not O(members).
        let cfg = config(vec![verify_role(1, r"@example\.com$")]);
        let scope = ReconScope::role_add(Id::new(1));
        let gw = MockRoles::default();

        let non_matcher = row(10, &["a@other.com"]);
        let outcome = run(&cfg, &scope, &non_matcher, &gw).await.unwrap();
        assert!(gw.calls().is_empty());
        assert!(outcome.matched_roles.is_empty());

        let matcher = row(11, &["b@example.com"]);
        let outcome = run(&cfg, &scope, &matcher, &gw).await.unwrap();
        assert_eq!(
            gw.calls(),
            vec![Call { action: "add", user_id: 11, role_id: 1 }]
        );
        assert_eq!(outcome.matched_roles, vec![Id::new(1)]);
    }

    #[tokio::test]
    async fn sync_scope_adds_or_removes_by_current_pattern() {
        // Pattern replaced: users matching the NEW pattern gain the role,
        // users matching only the old one lose it — without the worker ever
        // knowing the old pattern.
        let cfg = config(vec![verify_role(1, r"@new\.com$")]);
        let scope = ReconScope::role_sync(Id::new(1));
        let gw = MockRoles::default();

        let old_matcher = row(10, &["a@old.com"]);
        run(&cfg, &scope, &old_matcher, &gw).await.unwrap();
        let new_matcher = row(11, &["b@new.com"]);
        run(&cfg, &scope, &new_matcher, &gw).await.unwrap();

        assert_eq!(
            gw.calls(),
            vec![
                Call { action: "remove", user_id: 10, role_id: 1 },
                Call { action: "add", user_id: 11, role_id: 1 },
            ]
        );
    }

    #[tokio::test]
    async fn out_of_scope_roles_are_untouched() {
        let cfg = config(vec![
            verify_role(1, r"@example\.com$"),
            verify_role(2, r"@example\.com$"),
        ]);
        // Only role 1 is in scope; the user matches both patterns.
        let scope = ReconScope::role_add(Id::new(1));
        let gw = MockRoles::default();

        let user = row(10, &["a@example.com"]);
        let outcome = run(&cfg, &scope, &user, &gw).await.unwrap();

        assert_eq!(
            gw.calls(),
            vec![Call { action: "add", user_id: 10, role_id: 1 }]
        );
        assert_eq!(outcome.matched_roles, vec![Id::new(1)]);
    }

    #[tokio::test]
    async fn removal_strips_users_matching_the_captured_pattern() {
        // Role 9 was deleted from config; its pattern rides in the scope.
        let cfg = config(vec![]);
        let scope = ReconScope::role_remove(Id::new(9), r"@example\.com$".to_string());
        let gw = MockRoles::default();

        let matcher = row(10, &["a@example.com"]);
        run(&cfg, &scope, &matcher, &gw).await.unwrap();
        let non_matcher = row(11, &["b@other.com"]);
        run(&cfg, &scope, &non_matcher, &gw).await.unwrap();

        assert_eq!(
            gw.calls(),
            vec![Call { action: "remove", user_id: 10, role_id: 9 }]
        );
    }

    #[tokio::test]
    async fn removal_survives_merge_with_all_and_both_execute() {
        // The scope-algebra correctness point, end to end: recon requested
        // while a role deletion is pending must both strip the deleted role
        // AND fully sync the remaining config role.
        let cfg = config(vec![verify_role(1, r"@example\.com$")]);
        let mut scope = ReconScope::role_remove(Id::new(9), r"@example\.com$".to_string());
        scope.merge(ReconScope::all());
        let gw = MockRoles::default();

        let user = row(10, &["a@example.com"]);
        let outcome = run(&cfg, &scope, &user, &gw).await.unwrap();

        assert_eq!(
            gw.calls(),
            vec![
                Call { action: "remove", user_id: 10, role_id: 9 },
                Call { action: "add", user_id: 10, role_id: 1 },
            ]
        );
        assert_eq!(outcome.matched_roles, vec![Id::new(1)]);
    }

    #[tokio::test]
    async fn multiple_matching_links_count_a_user_once_per_role() {
        // One member, one seat: several matching addresses must not inflate
        // matched_roles (the count accumulator increments once per user).
        let cfg = config(vec![verify_role(1, r"@example\.com$")]);
        let gw = MockRoles::default();

        let user = row(10, &["a@example.com", "b@example.com"]);
        let outcome = run(&cfg, &ReconScope::all(), &user, &gw).await.unwrap();

        assert_eq!(outcome.matched_roles, vec![Id::new(1)]);
        assert_eq!(gw.calls().len(), 1);
    }

    #[tokio::test]
    async fn invalid_stored_pattern_matches_nothing_instead_of_panicking() {
        // Corrupted/legacy pattern in config: under full sync the user is
        // treated as non-matching (role removed), never a panic that would
        // poison the SQS message forever.
        let cfg = config(vec![verify_role(1, "(")]);
        let gw = MockRoles::default();

        let user = row(10, &["a@example.com"]);
        let outcome = run(&cfg, &ReconScope::all(), &user, &gw).await.unwrap();

        assert_eq!(
            gw.calls(),
            vec![Call { action: "remove", user_id: 10, role_id: 1 }]
        );
        assert!(outcome.matched_roles.is_empty());
    }

    // -- failure classification ---------------------------------------------------

    #[tokio::test]
    async fn skip_failures_are_counted_and_do_not_stop_the_user() {
        // 404 on one role (member left mid-scan / role deleted in Discord)
        // must not abort the user's remaining roles — parity with the legacy
        // handlers' NOT_FOUND tolerance.
        let cfg = config(vec![
            verify_role(1, r"@example\.com$"),
            verify_role(2, r"@example\.com$"),
        ]);
        let gw = MockRoles::default();
        gw.fail_next("add", 10, 1, MockFailure::Skip);

        let user = row(10, &["a@example.com"]);
        let outcome = run(&cfg, &ReconScope::all(), &user, &gw).await.unwrap();

        assert_eq!(outcome.errors, 1);
        // Role 1 failed (still counted as matched — the user qualifies),
        // role 2 succeeded.
        assert_eq!(outcome.matched_roles, vec![Id::new(1), Id::new(2)]);
        assert_eq!(gw.calls().len(), 2);
    }

    #[tokio::test]
    async fn rate_limit_interrupts_the_user_for_retry() {
        let cfg = config(vec![verify_role(1, r"@example\.com$")]);
        let gw = MockRoles::default();
        gw.fail_next("add", 10, 1, MockFailure::RateLimited(1500));

        let user = row(10, &["a@example.com"]);
        let result = run(&cfg, &ReconScope::all(), &user, &gw).await;

        match result {
            Err(WorkerInterrupt::RateLimited(d)) => {
                assert_eq!(d, Duration::from_millis(1500));
            }
            other => panic!("expected RateLimited, got {other:?}"),
        }

        // The retry succeeds (script consumed) — idempotent re-run.
        let outcome = run(&cfg, &ReconScope::all(), &user, &gw).await.unwrap();
        assert_eq!(outcome.matched_roles, vec![Id::new(1)]);
    }

    #[tokio::test]
    async fn fatal_failures_propagate() {
        let cfg = config(vec![verify_role(1, r"@example\.com$")]);
        let gw = MockRoles::default();
        gw.fail_next("add", 10, 1, MockFailure::Fatal);

        let user = row(10, &["a@example.com"]);
        let result = run(&cfg, &ReconScope::all(), &user, &gw).await;

        assert!(matches!(result, Err(WorkerInterrupt::Fatal(_))));
    }
}
