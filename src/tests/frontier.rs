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
    assert_eq!(g.turn_in(), Ok(BOUNTY_REWARD[0]));
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
