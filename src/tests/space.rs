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

#[test]
fn each_player_has_their_own_ship_and_sees_the_other() {
    let mut g = two(at(11, 10, 0, 0));
    for id in 0..2 {
        g.activate(id);
        g.swap(Scene::Frontier);
        g.me.scene = Scene::Frontier;
    }
    // separate pads, each player can only board their own ship
    for &(x, z, w, d) in &F_PADS {
        assert!(!f_walk(&g.fmap, x + w / 2, z + d / 2), "pad is solid");
    }
    for i in 0..2 {
        assert!(f_walk(&g.fmap, ramp_of(0, i).x as i32, ramp_of(0, i).y as i32), "Dustreach ramp {i} reachable");
        for k in [1u8, 2] {
            assert!(p_walk(k, ramp_of(k, i).x as i32, ramp_of(k, i).y as i32), "ramp {i} reachable on planet {k}");
            assert!(
                p_walk(k, ramp_of(k, i).x as i32, (ramp_of(k, i).y + 1.1) as i32),
                "step-out spot {i} on planet {k}"
            );
        }
    }
    g.activate(0);
    assert_eq!(g.my_ramp(), Some(ramp_of(0, 0)));
    g.me.ipos = ramp_of(0, 1); // standing at P2's ramp
    g.me.face = vec2(0., -1.);
    assert_ne!(g.target(), Some(Tg::Ship), "can't board someone else's ship");
    g.activate(1);
    assert_eq!(g.my_ramp(), Some(ramp_of(0, 1)));
    // both in Dustreach: they see each other
    g.activate(0);
    assert!(sees(&g.me, &g.others[0]));
    // P1 boards and takes off: P1's ship leaves the pad, P2's stays; P2 can't see P1 inside or in flight
    g.swap(Scene::Cockpit(0));
    g.me.scene = Scene::Cockpit(0);
    assert!(!sees(&g.others[0], &g.me), "nobody sees you inside your own ship");
    g.take_off();
    assert!(!ship_parked(&g.me, 0) && ship_parked(&g.others[0], 0));
    assert!(!sees(&g.others[0], &g.me));
    // P2 takes off too: they see each other's ships in the same system, not in different ones
    g.activate(1);
    g.swap(Scene::Cockpit(0));
    g.me.scene = Scene::Cockpit(0);
    g.take_off();
    assert!(sees_in_space(&g.me, &g.others[0]));
    g.plot_jump(2).unwrap();
    assert!(!sees_in_space(&g.me, &g.others[0]), "not while in the light-speed tunnel");
    for _ in 0..80 {
        g.update_player(&Inp::none(), 0.05);
    }
    assert!(!sees_in_space(&g.me, &g.others[0]), "different systems");
    // P2 lands on Verdance and steps out at their own pad; P1 follows and sees P2 there
    g.me.flight.as_mut().unwrap().pos = vec2(0., PLANET_R + 2.);
    g.land().unwrap();
    assert_eq!(g.me.ship_at, 2);
    g.swap(Scene::Planet(2));
    g.me.scene = Scene::Planet(2);
    assert!(g.free(g.me.ipos) && g.me.ipos.distance(ramp_of(2, 1)) < 1.6, "stepped out by pad 2");
    g.activate(0);
    g.plot_jump(2).unwrap();
    for _ in 0..80 {
        g.update_player(&Inp::none(), 0.05);
    }
    g.me.flight.as_mut().unwrap().pos = vec2(0., PLANET_R + 2.);
    g.land().unwrap();
    g.swap(Scene::Planet(2));
    g.me.scene = Scene::Planet(2);
    assert!(sees(&g.me, &g.others[0]), "both exploring Verdance");
    assert!(ship_parked(&g.me, 2) && ship_parked(&g.others[0], 2), "both ships on their own pads");
    assert_eq!(g.my_ramp(), Some(ramp_of(2, 0)));
    // coming back through the woods door sends your ship home
    g.swap(Scene::Town);
    g.me.scene = Scene::Town;
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
    assert_eq!(g.me.ship_at, 0);
    assert!(g.check().is_empty());
}

