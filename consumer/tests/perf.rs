//! Performance tests for the verify reconciliation worker at the design
//! target (50k linked members), against a real Postgres and an in-process
//! stub Discord API.
//!
//! Run via `scripts/verify-recon-tests.sh perf`, or manually:
//! `KB2_TEST_DATABASE_URL=postgres://... cargo test -p consumer --test perf --release -- --ignored --nocapture --test-threads=1`
//!
//! What is measured (and why):
//!
//! * **Scan floor** — a scoped role-add where nobody matches issues zero
//!   Discord calls, so the wall time is pure worker overhead: keyset
//!   pagination, JSON link parsing, regex matching, checkpoints. This is
//!   the fixed cost every job pays per 50k members.
//! * **Full-sync pipeline** — 100% matchers driven through the real
//!   `TwilightRoles` client over HTTP to a local stub (twilight's
//!   client-side ratelimiter disabled), measuring end-to-end call capacity:
//!   DB page → match → HTTP round-trip → checkpoint.
//!
//! Discord's real per-guild role bucket (~5–10 req/s) is orders of
//! magnitude below what these measure — the numbers exist to prove the
//! worker is never the bottleneck and to quantify its overhead against the
//! ~1.5–3 h rate-limit floor documented in the architecture docs.

mod harness;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use common::verify::{JobStatus, ReconMessage, ReconScope, VerifyJob};
use consumer::verify::consumer::{SliceConfig, SliceOutcome, run_job_slice};
use consumer::verify::discord::{DiscordRoles, TwilightRoles};
use harness::*;

const MEMBERS: u64 = 50_000;

fn perf_config() -> SliceConfig {
    // Production defaults, except the work budget: a 60s budget would let
    // the unthrottled scan finish in one slice, hiding checkpoint overhead.
    // 5s budgets force the chain through many slice handoffs, so the
    // measured figure INCLUDES lease/checkpoint/handoff costs.
    SliceConfig {
        work_budget: Duration::from_secs(5),
        ..SliceConfig::default()
    }
}

