#!/usr/bin/env bash
# Keeps target/ under a size cap and wipes it when the rustc version changes.
# Usage: tools/cargo-budget.sh <max-gib>
# Without rustc on PATH only the size check runs, and no stamp is read or written.
set -euo pipefail

if [[ $# -ne 1 || ! $1 =~ ^[0-9]+$ ]]; then
  echo "usage: ${0##*/} <max-gib>  (non-negative integer)" >&2
  exit 2
fi
cap_gib=$1
target=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/target
stamp=$target/.rustc-version

[[ -d $target ]] || exit 0

rustc_version=
if command -v rustc >/dev/null; then
  rustc_version=$(rustc --version)
fi

size_kib=$(du -sk "$target" | cut -f1)
reason=
if [[ -n $rustc_version && -f $stamp && $(<"$stamp") != "$rustc_version" ]]; then
  reason="toolchain changed from '$(<"$stamp")' to '$rustc_version'"
elif ((size_kib > cap_gib * 1024 * 1024)); then
  reason="size $((size_kib / 1024)) MiB exceeds cap $cap_gib GiB"
fi

if [[ -n $reason ]]; then
  echo "wiping $target: $reason"
  rm -rf "$target"
  mkdir "$target"
else
  echo "$target is $((size_kib / 1024)) MiB of $cap_gib GiB"
fi

# Stamp after a wipe, and on the first run for a target/ built before the stamp existed.
if [[ -n $rustc_version && ! -f $stamp ]]; then
  echo "$rustc_version" >"$stamp"
fi
