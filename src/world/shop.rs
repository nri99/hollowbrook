//! Museum donations and the General Store (selling, buying, stock, displays).
use super::*;

impl Game {
    // ---- museum ----
    pub fn donated_of(&self, k: K) -> usize {
        self.w.donated.iter().filter(|x| def(x.0).k == k).count()
    }
    /// Curator: identify unknown fossils, then donate every pocket item the museum doesn't have yet (shared, once per species).
    pub fn donate(&mut self) {
        let mut lines = vec![];
        while self.w.p.inv.iter().any(|s| s.def == 47) {
            self.remove_one(47);
            let f = FOSSILS[gen_range(0, FOSSILS.len())];
            if !self.add(f) {
                self.add(47);
                lines.push("Your pockets are too full for me to hand it back. Make some room first.".to_string());
                break;
            }
            lines.push(format!("Let me see... this fossil is a {}!", def(f).name));
        }
        let mut cands: Vec<u16> = self
            .w
            .p
            .inv
            .iter()
            .map(|s| s.def)
            .filter(|&d| matches!(def(d).k, K::Fish | K::Bug | K::Fossil | K::Sea | K::Art) && d != 47)
            .collect();
        cands.dedup();
        for d in cands {
            if self.w.donated.iter().any(|x| x.0 == d) {
                continue;
            }
            if self.remove_one(d) {
                self.w.donated.push((d, self.me.id));
                lines.push(format!("A {}! It will make a splendid exhibit.", def(d).name))
            }
        }
        if lines.is_empty() {
            lines.push(
                "Bring me fish, bugs, fossils or paintings the museum doesn't have yet, and I'll put them on display."
                    .into(),
            )
        } else {
            self.snd.push(Snd::Coin)
        }
        let tot = |k: K| DEFS.iter().filter(|d| d.k == k).count();
        lines.push(format!(
            "Collection: {}/{} fish, {}/{} bugs, {}/{} fossils, {}/{} paintings.",
            self.donated_of(K::Fish),
            tot(K::Fish),
            self.donated_of(K::Bug),
            tot(K::Bug),
            self.donated_of(K::Fossil),
            FOSSILS.len(),
            self.donated_of(K::Art),
            ARTS.len()
        ));
        self.me.ui = Ui::Talk { npc: None, who: "Professor Quill the heron".into(), lines, i: 0 };
    }

    // ---- shop ----
    pub fn sell_slot(&mut self, i: usize) {
        if i >= self.w.p.inv.len() {
            return;
        }
        let s = self.w.p.inv.remove(i);
        let v = def(s.def).sell * s.qty as u32;
        self.w.p.money += v;
        self.ev(Ev::Sell, v);
        self.snd.push(Snd::Coin);
        self.note(format!("Sold {} x{} for {v} coins.", def(s.def).name, s.qty))
    }
    pub fn sell_all(&mut self) {
        let (keep, sell): (Vec<Slot>, Vec<Slot>) =
            self.w.p.inv.iter().copied().partition(|s| def(s.def).k == K::Furn || def(s.def).sell == 0);
        let v: u32 = sell.iter().map(|s| def(s.def).sell * s.qty as u32).sum();
        self.w.p.inv = keep;
        self.w.p.money += v;
        self.ev(Ev::Sell, v);
        if v > 0 {
            self.snd.push(Snd::Coin)
        }
        self.note(format!("Sold all your finds for {v} coins."));
    }
    pub fn buy(&mut self, disp: usize) -> Result<(), &'static str> {
        let m0 = self.w.p.money;
        let r = self.buy_inner(disp);
        if r.is_ok() {
            self.ev(Ev::Buy, 1);
            self.w.spent += m0 - self.w.p.money
        }
        r
    }
    pub fn shop_tier(&self) -> u8 {
        SHOP_TIERS.iter().filter(|&&t| self.w.spent >= t).count() as u8
    }
    /// (item, price, sold) for display index: 0-4 stock, 5-6 tools, 7 extra stock, 8 weekly special.
    pub fn display_item(&self, d: usize) -> (u16, u32, bool) {
        match d {
            5 | 6 => {
                let t = TOOL_DISPLAY[d - 5];
                (t, def(t).buy, self.w.p.tools.contains(&t))
            }
            8 => {
                let wk = week_of(self.w.day);
                let s = SPECIALS[wk.rem_euclid(SPECIALS.len() as i64) as usize];
                (s, def(s).buy, self.w.special_week == wk)
            }
            _ => {
                let i = if d == 7 { 5 } else { d };
                self.w.stock.get(i).map_or((0, 0, true), |s| (s.def, def(s.def).buy, s.sold))
            }
        }
    }
    pub(super) fn buy_inner(&mut self, disp: usize) -> Result<(), &'static str> {
        if disp == 8 {
            if self.shop_tier() < 1 {
                return Err("Not available yet.");
            }
            let (d, p, sold) = self.display_item(8);
            if sold {
                return Err("Sold out!");
            }
            if self.w.p.money < p {
                return Err("Not enough coins.");
            }
            if !self.add_furn(d) {
                return Err("Your pockets are full.");
            }
            self.w.p.money -= p;
            self.w.special_week = week_of(self.w.day);
            return Ok(());
        }
        let disp = if disp == 7 {
            if self.w.stock.len() <= 5 {
                return Err("Not available yet.");
            }
            5
        } else {
            if disp >= 5 {
                100 + disp
            } else {
                disp
            }
        };
        if disp >= 100 {
            let disp = disp - 100;
            let t = TOOL_DISPLAY[disp - 5];
            if self.w.p.tools.contains(&t) {
                return Err("You already own one.");
            }
            if self.w.p.money < def(t).buy {
                return Err("Not enough coins.");
            }
            self.w.p.money -= def(t).buy;
            self.w.p.tools.push(t);
            self.w.p.equip = Some(t);
            return Ok(());
        }
        let st = self.w.stock[disp];
        if st.sold {
            return Err("Sold out!");
        }
        let p = def(st.def).buy;
        if self.w.p.money < p {
            return Err("Not enough coins.");
        }
        if self.w.p.inv.len() >= INV_CAP {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= p;
        self.w.stock[disp].sold = true;
        self.add_furn(st.def);
        Ok(())
    }
}
