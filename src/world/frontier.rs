//! Dustreach: an original desert frontier outpost reached through a gate (no door from Hollowbrook yet; dev entry
//! is F9 with HOLLOWBROOK_DEBUG). Players take bounties at the Guild Post board, track the target with the tracker,
//! detain them (runners flee, some hide among lookalikes) and turn them in for coins and rank.
//! All names, rules and numbers here are original and provisional.
use super::*;

pub const FW: i32 = 60; // the east side (x 40+) holds the scrapyard, the old watchtower and the salt flats
pub const FH: i32 = 30;
/// Where players appear when they come through the gate, and the gate back to Hollowbrook.
pub const F_SPAWN: (f32, f32) = (15.5, 26.5);
pub const F_GATE: (i32, i32) = (15, 28);
/// The oasis pond (centre, radii in tiles) and the natural stone arch (left pillar; the right one is 2 tiles east).
pub const F_OASIS: (f32, f32, f32, f32) = (33.5, 21.5, 2.6, 1.9);
pub const F_ARCH: (i32, i32) = (32, 14);
/// The Guild Post bounty board and the broker who stands beside it.
pub const F_BOARD: (i32, i32) = (13, 9);
pub const F_BROKER: (f32, f32) = (14.6, 9.6);
/// Structures: (kind, x, z, w, d). 0 Guild Post, 1 Longhouse, 2 market tent, 3 water tower, 4 wreck, 5 dome hut.
pub const F_STRUCTS: [(u8, i32, i32, i32, i32); 20] = [
    (0, 11, 5, 5, 4),
    (1, 18, 5, 6, 4),
    (2, 6, 12, 2, 2),
    (2, 9, 12, 2, 2),
    (2, 20, 12, 2, 2),
    (3, 24, 16, 2, 2),
    (4, 3, 2, 5, 3),
    (5, 5, 17, 2, 2),
    (5, 22, 19, 2, 2),
    (5, 26, 9, 2, 2),
    (5, 7, 7, 2, 2),
    (5, 34, 5, 2, 2),
    (5, 28, 25, 2, 2),
    (2, 28, 6, 2, 2),
    (3, 7, 23, 2, 2),
    (7, 44, 6, 3, 2), // scrapyard heaps
    (7, 50, 12, 2, 2),
    (8, 54, 4, 2, 2), // ruined watchtower
    (5, 47, 20, 2, 2),
    (2, 44, 14, 2, 2),
];
/// Where you stand to use the ship's ramp (holding pod), where the companion waits, and the guild master.
pub const F_RAMP: (f32, f32) = (24.0, 26.6);
pub const F_PAL_SPOT: (f32, f32) = (30.5, 18.5);
/// Hunter armour forged by the guild master for guild marks: (name, marks).
pub const ARMOR: [(&str, u32); 4] = [("Helmet", 3), ("Chest plate", 4), ("Gauntlets", 2), ("Boots", 2)];
/// Mandalorian beskar gear forged by Varro: (name, marks, beskar, mando bit).
pub const MANDO_FORGE: [(&str, u32, u32, u8); 3] =
    [("Jetpack (Q dash)", 3, 1, 1), ("Whistling Darts (R)", 2, 1, 2), ("Clan Signet", 2, 1, 4)];
