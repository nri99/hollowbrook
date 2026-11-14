# Progress log

## Current
- Current task: X-03 islet getaway
- Last completed: X-01/X-02 community projects + liveliness
- Known blockers: physical keyboard/controller verification (HUMAN_VERIFICATION.md); recreation-sensitive systems need an original design approved by the user (BACKLOG "Blocked")
- Next action: add a dock on the beach (tile + boat model), Ui confirm with fare, Scene::Islet (own small map rendered outdoors without town), islet critters + Sea Glass from a daily activity, islet stall, return trip; tests for per-player travel and item safety.

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

### 01:20 — P5-01 done (home expansion)
- `Game::rdims(b)` sizes homes by owner house level: 4x4 -> 5x5 -> 6x5; costs 3000 / 15000 [PROV]; bought from the last line of the shop counter menu; furniture on the new doorway tile moves to home storage; per profile (P2 home unaffected). No debt/loan (loans remain blocked, see BACKLOG).
- H: 36/36 (new: home_expansion_paid_upfront_keeps_furniture). X: phase 7 (coins set as logged FIXTURE, purchase via counter menu) -> level 1, room 5x5.
- S (inspected): p51_bigger_home (5x5 room, camera reframed).

### 01:40 — P6-01 done (mailbox)
- `Letter{from,text,coins,item,read,claimed}`, `Profile.mail` (serde default). Daily refresh writes resident letters into each profile mail (also for the inactive player) instead of paying instantly; mail capped at 20 (oldest read/claimed removed). Solid mailbox beside each home door (flag up when unread); only the owner can open; Ui::Mail list with keyboard/pad prompts; gifts claimed once.
- H: 37/37 (new: mail_delivered_per_profile_claim_once). X: phase 8 (letter inserted as logged FIXTURE) opened via mailbox: 1000 -> 1300 coins, second open no change.
- S (inspected): p61_mailbox_flag (both mailboxes, prompt "Check mailbox (1 new)").

### 02:00 — P7-01, P6-02 done (weather)
- `weather(day, month)`: wet days (provisional 1 in 8, deterministic) are Snow in Dec-Feb, otherwise Rain. Snowflakes per split view outdoors, HUD "Snow", resident snow lines (SNOW). Residents outdoors carry an umbrella in their clothing colour when it rains. Idle activities beyond pauses: not implemented.
- H: 38/38 (new: weather_rule_snow_in_winter). X/S: phase 9 on 2026-12-12 (snow) and 2026-11-14 (rain); inspected p7snow/p7_weather.png (snowy ground + flakes) and p7rain/p7_weather.png (Juniper with umbrella).

### 02:20 — P5-03 done; R15 rechecked
- `Profile.catalog` (serde default; seeded on load from owned furniture/clothing for every profile); `Game::order` at full price, delivered to pockets [PROV]; counter menu line "Order from catalog" -> Ui::Catalog.
- H: 39/39 (new: catalog_reorder_full_price). X: phase 10 bought Potted Plant at a stand, re-ordered it from the catalog (500 -> 0, two distinct uids). S (inspected): p53_catalog.
- R15 tool FX: re-inspected p3m/p3_dig_site.png: soil spheres around the hole, but the dirt-puff effect is not distinguishable from the hole mound; recorded PARTIAL.

### 02:40 — P6-03 done (gifts)
- During a resident conversation G (pad LB) opens Ui::Gift; one unit leaves pockets; liked kind (LIKES per personality [PROV]: fruit/fossil/fish/bug) +2 friendship with happy jump + hearts, other items +1; once per resident per day per profile (`NpcS.gifted`, serde default, reset 06:00).
- H: 40/40 (new: gifts_once_per_day_with_preferences). X: phase 11 shook a tree, picked a Peach, talked to Juniper, G -> Enter: friendship 0 -> 3 (1 chat + 2 liked gift). S (inspected): p63_gift_reaction. Fixed "Peachs" plural in the tree message seen in that screenshot.

### 03:00 — P7-02 done; R16 table written
- Harvest Fair [PROV]: 13-16 Nov, stall at plaza tile (11,10) solid only during the fair, lantern posts, residents mention it, one random furniture prize per player per year (`Profile.fair_year`, serde default; pockets-full keeps it unclaimed).
- H: 41/41 (new: harvest_fair_window_and_prize_once). X: phase 12 at 16:30 on 14 Nov: prize Lamp, second claim refused. S (inspected): p72_fair.
- DESIGN_SPEC: content coverage table with COMPLETE/PARTIAL/NOT IMPLEMENTED/BLOCKED and missing data; BACKLOG C-01..C-05 added from it.

### 03:20 — C-01 done (swimming + diving)
- [PROV] Walking from sand into SEA tiles starts swimming (player-only passability; residents never enter water); swim speed 2.0, no running/tools; Space/X dives at shared bubble spots (max 3 near players) holding Sea Star / Sea Urchin / Sea Slug / Octopus (new K::Sea, sellable, donatable, shown in the museum tank, journal column "Fish & sea"). Swimmers drawn waist-deep with a ripple ring (both players).
- H: 42/42 (new: swimming_and_diving). X: phase 13 walked off the beach, swam to a bubble, dived -> Sea Slug. S (inspected): c01_swimming.

