# Hollowbrook architecture

A map for anyone (human or AI) changing this code. Read this before opening the large files.
Rust + macroquad 0.4, one binary, no engine. Run the full check before every commit: `bash tools/check.sh`.

## Files

| File | What lives there |
|---|---|
| `src/main.rs` | Window setup, title screen, the frame loop, saving, and all scripted autotest phases (the dev bot). |
| `src/world.rs` | Core state types (`World`, `Profile`, `Player`, `Game`, `Tg`, `Ui`), constants, `activate`, saving glue, `Game::check()`, movement and scenes. |
| `src/world/*.rs` | `impl Game` split by system: `daily` (rollover), `interact` (targets, prompts, talk), `housing`, `furniture`, `shop` (+ museum), `tools` (+ fishing, diving), `home_garden`, `festivals`, `projects`, `islet`, `frontier` (Dustreach outpost, bounties), `extras` (campground, initiatives, surprises), `sim` (residents, wildlife, paths), `input` (update/tick, menus, placement). |
| `src/events.rs` | M3 activities: monthly contests ("surveys"), resident invitations, hide-and-seek ("Trail Seek"). |
| `src/data.rs` | Static content: item table `DEFS` and named ids `data::id`, buildings `BLDS`, residents, rooms, dialogue, guide/help text. |
| `src/render.rs` | Colours, the CPU mesh builder `R`, lighting. Never changes state. |
| `src/render/*.rs` | `characters`, `models` (furniture, buildings), `scene` (viewport `draw`, town, islet), `interior` (`room`), `hud` (all 2D menus), `frontier` (Dustreach scene, bounty HUD and board menu). |
| `src/save.rs` | JSON save/load, backups, corrupt-save handling, version upgrades (`upgrade`), legacy text-save migration. |
| `src/pad.rs` | Gamepads (gilrs) mapped onto the same virtual keys as the keyboard. |
| `src/net.rs` | LAN play: the host simulates, the friend sends input and receives snapshots. |
| `src/audio.rs` | Procedural music and sound effects. |
| `src/clock.rs` | Real-time clock, overridable with `HOLLOWBROOK_DEV_TIME`. |
| `src/tests.rs` + `src/tests/*.rs` | Headless tests by topic; shared helpers in `tests.rs`; `quality.rs` holds the stress test and guards. |
| `tools/check.sh` | fmt check + clippy correctness + tests (run before every commit). |
| `tools/shots.sh check` / `save` | Visual regression check: screenshots of phases 1, 30, 40, 44, 45, 48 (fixed randomness) compared with a saved baseline; changed images get before/after/difference strips in `test_output/shots_diff/`. |
| `tools/quick.sh`, `tools/quickshot.sh` | Seconds-fast loop: incremental `quick` profile tests (optional name filter) / one autotest phase screenshot into `test_output/quickshot/`. |
| `tools/ui_audit.sh` | Screenshots of every autotest phase as contact sheets (run from `test_output/`). |

## State: who owns what

```
Game
├── w: World                 saved; shared town (trees, ground items, shop stock, donations, projects, garden, day…)
│   ├── p: Profile           saved; the ACTIVE player's personal data (money, pockets, storage, home furniture…)
│   └── others: Vec<Profile> saved; the other players' profiles
├── me: Player               runtime only; the ACTIVE player (scene, position, facing, open menu `ui`, messages…)
├── others: Vec<Player>      runtime only; the other players
└── npcs, bugs, shadows, fx… runtime only; rebuilt each session
```

**The one gotcha to remember:** `self.w.p` and `self.me` always mean *the active player*.
`Game::activate(id)` swaps that player's `Profile` and `Player` into those slots. Every per-player function
(`update_player`, buying, placing furniture, rendering a split-screen half) assumes `activate` was called first.
Code that must see every player iterates `std::iter::once(&self.w.p).chain(self.w.others.iter())`.

Home stories (basement/upstairs) use the same swap idea: `Profile.room` holds the story the owner is on;
the others wait in `Profile.stories`. `Game::set_story` swaps them; leaving the home always returns to story 0.

## One frame (`main.rs`)

1. Read input (keyboard, pads, LAN client) into one `Inp` per player.
2. `Game::tick(dt, now)` once: clock, daily rollover (`catch_up`), residents, wildlife, effects.
3. For each player: `activate(id)` then `update_player(&inp, dt)` (camera, UI, movement, actions).
4. Render each player's viewport: `activate(id)`, `render::draw(...)` (split-screen = two halves).
5. Autosave every 15 s, each new day, and on quit.

## Interaction model

`Game::target()` collects candidates near the player (`cand(pos, range, …, Tg::…)`) and picks the best one.
`Tg` is the list of everything you can press E on. Each `Tg` variant needs: a candidate (in `target`),
a label position (`tg_pos`), a prompt text (`tg_label`) and an action (`interact`). The compiler's exhaustive `match` will point at
any you forget. Menus are the `Ui` enum on `me.ui`; their input lives in `update_player`, their drawing in `render.rs`.

## Saving

- File: `hollowbrook_save.json` in the working directory (the real player save lives in the repo root; never run
  tests or autotests there, use `test_output/`). Backups rotate in `save_backups/`.
- `SAVE_VERSION` (world.rs) is the current version. `save::upgrade` upgrades older JSON step by step.
- **Adding a saved field:** add it with `#[serde(default)]` so old saves still load; add a migration in `upgrade`
  only if the default is wrong for old players; add a reload test (`reload(&g, "name")` in tests.rs).

## How to…

