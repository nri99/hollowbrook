//! Tests: M3 activities: contests, invitations, hide-and-seek.
use super::*;

#[test]
fn m3_v4_migration_keeps_both_profiles_and_defaults_all_activity_fields() {
    let mut g = two(at(13, 12, 0, 0));
    g.w.p.inv.push(Slot { def: 12, qty: 9, uid: 0, size_mm: 0 });
    g.w.p.money = 4321;
    g.activate(1);
    g.add_furn(50);
    g.transfer(0, 0);
    g.w.p.debt = 700;
    g.activate(0);
    let expected = g.persist();
    let mut v = serde_json::to_value(&expected).unwrap();
    v["version"] = 4.into();
    fn old_profile(p: &mut serde_json::Value) {
        for key in ["contests", "invitation", "social_day", "missed_visit", "seek"] {
            p.as_object_mut().unwrap().remove(key);
        }
        for key in ["inv", "storage"] {
            for s in p[key].as_array_mut().unwrap() {
                s.as_object_mut().unwrap().remove("size_mm");
            }
        }
    }
    old_profile(&mut v["p"]);
    for p in v["others"].as_array_mut().unwrap() {
        old_profile(p);
    }
    let upgraded: World = serde_json::from_value(save::upgrade(v)).unwrap();
    assert_eq!(upgraded, expected);
}

fn seek_offer(g: &mut Game, slot: usize) {
    g.w.p.invitation =
        Some(SocialInvite { day: g.w.day, slot, character: g.w.roster[slot], kind: 1, hour: g.hr, state: 0 });
    g.accept_social().unwrap();
}
fn find_seeker(g: &mut Game, h: &HidingFriend) {
    g.me.scene = Scene::Town;
    g.me.ui = Ui::Play;
    for off in [vec2(0.7, 0.), vec2(-0.7, 0.), vec2(0., 0.7), vec2(0., -0.7)] {
        g.me.pos = tc(h.tile) + off;
        g.me.face = -off.normalize();
        if g.free(g.me.pos) && g.target() == Some(Tg::Npc(h.slot)) {
            g.update_player(&Inp::keys(&[], &[KeyCode::E]), 0.016);
            return;
        }
    }
    panic!("hidden friend must remain reachable for interaction: {h:?}");
}
#[test]
fn m3_seek_shared_walks_personal_progress_reload_and_once_only_rewards() {
    let now = at(13, 12, 0, 0);
    let mut g = two(now);
    g.hr = 12;
    g.w.p.npcs[0].friend = 5;
    g.npcs[0].inside = false;
    g.npcs[0].pos = g.me.pos + vec2(0.7, 0.);
    g.maybe_social_offer();
    assert_eq!(g.w.p.invitation.as_ref().unwrap().kind, 1);
    g.accept_social().unwrap();
    let spots = g.w.p.seek.as_ref().unwrap().spots.clone();
    assert_eq!(spots.len(), 3);
    for h in &spots {
        assert!(g.free(tc(h.tile)));
        assert!(g.bfs(tile_of(g.me.pos), h.tile, true).is_some());
        assert!(!g.mp_site_ok(h.tile));
    }
    for i in 0..500 {
        g.tick(0.2, now + chrono::Duration::milliseconds(i * 200));
    }
    for h in &spots {
        assert!(g.npcs[h.slot].pos.distance(tc(h.tile)) < 0.25, "resident walked to hiding spot");
    }
    g.activate(1);
    seek_offer(&mut g, 3);
    assert_eq!(g.w.p.seek.as_ref().unwrap().spots, spots);
    assert!(g.w.p.seek.as_ref().unwrap().found.is_empty());
    g.activate(0);
    find_seeker(&mut g, &spots[0]);
    assert_eq!(g.w.p.seek.as_ref().unwrap().found.len(), 1);
    let clock = g.clock_ms;
    let mut g = reload(&g, "m3_seek_partial");
    g.join("Partner");
    g.clock_ms = clock;
    g.hr = 12;
    assert_eq!(g.w.p.seek.as_ref().unwrap().found, vec![spots[0].slot]);
    for h in &spots {
        g.npcs[h.slot].inside = false;
        g.npcs[h.slot].pos = tc(h.tile);
    }
    let money = g.w.p.money;
    find_seeker(&mut g, &spots[1]);
    find_seeker(&mut g, &spots[2]);
    assert_eq!(g.w.p.money, money + 240);
    assert!(g.w.p.seek.as_ref().unwrap().done);
    assert!(g.seek_hide_position(spots[0].slot).is_some(), "partner's round still holds the same residents");
    let money = g.w.p.money;
    g.w.p.invitation.as_mut().unwrap().state = 0;
    assert!(g.accept_social().is_err());
    assert_eq!(g.w.p.money, money);
    g.activate(1);
    assert!(g.w.p.seek.as_ref().unwrap().found.is_empty());
    let money = g.w.p.money;
    for h in &spots {
        find_seeker(&mut g, h);
    }
    assert_eq!(g.w.p.money, money + 240);
    assert!(g.seek_hide_position(spots[0].slot).is_none());
    let clock = g.clock_ms;
    let mut g = reload(&g, "m3_seek_done");
    g.clock_ms = clock;
    assert!(g.w.p.seek.as_ref().unwrap().done);
}

