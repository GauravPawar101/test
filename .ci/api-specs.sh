#!/usr/bin/env bash
# The end-to-end lane: boot Postgres + Redis, apply the schema, build the engine, start it, and run
# the API specs (tests/api) against it.
#
# The engine's own Playwright config would also boot the dashboard; the API specs never look at the
# UI, so PW_NO_WEBSERVER keeps it out and the engine is started here instead.
set -euo pipefail

PSQL_URL="postgresql://db_user:db_pass@localhost:5432/decision_engine_db"
API_URL="${API_URL:-http://localhost:8080}"

cleanup() {
  [ -n "${ENGINE_PID:-}" ] && kill "$ENGINE_PID" 2>/dev/null || true
}
trap cleanup EXIT

echo "==> start datastores"
# Postgres and Redis for the engine, plus the analytics stack: config/development.toml enables the
# ClickHouse and Kafka audit pipelines, and the engine runs a startup connectivity check against
# ClickHouse (src/bin/open_router.rs) that panics the process when it cannot reach it. The analytics
# specs under tests/api read ClickHouse as well, so this is the same stack the engine's own CI boots.
COMPOSE_PROFILES= docker compose --profile postgres-ghcr --profile analytics-clickhouse \
  up -d postgresql redis kafka kafka-init clickhouse mailpit
COMPOSE_PROFILES= docker compose --profile postgres-ghcr --profile analytics-clickhouse \
  up -d --wait postgresql redis kafka clickhouse mailpit

echo "==> apply schema"
# Same migrations diesel would run, applied with psql so the lane does not have to compile
# diesel_cli first. Directory names are diesel version numbers, so lexical order is run order.
for migration in migrations_pg/*/up.sql; do
  echo "    $migration"
  psql "$PSQL_URL" -v ON_ERROR_STOP=1 -q -f "$migration"
done

echo "==> seed global service configs"
psql "$PSQL_URL" -v ON_ERROR_STOP=1 -q <<'SQL'
INSERT INTO service_configuration (name, value)
SELECT v.name, v.value
FROM (VALUES
    ('ENABLE_MERCHANT_ON_VOLUME_DISTRIBUTION_FEATURE_SR_V3', '{"enableAll":true,"enableAllRollout":100}'),
    ('merchants_enabled_for_score_keys_unification',         '{"enableAll":true,"enableAllRollout":100}'),
    ('SR_V3_INPUT_CONFIG_DEFAULT',                           '{"defaultLatencyThreshold":90,"defaultBucketSize":125,"defaultHedgingPercent":5}')
) AS v(name, value)
WHERE NOT EXISTS (
    SELECT 1 FROM service_configuration sc WHERE sc.name = v.name
);
SQL

echo "==> build the engine"
# No debug info: this build exists to be booted and poked by the specs, and dropping debuginfo is
# what keeps a full codegen of the ~700-dependency graph inside a 7.5 GB runner (it is also what
# the compile lane does for the test link).
export CARGO_PROFILE_DEV_DEBUG=0
cargo build --no-default-features --features postgres

echo "==> start the engine"
# The super-admin roster is config-only (tests/fixtures/super-admin.ts is the single source of
# truth for the identity, and playwright.config.ts injects the same value into the server it starts).
# Without it the super-admin-view spec gets 403 on an authenticated request.
DECISION_ENGINE__USER_AUTH__SUPER_ADMIN_EMAILS="superadmin@example.com" \
DECISION_ENGINE__LOG__CONSOLE__LEVEL=WARN \
  ./target/debug/open_router > /tmp/open_router.log 2>&1 &
ENGINE_PID=$!

for _ in $(seq 1 90); do
  if curl -fsS "$API_URL/health" >/dev/null 2>&1; then
    echo "    engine healthy"
    break
  fi
  if ! kill -0 "$ENGINE_PID" 2>/dev/null; then
    echo "engine exited during boot" >&2
    tail -100 /tmp/open_router.log >&2
    exit 1
  fi
  sleep 2
done
curl -fsS "$API_URL/health" >/dev/null

echo "==> run the API specs"
CI=true PW_NO_WEBSERVER=1 npx playwright test --project=api
