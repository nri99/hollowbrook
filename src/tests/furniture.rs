//! Tests: furniture
use super::*;

// ---- furniture ----
#[test]
fn furniture_buy_place_move_rotate_pick_no_dup() {
    let mut g = g0(at(11, 12, 0, 0));
    home(&mut g);
    g.add_furn(51);
    let uid = g.w.p.inv[0].uid;
    g.start_place(uid);
    g.me.placing.as_mut().unwrap().f.x = 0;
    g.me.placing.as_mut().unwrap().f.z = 0;
    g.place_confirm().unwrap();
    assert!(g.w.p.inv.is_empty());
    assert_eq!(g.w.p.room.len(), 1);
    assert_eq!(g.w.p.room[0].uid, uid);
    g.start_move(uid);
    assert!(g.w.p.room.is_empty());
    assert_eq!(g.persist().p.room.len(), 1); // still owned while moving
    {
        let p = g.me.placing.as_mut().unwrap();
        p.f.rot = 1;
        p.f.x = 0;
        p.f.z = 0;
    }
    g.place_confirm().unwrap();
    assert!(g.room_block(HOME, 0, 1, 0) && !g.room_block(HOME, 1, 0, 0)); // rotated footprint covers (0,0),(0,1)
    g.start_move(uid);
    g.place_cancel();
    assert_eq!(g.w.p.room.len(), 1);
    assert_eq!(g.w.p.room[0].rot, 1);
    g.pick_furn(uid);
    assert!(g.w.p.room.is_empty());
    assert_eq!(g.w.p.inv.iter().filter(|s| s.uid == uid).count(), 1);
    let g2 = reload(&g, "furn");
    assert_eq!(g2.w.p.inv, g.w.p.inv);
}
#[test]
fn placement_rejections() {
    let mut g = g0(at(11, 12, 0, 0));
    home(&mut g);
    let mk = |d: u16, x: i32, z: i32, rot: u8| Furn { uid: 99, def: d, x, z, rot, on: false };
    assert_eq!(g.place_ok(&mk(50, 2, 3, 0)), Err("Keep the doorway clear."));
    assert_eq!(g.place_ok(&mk(51, 3, 0, 0)), Err("It doesn't fit there."));
    g.me.ipos = vec2(2.5, 2.5);
    assert_eq!(g.place_ok(&mk(50, 2, 2, 0)), Err("You're standing there."));
    g.me.ipos = vec2(2.5, 3.5);
    g.w.p.room.push(Furn { uid: 5, def: 52, x: 0, z: 0, rot: 0, on: false });
    assert_eq!(g.place_ok(&mk(51, 0, 0, 0)), Err("Something is in the way."));
    assert_eq!(g.place_ok(&mk(57, 1, 0, 0)), Err("Wall items go on the back wall."));
    assert!(g.place_ok(&mk(57, 1, -1, 0)).is_ok());
    g.me.ipos = vec2(3.5, 0.5);
    g.w.p.room.push(Furn { uid: 6, def: 50, x: 3, z: 1, rot: 0, on: false });
    assert_eq!(g.place_ok(&mk(50, 2, 0, 0)), Err("That would block your way out."));
}
#[test]
fn furniture_interactions_do_not_skip_time() {
    let mut g = g0(at(11, 12, 0, 0));
    home(&mut g);
    for (u, d, x) in [(11, 53, 0), (12, 54, 3), (13, 55, 1)] {
        g.w.p.room.push(Furn { uid: u, def: d, x, z: 0, rot: 0, on: false })
    }
    let day = g.w.day;
    g.me.ipos = vec2(0.5, 2.4);
    g.me.face = vec2(0., -1.);
    g.interact();
    assert_eq!(g.me.pose, Pose::Lie(11));
    assert_eq!(g.w.day, day);
    g.step_move(vec2(0., 1.), false, 0.016);
    assert_eq!(g.me.pose, Pose::Stand);
    g.me.ipos = vec2(3.5, 1.5);
    g.me.face = vec2(0., -1.);
    g.interact();
    assert!(g.w.p.room.iter().any(|f| f.uid == 12 && f.on));
    g.me.ipos = vec2(1.5, 1.5);
    g.interact();
    assert_eq!(g.me.ui, Ui::Storage(0, 0));
}
