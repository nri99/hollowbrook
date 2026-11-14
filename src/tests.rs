use crate::clock::day_id;
use crate::data::*;
use crate::save;
use crate::world::*;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use macroquad::prelude::*;

fn at(d: u32, h: u32, m: u32, s: u32) -> NaiveDateTime { NaiveDate::from_ymd_opt(2026, 11, d).unwrap().and_hms_opt(h, m, s).unwrap() } // Nov 10 2026 = Tue
fn tmp(n: &str) -> String { std::env::temp_dir().join(format!("hb3_{n}.json")).to_string_lossy().into() }
fn g0(t: NaiveDateTime) -> Game { let mut g = Game::new("T".into(), t); g.hr = 12; g }
fn talk_daily(g: &mut Game) -> i32 { g.w.npcs[0].req = None; g.talk(0); g.ui = Ui::Play; g.w.npcs[0].friend }
fn reload(g: &Game, n: &str) -> Game { let p = tmp(n); save::save(&g.persist(), &p); Game::from_world(save::load(&p).unwrap(), town()) }
fn home(g: &mut Game) { g.swap(Scene::Room(HOME)) }

// ---- clock / daily refresh (carried over) ----
#[test] fn day_identity() { assert_eq!(at(10, 0, 0, 0).weekday(), chrono::Weekday::Tue); assert_eq!(day_id(at(10, 23, 59, 0)), day_id(at(11, 0, 1, 0))); assert_ne!(day_id(at(11, 5, 59, 0)), day_id(at(11, 6, 0, 0))) }
#[test] fn midnight_is_not_reset() { let mut g = g0(at(10, 23, 59, 50)); let f = talk_daily(&mut g); assert!(!g.catch_up(at(11, 0, 0, 10))); assert!(g.w.npcs[0].talked); assert_eq!(talk_daily(&mut g), f); assert!(g.log.is_empty()) }
#[test] fn six_am_refresh_once_and_stock_vs_hours() {
    let mut g = g0(at(11, 5, 59, 50)); talk_daily(&mut g); g.w.stock[0].sold = true;
    assert!(!g.catch_up(at(11, 5, 59, 59))); assert!(g.w.stock[0].sold);
    assert!(g.catch_up(at(11, 6, 0, 10))); assert_eq!(g.log.len(), 1); assert!(!g.w.npcs[0].talked); assert!(g.w.stock.iter().all(|s| !s.sold));
    assert!(!shop_open(6) && shop_open(8)); for i in 0..200 { assert!(!g.catch_up(at(11, 6, 1, i % 60))) } assert_eq!(g.log.len(), 1);
}
#[test] fn reload_same_day_keeps_daily_state() { let mut g = g0(at(11, 6, 30, 0)); talk_daily(&mut g); g.w.stock[2].sold = true; let mut g2 = reload(&g, "same"); assert!(!g2.catch_up(at(11, 7, 0, 0))); assert!(g2.w.npcs[0].talked && g2.w.stock[2].sold); assert_eq!(g2.w, g.persist()) }
#[test] fn closed_three_days_regrows_once() {
    let mut g = g0(at(11, 5, 0, 0)); g.w.trees[0].fruit = false; g.w.trees[0].shaken = g.w.day; g.w.inv = vec![Slot { def: 20, qty: 3, uid: 0 }]; g.w.money = 4321;
    let mut g2 = reload(&g, "three"); assert!(g2.catch_up(at(14, 5, 0, 0))); assert_eq!(g2.log.len(), 1); assert!(g2.log[0].contains("(3d)")); assert!(g2.w.trees[0].fruit);
    assert_eq!((g2.w.money, g2.w.inv.clone()), (4321, vec![Slot { def: 20, qty: 3, uid: 0 }])); assert!(!g2.catch_up(at(14, 5, 30, 0)));
}
#[test] fn fruit_not_regrown_early() { let mut g = g0(at(11, 7, 0, 0)); g.w.trees[0].fruit = false; g.w.trees[0].shaken = g.w.day; assert!(g.catch_up(at(13, 7, 0, 0))); assert!(!g.w.trees[0].fruit) }
#[test] fn clock_rollback() { let mut g = g0(at(11, 7, 0, 0)); let d = g.w.day; assert!(!g.catch_up(at(9, 7, 0, 0))); assert_eq!(g.w.day, d) }

