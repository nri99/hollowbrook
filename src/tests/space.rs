//! Tests: spaceflight, light-speed jumps, landing on Glacia and Verdance, gathering.
use super::*;

fn fly(g: &mut Game, keys: &[KeyCode], pressed: &[KeyCode], frames: usize) {
    for _ in 0..frames {
        g.update_player(&Inp::keys(keys, pressed), 0.05);
    }
}

#[test]
fn take_off_jump_land_gather_and_return() {
    let mut g = g0(at(11, 10, 0, 0));
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
    // board at the ramp, walk to the pilot's seat, take off
    g.me.ipos = vec2(F_RAMP.0, F_RAMP.1);
    g.me.face = vec2(0., -1.);
    assert_eq!(g.target(), Some(Tg::Ship));
    g.interact();
    fly(&mut g, &[], &[], 12); // the door transition
    assert_eq!(g.me.scene, Scene::Cockpit(0), "inside the ship");
    g.take_off(); // not at the seat yet is fine via the function; the seat is the in-game way:
    g.me.flight = None;
    g.me.ipos = vec2(CK_STAND.0, CK_STAND.1);
    g.me.face = vec2(0., -1.);
    assert_eq!(g.target(), Some(Tg::Pilot));
    g.interact();
    let f = g.me.flight.expect("flying after take-off");
    assert_eq!(f.sys, 0);
    // thrust moves the ship; ground movement is off while flying
    let p0 = f.pos;
    fly(&mut g, &[KeyCode::W], &[], 10);
    assert!(g.me.flight.unwrap().pos.distance(p0) > 0.5, "thrust moves the ship");
    assert_eq!(g.me.ipos, vec2(CK_STAND.0, CK_STAND.1), "you're at the controls, not walking");
    // the nav computer: jump to Glacia
    fly(&mut g, &[], &[KeyCode::N], 1);
    assert_eq!(g.me.ui, Ui::Nav(0));
    fly(&mut g, &[], &[KeyCode::S], 1);
    fly(&mut g, &[], &[KeyCode::Enter], 1);
    assert!(g.me.flight.unwrap().jump > 0.);
    assert_eq!(g.land(), Err("Wait until you drop out of light speed."));
    fly(&mut g, &[], &[], (JUMP_TIME / 0.05) as usize + 2);
    let f = g.me.flight.unwrap();
    assert_eq!((f.sys, f.jump), (1, 0.));
    assert_eq!(g.land(), Err("Fly closer to the planet to land."));
    g.me.flight.as_mut().unwrap().pos = vec2(0., PLANET_R + 2.);
    fly(&mut g, &[], &[KeyCode::E], 1);
    assert_eq!(g.me.scene, Scene::Cockpit(1), "landed, still inside the ship");
    assert!(g.me.flight.is_none());
    g.me.ipos = vec2(CK_EXIT.0, CK_EXIT.1 - 0.4);
    g.me.face = vec2(0., 1.);
    assert_eq!(g.target(), Some(Tg::ShipExit));
    g.interact();
    fly(&mut g, &[], &[], 12);
    assert_eq!(g.me.scene, Scene::Planet(1), "walked down the ramp");
    assert!(g.free(g.me.ipos), "on open ground");
    // gather every resource once
    for (s, &(x, z)) in SAMPLE_SPOTS[0].iter().enumerate() {
        assert!(p_walk(1, x, z), "Glacia spot {s} walkable");
        g.take_sample(1, s);
    }
    assert_eq!(g.w.p.inv.iter().filter(|s| s.def == SAMPLE_ITEM[0]).map(|s| s.qty).sum::<u16>(), 6);
    g.take_sample(1, 0);
    assert_eq!(g.w.p.inv.iter().filter(|s| s.def == SAMPLE_ITEM[0]).map(|s| s.qty).sum::<u16>(), 6, "once a day");
    let g2 = reload(&g, "space");
    assert!(g2.sample_taken(1, 0) && g2.check().is_empty());
    // take off again, jump home, land at the Dustreach pad
    g.me.ipos = vec2(P_RAMP.0, P_RAMP.1);
    g.me.face = vec2(0., -1.);
    assert_eq!(g.target(), Some(Tg::Ship));
    g.interact();
    fly(&mut g, &[], &[], 12);
    assert_eq!(g.me.scene, Scene::Cockpit(1));
    g.take_off();
    assert_eq!(g.me.flight.unwrap().sys, 1);
    assert_eq!(g.plot_jump(1), Err("You're already in this system."));
    g.plot_jump(0).unwrap();
    fly(&mut g, &[], &[], (JUMP_TIME / 0.05) as usize + 2);
    g.me.flight.as_mut().unwrap().pos = vec2(0., PLANET_R + 2.);
    g.land().unwrap();
    assert_eq!(g.me.scene, Scene::Cockpit(0));
    g.disembark();
    fly(&mut g, &[], &[], 12);
    assert_eq!(g.me.scene, Scene::Frontier);
    assert!(g.free(g.me.ipos) && g.me.ipos.distance(vec2(F_RAMP.0, F_RAMP.1)) < 1.5, "stepped out at the ramp");
}

