//! Gamepads (via gilrs): slot assignment, disconnect/reconnect, and mapping pad state onto the same virtual
//! key events the keyboard produces, so every menu/dialogue/placement path works unchanged.
//! Physical hardware has NOT been verified by the agent (see HUMAN_VERIFICATION.md).
use crate::world::Inp;
use macroquad::prelude::KeyCode;
use std::collections::{HashMap, HashSet};

pub const DEAD: f32 = 0.35;
const REPEAT_DELAY: f32 = 0.4;
const REPEAT_RATE: f32 = 0.12;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum B { South, East, West, North, Lb, Rb, Lt, Rt, Start, Select, Up, Down, Left, Right }
/// Raw state of one pad for one frame. `ly` > 0 means stick pushed up.
#[derive(Default, Clone, Debug)]
pub struct PadState { pub b: HashSet<B>, pub lx: f32, pub ly: f32 }

/// Button -> virtual keys. Provisional layout: A confirm/talk, B cancel, X use tool, Y pockets (and "sell all" in
/// the sell menu), RB cycle tool / rotate, LB pick up furniture, Select move furniture, Start journal, triggers run.
fn keys_for(b: B) -> &'static [KeyCode] {
    use KeyCode as K;
    match b { B::South => &[K::E, K::Enter], B::East => &[K::Escape], B::West => &[K::Space], B::North => &[K::I, K::X], B::Rb => &[K::Tab, K::R], B::Lb => &[K::G], B::Select => &[K::F], B::Start => &[K::J], _ => &[] }
}

#[derive(Default)]
pub struct Mapper { prev: HashSet<B>, held: [bool; 4], hold: [f32; 4] }
impl Mapper {
    /// Convert one frame of pad state into keyboard-equivalent input (edges + menu auto-repeat on directions).
    pub fn map(&mut self, s: &PadState, dt: f32) -> Inp {
        let mut inp = Inp::none();
        let dirs = [s.b.contains(&B::Up) || s.ly > DEAD, s.b.contains(&B::Down) || s.ly < -DEAD, s.b.contains(&B::Left) || s.lx < -DEAD, s.b.contains(&B::Right) || s.lx > DEAD];
        let keys = [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right];
        for k in 0..4 {
            if dirs[k] {
                inp.down.insert(keys[k]);
                if !self.held[k] { inp.pressed.insert(keys[k]); self.hold[k] = 0. }
                else { self.hold[k] += dt; if self.hold[k] >= REPEAT_DELAY { inp.pressed.insert(keys[k]); self.hold[k] -= REPEAT_RATE } }
            }
            self.held[k] = dirs[k];
        }
        if s.b.contains(&B::Lt) || s.b.contains(&B::Rt) { inp.down.insert(KeyCode::LeftShift); }
        for &b in &s.b { for &k in keys_for(b) { inp.down.insert(k); if !self.prev.contains(&b) { inp.pressed.insert(k); } } }
        self.prev = s.b.clone();
        inp
    }
    pub fn reset(&mut self) { *self = Mapper::default() }
}

/// Which pad drives which player slot. Slot 1 (player 2) is claimed first because the keyboard is always player 1.
#[derive(Default, Debug)]
pub struct Slots { pub ids: [Option<usize>; 2], pub connected: HashSet<usize> }
impl Slots {
    pub fn connect(&mut self, id: usize) { self.connected.insert(id); }
    pub fn disconnect(&mut self, id: usize) { self.connected.remove(&id); }
    /// A pad pressed Start/A. Assign it to a free slot, or to a slot whose pad is gone (reassignment). Returns the slot.
    pub fn claim(&mut self, id: usize) -> Option<usize> {
        if self.ids.contains(&Some(id)) { return None }
        for k in [1, 0] { if self.ids[k].map_or(true, |p| !self.connected.contains(&p)) { self.ids[k] = Some(id); return Some(k) } }
        None
    }
    pub fn slot_of(&self, id: usize) -> Option<usize> { self.ids.iter().position(|s| *s == Some(id)) }
    /// true when a pad was assigned to the slot but is currently disconnected
    pub fn lost(&self, slot: usize) -> bool { self.ids[slot].is_some_and(|p| !self.connected.contains(&p)) }
    pub fn active(&self, slot: usize) -> bool { self.ids[slot].is_some_and(|p| self.connected.contains(&p)) }
}

