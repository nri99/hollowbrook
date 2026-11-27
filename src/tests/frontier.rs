//! Tests: Dustreach frontier outpost and bounties.
use super::*;

fn to_frontier(g: &mut Game) {
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
}

#[test]
fn bounty_offers_take_detain_turn_in_once() {
    let mut g = g0(at(11, 10, 0, 0));
    let town_pos = g.me.pos;
    to_frontier(&mut g);
    assert_eq!(g.me.ipos, vec2(F_SPAWN.0, F_SPAWN.1));
    for &((x, z), _) in &F_SPOTS {
        assert!(f_walk(&g.fmap, x, z), "hiding spot {x},{z} on open ground");
    }
    assert!(!f_walk(&g.fmap, F_OASIS.0 as i32, F_OASIS.1 as i32), "the oasis is water");
    assert!(f_walk(&g.fmap, F_ARCH.0 + 1, F_ARCH.1), "you can walk under the arch");
    let offers = g.bounty_offers();
    assert_eq!(offers.len(), 3);
    assert_eq!(
        offers.iter().map(|o| o.name).collect::<Vec<_>>(),
        g.bounty_offers().iter().map(|o| o.name).collect::<Vec<_>>()
    ); // stable all day
    g.take_bounty(0).unwrap();
    assert!(g.take_bounty(1).is_err()); // one at a time
    assert!(g.turn_in().is_err()); // not caught yet
    let p = g.w.p.bounty.as_ref().unwrap().pos[0];
    assert!(f_walk(&g.fmap, p[0] as i32, p[1] as i32), "target stands on open ground");
    g.me.ipos = vec2(p[0], p[1] + 0.9);
    g.me.face = vec2(0., -1.);
    assert_eq!(g.target(), Some(Tg::Fugitive(0)));
    let money = g.w.p.money;
    g.interact();
    assert!(g.w.p.bounty.as_ref().unwrap().caught);
    assert_eq!(g.turn_in(), Err("Lock them in your ship's holding pod first."));
    g.deliver().unwrap();
    assert_eq!(g.turn_in(), Ok(BOUNTY_REWARD[0]));
    assert_eq!(g.w.p.marks, MARKS[0]);
    assert_eq!((g.w.p.money, g.w.p.bounties_done), (money + BOUNTY_REWARD[0], 1));
    assert!(g.turn_in().is_err()); // paid once
    assert_eq!(g.take_bounty(0), Err("You already brought that one in today."));
    g.swap(Scene::Town);
    g.me.scene = Scene::Town;
    assert_eq!(g.me.pos, town_pos); // the gate returns you where you left town
}

#[test]
fn lookalikes_runner_and_rollover() {
    let mut g = g0(at(11, 10, 0, 0));
    to_frontier(&mut g);
    g.take_bounty(2).unwrap();
    let b = g.w.p.bounty.clone().unwrap();
    assert_eq!(b.pos.len(), 3);
    g.detain(1);
    assert!(!g.w.p.bounty.as_ref().unwrap().caught); // wrong person
    g.drop_bounty();
    g.take_bounty(1).unwrap(); // runner
    let p0 = g.w.p.bounty.as_ref().unwrap().pos[0];
    g.me.ipos = vec2(p0[0], p0[1]) + vec2(1.5, 0.);
    for _ in 0..20 {
        g.frontier_tick(0.05);
    }
    let p1 = g.w.p.bounty.as_ref().unwrap().pos[0];
    assert!(vec2(p1[0], p1[1]).distance(g.me.ipos) > vec2(p0[0], p0[1]).distance(g.me.ipos), "runner flees");
    for _ in 0..200 {
        g.frontier_tick(0.05);
    }
    assert!(g.w.p.bounty.as_ref().unwrap().run <= 0., "runner tires out");
    assert!(g.catch_up(at(12, 7, 0, 0)));
    assert!(g.w.p.bounty.is_none(), "an uncaught bounty goes cold overnight");
}

