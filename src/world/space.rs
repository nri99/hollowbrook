//! Spaceflight from Dustreach (original design): take off from your ship's ramp, fly around the planet, pick a
//! destination in the nav computer and jump to light speed, then fly to the planet and land.
//! Destinations: Dustreach (0), Glacia (1, an ice world) and Verdance (2, a jungle moon). The two new worlds are
//! small landing sites with resources to gather once a day. All numbers are provisional.
use super::*;

/// (name, one-line description) per planet; index 0 is Dustreach.
pub const PLANETS: [(&str, &str); 3] = [
    ("Dustreach", "the desert outpost and your guild"),
    ("Glacia", "an ice world of blue crystal spires"),
    ("Verdance", "a jungle moon lit by glowing fungus"),
];
/// Seconds spent in the light-speed tunnel.
pub const JUMP_TIME: f32 = 2.6;
/// Ship upgrades bought from Dena in Dustreach: (name, [lv1..3 coin costs], description).
pub const SHIP_UP_MAX: u8 = 3;
pub const SHIP_UPS: [(&str, [u32; 3], &str); 4] = [
    ("Thrusters", [1200, 2800, 5500], "Faster thrust, boost speed and turning in space."),
    ("Hyperdrive", [1500, 3200, 6000], "Shorter light-speed jumps and wider orbital landing range."),
    ("Hull & Wings", [1000, 2500, 5000], "Upgraded hull & wings, sharper brakes, and +15/30/50% bounty pay."),
    ("Scanner", [1200, 2800, 5200], "Resource beacons on planets and extra crystals/glowcaps per spot."),
];
pub fn ship_tier(up: [u8; 4]) -> u8 {
    up.iter().map(|&lv| lv.min(SHIP_UP_MAX)).sum()
}
pub fn jump_time(hyper_lv: u8) -> f32 {
    (JUMP_TIME - hyper_lv.min(SHIP_UP_MAX) as f32 * 0.55).max(0.9)
}
pub fn land_range(hyper_lv: u8) -> f32 {
    PLANET_R + 4. + hyper_lv.min(SHIP_UP_MAX) as f32 * 2.
}
pub fn bounty_bonus_pct(hull_lv: u8) -> u32 {
    [0, 15, 30, 50][hull_lv.min(SHIP_UP_MAX) as usize]
}
pub fn sample_yield(scan_lv: u8, spot: usize) -> u16 {
    match scan_lv.min(SHIP_UP_MAX) {
        0 => 1,
        1 => {
            if spot.is_multiple_of(2) {
                2
            } else {
                1
            }
        }
        2 => 2,
        _ => 3,
    }
}
/// Planet radius in flight space (the planet sits at the origin).
pub const PLANET_R: f32 = 5.;
/// Landing-site maps (Glacia and Verdance share a layout generator).
pub const PW: i32 = 30;
pub const PH: i32 = 22;
/// Your ship parks at the top middle (tiles x 13..19, z 3..7); its ramp faces south.
pub const P_SHIP: (i32, i32, i32, i32) = (13, 3, 6, 4);
pub const P_RAMP: (f32, f32) = (16.0, 7.6);
pub const P_SPAWN: (f32, f32) = (16.0, 8.2);
/// Resource spots per planet (gathered once a day per player) and the item each gives.
pub const SAMPLE_SPOTS: [[(i32, i32); 6]; 2] = [
    [(5, 10), (24, 9), (9, 18), (21, 18), (27, 14), (3, 16)],
    [(5, 10), (25, 9), (9, 18), (22, 18), (27, 16), (4, 14)],
];
pub const SAMPLE_ITEM: [u16; 2] = [175, 176]; // Frost Crystal, Glowcap
/// Each landing site has a pond: (centre x, centre z, radius x, radius z).
pub const P_POND: [(f32, f32, f32, f32); 2] = [(9.0, 14.0, 3.2, 2.2), (21.0, 13.0, 3.0, 2.0)];

