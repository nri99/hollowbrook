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
pub enum K {
    Bug,
    Fish,
    Fruit,
    Shell,
    Treasure,
    Fossil,
    Furn,
    Tool,
    Cloth,
    Sea,
    Material,
    Art,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    None,
    Sit,
    Bed,
    Lamp,
    Storage,
}

/// Item definition. `buy` = shop price, `sell` = resale value. Furniture: `fp` footprint (w,d), `rot` quarter-turns
/// allowed, `wall` = hangs on the back wall (otherwise floor + solid), `act` = interaction.
pub struct D {
    pub id: u16,
    pub name: &'static str,
    pub k: K,
    pub buy: u32,
    pub sell: u32,
    pub col: (f32, f32, f32),
    pub size: f32,
    pub hab: u8,
    pub m: (u32, u32),
    pub h: (u32, u32),
    pub fp: (i32, i32),
    pub rot: bool,
    pub wall: bool,
    pub act: Act,
}
const fn b(id: u16, name: &'static str, k: K, sell: u32, col: (f32, f32, f32)) -> D {
    D {
        id,
        name,
        k,
        buy: 0,
        sell,
        col,
        size: 0.4,
        hab: 0,
        m: (1, 12),
        h: (0, 24),
        fp: (1, 1),
        rot: false,
        wall: false,
        act: Act::None,
    }
}
#[allow(clippy::too_many_arguments)]
const fn cr(
    id: u16,
    name: &'static str,
    k: K,
    sell: u32,
    col: (f32, f32, f32),
    size: f32,
    hab: u8,
    m0: u32,
    m1: u32,
    h0: u32,
    h1: u32,
) -> D {
    D { size, hab, m: (m0, m1), h: (h0, h1), ..b(id, name, k, sell, col) }
}
const fn fu(
    id: u16,
    name: &'static str,
    buy: u32,
    col: (f32, f32, f32),
    fp: (i32, i32),
    rot: bool,
    wall: bool,
    act: Act,
) -> D {
    D { buy, fp, rot, wall, act, ..b(id, name, K::Furn, buy / 4, col) }
}
const fn cl(id: u16, name: &'static str, buy: u32, col: (f32, f32, f32)) -> D {
    D { buy, ..b(id, name, K::Cloth, buy / 4, col) }
}
const fn tl(id: u16, name: &'static str, buy: u32) -> D {
    D { buy, ..b(id, name, K::Tool, 0, (0.6, 0.45, 0.3)) }
}

