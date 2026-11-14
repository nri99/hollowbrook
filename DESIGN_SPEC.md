# Hollowbrook — Design Specification

Original real-time village-life game in Rust (macroquad). All names, characters, art and dialogue are original.
Status tags: **[IMPL]** implemented & verified (see PROGRESS.md for evidence type) · **[PART]** partially implemented ·
**[TODO]** required, not built · **[PROV]** provisional invented rule (not a verified requirement) ·
**[UNSPEC]** behaviour the requirements leave open; needs later review · **[OUT]** deliberately out of scope.

## 1. Pillars
- Persistent everyday life: the world changes between visits (stock, letters, plants, weeds, visitors). [PART]
- Self-directed play: collecting, decorating, befriending, earning, no win state. [PART]
- Soft pressure only: no hunger/health/rent/quota. Time limits come from hours, availability, schedules. [IMPL]

## 2. Time and calendar
- Real device clock, 1 real minute = 1 game minute; no acceleration; menus/interiors/bed never skip time. [IMPL]
- Calendar date changes at midnight; gameplay day rolls over at 06:00 (`day_id`). [IMPL]
- One authoritative `Clock`; dev-only `HOLLOWBROOK_DEV_TIME`. [IMPL]
- Daily refresh idempotent, persisted, runs once even after multi-day absence; elapsed-time rules per system. [IMPL]
- Clock moved backwards: daily processing pauses until the clock catches up; nothing is undone. [PROV]
- Seasonal presentation: grass/foliage colours by month. [PART] Smooth blending, snow cover, seasonal particles/sounds. [TODO]

## 3. New game and identity
- Name entry; first profile. [IMPL] Travel-conversation intro, appearance choice, layout choice. [OUT/UNSPEC — intro sequence declined; appearance choice TODO in P5]
- Up to four local human profiles sharing a town. [TODO, UNSPEC priority]

## 4. World
- One authored compact town: home, shop, 4 resident homes, reserved civic building, river, 2 bridges, beach, sea, trees, flowers, rocks, paths. [IMPL]
- Stable building ids (index in `BLDS`; 1 = player home). [IMPL]
- Water blocks walking; bridges walkable; buildings/trees/rocks collide; residents path-find around obstacles. [IMPL]
- Separate scenes with own rules (town, interiors). [IMPL] Other destinations. [TODO P7]
- No free terrain sculpting. [IMPL by design]

## 5. Movement, camera, interaction
- Analog-style 8-way normalized walking, running, facing. [IMPL] Running scares bugs. [IMPL] Running damages flowers. [TODO]
- Elevated third-person perspective camera, smooth follow, rolling-world curvature (dev setting `HOLLOWBROOK_CURVE`). [IMPL]
- Interaction targeting by distance, facing, line of sight; doors preferred when faced; indicator + prompt. [IMPL]
- Objects between camera and player must not hide the player entirely. [TODO P2-01]

## 6. Interiors
- 3D rooms, cut-away walls, fixed indoor camera, fade transitions, no curvature indoors. [IMPL]
- Player home 4×4 persistent room; shop with keeper, counter, stands, price tags; 4 themed resident homes (enterable when the resident is home). [IMPL]
- Larger home rooms through housing progression. [TODO P5-01]

## 7. Furniture
- Catalog: chair, sofa, table, bed, lamp, cabinet (storage), potted plant, painting (wall). [IMPL]
- Definitions carry footprint, rotation, wall/floor surface, collision, interaction, price, resale value. [IMPL]
- Buy, carry, place, move, rotate (quarter turns), pick up, sit, lie (no time skip), lamp toggle, storage. [IMPL]
- Reject overlap / out of room / doorway / standing tile / exit blocked. [IMPL]
- Item definition id vs instance uid vs location kept separate; no duplication while moving. [IMPL]

