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
pub fn load(path: &str) -> Option<World> { let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?; let w: World = serde_json::from_value(upgrade(v)).ok()?; (w.version <= SAVE_VERSION).then_some(w) }
/// v3 (single profile at top level) -> v4 (`p` = profile 0, `others` = additional profiles). Shared fields are untouched.
pub fn upgrade(mut v: serde_json::Value) -> serde_json::Value {
    if v.get("version").and_then(|x| x.as_u64()) == Some(3) {
        if let Some(o) = v.as_object_mut() {
            let mut p = serde_json::Map::new(); p.insert("id".into(), 0.into());
            for k in ["name", "money", "home", "inv", "storage", "room", "npcs", "seen", "tools", "equip", "pos"] { if let Some(x) = o.remove(k) { p.insert(k.into(), x); } }
            o.insert("p".into(), serde_json::Value::Object(p)); o.insert("others".into(), serde_json::json!([])); o.insert("version".into(), 4.into());
        }
    }
    v
}
pub const BACKUPS: &str = "save_backups";
pub const KEEP_BACKUPS: usize = 5;

/// Copy `path` into `dir` with a timestamp; keep only the newest KEEP_BACKUPS backups of that file.
pub fn backup(path: &str, dir: &str) -> Option<String> {
    let src = std::path::Path::new(path); if !src.exists() { return None }
    fs::create_dir_all(dir).ok()?;
    let name = src.file_name()?.to_string_lossy().to_string();
    let dst = format!("{dir}/{name}.{}.bak", chrono::Local::now().format("%Y%m%d-%H%M%S%3f"));
    fs::copy(path, &dst).ok()?;
    let mut olds: Vec<std::path::PathBuf> = fs::read_dir(dir).ok()?.filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.file_name().is_some_and(|f| { let f = f.to_string_lossy(); f.starts_with(&format!("{name}.")) && f.ends_with(".bak") })).collect();
    olds.sort();
    while olds.len() > KEEP_BACKUPS { let _ = fs::remove_file(olds.remove(0)); }
    Some(dst)
}
pub enum Loaded { Ok(World), Missing, Corrupt(String) }
/// Load a JSON save; an unreadable/unparseable file is moved aside (never overwritten) and reported.
pub fn load_checked(path: &str) -> Loaded {
    let Ok(s) = fs::read_to_string(path) else { return Loaded::Missing };
    match serde_json::from_str::<serde_json::Value>(&s).ok().map(upgrade).and_then(|v| serde_json::from_value::<World>(v).ok()).ok_or(()) {
        Ok(w) if w.version <= SAVE_VERSION => Loaded::Ok(w),
        _ => { let dst = format!("{path}.corrupt-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")); let _ = fs::rename(path, &dst); Loaded::Corrupt(dst) }
    }
}

/// v1/v2 -> v3. Kept: name, gameplay day, money, house level, native fruit, pockets (same item ids), journal,
/// resident friendship/requests/talked flags, and (v2) the home room layout. Old test furniture maps to real
/// catalog items: Crate -> Cabinet, Bench -> Sofa (same uid, position, rotation). The old generated town layout
/// is replaced by the authored town, so the old outdoor position, fruit and weed state are dropped.
pub fn migrate(path: &str, map: &[u8]) -> Option<World> {
    let s = fs::read_to_string(path).ok()?; let l: Vec<&str> = s.lines().collect();
    if l.len() < 15 || !(l[0] == "v1" || l[0] == "v2") { return None }
    let mut w = World::fresh(l[1].to_string(), l[2].parse().ok()?, map);
    w.p.money = l[3].parse().ok()?; w.p.home = l[4].parse().ok()?; w.native = l[5].parse::<u16>().ok().filter(|f| FRUITS.contains(f)).unwrap_or(20);
    for id in l[8].split(',').filter_map(|x| x.parse::<u16>().ok()).filter(|&i| is_def(i)) {
        if let Some(s) = w.p.inv.iter_mut().find(|s| s.def == id && s.qty < STACK) { s.qty += 1 } else if w.p.inv.len() < INV_CAP { w.p.inv.push(Slot { def: id, qty: 1, uid: 0 }) }
    }
    w.p.seen = l[9].bytes().enumerate().filter(|(_, b)| *b == b'1').map(|(i, _)| i as u16).filter(|&i| is_def(i)).collect();
    for i in 0..4 { let v: Vec<f64> = l[11 + i].split(' ').filter_map(|x| x.parse().ok()).collect();
        if v.len() >= 3 { w.p.npcs[i].friend = v[0] as i32; w.p.npcs[i].req = Some(v[1] as u16).filter(|&r| r != 999 && is_def(r)); w.p.npcs[i].talked = v[2] != 0. } }
    if l.len() >= 17 && l[15].starts_with("room") { for e in l[16].split(',') { let v: Vec<i32> = e.split(':').filter_map(|x| x.parse().ok()).collect();
        if v.len() == 5 { let def = match v[1] { 0 => 55, 1 => 51, _ => continue }; w.p.room.push(Furn { uid: v[0] as u32, def, x: v[2], z: v[3], rot: v[4] as u8 % 4, on: false }) } } }
    w.uid = w.p.room.iter().map(|f| f.uid).max().unwrap_or(0) + 1;
    Some(w)
}
