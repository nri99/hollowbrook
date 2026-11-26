//! Game state and rules: persistent World, runtime Game, movement, interaction, economy, furniture, residents.
use crate::clock::day_id;
use crate::data::*;
use chrono::{Datelike, NaiveDateTime, Timelike};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
mod daily;
mod extras;
mod festivals;
mod frontier;
mod furniture;
mod home_garden;
mod housing;
mod input;
pub use frontier::*;
mod interact;
mod islet;
mod projects;
mod shop;
mod sim;
mod tools;

pub const INV_CAP: usize = 16;
pub const STORE_CAP: usize = 40;
pub const STACK: u16 = 10;
pub const PR: f32 = 0.28;
pub const SHOP: usize = 0;
pub const HOME: usize = 1;
pub const SAVE_VERSION: u32 = 5;
/// Home sizes per house level and the price of each expansion (provisional numbers).
pub const HOME_DIMS: [(i32, i32); 5] = [(4, 4), (5, 5), (6, 5), (6, 5), (6, 5)];
/// Levels 1-2 enlarge the main room; level 3 adds a basement, level 4 an upstairs (provisional prices).
pub const EXPAND_COST: [u32; 4] = [3000, 15000, 30000, 50000];
/// Home stories: 0 main floor, 1 basement, 2 upstairs. Stairs up sit in the back-left corner, stairs down beside them.
pub const STAIR_UP: (i32, i32) = (0, 0);
pub const STAIR_DOWN: (i32, i32) = (1, 0);
pub fn elev(story: u8) -> i32 {
    match story {
        1 => -1,
        2 => 1,
        _ => 0,
    }
}
/// Stair tiles on `story` of a home at house level `lv`: (tile, destination story).
pub fn stairs(lv: u32, story: u8) -> Vec<((i32, i32), u8)> {
    match story {
        0 => {
            let mut v = vec![];
            if lv >= 4 {
                v.push((STAIR_UP, 2))
            }
            if lv >= 3 {
                v.push((STAIR_DOWN, 1))
            }
            v
        }
        1 => vec![(STAIR_DOWN, 0)],
        _ => vec![(STAIR_UP, 0)],
    }
}

