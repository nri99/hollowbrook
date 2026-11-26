#!/usr/bin/env bash
# Player-visibility sweep: fails if the player is hidden behind anything at any walkable spot in Hollowbrook
# (incl. North Meadow and the woods path), Dustreach or the islet. Prints each bad spot; reproduce one with
#   bash tools/view.sh <scene> <x> <z>
cd "$(dirname "$0")/.."
cargo build -q --profile quick 2>&1 | grep -E "^error" -A6 && exit 1
d=test_output/occlusion; rm -rf "$d"; mkdir -p "$d"; cd "$d" || exit 1
HOLLOWBROOK_MARK_PLAYER=1 HOLLOWBROOK_AUTOTEST=. HOLLOWBROOK_AUTOTEST_PHASE=occl HOLLOWBROOK_DEV_TIME=2026-11-16T11:00:00 \
  timeout 600 ../../target/quick/hollowbrook.exe > run.log 2>&1
grep -h "OCCLUDED\|occlusion sweep\|panicked" run.log autotest_phaseocc*.txt 2>/dev/null | sort -u | tail -60
grep -q "occlusion sweep: 0 hidden" autotest_phaseocc*.txt run.log 2>/dev/null && echo "occlusion: OK" || { echo "occlusion: FAILED"; exit 1; }
