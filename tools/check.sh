#!/usr/bin/env bash
# One command before every commit (humans, Claude, Codex): formatting, lint errors, all tests.
# usage: bash tools/check.sh      (from the repo root)
set -e
cd "$(dirname "$0")/.."
cargo fmt --check || { echo "Run: cargo fmt"; exit 1; }
cargo clippy --release -- -D clippy::correctness 2>&1 | grep -E "^error" && { echo "clippy found correctness errors"; exit 1; } || true
cargo test --release 2>&1 | tee /dev/stderr | grep -q "^test result: ok" || { echo "tests failed"; exit 1; }
echo "check: OK"
