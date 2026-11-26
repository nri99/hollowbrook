#!/usr/bin/env bash
# Render any spot to reproduce a reported visual glitch, from the numbers in the F12 debug overlay.
# usage: bash tools/view.sh <Town|Frontier|Islet|Room:N> <x> <z> [time like 2026-11-26T19:30:00] [facing degrees, 180 = down]
#   e.g. bash tools/view.sh Frontier 15.98 20.32 2026-11-26T11:29:00
# Output: test_output/quickshot/view.png (+ autotest_phaseview.txt with the scene, position and target).
cd "$(dirname "$0")/.."
export HOLLOWBROOK_VIEW="$1 $2 $3 ${5:-180}"
bash tools/quickshot.sh view "${4:-2026-11-16T11:00:00}"