## 8. Gathering and economy
- Fruit grows, shake → drops → pick up → sell; regrow after 3 gameplay days. [IMPL] (3 days = [PROV])
- Shore collectibles respawn daily to 4. [IMPL] (count = [PROV])
- Digging: shovel, persistent holes, fill, 2 daily dig sites with buried treasure. [IMPL headless; rendered TODO P1-02]
- Fishing: rod, shadows by habitat/availability, approach, nibbles, bite window, catch/escape. [IMPL headless; rendered TODO P1-01]
- Bugs with net. [IMPL headless; rendered TODO P1-05]
- Prices, rarity weights, stack size 10, pocket size 16, storage 40. [PROV]

## 9. Shopping
- Five furniture stock slots refreshed at 06:00; sold-out persists until next refresh. [IMPL]
- Opening hours 08:00–22:00 separate from refresh. [IMPL] (hours = [PROV])
- Confirm, money check, pocket check, cancel without mutation, no double purchase. [IMPL]
- Tools sold once each (shovel, rod). [IMPL] Shop upgrades/progression. [TODO P5]

## 10. Residents
- Four original residents (fox, badger, frog, otter) with distinct silhouettes, clothes, personalities. [IMPL]
- Schedules: wake/sleep/home hours [PROV]; wander with BFS, pauses, face player while talking. [IMPL]
- Context dialogue (time of day, rain, home), no immediate repeats, requests with visible reward reaction. [IMPL]
- Relationship: +1 first chat per day, +3 per request [PROV — placeholder model, not final]. Letters with coin gifts at friendship ≥10 [PROV].
- Idle activities, umbrellas/weather behaviour, moving in/out. [TODO P6]

## 11. Atmosphere
- Day/dusk/night lighting from the real clock; lit windows at night; indoor lamp lighting. [IMPL; night rendered check TODO P1-04]
- Rain (deterministic per day) [PROV]; other weather [TODO P7-02].
- Procedural sound effects (steps, blips, splash, coin, dig, rustle, door). [IMPL] Music [TODO, UNSPEC].

## 12. Architecture and saves
- Modules: clock, data, world, render, save, tests. [IMPL]
- JSON save v3; explicit migration from v1/v2 text saves; legacy file kept. [IMPL] Automatic backups before migration/overwrite. [TODO P0-01]
- Resume outdoors (outside home if saved indoors). [PROV]

## 13. Collections & museum (P3) — [TODO]
Donate fish, bugs, fossils; persistent exhibit display; completion records; fossil identification step.

## 14. Civic systems (P4) — [OUT pending user decision]
Leader approval, donation-funded public projects and town-wide policies were part of the original spec, but in this
project they were declined as a recreation-sensitive feature set. Any civic layer needs an original design approved by the user.

## 15. Housing, clothing, catalog (P5) — [TODO]
Staged home expansion, clothing purchase/equip, reorder catalog, shop growth.

## 16. Events and destinations (P7) — [TODO / OUT]
Generic recurring seasonal events: TODO. Specific tropical-island tour/medal systems, figure-scanning visitors,
built-in third-party-style minigames and save-import from another game: [OUT].

## Unspecified, needs review
Exact prices and probabilities; friendship thresholds; relocation rules; multi-profile UI; music direction; event calendar.

## Content coverage (R16) — counts in the build vs. specification
Status: COMPLETE = system done with the content listed · PARTIAL = system works, content is a small sample ·
NOT IMPLEMENTED · BLOCKED (needs user decision). The spec provides no authoritative catalogs, so every count below is an
original sample, not a claim of full coverage.

