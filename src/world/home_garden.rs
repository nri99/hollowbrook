//! Home evaluation and repainting, gardening, tool upgrades, the keeper menu.
use super::*;

impl Game {
    // ---- Home evaluation ----
    /// Score of a profile's home with up to two improvement tips.
    pub fn home_score(p: &Profile) -> (u32, Vec<&'static str>) {
        let all: Vec<Furn> = p.room.iter().chain(p.stories.iter().flatten()).copied().collect();
        let room = &all;
        let n = room.len().min(30) as u32;
        let mut ids: Vec<u16> = room.iter().map(|f| f.def).collect();
        ids.sort();
        ids.dedup();
        let sets = SETS.iter().filter(|s| s.1.iter().all(|d| ids.contains(d))).count() as u32;
        let lamp = room.iter().any(|f| def(f.def).act == Act::Lamp);
        let walls = room.iter().filter(|f| def(f.def).wall).count().min(3) as u32;
        let decor = p.wall != 0 || p.floor != 0;
        let keeps = room.iter().filter(|f| KEEPSAKES.contains(&f.def)).count() as u32;
        let score = n * 10
            + ids.len() as u32 * 5
            + sets * 60
            + if lamp { 15 } else { 0 }
            + walls * 10
            + if decor { 20 } else { 0 }
            + p.home as u32 * 40
            + keeps * 15;
        let mut tips = vec![];
        if !lamp {
            tips.push("A lamp would make the room glow in the evening.")
        }
        if walls == 0 {
            tips.push("The walls look bare. Try hanging a painting or a portrait.")
        }
        if !decor {
            tips.push("New wallpaper or flooring would give the room character.")
        }
        if sets == 0 {
            tips.push("Completing a furniture collection impresses every judge.")
        }
        if ids.len() < 6 {
            tips.push("A little more variety in your furniture would help.")
        }
        if p.home == 0 {
            tips.push("A bigger home leaves room for more ideas.")
        }
        tips.truncate(2);
        (score, tips)
    }
    pub fn home_tier(score: u32) -> usize {
        HOME_TIERS.iter().rposition(|t| score >= t.0).unwrap_or(0)
    }
    /// Monday letter from the Home Society with the score, tips and a one-time reward for each newly reached rating.
    pub(super) fn home_letter(&mut self) {
        let (sc, tips) = Self::home_score(&self.w.p);
        let t = Self::home_tier(sc);
        let mut text = format!("This week your home scored {sc} points: {}! ", HOME_TIERS[t].1);
        for tip in &tips {
            text.push_str(tip);
            text.push(' ')
        }
        let (mut coins, mut item) = (0, None);
        if t as u8 > self.w.p.home_best {
            for k in (self.w.p.home_best as usize + 1)..=t {
                coins += HOME_TIERS[k].2
            }
            if t == HOME_TIERS.len() - 1 {
                item = Some(169)
            }
            self.w.p.home_best = t as u8;
            text.push_str(&format!("A new rating! Please accept {coins} coins from the Society."))
        }
        self.w.p.mail.push(Letter {
            from: "Hollowbrook Home Society".into(),
            text,
            coins,
            item,
            read: false,
            claimed: false,
        });
    }
    pub fn repaint(&mut self, k: usize) -> Result<(), &'static str> {
        if self.w.p.money < PAINT_FEE {
            return Err("Not enough coins for the paint.");
        }
        if self.w.p.paint == Some(k as u8) {
            return Err("Your house is already that colour.");
        }
        self.w.p.money -= PAINT_FEE;
        self.w.p.paint = Some(k as u8);
        Ok(())
    }
    // ---- Gardening ----
    pub fn plant_ok(&self, t: (i32, i32)) -> bool {
        self.me.scene == Scene::Town
            && tile_at(&self.map, t.0, t.1) == GRASS
            && self.walk_tile(t.0, t.1)
            && !self.occupied(t.0, t.1)
            && t != tile_of(self.me.pos)
            && ![mailbox_tile(0), mailbox_tile(1)].contains(&t)
    }
    pub fn plant(&mut self, t: (i32, i32)) -> Result<(), &'static str> {
        if !self.plant_ok(t) {
            return Err("You can't plant there. Try open grass.");
        }
        let Some(k) = SEEDS.iter().position(|&s| self.w.p.inv.iter().any(|x| x.def == s)) else {
            return Err("You have no seeds.");
        };
        self.remove_one(SEEDS[k]);
        self.w.garden.push(Plant { x: t.0, z: t.1, kind: k as u8, stage: 0, watered: i64::MIN });
        self.snd.push(Snd::Dig);
        self.note(format!("You planted {}. Water it every day!", def(SEEDS[k]).name));
        Ok(())
    }
    /// Water plants in front (area grows with the can's level). Returns how many were watered.
    pub fn water(&mut self) -> usize {
        let lv = self.tool_level(CAN);
        let f = tile_of(self.me.pos + self.me.face * 0.9);
        let (fx, fz) = (self.me.face.x.round() as i32, self.me.face.y.round() as i32);
        let mut tiles = vec![f];
        if lv >= 1 {
            tiles.push((f.0 + fz, f.1 + fx));
            tiles.push((f.0 - fz, f.1 - fx));
        }
        if lv >= 2 {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    tiles.push((f.0 + dx, f.1 + dz))
                }
            }
        }
        let day = self.w.day;
        let mut n = 0;
        for p in &mut self.w.garden {
            if tiles.contains(&(p.x, p.z)) && p.watered != day {
                p.watered = day;
                n += 1
            }
        }
        let fp = tc(f);
        self.fx.push((vec3(fp.x, 0.05, fp.y), 0., 0));
        self.snd.push(Snd::Splash);
        if n > 0 {
            self.ev(Ev::Water, n as u32);
            self.note(format!("Watered {n} flower{}.", if n == 1 { "" } else { "s" }))
        } else {
            self.note("Nothing here needs water.")
        }
        n
    }
    pub fn pick_flower(&mut self, i: usize) {
        let p = self.w.garden[i];
        if p.stage < 2 {
            return self.note(if p.watered == self.w.day {
                "It's growing. Come back tomorrow!"
            } else {
                "It needs water today to keep growing."
            });
        }
        if self.add(FLOWERS[p.kind as usize]) {
            self.w.garden[i].stage = 1;
            self.note(format!("You picked a {}. It will bloom again soon.", def(FLOWERS[p.kind as usize]).name))
        }
    }
    /// Daily growth (watered yesterday or rain) and cross-breeding of neighbouring blooms of different colours.
    pub fn grow_garden(&mut self) {
        let prev = self.w.day - 1;
        let rain = rainy(prev);
        for p in &mut self.w.garden {
            if p.watered >= prev || rain {
                p.stage = (p.stage + 1).min(2)
            }
        }
        let blooms: Vec<Plant> = self.w.garden.iter().copied().filter(|p| p.stage == 2 && p.kind < 4).collect();
        'outer: for a in &blooms {
            for b in &blooms {
                if (a.x - b.x).abs() + (a.z - b.z).abs() == 1 && a.kind < b.kind && gen_range(0, CROSS_CHANCE) == 0 {
                    for (dx, dz) in [(0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (-1, -1)] {
                        let t = (a.x + dx, a.z + dz);
                        if tile_at(&self.map, t.0, t.1) == GRASS && self.walk_tile(t.0, t.1) && !self.occupied(t.0, t.1)
                        {
                            self.w.garden.push(Plant { x: t.0, z: t.1, kind: 4, stage: 0, watered: i64::MIN });
                            self.mail_all("Town notice board", "A strange new sprout appeared in the town garden! Some say crossed flowers make something rare.");
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    // ---- Tool upgrades ----
    pub fn tool_level(&self, t: u16) -> u8 {
        UPGRADABLE.iter().position(|&u| u == t).and_then(|k| self.w.p.tool_lv.get(k).copied()).unwrap_or(0)
    }
    pub fn upgrade_tool(&mut self, t: u16) -> Result<u8, String> {
        let Some(k) = UPGRADABLE.iter().position(|&u| u == t) else {
            return Err("That tool can't be upgraded.".into());
        };
        if !self.w.p.tools.contains(&t) {
            return Err(format!("You need a {} first.", def(t).name));
        }
        let lv = self.tool_level(t);
        if lv >= 2 {
            return Err("That's already a masterwork!".into());
        }
        let (c, st, gl) = UPGRADE[lv as usize];
        let have = |g: &Game, d: u16| g.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum::<u16>();
        if self.w.p.money < c || have(self, 81) < st || have(self, 85) < gl {
            return Err(format!(
                "I need {c} coins, {st} Stone{} to do that.",
                if gl > 0 { format!(" and {gl} Sea Glass") } else { String::new() }
            ));
        }
        self.w.p.money -= c;
        for _ in 0..st {
            self.remove_one(81);
        }
        for _ in 0..gl {
            self.remove_one(85);
        }
        while self.w.p.tool_lv.len() < UPGRADABLE.len() {
            self.w.p.tool_lv.push(0)
        }
        self.w.p.tool_lv[k] = lv + 1;
        Ok(lv + 1)
    }
    pub fn keeper_items(&self) -> Vec<(String, u8)> {
        // (label, action) 0 chat, 1-4 seed, 5 can, 10+k upgrade tool k
        let mut v = vec![("Chat".to_string(), 0)];
        for (k, &s) in SEEDS.iter().enumerate() {
            v.push((format!("{}  -  {SEED_PRICE} coins", def(s).name), 1 + k as u8))
        }
        if !self.w.p.tools.contains(&CAN) {
            v.push((format!("Watering Can  -  {} coins", def(CAN).buy), 5))
        }
        for (k, &t) in UPGRADABLE.iter().enumerate() {
            if self.w.p.tools.contains(&t) {
                let lv = self.tool_level(t);
                if lv < 2 {
                    let (c, st, gl) = UPGRADE[lv as usize];
                    v.push((
                        format!(
                            "Upgrade {} -> {}{}  -  {c}c, {st} Stone{}",
                            def(t).name,
                            LV_NAME[lv as usize + 1],
                            def(t).name,
                            if gl > 0 { format!(", {gl} Sea Glass") } else { String::new() }
                        ),
                        10 + k as u8,
                    ))
                }
            }
        }
        v
    }
}
