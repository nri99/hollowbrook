//! Pockets and storage, catalog orders, home balance and expansion, letters, gifts.
use super::*;

impl Game {
    pub fn add(&mut self, d: u16) -> bool {
        if let Some(s) = self.w.p.inv.iter_mut().find(|s| s.def == d && s.uid == 0 && s.size_mm == 0 && s.qty < STACK) {
            s.qty += 1
        } else if self.w.p.inv.len() < INV_CAP {
            self.w.p.inv.push(Slot { size_mm: 0, def: d, qty: 1, uid: 0 })
        } else {
            self.note("Your pockets are full!");
            return false;
        }
        if !self.w.p.seen.contains(&d) {
            self.w.p.seen.push(d)
        }
        self.note(format!("Got: {}!", def(d).name));
        true
    }
    pub fn add_furn(&mut self, d: u16) -> bool {
        if self.w.p.inv.len() >= INV_CAP {
            self.note("Your pockets are full!");
            return false;
        }
        let uid = self.uid();
        self.w.p.inv.push(Slot { size_mm: 0, def: d, qty: 1, uid });
        if !self.w.p.catalog.contains(&d) {
            self.w.p.catalog.push(d)
        }
        true
    }
    /// Re-order a catalog item at full shop price (delivered to pockets immediately; provisional).
    pub fn order(&mut self, d: u16) -> Result<(), &'static str> {
        if !self.w.p.catalog.contains(&d) {
            return Err("That isn't in your catalog.");
        }
        let p = def(d).buy;
        if self.w.p.money < p {
            return Err("Not enough coins.");
        }
        if self.w.p.inv.len() >= INV_CAP {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= p;
        self.add_furn(d);
        self.w.spent += p;
        Ok(())
    }
    pub fn remove_one(&mut self, d: u16) -> bool {
        let Some(i) = self.w.p.inv.iter().position(|s| s.def == d && s.uid == 0) else { return false };
        self.w.p.inv[i].qty -= 1;
        if self.w.p.inv[i].qty == 0 {
            self.w.p.inv.remove(i);
        }
        true
    }

