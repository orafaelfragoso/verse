#!/usr/bin/env bash
# Fast M0 gates. Fuzz budgets and benchmarks run separately, as in CI.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python3 -m unittest discover -s tools/tests -v
python3 tools/check-dependencies.py
python3 tools/spec-coverage.py
tools/docs/check.sh
