# Hollowbrook v1.0.0-rc1 — release notes

Build and launch: `cargo run --release` (from the project folder). The build id is shown on the title screen and in the journal.

## New in Release 1
- **Title screen**: Continue / Start your town, Controls and guide, Quit. Nothing on it deletes a save.
- **Campground** (east meadow): four original travellers (Tamsin the hare, Barnaby the tortoise, Marigold the goat,
  Cobb the mole) rotate one per day, 8:00-22:00. Each has their own dialogue, three goods for coins (one of each per day
  for the whole town) and one favor that rewards a keepsake once per player. A sign explains today's and tomorrow's visitor.
- **Initiatives and coupons**: three personal tasks per player per day, one shared weekly town task, coupon balance per
  player, explicit claiming and eight rewards at the kiosk inside the Town Hall.
- **Journal pages**: Collection, Initiatives, Settings (music / sound volume), Controls, Guide. Tab or RB switches page.
- **Version identifier** (`v1.0.0-rc1 (build <git hash>)`).
- Content fixes: every item now has an acquisition path (non-native fruit washes up on Gullrock); Captain Pell's Sea Glass
  is mailed if pockets are full; split-screen journal and dialogue hints fit their viewport.

## Already in the game
Real-time days and seasons, weather, bugs/fish/diving/digging/fossils, museum with art wing, rotating shop, clothing,
catalog, recolouring, housing with interest-free balance, residents with routines/requests/gifts/letters/birthdays,
material and funded public projects, policies, Gullrock trips and timed activities, furniture collections, Saturday art
stall, two-player split-screen with keyboard and gamepads, autosave.

## Verification status
- Automated: unit tests (full suite) and scripted runs through the real input layer with inspected screenshots.
- Not verified by the agent: physical gamepads, listening on your speakers, human difficulty/fun. See HUMAN_VERIFICATION.md.

## Known limitations
- Town leader is always Player 1.
- Naming a new town requires a keyboard (Enter alone uses "Neighbor").
- Balance values are provisional (DESIGN_SPEC.md).