/// Landing pads, one per player (index = pad_of(player id)): Dustreach and the landing sites. Pad 0 keeps the
/// original spot (F_RAMP / P_SHIP / P_RAMP / P_SPAWN are pad 0).
pub const F_PADS: [(i32, i32, i32, i32); 2] = [(21, 22, 6, 4), (41, 23, 6, 4)];
pub const P_PADS: [(i32, i32, i32, i32); 2] = [P_SHIP, (22, 3, 6, 4)];
pub fn pad_of(id: usize) -> usize {
    id.min(1)
}
/// Where you stand to board the ship on pad `i`: Dustreach (planet 0) or a landing site.
pub fn ramp_of(planet: u8, i: usize) -> Vec2 {
    let (x, z, w, d) = if planet == 0 { F_PADS[i] } else { P_PADS[i] };
    vec2(x as f32 + w as f32 / 2., (z + d) as f32 + 0.6)
}
/// Is this player's ship parked on `planet` (and not flying)?
pub fn ship_parked(p: &Player, planet: u8) -> bool {
    p.flight.is_none() && p.ship_at == planet
}
/// Can player `a` see player `b` walking around? Same scene, `b` not flying, and never inside each other's ships.
pub fn sees(a: &Player, b: &Player) -> bool {
    a.scene == b.scene && b.flight.is_none() && !matches!(b.scene, Scene::Cockpit(_))
}
/// Can `a` see `b`'s ship in space? Both flying in the same system and neither in the light-speed tunnel.
pub fn sees_in_space(a: &Player, b: &Player) -> bool {
    match (a.flight, b.flight) {
        (Some(x), Some(y)) => x.sys == y.sys && x.jump <= 0. && y.jump <= 0.,
        _ => false,
    }
}
/// Ship interior: 6 x 5 tiles. Front console along the back wall, the pilot's seat in front of it, the holding
/// cell on the left, a bunk on the right, the ramp door in the middle of the near wall.
pub const CK: (i32, i32) = (6, 5);
pub const CK_SEAT: (i32, i32) = (3, 1);
pub const CK_EXIT: (f32, f32) = (3.5, 4.55);
pub const CK_STAND: (f32, f32) = (3.5, 2.5);
pub fn ck_block(x: i32, z: i32) -> bool {
    x < 0
        || z < 0
        || x >= CK.0
        || z >= CK.1
        || z == 0
        || (x, z) == CK_SEAT
        || (x == 0 && z <= 2)
        || (x == 5 && (1..=2).contains(&z))
}
/// Runtime flight state (never saved: quitting mid-flight puts you back in Hollowbrook like any Dustreach visit).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Flight {
    /// The system you are in (planet index).
    pub sys: u8,
    pub pos: Vec2,
    pub vel: Vec2,
    /// Heading; 0 = towards the top of the screen.
    pub yaw: f32,
    /// Seconds left in the light-speed tunnel (0 = not jumping) and where it goes.
    pub jump: f32,
    pub dest: u8,
    /// Thrust this frame (for the engine glow).
    pub thrust: f32,
}
pub fn fwd(yaw: f32) -> Vec2 {
    vec2(yaw.sin(), -yaw.cos())
}
fn h32(x: i32, z: i32, k: u8) -> u32 {
    let mut v = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (z as u32).wrapping_mul(0x85EB_CA77)
        ^ (k as u32).wrapping_mul(0xC2B2_AE3D);
    v ^= v >> 15;
    v = v.wrapping_mul(0x2C1B_3C6D);
    v ^ (v >> 12)
}
/// Tile of landing site `k` (1 Glacia, 2 Verdance): ROCK = crystal spire / giant mushroom / border cliff,
/// SEA = frozen pond / glowing pool, PATH = trodden trail, SAND (Glacia snow) or GRASS (Verdance moss) elsewhere.
pub fn ptile(k: u8, x: i32, z: i32) -> u8 {
    if !(0..PW).contains(&x) || !(0..PH).contains(&z) || x == 0 || z == 0 || x == PW - 1 || z == PH - 1 {
        return ROCK;
    }
    if P_PADS.iter().any(|&(sx, sz, sw, sd)| x >= sx && x < sx + sw && z >= sz && z < sz + sd) {
        return ROCK; // a landing pad
    }
    let i = (k as usize).clamp(1, 2) - 1;
    let (cx, cz, rx, rz) = P_POND[i];
    let (dx, dz) = ((x as f32 + 0.5 - cx) / rx, (z as f32 + 0.5 - cz) / rz);
    if dx * dx + dz * dz < 1. {
        return SEA;
    }
    if (15..=16).contains(&x) && (7..=12).contains(&z) || (z == 12 && (4..=26).contains(&x)) {
        return PATH;
    }
    let clear = SAMPLE_SPOTS[i].iter().any(|&(px, pz)| (px - x).abs() <= 1 && (pz - z).abs() <= 1)
        || ((12..=29).contains(&x) && (7..=13).contains(&z));
    if !clear && h32(x, z, k) % 9 == 0 {
        return ROCK;
    }
    if k == 1 {
        SAND
    } else {
        GRASS
    }
}
pub fn p_walk(k: u8, x: i32, z: i32) -> bool {
    matches!(ptile(k, x, z), SAND | GRASS | PATH)
}

