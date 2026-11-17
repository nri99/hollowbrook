//! Original M3 activities. All balance values and schedules here are provisional.
use crate::data::*;
use crate::world::*;
use chrono::{Datelike, NaiveDate};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use serde::{Deserialize, Serialize};

pub const CONTEST_SPOT: (f32, f32) = (13.5, 17.5);
pub const CONTEST_NAMES: [&str; 2] = ["River Measure", "Wing & Leaf Survey"];
pub const CONTEST_HOST: &str = "Mara Vale";
pub const CONTEST_PRIZES: [u32; 4] = [120, 350, 950, 1750];
pub const CONTEST_PLAQUES: [u16; 2] = [170, 171];

/// One player's registration, best entry, and prize receipt for a contest day.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ContestEntry {
    pub day: i64, pub kind: usize, pub best: u32, pub specimen: Option<u16>, pub size_mm: u16, pub claimed: bool,
}
/// Per-player invitation. State: 0 offered, 1 accepted, 2 completed, 3 missed, 4 declined.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SocialInvite { pub day: i64, pub slot: usize, pub character: usize, pub kind: u8, pub hour: u32, pub state: u8 }
/// Second Wednesday (fish), fourth Sunday (bugs), every month, including winter.
pub fn contest_on(day: i64) -> Option<usize> {
    let d = NaiveDate::from_num_days_from_ce_opt(i32::try_from(day).ok()?)?;
    match (d.weekday().num_days_from_monday(), (d.day() - 1) / 7) { (2, 1) => Some(0), (6, 3) => Some(1), _ => None }
}
pub fn catch_base(d: u16) -> u16 { (def(d).size * if def(d).k == K::Fish { 500. } else { 100. }).round().max(1.) as u16 }
pub fn catch_score(d: u16, mm: u16) -> u32 { mm as u32 * 80 / catch_base(d) as u32 + (1 + (def(d).sell / 400).min(4)) * 7 }
pub fn contest_tier(score: u32) -> usize { [68, 93, 116].iter().filter(|&&s| score >= s).count() }

