//! Original procedural music: 24 hourly compositions sharing one instrument palette, with normal / rain / indoor
//! arrangements, generated off the main thread and crossfaded. Music volume is separate from sound effects.
//! The agent cannot listen; audible quality is for the user to judge.
use crate::data::{weather, Wx};
use crate::world::{Game, Scene};
use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use std::sync::mpsc::{channel, Receiver, TryRecvError};

pub const SR: f32 = 22050.;
pub const FADE: f32 = 3.0; // seconds for every crossfade
pub const SETTINGS: &str = "hollowbrook_settings.json";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Var { Normal, Rain, Indoor, Festive }

// ---------------- settings (separate from the world save) ----------------
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Settings { pub music: f32, pub sfx: f32, pub muted: bool }
impl Default for Settings { fn default() -> Self { Settings { music: 0.6, sfx: 0.8, muted: false } } }
pub fn load_settings(path: &str) -> Settings {
    std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .map(|s| Settings { music: s.music.clamp(0., 1.), sfx: s.sfx.clamp(0., 1.), muted: s.muted }).unwrap_or_default()
}
pub fn save_settings(path: &str, s: &Settings) { if let Ok(j) = serde_json::to_string_pretty(s) { let _ = std::fs::write(path, j); } }
/// Volume actually applied to a music track.
pub fn effective(fade: f32, vol: f32, muted: bool) -> f32 { if muted { 0. } else { fade.clamp(0., 1.) * vol.clamp(0., 1.) } }

/// One arrangement for everyone (no competing tracks): outdoor if any local player is outdoors, else indoor; rain/snow softens it.
pub fn variant(g: &Game) -> Var {
    let outdoor = std::iter::once(&g.me).chain(g.others.iter()).any(|p| !matches!(p.scene, Scene::Room(_)));
    if !outdoor { Var::Indoor } else if g.fest().is_some() || g.fair() { Var::Festive } else if weather(g.w.day, g.mo) != Wx::Clear { Var::Rain } else { Var::Normal }
}

// ---------------- composition ----------------
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u32 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (self.0 >> 33) as u32 }
    fn range(&mut self, n: usize) -> usize { self.next() as usize % n }
    fn chance(&mut self, p: f32) -> bool { (self.next() % 1000) as f32 / 1000. < p }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Inst { MusicBox, Keys, Flute, Bass, Pad }
const MAJOR: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
const MIXO: [i32; 7] = [0, 2, 4, 5, 7, 9, 10];
const DORIAN: [i32; 7] = [0, 2, 3, 5, 7, 9, 10];
const PROGS: [[i32; 4]; 6] = [[0, 5, 3, 4], [0, 3, 1, 4], [5, 3, 0, 4], [0, 2, 3, 4], [3, 4, 2, 5], [0, 4, 5, 3]];
const RHYTHMS: [&[u32]; 9] = [&[2, 2, 2, 2], &[3, 1, 2, 2], &[2, 1, 1, 2, 2], &[4, 2, 2], &[1, 1, 2, 1, 1, 2], &[2, 2, 4], &[3, 3, 2], &[6, 2], &[2, 4, 2]];

/// Musical plan for an hour (all values are original design choices).
#[derive(Clone, Debug)]
pub struct Plan { pub bpm: f32, pub root: f32, pub scale: [i32; 7], pub prog: [i32; 4], pub swing: f32, pub lead: Inst, pub energy: f32, pub gain: f32, pub melody: Vec<(f32, f32, i32)> }

fn semis(scale: &[i32; 7], deg: i32) -> i32 { let o = deg.div_euclid(7); scale[deg.rem_euclid(7) as usize] + 12 * o }

