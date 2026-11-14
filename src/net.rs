//! LAN multiplayer: one host owns the town (and the save); one friend joins over the local network and plays as
//! Player 2. The client sends its input every frame; the host simulates and streams a state snapshot back.
//! Hosts announce themselves with a UDP broadcast so the friend can pick them from a list.
use crate::world::*;
use macroquad::prelude::KeyCode;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

pub const PORT: u16 = 47778;
pub const DISCO: u16 = 47777;
pub const PROTO: u32 = 1;
/// Every key the game reads; input travels as indices into this list.
pub const KEYS: [KeyCode; 25] = [KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right, KeyCode::LeftShift, KeyCode::RightShift,
    KeyCode::E, KeyCode::Enter, KeyCode::KpEnter, KeyCode::Escape, KeyCode::F, KeyCode::G, KeyCode::I, KeyCode::J, KeyCode::N, KeyCode::Q, KeyCode::R, KeyCode::Space, KeyCode::Tab, KeyCode::X, KeyCode::Y];

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct NetInp { pub down: Vec<u8>, pub pressed: Vec<u8>, pub pad: bool }
pub fn enc(i: &Inp, pad: bool) -> NetInp {
    let f = |s: &std::collections::HashSet<KeyCode>| KEYS.iter().enumerate().filter(|(_, k)| s.contains(k)).map(|(n, _)| n as u8).collect();
    NetInp { down: f(&i.down), pressed: f(&i.pressed), pad }
}
pub fn dec(n: &NetInp) -> Inp {
    let f = |v: &Vec<u8>| v.iter().filter_map(|&k| KEYS.get(k as usize).copied()).collect();
    Inp { down: f(&n.down), pressed: f(&n.pressed) }
}

/// Everything the client needs to draw its view.
#[derive(Serialize, Deserialize, Clone)]
pub struct Snap {
    pub w: World, pub players: Vec<Player>, pub active: usize, pub npcs: Vec<NpcRt>, pub shadows: Vec<Shadow>, pub bugs: Vec<(u16, macroquad::prelude::Vec2, f32)>,
    pub bubbles: Vec<(macroquad::prelude::Vec2, u16)>, pub ibubbles: Vec<(macroquad::prelude::Vec2, u16)>, pub ibugs: Vec<(u16, macroquad::prelude::Vec2, f32)>,
    pub act: Option<Activity>, pub chats: Vec<(usize, usize, f32)>, pub visit: Option<(usize, usize, f32)>, pub hearts: Vec<(macroquad::prelude::Vec3, f32)>,
    pub fx: Vec<(macroquad::prelude::Vec3, f32, u8)>, pub shake: Vec<(usize, f32)>, pub t: f32, pub hr: u32, pub mo: u32, pub dom: u32, pub yr: i32,
    pub now: String, pub snd: Vec<Snd>,
}
pub fn snap(g: &Game, now: chrono::NaiveDateTime, snd: Vec<Snd>) -> Snap {
    let mut players: Vec<Player> = std::iter::once(g.me.clone()).chain(g.others.iter().cloned()).collect(); players.sort_by_key(|p| p.id);
    Snap { w: g.w.clone(), players, active: g.me.id, npcs: g.npcs.clone(), shadows: g.shadows.clone(), bugs: g.bugs.clone(), bubbles: g.bubbles.clone(), ibubbles: g.ibubbles.clone(), ibugs: g.ibugs.clone(),
        act: g.act.clone(), chats: g.chats.clone(), visit: g.visit, hearts: g.hearts.clone(), fx: g.fx.clone(), shake: g.shake.clone(), t: g.t, hr: g.hr, mo: g.mo, dom: g.dom, yr: g.yr,
        now: now.format("%Y-%m-%d %H:%M:%S%.f").to_string(), snd }
}
/// Load a snapshot into the client's game and make `my` the active (viewed) player; the host's player is hidden.
pub fn apply(g: &mut Game, s: Snap, my: usize) -> (chrono::NaiveDateTime, Vec<Snd>) {
    let mut players = s.players; let k = players.iter().position(|p| p.id == s.active).unwrap_or(0); let me = players.remove(k);
    g.w = s.w; g.me = me; g.others = players; g.npcs = s.npcs; g.shadows = s.shadows; g.bugs = s.bugs; g.bubbles = s.bubbles; g.ibubbles = s.ibubbles; g.ibugs = s.ibugs; g.act = s.act;
    g.chats = s.chats; g.visit = s.visit; g.hearts = s.hearts; g.fx = s.fx; g.shake = s.shake; g.t = s.t; g.hr = s.hr; g.mo = s.mo; g.dom = s.dom; g.yr = s.yr;
    if g.me.id != my && g.others.iter().any(|p| p.id == my) { g.activate(my) }
    g.remote = g.others.first().map(|p| p.id);
    let now = chrono::NaiveDateTime::parse_from_str(&s.now, "%Y-%m-%d %H:%M:%S%.f").unwrap_or_default(); (now, s.snd)
}

