//! JSON save (v3) plus explicit migration from the earlier v1/v2 text saves.
use crate::data::*;
use crate::world::*;
use std::fs;

pub const SAVE: &str = "hollowbrook_save.json";
pub const LEGACY: &str = "hollowbrook_save.txt";

pub fn save(w: &World, path: &str) {
    let tmp = format!("{path}.tmp");
    if let Ok(s) = serde_json::to_string_pretty(w) { if fs::write(&tmp, s).is_ok() { let _ = fs::rename(&tmp, path); } }
}
pub fn load(path: &str) -> Option<World> { let w: World = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?; (w.version <= SAVE_VERSION).then_some(w) }

/// v1/v2 -> v3. Kept: name, gameplay day, money, house level, native fruit, pockets (same item ids), journal,
/// resident friendship/requests/talked flags, and (v2) the home room layout. Old test furniture maps to real
/// catalog items: Crate -> Cabinet, Bench -> Sofa (same uid, position, rotation). The old generated town layout
/// is replaced by the authored town, so the old outdoor position, fruit and weed state are dropped.
pub fn migrate(path: &str, map: &[u8]) -> Option<World> {
    let s = fs::read_to_string(path).ok()?; let l: Vec<&str> = s.lines().collect();
    if l.len() < 15 || !(l[0] == "v1" || l[0] == "v2") { return None }
    let mut w = World::fresh(l[1].to_string(), l[2].parse().ok()?, map);
    w.money = l[3].parse().ok()?; w.home = l[4].parse().ok()?; w.native = l[5].parse::<u16>().ok().filter(|f| FRUITS.contains(f)).unwrap_or(20);
    for id in l[8].split(',').filter_map(|x| x.parse::<u16>().ok()).filter(|&i| is_def(i)) {
        if let Some(s) = w.inv.iter_mut().find(|s| s.def == id && s.qty < STACK) { s.qty += 1 } else if w.inv.len() < INV_CAP { w.inv.push(Slot { def: id, qty: 1, uid: 0 }) }
    }
    w.seen = l[9].bytes().enumerate().filter(|(_, b)| *b == b'1').map(|(i, _)| i as u16).filter(|&i| is_def(i)).collect();
    for i in 0..4 { let v: Vec<f64> = l[11 + i].split(' ').filter_map(|x| x.parse().ok()).collect();
        if v.len() >= 3 { w.npcs[i].friend = v[0] as i32; w.npcs[i].req = Some(v[1] as u16).filter(|&r| r != 999 && is_def(r)); w.npcs[i].talked = v[2] != 0. } }
    if l.len() >= 17 && l[15].starts_with("room") { for e in l[16].split(',') { let v: Vec<i32> = e.split(':').filter_map(|x| x.parse().ok()).collect();
        if v.len() == 5 { let def = match v[1] { 0 => 55, 1 => 51, _ => continue }; w.room.push(Furn { uid: v[0] as u32, def, x: v[2], z: v[3], rot: v[4] as u8 % 4, on: false }) } } }
    w.uid = w.room.iter().map(|f| f.uid).max().unwrap_or(0) + 1;
    Some(w)
}
