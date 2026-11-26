//! Spaceflight and the two landing sites: the flight view (stars, the planet with its rings or caps, your ship),
//! the light-speed tunnel, the nav computer, and the Glacia / Verdance scenes.
use super::*;

/// Planet look in flight: (surface, band, atmosphere).
const PLANET_LOOK: [((f32, f32, f32), (f32, f32, f32), (f32, f32, f32)); 3] = [
    ((0.86, 0.64, 0.42), (0.7, 0.46, 0.3), (1., 0.85, 0.65)),
    ((0.78, 0.9, 1.), (0.55, 0.72, 0.92), (0.8, 0.92, 1.)),
    ((0.25, 0.6, 0.4), (0.15, 0.42, 0.32), (0.6, 1., 0.8)),
];

fn space_light() -> Light {
    Light {
        sky: Color::new(0.02, 0.02, 0.06, 1.),
        sun: vec3(-0.55, 0.65, 0.5).normalize(),
        amb: vec3(0.28, 0.28, 0.34),
        dif: vec3(0.9, 0.86, 0.8),
        night: 0.,
    }
}

/// Your ship in flight: the same patched freighter, legs tucked, engines glowing with thrust. Forward is -z.
fn flier(r: &mut R, p: Vec3, yaw: f32, thrust: f32, t: f32) {
    let hull = Color::new(0.62, 0.64, 0.66, 1.);
    let rust = Color::new(0.55, 0.36, 0.24, 1.);
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(-yaw), p), |r| {
        r.sphere(Vec3::ZERO, vec3(0.75, 0.3, 1.25), hull);
        r.sphere(vec3(0., 0.12, -0.25), vec3(0.32, 0.12, 0.5), rust);
        r.sphere(vec3(0., 0.12, -1.0), vec3(0.3, 0.2, 0.32), Color::new(0.16, 0.2, 0.26, 1.)); // cockpit
        r.cube(vec3(0., 0., 0.2), vec3(1.25, 0.04, 0.35), sh(hull, 0.85)); // wings
        for sx in [-0.42, 0.42] {
            r.with(Mat4::from_rotation_translation(Quat::from_rotation_x(FRAC_PI_2), vec3(sx, 0., 0.9)), |r| {
                r.cyl(vec3(0., -0.4, 0.), 0.2, 0.8, sh(hull, 0.85))
            });
            let u = r.unlit;
            r.unlit = true;
            let k = 0.55 + 0.45 * thrust.min(1.) + 0.08 * (t * 20. + sx).sin();
            r.sphere(
                vec3(sx, 0., 1.32 + 0.25 * thrust),
                vec3(0.17, 0.17, 0.2 + 0.35 * thrust),
                Color::new(0.45 * k, 0.78 * k, 1. * k, 1.),
            );
            r.unlit = u;
        }
    });
}

