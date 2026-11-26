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
    let mut faded_rocks: Vec<(f32, f32, f32, u32)> = vec![];
    let structure =
        |x: i32, z: i32| F_STRUCTS.iter().any(|&(_, sx, sz, w, d)| x >= sx && x < sx + w && z >= sz && z < sz + d);
    for z in -9..FH + 7 {
        for x in -12..FW + 12 {
            let (fx, fz) = (x as f32, z as f32);
            if (fx - cam.x).powi(2) + (fz - cam.y).powi(2) > 480. {
                continue;
            }
            let c = ftile(m, x, z);
            let salt = x >= 46 && z >= 17 && c != PATH;
            let base = if c == PATH {
                PATH_C
            } else if salt {
                Color::new(0.88, 0.8, 0.68, 1.) // salt flats
            } else {
                SAND_C
            };
            if salt && hash(x * 11, z * 13) % 3 == 0 {
                let (a, b) = ((hash(x, z) % 10) as f32 * 0.1, (hash(z, x) % 10) as f32 * 0.1);
                r.seg(
                    vec3(fx + a, 0.006, fz),
                    vec3(fx + 1. - b, 0.006, fz + 1.),
                    0.012,
                    Color::new(0.75, 0.72, 0.66, 1.),
                ); // cracks
            }
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
            let arch = z == F_ARCH.1 && (x == F_ARCH.0 || x == F_ARCH.0 + 2);
            if c == ROCK && !structure(x, z) && !arch {
                let k = hash(x, z);
                let edge = !inside || x == 0 || z == 0 || x == FW - 1 || z == FH - 1;
                // the camera looks north, so cliffs only rise on the far sides; the south rim stays low rubble
                let h = if z >= FH - 2 {
                    0.16 + (k % 3) as f32 * 0.06
                } else if edge {
                    if z <= 0 {
                        1.3 + (k % 4) as f32 * 0.3
                    } else {
                        2.2 + (k % 5) as f32 * 0.45
                    } // low north rim: open sky
                } else {
                    0.7 + (k % 3) as f32 * 0.3
                };
                let me = g.me.ipos;
                if inside && me.x > fx - 0.7 && me.x < fx + 1.7 && fz + 1. > me.y + 0.2 && fz < me.y + 2.5 {
                    faded_rocks.push((fx, fz, h, k));
                } else {
                    mesa(r, fx, fz, h, k);
                }
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
                mesa(r, fx, fz, 1.2 + (hash(x, z) % 4) as f32 * 0.35, hash(x, z));
                // low far mesas leave open sky
            }
        }
    }
    // the stone arch over the oasis trail: two banded pillars and a weathered span
    let (ax, az) = (F_ARCH.0 as f32, F_ARCH.1 as f32);
    let arch_blocks =
        g.me.ipos.x > ax - 0.6 && g.me.ipos.x < ax + 3.6 && g.me.ipos.y > az - 3.2 && g.me.ipos.y < az + 0.9;
    if !arch_blocks {
        draw_arch(r);
    }
    // the oasis: a soft oval of wet sand, deep teal water with a lighter shallow ring, drifting glints
    let (ox, oz, rx, rz) = F_OASIS;
    r.disc(vec3(ox, 0.006, oz), vec2(rx + 0.55, rz + 0.5), Color::new(0.66, 0.52, 0.36, 1.));
    r.disc(vec3(ox, 0.012, oz), vec2(rx + 0.05, rz + 0.05), Color::new(0.3, 0.72, 0.72, 1.));
    let ripple = (t * 1.1).sin() * 0.04;
    r.disc(vec3(ox, 0.018, oz), vec2(rx * 0.72 + ripple, rz * 0.7 + ripple), Color::new(0.15, 0.52, 0.6, 1.));
    let u = r.unlit;
    r.unlit = true;
    for k in 0..6 {
        let a = t * 0.25 + k as f32 * 1.05;
        let p = vec3(ox + a.cos() * rx * 0.55, 0.024, oz + (a * 1.3).sin() * rz * 0.5);
        r.disc(p, vec2(0.16, 0.035), Color::new(0.88, 0.98, 1., 0.55));
    }
    r.unlit = u;
    // palms, reeds and a few flowers round the oasis
    for k in 0..7 {
        let a = k as f32 / 7. * std::f32::consts::TAU + 0.4;
        let (px, pz) = (ox + a.cos() * (rx + 0.9), oz + a.sin() * (rz + 0.9));
        if k % 2 == 0 {
            palm(r, px, pz, t);
        } else {
            for j in 0..5 {
                let b = j as f32 * 1.3;
                let p = vec3(px + b.cos() * 0.12, 0., pz + b.sin() * 0.12);
                r.seg(
                    p,
                    p + vec3(b.cos() * 0.08, 0.45 + (j % 2) as f32 * 0.12, b.sin() * 0.08),
                    0.015,
                    Color::new(0.38, 0.55, 0.28, 1.),
                );
            }
            r.sphere(vec3(px + 0.25, 0.08, pz), vec3(0.06, 0.05, 0.06), Color::new(0.95, 0.55, 0.65, 1.));
        }
    }
    if g.me.ipos.distance(vec2(ox, oz)) < 6. {
        labels.push((vec3(ox, 1.2, oz), "Oasis".into(), Color::new(0.75, 0.95, 1., 1.)));
    }
    let night = lt.night;
    // anything standing between the camera (south) and the player is drawn faded, after the player
    let me = g.me.ipos;
    let blocks = |fx: f32, fz: f32, fw: f32, fd: f32| {
        me.x > fx - 0.3 && me.x < fx + fw + 0.3 && fz + fd > me.y + 0.4 && fz < me.y + 2.6
    };
    let mut faded: Vec<(u8, i32, i32, i32, i32)> = vec![];
    for &(kind, x, z, w, d) in &F_STRUCTS {
        if blocks(x as f32, z as f32, w as f32, d as f32) {
            faded.push((kind, x, z, w, d));
            continue;
        }
        draw_struct(r, kind, x, z, w, d, t, night, labels);
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
        labels.push((vec3(F_BROKER.0, 1.9, F_BROKER.1), "Guild Master Varro".into(), Color::new(1., 0.9, 0.7, 1.)));
    }
    // the active player's bounty: target (and lookalikes), each with a scarf
    if let Some(b) = g.w.p.bounty.as_ref().filter(|b| !b.delivered) {
        let looks = [(Some(63), Some(67), 1), (Some(65), Some(67), 1), (Some(64), Some(67), 1)];
        for (i, p) in b.pos.iter().enumerate().filter(|(i, _)| !b.caught || *i == 0) {
            let pos = vec3(p[0], 0., p[1]);
            let running =
                i == 0 && ((b.run > 0. && b.stun <= 0.) || (b.caught && vec2(p[0], p[1]).distance(g.me.ipos) > 1.2));
            let face = if running && !b.caught { vec2(p[0], p[1]) - g.me.ipos } else { g.me.ipos - vec2(p[0], p[1]) };
            if b.caught {
                // binders on the wrists
                r.cube(vec3(p[0], 0.45, p[1]), vec3(0.16, 0.04, 0.12), Color::new(0.3, 0.32, 0.36, 1.));
            }
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
            if i == 0 && b.stun > 0. {
                let u = r.unlit;
                r.unlit = true;
                for k in 0..3 {
                    let a = t * 5. + k as f32 * 2.1;
                    r.sphere(
                        pos + vec3(a.cos() * 0.28, 1.5, a.sin() * 0.28),
                        Vec3::splat(0.05),
                        Color::new(1., 0.92, 0.35, 1.),
                    );
                }
                r.unlit = u;
            }
            let sc = SCARVES[i % 3].1;
            let scol = Color::new(sc.0, sc.1, sc.2, 1.);
            let fwd = face.normalize_or_zero();
            r.cube(pos + vec3(0., 0.74, 0.), vec3(0.22, 0.06, 0.2), scol); // wrapped scarf
            let tail = pos + vec3(fwd.x * 0.2, 0.6, fwd.y * 0.2);
            r.cube(tail, vec3(0.06, 0.13, 0.06), sh(scol, 0.85)); // loose end hanging at the front
        }
    }
    // gear shots in flight
    let u = r.unlit;
    r.unlit = true;
    for s in &g.me.bolts {
        let yaw = s.vel.x.atan2(s.vel.y);
        let at = vec3(s.pos.x, 0.8, s.pos.y);
        if s.kind == 0 {
            r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), at), |r| {
                r.cube(Vec3::ZERO, vec3(0.03, 0.03, 0.24), Color::new(0.65, 0.97, 1., 1.));
                r.cube(Vec3::ZERO, vec3(0.06, 0.06, 0.14), Color::new(0.4, 0.85, 1., 0.5));
            });
        } else {
            let a = t * 14.;
            let off = vec3(a.cos() * 0.22, 0., a.sin() * 0.22);
            r.seg(at - off, at + off, 0.012, Color::new(0.85, 0.8, 0.6, 1.));
            for p in [at - off, at + off] {
                r.sphere(p, Vec3::splat(0.05), Color::new(0.25, 0.25, 0.28, 1.));
            }
        }
    }
    r.unlit = u;
    // tumbleweeds rolling down the cross street
    for k in 0..2 {
        let x = ((t * 1.3 + k as f32 * 13.) % 32.) - 1.;
        let p = vec3(x, 0.22 + (t * 5. + k as f32).sin().abs() * 0.12, 10.5 + k as f32 * 6.);
        r.sphere(p, Vec3::splat(0.22), Color::new(0.62, 0.48, 0.28, 1.));
        r.sphere(p + vec3(0.05, 0.05, 0.), Vec3::splat(0.16), Color::new(0.5, 0.38, 0.22, 1.));
    }
    others(r, g, t);
    if g.w.p.pal {
        sand_hopper(r, g.me.pal_pos, t, g.bounty_dist().is_some_and(|d| d < 6.));
    } else {
        sand_hopper(r, vec2(F_PAL_SPOT.0, F_PAL_SPOT.1), t, false);
        if g.me.ipos.distance(vec2(F_PAL_SPOT.0, F_PAL_SPOT.1)) < 3. {
            labels.push((vec3(F_PAL_SPOT.0, 0.9, F_PAL_SPOT.1), "Sand-hopper".into(), Color::new(1., 0.9, 0.7, 1.)));
        }
    }
    frontier_life(r, g, t, labels);
    frontier_wilds(r, t);
    if !g.me.riding {
        hover_bike(r, vec2(F_BIKE.0, F_BIKE.1), 0.4, t);
    } else {
        hover_bike(r, g.me.ipos, yaw_of(g.me.face), t);
    }
    hunter_gear(r, g, t);
    let shc = Color::new(0., 0., 0., 0.2);
    r.disc(vec3(g.me.ipos.x, 0.02, g.me.ipos.y), vec2(0.32, 0.26), shc);
    human(
        r,
        g.me.id,
        (g.w.p.shirt, g.w.p.hat, g.w.p.hair),
        vec3(g.me.ipos.x, if g.me.riding { 0.34 } else { 0. }, g.me.ipos.y),
        yaw_of(g.me.face),
        g.me.anim,
        g.me.spd,
        0,
        g.w.p.equip,
        t,
        g.me.swing,
    );
    mark_player(r, vec3(g.me.ipos.x, if g.me.riding { 0.34 } else { 0. }, g.me.ipos.y));
    // structures and rocks that block the view, faded, after the player so the player shows through
    let a_view = r.alpha;
    r.alpha = if marking() { 0. } else { 0.3 };
    for (kind, x, z, w, d) in faded {
        draw_struct(r, kind, x, z, w, d, t, night, labels);
    }
    for (fx, fz, h, k) in faded_rocks {
        mesa(r, fx, fz, h, k);
    }
    if arch_blocks {
        draw_arch(r);
    }
    r.alpha = a_view;
    // drawn last so that, when faded, the player behind it still shows through
    // the gate back to Hollowbrook: two posts, a crossbeam and a faint green shimmer
    let gx = F_GATE.0 as f32 + 1.;
    let gz = F_GATE.1 as f32 + 0.5;
    // the camera looks north, so the gate would hide a player standing just behind it: fade it while they are
    let behind = g.me.ipos.y < gz && g.me.ipos.y > gz - 4. && (g.me.ipos.x - gx).abs() < 2.5;
    let a0 = r.alpha;
    if behind {
        r.alpha = if marking() { 0. } else { 0.3 };
    }
    for px in [gx - 1.05, gx + 1.05] {
        r.cube(vec3(px, 1.3, gz), vec3(0.12, 1.3, 0.12), BEAM);
    }
    r.cube(vec3(gx, 2.6, gz), vec3(1.3, 0.1, 0.14), BEAM);
    r.cube(vec3(gx, 2.32, gz + 0.12), vec3(0.6, 0.14, 0.02), Color::new(0.35, 0.55, 0.32, 1.));
    r.alpha = a0;
    let a = r.alpha;
    r.alpha = (0.22 + 0.06 * (t * 1.7).sin()) * if behind { 0.4 } else { 1. };
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
}