#[test]
fn bounties_are_personal_and_saved() {
    let mut g = two(at(11, 10, 0, 0));
    to_frontier(&mut g);
    g.take_bounty(0).unwrap();
    g.detain(0);
    g.activate(1);
    assert!(g.w.p.bounty.is_none());
    assert!(g.turn_in().is_err());
    g.activate(0);
    let g2 = reload(&g, "frontier");
    assert!(g2.w.p.bounty.as_ref().is_some_and(|b| b.caught));
    assert!(g2.check().is_empty());
    assert_eq!(rank_of(0), "Drifter");
    assert_eq!(rank_of(8), "Hunter");
}

#[test]
fn gear_buy_stun_and_holstered_in_town() {
    let mut g = g0(at(11, 10, 0, 0));
    g.w.p.money = 6000;
    assert!(g.buy_gear(0).is_ok());
    assert_eq!((g.w.p.money, g.w.p.equip), (4500, Some(crate::data::id::STUN_PISTOL)));
    assert_eq!(g.buy_gear(0), Err("You already carry one."));
    g.use_tool(); // in Hollowbrook: holstered, nothing fired
    assert!(g.me.bolts.is_empty());
    to_frontier(&mut g);
    g.take_bounty(1).unwrap(); // runner
    let p = g.w.p.bounty.as_ref().unwrap().pos[0];
    g.me.ipos = vec2(p[0], p[1] + 4.);
    g.me.face = vec2(0., -1.);
    g.w.p.bounty.as_mut().unwrap().pos[0] = [g.me.ipos.x, g.me.ipos.y - 4.];
    g.use_tool();
    assert_eq!(g.me.bolts.len(), 1);
    for _ in 0..20 {
        g.frontier_tick(0.02);
    }
    let b = g.w.p.bounty.as_ref().unwrap();
    assert!(b.stun > 0. && b.run <= 0., "bolt stuns the runner: {b:?}");
    let before = b.pos[0];
    g.me.ipos = vec2(before[0], before[1] + 1.5); // walk right up: stunned, so no fleeing
    for _ in 0..20 {
        g.frontier_tick(0.02);
    }
    assert_eq!(g.w.p.bounty.as_ref().unwrap().pos[0], before);
    // the staff stuns at arm's length
    g.buy_gear(2).unwrap();
    g.w.p.bounty.as_mut().unwrap().stun = 0.;
    g.me.swing = 0.;
    g.me.ipos = vec2(before[0], before[1] + 1.0);
    g.use_tool();
    assert!(g.w.p.bounty.as_ref().unwrap().stun > 2.);
    assert!(g.check().is_empty());
}

#[test]
fn woods_path_door_leads_to_dustreach_and_back() {
    let mut g = g0(at(11, 10, 0, 0));
    for &(x, z) in &WOODS_PATH {
        assert!(g.walk_tile(x, z), "path tile {x},{z} walkable");
    }
    assert!(!g.walk_tile(WOODS_DOOR.0, WOODS_DOOR.1));
    g.me.pos = vec2(WOODS_DOOR.0 as f32 + 1.5, WOODS_DOOR.1 as f32 + 0.5);
    g.me.face = vec2(-1., 0.);
    assert_eq!(g.target(), Some(Tg::WoodsDoor));
    let at_door = g.me.pos;
    g.swap(Scene::Frontier);
    g.me.scene = Scene::Frontier;
    g.swap(Scene::Town);
    g.me.scene = Scene::Town;
    assert_eq!(g.me.pos, at_door); // the gate brings you back to the woods door
}

