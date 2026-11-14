# Hollowbrook — notes for Claude

When resuming work, first read `DESIGN_SPEC.md`, `BACKLOG.md` and `PROGRESS.md` (and `MORNING_REPORT.md` if present).
Take the highest-priority unblocked backlog item, finish and verify it, then update `PROGRESS.md` and `BACKLOG.md`.

- Build: `cargo build --release` · Tests: `cargo test --release` · Run: `cargo run --release`.
- Dev env vars: `HOLLOWBROOK_DEV_TIME`, `HOLLOWBROOK_DEBUG` (F12 screenshots), `HOLLOWBROOK_CURVE`,
  `HOLLOWBROOK_AUTOTEST=<dir>` + `HOLLOWBROOK_AUTOTEST_PHASE=1|2|3` (scripted bot through the real input layer).
- Run autotests only inside a disposable folder (e.g. `test_output/`), never in the project root: the root holds the player's save.
- Never send desktop-wide synthetic keyboard/mouse input. Never push. Back up saves before migrations.
- Keep the game original: no names, art, music or signature systems from existing commercial games.
- Mark invented numbers/rules as provisional in DESIGN_SPEC.md.