impl Game {
    /// Planet index of the scene you are standing in, if it has a landing pad.
    pub fn planet_here(&self) -> Option<u8> {
        match self.me.scene {
            Scene::Frontier => Some(0),
            Scene::Planet(k) => Some(k),
            Scene::Cockpit(k) => Some(k),
            _ => None,
        }
    }
    /// Walk up the ramp into your ship.
    pub fn board(&mut self) {
        if let Some(k) = self.planet_here() {
            if ship_parked(&self.me, k) {
                self.go(Scene::Cockpit(k));
            }
        }
    }
    /// The ramp of your own ship in the scene you are in, if your ship is parked here.
    pub fn my_ramp(&self) -> Option<Vec2> {
        let k = match self.me.scene {
            Scene::Frontier => 0,
            Scene::Planet(k) => k,
            _ => return None,
        };
        ship_parked(&self.me, k).then(|| ramp_of(k, pad_of(self.me.id)))
    }
    /// Walk back down the ramp onto the planet the ship is parked on.
    pub fn disembark(&mut self) {
        if let Scene::Cockpit(k) = self.me.scene {
            self.go(if k == 0 { Scene::Frontier } else { Scene::Planet(k) });
        }
    }
    /// Climb aboard and lift off into the planet's orbit.
    pub fn take_off(&mut self) {
        let Scene::Cockpit(sys) = self.me.scene else {
            return self.note("Take the pilot's seat inside your ship to fly.");
        };
        self.me.riding = false;
        self.me.standoff = 0.;
        self.me.bolts.clear();
        self.me.flight = Some(Flight {
            sys,
            pos: vec2(0., PLANET_R + 3.),
            vel: Vec2::ZERO,
            yaw: std::f32::consts::PI,
            jump: 0.,
            dest: sys,
            thrust: 0.,
        });
        self.snd.push(Snd::Door);
        self.note("Engines hot. W thrust, A/D turn, Shift boost, N nav computer, E to land near the planet.");
    }
    /// Upgrade system `k` of the active player's ship at Dena's shipyard.
    pub fn upgrade_ship(&mut self, k: usize) -> Result<u8, &'static str> {
        let &(name, costs, _) = SHIP_UPS.get(k).ok_or("Unknown ship system.")?;
        let lv = self.w.p.ship_up[k];
        if lv >= SHIP_UP_MAX {
            return Err("Already at maximum level.");
        }
        let cost = costs[lv as usize];
        if self.w.p.money < cost {
            return Err("Not enough coins.");
        }
        self.w.p.money -= cost;
        self.w.p.ship_up[k] = lv + 1;
        self.snd.push(Snd::Coin);
        self.note(format!("Upgraded {name} to Lv {}!", lv + 1));
        Ok(lv + 1)
    }
    /// Pick a destination in the nav computer: jump to light speed.
    pub fn plot_jump(&mut self, dest: u8) -> Result<(), &'static str> {
        let hyper = self.w.p.ship_up[1];
        let f = self.me.flight.as_mut().ok_or("You need to be flying.")?;
        if f.jump > 0. {
            return Err("Already in the light-speed tunnel.");
        }
        if dest == f.sys {
            return Err("You're already in this system.");
        }
        f.dest = dest;
        f.jump = jump_time(hyper);
        f.vel = Vec2::ZERO;
        self.snd.push(Snd::Swim);
        self.note(format!("Jumping to light speed... next stop, {}.", PLANETS[dest as usize].0));
        Ok(())
    }
    /// Touch down on the planet of the current system, at your landing pad.
    pub fn land(&mut self) -> Result<(), &'static str> {
        let f = self.me.flight.ok_or("You're not flying.")?;
        if f.jump > 0. {
            return Err("Wait until you drop out of light speed.");
        }
        if f.pos.length() > land_range(self.w.p.ship_up[1]) {
            return Err("Fly closer to the planet to land.");
        }
        self.me.flight = None;
        let scene = Scene::Cockpit(f.sys);
        self.me.ship_at = f.sys; // your ship is now parked on this planet's pad
        self.me.scene = scene; // you are still in the pilot's seat; walk down the ramp to step outside
        let at = vec2(CK_STAND.0, CK_STAND.1);
        self.me.ipos = at;
        self.me.cam = at;
        self.me.face = vec2(0., 1.);
        self.snd.push(Snd::Door);
        self.note(format!("Landed on {}. Walk down the ramp to step outside.", PLANETS[f.sys as usize].0));
        Ok(())
    }
    /// Flight controls (called by update_player while flying and no menu is open).
    pub(super) fn flight_input(&mut self, inp: &Inp, dt: f32) {
        use KeyCode::*;
        let Some(mut f) = self.me.flight else { return };
        if f.jump > 0. {
            f.jump -= dt;
            if f.jump <= 0. {
                f.jump = 0.;
                f.sys = f.dest;
                let drop = (11. - self.w.p.ship_up[1].min(SHIP_UP_MAX) as f32 * 1.5).max(6.5);
                f.pos = vec2(0., PLANET_R + drop);
                f.vel = Vec2::ZERO;
                f.yaw = 0.;
                self.me.flight = Some(f);
                self.note(format!(
                    "Arrived at {}, {}. Fly in and press E to land.",
                    PLANETS[f.sys as usize].0, PLANETS[f.sys as usize].1
                ));
                return;
            }
            self.me.flight = Some(f);
            return;
        }
        let thr = self.w.p.ship_up[0].min(SHIP_UP_MAX) as f32;
        let hull = self.w.p.ship_up[2].min(SHIP_UP_MAX) as f32;
        let turn = (inp.d(D) || inp.d(Right)) as i32 as f32 - (inp.d(A) || inp.d(Left)) as i32 as f32;
        f.yaw += turn * (2.6 + thr * 0.45) * dt;
        let boost = inp.d(LeftShift) || inp.d(RightShift);
        f.thrust = if inp.d(W) || inp.d(Up) {
            if boost {
                1.8
            } else {
                1.
            }
        } else {
            0.
        };
        f.vel += fwd(f.yaw) * f.thrust * (11. + thr * 3.5) * dt;
        if inp.d(S) || inp.d(Down) {
            f.vel *= 1. - ((3. + hull * 1.2) * dt).min(1.);
        }
        f.vel *= 1. - (0.7 * dt).min(1.);
        let top = if boost { 15. + thr * 4. } else { 9. + thr * 2.5 };
        if f.vel.length() > top {
            f.vel = f.vel.normalize() * top;
        }
        f.pos += f.vel * dt;
        // keep out of the planet itself and inside the system
        let d = f.pos.length();
        if d < PLANET_R + 1.2 {
            f.pos = f.pos.normalize_or_zero() * (PLANET_R + 1.2);
            f.vel *= 0.3;
        } else if d > 38. {
            f.pos = f.pos.normalize() * 38.;
            f.vel *= 0.5;
        }
        self.me.flight = Some(f);
        if inp.p(N) || inp.p(Space) {
            self.me.ui = Ui::Nav(0);
        } else if inp.p(E) {
            if let Err(e) = self.land() {
                self.note(e)
            }
        }
    }
    pub fn sample_taken(&self, k: u8, i: usize) -> bool {
        self.w.p.samples.iter().any(|&(d, p, s)| d == self.w.day && p == k && s as usize == i)
    }
    /// Gather today's resource at spot `i` of planet `k`.
    pub fn take_sample(&mut self, k: u8, i: usize) {
        if self.sample_taken(k, i) {
            return;
        }
        let d = SAMPLE_ITEM[(k as usize).clamp(1, 2) - 1];
        let want = sample_yield(self.w.p.ship_up[3], i);
        let mut got = 0u16;
        for _ in 0..want {
            if self.add(d) {
                got += 1;
            } else {
                break;
            }
        }
        if got == 0 {
            return self.note("Your pockets are full.");
        }
        self.w.p.samples.push((self.w.day, k, i as u8));
        self.w.p.samples.retain(|&(day, _, _)| day + 2 >= self.w.day);
        self.snd.push(Snd::Rustle);
        if got == 1 {
            self.note(format!("Gathered a {}. It sells well back home.", def(d).name));
        } else {
            self.note(format!("Scanner haul: {got}x {}!", def(d).name));
        }
    }
}
