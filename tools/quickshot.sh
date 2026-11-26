#!/usr/bin/env bash
# Quick visual check: incremental build, run one autotest phase, screenshots land in test_output/quickshot/.
# usage: bash tools/quickshot.sh <phase> [dev-time]     e.g. bash tools/quickshot.sh 47
# Phases are listed in ARCHITECTURE.md. Never runs in the repo root (the player save lives there).
cd "$(dirname "$0")/.."
cargo build -q --profile quick 2>&1 | grep -E "^error" -A6 && exit 1
d=test_output/quickshot; rm -rf "$d"; mkdir -p "$d"; cd "$d" || exit 1
HOLLOWBROOK_AUTOTEST=. HOLLOWBROOK_AUTOTEST_PHASE="$1" HOLLOWBROOK_DEV_TIME="${2:-2026-11-16T11:00:00}" timeout 240 ../../target/quick/hollowbrook.exe 2>&1 | grep -iE "panick" && exit 1
ls *.png
