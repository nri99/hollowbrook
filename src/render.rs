//! 3D presentation: a CPU mesh builder (per-vertex lighting, fog and the rolling-world bend), procedural
//! low-poly models, outdoor/indoor scenes, and the 2D HUD drawn on top.
use crate::data::*;
use crate::world::*;
use chrono::{Datelike, NaiveDateTime, Timelike};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

pub fn rgb(c: (f32, f32, f32)) -> Color { Color::new(c.0, c.1, c.2, 1.) }
fn sh(c: Color, k: f32) -> Color { Color::new(c.r * k, c.g * k, c.b * k, c.a) }
fn hash(x: i32, z: i32) -> u32 { ((x as u32).wrapping_mul(73856093) ^ (z as u32).wrapping_mul(19349663)).wrapping_mul(2654435761) >> 16 }
const DARK: Color = Color::new(0.09, 0.07, 0.07, 1.);
const WOOD: Color = Color::new(0.58, 0.4, 0.26, 1.);

pub struct R { v: Vec<Vertex>, i: Vec<u16>, pub xf: Mat4, pub curve: f32, pub c0: Vec2, pub sun: Vec3, pub amb: Vec3, pub dif: Vec3, pub sky: Color, pub fog: (f32, f32), pub unlit: bool, pub alpha: f32 }
impl R {
    pub fn new(curve: f32, c0: Vec2, lt: &Light, fog: (f32, f32)) -> R { R { v: Vec::with_capacity(60000), i: Vec::with_capacity(90000), xf: Mat4::IDENTITY, curve, c0, sun: lt.sun, amb: lt.amb, dif: lt.dif, sky: lt.sky, fog, unlit: false, alpha: 1. } }
    fn vtx(&mut self, p: Vec3, n: Vec3, c: Color) {
        let w = self.xf.transform_point3(p); let d2 = (w.x - self.c0.x).powi(2) + (w.z - self.c0.y).powi(2);
        let mut col = c;
        if !self.unlit { let n = self.xf.transform_vector3(n).normalize_or_zero(); let l = self.amb + self.dif * n.dot(self.sun).max(0.) + Vec3::splat(0.08 * n.y.max(0.)); col = Color::new(c.r * l.x, c.g * l.y, c.b * l.z, c.a) }
        let f = ((d2.sqrt() - self.fog.0) / (self.fog.1 - self.fog.0)).clamp(0., 1.);
        col = Color::new(col.r + (self.sky.r - col.r) * f, col.g + (self.sky.g - col.g) * f, col.b + (self.sky.b - col.b) * f, col.a * self.alpha);
        // rolling world: same camera-relative drop for every vertex; gameplay coordinates stay flat
        self.v.push(Vertex::new(w.x, w.y - self.curve * d2, w.z, 0., 0., col));
    }
    // macroquad clamps a single draw call (~10k vertices / 5k indices), so flush in small batches
    fn base(&mut self, n: usize) -> u16 { if self.v.len() + n > 2400 || self.i.len() > 4400 { self.flush() } self.v.len() as u16 }
    pub fn flush(&mut self) { if !self.v.is_empty() { draw_mesh(&Mesh { vertices: std::mem::take(&mut self.v), indices: std::mem::take(&mut self.i), texture: None }) } }
    pub fn with(&mut self, m: Mat4, f: impl FnOnce(&mut R)) { let o = self.xf; self.xf = o * m; f(self); self.xf = o }
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, n: Vec3, col: Color) { let o = self.base(4); for p in [a, b, c, d] { self.vtx(p, n, col) } self.i.extend_from_slice(&[o, o + 1, o + 2, o, o + 2, o + 3]) }
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, n: Vec3, col: Color) { let o = self.base(3); for p in [a, b, c] { self.vtx(p, n, col) } self.i.extend_from_slice(&[o, o + 1, o + 2]) }
    pub fn cube(&mut self, c: Vec3, h: Vec3, col: Color) {
        let p = |x: f32, y: f32, z: f32| c + vec3(x * h.x, y * h.y, z * h.z);
        self.quad(p(-1., 1., -1.), p(1., 1., -1.), p(1., 1., 1.), p(-1., 1., 1.), Vec3::Y, col);
        self.quad(p(-1., -1., 1.), p(1., -1., 1.), p(1., 1., 1.), p(-1., 1., 1.), Vec3::Z, col);
        self.quad(p(-1., -1., -1.), p(-1., 1., -1.), p(1., 1., -1.), p(1., -1., -1.), -Vec3::Z, col);
        self.quad(p(1., -1., -1.), p(1., 1., -1.), p(1., 1., 1.), p(1., -1., 1.), Vec3::X, col);
        self.quad(p(-1., -1., -1.), p(-1., -1., 1.), p(-1., 1., 1.), p(-1., 1., -1.), -Vec3::X, col);
    }
    pub fn sphere(&mut self, c: Vec3, r: Vec3, col: Color) {
        const LA: usize = 6; const LO: usize = 10;
        let o = self.base((LA + 1) * (LO + 1));
        for a in 0..=LA { let th = a as f32 / LA as f32 * PI; for b in 0..=LO { let ph = b as f32 / LO as f32 * 2. * PI; let n = vec3(th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()); self.vtx(c + n * r, n, col) } }
        for a in 0..LA { for b in 0..LO { let i0 = o + (a * (LO + 1) + b) as u16; let i1 = i0 + (LO + 1) as u16; self.i.extend_from_slice(&[i0, i1, i0 + 1, i0 + 1, i1, i1 + 1]) } }
    }
    pub fn cyl(&mut self, c: Vec3, r: f32, h: f32, col: Color) {
        for k in 0..10 { let (a0, a1) = (k as f32 / 10. * 2. * PI, (k + 1) as f32 / 10. * 2. * PI); let (p0, p1) = (vec3(a0.cos() * r, 0., a0.sin() * r), vec3(a1.cos() * r, 0., a1.sin() * r)); let m = (a0 + a1) / 2.;
            self.quad(c + p0, c + p1, c + p1 + Vec3::Y * h, c + p0 + Vec3::Y * h, vec3(m.cos(), 0., m.sin()), col); self.tri(c + Vec3::Y * h, c + p0 + Vec3::Y * h, c + p1 + Vec3::Y * h, Vec3::Y, col) }
    }
    pub fn cone(&mut self, c: Vec3, r: f32, h: f32, col: Color) {
        for k in 0..10 { let (a0, a1) = (k as f32 / 10. * 2. * PI, (k + 1) as f32 / 10. * 2. * PI); let m = (a0 + a1) / 2.;
            self.tri(c + vec3(a0.cos() * r, 0., a0.sin() * r), c + vec3(a1.cos() * r, 0., a1.sin() * r), c + Vec3::Y * h, vec3(m.cos(), r / h, m.sin()).normalize(), col) }
    }
    /// Gabled roof: ridge along x at height h above c.
    pub fn roof(&mut self, c: Vec3, hw: f32, hd: f32, h: f32, col: Color) {
        let p = |x: f32, y: f32, z: f32| c + vec3(x, y, z);
        self.quad(p(-hw, 0., hd), p(hw, 0., hd), p(hw, h, 0.), p(-hw, h, 0.), vec3(0., hd, h).normalize(), col);
        self.quad(p(-hw, 0., -hd), p(-hw, h, 0.), p(hw, h, 0.), p(hw, 0., -hd), vec3(0., hd, -h).normalize(), sh(col, 0.9));
        self.tri(p(hw, 0., hd), p(hw, 0., -hd), p(hw, h, 0.), Vec3::X, sh(col, 0.85)); self.tri(p(-hw, 0., hd), p(-hw, 0., -hd), p(-hw, h, 0.), -Vec3::X, sh(col, 0.85));
    }
    pub fn seg(&mut self, a: Vec3, b: Vec3, r: f32, col: Color) { let d = b - a; let l = d.length(); if l < 1e-4 { return } let q = Quat::from_rotation_arc(Vec3::Y, d / l); self.with(Mat4::from_rotation_translation(q, (a + b) * 0.5), |s| s.cube(Vec3::ZERO, vec3(r, l / 2., r), col)) }
    /// Unlit flat ellipse (contact shadows, decals).
    pub fn disc(&mut self, c: Vec3, r: Vec2, col: Color) {
        let u = self.unlit; self.unlit = true;
        for k in 0..12 { let (a0, a1) = (k as f32 / 12. * 2. * PI, (k + 1) as f32 / 12. * 2. * PI); self.tri(c, c + vec3(a0.cos() * r.x, 0., a0.sin() * r.y), c + vec3(a1.cos() * r.x, 0., a1.sin() * r.y), Vec3::Y, col) }
        self.unlit = u;
    }
}