#[derive(Serialize, Deserialize)]
pub enum ToHost { Hello { proto: u32, version: String }, Input(NetInp), Bye }
#[derive(Serialize, Deserialize)]
pub enum ToClient { Welcome { id: usize }, Refused(String), Snap(Box<Snap>) }

fn send_line<T: Serialize>(s: &mut TcpStream, m: &T) -> bool { let mut b = serde_json::to_vec(m).unwrap_or_default(); b.push(b'\n'); s.write_all(&b).is_ok() }
/// Read newline-delimited JSON messages on a background thread.
fn reader<T: for<'a> Deserialize<'a> + Send + 'static>(s: TcpStream) -> mpsc::Receiver<Option<T>> {
    let (tx, rx) = mpsc::channel(); std::thread::spawn(move || { let mut r = BufReader::new(s); let mut line = String::new();
        loop { line.clear(); match r.read_line(&mut line) { Ok(0) | Err(_) => { let _ = tx.send(None); break } Ok(_) => if let Ok(m) = serde_json::from_str(&line) { if tx.send(Some(m)).is_err() { break } } } } }); rx
}
/// Write messages on a background thread, keeping only the newest pending snapshot so a slow link never stalls the game.
fn writer(mut s: TcpStream) -> Arc<Mutex<Vec<Vec<u8>>>> {
    let q: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(vec![])); let q2 = q.clone();
    std::thread::spawn(move || loop { let next = { let mut v = q2.lock().unwrap(); if v.is_empty() { None } else { Some(v.remove(0)) } };
        match next { Some(b) => if s.write_all(&b).is_err() { break }, None => std::thread::sleep(Duration::from_millis(4)) } }); q
}

pub enum HostEv { Joined, Left }
struct Conn { rx: mpsc::Receiver<Option<ToHost>>, out: Arc<Mutex<Vec<Vec<u8>>>>, welcomed: bool }
pub struct Host { listener: TcpListener, conn: Option<Conn>, udp: Option<UdpSocket>, beacon: Instant, sent: Instant, down: Vec<u8>, pressed: Vec<u8>, pub pad: bool, name: String }
impl Host {
    pub fn start(name: &str) -> std::io::Result<Host> { Self::start_on(name, PORT) }
    pub fn start_on(name: &str, port: u16) -> std::io::Result<Host> {
        let bind = std::env::var("HOLLOWBROOK_LAN_BIND").unwrap_or_else(|_| "0.0.0.0".into()); let listener = TcpListener::bind((bind.as_str(), port))?; listener.set_nonblocking(true)?;
        let udp = UdpSocket::bind(("0.0.0.0", 0)).ok().filter(|u| u.set_broadcast(true).is_ok());
        Ok(Host { listener, conn: None, udp, beacon: Instant::now() - Duration::from_secs(5), sent: Instant::now(), down: vec![], pressed: vec![], pad: false, name: name.chars().take(16).collect() })
    }
    pub fn port(&self) -> u16 { self.listener.local_addr().map(|a| a.port()).unwrap_or(PORT) }
    /// Best-effort LAN address of this computer for display.
    pub fn lan_ip() -> String { UdpSocket::bind(("0.0.0.0", 0)).and_then(|u| { u.connect(("192.168.0.1", 9))?; u.local_addr() }).map(|a| a.ip().to_string()).unwrap_or_else(|_| "this computer".into()) }
    pub fn connected(&self) -> bool { self.conn.as_ref().is_some_and(|c| c.welcomed) }
    /// Accept a connection, read messages, broadcast the beacon. Returns join/leave events.
    pub fn poll(&mut self) -> Vec<HostEv> {
        let mut ev = vec![];
        if self.beacon.elapsed() > Duration::from_secs(1) { self.beacon = Instant::now(); if let Some(u) = &self.udp { let msg = format!("HOLLOWBROOK {PROTO} {} {}", self.port(), self.name); let _ = u.send_to(msg.as_bytes(), ("255.255.255.255", DISCO)); } }
        if let Ok((s, _)) = self.listener.accept() { let _ = s.set_nodelay(true); let _ = s.set_nonblocking(false);
            if self.conn.is_some() { let mut s = s; send_line(&mut s, &ToClient::Refused("Someone is already playing on this host.".into())); }
            else if let Ok(r) = s.try_clone() { self.conn = Some(Conn { rx: reader(r), out: writer(s), welcomed: false }) } }
        let mut drop = false;
        if let Some(c) = &mut self.conn { while let Ok(m) = c.rx.try_recv() { match m {
            Some(ToHost::Hello { proto, .. }) => { if proto == PROTO { if !c.welcomed { c.welcomed = true; ev.push(HostEv::Joined) } } else { Self::push(&c.out, &ToClient::Refused("Different game version.".into())); drop = true } }
            Some(ToHost::Input(i)) => { self.down = i.down; for k in i.pressed { if !self.pressed.contains(&k) { self.pressed.push(k) } } self.pad = i.pad }
            Some(ToHost::Bye) | None => { drop = true } } } }
        if drop { if self.conn.take().is_some_and(|c| c.welcomed) { ev.push(HostEv::Left) } self.down.clear(); self.pressed.clear() }
        ev
    }
    fn push<T: Serialize>(q: &Arc<Mutex<Vec<Vec<u8>>>>, m: &T) { let mut b = serde_json::to_vec(m).unwrap_or_default(); b.push(b'\n'); let mut v = q.lock().unwrap(); v.retain(|x| !x.starts_with(b"{\"Snap\"")); v.push(b) }
    pub fn welcome(&mut self, id: usize) { if let Some(c) = &self.conn { Self::push(&c.out, &ToClient::Welcome { id }) } }
    pub fn refuse(&mut self, why: &str) { if let Some(c) = self.conn.take() { Self::push(&c.out, &ToClient::Refused(why.into())) } }
    /// Input accumulated from the client since the last call (pressed edges are consumed).
    pub fn take_input(&mut self) -> Inp { let n = NetInp { down: self.down.clone(), pressed: std::mem::take(&mut self.pressed), pad: self.pad }; dec(&n) }
    /// True when a snapshot should be sent (about 20 per second while a friend is connected).
    pub fn due(&self) -> bool { self.connected() && self.sent.elapsed() >= Duration::from_millis(48) }
    pub fn send(&mut self, s: Snap) { self.sent = Instant::now(); if let Some(c) = &self.conn { if c.welcomed { Self::push(&c.out, &ToClient::Snap(Box::new(s))) } } }
}

