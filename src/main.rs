mod clock;
mod data;
mod render;
mod save;
#[cfg(test)]
mod tests;
mod world;

use chrono::{Local, Timelike};
use clock::Clock;
use data::*;
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use save::SAVE;
use world::*;

fn conf() -> Conf { Conf { window_title: "Hollowbrook".into(), window_width: 1280, window_height: 720, sample_count: 4, ..Default::default() } }

/// Tiny procedural sound effects (no external assets).
struct Sounds(Vec<(Snd, Sound)>);
fn wav(s: &[f32]) -> Vec<u8> {
    let (sr, n) = (22050u32, s.len() as u32); let mut b = Vec::with_capacity(44 + s.len() * 2);
    b.extend(b"RIFF"); b.extend((36 + n * 2).to_le_bytes()); b.extend(b"WAVEfmt "); b.extend(16u32.to_le_bytes()); b.extend(1u16.to_le_bytes()); b.extend(1u16.to_le_bytes());
    b.extend(sr.to_le_bytes()); b.extend((sr * 2).to_le_bytes()); b.extend(2u16.to_le_bytes()); b.extend(16u16.to_le_bytes()); b.extend(b"data"); b.extend((n * 2).to_le_bytes());
    for v in s { b.extend(((v.clamp(-1., 1.) * 32767.) as i16).to_le_bytes()) } b
}
fn synth(s: Snd) -> Vec<f32> {
    let sr = 22050.;
    let tone = |f: f32, d: f32, v: f32| (0..(sr * d) as usize).map(|i| { let t = i as f32 / sr; (t * f * std::f32::consts::TAU).sin() * v * (1. - t / d) }).collect::<Vec<f32>>();
    let noise = |d: f32, v: f32| (0..(sr * d) as usize).map(|i| { let t = i as f32 / sr; rand::gen_range(-1., 1.) * v * (1. - t / d).powi(2) }).collect::<Vec<f32>>();
    match s { Snd::Step => noise(0.05, 0.25), Snd::Blip => tone(880., 0.07, 0.35), Snd::Splash => noise(0.28, 0.4), Snd::Coin => [tone(988., 0.08, 0.3), tone(1319., 0.16, 0.3)].concat(),
        Snd::Dig => noise(0.12, 0.5), Snd::Rustle => noise(0.22, 0.2), Snd::Door => [tone(330., 0.08, 0.3), tone(262., 0.12, 0.3)].concat() }
}
impl Sounds {
    async fn load() -> Sounds { let mut v = vec![]; for s in [Snd::Step, Snd::Blip, Snd::Splash, Snd::Coin, Snd::Dig, Snd::Rustle, Snd::Door] { if let Ok(x) = load_sound_from_bytes(&wav(&synth(s))).await { v.push((s, x)) } } Sounds(v) }
    fn play(&self, q: &mut Vec<Snd>) { for s in q.drain(..) { if let Some((_, x)) = self.0.iter().find(|e| e.0 == s) { play_sound(x, PlaySoundParams { looped: false, volume: if s == Snd::Step { 0.2 } else { 0.45 } }) } } }
}

fn load_game() -> Option<Game> {
    if let Some(w) = save::load(SAVE) { return Some(Game::from_world(w, town())) }
    let map = town(); let w = save::migrate(save::LEGACY, &map)?;
    eprintln!("[save] migrated legacy save {} -> {SAVE} (legacy file kept)", save::LEGACY); save::save(&w, SAVE); Some(Game::from_world(w, map))
}

