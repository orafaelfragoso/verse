#!/usr/bin/env bash
# Standard library modules are top-level (`http`, not `std.http`).
# Prints every `std.` reference in docs/ and README.md and exits 1 if any remains.
set -euo pipefail
cd "$(dirname "$0")/../../.."
found=$(grep -rnE '\bstd\.' docs README.md || true)
[[ -n "$found" ]] && printf '%s\n' "$found"
count=$(printf '%s' "$found" | grep -c . || true)
echo "std. references: $count"
[[ "$count" -eq 0 ]]
