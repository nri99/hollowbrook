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
/// Planet radius in flight space (the planet sits at the origin).
pub const PLANET_R: f32 = 5.;
/// Landing-site maps (Glacia and Verdance share a layout generator).
pub const PW: i32 = 30;
pub const PH: i32 = 22;
/// Your ship parks at the top middle (tiles x 13..19, z 3..7); its ramp faces south.
pub const P_SHIP: (i32, i32, i32, i32) = (13, 3, 6, 4);
pub const P_RAMP: (f32, f32) = (16.0, 8.1);
pub const P_SPAWN: (f32, f32) = (16.0, 9.2);
/// Resource spots per planet (gathered once a day per player) and the item each gives.
pub const SAMPLE_SPOTS: [[(i32, i32); 6]; 2] = [
    [(5, 10), (24, 9), (9, 18), (21, 18), (27, 14), (3, 16)],
    [(5, 10), (25, 9), (9, 18), (22, 18), (27, 16), (4, 14)],
];
pub const SAMPLE_ITEM: [u16; 2] = [175, 176]; // Frost Crystal, Glowcap
/// Each landing site has a pond: (centre x, centre z, radius x, radius z).
pub const P_POND: [(f32, f32, f32, f32); 2] = [(9.0, 14.0, 3.2, 2.2), (21.0, 13.0, 3.0, 2.0)];

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
    let (sx, sz, sw, sd) = P_SHIP;
    if x >= sx && x < sx + sw && z >= sz && z < sz + sd {
        return ROCK; // your ship
    }
    let i = (k as usize).clamp(1, 2) - 1;
    let (cx, cz, rx, rz) = P_POND[i];
    let (dx, dz) = ((x as f32 + 0.5 - cx) / rx, (z as f32 + 0.5 - cz) / rz);
    if dx * dx + dz * dz < 1. {
        return SEA;
    }
    if (15..=16).contains(&x) && (8..=12).contains(&z) || (z == 12 && (4..=26).contains(&x)) {
        return PATH;
    }
    let clear = SAMPLE_SPOTS[i].iter().any(|&(px, pz)| (px - x).abs() <= 1 && (pz - z).abs() <= 1)
        || ((12..=20).contains(&x) && (7..=13).contains(&z));
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
            self.go(Scene::Cockpit(k));
        }
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
    /// Pick a destination in the nav computer: jump to light speed.
    pub fn plot_jump(&mut self, dest: u8) -> Result<(), &'static str> {
        let f = self.me.flight.as_mut().ok_or("You need to be flying.")?;
        if f.jump > 0. {
            return Err("Already in the light-speed tunnel.");
        }
        if dest == f.sys {
            return Err("You're already in this system.");
        }
        f.dest = dest;
        f.jump = JUMP_TIME;
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
        if f.pos.length() > PLANET_R + 4. {
            return Err("Fly closer to the planet to land.");
        }
        self.me.flight = None;
        let scene = Scene::Cockpit(f.sys);
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
                f.pos = vec2(0., PLANET_R + 11.);
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
        let turn = (inp.d(D) || inp.d(Right)) as i32 as f32 - (inp.d(A) || inp.d(Left)) as i32 as f32;
        f.yaw += turn * 2.6 * dt;
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
        f.vel += fwd(f.yaw) * f.thrust * 11. * dt;
        if inp.d(S) || inp.d(Down) {
            f.vel *= 1. - (3. * dt).min(1.);
        }
        f.vel *= 1. - (0.7 * dt).min(1.);
        let top = if boost { 15. } else { 9. };
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
        if !self.add(d) {
            return self.note("Your pockets are full.");
        }
        self.w.p.samples.push((self.w.day, k, i as u8));
        self.w.p.samples.retain(|&(day, _, _)| day + 2 >= self.w.day);
        self.snd.push(Snd::Rustle);
        self.note(format!("Gathered a {}. It sells well back home.", def(d).name));
    }
}
