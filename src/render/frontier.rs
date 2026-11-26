//! Dustreach visuals: warm desert ground, canyon walls, adobe buildings, market tents, the wreck, the gate,
//! the broker, bounty targets with their scarves, plus the bounty HUD panel and the board menu.
use super::*;

const SAND_C: Color = Color::new(0.87, 0.62, 0.42, 1.);
const PATH_C: Color = Color::new(0.68, 0.49, 0.35, 1.);
const ROCK_C: Color = Color::new(0.66, 0.36, 0.26, 1.);
const ADOBE: Color = Color::new(0.84, 0.66, 0.5, 1.);
const BEAM: Color = Color::new(0.42, 0.28, 0.18, 1.);
pub const TIER_NAMES: [&str; 3] = ["Easy", "Runner", "Lookalikes"];

/// Warm, dusty version of the day's light for the outpost.
pub(super) fn dusty(lt: &Light) -> Light {
    let mix = |a: f32, b: f32| a * 0.45 + b * 0.55;
    let warm = Color::new(0.98, 0.74, 0.52, 1.);
    let k = 1. - lt.night * 0.75;
    Light {
        sky: Color::new(mix(lt.sky.r, warm.r * k), mix(lt.sky.g, warm.g * k), mix(lt.sky.b, warm.b * k), 1.),
        sun: lt.sun,
        amb: lt.amb * vec3(1.06, 0.98, 0.9),
        dif: lt.dif * vec3(1.08, 1.0, 0.88),
        night: lt.night,
    }
}

fn mesa(r: &mut R, x: f32, z: f32, h: f32, k: u32) {
    // layered rock: three bands that step inward, each a slightly different rust tone
    let mut y = 0.;
    for b in 0..3 {
        let bh = h / 3.;
        let w = 0.5 - b as f32 * 0.04;
        let tone = 0.86 + ((k + b) % 4) as f32 * 0.05;
        r.cube(vec3(x + 0.5, y + bh / 2., z + 0.5), vec3(w, bh / 2., w), sh(ROCK_C, tone));
        r.cube(vec3(x + 0.5, y + bh - 0.02, z + 0.5), vec3(w + 0.01, 0.02, w + 0.01), sh(ROCK_C, tone * 0.82));
        y += bh;
    }
}

fn adobe_box(r: &mut R, x: f32, z: f32, w: f32, d: f32, h: f32, door: bool) {
    let c = vec3(x + w / 2., h / 2., z + d / 2.);
    r.cube(c, vec3(w / 2. - 0.05, h / 2., d / 2. - 0.05), ADOBE);
    r.cube(vec3(c.x, h + 0.08, c.z), vec3(w / 2., 0.08, d / 2.), sh(ADOBE, 0.92)); // parapet
    r.cube(vec3(c.x, 0.06, c.z), vec3(w / 2. - 0.02, 0.06, d / 2. - 0.02), sh(ADOBE, 0.78)); // weathered base
    let mut bx = x + 0.4;
    while bx < x + w - 0.2 {
        r.cube(vec3(bx, h - 0.22, z + d - 0.02), vec3(0.06, 0.06, 0.14), BEAM); // roof beam ends
        bx += 0.7;
    }
    if door {
        r.cube(vec3(c.x, 0.55, z + d - 0.02), vec3(0.32, 0.55, 0.04), Color::new(0.28, 0.18, 0.12, 1.));
        r.cube(vec3(c.x, 1.14, z + d - 0.01), vec3(0.4, 0.05, 0.05), BEAM);
    }
    for wx in [x + 0.6, x + w - 0.6] {
        r.cube(vec3(wx, 1.0, z + d - 0.02), vec3(0.2, 0.18, 0.03), Color::new(0.22, 0.16, 0.14, 1.));
    }
}

fn lantern(r: &mut R, p: Vec3, night: f32) {
    r.cube(p + vec3(0., 0.18, 0.), vec3(0.01, 0.12, 0.01), BEAM);
    let u = r.unlit;
    r.unlit = night > 0.3;
    r.cube(p, vec3(0.07, 0.09, 0.07), Color::new(1., 0.78, 0.42, 1.));
    r.unlit = u;
    if night > 0.3 {
        r.disc(vec3(p.x, 0.015, p.z + 0.4), vec2(0.9, 0.7), Color::new(1., 0.8, 0.45, 0.16 * night));
    }
}