#[test]
fn ship_pod_forge_companion_tracker_and_ambush() {
    let mut g = g0(at(11, 10, 0, 0));
    to_frontier(&mut g);
    // the ship's ramp is reachable and leads to the holding pod
    assert!(f_walk(&g.fmap, F_RAMP.0 as i32, F_RAMP.1 as i32), "ramp spot is open ground");
    assert!(f_walk(&g.fmap, F_PAL_SPOT.0 as i32, F_PAL_SPOT.1 as i32), "companion spot is open ground");
    // a runner ambushes once when you get close: you are dazed, they bolt
    g.take_bounty(1).unwrap();
    let p = g.w.p.bounty.as_ref().unwrap().pos[0];
    g.me.ipos = vec2(p[0], p[1]) + vec2(1.0, 0.);
    g.frontier_tick(0.02);
    let b = g.w.p.bounty.as_ref().unwrap();
    assert!(b.ambushed && b.run > 0. && g.me.dazed > 1.);
    assert!(g.snd.contains(&Snd::Blip), "tracker beeps while hunting");
    // detained targets follow you to the ship and only then pay out, with guild marks
    g.w.p.bounty.as_mut().unwrap().run = 0.;
    g.w.p.bounty.as_mut().unwrap().stun = 3.;
    g.detain(0);
    g.me.ipos = vec2(F_RAMP.0, F_RAMP.1);
    for _ in 0..400 {
        g.frontier_tick(0.02);
    }
    let q = g.w.p.bounty.as_ref().unwrap().pos[0];
    assert!(vec2(q[0], q[1]).distance(g.me.ipos) < 1.4, "prisoner walked to you");
    g.me.face = vec2(0., -1.);
    assert_eq!(g.target(), Some(Tg::Ship));
    g.interact();
    assert!(g.w.p.bounty.as_ref().unwrap().delivered);
    g.turn_in().unwrap();
    assert_eq!(g.w.p.marks, MARKS[1]);
    // the forge: marks buy armour pieces once each; armour softens the next ambush
    assert_eq!(g.forge(0), Err("Not enough guild marks. Bring in more bounties."));
    g.w.p.marks = 20;
    g.forge(0).unwrap();
    g.forge(1).unwrap();
    assert_eq!(g.forge(0), Err("You already wear that piece."));
    assert_eq!((g.w.p.marks, g.armor_pieces()), (13, 2));
    // the companion: adopted once at the oasis, then trots after you
    g.me.ipos = vec2(F_PAL_SPOT.0, F_PAL_SPOT.1 + 1.);
    g.me.face = vec2(0., -1.);
    assert_eq!(g.target(), Some(Tg::Pal));
    g.interact();
    assert!(g.w.p.pal);
    g.me.ipos = vec2(20.5, 10.5);
    for _ in 0..200 {
        g.frontier_tick(0.02);
    }
    assert!(g.me.pal_pos.distance(g.me.ipos) < 1.5, "companion follows");
    let g2 = reload(&g, "hunt");
    assert!(g2.w.p.pal && g2.w.p.armor == 0b11 && g2.w.p.marks == 13);
    assert!(g2.check().is_empty());
}

#[test]
fn informant_bike_and_townsfolk() {
    let mut g = g0(at(11, 10, 0, 0));
    to_frontier(&mut g);
    for i in 0..LOCALS.len() {
        let (x, z) = LOCALS[i].0;
        assert!(f_walk(&g.fmap, x as i32, z as i32), "{} stands on open ground", LOCALS[i].1);
    }
    assert!(f_walk(&g.fmap, F_BIKE.0 as i32, F_BIKE.1 as i32), "bike spot is open ground");
    assert_eq!(bearing(vec2(5., 5.), vec2(5., 1.)), "north");
    assert_eq!(bearing(vec2(5., 5.), vec2(9., 9.)), "south-east");
    // the informant: one bearing per bounty, for coins
    g.take_bounty(0).unwrap();
    let money = g.w.p.money;
    g.talk_local(INFORMANT);
    assert!(g.w.p.bounty.as_ref().unwrap().hinted);
    assert_eq!(g.w.p.money, money - HINT_PRICE);
    g.talk_local(INFORMANT);
    assert_eq!(g.w.p.money, money - HINT_PRICE, "paid once");
    // the hover-bike: faster in Dustreach, parked when you leave
    let start = vec2(15.5, 12.5);
    g.me.ipos = start;
    g.step_move(vec2(1., 0.), false, 0.1);
    let walk = g.me.ipos.distance(start);
    g.me.ipos = start;
    g.toggle_bike();
    g.step_move(vec2(1., 0.), false, 0.1);
    assert!(g.me.ipos.distance(start) > walk * 1.5, "riding is faster");
    g.swap(Scene::Town);
    g.me.scene = Scene::Town;
    g.frontier_tick(0.02);
    assert!(!g.me.riding, "the bike stays in Dustreach");
}