/// The bounty panel under the messages while a hunt is on.
pub(super) fn bounty_hud(g: &Game, sw: f32) {
    if g.me.scene != Scene::Frontier {
        return;
    }
    if g.me.standoff > 0. {
        let (msg, col) = if g.me.standoff <= DRAW_WINDOW {
            ("DRAW!", Color::new(1., 0.3, 0.2, 1.))
        } else {
            ("steady...", Color::new(1., 0.92, 0.75, 1.))
        };
        let size = if g.me.standoff <= DRAW_WINDOW { 72. } else { 40. };
        let w = measure_text(msg, None, size as u16, 1.).width;
        draw_rectangle(sw / 2. - w / 2. - 18., 520., w + 36., size + 14., Color::new(0., 0., 0., 0.45)); // below the player, clear of the target
        txt(msg, sw / 2. - w / 2., 520. + size, size, col);
    }
    let Some(b) = g.w.p.bounty.as_ref().filter(|_| g.me.ui == Ui::Play) else { return }; // menus cover it
    let o = &g.bounty_offers()[b.offer as usize];
    let bw = (sw - 24.).min(520.);
    draw_rectangle(12., 186., bw, 52., Color::new(0.3, 0.16, 0.08, 0.85));
    let line = if b.delivered {
        format!("{} is in your holding pod. Collect the pay at the board.", o.name)
    } else if b.caught {
        format!("{} detained. Walk them to your ship's holding pod.", o.name)
    } else {
        format!("Bounty: {}   Tracker: {}", o.name, g.tracker_word())
    };
    txt(&line, 22., 207., 21., WHITE);
    let sub = if b.hinted && !b.caught {
        format!("Moss says: head {} from here.", bearing(g.me.ipos, vec2(b.pos[0][0], b.pos[0][1])))
    } else if b.tier == 2 && !b.caught {
        format!("Last seen {}. Wears a {} scarf.", F_SPOTS[o.spot].1, SCARVES[0].0)
    } else {
        format!("Last seen {}.", F_SPOTS[o.spot].1)
    };
    txt(&sub, 22., 229., 17., Color::new(1., 0.9, 0.75, 1.));
}

