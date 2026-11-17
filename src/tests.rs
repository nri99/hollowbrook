use crate::clock::day_id;
use crate::data::*;
use crate::save;
use crate::world::*;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use macroquad::prelude::*;
use crate::events::*;

#[test] fn m3_contest_two_players_entries_prizes_and_reload() {
    let mut g = two(at(11, 9, 0, 0)); g.hr = 9;
    assert_eq!(g.contest_kind(), Some(0)); assert!(g.free(vec2(CONTEST_SPOT.0, CONTEST_SPOT.1)));
    g.contest_talk(); assert!(g.contest_entry().is_some());
    g.w.p.inv.push(Slot { def: 16, qty: 1, uid: 0, size_mm: catch_base(16) * 135 / 100 });
    let score = g.contest_submit(0).unwrap(); assert!(score >= 116); assert!(g.w.p.inv.is_empty());
    g.w.p.inv.push(Slot { def: 10, qty: 1, uid: 0, size_mm: 1 }); assert!(g.contest_submit(0).is_err()); assert_eq!(g.w.p.inv.len(), 1);
    assert!(g.contest_claim().is_err());
    g.activate(1); assert!(g.contest_entry().is_none()); assert!(g.contest_submit(0).is_err()); g.contest_talk();
    g.w.p.inv.push(Slot { def: 10, qty: 1, uid: 0, size_mm: catch_base(10) }); g.contest_submit(0).unwrap();
    g.activate(0); let mut g = reload(&g, "m3_contest"); g.join("Partner"); g.hr = 17;
    let money = g.w.p.money; g.contest_claim().unwrap(); assert!(g.w.p.money > money); assert!(g.contest_claim().is_err());
    assert!(g.w.p.inv.iter().any(|s| s.def == 170));
    let mut g = reload(&g, "m3_claimed"); g.hr = 17; assert!(g.contest_claim().is_err());
    g.join("Partner"); g.activate(1); assert!(!g.contest_entry().unwrap().claimed); g.contest_claim().unwrap();
    g.hr = 22; assert!(!g.contest_host_open());
}

#[test] fn m3_contest_rollover_full_pockets_bug_date_and_legacy() {
    assert_eq!(contest_on(crate::clock::day_id(at(22, 12, 0, 0))), Some(1));
    assert_eq!(contest_on(crate::clock::day_id(at(18, 12, 0, 0))), None);
    let mut g = two(at(11, 16, 59, 0)); g.hr = 16; g.contest_talk();
    g.w.p.inv.push(Slot { def: 16, qty: 1, uid: 0, size_mm: 500 }); g.contest_submit(0).unwrap();
    g.activate(1); g.contest_talk(); g.w.p.inv.push(Slot { def: 16, qty: 1, uid: 0, size_mm: 500 }); g.contest_submit(0).unwrap();
    for _ in 0..INV_CAP { g.add_furn(50); } g.hr = 17; g.contest_claim().unwrap(); assert!(g.w.p.mail.iter().any(|l| l.item == Some(170)));
    g.activate(0); assert!(g.catch_up(at(13, 6, 0, 0))); let n = g.w.p.mail.iter().filter(|l| l.from == CONTEST_HOST).count(); assert_eq!(n, 1);
    assert!(!g.catch_up(at(13, 6, 1, 0))); assert_eq!(g.w.p.mail.iter().filter(|l| l.from == CONTEST_HOST).count(), n);
    let mut old = serde_json::to_value(g.persist()).unwrap(); old["version"] = 4.into(); old["p"].as_object_mut().unwrap().remove("contests");
    old["p"]["inv"] = serde_json::json!([{ "def": 12, "qty": 9, "uid": 0 }]);
    let w: World = serde_json::from_value(save::upgrade(old)).unwrap(); assert_eq!(w.version, SAVE_VERSION); assert_eq!(w.p.inv[0].qty, 9); assert_eq!(w.p.inv[0].size_mm, 0); assert!(w.p.contests.is_empty());
}

#[test] fn m3_measured_capture_paths_storage_and_gift_selection() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.p.equip = Some(60); g.me.face = vec2(1., 0.);
    g.bugs = vec![(7, g.me.pos + vec2(0.7, 0.), 0.)]; g.use_tool(); assert_eq!(g.w.p.inv.len(), 1); assert!(g.w.p.inv[0].size_mm > 0);
    g.w.p.equip = Some(61); g.shadows = vec![Shadow { def: 12, pos: g.me.pos, vel: Vec2::ZERO }];
    g.me.fish = Some(Fishing { bob: g.me.pos, fish: Some(0), t: 0., nib: 0, need: 0, bite: true, dip: 0. }); g.use_tool();
    assert_eq!(g.w.p.inv[1].def, 12); let fish = g.w.p.inv[1]; assert!(fish.size_mm > 0);
    g.transfer(0, 1); assert_eq!(g.w.p.storage[0], fish); g.transfer(1, 0); assert_eq!(g.w.p.inv[1], fish);
    g.w.p.inv.push(Slot { def: 12, qty: 1, uid: 0, size_mm: fish.size_mm + 1 }); g.give_gift(0, 2); assert_eq!(g.w.p.inv[1], fish);
    let g2 = reload(&g, "m3_measurements"); assert_eq!(g2.w.p.inv, g.w.p.inv);
}

fn at(d: u32, h: u32, m: u32, s: u32) -> NaiveDateTime { NaiveDate::from_ymd_opt(2026, 11, d).unwrap().and_hms_opt(h, m, s).unwrap() } // Nov 10 2026 = Tue
fn tmp(n: &str) -> String { std::env::temp_dir().join(format!("hb3_{n}.json")).to_string_lossy().into() }
fn g0(t: NaiveDateTime) -> Game { let mut g = Game::new("T".into(), t); g.hr = 12; g }
fn talk_daily(g: &mut Game) -> i32 { g.w.p.npcs[0].req = None; g.talk(0); g.me.ui = Ui::Play; g.w.p.npcs[0].friend }
fn reload(g: &Game, n: &str) -> Game { let p = tmp(n); save::save(&g.persist(), &p); Game::from_world(save::load(&p).unwrap(), town()) }
fn home(g: &mut Game) { g.swap(Scene::Room(HOME)) }

// ---- clock / daily refresh (carried over) ----
#[test] fn day_identity() { assert_eq!(at(10, 0, 0, 0).weekday(), chrono::Weekday::Tue); assert_eq!(day_id(at(10, 23, 59, 0)), day_id(at(11, 0, 1, 0))); assert_ne!(day_id(at(11, 5, 59, 0)), day_id(at(11, 6, 0, 0))) }
#[test] fn midnight_is_not_reset() { let mut g = g0(at(10, 23, 59, 50)); let f = talk_daily(&mut g); assert!(!g.catch_up(at(11, 0, 0, 10))); assert!(g.w.p.npcs[0].talked); assert_eq!(talk_daily(&mut g), f); assert!(g.log.is_empty()) }
#[test] fn six_am_refresh_once_and_stock_vs_hours() {
    let mut g = g0(at(11, 5, 59, 50)); talk_daily(&mut g); g.w.stock[0].sold = true;
    assert!(!g.catch_up(at(11, 5, 59, 59))); assert!(g.w.stock[0].sold);
    assert!(g.catch_up(at(11, 6, 0, 10))); assert_eq!(g.log.len(), 1); assert!(!g.w.p.npcs[0].talked); assert!(g.w.stock.iter().all(|s| !s.sold));
    assert!(!shop_open(6) && shop_open(8)); for i in 0..200 { assert!(!g.catch_up(at(11, 6, 1, i % 60))) } assert_eq!(g.log.len(), 1);
}
#[test] fn reload_same_day_keeps_daily_state() { let mut g = g0(at(11, 6, 30, 0)); talk_daily(&mut g); g.w.stock[2].sold = true; let mut g2 = reload(&g, "same"); assert!(!g2.catch_up(at(11, 7, 0, 0))); assert!(g2.w.p.npcs[0].talked && g2.w.stock[2].sold); assert_eq!(g2.w, g.persist()) }
#[test] fn closed_three_days_regrows_once() {
    let mut g = g0(at(11, 5, 0, 0)); g.w.trees[0].fruit = false; g.w.trees[0].shaken = g.w.day; g.w.p.inv = vec![Slot { size_mm: 0, def: 20, qty: 3, uid: 0 }]; g.w.p.money = 4321;
    let mut g2 = reload(&g, "three"); assert!(g2.catch_up(at(14, 5, 0, 0))); assert_eq!(g2.log.len(), 1); assert!(g2.log[0].contains("(3d)")); assert!(g2.w.trees[0].fruit);
    assert_eq!((g2.w.p.money, g2.w.p.inv.clone()), (4321, vec![Slot { size_mm: 0, def: 20, qty: 3, uid: 0 }])); assert!(!g2.catch_up(at(14, 5, 30, 0)));
}
#[test] fn fruit_not_regrown_early() { let mut g = g0(at(11, 7, 0, 0)); g.w.trees[0].fruit = false; g.w.trees[0].shaken = g.w.day; assert!(g.catch_up(at(13, 7, 0, 0))); assert!(!g.w.trees[0].fruit) }
#[test] fn clock_rollback() { let mut g = g0(at(11, 7, 0, 0)); let d = g.w.day; assert!(!g.catch_up(at(9, 7, 0, 0))); assert_eq!(g.w.day, d) }

// ---- economy ----
#[test] fn shop_rules() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.p.money = 10000; let p = def(g.w.stock[0].def).buy;
    assert!(g.buy(0).is_ok()); assert_eq!(g.w.p.money, 10000 - p); assert!(g.w.stock[0].sold); let uid = g.w.p.inv[0].uid; assert!(uid != 0 && g.w.p.inv[0].def == g.w.stock[0].def);
    let m = g.w.p.money; assert_eq!(g.buy(0), Err("Sold out!")); assert_eq!(g.w.p.money, m); assert_eq!(g.w.p.inv.len(), 1); // repeated input: no duplicate
    g.w.p.money = 0; assert_eq!(g.buy(1), Err("Not enough coins.")); assert!(!g.w.stock[1].sold); assert_eq!(g.w.p.inv.len(), 1);
    g.w.p.money = 10000; while g.w.p.inv.len() < INV_CAP { g.w.p.inv.push(Slot { size_mm: 0, def: 40, qty: 1, uid: 0 }) } assert_eq!(g.buy(2), Err("Your pockets are full.")); assert_eq!(g.w.p.money, 10000); assert!(!g.w.stock[2].sold);
    g.me.ui = Ui::Confirm(3); let before = g.w.clone(); g.update(&Inp::keys(&[KeyCode::Escape], &[KeyCode::Escape]), 0.016, at(11, 12, 0, 1)); assert_eq!(g.me.ui, Ui::Play); assert_eq!(g.w.stock, before.stock); assert_eq!(g.w.p.money, before.p.money); // cancel
    assert!(g.buy(5).is_ok()); assert!(g.w.p.tools.contains(&62)); assert!(g.buy(5).is_err());
}
#[test] fn sold_out_persists_until_six() { let mut g = g0(at(11, 12, 0, 0)); g.w.p.money = 9999; g.buy(1).unwrap(); let mut g2 = reload(&g, "sold"); assert!(!g2.catch_up(at(12, 5, 59, 0))); assert!(g2.w.stock[1].sold); assert!(g2.catch_up(at(12, 6, 0, 1))); assert!(g2.w.stock.iter().all(|s| !s.sold)) }
#[test] fn gather_fruit_and_sell() {
    let mut g = g0(at(11, 12, 0, 0)); let ti = g.w.trees.iter().position(|t| g.walk_tile(t.x, t.z + 1)).unwrap(); let t = g.w.trees[ti];
    g.me.pos = vec2(t.x as f32 + 0.5, t.z as f32 + 1.5); g.me.face = vec2(0., -1.); for n in &mut g.npcs { n.inside = true }
    assert_eq!(g.target(), Some(Tg::Tree(ti))); g.interact(); assert!(!g.w.trees[ti].fruit);
    let drops: Vec<GItem> = g.w.ground.iter().filter(|i| i.def == g.w.native).copied().collect(); assert!(!drops.is_empty());
    for d in &drops { g.me.pos = vec2(d.x, d.z + 0.6); if !g.free(g.me.pos) { g.me.pos = vec2(d.x + 0.6, d.z) ; g.me.face = vec2(-1., 0.) } else { g.me.face = vec2(0., -1.) } if let Some(Tg::Item(_)) = g.target() { g.interact() } }
    let got: u16 = g.w.p.inv.iter().filter(|s| s.def == g.w.native).map(|s| s.qty).sum(); assert!(got >= 1);
    let m = g.w.p.money; g.sell_all(); assert_eq!(g.w.p.money, m + 100 * got as u32); assert!(g.w.p.inv.is_empty());
}
#[test] fn digging_holes_and_sites() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.p.tools.push(62); g.w.p.equip = Some(62); let site = g.w.digs[0];
    g.me.pos = tc((site.0, site.1 + 1)); g.me.face = vec2(0., -1.); if !g.free(g.me.pos) { g.me.pos = tc((site.0 + 1, site.1)); g.me.face = vec2(-1., 0.) }
    g.use_tool(); assert!(g.w.holes.contains(&site)); assert!(g.w.p.inv.iter().any(|s| matches!(s.def, 45 | 46 | 47))); assert!(!g.walk_tile(site.0, site.1));
    g.use_tool(); assert!(!g.w.holes.contains(&site)); assert!(g.walk_tile(site.0, site.1));
}