pub(super) fn frontier_scene(r: &mut R, g: &Game, lt: &Light, labels: &mut Vec<(Vec3, String, Color)>) {
    let t = g.t;
    let m = &g.fmap;
    let cam = g.me.cam;
    let structure =
        |x: i32, z: i32| F_STRUCTS.iter().any(|&(_, sx, sz, w, d)| x >= sx && x < sx + w && z >= sz && z < sz + d);
    for z in -9..FH + 7 {
        for x in -12..FW + 12 {
            let (fx, fz) = (x as f32, z as f32);
            if (fx - cam.x).powi(2) + (fz - cam.y).powi(2) > 480. {
                continue;
            }
            let c = ftile(m, x, z);
            let base = if c == PATH { PATH_C } else { SAND_C };
            let col = |dx: i32, dz: i32| gtint(base, gnoise(x + dx, z + dz));
            r.quad4(
                [vec3(fx, 0., fz), vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.)],
                Vec3::Y,
                [col(0, 0), col(1, 0), col(1, 1), col(0, 1)],
            );
            if c == PATH && hash(x, z) % 2 == 0 {
                for ox in [0.3, 0.7] {
                    r.quad(
                        vec3(fx + ox, 0.004, fz),
                        vec3(fx + ox + 0.05, 0.004, fz),
                        vec3(fx + ox + 0.05, 0.004, fz + 1.),
                        vec3(fx + ox, 0.004, fz + 1.),
                        Vec3::Y,
                        sh(PATH_C, 0.86),
                    ); // cart ruts
                }
            }
            let inside = (0..FW).contains(&x) && (0..FH).contains(&z);
            if c == ROCK && !structure(x, z) {
                let k = hash(x, z);
                let edge = !inside || x == 0 || z == 0 || x == FW - 1 || z == FH - 1;
                // the camera looks north, so cliffs only rise on the far sides; the south rim stays low rubble
                let h = if z >= FH - 2 {
                    0.16 + (k % 3) as f32 * 0.06
                } else if edge {
                    2.2 + (k % 5) as f32 * 0.45
                } else {
                    0.7 + (k % 3) as f32 * 0.3
                };
                mesa(r, fx, fz, h, k);
            } else if c == SAND && hash(x * 3, z * 7) % 11 == 0 {
                // scrub: a dry bush and the odd pebble
                let p = vec3(fx + 0.3 + (hash(x, z) % 5) as f32 * 0.1, 0., fz + 0.5);
                for k in 0..4 {
                    let a = k as f32 * 1.6;
                    r.seg(p, p + vec3(a.cos() * 0.18, 0.22, a.sin() * 0.18), 0.012, Color::new(0.5, 0.4, 0.25, 1.));
                }
                r.sphere(p + vec3(0.25, 0.03, 0.1), vec3(0.06, 0.04, 0.05), sh(ROCK_C, 0.9));
            }
            if !inside && z < FH - 1 && hash(x + 50, z) % 3 != 0 {
                mesa(r, fx, fz, 3.0 + (hash(x, z) % 4) as f32 * 0.6, hash(x, z));
            }
        }
    }
    let night = lt.night;
    for &(kind, x, z, w, d) in &F_STRUCTS {
        let (fx, fz, fw, fd) = (x as f32, z as f32, w as f32, d as f32);
        match kind {
            0 => {
                adobe_box(r, fx, fz, fw, fd, 1.9, true);
                r.cube(vec3(fx + fw / 2., 1.62, fz + fd + 0.02), vec3(0.9, 0.18, 0.03), BEAM); // sign plank
                r.cube(vec3(fx + 0.8, 1.6, fz + fd + 0.04), vec3(0.2, 0.32, 0.01), Color::new(0.72, 0.2, 0.16, 1.)); // banner
                lantern(r, vec3(fx + fw / 2. - 0.6, 1.25, fz + fd + 0.12), night);
                labels.push((vec3(fx + fw / 2., 2.35, fz + fd), "Guild Post".into(), Color::new(1., 0.86, 0.6, 1.)));
            }
            1 => {
                adobe_box(r, fx, fz, fw, fd, 1.7, true);
                r.sphere(vec3(fx + fw / 2., 1.86, fz + fd / 2.), vec3(1.1, 0.6, 1.1), sh(ADOBE, 1.04)); // dome
                let cloth = [Color::new(0.85, 0.42, 0.22, 1.), Color::new(0.95, 0.85, 0.65, 1.)];
                for k in 0..6 {
                    let x0 = fx + fw / 2. - 1.2 + k as f32 * 0.4;
                    r.quad(
                        vec3(x0, 1.3, fz + fd),
                        vec3(x0 + 0.4, 1.3, fz + fd),
                        vec3(x0 + 0.4, 1.05, fz + fd + 0.7),
                        vec3(x0, 1.05, fz + fd + 0.7),
                        vec3(0., 0.94, 0.34),
                        cloth[k % 2],
                    ); // striped awning
                }
                for px in [fx + fw / 2. - 1.15, fx + fw / 2. + 1.15] {
                    r.cube(vec3(px, 0.52, fz + fd + 0.68), vec3(0.03, 0.52, 0.03), BEAM);
                    lantern(r, vec3(px, 0.92, fz + fd + 0.68), night);
                }
                labels.push((
                    vec3(fx + fw / 2., 2.75, fz + fd / 2.),
                    "Longhouse".into(),
                    Color::new(1., 0.86, 0.6, 1.),
                ));
            }
            2 => {
                let c = vec3(fx + fw / 2., 0., fz + fd / 2.);
                for (dx, dz) in [(-0.8, -0.8), (0.8, -0.8), (-0.8, 0.8), (0.8, 0.8)] {
                    r.cube(c + vec3(dx, 0.6, dz), vec3(0.03, 0.6, 0.03), BEAM);
                }
                let tone =
                    [Color::new(0.75, 0.3, 0.25, 1.), Color::new(0.3, 0.5, 0.6, 1.), Color::new(0.8, 0.6, 0.25, 1.)]
                        [(x as usize / 3) % 3];
                r.roof(c + vec3(0., 1.2, 0.), 0.95, 0.95, 0.45, tone);
                r.cube(c + vec3(-0.3, 0.2, 0.2), vec3(0.22, 0.2, 0.2), Color::new(0.6, 0.45, 0.28, 1.)); // crate
                r.cube(c + vec3(0.35, 0.18, -0.25), vec3(0.18, 0.18, 0.18), Color::new(0.55, 0.4, 0.25, 1.));
                r.cyl(c + vec3(0.4, 0., 0.35), 0.14, 0.32, Color::new(0.72, 0.42, 0.3, 1.));
                // clay pot
            }
            3 => {
                let c = vec3(fx + 1., 0., fz + 1.);
                for (dx, dz) in [(-0.55, -0.55), (0.55, -0.55), (-0.55, 0.55), (0.55, 0.55)] {
                    r.seg(c + vec3(dx, 0., dz), c + vec3(dx * 0.7, 2.2, dz * 0.7), 0.05, BEAM);
                }
                r.cyl(c + vec3(0., 2.2, 0.), 0.75, 0.9, Color::new(0.55, 0.52, 0.48, 1.));
                r.cone(c + vec3(0., 3.1, 0.), 0.8, 0.35, Color::new(0.45, 0.42, 0.4, 1.));
                for k in 0..7 {
                    r.cube(c + vec3(0.62, 0.3 + k as f32 * 0.3, 0.62), vec3(0.12, 0.015, 0.015), BEAM);
                    // ladder rungs
                }
            }
            4 => {
                // a long-crashed hauler, half sunk in the sand
                let c = vec3(fx + fw / 2., 0.35, fz + fd / 2.);
                let hull = Color::new(0.55, 0.55, 0.52, 1.);
                r.with(
                    Mat4::from_rotation_translation(Quat::from_rotation_z(1.35) * Quat::from_rotation_y(0.3), c),
                    |r| {
                        r.cyl(vec3(0., -2.2, 0.), 0.85, 4.4, hull);
                        r.cone(vec3(0., 2.2, 0.), 0.85, 0.9, sh(hull, 0.9));
                        for k in 0..4 {
                            r.cyl(vec3(0., -1.6 + k as f32 * 1.1, 0.), 0.88, 0.08, sh(hull, 0.7));
                            // hull ribs
                        }
                    },
                );
                r.cube(c + vec3(-1.4, 0.5, 1.0), vec3(0.6, 0.04, 0.35), sh(hull, 0.8)); // broken fin
                r.cube(c + vec3(1.2, 0.15, -1.1), vec3(0.3, 0.15, 0.2), Color::new(0.25, 0.22, 0.2, 1.));
                // scorched panel
            }
            _ => {
                let c = vec3(fx + 1., 0., fz + 1.);
                r.sphere(c, vec3(0.95, 1.05, 0.95), ADOBE);
                r.cube(c + vec3(0., 0.4, 0.9), vec3(0.22, 0.4, 0.06), Color::new(0.28, 0.18, 0.12, 1.));
                r.cyl(c + vec3(0.3, 0.85, -0.2), 0.08, 0.35, sh(ADOBE, 0.85)); // little chimney
            }
        }
    }
    // the gate back to Hollowbrook: two posts, a crossbeam and a faint green shimmer
    let gx = F_GATE.0 as f32 + 1.;
    let gz = F_GATE.1 as f32 + 0.5;
    for px in [gx - 1.05, gx + 1.05] {
        r.cube(vec3(px, 1.3, gz), vec3(0.12, 1.3, 0.12), BEAM);
    }
    r.cube(vec3(gx, 2.6, gz), vec3(1.3, 0.1, 0.14), BEAM);
    r.cube(vec3(gx, 2.32, gz + 0.12), vec3(0.6, 0.14, 0.02), Color::new(0.35, 0.55, 0.32, 1.));
    let a = r.alpha;
    r.alpha = 0.22 + 0.06 * (t * 1.7).sin();
    let u = r.unlit;
    r.unlit = true;
    r.quad(
        vec3(gx - 0.95, 0.02, gz),
        vec3(gx + 0.95, 0.02, gz),
        vec3(gx + 0.95, 2.5, gz),
        vec3(gx - 0.95, 2.5, gz),
        Vec3::Z,
        Color::new(0.55, 0.95, 0.6, 1.),
    );
    r.unlit = u;
    r.alpha = a;
    if g.me.ipos.distance(vec2(gx, gz)) < 4. {
        labels.push((vec3(gx, 3.0, gz), "To Hollowbrook".into(), Color::new(0.75, 1., 0.75, 1.)));
    }
    // bounty board with one paper per open offer
    let (bx, bz) = (F_BOARD.0 as f32 + 0.5, F_BOARD.1 as f32 + 0.5);
    for px in [bx - 0.45, bx + 0.45] {
        r.cube(vec3(px, 0.65, bz), vec3(0.04, 0.65, 0.04), BEAM);
    }
    r.cube(vec3(bx, 1.05, bz), vec3(0.55, 0.35, 0.04), Color::new(0.55, 0.38, 0.24, 1.));
    for k in 0..3u8 {
        if !g.bounty_done_today(k) {
            r.cube(
                vec3(bx - 0.32 + k as f32 * 0.32, 1.05, bz + 0.05),
                vec3(0.12, 0.16, 0.005),
                Color::new(0.96, 0.92, 0.8, 1.),
            );
            r.cube(
                vec3(bx - 0.32 + k as f32 * 0.32, 1.13, bz + 0.056),
                vec3(0.06, 0.04, 0.003),
                Color::new(0.4, 0.3, 0.25, 1.),
            );
        }
    }
    // the broker
    human(
        r,
        1,
        (Some(66), Some(68), 2),
        vec3(F_BROKER.0, 0., F_BROKER.1),
        yaw_of(vec2(0., 1.)),
        0.,
        0.,
        0,
        None,
        t,
        0.,
    );
    if g.me.ipos.distance(vec2(F_BROKER.0, F_BROKER.1)) < 3.5 {
        labels.push((vec3(F_BROKER.0, 1.9, F_BROKER.1), "Broker Ines Varro".into(), Color::new(1., 0.9, 0.7, 1.)));
    }
    // the active player's bounty: target (and lookalikes), each with a scarf
    if let Some(b) = g.w.p.bounty.as_ref().filter(|b| !b.caught) {
        let looks = [(Some(63), Some(67), 1), (Some(65), Some(67), 1), (Some(64), Some(67), 1)];
        for (i, p) in b.pos.iter().enumerate() {
            let pos = vec3(p[0], 0., p[1]);
            let running = i == 0 && b.run > 0.;
            let face = if running { vec2(p[0], p[1]) - g.me.ipos } else { g.me.ipos - vec2(p[0], p[1]) };
            r.disc(vec3(pos.x, 0.02, pos.z), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.18));
            human(
                r,
                1,
                looks[i % 3],
                pos,
                yaw_of(face.normalize_or_zero()),
                t * 3.,
                if running { 4.2 } else { 0. },
                0,
                None,
                t,
                0.,
            );
            let sc = SCARVES[i % 3].1;
            let scol = Color::new(sc.0, sc.1, sc.2, 1.);
            let fwd = face.normalize_or_zero();
            r.cube(pos + vec3(0., 0.74, 0.), vec3(0.22, 0.06, 0.2), scol); // wrapped scarf
            let tail = pos + vec3(fwd.x * 0.2, 0.6, fwd.y * 0.2);
            r.cube(tail, vec3(0.06, 0.13, 0.06), sh(scol, 0.85)); // loose end hanging at the front
        }
    }
    // tumbleweeds rolling down the cross street
    for k in 0..2 {
        let x = ((t * 1.3 + k as f32 * 13.) % 32.) - 1.;
        let p = vec3(x, 0.22 + (t * 5. + k as f32).sin().abs() * 0.12, 10.5 + k as f32 * 6.);
        r.sphere(p, Vec3::splat(0.22), Color::new(0.62, 0.48, 0.28, 1.));
        r.sphere(p + vec3(0.05, 0.05, 0.), Vec3::splat(0.16), Color::new(0.5, 0.38, 0.22, 1.));
    }
    others(r, g, t);
    let shc = Color::new(0., 0., 0., 0.2);
    r.disc(vec3(g.me.ipos.x, 0.02, g.me.ipos.y), vec2(0.32, 0.26), shc);
    human(
        r,
        g.me.id,
        (g.w.p.shirt, g.w.p.hat, g.w.p.hair),
        vec3(g.me.ipos.x, 0., g.me.ipos.y),
        yaw_of(g.me.face),
        g.me.anim,
        g.me.spd,
        0,
        None,
        t,
        g.me.swing,
    );
}