// ---- economy ----
#[test] fn shop_rules() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.money = 10000; let p = def(g.w.stock[0].def).buy;
    assert!(g.buy(0).is_ok()); assert_eq!(g.w.money, 10000 - p); assert!(g.w.stock[0].sold); let uid = g.w.inv[0].uid; assert!(uid != 0 && g.w.inv[0].def == g.w.stock[0].def);
    let m = g.w.money; assert_eq!(g.buy(0), Err("Sold out!")); assert_eq!(g.w.money, m); assert_eq!(g.w.inv.len(), 1); // repeated input: no duplicate
    g.w.money = 0; assert_eq!(g.buy(1), Err("Not enough coins.")); assert!(!g.w.stock[1].sold); assert_eq!(g.w.inv.len(), 1);
    g.w.money = 10000; while g.w.inv.len() < INV_CAP { g.w.inv.push(Slot { def: 40, qty: 1, uid: 0 }) } assert_eq!(g.buy(2), Err("Your pockets are full.")); assert_eq!(g.w.money, 10000); assert!(!g.w.stock[2].sold);
    g.ui = Ui::Confirm(3); let before = g.w.clone(); g.update(&Inp::keys(&[KeyCode::Escape], &[KeyCode::Escape]), 0.016, at(11, 12, 0, 1)); assert_eq!(g.ui, Ui::Play); assert_eq!(g.w.stock, before.stock); assert_eq!(g.w.money, before.money); // cancel
    assert!(g.buy(5).is_ok()); assert!(g.w.tools.contains(&62)); assert!(g.buy(5).is_err());
}
#[test] fn sold_out_persists_until_six() { let mut g = g0(at(11, 12, 0, 0)); g.w.money = 9999; g.buy(1).unwrap(); let mut g2 = reload(&g, "sold"); assert!(!g2.catch_up(at(12, 5, 59, 0))); assert!(g2.w.stock[1].sold); assert!(g2.catch_up(at(12, 6, 0, 1))); assert!(g2.w.stock.iter().all(|s| !s.sold)) }
#[test] fn gather_fruit_and_sell() {
    let mut g = g0(at(11, 12, 0, 0)); let ti = g.w.trees.iter().position(|t| g.walk_tile(t.x, t.z + 1)).unwrap(); let t = g.w.trees[ti];
    g.pos = vec2(t.x as f32 + 0.5, t.z as f32 + 1.5); g.face = vec2(0., -1.); for n in &mut g.npcs { n.inside = true }
    assert_eq!(g.target(), Some(Tg::Tree(ti))); g.interact(); assert!(!g.w.trees[ti].fruit);
    let drops: Vec<GItem> = g.w.ground.iter().filter(|i| i.def == g.w.native).copied().collect(); assert!(!drops.is_empty());
    for d in &drops { g.pos = vec2(d.x, d.z + 0.6); if !g.free(g.pos) { g.pos = vec2(d.x + 0.6, d.z) ; g.face = vec2(-1., 0.) } else { g.face = vec2(0., -1.) } if let Some(Tg::Item(_)) = g.target() { g.interact() } }
    let got: u16 = g.w.inv.iter().filter(|s| s.def == g.w.native).map(|s| s.qty).sum(); assert!(got >= 1);
    let m = g.w.money; g.sell_all(); assert_eq!(g.w.money, m + 100 * got as u32); assert!(g.w.inv.is_empty());
}
#[test] fn digging_holes_and_sites() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.tools.push(62); g.w.equip = Some(62); let site = g.w.digs[0];
    g.pos = tc((site.0, site.1 + 1)); g.face = vec2(0., -1.); if !g.free(g.pos) { g.pos = tc((site.0 + 1, site.1)); g.face = vec2(-1., 0.) }
    g.use_tool(); assert!(g.w.holes.contains(&site)); assert!(g.w.inv.iter().any(|s| matches!(s.def, 45 | 46))); assert!(!g.walk_tile(site.0, site.1));
    g.use_tool(); assert!(!g.w.holes.contains(&site)); assert!(g.walk_tile(site.0, site.1));
}

