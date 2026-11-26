//! Scenes and cameras: the islet, trees, the per-player viewport (draw/view), the town and other players.
use super::*;

/// Gullrock Islet: own tropical terrain, palms, tide pool, dock, Captain Pell, Sea Glass stall, collectibles, both players.
pub(super) fn islet_scene(r: &mut R, g: &Game, lt: &Light, labels: &mut Vec<(Vec3, String, Color)>) {
    let t = g.t;
    let (sand, grass) = (Color::new(0.97, 0.9, 0.7, 1.), Color::new(0.42, 0.78, 0.38, 1.));
    let m = &g.imap;
    let water = |c: u8| matches!(c, SEA | RIVER | BRIDGE);
    for z in -10..IH as i32 + 8 {
        for x in -12..IW as i32 + 12 {
            let (fx, fz) = (x as f32, z as f32);
            if (fx - g.me.cam.x).powi(2) + (fz - g.me.cam.y).powi(2) > 480. {
                continue;
            }
            let c = tile_at_i(m, x, z);
            if water(c) {
                let sv = (t * 1.4 + (x + z) as f32 * 0.6).sin() * 0.03;
                let wc = if c == RIVER {
                    Color::new(0.25 + sv, 0.7 + sv, 0.72 + sv, 1.)
                } else {
                    Color::new(0.2 + sv, 0.62 + sv, 0.82 + sv, 1.)
                };
                r.quad(
                    vec3(fx, -0.2, fz),
                    vec3(fx + 1., -0.2, fz),
                    vec3(fx + 1., -0.2, fz + 1.),
                    vec3(fx, -0.2, fz + 1.),
                    Vec3::Y,
                    wc,
                );
                for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
                    if water(tile_at_i(m, x + dx, z + dz)) {
                        continue;
                    }
                    let (a, b, n) = match (dx, dz) {
                        (0, -1) => (vec3(fx, 0., fz), vec3(fx + 1., 0., fz), Vec3::Z),
                        (0, 1) => (vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), -Vec3::Z),
                        (-1, 0) => (vec3(fx, 0., fz + 1.), vec3(fx, 0., fz), Vec3::X),
                        _ => (vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), -Vec3::X),
                    };
                    r.quad(a, b, b - Vec3::Y * 0.2, a - Vec3::Y * 0.2, n, sand)
                }
            } else {
                r.quad(
                    vec3(fx, 0., fz),
                    vec3(fx + 1., 0., fz),
                    vec3(fx + 1., 0., fz + 1.),
                    vec3(fx, 0., fz + 1.),
                    Vec3::Y,
                    sh(
                        if c == GRASS || c == TREE || c == ROCK { grass } else { sand },
                        1. + ((hash(x, z) % 5) as f32 - 2.) * 0.015,
                    ),
                )
            }
            let (px, pz) = (fx + 0.5, fz + 0.5);
            match c {
                TREE => palm(r, px, pz, t),
                ROCK => r.sphere(vec3(px, 0.12, pz), vec3(0.4, 0.28, 0.34), Color::new(0.55, 0.5, 0.48, 1.)),
                BRIDGE => {
                    r.cube(vec3(px, 0.05, pz), vec3(0.5, 0.05, 0.5), WOOD);
                    r.cube(vec3(px - 0.45, -0.05, pz), vec3(0.05, 0.25, 0.05), sh(WOOD, 0.7))
                }
                _ => {}
            }
        }
    }
    let shc = Color::new(0., 0., 0., 0.22);
    r.disc(vec3(PELL_I.0, 0.02, PELL_I.1), vec2(0.3, 0.24), shc);
    let talking = matches!(g.me.ui, Ui::Talk { npc: None, .. });
    resident(
        r,
        12,
        vec3(PELL_I.0, 0., PELL_I.1),
        yaw_of(g.me.ipos - vec2(PELL_I.0, PELL_I.1)),
        0.,
        false,
        t,
        talking,
        0.,
    );
    r.cube(vec3(7.5 + 1.2, -0.05, 13.3), vec3(0.35, 0.12, 0.8), Color::new(0.85, 0.85, 0.9, 1.));
    let st = tc(ISTALL);
    r.cube(vec3(st.x, 0.4, st.y), vec3(0.45, 0.4, 0.35), Color::new(0.6, 0.48, 0.35, 1.));
    for k in 0..4 {
        let a = k as f32 * 1.6;
        r.sphere(
            vec3(st.x - 0.25 + k as f32 * 0.17, 0.92 + (t * 2. + a).sin() * 0.02, st.y),
            Vec3::splat(0.08),
            Color::new(0.45, 0.85 - k as f32 * 0.08, 0.78, 0.9),
        )
    }
    r.cube(vec3(st.x, 1.25, st.y + 0.25), vec3(0.5, 0.03, 0.35), Color::new(0.3, 0.7, 0.75, 1.));
    for it in &g.w.iground {
        let c = rgb(def(it.def).col);
        match it.def {
            82 => r.with(Mat4::from_translation(vec3(it.x, 0.06, it.z)) * Mat4::from_rotation_y(it.x * 3.), |r| {
                r.cube(Vec3::ZERO, vec3(0.28, 0.05, 0.05), c)
            }),
            75 => {
                r.sphere(vec3(it.x, 0.08, it.z), vec3(0.14, 0.07, 0.11), c);
                for s in [-1f32, 1.] {
                    r.sphere(vec3(it.x + s * 0.15, 0.08, it.z + 0.08), Vec3::splat(0.05), c)
                }
            }
            _ => {
                r.sphere(vec3(it.x, 0.05, it.z), vec3(0.12, 0.06, 0.12), c);
                r.cone(vec3(it.x, 0.05, it.z), 0.07, 0.12, sh(c, 0.9))
            }
        }
    }
    for b in &g.ibubbles {
        for k in 0..3 {
            let ph = (t * 1.5 + k as f32 * 2.1) % 1.;
            r.sphere(
                vec3(b.0.x + (k as f32 - 1.) * 0.12, -0.18 + ph * 0.05, b.0.y + ph * 0.1),
                Vec3::splat(0.04 + ph * 0.03),
                Color::new(0.95, 0.98, 1., 1.),
            )
        }
    }
    for &(p, a, k) in &g.fx {
        if k == 0 {
            let sz = a / 0.8;
            for i in 0..10 {
                let an = i as f32 / 10. * 2. * PI;
                let rr = 0.15 + sz * 0.6;
                r.sphere(
                    p + vec3(an.cos() * rr, 0.02, an.sin() * rr),
                    vec3(0.05, 0.015, 0.05),
                    Color::new(0.92, 0.97, 1., 1. - sz),
                )
            }
        }
    }
    {
        let bp = tc(ACT_BOARD);
        r.cube(vec3(bp.x, 0.45, bp.y), vec3(0.05, 0.45, 0.05), WOOD);
        r.cube(vec3(bp.x, 0.95, bp.y), vec3(0.4, 0.25, 0.04), Color::new(0.3, 0.6, 0.75, 1.));
        if g.me.ipos.distance(bp) < 3. {
            labels.push((vec3(bp.x, 1.5, bp.y), "Tidekeeper's board".into(), Color::new(0.7, 1., 0.95, 1.)))
        }
    }
    if let Some(a) = &g.act {
        match a.kind {
            0 => {
                for k in 0..12 {
                    let an = k as f32 / 12. * 2. * PI;
                    r.sphere(
                        vec3(PEN.0 + an.cos() * PEN.2, 0.08, PEN.1 + an.sin() * PEN.2),
                        vec3(0.14, 0.1, 0.12),
                        Color::new(0.6, 0.58, 0.55, 1.),
                    )
                }
                for (k, c) in a.ents.iter().enumerate() {
                    let pen = Game::in_pen(*c);
                    r.sphere(vec3(c.x, 0.12, c.y), vec3(0.22, 0.11, 0.17), Color::new(0.2, 0.45, 0.9, 1.));
                    for s in [-1f32, 1.] {
                        r.sphere(
                            vec3(c.x + s * 0.24, 0.14, c.y - 0.1),
                            Vec3::splat(0.075),
                            Color::new(0.35, 0.6, 1., 1.),
                        )
                    }
                    if !pen {
                        r.cone(
                            vec3(c.x, 0.75 + (t * 4. + k as f32).sin() * 0.06, c.y),
                            0.09,
                            -0.18,
                            Color::new(1., 0.85, 0.2, 1.),
                        )
                    }
                }
            }
            1 => {
                let b = a.ents[0];
                let pulse = (t * 6.).sin() * 0.5 + 0.5;
                r.cyl(vec3(b.x, 0., b.y), 0.12, 0.9, Color::new(1., 0.85, 0.3, 1.));
                r.sphere(
                    vec3(b.x, 1.0, b.y),
                    Vec3::splat(0.16 + pulse * 0.06),
                    Color::new(1., 0.95, 0.5 + pulse * 0.4, 1.),
                )
            }
            _ => {
                for (k, p) in a.ents.iter().enumerate() {
                    if k < a.next {
                        continue;
                    }
                    let c = if k == a.next { Color::new(1., 0.8, 0.2, 1.) } else { Color::new(0.95, 0.95, 0.95, 1.) };
                    for j in 0..14 {
                        let an = j as f32 / 14. * 2. * PI;
                        r.sphere(vec3(p.x + an.cos() * 0.55, 0.15 + an.sin() * 0.55, p.y), Vec3::splat(0.07), c)
                    }
                    if k == a.next || g.me.ipos.distance(*p) < 6. {
                        labels.push((vec3(p.x, 1.1, p.y), format!("{}", k + 1), c))
                    }
                }
            }
        }
    }
    for b in &g.ibugs {
        let c = rgb(def(b.0).col);
        let fl = (t * 20. + b.2).sin().abs();
        let y = 0.35 + (t * 3. + b.2).sin() * 0.1;
        for s in [-1f32, 1.] {
            r.sphere(vec3(b.1.x + s * 0.07, y + fl * 0.04, b.1.y), vec3(0.07, 0.01 + fl * 0.03, 0.05), c)
        }
        r.sphere(vec3(b.1.x, y, b.1.y), Vec3::splat(0.035), DARK)
    }
    let swim = g.swimming();
    let sy = if swim { -0.55 } else { 0. };
    if !swim {
        r.disc(vec3(g.me.ipos.x, 0.02, g.me.ipos.y), vec2(0.32, 0.26), shc)
    }
    human(
        r,
        g.me.id,
        (g.w.p.shirt, g.w.p.hat, g.w.p.hair),
        vec3(g.me.ipos.x, sy, g.me.ipos.y),
        yaw_of(g.me.face),
        g.me.anim,
        g.me.spd,
        0,
        None,
        t,
        g.me.swing,
    );
    others(r, g, t);
    if g.me.pos.distance(Vec2::ZERO) >= 0. && g.me.ipos.distance(st) < 3.5 {
        labels.push((vec3(st.x, 1.7, st.y), "Sea Glass stall".into(), Color::new(0.7, 1., 0.95, 1.)))
    }
    if g.me.ui == Ui::Play && g.me.trans.is_none() {
        if let Some(tg) = g.target() {
            indicator(r, g.tg_pos(tg), t)
        }
    }
    let _ = lt;
}
pub(super) fn tree(r: &mut R, p: Vec2, fruit: Option<Color>, shake: f32, t: f32, fol: Color) {
    // per-tree variety from its position (shape only; the tree's tile is unchanged)
    let hv = hash((p.x * 2.) as i32, (p.y * 2.) as i32);
    let v = |k: u32| ((hv >> k) % 100) as f32 / 100.;
    let bark = Color::new(0.5, 0.33, 0.2, 1.);
    r.cyl(vec3(p.x, 0., p.y), 0.17, 0.12, sh(bark, 0.9));
    r.cyl(vec3(p.x, 0., p.y), 0.13, 0.85, bark);
    r.seg(vec3(p.x, 0.7, p.y), vec3(p.x + 0.28 * (v(3) - 0.5).signum(), 1.0, p.y + 0.08), 0.04, bark);
    let sway = (t * 1.1 + p.x * 0.7 + p.y * 0.3).sin() * 0.025;
    let o = vec3(p.x + (t * 45.).sin() * 0.08 * shake + sway, 0., p.y);
    let fol = Color::new(fol.r * (0.95 + v(5) * 0.1), fol.g * (0.95 + v(7) * 0.1), fol.b, 1.);
    r.sphere(o + vec3(0., 1.08, 0.05), vec3(0.6, 0.38, 0.6), sh(fol, 0.82)); // shaded underside
    r.sphere(o + vec3(0., 1.2, 0.), vec3(0.65, 0.55, 0.65), fol);
    r.sphere(o + vec3(0.3, 1.5, 0.1), Vec3::splat(0.38), sh(fol, 1.08));
    r.sphere(o + vec3(-0.28, 1.42, -0.12), Vec3::splat(0.4), sh(fol, 0.94));
    r.sphere(o + vec3(0.05 + (v(9) - 0.5) * 0.2, 1.68, -0.05), Vec3::splat(0.3), sh(fol, 1.14));
    r.sphere(o + vec3(-0.15, 1.25, 0.42), Vec3::splat(0.3), sh(fol, 1.03));
    if let Some(fc) = fruit {
        for q in [vec3(0.32, 1.05, 0.48), vec3(-0.38, 1.12, 0.42), vec3(0.05, 1.38, 0.55), vec3(0.52, 1.28, 0.2)] {
            r.sphere(o + q, Vec3::splat(0.09), fc)
        }
    }
}
pub(super) fn pine(r: &mut R, x: f32, z: f32, s: f32) {
    let hv = hash((x * 3.) as i32, (z * 3.) as i32);
    let tone = ((hv % 100) as f32 / 100. - 0.5) * 0.06;
    let tw = ((hv >> 7) % 628) as f32 / 100.;
    r.cyl(vec3(x, 0., z), 0.1 * s, 0.5 * s, Color::new(0.45, 0.3, 0.2, 1.));
    for k in 0..3 {
        let kf = k as f32;
        let y = (0.4 + kf * 0.38) * s;
        let lean = (tw + kf).sin() * 0.02 * kf;
        r.cone(
            vec3(x + lean, y - 0.04 * s, z),
            (0.66 - kf * 0.15) * s,
            0.12 * s,
            Color::new(0.11, 0.3 + tone, 0.2, 1.),
        ); // dark skirt under each tier
        r.cone(
            vec3(x + lean, y, z),
            (0.62 - kf * 0.15) * s,
            0.65 * s,
            Color::new(0.16 + tone * 0.5, 0.42 + kf * 0.05 + tone, 0.26, 1.),
        )
    }
}
pub(super) fn indicator(r: &mut R, p: Vec3, t: f32) {
    let u = r.unlit;
    r.unlit = true;
    let y = p.y + (t * 4.).sin() * 0.06;
    r.with(Mat4::from_translation(vec3(p.x, y, p.z)) * Mat4::from_rotation_x(PI), |r| {
        r.cone(Vec3::ZERO, 0.1, 0.18, Color::new(1., 0.85, 0.2, 1.))
    });
    r.unlit = u
}

