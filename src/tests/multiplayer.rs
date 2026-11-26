//! Tests: local multiplayer (MP-01/02)
use super::*;

// ---- local multiplayer (MP-01/02) ----
#[test]
fn v3_json_upgrades_to_profiles() {
    let g = g0(at(11, 12, 0, 0));
    let mut v = serde_json::to_value(g.persist()).unwrap();
    let p = v["p"].clone();
    let o = v.as_object_mut().unwrap();
    o.remove("p");
    o.remove("others");
    o.insert("version".into(), 3.into());
    for (k, x) in p.as_object().unwrap() {
        if k != "id" {
            o.insert(k.clone(), x.clone());
        }
    }
    let w: World = serde_json::from_value(save::upgrade(v)).unwrap();
    assert_eq!(w.version, SAVE_VERSION);
    {
        let a = &w.p;
        let b = g.persist().p;
        assert_eq!(
            (&a.name, a.money, &a.inv, &a.room, &a.npcs, &a.tools),
            (&b.name, b.money, &b.inv, &b.room, &b.npcs, &b.tools)
        );
    }
    assert!(w.others.is_empty());
}
#[test]
fn two_profiles_independent_and_persist() {
    let mut g = two(at(11, 12, 0, 0));
    g.activate(1);
    g.w.p.money = 5;
    g.add(20);
    g.activate(0);
    assert_eq!(g.w.p.money, 1000);
    assert!(g.w.p.inv.is_empty());
    let mut g2 = reload(&g, "mp");
    assert_eq!(g2.w.p.id, 0);
    assert_eq!(g2.w.others.len(), 1);
    g2.join("x");
    g2.activate(1);
    assert_eq!(g2.w.p.money, 5);
    assert_eq!(g2.w.p.inv[0].def, 20);
    assert_eq!(g2.w.p.name, "P2");
}
#[test]
fn last_stock_and_pickup_only_once() {
    let mut g = two(at(11, 12, 0, 0));
    for id in [0, 1] {
        g.activate(id);
        g.w.p.money = 9000
    }
    g.activate(0);
    assert!(g.buy(0).is_ok());
    g.activate(1);
    assert_eq!(g.buy(0), Err("Sold out!"));
    assert_eq!(g.w.p.money, 9000);
    let it = g.w.ground[0];
    for id in [0, 1] {
        g.activate(id);
        g.me.scene = Scene::Town;
        g.me.pos = vec2(it.x, it.z + 0.6);
        g.me.face = vec2(0., -1.);
        if let Some(Tg::Item(_)) = g.target() {
            g.interact()
        }
    }
    let total: u16 = [0, 1]
        .iter()
        .map(|&id| {
            g.activate(id);
            g.w.p.inv.iter().filter(|s| s.def == it.def).map(|s| s.qty).sum::<u16>()
        })
        .sum();
    assert_eq!(total, 1);
}
#[test]
fn different_scenes_and_independent_ui() {
    let mut g = two(at(11, 12, 0, 0));
    g.activate(0);
    g.swap(Scene::Room(HOME));
    g.me.ui = Ui::Inv(0);
    g.activate(1);
    assert_eq!(g.me.scene, Scene::Town);
    assert_eq!(g.me.ui, Ui::Play);
    g.step_move(vec2(1., 0.), false, 0.1);
    let p1 = g.me.pos;
    g.activate(0);
    assert_eq!(g.me.scene, Scene::Room(HOME));
    assert_eq!(g.me.ui, Ui::Inv(0));
    g.activate(1);
    assert_eq!(g.me.pos, p1);
}
#[test]
fn rollover_with_players_in_different_scenes() {
    let mut g = two(at(11, 5, 59, 0));
    for id in [0, 1] {
        g.activate(id);
        g.w.p.npcs[0].talked = true
    }
    g.activate(0);
    g.swap(Scene::Room(HOME));
    assert!(g.catch_up(at(11, 6, 0, 1)));
    assert_eq!(g.log.len(), 1);
    for id in [0, 1] {
        g.activate(id);
        assert!(!g.w.p.npcs[0].talked)
    }
    assert!(!g.catch_up(at(11, 6, 0, 5)));
    g.activate(0);
    assert_eq!(g.me.scene, Scene::Room(HOME));
}
#[test]
fn leave_keeps_possessions() {
    let mut g = two(at(11, 12, 0, 0));
    g.activate(1);
    g.add(40);
    g.w.p.money = 77;
    g.activate(0);
    g.leave(1);
    assert_eq!(g.ids(), vec![0]);
    let w = g.persist();
    assert_eq!(w.others[0].money, 77);
    assert_eq!(w.others[0].inv[0].def, 40);
    g.join("again");
    g.activate(1);
    assert_eq!(g.w.p.money, 77);
}
#[test]
fn separate_homes_owner_only_furniture() {
    let mut g = two(at(11, 12, 0, 0));
    g.activate(1);
    assert_eq!(g.home(), 7);
    g.swap(Scene::Room(7));
    g.add_furn(50);
    let uid = g.w.p.inv[0].uid;
    g.start_place(uid);
    {
        let p = g.me.placing.as_mut().unwrap();
        p.f.x = 0;
        p.f.z = 0;
    }
    g.place_confirm().unwrap();
    assert_eq!(g.room_furn(7).len(), 1);
    assert!(g.room_furn(HOME).is_empty());
    g.activate(0);
    g.swap(Scene::Room(7));
    assert_eq!(g.room_furn(7).len(), 1);
    g.me.ipos = vec2(0.5, 1.5);
    g.me.face = vec2(0., -1.);
    assert!(matches!(g.target(), Some(Tg::Furn(u, _)) if u == uid));
    g.update(&Inp::keys(&[KeyCode::G], &[KeyCode::G]), 0.016, at(11, 12, 0, 1));
    assert_eq!(g.room_furn(7).len(), 1);
    assert!(g.w.p.inv.is_empty()); // visitor cannot take it
    let w = g.persist();
    assert_eq!(w.others[0].room.len(), 1);
    assert!(w.p.room.is_empty());
}
