# Hollowbrook — Release 1 scope (finite)

Release 1 = the current game plus the items below. Anything not listed is post-release.

## 1. Required implementation work
| ID | Item | Acceptance criteria | Status |
|---|---|---|---|
| R1 | Campground | One campground at tiles 31-34 x 19-22 (free meadow, east). Tent, campfire, goods crate, schedule sign. 4 original visitors rotate one per day (day id mod 4); present 8:00-22:00. Each visitor: distinct dialogue, 3 goods for coins (one unit each per day, shared stock), one favor (items in, unique reward out) saved per player. Survives reload; both players independent. Unit tests + rendered screenshot. | in progress |
| R2 | Initiatives + coupons | 3 personal daily tasks per player (reset 06:00), 1 shared weekly town task (reset Monday 06:00). ≥12 templates from existing activities, only assigned when attainable (tool owned, resource present). Progress, deadline, reward shown in journal page and at the kiosk. Per-player coupon balance. Explicit claim (one time) at the Town Hall kiosk. Kiosk sells ≥8 original rewards for coupons. Saved; no duplication via reload/simultaneous input. Tests (both players). | in progress |
| R3 | Title / start flow | Title screen: game name, version/build, Continue (or Start a new town when no save exists), Controls/help, Quit. No option deletes a save. | open |
| R4 | Help and guide | Controls page (keyboard + gamepad from src/pad.rs mapping) and a short guide to housing balance, projects, policies, travel, activities, campground, initiatives. Reachable from title and in-game journal (page switch). | open |
| R5 | Version identifier | Version + git short hash compiled in; shown on title and journal. | open |
| R6 | Content audit | Every obtainable item has an acquisition path; collectibles donate; menu options work; quests attainable; destinations enter/exit. Findings fixed or listed below. | open |
| R7 | Balance pass | No dead ends, unaffordable mandatory steps, repeat-reward exploits, rewards lost to full pockets, or tasks broken by sold-out stock. Documented in DESIGN_SPEC. | open |
| R8 | Release docs | RELEASE_NOTES.md, HUMAN_VERIFICATION.md (remaining checks only), final commit recorded. | open |

## 2. Already delivered (regression-protected by 67 tests + autotest phases)
Daily clock/rollover, seasons, weather; bugs/fish/diving/digging/fossils; museum with art wing; shop with rotating stock,
clothing, catalog, recolouring; housing with interest-free per-player balance; resident routines, requests, gifts,
letters, birthdays, visits, moving; material projects and leader-placed funded projects; policies at the Town Hall;
Gullrock trips, daily task, Sea Glass stall, three timed activities; furniture collections; Saturday art stall;
split-screen two-player with gamepads; music/SFX volume settings; autosave every 15 s, on rollover and on quit.

## 3. Content coverage (Release 1 targets)
Bugs 10, fish 13, sea creatures 6, fossils 4, paintings 8, furniture ~32 (incl. 2 six-piece sets + camp/kiosk items),
clothing 7, residents 4 of 6 characters, visitors 4, policies 3, funded projects 5, material projects 4, activities 3,
initiative templates ≥12, kiosk rewards ≥8.

## 4. Known defects / limits
- Leader fixed to Player 1 (accepted).
- Audio through HDMI: Windows meter shows output; listening unverified.
- Town Hall/Museum/Shop are single rooms; no interiors beyond those listed.

## 5. Optional post-release
Leader reassignment, more residents/visitors, seasonal events beyond the Harvest Fair, more furniture sets, online play.

## 6. Human/hardware checks (not software work)
Physical gamepads (two), keyboard feel, audio listening on the user's devices, activity difficulty, overall fun/pace.

## Progress log
- R1 campground, R2 initiatives/coupons/kiosk, R3 title flow, R4 help/guide (journal pages + title), R5 version (build.rs git hash): implemented 2026-11-14; tests campground_visitors_stock_favor_two_players, initiatives_tasks_claims_kiosk_two_players, task_events_from_normal_actions, version_and_help_present; autotest phase 30 screenshots inspected (title, help, camp menu, both players at camp, kiosk, journal pages incl. split collection).
- Remaining: R6 content audit, R7 balance pass, R8 release docs.
