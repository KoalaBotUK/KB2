#!/usr/bin/env bash
# Integration + performance test environment for the verify reconciliation
# worker (see docs/low-level-architecture/verify-role-reconciliation.md §9).
#
# Provisions a disposable local Postgres 16 cluster, then runs the consumer
# crate's DB-backed test suites against it:
#
#   ./scripts/verify-recon-tests.sh              # integration suite
#   ./scripts/verify-recon-tests.sh perf         # 50k-member perf suite (release build)
#   ./scripts/verify-recon-tests.sh all          # both
#   ./scripts/verify-recon-tests.sh stop         # tear the cluster down
#
# Prefers a local postgres install (initdb/pg_ctl); falls back to Docker.
# The cluster is tuned for test throughput (fsync off) — never reuse it for
# real data. Override the connection entirely by exporting
# KB2_TEST_DATABASE_URL before running.
set -euo pipefail

cd "$(dirname "$0")/.."

PG_PORT="${KB2_TEST_PG_PORT:-5544}"
PG_BIN="${KB2_TEST_PG_BIN:-/usr/lib/postgresql/16/bin}"
PG_DIR="${KB2_TEST_PG_DIR:-/var/lib/postgresql/kb2-test-pg}"
DB_NAME=kb2_test
DB_USER=kb2
DOCKER_NAME=kb2-test-pg

maybe_su() {
  # postgres refuses to run as root; drop to the postgres user when needed.
  if [ "$(id -u)" = "0" ]; then su postgres -c "$1"; else bash -c "$1"; fi
}

start_local() {
  if [ ! -f "$PG_DIR/PG_VERSION" ]; then
    mkdir -p "$PG_DIR"
    [ "$(id -u)" = "0" ] && chown postgres:postgres "$PG_DIR"
    maybe_su "$PG_BIN/initdb -D $PG_DIR -U $DB_USER --auth=trust -E UTF8" >/dev/null
  fi
  if ! maybe_su "$PG_BIN/pg_ctl -D $PG_DIR status" >/dev/null 2>&1; then
    maybe_su "$PG_BIN/pg_ctl -D $PG_DIR -l $PG_DIR/log \
      -o '-p $PG_PORT -k $(dirname "$PG_DIR") -c listen_addresses=127.0.0.1 -c fsync=off -c synchronous_commit=off -c full_page_writes=off' start" >/dev/null
  fi
  psql -h 127.0.0.1 -p "$PG_PORT" -U "$DB_USER" -d postgres -tc \
    "SELECT 1 FROM pg_database WHERE datname='$DB_NAME'" | grep -q 1 ||
    psql -h 127.0.0.1 -p "$PG_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE $DB_NAME" >/dev/null
}

start_docker() {
  docker ps --format '{{.Names}}' | grep -qx "$DOCKER_NAME" || docker run -d --rm \
    --name "$DOCKER_NAME" -e POSTGRES_USER=$DB_USER -e POSTGRES_PASSWORD=$DB_USER \
    -e POSTGRES_DB=$DB_NAME -p "$PG_PORT:5432" postgres:16-alpine \
    -c fsync=off -c synchronous_commit=off -c full_page_writes=off >/dev/null
  until docker exec "$DOCKER_NAME" pg_isready -U $DB_USER >/dev/null 2>&1; do sleep 0.5; done
}

stop_env() {
  maybe_su "$PG_BIN/pg_ctl -D $PG_DIR stop -m fast" 2>/dev/null || true
  docker stop "$DOCKER_NAME" 2>/dev/null || true
  echo "test postgres stopped"
}

case "${1:-integration}" in
  stop) stop_env; exit 0 ;;
esac

if [ -z "${KB2_TEST_DATABASE_URL:-}" ]; then
  if [ -x "$PG_BIN/initdb" ]; then start_local; else start_docker; fi
  export KB2_TEST_DATABASE_URL="postgres://$DB_USER@127.0.0.1:$PG_PORT/$DB_NAME"
fi
echo "KB2_TEST_DATABASE_URL=$KB2_TEST_DATABASE_URL"

run_integration() {
  cargo test -p consumer --test integration -- --ignored --nocapture
}

run_perf() {
  # Release build: perf numbers from a debug binary are meaningless.
  # Single-threaded so the 50k scenarios don't fight over the pool/CPU.
  cargo test -p consumer --test perf --release -- --ignored --nocapture --test-threads=1
}

case "${1:-integration}" in
  integration) run_integration ;;
  perf)        run_perf ;;
  all)         run_integration && run_perf ;;
  *) echo "usage: $0 [integration|perf|all|stop]"; exit 2 ;;
esac
