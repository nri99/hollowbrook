//! Interiors: homes (all stories), resident rooms, shop, museum, Town Hall.
use super::*;

pub(super) fn room(r: &mut R, g: &Game, b: usize, lt: &Light, labels: &mut Vec<(Vec3, String, Color)>) {
    let (w, d) = g.rdims(b);
    let (wf, df) = (w as f32, d as f32);
    let t = g.t;
    let (fl, wl) = match BLDS[b].kind {
        BK::Shop => ((0.72, 0.76, 0.66), (0.93, 0.95, 0.86)),
        BK::Civic => ((0.5, 0.36, 0.28), (0.9, 0.87, 0.78)),
        BK::Res(i) => THEME[i],
        BK::Home(o) => {
            let pr = g.owner(o);
            (FLOORS[pr.floor as usize % 4], WALLS[pr.wall as usize % 4])
        }
        _ => ((0.78, 0.6, 0.42), (0.97, 0.93, 0.84)),
    };
    let st = g.story_in(b);
    let stair = g.home_stairs(b);
    let (fl, wl) = match st {
        1 => ((0.6, 0.56, 0.5), (0.7, 0.5, 0.42)),
        2 => ((0.84, 0.7, 0.52), (0.88, 0.9, 0.92)),
        _ => (fl, wl),
    };
    let (floor, wall) = (rgb(fl), rgb(wl));
    let furn = g.room_furn(b);
    let lamp = furn.iter().any(|f| def(f.def).act == Act::Lamp && f.on);
    let dim = lt.night > 0.6 && !lamp;
    r.sun = vec3(0.35, 0.85, 0.5).normalize();
    r.amb = if dim {
        vec3(0.3, 0.32, 0.48)
    } else if lamp && lt.night > 0.3 {
        vec3(0.68, 0.57, 0.42)
    } else {
        vec3(0.56, 0.56, 0.56)
    };
    r.dif = if dim { vec3(0.14, 0.14, 0.22) } else { vec3(0.44, 0.42, 0.4) };
    if st == 1 {
        r.amb *= 0.86;
        r.dif *= 0.8
    } else if st == 2 {
        r.amb *= 1.05
    }
    let tiled = matches!(BLDS[b].kind, BK::Shop | BK::Museum) || st == 1;
    for z in 0..d {
        for x in 0..w {
            let (fx, fz) = (x as f32, z as f32);
            if stair.iter().any(|s| s.0 == (x, z) && elev(s.1) < elev(st)) {
                continue;
            } // stairwell opening
            if tiled {
                r.quad(
                    vec3(fx, 0., fz),
                    vec3(fx + 1., 0., fz),
                    vec3(fx + 1., 0., fz + 1.),
                    vec3(fx, 0., fz + 1.),
                    Vec3::Y,
                    sh(floor, if (x + z) % 2 == 0 { 1. } else { 0.95 }),
                );
                r.quad(
                    vec3(fx, 0.002, fz),
                    vec3(fx + 1., 0.002, fz),
                    vec3(fx + 1., 0.002, fz + 0.025),
                    vec3(fx, 0.002, fz + 0.025),
                    Vec3::Y,
                    sh(floor, 0.82),
                );
                r.quad(
                    vec3(fx, 0.002, fz),
                    vec3(fx + 0.025, 0.002, fz),
                    vec3(fx + 0.025, 0.002, fz + 1.),
                    vec3(fx, 0.002, fz + 1.),
                    Vec3::Y,
                    sh(floor, 0.82),
                )
            } else {
                // wooden boards: three planks per tile running left-right, staggered seams, gentle per-plank tone
                for k in 0..3 {
                    let (z0, z1) = (fz + k as f32 / 3., fz + (k + 1) as f32 / 3.);
                    let v = 0.94 + (hash(x * 3 + k * 7, z * 5 + k) % 9) as f32 * 0.012;
                    r.quad(
                        vec3(fx, 0., z0),
                        vec3(fx + 1., 0., z0),
                        vec3(fx + 1., 0., z1),
                        vec3(fx, 0., z1),
                        Vec3::Y,
                        sh(floor, v),
                    );
                    r.quad(
                        vec3(fx, 0.002, z0),
                        vec3(fx + 1., 0.002, z0),
                        vec3(fx + 1., 0.002, z0 + 0.018),
                        vec3(fx, 0.002, z0 + 0.018),
                        Vec3::Y,
                        sh(floor, 0.78),
                    );
                    let sx = fx + if (k + x) % 2 == 0 { 0.33 } else { 0.8 };
                    r.quad(
                        vec3(sx, 0.002, z0),
                        vec3(sx + 0.018, 0.002, z0),
                        vec3(sx + 0.018, 0.002, z1),
                        vec3(sx, 0.002, z1),
                        Vec3::Y,
                        sh(floor, 0.8),
                    )
                }
            }
        }
    }
    // back wall: wainscot panel below a chair rail, softly striped paper above
    r.quad(vec3(0., 0., 0.), vec3(wf, 0., 0.), vec3(wf, 0.85, 0.), vec3(0., 0.85, 0.), Vec3::Z, sh(wall, 0.86));
    r.quad(vec3(0., 0.85, 0.), vec3(wf, 0.85, 0.), vec3(wf, 2.4, 0.), vec3(0., 2.4, 0.), Vec3::Z, wall);
    if st == 1 {
        let mortar = sh(wall, 0.8);
        let mut row = 0;
        let mut y = 0.98;
        while y < 2.36 {
            r.quad(
                vec3(0., y, 0.005),
                vec3(wf, y, 0.005),
                vec3(wf, y + 0.02, 0.005),
                vec3(0., y + 0.02, 0.005),
                Vec3::Z,
                mortar,
            );
            let mut x = if row % 2 == 0 { 0.3 } else { 0. };
            while x < wf {
                r.quad(
                    vec3(x, y, 0.005),
                    vec3(x + 0.02, y, 0.005),
                    vec3(x + 0.02, (y + 0.2).min(2.38), 0.005),
                    vec3(x, (y + 0.2).min(2.38), 0.005),
                    Vec3::Z,
                    mortar,
                );
                x += 0.6
            }
            y += 0.2;
            row += 1
        }
        r.cube(vec3(wf / 2., 2.2, 0.08), vec3(wf / 2., 0.035, 0.035), Color::new(0.55, 0.42, 0.3, 1.));
    }
    // brick courses and an old pipe
    else {
        let mut x = 0.;
        let mut k = 0;
        while x < wf - 0.01 {
            if k % 2 == 1 {
                r.quad(
                    vec3(x, 0.88, 0.005),
                    vec3((x + 0.18).min(wf), 0.88, 0.005),
                    vec3((x + 0.18).min(wf), 2.34, 0.005),
                    vec3(x, 2.34, 0.005),
                    Vec3::Z,
                    sh(wall, 0.965),
                )
            }
            x += 0.18;
            k += 1
        }
    }
    r.cube(vec3(wf / 2., 0.86, 0.025), vec3(wf / 2., 0.025, 0.025), WOOD);
    {
        let mut x = 0.4;
        while x < wf - 0.2 {
            r.cube(vec3(x, 0.47, 0.006), vec3(0.008, 0.3, 0.006), sh(wall, 0.78));
            x += 0.8
        }
    }
    r.cube(vec3(wf / 2., 0.08, 0.03), vec3(wf / 2., 0.08, 0.03), WOOD);
    r.cube(vec3(wf / 2., 2.38, 0.03), vec3(wf / 2., 0.04, 0.04), WOOD);
    for x in [-0.08, wf + 0.08] {
        r.cube(vec3(x, 0.5, df / 2. - 0.04), vec3(0.08, 0.5, df / 2. + 0.04), sh(wall, 0.85))
    } // cut-away side walls
    let dxw = (w / 2) as f32;
    if st != 0 {
        r.cube(vec3(wf / 2., 0.12, df + 0.06), vec3(wf / 2., 0.12, 0.06), sh(wall, 0.8))
    } else {
        r.cube(vec3(dxw / 2., 0.12, df + 0.06), vec3(dxw / 2., 0.12, 0.06), sh(wall, 0.8));
        r.cube(vec3((dxw + 1. + wf) / 2., 0.12, df + 0.06), vec3((wf - dxw - 1.) / 2., 0.12, 0.06), sh(wall, 0.8));
        r.cube(vec3(dxw + 0.5, 0.005, df - 0.4), vec3(0.4, 0.005, 0.3), Color::new(0.7, 0.3, 0.3, 1.))
    } // door mat
    for &((sx, sz), to) in &stair {
        let (x0, z0) = (sx as f32, sz as f32);
        if elev(to) > elev(st) {
            // stairs climbing into the back wall, with a dark landing above and a rail
            for k in 0..6 {
                let h = (k + 1) as f32 * 0.38;
                let zc = z0 + 1. - (k as f32 + 0.5) / 6.;
                r.cube(vec3(x0 + 0.5, h / 2., zc), vec3(0.44, h / 2., 1. / 12.), sh(WOOD, 0.86 + k as f32 * 0.03));
                r.cube(vec3(x0 + 0.5, h + 0.01, zc + 0.07), vec3(0.45, 0.012, 0.02), sh(WOOD, 1.15))
            }
            r.quad(
                vec3(x0 + 0.06, 1.9, 0.004),
                vec3(x0 + 0.94, 1.9, 0.004),
                vec3(x0 + 0.94, 2.4, 0.004),
                vec3(x0 + 0.06, 2.4, 0.004),
                Vec3::Z,
                Color::new(0.16, 0.12, 0.1, 1.),
            );
            r.cube(vec3(x0 + 0.95, 0.47, z0 + 0.95), vec3(0.035, 0.47, 0.035), sh(WOOD, 0.7));
            r.cube(vec3(x0 + 0.95, 0.96, z0 + 0.95), vec3(0.05, 0.03, 0.05), sh(WOOD, 0.9)); // newel post
            r.with(Mat4::from_rotation_translation(Quat::from_rotation_x(1.0), vec3(x0 + 0.95, 1.65, z0 + 0.5)), |r| {
                r.cube(Vec3::ZERO, vec3(0.025, 0.025, 0.83), sh(WOOD, 0.75))
            });
        } else {
            // stairwell going down: steps sinking toward the back, shaded walls, a low rail
            let dark = |k: f32| Color::new(0.3 - k * 0.05, 0.24 - k * 0.04, 0.2 - k * 0.035, 1.);
            for k in 0..4 {
                let y = -(k as f32 + 1.) * 0.28;
                let (za, zb) = (z0 + 1. - (k + 1) as f32 * 0.25, z0 + 1. - k as f32 * 0.25);
                r.quad(
                    vec3(x0, y, za),
                    vec3(x0 + 1., y, za),
                    vec3(x0 + 1., y, zb),
                    vec3(x0, y, zb),
                    Vec3::Y,
                    sh(WOOD, 0.8 - k as f32 * 0.12),
                );
                r.quad(
                    vec3(x0, y, zb),
                    vec3(x0 + 1., y, zb),
                    vec3(x0 + 1., y + 0.28, zb),
                    vec3(x0, y + 0.28, zb),
                    Vec3::Z,
                    dark(k as f32),
                )
            }
            for x in [x0, x0 + 1.] {
                r.quad(
                    vec3(x, -1.2, z0),
                    vec3(x, -1.2, z0 + 1.),
                    vec3(x, 0., z0 + 1.),
                    vec3(x, 0., z0),
                    Vec3::X,
                    dark(1.5),
                )
            }
            r.quad(
                vec3(x0, -1.2, z0 + 0.002),
                vec3(x0 + 1., -1.2, z0 + 0.002),
                vec3(x0 + 1., 0., z0 + 0.002),
                vec3(x0, 0., z0 + 0.002),
                Vec3::Z,
                dark(3.),
            );
            let rx = if sx == 0 { x0 + 1. } else { x0 + 0.98 };
            for z in [z0 + 0.1, z0 + 0.9] {
                r.cube(vec3(rx, 0.32, z), vec3(0.025, 0.32, 0.025), sh(WOOD, 0.7))
            }
            r.cube(vec3(rx, 0.64, z0 + 0.5), vec3(0.03, 0.025, 0.45), WOOD);
        }
    }
    if st == 2 {
        let beam = sh(WOOD, 0.85);
        for (cx, a) in [(wf * 0.25, 0.3f32), (wf * 0.75, -0.3)] {
            r.with(Mat4::from_rotation_translation(Quat::from_rotation_z(a), vec3(cx, 1.95, 0.05)), |r| {
                r.cube(Vec3::ZERO, vec3(wf * 0.28, 0.05, 0.05), beam)
            })
        }
        r.cube(vec3(wf / 2., 2.33, 0.06), vec3(0.08, 0.08, 0.06), beam);
    } // attic rafters
    if st == 1 {
        let wx = wf - 0.9;
        r.cube(vec3(wx, 2.05, 0.04), vec3(0.36, 0.17, 0.04), WOOD);
        let u = r.unlit;
        r.unlit = true;
        r.cube(
            vec3(wx, 2.05, 0.07),
            vec3(0.3, 0.12, 0.02),
            Color::new(lt.sky.r * 0.8 + 0.1, lt.sky.g * 0.8 + 0.1, lt.sky.b * 0.8 + 0.1, 1.),
        );
        r.unlit = u;
        r.cube(vec3(wx, 2.05, 0.09), vec3(0.008, 0.12, 0.01), WOOD);
    } // small high window at ground level
    if !matches!(BLDS[b].kind, BK::Shop | BK::Museum) && st != 1 {
        let wx = if w > 3 && st != 2 { wf - 0.9 } else { wf / 2. };
        r.cube(vec3(wx, 1.45, 0.04), vec3(0.42, 0.36, 0.04), WOOD);
        let u = r.unlit;
        r.unlit = true;
        r.cube(
            vec3(wx, 1.45, 0.07),
            vec3(0.34, 0.28, 0.02),
            Color::new(lt.sky.r * 0.9 + 0.1, lt.sky.g * 0.9 + 0.1, lt.sky.b * 0.9 + 0.1, 1.),
        );
        let day = 1. - lt.night;
        if day > 0.05 {
            let a = 0.16 * day;
            let c = Color::new(1., 0.93, 0.7, a);
            r.quad(
                vec3(wx - 0.36, 0.004, 0.25),
                vec3(wx + 0.36, 0.004, 0.25),
                vec3(wx + 0.62, 0.004, 1.45),
                vec3(wx - 0.1, 0.004, 1.45),
                Vec3::Y,
                c,
            );
            r.quad(
                vec3(wx - 0.3, 0.005, 0.5),
                vec3(wx - 0.27, 0.005, 0.5),
                vec3(wx + 0.03, 0.005, 1.4),
                vec3(wx, 0.005, 1.4),
                Vec3::Y,
                Color::new(0.4, 0.3, 0.2, a * 0.6),
            );
        }
        r.unlit = u;
        if matches!(BLDS[b].kind, BK::Home(_)) {
            // Gathered linen panels, lifted above the glass (front face z = 0.09).
            let linen = Color::new(0.56, 0.72, 0.65, 1.);
            r.cube(vec3(wx, 1.845, 0.11), vec3(0.49, 0.015, 0.02), WOOD);
            for sx in [-1f32, 1.] {
                let cx = wx + sx * 0.33;
                r.quad(
                    vec3(cx - 0.05, 1.12, 0.118),
                    vec3(cx + 0.05, 1.12, 0.118),
                    vec3(cx + 0.09, 1.79, 0.118),
                    vec3(cx - 0.09, 1.79, 0.118),
                    Vec3::Z,
                    linen,
                );
                r.quad(
                    vec3(cx - 0.01, 1.12, 0.124),
                    vec3(cx + 0.01, 1.12, 0.124),
                    vec3(cx + 0.018, 1.79, 0.124),
                    vec3(cx - 0.018, 1.79, 0.124),
                    Vec3::Z,
                    sh(linen, 0.88),
                );
            }
            r.cube(vec3(wx, 1.065, 0.11), vec3(0.48, 0.03, 0.08), WOOD);
        }
    }
    if BLDS[b].kind == BK::Civic {
        r.cube(vec3(3.5, 0.005, 3.2), vec3(0.6, 0.005, 1.6), Color::new(0.55, 0.15, 0.18, 1.)); // carpet
        for &(x, z) in &CIVIC_DESK {
            r.cube(vec3(x as f32 + 0.5, 0.3, z as f32 + 0.5), vec3(0.5, 0.3, 0.42), Color::new(0.42, 0.28, 0.2, 1.));
            r.cube(vec3(x as f32 + 0.5, 0.62, z as f32 + 0.5), vec3(0.52, 0.03, 0.45), Color::new(0.3, 0.45, 0.35, 1.))
        }
        r.cube(vec3(2.6, 0.68, 1.4), vec3(0.12, 0.03, 0.09), WHITE);
        r.cube(vec3(4.4, 0.72, 1.45), vec3(0.04, 0.08, 0.04), GOLD); // papers, pen stand
        let talking = matches!(
            g.me.ui,
            Ui::Talk { npc: None, .. } | Ui::Civic(_) | Ui::Policy(_) | Ui::PolicyOk(_) | Ui::MPick(_)
        );
        resident(r, 13, vec3(3.5, 0., 0.5), 0., 0., false, t, talking, 0.);
        r.cube(vec3(3.5, 2.1, 0.06), vec3(0.5, 0.18, 0.03), Color::new(0.3, 0.45, 0.35, 1.));
        r.cube(vec3(3.5, 2.1, 0.09), vec3(0.42, 0.1, 0.01), GOLD); // crest
        for (k, &(x, z)) in CIVIC_PLANTS.iter().enumerate() {
            let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5);
            r.cyl(p, 0.22, 0.35, Color::new(0.7, 0.45, 0.3, 1.));
            r.sphere(p + vec3(0., 0.7, 0.), vec3(0.32, 0.4 + k as f32 * 0.05, 0.32), Color::new(0.25, 0.55, 0.3, 1.))
        }
        r.cube(vec3(1.0, 1.35, 0.05), vec3(0.45, 0.4, 0.03), Color::new(0.6, 0.45, 0.3, 1.));
        r.cube(vec3(1.0, 1.35, 0.08), vec3(0.4, 0.35, 0.01), Color::new(0.95, 0.92, 0.82, 1.)); // project board
        if let Some(m) = &g.w.mproject {
            let f = m.funded as f32 / MPROJECTS[m.idx].1 as f32;
            r.cube(vec3(0.65 + 0.35 * f, 1.2, 0.1), vec3(0.35 * f, 0.04, 0.01), Color::new(0.3, 0.7, 0.35, 1.))
        }
        for &(bx, bz) in &CIVIC_BENCH {
            let (x, z) = (bx as f32 + 0.5, bz as f32 + 0.5);
            r.cube(vec3(x, 0.22, z), vec3(0.4, 0.04, 0.3), WOOD);
            r.cube(vec3(x, 0.1, z), vec3(0.36, 0.1, 0.26), sh(WOOD, 0.8))
        } // waiting bench
        let ppos = g.me.pos;
        let _ = ppos;
        labels.push((
            vec3(1.0, 1.95, 0.1),
            match &g.w.mproject {
                Some(m) => format!("{}: {}/{}c", MPROJECTS[m.idx].0, m.funded, MPROJECTS[m.idx].1),
                None => "No funded project".into(),
            },
            Color::new(1., 0.95, 0.7, 1.),
        ));
        labels.push((
            vec3(3.5, 2.5, 0.1),
            format!("Policy: {}", g.pol().map_or("none", |p| p.name)),
            Color::new(0.8, 1., 0.85, 1.),
        ));
        {
            let k = tc(KIOSK);
            r.cube(vec3(k.x, 0.45, k.y), vec3(0.3, 0.45, 0.25), Color::new(0.3, 0.45, 0.6, 1.));
            let u = r.unlit;
            r.unlit = true;
            r.cube(vec3(k.x + 0.26, 0.85, k.y), vec3(0.01, 0.2, 0.17), Color::new(0.6, 0.9, 1., 1.));
            r.unlit = u;
            r.cube(vec3(k.x, 0.95, k.y), vec3(0.32, 0.04, 0.27), GOLD);
            if g.me.ipos.distance(k) < 2.5 {
                labels.push((vec3(k.x, 1.4, k.y), "Initiatives kiosk".into(), Color::new(0.8, 0.95, 1., 1.)))
            }
        }
    }
    if BLDS[b].kind == BK::Museum {
        let me = g.me.ipos;
        for (k, &d) in ARTS.iter().enumerate() {
            let p = vec3(GALLERY_X[k], 1.85, 0.05);
            if g.w.donated.iter().any(|x| x.0 == d) {
                painting(r, d, p, 0.62, 0.46);
                if me.distance(vec2(p.x, 0.8)) < 1.6 {
                    labels.push((vec3(p.x, 2.25, 0.1), def(d).name.into(), Color::new(1., 0.95, 0.75, 1.)))
                }
            } else {
                r.cube(p, vec3(0.33, 0.25, 0.01), Color::new(0.82, 0.8, 0.75, 1.))
            }
        }
        labels.push((
            vec3(3.5, 2.45, 0.1),
            format!("Art wing  {}/{}", g.donated_of(K::Art), ARTS.len()),
            Color::new(0.85, 0.9, 1., 1.),
        ));
        if me.distance(vec2(0.9, 2.5)) < 1.6 {
            labels.push((
                vec3(0.5, 1.4, 2.5),
                format!(
                    "Aquarium: {} fish & sea life",
                    g.w.donated.iter().filter(|x| matches!(def(x.0).k, K::Fish | K::Sea)).count()
                ),
                Color::new(0.75, 0.95, 1., 1.),
            ))
        }
        if me.distance(vec2(6.1, 2.5)) < 1.6 {
            labels.push((
                vec3(6.5, 1.3, 2.5),
                format!("Insect case: {} bugs", g.donated_of(K::Bug)),
                Color::new(0.9, 1., 0.8, 1.),
            ))
        }
        for &((x, z), fd) in &MUSEUM_PED {
            if me.distance(vec2(x as f32 + 0.5, z as f32 + 0.5)) < 1.5 {
                labels.push((
                    vec3(x as f32 + 0.5, 1.15, z as f32 + 0.5),
                    if g.w.donated.iter().any(|y| y.0 == fd) {
                        def(fd).name.to_string()
                    } else {
                        "(empty pedestal)".into()
                    },
                    Color::new(1., 0.92, 0.8, 1.),
                ))
            }
        }
        for &(x, z) in &MUSEUM_DESK {
            r.cube(vec3(x as f32 + 0.5, 0.45, z as f32 + 0.5), vec3(0.5, 0.45, 0.42), Color::new(0.4, 0.3, 0.25, 1.))
        }
        let talking = matches!(g.me.ui, Ui::Talk { npc: None, .. });
        resident(r, 11, vec3(3.5, 0., 0.5), 0., 0., false, t, talking, 0.);
        r.cube(vec3(0.5, 0.15, 2.5), vec3(0.48, 0.15, 1.48), Color::new(0.35, 0.3, 0.28, 1.)); // tank base
        for (k, d) in g.w.donated.iter().filter(|x| matches!(def(x.0).k, K::Fish | K::Sea)).enumerate() {
            let ph = t * 0.6 + k as f32 * 1.7;
            let s = def(d.0).size * 0.35;
            r.with(
                Mat4::from_rotation_translation(
                    Quat::from_rotation_y(if ph.cos() > 0. { 0. } else { PI }),
                    vec3(0.5, 0.55 + (k % 4) as f32 * 0.18, 2.5 + ph.sin() * 1.1),
                ),
                |r| {
                    r.sphere(Vec3::ZERO, vec3(s * 0.45, s * 0.5, s), rgb(def(d.0).col));
                    r.cone(vec3(0., 0., -s), s * 0.4, s * 0.6, rgb(def(d.0).col))
                },
            )
        }
        let ua = r.alpha;
        r.alpha = 0.35;
        r.cube(vec3(0.5, 0.75, 2.5), vec3(0.47, 0.45, 1.46), Color::new(0.45, 0.75, 0.95, 1.));
        r.alpha = ua;
        r.cube(vec3(6.5, 0.45, 2.5), vec3(0.45, 0.45, 1.45), Color::new(0.55, 0.38, 0.25, 1.));
        r.cube(vec3(6.5, 0.92, 2.5), vec3(0.43, 0.02, 1.43), Color::new(0.95, 0.93, 0.85, 1.));
        for (k, d) in g.w.donated.iter().filter(|x| def(x.0).k == K::Bug).enumerate() {
            let p = vec3(6.5 + ((k % 2) as f32 - 0.5) * 0.4, 0.95, 1.4 + (k / 2) as f32 * 0.5);
            r.sphere(p, vec3(0.09, 0.02, 0.06), rgb(def(d.0).col));
            r.sphere(p, Vec3::splat(0.025), DARK)
        }
        for &((x, z), fd) in &MUSEUM_PED {
            let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5);
            r.cyl(p, 0.36, 0.5, Color::new(0.9, 0.88, 0.84, 1.));
            if g.w.donated.iter().any(|x| x.0 == fd) {
                let c = rgb(def(fd).col);
                match fd {
                    48 => {
                        for k in 0..5 {
                            r.sphere(
                                p + vec3(0., 0.62 + k as f32 * 0.02, 0.),
                                Vec3::splat(0.2 - k as f32 * 0.035),
                                sh(c, 1. - k as f32 * 0.08),
                            )
                        }
                    }
                    49 => {
                        r.sphere(p + vec3(0., 0.58, 0.), vec3(0.16, 0.06, 0.24), c);
                        for k in 0..4 {
                            r.cube(p + vec3(0., 0.63, -0.15 + k as f32 * 0.1), vec3(0.15, 0.01, 0.015), sh(c, 0.7))
                        }
                    }
                    43 => {
                        r.cube(p + vec3(0., 0.7, 0.), vec3(0.22, 0.2, 0.04), Color::new(0.75, 0.72, 0.65, 1.));
                        for k in 0..5 {
                            r.cube(p + vec3(-0.1 + k as f32 * 0.05, 0.7, 0.045), vec3(0.01, 0.12, 0.005), c)
                        }
                    }
                    _ => r.cone(p + vec3(0., 0.5, 0.), 0.1, 0.3, c),
                }
            } else {
                r.cube(p + vec3(0., 0.53, 0.25), vec3(0.15, 0.03, 0.01), DARKGRAY)
            }
        }
    }
    if BLDS[b].kind == BK::Shop {
        for &(x, z) in &COUNTER {
            r.cube(vec3(x as f32 + 0.5, 0.45, z as f32 + 0.5), vec3(0.5, 0.45, 0.42), WOOD);
            r.cube(vec3(x as f32 + 0.5, 0.92, z as f32 + 0.5), vec3(0.52, 0.03, 0.45), sh(WOOD, 1.2))
        }
        r.cube(vec3(4.3, 1.05, 1.4), vec3(0.15, 0.1, 0.12), DARKGRAY);
        let rk = vec3(RACK.0 as f32 + 0.5, 0., RACK.1 as f32 + 0.5);
        for x in [-0.4, 0.4] {
            r.cube(rk + vec3(x, 0.7, 0.), vec3(0.03, 0.7, 0.03), DARKGRAY)
        }
        r.cube(rk + vec3(0., 1.38, 0.), vec3(0.42, 0.02, 0.02), DARKGRAY);
        for (k, &d) in CLOTHES.iter().enumerate() {
            r.cube(rk + vec3(-0.3 + k as f32 * 0.2, 1.05, 0.), vec3(0.08, 0.28, 0.12), rgb(def(d).col))
        }
        for k in 0..3 {
            r.cube(vec3(1.2 + k as f32 * 2.3, 1.4, 0.15), vec3(0.9, 0.04, 0.15), WOOD);
            for j in 0..4 {
                r.cube(
                    vec3(0.6 + k as f32 * 2.3 + j as f32 * 0.38, 1.56, 0.15),
                    vec3(0.12, 0.12, 0.1),
                    rgb(DEFS[(j * 7 + k * 3) % 30].col),
                )
            }
        }
        let talking = matches!(g.me.ui, Ui::Sell(_) | Ui::Confirm(_)) || matches!(g.me.ui, Ui::Talk { npc: None, .. });
        resident(r, 10, vec3(3.5, 0., 0.5), 0., 0., false, t, talking, 0.);
        for (di, tile) in [(7usize, EXTRA_DISPLAY), (8, SPECIAL_DISPLAY)] {
            if (di == 7 && g.w.stock.len() <= 5) || (di == 8 && g.shop_tier() < 1) {
                continue;
            }
            let p = vec3(tile.0 as f32 + 0.5, 0., tile.1 as f32 + 0.5);
            let (dd, price, sold) = g.display_item(di);
            r.cyl(
                p,
                0.4,
                0.22,
                if di == 8 { Color::new(0.85, 0.7, 0.35, 1.) } else { Color::new(0.85, 0.82, 0.75, 1.) },
            );
            if !sold {
                furniture(
                    r,
                    dd,
                    Mat4::from_translation(p + vec3(0., if def(dd).wall { -0.2 } else { 0.22 }, 0.))
                        * Mat4::from_scale(Vec3::splat(0.5)),
                    false,
                    0,
                )
            }
            if g.me.ipos.distance(vec2(p.x, p.z)) < 2.2 {
                labels.push((
                    p + vec3(0., 1.15, 0.),
                    if sold {
                        "SOLD OUT".into()
                    } else {
                        format!("{}{} {price}", if di == 8 { "Special: " } else { "" }, def(dd).name)
                    },
                    Color::new(1., 0.9, 0.5, 1.),
                ))
            }
        }
        for (i, &(x, z)) in DISPLAYS.iter().enumerate() {
            let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5);
            r.cyl(p, 0.4, 0.22, Color::new(0.85, 0.82, 0.75, 1.));
            let (txt, show) = if i < 5 {
                let s = g.w.stock[i];
                (
                    if s.sold { "SOLD OUT".to_string() } else { format!("{} {}", def(s.def).name, def(s.def).buy) },
                    (!s.sold).then_some(s.def),
                )
            } else {
                let tl = TOOL_DISPLAY[i - 5];
                (
                    if g.w.p.tools.contains(&tl) {
                        "Owned".into()
                    } else {
                        format!("{} {}", def(tl).name, def(tl).buy)
                    },
                    Some(tl),
                )
            };
            match show {
                Some(dd) if def(dd).k == K::Tool => r.with(
                    Mat4::from_translation(p + vec3(0., 0.3, 0.))
                        * Mat4::from_rotation_z(0.4)
                        * Mat4::from_scale(Vec3::splat(1.4)),
                    |r| draw_tool(r, dd),
                ),
                Some(dd) => {
                    let wall_it = def(dd).wall;
                    furniture(
                        r,
                        dd,
                        Mat4::from_translation(p + vec3(0., if wall_it { -0.2 } else { 0.22 }, 0.))
                            * Mat4::from_scale(Vec3::splat(0.5)),
                        false,
                        0,
                    )
                }
                None => r.cube(p + vec3(0., 0.4, 0.), vec3(0.25, 0.15, 0.02), Color::new(0.8, 0.2, 0.2, 1.)),
            }
            if g.me.ipos.distance(vec2(p.x, p.z)) < 2.2 {
                labels.push((
                    p + vec3(0., 1.15, 0.),
                    txt,
                    if i < 5 && g.w.stock[i].sold {
                        Color::new(1., 0.5, 0.5, 1.)
                    } else {
                        Color::new(1., 0.95, 0.6, 1.)
                    },
                ))
            }
        }
    }
    let owner = if let BK::Home(o) = BLDS[b].kind { Some(g.owner(o)) } else { None };
    for f in &furn {
        furniture(r, f.def, fbase(f), f.on, owner.map_or(0, |pr| Game::tint_of(pr, f.uid)));
        if def(f.def).act == Act::Lamp && f.on {
            let c = fcenter(f);
            r.disc(vec3(c.x, 0.012, c.y), vec2(0.9, 0.9), Color::new(1., 0.85, 0.4, 0.18))
        }
    }
    if let Some((vi, vb, _)) = g.visit {
        if vb == b {
            let s = g.visit_spot(b);
            let talking = matches!(g.me.ui, Ui::Talk { npc: Some(j), .. } if j == vi);
            r.disc(vec3(s.x, 0.02, s.y), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.2));
            resident(
                r,
                g.w.roster[vi],
                vec3(s.x, 0., s.y),
                yaw_of(g.me.ipos - s),
                0.,
                false,
                t,
                talking,
                g.npcs[vi].happy,
            )
        }
    }
    if let BK::Res(i) = BLDS[b].kind {
        if g.loc(i) == Loc::Home {
            let s = res_spot();
            let talking = matches!(g.me.ui, Ui::Talk { npc: Some(j), .. } if j == i);
            r.disc(vec3(s.x, 0.02, s.y), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.2));
            resident(
                r,
                g.w.roster[i],
                vec3(s.x, 0., s.y),
                yaw_of(g.me.ipos - s),
                0.,
                false,
                t,
                talking,
                g.npcs[i].happy,
            )
        }
    }
    if let Some(p) = &g.me.placing {
        let ok = g.place_ok(&p.f).is_ok();
        let c = if ok { Color::new(0.2, 0.95, 0.3, 0.7) } else { Color::new(0.95, 0.2, 0.2, 0.6) };
        if def(p.f.def).wall {
            r.disc(vec3(p.f.x as f32 + 0.5, 0.02, 0.3), vec2(0.45, 0.25), c)
        } else {
            for (x, z) in foot(p.f.def, p.f.x, p.f.z, p.f.rot) {
                r.disc(vec3(x as f32 + 0.5, 0.02, z as f32 + 0.5), vec2(0.48, 0.48), c)
            }
        }
        furniture(r, p.f.def, fbase(&p.f), false, Game::tint_of(&g.w.p, p.f.uid))
    }
    let pose = match g.me.pose {
        Pose::Stand => 0,
        Pose::Sit(_) => 1,
        Pose::Lie(_) => 2,
    };
    if pose == 0 {
        r.disc(vec3(g.me.ipos.x, 0.02, g.me.ipos.y), vec2(0.32, 0.26), Color::new(0., 0., 0., 0.2))
    }
    human(
        r,
        g.me.id,
        (g.w.p.shirt, g.w.p.hat, g.w.p.hair),
        vec3(g.me.ipos.x, 0., g.me.ipos.y),
        yaw_of(g.me.face),
        g.me.anim,
        g.me.spd,
        pose,
        None,
        t,
        0.,
    );
    others(r, g, t);
    if g.me.ui == Ui::Play && g.me.placing.is_none() && g.me.trans.is_none() {
        if let Some(tg) = g.target() {
            indicator(r, g.tg_pos(tg), t)
        }
    }
}