#[test]
fn m3_seek_timeout_indoors_offline_rollover_and_blocked_start_are_safe() {
    let now = at(13, 12, 0, 0);
    let mut g = two(now);
    g.hr = 12;
    seek_offer(&mut g, 0);
    let deadline = g.w.p.seek.as_ref().unwrap().deadline_ms;
    let before = (g.w.p.money, g.w.p.inv.clone());
    g.swap(Scene::Room(HOME));
    let mut g = reload(&g, "m3_seek_offline");
    g.swap(Scene::Room(HOME));
    g.tick(0., now + chrono::Duration::seconds(241));
    g.update_player(&Inp::none(), 0.);
    let s = g.w.p.seek.as_ref().unwrap();
    assert_eq!(s.deadline_ms, deadline);
    assert!(s.done && s.failed);
    assert_eq!((g.w.p.money, g.w.p.inv.clone()), before);
    assert_eq!(g.me.scene, Scene::Room(HOME));
    g.tick(0., now);
    assert!(!g.seek_live(g.w.p.seek.as_ref().unwrap()), "clock rollback cannot revive finished round");
    g.w.p.seek.as_mut().unwrap().done = false;
    g.w.p.seek.as_mut().unwrap().deadline_ms += 86_400_000;
    g.join("Partner");
    g.activate(1);
    g.w.p.seek = Some(g.w.others.iter().find(|p| p.id == 0).unwrap().seek.clone().unwrap());
    g.activate(0);
    g.catch_up(at(14, 6, 0, 0));
    for p in std::iter::once(&g.w.p).chain(g.w.others.iter()) {
        assert!(p.seek.as_ref().unwrap().failed);
        assert!(p.seek.as_ref().unwrap().done);
    }
    let mut g = g0(now);
    g.hr = 12;
    g.map.fill(BLD);
    g.w.p.invitation = Some(SocialInvite { day: g.w.day, slot: 0, character: 0, kind: 1, hour: 12, state: 0 });
    let before = g.w.p.clone();
    assert!(g.accept_social().is_err());
    assert_eq!(g.w.p, before);
}

#[test]
fn m3_seek_protects_appointments_and_mails_full_pocket_reward() {
    let mut g = two(at(13, 12, 0, 0));
    g.hr = 12;
    g.activate(1);
    g.w.p.invitation = Some(SocialInvite { day: g.w.day, slot: 0, character: 0, kind: 0, hour: 13, state: 1 });
    g.activate(0);
    seek_offer(&mut g, 1);
    let spots = g.w.p.seek.as_ref().unwrap().spots.clone();
    assert!(!spots.iter().any(|h| h.slot == 0));
    for h in &spots {
        g.npcs[h.slot].inside = false;
        g.npcs[h.slot].pos = tc(h.tile);
    }
    for _ in 0..INV_CAP {
        g.add_furn(50);
    }
    for h in &spots {
        find_seeker(&mut g, h);
    }
    assert!(g.w.p.mail.iter().any(|l| l.from == "Trail Seek friends" && l.item == Some(80)));
    assert_eq!(g.w.others[0].invitation.as_ref().unwrap().state, 1);
}