#[test]
fn landing_sites_are_sound() {
    for k in [1u8, 2] {
        assert!(p_walk(k, P_RAMP.0 as i32, P_RAMP.1 as i32), "ramp on open ground (planet {k})");
        assert!(p_walk(k, P_SPAWN.0 as i32, P_SPAWN.1 as i32));
        assert!(!p_walk(k, 15, 5), "the ship blocks its tiles");
        for &(x, z) in &SAMPLE_SPOTS[k as usize - 1] {
            assert!(p_walk(k, x, z), "spot {x},{z} walkable on planet {k}");
        }
        let (cx, cz, _, _) = P_POND[k as usize - 1];
        assert_eq!(ptile(k, cx as i32, cz as i32), SEA);
    }
    // the flight stays inside the system and out of the planet
    let mut g = g0(at(11, 10, 0, 0));
    g.swap(Scene::Cockpit(0));
    g.me.scene = Scene::Cockpit(0);
    g.take_off();
    g.me.flight.as_mut().unwrap().yaw = 0.; // nose towards the planet
    fly(&mut g, &[KeyCode::W, KeyCode::LeftShift], &[], 80);
    assert!(g.me.flight.unwrap().pos.length() >= PLANET_R + 1.1, "never inside the planet");
}

#[test]
fn debug_scene_key_ignored_while_flying() {
    std::env::set_var("HOLLOWBROOK_DEBUG", "1");
    let mut g = g0(at(11, 10, 0, 0));
    g.swap(Scene::Cockpit(0));
    g.me.scene = Scene::Cockpit(0);
    g.take_off();
    g.update_player(&Inp::keys(&[KeyCode::F9], &[KeyCode::F9]), 0.05);
    assert!(g.me.trans.is_none() && g.me.flight.is_some(), "F9 must not change scenes mid-flight");
}

#[test]
fn ship_interior_is_sound() {
    for z in 0..CK.1 {
        for x in 0..CK.0 {
            let _ = ck_block(x, z);
        }
    }
    assert!(!ck_block(CK_STAND.0 as i32, CK_STAND.1 as i32), "room to stand at the seat");
    assert!(!ck_block(CK_EXIT.0 as i32, CK_EXIT.1 as i32), "the ramp door is reachable");
    assert!(ck_block(CK_SEAT.0, CK_SEAT.1) && ck_block(0, 1) && ck_block(5, 1), "seat, cell and bunk are solid");
    let mut g = g0(at(11, 10, 0, 0));
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
    g.take_off();
    assert!(g.me.flight.is_none(), "you can only fly from the pilot's seat");
    // standing up / walking inside works and nobody gets stuck
    g.swap(Scene::Cockpit(0));
    g.me.scene = Scene::Cockpit(0);
    assert!(g.free(g.me.ipos));
    for _ in 0..30 {
        g.step_move(vec2(1., 0.), false, 0.05)
    }
    assert!(g.free(g.me.ipos));
}