| Area | In build | Status | Missing data / gaps |
|---|---|---|---|
| Bugs | 8 species, month/hour windows | PARTIAL | full roster, weather/location rules |
| Fish | 11 (8 river, 3 sea), habitat + hours | PARTIAL | ponds, river mouth, shadow sizes per species |
| Sea creatures / swimming / diving | swim in sea, dive at bubbles, 4 sea creatures | PARTIAL | river swimming, larger roster, season rules |
| Fossils | 4 + unknown (assessment at museum) | PARTIAL | full roster, multi-part fossils |
| Art | — | NOT IMPLEMENTED | art exhibits, authenticity |
| Museum exhibits | fish tank, bug case, 4 fossil pedestals, shared donations | PARTIAL | per-species exhibit layouts, art wing |
| Fruit / shells / treasures | 4 / 3 / 2 | PARTIAL | non-native fruit, special shells |
| Furniture | 8 items, 4 tints, 4 wallpapers, 4 floors | PARTIAL | sets, themes, rugs, more items |
| Clothing / appearance | 4 shirts, 3 hats, 4 hairstyles | PARTIAL | trousers, shoes, accessories, face options, custom designs |
| Shops | general store (5 daily stock, 2 tools, rack, counter, catalog) | PARTIAL | additional shops, upgrades, seasonal stock |
| Housing | 3 sizes (paid up front), storage 40 | PARTIAL | extra rooms, exterior customization; loans BLOCKED |
| Residents | 4 + shopkeeper + curator; schedules, requests, gifts, letters, home visits | PARTIAL | more personalities, birthdays (moving in/out: provisional rules implemented) |
| Weather | clear / rain / snow (deterministic, provisional) | PARTIAL | storms, fog, seasonal particles beyond snow |
| Events | Harvest Fair (provisional window 13–16 Nov) | PARTIAL | full annual calendar (not specified -> fidelity gap) |
| Visiting characters / entertainment | — | NOT IMPLEMENTED | spec gives no roster or rules |
| Local players | 2 profiles, split-screen, 2 pads | COMPLETE (physical input unverified) | up to 4 profiles (spec) not implemented |
| Civic permit / projects / policies | — | BLOCKED | original design needed (user decision) |
| Tropical travel / medals / reward shop | — | BLOCKED | original design needed (user decision) |
| Campground / initiatives | — | BLOCKED | original design needed (user decision) |
| Music | — | NOT IMPLEMENTED | direction unspecified |