pub struct Client { rx: mpsc::Receiver<Option<ToClient>>, out: TcpStream, pub id: Option<usize>, pub refused: Option<String>, pub closed: bool, latest: Option<Snap> }
impl Client {
    pub fn connect(addr: SocketAddr) -> std::io::Result<Client> {
        let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(4))?; let _ = s.set_nodelay(true);
        send_line(&mut s, &ToHost::Hello { proto: PROTO, version: crate::data::VERSION.into() }); let rx = reader(s.try_clone()?);
        Ok(Client { rx, out: s, id: None, refused: None, closed: false, latest: None })
    }
    pub fn poll(&mut self) { while let Ok(m) = self.rx.try_recv() { match m { Some(ToClient::Welcome { id }) => self.id = Some(id), Some(ToClient::Refused(r)) => { self.refused = Some(r); self.closed = true } Some(ToClient::Snap(s)) => self.latest = Some(*s), None => self.closed = true } } }
    pub fn take_snap(&mut self) -> Option<Snap> { self.latest.take() }
    pub fn send(&mut self, i: &Inp, pad: bool) { if !send_line(&mut self.out, &ToHost::Input(enc(i, pad))) { self.closed = true } }
    pub fn bye(&mut self) { send_line(&mut self.out, &ToHost::Bye); }
}

/// Listen for host beacons on the LAN (call every frame; keeps a list of hosts seen in the last few seconds).
pub struct Finder { udp: Option<UdpSocket>, pub hosts: Vec<(String, SocketAddr, Instant)> }
impl Finder {
    pub fn new() -> Finder { let udp = UdpSocket::bind(("0.0.0.0", DISCO)).ok(); if let Some(u) = &udp { let _ = u.set_nonblocking(true); } Finder { udp, hosts: vec![] } }
    pub fn ok(&self) -> bool { self.udp.is_some() }
    pub fn poll(&mut self) {
        let mut buf = [0u8; 256];
        if let Some(u) = &self.udp { while let Ok((n, from)) = u.recv_from(&mut buf) { let msg = String::from_utf8_lossy(&buf[..n]).to_string(); let mut it = msg.splitn(4, ' ');
            if it.next() == Some("HOLLOWBROOK") && it.next() == Some(&PROTO.to_string()) { if let Some(port) = it.next().and_then(|p| p.parse::<u16>().ok()) { let name = it.next().unwrap_or("Host").to_string(); let addr = SocketAddr::new(from.ip(), port);
                if let Some(h) = self.hosts.iter_mut().find(|h| h.1 == addr) { h.2 = Instant::now(); h.0 = name } else { self.hosts.push((name, addr, Instant::now())) } } } } }
        self.hosts.retain(|h| h.2.elapsed() < Duration::from_secs(4));
    }
}
