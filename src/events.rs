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
        if let Some(k) = self.contest_kind() { format!("Today: {}. Mara at plaza 09-22; entries 09-17, prizes after 17.", CONTEST_NAMES[k]) }
        else { "Monthly surveys: fish on 2nd Wednesday, bugs on 4th Sunday; entries 09-17, prizes 17-22.".into() }
    }
}
