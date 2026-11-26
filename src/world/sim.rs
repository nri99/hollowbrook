//! Per-frame simulation: wildlife spawns, residents walking and chatting, path finding.
use super::*;

impl Game {
    pub(super) fn sim_life(&mut self, dt: f32) {
        if self.ibubbles.len() < 2
            && std::iter::once(&self.me).chain(self.others.iter()).any(|p| p.scene == Scene::Islet)
            && gen_range(0, 50) == 0
        {
            let t = (gen_range(0, IW as i32), gen_range(0, IH as i32));
            if tile_at_i(&self.imap, t.0, t.1) == SEA && (t.0 - 7).abs() + (t.1 - 6).abs() < 10 {
                self.ibubbles.push((tc(t), [74, 74, 30, 72][gen_range(0, 4)]))
            }
        }
        let on_islet = std::iter::once(&self.me).chain(self.others.iter()).any(|p| p.scene == Scene::Islet);
        if !on_islet {
            self.ibugs.clear()
        } else {
            if self.ibugs.len() < 2 && gen_range(0, 120) == 0 {
                let t = (gen_range(1, IW as i32 - 1), gen_range(1, IH as i32 - 1));
                if tile_at_i(&self.imap, t.0, t.1) == GRASS {
                    let av = Self::islet_bug_pool(self.mo, self.hr);
                    if !av.is_empty() {
                        self.ibugs.push((av[gen_range(0, av.len())], tc(t), gen_range(0., std::f32::consts::TAU)))
                    }
                }
            }
            let t = self.t;
            for b in &mut self.ibugs {
                b.1 += vec2((t * 2. + b.2).sin(), (t * 1.3 + b.2).cos()) * 0.4 * dt
            }
        }
        let anchors: Vec<Vec2> = std::iter::once(&self.me)
            .chain(self.others.iter())
            .filter(|p| p.scene == Scene::Town)
            .map(|p| p.pos)
            .collect();
        if anchors.is_empty() {
            return;
        }
        let fishing = std::iter::once(&self.me).chain(self.others.iter()).any(|p| p.fish.is_some());
        let (p, mo, hr, t) = (anchors[gen_range(0, anchors.len())], self.mo, self.hr, self.t);
        let map = &self.map;
        if !fishing {
            self.shadows.retain(|s| anchors.iter().any(|a| s.pos.distance(*a) < 18.))
        }
        self.bugs.retain(|b| anchors.iter().any(|a| b.1.distance(*a) < 16.));
        if self.shadows.len() < 4 && gen_range(0, 30) == 0 {
            let tt = tile_of(p + vec2(gen_range(-11., 11.), gen_range(-11., 11.)));
            let c = tile_at(map, tt.0, tt.1);
            if matches!(c, RIVER | SEA) {
                let hab = if c == SEA { 2 } else { 1 };
                let mouth =
                    c == RIVER && (-3..=3).any(|dx| (-3..=3).any(|dz| tile_at(map, tt.0 + dx, tt.1 + dz) == SEA));
                let av = Self::fish_pool(hab, mouth, mo, hr);
                if !av.is_empty() {
                    let d = loop {
                        let d = av[gen_range(0, av.len())];
                        if def(d).sell < 1000 || gen_range(0, 4) == 0 {
                            break d;
                        }
                    };
                    self.shadows.push(Shadow { pos: tc(tt), def: d, vel: Vec2::ZERO })
                }
            }
        }
        let att: Vec<usize> = std::iter::once(&self.me)
            .chain(self.others.iter())
            .filter_map(|p| p.fish.as_ref().and_then(|f| f.fish))
            .collect();
        for (i, s) in self.shadows.iter_mut().enumerate() {
            if att.contains(&i) {
                continue;
            }
            if gen_range(0, 60) == 0 {
                s.vel = vec2(gen_range(-0.5, 0.5), gen_range(-0.5, 0.5))
            }
            let n = s.pos + s.vel * dt;
            let nt = tile_of(n);
            if matches!(tile_at(map, nt.0, nt.1), RIVER | SEA) {
                s.pos = n
            } else {
                s.vel = -s.vel
            }
        }
        self.bubbles.retain(|b| anchors.iter().any(|a| b.0.distance(*a) < 20.));
        if self.bubbles.len() < 3 && gen_range(0, 40) == 0 {
            let tt = tile_of(p + vec2(gen_range(-9., 9.), gen_range(-6., 6.)));
            if tile_at(map, tt.0, tt.1) == SEA {
                let d = [70, 70, 71, 71, 72, 73][gen_range(0, 6)];
                self.bubbles.push((tc(tt), d))
            }
        }
        if self.bugs.len() < 4 && gen_range(0, if rainy(self.w.day) { 400 } else { 90 }) == 0 {
            let tt = tile_of(p + vec2(gen_range(-9., 9.), gen_range(-9., 9.)));
            if matches!(tile_at(map, tt.0, tt.1), GRASS | FLOWER) {
                let av = Self::bug_pool(rainy(self.w.day), mo, hr);
                if !av.is_empty() {
                    self.bugs.push((av[gen_range(0, av.len())], tc(tt), gen_range(0., std::f32::consts::TAU)))
                }
            }
        }
        for b in &mut self.bugs {
            b.1 += vec2((t * 2. + b.2).sin(), (t * 1.3 + b.2).cos()) * 0.5 * dt
        }
    }
    pub(super) fn sim_npcs(&mut self, dt: f32) {
        let talkers: Vec<(usize, Vec2, bool)> = std::iter::once(&self.me)
            .chain(self.others.iter())
            .filter_map(|p| {
                if let Ui::Talk { npc: Some(j), .. } = p.ui {
                    Some((j, p.pos, p.scene == Scene::Town))
                } else {
                    None
                }
            })
            .collect();
        for i in 0..4 {
            if self.visit.is_some_and(|v| v.0 == i) {
                continue;
            }
            if let Some(spot) = self.seek_hide_position(i) {
                if self.npcs[i].inside {
                    self.npcs[i].inside = false;
                    self.npcs[i].pos = door_stand(HOUSES[i]);
                    self.npcs[i].path.clear();
                }
                self.npcs[i].moving = false;
                if talkers.iter().any(|t| t.0 == i) {
                    continue;
                }
                if self.npcs[i].pos.distance(spot) < 0.05 {
                    self.npcs[i].path.clear();
                    continue;
                }
                let tile = tile_of(spot);
                if self.npcs[i].path.last() != Some(&tile) {
                    self.npcs[i].path = self.bfs(tile_of(self.npcs[i].pos), tile, true).unwrap_or_default();
                }
                self.follow(i, dt);
                continue;
            }
            if let Some(&(a, b, _)) = self.chats.iter().find(|c| c.0 == i || c.1 == i) {
                let o = if a == i { b } else { a };
                let d = self.npcs[o].pos - self.npcs[i].pos;
                if d.length() > 0.01 {
                    self.npcs[i].yaw = yaw_of(d)
                }
                self.npcs[i].moving = false;
                continue;
            }
            let loc = self.loc(i);
            {
                let n = &mut self.npcs[i];
                n.happy = (n.happy - dt).max(0.);
                n.moving = false;
            }
            if let Some(&(_, pp, town)) = talkers.iter().find(|t| t.0 == i) {
                let d = pp - self.npcs[i].pos;
                if town && d.length() > 0.01 {
                    self.npcs[i].yaw = yaw_of(d)
                }
                continue;
            }
            if loc == Loc::Out {
                if self.npcs[i].inside {
                    let n = &mut self.npcs[i];
                    n.inside = false;
                    n.pos = door_stand(HOUSES[i]);
                    n.path.clear();
                    n.wait = 1.;
                    continue;
                }
                if self.npcs[i].wait > 0. {
                    self.npcs[i].wait -= dt;
                    continue;
                }
                if self.npcs[i].path.is_empty() {
                    let cur = tile_of(self.npcs[i].pos);
                    let mut found = None;
                    let buddy = (0..4)
                        .filter(|&j| j != i && !self.npcs[j].inside)
                        .nth(gen_range(0, 3))
                        .filter(|_| gen_range(0, 3) == 0);
                    if let Some(j) = buddy {
                        let bt = tile_of(self.npcs[j].pos);
                        for t in [(bt.0 + 1, bt.1), (bt.0 - 1, bt.1), (bt.0, bt.1 + 1)] {
                            if self.walk_tile(t.0, t.1) {
                                if let Some(p) = self.bfs(cur, t, true) {
                                    found = Some(p);
                                    break;
                                }
                            }
                        }
                    }
                    if found.is_none() {
                        for _ in 0..6 {
                            let t = (cur.0 + gen_range(-6, 7), cur.1 + gen_range(-6, 7));
                            if self.walk_tile(t.0, t.1) {
                                if let Some(p) = self.bfs(cur, t, true) {
                                    found = Some(p);
                                    break;
                                }
                            }
                        }
                    }
                    match found {
                        Some(p) => self.npcs[i].path = p,
                        None => self.npcs[i].wait = 1.,
                    }
                    continue;
                }
                if self.follow(i, dt) {
                    self.npcs[i].wait = gen_range(1.5, 5.)
                } // idle pause between strolls
            } else if !self.npcs[i].inside {
                let door = door_tile(HOUSES[i]);
                let cur = tile_of(self.npcs[i].pos);
                if cur == door {
                    self.npcs[i].inside = true
                } else if self.npcs[i].path.last() != Some(&door) {
                    match self.bfs(cur, door, true) {
                        Some(p) => self.npcs[i].path = p,
                        None => self.npcs[i].inside = true,
                    }
                } else {
                    self.follow(i, dt);
                }
            }
        }
    }
    /// Residents who meet outdoors sometimes stop and chat (speech bubbles), then carry on.
    pub fn sim_chats(&mut self, dt: f32) {
        for c in &mut self.chats {
            c.2 -= dt
        }
        let ended: Vec<(usize, usize, f32)> = self.chats.iter().copied().filter(|c| c.2 <= 0.).collect();
        for c in ended {
            self.npcs[c.0].wait = 1.;
            self.npcs[c.1].wait = 1.;
        }
        self.chats.retain(|c| c.2 > 0.);
        if gen_range(0, 60) != 0 {
            return;
        }
        for a in 0..4 {
            for b in a + 1..4 {
                let (na, nb) = (&self.npcs[a], &self.npcs[b]);
                if self.seek_hide_position(a).is_some()
                    || self.seek_hide_position(b).is_some()
                    || na.inside
                    || nb.inside
                    || self.loc(a) != Loc::Out
                    || self.loc(b) != Loc::Out
                    || self.chats.iter().any(|c| [c.0, c.1].contains(&a) || [c.0, c.1].contains(&b))
                {
                    continue;
                }
                if na.pos.distance(nb.pos) < 2.6 {
                    for k in [a, b] {
                        self.npcs[k].path.clear();
                        self.npcs[k].wait = 5.
                    }
                    self.chats.push((a, b, 5.));
                    return;
                }
            }
        }
    }
    pub(super) fn follow(&mut self, i: usize, dt: f32) -> bool {
        let Some(&nt) = self.npcs[i].path.first() else { return true };
        if !self.walk_tile(nt.0, nt.1) {
            self.npcs[i].path.clear();
            return true;
        }
        let n = &mut self.npcs[i];
        let to = tc(nt) - n.pos;
        let st = 1.3 * dt;
        if to.length() <= st {
            n.pos = tc(nt);
            n.path.remove(0);
        } else {
            n.pos += to.normalize() * st;
            n.yaw = yaw_of(to)
        }
        n.moving = true;
        n.anim += dt * 7.;
        n.path.is_empty()
    }
    /// 4-neighbour BFS. `town` = outdoor walkability, else the current scene's passability.
    pub fn bfs(&self, a: (i32, i32), b: (i32, i32), town: bool) -> Option<Vec<(i32, i32)>> {
        if a == b {
            return Some(vec![]);
        }
        let pass = |t: (i32, i32)| match (town, self.me.scene) {
            (false, Scene::Room(r)) => !self.room_block(r, t.0, t.1, 0),
            (false, Scene::Islet) => matches!(tile_at_i(&self.imap, t.0, t.1), GRASS | SAND | BRIDGE) && t != ISTALL,
            _ => self.walk_tile(t.0, t.1),
        };
        if !pass(b) {
            return None;
        }
        let mut prev = HashMap::from([(a, a)]);
        let mut q = VecDeque::from([a]);
        while let Some(c) = q.pop_front() {
            if c == b {
                break;
            }
            if prev.len() > 2500 {
                return None;
            }
            for d in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (c.0 + d.0, c.1 + d.1);
                if !prev.contains_key(&n) && pass(n) {
                    prev.insert(n, c);
                    q.push_back(n)
                }
            }
        }
        if !prev.contains_key(&b) {
            return None;
        }
        let mut path = vec![b];
        let mut c = b;
        while prev[&c] != a {
            c = prev[&c];
            path.push(c)
        }
        path.reverse();
        Some(path)
    }
}