// ---- furniture ----
#[test] fn furniture_buy_place_move_rotate_pick_no_dup() {
    let mut g = g0(at(11, 12, 0, 0)); home(&mut g); g.add_furn(51); let uid = g.w.p.inv[0].uid;
    g.start_place(uid); g.me.placing.as_mut().unwrap().f.x = 0; g.me.placing.as_mut().unwrap().f.z = 0; g.place_confirm().unwrap();
    assert!(g.w.p.inv.is_empty()); assert_eq!(g.w.p.room.len(), 1); assert_eq!(g.w.p.room[0].uid, uid);
    g.start_move(uid); assert!(g.w.p.room.is_empty()); assert_eq!(g.persist().p.room.len(), 1); // still owned while moving
    { let p = g.me.placing.as_mut().unwrap(); p.f.rot = 1; p.f.x = 0; p.f.z = 0; } g.place_confirm().unwrap();
    assert!(g.room_block(HOME, 0, 1, 0) && !g.room_block(HOME, 1, 0, 0)); // rotated footprint covers (0,0),(0,1)
    g.start_move(uid); g.place_cancel(); assert_eq!(g.w.p.room.len(), 1); assert_eq!(g.w.p.room[0].rot, 1);
    g.pick_furn(uid); assert!(g.w.p.room.is_empty()); assert_eq!(g.w.p.inv.iter().filter(|s| s.uid == uid).count(), 1);
    let g2 = reload(&g, "furn"); assert_eq!(g2.w.p.inv, g.w.p.inv);
}
#[test] fn placement_rejections() {
    let mut g = g0(at(11, 12, 0, 0)); home(&mut g); let mk = |d: u16, x: i32, z: i32, rot: u8| Furn { uid: 99, def: d, x, z, rot, on: false };
    assert_eq!(g.place_ok(&mk(50, 2, 3, 0)), Err("Keep the doorway clear.")); assert_eq!(g.place_ok(&mk(51, 3, 0, 0)), Err("It doesn't fit there."));
    g.me.ipos = vec2(2.5, 2.5); assert_eq!(g.place_ok(&mk(50, 2, 2, 0)), Err("You're standing there.")); g.me.ipos = vec2(2.5, 3.5); g.w.p.room.push(Furn { uid: 5, def: 52, x: 0, z: 0, rot: 0, on: false });
    assert_eq!(g.place_ok(&mk(51, 0, 0, 0)), Err("Something is in the way.")); assert_eq!(g.place_ok(&mk(57, 1, 0, 0)), Err("Wall items go on the back wall.")); assert!(g.place_ok(&mk(57, 1, -1, 0)).is_ok());
    g.me.ipos = vec2(3.5, 0.5); g.w.p.room.push(Furn { uid: 6, def: 50, x: 3, z: 1, rot: 0, on: false }); assert_eq!(g.place_ok(&mk(50, 2, 0, 0)), Err("That would block your way out."));
}
#[test] fn furniture_interactions_do_not_skip_time() {
    let mut g = g0(at(11, 12, 0, 0)); home(&mut g); for (u, d, x) in [(11, 53, 0), (12, 54, 3), (13, 55, 1)] { g.w.p.room.push(Furn { uid: u, def: d, x, z: 0, rot: 0, on: false }) }
    let day = g.w.day; g.me.ipos = vec2(0.5, 2.4); g.me.face = vec2(0., -1.); g.interact(); assert_eq!(g.me.pose, Pose::Lie(11)); assert_eq!(g.w.day, day);
    g.step_move(vec2(0., 1.), false, 0.016); assert_eq!(g.me.pose, Pose::Stand); g.me.ipos = vec2(3.5, 1.5); g.me.face = vec2(0., -1.); g.interact(); assert!(g.w.p.room.iter().any(|f| f.uid == 12 && f.on));
    g.me.ipos = vec2(1.5, 1.5); g.interact(); assert_eq!(g.me.ui, Ui::Storage(0, 0));
}

// ---- world / collision / targeting ----
#[test] fn door_beats_nearby_resident() {
    let mut g = g0(at(11, 12, 0, 0)); g.me.pos = door_stand(HOME); g.me.face = vec2(0., -1.); for n in &mut g.npcs { n.inside = true }
    g.npcs[0].inside = false; g.npcs[0].pos = door_stand(HOME) + vec2(0.7, 0.1);
    assert_eq!(g.target(), Some(Tg::Door(HOME))); g.me.face = vec2(1., 0.); assert_eq!(g.target(), Some(Tg::Npc(0)));
    g.me.pos = vec2(13.5, 11.5); g.me.face = vec2(0., 1.); assert_ne!(g.target(), Some(Tg::Door(HOME))); // no interacting through the house
}
#[test] fn water_bridge_buildings_collision() {
    let g = g0(at(11, 12, 0, 0)); assert!(!g.walk_tile(24, 5) && !g.walk_tile(25, 20)); assert!(g.walk_tile(24, 9) && g.walk_tile(25, 17)); assert!(!g.walk_tile(13, 13));
    assert!(g.bfs((20, 9), (30, 9), true).is_some()); for b in 0..BLDS.len() { let t = door_tile(b); assert!(g.walk_tile(t.0, t.1), "door {b} reachable"); assert!(g.bfs(door_tile(HOME), t, true).is_some()) }
}
#[test] fn enter_exit_room() {
    let mut g = g0(at(11, 12, 0, 0)); g.me.pos = door_stand(HOME); g.me.face = vec2(0., -1.); for n in &mut g.npcs { n.inside = true }
    g.interact(); for _ in 0..40 { g.update(&Inp::none(), 0.016, at(11, 12, 0, 1)) ; } assert_eq!(g.me.scene, Scene::Room(HOME));
    for _ in 0..120 { g.update(&Inp::keys(&[KeyCode::S], &[]), 0.016, at(11, 12, 0, 2)); if g.me.scene == Scene::Town { break } }
    for _ in 0..40 { g.update(&Inp::none(), 0.016, at(11, 12, 0, 3)); } assert_eq!(g.me.scene, Scene::Town); assert_eq!(g.me.pos, door_stand(HOME));
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
    assert_eq!((w.p.name.as_str(), w.p.money, w.p.home, w.native, w.day), ("Old", 777, 2, 22, 739900)); assert_eq!(w.p.inv, vec![Slot { size_mm: 0, def: 1, qty: 2, uid: 0 }, Slot { size_mm: 0, def: 12, qty: 1, uid: 0 }, Slot { size_mm: 0, def: 20, qty: 1, uid: 0 }]);
    assert_eq!(w.p.npcs[1].friend, 12); assert_eq!(w.p.room, vec![Furn { uid: 1, def: 55, x: 0, z: 0, rot: 0, on: false }, Furn { uid: 2, def: 51, x: 2, z: 1, rot: 1, on: false }]); assert_eq!(w.uid, 3);
    let g = Game::from_world(w, map); assert_eq!(g.w.stock.len(), 5);
}

// ---- save safety ----
#[test] fn backups_rotate() {
    let d = std::env::temp_dir().join("hb3_bk"); let _ = std::fs::remove_dir_all(&d); let f = std::env::temp_dir().join("hb3_bk_save.json"); std::fs::write(&f, "x").unwrap();
    for _ in 0..8 { assert!(save::backup(f.to_str().unwrap(), d.to_str().unwrap()).is_some()); std::thread::sleep(std::time::Duration::from_millis(5)) }
    assert_eq!(std::fs::read_dir(&d).unwrap().count(), save::KEEP_BACKUPS);
}
#[test] fn corrupt_save_moved_aside() {
    let f = std::env::temp_dir().join("hb3_corrupt.json"); std::fs::write(&f, "{not json").unwrap();
    match save::load_checked(f.to_str().unwrap()) { save::Loaded::Corrupt(p) => { assert!(!f.exists()); assert_eq!(std::fs::read_to_string(&p).unwrap(), "{not json"); let _ = std::fs::remove_file(p); } _ => panic!("expected corrupt") }
}
#[test] fn trampled_flowers_recover_next_morning() {
    let mut g = g0(at(11, 12, 0, 0)); let i = g.map.iter().position(|&c| c == FLOWER).unwrap(); let t = ((i % W) as i32, (i / W) as i32);
    let mut n = 0; for _ in 0..400 { g.me.pos = tc(t) - vec2(0.3, 0.); g.step_move(vec2(1., 0.), true, 0.016); n += 1; if g.w.trampled.contains(&t) { break } }
    assert!(g.w.trampled.contains(&t), "not trampled after {n} runs"); let g2 = reload(&g, "tr"); assert!(g2.w.trampled.contains(&t));
    let mut g3 = g2; assert!(g3.catch_up(at(12, 6, 0, 1))); assert!(g3.w.trampled.is_empty());
    let mut g4 = g0(at(11, 12, 0, 0)); for _ in 0..400 { g4.me.pos = tc(t) - vec2(0.3, 0.); g4.step_move(vec2(1., 0.), false, 0.016) } assert!(g4.w.trampled.is_empty()); // walking never tramples
}

// ---- local multiplayer (MP-01/02) ----
fn two(t: NaiveDateTime) -> Game { let mut g = g0(t); g.join("P2"); for n in &mut g.npcs { n.inside = true } g }
#[test] fn v3_json_upgrades_to_profiles() {
    let g = g0(at(11, 12, 0, 0)); let mut v = serde_json::to_value(g.persist()).unwrap();
    let p = v["p"].clone(); let o = v.as_object_mut().unwrap(); o.remove("p"); o.remove("others"); o.insert("version".into(), 3.into());
    for (k, x) in p.as_object().unwrap() { if k != "id" { o.insert(k.clone(), x.clone()); } }
    let w: World = serde_json::from_value(save::upgrade(v)).unwrap(); assert_eq!(w.version, SAVE_VERSION); { let a = &w.p; let b = g.persist().p; assert_eq!((&a.name, a.money, &a.inv, &a.room, &a.npcs, &a.tools), (&b.name, b.money, &b.inv, &b.room, &b.npcs, &b.tools)); } assert!(w.others.is_empty());
}
#[test] fn two_profiles_independent_and_persist() {
    let mut g = two(at(11, 12, 0, 0)); g.activate(1); g.w.p.money = 5; g.add(20); g.activate(0); assert_eq!(g.w.p.money, 1000); assert!(g.w.p.inv.is_empty());
    let mut g2 = reload(&g, "mp"); assert_eq!(g2.w.p.id, 0); assert_eq!(g2.w.others.len(), 1); g2.join("x"); g2.activate(1); assert_eq!(g2.w.p.money, 5); assert_eq!(g2.w.p.inv[0].def, 20); assert_eq!(g2.w.p.name, "P2");
}
#[test] fn last_stock_and_pickup_only_once() {
    let mut g = two(at(11, 12, 0, 0)); for id in [0, 1] { g.activate(id); g.w.p.money = 9000 }
    g.activate(0); assert!(g.buy(0).is_ok()); g.activate(1); assert_eq!(g.buy(0), Err("Sold out!")); assert_eq!(g.w.p.money, 9000);
    let it = g.w.ground[0]; for id in [0, 1] { g.activate(id); g.me.scene = Scene::Town; g.me.pos = vec2(it.x, it.z + 0.6); g.me.face = vec2(0., -1.); if let Some(Tg::Item(_)) = g.target() { g.interact() } }
    let total: u16 = [0, 1].iter().map(|&id| { g.activate(id); g.w.p.inv.iter().filter(|s| s.def == it.def).map(|s| s.qty).sum::<u16>() }).sum(); assert_eq!(total, 1);
}
#[test] fn different_scenes_and_independent_ui() {
    let mut g = two(at(11, 12, 0, 0)); g.activate(0); g.swap(Scene::Room(HOME)); g.me.ui = Ui::Inv(0);
    g.activate(1); assert_eq!(g.me.scene, Scene::Town); assert_eq!(g.me.ui, Ui::Play); g.step_move(vec2(1., 0.), false, 0.1); let p1 = g.me.pos;
    g.activate(0); assert_eq!(g.me.scene, Scene::Room(HOME)); assert_eq!(g.me.ui, Ui::Inv(0)); g.activate(1); assert_eq!(g.me.pos, p1);
}
#[test] fn rollover_with_players_in_different_scenes() {
    let mut g = two(at(11, 5, 59, 0)); for id in [0, 1] { g.activate(id); g.w.p.npcs[0].talked = true } g.activate(0); g.swap(Scene::Room(HOME));
    assert!(g.catch_up(at(11, 6, 0, 1))); assert_eq!(g.log.len(), 1); for id in [0, 1] { g.activate(id); assert!(!g.w.p.npcs[0].talked) } assert!(!g.catch_up(at(11, 6, 0, 5)));
    g.activate(0); assert_eq!(g.me.scene, Scene::Room(HOME));
}
#[test] fn leave_keeps_possessions() {
    let mut g = two(at(11, 12, 0, 0)); g.activate(1); g.add(40); g.w.p.money = 77; g.activate(0); g.leave(1); assert_eq!(g.ids(), vec![0]);
    let w = g.persist(); assert_eq!(w.others[0].money, 77); assert_eq!(w.others[0].inv[0].def, 40); g.join("again"); g.activate(1); assert_eq!(g.w.p.money, 77);
}
#[test] fn separate_homes_owner_only_furniture() {
    let mut g = two(at(11, 12, 0, 0)); g.activate(1); assert_eq!(g.home(), 7); g.swap(Scene::Room(7)); g.add_furn(50); let uid = g.w.p.inv[0].uid;
    g.start_place(uid); { let p = g.me.placing.as_mut().unwrap(); p.f.x = 0; p.f.z = 0; } g.place_confirm().unwrap(); assert_eq!(g.room_furn(7).len(), 1); assert!(g.room_furn(HOME).is_empty());
    g.activate(0); g.swap(Scene::Room(7)); assert_eq!(g.room_furn(7).len(), 1); g.me.ipos = vec2(0.5, 1.5); g.me.face = vec2(0., -1.);
    assert!(matches!(g.target(), Some(Tg::Furn(u, _)) if u == uid)); g.update(&Inp::keys(&[KeyCode::G], &[KeyCode::G]), 0.016, at(11, 12, 0, 1)); assert_eq!(g.room_furn(7).len(), 1); assert!(g.w.p.inv.is_empty()); // visitor cannot take it
    let w = g.persist(); assert_eq!(w.others[0].room.len(), 1); assert!(w.p.room.is_empty());
}

