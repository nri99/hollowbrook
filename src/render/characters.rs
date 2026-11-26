//! Characters: players and residents (bodies, heads, faces, emotes, hobby props).
use super::*;

/// Player: large head, hair, face, outfit; pose 0 stand/walk/run, 1 sit, 2 lie. Tool held in the right hand.
pub(super) fn shirt_base(id: usize) -> Color {
    if id == 0 {
        Color::new(0.95, 0.47, 0.36, 1.)
    } else {
        Color::new(0.3, 0.66, 0.48, 1.)
    }
}
/// `look` = (shirt, hat, hairstyle index) from the player's profile.
pub fn human(
    r: &mut R,
    id: usize,
    look: (Option<u16>, Option<u16>, u8),
    pos: Vec3,
    yaw: f32,
    anim: f32,
    spd: f32,
    pose: u8,
    tool: Option<u16>,
    t: f32,
    swing: f32,
) {
    let (skin, hair, shirt, pants, shoe) = if id == 0 {
        (
            Color::new(0.98, 0.82, 0.68, 1.),
            Color::new(0.36, 0.2, 0.12, 1.),
            Color::new(0.95, 0.47, 0.36, 1.),
            Color::new(0.27, 0.36, 0.62, 1.),
            Color::new(0.3, 0.2, 0.15, 1.),
        )
    } else {
        (
            Color::new(0.85, 0.66, 0.5, 1.),
            Color::new(0.12, 0.1, 0.1, 1.),
            Color::new(0.3, 0.66, 0.48, 1.),
            Color::new(0.35, 0.3, 0.26, 1.),
            Color::new(0.2, 0.2, 0.25, 1.),
        )
    };
    let shirt = look.0.map_or(shirt_base(id), |d| rgb(def(d).col));
    let hair = if look.2 == 0 { hair } else { rgb(HAIRS[look.2 as usize % HAIRS.len()]) };
    let k = (spd / 3.).min(1.8);
    let sw = anim.sin() * 0.7 * k;
    let bob = (anim * 2.).sin().abs() * 0.05 * k + (t * 2.2).sin() * 0.01;
    let lean = if spd > 4. { 0.2 } else { 0. };
    let mut base = Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), pos);
    if pose == 2 {
        base = base * Mat4::from_translation(vec3(0., 0.45, 0.55)) * Mat4::from_rotation_x(-FRAC_PI_2)
    }
    let hip = if pose == 1 { 0.14 } else { 0. };
    r.with(base, |r| {
        if spd > 4. && pose == 0 && pos.y > -0.1 {
            for k in 0..3 {
                let ph = (anim * 0.35 + k as f32 / 3.).fract();
                r.sphere(
                    vec3((k as f32 - 1.) * 0.08, 0.04 + ph * 0.08, -0.2 - ph * 0.35),
                    Vec3::splat(0.04 + ph * 0.05),
                    Color::new(0.92, 0.88, 0.78, 0.55 * (1. - ph)),
                )
            }
        }
        for s in [-1f32, 1.] {
            let a = if pose == 1 { -FRAC_PI_2 } else { sw * s };
            r.with(Mat4::from_translation(vec3(s * 0.085, 0.28 + hip, 0.)) * Mat4::from_rotation_x(a), |r| {
                r.cube(vec3(0., -0.12, 0.), vec3(0.06, 0.12, 0.065), pants);
                r.cube(vec3(0., -0.255, 0.03), vec3(0.065, 0.035, 0.095), shoe)
            })
        }
        r.with(Mat4::from_translation(vec3(0., 0.28 + hip + bob, 0.)) * Mat4::from_rotation_x(lean), |r| {
            r.cube(vec3(0., 0.15, 0.), vec3(0.16, 0.16, 0.11), shirt);
            r.cube(vec3(0., 0.02, 0.), vec3(0.165, 0.03, 0.115), pants);
            for s in [-1f32, 1.] {
                let a = if pose == 1 {
                    -0.6
                } else if s > 0. && swing > 0. {
                    -swing * 3.
                } else {
                    -sw * s
                };
                r.with(Mat4::from_translation(vec3(s * 0.2, 0.28, 0.)) * Mat4::from_rotation_x(a), |r| {
                    r.cube(vec3(0., -0.1, 0.), vec3(0.05, 0.1, 0.055), shirt);
                    r.sphere(vec3(0., -0.22, 0.), Vec3::splat(0.055), skin);
                    if s > 0. {
                        if let Some(tl) = tool {
                            r.with(Mat4::from_translation(vec3(0., -0.22, 0.03)), |r| draw_tool(r, tl))
                        }
                    }
                })
            }
            r.with(
                Mat4::from_translation(vec3(0., 0.6, 0.))
                    * Mat4::from_rotation_y((t * 0.6).sin() * 0.12 * (1. - k.min(1.))),
                |r| {
                    r.sphere(Vec3::ZERO, vec3(0.28, 0.27, 0.27), skin);
                    r.sphere(vec3(0., 0.08, -0.03), vec3(0.295, 0.25, 0.28), hair);
                    r.cube(vec3(0., 0.17, 0.19), vec3(0.2, 0.055, 0.07), hair);
                    if look.2 % 2 == 1 {
                        r.sphere(vec3(0., 0.2, -0.27), Vec3::splat(0.11), hair)
                    }
                    match look.1 {
                        Some(67) => {
                            r.cyl(vec3(0., 0.2, 0.), 0.42, 0.025, rgb(def(67).col));
                            r.cyl(vec3(0., 0.22, 0.), 0.24, 0.16, rgb(def(67).col));
                            r.cyl(vec3(0., 0.23, 0.), 0.245, 0.04, Color::new(0.8, 0.3, 0.3, 1.))
                        }
                        Some(68) => {
                            r.sphere(vec3(0., 0.12, -0.02), vec3(0.3, 0.24, 0.29), rgb(def(68).col));
                            r.sphere(vec3(0., 0.37, -0.02), Vec3::splat(0.06), WHITE)
                        }
                        Some(69) => {
                            for k in 0..9 {
                                let a = k as f32 / 9. * 2. * PI;
                                r.sphere(
                                    vec3(a.cos() * 0.26, 0.2, a.sin() * 0.25),
                                    Vec3::splat(0.05),
                                    [PINK, YELLOW, WHITE][k % 3],
                                )
                            }
                        }
                        _ => {}
                    }
                    let bl = if (t + id as f32 * 1.7) % 4.2 < 0.13 { 0.12 } else { 1. };
                    for s in [-1f32, 1.] {
                        r.cube(vec3(s * 0.1, 0.058, 0.268), vec3(0.042, 0.011, 0.008), sh(hair, 0.55));
                        r.sphere(vec3(s * 0.095, -0.01, 0.245), vec3(0.038, 0.055 * bl, 0.02), DARK);
                        if bl > 0.5 {
                            r.sphere(vec3(s * 0.09, 0.015, 0.262), Vec3::splat(0.012), WHITE)
                        }
                        r.sphere(
                            vec3(s * 0.165, -0.085, 0.215),
                            vec3(0.045, 0.03, 0.02),
                            Color::new(1., 0.62, 0.62, 1.),
                        )
                    }
                    r.cube(vec3(0., -0.12, 0.255), vec3(0.035, 0.01, 0.01), Color::new(0.6, 0.25, 0.25, 1.));
                },
            )
        })
    });
}
/// Residents 0-3 and the shopkeeper (4): distinct heads, ears, muzzles, tails, clothes; talk mouth + happy eyes.
/// Mood emote over a resident's head (0 note, 1 sleepy, 2 thinking, 3 excited, 4 greeting), shown in short idle bursts.
pub fn emote(g: &Game, i: usize, t: f32) -> Option<u8> {
    if g.seek_hide_position(i).is_some() {
        return None;
    }
    if g.w.p.invitation.as_ref().is_some_and(|v| v.slot == i && v.day == g.w.day && v.state == 0) {
        return Some(4);
    }
    let n = &g.npcs[i];
    if n.happy > 0. || g.chats.iter().any(|c| c.0 == i || c.1 == i) {
        return None;
    }
    let r = g.res(i);
    if n.pos.distance(g.me.pos) < 2.2 && !n.moving && !g.w.p.npcs[i].talked {
        return Some(4);
    }
    if (t * 0.25 + i as f32 * 1.7) % 5. > 1.4 {
        return None;
    }
    if g.hr + 1 >= r.sleep || g.hr < r.wake + 1 {
        return Some(1);
    }
    Some([0, 2, 2, 3][r.pers])
}
/// Idle hobby prop held at the resident's side (CHARS order): basket, watering can, telescope, towel, yarn, wrench.
pub fn hobby_prop(r: &mut R, ch: usize, pos: Vec3, yaw: f32, t: f32) {
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), pos + vec3(0., 0., 0.)), |r| {
        let h = vec3(0.36, 0.34, 0.2);
        r.seg(vec3(0.22, 0.42, 0.06), h, 0.022, Color::new(0.97, 0.94, 0.88, 1.)); // arm reaching to the prop
        match ch {
            0 => {
                r.cyl(h - vec3(0., 0.08, 0.), 0.09, 0.1, Color::new(0.75, 0.55, 0.3, 1.));
                r.sphere(h + vec3(0., 0.04, 0.), vec3(0.07, 0.03, 0.07), Color::new(0.95, 0.85, 0.6, 1.))
            }
            1 => {
                r.cyl(h - vec3(0., 0.08, 0.), 0.09, 0.15, Color::new(0.4, 0.6, 0.45, 1.));
                r.seg(h + vec3(0., 0., 0.05), h + vec3(0., -0.02, 0.2), 0.015, Color::new(0.4, 0.6, 0.45, 1.));
                let ph = (t * 3.) % 1.;
                r.sphere(h + vec3(0., -0.05 - ph * 0.25, 0.22), Vec3::splat(0.015), Color::new(0.5, 0.75, 1., 1.))
            }
            2 => {
                r.seg(h, h + vec3(0.05, 0.3, 0.12), 0.045, Color::new(0.8, 0.68, 0.35, 1.));
                r.cyl(h + vec3(0.05, 0.3, 0.12), 0.05, 0.04, Color::new(0.3, 0.3, 0.35, 1.))
            }
            3 => r.cube(h + vec3(0., -0.05, 0.), vec3(0.03, 0.12, 0.08), Color::new(0.95, 0.95, 0.95, 1.)),
            4 => {
                r.sphere(h, Vec3::splat(0.06), Color::new(0.85, 0.4, 0.55, 1.));
                r.seg(h + vec3(-0.05, 0.05, 0.), h + vec3(0.05, 0.12, 0.05), 0.006, Color::new(0.8, 0.75, 0.6, 1.))
            }
            _ => r.seg(h, h + vec3(0., 0.18, 0.06), 0.02, Color::new(0.6, 0.6, 0.65, 1.)),
        }
    });
}
pub fn draw_emote(r: &mut R, e: u8, p: Vec3, t: f32, labels: &mut Vec<(Vec3, String, Color)>) {
    match e {
        0 => {
            let c = Color::new(0.35, 0.3, 0.8, 1.);
            r.sphere(p, vec3(0.06, 0.045, 0.04), c);
            r.seg(p + vec3(0.05, 0., 0.), p + vec3(0.05, 0.2, 0.), 0.012, c);
            r.seg(p + vec3(0.05, 0.2, 0.), p + vec3(0.12, 0.15, 0.), 0.012, c)
        }
        1 => labels.push((
            p,
            if (t * 1.5) as i32 % 2 == 0 { "z".into() } else { "z Z".into() },
            Color::new(0.8, 0.85, 1., 1.),
        )),
        2 => labels.push((p, "...".into(), Color::new(0.95, 0.95, 0.95, 1.))),
        3 => labels.push((p, "!".into(), Color::new(1., 0.85, 0.3, 1.))),
        _ => {
            r.sphere(p, Vec3::splat(0.07), Color::new(1., 0.6, 0.65, 1.));
            r.sphere(p + vec3(0.07, 0.04, 0.), Vec3::splat(0.07), Color::new(1., 0.6, 0.65, 1.));
            r.cone(p + vec3(0.035, -0.02, 0.), 0.1, -0.12, Color::new(1., 0.6, 0.65, 1.))
        }
    }
}
pub fn resident(r: &mut R, i: usize, pos: Vec3, yaw: f32, anim: f32, moving: bool, t: f32, talk: bool, happy: f32) {
    resident_u(r, i, pos, yaw, anim, moving, t, talk, happy, false)
}
/// `umbrella`: held over the head (outdoors in rain).
pub fn resident_u(
    r: &mut R,
    i: usize,
    pos: Vec3,
    yaw: f32,
    anim: f32,
    moving: bool,
    t: f32,
    talk: bool,
    happy: f32,
    umbrella: bool,
) {
    let (fur, cloth) = if i < CHARS.len() {
        (rgb(CHARS[i].fur), rgb(CHARS[i].cloth))
    } else if (14..18).contains(&i) {
        (rgb(VISITORS[i - 14].fur), rgb(VISITORS[i - 14].cloth))
    } else if i == 13 {
        (Color::new(0.82, 0.66, 0.48, 1.), Color::new(0.3, 0.48, 0.44, 1.))
    } else if i == 12 {
        (Color::new(0.96, 0.96, 0.94, 1.), Color::new(0.2, 0.25, 0.45, 1.))
    } else if i == 11 {
        (Color::new(0.93, 0.93, 0.96, 1.), Color::new(0.32, 0.34, 0.6, 1.))
    } else {
        (Color::new(0.55, 0.65, 0.38, 1.), Color::new(0.92, 0.86, 0.7, 1.))
    };
    let light = Color::new(0.97, 0.94, 0.88, 1.);
    let sw = if moving { anim.sin() * 0.6 } else { 0. };
    let jump = if happy > 0. { (t * 10.).sin().abs() * 0.18 } else { 0. };
    let bob = (t * 2. + i as f32).sin() * 0.012;
    r.with(Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), pos + vec3(0., jump, 0.)), |r| {
        for s in [-1f32, 1.] {
            r.with(Mat4::from_translation(vec3(s * 0.09, 0.24, 0.)) * Mat4::from_rotation_x(sw * s), |r| {
                r.cube(vec3(0., -0.11, 0.02), vec3(0.065, 0.12, 0.08), fur)
            })
        }
        r.with(Mat4::from_translation(vec3(0., 0.24 + bob, 0.)), |r| {
            r.cube(vec3(0., 0.15, 0.), vec3(0.17, 0.16, 0.12), cloth);
            if i == 3 {
                for y in [0.08, 0.2] {
                    r.cube(vec3(0., y, 0.), vec3(0.175, 0.025, 0.125), WHITE)
                }
            }
            if i == 10 {
                r.sphere(vec3(0., 0.2, -0.13), vec3(0.3, 0.26, 0.24), Color::new(0.45, 0.38, 0.22, 1.))
            }
            for s in [-1f32, 1.] {
                let wave = if talk && s > 0. { (t * 5.).sin() * 0.4 - 0.5 } else { 0. };
                r.with(Mat4::from_translation(vec3(s * 0.21, 0.27, 0.)) * Mat4::from_rotation_x(-sw * s + wave), |r| {
                    r.cube(vec3(0., -0.1, 0.), vec3(0.05, 0.1, 0.05), fur)
                })
            }
            let wag = (t * 4.).sin();
            if i == 15 {
                r.sphere(vec3(0., 0.2, -0.14), vec3(0.24, 0.2, 0.16), Color::new(0.4, 0.5, 0.3, 1.));
                for k in 0..3 {
                    r.sphere(
                        vec3(-0.1 + k as f32 * 0.1, 0.28, -0.27),
                        vec3(0.05, 0.05, 0.02),
                        Color::new(0.55, 0.62, 0.38, 1.),
                    )
                }
            }
            match i {
                0 => {
                    r.sphere(vec3(wag * 0.05, 0.05, -0.28), vec3(0.12, 0.12, 0.22), fur);
                    r.sphere(vec3(wag * 0.07, 0.07, -0.47), Vec3::splat(0.08), light)
                }
                1 => r.sphere(vec3(0., 0.02, -0.15), Vec3::splat(0.07), fur),
                3 => {
                    r.sphere(vec3(wag * 0.03, 0., -0.22), vec3(0.08, 0.07, 0.16), fur);
                    r.sphere(vec3(wag * 0.06, -0.05, -0.38), vec3(0.06, 0.05, 0.12), fur)
                }
                _ => {}
            }
            r.with(
                Mat4::from_translation(vec3(0., 0.58, 0.02))
                    * Mat4::from_rotation_z(if talk { (t * 3.).sin() * 0.08 } else { 0. })
                    * Mat4::from_rotation_x(if talk { (t * 5.).sin().max(0.) * -0.06 } else { 0. }),
                |r| head(r, i, fur, light, t, talk, happy),
            );
            if umbrella {
                r.seg(vec3(0.2, 0.15, 0.08), vec3(0.08, 1.05, 0.02), 0.015, DARKGRAY);
                r.cone(vec3(0.08, 0.95, 0.02), 0.5, 0.22, rgb(CHARS[i.min(CHARS.len() - 1)].cloth));
            }
        })
    });
}
pub(super) fn head(r: &mut R, i: usize, fur: Color, light: Color, t: f32, talk: bool, happy: f32) {
    let (hr, ey, ez) = match i {
        2 => (vec3(0.3, 0.2, 0.26), 0.19, 0.13),
        10 => (vec3(0.2, 0.19, 0.21), 0.02, 0.17),
        _ => (vec3(0.26, 0.24, 0.25), 0.03, 0.2),
    };
    r.sphere(Vec3::ZERO, hr, fur);
    match i {
        // campground visitors
        14 => {
            for sx in [-1f32, 1.] {
                r.with(Mat4::from_translation(vec3(sx * 0.1, 0.3, -0.03)) * Mat4::from_rotation_z(sx * -0.15), |r| {
                    r.sphere(Vec3::ZERO, vec3(0.06, 0.2, 0.045), fur);
                    r.sphere(vec3(0., 0., 0.03), vec3(0.035, 0.15, 0.02), Color::new(0.95, 0.75, 0.75, 1.))
                })
            }
            r.sphere(vec3(0., -0.06, 0.22), Vec3::splat(0.035), Color::new(0.9, 0.5, 0.55, 1.))
        }
        15 => {
            r.sphere(vec3(0., -0.07, 0.2), vec3(0.12, 0.07, 0.08), light);
            for k in 0..3 {
                r.sphere(vec3(-0.12 + k as f32 * 0.12, 0.17, -0.05), Vec3::splat(0.035), sh(fur, 0.7))
            }
        }
        16 => {
            for sx in [-1f32, 1.] {
                r.with(
                    Mat4::from_translation(vec3(sx * 0.1, 0.22, -0.05))
                        * Mat4::from_rotation_x(-0.5)
                        * Mat4::from_rotation_z(sx * -0.3),
                    |r| r.cone(Vec3::ZERO, 0.04, 0.2, Color::new(0.75, 0.68, 0.55, 1.)),
                );
                r.sphere(vec3(sx * 0.25, 0.02, 0.), vec3(0.09, 0.04, 0.05), fur)
            }
            r.sphere(vec3(0., -0.2, 0.15), vec3(0.04, 0.07, 0.03), light)
        }
        17 => {
            r.sphere(vec3(0., -0.04, 0.24), vec3(0.07, 0.055, 0.06), Color::new(0.95, 0.6, 0.65, 1.));
            for sx in [-1f32, 1.] {
                r.sphere(vec3(sx * 0.18, 0.12, -0.02), Vec3::splat(0.04), sh(fur, 0.8))
            }
            r.cyl(vec3(0., 0.22, 0.), 0.2, 0.03, Color::new(0.75, 0.55, 0.25, 1.));
            r.cyl(vec3(0., 0.24, 0.), 0.13, 0.12, Color::new(0.75, 0.55, 0.25, 1.))
        }
        _ => {}
    }
    match i {
        0 => {
            r.sphere(vec3(0., -0.07, 0.18), vec3(0.11, 0.08, 0.14), light);
            r.sphere(vec3(0., -0.04, 0.32), Vec3::splat(0.035), DARK);
            for s in [-1f32, 1.] {
                r.cone(vec3(s * 0.14, 0.16, -0.02), 0.08, 0.2, fur);
                r.sphere(vec3(s * 0.13, -0.08, 0.16), vec3(0.06, 0.04, 0.05), light)
            }
        }
        1 => {
            r.cube(vec3(0., 0.13, 0.1), vec3(0.05, 0.11, 0.15), light);
            for s in [-1f32, 1.] {
                r.sphere(vec3(s * 0.09, 0.02, 0.17), vec3(0.075, 0.09, 0.06), DARK);
                r.sphere(vec3(s * 0.17, 0.17, -0.02), Vec3::splat(0.06), fur)
            }
            r.sphere(vec3(0., -0.08, 0.2), vec3(0.08, 0.06, 0.07), light);
            r.sphere(vec3(0., -0.06, 0.27), Vec3::splat(0.03), DARK)
        }
        2 => {
            for s in [-1f32, 1.] {
                r.sphere(vec3(s * 0.13, 0.16, 0.08), Vec3::splat(0.095), fur);
                r.sphere(vec3(s * 0.2, -0.06, 0.15), vec3(0.05, 0.03, 0.03), Color::new(1., 0.6, 0.6, 1.))
            }
        }
        4 => {
            for (x, y, z) in [
                (0., 0.2, -0.02),
                (-0.15, 0.14, -0.04),
                (0.15, 0.14, -0.04),
                (-0.08, 0.24, 0.08),
                (0.08, 0.24, 0.08),
                (0., 0.12, -0.18),
            ] {
                r.sphere(vec3(x, y, z), Vec3::splat(0.12), WHITE)
            }
            for s in [-1f32, 1.] {
                r.sphere(vec3(s * 0.25, 0.02, 0.), vec3(0.08, 0.04, 0.05), fur)
            }
        }
        5 => {
            for k in 0..7 {
                let a = -1.2 + k as f32 * 0.4;
                r.with(
                    Mat4::from_translation(vec3(a.sin() * 0.2, 0.1 + a.cos() * 0.05, -0.12))
                        * Mat4::from_rotation_x(-1.9),
                    |r| r.cone(Vec3::ZERO, 0.07, 0.22, sh(fur, 0.6)),
                )
            }
            r.sphere(vec3(0., -0.06, 0.2), vec3(0.08, 0.06, 0.1), light);
            r.sphere(vec3(0., -0.04, 0.29), Vec3::splat(0.03), DARK)
        }
        12 => {
            r.with(Mat4::from_translation(vec3(0., -0.06, 0.2)) * Mat4::from_rotation_x(FRAC_PI_2 * 0.92), |r| {
                r.cone(Vec3::ZERO, 0.09, 0.42, Color::new(0.98, 0.7, 0.25, 1.))
            });
            r.sphere(vec3(0., -0.13, 0.32), vec3(0.07, 0.06, 0.12), Color::new(0.98, 0.8, 0.5, 1.));
            r.cube(vec3(0., 0.22, 0.), vec3(0.2, 0.04, 0.2), Color::new(0.2, 0.25, 0.45, 1.))
        }
        11 => {
            r.with(Mat4::from_translation(vec3(0., -0.03, 0.2)) * Mat4::from_rotation_x(FRAC_PI_2), |r| {
                r.cone(Vec3::ZERO, 0.055, 0.3, GOLD)
            });
            r.sphere(vec3(0., 0.2, -0.12), vec3(0.04, 0.12, 0.04), DARKGRAY)
        }
        3 => {
            r.sphere(vec3(0., -0.07, 0.17), vec3(0.12, 0.08, 0.1), light);
            r.sphere(vec3(0., -0.04, 0.27), Vec3::splat(0.035), DARK);
            for s in [-1f32, 1.] {
                r.sphere(vec3(s * 0.18, 0.13, -0.02), Vec3::splat(0.05), fur)
            }
        }
        _ => {}
    }
    for s in [-1f32, 1.] {
        let ex = s * if i == 2 { 0.13 } else { 0.085 };
        if happy > 0. {
            r.cube(vec3(ex, ey, ez + 0.06), vec3(0.035, 0.009, 0.012), if i == 1 { WHITE } else { DARK })
        } else {
            let bl = if (t + i as f32 * 1.3) % 4.6 < 0.13 { 0.15 } else { 1. };
            r.sphere(vec3(ex, ey, ez + 0.05), vec3(0.03, 0.042 * bl, 0.02), if i == 1 { WHITE } else { DARK });
            r.sphere(vec3(ex + 0.01, ey + 0.012, ez + 0.068), Vec3::splat(0.011), if i == 1 { DARK } else { WHITE })
        }
    }
    let open = if talk { (t * 12.).sin().abs() * 0.028 } else { 0. } + 0.008;
    let (my, mz, mw) = match i {
        0 => (-0.14, 0.28, 0.035),
        2 => (-0.05, 0.255, 0.14),
        10 => (-0.08, 0.2, 0.03),
        3 => (-0.13, 0.25, 0.035),
        _ => (-0.13, 0.24, 0.035),
    };
    r.cube(vec3(0., my, mz), vec3(mw, open, 0.012), Color::new(0.45, 0.15, 0.15, 1.));
}
