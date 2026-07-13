//! Integration tests for the verify reconciliation worker against a real
//! Postgres: the full job chain (lease → batched scan → checkpoints →
//! completion), supersede semantics, crash recovery via lease expiry,
//! rate-limit deferral, and the legacy-blob backfill.
//!
//! Run via `scripts/verify-recon-tests.sh`, or manually:
//! `KB2_TEST_DATABASE_URL=postgres://... cargo test -p consumer --test integration -- --ignored`
//!
//! Driving `run_job_slice` in a `while Continue` loop is exactly what the
//! SQS chain does in production (each `Continue` is a delayed continuation
//! token), so these tests exercise the real control flow minus the queue.

mod harness;

use std::time::Duration;

use common::verify::{
    GuildUserLink, JobStatus, ReconMessage, ReconScope, VerifyJob, guild_verify_from_db,
};
use consumer::verify::consumer::{SliceConfig, SliceOutcome, run_job_slice};
use harness::*;

fn fast_config() -> SliceConfig {
    SliceConfig {
        batch_size: 50,
        work_budget: Duration::from_secs(30),
        lease: Duration::from_secs(30),
        short_wait: Duration::from_millis(200),
        max_inline_wait: Duration::from_millis(500),
    }
}

/// Drives the job chain to a terminal outcome, mimicking the SQS loop.
/// Panics if the chain doesn't terminate within `max_slices` (a wedged
/// cursor would otherwise loop forever).
async fn drive_chain(
    msg: &ReconMessage,
    pool: &sqlx::Pool<sqlx::Postgres>,
    gateway: &StubGateway,
    cfg: &SliceConfig,
    max_slices: usize,
) -> (SliceOutcome, usize) {
    let mut slices = 0;
    loop {
        slices += 1;
        assert!(slices <= max_slices, "job chain did not terminate");
        match run_job_slice(msg, pool, gateway, cfg).await.unwrap() {
            SliceOutcome::Continue { delay } => {
                // The queue would redeliver after `delay`; tests just wait it
                // out (scripted delays in these tests are tiny).
                tokio::time::sleep(delay).await;
            }
            outcome => return (outcome, slices),
        }
    }
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn full_recon_chain_completes_and_folds_member_counts() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_001;
    reset_guild(&pool, GUILD).await;

    // 500 users: 1..=300 match role 1, 301..=400 match role 2, rest match
    // nothing.
    seed_guild(
        &pool,
        GUILD,
        vec![
            verify_role(1, r"@example\.com$"),
            verify_role(2, r"@other\.com$"),
        ],
        None,
    )
    .await;
    seed_links(&pool, GUILD, 500, |user_id| {
        if user_id <= 300 {
            vec![active_link(&format!("u{user_id}@example.com"))]
        } else if user_id <= 400 {
            vec![active_link(&format!("u{user_id}@other.com"))]
        } else {
            vec![]
        }
    })
    .await;

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::all(), &pool)
        .await
        .unwrap();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };
    let gateway = StubGateway::default();

    let (outcome, _) = drive_chain(&msg, &pool, &gateway, &fast_config(), 50).await;

    assert_eq!(outcome, SliceOutcome::Completed);
    // Full sync parity with legacy post_recon: matchers added, non-matchers
    // stripped — per role.
    assert_eq!(gateway.adds(), 300 + 100);
    assert_eq!(gateway.removes(), 200 + 400);

    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Succeeded);
    assert_eq!(job.total, 500);
    assert_eq!(job.processed, 500);
    assert_eq!(job.errors, 0);

    // Completion folded the scan's counts into the guild's role members.
    let verify = guild_verify_from_db(gid(GUILD), &pool)
        .await
        .unwrap()
        .unwrap();
    let members: Vec<u32> = verify.roles.iter().map(|r| r.members).collect();
    assert_eq!(members, vec![300, 100]);
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn scoped_role_add_touches_only_matchers() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_002;
    reset_guild(&pool, GUILD).await;

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;
    seed_links(&pool, GUILD, 1000, |user_id| {
        if user_id % 10 == 0 {
            vec![active_link(&format!("u{user_id}@example.com"))]
        } else {
            vec![active_link(&format!("u{user_id}@elsewhere.com"))]
        }
    })
    .await;

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::role_add(rid(1)), &pool)
        .await
        .unwrap();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };
    let gateway = StubGateway::default();

    let (outcome, _) = drive_chain(&msg, &pool, &gateway, &fast_config(), 100).await;

    assert_eq!(outcome, SliceOutcome::Completed);
    // O(matchers), not O(members): 100 adds, ZERO removes for the other 900.
    assert_eq!(gateway.adds(), 100);
    assert_eq!(gateway.removes(), 0);

    let verify = guild_verify_from_db(gid(GUILD), &pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(verify.roles[0].members, 100);
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn stale_tokens_and_live_leases_are_not_acquired() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_003;
    reset_guild(&pool, GUILD).await;

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@x$")], None).await;
    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::all(), &pool)
        .await
        .unwrap();

    // First acquisition wins...
    let held = VerifyJob::acquire_lease(gid(GUILD), generation, Duration::from_secs(60), &pool)
        .await
        .unwrap();
    assert!(held.is_some());

    // ...a concurrent duplicate delivery of the same token loses...
    let dup = VerifyJob::acquire_lease(gid(GUILD), generation, Duration::from_secs(60), &pool)
        .await
        .unwrap();
    assert!(dup.is_none(), "second worker must not co-own the job");

    // ...and a stale token (older generation) never acquires, even though
    // the lease field would have allowed it.
    let stale = VerifyJob::acquire_lease(gid(GUILD), generation - 1, Duration::from_secs(60), &pool)
        .await
        .unwrap();
    assert!(stale.is_none(), "stale generation must be dropped");

    // A slice driven with the stale token reports NotAcquired (message
    // dropped as success — the newer chain owns the work).
    let outcome = run_job_slice(
        &ReconMessage {
            guild_id: gid(GUILD),
            generation: generation - 1,
        },
        &pool,
        &StubGateway::default(),
        &fast_config(),
    )
    .await
    .unwrap();
    assert_eq!(outcome, SliceOutcome::NotAcquired);
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn supersede_mid_chain_yields_to_the_new_generation() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_004;
    reset_guild(&pool, GUILD).await;

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;
    seed_links(&pool, GUILD, 300, |user_id| {
        vec![active_link(&format!("u{user_id}@example.com"))]
    })
    .await;

    let gen1 = VerifyJob::supersede(gid(GUILD), ReconScope::role_add(rid(1)), &pool)
        .await
        .unwrap();

    // Slow gateway + tiny budget: the first slice checkpoints partway.
    let slow = StubGateway::with_latency(Duration::from_millis(2));
    let cfg = SliceConfig {
        work_budget: Duration::from_millis(50),
        ..fast_config()
    };
    let msg1 = ReconMessage {
        guild_id: gid(GUILD),
        generation: gen1,
    };
    let outcome = run_job_slice(&msg1, &pool, &slow, &cfg).await.unwrap();
    assert!(matches!(outcome, SliceOutcome::Continue { .. }));
    let mid_job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert!(mid_job.processed > 0 && mid_job.processed < 300);

    // An admin edit lands mid-chain: generation bumps, scope merges, cursor
    // restarts.
    let gen2 = VerifyJob::supersede(gid(GUILD), ReconScope::role_sync(rid(1)), &pool)
        .await
        .unwrap();
    assert_eq!(gen2, gen1 + 1);

    // The old token is now worthless...
    let stale = run_job_slice(&msg1, &pool, &StubGateway::default(), &fast_config())
        .await
        .unwrap();
    assert_eq!(stale, SliceOutcome::NotAcquired);

    // ...and the new chain runs the merged scope (sync absorbed the add)
    // over ALL 300 users from a restarted cursor.
    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert!(job.cursor.is_none(), "supersede must restart the cursor");
    assert_eq!(job.scope.sync_role_ids, vec![rid(1)]);
    assert!(job.scope.add_role_ids.is_empty());

    let gateway = StubGateway::default();
    let msg2 = ReconMessage {
        guild_id: gid(GUILD),
        generation: gen2,
    };
    let (outcome, _) = drive_chain(&msg2, &pool, &gateway, &fast_config(), 50).await;
    assert_eq!(outcome, SliceOutcome::Completed);
    assert_eq!(gateway.adds(), 300);

    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Succeeded);
    assert_eq!(job.processed, 300);
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn crashed_invocation_recovers_after_lease_expiry() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_005;
    reset_guild(&pool, GUILD).await;

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;
    seed_links(&pool, GUILD, 100, |user_id| {
        vec![active_link(&format!("u{user_id}@example.com"))]
    })
    .await;

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::all(), &pool)
        .await
        .unwrap();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };

    // Simulate a crash: a worker acquires the lease and dies without
    // checkpointing or releasing (in production: OOM, timeout, ...).
    let crashed =
        VerifyJob::acquire_lease(gid(GUILD), generation, Duration::from_millis(300), &pool)
            .await
            .unwrap();
    assert!(crashed.is_some());

    // SQS redelivery arriving while the lease is live is dropped...
    let outcome = run_job_slice(&msg, &pool, &StubGateway::default(), &fast_config())
        .await
        .unwrap();
    assert_eq!(outcome, SliceOutcome::NotAcquired);

    // ...but once the lease expires (production: 240s, here 300ms), the next
    // redelivery resumes and finishes the job.
    tokio::time::sleep(Duration::from_millis(350)).await;
    let gateway = StubGateway::default();
    let (outcome, _) = drive_chain(&msg, &pool, &gateway, &fast_config(), 50).await;
    assert_eq!(outcome, SliceOutcome::Completed);
    assert_eq!(gateway.adds(), 100);
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn long_rate_limit_defers_with_the_429s_retry_after() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_006;
    reset_guild(&pool, GUILD).await;

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;
    seed_links(&pool, GUILD, 20, |user_id| {
        vec![active_link(&format!("u{user_id}@example.com"))]
    })
    .await;

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::all(), &pool)
        .await
        .unwrap();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };

    // A 429 with retry_after above short_wait must end the slice and carry
    // the wait out as the continuation's delay — never an in-process sleep.
    let gateway = StubGateway::default();
    gateway.rate_limit_next_call(Duration::from_millis(400));

    let outcome = run_job_slice(&msg, &pool, &gateway, &fast_config())
        .await
        .unwrap();
    assert_eq!(
        outcome,
        SliceOutcome::Continue {
            delay: Duration::from_millis(400)
        }
    );
    // Nothing processed: the very first call was limited, the cursor must
    // not have advanced past the un-reconciled user.
    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.processed, 0);

    // The chain then completes, retrying that user (20 adds total — nobody
    // was skipped or double-processed).
    let (outcome, _) = drive_chain(&msg, &pool, &gateway, &fast_config(), 50).await;
    assert_eq!(outcome, SliceOutcome::Completed);
    assert_eq!(gateway.adds(), 20);
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn short_rate_limit_is_absorbed_inline_within_the_slice() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_007;
    reset_guild(&pool, GUILD).await;

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;
    seed_links(&pool, GUILD, 20, |user_id| {
        vec![active_link(&format!("u{user_id}@example.com"))]
    })
    .await;

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::all(), &pool)
        .await
        .unwrap();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };

    // retry_after under short_wait: absorbed by an inline sleep, the slice
    // carries on and completes without an SQS round-trip.
    let gateway = StubGateway::default();
    gateway.rate_limit_next_call(Duration::from_millis(50));

    let outcome = run_job_slice(&msg, &pool, &gateway, &fast_config())
        .await
        .unwrap();
    assert_eq!(outcome, SliceOutcome::Completed);
    assert_eq!(gateway.adds(), 20);
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn legacy_blob_user_links_are_seeded_on_first_job() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_008;
    reset_guild(&pool, GUILD).await;

    // Pre-migration guild: links live INSIDE the verify blob; the
    // guild_user_links table has no rows for it.
    let legacy_links = (1..=50u64)
        .map(|user_id| {
            (
                twilight_model::id::Id::new(user_id),
                vec![active_link(&format!("u{user_id}@example.com"))],
            )
        })
        .collect();
    seed_guild(
        &pool,
        GUILD,
        vec![verify_role(1, r"@example\.com$")],
        Some(legacy_links),
    )
    .await;

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::all(), &pool)
        .await
        .unwrap();
    // The supersede counted 0 rows — the seed hadn't happened yet.
    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.total, 0);

    let gateway = StubGateway::default();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };
    let (outcome, _) = drive_chain(&msg, &pool, &gateway, &fast_config(), 50).await;

    assert_eq!(outcome, SliceOutcome::Completed);
    // The worker seeded the table, refreshed the total, and reconciled all
    // 50 legacy users.
    assert_eq!(
        GuildUserLink::count(gid(GUILD), &pool).await.unwrap(),
        50
    );
    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.total, 50);
    assert_eq!(job.processed, 50);
    assert_eq!(gateway.adds(), 50);
}

#[tokio::test]
#[ignore = "requires KB2_TEST_DATABASE_URL"]
async fn guild_with_no_links_completes_immediately() {
    let pool = test_pool().await;
    const GUILD: u64 = 910_009;
    reset_guild(&pool, GUILD).await;

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::role_add(rid(1)), &pool)
        .await
        .unwrap();
    let gateway = StubGateway::default();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };

    let outcome = run_job_slice(&msg, &pool, &gateway, &fast_config())
        .await
        .unwrap();

    // Small guilds see the async job complete within the first slice,
    // seconds after the API's 202.
    assert_eq!(outcome, SliceOutcome::Completed);
    assert_eq!(gateway.adds(), 0);
    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Succeeded);
}