impl Game {
    /// Every new fish/bug gets its own measured pocket slot. Legacy stacks remain intact.
    pub fn add_catch(&mut self, d: u16) -> bool {
        if self.w.p.inv.len() >= INV_CAP { self.note("Your pockets are full; the catch was released."); return false }
        let mm = (catch_base(d) as u32 * gen_range(65u32, 136) / 100).max(1) as u16;
        self.w.p.inv.push(Slot { def: d, qty: 1, uid: 0, size_mm: mm });
        if !self.w.p.seen.contains(&d) { self.w.p.seen.push(d) }
        self.note(format!("Caught {}: {mm} mm!", def(d).name)); true
    }
    pub fn contest_kind(&self) -> Option<usize> { contest_on(self.w.day) }
    pub fn contest_host_open(&self) -> bool { self.contest_kind().is_some() && (9..22).contains(&self.hr) }
    pub fn contest_entry(&self) -> Option<&ContestEntry> { self.w.p.contests.iter().find(|e| e.day == self.w.day && Some(e.kind) == self.contest_kind()) }
    pub fn contest_slots(&self) -> Vec<usize> {
        let Some(k) = self.contest_kind() else { return vec![] };
        self.w.p.inv.iter().enumerate().filter(|(_, s)| s.qty > 0 && s.size_mm > 0 && def(s.def).k == if k == 0 { K::Fish } else { K::Bug }).map(|(i, _)| i).collect()
    }
    pub fn contest_talk(&mut self) {
        let Some(k) = self.contest_kind().filter(|_| self.contest_host_open()) else { return };
        let lines = if self.hr >= 17 {
            match self.contest_claim() { Ok(s) => vec![s], Err(e) => vec![e.into()] }
        } else if self.contest_entry().is_none() {
            self.w.p.contests.push(ContestEntry { day: self.w.day, kind: k, best: 0, specimen: None, size_mm: 0, claimed: false });
            vec![format!("You're registered for {}! Entries close at 17:00.", CONTEST_NAMES[k]), "Bring measured catches. Size relative to species and rarity earn points.".into(), "Talk again to enter one catch. An improved entry is released; others return to you.".into(), "Prizes from 17:00 to 22:00. Missed prizes arrive by post next morning.".into()]
        } else { self.me.ui = Ui::Contest(0); return };
        self.me.ui = Ui::Talk { npc: None, who: CONTEST_HOST.into(), lines, i: 0 };
    }
    pub fn contest_submit(&mut self, slot: usize) -> Result<u32, &'static str> {
        if !self.contest_host_open() || self.hr >= 17 { return Err("Entries are open 09:00-17:00 on contest days.") }
        let Some(e) = self.contest_entry() else { return Err("Talk to Mara to register first.") };
        if !self.contest_slots().contains(&slot) { return Err("Bring a measured catch of today's kind.") }
        let s = self.w.p.inv[slot]; let score = catch_score(s.def, s.size_mm);
        if score <= e.best { return Err("That does not beat your best; your catch was returned.") }
        let day = self.w.day; let k = self.contest_kind().unwrap();
        let e = self.w.p.contests.iter_mut().find(|e| e.day == day && e.kind == k).unwrap();
        e.best = score; e.specimen = Some(s.def); e.size_mm = s.size_mm;
        self.w.p.inv[slot].qty -= 1; if self.w.p.inv[slot].qty == 0 { self.w.p.inv.remove(slot); }
        self.snd.push(Snd::Blip); Ok(score)
    }
    pub fn contest_claim(&mut self) -> Result<String, &'static str> {
        if !self.contest_host_open() || self.hr < 17 { return Err("Prizes are available after entries close at 17:00.") }
        let day = self.w.day;
        let Some(e) = self.w.p.contests.iter_mut().find(|e| e.day == day) else { return Err("You did not register for today's contest.") };
        if e.claimed { return Err("You already received this contest's prize.") }
        if e.best == 0 { return Err("No entry was recorded. Come enter next month's survey!") }
        e.claimed = true;
        let tier = contest_tier(e.best); let item = (tier == 3).then_some(CONTEST_PLAQUES[e.kind]); let coins = CONTEST_PRIZES[tier];
        self.w.p.money += coins;
        if let Some(d) = item { self.event_gift(CONTEST_HOST, "Your survey plaque.", d); }
        self.snd.push(Snd::Coin);
        Ok(format!("{} prize: {coins} coins{}!", ["Field", "Copper", "Silver", "Laurel"][tier], if item.is_some() { " and a plaque" } else { "" }))
    }
    /// Complete missed contest payouts without discarding them when a day is skipped/offline.
    pub fn settle_contests(&mut self) {
        let p = &mut self.w.p;
        for e in &mut p.contests {
            if e.day < self.w.day && !e.claimed && e.best > 0 {
                let tier = contest_tier(e.best); e.claimed = true;
                p.mail.push(Letter { from: CONTEST_HOST.into(), text: format!("Your {} prize from {}.", CONTEST_NAMES[e.kind], e.day), coins: CONTEST_PRIZES[tier], item: (tier == 3).then_some(CONTEST_PLAQUES[e.kind]), read: false, claimed: false });
            }
        }
    }
    pub fn event_gift(&mut self, who: &str, text: &str, d: u16) {
        let ok = if self.w.p.inv.len() < INV_CAP { if def(d).k == K::Furn { self.add_furn(d) } else { self.add(d) } } else { false };
        if !ok { self.w.p.mail.push(Letter { from: who.into(), text: text.into(), coins: 0, item: Some(d), read: false, claimed: false }); self.note("Your gift is safe in your mailbox."); }
    }
    pub fn contest_notice(&self) -> String {
        if let Some(k) = self.contest_kind() { format!("Today: {}. Plaza entries 09-17; prizes 17-22.", CONTEST_NAMES[k]) }
        else { "Fish: 2nd Wed; bugs: 4th Sun. Plaza entries 09-17; prizes 17-22.".into() }
    }
    /// A nearby friend initiates an offer without interrupting movement or existing dialogue.
    pub fn maybe_social_offer(&mut self) {
        if self.me.scene != Scene::Town || self.me.ui != Ui::Play || !(9..17).contains(&self.hr) || self.w.p.social_day == self.w.day { return }
        // The other activity day is reserved for hide-and-seek; one quiet day in each cycle.
        let kind = (self.w.day + self.me.id as i64).rem_euclid(3) as u8;
        if kind != 0 || self.w.p.invitation.as_ref().is_some_and(|v| v.day == self.w.day) { return }
        let slot = (0..4).filter(|&i| self.w.p.npcs[i].friend >= 5 && !self.npcs[i].inside && self.npcs[i].pos.distance(self.me.pos) < 2.8)
            .min_by(|&a, &b| self.npcs[a].pos.distance(self.me.pos).total_cmp(&self.npcs[b].pos.distance(self.me.pos)));
        if let Some(slot) = slot {
            self.w.p.social_day = self.w.day;
            self.w.p.invitation = Some(SocialInvite { day: self.w.day, slot, character: self.w.roster[slot], kind, hour: self.hr + 1, state: 0 });
            self.note(format!("{} would like to invite you over. Talk to them!", self.res(slot).name));
        }
    }
    pub fn refresh_social(&mut self) {
        if let Some(v) = self.w.p.invitation.clone() {
            if v.day < self.w.day {
                if v.kind == 0 && v.state == 1 { self.w.p.missed_visit = Some((v.slot, v.character, v.day)); }
                self.w.p.invitation = None;
            }
        }
    }
    pub fn social_tick(&mut self) {
        self.refresh_social();
        if let Some(v) = self.w.p.invitation.clone() {
            if self.w.roster.get(v.slot) != Some(&v.character) { self.w.p.invitation = None; }
            else if v.kind == 0 && v.state == 1 && self.hr > v.hour {
                self.w.p.missed_visit = Some((v.slot, v.character, v.day)); self.w.p.invitation.as_mut().unwrap().state = 3;
            }
            else if v.state == 0 && self.hr > v.hour { self.w.p.invitation.as_mut().unwrap().state = 4; }
        }
        if self.w.p.missed_visit.is_some_and(|(s, c, _)| self.w.roster.get(s) != Some(&c)) { self.w.p.missed_visit = None; }
        self.maybe_social_offer();
    }
    pub fn invited_home(&self, slot: usize) -> bool {
        std::iter::once(&self.w.p).chain(self.w.others.iter()).any(|p| p.invitation.as_ref().is_some_and(|v|
            v.day == self.w.day && v.kind == 0 && v.slot == slot && self.w.roster.get(slot) == Some(&v.character) && (v.state == 1 || v.state == 2) && self.hr == v.hour))
    }
    pub fn social_talk(&mut self, slot: usize) -> bool {
        if let Some((s, c, day)) = self.w.p.missed_visit {
            if s == slot && self.w.roster[slot] == c && day < self.w.day {
                self.w.p.missed_visit = None;
                self.me.ui = Ui::Talk { npc: Some(slot), who: self.res(slot).name.into(), lines: vec!["I saved a chair for our visit. No worries; we can try another day.".into()], i: 0 }; return true
            }
        }
        if self.w.p.invitation.as_ref().is_some_and(|v| v.slot == slot && v.day == self.w.day && self.w.roster[slot] == v.character && v.state == 0) {
            self.me.ui = Ui::Invitation; return true
        }
        false
    }
    pub fn accept_social(&mut self) -> Result<(), &'static str> {
        let Some(v) = self.w.p.invitation.as_ref() else { return Err("There is no invitation waiting.") };
        if v.day != self.w.day || v.state != 0 || self.hr > v.hour || self.w.roster.get(v.slot) != Some(&v.character) { return Err("That invitation has expired.") }
        if v.kind != 0 { return Err("That activity is not available.") }
        let hour = v.hour; let name = self.res(v.slot).name;
        self.w.p.invitation.as_mut().unwrap().state = 1;
        self.note(format!("Visit {name} at home today between {hour}:00 and {}:00.", hour + 1)); Ok(())
    }
    pub fn visit_arrival(&mut self) {
        let Some(v) = self.w.p.invitation.clone() else { return };
        if v.kind != 0 || v.state != 1 || v.day != self.w.day || self.hr != v.hour || self.w.roster.get(v.slot) != Some(&v.character) || self.me.scene != Scene::Room(HOUSES[v.slot]) { return }
        self.w.p.invitation.as_mut().unwrap().state = 2; self.w.p.npcs[v.slot].friend += 4;
        let name = self.res(v.slot).name;
        self.event_gift(name, "Thank you for coming to visit!", FLOWERS[v.slot]);
        self.note(format!("{}: You made it! Friendship +4, and a flower for you.", name)); self.npcs[v.slot].happy = 2.;
    }
    pub fn social_notice(&self) -> Option<String> {
        self.w.p.invitation.as_ref().filter(|v| v.day == self.w.day && v.kind == 0 && v.state <= 1).map(|v|
            format!("{} invites you home {}:00-{}:00 today{}.", self.res(v.slot).name, v.hour, v.hour + 1, if v.state == 0 { " (talk to accept)" } else { " (accepted)" }))
    }
}