#[macroquad::main(conf)]
async fn main() {
    let clock = Clock::from_env(); let dbg = clock.dev.is_some() || std::env::var("HOLLOWBROOK_DEBUG").is_ok();
    rand::srand(Local::now().timestamp() as u64);
    let sounds = Sounds::load().await;
    let auto = std::env::var("HOLLOWBROOK_AUTOTEST").ok();
    let mut g = match load_game() {
        Some(g) => g,
        None if auto.is_some() => Game::new("Tester".into(), clock.now()),
        None => {
            let mut name = String::new();
            loop {
                while let Some(c) = get_char_pressed() { if (c.is_alphanumeric() || c == ' ') && name.len() < 12 { name.push(c) } }
                if is_key_pressed(KeyCode::Backspace) { name.pop(); }
                if is_key_pressed(KeyCode::Enter) && !name.trim().is_empty() { break }
                clear_background(Color::new(0.3, 0.55, 0.35, 1.));
                draw_text("Hollowbrook", 80., 160., 80., WHITE); draw_text("A little town by the river is waiting for a new neighbor.", 80., 230., 30., WHITE);
                draw_text(&format!("What's your name?  {}_", name), 80., 320., 40., GOLD); draw_text("Type, then press Enter", 80., 370., 24., LIGHTGRAY);
                next_frame().await
            }
            Game::new(name.trim().to_string(), clock.now())
        }
    };
    g.curve = std::env::var("HOLLOWBROOK_CURVE").ok().and_then(|s| s.parse().ok()).unwrap_or(0.012); // dev setting
    if g.catch_up(clock.now()) { save::save(&g.persist(), SAVE) }
    prevent_quit();
    if let Some(dir) = auto { autotest(&mut g, &clock, &dir).await; save::save(&g.persist(), SAVE); return }
    let (mut st, mut shots) = (0., 0);
    loop {
        let dt = get_frame_time().min(0.05); let now = clock.now(); let inp = Inp { down: get_keys_down(), pressed: get_keys_pressed() };
        if g.update(&inp, dt, now) { save::save(&g.persist(), SAVE) } // persist a refresh immediately
        render::draw(&g, now, dbg.then(|| clock.source()));
        sounds.play(&mut g.snd);
        if dbg && is_key_pressed(KeyCode::F12) { shots += 1; get_screen_data().export_png(&format!("shot_{shots}.png")); eprintln!("[shot] shot_{shots}.png") }
        st += dt; if st > 15. { save::save(&g.persist(), SAVE); st = 0. }
        if is_quit_requested() { save::save(&g.persist(), SAVE); break }
        next_frame().await
    }
}

