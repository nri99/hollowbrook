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
- [B] Civic leader permit, donation-funded public construction, town policies.
- [B] Tropical travel with medal activities and reward shop; campground visitors and coupon initiatives.
- [B] Debt-based home loans (house expansion without debt stays in P5-01).

## P2 — presentation and readability
- [x] **P2-01 Occlusion fade.** Buildings/trees between camera and player are drawn translucent/cut so the player stays visible. *AC:* behind-house screenshot shows player.
- [ ] **P2-02 Tool feedback.** Swing animation + small effect (splash ring, dirt puff) for each tool. *AC:* screenshot during use.
- [x] **P2-03 Running tramples flowers.** Running over a flower tile can flatten it (persisted, regrows next day). *AC:* unit test.

## P3 — collections and museum
- [x] **P3-01 Museum building + donations.** New building on free grass (town otherwise unchanged; saves remap safely). Curator interaction donates any undonated fish/bug/treasure; donation log persisted. *AC:* tests: donate once, no duplicate donation, item leaves pockets.
- [x] **P3-02 Visible exhibits.** Museum interior shows donated species (fish tank models, bug cases, treasure pedestals). *AC:* screenshot inspected.
- [x] **P3-03 Completion records.** Journal shows donated/total per category. *AC:* test + screenshot.

## P4 — civic systems
- [B] **P4-01** Requires an original civic design approved by the user (see DESIGN_SPEC §14).

## P5 — housing, clothing, catalog, shop growth
- [x] **P5-01 Home expansion.** Pay at the shop counter to enlarge the home room (4×4 → 5×5 → 6×5); furniture keeps positions. *AC:* tests for cost, persistence, room dims; rendered check.
- [x] **P5-02 Clothing.** Shop sells 4 shirts; equipping changes outfit colour; persisted. *AC:* test + screenshot.
- [x] **P5-03 Catalog reorder.** Furniture ever owned can be re-ordered at the counter at full price. *AC:* test.

## P6 — residents
- [x] **P6-01 Mailbox and letters.** Letters are stored (not just toasts) in a mailbox by the home; read/claim attached gifts once. *AC:* tests: claim once, persists.
- [x] **P6-02 Idle activities & rain.** Residents sometimes stop to look at flowers / stretch; carry umbrellas in rain. *AC:* rendered.
- [ ] **P6-03 Gifts.** Give a resident an item from pockets; reaction depends on preference. *AC:* test.

## P7 — seasons, weather, events
- [x] **P7-01 Snow in winter** (ground tint + falling snow instead of rain). *AC:* rendered with dev clock.
- [ ] **P7-02 One recurring seasonal event** (provisional date and rules). *AC:* test for date window.