#[test]
fn m3_home_invitation_initiated_visit_independence_and_reload() {
    let mut g = two(at(12, 12, 0, 0));
    g.hr = 12;
    g.w.p.npcs[0].friend = 5;
    g.npcs[0].inside = false;
    g.npcs[0].pos = g.me.pos + vec2(0.8, 0.);
    g.maybe_social_offer();
    let v = g.w.p.invitation.clone().expect("resident initiates");
    assert_eq!((v.kind, v.hour, v.state), (0, 13, 0));
    let mut g = reload(&g, "m3_offer");
    g.join("Partner");
    g.talk(0);
    assert_eq!(g.me.ui, Ui::Invitation);
    g.update_player(&Inp::keys(&[], &[KeyCode::Enter]), 0.016);
    assert_eq!(g.w.p.invitation.as_ref().unwrap().state, 1);
    g.hr = 13;
    assert_eq!(g.loc(0), Loc::Home);
    g.activate(1);
    let p2friend = g.w.p.npcs[0].friend;
    g.swap(Scene::Room(HOUSES[0]));
    assert_eq!(g.w.p.npcs[0].friend, p2friend);
    assert!(g.w.p.inv.is_empty());
    g.activate(0);
    let friend = g.w.p.npcs[0].friend;
    g.swap(Scene::Room(HOUSES[0]));
    assert_eq!(g.w.p.npcs[0].friend, friend + 4);
    assert!(g.w.p.inv.iter().any(|s| s.def == FLOWERS[0]));
    let inv = g.w.p.inv.clone();
    let mut g = reload(&g, "m3_visit_done");
    g.hr = 13;
    g.swap(Scene::Room(HOUSES[0]));
    assert_eq!(g.w.p.inv, inv);
    assert_eq!(g.w.p.npcs[0].friend, friend + 4);
}

#[test]
fn m3_home_invitation_late_rollover_decline_and_full_pockets() {
    let mut g = g0(at(12, 12, 0, 0));
    g.hr = 12;
    g.w.p.invitation =
        Some(SocialInvite { day: g.w.day, slot: 0, character: g.w.roster[0], kind: 0, hour: 13, state: 0 });
    g.me.ui = Ui::Invitation;
    g.update_player(&Inp::keys(&[], &[KeyCode::Escape]), 0.016);
    assert_eq!(g.w.p.invitation.as_ref().unwrap().state, 4);
    g.w.p.invitation.as_mut().unwrap().state = 0;
    g.accept_social().unwrap();
    g.hr = 12;
    g.swap(Scene::Room(HOUSES[0]));
    assert_eq!(g.w.p.invitation.as_ref().unwrap().state, 1); // too early
    g.hr = 14;
    g.social_tick();
    assert_eq!(g.w.p.invitation.as_ref().unwrap().state, 3);
    let friend = g.w.p.npcs[0].friend;
    assert!(g.catch_up(at(13, 6, 0, 0)));
    g.talk(0);
    let Ui::Talk { lines, .. } = &g.me.ui else { panic!() };
    assert!(lines[0].contains("saved a chair"));
    assert_eq!(g.w.p.npcs[0].friend, friend);
    assert!(g.w.p.missed_visit.is_none());
    g.w.p.invitation =
        Some(SocialInvite { day: g.w.day, slot: 0, character: g.w.roster[0], kind: 0, hour: 13, state: 1 });
    for _ in 0..INV_CAP {
        g.add_furn(50);
    }
    g.hr = 13;
    g.swap(Scene::Room(HOUSES[0]));
    assert!(g.w.p.mail.iter().any(|l| l.item == Some(FLOWERS[0])));
    let n = g.w.p.mail.len();
    g.swap(Scene::Room(HOUSES[0]));
    assert_eq!(g.w.p.mail.len(), n);
    g.w.p.invitation.as_mut().unwrap().state = 1;
    g.w.p.invitation.as_mut().unwrap().character = 99;
    g.social_tick();
    assert!(g.w.p.invitation.is_none());
}

