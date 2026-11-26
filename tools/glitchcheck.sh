#!/usr/bin/env bash
# Every glitch guard in one go (about 4 minutes). Run before committing anything visual.
#   1. tools/check.sh      formatting, lint errors, all tests (rules, saves, random-input stress, UI text sizes)
#   2. tools/occlusion.sh  the player is never hidden behind something solid, at any walkable spot in any world
#   3. tools/shots.sh      no screenshot changed unexpectedly compared with the saved known-good set
cd "$(dirname "$0")/.."
bash tools/check.sh > test_output/check.log 2>&1 && echo "check: OK" || { tail -20 test_output/check.log; echo "glitchcheck: FAILED (tests)"; exit 1; }
bash tools/occlusion.sh || { echo "glitchcheck: FAILED (player hidden)"; exit 1; }
bash tools/shots.sh check || { echo "glitchcheck: screenshots changed - review test_output/shots_diff/, then 'bash tools/shots.sh save' if intended"; exit 1; }
echo "glitchcheck: OK"
