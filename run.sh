#!/usr/bin/env bash
# Kick a Travis build for this repository, optionally with the API-specs lane enabled.
#
#   ./run.sh          rebuild the current commit
#   ./run.sh specs    rebuild with RUN_API_SPECS=true for that build
#
# Travis is driven through a git push rather than its API, so no Travis token is needed: an empty
# commit is a new build. The script prints the build URL when Travis has published one.
set -euo pipefail

mode="${1:-default}"
specs_flag=""
if [ "$mode" = "specs" ]; then
  specs_flag=1
elif [ "$mode" != "default" ]; then
  echo "usage: $0 [default|specs]" >&2
  exit 2
fi

command -v git >/dev/null || {
  echo "git is required" >&2
  exit 2
}

# Travis reads build variables from the commit message trailer, so the opt-in travels with the
# commit rather than living in CI settings.
message="ci: rebuild"
[ -n "$specs_flag" ] && message="$message

RUN_API_SPECS=true"

git commit --allow-empty -q -m "$message"
git push

echo
echo "Pushed $(git rev-parse --short HEAD). Watch it at:"
echo "  https://app.travis-ci.com/github/GauravPawar101/test"