/// The bounty panel under the messages while a hunt is on.
pub(super) fn bounty_hud(g: &Game, sw: f32) {
    if g.me.scene != Scene::Frontier {
        return;
    }
    let Some(b) = g.w.p.bounty.as_ref() else { return };
    let o = &g.bounty_offers()[b.offer as usize];
    let bw = (sw - 24.).min(520.);
    draw_rectangle(12., 186., bw, 52., Color::new(0.3, 0.16, 0.08, 0.85));
    let line = if b.caught {
        format!("{} detained. Take them to the board.", o.name)
    } else {
        format!("Bounty: {}   Tracker: {}", o.name, g.tracker_word())
    };
    txt(&line, 22., 207., 21., WHITE);
    let sub = if b.tier == 2 && !b.caught {
        format!("Last seen {}. Wears a {} scarf.", F_SPOTS[o.spot].1, SCARVES[0].0)
    } else {
        format!("Last seen {}.", F_SPOTS[o.spot].1)
    };
    txt(&sub, 22., 229., 17., Color::new(1., 0.9, 0.75, 1.));
}

pub(super) fn bounty_menu(g: &Game, sel: usize, sw: f32) {
    let pw = (sw - 40.).min(700.);
    let x0 = sw / 2. - pw / 2.;
    panel(x0, 70., pw, 430.);
    txt("Guild Post  -  bounties", x0 + 20., 105., 24., Color::new(0.45, 0.3, 0.2, 1.));
    txt(
        &format!("Rank: {}   ({} brought in)", rank_of(g.w.p.bounties_done), g.w.p.bounties_done),
        x0 + 20.,
        130.,
        18.,
        DARKGRAY,
    );
    let offers = g.bounty_offers();
    let mut items: Vec<String> = offers
        .iter()
        .enumerate()
        .map(|(k, o)| {
            let done = if g.bounty_done_today(k as u8) { "  (done)" } else { "" };
            format!("{}: {}  -  {}c{done}", TIER_NAMES[o.tier as usize], o.name, o.reward)
        })
        .collect();
    items.push(match g.w.p.bounty.as_ref() {
        Some(b) if b.caught => {
            format!("Turn in {}  (+{}c)", offers[b.offer as usize].name, BOUNTY_REWARD[b.tier as usize])
        }
        Some(_) => "Drop current bounty".into(),
        None => "-".into(),
    });
    items.push("Close".into());
    list(&items, Some(sel), x0 + 30., 175.);
    if let Some(o) = offers.get(sel) {
        let how = [
            "Waits where last seen.",
            "Runs when you get close: run (Shift) to catch them.",
            "Hides among two lookalikes; check the scarf.",
        ][o.tier as usize];
        for (k, l) in wrap_txt(
            &format!("{} {}. Last seen {}. {how}", o.name, o.crime, F_SPOTS[o.spot].1),
            ((pw - 60.) / 10.) as usize,
        )
        .iter()
        .enumerate()
        {
            txt(l, x0 + 30., 345. + k as f32 * 22., 18., DARKGRAY);
        }
    }
    txt("W/S choose   Enter take   Esc close", x0 + 30., 480., 18., Color::new(0.45, 0.3, 0.2, 1.));
}
