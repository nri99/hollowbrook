# Hollowbrook — Final Report (autonomous run)

Stopping because every remaining task is either **blocked on a user decision**, **unspecified by the design
(would require inventing content the instructions forbid fabricating)**, or **physical-input verification** that the
agent cannot perform (see HUMAN_VERIFICATION.md). A bounded regression pass was run before stopping.

## Completed task IDs
P0-01, P0-02 · P1-01..P1-05 · P2-01, P2-02, P2-03 · P3-01..P3-03 · P5-01, P5-02, P5-03 · P6-01, P6-02, P6-03 ·
P7-01, P7-02 · MP-01..MP-05 · C-01..C-05 · R15 (tool FX) · R16 (content coverage table)

## Playable improvements this run
- Save safety: rotating backups (5) on every load/migration; corrupt saves set aside, never overwritten; save v4.
- Two local players: separate profiles (money, pockets, home + storage, tools, journal, relationships, clothing, mail,
  catalog), split-screen with independent cameras/HUD/menus/transitions, players in different scenes at once, a
  second home, owner-only furniture editing, safe join/leave (F2 or controller Start).
- Gamepads (gilrs): explicit slot assignment, dead zone, edges, menu repeat, button prompts, disconnect banner,
  reassignment.
- Museum with shared one-per-species donations, fossil identification, visible exhibits (tank, bug case, pedestals).
- Swimming and diving with sea creatures; weather (rain/snow), umbrellas; occlusion fade (buildings, trees, pines);
  visible tool effects.
- Clothing (shirts, hats), hairstyles, furniture tints, wallpaper/flooring, home expansion (paid up front), catalog
  reorders, mailbox with one-time gifts, resident gifts with preferences, resident home visits, residents moving
  out/in with a "please stay" option, Harvest Fair event.

## Test results (actual)
- `cargo test --release`: **48 passed, 0 failed** (last run during the regression pass).
- Executable action-path runs (autotest bot injecting keys into the real input layer; logged FIXTUREs where noted):
  phases 0–18 each run at least once; regression pass re-ran phases 1, 2, 4: full single-player loop + relaunch
  persistence (money 720, sold-out Lamp, Lamp placed and on) and two-player independence passed.

## Screenshots actually inspected (test_output/)
p1/02_behind_house_occlusion · p3/p3_fish_bite, p3_dig_site, p3_resident_talk, p3_night_town, p3_lamp_on ·
mp/mp_split_home_and_town, mp_p2_inventory, mp_same_scene, mp_together, mp_pad_prompts, mp_pad_disconnected ·
p3m/p3_museum_exhibits, p3_dig_site (recoloured puff) · p5/p5_new_outfit · p51/p51_bigger_home ·
p61/p61_mailbox_flag · p7snow/p7_weather, p7rain/p7_weather · p53/p53_catalog · p63/p63_gift_reaction ·
p72/p72_fair · c01/c01_swimming · c02/c02_visit · fx/fx_cast_splash, fx_net_swish · c03/c03_Clover, c03_Basil
(partial: mostly hidden) · c04/c04_custom_home · c05/c05_looks · regress/12_relaunch_home.

## Save compatibility
v1/v2 text saves → migrated (backup first); v3 JSON → upgraded to v4 (profile 0); all later additions use serde
defaults (older v4 saves load). The player's real legacy save was backed up to `save_backups/` before any work.

## Controls
Keyboard (P1): WASD/arrows move · Shift run · E interact · Space use tool / dive · Tab or Q cycle tool · I pockets ·
J journal · F/G move/pick up furniture · R rotate while placing · Enter confirm · Esc cancel · in conversation G gift,
X ask to stay · F2 add/remove Player 2 · dev: F12 screenshot (with HOLLOWBROOK_DEBUG).
Gamepad: stick/d-pad move · RT run · A confirm/talk · B cancel · X tool/dive · Y pockets (also "sell all" in the
sell menu, "ask to stay" in conversation) · RB next tool / rotate · LB pick up furniture / gift · Select move
furniture · Start journal (and joins as Player 2).

## Incomplete content (see DESIGN_SPEC "Content coverage")
Most areas are PARTIAL samples (8 bugs, 11 fish, 4 sea creatures, 4 fossils, 8 furniture, 7 clothing items, 6
characters, 1 event). NOT IMPLEMENTED for lack of specification: art exhibits, visiting characters/entertainment,
full event calendar, music, more than two local profiles.

## Blocked (needs the user)
- Civic permit / public construction / policies; tropical travel with medals and reward shop; campground and
  coupon initiatives; debt-based home loans — recreation-sensitive feature sets that need an original design approved
  by the user (BACKLOG "Blocked").
- Physical keyboard and controller verification (HUMAN_VERIFICATION.md H1–H5).

## Provisional behaviour
All prices, shop hours (8–22), regrowth (3 days), shells (4/day), friendship rules, letters, weather frequency,
schedules, visit/moving thresholds, fair dates (13–16 Nov), fossil identification, delivery timing, palettes and
fees are invented placeholders — tagged [PROV] in DESIGN_SPEC.md and PROGRESS.md.

## Hardware verification gaps
No physical keyboard or gamepad input has been verified by the agent. Gamepad compatibility is unverified.

## Next concrete actions
1. Run HUMAN_VERIFICATION.md H1–H5 and record results.
2. Decide on original designs (or keep out) for the blocked civic / travel / campground / loan systems.
3. Provide or approve content lists (items, events, visitors) if fuller coverage is wanted.
4. Visual pass on Basil (hedgehog) model, which was only partly visible in its screenshot.
5. Trim the debug overlay from split-screen dev screenshots and consider music direction.
