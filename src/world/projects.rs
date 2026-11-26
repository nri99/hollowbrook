//! Town liveliness, notice board, money and material projects, clothing.
use super::*;

impl Game {
    pub fn liveliness(&self) -> i32 {
        let flowers = self.map.iter().filter(|&&c| c == FLOWER).count() as i32 - self.w.trampled.len() as i32
            + 2 * self.w.garden.iter().filter(|p| p.stage == 2).count() as i32;
        (10 * self.w.built.len() as i32 + 2 * self.w.donated.len() as i32 + flowers / 2 - 3 * self.w.weeds.len() as i32)
            .clamp(0, 100)
    }
    pub fn live_word(l: i32) -> &'static str {
        if l < 20 {
            "Quiet"
        } else if l < 45 {
            "Cozy"
        } else if l < 70 {
            "Lively"
        } else {
            "Bustling"
        }
    }
    pub fn board_lines(&self) -> Vec<String> {
        let mut v =
            vec![format!("Town liveliness: {} / 100 ({}).", self.liveliness(), Game::live_word(self.liveliness()))];
        v.push(self.contest_notice());
        if let Some(s) = self.social_notice() {
            v.push(s);
        }
        if let Some(p) = &self.w.project {
            v.push(format!(
                "Current project: a {} proposed by {}. {}",
                PROJECTS[p.kind as usize].0,
                self.res(p.by).name,
                Game::remaining(p)
            ))
        }
        v.push("Funded public projects are started by the town leader at the Town Hall civic desk.".into());
        if let Some(m) = &self.w.mproject {
            v.push(format!(
                "Funded project: a {} ({} / {} coins). It is built the morning after it is fully funded.",
                MPROJECTS[m.idx].0, m.funded, MPROJECTS[m.idx].1
            ))
        }
        v.push(if self.w.built.is_empty() {
            "Built so far: nothing yet.".into()
        } else {
            format!(
                "Built so far: {}.",
                self.w.built.iter().map(|b| PROJECTS[b.kind as usize].0).collect::<Vec<_>>().join(", ")
            )
        });
        v
    }
    /// Front tile where the leader would place a money project.
    pub fn mp_front(&self) -> (i32, i32) {
        tile_of(self.me.pos + self.me.face)
    }
    /// Site check: plain free grass, not a reserved tile, and every building stays reachable.
    pub fn mp_site_ok(&mut self, t: (i32, i32)) -> bool {
        if self.seek_reserved(t) {
            return false;
        }
        if tile_at(&self.map, t.0, t.1) != GRASS
            || !self.walk_tile(t.0, t.1)
            || self.occupied(t.0, t.1)
            || [mailbox_tile(0), mailbox_tile(1), STALL, BOARD, DOCK, tile_of(self.me.pos)].contains(&t)
        {
            return false;
        }
        if self.w.project.as_ref().is_some_and(|p| (p.x, p.z) == t) {
            return false;
        }
        let keep = self.w.mproject.take();
        self.w.mproject = Some(MProject { idx: 0, x: t.0, z: t.1, funded: 0, log: vec![] });
        let ok = (0..BLDS.len()).all(|b| self.bfs(door_tile(SHOP), door_tile(b), true).is_some())
            && self.bfs(door_tile(SHOP), DOCK_APPROACH, true).is_some();
        self.w.mproject = keep;
        ok
    }
    /// Leader starts money project `idx` at the front tile.
    pub fn mp_start(&mut self, idx: usize) -> Result<(), &'static str> {
        if self.me.id != LEADER {
            return Err("Only the town leader can start public projects.");
        }
        if self.w.mproject.is_some() {
            return Err("A funded project is already underway.");
        }
        let t = self.mp_front();
        if !self.mp_site_ok(t) {
            return Err("Can't build there: needs open grass, paths clear.");
        }
        self.w.mproject = Some(MProject { idx, x: t.0, z: t.1, funded: 0, log: vec![] });
        Ok(())
    }
    /// Donate up to `amount` of the active player's own coins (capped by what is still needed).
    pub fn mp_fund(&mut self, amount: u32) -> Result<u32, &'static str> {
        let Some(m) = self.w.mproject.as_mut() else { return Err("No funded project right now.") };
        let left = MPROJECTS[m.idx].1 - m.funded;
        if left == 0 {
            return Err("It's fully funded! It will be built tomorrow morning.");
        }
        let a = amount.min(left).min(self.w.p.money);
        if a == 0 {
            return Err("Not enough coins.");
        }
        self.w.p.money -= a;
        m.funded += a;
        let id = self.me.id;
        if let Some(l) = m.log.iter_mut().find(|l| l.0 == id) {
            l.1 += a
        } else {
            m.log.push((id, a))
        }
        Ok(a)
    }
    /// A resident proposes the next project at the first valid candidate site (never blocking a building route).
    pub fn propose(&mut self) {
        if self.w.project.is_some() {
            return;
        }
        let has_bridge = self.w.built.iter().any(|b| b.kind == PK_BRIDGE);
        for step in 0..4 {
            let kind = ((self.w.built.len() + step) % 4) as u8;
            if kind == PK_BRIDGE && has_bridge {
                continue;
            }
            for &(x, z) in SITES[kind as usize] {
                let ts = ptiles(kind, x, z);
                let ok = if kind == PK_BRIDGE {
                    ts.iter().all(|t| tile_at(&self.map, t.0, t.1) == RIVER)
                } else {
                    ts.iter().all(|t| {
                        tile_at(&self.map, t.0, t.1) == GRASS
                            && self.walk_tile(t.0, t.1)
                            && !self.occupied(t.0, t.1)
                            && *t != mailbox_tile(0)
                            && *t != mailbox_tile(1)
                            && *t != STALL
                    })
                };
                if !ok {
                    continue;
                }
                let by = gen_range(0, 4);
                self.w.project = Some(Project {
                    kind,
                    x,
                    z,
                    need: PROJECTS[kind as usize].1.iter().map(|&(c, q)| (c, q, 0)).collect(),
                    by,
                    log: vec![],
                });
                let routes = kind == PK_BRIDGE
                    || (0..BLDS.len()).all(|b| self.bfs(door_tile(SHOP), door_tile(b), true).is_some());
                if !routes {
                    self.w.project = None;
                    continue;
                }
                let r = self.res(by);
                self.mail_all(
                    r.name,
                    &format!(
                        "Let's build a {} together! I put up a sign where it should go. Bring materials when you can.",
                        PROJECTS[kind as usize].0
                    ),
                );
                return;
            }
        }
    }
    /// Contribute matching pocket items to the active project (never more than still needed); completes it when full.
    pub fn contribute(&mut self) {
        let Some(mut p) = self.w.project.clone() else { return };
        let mut given = 0u16;
        for n in p.need.iter_mut() {
            while n.2 < n.1 {
                let Some(d) = self.w.p.inv.iter().find(|s| s.uid == 0 && mat_ok(n.0, s.def)).map(|s| s.def) else {
                    break;
                };
                self.remove_one(d);
                n.2 += 1;
                given += 1
            }
        }
        if given > 0 {
            if let Some(l) = p.log.iter_mut().find(|l| l.0 == self.me.id) {
                l.1 += given
            } else {
                p.log.push((self.me.id, given))
            }
            self.snd.push(Snd::Coin)
        }
        let name = PROJECTS[p.kind as usize].0;
        if p.need.iter().all(|n| n.2 >= n.1) {
            self.w.built.push(Built { kind: p.kind, x: p.x, z: p.z, day: self.w.day });
            self.w.project = None;
            for (k, n) in self.npcs.iter_mut().enumerate() {
                if !n.inside {
                    n.happy = 3.;
                    let hp = n.pos;
                    for j in 0..4 {
                        self.hearts.push((vec3(hp.x + (j as f32 - 1.5) * 0.12, 1.3, hp.y), -(j as f32) * 0.12))
                    }
                }
                let _ = k;
            }
            self.fx.push((vec3(p.x as f32 + 0.5, 0.05, p.z as f32 + 0.5), 0., 1));
            self.me.ui = Ui::Talk {
                npc: None,
                who: "Project sign".into(),
                lines: vec![
                    format!("You gave {given} item(s). The {name} is finished! Everyone gathers to admire it."),
                    format!("Town liveliness is now {} ({}).", self.liveliness(), Game::live_word(self.liveliness())),
                ],
                i: 0,
            };
        } else {
            let msg = if given > 0 {
                format!("You contributed {given} item(s) to the {name}.")
            } else {
                format!("You have nothing the {name} needs right now.")
            };
            self.me.ui =
                Ui::Talk { npc: None, who: "Project sign".into(), lines: vec![msg, Game::remaining(&p)], i: 0 };
            self.w.project = Some(p);
        }
    }

    // ---- clothing ----
    pub fn buy_cloth(&mut self, d: u16) -> Result<(), &'static str> {
        let p = def(d).buy;
        if self.w.p.money < p {
            return Err("Not enough coins.");
        }
        if self.w.p.inv.len() >= INV_CAP {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= p;
        self.add_furn(d);
        self.ev(Ev::Buy, 1);
        self.w.spent += p;
        Ok(())
    }
    /// Wear a shirt from pockets; the shirt worn before goes back into the same pocket slot (no duplication, no loss).
    pub fn wear(&mut self, uid: u32) {
        let Some(i) = self.w.p.inv.iter().position(|s| s.uid == uid && def(s.def).k == K::Cloth) else { return };
        let d = self.w.p.inv[i].def;
        let slot = if is_hat(d) { &mut self.w.p.hat } else { &mut self.w.p.shirt };
        match slot.replace(d) {
            Some(old) => self.w.p.inv[i].def = old,
            None => {
                self.w.p.inv.remove(i);
            }
        }
        self.note(format!("You put on the {}.", def(d).name));
    }
}
