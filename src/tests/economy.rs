//! Tests: economy
use super::*;

// ---- economy ----
#[test]
fn shop_rules() {
    let mut g = g0(at(11, 12, 0, 0));
    g.w.p.money = 10000;
    let p = def(g.w.stock[0].def).buy;
    assert!(g.buy(0).is_ok());
    assert_eq!(g.w.p.money, 10000 - p);
    assert!(g.w.stock[0].sold);
    let uid = g.w.p.inv[0].uid;
    assert!(uid != 0 && g.w.p.inv[0].def == g.w.stock[0].def);
    let m = g.w.p.money;
    assert_eq!(g.buy(0), Err("Sold out!"));
    assert_eq!(g.w.p.money, m);
    assert_eq!(g.w.p.inv.len(), 1); // repeated input: no duplicate
    g.w.p.money = 0;
    assert_eq!(g.buy(1), Err("Not enough coins."));
    assert!(!g.w.stock[1].sold);
    assert_eq!(g.w.p.inv.len(), 1);
    g.w.p.money = 10000;
    while g.w.p.inv.len() < INV_CAP {
        g.w.p.inv.push(Slot { size_mm: 0, def: 40, qty: 1, uid: 0 })
    }
    assert_eq!(g.buy(2), Err("Your pockets are full."));
    assert_eq!(g.w.p.money, 10000);
    assert!(!g.w.stock[2].sold);
    g.me.ui = Ui::Confirm(3);
    let before = g.w.clone();
    g.update(&Inp::keys(&[KeyCode::Escape], &[KeyCode::Escape]), 0.016, at(11, 12, 0, 1));
    assert_eq!(g.me.ui, Ui::Play);
    assert_eq!(g.w.stock, before.stock);
    assert_eq!(g.w.p.money, before.p.money); // cancel
    assert!(g.buy(5).is_ok());
    assert!(g.w.p.tools.contains(&62));
    assert!(g.buy(5).is_err());
}
#[test]
fn sold_out_persists_until_six() {
    let mut g = g0(at(11, 12, 0, 0));
    g.w.p.money = 9999;
    g.buy(1).unwrap();
    let mut g2 = reload(&g, "sold");
    assert!(!g2.catch_up(at(12, 5, 59, 0)));
    assert!(g2.w.stock[1].sold);
    assert!(g2.catch_up(at(12, 6, 0, 1)));
    assert!(g2.w.stock.iter().all(|s| !s.sold))
}
#[test]
fn gather_fruit_and_sell() {
    let mut g = g0(at(11, 12, 0, 0));
    let ti = g.w.trees.iter().position(|t| g.walk_tile(t.x, t.z + 1)).unwrap();
    let t = g.w.trees[ti];
    g.me.pos = vec2(t.x as f32 + 0.5, t.z as f32 + 1.5);
    g.me.face = vec2(0., -1.);
    for n in &mut g.npcs {
        n.inside = true
    }
    assert_eq!(g.target(), Some(Tg::Tree(ti)));
    g.interact();
    assert!(!g.w.trees[ti].fruit);
    let drops: Vec<GItem> = g.w.ground.iter().filter(|i| i.def == g.w.native).copied().collect();
    assert!(!drops.is_empty());
    for d in &drops {
        g.me.pos = vec2(d.x, d.z + 0.6);
        if !g.free(g.me.pos) {
            g.me.pos = vec2(d.x + 0.6, d.z);
            g.me.face = vec2(-1., 0.)
        } else {
            g.me.face = vec2(0., -1.)
        }
        if let Some(Tg::Item(_)) = g.target() {
            g.interact()
        }
    }
    let got: u16 = g.w.p.inv.iter().filter(|s| s.def == g.w.native).map(|s| s.qty).sum();
    assert!(got >= 1);
    let m = g.w.p.money;
    g.sell_all();
    assert_eq!(g.w.p.money, m + 100 * got as u32);
    assert!(g.w.p.inv.is_empty());
}
#[test]
fn digging_holes_and_sites() {
    let mut g = g0(at(11, 12, 0, 0));
    g.w.p.tools.push(62);
    g.w.p.equip = Some(62);
    let site = g.w.digs[0];
    g.me.pos = tc((site.0, site.1 + 1));
    g.me.face = vec2(0., -1.);
    if !g.free(g.me.pos) {
        g.me.pos = tc((site.0 + 1, site.1));
        g.me.face = vec2(-1., 0.)
    }
    g.use_tool();
    assert!(g.w.holes.contains(&site));
    assert!(g.w.p.inv.iter().any(|s| matches!(s.def, 45 | 46 | 47)));
    assert!(!g.walk_tile(site.0, site.1));
    g.use_tool();
    assert!(!g.w.holes.contains(&site));
    assert!(g.walk_tile(site.0, site.1));
}