async fn drive_chain<G: DiscordRoles>(
    msg: &ReconMessage,
    pool: &sqlx::Pool<sqlx::Postgres>,
    gateway: &G,
    cfg: &SliceConfig,
) -> usize {
    let mut slices = 0;
    loop {
        slices += 1;
        assert!(slices < 100_000, "job chain did not terminate");
        match run_job_slice(msg, pool, gateway, cfg).await.unwrap() {
            SliceOutcome::Continue { delay } => tokio::time::sleep(delay).await,
            SliceOutcome::Completed => return slices,
            other => panic!("unexpected outcome {other:?}"),
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires KB2_TEST_DATABASE_URL; run with --release"]
async fn perf_scan_floor_50k_members_no_matchers() {
    let pool = test_pool().await;
    const GUILD: u64 = 920_001;
    reset_guild(&pool, GUILD).await;

    let seed_started = Instant::now();
    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;
    seed_links(&pool, GUILD, MEMBERS, |user_id| {
        vec![active_link(&format!("u{user_id}@elsewhere.com"))]
    })
    .await;
    let seed_elapsed = seed_started.elapsed();

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::role_add(rid(1)), &pool)
        .await
        .unwrap();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };
    let gateway = StubGateway::default();

    let scan_started = Instant::now();
    let slices = drive_chain(&msg, &pool, &gateway, &perf_config()).await;
    let elapsed = scan_started.elapsed();

    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Succeeded);
    assert_eq!(job.processed as u64, MEMBERS);
    assert_eq!(gateway.adds() + gateway.removes(), 0);

    let rate = MEMBERS as f64 / elapsed.as_secs_f64();
    println!("== perf: scan floor (50k members, 0% matchers, 0 Discord calls) ==");
    println!("seed:       {seed_elapsed:?} for {MEMBERS} rows");
    println!("scan:       {elapsed:?} across {slices} slices");
    println!("throughput: {rate:.0} members/s (pure DB+match+checkpoint overhead)");

    // Regression guard, deliberately loose (CI hardware varies): the scan
    // floor for 50k members must stay in seconds, not minutes.
    assert!(
        elapsed < Duration::from_secs(120),
        "50k no-op scan took {elapsed:?} — worker overhead has regressed"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires KB2_TEST_DATABASE_URL; run with --release"]
async fn perf_full_sync_50k_members_via_http_stub() {
    let pool = test_pool().await;
    const GUILD: u64 = 920_002;
    reset_guild(&pool, GUILD).await;

    // In-process stub Discord API: every route answers 204 No Content —
    // exactly what PUT/DELETE member-role return on success.
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_handler = hits.clone();
    let app = axum::Router::new().fallback(move || {
        let hits = hits_handler.clone();
        async move {
            hits.fetch_add(1, Ordering::Relaxed);
            http::StatusCode::NO_CONTENT
        }
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // The REAL production gateway (TwilightRoles), pointed at the stub via
    // twilight's proxy support. The client-side ratelimiter is disabled so
    // the measurement isn't throttled to Discord's schedule — the point is
    // the pipeline's capacity, not Discord's.
    let client = twilight_http::Client::builder()
        .token("perf-test-token".to_string())
        .proxy(addr.to_string(), true)
        .ratelimiter(None)
        .build();
    let gateway = TwilightRoles::new(Arc::new(client));

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;
    seed_links(&pool, GUILD, MEMBERS, |user_id| {
        vec![active_link(&format!("u{user_id}@example.com"))]
    })
    .await;

    let generation = VerifyJob::supersede(gid(GUILD), ReconScope::role_add(rid(1)), &pool)
        .await
        .unwrap();
    let msg = ReconMessage {
        guild_id: gid(GUILD),
        generation,
    };

    let started = Instant::now();
    let slices = drive_chain(&msg, &pool, &gateway, &perf_config()).await;
    let elapsed = started.elapsed();

    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Succeeded);
    assert_eq!(job.processed as u64, MEMBERS);
    assert_eq!(hits.load(Ordering::Relaxed) as u64, MEMBERS);

    let rate = MEMBERS as f64 / elapsed.as_secs_f64();
    // Discord's per-guild role-modify bucket sustains roughly 5-10 req/s;
    // use the midpoint for the projection.
    let discord_rps = 7.5;
    let projected = MEMBERS as f64 / discord_rps / 60.0;
    println!("== perf: full sync (50k members, 100% matchers, HTTP round-trip per member) ==");
    println!("wall:       {elapsed:?} across {slices} slices");
    println!("throughput: {rate:.0} role calls/s end-to-end (DB page -> match -> HTTP -> checkpoint)");
    println!(
        "headroom:   {:.0}x Discord's ~{discord_rps} req/s per-guild bucket",
        rate / discord_rps
    );
    println!(
        "projection: at Discord pace the same job takes ~{projected:.0} min; worker overhead adds ~{:.2}%",
        (elapsed.as_secs_f64() / (MEMBERS as f64 / discord_rps)) * 100.0
    );

    assert!(
        rate > discord_rps * 10.0,
        "pipeline throughput ({rate:.0}/s) should exceed Discord's bucket by >=10x — the \
         rate limit, not the worker, must be the bottleneck"
    );
}

/// Sanity check that the counts accumulated across many slices match the
/// seeded distribution exactly at 50k scale (10% matchers), and that
/// progress is monotonically consistent: processed == total at completion.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires KB2_TEST_DATABASE_URL; run with --release"]
async fn perf_50k_partial_match_counts_stay_exact() {
    let pool = test_pool().await;
    const GUILD: u64 = 920_003;
    reset_guild(&pool, GUILD).await;

    seed_guild(&pool, GUILD, vec![verify_role(1, r"@example\.com$")], None).await;
    seed_links(&pool, GUILD, MEMBERS, |user_id| {
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

    let started = Instant::now();
    drive_chain(&msg, &pool, &gateway, &perf_config()).await;
    let elapsed = started.elapsed();

    assert_eq!(gateway.adds() as u64, MEMBERS / 10);
    let job = VerifyJob::from_db(gid(GUILD), &pool).await.unwrap().unwrap();
    assert_eq!(job.processed as u64, MEMBERS);
    assert_eq!(job.errors, 0);
    assert_eq!(
        job.counts.get(&rid(1)).copied(),
        Some((MEMBERS / 10) as u32)
    );

    let verify = common::verify::guild_verify_from_db(gid(GUILD), &pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(verify.roles[0].members as u64, MEMBERS / 10);

    println!("== perf: partial match (50k members, 10% matchers) ==");
    println!(
        "wall: {elapsed:?}; counts exact: {} matched",
        MEMBERS / 10
    );
}