// ---- museum (P3) ----
#[test] fn museum_shared_donation_once_and_fossil_id() {
    let mut g = two(at(11, 12, 0, 0)); g.activate(0); g.add(12); g.add(47); g.add(0); g.activate(1); g.add(12);
    g.activate(0); g.donate(); assert!(g.w.donated.iter().any(|x| *x == (12, 0)) && g.w.donated.iter().any(|x| *x == (0, 0))); assert_eq!(g.w.donated.iter().filter(|x| FOSSILS.contains(&x.0)).count(), 1); assert!(!g.w.p.inv.iter().any(|s| s.def == 47));
    g.activate(1); g.donate(); assert_eq!(g.w.donated.iter().filter(|x| x.0 == 12).count(), 1); assert!(g.w.p.inv.iter().any(|s| s.def == 12)); // already donated: P2 keeps the carp
    let g2 = reload(&g, "mus"); assert_eq!(g2.w.donated, g.w.donated);
}
#[test] fn museum_reachable_and_blocks() { let g = g0(at(11, 12, 0, 0)); let m = BLDS.iter().position(|b| b.kind == BK::Museum).unwrap(); let t = door_tile(m); assert!(g.walk_tile(t.0, t.1)); assert!(g.bfs(door_tile(HOME), t, true).is_some()); assert!(g.room_block(m, 0, 2, 0) && !g.room_block(m, 3, 3, 0)) }
#[test] fn clothing_buy_wear_swap_persist() {
    let mut g = two(at(11, 12, 0, 0)); g.activate(1); g.w.p.money = 2000; assert!(g.buy_cloth(63).is_ok()); assert!(g.buy_cloth(65).is_ok()); assert_eq!(g.w.p.money, 2000 - 400 - 650);
    let (a, b) = (g.w.p.inv[0].uid, g.w.p.inv[1].uid); g.wear(a); assert_eq!(g.w.p.shirt, Some(63)); assert_eq!(g.w.p.inv.len(), 1);
    g.wear(b); assert_eq!(g.w.p.shirt, Some(65)); assert_eq!(g.w.p.inv.len(), 1); assert_eq!(g.w.p.inv[0].def, 63); // old shirt back in pockets
    g.w.p.money = 10; assert_eq!(g.buy_cloth(64), Err("Not enough coins.")); g.activate(0); assert_eq!(g.w.p.shirt, None);
    let mut g2 = reload(&g, "cloth"); g2.join("x"); g2.activate(1); assert_eq!(g2.w.p.shirt, Some(65));
}
#[test] fn home_expansion_balance() {
    let mut g = two(at(11, 12, 0, 0)); g.w.p.room.push(Furn { uid: 51, def: 50, x: 3, z: 4, rot: 0, on: false });
    g.me.ui = Ui::Sell(0); g.update_player(&Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]), 0.016); assert_eq!(g.me.ui, Ui::ConfirmExpand);
    g.update_player(&Inp::keys(&[KeyCode::Escape], &[KeyCode::Escape]), 0.016); assert_eq!((g.w.p.home, g.w.p.debt), (0, 0)); // cancel: no change
    g.expand_home().unwrap(); assert_eq!((g.rdims(HOME), g.w.p.debt, g.w.p.money), ((5, 5), EXPAND_COST[0], 1000)); assert!(g.expand_home().is_err()); // must clear balance first
    g.pay_debt(); assert_eq!((g.w.p.money, g.w.p.debt), (0, EXPAND_COST[0] - 1000)); g.w.p.money = 99999; g.pay_debt(); assert_eq!(g.w.p.debt, 0);
    g.expand_home().unwrap(); assert!(g.w.p.storage.iter().any(|s| s.uid == 51)); g.activate(1); assert_eq!((g.w.p.debt, g.rdims(7)), (0, (4, 4)));
    g.activate(0); let g2 = reload(&g, "debt"); assert_eq!(g2.w.p.debt, EXPAND_COST[1]);
}
#[test] fn mail_delivered_per_profile_claim_once() {
    let mut g = two(at(11, 12, 0, 0)); g.activate(1); for n in &mut g.w.p.npcs { n.friend = 20 } let m0 = g.w.p.money; g.activate(0);
    for d in 12..40 { g.catch_up(at(1 + d % 28, 7, 0, 0).checked_add_months(chrono::Months::new((d / 28) as u32)).unwrap()); g.activate(1); if !g.w.p.mail.is_empty() { break } g.activate(0) }
    g.activate(1); assert!(!g.w.p.mail.is_empty(), "P2 got no letter"); assert_eq!(g.w.p.money, m0); // coins wait in the letter
    let c = g.w.p.mail.last().unwrap().coins; g.open_letter(0); assert_eq!(g.w.p.money, m0 + c); g.open_letter(0); assert_eq!(g.w.p.money, m0 + c); assert!(g.w.p.mail.last().unwrap().claimed);
    g.activate(0); assert!(!g.w.p.mail.iter().any(|l| l.coins > 0)); let mut g2 = reload(&g, "mail"); g2.join("x"); g2.activate(1); assert!(g2.w.p.mail.last().unwrap().claimed);
    assert!(!g.walk_tile(mailbox_tile(0).0, mailbox_tile(0).1));
}
#[test] fn weather_rule_snow_in_winter() { let wet = (739900..740400).find(|&d| rainy(d)).unwrap(); let dry = (739900..740400).find(|&d| !rainy(d)).unwrap();
    assert_eq!(weather(wet, 1), Wx::Snow); assert_eq!(weather(wet, 12), Wx::Snow); assert_eq!(weather(wet, 6), Wx::Rain); assert_eq!(weather(dry, 1), Wx::Clear); }
#[test] fn catalog_reorder_full_price() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.p.money = 5000; let d = g.w.stock[0].def; g.buy(0).unwrap(); assert!(g.w.p.catalog.contains(&d)); let m = g.w.p.money;
    g.order(d).unwrap(); assert_eq!(g.w.p.money, m - def(d).buy); assert_eq!(g.w.p.inv.iter().filter(|s| s.def == d).count(), 2); assert_ne!(g.w.p.inv[0].uid, g.w.p.inv[1].uid);
    assert_eq!(g.order(57).err(), if d == 57 { None } else { Some("That isn't in your catalog.") }.filter(|_| d != 57)); g.w.p.money = 0; assert_eq!(g.order(d), Err("Not enough coins."));
    let g2 = reload(&g, "cat"); assert!(g2.w.p.catalog.contains(&d));
}
#[test] fn gifts_once_per_day_with_preferences() {
    let mut g = g0(at(11, 12, 0, 0)); g.add(20); g.add(20); g.add(40); let f0 = g.w.p.npcs[0].friend;
    g.give_gift(0, 0); assert_eq!(g.w.p.npcs[0].friend, f0 + 2); assert_eq!(g.w.p.inv[0].qty, 1); // Juniper likes fruit
    g.give_gift(0, 0); assert_eq!(g.w.p.npcs[0].friend, f0 + 2); assert_eq!(g.w.p.inv[0].qty, 1); // once per day
    let f1 = g.w.p.npcs[1].friend; let shell = g.w.p.inv.iter().position(|s| s.def == 40).unwrap(); g.give_gift(1, shell); assert_eq!(g.w.p.npcs[1].friend, f1 + 1);
    assert!(g.catch_up(at(12, 6, 0, 1))); assert!(!g.w.p.npcs[0].gifted);
    g.me.ui = Ui::Talk { npc: Some(2), who: "x".into(), lines: vec!["a".into()], i: 0 }; g.update(&Inp::keys(&[KeyCode::G], &[KeyCode::G]), 0.016, at(12, 7, 0, 0)); assert_eq!(g.me.ui, Ui::Gift(2, 0));
}
#[test] fn harvest_fair_window_and_prize_once() {
    let mut g = g0(at(14, 12, 0, 0)); g.update(&Inp::none(), 0.016, at(14, 12, 0, 0)); assert!(g.fair()); assert_eq!(tile_at(&g.map, STALL.0, STALL.1), GRASS); assert!(!g.walk_tile(STALL.0, STALL.1));
    g.fair_prize(); assert_eq!(g.w.p.inv.len(), 1); assert_eq!(g.w.p.fair_year, 2026); g.fair_prize(); assert_eq!(g.w.p.inv.len(), 1);
    assert!(!fair_active(11, 17) && !fair_active(10, 14) && fair_active(11, 13)); let mut h = g0(at(20, 12, 0, 0)); h.update(&Inp::none(), 0.016, at(20, 12, 0, 0)); assert!(!h.fair()); h.fair_prize(); assert!(h.w.p.inv.is_empty()); assert!(h.walk_tile(STALL.0, STALL.1));
}
#[test] fn swimming_and_diving() {
    let mut g = g0(at(11, 12, 0, 0)); g.me.pos = vec2(13.5, 25.5); g.me.face = vec2(0., 1.); assert!(!g.swimming());
    for _ in 0..40 { g.step_move(vec2(0., 1.), false, 0.05) } assert!(g.swimming(), "pos {:?}", g.me.pos); assert!(!g.walk_tile(13, 27)); // residents never path into the sea
    g.bubbles = vec![(g.me.pos + vec2(0.4, 0.), 73)]; g.use_tool(); assert!(g.w.p.inv.iter().any(|s| s.def == 73)); assert!(g.bubbles.is_empty());
    g.use_tool(); assert_eq!(g.w.p.inv.len(), 1); g.donate(); assert!(g.w.donated.iter().any(|x| x.0 == 73));
    for _ in 0..60 { g.step_move(vec2(0., -1.), false, 0.05) } assert!(!g.swimming());
}
#[test] fn resident_visits_player_home() {
    let mut g = g0(at(14, 10, 30, 0)); for n in &mut g.w.p.npcs { n.friend = 5 } g.swap(Scene::Room(HOME));
    let mut tries = 0; while g.visit.is_none() && tries < 20000 { g.tick(1. / 60., at(14, 10, 30, 0)); tries += 1 } let (i, b, _) = g.visit.expect("no visit"); assert_eq!(b, HOME); assert!(g.w.p.npcs[i].visited);
    g.me.ipos = g.visit_spot(HOME) + vec2(0.8, 0.); g.me.face = vec2(-1., 0.); assert_eq!(g.target(), Some(Tg::Npc(i))); g.interact(); if let Ui::Talk { lines, .. } = &g.me.ui { assert_eq!(lines[0], VISIT[g.res(i).pers].replace("{p}", "T").replace("{fs}", &format!("{}s", def(g.w.native).name.to_lowercase())).replace("{f}", "")) }
    g.me.ui = Ui::Play; g.swap(Scene::Town); g.tick(0.016, at(14, 10, 30, 1)); assert!(g.visit.is_none()); assert!(!g.npcs[i].inside);
}
#[test] fn roster_defaults_and_mapping() { let g = g0(at(11, 12, 0, 0)); let mut v = serde_json::to_value(g.persist()).unwrap(); v.as_object_mut().unwrap().remove("roster"); let w: World = serde_json::from_value(v).unwrap(); assert_eq!(w.roster, vec![0, 1, 2, 3]);
    let mut g2 = Game::from_world(w, town()); assert_eq!(g2.res(0).name, "Juniper"); g2.w.roster[0] = 4; assert_eq!(g2.res(0).name, "Clover"); assert_eq!(HOUSES[0], 2); }
