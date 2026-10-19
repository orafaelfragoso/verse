#!/usr/bin/env bash
# Profiles 5 alternating builds of serde_json and regex in each configuration
# with the nightly self-profiler: rustup toolchain install nightly -c
# rustc-codegen-cranelift-preview, and cargo install --git
# https://github.com/rust-lang/measureme summarize.
set -euo pipefail
export PATH=$HOME/.cargo/bin:$PATH
cd "$(dirname "$0")"
# Its own target dir, so the wipe below never touches the repo target/.
td=$PWD/target
prof=$td/prof
rm -rf "$prof" && mkdir -p "$prof"
for run in 1 2 3 4 5; do
  for cfg in dev-llvm release-llvm dev-cranelift release-cranelift; do
    profile=${cfg%%-*}; backend=${cfg#*-}
    flags="-Zself-profile=$prof/$cfg-$run"
    [[ $backend == cranelift ]] && flags="$flags -Zcodegen-backend=cranelift"
    rm -rf "$td/debug" "$td/release"
    RUSTFLAGS="$flags" cargo +nightly build -q -j1 --target-dir "$td" $([[ $profile == release ]] && echo --release)
  done
done
python3 -I aggregate.py "$prof"
