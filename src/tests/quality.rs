//! Tests: cross-cutting guards (world invariants under random input, UI text budgets, item id sync) and home stories.
use super::*;

#[test]
fn basement_and_upstairs_stories() {
    let mut g = two(at(11, 12, 0, 0));
    g.w.p.home = 2;
    g.w.p.money = 0;
    g.w.p.room.push(Furn { uid: 91, def: 50, x: 1, z: 1, rot: 0, on: false });
    g.w.p.room.push(Furn { uid: 92, def: 50, x: 4, z: 2, rot: 0, on: false });
    g.expand_home().unwrap();
    assert_eq!((g.w.p.home, g.w.p.debt, g.rdims(HOME)), (3, EXPAND_COST[2], (6, 5)));
    assert!(g.w.p.storage.iter().any(|s| s.uid == 91) && g.w.p.room.iter().any(|f| f.uid == 92)); // only the piece on the stairwell moved
    g.w.p.debt = 0;
    g.expand_home().unwrap();
    assert_eq!(g.w.p.home, 4);
    assert!(g.expand_home().is_err());
    g.swap(Scene::Room(HOME));
    g.me.scene = Scene::Room(HOME);
    assert_eq!(g.home_stairs(HOME).len(), 2);
    assert!(g.room_block(HOME, 0, 0, 0));
    g.take_stairs(1);
    assert_eq!(g.story_in(HOME), 1);
    assert!(g.w.p.room.is_empty());
    assert_eq!(g.home_stairs(HOME), vec![(STAIR_DOWN, 0)]);
    let f = Furn { uid: 93, def: 50, x: 3, z: 2, rot: 0, on: false };
    assert!(g.place_ok(&f).is_ok());
    g.w.p.room.push(f);
    assert!(g.place_ok(&Furn { uid: 94, def: 50, x: 1, z: 1, rot: 0, on: false }).is_err()); // stairs stay clear
    g.activate(1);
    assert!(g.room_furn(HOME).iter().any(|f| f.uid == 92) && !g.room_furn(HOME).iter().any(|f| f.uid == 93));
    g.activate(0); // a visitor sees the main floor
    let g2 = reload(&g, "stories");
    assert_eq!(g2.w.p.story, 1);
    assert!(g2.w.p.room.iter().any(|f| f.uid == 93));
    g.take_stairs(0);
    g.take_stairs(2);
    assert!(g.w.p.room.is_empty());
    g.take_stairs(0);
    assert!(g.w.p.room.iter().any(|f| f.uid == 92));
    g.take_stairs(1);
    g.swap(Scene::Town);
    assert_eq!(g.w.p.story, 0);
    assert!(g.w.p.room.iter().any(|f| f.uid == 92)); // leaving always returns to the main floor
    let (s, _) = Game::home_score(&g.w.p);
    g.take_stairs(1);
    assert_eq!(Game::home_score(&g.w.p).0, s); // score counts every story
}

/// UI regression guard: text tables must keep fitting their panels, including a split-screen half (640x720).
#[test]
fn ui_text_budgets() {
    for l in HELP_KEYS.iter().chain(HELP_PAD.iter()) {
        assert!(l.len() <= 60, "controls line too long for a split-screen journal: {l}")
    }
    assert!(
        168 + 26 + 21 * HELP_KEYS.len() + 34 + 21 * (HELP_PAD.len() - 1) <= 630,
        "controls page overflows the journal panel"
    ); // last baseline inside the panel
    assert!(70 + 34 + 20 * HELP_KEYS.len() + 34 + 20 * HELP_PAD.len() + 36 <= 720 - 56, "help screen lists overflow");
    for d in DEFS.iter() {
        assert!(d.name.len() <= 24, "item name too long for menus: {}", d.name)
    }
}

/// Stress test: two players mash random keys for thousands of frames while game time races through several days.
/// After every frame the world invariants (`Game::check`) must hold, nothing may panic, and a save/reload must keep
/// both players' possessions. Deterministic: a fixed LCG seed picks the keys.
#[test]
fn random_input_stress_keeps_invariants() {
    use KeyCode::*;
    const KEYS: [KeyCode; 18] =
        [W, A, S, D, Up, Down, Left, Right, E, Enter, Escape, Space, Tab, I, J, F, G, LeftShift];
    let mut seed: u64 = 0x5eed_1234;
    let mut rnd = move |n: usize| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % n
    };
    let mut g = two(at(11, 8, 0, 0));
    g.w.p.money = 99_999; // let random presses actually buy things
    let mut now = at(11, 8, 0, 0);
    let mut held: [Vec<KeyCode>; 2] = [vec![], vec![]];
    g.activate(1);
    g.swap(Scene::Room(g.home())); // start player 2 indoors so both scenes are exercised
    g.me.scene = Scene::Room(g.home());
    for frame in 0..40000 {
        now += chrono::Duration::seconds(7); // ~3.2 game days in total
        g.tick(0.05, now);
        for id in 0..2 {
            if frame % 6 == 0 {
                held[id] = (0..rnd(3)).map(|_| KEYS[rnd(4)]).collect();
            } // keep walking in one direction for a while
            let pressed: Vec<KeyCode> = if rnd(3) == 0 { vec![KEYS[rnd(KEYS.len())]] } else { vec![] };
            let down: Vec<KeyCode> = held[id].iter().chain(pressed.iter()).copied().collect();
            g.activate(id);
            g.update_player(&Inp::keys(&down, &pressed), 0.05);
            let bad = g.check();
            assert!(bad.is_empty(), "frame {frame}, player {id}: {bad:?}");
            if g.me.pose == Pose::Stand && g.me.trans.is_none() {
                assert!(
                    g.free(g.ppos()),
                    "frame {frame}, player {id} stuck inside something at {:?} in {:?}",
                    g.ppos(),
                    g.me.scene
                );
            }
        }
    }
    g.activate(0);
    let g2 = reload(&g, "stress");
    assert_eq!((&g2.w.p.inv, &g2.w.p.storage, g2.w.p.money), (&g.w.p.inv, &g.w.p.storage, g.w.p.money));
    assert!(g2.check().is_empty());
}