#[test] fn residents_move_out_and_in_with_stay_option() {
    let mut g = g0(at(1, 12, 0, 0)); let d0 = g.w.day; g.w.p.npcs[1].friend = 2; for k in [0, 2, 3] { g.w.p.npcs[k].friend = 9 }
    let mut day = |g: &mut Game, n: i64| { g.w.day = d0 + n - 1; for p in std::iter::once(&mut g.w.p).chain(g.w.others.iter_mut()) { for s in &mut p.npcs { s.req = None } } g.w.day = d0 + n; g.refresh_moves(); };
    for n in 1..14 { day(&mut g, n); assert_eq!(g.w.moving[1], 0) } day(&mut g, 14); assert_eq!(g.w.moving[1], d0 + 16); assert!(g.w.p.mail.iter().any(|l| l.from == "Bramble"));
    assert!(g.w.moving.iter().enumerate().all(|(k, &m)| k == 1 || m == 0)); // friendly residents never announce
    g.me.ui = Ui::Talk { npc: Some(1), who: "x".into(), lines: vec!["a".into()], i: 0 }; g.update(&Inp::keys(&[KeyCode::X], &[KeyCode::X]), 0.016, at(1, 12, 0, 1)); assert_eq!(g.w.moving[1], 0); // asked to stay
    g.w.p.npcs[1].friend = 0; for n in 15..29 { day(&mut g, n) } assert_ne!(g.w.moving[1], 0); let leave = g.w.moving[1]; day(&mut g, leave - d0);
    assert_eq!(g.w.roster[1], 4); assert_eq!(g.res(1).name, "Clover"); assert_eq!(g.w.p.npcs[1].friend, 0); assert_eq!(g.bname(3), "Clover's House"); assert_eq!(g.w.moving[1], 0);
    let g2 = reload(&g, "move"); assert_eq!(g2.w.roster, g.w.roster);
}
#[test] fn plurals() { assert_eq!(plural("peach"), "peaches"); assert_eq!(plural("cherry"), "cherries"); assert_eq!(plural("apple"), "apples"); }
#[test] fn customize_tint_follows_item_and_decor() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.p.money = 2000; g.add_furn(50); let uid = g.w.p.inv[0].uid;
    assert!(g.customize(0).is_ok()); assert_eq!(Game::tint_of(&g.w.p, uid), 1); assert_eq!(g.w.p.money, 2000 - TINT_FEE);
    g.swap(Scene::Room(HOME)); g.start_place(uid); { let p = g.me.placing.as_mut().unwrap(); p.f.x = 0; p.f.z = 0; } g.place_confirm().unwrap(); assert_eq!(Game::tint_of(&g.w.p, uid), 1); g.pick_furn(uid); assert_eq!(Game::tint_of(&g.w.p, uid), 1);
    assert!(g.customize(1).is_ok()); assert_eq!(g.w.p.wall, 1); assert!(g.customize(2).is_ok()); assert_eq!(g.w.p.floor, 1); g.w.p.money = 0; assert_eq!(g.customize(2), Err("Not enough coins.")); assert_eq!(g.w.p.floor, 1);
    let g2 = reload(&g, "tint"); assert_eq!(Game::tint_of(&g2.w.p, uid), 1); assert_eq!((g2.w.p.wall, g2.w.p.floor), (1, 1));
}
#[test] fn hats_and_hair() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.p.money = 2000; g.buy_cloth(67).unwrap(); g.buy_cloth(68).unwrap(); g.buy_cloth(63).unwrap();
    let u: Vec<u32> = g.w.p.inv.iter().map(|s| s.uid).collect(); g.wear(u[0]); g.wear(u[2]); assert_eq!((g.w.p.hat, g.w.p.shirt), (Some(67), Some(63))); assert_eq!(g.w.p.inv.len(), 1);
    g.wear(u[1]); assert_eq!(g.w.p.hat, Some(68)); assert_eq!(g.w.p.inv[0].def, 67); // straw hat back in pockets
    g.me.ui = Ui::Rack(CLOTHES.len()); let m = g.w.p.money; g.update(&Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]), 0.016, at(11, 12, 0, 1)); assert_eq!(g.w.p.hair, 1); assert_eq!(g.w.p.money, m - HAIR_FEE);
    let g2 = reload(&g, "hat"); assert_eq!((g2.w.p.hat, g2.w.p.hair), (Some(68), 1));
}

// ---- community projects (X-01) ----
#[test] fn community_project_two_players_build_bench() {
    let mut g = two(at(11, 12, 0, 0)); let p = g.w.project.clone().expect("proposal"); assert_eq!(p.kind, 0); assert_eq!(tile_at(&g.map, p.x, p.z), GRASS); assert!(!g.walk_tile(p.x, p.z)); // sign blocks
    let l0 = g.liveliness();
    g.activate(0); for _ in 0..3 { g.add(80); } g.add(20); g.contribute(); let q = g.w.project.clone().unwrap(); assert_eq!(q.need[0].2, 3); assert_eq!(g.w.p.inv.iter().filter(|s| s.def == 80).count(), 0); assert!(g.w.p.inv.iter().any(|s| s.def == 20)); // fruit not needed: kept
    g.activate(1); for _ in 0..4 { g.add(80); } for _ in 0..3 { g.add(81); } g.contribute(); // gives 2 branches + 2 stones only
    assert!(g.w.project.is_none()); assert_eq!(g.w.built.len(), 1); assert_eq!(g.w.p.inv.iter().find(|s| s.def == 80).map(|s| s.qty), Some(2)); assert_eq!(g.w.p.inv.iter().find(|s| s.def == 81).map(|s| s.qty), Some(1));
    assert!(!g.walk_tile(p.x, p.z)); assert!(g.liveliness() > l0, "{} vs {}", g.liveliness(), l0); for b in 0..BLDS.len() { assert!(g.bfs(door_tile(SHOP), door_tile(b), true).is_some()) }
    let g2 = reload(&g, "proj"); assert_eq!(g2.w.built, g.w.built);
}
#[test] fn bridge_project_becomes_walkable_and_materials_once_per_day() {
    let mut g = g0(at(11, 12, 0, 0)); g.w.project = Some(Project { kind: PK_BRIDGE, x: 25, z: 21, need: vec![(0, 1, 0)], by: 0, log: vec![] }); assert!(!g.walk_tile(25, 21));
    g.add(80); g.contribute(); assert!(g.walk_tile(25, 21) && g.walk_tile(26, 21));
    let ti = g.w.trees.iter().position(|t| g.walk_tile(t.x, t.z + 1)).unwrap(); let n0 = g.w.ground.len(); g.w.trees[ti].fruit = false; g.shake_tree_pub(ti); assert_eq!(g.w.ground.len(), n0 + 1); g.shake_tree_pub(ti); assert_eq!(g.w.ground.len(), n0 + 1);
}

// ---- Gullrock Islet (X-03) ----
#[test] fn islet_travel_task_stall_and_safety() {
    let mut g = two(at(16, 12, 0, 0)); let day = g.w.day; let (td, n, _) = islet_task(day);
    g.activate(0); g.me.ui = Ui::Travel(true); g.update_player(&Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]), 0.016); for _ in 0..40 { g.update_player(&Inp::none(), 0.016) }
    assert_eq!(g.me.scene, Scene::Islet); assert_eq!(g.w.p.money, 1000 - FARE); g.activate(1); assert_eq!(g.me.scene, Scene::Town); assert_eq!(g.w.p.money, 1000); g.activate(0);
    assert!(!g.w.iground.is_empty()); for _ in 0..n { g.add(td); } g.add(40); g.pell(); assert_eq!(g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>(), 3); assert!(!g.w.p.inv.iter().any(|s| s.def == td));
    for _ in 0..n { g.add(td); } g.pell(); assert_eq!(g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>(), 3); // once per day
    assert_eq!(g.istall_buy(1), Err("Not enough Sea Glass.")); g.add(85); assert!(g.istall_buy(0).is_ok()); assert!(g.w.p.inv.iter().any(|s| s.def == 58 && s.uid != 0)); assert!(!g.w.p.inv.iter().any(|s| s.def == 85));
    for _ in 0..3 { g.add(85); } g.sell_all(); assert!(g.w.p.inv.iter().any(|s| s.def == 85)); // sea glass is never sold for 0
    let w = g.persist(); assert_eq!(w.p.pos, [dock_stand().x, dock_stand().y]); let inv = w.p.inv.clone(); let g2 = reload(&g, "islet"); assert_eq!(g2.w.p.inv, inv); assert_eq!(g2.me.scene, Scene::Town);
    assert!(!g.walk_tile(DOCK.0, DOCK.1));
}
#[test] fn parcels_birthdays_and_chats() {
    let mut g = g0(at(16, 12, 0, 0)); g.update(&Inp::none(), 0.016, at(16, 12, 0, 0)); for s in &mut g.w.p.npcs { s.req = None } g.w.p.npcs[0].req = Some(86);
    g.talk(0); let (from, to) = g.w.p.parcel.expect("parcel"); assert_eq!(from, 0); assert!(g.w.p.inv.iter().any(|s| s.def == 86)); g.me.ui = Ui::Play; let m = g.w.p.money;
    g.talk(to); assert!(g.w.p.parcel.is_none()); assert_eq!(g.w.p.money, m + 300); assert!(!g.w.p.inv.iter().any(|s| s.def == 86)); g.me.ui = Ui::Play;
    // Otto (character 3) has a birthday on 16 Nov in BDAYS
    let f = g.w.p.npcs[3].friend; g.add(40); let slot = g.w.p.inv.iter().position(|s| s.def == 40).unwrap(); g.give_gift(3, slot); assert!(g.w.p.npcs[3].friend >= f + 5);
    g.talk(3); if let Ui::Talk { lines, .. } = &g.me.ui { assert!(lines.iter().any(|l| l.contains("birthday"))) } g.me.ui = Ui::Play;
    for k in 0..4 { g.npcs[k].inside = false; g.npcs[k].pos = vec2(30.5 + k as f32 * 0.8, 9.5) } let mut n = 0; while g.chats.is_empty() && n < 20000 { g.sim_chats(0.016); n += 1 } assert!(!g.chats.is_empty());
}
#[test] fn footstep_surfaces() { let mut g = g0(at(16, 12, 0, 0)); g.me.pos = vec2(13.5, 17.5); assert_eq!(g.step_sound(), Snd::StepPath); g.me.pos = vec2(14.5, 24.5); assert_eq!(g.step_sound(), Snd::StepSand); g.me.pos = vec2(24.5, 9.5); assert_eq!(g.step_sound(), Snd::StepWood); g.swap(Scene::Room(HOME)); assert_eq!(g.step_sound(), Snd::StepFloor); }

