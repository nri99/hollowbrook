//! Daily rollover: catching up missed days, refreshing profiles, residents moving, the shared town.
use super::*;

impl Game {
    // ---- daily refresh (idempotent: one per gameplay day) ----
    pub fn catch_up(&mut self, now: NaiveDateTime) -> bool {
        let today = day_id(now);
        if today < self.w.day {
            if !self.rolled {
                self.rolled = true;
                self.note("Your clock went backwards. Daily events pause until it catches up.")
            }
            return false;
        }
        let n = today - self.w.day;
        if n == 0 {
            return false;
        }
        let l = format!("{now} refresh day {} -> {today} ({n}d)", self.w.day);
        eprintln!("[clock] {l}");
        self.log.push(l);
        self.w.day = today;
        self.note(if n > 1 {
            format!("Welcome back! You were away {n} days.")
        } else {
            "A new day begins in Hollowbrook.".into()
        });
        self.refresh(true);
        true
    }
    pub fn refresh(&mut self, letters: bool) {
        let n = self.w.others.len();
        self.refresh_profile(letters);
        for k in 0..n {
            std::mem::swap(&mut self.w.p, &mut self.w.others[k]);
            self.refresh_profile(letters);
            std::mem::swap(&mut self.w.p, &mut self.w.others[k])
        }
        self.refresh_world();
        self.refresh_moves();
        self.birthday_letters();
    }
    pub(super) fn refresh_profile(&mut self, letters: bool) {
        Self::expire_bounty(&mut self.w.p, self.w.day);
        self.refresh_social();
        self.settle_contests();
        self.assign_tasks();
        if weekday(self.w.day) == 0 {
            self.home_letter()
        }
        let mine = self.w.p.id == self.me.id;
        let reqs = [self.w.native, 40, 42, 10, 12, 30];
        for i in 0..4 {
            let s = &mut self.w.p.npcs[i];
            s.talked = false;
            s.gifted = false;
            s.visited = false;
            s.req = if gen_range(0, 5) == 0 {
                Some(86)
            } else if gen_range(0, 2) == 0 {
                Some(reqs[gen_range(0, reqs.len())])
            } else {
                None
            };
            if letters && s.friend >= 10 && gen_range(0, 3) == 0 {
                let m = gen_range(2u32, 8) * 100;
                const NOTES: [&str; 4] = [
                    "Thinking of you! Here's a little something.",
                    "Hmph. Found this. Don't make it weird.",
                    "I dreamed you liked coins. So here are some.",
                    "Training fund! Spend it on snacks!",
                ];
                self.w.p.mail.push(Letter {
                    from: self.res(i).name.into(),
                    text: NOTES[self.res(i).pers].into(),
                    coins: m,
                    item: None,
                    read: false,
                    claimed: false,
                });
                if mine {
                    self.me.msgs.push((format!("You have a letter from {} in your mailbox.", self.res(i).name), self.t))
                }
            }
        }
        while self.w.p.mail.len() > 20 {
            let Some(k) =
                self.w.p.mail.iter().position(|l| l.read && (l.claimed || (l.coins == 0 && l.item.is_none())))
            else {
                break;
            };
            self.w.p.mail.remove(k);
        }
    }
    pub(super) fn mail_all(&mut self, from: &str, text: &str) {
        for p in std::iter::once(&mut self.w.p).chain(self.w.others.iter_mut()) {
            p.mail.push(Letter {
                from: from.into(),
                text: text.into(),
                coins: 0,
                item: None,
                read: false,
                claimed: false,
            })
        }
    }
    /// Moving (provisional): a resident present >= 14 days whose friendship with every player is < 3 and who has no open
    /// request announces a move; two days later a reserve character takes the house unless someone asked them to stay.
    pub(super) fn birthday_letters(&mut self) {
        for k in 0..4 {
            let c = &CHARS[self.w.roster[k]];
            if (self.mo, self.dom) == BDAYS[self.w.roster[k]] {
                self.mail_all(c.name, "It's my birthday today! If you see me around, come say hi!")
            }
        }
    }
    pub fn refresh_moves(&mut self) {
        let day = self.w.day;
        for k in 0..4 {
            if self.w.moving[k] != 0 && day >= self.w.moving[k] {
                let old = self.res(k).name;
                let Some(new) = (0..CHARS.len()).find(|c| !self.w.roster.contains(c)) else { continue };
                self.w.roster[k] = new;
                self.w.arrived[k] = day;
                self.w.moving[k] = 0;
                let nn = CHARS[new].name;
                for p in std::iter::once(&mut self.w.p).chain(self.w.others.iter_mut()) {
                    p.npcs[k] = NpcS { friend: 0, req: None, talked: false, gifted: false, visited: false }
                }
                let n = &mut self.npcs[k];
                n.inside = true;
                n.path.clear();
                self.mail_all(
                    "Hollowbrook Post",
                    &format!("{old} has moved away. {nn} the {} has moved into the house!", CHARS[new].species),
                );
                self.log.push(format!("day {day}: slot {k} {old} -> {nn}"));
            } else if self.w.moving[k] == 0 && day - self.w.arrived[k] >= 14 {
                let maxf =
                    std::iter::once(&self.w.p).chain(self.w.others.iter()).map(|p| p.npcs[k].friend).max().unwrap_or(0);
                let req = std::iter::once(&self.w.p).chain(self.w.others.iter()).any(|p| p.npcs[k].req.is_some());
                if maxf < 3 && !req {
                    self.w.moving[k] = day + 2;
                    let r = self.res(k);
                    self.mail_all(r.name, "I've been thinking... I might move away in a couple of days. Come talk to me if you'd like me to stay.");
                }
            }
        }
    }
    pub fn refresh_world(&mut self) {
        let day = self.w.day;
        self.w.trampled.clear();
        self.w.branches.clear();
        self.w.rocks_hit.clear();
        self.w.geode_hits = 0;
        // one of the town's real rocks (rand_tile only returns walkable tiles, so it can never pick a rock)
        let rocks: Vec<(i32, i32)> = (0..H as i32)
            .flat_map(|z| (0..W as i32).map(move |x| (x, z)))
            .filter(|&(x, z)| self.tile(x, z) == ROCK)
            .collect();
        self.w.geode_rock = (!rocks.is_empty()).then(|| rocks[gen_range(0, rocks.len())]);
        self.refresh_islet();
        self.finish_mproject();
        self.assign_weekly();
        self.grow_garden();
        self.new_surprise();
        let lv = self.shop_tier();
        if lv > self.w.shop_lv {
            self.w.shop_lv = lv;
            self.mail_all("Mr. Tumble", if lv == 1 { "Thanks to all of you, the General Store has been renovated! Look for a new weekly special by the window." } else { "Another renovation! The General Store now stocks an extra item every day." })
        }
        if self.w.camp_day != self.w.day {
            self.w.camp_day = self.w.day;
            self.w.camp_sold.clear()
        }
        if self.w.project.is_none() {
            self.propose()
        } // flattened flowers recover each morning
        for t in &mut self.w.trees {
            if !t.fruit && day - t.shaken >= 3 {
                t.fruit = true
            }
        } // persisted elapsed-time regrowth rule
        for _ in 0..self.pol().map_or(2, |p| p.weeds) {
            if self.w.weeds.len() < 12 {
                if let Some(t) = self.rand_tile(GRASS) {
                    self.w.weeds.push(t)
                }
            }
        }
        while self.w.ground.iter().filter(|g| def(g.def).k == K::Shell).count() < 4 {
            let Some(t) = self.rand_tile(SAND) else { break };
            self.drop_item([40, 42, 40, 42, 41][gen_range(0, 5)], t.0, t.1)
        }
        while self.w.digs.len() < 2 {
            let Some(t) = self.rand_tile(GRASS) else { break };
            self.w.digs.push(t)
        }
        if self.w.stock_day != day {
            let mut pool = FURN_IDS.to_vec();
            let n = 5 + (self.shop_tier() >= 2) as usize;
            self.w.stock = (0..n).map(|_| Stock { def: pool.remove(gen_range(0, pool.len())), sold: false }).collect();
            self.w.stock_day = day
        }
    }
    pub(super) fn rand_tile(&self, kind: u8) -> Option<(i32, i32)> {
        for _ in 0..300 {
            let (x, z) = (gen_range(1, W as i32 - 1), gen_range(1, H as i32 - 1));
            if tile_at(&self.map, x, z) == kind
                && self.walk_tile(x, z)
                && !self.occupied(x, z)
                && (x, z) != tile_of(self.me.pos)
            {
                return Some((x, z));
            }
        }
        None
    }
    pub(super) fn occupied(&self, x: i32, z: i32) -> bool {
        self.w.garden.iter().any(|p| (p.x, p.z) == (x, z))
            || self.item_at(x, z)
            || self.w.weeds.contains(&(x, z))
            || self.w.digs.contains(&(x, z))
            || self.w.holes.contains(&(x, z))
    }
    pub(super) fn item_at(&self, x: i32, z: i32) -> bool {
        self.w.ground.iter().any(|g| tile_of(vec2(g.x, g.z)) == (x, z))
    }
    pub(super) fn drop_item(&mut self, d: u16, x: i32, z: i32) {
        let uid = self.uid();
        self.w.ground.push(GItem {
            uid,
            def: d,
            x: x as f32 + 0.5 + gen_range(-0.15, 0.15),
            z: z as f32 + 0.5 + gen_range(-0.15, 0.15),
        })
    }