### 03:40 — C-02 done (home visits)
- [PROV] While a player is in their own home, a resident who is Out with friendship >= 5 for that player and not yet visited today may knock (~1 in 600 frames of eligible time); stands by the door for 120 s or until nobody is home; `NpcS.visited` per profile (serde default, reset 06:00). VISIT dialogue lines; gifts/talk work as usual.
- H: 43/43 (new: resident_visits_player_home). X: phase 14 (friendship 5 set as logged FIXTURE) at home -> Juniper visited, talked. S (inspected): c02_visit. Fixed a "{f}s" plural in Mossy visit line.

### 03:55 — R15 recheck
- Phase 15 captures FX right after Space. Inspected fx_cast_splash.png: white splash ring around the bobber clearly visible. fx_net_swish.png: swish dots visible but small. Dig puff remains indistinct (PARTIAL).

### 04:10 — R15 done; pine occlusion
- Dig puff recoloured (light tan, larger, rises higher). Inspected p3m/p3_dig_site.png: puff clearly distinct over the dark hole. R15 complete.
- First rerun showed a pine fully hiding the player/hole (pines were not part of the occlusion fade). Pines between camera and player are now drawn translucent like trees (code + build only; not yet seen in a screenshot).

### 04:30 — C-03a done (roster indirection)
- `CHARS` (6 characters: 4 current + Clover the sheep, Basil the hedgehog, not yet in town), `HOUSES` slot -> building, `World.roster` slot -> character (serde default [0,1,2,3]), `Game::res(slot)`; `where_` takes a `&Res`; shopkeeper/curator draw ids moved to 10/11; head models for sheep (4) and hedgehog (5) added (not yet rendered in a screenshot).
- H: 44/44 (new: roster_defaults_and_mapping). X/S: phase 9 rerun -> p7rain/p7_weather.png unchanged (Juniper fox with umbrella).

### 04:50 — C-03b done (moving)
- [PROV] `World.arrived/moving` per slot (serde default; older saves start the 14-day clock at load). Daily: resident present >= 14 days, friendship < 3 with every player, no open request -> letter to every profile + dialogue line; X (pad Y) in conversation asks them to stay (cancels, restarts clock, +1 friendship). After 2 days a reserve character (Clover, Basil) takes the slot; friendship for that slot resets in every profile; house labels/door prompts follow the roster (`Game::bname`). Post letter announces the change.
- H: 46/46 (new: residents_move_out_and_in_with_stay_option, plurals). X: phase 16 (roster set as logged FIXTURE) talked to Clover and Basil. S (inspected): c03_Clover.png (sheep head + "Clover's House" label); c03_Basil.png (hedgehog mostly behind the dialogue box: spikes visible only -> partial).
- Fixed plural helper (cherry -> cherries) seen in the Clover screenshot.

### 05:10 — C-04 done (customization)
- [PROV] Counter "Customize": re-tint a furniture item in pockets (original/warm/cool/dark, 200c) or cycle home wallpaper/flooring palettes (4 each, 300c). Tints are stored per profile by item uid (`Profile.tints`, serde default), so they follow the item between pockets and room without touching item records; `Profile.wall/floor` drive the owner home room colours.
- H: 47/47 (new: customize_tint_follows_item_and_decor). X: phase 17 (coins logged FIXTURE): bought Chair, tinted warm, wallpaper 1, floor 1 via counter menu, placed at home. S (inspected): c04_custom_home (blue wallpaper, grey floor, warm chair).

### 05:30 — C-05 done (hats + hair)
- [PROV] 3 hats (Straw Hat, Beanie, Flower Crown) on the rack, worn via pockets (swap like shirts); hairstyle cycles 4 looks for 150c at the rack (odd styles add a bun). `Profile.hat/hair` (serde default).
- H: 48/48 (new: hats_and_hair). X: phase 18: P1 bought + wore Straw Hat, hairstyle x2 (P2 positioned beside P1 for the comparison shot = test positioning). S (inspected): c05_looks (P1 straw hat + blonde hair vs P2 default, both halves).

### 05:50 — Regression pass + stop
- H 48/48; X phases 1, 2, 4 re-run (single-player loop + relaunch persistence, two-player independence) passed; S regress/12_relaunch_home inspected.
- Remaining work is blocked (user decisions), unspecified content, or physical verification -> FINAL_REPORT.md written, loop cancelled.

## Expansion run (user-approved five directions)
### X-01 + X-02 done (community projects, liveliness, reactions)
- Materials: Branch (1 per tree per day when shaken), Stone (1 per rock per day with the shovel), plus any fruit/shell. One resident-proposed project at a time (bench -> lamp post -> planter -> bridge), validated sites that never block a building route; translucent preview + sign with progress bar/label; both players contribute at the sign (capped at what is still needed, transaction log per player); completion builds a usable object (bench: sit outdoors; lamp: lights + glow at night; planter; bridge: walkable river crossing), residents outdoors react (happy + hearts), dialogue mentions it for 2 days, proposer asks about progress. Liveliness 0-100 on the notice board (plaza) and in the journal.
- H: 50/50 (new: community_project_two_players_build_bench, bridge_project_becomes_walkable_and_materials_once_per_day; two older tests adjusted: P1 now receives the proposal letter; v3-upgrade test compares only v3-era fields).
- X: phase 19 bought shovel, shook trees -> 5 Branch, hit rocks -> 2 Stone, contributed at the sign -> bench built, sat on it.
- S (inspected): x01_site_before (ghost bench, sign "bench 0/7", notice board), x01_bench_built (player seated on the bench, hearts).
- Next: X-03 islet getaway.
