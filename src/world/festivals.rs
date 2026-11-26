//! Festivals, art stall, Harvest Fair, resident home visits, furniture customization, money projects.
use super::*;

impl Game {
    pub fn fest(&self) -> Option<usize> {
        fest_of(self.mo, self.dom)
    }
    /// Festival host: explains the event and gives the yearly keepsake once the task is done (once per player per year).
    pub fn fest_talk(&mut self) {
        let Some(f) = self.fest() else { return };
        let fe = &FESTS[f];
        let mut lines = vec![format!("Welcome to the {}! {}", fe.name, fe.about)];
        match self.fest_claim() {
            Ok(n) => {
                self.snd.push(Snd::Coin);
                lines.push(format!("Wonderful, thank you! Here is a {n} to remember this year's {}.", fe.name))
            }
            Err(e) => lines.push(e),
        }
        self.me.ui = Ui::Talk { npc: None, who: "Hazel Quill (festival host)".into(), lines, i: 0 }
    }
    pub fn fest_claim(&mut self) -> Result<&'static str, String> {
        let Some(f) = self.fest() else { return Err("No festival today.".into()) };
        let fe = &FESTS[f];
        if self.w.p.fest_years.contains(&(f as u8, self.yr)) {
            return Err(format!("You've already joined this year's {}. Enjoy the festivities!", fe.name));
        }
        let backup = self.w.p.inv.clone();
        let ok = match fe.task {
            FTask::Fruit(n) => {
                let d = self.w.native;
                let have: u16 = self.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum();
                have >= n && {
                    for _ in 0..n {
                        self.remove_one(d);
                    }
                    true
                }
            }
            FTask::Branch(n) => {
                let have: u16 = self.w.p.inv.iter().filter(|s| s.def == 80).map(|s| s.qty).sum();
                have >= n && {
                    for _ in 0..n {
                        self.remove_one(80);
                    }
                    true
                }
            }
            FTask::Fish(n) => {
                let fish: Vec<u16> = self.w.p.inv.iter().filter(|s| def(s.def).k == K::Fish).map(|s| s.def).collect();
                fish.len() as u16 >= n && {
                    for d in fish.into_iter().take(n as usize) {
                        self.remove_one(d);
                    }
                    true
                }
            }
            FTask::TalkAll => self.w.p.npcs.iter().all(|n| n.talked),
        };
        if !ok {
            return Err(format!("Could you help? {}", fe.ask));
        }
        if !self.add_furn(fe.reward) {
            self.w.p.inv = backup;
            return Err("Your pockets are full. Make room for your keepsake!".into());
        }
        self.w.p.fest_years.push((f as u8, self.yr));
        Ok(def(fe.reward).name)
    }
    /// Buy this week's painting (one per player per week, from your own coins).
    pub fn art_buy(&mut self) -> Result<&'static str, &'static str> {
        if !self.art_open() {
            return Err("The art stall only opens on Saturdays.");
        }
        let wk = (self.w.day - 1).div_euclid(7);
        if self.w.p.art_week == wk {
            return Err("You've already bought this week's painting.");
        }
        if self.w.p.money < ART_PRICE {
            return Err("Not enough coins.");
        }
        let d = art_of_week(self.w.day);
        if !self.add(d) {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= ART_PRICE;
        self.w.p.art_week = wk;
        Ok(def(d).name)
    }
    /// Note when a furniture collection is complete in the active player's home.
    pub fn sets_done(&self) -> Vec<&'static str> {
        SETS.iter().filter(|s| s.1.iter().all(|d| self.w.p.room.iter().any(|f| f.def == *d))).map(|s| s.0).collect()
    }
    pub fn fair_prize(&mut self) {
        if !self.fair() {
            return;
        }
        if self.w.p.fair_year == self.yr {
            return self.note("You already got your prize this year. Enjoy the fair!");
        }
        if self.w.p.inv.len() >= INV_CAP {
            return self.note("Make room in your pockets first!");
        }
        let d = FURN_IDS[gen_range(0, FURN_IDS.len())];
        self.add_furn(d);
        self.w.p.fair_year = self.yr;
        self.snd.push(Snd::Coin);
        self.note(format!("Harvest Fair prize: a {}!", def(d).name));
    }

    /// Where a visiting resident stands inside a home (next to the door).
    pub fn visit_spot(&self, b: usize) -> Vec2 {
        let (w, d) = self.rdims(b);
        vec2((w / 2) as f32 - 0.5, d as f32 - 1.5)
    }
    /// Start a visit by resident `i` to building `b` (the visited player's home). Provisional rules decide when this is called.
    pub fn start_visit(&mut self, i: usize, b: usize) {
        if self.visit.is_some() {
            return;
        }
        self.visit = Some((i, b, 120.));
        self.npcs[i].inside = true;
        self.npcs[i].path.clear();
        self.snd.push(Snd::Door);
        let vname = self.res(i).name;
        for pl in std::iter::once(&mut self.me).chain(self.others.iter_mut()) {
            if pl.scene == Scene::Room(b) {
                pl.msgs.push((format!("Knock knock! {} dropped by for a visit.", vname), self.t))
            }
        }
    }
    /// Visits (provisional): while a player is in their own home, a resident who is out, friendship >= 5, not yet visited
    /// that player today, may knock (about once every 10 s of eligible time). Ends after 120 s or when nobody is home.
    pub(super) fn sim_visit(&mut self, dt: f32) {
        if let Some((i, b, left)) = self.visit {
            let occupied = std::iter::once(&self.me).chain(self.others.iter()).any(|p| p.scene == Scene::Room(b));
            if left - dt <= 0. || !occupied {
                self.visit = None;
                self.npcs[i].inside = false;
                self.npcs[i].pos = door_stand(b);
                self.npcs[i].wait = 1.;
            } else {
                self.visit = Some((i, b, left - dt))
            }
            return;
        }
        if gen_range(0, 600) != 0 {
            return;
        }
        let hosts: Vec<usize> = std::iter::once(&self.me)
            .chain(self.others.iter())
            .filter(|p| p.scene == Scene::Room(home_of(p.id)))
            .map(|p| p.id)
            .collect();
        for id in hosts {
            let pr = if self.w.p.id == id {
                &self.w.p
            } else {
                match self.w.others.iter().find(|p| p.id == id) {
                    Some(p) => p,
                    None => continue,
                }
            };
            if let Some(i) = (0..4).find(|&i| {
                self.loc(i) == Loc::Out
                    && self.seek_hide_position(i).is_none()
                    && !self.npcs[i].inside
                    && pr.npcs[i].friend >= 5
                    && !pr.npcs[i].visited
            }) {
                if self.w.p.id == id {
                    self.w.p.npcs[i].visited = true
                } else if let Some(p) = self.w.others.iter_mut().find(|p| p.id == id) {
                    p.npcs[i].visited = true
                }
                self.start_visit(i, home_of(id));
                return;
            }
        }
    }

    pub fn tint_of(pr: &Profile, uid: u32) -> u8 {
        pr.tints.iter().find(|t| t.0 == uid).map_or(0, |t| t.1)
    }
    /// Profile owning a home building (or the active one).
    pub fn owner(&self, o: usize) -> &Profile {
        if self.w.p.id == o {
            &self.w.p
        } else {
            self.w.others.iter().find(|p| p.id == o).unwrap_or(&self.w.p)
        }
    }
    /// Furniture in pockets that can be re-coloured (pocket slot indices).
    pub fn tint_targets(&self) -> Vec<usize> {
        (0..self.w.p.inv.len()).filter(|&i| self.w.p.inv[i].uid != 0 && def(self.w.p.inv[i].def).k == K::Furn).collect()
    }
    /// Counter "Customize": entry `sel` = a furniture item (cycle tint), then wallpaper, then flooring (cycle palette).
    pub fn customize(&mut self, sel: usize) -> Result<String, &'static str> {
        let t = self.tint_targets();
        let fee = if sel < t.len() { TINT_FEE } else { DECOR_FEE };
        if self.w.p.money < fee {
            return Err("Not enough coins.");
        }
        self.w.p.money -= fee;
        if sel < t.len() {
            let s = self.w.p.inv[t[sel]];
            let cur = Game::tint_of(&self.w.p, s.uid);
            let nt = (cur + 1) % TINTS.len() as u8;
            self.w.p.tints.retain(|x| x.0 != s.uid);
            if nt != 0 {
                self.w.p.tints.push((s.uid, nt))
            }
            Ok(format!("Your {} is now {}.", def(s.def).name, TINT_NAMES[nt as usize]))
        } else if sel == t.len() {
            self.w.p.wall = (self.w.p.wall + 1) % WALLS.len() as u8;
            Ok("New wallpaper installed at home.".into())
        } else {
            self.w.p.floor = (self.w.p.floor + 1) % FLOORS.len() as u8;
            Ok("New flooring installed at home.".into())
        }
    }

    /// Fully funded money projects are built at the daily boundary.
    pub fn finish_mproject(&mut self) {
        if let Some(m) = self.w.mproject.clone() {
            if m.funded >= MPROJECTS[m.idx].1 {
                self.w.built.push(Built { kind: MPROJECTS[m.idx].2, x: m.x, z: m.z, day: self.w.day });
                self.w.mproject = None;
                self.mail_all(
                    "Town notice board",
                    &format!("The new {} is finished! Thank you to everyone who donated.", MPROJECTS[m.idx].0),
                )
            }
        }
    }
}
