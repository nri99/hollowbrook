//! Static content: item definitions, the authored town, buildings, residents, rooms, dialogue.
use macroquad::prelude::*;

pub const W: usize = 40;
pub const H: usize = 30;
pub const GRASS: u8 = b'.';
pub const PATH: u8 = b',';
pub const SAND: u8 = b':';
pub const RIVER: u8 = b'~';
pub const SEA: u8 = b'w';
pub const BRIDGE: u8 = b'=';
pub const TREE: u8 = b'T';
pub const PINE: u8 = b'P';
pub const ROCK: u8 = b'R';
pub const FLOWER: u8 = b'*';
pub const BLD: u8 = b'#';

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum K { Bug, Fish, Fruit, Shell, Treasure, Fossil, Furn, Tool, Cloth }
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act { None, Sit, Bed, Lamp, Storage }

/// Item definition. `buy` = shop price, `sell` = resale value. Furniture: `fp` footprint (w,d), `rot` quarter-turns
/// allowed, `wall` = hangs on the back wall (otherwise floor + solid), `act` = interaction.
pub struct D { pub id: u16, pub name: &'static str, pub k: K, pub buy: u32, pub sell: u32, pub col: (f32, f32, f32), pub size: f32, pub hab: u8, pub m: (u32, u32), pub h: (u32, u32), pub fp: (i32, i32), pub rot: bool, pub wall: bool, pub act: Act }
const fn b(id: u16, name: &'static str, k: K, sell: u32, col: (f32, f32, f32)) -> D { D { id, name, k, buy: 0, sell, col, size: 0.4, hab: 0, m: (1, 12), h: (0, 24), fp: (1, 1), rot: false, wall: false, act: Act::None } }
#[allow(clippy::too_many_arguments)]
const fn cr(id: u16, name: &'static str, k: K, sell: u32, col: (f32, f32, f32), size: f32, hab: u8, m0: u32, m1: u32, h0: u32, h1: u32) -> D { D { size, hab, m: (m0, m1), h: (h0, h1), ..b(id, name, k, sell, col) } }
const fn fu(id: u16, name: &'static str, buy: u32, col: (f32, f32, f32), fp: (i32, i32), rot: bool, wall: bool, act: Act) -> D { D { buy, fp, rot, wall, act, ..b(id, name, K::Furn, buy / 4, col) } }
const fn cl(id: u16, name: &'static str, buy: u32, col: (f32, f32, f32)) -> D { D { buy, ..b(id, name, K::Cloth, buy / 4, col) } }
const fn tl(id: u16, name: &'static str, buy: u32) -> D { D { buy, ..b(id, name, K::Tool, 0, (0.6, 0.45, 0.3)) } }

pub static DEFS: [D; 48] = [
    cr(0, "Cabbage White", K::Bug, 90, (1., 1., 0.95), 0.2, 0, 3, 10, 6, 17), cr(1, "Honeybee", K::Bug, 160, (1., 0.85, 0.2), 0.2, 0, 3, 7, 8, 17),
    cr(2, "Ladybug", K::Bug, 120, (0.9, 0.15, 0.1), 0.2, 0, 3, 6, 8, 17), cr(3, "Dragonfly", K::Bug, 230, (0.3, 0.8, 0.9), 0.2, 0, 5, 10, 8, 19),
    cr(4, "Cicada", K::Bug, 300, (0.4, 0.6, 0.3), 0.2, 0, 7, 8, 8, 17), cr(5, "Firefly", K::Bug, 450, (0.8, 1., 0.3), 0.2, 0, 6, 7, 19, 4),
    cr(6, "Moth", K::Bug, 130, (0.85, 0.8, 0.65), 0.2, 0, 1, 12, 19, 4), cr(7, "Pill Bug", K::Bug, 60, (0.5, 0.5, 0.55), 0.2, 0, 1, 12, 6, 23),
    cr(10, "Minnow", K::Fish, 70, (0.6, 0.65, 0.7), 0.25, 1, 1, 12, 0, 24), cr(11, "Bluegill", K::Fish, 120, (0.35, 0.5, 0.75), 0.35, 1, 1, 12, 9, 16),
    cr(12, "Carp", K::Fish, 250, (0.65, 0.55, 0.3), 0.5, 1, 1, 12, 0, 24), cr(13, "Trout", K::Fish, 400, (0.75, 0.6, 0.6), 0.5, 1, 3, 11, 0, 24),
    cr(14, "Catfish", K::Fish, 800, (0.4, 0.4, 0.35), 0.7, 1, 5, 10, 16, 9), cr(15, "Eel", K::Fish, 1200, (0.3, 0.3, 0.25), 0.6, 1, 6, 9, 16, 9),
    cr(16, "Pike", K::Fish, 1800, (0.45, 0.55, 0.35), 0.75, 1, 9, 12, 0, 24), cr(17, "Koi", K::Fish, 3000, (1., 0.5, 0.3), 0.55, 1, 1, 12, 16, 9),
    cr(30, "Sardine", K::Fish, 100, (0.7, 0.75, 0.85), 0.25, 2, 1, 12, 0, 24), cr(31, "Sea Bass", K::Fish, 400, (0.5, 0.55, 0.6), 0.8, 2, 1, 12, 0, 24),
    cr(32, "Pufferfish", K::Fish, 900, (0.85, 0.8, 0.5), 0.45, 2, 6, 9, 0, 24),
    b(20, "Apple", K::Fruit, 100, (0.85, 0.15, 0.15)), b(21, "Pear", K::Fruit, 100, (0.8, 0.85, 0.25)), b(22, "Cherry", K::Fruit, 100, (0.7, 0.05, 0.2)), b(23, "Peach", K::Fruit, 100, (1., 0.65, 0.5)),
    b(40, "Scallop", K::Shell, 80, (0.98, 0.82, 0.72)), b(41, "Conch", K::Shell, 250, (0.98, 0.62, 0.55)), b(42, "Sand Dollar", K::Shell, 120, (0.92, 0.9, 0.78)),
    b(45, "Amber Stone", K::Treasure, 600, (0.95, 0.6, 0.1)), b(46, "Old Coin", K::Treasure, 1000, (0.85, 0.75, 0.3)),
    b(47, "Unknown Fossil", K::Fossil, 100, (0.78, 0.72, 0.6)), b(43, "Fern Fossil", K::Fossil, 700, (0.55, 0.6, 0.45)), b(44, "Shark Tooth", K::Fossil, 900, (0.95, 0.93, 0.85)),
    b(48, "Ammonite", K::Fossil, 1100, (0.8, 0.65, 0.45)), b(49, "Trilobite", K::Fossil, 1300, (0.5, 0.45, 0.4)),
    fu(50, "Chair", 600, (0.75, 0.5, 0.3), (1, 1), true, false, Act::Sit), fu(51, "Sofa", 1200, (0.35, 0.5, 0.78), (2, 1), true, false, Act::Sit),
    fu(52, "Table", 800, (0.66, 0.46, 0.3), (1, 1), false, false, Act::None), fu(53, "Bed", 1800, (0.55, 0.7, 0.9), (1, 2), true, false, Act::Bed),
    fu(54, "Lamp", 700, (0.95, 0.88, 0.65), (1, 1), false, false, Act::Lamp), fu(55, "Cabinet", 1500, (0.58, 0.38, 0.24), (1, 1), true, false, Act::Storage),
    fu(56, "Potted Plant", 500, (0.3, 0.62, 0.3), (1, 1), false, false, Act::None), fu(57, "Painting", 900, (0.4, 0.62, 0.88), (1, 1), false, true, Act::None),
    cl(63, "Sky Tee", 400, (0.45, 0.7, 0.95)), cl(64, "Sunflower Tee", 400, (0.98, 0.82, 0.3)), cl(65, "Forest Hoodie", 650, (0.25, 0.5, 0.32)), cl(66, "Berry Sweater", 650, (0.75, 0.25, 0.45)),
    tl(60, "Net", 0), tl(61, "Fishing Rod", 400), tl(62, "Shovel", 500),
];
pub const FRUITS: [u16; 4] = [20, 21, 22, 23];
pub const FURN_IDS: [u16; 8] = [50, 51, 52, 53, 54, 55, 56, 57];
pub fn def(id: u16) -> &'static D { DEFS.iter().find(|d| d.id == id).unwrap_or(&DEFS[0]) }
pub fn is_def(id: u16) -> bool { DEFS.iter().any(|d| d.id == id) }
fn inr(v: u32, (a, b): (u32, u32), incl: bool) -> bool { let hi = if incl { v <= b } else { v < b }; if a <= b { v >= a && hi } else { v >= a || hi } }
pub fn avail(d: &D, m: u32, h: u32) -> bool { inr(m, d.m, true) && inr(h, d.h, false) }
/// Floor tiles covered by a furniture instance (footprint swaps axes on odd quarter turns).
pub fn foot(id: u16, x: i32, z: i32, rot: u8) -> Vec<(i32, i32)> {
    let (mut w, mut d) = def(id).fp; if rot % 2 == 1 { std::mem::swap(&mut w, &mut d) }
    (0..d).flat_map(|j| (0..w).map(move |i| (x + i, z + j))).collect()
}
pub fn rainy(d: i64) -> bool { (d as u64).wrapping_mul(2654435761) >> 8 & 7 == 0 }
/// Provisional content choice for this milestone: shop open 08:00-22:00; stock refresh is separate (06:00).
pub fn shop_open(hr: u32) -> bool { (8..22).contains(&hr) }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BK { Shop, Home(usize), Res(usize), Civic, Museum }
pub struct Bld { pub x: i32, pub z: i32, pub w: i32, pub d: i32, pub kind: BK, pub name: &'static str, pub wall: (f32, f32, f32), pub roof: (f32, f32, f32) }
/// Stable building ids = index. 1 = player 1's home (same id as earlier saves), 7 = player 2's home.
pub fn home_of(player: usize) -> usize { if player == 0 { 1 } else { 7 } }
pub const BLDS: [Bld; 9] = [
    Bld { x: 4, z: 3, w: 5, d: 4, kind: BK::Shop, name: "General Store", wall: (0.96, 0.9, 0.78), roof: (0.25, 0.55, 0.7) },
    Bld { x: 12, z: 12, w: 3, d: 3, kind: BK::Home(0), name: "Player 1's Home", wall: (0.98, 0.95, 0.88), roof: (0.78, 0.32, 0.3) },
    Bld { x: 4, z: 12, w: 3, d: 3, kind: BK::Res(0), name: "Juniper's House", wall: (0.98, 0.88, 0.75), roof: (0.92, 0.52, 0.2) },
    Bld { x: 29, z: 4, w: 3, d: 3, kind: BK::Res(1), name: "Bramble's House", wall: (0.85, 0.85, 0.85), roof: (0.4, 0.4, 0.52) },
    Bld { x: 31, z: 13, w: 3, d: 3, kind: BK::Res(2), name: "Mossy's House", wall: (0.88, 0.95, 0.82), roof: (0.35, 0.65, 0.35) },
    Bld { x: 17, z: 4, w: 3, d: 3, kind: BK::Res(3), name: "Otto's House", wall: (0.85, 0.9, 0.98), roof: (0.3, 0.45, 0.82) },
    Bld { x: 10, z: 3, w: 5, d: 4, kind: BK::Civic, name: "Town Hall", wall: (0.92, 0.86, 0.74), roof: (0.52, 0.3, 0.25) },
    Bld { x: 16, z: 12, w: 3, d: 3, kind: BK::Home(1), name: "Player 2's Home", wall: (0.9, 0.96, 0.9), roof: (0.3, 0.6, 0.5) },
    Bld { x: 33, z: 5, w: 4, d: 3, kind: BK::Museum, name: "Museum", wall: (0.93, 0.92, 0.88), roof: (0.45, 0.5, 0.6) },
];
/// Outdoor tile directly in front of the door.
pub fn door_tile(b: usize) -> (i32, i32) { let b = &BLDS[b]; (b.x + b.w / 2, b.z + b.d) }
pub fn door_stand(b: usize) -> Vec2 { let (x, z) = door_tile(b); vec2(x as f32 + 0.5, z as f32 + 0.5) }
pub fn door_face(b: usize) -> Vec2 { let (x, z) = door_tile(b); vec2(x as f32 + 0.5, z as f32) }
pub fn room_dims(b: usize) -> (i32, i32) { if matches!(BLDS[b].kind, BK::Shop | BK::Museum) { (7, 5) } else { (4, 4) } }
/// Museum interior (7x5): curator desk, fish tank (left), bug case (right), four fossil pedestals.
pub const MUSEUM_DESK: [(i32, i32); 3] = [(2, 1), (3, 1), (4, 1)];
pub const MUSEUM_TANK: [(i32, i32); 3] = [(0, 1), (0, 2), (0, 3)];
pub const MUSEUM_CASE: [(i32, i32); 3] = [(6, 1), (6, 2), (6, 3)];
pub const MUSEUM_PED: [((i32, i32), u16); 4] = [((1, 0), 48), ((5, 0), 49), ((1, 3), 43), ((5, 3), 44)];
pub const FOSSILS: [u16; 4] = [43, 44, 48, 49];

pub struct Res { pub name: &'static str, pub species: &'static str, pub fur: (f32, f32, f32), pub cloth: (f32, f32, f32), pub pers: usize, pub wake: u32, pub sleep: u32, pub bld: usize, pub home_h: [(u32, u32); 2] }
pub const RES: [Res; 4] = [
    Res { name: "Juniper", species: "fox", fur: (0.93, 0.5, 0.2), cloth: (0.2, 0.65, 0.6), pers: 0, wake: 7, sleep: 22, bld: 2, home_h: [(12, 13), (20, 22)] },
    Res { name: "Bramble", species: "badger", fur: (0.55, 0.55, 0.58), cloth: (0.62, 0.2, 0.25), pers: 1, wake: 9, sleep: 23, bld: 3, home_h: [(13, 14), (21, 23)] },
    Res { name: "Mossy", species: "frog", fur: (0.45, 0.75, 0.35), cloth: (0.95, 0.8, 0.3), pers: 2, wake: 8, sleep: 21, bld: 4, home_h: [(11, 12), (19, 21)] },
    Res { name: "Otto", species: "otter", fur: (0.55, 0.38, 0.25), cloth: (0.25, 0.4, 0.8), pers: 3, wake: 6, sleep: 20, bld: 5, home_h: [(14, 15), (18, 20)] },
];
pub fn res_spot() -> Vec2 { vec2(1.5, 1.6) }
/// Fixed furnishings of resident homes: (def, x, z, rot); z = -1 hangs on the back wall.
pub const TEMPL: [&[(u16, i32, i32, u8)]; 4] = [
    &[(53, 0, 0, 0), (52, 2, 1, 0), (50, 3, 1, 3), (56, 3, 0, 0), (57, 1, -1, 0)],
    &[(53, 3, 0, 0), (55, 0, 0, 0), (54, 1, 0, 0), (52, 0, 2, 0)],
    &[(53, 0, 1, 0), (56, 3, 0, 0), (56, 2, 0, 0), (54, 3, 2, 0), (57, 1, -1, 0)],
    &[(53, 3, 1, 0), (51, 0, 0, 0), (52, 0, 2, 0), (57, 2, -1, 0)],
];
pub const THEME: [((f32, f32, f32), (f32, f32, f32)); 4] = [((0.85, 0.6, 0.4), (0.98, 0.86, 0.7)), ((0.5, 0.5, 0.56), (0.76, 0.79, 0.86)), ((0.55, 0.7, 0.45), (0.86, 0.95, 0.78)), ((0.45, 0.55, 0.76), (0.83, 0.9, 0.98))];
/// Shop interior (7x5): five daily-stock stands, two tool stands, counter, keeper behind it.
pub const DISPLAYS: [(i32, i32); 7] = [(0, 1), (0, 3), (6, 1), (6, 3), (1, 3), (5, 3), (5, 1)];
pub const TOOL_DISPLAY: [u16; 2] = [62, 61];
pub const COUNTER: [(i32, i32); 3] = [(2, 1), (3, 1), (4, 1)];
/// Clothing rack in the shop (always stocked; provisional).
pub const RACK: (i32, i32) = (1, 1);
pub const CLOTHES: [u16; 4] = [63, 64, 65, 66];
pub const KEEPER: [&str; 2] = ["Welcome to the General Store! New furniture arrives every morning at 6.", "Sell your finds at the counter. We're open from 8:00 until 22:00."];

fn put(m: &mut [u8], x: i32, z: i32, c: u8) { if x >= 0 && z >= 0 && (x as usize) < W && (z as usize) < H { m[z as usize * W + x as usize] = c } }
fn path(m: &mut [u8], x: i32, z: i32) { let i = z as usize * W + x as usize; m[i] = match m[i] { RIVER => BRIDGE, BLD | BRIDGE => m[i], _ => PATH } }
/// The authored demonstration town.
pub fn town() -> Vec<u8> {
    let mut m = vec![GRASS; W * H];
    for z in 0..H as i32 { for x in 0..W as i32 { put(&mut m, x, z, if z >= 26 { SEA } else if z >= 24 { SAND } else if x == 0 || z == 0 || x == W as i32 - 1 { PINE } else { GRASS }) } }
    for z in 1..26 { let cx = if z <= 12 { 24 } else { 25 }; put(&mut m, cx, z, RIVER); put(&mut m, cx + 1, z, RIVER); if z == 13 { put(&mut m, 24, z, RIVER) } }
    for b in &BLDS { for z in b.z..b.z + b.d { for x in b.x..b.x + b.w { put(&mut m, x, z, BLD) } } }
    for x in 1..W as i32 - 1 { path(&mut m, x, 9); path(&mut m, x, 17) }
    for z in 9..=17 { path(&mut m, 9, z); path(&mut m, 20, z) }
    for i in 0..BLDS.len() { let (dx, dz) = door_tile(i); if dz < 9 { for z in dz..9 { path(&mut m, dx, z) } } else { for z in dz..=17 { path(&mut m, dx, z) } } }
    for z in 17..24 { path(&mut m, 13, z) }
    let feat: [(u8, &[(i32, i32)]); 4] = [
        (TREE, &[(2, 4), (2, 7), (3, 11), (8, 11), (15, 2), (21, 2), (22, 6), (16, 11), (21, 13), (2, 20), (8, 20), (17, 21), (22, 20), (28, 11), (35, 10), (36, 3), (33, 2), (29, 20), (36, 20), (10, 13)]),
        (PINE, &[(27, 6), (37, 13), (11, 21), (23, 15), (3, 22), (38, 7)]),
        (ROCK, &[(15, 6), (27, 2), (35, 15), (6, 21), (19, 14), (37, 22)]),
        (FLOWER, &[(3, 6), (7, 2), (14, 10), (15, 10), (16, 8), (11, 15), (15, 14), (19, 11), (3, 14), (7, 14), (28, 15), (34, 11), (36, 6), (30, 22), (27, 21), (10, 22), (19, 22), (16, 14)]),
    ];
    for (c, list) in feat { for &(x, z) in list { if m[z as usize * W + x as usize] == GRASS { put(&mut m, x, z, c) } } }
    m
}

// ---- dialogue (original, authored; two+ distinct personality styles) ----
pub const GREET: [[&str; 4]; 4] = [
    ["Morning, {p}! The air smells like {fs} today!", "Hi hi, {p}! Isn't this afternoon just perfect?", "Evening, {p}! The sky's going all peachy.", "Oh! {p}! You're up late too?"],
    ["Hmph. Morning, {p}. Too bright out.", "Afternoon. Mind my flowers, {p}.", "Evening, {p}. Finally, some quiet.", "It's the middle of the night, {p}. Go home."],
    ["Mmm... morning, {p}. I dreamed I was a cloud.", "Oh, hello {p}... the pond is humming today.", "Evening, {p}. The stars are warming up.", "{p}... the night smells like moss."],
    ["Morning, {p}! Already swam two laps!", "Hey {p}! Perfect weather for a sprint!", "Evening, {p}! Cooldown stretches, join me!", "Late-night training, {p}? Respect!"],
];
pub const CHAT: [[&str; 4]; 4] = [
    ["I tried baking a pie with {fs}. A disaster. A delicious disaster!", "If you ever need cheering up, come find me!", "I counted the flowers by the river. Lost count. Twice!", "Have you met everyone yet? Mossy tells the best stories."],
    ["The river was too loud last night. Rivers should know better.", "You again? ...Fine. It's not terrible to see you.", "Back in my day we caught bugs with our paws.", "That shop sells chairs. Chairs! As if anyone sits."],
    ["Do you think fish dream about us?", "A {f} fell on my head during my nap. A sign, maybe.", "The beach shells whisper if you hold them close.", "I'd like a house made entirely of pillows."],
    ["Fishing is basically arm day. Respect.", "Race you to the bridge! ...After my nap.", "Digging holes is great cardio. Don't tell Bramble.", "Shake a tree, catch the fruit. Reflexes!"],
];
pub const AGAIN: [&str; 4] = ["We already chatted today, but I don't mind more!", "Still here? Hmph.", "Oh... you're back. Like a tide.", "Round two? Let's go!"];
pub const RAIN: [&str; 4] = ["Rain means puddles! Puddles mean jumping!", "Rain. Wet fur. Wonderful. Truly.", "The rain is singing to the frogs. I'm a frog.", "Running in the rain builds character!"];
pub const ASK: [&str; 4] = ["Could you find me a {i}? I've been craving one!", "Bring me a {i}. If it's not too much trouble. Which it isn't.", "I keep dreaming of a {i}... could you find one?", "Challenge for you: get me a {i}! Go go go!"];
pub const THANKS: [&str; 4] = ["A {i}! You're the best! Here, take this!", "...A {i}. Hm. Thanks. Take this before I change my mind.", "Oh, a {i}... just like my dream. Thank you.", "A {i}! Record time! Here's your prize!"];
pub const HOMEL: [&str; 4] = ["Welcome to my place, {p}! Make yourself at home!", "You're in my house, {p}. Wipe your feet.", "Oh, a visitor... sit anywhere, {p}. Even the ceiling.", "Welcome to HQ, {p}! Mind the dumbbells!"];