#[test] fn money_projects_leader_funding_rollover() {
    let mut g = two(at(11, 12, 0, 0)); let cost = MPROJECTS[3].1;
    g.activate(1); g.me.pos = vec2(16.5, 20.5); g.me.face = vec2(1., 0.); assert_eq!(g.mp_start(3), Err("Only the town leader can start public projects."));
    g.activate(0); g.me.pos = vec2(13.5, 18.5); g.me.face = vec2(0., 1.); assert!(g.mp_start(3).is_err()); // path tile rejected
    g.me.pos = vec2(16.5, 20.5); g.me.face = vec2(1., 0.); g.w.weeds.retain(|&t| t != (17, 20)); g.w.digs.retain(|&t| t != (17, 20)); g.w.holes.retain(|&t| t != (17, 20)); g.w.ground.retain(|i| tile_of(vec2(i.x, i.z)) != (17, 20)); g.mp_start(3).unwrap(); let m = g.w.mproject.clone().unwrap(); assert!(!g.walk_tile(m.x, m.z));
    for b in 0..BLDS.len() { assert!(g.bfs(door_tile(SHOP), door_tile(b), true).is_some()) }
    g.w.p.money = 30000; g.activate(1); g.w.p.money = 25000; g.activate(0);
    assert_eq!(g.mp_fund(30000), Ok(30000)); g.activate(1); assert_eq!(g.mp_fund(25000), Ok(cost - 30000)); assert_eq!(g.w.p.money, 25000 - (cost - 30000)); assert!(g.mp_fund(1).is_err());
    g.w.p.money = 0; g.activate(0); assert_eq!(g.w.p.money, 0);
    let g2 = reload(&g, "mproj"); assert_eq!(g2.w.mproject, g.w.mproject);
    assert!(g.w.built.iter().all(|b| b.kind != 4)); assert!(g.catch_up(at(12, 6, 0, 1))); assert!(g.w.built.iter().any(|b| b.kind == 4 && (b.x, b.z) == (m.x, m.z))); assert!(g.w.mproject.is_none());
}

#[test] fn policies_one_active_no_stacking() {
    let mut g = two(at(11, 12, 7, 0)); let otto = (0..4).find(|&k| g.w.roster[k] == 3); let jun = (0..4).find(|&k| g.w.roster[k] == 0);
    g.activate(1); g.w.p.money = 99999; assert!(g.enact(Some(0)).is_err()); assert_eq!(g.w.p.money, 99999);
    g.activate(0); g.w.p.money = 100; assert_eq!(g.enact(Some(0)), Err("Not enough coins for the policy fee.")); assert_eq!(g.w.policy, None);
    g.w.p.money = 30000; g.enact(Some(0)).unwrap(); assert_eq!(g.w.p.money, 30000 - POLICIES[0].fee); assert!(g.enact(Some(0)).is_err());
    g.hr = 6; assert!(g.shop_is_open()); g.hr = 21; assert!(g.shop_is_open()); g.hr = 22; assert!(!g.shop_is_open());
    if let Some(j) = jun { g.hr = 5; assert_eq!(g.loc(j), Loc::Out); g.hr = 22; assert_eq!(g.loc(j), Loc::Asleep) }
    let g2 = reload(&g, "pol"); assert_eq!(g2.w.policy, Some(0)); let g3 = reload(&g2, "pol2"); assert_eq!(g3.shop_hours(), (6, 22));
    g.enact(Some(1)).unwrap(); assert_eq!(g.shop_hours(), (8, 24)); g.hr = 7; assert!(!g.shop_is_open()); g.hr = 23; assert!(g.shop_is_open());
    if let Some(j) = jun { g.hr = 5; assert_eq!(g.loc(j), Loc::Asleep); g.hr = 23; assert_eq!(g.loc(j), Loc::Home) }
    if let Some(o) = otto { g.hr = 21; assert_eq!(g.loc(o), Loc::Asleep) }
    g.w.p.money = 20000; g.enact(Some(2)).unwrap(); g.w.weeds.clear(); assert!(g.catch_up(at(12, 6, 0, 1))); assert!(g.w.weeds.is_empty());
    g.enact(None).unwrap(); assert_eq!(g.w.policy, None); assert_eq!(g.shop_hours(), (8, 22));
}

#[test] fn gullrock_activities_coop_and_separate() {
    let mut g = two(at(11, 12, 0, 0)); let day = g.w.day; let glass = |g: &Game| g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>();
    g.activate(0); g.me.scene = Scene::Islet; g.me.ipos = vec2(7.5, 10.5);
    g.activate(1); assert!(g.act_start(1).is_err()); let (m1, gl1) = (g.w.p.money, glass(&g));
    g.activate(0); g.act_start(1).unwrap(); assert_eq!(g.act.as_ref().unwrap().players, vec![0]); assert!(g.act_start(0).is_err());
    for _ in 0..7 { let b = g.act.as_ref().unwrap().ents[0]; g.me.ipos = b; g.act_step() } assert_eq!(g.act.as_ref().unwrap().score, 7);
    g.sim_act(ACTS[1].2 + 0.1); assert!(g.act.as_ref().unwrap().done);
    let g0 = glass(&g); g.act_step(); assert_eq!(glass(&g), g0 + ACT_REWARD[0]); g.act_step(); assert_eq!(glass(&g), g0 + ACT_REWARD[0]);
    g.activate(1); g.act_step(); assert_eq!((g.w.p.money, glass(&g)), (m1, gl1)); g.activate(0);
    let g2 = reload(&g, "act"); assert_eq!(g2.w.p.act_day[1], day); assert!(g2.act.is_none());
    g.sim_act(7.); assert!(g.act.is_none());
    g.act_start(1).unwrap(); g.sim_act(ACTS[1].2 + 0.1); g.act.as_mut().unwrap().score = 20; let g0 = glass(&g); g.act_step(); assert_eq!(glass(&g), g0); g.act = None; // once per day
    g.activate(1); g.me.scene = Scene::Islet; g.me.ipos = vec2(7.5, 10.5); g.activate(0); g.act_start(2).unwrap(); assert_eq!(g.act.as_ref().unwrap().players.len(), 2);
    let r = rings(); g.me.ipos = r[1]; g.act_step(); assert_eq!(g.act.as_ref().unwrap().score, 0);
    g.me.ipos = r[0]; assert!(g.swimming()); g.act_step(); g.activate(1); g.me.ipos = r[1]; g.act_step(); g.activate(0); assert_eq!(g.act.as_ref().unwrap().score, 2);
    assert_eq!(g.act.as_ref().unwrap().mine.len(), 2);
    g.act = None; g.act_start(0).unwrap(); let pen = vec2(PEN.0, PEN.1); g.act.as_mut().unwrap().ents[0] = pen + vec2(PEN.2 + 0.2, 0.); g.me.ipos = pen + vec2(PEN.2 + 0.5, 0.); g.act_step(); assert!(Game::in_pen(g.act.as_ref().unwrap().ents[0]));
    g.swap(Scene::Town); assert!(!g.act.as_ref().unwrap().players.contains(&0));
}

#[test] fn housing_migration_independent_balances_relaunch() {
    // a pre-balance save: P1 already at the largest home with furniture, P2 mid-size; no `debt` field anywhere
    let mut g = two(at(11, 12, 0, 0)); g.w.p.home = 2; g.w.p.room.push(Furn { uid: 77, def: 50, x: 1, z: 1, rot: 0, on: false }); g.w.p.money = 5000;
    g.activate(1); g.w.p.home = 1; g.w.p.money = 20000; g.activate(0);
    let mut v = serde_json::to_value(g.persist()).unwrap(); v["p"].as_object_mut().unwrap().remove("debt"); for o in v["others"].as_array_mut().unwrap() { o.as_object_mut().unwrap().remove("debt"); }
    let p = tmp("predebt"); std::fs::write(&p, v.to_string()).unwrap(); let mut g = Game::from_world(save::load(&p).unwrap(), town());
    assert_eq!((g.w.p.home, g.w.p.debt, g.w.p.money), (2, 0, 5000)); assert!(g.w.p.room.iter().any(|f| f.uid == 77)); assert!(g.expand_home().is_err()); // already max, no retroactive debt
    g.activate(1); assert_eq!((g.w.p.home, g.w.p.debt), (1, 0));
    g.expand_home().unwrap(); assert_eq!((g.w.p.home, g.w.p.debt), (2, EXPAND_COST[1]));
    g.activate(0); assert_eq!((g.w.p.debt, g.w.p.money), (0, 5000)); g.activate(1);
    g.pay_debt(); assert_eq!((g.w.p.money, g.w.p.debt), (5000, 0)); g.activate(0); assert_eq!(g.w.p.money, 5000); // only P2 paid
    // P1 on a fresh profile path: unpaid debt blocks, clearing unlocks
    g.w.p.home = 0; g.expand_home().unwrap(); assert_eq!(g.expand_home(), Err("Pay off your current balance first.")); g.w.p.money = EXPAND_COST[0]; g.pay_debt(); assert_eq!(g.w.p.debt, 0); g.expand_home().unwrap(); assert_eq!((g.w.p.home, g.w.p.debt), (2, EXPAND_COST[1]));
    let g2 = reload(&g, "housing2"); assert_eq!((g2.w.p.home, g2.w.p.debt, g2.w.p.money), (2, EXPAND_COST[1], 0)); assert!(g2.w.p.room.iter().any(|f| f.uid == 77));
    let o = g2.w.others.iter().find(|o| o.id == 1).unwrap(); assert_eq!((o.home, o.debt, o.money), (2, 0, 5000));
}
#[test] fn town_hall_interior_and_civic_menu() {
    let mut g = two(at(11, 12, 0, 0)); let th = BLDS.iter().position(|b| b.kind == BK::Civic).unwrap();
    g.swap(Scene::Room(th)); assert_eq!(g.rdims(th), (7, 5)); assert!(g.room_block(th, 3, 1, 0)); g.me.ipos = vec2(3.5, 2.4); g.me.face = vec2(0., -1.); assert_eq!(g.target(), Some(Tg::Hazel));
    let ent = |g: &mut Game| g.update_player(&Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]), 0.016); let down = |g: &mut Game| g.update_player(&Inp::keys(&[KeyCode::Down], &[KeyCode::Down]), 0.016);
    g.me.ui = Ui::Civic(0); ent(&mut g); assert_eq!(g.me.ui, Ui::Policy(0));
    g.me.ui = Ui::Civic(0); down(&mut g); ent(&mut g); assert_eq!(g.me.ui, Ui::MPick(0)); ent(&mut g); assert_eq!(g.me.mplace, Some(0)); assert!(g.me.trans.is_some()); // heads outside to place
    g.activate(1); g.swap(Scene::Room(th)); g.me.ui = Ui::Civic(1); ent(&mut g); assert_eq!(g.me.ui, Ui::Play); assert_eq!(g.me.mplace, None); // P2 not leader
}
#[test] fn funding_and_activity_edge_cases() {
    let mut g = two(at(11, 12, 0, 0)); let cost = MPROJECTS[0].1;
    g.me.pos = vec2(16.5, 20.5); g.me.face = vec2(1., 0.); g.w.weeds.retain(|&t| t != (17, 20)); g.w.digs.retain(|&t| t != (17, 20)); g.w.holes.retain(|&t| t != (17, 20)); g.w.ground.retain(|i| tile_of(vec2(i.x, i.z)) != (17, 20)); g.mp_start(0).unwrap();
    g.w.p.money = cost - 400; assert_eq!(g.mp_fund(cost), Ok(cost - 400)); assert_eq!(g.w.p.money, 0);
    g.activate(1); g.w.p.money = 700; assert_eq!(g.mp_fund(DONATE_STEP), Ok(400)); assert_eq!(g.w.p.money, 300); // remaining < 1000 is capped
    assert!(g.mp_fund(DONATE_STEP).is_err()); assert_eq!(g.w.p.money, 300);
    // fewer than 1000 coins donates what you have; simultaneous presses in one frame never overpay
    g.w.mproject.as_mut().unwrap().funded = cost - 1500; g.w.p.money = 600; g.activate(0); g.w.p.money = 5000;
    g.w.p.inv.clear(); g.me.ui = Ui::Fund; g.activate(1); g.me.ui = Ui::Fund; g.activate(0);
    let e = || Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]); g.activate(0); g.update_player(&e(), 0.016); g.activate(1); g.update_player(&e(), 0.016);
    assert_eq!(g.w.mproject.as_ref().unwrap().funded, cost); assert_eq!(g.w.p.money, 600 - 500); g.activate(0); assert_eq!(g.w.p.money, 5000 - 1000);
    // late join, one leaves while the other stays
    g.me.scene = Scene::Islet; g.me.ipos = vec2(7.5, 10.5); g.act_start(1).unwrap(); assert_eq!(g.act.as_ref().unwrap().players, vec![0]);
    g.activate(1); g.me.scene = Scene::Islet; g.me.ipos = vec2(7.5, 10.5); g.act_step(); assert_eq!(g.act.as_ref().unwrap().players, vec![0, 1]);
    g.swap(Scene::Town); g.activate(0); assert_eq!(g.act.as_ref().unwrap().players, vec![0]); g.sim_act(1.); assert!(g.act.is_some() && !g.act.as_ref().unwrap().done);
    // full pockets: Sea Glass goes to the mailbox, not lost
    g.w.p.inv = (0..INV_CAP).map(|k| Slot { size_mm: 0, def: 20 + (k % 3) as u16, qty: STACK, uid: 0 }).collect(); let mail0 = g.w.p.mail.len();
    g.act.as_mut().unwrap().score = 99; g.sim_act(99.); g.act_step(); assert_eq!(g.w.p.mail.len() - mail0, ACT_REWARD[2] as usize); assert!(g.w.p.mail.iter().rev().take(4).all(|l| l.item == Some(85)));
    // reload mid-activity: activity not saved, reward flag saved, so restarting cannot pay twice
    let mut g2 = reload(&g, "edge_act"); assert!(g2.act.is_none()); g2.me.scene = Scene::Islet; g2.me.ipos = vec2(7.5, 10.5); g2.act_start(1).unwrap(); g2.act.as_mut().unwrap().score = 99; g2.sim_act(99.); let m = g2.w.p.mail.len(); g2.act_step(); assert_eq!(g2.w.p.mail.len(), m);
}