pub struct Light { pub sky: Color, pub sun: Vec3, pub amb: Vec3, pub dif: Vec3, pub night: f32 }
fn ss(a: f32, b: f32, x: f32) -> f32 { let t = ((x - a) / (b - a)).clamp(0., 1.); t * t * (3. - 2. * t) }
/// Real-clock lighting: day, dawn/dusk warmth, night.
pub fn light(hf: f32, rain: bool) -> Light {
    let day = ss(5., 7., hf) * (1. - ss(18., 20.5, hf));
    let dusk = (1. - ((hf - 19.).abs() / 1.5).min(1.)).max(1. - ((hf - 6.).abs() / 1.2).min(1.));
    let ang = ((hf - 6.) / 12. * PI).clamp(0.2, PI - 0.2); let sun = vec3(ang.cos() * 0.7, ang.sin().max(0.3), 0.55).normalize();
    let tint = vec3(0.42, 0.5, 0.82).lerp(Vec3::ONE.lerp(vec3(1., 0.72, 0.52), dusk * 0.8), day.max(dusk * 0.6));
    let k = if rain { 0.8 } else { 1. };
    let s = vec3(0.07, 0.09, 0.2).lerp(vec3(0.56, 0.78, 0.95).lerp(vec3(0.98, 0.62, 0.45), dusk * 0.85), day.max(dusk * 0.5)) * if rain { 0.75 } else { 1. };
    Light { sky: Color::new(s.x, s.y, s.z, 1.), sun, amb: tint * (0.36 + 0.28 * day) * k, dif: tint * (0.12 + 0.55 * day) * k, night: 1. - day }
}
fn season(mo: u32) -> (Color, Color) { match mo { 3..=5 => (Color::new(0.5, 0.78, 0.4, 1.), Color::new(0.4, 0.7, 0.35, 1.)), 6..=8 => (Color::new(0.38, 0.7, 0.3, 1.), Color::new(0.2, 0.55, 0.25, 1.)), 9..=11 => (Color::new(0.62, 0.66, 0.34, 1.), Color::new(0.88, 0.5, 0.2, 1.)), _ => (Color::new(0.82, 0.86, 0.88, 1.), Color::new(0.25, 0.45, 0.35, 1.)) } }

