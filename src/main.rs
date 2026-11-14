mod audio;
mod clock;
mod data;
mod pad;
mod render;
mod save;
#[cfg(test)]
mod tests;
mod world;

use chrono::{Datelike, Local, Timelike};
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
    let lp = |v: Vec<f32>, k: f32| { let mut y = 0.; v.into_iter().map(|x| { y += (x - y) * k; y }).collect::<Vec<f32>>() }; // one-pole low-pass
    match s { Snd::StepPath => noise(0.06, 0.35), Snd::StepSand => lp(noise(0.09, 0.5), 0.25), Snd::StepWood => [tone(170., 0.06, 0.35), noise(0.02, 0.1)].concat(), Snd::StepFloor => tone(240., 0.045, 0.25), Snd::Swim => lp(noise(0.2, 0.35), 0.15),
        Snd::Step => lp(noise(0.06, 0.4), 0.35), Snd::Blip => tone(880., 0.07, 0.35), Snd::Splash => noise(0.28, 0.4), Snd::Coin => [tone(988., 0.08, 0.3), tone(1319., 0.16, 0.3)].concat(),
        Snd::Dig => noise(0.12, 0.5), Snd::Rustle => noise(0.22, 0.2), Snd::Door => [tone(330., 0.08, 0.3), tone(262., 0.12, 0.3)].concat() }
}
impl Sounds {
    async fn load() -> Sounds { let mut v = vec![]; for s in [Snd::StepPath, Snd::StepSand, Snd::StepWood, Snd::StepFloor, Snd::Swim, Snd::Step, Snd::Blip, Snd::Splash, Snd::Coin, Snd::Dig, Snd::Rustle, Snd::Door] { if let Ok(x) = load_sound_from_bytes(&wav(&synth(s))).await { v.push((s, x)) } } Sounds(v) }
    fn play(&self, q: &mut Vec<Snd>, vol: f32) { for s in q.drain(..) { if vol <= 0. { continue } if let Some((_, x)) = self.0.iter().find(|e| e.0 == s) { play_sound(x, PlaySoundParams { looped: false, volume: vol * if matches!(s, Snd::Step | Snd::StepPath | Snd::StepSand | Snd::StepWood | Snd::StepFloor | Snd::Swim) { 0.2 } else { 0.45 } }) } } }
}

fn load_game() -> Option<Game> {
    match save::load_checked(SAVE) {
        save::Loaded::Ok(w) => { save::backup(SAVE, save::BACKUPS); return Some(Game::from_world(w, town())) }
        save::Loaded::Corrupt(p) => eprintln!("[save] {SAVE} could not be read; moved aside to {p}"),
        save::Loaded::Missing => {}
    }
    let map = town(); save::backup(save::LEGACY, save::BACKUPS); let w = save::migrate(save::LEGACY, &map)?;
    eprintln!("[save] migrated legacy save {} -> {SAVE} (legacy file kept)", save::LEGACY); save::save(&w, SAVE); Some(Game::from_world(w, map))
}

#[macroquad::main(conf)]
async fn main() {
    let clock = Clock::from_env(); let dbg = clock.dev.is_some() || std::env::var("HOLLOWBROOK_DEBUG").is_ok();
    rand::srand(Local::now().timestamp() as u64);
    let sounds = Sounds::load().await;
    eprintln!("[audio] backend: quad-snd; {} sound effects loaded", sounds.0.len());
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
    { let st = audio::load_settings(audio::SETTINGS); g.music_vol = st.music; g.sfx_vol = st.sfx; g.muted = st.muted; }
    g.curve = std::env::var("HOLLOWBROOK_CURVE").ok().and_then(|s| s.parse().ok()).unwrap_or(0.012); // dev setting
    if g.catch_up(clock.now()) { save::save(&g.persist(), SAVE) }
    prevent_quit();
    if let Some(dir) = auto { autotest(&mut g, &clock, &dir).await; save::save(&g.persist(), SAVE); return }
    let mut music = audio::Music::new();
    let (mut st, mut shots) = (0., 0); let mut pads = pad::Pads::new(); eprintln!("[pad] {} gamepad(s) detected", pads.count());
    loop {
        let dt = get_frame_time().min(0.05); let now = clock.now(); let kb = Inp { down: get_keys_down(), pressed: get_keys_pressed() };
        if g.tick(dt, now) { save::save(&g.persist(), SAVE) } // persist a refresh immediately
        pads.poll();
        for slot in std::mem::take(&mut pads.claimed) { eprintln!("[pad] controller assigned to player {}", slot + 1); if slot == 1 && !g.ids().contains(&1) { g.join("Player 2"); g.note("Player 2 joined with a controller."); save::save(&g.persist(), SAVE) } }
        for id in g.ids() { g.activate(id); let p = pads.inp(id.min(1), dt); let used = !p.down.is_empty();
            let inp = if id == 0 { if !kb.down.is_empty() { g.me.pad = false } pad::merge(Inp { down: kb.down.clone(), pressed: kb.pressed.clone() }, p) } else { p };
            if used { g.me.pad = true } g.me.pad_lost = pads.slots.lost(id.min(1)); g.update_player(&inp, dt) }
        g.activate(0);
        if kb.p(KeyCode::F2) { if g.ids().len() == 1 { g.join("Player 2"); g.note("Player 2 joined (split-screen).") } else { g.leave(1); g.note("Player 2 left; their progress is saved.") } save::save(&g.persist(), SAVE) }
        render::draw(&mut g, now, dbg.then(|| clock.source()));
        sounds.play(&mut g.snd, g.sfx_vol);
        music.tick((now.hour(), audio::variant(&g)), dt, g.music_vol, g.muted).await; for l in music.log.drain(..) { eprintln!("[audio] {l}") }
        if g.settings_dirty { g.settings_dirty = false; audio::save_settings(audio::SETTINGS, &audio::Settings { music: g.music_vol, sfx: g.sfx_vol, muted: g.muted }) }
        if dbg && is_key_pressed(KeyCode::F12) { shots += 1; screenshot(&format!("shot_{shots}.png")); eprintln!("[shot] shot_{shots}.png") }
        st += dt; if st > 15. { save::save(&g.persist(), SAVE); st = 0. }
        if is_quit_requested() { save::save(&g.persist(), SAVE); break }
        next_frame().await
    }
}

