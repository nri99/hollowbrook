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
