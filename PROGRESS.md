# Progress log

## Current
- Current task: P5-01 Home expansion (paid up front, per profile, no debt)
- Last completed: P5-02 clothing (rack, buy, wear, per-profile, persisted)
- Known blockers: physical keyboard/controller verification (HUMAN_VERIFICATION.md); recreation-sensitive systems need an original design approved by the user (BACKLOG "Blocked")
- Next action: counter menu option "Expand home" (cost table [PROV] 3000/15000); Profile.home level -> room_dims for Home(o) buildings (4x4, 5x5, 6x5) with furniture kept; place_ok uses owner room dims; tests; rendered check of larger room.

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

### 23:40 — MP-02, MP-03, MP-05 done
- `Game::tick` (shared, once per frame) + `Game::update_player` (per player after `activate`). Main loop routes keyboard to P1; F2 joins/leaves P2 (pads: MP-04). NPC talk focus and wildlife anchors consider all players.
- Split-screen: per-player Camera3D viewport + Camera2D HUD viewport; narrow HUD layout with P1/P2 tag. Other players drawn when in the same scene, distinct outfits.
- Second home building id 7 (Home(1)) on free grass; homes are owner-aware; only the owner can move/pick up furniture; objects saved on tiles that are now buildings are dropped on load.
- H: 29/29 (new: separate_homes_owner_only_furniture). X: autotest phase 4 (two virtual inputs): P1 entered home while P2 walked outside; P2 inventory opened, P1 Esc did not close it, P2 Esc did.
- S (inspected): mp_split_home_and_town, mp_p2_inventory, mp_same_scene, mp_together (both players visible in both halves).

### 00:10 — MP-04 done (virtual)
- `src/pad.rs`: gilrs polling; `Slots` (pad->player, P2 claimed first, disconnected slot reclaimable); `Mapper` (stick/d-pad -> arrow keys, dead zone 0.35, edges, menu repeat 0.4 s / 0.12 s; A=E+Enter, B=Esc, X=Space, Y=I+X, RB=Tab+R, LB=G, Select=F, Start=J, triggers=run). Main loop merges pad 0 with keyboard for P1; first claimed pad joins P2. Pad prompts + "controller disconnected" banner per player half.
- H: 32/32 (new: dead_zone_and_diagonal, button_edges_and_repeat, slots_assign_disconnect_reassign).
- X: phase 5 — virtual pad states through Mapper into P2: A entered own home (Room 7), Y opened pockets, B closed; slot disconnect -> banner, another pad reclaimed slot; P2 progress intact.
- S (inspected): mp_pad_prompts, mp_pad_disconnected (banner was clipped in narrow view -> shortened, rechecked prompts).
- Physical controller: NOT RUN (no hardware access by agent). Steps in HUMAN_VERIFICATION.md.

### 00:40 — P3-01..P3-03 done (museum)
- Building 8 "Museum" on free grass (ids unchanged); interior 7x5 with curator Professor Quill (heron), desk, fish tank, bug case, 4 fossil pedestals. `World.donated: Vec<(def, player)>` shared (serde default). Curator identifies Unknown Fossils (new dig-site drop) into Fern Fossil / Shark Tooth / Ammonite / Trilobite (identification result random [PROV]), then donates every species not yet in the museum; a species can be donated once across both players. Journal marks [museum]; exhibit interaction shows counts and donor.
- H: 34/34 (new: museum_shared_donation_once_and_fossil_id, museum_reachable_and_blocks; dig test updated for fossils).
- X: phase 3A extended: dug Unknown Fossil, caught Trout, netted Pill Bug, entered museum, curator identified Shark Tooth and took all three; pockets empty.
- S (inspected): p3_museum_exhibits (trout in tank, bug in case, shark tooth on pedestal, curator).

### 01:00 — P5-02 done (clothing)
- 4 original shirts (K::Cloth, prices [PROV]) on a shop rack (tile (1,1)); Ui::Rack menu (keyboard + pad prompts). Wearing from pockets swaps with the previously worn shirt in the same slot. `Profile.shirt` (serde default) drives the character shirt colour for each player.
- H: 35/35 (new: clothing_buy_wear_swap_persist). X: phase 6 bought Sunflower Tee at the rack (1000->600), wore it via pockets.
- S (inspected): p5_new_outfit (yellow shirt, rack with 4 shirts; also confirms shop painting display now renders on its stand).