pub(super) fn bounty_menu(g: &Game, sel: usize, sw: f32) {
    let pw = (sw - 40.).min(700.);
    let x0 = sw / 2. - pw / 2.;
    panel(x0, 70., pw, 500.);
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
    for &(d, price, _) in &GEAR {
        let owned = if g.w.p.tools.contains(&d) { "  (owned)" } else { "" };
        items.push(format!("Gear: {}  -  {price}c{owned}", def(d).name));
    }
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
            txt(l, x0 + 30., 405. + k as f32 * 22., 18., DARKGRAY);
        }
    } else if let Some(gear) = sel.checked_sub(4).and_then(|k| GEAR.get(k)) {
        let extra = " Works only in Dustreach: Space to use, Tab to switch tools.";
        for (k, l) in wrap_txt(&format!("{}{extra}", gear.2), ((pw - 60.) / 10.) as usize).iter().enumerate() {
            txt(l, x0 + 30., 405. + k as f32 * 22., 18., DARKGRAY);
        }
    }
    txt("W/S choose   Enter take / buy   Esc close", x0 + 30., 550., 18., Color::new(0.45, 0.3, 0.2, 1.));
}

/// The door to Dustreach at the end of the woods path: a stone-footed wooden frame, a sun-bleached door,
/// a warm desert glow leaking round the edges, a lantern and a small sign.
pub(super) fn woods_door(r: &mut R, g: &Game, lt: &Light, labels: &mut Vec<(Vec3, String, Color)>) {
    let (x, z) = (WOODS_DOOR.0 as f32 + 0.5, WOODS_DOOR.1 as f32 + 0.5);
    if (x - g.me.cam.x).powi(2) + (z - g.me.cam.y).powi(2) > 480. {
        return;
    }
    // the path reaches the door from the east, so the door faces +x
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2), vec3(x, 0., z)), |r| {
        let (x, z) = (0f32, 0f32);
        let stone = Color::new(0.6, 0.58, 0.55, 1.);
        for px in [x - 0.55, x + 0.55] {
            r.cube(vec3(px, 0.15, z), vec3(0.14, 0.15, 0.16), stone);
            r.cube(vec3(px, 1.05, z), vec3(0.09, 0.8, 0.09), BEAM);
        }
        r.cube(vec3(x, 1.9, z), vec3(0.7, 0.09, 0.12), BEAM);
        r.cube(vec3(x, 2.02, z), vec3(0.58, 0.04, 0.1), Color::new(0.3, 0.55, 0.3, 1.)); // moss on the lintel
        let u = r.unlit;
        r.unlit = true;
        let a = r.alpha;
        r.alpha = 0.55 + 0.1 * (g.t * 1.6).sin();
        r.cube(vec3(x, 0.92, z + 0.03), vec3(0.47, 0.86, 0.02), Color::new(1., 0.72, 0.42, 1.)); // desert light behind the door
        r.alpha = a;
        r.unlit = u;
        r.cube(vec3(x, 0.9, z + 0.07), vec3(0.42, 0.82, 0.03), ADOBE);
        for k in 0..3 {
            r.cube(vec3(x - 0.28 + k as f32 * 0.28, 0.9, z + 0.1), vec3(0.01, 0.8, 0.01), sh(ADOBE, 0.8));
            // planks
        }
        r.sphere(vec3(x + 0.28, 0.92, z + 0.12), Vec3::splat(0.035), GOLD);
        lantern(r, vec3(x + 0.72, 1.55, z + 0.16), lt.night);
        r.cube(vec3(x - 0.95, 0.45, z + 0.35), vec3(0.03, 0.45, 0.03), BEAM);
        r.cube(vec3(x - 0.95, 0.85, z + 0.37), vec3(0.24, 0.1, 0.02), Color::new(0.85, 0.6, 0.38, 1.));
    });
    if g.me.pos.distance(vec2(x, z)) < 5. {
        labels.push((vec3(x, 2.5, z), "Dustreach".into(), Color::new(1., 0.82, 0.6, 1.)));
    }
}