// ---- furniture ----
#[test] fn furniture_buy_place_move_rotate_pick_no_dup() {
    let mut g = g0(at(11, 12, 0, 0)); home(&mut g); g.add_furn(51); let uid = g.w.inv[0].uid;
    g.start_place(uid); g.placing.as_mut().unwrap().f.x = 0; g.placing.as_mut().unwrap().f.z = 0; g.place_confirm().unwrap();
    assert!(g.w.inv.is_empty()); assert_eq!(g.w.room.len(), 1); assert_eq!(g.w.room[0].uid, uid);
    g.start_move(uid); assert!(g.w.room.is_empty()); assert_eq!(g.persist().room.len(), 1); // still owned while moving
    { let p = g.placing.as_mut().unwrap(); p.f.rot = 1; p.f.x = 0; p.f.z = 0; } g.place_confirm().unwrap();
    assert!(g.room_block(HOME, 0, 1, 0) && !g.room_block(HOME, 1, 0, 0)); // rotated footprint covers (0,0),(0,1)
    g.start_move(uid); g.place_cancel(); assert_eq!(g.w.room.len(), 1); assert_eq!(g.w.room[0].rot, 1);
    g.pick_furn(uid); assert!(g.w.room.is_empty()); assert_eq!(g.w.inv.iter().filter(|s| s.uid == uid).count(), 1);
    let g2 = reload(&g, "furn"); assert_eq!(g2.w.inv, g.w.inv);
}
#[test] fn placement_rejections() {
    let mut g = g0(at(11, 12, 0, 0)); home(&mut g); let mk = |d: u16, x: i32, z: i32, rot: u8| Furn { uid: 99, def: d, x, z, rot, on: false };
    assert_eq!(g.place_ok(&mk(50, 2, 3, 0)), Err("Keep the doorway clear.")); assert_eq!(g.place_ok(&mk(51, 3, 0, 0)), Err("It doesn't fit there."));
    g.ipos = vec2(2.5, 2.5); assert_eq!(g.place_ok(&mk(50, 2, 2, 0)), Err("You're standing there.")); g.ipos = vec2(2.5, 3.5); g.w.room.push(Furn { uid: 5, def: 52, x: 0, z: 0, rot: 0, on: false });
    assert_eq!(g.place_ok(&mk(51, 0, 0, 0)), Err("Something is in the way.")); assert_eq!(g.place_ok(&mk(57, 1, 0, 0)), Err("Wall items go on the back wall.")); assert!(g.place_ok(&mk(57, 1, -1, 0)).is_ok());
    g.ipos = vec2(3.5, 0.5); g.w.room.push(Furn { uid: 6, def: 50, x: 3, z: 1, rot: 0, on: false }); assert_eq!(g.place_ok(&mk(50, 2, 0, 0)), Err("That would block your way out."));
}
#[test] fn furniture_interactions_do_not_skip_time() {
    let mut g = g0(at(11, 12, 0, 0)); home(&mut g); for (u, d, x) in [(11, 53, 0), (12, 54, 3), (13, 55, 1)] { g.w.room.push(Furn { uid: u, def: d, x, z: 0, rot: 0, on: false }) }
    let day = g.w.day; g.ipos = vec2(0.5, 2.4); g.face = vec2(0., -1.); g.interact(); assert_eq!(g.pose, Pose::Lie(11)); assert_eq!(g.w.day, day);
    g.step_move(vec2(0., 1.), false, 0.016); assert_eq!(g.pose, Pose::Stand); g.ipos = vec2(3.5, 1.5); g.face = vec2(0., -1.); g.interact(); assert!(g.w.room.iter().any(|f| f.uid == 12 && f.on));
    g.ipos = vec2(1.5, 1.5); g.interact(); assert_eq!(g.ui, Ui::Storage(0, 0));
}

