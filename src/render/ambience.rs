//! Hollowbrook ambience (render only): grass tufts that sway in the wind, puddles on paths while it rains,
//! and butterflies on fair days from spring to autumn.
use super::*;

pub(super) fn town_ambience(r: &mut R, g: &Game, lt: &Light, now: NaiveDateTime) {
    let t = g.t;
    let (cx, cz) = (g.me.cam.x.floor() as i32, g.me.cam.y.floor() as i32);
    let wx = weather(g.w.day, now.month());
    let grass = season(now.month()).0;
    let blade = sh(grass, 0.78);
    for z in cz - 11..cz + 9 {
        for x in cx - 13..cx + 14 {
            let c = tile_at(&g.map, x, z);
            let k = hash(x * 7 + 3, z * 5 + 11);
            if c == GRASS && k % 4 == 0 {
                // a small tuft of three blades, leaning with a gentle gust that rolls across town
                let p = vec3(x as f32 + 0.2 + (k % 7) as f32 * 0.09, 0., z as f32 + 0.25 + (k / 7 % 7) as f32 * 0.08);
                let gust = (t * 1.5 + x as f32 * 0.35 + z as f32 * 0.2).sin() * 0.06 + 0.03;
                for j in 0..3 {
                    let a = j as f32 * 2.1 + (k % 5) as f32;
                    let base = p + vec3(a.cos() * 0.04, 0., a.sin() * 0.04);
                    let h = 0.13 + (j as f32) * 0.03;
                    r.seg(base, base + vec3(gust + a.cos() * 0.03, h, a.sin() * 0.03), 0.012, blade);
                }
            } else if c == PATH && wx == Wx::Rain && k % 6 == 0 {
                // a puddle that mirrors the sky, with rain rings spreading on it
                let p = vec3(x as f32 + 0.5, 0.008, z as f32 + 0.5);
                let sky = Color::new(lt.sky.r * 0.75, lt.sky.g * 0.8, lt.sky.b * 0.9, 0.7);
                r.disc(p, vec2(0.36, 0.22), sky);
                let ring = (t * 1.2 + (k % 10) as f32 * 0.1) % 1.;
                r.disc(
                    p + vec3(0.08, 0.002, 0.03),
                    vec2(0.04 + ring * 0.14, 0.03 + ring * 0.09),
                    Color::new(1., 1., 1., 0.25 * (1. - ring)),
                );
            }
        }
    }
    // butterflies drifting about on fair days, spring to early autumn
    let day = lt.night < 0.3;
    if day && wx == Wx::Clear && (3..=10).contains(&now.month()) {
        let cols = [Color::new(1., 0.85, 0.3, 1.), Color::new(0.95, 0.95, 1., 1.), Color::new(0.55, 0.75, 1., 1.)];
        for k in 0..5 {
            let kf = k as f32;
            let p = vec3(
                g.me.cam.x + (t * 0.23 + kf * 2.4).sin() * 5. + (t * 0.9 + kf).sin() * 0.4,
                0.55 + (t * 2.7 + kf).sin().abs() * 0.35,
                g.me.cam.y - 1. + (t * 0.17 + kf * 1.3).cos() * 3.5,
            );
            let flap = (t * 18. + kf).sin() * 0.9;
            let col = cols[k % 3];
            for s in [-1f32, 1.] {
                let tip = vec3(s * 0.09 * flap.cos().abs().max(0.25), 0.06 * flap.abs(), 0.02);
                r.quad(p, p + tip, p + tip + vec3(0., 0., -0.07), p + vec3(0., 0., -0.05), Vec3::Y, col);
            }
            r.seg(p + vec3(0., 0., 0.02), p + vec3(0., 0., -0.06), 0.008, Color::new(0.15, 0.12, 0.1, 1.));
        }
    }
}

