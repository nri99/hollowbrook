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
fn dark(now: NaiveDateTime) -> f32 { light(now.hour() as f32 + now.minute() as f32 / 60., false).night }
/// Smooth value noise on grid corners (0..1): a broad layer for meadow patches plus a fine layer.
fn gnoise(x: i32, z: i32) -> f32 {
    let h = |x: i32, z: i32| (hash(x, z) % 1000) as f32 / 1000.;
    let (bx, bz) = (x.div_euclid(4), z.div_euclid(4)); let (fx, fz) = (x.rem_euclid(4) as f32 / 4., z.rem_euclid(4) as f32 / 4.);
    let (sx, sz) = (fx * fx * (3. - 2. * fx), fz * fz * (3. - 2. * fz));
    let a = h(bx + 500, bz) + (h(bx + 501, bz) - h(bx + 500, bz)) * sx; let b = h(bx + 500, bz + 1) + (h(bx + 501, bz + 1) - h(bx + 500, bz + 1)) * sx;
    (a + (b - a) * sz) * 0.75 + h(x, z) * 0.25
}
/// Ground tint at a corner: darker/greener in hollows, sunnier on rises.
fn gtint(c: Color, n: f32) -> Color { let k = 0.89 + n * 0.21; Color::new(c.r * k * (0.97 + n * 0.06), c.g * k, c.b * k * (1.03 - n * 0.06), c.a) }
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
    /// Quad with a colour per corner (smooth ground shading).
    pub fn quad4(&mut self, p: [Vec3; 4], n: Vec3, c: [Color; 4]) { let o = self.base(4); for k in 0..4 { self.vtx(p[k], n, c[k]) } self.i.extend_from_slice(&[o, o + 1, o + 2, o, o + 2, o + 3]) }
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
    let (t, lv) = (t % 1000, t / 1000); if lv > 0 { r.sphere(vec3(0., 0.02, 0.), Vec3::splat(0.035), if lv == 1 { Color::new(0.8, 0.82, 0.88, 1.) } else { GOLD }) }
    match t {
        154 => { let c = if lv == 2 { GOLD } else if lv == 1 { Color::new(0.75, 0.78, 0.85, 1.) } else { Color::new(0.35, 0.6, 0.45, 1.) }; r.cyl(vec3(0., -0.12, 0.08), 0.08, 0.13, c); r.seg(vec3(0., -0.05, 0.14), vec3(0., 0.02, 0.3), 0.015, c); r.seg(vec3(0., 0.02, -0.02), vec3(0., 0.05, 0.12), 0.012, c) }
        60 => { r.seg(vec3(0., 0.05, 0.), vec3(0., 0.3, 0.45), 0.014, WOOD); for k in 0..10 { let a = k as f32 / 10. * 2. * PI; r.sphere(vec3(a.cos() * 0.12, 0.38 + a.sin() * 0.12, 0.55), Vec3::splat(0.02), WHITE) } }
        61 => { r.seg(vec3(0., 0., 0.), vec3(0., 0.5, 0.7), 0.012, WOOD); r.sphere(vec3(0., 0.05, 0.06), Vec3::splat(0.03), DARKGRAY) }
        _ => { r.seg(vec3(0., 0.1, -0.05), vec3(0., -0.2, 0.35), 0.014, WOOD); r.with(Mat4::from_translation(vec3(0., -0.25, 0.42)) * Mat4::from_rotation_x(0.9), |r| r.cube(Vec3::ZERO, vec3(0.08, 0.1, 0.012), GRAY)) }
    }
}
/// Player: large head, hair, face, outfit; pose 0 stand/walk/run, 1 sit, 2 lie. Tool held in the right hand.
fn shirt_base(id: usize) -> Color { if id == 0 { Color::new(0.95, 0.47, 0.36, 1.) } else { Color::new(0.3, 0.66, 0.48, 1.) } }
/// `look` = (shirt, hat, hairstyle index) from the player's profile.
pub fn human(r: &mut R, id: usize, look: (Option<u16>, Option<u16>, u8), pos: Vec3, yaw: f32, anim: f32, spd: f32, pose: u8, tool: Option<u16>, t: f32, swing: f32) {
    let (skin, hair, shirt, pants, shoe) = if id == 0 { (Color::new(0.98, 0.82, 0.68, 1.), Color::new(0.36, 0.2, 0.12, 1.), Color::new(0.95, 0.47, 0.36, 1.), Color::new(0.27, 0.36, 0.62, 1.), Color::new(0.3, 0.2, 0.15, 1.)) }
        else { (Color::new(0.85, 0.66, 0.5, 1.), Color::new(0.12, 0.1, 0.1, 1.), Color::new(0.3, 0.66, 0.48, 1.), Color::new(0.35, 0.3, 0.26, 1.), Color::new(0.2, 0.2, 0.25, 1.)) };
    let shirt = look.0.map_or(shirt_base(id), |d| rgb(def(d).col)); let hair = if look.2 == 0 { hair } else { rgb(HAIRS[look.2 as usize % HAIRS.len()]) };
    let k = (spd / 3.).min(1.8); let sw = anim.sin() * 0.7 * k; let bob = (anim * 2.).sin().abs() * 0.05 * k + (t * 2.2).sin() * 0.01; let lean = if spd > 4. { 0.2 } else { 0. };
    let mut base = Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), pos);
    if pose == 2 { base = base * Mat4::from_translation(vec3(0., 0.45, 0.55)) * Mat4::from_rotation_x(-FRAC_PI_2) }
    let hip = if pose == 1 { 0.14 } else { 0. };
    r.with(base, |r| {
        if spd > 4. && pose == 0 && pos.y > -0.1 { for k in 0..3 { let ph = (anim * 0.35 + k as f32 / 3.).fract(); r.sphere(vec3((k as f32 - 1.) * 0.08, 0.04 + ph * 0.08, -0.2 - ph * 0.35), Vec3::splat(0.04 + ph * 0.05), Color::new(0.92, 0.88, 0.78, 0.55 * (1. - ph))) } }
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
                if look.2 % 2 == 1 { r.sphere(vec3(0., 0.2, -0.27), Vec3::splat(0.11), hair) }
                match look.1 { Some(67) => { r.cyl(vec3(0., 0.2, 0.), 0.42, 0.025, rgb(def(67).col)); r.cyl(vec3(0., 0.22, 0.), 0.24, 0.16, rgb(def(67).col)); r.cyl(vec3(0., 0.23, 0.), 0.245, 0.04, Color::new(0.8, 0.3, 0.3, 1.)) }
                    Some(68) => { r.sphere(vec3(0., 0.12, -0.02), vec3(0.3, 0.24, 0.29), rgb(def(68).col)); r.sphere(vec3(0., 0.37, -0.02), Vec3::splat(0.06), WHITE) }
                    Some(69) => { for k in 0..9 { let a = k as f32 / 9. * 2. * PI; r.sphere(vec3(a.cos() * 0.26, 0.2, a.sin() * 0.25), Vec3::splat(0.05), [PINK, YELLOW, WHITE][k % 3]) } }
                    _ => {} }
                let bl = if (t + id as f32 * 1.7) % 4.2 < 0.13 { 0.12 } else { 1. };
                for s in [-1f32, 1.] { r.cube(vec3(s * 0.1, 0.058, 0.268), vec3(0.042, 0.011, 0.008), sh(hair, 0.55)); r.sphere(vec3(s * 0.095, -0.01, 0.245), vec3(0.038, 0.055 * bl, 0.02), DARK); if bl > 0.5 { r.sphere(vec3(s * 0.09, 0.015, 0.262), Vec3::splat(0.012), WHITE) } r.sphere(vec3(s * 0.165, -0.085, 0.215), vec3(0.045, 0.03, 0.02), Color::new(1., 0.62, 0.62, 1.)) }
                r.cube(vec3(0., -0.12, 0.255), vec3(0.035, 0.01, 0.01), Color::new(0.6, 0.25, 0.25, 1.)); }) }) });
}
/// Residents 0-3 and the shopkeeper (4): distinct heads, ears, muzzles, tails, clothes; talk mouth + happy eyes.
/// Mood emote over a resident's head (0 note, 1 sleepy, 2 thinking, 3 excited, 4 greeting), shown in short idle bursts.
pub fn emote(g: &Game, i: usize, t: f32) -> Option<u8> {
    if g.seek_hide_position(i).is_some() { return None }
    if g.w.p.invitation.as_ref().is_some_and(|v| v.slot == i && v.day == g.w.day && v.state == 0) { return Some(4) }
    let n = &g.npcs[i]; if n.happy > 0. || g.chats.iter().any(|c| c.0 == i || c.1 == i) { return None }
    let r = g.res(i); if n.pos.distance(g.me.pos) < 2.2 && !n.moving && !g.w.p.npcs[i].talked { return Some(4) }
    if (t * 0.25 + i as f32 * 1.7) % 5. > 1.4 { return None }
    if g.hr + 1 >= r.sleep || g.hr < r.wake + 1 { return Some(1) }
    Some([0, 2, 2, 3][r.pers])
}
/// Idle hobby prop held at the resident's side (CHARS order): basket, watering can, telescope, towel, yarn, wrench.
pub fn hobby_prop(r: &mut R, ch: usize, pos: Vec3, yaw: f32, t: f32) {
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), pos + vec3(0., 0., 0.)), |r| { let h = vec3(0.36, 0.34, 0.2); r.seg(vec3(0.22, 0.42, 0.06), h, 0.022, Color::new(0.97, 0.94, 0.88, 1.)); // arm reaching to the prop
        match ch { 0 => { r.cyl(h - vec3(0., 0.08, 0.), 0.09, 0.1, Color::new(0.75, 0.55, 0.3, 1.)); r.sphere(h + vec3(0., 0.04, 0.), vec3(0.07, 0.03, 0.07), Color::new(0.95, 0.85, 0.6, 1.)) }
            1 => { r.cyl(h - vec3(0., 0.08, 0.), 0.09, 0.15, Color::new(0.4, 0.6, 0.45, 1.)); r.seg(h + vec3(0., 0., 0.05), h + vec3(0., -0.02, 0.2), 0.015, Color::new(0.4, 0.6, 0.45, 1.)); let ph = (t * 3.) % 1.; r.sphere(h + vec3(0., -0.05 - ph * 0.25, 0.22), Vec3::splat(0.015), Color::new(0.5, 0.75, 1., 1.)) }
            2 => { r.seg(h, h + vec3(0.05, 0.3, 0.12), 0.045, Color::new(0.8, 0.68, 0.35, 1.)); r.cyl(h + vec3(0.05, 0.3, 0.12), 0.05, 0.04, Color::new(0.3, 0.3, 0.35, 1.)) }
            3 => r.cube(h + vec3(0., -0.05, 0.), vec3(0.03, 0.12, 0.08), Color::new(0.95, 0.95, 0.95, 1.)),
            4 => { r.sphere(h, Vec3::splat(0.06), Color::new(0.85, 0.4, 0.55, 1.)); r.seg(h + vec3(-0.05, 0.05, 0.), h + vec3(0.05, 0.12, 0.05), 0.006, Color::new(0.8, 0.75, 0.6, 1.)) }
            _ => r.seg(h, h + vec3(0., 0.18, 0.06), 0.02, Color::new(0.6, 0.6, 0.65, 1.)) } });
}
pub fn draw_emote(r: &mut R, e: u8, p: Vec3, t: f32, labels: &mut Vec<(Vec3, String, Color)>) {
    match e {
        0 => { let c = Color::new(0.35, 0.3, 0.8, 1.); r.sphere(p, vec3(0.06, 0.045, 0.04), c); r.seg(p + vec3(0.05, 0., 0.), p + vec3(0.05, 0.2, 0.), 0.012, c); r.seg(p + vec3(0.05, 0.2, 0.), p + vec3(0.12, 0.15, 0.), 0.012, c) }
        1 => labels.push((p, if (t * 1.5) as i32 % 2 == 0 { "z".into() } else { "z Z".into() }, Color::new(0.8, 0.85, 1., 1.))),
        2 => labels.push((p, "...".into(), Color::new(0.95, 0.95, 0.95, 1.))),
        3 => labels.push((p, "!".into(), Color::new(1., 0.85, 0.3, 1.))),
        _ => { r.sphere(p, Vec3::splat(0.07), Color::new(1., 0.6, 0.65, 1.)); r.sphere(p + vec3(0.07, 0.04, 0.), Vec3::splat(0.07), Color::new(1., 0.6, 0.65, 1.)); r.cone(p + vec3(0.035, -0.02, 0.), 0.1, -0.12, Color::new(1., 0.6, 0.65, 1.)) }
    }
}
pub fn resident(r: &mut R, i: usize, pos: Vec3, yaw: f32, anim: f32, moving: bool, t: f32, talk: bool, happy: f32) { resident_u(r, i, pos, yaw, anim, moving, t, talk, happy, false) }
/// `umbrella`: held over the head (outdoors in rain).
pub fn resident_u(r: &mut R, i: usize, pos: Vec3, yaw: f32, anim: f32, moving: bool, t: f32, talk: bool, happy: f32, umbrella: bool) {
    let (fur, cloth) = if i < CHARS.len() { (rgb(CHARS[i].fur), rgb(CHARS[i].cloth)) } else if (14..18).contains(&i) { (rgb(VISITORS[i - 14].fur), rgb(VISITORS[i - 14].cloth)) } else if i == 13 { (Color::new(0.82, 0.66, 0.48, 1.), Color::new(0.3, 0.48, 0.44, 1.)) } else if i == 12 { (Color::new(0.96, 0.96, 0.94, 1.), Color::new(0.2, 0.25, 0.45, 1.)) } else if i == 11 { (Color::new(0.93, 0.93, 0.96, 1.), Color::new(0.32, 0.34, 0.6, 1.)) } else { (Color::new(0.55, 0.65, 0.38, 1.), Color::new(0.92, 0.86, 0.7, 1.)) };
    let light = Color::new(0.97, 0.94, 0.88, 1.); let sw = if moving { anim.sin() * 0.6 } else { 0. };
    let jump = if happy > 0. { (t * 10.).sin().abs() * 0.18 } else { 0. }; let bob = (t * 2. + i as f32).sin() * 0.012;
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), pos + vec3(0., jump, 0.)), |r| {
        for s in [-1f32, 1.] { r.with(Mat4::from_translation(vec3(s * 0.09, 0.24, 0.)) * Mat4::from_rotation_x(sw * s), |r| r.cube(vec3(0., -0.11, 0.02), vec3(0.065, 0.12, 0.08), fur)) }
        r.with(Mat4::from_translation(vec3(0., 0.24 + bob, 0.)), |r| {
            r.cube(vec3(0., 0.15, 0.), vec3(0.17, 0.16, 0.12), cloth);
            if i == 3 { for y in [0.08, 0.2] { r.cube(vec3(0., y, 0.), vec3(0.175, 0.025, 0.125), WHITE) } }
            if i == 10 { r.sphere(vec3(0., 0.2, -0.13), vec3(0.3, 0.26, 0.24), Color::new(0.45, 0.38, 0.22, 1.)) }
            for s in [-1f32, 1.] { let wave = if talk && s > 0. { (t * 5.).sin() * 0.4 - 0.5 } else { 0. };
                r.with(Mat4::from_translation(vec3(s * 0.21, 0.27, 0.)) * Mat4::from_rotation_x(-sw * s + wave), |r| r.cube(vec3(0., -0.1, 0.), vec3(0.05, 0.1, 0.05), fur)) }
            let wag = (t * 4.).sin();
            if i == 15 { r.sphere(vec3(0., 0.2, -0.14), vec3(0.24, 0.2, 0.16), Color::new(0.4, 0.5, 0.3, 1.)); for k in 0..3 { r.sphere(vec3(-0.1 + k as f32 * 0.1, 0.28, -0.27), vec3(0.05, 0.05, 0.02), Color::new(0.55, 0.62, 0.38, 1.)) } }
            match i { 0 => { r.sphere(vec3(wag * 0.05, 0.05, -0.28), vec3(0.12, 0.12, 0.22), fur); r.sphere(vec3(wag * 0.07, 0.07, -0.47), Vec3::splat(0.08), light) }
                1 => r.sphere(vec3(0., 0.02, -0.15), Vec3::splat(0.07), fur),
                3 => { r.sphere(vec3(wag * 0.03, 0., -0.22), vec3(0.08, 0.07, 0.16), fur); r.sphere(vec3(wag * 0.06, -0.05, -0.38), vec3(0.06, 0.05, 0.12), fur) } _ => {} }
            r.with(Mat4::from_translation(vec3(0., 0.58, 0.02)) * Mat4::from_rotation_z(if talk { (t * 3.).sin() * 0.08 } else { 0. }) * Mat4::from_rotation_x(if talk { (t * 5.).sin().max(0.) * -0.06 } else { 0. }), |r| head(r, i, fur, light, t, talk, happy));
            if umbrella { r.seg(vec3(0.2, 0.15, 0.08), vec3(0.08, 1.05, 0.02), 0.015, DARKGRAY); r.cone(vec3(0.08, 0.95, 0.02), 0.5, 0.22, rgb(CHARS[i.min(CHARS.len() - 1)].cloth)); } }) });
}
fn head(r: &mut R, i: usize, fur: Color, light: Color, t: f32, talk: bool, happy: f32) {
    let (hr, ey, ez) = match i { 2 => (vec3(0.3, 0.2, 0.26), 0.19, 0.13), 10 => (vec3(0.2, 0.19, 0.21), 0.02, 0.17), _ => (vec3(0.26, 0.24, 0.25), 0.03, 0.2) };
    r.sphere(Vec3::ZERO, hr, fur);
    match i { // campground visitors
        14 => { for sx in [-1f32, 1.] { r.with(Mat4::from_translation(vec3(sx * 0.1, 0.3, -0.03)) * Mat4::from_rotation_z(sx * -0.15), |r| { r.sphere(Vec3::ZERO, vec3(0.06, 0.2, 0.045), fur); r.sphere(vec3(0., 0., 0.03), vec3(0.035, 0.15, 0.02), Color::new(0.95, 0.75, 0.75, 1.)) }) } r.sphere(vec3(0., -0.06, 0.22), Vec3::splat(0.035), Color::new(0.9, 0.5, 0.55, 1.)) }
        15 => { r.sphere(vec3(0., -0.07, 0.2), vec3(0.12, 0.07, 0.08), light); for k in 0..3 { r.sphere(vec3(-0.12 + k as f32 * 0.12, 0.17, -0.05), Vec3::splat(0.035), sh(fur, 0.7)) } }
        16 => { for sx in [-1f32, 1.] { r.with(Mat4::from_translation(vec3(sx * 0.1, 0.22, -0.05)) * Mat4::from_rotation_x(-0.5) * Mat4::from_rotation_z(sx * -0.3), |r| r.cone(Vec3::ZERO, 0.04, 0.2, Color::new(0.75, 0.68, 0.55, 1.))); r.sphere(vec3(sx * 0.25, 0.02, 0.), vec3(0.09, 0.04, 0.05), fur) } r.sphere(vec3(0., -0.2, 0.15), vec3(0.04, 0.07, 0.03), light) }
        17 => { r.sphere(vec3(0., -0.04, 0.24), vec3(0.07, 0.055, 0.06), Color::new(0.95, 0.6, 0.65, 1.)); for sx in [-1f32, 1.] { r.sphere(vec3(sx * 0.18, 0.12, -0.02), Vec3::splat(0.04), sh(fur, 0.8)) } r.cyl(vec3(0., 0.22, 0.), 0.2, 0.03, Color::new(0.75, 0.55, 0.25, 1.)); r.cyl(vec3(0., 0.24, 0.), 0.13, 0.12, Color::new(0.75, 0.55, 0.25, 1.)) }
        _ => {} }
    match i {
        0 => { r.sphere(vec3(0., -0.07, 0.18), vec3(0.11, 0.08, 0.14), light); r.sphere(vec3(0., -0.04, 0.32), Vec3::splat(0.035), DARK); for s in [-1f32, 1.] { r.cone(vec3(s * 0.14, 0.16, -0.02), 0.08, 0.2, fur); r.sphere(vec3(s * 0.13, -0.08, 0.16), vec3(0.06, 0.04, 0.05), light) } }
        1 => { r.cube(vec3(0., 0.13, 0.1), vec3(0.05, 0.11, 0.15), light); for s in [-1f32, 1.] { r.sphere(vec3(s * 0.09, 0.02, 0.17), vec3(0.075, 0.09, 0.06), DARK); r.sphere(vec3(s * 0.17, 0.17, -0.02), Vec3::splat(0.06), fur) } r.sphere(vec3(0., -0.08, 0.2), vec3(0.08, 0.06, 0.07), light); r.sphere(vec3(0., -0.06, 0.27), Vec3::splat(0.03), DARK) }
        2 => { for s in [-1f32, 1.] { r.sphere(vec3(s * 0.13, 0.16, 0.08), Vec3::splat(0.095), fur); r.sphere(vec3(s * 0.2, -0.06, 0.15), vec3(0.05, 0.03, 0.03), Color::new(1., 0.6, 0.6, 1.)) } }
        4 => { for (x, y, z) in [(0., 0.2, -0.02), (-0.15, 0.14, -0.04), (0.15, 0.14, -0.04), (-0.08, 0.24, 0.08), (0.08, 0.24, 0.08), (0., 0.12, -0.18)] { r.sphere(vec3(x, y, z), Vec3::splat(0.12), WHITE) } for s in [-1f32, 1.] { r.sphere(vec3(s * 0.25, 0.02, 0.), vec3(0.08, 0.04, 0.05), fur) } }
        5 => { for k in 0..7 { let a = -1.2 + k as f32 * 0.4; r.with(Mat4::from_translation(vec3(a.sin() * 0.2, 0.1 + a.cos() * 0.05, -0.12)) * Mat4::from_rotation_x(-1.9), |r| r.cone(Vec3::ZERO, 0.07, 0.22, sh(fur, 0.6))) } r.sphere(vec3(0., -0.06, 0.2), vec3(0.08, 0.06, 0.1), light); r.sphere(vec3(0., -0.04, 0.29), Vec3::splat(0.03), DARK) }
        12 => { r.with(Mat4::from_translation(vec3(0., -0.06, 0.2)) * Mat4::from_rotation_x(FRAC_PI_2 * 0.92), |r| r.cone(Vec3::ZERO, 0.09, 0.42, Color::new(0.98, 0.7, 0.25, 1.))); r.sphere(vec3(0., -0.13, 0.32), vec3(0.07, 0.06, 0.12), Color::new(0.98, 0.8, 0.5, 1.)); r.cube(vec3(0., 0.22, 0.), vec3(0.2, 0.04, 0.2), Color::new(0.2, 0.25, 0.45, 1.)) }
        11 => { r.with(Mat4::from_translation(vec3(0., -0.03, 0.2)) * Mat4::from_rotation_x(FRAC_PI_2), |r| r.cone(Vec3::ZERO, 0.055, 0.3, GOLD)); r.sphere(vec3(0., 0.2, -0.12), vec3(0.04, 0.12, 0.04), DARKGRAY) }
        3 => { r.sphere(vec3(0., -0.07, 0.17), vec3(0.12, 0.08, 0.1), light); r.sphere(vec3(0., -0.04, 0.27), Vec3::splat(0.035), DARK); for s in [-1f32, 1.] { r.sphere(vec3(s * 0.18, 0.13, -0.02), Vec3::splat(0.05), fur) } }
        _ => {}
    }
    for s in [-1f32, 1.] { let ex = s * if i == 2 { 0.13 } else { 0.085 };
        if happy > 0. { r.cube(vec3(ex, ey, ez + 0.06), vec3(0.035, 0.009, 0.012), if i == 1 { WHITE } else { DARK }) }
        else { let bl = if (t + i as f32 * 1.3) % 4.6 < 0.13 { 0.15 } else { 1. }; r.sphere(vec3(ex, ey, ez + 0.05), vec3(0.03, 0.042 * bl, 0.02), if i == 1 { WHITE } else { DARK }); r.sphere(vec3(ex + 0.01, ey + 0.012, ez + 0.068), Vec3::splat(0.011), if i == 1 { DARK } else { WHITE }) } }
    let open = if talk { (t * 12.).sin().abs() * 0.028 } else { 0. } + 0.008;
    let (my, mz, mw) = match i { 0 => (-0.14, 0.28, 0.035), 2 => (-0.05, 0.255, 0.14), 10 => (-0.08, 0.2, 0.03), 3 => (-0.13, 0.25, 0.035), _ => (-0.13, 0.24, 0.035) };
    r.cube(vec3(0., my, mz), vec3(mw, open, 0.012), Color::new(0.45, 0.15, 0.15, 1.));
}
pub fn furniture(r: &mut R, d: u16, base: Mat4, on: bool, tint: u8) {
    let (c0, k) = (def(d).col, TINTS[tint as usize % TINTS.len()]); let c = Color::new((c0.0 * k.0).min(1.), (c0.1 * k.1).min(1.), (c0.2 * k.2).min(1.), 1.);
    r.with(base, |r| { match model(d) {
        50 => { r.cube(vec3(0., 0.4, 0.), vec3(0.3, 0.05, 0.3), c); for (x, z) in [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)] { r.cube(vec3(x, 0.18, z), vec3(0.04, 0.18, 0.04), sh(c, 0.8)) } r.cube(vec3(0., 0.72, -0.27), vec3(0.3, 0.28, 0.04), c) }
        51 => { r.cube(vec3(0., 0.22, 0.), vec3(0.95, 0.18, 0.4), c); for x in [-0.47, 0.47] { r.cube(vec3(x, 0.45, 0.05), vec3(0.44, 0.07, 0.33), sh(c, 1.15)) } r.cube(vec3(0., 0.6, -0.32), vec3(0.95, 0.25, 0.09), c); for x in [-0.92, 0.92] { r.cube(vec3(x, 0.45, 0.), vec3(0.07, 0.15, 0.4), sh(c, 0.85)) } }
        52 => { r.cyl(vec3(0., 0.68, 0.), 0.42, 0.06, c); r.cyl(Vec3::ZERO, 0.06, 0.68, sh(c, 0.8)); r.cyl(Vec3::ZERO, 0.25, 0.04, sh(c, 0.8)) }
        53 => { r.cube(vec3(0., 0.18, 0.), vec3(0.45, 0.14, 0.95), WOOD); r.cube(vec3(0., 0.36, 0.05), vec3(0.42, 0.06, 0.88), WHITE); r.cube(vec3(0., 0.4, 0.3), vec3(0.44, 0.05, 0.62), c); r.cube(vec3(0., 0.45, -0.7), vec3(0.32, 0.06, 0.15), WHITE); r.cube(vec3(0., 0.5, -0.96), vec3(0.45, 0.4, 0.04), WOOD) }
        54 => { r.cyl(Vec3::ZERO, 0.18, 0.05, WOOD); r.cyl(vec3(0., 0.05, 0.), 0.03, 0.8, WOOD); let u = r.unlit; r.unlit = on; r.cone(vec3(0., 0.75, 0.), 0.27, 0.34, if on { Color::new(1., 0.93, 0.62, 1.) } else { c }); r.unlit = u }
        55 => { r.cube(vec3(0., 0.55, 0.), vec3(0.42, 0.55, 0.35), c); r.cube(vec3(0., 0.55, 0.352), vec3(0.01, 0.5, 0.005), sh(c, 0.6)); for x in [-0.08, 0.08] { r.sphere(vec3(x, 0.6, 0.37), Vec3::splat(0.03), GOLD) } }
        56 => { r.cyl(Vec3::ZERO, 0.22, 0.32, Color::new(0.8, 0.45, 0.3, 1.)); for (x, y, z, s) in [(0., 0.6, 0., 0.25), (0.15, 0.48, 0.08, 0.17), (-0.14, 0.5, -0.05, 0.18)] { r.sphere(vec3(x, y, z), Vec3::splat(s), c) } }
        57 => { r.cube(vec3(0., 1.35, 0.03), vec3(0.36, 0.28, 0.03), WOOD); r.cube(vec3(0., 1.35, 0.065), vec3(0.3, 0.22, 0.005), c); r.cube(vec3(0., 1.24, 0.072), vec3(0.3, 0.11, 0.003), Color::new(0.35, 0.65, 0.35, 1.)); r.sphere(vec3(0.15, 1.45, 0.07), Vec3::splat(0.05), YELLOW) }
        58 => { r.cyl(vec3(0., 0., 0.), 0.25, 0.38, sh(c, 0.85)); r.cyl(vec3(0., 0.38, 0.), 0.3, 0.06, c) }
        59 => { r.cyl(Vec3::ZERO, 0.2, 0.08, WOOD); r.sphere(vec3(0., 0.42, 0.), Vec3::splat(0.32), Color::new(c.r, c.g, c.b, 0.85)); for k in 0..6 { let a = k as f32; r.seg(vec3(a.cos() * 0.3, 0.3, a.sin() * 0.3), vec3(a.cos() * 0.15, 0.7, a.sin() * 0.15), 0.01, WOOD) } }
        _ => r.cube(vec3(0., 0.3, 0.), vec3(0.3, 0.3, 0.3), c),
    }
    match d { // collection accents
        90 => for x in [-0.2f32, 0., 0.2] { r.sphere(vec3(x, 0.47, 0.22), Vec3::splat(0.05), [PINK, YELLOW, WHITE][((x + 0.2) * 5.) as usize % 3]) },
        91 => { r.cyl(vec3(0., 0.74, 0.), 0.06, 0.12, Color::new(0.4, 0.6, 0.85, 1.)); for k in 0..3 { r.sphere(vec3(-0.03 + k as f32 * 0.03, 0.9, 0.), Vec3::splat(0.04), [PINK, YELLOW, WHITE][k]) } }
        92 => for k in 0..5 { let a = k as f32 * 1.256; r.sphere(vec3(a.cos() * 0.18, 0.06, a.sin() * 0.18), Vec3::splat(0.04), PINK) },
        93 | 94 => r.cube(vec3(0., 0.28, 0.), vec3(if d == 93 { 0.97 } else { 0.43 }, 0.025, if d == 93 { 0.42 } else { 0.36 }), Color::new(0.85, 0.8, 0.65, 1.)),
        95 => r.cube(vec3(0., 0.72, 0.), vec3(0.22, 0.02, 0.22), Color::new(0.75, 0.6, 0.3, 1.)),
        _ => {}
    } });
}
/// Original procedural painting for art id `d`, drawn on a wall plane centred at `p` (facing +z).
pub fn painting(r: &mut R, d: u16, p: Vec3, w: f32, h: f32) {
    let k = (d - 100) as usize; let (sky, ground, acc) = [((1., 0.82, 0.6), (0.45, 0.7, 0.35), (1., 0.95, 0.6)), ((0.6, 0.85, 0.95), (0.95, 0.88, 0.65), (0.2, 0.55, 0.75)), ((0.85, 0.9, 1.), (0.5, 0.72, 0.4), (0.98, 0.65, 0.75)),
        ((0.1, 0.12, 0.3), (0.2, 0.25, 0.3), (1., 0.85, 0.4)), ((0.7, 0.88, 0.95), (0.3, 0.6, 0.35), (0.3, 0.55, 0.85)), ((0.75, 0.8, 0.88), (0.95, 0.97, 1.), (0.35, 0.45, 0.4)),
        ((0.85, 0.8, 0.9), (0.4, 0.55, 0.3), (0.75, 0.3, 0.5)), ((0.78, 0.8, 0.82), (0.45, 0.55, 0.62), (0.55, 0.35, 0.25))][k];
    r.cube(p, vec3(w / 2. + 0.05, h / 2. + 0.05, 0.02), Color::new(0.55, 0.4, 0.2, 1.));
    let z = p.z + 0.025; r.cube(vec3(p.x, p.y + h * 0.15, z), vec3(w / 2., h * 0.35, 0.003), rgb(sky)); r.cube(vec3(p.x, p.y - h * 0.35, z), vec3(w / 2., h * 0.15, 0.003), rgb(ground));
    let z2 = z + 0.004;
    match k {
        0 => { r.sphere(vec3(p.x + w * 0.25, p.y + h * 0.2, z2), vec3(h * 0.12, h * 0.12, 0.002), rgb(acc)); for i in 0..3 { r.cube(vec3(p.x - w * 0.3 + i as f32 * w * 0.2, p.y - h * 0.12, z2), vec3(w * 0.06, h * 0.08, 0.002), Color::new(0.9, 0.85, 0.75, 1.)) } }
        1 => { r.sphere(vec3(p.x, p.y - h * 0.05, z2), vec3(w * 0.22, h * 0.14, 0.002), Color::new(0.55, 0.5, 0.48, 1.)); r.cube(vec3(p.x, p.y - h * 0.2, z2), vec3(w / 2., h * 0.04, 0.002), rgb(acc)) }
        2 => for i in 0..4 { let x = p.x - w * 0.35 + i as f32 * w * 0.23; r.cube(vec3(x, p.y - h * 0.12, z2), vec3(0.012, h * 0.12, 0.002), Color::new(0.4, 0.25, 0.15, 1.)); r.sphere(vec3(x, p.y + h * 0.05, z2), vec3(w * 0.09, h * 0.1, 0.002), rgb(acc)) },
        3 => for i in 0..5 { let x = p.x - w * 0.4 + i as f32 * w * 0.2; r.sphere(vec3(x, p.y + h * (0.1 + (i % 2) as f32 * 0.12), z2), vec3(0.025, 0.035, 0.002), rgb(acc)) },
        6 => for i in 0..9 { let x = p.x - w * 0.4 + (i % 5) as f32 * w * 0.2; let y = p.y - h * (0.3 - (i / 5) as f32 * 0.12); r.cube(vec3(x, y + h * 0.06, z2), vec3(0.008, h * 0.06, 0.002), Color::new(0.3, 0.5, 0.25, 1.)); r.sphere(vec3(x, y + h * 0.13, z2 + 0.001), vec3(0.03, 0.03, 0.001), [rgb(acc), YELLOW, WHITE][i % 3]) },
        7 => { r.cube(vec3(p.x + w * 0.1, p.y - h * 0.12, z2), vec3(w * 0.18, h * 0.05, 0.002), rgb(acc)); r.cube(vec3(p.x + w * 0.12, p.y + h * 0.02, z2), vec3(0.01, h * 0.12, 0.002), rgb(acc)); for i in 0..3 { r.cube(vec3(p.x, p.y + h * (0.05 + i as f32 * 0.1), z2 + 0.001), vec3(w / 2., h * 0.03, 0.001), Color::new(1., 1., 1., 0.35)) } }
        4 => { r.sphere(vec3(p.x, p.y - h * 0.28, z2), vec3(w * 0.3, h * 0.08, 0.002), rgb(acc)); r.sphere(vec3(p.x - w * 0.1, p.y - h * 0.27, z2 + 0.001), vec3(0.03, 0.02, 0.001), Color::new(0.3, 0.75, 0.3, 1.)) }
        _ => { for i in 0..3 { r.cone(vec3(p.x - w * 0.3 + i as f32 * w * 0.3, p.y - h * 0.2, z2), 0.06, h * 0.35, rgb(acc)) } for i in 0..8 { r.sphere(vec3(p.x - w * 0.4 + (i * 37 % 10) as f32 * w * 0.09, p.y + h * (0.05 + (i * 13 % 7) as f32 * 0.04), z2 + 0.001), Vec3::splat(0.012), WHITE) } }
    }
}
pub fn fbase(f: &Furn) -> Mat4 { let c = if def(f.def).wall { vec2(f.x as f32 + 0.5, 0.) } else { fcenter(f) }; Mat4::from_rotation_translation(Quat::from_rotation_y(f.rot as f32 * FRAC_PI_2), vec3(c.x, 0., c.y)) }
fn building(r: &mut R, bi: usize, night: f32, paint: Option<((f32, f32, f32), (f32, f32, f32))>) {
    let b = &BLDS[bi]; let (x, z, w, d) = (b.x as f32, b.z as f32, b.w as f32, b.d as f32); let civic = b.kind == BK::Civic;
    let hg = if civic { 2.0 } else { 1.5 }; let (wall, roof) = paint.map_or((rgb(b.wall), rgb(b.roof)), |(w, r)| (rgb(w), rgb(r))); let c = vec3(x + w / 2., 0., z + d / 2.);
    r.cube(c + vec3(0., 0.08, 0.), vec3(w / 2. + 0.04, 0.08, d / 2. + 0.04), Color::new(0.62, 0.6, 0.57, 1.));
    r.cube(c + vec3(0., hg / 2., 0.), vec3(w / 2. - 0.05, hg / 2., d / 2. - 0.05), wall);
    let (sid, trim) = (sh(wall, 0.9), sh(wall, 1.12)); let n = (hg / 0.22) as i32;
    for k in 1..n { let y = k as f32 * 0.22; r.cube(vec3(c.x, y, z + d - 0.045), vec3(w / 2. - 0.05, 0.012, 0.006), sid); for sx in [x + 0.045, x + w - 0.045] { r.cube(vec3(sx, y, c.z), vec3(0.006, 0.012, d / 2. - 0.05), sid) } }
    for (cx, cz) in [(x + 0.05, z + d - 0.05), (x + w - 0.05, z + d - 0.05), (x + 0.05, z + 0.05), (x + w - 0.05, z + 0.05)] { r.cube(vec3(cx, hg / 2., cz), vec3(0.05, hg / 2., 0.05), trim) }
    r.roof(c + vec3(0., hg, 0.), w / 2. + 0.25, d / 2. + 0.3, if civic { 1.0 } else { 0.85 }, roof);
    // Fascia beneath the existing overhang: the roof surface and silhouette stay intact.
    for sz in [-1f32, 1.] {
        r.cube(c + vec3(0., hg - 0.03, sz * (d / 2. + 0.26)), vec3(w / 2. + 0.24, 0.025, 0.04), trim);
    }
    let dx = door_face(bi).x; let fz = z + d - 0.05;
    r.cube(vec3(dx, 0.52, fz), vec3(0.3, 0.52, 0.06), Color::new(0.45, 0.28, 0.18, 1.)); r.sphere(vec3(dx + 0.18, 0.5, fz + 0.07), Vec3::splat(0.04), GOLD);
    let lit = night > 0.35; let win = if lit { Color::new(1., 0.85, 0.5, 1.) } else { Color::new(0.68, 0.84, 0.95, 1.) };
    for wx in [x + 0.5, x + w - 0.5] {
        r.cube(vec3(wx, 0.98, fz), vec3(0.24, 0.24, 0.05), WHITE);
        let u = r.unlit; r.unlit = lit; r.cube(vec3(wx, 0.98, fz + 0.012), vec3(0.19, 0.19, 0.05), win); r.unlit = u;
        // Narrow painted dividers sit above the glass face (fz + 0.062).
        let wz = fz + 0.076;
        r.quad(vec3(wx - 0.012, 0.79, wz), vec3(wx + 0.012, 0.79, wz), vec3(wx + 0.012, 1.17, wz), vec3(wx - 0.012, 1.17, wz), Vec3::Z, WHITE);
        r.quad(vec3(wx - 0.19, 0.968, wz + 0.002), vec3(wx + 0.19, 0.968, wz + 0.002), vec3(wx + 0.19, 0.992, wz + 0.002), vec3(wx - 0.19, 0.992, wz + 0.002), Vec3::Z, WHITE);
        r.cube(vec3(wx, 0.735, fz + 0.045), vec3(0.27, 0.025, 0.1), WHITE);
    }
    match b.kind {
        BK::Shop => { for k in 0..7 { r.cube(vec3(x + 0.4 + k as f32 * (w - 0.8) / 6., hg * 0.82, fz + 0.32), vec3((w - 0.8) / 14., 0.04, 0.3), if k % 2 == 0 { WHITE } else { Color::new(0.2, 0.6, 0.6, 1.) }) } r.cube(vec3(c.x, hg + 0.15, fz + 0.1), vec3(0.8, 0.22, 0.05), WOOD) }
        BK::Museum => { for k in 0..4 { r.cyl(vec3(x + 0.45 + k as f32 * (w - 0.9) / 3., 0.16, fz + 0.3), 0.1, hg - 0.16, Color::new(0.97, 0.96, 0.92, 1.)) } r.cube(vec3(c.x, hg - 0.05, fz + 0.3), vec3(w / 2., 0.08, 0.14), Color::new(0.97, 0.96, 0.92, 1.)) }
        BK::Civic => { r.cube(vec3(c.x, hg + 1.15, c.z), vec3(0.38, 0.55, 0.38), wall); r.roof(vec3(c.x, hg + 1.7, c.z), 0.48, 0.48, 0.5, roof); r.cylinder_clock(c.x, hg + 1.25, c.z + 0.39);
            r.seg(vec3(c.x, hg + 2.2, c.z), vec3(c.x, hg + 2.9, c.z), 0.02, DARKGRAY); r.cube(vec3(c.x + 0.2, hg + 2.75, c.z), vec3(0.2, 0.12, 0.01), Color::new(0.9, 0.3, 0.3, 1.)); }
        _ => r.cube(c + vec3(w / 2. - 0.6, hg + 0.65, -0.25), vec3(0.13, 0.32, 0.13), Color::new(0.65, 0.35, 0.3, 1.)),
    }
}
impl R { fn cylinder_clock(&mut self, x: f32, y: f32, z: f32) { self.with(Mat4::from_translation(vec3(x, y, z)) * Mat4::from_rotation_x(FRAC_PI_2), |r| { r.cyl(Vec3::ZERO, 0.22, 0.02, WHITE); r.cube(vec3(0., 0.03, -0.08), vec3(0.015, 0.01, 0.09), DARK) }) } }
/// Community project models (also drawn translucent as a site preview).
fn draw_project(r: &mut R, kind: u8, x: i32, z: i32, night: f32, ghost: bool) {
    let (px, pz) = (x as f32 + 0.5, z as f32 + 0.5);
    match kind {
        0 => { r.cube(vec3(px, 0.38, pz), vec3(0.45, 0.05, 0.2), WOOD); r.cube(vec3(px, 0.62, pz - 0.18), vec3(0.45, 0.16, 0.03), WOOD); for xx in [-0.38, 0.38] { r.cube(vec3(px + xx, 0.19, pz), vec3(0.04, 0.19, 0.18), DARKGRAY) } }
        1 => { r.cyl(vec3(px, 0., pz), 0.06, 1.6, Color::new(0.25, 0.27, 0.3, 1.)); let lit = night > 0.3 && !ghost; let u = r.unlit; r.unlit = lit; r.cube(vec3(px, 1.72, pz), vec3(0.14, 0.16, 0.14), if lit { Color::new(1., 0.88, 0.55, 1.) } else { Color::new(0.85, 0.85, 0.75, 1.) }); r.unlit = u;
            r.roof(vec3(px, 1.88, pz), 0.18, 0.18, 0.12, Color::new(0.25, 0.27, 0.3, 1.)); if lit { r.disc(vec3(px, 0.025, pz), vec2(1.3, 1.3), Color::new(1., 0.85, 0.45, 0.22)) } }
        2 => { r.cube(vec3(px, 0.22, pz), vec3(0.42, 0.22, 0.42), Color::new(0.62, 0.6, 0.58, 1.)); r.cube(vec3(px, 0.45, pz), vec3(0.38, 0.02, 0.38), Color::new(0.4, 0.28, 0.18, 1.)); for (k, (ox, oz)) in [(-0.2, -0.2), (0.2, -0.15), (0., 0.1), (-0.15, 0.22), (0.22, 0.2)].iter().enumerate() { r.sphere(vec3(px + ox, 0.58, pz + oz), Vec3::splat(0.09), [RED, YELLOW, PINK, PURPLE, WHITE][k]) } }
        4 => { r.cyl(vec3(px, 0., pz), 0.48, 0.3, Color::new(0.7, 0.7, 0.72, 1.)); r.cyl(vec3(px, 0.3, pz), 0.4, 0.01, Color::new(0.35, 0.65, 0.9, 1.)); r.cyl(vec3(px, 0.3, pz), 0.08, 0.6, Color::new(0.7, 0.7, 0.72, 1.)); r.sphere(vec3(px, 0.95, pz), Vec3::splat(0.1), Color::new(0.6, 0.85, 1., 1.)) }
        5 => { for s in [-0.4f32, 0.4] { r.cyl(vec3(px + s, 0., pz), 0.05, 1.4, Color::new(0.95, 0.95, 0.9, 1.)) } for k in 0..9 { let a = k as f32 / 8. * PI; r.sphere(vec3(px - a.cos() * 0.4, 1.4 + a.sin() * 0.3, pz), Vec3::splat(0.09), [PINK, WHITE, YELLOW][k % 3]) } }
        _ => { for k in 0..2 { let bx = px + k as f32; r.cube(vec3(bx, 0.06, pz), vec3(0.5, 0.06, 0.48), Color::new(0.65, 0.62, 0.58, 1.)); for zz in [pz - 0.45, pz + 0.45] { r.cube(vec3(bx, 0.35, zz), vec3(0.5, 0.05, 0.04), Color::new(0.55, 0.52, 0.5, 1.)); r.cube(vec3(bx - 0.4, 0.2, zz), vec3(0.05, 0.2, 0.05), Color::new(0.55, 0.52, 0.5, 1.)) } } }
    }
}
fn palm(r: &mut R, x: f32, z: f32, t: f32) {
    let sway = (t * 0.8 + x).sin() * 0.04;
    for k in 0..6 { let y = k as f32 * 0.3; r.cyl(vec3(x + k as f32 * 0.06 + sway * k as f32, y, z), 0.11 - k as f32 * 0.008, 0.3, Color::new(0.6, 0.45, 0.3, 1.)) }
    let top = vec3(x + 0.36 + sway * 6., 1.8, z);
    for k in 0..6 { let a = k as f32 / 6. * 2. * PI; r.with(Mat4::from_translation(top) * Mat4::from_rotation_y(a) * Mat4::from_rotation_z(-0.35), |r| r.cube(vec3(0.42, 0., 0.), vec3(0.42, 0.02, 0.1), Color::new(0.25, 0.6, 0.3, 1.))) }
    for k in 0..3 { let a = k as f32 * 2.1; r.sphere(top + vec3(a.cos() * 0.12, -0.12, a.sin() * 0.12), Vec3::splat(0.08), Color::new(0.45, 0.32, 0.2, 1.)) }
}
/// Gullrock Islet: own tropical terrain, palms, tide pool, dock, Captain Pell, Sea Glass stall, collectibles, both players.
fn islet_scene(r: &mut R, g: &Game, lt: &Light, labels: &mut Vec<(Vec3, String, Color)>) {
    let t = g.t; let (sand, grass) = (Color::new(0.97, 0.9, 0.7, 1.), Color::new(0.42, 0.78, 0.38, 1.));
    let m = &g.imap; let water = |c: u8| matches!(c, SEA | RIVER | BRIDGE);
    for z in -10..IH as i32 + 8 { for x in -12..IW as i32 + 12 { let (fx, fz) = (x as f32, z as f32); if (fx - g.me.cam.x).powi(2) + (fz - g.me.cam.y).powi(2) > 480. { continue }
        let c = tile_at_i(m, x, z);
        if water(c) { let sv = (t * 1.4 + (x + z) as f32 * 0.6).sin() * 0.03; let wc = if c == RIVER { Color::new(0.25 + sv, 0.7 + sv, 0.72 + sv, 1.) } else { Color::new(0.2 + sv, 0.62 + sv, 0.82 + sv, 1.) };
            r.quad(vec3(fx, -0.2, fz), vec3(fx + 1., -0.2, fz), vec3(fx + 1., -0.2, fz + 1.), vec3(fx, -0.2, fz + 1.), Vec3::Y, wc);
            for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] { if water(tile_at_i(m, x + dx, z + dz)) { continue } let (a, b, n) = match (dx, dz) { (0, -1) => (vec3(fx, 0., fz), vec3(fx + 1., 0., fz), Vec3::Z), (0, 1) => (vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), -Vec3::Z), (-1, 0) => (vec3(fx, 0., fz + 1.), vec3(fx, 0., fz), Vec3::X), _ => (vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), -Vec3::X) }; r.quad(a, b, b - Vec3::Y * 0.2, a - Vec3::Y * 0.2, n, sand) } }
        else { r.quad(vec3(fx, 0., fz), vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), Vec3::Y, sh(if c == GRASS || c == TREE || c == ROCK { grass } else { sand }, 1. + ((hash(x, z) % 5) as f32 - 2.) * 0.015)) }
        let (px, pz) = (fx + 0.5, fz + 0.5);
        match c { TREE => palm(r, px, pz, t), ROCK => r.sphere(vec3(px, 0.12, pz), vec3(0.4, 0.28, 0.34), Color::new(0.55, 0.5, 0.48, 1.)), BRIDGE => { r.cube(vec3(px, 0.05, pz), vec3(0.5, 0.05, 0.5), WOOD); r.cube(vec3(px - 0.45, -0.05, pz), vec3(0.05, 0.25, 0.05), sh(WOOD, 0.7)) } _ => {} }
    } }
    let shc = Color::new(0., 0., 0., 0.22);
    r.disc(vec3(PELL_I.0, 0.02, PELL_I.1), vec2(0.3, 0.24), shc); let talking = matches!(g.me.ui, Ui::Talk { npc: None, .. }); resident(r, 12, vec3(PELL_I.0, 0., PELL_I.1), yaw_of(g.me.ipos - vec2(PELL_I.0, PELL_I.1)), 0., false, t, talking, 0.);
    r.cube(vec3(7.5 + 1.2, -0.05, 13.3), vec3(0.35, 0.12, 0.8), Color::new(0.85, 0.85, 0.9, 1.));
    let st = tc(ISTALL); r.cube(vec3(st.x, 0.4, st.y), vec3(0.45, 0.4, 0.35), Color::new(0.6, 0.48, 0.35, 1.)); for k in 0..4 { let a = k as f32 * 1.6; r.sphere(vec3(st.x - 0.25 + k as f32 * 0.17, 0.92 + (t * 2. + a).sin() * 0.02, st.y), Vec3::splat(0.08), Color::new(0.45, 0.85 - k as f32 * 0.08, 0.78, 0.9)) }
    r.cube(vec3(st.x, 1.25, st.y + 0.25), vec3(0.5, 0.03, 0.35), Color::new(0.3, 0.7, 0.75, 1.));
    for it in &g.w.iground { let c = rgb(def(it.def).col); match it.def { 82 => r.with(Mat4::from_translation(vec3(it.x, 0.06, it.z)) * Mat4::from_rotation_y(it.x * 3.), |r| r.cube(Vec3::ZERO, vec3(0.28, 0.05, 0.05), c)), 75 => { r.sphere(vec3(it.x, 0.08, it.z), vec3(0.14, 0.07, 0.11), c); for s in [-1f32, 1.] { r.sphere(vec3(it.x + s * 0.15, 0.08, it.z + 0.08), Vec3::splat(0.05), c) } } _ => { r.sphere(vec3(it.x, 0.05, it.z), vec3(0.12, 0.06, 0.12), c); r.cone(vec3(it.x, 0.05, it.z), 0.07, 0.12, sh(c, 0.9)) } } }
    for b in &g.ibubbles { for k in 0..3 { let ph = (t * 1.5 + k as f32 * 2.1) % 1.; r.sphere(vec3(b.0.x + (k as f32 - 1.) * 0.12, -0.18 + ph * 0.05, b.0.y + ph * 0.1), Vec3::splat(0.04 + ph * 0.03), Color::new(0.95, 0.98, 1., 1.)) } }
    for &(p, a, k) in &g.fx { if k == 0 { let sz = a / 0.8; for i in 0..10 { let an = i as f32 / 10. * 2. * PI; let rr = 0.15 + sz * 0.6; r.sphere(p + vec3(an.cos() * rr, 0.02, an.sin() * rr), vec3(0.05, 0.015, 0.05), Color::new(0.92, 0.97, 1., 1. - sz)) } } }
    { let bp = tc(ACT_BOARD); r.cube(vec3(bp.x, 0.45, bp.y), vec3(0.05, 0.45, 0.05), WOOD); r.cube(vec3(bp.x, 0.95, bp.y), vec3(0.4, 0.25, 0.04), Color::new(0.3, 0.6, 0.75, 1.));
      if g.me.ipos.distance(bp) < 3. { labels.push((vec3(bp.x, 1.5, bp.y), "Tidekeeper's board".into(), Color::new(0.7, 1., 0.95, 1.))) } }
    if let Some(a) = &g.act { match a.kind {
        0 => { for k in 0..12 { let an = k as f32 / 12. * 2. * PI; r.sphere(vec3(PEN.0 + an.cos() * PEN.2, 0.08, PEN.1 + an.sin() * PEN.2), vec3(0.14, 0.1, 0.12), Color::new(0.6, 0.58, 0.55, 1.)) }
            for (k, c) in a.ents.iter().enumerate() { let pen = Game::in_pen(*c); r.sphere(vec3(c.x, 0.12, c.y), vec3(0.22, 0.11, 0.17), Color::new(0.2, 0.45, 0.9, 1.)); for s in [-1f32, 1.] { r.sphere(vec3(c.x + s * 0.24, 0.14, c.y - 0.1), Vec3::splat(0.075), Color::new(0.35, 0.6, 1., 1.)) }
                if !pen { r.cone(vec3(c.x, 0.75 + (t * 4. + k as f32).sin() * 0.06, c.y), 0.09, -0.18, Color::new(1., 0.85, 0.2, 1.)) } } }
        1 => { let b = a.ents[0]; let pulse = (t * 6.).sin() * 0.5 + 0.5; r.cyl(vec3(b.x, 0., b.y), 0.12, 0.9, Color::new(1., 0.85, 0.3, 1.)); r.sphere(vec3(b.x, 1.0, b.y), Vec3::splat(0.16 + pulse * 0.06), Color::new(1., 0.95, 0.5 + pulse * 0.4, 1.)) }
        _ => for (k, p) in a.ents.iter().enumerate() { if k < a.next { continue } let c = if k == a.next { Color::new(1., 0.8, 0.2, 1.) } else { Color::new(0.95, 0.95, 0.95, 1.) };
            for j in 0..14 { let an = j as f32 / 14. * 2. * PI; r.sphere(vec3(p.x + an.cos() * 0.55, 0.15 + an.sin() * 0.55, p.y), Vec3::splat(0.07), c) }
            if k == a.next || g.me.ipos.distance(*p) < 6. { labels.push((vec3(p.x, 1.1, p.y), format!("{}", k + 1), c)) } },
    } }
    for b in &g.ibugs { let c = rgb(def(b.0).col); let fl = (t * 20. + b.2).sin().abs(); let y = 0.35 + (t * 3. + b.2).sin() * 0.1; for s in [-1f32, 1.] { r.sphere(vec3(b.1.x + s * 0.07, y + fl * 0.04, b.1.y), vec3(0.07, 0.01 + fl * 0.03, 0.05), c) } r.sphere(vec3(b.1.x, y, b.1.y), Vec3::splat(0.035), DARK) }
    let swim = g.swimming(); let sy = if swim { -0.55 } else { 0. }; if !swim { r.disc(vec3(g.me.ipos.x, 0.02, g.me.ipos.y), vec2(0.32, 0.26), shc) }
    human(r, g.me.id, (g.w.p.shirt, g.w.p.hat, g.w.p.hair), vec3(g.me.ipos.x, sy, g.me.ipos.y), yaw_of(g.me.face), g.me.anim, g.me.spd, 0, None, t, g.me.swing);
    others(r, g, t);
    if g.me.pos.distance(Vec2::ZERO) >= 0. && g.me.ipos.distance(st) < 3.5 { labels.push((vec3(st.x, 1.7, st.y), "Sea Glass stall".into(), Color::new(0.7, 1., 0.95, 1.))) }
    if g.me.ui == Ui::Play && g.me.trans.is_none() { if let Some(tg) = g.target() { indicator(r, g.tg_pos(tg), t) } }
    let _ = lt;
}
fn tree(r: &mut R, p: Vec2, fruit: Option<Color>, shake: f32, t: f32, fol: Color) {
    // per-tree variety from its position (shape only; the tree's tile is unchanged)
    let hv = hash((p.x * 2.) as i32, (p.y * 2.) as i32); let v = |k: u32| ((hv >> k) % 100) as f32 / 100.;
    let bark = Color::new(0.5, 0.33, 0.2, 1.);
    r.cyl(vec3(p.x, 0., p.y), 0.17, 0.12, sh(bark, 0.9)); r.cyl(vec3(p.x, 0., p.y), 0.13, 0.85, bark);
    r.seg(vec3(p.x, 0.7, p.y), vec3(p.x + 0.28 * (v(3) - 0.5).signum(), 1.0, p.y + 0.08), 0.04, bark);
    let sway = (t * 1.1 + p.x * 0.7 + p.y * 0.3).sin() * 0.025;
    let o = vec3(p.x + (t * 45.).sin() * 0.08 * shake + sway, 0., p.y);
    let fol = Color::new(fol.r * (0.95 + v(5) * 0.1), fol.g * (0.95 + v(7) * 0.1), fol.b, 1.);
    r.sphere(o + vec3(0., 1.08, 0.05), vec3(0.6, 0.38, 0.6), sh(fol, 0.82)); // shaded underside
    r.sphere(o + vec3(0., 1.2, 0.), vec3(0.65, 0.55, 0.65), fol); r.sphere(o + vec3(0.3, 1.5, 0.1), Vec3::splat(0.38), sh(fol, 1.08)); r.sphere(o + vec3(-0.28, 1.42, -0.12), Vec3::splat(0.4), sh(fol, 0.94));
    r.sphere(o + vec3(0.05 + (v(9) - 0.5) * 0.2, 1.68, -0.05), Vec3::splat(0.3), sh(fol, 1.14)); r.sphere(o + vec3(-0.15, 1.25, 0.42), Vec3::splat(0.3), sh(fol, 1.03));
    if let Some(fc) = fruit { for q in [vec3(0.32, 1.05, 0.48), vec3(-0.38, 1.12, 0.42), vec3(0.05, 1.38, 0.55), vec3(0.52, 1.28, 0.2)] { r.sphere(o + q, Vec3::splat(0.09), fc) } }
}
fn pine(r: &mut R, x: f32, z: f32, s: f32) {
    let hv = hash((x * 3.) as i32, (z * 3.) as i32); let tone = ((hv % 100) as f32 / 100. - 0.5) * 0.06; let tw = ((hv >> 7) % 628) as f32 / 100.;
    r.cyl(vec3(x, 0., z), 0.1 * s, 0.5 * s, Color::new(0.45, 0.3, 0.2, 1.));
    for k in 0..3 { let kf = k as f32; let y = (0.4 + kf * 0.38) * s; let lean = (tw + kf).sin() * 0.02 * kf;
        r.cone(vec3(x + lean, y - 0.04 * s, z), (0.66 - kf * 0.15) * s, 0.12 * s, Color::new(0.11, 0.3 + tone, 0.2, 1.)); // dark skirt under each tier
        r.cone(vec3(x + lean, y, z), (0.62 - kf * 0.15) * s, 0.65 * s, Color::new(0.16 + tone * 0.5, 0.42 + kf * 0.05 + tone, 0.26, 1.)) } }