#[test]
fn ship_ramp_aligns_with_ramp_of_on_every_pad_and_planet() {
    use crate::render::{draw_pads, light, PAD_DECK_C, R, SHIP_RAMP_C};
    let mut g = two(at(11, 10, 0, 0));
    let ramp_col = Vertex::new(0., 0., 0., 0., 0., SHIP_RAMP_C).color;
    let deck_col = Vertex::new(0., 0., 0., 0., 0., PAD_DECK_C).color;
    for planet in [0u8, 1, 2] {
        let sc = if planet == 0 { Scene::Frontier } else { Scene::Planet(planet) };
        for id in 0..2 {
            g.activate(id);
            g.swap(sc);
            g.me.scene = sc;
            g.me.flight = None;
            g.me.ship_at = planet;
            g.me.ipos = vec2(10., 18.);
        }
        g.activate(0);
        let mut r = R::new(0., Vec2::ZERO, &light(12., false), (1e5, 2e5));
        r.keep = true;
        r.unlit = true;
        let mut labels = vec![];
        draw_pads(&mut r, &g, planet, 0., 0., &mut labels, false);
        for i in 0..2 {
            let (px, pz, pw, pd) = if planet == 0 { F_PADS[i] } else { P_PADS[i] };
            assert!(
                r.verts().iter().any(|v| {
                    v.color == deck_col
                        && v.position.x >= px as f32
                        && v.position.x <= (px + pw) as f32
                        && v.position.z >= pz as f32
                        && v.position.z <= (pz + pd) as f32
                }),
                "planet {planet} pad {i}: landing pad deck missing under parked ship"
            );
            let r_pos = ramp_of(planet, i);
            let verts: Vec<Vec3> = r
                .verts()
                .iter()
                .filter(|v| v.color == ramp_col && v.position.x >= px as f32 && v.position.x <= (px + pw) as f32)
                .map(|v| v.position)
                .collect();
            assert!(!verts.is_empty(), "planet {planet} pad {i}: missing ramp vertices");
            let max_z = verts.iter().map(|p| p.z).fold(f32::MIN, f32::max);
            let foot: Vec<Vec3> = verts.iter().copied().filter(|p| (p.z - max_z).abs() < 0.05).collect();
            let min_x = foot.iter().map(|p| p.x).fold(f32::MAX, f32::min);
            let max_x = foot.iter().map(|p| p.x).fold(f32::MIN, f32::max);
            let mid_x = (min_x + max_x) / 2.;
            assert!((mid_x - r_pos.x).abs() < 0.05, "planet {planet} pad {i}: ramp x {mid_x} != {}", r_pos.x);
            assert!((max_z - r_pos.y).abs() < 0.15, "planet {planet} pad {i}: ramp z {max_z} != {}", r_pos.y);
            assert!(max_z > (pz + pd) as f32, "planet {planet} pad {i}: ramp foot {max_z} inside solid pad");
        }
        for id in 0..2 {
            g.activate(id);
            let r_pos = ramp_of(planet, pad_of(id));
            assert!(g.free(r_pos), "planet {planet} player {id}: ramp_of not walkable");
            g.me.ipos = r_pos;
            g.me.face = vec2(0., -1.);
            assert_eq!(g.target(), Some(Tg::Ship), "planet {planet} player {id}: can't target ship at ramp_of");
            g.swap(Scene::Cockpit(planet));
            g.me.scene = Scene::Cockpit(planet);
            g.swap(sc);
            g.me.scene = sc;
            assert!(g.free(g.me.ipos), "planet {planet} player {id}: stepped out onto blocked tile");
            assert!(
                g.me.ipos.distance(r_pos) < 1.0,
                "planet {planet} player {id}: step-out {:?} too far from ramp {:?}",
                g.me.ipos,
                r_pos
            );
        }
    }
}