pub static DEFS: [D; 137] = [
    cr(0, "Cabbage White", K::Bug, 90, (1., 1., 0.95), 0.2, 0, 3, 10, 6, 17),
    cr(1, "Honeybee", K::Bug, 160, (1., 0.85, 0.2), 0.2, 0, 3, 7, 8, 17),
    cr(2, "Ladybug", K::Bug, 120, (0.9, 0.15, 0.1), 0.2, 0, 3, 6, 8, 17),
    cr(3, "Dragonfly", K::Bug, 230, (0.3, 0.8, 0.9), 0.2, 0, 5, 10, 8, 19),
    cr(4, "Cicada", K::Bug, 300, (0.4, 0.6, 0.3), 0.2, 0, 7, 8, 8, 17),
    cr(5, "Firefly", K::Bug, 450, (0.8, 1., 0.3), 0.2, 0, 6, 7, 19, 4),
    cr(6, "Moth", K::Bug, 130, (0.85, 0.8, 0.65), 0.2, 0, 1, 12, 19, 4),
    cr(7, "Pill Bug", K::Bug, 60, (0.5, 0.5, 0.55), 0.2, 0, 1, 12, 6, 23),
    cr(10, "Minnow", K::Fish, 70, (0.6, 0.65, 0.7), 0.25, 1, 1, 12, 0, 24),
    cr(11, "Bluegill", K::Fish, 120, (0.35, 0.5, 0.75), 0.35, 1, 1, 12, 9, 16),
    cr(12, "Carp", K::Fish, 250, (0.65, 0.55, 0.3), 0.5, 1, 1, 12, 0, 24),
    cr(13, "Trout", K::Fish, 400, (0.75, 0.6, 0.6), 0.5, 1, 3, 11, 0, 24),
    cr(14, "Catfish", K::Fish, 800, (0.4, 0.4, 0.35), 0.7, 1, 5, 10, 16, 9),
    cr(15, "Eel", K::Fish, 1200, (0.3, 0.3, 0.25), 0.6, 1, 6, 9, 16, 9),
    cr(16, "Pike", K::Fish, 1800, (0.45, 0.55, 0.35), 0.75, 1, 9, 12, 0, 24),
    cr(17, "Koi", K::Fish, 3000, (1., 0.5, 0.3), 0.55, 1, 1, 12, 16, 9),
    cr(30, "Sardine", K::Fish, 100, (0.7, 0.75, 0.85), 0.25, 2, 1, 12, 0, 24),
    cr(31, "Sea Bass", K::Fish, 400, (0.5, 0.55, 0.6), 0.8, 2, 1, 12, 0, 24),
    cr(32, "Pufferfish", K::Fish, 900, (0.85, 0.8, 0.5), 0.45, 2, 6, 9, 0, 24),
    // X-04 critters (original, provisional): hab 3 = river mouth (river tiles near the sea), hab 4 = Gullrock only; Rain Snail needs rain
    cr(8, "Rain Snail", K::Bug, 250, (0.7, 0.55, 0.4), 0.2, 0, 3, 11, 0, 24),
    cr(9, "Dune Beetle", K::Bug, 600, (0.2, 0.55, 0.45), 0.2, 4, 1, 12, 8, 20),
    cr(18, "Ice Smelt", K::Fish, 500, (0.8, 0.88, 0.95), 0.3, 1, 12, 2, 0, 24),
    cr(19, "Brackish Goby", K::Fish, 650, (0.55, 0.5, 0.35), 0.35, 3, 1, 12, 0, 24),
    // X-04 furniture collections (original): Meadow set and Harbor set
    fu(90, "Meadow Chair", 900, (0.62, 0.8, 0.45), (1, 1), true, false, Act::Sit),
    fu(91, "Meadow Table", 1100, (0.85, 0.75, 0.45), (1, 1), false, false, Act::None),
    fu(92, "Meadow Lamp", 1000, (0.95, 0.85, 0.55), (1, 1), false, false, Act::Lamp),
    fu(93, "Harbor Bench", 1600, (0.3, 0.42, 0.6), (2, 1), true, false, Act::Sit),
    fu(94, "Harbor Crate", 1300, (0.6, 0.48, 0.32), (1, 1), true, false, Act::Storage),
    fu(95, "Harbor Lantern", 1200, (0.25, 0.3, 0.38), (1, 1), false, false, Act::Lamp),
    // X-04 paintings (original procedural art; sold at the weekly art stall, donatable to the art wing)
    b(100, "Morning Over Hollowbrook", K::Art, 600, (0.95, 0.8, 0.55)),
    b(101, "Gullrock at Low Tide", K::Art, 600, (0.4, 0.75, 0.85)),
    b(102, "The Cherry Lane", K::Art, 600, (0.95, 0.6, 0.7)),
    fu(110, "Meadow Bed", 2200, (0.75, 0.88, 0.6), (1, 2), true, false, Act::Bed),
    fu(111, "Meadow Planter", 700, (0.55, 0.78, 0.4), (1, 1), false, false, Act::None),
    fu(112, "Meadow Wall Art", 1000, (0.98, 0.75, 0.8), (1, 1), false, true, Act::None),
    fu(113, "Harbor Stool", 800, (0.35, 0.45, 0.6), (1, 1), false, false, Act::Sit),
    fu(114, "Harbor Buoy Light", 1400, (0.9, 0.35, 0.3), (1, 1), false, false, Act::Lamp),
    fu(115, "Harbor Bunk", 2400, (0.3, 0.38, 0.55), (1, 2), true, false, Act::Bed),
    // campground goods and favor rewards (original)
    fu(120, "Woven Wall Hanging", 1400, (0.75, 0.35, 0.3), (1, 1), false, true, Act::None),
    fu(121, "Loom Stool", 1000, (0.85, 0.7, 0.5), (1, 1), false, false, Act::Sit),
    fu(122, "Ship's Lantern", 1600, (0.85, 0.65, 0.25), (1, 1), false, false, Act::Lamp),
    fu(123, "Ship's Wheel", 2000, (0.55, 0.38, 0.22), (1, 1), false, true, Act::None),
    fu(124, "Herb Planter", 900, (0.4, 0.65, 0.35), (1, 1), false, false, Act::None),
    fu(125, "Garden Arch Lamp", 1800, (0.6, 0.8, 0.5), (1, 1), false, false, Act::Lamp),
    fu(126, "Tinker's Cabinet", 2200, (0.45, 0.4, 0.35), (1, 1), true, false, Act::Storage),
    fu(127, "Brass Clock", 2400, (0.85, 0.7, 0.3), (1, 1), false, false, Act::None),
    // initiative kiosk rewards (original)
    fu(130, "Town Pennant", 800, (0.3, 0.55, 0.45), (1, 1), false, true, Act::None),
    fu(131, "Lantern Post", 1200, (0.35, 0.35, 0.4), (1, 1), false, false, Act::Lamp),
    fu(132, "Reading Nook Sofa", 2000, (0.7, 0.45, 0.35), (2, 1), true, false, Act::Sit),
    fu(133, "Keepsake Chest", 1500, (0.6, 0.35, 0.25), (1, 1), true, false, Act::Storage),
    cl(134, "Initiative Tee", 500, (0.95, 0.75, 0.3)),
    cl(135, "Harbor Stripe Tee", 500, (0.3, 0.45, 0.75)),
    b(136, "Ferry Pass", K::Material, 0, (0.95, 0.85, 0.55)),
    b(137, "Sea Glass Pouch", K::Material, 0, (0.45, 0.85, 0.75)),
    // best-friend keepsake portraits, one per character (CHARS order)
    fu(140, "Juniper's Portrait", 2000, (0.93, 0.5, 0.2), (1, 1), false, true, Act::None),
    fu(141, "Bramble's Portrait", 2000, (0.55, 0.55, 0.58), (1, 1), false, true, Act::None),
    fu(142, "Mossy's Portrait", 2000, (0.45, 0.75, 0.35), (1, 1), false, true, Act::None),
    fu(143, "Otto's Portrait", 2000, (0.55, 0.38, 0.25), (1, 1), false, true, Act::None),
    fu(144, "Clover's Portrait", 2000, (0.3, 0.28, 0.3), (1, 1), false, true, Act::None),
    fu(145, "Basil's Portrait", 2000, (0.62, 0.47, 0.32), (1, 1), false, true, Act::None),
    // festival keepsakes (original)
    fu(146, "Blossom Lantern", 1500, (1., 0.7, 0.8), (1, 1), false, false, Act::Lamp),
    fu(147, "Paper Boat Model", 1500, (0.95, 0.95, 0.9), (1, 1), false, false, Act::None),
    fu(148, "Frost Lantern", 1500, (0.7, 0.85, 1.), (1, 1), false, false, Act::Lamp),
    fu(149, "Sunrise Banner", 1500, (1., 0.6, 0.3), (1, 1), false, true, Act::None),
    fu(151, "Velvet Armchair", 4000, (0.6, 0.15, 0.25), (1, 1), true, false, Act::Sit),
    fu(152, "Crystal Lamp", 3500, (0.85, 0.95, 1.), (1, 1), false, false, Act::Lamp),
    fu(153, "Grand Wardrobe", 5000, (0.35, 0.22, 0.15), (1, 1), true, false, Act::Storage),
    tl(154, "Watering Can", 500),
    fu(169, "Golden Home Plaque", 3000, (0.95, 0.8, 0.3), (1, 1), false, true, Act::None),
    // gardening (original): seeds from Mr. Tumble, flowers picked from blooms
    b(160, "Sunbell Seeds", K::Material, 30, (0.95, 0.85, 0.3)),
    b(161, "Moonpetal Seeds", K::Material, 30, (0.92, 0.92, 0.98)),
    b(162, "Emberbloom Seeds", K::Material, 30, (0.9, 0.3, 0.25)),
    b(163, "Skycup Seeds", K::Material, 30, (0.4, 0.6, 0.95)),
    b(164, "Sunbell", K::Material, 80, (0.98, 0.85, 0.25)),
    b(165, "Moonpetal", K::Material, 80, (0.96, 0.96, 1.)),
    b(166, "Emberbloom", K::Material, 80, (0.92, 0.28, 0.22)),
    b(167, "Skycup", K::Material, 80, (0.4, 0.6, 1.)),
    b(168, "Starfleck Bloom", K::Material, 600, (0.8, 0.55, 0.95)),
    b(150, "Lost Scarf", K::Material, 0, (0.85, 0.35, 0.45)),
    b(106, "Bramble's Garden", K::Art, 600, (0.55, 0.45, 0.6)),
    b(107, "Harbor Fog", K::Art, 600, (0.7, 0.75, 0.78)),
    b(103, "Lantern Night", K::Art, 600, (0.2, 0.22, 0.45)),
    b(104, "Pond in Summer", K::Art, 600, (0.4, 0.7, 0.4)),
    b(105, "First Snow", K::Art, 600, (0.85, 0.9, 0.98)),
    b(20, "Apple", K::Fruit, 100, (0.85, 0.15, 0.15)),
    b(21, "Pear", K::Fruit, 100, (0.8, 0.85, 0.25)),
    b(22, "Cherry", K::Fruit, 100, (0.7, 0.05, 0.2)),
    b(23, "Peach", K::Fruit, 100, (1., 0.65, 0.5)),
    b(40, "Scallop", K::Shell, 80, (0.98, 0.82, 0.72)),
    b(41, "Conch", K::Shell, 250, (0.98, 0.62, 0.55)),
    b(42, "Sand Dollar", K::Shell, 120, (0.92, 0.9, 0.78)),
    b(45, "Amber Stone", K::Treasure, 600, (0.95, 0.6, 0.1)),
    b(46, "Old Coin", K::Treasure, 1000, (0.85, 0.75, 0.3)),
    b(47, "Unknown Fossil", K::Fossil, 100, (0.78, 0.72, 0.6)),
    b(43, "Fern Fossil", K::Fossil, 700, (0.55, 0.6, 0.45)),
    b(44, "Shark Tooth", K::Fossil, 900, (0.95, 0.93, 0.85)),
    b(48, "Ammonite", K::Fossil, 1100, (0.8, 0.65, 0.45)),
    b(49, "Trilobite", K::Fossil, 1300, (0.5, 0.45, 0.4)),
    fu(50, "Chair", 600, (0.75, 0.5, 0.3), (1, 1), true, false, Act::Sit),
    fu(51, "Sofa", 1200, (0.35, 0.5, 0.78), (2, 1), true, false, Act::Sit),
    fu(52, "Table", 800, (0.66, 0.46, 0.3), (1, 1), false, false, Act::None),
    fu(53, "Bed", 1800, (0.55, 0.7, 0.9), (1, 2), true, false, Act::Bed),
    fu(54, "Lamp", 700, (0.95, 0.88, 0.65), (1, 1), false, false, Act::Lamp),
    fu(55, "Cabinet", 1500, (0.58, 0.38, 0.24), (1, 1), true, false, Act::Storage),
    fu(56, "Potted Plant", 500, (0.3, 0.62, 0.3), (1, 1), false, false, Act::None),
    fu(57, "Painting", 900, (0.4, 0.62, 0.88), (1, 1), false, true, Act::None),
    b(70, "Sea Star", K::Sea, 200, (0.95, 0.55, 0.35)),
    b(71, "Sea Urchin", K::Sea, 350, (0.35, 0.2, 0.45)),
    b(72, "Sea Slug", K::Sea, 500, (0.6, 0.85, 0.5)),
    b(73, "Octopus", K::Sea, 1200, (0.85, 0.35, 0.4)),
    b(82, "Driftwood", K::Material, 10, (0.75, 0.68, 0.55)),
    b(84, "Moon Shell", K::Shell, 60, (0.93, 0.93, 0.98)),
    b(85, "Sea Glass", K::Material, 0, (0.45, 0.85, 0.75)),
    b(74, "Glass Shrimp", K::Sea, 700, (0.8, 0.95, 0.95)),
    b(75, "Palm Crab", K::Sea, 450, (0.85, 0.3, 0.2)),
    fu(58, "Driftwood Stool", 1200, (0.72, 0.62, 0.47), (1, 1), false, false, Act::Sit),
    fu(59, "Glass Float", 1500, (0.4, 0.78, 0.82), (1, 1), false, false, Act::None),
    b(86, "Parcel", K::Material, 0, (0.75, 0.55, 0.35)),
    b(80, "Branch", K::Material, 20, (0.55, 0.4, 0.25)),
    b(81, "Stone", K::Material, 30, (0.6, 0.6, 0.62)),
    cl(67, "Straw Hat", 350, (0.9, 0.8, 0.45)),
    cl(68, "Beanie", 350, (0.3, 0.42, 0.8)),
    cl(69, "Flower Crown", 350, (0.95, 0.6, 0.72)),
    cl(63, "Sky Tee", 400, (0.45, 0.7, 0.95)),
    cl(64, "Sunflower Tee", 400, (0.98, 0.82, 0.3)),
    cl(65, "Forest Hoodie", 650, (0.25, 0.5, 0.32)),
    cl(66, "Berry Sweater", 650, (0.75, 0.25, 0.45)),
    fu(170, "Riverglass Plaque", 1800, (0.28, 0.68, 0.82), (1, 1), false, true, Act::None),
    fu(171, "Leafprint Plaque", 1800, (0.42, 0.72, 0.38), (1, 1), false, true, Act::None),
    tl(60, "Net", 0),
    tl(61, "Fishing Rod", 400),
    tl(62, "Shovel", 500),
    tl(172, "Stun Pistol", 1500),
    tl(173, "Tangle Launcher", 2500),
    tl(174, "Shock Staff", 1000),
    b(175, "Frost Crystal", K::Material, 350, (0.6, 0.85, 1.)),
    b(176, "Glowcap", K::Material, 300, (0.4, 1., 0.7)),
];
pub const FRUITS: [u16; 4] = [20, 21, 22, 23];
pub const FURN_IDS: [u16; 20] = [50, 51, 52, 53, 54, 55, 56, 57, 90, 91, 92, 93, 94, 95, 110, 111, 112, 113, 114, 115];
/// Furniture collections: (name, members). Completing one in your home shows a note.
pub const SETS: [(&str, [u16; 6]); 2] =
    [("Meadow collection", [90, 91, 92, 110, 111, 112]), ("Harbor collection", [93, 94, 95, 113, 114, 115])];
