mod audio;
mod clock;
mod data;
mod events;
mod net;
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

fn conf() -> Conf {
    Conf {
        window_title: "Hollowbrook".into(),
        window_width: 1280,
        window_height: 720,
        sample_count: 4,
        ..Default::default()
    }
}

/// Tiny procedural sound effects (no external assets).
struct Sounds(Vec<(Snd, Sound)>);
fn wav(s: &[f32]) -> Vec<u8> {
    let (sr, n) = (22050u32, s.len() as u32);
    let mut b = Vec::with_capacity(44 + s.len() * 2);
    b.extend(b"RIFF");
    b.extend((36 + n * 2).to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(sr.to_le_bytes());
    b.extend((sr * 2).to_le_bytes());
    b.extend(2u16.to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend((n * 2).to_le_bytes());
    for v in s {
        b.extend(((v.clamp(-1., 1.) * 32767.) as i16).to_le_bytes())
    }
    b
}
fn synth(s: Snd) -> Vec<f32> {
    let sr = 22050.;
    let tone = |f: f32, d: f32, v: f32| {
        (0..(sr * d) as usize)
            .map(|i| {
                let t = i as f32 / sr;
                (t * f * std::f32::consts::TAU).sin() * v * (1. - t / d)
            })
            .collect::<Vec<f32>>()
    };
    let noise = |d: f32, v: f32| {
        (0..(sr * d) as usize)
            .map(|i| {
                let t = i as f32 / sr;
                rand::gen_range(-1., 1.) * v * (1. - t / d).powi(2)
            })
            .collect::<Vec<f32>>()
    };
    let lp = |v: Vec<f32>, k: f32| {
        let mut y = 0.;
        v.into_iter()
            .map(|x| {
                y += (x - y) * k;
                y
            })
            .collect::<Vec<f32>>()
    }; // one-pole low-pass
    match s {
        Snd::StepPath => noise(0.06, 0.35),
        Snd::StepSand => lp(noise(0.09, 0.5), 0.25),
        Snd::StepWood => [tone(170., 0.06, 0.35), noise(0.02, 0.1)].concat(),
        Snd::StepFloor => tone(240., 0.045, 0.25),
        Snd::Swim => lp(noise(0.2, 0.35), 0.15),
        Snd::Step => lp(noise(0.06, 0.4), 0.35),
        Snd::Blip => tone(880., 0.07, 0.35),
        Snd::Splash => noise(0.28, 0.4),
        Snd::Coin => [tone(988., 0.08, 0.3), tone(1319., 0.16, 0.3)].concat(),
        Snd::Dig => noise(0.12, 0.5),
        Snd::Rustle => noise(0.22, 0.2),
        Snd::Door => [tone(330., 0.08, 0.3), tone(262., 0.12, 0.3)].concat(),
    }
}
impl Sounds {
    async fn load() -> Sounds {
        let mut v = vec![];
        for s in [
            Snd::StepPath,
            Snd::StepSand,
            Snd::StepWood,
            Snd::StepFloor,
            Snd::Swim,
            Snd::Step,
            Snd::Blip,
            Snd::Splash,
            Snd::Coin,
            Snd::Dig,
            Snd::Rustle,
            Snd::Door,
        ] {
            if let Ok(x) = load_sound_from_bytes(&wav(&synth(s))).await {
                v.push((s, x))
            }
        }
        Sounds(v)
    }
    fn play(&self, q: &mut Vec<Snd>, vol: f32) {
        for s in q.drain(..) {
            if vol <= 0. {
                continue;
            }
            if let Some((_, x)) = self.0.iter().find(|e| e.0 == s) {
                play_sound(
                    x,
                    PlaySoundParams {
                        looped: false,
                        volume: vol
                            * if matches!(
                                s,
                                Snd::Step | Snd::StepPath | Snd::StepSand | Snd::StepWood | Snd::StepFloor | Snd::Swim
                            ) {
                                0.2
                            } else {
                                0.45
                            },
                    },
                )
            }
        }
    }
}

/// Join screen + client loop. The local save is never touched while connected. Returns true if the window was closed.
async fn lan_join(sounds: &Sounds, pads: &mut pad::Pads, vol: (f32, f32, bool)) -> bool {
    let mut finder = net::Finder::new();
    let (mut sel, mut ip, mut msg) = (0usize, String::new(), String::new());
    let mut client: Option<net::Client> = None;
    while client.is_none() {
        finder.poll();
        pads.poll();
        let dt = get_frame_time();
        let inp = pad::merge(Inp { down: get_keys_down(), pressed: get_keys_pressed() }, pads.inp(0, dt));
        while let Some(c) = get_char_pressed() {
            if (c.is_ascii_digit() || c == '.') && ip.len() < 15 {
                ip.push(c)
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            ip.pop();
        }
        let n = finder.hosts.len();
        if inp.p(KeyCode::Up) || inp.p(KeyCode::W) {
            sel = sel.saturating_sub(1)
        }
        if inp.p(KeyCode::Down) || inp.p(KeyCode::S) {
            sel = (sel + 1).min(n.saturating_sub(1))
        }
        if is_quit_requested() {
            return true;
        }
        if inp.p(KeyCode::Escape) {
            return false;
        }
        if inp.p(KeyCode::Enter) || inp.p(KeyCode::KpEnter) || (inp.p(KeyCode::E) && ip.is_empty()) {
            let addr: Option<std::net::SocketAddr> = if !ip.is_empty() {
                format!("{ip}:{}", net::PORT).parse().ok()
            } else {
                finder.hosts.get(sel).map(|h| h.1)
            };
            match addr {
                Some(a) => match net::Client::connect(a) {
                    Ok(c) => client = Some(c),
                    Err(e) => msg = format!("Could not connect to {a}: {e}"),
                },
                None => {
                    msg = if ip.is_empty() {
                        "No host found yet. Ask your friend to choose Host a LAN game.".into()
                    } else {
                        "That address doesn't look right.".into()
                    }
                }
            }
        }
        clear_background(Color::new(0.12, 0.2, 0.16, 1.));
        let (sw, sh) = (screen_width(), screen_height());
        let wcol = Color::new(0.92, 0.95, 0.92, 1.);
        draw_text("Join a LAN game", 60., 90., 48., Color::new(0.95, 0.9, 0.7, 1.));
        draw_text("Hosts on your network:", 60., 150., 26., wcol);
        if !finder.ok() {
            draw_text(
                "(Can't listen for hosts on this computer; type the host's address instead.)",
                70.,
                185.,
                20.,
                Color::new(1., 0.7, 0.6, 1.),
            );
        }
        for (k, h) in finder.hosts.iter().enumerate() {
            let y = 190. + k as f32 * 34.;
            if k == sel && ip.is_empty() {
                draw_rectangle(60., y - 24., 520., 32., Color::new(0.75, 0.6, 0.25, 1.))
            }
            draw_text(&format!("{}'s town  ({})", h.0, h.1.ip()), 70., y, 26., wcol);
        }
        if finder.hosts.is_empty() {
            draw_text("Looking for hosts...", 70., 190., 24., Color::new(0.75, 0.8, 0.75, 1.));
        }
        draw_text(&format!("Or type the host's IP address: {}_", ip), 60., sh - 170., 24., GOLD);
        draw_text(&msg, 60., sh - 130., 20., Color::new(1., 0.7, 0.6, 1.));
        draw_text("Up/Down choose   Enter/A join   Esc/B back   Backspace delete", 60., sh - 80., 20., wcol);
        draw_text(
            "You play as the host town's Player 2; progress is saved on the host.",
            60.,
            sh - 50.,
            18.,
            Color::new(0.75, 0.8, 0.75, 1.),
        );
        let _ = sw;
        next_frame().await
    }
    let mut c = client.unwrap();
    let mut g: Option<Game> = None;
    let mut music = audio::Music::new();
    let t0 = std::time::Instant::now();
    let mut now = chrono::Local::now().naive_local();
    loop {
        c.poll();
        let dt = get_frame_time().min(0.05);
        if c.closed || (g.is_none() && t0.elapsed().as_secs() > 8) {
            let why = c.refused.clone().unwrap_or_else(|| {
                if g.is_none() {
                    "The host didn't answer.".into()
                } else {
                    "The connection to the host was lost.".into()
                }
            });
            for _ in 0..180 {
                clear_background(Color::new(0.12, 0.2, 0.16, 1.));
                draw_text(&why, 60., 200., 30., WHITE);
                draw_text("Returning to the title screen...", 60., 250., 22., LIGHTGRAY);
                if is_quit_requested() {
                    return true;
                }
                next_frame().await
            }
            return false;
        }
        pads.poll();
        let pin = pads.inp(0, dt);
        let used = !pin.down.is_empty();
        let inp = pad::merge(Inp { down: get_keys_down(), pressed: get_keys_pressed() }, pin);
        if c.id.is_some() {
            c.send(&inp, used)
        }
        if let (Some(id), Some(s)) = (c.id, c.take_snap()) {
            let gg = g.get_or_insert_with(|| Game::from_world(s.w.clone(), town()));
            let (n, snd) = net::apply(gg, s, id);
            now = n;
            gg.snd.extend(snd)
        }
        match &mut g {
            Some(gg) => {
                render::draw(gg, now, None);
                set_default_camera();
                draw_text("LAN", screen_width() - 60., screen_height() - 12., 18., Color::new(0.7, 1., 0.8, 1.));
                sounds.play(&mut gg.snd, vol.1);
                music.tick((now.hour(), audio::variant(gg)), dt, vol.0, vol.2).await;
            }
            None => {
                clear_background(Color::new(0.12, 0.2, 0.16, 1.));
                draw_text("Connecting to the host...", 60., 200., 30., WHITE);
            }
        }
        if is_quit_requested() {
            c.bye();
            return true;
        }
        next_frame().await
    }
}
fn load_game() -> Option<Game> {
    match save::load_checked(SAVE) {
        save::Loaded::Ok(w) => {
            save::backup(SAVE, save::BACKUPS);
            return Some(Game::from_world(w, town()));
        }
        save::Loaded::Corrupt(p) => eprintln!("[save] {SAVE} could not be read; moved aside to {p}"),
        save::Loaded::Missing => {}
    }
    let map = town();
    save::backup(save::LEGACY, save::BACKUPS);
    let w = save::migrate(save::LEGACY, &map)?;
    eprintln!("[save] migrated legacy save {} -> {SAVE} (legacy file kept)", save::LEGACY);
    save::save(&w, SAVE);
    Some(Game::from_world(w, map))
}

#[macroquad::main(conf)]
async fn main() {
    let clock = Clock::from_env();
    let dbg = clock.dev.is_some() || std::env::var("HOLLOWBROOK_DEBUG").is_ok();
    rand::srand(Local::now().timestamp() as u64);
    let sounds = Sounds::load().await;
    eprintln!("[audio] backend: quad-snd; {} sound effects loaded", sounds.0.len());
    let auto = std::env::var("HOLLOWBROOK_AUTOTEST").ok();
    if auto.is_some() {
        rand::srand(0x5eed); // scripted runs use fixed randomness so screenshots repeat (see tools/shots.sh)
    }
    let mut g = match load_game() {
        Some(g) => g,
        None if auto.is_some() => Game::new("Tester".into(), clock.now()),
        None => {
            let mut name = String::new();
            loop {
                while let Some(c) = get_char_pressed() {
                    if (c.is_alphanumeric() || c == ' ') && name.len() < 12 {
                        name.push(c)
                    }
                }
                if is_key_pressed(KeyCode::Backspace) {
                    name.pop();
                }
                if is_key_pressed(KeyCode::Enter) {
                    if name.trim().is_empty() {
                        name = "Neighbor".into()
                    }
                    break;
                }
                clear_background(Color::new(0.3, 0.55, 0.35, 1.));
                draw_text("Hollowbrook", 80., 160., 80., WHITE);
                draw_text("A little town by the river is waiting for a new neighbor.", 80., 230., 30., WHITE);
                draw_text(&format!("What's your name?  {}_", name), 80., 320., 40., GOLD);
                draw_text("Type, then press Enter (or just Enter for \"Neighbor\")", 80., 370., 24., LIGHTGRAY);
                draw_text(VERSION, 80., 680., 20., LIGHTGRAY);
                next_frame().await
            }
            let mut g = Game::new(name.trim().to_string(), clock.now());
            g.fresh = true;
            g
        }
    };
    {
        let st = audio::load_settings(audio::SETTINGS);
        g.music_vol = st.music;
        g.sfx_vol = st.sfx;
        g.muted = st.muted;
    }
    g.curve = std::env::var("HOLLOWBROOK_CURVE").ok().and_then(|s| s.parse().ok()).unwrap_or(0.012); // dev setting
    if g.catch_up(clock.now()) {
        save::save(&g.persist(), SAVE)
    }
    prevent_quit();
    if auto.is_none() {
        g.me.ui = Ui::Title(0)
    }
    eprintln!("[hollowbrook] {VERSION}");
    if let Some(dir) = auto {
        autotest(&mut g, &clock, &dir).await;
        save::save(&g.persist(), SAVE);
        return;
    }
    let mut music = audio::Music::new();
    let mut host: Option<net::Host> = None;
    let mut remote_snd: Vec<Snd> = vec![];
    let (mut st, mut shots) = (0., 0);
    let mut pads = pad::Pads::new();
    eprintln!("[pad] {} gamepad(s) detected", pads.count());
    loop {
        let dt = get_frame_time().min(0.05);
        let now = clock.now();
        let kb = Inp { down: get_keys_down(), pressed: get_keys_pressed() };
        if g.tick(dt, now) {
            save::save(&g.persist(), SAVE)
        } // persist a refresh immediately
        pads.poll();
        if std::mem::take(&mut pads.switched) {
            let p2 = pads.slots.active(1);
            let joined = g.ids().contains(&1);
            if p2 && !joined {
                g.join("Player 2");
                g.note("Controller switched: Player 2 joined.")
            } else if !p2 && joined {
                g.leave(1);
                g.note("Controller switched to Player 1. Player 2 left; their progress is saved.")
            } else {
                g.note("Controllers switched players.")
            }
            eprintln!("[pad] switched: slots {:?}", pads.slots.ids);
            save::save(&g.persist(), SAVE)
        }
        for slot in std::mem::take(&mut pads.claimed) {
            eprintln!("[pad] controller assigned to player {}", slot + 1);
            if slot == 1 && !g.ids().contains(&1) {
                g.join("Player 2");
                g.note("Player 2 joined with a controller.");
                save::save(&g.persist(), SAVE)
            }
        }
        if g.net_req == 1 {
            g.net_req = 0;
            if host.is_none() {
                match net::Host::start(&g.w.p.name) {
                    Ok(h) => {
                        g.note(format!(
                            "Hosting a LAN game at {}. On the other computer choose Join a LAN game.",
                            net::Host::lan_ip()
                        ));
                        eprintln!("[lan] hosting on port {}", h.port());
                        host = Some(h)
                    }
                    Err(e) => g.note(format!("Couldn't start hosting: {e}")),
                }
            } else {
                g.note("Already hosting.")
            }
        }
        if g.net_req == 2 {
            g.net_req = 0;
            save::save(&g.persist(), SAVE);
            if lan_join(&sounds, &mut pads, (g.music_vol, g.sfx_vol, g.muted)).await {
                break;
            }
            g.me.ui = Ui::Title(0)
        }
        if let Some(h) = &mut host {
            for ev in h.poll() {
                match ev {
                    net::HostEv::Joined => {
                        if g.ids().contains(&1) && g.remote != Some(1) {
                            h.refuse("Player 2 is already playing on the host computer.");
                            g.note("A LAN player tried to join, but Player 2 is playing here.")
                        } else {
                            if !g.ids().contains(&1) {
                                g.join("Player 2");
                            }
                            g.remote = Some(1);
                            h.welcome(1);
                            g.note("A friend joined over LAN as Player 2!");
                            eprintln!("[lan] friend joined");
                            save::save(&g.persist(), SAVE)
                        }
                    }
                    net::HostEv::Left => {
                        if g.remote == Some(1) {
                            g.remote = None;
                            g.activate(0);
                            g.leave(1);
                            g.note("Your LAN friend left. Their progress is saved.");
                            eprintln!("[lan] friend left");
                            save::save(&g.persist(), SAVE)
                        }
                    }
                }
            }
        }
        for id in g.ids() {
            g.activate(id);
            if g.remote == Some(id) {
                let inp = host.as_mut().map_or(Inp::none(), |h| h.take_input());
                g.me.pad = host.as_ref().is_some_and(|h| h.pad);
                let before = g.snd.len();
                g.update_player(&inp, dt);
                remote_snd.extend(g.snd.drain(before..));
                continue;
            }
            let p = pads.inp(id.min(1), dt);
            let used = !p.down.is_empty();
            let inp = if id == 0 {
                if !kb.down.is_empty() {
                    g.me.pad = false
                }
                pad::merge(Inp { down: kb.down.clone(), pressed: kb.pressed.clone() }, p)
            } else {
                p
            };
            if used {
                g.me.pad = true
            }
            g.me.pad_lost = pads.slots.lost(id.min(1));
            g.update_player(&inp, dt)
        }
        g.activate(0);
        if let Some(h) = &mut host {
            if g.remote.is_some() && h.due() {
                let snap = net::snap(&g, now, std::mem::take(&mut remote_snd));
                h.send(snap)
            } else if g.remote.is_none() {
                remote_snd.clear()
            }
        }
        if kb.p(KeyCode::F2) && g.remote.is_some() {
            g.note("Player 2 is connected over LAN.")
        } else if kb.p(KeyCode::F2) {
            if g.ids().len() == 1 {
                g.join("Player 2");
                g.note("Player 2 joined (split-screen).")
            } else {
                g.leave(1);
                g.note("Player 2 left; their progress is saved.")
            }
            save::save(&g.persist(), SAVE)
        }
        render::draw(&mut g, now, dbg.then(|| clock.source()));
        sounds.play(&mut g.snd, g.sfx_vol);
        music.tick((now.hour(), audio::variant(&g)), dt, g.music_vol, g.muted).await;
        for l in music.log.drain(..) {
            eprintln!("[audio] {l}")
        }
        if g.settings_dirty {
            g.settings_dirty = false;
            audio::save_settings(
                audio::SETTINGS,
                &audio::Settings { music: g.music_vol, sfx: g.sfx_vol, muted: g.muted },
            )
        }
        if dbg && is_key_pressed(KeyCode::F12) {
            shots += 1;
            screenshot(&format!("shot_{shots}.png"));
            eprintln!("[shot] shot_{shots}.png")
        }
        st += dt;
        if st > 15. {
            save::save(&g.persist(), SAVE);
            st = 0.
        }
        if is_quit_requested() || g.quit {
            save::save(&g.persist(), SAVE);
            break;
        }
        next_frame().await
    }
}

// ---- dev-only end-to-end bot: injects key events into the same Inp -> Game::update path the keyboard uses ----
async fn step(g: &mut Game, c: &Clock, down: &[KeyCode], pressed: &[KeyCode]) {
    let now = c.now();
    if g.update(&Inp::keys(down, pressed), 1. / 60., now) {
        save::save(&g.persist(), SAVE)
    }
    render::draw(g, now, Some(c.source()));
    g.snd.clear();
    next_frame().await
}
async fn step2(g: &mut Game, c: &Clock, i0: Inp, i1: Inp) {
    let now = c.now();
    if g.tick(1. / 60., now) {
        save::save(&g.persist(), SAVE)
    }
    g.activate(0);
    g.update_player(&i0, 1. / 60.);
    g.activate(1);
    g.update_player(&i1, 1. / 60.);
    g.activate(0);
    render::draw(g, now, Some(c.source()));
    g.snd.clear();
    next_frame().await
}
async fn press(g: &mut Game, c: &Clock, k: KeyCode) {
    step(g, c, &[k], &[k]).await;
    step(g, c, &[], &[]).await
}
async fn wait(g: &mut Game, c: &Clock, n: usize) {
    for _ in 0..n {
        step(g, c, &[], &[]).await
    }
}
fn keys_toward(d: Vec2) -> Vec<KeyCode> {
    let mut k = vec![];
    if d.y < -0.06 {
        k.push(KeyCode::W)
    }
    if d.y > 0.06 {
        k.push(KeyCode::S)
    }
    if d.x < -0.06 {
        k.push(KeyCode::A)
    }
    if d.x > 0.06 {
        k.push(KeyCode::D)
    }
    k
}
async fn go(g: &mut Game, c: &Clock, to: Vec2, tol: f32) -> bool {
    let (mut last, mut still) = (g.ppos(), 0);
    for f in 0..2400 {
        let p = g.ppos();
        if p.distance(to) < tol {
            return true;
        }
        let cur = tile_of(p);
        let path = g.bfs(cur, tile_of(to), false).unwrap_or_default();
        let keys = if path.len() > 1 {
            let nx = path[0];
            let cc = tc(cur);
            // keep the box in the lane before crossing into the next tile (avoids catching corners)
            if nx.0 != cur.0 && (p.y - cc.y).abs() > 0.1 {
                keys_toward(vec2(0., cc.y - p.y))
            } else if nx.1 != cur.1 && (p.x - cc.x).abs() > 0.1 {
                keys_toward(vec2(cc.x - p.x, 0.))
            } else {
                keys_toward(tc(nx) - p)
            }
        } else {
            keys_toward(to - p)
        };
        let keys = if still > 30 {
            still = 0;
            vec![[KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D][f % 4]]
        } else {
            keys
        };
        step(g, c, &keys, &[]).await;
        if g.ppos().distance(last) < 0.001 {
            still += 1
        } else {
            still = 0
        }
        last = g.ppos();
    }
    false
}
async fn face(g: &mut Game, c: &Clock, at: Vec2) {
    let d = at - g.ppos();
    let k = if d.x.abs() > d.y.abs() {
        if d.x > 0. {
            KeyCode::D
        } else {
            KeyCode::A
        }
    } else if d.y > 0. {
        KeyCode::S
    } else {
        KeyCode::W
    };
    step(g, c, &[k], &[]).await;
    step(g, c, &[], &[]).await
}
/// Walk next to `at` (a free neighbouring spot) and face it.
async fn approach(g: &mut Game, c: &Clock, at: Vec2) -> bool {
    let mut spots: Vec<Vec2> = [(0., 0.8), (0., -0.8), (0.8, 0.), (-0.8, 0.)]
        .iter()
        .map(|&(x, z)| at + vec2(x, z))
        .filter(|&s| g.free(s))
        .collect();
    spots.sort_by(|a, b| a.distance(g.ppos()).total_cmp(&b.distance(g.ppos())));
    for s in spots {
        if go(g, c, s, 0.12).await {
            face(g, c, at).await;
            return true;
        }
    }
    false
}
async fn autotest(g: &mut Game, c: &Clock, dir: &str) {
    let phase = std::env::var("HOLLOWBROOK_AUTOTEST_PHASE").unwrap_or("1".into());
    let mut rep = String::new();
    macro_rules! log { ($($a:tt)*) => {{ let s = format!($($a)*); eprintln!("[auto] {s}"); rep += &s; rep.push('\n'); }} }
    macro_rules! shot {
        ($n:expr) => {{
            wait(g, c, 2).await;
            render::draw(g, c.now(), None);
            screenshot(&format!("{dir}/{}.png", $n));
            next_frame().await;
            log!("shot {}", $n);
        }};
    }
    let state = |g: &Game| {
        format!(
            "money={} inv={:?} stock={:?} room={:?} tools={:?}",
            g.w.p.money,
            g.w.p.inv.iter().map(|s| (def(s.def).name, s.qty, s.uid)).collect::<Vec<_>>(),
            g.w.stock.iter().map(|s| (def(s.def).name, s.sold)).collect::<Vec<_>>(),
            g.w.p.room.iter().map(|f| (def(f.def).name, f.uid, f.x, f.z, f.rot, f.on)).collect::<Vec<_>>(),
            g.w.p.tools
        )
    };
    log!("phase {phase} start {} | {}", c.now(), state(g));
    wait(g, c, 30).await;
    if phase == "1" {
        shot!("01_outdoor_start");
        go(g, c, vec2(13.5, 11.4), 0.15).await;
        face(g, c, vec2(13.5, 13.)).await;
        shot!("02_behind_house_occlusion");
        go(g, c, vec2(16.5, 23.4), 0.2).await;
        face(g, c, vec2(16.5, 10.)).await;
        shot!("03_beach_rolling_horizon");
        let shell =
            g.w.ground
                .iter()
                .filter(|i| def(i.def).k == K::Shell)
                .min_by(|a, b| vec2(a.x, a.z).distance(g.me.pos).total_cmp(&vec2(b.x, b.z).distance(g.me.pos)))
                .map(|i| vec2(i.x, i.z));
        if let Some(s) = shell {
            approach(g, c, s).await;
            press(g, c, KeyCode::E).await;
        }
        log!("after shell: inv={:?}", g.w.p.inv);
        if let Some(i) = (0..4).find(|&i| !g.npcs[i].inside) {
            for _ in 0..900 {
                let np = g.npcs[i].pos;
                if g.me.pos.distance(np) < 1.1 {
                    break;
                }
                let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default();
                let wp = if path.len() > 1 { tc(path[0]) } else { np };
                step(g, c, &keys_toward(wp - g.me.pos), &[]).await
            }
            face(g, c, g.npcs[i].pos).await;
            press(g, c, KeyCode::E).await;
            log!("talk to resident {i}: ui={:?}", g.me.ui);
            if matches!(g.me.ui, Ui::Talk { .. }) {
                shot!("04_resident_conversation")
            }
            for _ in 0..4 {
                if g.me.ui == Ui::Play {
                    break;
                }
                press(g, c, KeyCode::E).await
            }
        }
        if let Some(ti) = (0..g.w.trees.len()).filter(|&i| g.w.trees[i].fruit).min_by(|&a, &b| {
            tc((g.w.trees[a].x, g.w.trees[a].z))
                .distance(g.me.pos)
                .total_cmp(&tc((g.w.trees[b].x, g.w.trees[b].z)).distance(g.me.pos))
        }) {
            let tp = tc((g.w.trees[ti].x, g.w.trees[ti].z));
            approach(g, c, tp).await;
            press(g, c, KeyCode::E).await;
            log!(
                "shook tree {ti}: fruit now {} ground fruit {}",
                g.w.trees[ti].fruit,
                g.w.ground.iter().filter(|i| i.def == g.w.native).count()
            );
            for _ in 0..3 {
                let Some(f) =
                    g.w.ground
                        .iter()
                        .filter(|i| i.def == g.w.native)
                        .map(|i| vec2(i.x, i.z))
                        .min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos)))
                else {
                    break;
                };
                approach(g, c, f).await;
                press(g, c, KeyCode::E).await
            }
            log!("after fruit: inv={:?}", g.w.p.inv);
        }
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        log!("scene after shop door: {:?}", g.me.scene);
        shot!("05_shop_interior");
        go(g, c, vec2(3.5, 2.5), 0.12).await;
        face(g, c, vec2(3.5, 1.5)).await;
        press(g, c, KeyCode::E).await;
        log!("counter ui={:?}", g.me.ui);
        shot!("06_sell_menu");
        let m0 = g.w.p.money;
        press(g, c, KeyCode::A).await;
        log!("sold all finds: money {m0} -> {}", g.w.p.money);
        press(g, c, KeyCode::Escape).await;
        if let Some(d) = (0..5)
            .filter(|&i| !g.w.stock[i].sold && def(g.w.stock[i].def).buy <= g.w.p.money && !def(g.w.stock[i].def).wall)
            .max_by_key(|&i| {
                let d = def(g.w.stock[i].def);
                ((d.act == Act::Sit) as u32 * 4 + d.rot as u32 * 2 + (d.act != Act::None) as u32, u32::MAX - d.buy)
            })
        {
            approach(g, c, tc(DISPLAYS[d])).await;
            press(g, c, KeyCode::E).await;
            log!("display {d} ui={:?}", g.me.ui);
            let m1 = g.w.p.money;
            press(g, c, KeyCode::Enter).await;
            press(g, c, KeyCode::Enter).await;
            log!("bought {}: money {m1} -> {}, sold={}", def(g.w.stock[d].def).name, g.w.p.money, g.w.stock[d].sold);
            shot!("07_shop_after_purchase");
        }
        go(g, c, vec2(3.5, 4.5), 0.12).await;
        for _ in 0..30 {
            if g.me.scene == Scene::Town {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        log!("left shop: scene={:?} pos={:?}", g.me.scene, g.me.pos);
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        log!("home: scene={:?}", g.me.scene);
        if let Some(idx) = g.w.p.inv.iter().position(|s| def(s.def).k == K::Furn) {
            go(g, c, vec2(2.5, 3.5), 0.1).await;
            face(g, c, vec2(2.5, 1.)).await;
            press(g, c, KeyCode::I).await;
            for _ in 0..idx {
                press(g, c, KeyCode::S).await
            }
            press(g, c, KeyCode::Enter).await;
            log!("placing={:?}", g.me.placing.as_ref().map(|p| p.f));
            press(g, c, KeyCode::A).await;
            press(g, c, KeyCode::W).await;
            if g.me.placing.as_ref().is_some_and(|p| def(p.f.def).rot) {
                press(g, c, KeyCode::R).await
            }
            shot!("08_placing_ghost");
            press(g, c, KeyCode::Enter).await;
            if g.me.placing.is_some() {
                press(g, c, KeyCode::D).await;
                press(g, c, KeyCode::Enter).await
            }
            log!("after place: placing={} room={:?} inv={:?}", g.me.placing.is_some(), g.w.p.room, g.w.p.inv);
            if let Some(f) = g.w.p.room.last().copied() {
                let cpos = fcenter(&f);
                approach(g, c, cpos).await;
                let tg = g.target();
                press(g, c, KeyCode::E).await;
                wait(g, c, 20).await;
                log!(
                    "used furniture target={:?} pose={:?} ui={:?} lamp_on={:?}",
                    tg,
                    g.me.pose,
                    g.me.ui,
                    g.w.p.room.iter().find(|x| x.uid == f.uid).map(|x| x.on)
                );
                shot!("09_furniture_in_use");
                if g.me.ui != Ui::Play {
                    press(g, c, KeyCode::Escape).await
                }
                if g.me.pose != Pose::Stand {
                    step(g, c, &[KeyCode::S], &[]).await
                }
                log!("stood up: pose={:?}", g.me.pose);
                go(g, c, vec2(3.4, 3.3), 0.15).await;
                face(g, c, vec2(3.4, 4.)).await;
                shot!("10_furnished_home");
            }
        }
        log!("phase 1 end | {}", state(g));
    } else if phase == "4" {
        // MP-02/03: two players with independent virtual inputs; one goes indoors, the other stays outside
        g.join("Player 2");
        log!("joined: ids={:?}", g.ids());
        let k = |d: &[KeyCode], p: &[KeyCode]| Inp::keys(d, p);
        for _ in 0..20 {
            step2(g, c, k(&[], &[]), k(&[], &[])).await
        }
        shot!("mp_same_scene");
        for _ in 0..45 {
            step2(g, c, k(&[], &[]), k(&[KeyCode::A], &[])).await
        }
        shot!("mp_together");
        step2(g, c, k(&[KeyCode::W], &[]), k(&[], &[])).await;
        step2(g, c, k(&[KeyCode::E], &[KeyCode::E]), k(&[], &[])).await;
        for _ in 0..50 {
            step2(g, c, k(&[], &[]), k(&[KeyCode::D, KeyCode::S], &[])).await
        }
        let st = |g: &mut Game, id: usize| {
            g.activate(id);
            let s = format!(
                "P{} scene={:?} pos=({:.2},{:.2}) ipos=({:.2},{:.2}) ui={:?}",
                id + 1,
                g.me.scene,
                g.me.pos.x,
                g.me.pos.y,
                g.me.ipos.x,
                g.me.ipos.y,
                g.me.ui
            );
            g.activate(0);
            s
        };
        log!("{} | {}", st(g, 0), st(g, 1));
        shot!("mp_split_home_and_town");
        step2(g, c, k(&[], &[]), k(&[KeyCode::I], &[KeyCode::I])).await;
        step2(g, c, k(&[], &[]), k(&[], &[])).await;
        log!("after P2 pressed I: {} | {}", st(g, 0), st(g, 1));
        shot!("mp_p2_inventory");
        step2(g, c, k(&[KeyCode::Escape], &[KeyCode::Escape]), k(&[], &[])).await;
        log!("P1 Esc does not close P2 menu: {}", st(g, 1));
        step2(g, c, k(&[], &[]), k(&[KeyCode::Escape], &[KeyCode::Escape])).await;
        log!("P2 Esc: {}", st(g, 1));
    } else if phase == "5" {
        // MP-04: virtual gamepad states -> pad::Mapper -> the same update path; slot disconnect/reconnect
        g.join("Player 2");
        let mut m = pad::Mapper::default();
        let mut sl = pad::Slots::default();
        sl.connect(5);
        log!("pad 5 claims slot {:?}", sl.claim(5));
        let ps = |b: &[pad::B], lx: f32, ly: f32| pad::PadState { b: b.iter().copied().collect(), lx, ly };
        for _ in 0..10 {
            let i = m.map(&ps(&[], 0., 0.), 1. / 60.);
            step2(g, c, Inp::none(), i).await
        }
        for _ in 0..3 {
            let i = m.map(&ps(&[], 0., 0.8), 1. / 60.);
            step2(g, c, Inp::none(), i).await
        } // stick up: face the door
        let i = m.map(&ps(&[pad::B::South], 0., 0.), 1. / 60.);
        step2(g, c, Inp::none(), i).await;
        for _ in 0..40 {
            let i = m.map(&ps(&[], 0., 0.), 1. / 60.);
            step2(g, c, Inp::none(), i).await
        }
        g.activate(1);
        g.me.pad = true;
        log!("P2 after A at own door: scene={:?}", g.me.scene);
        g.activate(0);
        let i = m.map(&ps(&[pad::B::North], 0., 0.), 1. / 60.);
        step2(g, c, Inp::none(), i).await;
        let i = m.map(&ps(&[], 0., 0.), 1. / 60.);
        step2(g, c, Inp::none(), i).await;
        g.activate(1);
        log!("P2 after Y: ui={:?}", g.me.ui);
        g.activate(0);
        shot!("mp_pad_prompts");
        let i = m.map(&ps(&[pad::B::East], 0., 0.), 1. / 60.);
        step2(g, c, Inp::none(), i).await;
        g.activate(1);
        log!("P2 after B: ui={:?}", g.me.ui);
        g.activate(0);
        sl.disconnect(5);
        g.activate(1);
        g.me.pad_lost = sl.lost(1);
        g.activate(0);
        log!("disconnect: lost={}", sl.lost(1));
        shot!("mp_pad_disconnected");
        sl.connect(6);
        log!("pad 6 claims {:?}; lost={}", sl.claim(6), sl.lost(1));
        g.activate(1);
        g.me.pad_lost = sl.lost(1);
        log!("P2 progress intact: scene={:?} money={}", g.me.scene, g.w.p.money);
        g.activate(0);
    } else if phase == "6" {
        // P5-02: buy a shirt at the rack, wear it from the pockets menu
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        approach(g, c, tc(RACK)).await;
        press(g, c, KeyCode::E).await;
        log!("rack ui={:?}", g.me.ui);
        press(g, c, KeyCode::S).await;
        shot!("p5_rack");
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::Escape).await;
        log!("after buy: money={} inv={:?}", g.w.p.money, g.w.p.inv);
        let idx = g.w.p.inv.iter().position(|s| def(s.def).k == K::Cloth).unwrap_or(0);
        press(g, c, KeyCode::I).await;
        for _ in 0..idx {
            press(g, c, KeyCode::S).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("wearing={:?} inv={:?}", g.w.p.shirt, g.w.p.inv);
        go(g, c, vec2(3.5, 2.6), 0.12).await;
        face(g, c, vec2(3.5, 4.)).await;
        shot!("p5_new_outfit");
    } else if phase == "7" {
        // P5-01: expand home at the shop counter (coins granted as a logged FIXTURE; the purchase path is the normal menu)
        g.w.p.money = 3000;
        log!("FIXTURE: money set to 3000");
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        go(g, c, vec2(3.5, 2.5), 0.12).await;
        face(g, c, vec2(3.5, 1.5)).await;
        press(g, c, KeyCode::E).await;
        for _ in 0..g.w.p.inv.len() {
            press(g, c, KeyCode::S).await
        }
        shot!("p51_counter_expand");
        press(g, c, KeyCode::Enter).await;
        log!("after expand: level={} money={} dims={:?}", g.w.p.home, g.w.p.money, g.rdims(HOME));
        press(g, c, KeyCode::Escape).await;
        go(g, c, vec2(3.5, 4.5), 0.12).await;
        for _ in 0..30 {
            if g.me.scene == Scene::Town {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        log!("home scene={:?} ipos={:?}", g.me.scene, g.me.ipos);
        shot!("p51_bigger_home");
    } else if phase == "8" {
        // P6-01: mailbox (one letter inserted as a logged FIXTURE; opening/claiming uses the normal interaction)
        g.w.p.mail.push(world::Letter {
            from: "Juniper".into(),
            text: "Thinking of you! Here's a little something.".into(),
            coins: 300,
            item: None,
            read: false,
            claimed: false,
        });
        log!("FIXTURE: letter with 300 coins placed in P1 mail");
        let mb = tc(mailbox_tile(0));
        approach(g, c, mb).await;
        log!("target={:?}", g.target());
        shot!("p61_mailbox_flag");
        let m0 = g.w.p.money;
        press(g, c, KeyCode::E).await;
        log!("ui={:?}", g.me.ui);
        press(g, c, KeyCode::Enter).await;
        log!("money {m0} -> {}, letter={:?}", g.w.p.money, g.w.p.mail[0]);
        shot!("p61_mail_menu");
        press(g, c, KeyCode::Enter).await;
        log!("second open: money {}", g.w.p.money);
        press(g, c, KeyCode::Escape).await;
    } else if phase == "9" {
        // P7-01/P6-02: weather presentation near a resident who is out
        wait(g, c, 60).await;
        if let Some(i) = (0..4).find(|&i| !g.npcs[i].inside) {
            for _ in 0..900 {
                let np = g.npcs[i].pos;
                if g.me.pos.distance(np) < 2.2 {
                    break;
                }
                let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default();
                let wp = if path.len() > 1 { tc(path[0]) } else { np };
                step(g, c, &keys_toward(wp - g.me.pos), &[]).await
            }
        }
        log!("weather={:?} pos={:?}", weather(g.w.day, c.now().month()), g.me.pos);
        shot!("p7_weather");
    } else if phase == "10" {
        // P5-03: buy at a stand, then re-order the same item from the counter catalog
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        let d = (0..5).filter(|&i| !def(g.w.stock[i].def).wall).min_by_key(|&i| def(g.w.stock[i].def).buy).unwrap();
        approach(g, c, tc(DISPLAYS[d])).await;
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::Enter).await;
        log!("bought {} money={}", def(g.w.stock[d].def).name, g.w.p.money);
        go(g, c, vec2(3.5, 2.5), 0.12).await;
        face(g, c, vec2(3.5, 1.5)).await;
        press(g, c, KeyCode::E).await;
        for _ in 0..g.w.p.inv.len() + 1 {
            press(g, c, KeyCode::S).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("ui={:?} catalog={:?}", g.me.ui, g.w.p.catalog);
        shot!("p53_catalog");
        press(g, c, KeyCode::Enter).await;
        log!(
            "after order: money={} inv={:?}",
            g.w.p.money,
            g.w.p.inv.iter().map(|s| (def(s.def).name, s.uid)).collect::<Vec<_>>()
        );
    } else if phase == "11" {
        // P6-03: fruit gathered normally, then given to Juniper through the talk -> G -> gift menu path
        if let Some(ti) = (0..g.w.trees.len()).filter(|&i| g.w.trees[i].fruit).min_by(|&a, &b| {
            tc((g.w.trees[a].x, g.w.trees[a].z))
                .distance(g.me.pos)
                .total_cmp(&tc((g.w.trees[b].x, g.w.trees[b].z)).distance(g.me.pos))
        }) {
            approach(g, c, tc((g.w.trees[ti].x, g.w.trees[ti].z))).await;
            press(g, c, KeyCode::E).await;
            if let Some(f) =
                g.w.ground
                    .iter()
                    .filter(|i| i.def == g.w.native)
                    .map(|i| vec2(i.x, i.z))
                    .min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos)))
            {
                approach(g, c, f).await;
                press(g, c, KeyCode::E).await
            }
        }
        log!("pockets: {:?}", g.w.p.inv);
        for _ in 0..1500 {
            let np = g.npcs[0].pos;
            if !g.npcs[0].inside && g.me.pos.distance(np) < 1.1 {
                break;
            }
            let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default();
            let wp = if path.len() > 1 { tc(path[0]) } else { np };
            step(g, c, &keys_toward(wp - g.me.pos), &[]).await
        }
        face(g, c, g.npcs[0].pos).await;
        let f0 = g.w.p.npcs[0].friend;
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::G).await;
        log!("ui={:?}", g.me.ui);
        press(g, c, KeyCode::Enter).await;
        log!("friendship {f0} -> {} gifted={} ui={:?}", g.w.p.npcs[0].friend, g.w.p.npcs[0].gifted, g.me.ui);
        shot!("p63_gift_reaction");
    } else if phase == "12" {
        // P7-02: Harvest Fair stall (dev clock inside the provisional window)
        approach(g, c, tc(STALL)).await;
        log!("fair={} target={:?}", g.fair(), g.target());
        press(g, c, KeyCode::E).await;
        log!("prize: inv={:?} fair_year={}", g.w.p.inv, g.w.p.fair_year);
        press(g, c, KeyCode::E).await;
        log!("second try inv len={}", g.w.p.inv.len());
        step(g, c, &[KeyCode::S], &[]).await;
        for _ in 0..12 {
            step(g, c, &[KeyCode::S], &[]).await
        }
        face(g, c, tc(STALL)).await;
        shot!("p72_fair");
    } else if phase == "13" {
        // C-01: walk off the beach, swim to a bubble, dive
        go(g, c, vec2(13.5, 25.4), 0.15).await;
        for _ in 0..40 {
            if g.swimming() {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        for _ in 0..15 {
            step(g, c, &[KeyCode::S], &[]).await
        }
        log!("swimming={} pos={:?}", g.swimming(), g.me.pos);
        for _ in 0..600 {
            if !g.bubbles.is_empty() {
                break;
            }
            step(g, c, &[], &[]).await
        }
        if let Some(b) =
            g.bubbles.iter().map(|b| b.0).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos)))
        {
            for _ in 0..600 {
                if g.me.pos.distance(b) < 0.9 {
                    break;
                }
                let k = keys_toward(b - g.me.pos);
                step(g, c, &k, &[]).await
            }
        }
        shot!("c01_swimming");
        press(g, c, KeyCode::Space).await;
        log!("after dive: inv={:?} bubbles={}", g.w.p.inv, g.bubbles.len());
    } else if phase == "14" {
        // C-02: resident visit while at home (friendship 5 set as a logged FIXTURE; the visit itself is the normal provisional trigger)
        for n in &mut g.w.p.npcs {
            n.friend = 5
        }
        log!("FIXTURE: friendship 5 with all residents");
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        for _ in 0..3000 {
            if g.visit.is_some() {
                break;
            }
            step(g, c, &[], &[]).await
        }
        log!("visit={:?}", g.visit);
        if let Some((_, b, _)) = g.visit {
            let s = g.visit_spot(b);
            approach(g, c, s).await;
            press(g, c, KeyCode::E).await;
            log!("talk ui={:?}", g.me.ui);
            shot!("c02_visit")
        }
    } else if phase == "15" {
        // R15: tool effects captured right after use (net swish, rod cast splash)
        for _ in 0..4 {
            if g.w.p.equip == Some(60) {
                break;
            }
            press(g, c, KeyCode::Tab).await
        }
        go(g, c, vec2(11.5, 16.5), 0.12).await;
        face(g, c, vec2(11.5, 15.)).await;
        step(g, c, &[KeyCode::Space], &[KeyCode::Space]).await;
        log!("fx after net: {:?}", g.fx);
        shot!("fx_net_swish");
        for _ in 0..4 {
            if g.w.p.equip == Some(61) {
                break;
            }
            press(g, c, KeyCode::Tab).await
        }
        go(g, c, vec2(23.5, 5.5), 0.12).await;
        face(g, c, vec2(25., 5.5)).await;
        step(g, c, &[KeyCode::Space], &[KeyCode::Space]).await;
        log!("fx after cast: {:?} fishing={}", g.fx, g.me.fish.is_some());
        shot!("fx_cast_splash");
    } else if phase == "16" {
        // C-03b render check: reserve characters placed by a logged FIXTURE (the move workflow itself is covered headlessly)
        g.w.roster[0] = 4;
        g.w.roster[2] = 5;
        for k in [0, 2] {
            g.npcs[k].inside = true
        }
        log!("FIXTURE: roster {:?}", g.w.roster);
        wait(g, c, 90).await;
        for k in [0usize, 2] {
            for _ in 0..1500 {
                let np = g.npcs[k].pos;
                if !g.npcs[k].inside && g.me.pos.distance(np) < 1.6 {
                    break;
                }
                let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default();
                let wp = if path.len() > 1 { tc(path[0]) } else { np };
                step(g, c, &keys_toward(wp - g.me.pos), &[]).await
            }
            face(g, c, g.npcs[k].pos).await;
            press(g, c, KeyCode::E).await;
            log!("talk {}: {:?}", g.res(k).name, g.me.ui);
            shot!(format!("c03_{}", g.res(k).name));
            for _ in 0..5 {
                if g.me.ui == Ui::Play {
                    break;
                }
                press(g, c, KeyCode::E).await
            }
        }
    } else if phase == "17" {
        // C-04: buy furniture, customize tint + wallpaper + flooring at the counter, place it at home (coins: logged FIXTURE)
        g.w.p.money = 3000;
        log!("FIXTURE: money 3000");
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        let d = (0..5)
            .filter(|&i| !def(g.w.stock[i].def).wall)
            .max_by_key(|&i| (def(g.w.stock[i].def).act == Act::Sit) as u8)
            .unwrap();
        approach(g, c, tc(DISPLAYS[d])).await;
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::Enter).await;
        log!("bought {}", def(g.w.stock[d].def).name);
        go(g, c, vec2(3.5, 2.5), 0.12).await;
        face(g, c, vec2(3.5, 1.5)).await;
        press(g, c, KeyCode::E).await;
        for _ in 0..g.w.p.inv.len() + 2 {
            press(g, c, KeyCode::S).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("ui={:?}", g.me.ui);
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::S).await;
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::S).await;
        press(g, c, KeyCode::Enter).await;
        shot!("c04_customize_menu");
        press(g, c, KeyCode::Escape).await;
        log!("tints={:?} wall={} floor={} money={}", g.w.p.tints, g.w.p.wall, g.w.p.floor, g.w.p.money);
        go(g, c, vec2(3.5, 4.5), 0.12).await;
        for _ in 0..30 {
            if g.me.scene == Scene::Town {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        go(g, c, vec2(2.5, 3.5), 0.1).await;
        face(g, c, vec2(2.5, 1.)).await;
        press(g, c, KeyCode::I).await;
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::A).await;
        press(g, c, KeyCode::W).await;
        press(g, c, KeyCode::Enter).await;
        log!("room={:?}", g.w.p.room);
        go(g, c, vec2(3.4, 3.3), 0.15).await;
        face(g, c, vec2(3.4, 4.)).await;
        shot!("c04_custom_home");
    } else if phase == "18" {
        // C-05: P1 buys and wears a hat and changes hairstyle at the rack; P2 joins (default look) for comparison
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        approach(g, c, tc(RACK)).await;
        press(g, c, KeyCode::E).await;
        for _ in 0..4 {
            press(g, c, KeyCode::S).await
        }
        press(g, c, KeyCode::Enter).await; // Straw Hat
        for _ in 0..3 {
            press(g, c, KeyCode::S).await
        }
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::Escape).await; // hairstyle x2
        let idx = g.w.p.inv.iter().position(|s| is_hat(s.def)).unwrap_or(0);
        press(g, c, KeyCode::I).await;
        for _ in 0..idx {
            press(g, c, KeyCode::S).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("hat={:?} hair={} money={}", g.w.p.hat, g.w.p.hair, g.w.p.money);
        go(g, c, vec2(3.5, 4.5), 0.12).await;
        for _ in 0..30 {
            if g.me.scene == Scene::Town {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        g.join("Player 2");
        g.activate(1);
        g.me.pos = g.ids().iter().map(|_| door_stand(SHOP) + vec2(1., 0.)).next().unwrap();
        g.me.cam = g.me.pos;
        g.activate(0);
        log!("P2 placed beside P1 (test positioning)");
        for _ in 0..10 {
            step2(g, c, Inp::none(), Inp::none()).await
        }
        face(g, c, door_stand(SHOP) + vec2(0., 2.)).await;
        shot!("c05_looks");
    } else if phase == "19" {
        // X-01: gather branches (trees) and stones (rocks + shovel) normally, build the proposed bench, sit on it
        log!("project={:?}", g.w.project);
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        approach(g, c, tc(DISPLAYS[5])).await;
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::Enter).await;
        go(g, c, vec2(3.5, 4.5), 0.12).await;
        for _ in 0..30 {
            if g.me.scene == Scene::Town {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        let have = |g: &Game, d: u16| g.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum::<u16>();
        for _ in 0..10 {
            if have(g, 80) >= 5 {
                break;
            }
            let Some(ti) = (0..g.w.trees.len())
                .filter(|&i| !g.w.branches.contains(&(g.w.trees[i].x, g.w.trees[i].z)))
                .min_by(|&a, &b| {
                    tc((g.w.trees[a].x, g.w.trees[a].z))
                        .distance(g.me.pos)
                        .total_cmp(&tc((g.w.trees[b].x, g.w.trees[b].z)).distance(g.me.pos))
                })
            else {
                break;
            };
            approach(g, c, tc((g.w.trees[ti].x, g.w.trees[ti].z))).await;
            press(g, c, KeyCode::E).await;
            while let Some(it) =
                g.w.ground
                    .iter()
                    .filter(|i| i.def == 80)
                    .map(|i| vec2(i.x, i.z))
                    .min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos)))
            {
                if it.distance(g.me.pos) > 4. {
                    break;
                }
                approach(g, c, it).await;
                press(g, c, KeyCode::E).await;
                if g.w.ground.iter().any(|i| vec2(i.x, i.z) == it) {
                    break;
                }
            }
        }
        log!("branches={}", have(g, 80));
        for _ in 0..4 {
            if g.w.p.equip == Some(62) {
                break;
            }
            press(g, c, KeyCode::Tab).await
        }
        let rocks: Vec<(i32, i32)> =
            (0..W * H).filter(|&i| g.map[i] == ROCK).map(|i| ((i % W) as i32, (i / W) as i32)).collect();
        for _ in 0..4 {
            if have(g, 81) >= 2 {
                break;
            }
            let Some(&rk) = rocks
                .iter()
                .filter(|r| !g.w.rocks_hit.contains(r))
                .min_by(|a, b| tc(**a).distance(g.me.pos).total_cmp(&tc(**b).distance(g.me.pos)))
            else {
                break;
            };
            approach(g, c, tc(rk)).await;
            press(g, c, KeyCode::Space).await
        }
        log!("stones={}", have(g, 81));
        if let Some(p) = g.w.project.clone() {
            approach(g, c, tc((p.x, p.z))).await;
            shot!("x01_site_before");
            press(g, c, KeyCode::E).await;
            log!("after contribute: project={:?} built={:?} ui={:?}", g.w.project, g.w.built, g.me.ui);
            for _ in 0..4 {
                if g.me.ui == Ui::Play {
                    break;
                }
                press(g, c, KeyCode::E).await
            }
        }
        if let Some(b) = g.w.built.first().copied() {
            approach(g, c, tc((b.x, b.z))).await;
            press(g, c, KeyCode::E).await;
            log!("pose={:?}", g.me.pose);
            wait(g, c, 30).await;
            shot!("x01_bench_built")
        }
    } else if phase == "37" {
        // LAN host: wait for a client process, drive Player 2 from its input, stream snapshots, screenshot the host view.
        let mut h = net::Host::start_on(&g.w.p.name, 47790).expect("host");
        let t0 = std::time::Instant::now();
        let mut remote_snd = vec![];
        while t0.elapsed().as_secs() < 30 {
            let evs = h.poll();
            for e in evs {
                if let net::HostEv::Joined = e {
                    if !g.ids().contains(&1) {
                        g.join("Player 2");
                    }
                    g.remote = Some(1);
                    h.welcome(1);
                    log!("client joined")
                }
            }
            if g.remote.is_some() {
                let ids = g.ids();
                for id in ids {
                    g.activate(id);
                    if g.remote == Some(id) {
                        let i = h.take_input();
                        let b = g.snd.len();
                        g.update_player(&i, 1. / 60.);
                        remote_snd.extend(g.snd.drain(b..))
                    }
                }
                g.activate(0);
                if h.due() {
                    let s = net::snap(&g, c.now(), std::mem::take(&mut remote_snd));
                    h.send(s)
                }
            }
            step(g, c, &[], &[]).await;
            if g.remote.is_some() && t0.elapsed().as_secs() > 14 {
                break;
            }
        }
        g.activate(1);
        log!("remote P2 pos={:?} scene={:?}", g.me.pos, g.me.scene);
        g.activate(0);
        log!("host P1 pos={:?}", g.me.pos);
        shot!("lan_host_view");
    } else if phase == "38" {
        // LAN client: connect, walk Player 2 south via input messages, render from snapshots.
        let mut cl = None;
        for _ in 0..100 {
            if let Ok(x) = net::Client::connect("127.0.0.1:47790".parse().unwrap()) {
                cl = Some(x);
                break;
            }
            wait(g, c, 6).await
        }
        let mut cl = cl.expect("connect");
        let mut cg: Option<Game> = None;
        let mut now = c.now();
        let t0 = std::time::Instant::now();
        while t0.elapsed().as_secs() < 10 {
            cl.poll();
            if let Some(_id) = cl.id {
                let keys = if t0.elapsed().as_millis() > 1500 && t0.elapsed().as_millis() < 4000 {
                    vec![KeyCode::S]
                } else {
                    vec![]
                };
                cl.send(&Inp::keys(&keys, &[]), false)
            }
            if let (Some(id), Some(s)) = (cl.id, cl.take_snap()) {
                let gg = cg.get_or_insert_with(|| Game::from_world(s.w.clone(), town()));
                now = net::apply(gg, s, id).0
            }
            if let Some(gg) = &mut cg {
                render::draw(gg, now, None)
            }
            next_frame().await
        }
        let gg = cg.as_mut().expect("snapshots");
        log!("client view: me id={} pos={:?} remote={:?} coins={}", gg.me.id, gg.me.pos, gg.remote, gg.w.p.money);
        render::draw(gg, now, None);
        screenshot(&format!("{dir}/lan_client_view.png"));
        next_frame().await;
        cl.bye();
    } else if phase == "36" {
        // M2-3: repaint through Hazel's menu, see the house outside; home score on the Friends page. Fixture: coins.
        g.w.p.money = 3000;
        let th = BLDS.iter().position(|b| b.kind == BK::Civic).unwrap();
        go(g, c, door_stand(th), 0.12).await;
        face(g, c, door_face(th)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        approach(g, c, vec2(3.5, 1.5)).await;
        press(g, c, KeyCode::E).await;
        for _ in 0..3 {
            press(g, c, KeyCode::Down).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("ui={:?}", g.me.ui);
        press(g, c, KeyCode::Down).await;
        press(g, c, KeyCode::Down).await;
        shot!("m2_paint_menu");
        press(g, c, KeyCode::Enter).await;
        log!("paint={:?} money={}", g.w.p.paint, g.w.p.money);
        go(g, c, vec2(3.5, 4.4), 0.15).await;
        for _ in 0..40 {
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        go(g, c, door_stand(HOME) + vec2(0., 2.5), 0.3).await;
        face(g, c, door_face(HOME)).await;
        shot!("m2_house_painted");
        press(g, c, KeyCode::J).await;
        for _ in 0..5 {
            press(g, c, KeyCode::Tab).await
        }
        shot!("m2_home_score");
        press(g, c, KeyCode::Escape).await;
    } else if phase == "35" {
        // M2-1/M2-2: keeper menu, buy seeds + can through the menu, plant and water through normal input; fixture adds grown plants for the stage frame.
        g.w.p.money = 6000;
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        approach(g, c, vec2(3.5, 1.5)).await;
        log!("target={:?}", g.target());
        press(g, c, KeyCode::E).await;
        for _ in 0..3 {
            press(g, c, KeyCode::Down).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("ui={:?}", g.me.ui);
        shot!("m2_keeper_menu");
        press(g, c, KeyCode::Down).await;
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::Down).await;
        press(g, c, KeyCode::Down).await;
        press(g, c, KeyCode::Down).await;
        press(g, c, KeyCode::Down).await;
        press(g, c, KeyCode::Enter).await;
        log!(
            "money={} tools={:?} inv={:?}",
            g.w.p.money,
            g.w.p.tools,
            g.w.p.inv.iter().map(|s| (def(s.def).name, s.qty)).collect::<Vec<_>>()
        );
        press(g, c, KeyCode::Escape).await;
        go(g, c, vec2(3.5, 4.4), 0.15).await;
        for _ in 0..40 {
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        go(g, c, vec2(15.5, 20.5), 0.15).await;
        face(g, c, vec2(16.5, 20.5)).await;
        log!("target={:?}", g.target());
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::E).await;
        log!("garden={:?}", g.w.garden);
        for _ in 0..4 {
            if g.w.p.equip == Some(CAN) {
                break;
            }
            press(g, c, KeyCode::Tab).await
        }
        press(g, c, KeyCode::Space).await;
        wait(g, c, 10).await;
        log!("after water: {:?}", g.w.garden);
        for (k, (dx, dz, kind, stage)) in
            [(2, -1, 1u8, 2u8), (3, -1, 2, 2), (2, 1, 3, 1), (3, 1, 4, 2), (4, 0, 0, 2)].into_iter().enumerate()
        {
            let _ = k;
            g.w.garden.push(Plant { x: 15 + dx, z: 20 + dz, kind, stage, watered: 0 })
        }
        log!("FIXTURE: grown plants added for the stage frame");
        go(g, c, vec2(16.5, 22.0), 0.2).await;
        face(g, c, vec2(17.5, 20.5)).await;
        shot!("m2_garden");
    } else if phase == "34" {
        // Visual gap fixes from ordinary cameras: riverside fences + Regatta boats near the river at night (split-screen), hobby props on each resident.
        g.join("Player 2");
        let c2 = Clock {
            dev: Some((
                chrono::NaiveDate::from_ymd_opt(2026, 7, 25).unwrap().and_hms_opt(21, 0, 0).unwrap(),
                std::time::Instant::now(),
            )),
        };
        let c = &c2;
        wait(g, c, 5).await;
        let riv = (0..H as i32)
            .flat_map(|z| (0..W as i32).map(move |x| (x, z)))
            .find(|&(x, z)| tile_at(&g.map, x, z) == RIVER && g.walk_tile(x - 1, z))
            .unwrap();
        go(g, c, tc((riv.0 - 1, riv.1)) + vec2(-0.6, 1.0), 0.3).await;
        face(g, c, tc(riv)).await;
        log!("fest={:?} river {:?} pos={:?}", g.fest(), riv, g.me.pos);
        render::draw(g, c.now(), None);
        screenshot(&format!("{dir}/fix_regatta_fences_split.png"));
        next_frame().await;
        g.leave(1);
        let dd = (1..28)
            .map(|d| chrono::NaiveDate::from_ymd_opt(2026, 11, d).unwrap())
            .filter(|d| d.day() > 16)
            .find(|d| weather(d.num_days_from_ce() as i64, 11) == Wx::Clear)
            .unwrap();
        let c3 = Clock { dev: Some((dd.and_hms_opt(11, 0, 0).unwrap(), std::time::Instant::now())) };
        let c = &c3;
        wait(g, c, 5).await;
        go(g, c, vec2(16.5, 17.5), 0.3).await;
        log!("weather={:?} day={} dd={} mo={}", weather(g.w.day, 11), g.w.day, dd, g.mo);
        for (k, j) in (0..4).enumerate() {
            let pp = g.me.pos;
            let n = &mut g.npcs[j];
            n.inside = false;
            n.pos = pp + vec2(-2.4 + k as f32 * 1.6, -1.4);
            n.moving = false;
            n.path.clear();
            n.yaw = 0.;
        }
        g.me.pos += vec2(0., 1.2);
        g.w.p.npcs.iter_mut().for_each(|n| n.talked = true);
        g.t = 1.3;
        render::draw(g, c.now(), None);
        screenshot(&format!("{dir}/fix_props.png"));
        next_frame().await;
        log!("FIXTURE: residents placed idle in a row for the prop frame");
    } else if phase == "33" {
        // D-05 renovated shop (fixture: spending set to tier 2, then a day passes) and D-06 smoke/birds (dev clock 8:00 on a clear day).
        g.w.spent = SHOP_TIERS[1];
        let next = (c.now() + chrono::Duration::days(1)).date().and_hms_opt(8, 0, 0).unwrap();
        let c2 = Clock { dev: Some((next, std::time::Instant::now())) };
        let c = &c2;
        macro_rules! shotc {
            ($n:expr) => {{
                wait(g, c, 2).await;
                render::draw(g, c.now(), None);
                screenshot(&format!("{dir}/{}.png", $n));
                next_frame().await;
                log!("shot {}", $n);
            }};
        }
        wait(g, c, 5).await;
        log!("tier={} stock={} weather={:?}", g.shop_tier(), g.w.stock.len(), weather(g.w.day, c.now().month()));
        go(g, c, vec2(16.5, 17.5), 0.3).await;
        shotc!("d06_town_morning");
        for (k, j) in (0..4).enumerate() {
            let pp = g.me.pos;
            let n = &mut g.npcs[j];
            n.inside = false;
            n.pos = pp + vec2(-1.2 + k as f32 * 0.8, -1.0);
            n.moving = false;
            n.path.clear();
            n.yaw = 0.;
        }
        g.t = 1.3;
        render::draw(g, c.now(), None);
        screenshot(&format!("{dir}/d04_props.png"));
        next_frame().await;
        log!("FIXTURE: residents placed idle near the player for the prop frame");
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        go(g, c, vec2(3.5, 2.6), 0.15).await;
        shotc!("d05_shop_renovated");
    } else if phase == "32" {
        // D-03 surprises (fixture placement near the player) and D-04 emotes (fixed times).
        let here = tile_of(g.me.pos);
        for (k, kind) in [0u8, 1, 2].into_iter().enumerate() {
            let t = (here.0 + 1, here.1 + 1);
            g.w.surprise = Some(Surprise { kind, x: t.0, z: t.1, owner: 1, day: g.w.day });
            wait(g, c, 3).await;
            render::draw(g, c.now(), None);
            screenshot(&format!("{dir}/d03_surprise_{k}.png"));
            next_frame().await;
            log!("surprise {kind} at {:?}", t)
        }
        g.w.surprise = None;
        let j = (0..4).find(|&j| !g.npcs[j].inside).unwrap_or(0);
        let np = g.npcs[j].pos;
        go(g, c, np + vec2(1.6, 0.8), 0.3).await;
        for (k, tt) in [(0, 0.5f32), (1, 3.0)] {
            g.t = tt;
            render::draw(g, c.now(), None);
            screenshot(&format!("{dir}/d04_emote_{k}.png"));
            next_frame().await;
            log!("emote frame {k}: npc {j} emote={:?} moving={}", render::emote(g, j, tt), g.npcs[j].moving)
        }
    } else if phase == "31" {
        // Depth pass: festival decorations on each date (dev clock), Friends journal page.
        let dates = [(2026, 4, 11, 14), (2026, 7, 25, 21), (2026, 12, 22, 18), (2027, 1, 1, 11)];
        for (k, (y, m, d, h)) in dates.into_iter().enumerate() {
            let c2 = Clock {
                dev: Some((
                    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap().and_hms_opt(h, 0, 0).unwrap(),
                    std::time::Instant::now(),
                )),
            };
            let c = &c2;
            wait(g, c, 3).await;
            g.me.ui = Ui::Play;
            go(g, c, tc(STALL) + vec2(0., 2.2), 0.3).await;
            face(g, c, tc(STALL)).await;
            wait(g, c, 3).await;
            log!("fest={:?} target={:?}", g.fest(), g.target());
            render::draw(g, c.now(), None);
            screenshot(&format!("{dir}/d02_fest_{k}.png"));
            next_frame().await;
        }
        for k in 0..4 {
            g.w.p.npcs[k].friend = [3, 12, 33, 55][k]
        }
        press(g, c, KeyCode::J).await;
        for _ in 0..5 {
            press(g, c, KeyCode::Tab).await
        }
        shot!("d01_friends_page");
        press(g, c, KeyCode::Escape).await;
    } else if phase == "30" {
        // Release 1: title, campground (both players), kiosk, journal pages. Fixture: coins and coupons set directly; dev clock 12:00.
        let c2 = Clock {
            dev: Some((
                chrono::NaiveDate::from_ymd_opt(2026, 11, 20).unwrap().and_hms_opt(12, 0, 0).unwrap(),
                std::time::Instant::now(),
            )),
        };
        let c = &c2;
        macro_rules! shotc {
            ($n:expr) => {{
                wait(g, c, 2).await;
                render::draw(g, c.now(), None);
                screenshot(&format!("{dir}/{}.png", $n));
                next_frame().await;
                log!("shot {}", $n);
            }};
        }
        wait(g, c, 5).await;
        g.me.ui = Ui::Title(0);
        shotc!("r1_title");
        for _ in 0..3 {
            press(g, c, KeyCode::Down).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("ui={:?}", g.me.ui);
        shotc!("r1_help");
        press(g, c, KeyCode::Escape).await;
        for _ in 0..3 {
            press(g, c, KeyCode::Up).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("ui={:?}", g.me.ui);
        g.join("Player 2");
        g.w.p.money = 5000;
        log!(
            "visitor={} tasks={:?}",
            VISITORS[g.visitor()].name,
            g.w.p.tasks.iter().map(|t| TASKS[t.t].2).collect::<Vec<_>>()
        );
        approach(g, c, vec2(CAMP_SPOT.0, CAMP_SPOT.1)).await;
        log!("target={:?}", g.target());
        press(g, c, KeyCode::E).await;
        shotc!("r1_camp_menu");
        press(g, c, KeyCode::Enter).await;
        log!("camp_sold={:?} money={}", g.w.camp_sold, g.w.p.money);
        press(g, c, KeyCode::Escape).await;
        g.activate(1);
        g.w.p.money = 5000;
        go(g, c, vec2(30.5, 22.5), 0.2).await;
        face(g, c, vec2(CAMP_SPOT.0, CAMP_SPOT.1)).await;
        approach(g, c, vec2(CAMP_SPOT.0, CAMP_SPOT.1)).await;
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::Enter).await;
        log!("P2 try sold item: msgs={:?}", g.me.msgs.last().map(|m| m.0.clone()));
        press(g, c, KeyCode::Escape).await;
        g.activate(0);
        go(g, c, vec2(31.5, 23.0), 0.2).await;
        shotc!("r1_camp_both");
        let th = BLDS.iter().position(|b| b.kind == BK::Civic).unwrap();
        go(g, c, door_stand(th), 0.12).await;
        face(g, c, door_face(th)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        g.w.p.coupons = 20;
        approach(g, c, tc(KIOSK)).await;
        log!("target={:?}", g.target());
        press(g, c, KeyCode::E).await;
        shotc!("r1_kiosk");
        for _ in 0..4 {
            press(g, c, KeyCode::Down).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("coupons={} inv={:?}", g.w.p.coupons, g.w.p.inv.iter().map(|s| def(s.def).name).collect::<Vec<_>>());
        press(g, c, KeyCode::Escape).await;
        press(g, c, KeyCode::J).await;
        press(g, c, KeyCode::Tab).await;
        shotc!("r1_journal_initiatives");
        press(g, c, KeyCode::Tab).await;
        press(g, c, KeyCode::Tab).await;
        shotc!("r1_journal_controls");
        press(g, c, KeyCode::Tab).await;
        shotc!("r1_journal_guide");
        press(g, c, KeyCode::Tab).await;
        shotc!("r1_journal_collection_split");
    } else if phase == "29" {
        // X-06: running dust, building siding/trim, Town Hall exterior without the boarding plank
        let th = BLDS.iter().position(|b| b.kind == BK::Civic).unwrap();
        go(g, c, vec2(15.5, 17.5), 0.2).await;
        for _ in 0..14 {
            step(g, c, &[KeyCode::D, KeyCode::LeftShift], &[]).await
        }
        log!("spd={:.1} pos={:?} ui={:?} scene={:?}", g.me.spd, g.me.pos, g.me.ui, g.me.scene);
        render::draw(g, c.now(), None);
        screenshot(&format!("{dir}/x06_run_dust.png"));
        next_frame().await;
        go(g, c, door_stand(th) + vec2(0., 1.0), 0.2).await;
        face(g, c, door_face(th)).await;
        shot!("x06_townhall_ext");
    } else if phase == "28" {
        // X-04: Saturday art stall purchase, art wing donation + labels, furniture collections in the home. Fixture: coins and room furniture set directly; dev clock set to Saturday 2026-11-21.
        let c2 = Clock {
            dev: Some((
                chrono::NaiveDate::from_ymd_opt(2026, 11, 21).unwrap().and_hms_opt(12, 0, 0).unwrap(),
                std::time::Instant::now(),
            )),
        };
        let c = &c2;
        macro_rules! shotc {
            ($n:expr) => {{
                wait(g, c, 2).await;
                render::draw(g, c.now(), None);
                screenshot(&format!("{dir}/{}.png", $n));
                next_frame().await;
                log!("shot {}", $n);
            }};
        }
        wait(g, c, 5).await;
        g.w.p.money = 5000;
        log!("art_open={} week art={}", g.art_open(), def(art_of_week(g.w.day)).name);
        approach(g, c, tc(STALL)).await;
        log!("target={:?}", g.target());
        shotc!("x04_art_stall");
        press(g, c, KeyCode::E).await;
        log!("money={} inv={:?}", g.w.p.money, g.w.p.inv.iter().map(|s| def(s.def).name).collect::<Vec<_>>());
        let mb = BLDS.iter().position(|b| b.kind == BK::Museum).unwrap();
        go(g, c, door_stand(mb), 0.12).await;
        face(g, c, door_face(mb)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        for d in [8u16, 9, 18, 19] {
            g.add(d);
        }
        log!("FIXTURE: added Rain Snail, Dune Beetle, Ice Smelt, Brackish Goby to pockets");
        approach(g, c, vec2(3.5, 1.5)).await;
        press(g, c, KeyCode::E).await;
        for (n, t) in [("x06_frame_a", 4.15f32), ("x06_frame_b", 5.0), ("x06_frame_c", 8.45)] {
            g.t = t;
            render::draw(g, c.now(), None);
            screenshot(&format!("{dir}/{n}.png"));
            next_frame().await;
            log!("frame {n} t={t}")
        }
        for _ in 0..8 {
            press(g, c, KeyCode::E).await
        }
        press(g, c, KeyCode::Escape).await;
        log!("donated critters={:?}", [8u16, 9, 18, 19].map(|d| g.w.donated.iter().any(|x| x.0 == d)));
        go(g, c, vec2(3.5, 3.3), 0.12).await;
        shotc!("x04_museum_exhibits");
        face(g, c, vec2(3.5, 4.5)).await;
        for (n, t) in [("x06_player_open", 9.5f32), ("x06_player_blink", 8.45)] {
            g.t = t;
            render::draw(g, c.now(), None);
            screenshot(&format!("{dir}/{n}.png"));
            next_frame().await;
            log!("frame {n} t={t} face={:?}", g.me.face)
        }
        log!("donated art={} ui={:?}", g.donated_of(K::Art), g.me.ui);
        go(g, c, vec2(2.6, 2.4), 0.12).await;
        face(g, c, vec2(2.6, 0.5)).await;
        shotc!("x04_art_wing");
        go(g, c, vec2(3.5, 4.4), 0.15).await;
        for _ in 0..40 {
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        log!("scene={:?}", g.me.scene);
        g.w.p.home = 2;
        g.w.p.debt = 0;
        g.w.p.room.clear(); // fixture: largest home, cleared, then every piece placed through place_ok
        for (k, (d, x, z)) in [
            (90u16, 0, 4),
            (91, 1, 4),
            (92, 4, 4),
            (110, 5, 3),
            (111, 5, 0),
            (112, 1, -1),
            (93, 0, 0),
            (94, 2, 0),
            (95, 3, 0),
            (113, 4, 0),
            (114, 5, 2),
            (115, 0, 2),
            (112, 1, -1),
        ]
        .into_iter()
        .enumerate()
        .take(12)
        {
            let f = Furn { uid: 5000 + k as u32, def: d, x, z, rot: 0, on: true };
            g.me.ipos = vec2(2.5, 2.5);
            match g.place_ok(&f) {
                Ok(()) => g.w.p.room.push(f),
                Err(e) => log!("fixture place {} at {},{} rejected: {e}", def(d).name, x, z),
            }
        }
        log!("sets_done={:?}", g.sets_done());
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        shotc!("x04_home_collections");
    } else if phase == "25" {
        // Town Hall interior + money project: pick at Hazel's desk, invalid/valid preview, partial funding, P2 tops up, rollover builds it.
        // Fixture: coins are set directly so the project can be funded within one run.
        g.join("Player 2");
        let th = BLDS.iter().position(|b| b.kind == BK::Civic).unwrap();
        go(g, c, door_stand(th), 0.12).await;
        face(g, c, door_face(th)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        log!("scene={:?}", g.me.scene);
        go(g, c, vec2(4.5, 3.5), 0.15).await;
        face(g, c, vec2(3.5, 1.5)).await;
        shot!("th_interior");
        approach(g, c, vec2(3.5, 1.5)).await;
        log!("target={:?}", g.target());
        press(g, c, KeyCode::E).await;
        log!("ui={:?}", g.me.ui);
        shot!("th_civic_menu");
        press(g, c, KeyCode::Down).await;
        press(g, c, KeyCode::Enter).await;
        log!("ui={:?}", g.me.ui);
        press(g, c, KeyCode::Enter).await;
        wait(g, c, 40).await;
        log!("mplace={:?} scene={:?}", g.me.mplace, g.me.scene);
        go(g, c, vec2(13.5, 17.5), 0.12).await;
        face(g, c, vec2(13.5, 18.5)).await;
        shot!("place_invalid");
        press(g, c, KeyCode::Enter).await;
        log!("invalid attempt: mproject={:?}", g.w.mproject.is_some());
        go(g, c, vec2(16.5, 20.5), 0.12).await;
        face(g, c, vec2(17.5, 20.5)).await;
        shot!("place_valid");
        press(g, c, KeyCode::Enter).await;
        let m = g.w.mproject.clone();
        log!("placed={:?}", m);
        let site = m.map(|m| tc((m.x, m.z))).unwrap_or_default();
        g.w.p.money = 3000;
        approach(g, c, site).await;
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::Enter).await;
        press(g, c, KeyCode::Enter).await;
        shot!("project_partial_fund_ui");
        press(g, c, KeyCode::Escape).await;
        wait(g, c, 5).await;
        shot!("project_partial");
        log!("after P1: funded={:?} P1 money={}", g.w.mproject.as_ref().map(|m| m.funded), g.w.p.money);
        g.activate(1);
        g.w.p.money = 9000;
        approach(g, c, site).await;
        press(g, c, KeyCode::E).await;
        for _ in 0..8 {
            press(g, c, KeyCode::Enter).await
        }
        press(g, c, KeyCode::Escape).await;
        log!(
            "after P2: funded={:?} P2 money={}",
            g.w.mproject.as_ref().map(|m| (m.funded, m.log.clone())),
            g.w.p.money
        );
        g.activate(0);
        let next = (c.now() + chrono::Duration::days(1)).date().and_hms_opt(6, 0, 30).unwrap();
        let c2 = Clock { dev: Some((next, std::time::Instant::now())) };
        wait(g, &c2, 30).await;
        log!(
            "after rollover: mproject={:?} built={:?}",
            g.w.mproject.is_some(),
            g.w.built.iter().map(|b| (b.kind, b.x, b.z)).collect::<Vec<_>>()
        );
        render::draw(g, c2.now(), None);
        screenshot(&format!("{dir}/project_built.png"));
        next_frame().await;
    } else if phase == "26" || phase == "27" {
        // 26: both players sail and play Crab Corral together. 27: P1 plays Ring Swim while P2 stays in town.
        g.join("Player 2");
        let both = phase == "26";
        for pid in if both { vec![0, 1] } else { vec![0] } {
            g.activate(pid);
            go(g, c, dock_stand(), 0.12).await;
            face(g, c, vec2(20.5, 26.)).await;
            press(g, c, KeyCode::E).await;
            press(g, c, KeyCode::Enter).await;
            wait(g, c, 40).await;
            log!("P{} scene={:?}", pid + 1, g.me.scene)
        }
        g.activate(0);
        approach(g, c, tc(ACT_BOARD)).await;
        press(g, c, KeyCode::E).await;
        if !both {
            press(g, c, KeyCode::Down).await;
            press(g, c, KeyCode::Down).await
        }
        press(g, c, KeyCode::Enter).await;
        log!("act kind={:?} players={:?}", g.act.as_ref().map(|a| a.kind), g.act.as_ref().map(|a| a.players.clone()));
        let pen = vec2(PEN.0, PEN.1);
        let mut shots = 0;
        for it in 0..80 {
            let Some(ents) = g.act.as_ref().filter(|a| !a.done).map(|a| (a.ents.clone(), a.next)) else { break };
            if both {
                let pid = it % 2;
                g.activate(pid);
                let Some(cr) = ents
                    .0
                    .iter()
                    .copied()
                    .filter(|&x| !Game::in_pen(x))
                    .min_by(|x, y| x.distance(g.ppos()).total_cmp(&y.distance(g.ppos())))
                else {
                    g.activate(0);
                    break;
                };
                let mut tgt = cr;
                for _ in 0..360 {
                    let Some(a) = g.act.as_ref().filter(|a| !a.done) else { break };
                    let Some(&cb) = a
                        .ents
                        .iter()
                        .filter(|&&x| !Game::in_pen(x))
                        .min_by(|x, y| x.distance(tgt).total_cmp(&y.distance(tgt)))
                    else {
                        break;
                    };
                    tgt = cb;
                    let dir = (pen - cb).normalize_or_zero();
                    let behind = cb - dir * 0.5;
                    let side = (g.ppos() - cb).dot(dir) > -0.2; // not yet behind: circle around at a distance
                    let want = if side { cb - dir * 0.5 + vec2(-dir.y, dir.x) * 1.0 } else { behind };
                    step(g, c, &keys_toward(want - g.ppos()), &[]).await
                }
                g.activate(0);
            } else {
                let Some(&r) = ents.0.get(ents.1) else { break };
                for f in 0..600 {
                    if g.act.as_ref().map_or(true, |a| a.next != ents.1 || a.done) {
                        break;
                    }
                    step(g, c, &keys_toward(r - g.ppos()), &[]).await;
                    if it == 1 && f == 40 {
                        shot!("ring_swim_approach")
                    }
                }
            }
            if shots == 0 && (it == 3 || g.act.as_ref().is_some_and(|a| a.score >= 1)) {
                shot!(if both { "crab_corral_coop" } else { "ring_swim_solo" });
                shots += 1
            }
        }
        log!("act end state={:?}", g.act.as_ref().map(|a| (a.score, a.done, a.left, a.mine.clone())));
        for _ in 0..30 {
            step2(g, c, Inp::none(), Inp::none()).await
        }
        let gl = |p: &Profile| p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>();
        log!(
            "P1 glass={} msg={:?} | P2 scene={:?} glass={}",
            gl(&g.w.p),
            g.me.msgs.last().map(|m| m.0.clone()),
            g.others.first().map(|o| o.scene),
            g.w.others.first().map_or(0, gl)
        );
    } else if phase == "24" {
        // Gullrock timed activity through the real input path: P1 sails, starts Beacon Dash at the board, chases beacons; P2 stays in town
        g.join("Player 2");
        go(g, c, dock_stand(), 0.12).await;
        face(g, c, vec2(20.5, 26.)).await;
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::Enter).await;
        wait(g, c, 40).await;
        log!("P1 scene={:?}", g.me.scene);
        approach(g, c, tc(ACT_BOARD)).await;
        log!("target={:?}", g.target());
        press(g, c, KeyCode::E).await;
        log!("ui={:?}", g.me.ui);
        press(g, c, KeyCode::Down).await;
        press(g, c, KeyCode::Enter).await;
        log!("act players={:?}", g.act.as_ref().map(|a| a.players.clone()));
        let mut shot_done = false;
        for _ in 0..120 {
            let Some(b) = g.act.as_ref().filter(|a| !a.done).map(|a| a.ents[0]) else { break };
            approach(g, c, b).await;
            wait(g, c, 2).await;
            if !shot_done && g.act.as_ref().is_some_and(|a| a.score >= 2) {
                shot!("act_beacon_dash");
                shot_done = true
            }
        }
        wait(g, c, 30).await;
        let gl: u16 = g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum();
        log!(
            "act={:?} sea_glass={gl} msgs={:?}",
            g.act.as_ref().map(|a| (a.score, a.done, a.left)),
            g.me.msgs.iter().rev().take(2).map(|m| m.0.clone()).collect::<Vec<_>>()
        );
        log!("P2 scene={:?} act_day={:?}", g.others.first().map(|o| o.scene), g.w.others.first().map(|o| o.act_day));
    } else if phase == "20" {
        // X-03: P1 sails to the islet (P2 stays in town), does today's task normally, gets Sea Glass, sails back
        g.join("Player 2");
        go(g, c, dock_stand(), 0.12).await;
        face(g, c, vec2(20.5, 26.)).await;
        log!("target={:?}", g.target());
        press(g, c, KeyCode::E).await;
        log!("ui={:?}", g.me.ui);
        press(g, c, KeyCode::Enter).await;
        wait(g, c, 40).await;
        log!("P1 scene={:?} money={}", g.me.scene, g.w.p.money);
        let (td, n, label) = islet_task(g.w.day);
        log!("task: {label}");
        for _ in 0..12 {
            let have: u16 = g.w.p.inv.iter().filter(|s| s.def == td).map(|s| s.qty).sum();
            if have >= n {
                break;
            }
            let Some(it) =
                g.w.iground
                    .iter()
                    .filter(|i| i.def == td)
                    .map(|i| vec2(i.x, i.z))
                    .min_by(|a, b| a.distance(g.me.ipos).total_cmp(&b.distance(g.me.ipos)))
            else {
                break;
            };
            approach(g, c, it).await;
            press(g, c, KeyCode::E).await
        }
        log!("pockets={:?}", g.w.p.inv);
        for _ in 0..10 {
            step2(g, c, Inp::none(), Inp::none()).await
        }
        shot!("x03_split_islet_town");
        approach(g, c, vec2(PELL_I.0, PELL_I.1)).await;
        press(g, c, KeyCode::E).await;
        log!("pell: {:?}", g.me.ui);
        for _ in 0..4 {
            if !matches!(g.me.ui, Ui::Talk { .. }) {
                break;
            }
            press(g, c, KeyCode::E).await
        }
        log!("sea glass={} ui={:?}", g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>(), g.me.ui);
        press(g, c, KeyCode::Enter).await;
        wait(g, c, 40).await;
        log!("back: scene={:?} pos={:?}", g.me.scene, g.me.pos);
    } else if phase == "21" {
        // X-05: wait for residents to start chatting on their own, walk over, capture the speech bubbles
        for _ in 0..7200 {
            if !g.chats.is_empty() {
                break;
            }
            step(g, c, &[], &[]).await
        }
        log!("chats={:?}", g.chats);
        if let Some(&(a, _, _)) = g.chats.first() {
            for _ in 0..600 {
                let np = g.npcs[a].pos;
                if g.me.pos.distance(np) < 2.5 || g.chats.is_empty() {
                    break;
                }
                let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default();
                let wp = if path.len() > 1 { tc(path[0]) } else { np };
                step(g, c, &keys_toward(wp - g.me.pos), &[]).await
            }
        }
        log!("still chatting={:?}", g.chats);
        shot!("x05_resident_chat");
    } else if phase == "22" {
        // X-06: dialogue framing in split-screen (P1 talks while P2 is joined)
        g.join("Player 2");
        let i = (0..4).find(|&i| !g.npcs[i].inside).unwrap_or(0);
        for _ in 0..1500 {
            let np = g.npcs[i].pos;
            if !g.npcs[i].inside && g.me.pos.distance(np) < 1.1 {
                break;
            }
            let path = g.bfs(tile_of(g.me.pos), tile_of(np), false).unwrap_or_default();
            let wp = if path.len() > 1 { tc(path[0]) } else { np };
            step2(g, c, Inp::keys(&keys_toward(wp - g.me.pos), &[]), Inp::none()).await
        }
        face(g, c, g.npcs[i].pos).await;
        step2(g, c, Inp::keys(&[KeyCode::E], &[KeyCode::E]), Inp::none()).await;
        step2(g, c, Inp::none(), Inp::none()).await;
        log!("P1 ui={:?}", g.me.ui);
        shot!("x06_split_dialogue");
    } else if phase == "23" {
        // audio: hourly crossfade (dev clock crosses 17:00), indoor arrangement switch, journal mute/volume
        let mut m = audio::Music::new();
        let mut frame = 0usize;
        macro_rules! tickn {
            ($n:expr, $d:expr, $p:expr) => {{
                for _ in 0..$n {
                    m.tick((c.now().hour(), audio::variant(g)), 1. / 60., g.music_vol, g.muted).await;
                    step(g, c, $d, $p).await;
                    frame += 1;
                    if frame % 60 == 0 {
                        log!(
                            "t={} cur={:?} fading_out={} applied={:.2}",
                            c.now().format("%H:%M:%S"),
                            m.current(),
                            m.fading_out(),
                            m.applied
                        )
                    }
                }
                for l in m.log.drain(..) {
                    log!("{l}")
                }
            }};
        }
        tickn!(600, &[], &[]);
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        tickn!(300, &[], &[]);
        log!("scene={:?} variant={:?}", g.me.scene, audio::variant(g));
        press(g, c, KeyCode::J).await;
        press(g, c, KeyCode::X).await;
        tickn!(10, &[], &[]);
        log!("after mute: muted={} applied={:.2}", g.muted, m.applied);
        press(g, c, KeyCode::X).await;
        press(g, c, KeyCode::Left).await;
        press(g, c, KeyCode::Down).await;
        tickn!(10, &[], &[]);
        log!(
            "after unmute/-music/-sfx: music_vol={:.1} sfx_vol={:.1} applied={:.2} dirty={}",
            g.music_vol,
            g.sfx_vol,
            m.applied,
            g.settings_dirty
        );
    } else if phase == "view" {
        // Reproduce a reported view: HOLLOWBROOK_VIEW="<Town|Frontier|Islet|Room:N> <x> <z> [facing degrees]".
        // Use the scene and pos from the F12 debug overlay of a bug screenshot (see tools/view.sh).
        let spec = std::env::var("HOLLOWBROOK_VIEW").unwrap_or_else(|_| "Town 15.5 15.5".into());
        let f: Vec<&str> = spec.split_whitespace().collect();
        let num = |i: usize, d: f32| f.get(i).and_then(|v| v.parse::<f32>().ok()).unwrap_or(d);
        let scene = match f.first().copied().unwrap_or("Town") {
            "Frontier" => Scene::Frontier,
            "Islet" => Scene::Islet,
            s if s.starts_with("Room:") => Scene::Room(s[5..].parse().unwrap_or(HOME)),
            _ => Scene::Town,
        };
        if scene != Scene::Town {
            g.swap(scene);
        }
        g.me.scene = scene;
        let p = vec2(num(1, 15.5), num(2, 15.5));
        let a = num(3, 180.).to_radians();
        if scene == Scene::Town {
            g.me.pos = p
        } else {
            g.me.ipos = p
        }
        g.me.cam = p;
        g.me.face = vec2(a.sin(), -a.cos());
        wait(g, c, 40).await;
        log!("view {spec}: scene={:?} pos={:?} target={:?}", g.me.scene, p, g.target());
        shot!("view");
    } else if phase == "48" {
        // Dustreach hunter life: full armour, companion, a prisoner walking to the ship, the guild master's forge.
        g.w.p.armor = 0b1111; // logged FIXTURE: all armour pieces
        g.w.p.pal = true;
        g.w.p.marks = 5;
        g.swap(Scene::Frontier);
        g.me.scene = Scene::Frontier;
        g.take_bounty(0).ok();
        if let Some(b) = g.w.p.bounty.as_mut() {
            b.caught = true; // logged FIXTURE: target already detained
            b.pos[0] = [F_RAMP.0 - 1.5, F_RAMP.1 - 2.5];
        }
        g.me.ipos = vec2(F_RAMP.0 - 1.0, F_RAMP.1 + 0.4);
        g.me.pal_pos = g.me.ipos + vec2(0.8, 0.3);
        g.me.cam = g.me.ipos;
        g.me.face = vec2(0.3, 1.).normalize();
        wait(g, c, 40).await;
        shot!("h_ship");
        g.me.ipos = vec2(13.6, 10.7);
        g.me.cam = g.me.ipos;
        g.me.face = vec2(0.6, -1.).normalize();
        wait(g, c, 20).await;
        press(g, c, KeyCode::E).await;
        shot!("h_forge");
        press(g, c, KeyCode::Escape).await;
        g.me.riding = true; // logged FIXTURE: on the hover-bike
        g.me.ipos = vec2(14.2, 13.4);
        g.me.cam = g.me.ipos;
        g.me.face = vec2(-1., -0.2).normalize();
        wait(g, c, 30).await;
        shot!("h_street");
        log!("ui={:?} target={:?}", g.me.ui, g.target());
    } else if phase == "47" {
        // The woods path and the door to Dustreach, then through it.
        g.me.pos = vec2(2.3, 17.5);
        g.me.cam = vec2(-2.5, 17.5);
        g.me.face = vec2(-1., 0.);
        wait(g, c, 30).await;
        shot!("w_path");
        g.me.pos = vec2(WOODS_DOOR.0 as f32 + 1.5, WOODS_DOOR.1 as f32 + 0.5);
        g.me.cam = g.me.pos;
        wait(g, c, 20).await;
        shot!("w_door");
        press(g, c, KeyCode::E).await;
        wait(g, c, 60).await;
        log!("scene after door: {:?}", g.me.scene);
    } else if phase == "46" {
        // Dustreach gear: buy through the board menu, fire the stun pistol at a runner, then the staff up close.
        g.w.p.money = 9000; // logged FIXTURE so the purchases are affordable
        g.swap(Scene::Frontier);
        g.me.scene = Scene::Frontier;
        g.me.ipos = vec2(13.6, 10.7);
        g.me.cam = g.me.ipos;
        g.me.face = vec2(0., -1.);
        wait(g, c, 20).await;
        press(g, c, KeyCode::E).await;
        for _ in 0..4 {
            press(g, c, KeyCode::S).await;
        }
        shot!("g_gear_menu");
        press(g, c, KeyCode::Enter).await;
        for _ in 0..3 {
            press(g, c, KeyCode::W).await;
        }
        press(g, c, KeyCode::Enter).await; // take the runner
        log!("equip={:?} money={} bounty={:?}", g.w.p.equip, g.w.p.money, g.w.p.bounty);
        let p = g.w.p.bounty.as_ref().map(|b| b.pos[0]).unwrap_or([15., 15.]);
        g.me.ipos = vec2(p[0], p[1] - 3.5);
        g.me.cam = g.me.ipos;
        g.me.face = vec2(0., 1.);
        wait(g, c, 20).await;
        press(g, c, KeyCode::Space).await;
        wait(g, c, 4).await;
        shot!("g_pistol_shot");
        wait(g, c, 20).await;
        shot!("g_stunned");
        log!("after shot: {:?}", g.w.p.bounty);
    } else if phase == "45" {
        // Dustreach: arrival, the street, the board menu through the real input path, a lookalike bounty up close.
        g.swap(Scene::Frontier);
        g.me.scene = Scene::Frontier;
        wait(g, c, 30).await;
        shot!("f_arrival");
        g.me.ipos = vec2(13.6, 10.7);
        g.me.cam = g.me.ipos;
        g.me.face = vec2(0., -1.);
        wait(g, c, 20).await;
        press(g, c, KeyCode::E).await;
        shot!("f_board_menu");
        press(g, c, KeyCode::S).await;
        press(g, c, KeyCode::S).await;
        press(g, c, KeyCode::Enter).await;
        log!("bounty={:?}", g.w.p.bounty);
        let p = g.w.p.bounty.as_ref().map(|b| b.pos[0]).unwrap_or([15., 15.]);
        g.me.ipos = vec2(p[0], p[1] + 2.4);
        g.me.cam = g.me.ipos;
        g.me.face = vec2(0., -1.);
        wait(g, c, 30).await;
        shot!("f_lookalikes");
        g.me.ipos = vec2(19.5, 12.5);
        g.me.cam = g.me.ipos;
        wait(g, c, 20).await;
        shot!("f_market");
        g.me.ipos = vec2(33.5, 25.2);
        g.me.cam = g.me.ipos;
        g.me.face = vec2(0., 1.);
        wait(g, c, 20).await;
        shot!("f_oasis");
    } else if phase == "44" {
        // Home stories: main floor with both stairways, then basement and upstairs.
        g.w.p.home = 4;
        g.swap(Scene::Room(HOME));
        g.me.scene = Scene::Room(HOME);
        wait(g, c, 30).await;
        shot!("h_main");
        g.take_stairs(1);
        wait(g, c, 30).await;
        shot!("h_basement");
        g.take_stairs(0);
        g.take_stairs(2);
        wait(g, c, 30).await;
        shot!("h_upstairs");
    } else if phase == "43" {
        g.join("Seek partner");
        g.activate(1);
        g.me.pos = vec2(13.5, 17.5);
        g.me.cam = g.me.pos;
        g.activate(0);
        g.me.pos = vec2(13.5, 17.5);
        g.me.cam = g.me.pos;
        g.me.face = vec2(0., 1.);
        g.w.p.npcs[0].friend = 5;
        g.npcs[0].inside = false;
        g.npcs[0].pos = g.me.pos + vec2(0., 0.9);
        g.npcs[0].wait = 60.;
        g.maybe_social_offer();
        press(g, c, KeyCode::E).await;
        shot!("m3_seek_invitation");
        press(g, c, KeyCode::Enter).await;
        assert!(g.w.p.seek.is_some(), "real invitation input starts a round");
        shot!("m3_seek_start");
        if g.hr == 16 {
            // Near-expiry fixture: demonstrate that entering a room does not pause the saved deadline.
            g.w.p.seek.as_mut().unwrap().deadline_ms = g.clock_ms + 700;
            g.me.pos = door_stand(HOME) + vec2(0., 0.35);
            g.me.face = vec2(0., -1.);
            press(g, c, KeyCode::E).await;
            wait(g, c, 90).await;
            assert!(g.w.p.seek.as_ref().unwrap().failed);
            assert_eq!(g.me.scene, Scene::Room(HOME));
            shot!("m3_seek_timeout_home");
        } else {
            let spots = g.w.p.seek.as_ref().unwrap().spots.clone();
            for (j, h) in spots.iter().enumerate() {
                for _ in 0..2 {
                    if go(g, c, tc(h.tile), 0.2).await {
                        break;
                    }
                }
                for _ in 0..2400 {
                    if g.npcs[h.slot].pos.distance(tc(h.tile)) < 0.25 {
                        break;
                    }
                    step(g, c, &[], &[]).await;
                }
                let pos = g.npcs[h.slot].pos;
                assert!(approach(g, c, pos).await, "reach hiding friend");
                press(g, c, KeyCode::E).await;
                assert!(
                    g.w.p.seek.as_ref().unwrap().found.contains(&h.slot),
                    "talk finds friend through normal targeting"
                );
                if j == 0 {
                    shot!("m3_seek_first_found");
                }
                if j == 2 {
                    shot!("m3_seek_complete");
                }
                press(g, c, KeyCode::Escape).await;
            }
            log!("personal completion={:?}; coins={}; partner={:?}", g.w.p.seek, g.w.p.money, g.w.others[0].seek);
        }
        press(g, c, KeyCode::Escape).await;
        g.me.jpage = 4;
        g.me.jcol = GUIDE.iter().position(|s| s.starts_with("Monthly surveys")).unwrap() as u8;
        press(g, c, KeyCode::J).await;
        shot!("m3_contest_calendar");
    } else if phase == "42" {
        g.join("Visit partner");
        g.activate(1);
        g.me.pos = vec2(13.5, 17.5);
        g.me.cam = g.me.pos;
        g.activate(0);
        if g.hr == 12 && c.now().day() == 12 {
            g.me.pos = vec2(13.5, 17.5);
            g.me.cam = g.me.pos;
            g.me.face = vec2(0., 1.);
            g.w.p.npcs[0].friend = 5;
            g.npcs[0].inside = false;
            g.npcs[0].pos = g.me.pos + vec2(0., 0.9);
            g.npcs[0].wait = 60.;
            g.maybe_social_offer();
            press(g, c, KeyCode::E).await;
            shot!("m3_invitation");
            press(g, c, KeyCode::Enter).await;
            log!("accepted={:?}", g.w.p.invitation);
            shot!("m3_appointment");
        } else if c.now().day() == 12 {
            g.w.p.invitation = Some(events::SocialInvite {
                day: g.w.day,
                slot: 0,
                character: g.w.roster[0],
                kind: 0,
                hour: 13,
                state: 1,
            });
            g.me.pos = door_stand(HOUSES[0]) + vec2(0., 0.35);
            g.me.cam = g.me.pos;
            g.me.face = vec2(0., -1.);
            press(g, c, KeyCode::E).await;
            wait(g, c, 40).await;
            shot!("m3_home_visit");
            log!("visit={:?} friend={} inv={:?}", g.w.p.invitation, g.w.p.npcs[0].friend, g.w.p.inv);
        } else {
            g.w.p.invitation = Some(events::SocialInvite {
                day: g.w.day - 1,
                slot: 0,
                character: g.w.roster[0],
                kind: 0,
                hour: 13,
                state: 1,
            });
            g.refresh_social();
            g.me.pos = vec2(13.5, 17.5);
            g.me.cam = g.me.pos;
            g.me.face = vec2(0., 1.);
            g.npcs[0].inside = false;
            g.npcs[0].pos = g.me.pos + vec2(0., 0.9);
            g.npcs[0].wait = 60.;
            press(g, c, KeyCode::E).await;
            shot!("m3_missed_visit");
        }
        press(g, c, KeyCode::Escape).await;
        g.me.jpage = 4;
        g.me.jcol = GUIDE.iter().position(|s| s.starts_with("Resident invitations")).unwrap() as u8;
        press(g, c, KeyCode::J).await;
        shot!("m3_social_guide");
    } else if phase == "41" {
        // Fixture catches; registration, entry selection and prize claim use the real input layer.
        g.join("Survey partner");
        let sp = vec2(events::CONTEST_SPOT.0, events::CONTEST_SPOT.1);
        for id in [0, 1] {
            g.activate(id);
            g.me.pos = sp + vec2(if id == 0 { -0.8 } else { 0.8 }, 0.7);
            g.me.cam = g.me.pos;
            g.me.face = (sp - g.me.pos).normalize();
        }
        g.activate(0);
        if g.hr < 17 {
            g.add_catch(if g.contest_kind() == Some(0) { 16 } else { 7 });
            press(g, c, KeyCode::E).await;
            shot!("m3_registration");
            press(g, c, KeyCode::Escape).await;
            press(g, c, KeyCode::E).await;
            shot!("m3_entry_menu");
            press(g, c, KeyCode::Enter).await;
            log!("P1 best={:?}; P2={:?}", g.w.p.contests, g.w.others[0].contests);
            shot!("m3_best_entry");
        } else {
            g.w.p.contests.push(events::ContestEntry {
                day: g.w.day,
                kind: g.contest_kind().unwrap(),
                best: 130,
                specimen: Some(16),
                size_mm: 500,
                claimed: false,
            });
            press(g, c, KeyCode::E).await;
            log!("prize received: money={} records={:?}", g.w.p.money, g.w.p.contests);
            shot!("m3_prize");
        }
    } else if phase == "40" {
        // Visual tour for presentation work (no gameplay changes): fixed viewpoints, compare before/after.
        shot!("v_start");
        go(g, c, vec2(13.5, 17.5), 0.15).await;
        face(g, c, vec2(13.5, 10.)).await;
        shot!("v_plaza_north");
        go(g, c, vec2(16.5, 23.4), 0.2).await;
        face(g, c, vec2(16.5, 10.)).await;
        shot!("v_beach_north");
        face(g, c, vec2(16.5, 30.)).await;
        shot!("v_beach_sea");
        go(g, c, vec2(22.5, 8.5), 0.15).await;
        face(g, c, vec2(26., 8.5)).await;
        shot!("v_river");
        go(g, c, vec2(6.5, 6.5), 0.15).await;
        face(g, c, vec2(6.5, 3.)).await;
        shot!("v_northwest");
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        shot!("v_shop");
        go(g, c, vec2(3.5, 4.5), 0.12).await;
        for _ in 0..30 {
            if g.me.scene == Scene::Town {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        go(g, c, vec2(3.4, 3.3), 0.15).await;
        face(g, c, vec2(3.4, 4.)).await;
        shot!("v_home");
    } else if phase == "0" {
        log!("load-only check");
    } else if phase == "3" && c.now().hour() < 18 {
        // P1-01..04: shovel purchase, digging, fishing, bugs, resident interior
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        approach(g, c, tc(DISPLAYS[5])).await;
        press(g, c, KeyCode::E).await;
        press(g, c, KeyCode::Enter).await;
        log!("shovel bought: tools={:?} money={}", g.w.p.tools, g.w.p.money);
        go(g, c, vec2(3.5, 4.5), 0.12).await;
        for _ in 0..30 {
            if g.me.scene == Scene::Town {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        for _ in 0..4 {
            if g.w.p.equip == Some(62) {
                break;
            }
            press(g, c, KeyCode::Tab).await
        }
        if let Some(site) = g.w.digs.first().copied() {
            approach(g, c, tc(site)).await;
            press(g, c, KeyCode::Space).await;
            log!("dig site {:?}: hole={} inv={:?}", site, g.w.holes.contains(&site), g.w.p.inv);
            shot!("p3_dig_site");
            press(g, c, KeyCode::Space).await;
            log!("filled again: hole={}", g.w.holes.contains(&site));
        }
        for _ in 0..4 {
            if g.w.p.equip == Some(61) {
                break;
            }
            press(g, c, KeyCode::Tab).await
        }
        go(g, c, vec2(23.5, 5.5), 0.12).await;
        face(g, c, vec2(25., 5.5)).await;
        'fish: for attempt in 0..8 {
            press(g, c, KeyCode::Space).await;
            log!(
                "cast {attempt}: fishing={} target_fish={:?} shadows={}",
                g.me.fish.is_some(),
                g.me.fish.as_ref().and_then(|f| f.fish),
                g.shadows.len()
            );
            for _ in 0..600 {
                if g.me.fish.is_none() {
                    break;
                }
                if g.me.fish.as_ref().is_some_and(|f| f.bite) {
                    shot!("p3_fish_bite");
                    press(g, c, KeyCode::Space).await;
                    log!("reeled at bite: inv={:?}", g.w.p.inv);
                    break 'fish;
                }
                step(g, c, &[], &[]).await
            }
            wait(g, c, 60).await
        }
        for _ in 0..4 {
            if g.w.p.equip == Some(60) {
                break;
            }
            press(g, c, KeyCode::Tab).await
        }
        for _ in 0..3 {
            let mut b = None;
            for _ in 0..900 {
                if let Some(x) =
                    g.bugs.iter().map(|b| b.1).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos)))
                {
                    b = Some(x);
                    break;
                }
                step(g, c, &[], &[]).await
            }
            let Some(bp) = b else {
                log!("no bug spawned within 15 s");
                break;
            };
            approach(g, c, bp).await;
            let bp2 = g.bugs.iter().map(|b| b.1).min_by(|a, b| a.distance(g.me.pos).total_cmp(&b.distance(g.me.pos)));
            if let Some(x) = bp2 {
                face(g, c, x).await
            }
            press(g, c, KeyCode::Space).await;
            log!("net swing: inv={:?}", g.w.p.inv);
            if g.w.p.inv.iter().any(|s| def(s.def).k == K::Bug) {
                shot!("p3_bug_caught");
                break;
            }
        }
        if let Some(r) = (0..4).find(|&r| g.loc(r) == Loc::Home) {
            let b = HOUSES[r];
            go(g, c, door_stand(b), 0.12).await;
            face(g, c, door_face(b)).await;
            press(g, c, KeyCode::E).await;
            wait(g, c, 40).await;
            log!("resident {r} home: scene={:?}", g.me.scene);
            shot!("p3_resident_home");
            approach(g, c, res_spot()).await;
            press(g, c, KeyCode::E).await;
            log!("talk ui={:?}", g.me.ui);
            shot!("p3_resident_talk");
            for _ in 0..4 {
                if g.me.ui == Ui::Play {
                    break;
                }
                press(g, c, KeyCode::E).await
            }
        }
        if let Some(site) = g.w.digs.first().copied() {
            for _ in 0..4 {
                if g.w.p.equip == Some(62) {
                    break;
                }
                press(g, c, KeyCode::Tab).await
            }
            approach(g, c, tc(site)).await;
            press(g, c, KeyCode::Space).await;
            log!("second dig: inv={:?}", g.w.p.inv)
        }
        if g.me.scene != Scene::Town {
            go(g, c, vec2(1.5, 3.5), 0.12).await;
            for _ in 0..30 {
                if g.me.scene == Scene::Town {
                    break;
                }
                step(g, c, &[KeyCode::S], &[]).await
            }
            wait(g, c, 40).await
        }
        let mb = BLDS.iter().position(|b| b.kind == BK::Museum).unwrap();
        go(g, c, door_stand(mb), 0.12).await;
        face(g, c, door_face(mb)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        log!("museum: scene={:?}", g.me.scene);
        go(g, c, vec2(3.5, 2.5), 0.12).await;
        face(g, c, vec2(3.5, 1.)).await;
        press(g, c, KeyCode::E).await;
        log!("curator: ui={:?}", g.me.ui);
        shot!("p3_museum_curator");
        for _ in 0..12 {
            if g.me.ui == Ui::Play {
                break;
            }
            press(g, c, KeyCode::E).await
        }
        log!("donated={:?} inv={:?}", g.w.donated, g.w.p.inv);
        go(g, c, vec2(3.5, 3.4), 0.12).await;
        face(g, c, vec2(3.5, 1.)).await;
        shot!("p3_museum_exhibits");
        log!("phase 3A end | {}", state(g));
    } else if phase == "3" {
        // P1-05: night lighting outdoors, lamp on/off indoors (lamp granted as a logged test fixture)
        go(g, c, vec2(13.5, 17.5), 0.15).await;
        face(g, c, vec2(13.5, 10.)).await;
        shot!("p3_night_town");
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        if !g.w.p.room.iter().any(|f| f.def == 54) {
            g.add_furn(54);
            log!("FIXTURE: lamp added to pockets for lighting check");
            let idx = g.w.p.inv.iter().position(|s| s.def == 54).unwrap();
            go(g, c, vec2(2.5, 3.5), 0.1).await;
            face(g, c, vec2(2.5, 1.)).await;
            press(g, c, KeyCode::I).await;
            for _ in 0..idx {
                press(g, c, KeyCode::S).await
            }
            press(g, c, KeyCode::Enter).await;
            press(g, c, KeyCode::D).await;
            press(g, c, KeyCode::Enter).await;
            log!("lamp placed: {:?}", g.w.p.room)
        }
        let lamp = g.w.p.room.iter().find(|f| f.def == 54).copied().unwrap();
        approach(g, c, fcenter(&lamp)).await;
        go(g, c, vec2(1.5, 3.4), 0.12).await;
        face(g, c, vec2(1.5, 0.)).await;
        shot!("p3_lamp_off");
        approach(g, c, fcenter(&lamp)).await;
        press(g, c, KeyCode::E).await;
        log!("lamp on={:?}", g.w.p.room.iter().find(|f| f.def == 54).map(|f| f.on));
        go(g, c, vec2(1.5, 3.4), 0.12).await;
        face(g, c, vec2(1.5, 0.)).await;
        shot!("p3_lamp_on");
        log!("phase 3B end | {}", state(g));
    } else {
        shot!("11_relaunch_outdoor");
        go(g, c, door_stand(HOME), 0.12).await;
        face(g, c, door_face(HOME)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        log!("home: scene={:?}", g.me.scene);
        shot!("12_relaunch_home");
        go(g, c, vec2(2.5, 3.5), 0.12).await;
        for _ in 0..30 {
            if g.me.scene == Scene::Town {
                break;
            }
            step(g, c, &[KeyCode::S], &[]).await
        }
        wait(g, c, 40).await;
        go(g, c, door_stand(SHOP), 0.12).await;
        face(g, c, door_face(SHOP)).await;
        press(g, c, KeyCode::E).await;
        wait(g, c, 40).await;
        go(g, c, vec2(3.5, 2.6), 0.15).await;
        shot!("13_relaunch_shop_soldout");
        log!("phase 2 end | {}", state(g));
    }
    log!("end time {} hr {}", c.now(), c.now().hour());
    let _ = std::fs::write(format!("{dir}/autotest_phase{phase}.txt"), rep);
}

/// Save the current frame; alpha forced opaque (shadow blending lowers framebuffer alpha, which PNG viewers would show as transparency).
fn screenshot(path: &str) {
    let mut img = get_screen_data();
    for px in img.bytes.chunks_mut(4) {
        px[3] = 255
    }
    img.export_png(path)
}
