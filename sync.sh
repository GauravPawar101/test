#!/usr/bin/env bash
# Re-sync this repository's copy of the engine source from the fork.
#
#   ./sync.sh                       # sync origin/main of the fork onto this repo's main
#   ./sync.sh fix/some-branch       # sync that branch instead
#
# The .ci/, .github/ and CI.md files belong to this repository and are preserved; everything else
# mirrors the fork, so a change in the fork lands here by running this and pushing.
set -euo pipefail

FORK="${ENGINE_FORK:-GauravPawar101/decision-engine}"
REF="${1:-main}"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

echo "==> fetching $FORK@$REF"
git clone --quiet --depth 1 --branch "$REF" "https://github.com/$FORK.git" "$work/engine"

# Everything the fork owns, minus build output and its own CI (this repo brings its own lanes).
find "$work/engine" -mindepth 1 -maxdepth 1 \
  ! -name '.git' ! -name 'target' ! -name 'node_modules' ! -name 'playwright-report' \
  ! -name 'test-results' ! -name '.github' -exec mv -t . {} +

# .github, minus the fork's own workflows, which would duplicate the lanes configured here.
mkdir -p .github
if [ -d "$work/engine/.github" ]; then
  find "$work/engine/.github" -mindepth 1 -maxdepth 1 ! -name 'workflows' -exec mv -t .github {} +
fi

echo "==> staged changes"
git status --short
echo
echo "Review, then: git commit -am 'sync $FORK@$REF' && git push"
