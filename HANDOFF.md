# Hollowbrook handoff (for a new AI agent)

Read this, then `ARCHITECTURE.md` (code map). Do not read the whole repo.

## What it is
Original cozy 3D village-life game in Rust + macroquad 0.4. Worlds: Hollowbrook (town + North Meadow, islet
Gullrock), Dustreach (desert bounty-hunter outpost, woods door on the town's west edge), spaceflight to Glacia
(ice) and Verdance (jungle moon). 2 players: split-screen or LAN (host + 1 friend).

## Rules (from CLAUDE.md)
- Never touch `hollowbrook_save.json`, `hollowbrook_save.txt` or `save_backups/` in the repo root.
  Run the game and autotests only inside `test_output/`.
- Commits end with a blank line, then `Authored by nri99`, and nothing else. Push only when the user asks.
- Only commit if `bash tools/check.sh` exits 0. Gate on the exit code, never on grepped output.

## Workflow (fast)
- `bash tools/quick.sh [filter]`: tests in about 2-6 s.
- `bash tools/quickshot.sh <phase>`: screenshots for one autotest phase. Phases are listed in ARCHITECTURE.md.
- `bash tools/view.sh <Town|Frontier|Planet:1|Planet:2|Islet|Room:N> <x> <z> [time]`: render any spot.
  F12 in-game shows the scene and position to plug in here.
- `bash tools/glitchcheck.sh`: runs everything (tests, player-visibility sweep, screenshot comparison).
  If screenshots changed on purpose, review `test_output/shots_diff/` and then run `bash tools/shots.sh save`.

## Where things live
- `src/world.rs` + `src/world/*.rs`: all state and rules.
  - `w.p` and `me` are the ACTIVE player; `activate(id)` swaps players in.
  - `space.rs`: flight, pads, `ship_at`, cockpit.
  - `frontier.rs`: bounties, gear, armour, standoffs.
- `src/render.rs` + `src/render/*.rs`: drawing only.
  - `frontier.rs`: Dustreach, armour (`hunter_gear`, `armor_on`), ship model `ship()`.
  - `space.rs`: flight view, cockpit, planets, `draw_pads`.
- Tests are in `src/tests/*.rs` (115 pass).
  - `quality.rs` has the stress tests, the interaction sweep and the UI text-size budgets.

## Known open bugs (fix these first)
1. (Fixed) Armour is now drawn inside the body rig: `human_g(..., Gear)` in `render/characters.rs` puts each
   piece in its limb's transform, for you and every other player, in every pose. The helmet replaces the hat.
   It shows in Dustreach, on planets and in the cockpit. Guarded by `armour_stays_on_the_body_in_every_pose`.
   - (Fixed) Other players riding the hover-bike now draw the bike and sit at ride height (`0.34`), and their
     sand-hopper companion is shown in Dustreach (`other_player_shows_armour_bike_and_companion`).
2. (Fixed) Ship position vs pad: `draw_pads` now centres ships on every pad (`fz + fd / 2.`), `P_RAMP`/`P_SPAWN`/
   `ramp_of` match the ramp foot (`+ 0.6`), planet pad-0's path meets the ramp, and `planet_scene` clears props
   from all `P_PADS` (fixing mushrooms/spires on Player 2's pad). Guarded by
   `ship_ramp_aligns_with_ramp_of_on_every_pad_and_planet`.
3. The user reports more glitches they haven't pinned down. Ask for F12 screenshots and reproduce each with
   `tools/view.sh`.

## Checks to add next (put them in tools/glitchcheck.sh)
1. (Done, in `cargo test`) Armour placement test: every pose, the pieces stay on the body.
2. (Done, in `cargo test`) Ship landing alignment test: every pad and planet, the ship's ramp matches `ramp_of()`.
3. Menu sweep: open every menu and try every option for both players, then run `Game::check()`.
4. Two-player screenshot comparison: one player always shows the other's ship, armour and companion.
   (Note: after playtesting these fixes, refresh the screenshot baseline with `bash tools/shots.sh save`.)

## Limits to be honest about
- At most 2 players.
- LAN has never been tested between two real computers.
- Physical gamepads are not verified.

## Session notes (Claude, 2026-11-26)
- Original-feature direction agreed with the user (do NOT build Animal Crossing's signature systems: no money
  rock/money tree, balloon presents, wishing stars/star fragments, turnip "stalk market", town tune, cafe shift).
  Build these original versions instead, one at a time, each with a test, then run tools/check.sh:
  1. DONE: Geode rock. One real town rock per day (`World.geode_rock`, picked at the morning refresh) cracks
     open up to `GEODE_MAX` (3) times with the shovel, each giving a Geode Crystal (item 177, 150c). Test:
     `daily_geode_rock`. (Antigravity's money rock was reworked into this. Its money rock never spawned, because
     `rand_tile` only returns walkable tiles and rocks are not walkable.)
  2. TODO: Seed-pod drifts. On windy days fluffy pods float over town; net one for a rare flower seed.
  3. TODO: Night meteors. On clear nights a glowing crater appears; dig it next morning for a meteorite
     (museum donation).
  4. TODO: Weekly market. A travelling trader with daily-changing stock and prices (no turnip speculation).
  5. TODO: Residents' sayings. Best friends teach you an emote and leave handwritten notes in the mailbox.
  6. TODO: Town bell. Choose one of several original chimes for the Town Hall clock (no melody editor).
  7. TODO: Tea house. Run by a new resident, unlocked through community projects.
- Unrelated originality issue to fix: commit c9a8d0c uses show names "Beskar" (rename e.g. "Starsteel") and
  "Whistling Darts" (e.g. "Hornet Darts"). Text only, gameplay unchanged.
- Screenshot reference set is stale: 36 shots differ because of commits 7953224 and c9a8d0c. Review
  test_output/shots_diff/, then run `bash tools/shots.sh save`.