#[test]
fn quick_draw_standoff() {
    let mut g = g0(at(11, 10, 0, 0));
    to_frontier(&mut g);
    g.take_bounty(2).unwrap();
    let p = g.w.p.bounty.as_ref().unwrap().pos[0];
    let start = vec2(p[0], p[1] + 2.5);
    g.me.ipos = start;
    g.frontier_tick(0.02);
    assert!(g.me.standoff > DRAW_WINDOW, "standoff begins near the most wanted");
    g.update_player(&Inp::keys(&[KeyCode::D], &[]), 0.1);
    assert_eq!(g.me.ipos, start, "you hold still during a standoff");
    g.standoff_press(); // too early
    assert!(g.w.p.bounty.as_ref().unwrap().run > 0. && g.me.standoff == 0.);
    // a fresh standoff, drawn in the window, stuns
    g.drop_bounty();
    g.take_bounty(2).unwrap();
    g.me.ipos = start;
    g.frontier_tick(0.02);
    while g.me.standoff > DRAW_WINDOW * 0.5 {
        g.frontier_tick(0.05);
    }
    g.standoff_press();
    let b = g.w.p.bounty.as_ref().unwrap();
    assert!(b.stun > 3. && b.run <= 0., "quick draw stuns: {b:?}");
    // leaving Dustreach clears it; missing the window lets them run
    g.drop_bounty();
    g.take_bounty(2).unwrap();
    g.me.ipos = start;
    g.frontier_tick(0.02);
    for _ in 0..200 {
        g.frontier_tick(0.05);
    }
    assert!(g.me.standoff == 0. && g.w.p.bounty.as_ref().unwrap().ambushed);
    // a standoff never lingers once you are far from the target
    g.drop_bounty();
    g.take_bounty(2).unwrap();
    g.me.ipos = start;
    g.frontier_tick(0.02);
    assert!(g.me.standoff > 0.);
    g.me.ipos = start + vec2(9., 0.);
    g.frontier_tick(0.02);
    assert_eq!(g.me.standoff, 0., "standoff cancelled out of range");
}

#[test]
fn fresh_bounties_after_all_three_are_brought_in() {
    let mut g = g0(at(11, 10, 0, 0));
    to_frontier(&mut g);
    let first: Vec<&str> = g.bounty_offers().iter().map(|o| o.name).collect();
    for round in 0..2 {
        for k in 0..3u8 {
            assert!(!g.bounty_done_today(k), "round {round}: offer {k} is open");
            g.take_bounty(k).unwrap();
            g.detain(0);
            g.deliver().unwrap();
            g.turn_in().unwrap();
        }
        assert_eq!(g.bounty_batch(g.w.day), round + 1);
    }
    let third: Vec<&str> = g.bounty_offers().iter().map(|o| o.name).collect();
    assert_ne!(first, third, "a new set of targets");
    assert!((0..3).all(|k| !g.bounty_done_today(k)), "all three of the new set are open");
    assert_eq!(g.w.p.bounties_done, 6);
    let g2 = reload(&g, "batches");
    assert_eq!(g2.bounty_batch(g2.w.day), 2);
}

