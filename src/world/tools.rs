//! Tools: cycling, swimming and diving, net/rod/shovel/can use, fishing, wildlife pools.
use super::*;

impl Game {
    // ---- tools ----
    pub fn cycle_tool(&mut self) {
        let mut opts: Vec<Option<u16>> = vec![None];
        let mut ts = self.w.p.tools.clone();
        ts.sort();
        opts.extend(ts.into_iter().map(Some));
        let i = opts.iter().position(|&o| o == self.w.p.equip).unwrap_or(0);
        self.w.p.equip = opts[(i + 1) % opts.len()];
        self.note(match self.w.p.equip {
            Some(t) => format!("Equipped: {}", def(t).name),
            None => "Tools put away.".into(),
        });
    }
    /// Player is in the sea (outdoors). Swimming is entered by walking off the beach (provisional rule).
    pub fn swimming(&self) -> bool {
        match self.me.scene {
            Scene::Town => {
                let t = tile_of(self.me.pos);
                self.tile(t.0, t.1) == SEA
            }
            Scene::Islet => {
                let t = tile_of(self.me.ipos);
                tile_at_i(&self.imap, t.0, t.1) == SEA
            }
            _ => false,
        }
    }
    /// Dive under the nearest bubble spot within reach and collect the sea creature there.
    pub fn dive(&mut self) {
        let p = self.ppos();
        self.fx.push((vec3(p.x, -0.17, p.y), 0., 0));
        self.snd.push(Snd::Splash);
        let islet = self.me.scene == Scene::Islet;
        let list = if islet { &self.ibubbles } else { &self.bubbles };
        let Some(i) = list.iter().position(|b| b.0.distance(p) < 1.3) else {
            return self.note("You dive... nothing down here.");
        };
        let d = list[i].1;
        if self.add(d) {
            if islet {
                self.ibubbles.remove(i);
            } else {
                self.bubbles.remove(i);
            }
            self.note(format!("You found a {} while diving!", def(d).name));
            self.ev(Ev::Dive, 1)
        }
    }
    pub fn use_tool(&mut self) {
        if let Some(d) = self.w.p.equip.filter(|&d| is_gear(d)) {
            return self.use_gear(d);
        }
        if matches!(self.me.scene, Scene::Frontier | Scene::Planet(_)) {
            return self.note("Hollowbrook tools aren't much use out here.");
        }
        if self.swimming() {
            return self.dive();
        }
        if self.me.scene == Scene::Islet && self.swimming() {
            return self.dive();
        }
        if self.me.scene == Scene::Islet {
            // only the net works on Gullrock (for island bugs)
            if self.w.p.equip != Some(60) {
                return self.note("Only the net is useful on Gullrock.");
            }
            self.me.swing = 0.35;
            let at = self.me.ipos + self.me.face * 0.7;
            self.fx.push((vec3(at.x, 0.6, at.y), 0., 2));
            if let Some(j) = {
                let reach = 0.8 + 0.3 * self.tool_level(60) as f32;
                self.ibugs.iter().position(|b| b.1.distance(at) < reach)
            } {
                let d = self.ibugs.remove(j).0;
                if self.add_catch(d) {
                    self.ev(Ev::Bug, 1)
                }
            } else {
                self.snd.push(Snd::Rustle)
            }
            return;
        }
        if self.me.scene != Scene::Town {
            return;
        }
        if self.me.fish.is_some() {
            return self.reel();
        }
        let (p, f) = (self.me.pos, self.me.face);
        match self.w.p.equip {
            None => self.note("No tool equipped. Press Tab to cycle tools."),
            Some(60) => {
                self.me.swing = 0.35;
                let at = p + f * 0.7;
                self.fx.push((vec3(at.x, 0.6, at.y), 0., 2));
                if self.me.scene == Scene::Islet {
                    if let Some(j) = self.ibugs.iter().position(|b| b.1.distance(at) < 0.8) {
                        let d = self.ibugs.remove(j).0;
                        if self.add_catch(d) {
                            self.ev(Ev::Bug, 1)
                        }
                    } else {
                        self.snd.push(Snd::Rustle)
                    }
                } else if let Some(j) = {
                    let reach = 0.8 + 0.3 * self.tool_level(60) as f32;
                    self.bugs.iter().position(|b| b.1.distance(at) < reach)
                } {
                    let d = self.bugs.remove(j).0;
                    if self.add_catch(d) {
                        self.ev(Ev::Bug, 1)
                    }
                } else {
                    self.snd.push(Snd::Rustle)
                }
            }
            Some(CAN) => {
                self.me.swing = 0.35;
                self.water();
            }
            Some(61) => {
                let spot = [1.6, 2.2, 2.8].iter().map(|&k| p + f * k).find(|&q| {
                    let t = tile_of(q);
                    matches!(self.tile(t.0, t.1), RIVER | SEA)
                });
                let Some(bob) = spot else { return self.note("Face the water to cast.") };
                self.me.swing = 0.35;
                let bt = tile_of(bob);
                let hab = if self.tile(bt.0, bt.1) == SEA { 2 } else { 1 };
                let fish = self
                    .shadows
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| (def(s.def).hab == hab || def(s.def).hab == 3) && s.pos.distance(bob) < 5.)
                    .min_by(|a, b| a.1.pos.distance(bob).total_cmp(&b.1.pos.distance(bob)))
                    .map(|(i, _)| i);
                self.me.fish = Some(Fishing {
                    bob,
                    fish,
                    t: 0.,
                    nib: 0,
                    need: gen_range(1u8, 4 - self.tool_level(61)),
                    bite: false,
                    dip: 0.3,
                });
                self.snd.push(Snd::Splash);
                self.fx.push((vec3(bob.x, -0.17, bob.y), 0., 0));
            }
            Some(_) => {
                self.me.swing = 0.35;
                let t = tile_of(p + f * 0.9);
                self.fx.push((vec3(t.0 as f32 + 0.5, 0.05, t.1 as f32 + 0.5), 0., 1));
                if self.tile(t.0, t.1) == ROCK {
                    self.snd.push(Snd::Dig);
                    if self.w.rocks_hit.contains(&t) {
                        return self.note("This rock has nothing more to give today.");
                    }
                    if self.add(81) {
                        self.w.rocks_hit.push(t);
                        self.ev(Ev::Rock, 1);
                        for _ in 0..self.tool_level(62) {
                            self.add(81);
                        }
                    }
                    return;
                }
                if let Some(i) = self.w.garden.iter().position(|p| (p.x, p.z) == t) {
                    self.w.garden.remove(i);
                    self.snd.push(Snd::Dig);
                    return self.note("You dug up the flower.");
                }
                if let Some(i) = self.w.holes.iter().position(|&h| h == t) {
                    self.w.holes.remove(i);
                    self.snd.push(Snd::Dig);
                    self.note("You filled the hole.")
                } else if let Some(i) = self.w.digs.iter().position(|&h| h == t) {
                    self.w.digs.remove(i);
                    self.w.holes.push(t);
                    self.snd.push(Snd::Dig);
                    let d = [47, 47, 47, 45, 46][gen_range(0, 5)];
                    if self.add(d) {
                        self.ev(Ev::Dig, 1);
                        if self.tool_level(62) >= 2 && gen_range(0, 3) == 0 {
                            let b = [45, 46][gen_range(0, 2)];
                            if self.add(b) {
                                self.note(format!("Your masterwork shovel turned up a {} too!", def(b).name))
                            }
                        }
                    }
                } else if matches!(self.tile(t.0, t.1), GRASS | SAND | PATH)
                    && self.walk_tile(t.0, t.1)
                    && !self.occupied(t.0, t.1)
                    && t != tile_of(p)
                {
                    self.w.holes.push(t);
                    self.snd.push(Snd::Dig);
                    self.note("You dug a hole.")
                } else {
                    self.note("You can't dig there.")
                }
            }
        }
    }
    pub(super) fn reel(&mut self) {
        let Some(f) = self.me.fish.take() else { return };
        match f.fish {
            Some(si) if f.bite => {
                self.fx.push((vec3(f.bob.x, -0.17, f.bob.y), 0., 0));
                let Some(sh) = self.take_shadow(si) else { return self.note("The fish slipped away.") };
                let d = sh.def;
                self.snd.push(Snd::Splash);
                if self.add_catch(d) {
                    self.note(format!("You caught a {}!", def(d).name));
                    self.ev(Ev::Fish, 1)
                }
            }
            Some(si) => {
                self.take_shadow(si);
                self.note("Too early! The fish swam off.")
            }
            None => self.note("You reel in your line."),
        }
    }

    /// Remove a fish shadow and fix every player's hooked-fish index (shadows are shared, so indices shift).
    pub(super) fn take_shadow(&mut self, k: usize) -> Option<Shadow> {
        if k >= self.shadows.len() {
            return None;
        }
        let s = self.shadows.remove(k);
        for p in std::iter::once(&mut self.me).chain(self.others.iter_mut()) {
            if let Some(f) = &mut p.fish {
                match f.fish {
                    Some(j) if j == k => f.fish = None,
                    Some(j) if j > k => f.fish = Some(j - 1),
                    _ => {}
                }
            }
        }
        Some(s)
    }
    // ---- simulation ----
    pub(super) fn sim_fish(&mut self, dt: f32) {
        let Some(mut f) = self.me.fish.take() else { return };
        f.t += dt;
        f.dip = (f.dip - dt * 2.5).max(0.);
        let mut keep = true;
        match f.fish {
            None => {
                if f.t > 6. {
                    keep = false;
                    self.note("Nothing seems interested...")
                }
            }
            Some(si) => {
                if f.bite {
                    if f.t > 0.8 + 0.3 * self.tool_level(61) as f32 {
                        keep = false;
                        self.take_shadow(si);
                        self.note("It got away!")
                    }
                } else if let Some(sh) = self.shadows.get_mut(si) {
                    let to = f.bob - sh.pos;
                    if to.length() > 0.3 {
                        sh.pos += to.normalize() * 0.8 * dt;
                        f.t = 0.
                    } else if f.t > 0.9 {
                        f.t = 0.;
                        f.nib += 1;
                        f.dip = 0.5;
                        self.snd.push(Snd::Blip);
                        if f.nib >= f.need {
                            f.bite = true;
                            f.dip = 1.2;
                            self.snd.push(Snd::Splash)
                        }
                    }
                } else {
                    keep = false;
                    self.note("The fish swam away.")
                }
            }
        }
        if keep {
            self.me.fish = Some(f)
        }
    }
    /// Town bugs available now (Rain Snail only in rain; Gullrock-only bugs excluded).
    pub fn bug_pool(rain: bool, mo: u32, hr: u32) -> Vec<u16> {
        DEFS.iter()
            .filter(|d| d.k == K::Bug && d.hab == 0 && (d.id != 8 || rain) && avail(d, mo, hr))
            .map(|d| d.id)
            .collect()
    }
    pub fn islet_bug_pool(mo: u32, hr: u32) -> Vec<u16> {
        DEFS.iter().filter(|d| d.k == K::Bug && d.hab == 4 && avail(d, mo, hr)).map(|d| d.id).collect()
    }
    /// Fish for a water tile: hab 1 river / 2 sea; river tiles near the sea (`mouth`) also hold hab 3 fish.
    pub fn fish_pool(hab: u8, mouth: bool, mo: u32, hr: u32) -> Vec<u16> {
        DEFS.iter()
            .filter(|d| d.k == K::Fish && (d.hab == hab || (mouth && d.hab == 3)) && avail(d, mo, hr))
            .map(|d| d.id)
            .collect()
    }
}