#[test]
fn m3_contest_two_players_entries_prizes_and_reload() {
    let mut g = two(at(11, 9, 0, 0));
    g.hr = 9;
    assert_eq!(g.contest_kind(), Some(0));
    assert!(g.free(vec2(CONTEST_SPOT.0, CONTEST_SPOT.1)));
    g.contest_talk();
    assert!(g.contest_entry().is_some());
    g.w.p.inv.push(Slot { def: 16, qty: 1, uid: 0, size_mm: catch_base(16) * 135 / 100 });
    let score = g.contest_submit(0).unwrap();
    assert!(score >= 116);
    assert!(g.w.p.inv.is_empty());
    g.w.p.inv.push(Slot { def: 10, qty: 1, uid: 0, size_mm: 1 });
    assert!(g.contest_submit(0).is_err());
    assert_eq!(g.w.p.inv.len(), 1);
    assert!(g.contest_claim().is_err());
    g.activate(1);
    assert!(g.contest_entry().is_none());
    assert!(g.contest_submit(0).is_err());
    g.contest_talk();
    g.w.p.inv.push(Slot { def: 10, qty: 1, uid: 0, size_mm: catch_base(10) });
    g.contest_submit(0).unwrap();
    g.activate(0);
    let mut g = reload(&g, "m3_contest");
    g.join("Partner");
    g.hr = 17;
    let money = g.w.p.money;
    g.contest_claim().unwrap();
    assert!(g.w.p.money > money);
    assert!(g.contest_claim().is_err());
    assert!(g.w.p.inv.iter().any(|s| s.def == 170));
    let mut g = reload(&g, "m3_claimed");
    g.hr = 17;
    assert!(g.contest_claim().is_err());
    g.join("Partner");
    g.activate(1);
    assert!(!g.contest_entry().unwrap().claimed);
    g.contest_claim().unwrap();
    g.hr = 22;
    assert!(!g.contest_host_open());
}

#[test]
fn m3_contest_rollover_full_pockets_bug_date_and_legacy() {
    assert_eq!(contest_on(crate::clock::day_id(at(22, 12, 0, 0))), Some(1));
    assert_eq!(contest_on(crate::clock::day_id(at(18, 12, 0, 0))), None);
    let mut g = two(at(11, 16, 59, 0));
    g.hr = 16;
    g.contest_talk();
    g.w.p.inv.push(Slot { def: 16, qty: 1, uid: 0, size_mm: 500 });
    g.contest_submit(0).unwrap();
    g.activate(1);
    g.contest_talk();
    g.w.p.inv.push(Slot { def: 16, qty: 1, uid: 0, size_mm: 500 });
    g.contest_submit(0).unwrap();
    for _ in 0..INV_CAP {
        g.add_furn(50);
    }
    g.hr = 17;
    g.contest_claim().unwrap();
    assert!(g.w.p.mail.iter().any(|l| l.item == Some(170)));
    g.activate(0);
    assert!(g.catch_up(at(13, 6, 0, 0)));
    let n = g.w.p.mail.iter().filter(|l| l.from == CONTEST_HOST).count();
    assert_eq!(n, 1);
    assert!(!g.catch_up(at(13, 6, 1, 0)));
    assert_eq!(g.w.p.mail.iter().filter(|l| l.from == CONTEST_HOST).count(), n);
    let mut old = serde_json::to_value(g.persist()).unwrap();
    old["version"] = 4.into();
    old["p"].as_object_mut().unwrap().remove("contests");
    old["p"]["inv"] = serde_json::json!([{ "def": 12, "qty": 9, "uid": 0 }]);
    let w: World = serde_json::from_value(save::upgrade(old)).unwrap();
    assert_eq!(w.version, SAVE_VERSION);
    assert_eq!(w.p.inv[0].qty, 9);
    assert_eq!(w.p.inv[0].size_mm, 0);
    assert!(w.p.contests.is_empty());
}

#[test]
fn m3_measured_capture_paths_storage_and_gift_selection() {
    let mut g = g0(at(11, 12, 0, 0));
    g.w.p.equip = Some(60);
    g.me.face = vec2(1., 0.);
    g.bugs = vec![(7, g.me.pos + vec2(0.7, 0.), 0.)];
    g.use_tool();
    assert_eq!(g.w.p.inv.len(), 1);
    assert!(g.w.p.inv[0].size_mm > 0);
    g.w.p.equip = Some(61);
    g.shadows = vec![Shadow { def: 12, pos: g.me.pos, vel: Vec2::ZERO }];
    g.me.fish = Some(Fishing { bob: g.me.pos, fish: Some(0), t: 0., nib: 0, need: 0, bite: true, dip: 0. });
    g.use_tool();
    assert_eq!(g.w.p.inv[1].def, 12);
    let fish = g.w.p.inv[1];
    assert!(fish.size_mm > 0);
    g.transfer(0, 1);
    assert_eq!(g.w.p.storage[0], fish);
    g.transfer(1, 0);
    assert_eq!(g.w.p.inv[1], fish);
    g.w.p.inv.push(Slot { def: 12, qty: 1, uid: 0, size_mm: fish.size_mm + 1 });
    g.give_gift(0, 2);
    assert_eq!(g.w.p.inv[1], fish);
    let g2 = reload(&g, "m3_measurements");
    assert_eq!(g2.w.p.inv, g.w.p.inv);
}