## Original expansion — product decisions (approved directions; values are initial balance choices, not reference rules)
- **Housing (revised):** expansion adds an interest-free per-player balance (EXPAND_COST in world.rs: 3000, 15000; HOME_DIMS 4x4, 5x5, 6x5), paid at the shop counter, no deadline; the next expansion unlocks when it is 0. Older saves: rooms kept, balance 0.
- **Community projects:** one active project at a time, proposed by a resident at the daily refresh (or at load if none). Cycle: bench -> lamp post -> planter -> bridge (bridge once). Costs: bench 5 Branch + 2 Stone; lamp 3 Branch + 4 Stone + 2 shells; planter 4 Stone + 3 fruit; bridge 12 Branch + 8 Stone. Branch: 1 per tree per day when shaken; Stone: 1 per rock per day with the shovel. Sites come from authored candidate lists and are rejected if they would block any building route.
- **Liveliness:** 0-100 = 10 x built projects + 2 x museum donations + flowers/2 - 3 x weeds (Quiet < 20 <= Cozy < 45 <= Lively < 70 <= Bustling).
- **Gullrock Islet:** Captain Pell (pelican) at the beach dock; fare 200 coins out, return free (confirm dialogs). Separate 16x14 scene (sand, grass, palms, rocks, tide pool, dock). Daily activity alternates: even days gather 5 Driftwood, odd days find 3 Moon Shells; hand in to Pell for 3 Sea Glass, once per player per day. Islet-only wildlife: Palm Crab (pickup), Glass Shrimp (diving). Sea Glass stall: Driftwood Stool (4), Glass Float (6). Sea Glass is never sold for coins. Quitting on the islet resumes at the town dock.
- **Social depth:** residents sometimes stroll toward each other and chat for 5 s (speech bubbles from per-personality CHATTER). Birthdays per character (BDAYS; e.g. Otto 16 Nov): letter to everyone that morning, birthday line, gifts count as liked +3 bonus. 1 in 5 daily requests is a parcel delivery to another resident: 300 coins + friendship +2 with both residents.
- **Atmosphere:** original procedural music (pentatonic melody + bass; day 96 bpm, evening 80, night 64; normalized to 0.8 peak, looped at 50% volume), footsteps per surface (grass/path/sand/wood/indoor floor/water), screen-space seasonal particles (spring petals, autumn leaves, summer-night fireflies), dialogue box fitted to each split-screen view.
- Public construction (money, provisional): MPROJECTS in data.rs (bench 8000, lamp post 12000, planter 10000, fountain 40000, flower arch 25000). Town leader = player 1 (LEADER), chooses at the notice board and places with a green/red preview; site must be free grass keeping every door and the dock reachable. Anyone donates own coins (1000 per press) at the site sign; built at the next 06:00 once fully funded. Material projects remain separate.
- Policies (provisional): POLICIES in data.rs; one active; leader pays fee from own coins at the civic desk inside the Town Hall (Administrator Hazel Quill). Dawn Hours 12000 (store opens 6:00, residents wake 2h earlier except Bramble/Basil), Lantern Hours 12000 (store closes 24:00, residents stay up/home 2h later except Otto), Tidy Commons 15000 (no new daily weeds). Repeal free. Effects derived at runtime, never stored.
- Gullrock activities (provisional): ACTS in data.rs. Crab Corral 60s (2/4/6 crabs penned), Beacon Dash 45s (5/9/13 beacons), Ring Swim 75s (2/4/6 rings in order). Every player on Gullrock joins and shares the team score; each participant earns 1/2/4 Sea Glass into their own pockets, once per activity per day. Leaving the islet forfeits; activity state is not saved.
- Town Hall (2026-11-14): enterable 7x5 interior with Administrator Hazel Quill at the civic desk (policies, start a funded project, status). Outdoor notice board is read-only. Activity thresholds are invented and HUMAN DIFFICULTY ASSESSMENT IS PENDING; they are not tuned against the autotest bot. Players arriving at Gullrock mid-activity join it; Sea Glass that doesn't fit in pockets arrives by letter.
- X-04 (provisional): Meadow set (Chair 900, Table 1100, Lamp 1000) and Harbor set (Bench 1600, Crate 1300, Lantern 1200) join the daily stock pool; completing a set in your home shows a note. Critters: Rain Snail (bug, Mar-Nov, rain only), Dune Beetle (bug, Gullrock only, 8-20h), Ice Smelt (river fish, Dec-Feb), Brackish Goby (river-mouth fish). Six original procedural paintings; Saturday art stall at the fair spot (not during the fair) sells the week's painting for 2500, one per player per week; donated paintings hang in the museum art wing. Exhibit labels appear near museum exhibits.
- X-04 reconciliation (2026-11-14): no written record set counts; delivered at the larger counts the user cited: 6 pieces per set (Meadow adds Bed 2200, Planter 700, Wall Art 1000; Harbor adds Stool 800, Buoy Light 1400, Bunk 2400) and 8 paintings (adds Bramble's Garden, Harbor Fog). Art stall rule: open on Saturdays except Harvest Fair days (event restriction: both use the same STALL tile; the fair takes priority). One painting per player per week (Profile.art_week, saved).
- Release 1 balance (original, provisional; see RELEASE_NOTES.md): Campground goods at their catalog price (one unit each per day, shared); favors want 2 Branches / 1 Sardine / 3 native fruit / 2 Stones and pay a unique keepsake once per player per visitor. Initiatives: 14 daily templates worth 2-4 coupons, assigned only when attainable (tool owned, enough weeds/fruit trees/coins); 5 weekly town templates worth 12 coupons per player. Kiosk: Town Pennant 6, Lantern Post 10, Reading Nook Sofa 16, Keepsake Chest 12, two tees 5 each, Ferry Pass 4 (one free trip), Sea Glass Pouch 8 (3 Sea Glass). Coupon rewards resell at 1/4 of catalog value, so coupons cannot become unbounded coins. One non-native fruit washes up on Gullrock daily (2 pieces) so all four fruits are obtainable.
