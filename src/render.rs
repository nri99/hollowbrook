//! 3D presentation: a CPU mesh builder (per-vertex lighting, fog and the rolling-world bend), procedural
//! low-poly models, outdoor/indoor scenes, and the 2D HUD drawn on top.
use crate::data::*;
use crate::world::*;
use chrono::{Datelike, NaiveDateTime, Timelike};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};
mod ambience;
mod characters;
mod frontier;
mod hud;
mod interior;
mod models;
mod scene;
pub use {characters::*, frontier::*, hud::*, interior::*, models::*, scene::*};

pub fn rgb(c: (f32, f32, f32)) -> Color {
    Color::new(c.0, c.1, c.2, 1.)
}
fn sh(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}
fn dark(now: NaiveDateTime) -> f32 {
    light(now.hour() as f32 + now.minute() as f32 / 60., false).night
}
/// Smooth value noise on grid corners (0..1): a broad layer for meadow patches plus a fine layer.
fn gnoise(x: i32, z: i32) -> f32 {
    let h = |x: i32, z: i32| (hash(x, z) % 1000) as f32 / 1000.;
    let (bx, bz) = (x.div_euclid(4), z.div_euclid(4));
    let (fx, fz) = (x.rem_euclid(4) as f32 / 4., z.rem_euclid(4) as f32 / 4.);
    let (sx, sz) = (fx * fx * (3. - 2. * fx), fz * fz * (3. - 2. * fz));
    let a = h(bx + 500, bz) + (h(bx + 501, bz) - h(bx + 500, bz)) * sx;
    let b = h(bx + 500, bz + 1) + (h(bx + 501, bz + 1) - h(bx + 500, bz + 1)) * sx;
    (a + (b - a) * sz) * 0.75 + h(x, z) * 0.25
}
/// Ground tint at a corner: darker/greener in hollows, sunnier on rises.
fn gtint(c: Color, n: f32) -> Color {
    let k = 0.89 + n * 0.21;
    Color::new(c.r * k * (0.97 + n * 0.06), c.g * k, c.b * k * (1.03 - n * 0.06), c.a)
}
fn hash(x: i32, z: i32) -> u32 {
    ((x as u32).wrapping_mul(73856093) ^ (z as u32).wrapping_mul(19349663)).wrapping_mul(2654435761) >> 16
}
const DARK: Color = Color::new(0.09, 0.07, 0.07, 1.);
const WOOD: Color = Color::new(0.58, 0.4, 0.26, 1.);