/// One phrase of `bars` bars: (beat, length in beats, scale degree). Chord tones on bar starts, mostly stepwise motion.
fn phrase(r: &mut Rng, bars: usize, chords: &[i32], lo: i32, hi: i32, rest_p: f32, end_on: Option<i32>) -> Vec<(f32, f32, i32)> {
    let mut out = vec![]; let mut prev = chords[0] + 2 + 7;
    for b in 0..bars {
        let c = chords[b % chords.len()]; let rhythm = RHYTHMS[r.range(RHYTHMS.len())]; let mut pos = 0u32;
        for (k, &len) in rhythm.iter().enumerate() {
            let last = b == bars - 1 && k == rhythm.len() - 1;
            if k > 0 && !last && r.chance(rest_p) { pos += len; continue }
            let deg = if let (true, Some(e)) = (last, end_on) { e } else if pos % 4 == 0 {
                let tones = [c, c + 2, c + 4, c + 7, c + 9]; *tones.iter().filter(|&&t| t >= lo && t <= hi).min_by_key(|&&t| (t - prev).abs()).unwrap_or(&(c + 7))
            } else { (prev + [-2, -1, -1, 1, 1, 2][r.range(6)]).clamp(lo, hi) };
            let dur = if last { len.max(4) } else { len } as f32 / 2.;
            out.push(((b * 4) as f32 + pos as f32 / 2., dur, deg)); prev = deg; pos += len;
        }
    }
    out
}

pub fn plan(hour: u32) -> Plan {
    let h = hour % 24; let mut r = Rng(0x9E37_79B9 ^ (h as u64 * 7919 + 13));
    let (bpm0, energy, gain) = match h { 0..=4 | 22 | 23 => (62., 0.2, 0.55), 5..=7 => (74., 0.45, 0.75), 8..=11 => (92., 0.8, 1.0), 12..=16 => (100., 0.9, 1.0), _ => (80., 0.55, 0.8) };
    let bpm = bpm0 + (h % 3) as f32 * 3.;
    let root_semis = [0, 7, 2, 9, 4, 11, 5, 10, 3, 8, 1, 6][(h % 12) as usize] as f32 - if h % 2 == 1 { 12. } else { 0. }; // walk the circle of fifths
    let root = 220. * 2f32.powf(root_semis / 12.).max(0.75).min(1.5);
    let scale = if energy < 0.3 { if h % 2 == 0 { DORIAN } else { MAJOR } } else if h % 4 == 1 { MIXO } else { MAJOR };
    let prog = PROGS[(h as usize * 5 + 1) % PROGS.len()];
    let swing = if h % 3 == 0 { 0.62 } else { 0.5 };
    let lead = if energy < 0.3 { Inst::Keys } else { [Inst::MusicBox, Inst::Flute, Inst::Keys][(h % 3) as usize] };
    let rest_p = if energy < 0.3 { 0.35 } else { 0.12 };
    // 16 bars: A A' B A'' with 2 bars per chord (progression plays twice)
    let ch = |i: usize| prog[(i / 2) % 4];
    let chords_a: Vec<i32> = (0..4).map(ch).collect(); let chords_b: Vec<i32> = (4..8).map(ch).collect();
    let a = phrase(&mut r, 4, &chords_a, 5, 12, rest_p, None);
    let mut a2: Vec<(f32, f32, i32)> = a.iter().copied().filter(|n| n.0 < 12.).collect(); a2.extend(phrase(&mut r, 1, &chords_a[3..], 5, 12, rest_p, Some(chords_a[3] + 7)).into_iter().map(|n| (n.0 + 12., n.1, n.2)));
    let bph = phrase(&mut r, 4, &chords_b, 8, 14, rest_p * 0.8, None);
    let mut a3: Vec<(f32, f32, i32)> = a.iter().copied().filter(|n| n.0 < 4. || (8. ..12.).contains(&n.0)).collect();
    a3.extend(phrase(&mut r, 1, &chords_a[1..2], 5, 12, rest_p, None).into_iter().map(|n| (n.0 + 4., n.1, n.2)));
    a3.extend(phrase(&mut r, 1, &chords_a[3..], 5, 12, rest_p, Some(7)).into_iter().map(|n| (n.0 + 12., n.1, n.2)));
    let mut melody = vec![]; for (off, p) in [(0., &a), (16., &a2), (32., &bph), (48., &a3)] { melody.extend(p.iter().map(|n| (n.0 + off, n.1, n.2))) }
    melody.sort_by(|x, y| x.0.total_cmp(&y.0));
    Plan { bpm, root, scale, prog, swing, lead, energy, gain, melody }
}