/// Longhouse cantina entrance in Dustreach & interior coordinates (6x5 room).
pub const F_CANTINA: (f32, f32) = (21.0, 9.5);
pub const CT_EXIT: (f32, f32) = (3.0, 4.55);
pub const CT_BROKER: (f32, f32) = (1.5, 1.8);
pub fn ct_block(x: i32, z: i32) -> bool {
    !(0..6).contains(&x) || !(0..5).contains(&z) || z == 0 || (x == 5 && (1..=2).contains(&z)) || (x == 0 && z == 1)
}
/// Guild marks paid per bounty tier, on top of the coins.
pub const MARKS: [u32; 3] = [1, 2, 3];
/// Where the hover-bike parks beside your ship.
pub const F_BIKE: (f32, f32) = (19.6, 24.6);
/// Townsfolk: home spot, name, what they say. Index 2 is the informant who sells a bearing on your target.
pub const LOCALS: [((f32, f32), &str, &str); 4] = [
    ((8.5, 14.6), "Old Sel", "Spice, rope, sun-cloth. Everything a long walk needs."),
    ((21.0, 14.6), "Kip", "Water's two coins a cup today. The oasis is free, if you like grit."),
    ((18.5, 11.6), "Moss", "I hear things. For 60 coins I'll tell you which way your mark went."),
    ((27.5, 27.0), "Dena", "Want to tune up that freighter? Bring coins and I'll bolt on real hardware."),
];
pub const INFORMANT: usize = 2;
pub const MECHANIC: usize = 3;
pub const HINT_PRICE: u32 = 60;
/// Townsfolk drift around their spot; the same function drives drawing and talking.
pub fn local_pos(i: usize, t: f32) -> Vec2 {
    let (x, z) = LOCALS[i].0;
    let k = i as f32 * 1.7;
    vec2(x + (t * 0.21 + k).sin() * 0.9, z + (t * 0.17 + k * 2.).cos() * 0.5)
}
/// Compass word from `a` towards `b` (north is up the screen, towards smaller z).
pub fn bearing(a: Vec2, b: Vec2) -> &'static str {
    let d = b - a;
    let ang = d.x.atan2(-d.y).to_degrees().rem_euclid(360.);
    ["north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west"]
        [((ang + 22.5) / 45.) as usize % 8]
}
/// Quick-draw window: press Space within this many seconds of "DRAW!".
pub const DRAW_WINDOW: f32 = 0.6;
pub const RUMOURS: [&str; 3] = [
    "They're worn out and won't run far.",
    "Word is they bolt the moment they see you. A stun pistol helps.",
    "They've paid two lookalikes to wander about. Watch for the red scarf.",
];
/// Hiding spots and how the broker describes them.
pub const F_SPOTS: [((i32, i32), &str); 14] = [
    ((6, 6), "by the old wreck in the northwest"),
    ((26, 3), "among the red rocks in the northeast"),
    ((8, 15), "behind the market tents"),
    ((25, 13), "near the water tower"),
    ((3, 20), "out in the southwest dunes"),
    ((27, 21), "past the huts in the southeast"),
    ((22, 9), "round the back of the Longhouse"),
    ((2, 11), "along the western ridge"),
    ((29, 21), "resting by the oasis"),
    ((37, 3), "under the far north mesas"),
    ((33, 15), "beneath the stone arch"),
    ((46, 9), "picking through the scrapyard"),
    ((55, 8), "below the ruined watchtower"),
    ((52, 23), "out on the salt flats"),
];
const NAMES: [&str; 12] = [
    "Corvin Slake",
    "Ada Brisk",
    "Tobin Marr",
    "Wren Halloway",
    "Dax Fennick",
    "Mira Cole",
    "Jory Vance",
    "Esk Talloway",
    "Nell Garrow",
    "Bram Quill",
    "Sable Ott",
    "Rook Danvers",
];
const CRIMES: [&str; 8] = [
    "skipped out on a water debt",
    "smuggled glowmoss past the toll",
    "rustled a herd of sand runners",
    "forged trading permits",
    "stole a crate of solar cells",
    "broke out of the stockade",
    "sold fake maps to travellers",
    "jumped a supply caravan",
];
/// Scarf colours: the real target wears the first, lookalikes the others.
pub const SCARVES: [(&str, (f32, f32, f32)); 3] =
    [("red", (0.85, 0.2, 0.18)), ("blue", (0.2, 0.4, 0.85)), ("green", (0.25, 0.65, 0.3))];
/// Tier 0 waits, tier 1 runs when you get close, tier 2 hides among two lookalikes (and runs).
pub const BOUNTY_REWARD: [u32; 3] = [400, 900, 1600];
pub const RANKS: [(u32, &str); 5] = [(0, "Drifter"), (3, "Tracker"), (8, "Hunter"), (15, "Warden"), (30, "Legend")];

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Bounty {
    pub day: i64,
    pub offer: u8,
    pub tier: u8,
    /// Target first, then lookalikes (tier 2). Positions in outpost coordinates.
    pub pos: Vec<[f32; 2]>,
    pub caught: bool,
    /// Seconds left of the current run; negative while the target catches its breath.
    #[serde(default)]
    pub run: f32,
    /// Seconds the target stays stunned by gear.
    #[serde(default)]
    pub stun: f32,
    /// Locked in the ship's holding pod (ready to be paid at the board).
    #[serde(default)]
    pub delivered: bool,
    /// Runners and lookalike targets ambush once when you first get close.
    #[serde(default)]
    pub ambushed: bool,
    /// The informant has been paid for this bounty: the HUD shows a live bearing.
    #[serde(default)]
    pub hinted: bool,
    /// Planet index where the target is hiding (0 Dustreach, 1 Glacia, 2 Verdance).
    #[serde(default)]
    pub planet: u8,
}
/// Frontier gear (original designs, non-lethal, works only in Dustreach): item id, price, what it does.
pub const GEAR: [(u16, u32, &str); 3] = [
    (172, 1500, "Snaps a stun bolt across the street. Stops a runner for a few seconds."),
    (173, 2500, "Fires weighted cords that tangle a target's legs. Longer stun, slower to reload."),
    (174, 1000, "A charged staff for close work. Stuns anyone right in front of you."),
];
pub fn is_gear(d: u16) -> bool {
    GEAR.iter().any(|g| g.0 == d)
}
/// A shot in flight (runtime only): position, velocity, seconds left, kind (0 stun bolt, 1 tangle cords).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Bolt {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub kind: u8,
}
pub struct Offer {
    pub name: &'static str,
    pub crime: &'static str,
    pub tier: u8,
    pub spot: usize,
    pub reward: u32,
}