- **Add an item:** append to `DEFS` in data.rs with a new id (never renumber or reuse an id; saves store ids), then add
  its named constant to `data::id` (`id::CARP` instead of `14`). Older code still uses bare numbers; prefer `id::` in new code.
  `named_item_ids_match_defs` fails if a constant and the table drift apart.
- **Add something to press E on:** new `Tg` variant + `target`, `tg_pos`, `tg_label`, `interact`.
- **Add a menu:** new `Ui` variant, input handling in `update_player`, drawing in `render.rs`; keep it inside a
  640-px split-screen half (`ui_text_budgets` test guards the fixed text tables).
- **Add a rule that must never break:** add it to `Game::check()`; the stress test then checks it every frame.
- **Verify visuals:** add an autotest phase in `main.rs` (next free number), run it from `test_output/`, look at
  the PNGs. Use `test_output/tour.sh` for before/after comparisons of the town.

## Testing and bug finding

- `cargo test --release`: ~96 headless tests. Helpers in tests.rs: `g0(at(day,h,m,s))` one player,
  `two(...)` two players, `reload(&g, name)` save + load, `home(&mut g)` enter the home.
- `random_input_stress_keeps_invariants`: two players press random keys for 40,000 frames over ~3 game days;
  `Game::check()` must stay empty every frame and save/reload must keep possessions.
- Randomness uses macroquad's global generator, shared by tests running in parallel, so a test that depends on
  random placement can be flaky. Pin the state it needs (as tests do with `g.w.surprise = …`).
- Evidence types are kept separate in PROGRESS.md: headless (H), executable bot path (X), screenshot (S), physical (P).

## Autotest phases (`HOLLOWBROOK_AUTOTEST=. HOLLOWBROOK_AUTOTEST_PHASE=<n>`, run inside `test_output/<dir>/`)

| Phase | What it does (screenshots) |
|---|---|
| 0 | Plain scripted session, no screenshots |
| 1 | Core loop: outdoors, occlusion, beach, conversation, shop, selling, relaunch |
| 3 | Shovel, digging, fishing, bugs, museum, resident home; night + lamp (dev time at night) |
| 4 | Two players, one indoors one outdoors, P2 inventory |
| 5 | Virtual gamepads: prompts, disconnect/reconnect |
| 6 | Clothing rack, new outfit |
| 7 | Home expansion at the counter |
| 8 | Mailbox and letters |
| 9 | Weather near a resident |
| 10 | Catalog re-order |
| 11 | Gift to a resident |
| 12 | Harvest Fair stall |
| 13 | Swimming and diving |
| 14 | Resident visits the player's home |
| 15 | Tool effects (cast splash, net swish) |
| 16 | Residents moving in/out |
| 17 | Furniture tint, wallpaper, flooring |
| 18 | Hat and hairstyle |
| 19 | Community project: gather materials, build a bench |
| 20 | Islet trip in split-screen |
| 21 | Residents chatting with each other |
| 22 | Split-screen dialogue framing |
| 23 | Audio crossfade and mute (no screenshots) |
| 24 | Beacon Dash timed activity |
| 25 | Town Hall, money project placement and funding |
| 26 / 27 | Crab Corral co-op / Ring Swim solo |
| 28 | Art stall, art wing, furniture collections |
| 29 | Running dust, building trim, Town Hall exterior |
| 30 | Title screen, campground, kiosk, journal pages |
| 31 | Festival decorations, Friends page |
| 32 | Daily surprises and emotes |
| 33 | Renovated shop, smoke and birds |
| 34 | Riverside fences, regatta boats |
| 35 | Keeper menu, seeds, gardening |
| 36 | House repaint, home score |
| 37 / 38 | LAN host / LAN client |
| 40 | Visual tour, 8 fixed viewpoints (`test_output/tour.sh`) |
| 41 | Contests: registration, entry, prize (dev time Nov 11 10:00 / 17:01) |
| 42 | Invitations: offer, home visit, missed visit (Nov 12 12:00 / 13:15) |
| 43 | Hide-and-seek: start, finds, completion, timeout (Nov 13 12:00 / 16:00) |
| 44 | Home stories: main floor, basement, upstairs |
| 45 | Dustreach: arrival at the gate, bounty board menu, lookalike bounty, market (day and dusk) |
| 46 | Dustreach gear: buy at the board, stun pistol shot at a runner, stunned target |
| 47 | Woods path and door to Dustreach |
| 48 | Hunter life: full armour, companion, prisoner walking to the ship, guild master's forge |

## Glossary of short names

`w` world · `p` active profile · `me` active player · `Tg` interaction target · `BK` building kind ·
`BLDS` buildings · `DEFS`/`def(id)` item table · `Ui` open menu · `tc(tile)` tile centre · `ipos` indoor position ·
`hr`/`dom`/`mo`/`yr` hour/day of month/month/year · `fx` effects · `act` timed activity · `mproject` money project ·
`project` material project · `wtask` weekly town task · `story` home floor (0 main, 1 basement, 2 upstairs) · `fmap`/`Scene::Frontier` Dustreach outpost (dev entry F9 with `HOLLOWBROOK_DEBUG`; no door from town yet).

## Finding and fixing bugs (workflow)

1. Reproduce: write a failing headless test in the matching `src/tests/<topic>.rs` (helpers: `g0`, `two`, `reload`).
   If the bug is a broken rule (duplication, overflow, stuck state), add the rule to `Game::check()` instead: the
   random-input stress test then guards it for every future change.
2. Fix it, then `bash tools/quick.sh <topic>` (seconds).
3. Visual changes: `bash tools/quickshot.sh <phase>` to look, `bash tools/shots.sh check` to catch side effects elsewhere.
4. `bash tools/check.sh` must pass before committing (gate the commit on its exit code, never on grepped output).