/// North Meadow details: lily pads and reeds on the lake, a little jetty, a bench overlooking the water,
/// a signpost, and wildflower clumps.
pub(super) fn meadow(r: &mut R, g: &Game, labels: &mut Vec<(Vec3, String, Color)>) {
    let cam = g.me.cam;
    if cam.y > 6. {
        return; // too far south to see any of it
    }
    let t = g.t;
    let (cx, cz, rx, rz) = MEADOW_LAKE;
    for k in 0..14 {
        let a = k as f32 * 2.39;
        let rr = 0.35 + (k % 5) as f32 * 0.13;
        let p = vec3(cx + a.cos() * rx * rr, -0.17, cz + a.sin() * rz * rr);
        let bob = (t * 0.8 + k as f32).sin() * 0.01;
        r.disc(p + vec3(0., bob, 0.), vec2(0.2, 0.17), Color::new(0.3, 0.6, 0.3, 1.));
        if k % 4 == 0 {
            r.sphere(p + vec3(0.05, 0.05, 0.), Vec3::splat(0.06), Color::new(1., 0.75, 0.85, 1.));
            // water lily
        }
    }
    for k in 0..18 {
        let a = k as f32 * 0.349 + 0.2;
        let p = vec3(cx + a.cos() * (rx + 0.1), 0., cz + a.sin() * (rz + 0.1));
        if (24.0..26.0).contains(&p.x) && p.z > cz {
            continue; // keep the river mouth open
        }
        let sway = (t * 1.3 + k as f32).sin() * 0.04;
        for j in 0..3 {
            let o = vec3((j as f32 - 1.) * 0.07, 0., (j % 2) as f32 * 0.05);
            r.seg(p + o, p + o + vec3(sway, 0.55 + (j % 2) as f32 * 0.15, 0.), 0.014, Color::new(0.35, 0.52, 0.25, 1.));
        }
        r.cube(p + vec3(sway, 0.62, 0.), vec3(0.025, 0.07, 0.025), Color::new(0.45, 0.3, 0.18, 1.));
        // cattail head
    }
    // jetty on the west shore and a bench looking over the lake
    let wood = Color::new(0.6, 0.44, 0.28, 1.);
    let jx = cx - rx + 0.3;
    for k in 0..4 {
        r.cube(vec3(jx + k as f32 * 0.42, 0.05, cz), vec3(0.2, 0.03, 0.42), sh(wood, 0.9 + (k % 2) as f32 * 0.1));
        r.cube(vec3(jx + k as f32 * 0.42, -0.1, cz + 0.38), vec3(0.04, 0.16, 0.04), sh(wood, 0.7));
    }
    let b = vec3(21.5, 0., -11.3);
    r.cube(b + vec3(0., 0.26, 0.), vec3(0.42, 0.03, 0.15), wood);
    r.cube(b + vec3(0., 0.42, -0.14), vec3(0.42, 0.1, 0.02), wood);
    for sx in [-0.36, 0.36] {
        r.cube(b + vec3(sx, 0.13, 0.), vec3(0.03, 0.13, 0.12), sh(wood, 0.75));
    }
    // signpost at the meadow entrance
    let s = vec3(19.3, 0., -1.2);
    r.cube(s + vec3(0., 0.45, 0.), vec3(0.03, 0.45, 0.03), sh(wood, 0.8));
    r.cube(s + vec3(0., 0.82, 0.02), vec3(0.3, 0.09, 0.02), wood);
    if g.me.pos.distance(vec2(s.x, s.z)) < 4. {
        labels.push((s + vec3(0., 1.2, 0.), "North Meadow".into(), Color::new(0.85, 1., 0.8, 1.)));
    }
    // wildflower clumps on the open grass
    let cols = [Color::new(1., 0.85, 0.3, 1.), Color::new(0.95, 0.5, 0.65, 1.), Color::new(0.65, 0.6, 1., 1.), WHITE];
    for z in -12..0 {
        for x in 4..37 {
            if tile_at(&g.map, x, z) != GRASS || hash(x * 5, z * 3) % 9 != 0 {
                continue;
            }
            let p = vec3(x as f32 + 0.5, 0., z as f32 + 0.5);
            if p.distance(vec3(cam.x, 0., cam.y)) > 16. {
                continue;
            }
            for j in 0..4 {
                let a = j as f32 * 1.7 + x as f32;
                let q = p + vec3(a.cos() * 0.22, 0., a.sin() * 0.18);
                r.seg(q, q + vec3(0., 0.16, 0.), 0.01, Color::new(0.3, 0.55, 0.25, 1.));
                r.sphere(q + vec3(0., 0.18, 0.), Vec3::splat(0.035), cols[(hash(x, z) as usize + j) % 4]);
            }
        }
    }
}