/// Your ship: a weathered, patched-up freighter on stubby legs with a lowered ramp and a holding pod.
fn ship(r: &mut R, cx: f32, cz: f32, t: f32, night: f32, labels: &mut Vec<(Vec3, String, Color)>) {
    let hull = Color::new(0.62, 0.64, 0.66, 1.);
    let rust = Color::new(0.55, 0.36, 0.24, 1.);
    let c = vec3(cx, 1.05, cz);
    for (dx, dz) in [(-2.0, -1.2), (2.0, -1.2), (-2.0, 1.2), (2.0, 1.2)] {
        r.seg(c + vec3(dx, -0.2, dz), vec3(cx + dx * 1.08, 0., cz + dz * 1.1), 0.06, Color::new(0.35, 0.36, 0.38, 1.)); // landing legs
        r.cyl(vec3(cx + dx * 1.08, 0., cz + dz * 1.1), 0.18, 0.05, Color::new(0.3, 0.3, 0.32, 1.));
    }
    r.sphere(c, vec3(2.9, 0.75, 1.55), hull); // main hull
    r.cube(c + vec3(0., -0.2, 0.), vec3(2.1, 0.3, 1.05), sh(hull, 0.9)); // belly keel under the rounded hull
    r.sphere(c + vec3(-1.2, 0.55, -0.3), vec3(0.7, 0.18, 0.5), rust); // patched plates
    r.cube(c + vec3(1.1, 0.42, 0.9), vec3(0.45, 0.08, 0.32), sh(rust, 1.1));
    r.cube(c + vec3(0.2, 0.62, -0.6), vec3(0.6, 0.05, 0.25), sh(hull, 0.8));
    // cockpit at the front (+x), dark glass with a sky glint
    r.sphere(c + vec3(2.45, 0.35, 0.), vec3(0.7, 0.45, 0.62), Color::new(0.18, 0.22, 0.28, 1.));
    r.cube(c + vec3(2.7, 0.55, 0.25), vec3(0.18, 0.04, 0.12), Color::new(0.75, 0.85, 0.95, 1.));
    // twin engines at the back with a soft glow
    for dz in [-0.75, 0.75] {
        r.with(
            Mat4::from_rotation_translation(
                Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                c + vec3(-2.6, 0.0, dz),
            ),
            |r| {
                r.cyl(vec3(0., -0.6, 0.), 0.42, 1.2, sh(hull, 0.85));
                r.cyl(vec3(0., -0.66, 0.), 0.32, 0.08, Color::new(0.25, 0.25, 0.27, 1.));
            },
        );
        let u = r.unlit;
        r.unlit = true;
        let k = 0.7 + 0.3 * (t * 6. + dz).sin();
        r.sphere(c + vec3(-3.25, 0.0, dz), Vec3::splat(0.26), Color::new(0.45 * k, 0.75 * k, 1. * k, 1.));
        r.unlit = u;
    }
    r.cube(c + vec3(-0.6, 1.0, 0.), vec3(0.08, 0.32, 0.03), sh(hull, 0.8)); // antenna mast
    r.cube(c + vec3(-0.6, 1.32, 0.), vec3(0.25, 0.03, 0.03), sh(hull, 0.8));
    // lowered ramp to the holding pod, with a warm light inside
    let ramp = Color::new(0.48, 0.5, 0.52, 1.);
    r.quad(
        vec3(cx - 0.5, 0.75, cz + 1.25),
        vec3(cx + 0.5, 0.75, cz + 1.25),
        vec3(cx + 0.5, 0.02, cz + 2.55),
        vec3(cx - 0.5, 0.02, cz + 2.55),
        vec3(0., 0.87, 0.5),
        ramp,
    );
    for k in 0..4 {
        let f = (k as f32 + 0.5) / 4.;
        r.cube(vec3(cx, 0.75 - 0.73 * f + 0.01, cz + 1.25 + 1.3 * f), vec3(0.48, 0.008, 0.02), sh(ramp, 0.75));
    }
    let u = r.unlit;
    r.unlit = true;
    r.cube(vec3(cx, 1.05, cz + 1.28), vec3(0.42, 0.3, 0.02), Color::new(1., 0.8 + 0.1 * night, 0.5, 1.));
    r.unlit = u;
    labels.push((vec3(cx, 2.6, cz), "Your ship".into(), Color::new(0.85, 0.92, 1., 1.)));
}

/// The companion: a round little sand-hopper with long ears and glowing antenna tips. It hops, and its tips
/// flash when the bounty is close.
fn sand_hopper(r: &mut R, p: Vec2, t: f32, alert: bool) {
    let hop = ((t * 5.).sin().abs()) * 0.12;
    let base = vec3(p.x, hop, p.y);
    let fur = Color::new(0.92, 0.66, 0.4, 1.);
    r.disc(vec3(p.x, 0.015, p.y), vec2(0.16, 0.13), Color::new(0., 0., 0., 0.18));
    r.sphere(base + vec3(0., 0.2, 0.), vec3(0.17, 0.16, 0.17), fur);
    r.sphere(base + vec3(0., 0.17, 0.11), vec3(0.11, 0.09, 0.07), Color::new(0.98, 0.88, 0.72, 1.)); // pale belly
    for sx in [-0.06, 0.06] {
        r.sphere(base + vec3(sx, 0.26, 0.15), Vec3::splat(0.028), Color::new(0.1, 0.08, 0.08, 1.)); // eyes
        r.seg(base + vec3(sx * 1.3, 0.32, 0.), base + vec3(sx * 2.6, 0.55, -0.05), 0.022, sh(fur, 0.9)); // ears
        r.seg(
            base + vec3(sx * 0.8, 0.34, 0.06),
            base + vec3(sx * 1.4, 0.5, 0.12),
            0.008,
            Color::new(0.3, 0.2, 0.15, 1.),
        );
        let u = r.unlit;
        r.unlit = true;
        let glow =
            if alert && (t * 8.).sin() > 0. { Color::new(1., 0.35, 0.3, 1.) } else { Color::new(0.6, 1., 0.8, 1.) };
        r.sphere(base + vec3(sx * 1.4, 0.51, 0.12), Vec3::splat(0.025), glow); // antenna tips
        r.unlit = u;
    }
    for sx in [-0.08, 0.08] {
        r.sphere(base + vec3(sx, 0.04, 0.05), vec3(0.05, 0.035, 0.07), sh(fur, 0.85));
        // feet
    }
}

/// Armour pieces the player has earned, the wrist tracker, and stars while dazed.
fn hunter_gear(r: &mut R, g: &Game, t: f32) {
    let p = vec3(g.me.ipos.x, 0., g.me.ipos.y);
    let yaw = yaw_of(g.me.face);
    let plate = Color::new(0.58, 0.62, 0.68, 1.);
    let trim = Color::new(0.8, 0.55, 0.3, 1.);
    let a = g.w.p.armor;
    let hunting = g.bounty_dist();
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), p), |r| {
        if a & 1 != 0 {
            r.sphere(vec3(0., 1.07, 0.), vec3(0.33, 0.31, 0.33), plate); // helmet shell
            r.cube(vec3(0., 1.05, 0.29), vec3(0.22, 0.05, 0.04), Color::new(0.12, 0.14, 0.18, 1.)); // visor slit
            r.cube(vec3(0., 1.36, 0.), vec3(0.03, 0.05, 0.22), trim); // crest
        }
        if a & 2 != 0 {
            r.cube(vec3(0., 0.6, 0.06), vec3(0.2, 0.13, 0.12), plate); // chest plate
            r.cube(vec3(0., 0.6, 0.19), vec3(0.12, 0.02, 0.01), trim);
            for sx in [-0.22, 0.22] {
                r.sphere(vec3(sx, 0.72, 0.), vec3(0.09, 0.06, 0.09), sh(plate, 0.9));
                // shoulder pads
            }
        }
        if a & 4 != 0 {
            for sx in [-0.24, 0.24] {
                r.cube(vec3(sx, 0.48, 0.03), vec3(0.05, 0.07, 0.05), plate); // gauntlets
            }
        }
        if a & 8 != 0 {
            for sx in [-0.09, 0.09] {
                r.cube(vec3(sx, 0.07, 0.02), vec3(0.07, 0.07, 0.09), plate); // boots
            }
        }
        if let Some(d) = hunting {
            // wrist tracker: a brass dial whose light blinks faster, and turns from amber to red, as you close in
            r.cube(vec3(-0.25, 0.5, 0.06), vec3(0.04, 0.025, 0.05), Color::new(0.75, 0.6, 0.3, 1.));
            let rate = (12. / d.max(0.5)).clamp(1.5, 14.);
            if (t * rate).sin() > 0. {
                let u = r.unlit;
                r.unlit = true;
                let c = if d < 6. { Color::new(1., 0.2, 0.15, 1.) } else { Color::new(1., 0.7, 0.2, 1.) };
                r.sphere(vec3(-0.25, 0.53, 0.06), Vec3::splat(0.028), c);
                r.unlit = u;
            }
        }
    });
    if g.me.dazed > 0. {
        let u = r.unlit;
        r.unlit = true;
        for k in 0..3 {
            let a = t * 6. + k as f32 * 2.1;
            r.sphere(p + vec3(a.cos() * 0.3, 1.55, a.sin() * 0.3), Vec3::splat(0.05), Color::new(1., 0.9, 0.4, 1.));
        }
        r.unlit = u;
    }
}

