//! Models: furniture, paintings, buildings, project sites, palms.
use super::*;

pub fn furniture(r: &mut R, d: u16, base: Mat4, on: bool, tint: u8) {
    let (c0, k) = (def(d).col, TINTS[tint as usize % TINTS.len()]);
    let c = Color::new((c0.0 * k.0).min(1.), (c0.1 * k.1).min(1.), (c0.2 * k.2).min(1.), 1.);
    r.with(base, |r| {
        match model(d) {
            50 => {
                r.cube(vec3(0., 0.4, 0.), vec3(0.3, 0.05, 0.3), c);
                for (x, z) in [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)] {
                    r.cube(vec3(x, 0.18, z), vec3(0.04, 0.18, 0.04), sh(c, 0.8))
                }
                r.cube(vec3(0., 0.72, -0.27), vec3(0.3, 0.28, 0.04), c)
            }
            51 => {
                r.cube(vec3(0., 0.22, 0.), vec3(0.95, 0.18, 0.4), c);
                for x in [-0.47, 0.47] {
                    r.cube(vec3(x, 0.45, 0.05), vec3(0.44, 0.07, 0.33), sh(c, 1.15))
                }
                r.cube(vec3(0., 0.6, -0.32), vec3(0.95, 0.25, 0.09), c);
                for x in [-0.92, 0.92] {
                    r.cube(vec3(x, 0.45, 0.), vec3(0.07, 0.15, 0.4), sh(c, 0.85))
                }
            }
            52 => {
                r.cyl(vec3(0., 0.68, 0.), 0.42, 0.06, c);
                r.cyl(Vec3::ZERO, 0.06, 0.68, sh(c, 0.8));
                r.cyl(Vec3::ZERO, 0.25, 0.04, sh(c, 0.8))
            }
            53 => {
                r.cube(vec3(0., 0.18, 0.), vec3(0.45, 0.14, 0.95), WOOD);
                r.cube(vec3(0., 0.36, 0.05), vec3(0.42, 0.06, 0.88), WHITE);
                r.cube(vec3(0., 0.4, 0.3), vec3(0.44, 0.05, 0.62), c);
                r.cube(vec3(0., 0.45, -0.7), vec3(0.32, 0.06, 0.15), WHITE);
                r.cube(vec3(0., 0.5, -0.96), vec3(0.45, 0.4, 0.04), WOOD)
            }
            54 => {
                r.cyl(Vec3::ZERO, 0.18, 0.05, WOOD);
                r.cyl(vec3(0., 0.05, 0.), 0.03, 0.8, WOOD);
                let u = r.unlit;
                r.unlit = on;
                r.cone(vec3(0., 0.75, 0.), 0.27, 0.34, if on { Color::new(1., 0.93, 0.62, 1.) } else { c });
                r.unlit = u
            }
            55 => {
                r.cube(vec3(0., 0.55, 0.), vec3(0.42, 0.55, 0.35), c);
                r.cube(vec3(0., 0.55, 0.352), vec3(0.01, 0.5, 0.005), sh(c, 0.6));
                for x in [-0.08, 0.08] {
                    r.sphere(vec3(x, 0.6, 0.37), Vec3::splat(0.03), GOLD)
                }
            }
            56 => {
                r.cyl(Vec3::ZERO, 0.22, 0.32, Color::new(0.8, 0.45, 0.3, 1.));
                for (x, y, z, s) in [(0., 0.6, 0., 0.25), (0.15, 0.48, 0.08, 0.17), (-0.14, 0.5, -0.05, 0.18)] {
                    r.sphere(vec3(x, y, z), Vec3::splat(s), c)
                }
            }
            57 => {
                r.cube(vec3(0., 1.35, 0.03), vec3(0.36, 0.28, 0.03), WOOD);
                r.cube(vec3(0., 1.35, 0.065), vec3(0.3, 0.22, 0.005), c);
                r.cube(vec3(0., 1.24, 0.072), vec3(0.3, 0.11, 0.003), Color::new(0.35, 0.65, 0.35, 1.));
                r.sphere(vec3(0.15, 1.45, 0.07), Vec3::splat(0.05), YELLOW)
            }
            58 => {
                r.cyl(vec3(0., 0., 0.), 0.25, 0.38, sh(c, 0.85));
                r.cyl(vec3(0., 0.38, 0.), 0.3, 0.06, c)
            }
            59 => {
                r.cyl(Vec3::ZERO, 0.2, 0.08, WOOD);
                r.sphere(vec3(0., 0.42, 0.), Vec3::splat(0.32), Color::new(c.r, c.g, c.b, 0.85));
                for k in 0..6 {
                    let a = k as f32;
                    r.seg(
                        vec3(a.cos() * 0.3, 0.3, a.sin() * 0.3),
                        vec3(a.cos() * 0.15, 0.7, a.sin() * 0.15),
                        0.01,
                        WOOD,
                    )
                }
            }
            _ => r.cube(vec3(0., 0.3, 0.), vec3(0.3, 0.3, 0.3), c),
        }
        match d {
            // collection accents
            90 => {
                for x in [-0.2f32, 0., 0.2] {
                    r.sphere(
                        vec3(x, 0.47, 0.22),
                        Vec3::splat(0.05),
                        [PINK, YELLOW, WHITE][((x + 0.2) * 5.) as usize % 3],
                    )
                }
            }
            91 => {
                r.cyl(vec3(0., 0.74, 0.), 0.06, 0.12, Color::new(0.4, 0.6, 0.85, 1.));
                for k in 0..3 {
                    r.sphere(vec3(-0.03 + k as f32 * 0.03, 0.9, 0.), Vec3::splat(0.04), [PINK, YELLOW, WHITE][k])
                }
            }
            92 => {
                for k in 0..5 {
                    let a = k as f32 * 1.256;
                    r.sphere(vec3(a.cos() * 0.18, 0.06, a.sin() * 0.18), Vec3::splat(0.04), PINK)
                }
            }
            93 | 94 => r.cube(
                vec3(0., 0.28, 0.),
                vec3(if d == 93 { 0.97 } else { 0.43 }, 0.025, if d == 93 { 0.42 } else { 0.36 }),
                Color::new(0.85, 0.8, 0.65, 1.),
            ),
            95 => r.cube(vec3(0., 0.72, 0.), vec3(0.22, 0.02, 0.22), Color::new(0.75, 0.6, 0.3, 1.)),
            _ => {}
        }
    });
}
/// Original procedural painting for art id `d`, drawn on a wall plane centred at `p` (facing +z).
pub fn painting(r: &mut R, d: u16, p: Vec3, w: f32, h: f32) {
    let k = (d - 100) as usize;
    let (sky, ground, acc) = [
        ((1., 0.82, 0.6), (0.45, 0.7, 0.35), (1., 0.95, 0.6)),
        ((0.6, 0.85, 0.95), (0.95, 0.88, 0.65), (0.2, 0.55, 0.75)),
        ((0.85, 0.9, 1.), (0.5, 0.72, 0.4), (0.98, 0.65, 0.75)),
        ((0.1, 0.12, 0.3), (0.2, 0.25, 0.3), (1., 0.85, 0.4)),
        ((0.7, 0.88, 0.95), (0.3, 0.6, 0.35), (0.3, 0.55, 0.85)),
        ((0.75, 0.8, 0.88), (0.95, 0.97, 1.), (0.35, 0.45, 0.4)),
        ((0.85, 0.8, 0.9), (0.4, 0.55, 0.3), (0.75, 0.3, 0.5)),
        ((0.78, 0.8, 0.82), (0.45, 0.55, 0.62), (0.55, 0.35, 0.25)),
    ][k];
    r.cube(p, vec3(w / 2. + 0.05, h / 2. + 0.05, 0.02), Color::new(0.55, 0.4, 0.2, 1.));
    let z = p.z + 0.025;
    r.cube(vec3(p.x, p.y + h * 0.15, z), vec3(w / 2., h * 0.35, 0.003), rgb(sky));
    r.cube(vec3(p.x, p.y - h * 0.35, z), vec3(w / 2., h * 0.15, 0.003), rgb(ground));
    let z2 = z + 0.004;
    match k {
        0 => {
            r.sphere(vec3(p.x + w * 0.25, p.y + h * 0.2, z2), vec3(h * 0.12, h * 0.12, 0.002), rgb(acc));
            for i in 0..3 {
                r.cube(
                    vec3(p.x - w * 0.3 + i as f32 * w * 0.2, p.y - h * 0.12, z2),
                    vec3(w * 0.06, h * 0.08, 0.002),
                    Color::new(0.9, 0.85, 0.75, 1.),
                )
            }
        }
        1 => {
            r.sphere(vec3(p.x, p.y - h * 0.05, z2), vec3(w * 0.22, h * 0.14, 0.002), Color::new(0.55, 0.5, 0.48, 1.));
            r.cube(vec3(p.x, p.y - h * 0.2, z2), vec3(w / 2., h * 0.04, 0.002), rgb(acc))
        }
        2 => {
            for i in 0..4 {
                let x = p.x - w * 0.35 + i as f32 * w * 0.23;
                r.cube(vec3(x, p.y - h * 0.12, z2), vec3(0.012, h * 0.12, 0.002), Color::new(0.4, 0.25, 0.15, 1.));
                r.sphere(vec3(x, p.y + h * 0.05, z2), vec3(w * 0.09, h * 0.1, 0.002), rgb(acc))
            }
        }
        3 => {
            for i in 0..5 {
                let x = p.x - w * 0.4 + i as f32 * w * 0.2;
                r.sphere(vec3(x, p.y + h * (0.1 + (i % 2) as f32 * 0.12), z2), vec3(0.025, 0.035, 0.002), rgb(acc))
            }
        }
        6 => {
            for i in 0..9 {
                let x = p.x - w * 0.4 + (i % 5) as f32 * w * 0.2;
                let y = p.y - h * (0.3 - (i / 5) as f32 * 0.12);
                r.cube(vec3(x, y + h * 0.06, z2), vec3(0.008, h * 0.06, 0.002), Color::new(0.3, 0.5, 0.25, 1.));
                r.sphere(vec3(x, y + h * 0.13, z2 + 0.001), vec3(0.03, 0.03, 0.001), [rgb(acc), YELLOW, WHITE][i % 3])
            }
        }
        7 => {
            r.cube(vec3(p.x + w * 0.1, p.y - h * 0.12, z2), vec3(w * 0.18, h * 0.05, 0.002), rgb(acc));
            r.cube(vec3(p.x + w * 0.12, p.y + h * 0.02, z2), vec3(0.01, h * 0.12, 0.002), rgb(acc));
            for i in 0..3 {
                r.cube(
                    vec3(p.x, p.y + h * (0.05 + i as f32 * 0.1), z2 + 0.001),
                    vec3(w / 2., h * 0.03, 0.001),
                    Color::new(1., 1., 1., 0.35),
                )
            }
        }
        4 => {
            r.sphere(vec3(p.x, p.y - h * 0.28, z2), vec3(w * 0.3, h * 0.08, 0.002), rgb(acc));
            r.sphere(
                vec3(p.x - w * 0.1, p.y - h * 0.27, z2 + 0.001),
                vec3(0.03, 0.02, 0.001),
                Color::new(0.3, 0.75, 0.3, 1.),
            )
        }
        _ => {
            for i in 0..3 {
                r.cone(vec3(p.x - w * 0.3 + i as f32 * w * 0.3, p.y - h * 0.2, z2), 0.06, h * 0.35, rgb(acc))
            }
            for i in 0..8 {
                r.sphere(
                    vec3(
                        p.x - w * 0.4 + (i * 37 % 10) as f32 * w * 0.09,
                        p.y + h * (0.05 + (i * 13 % 7) as f32 * 0.04),
                        z2 + 0.001,
                    ),
                    Vec3::splat(0.012),
                    WHITE,
                )
            }
        }
    }
}
pub fn fbase(f: &Furn) -> Mat4 {
    let c = if def(f.def).wall { vec2(f.x as f32 + 0.5, 0.) } else { fcenter(f) };
    Mat4::from_rotation_translation(Quat::from_rotation_y(f.rot as f32 * FRAC_PI_2), vec3(c.x, 0., c.y))
}
pub(super) fn building(r: &mut R, bi: usize, night: f32, paint: Option<((f32, f32, f32), (f32, f32, f32))>) {
    let b = &BLDS[bi];
    let (x, z, w, d) = (b.x as f32, b.z as f32, b.w as f32, b.d as f32);
    let civic = b.kind == BK::Civic;
    let hg = if civic { 2.0 } else { 1.5 };
    let (wall, roof) = paint.map_or((rgb(b.wall), rgb(b.roof)), |(w, r)| (rgb(w), rgb(r)));
    let c = vec3(x + w / 2., 0., z + d / 2.);
    // soft contact shadow: two stacked translucent rings fading out from the walls
    for (grow, a) in [(0.55, 0.1), (0.28, 0.12)] {
        r.quad(
            vec3(x - grow, 0.004, z - grow * 0.6),
            vec3(x + w + grow, 0.004, z - grow * 0.6),
            vec3(x + w + grow, 0.004, z + d + grow),
            vec3(x - grow, 0.004, z + d + grow),
            Vec3::Y,
            Color::new(0.05, 0.06, 0.04, a),
        );
    }
    r.cube(c + vec3(0., 0.08, 0.), vec3(w / 2. + 0.04, 0.08, d / 2. + 0.04), Color::new(0.62, 0.6, 0.57, 1.));
    r.cube(c + vec3(0., hg / 2., 0.), vec3(w / 2. - 0.05, hg / 2., d / 2. - 0.05), wall);
    let (sid, trim) = (sh(wall, 0.9), sh(wall, 1.12));
    let n = (hg / 0.22) as i32;
    for k in 1..n {
        let y = k as f32 * 0.22;
        r.cube(vec3(c.x, y, z + d - 0.045), vec3(w / 2. - 0.05, 0.012, 0.006), sid);
        for sx in [x + 0.045, x + w - 0.045] {
            r.cube(vec3(sx, y, c.z), vec3(0.006, 0.012, d / 2. - 0.05), sid)
        }
    }
    for (cx, cz) in
        [(x + 0.05, z + d - 0.05), (x + w - 0.05, z + d - 0.05), (x + 0.05, z + 0.05), (x + w - 0.05, z + 0.05)]
    {
        r.cube(vec3(cx, hg / 2., cz), vec3(0.05, hg / 2., 0.05), trim)
    }
    let (rhw, rhd, rh) = (w / 2. + 0.25, d / 2. + 0.3, if civic { 1.0 } else { 0.85 });
    r.roof(c + vec3(0., hg, 0.), rhw, rhd, rh, roof);
    r.cube(c + vec3(0., hg + rh + 0.015, 0.), vec3(rhw + 0.02, 0.035, 0.05), sh(roof, 0.72)); // ridge beam
                                                                                              // shingle courses: darker strips laid parallel to the ridge on both slopes, just above the surface
                                                                                              // (skipped while the house is faded out of the camera's way, so no stripes cross the player)
    for sz in if r.alpha > 0.99 { [-1f32, 1.].as_slice() } else { [].as_slice() } {
        let sz = *sz;
        let nrm = vec3(0., rhd, sz * rh).normalize();
        for k in 1..5 {
            let f = k as f32 / 5.;
            let p = c + vec3(0., hg + rh * f, sz * rhd * (1. - f)) + nrm * 0.006;
            let q = c + vec3(0., hg + rh * (f + 0.035), sz * rhd * (1. - f - 0.035)) + nrm * 0.006;
            r.quad(
                p - vec3(rhw, 0., 0.),
                p + vec3(rhw, 0., 0.),
                q + vec3(rhw, 0., 0.),
                q - vec3(rhw, 0., 0.),
                nrm,
                sh(roof, 0.82),
            );
        }
    }
    if night > 0.4 && !civic {
        // lamplight from the front windows spilling onto the ground in the evening
        r.disc(vec3(c.x, 0.012, z + d + 0.75), vec2(w * 0.45, 0.6), Color::new(1., 0.82, 0.45, 0.14 * night));
    }
    // Fascia beneath the existing overhang: the roof surface and silhouette stay intact.
    for sz in [-1f32, 1.] {
        r.cube(c + vec3(0., hg - 0.03, sz * (d / 2. + 0.26)), vec3(w / 2. + 0.24, 0.025, 0.04), trim);
    }
    let dx = door_face(bi).x;
    let fz = z + d - 0.05;
    r.cube(vec3(dx, 0.52, fz), vec3(0.3, 0.52, 0.06), Color::new(0.45, 0.28, 0.18, 1.));
    r.sphere(vec3(dx + 0.18, 0.5, fz + 0.07), Vec3::splat(0.04), GOLD);
    r.cube(vec3(dx, 0.03, fz + 0.18), vec3(0.42, 0.03, 0.16), Color::new(0.74, 0.72, 0.68, 1.)); // doorstep
    r.cube(vec3(dx, 0.064, fz + 0.19), vec3(0.28, 0.006, 0.11), sh(roof, 0.85)); // doormat
    let lit = night > 0.35;
    let win = if lit { Color::new(1., 0.85, 0.5, 1.) } else { Color::new(0.68, 0.84, 0.95, 1.) };
    for wx in [x + 0.5, x + w - 0.5] {
        r.cube(vec3(wx, 0.98, fz), vec3(0.24, 0.24, 0.05), WHITE);
        let u = r.unlit;
        r.unlit = lit;
        r.cube(vec3(wx, 0.98, fz + 0.012), vec3(0.19, 0.19, 0.05), win);
        r.unlit = u;
        // Narrow painted dividers sit above the glass face (fz + 0.062).
        let wz = fz + 0.076;
        r.quad(
            vec3(wx - 0.012, 0.79, wz),
            vec3(wx + 0.012, 0.79, wz),
            vec3(wx + 0.012, 1.17, wz),
            vec3(wx - 0.012, 1.17, wz),
            Vec3::Z,
            WHITE,
        );
        r.quad(
            vec3(wx - 0.19, 0.968, wz + 0.002),
            vec3(wx + 0.19, 0.968, wz + 0.002),
            vec3(wx + 0.19, 0.992, wz + 0.002),
            vec3(wx - 0.19, 0.992, wz + 0.002),
            Vec3::Z,
            WHITE,
        );
        r.cube(vec3(wx, 0.735, fz + 0.045), vec3(0.27, 0.025, 0.1), WHITE);
        if !civic {
            r.cube(vec3(wx, 0.72, fz + 0.12), vec3(0.22, 0.05, 0.055), WOOD); // window flower box
            r.cube(vec3(wx, 0.775, fz + 0.12), vec3(0.2, 0.025, 0.04), Color::new(0.28, 0.62, 0.28, 1.));
            for (fk, fc) in [PINK, YELLOW, WHITE].iter().enumerate() {
                r.sphere(vec3(wx - 0.12 + fk as f32 * 0.12, 0.815, fz + 0.125), Vec3::splat(0.036), *fc);
            }
        }
    }
    match b.kind {
        BK::Shop => {
            for k in 0..7 {
                r.cube(
                    vec3(x + 0.4 + k as f32 * (w - 0.8) / 6., hg * 0.82, fz + 0.32),
                    vec3((w - 0.8) / 14., 0.04, 0.3),
                    if k % 2 == 0 { WHITE } else { Color::new(0.2, 0.6, 0.6, 1.) },
                )
            }
            r.cube(vec3(c.x, hg + 0.15, fz + 0.1), vec3(0.8, 0.22, 0.05), WOOD)
        }
        BK::Museum => {
            for k in 0..4 {
                r.cyl(
                    vec3(x + 0.45 + k as f32 * (w - 0.9) / 3., 0.16, fz + 0.3),
                    0.1,
                    hg - 0.16,
                    Color::new(0.97, 0.96, 0.92, 1.),
                )
            }
            r.cube(vec3(c.x, hg - 0.05, fz + 0.3), vec3(w / 2., 0.08, 0.14), Color::new(0.97, 0.96, 0.92, 1.))
        }
        BK::Civic => {
            r.cube(vec3(c.x, hg + 1.15, c.z), vec3(0.38, 0.55, 0.38), wall);
            r.roof(vec3(c.x, hg + 1.7, c.z), 0.48, 0.48, 0.5, roof);
            r.cylinder_clock(c.x, hg + 1.25, c.z + 0.39);
            r.seg(vec3(c.x, hg + 2.2, c.z), vec3(c.x, hg + 2.9, c.z), 0.02, DARKGRAY);
            r.cube(vec3(c.x + 0.2, hg + 2.75, c.z), vec3(0.2, 0.12, 0.01), Color::new(0.9, 0.3, 0.3, 1.));
        }
        _ => r.cube(c + vec3(w / 2. - 0.6, hg + 0.65, -0.25), vec3(0.13, 0.32, 0.13), Color::new(0.65, 0.35, 0.3, 1.)),
    }
}
impl R {
    pub(super) fn cylinder_clock(&mut self, x: f32, y: f32, z: f32) {
        self.with(Mat4::from_translation(vec3(x, y, z)) * Mat4::from_rotation_x(FRAC_PI_2), |r| {
            r.cyl(Vec3::ZERO, 0.22, 0.02, WHITE);
            r.cube(vec3(0., 0.03, -0.08), vec3(0.015, 0.01, 0.09), DARK)
        })
    }
}
/// Community project models (also drawn translucent as a site preview).
pub(super) fn draw_project(r: &mut R, kind: u8, x: i32, z: i32, night: f32, ghost: bool) {
    let (px, pz) = (x as f32 + 0.5, z as f32 + 0.5);
    match kind {
        0 => {
            r.cube(vec3(px, 0.38, pz), vec3(0.45, 0.05, 0.2), WOOD);
            r.cube(vec3(px, 0.62, pz - 0.18), vec3(0.45, 0.16, 0.03), WOOD);
            for xx in [-0.38, 0.38] {
                r.cube(vec3(px + xx, 0.19, pz), vec3(0.04, 0.19, 0.18), DARKGRAY)
            }
        }
        1 => {
            r.cyl(vec3(px, 0., pz), 0.06, 1.6, Color::new(0.25, 0.27, 0.3, 1.));
            let lit = night > 0.3 && !ghost;
            let u = r.unlit;
            r.unlit = lit;
            r.cube(
                vec3(px, 1.72, pz),
                vec3(0.14, 0.16, 0.14),
                if lit { Color::new(1., 0.88, 0.55, 1.) } else { Color::new(0.85, 0.85, 0.75, 1.) },
            );
            r.unlit = u;
            r.roof(vec3(px, 1.88, pz), 0.18, 0.18, 0.12, Color::new(0.25, 0.27, 0.3, 1.));
            if lit {
                r.disc(vec3(px, 0.025, pz), vec2(1.3, 1.3), Color::new(1., 0.85, 0.45, 0.22))
            }
        }
        2 => {
            r.cube(vec3(px, 0.22, pz), vec3(0.42, 0.22, 0.42), Color::new(0.62, 0.6, 0.58, 1.));
            r.cube(vec3(px, 0.45, pz), vec3(0.38, 0.02, 0.38), Color::new(0.4, 0.28, 0.18, 1.));
            for (k, (ox, oz)) in [(-0.2, -0.2), (0.2, -0.15), (0., 0.1), (-0.15, 0.22), (0.22, 0.2)].iter().enumerate()
            {
                r.sphere(vec3(px + ox, 0.58, pz + oz), Vec3::splat(0.09), [RED, YELLOW, PINK, PURPLE, WHITE][k])
            }
        }
        4 => {
            r.cyl(vec3(px, 0., pz), 0.48, 0.3, Color::new(0.7, 0.7, 0.72, 1.));
            r.cyl(vec3(px, 0.3, pz), 0.4, 0.01, Color::new(0.35, 0.65, 0.9, 1.));
            r.cyl(vec3(px, 0.3, pz), 0.08, 0.6, Color::new(0.7, 0.7, 0.72, 1.));
            r.sphere(vec3(px, 0.95, pz), Vec3::splat(0.1), Color::new(0.6, 0.85, 1., 1.))
        }
        5 => {
            for s in [-0.4f32, 0.4] {
                r.cyl(vec3(px + s, 0., pz), 0.05, 1.4, Color::new(0.95, 0.95, 0.9, 1.))
            }
            for k in 0..9 {
                let a = k as f32 / 8. * PI;
                r.sphere(
                    vec3(px - a.cos() * 0.4, 1.4 + a.sin() * 0.3, pz),
                    Vec3::splat(0.09),
                    [PINK, WHITE, YELLOW][k % 3],
                )
            }
        }
        _ => {
            for k in 0..2 {
                let bx = px + k as f32;
                r.cube(vec3(bx, 0.06, pz), vec3(0.5, 0.06, 0.48), Color::new(0.65, 0.62, 0.58, 1.));
                for zz in [pz - 0.45, pz + 0.45] {
                    r.cube(vec3(bx, 0.35, zz), vec3(0.5, 0.05, 0.04), Color::new(0.55, 0.52, 0.5, 1.));
                    r.cube(vec3(bx - 0.4, 0.2, zz), vec3(0.05, 0.2, 0.05), Color::new(0.55, 0.52, 0.5, 1.))
                }
            }
        }
    }
}
pub(super) fn palm(r: &mut R, x: f32, z: f32, t: f32) {
    let sway = (t * 0.8 + x).sin() * 0.04;
    for k in 0..6 {
        let y = k as f32 * 0.3;
        r.cyl(
            vec3(x + k as f32 * 0.06 + sway * k as f32, y, z),
            0.11 - k as f32 * 0.008,
            0.3,
            Color::new(0.6, 0.45, 0.3, 1.),
        )
    }
    let top = vec3(x + 0.36 + sway * 6., 1.8, z);
    for k in 0..6 {
        let a = k as f32 / 6. * 2. * PI;
        r.with(Mat4::from_translation(top) * Mat4::from_rotation_y(a) * Mat4::from_rotation_z(-0.35), |r| {
            r.cube(vec3(0.42, 0., 0.), vec3(0.42, 0.02, 0.1), Color::new(0.25, 0.6, 0.3, 1.))
        })
    }
    for k in 0..3 {
        let a = k as f32 * 2.1;
        r.sphere(top + vec3(a.cos() * 0.12, -0.12, a.sin() * 0.12), Vec3::splat(0.08), Color::new(0.45, 0.32, 0.2, 1.))
    }
}