fn mix(a: u64, b: u64) -> u64 {
    let mut x = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_add(0x632B_E59B_D9B4_E019);
    x ^= x >> 29;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^ (x >> 32)
}
pub fn frontier_map() -> Vec<u8> {
    let mut m = vec![SAND; (FW * FH) as usize];
    let mut set = |x: i32, z: i32, c: u8| {
        if (0..FW).contains(&x) && (0..FH).contains(&z) {
            m[(z * FW + x) as usize] = c
        }
    };
    for z in 0..FH {
        for x in 0..FW {
            if x == 0 || z == 0 || x == FW - 1 || z == FH - 1 {
                set(x, z, ROCK) // canyon walls ring the outpost
            }
        }
    }
    for z in 9..FH - 1 {
        set(15, z, PATH);
        set(16, z, PATH)
    } // main street from the gate to the Guild Post
    for x in 4..FW - 3 {
        set(x, 10, PATH)
    } // cross street, now running out to the arch and the oasis
    for z in 11..18 {
        set(33, z, PATH)
    } // a trail south under the arch to the oasis
    let (ox, oz, rx, rz) = F_OASIS;
    for z in 0..FH {
        for x in 0..FW {
            let (dx, dz) = ((x as f32 + 0.5 - ox) / rx, (z as f32 + 0.5 - oz) / rz);
            if dx * dx + dz * dz < 1. {
                set(x, z, SEA)
            }
        }
    }
    set(F_ARCH.0, F_ARCH.1, ROCK);
    set(F_ARCH.0 + 2, F_ARCH.1, ROCK);
    for &(x, z) in &[
        (25, 2),
        (26, 2),
        (27, 4),
        (1, 9),
        (12, 18),
        (19, 16),
        (28, 14),
        (9, 21),
        (37, 25),
        (38, 9),
        (24, 20),
        (3, 26),
    ] {
        set(x, z, ROCK)
    }
    for &(x, z, w, d) in &F_PADS {
        for zz in z..z + d {
            for xx in x..x + w {
                set(xx, zz, ROCK) // landing pads (a ship or a raised empty platform)
            }
        }
    }
    for &(_, x, z, w, d) in &F_STRUCTS {
        for zz in z..z + d {
            for xx in x..x + w {
                set(xx, zz, ROCK)
            }
        }
    }
    set(F_GATE.0, F_GATE.1, PATH);
    set(F_GATE.0 + 1, F_GATE.1, PATH);
    m
}
pub fn ftile(m: &[u8], x: i32, z: i32) -> u8 {
    if (0..FW).contains(&x) && (0..FH).contains(&z) {
        m[(z * FW + x) as usize]
    } else {
        ROCK
    }
}
pub fn f_walk(m: &[u8], x: i32, z: i32) -> bool {
    matches!(ftile(m, x, z), SAND | PATH) && (x, z) != F_BOARD
}
pub fn rank_of(done: u32) -> &'static str {
    RANKS.iter().rev().find(|r| done >= r.0).map_or("Drifter", |r| r.1)
}