/// The guild master's forge: armour pieces for guild marks, with a rumour about your current target.
pub(super) fn forge_menu(g: &Game, sel: usize, sw: f32) {
    let pw = (sw - 40.).min(700.);
    let x0 = sw / 2. - pw / 2.;
    panel(x0, 70., pw, 420.);
    txt("Guild Master Varro", x0 + 20., 105., 24., Color::new(0.45, 0.3, 0.2, 1.));
    let rumour = match g.w.p.bounty.as_ref().filter(|b| !b.caught) {
        Some(b) => format!("\"{}\"", RUMOURS[b.tier as usize]),
        None => "\"Bring them in, and I'll make you armour worth wearing.\"".into(),
    };
    for (k, l) in wrap_txt(&rumour, ((pw - 50.) / 10.) as usize).iter().enumerate() {
        txt(l, x0 + 20., 132. + k as f32 * 20., 17., DARKGRAY);
    }
    txt(&format!("Guild marks: {}", g.w.p.marks), x0 + 20., 190., 20., Color::new(0.45, 0.3, 0.2, 1.));
    let mut items: Vec<String> = ARMOR
        .iter()
        .enumerate()
        .map(
            |(k, (n, c))| {
                if g.w.p.armor & (1 << k) != 0 {
                    format!("{n}  (worn)")
                } else {
                    format!("Forge {n}  -  {c} marks")
                }
            },
        )
        .collect();
    items.push("Close".into());
    list(&items, Some(sel), x0 + 30., 230.);
    txt("Earn marks by bringing in bounties: 1 / 2 / 3 by difficulty.", x0 + 30., 400., 17., DARKGRAY);
    txt("W/S choose   Enter forge   Esc close", x0 + 30., 470., 18., Color::new(0.45, 0.3, 0.2, 1.));
}

/// A hover-bike: a long low body, twin forward engine pods, a seat and a soft blue hover glow underneath.
fn hover_bike(r: &mut R, p: Vec2, yaw: f32, t: f32) {
    let bob = (t * 2.2).sin() * 0.03;
    let paint = Color::new(0.62, 0.22, 0.18, 1.);
    let metal = Color::new(0.45, 0.47, 0.5, 1.);
    let u = r.unlit;
    r.unlit = true;
    r.disc(vec3(p.x, 0.02, p.y), vec2(0.45, 0.45), Color::new(0.45, 0.8, 1., 0.25));
    r.unlit = u;
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), vec3(p.x, 0.3 + bob, p.y)), |r| {
        r.sphere(Vec3::ZERO, vec3(0.2, 0.12, 0.62), paint); // body
        r.cube(vec3(0., 0.1, -0.12), vec3(0.11, 0.04, 0.2), Color::new(0.25, 0.18, 0.14, 1.)); // seat
        for sx in [-0.16, 0.16] {
            r.with(
                Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    vec3(sx, 0.02, 0.45),
                ),
                |r| {
                    r.cyl(vec3(0., -0.25, 0.), 0.07, 0.5, metal); // engine pods
                },
            );
            let u = r.unlit;
            r.unlit = true;
            r.sphere(vec3(sx, 0.02, 0.18), Vec3::splat(0.05), Color::new(0.5, 0.85, 1., 1.));
            r.unlit = u;
        }
        r.seg(vec3(-0.14, 0.2, 0.18), vec3(0.14, 0.2, 0.18), 0.015, metal); // handlebars
        r.cube(vec3(0., -0.1, -0.5), vec3(0.06, 0.04, 0.1), metal); // tail fin
    });
}