fn indicator(r: &mut R, p: Vec3, t: f32) { let u = r.unlit; r.unlit = true; let y = p.y + (t * 4.).sin() * 0.06; r.with(Mat4::from_translation(vec3(p.x, y, p.z)) * Mat4::from_rotation_x(PI), |r| r.cone(Vec3::ZERO, 0.1, 0.18, Color::new(1., 0.85, 0.2, 1.))); r.unlit = u }

// ---- scenes ----
thread_local! { static VIEW: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((1280., 720.)) }; }
fn vw() -> f32 { VIEW.with(|v| v.get().0) }
fn vh() -> f32 { VIEW.with(|v| v.get().1) }
/// Draw every player's view: full screen for one player, side-by-side split-screen for two.
pub fn draw(g: &mut Game, now: NaiveDateTime, dbg: Option<&str>) {
    let ids: Vec<usize> = g.ids().into_iter().filter(|&i| Some(i) != g.remote).collect(); let act = g.me.id; let n = ids.len() as f32; let (sw, sh) = (screen_width(), screen_height());
    if n > 1. { clear_background(BLACK) }
    for (k, &id) in ids.iter().enumerate() { g.activate(id); VIEW.with(|v| v.set((sw / n, sh))); view(g, now, dbg, (n > 1.).then(|| (k as f32 * sw / n, sw / n))) }
    g.activate(act); VIEW.with(|v| v.set((sw, sh)));
    if n > 1. { set_default_camera(); draw_rectangle(sw / 2. - 2., 0., 4., sh, BLACK) }
}
fn hud_cam(vp: Option<(f32, f32)>) { match vp { Some((x, w)) => set_camera(&Camera2D { target: vec2(w / 2., vh() / 2.), zoom: vec2(2. / w, 2. / vh()), viewport: Some((x as i32, 0, w as i32, vh() as i32)), ..Default::default() }), None => set_default_camera() } }
fn view(g: &Game, now: NaiveDateTime, dbg: Option<&str>, vp: Option<(f32, f32)>) {
    let hf = now.hour() as f32 + now.minute() as f32 / 60.; let wx = weather(g.w.day, now.month()); let rain = wx != Wx::Clear; let lt = light(hf, rain);
    let (cam, curve, c0) = match g.me.scene {
        Scene::Town | Scene::Islet => (Camera3D { position: vec3(g.me.cam.x, 7.4, g.me.cam.y + 7.6), target: vec3(g.me.cam.x, 0.5, g.me.cam.y - 0.8), up: Vec3::Y, fovy: 40f32.to_radians(), ..Default::default() }, g.curve, g.me.cam),
        Scene::Room(b) => { let (w, d) = g.rdims(b); let s = w.max(d) as f32 / 4.; (Camera3D { position: vec3(w as f32 / 2., 5.4 * s, d as f32 + 3.4 * s), target: vec3(w as f32 / 2., 0.3, d as f32 / 2. + 0.1), up: Vec3::Y, fovy: 45f32.to_radians(), ..Default::default() }, 0., Vec2::ZERO) }
    };
    let bg = if g.me.scene != Scene::Room(0) && !matches!(g.me.scene, Scene::Room(_)) { lt.sky } else { Color::new(0.08, 0.06, 0.06, 1.) };
    let cam = Camera3D { viewport: vp.map(|(x, w)| (x as i32, 0, w as i32, vh() as i32)), aspect: Some(vw() / vh()), ..cam };
    if vp.is_some() { hud_cam(vp); draw_rectangle(0., 0., vw(), vh(), bg) } else { clear_background(bg) }
    { hud_cam(vp); let indoor = matches!(g.me.scene, Scene::Room(_)); let (w, h) = (vw(), vh());
        // outdoors: deeper zenith fading to the horizon (fog) colour; indoors: warm dim glow behind the room instead of flat black
        let top = if indoor { Color::new(0.05, 0.04, 0.05, 1.) } else { Color::new(bg.r * 0.62, bg.g * 0.72, (bg.b * 0.9 + 0.08).min(1.), 1.) };
        let bot = if indoor { Color::new(0.2, 0.15, 0.12, 1.) } else { bg }; let n = 24;
        for k in 0..n { let (a, b) = (k as f32 / n as f32, (k + 1) as f32 / n as f32); let f = |t: f32| if indoor { t * t } else { (t / 0.55).min(1.).sqrt() }; let t = f((a + b) / 2.);
            draw_rectangle(0., a * h, w, (b - a) * h + 1., Color::new(top.r + (bot.r - top.r) * t, top.g + (bot.g - top.g) * t, top.b + (bot.b - top.b) * t, 1.)) } }
    set_camera(&cam);
    let mut r = R::new(curve, c0, &lt, if matches!(g.me.scene, Scene::Room(_)) { (100., 200.) } else { (12., 21.) });
    let mut labels: Vec<(Vec3, String, Color)> = vec![];
    match g.me.scene { Scene::Islet => islet_scene(&mut r, g, &lt, &mut labels), Scene::Town => town(&mut r, g, &lt, now, &mut labels), Scene::Room(b) => room(&mut r, g, b, &lt, &mut labels) }
    r.flush(); hud_cam(vp);
    let m = cam.matrix();
    let menu = matches!(g.me.ui, Ui::Journal | Ui::Help | Ui::Inv(..) | Ui::Mail(_) | Ui::Catalog(_)); for (p, s, c) in labels { if menu { break } let d2 = (p.x - c0.x).powi(2) + (p.z - c0.y).powi(2); let q = m * vec4(p.x, p.y - curve * d2, p.z, 1.); if q.w <= 0. { continue }
        let (x, y) = ((q.x / q.w + 1.) * 0.5 * vw(), (1. - q.y / q.w) * 0.5 * vh()); let tw = measure_text(&s, None, 20, 1.).width;
        draw_rectangle(x - tw / 2. - 5., y - 17., tw + 10., 23., Color::new(0., 0., 0., 0.55)); draw_text(&s, x - tw / 2., y, 20., c); }
    if !matches!(g.me.scene, Scene::Room(_)) { let (mo, w, h, t) = (now.month(), vw(), vh(), g.t);
        match mo { 3..=5 => for i in 0..40 { let x = ((i * 131) as f32 + (t * 0.6 + i as f32).sin() * 40. + t * 25.) % w; let y = ((i * 71) as f32 + t * 35.) % h; draw_circle(x, y, 3., Color::new(1., 0.75, 0.85, 0.85)); draw_circle(x + 2., y + 1., 2., Color::new(1., 0.85, 0.9, 0.85)) },
            9..=11 => for i in 0..30 { let x = ((i * 157) as f32 + (t * 0.9 + i as f32 * 1.3).sin() * 60. + t * 18.) % w; let y = ((i * 89) as f32 + t * 42.) % h; let c = [Color::new(0.9, 0.45, 0.15, 0.9), Color::new(0.8, 0.25, 0.15, 0.9), Color::new(0.95, 0.7, 0.2, 0.9)][i % 3];
                draw_rectangle_ex(x, y, 9., 5., DrawRectangleParams { rotation: t * 2. + i as f32, offset: vec2(0.5, 0.5), color: c }) },
            6..=8 if dark(now) > 0.5 => for i in 0..35 { let x = ((i * 97) as f32 + (t * 0.5 + i as f32).sin() * 30.) % w; let y = h * 0.35 + ((i * 53) as f32 + (t * 0.4 + i as f32).cos() * 25.) % (h * 0.6); let a = ((t * 3. + i as f32).sin() * 0.5 + 0.5) * 0.9; draw_circle(x, y, 2.5, Color::new(0.9, 1., 0.4, a)); draw_circle(x, y, 6., Color::new(0.9, 1., 0.4, a * 0.2)) },
            _ => {} } }
    if wx == Wx::Snow && g.me.scene == Scene::Town { for i in 0..160 { let x = ((i * 97) as f32 + (g.t * 0.7 + i as f32).sin() * 25. + g.t * 15.) % vw(); let y = ((i * 53) as f32 + g.t * 70.) % vh(); draw_circle(x, y, 2. + (i % 3) as f32, Color::new(1., 1., 1., 0.85)) } }
    else if rain && g.me.scene == Scene::Town { for i in 0..140 { let x = ((i * 97) as f32 + g.t * 40.) % vw(); let y = ((i * 53) as f32 + g.t * 520.) % vh(); draw_line(x, y, x - 3., y + 12., 1., Color::new(0.8, 0.85, 1., 0.45)) } }
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
        if water(c) {
            // smooth rolling ripples per corner; the sea deepens away from the beach, shallows near banks turn turquoise
            let wc = |dx: i32, dz: i32| { let (xc, zc) = ((x + dx) as f32, (z + dz) as f32);
                let s = ((t * 1.3 + xc * 0.9 + zc * 0.6).sin() * 0.5 + (t * 0.8 - xc * 0.45 + zc * 1.1).sin() * 0.5) * 0.04;
                let shore = [(-1, -1), (0, -1), (-1, 0), (0, 0)].iter().any(|&(ox, oz)| !water(ext(x + dx + ox, z + dz + oz))) as i32 as f32;
                let (mut cr, mut cg, mut cb) = if c == SEA { let d = ((zc - 26.) / 7.).clamp(0., 1.); (0.24 - 0.1 * d, 0.56 - 0.14 * d, 0.8 - 0.06 * d) } else { (0.22, 0.52, 0.8) };
                cr += shore * 0.07; cg += shore * 0.09; cb += shore * 0.03; Color::new(cr + s, cg + s, cb + s, 1.) };
            r.quad4([vec3(fx, -0.2, fz), vec3(fx + 1., -0.2, fz), vec3(fx + 1., -0.2, fz + 1.), vec3(fx, -0.2, fz + 1.)], Vec3::Y, [wc(0, 0), wc(1, 0), wc(1, 1), wc(0, 1)]);
            { let u = r.unlit; r.unlit = true; let day = 1. - lt.night;
                // sun glints: brief bright flecks that drift with the current
                if day > 0.2 { for k in 0..2u32 { let hv = hash(x * 7 + k as i32, z * 13 - k as i32); let ph = (hv % 628) as f32 / 100.; let v = (t * (1.4 + k as f32 * 0.5) + ph).sin();
                    if v > 0.8 { let a = (v - 0.8) / 0.2 * 0.75 * day; let gx = fx + 0.15 + ((hv >> 3) % 70) as f32 / 100.; let gz = fz + 0.15 + ((hv >> 9) % 70) as f32 / 100. + (t * 0.15).fract() * 0.1;
                        r.quad(vec3(gx - 0.07, -0.19, gz - 0.015), vec3(gx + 0.07, -0.19, gz - 0.015), vec3(gx + 0.07, -0.19, gz + 0.015), vec3(gx - 0.07, -0.19, gz + 0.015), Vec3::Y, Color::new(1., 1., 0.95, a)) } } }
                // lapping foam along every bank edge
                let fa = 0.32 + (t * 1.7 + (x * 3 + z * 5) as f32 * 0.4).sin() * 0.12; let fw = 0.07 + (t * 1.1 + (x + z) as f32).sin().abs() * 0.05; let fcol = Color::new(1., 1., 1., fa);
                for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] { if water(ext(x + dx, z + dz)) || ext(x + dx, z + dz) == BRIDGE { continue } let y = -0.195;
                    match (dx, dz) { (0, -1) => r.quad(vec3(fx, y, fz), vec3(fx + 1., y, fz), vec3(fx + 1., y, fz + fw), vec3(fx, y, fz + fw), Vec3::Y, fcol),
                        (0, 1) => r.quad(vec3(fx, y, fz + 1. - fw), vec3(fx + 1., y, fz + 1. - fw), vec3(fx + 1., y, fz + 1.), vec3(fx, y, fz + 1.), Vec3::Y, fcol),
                        (-1, 0) => r.quad(vec3(fx, y, fz), vec3(fx + fw, y, fz), vec3(fx + fw, y, fz + 1.), vec3(fx, y, fz + 1.), Vec3::Y, fcol),
                        _ => r.quad(vec3(fx + 1. - fw, y, fz), vec3(fx + 1., y, fz), vec3(fx + 1., y, fz + 1.), vec3(fx + 1. - fw, y, fz + 1.), Vec3::Y, fcol) } }
                r.unlit = u }
            for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] { let nc = ext(x + dx, z + dz); if water(nc) { continue } let bc = if nc == SAND { sand } else { soil };
                let (a, b, n) = match (dx, dz) { (0, -1) => (vec3(fx, 0., fz), vec3(fx + 1., 0., fz), Vec3::Z), (0, 1) => (vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), -Vec3::Z), (-1, 0) => (vec3(fx, 0., fz + 1.), vec3(fx, 0., fz), Vec3::X), _ => (vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), -Vec3::X) };
                r.quad(a, b, b - Vec3::Y * 0.2, a - Vec3::Y * 0.2, n, bc) } }
        else { let base = |c: u8| match c { PATH => Color::new(0.77, 0.65, 0.47, 1.), SAND => sand, b'f' => sh(grass, 0.8), RIVER | SEA | BRIDGE => sand, _ => grass }; let col = base(c); let amt = if c == PATH || c == SAND { 0.45 } else { 1. };
            // soft borders: each corner leans toward the average of the four tiles that share it
            let cc = |dx: i32, dz: i32| { let n = 0.5 + (gnoise(x + dx, z + dz) - 0.5) * amt; let (cx, cz) = (x + dx, z + dz);
                let mut a = [0f32; 3]; for (ox, oz) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] { let b = if water(ext(cx + ox, cz + oz)) { col } else { base(ext(cx + ox, cz + oz)) }; a[0] += b.r / 4.; a[1] += b.g / 4.; a[2] += b.b / 4. }
                gtint(Color::new(col.r * 0.55 + a[0] * 0.45, col.g * 0.55 + a[1] * 0.45, col.b * 0.55 + a[2] * 0.45, 1.), n) };
            r.quad4([vec3(fx, 0., fz), vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.)], Vec3::Y, [cc(0, 0), cc(1, 0), cc(1, 1), cc(0, 1)]) } } }
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
            PINE if (px - g.me.pos.x).abs() < 1.1 && pz > g.me.pos.y + 0.2 && pz < g.me.pos.y + 2.6 => {} // drawn translucent later (occlusion fade)
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
    for bi in 0..BLDS.len() { let b = &BLDS[bi]; if near(b.x as f32 + b.w as f32 / 2., b.z as f32 + b.d as f32 / 2.) && !fade_b.contains(&bi) { let b = &BLDS[bi]; let paint = if let BK::Home(o) = b.kind { g.owner(o).paint.map(|k| (PAINTS[k as usize % PAINTS.len()].1, PAINTS[k as usize % PAINTS.len()].2)) } else { None }; building(r, bi, lt.night, paint) }
        if g.me.pos.distance(door_stand(bi)) < 3.5 { let p = door_face(bi); labels.push((vec3(p.x, 2.7, p.y), g.bname(bi), WHITE)) } }
    for z in 0..H as i32 { for x in 0..W as i32 { if !near(x as f32 + 0.5, z as f32 + 0.5) || !matches!(tile_at(&g.map, x, z), GRASS | FLOWER) { continue }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] { if tile_at(&g.map, x + dx, z + dz) != RIVER { continue }
            let (cx, cz) = (x as f32 + 0.5 + dx as f32 * 0.45, z as f32 + 0.5 + dz as f32 * 0.45); let along = if dx != 0 { vec3(0., 0., 0.5) } else { vec3(0.5, 0., 0.) }; let c = vec3(cx, 0., cz); let wood = Color::new(0.62, 0.45, 0.28, 1.);
            for e in [-1f32, 1.] { r.cube(c + along * e * 0.92 + vec3(0., 0.2, 0.), vec3(0.035, 0.2, 0.035), sh(wood, 0.85)) }
            for y in [0.16f32, 0.3] { r.cube(c + vec3(0., y, 0.), if dx != 0 { vec3(0.015, 0.02, 0.48) } else { vec3(0.48, 0.02, 0.015) }, wood) } } } }
    // chimney smoke from homes in the morning, evening and cold months
    let smoky = matches!(g.hr, 6..=9 | 17..=23) || matches!(now.month(), 11 | 12 | 1 | 2); if smoky { let ua = r.alpha; r.alpha = 0.5;
        for b in BLDS.iter().filter(|b| matches!(b.kind, BK::Home(_) | BK::Res(_))) { let (cx, cz) = (b.x as f32 + b.w as f32 / 2. + b.w as f32 / 2. - 0.6, b.z as f32 + b.d as f32 / 2. - 0.25); if !near(cx, cz) { continue }
            for k in 0..4 { let ph = (t * 0.35 + k as f32 * 0.25 + b.x as f32 * 0.13) % 1.; r.sphere(vec3(cx + ph * 0.5 + (t + k as f32).sin() * 0.05, 2.45 + ph * 1.3, cz - ph * 0.2), Vec3::splat(0.1 + ph * 0.16), Color::new(0.9, 0.9, 0.92, 1.)) } } r.alpha = ua }
    // a small flock of birds circling over town in fair daytime weather
    if (7..18).contains(&g.hr) && weather(g.w.day, now.month()) == Wx::Clear { let c0 = g.me.cam + vec2((t * 0.07).sin() * 5., (t * 0.05).cos() * 2. - 9.);
        for k in 0..5 { let a = t * 0.4 + k as f32 * 0.5; let p = vec3(c0.x + a.cos() * (2.5 + k as f32 * 0.3), 2.6 + (k % 2) as f32 * 0.25, c0.y + a.sin() * 1.5); let fl = (t * 8. + k as f32).sin() * 0.5; let yaw = -a;
            r.with(Mat4::from_scale_rotation_translation(Vec3::splat(0.6), Quat::from_rotation_y(yaw), p), |r| { r.sphere(Vec3::ZERO, vec3(0.05, 0.04, 0.09), Color::new(0.3, 0.3, 0.35, 1.)); for sgn in [-1f32, 1.] { r.with(Mat4::from_rotation_z(sgn * fl), |r| r.cube(vec3(sgn * 0.12, 0., 0.), vec3(0.11, 0.008, 0.04), Color::new(0.35, 0.35, 0.4, 1.))) } }) } }
    { let v = &VISITORS[g.visitor()]; let tc0 = vec3(33., 0., 20.); let col = rgb(v.tent);
        r.with(Mat4::from_translation(tc0), |r| { r.cube(vec3(0., 0.01, 0.), vec3(1.0, 0.01, 0.95), Color::new(0.55, 0.45, 0.3, 1.)); for s in [-1f32, 1.] { r.with(Mat4::from_translation(vec3(s * 0.45, 0.55, 0.)) * Mat4::from_rotation_z(s * 0.8), |r| r.cube(Vec3::ZERO, vec3(0.62, 0.03, 0.9), sh(col, if s > 0. { 0.85 } else { 1. }))) }
            r.cube(vec3(0., 0.45, 0.92), vec3(0.22, 0.4, 0.01), Color::new(0.2, 0.15, 0.12, 1.)); r.seg(vec3(0., 0., 0.95), vec3(0., 1.15, 0.95), 0.02, WOOD); r.cube(vec3(0.2, 1.12, 0.95), vec3(0.15, 0.08, 0.01), YELLOW) });
        let f = tc(CAMP_FIRE); for k in 0..6 { let a = k as f32 * 1.047; r.sphere(vec3(f.x + a.cos() * 0.3, 0.06, f.y + a.sin() * 0.3), vec3(0.1, 0.07, 0.09), Color::new(0.5, 0.48, 0.45, 1.)) }
        for k in 0..2 { r.with(Mat4::from_translation(vec3(f.x, 0.06, f.y)) * Mat4::from_rotation_y(k as f32 * 1.57), |r| r.cube(Vec3::ZERO, vec3(0.25, 0.04, 0.04), WOOD)) }
        let fl = (g.t * 9.).sin() * 0.04; let u = r.unlit; r.unlit = true; r.cone(vec3(f.x, 0.08, f.y), 0.15, 0.32 + fl, Color::new(1., 0.55, 0.15, 1.)); r.cone(vec3(f.x, 0.08, f.y), 0.08, 0.22 - fl, Color::new(1., 0.9, 0.4, 1.)); r.unlit = u;
        let cr = tc(CAMP_CRATE); r.cube(vec3(cr.x, 0.3, cr.y), vec3(0.32, 0.3, 0.3), Color::new(0.62, 0.45, 0.28, 1.)); for (k, &d) in v.goods.iter().enumerate() { if !g.w.camp_sold.contains(&d) { r.cube(vec3(cr.x - 0.18 + k as f32 * 0.18, 0.68, cr.y), Vec3::splat(0.07), rgb(def(d).col)) } }
        let sg = tc(CAMP_SIGN); r.cube(vec3(sg.x, 0.45, sg.y), vec3(0.04, 0.45, 0.04), WOOD); r.cube(vec3(sg.x, 0.85, sg.y + 0.05), vec3(0.32, 0.18, 0.03), Color::new(0.9, 0.82, 0.6, 1.));
        if g.camp_open() { let sp = vec3(CAMP_SPOT.0, 0., CAMP_SPOT.1); let talking = g.me.scene == Scene::Town && matches!(g.me.ui, Ui::Camp(_) | Ui::Talk { npc: None, .. }) && g.me.pos.distance(vec2(sp.x, sp.z)) < 2.5;
            r.disc(sp + vec3(0., 0.02, 0.), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.22)); resident(r, 14 + g.visitor(), sp, yaw_of(g.me.pos - vec2(sp.x, sp.z)), 0., false, g.t, talking, 0.) }
        if g.me.pos.distance(vec2(32.5, 20.5)) < 5. { labels.push((vec3(sg.x, 1.35, sg.y), format!("Campground: {} the {}", v.name, v.species), Color::new(1., 0.9, 0.7, 1.))) } }
    if g.art_open() { let c = tc(STALL); r.cube(vec3(c.x, 0.4, c.y), vec3(0.5, 0.4, 0.35), Color::new(0.35, 0.3, 0.45, 1.)); r.cube(vec3(c.x, 1.3, c.y), vec3(0.6, 0.04, 0.45), Color::new(0.85, 0.3, 0.4, 1.));
        for x in [-0.55f32, 0.55] { r.cube(vec3(c.x + x, 0.65, c.y + 0.4), vec3(0.03, 0.65, 0.03), WOOD) } painting(r, art_of_week(g.w.day), vec3(c.x, 1.0, c.y + 0.37), 0.5, 0.36);
        if g.me.pos.distance(c) < 4. { labels.push((vec3(c.x, 1.7, c.y), "Saturday art stall".into(), Color::new(1., 0.85, 0.9, 1.))) } }
    for pl in &g.w.garden { let p = tc((pl.x, pl.z)); if !near(p.x, p.y) { continue } let col = rgb(def(FLOWERS[pl.kind as usize]).col); let stem = Color::new(0.3, 0.6, 0.3, 1.);
        r.disc(vec3(p.x, 0.012, p.y), vec2(0.3, 0.3), if pl.watered == g.w.day { Color::new(0.35, 0.25, 0.18, 1.) } else { Color::new(0.5, 0.38, 0.26, 1.) });
        match pl.stage { 0 => { for k in 0..2 { r.with(Mat4::from_translation(vec3(p.x, 0.05, p.y)) * Mat4::from_rotation_y(k as f32 * 1.6), |r| r.cube(vec3(0.05, 0.04, 0.), vec3(0.05, 0.01, 0.025), stem)) } }
            1 => { r.cyl(vec3(p.x, 0., p.y), 0.025, 0.4, stem); r.sphere(vec3(p.x + 0.08, 0.15, p.y), vec3(0.08, 0.02, 0.04), stem); r.sphere(vec3(p.x, 0.46, p.y), vec3(0.08, 0.11, 0.08), sh(col, 0.8)) }
            _ => { r.cyl(vec3(p.x, 0., p.y), 0.025, 0.45, stem); r.sphere(vec3(p.x + 0.09, 0.16, p.y), vec3(0.09, 0.02, 0.045), stem); let u = r.unlit; r.unlit = pl.kind == 4 && lt.night > 0.3; for k in 0..6 { let a = k as f32 * 1.047 + pl.x as f32; r.sphere(vec3(p.x + a.cos() * 0.13, 0.5, p.y + a.sin() * 0.13), vec3(0.1, 0.035, 0.1), col) } r.unlit = u; r.sphere(vec3(p.x, 0.52, p.y), Vec3::splat(0.06), if pl.kind == 4 { WHITE } else { YELLOW }) } } }
    if let Some(sp) = &g.w.surprise { let p = tc((sp.x, sp.z)); let t = g.t;
        match sp.kind { 0 => { r.with(Mat4::from_translation(vec3(p.x, 0.08, p.y)) * Mat4::from_rotation_z(FRAC_PI_2) * Mat4::from_rotation_x(0.3), |r| { r.cyl(Vec3::ZERO, 0.07, 0.22, Color::new(0.45, 0.75, 0.6, 0.85)); r.cyl(vec3(0., 0.22, 0.), 0.03, 0.08, Color::new(0.45, 0.75, 0.6, 0.85)); r.cube(vec3(0., 0.31, 0.), Vec3::splat(0.025), WOOD) }) }
            1 => { r.cube(vec3(p.x, 0.05, p.y), vec3(0.2, 0.04, 0.08), rgb(def(150).col)); r.cube(vec3(p.x + 0.12, 0.05, p.y + 0.1), vec3(0.06, 0.03, 0.12), sh(rgb(def(150).col), 0.85)) }
            _ => { r.cube(vec3(p.x, 0.16, p.y), Vec3::splat(0.16), Color::new(0.4, 0.6, 0.95, 1.)); r.cube(vec3(p.x, 0.16, p.y), vec3(0.17, 0.17, 0.04), YELLOW); r.cube(vec3(p.x, 0.16, p.y), vec3(0.04, 0.17, 0.17), YELLOW);
                let by = 1.3 + (t * 1.2).sin() * 0.05; r.seg(vec3(p.x, 0.33, p.y), vec3(p.x + 0.05, by - 0.15, p.y), 0.008, WHITE); r.sphere(vec3(p.x + 0.05, by, p.y), vec3(0.15, 0.18, 0.15), Color::new(0.95, 0.35, 0.4, 1.)) } }
        let u = r.unlit; r.unlit = true; for k in 0..3 { let a = t * 2. + k as f32 * 2.1; let y = 0.35 + ((t * 1.5 + k as f32) % 1.) * 0.5; r.sphere(vec3(p.x + a.cos() * 0.25, y, p.y + a.sin() * 0.25), Vec3::splat(0.025), Color::new(1., 0.95, 0.6, 1.)) } r.unlit = u }
    if let Some(f) = g.fest() { let c = tc(STALL); let t = g.t; let night = lt.night > 0.4;
        let (main, acc) = [(Color::new(1., 0.72, 0.82, 1.), Color::new(1., 1., 1., 1.)), (Color::new(0.25, 0.35, 0.7, 1.), Color::new(1., 0.85, 0.4, 1.)), (Color::new(0.75, 0.88, 1., 1.), Color::new(0.4, 0.6, 1., 1.)), (Color::new(1., 0.55, 0.25, 1.), Color::new(1., 0.9, 0.3, 1.))][f];
        r.cube(vec3(c.x, 0.4, c.y), vec3(0.5, 0.4, 0.35), WOOD); r.cube(vec3(c.x, 0.82, c.y), vec3(0.55, 0.03, 0.4), main);
        let talking = matches!(g.me.ui, Ui::Talk { npc: None, .. }) && g.me.pos.distance(c) < 2.5; resident(r, 13, vec3(c.x, 0., c.y - 0.6), 0., 0., false, t, talking, 0.);
        let posts = [(-2.5f32, -1.5f32), (2.5, -1.5), (-2.5, 1.5), (2.5, 1.5)];
        for (px, pz) in posts { let p = vec3(c.x + px, 0., c.y + pz); r.cube(p + vec3(0., 0.8, 0.), vec3(0.04, 0.8, 0.04), WOOD); let u = r.unlit; r.unlit = night; r.sphere(p + vec3(0., 1.62, 0.), Vec3::splat(0.14), if night { acc } else { main }); r.unlit = u }
        for k in 0..2 { let (a, b) = if k == 0 { (posts[0], posts[1]) } else { (posts[2], posts[3]) }; for j in 0..9 { let s = j as f32 / 8.; let x = c.x + a.0 + (b.0 - a.0) * s; let y = 1.45 - (s * PI).sin() * 0.25; r.cone(vec3(x, y, c.y + a.1), 0.07, -0.16, if j % 2 == 0 { main } else { acc }) } }
        match f {
            0 => { for (k, (bx, bz)) in [(-1.5f32, 2.6f32), (1.4, 2.8), (0., 3.4)].into_iter().enumerate() { r.cube(vec3(c.x + bx, 0.02, c.y + bz), vec3(0.55, 0.01, 0.45), [Color::new(0.95, 0.4, 0.45, 1.), Color::new(0.4, 0.6, 0.95, 1.), Color::new(0.95, 0.85, 0.4, 1.)][k]); r.cube(vec3(c.x + bx, 0.12, c.y + bz), vec3(0.15, 0.1, 0.1), WOOD) }
                for i in 0..30 { let a = i as f32 * 2.4 + t * 0.3; let rr = 1. + (i % 5) as f32 * 0.6; r.sphere(vec3(c.x + a.cos() * rr, 2.2 - ((t * 0.4 + i as f32 * 0.37) % 2.2), c.y + 1. + a.sin() * rr), vec3(0.04, 0.01, 0.03), main) } }
            1 => { let mut riv: Vec<(i32, i32)> = (0..H as i32).flat_map(|z| (0..W as i32).map(move |x| (x, z))).filter(|&(x, z)| tile_at(&g.map, x, z) == RIVER && (x + z) % 2 == 0).collect(); let cam = g.me.cam; riv.sort_by(|a, b| tc(*a).distance(cam).total_cmp(&tc(*b).distance(cam)));
                for &(x, z) in riv.iter().take(14) { let bob = (t * 1.5 + x as f32).sin() * 0.02; let drift = ((t * 0.15 + (x * 3 + z) as f32 * 0.37) % 1. - 0.5) * 0.6; let p = vec3(x as f32 + 0.5 + drift * 0.3, -0.12 + bob, z as f32 + 0.5 + drift);
                r.cube(p, vec3(0.16, 0.03, 0.08), WHITE); r.cone(p + vec3(0., 0.03, 0.), 0.06, 0.1, Color::new(0.95, 0.95, 0.9, 1.)); let u = r.unlit; r.unlit = night; r.sphere(p + vec3(0., 0.16, 0.), Vec3::splat(0.04), acc); r.unlit = u } }
            2 => { let p = vec3(c.x, 0., c.y + 2.6); r.cyl(p, 0.08, 0.4, WOOD); for k in 0..3 { r.cone(p + vec3(0., 0.35 + k as f32 * 0.45, 0.), 0.7 - k as f32 * 0.18, 0.7, Color::new(0.2, 0.45, 0.35, 1.)) }
                let u = r.unlit; r.unlit = true; for i in 0..14 { let a = i as f32 * 1.9; let h = 0.5 + (i % 7) as f32 * 0.22; let rr = 0.65 - (h - 0.5) * 0.35; r.sphere(p + vec3(a.cos() * rr, h, a.sin() * rr), Vec3::splat(0.05), if (t * 2. + i as f32).sin() > 0. { acc } else { WHITE }) } r.sphere(p + vec3(0., 1.85, 0.), Vec3::splat(0.1), Color::new(1., 0.95, 0.6, 1.)); r.unlit = u }
            _ => for k in 0..3 { let x = c.x - 2. + k as f32 * 2.; r.cube(vec3(x, 0.9, c.y + 2.6), vec3(0.03, 0.9, 0.03), WOOD); r.cube(vec3(x + 0.25, 1.55, c.y + 2.6), vec3(0.25, 0.18, 0.01), [main, acc, Color::new(0.95, 0.35, 0.35, 1.)][k]) },
        }
        if g.me.pos.distance(c) < 6. { labels.push((vec3(c.x, 2.1, c.y), FESTS[f].name.into(), main)) } }
    if fair_active(now.month(), now.day()) { let c = tc(STALL);
        r.cube(vec3(c.x, 0.45, c.y), vec3(0.48, 0.45, 0.4), WOOD); for k in 0..5 { r.cube(vec3(c.x - 0.4 + k as f32 * 0.2, 1.25, c.y + 0.25), vec3(0.1, 0.03, 0.35), if k % 2 == 0 { Color::new(0.95, 0.55, 0.2, 1.) } else { WHITE }) }
        for x in [-0.45, 0.45] { r.cube(vec3(c.x + x, 0.62, c.y + 0.3), vec3(0.03, 0.62, 0.03), WOOD) } for (k, f) in [20u16, 21, 22, 23].iter().enumerate() { r.sphere(vec3(c.x - 0.3 + k as f32 * 0.2, 0.98, c.y + 0.1), Vec3::splat(0.08), rgb(def(*f).col)) }
        let u = r.unlit; for (px, pz) in [(-2.5, -1.5), (2.5, -1.5), (-2.5, 1.5), (2.5, 1.5)] { let p = vec3(c.x + px, 0., c.y + pz); r.unlit = false; r.cube(p + vec3(0., 0.7, 0.), vec3(0.04, 0.7, 0.04), WOOD); r.unlit = true; r.sphere(p + vec3(0., 1.45, 0.), Vec3::splat(0.12), Color::new(1., 0.75 + 0.1 * px.signum(), 0.35, 1.)) } r.unlit = u;
        if g.me.pos.distance(c) < 5. { labels.push((vec3(c.x, 2.0, c.y), "Harvest Fair".into(), Color::new(1., 0.8, 0.4, 1.))) } }
    for b in &g.bubbles { for k in 0..3 { let ph = (t * 1.5 + k as f32 * 2.1) % 1.; r.sphere(vec3(b.0.x + (k as f32 - 1.) * 0.12, -0.18 + ph * 0.05, b.0.y + ph * 0.1), Vec3::splat(0.04 + ph * 0.03), Color::new(0.95, 0.98, 1., 1.)) } }
    { let b = tc(BOARD); r.cube(vec3(b.x - 0.35, 0.6, b.y), vec3(0.04, 0.6, 0.04), WOOD); r.cube(vec3(b.x + 0.35, 0.6, b.y), vec3(0.04, 0.6, 0.04), WOOD); r.cube(vec3(b.x, 0.95, b.y), vec3(0.45, 0.3, 0.04), Color::new(0.5, 0.35, 0.22, 1.));
        for (k, c) in [WHITE, Color::new(1., 0.9, 0.6, 1.), Color::new(0.8, 0.9, 1., 1.)].iter().enumerate() { r.cube(vec3(b.x - 0.25 + k as f32 * 0.25, 0.97, b.y + 0.045), vec3(0.09, 0.12, 0.005), *c) } }
    for bt in &g.w.built { draw_project(r, bt.kind, bt.x, bt.z, lt.night, false) }
    if g.contest_host_open() {
        let (x, z) = crate::events::CONTEST_SPOT;
        r.disc(vec3(x, 0.02, z), vec2(0.28, 0.23), Color::new(0., 0., 0., 0.2));
        human(r, 1, (None, Some(67), 0), vec3(x, 0., z), 0., 0., 0., 0, None, t, 0.);
        if g.me.pos.distance(vec2(x, z)) < 5. { labels.push((vec3(x, 1.5, z), crate::events::CONTEST_HOST.into(), WHITE)); }
    }
    if let Some(m) = &g.w.mproject { let p = tc((m.x, m.z)); r.cube(vec3(p.x, 0.45, p.y + 0.35), vec3(0.04, 0.45, 0.04), WOOD); r.cube(vec3(p.x, 0.85, p.y + 0.35), vec3(0.3, 0.18, 0.03), Color::new(0.95, 0.85, 0.5, 1.)); r.alpha = 0.35; draw_project(r, MPROJECTS[m.idx].2, m.x, m.z, 0., true); r.alpha = 1.;
        if g.me.pos.distance(p) < 4. { labels.push((vec3(p.x, 1.4, p.y + 0.35), format!("{} {}/{}c", MPROJECTS[m.idx].0, m.funded, MPROJECTS[m.idx].1), Color::new(1., 0.95, 0.6, 1.))) } }
    if let Some(idx) = g.me.mplace { let t = tile_of(g.me.pos + g.me.face); let mut gc = Game::from_world(g.w.clone(), g.map.clone()); gc.me.pos = g.me.pos; gc.me.face = g.me.face; let ok = gc.mp_site_ok(t);
        r.disc(vec3(t.0 as f32 + 0.5, 0.03, t.1 as f32 + 0.5), vec2(0.48, 0.48), if ok { Color::new(0.2, 0.95, 0.3, 0.7) } else { Color::new(0.95, 0.2, 0.2, 0.6) }); r.alpha = 0.4; draw_project(r, MPROJECTS[idx].2, t.0, t.1, 0., true); r.alpha = 1. }
    { let (dx, dz) = (DOCK.0 as f32 + 0.5, DOCK.1 as f32); for k in 0..3 { r.cube(vec3(dx, 0.05, dz + 0.5 + k as f32), vec3(0.5, 0.05, 0.5), WOOD); r.cube(vec3(dx - 0.45, -0.05, dz + 0.5 + k as f32), vec3(0.05, 0.25, 0.05), sh(WOOD, 0.7)); r.cube(vec3(dx + 0.45, -0.05, dz + 0.5 + k as f32), vec3(0.05, 0.25, 0.05), sh(WOOD, 0.7)) }
        r.cube(vec3(dx + 1.0, -0.05, dz + 2.3), vec3(0.35, 0.12, 0.8), Color::new(0.85, 0.85, 0.9, 1.)); r.cube(vec3(dx + 1.0, 0.08, dz + 2.3), vec3(0.3, 0.02, 0.7), WOOD);
        resident(r, 12, vec3(dx, 0.1, dz + 0.4), 0., 0., false, g.t, false, 0.);
        if g.me.pos.distance(vec2(dx, dz)) < 4. { labels.push((vec3(dx, 1.9, dz + 0.4), "Captain Pell's boat".into(), Color::new(0.8, 0.95, 1., 1.))) } }
    if let Some(p) = &g.w.project { let (sx, sz) = if p.kind == PK_BRIDGE { (p.x as f32 + 1., p.z as f32 + 1.3) } else { (p.x as f32 + 0.5, p.z as f32 + 0.5) };
        r.cube(vec3(sx, 0.45, sz + 0.3), vec3(0.04, 0.45, 0.04), WOOD); r.cube(vec3(sx, 0.85, sz + 0.3), vec3(0.3, 0.18, 0.03), Color::new(0.85, 0.75, 0.55, 1.)); r.cube(vec3(sx, 0.85, sz + 0.335), vec3(0.3 * p.need.iter().map(|n| n.2 as f32 / n.1 as f32).sum::<f32>() / p.need.len() as f32, 0.04, 0.005), Color::new(0.3, 0.7, 0.35, 1.));
        r.alpha = 0.35; draw_project(r, p.kind, p.x, p.z, 0., true); r.alpha = 1.;
        if g.me.pos.distance(vec2(sx, sz)) < 4. { let done: u16 = p.need.iter().map(|n| n.2).sum(); let tot: u16 = p.need.iter().map(|n| n.1).sum(); labels.push((vec3(sx, 1.4, sz + 0.3), format!("{} {done}/{tot}", PROJECTS[p.kind as usize].0), Color::new(0.8, 1., 0.8, 1.))) } }
    for o in 0..2 { let p = tc(mailbox_tile(o)); let unread = if g.w.p.id == o { g.w.p.mail.iter().any(|l| !l.read) } else { g.w.others.iter().find(|pr| pr.id == o).is_some_and(|pr| pr.mail.iter().any(|l| !l.read)) };
        r.cube(vec3(p.x, 0.35, p.y), vec3(0.04, 0.35, 0.04), WOOD); r.cube(vec3(p.x, 0.78, p.y), vec3(0.16, 0.12, 0.22), if o == 0 { Color::new(0.85, 0.3, 0.3, 1.) } else { Color::new(0.3, 0.6, 0.5, 1.) });
        r.with(Mat4::from_translation(vec3(p.x + 0.18, 0.8, p.y - 0.05)) * Mat4::from_rotation_z(if unread { 0. } else { -FRAC_PI_2 }), |r| { r.cube(vec3(0., 0.12, 0.), vec3(0.012, 0.12, 0.012), DARKGRAY); r.cube(vec3(0., 0.2, 0.06), vec3(0.01, 0.04, 0.06), YELLOW) }) }
    for &w in &g.w.weeds { let p = tc(w); for (ox, oz) in [(-0.1, 0.), (0.08, 0.06), (0., -0.08)] { r.cone(vec3(p.x + ox, 0., p.y + oz), 0.05, 0.22, Color::new(0.25, 0.5, 0.2, 1.)) } }
    for &h in &g.w.holes { let p = tc(h); for k in 0..5 { let a = k as f32 * 1.26; r.sphere(vec3(p.x + a.cos() * 0.36, 0.03, p.y + a.sin() * 0.32), vec3(0.1, 0.06, 0.1), Color::new(0.5, 0.36, 0.24, 1.)) } }
    for &dg in &g.w.digs { let p = tc(dg); for a in [0.7f32, -0.7] { r.with(Mat4::from_translation(vec3(p.x, 0.012, p.y)) * Mat4::from_rotation_y(a), |r| r.cube(Vec3::ZERO, vec3(0.2, 0.01, 0.025), Color::new(0.35, 0.25, 0.16, 1.))) } }
    for it in &g.w.ground { let c = rgb(def(it.def).col); match def(it.def).k { K::Shell => { r.sphere(vec3(it.x, 0.05, it.z), vec3(0.14, 0.06, 0.12), c); r.cone(vec3(it.x, 0.05, it.z), 0.08, 0.1, sh(c, 0.9)) } _ => r.sphere(vec3(it.x, 0.11, it.z), Vec3::splat(0.11), c) } }
    for (i, n) in g.npcs.iter().enumerate() { if n.inside || !near(n.pos.x, n.pos.y) { continue }
        let talking = matches!(g.me.ui, Ui::Talk { npc: Some(j), .. } if j == i);
        let near_me = !n.moving && n.pos.distance(g.me.pos) < 3. && g.me.scene == Scene::Town; let yaw = if near_me || talking { yaw_of(g.me.pos - n.pos) } else { n.yaw }; // idle residents look at a nearby player
        resident_u(r, g.w.roster[i], vec3(n.pos.x, 0., n.pos.y), yaw, n.anim, n.moving, t, talking, n.happy, weather(g.w.day, now.month()) == Wx::Rain);
        if let Some(e) = emote(g, i, t) { let hp = vec3(n.pos.x + 0.25, 1.55 + (t * 2.).sin() * 0.03, n.pos.y); draw_emote(r, e, hp, t, labels) }
        if !n.moving && !talking && g.seek_hide_position(i).is_none() && weather(g.w.day, now.month()) != Wx::Rain { hobby_prop(r, g.w.roster[i], vec3(n.pos.x, 0., n.pos.y), yaw, t) }
        if let Some(&(a, b, left)) = g.chats.iter().find(|c| c.0 == i || c.1 == i) { let first = a == i; let speak = ((5. - left) as i32 % 2 == 0) == first; let pers = g.res(i).pers;
            if speak && n.pos.distance(g.me.pos) < 7. { labels.push((vec3(n.pos.x, 1.95, n.pos.y), format!("\"{}\"", CHATTER[pers][(i + b) % 3]), WHITE)) } }
        else if n.pos.distance(g.me.pos) < 4. { labels.push((vec3(n.pos.x, 1.75, n.pos.y), g.res(i).name.into(), Color::new(1., 0.95, 0.7, 1.))) } }
    let sy = if g.swimming() { for k in 0..10 { let a = k as f32 * 0.63 + t; r.sphere(vec3(g.me.pos.x + a.cos() * 0.38, -0.17, g.me.pos.y + a.sin() * 0.3), vec3(0.05, 0.015, 0.05), Color::new(0.9, 0.97, 1., 1.)) } -0.55 } else { 0. };
    let opose = match g.me.pose { Pose::Stand => 0, Pose::Sit(_) => 1, Pose::Lie(_) => 2 };
    human(r, g.me.id, (g.w.p.shirt, g.w.p.hat, g.w.p.hair), vec3(g.me.pos.x, sy + if opose == 1 { 0.02 } else { 0. }, g.me.pos.y), yaw_of(g.me.face), g.me.anim, g.me.spd, opose, g.w.p.equip.map(|t| t + 1000 * g.tool_level(t) as u16), t, g.me.swing);
    others(r, g, t);
    for b in &g.bugs { let c = rgb(def(b.0).col); let fl = (t * 20. + b.2).sin().abs(); let y = 0.6 + (t * 3. + b.2).sin() * 0.15;
        for s in [-1f32, 1.] { r.sphere(vec3(b.1.x + s * 0.07, y + fl * 0.04, b.1.y), vec3(0.07, 0.01 + fl * 0.03, 0.05), c) } r.sphere(vec3(b.1.x, y, b.1.y), Vec3::splat(0.03), DARK) }
    if let Some(f) = &g.me.fish { let y = -0.15 - f.dip * 0.1; r.sphere(vec3(f.bob.x, y, f.bob.y), Vec3::splat(0.06), RED); r.sphere(vec3(f.bob.x, y + 0.05, f.bob.y), Vec3::splat(0.04), WHITE);
        let tip = g.me.pos + g.me.face * 0.75; r.seg(vec3(tip.x, 1.05, tip.y), vec3(f.bob.x, y + 0.05, f.bob.y), 0.004, Color::new(0.95, 0.95, 0.95, 1.)) }
    r.alpha = 0.32; for z in (pp.y as i32)..(pp.y as i32 + 3) { for x in (pp.x as i32 - 1)..=(pp.x as i32 + 1) { let (px, pz) = (x as f32 + 0.5, z as f32 + 0.5); if ext(x, z) == PINE && (px - pp.x).abs() < 1.1 && pz > pp.y + 0.2 && pz < pp.y + 2.6 { pine(r, px, pz, 1.1 + (hash(x, z) % 3) as f32 * 0.12) } } } for &i in &fade_t { tree_at(r, i) } for &bi in &fade_b { let b = &BLDS[bi]; let paint = if let BK::Home(o) = b.kind { g.owner(o).paint.map(|k| (PAINTS[k as usize % PAINTS.len()].1, PAINTS[k as usize % PAINTS.len()].2)) } else { None }; building(r, bi, lt.night, paint) } r.alpha = 1.;
    for &(p, a, k) in &g.fx { let s = a / 0.8; match k {
        0 => for i in 0..10 { let an = i as f32 / 10. * 2. * PI; let rr = 0.15 + s * 0.6; r.sphere(p + vec3(an.cos() * rr, 0.02, an.sin() * rr), vec3(0.05, 0.015, 0.05), Color::new(0.92, 0.97, 1., 1. - s)) },
        1 => for i in 0..6 { let an = i as f32 * 1.05; r.sphere(p + vec3(an.cos() * s * 0.45, s * 1.1 - s * s * 0.8, an.sin() * s * 0.6), Vec3::splat(0.12 * (1. - s) + 0.03), Color::new(0.86, 0.74, 0.55, 1.)) },
        _ => for i in 0..5 { let an = -0.8 + i as f32 * 0.4; let d = vec3(an.sin() * 0.35, 0., an.cos() * 0.35); r.sphere(p + d * (0.5 + s), Vec3::splat(0.035), Color::new(1., 1., 1., 1. - s)) },
    } }
    for h in &g.hearts { if h.1 > 0. { r.sphere(h.0, Vec3::splat(0.06), Color::new(1., 0.4, 0.55, 1.)) } }
    if g.me.ui == Ui::Play && g.me.trans.is_none() { if let Some(tg) = g.target() { indicator(r, g.tg_pos(tg), t) } }
}
/// Other local players who are in the same scene as the viewing player.
fn others(r: &mut R, g: &Game, t: f32) {
    for o in g.others.iter().filter(|o| o.scene == g.me.scene) {
        let p = if o.scene == Scene::Town { o.pos } else { o.ipos }; let pose = match o.pose { Pose::Stand => 0, Pose::Sit(_) => 1, Pose::Lie(_) => 2 };
        let tool = if o.scene == Scene::Town { g.w.others.iter().find(|pr| pr.id == o.id).and_then(|pr| pr.equip.map(|t| t + 1000 * UPGRADABLE.iter().position(|&u| u == t).and_then(|k| pr.tool_lv.get(k).copied()).unwrap_or(0) as u16)) } else { None };
        if pose == 0 { r.disc(vec3(p.x, 0.02, p.y), vec2(0.32, 0.26), Color::new(0., 0., 0., 0.22)) }
        let shirt = g.w.others.iter().find(|pr| pr.id == o.id).map_or((None, None, 0), |pr| (pr.shirt, pr.hat, pr.hair));
        let sy = if o.scene == Scene::Town && tile_at(&g.map, p.x.floor() as i32, p.y.floor() as i32) == SEA { -0.55 } else { 0. };
        human(r, o.id, shirt, vec3(p.x, sy, p.y), yaw_of(o.face), o.anim, o.spd, pose, tool, t, o.swing);
    }
}
fn room(r: &mut R, g: &Game, b: usize, lt: &Light, labels: &mut Vec<(Vec3, String, Color)>) {
    let (w, d) = g.rdims(b); let (wf, df) = (w as f32, d as f32); let t = g.t;
    let (fl, wl) = match BLDS[b].kind { BK::Shop => ((0.72, 0.76, 0.66), (0.93, 0.95, 0.86)), BK::Civic => ((0.5, 0.36, 0.28), (0.9, 0.87, 0.78)), BK::Res(i) => THEME[i], BK::Home(o) => { let pr = g.owner(o); (FLOORS[pr.floor as usize % 4], WALLS[pr.wall as usize % 4]) } _ => ((0.78, 0.6, 0.42), (0.97, 0.93, 0.84)) };
    let st = g.story_in(b); let stair = g.home_stairs(b);
    let (fl, wl) = match st { 1 => ((0.6, 0.56, 0.5), (0.7, 0.5, 0.42)), 2 => ((0.84, 0.7, 0.52), (0.88, 0.9, 0.92)), _ => (fl, wl) };
    let (floor, wall) = (rgb(fl), rgb(wl)); let furn = g.room_furn(b); let lamp = furn.iter().any(|f| def(f.def).act == Act::Lamp && f.on);
    let dim = lt.night > 0.6 && !lamp;
    r.sun = vec3(0.35, 0.85, 0.5).normalize();
    r.amb = if dim { vec3(0.3, 0.32, 0.48) } else if lamp && lt.night > 0.3 { vec3(0.68, 0.57, 0.42) } else { vec3(0.56, 0.56, 0.56) };
    r.dif = if dim { vec3(0.14, 0.14, 0.22) } else { vec3(0.44, 0.42, 0.4) };
    if st == 1 { r.amb *= 0.86; r.dif *= 0.8 } else if st == 2 { r.amb *= 1.05 }
    let tiled = matches!(BLDS[b].kind, BK::Shop | BK::Museum) || st == 1;
    for z in 0..d { for x in 0..w { let (fx, fz) = (x as f32, z as f32);
        if stair.iter().any(|s| s.0 == (x, z) && elev(s.1) < elev(st)) { continue } // stairwell opening
        if tiled { r.quad(vec3(fx, 0., fz), vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), Vec3::Y, sh(floor, if (x + z) % 2 == 0 { 1. } else { 0.95 }));
            r.quad(vec3(fx, 0.002, fz), vec3(fx + 1., 0.002, fz), vec3(fx + 1., 0.002, fz + 0.025), vec3(fx, 0.002, fz + 0.025), Vec3::Y, sh(floor, 0.82)); r.quad(vec3(fx, 0.002, fz), vec3(fx + 0.025, 0.002, fz), vec3(fx + 0.025, 0.002, fz + 1.), vec3(fx, 0.002, fz + 1.), Vec3::Y, sh(floor, 0.82)) }
        else { // wooden boards: three planks per tile running left-right, staggered seams, gentle per-plank tone
            for k in 0..3 { let (z0, z1) = (fz + k as f32 / 3., fz + (k + 1) as f32 / 3.); let v = 0.94 + (hash(x * 3 + k * 7, z * 5 + k) % 9) as f32 * 0.012;
                r.quad(vec3(fx, 0., z0), vec3(fx + 1., 0., z0), vec3(fx + 1., 0., z1), vec3(fx, 0., z1), Vec3::Y, sh(floor, v));
                r.quad(vec3(fx, 0.002, z0), vec3(fx + 1., 0.002, z0), vec3(fx + 1., 0.002, z0 + 0.018), vec3(fx, 0.002, z0 + 0.018), Vec3::Y, sh(floor, 0.78));
                let sx = fx + if (k + x) % 2 == 0 { 0.33 } else { 0.8 }; r.quad(vec3(sx, 0.002, z0), vec3(sx + 0.018, 0.002, z0), vec3(sx + 0.018, 0.002, z1), vec3(sx, 0.002, z1), Vec3::Y, sh(floor, 0.8)) } } } }
    // back wall: wainscot panel below a chair rail, softly striped paper above
    r.quad(vec3(0., 0., 0.), vec3(wf, 0., 0.), vec3(wf, 0.85, 0.), vec3(0., 0.85, 0.), Vec3::Z, sh(wall, 0.86));
    r.quad(vec3(0., 0.85, 0.), vec3(wf, 0.85, 0.), vec3(wf, 2.4, 0.), vec3(0., 2.4, 0.), Vec3::Z, wall);
    if st == 1 { let mortar = sh(wall, 0.8); let mut row = 0; let mut y = 0.98; while y < 2.36 { r.quad(vec3(0., y, 0.005), vec3(wf, y, 0.005), vec3(wf, y + 0.02, 0.005), vec3(0., y + 0.02, 0.005), Vec3::Z, mortar);
            let mut x = if row % 2 == 0 { 0.3 } else { 0. }; while x < wf { r.quad(vec3(x, y, 0.005), vec3(x + 0.02, y, 0.005), vec3(x + 0.02, (y + 0.2).min(2.38), 0.005), vec3(x, (y + 0.2).min(2.38), 0.005), Vec3::Z, mortar); x += 0.6 } y += 0.2; row += 1 }
        r.cube(vec3(wf / 2., 2.2, 0.08), vec3(wf / 2., 0.035, 0.035), Color::new(0.55, 0.42, 0.3, 1.)); } // brick courses and an old pipe
    else { let mut x = 0.; let mut k = 0; while x < wf - 0.01 { if k % 2 == 1 { r.quad(vec3(x, 0.88, 0.005), vec3((x + 0.18).min(wf), 0.88, 0.005), vec3((x + 0.18).min(wf), 2.34, 0.005), vec3(x, 2.34, 0.005), Vec3::Z, sh(wall, 0.965)) } x += 0.18; k += 1 } }
    r.cube(vec3(wf / 2., 0.86, 0.025), vec3(wf / 2., 0.025, 0.025), WOOD);
    { let mut x = 0.4; while x < wf - 0.2 { r.cube(vec3(x, 0.47, 0.006), vec3(0.008, 0.3, 0.006), sh(wall, 0.78)); x += 0.8 } }
    r.cube(vec3(wf / 2., 0.08, 0.03), vec3(wf / 2., 0.08, 0.03), WOOD); r.cube(vec3(wf / 2., 2.38, 0.03), vec3(wf / 2., 0.04, 0.04), WOOD);
    for x in [-0.08, wf + 0.08] { r.cube(vec3(x, 0.5, df / 2. - 0.04), vec3(0.08, 0.5, df / 2. + 0.04), sh(wall, 0.85)) } // cut-away side walls
    let dxw = (w / 2) as f32; if st != 0 { r.cube(vec3(wf / 2., 0.12, df + 0.06), vec3(wf / 2., 0.12, 0.06), sh(wall, 0.8)) } else { r.cube(vec3(dxw / 2., 0.12, df + 0.06), vec3(dxw / 2., 0.12, 0.06), sh(wall, 0.8)); r.cube(vec3((dxw + 1. + wf) / 2., 0.12, df + 0.06), vec3((wf - dxw - 1.) / 2., 0.12, 0.06), sh(wall, 0.8));
    r.cube(vec3(dxw + 0.5, 0.005, df - 0.4), vec3(0.4, 0.005, 0.3), Color::new(0.7, 0.3, 0.3, 1.)) } // door mat
    for &((sx, sz), to) in &stair { let (x0, z0) = (sx as f32, sz as f32);
        if elev(to) > elev(st) { // stairs climbing into the back wall, with a dark landing above and a rail
            for k in 0..6 { let h = (k + 1) as f32 * 0.38; let zc = z0 + 1. - (k as f32 + 0.5) / 6.; r.cube(vec3(x0 + 0.5, h / 2., zc), vec3(0.44, h / 2., 1. / 12.), sh(WOOD, 0.86 + k as f32 * 0.03)); r.cube(vec3(x0 + 0.5, h + 0.01, zc + 0.07), vec3(0.45, 0.012, 0.02), sh(WOOD, 1.15)) }
            r.quad(vec3(x0 + 0.06, 1.9, 0.004), vec3(x0 + 0.94, 1.9, 0.004), vec3(x0 + 0.94, 2.4, 0.004), vec3(x0 + 0.06, 2.4, 0.004), Vec3::Z, Color::new(0.16, 0.12, 0.1, 1.));
            r.cube(vec3(x0 + 0.95, 0.47, z0 + 0.95), vec3(0.035, 0.47, 0.035), sh(WOOD, 0.7)); r.cube(vec3(x0 + 0.95, 0.96, z0 + 0.95), vec3(0.05, 0.03, 0.05), sh(WOOD, 0.9)); // newel post
            r.with(Mat4::from_rotation_translation(Quat::from_rotation_x(1.0), vec3(x0 + 0.95, 1.65, z0 + 0.5)), |r| r.cube(Vec3::ZERO, vec3(0.025, 0.025, 0.83), sh(WOOD, 0.75)));
        } else { // stairwell going down: steps sinking toward the back, shaded walls, a low rail
            let dark = |k: f32| Color::new(0.3 - k * 0.05, 0.24 - k * 0.04, 0.2 - k * 0.035, 1.);
            for k in 0..4 { let y = -(k as f32 + 1.) * 0.28; let (za, zb) = (z0 + 1. - (k + 1) as f32 * 0.25, z0 + 1. - k as f32 * 0.25);
                r.quad(vec3(x0, y, za), vec3(x0 + 1., y, za), vec3(x0 + 1., y, zb), vec3(x0, y, zb), Vec3::Y, sh(WOOD, 0.8 - k as f32 * 0.12));
                r.quad(vec3(x0, y, zb), vec3(x0 + 1., y, zb), vec3(x0 + 1., y + 0.28, zb), vec3(x0, y + 0.28, zb), Vec3::Z, dark(k as f32)) }
            for x in [x0, x0 + 1.] { r.quad(vec3(x, -1.2, z0), vec3(x, -1.2, z0 + 1.), vec3(x, 0., z0 + 1.), vec3(x, 0., z0), Vec3::X, dark(1.5)) }
            r.quad(vec3(x0, -1.2, z0 + 0.002), vec3(x0 + 1., -1.2, z0 + 0.002), vec3(x0 + 1., 0., z0 + 0.002), vec3(x0, 0., z0 + 0.002), Vec3::Z, dark(3.));
            let rx = if sx == 0 { x0 + 1. } else { x0 + 0.98 }; for z in [z0 + 0.1, z0 + 0.9] { r.cube(vec3(rx, 0.32, z), vec3(0.025, 0.32, 0.025), sh(WOOD, 0.7)) } r.cube(vec3(rx, 0.64, z0 + 0.5), vec3(0.03, 0.025, 0.45), WOOD);
        } }
    if st == 2 { let beam = sh(WOOD, 0.85); for (cx, a) in [(wf * 0.25, 0.3f32), (wf * 0.75, -0.3)] { r.with(Mat4::from_rotation_translation(Quat::from_rotation_z(a), vec3(cx, 1.95, 0.05)), |r| r.cube(Vec3::ZERO, vec3(wf * 0.28, 0.05, 0.05), beam)) }
        r.cube(vec3(wf / 2., 2.33, 0.06), vec3(0.08, 0.08, 0.06), beam); } // attic rafters
    if st == 1 { let wx = wf - 0.9; r.cube(vec3(wx, 2.05, 0.04), vec3(0.36, 0.17, 0.04), WOOD); let u = r.unlit; r.unlit = true; r.cube(vec3(wx, 2.05, 0.07), vec3(0.3, 0.12, 0.02), Color::new(lt.sky.r * 0.8 + 0.1, lt.sky.g * 0.8 + 0.1, lt.sky.b * 0.8 + 0.1, 1.)); r.unlit = u;
        r.cube(vec3(wx, 2.05, 0.09), vec3(0.008, 0.12, 0.01), WOOD); } // small high window at ground level
    if !matches!(BLDS[b].kind, BK::Shop | BK::Museum) && st != 1 { let wx = if w > 3 && st != 2 { wf - 0.9 } else { wf / 2. }; r.cube(vec3(wx, 1.45, 0.04), vec3(0.42, 0.36, 0.04), WOOD); let u = r.unlit; r.unlit = true; r.cube(vec3(wx, 1.45, 0.07), vec3(0.34, 0.28, 0.02), Color::new(lt.sky.r * 0.9 + 0.1, lt.sky.g * 0.9 + 0.1, lt.sky.b * 0.9 + 0.1, 1.));
        let day = 1. - lt.night; if day > 0.05 { let a = 0.16 * day; let c = Color::new(1., 0.93, 0.7, a); r.quad(vec3(wx - 0.36, 0.004, 0.25), vec3(wx + 0.36, 0.004, 0.25), vec3(wx + 0.62, 0.004, 1.45), vec3(wx - 0.1, 0.004, 1.45), Vec3::Y, c);
            r.quad(vec3(wx - 0.3, 0.005, 0.5), vec3(wx - 0.27, 0.005, 0.5), vec3(wx + 0.03, 0.005, 1.4), vec3(wx, 0.005, 1.4), Vec3::Y, Color::new(0.4, 0.3, 0.2, a * 0.6)); }
        r.unlit = u;
        if matches!(BLDS[b].kind, BK::Home(_)) {
            // Gathered linen panels, lifted above the glass (front face z = 0.09).
            let linen = Color::new(0.56, 0.72, 0.65, 1.);
            r.cube(vec3(wx, 1.845, 0.11), vec3(0.49, 0.015, 0.02), WOOD);
            for sx in [-1f32, 1.] {
                let cx = wx + sx * 0.33;
                r.quad(vec3(cx - 0.05, 1.12, 0.118), vec3(cx + 0.05, 1.12, 0.118), vec3(cx + 0.09, 1.79, 0.118), vec3(cx - 0.09, 1.79, 0.118), Vec3::Z, linen);
                r.quad(vec3(cx - 0.01, 1.12, 0.124), vec3(cx + 0.01, 1.12, 0.124), vec3(cx + 0.018, 1.79, 0.124), vec3(cx - 0.018, 1.79, 0.124), Vec3::Z, sh(linen, 0.88));
            }
            r.cube(vec3(wx, 1.065, 0.11), vec3(0.48, 0.03, 0.08), WOOD);
        }
    }
    if BLDS[b].kind == BK::Civic {
        r.cube(vec3(3.5, 0.005, 3.2), vec3(0.6, 0.005, 1.6), Color::new(0.55, 0.15, 0.18, 1.)); // carpet
        for &(x, z) in &CIVIC_DESK { r.cube(vec3(x as f32 + 0.5, 0.3, z as f32 + 0.5), vec3(0.5, 0.3, 0.42), Color::new(0.42, 0.28, 0.2, 1.)); r.cube(vec3(x as f32 + 0.5, 0.62, z as f32 + 0.5), vec3(0.52, 0.03, 0.45), Color::new(0.3, 0.45, 0.35, 1.)) }
        r.cube(vec3(2.6, 0.68, 1.4), vec3(0.12, 0.03, 0.09), WHITE); r.cube(vec3(4.4, 0.72, 1.45), vec3(0.04, 0.08, 0.04), GOLD); // papers, pen stand
        let talking = matches!(g.me.ui, Ui::Talk { npc: None, .. } | Ui::Civic(_) | Ui::Policy(_) | Ui::PolicyOk(_) | Ui::MPick(_)); resident(r, 13, vec3(3.5, 0., 0.5), 0., 0., false, t, talking, 0.);
        r.cube(vec3(3.5, 2.1, 0.06), vec3(0.5, 0.18, 0.03), Color::new(0.3, 0.45, 0.35, 1.)); r.cube(vec3(3.5, 2.1, 0.09), vec3(0.42, 0.1, 0.01), GOLD); // crest
        for (k, &(x, z)) in CIVIC_PLANTS.iter().enumerate() { let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5); r.cyl(p, 0.22, 0.35, Color::new(0.7, 0.45, 0.3, 1.)); r.sphere(p + vec3(0., 0.7, 0.), vec3(0.32, 0.4 + k as f32 * 0.05, 0.32), Color::new(0.25, 0.55, 0.3, 1.)) }
        r.cube(vec3(1.0, 1.35, 0.05), vec3(0.45, 0.4, 0.03), Color::new(0.6, 0.45, 0.3, 1.)); r.cube(vec3(1.0, 1.35, 0.08), vec3(0.4, 0.35, 0.01), Color::new(0.95, 0.92, 0.82, 1.)); // project board
        if let Some(m) = &g.w.mproject { let f = m.funded as f32 / MPROJECTS[m.idx].1 as f32; r.cube(vec3(0.65 + 0.35 * f, 1.2, 0.1), vec3(0.35 * f, 0.04, 0.01), Color::new(0.3, 0.7, 0.35, 1.)) }
        for &(bx, bz) in &CIVIC_BENCH { let (x, z) = (bx as f32 + 0.5, bz as f32 + 0.5); r.cube(vec3(x, 0.22, z), vec3(0.4, 0.04, 0.3), WOOD); r.cube(vec3(x, 0.1, z), vec3(0.36, 0.1, 0.26), sh(WOOD, 0.8)) } // waiting bench
        let ppos = g.me.pos; let _ = ppos; labels.push((vec3(1.0, 1.95, 0.1), match &g.w.mproject { Some(m) => format!("{}: {}/{}c", MPROJECTS[m.idx].0, m.funded, MPROJECTS[m.idx].1), None => "No funded project".into() }, Color::new(1., 0.95, 0.7, 1.)));
        labels.push((vec3(3.5, 2.5, 0.1), format!("Policy: {}", g.pol().map_or("none", |p| p.name)), Color::new(0.8, 1., 0.85, 1.)));
        { let k = tc(KIOSK); r.cube(vec3(k.x, 0.45, k.y), vec3(0.3, 0.45, 0.25), Color::new(0.3, 0.45, 0.6, 1.)); let u = r.unlit; r.unlit = true; r.cube(vec3(k.x + 0.26, 0.85, k.y), vec3(0.01, 0.2, 0.17), Color::new(0.6, 0.9, 1., 1.)); r.unlit = u; r.cube(vec3(k.x, 0.95, k.y), vec3(0.32, 0.04, 0.27), GOLD);
          if g.me.ipos.distance(k) < 2.5 { labels.push((vec3(k.x, 1.4, k.y), "Initiatives kiosk".into(), Color::new(0.8, 0.95, 1., 1.))) } }
    }
    if BLDS[b].kind == BK::Museum {
        let me = g.me.ipos;
        for (k, &d) in ARTS.iter().enumerate() { let p = vec3(GALLERY_X[k], 1.85, 0.05); if g.w.donated.iter().any(|x| x.0 == d) { painting(r, d, p, 0.62, 0.46); if me.distance(vec2(p.x, 0.8)) < 1.6 { labels.push((vec3(p.x, 2.25, 0.1), def(d).name.into(), Color::new(1., 0.95, 0.75, 1.))) } } else { r.cube(p, vec3(0.33, 0.25, 0.01), Color::new(0.82, 0.8, 0.75, 1.)) } }
        labels.push((vec3(3.5, 2.45, 0.1), format!("Art wing  {}/{}", g.donated_of(K::Art), ARTS.len()), Color::new(0.85, 0.9, 1., 1.)));
        if me.distance(vec2(0.9, 2.5)) < 1.6 { labels.push((vec3(0.5, 1.4, 2.5), format!("Aquarium: {} fish & sea life", g.w.donated.iter().filter(|x| matches!(def(x.0).k, K::Fish | K::Sea)).count()), Color::new(0.75, 0.95, 1., 1.))) }
        if me.distance(vec2(6.1, 2.5)) < 1.6 { labels.push((vec3(6.5, 1.3, 2.5), format!("Insect case: {} bugs", g.donated_of(K::Bug)), Color::new(0.9, 1., 0.8, 1.))) }
        for &((x, z), fd) in &MUSEUM_PED { if me.distance(vec2(x as f32 + 0.5, z as f32 + 0.5)) < 1.5 { labels.push((vec3(x as f32 + 0.5, 1.15, z as f32 + 0.5), if g.w.donated.iter().any(|y| y.0 == fd) { def(fd).name.to_string() } else { "(empty pedestal)".into() }, Color::new(1., 0.92, 0.8, 1.))) } }
        for &(x, z) in &MUSEUM_DESK { r.cube(vec3(x as f32 + 0.5, 0.45, z as f32 + 0.5), vec3(0.5, 0.45, 0.42), Color::new(0.4, 0.3, 0.25, 1.)) }
        let talking = matches!(g.me.ui, Ui::Talk { npc: None, .. }); resident(r, 11, vec3(3.5, 0., 0.5), 0., 0., false, t, talking, 0.);
        r.cube(vec3(0.5, 0.15, 2.5), vec3(0.48, 0.15, 1.48), Color::new(0.35, 0.3, 0.28, 1.)); // tank base
        for (k, d) in g.w.donated.iter().filter(|x| matches!(def(x.0).k, K::Fish | K::Sea)).enumerate() { let ph = t * 0.6 + k as f32 * 1.7; let s = def(d.0).size * 0.35;
            r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(if ph.cos() > 0. { 0. } else { PI }), vec3(0.5, 0.55 + (k % 4) as f32 * 0.18, 2.5 + ph.sin() * 1.1)), |r| { r.sphere(Vec3::ZERO, vec3(s * 0.45, s * 0.5, s), rgb(def(d.0).col)); r.cone(vec3(0., 0., -s), s * 0.4, s * 0.6, rgb(def(d.0).col)) }) }
        let ua = r.alpha; r.alpha = 0.35; r.cube(vec3(0.5, 0.75, 2.5), vec3(0.47, 0.45, 1.46), Color::new(0.45, 0.75, 0.95, 1.)); r.alpha = ua;
        r.cube(vec3(6.5, 0.45, 2.5), vec3(0.45, 0.45, 1.45), Color::new(0.55, 0.38, 0.25, 1.)); r.cube(vec3(6.5, 0.92, 2.5), vec3(0.43, 0.02, 1.43), Color::new(0.95, 0.93, 0.85, 1.));
        for (k, d) in g.w.donated.iter().filter(|x| def(x.0).k == K::Bug).enumerate() { let p = vec3(6.5 + ((k % 2) as f32 - 0.5) * 0.4, 0.95, 1.4 + (k / 2) as f32 * 0.5); r.sphere(p, vec3(0.09, 0.02, 0.06), rgb(def(d.0).col)); r.sphere(p, Vec3::splat(0.025), DARK) }
        for &((x, z), fd) in &MUSEUM_PED { let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5); r.cyl(p, 0.36, 0.5, Color::new(0.9, 0.88, 0.84, 1.));
            if g.w.donated.iter().any(|x| x.0 == fd) { let c = rgb(def(fd).col); match fd { 48 => { for k in 0..5 { r.sphere(p + vec3(0., 0.62 + k as f32 * 0.02, 0.), Vec3::splat(0.2 - k as f32 * 0.035), sh(c, 1. - k as f32 * 0.08)) } } 49 => { r.sphere(p + vec3(0., 0.58, 0.), vec3(0.16, 0.06, 0.24), c); for k in 0..4 { r.cube(p + vec3(0., 0.63, -0.15 + k as f32 * 0.1), vec3(0.15, 0.01, 0.015), sh(c, 0.7)) } }
                43 => { r.cube(p + vec3(0., 0.7, 0.), vec3(0.22, 0.2, 0.04), Color::new(0.75, 0.72, 0.65, 1.)); for k in 0..5 { r.cube(p + vec3(-0.1 + k as f32 * 0.05, 0.7, 0.045), vec3(0.01, 0.12, 0.005), c) } } _ => r.cone(p + vec3(0., 0.5, 0.), 0.1, 0.3, c) } }
            else { r.cube(p + vec3(0., 0.53, 0.25), vec3(0.15, 0.03, 0.01), DARKGRAY) } }
    }
    if BLDS[b].kind == BK::Shop {
        for &(x, z) in &COUNTER { r.cube(vec3(x as f32 + 0.5, 0.45, z as f32 + 0.5), vec3(0.5, 0.45, 0.42), WOOD); r.cube(vec3(x as f32 + 0.5, 0.92, z as f32 + 0.5), vec3(0.52, 0.03, 0.45), sh(WOOD, 1.2)) }
        r.cube(vec3(4.3, 1.05, 1.4), vec3(0.15, 0.1, 0.12), DARKGRAY);
        let rk = vec3(RACK.0 as f32 + 0.5, 0., RACK.1 as f32 + 0.5); for x in [-0.4, 0.4] { r.cube(rk + vec3(x, 0.7, 0.), vec3(0.03, 0.7, 0.03), DARKGRAY) } r.cube(rk + vec3(0., 1.38, 0.), vec3(0.42, 0.02, 0.02), DARKGRAY);
        for (k, &d) in CLOTHES.iter().enumerate() { r.cube(rk + vec3(-0.3 + k as f32 * 0.2, 1.05, 0.), vec3(0.08, 0.28, 0.12), rgb(def(d).col)) }
        for k in 0..3 { r.cube(vec3(1.2 + k as f32 * 2.3, 1.4, 0.15), vec3(0.9, 0.04, 0.15), WOOD); for j in 0..4 { r.cube(vec3(0.6 + k as f32 * 2.3 + j as f32 * 0.38, 1.56, 0.15), vec3(0.12, 0.12, 0.1), rgb(DEFS[(j * 7 + k * 3) % 30].col)) } }
        let talking = matches!(g.me.ui, Ui::Sell(_) | Ui::Confirm(_)) || matches!(g.me.ui, Ui::Talk { npc: None, .. });
        resident(r, 10, vec3(3.5, 0., 0.5), 0., 0., false, t, talking, 0.);
        for (di, tile) in [(7usize, EXTRA_DISPLAY), (8, SPECIAL_DISPLAY)] { if (di == 7 && g.w.stock.len() <= 5) || (di == 8 && g.shop_tier() < 1) { continue }
            let p = vec3(tile.0 as f32 + 0.5, 0., tile.1 as f32 + 0.5); let (dd, price, sold) = g.display_item(di); r.cyl(p, 0.4, 0.22, if di == 8 { Color::new(0.85, 0.7, 0.35, 1.) } else { Color::new(0.85, 0.82, 0.75, 1.) });
            if !sold { furniture(r, dd, Mat4::from_translation(p + vec3(0., if def(dd).wall { -0.2 } else { 0.22 }, 0.)) * Mat4::from_scale(Vec3::splat(0.5)), false, 0) }
            if g.me.ipos.distance(vec2(p.x, p.z)) < 2.2 { labels.push((p + vec3(0., 1.15, 0.), if sold { "SOLD OUT".into() } else { format!("{}{} {price}", if di == 8 { "Special: " } else { "" }, def(dd).name) }, Color::new(1., 0.9, 0.5, 1.))) } }
        for (i, &(x, z)) in DISPLAYS.iter().enumerate() { let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5); r.cyl(p, 0.4, 0.22, Color::new(0.85, 0.82, 0.75, 1.));
            let (txt, show) = if i < 5 { let s = g.w.stock[i]; (if s.sold { "SOLD OUT".to_string() } else { format!("{} {}", def(s.def).name, def(s.def).buy) }, (!s.sold).then_some(s.def)) }
                else { let tl = TOOL_DISPLAY[i - 5]; (if g.w.p.tools.contains(&tl) { "Owned".into() } else { format!("{} {}", def(tl).name, def(tl).buy) }, Some(tl)) };
            match show { Some(dd) if def(dd).k == K::Tool => r.with(Mat4::from_translation(p + vec3(0., 0.3, 0.)) * Mat4::from_rotation_z(0.4) * Mat4::from_scale(Vec3::splat(1.4)), |r| draw_tool(r, dd)),
                Some(dd) => { let wall_it = def(dd).wall; furniture(r, dd, Mat4::from_translation(p + vec3(0., if wall_it { -0.2 } else { 0.22 }, 0.)) * Mat4::from_scale(Vec3::splat(0.5)), false, 0) }
                None => r.cube(p + vec3(0., 0.4, 0.), vec3(0.25, 0.15, 0.02), Color::new(0.8, 0.2, 0.2, 1.)) }
            if g.me.ipos.distance(vec2(p.x, p.z)) < 2.2 { labels.push((p + vec3(0., 1.15, 0.), txt, if i < 5 && g.w.stock[i].sold { Color::new(1., 0.5, 0.5, 1.) } else { Color::new(1., 0.95, 0.6, 1.) })) } }
    }
    let owner = if let BK::Home(o) = BLDS[b].kind { Some(g.owner(o)) } else { None };
    for f in &furn { furniture(r, f.def, fbase(f), f.on, owner.map_or(0, |pr| Game::tint_of(pr, f.uid))); if def(f.def).act == Act::Lamp && f.on { let c = fcenter(f); r.disc(vec3(c.x, 0.012, c.y), vec2(0.9, 0.9), Color::new(1., 0.85, 0.4, 0.18)) } }
    if let Some((vi, vb, _)) = g.visit { if vb == b { let s = g.visit_spot(b); let talking = matches!(g.me.ui, Ui::Talk { npc: Some(j), .. } if j == vi);
        r.disc(vec3(s.x, 0.02, s.y), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.2)); resident(r, g.w.roster[vi], vec3(s.x, 0., s.y), yaw_of(g.me.ipos - s), 0., false, t, talking, g.npcs[vi].happy) } }
    if let BK::Res(i) = BLDS[b].kind { if g.loc(i) == Loc::Home { let s = res_spot(); let talking = matches!(g.me.ui, Ui::Talk { npc: Some(j), .. } if j == i);
        r.disc(vec3(s.x, 0.02, s.y), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.2)); resident(r, g.w.roster[i], vec3(s.x, 0., s.y), yaw_of(g.me.ipos - s), 0., false, t, talking, g.npcs[i].happy) } }
    if let Some(p) = &g.me.placing { let ok = g.place_ok(&p.f).is_ok(); let c = if ok { Color::new(0.2, 0.95, 0.3, 0.7) } else { Color::new(0.95, 0.2, 0.2, 0.6) };
        if def(p.f.def).wall { r.disc(vec3(p.f.x as f32 + 0.5, 0.02, 0.3), vec2(0.45, 0.25), c) } else { for (x, z) in foot(p.f.def, p.f.x, p.f.z, p.f.rot) { r.disc(vec3(x as f32 + 0.5, 0.02, z as f32 + 0.5), vec2(0.48, 0.48), c) } }
        furniture(r, p.f.def, fbase(&p.f), false, Game::tint_of(&g.w.p, p.f.uid)) }
    let pose = match g.me.pose { Pose::Stand => 0, Pose::Sit(_) => 1, Pose::Lie(_) => 2 };
    if pose == 0 { r.disc(vec3(g.me.ipos.x, 0.02, g.me.ipos.y), vec2(0.32, 0.26), Color::new(0., 0., 0., 0.2)) }
    human(r, g.me.id, (g.w.p.shirt, g.w.p.hat, g.w.p.hair), vec3(g.me.ipos.x, 0., g.me.ipos.y), yaw_of(g.me.face), g.me.anim, g.me.spd, pose, None, t, 0.);
    others(r, g, t);
    if g.me.ui == Ui::Play && g.me.placing.is_none() && g.me.trans.is_none() { if let Some(tg) = g.target() { indicator(r, g.tg_pos(tg), t) } }
}