pub struct R {
    v: Vec<Vertex>,
    i: Vec<u16>,
    pub xf: Mat4,
    pub curve: f32,
    pub c0: Vec2,
    pub sun: Vec3,
    pub amb: Vec3,
    pub dif: Vec3,
    pub sky: Color,
    pub fog: (f32, f32),
    pub unlit: bool,
    pub alpha: f32,
}
impl R {
    pub fn new(curve: f32, c0: Vec2, lt: &Light, fog: (f32, f32)) -> R {
        R {
            v: Vec::with_capacity(60000),
            i: Vec::with_capacity(90000),
            xf: Mat4::IDENTITY,
            curve,
            c0,
            sun: lt.sun,
            amb: lt.amb,
            dif: lt.dif,
            sky: lt.sky,
            fog,
            unlit: false,
            alpha: 1.,
        }
    }
    fn vtx(&mut self, p: Vec3, n: Vec3, c: Color) {
        let w = self.xf.transform_point3(p);
        let d2 = (w.x - self.c0.x).powi(2) + (w.z - self.c0.y).powi(2);
        let mut col = c;
        if !self.unlit {
            let n = self.xf.transform_vector3(n).normalize_or_zero();
            let l = self.amb + self.dif * n.dot(self.sun).max(0.) + Vec3::splat(0.08 * n.y.max(0.));
            col = Color::new(c.r * l.x, c.g * l.y, c.b * l.z, c.a)
        }
        let f = ((d2.sqrt() - self.fog.0) / (self.fog.1 - self.fog.0)).clamp(0., 1.);
        col = Color::new(
            col.r + (self.sky.r - col.r) * f,
            col.g + (self.sky.g - col.g) * f,
            col.b + (self.sky.b - col.b) * f,
            col.a * self.alpha,
        );
        // rolling world: same camera-relative drop for every vertex; gameplay coordinates stay flat
        self.v.push(Vertex::new(w.x, w.y - self.curve * d2, w.z, 0., 0., col));
    }
    // macroquad clamps a single draw call (~10k vertices / 5k indices), so flush in small batches
    fn base(&mut self, n: usize) -> u16 {
        if self.v.len() + n > 2400 || self.i.len() > 4400 {
            self.flush()
        }
        self.v.len() as u16
    }
    pub fn flush(&mut self) {
        if !self.v.is_empty() {
            draw_mesh(&Mesh {
                vertices: std::mem::take(&mut self.v),
                indices: std::mem::take(&mut self.i),
                texture: None,
            })
        }
    }
    pub fn with(&mut self, m: Mat4, f: impl FnOnce(&mut R)) {
        let o = self.xf;
        self.xf = o * m;
        f(self);
        self.xf = o
    }
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, n: Vec3, col: Color) {
        let o = self.base(4);
        for p in [a, b, c, d] {
            self.vtx(p, n, col)
        }
        self.i.extend_from_slice(&[o, o + 1, o + 2, o, o + 2, o + 3])
    }
    /// Quad with a colour per corner (smooth ground shading).
    pub fn quad4(&mut self, p: [Vec3; 4], n: Vec3, c: [Color; 4]) {
        let o = self.base(4);
        for k in 0..4 {
            self.vtx(p[k], n, c[k])
        }
        self.i.extend_from_slice(&[o, o + 1, o + 2, o, o + 2, o + 3])
    }
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, n: Vec3, col: Color) {
        let o = self.base(3);
        for p in [a, b, c] {
            self.vtx(p, n, col)
        }
        self.i.extend_from_slice(&[o, o + 1, o + 2])
    }
    pub fn cube(&mut self, c: Vec3, h: Vec3, col: Color) {
        let p = |x: f32, y: f32, z: f32| c + vec3(x * h.x, y * h.y, z * h.z);
        self.quad(p(-1., 1., -1.), p(1., 1., -1.), p(1., 1., 1.), p(-1., 1., 1.), Vec3::Y, col);
        self.quad(p(-1., -1., 1.), p(1., -1., 1.), p(1., 1., 1.), p(-1., 1., 1.), Vec3::Z, col);
        self.quad(p(-1., -1., -1.), p(-1., 1., -1.), p(1., 1., -1.), p(1., -1., -1.), -Vec3::Z, col);
        self.quad(p(1., -1., -1.), p(1., 1., -1.), p(1., 1., 1.), p(1., -1., 1.), Vec3::X, col);
        self.quad(p(-1., -1., -1.), p(-1., -1., 1.), p(-1., 1., 1.), p(-1., 1., -1.), -Vec3::X, col);
    }
    pub fn sphere(&mut self, c: Vec3, r: Vec3, col: Color) {
        const LA: usize = 6;
        const LO: usize = 10;
        let o = self.base((LA + 1) * (LO + 1));
        for a in 0..=LA {
            let th = a as f32 / LA as f32 * PI;
            for b in 0..=LO {
                let ph = b as f32 / LO as f32 * 2. * PI;
                let n = vec3(th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin());
                self.vtx(c + n * r, n, col)
            }
        }
        for a in 0..LA {
            for b in 0..LO {
                let i0 = o + (a * (LO + 1) + b) as u16;
                let i1 = i0 + (LO + 1) as u16;
                self.i.extend_from_slice(&[i0, i1, i0 + 1, i0 + 1, i1, i1 + 1])
            }
        }
    }
    pub fn cyl(&mut self, c: Vec3, r: f32, h: f32, col: Color) {
        for k in 0..10 {
            let (a0, a1) = (k as f32 / 10. * 2. * PI, (k + 1) as f32 / 10. * 2. * PI);
            let (p0, p1) = (vec3(a0.cos() * r, 0., a0.sin() * r), vec3(a1.cos() * r, 0., a1.sin() * r));
            let m = (a0 + a1) / 2.;
            self.quad(c + p0, c + p1, c + p1 + Vec3::Y * h, c + p0 + Vec3::Y * h, vec3(m.cos(), 0., m.sin()), col);
            self.tri(c + Vec3::Y * h, c + p0 + Vec3::Y * h, c + p1 + Vec3::Y * h, Vec3::Y, col)
        }
    }
    pub fn cone(&mut self, c: Vec3, r: f32, h: f32, col: Color) {
        for k in 0..10 {
            let (a0, a1) = (k as f32 / 10. * 2. * PI, (k + 1) as f32 / 10. * 2. * PI);
            let m = (a0 + a1) / 2.;
            self.tri(
                c + vec3(a0.cos() * r, 0., a0.sin() * r),
                c + vec3(a1.cos() * r, 0., a1.sin() * r),
                c + Vec3::Y * h,
                vec3(m.cos(), r / h, m.sin()).normalize(),
                col,
            )
        }
    }
    /// Gabled roof: ridge along x at height h above c.
    pub fn roof(&mut self, c: Vec3, hw: f32, hd: f32, h: f32, col: Color) {
        let p = |x: f32, y: f32, z: f32| c + vec3(x, y, z);
        self.quad(p(-hw, 0., hd), p(hw, 0., hd), p(hw, h, 0.), p(-hw, h, 0.), vec3(0., hd, h).normalize(), col);
        self.quad(
            p(-hw, 0., -hd),
            p(-hw, h, 0.),
            p(hw, h, 0.),
            p(hw, 0., -hd),
            vec3(0., hd, -h).normalize(),
            sh(col, 0.9),
        );
        self.tri(p(hw, 0., hd), p(hw, 0., -hd), p(hw, h, 0.), Vec3::X, sh(col, 0.85));
        self.tri(p(-hw, 0., hd), p(-hw, 0., -hd), p(-hw, h, 0.), -Vec3::X, sh(col, 0.85));
    }
    pub fn seg(&mut self, a: Vec3, b: Vec3, r: f32, col: Color) {
        let d = b - a;
        let l = d.length();
        if l < 1e-4 {
            return;
        }
        let q = Quat::from_rotation_arc(Vec3::Y, d / l);
        self.with(Mat4::from_rotation_translation(q, (a + b) * 0.5), |s| s.cube(Vec3::ZERO, vec3(r, l / 2., r), col))
    }
    /// Unlit flat ellipse (contact shadows, decals).
    pub fn disc(&mut self, c: Vec3, r: Vec2, col: Color) {
        let u = self.unlit;
        self.unlit = true;
        for k in 0..12 {
            let (a0, a1) = (k as f32 / 12. * 2. * PI, (k + 1) as f32 / 12. * 2. * PI);
            self.tri(
                c,
                c + vec3(a0.cos() * r.x, 0., a0.sin() * r.y),
                c + vec3(a1.cos() * r.x, 0., a1.sin() * r.y),
                Vec3::Y,
                col,
            )
        }
        self.unlit = u;
    }
}