// ---- models ----
pub fn draw_tool(r: &mut R, t: u16) {
    match t {
        60 => { r.seg(vec3(0., 0.05, 0.), vec3(0., 0.3, 0.45), 0.014, WOOD); for k in 0..10 { let a = k as f32 / 10. * 2. * PI; r.sphere(vec3(a.cos() * 0.12, 0.38 + a.sin() * 0.12, 0.55), Vec3::splat(0.02), WHITE) } }
        61 => { r.seg(vec3(0., 0., 0.), vec3(0., 0.5, 0.7), 0.012, WOOD); r.sphere(vec3(0., 0.05, 0.06), Vec3::splat(0.03), DARKGRAY) }
        _ => { r.seg(vec3(0., 0.1, -0.05), vec3(0., -0.2, 0.35), 0.014, WOOD); r.with(Mat4::from_translation(vec3(0., -0.25, 0.42)) * Mat4::from_rotation_x(0.9), |r| r.cube(Vec3::ZERO, vec3(0.08, 0.1, 0.012), GRAY)) }
    }
}
/// Player: large head, hair, face, outfit; pose 0 stand/walk/run, 1 sit, 2 lie. Tool held in the right hand.
pub fn human(r: &mut R, id: usize, pos: Vec3, yaw: f32, anim: f32, spd: f32, pose: u8, tool: Option<u16>, t: f32, swing: f32) {
    let (skin, hair, shirt, pants, shoe) = if id == 0 { (Color::new(0.98, 0.82, 0.68, 1.), Color::new(0.36, 0.2, 0.12, 1.), Color::new(0.95, 0.47, 0.36, 1.), Color::new(0.27, 0.36, 0.62, 1.), Color::new(0.3, 0.2, 0.15, 1.)) }
        else { (Color::new(0.85, 0.66, 0.5, 1.), Color::new(0.12, 0.1, 0.1, 1.), Color::new(0.3, 0.66, 0.48, 1.), Color::new(0.35, 0.3, 0.26, 1.), Color::new(0.2, 0.2, 0.25, 1.)) };
    let k = (spd / 3.).min(1.8); let sw = anim.sin() * 0.7 * k; let bob = (anim * 2.).sin().abs() * 0.05 * k + (t * 2.2).sin() * 0.01; let lean = if spd > 4. { 0.2 } else { 0. };
    let mut base = Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), pos);
    if pose == 2 { base = base * Mat4::from_translation(vec3(0., 0.45, 0.55)) * Mat4::from_rotation_x(-FRAC_PI_2) }
    let hip = if pose == 1 { 0.14 } else { 0. };
    r.with(base, |r| {
        for s in [-1f32, 1.] { let a = if pose == 1 { -FRAC_PI_2 } else { sw * s };
            r.with(Mat4::from_translation(vec3(s * 0.085, 0.28 + hip, 0.)) * Mat4::from_rotation_x(a), |r| { r.cube(vec3(0., -0.12, 0.), vec3(0.06, 0.12, 0.065), pants); r.cube(vec3(0., -0.255, 0.03), vec3(0.065, 0.035, 0.095), shoe) }) }
        r.with(Mat4::from_translation(vec3(0., 0.28 + hip + bob, 0.)) * Mat4::from_rotation_x(lean), |r| {
            r.cube(vec3(0., 0.15, 0.), vec3(0.16, 0.16, 0.11), shirt); r.cube(vec3(0., 0.02, 0.), vec3(0.165, 0.03, 0.115), pants);
            for s in [-1f32, 1.] { let a = if pose == 1 { -0.6 } else if s > 0. && swing > 0. { -swing * 3. } else { -sw * s };
                r.with(Mat4::from_translation(vec3(s * 0.2, 0.28, 0.)) * Mat4::from_rotation_x(a), |r| {
                    r.cube(vec3(0., -0.1, 0.), vec3(0.05, 0.1, 0.055), shirt); r.sphere(vec3(0., -0.22, 0.), Vec3::splat(0.055), skin);
                    if s > 0. { if let Some(tl) = tool { r.with(Mat4::from_translation(vec3(0., -0.22, 0.03)), |r| draw_tool(r, tl)) } } }) }
            r.with(Mat4::from_translation(vec3(0., 0.6, 0.)) * Mat4::from_rotation_y((t * 0.6).sin() * 0.12 * (1. - k.min(1.))), |r| {
                r.sphere(Vec3::ZERO, vec3(0.28, 0.27, 0.27), skin); r.sphere(vec3(0., 0.08, -0.03), vec3(0.295, 0.25, 0.28), hair); r.cube(vec3(0., 0.17, 0.19), vec3(0.2, 0.055, 0.07), hair);
                for s in [-1f32, 1.] { r.sphere(vec3(s * 0.095, -0.01, 0.245), vec3(0.038, 0.055, 0.02), DARK); r.sphere(vec3(s * 0.09, 0.015, 0.262), Vec3::splat(0.012), WHITE); r.sphere(vec3(s * 0.165, -0.085, 0.215), vec3(0.045, 0.03, 0.02), Color::new(1., 0.62, 0.62, 1.)) }
                r.cube(vec3(0., -0.12, 0.255), vec3(0.035, 0.01, 0.01), Color::new(0.6, 0.25, 0.25, 1.)); }) }) });
}
/// Residents 0-3 and the shopkeeper (4): distinct heads, ears, muzzles, tails, clothes; talk mouth + happy eyes.
pub fn resident(r: &mut R, i: usize, pos: Vec3, yaw: f32, anim: f32, moving: bool, t: f32, talk: bool, happy: f32) {
    let (fur, cloth) = if i < 4 { (rgb(RES[i].fur), rgb(RES[i].cloth)) } else { (Color::new(0.55, 0.65, 0.38, 1.), Color::new(0.92, 0.86, 0.7, 1.)) };
    let light = Color::new(0.97, 0.94, 0.88, 1.); let sw = if moving { anim.sin() * 0.6 } else { 0. };
    let jump = if happy > 0. { (t * 10.).sin().abs() * 0.18 } else { 0. }; let bob = (t * 2. + i as f32).sin() * 0.012;
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), pos + vec3(0., jump, 0.)), |r| {
        for s in [-1f32, 1.] { r.with(Mat4::from_translation(vec3(s * 0.09, 0.24, 0.)) * Mat4::from_rotation_x(sw * s), |r| r.cube(vec3(0., -0.11, 0.02), vec3(0.065, 0.12, 0.08), fur)) }
        r.with(Mat4::from_translation(vec3(0., 0.24 + bob, 0.)), |r| {
            r.cube(vec3(0., 0.15, 0.), vec3(0.17, 0.16, 0.12), cloth);
            if i == 3 { for y in [0.08, 0.2] { r.cube(vec3(0., y, 0.), vec3(0.175, 0.025, 0.125), WHITE) } }
            if i == 4 { r.sphere(vec3(0., 0.2, -0.13), vec3(0.3, 0.26, 0.24), Color::new(0.45, 0.38, 0.22, 1.)) }
            for s in [-1f32, 1.] { let wave = if talk && s > 0. { (t * 5.).sin() * 0.4 - 0.5 } else { 0. };
                r.with(Mat4::from_translation(vec3(s * 0.21, 0.27, 0.)) * Mat4::from_rotation_x(-sw * s + wave), |r| r.cube(vec3(0., -0.1, 0.), vec3(0.05, 0.1, 0.05), fur)) }
            let wag = (t * 4.).sin();
            match i { 0 => { r.sphere(vec3(wag * 0.05, 0.05, -0.28), vec3(0.12, 0.12, 0.22), fur); r.sphere(vec3(wag * 0.07, 0.07, -0.47), Vec3::splat(0.08), light) }
                1 => r.sphere(vec3(0., 0.02, -0.15), Vec3::splat(0.07), fur),
                3 => { r.sphere(vec3(wag * 0.03, 0., -0.22), vec3(0.08, 0.07, 0.16), fur); r.sphere(vec3(wag * 0.06, -0.05, -0.38), vec3(0.06, 0.05, 0.12), fur) } _ => {} }
            r.with(Mat4::from_translation(vec3(0., 0.58, 0.02)), |r| head(r, i, fur, light, t, talk, happy)) }) });
}
fn head(r: &mut R, i: usize, fur: Color, light: Color, t: f32, talk: bool, happy: f32) {
    let (hr, ey, ez) = match i { 2 => (vec3(0.3, 0.2, 0.26), 0.19, 0.13), 4 => (vec3(0.2, 0.19, 0.21), 0.02, 0.17), _ => (vec3(0.26, 0.24, 0.25), 0.03, 0.2) };
    r.sphere(Vec3::ZERO, hr, fur);
    match i {
        0 => { r.sphere(vec3(0., -0.07, 0.18), vec3(0.11, 0.08, 0.14), light); r.sphere(vec3(0., -0.04, 0.32), Vec3::splat(0.035), DARK); for s in [-1f32, 1.] { r.cone(vec3(s * 0.14, 0.16, -0.02), 0.08, 0.2, fur); r.sphere(vec3(s * 0.13, -0.08, 0.16), vec3(0.06, 0.04, 0.05), light) } }
        1 => { r.cube(vec3(0., 0.13, 0.1), vec3(0.05, 0.11, 0.15), light); for s in [-1f32, 1.] { r.sphere(vec3(s * 0.09, 0.02, 0.17), vec3(0.075, 0.09, 0.06), DARK); r.sphere(vec3(s * 0.17, 0.17, -0.02), Vec3::splat(0.06), fur) } r.sphere(vec3(0., -0.08, 0.2), vec3(0.08, 0.06, 0.07), light); r.sphere(vec3(0., -0.06, 0.27), Vec3::splat(0.03), DARK) }
        2 => { for s in [-1f32, 1.] { r.sphere(vec3(s * 0.13, 0.16, 0.08), Vec3::splat(0.095), fur); r.sphere(vec3(s * 0.2, -0.06, 0.15), vec3(0.05, 0.03, 0.03), Color::new(1., 0.6, 0.6, 1.)) } }
        3 => { r.sphere(vec3(0., -0.07, 0.17), vec3(0.12, 0.08, 0.1), light); r.sphere(vec3(0., -0.04, 0.27), Vec3::splat(0.035), DARK); for s in [-1f32, 1.] { r.sphere(vec3(s * 0.18, 0.13, -0.02), Vec3::splat(0.05), fur) } }
        _ => {}
    }
    for s in [-1f32, 1.] { let ex = s * if i == 2 { 0.13 } else { 0.085 };
        if happy > 0. { r.cube(vec3(ex, ey, ez + 0.06), vec3(0.035, 0.009, 0.012), if i == 1 { WHITE } else { DARK }) }
        else { r.sphere(vec3(ex, ey, ez + 0.05), vec3(0.03, 0.042, 0.02), if i == 1 { WHITE } else { DARK }); r.sphere(vec3(ex + 0.01, ey + 0.012, ez + 0.068), Vec3::splat(0.011), if i == 1 { DARK } else { WHITE }) } }
    let open = if talk { (t * 12.).sin().abs() * 0.028 } else { 0. } + 0.008;
    let (my, mz, mw) = match i { 0 => (-0.14, 0.28, 0.035), 2 => (-0.05, 0.255, 0.14), 4 => (-0.08, 0.2, 0.03), 3 => (-0.13, 0.25, 0.035), _ => (-0.13, 0.24, 0.035) };
    r.cube(vec3(0., my, mz), vec3(mw, open, 0.012), Color::new(0.45, 0.15, 0.15, 1.));
}
pub fn furniture(r: &mut R, d: u16, base: Mat4, on: bool) {
    let c = rgb(def(d).col);
    r.with(base, |r| match d {
        50 => { r.cube(vec3(0., 0.4, 0.), vec3(0.3, 0.05, 0.3), c); for (x, z) in [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)] { r.cube(vec3(x, 0.18, z), vec3(0.04, 0.18, 0.04), sh(c, 0.8)) } r.cube(vec3(0., 0.72, -0.27), vec3(0.3, 0.28, 0.04), c) }
        51 => { r.cube(vec3(0., 0.22, 0.), vec3(0.95, 0.18, 0.4), c); for x in [-0.47, 0.47] { r.cube(vec3(x, 0.45, 0.05), vec3(0.44, 0.07, 0.33), sh(c, 1.15)) } r.cube(vec3(0., 0.6, -0.32), vec3(0.95, 0.25, 0.09), c); for x in [-0.92, 0.92] { r.cube(vec3(x, 0.45, 0.), vec3(0.07, 0.15, 0.4), sh(c, 0.85)) } }
        52 => { r.cyl(vec3(0., 0.68, 0.), 0.42, 0.06, c); r.cyl(Vec3::ZERO, 0.06, 0.68, sh(c, 0.8)); r.cyl(Vec3::ZERO, 0.25, 0.04, sh(c, 0.8)) }
        53 => { r.cube(vec3(0., 0.18, 0.), vec3(0.45, 0.14, 0.95), WOOD); r.cube(vec3(0., 0.36, 0.05), vec3(0.42, 0.06, 0.88), WHITE); r.cube(vec3(0., 0.4, 0.3), vec3(0.44, 0.05, 0.62), c); r.cube(vec3(0., 0.45, -0.7), vec3(0.32, 0.06, 0.15), WHITE); r.cube(vec3(0., 0.5, -0.96), vec3(0.45, 0.4, 0.04), WOOD) }
        54 => { r.cyl(Vec3::ZERO, 0.18, 0.05, WOOD); r.cyl(vec3(0., 0.05, 0.), 0.03, 0.8, WOOD); let u = r.unlit; r.unlit = on; r.cone(vec3(0., 0.75, 0.), 0.27, 0.34, if on { Color::new(1., 0.93, 0.62, 1.) } else { c }); r.unlit = u }
        55 => { r.cube(vec3(0., 0.55, 0.), vec3(0.42, 0.55, 0.35), c); r.cube(vec3(0., 0.55, 0.352), vec3(0.01, 0.5, 0.005), sh(c, 0.6)); for x in [-0.08, 0.08] { r.sphere(vec3(x, 0.6, 0.37), Vec3::splat(0.03), GOLD) } }
        56 => { r.cyl(Vec3::ZERO, 0.22, 0.32, Color::new(0.8, 0.45, 0.3, 1.)); for (x, y, z, s) in [(0., 0.6, 0., 0.25), (0.15, 0.48, 0.08, 0.17), (-0.14, 0.5, -0.05, 0.18)] { r.sphere(vec3(x, y, z), Vec3::splat(s), c) } }
        57 => { r.cube(vec3(0., 1.35, 0.03), vec3(0.36, 0.28, 0.03), WOOD); r.cube(vec3(0., 1.35, 0.065), vec3(0.3, 0.22, 0.005), c); r.cube(vec3(0., 1.24, 0.072), vec3(0.3, 0.11, 0.003), Color::new(0.35, 0.65, 0.35, 1.)); r.sphere(vec3(0.15, 1.45, 0.07), Vec3::splat(0.05), YELLOW) }
        _ => r.cube(vec3(0., 0.3, 0.), vec3(0.3, 0.3, 0.3), c),
    });
}
pub fn fbase(f: &Furn) -> Mat4 { let c = if def(f.def).wall { vec2(f.x as f32 + 0.5, 0.) } else { fcenter(f) }; Mat4::from_rotation_translation(Quat::from_rotation_y(f.rot as f32 * FRAC_PI_2), vec3(c.x, 0., c.y)) }
fn building(r: &mut R, bi: usize, night: f32) {
    let b = &BLDS[bi]; let (x, z, w, d) = (b.x as f32, b.z as f32, b.w as f32, b.d as f32); let civic = b.kind == BK::Civic;
    let hg = if civic { 2.0 } else { 1.5 }; let (wall, roof) = (rgb(b.wall), rgb(b.roof)); let c = vec3(x + w / 2., 0., z + d / 2.);
    r.cube(c + vec3(0., 0.08, 0.), vec3(w / 2. + 0.04, 0.08, d / 2. + 0.04), Color::new(0.62, 0.6, 0.57, 1.));
    r.cube(c + vec3(0., hg / 2., 0.), vec3(w / 2. - 0.05, hg / 2., d / 2. - 0.05), wall);
    r.roof(c + vec3(0., hg, 0.), w / 2. + 0.25, d / 2. + 0.3, if civic { 1.0 } else { 0.85 }, roof);
    let dx = door_face(bi).x; let fz = z + d - 0.05;
    r.cube(vec3(dx, 0.52, fz), vec3(0.3, 0.52, 0.06), Color::new(0.45, 0.28, 0.18, 1.)); r.sphere(vec3(dx + 0.18, 0.5, fz + 0.07), Vec3::splat(0.04), GOLD);
    let lit = night > 0.35; let win = if lit { Color::new(1., 0.85, 0.5, 1.) } else { Color::new(0.68, 0.84, 0.95, 1.) };
    for wx in [x + 0.5, x + w - 0.5] { r.cube(vec3(wx, 0.98, fz), vec3(0.24, 0.24, 0.05), WHITE); let u = r.unlit; r.unlit = lit; r.cube(vec3(wx, 0.98, fz + 0.012), vec3(0.19, 0.19, 0.05), win); r.unlit = u }
    match b.kind {
        BK::Shop => { for k in 0..7 { r.cube(vec3(x + 0.4 + k as f32 * (w - 0.8) / 6., hg * 0.82, fz + 0.32), vec3((w - 0.8) / 14., 0.04, 0.3), if k % 2 == 0 { WHITE } else { Color::new(0.2, 0.6, 0.6, 1.) }) } r.cube(vec3(c.x, hg + 0.15, fz + 0.1), vec3(0.8, 0.22, 0.05), WOOD) }
        BK::Civic => { r.cube(vec3(c.x, hg + 1.15, c.z), vec3(0.38, 0.55, 0.38), wall); r.roof(vec3(c.x, hg + 1.7, c.z), 0.48, 0.48, 0.5, roof); r.cylinder_clock(c.x, hg + 1.25, c.z + 0.39);
            r.seg(vec3(c.x, hg + 2.2, c.z), vec3(c.x, hg + 2.9, c.z), 0.02, DARKGRAY); r.cube(vec3(c.x + 0.2, hg + 2.75, c.z), vec3(0.2, 0.12, 0.01), Color::new(0.9, 0.3, 0.3, 1.));
            r.with(Mat4::from_translation(vec3(dx, 0.55, fz + 0.09)) * Mat4::from_rotation_z(0.35), |r| r.cube(Vec3::ZERO, vec3(0.42, 0.05, 0.02), WOOD)) }
        _ => r.cube(c + vec3(w / 2. - 0.6, hg + 0.65, -0.25), vec3(0.13, 0.32, 0.13), Color::new(0.65, 0.35, 0.3, 1.)),
    }
}
impl R { fn cylinder_clock(&mut self, x: f32, y: f32, z: f32) { self.with(Mat4::from_translation(vec3(x, y, z)) * Mat4::from_rotation_x(FRAC_PI_2), |r| { r.cyl(Vec3::ZERO, 0.22, 0.02, WHITE); r.cube(vec3(0., 0.03, -0.08), vec3(0.015, 0.01, 0.09), DARK) }) } }
fn tree(r: &mut R, p: Vec2, fruit: Option<Color>, shake: f32, t: f32, fol: Color) {
    r.cyl(vec3(p.x, 0., p.y), 0.13, 0.85, Color::new(0.5, 0.33, 0.2, 1.));
    let o = vec3(p.x + (t * 45.).sin() * 0.08 * shake, 0., p.y);
    r.sphere(o + vec3(0., 1.2, 0.), vec3(0.65, 0.55, 0.65), fol); r.sphere(o + vec3(0.3, 1.5, 0.1), Vec3::splat(0.38), sh(fol, 1.08)); r.sphere(o + vec3(-0.28, 1.42, -0.12), Vec3::splat(0.4), sh(fol, 0.94));
    if let Some(fc) = fruit { for q in [vec3(0.32, 1.05, 0.48), vec3(-0.38, 1.12, 0.42), vec3(0.05, 1.38, 0.55), vec3(0.52, 1.28, 0.2)] { r.sphere(o + q, Vec3::splat(0.09), fc) } }
}
fn pine(r: &mut R, x: f32, z: f32, s: f32) { r.cyl(vec3(x, 0., z), 0.1 * s, 0.5 * s, Color::new(0.45, 0.3, 0.2, 1.)); for k in 0..3 { r.cone(vec3(x, (0.4 + k as f32 * 0.38) * s, z), (0.62 - k as f32 * 0.15) * s, 0.65 * s, Color::new(0.16, 0.42 + k as f32 * 0.04, 0.26, 1.)) } }
fn indicator(r: &mut R, p: Vec3, t: f32) { let u = r.unlit; r.unlit = true; let y = p.y + (t * 4.).sin() * 0.06; r.with(Mat4::from_translation(vec3(p.x, y, p.z)) * Mat4::from_rotation_x(PI), |r| r.cone(Vec3::ZERO, 0.1, 0.18, Color::new(1., 0.85, 0.2, 1.))); r.unlit = u }

