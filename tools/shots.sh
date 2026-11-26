#!/usr/bin/env bash
# Visual regression check: screenshots of key scenarios compared against a saved known-good set.
#   bash tools/shots.sh save    -> capture and store as the baseline (after you have checked they look right)
#   bash tools/shots.sh check   -> capture again and report every screenshot that changed noticeably
# Scripted runs use fixed randomness, but animation timing still varies a little, so small differences are normal;
# anything flagged should be looked at (test_output/shots_diff/ has before|after|difference images).
cd "$(dirname "$0")/.."
mode=${1:-check}
PHASES="1 30 40 44 45 48 50 51"
cargo build -q --profile quick 2>&1 | grep -E "^error" -A6 && exit 1
out=test_output/shots_new; rm -rf "$out"; mkdir -p "$out"
for p in $PHASES; do
  ( d=$out/p$p; mkdir -p "$d"; cd "$d" && HOLLOWBROOK_AUTOTEST=. HOLLOWBROOK_AUTOTEST_PHASE=$p HOLLOWBROOK_DEV_TIME=2026-11-16T11:00:00 timeout 240 ../../../target/quick/hollowbrook.exe > run.log 2>&1 ) &
  [ $(jobs -r | wc -l) -ge 3 ] && wait -n
done
wait
grep -l "panicked" $out/p*/run.log && { echo "shots: a scenario crashed (see run.log above)"; exit 1; }
if [ "$mode" = save ]; then rm -rf test_output/shots_base; cp -r "$out" test_output/shots_base; echo "shots: baseline saved ($(ls $out/p*/*.png | wc -l) images)"; exit 0; fi
[ -d test_output/shots_base ] || { echo "shots: no baseline yet, run: bash tools/shots.sh save"; exit 1; }
python tools/shotdiff.py test_output/shots_base "$out" test_output/shots_diff
