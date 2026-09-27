#!/usr/bin/env bash
# Dispatch decision-engine-verify for a ref and wait for the result.
#
#   ./run.sh <ref> [--api-specs]
#
# Exits 0 when the run succeeds, 1 when it fails or is cancelled, 2 on a usage/lookup problem.
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-GauravPawar101/test}"
WORKFLOW="decision-engine-verify"
REF="${1:-main}"
shift || true

flags=()
for arg in "$@"; do
  case "$arg" in
    --api-specs) flags+=(-f "api-specs=true") ;;
    -h | --help)
      sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

command -v gh >/dev/null || {
  echo "gh CLI is required: https://cli.github.com" >&2
  exit 2
}
gh auth status >/dev/null 2>&1 || {
  echo "gh is not authenticated: run 'gh auth login'" >&2
  exit 2
}

# The workflow refuses to start until the workflow file exists on the default branch, so a first
# run may need a moment; retry the dispatch briefly rather than failing the whole script.
before="$(gh run list --repo "$REPO" --workflow "$WORKFLOW" --limit 100 --json databaseId 2>/dev/null || echo '[]')"

echo "Dispatching $WORKFLOW for $REF ${flags[*]:-}"
gh workflow run "$WORKFLOW" --repo "$REPO" -f "ref=$REF" ${flags[@]+"${flags[@]}"}

run_id=""
for _ in $(seq 1 30); do
  run_id="$(
    gh run list --repo "$REPO" --workflow "$WORKFLOW" --branch main \
      --event workflow_dispatch --limit 10 --json databaseId,status \
      | tr -d '[]" ' | tr ',' '\n' | grep -E '^[0-9]+$' | tail -1 || true
  )"
  [ -n "$run_id" ] && break
  sleep 2
done

if [ -z "$run_id" ]; then
  echo "could not find the dispatched run; check https://github.com/$REPO/actions" >&2
  exit 2
fi

url="https://github.com/$REPO/actions/runs/$run_id"
echo "Run: $url"

gh run watch "$run_id" --repo "$REPO" --exit-status || {
  echo "Run did not succeed: $url" >&2
  gh run view "$run_id" --repo "$REPO" --log-failed 2>/dev/null | tail -100 || true
  exit 1
}

echo "OK: $url"
