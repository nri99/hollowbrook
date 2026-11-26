use crate::clock::day_id;
use crate::data::*;
use crate::events::*;
use crate::save;
use crate::world::*;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use macroquad::prelude::*;

// Shared test helpers. Tests live in src/tests/<topic>.rs.
fn at(d: u32, h: u32, m: u32, s: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 11, d).unwrap().and_hms_opt(h, m, s).unwrap()
} // Nov 10 2026 = Tue
fn tmp(n: &str) -> String {
    std::env::temp_dir().join(format!("hb3_{n}.json")).to_string_lossy().into()
}
fn g0(t: NaiveDateTime) -> Game {
    let mut g = Game::new("T".into(), t);
    g.hr = 12;
    g
}
fn talk_daily(g: &mut Game) -> i32 {
    g.w.p.npcs[0].req = None;
    g.talk(0);
    g.me.ui = Ui::Play;
    g.w.p.npcs[0].friend
}
fn reload(g: &Game, n: &str) -> Game {
    let p = tmp(n);
    save::save(&g.persist(), &p);
    Game::from_world(save::load(&p).unwrap(), town())
}
fn home(g: &mut Game) {
    g.swap(Scene::Room(HOME))
}

fn two(t: NaiveDateTime) -> Game {
    let mut g = g0(t);
    g.join("P2");
    for n in &mut g.npcs {
        n.inside = true
    }
    g
}

mod activities;
mod daily;
mod economy;
mod frontier;
mod furniture;
mod islet;
mod multiplayer;
mod museum;
mod projects;
mod quality;
mod save_safety;
mod world;