/// Armour is worn on the body (bug: it used to be cubes at fixed offsets): in every pose, gait, tool swing, facing,
/// swim depth and ride height, every plate stays on the body it covers, and the helmet replaces any hat.
#[test]
fn armour_stays_on_the_body_in_every_pose() {
    use crate::render::{human_g, light, Gear, ARMOR_PLATE, R};
    let key = |c: Color| Vertex::new(0., 0., 0., 0., 0., c).color;
    let plate = [key(ARMOR_PLATE), key(Color::new(0.58 * 0.9, 0.62 * 0.9, 0.68 * 0.9, 1.))];
    let hat = key(Color::new(def(67).col.0, def(67).col.1, def(67).col.2, 1.));
    for pose in [0u8, 1, 2] {
        for (anim, spd) in [(0., 0.), (1.3, 2.5), (2.9, 6.)] {
            for swing in [0., 0.4] {
                for yaw in [0., 1.9] {
                    for y in [0., -0.55, 0.34] {
                        let draw = |armor: u8| {
                            let mut r = R::new(0., Vec2::ZERO, &light(12., false), (1e5, 2e5));
                            r.keep = true;
                            r.unlit = true;
                            let pos = vec3(3., y, -2.);
                            let gear = Gear { armor, ..Gear::default() };
                            human_g(&mut r, 0, (None, Some(67), 1), pos, yaw, anim, spd, pose, None, 1., swing, gear);
                            r.verts().to_vec()
                        };
                        let (body, worn) = (draw(0), draw(0b1111));
                        let case = format!("pose {pose} spd {spd} swing {swing} yaw {yaw} y {y}");
                        let pieces: Vec<Vec3> =
                            worn.iter().filter(|v| plate.contains(&v.color)).map(|v| v.position).collect();
                        assert!(pieces.len() > 300, "{case}: armour missing");
                        for p in pieces {
                            let d = body.iter().map(|v| v.position.distance(p)).fold(f32::MAX, f32::min);
                            assert!(d < 0.2, "{case}: armour plate at {p} is {d:.2} away from the body");
                        }
                        assert!(body.iter().any(|v| v.color == hat), "{case}: hat drawn without a helmet");
                        assert!(!worn.iter().any(|v| v.color == hat), "{case}: hat clashes with the helmet");
                    }
                }
            }
        }
    }
}

#[test]
fn other_player_shows_armour_bike_and_companion() {
    use crate::render::{light, others, ARMOR_PLATE, R};
    let mut g = two(at(11, 10, 0, 0));
    for id in 0..2 {
        g.activate(id);
        g.swap(Scene::Frontier);
        g.me.scene = Scene::Frontier;
    }
    g.activate(1);
    g.w.p.armor = 0b1111;
    g.w.p.pal = true;
    g.me.riding = true;
    g.me.ipos = vec2(15., 15.);
    g.me.pal_pos = vec2(16., 15.5);
    g.activate(0);
    let mut r = R::new(0., Vec2::ZERO, &light(12., false), (1e5, 2e5));
    r.keep = true;
    r.unlit = true;
    others(&mut r, &g, 0.);
    let key = |c: Color| Vertex::new(0., 0., 0., 0., 0., c).color;
    let plate = [key(ARMOR_PLATE), key(Color::new(0.58 * 0.9, 0.62 * 0.9, 0.68 * 0.9, 1.))];
    let bike_paint = key(Color::new(0.62, 0.22, 0.18, 1.));
    let pal_fur = key(Color::new(0.92, 0.66, 0.4, 1.));
    let armour_verts: Vec<Vec3> = r.verts().iter().filter(|v| plate.contains(&v.color)).map(|v| v.position).collect();
    assert!(!armour_verts.is_empty(), "other player's armour missing");
    let min_y = armour_verts.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    assert!((min_y - 0.34).abs() < 0.01, "other player's armour should sit at bike ride height, min_y={min_y}");
    assert!(
        r.verts()
            .iter()
            .any(|v| v.color == bike_paint && vec2(v.position.x, v.position.z).distance(vec2(15., 15.)) < 1.2),
        "other player's hover-bike missing"
    );
    assert!(
        r.verts()
            .iter()
            .any(|v| v.color == pal_fur && vec2(v.position.x, v.position.z).distance(vec2(16., 15.5)) < 0.5),
        "other player's sand-hopper companion missing"
    );
}
