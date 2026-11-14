# Progress log

## Current
- Current task: MP-02 two players in the live loop (input routing per player, NPC talk focus per player)
- Last completed: MP-01 profiles + runtime players (save v4)
- Known blockers: P0-03 physical input (needs user); P4 civic design (needs user decision)
- Next action: main loop updates every joined player with its own Inp (keyboard = player 0 for now), sim once per frame; NPC talk focus checks all players; then MP-03 split-screen viewports.

## Baseline verification (before this run)
- 19 headless tests pass. Executable bot run (2 launches) completed the full loop; screenshots inspected:
  outdoor, behind-house, horizon, conversation, shop, sitting, relaunch sold-out.
- Player legacy save found in project root; backed up to `save_backups/hollowbrook_save.txt.20261113-212320.bak`.

## Remaining provisional behaviour
See DESIGN_SPEC.md [PROV] tags (prices, hours 8–22, regrow 3 days, 4 shells, friendship +1/+3, letters ≥10, rain rule,
schedules, resume-outdoors policy, clock-rollback pause).

## Log
### 21:50 — P0-01, P0-02 done
- `save.rs`: `backup()` (timestamped copy into `save_backups/`, keep newest 5), `load_checked()` (unparseable JSON is renamed `.corrupt-<time>`, never overwritten). `main.rs::load_game` backs up the JSON on every load and the legacy file before migrating.
- Headless: `backups_rotate`, `corrupt_save_moved_aside` pass (21/21).
- Executable: copy of the real legacy save migrated in `test_output/mig` → backup created, JSON written, pockets (Peach x2) kept, one daily refresh.

### 22:05 — P1-01..P1-05 done (autotest phase 3, real executable, injected keys)
- 3A @11:20: bought shovel (500), dug dig site → Old Coin, refilled; cast rod → bite → Trout; net → Pill Bug; entered Mossy's house during home hours and talked.
- 3B @21:30: night town with lit windows; lamp placed (lamp given as logged FIXTURE, placement via inventory UI), toggled on.
- Rendered (inspected): p3_fish_bite, p3_dig_site, p3_resident_talk, p3_night_town, p3_lamp_on.
- Found + fixed: screenshots showed shadow areas pale because framebuffer alpha < 1; `screenshot()` now forces opaque alpha (screen output itself was unaffected). Re-inspected night shot: correct dark shadows.
### 22:25 — P2-01 done
- `render.rs`: buildings/trees between camera and player drawn after the player at 32% alpha (`R.alpha`).
- Regression found during verification: autotest bot got stuck on corners (bot steering, not game state). `main.rs::go` now lane-aligns before crossing tiles and has stuck recovery. Phase 1 loop passes again (chair bought, placed rotated).
- Rendered (inspected): test_output/p1/02_behind_house_occlusion.png — house translucent, player visible.

### 22:45 — P2-02, P2-03 done; loop instructions recorded
- Tool FX (splash ring, dirt puff, net swish) and trampling (running only, 1/25 per frame on a flower, cleared at 06:00). H: `trampled_flowers_recover_next_morning` (22/22). Tool FX rendered check still open (R15).
- New standing instructions saved in CLAUDE.md; RELEASE_CHECKLIST.md created; MP-01..05 added to BACKLOG.

### 23:05 — MP-01 done
- `World` = shared state + `p: Profile` (active) + `others`; runtime `Player` in `Game.me`/`Game.others`; `Game::activate(id)` swaps state (no copies). `join`/`leave` keep profiles. Daily refresh runs relationship reset for every profile, shared refresh once. Save v4; `save::upgrade` turns v3 JSON into profile 0.
- H: 28/28 incl. v3_json_upgrades_to_profiles, two_profiles_independent_and_persist, last_stock_and_pickup_only_once, different_scenes_and_independent_ui, rollover_with_players_in_different_scenes, leave_keeps_possessions.
- X: real v3 save upgraded in test_output/mig (backup written, Peach x2 kept). Phase-1 single-player loop still passes (chair bought/placed).