    /// Pay as much of the expansion balance as current coins allow.
    pub fn pay_debt(&mut self) {
        let p = self.w.p.money.min(self.w.p.debt);
        self.w.p.money -= p;
        self.w.p.debt -= p;
        self.snd.push(Snd::Coin);
        self.note(if self.w.p.debt == 0 {
            format!("Paid {p} coins. Your balance is cleared!")
        } else {
            format!("Paid {p} coins. {} left.", self.w.p.debt)
        })
    }
    /// Interior size of a building; homes grow with their owner's house level.
    pub fn rdims(&self, b: usize) -> (i32, i32) {
        if let BK::Home(o) = BLDS[b].kind {
            let lv = if self.w.p.id == o {
                self.w.p.home
            } else {
                self.w.others.iter().find(|p| p.id == o).map_or(0, |p| p.home)
            };
            HOME_DIMS[(lv as usize).min(HOME_DIMS.len() - 1)]
        } else {
            room_dims(b)
        }
    }
    /// Pay up front to enlarge the active player's home. Furniture keeps its tiles; anything on the new doorway goes to home storage.
    pub fn expand_home(&mut self) -> Result<(), &'static str> {
        let lv = self.w.p.home as usize;
        if lv >= EXPAND_COST.len() {
            return Err("Your home is already as big as it gets!");
        }
        if self.w.p.debt > 0 {
            return Err("Pay off your current balance first.");
        }
        let cost = EXPAND_COST[lv];
        let (w, d) = HOME_DIMS[lv + 1];
        let mut clear = vec![(w / 2, d - 1)];
        for (t, _) in stairs(lv as u32 + 1, 0) {
            clear.push(t);
            clear.push((t.0, t.1 + 1))
        }
        let hit = |f: &Furn| {
            if def(f.def).wall {
                lv + 1 >= 4 && f.x == STAIR_UP.0
            } else {
                foot(f.def, f.x, f.z, f.rot).iter().any(|t| clear.contains(t))
            }
        };
        self.set_story(0);
        if self.w.p.storage.len() + self.w.p.room.iter().filter(|f| hit(f)).count() > STORE_CAP {
            return Err("Clear some home storage first.");
        }
        self.w.p.debt = cost;
        self.w.p.home += 1;
        let (moved, keep): (Vec<Furn>, Vec<Furn>) = self.w.p.room.iter().copied().partition(|f| hit(f));
        self.w.p.room = keep;
        for f in &moved {
            self.w.p.storage.push(Slot { size_mm: 0, def: f.def, qty: 1, uid: f.uid })
        }
        if !moved.is_empty() {
            self.note("Furniture in the way of the new doorway or stairs was moved to your storage.")
        }
        Ok(())
    }

    /// Open letter `i` (newest first in the menu): marks it read and claims its gift once.
    pub fn open_letter(&mut self, i: usize) {
        let n = self.w.p.mail.len();
        if i >= n {
            return;
        }
        let k = n - 1 - i;
        self.w.p.mail[k].read = true;
        let l = self.w.p.mail[k].clone();
        if l.claimed {
            return;
        }
        if let Some(d) = l.item {
            if self.w.p.inv.len() >= INV_CAP {
                return self.note("Your pockets are full; the gift stays in the letter.");
            }
            if def(d).k == K::Furn || def(d).k == K::Cloth {
                self.add_furn(d);
            } else {
                self.add(d);
            }
        }
        if l.coins > 0 {
            self.w.p.money += l.coins;
            self.note(format!("{} coins were enclosed.", l.coins));
            self.snd.push(Snd::Coin)
        }
        self.w.p.mail[k].claimed = true;
    }

    /// Give one unit of pocket slot `slot` to resident `j` (once per resident per day per player).
    pub fn give_gift(&mut self, j: usize, slot: usize) {
        let before = self.w.p.npcs[j].gifted;
        self.give_gift_inner(j, slot);
        if !before && self.w.p.npcs[j].gifted {
            self.ev(Ev::Gift, 1)
        }
    }
    pub(super) fn give_gift_inner(&mut self, j: usize, slot: usize) {
        let r = &self.res(j);
        let who = format!("{} the {}", r.name, r.species);
        if self.w.p.npcs[j].gifted {
            self.me.ui = Ui::Talk {
                npc: Some(j),
                who,
                lines: vec!["You already gave me something today. Save it for tomorrow!".into()],
                i: 0,
            };
            return;
        }
        let Some(sl) = self.w.p.inv.get(slot).copied() else { return };
        self.w.p.inv[slot].qty -= 1;
        if self.w.p.inv[slot].qty == 0 {
            self.w.p.inv.remove(slot);
        }
        let bday = (self.mo, self.dom) == BDAYS[self.w.roster[j]];
        let liked = def(sl.def).k == LIKES[r.pers] || bday;
        let s = &mut self.w.p.npcs[j];
        s.gifted = true;
        s.friend += if liked { 2 } else { 1 } + if bday { 3 } else { 0 };
        let name = def(sl.def).name;
        let line = if liked {
            self.npcs[j].happy = 2.;
            let hp = if self.me.scene == Scene::Town { self.npcs[j].pos } else { res_spot() };
            for k in 0..4 {
                self.hearts.push((vec3(hp.x + (k as f32 - 1.5) * 0.12, 1.3, hp.y), -(k as f32) * 0.12))
            }
            if bday {
                format!("A {name} for my birthday?! You remembered! Thank you so much!")
            } else {
                format!("A {name}! Oh, I really love these. Thank you!")
            }
        } else {
            format!("A {name}? How kind. Thank you!")
        };
        self.snd.push(Snd::Blip);
        self.me.ui = Ui::Talk { npc: Some(j), who, lines: vec![line], i: 0 };
    }
}