// ---- scenes ----
thread_local! { static VIEW: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((1280., 720.)) }; }
fn vw() -> f32 { VIEW.with(|v| v.get().0) }
fn vh() -> f32 { VIEW.with(|v| v.get().1) }
/// Draw every player's view: full screen for one player, side-by-side split-screen for two.
pub fn draw(g: &mut Game, now: NaiveDateTime, dbg: Option<&str>) {
    let ids = g.ids(); let act = g.me.id; let n = ids.len() as f32; let (sw, sh) = (screen_width(), screen_height());
    if n > 1. { clear_background(BLACK) }
    for (k, &id) in ids.iter().enumerate() { g.activate(id); VIEW.with(|v| v.set((sw / n, sh))); view(g, now, dbg, (n > 1.).then(|| (k as f32 * sw / n, sw / n))) }
    g.activate(act); VIEW.with(|v| v.set((sw, sh)));
    if n > 1. { set_default_camera(); draw_rectangle(sw / 2. - 2., 0., 4., sh, BLACK) }
}
fn hud_cam(vp: Option<(f32, f32)>) { match vp { Some((x, w)) => set_camera(&Camera2D { target: vec2(w / 2., vh() / 2.), zoom: vec2(2. / w, 2. / vh()), viewport: Some((x as i32, 0, w as i32, vh() as i32)), ..Default::default() }), None => set_default_camera() } }
fn view(g: &Game, now: NaiveDateTime, dbg: Option<&str>, vp: Option<(f32, f32)>) {
    let hf = now.hour() as f32 + now.minute() as f32 / 60.; let rain = rainy(g.w.day); let lt = light(hf, rain);
    let (cam, curve, c0) = match g.me.scene {
        Scene::Town => (Camera3D { position: vec3(g.me.cam.x, 7.4, g.me.cam.y + 7.6), target: vec3(g.me.cam.x, 0.5, g.me.cam.y - 0.8), up: Vec3::Y, fovy: 40f32.to_radians(), ..Default::default() }, g.curve, g.me.cam),
        Scene::Room(b) => { let (w, d) = room_dims(b); let s = w.max(d) as f32 / 4.; (Camera3D { position: vec3(w as f32 / 2., 5.4 * s, d as f32 + 3.4 * s), target: vec3(w as f32 / 2., 0.3, d as f32 / 2. + 0.1), up: Vec3::Y, fovy: 45f32.to_radians(), ..Default::default() }, 0., Vec2::ZERO) }
    };
    let bg = if g.me.scene == Scene::Town { lt.sky } else { Color::new(0.08, 0.06, 0.06, 1.) };
    let cam = Camera3D { viewport: vp.map(|(x, w)| (x as i32, 0, w as i32, vh() as i32)), aspect: Some(vw() / vh()), ..cam };
    if vp.is_some() { hud_cam(vp); draw_rectangle(0., 0., vw(), vh(), bg) } else { clear_background(bg) }
    set_camera(&cam);
    let mut r = R::new(curve, c0, &lt, if g.me.scene == Scene::Town { (12., 21.) } else { (100., 200.) });
    let mut labels: Vec<(Vec3, String, Color)> = vec![];
    match g.me.scene { Scene::Town => town(&mut r, g, &lt, now, &mut labels), Scene::Room(b) => room(&mut r, g, b, &lt, &mut labels) }
    r.flush(); hud_cam(vp);
    let m = cam.matrix();
    for (p, s, c) in labels { let d2 = (p.x - c0.x).powi(2) + (p.z - c0.y).powi(2); let q = m * vec4(p.x, p.y - curve * d2, p.z, 1.); if q.w <= 0. { continue }
        let (x, y) = ((q.x / q.w + 1.) * 0.5 * vw(), (1. - q.y / q.w) * 0.5 * vh()); let tw = measure_text(&s, None, 20, 1.).width;
        draw_rectangle(x - tw / 2. - 5., y - 17., tw + 10., 23., Color::new(0., 0., 0., 0.55)); draw_text(&s, x - tw / 2., y, 20., c); }
    if rain && g.me.scene == Scene::Town { for i in 0..140 { let x = ((i * 97) as f32 + g.t * 40.) % vw(); let y = ((i * 53) as f32 + g.t * 520.) % vh(); draw_line(x, y, x - 3., y + 12., 1., Color::new(0.8, 0.85, 1., 0.45)) } }
    hud(g, now, dbg);
}
fn town(r: &mut R, g: &Game, lt: &Light, now: NaiveDateTime, labels: &mut Vec<(Vec3, String, Color)>) {
    let t = g.t; let (grass, fol) = season(now.month()); let (cx, cz) = (g.me.cam.x, g.me.cam.y); let rad = 22i32;
    let near = |x: f32, z: f32| (x - cx).powi(2) + (z - cz).powi(2) < (rad * rad) as f32;
    let water = |c: u8| matches!(c, RIVER | SEA | BRIDGE);
    let ext = |x: i32, z: i32| if x >= 0 && z >= 0 && x < W as i32 && z < H as i32 { g.map[z as usize * W + x as usize] } else if z >= 26 { SEA } else if z >= 24 { SAND } else { b'f' };
    let sand = Color::new(0.93, 0.86, 0.64, 1.); let soil = Color::new(0.55, 0.42, 0.3, 1.);
    let (z0, z1, x0, x1) = (cz as i32 - rad, cz as i32 + 9, cx as i32 - rad, cx as i32 + rad);
    // terrain
    for z in z0..z1 { for x in x0..x1 { let (fx, fz) = (x as f32, z as f32); if !near(fx + 0.5, fz + 0.5) { continue }
        let c = ext(x, z);
        if water(c) { let s = (t * 1.5 + (x + z) as f32 * 0.7).sin() * 0.035; let wc = if c == SEA { Color::new(0.2 + s, 0.5 + s, 0.78 + s, 1.) } else { Color::new(0.3 + s, 0.62 + s, 0.86 + s, 1.) };
            r.quad(vec3(fx, -0.2, fz), vec3(fx + 1., -0.2, fz), vec3(fx + 1., -0.2, fz + 1.), vec3(fx, -0.2, fz + 1.), Vec3::Y, wc);
            for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] { let nc = ext(x + dx, z + dz); if water(nc) { continue } let bc = if nc == SAND { sand } else { soil };
                let (a, b, n) = match (dx, dz) { (0, -1) => (vec3(fx, 0., fz), vec3(fx + 1., 0., fz), Vec3::Z), (0, 1) => (vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), -Vec3::Z), (-1, 0) => (vec3(fx, 0., fz + 1.), vec3(fx, 0., fz), Vec3::X), _ => (vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), -Vec3::X) };
                r.quad(a, b, b - Vec3::Y * 0.2, a - Vec3::Y * 0.2, n, bc) } }
        else { let col = match c { PATH => Color::new(0.77, 0.65, 0.47, 1.), SAND => sand, b'f' => sh(grass, 0.8), _ => grass }; let v = 1. + ((hash(x, z) % 7) as f32 - 3.) * 0.013;
            r.quad(vec3(fx, 0., fz), vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), Vec3::Y, sh(col, v)) } } }
    // ground decals: contact shadows, holes, fish shadows
    let shc = Color::new(0., 0., 0., 0.22);
    r.disc(vec3(g.me.pos.x, 0.02, g.me.pos.y), vec2(0.32, 0.26), shc);
    for n in g.npcs.iter().filter(|n| !n.inside) { r.disc(vec3(n.pos.x, 0.02, n.pos.y), vec2(0.3, 0.24), shc) }
    for tr in &g.w.trees { let p = tc((tr.x, tr.z)); if near(p.x, p.y) { r.disc(vec3(p.x, 0.02, p.y), vec2(0.7, 0.6), shc) } }
    for &h in &g.w.holes { let p = tc(h); r.disc(vec3(p.x, 0.015, p.y), vec2(0.34, 0.3), Color::new(0.22, 0.14, 0.08, 1.)) }
    for s in &g.shadows { let sz = def(s.def).size; r.disc(vec3(s.pos.x, -0.18, s.pos.y), vec2(sz * 0.35, sz * 0.6), Color::new(0.03, 0.08, 0.14, 0.5)) }
    // static scenery
    for z in z0..z1 { for x in x0..x1 { let (px, pz) = (x as f32 + 0.5, z as f32 + 0.5); if !near(px, pz) { continue } let hsh = hash(x, z);
        match ext(x, z) {
            PINE => pine(r, px, pz, 1.1 + (hsh % 3) as f32 * 0.12),
            b'f' if hsh % 3 == 0 => pine(r, px + ((hsh % 5) as f32 - 2.) * 0.1, pz, 1.2 + (hsh % 4) as f32 * 0.15),
            ROCK => { r.sphere(vec3(px, 0.12, pz), vec3(0.42, 0.28, 0.36), Color::new(0.6, 0.6, 0.62, 1.)); r.sphere(vec3(px + 0.25, 0.08, pz + 0.18), vec3(0.18, 0.13, 0.16), Color::new(0.52, 0.52, 0.55, 1.)) }
            FLOWER if g.w.trampled.contains(&(x, z)) => { let fc = [RED, YELLOW, PINK, WHITE, PURPLE][(hsh % 5) as usize]; for (ox, oz) in [(-0.2, -0.1), (0.15, -0.2), (0.05, 0.2)] { r.sphere(vec3(px + ox, 0.02, pz + oz), vec3(0.1, 0.015, 0.07), fc) } }
            FLOWER => { let fc = [RED, YELLOW, PINK, WHITE, PURPLE][(hsh % 5) as usize]; for (ox, oz) in [(-0.2, -0.1), (0.15, -0.2), (0.05, 0.2)] { r.cube(vec3(px + ox, 0.12, pz + oz), vec3(0.015, 0.12, 0.015), DARKGREEN); r.sphere(vec3(px + ox, 0.26, pz + oz), vec3(0.09, 0.05, 0.09), fc); r.sphere(vec3(px + ox, 0.29, pz + oz), Vec3::splat(0.03), GOLD) } }
            BRIDGE => { r.cube(vec3(px, 0.04, pz), vec3(0.5, 0.05, 0.48), WOOD); for k in 0..4 { r.cube(vec3(px - 0.375 + k as f32 * 0.25, 0.095, pz), vec3(0.01, 0.005, 0.47), sh(WOOD, 0.75)) }
                for zz in [pz - 0.45, pz + 0.45] { r.cube(vec3(px, 0.38, zz), vec3(0.5, 0.03, 0.03), sh(WOOD, 0.85)); r.cube(vec3(px - 0.4, 0.2, zz), vec3(0.04, 0.2, 0.04), sh(WOOD, 0.8)) } }
            _ => {}
        } } }
    // P2-01: buildings/trees between the camera (south) and the player are drawn translucent, after the player
    let pp = g.me.pos;
    let fade_b: Vec<usize> = (0..BLDS.len()).filter(|&i| { let b = &BLDS[i]; pp.x > b.x as f32 - 0.4 && pp.x < (b.x + b.w) as f32 + 0.4 && pp.y > b.z as f32 - 2.6 && pp.y < (b.z + b.d) as f32 - 0.2 }).collect();
    let fade_t: Vec<usize> = (0..g.w.trees.len()).filter(|&i| { let p = tc((g.w.trees[i].x, g.w.trees[i].z)); (p.x - pp.x).abs() < 1.1 && p.y > pp.y + 0.2 && p.y < pp.y + 2.6 }).collect();
    let tree_at = |r: &mut R, i: usize| { let tr = &g.w.trees[i]; let sk = g.shake.iter().find(|s| s.0 == i).map_or(0., |s| s.1); tree(r, tc((tr.x, tr.z)), tr.fruit.then(|| rgb(def(g.w.native).col)), sk, t, fol) };
    for (i, tr) in g.w.trees.iter().enumerate() { let p = tc((tr.x, tr.z)); if near(p.x, p.y) && !fade_t.contains(&i) { let sk = g.shake.iter().find(|s| s.0 == i).map_or(0., |s| s.1); tree(r, p, tr.fruit.then(|| rgb(def(g.w.native).col)), sk, t, fol) } }
    for bi in 0..BLDS.len() { let b = &BLDS[bi]; if near(b.x as f32 + b.w as f32 / 2., b.z as f32 + b.d as f32 / 2.) && !fade_b.contains(&bi) { building(r, bi, lt.night) }
        if g.me.pos.distance(door_stand(bi)) < 3.5 { let p = door_face(bi); labels.push((vec3(p.x, 2.7, p.y), BLDS[bi].name.into(), WHITE)) } }
    for &w in &g.w.weeds { let p = tc(w); for (ox, oz) in [(-0.1, 0.), (0.08, 0.06), (0., -0.08)] { r.cone(vec3(p.x + ox, 0., p.y + oz), 0.05, 0.22, Color::new(0.25, 0.5, 0.2, 1.)) } }
    for &h in &g.w.holes { let p = tc(h); for k in 0..5 { let a = k as f32 * 1.26; r.sphere(vec3(p.x + a.cos() * 0.36, 0.03, p.y + a.sin() * 0.32), vec3(0.1, 0.06, 0.1), Color::new(0.5, 0.36, 0.24, 1.)) } }
    for &dg in &g.w.digs { let p = tc(dg); for a in [0.7f32, -0.7] { r.with(Mat4::from_translation(vec3(p.x, 0.012, p.y)) * Mat4::from_rotation_y(a), |r| r.cube(Vec3::ZERO, vec3(0.2, 0.01, 0.025), Color::new(0.35, 0.25, 0.16, 1.))) } }
    for it in &g.w.ground { let c = rgb(def(it.def).col); match def(it.def).k { K::Shell => { r.sphere(vec3(it.x, 0.05, it.z), vec3(0.14, 0.06, 0.12), c); r.cone(vec3(it.x, 0.05, it.z), 0.08, 0.1, sh(c, 0.9)) } _ => r.sphere(vec3(it.x, 0.11, it.z), Vec3::splat(0.11), c) } }
    for (i, n) in g.npcs.iter().enumerate() { if n.inside || !near(n.pos.x, n.pos.y) { continue }
        let talking = matches!(g.me.ui, Ui::Talk { npc: Some(j), .. } if j == i);
        resident(r, i, vec3(n.pos.x, 0., n.pos.y), n.yaw, n.anim, n.moving, t, talking, n.happy);
        if n.pos.distance(g.me.pos) < 4. { labels.push((vec3(n.pos.x, 1.75, n.pos.y), RES[i].name.into(), Color::new(1., 0.95, 0.7, 1.))) } }
    human(r, g.me.id, vec3(g.me.pos.x, 0., g.me.pos.y), yaw_of(g.me.face), g.me.anim, g.me.spd, 0, g.w.p.equip, t, g.me.swing);
    others(r, g, t);
    for b in &g.bugs { let c = rgb(def(b.0).col); let fl = (t * 20. + b.2).sin().abs(); let y = 0.6 + (t * 3. + b.2).sin() * 0.15;
        for s in [-1f32, 1.] { r.sphere(vec3(b.1.x + s * 0.07, y + fl * 0.04, b.1.y), vec3(0.07, 0.01 + fl * 0.03, 0.05), c) } r.sphere(vec3(b.1.x, y, b.1.y), Vec3::splat(0.03), DARK) }
    if let Some(f) = &g.me.fish { let y = -0.15 - f.dip * 0.1; r.sphere(vec3(f.bob.x, y, f.bob.y), Vec3::splat(0.06), RED); r.sphere(vec3(f.bob.x, y + 0.05, f.bob.y), Vec3::splat(0.04), WHITE);
        let tip = g.me.pos + g.me.face * 0.75; r.seg(vec3(tip.x, 1.05, tip.y), vec3(f.bob.x, y + 0.05, f.bob.y), 0.004, Color::new(0.95, 0.95, 0.95, 1.)) }
    r.alpha = 0.32; for &i in &fade_t { tree_at(r, i) } for &bi in &fade_b { building(r, bi, lt.night) } r.alpha = 1.;
    for &(p, a, k) in &g.fx { let s = a / 0.8; match k {
        0 => for i in 0..10 { let an = i as f32 / 10. * 2. * PI; let rr = 0.15 + s * 0.6; r.sphere(p + vec3(an.cos() * rr, 0.02, an.sin() * rr), vec3(0.05, 0.015, 0.05), Color::new(0.92, 0.97, 1., 1. - s)) },
        1 => for i in 0..6 { let an = i as f32 * 1.05; r.sphere(p + vec3(an.cos() * s * 0.45, s * 0.5 - s * s * 0.4, an.sin() * s * 0.45), Vec3::splat(0.07 * (1. - s) + 0.02), Color::new(0.55, 0.4, 0.26, 1.)) },
        _ => for i in 0..5 { let an = -0.8 + i as f32 * 0.4; let d = vec3(an.sin() * 0.35, 0., an.cos() * 0.35); r.sphere(p + d * (0.5 + s), Vec3::splat(0.035), Color::new(1., 1., 1., 1. - s)) },
    } }
    for h in &g.hearts { if h.1 > 0. { r.sphere(h.0, Vec3::splat(0.06), Color::new(1., 0.4, 0.55, 1.)) } }
    if g.me.ui == Ui::Play && g.me.trans.is_none() { if let Some(tg) = g.target() { indicator(r, g.tg_pos(tg), t) } }
}
/// Other local players who are in the same scene as the viewing player.
fn others(r: &mut R, g: &Game, t: f32) {
    for o in g.others.iter().filter(|o| o.scene == g.me.scene) {
        let p = if o.scene == Scene::Town { o.pos } else { o.ipos }; let pose = match o.pose { Pose::Stand => 0, Pose::Sit(_) => 1, Pose::Lie(_) => 2 };
        let tool = if o.scene == Scene::Town { g.w.others.iter().find(|pr| pr.id == o.id).and_then(|pr| pr.equip) } else { None };
        if pose == 0 { r.disc(vec3(p.x, 0.02, p.y), vec2(0.32, 0.26), Color::new(0., 0., 0., 0.22)) }
        human(r, o.id, vec3(p.x, 0., p.y), yaw_of(o.face), o.anim, o.spd, pose, tool, t, o.swing);
    }
}
fn room(r: &mut R, g: &Game, b: usize, lt: &Light, labels: &mut Vec<(Vec3, String, Color)>) {
    let (w, d) = room_dims(b); let (wf, df) = (w as f32, d as f32); let t = g.t;
    let (fl, wl) = match BLDS[b].kind { BK::Shop => ((0.72, 0.76, 0.66), (0.93, 0.95, 0.86)), BK::Res(i) => THEME[i], _ => ((0.78, 0.6, 0.42), (0.97, 0.93, 0.84)) };
    let (floor, wall) = (rgb(fl), rgb(wl)); let furn = g.room_furn(b); let lamp = furn.iter().any(|f| f.def == 54 && f.on);
    let dim = lt.night > 0.6 && !lamp;
    r.sun = vec3(0.35, 0.85, 0.5).normalize();
    r.amb = if dim { vec3(0.3, 0.32, 0.48) } else if lamp && lt.night > 0.3 { vec3(0.68, 0.57, 0.42) } else { vec3(0.56, 0.56, 0.56) };
    r.dif = if dim { vec3(0.14, 0.14, 0.22) } else { vec3(0.44, 0.42, 0.4) };
    for z in 0..d { for x in 0..w { let (fx, fz) = (x as f32, z as f32); r.quad(vec3(fx, 0., fz), vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), Vec3::Y, sh(floor, if (x + z) % 2 == 0 { 1. } else { 0.93 })) } }
    r.quad(vec3(0., 0., 0.), vec3(wf, 0., 0.), vec3(wf, 2.4, 0.), vec3(0., 2.4, 0.), Vec3::Z, wall);
    r.cube(vec3(wf / 2., 0.08, 0.03), vec3(wf / 2., 0.08, 0.03), WOOD); r.cube(vec3(wf / 2., 2.38, 0.03), vec3(wf / 2., 0.04, 0.04), WOOD);
    for x in [-0.08, wf + 0.08] { r.cube(vec3(x, 0.5, df / 2. - 0.04), vec3(0.08, 0.5, df / 2. + 0.04), sh(wall, 0.85)) } // cut-away side walls
    let dxw = (w / 2) as f32; r.cube(vec3(dxw / 2., 0.12, df + 0.06), vec3(dxw / 2., 0.12, 0.06), sh(wall, 0.8)); r.cube(vec3((dxw + 1. + wf) / 2., 0.12, df + 0.06), vec3((wf - dxw - 1.) / 2., 0.12, 0.06), sh(wall, 0.8));
    r.cube(vec3(dxw + 0.5, 0.005, df - 0.4), vec3(0.4, 0.005, 0.3), Color::new(0.7, 0.3, 0.3, 1.)); // door mat
    if BLDS[b].kind != BK::Shop { let wx = if w > 3 { wf - 0.9 } else { wf / 2. }; r.cube(vec3(wx, 1.45, 0.04), vec3(0.42, 0.36, 0.04), WOOD); let u = r.unlit; r.unlit = true; r.cube(vec3(wx, 1.45, 0.07), vec3(0.34, 0.28, 0.02), Color::new(lt.sky.r * 0.9 + 0.1, lt.sky.g * 0.9 + 0.1, lt.sky.b * 0.9 + 0.1, 1.)); r.unlit = u }
    if BLDS[b].kind == BK::Shop {
        for &(x, z) in &COUNTER { r.cube(vec3(x as f32 + 0.5, 0.45, z as f32 + 0.5), vec3(0.5, 0.45, 0.42), WOOD); r.cube(vec3(x as f32 + 0.5, 0.92, z as f32 + 0.5), vec3(0.52, 0.03, 0.45), sh(WOOD, 1.2)) }
        r.cube(vec3(4.3, 1.05, 1.4), vec3(0.15, 0.1, 0.12), DARKGRAY);
        for k in 0..3 { r.cube(vec3(1.2 + k as f32 * 2.3, 1.4, 0.15), vec3(0.9, 0.04, 0.15), WOOD); for j in 0..4 { r.cube(vec3(0.6 + k as f32 * 2.3 + j as f32 * 0.38, 1.56, 0.15), vec3(0.12, 0.12, 0.1), rgb(DEFS[(j * 7 + k * 3) % 30].col)) } }
        let talking = matches!(g.me.ui, Ui::Sell(_) | Ui::Confirm(_)) || matches!(g.me.ui, Ui::Talk { npc: None, .. });
        resident(r, 4, vec3(3.5, 0., 0.5), 0., 0., false, t, talking, 0.);
        for (i, &(x, z)) in DISPLAYS.iter().enumerate() { let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5); r.cyl(p, 0.4, 0.22, Color::new(0.85, 0.82, 0.75, 1.));
            let (txt, show) = if i < 5 { let s = g.w.stock[i]; (if s.sold { "SOLD OUT".to_string() } else { format!("{} {}", def(s.def).name, def(s.def).buy) }, (!s.sold).then_some(s.def)) }
                else { let tl = TOOL_DISPLAY[i - 5]; (if g.w.p.tools.contains(&tl) { "Owned".into() } else { format!("{} {}", def(tl).name, def(tl).buy) }, Some(tl)) };
            match show { Some(dd) if def(dd).k == K::Tool => r.with(Mat4::from_translation(p + vec3(0., 0.3, 0.)) * Mat4::from_rotation_z(0.4) * Mat4::from_scale(Vec3::splat(1.4)), |r| draw_tool(r, dd)),
                Some(dd) => { let wall_it = def(dd).wall; furniture(r, dd, Mat4::from_translation(p + vec3(0., if wall_it { -0.2 } else { 0.22 }, 0.)) * Mat4::from_scale(Vec3::splat(0.5)), false) }
                None => r.cube(p + vec3(0., 0.4, 0.), vec3(0.25, 0.15, 0.02), Color::new(0.8, 0.2, 0.2, 1.)) }
            if g.me.ipos.distance(vec2(p.x, p.z)) < 2.2 { labels.push((p + vec3(0., 1.15, 0.), txt, if i < 5 && g.w.stock[i].sold { Color::new(1., 0.5, 0.5, 1.) } else { Color::new(1., 0.95, 0.6, 1.) })) } }
    }
    for f in &furn { furniture(r, f.def, fbase(f), f.on); if f.def == 54 && f.on { let c = fcenter(f); r.disc(vec3(c.x, 0.012, c.y), vec2(0.9, 0.9), Color::new(1., 0.85, 0.4, 0.18)) } }
    if let BK::Res(i) = BLDS[b].kind { if where_(i, g.hr) == Loc::Home { let s = res_spot(); let talking = matches!(g.me.ui, Ui::Talk { npc: Some(j), .. } if j == i);
        r.disc(vec3(s.x, 0.02, s.y), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.2)); resident(r, i, vec3(s.x, 0., s.y), yaw_of(g.me.ipos - s), 0., false, t, talking, g.npcs[i].happy) } }
    if let Some(p) = &g.me.placing { let ok = g.place_ok(&p.f).is_ok(); let c = if ok { Color::new(0.2, 0.9, 0.3, 0.45) } else { Color::new(0.95, 0.2, 0.2, 0.45) };
        if def(p.f.def).wall { r.disc(vec3(p.f.x as f32 + 0.5, 0.02, 0.3), vec2(0.45, 0.25), c) } else { for (x, z) in foot(p.f.def, p.f.x, p.f.z, p.f.rot) { r.disc(vec3(x as f32 + 0.5, 0.02, z as f32 + 0.5), vec2(0.48, 0.48), c) } }
        furniture(r, p.f.def, fbase(&p.f), false) }
    let pose = match g.me.pose { Pose::Stand => 0, Pose::Sit(_) => 1, Pose::Lie(_) => 2 };
    if pose == 0 { r.disc(vec3(g.me.ipos.x, 0.02, g.me.ipos.y), vec2(0.32, 0.26), Color::new(0., 0., 0., 0.2)) }
    human(r, g.me.id, vec3(g.me.ipos.x, 0., g.me.ipos.y), yaw_of(g.me.face), g.me.anim, g.me.spd, pose, None, t, 0.);
    others(r, g, t);
    if g.me.ui == Ui::Play && g.me.placing.is_none() && g.me.trans.is_none() { if let Some(tg) = g.target() { indicator(r, g.tg_pos(tg), t) } }
}