/// The flight view, drawn in place of the ground scene while flying.
pub(super) fn space_view(g: &Game, f: &Flight, vp: Option<(f32, f32)>) {
    hud_cam(vp);
    let (w, h, t) = (vw(), vh(), g.t);
    let space = Color::new(0.02, 0.02, 0.06, 1.);
    if vp.is_some() {
        draw_rectangle(0., 0., w, h, space)
    } else {
        clear_background(space) // also clears depth, or last frame's depth hides the planet and ship
    }
    if f.jump > 0. {
        // the light-speed tunnel: stars stretch into streaks rushing past, then a flash on arrival
        let p = 1. - f.jump / JUMP_TIME;
        let (cx, cy) = (w / 2., h / 2.);
        let maxr = (w * w + h * h).sqrt() * 0.6;
        draw_circle(cx, cy, h * (0.05 + 0.1 * p), Color::new(0.75, 0.88, 1., 0.25));
        for i in 0..240u32 {
            let a = (hash(i as i32, 3) % 6283) as f32 / 1000.;
            let base = (hash(i as i32, 9) % 1000) as f32 / 1000.;
            let speed = 0.6 + (i % 5) as f32 * 0.25;
            let r0 = ((base + t * speed * (0.4 + 2.2 * p)) % 1.).powf(1.6) * maxr;
            let len = maxr * (0.02 + 0.3 * p.min(0.8)) * (r0 / maxr + 0.1);
            let (dx, dy) = (a.cos(), a.sin());
            let col = if i % 4 == 0 { Color::new(0.65, 0.8, 1., 0.9) } else { Color::new(0.95, 0.97, 1., 0.85) };
            draw_line(cx + dx * r0, cy + dy * r0, cx + dx * (r0 + len), cy + dy * (r0 + len), 1.2 + p * 1.5, col);
        }
        if f.jump < 0.25 {
            draw_rectangle(0., 0., w, h, Color::new(1., 1., 1., (0.25 - f.jump) * 3.));
        }
        let msg = format!("Light speed  -  {}", PLANETS[f.dest as usize].0);
        let tw = measure_text(&msg, None, 26, 1.).width;
        txt(&msg, w / 2. - tw / 2., h - 70., 26., Color::new(0.85, 0.92, 1., 1.));
        return;
    }
    // star field with slight parallax as you fly
    for i in 0..180u32 {
        let layer = 0.3 + (i % 3) as f32 * 0.35;
        let x = ((hash(i as i32, 11) % 4000) as f32 - f.pos.x * 14. * layer).rem_euclid(w);
        let y = ((hash(i as i32, 17) % 4000) as f32 - f.pos.y * 14. * layer).rem_euclid(h);
        let tw = 0.6 + 0.4 * (t * 2. + i as f32).sin();
        draw_circle(x, y, 0.7 + layer, Color::new(0.85, 0.9, 1., 0.45 + 0.4 * tw * layer));
    }
    let cam = Camera3D {
        position: vec3(f.pos.x, 15., f.pos.y + 12.),
        target: vec3(f.pos.x, 0., f.pos.y - 1.),
        up: Vec3::Y,
        fovy: 45f32.to_radians(),
        viewport: vp.map(|(x, w)| (x as i32, 0, w as i32, vh() as i32)),
        aspect: Some(vw() / vh()),
        ..Default::default()
    };
    set_camera(&cam);
    let lt = space_light();
    let mut r = R::new(0., Vec2::ZERO, &lt, (300., 600.));
    let (surf, band, atmo) = PLANET_LOOK[f.sys as usize];
    let c = vec3(0., -2.5, 0.);
    r.sphere(c, Vec3::splat(PLANET_R), rgb(surf));
    for k in 0..3 {
        // darker bands / continents as slightly raised caps at different latitudes
        let y = -0.6 + k as f32 * 0.55;
        let rr = (PLANET_R * PLANET_R - (y * PLANET_R * 0.6).powi(2)).sqrt();
        r.sphere(c + vec3(0., y * PLANET_R * 0.6, 0.), vec3(rr * 1.005, 0.22, rr * 1.005), rgb(band));
    }
    if f.sys == 1 {
        r.sphere(c + vec3(0., PLANET_R * 0.86, 0.), vec3(PLANET_R * 0.55, 0.8, PLANET_R * 0.55), WHITE);
        // polar cap
    }
    if f.sys == 2 {
        let u = r.unlit;
        r.unlit = true;
        for k in 0..9 {
            let a = k as f32 * 0.7 + t * 0.05;
            r.sphere(
                c + vec3(a.cos() * PLANET_R * 0.7, PLANET_R * 0.68, a.sin() * PLANET_R * 0.7),
                Vec3::splat(0.18),
                Color::new(0.5, 1., 0.75, 1.),
            ); // glowing forests
        }
        r.unlit = u;
    }
    let (a0, u0) = (r.alpha, r.unlit);
    r.alpha = 0.1;
    r.unlit = true;
    r.sphere(c, Vec3::splat(PLANET_R * 1.07), rgb(atmo)); // soft atmosphere glow (unlit, so it brightens)
    r.alpha = a0;
    r.unlit = u0;
    if f.sys == 0 {
        for k in 0..72 {
            let (a, b) = (k as f32 / 72. * TAU, (k + 1) as f32 / 72. * TAU);
            for (rad, col) in
                [(PLANET_R * 1.5, Color::new(0.95, 0.85, 0.7, 1.)), (PLANET_R * 1.62, Color::new(0.85, 0.72, 0.58, 1.))]
            {
                r.seg(
                    c + vec3(a.cos() * rad, 0., a.sin() * rad),
                    c + vec3(b.cos() * rad, 0., b.sin() * rad),
                    0.12,
                    col,
                );
            }
        }
    }
    let ma = t * 0.12 + f.sys as f32;
    r.sphere(vec3(ma.cos() * 13., -1., ma.sin() * 13.), Vec3::splat(0.9), Color::new(0.7, 0.7, 0.74, 1.)); // a moon
    flier(&mut r, vec3(f.pos.x, 0.6, f.pos.y), f.yaw, f.thrust, t);
    r.flush();
    hud_cam(vp);
    let near = f.pos.length() < PLANET_R + 4.;
    let lines = [
        format!("{}  -  {}", PLANETS[f.sys as usize].0, PLANETS[f.sys as usize].1),
        if near { "Press E to land".into() } else { String::new() },
    ];
    for (k, l) in lines.iter().enumerate().filter(|(_, l)| !l.is_empty()) {
        let size = if k == 0 { 24. } else { 20. };
        let tw = measure_text(l, None, size as u16, 1.).width;
        draw_rectangle(
            w / 2. - tw / 2. - 10.,
            h - 104. + k as f32 * 30. - size + 2.,
            tw + 20.,
            size + 8.,
            Color::new(0., 0., 0., 0.45),
        );
        txt(l, w / 2. - tw / 2., h - 104. + k as f32 * 30., size, if k == 1 && near { GOLD } else { WHITE });
    }
}

