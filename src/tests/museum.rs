//! Tests: museum (P3)
use super::*;

// ---- museum (P3) ----
#[test]
fn museum_shared_donation_once_and_fossil_id() {
    let mut g = two(at(11, 12, 0, 0));
    g.activate(0);
    g.add(12);
    g.add(47);
    g.add(0);
    g.activate(1);
    g.add(12);
    g.activate(0);
    g.donate();
    assert!(g.w.donated.iter().any(|x| *x == (12, 0)) && g.w.donated.iter().any(|x| *x == (0, 0)));
    assert_eq!(g.w.donated.iter().filter(|x| FOSSILS.contains(&x.0)).count(), 1);
    assert!(!g.w.p.inv.iter().any(|s| s.def == 47));
    g.activate(1);
    g.donate();
    assert_eq!(g.w.donated.iter().filter(|x| x.0 == 12).count(), 1);
    assert!(g.w.p.inv.iter().any(|s| s.def == 12)); // already donated: P2 keeps the carp
    let g2 = reload(&g, "mus");
    assert_eq!(g2.w.donated, g.w.donated);
}
#[test]
fn museum_reachable_and_blocks() {
    let g = g0(at(11, 12, 0, 0));
    let m = BLDS.iter().position(|b| b.kind == BK::Museum).unwrap();
    let t = door_tile(m);
    assert!(g.walk_tile(t.0, t.1));
    assert!(g.bfs(door_tile(HOME), t, true).is_some());
    assert!(g.room_block(m, 0, 2, 0) && !g.room_block(m, 3, 3, 0))
}
#[test]
fn clothing_buy_wear_swap_persist() {
    let mut g = two(at(11, 12, 0, 0));
    g.activate(1);
    g.w.p.money = 2000;
    assert!(g.buy_cloth(63).is_ok());
    assert!(g.buy_cloth(65).is_ok());
    assert_eq!(g.w.p.money, 2000 - 400 - 650);
    let (a, b) = (g.w.p.inv[0].uid, g.w.p.inv[1].uid);
    g.wear(a);
    assert_eq!(g.w.p.shirt, Some(63));
    assert_eq!(g.w.p.inv.len(), 1);
    g.wear(b);
    assert_eq!(g.w.p.shirt, Some(65));
    assert_eq!(g.w.p.inv.len(), 1);
    assert_eq!(g.w.p.inv[0].def, 63); // old shirt back in pockets
    g.w.p.money = 10;
    assert_eq!(g.buy_cloth(64), Err("Not enough coins."));
    g.activate(0);
    assert_eq!(g.w.p.shirt, None);
    let mut g2 = reload(&g, "cloth");
    g2.join("x");
    g2.activate(1);
    assert_eq!(g2.w.p.shirt, Some(65));
}
#[test]
fn home_expansion_balance() {
    let mut g = two(at(11, 12, 0, 0));
    g.w.p.room.push(Furn { uid: 51, def: 50, x: 3, z: 4, rot: 0, on: false });
    g.me.ui = Ui::Sell(0);
    g.update_player(&Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]), 0.016);
    assert_eq!(g.me.ui, Ui::ConfirmExpand);
    g.update_player(&Inp::keys(&[KeyCode::Escape], &[KeyCode::Escape]), 0.016);
    assert_eq!((g.w.p.home, g.w.p.debt), (0, 0)); // cancel: no change
    g.expand_home().unwrap();
    assert_eq!((g.rdims(HOME), g.w.p.debt, g.w.p.money), ((5, 5), EXPAND_COST[0], 1000));
    assert!(g.expand_home().is_err()); // must clear balance first
    g.pay_debt();
    assert_eq!((g.w.p.money, g.w.p.debt), (0, EXPAND_COST[0] - 1000));
    g.w.p.money = 99999;
    g.pay_debt();
    assert_eq!(g.w.p.debt, 0);
    g.expand_home().unwrap();
    assert!(g.w.p.storage.iter().any(|s| s.uid == 51));
    g.activate(1);
    assert_eq!((g.w.p.debt, g.rdims(7)), (0, (4, 4)));
    g.activate(0);
    let g2 = reload(&g, "debt");
    assert_eq!(g2.w.p.debt, EXPAND_COST[1]);
}
#[test]
fn mail_delivered_per_profile_claim_once() {
    let mut g = two(at(11, 12, 0, 0));
    g.activate(1);
    for n in &mut g.w.p.npcs {
        n.friend = 20
    }
    let m0 = g.w.p.money;
    g.activate(0);
    for d in 12..40 {
        g.catch_up(at(1 + d % 28, 7, 0, 0).checked_add_months(chrono::Months::new((d / 28) as u32)).unwrap());
        g.activate(1);
        if !g.w.p.mail.is_empty() {
            break;
        }
        g.activate(0)
    }
    g.activate(1);
    assert!(!g.w.p.mail.is_empty(), "P2 got no letter");
    assert_eq!(g.w.p.money, m0); // coins wait in the letter
    let c = g.w.p.mail.last().unwrap().coins;
    g.open_letter(0);
    assert_eq!(g.w.p.money, m0 + c);
    g.open_letter(0);
    assert_eq!(g.w.p.money, m0 + c);
    assert!(g.w.p.mail.last().unwrap().claimed);
    g.activate(0);
    assert!(!g.w.p.mail.iter().any(|l| l.coins > 0));
    let mut g2 = reload(&g, "mail");
    g2.join("x");
    g2.activate(1);
    assert!(g2.w.p.mail.last().unwrap().claimed);
    assert!(!g.walk_tile(mailbox_tile(0).0, mailbox_tile(0).1));
}
#[test]
fn weather_rule_snow_in_winter() {
    let wet = (739900..740400).find(|&d| rainy(d)).unwrap();
    let dry = (739900..740400).find(|&d| !rainy(d)).unwrap();
    assert_eq!(weather(wet, 1), Wx::Snow);
    assert_eq!(weather(wet, 12), Wx::Snow);
    assert_eq!(weather(wet, 6), Wx::Rain);
    assert_eq!(weather(dry, 1), Wx::Clear);
}
#[test]
fn catalog_reorder_full_price() {
    let mut g = g0(at(11, 12, 0, 0));
    g.w.p.money = 5000;
    let d = g.w.stock[0].def;
    g.buy(0).unwrap();
    assert!(g.w.p.catalog.contains(&d));
    let m = g.w.p.money;
    g.order(d).unwrap();
    assert_eq!(g.w.p.money, m - def(d).buy);
    assert_eq!(g.w.p.inv.iter().filter(|s| s.def == d).count(), 2);
    assert_ne!(g.w.p.inv[0].uid, g.w.p.inv[1].uid);
    assert_eq!(g.order(57).err(), if d == 57 { None } else { Some("That isn't in your catalog.") }.filter(|_| d != 57));
    g.w.p.money = 0;
    assert_eq!(g.order(d), Err("Not enough coins."));
    let g2 = reload(&g, "cat");
    assert!(g2.w.p.catalog.contains(&d));
}
#[test]
fn gifts_once_per_day_with_preferences() {
    let mut g = g0(at(11, 12, 0, 0));
    g.add(20);
    g.add(20);
    g.add(40);
    let f0 = g.w.p.npcs[0].friend;
    g.give_gift(0, 0);
    assert_eq!(g.w.p.npcs[0].friend, f0 + 2);
    assert_eq!(g.w.p.inv[0].qty, 1); // Juniper likes fruit
    g.give_gift(0, 0);
    assert_eq!(g.w.p.npcs[0].friend, f0 + 2);
    assert_eq!(g.w.p.inv[0].qty, 1); // once per day
    let f1 = g.w.p.npcs[1].friend;
    let shell = g.w.p.inv.iter().position(|s| s.def == 40).unwrap();
    g.give_gift(1, shell);
    assert_eq!(g.w.p.npcs[1].friend, f1 + 1);
    assert!(g.catch_up(at(12, 6, 0, 1)));
    assert!(!g.w.p.npcs[0].gifted);
    g.me.ui = Ui::Talk { npc: Some(2), who: "x".into(), lines: vec!["a".into()], i: 0 };
    g.update(&Inp::keys(&[KeyCode::G], &[KeyCode::G]), 0.016, at(12, 7, 0, 0));
    assert_eq!(g.me.ui, Ui::Gift(2, 0));
}
#[test]
fn harvest_fair_window_and_prize_once() {
    let mut g = g0(at(14, 12, 0, 0));
    g.update(&Inp::none(), 0.016, at(14, 12, 0, 0));
    assert!(g.fair());
    assert_eq!(tile_at(&g.map, STALL.0, STALL.1), GRASS);
    assert!(!g.walk_tile(STALL.0, STALL.1));
    g.fair_prize();
    assert_eq!(g.w.p.inv.len(), 1);
    assert_eq!(g.w.p.fair_year, 2026);
    g.fair_prize();
    assert_eq!(g.w.p.inv.len(), 1);
    assert!(!fair_active(11, 17) && !fair_active(10, 14) && fair_active(11, 13));
    let mut h = g0(at(20, 12, 0, 0));
    h.update(&Inp::none(), 0.016, at(20, 12, 0, 0));
    assert!(!h.fair());
    h.fair_prize();
    assert!(h.w.p.inv.is_empty());
    assert!(h.walk_tile(STALL.0, STALL.1));
}
#[test]
fn swimming_and_diving() {
    let mut g = g0(at(11, 12, 0, 0));
    g.me.pos = vec2(13.5, 25.5);
    g.me.face = vec2(0., 1.);
    assert!(!g.swimming());
    for _ in 0..40 {
        g.step_move(vec2(0., 1.), false, 0.05)
    }
    assert!(g.swimming(), "pos {:?}", g.me.pos);
    assert!(!g.walk_tile(13, 27)); // residents never path into the sea
    g.bubbles = vec![(g.me.pos + vec2(0.4, 0.), 73)];
    g.use_tool();
    assert!(g.w.p.inv.iter().any(|s| s.def == 73));
    assert!(g.bubbles.is_empty());
    g.use_tool();
    assert_eq!(g.w.p.inv.len(), 1);
    g.donate();
    assert!(g.w.donated.iter().any(|x| x.0 == 73));
    for _ in 0..60 {
        g.step_move(vec2(0., -1.), false, 0.05)
    }
    assert!(!g.swimming());
}
#[test]
fn resident_visits_player_home() {
    let mut g = g0(at(14, 10, 30, 0));
    for n in &mut g.w.p.npcs {
        n.friend = 5
    }
    g.swap(Scene::Room(HOME));
    let mut tries = 0;
    while g.visit.is_none() && tries < 20000 {
        g.tick(1. / 60., at(14, 10, 30, 0));
        tries += 1
    }
    let (i, b, _) = g.visit.expect("no visit");
    assert_eq!(b, HOME);
    assert!(g.w.p.npcs[i].visited);
    g.me.ipos = g.visit_spot(HOME) + vec2(0.8, 0.);
    g.me.face = vec2(-1., 0.);
    assert_eq!(g.target(), Some(Tg::Npc(i)));
    g.interact();
    if let Ui::Talk { lines, .. } = &g.me.ui {
        assert_eq!(
            lines[0],
            VISIT[g.res(i).pers]
                .replace("{p}", "T")
                .replace("{fs}", &format!("{}s", def(g.w.native).name.to_lowercase()))
                .replace("{f}", "")
        )
    }
    g.me.ui = Ui::Play;
    g.swap(Scene::Town);
    g.tick(0.016, at(14, 10, 30, 1));
    assert!(g.visit.is_none());
    assert!(!g.npcs[i].inside);
}
#[test]
fn roster_defaults_and_mapping() {
    let g = g0(at(11, 12, 0, 0));
    let mut v = serde_json::to_value(g.persist()).unwrap();
    v.as_object_mut().unwrap().remove("roster");
    let w: World = serde_json::from_value(v).unwrap();
    assert_eq!(w.roster, vec![0, 1, 2, 3]);
    let mut g2 = Game::from_world(w, town());
    assert_eq!(g2.res(0).name, "Juniper");
    g2.w.roster[0] = 4;
    assert_eq!(g2.res(0).name, "Clover");
    assert_eq!(HOUSES[0], 2);
}
#[test]
fn residents_move_out_and_in_with_stay_option() {
    let mut g = g0(at(1, 12, 0, 0));
    let d0 = g.w.day;
    g.w.p.npcs[1].friend = 2;
    for k in [0, 2, 3] {
        g.w.p.npcs[k].friend = 9
    }
    let mut day = |g: &mut Game, n: i64| {
        g.w.day = d0 + n - 1;
        for p in std::iter::once(&mut g.w.p).chain(g.w.others.iter_mut()) {
            for s in &mut p.npcs {
                s.req = None
            }
        }
        g.w.day = d0 + n;
        g.refresh_moves();
    };
    for n in 1..14 {
        day(&mut g, n);
        assert_eq!(g.w.moving[1], 0)
    }
    day(&mut g, 14);
    assert_eq!(g.w.moving[1], d0 + 16);
    assert!(g.w.p.mail.iter().any(|l| l.from == "Bramble"));
    assert!(g.w.moving.iter().enumerate().all(|(k, &m)| k == 1 || m == 0)); // friendly residents never announce
    g.me.ui = Ui::Talk { npc: Some(1), who: "x".into(), lines: vec!["a".into()], i: 0 };
    g.update(&Inp::keys(&[KeyCode::X], &[KeyCode::X]), 0.016, at(1, 12, 0, 1));
    assert_eq!(g.w.moving[1], 0); // asked to stay
    g.w.p.npcs[1].friend = 0;
    for n in 15..29 {
        day(&mut g, n)
    }
    assert_ne!(g.w.moving[1], 0);
    let leave = g.w.moving[1];
    day(&mut g, leave - d0);
    assert_eq!(g.w.roster[1], 4);
    assert_eq!(g.res(1).name, "Clover");
    assert_eq!(g.w.p.npcs[1].friend, 0);
    assert_eq!(g.bname(3), "Clover's House");
    assert_eq!(g.w.moving[1], 0);
    let g2 = reload(&g, "move");
    assert_eq!(g2.w.roster, g.w.roster);
}
#[test]
fn plurals() {
    assert_eq!(plural("peach"), "peaches");
    assert_eq!(plural("cherry"), "cherries");
    assert_eq!(plural("apple"), "apples");
}
#[test]
fn customize_tint_follows_item_and_decor() {
    let mut g = g0(at(11, 12, 0, 0));
    g.w.p.money = 2000;
    g.add_furn(50);
    let uid = g.w.p.inv[0].uid;
    assert!(g.customize(0).is_ok());
    assert_eq!(Game::tint_of(&g.w.p, uid), 1);
    assert_eq!(g.w.p.money, 2000 - TINT_FEE);
    g.swap(Scene::Room(HOME));
    g.start_place(uid);
    {
        let p = g.me.placing.as_mut().unwrap();
        p.f.x = 0;
        p.f.z = 0;
    }
    g.place_confirm().unwrap();
    assert_eq!(Game::tint_of(&g.w.p, uid), 1);
    g.pick_furn(uid);
    assert_eq!(Game::tint_of(&g.w.p, uid), 1);
    assert!(g.customize(1).is_ok());
    assert_eq!(g.w.p.wall, 1);
    assert!(g.customize(2).is_ok());
    assert_eq!(g.w.p.floor, 1);
    g.w.p.money = 0;
    assert_eq!(g.customize(2), Err("Not enough coins."));
    assert_eq!(g.w.p.floor, 1);
    let g2 = reload(&g, "tint");
    assert_eq!(Game::tint_of(&g2.w.p, uid), 1);
    assert_eq!((g2.w.p.wall, g2.w.p.floor), (1, 1));
}
#[test]
fn hats_and_hair() {
    let mut g = g0(at(11, 12, 0, 0));
    g.w.p.money = 2000;
    g.buy_cloth(67).unwrap();
    g.buy_cloth(68).unwrap();
    g.buy_cloth(63).unwrap();
    let u: Vec<u32> = g.w.p.inv.iter().map(|s| s.uid).collect();
    g.wear(u[0]);
    g.wear(u[2]);
    assert_eq!((g.w.p.hat, g.w.p.shirt), (Some(67), Some(63)));
    assert_eq!(g.w.p.inv.len(), 1);
    g.wear(u[1]);
    assert_eq!(g.w.p.hat, Some(68));
    assert_eq!(g.w.p.inv[0].def, 67); // straw hat back in pockets
    g.me.ui = Ui::Rack(CLOTHES.len());
    let m = g.w.p.money;
    g.update(&Inp::keys(&[KeyCode::Enter], &[KeyCode::Enter]), 0.016, at(11, 12, 0, 1));
    assert_eq!(g.w.p.hair, 1);
    assert_eq!(g.w.p.money, m - HAIR_FEE);
    let g2 = reload(&g, "hat");
    assert_eq!((g2.w.p.hat, g2.w.p.hair), (Some(68), 1));
}
