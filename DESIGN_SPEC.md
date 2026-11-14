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
