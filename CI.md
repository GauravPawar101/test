# CI mirror for decision-engine

This repository is a **copy of [decision-engine](https://github.com/GauravPawar101/decision-engine)
with its CI attached**, used to run the heavy builds and the end-to-end suite before anything is
pushed to the fork.

Why a copy rather than a workflow that checks the fork out: Travis only builds the repository a
`.travis.yml` lives in, and the engine's build is heavy enough that it wants to live next to the
source it builds. So the source lives here, Travis builds it, and the fork is updated once this is
green.

## Lanes

| Where | What | Cost |
| --- | --- | --- |
| **GitHub Actions** (`verify`) | `cargo +nightly fmt --all --check`, `cargo check --all-targets` on **both** database tracks, `cargo test`, `tsc` over the Playwright suite | the primary heavy lane: 4 vCPU / 16 GB runners, free for a public repo |
| **GitHub Actions** (`light-checks`) | format + `tsc` + typos — no dependency build | ~30 s, fast signal on a PR |
| **GitLab** (`.gitlab-ci.yml`) | the same `.ci/heavy-checks.sh` steps on shared runners | **fallback only** — for when GitHub is unavailable |
| **Travis** (`rust`) | the same `.ci/heavy-checks.sh` steps, one per job step | 2 vCPU / 7.5 GB, which cannot link the lib test binary (OOM-killed) |
| **Travis** (`specs`) | boot Postgres + Redis, apply the schema, build the engine, start it, run `tests/api` with Playwright | very heavy; runs on `main`, opt in elsewhere |
| **Codespaces** (`.devcontainer`) | the same scripts, interactively | dev environment, not a gate |

Failover: GitHub Actions first, GitLab when GitHub is down (it mirrors this repository, so it
already holds whatever was last pushed here), Travis for the specs lane.

Both database tracks are compiled because the engine ships MySQL by default and Postgres behind
`--no-default-features --features postgres`; a change under `src/` has to hold under both.

## Running it

Push to a branch, or start a build from the Travis UI. To force a rebuild of an unchanged commit:

```bash
./run.sh                     # empty commit on the current branch, pushed
./run.sh specs               # same, with RUN_API_SPECS=true set for that build
```

`specs` is opt-in off `main` because it costs a full build plus a booted stack. Set it from
Travis → *More options* → *Env variables* → `RUN_API_SPECS=true`, or with `./run.sh specs`.

## Updating the mirror

```bash
./sync.sh                     # mirror origin/main of the fork onto this repo's main
./sync.sh fix/some-branch     # mirror a branch instead
```

`./sync.sh` replaces everything the fork owns and leaves this repository's own files — `.ci/`,
`.github/workflows/light-checks.yml`, `CI.md`, `run.sh`, `sync.sh` — alone.

## Promoting a green build to the fork

1. Travis is green on the mirror (both lanes, or the heavy lane plus the light lane on the PR).
2. `./sync.sh` back is not involved: the change already came *from* the fork. For work developed
   here, push the branch to the fork and open a PR.

The rule this repository exists to enforce: **the fork only receives code that has been through
Travis.**

## Layout

```
.ci/install-deps.sh    system packages + pinned toolchains
.ci/heavy-checks.sh   fmt, both cargo tracks, unit tests, tsc
.ci/api-specs.sh      datastores, schema, build, boot, playwright --project=api
.travis.yml           the heavy lanes
.github/workflows/    the light lane
sync.sh               mirror the fork's source in
run.sh                force a rebuild
```
