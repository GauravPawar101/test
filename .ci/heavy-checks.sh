#!/usr/bin/env bash
# The heavy compile lane: format, type check both database tracks, run the unit tests, and type
# check the Playwright suite.
#
# Both tracks matter — the engine ships MySQL as the default and Postgres behind
# `--no-default-features --features postgres`, and a change in src/ has to compile under both.
# `--all-targets` also compiles the `#[cfg(test)]` modules, so a unit test that does not type check
# fails here rather than in the test step.
set -euo pipefail

echo "==> format"
cargo +nightly fmt --all --check

echo "==> type check: postgres track"
cargo check --all-targets --no-default-features --features postgres

echo "==> unit tests: postgres track"
cargo test --no-default-features --features postgres

echo "==> type check: mysql track"
cargo check --all-targets --features release

echo "==> type check: Playwright suite"
npm ci
npm run typecheck