/// Named ids in `data::id` must keep pointing at the same items (saves store ids).
#[test]
fn named_item_ids_match_defs() {
    use crate::data::id;
    for (i, n) in [(id::CABBAGE_WHITE, "Cabbage White"), (id::CARP, "Carp"), (id::TROUT, "Trout")] {
        assert_eq!(def(i).name, n);
    }
    let src = include_str!("../data.rs");
    let m = &src[src.find("pub mod id {").unwrap()..];
    for line in m.lines().filter(|l| l.contains("pub const")) {
        let num: u16 = line.split('=').nth(1).unwrap().split(';').next().unwrap().trim().parse().unwrap();
        let name = line.split("// ").nth(1).unwrap().trim();
        assert_eq!(def(num).name, name, "id constant out of sync: {line}");
    }
}

/// Stress test for the newer worlds: P1 starts in Dustreach with gear, armour and coins, P2 on Glacia; both mash
/// random keys (incl. flight, nav computer, gear and standoffs) for thousands of frames across a day change.
/// Every frame: Game::check() is clean, nobody stands inside a wall, flight and bounty numbers stay finite.
#[test]
fn random_input_stress_new_worlds() {
    use KeyCode::*;
    const KEYS: [KeyCode; 20] =
        [W, A, S, D, W, W, E, E, Enter, Escape, Space, Tab, N, LeftShift, I, J, Up, Down, Left, Right];
    let mut seed: u64 = 0xd057_2ea5;
    let mut rnd = move |n: usize| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % n
    };
    let mut g = two(at(11, 8, 0, 0));
    let mut now = at(11, 8, 0, 0);
    g.w.p.money = 50_000;
    g.w.p.armor = 0b1111;
    g.w.p.tools.extend([172, 173, 174]);
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
    g.activate(1);
    g.swap(Scene::Planet(1));
    g.me.scene = Scene::Planet(1);
    let mut held: [Vec<KeyCode>; 2] = [vec![], vec![]];
    let mut seen = std::collections::BTreeSet::new();
    for frame in 0..30000 {
        now += chrono::Duration::seconds(4); // ~1.4 game days
        g.tick(0.05, now);
        for id in 0..2 {
            if frame % 8 == 0 {
                held[id] = (0..rnd(3)).map(|_| KEYS[rnd(4)]).collect();
            }
            let pressed: Vec<KeyCode> = if rnd(3) == 0 { vec![KEYS[rnd(KEYS.len())]] } else { vec![] };
            let down: Vec<KeyCode> = held[id].iter().chain(pressed.iter()).copied().collect();
            g.activate(id);
            if frame % 1500 == 0 && g.me.scene == Scene::Frontier && g.me.flight.is_none() && g.w.p.bounty.is_none() {
                // hand out a bounty and stand near the target, so hunts, standoffs and gear get exercised too
                let k = rnd(3) as u8;
                if g.take_bounty(k).is_ok() {
                    let p = g.w.p.bounty.as_ref().unwrap().pos[0];
                    let near = vec2(p[0], p[1] + 2.);
                    if g.free(near) {
                        g.me.ipos = near;
                    }
                    g.w.p.equip = Some(172 + rnd(3) as u16);
                }
            }
            g.update_player(&Inp::keys(&down, &pressed), 0.05);
            let bad = g.check();
            assert!(bad.is_empty(), "frame {frame}, player {id}: {bad:?}");
            if g.me.flight.is_none() && g.me.trans.is_none() && g.me.pose == Pose::Stand {
                assert!(
                    g.free(g.ppos()),
                    "frame {frame}, player {id} stuck in a wall at {:?} in {:?}",
                    g.ppos(),
                    g.me.scene
                );
            }
            seen.insert(format!("{:?}", g.me.scene).chars().take(9).collect::<String>());
            if g.me.flight.is_some_and(|f| f.jump > 0.) {
                seen.insert("jump".into());
            }
            if g.me.flight.is_some() {
                seen.insert("flying".into());
            }
            if g.w.p.bounty.is_some() {
                seen.insert("bounty".into());
            }
            if g.me.standoff > 0. {
                seen.insert("standoff".into());
            }
            if let Some(f) = g.me.flight {
                assert!(f.pos.is_finite() && f.vel.is_finite() && f.jump >= 0., "frame {frame}: bad flight {f:?}");
            }
            assert!(g.me.standoff >= 0. && g.me.dazed >= 0.);
            if let Some(b) = g.w.p.bounty.as_ref() {
                assert!(b.pos.iter().all(|p| p[0].is_finite() && p[1].is_finite()), "frame {frame}: bad bounty {b:?}");
            }
        }
    }
    eprintln!("stress worlds visited: {seen:?}");
    g.activate(0);
    let g2 = reload(&g, "stress_worlds");
    assert_eq!((&g2.w.p.inv, g2.w.p.money), (&g.w.p.inv, g.w.p.money));
    assert!(g2.check().is_empty());
}
