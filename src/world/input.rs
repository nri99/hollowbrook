//! Frame entry points (update, tick) and per-player input: movement, menus, placement.
use super::*;

impl Game {
    // ---- per-frame update through the input layer ----
    /// Single-player convenience: shared tick + update of the active player.
    pub fn update(&mut self, inp: &Inp, dt: f32, now: NaiveDateTime) -> bool {
        let f = self.tick(dt, now);
        self.update_player(inp, dt);
        f
    }
    /// Shared per-frame simulation (clock, daily refresh, residents, wildlife, effects). Call once per frame.
    pub fn tick(&mut self, dt: f32, now: NaiveDateTime) -> bool {
        self.clock_ms = now.and_utc().timestamp_millis();
        self.t += dt;
        self.hr = now.hour();
        self.mo = now.month();
        self.dom = now.day();
        self.yr = now.year();
        let fresh = self.catch_up(now);
        for e in &mut self.fx {
            e.1 += dt
        }
        self.fx.retain(|e| e.1 < 0.8);
        for s in &mut self.shake {
            s.1 -= dt
        }
        self.shake.retain(|s| s.1 > 0.);
        for h in &mut self.hearts {
            h.1 += dt;
            if h.1 > 0. {
                h.0.y += dt * 0.8
            }
        }
        self.hearts.retain(|h| h.1 < 1.6);
        self.sim_npcs(dt);
        self.sim_chats(dt);
        self.sim_life(dt);
        self.sim_visit(dt);
        self.sim_act(dt);
        fresh
    }
    /// Per-player update for the active player (`activate` first): its own input, camera, UI, movement, actions.
    pub fn update_player(&mut self, inp: &Inp, dt: f32) {
        self.social_tick();
        self.frontier_tick(dt);
        // Dev-only entry to Dustreach until the real gate exists in Hollowbrook.
        if inp.p(KeyCode::F9)
            && self.me.ui == Ui::Play
            && matches!(self.me.scene, Scene::Town | Scene::Frontier)
            && std::env::var("HOLLOWBROOK_DEBUG").is_ok()
        {
            self.go(if self.me.scene == Scene::Frontier { Scene::Town } else { Scene::Frontier })
        }
        self.me.swing = (self.me.swing - dt).max(0.);
        self.act_step();
        if self.w.p.task_day != self.w.day {
            self.assign_tasks()
        }
        if self.w.wtask.is_none() {
            self.assign_weekly()
        }
        if self.me.spd > 0.1 {
            self.me.anim += dt * self.me.spd * 3.2
        }
        self.sim_fish(dt);
        let pp = self.ppos();
        self.me.cam = self.me.cam.lerp(pp, 1. - (-12. * dt).exp());
        if let Some((t, to, done)) = self.me.trans {
            let t = t + dt;
            let mut done = done;
            if t >= 0.22 && !done {
                self.swap(to);
                done = true
            }
            self.me.trans = if t >= 0.45 { None } else { Some((t, to, done)) };
            self.me.spd = 0.;
            return;
        }
        if self.ui_input(inp) {
            self.me.spd = 0.;
            return;
        }
        if self.me.flight.is_some() {
            self.flight_input(inp, dt);
            self.me.spd = 0.;
            return;
        }
        if self.me.placing.is_some() {
            self.place_input(inp);
            self.me.spd = 0.;
            return;
        }
        if let Some(idx) = self.me.mplace {
            if inp.p(KeyCode::Escape) {
                self.me.mplace = None;
                self.note("Placement cancelled.")
            } else if self.me.scene == Scene::Town && (inp.p(KeyCode::Enter) || inp.p(KeyCode::KpEnter)) {
                match self.mp_start(idx) {
                    Ok(()) => {
                        self.me.mplace = None;
                        self.snd.push(Snd::Blip);
                        self.note(format!("The {} site is set! Anyone can donate at its sign.", MPROJECTS[idx].0))
                    }
                    Err(e) => self.note(e),
                }
            }
        }
        use KeyCode::*;
        let mut d = Vec2::ZERO;
        if inp.d(W) || inp.d(Up) {
            d.y -= 1.
        }
        if inp.d(S) || inp.d(Down) {
            d.y += 1.
        }
        if inp.d(A) || inp.d(Left) {
            d.x -= 1.
        }
        if inp.d(D) || inp.d(Right) {
            d.x += 1.
        }
        if self.me.dazed > 0. || self.me.standoff > 0. {
            d = Vec2::ZERO; // dazed by an ambush
        }
        self.step_move(d, inp.d(LeftShift) || inp.d(RightShift), dt);
        if self.me.trans.is_some() {
            return;
        }
        if inp.p(E) {
            self.interact()
        }
        if inp.p(Space) {
            if self.me.standoff > 0. {
                self.standoff_press()
            } else {
                self.use_tool()
            }
        }
        if inp.p(Tab) || inp.p(Q) {
            self.cycle_tool()
        }
        if inp.p(I) {
            self.me.ui = Ui::Inv(0)
        }
        if inp.p(J) {
            self.me.ui = Ui::Journal
        }
        if inp.p(F) || inp.p(G) {
            match self.target() {
                Some(Tg::Furn(uid, _)) if self.me.scene == Scene::Room(self.home()) && uid != 0 => {
                    if inp.p(F) {
                        self.start_move(uid)
                    } else {
                        self.pick_furn(uid)
                    }
                }
                _ => self.note("Face your own furniture to move (F) or pick it up (G)."),
            }
        }
        if inp.p(Escape) && self.me.fish.is_some() {
            self.me.fish = None;
            self.note("You reel in your line.")
        }
    }
    pub(super) fn ui_input(&mut self, inp: &Inp) -> bool {
        use KeyCode::*;
        if self.me.ui == Ui::Play {
            return false;
        }
        let (up, down) = (inp.p(Up) || inp.p(W), inp.p(Down) || inp.p(S));
        let (left, right) = (inp.p(Left) || inp.p(A), inp.p(Right) || inp.p(D));
        let ok = inp.p(Enter) || inp.p(KpEnter) || inp.p(E);
        let esc = inp.p(Escape);
        if let Ui::Talk { npc: Some(j), .. } = self.me.ui {
            if inp.p(X) && self.w.moving[j] != 0 {
                self.w.moving[j] = 0;
                self.w.arrived[j] = self.w.day;
                self.w.p.npcs[j].friend += 1;
                let r = self.res(j);
                self.me.ui = Ui::Talk {
                    npc: Some(j),
                    who: format!("{} the {}", r.name, r.species),
                    lines: vec!["You really want me to stay? ...Okay. I'll stay!".into()],
                    i: 0,
                };
                return true;
            }
        }
        if let Ui::Talk { npc: Some(j), .. } = self.me.ui {
            if inp.p(G) {
                if self.w.p.inv.is_empty() {
                    self.note("You have nothing to give.")
                } else {
                    self.me.ui = Ui::Gift(j, 0)
                }
                return true;
            }
        }
        if let Ui::Talk { lines, i, .. } = &mut self.me.ui {
            if ok || inp.p(Space) {
                *i += 1;
                if *i >= lines.len() {
                    self.me.ui =
                        if self.pell_talked && self.me.scene == Scene::Islet { Ui::Travel(false) } else { Ui::Play };
                    self.pell_talked = false
                }
            } else if esc {
                self.me.ui = Ui::Play
            }
            return true;
        }
        let n = self.w.p.inv.len();
        let mv = |s: usize, len: usize| {
            if up && s > 0 {
                s - 1
            } else if down && s + 1 < len {
                s + 1
            } else {
                s.min(len.saturating_sub(1))
            }
        };
        match self.me.ui {
            Ui::Invitation => {
                if esc {
                    if let Some(v) = &mut self.w.p.invitation {
                        v.state = 4;
                    }
                    self.me.ui = Ui::Play;
                } else if ok {
                    let r = self.accept_social();
                    self.me.ui = Ui::Play;
                    if let Err(e) = r {
                        self.note(e)
                    }
                }
            }
            Ui::Contest(s) => {
                let slots = self.contest_slots();
                let s = mv(s, slots.len());
                self.me.ui = Ui::Contest(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    if let Some(&slot) = slots.get(s) {
                        match self.contest_submit(slot) {
                            Ok(score) => self.note(format!("New best: {score} points. Catch released.")),
                            Err(e) => self.note(e),
                        }
                    } else {
                        self.note("Bring a measured catch. Its size is shown in pockets.")
                    }
                }
            }
            Ui::Inv(s) => {
                let s = mv(s, n);
                self.me.ui = Ui::Inv(s);
                if esc || inp.p(I) {
                    self.me.ui = Ui::Play
                } else if ok && s < n {
                    let sl = self.w.p.inv[s];
                    if def(sl.def).k == crate::data::K::Cloth {
                        self.me.ui = Ui::Play;
                        self.wear(sl.uid)
                    } else if def(sl.def).k != crate::data::K::Furn {
                        self.note(format!("{} sells for {} coins at the shop.", def(sl.def).name, def(sl.def).sell))
                    } else if self.me.scene == Scene::Room(self.home()) {
                        self.me.ui = Ui::Play;
                        self.start_place(sl.uid)
                    } else {
                        self.note("Furniture can be placed inside your home.")
                    }
                }
            }
            Ui::Sell(s) => {
                let s = mv(s, n + 4);
                if ok && s == n + 3 {
                    self.me.ui = Ui::Keeper(0);
                    return true;
                }
                if ok && s == n + 1 {
                    self.me.ui = Ui::Catalog(0);
                    return true;
                }
                if ok && s == n + 2 {
                    self.me.ui = Ui::Tint(0);
                    return true;
                }
                self.me.ui = Ui::Sell(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s < n {
                    self.sell_slot(s)
                } else if ok && s == n {
                    if self.w.p.debt > 0 {
                        self.pay_debt()
                    } else if (self.w.p.home as usize) < EXPAND_COST.len() {
                        self.me.ui = Ui::ConfirmExpand
                    } else {
                        self.note("Your home is already as big as it gets!")
                    }
                } else if inp.p(A) || inp.p(X) {
                    self.sell_all()
                }
            }
            Ui::Confirm(d) => {
                if ok || inp.p(Y) {
                    self.me.ui = Ui::Play;
                    match self.buy(d) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note("Thank you for your purchase!")
                        }
                        Err(e) => self.note(e),
                    }
                } else if esc || inp.p(N) {
                    self.me.ui = Ui::Play
                }
            }
            Ui::Storage(c, s) => {
                let c = if left {
                    0
                } else if right {
                    1
                } else {
                    c
                };
                let len = if c == 0 { n } else { self.w.p.storage.len() };
                let s = mv(s, len);
                self.me.ui = Ui::Storage(c, s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s < len {
                    self.transfer(c, s)
                }
            }
            Ui::Journal => {
                if esc || inp.p(J) {
                    self.me.ui = Ui::Play
                }
                if inp.p(Tab) || inp.p(R) {
                    self.me.jpage = (self.me.jpage + 1) % 6;
                    self.me.jcol = 0;
                }
                if self.me.jpage == 4 {
                    if up {
                        self.me.jcol = self.me.jcol.saturating_sub(1);
                    }
                    if down {
                        self.me.jcol = (self.me.jcol + 1).min(GUIDE.len() as u8 - 1);
                    }
                }
                if self.me.jpage == 0 {
                    if left {
                        self.me.jcol = (self.me.jcol + 2) % 3
                    }
                    if right {
                        self.me.jcol = (self.me.jcol + 1) % 3
                    }
                }
                let (mv0, sv0, mu0) = (self.music_vol, self.sfx_vol, self.muted);
                if self.me.jpage == 2 {
                    if left {
                        self.music_vol = (self.music_vol - 0.1).max(0.)
                    }
                    if right {
                        self.music_vol = (self.music_vol + 0.1).min(1.)
                    }
                    if down {
                        self.sfx_vol = (self.sfx_vol - 0.1).max(0.)
                    }
                    if up {
                        self.sfx_vol = (self.sfx_vol + 0.1).min(1.)
                    }
                    if inp.p(X) {
                        self.muted = !self.muted
                    }
                    self.music_vol = (self.music_vol * 10.).round() / 10.;
                    self.sfx_vol = (self.sfx_vol * 10.).round() / 10.;
                }
                if (mv0, sv0, mu0) != (self.music_vol, self.sfx_vol, self.muted) {
                    self.settings_dirty = true
                }
            }
            Ui::Gift(j, s) => {
                let s = mv(s, n);
                self.me.ui = Ui::Gift(j, s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s < n {
                    self.give_gift(j, s)
                }
            }
            Ui::MPick(s) => {
                let s = mv(s, MPROJECTS.len());
                self.me.ui = Ui::MPick(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.me.ui = Ui::Play;
                    self.me.mplace = Some(s);
                    if self.me.scene != Scene::Town {
                        self.go(Scene::Town)
                    }
                    self.note("Face a spot outside (green = OK), Enter places.")
                }
            }
            Ui::Paint(s) => {
                let s = mv(s, PAINTS.len());
                self.me.ui = Ui::Paint(s);
                if esc {
                    self.me.ui = Ui::Civic(0)
                } else if ok {
                    match self.repaint(s) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.me.ui = Ui::Play;
                            self.note(format!("Your house will be {} by the time you step outside!", PAINTS[s].0))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Civic(s) => {
                let s = mv(s, 4);
                self.me.ui = Ui::Civic(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    match s {
                        0 => self.me.ui = Ui::Policy(0),
                        1 => {
                            if self.me.id != LEADER {
                                self.me.ui = Ui::Play;
                                self.note("Only the town leader can start public projects.")
                            } else if self.w.mproject.is_some() {
                                self.me.ui = Ui::Play;
                                self.note("A funded project is already underway.")
                            } else {
                                self.me.ui = Ui::MPick(0)
                            }
                        }
                        3 => self.me.ui = Ui::Paint(0),
                        _ => {
                            let lines = self.board_lines();
                            self.me.ui = Ui::Talk { npc: None, who: ADMIN.into(), lines, i: 0 }
                        }
                    }
                }
            }
            Ui::Policy(s) => {
                let s = mv(s, POLICIES.len() + 1);
                self.me.ui = Ui::Policy(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.me.ui = Ui::PolicyOk(s)
                }
            }
            Ui::PolicyOk(s) => {
                if esc {
                    self.me.ui = Ui::Policy(s)
                } else if ok {
                    let r = self.enact((s < POLICIES.len()).then_some(s));
                    self.me.ui = Ui::Play;
                    match r {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note(self.pol().map_or("Policy repealed. Normal hours apply.".to_string(), |p| {
                                format!("{} is now in effect.", p.name)
                            }))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Act(s) => {
                let s = mv(s, ACTS.len());
                self.me.ui = Ui::Act(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.me.ui = Ui::Play;
                    match self.act_start(s) {
                        Ok(()) => {
                            self.snd.push(Snd::Blip);
                            self.note(format!("{} started!", ACTS[s].0))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Keeper(s) => {
                let items = self.keeper_items();
                let s = mv(s, items.len());
                self.me.ui = Ui::Keeper(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    let a = items[s].1;
                    match a {
                        0 => {
                            self.me.ui = Ui::Talk {
                                npc: None,
                                who: "Mr. Tumble the tortoise".into(),
                                lines: KEEPER.iter().map(|s| s.to_string()).collect(),
                                i: 0,
                            }
                        }
                        1..=4 => {
                            if self.w.p.money < SEED_PRICE {
                                self.note("Not enough coins.")
                            } else if self.add(SEEDS[a as usize - 1]) {
                                self.w.p.money -= SEED_PRICE;
                                self.w.spent += SEED_PRICE;
                                self.snd.push(Snd::Coin);
                                self.note(format!("Bought {}.", def(SEEDS[a as usize - 1]).name))
                            }
                        }
                        5 => {
                            let p = def(CAN).buy;
                            if self.w.p.money < p {
                                self.note("Not enough coins.")
                            } else {
                                self.w.p.money -= p;
                                self.w.spent += p;
                                self.w.p.tools.push(CAN);
                                self.w.p.equip = Some(CAN);
                                self.snd.push(Snd::Coin);
                                self.note("Bought a Watering Can (equipped). Use it with Space / X.")
                            }
                        }
                        _ => {
                            let t = UPGRADABLE[(a - 10) as usize];
                            match self.upgrade_tool(t) {
                                Ok(lv) => {
                                    self.snd.push(Snd::Coin);
                                    self.note(format!(
                                        "Your {} is now a {}{}!",
                                        def(t).name,
                                        LV_NAME[lv as usize],
                                        def(t).name
                                    ))
                                }
                                Err(e) => self.note(e),
                            }
                        }
                    }
                }
            }
            Ui::Camp(s) => {
                let s = mv(s, 5);
                self.me.ui = Ui::Camp(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    let v = &VISITORS[self.visitor()];
                    let who = format!("{} the {}", v.name, v.species);
                    match s {
                        0..=2 => match self.camp_buy(s) {
                            Ok(n) => {
                                self.snd.push(Snd::Coin);
                                self.note(format!("Bought {n}."))
                            }
                            Err(e) => self.note(e),
                        },
                        3 => match self.camp_favor() {
                            Ok(n) => {
                                self.snd.push(Snd::Coin);
                                self.me.ui = Ui::Talk {
                                    npc: None,
                                    who,
                                    lines: vec![format!("You found them! Please take this {n} as thanks.")],
                                    i: 0,
                                }
                            }
                            Err(e) => self.me.ui = Ui::Talk { npc: None, who, lines: vec![e], i: 0 },
                        },
                        _ => {
                            self.ev(Ev::Camp, 1);
                            let lines = v.lines.iter().map(|l| l.to_string()).collect();
                            self.me.ui = Ui::Talk { npc: None, who, lines, i: 0 }
                        }
                    }
                }
            }
            Ui::Nav(s) => {
                let s = mv(s, PLANETS.len() + 1);
                self.me.ui = Ui::Nav(s);
                if esc || (ok && s == PLANETS.len()) {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.me.ui = Ui::Play;
                    if let Err(e) = self.plot_jump(s as u8) {
                        self.note(e)
                    }
                }
            }
            Ui::Forge(s) => {
                let s = mv(s, ARMOR.len() + 1);
                self.me.ui = Ui::Forge(s);
                if esc || (ok && s == ARMOR.len()) {
                    self.me.ui = Ui::Play
                } else if ok {
                    if let Err(e) = self.forge(s) {
                        self.note(e)
                    }
                }
            }
            Ui::Bounty(s) => {
                let s = mv(s, 8);
                self.me.ui = Ui::Bounty(s);
                if ok && (4..7).contains(&s) {
                    if let Err(e) = self.buy_gear(s - 4) {
                        self.note(e)
                    }
                } else if esc || (ok && s == 7) {
                    self.me.ui = Ui::Play
                } else if ok && s < 3 {
                    match self.take_bounty(s as u8) {
                        Ok(m) => {
                            self.note(m);
                            self.me.ui = Ui::Play
                        }
                        Err(e) => self.note(e),
                    }
                } else if ok && s == 3 {
                    if self.w.p.bounty.as_ref().is_some_and(|b| b.caught) {
                        let _ = self.turn_in();
                    } else {
                        self.drop_bounty()
                    }
                }
            }
            Ui::Kiosk(s) => {
                let s = mv(s, 4 + KIOSK_ITEMS.len());
                self.me.ui = Ui::Kiosk(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    if s < 4 {
                        match self.claim(s) {
                            Ok(c) => {
                                self.snd.push(Snd::Coin);
                                self.note(format!("Claimed {c} coupons!"))
                            }
                            Err(e) => self.note(e),
                        }
                    } else {
                        match self.kiosk_buy(s - 4) {
                            Ok(n) => {
                                self.snd.push(Snd::Coin);
                                self.note(format!("Traded coupons for {n}."))
                            }
                            Err(e) => self.note(e),
                        }
                    }
                }
            }
            Ui::Title(s) => {
                let s = mv(s, 5);
                self.me.ui = Ui::Title(s);
                if ok {
                    match s {
                        0 => {
                            self.me.ui = Ui::Play;
                            if self.fresh {
                                self.note(
                                    "Welcome to Hollowbrook! Press J for the journal: controls and a guide are inside.",
                                )
                            }
                        }
                        1 => {
                            self.me.ui = Ui::Play;
                            self.net_req = 1
                        }
                        2 => {
                            self.me.ui = Ui::Play;
                            self.net_req = 2
                        }
                        3 => self.me.ui = Ui::Help,
                        _ => self.quit = true,
                    }
                }
            }
            Ui::Help => {
                if ok || esc {
                    self.me.ui = Ui::Title(3)
                }
            }
            Ui::Fund => {
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    match self.mp_fund(DONATE_STEP) {
                        Ok(a) => {
                            self.snd.push(Snd::Coin);
                            self.note(format!("Donated {a} coins."))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::ConfirmExpand => {
                if ok || inp.p(Y) {
                    self.me.ui = Ui::Play;
                    match self.expand_home() {
                        Ok(()) => {
                            self.snd.push(Snd::Door);
                            self.note(format!("Your home was expanded! Balance: {} coins, no deadline.", self.w.p.debt))
                        }
                        Err(e) => self.note(e),
                    }
                } else if esc || inp.p(N) {
                    self.me.ui = Ui::Play
                }
            }
            Ui::Travel(go_out) => {
                if ok || inp.p(Y) {
                    self.me.ui = Ui::Play;
                    if go_out {
                        if self.w.p.ferry > 0 {
                            self.w.p.ferry -= 1;
                            self.note("Ferry Pass used. Enjoy the trip!");
                            self.go(Scene::Islet)
                        } else if self.w.p.money < FARE {
                            self.note("Not enough coins for the fare.")
                        } else {
                            self.w.p.money -= FARE;
                            self.go(Scene::Islet)
                        }
                    } else {
                        self.go(Scene::Town)
                    }
                } else if esc || inp.p(N) {
                    self.me.ui = Ui::Play
                }
            }
            Ui::IStall(s) => {
                let s = mv(s, ISTALL_ITEMS.len());
                self.me.ui = Ui::IStall(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    match self.istall_buy(s) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note(format!("Traded for a {}!", def(ISTALL_ITEMS[s].0).name))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Tint(s) => {
                let len = self.tint_targets().len() + 2;
                let s = mv(s, len);
                self.me.ui = Ui::Tint(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    match self.customize(s) {
                        Ok(m) => {
                            self.snd.push(Snd::Coin);
                            self.note(m)
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Catalog(s) => {
                let s = mv(s, self.w.p.catalog.len());
                self.me.ui = Ui::Catalog(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s < self.w.p.catalog.len() {
                    let d = self.w.p.catalog[s];
                    match self.order(d) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note(format!("Ordered a {}.", def(d).name))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Mail(s) => {
                let s = mv(s, self.w.p.mail.len());
                self.me.ui = Ui::Mail(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.open_letter(s)
                }
            }
            Ui::Rack(s) => {
                let s = mv(s, CLOTHES.len() + 1);
                self.me.ui = Ui::Rack(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s == CLOTHES.len() {
                    if self.w.p.money < HAIR_FEE {
                        self.note("Not enough coins.")
                    } else {
                        self.w.p.money -= HAIR_FEE;
                        self.w.p.hair = (self.w.p.hair + 1) % HAIRS.len() as u8;
                        self.snd.push(Snd::Coin);
                        self.note("New hairstyle!")
                    }
                } else if ok {
                    match self.buy_cloth(CLOTHES[s]) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note(format!("Bought the {}. Open your pockets to wear it.", def(CLOTHES[s]).name))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            _ => {}
        }
        true
    }
    pub(super) fn place_input(&mut self, inp: &Inp) {
        use KeyCode::*;
        let dims = self.rdims(self.home());
        let Some(p) = self.me.placing.as_mut() else { return };
        let df = def(p.f.def);
        if inp.p(A) || inp.p(Left) {
            p.f.x -= 1
        }
        if inp.p(D) || inp.p(Right) {
            p.f.x += 1
        }
        if !df.wall {
            if inp.p(W) || inp.p(Up) {
                p.f.z -= 1
            }
            if inp.p(S) || inp.p(Down) {
                p.f.z += 1
            }
            if inp.p(R) && df.rot {
                p.f.rot = (p.f.rot + 1) % 4
            }
        }
        let (w, d) = dims;
        p.f.x = p.f.x.clamp(-1, w);
        p.f.z = if df.wall { -1 } else { p.f.z.clamp(-1, d) };
        if inp.p(Enter) || inp.p(KpEnter) {
            match self.place_confirm() {
                Ok(()) => {
                    self.snd.push(Snd::Blip);
                    self.note("Placed!")
                }
                Err(e) => self.note(e),
            }
        } else if inp.p(Escape) {
            self.place_cancel();
            self.note("Cancelled.")
        }
    }
}