// ---- dev-only end-to-end bot: injects key events into the same Inp -> Game::update path the keyboard uses ----
async fn step(g: &mut Game, c: &Clock, down: &[KeyCode], pressed: &[KeyCode]) { let now = c.now(); if g.update(&Inp::keys(down, pressed), 1. / 60., now) { save::save(&g.persist(), SAVE) } render::draw(g, now, Some(c.source())); g.snd.clear(); next_frame().await }
async fn press(g: &mut Game, c: &Clock, k: KeyCode) { step(g, c, &[k], &[k]).await; step(g, c, &[], &[]).await }
async fn wait(g: &mut Game, c: &Clock, n: usize) { for _ in 0..n { step(g, c, &[], &[]).await } }
fn keys_toward(d: Vec2) -> Vec<KeyCode> { let mut k = vec![]; if d.y < -0.06 { k.push(KeyCode::W) } if d.y > 0.06 { k.push(KeyCode::S) } if d.x < -0.06 { k.push(KeyCode::A) } if d.x > 0.06 { k.push(KeyCode::D) } k }
async fn go(g: &mut Game, c: &Clock, to: Vec2, tol: f32) -> bool {
    for _ in 0..2400 { let p = g.ppos(); if p.distance(to) < tol { return true }
        let path = g.bfs(tile_of(p), tile_of(to), false).unwrap_or_default(); let wp = if path.len() > 1 { tc(path[0]) } else { to };
        step(g, c, &keys_toward(wp - p), &[]).await }
    false
}
async fn face(g: &mut Game, c: &Clock, at: Vec2) { let d = at - g.ppos(); let k = if d.x.abs() > d.y.abs() { if d.x > 0. { KeyCode::D } else { KeyCode::A } } else if d.y > 0. { KeyCode::S } else { KeyCode::W }; step(g, c, &[k], &[]).await; step(g, c, &[], &[]).await }
/// Walk next to `at` (a free neighbouring spot) and face it.
async fn approach(g: &mut Game, c: &Clock, at: Vec2) -> bool {
    let mut spots: Vec<Vec2> = [(0., 0.8), (0., -0.8), (0.8, 0.), (-0.8, 0.)].iter().map(|&(x, z)| at + vec2(x, z)).filter(|&s| g.free(s)).collect();
    spots.sort_by(|a, b| a.distance(g.ppos()).total_cmp(&b.distance(g.ppos())));
    for s in spots { if go(g, c, s, 0.12).await { face(g, c, at).await; return true } } false
}
async fn autotest(g: &mut Game, c: &Clock, dir: &str) {
    let phase = std::env::var("HOLLOWBROOK_AUTOTEST_PHASE").unwrap_or("1".into());
    let mut rep = String::new();
    macro_rules! log { ($($a:tt)*) => {{ let s = format!($($a)*); eprintln!("[auto] {s}"); rep += &s; rep.push('\n'); }} }
    macro_rules! shot { ($n:expr) => {{ wait(g, c, 2).await; render::draw(g, c.now(), Some(c.source())); get_screen_data().export_png(&format!("{dir}/{}.png", $n)); next_frame().await; log!("shot {}", $n); }} }
    let state = |g: &Game| format!("money={} inv={:?} stock={:?} room={:?} tools={:?}", g.w.money, g.w.inv.iter().map(|s| (def(s.def).name, s.qty, s.uid)).collect::<Vec<_>>(),
        g.w.stock.iter().map(|s| (def(s.def).name, s.sold)).collect::<Vec<_>>(), g.w.room.iter().map(|f| (def(f.def).name, f.uid, f.x, f.z, f.rot, f.on)).collect::<Vec<_>>(), g.w.tools);
    log!("phase {phase} start {} | {}", c.now(), state(g));
    wait(g, c, 30).await;
    if phase == "1" {
        shot!("01_outdoor_start");
        go(g, c, vec2(13.5, 11.4), 0.15).await; face(g, c, vec2(13.5, 13.)).await; shot!("02_behind_house_occlusion");
        go(g, c, vec2(16.5, 23.4), 0.2).await; face(g, c, vec2(16.5, 10.)).await; shot!("03_beach_rolling_horizon");
        let shell = g.w.ground.iter().filter(|i| def(i.def).k == K::Shell).min_by(|a, b| vec2(a.x, a.z).distance(g.pos).total_cmp(&vec2(b.x, b.z).distance(g.pos))).map(|i| vec2(i.x, i.z));
        if let Some(s) = shell { approach(g, c, s).await; press(g, c, KeyCode::E).await; } log!("after shell: inv={:?}", g.w.inv);
        if let Some(i) = (0..4).find(|&i| !g.npcs[i].inside) {
            for _ in 0..900 { let np = g.npcs[i].pos; if g.pos.distance(np) < 1.1 { break } let path = g.bfs(tile_of(g.pos), tile_of(np), false).unwrap_or_default(); let wp = if path.len() > 1 { tc(path[0]) } else { np }; step(g, c, &keys_toward(wp - g.pos), &[]).await }
            face(g, c, g.npcs[i].pos).await; press(g, c, KeyCode::E).await; log!("talk to resident {i}: ui={:?}", g.ui);
            if matches!(g.ui, Ui::Talk { .. }) { shot!("04_resident_conversation") } for _ in 0..4 { if g.ui == Ui::Play { break } press(g, c, KeyCode::E).await }
        }
        if let Some(ti) = (0..g.w.trees.len()).filter(|&i| g.w.trees[i].fruit).min_by(|&a, &b| tc((g.w.trees[a].x, g.w.trees[a].z)).distance(g.pos).total_cmp(&tc((g.w.trees[b].x, g.w.trees[b].z)).distance(g.pos))) {
            let tp = tc((g.w.trees[ti].x, g.w.trees[ti].z)); approach(g, c, tp).await; press(g, c, KeyCode::E).await; log!("shook tree {ti}: fruit now {} ground fruit {}", g.w.trees[ti].fruit, g.w.ground.iter().filter(|i| i.def == g.w.native).count());
            for _ in 0..3 { let Some(f) = g.w.ground.iter().filter(|i| i.def == g.w.native).map(|i| vec2(i.x, i.z)).min_by(|a, b| a.distance(g.pos).total_cmp(&b.distance(g.pos))) else { break }; approach(g, c, f).await; press(g, c, KeyCode::E).await }
            log!("after fruit: inv={:?}", g.w.inv);
        }
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; log!("scene after shop door: {:?}", g.scene); shot!("05_shop_interior");
        go(g, c, vec2(3.5, 2.5), 0.12).await; face(g, c, vec2(3.5, 1.5)).await; press(g, c, KeyCode::E).await; log!("counter ui={:?}", g.ui); shot!("06_sell_menu");
        let m0 = g.w.money; press(g, c, KeyCode::A).await; log!("sold all finds: money {m0} -> {}", g.w.money); press(g, c, KeyCode::Escape).await;
        if let Some(d) = (0..5).filter(|&i| !g.w.stock[i].sold && def(g.w.stock[i].def).buy <= g.w.money && !def(g.w.stock[i].def).wall).max_by_key(|&i| { let d = def(g.w.stock[i].def); ((d.act == Act::Sit) as u32 * 4 + d.rot as u32 * 2 + (d.act != Act::None) as u32, u32::MAX - d.buy) }) {
            approach(g, c, tc(DISPLAYS[d])).await; press(g, c, KeyCode::E).await; log!("display {d} ui={:?}", g.ui); let m1 = g.w.money; press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::Enter).await;
            log!("bought {}: money {m1} -> {}, sold={}", def(g.w.stock[d].def).name, g.w.money, g.w.stock[d].sold); shot!("07_shop_after_purchase");
        }
        go(g, c, vec2(3.5, 4.5), 0.12).await; for _ in 0..30 { if g.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await; log!("left shop: scene={:?} pos={:?}", g.scene, g.pos);
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; log!("home: scene={:?}", g.scene);
        if let Some(idx) = g.w.inv.iter().position(|s| def(s.def).k == K::Furn) {
            go(g, c, vec2(2.5, 3.5), 0.1).await; face(g, c, vec2(2.5, 1.)).await;
            press(g, c, KeyCode::I).await; for _ in 0..idx { press(g, c, KeyCode::S).await } press(g, c, KeyCode::Enter).await; log!("placing={:?}", g.placing.as_ref().map(|p| p.f));
            press(g, c, KeyCode::A).await; press(g, c, KeyCode::W).await; if g.placing.as_ref().is_some_and(|p| def(p.f.def).rot) { press(g, c, KeyCode::R).await }
            shot!("08_placing_ghost"); press(g, c, KeyCode::Enter).await;
            if g.placing.is_some() { press(g, c, KeyCode::D).await; press(g, c, KeyCode::Enter).await }
            log!("after place: placing={} room={:?} inv={:?}", g.placing.is_some(), g.w.room, g.w.inv);
            if let Some(f) = g.w.room.last().copied() { let cpos = fcenter(&f); approach(g, c, cpos).await; let tg = g.target(); press(g, c, KeyCode::E).await; wait(g, c, 20).await;
                log!("used furniture target={:?} pose={:?} ui={:?} lamp_on={:?}", tg, g.pose, g.ui, g.w.room.iter().find(|x| x.uid == f.uid).map(|x| x.on)); shot!("09_furniture_in_use");
                if g.ui != Ui::Play { press(g, c, KeyCode::Escape).await } if g.pose != Pose::Stand { step(g, c, &[KeyCode::S], &[]).await } log!("stood up: pose={:?}", g.pose); go(g, c, vec2(3.4, 3.3), 0.15).await; face(g, c, vec2(3.4, 4.)).await; shot!("10_furnished_home"); }
        }
        log!("phase 1 end | {}", state(g));
    } else {
        shot!("11_relaunch_outdoor");
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; log!("home: scene={:?}", g.scene); shot!("12_relaunch_home");
        go(g, c, vec2(2.5, 3.5), 0.12).await; for _ in 0..30 { if g.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await;
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; go(g, c, vec2(3.5, 2.6), 0.15).await; shot!("13_relaunch_shop_soldout");
        log!("phase 2 end | {}", state(g));
    }
    log!("end time {} hr {}", c.now(), c.now().hour());
    let _ = std::fs::write(format!("{dir}/autotest_phase{phase}.txt"), rep);
}
