# test — CI harness for decision-engine

This repo holds no source code. It runs
[decision-engine](https://github.com/GauravPawar101/decision-engine)'s own checks on
GitHub's free runners, so the engine repo does not need throwaway CI and any ref can be
verified from here.

## Run it

```bash
./run.sh main                                          # verify the engine's main
./run.sh fix/rule-based-gateway-score                  # verify a branch
./run.sh main --api-specs                              # ... and run the Playwright API specs
```

or from the Actions tab: **Actions → decision-engine-verify → Run workflow**.

The script prints the run URL and exits non-zero if the run fails.

## What runs

| Step | Command | Why |
| --- | --- | --- |
| Format | `cargo +nightly fmt --all --check` | the engine formats with nightly rustfmt |
| Type check (postgres) | `cargo check --all-targets --no-default-features --features postgres` | the default local/dev track; `--all-targets` also compiles the unit tests |
| Type check (mysql) | `cargo check --all-targets --features release` | the default production track; the engine's CI checks both |
| Unit tests | `cargo test --no-default-features --features postgres` | runs the crate's `#[test]`s |
| Spec type check | `npm ci && npm run typecheck` | `tsc --noEmit` over the Playwright suite |
| API specs (opt-in) | `npx playwright test --project=api` | boots Postgres + Redis, migrates, builds the engine, runs `tests/api` |

The API specs are opt-in because they cost ~25 minutes: they need a database, a build and a
booted server. Add `--api-specs` (or tick the box in the workflow form) to include them.

## Cost

Free on a public repository: GitHub-hosted `ubuntu-latest` runners, first-party actions
(`actions/checkout`, `actions/setup-node`, `actions/upload-artifact`), and the Actions cache.
Third-party actions are deliberately avoided; `dtolnay/rust-toolchain` and
`Swatinem/rust-cache` are the two exceptions, both MIT and both already used by the engine's
own CI.

Caching is keyed on the ref, so verifying a new branch pays for the dependency build once and
reuses it after that (restored cache misses fall back to a full build).

## Layout

```
.github/workflows/decision-engine-verify.yml   the workflow
run.sh                                         dispatch + wait + status
```