/// Everyday Dustreach: townsfolk, pack beasts plodding down the street, a helper robot at the market and
/// small craft crossing the sky.
fn frontier_life(r: &mut R, g: &Game, t: f32, labels: &mut Vec<(Vec3, String, Color)>) {
    let looks = [(Some(68), Some(67), 2), (Some(64), None, 1), (Some(66), Some(68), 0), (Some(69), None, 3)];
    for i in 0..LOCALS.len() {
        let p = local_pos(i, t);
        let near = g.me.ipos.distance(p) < 3.;
        let face = if near { g.me.ipos - p } else { local_pos(i, t + 0.5) - p };
        r.disc(vec3(p.x, 0.02, p.y), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.18));
        human(
            r,
            1,
            looks[i],
            vec3(p.x, 0., p.y),
            yaw_of(face.normalize_or_zero()),
            t * 2.,
            if near { 0. } else { 0.6 },
            0,
            None,
            t,
            0.,
        );
        if i == INFORMANT {
            r.sphere(vec3(p.x, 1.08, p.y), vec3(0.34, 0.33, 0.34), Color::new(0.3, 0.26, 0.24, 1.));
            // hooded cloak
        }
        if near {
            labels.push((vec3(p.x, 1.9, p.y), LOCALS[i].1.into(), Color::new(1., 0.9, 0.75, 1.)));
        }
    }
    // a blue hologram of your current target flickering above the bounty board
    if let Some(b) = g.w.p.bounty.as_ref().filter(|b| !b.caught) {
        let (bx, bz) = (F_BOARD.0 as f32 + 0.5, F_BOARD.1 as f32 + 0.5);
        let (a, u) = (r.alpha, r.unlit);
        r.unlit = true;
        r.alpha = 0.45 + 0.15 * (t * 9.).sin().abs() * if (t * 3.).fract() < 0.08 { 0. } else { 1. };
        let holo = Color::new(0.45, 0.8, 1., 1.);
        r.disc(vec3(bx, 1.45, bz + 0.2), vec2(0.28, 0.2), holo); // projector beam base
        r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(t * 0.8), vec3(bx, 1.5, bz + 0.2)), |r| {
            r.sphere(vec3(0., 0.62, 0.), vec3(0.13, 0.13, 0.13), holo); // head
            r.cube(vec3(0., 0.36, 0.), vec3(0.12, 0.13, 0.08), holo); // body
            r.cube(vec3(0., 0.1, 0.), vec3(0.08, 0.13, 0.06), holo); // legs
            if b.tier == 2 {
                r.cube(vec3(0., 0.5, 0.), vec3(0.14, 0.03, 0.1), Color::new(1., 0.5, 0.5, 1.));
                // the tell-tale scarf
            }
        });
        for k in 0..4 {
            let y = 1.5 + ((t * 0.7 + k as f32 * 0.25) % 1.) * 0.85;
            r.quad(
                vec3(bx - 0.22, y, bz + 0.2),
                vec3(bx + 0.22, y, bz + 0.2),
                vec3(bx + 0.22, y + 0.012, bz + 0.2),
                vec3(bx - 0.22, y + 0.012, bz + 0.2),
                Vec3::Z,
                holo,
            ); // scan lines
        }
        r.alpha = a;
        r.unlit = u;
    }
    // the Longhouse band: three alien musicians with a horn, a drum and a long-necked lute
    let skins = [Color::new(0.45, 0.7, 0.5, 1.), Color::new(0.6, 0.5, 0.75, 1.), Color::new(0.85, 0.6, 0.4, 1.)];
    for (k, x) in [24.7f32, 25.6, 26.5].iter().enumerate() {
        let bob = ((t * 4. + k as f32 * 1.3).sin()).abs() * 0.06;
        let p = vec3(*x, bob, 9.55 + (k % 2) as f32 * 0.3);
        let skin = skins[k];
        r.disc(vec3(p.x, 0.02, p.z), vec2(0.26, 0.2), Color::new(0., 0., 0., 0.18));
        r.cube(p + vec3(0., 0.42, 0.), vec3(0.17, 0.28, 0.12), Color::new(0.35, 0.28, 0.24, 1.)); // robe
        r.sphere(p + vec3(0., 0.85, 0.), vec3(0.2, 0.24, 0.2), skin); // big domed head
        for sx in [-0.08, 0.08] {
            r.sphere(p + vec3(sx, 0.9, 0.17), vec3(0.05, 0.06, 0.03), Color::new(0.08, 0.06, 0.1, 1.));
            // large dark eyes
        }
        if k == 0 {
            r.seg(p + vec3(0.1, 0.62, 0.15), p + vec3(0.32, 0.5, 0.35), 0.03, GOLD); // horn
            r.cone(p + vec3(0.32, 0.5, 0.35), 0.09, -0.12, GOLD);
        } else if k == 1 {
            r.cyl(p + vec3(0., 0.15, 0.25), 0.16, 0.28, Color::new(0.6, 0.35, 0.2, 1.)); // drum
            r.disc(p + vec3(0., 0.44, 0.25), vec2(0.15, 0.15), Color::new(0.92, 0.85, 0.7, 1.));
        } else {
            r.seg(p + vec3(-0.15, 0.4, 0.15), p + vec3(0.3, 0.95, 0.1), 0.02, Color::new(0.55, 0.36, 0.2, 1.)); // lute neck
            r.sphere(p + vec3(-0.1, 0.42, 0.16), vec3(0.13, 0.13, 0.05), Color::new(0.65, 0.42, 0.24, 1.));
        }
    }
    let lx = 25.6;
    for k in 0..3 {
        let ph = (t * 0.6 + k as f32 * 0.33) % 1.;
        let u = r.unlit;
        r.unlit = true;
        r.sphere(
            vec3(lx - 0.4 + k as f32 * 0.4 + (ph * 6.).sin() * 0.1, 1.3 + ph * 0.9, 9.6),
            Vec3::splat(0.04 * (1. - ph)),
            Color::new(1., 0.9, 0.5, 1.),
        ); // drifting notes
        r.unlit = u;
    }
    if g.me.ipos.distance(vec2(lx, 9.7)) < 3.5 {
        labels.push((vec3(lx, 1.6, 9.7), "The Longhouse Trio".into(), Color::new(1., 0.9, 0.7, 1.)));
    }
    // two shaggy pack beasts with saddle bags, walking the cross street
    for k in 0..2 {
        let ph = t * 0.045 + k as f32 * 3.1;
        let x = 18. + ph.sin() * 11.;
        let dir = if ph.cos() > 0. { 1. } else { -1. };
        let p = vec3(x, 0., 9.35 + k as f32 * 1.6);
        let fur = Color::new(0.5, 0.38, 0.27, 1.);
        r.disc(vec3(p.x, 0.02, p.z), vec2(0.7, 0.4), Color::new(0., 0., 0., 0.18));
        r.with(
            Mat4::from_rotation_translation(
                Quat::from_rotation_y(if dir > 0. {
                    std::f32::consts::FRAC_PI_2
                } else {
                    -std::f32::consts::FRAC_PI_2
                }),
                p,
            ),
            |r| {
                r.sphere(vec3(0., 0.8, 0.), vec3(0.42, 0.4, 0.75), fur);
                r.sphere(vec3(0., 1.15, -0.1), vec3(0.28, 0.22, 0.32), sh(fur, 0.9)); // hump
                r.sphere(vec3(0., 0.9, 0.78), vec3(0.2, 0.2, 0.26), sh(fur, 1.1)); // head
                for sx in [-0.14, 0.14] {
                    r.seg(vec3(sx, 1.02, 0.85), vec3(sx * 2.2, 1.18, 0.75), 0.03, Color::new(0.85, 0.8, 0.7, 1.));
                    // horns
                }
                for (j, (sx, sz)) in [(-0.25, 0.45), (0.25, 0.45), (-0.25, -0.45), (0.25, -0.45)].iter().enumerate() {
                    let swing = (t * 4. + j as f32 * 1.6).sin() * 0.12;
                    r.seg(vec3(*sx, 0.55, *sz), vec3(*sx, 0.0, sz + swing), 0.06, sh(fur, 0.8));
                    // legs
                }
                for sx in [-0.45, 0.45] {
                    r.cube(vec3(sx, 0.75, 0.0), vec3(0.08, 0.18, 0.25), Color::new(0.6, 0.5, 0.32, 1.));
                    // saddle bags
                }
            },
        );
    }
    // a helper robot trundling between the market tents
    let rx = 8.5 + (t * 0.3).sin() * 2.2;
    let rdir = if (t * 0.3).cos() > 0. { 1. } else { -1. };
    let rp = vec3(rx, 0., 13.6);
    r.disc(vec3(rp.x, 0.02, rp.z), vec2(0.26, 0.2), Color::new(0., 0., 0., 0.18));
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(rdir * std::f32::consts::FRAC_PI_2), rp), |r| {
        let shell = Color::new(0.86, 0.82, 0.72, 1.);
        r.cube(vec3(0., 0.42, 0.), vec3(0.2, 0.24, 0.18), shell);
        r.cube(vec3(0., 0.42, 0.19), vec3(0.12, 0.1, 0.01), Color::new(0.75, 0.45, 0.25, 1.)); // panel
        r.sphere(vec3(0., 0.74, 0.), vec3(0.17, 0.12, 0.17), sh(shell, 0.92)); // dome
        let u = r.unlit;
        r.unlit = true;
        r.sphere(vec3(0., 0.76, 0.15), Vec3::splat(0.045), Color::new(0.4, 0.9, 1., 1.)); // eye
        r.unlit = u;
        r.seg(vec3(0.08, 0.84, 0.), vec3(0.12, 1.05, -0.03), 0.01, Color::new(0.3, 0.3, 0.32, 1.)); // antenna
        for sx in [-0.21, 0.21] {
            r.with(
                Mat4::from_rotation_translation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), vec3(sx, 0.12, 0.)),
                |r| {
                    r.cyl(vec3(0., -0.04, 0.), 0.12, 0.08, Color::new(0.25, 0.25, 0.27, 1.));
                    // wheels
                },
            );
        }
    });
    // small craft crossing the sky beyond the north cliffs
    let ph = (t % 40.) / 40.;
    let sx = 46. - ph * 54.;
    let sp = vec3(sx, 6.2 + (ph * 6.).sin() * 0.3, -1.5);
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2), sp), |r| {
        let body = Color::new(0.7, 0.72, 0.75, 1.);
        r.sphere(Vec3::ZERO, vec3(0.3, 0.22, 0.9), body);
        r.cube(vec3(0., 0., -0.1), vec3(1.0, 0.04, 0.3), sh(body, 0.85)); // wings
        let u = r.unlit;
        r.unlit = true;
        r.sphere(vec3(0., 0., -0.9), Vec3::splat(0.16), Color::new(1., 0.7, 0.4, 1.)); // engine glow
        r.unlit = u;
    });
}