/// The nav computer: pick a destination for the light-speed jump.
pub(super) fn nav_menu(g: &Game, sel: usize, sw: f32) {
    let pw = (sw - 40.).min(620.);
    let x0 = sw / 2. - pw / 2.;
    panel(x0, 90., pw, 290.);
    txt("Nav computer", x0 + 20., 125., 24., Color::new(0.45, 0.3, 0.2, 1.));
    let here = g.me.flight.map_or(0, |f| f.sys as usize);
    let mut items: Vec<String> = PLANETS
        .iter()
        .enumerate()
        .map(|(k, (n, d))| if k == here { format!("{n}  (you are here)") } else { format!("{n}  -  {d}") })
        .collect();
    items.push("Cancel".into());
    list(&items, Some(sel), x0 + 30., 175.);
    txt("W/S choose   Enter jump to light speed   Esc cancel", x0 + 30., 355., 18., Color::new(0.45, 0.3, 0.2, 1.));
}

/// Cool blue light for Glacia, teal twilight for Verdance.
pub(super) fn planet_light(lt: &Light, k: u8) -> Light {
    let (sky, amb, dif) = if k == 1 {
        (Color::new(0.72, 0.84, 0.97, 1.), vec3(0.95, 1.0, 1.1), vec3(0.95, 1.0, 1.08))
    } else {
        (Color::new(0.22, 0.32, 0.38, 1.), vec3(0.72, 0.95, 0.9), vec3(0.75, 0.9, 0.85))
    };
    let n = 1. - lt.night * 0.6;
    Light {
        sky: Color::new(sky.r * n, sky.g * n, sky.b * n, 1.),
        sun: lt.sun,
        amb: lt.amb * amb,
        dif: lt.dif * dif,
        night: lt.night,
    }
}

/// A crystal spire (Glacia) or a giant glowing mushroom (Verdance) on a ROCK tile.
fn prop(r: &mut R, k: u8, x: i32, z: i32, t: f32) {
    let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5);
    let hsh = hash(x, z);
    if k == 1 {
        let ice = Color::new(0.62, 0.82, 1., 1.);
        for j in 0..3 {
            let a = j as f32 * 2.1 + hsh as f32;
            let h = 0.9 + ((hsh >> j) % 4) as f32 * 0.35;
            r.cone(p + vec3(a.cos() * 0.18, 0., a.sin() * 0.18), 0.17, h, sh(ice, 0.9 + j as f32 * 0.07));
        }
    } else {
        let h = 1.0 + (hsh % 3) as f32 * 0.35;
        r.cyl(p, 0.11, h, Color::new(0.85, 0.82, 0.7, 1.));
        let u = r.unlit;
        r.unlit = true;
        let glow = 0.75 + 0.25 * (t * 1.5 + x as f32).sin();
        let cap = [Color::new(0.35, 0.95 * glow, 0.8 * glow, 1.), Color::new(0.75 * glow, 0.45, 0.95 * glow, 1.)]
            [(hsh % 2) as usize];
        r.sphere(p + vec3(0., h, 0.), vec3(0.5, 0.2, 0.5), cap);
        r.unlit = u;
    }
}