pub struct Pads { gil: Option<gilrs::Gilrs>, map: HashMap<usize, gilrs::GamepadId>, pub slots: Slots, mappers: [Mapper; 2], pub claimed: Vec<usize> }
impl Pads {
    pub fn new() -> Pads {
        let gil = gilrs::Gilrs::new().ok(); let mut p = Pads { gil, map: HashMap::new(), slots: Slots::default(), mappers: Default::default(), claimed: vec![] };
        if let Some(g) = &p.gil { for (id, _) in g.gamepads() { let u: usize = id.into(); p.map.insert(u, id); p.slots.connect(u) } }
        p
    }
    /// Process connection events and Start/A presses; newly claimed slots are pushed to `claimed`.
    pub fn poll(&mut self) {
        let Some(g) = self.gil.as_mut() else { return };
        while let Some(gilrs::Event { id, event, .. }) = g.next_event() {
            let u: usize = id.into(); self.map.insert(u, id);
            match event {
                gilrs::EventType::Connected => self.slots.connect(u),
                gilrs::EventType::Disconnected => { self.slots.disconnect(u); if let Some(s) = self.slots.slot_of(u) { self.mappers[s].reset() } }
                gilrs::EventType::ButtonPressed(gilrs::Button::Start | gilrs::Button::South, _) => { self.slots.connect(u); if let Some(s) = self.slots.claim(u) { self.mappers[s].reset(); self.claimed.push(s) } }
                _ => {}
            }
        }
    }
    fn state(&self, slot: usize) -> PadState {
        let mut s = PadState::default();
        let (Some(g), Some(u)) = (self.gil.as_ref(), self.slots.ids[slot]) else { return s };
        if !self.slots.connected.contains(&u) { return s }
        let Some(&id) = self.map.get(&u) else { return s }; let gp = g.gamepad(id);
        use gilrs::Button as G;
        for (gb, b) in [(G::South, B::South), (G::East, B::East), (G::West, B::West), (G::North, B::North), (G::LeftTrigger, B::Lb), (G::RightTrigger, B::Rb), (G::LeftTrigger2, B::Lt), (G::RightTrigger2, B::Rt),
            (G::Start, B::Start), (G::Select, B::Select), (G::DPadUp, B::Up), (G::DPadDown, B::Down), (G::DPadLeft, B::Left), (G::DPadRight, B::Right)] { if gp.is_pressed(gb) { s.b.insert(b); } }
        s.lx = gp.value(gilrs::Axis::LeftStickX); s.ly = gp.value(gilrs::Axis::LeftStickY); s
    }
    /// Keyboard-equivalent input for a player slot this frame (empty if no pad is assigned/connected).
    pub fn inp(&mut self, slot: usize, dt: f32) -> Inp { let s = self.state(slot); self.mappers[slot].map(&s, dt) }
    pub fn count(&self) -> usize { self.slots.connected.len() }
}
pub fn merge(a: Inp, b: Inp) -> Inp { Inp { down: a.down.union(&b.down).copied().collect(), pressed: a.pressed.union(&b.pressed).copied().collect() } }

#[cfg(test)]
mod tests {
    use super::*;
    fn st(b: &[B], lx: f32, ly: f32) -> PadState { PadState { b: b.iter().copied().collect(), lx, ly } }
    #[test] fn dead_zone_and_diagonal() {
        let mut m = Mapper::default();
        let i = m.map(&st(&[], 0.2, -0.3), 0.016); assert!(i.down.is_empty()); // inside dead zone
        let i = m.map(&st(&[], 0.7, 0.7), 0.016); assert!(i.down.contains(&KeyCode::Right) && i.down.contains(&KeyCode::Up)); // diagonal -> two keys, game normalizes
    }
    #[test] fn button_edges_and_repeat() {
        let mut m = Mapper::default();
        let i = m.map(&st(&[B::South], 0., 0.), 0.016); assert!(i.pressed.contains(&KeyCode::E) && i.pressed.contains(&KeyCode::Enter));
        let i = m.map(&st(&[B::South], 0., 0.), 0.016); assert!(i.down.contains(&KeyCode::E) && !i.pressed.contains(&KeyCode::E)); // held: no new edge
        let mut presses = 0; for f in 0..60 { let i = m.map(&st(&[B::Down], 0., 0.), 1. / 60.); if i.pressed.contains(&KeyCode::Down) { presses += 1 } let _ = f; }
        assert!((3..=6).contains(&presses), "repeat count {presses}"); // 1 initial + repeats after 0.4 s every 0.12 s
    }
    #[test] fn slots_assign_disconnect_reassign() {
        let mut s = Slots::default(); s.connect(10); s.connect(11);
        assert_eq!(s.claim(10), Some(1)); assert_eq!(s.claim(10), None); assert_eq!(s.claim(11), Some(0));
        s.disconnect(10); assert!(s.lost(1)); assert!(!s.active(1));
        s.connect(12); assert_eq!(s.claim(12), Some(1)); assert!(s.active(1)); // another pad takes over player 2
        s.connect(10); assert_eq!(s.claim(10), None); // old pad no longer has a free slot
    }
}