// ---- HUD / menus (2D, undistorted) ----
fn txt(s: &str, x: f32, y: f32, z: f32, c: Color) { draw_text(s, x, y, z, c); }
fn panel(x: f32, y: f32, w: f32, h: f32) { draw_rectangle(x, y, w, h, Color::new(0.99, 0.96, 0.88, 0.97)); draw_rectangle_lines(x, y, w, h, 3., Color::new(0.45, 0.3, 0.2, 1.)) }
fn wrap_txt(s: &str, n: usize) -> Vec<String> { let mut o = vec![String::new()]; for w in s.split(' ') { if o.last().unwrap().len() + w.len() > n { o.push(String::new()) } let l = o.last_mut().unwrap(); if !l.is_empty() { l.push(' ') } l.push_str(w) } o }
fn slot_name(s: &Slot) -> String { if s.size_mm > 0 { format!("{}  {} mm", def(s.def).name, s.size_mm) } else if s.qty > 1 { format!("{} x{}", def(s.def).name, s.qty) } else { def(s.def).name.to_string() } }
fn list(items: &[String], sel: Option<usize>, x: f32, y: f32) { for (k, s) in items.iter().enumerate() { let on = sel == Some(k); if on { draw_rectangle(x - 6., y + k as f32 * 26. - 19., 330., 25., Color::new(1., 0.85, 0.4, 0.6)) } txt(s, x, y + k as f32 * 26., 22., BLACK) } }
fn hud(g: &Game, now: NaiveDateTime, dbg: Option<&str>) {
    let (sw, sh) = (vw(), vh());
    let season = match now.month() { 3..=5 => "Spring", 6..=8 => "Summer", 9..=11 => "Autumn", _ => "Winter" };
    draw_rectangle(10., 10., 540., 62., Color::new(0., 0., 0., 0.45));
    const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    txt(&format!("{} {:02}:{:02}  {} {}  {}{}", now.weekday(), now.hour(), now.minute(), MON[now.month0() as usize], now.day(), season, match weather(g.w.day, now.month()) { Wx::Rain => "  Rain", Wx::Snow => "  Snow", Wx::Clear => "" }), 20., 35., 26., WHITE);
    txt(&format!("{} coins    pockets {}/{}    tool: {}", g.w.p.money, g.w.p.inv.len(), INV_CAP, g.w.p.equip.map_or("none", |t| def(t).name)), 20., 62., 22., GOLD);
    // Recent messages: only during play (menus cover them), shortened to fit the viewport, on a soft backdrop for legibility.
    if g.me.ui == Ui::Play { for (k, (m, _)) in g.me.msgs.iter().rev().filter(|m| g.t - m.1 < 5.).take(4).enumerate() { let mut s = m.clone();
        if measure_text(&s, None, 22, 1.).width > sw - 40. { while s.len() > 4 && measure_text(&s, None, 22, 1.).width > sw - 70. { s.pop(); } s = format!("{}...", s.trim_end()) }
        let y = 100. + k as f32 * 24.; draw_rectangle(8., y - 18., measure_text(&s, None, 22, 1.).width + 12., 24., Color::new(0., 0., 0., 0.28)); txt(&s, 14., y, 22., WHITE) } }
    if g.me.ui == Ui::Play && g.me.placing.is_none() && g.me.trans.is_none() { if let Some(tg) = g.target() { let s = if g.me.pad { format!("[A] {}", g.tg_label(tg)).replace("[F]", "[Select]").replace("[G]", "[LB]") } else { format!("[E] {}", g.tg_label(tg)) }; let tw = measure_text(&s, None, 24, 1.).width; draw_rectangle(sw / 2. - tw / 2. - 10., sh - 92., tw + 20., 32., Color::new(0., 0., 0., 0.55)); txt(&s, sw / 2. - tw / 2., sh - 69., 24., WHITE) } }
    if g.me.fish.as_ref().is_some_and(|f| f.bite) { txt("!", sw / 2. - 10., sh / 2. - 80., 90., Color::new(1., 0.3, 0.2, 1.)) }
    let narrow = sw < 900.;
    if narrow { draw_rectangle(sw - 62., 14., 50., 30., if g.me.id == 0 { Color::new(0.2, 0.45, 0.9, 0.9) } else { Color::new(0.9, 0.35, 0.3, 0.9) }); txt(&format!("P{}", g.me.id + 1), sw - 52., 38., 26., WHITE) }
    if let Some(a) = &g.act { if g.me.scene == Scene::Islet && a.players.contains(&g.me.id) { let (n, obj, _, th) = ACTS[a.kind]; let bw = 600f32.min(sw - 20.); let (x0, y0) = (sw / 2. - bw / 2., sh - 134.);
        draw_rectangle(x0, y0, bw, 92., Color::new(0., 0.15, 0.25, 0.75)); let mine = a.mine.iter().find(|m| m.0 == g.me.id).map_or(0, |m| m.1);
        txt(&format!("{n}   {:.0}s left   team score {}{}", a.left.ceil(), a.score, if a.done { "   - finished!" } else { "" }), x0 + 12., y0 + 26., 24., WHITE);
        txt(obj, x0 + 12., y0 + 52., 17., Color::new(0.8, 0.95, 1., 1.));
        txt(&format!("Bronze {} / Silver {} / Gold {}  ->  {} / {} / {} Sea Glass   you: {mine}{}", th[0], th[1], th[2], ACT_REWARD[0], ACT_REWARD[1], ACT_REWARD[2], if a.players.len() > 1 { "   (team)" } else { "" }), x0 + 12., y0 + 78., 17., GOLD) } }
    let hint = if g.me.mplace.is_some() { "Placing project: walk/face a spot  Enter place  Esc cancel" } else if g.swimming() { if g.me.pad { "Swimming:  X dive  stick swim" } else { "Swimming:  Space dive  WASD swim" } } else if g.me.pad { if g.me.placing.is_some() { "Stick move  RB rotate  A place  B cancel" } else { if narrow { "Stick move  A act  X tool  RB tool  Y bag  B back" } else { "Stick move  RT run  A act  X tool  RB next tool  Y bag  Start journal  B cancel" } } } else if narrow { if g.me.placing.is_some() { "Move  R rotate  Enter place  Esc cancel" } else { "Move  E act  Space tool  Tab tool  I bag  J journal" } } else if g.me.placing.is_some() { "Placing:  WASD/Arrows move   R rotate   Enter place   Esc cancel" } else { "WASD move  Shift run  E interact  Space use tool  Tab tool  I inventory  J journal  F/G move/pick up furniture  Esc cancel" };
    draw_rectangle(4., sh - 34., measure_text(hint, None, 19, 1.).width + 24., 28., Color::new(0., 0., 0., 0.4)); txt(hint, 12., sh - 14., 19., WHITE);
    if let Some(s) = g.w.p.seek.as_ref().filter(|s| g.seek_live(s)) {
        let bw = (sw - 24.).min(460.); draw_rectangle(12., 186., bw, 50., Color::new(0.1, 0.24, 0.16, 0.85));
        txt(&format!("Trail Seek: {}/3 found   {}s left", s.found.len(), (s.deadline_ms - g.clock_ms + 999) / 1000), 22., 207., 21., WHITE);
        txt("Talk to friends at their hiding spots.", 22., 228., 17., WHITE);
    }
    match &g.me.ui {
        Ui::Play => {}
        Ui::Invitation => {
            let pw = (sw - 24.).min(620.); let x = (sw - pw) / 2.; panel(x, 180., pw, 270.);
            txt("An invitation", x + 18., 216., 26., BLACK);
            if let Some(v) = &g.w.p.invitation { let s = if v.kind == 0 { format!("{} invites you home today, {}:00-{}:00.", g.res(v.slot).name, v.hour, v.hour + 1) } else { format!("{} proposes Trail Seek: find three friends behind trees or buildings in four minutes.", g.res(v.slot).name) };
                for (i, l) in wrap_txt(&s, ((pw - 36.) / 11.) as usize).iter().enumerate() { txt(l, x + 18., 258. + i as f32 * 25., 21., BLACK); }
                txt(if v.kind == 0 { "A friendly visit, with a small gift." } else { "They walk to their spots. Nothing lost on timeout." }, x + 18., 350., 19., DARKGRAY);
            }
            txt(if g.me.pad { "A accept   B decline" } else { "Enter accept   Esc decline" }, x + 18., 420., 20., DARKGRAY);
        }
        Ui::Contest(sel) => {
            let pw = (sw - 24.).min(620.); let x = (sw - pw) / 2.;
            panel(x, 95., pw, 490.);
            txt(g.contest_kind().map_or("Survey", |k| crate::events::CONTEST_NAMES[k]), x + 16., 128., 26., BLACK);
            let best = g.contest_entry().map_or(0, |e| e.best);
            txt(&format!("Best: {best} points. Entries close 17:00."), x + 16., 158., 19., DARKGRAY);
            let items: Vec<String> = g.contest_slots().iter().map(|&i| { let s = &g.w.p.inv[i]; format!("{} / {} pt", slot_name(s), crate::events::catch_score(s.def, s.size_mm)) }).collect();
            if items.is_empty() { txt("No measured catches of today's kind.", x + 16., 195., 19., DARKGRAY); }
            list(&items, Some(*sel), x + 16., 195.);
            txt("Improved entries are released. Others return.", x + 16., 539., 18., DARKGRAY);
            txt(if g.me.pad { "Up/Down choose   A enter   B close" } else { "W/S choose   Enter enter catch   Esc close" }, x + 16., 570., 18., DARKGRAY);
        }
        Ui::Talk { who, lines, i, .. } => { let pw = (sw - 24.).min(860.); let x0 = sw / 2. - pw / 2.; panel(x0, sh - 200., pw, 140.); txt(who, x0 + 20., sh - 168., 26., Color::new(0.55, 0.3, 0.15, 1.)); for (k, l) in wrap_txt(lines.get(*i).map_or("", |s| s.as_str()), ((pw - 40.) / 12.5) as usize).iter().take(2).enumerate() { txt(l, x0 + 20., sh - 128. + k as f32 * 26., 24., BLACK) } if let Ui::Talk { npc: Some(j), .. } = g.me.ui { let mv = g.w.moving.get(j).is_some_and(|&m| m != 0); txt(&format!("{}{}", if g.me.pad { "[LB] give gift" } else { "[G] give gift" }, if mv { if g.me.pad { "   [Y] ask to stay" } else { "   [X] ask to stay" } } else { "" }), x0 + 20., sh - 75., 20., GRAY) }
            txt(if g.me.pad { "[A] next" } else { "[E] next" }, x0 + pw - 100., sh - 72., 20., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Inv(s) | Ui::Sell(s) => { let sell = matches!(g.me.ui, Ui::Sell(_)); panel(sw / 2. - 260., 90., 520., 520.);
            txt(if sell { "Sell to Mr. Tumble" } else { "Pockets" }, sw / 2. - 240., 125., 30., Color::new(0.45, 0.3, 0.2, 1.));
            let mut items: Vec<String> = g.w.p.inv.iter().map(|it| if sell { format!("{}  -  {}c", slot_name(it), def(it.def).sell * it.qty as u32) } else { slot_name(it) }).collect();
            if sell { let lv = g.w.p.home as usize; items.push(if g.w.p.debt > 0 { format!("Pay home balance  ({} left)", g.w.p.debt) } else if lv < EXPAND_COST.len() { format!("{}  (adds {}c balance)", ["Expand your home", "Expand your home", "Dig a basement", "Build an upstairs"][lv], EXPAND_COST[lv]) } else { "Home fully expanded".into() }); items.push(format!("Order from catalog ({} items)", g.w.p.catalog.len())); items.push("Customize furniture / wallpaper / flooring".into()); items.push("Seeds, watering can and tool upgrades".into()) }
            if items.is_empty() { txt("(empty)", sw / 2. - 230., 170., 22., GRAY) } list(&items, Some(*s), sw / 2. - 230., 170.);
            txt(match (sell, g.me.pad) { (true, false) => "W/S select  Enter sell  A sell all finds  Esc close", (true, true) => "Up/Down select  A sell  Y sell all finds  B close", (false, false) => "W/S select  Enter place furniture (at home)  Esc close", _ => "Up/Down select  A place furniture (at home)  B close" }, sw / 2. - 240., 590., 19., DARKGRAY);
            if !sell { txt(&format!("Tools: {}", g.w.p.tools.iter().map(|t| def(*t).name).collect::<Vec<_>>().join(", ")), sw / 2. - 240., 565., 19., DARKGRAY) } }
        Ui::Confirm(d) => { let (dd, p, _) = g.display_item(*d); let n = def(dd).name;
            panel(sw / 2. - 260., sh / 2. - 80., 520., 150.); txt(&format!("Buy the {n} for {p} coins?"), sw / 2. - 235., sh / 2. - 35., 26., BLACK); txt(&format!("You have {} coins.", g.w.p.money), sw / 2. - 235., sh / 2. - 2., 22., DARKGRAY); txt(if g.me.pad { "[A] Yes     [B] No" } else { "[Enter] Yes     [Esc] No" }, sw / 2. - 235., sh / 2. + 40., 22., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Storage(c, s) => { panel(sw / 2. - 380., 90., 760., 520.); txt("Pockets", sw / 2. - 360., 125., 28., BLACK); txt("Cabinet storage", sw / 2. + 20., 125., 28., BLACK);
            let a: Vec<String> = g.w.p.inv.iter().map(slot_name).collect(); let bb: Vec<String> = g.w.p.storage.iter().map(slot_name).collect();
            list(&a, (*c == 0).then_some(*s), sw / 2. - 350., 165.); list(&bb, (*c == 1).then_some(*s), sw / 2. + 30., 165.); txt(if g.me.pad { "Left/Right switch side  Up/Down select  A move item  B close" } else { "A/D switch side  W/S select  Enter move item  Esc close" }, sw / 2. - 360., 590., 19., DARKGRAY) }
        Ui::Gift(j, sel) => { panel(sw / 2. - 260., 100., 520., 440.); txt(&format!("Give {} a gift", g.res(*j).name), sw / 2. - 240., 135., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = g.w.p.inv.iter().map(slot_name).collect(); list(&items, Some(*sel), sw / 2. - 230., 175.);
            txt(if g.me.pad { "Up/Down select  A give  B cancel" } else { "W/S select  Enter give  Esc cancel" }, sw / 2. - 240., 525., 19., DARKGRAY) }
        Ui::MPick(sel) => { panel(sw / 2. - 270., 100., 540., 330.); txt("Start a funded public project", sw / 2. - 250., 135., 26., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = MPROJECTS.iter().map(|p| format!("{}  -  {} coins", p.0, p.1)).collect(); list(&items, Some(*sel), sw / 2. - 240., 175.);
            txt("Pick one; you head outside to choose the spot (green = valid).", sw / 2. - 250., 330., 18., DARKGRAY); txt("Built the morning after it is fully funded. Esc cancels.", sw / 2. - 250., 355., 18., DARKGRAY) }
        Ui::Keeper(sel) => { let items = g.keeper_items(); let pw = (sw - 40.).min(720.); let x0 = sw / 2. - pw / 2.; panel(x0, 80., pw, 120. + items.len() as f32 * 28.); txt("Mr. Tumble  -  seeds, watering can, tool upgrades", x0 + 20., 115., 22., Color::new(0.45, 0.3, 0.2, 1.));
            let labels: Vec<String> = items.iter().map(|x| x.0.clone()).collect(); list(&labels, Some(*sel), x0 + 30., 145.); txt(&format!("You have {} coins.  {}", g.w.p.money, if g.me.pad { "[A] Choose  [B] Leave" } else { "[Enter] Choose  [Esc] Leave" }), x0 + 20., 120. + items.len() as f32 * 28. + 60., 17., DARKGRAY) }
        Ui::Paint(sel) => { panel(sw / 2. - 280., 110., 560., 320.); txt(&format!("Repaint your house  -  {PAINT_FEE} coins (walls & roof)"), sw / 2. - 260., 145., 21., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = PAINTS.iter().enumerate().map(|(k, p)| format!("{}{}", p.0, if g.w.p.paint == Some(k as u8) { "  (current)" } else { "" })).collect(); list(&items, Some(*sel), sw / 2. - 250., 180.);
            for (k, p) in PAINTS.iter().enumerate() { let y = 168. + k as f32 * 26.; draw_rectangle(sw / 2. + 150., y, 30., 18., rgb(p.1)); draw_rectangle(sw / 2. + 184., y, 30., 18., rgb(p.2)) }
            txt(if g.me.pad { "[A] Paint  [B] Back" } else { "[Enter] Paint  [Esc] Back" }, sw / 2. - 260., 405., 19., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Camp(sel) => { let v = &VISITORS[g.visitor()]; panel(sw / 2. - 300., 100., 600., 330.); txt(&format!("{} the {}  -  campground", v.name, v.species), sw / 2. - 280., 135., 24., Color::new(0.45, 0.3, 0.2, 1.));
            let (fd, fq) = g.favor_item(); let done = g.w.p.camp_favors.contains(&(g.visitor() as u8));
            let mut items: Vec<String> = v.goods.iter().map(|&d| format!("{}  -  {} coins{}", def(d).name, def(d).buy, if g.w.camp_sold.contains(&d) { "  (sold out today)" } else { "" })).collect();
            items.push(if done { "Favor: done - thank you!".into() } else { format!("Favor: bring {fq} {}", def(fd).name) }); items.push("Chat".into()); list(&items, Some(*sel), sw / 2. - 270., 170.);
            txt(&format!("You have {} coins. One of each item per day for the whole town.", g.w.p.money), sw / 2. - 280., 360., 17., DARKGRAY); txt(if g.me.pad { "[A] Choose   [B] Leave" } else { "[Enter] Choose   [Esc] Leave" }, sw / 2. - 280., 400., 20., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Kiosk(sel) => { let pw = (sw - 40.).min(700.); let x0 = sw / 2. - pw / 2.; panel(x0, 70., pw, 470.); txt(&format!("Initiatives kiosk  -  {} coupons", g.w.p.coupons), x0 + 20., 105., 24., Color::new(0.45, 0.3, 0.2, 1.));
            let mut items: Vec<String> = (0..3).map(|k| g.w.p.tasks.get(k).map_or("(no task)".to_string(), |t| { let (_, n, text, c) = TASKS[t.t]; format!("Claim: {text} {}/{n} -> {c}{}", t.prog, if t.claimed { " (claimed)" } else { "" }) })).collect();
            items.push(g.w.wtask.as_ref().map_or("(no weekly task)".into(), |w| { let (_, n, text) = WTASKS[w.t]; format!("Claim weekly: {text} {}/{n} -> {WEEKLY_COUPONS}{}", w.prog, if w.claimed.contains(&g.w.p.id) { " (claimed)" } else { "" }) }));
            for &(d, c) in &KIOSK_ITEMS { items.push(format!("Reward: {}  -  {c} coupons", def(d).name)) } list(&items, Some(*sel), x0 + 30., 135.);
            txt(if g.me.pad { "[A] Claim / trade   [B] Close" } else { "[Enter] Claim / trade   [Esc] Close" }, x0 + 20., 525., 19., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Title(sel) => { draw_rectangle(0., 0., sw, sh, Color::new(0.12, 0.2, 0.16, 0.92)); txt("Hollowbrook", sw / 2. - 150., sh * 0.28, 64., Color::new(0.95, 0.9, 0.7, 1.)); txt(VERSION, sw / 2. - 150., sh * 0.28 + 34., 18., Color::new(0.8, 0.85, 0.8, 1.));
            let items = vec![if g.fresh { "Start your town".to_string() } else { "Continue".into() }, "Host a LAN game".into(), "Join a LAN game".into(), "Controls and guide".into(), "Quit".into()]; list(&items, Some(*sel), sw / 2. - 140., sh * 0.42);
            txt("Your town saves automatically (every 15 s, each new day, and on quit).", sw / 2. - 300., sh * 0.75, 18., Color::new(0.85, 0.9, 0.85, 1.));
            txt(if g.me.pad { "Stick: choose   A: confirm" } else { "W/S: choose   Enter: confirm" }, sw / 2. - 300., sh * 0.75 + 26., 18., Color::new(0.85, 0.9, 0.85, 1.)) }
        Ui::Help => { draw_rectangle(0., 0., sw, sh, Color::new(0.12, 0.2, 0.16, 0.95)); let mut y = 70.; txt("Controls", 60., y, 30., Color::new(0.95, 0.9, 0.7, 1.)); y += 34.; let wcol = Color::new(0.92, 0.95, 0.92, 1.);
            for l in HELP_KEYS { txt(l, 70., y, 17., wcol); y += 20. } y += 8.; txt("Gamepad", 60., y + 4., 22., Color::new(0.95, 0.9, 0.7, 1.)); y += 26.; for l in HELP_PAD { txt(l, 70., y, 17., wcol); y += 20. } y += 10.; txt("Guide", 60., y, 26., Color::new(0.95, 0.9, 0.7, 1.)); y += 26.;
            'g: for l in GUIDE.iter().take(5) { for w in wrap(l, ((sw - 140.) / 9.) as usize) { if y > sh - 56. { txt("More in the journal's Guide page.", 70., y, 16., GOLD); break 'g } txt(&w, 70., y, 16., wcol); y += 19. } } txt(if g.me.pad { "[A]/[B] back" } else { "[Enter]/[Esc] back" }, 60., sh - 24., 18., GOLD) }
        Ui::Civic(sel) => { panel(sw / 2. - 280., 120., 560., 260.); txt(ADMIN, sw / 2. - 260., 155., 24., Color::new(0.45, 0.3, 0.2, 1.)); txt("Welcome to the Town Hall. How can I help?", sw / 2. - 260., 182., 18., DARKGRAY);
            let items = vec!["Town policies".to_string(), if g.me.id == LEADER { "Start a funded public project".into() } else { "Start a funded public project (leader only)".into() }, "Project and town status".into(), format!("Repaint your house ({PAINT_FEE} coins)")]; list(&items, Some(*sel), sw / 2. - 250., 210.);
            txt(if g.me.pad { "[A] Choose   [B] Leave" } else { "[Enter] Choose   [Esc] Leave" }, sw / 2. - 260., 355., 20., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Policy(sel) => { panel(sw / 2. - 330., 80., 660., 400.); txt(ADMIN, sw / 2. - 310., 115., 24., Color::new(0.45, 0.3, 0.2, 1.));
            txt(&format!("Civic desk. Current policy: {}. Only one policy can be active.", g.pol().map_or("none", |p| p.name)), sw / 2. - 310., 145., 18., DARKGRAY);
            let mut items: Vec<String> = POLICIES.iter().map(|p| format!("{}  -  {} coins", p.name, p.fee)).collect(); items.push("Repeal current policy (free)".into()); list(&items, Some(*sel), sw / 2. - 300., 180.);
            let e = POLICIES.get(*sel).map_or("Return to normal hours and maintenance.", |p| p.effect); for (k, l) in wrap(e, 70).iter().enumerate() { txt(l, sw / 2. - 310., 360. + k as f32 * 22., 18., BLACK) }
            if g.me.id != LEADER { txt("Only the town leader can change policies.", sw / 2. - 310., 455., 18., Color::new(0.6, 0.2, 0.2, 1.)) } }
        Ui::PolicyOk(sel) => { panel(sw / 2. - 330., sh / 2. - 110., 660., 210.); let (t, fee, e) = POLICIES.get(*sel).map_or(("Repeal current policy", 0, "Return to normal hours and maintenance."), |p| (p.name, p.fee, p.effect));
            txt(&format!("{t}  -  fee {fee} coins (you have {})", g.w.p.money), sw / 2. - 310., sh / 2. - 75., 22., BLACK); for (k, l) in wrap(e, 70).iter().enumerate() { txt(l, sw / 2. - 310., sh / 2. - 40. + k as f32 * 22., 18., DARKGRAY) }
            txt(if g.me.pad { "[A] Confirm   [B] Back" } else { "[Enter] Confirm   [Esc] Back" }, sw / 2. - 310., sh / 2. + 70., 22., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Act(sel) => { panel(sw / 2. - 320., 90., 640., 370.); txt("Tidekeeper's board - timed activities", sw / 2. - 300., 125., 24., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = ACTS.iter().map(|a| format!("{}  -  {:.0} seconds", a.0, a.2)).collect(); list(&items, Some(*sel), sw / 2. - 290., 160.);
            let a = ACTS[*sel]; for (k, l) in wrap(a.1, 66).iter().enumerate() { txt(l, sw / 2. - 300., 290. + k as f32 * 22., 18., BLACK) }
            txt(&format!("Team score {} / {} / {} earns {} / {} / {} Sea Glass each (once per activity per day).", a.3[0], a.3[1], a.3[2], ACT_REWARD[0], ACT_REWARD[1], ACT_REWARD[2]), sw / 2. - 300., 345., 17., DARKGRAY);
            txt("Everyone on Gullrock joins and shares the score; a player in town is unaffected.", sw / 2. - 300., 370., 17., DARKGRAY);
            txt(if g.me.pad { "[A] Start   [B] Close" } else { "[Enter] Start   [Esc] Close" }, sw / 2. - 300., 430., 22., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Fund => { if let Some(m) = &g.w.mproject { panel(sw / 2. - 260., sh / 2. - 80., 520., 150.); txt(&format!("{}: {} / {} coins", MPROJECTS[m.idx].0, m.funded, MPROJECTS[m.idx].1), sw / 2. - 240., sh / 2. - 40., 24., BLACK);
            txt(&format!("You have {} coins. Each press donates up to {}.", g.w.p.money, DONATE_STEP), sw / 2. - 240., sh / 2. - 6., 20., DARKGRAY); txt(if g.me.pad { "[A] Donate   [B] Close" } else { "[Enter] Donate   [Esc] Close" }, sw / 2. - 240., sh / 2. + 36., 22., Color::new(0.45, 0.3, 0.2, 1.)) } }
        Ui::ConfirmExpand => { let lv = g.w.p.home as usize; let (a, b) = (HOME_DIMS[lv], HOME_DIMS[(lv + 1).min(HOME_DIMS.len() - 1)]); panel(sw / 2. - 270., sh / 2. - 90., 540., 170.);
            txt(&if lv >= 2 { format!("{} for your home", if lv == 2 { "Dig a cosy basement" } else { "Build a bright upstairs" }) } else { format!("Expand your home: {}x{} -> {}x{} floor", a.0, a.1, b.0, b.1) }, sw / 2. - 245., sh / 2. - 45., 24., BLACK); txt(&format!("Adds a {} coin balance. No interest, no deadline.", EXPAND_COST[lv.min(EXPAND_COST.len() - 1)]), sw / 2. - 245., sh / 2. - 12., 20., DARKGRAY);
            txt("Pay it off at the counter to unlock the next expansion.", sw / 2. - 245., sh / 2. + 14., 20., DARKGRAY); txt(if g.me.pad { "[A] Expand   [B] Cancel" } else { "[Enter] Expand   [Esc] Cancel" }, sw / 2. - 245., sh / 2. + 52., 22., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::Travel(out) => { panel(sw / 2. - 270., sh / 2. - 80., 540., 150.); txt(if *out { "Sail to Gullrock Islet?" } else { "Sail back to Hollowbrook?" }, sw / 2. - 245., sh / 2. - 35., 26., BLACK);
            txt(&if *out { format!("Fare: {FARE} coins (you have {}).", g.w.p.money) } else { "The trip home is free.".into() }, sw / 2. - 245., sh / 2. - 2., 22., DARKGRAY); txt(if g.me.pad { "[A] Sail     [B] Stay" } else { "[Enter] Sail     [Esc] Stay" }, sw / 2. - 245., sh / 2. + 40., 22., Color::new(0.45, 0.3, 0.2, 1.)) }
        Ui::IStall(sel) => { panel(sw / 2. - 260., 120., 520., 260.); txt("Sea Glass stall", sw / 2. - 240., 155., 28., Color::new(0.2, 0.5, 0.5, 1.));
            let items: Vec<String> = ISTALL_ITEMS.iter().map(|&(d, c)| format!("{}  -  {} Sea Glass", def(d).name, c)).collect(); list(&items, Some(*sel), sw / 2. - 230., 195.);
            txt(&format!("You have {} Sea Glass.", g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>()), sw / 2. - 240., 320., 20., DARKGRAY); txt(if g.me.pad { "Up/Down select  A trade  B close" } else { "W/S select  Enter trade  Esc close" }, sw / 2. - 240., 350., 19., DARKGRAY) }
        Ui::Tint(sel) => { panel(sw / 2. - 270., 100., 540., 440.); txt("Customize", sw / 2. - 250., 135., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let mut items: Vec<String> = g.tint_targets().iter().map(|&i| { let s = g.w.p.inv[i]; format!("{} ({})  -  {}c", def(s.def).name, TINT_NAMES[Game::tint_of(&g.w.p, s.uid) as usize], TINT_FEE) }).collect();
            items.push(format!("Wallpaper (style {})  -  {}c", g.w.p.wall + 1, DECOR_FEE)); items.push(format!("Flooring (style {})  -  {}c", g.w.p.floor + 1, DECOR_FEE));
            list(&items, Some(*sel), sw / 2. - 240., 175.); txt(if g.me.pad { "Up/Down select  A change  B close" } else { "W/S select  Enter change  Esc close" }, sw / 2. - 250., 525., 19., DARKGRAY) }
        Ui::Catalog(sel) => { panel(sw / 2. - 260., 100., 520., 440.); txt("Catalog", sw / 2. - 240., 135., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = g.w.p.catalog.iter().map(|&d| format!("{}  -  {}c", def(d).name, def(d).buy)).collect(); if items.is_empty() { txt("Items you own appear here.", sw / 2. - 230., 175., 22., GRAY) }
            list(&items[..items.len().min(12)], Some(*sel), sw / 2. - 230., 175.); txt(if g.me.pad { "Up/Down select  A order  B close" } else { "W/S select  Enter order  Esc close" }, sw / 2. - 240., 525., 19., DARKGRAY) }
        Ui::Mail(sel) => { panel(sw / 2. - 280., 100., 560., 440.); txt("Mailbox", sw / 2. - 260., 135., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = g.w.p.mail.iter().rev().map(|l| format!("{}{}{}", if l.read { "   " } else { "* " }, l.from, if !l.claimed && (l.coins > 0 || l.item.is_some()) { "  (gift)" } else { "" })).collect();
            if items.is_empty() { txt("No letters yet.", sw / 2. - 250., 175., 22., GRAY) } list(&items[..items.len().min(8)], Some(*sel), sw / 2. - 250., 175.);
            if let Some(l) = g.w.p.mail.iter().rev().nth(*sel) { for (k, line) in wrap_txt(&l.text, 44).iter().enumerate() { txt(line, sw / 2. - 250., 400. + k as f32 * 24., 21., BLACK) } }
            txt(if g.me.pad { "Up/Down select  A open/claim  B close" } else { "W/S select  Enter open/claim  Esc close" }, sw / 2. - 260., 525., 19., DARKGRAY) }
        Ui::Rack(sel) => { panel(sw / 2. - 260., 120., 520., 380.); txt("Clothing rack", sw / 2. - 240., 155., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let mut items: Vec<String> = CLOTHES.iter().map(|&d| format!("{}  -  {} coins", def(d).name, def(d).buy)).collect(); items.push(format!("Change hairstyle  -  {} coins", HAIR_FEE)); list(&items, Some(*sel), sw / 2. - 230., 195.);
            txt(&format!("You have {} coins.", g.w.p.money), sw / 2. - 240., 440., 20., DARKGRAY); txt(if g.me.pad { "Up/Down select  A buy  B close" } else { "W/S select  Enter buy  Esc close" }, sw / 2. - 240., 470., 19., DARKGRAY) }
        Ui::Journal => { panel(60., 80., sw - 120., sh - 160.); txt(["Field Journal", "Initiatives", "Settings", "Controls", "Guide", "Friends"][g.me.jpage as usize % 6], 80., 115., 30., Color::new(0.45, 0.3, 0.2, 1.));
            txt(&format!("Page {}/6   {}   {}", g.me.jpage + 1, if g.me.pad { "[RB] next page" } else { "[Tab] next page" }, VERSION), 80., 140., 16., DARKGRAY);
            if g.me.jpage == 0 {
            for (col, (k, title)) in [(K::Fish, "Fish & sea"), (K::Bug, "Bugs"), (K::Shell, "Shells & finds")].iter().enumerate().filter(|(c, _)| sw >= 1100. || *c == g.me.jcol as usize) { let x = if sw >= 1100. { 80. + col as f32 * 380. } else { 80. }; txt(title, x, 170., 24., BLACK);
                for (j, dd) in DEFS.iter().filter(|dd| dd.k == *k || (*k == K::Shell && matches!(dd.k, K::Treasure | K::Fruit | K::Fossil)) || (*k == K::Fish && dd.k == K::Sea)).enumerate() { let seen = g.w.p.seen.contains(&dd.id);
                    txt(&format!("{}{}", if avail(dd, g.mo, g.hr) && matches!(dd.k, K::Fish | K::Bug) { "* " } else { "  " }, if seen { format!("{}  {}c{}", dd.name, dd.sell, if g.w.donated.iter().any(|x| x.0 == dd.id) { "  [museum]" } else { "" }) } else { "???".into() }), x, 194. + j as f32 * ((sh - 340.) / 19.).min(24.), 18., if seen { BLACK } else { GRAY }) } }
            txt("* = active right now", 80., sh - 100., 18., DARKGRAY); txt(&format!("Town liveliness: {} ({})", g.liveliness(), Game::live_word(g.liveliness())), 80., sh - 125., 20., Color::new(0.3, 0.5, 0.3, 1.));
                if sw < 1100. { txt(if g.me.pad { "stick left/right: other columns" } else { "Left/Right: other columns" }, 80., sh - 150., 18., DARKGRAY) } }
            if g.me.jpage == 2 {
            txt(&format!("Music {:.0}%{}  ({})    Sound effects {:.0}%  ({})    {} mute music", g.music_vol * 100., if g.muted { " (muted)" } else { "" }, if g.me.pad { "stick left/right" } else { "Left/Right" }, g.sfx_vol * 100., if g.me.pad { "stick up/down" } else { "Up/Down" }, if g.me.pad { "[Y]" } else { "[X]" }), 80., 170., 20., Color::new(0.3, 0.3, 0.55, 1.)); txt("Music and sound settings are saved for this computer.", 80., 205., 18., DARKGRAY) }
            if g.me.jpage == 1 { let mut y = 160.; txt(&format!("Coupons: {}  (kiosk in the Town Hall)", g.w.p.coupons), 80., y + 8., 20., Color::new(0.3, 0.3, 0.55, 1.)); y += 34.;
                txt("Today's tasks (end at 6:00):", 80., y, 22., BLACK); y += 28.;
                for t in &g.w.p.tasks { let (_, n, text, c) = TASKS[t.t]; txt(&format!("{}  {}  {}/{}  -  {} coupons", if t.claimed { "[claimed]" } else if t.prog >= n { "[done!]" } else { "[ ]" }, text, t.prog, n, c), 90., y, 19., if t.claimed { GRAY } else { BLACK }); y += 26. }
                if g.w.p.tasks.is_empty() { txt("No tasks yet; they arrive at the next new day.", 90., y, 19., GRAY); y += 26. }
                if let Some(w) = &g.w.wtask { let (_, n, text) = WTASKS[w.t]; y += 10.; txt("Weekly town task (shared, ends Mon 6:00):", 80., y, 22., BLACK); y += 28.;
                    txt(&format!("{}  {}/{}  -  {} coupons each{}", text, w.prog, n, WEEKLY_COUPONS, if w.claimed.contains(&g.w.p.id) { "  [claimed]" } else if w.prog >= n { "  [done!]" } else { "" }), 90., y, 19., BLACK) } }
            if g.me.jpage == 5 { let mut y = 168.; for k in 0..4 { let r = g.res(k); let ch = g.w.roster[k]; let f = g.w.p.npcs[k].friend; let t = tier(f);
                    txt(&format!("{} the {} ({})", r.name, r.species, PERS_NAME[r.pers]), 80., y, 21., BLACK); y += 22.;
                    txt(&format!("{}  {}   Birthday {}/{}   Loves {}   Likes {:?} gifts", TIERS[t], "*".repeat(t + 1), BDAYS[ch].1, BDAYS[ch].0, HOBBY[ch], LIKES[r.pers]), 95., y, 16., Color::new(0.35, 0.3, 0.25, 1.)); y += 18.;
                    let bw = (sw - 260.).min(360.); draw_rectangle(95., y, bw, 8., Color::new(0.85, 0.82, 0.75, 1.)); draw_rectangle(95., y, bw * (f.clamp(0, 60) as f32 / 60.), 8., Color::new(0.9, 0.45, 0.5, 1.)); y += 24.;
                    if g.w.p.keepsakes.contains(&(ch as u8)) { txt("Keepsake portrait received", 95., y - 6., 15., Color::new(0.4, 0.55, 0.4, 1.)); y += 12. } }
                let (sc, tips) = Game::home_score(&g.w.p); txt(&format!("Your home: {sc} points ({})  -  best rating rewarded: {}", HOME_TIERS[Game::home_tier(sc)].1, HOME_TIERS[g.w.p.home_best as usize].1), 80., y + 6., 19., Color::new(0.3, 0.3, 0.55, 1.)); if let Some(t) = tips.first() { txt(&format!("Tip: {t}"), 95., y + 30., 16., DARKGRAY) } }
            if g.me.jpage == 3 { let mut y = 168.; txt("Keyboard", 80., y, 22., BLACK); y += 26.; for l in HELP_KEYS { txt(l, 90., y, 17., BLACK); y += 21. } y += 8.; txt("Gamepad", 80., y, 22., BLACK); y += 26.; for l in HELP_PAD { txt(l, 90., y, 17., BLACK); y += 21. } }
            if g.me.jpage == 4 { let mut y = 195.; let wch = ((sw - 180.) / 9.5).max(20.) as usize; let topic = (g.me.jcol as usize).min(GUIDE.len() - 1);
                txt(&format!("Topic {}/{}   Up/Down: topics", topic + 1, GUIDE.len()), 80., 170., 18., DARKGRAY);
                for w in wrap(GUIDE[topic], wch) { txt(&w, 80., y, 17., BLACK); y += 22.; }
            } }
    }
    if let Some(src) = dbg {
        let lines = [format!("[{src} clock] {}", now.format("%a %Y-%m-%d %H:%M:%S")), format!("gameplay day {} last processed {}", crate::clock::day_id(now), g.w.day),
            format!("scene {:?} pos ({:.2},{:.2}) curve {}", g.me.scene, g.ppos().x, g.ppos().y, g.curve), format!("target {:?}", g.target()),
            format!("stock {:?}", g.w.stock.iter().map(|s| (def(s.def).name, s.sold)).collect::<Vec<_>>())];
        let (px, py, pw) = if narrow { (10., 150., sw - 20.) } else { (sw - 560., 10., 550.) };
        panel(px, py, pw, 30. + 22. * (lines.len() + g.log.len().min(3)) as f32);
        for (k, l) in lines.iter().chain(g.log.iter().rev().take(3)).enumerate() { txt(l, px + 12., py + 22. + k as f32 * 22., 17., BLACK) }
    }
    if g.me.pad_lost { for (k, m) in ["Controller disconnected", "Press Start on a controller to reconnect"].iter().enumerate() { let tw = measure_text(m, None, 24, 1.).width; draw_rectangle(sw / 2. - tw / 2. - 12., sh / 2. - 34. + k as f32 * 40., tw + 24., 40., Color::new(0.6, 0.1, 0.1, 0.9)); txt(m, sw / 2. - tw / 2., sh / 2. - 6. + k as f32 * 40., 24., WHITE) } }
    if let Some((t, _, _)) = g.me.trans { draw_rectangle(0., 0., sw, sh, Color::new(0., 0., 0., 1. - ((t - 0.22).abs() / 0.22).min(1.))) }
}

fn wrap(t: &str, n: usize) -> Vec<String> { let mut v = vec![String::new()]; for w in t.split(' ') { if v.last().unwrap().len() + w.len() + 1 > n { v.push(String::new()) } let l = v.last_mut().unwrap(); if !l.is_empty() { l.push(' ') } l.push_str(w) } v }
