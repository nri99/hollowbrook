//! Tests: clock / daily refresh (carried over)
use super::*;

// ---- clock / daily refresh (carried over) ----
#[test]
fn day_identity() {
    assert_eq!(at(10, 0, 0, 0).weekday(), chrono::Weekday::Tue);
    assert_eq!(day_id(at(10, 23, 59, 0)), day_id(at(11, 0, 1, 0)));
    assert_ne!(day_id(at(11, 5, 59, 0)), day_id(at(11, 6, 0, 0)))
}
#[test]
fn midnight_is_not_reset() {
    let mut g = g0(at(10, 23, 59, 50));
    let f = talk_daily(&mut g);
    assert!(!g.catch_up(at(11, 0, 0, 10)));
    assert!(g.w.p.npcs[0].talked);
    assert_eq!(talk_daily(&mut g), f);
    assert!(g.log.is_empty())
}
#[test]
fn six_am_refresh_once_and_stock_vs_hours() {
    let mut g = g0(at(11, 5, 59, 50));
    talk_daily(&mut g);
    g.w.stock[0].sold = true;
    assert!(!g.catch_up(at(11, 5, 59, 59)));
    assert!(g.w.stock[0].sold);
    assert!(g.catch_up(at(11, 6, 0, 10)));
    assert_eq!(g.log.len(), 1);
    assert!(!g.w.p.npcs[0].talked);
    assert!(g.w.stock.iter().all(|s| !s.sold));
    assert!(!shop_open(6) && shop_open(8));
    for i in 0..200 {
        assert!(!g.catch_up(at(11, 6, 1, i % 60)))
    }
    assert_eq!(g.log.len(), 1);
}
#[test]
fn reload_same_day_keeps_daily_state() {
    let mut g = g0(at(11, 6, 30, 0));
    talk_daily(&mut g);
    g.w.stock[2].sold = true;
    let mut g2 = reload(&g, "same");
    assert!(!g2.catch_up(at(11, 7, 0, 0)));
    assert!(g2.w.p.npcs[0].talked && g2.w.stock[2].sold);
    assert_eq!(g2.w, g.persist())
}
#[test]
fn closed_three_days_regrows_once() {
    let mut g = g0(at(11, 5, 0, 0));
    g.w.trees[0].fruit = false;
    g.w.trees[0].shaken = g.w.day;
    g.w.p.inv = vec![Slot { size_mm: 0, def: 20, qty: 3, uid: 0 }];
    g.w.p.money = 4321;
    let mut g2 = reload(&g, "three");
    assert!(g2.catch_up(at(14, 5, 0, 0)));
    assert_eq!(g2.log.len(), 1);
    assert!(g2.log[0].contains("(3d)"));
    assert!(g2.w.trees[0].fruit);
    assert_eq!((g2.w.p.money, g2.w.p.inv.clone()), (4321, vec![Slot { size_mm: 0, def: 20, qty: 3, uid: 0 }]));
    assert!(!g2.catch_up(at(14, 5, 30, 0)));
}
#[test]
fn fruit_not_regrown_early() {
    let mut g = g0(at(11, 7, 0, 0));
    g.w.trees[0].fruit = false;
    g.w.trees[0].shaken = g.w.day;
    assert!(g.catch_up(at(13, 7, 0, 0)));
    assert!(!g.w.trees[0].fruit)
}
#[test]
fn clock_rollback() {
    let mut g = g0(at(11, 7, 0, 0));
    let d = g.w.day;
    assert!(!g.catch_up(at(9, 7, 0, 0)));
    assert_eq!(g.w.day, d)
}

#[test]
fn daily_geode_rock() {
    let mut g = g0(at(11, 10, 0, 0));
    assert!(g.catch_up(at(12, 7, 0, 0))); // geodes appear with each morning's refresh
    let rock = g.w.geode_rock.expect("a geode rock each day");
    g.w.p.equip = Some(62);
    g.me.face = vec2(0., -1.);
    g.me.pos = tc(rock) + vec2(0., 0.9);
    let money = g.w.p.money;
    for _ in 0..(GEODE_MAX + 2) {
        g.use_tool();
    }
    let crystals: u16 = g.w.p.inv.iter().filter(|s| s.def == 177).map(|s| s.qty).sum();
    assert_eq!((crystals, g.w.geode_hits, g.w.p.money), (GEODE_MAX as u16, GEODE_MAX, money), "crystals, not coins");
    let g2 = reload(&g, "geode");
    assert_eq!(g2.w.geode_hits, GEODE_MAX);
    assert!(g2.check().is_empty());
    let mut g3 = g2;
    assert!(g3.catch_up(at(13, 7, 0, 0)));
    assert_eq!(g3.w.geode_hits, 0, "a fresh geode tomorrow");
}