#[test]
fn ship_upgrades_buy_persist_and_improve_flight_and_gathering() {
    let mut g = two(at(11, 10, 0, 0));
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
    // talking to Dena opens the shipyard menu
    g.talk_local(MECHANIC);
    assert_eq!(g.me.ui, Ui::ShipYard(0));
    g.me.ui = Ui::Play;
    // coin check, buying 1..=3, max cap, save/reload, and 2P independence
    g.w.p.money = 1000;
    assert_eq!(g.upgrade_ship(0), Err("Not enough coins."));
    g.w.p.money = 50_000;
    for lv in 1..=SHIP_UP_MAX {
        assert_eq!(g.upgrade_ship(0), Ok(lv));
    }
    assert_eq!(g.upgrade_ship(0), Err("Already at maximum level."));
    assert_eq!(g.w.p.ship_up[0], SHIP_UP_MAX);
    assert_eq!(ship_tier(g.w.p.ship_up), SHIP_UP_MAX);
    g.activate(1);
    assert_eq!(g.w.p.ship_up, [0; 4], "ship upgrades are per player");
    g.activate(0);
    let g2 = reload(&g, "ship_upgrades");
    assert_eq!(g2.w.p.ship_up, [SHIP_UP_MAX, 0, 0, 0]);
    assert!(g2.check().is_empty());

    // Thrusters lv 3 moves further and turns faster than lv 0
    g.activate(0);
    g.swap(Scene::Cockpit(0));
    g.me.scene = Scene::Cockpit(0);
    g.take_off();
    fly(&mut g, &[KeyCode::W, KeyCode::D], &[], 20);
    let f0 = g.me.flight.unwrap();
    g.activate(1);
    g.swap(Scene::Cockpit(0));
    g.me.scene = Scene::Cockpit(0);
    g.take_off();
    fly(&mut g, &[KeyCode::W, KeyCode::D], &[], 20);
    let f1 = g.me.flight.unwrap();
    let start = vec2(0., PLANET_R + 3.);
    assert!(f0.pos.distance(start) > f1.pos.distance(start), "upgraded thrusters fly faster");
    assert!(
        (f0.yaw - std::f32::consts::PI).abs() > (f1.yaw - std::f32::consts::PI).abs(),
        "upgraded thrusters turn faster"
    );

    // Hyperdrive lv 3 gives shorter jump and wider landing range
    g.activate(0);
    for _ in 0..SHIP_UP_MAX {
        g.upgrade_ship(1).unwrap();
    }
    g.plot_jump(1).unwrap();
    let j0 = g.me.flight.unwrap().jump;
    g.activate(1);
    g.plot_jump(1).unwrap();
    let j1 = g.me.flight.unwrap().jump;
    assert!(j0 < j1, "upgraded hyperdrive jumps faster ({j0} < {j1})");
    g.activate(0);
    fly(&mut g, &[], &[], (j0 / 0.05) as usize + 2);
    g.me.flight.as_mut().unwrap().pos = vec2(0., PLANET_R + 8.0);
    assert!(g.land().is_ok(), "hyperdrive lv 3 lands from orbit further out");
    g.activate(1);
    fly(&mut g, &[], &[], (j1 / 0.05) as usize + 2);
    g.me.flight.as_mut().unwrap().pos = vec2(0., PLANET_R + 8.0);
    assert!(g.land().is_err(), "stock ship cannot land from that far out");

    // Hull & Wings lv 3 gives +50% bounty reward
    g.activate(0);
    for _ in 0..SHIP_UP_MAX {
        g.upgrade_ship(2).unwrap();
    }
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
    g.take_bounty(0).unwrap();
    let b = g.w.p.bounty.as_mut().unwrap();
    b.caught = true;
    b.delivered = true;
    let pay = g.turn_in().unwrap();
    assert_eq!(pay, BOUNTY_REWARD[0] * 3 / 2);

    // Scanner lv 3 gathers 3 items per planet spot vs 1 at lv 0
    for _ in 0..SHIP_UP_MAX {
        g.upgrade_ship(3).unwrap();
    }
    g.take_sample(1, 0);
    let p0_count: u16 = g.w.p.inv.iter().filter(|s| s.def == SAMPLE_ITEM[0]).map(|s| s.qty).sum();
    assert_eq!(p0_count, 3);
    g.activate(1);
    g.take_sample(1, 0);
    let p1_count: u16 = g.w.p.inv.iter().filter(|s| s.def == SAMPLE_ITEM[0]).map(|s| s.qty).sum();
    assert_eq!(p1_count, 1);
    assert!(g.check().is_empty());
}

#[test]
fn upgraded_ship_visuals_change_on_pad_and_in_flight() {
    use crate::render::{draw_pads, flier, light, R};
    let mut g = g0(at(11, 10, 0, 0));
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
    g.me.ipos = vec2(10., 18.);

    let mut r0 = R::new(0., Vec2::ZERO, &light(12., false), (1e5, 2e5));
    r0.keep = true;
    let mut l0 = vec![];
    draw_pads(&mut r0, &g, 0, 0., 0., &mut l0, false);

    g.w.p.ship_up = [3, 3, 3, 3];
    let mut r1 = R::new(0., Vec2::ZERO, &light(12., false), (1e5, 2e5));
    r1.keep = true;
    let mut l1 = vec![];
    draw_pads(&mut r1, &g, 0, 0., 0., &mut l1, false);
    assert!(r1.verts().len() > r0.verts().len(), "upgraded ship on pad has extra geometry");

    let mut f0 = R::new(0., Vec2::ZERO, &light(12., false), (1e5, 2e5));
    f0.keep = true;
    flier(&mut f0, Vec3::ZERO, 0., 1., [0; 4], 0., (0, (None, None, 0)));

    let mut f1 = R::new(0., Vec2::ZERO, &light(12., false), (1e5, 2e5));
    f1.keep = true;
    flier(&mut f1, Vec3::ZERO, 0., 1., [3, 3, 3, 3], 0., (0, (None, None, 0)));
    assert!(f1.verts().len() > f0.verts().len(), "upgraded ship in flight has extra geometry");
}