// ---- scenes ----
thread_local! { static VIEW: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((1280., 720.)) }; }
pub(super) fn vw() -> f32 {
    VIEW.with(|v| v.get().0)
}
pub(super) fn vh() -> f32 {
    VIEW.with(|v| v.get().1)
}
/// Draw every player's view: full screen for one player, side-by-side split-screen for two.
pub fn draw(g: &mut Game, now: NaiveDateTime, dbg: Option<&str>) {
    let ids: Vec<usize> = g.ids().into_iter().filter(|&i| Some(i) != g.remote).collect();
    let act = g.me.id;
    let n = ids.len() as f32;
    let (sw, sh) = (screen_width(), screen_height());
    if n > 1. {
        clear_background(BLACK)
    }
    for (k, &id) in ids.iter().enumerate() {
        g.activate(id);
        VIEW.with(|v| v.set((sw / n, sh)));
        view(g, now, dbg, (n > 1.).then(|| (k as f32 * sw / n, sw / n)))
    }
    g.activate(act);
    VIEW.with(|v| v.set((sw, sh)));
    if n > 1. {
        set_default_camera();
        draw_rectangle(sw / 2. - 2., 0., 4., sh, BLACK)
    }
}
pub(super) fn hud_cam(vp: Option<(f32, f32)>) {
    match vp {
        Some((x, w)) => set_camera(&Camera2D {
            target: vec2(w / 2., vh() / 2.),
            zoom: vec2(2. / w, 2. / vh()),
            viewport: Some((x as i32, 0, w as i32, vh() as i32)),
            ..Default::default()
        }),
        None => set_default_camera(),
    }
}
pub(super) fn view(g: &Game, now: NaiveDateTime, dbg: Option<&str>, vp: Option<(f32, f32)>) {
    let hf = now.hour() as f32 + now.minute() as f32 / 60.;
    let wx = weather(g.w.day, now.month());
    let rain = wx != Wx::Clear;
    let lt = light(hf, rain);
    let (cam, curve, c0) = match g.me.scene {
        Scene::Town | Scene::Islet => (
            Camera3D {
                position: vec3(g.me.cam.x, 7.4, g.me.cam.y + 7.6),
                target: vec3(g.me.cam.x, 0.5, g.me.cam.y - 0.8),
                up: Vec3::Y,
                fovy: 40f32.to_radians(),
                ..Default::default()
            },
            g.curve,
            g.me.cam,
        ),
        Scene::Room(b) => {
            let (w, d) = g.rdims(b);
            let s = w.max(d) as f32 / 4.;
            (
                Camera3D {
                    position: vec3(w as f32 / 2., 5.4 * s, d as f32 + 3.4 * s),
                    target: vec3(w as f32 / 2., 0.3, d as f32 / 2. + 0.1),
                    up: Vec3::Y,
                    fovy: 45f32.to_radians(),
                    ..Default::default()
                },
                0.,
                Vec2::ZERO,
            )
        }
    };
    let bg = if g.me.scene != Scene::Room(0) && !matches!(g.me.scene, Scene::Room(_)) {
        lt.sky
    } else {
        Color::new(0.08, 0.06, 0.06, 1.)
    };
    let cam =
        Camera3D { viewport: vp.map(|(x, w)| (x as i32, 0, w as i32, vh() as i32)), aspect: Some(vw() / vh()), ..cam };
    if vp.is_some() {
        hud_cam(vp);
        draw_rectangle(0., 0., vw(), vh(), bg)
    } else {
        clear_background(bg)
    }
    {
        hud_cam(vp);
        let indoor = matches!(g.me.scene, Scene::Room(_));
        let (w, h) = (vw(), vh());
        // outdoors: deeper zenith fading to the horizon (fog) colour; indoors: warm dim glow behind the room instead of flat black
        let top = if indoor {
            Color::new(0.05, 0.04, 0.05, 1.)
        } else {
            Color::new(bg.r * 0.62, bg.g * 0.72, (bg.b * 0.9 + 0.08).min(1.), 1.)
        };
        let bot = if indoor { Color::new(0.2, 0.15, 0.12, 1.) } else { bg };
        let n = 24;
        for k in 0..n {
            let (a, b) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
            let f = |t: f32| if indoor { t * t } else { (t / 0.55).min(1.).sqrt() };
            let t = f((a + b) / 2.);
            draw_rectangle(
                0.,
                a * h,
                w,
                (b - a) * h + 1.,
                Color::new(top.r + (bot.r - top.r) * t, top.g + (bot.g - top.g) * t, top.b + (bot.b - top.b) * t, 1.),
            )
        }
    }
    set_camera(&cam);
    let mut r = R::new(curve, c0, &lt, if matches!(g.me.scene, Scene::Room(_)) { (100., 200.) } else { (12., 21.) });
    let mut labels: Vec<(Vec3, String, Color)> = vec![];
    match g.me.scene {
        Scene::Islet => islet_scene(&mut r, g, &lt, &mut labels),
        Scene::Town => town(&mut r, g, &lt, now, &mut labels),
        Scene::Room(b) => room(&mut r, g, b, &lt, &mut labels),
    }
    r.flush();
    hud_cam(vp);
    let m = cam.matrix();
    let menu = matches!(g.me.ui, Ui::Journal | Ui::Help | Ui::Inv(..) | Ui::Mail(_) | Ui::Catalog(_));
    for (p, s, c) in labels {
        if menu {
            break;
        }
        let d2 = (p.x - c0.x).powi(2) + (p.z - c0.y).powi(2);
        let q = m * vec4(p.x, p.y - curve * d2, p.z, 1.);
        if q.w <= 0. {
            continue;
        }
        let (x, y) = ((q.x / q.w + 1.) * 0.5 * vw(), (1. - q.y / q.w) * 0.5 * vh());
        let tw = measure_text(&s, None, 20, 1.).width;
        draw_rectangle(x - tw / 2. - 5., y - 17., tw + 10., 23., Color::new(0., 0., 0., 0.55));
        draw_text(&s, x - tw / 2., y, 20., c);
    }
    if !matches!(g.me.scene, Scene::Room(_)) {
        let (mo, w, h, t) = (now.month(), vw(), vh(), g.t);
        match mo {
            3..=5 => {
                for i in 0..40 {
                    let x = ((i * 131) as f32 + (t * 0.6 + i as f32).sin() * 40. + t * 25.) % w;
                    let y = ((i * 71) as f32 + t * 35.) % h;
                    draw_circle(x, y, 3., Color::new(1., 0.75, 0.85, 0.85));
                    draw_circle(x + 2., y + 1., 2., Color::new(1., 0.85, 0.9, 0.85))
                }
            }
            9..=11 => {
                for i in 0..30 {
                    let x = ((i * 157) as f32 + (t * 0.9 + i as f32 * 1.3).sin() * 60. + t * 18.) % w;
                    let y = ((i * 89) as f32 + t * 42.) % h;
                    let c = [
                        Color::new(0.9, 0.45, 0.15, 0.9),
                        Color::new(0.8, 0.25, 0.15, 0.9),
                        Color::new(0.95, 0.7, 0.2, 0.9),
                    ][i % 3];
                    draw_rectangle_ex(
                        x,
                        y,
                        9.,
                        5.,
                        DrawRectangleParams { rotation: t * 2. + i as f32, offset: vec2(0.5, 0.5), color: c },
                    )
                }
            }
            6..=8 if dark(now) > 0.5 => {
                for i in 0..35 {
                    let x = ((i * 97) as f32 + (t * 0.5 + i as f32).sin() * 30.) % w;
                    let y = h * 0.35 + ((i * 53) as f32 + (t * 0.4 + i as f32).cos() * 25.) % (h * 0.6);
                    let a = ((t * 3. + i as f32).sin() * 0.5 + 0.5) * 0.9;
                    draw_circle(x, y, 2.5, Color::new(0.9, 1., 0.4, a));
                    draw_circle(x, y, 6., Color::new(0.9, 1., 0.4, a * 0.2))
                }
            }
            _ => {}
        }
    }
    if wx == Wx::Snow && g.me.scene == Scene::Town {
        for i in 0..160 {
            let x = ((i * 97) as f32 + (g.t * 0.7 + i as f32).sin() * 25. + g.t * 15.) % vw();
            let y = ((i * 53) as f32 + g.t * 70.) % vh();
            draw_circle(x, y, 2. + (i % 3) as f32, Color::new(1., 1., 1., 0.85))
        }
    } else if rain && g.me.scene == Scene::Town {
        for i in 0..140 {
            let x = ((i * 97) as f32 + g.t * 40.) % vw();
            let y = ((i * 53) as f32 + g.t * 520.) % vh();
            draw_line(x, y, x - 3., y + 12., 1., Color::new(0.8, 0.85, 1., 0.45))
        }
    }
    hud(g, now, dbg);
}
pub(super) fn town(r: &mut R, g: &Game, lt: &Light, now: NaiveDateTime, labels: &mut Vec<(Vec3, String, Color)>) {
    let t = g.t;
    let (grass, fol) = season(now.month());
    let (cx, cz) = (g.me.cam.x, g.me.cam.y);
    let rad = 22i32;
    let near = |x: f32, z: f32| (x - cx).powi(2) + (z - cz).powi(2) < (rad * rad) as f32;
    let water = |c: u8| matches!(c, RIVER | SEA | BRIDGE);
    let ext = |x: i32, z: i32| {
        if x >= 0 && z >= 0 && x < W as i32 && z < H as i32 {
            g.map[z as usize * W + x as usize]
        } else if z >= 26 {
            SEA
        } else if z >= 24 {
            SAND
        } else {
            b'f'
        }
    };
    let sand = Color::new(0.93, 0.86, 0.64, 1.);
    let soil = Color::new(0.55, 0.42, 0.3, 1.);
    let (z0, z1, x0, x1) = (cz as i32 - rad, cz as i32 + 9, cx as i32 - rad, cx as i32 + rad);
    // terrain
    for z in z0..z1 {
        for x in x0..x1 {
            let (fx, fz) = (x as f32, z as f32);
            if !near(fx + 0.5, fz + 0.5) {
                continue;
            }
            let c = ext(x, z);
            if water(c) {
                // smooth rolling ripples per corner; the sea deepens away from the beach, shallows near banks turn turquoise
                let wc = |dx: i32, dz: i32| {
                    let (xc, zc) = ((x + dx) as f32, (z + dz) as f32);
                    let s = ((t * 1.3 + xc * 0.9 + zc * 0.6).sin() * 0.5
                        + (t * 0.8 - xc * 0.45 + zc * 1.1).sin() * 0.5)
                        * 0.04;
                    let shore = [(-1, -1), (0, -1), (-1, 0), (0, 0)]
                        .iter()
                        .any(|&(ox, oz)| !water(ext(x + dx + ox, z + dz + oz))) as i32
                        as f32;
                    let (mut cr, mut cg, mut cb) = if c == SEA {
                        let d = ((zc - 26.) / 7.).clamp(0., 1.);
                        (0.24 - 0.1 * d, 0.56 - 0.14 * d, 0.8 - 0.06 * d)
                    } else {
                        (0.22, 0.52, 0.8)
                    };
                    cr += shore * 0.07;
                    cg += shore * 0.09;
                    cb += shore * 0.03;
                    Color::new(cr + s, cg + s, cb + s, 1.)
                };
                r.quad4(
                    [
                        vec3(fx, -0.2, fz),
                        vec3(fx + 1., -0.2, fz),
                        vec3(fx + 1., -0.2, fz + 1.),
                        vec3(fx, -0.2, fz + 1.),
                    ],
                    Vec3::Y,
                    [wc(0, 0), wc(1, 0), wc(1, 1), wc(0, 1)],
                );
                {
                    let u = r.unlit;
                    r.unlit = true;
                    let day = 1. - lt.night;
                    // sun glints: brief bright flecks that drift with the current
                    if day > 0.2 {
                        for k in 0..2u32 {
                            let hv = hash(x * 7 + k as i32, z * 13 - k as i32);
                            let ph = (hv % 628) as f32 / 100.;
                            let v = (t * (1.4 + k as f32 * 0.5) + ph).sin();
                            if v > 0.8 {
                                let a = (v - 0.8) / 0.2 * 0.75 * day;
                                let gx = fx + 0.15 + ((hv >> 3) % 70) as f32 / 100.;
                                let gz = fz + 0.15 + ((hv >> 9) % 70) as f32 / 100. + (t * 0.15).fract() * 0.1;
                                r.quad(
                                    vec3(gx - 0.07, -0.19, gz - 0.015),
                                    vec3(gx + 0.07, -0.19, gz - 0.015),
                                    vec3(gx + 0.07, -0.19, gz + 0.015),
                                    vec3(gx - 0.07, -0.19, gz + 0.015),
                                    Vec3::Y,
                                    Color::new(1., 1., 0.95, a),
                                )
                            }
                        }
                    }
                    // lapping foam along every bank edge
                    let fa = 0.32 + (t * 1.7 + (x * 3 + z * 5) as f32 * 0.4).sin() * 0.12;
                    let fw = 0.07 + (t * 1.1 + (x + z) as f32).sin().abs() * 0.05;
                    let fcol = Color::new(1., 1., 1., fa);
                    for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
                        if water(ext(x + dx, z + dz)) || ext(x + dx, z + dz) == BRIDGE {
                            continue;
                        }
                        let y = -0.195;
                        match (dx, dz) {
                            (0, -1) => r.quad(
                                vec3(fx, y, fz),
                                vec3(fx + 1., y, fz),
                                vec3(fx + 1., y, fz + fw),
                                vec3(fx, y, fz + fw),
                                Vec3::Y,
                                fcol,
                            ),
                            (0, 1) => r.quad(
                                vec3(fx, y, fz + 1. - fw),
                                vec3(fx + 1., y, fz + 1. - fw),
                                vec3(fx + 1., y, fz + 1.),
                                vec3(fx, y, fz + 1.),
                                Vec3::Y,
                                fcol,
                            ),
                            (-1, 0) => r.quad(
                                vec3(fx, y, fz),
                                vec3(fx + fw, y, fz),
                                vec3(fx + fw, y, fz + 1.),
                                vec3(fx, y, fz + 1.),
                                Vec3::Y,
                                fcol,
                            ),
                            _ => r.quad(
                                vec3(fx + 1. - fw, y, fz),
                                vec3(fx + 1., y, fz),
                                vec3(fx + 1., y, fz + 1.),
                                vec3(fx + 1. - fw, y, fz + 1.),
                                Vec3::Y,
                                fcol,
                            ),
                        }
                    }
                    r.unlit = u
                }
                for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
                    let nc = ext(x + dx, z + dz);
                    if water(nc) {
                        continue;
                    }
                    let bc = if nc == SAND { sand } else { soil };
                    let (a, b, n) = match (dx, dz) {
                        (0, -1) => (vec3(fx, 0., fz), vec3(fx + 1., 0., fz), Vec3::Z),
                        (0, 1) => (vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.), -Vec3::Z),
                        (-1, 0) => (vec3(fx, 0., fz + 1.), vec3(fx, 0., fz), Vec3::X),
                        _ => (vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), -Vec3::X),
                    };
                    r.quad(a, b, b - Vec3::Y * 0.2, a - Vec3::Y * 0.2, n, bc)
                }
            } else {
                let base = |c: u8| match c {
                    PATH => Color::new(0.77, 0.65, 0.47, 1.),
                    SAND => sand,
                    b'f' => sh(grass, 0.8),
                    RIVER | SEA | BRIDGE => sand,
                    _ => grass,
                };
                let col = base(c);
                let amt = if c == PATH || c == SAND { 0.45 } else { 1. };
                // soft borders: each corner leans toward the average of the four tiles that share it
                let cc = |dx: i32, dz: i32| {
                    let n = 0.5 + (gnoise(x + dx, z + dz) - 0.5) * amt;
                    let (cx, cz) = (x + dx, z + dz);
                    let mut a = [0f32; 3];
                    for (ox, oz) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
                        let b = if water(ext(cx + ox, cz + oz)) { col } else { base(ext(cx + ox, cz + oz)) };
                        a[0] += b.r / 4.;
                        a[1] += b.g / 4.;
                        a[2] += b.b / 4.
                    }
                    gtint(
                        Color::new(
                            col.r * 0.55 + a[0] * 0.45,
                            col.g * 0.55 + a[1] * 0.45,
                            col.b * 0.55 + a[2] * 0.45,
                            1.,
                        ),
                        n,
                    )
                };
                r.quad4(
                    [vec3(fx, 0., fz), vec3(fx + 1., 0., fz), vec3(fx + 1., 0., fz + 1.), vec3(fx, 0., fz + 1.)],
                    Vec3::Y,
                    [cc(0, 0), cc(1, 0), cc(1, 1), cc(0, 1)],
                )
            }
        }
    }
    // ground decals: contact shadows, holes, fish shadows
    let shc = Color::new(0., 0., 0., 0.22);
    r.disc(vec3(g.me.pos.x, 0.02, g.me.pos.y), vec2(0.32, 0.26), shc);
    for n in g.npcs.iter().filter(|n| !n.inside) {
        r.disc(vec3(n.pos.x, 0.02, n.pos.y), vec2(0.3, 0.24), shc)
    }
    for tr in &g.w.trees {
        let p = tc((tr.x, tr.z));
        if near(p.x, p.y) {
            r.disc(vec3(p.x, 0.02, p.y), vec2(0.7, 0.6), shc)
        }
    }
    for &h in &g.w.holes {
        let p = tc(h);
        r.disc(vec3(p.x, 0.015, p.y), vec2(0.34, 0.3), Color::new(0.22, 0.14, 0.08, 1.))
    }
    for s in &g.shadows {
        let sz = def(s.def).size;
        r.disc(vec3(s.pos.x, -0.18, s.pos.y), vec2(sz * 0.35, sz * 0.6), Color::new(0.03, 0.08, 0.14, 0.5))
    }
    // static scenery
    for z in z0..z1 {
        for x in x0..x1 {
            let (px, pz) = (x as f32 + 0.5, z as f32 + 0.5);
            if !near(px, pz) {
                continue;
            }
            let hsh = hash(x, z);
            match ext(x, z) {
                PINE if (px - g.me.pos.x).abs() < 1.1 && pz > g.me.pos.y + 0.2 && pz < g.me.pos.y + 2.6 => {} // drawn translucent later (occlusion fade)
                PINE => pine(r, px, pz, 1.1 + (hsh % 3) as f32 * 0.12),
                b'f' if hsh % 3 == 0 => pine(r, px + ((hsh % 5) as f32 - 2.) * 0.1, pz, 1.2 + (hsh % 4) as f32 * 0.15),
                ROCK => {
                    r.sphere(vec3(px, 0.12, pz), vec3(0.42, 0.28, 0.36), Color::new(0.6, 0.6, 0.62, 1.));
                    r.sphere(vec3(px + 0.25, 0.08, pz + 0.18), vec3(0.18, 0.13, 0.16), Color::new(0.52, 0.52, 0.55, 1.))
                }
                FLOWER if g.w.trampled.contains(&(x, z)) => {
                    let fc = [RED, YELLOW, PINK, WHITE, PURPLE][(hsh % 5) as usize];
                    for (ox, oz) in [(-0.2, -0.1), (0.15, -0.2), (0.05, 0.2)] {
                        r.sphere(vec3(px + ox, 0.02, pz + oz), vec3(0.1, 0.015, 0.07), fc)
                    }
                }
                FLOWER => {
                    let fc = [RED, YELLOW, PINK, WHITE, PURPLE][(hsh % 5) as usize];
                    for (ox, oz) in [(-0.2, -0.1), (0.15, -0.2), (0.05, 0.2)] {
                        r.cube(vec3(px + ox, 0.12, pz + oz), vec3(0.015, 0.12, 0.015), DARKGREEN);
                        r.sphere(vec3(px + ox, 0.26, pz + oz), vec3(0.09, 0.05, 0.09), fc);
                        r.sphere(vec3(px + ox, 0.29, pz + oz), Vec3::splat(0.03), GOLD)
                    }
                }
                BRIDGE => {
                    r.cube(vec3(px, 0.04, pz), vec3(0.5, 0.05, 0.48), WOOD);
                    for k in 0..4 {
                        r.cube(vec3(px - 0.375 + k as f32 * 0.25, 0.095, pz), vec3(0.01, 0.005, 0.47), sh(WOOD, 0.75))
                    }
                    for zz in [pz - 0.45, pz + 0.45] {
                        r.cube(vec3(px, 0.38, zz), vec3(0.5, 0.03, 0.03), sh(WOOD, 0.85));
                        r.cube(vec3(px - 0.4, 0.2, zz), vec3(0.04, 0.2, 0.04), sh(WOOD, 0.8))
                    }
                }
                _ => {}
            }
        }
    }
    // P2-01: buildings/trees between the camera (south) and the player are drawn translucent, after the player
    let pp = g.me.pos;
    let fade_b: Vec<usize> = (0..BLDS.len())
        .filter(|&i| {
            let b = &BLDS[i];
            pp.x > b.x as f32 - 0.4
                && pp.x < (b.x + b.w) as f32 + 0.4
                && pp.y > b.z as f32 - 2.6
                && pp.y < (b.z + b.d) as f32 - 0.2
        })
        .collect();
    let fade_t: Vec<usize> = (0..g.w.trees.len())
        .filter(|&i| {
            let p = tc((g.w.trees[i].x, g.w.trees[i].z));
            (p.x - pp.x).abs() < 1.1 && p.y > pp.y + 0.2 && p.y < pp.y + 2.6
        })
        .collect();
    let tree_at = |r: &mut R, i: usize| {
        let tr = &g.w.trees[i];
        let sk = g.shake.iter().find(|s| s.0 == i).map_or(0., |s| s.1);
        tree(r, tc((tr.x, tr.z)), tr.fruit.then(|| rgb(def(g.w.native).col)), sk, t, fol)
    };
    for (i, tr) in g.w.trees.iter().enumerate() {
        let p = tc((tr.x, tr.z));
        if near(p.x, p.y) && !fade_t.contains(&i) {
            let sk = g.shake.iter().find(|s| s.0 == i).map_or(0., |s| s.1);
            tree(r, p, tr.fruit.then(|| rgb(def(g.w.native).col)), sk, t, fol)
        }
    }
    for bi in 0..BLDS.len() {
        let b = &BLDS[bi];
        if near(b.x as f32 + b.w as f32 / 2., b.z as f32 + b.d as f32 / 2.) && !fade_b.contains(&bi) {
            let b = &BLDS[bi];
            let paint = if let BK::Home(o) = b.kind {
                g.owner(o).paint.map(|k| (PAINTS[k as usize % PAINTS.len()].1, PAINTS[k as usize % PAINTS.len()].2))
            } else {
                None
            };
            building(r, bi, lt.night, paint)
        }
        if g.me.pos.distance(door_stand(bi)) < 3.5 {
            let p = door_face(bi);
            labels.push((vec3(p.x, 2.7, p.y), g.bname(bi), WHITE))
        }
    }
    for z in 0..H as i32 {
        for x in 0..W as i32 {
            if !near(x as f32 + 0.5, z as f32 + 0.5) || !matches!(tile_at(&g.map, x, z), GRASS | FLOWER) {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if tile_at(&g.map, x + dx, z + dz) != RIVER {
                    continue;
                }
                let (cx, cz) = (x as f32 + 0.5 + dx as f32 * 0.45, z as f32 + 0.5 + dz as f32 * 0.45);
                let along = if dx != 0 { vec3(0., 0., 0.5) } else { vec3(0.5, 0., 0.) };
                let c = vec3(cx, 0., cz);
                let wood = Color::new(0.62, 0.45, 0.28, 1.);
                for e in [-1f32, 1.] {
                    r.cube(c + along * e * 0.92 + vec3(0., 0.2, 0.), vec3(0.035, 0.2, 0.035), sh(wood, 0.85))
                }
                for y in [0.16f32, 0.3] {
                    r.cube(
                        c + vec3(0., y, 0.),
                        if dx != 0 { vec3(0.015, 0.02, 0.48) } else { vec3(0.48, 0.02, 0.015) },
                        wood,
                    )
                }
            }
        }
    }
    // chimney smoke from homes in the morning, evening and cold months
    let smoky = matches!(g.hr, 6..=9 | 17..=23) || matches!(now.month(), 11 | 12 | 1 | 2);
    if smoky {
        let ua = r.alpha;
        r.alpha = 0.5;
        for b in BLDS.iter().filter(|b| matches!(b.kind, BK::Home(_) | BK::Res(_))) {
            let (cx, cz) = (b.x as f32 + b.w as f32 / 2. + b.w as f32 / 2. - 0.6, b.z as f32 + b.d as f32 / 2. - 0.25);
            if !near(cx, cz) {
                continue;
            }
            for k in 0..4 {
                let ph = (t * 0.35 + k as f32 * 0.25 + b.x as f32 * 0.13) % 1.;
                r.sphere(
                    vec3(cx + ph * 0.5 + (t + k as f32).sin() * 0.05, 2.45 + ph * 1.3, cz - ph * 0.2),
                    Vec3::splat(0.1 + ph * 0.16),
                    Color::new(0.9, 0.9, 0.92, 1.),
                )
            }
        }
        r.alpha = ua
    }
    // a small flock of birds circling over town in fair daytime weather
    if (7..18).contains(&g.hr) && weather(g.w.day, now.month()) == Wx::Clear {
        let c0 = g.me.cam + vec2((t * 0.07).sin() * 5., (t * 0.05).cos() * 2. - 9.);
        for k in 0..5 {
            let a = t * 0.4 + k as f32 * 0.5;
            let p = vec3(c0.x + a.cos() * (2.5 + k as f32 * 0.3), 2.6 + (k % 2) as f32 * 0.25, c0.y + a.sin() * 1.5);
            let fl = (t * 8. + k as f32).sin() * 0.5;
            let yaw = -a;
            r.with(Mat4::from_scale_rotation_translation(Vec3::splat(0.6), Quat::from_rotation_y(yaw), p), |r| {
                r.sphere(Vec3::ZERO, vec3(0.05, 0.04, 0.09), Color::new(0.3, 0.3, 0.35, 1.));
                for sgn in [-1f32, 1.] {
                    r.with(Mat4::from_rotation_z(sgn * fl), |r| {
                        r.cube(vec3(sgn * 0.12, 0., 0.), vec3(0.11, 0.008, 0.04), Color::new(0.35, 0.35, 0.4, 1.))
                    })
                }
            })
        }
    }
    {
        let v = &VISITORS[g.visitor()];
        let tc0 = vec3(33., 0., 20.);
        let col = rgb(v.tent);
        r.with(Mat4::from_translation(tc0), |r| {
            r.cube(vec3(0., 0.01, 0.), vec3(1.0, 0.01, 0.95), Color::new(0.55, 0.45, 0.3, 1.));
            for s in [-1f32, 1.] {
                r.with(Mat4::from_translation(vec3(s * 0.45, 0.55, 0.)) * Mat4::from_rotation_z(s * 0.8), |r| {
                    r.cube(Vec3::ZERO, vec3(0.62, 0.03, 0.9), sh(col, if s > 0. { 0.85 } else { 1. }))
                })
            }
            r.cube(vec3(0., 0.45, 0.92), vec3(0.22, 0.4, 0.01), Color::new(0.2, 0.15, 0.12, 1.));
            r.seg(vec3(0., 0., 0.95), vec3(0., 1.15, 0.95), 0.02, WOOD);
            r.cube(vec3(0.2, 1.12, 0.95), vec3(0.15, 0.08, 0.01), YELLOW)
        });
        let f = tc(CAMP_FIRE);
        for k in 0..6 {
            let a = k as f32 * 1.047;
            r.sphere(
                vec3(f.x + a.cos() * 0.3, 0.06, f.y + a.sin() * 0.3),
                vec3(0.1, 0.07, 0.09),
                Color::new(0.5, 0.48, 0.45, 1.),
            )
        }
        for k in 0..2 {
            r.with(Mat4::from_translation(vec3(f.x, 0.06, f.y)) * Mat4::from_rotation_y(k as f32 * 1.57), |r| {
                r.cube(Vec3::ZERO, vec3(0.25, 0.04, 0.04), WOOD)
            })
        }
        let fl = (g.t * 9.).sin() * 0.04;
        let u = r.unlit;
        r.unlit = true;
        r.cone(vec3(f.x, 0.08, f.y), 0.15, 0.32 + fl, Color::new(1., 0.55, 0.15, 1.));
        r.cone(vec3(f.x, 0.08, f.y), 0.08, 0.22 - fl, Color::new(1., 0.9, 0.4, 1.));
        r.unlit = u;
        let cr = tc(CAMP_CRATE);
        r.cube(vec3(cr.x, 0.3, cr.y), vec3(0.32, 0.3, 0.3), Color::new(0.62, 0.45, 0.28, 1.));
        for (k, &d) in v.goods.iter().enumerate() {
            if !g.w.camp_sold.contains(&d) {
                r.cube(vec3(cr.x - 0.18 + k as f32 * 0.18, 0.68, cr.y), Vec3::splat(0.07), rgb(def(d).col))
            }
        }
        let sg = tc(CAMP_SIGN);
        r.cube(vec3(sg.x, 0.45, sg.y), vec3(0.04, 0.45, 0.04), WOOD);
        r.cube(vec3(sg.x, 0.85, sg.y + 0.05), vec3(0.32, 0.18, 0.03), Color::new(0.9, 0.82, 0.6, 1.));
        if g.camp_open() {
            let sp = vec3(CAMP_SPOT.0, 0., CAMP_SPOT.1);
            let talking = g.me.scene == Scene::Town
                && matches!(g.me.ui, Ui::Camp(_) | Ui::Talk { npc: None, .. })
                && g.me.pos.distance(vec2(sp.x, sp.z)) < 2.5;
            r.disc(sp + vec3(0., 0.02, 0.), vec2(0.3, 0.24), Color::new(0., 0., 0., 0.22));
            resident(r, 14 + g.visitor(), sp, yaw_of(g.me.pos - vec2(sp.x, sp.z)), 0., false, g.t, talking, 0.)
        }
        if g.me.pos.distance(vec2(32.5, 20.5)) < 5. {
            labels.push((
                vec3(sg.x, 1.35, sg.y),
                format!("Campground: {} the {}", v.name, v.species),
                Color::new(1., 0.9, 0.7, 1.),
            ))
        }
    }
    if g.art_open() {
        let c = tc(STALL);
        r.cube(vec3(c.x, 0.4, c.y), vec3(0.5, 0.4, 0.35), Color::new(0.35, 0.3, 0.45, 1.));
        r.cube(vec3(c.x, 1.3, c.y), vec3(0.6, 0.04, 0.45), Color::new(0.85, 0.3, 0.4, 1.));
        for x in [-0.55f32, 0.55] {
            r.cube(vec3(c.x + x, 0.65, c.y + 0.4), vec3(0.03, 0.65, 0.03), WOOD)
        }
        painting(r, art_of_week(g.w.day), vec3(c.x, 1.0, c.y + 0.37), 0.5, 0.36);
        if g.me.pos.distance(c) < 4. {
            labels.push((vec3(c.x, 1.7, c.y), "Saturday art stall".into(), Color::new(1., 0.85, 0.9, 1.)))
        }
    }
    for pl in &g.w.garden {
        let p = tc((pl.x, pl.z));
        if !near(p.x, p.y) {
            continue;
        }
        let col = rgb(def(FLOWERS[pl.kind as usize]).col);
        let stem = Color::new(0.3, 0.6, 0.3, 1.);
        r.disc(
            vec3(p.x, 0.012, p.y),
            vec2(0.3, 0.3),
            if pl.watered == g.w.day { Color::new(0.35, 0.25, 0.18, 1.) } else { Color::new(0.5, 0.38, 0.26, 1.) },
        );
        match pl.stage {
            0 => {
                for k in 0..2 {
                    r.with(Mat4::from_translation(vec3(p.x, 0.05, p.y)) * Mat4::from_rotation_y(k as f32 * 1.6), |r| {
                        r.cube(vec3(0.05, 0.04, 0.), vec3(0.05, 0.01, 0.025), stem)
                    })
                }
            }
            1 => {
                r.cyl(vec3(p.x, 0., p.y), 0.025, 0.4, stem);
                r.sphere(vec3(p.x + 0.08, 0.15, p.y), vec3(0.08, 0.02, 0.04), stem);
                r.sphere(vec3(p.x, 0.46, p.y), vec3(0.08, 0.11, 0.08), sh(col, 0.8))
            }
            _ => {
                r.cyl(vec3(p.x, 0., p.y), 0.025, 0.45, stem);
                r.sphere(vec3(p.x + 0.09, 0.16, p.y), vec3(0.09, 0.02, 0.045), stem);
                let u = r.unlit;
                r.unlit = pl.kind == 4 && lt.night > 0.3;
                for k in 0..6 {
                    let a = k as f32 * 1.047 + pl.x as f32;
                    r.sphere(vec3(p.x + a.cos() * 0.13, 0.5, p.y + a.sin() * 0.13), vec3(0.1, 0.035, 0.1), col)
                }
                r.unlit = u;
                r.sphere(vec3(p.x, 0.52, p.y), Vec3::splat(0.06), if pl.kind == 4 { WHITE } else { YELLOW })
            }
        }
    }
    if let Some(sp) = &g.w.surprise {
        let p = tc((sp.x, sp.z));
        let t = g.t;
        match sp.kind {
            0 => r.with(
                Mat4::from_translation(vec3(p.x, 0.08, p.y))
                    * Mat4::from_rotation_z(FRAC_PI_2)
                    * Mat4::from_rotation_x(0.3),
                |r| {
                    r.cyl(Vec3::ZERO, 0.07, 0.22, Color::new(0.45, 0.75, 0.6, 0.85));
                    r.cyl(vec3(0., 0.22, 0.), 0.03, 0.08, Color::new(0.45, 0.75, 0.6, 0.85));
                    r.cube(vec3(0., 0.31, 0.), Vec3::splat(0.025), WOOD)
                },
            ),
            1 => {
                r.cube(vec3(p.x, 0.05, p.y), vec3(0.2, 0.04, 0.08), rgb(def(150).col));
                r.cube(vec3(p.x + 0.12, 0.05, p.y + 0.1), vec3(0.06, 0.03, 0.12), sh(rgb(def(150).col), 0.85))
            }
            _ => {
                r.cube(vec3(p.x, 0.16, p.y), Vec3::splat(0.16), Color::new(0.4, 0.6, 0.95, 1.));
                r.cube(vec3(p.x, 0.16, p.y), vec3(0.17, 0.17, 0.04), YELLOW);
                r.cube(vec3(p.x, 0.16, p.y), vec3(0.04, 0.17, 0.17), YELLOW);
                let by = 1.3 + (t * 1.2).sin() * 0.05;
                r.seg(vec3(p.x, 0.33, p.y), vec3(p.x + 0.05, by - 0.15, p.y), 0.008, WHITE);
                r.sphere(vec3(p.x + 0.05, by, p.y), vec3(0.15, 0.18, 0.15), Color::new(0.95, 0.35, 0.4, 1.))
            }
        }
        let u = r.unlit;
        r.unlit = true;
        for k in 0..3 {
            let a = t * 2. + k as f32 * 2.1;
            let y = 0.35 + ((t * 1.5 + k as f32) % 1.) * 0.5;
            r.sphere(
                vec3(p.x + a.cos() * 0.25, y, p.y + a.sin() * 0.25),
                Vec3::splat(0.025),
                Color::new(1., 0.95, 0.6, 1.),
            )
        }
        r.unlit = u
    }
    if let Some(f) = g.fest() {
        let c = tc(STALL);
        let t = g.t;
        let night = lt.night > 0.4;
        let (main, acc) = [
            (Color::new(1., 0.72, 0.82, 1.), Color::new(1., 1., 1., 1.)),
            (Color::new(0.25, 0.35, 0.7, 1.), Color::new(1., 0.85, 0.4, 1.)),
            (Color::new(0.75, 0.88, 1., 1.), Color::new(0.4, 0.6, 1., 1.)),
            (Color::new(1., 0.55, 0.25, 1.), Color::new(1., 0.9, 0.3, 1.)),
        ][f];
        r.cube(vec3(c.x, 0.4, c.y), vec3(0.5, 0.4, 0.35), WOOD);
        r.cube(vec3(c.x, 0.82, c.y), vec3(0.55, 0.03, 0.4), main);
        let talking = matches!(g.me.ui, Ui::Talk { npc: None, .. }) && g.me.pos.distance(c) < 2.5;
        resident(r, 13, vec3(c.x, 0., c.y - 0.6), 0., 0., false, t, talking, 0.);
        let posts = [(-2.5f32, -1.5f32), (2.5, -1.5), (-2.5, 1.5), (2.5, 1.5)];
        for (px, pz) in posts {
            let p = vec3(c.x + px, 0., c.y + pz);
            r.cube(p + vec3(0., 0.8, 0.), vec3(0.04, 0.8, 0.04), WOOD);
            let u = r.unlit;
            r.unlit = night;
            r.sphere(p + vec3(0., 1.62, 0.), Vec3::splat(0.14), if night { acc } else { main });
            r.unlit = u
        }
        for k in 0..2 {
            let (a, b) = if k == 0 { (posts[0], posts[1]) } else { (posts[2], posts[3]) };
            for j in 0..9 {
                let s = j as f32 / 8.;
                let x = c.x + a.0 + (b.0 - a.0) * s;
                let y = 1.45 - (s * PI).sin() * 0.25;
                r.cone(vec3(x, y, c.y + a.1), 0.07, -0.16, if j % 2 == 0 { main } else { acc })
            }
        }
        match f {
            0 => {
                for (k, (bx, bz)) in [(-1.5f32, 2.6f32), (1.4, 2.8), (0., 3.4)].into_iter().enumerate() {
                    r.cube(
                        vec3(c.x + bx, 0.02, c.y + bz),
                        vec3(0.55, 0.01, 0.45),
                        [
                            Color::new(0.95, 0.4, 0.45, 1.),
                            Color::new(0.4, 0.6, 0.95, 1.),
                            Color::new(0.95, 0.85, 0.4, 1.),
                        ][k],
                    );
                    r.cube(vec3(c.x + bx, 0.12, c.y + bz), vec3(0.15, 0.1, 0.1), WOOD)
                }
                for i in 0..30 {
                    let a = i as f32 * 2.4 + t * 0.3;
                    let rr = 1. + (i % 5) as f32 * 0.6;
                    r.sphere(
                        vec3(c.x + a.cos() * rr, 2.2 - ((t * 0.4 + i as f32 * 0.37) % 2.2), c.y + 1. + a.sin() * rr),
                        vec3(0.04, 0.01, 0.03),
                        main,
                    )
                }
            }
            1 => {
                let mut riv: Vec<(i32, i32)> = (0..H as i32)
                    .flat_map(|z| (0..W as i32).map(move |x| (x, z)))
                    .filter(|&(x, z)| tile_at(&g.map, x, z) == RIVER && (x + z) % 2 == 0)
                    .collect();
                let cam = g.me.cam;
                riv.sort_by(|a, b| tc(*a).distance(cam).total_cmp(&tc(*b).distance(cam)));
                for &(x, z) in riv.iter().take(14) {
                    let bob = (t * 1.5 + x as f32).sin() * 0.02;
                    let drift = ((t * 0.15 + (x * 3 + z) as f32 * 0.37) % 1. - 0.5) * 0.6;
                    let p = vec3(x as f32 + 0.5 + drift * 0.3, -0.12 + bob, z as f32 + 0.5 + drift);
                    r.cube(p, vec3(0.16, 0.03, 0.08), WHITE);
                    r.cone(p + vec3(0., 0.03, 0.), 0.06, 0.1, Color::new(0.95, 0.95, 0.9, 1.));
                    let u = r.unlit;
                    r.unlit = night;
                    r.sphere(p + vec3(0., 0.16, 0.), Vec3::splat(0.04), acc);
                    r.unlit = u
                }
            }
            2 => {
                let p = vec3(c.x, 0., c.y + 2.6);
                r.cyl(p, 0.08, 0.4, WOOD);
                for k in 0..3 {
                    r.cone(
                        p + vec3(0., 0.35 + k as f32 * 0.45, 0.),
                        0.7 - k as f32 * 0.18,
                        0.7,
                        Color::new(0.2, 0.45, 0.35, 1.),
                    )
                }
                let u = r.unlit;
                r.unlit = true;
                for i in 0..14 {
                    let a = i as f32 * 1.9;
                    let h = 0.5 + (i % 7) as f32 * 0.22;
                    let rr = 0.65 - (h - 0.5) * 0.35;
                    r.sphere(
                        p + vec3(a.cos() * rr, h, a.sin() * rr),
                        Vec3::splat(0.05),
                        if (t * 2. + i as f32).sin() > 0. { acc } else { WHITE },
                    )
                }
                r.sphere(p + vec3(0., 1.85, 0.), Vec3::splat(0.1), Color::new(1., 0.95, 0.6, 1.));
                r.unlit = u
            }
            _ => {
                for k in 0..3 {
                    let x = c.x - 2. + k as f32 * 2.;
                    r.cube(vec3(x, 0.9, c.y + 2.6), vec3(0.03, 0.9, 0.03), WOOD);
                    r.cube(
                        vec3(x + 0.25, 1.55, c.y + 2.6),
                        vec3(0.25, 0.18, 0.01),
                        [main, acc, Color::new(0.95, 0.35, 0.35, 1.)][k],
                    )
                }
            }
        }
        if g.me.pos.distance(c) < 6. {
            labels.push((vec3(c.x, 2.1, c.y), FESTS[f].name.into(), main))
        }
    }
    if fair_active(now.month(), now.day()) {
        let c = tc(STALL);
        r.cube(vec3(c.x, 0.45, c.y), vec3(0.48, 0.45, 0.4), WOOD);
        for k in 0..5 {
            r.cube(
                vec3(c.x - 0.4 + k as f32 * 0.2, 1.25, c.y + 0.25),
                vec3(0.1, 0.03, 0.35),
                if k % 2 == 0 { Color::new(0.95, 0.55, 0.2, 1.) } else { WHITE },
            )
        }
        for x in [-0.45, 0.45] {
            r.cube(vec3(c.x + x, 0.62, c.y + 0.3), vec3(0.03, 0.62, 0.03), WOOD)
        }
        for (k, f) in [20u16, 21, 22, 23].iter().enumerate() {
            r.sphere(vec3(c.x - 0.3 + k as f32 * 0.2, 0.98, c.y + 0.1), Vec3::splat(0.08), rgb(def(*f).col))
        }
        let u = r.unlit;
        for (px, pz) in [(-2.5, -1.5), (2.5, -1.5), (-2.5, 1.5), (2.5, 1.5)] {
            let p = vec3(c.x + px, 0., c.y + pz);
            r.unlit = false;
            r.cube(p + vec3(0., 0.7, 0.), vec3(0.04, 0.7, 0.04), WOOD);
            r.unlit = true;
            r.sphere(p + vec3(0., 1.45, 0.), Vec3::splat(0.12), Color::new(1., 0.75 + 0.1 * px.signum(), 0.35, 1.))
        }
        r.unlit = u;
        if g.me.pos.distance(c) < 5. {
            labels.push((vec3(c.x, 2.0, c.y), "Harvest Fair".into(), Color::new(1., 0.8, 0.4, 1.)))
        }
    }
    for b in &g.bubbles {
        for k in 0..3 {
            let ph = (t * 1.5 + k as f32 * 2.1) % 1.;
            r.sphere(
                vec3(b.0.x + (k as f32 - 1.) * 0.12, -0.18 + ph * 0.05, b.0.y + ph * 0.1),
                Vec3::splat(0.04 + ph * 0.03),
                Color::new(0.95, 0.98, 1., 1.),
            )
        }
    }
    {
        let b = tc(BOARD);
        r.cube(vec3(b.x - 0.35, 0.6, b.y), vec3(0.04, 0.6, 0.04), WOOD);
        r.cube(vec3(b.x + 0.35, 0.6, b.y), vec3(0.04, 0.6, 0.04), WOOD);
        r.cube(vec3(b.x, 0.95, b.y), vec3(0.45, 0.3, 0.04), Color::new(0.5, 0.35, 0.22, 1.));
        for (k, c) in [WHITE, Color::new(1., 0.9, 0.6, 1.), Color::new(0.8, 0.9, 1., 1.)].iter().enumerate() {
            r.cube(vec3(b.x - 0.25 + k as f32 * 0.25, 0.97, b.y + 0.045), vec3(0.09, 0.12, 0.005), *c)
        }
    }
    for bt in &g.w.built {
        draw_project(r, bt.kind, bt.x, bt.z, lt.night, false)
    }
    if g.contest_host_open() {
        let (x, z) = crate::events::CONTEST_SPOT;
        r.disc(vec3(x, 0.02, z), vec2(0.28, 0.23), Color::new(0., 0., 0., 0.2));
        human(r, 1, (None, Some(67), 0), vec3(x, 0., z), 0., 0., 0., 0, None, t, 0.);
        if g.me.pos.distance(vec2(x, z)) < 5. {
            labels.push((vec3(x, 1.5, z), crate::events::CONTEST_HOST.into(), WHITE));
        }
    }
    if let Some(m) = &g.w.mproject {
        let p = tc((m.x, m.z));
        r.cube(vec3(p.x, 0.45, p.y + 0.35), vec3(0.04, 0.45, 0.04), WOOD);
        r.cube(vec3(p.x, 0.85, p.y + 0.35), vec3(0.3, 0.18, 0.03), Color::new(0.95, 0.85, 0.5, 1.));
        r.alpha = 0.35;
        draw_project(r, MPROJECTS[m.idx].2, m.x, m.z, 0., true);
        r.alpha = 1.;
        if g.me.pos.distance(p) < 4. {
            labels.push((
                vec3(p.x, 1.4, p.y + 0.35),
                format!("{} {}/{}c", MPROJECTS[m.idx].0, m.funded, MPROJECTS[m.idx].1),
                Color::new(1., 0.95, 0.6, 1.),
            ))
        }
    }
    if let Some(idx) = g.me.mplace {
        let t = tile_of(g.me.pos + g.me.face);
        let mut gc = Game::from_world(g.w.clone(), g.map.clone());
        gc.me.pos = g.me.pos;
        gc.me.face = g.me.face;
        let ok = gc.mp_site_ok(t);
        r.disc(
            vec3(t.0 as f32 + 0.5, 0.03, t.1 as f32 + 0.5),
            vec2(0.48, 0.48),
            if ok { Color::new(0.2, 0.95, 0.3, 0.7) } else { Color::new(0.95, 0.2, 0.2, 0.6) },
        );
        r.alpha = 0.4;
        draw_project(r, MPROJECTS[idx].2, t.0, t.1, 0., true);
        r.alpha = 1.
    }
    {
        let (dx, dz) = (DOCK.0 as f32 + 0.5, DOCK.1 as f32);
        for k in 0..3 {
            r.cube(vec3(dx, 0.05, dz + 0.5 + k as f32), vec3(0.5, 0.05, 0.5), WOOD);
            r.cube(vec3(dx - 0.45, -0.05, dz + 0.5 + k as f32), vec3(0.05, 0.25, 0.05), sh(WOOD, 0.7));
            r.cube(vec3(dx + 0.45, -0.05, dz + 0.5 + k as f32), vec3(0.05, 0.25, 0.05), sh(WOOD, 0.7))
        }
        r.cube(vec3(dx + 1.0, -0.05, dz + 2.3), vec3(0.35, 0.12, 0.8), Color::new(0.85, 0.85, 0.9, 1.));
        r.cube(vec3(dx + 1.0, 0.08, dz + 2.3), vec3(0.3, 0.02, 0.7), WOOD);
        resident(r, 12, vec3(dx, 0.1, dz + 0.4), 0., 0., false, g.t, false, 0.);
        if g.me.pos.distance(vec2(dx, dz)) < 4. {
            labels.push((vec3(dx, 1.9, dz + 0.4), "Captain Pell's boat".into(), Color::new(0.8, 0.95, 1., 1.)))
        }
    }
    if let Some(p) = &g.w.project {
        let (sx, sz) = if p.kind == PK_BRIDGE {
            (p.x as f32 + 1., p.z as f32 + 1.3)
        } else {
            (p.x as f32 + 0.5, p.z as f32 + 0.5)
        };
        r.cube(vec3(sx, 0.45, sz + 0.3), vec3(0.04, 0.45, 0.04), WOOD);
        r.cube(vec3(sx, 0.85, sz + 0.3), vec3(0.3, 0.18, 0.03), Color::new(0.85, 0.75, 0.55, 1.));
        r.cube(
            vec3(sx, 0.85, sz + 0.335),
            vec3(0.3 * p.need.iter().map(|n| n.2 as f32 / n.1 as f32).sum::<f32>() / p.need.len() as f32, 0.04, 0.005),
            Color::new(0.3, 0.7, 0.35, 1.),
        );
        r.alpha = 0.35;
        draw_project(r, p.kind, p.x, p.z, 0., true);
        r.alpha = 1.;
        if g.me.pos.distance(vec2(sx, sz)) < 4. {
            let done: u16 = p.need.iter().map(|n| n.2).sum();
            let tot: u16 = p.need.iter().map(|n| n.1).sum();
            labels.push((
                vec3(sx, 1.4, sz + 0.3),
                format!("{} {done}/{tot}", PROJECTS[p.kind as usize].0),
                Color::new(0.8, 1., 0.8, 1.),
            ))
        }
    }
    for o in 0..2 {
        let p = tc(mailbox_tile(o));
        let unread = if g.w.p.id == o {
            g.w.p.mail.iter().any(|l| !l.read)
        } else {
            g.w.others.iter().find(|pr| pr.id == o).is_some_and(|pr| pr.mail.iter().any(|l| !l.read))
        };
        r.cube(vec3(p.x, 0.35, p.y), vec3(0.04, 0.35, 0.04), WOOD);
        r.cube(
            vec3(p.x, 0.78, p.y),
            vec3(0.16, 0.12, 0.22),
            if o == 0 { Color::new(0.85, 0.3, 0.3, 1.) } else { Color::new(0.3, 0.6, 0.5, 1.) },
        );
        r.with(
            Mat4::from_translation(vec3(p.x + 0.18, 0.8, p.y - 0.05))
                * Mat4::from_rotation_z(if unread { 0. } else { -FRAC_PI_2 }),
            |r| {
                r.cube(vec3(0., 0.12, 0.), vec3(0.012, 0.12, 0.012), DARKGRAY);
                r.cube(vec3(0., 0.2, 0.06), vec3(0.01, 0.04, 0.06), YELLOW)
            },
        )
    }
    for &w in &g.w.weeds {
        let p = tc(w);
        for (ox, oz) in [(-0.1, 0.), (0.08, 0.06), (0., -0.08)] {
            r.cone(vec3(p.x + ox, 0., p.y + oz), 0.05, 0.22, Color::new(0.25, 0.5, 0.2, 1.))
        }
    }
    for &h in &g.w.holes {
        let p = tc(h);
        for k in 0..5 {
            let a = k as f32 * 1.26;
            r.sphere(
                vec3(p.x + a.cos() * 0.36, 0.03, p.y + a.sin() * 0.32),
                vec3(0.1, 0.06, 0.1),
                Color::new(0.5, 0.36, 0.24, 1.),
            )
        }
    }
    for &dg in &g.w.digs {
        let p = tc(dg);
        for a in [0.7f32, -0.7] {
            r.with(Mat4::from_translation(vec3(p.x, 0.012, p.y)) * Mat4::from_rotation_y(a), |r| {
                r.cube(Vec3::ZERO, vec3(0.2, 0.01, 0.025), Color::new(0.35, 0.25, 0.16, 1.))
            })
        }
    }
    for it in &g.w.ground {
        let c = rgb(def(it.def).col);
        match def(it.def).k {
            K::Shell => {
                r.sphere(vec3(it.x, 0.05, it.z), vec3(0.14, 0.06, 0.12), c);
                r.cone(vec3(it.x, 0.05, it.z), 0.08, 0.1, sh(c, 0.9))
            }
            _ => r.sphere(vec3(it.x, 0.11, it.z), Vec3::splat(0.11), c),
        }
    }
    for (i, n) in g.npcs.iter().enumerate() {
        if n.inside || !near(n.pos.x, n.pos.y) {
            continue;
        }
        let talking = matches!(g.me.ui, Ui::Talk { npc: Some(j), .. } if j == i);
        let near_me = !n.moving && n.pos.distance(g.me.pos) < 3. && g.me.scene == Scene::Town;
        let yaw = if near_me || talking { yaw_of(g.me.pos - n.pos) } else { n.yaw }; // idle residents look at a nearby player
        resident_u(
            r,
            g.w.roster[i],
            vec3(n.pos.x, 0., n.pos.y),
            yaw,
            n.anim,
            n.moving,
            t,
            talking,
            n.happy,
            weather(g.w.day, now.month()) == Wx::Rain,
        );
        if let Some(e) = emote(g, i, t) {
            let hp = vec3(n.pos.x + 0.25, 1.55 + (t * 2.).sin() * 0.03, n.pos.y);
            draw_emote(r, e, hp, t, labels)
        }
        if !n.moving && !talking && g.seek_hide_position(i).is_none() && weather(g.w.day, now.month()) != Wx::Rain {
            hobby_prop(r, g.w.roster[i], vec3(n.pos.x, 0., n.pos.y), yaw, t)
        }
        if let Some(&(a, b, left)) = g.chats.iter().find(|c| c.0 == i || c.1 == i) {
            let first = a == i;
            let speak = ((5. - left) as i32 % 2 == 0) == first;
            let pers = g.res(i).pers;
            if speak && n.pos.distance(g.me.pos) < 7. {
                labels.push((vec3(n.pos.x, 1.95, n.pos.y), format!("\"{}\"", CHATTER[pers][(i + b) % 3]), WHITE))
            }
        } else if n.pos.distance(g.me.pos) < 4. {
            labels.push((vec3(n.pos.x, 1.75, n.pos.y), g.res(i).name.into(), Color::new(1., 0.95, 0.7, 1.)))
        }
    }
    let sy = if g.swimming() {
        for k in 0..10 {
            let a = k as f32 * 0.63 + t;
            r.sphere(
                vec3(g.me.pos.x + a.cos() * 0.38, -0.17, g.me.pos.y + a.sin() * 0.3),
                vec3(0.05, 0.015, 0.05),
                Color::new(0.9, 0.97, 1., 1.),
            )
        }
        -0.55
    } else {
        0.
    };
    let opose = match g.me.pose {
        Pose::Stand => 0,
        Pose::Sit(_) => 1,
        Pose::Lie(_) => 2,
    };
    human(
        r,
        g.me.id,
        (g.w.p.shirt, g.w.p.hat, g.w.p.hair),
        vec3(g.me.pos.x, sy + if opose == 1 { 0.02 } else { 0. }, g.me.pos.y),
        yaw_of(g.me.face),
        g.me.anim,
        g.me.spd,
        opose,
        g.w.p.equip.map(|t| t + 1000 * g.tool_level(t) as u16),
        t,
        g.me.swing,
    );
    others(r, g, t);
    for b in &g.bugs {
        let c = rgb(def(b.0).col);
        let fl = (t * 20. + b.2).sin().abs();
        let y = 0.6 + (t * 3. + b.2).sin() * 0.15;
        for s in [-1f32, 1.] {
            r.sphere(vec3(b.1.x + s * 0.07, y + fl * 0.04, b.1.y), vec3(0.07, 0.01 + fl * 0.03, 0.05), c)
        }
        r.sphere(vec3(b.1.x, y, b.1.y), Vec3::splat(0.03), DARK)
    }
    if let Some(f) = &g.me.fish {
        let y = -0.15 - f.dip * 0.1;
        r.sphere(vec3(f.bob.x, y, f.bob.y), Vec3::splat(0.06), RED);
        r.sphere(vec3(f.bob.x, y + 0.05, f.bob.y), Vec3::splat(0.04), WHITE);
        let tip = g.me.pos + g.me.face * 0.75;
        r.seg(vec3(tip.x, 1.05, tip.y), vec3(f.bob.x, y + 0.05, f.bob.y), 0.004, Color::new(0.95, 0.95, 0.95, 1.))
    }
    r.alpha = 0.32;
    for z in (pp.y as i32)..(pp.y as i32 + 3) {
        for x in (pp.x as i32 - 1)..=(pp.x as i32 + 1) {
            let (px, pz) = (x as f32 + 0.5, z as f32 + 0.5);
            if ext(x, z) == PINE && (px - pp.x).abs() < 1.1 && pz > pp.y + 0.2 && pz < pp.y + 2.6 {
                pine(r, px, pz, 1.1 + (hash(x, z) % 3) as f32 * 0.12)
            }
        }
    }
    for &i in &fade_t {
        tree_at(r, i)
    }
    for &bi in &fade_b {
        let b = &BLDS[bi];
        let paint = if let BK::Home(o) = b.kind {
            g.owner(o).paint.map(|k| (PAINTS[k as usize % PAINTS.len()].1, PAINTS[k as usize % PAINTS.len()].2))
        } else {
            None
        };
        building(r, bi, lt.night, paint)
    }
    r.alpha = 1.;
    for &(p, a, k) in &g.fx {
        let s = a / 0.8;
        match k {
            0 => {
                for i in 0..10 {
                    let an = i as f32 / 10. * 2. * PI;
                    let rr = 0.15 + s * 0.6;
                    r.sphere(
                        p + vec3(an.cos() * rr, 0.02, an.sin() * rr),
                        vec3(0.05, 0.015, 0.05),
                        Color::new(0.92, 0.97, 1., 1. - s),
                    )
                }
            }
            1 => {
                for i in 0..6 {
                    let an = i as f32 * 1.05;
                    r.sphere(
                        p + vec3(an.cos() * s * 0.45, s * 1.1 - s * s * 0.8, an.sin() * s * 0.6),
                        Vec3::splat(0.12 * (1. - s) + 0.03),
                        Color::new(0.86, 0.74, 0.55, 1.),
                    )
                }
            }
            _ => {
                for i in 0..5 {
                    let an = -0.8 + i as f32 * 0.4;
                    let d = vec3(an.sin() * 0.35, 0., an.cos() * 0.35);
                    r.sphere(p + d * (0.5 + s), Vec3::splat(0.035), Color::new(1., 1., 1., 1. - s))
                }
            }
        }
    }
    for h in &g.hearts {
        if h.1 > 0. {
            r.sphere(h.0, Vec3::splat(0.06), Color::new(1., 0.4, 0.55, 1.))
        }
    }
    if g.me.ui == Ui::Play && g.me.trans.is_none() {
        if let Some(tg) = g.target() {
            indicator(r, g.tg_pos(tg), t)
        }
    }
}
/// Other local players who are in the same scene as the viewing player.
pub(super) fn others(r: &mut R, g: &Game, t: f32) {
    for o in g.others.iter().filter(|o| o.scene == g.me.scene) {
        let p = if o.scene == Scene::Town { o.pos } else { o.ipos };
        let pose = match o.pose {
            Pose::Stand => 0,
            Pose::Sit(_) => 1,
            Pose::Lie(_) => 2,
        };
        let tool = if o.scene == Scene::Town {
            g.w.others.iter().find(|pr| pr.id == o.id).and_then(|pr| {
                pr.equip.map(|t| {
                    t + 1000
                        * UPGRADABLE.iter().position(|&u| u == t).and_then(|k| pr.tool_lv.get(k).copied()).unwrap_or(0)
                            as u16
                })
            })
        } else {
            None
        };
        if pose == 0 {
            r.disc(vec3(p.x, 0.02, p.y), vec2(0.32, 0.26), Color::new(0., 0., 0., 0.22))
        }
        let shirt =
            g.w.others.iter().find(|pr| pr.id == o.id).map_or((None, None, 0), |pr| (pr.shirt, pr.hat, pr.hair));
        let sy = if o.scene == Scene::Town && tile_at(&g.map, p.x.floor() as i32, p.y.floor() as i32) == SEA {
            -0.55
        } else {
            0.
        };
        human(r, o.id, shirt, vec3(p.x, sy, p.y), yaw_of(o.face), o.anim, o.spd, pose, tool, t, o.swing);
    }
}