pub struct Light {
    pub sky: Color,
    pub sun: Vec3,
    pub amb: Vec3,
    pub dif: Vec3,
    pub night: f32,
}
fn ss(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
/// Real-clock lighting: day, dawn/dusk warmth, night.
pub fn light(hf: f32, rain: bool) -> Light {
    let day = ss(5., 7., hf) * (1. - ss(18., 20.5, hf));
    let dusk = (1. - ((hf - 19.).abs() / 1.5).min(1.)).max(1. - ((hf - 6.).abs() / 1.2).min(1.));
    let ang = ((hf - 6.) / 12. * PI).clamp(0.2, PI - 0.2);
    let sun = vec3(ang.cos() * 0.7, ang.sin().max(0.3), 0.55).normalize();
    let tint = vec3(0.42, 0.5, 0.82).lerp(Vec3::ONE.lerp(vec3(1., 0.72, 0.52), dusk * 0.8), day.max(dusk * 0.6));
    let k = if rain { 0.8 } else { 1. };
    let s = vec3(0.07, 0.09, 0.2)
        .lerp(vec3(0.56, 0.78, 0.95).lerp(vec3(0.98, 0.62, 0.45), dusk * 0.85), day.max(dusk * 0.5))
        * if rain { 0.75 } else { 1. };
    Light {
        sky: Color::new(s.x, s.y, s.z, 1.),
        sun,
        amb: tint * (0.36 + 0.28 * day) * k,
        dif: tint * (0.12 + 0.55 * day) * k,
        night: 1. - day,
    }
}
fn season(mo: u32) -> (Color, Color) {
    match mo {
        3..=5 => (Color::new(0.5, 0.78, 0.4, 1.), Color::new(0.4, 0.7, 0.35, 1.)),
        6..=8 => (Color::new(0.38, 0.7, 0.3, 1.), Color::new(0.2, 0.55, 0.25, 1.)),
        9..=11 => (Color::new(0.62, 0.66, 0.34, 1.), Color::new(0.88, 0.5, 0.2, 1.)),
        _ => (Color::new(0.82, 0.86, 0.88, 1.), Color::new(0.25, 0.45, 0.35, 1.)),
    }
}

// ---- models ----
pub fn draw_tool(r: &mut R, t: u16) {
    let (t, lv) = (t % 1000, t / 1000);
    if lv > 0 {
        r.sphere(vec3(0., 0.02, 0.), Vec3::splat(0.035), if lv == 1 { Color::new(0.8, 0.82, 0.88, 1.) } else { GOLD })
    }
    match t {
        154 => {
            let c = if lv == 2 {
                GOLD
            } else if lv == 1 {
                Color::new(0.75, 0.78, 0.85, 1.)
            } else {
                Color::new(0.35, 0.6, 0.45, 1.)
            };
            r.cyl(vec3(0., -0.12, 0.08), 0.08, 0.13, c);
            r.seg(vec3(0., -0.05, 0.14), vec3(0., 0.02, 0.3), 0.015, c);
            r.seg(vec3(0., 0.02, -0.02), vec3(0., 0.05, 0.12), 0.012, c)
        }
        172 => {
            // Stun Pistol: gunmetal body, copper coil, a cyan emitter
            let metal = Color::new(0.34, 0.36, 0.4, 1.);
            r.cube(vec3(0., -0.05, 0.), vec3(0.022, 0.06, 0.03), Color::new(0.25, 0.18, 0.13, 1.));
            r.cube(vec3(0., 0.02, 0.09), vec3(0.03, 0.035, 0.11), metal);
            r.cube(vec3(0., 0.02, 0.12), vec3(0.037, 0.042, 0.016), Color::new(0.82, 0.52, 0.26, 1.));
            let u = r.unlit;
            r.unlit = true;
            r.sphere(vec3(0., 0.02, 0.21), Vec3::splat(0.022), Color::new(0.55, 0.95, 1., 1.));
            r.unlit = u;
        }
        173 => {
            // Tangle Launcher: olive tube with two cord weights at the muzzle
            r.seg(vec3(0., 0.0, -0.06), vec3(0., 0.06, 0.32), 0.036, Color::new(0.45, 0.43, 0.3, 1.));
            r.cube(vec3(0., -0.05, 0.02), vec3(0.02, 0.05, 0.025), Color::new(0.25, 0.18, 0.13, 1.));
            for x in [-0.035, 0.035] {
                r.sphere(vec3(x, 0.07, 0.34), Vec3::splat(0.024), Color::new(0.2, 0.2, 0.22, 1.));
            }
        }
        174 => {
            // Shock Staff: a long dark staff with leather wraps and a glowing blue tip
            r.seg(vec3(0., -0.42, -0.12), vec3(0., 0.58, 0.26), 0.018, Color::new(0.24, 0.2, 0.18, 1.));
            for k in 0..3 {
                let p = vec3(0., -0.05 + k as f32 * 0.07, 0.02 + k as f32 * 0.027);
                r.sphere(p, vec3(0.026, 0.02, 0.026), Color::new(0.5, 0.33, 0.2, 1.));
            }
            let u = r.unlit;
            r.unlit = true;
            r.sphere(vec3(0., 0.62, 0.28), Vec3::splat(0.045), Color::new(0.5, 0.75, 1., 1.));
            r.unlit = u;
        }
        60 => {
            r.seg(vec3(0., 0.05, 0.), vec3(0., 0.3, 0.45), 0.014, WOOD);
            for k in 0..10 {
                let a = k as f32 / 10. * 2. * PI;
                r.sphere(vec3(a.cos() * 0.12, 0.38 + a.sin() * 0.12, 0.55), Vec3::splat(0.02), WHITE)
            }
        }
        61 => {
            r.seg(vec3(0., 0., 0.), vec3(0., 0.5, 0.7), 0.012, WOOD);
            r.sphere(vec3(0., 0.05, 0.06), Vec3::splat(0.03), DARKGRAY)
        }
        _ => {
            r.seg(vec3(0., 0.1, -0.05), vec3(0., -0.2, 0.35), 0.014, WOOD);
            r.with(Mat4::from_translation(vec3(0., -0.25, 0.42)) * Mat4::from_rotation_x(0.9), |r| {
                r.cube(Vec3::ZERO, vec3(0.08, 0.1, 0.012), GRAY)
            })
        }
    }
}

/// Occlusion test marker (only with HOLLOWBROOK_MARK_PLAYER): a small pure-magenta dot on the camera side of the
/// player's head, drawn right after the player so anything opaque in front hides it. tools/occlusion.sh sweeps
/// every walkable spot and fails where the dot is not visible (the player would be hidden).
/// True in occlusion-test mode: faded (see-through) scenery is skipped so only solid things can hide the marker.
pub fn marking() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("HOLLOWBROOK_MARK_PLAYER").is_ok())
}
pub fn mark_player(r: &mut R, p: Vec3) {
    if !marking() {
        return;
    }
    let (u, a) = (r.unlit, r.alpha);
    r.unlit = true;
    r.alpha = 1.;
    r.sphere(p + vec3(0., 1.0, 0.42), Vec3::splat(0.07), Color::new(1., 0., 1., 1.));
    r.unlit = u;
    r.alpha = a;
}