    // ---- collision ----
    pub fn tile(&self, x: i32, z: i32) -> u8 {
        tile_at(&self.map, x, z)
    }
    pub fn fair(&self) -> bool {
        fair_active(self.mo, self.dom)
    }
    /// Display name of a building (resident houses follow their current resident).
    pub fn bname(&self, b: usize) -> String {
        match BLDS[b].kind {
            BK::Res(k) => format!("{}'s House", self.res(k).name),
            _ => BLDS[b].name.into(),
        }
    }
    /// Character living in house slot `slot`.
    pub fn res(&self, slot: usize) -> &'static Res {
        &CHARS[self.w.roster[slot]]
    }
    pub fn pol(&self) -> Option<&'static Policy> {
        self.w.policy.and_then(|i| POLICIES.get(i))
    }
    /// Resident location with the active policy applied (wake/sleep shifts; exempt characters unchanged).
    pub fn loc(&self, slot: usize) -> Loc {
        if self.invited_home(slot) {
            return Loc::Home;
        }
        if self.seek_hide_position(slot).is_some() {
            return Loc::Out;
        }
        let r = self.res(slot);
        let (dw, ds) = match self.pol() {
            Some(p) if !p.exempt.contains(&self.w.roster[slot]) => (p.wake, p.sleep),
            _ => (0, 0),
        };
        let (wake, sleep, hr) = ((r.wake as i32 + dw).max(0), (r.sleep as i32 + ds).min(24), self.hr as i32);
        if !(hr >= wake && hr < sleep) {
            Loc::Asleep
        } else if r
            .home_h
            .iter()
            .enumerate()
            .any(|(k, &(a, b))| hr >= a as i32 && hr < b as i32 + if k == 1 { ds } else { 0 })
        {
            Loc::Home
        } else {
            Loc::Out
        }
    }
    pub fn shop_hours(&self) -> (u32, u32) {
        self.pol().map_or((8, 22), |p| p.shop)
    }
    pub fn shop_is_open(&self) -> bool {
        let (a, b) = self.shop_hours();
        (a..b).contains(&self.hr)
    }
    /// Leader enacts policy `i` (fee from the leader's own coins), or repeals with None (free).
    pub fn enact(&mut self, i: Option<usize>) -> Result<(), &'static str> {
        if self.me.id != LEADER {
            return Err("Only the town leader can change policies.");
        }
        let Some(i) = i else {
            self.w.policy = None;
            return Ok(());
        };
        if self.w.policy == Some(i) {
            return Err("That policy is already in effect.");
        }
        let fee = POLICIES[i].fee;
        if self.w.p.money < fee {
            return Err("Not enough coins for the policy fee.");
        }
        self.w.p.money -= fee;
        self.w.policy = Some(i);
        Ok(())
    }
    pub fn walk_tile(&self, x: i32, z: i32) -> bool {
        if (x, z) == DOCK {
            return false;
        }
        if (self.fair() || self.fest().is_some()) && (x, z) == STALL {
            return false;
        }
        if (x, z) == BOARD {
            return false;
        }
        if camp_block((x, z)) {
            return false;
        }
        if self.w.mproject.as_ref().is_some_and(|m| (m.x, m.z) == (x, z)) {
            return false;
        }
        if let Some(b) = self.w.built.iter().find(|b| ptiles(b.kind, b.x, b.z).contains(&(x, z))) {
            return b.kind == PK_BRIDGE;
        }
        if self.w.project.as_ref().is_some_and(|p| p.kind != PK_BRIDGE && (p.x, p.z) == (x, z)) {
            return false;
        }
        matches!(self.tile(x, z), GRASS | PATH | SAND | BRIDGE | FLOWER)
            && !self.w.holes.contains(&(x, z))
            && (x, z) != mailbox_tile(0)
            && (x, z) != mailbox_tile(1)
    }
    pub fn room_furn(&self, b: usize) -> Vec<Furn> {
        match BLDS[b].kind {
            BK::Home(o) => {
                if self.w.p.id == o {
                    self.w.p.room.clone()
                } else {
                    self.w
                        .others
                        .iter()
                        .find(|p| p.id == o)
                        .map(
                            |p| {
                                if p.story == 0 {
                                    p.room.clone()
                                } else {
                                    p.stories.first().cloned().unwrap_or_default()
                                }
                            },
                        )
                        .unwrap_or_default()
                }
            }
            BK::Res(r) => TEMPL[r].iter().map(|&(def, x, z, rot)| Furn { uid: 0, def, x, z, rot, on: true }).collect(),
            _ => vec![],
        }
    }
    /// Story of building `b` the active player is on (only owners use stairs; visitors stay on the main floor).
    pub fn story_in(&self, b: usize) -> u8 {
        match BLDS[b].kind {
            BK::Home(o) if self.w.p.id == o => self.w.p.story,
            _ => 0,
        }
    }
    /// Stairs visible on the active player's story of building `b`.
    pub fn home_stairs(&self, b: usize) -> Vec<((i32, i32), u8)> {
        if let BK::Home(o) = BLDS[b].kind {
            stairs(self.owner(o).home, self.story_in(b))
        } else {
            vec![]
        }
    }
    /// Swap the active player's furniture to another story (no movement).
    pub fn set_story(&mut self, to: u8) {
        let p = &mut self.w.p;
        if p.story == to {
            return;
        }
        p.stories.resize(3, vec![]);
        let cur = p.story as usize;
        p.stories[cur] = std::mem::take(&mut p.room);
        p.room = std::mem::take(&mut p.stories[to as usize]);
        p.story = to
    }
    pub fn take_stairs(&mut self, to: u8) {
        let t = if to == 2 || self.w.p.story == 2 { STAIR_UP } else { STAIR_DOWN };
        self.place_cancel();
        self.set_story(to);
        self.me.ipos = vec2(t.0 as f32 + 0.5, t.1 as f32 + 1.5);
        self.me.face = vec2(0., 1.);
        self.me.cam = self.me.ipos;
        self.snd.push(Snd::Door);
        self.note(match to {
            0 => "Back on the main floor.",
            1 => "Down in the basement.",
            _ => "Up in the attic room.",
        });
    }
    pub fn room_block(&self, b: usize, x: i32, z: i32, skip: u32) -> bool {
        let (w, d) = self.rdims(b);
        if x < 0 || z < 0 || x >= w || z >= d {
            return true;
        }
        if self.home_stairs(b).iter().any(|s| s.0 == (x, z)) {
            return true;
        }
        if BLDS[b].kind == BK::Shop {
            return COUNTER.contains(&(x, z))
                || DISPLAYS.contains(&(x, z))
                || (x, z) == RACK
                || ((x, z) == SPECIAL_DISPLAY && self.shop_tier() >= 1)
                || ((x, z) == EXTRA_DISPLAY && self.w.stock.len() > 5);
        }
        if BLDS[b].kind == BK::Civic {
            return CIVIC_DESK.contains(&(x, z))
                || CIVIC_PLANTS.contains(&(x, z))
                || CIVIC_BENCH.contains(&(x, z))
                || (x, z) == KIOSK;
        }
        if BLDS[b].kind == BK::Museum {
            return MUSEUM_DESK.contains(&(x, z))
                || MUSEUM_TANK.contains(&(x, z))
                || MUSEUM_CASE.contains(&(x, z))
                || MUSEUM_PED.iter().any(|p| p.0 == (x, z));
        }
        self.room_furn(b)
            .iter()
            .filter(|f| !def(f.def).wall && !(skip != 0 && f.uid == skip))
            .any(|f| foot(f.def, f.x, f.z, f.rot).contains(&(x, z)))
    }
    pub fn free(&self, p: Vec2) -> bool {
        [(-PR, -PR), (PR, -PR), (-PR, PR), (PR, PR)].iter().all(|&(dx, dz)| {
            let (x, z) = tile_of(p + vec2(dx, dz));
            match self.me.scene {
                Scene::Town => self.walk_tile(x, z) || self.tile(x, z) == SEA,
                Scene::Room(b) => !self.room_block(b, x, z, 0),
                Scene::Islet => matches!(tile_at_i(&self.imap, x, z), GRASS | SAND | BRIDGE | SEA) && (x, z) != ISTALL,
                Scene::Frontier => f_walk(&self.fmap, x, z),
                Scene::Planet(k) => p_walk(k, x, z),
                Scene::Cockpit(_) => !ck_block(x, z),
                Scene::Cantina => !ct_block(x, z),
            }
        })
    }
    /// Get up from a seat or bed: back to where you sat down from, or the nearest open spot if that is blocked
    /// (so you can never end up standing inside the furniture).
    pub fn stand_up(&mut self) {
        if self.me.pose == Pose::Stand {
            return;
        }
        self.me.pose = Pose::Stand;
        let seat = self.ppos();
        let mut at = self.me.stand_from;
        if !self.free(at) || at.distance(seat) > 3. {
            at = seat;
            'search: for ring in 1..=8 {
                let r = ring as f32 * 0.35;
                for k in 0..16 {
                    let a = k as f32 / 16. * std::f32::consts::TAU;
                    let p = seat + vec2(a.sin(), a.cos()) * r; // starts straight down (towards the camera)
                    if self.free(p) {
                        at = p;
                        break 'search;
                    }
                }
            }
        }
        self.set_ppos(at);
    }
    pub fn step_move(&mut self, d: Vec2, run: bool, dt: f32) {
        self.me.spd = 0.;
        if d == Vec2::ZERO {
            return;
        }
        if self.me.pose != Pose::Stand {
            self.stand_up();
            return;
        }
        if self.me.fish.is_some() {
            self.me.fish = None;
            self.note("You reel in your line.")
        }
        let d = d.normalize();
        self.me.face = d;
        let s = (if self.swimming() {
            2.0
        } else if run {
            5.5
        } else {
            3.0
        }) * dt
            * if self.me.riding && self.me.scene == Scene::Frontier { 1.9 } else { 1. }
            * if self.me.jet > 0. && matches!(self.me.scene, Scene::Frontier | Scene::Planet(_)) { 2.4 } else { 1. };
        let p0 = self.ppos();
        let mut p = p0;
        let nx = p + vec2(d.x * s, 0.);
        if self.free(nx) {
            p = nx
        }
        let nz = p + vec2(0., d.y * s);
        if self.free(nz) {
            p = nz
        } else if let Scene::Room(b) = self.me.scene {
            let (w, dd) = self.rdims(b);
            if d.y > 0. && self.story_in(b) == 0 && p.x.floor() as i32 == w / 2 && p.y > dd as f32 - 0.5 {
                self.set_ppos(p);
                self.go(Scene::Town);
                return;
            }
        }
        self.set_ppos(p);
        let moved = (p - p0).length();
        self.me.spd = moved / dt.max(1e-4);
        self.me.step_acc += moved;
        if self.me.step_acc > if run { 0.9 } else { 0.6 } {
            self.me.step_acc = 0.;
            let st = self.step_sound();
            self.snd.push(st)
        }
        if run && self.me.scene == Scene::Town {
            self.bugs.retain(|b| b.1.distance(p) > 2.5);
            let t = tile_of(p);
            if self.tile(t.0, t.1) == FLOWER && !self.w.trampled.contains(&t) && gen_range(0, 25) == 0 {
                self.w.trampled.push(t);
                self.note("Oops, you trampled a flower. It'll perk up by morning.")
            }
        }
    }
    /// Footstep sound for the surface under the active player.
    pub fn step_sound(&self) -> Snd {
        let t = tile_of(self.ppos());
        let c = match self.me.scene {
            Scene::Town => self.tile(t.0, t.1),
            Scene::Islet => tile_at_i(&self.imap, t.0, t.1),
            Scene::Frontier => ftile(&self.fmap, t.0, t.1),
            Scene::Planet(k) => match ptile(k, t.0, t.1) {
                SAND => SAND,
                c => c,
            },
            Scene::Room(_) | Scene::Cockpit(_) | Scene::Cantina => return Snd::StepFloor,
        };
        if self.w.built.iter().any(|b| b.kind == PK_BRIDGE && ptiles(b.kind, b.x, b.z).contains(&t)) {
            return Snd::StepWood;
        }
        match c {
            PATH => Snd::StepPath,
            SAND => Snd::StepSand,
            BRIDGE => Snd::StepWood,
            SEA | RIVER => Snd::Swim,
            _ => Snd::Step,
        }
    }
    pub fn go(&mut self, to: Scene) {
        if self.me.trans.is_none() {
            self.me.trans = Some((0., to, false));
            self.snd.push(Snd::Door)
        }
    }
    pub fn swap(&mut self, to: Scene) {
        self.place_cancel();
        self.set_story(0);
        if to != Scene::Islet {
            let id = self.me.id;
            if let Some(a) = &mut self.act {
                if !a.done {
                    a.players.retain(|&p| p != id)
                }
            }
        } // leaving forfeits
        match to {
            Scene::Cockpit(_) => {
                self.me.ipos = vec2(CK_EXIT.0, CK_EXIT.1 - 0.7);
                self.me.face = vec2(0., -1.);
                self.me.cam = self.me.ipos
            }
            Scene::Planet(k) => {
                let r = ramp_of(k, pad_of(self.me.id));
                self.me.ipos = r + vec2(0., 0.6);
                self.me.face = vec2(0., 1.);
                self.me.cam = self.me.ipos
            }
            Scene::Cantina => {
                self.me.ipos = vec2(3.0, 3.8);
                self.me.face = vec2(0., -1.);
                self.me.cam = self.me.ipos
            }
            Scene::Frontier => {
                // off the ship's ramp when stepping out of it, outside the cantina, or at the gate
                let from_ship = matches!(self.me.scene, Scene::Cockpit(_));
                let from_cantina = self.me.scene == Scene::Cantina;
                if self.me.scene == Scene::Town {
                    self.me.ship_at = 0; // your ship flies itself home to its Dustreach pad
                }
                self.me.ipos = if from_ship {
                    ramp_of(0, pad_of(self.me.id)) + vec2(0., 0.6)
                } else if from_cantina {
                    vec2(F_CANTINA.0, F_CANTINA.1 + 0.7)
                } else {
                    vec2(F_SPAWN.0, F_SPAWN.1)
                };
                self.me.face = if from_ship || from_cantina { vec2(0., 1.) } else { vec2(0., -1.) };
                self.me.cam = self.me.ipos
            }
            Scene::Islet => {
                self.me.ipos = vec2(ISLET_SPAWN.0, ISLET_SPAWN.1);
                self.me.face = vec2(0., -1.);
                self.me.cam = self.me.ipos
            }
            Scene::Town => {
                if let Scene::Room(b) = self.me.scene {
                    self.me.pos = door_stand(b)
                }
                if self.me.scene == Scene::Islet {
                    self.me.pos = dock_stand()
                }
                self.me.face = vec2(0., 1.);
                self.me.cam = self.me.pos
            }
            Scene::Room(b) => {
                let (w, d) = self.rdims(b);
                self.me.ipos = vec2((w / 2) as f32 + 0.5, d as f32 - 0.5);
                self.me.face = vec2(0., -1.);
                self.me.cam = self.me.ipos
            }
        }
        eprintln!("[scene] {:?} -> {:?}", self.me.scene, to);
        self.me.scene = to;
        self.visit_arrival();
        self.me.pose = Pose::Stand;
        self.me.fish = None;
    }
}
