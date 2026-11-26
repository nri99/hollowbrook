//! Tests: save safety
use super::*;

// ---- save safety ----
#[test]
fn backups_rotate() {
    let d = std::env::temp_dir().join("hb3_bk");
    let _ = std::fs::remove_dir_all(&d);
    let f = std::env::temp_dir().join("hb3_bk_save.json");
    std::fs::write(&f, "x").unwrap();
    for _ in 0..8 {
        assert!(save::backup(f.to_str().unwrap(), d.to_str().unwrap()).is_some());
        std::thread::sleep(std::time::Duration::from_millis(5))
    }
    assert_eq!(std::fs::read_dir(&d).unwrap().count(), save::KEEP_BACKUPS);
}
#[test]
fn corrupt_save_moved_aside() {
    let f = std::env::temp_dir().join("hb3_corrupt.json");
    std::fs::write(&f, "{not json").unwrap();
    match save::load_checked(f.to_str().unwrap()) {
        save::Loaded::Corrupt(p) => {
            assert!(!f.exists());
            assert_eq!(std::fs::read_to_string(&p).unwrap(), "{not json");
            let _ = std::fs::remove_file(p);
        }
        _ => panic!("expected corrupt"),
    }
}
#[test]
fn trampled_flowers_recover_next_morning() {
    let mut g = g0(at(11, 12, 0, 0));
    let i = g.map.iter().position(|&c| c == FLOWER).unwrap();
    let t = ((i % W) as i32, (i / W) as i32);
    let mut n = 0;
    for _ in 0..400 {
        g.me.pos = tc(t) - vec2(0.3, 0.);
        g.step_move(vec2(1., 0.), true, 0.016);
        n += 1;
        if g.w.trampled.contains(&t) {
            break;
        }
    }
    assert!(g.w.trampled.contains(&t), "not trampled after {n} runs");
    let g2 = reload(&g, "tr");
    assert!(g2.w.trampled.contains(&t));
    let mut g3 = g2;
    assert!(g3.catch_up(at(12, 6, 0, 1)));
    assert!(g3.w.trampled.is_empty());
    let mut g4 = g0(at(11, 12, 0, 0));
    for _ in 0..400 {
        g4.me.pos = tc(t) - vec2(0.3, 0.);
        g4.step_move(vec2(1., 0.), false, 0.016)
    }
    assert!(g4.w.trampled.is_empty()); // walking never tramples
}