fn hn(k: usize) -> f32 { let x = (k as u32).wrapping_mul(2654435761) ^ 0x5bd1e995; ((x.wrapping_mul(x ^ (x >> 15))) >> 8) as f32 / 8388608. - 1. } // deterministic noise

/// Add one note into a looping buffer (tails wrap to the start, so the loop seam is seamless). Envelopes start and end at 0.
fn note(buf: &mut [f32], inst: Inst, f: f32, start: f32, dur: f32, vel: f32) {
    let n_buf = buf.len(); let s0 = (start * SR) as usize;
    let (att, rel, ring) = match inst { Inst::MusicBox => (0.004, 0.05, 1.5), Inst::Keys => (0.008, 0.25, 0.0), Inst::Flute => (0.07, 0.12, 0.0), Inst::Bass => (0.02, 0.1, 0.0), Inst::Pad => (0.4, 0.6, 0.0) };
    let len = if inst == Inst::MusicBox { dur.max(0.25) + ring } else { dur + rel }; let n = (len * SR) as usize; let mut lp = 0f32;
    for k in 0..n {
        let t = k as f32 / SR; let w = TAU * f * t;
        let a = (t / att).min(1.); let tail = ((len - t) / 0.02).clamp(0., 1.);
        let rel_env = if inst != Inst::MusicBox && t > dur { (1. - (t - dur) / rel).max(0.) } else { 1. };
        let env = a * tail * rel_env * match inst { Inst::MusicBox => (-2.4 * t).exp(), Inst::Keys => (-1.6 * t).exp() * 0.85 + 0.15, _ => 1. };
        let v = match inst {
            Inst::MusicBox => w.sin() + 0.3 * (2. * w).sin() * (-4. * t).exp() + 0.1 * (3. * w).sin() * (-6. * t).exp() + 0.05 * (4.2 * w).sin() * (-9. * t).exp(),
            Inst::Keys => (w + 1.1 * (-3. * t).exp() * (2. * w).sin()).sin(),
            Inst::Flute => { let vib = 1. + 0.004 * (TAU * 5. * t).sin() * (t / 0.3).min(1.); lp += (hn(s0 + k) - lp) * 0.08; (w * vib).sin() + 0.12 * (2. * w * vib).sin() + 0.25 * lp }
            Inst::Bass => w.sin() + 0.15 * (2. * w).sin(),
            Inst::Pad => w.sin() + 0.5 * (w * 1.004).sin() + 0.25 * (2. * w * 0.997).sin(),
        };
        buf[(s0 + k) % n_buf] += v * env * vel;
    }
}
fn perc(buf: &mut [f32], shaker: bool, start: f32, vel: f32) {
    let n_buf = buf.len(); let s0 = (start * SR) as usize; let n = (if shaker { 0.05 } else { 0.035 } * SR) as usize; let mut lp = 0f32;
    for k in 0..n { let t = k as f32 / SR; let a = (t / 0.002).min(1.);
        let v = if shaker { lp += (hn(s0 * 3 + k) - lp) * 0.35; lp * a * (-t * 70.).exp() } else { (TAU * 950. * t).sin() * a * (-t * 90.).exp() };
        buf[(s0 + k) % n_buf] += v * vel }
}
/// One-pole low-pass over a looping buffer, run twice so the filter state matches across the loop seam.
fn lowpass_loop(buf: &mut [f32], alpha: f32) { let mut y = 0.; for &x in buf.iter() { y += (x - y) * alpha } for x in buf.iter_mut() { y += (*x - y) * alpha; *x = y } }

