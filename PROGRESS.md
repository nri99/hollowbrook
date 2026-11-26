# Progress log

## Current
- Current task: X-04 collections
- Last completed: X-06 (partial) atmosphere
- Known blockers: physical keyboard/controller verification (HUMAN_VERIFICATION.md); recreation-sensitive systems need an original design approved by the user (BACKLOG "Blocked")
- Next action: X-04: two themed furniture sets (e.g. Meadow set: 3 items, Harbor set: 3 items) in shop stock pool + set-completion note; +4 critters with habitat/season (river mouth, rain-only bug, winter fish, islet bug); art wing in museum (procedural original paintings: 6, donated from a weekly art stall), museum labels (projected names over exhibits). Then X-06 remainder.

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

### X-03 done (Gullrock Islet)
- Scene::Islet with own map/terrain/palms/tide pool/dock, Captain Pell (town dock + islet), travel confirm + fare, daily alternating task, Sea Glass reward once per player per day, islet stall, islet-only Palm Crab and Glass Shrimp, per-player travel (the other player stays in town), save resumes at the dock.
- H: 51/51 (new: islet_travel_task_stall_and_safety). X: phase 20 two players: P1 sailed (1000->800), gathered 5 Driftwood, Pell paid 3 Sea Glass, sailed home; P2 stayed in town. S (inspected): x03_split_islet_town.
- Remaining: X-04 collections, X-05 social depth, X-06 atmosphere.

### X-05 done (social depth)
- Resident-to-resident chats with speech bubbles (residents sometimes walk toward a neighbour); birthdays (letters, dialogue, gift bonus); parcel-delivery requests between residents.
- H: 52/52 (new: parcels_birthdays_and_chats). X: phase 21 — first run no chat in 2 min (residents too spread out) -> residents now sometimes stroll toward a neighbour; rerun: chat started within ~13 s. S (inspected): x05_resident_chat (Juniper bubble beside Otto).
- Remaining: X-04 collections (themed furniture sets, more critters with habitat/season, art wing, museum labels), X-06 atmosphere (music, surface footsteps, seasonal particles, animation polish).

### X-06 partial (atmosphere)
- Procedural music (3 moods by hour), surface footsteps, seasonal particles, dialogue box fits split-screen views (wraps 2 lines), narrow debug panel moved to the top.
- H: 53/53 (new: footstep_surfaces). X: [audio] 3 music tracks, 12 sound effects loaded (log). Audio output NOT heard (no audio verification possible for the agent).
- S (inspected): x06/p7_weather.png (autumn leaves), x06/x06_split_dialogue.png (dialogue inside P1 half; "[E] next" hint not visible -> small open issue).
- Open: X-04 collections; X-06 remainder (animation/face polish, materials).

### Audio fix
- Root cause of silence: macroquad 0.4.16 has no default features; the `audio` feature (quad-snd) was never enabled, so all sounds used the silent dummy backend. Enabled `features = ["audio"]`. Evidence: quad-snd compiled; game process now loads AudioSes.dll/MMDevApi.dll (audio session opened). Audible output still to be confirmed by the user.

