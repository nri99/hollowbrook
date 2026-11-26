//! 2D overlay: text helpers, panels, lists, the HUD and every menu.
use super::*;

// ---- HUD / menus (2D, undistorted) ----
pub(super) fn txt(s: &str, x: f32, y: f32, z: f32, c: Color) {
    draw_text(s, x, y, z, c);
}
pub(super) fn panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::new(0.99, 0.96, 0.88, 0.97));
    draw_rectangle_lines(x, y, w, h, 3., Color::new(0.45, 0.3, 0.2, 1.))
}
pub(super) fn wrap_txt(s: &str, n: usize) -> Vec<String> {
    let mut o = vec![String::new()];
    for w in s.split(' ') {
        if o.last().unwrap().len() + w.len() > n {
            o.push(String::new())
        }
        let l = o.last_mut().unwrap();
        if !l.is_empty() {
            l.push(' ')
        }
        l.push_str(w)
    }
    o
}
pub(super) fn slot_name(s: &Slot) -> String {
    if s.size_mm > 0 {
        format!("{}  {} mm", def(s.def).name, s.size_mm)
    } else if s.qty > 1 {
        format!("{} x{}", def(s.def).name, s.qty)
    } else {
        def(s.def).name.to_string()
    }
}
pub(super) fn list(items: &[String], sel: Option<usize>, x: f32, y: f32) {
    for (k, s) in items.iter().enumerate() {
        let on = sel == Some(k);
        if on {
            draw_rectangle(x - 6., y + k as f32 * 26. - 19., 330., 25., Color::new(1., 0.85, 0.4, 0.6))
        }
        txt(s, x, y + k as f32 * 26., 22., BLACK)
    }
}
pub(super) fn hud(g: &Game, now: NaiveDateTime, dbg: Option<&str>) {
    let (sw, sh) = (vw(), vh());
    let season = match now.month() {
        3..=5 => "Spring",
        6..=8 => "Summer",
        9..=11 => "Autumn",
        _ => "Winter",
    };
    draw_rectangle(10., 10., 540., 62., Color::new(0., 0., 0., 0.45));
    const MON: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    txt(
        &format!(
            "{} {:02}:{:02}  {} {}  {}{}",
            now.weekday(),
            now.hour(),
            now.minute(),
            MON[now.month0() as usize],
            now.day(),
            season,
            match weather(g.w.day, now.month()) {
                Wx::Rain => "  Rain",
                Wx::Snow => "  Snow",
                Wx::Clear => "",
            }
        ),
        20.,
        35.,
        26.,
        WHITE,
    );
    txt(
        &format!(
            "{} coins    pockets {}/{}    tool: {}",
            g.w.p.money,
            g.w.p.inv.len(),
            INV_CAP,
            g.w.p.equip.map_or("none", |t| def(t).name)
        ),
        20.,
        62.,
        22.,
        GOLD,
    );
    // Recent messages: only during play (menus cover them), shortened to fit the viewport, on a soft backdrop for legibility.
    if g.me.ui == Ui::Play {
        for (k, (m, _)) in g.me.msgs.iter().rev().filter(|m| g.t - m.1 < 5.).take(4).enumerate() {
            let mut s = m.clone();
            if measure_text(&s, None, 22, 1.).width > sw - 40. {
                while s.len() > 4 && measure_text(&s, None, 22, 1.).width > sw - 70. {
                    s.pop();
                }
                s = format!("{}...", s.trim_end())
            }
            let y = 100. + k as f32 * 24.;
            draw_rectangle(8., y - 18., measure_text(&s, None, 22, 1.).width + 12., 24., Color::new(0., 0., 0., 0.28));
            txt(&s, 14., y, 22., WHITE)
        }
    }
    if g.me.ui == Ui::Play && g.me.placing.is_none() && g.me.trans.is_none() {
        if let Some(tg) = g.target() {
            let s = if g.me.pad {
                format!("[A] {}", g.tg_label(tg)).replace("[F]", "[Select]").replace("[G]", "[LB]")
            } else {
                format!("[E] {}", g.tg_label(tg))
            };
            let tw = measure_text(&s, None, 24, 1.).width;
            draw_rectangle(sw / 2. - tw / 2. - 10., sh - 92., tw + 20., 32., Color::new(0., 0., 0., 0.55));
            txt(&s, sw / 2. - tw / 2., sh - 69., 24., WHITE)
        }
    }
    if g.me.fish.as_ref().is_some_and(|f| f.bite) {
        txt("!", sw / 2. - 10., sh / 2. - 80., 90., Color::new(1., 0.3, 0.2, 1.))
    }
    let narrow = sw < 900.;
    if narrow {
        draw_rectangle(
            sw - 62.,
            14.,
            50.,
            30.,
            if g.me.id == 0 { Color::new(0.2, 0.45, 0.9, 0.9) } else { Color::new(0.9, 0.35, 0.3, 0.9) },
        );
        txt(&format!("P{}", g.me.id + 1), sw - 52., 38., 26., WHITE)
    }
    if let Some(a) = &g.act {
        if g.me.scene == Scene::Islet && a.players.contains(&g.me.id) {
            let (n, obj, _, th) = ACTS[a.kind];
            let bw = 600f32.min(sw - 20.);
            let (x0, y0) = (sw / 2. - bw / 2., sh - 134.);
            draw_rectangle(x0, y0, bw, 92., Color::new(0., 0.15, 0.25, 0.75));
            let mine = a.mine.iter().find(|m| m.0 == g.me.id).map_or(0, |m| m.1);
            txt(
                &format!(
                    "{n}   {:.0}s left   team score {}{}",
                    a.left.ceil(),
                    a.score,
                    if a.done { "   - finished!" } else { "" }
                ),
                x0 + 12.,
                y0 + 26.,
                24.,
                WHITE,
            );
            txt(obj, x0 + 12., y0 + 52., 17., Color::new(0.8, 0.95, 1., 1.));
            txt(
                &format!(
                    "Bronze {} / Silver {} / Gold {}  ->  {} / {} / {} Sea Glass   you: {mine}{}",
                    th[0],
                    th[1],
                    th[2],
                    ACT_REWARD[0],
                    ACT_REWARD[1],
                    ACT_REWARD[2],
                    if a.players.len() > 1 { "   (team)" } else { "" }
                ),
                x0 + 12.,
                y0 + 78.,
                17.,
                GOLD,
            )
        }
    }
    let hint = if g.me.mplace.is_some() {
        "Placing project: walk/face a spot  Enter place  Esc cancel"
    } else if g.swimming() {
        if g.me.pad {
            "Swimming:  X dive  stick swim"
        } else {
            "Swimming:  Space dive  WASD swim"
        }
    } else if g.me.pad {
        if g.me.placing.is_some() {
            "Stick move  RB rotate  A place  B cancel"
        } else {
            if narrow {
                "Stick move  A act  X tool  RB tool  Y bag  B back"
            } else {
                "Stick move  RT run  A act  X tool  RB next tool  Y bag  Start journal  B cancel"
            }
        }
    } else if narrow {
        if g.me.placing.is_some() {
            "Move  R rotate  Enter place  Esc cancel"
        } else {
            "Move  E act  Space tool  Tab tool  I bag  J journal"
        }
    } else if g.me.placing.is_some() {
        "Placing:  WASD/Arrows move   R rotate   Enter place   Esc cancel"
    } else {
        "WASD move  Shift run  E interact  Space use tool  Tab tool  I inventory  J journal  F/G move/pick up furniture  Esc cancel"
    };
    draw_rectangle(4., sh - 34., measure_text(hint, None, 19, 1.).width + 24., 28., Color::new(0., 0., 0., 0.4));
    txt(hint, 12., sh - 14., 19., WHITE);
    if let Some(s) = g.w.p.seek.as_ref().filter(|s| g.seek_live(s)) {
        let bw = (sw - 24.).min(460.);
        draw_rectangle(12., 186., bw, 50., Color::new(0.1, 0.24, 0.16, 0.85));
        txt(
            &format!("Trail Seek: {}/3 found   {}s left", s.found.len(), (s.deadline_ms - g.clock_ms + 999) / 1000),
            22.,
            207.,
            21.,
            WHITE,
        );
        txt("Talk to friends at their hiding spots.", 22., 228., 17., WHITE);
    }
    bounty_hud(g, sw);
    match &g.me.ui {
        Ui::Play => {}
        Ui::Bounty(sel) => bounty_menu(g, *sel, sw),
        Ui::Forge(sel) => forge_menu(g, *sel, sw),
        Ui::Invitation => {
            let pw = (sw - 24.).min(620.);
            let x = (sw - pw) / 2.;
            panel(x, 180., pw, 270.);
            txt("An invitation", x + 18., 216., 26., BLACK);
            if let Some(v) = &g.w.p.invitation {
                let s = if v.kind == 0 {
                    format!("{} invites you home today, {}:00-{}:00.", g.res(v.slot).name, v.hour, v.hour + 1)
                } else {
                    format!(
                        "{} proposes Trail Seek: find three friends behind trees or buildings in four minutes.",
                        g.res(v.slot).name
                    )
                };
                for (i, l) in wrap_txt(&s, ((pw - 36.) / 11.) as usize).iter().enumerate() {
                    txt(l, x + 18., 258. + i as f32 * 25., 21., BLACK);
                }
                txt(
                    if v.kind == 0 {
                        "A friendly visit, with a small gift."
                    } else {
                        "They walk to their spots. Nothing lost on timeout."
                    },
                    x + 18.,
                    350.,
                    19.,
                    DARKGRAY,
                );
            }
            txt(
                if g.me.pad { "A accept   B decline" } else { "Enter accept   Esc decline" },
                x + 18.,
                420.,
                20.,
                DARKGRAY,
            );
        }
        Ui::Contest(sel) => {
            let pw = (sw - 24.).min(620.);
            let x = (sw - pw) / 2.;
            panel(x, 95., pw, 490.);
            txt(g.contest_kind().map_or("Survey", |k| crate::events::CONTEST_NAMES[k]), x + 16., 128., 26., BLACK);
            let best = g.contest_entry().map_or(0, |e| e.best);
            txt(&format!("Best: {best} points. Entries close 17:00."), x + 16., 158., 19., DARKGRAY);
            let items: Vec<String> = g
                .contest_slots()
                .iter()
                .map(|&i| {
                    let s = &g.w.p.inv[i];
                    format!("{} / {} pt", slot_name(s), crate::events::catch_score(s.def, s.size_mm))
                })
                .collect();
            if items.is_empty() {
                txt("No measured catches of today's kind.", x + 16., 195., 19., DARKGRAY);
            }
            list(&items, Some(*sel), x + 16., 195.);
            txt("Improved entries are released. Others return.", x + 16., 539., 18., DARKGRAY);
            txt(
                if g.me.pad {
                    "Up/Down choose   A enter   B close"
                } else {
                    "W/S choose   Enter enter catch   Esc close"
                },
                x + 16.,
                570.,
                18.,
                DARKGRAY,
            );
        }
        Ui::Talk { who, lines, i, .. } => {
            let pw = (sw - 24.).min(860.);
            let x0 = sw / 2. - pw / 2.;
            panel(x0, sh - 200., pw, 140.);
            txt(who, x0 + 20., sh - 168., 26., Color::new(0.55, 0.3, 0.15, 1.));
            for (k, l) in wrap_txt(lines.get(*i).map_or("", |s| s.as_str()), ((pw - 40.) / 12.5) as usize)
                .iter()
                .take(2)
                .enumerate()
            {
                txt(l, x0 + 20., sh - 128. + k as f32 * 26., 24., BLACK)
            }
            if let Ui::Talk { npc: Some(j), .. } = g.me.ui {
                let mv = g.w.moving.get(j).is_some_and(|&m| m != 0);
                txt(
                    &format!(
                        "{}{}",
                        if g.me.pad { "[LB] give gift" } else { "[G] give gift" },
                        if mv {
                            if g.me.pad {
                                "   [Y] ask to stay"
                            } else {
                                "   [X] ask to stay"
                            }
                        } else {
                            ""
                        }
                    ),
                    x0 + 20.,
                    sh - 75.,
                    20.,
                    GRAY,
                )
            }
            txt(
                if g.me.pad { "[A] next" } else { "[E] next" },
                x0 + pw - 100.,
                sh - 72.,
                20.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Inv(s) | Ui::Sell(s) => {
            let sell = matches!(g.me.ui, Ui::Sell(_));
            panel(sw / 2. - 260., 90., 520., 520.);
            txt(
                if sell { "Sell to Mr. Tumble" } else { "Pockets" },
                sw / 2. - 240.,
                125.,
                30.,
                Color::new(0.45, 0.3, 0.2, 1.),
            );
            let mut items: Vec<String> =
                g.w.p
                    .inv
                    .iter()
                    .map(|it| {
                        if sell {
                            format!("{}  -  {}c", slot_name(it), def(it.def).sell * it.qty as u32)
                        } else {
                            slot_name(it)
                        }
                    })
                    .collect();
            if sell {
                let lv = g.w.p.home as usize;
                items.push(if g.w.p.debt > 0 {
                    format!("Pay home balance  ({} left)", g.w.p.debt)
                } else if lv < EXPAND_COST.len() {
                    format!(
                        "{}  (adds {}c balance)",
                        ["Expand your home", "Expand your home", "Dig a basement", "Build an upstairs"][lv],
                        EXPAND_COST[lv]
                    )
                } else {
                    "Home fully expanded".into()
                });
                items.push(format!("Order from catalog ({} items)", g.w.p.catalog.len()));
                items.push("Customize furniture / wallpaper / flooring".into());
                items.push("Seeds, watering can and tool upgrades".into())
            }
            if items.is_empty() {
                txt("(empty)", sw / 2. - 230., 170., 22., GRAY)
            }
            list(&items, Some(*s), sw / 2. - 230., 170.);
            txt(
                match (sell, g.me.pad) {
                    (true, false) => "W/S select  Enter sell  A sell all finds  Esc close",
                    (true, true) => "Up/Down select  A sell  Y sell all finds  B close",
                    (false, false) => "W/S select  Enter place furniture (at home)  Esc close",
                    _ => "Up/Down select  A place furniture (at home)  B close",
                },
                sw / 2. - 240.,
                590.,
                19.,
                DARKGRAY,
            );
            if !sell {
                txt(
                    &format!("Tools: {}", g.w.p.tools.iter().map(|t| def(*t).name).collect::<Vec<_>>().join(", ")),
                    sw / 2. - 240.,
                    565.,
                    19.,
                    DARKGRAY,
                )
            }
        }
        Ui::Confirm(d) => {
            let (dd, p, _) = g.display_item(*d);
            let n = def(dd).name;
            panel(sw / 2. - 260., sh / 2. - 80., 520., 150.);
            txt(&format!("Buy the {n} for {p} coins?"), sw / 2. - 235., sh / 2. - 35., 26., BLACK);
            txt(&format!("You have {} coins.", g.w.p.money), sw / 2. - 235., sh / 2. - 2., 22., DARKGRAY);
            txt(
                if g.me.pad { "[A] Yes     [B] No" } else { "[Enter] Yes     [Esc] No" },
                sw / 2. - 235.,
                sh / 2. + 40.,
                22.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Storage(c, s) => {
            panel(sw / 2. - 380., 90., 760., 520.);
            txt("Pockets", sw / 2. - 360., 125., 28., BLACK);
            txt("Cabinet storage", sw / 2. + 20., 125., 28., BLACK);
            let a: Vec<String> = g.w.p.inv.iter().map(slot_name).collect();
            let bb: Vec<String> = g.w.p.storage.iter().map(slot_name).collect();
            list(&a, (*c == 0).then_some(*s), sw / 2. - 350., 165.);
            list(&bb, (*c == 1).then_some(*s), sw / 2. + 30., 165.);
            txt(
                if g.me.pad {
                    "Left/Right switch side  Up/Down select  A move item  B close"
                } else {
                    "A/D switch side  W/S select  Enter move item  Esc close"
                },
                sw / 2. - 360.,
                590.,
                19.,
                DARKGRAY,
            )
        }
        Ui::Gift(j, sel) => {
            panel(sw / 2. - 260., 100., 520., 440.);
            txt(&format!("Give {} a gift", g.res(*j).name), sw / 2. - 240., 135., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = g.w.p.inv.iter().map(slot_name).collect();
            list(&items, Some(*sel), sw / 2. - 230., 175.);
            txt(
                if g.me.pad { "Up/Down select  A give  B cancel" } else { "W/S select  Enter give  Esc cancel" },
                sw / 2. - 240.,
                525.,
                19.,
                DARKGRAY,
            )
        }
        Ui::MPick(sel) => {
            panel(sw / 2. - 270., 100., 540., 330.);
            txt("Start a funded public project", sw / 2. - 250., 135., 26., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = MPROJECTS.iter().map(|p| format!("{}  -  {} coins", p.0, p.1)).collect();
            list(&items, Some(*sel), sw / 2. - 240., 175.);
            txt("Pick one; you head outside to choose the spot (green = valid).", sw / 2. - 250., 330., 18., DARKGRAY);
            txt("Built the morning after it is fully funded. Esc cancels.", sw / 2. - 250., 355., 18., DARKGRAY)
        }
        Ui::Keeper(sel) => {
            let items = g.keeper_items();
            let pw = (sw - 40.).min(720.);
            let x0 = sw / 2. - pw / 2.;
            panel(x0, 80., pw, 120. + items.len() as f32 * 28.);
            txt(
                "Mr. Tumble  -  seeds, watering can, tool upgrades",
                x0 + 20.,
                115.,
                22.,
                Color::new(0.45, 0.3, 0.2, 1.),
            );
            let labels: Vec<String> = items.iter().map(|x| x.0.clone()).collect();
            list(&labels, Some(*sel), x0 + 30., 145.);
            txt(
                &format!(
                    "You have {} coins.  {}",
                    g.w.p.money,
                    if g.me.pad { "[A] Choose  [B] Leave" } else { "[Enter] Choose  [Esc] Leave" }
                ),
                x0 + 20.,
                120. + items.len() as f32 * 28. + 60.,
                17.,
                DARKGRAY,
            )
        }
        Ui::Paint(sel) => {
            panel(sw / 2. - 280., 110., 560., 320.);
            txt(
                &format!("Repaint your house  -  {PAINT_FEE} coins (walls & roof)"),
                sw / 2. - 260.,
                145.,
                21.,
                Color::new(0.45, 0.3, 0.2, 1.),
            );
            let items: Vec<String> = PAINTS
                .iter()
                .enumerate()
                .map(|(k, p)| format!("{}{}", p.0, if g.w.p.paint == Some(k as u8) { "  (current)" } else { "" }))
                .collect();
            list(&items, Some(*sel), sw / 2. - 250., 180.);
            for (k, p) in PAINTS.iter().enumerate() {
                let y = 168. + k as f32 * 26.;
                draw_rectangle(sw / 2. + 150., y, 30., 18., rgb(p.1));
                draw_rectangle(sw / 2. + 184., y, 30., 18., rgb(p.2))
            }
            txt(
                if g.me.pad { "[A] Paint  [B] Back" } else { "[Enter] Paint  [Esc] Back" },
                sw / 2. - 260.,
                405.,
                19.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Camp(sel) => {
            let v = &VISITORS[g.visitor()];
            panel(sw / 2. - 300., 100., 600., 330.);
            txt(
                &format!("{} the {}  -  campground", v.name, v.species),
                sw / 2. - 280.,
                135.,
                24.,
                Color::new(0.45, 0.3, 0.2, 1.),
            );
            let (fd, fq) = g.favor_item();
            let done = g.w.p.camp_favors.contains(&(g.visitor() as u8));
            let mut items: Vec<String> = v
                .goods
                .iter()
                .map(|&d| {
                    format!(
                        "{}  -  {} coins{}",
                        def(d).name,
                        def(d).buy,
                        if g.w.camp_sold.contains(&d) { "  (sold out today)" } else { "" }
                    )
                })
                .collect();
            items.push(if done {
                "Favor: done - thank you!".into()
            } else {
                format!("Favor: bring {fq} {}", def(fd).name)
            });
            items.push("Chat".into());
            list(&items, Some(*sel), sw / 2. - 270., 170.);
            txt(
                &format!("You have {} coins. One of each item per day for the whole town.", g.w.p.money),
                sw / 2. - 280.,
                360.,
                17.,
                DARKGRAY,
            );
            txt(
                if g.me.pad { "[A] Choose   [B] Leave" } else { "[Enter] Choose   [Esc] Leave" },
                sw / 2. - 280.,
                400.,
                20.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Kiosk(sel) => {
            let pw = (sw - 40.).min(700.);
            let x0 = sw / 2. - pw / 2.;
            panel(x0, 70., pw, 470.);
            txt(
                &format!("Initiatives kiosk  -  {} coupons", g.w.p.coupons),
                x0 + 20.,
                105.,
                24.,
                Color::new(0.45, 0.3, 0.2, 1.),
            );
            let mut items: Vec<String> = (0..3)
                .map(|k| {
                    g.w.p.tasks.get(k).map_or("(no task)".to_string(), |t| {
                        let (_, n, text, c) = TASKS[t.t];
                        format!("Claim: {text} {}/{n} -> {c}{}", t.prog, if t.claimed { " (claimed)" } else { "" })
                    })
                })
                .collect();
            items.push(g.w.wtask.as_ref().map_or("(no weekly task)".into(), |w| {
                let (_, n, text) = WTASKS[w.t];
                format!(
                    "Claim weekly: {text} {}/{n} -> {WEEKLY_COUPONS}{}",
                    w.prog,
                    if w.claimed.contains(&g.w.p.id) { " (claimed)" } else { "" }
                )
            }));
            for &(d, c) in &KIOSK_ITEMS {
                items.push(format!("Reward: {}  -  {c} coupons", def(d).name))
            }
            list(&items, Some(*sel), x0 + 30., 135.);
            txt(
                if g.me.pad { "[A] Claim / trade   [B] Close" } else { "[Enter] Claim / trade   [Esc] Close" },
                x0 + 20.,
                525.,
                19.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Title(sel) => {
            draw_rectangle(0., 0., sw, sh, Color::new(0.12, 0.2, 0.16, 0.92));
            txt("Hollowbrook", sw / 2. - 150., sh * 0.28, 64., Color::new(0.95, 0.9, 0.7, 1.));
            txt(VERSION, sw / 2. - 150., sh * 0.28 + 34., 18., Color::new(0.8, 0.85, 0.8, 1.));
            let items = vec![
                if g.fresh { "Start your town".to_string() } else { "Continue".into() },
                "Host a LAN game".into(),
                "Join a LAN game".into(),
                "Controls and guide".into(),
                "Quit".into(),
            ];
            list(&items, Some(*sel), sw / 2. - 140., sh * 0.42);
            txt(
                "Your town saves automatically (every 15 s, each new day, and on quit).",
                sw / 2. - 300.,
                sh * 0.75,
                18.,
                Color::new(0.85, 0.9, 0.85, 1.),
            );
            txt(
                if g.me.pad { "Stick: choose   A: confirm" } else { "W/S: choose   Enter: confirm" },
                sw / 2. - 300.,
                sh * 0.75 + 26.,
                18.,
                Color::new(0.85, 0.9, 0.85, 1.),
            )
        }
        Ui::Help => {
            draw_rectangle(0., 0., sw, sh, Color::new(0.12, 0.2, 0.16, 0.95));
            let mut y = 70.;
            txt("Controls", 60., y, 30., Color::new(0.95, 0.9, 0.7, 1.));
            y += 34.;
            let wcol = Color::new(0.92, 0.95, 0.92, 1.);
            for l in HELP_KEYS {
                txt(l, 70., y, 17., wcol);
                y += 20.
            }
            y += 8.;
            txt("Gamepad", 60., y + 4., 22., Color::new(0.95, 0.9, 0.7, 1.));
            y += 26.;
            for l in HELP_PAD {
                txt(l, 70., y, 17., wcol);
                y += 20.
            }
            y += 10.;
            txt("Guide", 60., y, 26., Color::new(0.95, 0.9, 0.7, 1.));
            y += 26.;
            'g: for l in GUIDE.iter().take(5) {
                for w in wrap(l, ((sw - 140.) / 9.) as usize) {
                    if y > sh - 56. {
                        txt("More in the journal's Guide page.", 70., y, 16., GOLD);
                        break 'g;
                    }
                    txt(&w, 70., y, 16., wcol);
                    y += 19.
                }
            }
            txt(if g.me.pad { "[A]/[B] back" } else { "[Enter]/[Esc] back" }, 60., sh - 24., 18., GOLD)
        }
        Ui::Civic(sel) => {
            panel(sw / 2. - 280., 120., 560., 260.);
            txt(ADMIN, sw / 2. - 260., 155., 24., Color::new(0.45, 0.3, 0.2, 1.));
            txt("Welcome to the Town Hall. How can I help?", sw / 2. - 260., 182., 18., DARKGRAY);
            let items = vec![
                "Town policies".to_string(),
                if g.me.id == LEADER {
                    "Start a funded public project".into()
                } else {
                    "Start a funded public project (leader only)".into()
                },
                "Project and town status".into(),
                format!("Repaint your house ({PAINT_FEE} coins)"),
            ];
            list(&items, Some(*sel), sw / 2. - 250., 210.);
            txt(
                if g.me.pad { "[A] Choose   [B] Leave" } else { "[Enter] Choose   [Esc] Leave" },
                sw / 2. - 260.,
                355.,
                20.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Policy(sel) => {
            panel(sw / 2. - 330., 80., 660., 400.);
            txt(ADMIN, sw / 2. - 310., 115., 24., Color::new(0.45, 0.3, 0.2, 1.));
            txt(
                &format!(
                    "Civic desk. Current policy: {}. Only one policy can be active.",
                    g.pol().map_or("none", |p| p.name)
                ),
                sw / 2. - 310.,
                145.,
                18.,
                DARKGRAY,
            );
            let mut items: Vec<String> = POLICIES.iter().map(|p| format!("{}  -  {} coins", p.name, p.fee)).collect();
            items.push("Repeal current policy (free)".into());
            list(&items, Some(*sel), sw / 2. - 300., 180.);
            let e = POLICIES.get(*sel).map_or("Return to normal hours and maintenance.", |p| p.effect);
            for (k, l) in wrap(e, 70).iter().enumerate() {
                txt(l, sw / 2. - 310., 360. + k as f32 * 22., 18., BLACK)
            }
            if g.me.id != LEADER {
                txt(
                    "Only the town leader can change policies.",
                    sw / 2. - 310.,
                    455.,
                    18.,
                    Color::new(0.6, 0.2, 0.2, 1.),
                )
            }
        }
        Ui::PolicyOk(sel) => {
            panel(sw / 2. - 330., sh / 2. - 110., 660., 210.);
            let (t, fee, e) = POLICIES
                .get(*sel)
                .map_or(("Repeal current policy", 0, "Return to normal hours and maintenance."), |p| {
                    (p.name, p.fee, p.effect)
                });
            txt(
                &format!("{t}  -  fee {fee} coins (you have {})", g.w.p.money),
                sw / 2. - 310.,
                sh / 2. - 75.,
                22.,
                BLACK,
            );
            for (k, l) in wrap(e, 70).iter().enumerate() {
                txt(l, sw / 2. - 310., sh / 2. - 40. + k as f32 * 22., 18., DARKGRAY)
            }
            txt(
                if g.me.pad { "[A] Confirm   [B] Back" } else { "[Enter] Confirm   [Esc] Back" },
                sw / 2. - 310.,
                sh / 2. + 70.,
                22.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Act(sel) => {
            panel(sw / 2. - 320., 90., 640., 370.);
            txt("Tidekeeper's board - timed activities", sw / 2. - 300., 125., 24., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> = ACTS.iter().map(|a| format!("{}  -  {:.0} seconds", a.0, a.2)).collect();
            list(&items, Some(*sel), sw / 2. - 290., 160.);
            let a = ACTS[*sel];
            for (k, l) in wrap(a.1, 66).iter().enumerate() {
                txt(l, sw / 2. - 300., 290. + k as f32 * 22., 18., BLACK)
            }
            txt(
                &format!(
                    "Team score {} / {} / {} earns {} / {} / {} Sea Glass each (once per activity per day).",
                    a.3[0], a.3[1], a.3[2], ACT_REWARD[0], ACT_REWARD[1], ACT_REWARD[2]
                ),
                sw / 2. - 300.,
                345.,
                17.,
                DARKGRAY,
            );
            txt(
                "Everyone on Gullrock joins and shares the score; a player in town is unaffected.",
                sw / 2. - 300.,
                370.,
                17.,
                DARKGRAY,
            );
            txt(
                if g.me.pad { "[A] Start   [B] Close" } else { "[Enter] Start   [Esc] Close" },
                sw / 2. - 300.,
                430.,
                22.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Fund => {
            if let Some(m) = &g.w.mproject {
                panel(sw / 2. - 260., sh / 2. - 80., 520., 150.);
                txt(
                    &format!("{}: {} / {} coins", MPROJECTS[m.idx].0, m.funded, MPROJECTS[m.idx].1),
                    sw / 2. - 240.,
                    sh / 2. - 40.,
                    24.,
                    BLACK,
                );
                txt(
                    &format!("You have {} coins. Each press donates up to {}.", g.w.p.money, DONATE_STEP),
                    sw / 2. - 240.,
                    sh / 2. - 6.,
                    20.,
                    DARKGRAY,
                );
                txt(
                    if g.me.pad { "[A] Donate   [B] Close" } else { "[Enter] Donate   [Esc] Close" },
                    sw / 2. - 240.,
                    sh / 2. + 36.,
                    22.,
                    Color::new(0.45, 0.3, 0.2, 1.),
                )
            }
        }
        Ui::ConfirmExpand => {
            let lv = g.w.p.home as usize;
            let (a, b) = (HOME_DIMS[lv], HOME_DIMS[(lv + 1).min(HOME_DIMS.len() - 1)]);
            panel(sw / 2. - 270., sh / 2. - 90., 540., 170.);
            txt(
                &if lv >= 2 {
                    format!("{} for your home", if lv == 2 { "Dig a cosy basement" } else { "Build a bright upstairs" })
                } else {
                    format!("Expand your home: {}x{} -> {}x{} floor", a.0, a.1, b.0, b.1)
                },
                sw / 2. - 245.,
                sh / 2. - 45.,
                24.,
                BLACK,
            );
            txt(
                &format!(
                    "Adds a {} coin balance. No interest, no deadline.",
                    EXPAND_COST[lv.min(EXPAND_COST.len() - 1)]
                ),
                sw / 2. - 245.,
                sh / 2. - 12.,
                20.,
                DARKGRAY,
            );
            txt(
                "Pay it off at the counter to unlock the next expansion.",
                sw / 2. - 245.,
                sh / 2. + 14.,
                20.,
                DARKGRAY,
            );
            txt(
                if g.me.pad { "[A] Expand   [B] Cancel" } else { "[Enter] Expand   [Esc] Cancel" },
                sw / 2. - 245.,
                sh / 2. + 52.,
                22.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::Travel(out) => {
            panel(sw / 2. - 270., sh / 2. - 80., 540., 150.);
            txt(
                if *out { "Sail to Gullrock Islet?" } else { "Sail back to Hollowbrook?" },
                sw / 2. - 245.,
                sh / 2. - 35.,
                26.,
                BLACK,
            );
            txt(
                &if *out {
                    format!("Fare: {FARE} coins (you have {}).", g.w.p.money)
                } else {
                    "The trip home is free.".into()
                },
                sw / 2. - 245.,
                sh / 2. - 2.,
                22.,
                DARKGRAY,
            );
            txt(
                if g.me.pad { "[A] Sail     [B] Stay" } else { "[Enter] Sail     [Esc] Stay" },
                sw / 2. - 245.,
                sh / 2. + 40.,
                22.,
                Color::new(0.45, 0.3, 0.2, 1.),
            )
        }
        Ui::IStall(sel) => {
            panel(sw / 2. - 260., 120., 520., 260.);
            txt("Sea Glass stall", sw / 2. - 240., 155., 28., Color::new(0.2, 0.5, 0.5, 1.));
            let items: Vec<String> =
                ISTALL_ITEMS.iter().map(|&(d, c)| format!("{}  -  {} Sea Glass", def(d).name, c)).collect();
            list(&items, Some(*sel), sw / 2. - 230., 195.);
            txt(
                &format!(
                    "You have {} Sea Glass.",
                    g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>()
                ),
                sw / 2. - 240.,
                320.,
                20.,
                DARKGRAY,
            );
            txt(
                if g.me.pad { "Up/Down select  A trade  B close" } else { "W/S select  Enter trade  Esc close" },
                sw / 2. - 240.,
                350.,
                19.,
                DARKGRAY,
            )
        }
        Ui::Tint(sel) => {
            panel(sw / 2. - 270., 100., 540., 440.);
            txt("Customize", sw / 2. - 250., 135., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let mut items: Vec<String> = g
                .tint_targets()
                .iter()
                .map(|&i| {
                    let s = g.w.p.inv[i];
                    format!(
                        "{} ({})  -  {}c",
                        def(s.def).name,
                        TINT_NAMES[Game::tint_of(&g.w.p, s.uid) as usize],
                        TINT_FEE
                    )
                })
                .collect();
            items.push(format!("Wallpaper (style {})  -  {}c", g.w.p.wall + 1, DECOR_FEE));
            items.push(format!("Flooring (style {})  -  {}c", g.w.p.floor + 1, DECOR_FEE));
            list(&items, Some(*sel), sw / 2. - 240., 175.);
            txt(
                if g.me.pad { "Up/Down select  A change  B close" } else { "W/S select  Enter change  Esc close" },
                sw / 2. - 250.,
                525.,
                19.,
                DARKGRAY,
            )
        }
        Ui::Catalog(sel) => {
            panel(sw / 2. - 260., 100., 520., 440.);
            txt("Catalog", sw / 2. - 240., 135., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> =
                g.w.p.catalog.iter().map(|&d| format!("{}  -  {}c", def(d).name, def(d).buy)).collect();
            if items.is_empty() {
                txt("Items you own appear here.", sw / 2. - 230., 175., 22., GRAY)
            }
            list(&items[..items.len().min(12)], Some(*sel), sw / 2. - 230., 175.);
            txt(
                if g.me.pad { "Up/Down select  A order  B close" } else { "W/S select  Enter order  Esc close" },
                sw / 2. - 240.,
                525.,
                19.,
                DARKGRAY,
            )
        }
        Ui::Mail(sel) => {
            panel(sw / 2. - 280., 100., 560., 440.);
            txt("Mailbox", sw / 2. - 260., 135., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let items: Vec<String> =
                g.w.p
                    .mail
                    .iter()
                    .rev()
                    .map(|l| {
                        format!(
                            "{}{}{}",
                            if l.read { "   " } else { "* " },
                            l.from,
                            if !l.claimed && (l.coins > 0 || l.item.is_some()) { "  (gift)" } else { "" }
                        )
                    })
                    .collect();
            if items.is_empty() {
                txt("No letters yet.", sw / 2. - 250., 175., 22., GRAY)
            }
            list(&items[..items.len().min(8)], Some(*sel), sw / 2. - 250., 175.);
            if let Some(l) = g.w.p.mail.iter().rev().nth(*sel) {
                for (k, line) in wrap_txt(&l.text, 44).iter().enumerate() {
                    txt(line, sw / 2. - 250., 400. + k as f32 * 24., 21., BLACK)
                }
            }
            txt(
                if g.me.pad {
                    "Up/Down select  A open/claim  B close"
                } else {
                    "W/S select  Enter open/claim  Esc close"
                },
                sw / 2. - 260.,
                525.,
                19.,
                DARKGRAY,
            )
        }
        Ui::Rack(sel) => {
            panel(sw / 2. - 260., 120., 520., 380.);
            txt("Clothing rack", sw / 2. - 240., 155., 28., Color::new(0.45, 0.3, 0.2, 1.));
            let mut items: Vec<String> =
                CLOTHES.iter().map(|&d| format!("{}  -  {} coins", def(d).name, def(d).buy)).collect();
            items.push(format!("Change hairstyle  -  {} coins", HAIR_FEE));
            list(&items, Some(*sel), sw / 2. - 230., 195.);
            txt(&format!("You have {} coins.", g.w.p.money), sw / 2. - 240., 440., 20., DARKGRAY);
            txt(
                if g.me.pad { "Up/Down select  A buy  B close" } else { "W/S select  Enter buy  Esc close" },
                sw / 2. - 240.,
                470.,
                19.,
                DARKGRAY,
            )
        }
        Ui::Journal => {
            panel(60., 80., sw - 120., sh - 160.);
            txt(
                ["Field Journal", "Initiatives", "Settings", "Controls", "Guide", "Friends"][g.me.jpage as usize % 6],
                80.,
                115.,
                30.,
                Color::new(0.45, 0.3, 0.2, 1.),
            );
            txt(
                &format!(
                    "Page {}/6   {}   {}",
                    g.me.jpage + 1,
                    if g.me.pad { "[RB] next page" } else { "[Tab] next page" },
                    VERSION
                ),
                80.,
                140.,
                16.,
                DARKGRAY,
            );
            if g.me.jpage == 0 {
                for (col, (k, title)) in [(K::Fish, "Fish & sea"), (K::Bug, "Bugs"), (K::Shell, "Shells & finds")]
                    .iter()
                    .enumerate()
                    .filter(|(c, _)| sw >= 1100. || *c == g.me.jcol as usize)
                {
                    let x = if sw >= 1100. { 80. + col as f32 * 380. } else { 80. };
                    txt(title, x, 170., 24., BLACK);
                    for (j, dd) in DEFS
                        .iter()
                        .filter(|dd| {
                            dd.k == *k
                                || (*k == K::Shell && matches!(dd.k, K::Treasure | K::Fruit | K::Fossil))
                                || (*k == K::Fish && dd.k == K::Sea)
                        })
                        .enumerate()
                    {
                        let seen = g.w.p.seen.contains(&dd.id);
                        txt(
                            &format!(
                                "{}{}",
                                if avail(dd, g.mo, g.hr) && matches!(dd.k, K::Fish | K::Bug) { "* " } else { "  " },
                                if seen {
                                    format!(
                                        "{}  {}c{}",
                                        dd.name,
                                        dd.sell,
                                        if g.w.donated.iter().any(|x| x.0 == dd.id) { "  [museum]" } else { "" }
                                    )
                                } else {
                                    "???".into()
                                }
                            ),
                            x,
                            194. + j as f32 * ((sh - 340.) / 19.).min(24.),
                            18.,
                            if seen { BLACK } else { GRAY },
                        )
                    }
                }
                txt("* = active right now", 80., sh - 100., 18., DARKGRAY);
                txt(
                    &format!("Town liveliness: {} ({})", g.liveliness(), Game::live_word(g.liveliness())),
                    80.,
                    sh - 125.,
                    20.,
                    Color::new(0.3, 0.5, 0.3, 1.),
                );
                if sw < 1100. {
                    txt(
                        if g.me.pad { "stick left/right: other columns" } else { "Left/Right: other columns" },
                        80.,
                        sh - 150.,
                        18.,
                        DARKGRAY,
                    )
                }
            }
            if g.me.jpage == 2 {
                txt(
                    &format!(
                        "Music {:.0}%{}  ({})    Sound effects {:.0}%  ({})    {} mute music",
                        g.music_vol * 100.,
                        if g.muted { " (muted)" } else { "" },
                        if g.me.pad { "stick left/right" } else { "Left/Right" },
                        g.sfx_vol * 100.,
                        if g.me.pad { "stick up/down" } else { "Up/Down" },
                        if g.me.pad { "[Y]" } else { "[X]" }
                    ),
                    80.,
                    170.,
                    20.,
                    Color::new(0.3, 0.3, 0.55, 1.),
                );
                txt("Music and sound settings are saved for this computer.", 80., 205., 18., DARKGRAY)
            }
            if g.me.jpage == 1 {
                let mut y = 160.;
                txt(
                    &format!("Coupons: {}  (kiosk in the Town Hall)", g.w.p.coupons),
                    80.,
                    y + 8.,
                    20.,
                    Color::new(0.3, 0.3, 0.55, 1.),
                );
                y += 34.;
                txt("Today's tasks (end at 6:00):", 80., y, 22., BLACK);
                y += 28.;
                for t in &g.w.p.tasks {
                    let (_, n, text, c) = TASKS[t.t];
                    txt(
                        &format!(
                            "{}  {}  {}/{}  -  {} coupons",
                            if t.claimed {
                                "[claimed]"
                            } else if t.prog >= n {
                                "[done!]"
                            } else {
                                "[ ]"
                            },
                            text,
                            t.prog,
                            n,
                            c
                        ),
                        90.,
                        y,
                        19.,
                        if t.claimed { GRAY } else { BLACK },
                    );
                    y += 26.
                }
                if g.w.p.tasks.is_empty() {
                    txt("No tasks yet; they arrive at the next new day.", 90., y, 19., GRAY);
                    y += 26.
                }
                if let Some(w) = &g.w.wtask {
                    let (_, n, text) = WTASKS[w.t];
                    y += 10.;
                    txt("Weekly town task (shared, ends Mon 6:00):", 80., y, 22., BLACK);
                    y += 28.;
                    txt(
                        &format!(
                            "{}  {}/{}  -  {} coupons each{}",
                            text,
                            w.prog,
                            n,
                            WEEKLY_COUPONS,
                            if w.claimed.contains(&g.w.p.id) {
                                "  [claimed]"
                            } else if w.prog >= n {
                                "  [done!]"
                            } else {
                                ""
                            }
                        ),
                        90.,
                        y,
                        19.,
                        BLACK,
                    )
                }
            }
            if g.me.jpage == 5 {
                let mut y = 168.;
                for k in 0..4 {
                    let r = g.res(k);
                    let ch = g.w.roster[k];
                    let f = g.w.p.npcs[k].friend;
                    let t = tier(f);
                    txt(&format!("{} the {} ({})", r.name, r.species, PERS_NAME[r.pers]), 80., y, 21., BLACK);
                    y += 22.;
                    txt(
                        &format!(
                            "{}  {}   Birthday {}/{}   Loves {}   Likes {:?} gifts",
                            TIERS[t],
                            "*".repeat(t + 1),
                            BDAYS[ch].1,
                            BDAYS[ch].0,
                            HOBBY[ch],
                            LIKES[r.pers]
                        ),
                        95.,
                        y,
                        16.,
                        Color::new(0.35, 0.3, 0.25, 1.),
                    );
                    y += 18.;
                    let bw = (sw - 260.).min(360.);
                    draw_rectangle(95., y, bw, 8., Color::new(0.85, 0.82, 0.75, 1.));
                    draw_rectangle(95., y, bw * (f.clamp(0, 60) as f32 / 60.), 8., Color::new(0.9, 0.45, 0.5, 1.));
                    y += 24.;
                    if g.w.p.keepsakes.contains(&(ch as u8)) {
                        txt("Keepsake portrait received", 95., y - 6., 15., Color::new(0.4, 0.55, 0.4, 1.));
                        y += 12.
                    }
                }
                let (sc, tips) = Game::home_score(&g.w.p);
                txt(
                    &format!(
                        "Your home: {sc} points ({})  -  best rating rewarded: {}",
                        HOME_TIERS[Game::home_tier(sc)].1,
                        HOME_TIERS[g.w.p.home_best as usize].1
                    ),
                    80.,
                    y + 6.,
                    19.,
                    Color::new(0.3, 0.3, 0.55, 1.),
                );
                if let Some(t) = tips.first() {
                    txt(&format!("Tip: {t}"), 95., y + 30., 16., DARKGRAY)
                }
            }
            if g.me.jpage == 3 {
                let mut y = 168.;
                txt("Keyboard", 80., y, 22., BLACK);
                y += 26.;
                for l in HELP_KEYS {
                    txt(l, 90., y, 17., BLACK);
                    y += 21.
                }
                y += 8.;
                txt("Gamepad", 80., y, 22., BLACK);
                y += 26.;
                for l in HELP_PAD {
                    txt(l, 90., y, 17., BLACK);
                    y += 21.
                }
            }
            if g.me.jpage == 4 {
                let mut y = 195.;
                let wch = ((sw - 180.) / 9.5).max(20.) as usize;
                let topic = (g.me.jcol as usize).min(GUIDE.len() - 1);
                txt(&format!("Topic {}/{}   Up/Down: topics", topic + 1, GUIDE.len()), 80., 170., 18., DARKGRAY);
                for w in wrap(GUIDE[topic], wch) {
                    txt(&w, 80., y, 17., BLACK);
                    y += 22.;
                }
            }
        }
    }
    if let Some(src) = dbg {
        let lines = [
            format!("[{src} clock] {}", now.format("%a %Y-%m-%d %H:%M:%S")),
            format!("gameplay day {} last processed {}", crate::clock::day_id(now), g.w.day),
            format!("scene {:?} pos ({:.2},{:.2}) curve {}", g.me.scene, g.ppos().x, g.ppos().y, g.curve),
            format!("target {:?}", g.target()),
            format!("stock {:?}", g.w.stock.iter().map(|s| (def(s.def).name, s.sold)).collect::<Vec<_>>()),
        ];
        let (px, py, pw) = if narrow { (10., 150., sw - 20.) } else { (sw - 560., 10., 550.) };
        panel(px, py, pw, 30. + 22. * (lines.len() + g.log.len().min(3)) as f32);
        for (k, l) in lines.iter().chain(g.log.iter().rev().take(3)).enumerate() {
            txt(l, px + 12., py + 22. + k as f32 * 22., 17., BLACK)
        }
    }
    if g.me.pad_lost {
        for (k, m) in ["Controller disconnected", "Press Start on a controller to reconnect"].iter().enumerate() {
            let tw = measure_text(m, None, 24, 1.).width;
            draw_rectangle(
                sw / 2. - tw / 2. - 12.,
                sh / 2. - 34. + k as f32 * 40.,
                tw + 24.,
                40.,
                Color::new(0.6, 0.1, 0.1, 0.9),
            );
            txt(m, sw / 2. - tw / 2., sh / 2. - 6. + k as f32 * 40., 24., WHITE)
        }
    }
    if let Some((t, _, _)) = g.me.trans {
        draw_rectangle(0., 0., sw, sh, Color::new(0., 0., 0., 1. - ((t - 0.22).abs() / 0.22).min(1.)))
    }
}

pub(super) fn wrap(t: &str, n: usize) -> Vec<String> {
    let mut v = vec![String::new()];
    for w in t.split(' ') {
        if v.last().unwrap().len() + w.len() + 1 > n {
            v.push(String::new())
        }
        let l = v.last_mut().unwrap();
        if !l.is_empty() {
            l.push(' ')
        }
        l.push_str(w)
    }
    v
}