/// Base model drawn for a furniture id (collections reuse a model with their own colors and accents).
pub fn model(d: u16) -> u16 {
    match d {
        90 => 50,
        91 => 52,
        92 | 95 => 54,
        93 => 51,
        94 => 55,
        110 | 115 => 53,
        111 | 124 => 56,
        112 | 120 | 123 | 130 => 57,
        113 | 121 => 58,
        114 => 59,
        122 | 125 | 131 => 54,
        126 | 133 => 55,
        127 => 52,
        132 => 51,
        140..=145 | 149 | 169 | 170 | 171 => 57,
        146 | 148 | 152 => 54,
        147 => 52,
        151 => 50,
        153 => 55,
        _ => d,
    }
}
pub const ARTS: [u16; 8] = [100, 101, 102, 103, 104, 105, 106, 107];
/// Weekly art stall: open on Saturdays (when the fair isn't on) at STALL; price per painting (provisional).
pub const ART_PRICE: u32 = 2500;
pub fn weekday(day: i64) -> i64 {
    (day - 1).rem_euclid(7)
} // 0 = Monday (day = days from CE)
pub fn art_of_week(day: i64) -> u16 {
    ARTS[((day - 1).div_euclid(7) % ARTS.len() as i64) as usize]
}
/// Gallery slots on the museum's back wall (x positions), in ARTS order.
pub const GALLERY_X: [f32; 8] = [0.5, 1.35, 2.2, 3.05, 3.95, 4.8, 5.65, 6.5];
pub fn def(id: u16) -> &'static D {
    DEFS.iter().find(|d| d.id == id).unwrap_or(&DEFS[0])
}
pub fn is_def(id: u16) -> bool {
    DEFS.iter().any(|d| d.id == id)
}
fn inr(v: u32, (a, b): (u32, u32), incl: bool) -> bool {
    let hi = if incl { v <= b } else { v < b };
    if a <= b {
        v >= a && hi
    } else {
        v >= a || hi
    }
}
pub fn avail(d: &D, m: u32, h: u32) -> bool {
    inr(m, d.m, true) && inr(h, d.h, false)
}
/// Floor tiles covered by a furniture instance (footprint swaps axes on odd quarter turns).
pub fn foot(id: u16, x: i32, z: i32, rot: u8) -> Vec<(i32, i32)> {
    let (mut w, mut d) = def(id).fp;
    if rot % 2 == 1 {
        std::mem::swap(&mut w, &mut d)
    }
    (0..d).flat_map(|j| (0..w).map(move |i| (x + i, z + j))).collect()
}
pub fn rainy(d: i64) -> bool {
    (d as u64).wrapping_mul(2654435761) >> 8 & 7 == 0
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Wx {
    Clear,
    Rain,
    Snow,
}
/// Weather for a gameplay day: wet days (deterministic per day, provisional 1 in 8) fall as snow in Dec-Feb.
pub fn weather(day: i64, month: u32) -> Wx {
    if !rainy(day) {
        Wx::Clear
    } else if month == 12 || month <= 2 {
        Wx::Snow
    } else {
        Wx::Rain
    }
}
/// Provisional content choice for this milestone: shop open 08:00-22:00; stock refresh is separate (06:00).
pub fn shop_open(hr: u32) -> bool {
    (8..22).contains(&hr)
}
/// Town policies (original, provisional values). Effects are computed from this table on the fly and are
/// never written into saved schedules, so changing or reloading a policy cannot stack modifiers.
pub struct Policy {
    pub name: &'static str,
    pub fee: u32,
    pub shop: (u32, u32),
    pub wake: i32,
    pub sleep: i32,
    pub weeds: usize,
    pub exempt: &'static [usize],
    pub effect: &'static str,
}
pub const POLICIES: [Policy; 3] = [
    Policy { name: "Dawn Hours", fee: 12000, shop: (6, 22), wake: -2, sleep: 0, weeds: 2, exempt: &[1, 5], effect: "Store opens 6:00 (closing unchanged). Residents wake 2h earlier; bedtimes unchanged. Bramble and Basil keep their usual hours." },
    Policy { name: "Lantern Hours", fee: 12000, shop: (8, 24), wake: 0, sleep: 2, weeds: 2, exempt: &[3], effect: "Store closes 24:00 (opening unchanged). Residents stay up and home 2h later; waking unchanged. Otto keeps his usual hours." },
    Policy { name: "Tidy Commons", fee: 15000, shop: (8, 22), wake: 0, sleep: 0, weeds: 0, exempt: &[], effect: "No new weeds sprout each morning (normally 2 per day). Schedules unchanged." },
];
pub const ADMIN: &str = "Administrator Hazel Quill";

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BK {
    Shop,
    Home(usize),
    Res(usize),
    Civic,
    Museum,
}
pub struct Bld {
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub d: i32,
    pub kind: BK,
    pub name: &'static str,
    pub wall: (f32, f32, f32),
    pub roof: (f32, f32, f32),
}
/// Stable building ids = index. 1 = player 1's home (same id as earlier saves), 7 = player 2's home.
pub fn home_of(player: usize) -> usize {
    if player == 0 {
        1
    } else {
        7
    }
}
/// Mailbox tile beside each home's door (solid).
pub fn mailbox_tile(player: usize) -> (i32, i32) {
    let (x, z) = door_tile(home_of(player));
    (x + 1, z)
}
pub const BLDS: [Bld; 9] = [
    Bld {
        x: 4,
        z: 3,
        w: 5,
        d: 4,
        kind: BK::Shop,
        name: "General Store",
        wall: (0.96, 0.9, 0.78),
        roof: (0.25, 0.55, 0.7),
    },
    Bld {
        x: 12,
        z: 12,
        w: 3,
        d: 3,
        kind: BK::Home(0),
        name: "Player 1's Home",
        wall: (0.98, 0.95, 0.88),
        roof: (0.78, 0.32, 0.3),
    },
    Bld {
        x: 4,
        z: 12,
        w: 3,
        d: 3,
        kind: BK::Res(0),
        name: "Juniper's House",
        wall: (0.98, 0.88, 0.75),
        roof: (0.92, 0.52, 0.2),
    },
    Bld {
        x: 29,
        z: 4,
        w: 3,
        d: 3,
        kind: BK::Res(1),
        name: "Bramble's House",
        wall: (0.85, 0.85, 0.85),
        roof: (0.4, 0.4, 0.52),
    },
    Bld {
        x: 31,
        z: 13,
        w: 3,
        d: 3,
        kind: BK::Res(2),
        name: "Mossy's House",
        wall: (0.88, 0.95, 0.82),
        roof: (0.35, 0.65, 0.35),
    },
    Bld {
        x: 17,
        z: 4,
        w: 3,
        d: 3,
        kind: BK::Res(3),
        name: "Otto's House",
        wall: (0.85, 0.9, 0.98),
        roof: (0.3, 0.45, 0.82),
    },
    Bld {
        x: 10,
        z: 3,
        w: 5,
        d: 4,
        kind: BK::Civic,
        name: "Town Hall",
        wall: (0.92, 0.86, 0.74),
        roof: (0.52, 0.3, 0.25),
    },
    Bld {
        x: 16,
        z: 12,
        w: 3,
        d: 3,
        kind: BK::Home(1),
        name: "Player 2's Home",
        wall: (0.9, 0.96, 0.9),
        roof: (0.3, 0.6, 0.5),
    },
    Bld { x: 33, z: 5, w: 4, d: 3, kind: BK::Museum, name: "Museum", wall: (0.93, 0.92, 0.88), roof: (0.45, 0.5, 0.6) },
];
/// Outdoor tile directly in front of the door.
pub fn door_tile(b: usize) -> (i32, i32) {
    let b = &BLDS[b];
    (b.x + b.w / 2, b.z + b.d)
}
pub fn door_stand(b: usize) -> Vec2 {
    let (x, z) = door_tile(b);
    vec2(x as f32 + 0.5, z as f32 + 0.5)
}
pub fn door_face(b: usize) -> Vec2 {
    let (x, z) = door_tile(b);
    vec2(x as f32 + 0.5, z as f32)
}
pub fn room_dims(b: usize) -> (i32, i32) {
    if matches!(BLDS[b].kind, BK::Shop | BK::Museum | BK::Civic) {
        (7, 5)
    } else {
        (4, 4)
    }
}
/// Museum interior (7x5): curator desk, fish tank (left), bug case (right), four fossil pedestals.
/// Town Hall interior: civic desk tiles and potted plants (blocked).
pub const CIVIC_DESK: [(i32, i32); 3] = [(2, 1), (3, 1), (4, 1)];
pub const CIVIC_PLANTS: [(i32, i32); 2] = [(0, 0), (6, 0)];
pub const CIVIC_BENCH: [(i32, i32); 2] = [(5, 3), (6, 3)];
pub const MUSEUM_DESK: [(i32, i32); 3] = [(2, 1), (3, 1), (4, 1)];
pub const MUSEUM_TANK: [(i32, i32); 3] = [(0, 1), (0, 2), (0, 3)];
pub const MUSEUM_CASE: [(i32, i32); 3] = [(6, 1), (6, 2), (6, 3)];
pub const MUSEUM_PED: [((i32, i32), u16); 4] = [((1, 0), 48), ((5, 0), 49), ((1, 3), 43), ((5, 3), 44)];
pub const FOSSILS: [u16; 4] = [43, 44, 48, 49];

pub struct Res {
    pub name: &'static str,
    pub species: &'static str,
    pub fur: (f32, f32, f32),
    pub cloth: (f32, f32, f32),
    pub pers: usize,
    pub wake: u32,
    pub sleep: u32,
    pub bld: usize,
    pub home_h: [(u32, u32); 2],
}
/// Character pool. Houses are slots (HOUSES); `World.roster[slot]` says which character lives there.
pub const HOUSES: [usize; 4] = [2, 3, 4, 5];
pub const CHARS: [Res; 6] = [
    Res {
        name: "Juniper",
        species: "fox",
        fur: (0.93, 0.5, 0.2),
        cloth: (0.2, 0.65, 0.6),
        pers: 0,
        wake: 7,
        sleep: 22,
        bld: 2,
        home_h: [(12, 13), (20, 22)],
    },
    Res {
        name: "Bramble",
        species: "badger",
        fur: (0.55, 0.55, 0.58),
        cloth: (0.62, 0.2, 0.25),
        pers: 1,
        wake: 9,
        sleep: 23,
        bld: 3,
        home_h: [(13, 14), (21, 23)],
    },
    Res {
        name: "Mossy",
        species: "frog",
        fur: (0.45, 0.75, 0.35),
        cloth: (0.95, 0.8, 0.3),
        pers: 2,
        wake: 8,
        sleep: 21,
        bld: 4,
        home_h: [(11, 12), (19, 21)],
    },
    Res {
        name: "Otto",
        species: "otter",
        fur: (0.55, 0.38, 0.25),
        cloth: (0.25, 0.4, 0.8),
        pers: 3,
        wake: 6,
        sleep: 20,
        bld: 5,
        home_h: [(14, 15), (18, 20)],
    },
    Res {
        name: "Clover",
        species: "sheep",
        fur: (0.3, 0.28, 0.3),
        cloth: (0.95, 0.6, 0.7),
        pers: 0,
        wake: 7,
        sleep: 21,
        bld: 0,
        home_h: [(12, 13), (19, 21)],
    },
    Res {
        name: "Basil",
        species: "hedgehog",
        fur: (0.62, 0.47, 0.32),
        cloth: (0.35, 0.6, 0.35),
        pers: 1,
        wake: 9,
        sleep: 23,
        bld: 0,
        home_h: [(13, 14), (21, 23)],
    },
];
pub fn res_spot() -> Vec2 {
    vec2(1.5, 1.6)
}
/// Fixed furnishings of resident homes: (def, x, z, rot); z = -1 hangs on the back wall.
pub const TEMPL: [&[(u16, i32, i32, u8)]; 4] = [
    &[(53, 0, 0, 0), (52, 2, 1, 0), (50, 3, 1, 3), (56, 3, 0, 0), (57, 1, -1, 0)],
    &[(53, 3, 0, 0), (55, 0, 0, 0), (54, 1, 0, 0), (52, 0, 2, 0)],
    &[(53, 0, 1, 0), (56, 3, 0, 0), (56, 2, 0, 0), (54, 3, 2, 0), (57, 1, -1, 0)],
    &[(53, 3, 1, 0), (51, 0, 0, 0), (52, 0, 2, 0), (57, 2, -1, 0)],
];
pub const THEME: [((f32, f32, f32), (f32, f32, f32)); 4] = [
    ((0.85, 0.6, 0.4), (0.98, 0.86, 0.7)),
    ((0.5, 0.5, 0.56), (0.76, 0.79, 0.86)),
    ((0.55, 0.7, 0.45), (0.86, 0.95, 0.78)),
    ((0.45, 0.55, 0.76), (0.83, 0.9, 0.98)),
];
/// Shop interior (7x5): five daily-stock stands, two tool stands, counter, keeper behind it.
pub const DISPLAYS: [(i32, i32); 7] = [(0, 1), (0, 3), (6, 1), (6, 3), (1, 3), (5, 3), (5, 1)];
pub const TOOL_DISPLAY: [u16; 2] = [62, 61];
pub const COUNTER: [(i32, i32); 3] = [(2, 1), (3, 1), (4, 1)];
/// Clothing rack in the shop (always stocked; provisional).
pub const RACK: (i32, i32) = (1, 1);
pub const CLOTHES: [u16; 7] = [63, 64, 65, 66, 67, 68, 69];
pub fn is_hat(d: u16) -> bool {
    (67..=69).contains(&d)
}
/// Hairstyles (provisional): colour per index; odd indices add a bun.
pub const HAIRS: [(f32, f32, f32); 4] = [(0.36, 0.2, 0.12), (0.12, 0.1, 0.1), (0.85, 0.68, 0.35), (0.7, 0.3, 0.15)];
pub const HAIR_FEE: u32 = 150;
/// Customization palettes (provisional): furniture tints multiply the item colour; home wallpaper/flooring colours.
pub const TINTS: [(f32, f32, f32); 4] = [(1., 1., 1.), (1.2, 0.85, 0.65), (0.7, 0.9, 1.25), (0.55, 0.55, 0.6)];
pub const TINT_NAMES: [&str; 4] = ["original", "warm", "cool", "dark"];
pub const WALLS: [(f32, f32, f32); 4] = [(0.97, 0.93, 0.84), (0.82, 0.9, 0.97), (0.96, 0.84, 0.88), (0.86, 0.95, 0.83)];
pub const FLOORS: [(f32, f32, f32); 4] =
    [(0.78, 0.6, 0.42), (0.62, 0.64, 0.68), (0.86, 0.76, 0.56), (0.48, 0.36, 0.28)];
pub const TINT_FEE: u32 = 200;
pub const DECOR_FEE: u32 = 300;
/// Community projects (original design, initial balance): name and material needs (code, qty);
/// codes: 0 Branch, 1 Stone, 2 any fruit, 3 any shell.
pub const PROJECTS: [(&str, &[(u8, u16)]); 4] = [
    ("bench", &[(0, 5), (1, 2)]),
    ("lamp post", &[(0, 3), (1, 4), (3, 2)]),
    ("planter", &[(1, 4), (2, 3)]),
    ("bridge", &[(0, 12), (1, 8)]),
];
pub const MAT_NAMES: [&str; 4] = ["Branch", "Stone", "fruit", "shell"];
pub const PK_BRIDGE: u8 = 3;
/// Candidate sites per project kind (validated at proposal time; bridge = two river tiles from x).
pub const SITES: [&[(i32, i32)]; 4] = [
    &[(10, 8), (16, 16), (22, 8), (5, 18), (28, 18)],
    &[(8, 10), (19, 16), (27, 10), (14, 18), (10, 16)],
    &[(12, 11), (18, 10), (29, 8), (7, 16), (34, 18)],
    &[(25, 21)],
];
pub const BOARD: (i32, i32) = (14, 8);
/// Money-funded public projects (original, extensible table): name, coin cost, model id for draw_project.
pub const MPROJECTS: [(&str, u32, u8); 5] = [
    ("bench", 8000, 0),
    ("lamp post", 12000, 1),
    ("planter", 10000, 2),
    ("fountain", 40000, 4),
    ("flower arch", 25000, 5),
];
/// Player id of the town leader (initiates and places money-funded projects).
pub const LEADER: usize = 0;
pub const DOCK_APPROACH: (i32, i32) = (20, 24);
pub const DONATE_STEP: u32 = 1000;
/// Gullrock Islet (original getaway; initial balance): fare 200 coins one way, return free; daily activity alternates
/// between gathering 5 Driftwood and finding 3 Moon Shells; completing it once per player per day pays 3 Sea Glass;
/// the islet stall trades Sea Glass for exclusive furniture.
pub const IW: usize = 16;
pub const IH: usize = 14;
pub const FARE: u32 = 200;
pub const DOCK: (i32, i32) = (20, 25);
pub fn dock_stand() -> Vec2 {
    vec2(20.5, 24.5)
}
pub const ISLET_SPAWN: (f32, f32) = (7.5, 10.5);
pub const PELL_I: (f32, f32) = (6.5, 11.5);
pub const ISTALL: (i32, i32) = (10, 9);
/// Gullrock timed activities (original, provisional): name, objective, seconds, bronze/silver/gold team-score thresholds.
pub const ACTS: [(&str, &str, f32, [u32; 3]); 3] = [
    ("Crab Corral", "Walk into the crabs to nudge them inside the ring of stones.", 60., [2, 4, 6]),
    ("Beacon Dash", "Touch each glowing beacon before it fades (it moves every few seconds).", 45., [5, 9, 13]),
    ("Ring Swim", "Swim through the floating rings in numbered order.", 75., [2, 4, 6]),
];
/// Sea Glass paid to each participant per tier (bronze, silver, gold); once per activity per player per day.
pub const ACT_REWARD: [u16; 3] = [1, 2, 4];
pub const BEACON_LIFE: f32 = 4.5;
pub const PEN: (f32, f32, f32) = (3.5, 6.5, 1.1);
pub const ACT_BOARD: (i32, i32) = (9, 11);
pub fn rings() -> Vec<Vec2> {
    [0f32, -45., -90., -135., 180., 135.]
        .iter()
        .map(|a| {
            let r = a.to_radians();
            vec2(7.5 + 6.8 * 1.18 * r.cos(), 6.5 + 5.6 * 1.18 * r.sin())
        })
        .collect()
}
pub const ISTALL_ITEMS: [(u16, u16); 2] = [(58, 4), (59, 6)];
pub fn islet_task(day: i64) -> (u16, u16, &'static str) {
    if day % 2 == 0 {
        (82, 5, "Gather 5 Driftwood")
    } else {
        (84, 3, "Find 3 Moon Shells")
    }
}
pub fn islet() -> Vec<u8> {
    let mut m = vec![SEA; IW * IH];
    for z in 0..IH as i32 {
        for x in 0..IW as i32 {
            let (dx, dz) = ((x as f32 - 7.5) / 6.8, (z as f32 - 6.5) / 5.6);
            let d = dx * dx + dz * dz;
            m[z as usize * IW + x as usize] = if d < 0.45 {
                GRASS
            } else if d < 1.0 {
                SAND
            } else {
                SEA
            }
        }
    }
    for &(x, z) in &[(5, 5), (9, 4), (11, 7), (4, 8)] {
        m[z * IW + x] = TREE
    }
    for &(x, z) in &[(8, 7), (12, 5)] {
        m[z * IW + x] = ROCK
    }
    for z in 12..14 {
        m[z * IW + 7] = BRIDGE
    }
    m[10 * IW + 3] = RIVER;
    m[10 * IW + 4] = RIVER; // tide pool
    m
}
pub const KEEPER: [&str; 2] = [
    "Welcome to the General Store! New furniture arrives every morning at 6.",
    "Sell your finds at the counter. We're open from 8:00 until 22:00.",
];

fn put(m: &mut [u8], x: i32, z: i32, c: u8) {
    if x >= 0 && z >= 0 && (x as usize) < W && (z as usize) < H {
        m[z as usize * W + x as usize] = c
    }
}
fn path(m: &mut [u8], x: i32, z: i32) {
    let i = z as usize * W + x as usize;
    m[i] = match m[i] {
        RIVER => BRIDGE,
        BLD | BRIDGE => m[i],
        _ => PATH,
    }
}
/// The authored demonstration town.
/// Harvest Fair (provisional date window and rules): 13-16 November; stall by the plaza; one prize per player per year.
pub fn fair_active(month: u32, day: u32) -> bool {
    month == 11 && (13..=16).contains(&day)
}
pub const STALL: (i32, i32) = (11, 10);
pub fn town() -> Vec<u8> {
    let mut m = vec![GRASS; W * H];
    for z in 0..H as i32 {
        for x in 0..W as i32 {
            put(
                &mut m,
                x,
                z,
                if z >= 26 {
                    SEA
                } else if z >= 24 {
                    SAND
                } else if x == 0 || z == 0 || x == W as i32 - 1 {
                    PINE
                } else {
                    GRASS
                },
            )
        }
    }
    for z in 1..26 {
        let cx = if z <= 12 { 24 } else { 25 };
        put(&mut m, cx, z, RIVER);
        put(&mut m, cx + 1, z, RIVER);
        if z == 13 {
            put(&mut m, 24, z, RIVER)
        }
    }
    for b in &BLDS {
        for z in b.z..b.z + b.d {
            for x in b.x..b.x + b.w {
                put(&mut m, x, z, BLD)
            }
        }
    }
    for x in 1..W as i32 - 1 {
        path(&mut m, x, 9);
        path(&mut m, x, 17)
    }
    for z in 9..=17 {
        path(&mut m, 9, z);
        path(&mut m, 20, z)
    }
    for i in 0..BLDS.len() {
        let (dx, dz) = door_tile(i);
        if dz < 9 {
            for z in dz..9 {
                path(&mut m, dx, z)
            }
        } else {
            for z in dz..=17 {
                path(&mut m, dx, z)
            }
        }
    }
    for z in 17..24 {
        path(&mut m, 13, z)
    }
    let feat: [(u8, &[(i32, i32)]); 4] = [
        (
            TREE,
            &[
                (2, 4),
                (2, 7),
                (3, 11),
                (8, 11),
                (15, 2),
                (21, 2),
                (22, 6),
                (16, 11),
                (21, 13),
                (2, 20),
                (8, 20),
                (17, 21),
                (22, 20),
                (28, 11),
                (35, 10),
                (36, 3),
                (33, 2),
                (29, 20),
                (36, 20),
                (10, 13),
            ],
        ),
        (PINE, &[(27, 6), (37, 13), (11, 21), (23, 15), (3, 22), (38, 7)]),
        (ROCK, &[(15, 6), (27, 2), (35, 15), (6, 21), (19, 14), (37, 22)]),
        (
            FLOWER,
            &[
                (3, 6),
                (7, 2),
                (14, 10),
                (15, 10),
                (16, 8),
                (11, 15),
                (15, 14),
                (19, 11),
                (3, 14),
                (7, 14),
                (28, 15),
                (34, 11),
                (36, 6),
                (30, 22),
                (27, 21),
                (10, 22),
                (19, 22),
                (16, 14),
            ],
        ),
    ];
    for (c, list) in feat {
        for &(x, z) in list {
            if m[z as usize * W + x as usize] == GRASS {
                put(&mut m, x, z, c)
            }
        }
    }
    // openings in the northern pines: the path to North Meadow and the river flowing in from its lake
    for x in [17, 18] {
        put(&mut m, x, 0, PATH);
    }
    for x in [24, 25] {
        put(&mut m, x, 0, RIVER);
    }
    m
}

// ---- dialogue (original, authored; two+ distinct personality styles) ----
pub const GREET: [[&str; 4]; 4] = [
    [
        "Morning, {p}! The air smells like {fs} today!",
        "Hi hi, {p}! Isn't this afternoon just perfect?",
        "Evening, {p}! The sky's going all peachy.",
        "Oh! {p}! You're up late too?",
    ],
    [
        "Hmph. Morning, {p}. Too bright out.",
        "Afternoon. Mind my flowers, {p}.",
        "Evening, {p}. Finally, some quiet.",
        "It's the middle of the night, {p}. Go home.",
    ],
    [
        "Mmm... morning, {p}. I dreamed I was a cloud.",
        "Oh, hello {p}... the pond is humming today.",
        "Evening, {p}. The stars are warming up.",
        "{p}... the night smells like moss.",
    ],
    [
        "Morning, {p}! Already swam two laps!",
        "Hey {p}! Perfect weather for a sprint!",
        "Evening, {p}! Cooldown stretches, join me!",
        "Late-night training, {p}? Respect!",
    ],
];
pub const CHAT: [[&str; 4]; 4] = [
    [
        "I tried baking a pie with {fs}. A disaster. A delicious disaster!",
        "If you ever need cheering up, come find me!",
        "I counted the flowers by the river. Lost count. Twice!",
        "Have you met everyone yet? Mossy tells the best stories.",
    ],
    [
        "The river was too loud last night. Rivers should know better.",
        "You again? ...Fine. It's not terrible to see you.",
        "Back in my day we caught bugs with our paws.",
        "That shop sells chairs. Chairs! As if anyone sits.",
    ],
    [
        "Do you think fish dream about us?",
        "A {f} fell on my head during my nap. A sign, maybe.",
        "The beach shells whisper if you hold them close.",
        "I'd like a house made entirely of pillows.",
    ],
    [
        "Fishing is basically arm day. Respect.",
        "Race you to the bridge! ...After my nap.",
        "Digging holes is great cardio. Don't tell Bramble.",
        "Shake a tree, catch the fruit. Reflexes!",
    ],
];
/// Gift preference per resident (provisional): liked kinds give more friendship.
pub const LIKES: [K; 4] = [K::Fruit, K::Fossil, K::Fish, K::Bug];
/// Birthdays per character (CHARS index), initial content choice: (month, day).
pub const BDAYS: [(u32, u32); 6] = [(4, 12), (9, 3), (6, 21), (11, 16), (3, 2), (12, 8)];
/// Short lines residents say to each other (speech bubbles), per personality.
pub const CHATTER: [[&str; 3]; 4] = [
    ["Isn't today lovely?", "I love your scarf!", "Let's picnic soon!"],
    ["Hmph. Nice weather. I suppose.", "Don't step on my flowers.", "...Fine. Hello."],
    ["The clouds look like fish today.", "Do pebbles dream?", "Shh... listen to the wind."],
    ["Race you to the bridge!", "Stretch break!", "I did fifty laps today!"],
];
pub const AGAIN: [&str; 4] = [
    "We already chatted today, but I don't mind more!",
    "Still here? Hmph.",
    "Oh... you're back. Like a tide.",
    "Round two? Let's go!",
];
pub const SNOW: [&str; 4] = [
    "Snow! Let's build a snow-fox!",
    "Snow. Cold paws. Marvelous.",
    "Every snowflake is a tiny cloud that gave up.",
    "Snow sprints are the best sprints!",
];
pub const RAIN: [&str; 4] = [
    "Rain means puddles! Puddles mean jumping!",
    "Rain. Wet fur. Wonderful. Truly.",
    "The rain is singing to the frogs. I'm a frog.",
    "Running in the rain builds character!",
];
pub const ASK: [&str; 4] = [
    "Could you find me a {i}? I've been craving one!",
    "Bring me a {i}. If it's not too much trouble. Which it isn't.",
    "I keep dreaming of a {i}... could you find one?",
    "Challenge for you: get me a {i}! Go go go!",
];
pub const THANKS: [&str; 4] = [
    "A {i}! You're the best! Here, take this!",
    "...A {i}. Hm. Thanks. Take this before I change my mind.",
    "Oh, a {i}... just like my dream. Thank you.",
    "A {i}! Record time! Here's your prize!",
];
pub const VISIT: [&str; 4] = [
    "Hi {p}! I was passing by and thought I'd say hello. Cute place!",
    "Hmph. I was in the area. Your house is... acceptable.",
    "Your home smells like {fs} and good dreams, {p}.",
    "Pit stop! Nice digs, {p}. Very aerodynamic.",
];
pub const HOMEL: [&str; 4] = [
    "Welcome to my place, {p}! Make yourself at home!",
    "You're in my house, {p}. Wipe your feet.",
    "Oh, a visitor... sit anywhere, {p}. Even the ceiling.",
    "Welcome to HQ, {p}! Mind the dumbbells!",
];

// ---- Campground (original) ----
pub const CAMP_TENT: [(i32, i32); 4] = [(32, 19), (33, 19), (32, 20), (33, 20)];
pub const CAMP_FIRE: (i32, i32) = (31, 21);
pub const CAMP_CRATE: (i32, i32) = (34, 20);
pub const CAMP_SIGN: (i32, i32) = (31, 19);
pub const CAMP_SPOT: (f32, f32) = (32.5, 21.55);
/// Visitors are at the campfire from 8:00 until 22:00 (provisional).
pub const CAMP_HOURS: (u32, u32) = (8, 22);
pub fn camp_block(t: (i32, i32)) -> bool {
    CAMP_TENT.contains(&t) || [CAMP_FIRE, CAMP_CRATE, CAMP_SIGN].contains(&t)
}
/// A travelling visitor: favor = (item wanted, quantity; 0 = the town's native fruit).
pub struct Visitor {
    pub name: &'static str,
    pub species: &'static str,
    pub fur: (f32, f32, f32),
    pub cloth: (f32, f32, f32),
    pub tent: (f32, f32, f32),
    pub lines: [&'static str; 3],
    pub goods: [u16; 3],
    pub favor: (u16, u16),
    pub ask: &'static str,
    pub reward: u16,
}
pub const VISITORS: [Visitor; 4] = [
    Visitor {
        name: "Tamsin",
        species: "hare",
        fur: (0.82, 0.72, 0.6),
        cloth: (0.75, 0.35, 0.3),
        tent: (0.85, 0.45, 0.35),
        lines: [
            "I weave as I wander. Every town gets a thread in my blanket.",
            "Hollowbrook's thread will be soft green, I think. Like your meadow.",
            "I only stay one night. The road is my loom!",
        ],
        goods: [120, 66, 69],
        favor: (80, 2),
        ask: "Could you find me 2 Branches? My loom frame cracked.",
        reward: 121,
    },
    Visitor {
        name: "Barnaby",
        species: "tortoise",
        fur: (0.5, 0.6, 0.4),
        cloth: (0.25, 0.32, 0.5),
        tent: (0.3, 0.4, 0.6),
        lines: [
            "Forty years at sea, and now I sail by tent. Slower, but drier.",
            "Gullrock? I mapped that rock when it was half the size, I swear it.",
            "A lantern and a good story. That's all a camp needs.",
        ],
        goods: [122, 59, 58],
        favor: (30, 1),
        ask: "I'd love one Sardine for supper. My old hands can't cast anymore.",
        reward: 123,
    },
    Visitor {
        name: "Marigold",
        species: "goat",
        fur: (0.92, 0.9, 0.85),
        cloth: (0.55, 0.7, 0.35),
        tent: (0.6, 0.75, 0.4),
        lines: [
            "Smell that? Wild thyme. I could follow it across three valleys.",
            "I trade in seeds and sun hats. Mostly sun hats, honestly.",
            "Every garden is a little conversation with the weather.",
        ],
        goods: [124, 67, 56],
        favor: (0, 3),
        ask: "Three of your town's fruit, please! I'm brewing a jam for the road.",
        reward: 125,
    },
    Visitor {
        name: "Cobb",
        species: "mole",
        fur: (0.35, 0.3, 0.32),
        cloth: (0.7, 0.55, 0.3),
        tent: (0.55, 0.5, 0.35),
        lines: [
            "Tick, tick, tick. That's the sound of a happy clock. Or a nervous one.",
            "I fix things. Clocks, cabinets, the occasional bad mood.",
            "Underground it's always the same hour. Up here, every hour is new!",
        ],
        goods: [126, 54, 55],
        favor: (81, 2),
        ask: "Bring me 2 Stones? I need weights for a pendulum.",
        reward: 127,
    },
];
pub fn visitor_of(day: i64) -> usize {
    day.rem_euclid(4) as usize
}

// ---- Initiatives (original): coupon kiosk inside the Town Hall ----
pub const KIOSK: (i32, i32) = (0, 2);
/// Kiosk rewards: (item, coupon price). 136 Ferry Pass = one free trip; 137 = 3 Sea Glass.
pub const KIOSK_ITEMS: [(u16, u32); 8] =
    [(130, 6), (131, 10), (132, 16), (133, 12), (134, 5), (135, 5), (136, 4), (137, 8)];
pub const WEEKLY_COUPONS: u32 = 12;

// ---- Release / help text ----
pub const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"), " (build ", env!("HB_GIT"), ")");
pub const HELP_KEYS: [&str; 10] = [
    "Move: WASD or arrows   Run: hold Shift",
    "Interact / confirm / next: E or Enter   Cancel / close: Esc",
    "Use tool / dive: Space   Cycle tools: Tab",
    "Pockets: I   Journal (help, settings, tasks): J",
    "Gift during a chat: G",
    "Furniture: F move, G pick up, R rotate",
    "Player 2 join / leave: F2",
    "Menus: W/S or Up/Down to choose, Enter to confirm",
    "Journal pages: Tab next page",
    "Quit: close the window (saves first)",
];
pub const HELP_PAD: [&str; 10] = [
    "Move: left stick or d-pad   Run: hold LT or RT",
    "Interact / confirm / next: A   Cancel / close: B",
    "Use tool / dive: X   Cycle tools: RB",
    "Pockets: Y   Journal: Menu (Start)",
    "Gift during a chat: LB",
    "Furniture: View move, LB pick up, RB rotate",
    "Menu or A on a new pad: join as P2",
    "Hold View + press Menu: switch pad P1 <-> P2",
    "Journal pages: RB next page",
    "Xbox names; PlayStation: Cross = A, Circle = B",
];
pub const GUIDE: [&str; 15] = [
    "Home: expand at the General Store counter. Each expansion adds an interest-free balance to pay off whenever you like; pay it off before the next one.",
    "Projects: residents propose material projects at the notice board; the town leader (Player 1) starts coin-funded projects at Hazel's desk. Fully funded = built next morning.",
    "Policies: Player 1 can enact one policy at Hazel's desk (fee shown first): earlier store hours, later hours, or fewer weeds.",
    "Gullrock: sail from the dock (200 coins, or a Ferry Pass). Captain Pell has a daily task; the board runs timed activities for Sea Glass.",
    "Campground (east meadow): a different traveller visits each day, 8:00-22:00. Buy their goods (one of each per day) and do their favor once for a keepsake.",
    "Initiatives: 3 personal tasks a day plus a weekly town task. Claim coupons at the kiosk in the Town Hall and trade them for rewards.",
    "Home Society: every Monday a letter scores your home (furniture, variety, collections, lamps, wall items, decor, size) with tips; reaching a new rating pays a reward. Hazel can repaint your house outside.",
    "Gardening: buy seeds and a watering can from Mr. Tumble, plant on grass, water daily (rain counts). Blooms can be picked; different colours side by side may cross. He also upgrades tools.",
    "Museum: donate fish, bugs, fossils and paintings. The art stall opens on Saturdays (not during festivals).",
    "Trail Seek: a friend may invite you to find three residents behind trees or buildings. They walk to hiding spots first. Talk to each within four real-clock minutes for a personal reward. Indoors, menus, and reloads do not pause the timer; timeout costs nothing.",
    "Resident invitations: nearby friends may ask to chat. Talk to accept or decline. Visit the named home during the agreed hour today for friendship and a flower. Missed visits cause no penalty; a gentle follow-up arrives next day. Check the notice board for your appointment.",
    "Monthly surveys (Mara at plaza): fish on 2nd Wednesday, bugs on 4th Sunday. Register/enter 09-17; prizes 17-22. Measured catches score by relative size and rarity. Improved entries are released. Old unsized stacks stay usable but cannot enter.",
    "Calendar: First Light Jan 1-2, Blossom Picnic Apr 10-12, Starlit Regatta Jul 24-26, Harvest Fair Nov 13-16, Frost Lantern Festival Dec 20-24. Visit the plaza host.",
    "Days change at 6:00 real time. The game saves every 15 seconds, at each new day, and when you close it.",
    "Two players: F2 or a second controller adds Player 2 with their own home, pockets, coins and tasks."];

// ---- Resident depth (original writing) ----
pub const TIERS: [&str; 5] = ["Acquaintance", "Friend", "Good friend", "Close friend", "Best friend"];
pub fn tier(f: i32) -> usize {
    match f {
        i32::MIN..=4 => 0,
        5..=14 => 1,
        15..=29 => 2,
        30..=49 => 3,
        _ => 4,
    }
}
pub const PERS_NAME: [&str; 4] = ["cheerful", "grumpy", "dreamy", "sporty"];
pub const KEEPSAKES: [u16; 6] = [140, 141, 142, 143, 144, 145];
/// One line per personality per friendship tier, said on the first chat of the day.
pub const TIER_LINES: [[&str; 5]; 4] = [
    [
        "Oh, you're new-ish! I'm still learning your favorite color.",
        "I saved you a wave this morning, {p}!",
        "You're on my list of people who make the day sparkle.",
        "If I bake anything, the first slice is yours. That's a rule now.",
        "Best friends forever, {p}. I wrote it in my diary in glitter.",
    ],
    [
        "Hmph. Do I know you?",
        "Oh. It's you. ...That's fine.",
        "I suppose you're one of the tolerable ones, {p}.",
        "Don't tell anyone, but I look forward to you stopping by.",
        "You're my best friend, {p}. If you repeat that, I'll deny it.",
    ],
    [
        "Have we met in a dream? You feel familiar.",
        "Hello, {p}. The wind said you'd come.",
        "I named a cloud after you. It's the fluffy one.",
        "When I count stars, I count you too, {p}.",
        "You're my favorite person in every version of the world, {p}.",
    ],
    [
        "New face! Let's see your stretch form.",
        "Hey {p}! You're getting quicker, I can tell!",
        "Training buddy! We should race to the beach sometime.",
        "You're my go-to spotter, {p}. Total trust.",
        "Best friends, {p}! Like a perfect relay handoff!",
    ],
];
/// Seasonal lines per personality (spring, summer, autumn, winter).
pub const SEASON_LINES: [[&str; 4]; 4] = [
    [
        "Spring! Everything is waking up and saying hello!",
        "Summer means picnics by the river every single day!",
        "The leaves are turning! It's like the trees are getting dressed up.",
        "Winter is cozy season. Blankets! Cocoa! More blankets!",
    ],
    [
        "Spring. Pollen. My nose is filing a complaint.",
        "Summer. Too hot. I'm staying in the shade until autumn.",
        "Autumn is acceptable. Crisp. Quiet. Correct.",
        "Winter keeps the noisy bugs away. I approve.",
    ],
    [
        "In spring the ground hums. Can you hear it?",
        "Summer nights are when the fireflies tell stories.",
        "Autumn leaves are letters the trees write to the ground.",
        "Snow makes everything whisper. I like whispering.",
    ],
    [
        "Spring training season! New goals!",
        "Summer swim season! The river is my gym!",
        "Autumn runs are the best runs. Crunchy leaves!",
        "Winter? Indoor push-ups. Lots of them.",
    ],
];
/// Gossip about another resident ({o}) per personality.
pub const GOSSIP: [[&str; 2]; 4] = [
    [
        "{o} lent me a book yesterday! I read the first page four times.",
        "Have you visited {o} lately? They'd love the company!",
    ],
    [
        "{o} walked past my garden twice today. Suspicious.",
        "{o} said hello to me. I said hello back. Don't make it weird.",
    ],
    ["{o} has a very calm shadow. I noticed.", "I think {o} would make a good lighthouse."],
    ["{o} almost beat me in a race. Almost!", "I'm teaching {o} to stretch properly. Slow progress."],
];
/// Hobby and personal lines per character (CHARS order), so residents sharing a personality still differ.
pub const HOBBY: [&str; 6] = ["baking", "gardening", "stargazing", "swimming", "knitting", "clock repair"];
pub const CHAR_LINES: [[&str; 3]; 6] = [
    [
        "I'm perfecting a {f} tart. Attempt number nine!",
        "Flour on my nose again? It's my signature look.",
        "Baking is just science you can eat.",
    ],
    [
        "My tulips have names. I won't tell you them.",
        "Weeds are just plants with bad manners.",
        "Gardening teaches patience. Mostly to the gardener.",
    ],
    [
        "I found a new star last night. It was a firefly.",
        "The moon looks lonely on Tuesdays.",
        "If you look up long enough, you start floating. Inside.",
    ],
    ["I timed my swim to the bridge. New record!", "The river current is my rival.", "Hydration is a lifestyle!"],
    [
        "I'm knitting a scarf long enough for the whole town.",
        "Knit one, purl two, chat three!",
        "Soft yarn makes soft days.",
    ],
    [
        "Every clock in my house says a different time. On purpose!",
        "A gear went missing. I suspect a squirrel.",
        "Fixing things is my favorite way to listen.",
    ],
];

// ---- Calendar celebrations (original). The Harvest Fair (Nov 13-16) is separate. ----
/// Festival task: what the host asks for once per player per year.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FTask {
    Fruit(u16),
    Fish(u16),
    Branch(u16),
    TalkAll,
}
pub struct Fest {
    pub name: &'static str,
    pub m: u32,
    pub d0: u32,
    pub d1: u32,
    pub task: FTask,
    pub ask: &'static str,
    pub reward: u16,
    pub about: &'static str,
    pub chat: &'static str,
}
pub const FESTS: [Fest; 4] = [
    Fest {
        name: "Blossom Picnic",
        m: 4,
        d0: 10,
        d1: 12,
        task: FTask::Fruit(2),
        ask: "Bring 2 of the town's fruit for the shared picnic basket.",
        reward: 146,
        about: "Spring's first picnic under the blossoms. Everyone brings something to share.",
        chat: "The Blossom Picnic is on! Bring some fruit to the plaza!",
    },
    Fest {
        name: "Starlit Regatta",
        m: 7,
        d0: 24,
        d1: 26,
        task: FTask::Fish(1),
        ask: "Catch any fish so we can wish it luck before the paper boats sail.",
        reward: 147,
        about: "Paper boats with tiny lanterns drift down the river after dark.",
        chat: "Have you seen the paper boats on the river? The Regatta is so pretty at night!",
    },
    Fest {
        name: "Frost Lantern Festival",
        m: 12,
        d0: 20,
        d1: 24,
        task: FTask::Branch(2),
        ask: "Bring 2 Branches to frame the frost lanterns.",
        reward: 148,
        about: "Blue lanterns light the long winter nights around the plaza tree.",
        chat: "The frost lanterns are up! The plaza is glowing!",
    },
    Fest {
        name: "First Light",
        m: 1,
        d0: 1,
        d1: 2,
        task: FTask::TalkAll,
        ask: "Wish all four residents a happy new year today.",
        reward: 149,
        about: "The town greets the first sunrise of the year with banners and good wishes.",
        chat: "Happy First Light! I hope this year is your best one yet!",
    },
];
pub fn fest_of(m: u32, d: u32) -> Option<usize> {
    FESTS.iter().position(|f| f.m == m && (f.d0..=f.d1).contains(&d))
}

// ---- Daily surprises (original): one shared surprise appears in town each day ----
pub const SURPRISES: [&str; 3] = ["message bottle", "lost scarf", "drifting gift"];
pub const BOTTLE_NOTES: [&str; 6] = [
    "\"To whoever finds this: the river is wider than it looks, and so are you. - a traveller\"",
    "\"Day 40 at sea. Still no land. Still plenty of snacks. Morale: excellent.\"",
    "\"If you shake a tree and nothing falls, try again in three days. Trees need naps too.\"",
    "\"I buried a wish under the old oak. You can have it. I have more.\"",
    "\"Note to self: the fish bite best when you stop staring at the float.\"",
    "\"Hello from the other side of the sea! It's mostly the same over here, but saltier.\"",
];
pub const BOTTLE_COINS: u32 = 250;
pub const RETURN_COINS: u32 = 400;

// ---- Evolving General Store (original, provisional thresholds) ----
/// Cumulative town spending at the store that unlocks renovation tiers 1 and 2.
pub const SHOP_TIERS: [u32; 2] = [8000, 25000];
/// Tier 1: a weekly special (one per week for the whole town). Tier 2: one more daily stock slot.
pub const SPECIALS: [u16; 3] = [151, 152, 153];
pub const SPECIAL_DISPLAY: (i32, i32) = (6, 2);
pub const EXTRA_DISPLAY: (i32, i32) = (0, 2);

// ---- Gardening (original, provisional) ----
pub const SEEDS: [u16; 4] = [160, 161, 162, 163];
pub const SEED_PRICE: u32 = 120;
/// Flower item for plant kind 0-3 (colours) and 4 (rare cross-bred Starfleck).
pub const FLOWERS: [u16; 5] = [164, 165, 166, 167, 168];
pub const CAN: u16 = 154;
/// Chance per adjacent pair of different-coloured blooms per day to cross into a Starfleck (provisional).
pub const CROSS_CHANCE: u32 = 4; // 1 in 4
                                 // ---- Tool upgrades (original, provisional): level 1 and 2 costs (coins, stones, sea glass) ----
pub const UPGRADE: [(u32, u16, u16); 2] = [(1500, 3, 0), (4000, 5, 3)];
pub const UPGRADABLE: [u16; 4] = [60, 61, 62, 154];
pub const LV_NAME: [&str; 3] = ["", "Sturdy ", "Masterwork "];

// ---- Home evaluation and exterior paint (original, provisional) ----
pub const HOME_TIERS: [(u32, &str, u32); 5] =
    [(0, "Bare", 0), (100, "Cozy", 500), (250, "Charming", 1000), (450, "Splendid", 2000), (700, "Masterpiece", 3000)];
pub const PAINTS: [(&str, (f32, f32, f32), (f32, f32, f32)); 6] = [
    ("Cream & Brick", (0.95, 0.9, 0.78), (0.72, 0.3, 0.25)),
    ("Sky & Slate", (0.78, 0.88, 0.97), (0.35, 0.4, 0.5)),
    ("Mint & Pine", (0.82, 0.95, 0.85), (0.2, 0.45, 0.3)),
    ("Rose & Plum", (0.97, 0.84, 0.88), (0.5, 0.25, 0.4)),
    ("Sand & Terracotta", (0.93, 0.85, 0.68), (0.78, 0.45, 0.28)),
    ("Snow & Charcoal", (0.96, 0.96, 0.96), (0.25, 0.25, 0.28)),
];
pub const PAINT_FEE: u32 = 800;

/// Named item ids (generated from `DEFS`; ids are stored in saves, so never renumber). Use these instead of bare numbers.
#[allow(dead_code)]
pub mod id {
    pub const CABBAGE_WHITE: u16 = 0; // Cabbage White
    pub const HONEYBEE: u16 = 1; // Honeybee
    pub const LADYBUG: u16 = 2; // Ladybug
    pub const DRAGONFLY: u16 = 3; // Dragonfly
    pub const CICADA: u16 = 4; // Cicada
    pub const FIREFLY: u16 = 5; // Firefly
    pub const MOTH: u16 = 6; // Moth
    pub const PILL_BUG: u16 = 7; // Pill Bug
    pub const MINNOW: u16 = 10; // Minnow
    pub const BLUEGILL: u16 = 11; // Bluegill
    pub const CARP: u16 = 12; // Carp
    pub const TROUT: u16 = 13; // Trout
    pub const CATFISH: u16 = 14; // Catfish
    pub const EEL: u16 = 15; // Eel
    pub const PIKE: u16 = 16; // Pike
    pub const KOI: u16 = 17; // Koi
    pub const SARDINE: u16 = 30; // Sardine
    pub const SEA_BASS: u16 = 31; // Sea Bass
    pub const PUFFERFISH: u16 = 32; // Pufferfish
    pub const RAIN_SNAIL: u16 = 8; // Rain Snail
    pub const DUNE_BEETLE: u16 = 9; // Dune Beetle
    pub const ICE_SMELT: u16 = 18; // Ice Smelt
    pub const BRACKISH_GOBY: u16 = 19; // Brackish Goby
    pub const MEADOW_CHAIR: u16 = 90; // Meadow Chair
    pub const MEADOW_TABLE: u16 = 91; // Meadow Table
    pub const MEADOW_LAMP: u16 = 92; // Meadow Lamp
    pub const HARBOR_BENCH: u16 = 93; // Harbor Bench
    pub const HARBOR_CRATE: u16 = 94; // Harbor Crate
    pub const HARBOR_LANTERN: u16 = 95; // Harbor Lantern
    pub const MORNING_OVER_HOLLOWBROOK: u16 = 100; // Morning Over Hollowbrook
    pub const GULLROCK_AT_LOW_TIDE: u16 = 101; // Gullrock at Low Tide
    pub const THE_CHERRY_LANE: u16 = 102; // The Cherry Lane
    pub const MEADOW_BED: u16 = 110; // Meadow Bed
    pub const MEADOW_PLANTER: u16 = 111; // Meadow Planter
    pub const MEADOW_WALL_ART: u16 = 112; // Meadow Wall Art
    pub const HARBOR_STOOL: u16 = 113; // Harbor Stool
    pub const HARBOR_BUOY_LIGHT: u16 = 114; // Harbor Buoy Light
    pub const HARBOR_BUNK: u16 = 115; // Harbor Bunk
    pub const WOVEN_WALL_HANGING: u16 = 120; // Woven Wall Hanging
    pub const LOOM_STOOL: u16 = 121; // Loom Stool
    pub const SHIP_S_LANTERN: u16 = 122; // Ship's Lantern
    pub const SHIP_S_WHEEL: u16 = 123; // Ship's Wheel
    pub const HERB_PLANTER: u16 = 124; // Herb Planter
    pub const GARDEN_ARCH_LAMP: u16 = 125; // Garden Arch Lamp
    pub const TINKER_S_CABINET: u16 = 126; // Tinker's Cabinet
    pub const BRASS_CLOCK: u16 = 127; // Brass Clock
    pub const TOWN_PENNANT: u16 = 130; // Town Pennant
    pub const LANTERN_POST: u16 = 131; // Lantern Post
    pub const READING_NOOK_SOFA: u16 = 132; // Reading Nook Sofa
    pub const KEEPSAKE_CHEST: u16 = 133; // Keepsake Chest
    pub const INITIATIVE_TEE: u16 = 134; // Initiative Tee
    pub const HARBOR_STRIPE_TEE: u16 = 135; // Harbor Stripe Tee
    pub const FERRY_PASS: u16 = 136; // Ferry Pass
    pub const SEA_GLASS_POUCH: u16 = 137; // Sea Glass Pouch
    pub const JUNIPER_S_PORTRAIT: u16 = 140; // Juniper's Portrait
    pub const BRAMBLE_S_PORTRAIT: u16 = 141; // Bramble's Portrait
    pub const MOSSY_S_PORTRAIT: u16 = 142; // Mossy's Portrait
    pub const OTTO_S_PORTRAIT: u16 = 143; // Otto's Portrait
    pub const CLOVER_S_PORTRAIT: u16 = 144; // Clover's Portrait
    pub const BASIL_S_PORTRAIT: u16 = 145; // Basil's Portrait
    pub const BLOSSOM_LANTERN: u16 = 146; // Blossom Lantern
    pub const PAPER_BOAT_MODEL: u16 = 147; // Paper Boat Model
    pub const FROST_LANTERN: u16 = 148; // Frost Lantern
    pub const SUNRISE_BANNER: u16 = 149; // Sunrise Banner
    pub const VELVET_ARMCHAIR: u16 = 151; // Velvet Armchair
    pub const CRYSTAL_LAMP: u16 = 152; // Crystal Lamp
    pub const GRAND_WARDROBE: u16 = 153; // Grand Wardrobe
    pub const WATERING_CAN: u16 = 154; // Watering Can
    pub const GOLDEN_HOME_PLAQUE: u16 = 169; // Golden Home Plaque
    pub const SUNBELL_SEEDS: u16 = 160; // Sunbell Seeds
    pub const MOONPETAL_SEEDS: u16 = 161; // Moonpetal Seeds
    pub const EMBERBLOOM_SEEDS: u16 = 162; // Emberbloom Seeds
    pub const SKYCUP_SEEDS: u16 = 163; // Skycup Seeds
    pub const SUNBELL: u16 = 164; // Sunbell
    pub const MOONPETAL: u16 = 165; // Moonpetal
    pub const EMBERBLOOM: u16 = 166; // Emberbloom
    pub const SKYCUP: u16 = 167; // Skycup
    pub const STARFLECK_BLOOM: u16 = 168; // Starfleck Bloom
    pub const LOST_SCARF: u16 = 150; // Lost Scarf
    pub const BRAMBLE_S_GARDEN: u16 = 106; // Bramble's Garden
    pub const HARBOR_FOG: u16 = 107; // Harbor Fog
    pub const LANTERN_NIGHT: u16 = 103; // Lantern Night
    pub const POND_IN_SUMMER: u16 = 104; // Pond in Summer
    pub const FIRST_SNOW: u16 = 105; // First Snow
    pub const APPLE: u16 = 20; // Apple
    pub const PEAR: u16 = 21; // Pear
    pub const CHERRY: u16 = 22; // Cherry
    pub const PEACH: u16 = 23; // Peach
    pub const SCALLOP: u16 = 40; // Scallop
    pub const CONCH: u16 = 41; // Conch
    pub const SAND_DOLLAR: u16 = 42; // Sand Dollar
    pub const AMBER_STONE: u16 = 45; // Amber Stone
    pub const OLD_COIN: u16 = 46; // Old Coin
    pub const UNKNOWN_FOSSIL: u16 = 47; // Unknown Fossil
    pub const FERN_FOSSIL: u16 = 43; // Fern Fossil
    pub const SHARK_TOOTH: u16 = 44; // Shark Tooth
    pub const AMMONITE: u16 = 48; // Ammonite
    pub const TRILOBITE: u16 = 49; // Trilobite
    pub const CHAIR: u16 = 50; // Chair
    pub const SOFA: u16 = 51; // Sofa
    pub const TABLE: u16 = 52; // Table
    pub const BED: u16 = 53; // Bed
    pub const LAMP: u16 = 54; // Lamp
    pub const CABINET: u16 = 55; // Cabinet
    pub const POTTED_PLANT: u16 = 56; // Potted Plant
    pub const PAINTING: u16 = 57; // Painting
    pub const SEA_STAR: u16 = 70; // Sea Star
    pub const SEA_URCHIN: u16 = 71; // Sea Urchin
    pub const SEA_SLUG: u16 = 72; // Sea Slug
    pub const OCTOPUS: u16 = 73; // Octopus
    pub const DRIFTWOOD: u16 = 82; // Driftwood
    pub const MOON_SHELL: u16 = 84; // Moon Shell
    pub const SEA_GLASS: u16 = 85; // Sea Glass
    pub const GLASS_SHRIMP: u16 = 74; // Glass Shrimp
    pub const PALM_CRAB: u16 = 75; // Palm Crab
    pub const DRIFTWOOD_STOOL: u16 = 58; // Driftwood Stool
    pub const GLASS_FLOAT: u16 = 59; // Glass Float
    pub const PARCEL: u16 = 86; // Parcel
    pub const BRANCH: u16 = 80; // Branch
    pub const STONE: u16 = 81; // Stone
    pub const STRAW_HAT: u16 = 67; // Straw Hat
    pub const BEANIE: u16 = 68; // Beanie
    pub const FLOWER_CROWN: u16 = 69; // Flower Crown
    pub const SKY_TEE: u16 = 63; // Sky Tee
    pub const SUNFLOWER_TEE: u16 = 64; // Sunflower Tee
    pub const FOREST_HOODIE: u16 = 65; // Forest Hoodie
    pub const BERRY_SWEATER: u16 = 66; // Berry Sweater
    pub const RIVERGLASS_PLAQUE: u16 = 170; // Riverglass Plaque
    pub const LEAFPRINT_PLAQUE: u16 = 171; // Leafprint Plaque
    pub const NET: u16 = 60; // Net
    pub const FISHING_ROD: u16 = 61; // Fishing Rod
    pub const SHOVEL: u16 = 62; // Shovel
    pub const STUN_PISTOL: u16 = 172; // Stun Pistol
    pub const TANGLE_LAUNCHER: u16 = 173; // Tangle Launcher
    pub const SHOCK_STAFF: u16 = 174; // Shock Staff
    pub const FROST_CRYSTAL: u16 = 175; // Frost Crystal
    pub const GLOWCAP: u16 = 176; // Glowcap
}
