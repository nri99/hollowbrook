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
