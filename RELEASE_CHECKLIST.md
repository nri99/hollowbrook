# Release checklist

Each item needs the listed evidence. Evidence types: **H** headless test · **X** executable action path (autotest bot through
the real input layer) · **S** rendered screenshot inspected · **P** physical keyboard/controller (human). An item is done only
when every listed evidence type exists and is recorded in PROGRESS.md.

| ID | Requirement | Evidence | Status |
|---|---|---|---|
| R01 | Single-player loop: gather → sell → buy → place → use → save → relaunch | H X S | done (H X S) |
| R02 | Save migration v1/v2 → current, with backup; corrupt save never overwritten | H X | done |
| R03 | Daily rollover at 06:00 once, also across interiors and multi-day absence | H X | done |
| R04 | Two local profiles: separate money, pockets, home room, storage, tools, journal, relationships | H | done (H) |
| R05 | Two-profile save/reload; v3 single-player save migrates to profile 1 | H X | done (H; X v3→v4 upgrade) |
| R06 | Split-screen: independent cameras, HUD, prompts, menus, transitions per half | X S | done (X S; P pending) |
| R07 | Players in different scenes at once (one outdoors, one indoors), both updated + rendered | H X S | done |
| R08 | Shared world transactions: one pickup, one last-stock purchase, no double spend | H | done (H) |
| R09 | Independent dialogue/menus: one player's input never drives the other's UI | H X | done |
| R10 | Gamepad support (2 pads): explicit assignment, dead zone, edges, menu navigation, prompts | H (virtual) P | H + X(virtual pad) done; P NOT RUN |
| R11 | Controller disconnect/reconnect/reassign without progress loss; join/leave safely | H P | H + X(virtual) done; P NOT RUN |
| R12 | Daily rollover with players in different scenes | H | done (H) |
| R13 | Museum donations shared + persistent visible exhibits | H X S | done |
| R14 | Keyboard single-player still works | X P | X done, P open |
| R15 | Tool feedback, occlusion fade, night lighting rendered | S | occlusion, night, lamp done; tool FX: cast splash ring S verified (fx/fx_cast_splash.png); net swish visible but small; dig puff S verified after recolour (p3m/p3_dig_site.png) -> done |
| R16 | Content coverage table kept current (DESIGN_SPEC §Content) | doc | done (table in DESIGN_SPEC, keep updated) |

Physical items (P) cannot be performed by the agent; they go to HUMAN_VERIFICATION.md.
- [x] Money projects: leader-only start, route validation, per-player donations, built at 06:00, save/reload — unit test (headless).
- [x] Policies: one active, fee/insufficient funds, leader-only, no stacking after reload, upkeep effect at rollover — unit test (headless).
- [x] Gullrock activities: co-op shared score, separate rewards, once/day, forfeits on leaving — unit test; Beacon Dash executable action path + screenshot inspected. Physical controller: NOT RUN.
- [x] Housing balance migration/independence/relaunch — unit test (headless).
- [x] Town Hall interior + civic desk — unit test + rendered screenshot (phase 25).
- [x] Construction preview valid/invalid, partial funding, built after rollover — rendered screenshots (phase 25).
- [x] Crab Corral co-op, Ring Swim with other player in town — executable action path + rendered screenshots (phases 26/27).
- [ ] Activity difficulty: human assessment PENDING. Physical controllers: PENDING.

- [x] M3 monthly surveys: independent registration/entries/prizes, measured captures, backward-compatible save migration and once-only payout across reload/rollover (H); plaza host and entry/prize screens inspected in split-screen (X/S phase 41).
- [x] M3 resident home invitations: persisted offers/receipts, independent arrival gifts, missed/declined/changed-resident and full-pocket safety (H); invitation, real door arrival, guide and missed follow-up inspected in split-screen (X/S phase 42).
- [x] M3 Trail Seek: real shared resident walking, independent progress/rewards, saved deadlines/receipts, indoor/offline expiry, rollover, blocked starts, protected home appointments and full-pocket mail (H); accepted offer, normal talk finds/completion and real door entry followed by timeout inspected in split-screen (X/S phase 43).
- [x] M3 full v4-to-v5 two-profile migration preserves the entire world and possessions while defaulting all added activity fields (H). Final release suite: 93 passed, 0 failed, 2 ignored; no new ignored tests.
- [x] M3 final day/dusk visual regression comparison against accepted round-2 curtains baseline: eight views at both times inspected; no visible regression (S phase 40, `test_output/curtains_r2_vs_game3_regression_*`). Temporal flicker and measured frame rate are not established by still screenshots.
