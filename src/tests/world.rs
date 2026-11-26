//! Tests: world / collision / targeting
use super::*;

// ---- world / collision / targeting ----
#[test]
fn door_beats_nearby_resident() {
    let mut g = g0(at(11, 12, 0, 0));
    g.me.pos = door_stand(HOME);
    g.me.face = vec2(0., -1.);
    for n in &mut g.npcs {
        n.inside = true
    }
    g.npcs[0].inside = false;
    g.npcs[0].pos = door_stand(HOME) + vec2(0.7, 0.1);
    assert_eq!(g.target(), Some(Tg::Door(HOME)));
    g.me.face = vec2(1., 0.);
    assert_eq!(g.target(), Some(Tg::Npc(0)));
    g.me.pos = vec2(13.5, 11.5);
    g.me.face = vec2(0., 1.);
    assert_ne!(g.target(), Some(Tg::Door(HOME))); // no interacting through the house
}
#[test]
fn water_bridge_buildings_collision() {
    let g = g0(at(11, 12, 0, 0));
    assert!(!g.walk_tile(24, 5) && !g.walk_tile(25, 20));
    assert!(g.walk_tile(24, 9) && g.walk_tile(25, 17));
    assert!(!g.walk_tile(13, 13));
    assert!(g.bfs((20, 9), (30, 9), true).is_some());
    for b in 0..BLDS.len() {
        let t = door_tile(b);
        assert!(g.walk_tile(t.0, t.1), "door {b} reachable");
        assert!(g.bfs(door_tile(HOME), t, true).is_some())
    }
}
#[test]
fn enter_exit_room() {
    let mut g = g0(at(11, 12, 0, 0));
    g.me.pos = door_stand(HOME);
    g.me.face = vec2(0., -1.);
    for n in &mut g.npcs {
        n.inside = true
    }
    g.interact();
    for _ in 0..40 {
        g.update(&Inp::none(), 0.016, at(11, 12, 0, 1));
    }
    assert_eq!(g.me.scene, Scene::Room(HOME));
    for _ in 0..120 {
        g.update(&Inp::keys(&[KeyCode::S], &[]), 0.016, at(11, 12, 0, 2));
        if g.me.scene == Scene::Town {
            break;
        }
    }
    for _ in 0..40 {
        g.update(&Inp::none(), 0.016, at(11, 12, 0, 3));
    }
    assert_eq!(g.me.scene, Scene::Town);
    assert_eq!(g.me.pos, door_stand(HOME));
}
#[test]
fn residents_avoid_water_and_buildings() {
    let mut g = g0(at(11, 10, 30, 0));
    for _ in 0..3000 {
        g.update(&Inp::none(), 0.033, at(11, 10, 30, 0));
        for n in g.npcs.iter().filter(|n| !n.inside) {
            let t = tile_of(n.pos);
            assert!(g.walk_tile(t.0, t.1), "resident on {:?}", t)
        }
    }
    assert!(g.npcs.iter().any(|n| !n.inside));
}
#[test]
fn legacy_v2_migrates() {
    let p = std::env::temp_dir().join("hb3_legacy.txt");
    let map = town();
    let mut s = format!(
        "v2\nOld\n739900\n777\n2\n22\n{}\n{}\n1,1,12,20\n{}\n100 100\n",
        ".".repeat(64 * 48),
        "0".repeat(64 * 48),
        "1".repeat(20)
    );
    for f in [3, 12, 0, 20] {
        s += &format!("{f} 999 1 100 100\n")
    }
    s += "room 1 4 4\n1:0:0:0:0,2:1:2:1:1\n";
    std::fs::write(&p, s).unwrap();
    let w = save::migrate(p.to_str().unwrap(), &map).unwrap();
    assert_eq!((w.p.name.as_str(), w.p.money, w.p.home, w.native, w.day), ("Old", 777, 2, 22, 739900));
    assert_eq!(
        w.p.inv,
        vec![
            Slot { size_mm: 0, def: 1, qty: 2, uid: 0 },
            Slot { size_mm: 0, def: 12, qty: 1, uid: 0 },
            Slot { size_mm: 0, def: 20, qty: 1, uid: 0 }
        ]
    );
    assert_eq!(w.p.npcs[1].friend, 12);
    assert_eq!(
        w.p.room,
        vec![
            Furn { uid: 1, def: 55, x: 0, z: 0, rot: 0, on: false },
            Furn { uid: 2, def: 51, x: 2, z: 1, rot: 1, on: false }
        ]
    );
    assert_eq!(w.uid, 3);
    let g = Game::from_world(w, map);
    assert_eq!(g.w.stock.len(), 5);
}
