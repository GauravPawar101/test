#!/usr/bin/env bash
# The heavy compile lane.
#
#   ./.ci/heavy-checks.sh            all of it
#   ./.ci/heavy-checks.sh test-pg    just the unit tests
#
# The steps are individually addressable so CI can run them as separate steps: a failure then names
# itself instead of hiding behind a single exit code.
#
#   fmt         cargo +nightly fmt --all --check
#   check-pg    cargo check --all-targets, postgres track
#   test-pg     cargo test, postgres track
#   check-mysql cargo check --all-targets, mysql track
#   tsc         tsc --noEmit over the Playwright suite
#
# Both database tracks are compiled because the engine ships MySQL by default and Postgres behind
# --no-default-features --features postgres; a change under src/ has to hold under both.
# --all-targets also compiles the #[cfg(test)] modules, so a unit test that does not type check
# fails at `check-pg` rather than at `test-pg`.
set -euo pipefail

# Linking the lib test binary is the memory peak of this whole project: the crate is a ~700
# dependency graph, and with debuginfo the link was OOM-killed (SIGKILL) on a 7.5 GB runner. Debug
# info in the test profile costs nothing here — the assertions do not need line tables — and it is
# scoped to the test profile, so the dependency cache from `check-pg` is still reused.
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0

step_fmtr() { echo "==> $1"; }

run_fmt() {
  step_fmtr "format"
  cargo +nightly fmt --all --check
}

run_check_pg() {
  step_fmtr "type check: postgres track"
  cargo check --all-targets --no-default-features --features postgres
}

run_test_pg() {
  step_fmtr "unit tests: postgres track"
  # --lib: every unit test lives in the library, and skipping the bin test target avoids a second
  # link of the same code.
  cargo test --lib --no-default-features --features postgres
}

run_check_mysql() {
  step_fmtr "type check: mysql track"
  cargo check --all-targets --features release
}

run_tsc() {
  step_fmtr "type check: Playwright suite"
  npm ci
  npm run typecheck
}

steps_for() {
  case "$1" in
    all) echo "fmt check-pg test-pg check-mysql tsc" ;;
    "") echo "fmt check-pg test-pg check-mysql tsc" ;;
    fmt | check-pg | test-pg | check-mysql | tsc) echo "$1" ;;
    *)
      echo "unknown step: $1 (expected one of: fmt check-pg test-pg check-mysql tsc, or all)" >&2
      exit 2
      ;;
  esac
}

for step in $(steps_for "${1:-all}"); do
  "run_$step"
done
