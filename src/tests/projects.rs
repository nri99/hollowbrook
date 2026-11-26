//! Tests: community projects (X-01)
use super::*;

// ---- community projects (X-01) ----
#[test]
fn community_project_two_players_build_bench() {
    let mut g = two(at(11, 12, 0, 0));
    let p = g.w.project.clone().expect("proposal");
    assert_eq!(p.kind, 0);
    assert_eq!(tile_at(&g.map, p.x, p.z), GRASS);
    assert!(!g.walk_tile(p.x, p.z)); // sign blocks
    let l0 = g.liveliness();
    g.activate(0);
    for _ in 0..3 {
        g.add(80);
    }
    g.add(20);
    g.contribute();
    let q = g.w.project.clone().unwrap();
    assert_eq!(q.need[0].2, 3);
    assert_eq!(g.w.p.inv.iter().filter(|s| s.def == 80).count(), 0);
    assert!(g.w.p.inv.iter().any(|s| s.def == 20)); // fruit not needed: kept
    g.activate(1);
    for _ in 0..4 {
        g.add(80);
    }
    for _ in 0..3 {
        g.add(81);
    }
    g.contribute(); // gives 2 branches + 2 stones only
    assert!(g.w.project.is_none());
    assert_eq!(g.w.built.len(), 1);
    assert_eq!(g.w.p.inv.iter().find(|s| s.def == 80).map(|s| s.qty), Some(2));
    assert_eq!(g.w.p.inv.iter().find(|s| s.def == 81).map(|s| s.qty), Some(1));
    assert!(!g.walk_tile(p.x, p.z));
    assert!(g.liveliness() > l0, "{} vs {}", g.liveliness(), l0);
    for b in 0..BLDS.len() {
        assert!(g.bfs(door_tile(SHOP), door_tile(b), true).is_some())
    }
    let g2 = reload(&g, "proj");
    assert_eq!(g2.w.built, g.w.built);
}
#[test]
fn bridge_project_becomes_walkable_and_materials_once_per_day() {
    let mut g = g0(at(11, 12, 0, 0));
    g.w.project = Some(Project { kind: PK_BRIDGE, x: 25, z: 21, need: vec![(0, 1, 0)], by: 0, log: vec![] });
    assert!(!g.walk_tile(25, 21));
    g.add(80);
    g.contribute();
    assert!(g.walk_tile(25, 21) && g.walk_tile(26, 21));
    let ti = g.w.trees.iter().position(|t| g.walk_tile(t.x, t.z + 1)).unwrap();
    let n0 = g.w.ground.len();
    g.w.trees[ti].fruit = false;
    g.shake_tree_pub(ti);
    assert_eq!(g.w.ground.len(), n0 + 1);
    g.shake_tree_pub(ti);
    assert_eq!(g.w.ground.len(), n0 + 1);
}
