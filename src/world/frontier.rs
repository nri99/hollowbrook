//! Dustreach: an original desert frontier outpost reached through a gate (no door from Hollowbrook yet; dev entry
//! is F9 with HOLLOWBROOK_DEBUG). Players take bounties at the Guild Post board, track the target with the tracker,
//! detain them (runners flee, some hide among lookalikes) and turn them in for coins and rank.
//! All names, rules and numbers here are original and provisional.
use super::*;

pub const FW: i32 = 40;
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
pub const F_STRUCTS: [(u8, i32, i32, i32, i32); 15] = [
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
];
/// Hiding spots and how the broker describes them.
pub const F_SPOTS: [((i32, i32), &str); 11] = [
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
    for x in 4..38 {
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
        (22, 25),
        (38, 9),
        (24, 20),
        (3, 26),
    ] {
        set(x, z, ROCK)
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
    pub fn bounty_offers(&self) -> Vec<Offer> {
        (0..3u8)
            .map(|k| {
                let h = mix(self.w.day as u64 * 7 + k as u64, self.w.p.id as u64 + 11);
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
        self.w.p.bounty_log.iter().any(|&(d, o)| d == self.w.day && o == k)
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
        self.w.p.bounty =
            Some(Bounty { day: self.w.day, offer: k, tier: o.tier, pos, caught: false, run: 0., stun: 0. });
        Ok(format!("Bounty taken: {}. Last seen {}.", o.name, F_SPOTS[o.spot].1))
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
        let reward = BOUNTY_REWARD[b.tier as usize];
        let before = rank_of(self.w.p.bounties_done);
        self.w.p.money = self.w.p.money.saturating_add(reward);
        self.w.p.bounties_done += 1;
        self.w.p.bounty_log.push((b.day, b.offer));
        self.w.p.bounty_log.retain(|&(d, _)| d + 7 >= self.w.day);
        self.w.p.bounty = None;
        self.snd.push(Snd::Coin);
        let after = rank_of(self.w.p.bounties_done);
        self.note(format!("Bounty paid: {reward} coins."));
        if before != after {
            self.note(format!("The guild names you {after}!"))
        }
        Ok(reward)
    }
    /// Distance from the active player to their target, if a hunt is on.
    pub fn bounty_dist(&self) -> Option<f32> {
        let b = self.w.p.bounty.as_ref()?;
        (!b.caught).then(|| self.me.ipos.distance(vec2(b.pos[0][0], b.pos[0][1])))
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
    /// Use the equipped gear (called from use_tool). Gear stays holstered outside Dustreach.
    pub(super) fn use_gear(&mut self, d: u16) {
        if self.me.scene != Scene::Frontier {
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
        if self.me.scene != Scene::Frontier {
            self.me.bolts.clear();
            return;
        }
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
            if s.life <= 0. || !matches!(ftile(fmap, t.0, t.1), SAND | PATH) {
                return false;
            }
            if let Some(b) = bounty.as_mut().filter(|b| !b.caught) {
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
        let Some(b) = self.w.p.bounty.as_mut() else { return };
        if b.caught {
            return;
        }
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
            if f_walk(&self.fmap, t.0, t.1) {
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
