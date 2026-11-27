//! Interaction: choosing what the player faces (Tg), labels, prompts, interact(), doors, conversations.
use super::*;

impl Game {
    // ---- targeting: distance + facing + line of sight; doors win when deliberately faced ----
    pub fn target(&self) -> Option<Tg> {
        let p = self.ppos();
        let f = self.me.face;
        let mut best: Option<(f32, Tg)> = None;
        let mut cand = |c: Vec2, reach: f32, wt: f32, tg: Tg, own: Option<(i32, i32)>| {
            let v = c - p;
            let d = v.length();
            if d > reach {
                return;
            }
            let dot = if d < 0.05 { 1. } else { v.dot(f) / d };
            if dot < 0.3 {
                return;
            }
            if self.me.scene == Scene::Town && !self.los(p, c, own) {
                return;
            }
            let s = d * (1.6 - dot) * wt;
            if best.map_or(true, |b| s < b.0) {
                best = Some((s, tg))
            }
        };
        match self.me.scene {
            Scene::Town => {
                for (i, n) in self.npcs.iter().enumerate() {
                    if !n.inside {
                        cand(n.pos, 1.7, 1., Tg::Npc(i), None)
                    }
                }
                for b in 0..BLDS.len() {
                    cand(door_face(b), 1.35, 0.6, Tg::Door(b), None)
                }
                for (i, t) in self.w.trees.iter().enumerate() {
                    cand(tc((t.x, t.z)), 1.45, 1., Tg::Tree(i), Some((t.x, t.z)))
                }
                for (i, g) in self.w.ground.iter().enumerate() {
                    cand(vec2(g.x, g.z), 1.2, 0.8, Tg::Item(i), None)
                }
                for (i, &w) in self.w.weeds.iter().enumerate() {
                    cand(tc(w), 1.2, 1., Tg::Weed(i), None)
                }
                for o in 0..2 {
                    cand(tc(mailbox_tile(o)), 1.4, 0.9, Tg::Mailbox(o), Some(mailbox_tile(o)))
                }
                if let Some(p) = &self.w.project {
                    let c = if p.kind == PK_BRIDGE { vec2(p.x as f32 + 1., p.z as f32 + 1.2) } else { tc((p.x, p.z)) };
                    cand(c, 1.5, 0.8, Tg::Site, Some((p.x, p.z)))
                }
                cand(tc(BOARD), 1.5, 0.8, Tg::Board, Some(BOARD));
                cand(tc(WOODS_DOOR), 1.6, 0.9, Tg::WoodsDoor, Some(WOODS_DOOR));
                if self.contest_host_open() {
                    let (x, z) = crate::events::CONTEST_SPOT;
                    cand(vec2(x, z), 1.5, 0.7, Tg::Contest, None);
                }
                if let Some(m) = &self.w.mproject {
                    cand(tc((m.x, m.z)), 1.5, 0.8, Tg::MSite, Some((m.x, m.z)))
                }
                cand(vec2(DOCK.0 as f32 + 0.5, DOCK.1 as f32 + 0.3), 1.4, 0.7, Tg::Dock, None);
                for (i, b) in self.w.built.iter().enumerate() {
                    if b.kind == 0 {
                        cand(tc((b.x, b.z)), 1.4, 0.9, Tg::Built(i), Some((b.x, b.z)))
                    }
                }
                if self.camp_open() {
                    cand(vec2(CAMP_SPOT.0, CAMP_SPOT.1), 1.5, 0.8, Tg::Camper, None)
                }
                cand(tc(CAMP_SIGN), 1.4, 0.8, Tg::CampSign, Some(CAMP_SIGN));
                if let Some(sp) = &self.w.surprise {
                    cand(tc((sp.x, sp.z)), 1.3, 0.9, Tg::Surprise, Some((sp.x, sp.z)))
                }
                for (i, p) in self.w.garden.iter().enumerate() {
                    cand(tc((p.x, p.z)), 1.2, 0.9, Tg::Plant(i), Some((p.x, p.z)))
                }
                if self.w.p.inv.iter().any(|s| SEEDS.contains(&s.def)) {
                    let ft = tile_of(self.me.pos + self.me.face * 0.9);
                    if self.plant_ok(ft) {
                        cand(tc(ft), 1.2, 0.85, Tg::PlantSpot, Some(ft))
                    }
                }
                if self.fair() {
                    cand(tc(STALL), 1.5, 0.8, Tg::Stall, Some(STALL))
                } else if self.fest().is_some() {
                    cand(tc(STALL), 1.5, 0.8, Tg::Fest, Some(STALL))
                } else if self.art_open() {
                    cand(tc(STALL), 1.5, 0.8, Tg::ArtStall, Some(STALL))
                }
            }
            Scene::Cockpit(_) => {
                cand(tc(CK_SEAT), 1.6, 0.8, Tg::Pilot, None);
                cand(vec2(CK_EXIT.0, CK_EXIT.1), 1.4, 0.8, Tg::ShipExit, None);
            }
            Scene::Planet(k) => {
                if let Some(r) = self.my_ramp() {
                    cand(r, 1.6, 0.8, Tg::Ship, None);
                }
                let i = (k as usize).clamp(1, 2) - 1;
                for (s, &(x, z)) in SAMPLE_SPOTS[i].iter().enumerate() {
                    if !self.sample_taken(k, s) {
                        cand(tc((x, z)), 1.4, 0.9, Tg::Sample(s), None);
                    }
                }
                if let Some(b) = self.w.p.bounty.as_ref().filter(|b| !b.caught && b.planet == k) {
                    for (i, p) in b.pos.iter().enumerate() {
                        cand(vec2(p[0], p[1]), 1.4, 0.9, Tg::Fugitive(i), None)
                    }
                }
            }
            Scene::Cantina => {
                cand(vec2(CT_EXIT.0, CT_EXIT.1), 1.4, 0.8, Tg::CantinaExit, None);
                cand(vec2(CT_BROKER.0, CT_BROKER.1), 1.5, 0.8, Tg::CantinaBroker, None);
            }
            Scene::Frontier => {
                cand(tc(F_BOARD), 1.6, 0.8, Tg::BountyBoard, None);
                cand(vec2(F_BROKER.0, F_BROKER.1), 1.6, 0.8, Tg::GuildMaster, None);
                cand(vec2(F_CANTINA.0, F_CANTINA.1), 1.6, 0.85, Tg::Cantina, None);
                if let Some(r) = self.my_ramp() {
                    cand(r, 1.6, 0.8, Tg::Ship, None);
                }
                if !self.w.p.pal {
                    cand(vec2(F_PAL_SPOT.0, F_PAL_SPOT.1), 1.4, 0.9, Tg::Pal, None);
                }
                cand(vec2(F_BIKE.0, F_BIKE.1), 1.7, 0.85, Tg::Bike, None);
                for i in 0..LOCALS.len() {
                    cand(local_pos(i, self.t), 1.5, 0.9, Tg::Local(i), None);
                }
                cand(vec2(F_GATE.0 as f32 + 1., F_GATE.1 as f32 + 0.5), 1.6, 0.8, Tg::FGate, None);
                if let Some(b) = self.w.p.bounty.as_ref().filter(|b| !b.caught) {
                    for (i, p) in b.pos.iter().enumerate() {
                        cand(vec2(p[0], p[1]), 1.4, 0.9, Tg::Fugitive(i), None)
                    }
                }
            }
            Scene::Islet => {
                cand(vec2(PELL_I.0, PELL_I.1), 1.6, 0.8, Tg::Pell, None);
                cand(tc(ACT_BOARD), 1.4, 0.8, Tg::ActBoard, None);
                cand(tc(ISTALL), 1.5, 0.9, Tg::IStall, None);
                for (i, g) in self.w.iground.iter().enumerate() {
                    cand(vec2(g.x, g.z), 1.2, 0.8, Tg::IItem(i), None)
                }
            }
            Scene::Room(b) => {
                for fu in self.room_furn(b) {
                    cand(fcenter(&fu), 1.5, 1., Tg::Furn(fu.uid, fu.def), None)
                }
                if matches!(BLDS[b].kind, BK::Home(o) if o == self.w.p.id) {
                    for (t, to) in self.home_stairs(b) {
                        cand(tc(t), 1.6, 1., Tg::Stairs(to), None)
                    }
                }
                if BLDS[b].kind == BK::Shop {
                    cand(vec2(3.5, 1.5), 1.4, 0.9, Tg::Counter, None);
                    cand(vec2(3.5, 0.5), 2.6, 1.3, Tg::Keeper, None);
                    for (i, &d) in DISPLAYS.iter().enumerate() {
                        cand(tc(d), 1.5, 1., Tg::Display(i), None)
                    }
                    cand(tc(RACK), 1.5, 1., Tg::Rack, None);
                    if self.w.stock.len() > 5 {
                        cand(tc(EXTRA_DISPLAY), 1.5, 1., Tg::Display(7), None)
                    }
                    if self.shop_tier() >= 1 {
                        cand(tc(SPECIAL_DISPLAY), 1.5, 1., Tg::Display(8), None)
                    }
                }
                if BLDS[b].kind == BK::Civic {
                    cand(vec2(3.5, 1.5), 1.4, 0.9, Tg::Hazel, None);
                    cand(tc(KIOSK), 1.4, 0.9, Tg::Kiosk, None)
                }
                if BLDS[b].kind == BK::Museum {
                    cand(vec2(3.5, 1.5), 1.4, 0.9, Tg::Curator, None);
                    cand(vec2(0.5, 2.5), 1.5, 1., Tg::Exhibit(0), None);
                    cand(vec2(6.5, 2.5), 1.5, 1., Tg::Exhibit(1), None);
                    for (k, p) in MUSEUM_PED.iter().enumerate() {
                        cand(tc(p.0), 1.5, 1., Tg::Exhibit(2 + k), None)
                    }
                }
                if let BK::Res(r) = BLDS[b].kind {
                    if self.loc(r) == Loc::Home {
                        cand(res_spot(), 1.7, 0.9, Tg::Npc(r), None)
                    }
                }
                if let Some((r, vb, _)) = self.visit {
                    if vb == b {
                        cand(self.visit_spot(b), 1.7, 0.9, Tg::Npc(r), None)
                    }
                }
            }
        }
        best.map(|b| b.1)
    }
    pub(super) fn los(&self, a: Vec2, b: Vec2, own: Option<(i32, i32)>) -> bool {
        (1..10).all(|k| {
            let t = tile_of(a.lerp(b, k as f32 / 10.));
            Some(t) == own || !matches!(self.tile(t.0, t.1), BLD | PINE)
        })
    }
    pub fn tg_pos(&self, tg: Tg) -> Vec3 {
        let v = |p: Vec2, y: f32| vec3(p.x, y, p.y);
        match tg {
            Tg::Contest => vec3(crate::events::CONTEST_SPOT.0, 1.5, crate::events::CONTEST_SPOT.1),
            Tg::Npc(i) => v(
                match self.me.scene {
                    Scene::Town => self.npcs[i].pos,
                    Scene::Room(b) if self.visit.is_some_and(|x| x.0 == i && x.1 == b) => self.visit_spot(b),
                    _ => res_spot(),
                },
                1.5,
            ),
            Tg::Door(b) => v(door_face(b), 1.3),
            Tg::Tree(i) => v(tc((self.w.trees[i].x, self.w.trees[i].z)), 2.15),
            Tg::Item(i) => vec3(self.w.ground[i].x, 0.45, self.w.ground[i].z),
            Tg::Stairs(_) => {
                let t = self
                    .home_stairs(match self.me.scene {
                        Scene::Room(b) => b,
                        _ => HOME,
                    })
                    .first()
                    .map_or(STAIR_UP, |s| s.0);
                vec3(t.0 as f32 + 0.5, 1.6, t.1 as f32 + 0.5)
            }
            Tg::Weed(i) => v(tc(self.w.weeds[i]), 0.55),
            Tg::Rack => vec3(1.5, 1.8, 1.5),
            Tg::Mailbox(o) => v(tc(mailbox_tile(o)), 1.4),
            Tg::Stall | Tg::ArtStall | Tg::Fest => v(tc(STALL), 1.9),
            Tg::Camper => vec3(CAMP_SPOT.0, 1.7, CAMP_SPOT.1),
            Tg::Plant(i) => v(tc((self.w.garden[i].x, self.w.garden[i].z)), 0.7),
            Tg::PlantSpot => v(tc(tile_of(self.me.pos + self.me.face * 0.9)), 0.4),
            Tg::Surprise => self.w.surprise.as_ref().map_or(Vec3::ZERO, |sp| v(tc((sp.x, sp.z)), 0.9)),
            Tg::CampSign => v(tc(CAMP_SIGN), 1.5),
            Tg::Kiosk => v(tc(KIOSK), 1.5),
            Tg::BountyBoard => v(tc(F_BOARD), 1.9),
            Tg::WoodsDoor => v(tc(WOODS_DOOR), 2.4),
            Tg::Ship => {
                let r = self.my_ramp().unwrap_or(vec2(F_RAMP.0, F_RAMP.1));
                vec3(r.x, 1.9, r.y - 0.6)
            }
            Tg::Pilot => vec3(CK_SEAT.0 as f32 + 0.5, 1.6, CK_SEAT.1 as f32 + 0.5),
            Tg::ShipExit => vec3(CK_EXIT.0, 1.4, CK_EXIT.1),
            Tg::Sample(s) => {
                let k = match self.me.scene {
                    Scene::Planet(k) => k,
                    _ => 1,
                };
                let (x, z) = SAMPLE_SPOTS[(k as usize).clamp(1, 2) - 1][s];
                vec3(x as f32 + 0.5, 0.9, z as f32 + 0.5)
            }
            Tg::GuildMaster => vec3(F_BROKER.0, 1.9, F_BROKER.1),
            Tg::Pal => vec3(F_PAL_SPOT.0, 0.8, F_PAL_SPOT.1),
            Tg::Bike => vec3(F_BIKE.0, 1.2, F_BIKE.1),
            Tg::Local(i) => {
                let p = local_pos(i, self.t);
                vec3(p.x, 1.9, p.y)
            }
            Tg::FGate => vec3(F_GATE.0 as f32 + 1., 2.6, F_GATE.1 as f32 + 0.5),
            Tg::Cantina => vec3(F_CANTINA.0, 1.8, F_CANTINA.1),
            Tg::CantinaExit => vec3(CT_EXIT.0, 1.4, CT_EXIT.1),
            Tg::CantinaBroker => vec3(CT_BROKER.0, 1.6, CT_BROKER.1),
            Tg::Fugitive(i) => {
                self.w.p.bounty.as_ref().and_then(|b| b.pos.get(i)).map_or(Vec3::ZERO, |p| vec3(p[0], 1.9, p[1]))
            }
            Tg::Board => v(tc(BOARD), 1.9),
            Tg::MSite => self.w.mproject.as_ref().map_or(Vec3::ZERO, |m| v(tc((m.x, m.z)), 1.6)),
            Tg::Dock => vec3(DOCK.0 as f32 + 0.5, 1.6, DOCK.1 as f32 + 0.3),
            Tg::ActBoard => v(tc(ACT_BOARD), 1.5),
            Tg::Pell => vec3(PELL_I.0, 1.6, PELL_I.1),
            Tg::IStall => v(tc(ISTALL), 1.5),
            Tg::IItem(i) => vec3(self.w.iground[i].x, 0.45, self.w.iground[i].z),
            Tg::Built(i) => v(tc((self.w.built[i].x, self.w.built[i].z)), 1.2),
            Tg::Site => self.w.project.as_ref().map_or(Vec3::ZERO, |p| v(tc((p.x, p.z)), 1.6)),
            Tg::Curator | Tg::Hazel => vec3(3.5, 1.3, 1.5),
            Tg::Exhibit(k) => match k {
                0 => vec3(0.5, 1.5, 2.5),
                1 => vec3(6.5, 1.5, 2.5),
                _ => {
                    let p = tc(MUSEUM_PED[k - 2].0);
                    vec3(p.x, 1.2, p.y)
                }
            },
            Tg::Counter => vec3(3.5, 1.3, 1.5),
            Tg::Keeper => vec3(3.5, 1.7, 0.5),
            Tg::Display(i) => v(
                tc(match i {
                    7 => EXTRA_DISPLAY,
                    8 => SPECIAL_DISPLAY,
                    _ => DISPLAYS[i],
                }),
                1.3,
            ),
            Tg::Furn(uid, d) => {
                let c = self
                    .room_furn(match self.me.scene {
                        Scene::Room(b) => b,
                        _ => HOME,
                    })
                    .into_iter()
                    .find(|f| f.uid == uid && f.def == d)
                    .map(|f| fcenter(&f))
                    .unwrap_or_default();
                v(c, if def(d).wall { 1.9 } else { 1.2 })
            }
        }
    }
    pub fn tg_label(&self, tg: Tg) -> String {
        match tg {
            Tg::Contest => "Talk to Mara (monthly survey)".into(),
            Tg::Npc(i) => format!("Talk to {}", self.res(i).name),
            Tg::Door(b) => format!("Enter {}", self.bname(b)),
            Tg::Tree(i) => {
                if self.w.trees[i].fruit {
                    "Shake tree".into()
                } else {
                    "Shake tree (no fruit)".into()
                }
            }
            Tg::Item(i) => format!("Pick up {}", def(self.w.ground[i].def).name),
            Tg::Weed(_) => "Pull weed".into(),
            Tg::Furn(uid, d) => {
                let a = match def(d).act {
                    Act::Sit => "Sit on",
                    Act::Bed => "Lie on",
                    Act::Lamp => "Toggle",
                    Act::Storage => "Open",
                    Act::None => "Look at",
                };
                if uid != 0 && self.me.scene == Scene::Room(self.home()) {
                    format!("{a} {}   [F] Move  [G] Pick up", def(d).name)
                } else {
                    format!("{a} {}", def(d).name)
                }
            }
            Tg::Counter => "Sell items".into(),
            Tg::Keeper => "Talk to Mr. Tumble (seeds, tool upgrades)".into(),
            Tg::Hazel => "Talk to Hazel Quill (civic desk)".into(),
            Tg::Curator => "Donate / identify fossils".into(),
            Tg::Rack => "Browse clothes".into(),
            Tg::Site => self
                .w
                .project
                .as_ref()
                .map_or(String::new(), |p| format!("Contribute to the {}", PROJECTS[p.kind as usize].0)),
            Tg::Board => "Read the town notice board".into(),
            Tg::MSite => self.w.mproject.as_ref().map_or(String::new(), |m| format!("Fund the {}", MPROJECTS[m.idx].0)),
            Tg::Dock => format!("Sail to Gullrock Islet ({FARE} coins)"),
            Tg::ActBoard => "Timed activities board".into(),
            Tg::Pell => "Talk to Captain Pell".into(),
            Tg::IStall => "Sea Glass stall".into(),
            Tg::IItem(i) => format!("Pick up {}", def(self.w.iground[i].def).name),
            Tg::Built(_) => "Sit on the bench".into(),
            Tg::Camper => format!("Talk to {} the {}", VISITORS[self.visitor()].name, VISITORS[self.visitor()].species),
            Tg::CampSign => "Read the campground sign".into(),
            Tg::Kiosk => format!("Initiatives kiosk ({} coupons)", self.w.p.coupons),
            Tg::BountyBoard => "Guild Post bounty board".into(),
            Tg::WoodsDoor => "Open the door to Dustreach".into(),
            Tg::Ship => if self.w.p.bounty.as_ref().is_some_and(|b| b.caught && !b.delivered) {
                "Lock them in the holding pod"
            } else {
                "Board your ship"
            }
            .into(),
            Tg::GuildMaster => "Talk to the guild master".into(),
            Tg::Pal => "Say hello to the sand-hopper".into(),
            Tg::Pilot => "Take the pilot's seat and fly".into(),
            Tg::ShipExit => "Walk down the ramp".into(),
            Tg::Sample(_) => {
                if matches!(self.me.scene, Scene::Planet(1)) { "Chip off a frost crystal" } else { "Pick a glowcap" }
                    .into()
            }
            Tg::Bike => if self.me.riding { "Park the hover-bike" } else { "Ride the hover-bike" }.into(),
            Tg::Local(i) if i == MECHANIC => "Upgrade ship (Dena)".into(),
            Tg::Local(i) => format!("Talk to {}", LOCALS[i].1),
            Tg::FGate => "Gate back to Hollowbrook".into(),
            Tg::Cantina => "Enter the Longhouse Cantina".into(),
            Tg::CantinaExit => "Step back outside".into(),
            Tg::CantinaBroker => "Talk to the Off-world Broker".into(),
            Tg::Fugitive(_) => "Detain".into(),
            Tg::Plant(i) => {
                let p = self.w.garden[i];
                let n = def(FLOWERS[p.kind as usize]).name;
                if p.stage == 2 {
                    format!("Pick the {n}")
                } else {
                    format!(
                        "{n} ({}{})",
                        ["sprout", "bud"][p.stage as usize],
                        if p.watered == self.w.day { ", watered" } else { ", needs water" }
                    )
                }
            }
            Tg::PlantSpot => format!(
                "Plant {}",
                self.w.p.inv.iter().find(|s| SEEDS.contains(&s.def)).map_or("", |s| def(s.def).name)
            ),
            Tg::Surprise => self
                .w
                .surprise
                .as_ref()
                .map_or(String::new(), |sp| format!("Pick up the {}", SURPRISES[sp.kind as usize])),
            Tg::Stairs(to) => match to {
                0 => "Go back to the main floor",
                1 => "Go down to the basement",
                _ => "Go upstairs",
            }
            .into(),
            Tg::Fest => format!("Talk to Hazel about the {}", self.fest().map_or("", |f| FESTS[f].name)),
            Tg::ArtStall => format!("Art stall: {} ({ART_PRICE} coins)", def(art_of_week(self.w.day)).name),
            Tg::Stall => {
                if self.w.p.fair_year == self.yr {
                    "Harvest Fair stall (prize claimed)".into()
                } else {
                    "Claim a Harvest Fair prize".into()
                }
            }
            Tg::Mailbox(o) => {
                if o == self.me.id {
                    let n = self.w.p.mail.iter().filter(|l| !l.read).count();
                    if n > 0 {
                        format!("Check mailbox ({n} new)")
                    } else {
                        "Check mailbox".into()
                    }
                } else {
                    format!("Player {}'s mailbox", o + 1)
                }
            }
            Tg::Exhibit(k) => match k {
                0 => "Look at the fish tank".into(),
                1 => "Look at the bug case".into(),
                _ => format!("Look at the {} pedestal", def(MUSEUM_PED[k - 2].1).name),
            },
            Tg::Display(i) if i >= 7 => {
                let (d, p, sold) = self.display_item(i);
                if sold {
                    "Sold out".into()
                } else {
                    format!("{}Buy {} ({p} coins)", if i == 8 { "Weekly special: " } else { "" }, def(d).name)
                }
            }
            Tg::Display(i) => {
                if i < 5 {
                    let s = self.w.stock[i];
                    if s.sold {
                        "Sold out".into()
                    } else {
                        format!("Buy {} ({} coins)", def(s.def).name, def(s.def).buy)
                    }
                } else {
                    let t = TOOL_DISPLAY[i - 5];
                    format!("Buy {} ({} coins)", def(t).name, def(t).buy)
                }
            }
        }
    }

