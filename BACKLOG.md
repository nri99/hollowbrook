# Backlog (ordered; take the highest unblocked item)

Status: `[ ]` open · `[x]` done · `[B]` blocked (reason given)

## P0 — crashes, saves, duplication, controls
- [x] **P0-01 Save backups.** Before migrating a legacy save or loading/overwriting a JSON save at startup, copy it to `save_backups/` (keep newest 5 per kind). *AC:* test shows backup created on migrate; rotation keeps ≤5.
- [x] **P0-02 Corrupt save safety.** If `hollowbrook_save.json` exists but fails to parse, do not start a new game over it: rename to `.corrupt-<time>` and fall back to legacy/new. *AC:* unit test with garbage JSON; original bytes preserved.
- [B] **P0-03 Physical keyboard verification.** Needs the user at the keyboard (desktop-wide synthetic input is not permitted). *AC:* user walks, presses E at home door, opens I, exits.

## P1 — finish current milestone (rendered verification)
- [x] **P1-01 Fishing rendered.** Autotest phase 3 equips rod, casts, waits for bite, catches; screenshot of shadow/bobber. *AC:* fish in pockets; screenshot inspected.
- [x] **P1-02 Digging rendered.** Buy shovel, dig a dig site and a plain hole, fill one. *AC:* treasure obtained, hole visible in screenshot.
- [x] **P1-03 Bug catching rendered.** Net swing catches a bug. *AC:* bug in pockets or documented absence of spawn.
- [x] **P1-04 Resident interior visit.** During a resident's home hours enter their home and talk. *AC:* screenshot shows themed room + resident.
- [x] **P1-05 Night + lamp.** Dev clock at night: lit windows outdoors; lamp on/off indoors changes lighting. *AC:* two screenshots inspected.

## MP — local multiplayer (priority 2, before P3+)
- [x] **MP-01 Profiles.** `Profile` (name, money, house level, pockets, storage, home room, tools, equip, journal, relationships, outdoor pos) separate from shared world; runtime `Player` (scene, position, facing, pose, UI, placing, fishing, transition). Save v4 with profiles; v3 migrates into profile 1. *AC:* H tests: v3→v4 keeps everything; two profiles independent; save/reload.
- [x] **MP-02 Two players in the simulation.** Join player 2 (new profile, own home), per-player update with its own input; NPC talk focus per player; shared transactions safe. *AC:* H: simultaneous pickup, last stock, different scenes, rollover with both.
- [x] **MP-03 Split-screen rendering.** Two viewports, independent cameras/HUD/menus/transitions. *AC:* X + S screenshot with one player outdoors and one indoors.
- [x] **MP-04 Gamepads.** gilrs input: assignment (press Start to join, slot per pad), dead zone, edges, menu navigation, button prompts, disconnect/reconnect. *AC:* H with virtual pad state; P physical NOT RUN until human verifies.
- [x] **MP-05 Second home.** Player 2 home building on free grass with its own room. *AC:* H + S.

## Blocked pending an original design approved by the user (recreation-sensitive feature sets)
- [x] Donation-funded public construction (leader placement, preview, both players donate) and town policies (civic desk) — 2026-11-14. Leader permit flow: leader is fixed to player 1 (provisional).
- [~] Tropical travel: DONE as Gullrock (timed activities, Sea Glass reward stall; user-approved item 7). [B] Campground visitors and coupon initiatives: not among the approved items; needs a user decision and an original design.
- [x] Debt-based home loans: interest-free per-player expansion balance (user-approved item 2, 7da590e).

## P2 — presentation and readability
- [x] **P2-01 Occlusion fade.** Buildings/trees between camera and player are drawn translucent/cut so the player stays visible. *AC:* behind-house screenshot shows player.
- [x] **P2-02 Tool feedback.** Swing animation + small effect (splash ring, dirt puff) for each tool. *AC:* screenshot during use.
- [x] **P2-03 Running tramples flowers.** Running over a flower tile can flatten it (persisted, regrows next day). *AC:* unit test.

## P3 — collections and museum
- [x] **P3-01 Museum building + donations.** New building on free grass (town otherwise unchanged; saves remap safely). Curator interaction donates any undonated fish/bug/treasure; donation log persisted. *AC:* tests: donate once, no duplicate donation, item leaves pockets.
- [x] **P3-02 Visible exhibits.** Museum interior shows donated species (fish tank models, bug cases, treasure pedestals). *AC:* screenshot inspected.
- [x] **P3-03 Completion records.** Journal shows donated/total per category. *AC:* test + screenshot.

## P4 — civic systems
- [x] **P4-01** Civic systems delivered per user-approved items 3/4 (funded projects, policies, Town Hall interior).

## P5 — housing, clothing, catalog, shop growth
- [x] **P5-01 Home expansion.** Pay at the shop counter to enlarge the home room (4×4 → 5×5 → 6×5); furniture keeps positions. *AC:* tests for cost, persistence, room dims; rendered check.
- [x] **P5-02 Clothing.** Shop sells 4 shirts; equipping changes outfit colour; persisted. *AC:* test + screenshot.
- [x] **P5-03 Catalog reorder.** Furniture ever owned can be re-ordered at the counter at full price. *AC:* test.