pub(super) fn planet_scene(r: &mut R, g: &Game, k: u8, lt: &Light, labels: &mut Vec<(Vec3, String, Color)>) {
    let t = g.t;
    let cam = g.me.cam;
    let me = g.me.ipos;
    let (ground, path, cliff) = if k == 1 {
        (Color::new(0.92, 0.95, 1., 1.), Color::new(0.78, 0.84, 0.94, 1.), Color::new(0.82, 0.9, 0.98, 1.))
    } else {
        (Color::new(0.24, 0.5, 0.36, 1.), Color::new(0.4, 0.42, 0.3, 1.), Color::new(0.12, 0.3, 0.22, 1.))
    };
    let mut faded: Vec<(i32, i32, bool)> = vec![];
    for z in -9..PH + 7 {
        for x in -12..PW + 12 {
            let (fx, fz) = (x as f32, z as f32);
            if (fx - cam.x).powi(2) + (fz - cam.y).powi(2) > 480. {
                continue;
            }
            let c = ptile(k, x, z);
            let base = if c == PATH { path } else { ground };
            let col = |dx: i32, dz: i32| gtint(base, gnoise(x + dx, z + dz));
            r.quad4(
                [vec3(fx, 0., fz), vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.)],
                Vec3::Y,
                [col(0, 0), col(1, 0), col(1, 1), col(0, 1)],
            );
            let inside = (1..PW - 1).contains(&x) && (1..PH - 1).contains(&z);
            let (sx, sz, sw, sd) = P_SHIP;
            let ship_tile = x >= sx && x < sx + sw && z >= sz && z < sz + sd;
            if c == ROCK && !ship_tile {
                let blocks = (x as f32 + 0.5 - me.x).abs() < 1.0 && fz + 0.5 > me.y + 0.2 && fz + 0.5 < me.y + 2.8;
                if blocks {
                    faded.push((x, z, inside));
                } else if inside {
                    prop(r, k, x, z, t);
                } else {
                    border(r, k, x, z, cliff, t);
                }
            } else if c == ROCK && !inside {
                border(r, k, x, z, cliff, t);
            } else if c != ROCK && hash(x * 3, z * 7) % 6 == 0 {
                // ground detail: ice shards in the snow, glowing moss tufts on Verdance
                let p = vec3(fx + 0.3 + (hash(x, z) % 5) as f32 * 0.1, 0., fz + 0.5);
                if k == 1 {
                    r.cone(p, 0.05, 0.14, Color::new(0.7, 0.86, 1., 1.));
                } else {
                    let u = r.unlit;
                    r.unlit = true;
                    r.sphere(p + vec3(0., 0.04, 0.), vec3(0.08, 0.04, 0.08), Color::new(0.4, 0.9, 0.6, 1.));
                    r.unlit = u;
                }
            }
            if (x < 0 || z < 0 || x >= PW || z >= PH) && z < PH {
                border(r, k, x, z, cliff, t);
            }
        }
    }
    // the pond: frozen on Glacia, glowing on Verdance
    let (pcx, pcz, prx, prz) = P_POND[(k as usize).clamp(1, 2) - 1];
    if k == 1 {
        r.disc(vec3(pcx, 0.01, pcz), vec2(prx + 0.3, prz + 0.3), Color::new(0.72, 0.86, 0.98, 1.));
        r.disc(vec3(pcx, 0.016, pcz), vec2(prx * 0.8, prz * 0.8), Color::new(0.58, 0.78, 0.95, 1.));
        for j in 0..5 {
            let a = j as f32 * 1.3;
            r.seg(vec3(pcx, 0.02, pcz), vec3(pcx + a.cos() * prx * 0.8, 0.02, pcz + a.sin() * prz * 0.8), 0.015, WHITE);
            // cracks
        }
    } else {
        let u = r.unlit;
        r.unlit = true;
        r.disc(vec3(pcx, 0.01, pcz), vec2(prx + 0.3, prz + 0.3), Color::new(0.1, 0.35, 0.3, 1.));
        let glow = 0.8 + 0.2 * (t * 1.2).sin();
        r.disc(
            vec3(pcx, 0.016, pcz),
            vec2(prx * 0.85, prz * 0.85),
            Color::new(0.2 * glow, 0.75 * glow, 0.7 * glow, 1.),
        );
        r.unlit = u;
    }
    // your ship on the landing pad
    let (sx, sz, sw, sd) = P_SHIP;
    let behind_ship = me.y < (sz + sd) as f32 + 0.2 && me.x > sx as f32 - 0.5 && me.x < (sx + sw) as f32 + 0.5;
    if !behind_ship {
        ship(r, sx as f32 + sw as f32 / 2., sz as f32 + sd as f32 / 2. - 1., t, lt.night, labels);
    }
    // today's resources
    let i = (k as usize).clamp(1, 2) - 1;
    for (s, &(x, z)) in SAMPLE_SPOTS[i].iter().enumerate() {
        if g.sample_taken(k, s) {
            continue;
        }
        let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5);
        let u = r.unlit;
        r.unlit = true;
        let pulse = 0.8 + 0.2 * (t * 3. + s as f32).sin();
        if k == 1 {
            for j in 0..4 {
                let a = j as f32 * 1.57 + 0.4;
                r.cone(
                    p + vec3(a.cos() * 0.1, 0., a.sin() * 0.1),
                    0.07,
                    0.35 + (j % 2) as f32 * 0.12,
                    Color::new(0.5 * pulse, 0.85 * pulse, 1., 1.),
                );
            }
        } else {
            for j in 0..3 {
                let a = j as f32 * 2.1;
                let q = p + vec3(a.cos() * 0.12, 0., a.sin() * 0.12);
                r.seg(q, q + vec3(0., 0.2, 0.), 0.025, Color::new(0.85, 0.85, 0.75, 1.));
                r.sphere(
                    q + vec3(0., 0.22, 0.),
                    vec3(0.1, 0.05, 0.1),
                    Color::new(0.45 * pulse, 1. * pulse, 0.7 * pulse, 1.),
                );
            }
        }
        r.unlit = u;
        if me.distance(vec2(p.x, p.z)) < 2.5 {
            labels.push((p + vec3(0., 0.8, 0.), def(SAMPLE_ITEM[i]).name.into(), Color::new(0.85, 1., 0.95, 1.)));
        }
    }
    let shc = Color::new(0., 0., 0., 0.2);
    r.disc(vec3(me.x, 0.02, me.y), vec2(0.32, 0.26), shc);
    human(
        r,
        g.me.id,
        (g.w.p.shirt, g.w.p.hat, g.w.p.hair),
        vec3(me.x, 0., me.y),
        yaw_of(g.me.face),
        g.me.anim,
        g.me.spd,
        0,
        None,
        t,
        g.me.swing,
    );
    mark_player(r, vec3(me.x, 0., me.y));
    // things between the camera and you, faded and drawn after you
    let a = r.alpha;
    r.alpha = if marking() { 0. } else { 0.3 };
    for (x, z, inside) in faded {
        if inside {
            prop(r, k, x, z, t)
        } else {
            border(r, k, x, z, cliff, t)
        }
    }
    if behind_ship {
        ship(r, sx as f32 + sw as f32 / 2., sz as f32 + sd as f32 / 2. - 1., t, lt.night, labels);
    }
    r.alpha = a;
    if me.distance(vec2(P_RAMP.0, P_RAMP.1)) < 4. {
        labels.push((vec3(P_RAMP.0 - 2.2, 1.6, P_RAMP.1), PLANETS[k as usize].0.into(), Color::new(0.9, 0.97, 1., 1.)));
    }
}