    // ---- interaction ----
    pub fn interact(&mut self) {
        if self.me.pose != Pose::Stand {
            return self.stand_up(); // E while seated or lying down gets you up (never re-seats you)
        }
        let Some(tg) = self.target() else { return };
        self.snd.push(Snd::Blip);
        match tg {
            Tg::Contest => self.contest_talk(),
            Tg::Npc(i) => self.talk(i),
            Tg::Door(b) => self.door(b),
            Tg::Tree(i) => self.shake_tree(i),
            Tg::Item(i) => {
                let it = self.w.ground[i];
                if self.add(it.def) {
                    self.w.ground.retain(|g| g.uid != it.uid);
                    if def(it.def).k == K::Shell {
                        self.ev(Ev::Shell, 1)
                    }
                }
            }
            Tg::Weed(i) => {
                self.w.weeds.remove(i);
                self.note("You pulled a weed. Tidy!");
                self.ev(Ev::Weed, 1)
            }
            Tg::Furn(uid, d) => self.use_furn(uid, d),
            Tg::Counter => self.me.ui = Ui::Sell(0),
            Tg::Curator => self.donate(),
            Tg::Hazel => self.me.ui = Ui::Civic(0),
            Tg::Rack => self.me.ui = Ui::Rack(0),
            Tg::Stall => self.fair_prize(),
            Tg::Fest => self.fest_talk(),
            Tg::Surprise => self.take_surprise(),
            Tg::Stairs(to) => self.take_stairs(to),
            Tg::Plant(i) => self.pick_flower(i),
            Tg::PlantSpot => {
                let ft = tile_of(self.me.pos + self.me.face * 0.9);
                if let Err(e) = self.plant(ft) {
                    self.note(e)
                }
            }
            Tg::Camper => self.me.ui = Ui::Camp(0),
            Tg::CampSign => {
                let v = &VISITORS[self.visitor()];
                let n = &VISITORS[visitor_of(self.w.day + 1)];
                self.me.ui = Ui::Talk { npc: None, who: "Campground sign".into(), lines: vec![format!("Hollowbrook Campground. Travellers stay one day each, at the campfire from {}:00 to {}:00.", CAMP_HOURS.0, CAMP_HOURS.1), format!("Today: {} the {}. Tomorrow: {} the {}.", v.name, v.species, n.name, n.species), "Each traveller sells one of each item per day and has one favor to ask.".into()], i: 0 }
            }
            Tg::Kiosk => self.me.ui = Ui::Kiosk(0),
            Tg::BountyBoard => self.me.ui = Ui::Bounty(0),
            Tg::WoodsDoor => self.go(Scene::Frontier),
            Tg::Ship => {
                if self.deliver().is_err() {
                    self.board()
                }
            }
            Tg::GuildMaster => self.me.ui = Ui::Forge(0),
            Tg::Pal => self.adopt_pal(),
            Tg::Pilot => self.take_off(),
            Tg::ShipExit => self.disembark(),
            Tg::Sample(s) => {
                if let Scene::Planet(k) = self.me.scene {
                    self.take_sample(k, s)
                }
            }
            Tg::Bike => self.toggle_bike(),
            Tg::Local(i) => self.talk_local(i),
            Tg::FGate => self.go(Scene::Town),
            Tg::Cantina => self.go(Scene::Cantina),
            Tg::CantinaExit => self.go(Scene::Frontier),
            Tg::CantinaBroker => self.me.ui = Ui::Cantina(0),
            Tg::Fugitive(i) => self.detain(i),
            Tg::ArtStall => {
                let r = self.art_buy();
                match r {
                    Ok(n) => {
                        self.snd.push(Snd::Coin);
                        self.note(format!("You bought \"{n}\". The museum's art wing would love it!"))
                    }
                    Err(e) => self.note(e),
                }
            }
            Tg::Site => self.contribute(),
            Tg::Dock => self.me.ui = Ui::Travel(true),
            Tg::Pell => self.pell(),
            Tg::IStall => self.me.ui = Ui::IStall(0),
            Tg::ActBoard => {
                if self.act.is_some() {
                    self.note("An activity is already running.")
                } else {
                    self.me.ui = Ui::Act(0)
                }
            }
            Tg::IItem(i) => {
                let it = self.w.iground[i];
                if self.add(it.def) {
                    self.w.iground.retain(|g| g.uid != it.uid)
                }
            }
            Tg::Board => {
                let lines = self.board_lines();
                self.me.ui = Ui::Talk { npc: None, who: "Town notice board".into(), lines, i: 0 }
            }
            Tg::MSite => self.me.ui = Ui::Fund,
            Tg::Built(i) => {
                let b = self.w.built[i];
                self.me.stand_from = self.me.pos;
                self.me.pos = tc((b.x, b.z));
                self.me.face = vec2(0., 1.);
                self.me.pose = Pose::Sit(0);
                self.note("You sit on the bench. Move or press E to stand up.")
            }
            Tg::Mailbox(o) => {
                if o == self.me.id {
                    self.me.ui = Ui::Mail(0)
                } else {
                    self.note(format!("That's Player {}'s mailbox. Better not peek.", o + 1))
                }
            }
            Tg::Exhibit(k) => {
                let (n, t) = match k {
                    0 => (self.donated_of(K::Fish), DEFS.iter().filter(|d| d.k == K::Fish).count()),
                    1 => (self.donated_of(K::Bug), DEFS.iter().filter(|d| d.k == K::Bug).count()),
                    _ => (self.w.donated.iter().any(|x| x.0 == MUSEUM_PED[k - 2].1) as usize, 1),
                };
                let who = if k >= 2 {
                    self.w
                        .donated
                        .iter()
                        .find(|x| x.0 == MUSEUM_PED[k - 2].1)
                        .map(|x| format!(" (donated by Player {})", x.1 + 1))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                self.note(format!(
                    "{}: {n}/{t} on display.{who}",
                    ["Fish tank", "Bug case"].get(k).copied().unwrap_or(def(MUSEUM_PED[k.max(2) - 2].1).name)
                ))
            }
            Tg::Keeper => self.me.ui = Ui::Keeper(0),
            Tg::Display(i) => {
                if i >= 7 && self.display_item(i).2 {
                    self.note(if i == 8 {
                        "This week's special is sold. A new one arrives next week."
                    } else {
                        "That one's sold out. New stock arrives at 6:00."
                    })
                } else if i < 5 && self.w.stock[i].sold {
                    self.note("That one's sold out. New stock arrives at 6:00.")
                } else if (5..7).contains(&i) && self.w.p.tools.contains(&TOOL_DISPLAY[i - 5]) {
                    self.note("You already own this tool.")
                } else {
                    self.me.ui = Ui::Confirm(i)
                }
            }
        }
    }
    pub(super) fn door(&mut self, b: usize) {
        match BLDS[b].kind {
            BK::Home(_) => self.go(Scene::Room(b)),
            BK::Shop => {
                if self.shop_is_open() {
                    self.go(Scene::Room(b))
                } else {
                    let (a, c) = self.shop_hours();
                    self.note(format!("The General Store is closed. Hours: {a}:00-{c}:00."))
                }
            }
            BK::Res(r) => match self.loc(r) {
                Loc::Home => self.go(Scene::Room(b)),
                Loc::Asleep => self.note(format!("It's late. {} is asleep.", self.res(r).name)),
                Loc::Out => self.note(format!("{} isn't home right now.", self.res(r).name)),
            },
            BK::Civic => self.go(Scene::Room(b)),
            BK::Museum => self.go(Scene::Room(b)),
        }
    }
    pub fn talk(&mut self, i: usize) {
        if self.social_talk(i) {
            return;
        }
        let r = &self.res(i);
        let pers = r.pers;
        let hr = self.hr;
        let slot = if !(5..21).contains(&hr) {
            3
        } else if hr < 12 {
            0
        } else if hr < 17 {
            1
        } else {
            2
        };
        let fruit = def(self.w.native).name.to_lowercase();
        let fruits = plural(&fruit);
        let name = self.w.p.name.clone();
        let fill = |s: &str, item: &str| {
            s.replace("{p}", &name).replace("{fs}", &fruits).replace("{f}", &fruit).replace("{i}", item)
        };
        let indoor = self.me.scene != Scene::Town && !self.visit.is_some_and(|v| v.0 == i);
        let visiting = self.visit.is_some_and(|v| v.0 == i);
        let mut lines = vec![fill(
            if visiting {
                VISIT[pers]
            } else if indoor {
                HOMEL[pers]
            } else {
                GREET[pers][slot]
            },
            "",
        )];
        if self.w.moving.get(i).is_some_and(|&m| m != 0) && !visiting {
            lines.push("I'm planning to move away soon... unless someone gives me a reason to stay.".into())
        }
        if let Some(p) = &self.w.project {
            if p.by == i {
                lines.push(format!("How's our {} coming along? {}", PROJECTS[p.kind as usize].0, Game::remaining(p)))
            }
        }
        if let Some(b) = self.w.built.iter().rev().find(|b| self.w.day - b.day <= 2) {
            lines.push(format!(
                "Have you seen the new {}? The town feels {} already!",
                PROJECTS[b.kind as usize].0,
                Game::live_word(self.liveliness()).to_lowercase()
            ))
        }
        if self.fair() {
            lines.push("The Harvest Fair is on! There's a prize stall by the plaza.".into())
        }
        if let Some(f) = self.fest() {
            lines.push(FESTS[f].chat.into())
        }
        match weather(self.w.day, self.mo) {
            Wx::Rain if !indoor => lines.push(fill(RAIN[pers], "")),
            Wx::Snow if !indoor => lines.push(fill(SNOW[pers], "")),
            _ => {}
        }
        let ch = self.w.roster[i];
        if (self.mo, self.dom) == BDAYS[ch] {
            lines.push(format!("Guess what, {name}? It's my birthday today!"))
        }
        if self.w.p.lost_owner == Some(i) && self.remove_one(150) {
            self.w.p.lost_owner = None;
            self.w.p.npcs[i].friend += 4;
            self.w.p.money += RETURN_COINS;
            self.npcs[i].happy = 2.5;
            self.snd.push(Snd::Coin);
            lines.push(format!(
                "My scarf! I looked everywhere for it! Thank you so much, {name}. Please take {RETURN_COINS} coins!"
            ))
        }
        if let Some((from, to)) = self.w.p.parcel {
            if to == i && self.w.p.inv.iter().any(|s| s.def == 86) {
                self.remove_one(86);
                self.w.p.parcel = None;
                self.w.p.money += 300;
                self.w.p.npcs[i].friend += 2;
                self.w.p.npcs[from].friend += 2;
                self.npcs[i].happy = 2.;
                self.snd.push(Snd::Coin);
                lines.push(format!(
                    "A parcel from {}? How thoughtful! Here, take 300 coins for your trouble.",
                    self.res(from).name
                ));
            }
        }
        if self.w.p.npcs[i].req == Some(86) {
            if self.w.p.parcel.is_none() && self.w.p.inv.len() < INV_CAP {
                let to = (i + 1 + gen_range(0, 3)) % 4;
                self.add(86);
                self.w.p.parcel = Some((i, to));
                self.w.p.npcs[i].req = None;
                lines.push(format!(
                    "Could you take this parcel to {} for me? I'd really appreciate it!",
                    self.res(to).name
                ))
            } else {
                self.w.p.npcs[i].req = None
            }
        }
        if let Some(q) = self.w.p.npcs[i].req.filter(|&q| q != 86) {
            let item = def(q).name;
            if self.remove_one(q) {
                lines.push(fill(THANKS[pers], item));
                let s = &mut self.w.p.npcs[i];
                s.req = None;
                s.friend += 3;
                let fr = s.friend;
                self.npcs[i].happy = 2.5;
                let hp = if indoor { res_spot() } else { self.npcs[i].pos };
                for k in 0..6 {
                    self.hearts.push((vec3(hp.x + (k as f32 - 2.5) * 0.12, 1.3, hp.y), -(k as f32) * 0.12))
                }
                if fr >= 6 && self.w.p.inv.len() < INV_CAP {
                    let f = FURN_IDS[gen_range(0, 8)];
                    self.add_furn(f);
                    self.note(format!("{} gave you a {}!", r.name, def(f).name))
                } else {
                    let m = def(q).sell * 2;
                    self.w.p.money += m;
                    self.note(format!("{} gave you {m} coins.", r.name))
                }
                self.snd.push(Snd::Coin);
            } else {
                lines.push(fill(ASK[pers], item))
            }
        } else if self.w.p.npcs[i].talked {
            lines.push(AGAIN[pers].to_string())
        } else {
            let mut k = gen_range(0, 4);
            if k == self.npcs[i].last_chat {
                k = (k + 1) % 4
            }
            self.npcs[i].last_chat = k;
            lines.push(fill(CHAT[pers][k], ""));
            let fr = self.w.p.npcs[i].friend;
            lines.push(fill(TIER_LINES[pers][tier(fr)], ""));
            let mut ctx = self.ctx_lines(i);
            for _ in 0..2 {
                if ctx.is_empty() {
                    break;
                }
                let c = ctx.remove(gen_range(0, ctx.len()));
                lines.push(fill(&c, ""))
            }
            let ch = self.w.roster[i];
            if tier(fr) == 4 && !self.w.p.keepsakes.contains(&(ch as u8)) && self.add_furn(KEEPSAKES[ch]) {
                self.w.p.keepsakes.push(ch as u8);
                self.npcs[i].happy = 2.5;
                self.snd.push(Snd::Coin);
                lines.push(format!(
                    "{name}... you're my best friend. I want you to have this portrait of me. Hang it somewhere nice!"
                ))
            }
        }
        if !self.w.p.npcs[i].talked {
            self.w.p.npcs[i].talked = true;
            self.w.p.npcs[i].friend += 1;
            self.ev(Ev::Talk, 1)
        } // temporary prototype relationship rule
        self.me.ui = Ui::Talk { npc: Some(i), who: format!("{} the {}", r.name, r.species), lines, i: 0 };
    }
    /// Contextual line candidates for resident slot `i` (season, the player's progress, town state, other residents, hobby).
    pub fn ctx_lines(&self, i: usize) -> Vec<String> {
        let pers = self.res(i).pers;
        let ch = self.w.roster[i];
        let p = &self.w.p;
        let mut v: Vec<String> = vec![];
        let season = match self.mo {
            3..=5 => 0,
            6..=8 => 1,
            9..=11 => 2,
            _ => 3,
        };
        v.push(SEASON_LINES[pers][season].into());
        v.push(CHAR_LINES[ch][gen_range(0, 3)].into());
        v.push(format!("Did you know I love {}? It's my favorite thing to do.", HOBBY[ch]));
        let o = (i + 1 + gen_range(0, 3)) % 4;
        v.push(GOSSIP[pers][gen_range(0, 2)].replace("{o}", self.res(o).name));
        if p.home > 0 {
            v.push(
                [
                    "Your house got bigger! You'll have to show me around!",
                    "Bigger house, huh? More room for clutter.",
                    "Your home grew, like a mushroom after rain.",
                    "Expanded your house? That's a power move!",
                ][pers]
                    .into(),
            )
        }
        if p.debt > 0 {
            v.push(
                [
                    "Paying off your home bit by bit? You've got this!",
                    "A balance on your house? Pay it off when you can. No rush. I'm not your mother.",
                    "Coins come and go like tides. Your home stays.",
                    "Chipping away at that home balance? Reps, {p}! Reps!",
                ][pers]
                    .into(),
            )
        }
        if self.w.donated.len() >= 10 {
            v.push(format!("The museum has {} exhibits now. The professor must be thrilled!", self.w.donated.len()))
        }
        if self.camp_open() {
            let vv = &VISITORS[self.visitor()];
            v.push(format!("Have you met {} at the campground? A {} with lots of stories!", vv.name, vv.species))
        }
        if let Some(pol) = self.pol() {
            v.push(format!("With {} in effect, the town feels different. I'm still deciding if I like it!", pol.name))
        }
        if self.w.built.len() >= 3 {
            v.push("Our town has so many new things now. It feels like home.".into())
        }
        if let Some(sp) = &self.w.surprise {
            let where_ = if sp.x < 20 { "west" } else { "east" };
            v.push(match sp.kind {
                0 => format!("I saw something glinting on the beach, over on the {where_} side."),
                1 => format!("Someone dropped a scarf somewhere on the {where_} side of town. I hope they find it!"),
                _ => format!("A balloon came down somewhere on the {where_} side of town! I wonder what it carried."),
            })
        }
        v
    }
    #[cfg(test)]
    pub fn shake_tree_pub(&mut self, ti: usize) {
        self.shake_tree(ti)
    }
    #[cfg(test)]
    pub fn reel_pub(&mut self) {
        self.reel()
    }
    pub(super) fn shake_tree(&mut self, ti: usize) {
        self.shake.push((ti, 0.6));
        self.snd.push(Snd::Rustle);
        let tr = self.w.trees[ti];
        if !self.w.branches.contains(&(tr.x, tr.z)) {
            if let Some((x, z)) = [(0, 1), (1, 0), (-1, 0), (0, -1), (1, 1), (-1, 1)]
                .iter()
                .map(|&(dx, dz)| (tr.x + dx, tr.z + dz))
                .find(|&(x, z)| self.walk_tile(x, z) && !self.item_at(x, z))
            {
                self.w.branches.push((tr.x, tr.z));
                self.drop_item(80, x, z)
            }
        }
        if !tr.fruit {
            return self.note("Rustle rustle... no fruit right now.");
        }
        self.w.trees[ti].fruit = false;
        self.w.trees[ti].shaken = self.w.day;
        let mut n = 0;
        for (dx, dz) in [(0, 1), (-1, 1), (1, 1), (-1, 0), (1, 0), (0, -1), (-1, -1), (1, -1)] {
            if n == 3 {
                break;
            }
            let (x, z) = (tr.x + dx, tr.z + dz);
            if self.walk_tile(x, z) && !self.item_at(x, z) {
                self.drop_item(self.w.native, x, z);
                n += 1
            }
        }
        let nm = plural(&def(self.w.native).name.to_lowercase());
        self.note(format!("{n} {nm} fell from the tree!"));
        if n > 0 {
            self.ev(Ev::Fruit, 1)
        }
    }
}
