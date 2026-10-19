#!/usr/bin/env bash
# Run every docs check and build the single-file spec without rendering.
# Exits 1 if any of them fails.
#   tools/docs/check.sh
set -uo pipefail
TOOLS="$(cd "$(dirname "$0")" && pwd)"
failed=0
run() {
  echo "== $1"
  "${@:2}" || { echo "FAILED: $1"; failed=$((failed + 1)); }
}
for f in "$TOOLS"/check/*.py; do run "$(basename "$f" .py)" python3 "$f"; done
run std-prefix "$TOOLS/check/std-prefix.sh"
run build env NO_RENDER=1 "$TOOLS/build.sh"
echo "docs check: $failed failed"
(( failed == 0 ))