// ---- dev-only end-to-end bot: injects key events into the same Inp -> Game::update path the keyboard uses ----
async fn step(g: &mut Game, c: &Clock, down: &[KeyCode], pressed: &[KeyCode]) { let now = c.now(); if g.update(&Inp::keys(down, pressed), 1. / 60., now) { save::save(&g.persist(), SAVE) } render::draw(g, now, Some(c.source())); g.snd.clear(); next_frame().await }
async fn step2(g: &mut Game, c: &Clock, i0: Inp, i1: Inp) { let now = c.now(); if g.tick(1. / 60., now) { save::save(&g.persist(), SAVE) } g.activate(0); g.update_player(&i0, 1. / 60.); g.activate(1); g.update_player(&i1, 1. / 60.); g.activate(0); render::draw(g, now, Some(c.source())); g.snd.clear(); next_frame().await }
async fn press(g: &mut Game, c: &Clock, k: KeyCode) { step(g, c, &[k], &[k]).await; step(g, c, &[], &[]).await }
async fn wait(g: &mut Game, c: &Clock, n: usize) { for _ in 0..n { step(g, c, &[], &[]).await } }
fn keys_toward(d: Vec2) -> Vec<KeyCode> { let mut k = vec![]; if d.y < -0.06 { k.push(KeyCode::W) } if d.y > 0.06 { k.push(KeyCode::S) } if d.x < -0.06 { k.push(KeyCode::A) } if d.x > 0.06 { k.push(KeyCode::D) } k }
async fn go(g: &mut Game, c: &Clock, to: Vec2, tol: f32) -> bool {
    let (mut last, mut still) = (g.ppos(), 0);
    for f in 0..2400 { let p = g.ppos(); if p.distance(to) < tol { return true }
        let cur = tile_of(p); let path = g.bfs(cur, tile_of(to), false).unwrap_or_default();
        let keys = if path.len() > 1 { let nx = path[0]; let cc = tc(cur);
            // keep the box in the lane before crossing into the next tile (avoids catching corners)
            if nx.0 != cur.0 && (p.y - cc.y).abs() > 0.1 { keys_toward(vec2(0., cc.y - p.y)) } else if nx.1 != cur.1 && (p.x - cc.x).abs() > 0.1 { keys_toward(vec2(cc.x - p.x, 0.)) } else { keys_toward(tc(nx) - p) }
        } else { keys_toward(to - p) };
        let keys = if still > 30 { still = 0; vec![[KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D][f % 4]] } else { keys };
        step(g, c, &keys, &[]).await;
        if g.ppos().distance(last) < 0.001 { still += 1 } else { still = 0 } last = g.ppos(); }
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
    macro_rules! shot { ($n:expr) => {{ wait(g, c, 2).await; render::draw(g, c.now(), Some(c.source())); screenshot(&format!("{dir}/{}.png", $n)); next_frame().await; log!("shot {}", $n); }} }
    let state = |g: &Game| format!("money={} inv={:?} stock={:?} room={:?} tools={:?}", g.w.p.money, g.w.p.inv.iter().map(|s| (def(s.def).name, s.qty, s.uid)).collect::<Vec<_>>(),
        g.w.stock.iter().map(|s| (def(s.def).name, s.sold)).collect::<Vec<_>>(), g.w.p.room.iter().map(|f| (def(f.def).name, f.uid, f.x, f.z, f.rot, f.on)).collect::<Vec<_>>(), g.w.p.tools);
    log!("phase {phase} start {} | {}", c.now(), state(g));
    wait(g, c, 30).await;
    if phase == "1" {
        shot!("01_outdoor_start");
        go(g, c, vec2(13.5, 11.4), 0.15).await; face(g, c, vec2(13.5, 13.)).await; shot!("02_behind_house_occlusion");
        go(g, c, vec2(16.5, 23.4), 0.2).await; face(g, c, vec2(16.5, 10.)).await; shot!("03_beach_rolling_horizon");
        let shell = g.w.ground.iter().filter(|i| def(i.def).k == K::Shell).min_by(|a, b| vec2(a.x, a.z).distance(g.me.pos).total_cmp(&vec2(b.x, b.z).distance(g.me.pos))).map(|i| vec2(i.x, i.z));
        if let Some(s) = shell { approach(g, c, s).await; press(g, c, KeyCode::E).await; } log!("after shell: inv={:?}", g.w.p.inv);
        if let Some(i) = (0..4).find(|&i| !g.npcs[i].inside) {
            for _ in 0..900 { let np = g.npcs[i].pos; if g.me.pos.distance(np) < 1.1 { break } let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default(); let wp = if path.len() > 1 { tc(path[0]) } else { np }; step(g, c, &keys_toward(wp - g.me.pos), &[]).await }
            face(g, c, g.npcs[i].pos).await; press(g, c, KeyCode::E).await; log!("talk to resident {i}: ui={:?}", g.me.ui);
            if matches!(g.me.ui, Ui::Talk { .. }) { shot!("04_resident_conversation") } for _ in 0..4 { if g.me.ui == Ui::Play { break } press(g, c, KeyCode::E).await }
        }
        if let Some(ti) = (0..g.w.trees.len()).filter(|&i| g.w.trees[i].fruit).min_by(|&a, &b| tc((g.w.trees[a].x, g.w.trees[a].z)).distance(g.me.pos).total_cmp(&tc((g.w.trees[b].x, g.w.trees[b].z)).distance(g.me.pos))) {
            let tp = tc((g.w.trees[ti].x, g.w.trees[ti].z)); approach(g, c, tp).await; press(g, c, KeyCode::E).await; log!("shook tree {ti}: fruit now {} ground fruit {}", g.w.trees[ti].fruit, g.w.ground.iter().filter(|i| i.def == g.w.native).count());
            for _ in 0..3 { let Some(f) = g.w.ground.iter().filter(|i| i.def == g.w.native).map(|i| vec2(i.x, i.z)).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos))) else { break }; approach(g, c, f).await; press(g, c, KeyCode::E).await }
            log!("after fruit: inv={:?}", g.w.p.inv);
        }
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; log!("scene after shop door: {:?}", g.me.scene); shot!("05_shop_interior");
        go(g, c, vec2(3.5, 2.5), 0.12).await; face(g, c, vec2(3.5, 1.5)).await; press(g, c, KeyCode::E).await; log!("counter ui={:?}", g.me.ui); shot!("06_sell_menu");
        let m0 = g.w.p.money; press(g, c, KeyCode::A).await; log!("sold all finds: money {m0} -> {}", g.w.p.money); press(g, c, KeyCode::Escape).await;
        if let Some(d) = (0..5).filter(|&i| !g.w.stock[i].sold && def(g.w.stock[i].def).buy <= g.w.p.money && !def(g.w.stock[i].def).wall).max_by_key(|&i| { let d = def(g.w.stock[i].def); ((d.act == Act::Sit) as u32 * 4 + d.rot as u32 * 2 + (d.act != Act::None) as u32, u32::MAX - d.buy) }) {
            approach(g, c, tc(DISPLAYS[d])).await; press(g, c, KeyCode::E).await; log!("display {d} ui={:?}", g.me.ui); let m1 = g.w.p.money; press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::Enter).await;
            log!("bought {}: money {m1} -> {}, sold={}", def(g.w.stock[d].def).name, g.w.p.money, g.w.stock[d].sold); shot!("07_shop_after_purchase");
        }
        go(g, c, vec2(3.5, 4.5), 0.12).await; for _ in 0..30 { if g.me.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await; log!("left shop: scene={:?} pos={:?}", g.me.scene, g.me.pos);
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; log!("home: scene={:?}", g.me.scene);
        if let Some(idx) = g.w.p.inv.iter().position(|s| def(s.def).k == K::Furn) {
            go(g, c, vec2(2.5, 3.5), 0.1).await; face(g, c, vec2(2.5, 1.)).await;
            press(g, c, KeyCode::I).await; for _ in 0..idx { press(g, c, KeyCode::S).await } press(g, c, KeyCode::Enter).await; log!("placing={:?}", g.me.placing.as_ref().map(|p| p.f));
            press(g, c, KeyCode::A).await; press(g, c, KeyCode::W).await; if g.me.placing.as_ref().is_some_and(|p| def(p.f.def).rot) { press(g, c, KeyCode::R).await }
            shot!("08_placing_ghost"); press(g, c, KeyCode::Enter).await;
            if g.me.placing.is_some() { press(g, c, KeyCode::D).await; press(g, c, KeyCode::Enter).await }
            log!("after place: placing={} room={:?} inv={:?}", g.me.placing.is_some(), g.w.p.room, g.w.p.inv);
            if let Some(f) = g.w.p.room.last().copied() { let cpos = fcenter(&f); approach(g, c, cpos).await; let tg = g.target(); press(g, c, KeyCode::E).await; wait(g, c, 20).await;
                log!("used furniture target={:?} pose={:?} ui={:?} lamp_on={:?}", tg, g.me.pose, g.me.ui, g.w.p.room.iter().find(|x| x.uid == f.uid).map(|x| x.on)); shot!("09_furniture_in_use");
                if g.me.ui != Ui::Play { press(g, c, KeyCode::Escape).await } if g.me.pose != Pose::Stand { step(g, c, &[KeyCode::S], &[]).await } log!("stood up: pose={:?}", g.me.pose); go(g, c, vec2(3.4, 3.3), 0.15).await; face(g, c, vec2(3.4, 4.)).await; shot!("10_furnished_home"); }
        }
        log!("phase 1 end | {}", state(g));
    } else if phase == "4" {
        // MP-02/03: two players with independent virtual inputs; one goes indoors, the other stays outside
        g.join("Player 2"); log!("joined: ids={:?}", g.ids());
        let k = |d: &[KeyCode], p: &[KeyCode]| Inp::keys(d, p);
        for _ in 0..20 { step2(g, c, k(&[], &[]), k(&[], &[])).await } shot!("mp_same_scene"); for _ in 0..45 { step2(g, c, k(&[], &[]), k(&[KeyCode::A], &[])).await } shot!("mp_together");
        step2(g, c, k(&[KeyCode::W], &[]), k(&[], &[])).await; step2(g, c, k(&[KeyCode::E], &[KeyCode::E]), k(&[], &[])).await;
        for _ in 0..50 { step2(g, c, k(&[], &[]), k(&[KeyCode::D, KeyCode::S], &[])).await }
        let st = |g: &mut Game, id: usize| { g.activate(id); let s = format!("P{} scene={:?} pos=({:.2},{:.2}) ipos=({:.2},{:.2}) ui={:?}", id + 1, g.me.scene, g.me.pos.x, g.me.pos.y, g.me.ipos.x, g.me.ipos.y, g.me.ui); g.activate(0); s };
        log!("{} | {}", st(g, 0), st(g, 1)); shot!("mp_split_home_and_town");
        step2(g, c, k(&[], &[]), k(&[KeyCode::I], &[KeyCode::I])).await; step2(g, c, k(&[], &[]), k(&[], &[])).await;
        log!("after P2 pressed I: {} | {}", st(g, 0), st(g, 1)); shot!("mp_p2_inventory");
        step2(g, c, k(&[KeyCode::Escape], &[KeyCode::Escape]), k(&[], &[])).await; log!("P1 Esc does not close P2 menu: {}", st(g, 1));
        step2(g, c, k(&[], &[]), k(&[KeyCode::Escape], &[KeyCode::Escape])).await; log!("P2 Esc: {}", st(g, 1));
    } else if phase == "5" {
        // MP-04: virtual gamepad states -> pad::Mapper -> the same update path; slot disconnect/reconnect
        g.join("Player 2"); let mut m = pad::Mapper::default(); let mut sl = pad::Slots::default(); sl.connect(5); log!("pad 5 claims slot {:?}", sl.claim(5));
        let ps = |b: &[pad::B], lx: f32, ly: f32| pad::PadState { b: b.iter().copied().collect(), lx, ly };
        for _ in 0..10 { let i = m.map(&ps(&[], 0., 0.), 1. / 60.); step2(g, c, Inp::none(), i).await }
        for _ in 0..3 { let i = m.map(&ps(&[], 0., 0.8), 1. / 60.); step2(g, c, Inp::none(), i).await } // stick up: face the door
        let i = m.map(&ps(&[pad::B::South], 0., 0.), 1. / 60.); step2(g, c, Inp::none(), i).await; for _ in 0..40 { let i = m.map(&ps(&[], 0., 0.), 1. / 60.); step2(g, c, Inp::none(), i).await }
        g.activate(1); g.me.pad = true; log!("P2 after A at own door: scene={:?}", g.me.scene); g.activate(0);
        let i = m.map(&ps(&[pad::B::North], 0., 0.), 1. / 60.); step2(g, c, Inp::none(), i).await; let i = m.map(&ps(&[], 0., 0.), 1. / 60.); step2(g, c, Inp::none(), i).await;
        g.activate(1); log!("P2 after Y: ui={:?}", g.me.ui); g.activate(0); shot!("mp_pad_prompts");
        let i = m.map(&ps(&[pad::B::East], 0., 0.), 1. / 60.); step2(g, c, Inp::none(), i).await; g.activate(1); log!("P2 after B: ui={:?}", g.me.ui); g.activate(0);
        sl.disconnect(5); g.activate(1); g.me.pad_lost = sl.lost(1); g.activate(0); log!("disconnect: lost={}", sl.lost(1)); shot!("mp_pad_disconnected");
        sl.connect(6); log!("pad 6 claims {:?}; lost={}", sl.claim(6), sl.lost(1)); g.activate(1); g.me.pad_lost = sl.lost(1); log!("P2 progress intact: scene={:?} money={}", g.me.scene, g.w.p.money); g.activate(0);
    } else if phase == "6" {
        // P5-02: buy a shirt at the rack, wear it from the pockets menu
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        approach(g, c, tc(RACK)).await; press(g, c, KeyCode::E).await; log!("rack ui={:?}", g.me.ui); press(g, c, KeyCode::S).await; shot!("p5_rack"); press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::Escape).await;
        log!("after buy: money={} inv={:?}", g.w.p.money, g.w.p.inv);
        let idx = g.w.p.inv.iter().position(|s| def(s.def).k == K::Cloth).unwrap_or(0); press(g, c, KeyCode::I).await; for _ in 0..idx { press(g, c, KeyCode::S).await } press(g, c, KeyCode::Enter).await;
        log!("wearing={:?} inv={:?}", g.w.p.shirt, g.w.p.inv); go(g, c, vec2(3.5, 2.6), 0.12).await; face(g, c, vec2(3.5, 4.)).await; shot!("p5_new_outfit");
    } else if phase == "7" {
        // P5-01: expand home at the shop counter (coins granted as a logged FIXTURE; the purchase path is the normal menu)
        g.w.p.money = 3000; log!("FIXTURE: money set to 3000");
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        go(g, c, vec2(3.5, 2.5), 0.12).await; face(g, c, vec2(3.5, 1.5)).await; press(g, c, KeyCode::E).await; for _ in 0..g.w.p.inv.len() { press(g, c, KeyCode::S).await }
        shot!("p51_counter_expand"); press(g, c, KeyCode::Enter).await; log!("after expand: level={} money={} dims={:?}", g.w.p.home, g.w.p.money, g.rdims(HOME)); press(g, c, KeyCode::Escape).await;
        go(g, c, vec2(3.5, 4.5), 0.12).await; for _ in 0..30 { if g.me.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await;
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; log!("home scene={:?} ipos={:?}", g.me.scene, g.me.ipos); shot!("p51_bigger_home");
    } else if phase == "8" {
        // P6-01: mailbox (one letter inserted as a logged FIXTURE; opening/claiming uses the normal interaction)
        g.w.p.mail.push(world::Letter { from: "Juniper".into(), text: "Thinking of you! Here's a little something.".into(), coins: 300, item: None, read: false, claimed: false }); log!("FIXTURE: letter with 300 coins placed in P1 mail");
        let mb = tc(mailbox_tile(0)); approach(g, c, mb).await; log!("target={:?}", g.target()); shot!("p61_mailbox_flag"); let m0 = g.w.p.money;
        press(g, c, KeyCode::E).await; log!("ui={:?}", g.me.ui); press(g, c, KeyCode::Enter).await; log!("money {m0} -> {}, letter={:?}", g.w.p.money, g.w.p.mail[0]); shot!("p61_mail_menu");
        press(g, c, KeyCode::Enter).await; log!("second open: money {}", g.w.p.money); press(g, c, KeyCode::Escape).await;
    } else if phase == "9" {
        // P7-01/P6-02: weather presentation near a resident who is out
        wait(g, c, 60).await;
        if let Some(i) = (0..4).find(|&i| !g.npcs[i].inside) { for _ in 0..900 { let np = g.npcs[i].pos; if g.me.pos.distance(np) < 2.2 { break } let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default(); let wp = if path.len() > 1 { tc(path[0]) } else { np }; step(g, c, &keys_toward(wp - g.me.pos), &[]).await } }
        log!("weather={:?} pos={:?}", weather(g.w.day, c.now().month()), g.me.pos); shot!("p7_weather");
    } else if phase == "10" {
        // P5-03: buy at a stand, then re-order the same item from the counter catalog
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        let d = (0..5).filter(|&i| !def(g.w.stock[i].def).wall).min_by_key(|&i| def(g.w.stock[i].def).buy).unwrap();
        approach(g, c, tc(DISPLAYS[d])).await; press(g, c, KeyCode::E).await; press(g, c, KeyCode::Enter).await; log!("bought {} money={}", def(g.w.stock[d].def).name, g.w.p.money);
        go(g, c, vec2(3.5, 2.5), 0.12).await; face(g, c, vec2(3.5, 1.5)).await; press(g, c, KeyCode::E).await; for _ in 0..g.w.p.inv.len() + 1 { press(g, c, KeyCode::S).await }
        press(g, c, KeyCode::Enter).await; log!("ui={:?} catalog={:?}", g.me.ui, g.w.p.catalog); shot!("p53_catalog"); press(g, c, KeyCode::Enter).await;
        log!("after order: money={} inv={:?}", g.w.p.money, g.w.p.inv.iter().map(|s| (def(s.def).name, s.uid)).collect::<Vec<_>>());
    } else if phase == "11" {
        // P6-03: fruit gathered normally, then given to Juniper through the talk -> G -> gift menu path
        if let Some(ti) = (0..g.w.trees.len()).filter(|&i| g.w.trees[i].fruit).min_by(|&a, &b| tc((g.w.trees[a].x, g.w.trees[a].z)).distance(g.me.pos).total_cmp(&tc((g.w.trees[b].x, g.w.trees[b].z)).distance(g.me.pos))) {
            approach(g, c, tc((g.w.trees[ti].x, g.w.trees[ti].z))).await; press(g, c, KeyCode::E).await;
            if let Some(f) = g.w.ground.iter().filter(|i| i.def == g.w.native).map(|i| vec2(i.x, i.z)).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos))) { approach(g, c, f).await; press(g, c, KeyCode::E).await } }
        log!("pockets: {:?}", g.w.p.inv);
        for _ in 0..1500 { let np = g.npcs[0].pos; if !g.npcs[0].inside && g.me.pos.distance(np) < 1.1 { break } let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default(); let wp = if path.len() > 1 { tc(path[0]) } else { np }; step(g, c, &keys_toward(wp - g.me.pos), &[]).await }
        face(g, c, g.npcs[0].pos).await; let f0 = g.w.p.npcs[0].friend; press(g, c, KeyCode::E).await; press(g, c, KeyCode::G).await; log!("ui={:?}", g.me.ui); press(g, c, KeyCode::Enter).await;
        log!("friendship {f0} -> {} gifted={} ui={:?}", g.w.p.npcs[0].friend, g.w.p.npcs[0].gifted, g.me.ui); shot!("p63_gift_reaction");
    } else if phase == "12" {
        // P7-02: Harvest Fair stall (dev clock inside the provisional window)
        approach(g, c, tc(STALL)).await; log!("fair={} target={:?}", g.fair(), g.target()); press(g, c, KeyCode::E).await; log!("prize: inv={:?} fair_year={}", g.w.p.inv, g.w.p.fair_year);
        press(g, c, KeyCode::E).await; log!("second try inv len={}", g.w.p.inv.len()); step(g, c, &[KeyCode::S], &[]).await; for _ in 0..12 { step(g, c, &[KeyCode::S], &[]).await } face(g, c, tc(STALL)).await; shot!("p72_fair");
    } else if phase == "13" {
        // C-01: walk off the beach, swim to a bubble, dive
        go(g, c, vec2(13.5, 25.4), 0.15).await; for _ in 0..40 { if g.swimming() { break } step(g, c, &[KeyCode::S], &[]).await } for _ in 0..15 { step(g, c, &[KeyCode::S], &[]).await }
        log!("swimming={} pos={:?}", g.swimming(), g.me.pos);
        for _ in 0..600 { if !g.bubbles.is_empty() { break } step(g, c, &[], &[]).await }
        if let Some(b) = g.bubbles.iter().map(|b| b.0).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos))) { for _ in 0..600 { if g.me.pos.distance(b) < 0.9 { break } let k = keys_toward(b - g.me.pos); step(g, c, &k, &[]).await } }
        shot!("c01_swimming"); press(g, c, KeyCode::Space).await; log!("after dive: inv={:?} bubbles={}", g.w.p.inv, g.bubbles.len());
    } else if phase == "14" {
        // C-02: resident visit while at home (friendship 5 set as a logged FIXTURE; the visit itself is the normal provisional trigger)
        for n in &mut g.w.p.npcs { n.friend = 5 } log!("FIXTURE: friendship 5 with all residents");
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        for _ in 0..3000 { if g.visit.is_some() { break } step(g, c, &[], &[]).await } log!("visit={:?}", g.visit);
        if let Some((_, b, _)) = g.visit { let s = g.visit_spot(b); approach(g, c, s).await; press(g, c, KeyCode::E).await; log!("talk ui={:?}", g.me.ui); shot!("c02_visit") }
    } else if phase == "15" {
        // R15: tool effects captured right after use (net swish, rod cast splash)
        for _ in 0..4 { if g.w.p.equip == Some(60) { break } press(g, c, KeyCode::Tab).await } go(g, c, vec2(11.5, 16.5), 0.12).await; face(g, c, vec2(11.5, 15.)).await;
        step(g, c, &[KeyCode::Space], &[KeyCode::Space]).await; log!("fx after net: {:?}", g.fx); shot!("fx_net_swish");
        for _ in 0..4 { if g.w.p.equip == Some(61) { break } press(g, c, KeyCode::Tab).await } go(g, c, vec2(23.5, 5.5), 0.12).await; face(g, c, vec2(25., 5.5)).await;
        step(g, c, &[KeyCode::Space], &[KeyCode::Space]).await; log!("fx after cast: {:?} fishing={}", g.fx, g.me.fish.is_some()); shot!("fx_cast_splash");
    } else if phase == "16" {
        // C-03b render check: reserve characters placed by a logged FIXTURE (the move workflow itself is covered headlessly)
        g.w.roster[0] = 4; g.w.roster[2] = 5; for k in [0, 2] { g.npcs[k].inside = true } log!("FIXTURE: roster {:?}", g.w.roster);
        wait(g, c, 90).await; for k in [0usize, 2] { for _ in 0..1500 { let np = g.npcs[k].pos; if !g.npcs[k].inside && g.me.pos.distance(np) < 1.6 { break } let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default(); let wp = if path.len() > 1 { tc(path[0]) } else { np }; step(g, c, &keys_toward(wp - g.me.pos), &[]).await }
            face(g, c, g.npcs[k].pos).await; press(g, c, KeyCode::E).await; log!("talk {}: {:?}", g.res(k).name, g.me.ui); shot!(format!("c03_{}", g.res(k).name)); for _ in 0..5 { if g.me.ui == Ui::Play { break } press(g, c, KeyCode::E).await } }
    } else if phase == "17" {
        // C-04: buy furniture, customize tint + wallpaper + flooring at the counter, place it at home (coins: logged FIXTURE)
        g.w.p.money = 3000; log!("FIXTURE: money 3000");
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        let d = (0..5).filter(|&i| !def(g.w.stock[i].def).wall).max_by_key(|&i| (def(g.w.stock[i].def).act == Act::Sit) as u8).unwrap();
        approach(g, c, tc(DISPLAYS[d])).await; press(g, c, KeyCode::E).await; press(g, c, KeyCode::Enter).await; log!("bought {}", def(g.w.stock[d].def).name);
        go(g, c, vec2(3.5, 2.5), 0.12).await; face(g, c, vec2(3.5, 1.5)).await; press(g, c, KeyCode::E).await; for _ in 0..g.w.p.inv.len() + 2 { press(g, c, KeyCode::S).await } press(g, c, KeyCode::Enter).await;
        log!("ui={:?}", g.me.ui); press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::S).await; press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::S).await; press(g, c, KeyCode::Enter).await; shot!("c04_customize_menu"); press(g, c, KeyCode::Escape).await;
        log!("tints={:?} wall={} floor={} money={}", g.w.p.tints, g.w.p.wall, g.w.p.floor, g.w.p.money);
        go(g, c, vec2(3.5, 4.5), 0.12).await; for _ in 0..30 { if g.me.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await;
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        go(g, c, vec2(2.5, 3.5), 0.1).await; face(g, c, vec2(2.5, 1.)).await; press(g, c, KeyCode::I).await; press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::A).await; press(g, c, KeyCode::W).await; press(g, c, KeyCode::Enter).await;
        log!("room={:?}", g.w.p.room); go(g, c, vec2(3.4, 3.3), 0.15).await; face(g, c, vec2(3.4, 4.)).await; shot!("c04_custom_home");
    } else if phase == "18" {
        // C-05: P1 buys and wears a hat and changes hairstyle at the rack; P2 joins (default look) for comparison
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        approach(g, c, tc(RACK)).await; press(g, c, KeyCode::E).await; for _ in 0..4 { press(g, c, KeyCode::S).await } press(g, c, KeyCode::Enter).await; // Straw Hat
        for _ in 0..3 { press(g, c, KeyCode::S).await } press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::Escape).await; // hairstyle x2
        let idx = g.w.p.inv.iter().position(|s| is_hat(s.def)).unwrap_or(0); press(g, c, KeyCode::I).await; for _ in 0..idx { press(g, c, KeyCode::S).await } press(g, c, KeyCode::Enter).await;
        log!("hat={:?} hair={} money={}", g.w.p.hat, g.w.p.hair, g.w.p.money);
        go(g, c, vec2(3.5, 4.5), 0.12).await; for _ in 0..30 { if g.me.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await;
        g.join("Player 2"); g.activate(1); g.me.pos = g.ids().iter().map(|_| door_stand(SHOP) + vec2(1., 0.)).next().unwrap(); g.me.cam = g.me.pos; g.activate(0); log!("P2 placed beside P1 (test positioning)");
        for _ in 0..10 { step2(g, c, Inp::none(), Inp::none()).await } face(g, c, door_stand(SHOP) + vec2(0., 2.)).await; shot!("c05_looks");
    } else if phase == "19" {
        // X-01: gather branches (trees) and stones (rocks + shovel) normally, build the proposed bench, sit on it
        log!("project={:?}", g.w.project);
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        approach(g, c, tc(DISPLAYS[5])).await; press(g, c, KeyCode::E).await; press(g, c, KeyCode::Enter).await; go(g, c, vec2(3.5, 4.5), 0.12).await; for _ in 0..30 { if g.me.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await;
        let have = |g: &Game, d: u16| g.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum::<u16>();
        for _ in 0..10 { if have(g, 80) >= 5 { break }
            let Some(ti) = (0..g.w.trees.len()).filter(|&i| !g.w.branches.contains(&(g.w.trees[i].x, g.w.trees[i].z))).min_by(|&a, &b| tc((g.w.trees[a].x, g.w.trees[a].z)).distance(g.me.pos).total_cmp(&tc((g.w.trees[b].x, g.w.trees[b].z)).distance(g.me.pos))) else { break };
            approach(g, c, tc((g.w.trees[ti].x, g.w.trees[ti].z))).await; press(g, c, KeyCode::E).await;
            while let Some(it) = g.w.ground.iter().filter(|i| i.def == 80).map(|i| vec2(i.x, i.z)).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos))) { if it.distance(g.me.pos) > 4. { break } approach(g, c, it).await; press(g, c, KeyCode::E).await; if g.w.ground.iter().any(|i| vec2(i.x, i.z) == it) { break } } }
        log!("branches={}", have(g, 80));
        for _ in 0..4 { if g.w.p.equip == Some(62) { break } press(g, c, KeyCode::Tab).await }
        let rocks: Vec<(i32, i32)> = (0..W * H).filter(|&i| g.map[i] == ROCK).map(|i| ((i % W) as i32, (i / W) as i32)).collect();
        for _ in 0..4 { if have(g, 81) >= 2 { break } let Some(&rk) = rocks.iter().filter(|r| !g.w.rocks_hit.contains(r)).min_by(|a, b| tc(**a).distance(g.me.pos).total_cmp(&tc(**b).distance(g.me.pos))) else { break }; approach(g, c, tc(rk)).await; press(g, c, KeyCode::Space).await }
        log!("stones={}", have(g, 81));
        if let Some(p) = g.w.project.clone() { approach(g, c, tc((p.x, p.z))).await; shot!("x01_site_before"); press(g, c, KeyCode::E).await; log!("after contribute: project={:?} built={:?} ui={:?}", g.w.project, g.w.built, g.me.ui); for _ in 0..4 { if g.me.ui == Ui::Play { break } press(g, c, KeyCode::E).await } }
        if let Some(b) = g.w.built.first().copied() { approach(g, c, tc((b.x, b.z))).await; press(g, c, KeyCode::E).await; log!("pose={:?}", g.me.pose); wait(g, c, 30).await; shot!("x01_bench_built") }
    } else if phase == "20" {
        // X-03: P1 sails to the islet (P2 stays in town), does today's task normally, gets Sea Glass, sails back
        g.join("Player 2");
        go(g, c, dock_stand(), 0.12).await; face(g, c, vec2(20.5, 26.)).await; log!("target={:?}", g.target()); press(g, c, KeyCode::E).await; log!("ui={:?}", g.me.ui); press(g, c, KeyCode::Enter).await; wait(g, c, 40).await;
        log!("P1 scene={:?} money={}", g.me.scene, g.w.p.money);
        let (td, n, label) = islet_task(g.w.day); log!("task: {label}");
        for _ in 0..12 { let have: u16 = g.w.p.inv.iter().filter(|s| s.def == td).map(|s| s.qty).sum(); if have >= n { break }
            let Some(it) = g.w.iground.iter().filter(|i| i.def == td).map(|i| vec2(i.x, i.z)).min_by(|a, b| a.distance(g.me.ipos).total_cmp(&b.distance(g.me.ipos))) else { break }; approach(g, c, it).await; press(g, c, KeyCode::E).await }
        log!("pockets={:?}", g.w.p.inv); for _ in 0..10 { step2(g, c, Inp::none(), Inp::none()).await } shot!("x03_split_islet_town");
        approach(g, c, vec2(PELL_I.0, PELL_I.1)).await; press(g, c, KeyCode::E).await; log!("pell: {:?}", g.me.ui); for _ in 0..4 { if !matches!(g.me.ui, Ui::Talk { .. }) { break } press(g, c, KeyCode::E).await }
        log!("sea glass={} ui={:?}", g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>(), g.me.ui); press(g, c, KeyCode::Enter).await; wait(g, c, 40).await; log!("back: scene={:?} pos={:?}", g.me.scene, g.me.pos);
    } else if phase == "21" {
        // X-05: wait for residents to start chatting on their own, walk over, capture the speech bubbles
        for _ in 0..7200 { if !g.chats.is_empty() { break } step(g, c, &[], &[]).await } log!("chats={:?}", g.chats);
        if let Some(&(a, _, _)) = g.chats.first() { for _ in 0..600 { let np = g.npcs[a].pos; if g.me.pos.distance(np) < 2.5 || g.chats.is_empty() { break } let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default(); let wp = if path.len() > 1 { tc(path[0]) } else { np }; step(g, c, &keys_toward(wp - g.me.pos), &[]).await } }
        log!("still chatting={:?}", g.chats); shot!("x05_resident_chat");
    } else if phase == "22" {
        // X-06: dialogue framing in split-screen (P1 talks while P2 is joined)
        g.join("Player 2"); let i = (0..4).find(|&i| !g.npcs[i].inside).unwrap_or(0);
        for _ in 0..1500 { let np = g.npcs[i].pos; if !g.npcs[i].inside && g.me.pos.distance(np) < 1.1 { break } let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default(); let wp = if path.len() > 1 { tc(path[0]) } else { np }; step2(g, c, Inp::keys(&keys_toward(wp - g.me.pos), &[]), Inp::none()).await }
        face(g, c, g.npcs[i].pos).await; step2(g, c, Inp::keys(&[KeyCode::E], &[KeyCode::E]), Inp::none()).await; step2(g, c, Inp::none(), Inp::none()).await; log!("P1 ui={:?}", g.me.ui); shot!("x06_split_dialogue");
    } else if phase == "23" {
        // audio: hourly crossfade (dev clock crosses 17:00), indoor arrangement switch, journal mute/volume
        let mut m = audio::Music::new(); let mut frame = 0usize;
        macro_rules! tickn { ($n:expr, $d:expr, $p:expr) => {{ for _ in 0..$n { m.tick((c.now().hour(), audio::variant(g)), 1. / 60., g.music_vol, g.muted).await; step(g, c, $d, $p).await; frame += 1;
            if frame % 60 == 0 { log!("t={} cur={:?} fading_out={} applied={:.2}", c.now().format("%H:%M:%S"), m.current(), m.fading_out(), m.applied) } } for l in m.log.drain(..) { log!("{l}") } }} }
        tickn!(600, &[], &[]);
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; tickn!(300, &[], &[]); log!("scene={:?} variant={:?}", g.me.scene, audio::variant(g));
        press(g, c, KeyCode::J).await; press(g, c, KeyCode::X).await; tickn!(10, &[], &[]); log!("after mute: muted={} applied={:.2}", g.muted, m.applied);
        press(g, c, KeyCode::X).await; press(g, c, KeyCode::Left).await; press(g, c, KeyCode::Down).await; tickn!(10, &[], &[]); log!("after unmute/-music/-sfx: music_vol={:.1} sfx_vol={:.1} applied={:.2} dirty={}", g.music_vol, g.sfx_vol, m.applied, g.settings_dirty);
    } else if phase == "0" {
        log!("load-only check");
    } else if phase == "3" && c.now().hour() < 18 {
        // P1-01..04: shovel purchase, digging, fishing, bugs, resident interior
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        approach(g, c, tc(DISPLAYS[5])).await; press(g, c, KeyCode::E).await; press(g, c, KeyCode::Enter).await; log!("shovel bought: tools={:?} money={}", g.w.p.tools, g.w.p.money);
        go(g, c, vec2(3.5, 4.5), 0.12).await; for _ in 0..30 { if g.me.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await;
        for _ in 0..4 { if g.w.p.equip == Some(62) { break } press(g, c, KeyCode::Tab).await }
        if let Some(site) = g.w.digs.first().copied() { approach(g, c, tc(site)).await; press(g, c, KeyCode::Space).await; log!("dig site {:?}: hole={} inv={:?}", site, g.w.holes.contains(&site), g.w.p.inv); shot!("p3_dig_site");
            press(g, c, KeyCode::Space).await; log!("filled again: hole={}", g.w.holes.contains(&site)); }
        for _ in 0..4 { if g.w.p.equip == Some(61) { break } press(g, c, KeyCode::Tab).await }
        go(g, c, vec2(23.5, 5.5), 0.12).await; face(g, c, vec2(25., 5.5)).await;
        'fish: for attempt in 0..8 { press(g, c, KeyCode::Space).await; log!("cast {attempt}: fishing={} target_fish={:?} shadows={}", g.me.fish.is_some(), g.me.fish.as_ref().and_then(|f| f.fish), g.shadows.len());
            for _ in 0..600 { if g.me.fish.is_none() { break } if g.me.fish.as_ref().is_some_and(|f| f.bite) { shot!("p3_fish_bite"); press(g, c, KeyCode::Space).await; log!("reeled at bite: inv={:?}", g.w.p.inv); break 'fish } step(g, c, &[], &[]).await }
            wait(g, c, 60).await }
        for _ in 0..4 { if g.w.p.equip == Some(60) { break } press(g, c, KeyCode::Tab).await }
        for _ in 0..3 { let mut b = None; for _ in 0..900 { if let Some(x) = g.bugs.iter().map(|b| b.1).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos))) { b = Some(x); break } step(g, c, &[], &[]).await }
            let Some(bp) = b else { log!("no bug spawned within 15 s"); break }; approach(g, c, bp).await; let bp2 = g.bugs.iter().map(|b| b.1).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos))); if let Some(x) = bp2 { face(g, c, x).await } press(g, c, KeyCode::Space).await;
            log!("net swing: inv={:?}", g.w.p.inv); if g.w.p.inv.iter().any(|s| def(s.def).k == K::Bug) { shot!("p3_bug_caught"); break } }
        if let Some(r) = (0..4).find(|&r| where_(g.res(r), g.hr) == Loc::Home) { let b = HOUSES[r]; go(g, c, door_stand(b), 0.12).await; face(g, c, door_face(b)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
            log!("resident {r} home: scene={:?}", g.me.scene); shot!("p3_resident_home"); approach(g, c, res_spot()).await; press(g, c, KeyCode::E).await; log!("talk ui={:?}", g.me.ui); shot!("p3_resident_talk"); for _ in 0..4 { if g.me.ui == Ui::Play { break } press(g, c, KeyCode::E).await } }
        if let Some(site) = g.w.digs.first().copied() { for _ in 0..4 { if g.w.p.equip == Some(62) { break } press(g, c, KeyCode::Tab).await } approach(g, c, tc(site)).await; press(g, c, KeyCode::Space).await; log!("second dig: inv={:?}", g.w.p.inv) }
        if g.me.scene != Scene::Town { go(g, c, vec2(1.5, 3.5), 0.12).await; for _ in 0..30 { if g.me.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await }
        let mb = BLDS.iter().position(|b| b.kind == BK::Museum).unwrap(); go(g, c, door_stand(mb), 0.12).await; face(g, c, door_face(mb)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; log!("museum: scene={:?}", g.me.scene);
        go(g, c, vec2(3.5, 2.5), 0.12).await; face(g, c, vec2(3.5, 1.)).await; press(g, c, KeyCode::E).await; log!("curator: ui={:?}", g.me.ui); shot!("p3_museum_curator");
        for _ in 0..12 { if g.me.ui == Ui::Play { break } press(g, c, KeyCode::E).await } log!("donated={:?} inv={:?}", g.w.donated, g.w.p.inv);
        go(g, c, vec2(3.5, 3.4), 0.12).await; face(g, c, vec2(3.5, 1.)).await; shot!("p3_museum_exhibits");
        log!("phase 3A end | {}", state(g));
    } else if phase == "3" {
        // P1-05: night lighting outdoors, lamp on/off indoors (lamp granted as a logged test fixture)
        go(g, c, vec2(13.5, 17.5), 0.15).await; face(g, c, vec2(13.5, 10.)).await; shot!("p3_night_town");
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await;
        if !g.w.p.room.iter().any(|f| f.def == 54) { g.add_furn(54); log!("FIXTURE: lamp added to pockets for lighting check");
            let idx = g.w.p.inv.iter().position(|s| s.def == 54).unwrap(); go(g, c, vec2(2.5, 3.5), 0.1).await; face(g, c, vec2(2.5, 1.)).await;
            press(g, c, KeyCode::I).await; for _ in 0..idx { press(g, c, KeyCode::S).await } press(g, c, KeyCode::Enter).await; press(g, c, KeyCode::D).await; press(g, c, KeyCode::Enter).await; log!("lamp placed: {:?}", g.w.p.room) }
        let lamp = g.w.p.room.iter().find(|f| f.def == 54).copied().unwrap(); approach(g, c, fcenter(&lamp)).await; go(g, c, vec2(1.5, 3.4), 0.12).await; face(g, c, vec2(1.5, 0.)).await; shot!("p3_lamp_off");
        approach(g, c, fcenter(&lamp)).await; press(g, c, KeyCode::E).await; log!("lamp on={:?}", g.w.p.room.iter().find(|f| f.def == 54).map(|f| f.on)); go(g, c, vec2(1.5, 3.4), 0.12).await; face(g, c, vec2(1.5, 0.)).await; shot!("p3_lamp_on");
        log!("phase 3B end | {}", state(g));
    } else {
        shot!("11_relaunch_outdoor");
        go(g, c, door_stand(HOME), 0.12).await; face(g, c, door_face(HOME)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; log!("home: scene={:?}", g.me.scene); shot!("12_relaunch_home");
        go(g, c, vec2(2.5, 3.5), 0.12).await; for _ in 0..30 { if g.me.scene == Scene::Town { break } step(g, c, &[KeyCode::S], &[]).await } wait(g, c, 40).await;
        go(g, c, door_stand(SHOP), 0.12).await; face(g, c, door_face(SHOP)).await; press(g, c, KeyCode::E).await; wait(g, c, 40).await; go(g, c, vec2(3.5, 2.6), 0.15).await; shot!("13_relaunch_shop_soldout");
        log!("phase 2 end | {}", state(g));
    }
    log!("end time {} hr {}", c.now(), c.now().hour());
    let _ = std::fs::write(format!("{dir}/autotest_phase{phase}.txt"), rep);
}

/// Save the current frame; alpha forced opaque (shadow blending lowers framebuffer alpha, which PNG viewers would show as transparency).
fn screenshot(path: &str) { let mut img = get_screen_data(); for px in img.bytes.chunks_mut(4) { px[3] = 255 } img.export_png(path) }