impl Game {
    /// Today's three bounties for the active player (same list all day; different for each player).
    /// Which set of three bounties is up on `day`: a new set appears each time all three are brought in.
    pub fn bounty_batch(&self, day: i64) -> u8 {
        (self.w.p.bounty_log.iter().filter(|&&(d, _)| d == day).count() / 3) as u8
    }
    pub fn bounty_offers(&self) -> Vec<Offer> {
        let batch = self.bounty_batch(self.w.day) as u64;
        (0..3u8)
            .map(|k| {
                let h = mix(self.w.day as u64 * 7 + k as u64 + batch * 131, self.w.p.id as u64 + 11);
                Offer {
                    name: NAMES[(h % NAMES.len() as u64) as usize],
                    crime: CRIMES[((h >> 8) % CRIMES.len() as u64) as usize],
                    tier: k,
                    spot: ((h >> 16) % F_SPOTS.len() as u64) as usize,
                    reward: BOUNTY_REWARD[k as usize],
                }
            })
            .collect()
    }
    pub fn bounty_done_today(&self, k: u8) -> bool {
        let code = k + 3 * self.bounty_batch(self.w.day); // log entries store offer + 3 * set number
        self.w.p.bounty_log.iter().any(|&(d, o)| d == self.w.day && o == code)
    }
    /// Accept offer `k`. One bounty at a time; each offer once per day.
    pub fn take_bounty(&mut self, k: u8) -> Result<String, &'static str> {
        if self.w.p.bounty.is_some() {
            return Err("Finish or drop your current bounty first.");
        }
        if self.bounty_done_today(k) {
            return Err("You already brought that one in today.");
        }
        let o = self.bounty_offers().into_iter().nth(k as usize).ok_or("No such bounty.")?;
        let (sx, sz) = F_SPOTS[o.spot].0;
        let mut pos = vec![[sx as f32 + 0.5, sz as f32 + 0.5]];
        if o.tier == 2 {
            for (dx, dz) in [(-3, 1), (3, -1)] {
                let t = (sx + dx, sz + dz);
                let t = if f_walk(&self.fmap, t.0, t.1) { t } else { (sx, sz + dz.signum()) };
                pos.push([t.0 as f32 + 0.5, t.1 as f32 + 0.5]);
            }
        }
        self.w.p.bounty = Some(Bounty {
            day: self.w.day,
            offer: k,
            tier: o.tier,
            pos,
            caught: false,
            run: 0.,
            stun: 0.,
            delivered: false,
            ambushed: false,
            hinted: false,
            planet: 0,
        });
        Ok(format!("Bounty taken: {}. Last seen {}.", o.name, F_SPOTS[o.spot].1))
    }
    /// Accept an off-world cantina contract on Glacia (1) or Verdance (2).
    pub fn take_offworld_bounty(&mut self, planet: u8) -> Result<String, &'static str> {
        if self.w.p.bounty.is_some() {
            return Err("Finish or drop your current bounty first.");
        }
        if !(1..=2).contains(&planet) {
            return Err("No contract for that sector.");
        }
        let s = (self.w.day.rem_euclid(SAMPLE_SPOTS[0].len() as i64)) as usize;
        let (sx, sz) = SAMPLE_SPOTS[(planet - 1) as usize][s];
        let mut pos = vec![[sx as f32 + 0.5, sz as f32 + 0.5]];
        for (dx, dz) in [(-1, 1), (1, -1)] {
            let (tx, tz) = if p_walk(planet, sx + dx, sz + dz) { (sx + dx, sz + dz) } else { (sx, sz) };
            pos.push([tx as f32 + 0.5, tz as f32 + 0.5]);
        }
        self.w.p.bounty = Some(Bounty {
            day: self.w.day,
            offer: 2,
            tier: 2,
            pos,
            caught: false,
            run: 0.,
            stun: 0.,
            delivered: false,
            ambushed: false,
            hinted: false,
            planet,
        });
        let name = self.bounty_offers()[2].name;
        Ok(format!("Off-world puck: {name} on {}. Fly there, stun & haul them back!", PLANETS[planet as usize].0))
    }
    pub fn drop_bounty(&mut self) {
        if self.w.p.bounty.take().is_some() {
            self.note("Bounty dropped. The broker shrugs.")
        }
    }
    /// Detain figure `i` (0 = the real target). Lookalikes cost nothing but time.
    pub fn detain(&mut self, i: usize) {
        let Some(b) = self.w.p.bounty.as_mut() else { return };
        if b.caught {
            return;
        }
        if i == 0 {
            b.caught = true;
            let name = self.bounty_offers()[self.w.p.bounty.as_ref().unwrap().offer as usize].name;
            self.snd.push(Snd::Coin);
            self.note(format!("Detained {name}! Take them to the Guild Post board."));
        } else {
            self.note(format!("Wrong person. Your target wears a {} scarf.", SCARVES[0].0));
        }
    }
    /// Turn in a detained target at the board: pays once, raises rank.
    pub fn turn_in(&mut self) -> Result<u32, &'static str> {
        let b = self.w.p.bounty.clone().ok_or("No bounty to turn in.")?;
        if !b.caught {
            return Err("You haven't detained them yet.");
        }
        if !b.delivered {
            return Err("Lock them in your ship's holding pod first.");
        }
        self.w.p.marks += MARKS[b.tier as usize];
        let base = BOUNTY_REWARD[b.tier as usize] + if b.planet > 0 { 500 } else { 0 };
        let reward = base + base * bounty_bonus_pct(self.w.p.ship_up[2]) / 100;
        if b.planet > 0 {
            self.w.p.beskar += 1;
        }
        let before = rank_of(self.w.p.bounties_done);
        self.w.p.money = self.w.p.money.saturating_add(reward);
        self.w.p.bounties_done += 1;
        let set_done = if b.planet == 0 {
            let batch = self.bounty_batch(b.day);
            self.w.p.bounty_log.push((b.day, b.offer + 3 * batch));
            self.bounty_batch(b.day) > batch
        } else {
            false
        };
        self.w.p.bounty_log.retain(|&(d, _)| d + 7 >= self.w.day);
        self.w.p.bounty = None;
        self.snd.push(Snd::Coin);
        let after = rank_of(self.w.p.bounties_done);
        self.note(format!("Bounty paid: {reward} coins."));
        if set_done && b.day == self.w.day {
            self.note("The guild master pins three fresh bounties to the board.");
        }
        if before != after {
            self.note(format!("The guild names you {after}!"))
        }
        Ok(reward)
    }
    /// Distance from the active player to their target on the current planet, if a hunt is on.
    pub fn bounty_dist(&self) -> Option<f32> {
        let cur = match self.me.scene {
            Scene::Frontier => 0,
            Scene::Planet(k) => k,
            _ => return None,
        };
        let b = self.w.p.bounty.as_ref().filter(|b| !b.caught && b.planet == cur)?;
        Some(self.me.ipos.distance(vec2(b.pos[0][0], b.pos[0][1])))
    }
    pub fn tracker_word(&self) -> &'static str {
        match self.bounty_dist() {
            None => "",
            Some(d) if d < 4. => "target in sight",
            Some(d) if d < 8. => "hot",
            Some(d) if d < 14. => "warm",
            _ => "cold",
        }
    }
    /// Per-player frame update in the outpost: runners flee when the hunter gets close.
    /// Walk a detained target up the ramp into the ship's holding pod.
    pub fn deliver(&mut self) -> Result<(), &'static str> {
        let b = self.w.p.bounty.as_mut().filter(|b| b.caught && !b.delivered).ok_or("The holding pod is empty.")?;
        b.delivered = true;
        let name = self.bounty_offers()[self.w.p.bounty.as_ref().unwrap().offer as usize].name;
        self.snd.push(Snd::Door);
        self.note(format!("{name} is locked in the holding pod. Collect the pay at the Guild Post board."));
        Ok(())
    }
    /// The guild master forges armour piece `k` (0..4) or beskar gear (`4..7`).
    pub fn forge(&mut self, k: usize) -> Result<(), &'static str> {
        if let Some(&(name, marks, beskar, bit)) = k.checked_sub(ARMOR.len()).and_then(|i| MANDO_FORGE.get(i)) {
            if self.w.p.mando & bit != 0 {
                return Err("You already carry that.");
            }
            if bit == 1 && self.w.p.armor != 0b1111 {
                return Err("Forge all 4 armour pieces first for a jetpack.");
            }
            if self.w.p.marks < marks {
                return Err("Not enough guild marks.");
            }
            if self.w.p.beskar < beskar {
                return Err("Need Beskar ore (from orbit or off-world hunts).");
            }
            self.w.p.marks -= marks;
            self.w.p.beskar -= beskar;
            self.w.p.mando |= bit;
            self.snd.push(Snd::Coin);
            self.note(format!("Varro forges your {name}. This is the way."));
            return Ok(());
        }
        let (name, cost) = ARMOR[k];
        if self.w.p.armor & (1 << k) != 0 {
            return Err("You already wear that piece.");
        }
        if self.w.p.marks < cost {
            return Err("Not enough guild marks. Bring in more bounties.");
        }
        self.w.p.marks -= cost;
        self.w.p.armor |= 1 << k;
        self.snd.push(Snd::Coin);
        self.note(format!("The guild master hammers out your {name}. You wear it in Dustreach."));
        Ok(())
    }
    /// Short jetpack burst (`Q`) in Dustreach or on planets.
    pub fn jetpack_dash(&mut self) {
        if self.w.p.mando & 1 == 0 || !matches!(self.me.scene, Scene::Frontier | Scene::Planet(_)) {
            return;
        }
        if self.me.jet_cd > 0. {
            return self.note("Jetpack recharging...");
        }
        self.me.jet = 0.55;
        self.me.jet_cd = 2.5;
        self.snd.push(Snd::Swim);
    }
    /// Whistling darts (`R`): stuns an uncaught target on the current planet within range.
    pub fn whistling_darts(&mut self) {
        if self.w.p.mando & 2 == 0 || !matches!(self.me.scene, Scene::Frontier | Scene::Planet(_)) {
            return;
        }
        let cur = if self.me.scene == Scene::Frontier {
            0
        } else if let Scene::Planet(k) = self.me.scene {
            k
        } else {
            return;
        };
        self.snd.push(Snd::Blip);
        self.me.swing = 0.35;
        if let Some(b) = self.w.p.bounty.as_mut().filter(|b| !b.caught && b.planet == cur) {
            let p = vec2(b.pos[0][0], b.pos[0][1]);
            if p.distance(self.me.ipos) < 7.5 {
                b.stun = 4.0;
                b.run = b.run.min(0.);
                return self.note("Whistling darts lock on! Target stunned.");
            }
        }
        self.note("Whistling darts find no target in range.");
    }
    pub fn armor_pieces(&self) -> u32 {
        self.w.p.armor.count_ones()
    }
    /// Talk to townsperson `i`. The informant sells a live bearing on your target; Dena opens the shipyard.
    pub fn talk_local(&mut self, i: usize) {
        if i == MECHANIC {
            self.me.ui = Ui::ShipYard(0);
            return;
        }
        let (_, name, line) = LOCALS[i];
        if i == INFORMANT {
            match self.w.p.bounty.as_ref() {
                Some(b) if !b.caught && b.hinted => {
                    return self.note(format!("{name}: \"I've told you all I know. Follow the bearing.\""))
                }
                Some(b) if !b.caught => {
                    if self.w.p.money < HINT_PRICE {
                        return self.note(format!("{name}: \"Come back with {HINT_PRICE} coins.\""));
                    }
                    self.w.p.money -= HINT_PRICE;
                    let t = vec2(b.pos[0][0], b.pos[0][1]);
                    let dir = bearing(self.me.ipos, t);
                    self.w.p.bounty.as_mut().unwrap().hinted = true;
                    self.snd.push(Snd::Coin);
                    return self.note(format!("{name}: \"Your mark went {dir}. I'll keep you posted.\""));
                }
                _ => {}
            }
        }
        self.note(format!("{name}: \"{line}\""))
    }
    /// Space during a standoff: too early and the target bolts; in the DRAW window they are stunned.
    pub fn standoff_press(&mut self) {
        let early = self.me.standoff > DRAW_WINDOW;
        self.me.standoff = 0.;
        let Some(b) = self.w.p.bounty.as_mut().filter(|b| !b.caught) else { return };
        if early {
            b.run = 4.5;
            self.note("Too early! They bolt for cover.");
        } else {
            b.stun = 4.;
            b.run = 0.;
            self.snd.push(Snd::Coin);
            self.note("Quick draw! They're stunned. Detain them.");
        }
    }
    /// Climb on or off the hover-bike parked by your ship.
    pub fn toggle_bike(&mut self) {
        self.me.riding = !self.me.riding;
        self.snd.push(Snd::Door);
        self.note(if self.me.riding {
            "You fire up the hover-bike. Twice as fast; park it back by the ship."
        } else {
            "Hover-bike parked."
        });
    }
    /// The little sand-hopper at the oasis decides to follow you (once, kept forever).
    pub fn adopt_pal(&mut self) {
        if !self.w.p.pal {
            self.w.p.pal = true;
            self.me.pal_pos = vec2(F_PAL_SPOT.0, F_PAL_SPOT.1);
            self.snd.push(Snd::Blip);
            self.note("A little sand-hopper hops after you. It sniffs out bounties!");
        }
    }
    /// Buy gear `k` at the Guild Post. One of each; equips it straight away.
    pub fn buy_gear(&mut self, k: usize) -> Result<(), &'static str> {
        let (d, price, _) = GEAR[k];
        if self.w.p.tools.contains(&d) {
            return Err("You already carry one.");
        }
        if self.w.p.money < price {
            return Err("Not enough coins.");
        }
        self.w.p.money -= price;
        self.w.p.tools.push(d);
        self.w.p.equip = Some(d);
        self.snd.push(Snd::Coin);
        self.note(format!("Bought the {}. Space to use it, Tab to switch.", def(d).name));
        Ok(())
    }
    /// Use the equipped gear (called from use_tool). Works in Dustreach and on landing sites.
    pub(super) fn use_gear(&mut self, d: u16) {
        if !matches!(self.me.scene, Scene::Frontier | Scene::Planet(_)) {
            return self.note("Hollowbrook is peaceful. Your gear stays holstered here.");
        }
        if self.me.swing > 0. {
            return;
        }
        let f = self.me.face.normalize_or_zero();
        let at = self.me.ipos + f * 0.5;
        self.snd.push(Snd::Blip);
        match d {
            172 => {
                self.me.swing = 0.3;
                self.me.bolts.push(Bolt { pos: at, vel: f * 14., life: 0.6, kind: 0 });
            }
            173 => {
                self.me.swing = 0.9;
                self.me.bolts.push(Bolt { pos: at, vel: f * 8., life: 0.9, kind: 1 });
            }
            _ => {
                self.me.swing = 0.45;
                self.fx.push((vec3(at.x + f.x * 0.4, 0.7, at.y + f.y * 0.4), 0., 2));
                let me = self.me.ipos;
                let mut stunned = false;
                if let Some(b) = self.w.p.bounty.as_mut().filter(|b| !b.caught) {
                    let p = vec2(b.pos[0][0], b.pos[0][1]);
                    if p.distance(me) < 1.6 && (p - me).normalize_or_zero().dot(f) > 0.3 {
                        b.stun = 3.;
                        b.run = b.run.min(0.);
                        stunned = true;
                    }
                }
                if stunned {
                    self.note("Stunned! Detain them now.");
                }
            }
        }
    }
    pub fn frontier_tick(&mut self, dt: f32) {
        self.me.jet = (self.me.jet - dt).max(0.);
        self.me.jet_cd = (self.me.jet_cd - dt).max(0.);
        self.me.pal_cd = (self.me.pal_cd - dt).max(0.);
        if self.me.flight.is_some() {
            return; // nothing on the ground moves for you while you're flying
        }
        let planet = match self.me.scene {
            Scene::Frontier => 0,
            Scene::Planet(k) => {
                self.me.riding = false;
                k
            }
            _ => {
                self.me.bolts.clear();
                self.me.standoff = 0.;
                self.me.riding = false; // the bike stays in Dustreach
                return;
            }
        };
        let me = self.me.ipos;
        // shots in flight: walls stop them; the real target is stunned, lookalikes just flinch
        let mut bolts = std::mem::take(&mut self.me.bolts);
        let mut hit = false;
        let fmap = &self.fmap;
        let bounty = &mut self.w.p.bounty;
        bolts.retain_mut(|s| {
            s.pos += s.vel * dt;
            s.life -= dt;
            let t = tile_of(s.pos);
            let clear =
                if planet == 0 { matches!(ftile(fmap, t.0, t.1), SAND | PATH) } else { p_walk(planet, t.0, t.1) };
            if s.life <= 0. || !clear {
                return false;
            }
            if let Some(b) = bounty.as_mut().filter(|b| !b.caught && b.planet == planet) {
                if let Some(i) = b.pos.iter().position(|p| vec2(p[0], p[1]).distance(s.pos) < 0.45) {
                    if i == 0 {
                        b.stun = if s.kind == 1 { 5. } else { 2.5 };
                        b.run = b.run.min(0.);
                        hit = true;
                    }
                    return false;
                }
            }
            true
        });
        self.me.bolts = bolts;
        if hit {
            self.note("Stunned! Detain them now.");
        }
        self.me.dazed = (self.me.dazed - dt).max(0.);
        // the companion trots along beside you, points out lookalikes, and startles fleeing targets
        let mut pal_chirp = false;
        if self.w.p.pal {
            let f = self.me.face.normalize_or_zero();
            let mut want = me - f * 0.8 + vec2(-f.y, f.x) * 0.45;
            if let Some(b) = self.w.p.bounty.as_mut().filter(|b| !b.caught && b.planet == planet) {
                let tgt = vec2(b.pos[0][0], b.pos[0][1]);
                if b.pos.len() > 1 && me.distance(tgt) < 6. {
                    want = tgt + vec2(0.7, 0.2); // lookalike reveal
                }
                if b.run > 0. && b.stun <= 0. && me.distance(tgt) < 3.5 && self.me.pal_cd <= 0. {
                    b.stun = 2.0;
                    b.run = 0.;
                    self.me.pal_cd = 10.;
                    pal_chirp = true;
                }
            }
            if self.me.pal_pos == Vec2::ZERO || self.me.pal_pos.distance(want) > 6. {
                self.me.pal_pos = want;
            } else {
                self.me.pal_pos += (want - self.me.pal_pos) * (dt * 4.).min(1.);
            }
        }
        if pal_chirp {
            self.snd.push(Snd::Blip);
            self.note("Your sand-hopper's chirp startles the fugitive!");
        }
        // the tracker beeps faster the closer the target is
        if let Some(d) = self.bounty_dist() {
            self.me.beep -= dt;
            if self.me.beep <= 0. {
                self.snd.push(Snd::Blip);
                self.me.beep = (d / 7.).clamp(0.12, 1.4);
            }
        }
        let armor = self.w.p.armor.count_ones() as f32;
        let mut ambush = false;
        let Some(b) = self.w.p.bounty.as_mut().filter(|b| b.planet == planet) else { return };
        if b.caught {
            if !b.delivered {
                // a detained target walks behind you to the ship
                let f = self.me.face.normalize_or_zero();
                let want = me - f * 0.9;
                let p = vec2(b.pos[0][0], b.pos[0][1]);
                if p.distance(want) > 0.3 {
                    let n = p + (want - p).normalize_or_zero() * (3.6 * dt).min(p.distance(want));
                    b.pos[0] = [n.x, n.y];
                }
            }
            return;
        }
        let p = vec2(b.pos[0][0], b.pos[0][1]);
        // the most wanted (lookalike tier) face you down: a quick-draw standoff instead of an ambush
        if b.tier == 2 && !b.ambushed && b.stun <= 0. && p.distance(me) < 3.2 {
            b.ambushed = true;
            let wait = 1.2 + (mix(self.w.day as u64, 7) % 13) as f32 * 0.1;
            self.me.standoff = wait + DRAW_WINDOW;
            self.note("Standoff... wait for it. Press Space the moment you see DRAW!");
            return;
        }
        if self.me.standoff > 0. && p.distance(me) > 5. {
            self.me.standoff = 0.; // out of range (e.g. moved away by a cutscene or a door): no standoff
            return;
        }
        if self.me.standoff > 0. {
            self.me.standoff -= dt;
            if self.me.standoff <= 0. {
                self.me.standoff = 0.;
                b.run = 4.5;
                self.note("Too slow! They got away from the standoff.");
            }
            return;
        }
        if b.tier >= 1 && !b.ambushed && b.stun <= 0. && p.distance(me) < 1.4 {
            // the target throws a fistful of sand and makes a break for it; armour softens it
            b.ambushed = true;
            b.run = 4.5;
            self.me.dazed = (1.6 - 0.3 * armor).max(0.4);
            ambush = true;
        }
        if ambush {
            self.note("Sand in your eyes! They're making a run for it.");
            return;
        }
        let Some(b) = self.w.p.bounty.as_mut() else { return };
        if b.stun > 0. {
            b.stun -= dt;
            return;
        }
        if b.tier == 0 {
            return;
        }
        let p = vec2(b.pos[0][0], b.pos[0][1]);
        if b.run <= 0. {
            b.run += dt; // resting (negative) counts back up to zero
            if b.run >= 0. && p.distance(me) < 2.6 {
                b.run = 4.5; // spooked: run for a while
            } else {
                b.run = b.run.min(0.);
                return;
            }
        }
        b.run -= dt;
        if b.run <= 0. {
            b.run = -3.; // out of breath: catch them now
        }
        let away = (p - me).normalize_or_zero();
        let side = vec2(-away.y, away.x);
        for d in [away, (away + side).normalize(), (away - side).normalize(), side, -side] {
            let n = p + d * 4.2 * dt;
            let t = tile_of(n);
            let ok = if planet == 0 { f_walk(&self.fmap, t.0, t.1) } else { p_walk(planet, t.0, t.1) };
            if ok {
                b.pos[0] = [n.x, n.y];
                break;
            }
        }
    }
    /// Day rollover: an uncaught bounty goes cold; a detained one can still be turned in.
    pub(super) fn expire_bounty(p: &mut Profile, day: i64) {
        if p.bounty.as_ref().is_some_and(|b| !b.caught && b.day != day) {
            p.bounty = None;
        }
    }
}