/// One Dustreach structure (kind as in F_STRUCTS). Drawn normally, or faded after the player when it blocks the view.
fn draw_struct(
    r: &mut R,
    kind: u8,
    x: i32,
    z: i32,
    w: i32,
    d: i32,
    t: f32,
    night: f32,
    labels: &mut Vec<(Vec3, String, Color)>,
) {
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
            labels.push((vec3(fx + fw / 2., 2.75, fz + fd / 2.), "Longhouse".into(), Color::new(1., 0.86, 0.6, 1.)));
        }
        2 => {
            let c = vec3(fx + fw / 2., 0., fz + fd / 2.);
            for (dx, dz) in [(-0.8, -0.8), (0.8, -0.8), (-0.8, 0.8), (0.8, 0.8)] {
                r.cube(c + vec3(dx, 0.6, dz), vec3(0.03, 0.6, 0.03), BEAM);
            }
            let tone = [Color::new(0.75, 0.3, 0.25, 1.), Color::new(0.3, 0.5, 0.6, 1.), Color::new(0.8, 0.6, 0.25, 1.)]
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
            r.with(Mat4::from_rotation_translation(Quat::from_rotation_z(1.35) * Quat::from_rotation_y(0.3), c), |r| {
                r.cyl(vec3(0., -2.2, 0.), 0.85, 4.4, hull);
                r.cone(vec3(0., 2.2, 0.), 0.85, 0.9, sh(hull, 0.9));
                for k in 0..4 {
                    r.cyl(vec3(0., -1.6 + k as f32 * 1.1, 0.), 0.88, 0.08, sh(hull, 0.7));
                    // hull ribs
                }
            });
            r.cube(c + vec3(-1.4, 0.5, 1.0), vec3(0.6, 0.04, 0.35), sh(hull, 0.8)); // broken fin
            r.cube(c + vec3(1.2, 0.15, -1.1), vec3(0.3, 0.15, 0.2), Color::new(0.25, 0.22, 0.2, 1.));
            // scorched panel
        }
        6 => ship(r, fx + fw / 2., fz + fd / 2., t, night, labels),
        7 => {
            // a heap of salvage: tilted hull plates, a drum, a big wheel and a bent pipe
            let c = vec3(fx + fw / 2., 0., fz + fd / 2.);
            let metal =
                [Color::new(0.55, 0.56, 0.58, 1.), Color::new(0.62, 0.4, 0.28, 1.), Color::new(0.42, 0.45, 0.4, 1.)];
            for k in 0..5 {
                let a = k as f32 * 1.3 + fx;
                r.with(
                    Mat4::from_rotation_translation(
                        Quat::from_rotation_y(a) * Quat::from_rotation_z(0.5 + (k % 2) as f32 * 0.4),
                        c + vec3(a.cos() * 0.5, 0.35, a.sin() * 0.4),
                    ),
                    |r| r.cube(Vec3::ZERO, vec3(0.45, 0.04, 0.3), metal[k % 3]),
                );
            }
            r.cyl(c + vec3(fw / 2. - 0.4, 0., 0.2), 0.22, 0.55, metal[1]);
            r.with(
                Mat4::from_rotation_translation(
                    Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                    c + vec3(-fw / 2. + 0.5, 0.45, -0.3),
                ),
                |r| r.cyl(vec3(0., -0.06, 0.), 0.45, 0.12, Color::new(0.2, 0.2, 0.22, 1.)),
            );
            r.seg(c + vec3(-0.2, 0.1, 0.5), c + vec3(0.6, 0.9, 0.3), 0.06, metal[2]);
            if night > 0.3 {
                lantern(r, c + vec3(0., 1.1, 0.), night);
            }
            labels.push((c + vec3(0., 1.4, 0.), "Scrapyard".into(), Color::new(1., 0.86, 0.6, 1.)));
        }
        8 => {
            // the old watchtower: a broken stone drum, a leaning platform and a torn banner
            let c = vec3(fx + 1., 0., fz + 1.);
            let stone = Color::new(0.7, 0.6, 0.5, 1.);
            r.cyl(c, 0.85, 1.6, stone);
            r.cyl(c + vec3(0., 1.6, 0.), 0.8, 0.7, sh(stone, 0.92));
            for k in 0..5 {
                let a = k as f32 * 1.25;
                r.cube(
                    c + vec3(a.cos() * 0.72, 2.45 + (k % 2) as f32 * 0.15, a.sin() * 0.72),
                    vec3(0.14, 0.15 + (k % 2) as f32 * 0.1, 0.14),
                    sh(stone, 0.85),
                ); // ragged crown
            }
            r.cube(c + vec3(0., 0.55, 0.84), vec3(0.24, 0.5, 0.03), Color::new(0.2, 0.14, 0.1, 1.)); // doorway
            r.with(Mat4::from_rotation_translation(Quat::from_rotation_z(0.15), c + vec3(0.9, 2.0, 0.)), |r| {
                r.cube(Vec3::ZERO, vec3(0.5, 0.04, 0.4), BEAM)
            });
            let wave = (t * 2.).sin() * 0.06;
            r.cube(c + vec3(0.15, 3.0, 0.), vec3(0.02, 0.5, 0.02), BEAM);
            r.quad(
                c + vec3(0.17, 3.4, 0.),
                c + vec3(0.65, 3.35 + wave, 0.),
                c + vec3(0.6, 3.1 + wave, 0.),
                c + vec3(0.17, 3.15, 0.),
                Vec3::Z,
                Color::new(0.72, 0.22, 0.18, 1.),
            );
            labels.push((c + vec3(0., 3.7, 0.), "Old Watchtower".into(), Color::new(1., 0.86, 0.6, 1.)));
        }
        _ => {
            let c = vec3(fx + 1., 0., fz + 1.);
            r.sphere(c, vec3(0.95, 1.05, 0.95), ADOBE);
            r.cube(c + vec3(0., 0.4, 0.9), vec3(0.22, 0.4, 0.06), Color::new(0.28, 0.18, 0.12, 1.));
            r.cyl(c + vec3(0.3, 0.85, -0.2), 0.08, 0.35, sh(ADOBE, 0.85)); // little chimney
        }
    }
}