#[test] fn x04_collections_art_critters() {
    let mut g = two(at(14, 12, 0, 0)); g.mo = 11; g.dom = 14; // 2026-11-14 is a Saturday, but the Harvest Fair (13-16) takes the stall
    assert_eq!(weekday(g.w.day), 5); assert!(!g.art_open()); assert!(g.art_buy().is_err());
    let mut g = two(at(21, 12, 0, 0)); g.mo = 11; g.dom = 21; assert!(g.art_open()); let d = art_of_week(g.w.day);
    g.w.p.money = 100; assert_eq!(g.art_buy(), Err("Not enough coins."));
    g.w.p.money = 6000; assert_eq!(g.art_buy(), Ok(def(d).name)); assert_eq!(g.w.p.money, 6000 - ART_PRICE); assert!(g.art_buy().is_err()); // once per week per player
    g.activate(1); g.w.p.money = 3000; assert!(g.art_buy().is_ok()); assert_eq!(g.w.p.money, 500); g.activate(0);
    let g2 = reload(&g, "art"); assert_eq!(g2.w.p.art_week, g.w.p.art_week);
    g.donate(); assert!(g.w.donated.iter().any(|x| x.0 == d)); assert_eq!(g.donated_of(K::Art), 1);
    assert_ne!(art_of_week(g.w.day), art_of_week(g.w.day + 7));
    // collections: three pieces complete a set
    g.w.p.room.clear(); for (k, &d) in SETS[0].1.iter().enumerate().take(5) { g.w.p.room.push(Furn { uid: 900 + k as u32, def: d, x: k as i32, z: 0, rot: 0, on: false }) }
    assert!(g.sets_done().is_empty()); g.w.p.room.push(Furn { uid: 999, def: SETS[0].1[5], x: 0, z: -1, rot: 0, on: false }); assert_eq!(g.sets_done(), vec!["Meadow collection"]);
    assert!(FURN_IDS.contains(&95) && def(92).act == Act::Lamp && model(93) == 51);
    // critters: rain snail only in rain filter, Dune Beetle is islet-only, river-mouth goby, winter smelt
    assert!(avail(def(18), 1, 12) && !avail(def(18), 6, 12)); assert_eq!(def(9).hab, 4); assert_eq!(def(19).hab, 3);
    assert_eq!(DEFS.iter().filter(|d| d.k == K::Bug).count(), 10); assert_eq!(DEFS.iter().filter(|d| d.k == K::Fish).count(), 13); assert_eq!(ARTS.len(), 8); assert!(SETS.iter().all(|s| s.1.len() == 6));
}

#[test] fn placement_never_blocks_door_or_escape() {
    let mut g = two(at(11, 12, 0, 0)); home(&mut g); assert_eq!(g.rdims(HOME), (4, 4)); g.w.p.room.clear(); g.me.ipos = vec2(1.5, 0.5);
    let place = |g: &mut Game, x: i32, z: i32| -> Result<(), &'static str> { assert!(g.add_furn(92)); let uid = g.w.p.inv.last().unwrap().uid; g.start_place(uid); let p = g.me.placing.as_mut().unwrap(); p.f.x = x; p.f.z = z; p.f.rot = 0;
        let r = g.place_ok(&g.me.placing.as_ref().unwrap().f); g.update_player(&Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]), 0.016); if g.me.placing.is_some() { g.place_cancel(); g.me.placing = None; } r };
    assert_eq!(place(&mut g, 2, 3), Err("Keep the doorway clear.")); assert!(!g.w.p.room.iter().any(|f| (f.x, f.z) == (2, 3)));
    assert_eq!(place(&mut g, 1, 0), Err("You're standing there."));
    for x in [0, 1, 3] { assert_eq!(place(&mut g, x, 2), Ok(())) } assert_eq!(g.w.p.room.len(), 3);
    assert_eq!(place(&mut g, 2, 2), Err("That would block your way out.")); assert_eq!(g.w.p.room.len(), 3);
}
#[test] fn new_critters_rules_capture_dex_donation() {
    let (rain, dry) = (Game::bug_pool(true, 6, 12), Game::bug_pool(false, 6, 12)); assert!(rain.contains(&8) && !dry.contains(&8)); assert!(!Game::bug_pool(true, 1, 12).contains(&8)); // Mar-Nov only
    assert!(!rain.contains(&9) && Game::islet_bug_pool(6, 12).contains(&9) && !Game::islet_bug_pool(6, 22).contains(&9)); // Gullrock only, 8-20h
    assert!(Game::fish_pool(1, false, 1, 12).contains(&18) && !Game::fish_pool(1, false, 6, 12).contains(&18) && !Game::fish_pool(2, false, 1, 12).contains(&18)); // winter river
    assert!(Game::fish_pool(1, true, 6, 12).contains(&19) && !Game::fish_pool(1, false, 6, 12).contains(&19) && !Game::fish_pool(2, false, 6, 12).contains(&19)); // river mouth only
    let mut g = g0(at(11, 12, 0, 0)); g.w.p.inv.clear(); g.w.p.tools = vec![60, 61]; g.w.p.equip = Some(60); g.me.scene = Scene::Islet; g.me.ipos = vec2(7.5, 8.5); g.me.face = vec2(1., 0.);
    g.ibugs = vec![(9, vec2(8.2, 8.5), 0.)]; g.use_tool(); assert!(g.w.p.inv.iter().any(|s| s.def == 9)); assert!(g.w.p.seen.contains(&9)); assert!(g.ibugs.is_empty());
    for d in [8, 18, 19] { g.add(d); } g.me.scene = Scene::Town; g.donate(); for d in [8, 9, 18, 19] { assert!(g.w.donated.iter().any(|x| x.0 == d), "{d} not donated") }
}
#[test] fn art_week_limit_per_player_survives_reload() {
    let mut g = two(at(21, 12, 0, 0)); g.mo = 11; g.dom = 21; g.w.p.money = 9000; g.art_buy().unwrap(); g.activate(1); g.w.p.money = 9000; g.art_buy().unwrap(); g.activate(0);
    let mut g2 = reload(&g, "artwk"); g2.mo = 11; g2.dom = 21; assert!(g2.art_buy().is_err()); g2.activate(1); assert_eq!(g2.art_buy(), Err("You've already bought this week's painting.")); assert_eq!(g2.w.p.money, 9000 - ART_PRICE);
    let mut g3 = reload(&g2, "artwk2"); g3.w.day += 7; g3.mo = 11; g3.dom = 28; assert!(g3.art_buy().is_ok());
}
#[test] #[ignore] fn find_camp_site() {
    let g = g0(at(11, 12, 0, 0));
    for z in 1..H as i32 - 4 { for x in 1..W as i32 - 4 {
        let area: Vec<(i32, i32)> = (0..4).flat_map(|dx| (0..4).map(move |dz| (x + dx, z + dz))).collect();
        if area.iter().all(|&(a, b)| g.tile(a, b) == GRASS && g.walk_tile(a, b) && !g.w.trees.iter().any(|t| (t.x, t.z) == (a, b))) && BLDS.iter().all(|bl| (bl.x - x).abs() > 3 || (bl.z - z).abs() > 3) { println!("cand {x},{z}") } } }
}