### Music overhaul
- `src/audio.rs`: 24 original hourly compositions (key walks the circle of fifths, mode/tempo/progression/swing/lead per hour; 16-bar A A' B A'' form, 30-45 s loops), shared palette (music box, FM keys, flute, pad, bass, soft shaker/tick), quieter sparse nights, Normal/Rain/Indoor arrangements, seamless wrapped loops, generation on a worker thread, 3 s crossfades, one track for both players (outdoor if any player is outdoors). Music/SFX volumes + mute saved in hollowbrook_settings.json (gitignored), set in the journal.
- H: 57/57 (new: hourly_melodies_are_distinct, loops_are_long_clean_and_unclipped, nights_quieter_and_indoor_muffled, settings_roundtrip_and_volume_math). Release build includes quad-snd (cargo tree).
- X: phase 23: 16:59:55 -> hour 16 loaded (37.3 s), 17:00 crossfade to hour 17 (44.7 s) over ~3 s, entering home -> Indoor arrangement crossfade, journal X mute -> applied 0.00, unmute + volume steps applied.
- NOT verified: how it sounds (agent cannot listen).

## Items 2/3/4/7 (user request, 2026-11-14)
- 2 Housing balance: DONE (7da590e).
- 3 Money-funded public construction: DONE. MPROJECTS table (data.rs), LEADER=player 0 picks at notice board, ghost preview green/red with route validation (mp_site_ok), donations per player in 1000c steps at the site sign (log saved), built at next 06:00 (finish_mproject). Material projects unchanged. Test money_projects_leader_funding_rollover.
- 4 Policies: DONE. Town Hall door opens the civic desk (Administrator Hazel Quill); POLICIES table in data.rs (fee, shop hours, wake/sleep shifts, weeds, exempt chars, effect text); leader-only, fee from own coins, effect + fee shown before confirm, repeal free; effects derived at runtime (Game::loc, shop_hours) so no stacking. Test policies_one_active_no_stacking. Town Hall interior not modelled (desk panel opens at the door).
- 7 Gullrock timed activities: DONE. Tidekeeper's board on the islet; Crab Corral (push), Beacon Dash (timed touch), Ring Swim (ordered swim course); HUD shows objective, time, team + own score, thresholds and rewards; rewards to each participant's own pockets once per activity per day. Test gullrock_activities_coop_and_separate; autotest phase 24 (real input: gold, +4 Sea Glass, P2 in town unaffected; screenshot act_beacon_dash inspected). Crab Corral and Ring Swim not yet driven by an autotest or seen rendered.

## Milestone verification (2026-11-14, after user review)
- Housing (7da590e) re-verified: test housing_migration_independent_balances_relaunch (pre-balance save with expanded homes loads with debt 0; per-player balances; repayment deducts only the payer; debt blocks next expansion; clearing unlocks; rooms/money/balances survive reload).
- Town Hall interior added (Hazel Quill, civic desk, project board, benches); civic actions moved inside; test town_hall_interior_and_civic_menu.
- Edge cases: test funding_and_activity_edge_cases (remaining <1000 capped, <1000 coins, same-frame donations no overpay, late join, one leaves other stays, full pockets -> mailbox, reload/restart no duplicate reward).
- Rendered (autotest phases 25/26/27, screenshots inspected): Town Hall interior + civic menu; invalid (red) and valid (green) placement; partial funding label; bench built after 06:00 rollover; Crab Corral co-op (both players scored, silver, 2 Sea Glass each); Ring Swim solo while P2 in town (gold). Beacon Dash solo/town earlier (phase 24).
- Visual fixes: debug panel off in screenshots, hint bar background, shorter notes for split view, distinct blue activity crabs with markers, stronger preview disc, lower civic desk.
- Beacon Dash thresholds reverted to 5/9/13 (earlier bump was bot-driven). Human difficulty assessment pending.

## Paused for user feedback (2026-11-14)
- Items 2/3/4/7 milestone accepted by user. Awaiting human checklist results.
- Unfinished: X-04 collections NOT STARTED; X-06 PARTIAL (animation/face polish, materials open); activity difficulty human assessment pending; physical controllers pending; split-screen "[E] next" dialogue hint not visible (known minor); leader fixed to P1.
- Next action: wait for user feedback before new work.

## 2026-11-14 continuation
- Item 1 DONE: split-screen dialogue "[A]/[E] next" now anchored inside each viewport's panel (rendered: phase 22 screenshot). HUMAN_VERIFICATION.md now has the controller mapping table taken from src/pad.rs.
- X-04 DONE: see DESIGN_SPEC. Test x04_collections_art_critters; autotest phase 28 (Saturday dev clock) screenshots inspected: art stall, art wing with donated painting, home with both collections.
- Next: X-06 remaining (animation, facial readability, materials).
- X-06 DONE: player blink + brows, resident blink + talking head nod/tilt, running dust puffs, lap siding + corner trim on all buildings; removed the leftover boarding plank from the (now open) Town Hall door. Autotest phase 29 screenshots inspected (siding/trim/open door; mid-run frame with dust). Blink is transient (0.13 s every ~4 s) and was not captured in a screenshot: code inspection only.
- Full suite: 64/64 pass.
- Remaining (human): physical controllers, activity difficulty, Windows HDMI audio (game produces sound at peak 0.33 on the TV endpoint per Core Audio meter; output path beyond Windows unverified).
- Next action: await user feedback; no open agent tasks.

## X-04/X-06 reconciliation (2026-11-14)
- Counts: no record of 6/8 found (BACKLOG AC had no counts; my handoff said 3/6). Delivered 6 per set, 8 paintings.
- Doorway lamp: screenshot fixture bypassed validation (pushed into room). Normal placement already rejects doorway, own tile and escape-route blocking; test placement_never_blocks_door_or_escape drives the real Enter path. Fixture now goes through place_ok (all 12 accepted; lamp at 4,4, visible).
- DEFECT FIXED: tools were disabled on Gullrock, so the Dune Beetle was uncatchable; the net now works there (other tools show a note).
- Critters: test new_critters_rules_capture_dex_donation (rain/season/hours/habitat pools, net capture on islet, encyclopedia, donation). Rendered: museum shows donated fish in tank, bugs in case (phase 28). Screenshots don't prove the availability rules; the tests do.
- Blink/nod: three fixed-time frames rendered (phase 28); curator blink + head tilt confirmed in crops (test_output/a28/x06_frames_compare.png). Player blink not visible in those frames (player faces away): rendering unverified for players.
- Art stall: Saturday && !fair (event restriction); per-player weekly limit survives reload (test art_week_limit_per_player_survives_reload).
- Full suite 67/67.

## 2026-11-14 resume (from 9737f16)
- No new playtest feedback found.
- Player blink rendered-verified (phase 28 frames at t=9.5 open / t=8.45 closed; crop test_output/a28/x06_player_blink_compare.png). Brows moved down/forward and darkened; now visible from the game camera.
- Backlog reconciled: home loans and P4-01 marked done (were stale [B]). Only open item: campground visitors + coupon initiatives, blocked on a user decision (not in approved items 2/3/4/7).
- Pending human: physical controllers, activity difficulty, HDMI audio listening.
- Next action: wait for user feedback or approval of a new original design.

## Release 1 (RELEASE_1.md) — 2026-11-14
- Done: R1-R5 (see RELEASE_1.md progress log). Suite 71 pass + 1 ignored helper.
- Next action: R6 content audit (acquisition paths, menu options, quests), R7 balance pass, then R8 docs (RELEASE_NOTES.md, HUMAN_VERIFICATION.md trim), final commit.
- R6-R8 done. Full suite 73 pass (+2 ignored helpers: camp-site finder, real-save migration). Release 1 implementation complete; human checks pending.
- Next action: none until user playtest feedback. Launch: `cargo run --release`.
- 2026-11-14: Controller player switch: hold View + press Menu moves a pad between Player 1 and Player 2 (two pads swap; P2 leaves if no pad remains on P2, progress saved). Corrects the earlier claim that a lone pad drives P1 (it claimed P2). Unit test view_menu_switches_pad_between_players; physical check added to HUMAN_VERIFICATION H2. Suite 74 pass.

## Depth pass (BACKLOG D-01..D-06)
- D-01 DONE: friendship tiers + tier lines, seasonal/contextual/gossip/hobby lines (6 characters), best-friend keepsake portraits (140-145), Friends journal page. Test resident_depth_tiers_context_keepsake; screenshot d01_friends_page (phase 31).
- D-02 DONE: Blossom Picnic, Starlit Regatta, Frost Lantern Festival, First Light (FESTS in data.rs), host Hazel at the plaza, yearly keepsake per player per festival (146-149), decorations, resident lines, calendar in guide. Test festivals_tasks_rewards_yearly; phase 31 screenshots of all four (Regatta boats are on river tiles out of that camera view: unverified visually).
- Fixed two flaky tests (random weeds/items on the test site).
- Next action: D-03 daily surprises.
- D-03 DONE: daily shared surprise (message bottle -> mailbox letter + 250c; lost scarf -> return to owner +4 friendship +400c; drifting gift -> furniture), residents hint at it; test daily_surprises_shared_and_saved; phase 32 screenshots.
- D-04 DONE (partial scope): mood emotes (note, z, ..., !, greeting heart) and idle residents facing a nearby player; phase 32 frames. Idle prop actions (watering etc.) not implemented.
- Next action: D-05 evolving shop.
- D-05 DONE: store renovation tiers (spending tracked on all store purchases), weekly special display, extra daily stock slot, renovation letters; test shop_renovation_tiers_special_extra_stock; phase 33 screenshot.
- D-06 PARTIAL: chimney smoke (morning/evening/cold months) and a distant bird flock on clear days; phase 33 screenshot. Open: riverside fences, festival music arrangement.
- Remaining depth backlog: D-04 idle prop actions, D-06 fences + festival music. Next action: D-06 festival music variant in audio.rs (Var::Festive), then idle actions.
- Festival music: audio Var::Festive (bell doubling, full arpeggio, light percussion) on festival and Harvest Fair days outdoors; passes the click/seam test; test festive_music_on_event_days. Not listened to (human check).
- D-04 idle hobby props (basket, watering can, telescope, towel, yarn, wrench) when residents stand still in dry weather; phase 33 frame d04_props (fixture places residents) shows basket/towel; others occluded from that angle.
- Depth backlog status: D-01..D-05 done, D-06 done except riverside fences (low priority). Suite 79 pass.
- Next action: optional riverside fences; otherwise await user playtest feedback.
- Visual gap fixes: riverside fences (grass edges meeting the river), Regatta boats now along the river tiles nearest the camera, larger/offset hobby props with an arm. Verified in phase 34 (split-screen night Regatta frame; clear-day prop frame with all 4 props visible).
- M2 defined in BACKLOG (gardening, tool upgrades, home evaluation + exterior paint). Next action: M2-1 gardening.
- M2-1 DONE: gardening (shared garden, seeds/can via counter menu entry "Seeds, watering can and tool upgrades", plant/water/grow/pick/cross, liveliness, initiative template "Water 3 flowers"). Tests gardening_plant_water_grow_pick_cross; phase 35 (menu purchase, plant + water through normal input; fixture adds grown plants for the stage frame).
- M2-2 DONE: tool levels per player with real effects and tinted tools; test tool_upgrades_costs_and_effects.
- Found + fixed: Mr. Tumble's tile is shadowed by the counter target, so the shop menu got an entry for his services.
- Next action: M2-3 home evaluation + exterior paint.
- M2-3 DONE: weekly Home Society letter (score, tips, one-time tier rewards incl. Golden Home Plaque), home score on the Friends page, exterior repaint via Hazel (6 palettes) shown on the town model. Test home_evaluation_letters_and_paint; phase 36 screenshots (paint menu, repainted house, home score).
- Milestone M2 complete. Suite 82 pass (+2 ignored helpers). Next milestone candidates (M3, not started): resident invitations (meet-ups, hide-and-seek), monthly fishing/bug contests.
- LAN multiplayer added (src/net.rs): host-authoritative, client sends input (key indices) each frame, host streams JSON snapshots (~20/s) of the world + runtime state; UDP discovery; refuse when Player 2 is local; F2 disabled while a LAN friend is connected. Tests: lan_host_client_loopback (unit, real sockets on loopback); autotest phases 37/38 run as two processes on 127.0.0.1 (HOLLOWBROOK_LAN_BIND), screenshots of both views inspected. NOT verified across two physical computers or through Windows Firewall (human check).
- 2026-11-15 CRASH FIX (user report, LAN session): world.rs sim_fish indexed shadows[si] after another player's catch had removed a shadow -> index out of bounds. Shadows removal now goes through take_shadow(), which re-indexes every player's hooked fish; a stale index releases the line. Test two_players_fishing_same_river_no_crash.

## 2026-11-15 — Visual polish loop (presentation only, no gameplay changes)
- Autotest phase 40: fixed-viewpoint visual tour (test_output/tour.sh <prefix> runs it at 11:00 and 17:10 and builds contact sheets).
- Terrain: per-corner smooth value noise replaces the per-tile checker; soft blended borders between grass, paths and sand.
- Sky: vertical gradient (deeper zenith to horizon/fog colour); interiors get a warm dim backdrop instead of flat black.
- Interiors: plank floors with staggered seams (homes/residents/civic), grouted tiles (shop/museum), wainscot + chair rail + striped wallpaper, faint daytime sunbeam under windows.
- Evidence: rendered screenshots inspected (test_output/v0_* baseline vs v4_*); 84 tests pass.
- Water: smooth per-corner ripples, sea deepens offshore, turquoise shallows at banks, animated foam along bank edges, daytime sun glints (render only).
- Trees: root flare, branch stub, extra canopy lobes with shaded underside, per-tree tint/shape variety, gentle wind sway; pines get tone variety and dark tier skirts. Not screenshot-verified (tour run skipped at user's request).
- Delegated to Codex (GPT-6.1-Sol high), screenshot-verified by both: window dividers + sills (6e61ed0), roof eaves fascia (caf038b), home curtains + sill (ccdb79e). Golden-hour tint and clothing cuffs tried and rejected (no clear improvement).

## M3-1 — Monthly surveys (round 3)
- Original River Measure / Wing & Leaf Survey calendars, measured fish/bug slots, per-player registration/best entry/prize receipts, safe delayed/full-pocket mail rewards, new original plaques, guide topics and notice-board text. Save v4 -> v5 preserves all existing quantities; LAN protocol 2 rejects incompatible menu schemas.
- H: cargo test --release: 87 passed, 0 failed, 2 ignored (new tests cover both players, capture paths, storage/gifts, legacy migration, reload, rollover, duplicate/full-pocket payouts).
- X/S: phase 41, test_output/contest_fish_g3_final, contest_bug_g3_final, contest_award_g3_final. Registration, submission, and prize collection through real input; catches/award eligibility are fixtures. Screenshots inspected; initial host overlapped a tree and was moved to the open crossroads before acceptance.
- Build: release succeeded, existing warnings only. Next action: implement and verify resident home invitations, then hide-and-seek. No physical difficulty assessment claimed.

## M3-2a — Resident home invitations
- Nearby friends initiate saved, non-interrupting visit offers; talk to accept/decline. Accepted-hour home presence is shared; arrival bonus/gift and receipt are personal. Missed appointments give one gentle later line with no losses. Old saves default to no invitation; roster changes expire stale offers. Full-pocket gifts are mailed.
- H: 89 passed, 0 failed, 2 ignored (new tests: initiated offer/reload/real acceptance input, early/late visit, independent reward, repeated arrival, decline, missed next-day line, full pockets, changed resident).
- X/S: phase 42, test_output/invitation_offer_g3, invitation_visit_g3, invitation_missed_g3. Accepted through normal dialogue; visit enters the real door at 13:15 with a saved eligibility fixture, receipt/gift logged; guide topic and follow-up inspected in split-screen. Release build succeeded with existing warnings.
- Next action: implement/verify per-player hide-and-seek with shared resident pathfinding and absolute deadlines.

## M3-2b — Trail Seek and final round-3 verification
- Residents initiate original Trail Seek offers on the personal activity cycle. Three shared residents use normal pathfinding to saved, reachable hiding spots. Each player has independent progress, an absolute four-minute deadline, and a once-only reward receipt. Overlapping rounds reuse the same actors/locations. Existing appointments and ambient visits are protected; construction cannot occupy active hiding tiles.
- H: final `cargo test --release`: 93 passed, 0 failed, 2 ignored; 9 new M3 tests total. Seek tests cover actual NPC walking, normal talk targeting, two-player progress/rewards, partial/completed reload, indoor/offline expiry, rollback after expiry, active/inactive rollover, blocked starts, appointment protection and full-pocket mail. A complete v4 two-profile migration test compares the entire upgraded world, including possessions and default activity fields.
- X/S: phase 43 in `test_output/seek_success_g3` and `test_output/seek_expiry_g3`. Eligibility/near-expiry are fixtures; acceptance, walking, finding through talk, and entering the home use normal inputs. Inspected invitation, countdown, first find, completion, indoor timeout, and monthly-survey guide PNGs. P1 received 240 coins and a Branch while P2 remained unchanged. The scripted route is not a human difficulty assessment.
- Final release build succeeded (existing five warnings only). Test log: `test_output/visual/game3_final-test.log`; build log: `test_output/seek-g3-build.log`. LAN protocol is now 3, carrying the host clock through snapshots for consistent countdown rendering.
- S: final phase-40 tour `test_output/game3_regression_day` / `_dusk` compared with accepted `curtains_r2_day` / `_dusk`. Inspected all four `test_output/curtains_r2_vs_game3_regression_{day,dusk}_{0,1}.png` pages (eight views at both times). No visible loss of detail, colour or geometry; normal random stock/fruit/items, NPCs and particles differ. No scene lighting/mesh/material changes added for M3.
- Round 3 implementation and requested verification complete. Next action: await user playtest feedback; human activity difficulty, physical controllers and measured frame-rate checks remain unclaimed. Full results and local commits are recorded in `.herd/report-game-3.md`.

- 2026-11-17: Home stories. Expansion levels 3 (basement, 30000c) and 4 (upstairs, 50000c), provisional prices. Stairs in the back-left corner; each story keeps its own furniture (Profile.story/stories, serde-default so old saves load); visitors see the main floor; leaving the home always returns to the main floor; home score counts every story. Evidence: H basement_and_upstairs_stories; S test_output/floors/h_main|h_basement|h_upstairs.png (phase 44).

- 2026-11-17: UI glitch pass (screenshots of all 42 autotest phases, test_output/ui_audit). Fixed: recent messages showed through menus and ran past split-screen halves (now hidden in menus, shortened with "...", soft backdrop); help screen guide text ran into the bottom bar (now stops with a pointer to the journal Guide); 3D name labels peeked out beside full menus; Trail Seek panel moved below the 4-line message area. Regression guards: H ui_text_budgets (controls/help/item-name sizes); rendered: tools/ui_audit.sh. S: test_output/ui_fix/p30, p30b, p43.

- 2026-11-26: Dustreach frontier outpost (separate world, original design). Scene::Frontier with its own map: Guild Post, Longhouse, market tents, water tower, wreck, dome huts, canyon walls, gate back to Hollowbrook. Bounty board: 3 personal offers per day (Easy waits / Runner flees when you get close and tires / Lookalikes: find the red scarf among three), tracker (cold-warm-hot), detain with E, turn in at the board for coins (400/900/1600, provisional) and guild rank (Drifter..Legend). Per-player, saved (Profile.bounty, bounties_done, bounty_log; serde-default), uncaught bounties go cold overnight. No door from Hollowbrook yet (awaiting the user design); dev entry F9 with HOLLOWBROOK_DEBUG. Evidence: H tests/frontier.rs (3); S test_output/frontier_day + frontier_dusk (phase 45).

- 2026-11-26: Dustreach gear (original, non-lethal, works only in Dustreach; holstered in Hollowbrook): Stun Pistol 1500c (fast bolt, 2.5 s stun), Tangle Launcher 2500c (slower cords, 5 s stun, longer reload), Shock Staff 1000c (melee arc, 3 s stun). Bought at the Guild Post board, equipped as tools (Tab/Space). Stunned targets stop running and show stars. Provisional numbers. Evidence: H tests/frontier.rs gear_buy_stun_and_holstered_in_town; S test_output/gear_shots (phase 46).

- 2026-11-26: Door to Dustreach: a winding path from the west edge of town (by the pines at x=1, z=17) into the woods, ending at a wooden door with a desert glow, lantern and sign. Path tiles lie outside the saved map (tile_at override), so saves are unchanged; the Dustreach gate returns you to the door. Camera moved slightly closer (FOV 40->35 outdoors, 45->40 indoors) so everything looks a bit bigger; no sizes or rules changed. Evidence: H woods_path_door_leads_to_dustreach_and_back; S test_output/woods (phase 47).

- 2026-11-26: Dustreach enlarged 30x24 -> 40x30: oasis pond with palms, reeds and glints (water by tiles, drawn as soft ovals), natural stone arch over a trail to the oasis, 4 more structures, 3 more hiding spots, longer cross street; gate/spawn moved south. Test asserts every hiding spot is open ground, the oasis is water and the arch is walkable. S: test_output/quickshot (phase 45 incl. f_oasis).
