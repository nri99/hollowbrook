# Progress log

## Current
- Current task: P2-01 Occlusion fade
- Last completed: P1-05 (night + lamp)
- Known blockers: P0-03 physical input (needs user); P4 civic design (needs user decision)
- Next action: make buildings/trees between camera and player translucent.

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