#[test] fn campground_visitors_stock_favor_two_players() {
    let mut g = two(at(11, 12, 0, 0)); let v = g.visitor(); assert_eq!(v, visitor_of(g.w.day));
    for t in CAMP_TENT.iter().chain([CAMP_FIRE, CAMP_CRATE, CAMP_SIGN].iter()) { assert!(!g.walk_tile(t.0, t.1)) }
    for b in 0..BLDS.len() { assert!(g.bfs(door_tile(SHOP), door_tile(b), true).is_some()) } assert!(g.bfs(door_tile(SHOP), (32, 21), true).is_some());
    assert_ne!(visitor_of(g.w.day), visitor_of(g.w.day + 1)); g.hr = 23; assert!(g.camp_buy(0).is_err()); g.hr = 12;
    let d = VISITORS[v].goods[0]; g.w.p.money = 100; assert_eq!(g.camp_buy(0), Err("Not enough coins."));
    g.w.p.money = 9000; assert_eq!(g.camp_buy(0), Ok(def(d).name)); assert_eq!(g.w.p.money, 9000 - def(d).buy);
    g.activate(1); g.w.p.money = 9000; assert_eq!(g.camp_buy(0), Err("Sold out for today.")); assert_eq!(g.w.p.money, 9000); // shared stock, no double sale
    g.w.p.inv = (0..INV_CAP).map(|k| Slot { size_mm: 0, def: 20, qty: 1, uid: 0 + k as u32 * 0 }).collect(); assert_eq!(g.camp_buy(1), Err("Your pockets are full.")); assert_eq!(g.w.p.money, 9000); g.w.p.inv.clear();
    // favor: needs items, gives reward once, saved per player
    let (fd, fq) = g.favor_item(); assert!(g.camp_favor().is_err()); for _ in 0..fq { g.add(fd); }
    assert_eq!(g.camp_favor(), Ok(def(VISITORS[v].reward).name)); assert!(g.w.p.inv.iter().any(|s| s.def == VISITORS[v].reward)); assert!(g.camp_favor().is_err());
    g.activate(0); assert!(!g.w.p.camp_favors.contains(&(v as u8)));
    let g2 = reload(&g, "camp"); assert_eq!(g2.w.camp_sold, vec![d]); assert_eq!(g2.visitor(), v); assert!(g2.w.others[0].camp_favors.contains(&(v as u8)));
    assert!(g.catch_up(at(12, 6, 0, 1))); assert!(g.w.camp_sold.is_empty()); assert_ne!(g.visitor(), v);
}
#[test] fn initiatives_tasks_claims_kiosk_two_players() {
    let mut g = two(at(11, 12, 0, 0)); assert!(g.catch_up(at(12, 6, 0, 1)));
    assert_eq!(g.w.p.tasks.len(), 3); g.activate(1); assert_eq!(g.w.p.tasks.len(), 3); g.activate(0); assert!(g.w.wtask.is_some());
    for t in &g.w.p.tasks { assert!(g.task_ok(t.t)) } assert!(TASKS.len() >= 12 && KIOSK_ITEMS.len() >= 8);
    // force a known task and progress it through normal events
    g.w.p.tasks[0] = PTask { t: 5, prog: 0, claimed: false }; assert_eq!(g.claim(0), Err("Not finished yet."));
    g.ev(Ev::Talk, 1); g.ev(Ev::Talk, 5); assert_eq!(g.w.p.tasks[0].prog, 3); assert_eq!(g.claim(0), Ok(2)); assert_eq!(g.claim(0), Err("Already claimed.")); assert_eq!(g.w.p.coupons, 2);
    g.activate(1); assert_eq!(g.w.p.coupons, 0); g.activate(0);
    // weekly: shared progress, each player claims once
    let wt = g.w.wtask.as_ref().unwrap().t; let (e, n, _) = WTASKS[wt]; g.ev(e, n / 2); g.activate(1); g.ev(e, n); assert_eq!(g.claim(3), Ok(WEEKLY_COUPONS)); assert!(g.claim(3).is_err()); g.activate(0); assert_eq!(g.claim(3), Ok(WEEKLY_COUPONS));
    let g2 = reload(&g, "init"); assert_eq!(g2.w.p.coupons, 2 + WEEKLY_COUPONS); assert!(g2.w.p.tasks[0].claimed); assert_eq!(g2.w.wtask.as_ref().unwrap().claimed.len(), 2);
    let mut g3 = g2; assert!(g3.claim(0).is_err() && g3.claim(3).is_err()); // no duplicate claims after reload
    // kiosk: deduct only on success; ferry pass works for travel
    g3.w.p.coupons = 4; assert!(g3.kiosk_buy(1).is_err()); assert_eq!(g3.w.p.coupons, 4); assert_eq!(g3.kiosk_buy(6), Ok("Ferry Pass")); assert_eq!((g3.w.p.coupons, g3.w.p.ferry), (0, 1));
    g3.w.p.coupons = 50; g3.w.p.inv = (0..INV_CAP).map(|_| Slot { size_mm: 0, def: 20, qty: STACK, uid: 0 }).collect(); assert_eq!(g3.kiosk_buy(0), Err("Your pockets are full.")); assert_eq!(g3.kiosk_buy(7), Err("Your pockets are full.")); assert_eq!(g3.w.p.coupons, 50);
    g3.w.p.inv.clear(); assert!(g3.kiosk_buy(7).is_ok()); assert_eq!(g3.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum::<u16>(), 3);
    // simultaneous claim input from both players in one frame
    let mut g = two(at(11, 12, 0, 0)); assert!(g.catch_up(at(12, 6, 0, 1))); let wt = g.w.wtask.as_ref().unwrap().t; g.ev(WTASKS[wt].0, WTASKS[wt].1);
    let th = BLDS.iter().position(|b| b.kind == BK::Civic).unwrap(); let e = || Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]);
    for id in [0, 1] { g.activate(id); g.swap(Scene::Room(th)); g.me.ui = Ui::Kiosk(3) } for _ in 0..3 { for id in [0, 1] { g.activate(id); g.me.ui = Ui::Kiosk(3); g.update_player(&e(), 0.016) } }
    g.activate(0); assert_eq!(g.w.p.coupons, WEEKLY_COUPONS); g.activate(1); assert_eq!(g.w.p.coupons, WEEKLY_COUPONS);
}
#[test] fn task_events_from_normal_actions() {
    let mut g = g0(at(11, 12, 0, 0)); assert!(g.catch_up(at(12, 6, 0, 1))); g.w.p.tasks = vec![PTask { t: 7, prog: 0, claimed: false }, PTask { t: 4, prog: 0, claimed: false }, PTask { t: 11, prog: 0, claimed: false }];
    for it in g.w.ground.clone().iter().filter(|i| def(i.def).k == K::Shell).take(2) { g.me.pos = vec2(it.x, it.z) - vec2(0., 0.6); g.me.face = vec2(0., 1.); if g.target().is_some() { g.interact() } }
    g.w.p.inv = vec![Slot { size_mm: 0, def: 41, qty: 2, uid: 0 }]; g.sell_all(); assert_eq!(g.w.p.tasks[1].prog, 500);
    g.bubbles = vec![(vec2(5.5, 5.5), 70)]; g.me.pos = vec2(5.5, 5.5); g.dive(); assert_eq!(g.w.p.tasks[2].prog, 1);
}
#[test] fn version_and_help_present() { assert!(VERSION.starts_with("v1.1.0")); assert!(HELP_KEYS.len() >= 8 && HELP_PAD.len() >= 8 && GUIDE.len() >= 8); }
#[test] fn release_content_acquisition_audit() {
    let g = g0(at(11, 12, 0, 0)); let mut src: Vec<u16> = vec![];
    for m in 1..=12 { for h in 0..24 { for rain in [false, true] { src.extend(Game::bug_pool(rain, m, h)) } src.extend(Game::islet_bug_pool(m, h)); for hab in [1, 2] { for mouth in [false, true] { src.extend(Game::fish_pool(hab, mouth, m, h)) } } } }
    src.extend(FRUITS); src.extend([40, 41, 42, 84, 82, 75, 85, 86, 80, 81, 45, 46, 47, 70, 71, 72, 73, 74]); src.extend(FOSSILS); src.extend(FURN_IDS); src.extend(ISTALL_ITEMS.iter().map(|x| x.0)); src.extend(CLOTHES); src.extend(ARTS);
    for v in &VISITORS { src.extend(v.goods); src.push(v.reward) } src.extend(CONTEST_PLAQUES); src.extend(KEEPSAKES); src.push(150); src.extend(SPECIALS); src.extend(SEEDS); src.extend(FLOWERS); src.push(CAN); src.push(169); src.extend(FESTS.iter().map(|f| f.reward)); src.extend(KIOSK_ITEMS.iter().map(|x| x.0)); src.extend([60, 61, 62]);
    let missing: Vec<&str> = DEFS.iter().filter(|d| !src.contains(&d.id)).map(|d| d.name).collect(); assert!(missing.is_empty(), "no acquisition path: {missing:?}");
    // the islet offers every non-native fruit over consecutive days
    let mut g = g; let mut seen = vec![]; for k in 0..3 { g.w.day += 1; g.refresh_islet(); let _ = k; seen.extend(g.w.iground.iter().map(|i| i.def).filter(|d| FRUITS.contains(d))) } for f in FRUITS { if f != g.w.native { assert!(seen.contains(&f), "fruit {f} never on Gullrock") } }
}
#[test] fn pell_reward_not_lost_with_full_pockets() {
    let mut g = g0(at(11, 12, 0, 0)); g.me.scene = Scene::Islet; let (d, n, _) = islet_task(g.w.day);
    g.w.p.inv = vec![Slot { size_mm: 0, def: d, qty: n, uid: 0 }]; g.w.p.inv.extend((1..INV_CAP).map(|_| Slot { size_mm: 0, def: 20, qty: STACK, uid: 0 })); let m0 = g.w.p.mail.len();
    g.pell(); let glass: u16 = g.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum(); assert_eq!(glass as usize + g.w.p.mail.len() - m0, 3);
}
#[test] #[ignore] fn migrate_real_save_copy() {
    let p = std::env::var("HB_SAVE_COPY").unwrap(); let w = save::load(&p).expect("loads"); let (money, home, n) = (w.p.money, w.p.home, w.others.len());
    let mut g = Game::from_world(w, town()); assert_eq!((g.w.p.money, g.w.p.home, g.w.others.len()), (money, home, n));
    g.update_player(&Inp::none(), 0.016); assert_eq!(g.w.p.tasks.len(), 3); println!("migrated: money {money} home {home} others {n} tasks {:?} visitor {}", g.w.p.tasks.iter().map(|t| TASKS[t.t].2).collect::<Vec<_>>(), VISITORS[g.visitor()].name);
}

#[test] fn resident_depth_tiers_context_keepsake() {
    let mut g = two(at(11, 12, 0, 0)); assert_eq!((tier(0), tier(5), tier(15), tier(30), tier(50)), (0, 1, 2, 3, 4));
    let c = g.ctx_lines(0); assert!(c.len() >= 4); assert!(c.iter().any(|l| SEASON_LINES.iter().any(|s| s.contains(&l.as_str()))));
    g.npcs[0].inside = false; g.w.p.npcs[0].friend = 49; g.w.p.npcs[0].req = None; g.w.p.parcel = None; g.talk(0); assert!(g.w.p.keepsakes.is_empty());
    let Ui::Talk { lines, .. } = &g.me.ui else { panic!("ui {:?}", g.me.ui) }; assert!(lines.len() >= 3, "{lines:?}");
    g.w.p.npcs[0].talked = false; g.w.p.npcs[0].friend = 50; g.w.p.npcs[0].req = None; g.talk(0); let ch = g.w.roster[0]; assert!(g.w.p.keepsakes.contains(&(ch as u8))); assert!(g.w.p.inv.iter().any(|s| s.def == KEEPSAKES[ch]));
    g.w.p.npcs[0].talked = false; g.talk(0); assert_eq!(g.w.p.inv.iter().filter(|s| s.def == KEEPSAKES[ch]).count(), 1); // once
    g.activate(1); assert!(g.w.p.keepsakes.is_empty()); g.activate(0); let g2 = reload(&g, "keep"); assert_eq!(g2.w.p.keepsakes, vec![ch as u8]);
}

#[test] fn festivals_tasks_rewards_yearly() {
    let mut g = two(at(11, 12, 0, 0)); assert_eq!(fest_of(4, 11), Some(0)); assert_eq!(fest_of(7, 25), Some(1)); assert_eq!(fest_of(12, 24), Some(2)); assert_eq!(fest_of(1, 1), Some(3)); assert_eq!(fest_of(6, 1), None);
    for f in &FESTS { assert!(!fair_active(f.m, f.d0)) }
    g.mo = 4; g.dom = 11; assert!(g.fest_claim().is_err()); for _ in 0..2 { g.add(g.w.native); } assert_eq!(g.fest_claim(), Ok("Blossom Lantern")); assert!(g.fest_claim().is_err());
    assert!(!g.walk_tile(STALL.0, STALL.1)); assert!(!g.art_open());
    g.activate(1); assert!(g.fest_claim().is_err()); g.activate(0); // other player still needs to do it
    g.mo = 1; g.dom = 1; assert!(g.fest_claim().is_err()); for n in &mut g.w.p.npcs { n.talked = true } assert_eq!(g.fest_claim(), Ok("Sunrise Banner"));
    g.mo = 7; g.dom = 24; g.w.p.inv.clear(); g.add(10); g.w.p.inv.extend((1..INV_CAP).map(|_| Slot { size_mm: 0, def: 20, qty: STACK, uid: 0 })); assert_eq!(g.fest_claim(), Ok("Paper Boat Model")); assert!(!g.w.p.inv.iter().any(|s| s.def == 10)); // the fish frees the slot for the keepsake
    let g2 = reload(&g, "fest"); assert_eq!(g2.w.p.fest_years.len(), 3); g.yr += 1; g.mo = 4; g.dom = 10; g.w.p.inv.clear(); for _ in 0..2 { g.add(g.w.native); } assert!(g.fest_claim().is_ok()); // next year again
}

#[test] fn daily_surprises_shared_and_saved() {
    let mut g = two(at(11, 12, 0, 0)); assert!(g.catch_up(at(12, 6, 0, 1))); assert!(g.w.surprise.is_some());
    // bottle -> mailbox letter with coins, gone for the other player
    g.w.surprise = Some(Surprise { kind: 0, x: 5, z: 25, owner: 0, day: g.w.day }); let m = g.w.p.mail.len(); g.take_surprise(); assert_eq!(g.w.p.mail.len(), m + 1); assert_eq!(g.w.p.mail.last().unwrap().coins, BOTTLE_COINS); assert!(g.w.surprise.is_none());
    g.activate(1); let m1 = g.w.p.mail.len(); g.take_surprise(); assert_eq!(g.w.p.mail.len(), m1); g.activate(0);
    // lost scarf -> return to its owner for friendship + coins, once
    g.w.surprise = Some(Surprise { kind: 1, x: 6, z: 20, owner: 2, day: g.w.day }); g.take_surprise(); assert_eq!(g.w.p.lost_owner, Some(2)); assert!(g.w.p.inv.iter().any(|s| s.def == 150));
    let g2 = reload(&g, "surp"); assert_eq!(g2.w.p.lost_owner, Some(2));
    let (f0, c0) = (g.w.p.npcs[2].friend, g.w.p.money); g.w.p.npcs[1].req = None; g.talk(1); assert!(g.w.p.inv.iter().any(|s| s.def == 150)); // wrong resident keeps it
    g.w.p.npcs[2].req = None; g.talk(2); assert!(!g.w.p.inv.iter().any(|s| s.def == 150)); assert_eq!(g.w.p.money, c0 + RETURN_COINS); assert!(g.w.p.npcs[2].friend >= f0 + 4); assert_eq!(g.w.p.lost_owner, None);
    // drifting gift -> furniture; full pockets leave it in place
    g.w.surprise = Some(Surprise { kind: 2, x: 6, z: 20, owner: 0, day: g.w.day }); g.w.p.inv = (0..INV_CAP).map(|_| Slot { size_mm: 0, def: 20, qty: STACK, uid: 0 }).collect(); g.take_surprise(); assert!(g.w.surprise.is_some());
    g.w.p.inv.clear(); g.take_surprise(); assert!(g.w.surprise.is_none() && g.w.p.inv.iter().any(|s| def(s.def).k == K::Furn));
}