## P6 — residents
- [x] **P6-01 Mailbox and letters.** Letters are stored (not just toasts) in a mailbox by the home; read/claim attached gifts once. *AC:* tests: claim once, persists.
- [x] **P6-02 Idle activities & rain.** Residents sometimes stop to look at flowers / stretch; carry umbrellas in rain. *AC:* rendered.
- [x] **P6-03 Gifts.** Give a resident an item from pockets; reaction depends on preference. *AC:* test.

## P7 — seasons, weather, events
- [x] **P7-01 Snow in winter** (ground tint + falling snow instead of rain). *AC:* rendered with dev clock.
- [x] **P7-02 One recurring seasonal event** (provisional date and rules). *AC:* test for date window.

## Next slices (from the R16 coverage table)
- [x] **C-01 Swimming + diving** in the sea with a small original sea-creature roster (provisional rules). *AC:* H + X + S.
- [x] **C-02 Resident home visits**: a resident occasionally visits a player's home while they are inside (provisional schedule). *AC:* H + S.
- [x] **C-03 Resident moving out/in** with safeguards (announce a day ahead, never while a request is active) — provisional. *AC:* H.
- [x] **C-04 Furniture customization** (re-colour at the counter) and wallpaper/flooring items. *AC:* H + S.
- [x] **C-05 More clothing slots** (hats) and a hairstyle choice at the shop. *AC:* H + S.

## Approved original expansion (user-approved 5 directions) — finite, in order
- [x] **X-01 Community projects.** Materials (Branch from shaking trees, Stone from hitting rocks with the shovel, any fruit/shell) fund resident-proposed bench / lamp post / planter / bridge at validated sites; both players contribute at the site sign; progress shown; completed objects usable (sit on bench, lamps light at night, planters bloom, bridge walkable); persisted. *AC:* H (contribution caps, two players, persistence, route safety) + X + S.
- [x] **X-02 Liveliness + reactions.** Town liveliness score (notice board, journal), residents react to completed projects (dialogue + happy). *AC:* H + S.
- [x] **X-03 Islet getaway.** Dock + boat (confirm, fare), Scene::Islet with own scenery/wildlife, rotating daily activity, Sea Glass reward + islet stall with original items; per-player travel; no item loss. *AC:* H + X + S (split-screen with one player away).
- [x] **X-04 Collections.** (DONE 2026-11-14: Meadow + Harbor sets of 6, 4 critters, art wing of 8, Saturday art stall, exhibit labels) Themed furniture sets, more critters with habitat/season rules, art wing with procedural original paintings, museum labels. *AC:* H + S; counts updated.
- [x] **X-05 Social depth.** Birthdays, resident-to-resident chats (speech bubbles), varied requests (deliver furniture / catch at location / weather-specific), more dialogue variety. *AC:* H + S.
- [x] **X-06 Atmosphere.** (DONE 2026-11-14: music, footsteps, particles, split dialogue framing, blinking + brows, talking nods, run dust, building siding/trim) Procedural music by hour/weather, surface-dependent footsteps, seasonal particles (leaves, fireflies, petals), animation/face polish, dialogue framing in split-screen. *AC:* S (+ audio by code inspection only).
- [x] Gullrock timed activities: Crab Corral, Beacon Dash, Ring Swim (solo/co-op, HUD, tiered Sea Glass) — 2026-11-14.
- [x] Town Hall interior with Hazel Quill and the civic desk — 2026-11-14.
- [x] Autotest + rendered checks: Crab Corral co-op, Ring Swim solo, placement preview valid/invalid, partial + built project (phases 25-27).

## Depth pass (2026-11-14, user-approved direction) — finite, in priority order
- [x] **D-01 Resident depth.** (DONE 2026-11-14) Friendship tiers (Acquaintance → Best friend) with tier dialogue; per-character hobbies and personal lines (6 characters); seasonal and contextual lines (player home, museum, campground, policies, other residents); best-friend keepsake portrait (once per player); Friends journal page. *AC:* tests + screenshot.
- [x] **D-02 Calendar celebrations.** (DONE 2026-11-14) Four new original events (Blossom Picnic, Starlit Regatta, Frost Lantern Festival, First Light) besides the Harvest Fair: decorations, host at the plaza, event task with a yearly reward per player, resident lines, calendar in the guide. *AC:* tests + screenshot per event type.
- [x] **D-03 Daily surprises.** (DONE 2026-11-14) One shared surprise per day (message bottle, resident's lost item to return, drifting gift) placed in town. *AC:* tests + screenshot.
- [x] **D-04 Expressive residents.** (DONE 2026-11-14: emotes + look-at; idle prop actions not done) Mood emotes over heads (happy, sleepy, annoyed, music), contextual idle actions (watering, stretching, reading). *AC:* rendered frames.
- [ ] **D-05 Evolving shop.** Store renovation tiers from cumulative town spending: more daily stock and a weekly special. *AC:* tests + screenshot.
- [ ] **D-06 Environment detail.** Chimney smoke, flying birds, riverside fences and lamp posts lit at night, event-specific music variant. *AC:* screenshots.