// ---- world / collision / targeting ----
#[test] fn door_beats_nearby_resident() {
    let mut g = g0(at(11, 12, 0, 0)); g.pos = door_stand(HOME); g.face = vec2(0., -1.); for n in &mut g.npcs { n.inside = true }
    g.npcs[0].inside = false; g.npcs[0].pos = door_stand(HOME) + vec2(0.7, 0.1);
    assert_eq!(g.target(), Some(Tg::Door(HOME))); g.face = vec2(1., 0.); assert_eq!(g.target(), Some(Tg::Npc(0)));
    g.pos = vec2(13.5, 11.5); g.face = vec2(0., 1.); assert_ne!(g.target(), Some(Tg::Door(HOME))); // no interacting through the house
}
#[test] fn water_bridge_buildings_collision() {
    let g = g0(at(11, 12, 0, 0)); assert!(!g.walk_tile(24, 5) && !g.walk_tile(25, 20)); assert!(g.walk_tile(24, 9) && g.walk_tile(25, 17)); assert!(!g.walk_tile(13, 13));
    assert!(g.bfs((20, 9), (30, 9), true).is_some()); for b in 0..BLDS.len() { let t = door_tile(b); assert!(g.walk_tile(t.0, t.1), "door {b} reachable"); assert!(g.bfs(door_tile(HOME), t, true).is_some()) }
}
#[test] fn enter_exit_room() {
    let mut g = g0(at(11, 12, 0, 0)); g.pos = door_stand(HOME); g.face = vec2(0., -1.); for n in &mut g.npcs { n.inside = true }
    g.interact(); for _ in 0..40 { g.update(&Inp::none(), 0.016, at(11, 12, 0, 1)) ; } assert_eq!(g.scene, Scene::Room(HOME));
    for _ in 0..120 { g.update(&Inp::keys(&[KeyCode::S], &[]), 0.016, at(11, 12, 0, 2)); if g.scene == Scene::Town { break } }
    for _ in 0..40 { g.update(&Inp::none(), 0.016, at(11, 12, 0, 3)); } assert_eq!(g.scene, Scene::Town); assert_eq!(g.pos, door_stand(HOME));
}
#[test] fn residents_avoid_water_and_buildings() {
    let mut g = g0(at(11, 10, 30, 0)); for _ in 0..3000 { g.update(&Inp::none(), 0.033, at(11, 10, 30, 0)); for n in g.npcs.iter().filter(|n| !n.inside) { let t = tile_of(n.pos); assert!(g.walk_tile(t.0, t.1), "resident on {:?}", t) } }
    assert!(g.npcs.iter().any(|n| !n.inside));
}
#[test] fn legacy_v2_migrates() {
    let p = std::env::temp_dir().join("hb3_legacy.txt"); let map = town();
    let mut s = format!("v2\nOld\n739900\n777\n2\n22\n{}\n{}\n1,1,12,20\n{}\n100 100\n", ".".repeat(64 * 48), "0".repeat(64 * 48), "1".repeat(20));
    for f in [3, 12, 0, 20] { s += &format!("{f} 999 1 100 100\n") } s += "room 1 4 4\n1:0:0:0:0,2:1:2:1:1\n"; std::fs::write(&p, s).unwrap();
    let w = save::migrate(p.to_str().unwrap(), &map).unwrap();
    assert_eq!((w.name.as_str(), w.money, w.home, w.native, w.day), ("Old", 777, 2, 22, 739900)); assert_eq!(w.inv, vec![Slot { def: 1, qty: 2, uid: 0 }, Slot { def: 12, qty: 1, uid: 0 }, Slot { def: 20, qty: 1, uid: 0 }]);
    assert_eq!(w.npcs[1].friend, 12); assert_eq!(w.room, vec![Furn { uid: 1, def: 55, x: 0, z: 0, rot: 0, on: false }, Furn { uid: 2, def: 51, x: 2, z: 1, rot: 1, on: false }]); assert_eq!(w.uid, 3);
    let g = Game::from_world(w, map); assert_eq!(g.w.stock.len(), 5);
}