#[test] fn shop_renovation_tiers_special_extra_stock() {
    let mut g = two(at(11, 12, 0, 0)); assert_eq!(g.shop_tier(), 0); assert!(g.buy(8).is_err() && g.buy(7).is_err());
    g.w.p.money = 99999; let b = g.w.p.money; g.buy(0).unwrap(); assert_eq!(g.w.spent, b - g.w.p.money);
    g.w.spent = SHOP_TIERS[0]; assert_eq!(g.shop_tier(), 1); let (d, p, sold) = g.display_item(8); assert!(SPECIALS.contains(&d) && !sold);
    let m = g.w.p.money; g.buy(8).unwrap(); assert_eq!(g.w.p.money, m - p); assert!(g.display_item(8).2); g.activate(1); g.w.p.money = 99999; assert_eq!(g.buy(8), Err("Sold out!")); g.activate(0); // weekly, shared
    let g2 = reload(&g, "shop"); assert!(g2.display_item(8).2);
    let m0 = g.w.p.mail.len(); assert!(g.catch_up(at(12, 6, 0, 1))); assert!(g.w.p.mail.iter().skip(m0).any(|l| l.from == "Mr. Tumble")); assert_eq!(g.w.stock.len(), 5);
    g.w.spent = SHOP_TIERS[1]; assert!(g.catch_up(at(13, 6, 0, 1))); assert_eq!(g.w.stock.len(), 6); let (d7, _, _) = g.display_item(7); assert_eq!(d7, g.w.stock[5].def); g.buy(7).unwrap(); assert!(g.w.stock[5].sold);
    let th = BLDS.iter().position(|b| b.kind == BK::Shop).unwrap(); assert!(g.room_block(th, EXTRA_DISPLAY.0, EXTRA_DISPLAY.1, 0) && g.room_block(th, SPECIAL_DISPLAY.0, SPECIAL_DISPLAY.1, 0));
    g.w.day += 7; assert!(!g.display_item(8).2); // next week's special
}
#[test] fn festive_music_on_event_days() {
    let mut g = g0(at(11, 12, 0, 0)); g.mo = 6; g.dom = 1; assert_ne!(crate::audio::variant(&g), crate::audio::Var::Festive); g.mo = 4; g.dom = 11; assert_eq!(crate::audio::variant(&g), crate::audio::Var::Festive);
    let (n, f) = (crate::audio::compose(2, crate::audio::Var::Normal), crate::audio::compose(2, crate::audio::Var::Festive)); assert_eq!(n.len(), f.len()); assert!(n.iter().zip(&f).any(|(a, b)| (a - b).abs() > 1e-3));
}

#[test] fn gardening_plant_water_grow_pick_cross() {
    let mut g = two(at(11, 12, 0, 0)); g.w.p.money = 5000; g.me.ui = Ui::Keeper(0);
    let items = g.keeper_items(); assert!(items.iter().any(|x| x.1 == 5)); g.add(160); g.add(162); g.w.p.tools.push(CAN);
    let site = (16, 20); g.w.weeds.clear(); g.w.digs.clear(); g.w.holes.clear(); g.w.ground.clear(); g.w.surprise = None; g.me.pos = vec2(15.5, 20.5); g.me.face = vec2(1., 0.);
    assert!(g.plant_ok(site)); g.plant(site).unwrap(); assert!(g.plant(site).is_err()); g.me.pos = vec2(15.5, 21.5); g.plant((16, 21)).unwrap(); assert_eq!(g.w.garden.len(), 2);
    assert!(!g.w.p.inv.iter().any(|s| SEEDS.contains(&s.def)));
    // watering: front tile only at level 0
    g.me.pos = vec2(15.5, 20.5); assert_eq!(g.water(), 1); assert_eq!(g.water(), 0);
    let d = g.w.day; let rain_y = rainy(d); assert!(g.catch_up(at(12, 6, 0, 1))); assert_eq!(g.w.garden[0].stage, 1); assert_eq!(g.w.garden[1].stage, if rain_y { 1 } else { 0 });
    for _ in 0..2 { for p in &mut g.w.garden { p.watered = g.w.day } g.w.day += 1; g.grow_garden() } assert!(g.w.garden.iter().take(2).all(|p| p.stage == 2));
    let ix = 0; g.pick_flower(ix); assert!(g.w.p.inv.iter().any(|s| s.def == 164)); assert_eq!(g.w.garden[0].stage, 1);
    // crossing: two adjacent blooms of different colours eventually make a Starfleck
    g.w.garden[0].stage = 2; let mut crossed = false; for _ in 0..60 { g.w.day += 1; for p in &mut g.w.garden { p.watered = g.w.day - 1 } g.grow_garden(); if g.w.garden.iter().any(|p| p.kind == 4) { crossed = true; break } } assert!(crossed);
    let g2 = reload(&g, "garden"); assert_eq!(g2.w.garden, g.w.garden);
    g.activate(1); assert!(!g.w.garden.is_empty()); g.activate(0); // shared town garden
}
#[test] fn tool_upgrades_costs_and_effects() {
    let mut g = two(at(11, 12, 0, 0)); assert_eq!(g.tool_level(60), 0); assert!(g.upgrade_tool(62).is_err()); // no shovel yet
    g.w.p.money = 1000; assert!(g.upgrade_tool(60).is_err()); g.w.p.money = 10000; for _ in 0..3 { g.add(81); } assert_eq!(g.upgrade_tool(60), Ok(1)); assert_eq!(g.w.p.money, 10000 - UPGRADE[0].0);
    assert!(!g.w.p.inv.iter().any(|s| s.def == 81)); assert!(g.upgrade_tool(60).is_err()); for _ in 0..5 { g.add(81); } for _ in 0..3 { g.add(85); } assert_eq!(g.upgrade_tool(60), Ok(2)); assert!(g.upgrade_tool(60).is_err());
    g.activate(1); assert_eq!(g.tool_level(60), 0); g.activate(0); let g2 = reload(&g, "tools"); assert_eq!(g2.tool_level(60), 2);
    // net reach: a bug 1.3 tiles in front is caught at level 2 only
    g.w.p.equip = Some(60); g.me.pos = vec2(10.5, 12.5); g.me.face = vec2(1., 0.); g.bugs = vec![(1, g.me.pos + vec2(0.7 + 1.3, 0.), 0.)]; g.use_tool(); assert!(g.bugs.is_empty());
    g.activate(1); g.w.p.equip = Some(60); g.me.pos = vec2(10.5, 12.5); g.me.face = vec2(1., 0.); g.bugs = vec![(1, g.me.pos + vec2(0.7 + 1.3, 0.), 0.)]; g.use_tool(); assert_eq!(g.bugs.len(), 1);
    // watering can level 2 waters a 3x3 patch
    g.activate(0); g.w.p.tools.push(CAN); while g.w.p.tool_lv.len() < 4 { g.w.p.tool_lv.push(0) } g.w.p.tool_lv[3] = 2; g.w.garden = (0..9).map(|k| Plant { x: 20 + k % 3, z: 10 + k / 3, kind: 0, stage: 0, watered: 0 }).collect();
    g.me.pos = vec2(20.3, 11.5); g.me.face = vec2(1., 0.); assert_eq!(g.water(), 9);
}

#[test] fn home_evaluation_letters_and_paint() {
    let mut g = two(at(11, 12, 0, 0)); let (s0, tips) = Game::home_score(&g.w.p); assert_eq!(Game::home_tier(s0), 0); assert_eq!(tips.len(), 2);
    for (k, &d) in SETS[0].1.iter().enumerate() { g.w.p.room.push(Furn { uid: 700 + k as u32, def: d, x: k as i32 % 4, z: if def(d).wall { -1 } else { k as i32 / 4 }, rot: 0, on: false }) }
    g.w.p.wall = 1; let (s1, _) = Game::home_score(&g.w.p); assert!(Game::home_tier(s1) >= 1, "score {s1}");
    // Monday letter pays the new tiers once
    let mut d = 12; while weekday(chrono::NaiveDate::from_ymd_opt(2026, 11, d).unwrap().num_days_from_ce() as i64) != 0 { d += 1 } let m0 = g.w.p.mail.len(); assert!(g.catch_up(at(d, 6, 0, 1)));
    let l = g.w.p.mail.iter().skip(m0).find(|l| l.from == "Hollowbrook Home Society").expect("letter"); assert!(l.coins >= HOME_TIERS[1].2); assert!(g.w.p.home_best >= 1);
    g.activate(1); assert!(g.w.p.mail.iter().any(|l| l.from == "Hollowbrook Home Society" && l.coins == 0)); g.activate(0); // P2 scored separately (bare home, no reward)
    let best = g.w.p.home_best; assert!(g.catch_up(at(d + 7, 6, 0, 1))); assert_eq!(g.w.p.mail.iter().filter(|l| l.from == "Hollowbrook Home Society").last().unwrap().coins, 0); assert_eq!(g.w.p.home_best, best); // no repeat reward
    // repaint
    g.w.p.money = 100; assert!(g.repaint(2).is_err()); g.w.p.money = 5000; g.repaint(2).unwrap(); assert_eq!((g.w.p.paint, g.w.p.money), (Some(2), 5000 - PAINT_FEE)); assert!(g.repaint(2).is_err());
    let g2 = reload(&g, "paint"); assert_eq!(g2.w.p.paint, Some(2)); assert_eq!(g2.w.p.home_best, best);
}

#[test] fn lan_host_client_loopback() {
    use crate::net::*;
    let enc_dec = dec(&enc(&Inp::keys(&[KeyCode::W, KeyCode::LeftShift], &[KeyCode::E]), false)); assert!(enc_dec.down.contains(&KeyCode::W) && enc_dec.pressed.contains(&KeyCode::E));
    let mut host_g = g0(at(11, 12, 0, 0)); let mut h = Host::start_on("Tester", 0).expect("host"); let port = h.port();
    let mut c = Client::connect(std::net::SocketAddr::from(([127, 0, 0, 1], port))).expect("connect");
    let t0 = std::time::Instant::now(); let mut joined = false; while t0.elapsed().as_secs() < 5 && !joined { for e in h.poll() { if let HostEv::Joined = e { joined = true } } std::thread::sleep(std::time::Duration::from_millis(10)) } assert!(joined);
    host_g.join("Player 2"); host_g.remote = Some(1); h.welcome(1);
    while c.id.is_none() && t0.elapsed().as_secs() < 5 { c.poll(); std::thread::sleep(std::time::Duration::from_millis(10)) } assert_eq!(c.id, Some(1));
    // client walks right for a while: the host moves Player 2 only
    host_g.activate(1); let start = host_g.me.pos; host_g.activate(0); let p1 = host_g.me.pos;
    for _ in 0..40 { c.send(&Inp::keys(&[KeyCode::S], &[]), false); std::thread::sleep(std::time::Duration::from_millis(5)); h.poll(); host_g.activate(1); let i = h.take_input(); host_g.update_player(&i, 1. / 60.); host_g.activate(0); }
    host_g.activate(1); let moved = host_g.me.pos; host_g.activate(0); assert!(moved.y > start.y + 0.3, "remote player did not move: {start:?} -> {moved:?}"); assert_eq!(host_g.me.pos, p1);
    // snapshot reaches the client and shows Player 2 as the viewed player with the host hidden
    std::thread::sleep(std::time::Duration::from_millis(60)); assert!(h.due()); h.send(snap(&host_g, at(11, 12, 0, 0), vec![Snd::Coin]));
    let mut got = None; while got.is_none() && t0.elapsed().as_secs() < 8 { c.poll(); got = c.take_snap(); std::thread::sleep(std::time::Duration::from_millis(10)) }
    let s = got.expect("snapshot"); let mut cg = Game::from_world(s.w.clone(), town()); let (_, snd) = apply(&mut cg, s, 1);
    assert_eq!(cg.me.id, 1); assert_eq!(cg.remote, Some(0)); assert!((cg.me.pos - moved).length() < 0.01); assert_eq!(snd, vec![Snd::Coin]); assert_eq!(cg.w.p.id, 1);
    c.bye(); let t1 = std::time::Instant::now(); let mut left = false; while t1.elapsed().as_secs() < 5 && !left { for e in h.poll() { if let HostEv::Left = e { left = true } } std::thread::sleep(std::time::Duration::from_millis(10)) } assert!(left);
}
#[test] fn two_players_fishing_same_river_no_crash() {
    let mut g = two(at(11, 12, 0, 0)); let bob = vec2(20.5, 10.5);
    g.shadows = vec![Shadow { pos: bob, def: 10, vel: Vec2::ZERO }, Shadow { pos: bob + vec2(1., 0.), def: 12, vel: Vec2::ZERO }];
    g.me.fish = Some(Fishing { bob: bob + vec2(1., 0.), fish: Some(1), t: 0., nib: 0, need: 3, bite: false, dip: 0. }); // P1 hooked shadow 1
    g.activate(1); g.me.fish = Some(Fishing { bob, fish: Some(0), t: 0., nib: 3, need: 3, bite: true, dip: 0. }); g.w.p.inv.clear(); g.reel_pub(); assert!(g.w.p.inv.iter().any(|s| s.def == 10)); // P2 catches shadow 0
    g.activate(0); assert_eq!(g.me.fish.as_ref().unwrap().fish, Some(0)); assert_eq!(g.shadows[0].def, 12); // P1's index followed its fish
    for _ in 0..200 { g.tick(1. / 60., at(11, 12, 0, 0)); g.update_player(&Inp::none(), 1. / 60.); } // simulating must not panic
    g.me.fish = Some(Fishing { bob, fish: Some(7), t: 0., nib: 0, need: 3, bite: false, dip: 0. }); g.update_player(&Inp::none(), 1. / 60.); assert!(g.me.fish.is_none()); // stale index: let go, no panic
}