/// Edge of the landing site: ice cliffs on Glacia, dense jungle on Verdance.
fn border(r: &mut R, k: u8, x: i32, z: i32, col: Color, t: f32) {
    let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5);
    let hsh = hash(x, z);
    let low = z >= PH - 2;
    if k == 1 {
        let h = if low { 0.3 } else { 1.6 + (hsh % 4) as f32 * 0.4 };
        r.cube(p + vec3(0., h / 2., 0.), vec3(0.5, h / 2., 0.5), sh(col, 0.92 + (hsh % 3) as f32 * 0.04));
        r.cube(p + vec3(0., h + 0.04, 0.), vec3(0.52, 0.05, 0.52), WHITE);
    } else if low {
        r.sphere(p + vec3(0., 0.15, 0.), vec3(0.5, 0.2, 0.5), sh(col, 1.3)); // low ferns on the near edge
    } else {
        let h = 1.6 + (hsh % 4) as f32 * 0.35;
        let sway = (t * 0.8 + x as f32).sin() * 0.04;
        r.cyl(p, 0.12, h, Color::new(0.32, 0.24, 0.18, 1.));
        r.sphere(p + vec3(sway, h, 0.), vec3(0.7, 0.45, 0.7), sh(col, 1.1 + (hsh % 3) as f32 * 0.1));
    }
}
