//! Furniture: using, placing, moving, picking up, home storage transfer.
use super::*;

impl Game {
    // ---- furniture ----
    pub fn use_furn(&mut self, uid: u32, d: u16) {
        let df = def(d);
        if uid == 0 {
            return self.note(format!("A cozy {}. Better not touch someone else's things.", df.name.to_lowercase()));
        }
        let Some(idx) = self.w.p.room.iter().position(|f| f.uid == uid) else { return };
        let f = self.w.p.room[idx];
        match df.act {
            Act::Sit => {
                self.me.stand_from = self.me.ipos;
                let ip = self.me.ipos;
                self.me.ipos = foot(f.def, f.x, f.z, f.rot)
                    .into_iter()
                    .map(tc)
                    .min_by(|a, b| a.distance(ip).total_cmp(&b.distance(ip)))
                    .unwrap();
                self.me.face = dir(f.rot);
                self.me.pose = Pose::Sit(uid);
                self.note("You sit down. Move or press E to stand up.")
            }
            Act::Bed => {
                self.me.stand_from = self.me.ipos;
                self.me.ipos = fcenter(&f);
                self.me.face = dir(f.rot);
                self.me.pose = Pose::Lie(uid);
                self.note("You lie down for a rest. Time keeps flowing as usual.")
            }
            Act::Lamp => {
                self.w.p.room[idx].on = !f.on;
                self.note(if f.on { "Lamp off." } else { "Lamp on." })
            }
            Act::Storage => self.me.ui = Ui::Storage(0, 0),
            Act::None => self.note(format!("Your {} looks great here.", df.name.to_lowercase())),
        }
    }
    pub fn start_place(&mut self, uid: u32) {
        let Some(s) = self.w.p.inv.iter().find(|s| s.uid == uid).copied() else { return };
        let t = tile_of(self.me.ipos + self.me.face);
        let wall = def(s.def).wall;
        self.me.placing = Some(Placing {
            f: Furn { uid, def: s.def, x: t.0, z: if wall { -1 } else { t.1 }, rot: 0, on: false },
            orig: None,
            slot_uid: Some(uid),
        });
    }
    pub fn start_move(&mut self, uid: u32) {
        if let Some(i) = self.w.p.room.iter().position(|f| f.uid == uid) {
            let f = self.w.p.room.remove(i);
            self.me.placing = Some(Placing { f, orig: Some(f), slot_uid: None })
        }
    }
    pub fn place_cancel(&mut self) {
        if let Some(p) = self.me.placing.take() {
            if let Some(o) = p.orig {
                self.w.p.room.push(o)
            }
        }
    }
    pub fn place_ok(&self, f: &Furn) -> Result<(), &'static str> {
        let (w, d) = self.rdims(self.home());
        let df = def(f.def);
        if df.wall {
            if f.z != -1 || f.x < 0 || f.x >= w {
                return Err("Wall items go on the back wall.");
            }
            if self.w.p.room.iter().any(|o| def(o.def).wall && o.x == f.x && o.uid != f.uid) {
                return Err("Something already hangs there.");
            }
            if self.home_stairs(self.home()).iter().any(|s| elev(s.1) > elev(self.w.p.story) && s.0 .0 == f.x) {
                return Err("The stairs are in the way.");
            }
            return Ok(());
        }
        let fp = foot(f.def, f.x, f.z, f.rot);
        if fp.iter().any(|&(x, z)| x < 0 || z < 0 || x >= w || z >= d) {
            return Err("It doesn't fit there.");
        }
        if self
            .home_stairs(self.home())
            .iter()
            .any(|s| fp.iter().any(|&(x, z)| x == s.0 .0 && (z == s.0 .1 || z == s.0 .1 + 1)))
        {
            return Err("Keep the stairs clear.");
        }
        if fp.contains(&(w / 2, d - 1)) {
            return Err("Keep the doorway clear.");
        }
        if fp.iter().any(|&t| self.room_block(self.home(), t.0, t.1, f.uid)) {
            return Err("Something is in the way.");
        }
        let me: Vec<(i32, i32)> = [(-PR, -PR), (PR, -PR), (-PR, PR), (PR, PR)]
            .iter()
            .map(|&(a, b)| tile_of(self.me.ipos + vec2(a, b)))
            .collect();
        if fp.iter().any(|t| me.contains(t)) {
            return Err("You're standing there.");
        }
        if !self.reach(tile_of(self.me.ipos), (w / 2, d - 1), &fp) {
            return Err("That would block your way out.");
        }
        Ok(())
    }
    pub(super) fn reach(&self, a: (i32, i32), b: (i32, i32), extra: &[(i32, i32)]) -> bool {
        let mut seen = HashSet::from([a]);
        let mut q = VecDeque::from([a]);
        while let Some(c) = q.pop_front() {
            if c == b {
                return true;
            }
            for d in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (c.0 + d.0, c.1 + d.1);
                if !seen.contains(&n) && !self.room_block(self.home(), n.0, n.1, 0) && !extra.contains(&n) {
                    seen.insert(n);
                    q.push_back(n)
                }
            }
        }
        false
    }
    pub fn place_confirm(&mut self) -> Result<(), &'static str> {
        let (f, su) = {
            let p = self.me.placing.as_ref().ok_or("Nothing to place.")?;
            (p.f, p.slot_uid)
        };
        self.place_ok(&f)?;
        if let Some(uid) = su {
            let i = self.w.p.inv.iter().position(|s| s.uid == uid).ok_or("Item missing.")?;
            self.w.p.inv.remove(i);
        }
        let before = self.sets_done();
        self.w.p.room.push(f);
        self.me.placing = None;
        for n in self.sets_done() {
            if !before.contains(&n) {
                self.note(format!("{n} complete!"));
                self.snd.push(Snd::Coin)
            }
        }
        Ok(())
    }
    pub fn pick_furn(&mut self, uid: u32) {
        if self.w.p.inv.len() >= INV_CAP {
            return self.note("Your pockets are full!");
        }
        if let Some(i) = self.w.p.room.iter().position(|f| f.uid == uid) {
            let f = self.w.p.room.remove(i);
            self.w.p.inv.push(Slot { size_mm: 0, def: f.def, qty: 1, uid });
            self.note(format!("Picked up the {}.", def(f.def).name))
        }
    }
    pub fn transfer(&mut self, col: usize, s: usize) {
        let w = &mut self.w;
        let (src, dst, cap) = if col == 0 {
            (&mut w.p.inv, &mut w.p.storage, STORE_CAP)
        } else {
            (&mut w.p.storage, &mut w.p.inv, INV_CAP)
        };
        if s >= src.len() {
            return;
        }
        if dst.len() >= cap {
            return self.note("No room there.");
        }
        let it = src.remove(s);
        dst.push(it);
    }
}