/// Render the hour's composition for one arrangement as mono samples (exact loop).
pub fn compose(hour: u32, var: Var) -> Vec<f32> {
    let p = plan(hour); let beat = 60. / p.bpm; let n = (64. * beat * SR) as usize; let mut buf = vec![0f32; n];
    let freq = |deg: i32, oct: f32| p.root * oct * 2f32.powf(semis(&p.scale, deg) as f32 / 12.);
    let sw = |b: f32| { let f = b.fract(); b.floor() + if (f - 0.5).abs() < 0.01 { p.swing } else { f } };
    let lead_vel = if var == Var::Rain { 0.55 } else { 0.8 };
    for &(b, d, deg) in &p.melody { note(&mut buf, p.lead, freq(deg, 1.), sw(b) * beat, d * beat * 0.92, lead_vel); if var == Var::Festive { note(&mut buf, Inst::Keys, freq(deg, 2.), sw(b) * beat, d * beat * 0.6, 0.1) } } // festive: bell doubling an octave up
    for bar in 0..16 { let c = p.prog[(bar / 2) % 4]; let t0 = (bar * 4) as f32 * beat;
        if p.energy < 0.3 && var != Var::Festive { if bar % 2 == 0 { for d in [c, c + 2, c + 4] { note(&mut buf, Inst::Pad, freq(d, 0.5), t0, 8. * beat * 0.95, 0.12) } } note(&mut buf, Inst::Bass, freq(c, 0.25), t0, 4. * beat * 0.9, 0.35) }
        else {
            let arp = [c, c + 2, c + 4, c + 2, c + 7, c + 4, c + 2, c + 4];
            for (k, &d) in arp.iter().enumerate() { if p.energy < 0.6 && k % 2 == 1 && var != Var::Festive { continue } note(&mut buf, Inst::Keys, freq(d, 0.5), t0 + sw(k as f32 / 2.) * beat, 0.45 * beat, 0.16) }
            note(&mut buf, Inst::Bass, freq(c, 0.25), t0, 1.8 * beat, 0.4); note(&mut buf, Inst::Bass, freq(c + 4, 0.25), t0 + 2. * beat, 1.8 * beat, 0.32);
            if (var == Var::Normal && p.energy >= 0.6) || var == Var::Festive { for k in 0..8 { if k % 2 == 1 { perc(&mut buf, true, t0 + sw(k as f32 / 2.) * beat, 0.05) } } for k in [1., 3.] { perc(&mut buf, false, t0 + k * beat, 0.05) } }
        }
    }
    match var { Var::Indoor => lowpass_loop(&mut buf, 0.25), Var::Rain => lowpass_loop(&mut buf, 0.5), Var::Normal => lowpass_loop(&mut buf, 0.85), Var::Festive => lowpass_loop(&mut buf, 0.7) }
    let peak = buf.iter().fold(0f32, |m, v| m.max(v.abs())).max(1e-6);
    let g = 0.8 / peak * p.gain * match var { Var::Normal | Var::Festive => 1., Var::Rain => 0.85, Var::Indoor => 0.7 };
    for v in &mut buf { *v *= g } buf
}

pub fn wav16(s: &[f32]) -> Vec<u8> {
    let (sr, n) = (SR as u32, s.len() as u32); let mut b = Vec::with_capacity(44 + s.len() * 2);
    b.extend(b"RIFF"); b.extend((36 + n * 2).to_le_bytes()); b.extend(b"WAVEfmt "); b.extend(16u32.to_le_bytes()); b.extend(1u16.to_le_bytes()); b.extend(1u16.to_le_bytes());
    b.extend(sr.to_le_bytes()); b.extend((sr * 2).to_le_bytes()); b.extend(2u16.to_le_bytes()); b.extend(16u16.to_le_bytes()); b.extend(b"data"); b.extend((n * 2).to_le_bytes());
    for v in s { b.extend(((v.clamp(-1., 1.) * 32767.) as i16).to_le_bytes()) } b
}