// ---- HUD / menus (2D, undistorted) ----
fn txt(s: &str, x: f32, y: f32, z: f32, c: Color) { draw_text(s, x, y, z, c); }
fn panel(x: f32, y: f32, w: f32, h: f32) { draw_rectangle(x, y, w, h, Color::new(0.99, 0.96, 0.88, 0.97)); draw_rectangle_lines(x, y, w, h, 3., Color::new(0.45, 0.3, 0.2, 1.)) }
fn slot_name(s: &Slot) -> String { if s.qty > 1 { format!("{} x{}", def(s.def).name, s.qty) } else { def(s.def).name.to_string() } }
fn list(items: &[String], sel: Option<usize>, x: f32, y: f32) { for (k, s) in items.iter().enumerate() { let on = sel == Some(k); if on { draw_rectangle(x - 6., y + k as f32 * 26. - 19., 330., 25., Color::new(1., 0.85, 0.4, 0.6)) } txt(s, x, y + k as f32 * 26., 22., BLACK) } }
fn hud(g: &Game, now: NaiveDateTime, dbg: Option<&str>) {
    let (sw, sh) = (vw(), vh());
    let season = match now.month() { 3..=5 => "Spring", 6..=8 => "Summer", 9..=11 => "Autumn", _ => "Winter" };
    draw_rectangle(10., 10., 540., 62., Color::new(0., 0., 0., 0.45));
    const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    txt(&format!("{} {:02}:{:02}  {} {}  {}{}", now.weekday(), now.hour(), now.minute(), MON[now.month0() as usize], now.day(), season, if rainy(g.w.day) { "  Rain" } else { "" }), 20., 35., 26., WHITE);
    txt(&format!("{} coins    pockets {}/{}    tool: {}", g.w.p.money, g.w.p.inv.len(), INV_CAP, g.w.p.equip.map_or("none", |t| def(t).name)), 20., 62., 22., GOLD);
    for (k, (m, _)) in g.me.msgs.iter().rev().filter(|m| g.t - m.1 < 5.).take(4).enumerate() { txt(m, 14., 100. + k as f32 * 24., 22., WHITE) }
    if g.me.ui == Ui::Play && g.me.placing.is_none() && g.me.trans.is_none() { if let Some(tg) = g.target() { let s = format!("[E] {}", g.tg_label(tg)); let tw = measure_text(&s, None, 24, 1.).width; draw_rectangle(sw / 2. - tw / 2. - 10., sh - 92., tw + 20., 32., Color::new(0., 0., 0., 0.55)); txt(&s, sw / 2. - tw / 2., sh - 69., 24., WHITE) } }
    if g.me.fish.as_ref().is_some_and(|f| f.bite) { txt("!", sw / 2. - 10., sh / 2. - 80., 90., Color::new(1., 0.3, 0.2, 1.)) }
    let narrow = sw < 900.;
    if narrow { draw_rectangle(sw - 62., 14., 50., 30., if g.me.id == 0 { Color::new(0.2, 0.45, 0.9, 0.9) } else { Color::new(0.9, 0.35, 0.3, 0.9) }); txt(&format!("P{}", g.me.id + 1), sw - 52., 38., 26., WHITE) }
    let hint = if narrow { if g.me.placing.is_some() { "Move  R rotate  Enter place  Esc cancel" } else { "Move  E act  Space tool  Tab tool  I bag  J journal" } } else if g.me.placing.is_some() { "Placing:  WASD/Arrows move   R rotate   Enter place   Esc cancel" } else { "WASD move  Shift run  E interact  Space use tool  Tab tool  I inventory  J journal  F/G move/pick up furniture  Esc cancel" };
    txt(hint, 12., sh - 14., 19., WHITE);
    match &g.me.ui {
        Ui::Play => {}
        Ui::Talk { who, lines, i, .. } => { panel(sw / 2. - 430., sh - 200., 860., 140.); txt(who, sw / 2. - 410., sh - 168., 26., Color::new(0.55, 0.3, 0.15, 1.)); txt(lines.get(*i).map_or("", |s| s.as_str()), sw / 2. - 410., sh - 128., 24., BLACK); txt("[E] next", sw / 2. + 330., sh - 75., 20., GRAY) }
        Ui::Inv(s) | Ui::Sell(s) => { let sell = matches!(g.me.ui, Ui::Sell(_)); panel(sw / 2. - 260., 90., 520., 520.);
            txt(if sell { "Sell to Mr. Tumble" } else { "Pockets" }, sw / 2. - 240., 125., 30., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = g.w.p.inv.iter().map(|it| if sell { format!("{}  -  {}c", slot_name(it), def(it.def).sell * it.qty as u32) } else { slot_name(it) }).collect();
            if items.is_empty() { txt("(empty)", sw / 2. - 230., 170., 22., GRAY) } list(&items, Some(*s), sw / 2. - 230., 170.);
            txt(if sell { "W/S select  Enter sell  A sell all finds  Esc close" } else { "W/S select  Enter place furniture (at home)  Esc close" }, sw / 2. - 240., 590., 19., DARKGRAY);
            if !sell { txt(&format!("Tools: {}", g.w.p.tools.iter().map(|t| def(*t).name).collect::<Vec<_>>().join(", ")), sw / 2. - 240., 565., 19., DARKGRAY) } }
        Ui::Confirm(d) => { let (n, p) = if *d < 5 { let s = g.w.stock[*d]; (def(s.def).name, def(s.def).buy) } else { let t = TOOL_DISPLAY[*d - 5]; (def(t).name, def(t).buy) };
            panel(sw / 2. - 260., sh / 2. - 80., 520., 150.); txt(&format!("Buy the {n} for {p} coins?"), sw / 2. - 235., sh / 2. - 35., 26., BLACK); txt(&format!("You have {} coins.", g.w.p.money), sw / 2. - 235., sh / 2. - 2., 22., DARKGRAY); txt("[Enter] Yes     [Esc] No", sw / 2. - 235., sh / 2. + 40., 22., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Storage(c, s) => { panel(sw / 2. - 380., 90., 760., 520.); txt("Pockets", sw / 2. - 360., 125., 28., BLACK); txt("Cabinet storage", sw / 2. + 20., 125., 28., BLACK);
            let a: Vec<String> = g.w.p.inv.iter().map(slot_name).collect(); let bb: Vec<String> = g.w.p.storage.iter().map(slot_name).collect();
            list(&a, (*c == 0).then_some(*s), sw / 2. - 350., 165.); list(&bb, (*c == 1).then_some(*s), sw / 2. + 30., 165.); txt("A/D switch side  W/S select  Enter move item  Esc close", sw / 2. - 360., 590., 19., DARKGRAY) }
        Ui::Journal => { panel(60., 80., sw - 120., sh - 160.); txt("Field Journal", 80., 115., 30., Color::new(0.45, 0.3, 0.2, 1.));
            for (col, (k, title)) in [(K::Fish, "Fish"), (K::Bug, "Bugs"), (K::Shell, "Shells & finds")].iter().enumerate() { let x = 80. + col as f32 * 380.; txt(title, x, 155., 26., BLACK);
                for (j, dd) in DEFS.iter().filter(|dd| dd.k == *k || (*k == K::Shell && matches!(dd.k, K::Treasure | K::Fruit))).enumerate() { let seen = g.w.p.seen.contains(&dd.id);
                    txt(&format!("{}{}", if avail(dd, g.mo, g.hr) && matches!(dd.k, K::Fish | K::Bug) { "* " } else { "  " }, if seen { format!("{}  {}c", dd.name, dd.sell) } else { "???".into() }), x, 185. + j as f32 * 24., 20., if seen { BLACK } else { GRAY }) } }
            txt("* = active right now", 80., sh - 100., 18., DARKGRAY) }
    }
    if let Some(src) = dbg {
        let lines = [format!("[{src} clock] {}", now.format("%a %Y-%m-%d %H:%M:%S")), format!("gameplay day {} last processed {}", crate::clock::day_id(now), g.w.day),
            format!("scene {:?} pos ({:.2},{:.2}) curve {}", g.me.scene, g.ppos().x, g.ppos().y, g.curve), format!("target {:?}", g.target()),
            format!("stock {:?}", g.w.stock.iter().map(|s| (def(s.def).name, s.sold)).collect::<Vec<_>>())];
        let (px, py, pw) = if narrow { (10., sh - 190., sw - 20.) } else { (sw - 560., 10., 550.) };
        panel(px, py, pw, 30. + 22. * (lines.len() + g.log.len().min(3)) as f32);
        for (k, l) in lines.iter().chain(g.log.iter().rev().take(3)).enumerate() { txt(l, px + 12., py + 22. + k as f32 * 22., 17., BLACK) }
    }
    if let Some((t, _, _)) = g.me.trans { draw_rectangle(0., 0., sw, sh, Color::new(0., 0., 0., 1. - ((t - 0.22).abs() / 0.22).min(1.))) }
}