/// Dustreach sky (2D, behind the world): a huge ringed gas giant and a small pale moon hanging over the canyon,
/// which drift slowly with the hour and glow brighter as the light fades.
pub(super) fn frontier_sky(g: &Game, lt: &Light, w: f32, h: f32) {
    let k = 1. - lt.night * 0.5;
    let glow = 0.6 + lt.night * 0.35;
    let drift = (g.hr as f32 + g.t / 3600.) * 6.;
    // the gas giant: soft banded disc partly hidden behind its ring
    let (px, py, pr) = (w * 0.72 - drift, h * 0.17, h * 0.13);
    let band =
        [Color::new(0.93, 0.78, 0.62, glow), Color::new(0.85, 0.62, 0.5, glow), Color::new(0.96, 0.86, 0.72, glow)];
    draw_circle(px, py, pr, band[0]);
    for i in 0..6 {
        let y = py - pr + (i as f32 + 0.5) * pr / 3.;
        let dy = (y - py).abs() + pr / 12.; // band edge furthest from the centre stays inside the disc
        let half = (pr * pr - dy * dy).max(0.).sqrt();
        draw_rectangle(px - half, y - pr / 12., half * 2., pr / 6., band[i % 3]);
    }
    draw_circle(px - pr * 0.35, py - pr * 0.3, pr * 0.18, Color::new(1., 0.95, 0.85, glow * 0.4)); // highlight
    let ring = Color::new(0.98, 0.9, 0.78, glow * 0.85);
    for j in 0..3 {
        let rx = pr * (1.75 + j as f32 * 0.12);
        let ry = pr * (0.32 + j as f32 * 0.02);
        let steps = 60;
        for s in 0..steps {
            let (a0, a1) = (
                s as f32 / steps as f32 * std::f32::consts::TAU,
                (s + 1) as f32 / steps as f32 * std::f32::consts::TAU,
            );
            let (x0, y0) = (px + a0.cos() * rx, py + a0.sin() * ry * 1. - a0.cos() * ry * 0.5);
            let (x1, y1) = (px + a1.cos() * rx, py + a1.sin() * ry * 1. - a1.cos() * ry * 0.5);
            let front = a0.sin() > 0.;
            if front || ((x0 - px).powi(2) + (y0 - py).powi(2)).sqrt() > pr {
                draw_line(x0, y0, x1, y1, 2.2 - j as f32 * 0.5, ring);
            }
        }
    }
    // a small pale moon
    let (mx, my) = (w * 0.9 - drift * 0.3, h * 0.07); // clear of the info panel at top left
    draw_circle(mx, my, h * 0.028, Color::new(0.95 * k + 0.05, 0.95 * k + 0.05, 1., glow + 0.1));
    draw_circle(mx + h * 0.008, my - h * 0.004, h * 0.007, Color::new(0.75, 0.75, 0.82, glow));
    // crater
}

/// Dust devils spinning across the salt flats, and the bleached ribcage of some enormous desert beast.
pub(super) fn frontier_wilds(r: &mut R, t: f32) {
    let a = r.alpha;
    for k in 0..2 {
        let ph = t * 0.06 + k as f32 * 2.7;
        let base = vec3(52. + ph.sin() * 5., 0., 22. + (ph * 1.3).cos() * 4.);
        r.alpha = 0.22;
        for j in 0..7 {
            let y = j as f32 * 0.32;
            let ang = t * 6. + j as f32 * 0.9;
            let rad = 0.12 + j as f32 * 0.09;
            r.sphere(
                base + vec3(ang.cos() * rad * 0.4, y, ang.sin() * rad * 0.4),
                vec3(rad, 0.14, rad),
                Color::new(0.92, 0.82, 0.66, 1.),
            );
        }
    }
    r.alpha = a;
    // the old bones half sunk in the flats
    let bone = Color::new(0.93, 0.9, 0.82, 1.);
    let c = vec3(56.5, 0., 26.);
    r.seg(c + vec3(-2.2, 0.15, 0.), c + vec3(2.0, 0.25, 0.), 0.09, bone); // spine
    for i in 0..6 {
        let x = -1.6 + i as f32 * 0.65;
        let hgt = 1.3 - (i as f32 - 2.5).abs() * 0.18;
        for sz in [-1f32, 1.] {
            r.seg(c + vec3(x, 0.25, 0.), c + vec3(x + 0.1, hgt, sz * 0.55), 0.05, bone);
            r.seg(c + vec3(x + 0.1, hgt, sz * 0.55), c + vec3(x + 0.18, 0.0, sz * 0.95), 0.045, bone);
        }
    }
    r.sphere(c + vec3(2.5, 0.35, 0.), vec3(0.45, 0.32, 0.38), bone); // skull
    r.sphere(c + vec3(2.75, 0.4, 0.18), Vec3::splat(0.09), Color::new(0.25, 0.2, 0.18, 1.));
    r.seg(c + vec3(2.7, 0.55, 0.2), c + vec3(3.2, 1.0, 0.45), 0.05, bone); // horn
}

/// The natural stone arch over the oasis trail (faded and drawn after the player when it is in the way).
fn draw_arch(r: &mut R) {
    let (ax, az) = (F_ARCH.0 as f32, F_ARCH.1 as f32);
    for px in [ax, ax + 2.] {
        mesa(r, px, az, 2.4, hash(px as i32, 7));
    }
    r.cube(vec3(ax + 1.5, 2.55, az + 0.5), vec3(1.55, 0.22, 0.42), sh(ROCK_C, 0.95));
    r.cube(vec3(ax + 1.5, 2.36, az + 0.5), vec3(1.2, 0.06, 0.38), sh(ROCK_C, 0.8));
}
