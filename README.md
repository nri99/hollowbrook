# Hollowbrook

An original, cozy 3D village-life game written in Rust with [macroquad](https://github.com/not-fl3/macroquad).
Live in a small riverside town on a real-time clock: collect bugs, fish and fossils, decorate and expand your home,
befriend residents, fund town projects, visit the islet of Gullrock, meet travelling campers, and play with a friend in
local split-screen.

## Features
- Real-time days (new day at 6:00), seasons, weather, day/night lighting and hourly music.
- Bugs, fish, sea creatures, digging, fossils; a museum with exhibits and an art wing.
- Home with furniture placement, storage, recolouring and expansions paid off through an interest-free balance.
- Four residents with routines, requests, gifts, letters, birthdays and visits.
- Material projects proposed by residents and coin-funded public projects placed by the town leader.
- Town policies at the Town Hall, a Saturday art stall, furniture collections.
- Gullrock islet: daily task, Sea Glass stall and three timed co-op activities.
- Campground with four rotating visitors, their goods and favors.
- Initiatives: daily and weekly tasks that earn coupons for kiosk rewards.
- Two-player split-screen with keyboard and gamepads (each player has their own home, pockets, coins and tasks).

## Build and run
Requires a Rust toolchain (stable).

```
cargo run --release
```

Tests: `cargo test --release`

The window title screen and the in-game journal show the version and build id.

## Controls
| Action | Keyboard | Gamepad (Xbox names) |
|---|---|---|
| Move / run | WASD or arrows / hold Shift | Left stick or d-pad / hold LT or RT |
| Interact, confirm, next | E or Enter | A |
| Cancel, close | Esc | B |
| Use tool, dive | Space | X |
| Cycle tools | Tab | RB |
| Pockets | I | Y |
| Journal (collection, tasks, settings, controls, guide) | J, then Tab for pages | Menu, then RB for pages |
| Player 2 join / leave | F2 | Menu or A on a new controller |
| Switch a controller between Player 1 and 2 | | Hold View, press Menu |

## Saves
The game saves to `hollowbrook_save.json` in the folder it runs from: every 15 seconds, at each new day and when the
window closes. A backup is copied to `save_backups/` on every launch. Audio settings are stored in
`hollowbrook_settings.json`.

## Project documents
- `RELEASE_NOTES.md` - what is in the current release.
- `ARCHITECTURE.md` - code map: files, state ownership, frame loop, saving, how-tos, autotest phases.
- `docs/archive/` - finished release scope (`RELEASE_1.md`) and the earlier autonomous run report.
- `DESIGN_SPEC.md` - design and (provisional) balance values.
- `HUMAN_VERIFICATION.md` - checks that need a person (physical controllers, listening, playtesting).

All characters, art (procedural), music (procedural) and content are original to this project.
