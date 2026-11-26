# Hollowbrook — notes for Claude

Code map: read `ARCHITECTURE.md` first (state ownership, the `activate` gotcha, how-tos, autotest phase index).
Fast loop for small changes: `bash tools/quick.sh [filter]` (tests, ~2-6 s) and `bash tools/quickshot.sh <phase>` (screenshots, ~25 s).
Visual regressions: `bash tools/shots.sh check` compares key screenshots with the saved baseline (`save` after an intended visual change you have looked at).
When fixing a bug, first add a failing test (or a `Game::check()` rule) that reproduces it, then fix it.
Before every commit run `bash tools/check.sh` (fmt, clippy correctness, all tests incl. the random-input stress test).

When resuming work, first read `DESIGN_SPEC.md`, `BACKLOG.md`, `PROGRESS.md`, `RELEASE_CHECKLIST.md` (and `MORNING_REPORT.md` if present).
Take the highest-priority unblocked backlog item, finish and verify it, then update `PROGRESS.md`, `BACKLOG.md`, `RELEASE_CHECKLIST.md`.

## Standing instructions (autonomous loop, set by the user)
- Goal: complete persistent 3D village-life game per DESIGN_SPEC plus 2-player local split-screen with two gamepads.
- Priority: crashes/saves/dupes/controls → input architecture + local multiplayer → verify started features → core
  progression → presentation → content + regression.
- One recurring task only; no nested loops, parallel editors, watchdogs. Record the exact next action before ending a turn.
- No deadline. Stop when only human/hardware verification remains (write HUMAN_VERIFICATION.md + FINAL_REPORT.md) or
  everything is complete/blocked (write FINAL_REPORT.md), then cancel the loop. Stop immediately if `STOP_AUTONOMY` exists.
- Three failed approaches to a blocker → record it and move on. Hardware, payment, permissions, destructive or
  consequential product decisions → record as blocked, never guess or claim success.
- Evidence types must be kept separate: code inspection / headless / executable action path / rendered screenshot / physical.

## Safety
- Work only inside this project and `test_output/`. Run autotests only in `test_output/`, never in the root (player save lives there).
- Back up saves before migrations. Never reset the town or remove possessions.
- Never send desktop-wide synthetic keyboard/mouse input. Never push/publish. No destructive git. No paid services.
- Mark invented numbers/rules as provisional in DESIGN_SPEC.md.

## Build
- `cargo build --release` · `cargo test --release` · `cargo run --release`.
- Dev env vars: `HOLLOWBROOK_DEV_TIME`, `HOLLOWBROOK_DEBUG` (F12 screenshots), `HOLLOWBROOK_CURVE`,
  `HOLLOWBROOK_AUTOTEST=<dir>` + `HOLLOWBROOK_AUTOTEST_PHASE=0|1|2|3` (scripted bot through the real input layer).
