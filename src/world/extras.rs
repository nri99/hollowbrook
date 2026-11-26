//! Campground visitor, initiatives (tasks, coupons, kiosk), daily surprises.
use super::*;

impl Game {
    /// Harvest Fair: one free furniture prize per player per year (needs a free pocket slot; nothing is lost if full).
    // ---- Campground ----
    pub fn visitor(&self) -> usize {
        visitor_of(self.w.day)
    }
    pub fn camp_open(&self) -> bool {
        (CAMP_HOURS.0..CAMP_HOURS.1).contains(&self.hr)
    }
    /// Buy good `k` from today's visitor (one unit of each per day, shared between players).
    pub fn camp_buy(&mut self, k: usize) -> Result<&'static str, &'static str> {
        if !self.camp_open() {
            return Err("The camper is asleep in the tent.");
        }
        let d = VISITORS[self.visitor()].goods[k];
        if self.w.camp_sold.contains(&d) {
            return Err("Sold out for today.");
        }
        let p = def(d).buy;
        if self.w.p.money < p {
            return Err("Not enough coins.");
        }
        if !self.add_furn(d) {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= p;
        self.w.camp_sold.push(d);
        self.ev(Ev::Camp, 1);
        Ok(def(d).name)
    }
    pub fn favor_item(&self) -> (u16, u16) {
        let f = VISITORS[self.visitor()].favor;
        (if f.0 == 0 { self.w.native } else { f.0 }, f.1)
    }
    /// Hand over the favor items for today's visitor's keepsake (once per player per visitor).
    pub fn camp_favor(&mut self) -> Result<&'static str, String> {
        let v = self.visitor();
        if self.w.p.camp_favors.contains(&(v as u8)) {
            return Err("You've already helped me. Thank you again!".into());
        }
        let (d, n) = self.favor_item();
        let have: u16 = self.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum();
        if have < n {
            return Err(format!("{} (You have {have} of {n} {}.)", VISITORS[v].ask, def(d).name));
        }
        let backup = self.w.p.inv.clone();
        for _ in 0..n {
            self.remove_one(d);
        }
        if !self.add_furn(VISITORS[v].reward) {
            self.w.p.inv = backup;
            return Err("Your pockets are full. Make room for my thank-you gift!".into());
        }
        self.w.p.camp_favors.push(v as u8);
        self.ev(Ev::Camp, 1);
        Ok(def(VISITORS[v].reward).name)
    }
    // ---- Initiatives ----
    /// Can template `t` be completed today by the active player? (tools owned, resources present, places open)
    pub fn task_ok(&self, t: usize) -> bool {
        let tools = &self.w.p.tools;
        let money = self.w.p.money;
        match TASKS[t].0 {
            Ev::Fish => tools.contains(&61),
            Ev::Dig => tools.contains(&62) && self.w.digs.len() >= 2,
            Ev::Rock => tools.contains(&62),
            Ev::Weed => self.w.weeds.len() >= 4,
            Ev::Fruit => self.w.trees.iter().filter(|t| t.fruit).count() >= 3,
            Ev::Buy => money >= 600,
            Ev::Activity => money >= FARE + 100 || self.w.p.ferry > 0,
            Ev::Gift => true,
            Ev::Bug => tools.contains(&60),
            Ev::Water => tools.contains(&CAN) && self.w.garden.len() >= 3,
            _ => true,
        }
    }
    /// Give the active player three attainable tasks for today (once per day).
    pub fn assign_tasks(&mut self) {
        if self.w.p.task_day == self.w.day {
            return;
        }
        let mut pool: Vec<usize> = (0..TASKS.len()).filter(|&t| self.task_ok(t)).collect();
        let mut v = vec![];
        while v.len() < 3 && !pool.is_empty() {
            let t = pool.remove(gen_range(0, pool.len()));
            v.push(PTask { t, prog: 0, claimed: false })
        }
        self.w.p.tasks = v;
        self.w.p.task_day = self.w.day;
    }
    pub fn assign_weekly(&mut self) {
        let wk = week_of(self.w.day);
        if self.w.wtask.as_ref().map_or(true, |w| w.week != wk) {
            self.w.wtask =
                Some(WTask { t: (wk.rem_euclid(WTASKS.len() as i64)) as usize, prog: 0, week: wk, claimed: vec![] })
        }
    }
    /// Count progress for the active player's tasks and the shared weekly task.
    pub fn ev(&mut self, e: Ev, n: u32) {
        let day = self.w.day;
        if self.w.p.task_day == day {
            for t in &mut self.w.p.tasks {
                if TASKS[t.t].0 == e {
                    t.prog = (t.prog + n).min(TASKS[t.t].1)
                }
            }
        }
        if let Some(w) = &mut self.w.wtask {
            if WTASKS[w.t].0 == e {
                w.prog = (w.prog + n).min(WTASKS[w.t].1)
            }
        }
    }
    /// Claim a finished daily task (index 0-2) or the weekly task (index 3). One time only.
    pub fn claim(&mut self, k: usize) -> Result<u32, &'static str> {
        if k < 3 {
            let Some(t) = self.w.p.tasks.get_mut(k) else { return Err("No task there.") };
            if t.claimed {
                return Err("Already claimed.");
            }
            if t.prog < TASKS[t.t].1 {
                return Err("Not finished yet.");
            }
            t.claimed = true;
            let c = TASKS[t.t].3;
            self.w.p.coupons += c;
            return Ok(c);
        }
        let id = self.w.p.id;
        let Some(w) = self.w.wtask.as_mut() else { return Err("No weekly task.") };
        if w.claimed.contains(&id) {
            return Err("Already claimed.");
        }
        if w.prog < WTASKS[w.t].1 {
            return Err("The town hasn't finished it yet.");
        }
        w.claimed.push(id);
        self.w.p.coupons += WEEKLY_COUPONS;
        Ok(WEEKLY_COUPONS)
    }
    /// Trade coupons for kiosk reward `k`. Nothing is deducted unless the reward fits in pockets.
    pub fn kiosk_buy(&mut self, k: usize) -> Result<&'static str, &'static str> {
        let (d, c) = KIOSK_ITEMS[k];
        if self.w.p.coupons < c {
            return Err("Not enough coupons.");
        }
        match d {
            136 => self.w.p.ferry += 1,
            137 => {
                let room =
                    self.w.p.inv.iter().any(|s| s.def == 85 && s.qty + 3 <= STACK) || self.w.p.inv.len() < INV_CAP;
                if !room {
                    return Err("Your pockets are full.");
                }
                for _ in 0..3 {
                    self.add(85);
                }
            }
            _ => {
                if !self.add_furn(d) {
                    return Err("Your pockets are full.");
                }
            }
        }
        self.w.p.coupons -= c;
        Ok(def(d).name)
    }
    pub fn art_open(&self) -> bool {
        weekday(self.w.day) == 5 && !self.fair() && self.fest().is_none()
    }
    /// Place today's surprise (shared; first finder gets it).
    pub fn new_surprise(&mut self) {
        let kind = gen_range(0u8, 3);
        let t = if kind == 0 { self.rand_tile(SAND) } else { self.rand_tile(GRASS) };
        self.w.surprise = t.map(|t| Surprise { kind, x: t.0, z: t.1, owner: gen_range(0, 4), day: self.w.day });
    }
    pub fn take_surprise(&mut self) {
        let Some(sp) = self.w.surprise.clone() else { return };
        match sp.kind {
            0 => {
                let note = BOTTLE_NOTES[(sp.day.rem_euclid(BOTTLE_NOTES.len() as i64)) as usize];
                self.w.p.mail.push(Letter {
                    from: "A message bottle".into(),
                    text: format!("{note} (Tucked inside: {BOTTLE_COINS} coins.)"),
                    coins: BOTTLE_COINS,
                    item: None,
                    read: false,
                    claimed: false,
                });
                self.note("You found a message bottle! The note and coins are waiting in your mailbox.")
            }
            1 => {
                if self.w.p.lost_owner.is_some() {
                    return self.note("You're already carrying someone's lost scarf.");
                }
                if !self.add(150) {
                    return;
                }
                self.w.p.lost_owner = Some(sp.owner);
                let o = self.res(sp.owner).name;
                self.note(format!("A lost scarf with {o}'s name stitched on it. {o} will be glad to get it back!"))
            }
            _ => {
                let d = FURN_IDS[gen_range(0, FURN_IDS.len())];
                if !self.add_furn(d) {
                    return;
                }
                self.note(format!("A gift drifted down on a balloon! Inside: a {}.", def(d).name))
            }
        }
        self.snd.push(Snd::Coin);
        self.w.surprise = None;
    }
}