// ---- persistent state (definition id `def` is separate from instance id `uid` and from location) ----
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Slot {
    pub def: u16,
    pub qty: u16,
    pub uid: u32,
    #[serde(default)]
    pub size_mm: u16,
}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Furn {
    pub uid: u32,
    pub def: u16,
    pub x: i32,
    pub z: i32,
    pub rot: u8,
    pub on: bool,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct NpcS {
    pub friend: i32,
    pub req: Option<u16>,
    pub talked: bool,
    #[serde(default)]
    pub gifted: bool,
    #[serde(default)]
    pub visited: bool,
}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Tree {
    pub x: i32,
    pub z: i32,
    pub fruit: bool,
    pub shaken: i64,
}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct GItem {
    pub uid: u32,
    pub def: u16,
    pub x: f32,
    pub z: f32,
}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Stock {
    pub def: u16,
    pub sold: bool,
}
/// Active community project: needs are (material code, required, contributed); `log` = (player id, items given).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Project {
    pub kind: u8,
    pub x: i32,
    pub z: i32,
    pub need: Vec<(u8, u16, u16)>,
    pub by: usize,
    pub log: Vec<(usize, u16)>,
}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Built {
    pub kind: u8,
    pub x: i32,
    pub z: i32,
    pub day: i64,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MProject {
    pub idx: usize,
    pub x: i32,
    pub z: i32,
    pub funded: u32,
    pub log: Vec<(usize, u32)>,
}
/// Initiative events counted from normal play.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Ev {
    Bug,
    Fish,
    Weed,
    Fruit,
    Sell,
    Talk,
    Dig,
    Shell,
    Gift,
    Buy,
    Activity,
    Rock,
    Dive,
    Camp,
    Water,
}
/// Initiative templates (original balance, provisional): event, target, text, coupons.
pub const TASKS: [(Ev, u32, &str, u32); 15] = [
    (Ev::Bug, 2, "Catch 2 bugs with the net", 3),
    (Ev::Fish, 1, "Catch a fish", 3),
    (Ev::Weed, 2, "Pull 2 weeds", 2),
    (Ev::Fruit, 1, "Shake fruit from a tree", 2),
    (Ev::Sell, 500, "Sell items worth 500 coins", 3),
    (Ev::Talk, 3, "Chat with 3 residents", 2),
    (Ev::Dig, 1, "Dig up something buried", 3),
    (Ev::Shell, 2, "Pick up 2 shells", 2),
    (Ev::Gift, 1, "Give a resident a gift", 3),
    (Ev::Buy, 1, "Buy something at the General Store", 2),
    (Ev::Rock, 1, "Get a stone from a rock (shovel)", 2),
    (Ev::Dive, 1, "Dive up a sea creature", 3),
    (Ev::Camp, 1, "Chat with today's camper", 2),
    (Ev::Activity, 1, "Finish a Gullrock timed activity", 4),
    (Ev::Water, 3, "Water 3 flowers", 2),
];
/// Weekly town task templates: (event, target, text). Shared progress; each player claims WEEKLY_COUPONS once.
pub const WTASKS: [(Ev, u32, &str); 5] = [
    (Ev::Bug, 15, "Town goal: catch 15 bugs"),
    (Ev::Sell, 6000, "Town goal: sell 6,000 coins of goods"),
    (Ev::Talk, 20, "Town goal: 20 chats with residents"),
    (Ev::Shell, 12, "Town goal: pick up 12 shells"),
    (Ev::Dive, 6, "Town goal: dive up 6 sea creatures"),
];
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PTask {
    pub t: usize,
    pub prog: u32,
    pub claimed: bool,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WTask {
    pub t: usize,
    pub prog: u32,
    pub week: i64,
    pub claimed: Vec<usize>,
}
pub fn week_of(day: i64) -> i64 {
    (day - 1).div_euclid(7)
}
/// Today's shared surprise: kind (index into SURPRISES), tile, resident slot that lost the scarf.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Surprise {
    pub kind: u8,
    pub x: i32,
    pub z: i32,
    pub owner: usize,
    pub day: i64,
}
/// A planted flower in the shared town garden: kind 0-3 colour, 4 Starfleck; stage 0 sprout, 1 bud, 2 bloom.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Plant {
    pub x: i32,
    pub z: i32,
    pub kind: u8,
    pub stage: u8,
    pub watered: i64,
}
/// A running Gullrock activity (runtime only, not saved): shared by every participant.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Activity {
    pub kind: usize,
    pub left: f32,
    pub score: u32,
    pub mine: Vec<(usize, u32)>,
    pub players: Vec<usize>,
    pub ents: Vec<Vec2>,
    pub next: usize,
    pub life: f32,
    pub done: bool,
    pub paid: Vec<usize>,
}
pub fn ptiles(kind: u8, x: i32, z: i32) -> Vec<(i32, i32)> {
    if kind == PK_BRIDGE {
        vec![(x, z), (x + 1, z)]
    } else {
        vec![(x, z)]
    }
}
pub fn mat_ok(code: u8, d: u16) -> bool {
    match code {
        0 => d == 80,
        1 => d == 81,
        2 => def(d).k == K::Fruit,
        _ => def(d).k == K::Shell,
    }
}
/// A stored letter. Gifts (coins and/or an item) can be claimed exactly once.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Letter {
    pub from: String,
    pub text: String,
    pub coins: u32,
    pub item: Option<u16>,
    pub read: bool,
    pub claimed: bool,
}
/// Personal progress of one local player (save v4). Everything else in `World` is shared by both players.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Profile {
    #[serde(default)]
    pub contests: Vec<crate::events::ContestEntry>,
    #[serde(default)]
    pub invitation: Option<crate::events::SocialInvite>,
    #[serde(default)]
    pub social_day: i64,
    #[serde(default)]
    pub missed_visit: Option<(usize, usize, i64)>,
    #[serde(default)]
    pub seek: Option<crate::events::Seek>,
    #[serde(default)]
    pub id: usize,
    pub name: String,
    pub money: u32,
    pub home: u32,
    pub inv: Vec<Slot>,
    pub storage: Vec<Slot>,
    pub room: Vec<Furn>,
    #[serde(default)]
    pub story: u8,
    #[serde(default)]
    pub stories: Vec<Vec<Furn>>,
    /// Dustreach: active bounty, bounties turned in (rank), and (day, offer) already completed.
    #[serde(default)]
    pub bounty: Option<Bounty>,
    #[serde(default)]
    pub bounties_done: u32,
    #[serde(default)]
    pub bounty_log: Vec<(i64, u8)>,
    pub npcs: Vec<NpcS>,
    pub seen: Vec<u16>,
    pub tools: Vec<u16>,
    pub equip: Option<u16>,
    pub pos: [f32; 2],
    /// worn shirt (item def); None = starting outfit
    #[serde(default)]
    pub shirt: Option<u16>,
    #[serde(default)]
    pub hat: Option<u16>,
    /// hairstyle index into HAIRS (None-default 0 keeps the player's starting look)
    #[serde(default)]
    pub hair: u8,
    #[serde(default)]
    pub mail: Vec<Letter>,
    /// furniture/clothing ever owned, re-orderable at the shop counter
    #[serde(default)]
    pub catalog: Vec<u16>,
    /// year in which this player last claimed the Harvest Fair prize
    #[serde(default)]
    pub fair_year: i32,
    /// week index of the last art-stall purchase
    #[serde(default)]
    pub art_week: i64,
    /// initiatives: coupon balance, today's tasks (assigned for task_day), campground favors done (visitor index)
    #[serde(default)]
    pub keepsakes: Vec<u8>,
    #[serde(default)]
    pub fest_years: Vec<(u8, i32)>,
    #[serde(default)]
    pub lost_owner: Option<usize>,
    #[serde(default)]
    pub tool_lv: Vec<u8>,
    #[serde(default)]
    pub home_best: u8,
    #[serde(default)]
    pub paint: Option<u8>,
    #[serde(default)]
    pub coupons: u32,
    #[serde(default)]
    pub tasks: Vec<PTask>,
    #[serde(default)]
    pub task_day: i64,
    #[serde(default)]
    pub camp_favors: Vec<u8>,
    #[serde(default)]
    pub ferry: u32,
    /// interest-free home expansion balance (no deadline)
    #[serde(default)]
    pub debt: u32,
    /// furniture tint per item uid; home wallpaper/flooring palette index
    /// last gameplay day this player completed the islet activity
    #[serde(default)]
    pub islet_done: i64,
    /// day each Gullrock activity last paid this player
    #[serde(default)]
    pub act_day: [i64; 3],
    /// active parcel delivery: (from slot, to slot)
    #[serde(default)]
    pub parcel: Option<(usize, usize)>,
    #[serde(default)]
    pub tints: Vec<(u32, u8)>,
    #[serde(default)]
    pub wall: u8,
    #[serde(default)]
    pub floor: u8,
}
impl Profile {
    pub fn new(id: usize, name: String, pos: Vec2) -> Profile {
        Profile {
            seek: None,
            invitation: None,
            social_day: 0,
            missed_visit: None,
            contests: vec![],
            story: 0,
            bounty: None,
            bounties_done: 0,
            bounty_log: vec![],
            stories: vec![],
            id,
            name,
            money: 1000,
            home: 0,
            inv: vec![],
            storage: vec![],
            room: vec![],
            npcs: (0..4).map(|_| NpcS { friend: 0, req: None, talked: false, gifted: false, visited: false }).collect(),
            seen: vec![],
            tools: vec![60, 61],
            equip: None,
            pos: [pos.x, pos.y],
            shirt: None,
            hat: None,
            hair: 0,
            mail: vec![],
            catalog: vec![],
            fair_year: 0,
            art_week: 0,
            keepsakes: vec![],
            fest_years: vec![],
            lost_owner: None,
            tool_lv: vec![],
            home_best: 0,
            paint: None,
            coupons: 0,
            tasks: vec![],
            task_day: 0,
            camp_favors: vec![],
            ferry: 0,
            debt: 0,
            islet_done: 0,
            act_day: [0; 3],
            parcel: None,
            tints: vec![],
            wall: 0,
            floor: 0,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct World {
    pub version: u32,
    pub day: i64,
    pub native: u16,
    pub trees: Vec<Tree>,
    pub ground: Vec<GItem>,
    pub weeds: Vec<(i32, i32)>,
    pub holes: Vec<(i32, i32)>,
    pub digs: Vec<(i32, i32)>,
    pub stock: Vec<Stock>,
    pub stock_day: i64,
    pub uid: u32,
    #[serde(default)]
    pub trampled: Vec<(i32, i32)>,
    /// museum donations shared by all players: (item def, donating player id)
    #[serde(default)]
    pub donated: Vec<(u16, usize)>,
    /// house slot -> character index in CHARS (residents can move)
    #[serde(default = "roster0")]
    pub roster: Vec<usize>,
    /// per house slot: gameplay day the current resident arrived, and the day they leave (0 = not moving)
    #[serde(default)]
    pub arrived: Vec<i64>,
    #[serde(default)]
    pub moving: Vec<i64>,
    #[serde(default)]
    pub project: Option<Project>,
    #[serde(default)]
    pub mproject: Option<MProject>,
    /// active town policy (index into POLICIES); effects are derived, never stored
    #[serde(default)]
    pub policy: Option<usize>,
    #[serde(default)]
    pub wtask: Option<WTask>,
    #[serde(default)]
    pub surprise: Option<Surprise>,
    #[serde(default)]
    pub garden: Vec<Plant>,
    #[serde(default)]
    pub spent: u32,
    #[serde(default)]
    pub shop_lv: u8,
    #[serde(default)]
    pub special_week: i64,
    #[serde(default)]
    pub camp_day: i64,
    #[serde(default)]
    pub camp_sold: Vec<u16>,
    #[serde(default)]
    pub built: Vec<Built>,
    /// trees already yielding a branch / rocks already yielding a stone today
    #[serde(default)]
    pub branches: Vec<(i32, i32)>,
    #[serde(default)]
    pub rocks_hit: Vec<(i32, i32)>,
    /// islet collectibles (islet coordinates), refreshed daily
    #[serde(default)]
    pub iground: Vec<GItem>,
    /// profile of the player currently being processed (swapped in by `Game::activate`)
    pub p: Profile,
    #[serde(default)]
    pub others: Vec<Profile>,
}
pub fn roster0() -> Vec<usize> {
    vec![0, 1, 2, 3]
}
impl World {
    pub fn fresh(name: String, day: i64, map: &[u8]) -> World {
        let trees = (0..W * H)
            .filter(|&i| map[i] == TREE)
            .map(|i| Tree { x: (i % W) as i32, z: (i / W) as i32, fruit: true, shaken: i64::MIN / 2 })
            .collect();
        World {
            version: SAVE_VERSION,
            day,
            native: FRUITS[gen_range(0usize, 4)],
            trees,
            ground: vec![],
            weeds: vec![],
            holes: vec![],
            digs: vec![],
            stock: vec![],
            stock_day: i64::MIN,
            uid: 1,
            trampled: vec![],
            donated: vec![],
            roster: roster0(),
            arrived: vec![day; 4],
            moving: vec![0; 4],
            project: None,
            mproject: None,
            policy: None,
            wtask: None,
            surprise: None,
            garden: vec![],
            spent: 0,
            shop_lv: 0,
            special_week: -1,
            camp_day: 0,
            camp_sold: vec![],
            built: vec![],
            branches: vec![],
            rocks_hit: vec![],
            iground: vec![],
            p: Profile::new(0, name, door_stand(HOME)),
            others: vec![],
        }
    }
}

// ---- runtime ----
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Scene {
    Town,
    Room(usize),
    Islet,
    /// Dustreach, the frontier outpost (separate world; dev entry only for now).
    Frontier,
}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Pose {
    Stand,
    Sit(u32),
    Lie(u32),
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Loc {
    Out,
    Home,
    Asleep,
}
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Snd {
    Step,
    Blip,
    Splash,
    Coin,
    Dig,
    Rustle,
    Door,
    StepPath,
    StepSand,
    StepWood,
    StepFloor,
    Swim,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tg {
    Contest,
    Npc(usize),
    Door(usize),
    Tree(usize),
    Item(usize),
    Weed(usize),
    Furn(u32, u16),
    Counter,
    Keeper,
    Display(usize),
    Curator,
    Exhibit(usize),
    Rack,
    Mailbox(usize),
    Stall,
    Site,
    Board,
    Built(usize),
    Dock,
    Pell,
    IItem(usize),
    IStall,
    MSite,
    ActBoard,
    Hazel,
    ArtStall,
    Camper,
    CampSign,
    Kiosk,
    Fest,
    Surprise,
    Plant(usize),
    PlantSpot,
    Stairs(u8),
    BountyBoard,
    WoodsDoor,
    Fugitive(usize),
    FGate,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ui {
    Invitation,
    Contest(usize),
    Play,
    Talk { npc: Option<usize>, who: String, lines: Vec<String>, i: usize },
    Inv(usize),
    Sell(usize),
    Confirm(usize),
    Storage(usize, usize),
    Journal,
    Rack(usize),
    Mail(usize),
    Catalog(usize),
    Gift(usize, usize),
    Tint(usize),
    Travel(bool),
    IStall(usize),
    ConfirmExpand,
    MPick(usize),
    Fund,
    Policy(usize),
    PolicyOk(usize),
    Act(usize),
    Civic(usize),
    Paint(usize),
    Keeper(usize),
    Camp(usize),
    Kiosk(usize),
    Bounty(usize),
    Title(usize),
    Help,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Placing {
    pub f: Furn,
    pub orig: Option<Furn>,
    pub slot_uid: Option<u32>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fishing {
    pub bob: Vec2,
    pub fish: Option<usize>,
    pub t: f32,
    pub nib: u8,
    pub need: u8,
    pub bite: bool,
    pub dip: f32,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shadow {
    pub pos: Vec2,
    pub def: u16,
    pub vel: Vec2,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NpcRt {
    pub pos: Vec2,
    pub path: Vec<(i32, i32)>,
    pub wait: f32,
    pub yaw: f32,
    pub inside: bool,
    pub happy: f32,
    pub moving: bool,
    pub last_chat: usize,
    pub anim: f32,
}
/// Keyboard state for one frame; filled from the OS (macroquad) or injected by the dev autotest bot.
pub struct Inp {
    pub down: HashSet<KeyCode>,
    pub pressed: HashSet<KeyCode>,
}
impl Inp {
    pub fn none() -> Inp {
        Inp { down: HashSet::new(), pressed: HashSet::new() }
    }
    pub fn keys(down: &[KeyCode], pressed: &[KeyCode]) -> Inp {
        Inp { down: down.iter().copied().collect(), pressed: pressed.iter().copied().collect() }
    }
    pub fn d(&self, k: KeyCode) -> bool {
        self.down.contains(&k)
    }
    pub fn p(&self, k: KeyCode) -> bool {
        self.pressed.contains(&k)
    }
}
pub fn where_(r: &Res, hr: u32) -> Loc {
    if !(hr >= r.wake && hr < r.sleep) {
        Loc::Asleep
    } else if r.home_h.iter().any(|&(a, b)| hr >= a && hr < b) {
        Loc::Home
    } else {
        Loc::Out
    }
}
/// English plural for item names (peach -> peaches, cherry -> cherries).
pub fn plural(w: &str) -> String {
    if w.ends_with("ch") {
        format!("{w}es")
    } else if let Some(b) = w.strip_suffix('y') {
        format!("{b}ies")
    } else {
        format!("{w}s")
    }
}
pub fn tile_of(p: Vec2) -> (i32, i32) {
    (p.x.floor() as i32, p.y.floor() as i32)
}
pub fn tc(t: (i32, i32)) -> Vec2 {
    vec2(t.0 as f32 + 0.5, t.1 as f32 + 0.5)
}
pub fn dir(rot: u8) -> Vec2 {
    [vec2(0., 1.), vec2(1., 0.), vec2(0., -1.), vec2(-1., 0.)][rot as usize % 4]
}
pub fn yaw_of(v: Vec2) -> f32 {
    v.x.atan2(v.y)
}
pub fn tile_at_i(map: &[u8], x: i32, z: i32) -> u8 {
    if x < 0 || z < 0 || x >= IW as i32 || z >= IH as i32 {
        SEA
    } else {
        map[z as usize * IW + x as usize]
    }
}
/// A winding path from the west edge of town into the woods, ending at the door to Dustreach.
/// These tiles lie outside the saved map, so existing towns and saves are unaffected.
pub const WOODS_PATH: [(i32, i32); 10] =
    [(0, 17), (-1, 17), (-2, 17), (-2, 16), (-3, 16), (-4, 16), (-5, 16), (-5, 17), (-6, 17), (-7, 17)];
pub const WOODS_DOOR: (i32, i32) = (-8, 17);
pub fn tile_at(map: &[u8], x: i32, z: i32) -> u8 {
    if WOODS_PATH.contains(&(x, z)) {
        return PATH;
    }
    if x < 0 || z < 0 || x >= W as i32 || z >= H as i32 {
        PINE
    } else {
        map[z as usize * W + x as usize]
    }
}
pub fn fcenter(f: &Furn) -> Vec2 {
    if def(f.def).wall {
        return vec2(f.x as f32 + 0.5, 0.15);
    }
    let t = foot(f.def, f.x, f.z, f.rot);
    let (a, b) = (t[0], t[t.len() - 1]);
    vec2((a.0 + b.0 + 1) as f32 / 2., (a.1 + b.1 + 1) as f32 / 2.)
}

/// Runtime state of one local player (persisted only through its Profile).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: usize,
    pub scene: Scene,
    pub pos: Vec2,
    pub ipos: Vec2,
    pub face: Vec2,
    pub cam: Vec2,
    pub anim: f32,
    pub spd: f32,
    pub pose: Pose,
    pub stand_from: Vec2,
    pub step_acc: f32,
    pub ui: Ui,
    pub placing: Option<Placing>,
    pub fish: Option<Fishing>,
    pub trans: Option<(f32, Scene, bool)>,
    pub swing: f32,
    pub msgs: Vec<(String, f32)>,
    /// last input came from a gamepad (button prompts) / its assigned pad is disconnected
    pub pad: bool,
    pub pad_lost: bool,
    /// leader choosing a site for money project MPROJECTS[i]
    pub mplace: Option<usize>,
    /// journal page (0 collection, 1 initiatives, 2 settings, 3 controls, 4 guide) and collection column in split view
    pub jpage: u8,
    pub jcol: u8,
    /// Dustreach gear shots in flight (never saved).
    #[serde(default)]
    pub bolts: Vec<Bolt>,
}
impl Player {
    pub fn new(id: usize, pos: Vec2) -> Player {
        Player {
            id,
            scene: Scene::Town,
            pos,
            ipos: Vec2::ZERO,
            face: vec2(0., 1.),
            cam: pos,
            anim: 0.,
            spd: 0.,
            pose: Pose::Stand,
            stand_from: Vec2::ZERO,
            step_acc: 0.,
            ui: Ui::Play,
            placing: None,
            fish: None,
            trans: None,
            swing: 0.,
            msgs: vec![],
            pad: false,
            pad_lost: false,
            mplace: None,
            jpage: 0,
            jcol: 0,
            bolts: vec![],
        }
    }
}
pub struct Game {
    pub clock_ms: i64,
    pub w: World,
    pub map: Vec<u8>,
    pub me: Player,
    pub others: Vec<Player>,
    pub npcs: Vec<NpcRt>,
    pub t: f32,
    pub log: Vec<String>,
    pub rolled: bool,
    pub shadows: Vec<Shadow>,
    pub bugs: Vec<(u16, Vec2, f32)>,
    pub shake: Vec<(usize, f32)>,
    pub hearts: Vec<(Vec3, f32)>,
    pub snd: Vec<Snd>,
    pub hr: u32,
    pub mo: u32,
    pub curve: f32,
    pub dom: u32,
    pub yr: i32,
    /// transient tool effects: (position, age, kind 0 splash / 1 dirt / 2 swish)
    pub fx: Vec<(Vec3, f32, u8)>,
    /// shared bubble spots in the sea marking a sea creature below
    pub bubbles: Vec<(Vec2, u16)>,
    pub imap: Vec<u8>,
    pub fmap: Vec<u8>,
    pub ibubbles: Vec<(Vec2, u16)>,
    pub ibugs: Vec<(u16, Vec2, f32)>,
    pub pell_talked: bool,
    pub act: Option<Activity>,
    pub quit: bool,
    pub fresh: bool,
    /// LAN: player id driven over the network (hidden from this screen); title-menu request 1 host / 2 join
    pub remote: Option<usize>,
    pub net_req: u8,
    /// residents chatting with each other: (slot a, slot b, seconds left)
    pub chats: Vec<(usize, usize, f32)>,
    /// audio settings (stored in hollowbrook_settings.json, not the world save)
    pub music_vol: f32,
    pub sfx_vol: f32,
    pub muted: bool,
    pub settings_dirty: bool,
    /// resident currently visiting a player's home: (resident, home building, seconds left)
    pub visit: Option<(usize, usize, f32)>,
}

impl Game {
    pub fn new(name: String, now: NaiveDateTime) -> Game {
        let map = town();
        let w = World::fresh(name, day_id(now), &map);
        let mut g = Game::from_world(w, map);
        g.clock_ms = now.and_utc().timestamp_millis();
        g.refresh(false);
        g.me.face = vec2(0., -1.);
        g.hr = now.hour();
        g.mo = now.month();
        g.note("Welcome to Hollowbrook! Walk up to doors, residents and trees and press E.");
        g
    }
    pub fn from_world(w: World, map: Vec<u8>) -> Game {
        let pos = vec2(w.p.pos[0], w.p.pos[1]);
        let npcs = (0..4)
            .map(|i| NpcRt {
                pos: door_stand(HOUSES[i]),
                path: vec![],
                wait: gen_range(0., 2.),
                yaw: 0.,
                inside: true,
                happy: 0.,
                moving: false,
                last_chat: 9,
                anim: 0.,
            })
            .collect();
        let mut g = Game {
            clock_ms: 0,
            me: Player::new(w.p.id, pos),
            w,
            map,
            others: vec![],
            npcs,
            t: 0.,
            log: vec![],
            rolled: false,
            shadows: vec![],
            bugs: vec![],
            shake: vec![],
            hearts: vec![],
            snd: vec![],
            hr: 12,
            mo: 6,
            curve: 0.01,
            dom: 1,
            yr: 2026,
            bubbles: vec![],
            visit: None,
            imap: islet(),
            fmap: frontier_map(),
            ibubbles: vec![],
            pell_talked: false,
            ibugs: vec![],
            act: None,
            quit: false,
            fresh: false,
            remote: None,
            net_req: 0,
            chats: vec![],
            music_vol: 0.6,
            sfx_vol: 0.8,
            muted: false,
            settings_dirty: false,
            fx: vec![],
        };
        if !g.free(g.me.pos) {
            g.me.pos = door_stand(HOME);
            g.me.cam = g.me.pos
        }
        let map = &g.map;
        let open = |t: &(i32, i32)| tile_at(map, t.0, t.1) != BLD && *t != mailbox_tile(0) && *t != mailbox_tile(1); // objects saved before a building existed there
        g.w.weeds.retain(open);
        g.w.digs.retain(open);
        g.w.holes.retain(open);
        g.w.ground.retain(|i| open(&tile_of(vec2(i.x, i.z))));
        for p in std::iter::once(&mut g.w.p).chain(g.w.others.iter_mut()) {
            let owned: Vec<u16> = p
                .room
                .iter()
                .map(|f| f.def)
                .chain(p.inv.iter().chain(p.storage.iter()).map(|s| s.def))
                .filter(|&d| matches!(def(d).k, K::Furn | K::Cloth))
                .chain(p.shirt)
                .collect();
            for d in owned {
                if !p.catalog.contains(&d) {
                    p.catalog.push(d)
                }
            }
        }
        if g.w.iground.is_empty() {
            g.refresh_islet()
        }
        if g.w.project.is_none() {
            g.propose()
        } // first project for new and older saves
        if g.w.arrived.len() != 4 {
            g.w.arrived = vec![g.w.day; 4]
        }
        if g.w.moving.len() != 4 {
            g.w.moving = vec![0; 4]
        } // older saves start the 14-day clock now
        if g.w.stock.is_empty() {
            g.refresh_world()
        } // migrated saves get shop stock, shells and dig sites
        g
    }
    /// Ids of all players in the session (active one included), ascending.
    pub fn ids(&self) -> Vec<usize> {
        let mut v: Vec<usize> = std::iter::once(self.me.id).chain(self.others.iter().map(|p| p.id)).collect();
        v.sort();
        v
    }
    /// Make player `id` the one that `me` / `w.p` refer to (swaps runtime state and profile; nothing is copied).
    pub fn activate(&mut self, id: usize) {
        if self.me.id != id {
            if let Some(k) = self.others.iter().position(|p| p.id == id) {
                std::mem::swap(&mut self.me, &mut self.others[k])
            }
        }
        if self.w.p.id != id {
            if let Some(k) = self.w.others.iter().position(|p| p.id == id) {
                std::mem::swap(&mut self.w.p, &mut self.w.others[k])
            }
        }
    }
    /// Add the second local player: a fresh profile, or the saved one if it already exists. Returns its id.
    pub fn join(&mut self, name: &str) -> usize {
        let id = 1;
        if self.ids().contains(&id) {
            return id;
        }
        let start = door_stand(home_of(id));
        if self.w.p.id != id && !self.w.others.iter().any(|p| p.id == id) {
            self.w.others.push(Profile::new(id, name.into(), start))
        }
        let pr = if self.w.p.id == id { &self.w.p } else { self.w.others.iter().find(|p| p.id == id).unwrap() };
        let mut pl = Player::new(id, vec2(pr.pos[0], pr.pos[1]));
        if !self.free_town(pl.pos) {
            pl.pos = start;
            pl.cam = start
        }
        pl.face = vec2(0., -1.);
        self.others.push(pl);
        id
    }
    /// Remove player `id` from the session; its profile and everything it owns stays in the save.
    pub fn leave(&mut self, id: usize) {
        if id == 0 || !self.ids().contains(&id) {
            return;
        }
        self.activate(id);
        self.place_cancel();
        self.me.ui = Ui::Play;
        self.me.fish = None;
        let pos = match self.me.scene {
            Scene::Town => self.me.pos,
            Scene::Room(b) => door_stand(b),
            Scene::Islet => dock_stand(),
            Scene::Frontier => self.me.pos,
        };
        self.w.p.pos = [pos.x, pos.y];
        self.activate(0);
        self.others.retain(|p| p.id != id);
    }
    fn free_town(&self, p: Vec2) -> bool {
        [(-PR, -PR), (PR, -PR), (-PR, PR), (PR, PR)].iter().all(|&(dx, dz)| {
            let (x, z) = tile_of(p + vec2(dx, dz));
            self.walk_tile(x, z)
        })
    }
    pub fn note(&mut self, s: impl Into<String>) {
        self.me.msgs.push((s.into(), self.t))
    }
    /// Building id of the active player's own home.
    pub fn home(&self) -> usize {
        home_of(self.me.id)
    }
    pub fn ppos(&self) -> Vec2 {
        if self.me.scene == Scene::Town {
            self.me.pos
        } else {
            self.me.ipos
        }
    }
    fn set_ppos(&mut self, p: Vec2) {
        if self.me.scene == Scene::Town {
            self.me.pos = p
        } else {
            self.me.ipos = p
        }
    }
    fn uid(&mut self) -> u32 {
        self.w.uid += 1;
        self.w.uid
    }
    /// Snapshot for saving: an object being moved is still owned by the room; resume point is outdoors.
    /// Invariants that must hold after every frame (tests and the stress test call this).
    #[cfg_attr(not(test), allow(dead_code))]
    /// Returns a description of each broken rule; empty means the world is consistent.
    pub fn check(&self) -> Vec<String> {
        let mut bad = vec![];
        let mut uids: Vec<u32> = self.w.ground.iter().map(|g| g.uid).collect();
        for p in std::iter::once(&self.w.p).chain(self.w.others.iter()) {
            let who = &p.name;
            if p.inv.len() > INV_CAP {
                bad.push(format!("{who}: {} pocket slots > {INV_CAP}", p.inv.len()))
            }
            if p.storage.len() > STORE_CAP {
                bad.push(format!("{who}: {} storage slots > {STORE_CAP}", p.storage.len()))
            }
            for s in p.inv.iter().chain(p.storage.iter()) {
                if s.qty == 0 || s.qty > STACK {
                    bad.push(format!("{who}: stack of {} x{}", def(s.def).name, s.qty))
                }
            }
            if p.story > 2 || (p.story != 0 && p.stories.get(p.story as usize).is_some_and(|v| !v.is_empty())) {
                bad.push(format!("{who}: story {} has furniture in two places", p.story))
            }
            uids.extend(p.inv.iter().chain(p.storage.iter()).map(|s| s.uid).filter(|&u| u != 0));
            uids.extend(p.room.iter().chain(p.stories.iter().flatten()).map(|f| f.uid));
        }
        uids.extend(self.me.placing.iter().map(|pl| pl.f.uid));
        uids.sort();
        for w in uids.windows(2) {
            if w[0] == w[1] && w[0] != 0 {
                bad.push(format!("item uid {} exists twice (duplication)", w[0]))
            }
        }
        let mut don: Vec<u16> = self.w.donated.iter().map(|d| d.0).collect();
        don.sort();
        if don.windows(2).any(|w| w[0] == w[1]) {
            bad.push("a museum donation is recorded twice".into())
        }
        bad
    }
    pub fn persist(&self) -> World {
        let mut w = self.w.clone();
        for pl in std::iter::once(&self.me).chain(self.others.iter()) {
            let pr = if w.p.id == pl.id { &mut w.p } else { w.others.iter_mut().find(|p| p.id == pl.id).unwrap() };
            if let Some(o) = pl.placing.as_ref().and_then(|p| p.orig) {
                pr.room.push(o)
            }
            let p = match pl.scene {
                Scene::Town => pl.pos,
                Scene::Room(b) => door_stand(b),
                Scene::Islet => dock_stand(),
                Scene::Frontier => pl.pos,
            };
            pr.pos = [p.x, p.y];
        }
        if w.p.id != 0 {
            if let Some(k) = w.others.iter().position(|p| p.id == 0) {
                std::mem::swap(&mut w.p, &mut w.others[k])
            }
        }
        w.others.sort_by_key(|p| p.id);
        w
    }
}