// ---------------- playback: generation thread + crossfades ----------------
pub struct Music { cur: Option<((u32, Var), Sound, f32)>, old: Vec<(Sound, f32)>, pending: Option<((u32, Var), Receiver<Vec<u8>>)>, pub applied: f32, pub log: Vec<String> }
impl Music {
    pub fn new() -> Music { Music { cur: None, old: vec![], pending: None, applied: 0., log: vec![] } }
    pub fn current(&self) -> Option<(u32, Var, f32)> { self.cur.as_ref().map(|c| (c.0 .0, c.0 .1, c.2)) }
    pub fn fading_out(&self) -> usize { self.old.len() }
    /// Keep exactly one target track (hour, arrangement); new tracks fade in over FADE s while old ones fade out.
    pub async fn tick(&mut self, want: (u32, Var), dt: f32, vol: f32, muted: bool) {
        let have = self.cur.as_ref().map(|c| c.0);
        if have != Some(want) && self.pending.as_ref().map(|p| p.0) != Some(want) {
            let (tx, rx) = channel(); std::thread::spawn(move || { let _ = tx.send(wav16(&compose(want.0, want.1))); }); self.pending = Some((want, rx));
        }
        if let Some((k, rx)) = self.pending.take() {
            match rx.try_recv() {
                Ok(bytes) => if let Ok(snd) = load_sound_from_bytes(&bytes).await {
                    play_sound(&snd, PlaySoundParams { looped: true, volume: 0. });
                    if let Some(c) = self.cur.take() { self.old.push((c.1, c.2)) }
                    self.log.push(format!("music -> hour {} {:?} ({:.1}s loop)", k.0, k.1, (bytes.len() as f32 - 44.) / 2. / SR)); self.cur = Some((k, snd, 0.));
                },
                Err(TryRecvError::Empty) => self.pending = Some((k, rx)),
                Err(_) => {}
            }
        }
        let rate = dt / FADE;
        if let Some(c) = &mut self.cur { c.2 = (c.2 + rate).min(1.); self.applied = effective(c.2, vol, muted); set_sound_volume(&c.1, self.applied) }
        for o in &mut self.old { o.1 -= rate; set_sound_volume(&o.0, effective(o.1, vol, muted)) }
        for o in self.old.iter().filter(|o| o.1 <= 0.) { stop_sound(&o.0) } self.old.retain(|o| o.1 > 0.);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rms(v: &[f32]) -> f32 { (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt() }
    fn hf(v: &[f32]) -> f32 { v.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum::<f32>() / v.len() as f32 }
    #[test] fn hourly_melodies_are_distinct() {
        let ms: Vec<Vec<(i32, i32)>> = (0..24).map(|h| plan(h).melody.iter().map(|n| ((n.0 * 2.) as i32, n.2)).collect()).collect();
        for a in 0..24 { for b in a + 1..24 { let same = ms[a].iter().filter(|n| ms[b].contains(n)).count() as f32 / ms[a].len().max(1) as f32; assert!(same < 0.6, "hours {a} and {b} too similar ({same:.2})") } }
        let keys: std::collections::HashSet<(i32, i32)> = (0..24).map(|h| { let p = plan(h); ((p.bpm * 10.) as i32, (p.root * 10.) as i32) }).collect(); assert!(keys.len() >= 12);
    }
    #[test] fn loops_are_long_clean_and_unclipped() {
        for h in [2u32, 7, 10, 14, 18, 23] { for v in [Var::Normal, Var::Rain, Var::Indoor, Var::Festive] { let s = compose(h, v);
            assert!(s.len() as f32 / SR >= 30., "hour {h} loop too short"); assert!(s.iter().all(|x| x.abs() <= 0.81));
            let max_step = s.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0f32, f32::max); assert!(max_step < 0.35, "hour {h} {v:?} click-like step {max_step}");
            assert!((s[0] - s[s.len() - 1]).abs() < 0.2, "loop seam jump at hour {h} {v:?}") } }
    }
    #[test] fn nights_quieter_and_indoor_muffled() {
        assert!(rms(&compose(2, Var::Normal)) < rms(&compose(14, Var::Normal)) * 0.75);
        let (n, i) = (compose(10, Var::Normal), compose(10, Var::Indoor)); assert!(hf(&i) < hf(&n) * 0.5); assert!(rms(&i) < rms(&n));
    }
    #[test] fn settings_roundtrip_and_volume_math() {
        let p = std::env::temp_dir().join("hb_settings_test.json"); let p = p.to_str().unwrap();
        save_settings(p, &Settings { music: 0.3, sfx: 1.5, muted: true }); let s = load_settings(p); assert_eq!((s.music, s.sfx, s.muted), (0.3, 1.0, true));
        assert_eq!(effective(0.5, 0.8, false), 0.4); assert_eq!(effective(1., 1., true), 0.); assert_eq!(load_settings("does-not-exist.json"), Settings::default());
    }
}
