#!/usr/bin/env bash
# Quick loop for tiny changes (seconds after the first run): incremental build + the tests.
# usage: bash tools/quick.sh [test-name-filter]     e.g. bash tools/quick.sh frontier
# Before committing still run: bash tools/check.sh   (fmt + clippy + the full release test suite)
cd "$(dirname "$0")/.."
start=$(date +%s)
out=$(cargo test -q --profile quick "$@" 2>&1); code=$?
echo "$out" | grep -E "^test result|panicked|^error|FAILED|^ *--> src" | grep -v "warning" | head -20
[ $code -eq 0 ] && echo "quick: OK ($(( $(date +%s) - start ))s)" || { echo "quick: FAILED"; exit 1; }
