//! Gullrock islet: map, timed activities, daily refresh, Pell and the stall.
use super::*;

impl Game {
    // ---- Gullrock timed activities ----
    pub fn iland(&self, p: Vec2) -> bool {
        let t = tile_of(p);
        matches!(tile_at_i(&self.imap, t.0, t.1), GRASS | SAND) && t != ISTALL && t != ACT_BOARD
    }
    pub(super) fn iland_rand(&self) -> Vec2 {
        let pen = vec2(PEN.0, PEN.1);
        for _ in 0..300 {
            let p = vec2(gen_range(1, IW as i32 - 1) as f32 + 0.5, gen_range(1, IH as i32 - 1) as f32 + 0.5);
            if self.iland(p) && p.distance(pen) > PEN.2 + 0.8 {
                return p;
            }
        }
        vec2(7.5, 6.5)
    }
    pub fn in_pen(p: Vec2) -> bool {
        p.distance(vec2(PEN.0, PEN.1)) < PEN.2
    }
    pub fn act_tier(kind: usize, score: u32) -> usize {
        ACTS[kind].3.iter().filter(|&&t| score >= t).count()
    }
    /// Start activity `kind` for every player currently on Gullrock (a player in town is unaffected).
    pub fn act_start(&mut self, kind: usize) -> Result<(), &'static str> {
        if self.me.scene != Scene::Islet {
            return Err("Activities start at Gullrock.");
        }
        if self.act.is_some() {
            return Err("An activity is already running.");
        }
        let mut players = vec![self.me.id];
        players.extend(self.others.iter().filter(|o| o.scene == Scene::Islet).map(|o| o.id));
        let ents = match kind {
            0 => (0..6).map(|_| self.iland_rand()).collect(),
            1 => vec![self.iland_rand()],
            _ => rings(),
        };
        self.act = Some(Activity {
            kind,
            left: ACTS[kind].2,
            score: 0,
            mine: vec![],
            players,
            ents,
            next: 0,
            life: BEACON_LIFE,
            done: false,
            paid: vec![],
        });
        Ok(())
    }
    /// Shared timer and world behaviour (crabs wander, beacons move). Runs once per frame.
    pub fn sim_act(&mut self, dt: f32) {
        let Some(mut a) = self.act.take() else { return };
        if a.done {
            a.life -= dt;
            if a.life > 0. {
                self.act = Some(a)
            }
            return;
        }
        if a.players.is_empty() {
            return;
        } // everyone left: cancelled, no reward
        a.left -= dt;
        match a.kind {
            0 => {
                for k in 0..a.ents.len() {
                    let c = a.ents[k];
                    let an = (self.t * 0.7 + k as f32 * 2.1).sin() * 6.3 + k as f32;
                    let n = c + vec2(an.cos(), an.sin()) * dt * 0.5;
                    if self.iland(n) && Self::in_pen(n) == Self::in_pen(c) {
                        a.ents[k] = n
                    }
                }
                a.score = a.ents.iter().filter(|&&c| Self::in_pen(c)).count() as u32;
                if a.score as usize == a.ents.len() {
                    a.done = true
                }
            }
            1 => {
                a.life -= dt;
                if a.life <= 0. {
                    a.ents[0] = self.iland_rand();
                    a.life = BEACON_LIFE
                }
            }
            _ => {
                if a.next >= a.ents.len() {
                    a.done = true
                }
            }
        }
        if a.left <= 0. {
            a.left = 0.;
            a.done = true
        }
        if a.done {
            a.life = 6.
        }
        self.act = Some(a);
    }
    /// Per-player part: the active player's touches/pushes, and their own reward once it ends.
    pub fn act_step(&mut self) {
        let id = self.me.id;
        let (p, face, swim) = (self.me.ipos, self.me.face, self.swimming());
        let Some(mut a) = self.act.take() else { return };
        if !a.done && self.me.scene == Scene::Islet && !a.players.contains(&id) {
            a.players.push(id);
            self.note(format!("You joined {} in progress!", ACTS[a.kind].0))
        }
        if !a.done && self.me.scene == Scene::Islet && a.players.contains(&id) {
            let mut gain = 0;
            match a.kind {
                0 => {
                    for k in 0..a.ents.len() {
                        let d = a.ents[k] - p;
                        if d.length() < 0.65 {
                            let n = p + d.try_normalize().unwrap_or(face) * 0.65;
                            if self.iland(n) {
                                let was = Self::in_pen(a.ents[k]);
                                a.ents[k] = n;
                                if !was && Self::in_pen(n) {
                                    gain += 1
                                }
                            }
                        }
                    }
                }
                1 => {
                    if a.ents[0].distance(p) < 0.7 {
                        gain = 1;
                        a.score += 1;
                        a.ents[0] = self.iland_rand();
                        a.life = BEACON_LIFE
                    }
                }
                _ => {
                    if swim && a.next < a.ents.len() && a.ents[a.next].distance(p) < 0.9 {
                        gain = 1;
                        a.score += 1;
                        a.next += 1
                    }
                }
            }
            if gain > 0 {
                self.snd.push(Snd::Blip);
                if let Some(m) = a.mine.iter_mut().find(|m| m.0 == id) {
                    m.1 += gain
                } else {
                    a.mine.push((id, gain))
                }
            }
        }
        if a.done && a.players.contains(&id) && !a.paid.contains(&id) {
            a.paid.push(id);
            let day = self.w.day;
            if self.w.p.task_day == day {
                for t in &mut self.w.p.tasks {
                    if TASKS[t.t].0 == Ev::Activity {
                        t.prog = TASKS[t.t].1
                    }
                }
            }
            let tier = Self::act_tier(a.kind, a.score);
            let name = ACTS[a.kind].0;
            if tier == 0 {
                self.note(format!("{name} over: score {}. No reward this time.", a.score))
            } else if self.w.p.act_day[a.kind] == self.w.day {
                self.note(format!("{name} over: score {}. (Already rewarded for this one today.)", a.score))
            } else {
                let n = ACT_REWARD[tier - 1];
                let mut got = 0;
                for _ in 0..n {
                    if self.add(85) {
                        got += 1
                    }
                }
                for _ in got..n {
                    self.w.p.mail.push(Letter {
                        from: "Captain Pell".into(),
                        text: "Your pockets were full, so here is the Sea Glass you earned.".into(),
                        coins: 0,
                        item: Some(85),
                        read: false,
                        claimed: false,
                    })
                }
                if got < n {
                    self.note(format!("Pockets full: {} Sea Glass sent to your mailbox.", n - got))
                }
                self.w.p.act_day[a.kind] = self.w.day;
                self.snd.push(Snd::Coin);
                self.note(format!(
                    "{name} over: score {} ({}). +{got} Sea Glass!",
                    a.score,
                    ["", "bronze", "silver", "gold"][tier]
                ))
            }
        }
        self.act = Some(a);
    }
    // ---- Gullrock Islet ----
    pub fn refresh_islet(&mut self) {
        self.w.iground.clear();
        let mut free: Vec<(i32, i32)> = (0..IW * IH)
            .filter(|&i| self.imap[i] == SAND)
            .map(|i| ((i % IW) as i32, (i / IW) as i32))
            .filter(|&t| t != ISTALL && t != (7, 11) && t != (6, 11) && t != (7, 10))
            .collect();
        let foreign: Vec<u16> = FRUITS.iter().copied().filter(|&f| f != self.w.native).collect();
        let ff = foreign[self.w.day.rem_euclid(foreign.len() as i64) as usize]; // one non-native fruit washes ashore daily
        for (d, n) in [(82, 7), (84, 4), (75, 2), (ff, 2)] {
            for _ in 0..n {
                if free.is_empty() {
                    break;
                }
                let t = free.remove(gen_range(0, free.len()));
                let uid = self.uid();
                self.w.iground.push(GItem { uid, def: d, x: t.0 as f32 + 0.5, z: t.1 as f32 + 0.5 })
            }
        }
    }
    /// Captain Pell: offers the return trip, explains today's activity, and pays 3 Sea Glass once per player per day.
    pub fn pell(&mut self) {
        let (d, n, label) = islet_task(self.w.day);
        let have: u16 = self.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum();
        let mut lines = vec![format!("Ahoy! Today's islet task: {label}.")];
        if self.w.p.islet_done == self.w.day {
            lines.push("You've already done today's task. Come back tomorrow!".into())
        } else if have >= n {
            for _ in 0..n {
                self.remove_one(d);
            }
            let mut got = 0;
            for _ in 0..3 {
                if self.add(85) {
                    got += 1
                }
            }
            for _ in got..3 {
                self.w.p.mail.push(Letter {
                    from: "Captain Pell".into(),
                    text: "Your pockets were full, so here is the Sea Glass you earned.".into(),
                    coins: 0,
                    item: Some(85),
                    read: false,
                    claimed: false,
                })
            }
            self.w.p.islet_done = self.w.day;
            self.snd.push(Snd::Coin);
            lines.push("Splendid work! Here are 3 pieces of Sea Glass.".into())
        } else {
            lines.push(format!("You have {have} of {n} so far."))
        }
        lines.push("Talk to me again after this to sail home (free).".into());
        self.me.ui = Ui::Talk { npc: None, who: "Captain Pell the pelican".into(), lines, i: 0 };
        self.pell_talked = true;
    }
    pub fn istall_buy(&mut self, k: usize) -> Result<(), &'static str> {
        let (d, cost) = ISTALL_ITEMS[k];
        let have: u16 = self.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum();
        if have < cost {
            return Err("Not enough Sea Glass.");
        }
        let frees_slot = self.w.p.inv.iter().filter(|s| s.def == 85).count() == 1 && have == cost;
        if self.w.p.inv.len() >= INV_CAP && !frees_slot {
            return Err("Your pockets are full.");
        }
        for _ in 0..cost {
            self.remove_one(85);
        }
        self.add_furn(d);
        Ok(())
    }

    // ---- community projects ----
    pub fn remaining(p: &Project) -> String {
        let r: Vec<String> =
            p.need.iter().filter(|n| n.2 < n.1).map(|n| format!("{} {}", n.1 - n.2, MAT_NAMES[n.0 as usize])).collect();
        if r.is_empty() {
            "It's ready!".into()
        } else {
            format!("Still needed: {}.", r.join(", "))
        }
    }
}
