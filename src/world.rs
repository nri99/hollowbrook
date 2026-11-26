//! Game state and rules: persistent World, runtime Game, movement, interaction, economy, furniture, residents.
use crate::clock::day_id;
use crate::data::*;
use chrono::{Datelike, NaiveDateTime, Timelike};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

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
pub fn tile_at(map: &[u8], x: i32, z: i32) -> u8 {
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
    // ---- daily refresh (idempotent: one per gameplay day) ----
    pub fn catch_up(&mut self, now: NaiveDateTime) -> bool {
        let today = day_id(now);
        if today < self.w.day {
            if !self.rolled {
                self.rolled = true;
                self.note("Your clock went backwards. Daily events pause until it catches up.")
            }
            return false;
        }
        let n = today - self.w.day;
        if n == 0 {
            return false;
        }
        let l = format!("{now} refresh day {} -> {today} ({n}d)", self.w.day);
        eprintln!("[clock] {l}");
        self.log.push(l);
        self.w.day = today;
        self.note(if n > 1 {
            format!("Welcome back! You were away {n} days.")
        } else {
            "A new day begins in Hollowbrook.".into()
        });
        self.refresh(true);
        true
    }
    pub fn refresh(&mut self, letters: bool) {
        let n = self.w.others.len();
        self.refresh_profile(letters);
        for k in 0..n {
            std::mem::swap(&mut self.w.p, &mut self.w.others[k]);
            self.refresh_profile(letters);
            std::mem::swap(&mut self.w.p, &mut self.w.others[k])
        }
        self.refresh_world();
        self.refresh_moves();
        self.birthday_letters();
    }
    fn refresh_profile(&mut self, letters: bool) {
        self.refresh_social();
        self.settle_contests();
        self.assign_tasks();
        if weekday(self.w.day) == 0 {
            self.home_letter()
        }
        let mine = self.w.p.id == self.me.id;
        let reqs = [self.w.native, 40, 42, 10, 12, 30];
        for i in 0..4 {
            let s = &mut self.w.p.npcs[i];
            s.talked = false;
            s.gifted = false;
            s.visited = false;
            s.req = if gen_range(0, 5) == 0 {
                Some(86)
            } else if gen_range(0, 2) == 0 {
                Some(reqs[gen_range(0, reqs.len())])
            } else {
                None
            };
            if letters && s.friend >= 10 && gen_range(0, 3) == 0 {
                let m = gen_range(2u32, 8) * 100;
                const NOTES: [&str; 4] = [
                    "Thinking of you! Here's a little something.",
                    "Hmph. Found this. Don't make it weird.",
                    "I dreamed you liked coins. So here are some.",
                    "Training fund! Spend it on snacks!",
                ];
                self.w.p.mail.push(Letter {
                    from: self.res(i).name.into(),
                    text: NOTES[self.res(i).pers].into(),
                    coins: m,
                    item: None,
                    read: false,
                    claimed: false,
                });
                if mine {
                    self.me.msgs.push((format!("You have a letter from {} in your mailbox.", self.res(i).name), self.t))
                }
            }
        }
        while self.w.p.mail.len() > 20 {
            let Some(k) =
                self.w.p.mail.iter().position(|l| l.read && (l.claimed || (l.coins == 0 && l.item.is_none())))
            else {
                break;
            };
            self.w.p.mail.remove(k);
        }
    }
    fn mail_all(&mut self, from: &str, text: &str) {
        for p in std::iter::once(&mut self.w.p).chain(self.w.others.iter_mut()) {
            p.mail.push(Letter {
                from: from.into(),
                text: text.into(),
                coins: 0,
                item: None,
                read: false,
                claimed: false,
            })
        }
    }
    /// Moving (provisional): a resident present >= 14 days whose friendship with every player is < 3 and who has no open
    /// request announces a move; two days later a reserve character takes the house unless someone asked them to stay.
    fn birthday_letters(&mut self) {
        for k in 0..4 {
            let c = &CHARS[self.w.roster[k]];
            if (self.mo, self.dom) == BDAYS[self.w.roster[k]] {
                self.mail_all(c.name, "It's my birthday today! If you see me around, come say hi!")
            }
        }
    }
    pub fn refresh_moves(&mut self) {
        let day = self.w.day;
        for k in 0..4 {
            if self.w.moving[k] != 0 && day >= self.w.moving[k] {
                let old = self.res(k).name;
                let Some(new) = (0..CHARS.len()).find(|c| !self.w.roster.contains(c)) else { continue };
                self.w.roster[k] = new;
                self.w.arrived[k] = day;
                self.w.moving[k] = 0;
                let nn = CHARS[new].name;
                for p in std::iter::once(&mut self.w.p).chain(self.w.others.iter_mut()) {
                    p.npcs[k] = NpcS { friend: 0, req: None, talked: false, gifted: false, visited: false }
                }
                let n = &mut self.npcs[k];
                n.inside = true;
                n.path.clear();
                self.mail_all(
                    "Hollowbrook Post",
                    &format!("{old} has moved away. {nn} the {} has moved into the house!", CHARS[new].species),
                );
                self.log.push(format!("day {day}: slot {k} {old} -> {nn}"));
            } else if self.w.moving[k] == 0 && day - self.w.arrived[k] >= 14 {
                let maxf =
                    std::iter::once(&self.w.p).chain(self.w.others.iter()).map(|p| p.npcs[k].friend).max().unwrap_or(0);
                let req = std::iter::once(&self.w.p).chain(self.w.others.iter()).any(|p| p.npcs[k].req.is_some());
                if maxf < 3 && !req {
                    self.w.moving[k] = day + 2;
                    let r = self.res(k);
                    self.mail_all(r.name, "I've been thinking... I might move away in a couple of days. Come talk to me if you'd like me to stay.");
                }
            }
        }
    }
    pub fn refresh_world(&mut self) {
        let day = self.w.day;
        self.w.trampled.clear();
        self.w.branches.clear();
        self.w.rocks_hit.clear();
        self.refresh_islet();
        self.finish_mproject();
        self.assign_weekly();
        self.grow_garden();
        self.new_surprise();
        let lv = self.shop_tier();
        if lv > self.w.shop_lv {
            self.w.shop_lv = lv;
            self.mail_all("Mr. Tumble", if lv == 1 { "Thanks to all of you, the General Store has been renovated! Look for a new weekly special by the window." } else { "Another renovation! The General Store now stocks an extra item every day." })
        }
        if self.w.camp_day != self.w.day {
            self.w.camp_day = self.w.day;
            self.w.camp_sold.clear()
        }
        if self.w.project.is_none() {
            self.propose()
        } // flattened flowers recover each morning
        for t in &mut self.w.trees {
            if !t.fruit && day - t.shaken >= 3 {
                t.fruit = true
            }
        } // persisted elapsed-time regrowth rule
        for _ in 0..self.pol().map_or(2, |p| p.weeds) {
            if self.w.weeds.len() < 12 {
                if let Some(t) = self.rand_tile(GRASS) {
                    self.w.weeds.push(t)
                }
            }
        }
        while self.w.ground.iter().filter(|g| def(g.def).k == K::Shell).count() < 4 {
            let Some(t) = self.rand_tile(SAND) else { break };
            self.drop_item([40, 42, 40, 42, 41][gen_range(0, 5)], t.0, t.1)
        }
        while self.w.digs.len() < 2 {
            let Some(t) = self.rand_tile(GRASS) else { break };
            self.w.digs.push(t)
        }
        if self.w.stock_day != day {
            let mut pool = FURN_IDS.to_vec();
            let n = 5 + (self.shop_tier() >= 2) as usize;
            self.w.stock = (0..n).map(|_| Stock { def: pool.remove(gen_range(0, pool.len())), sold: false }).collect();
            self.w.stock_day = day
        }
    }
    fn rand_tile(&self, kind: u8) -> Option<(i32, i32)> {
        for _ in 0..300 {
            let (x, z) = (gen_range(1, W as i32 - 1), gen_range(1, H as i32 - 1));
            if tile_at(&self.map, x, z) == kind
                && self.walk_tile(x, z)
                && !self.occupied(x, z)
                && (x, z) != tile_of(self.me.pos)
            {
                return Some((x, z));
            }
        }
        None
    }
    fn occupied(&self, x: i32, z: i32) -> bool {
        self.w.garden.iter().any(|p| (p.x, p.z) == (x, z))
            || self.item_at(x, z)
            || self.w.weeds.contains(&(x, z))
            || self.w.digs.contains(&(x, z))
            || self.w.holes.contains(&(x, z))
    }
    fn item_at(&self, x: i32, z: i32) -> bool {
        self.w.ground.iter().any(|g| tile_of(vec2(g.x, g.z)) == (x, z))
    }
    fn drop_item(&mut self, d: u16, x: i32, z: i32) {
        let uid = self.uid();
        self.w.ground.push(GItem {
            uid,
            def: d,
            x: x as f32 + 0.5 + gen_range(-0.15, 0.15),
            z: z as f32 + 0.5 + gen_range(-0.15, 0.15),
        })
    }

    // ---- collision ----
    pub fn tile(&self, x: i32, z: i32) -> u8 {
        tile_at(&self.map, x, z)
    }
    pub fn fair(&self) -> bool {
        fair_active(self.mo, self.dom)
    }
    /// Display name of a building (resident houses follow their current resident).
    pub fn bname(&self, b: usize) -> String {
        match BLDS[b].kind {
            BK::Res(k) => format!("{}'s House", self.res(k).name),
            _ => BLDS[b].name.into(),
        }
    }
    /// Character living in house slot `slot`.
    pub fn res(&self, slot: usize) -> &'static Res {
        &CHARS[self.w.roster[slot]]
    }
    pub fn pol(&self) -> Option<&'static Policy> {
        self.w.policy.and_then(|i| POLICIES.get(i))
    }
    /// Resident location with the active policy applied (wake/sleep shifts; exempt characters unchanged).
    pub fn loc(&self, slot: usize) -> Loc {
        if self.invited_home(slot) {
            return Loc::Home;
        }
        if self.seek_hide_position(slot).is_some() {
            return Loc::Out;
        }
        let r = self.res(slot);
        let (dw, ds) = match self.pol() {
            Some(p) if !p.exempt.contains(&self.w.roster[slot]) => (p.wake, p.sleep),
            _ => (0, 0),
        };
        let (wake, sleep, hr) = ((r.wake as i32 + dw).max(0), (r.sleep as i32 + ds).min(24), self.hr as i32);
        if !(hr >= wake && hr < sleep) {
            Loc::Asleep
        } else if r
            .home_h
            .iter()
            .enumerate()
            .any(|(k, &(a, b))| hr >= a as i32 && hr < b as i32 + if k == 1 { ds } else { 0 })
        {
            Loc::Home
        } else {
            Loc::Out
        }
    }
    pub fn shop_hours(&self) -> (u32, u32) {
        self.pol().map_or((8, 22), |p| p.shop)
    }
    pub fn shop_is_open(&self) -> bool {
        let (a, b) = self.shop_hours();
        (a..b).contains(&self.hr)
    }
    /// Leader enacts policy `i` (fee from the leader's own coins), or repeals with None (free).
    pub fn enact(&mut self, i: Option<usize>) -> Result<(), &'static str> {
        if self.me.id != LEADER {
            return Err("Only the town leader can change policies.");
        }
        let Some(i) = i else {
            self.w.policy = None;
            return Ok(());
        };
        if self.w.policy == Some(i) {
            return Err("That policy is already in effect.");
        }
        let fee = POLICIES[i].fee;
        if self.w.p.money < fee {
            return Err("Not enough coins for the policy fee.");
        }
        self.w.p.money -= fee;
        self.w.policy = Some(i);
        Ok(())
    }
    pub fn walk_tile(&self, x: i32, z: i32) -> bool {
        if (x, z) == DOCK {
            return false;
        }
        if (self.fair() || self.fest().is_some()) && (x, z) == STALL {
            return false;
        }
        if (x, z) == BOARD {
            return false;
        }
        if camp_block((x, z)) {
            return false;
        }
        if self.w.mproject.as_ref().is_some_and(|m| (m.x, m.z) == (x, z)) {
            return false;
        }
        if let Some(b) = self.w.built.iter().find(|b| ptiles(b.kind, b.x, b.z).contains(&(x, z))) {
            return b.kind == PK_BRIDGE;
        }
        if self.w.project.as_ref().is_some_and(|p| p.kind != PK_BRIDGE && (p.x, p.z) == (x, z)) {
            return false;
        }
        matches!(self.tile(x, z), GRASS | PATH | SAND | BRIDGE | FLOWER)
            && !self.w.holes.contains(&(x, z))
            && (x, z) != mailbox_tile(0)
            && (x, z) != mailbox_tile(1)
    }
    pub fn room_furn(&self, b: usize) -> Vec<Furn> {
        match BLDS[b].kind {
            BK::Home(o) => {
                if self.w.p.id == o {
                    self.w.p.room.clone()
                } else {
                    self.w
                        .others
                        .iter()
                        .find(|p| p.id == o)
                        .map(
                            |p| {
                                if p.story == 0 {
                                    p.room.clone()
                                } else {
                                    p.stories.first().cloned().unwrap_or_default()
                                }
                            },
                        )
                        .unwrap_or_default()
                }
            }
            BK::Res(r) => TEMPL[r].iter().map(|&(def, x, z, rot)| Furn { uid: 0, def, x, z, rot, on: true }).collect(),
            _ => vec![],
        }
    }
    /// Story of building `b` the active player is on (only owners use stairs; visitors stay on the main floor).
    pub fn story_in(&self, b: usize) -> u8 {
        match BLDS[b].kind {
            BK::Home(o) if self.w.p.id == o => self.w.p.story,
            _ => 0,
        }
    }
    /// Stairs visible on the active player's story of building `b`.
    pub fn home_stairs(&self, b: usize) -> Vec<((i32, i32), u8)> {
        if let BK::Home(o) = BLDS[b].kind {
            stairs(self.owner(o).home, self.story_in(b))
        } else {
            vec![]
        }
    }
    /// Swap the active player's furniture to another story (no movement).
    pub fn set_story(&mut self, to: u8) {
        let p = &mut self.w.p;
        if p.story == to {
            return;
        }
        p.stories.resize(3, vec![]);
        let cur = p.story as usize;
        p.stories[cur] = std::mem::take(&mut p.room);
        p.room = std::mem::take(&mut p.stories[to as usize]);
        p.story = to
    }
    pub fn take_stairs(&mut self, to: u8) {
        let t = if to == 2 || self.w.p.story == 2 { STAIR_UP } else { STAIR_DOWN };
        self.place_cancel();
        self.set_story(to);
        self.me.ipos = vec2(t.0 as f32 + 0.5, t.1 as f32 + 1.5);
        self.me.face = vec2(0., 1.);
        self.me.cam = self.me.ipos;
        self.snd.push(Snd::Door);
        self.note(match to {
            0 => "Back on the main floor.",
            1 => "Down in the basement.",
            _ => "Up in the attic room.",
        });
    }
    pub fn room_block(&self, b: usize, x: i32, z: i32, skip: u32) -> bool {
        let (w, d) = self.rdims(b);
        if x < 0 || z < 0 || x >= w || z >= d {
            return true;
        }
        if self.home_stairs(b).iter().any(|s| s.0 == (x, z)) {
            return true;
        }
        if BLDS[b].kind == BK::Shop {
            return COUNTER.contains(&(x, z))
                || DISPLAYS.contains(&(x, z))
                || (x, z) == RACK
                || ((x, z) == SPECIAL_DISPLAY && self.shop_tier() >= 1)
                || ((x, z) == EXTRA_DISPLAY && self.w.stock.len() > 5);
        }
        if BLDS[b].kind == BK::Civic {
            return CIVIC_DESK.contains(&(x, z))
                || CIVIC_PLANTS.contains(&(x, z))
                || CIVIC_BENCH.contains(&(x, z))
                || (x, z) == KIOSK;
        }
        if BLDS[b].kind == BK::Museum {
            return MUSEUM_DESK.contains(&(x, z))
                || MUSEUM_TANK.contains(&(x, z))
                || MUSEUM_CASE.contains(&(x, z))
                || MUSEUM_PED.iter().any(|p| p.0 == (x, z));
        }
        self.room_furn(b)
            .iter()
            .filter(|f| !def(f.def).wall && !(skip != 0 && f.uid == skip))
            .any(|f| foot(f.def, f.x, f.z, f.rot).contains(&(x, z)))
    }
    pub fn free(&self, p: Vec2) -> bool {
        [(-PR, -PR), (PR, -PR), (-PR, PR), (PR, PR)].iter().all(|&(dx, dz)| {
            let (x, z) = tile_of(p + vec2(dx, dz));
            match self.me.scene {
                Scene::Town => self.walk_tile(x, z) || self.tile(x, z) == SEA,
                Scene::Room(b) => !self.room_block(b, x, z, 0),
                Scene::Islet => matches!(tile_at_i(&self.imap, x, z), GRASS | SAND | BRIDGE | SEA) && (x, z) != ISTALL,
            }
        })
    }
    pub fn step_move(&mut self, d: Vec2, run: bool, dt: f32) {
        self.me.spd = 0.;
        if d == Vec2::ZERO {
            return;
        }
        if self.me.pose != Pose::Stand {
            self.me.pose = Pose::Stand;
            let sf = self.me.stand_from;
            self.set_ppos(sf);
            return;
        }
        if self.me.fish.is_some() {
            self.me.fish = None;
            self.note("You reel in your line.")
        }
        let d = d.normalize();
        self.me.face = d;
        let s = (if self.swimming() {
            2.0
        } else if run {
            5.5
        } else {
            3.0
        }) * dt;
        let p0 = self.ppos();
        let mut p = p0;
        let nx = p + vec2(d.x * s, 0.);
        if self.free(nx) {
            p = nx
        }
        let nz = p + vec2(0., d.y * s);
        if self.free(nz) {
            p = nz
        } else if let Scene::Room(b) = self.me.scene {
            let (w, dd) = self.rdims(b);
            if d.y > 0. && self.story_in(b) == 0 && p.x.floor() as i32 == w / 2 && p.y > dd as f32 - 0.5 {
                self.set_ppos(p);
                self.go(Scene::Town);
                return;
            }
        }
        self.set_ppos(p);
        let moved = (p - p0).length();
        self.me.spd = moved / dt.max(1e-4);
        self.me.step_acc += moved;
        if self.me.step_acc > if run { 0.9 } else { 0.6 } {
            self.me.step_acc = 0.;
            let st = self.step_sound();
            self.snd.push(st)
        }
        if run && self.me.scene == Scene::Town {
            self.bugs.retain(|b| b.1.distance(p) > 2.5);
            let t = tile_of(p);
            if self.tile(t.0, t.1) == FLOWER && !self.w.trampled.contains(&t) && gen_range(0, 25) == 0 {
                self.w.trampled.push(t);
                self.note("Oops, you trampled a flower. It'll perk up by morning.")
            }
        }
    }
    /// Footstep sound for the surface under the active player.
    pub fn step_sound(&self) -> Snd {
        let t = tile_of(self.ppos());
        let c = match self.me.scene {
            Scene::Town => self.tile(t.0, t.1),
            Scene::Islet => tile_at_i(&self.imap, t.0, t.1),
            Scene::Room(_) => return Snd::StepFloor,
        };
        if self.w.built.iter().any(|b| b.kind == PK_BRIDGE && ptiles(b.kind, b.x, b.z).contains(&t)) {
            return Snd::StepWood;
        }
        match c {
            PATH => Snd::StepPath,
            SAND => Snd::StepSand,
            BRIDGE => Snd::StepWood,
            SEA | RIVER => Snd::Swim,
            _ => Snd::Step,
        }
    }
    pub fn go(&mut self, to: Scene) {
        if self.me.trans.is_none() {
            self.me.trans = Some((0., to, false));
            self.snd.push(Snd::Door)
        }
    }
    pub fn swap(&mut self, to: Scene) {
        self.place_cancel();
        self.set_story(0);
        if to != Scene::Islet {
            let id = self.me.id;
            if let Some(a) = &mut self.act {
                if !a.done {
                    a.players.retain(|&p| p != id)
                }
            }
        } // leaving forfeits
        match to {
            Scene::Islet => {
                self.me.ipos = vec2(ISLET_SPAWN.0, ISLET_SPAWN.1);
                self.me.face = vec2(0., -1.);
                self.me.cam = self.me.ipos
            }
            Scene::Town => {
                if let Scene::Room(b) = self.me.scene {
                    self.me.pos = door_stand(b)
                }
                if self.me.scene == Scene::Islet {
                    self.me.pos = dock_stand()
                }
                self.me.face = vec2(0., 1.);
                self.me.cam = self.me.pos
            }
            Scene::Room(b) => {
                let (w, d) = self.rdims(b);
                self.me.ipos = vec2((w / 2) as f32 + 0.5, d as f32 - 0.5);
                self.me.face = vec2(0., -1.);
                self.me.cam = self.me.ipos
            }
        }
        eprintln!("[scene] {:?} -> {:?}", self.me.scene, to);
        self.me.scene = to;
        self.visit_arrival();
        self.me.pose = Pose::Stand;
        self.me.fish = None;
    }

    // ---- targeting: distance + facing + line of sight; doors win when deliberately faced ----
    pub fn target(&self) -> Option<Tg> {
        let p = self.ppos();
        let f = self.me.face;
        let mut best: Option<(f32, Tg)> = None;
        let mut cand = |c: Vec2, reach: f32, wt: f32, tg: Tg, own: Option<(i32, i32)>| {
            let v = c - p;
            let d = v.length();
            if d > reach {
                return;
            }
            let dot = if d < 0.05 { 1. } else { v.dot(f) / d };
            if dot < 0.3 {
                return;
            }
            if self.me.scene == Scene::Town && !self.los(p, c, own) {
                return;
            }
            let s = d * (1.6 - dot) * wt;
            if best.map_or(true, |b| s < b.0) {
                best = Some((s, tg))
            }
        };
        match self.me.scene {
            Scene::Town => {
                for (i, n) in self.npcs.iter().enumerate() {
                    if !n.inside {
                        cand(n.pos, 1.7, 1., Tg::Npc(i), None)
                    }
                }
                for b in 0..BLDS.len() {
                    cand(door_face(b), 1.35, 0.6, Tg::Door(b), None)
                }
                for (i, t) in self.w.trees.iter().enumerate() {
                    cand(tc((t.x, t.z)), 1.45, 1., Tg::Tree(i), Some((t.x, t.z)))
                }
                for (i, g) in self.w.ground.iter().enumerate() {
                    cand(vec2(g.x, g.z), 1.2, 0.8, Tg::Item(i), None)
                }
                for (i, &w) in self.w.weeds.iter().enumerate() {
                    cand(tc(w), 1.2, 1., Tg::Weed(i), None)
                }
                for o in 0..2 {
                    cand(tc(mailbox_tile(o)), 1.4, 0.9, Tg::Mailbox(o), Some(mailbox_tile(o)))
                }
                if let Some(p) = &self.w.project {
                    let c = if p.kind == PK_BRIDGE { vec2(p.x as f32 + 1., p.z as f32 + 1.2) } else { tc((p.x, p.z)) };
                    cand(c, 1.5, 0.8, Tg::Site, Some((p.x, p.z)))
                }
                cand(tc(BOARD), 1.5, 0.8, Tg::Board, Some(BOARD));
                if self.contest_host_open() {
                    let (x, z) = crate::events::CONTEST_SPOT;
                    cand(vec2(x, z), 1.5, 0.7, Tg::Contest, None);
                }
                if let Some(m) = &self.w.mproject {
                    cand(tc((m.x, m.z)), 1.5, 0.8, Tg::MSite, Some((m.x, m.z)))
                }
                cand(vec2(DOCK.0 as f32 + 0.5, DOCK.1 as f32 + 0.3), 1.4, 0.7, Tg::Dock, None);
                for (i, b) in self.w.built.iter().enumerate() {
                    if b.kind == 0 {
                        cand(tc((b.x, b.z)), 1.4, 0.9, Tg::Built(i), Some((b.x, b.z)))
                    }
                }
                if self.camp_open() {
                    cand(vec2(CAMP_SPOT.0, CAMP_SPOT.1), 1.5, 0.8, Tg::Camper, None)
                }
                cand(tc(CAMP_SIGN), 1.4, 0.8, Tg::CampSign, Some(CAMP_SIGN));
                if let Some(sp) = &self.w.surprise {
                    cand(tc((sp.x, sp.z)), 1.3, 0.9, Tg::Surprise, Some((sp.x, sp.z)))
                }
                for (i, p) in self.w.garden.iter().enumerate() {
                    cand(tc((p.x, p.z)), 1.2, 0.9, Tg::Plant(i), Some((p.x, p.z)))
                }
                if self.w.p.inv.iter().any(|s| SEEDS.contains(&s.def)) {
                    let ft = tile_of(self.me.pos + self.me.face * 0.9);
                    if self.plant_ok(ft) {
                        cand(tc(ft), 1.2, 0.85, Tg::PlantSpot, Some(ft))
                    }
                }
                if self.fair() {
                    cand(tc(STALL), 1.5, 0.8, Tg::Stall, Some(STALL))
                } else if self.fest().is_some() {
                    cand(tc(STALL), 1.5, 0.8, Tg::Fest, Some(STALL))
                } else if self.art_open() {
                    cand(tc(STALL), 1.5, 0.8, Tg::ArtStall, Some(STALL))
                }
            }
            Scene::Islet => {
                cand(vec2(PELL_I.0, PELL_I.1), 1.6, 0.8, Tg::Pell, None);
                cand(tc(ACT_BOARD), 1.4, 0.8, Tg::ActBoard, None);
                cand(tc(ISTALL), 1.5, 0.9, Tg::IStall, None);
                for (i, g) in self.w.iground.iter().enumerate() {
                    cand(vec2(g.x, g.z), 1.2, 0.8, Tg::IItem(i), None)
                }
            }
            Scene::Room(b) => {
                for fu in self.room_furn(b) {
                    cand(fcenter(&fu), 1.5, 1., Tg::Furn(fu.uid, fu.def), None)
                }
                if matches!(BLDS[b].kind, BK::Home(o) if o == self.w.p.id) {
                    for (t, to) in self.home_stairs(b) {
                        cand(tc(t), 1.6, 1., Tg::Stairs(to), None)
                    }
                }
                if BLDS[b].kind == BK::Shop {
                    cand(vec2(3.5, 1.5), 1.4, 0.9, Tg::Counter, None);
                    cand(vec2(3.5, 0.5), 2.6, 1.3, Tg::Keeper, None);
                    for (i, &d) in DISPLAYS.iter().enumerate() {
                        cand(tc(d), 1.5, 1., Tg::Display(i), None)
                    }
                    cand(tc(RACK), 1.5, 1., Tg::Rack, None);
                    if self.w.stock.len() > 5 {
                        cand(tc(EXTRA_DISPLAY), 1.5, 1., Tg::Display(7), None)
                    }
                    if self.shop_tier() >= 1 {
                        cand(tc(SPECIAL_DISPLAY), 1.5, 1., Tg::Display(8), None)
                    }
                }
                if BLDS[b].kind == BK::Civic {
                    cand(vec2(3.5, 1.5), 1.4, 0.9, Tg::Hazel, None);
                    cand(tc(KIOSK), 1.4, 0.9, Tg::Kiosk, None)
                }
                if BLDS[b].kind == BK::Museum {
                    cand(vec2(3.5, 1.5), 1.4, 0.9, Tg::Curator, None);
                    cand(vec2(0.5, 2.5), 1.5, 1., Tg::Exhibit(0), None);
                    cand(vec2(6.5, 2.5), 1.5, 1., Tg::Exhibit(1), None);
                    for (k, p) in MUSEUM_PED.iter().enumerate() {
                        cand(tc(p.0), 1.5, 1., Tg::Exhibit(2 + k), None)
                    }
                }
                if let BK::Res(r) = BLDS[b].kind {
                    if self.loc(r) == Loc::Home {
                        cand(res_spot(), 1.7, 0.9, Tg::Npc(r), None)
                    }
                }
                if let Some((r, vb, _)) = self.visit {
                    if vb == b {
                        cand(self.visit_spot(b), 1.7, 0.9, Tg::Npc(r), None)
                    }
                }
            }
        }
        best.map(|b| b.1)
    }
    fn los(&self, a: Vec2, b: Vec2, own: Option<(i32, i32)>) -> bool {
        (1..10).all(|k| {
            let t = tile_of(a.lerp(b, k as f32 / 10.));
            Some(t) == own || !matches!(self.tile(t.0, t.1), BLD | PINE)
        })
    }
    pub fn tg_pos(&self, tg: Tg) -> Vec3 {
        let v = |p: Vec2, y: f32| vec3(p.x, y, p.y);
        match tg {
            Tg::Contest => vec3(crate::events::CONTEST_SPOT.0, 1.5, crate::events::CONTEST_SPOT.1),
            Tg::Npc(i) => v(
                match self.me.scene {
                    Scene::Town => self.npcs[i].pos,
                    Scene::Room(b) if self.visit.is_some_and(|x| x.0 == i && x.1 == b) => self.visit_spot(b),
                    _ => res_spot(),
                },
                1.5,
            ),
            Tg::Door(b) => v(door_face(b), 1.3),
            Tg::Tree(i) => v(tc((self.w.trees[i].x, self.w.trees[i].z)), 2.15),
            Tg::Item(i) => vec3(self.w.ground[i].x, 0.45, self.w.ground[i].z),
            Tg::Stairs(_) => {
                let t = self
                    .home_stairs(match self.me.scene {
                        Scene::Room(b) => b,
                        _ => HOME,
                    })
                    .first()
                    .map_or(STAIR_UP, |s| s.0);
                vec3(t.0 as f32 + 0.5, 1.6, t.1 as f32 + 0.5)
            }
            Tg::Weed(i) => v(tc(self.w.weeds[i]), 0.55),
            Tg::Rack => vec3(1.5, 1.8, 1.5),
            Tg::Mailbox(o) => v(tc(mailbox_tile(o)), 1.4),
            Tg::Stall | Tg::ArtStall | Tg::Fest => v(tc(STALL), 1.9),
            Tg::Camper => vec3(CAMP_SPOT.0, 1.7, CAMP_SPOT.1),
            Tg::Plant(i) => v(tc((self.w.garden[i].x, self.w.garden[i].z)), 0.7),
            Tg::PlantSpot => v(tc(tile_of(self.me.pos + self.me.face * 0.9)), 0.4),
            Tg::Surprise => self.w.surprise.as_ref().map_or(Vec3::ZERO, |sp| v(tc((sp.x, sp.z)), 0.9)),
            Tg::CampSign => v(tc(CAMP_SIGN), 1.5),
            Tg::Kiosk => v(tc(KIOSK), 1.5),
            Tg::Board => v(tc(BOARD), 1.9),
            Tg::MSite => self.w.mproject.as_ref().map_or(Vec3::ZERO, |m| v(tc((m.x, m.z)), 1.6)),
            Tg::Dock => vec3(DOCK.0 as f32 + 0.5, 1.6, DOCK.1 as f32 + 0.3),
            Tg::ActBoard => v(tc(ACT_BOARD), 1.5),
            Tg::Pell => vec3(PELL_I.0, 1.6, PELL_I.1),
            Tg::IStall => v(tc(ISTALL), 1.5),
            Tg::IItem(i) => vec3(self.w.iground[i].x, 0.45, self.w.iground[i].z),
            Tg::Built(i) => v(tc((self.w.built[i].x, self.w.built[i].z)), 1.2),
            Tg::Site => self.w.project.as_ref().map_or(Vec3::ZERO, |p| v(tc((p.x, p.z)), 1.6)),
            Tg::Curator | Tg::Hazel => vec3(3.5, 1.3, 1.5),
            Tg::Exhibit(k) => match k {
                0 => vec3(0.5, 1.5, 2.5),
                1 => vec3(6.5, 1.5, 2.5),
                _ => {
                    let p = tc(MUSEUM_PED[k - 2].0);
                    vec3(p.x, 1.2, p.y)
                }
            },
            Tg::Counter => vec3(3.5, 1.3, 1.5),
            Tg::Keeper => vec3(3.5, 1.7, 0.5),
            Tg::Display(i) => v(
                tc(match i {
                    7 => EXTRA_DISPLAY,
                    8 => SPECIAL_DISPLAY,
                    _ => DISPLAYS[i],
                }),
                1.3,
            ),
            Tg::Furn(uid, d) => {
                let c = self
                    .room_furn(match self.me.scene {
                        Scene::Room(b) => b,
                        _ => HOME,
                    })
                    .into_iter()
                    .find(|f| f.uid == uid && f.def == d)
                    .map(|f| fcenter(&f))
                    .unwrap_or_default();
                v(c, if def(d).wall { 1.9 } else { 1.2 })
            }
        }
    }
    pub fn tg_label(&self, tg: Tg) -> String {
        match tg {
            Tg::Contest => "Talk to Mara (monthly survey)".into(),
            Tg::Npc(i) => format!("Talk to {}", self.res(i).name),
            Tg::Door(b) => format!("Enter {}", self.bname(b)),
            Tg::Tree(i) => {
                if self.w.trees[i].fruit {
                    "Shake tree".into()
                } else {
                    "Shake tree (no fruit)".into()
                }
            }
            Tg::Item(i) => format!("Pick up {}", def(self.w.ground[i].def).name),
            Tg::Weed(_) => "Pull weed".into(),
            Tg::Furn(uid, d) => {
                let a = match def(d).act {
                    Act::Sit => "Sit on",
                    Act::Bed => "Lie on",
                    Act::Lamp => "Toggle",
                    Act::Storage => "Open",
                    Act::None => "Look at",
                };
                if uid != 0 && self.me.scene == Scene::Room(self.home()) {
                    format!("{a} {}   [F] Move  [G] Pick up", def(d).name)
                } else {
                    format!("{a} {}", def(d).name)
                }
            }
            Tg::Counter => "Sell items".into(),
            Tg::Keeper => "Talk to Mr. Tumble (seeds, tool upgrades)".into(),
            Tg::Hazel => "Talk to Hazel Quill (civic desk)".into(),
            Tg::Curator => "Donate / identify fossils".into(),
            Tg::Rack => "Browse clothes".into(),
            Tg::Site => self
                .w
                .project
                .as_ref()
                .map_or(String::new(), |p| format!("Contribute to the {}", PROJECTS[p.kind as usize].0)),
            Tg::Board => "Read the town notice board".into(),
            Tg::MSite => self.w.mproject.as_ref().map_or(String::new(), |m| format!("Fund the {}", MPROJECTS[m.idx].0)),
            Tg::Dock => format!("Sail to Gullrock Islet ({FARE} coins)"),
            Tg::ActBoard => "Timed activities board".into(),
            Tg::Pell => "Talk to Captain Pell".into(),
            Tg::IStall => "Sea Glass stall".into(),
            Tg::IItem(i) => format!("Pick up {}", def(self.w.iground[i].def).name),
            Tg::Built(_) => "Sit on the bench".into(),
            Tg::Camper => format!("Talk to {} the {}", VISITORS[self.visitor()].name, VISITORS[self.visitor()].species),
            Tg::CampSign => "Read the campground sign".into(),
            Tg::Kiosk => format!("Initiatives kiosk ({} coupons)", self.w.p.coupons),
            Tg::Plant(i) => {
                let p = self.w.garden[i];
                let n = def(FLOWERS[p.kind as usize]).name;
                if p.stage == 2 {
                    format!("Pick the {n}")
                } else {
                    format!(
                        "{n} ({}{})",
                        ["sprout", "bud"][p.stage as usize],
                        if p.watered == self.w.day { ", watered" } else { ", needs water" }
                    )
                }
            }
            Tg::PlantSpot => format!(
                "Plant {}",
                self.w.p.inv.iter().find(|s| SEEDS.contains(&s.def)).map_or("", |s| def(s.def).name)
            ),
            Tg::Surprise => self
                .w
                .surprise
                .as_ref()
                .map_or(String::new(), |sp| format!("Pick up the {}", SURPRISES[sp.kind as usize])),
            Tg::Stairs(to) => match to {
                0 => "Go back to the main floor",
                1 => "Go down to the basement",
                _ => "Go upstairs",
            }
            .into(),
            Tg::Fest => format!("Talk to Hazel about the {}", self.fest().map_or("", |f| FESTS[f].name)),
            Tg::ArtStall => format!("Art stall: {} ({ART_PRICE} coins)", def(art_of_week(self.w.day)).name),
            Tg::Stall => {
                if self.w.p.fair_year == self.yr {
                    "Harvest Fair stall (prize claimed)".into()
                } else {
                    "Claim a Harvest Fair prize".into()
                }
            }
            Tg::Mailbox(o) => {
                if o == self.me.id {
                    let n = self.w.p.mail.iter().filter(|l| !l.read).count();
                    if n > 0 {
                        format!("Check mailbox ({n} new)")
                    } else {
                        "Check mailbox".into()
                    }
                } else {
                    format!("Player {}'s mailbox", o + 1)
                }
            }
            Tg::Exhibit(k) => match k {
                0 => "Look at the fish tank".into(),
                1 => "Look at the bug case".into(),
                _ => format!("Look at the {} pedestal", def(MUSEUM_PED[k - 2].1).name),
            },
            Tg::Display(i) if i >= 7 => {
                let (d, p, sold) = self.display_item(i);
                if sold {
                    "Sold out".into()
                } else {
                    format!("{}Buy {} ({p} coins)", if i == 8 { "Weekly special: " } else { "" }, def(d).name)
                }
            }
            Tg::Display(i) => {
                if i < 5 {
                    let s = self.w.stock[i];
                    if s.sold {
                        "Sold out".into()
                    } else {
                        format!("Buy {} ({} coins)", def(s.def).name, def(s.def).buy)
                    }
                } else {
                    let t = TOOL_DISPLAY[i - 5];
                    format!("Buy {} ({} coins)", def(t).name, def(t).buy)
                }
            }
        }
    }

    // ---- interaction ----
    pub fn interact(&mut self) {
        let Some(tg) = self.target() else { return };
        self.snd.push(Snd::Blip);
        match tg {
            Tg::Contest => self.contest_talk(),
            Tg::Npc(i) => self.talk(i),
            Tg::Door(b) => self.door(b),
            Tg::Tree(i) => self.shake_tree(i),
            Tg::Item(i) => {
                let it = self.w.ground[i];
                if self.add(it.def) {
                    self.w.ground.retain(|g| g.uid != it.uid);
                    if def(it.def).k == K::Shell {
                        self.ev(Ev::Shell, 1)
                    }
                }
            }
            Tg::Weed(i) => {
                self.w.weeds.remove(i);
                self.note("You pulled a weed. Tidy!");
                self.ev(Ev::Weed, 1)
            }
            Tg::Furn(uid, d) => self.use_furn(uid, d),
            Tg::Counter => self.me.ui = Ui::Sell(0),
            Tg::Curator => self.donate(),
            Tg::Hazel => self.me.ui = Ui::Civic(0),
            Tg::Rack => self.me.ui = Ui::Rack(0),
            Tg::Stall => self.fair_prize(),
            Tg::Fest => self.fest_talk(),
            Tg::Surprise => self.take_surprise(),
            Tg::Stairs(to) => self.take_stairs(to),
            Tg::Plant(i) => self.pick_flower(i),
            Tg::PlantSpot => {
                let ft = tile_of(self.me.pos + self.me.face * 0.9);
                if let Err(e) = self.plant(ft) {
                    self.note(e)
                }
            }
            Tg::Camper => self.me.ui = Ui::Camp(0),
            Tg::CampSign => {
                let v = &VISITORS[self.visitor()];
                let n = &VISITORS[visitor_of(self.w.day + 1)];
                self.me.ui = Ui::Talk { npc: None, who: "Campground sign".into(), lines: vec![format!("Hollowbrook Campground. Travellers stay one day each, at the campfire from {}:00 to {}:00.", CAMP_HOURS.0, CAMP_HOURS.1), format!("Today: {} the {}. Tomorrow: {} the {}.", v.name, v.species, n.name, n.species), "Each traveller sells one of each item per day and has one favor to ask.".into()], i: 0 }
            }
            Tg::Kiosk => self.me.ui = Ui::Kiosk(0),
            Tg::ArtStall => {
                let r = self.art_buy();
                match r {
                    Ok(n) => {
                        self.snd.push(Snd::Coin);
                        self.note(format!("You bought \"{n}\". The museum's art wing would love it!"))
                    }
                    Err(e) => self.note(e),
                }
            }
            Tg::Site => self.contribute(),
            Tg::Dock => self.me.ui = Ui::Travel(true),
            Tg::Pell => self.pell(),
            Tg::IStall => self.me.ui = Ui::IStall(0),
            Tg::ActBoard => {
                if self.act.is_some() {
                    self.note("An activity is already running.")
                } else {
                    self.me.ui = Ui::Act(0)
                }
            }
            Tg::IItem(i) => {
                let it = self.w.iground[i];
                if self.add(it.def) {
                    self.w.iground.retain(|g| g.uid != it.uid)
                }
            }
            Tg::Board => {
                let lines = self.board_lines();
                self.me.ui = Ui::Talk { npc: None, who: "Town notice board".into(), lines, i: 0 }
            }
            Tg::MSite => self.me.ui = Ui::Fund,
            Tg::Built(i) => {
                let b = self.w.built[i];
                self.me.stand_from = self.me.pos;
                self.me.pos = tc((b.x, b.z));
                self.me.face = vec2(0., 1.);
                self.me.pose = Pose::Sit(0);
                self.note("You sit on the bench. Move to stand up.")
            }
            Tg::Mailbox(o) => {
                if o == self.me.id {
                    self.me.ui = Ui::Mail(0)
                } else {
                    self.note(format!("That's Player {}'s mailbox. Better not peek.", o + 1))
                }
            }
            Tg::Exhibit(k) => {
                let (n, t) = match k {
                    0 => (self.donated_of(K::Fish), DEFS.iter().filter(|d| d.k == K::Fish).count()),
                    1 => (self.donated_of(K::Bug), DEFS.iter().filter(|d| d.k == K::Bug).count()),
                    _ => (self.w.donated.iter().any(|x| x.0 == MUSEUM_PED[k - 2].1) as usize, 1),
                };
                let who = if k >= 2 {
                    self.w
                        .donated
                        .iter()
                        .find(|x| x.0 == MUSEUM_PED[k - 2].1)
                        .map(|x| format!(" (donated by Player {})", x.1 + 1))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                self.note(format!(
                    "{}: {n}/{t} on display.{who}",
                    ["Fish tank", "Bug case"].get(k).copied().unwrap_or(def(MUSEUM_PED[k.max(2) - 2].1).name)
                ))
            }
            Tg::Keeper => self.me.ui = Ui::Keeper(0),
            Tg::Display(i) => {
                if i >= 7 && self.display_item(i).2 {
                    self.note(if i == 8 {
                        "This week's special is sold. A new one arrives next week."
                    } else {
                        "That one's sold out. New stock arrives at 6:00."
                    })
                } else if i < 5 && self.w.stock[i].sold {
                    self.note("That one's sold out. New stock arrives at 6:00.")
                } else if (5..7).contains(&i) && self.w.p.tools.contains(&TOOL_DISPLAY[i - 5]) {
                    self.note("You already own this tool.")
                } else {
                    self.me.ui = Ui::Confirm(i)
                }
            }
        }
    }
    fn door(&mut self, b: usize) {
        match BLDS[b].kind {
            BK::Home(_) => self.go(Scene::Room(b)),
            BK::Shop => {
                if self.shop_is_open() {
                    self.go(Scene::Room(b))
                } else {
                    let (a, c) = self.shop_hours();
                    self.note(format!("The General Store is closed. Hours: {a}:00-{c}:00."))
                }
            }
            BK::Res(r) => match self.loc(r) {
                Loc::Home => self.go(Scene::Room(b)),
                Loc::Asleep => self.note(format!("It's late. {} is asleep.", self.res(r).name)),
                Loc::Out => self.note(format!("{} isn't home right now.", self.res(r).name)),
            },
            BK::Civic => self.go(Scene::Room(b)),
            BK::Museum => self.go(Scene::Room(b)),
        }
    }
    pub fn talk(&mut self, i: usize) {
        if self.social_talk(i) {
            return;
        }
        let r = &self.res(i);
        let pers = r.pers;
        let hr = self.hr;
        let slot = if !(5..21).contains(&hr) {
            3
        } else if hr < 12 {
            0
        } else if hr < 17 {
            1
        } else {
            2
        };
        let fruit = def(self.w.native).name.to_lowercase();
        let fruits = plural(&fruit);
        let name = self.w.p.name.clone();
        let fill = |s: &str, item: &str| {
            s.replace("{p}", &name).replace("{fs}", &fruits).replace("{f}", &fruit).replace("{i}", item)
        };
        let indoor = self.me.scene != Scene::Town && !self.visit.is_some_and(|v| v.0 == i);
        let visiting = self.visit.is_some_and(|v| v.0 == i);
        let mut lines = vec![fill(
            if visiting {
                VISIT[pers]
            } else if indoor {
                HOMEL[pers]
            } else {
                GREET[pers][slot]
            },
            "",
        )];
        if self.w.moving.get(i).is_some_and(|&m| m != 0) && !visiting {
            lines.push("I'm planning to move away soon... unless someone gives me a reason to stay.".into())
        }
        if let Some(p) = &self.w.project {
            if p.by == i {
                lines.push(format!("How's our {} coming along? {}", PROJECTS[p.kind as usize].0, Game::remaining(p)))
            }
        }
        if let Some(b) = self.w.built.iter().rev().find(|b| self.w.day - b.day <= 2) {
            lines.push(format!(
                "Have you seen the new {}? The town feels {} already!",
                PROJECTS[b.kind as usize].0,
                Game::live_word(self.liveliness()).to_lowercase()
            ))
        }
        if self.fair() {
            lines.push("The Harvest Fair is on! There's a prize stall by the plaza.".into())
        }
        if let Some(f) = self.fest() {
            lines.push(FESTS[f].chat.into())
        }
        match weather(self.w.day, self.mo) {
            Wx::Rain if !indoor => lines.push(fill(RAIN[pers], "")),
            Wx::Snow if !indoor => lines.push(fill(SNOW[pers], "")),
            _ => {}
        }
        let ch = self.w.roster[i];
        if (self.mo, self.dom) == BDAYS[ch] {
            lines.push(format!("Guess what, {name}? It's my birthday today!"))
        }
        if self.w.p.lost_owner == Some(i) && self.remove_one(150) {
            self.w.p.lost_owner = None;
            self.w.p.npcs[i].friend += 4;
            self.w.p.money += RETURN_COINS;
            self.npcs[i].happy = 2.5;
            self.snd.push(Snd::Coin);
            lines.push(format!(
                "My scarf! I looked everywhere for it! Thank you so much, {name}. Please take {RETURN_COINS} coins!"
            ))
        }
        if let Some((from, to)) = self.w.p.parcel {
            if to == i && self.w.p.inv.iter().any(|s| s.def == 86) {
                self.remove_one(86);
                self.w.p.parcel = None;
                self.w.p.money += 300;
                self.w.p.npcs[i].friend += 2;
                self.w.p.npcs[from].friend += 2;
                self.npcs[i].happy = 2.;
                self.snd.push(Snd::Coin);
                lines.push(format!(
                    "A parcel from {}? How thoughtful! Here, take 300 coins for your trouble.",
                    self.res(from).name
                ));
            }
        }
        if self.w.p.npcs[i].req == Some(86) {
            if self.w.p.parcel.is_none() && self.w.p.inv.len() < INV_CAP {
                let to = (i + 1 + gen_range(0, 3)) % 4;
                self.add(86);
                self.w.p.parcel = Some((i, to));
                self.w.p.npcs[i].req = None;
                lines.push(format!(
                    "Could you take this parcel to {} for me? I'd really appreciate it!",
                    self.res(to).name
                ))
            } else {
                self.w.p.npcs[i].req = None
            }
        }
        if let Some(q) = self.w.p.npcs[i].req.filter(|&q| q != 86) {
            let item = def(q).name;
            if self.remove_one(q) {
                lines.push(fill(THANKS[pers], item));
                let s = &mut self.w.p.npcs[i];
                s.req = None;
                s.friend += 3;
                let fr = s.friend;
                self.npcs[i].happy = 2.5;
                let hp = if indoor { res_spot() } else { self.npcs[i].pos };
                for k in 0..6 {
                    self.hearts.push((vec3(hp.x + (k as f32 - 2.5) * 0.12, 1.3, hp.y), -(k as f32) * 0.12))
                }
                if fr >= 6 && self.w.p.inv.len() < INV_CAP {
                    let f = FURN_IDS[gen_range(0, 8)];
                    self.add_furn(f);
                    self.note(format!("{} gave you a {}!", r.name, def(f).name))
                } else {
                    let m = def(q).sell * 2;
                    self.w.p.money += m;
                    self.note(format!("{} gave you {m} coins.", r.name))
                }
                self.snd.push(Snd::Coin);
            } else {
                lines.push(fill(ASK[pers], item))
            }
        } else if self.w.p.npcs[i].talked {
            lines.push(AGAIN[pers].to_string())
        } else {
            let mut k = gen_range(0, 4);
            if k == self.npcs[i].last_chat {
                k = (k + 1) % 4
            }
            self.npcs[i].last_chat = k;
            lines.push(fill(CHAT[pers][k], ""));
            let fr = self.w.p.npcs[i].friend;
            lines.push(fill(TIER_LINES[pers][tier(fr)], ""));
            let mut ctx = self.ctx_lines(i);
            for _ in 0..2 {
                if ctx.is_empty() {
                    break;
                }
                let c = ctx.remove(gen_range(0, ctx.len()));
                lines.push(fill(&c, ""))
            }
            let ch = self.w.roster[i];
            if tier(fr) == 4 && !self.w.p.keepsakes.contains(&(ch as u8)) && self.add_furn(KEEPSAKES[ch]) {
                self.w.p.keepsakes.push(ch as u8);
                self.npcs[i].happy = 2.5;
                self.snd.push(Snd::Coin);
                lines.push(format!(
                    "{name}... you're my best friend. I want you to have this portrait of me. Hang it somewhere nice!"
                ))
            }
        }
        if !self.w.p.npcs[i].talked {
            self.w.p.npcs[i].talked = true;
            self.w.p.npcs[i].friend += 1;
            self.ev(Ev::Talk, 1)
        } // temporary prototype relationship rule
        self.me.ui = Ui::Talk { npc: Some(i), who: format!("{} the {}", r.name, r.species), lines, i: 0 };
    }
    /// Contextual line candidates for resident slot `i` (season, the player's progress, town state, other residents, hobby).
    pub fn ctx_lines(&self, i: usize) -> Vec<String> {
        let pers = self.res(i).pers;
        let ch = self.w.roster[i];
        let p = &self.w.p;
        let mut v: Vec<String> = vec![];
        let season = match self.mo {
            3..=5 => 0,
            6..=8 => 1,
            9..=11 => 2,
            _ => 3,
        };
        v.push(SEASON_LINES[pers][season].into());
        v.push(CHAR_LINES[ch][gen_range(0, 3)].into());
        v.push(format!("Did you know I love {}? It's my favorite thing to do.", HOBBY[ch]));
        let o = (i + 1 + gen_range(0, 3)) % 4;
        v.push(GOSSIP[pers][gen_range(0, 2)].replace("{o}", self.res(o).name));
        if p.home > 0 {
            v.push(
                [
                    "Your house got bigger! You'll have to show me around!",
                    "Bigger house, huh? More room for clutter.",
                    "Your home grew, like a mushroom after rain.",
                    "Expanded your house? That's a power move!",
                ][pers]
                    .into(),
            )
        }
        if p.debt > 0 {
            v.push(
                [
                    "Paying off your home bit by bit? You've got this!",
                    "A balance on your house? Pay it off when you can. No rush. I'm not your mother.",
                    "Coins come and go like tides. Your home stays.",
                    "Chipping away at that home balance? Reps, {p}! Reps!",
                ][pers]
                    .into(),
            )
        }
        if self.w.donated.len() >= 10 {
            v.push(format!("The museum has {} exhibits now. The professor must be thrilled!", self.w.donated.len()))
        }
        if self.camp_open() {
            let vv = &VISITORS[self.visitor()];
            v.push(format!("Have you met {} at the campground? A {} with lots of stories!", vv.name, vv.species))
        }
        if let Some(pol) = self.pol() {
            v.push(format!("With {} in effect, the town feels different. I'm still deciding if I like it!", pol.name))
        }
        if self.w.built.len() >= 3 {
            v.push("Our town has so many new things now. It feels like home.".into())
        }
        if let Some(sp) = &self.w.surprise {
            let where_ = if sp.x < 20 { "west" } else { "east" };
            v.push(match sp.kind {
                0 => format!("I saw something glinting on the beach, over on the {where_} side."),
                1 => format!("Someone dropped a scarf somewhere on the {where_} side of town. I hope they find it!"),
                _ => format!("A balloon came down somewhere on the {where_} side of town! I wonder what it carried."),
            })
        }
        v
    }
    #[cfg(test)]
    pub fn shake_tree_pub(&mut self, ti: usize) {
        self.shake_tree(ti)
    }
    #[cfg(test)]
    pub fn reel_pub(&mut self) {
        self.reel()
    }
    fn shake_tree(&mut self, ti: usize) {
        self.shake.push((ti, 0.6));
        self.snd.push(Snd::Rustle);
        let tr = self.w.trees[ti];
        if !self.w.branches.contains(&(tr.x, tr.z)) {
            if let Some((x, z)) = [(0, 1), (1, 0), (-1, 0), (0, -1), (1, 1), (-1, 1)]
                .iter()
                .map(|&(dx, dz)| (tr.x + dx, tr.z + dz))
                .find(|&(x, z)| self.walk_tile(x, z) && !self.item_at(x, z))
            {
                self.w.branches.push((tr.x, tr.z));
                self.drop_item(80, x, z)
            }
        }
        if !tr.fruit {
            return self.note("Rustle rustle... no fruit right now.");
        }
        self.w.trees[ti].fruit = false;
        self.w.trees[ti].shaken = self.w.day;
        let mut n = 0;
        for (dx, dz) in [(0, 1), (-1, 1), (1, 1), (-1, 0), (1, 0), (0, -1), (-1, -1), (1, -1)] {
            if n == 3 {
                break;
            }
            let (x, z) = (tr.x + dx, tr.z + dz);
            if self.walk_tile(x, z) && !self.item_at(x, z) {
                self.drop_item(self.w.native, x, z);
                n += 1
            }
        }
        let nm = plural(&def(self.w.native).name.to_lowercase());
        self.note(format!("{n} {nm} fell from the tree!"));
        if n > 0 {
            self.ev(Ev::Fruit, 1)
        }
    }
    pub fn add(&mut self, d: u16) -> bool {
        if let Some(s) = self.w.p.inv.iter_mut().find(|s| s.def == d && s.uid == 0 && s.size_mm == 0 && s.qty < STACK) {
            s.qty += 1
        } else if self.w.p.inv.len() < INV_CAP {
            self.w.p.inv.push(Slot { size_mm: 0, def: d, qty: 1, uid: 0 })
        } else {
            self.note("Your pockets are full!");
            return false;
        }
        if !self.w.p.seen.contains(&d) {
            self.w.p.seen.push(d)
        }
        self.note(format!("Got: {}!", def(d).name));
        true
    }
    pub fn add_furn(&mut self, d: u16) -> bool {
        if self.w.p.inv.len() >= INV_CAP {
            self.note("Your pockets are full!");
            return false;
        }
        let uid = self.uid();
        self.w.p.inv.push(Slot { size_mm: 0, def: d, qty: 1, uid });
        if !self.w.p.catalog.contains(&d) {
            self.w.p.catalog.push(d)
        }
        true
    }
    /// Re-order a catalog item at full shop price (delivered to pockets immediately; provisional).
    pub fn order(&mut self, d: u16) -> Result<(), &'static str> {
        if !self.w.p.catalog.contains(&d) {
            return Err("That isn't in your catalog.");
        }
        let p = def(d).buy;
        if self.w.p.money < p {
            return Err("Not enough coins.");
        }
        if self.w.p.inv.len() >= INV_CAP {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= p;
        self.add_furn(d);
        self.w.spent += p;
        Ok(())
    }
    pub fn remove_one(&mut self, d: u16) -> bool {
        let Some(i) = self.w.p.inv.iter().position(|s| s.def == d && s.uid == 0) else { return false };
        self.w.p.inv[i].qty -= 1;
        if self.w.p.inv[i].qty == 0 {
            self.w.p.inv.remove(i);
        }
        true
    }

    /// Pay as much of the expansion balance as current coins allow.
    pub fn pay_debt(&mut self) {
        let p = self.w.p.money.min(self.w.p.debt);
        self.w.p.money -= p;
        self.w.p.debt -= p;
        self.snd.push(Snd::Coin);
        self.note(if self.w.p.debt == 0 {
            format!("Paid {p} coins. Your balance is cleared!")
        } else {
            format!("Paid {p} coins. {} left.", self.w.p.debt)
        })
    }
    /// Interior size of a building; homes grow with their owner's house level.
    pub fn rdims(&self, b: usize) -> (i32, i32) {
        if let BK::Home(o) = BLDS[b].kind {
            let lv = if self.w.p.id == o {
                self.w.p.home
            } else {
                self.w.others.iter().find(|p| p.id == o).map_or(0, |p| p.home)
            };
            HOME_DIMS[(lv as usize).min(HOME_DIMS.len() - 1)]
        } else {
            room_dims(b)
        }
    }
    /// Pay up front to enlarge the active player's home. Furniture keeps its tiles; anything on the new doorway goes to home storage.
    pub fn expand_home(&mut self) -> Result<(), &'static str> {
        let lv = self.w.p.home as usize;
        if lv >= EXPAND_COST.len() {
            return Err("Your home is already as big as it gets!");
        }
        if self.w.p.debt > 0 {
            return Err("Pay off your current balance first.");
        }
        let cost = EXPAND_COST[lv];
        let (w, d) = HOME_DIMS[lv + 1];
        let mut clear = vec![(w / 2, d - 1)];
        for (t, _) in stairs(lv as u32 + 1, 0) {
            clear.push(t);
            clear.push((t.0, t.1 + 1))
        }
        let hit = |f: &Furn| {
            if def(f.def).wall {
                lv + 1 >= 4 && f.x == STAIR_UP.0
            } else {
                foot(f.def, f.x, f.z, f.rot).iter().any(|t| clear.contains(t))
            }
        };
        self.set_story(0);
        if self.w.p.storage.len() + self.w.p.room.iter().filter(|f| hit(f)).count() > STORE_CAP {
            return Err("Clear some home storage first.");
        }
        self.w.p.debt = cost;
        self.w.p.home += 1;
        let (moved, keep): (Vec<Furn>, Vec<Furn>) = self.w.p.room.iter().copied().partition(|f| hit(f));
        self.w.p.room = keep;
        for f in &moved {
            self.w.p.storage.push(Slot { size_mm: 0, def: f.def, qty: 1, uid: f.uid })
        }
        if !moved.is_empty() {
            self.note("Furniture in the way of the new doorway or stairs was moved to your storage.")
        }
        Ok(())
    }

    /// Open letter `i` (newest first in the menu): marks it read and claims its gift once.
    pub fn open_letter(&mut self, i: usize) {
        let n = self.w.p.mail.len();
        if i >= n {
            return;
        }
        let k = n - 1 - i;
        self.w.p.mail[k].read = true;
        let l = self.w.p.mail[k].clone();
        if l.claimed {
            return;
        }
        if let Some(d) = l.item {
            if self.w.p.inv.len() >= INV_CAP {
                return self.note("Your pockets are full; the gift stays in the letter.");
            }
            if def(d).k == K::Furn || def(d).k == K::Cloth {
                self.add_furn(d);
            } else {
                self.add(d);
            }
        }
        if l.coins > 0 {
            self.w.p.money += l.coins;
            self.note(format!("{} coins were enclosed.", l.coins));
            self.snd.push(Snd::Coin)
        }
        self.w.p.mail[k].claimed = true;
    }

    /// Give one unit of pocket slot `slot` to resident `j` (once per resident per day per player).
    pub fn give_gift(&mut self, j: usize, slot: usize) {
        let before = self.w.p.npcs[j].gifted;
        self.give_gift_inner(j, slot);
        if !before && self.w.p.npcs[j].gifted {
            self.ev(Ev::Gift, 1)
        }
    }
    fn give_gift_inner(&mut self, j: usize, slot: usize) {
        let r = &self.res(j);
        let who = format!("{} the {}", r.name, r.species);
        if self.w.p.npcs[j].gifted {
            self.me.ui = Ui::Talk {
                npc: Some(j),
                who,
                lines: vec!["You already gave me something today. Save it for tomorrow!".into()],
                i: 0,
            };
            return;
        }
        let Some(sl) = self.w.p.inv.get(slot).copied() else { return };
        self.w.p.inv[slot].qty -= 1;
        if self.w.p.inv[slot].qty == 0 {
            self.w.p.inv.remove(slot);
        }
        let bday = (self.mo, self.dom) == BDAYS[self.w.roster[j]];
        let liked = def(sl.def).k == LIKES[r.pers] || bday;
        let s = &mut self.w.p.npcs[j];
        s.gifted = true;
        s.friend += if liked { 2 } else { 1 } + if bday { 3 } else { 0 };
        let name = def(sl.def).name;
        let line = if liked {
            self.npcs[j].happy = 2.;
            let hp = if self.me.scene == Scene::Town { self.npcs[j].pos } else { res_spot() };
            for k in 0..4 {
                self.hearts.push((vec3(hp.x + (k as f32 - 1.5) * 0.12, 1.3, hp.y), -(k as f32) * 0.12))
            }
            if bday {
                format!("A {name} for my birthday?! You remembered! Thank you so much!")
            } else {
                format!("A {name}! Oh, I really love these. Thank you!")
            }
        } else {
            format!("A {name}? How kind. Thank you!")
        };
        self.snd.push(Snd::Blip);
        self.me.ui = Ui::Talk { npc: Some(j), who, lines: vec![line], i: 0 };
    }

    /// Harvest Fair: one free furniture prize per player per year (needs a free pocket slot; nothing is lost if full).
    // ---- Campground ----
    pub fn visitor(&self) -> usize {
        visitor_of(self.w.day)
    }
    pub fn camp_open(&self) -> bool {
        (CAMP_HOURS.0..CAMP_HOURS.1).contains(&self.hr)
    }
    /// Buy good `k` from today's visitor (one unit of each per day, shared between players).
    pub fn camp_buy(&mut self, k: usize) -> Result<&'static str, &'static str> {
        if !self.camp_open() {
            return Err("The camper is asleep in the tent.");
        }
        let d = VISITORS[self.visitor()].goods[k];
        if self.w.camp_sold.contains(&d) {
            return Err("Sold out for today.");
        }
        let p = def(d).buy;
        if self.w.p.money < p {
            return Err("Not enough coins.");
        }
        if !self.add_furn(d) {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= p;
        self.w.camp_sold.push(d);
        self.ev(Ev::Camp, 1);
        Ok(def(d).name)
    }
    pub fn favor_item(&self) -> (u16, u16) {
        let f = VISITORS[self.visitor()].favor;
        (if f.0 == 0 { self.w.native } else { f.0 }, f.1)
    }
    /// Hand over the favor items for today's visitor's keepsake (once per player per visitor).
    pub fn camp_favor(&mut self) -> Result<&'static str, String> {
        let v = self.visitor();
        if self.w.p.camp_favors.contains(&(v as u8)) {
            return Err("You've already helped me. Thank you again!".into());
        }
        let (d, n) = self.favor_item();
        let have: u16 = self.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum();
        if have < n {
            return Err(format!("{} (You have {have} of {n} {}.)", VISITORS[v].ask, def(d).name));
        }
        let backup = self.w.p.inv.clone();
        for _ in 0..n {
            self.remove_one(d);
        }
        if !self.add_furn(VISITORS[v].reward) {
            self.w.p.inv = backup;
            return Err("Your pockets are full. Make room for my thank-you gift!".into());
        }
        self.w.p.camp_favors.push(v as u8);
        self.ev(Ev::Camp, 1);
        Ok(def(VISITORS[v].reward).name)
    }
    // ---- Initiatives ----
    /// Can template `t` be completed today by the active player? (tools owned, resources present, places open)
    pub fn task_ok(&self, t: usize) -> bool {
        let tools = &self.w.p.tools;
        let money = self.w.p.money;
        match TASKS[t].0 {
            Ev::Fish => tools.contains(&61),
            Ev::Dig => tools.contains(&62) && self.w.digs.len() >= 2,
            Ev::Rock => tools.contains(&62),
            Ev::Weed => self.w.weeds.len() >= 4,
            Ev::Fruit => self.w.trees.iter().filter(|t| t.fruit).count() >= 3,
            Ev::Buy => money >= 600,
            Ev::Activity => money >= FARE + 100 || self.w.p.ferry > 0,
            Ev::Gift => true,
            Ev::Bug => tools.contains(&60),
            Ev::Water => tools.contains(&CAN) && self.w.garden.len() >= 3,
            _ => true,
        }
    }
    /// Give the active player three attainable tasks for today (once per day).
    pub fn assign_tasks(&mut self) {
        if self.w.p.task_day == self.w.day {
            return;
        }
        let mut pool: Vec<usize> = (0..TASKS.len()).filter(|&t| self.task_ok(t)).collect();
        let mut v = vec![];
        while v.len() < 3 && !pool.is_empty() {
            let t = pool.remove(gen_range(0, pool.len()));
            v.push(PTask { t, prog: 0, claimed: false })
        }
        self.w.p.tasks = v;
        self.w.p.task_day = self.w.day;
    }
    pub fn assign_weekly(&mut self) {
        let wk = week_of(self.w.day);
        if self.w.wtask.as_ref().map_or(true, |w| w.week != wk) {
            self.w.wtask =
                Some(WTask { t: (wk.rem_euclid(WTASKS.len() as i64)) as usize, prog: 0, week: wk, claimed: vec![] })
        }
    }
    /// Count progress for the active player's tasks and the shared weekly task.
    pub fn ev(&mut self, e: Ev, n: u32) {
        let day = self.w.day;
        if self.w.p.task_day == day {
            for t in &mut self.w.p.tasks {
                if TASKS[t.t].0 == e {
                    t.prog = (t.prog + n).min(TASKS[t.t].1)
                }
            }
        }
        if let Some(w) = &mut self.w.wtask {
            if WTASKS[w.t].0 == e {
                w.prog = (w.prog + n).min(WTASKS[w.t].1)
            }
        }
    }
    /// Claim a finished daily task (index 0-2) or the weekly task (index 3). One time only.
    pub fn claim(&mut self, k: usize) -> Result<u32, &'static str> {
        if k < 3 {
            let Some(t) = self.w.p.tasks.get_mut(k) else { return Err("No task there.") };
            if t.claimed {
                return Err("Already claimed.");
            }
            if t.prog < TASKS[t.t].1 {
                return Err("Not finished yet.");
            }
            t.claimed = true;
            let c = TASKS[t.t].3;
            self.w.p.coupons += c;
            return Ok(c);
        }
        let id = self.w.p.id;
        let Some(w) = self.w.wtask.as_mut() else { return Err("No weekly task.") };
        if w.claimed.contains(&id) {
            return Err("Already claimed.");
        }
        if w.prog < WTASKS[w.t].1 {
            return Err("The town hasn't finished it yet.");
        }
        w.claimed.push(id);
        self.w.p.coupons += WEEKLY_COUPONS;
        Ok(WEEKLY_COUPONS)
    }
    /// Trade coupons for kiosk reward `k`. Nothing is deducted unless the reward fits in pockets.
    pub fn kiosk_buy(&mut self, k: usize) -> Result<&'static str, &'static str> {
        let (d, c) = KIOSK_ITEMS[k];
        if self.w.p.coupons < c {
            return Err("Not enough coupons.");
        }
        match d {
            136 => self.w.p.ferry += 1,
            137 => {
                let room =
                    self.w.p.inv.iter().any(|s| s.def == 85 && s.qty + 3 <= STACK) || self.w.p.inv.len() < INV_CAP;
                if !room {
                    return Err("Your pockets are full.");
                }
                for _ in 0..3 {
                    self.add(85);
                }
            }
            _ => {
                if !self.add_furn(d) {
                    return Err("Your pockets are full.");
                }
            }
        }
        self.w.p.coupons -= c;
        Ok(def(d).name)
    }
    pub fn art_open(&self) -> bool {
        weekday(self.w.day) == 5 && !self.fair() && self.fest().is_none()
    }
    /// Place today's surprise (shared; first finder gets it).
    pub fn new_surprise(&mut self) {
        let kind = gen_range(0u8, 3);
        let t = if kind == 0 { self.rand_tile(SAND) } else { self.rand_tile(GRASS) };
        self.w.surprise = t.map(|t| Surprise { kind, x: t.0, z: t.1, owner: gen_range(0, 4), day: self.w.day });
    }
    pub fn take_surprise(&mut self) {
        let Some(sp) = self.w.surprise.clone() else { return };
        match sp.kind {
            0 => {
                let note = BOTTLE_NOTES[(sp.day.rem_euclid(BOTTLE_NOTES.len() as i64)) as usize];
                self.w.p.mail.push(Letter {
                    from: "A message bottle".into(),
                    text: format!("{note} (Tucked inside: {BOTTLE_COINS} coins.)"),
                    coins: BOTTLE_COINS,
                    item: None,
                    read: false,
                    claimed: false,
                });
                self.note("You found a message bottle! The note and coins are waiting in your mailbox.")
            }
            1 => {
                if self.w.p.lost_owner.is_some() {
                    return self.note("You're already carrying someone's lost scarf.");
                }
                if !self.add(150) {
                    return;
                }
                self.w.p.lost_owner = Some(sp.owner);
                let o = self.res(sp.owner).name;
                self.note(format!("A lost scarf with {o}'s name stitched on it. {o} will be glad to get it back!"))
            }
            _ => {
                let d = FURN_IDS[gen_range(0, FURN_IDS.len())];
                if !self.add_furn(d) {
                    return;
                }
                self.note(format!("A gift drifted down on a balloon! Inside: a {}.", def(d).name))
            }
        }
        self.snd.push(Snd::Coin);
        self.w.surprise = None;
    }
    // ---- Home evaluation ----
    /// Score of a profile's home with up to two improvement tips.
    pub fn home_score(p: &Profile) -> (u32, Vec<&'static str>) {
        let all: Vec<Furn> = p.room.iter().chain(p.stories.iter().flatten()).copied().collect();
        let room = &all;
        let n = room.len().min(30) as u32;
        let mut ids: Vec<u16> = room.iter().map(|f| f.def).collect();
        ids.sort();
        ids.dedup();
        let sets = SETS.iter().filter(|s| s.1.iter().all(|d| ids.contains(d))).count() as u32;
        let lamp = room.iter().any(|f| def(f.def).act == Act::Lamp);
        let walls = room.iter().filter(|f| def(f.def).wall).count().min(3) as u32;
        let decor = p.wall != 0 || p.floor != 0;
        let keeps = room.iter().filter(|f| KEEPSAKES.contains(&f.def)).count() as u32;
        let score = n * 10
            + ids.len() as u32 * 5
            + sets * 60
            + if lamp { 15 } else { 0 }
            + walls * 10
            + if decor { 20 } else { 0 }
            + p.home as u32 * 40
            + keeps * 15;
        let mut tips = vec![];
        if !lamp {
            tips.push("A lamp would make the room glow in the evening.")
        }
        if walls == 0 {
            tips.push("The walls look bare. Try hanging a painting or a portrait.")
        }
        if !decor {
            tips.push("New wallpaper or flooring would give the room character.")
        }
        if sets == 0 {
            tips.push("Completing a furniture collection impresses every judge.")
        }
        if ids.len() < 6 {
            tips.push("A little more variety in your furniture would help.")
        }
        if p.home == 0 {
            tips.push("A bigger home leaves room for more ideas.")
        }
        tips.truncate(2);
        (score, tips)
    }
    pub fn home_tier(score: u32) -> usize {
        HOME_TIERS.iter().rposition(|t| score >= t.0).unwrap_or(0)
    }
    /// Monday letter from the Home Society with the score, tips and a one-time reward for each newly reached rating.
    fn home_letter(&mut self) {
        let (sc, tips) = Self::home_score(&self.w.p);
        let t = Self::home_tier(sc);
        let mut text = format!("This week your home scored {sc} points: {}! ", HOME_TIERS[t].1);
        for tip in &tips {
            text.push_str(tip);
            text.push(' ')
        }
        let (mut coins, mut item) = (0, None);
        if t as u8 > self.w.p.home_best {
            for k in (self.w.p.home_best as usize + 1)..=t {
                coins += HOME_TIERS[k].2
            }
            if t == HOME_TIERS.len() - 1 {
                item = Some(169)
            }
            self.w.p.home_best = t as u8;
            text.push_str(&format!("A new rating! Please accept {coins} coins from the Society."))
        }
        self.w.p.mail.push(Letter {
            from: "Hollowbrook Home Society".into(),
            text,
            coins,
            item,
            read: false,
            claimed: false,
        });
    }
    pub fn repaint(&mut self, k: usize) -> Result<(), &'static str> {
        if self.w.p.money < PAINT_FEE {
            return Err("Not enough coins for the paint.");
        }
        if self.w.p.paint == Some(k as u8) {
            return Err("Your house is already that colour.");
        }
        self.w.p.money -= PAINT_FEE;
        self.w.p.paint = Some(k as u8);
        Ok(())
    }
    // ---- Gardening ----
    pub fn plant_ok(&self, t: (i32, i32)) -> bool {
        self.me.scene == Scene::Town
            && tile_at(&self.map, t.0, t.1) == GRASS
            && self.walk_tile(t.0, t.1)
            && !self.occupied(t.0, t.1)
            && t != tile_of(self.me.pos)
            && ![mailbox_tile(0), mailbox_tile(1)].contains(&t)
    }
    pub fn plant(&mut self, t: (i32, i32)) -> Result<(), &'static str> {
        if !self.plant_ok(t) {
            return Err("You can't plant there. Try open grass.");
        }
        let Some(k) = SEEDS.iter().position(|&s| self.w.p.inv.iter().any(|x| x.def == s)) else {
            return Err("You have no seeds.");
        };
        self.remove_one(SEEDS[k]);
        self.w.garden.push(Plant { x: t.0, z: t.1, kind: k as u8, stage: 0, watered: i64::MIN });
        self.snd.push(Snd::Dig);
        self.note(format!("You planted {}. Water it every day!", def(SEEDS[k]).name));
        Ok(())
    }
    /// Water plants in front (area grows with the can's level). Returns how many were watered.
    pub fn water(&mut self) -> usize {
        let lv = self.tool_level(CAN);
        let f = tile_of(self.me.pos + self.me.face * 0.9);
        let (fx, fz) = (self.me.face.x.round() as i32, self.me.face.y.round() as i32);
        let mut tiles = vec![f];
        if lv >= 1 {
            tiles.push((f.0 + fz, f.1 + fx));
            tiles.push((f.0 - fz, f.1 - fx));
        }
        if lv >= 2 {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    tiles.push((f.0 + dx, f.1 + dz))
                }
            }
        }
        let day = self.w.day;
        let mut n = 0;
        for p in &mut self.w.garden {
            if tiles.contains(&(p.x, p.z)) && p.watered != day {
                p.watered = day;
                n += 1
            }
        }
        let fp = tc(f);
        self.fx.push((vec3(fp.x, 0.05, fp.y), 0., 0));
        self.snd.push(Snd::Splash);
        if n > 0 {
            self.ev(Ev::Water, n as u32);
            self.note(format!("Watered {n} flower{}.", if n == 1 { "" } else { "s" }))
        } else {
            self.note("Nothing here needs water.")
        }
        n
    }
    pub fn pick_flower(&mut self, i: usize) {
        let p = self.w.garden[i];
        if p.stage < 2 {
            return self.note(if p.watered == self.w.day {
                "It's growing. Come back tomorrow!"
            } else {
                "It needs water today to keep growing."
            });
        }
        if self.add(FLOWERS[p.kind as usize]) {
            self.w.garden[i].stage = 1;
            self.note(format!("You picked a {}. It will bloom again soon.", def(FLOWERS[p.kind as usize]).name))
        }
    }
    /// Daily growth (watered yesterday or rain) and cross-breeding of neighbouring blooms of different colours.
    pub fn grow_garden(&mut self) {
        let prev = self.w.day - 1;
        let rain = rainy(prev);
        for p in &mut self.w.garden {
            if p.watered >= prev || rain {
                p.stage = (p.stage + 1).min(2)
            }
        }
        let blooms: Vec<Plant> = self.w.garden.iter().copied().filter(|p| p.stage == 2 && p.kind < 4).collect();
        'outer: for a in &blooms {
            for b in &blooms {
                if (a.x - b.x).abs() + (a.z - b.z).abs() == 1 && a.kind < b.kind && gen_range(0, CROSS_CHANCE) == 0 {
                    for (dx, dz) in [(0, 1), (1, 0), (0, -1), (-1, 0), (1, 1), (-1, -1)] {
                        let t = (a.x + dx, a.z + dz);
                        if tile_at(&self.map, t.0, t.1) == GRASS && self.walk_tile(t.0, t.1) && !self.occupied(t.0, t.1)
                        {
                            self.w.garden.push(Plant { x: t.0, z: t.1, kind: 4, stage: 0, watered: i64::MIN });
                            self.mail_all("Town notice board", "A strange new sprout appeared in the town garden! Some say crossed flowers make something rare.");
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    // ---- Tool upgrades ----
    pub fn tool_level(&self, t: u16) -> u8 {
        UPGRADABLE.iter().position(|&u| u == t).and_then(|k| self.w.p.tool_lv.get(k).copied()).unwrap_or(0)
    }
    pub fn upgrade_tool(&mut self, t: u16) -> Result<u8, String> {
        let Some(k) = UPGRADABLE.iter().position(|&u| u == t) else {
            return Err("That tool can't be upgraded.".into());
        };
        if !self.w.p.tools.contains(&t) {
            return Err(format!("You need a {} first.", def(t).name));
        }
        let lv = self.tool_level(t);
        if lv >= 2 {
            return Err("That's already a masterwork!".into());
        }
        let (c, st, gl) = UPGRADE[lv as usize];
        let have = |g: &Game, d: u16| g.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum::<u16>();
        if self.w.p.money < c || have(self, 81) < st || have(self, 85) < gl {
            return Err(format!(
                "I need {c} coins, {st} Stone{} to do that.",
                if gl > 0 { format!(" and {gl} Sea Glass") } else { String::new() }
            ));
        }
        self.w.p.money -= c;
        for _ in 0..st {
            self.remove_one(81);
        }
        for _ in 0..gl {
            self.remove_one(85);
        }
        while self.w.p.tool_lv.len() < UPGRADABLE.len() {
            self.w.p.tool_lv.push(0)
        }
        self.w.p.tool_lv[k] = lv + 1;
        Ok(lv + 1)
    }
    pub fn keeper_items(&self) -> Vec<(String, u8)> {
        // (label, action) 0 chat, 1-4 seed, 5 can, 10+k upgrade tool k
        let mut v = vec![("Chat".to_string(), 0)];
        for (k, &s) in SEEDS.iter().enumerate() {
            v.push((format!("{}  -  {SEED_PRICE} coins", def(s).name), 1 + k as u8))
        }
        if !self.w.p.tools.contains(&CAN) {
            v.push((format!("Watering Can  -  {} coins", def(CAN).buy), 5))
        }
        for (k, &t) in UPGRADABLE.iter().enumerate() {
            if self.w.p.tools.contains(&t) {
                let lv = self.tool_level(t);
                if lv < 2 {
                    let (c, st, gl) = UPGRADE[lv as usize];
                    v.push((
                        format!(
                            "Upgrade {} -> {}{}  -  {c}c, {st} Stone{}",
                            def(t).name,
                            LV_NAME[lv as usize + 1],
                            def(t).name,
                            if gl > 0 { format!(", {gl} Sea Glass") } else { String::new() }
                        ),
                        10 + k as u8,
                    ))
                }
            }
        }
        v
    }
    pub fn fest(&self) -> Option<usize> {
        fest_of(self.mo, self.dom)
    }
    /// Festival host: explains the event and gives the yearly keepsake once the task is done (once per player per year).
    pub fn fest_talk(&mut self) {
        let Some(f) = self.fest() else { return };
        let fe = &FESTS[f];
        let mut lines = vec![format!("Welcome to the {}! {}", fe.name, fe.about)];
        match self.fest_claim() {
            Ok(n) => {
                self.snd.push(Snd::Coin);
                lines.push(format!("Wonderful, thank you! Here is a {n} to remember this year's {}.", fe.name))
            }
            Err(e) => lines.push(e),
        }
        self.me.ui = Ui::Talk { npc: None, who: "Hazel Quill (festival host)".into(), lines, i: 0 }
    }
    pub fn fest_claim(&mut self) -> Result<&'static str, String> {
        let Some(f) = self.fest() else { return Err("No festival today.".into()) };
        let fe = &FESTS[f];
        if self.w.p.fest_years.contains(&(f as u8, self.yr)) {
            return Err(format!("You've already joined this year's {}. Enjoy the festivities!", fe.name));
        }
        let backup = self.w.p.inv.clone();
        let ok = match fe.task {
            FTask::Fruit(n) => {
                let d = self.w.native;
                let have: u16 = self.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum();
                have >= n && {
                    for _ in 0..n {
                        self.remove_one(d);
                    }
                    true
                }
            }
            FTask::Branch(n) => {
                let have: u16 = self.w.p.inv.iter().filter(|s| s.def == 80).map(|s| s.qty).sum();
                have >= n && {
                    for _ in 0..n {
                        self.remove_one(80);
                    }
                    true
                }
            }
            FTask::Fish(n) => {
                let fish: Vec<u16> = self.w.p.inv.iter().filter(|s| def(s.def).k == K::Fish).map(|s| s.def).collect();
                fish.len() as u16 >= n && {
                    for d in fish.into_iter().take(n as usize) {
                        self.remove_one(d);
                    }
                    true
                }
            }
            FTask::TalkAll => self.w.p.npcs.iter().all(|n| n.talked),
        };
        if !ok {
            return Err(format!("Could you help? {}", fe.ask));
        }
        if !self.add_furn(fe.reward) {
            self.w.p.inv = backup;
            return Err("Your pockets are full. Make room for your keepsake!".into());
        }
        self.w.p.fest_years.push((f as u8, self.yr));
        Ok(def(fe.reward).name)
    }
    /// Buy this week's painting (one per player per week, from your own coins).
    pub fn art_buy(&mut self) -> Result<&'static str, &'static str> {
        if !self.art_open() {
            return Err("The art stall only opens on Saturdays.");
        }
        let wk = (self.w.day - 1).div_euclid(7);
        if self.w.p.art_week == wk {
            return Err("You've already bought this week's painting.");
        }
        if self.w.p.money < ART_PRICE {
            return Err("Not enough coins.");
        }
        let d = art_of_week(self.w.day);
        if !self.add(d) {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= ART_PRICE;
        self.w.p.art_week = wk;
        Ok(def(d).name)
    }
    /// Note when a furniture collection is complete in the active player's home.
    pub fn sets_done(&self) -> Vec<&'static str> {
        SETS.iter().filter(|s| s.1.iter().all(|d| self.w.p.room.iter().any(|f| f.def == *d))).map(|s| s.0).collect()
    }
    pub fn fair_prize(&mut self) {
        if !self.fair() {
            return;
        }
        if self.w.p.fair_year == self.yr {
            return self.note("You already got your prize this year. Enjoy the fair!");
        }
        if self.w.p.inv.len() >= INV_CAP {
            return self.note("Make room in your pockets first!");
        }
        let d = FURN_IDS[gen_range(0, FURN_IDS.len())];
        self.add_furn(d);
        self.w.p.fair_year = self.yr;
        self.snd.push(Snd::Coin);
        self.note(format!("Harvest Fair prize: a {}!", def(d).name));
    }

    /// Where a visiting resident stands inside a home (next to the door).
    pub fn visit_spot(&self, b: usize) -> Vec2 {
        let (w, d) = self.rdims(b);
        vec2((w / 2) as f32 - 0.5, d as f32 - 1.5)
    }
    /// Start a visit by resident `i` to building `b` (the visited player's home). Provisional rules decide when this is called.
    pub fn start_visit(&mut self, i: usize, b: usize) {
        if self.visit.is_some() {
            return;
        }
        self.visit = Some((i, b, 120.));
        self.npcs[i].inside = true;
        self.npcs[i].path.clear();
        self.snd.push(Snd::Door);
        let vname = self.res(i).name;
        for pl in std::iter::once(&mut self.me).chain(self.others.iter_mut()) {
            if pl.scene == Scene::Room(b) {
                pl.msgs.push((format!("Knock knock! {} dropped by for a visit.", vname), self.t))
            }
        }
    }
    /// Visits (provisional): while a player is in their own home, a resident who is out, friendship >= 5, not yet visited
    /// that player today, may knock (about once every 10 s of eligible time). Ends after 120 s or when nobody is home.
    fn sim_visit(&mut self, dt: f32) {
        if let Some((i, b, left)) = self.visit {
            let occupied = std::iter::once(&self.me).chain(self.others.iter()).any(|p| p.scene == Scene::Room(b));
            if left - dt <= 0. || !occupied {
                self.visit = None;
                self.npcs[i].inside = false;
                self.npcs[i].pos = door_stand(b);
                self.npcs[i].wait = 1.;
            } else {
                self.visit = Some((i, b, left - dt))
            }
            return;
        }
        if gen_range(0, 600) != 0 {
            return;
        }
        let hosts: Vec<usize> = std::iter::once(&self.me)
            .chain(self.others.iter())
            .filter(|p| p.scene == Scene::Room(home_of(p.id)))
            .map(|p| p.id)
            .collect();
        for id in hosts {
            let pr = if self.w.p.id == id {
                &self.w.p
            } else {
                match self.w.others.iter().find(|p| p.id == id) {
                    Some(p) => p,
                    None => continue,
                }
            };
            if let Some(i) = (0..4).find(|&i| {
                self.loc(i) == Loc::Out
                    && self.seek_hide_position(i).is_none()
                    && !self.npcs[i].inside
                    && pr.npcs[i].friend >= 5
                    && !pr.npcs[i].visited
            }) {
                if self.w.p.id == id {
                    self.w.p.npcs[i].visited = true
                } else if let Some(p) = self.w.others.iter_mut().find(|p| p.id == id) {
                    p.npcs[i].visited = true
                }
                self.start_visit(i, home_of(id));
                return;
            }
        }
    }

    pub fn tint_of(pr: &Profile, uid: u32) -> u8 {
        pr.tints.iter().find(|t| t.0 == uid).map_or(0, |t| t.1)
    }
    /// Profile owning a home building (or the active one).
    pub fn owner(&self, o: usize) -> &Profile {
        if self.w.p.id == o {
            &self.w.p
        } else {
            self.w.others.iter().find(|p| p.id == o).unwrap_or(&self.w.p)
        }
    }
    /// Furniture in pockets that can be re-coloured (pocket slot indices).
    pub fn tint_targets(&self) -> Vec<usize> {
        (0..self.w.p.inv.len()).filter(|&i| self.w.p.inv[i].uid != 0 && def(self.w.p.inv[i].def).k == K::Furn).collect()
    }
    /// Counter "Customize": entry `sel` = a furniture item (cycle tint), then wallpaper, then flooring (cycle palette).
    pub fn customize(&mut self, sel: usize) -> Result<String, &'static str> {
        let t = self.tint_targets();
        let fee = if sel < t.len() { TINT_FEE } else { DECOR_FEE };
        if self.w.p.money < fee {
            return Err("Not enough coins.");
        }
        self.w.p.money -= fee;
        if sel < t.len() {
            let s = self.w.p.inv[t[sel]];
            let cur = Game::tint_of(&self.w.p, s.uid);
            let nt = (cur + 1) % TINTS.len() as u8;
            self.w.p.tints.retain(|x| x.0 != s.uid);
            if nt != 0 {
                self.w.p.tints.push((s.uid, nt))
            }
            Ok(format!("Your {} is now {}.", def(s.def).name, TINT_NAMES[nt as usize]))
        } else if sel == t.len() {
            self.w.p.wall = (self.w.p.wall + 1) % WALLS.len() as u8;
            Ok("New wallpaper installed at home.".into())
        } else {
            self.w.p.floor = (self.w.p.floor + 1) % FLOORS.len() as u8;
            Ok("New flooring installed at home.".into())
        }
    }

    /// Fully funded money projects are built at the daily boundary.
    pub fn finish_mproject(&mut self) {
        if let Some(m) = self.w.mproject.clone() {
            if m.funded >= MPROJECTS[m.idx].1 {
                self.w.built.push(Built { kind: MPROJECTS[m.idx].2, x: m.x, z: m.z, day: self.w.day });
                self.w.mproject = None;
                self.mail_all(
                    "Town notice board",
                    &format!("The new {} is finished! Thank you to everyone who donated.", MPROJECTS[m.idx].0),
                )
            }
        }
    }
    // ---- Gullrock timed activities ----
    pub fn iland(&self, p: Vec2) -> bool {
        let t = tile_of(p);
        matches!(tile_at_i(&self.imap, t.0, t.1), GRASS | SAND) && t != ISTALL && t != ACT_BOARD
    }
    fn iland_rand(&self) -> Vec2 {
        let pen = vec2(PEN.0, PEN.1);
        for _ in 0..300 {
            let p = vec2(gen_range(1, IW as i32 - 1) as f32 + 0.5, gen_range(1, IH as i32 - 1) as f32 + 0.5);
            if self.iland(p) && p.distance(pen) > PEN.2 + 0.8 {
                return p;
            }
        }
        vec2(7.5, 6.5)
    }
    pub fn in_pen(p: Vec2) -> bool {
        p.distance(vec2(PEN.0, PEN.1)) < PEN.2
    }
    pub fn act_tier(kind: usize, score: u32) -> usize {
        ACTS[kind].3.iter().filter(|&&t| score >= t).count()
    }
    /// Start activity `kind` for every player currently on Gullrock (a player in town is unaffected).
    pub fn act_start(&mut self, kind: usize) -> Result<(), &'static str> {
        if self.me.scene != Scene::Islet {
            return Err("Activities start at Gullrock.");
        }
        if self.act.is_some() {
            return Err("An activity is already running.");
        }
        let mut players = vec![self.me.id];
        players.extend(self.others.iter().filter(|o| o.scene == Scene::Islet).map(|o| o.id));
        let ents = match kind {
            0 => (0..6).map(|_| self.iland_rand()).collect(),
            1 => vec![self.iland_rand()],
            _ => rings(),
        };
        self.act = Some(Activity {
            kind,
            left: ACTS[kind].2,
            score: 0,
            mine: vec![],
            players,
            ents,
            next: 0,
            life: BEACON_LIFE,
            done: false,
            paid: vec![],
        });
        Ok(())
    }
    /// Shared timer and world behaviour (crabs wander, beacons move). Runs once per frame.
    pub fn sim_act(&mut self, dt: f32) {
        let Some(mut a) = self.act.take() else { return };
        if a.done {
            a.life -= dt;
            if a.life > 0. {
                self.act = Some(a)
            }
            return;
        }
        if a.players.is_empty() {
            return;
        } // everyone left: cancelled, no reward
        a.left -= dt;
        match a.kind {
            0 => {
                for k in 0..a.ents.len() {
                    let c = a.ents[k];
                    let an = (self.t * 0.7 + k as f32 * 2.1).sin() * 6.3 + k as f32;
                    let n = c + vec2(an.cos(), an.sin()) * dt * 0.5;
                    if self.iland(n) && Self::in_pen(n) == Self::in_pen(c) {
                        a.ents[k] = n
                    }
                }
                a.score = a.ents.iter().filter(|&&c| Self::in_pen(c)).count() as u32;
                if a.score as usize == a.ents.len() {
                    a.done = true
                }
            }
            1 => {
                a.life -= dt;
                if a.life <= 0. {
                    a.ents[0] = self.iland_rand();
                    a.life = BEACON_LIFE
                }
            }
            _ => {
                if a.next >= a.ents.len() {
                    a.done = true
                }
            }
        }
        if a.left <= 0. {
            a.left = 0.;
            a.done = true
        }
        if a.done {
            a.life = 6.
        }
        self.act = Some(a);
    }
    /// Per-player part: the active player's touches/pushes, and their own reward once it ends.
    pub fn act_step(&mut self) {
        let id = self.me.id;
        let (p, face, swim) = (self.me.ipos, self.me.face, self.swimming());
        let Some(mut a) = self.act.take() else { return };
        if !a.done && self.me.scene == Scene::Islet && !a.players.contains(&id) {
            a.players.push(id);
            self.note(format!("You joined {} in progress!", ACTS[a.kind].0))
        }
        if !a.done && self.me.scene == Scene::Islet && a.players.contains(&id) {
            let mut gain = 0;
            match a.kind {
                0 => {
                    for k in 0..a.ents.len() {
                        let d = a.ents[k] - p;
                        if d.length() < 0.65 {
                            let n = p + d.try_normalize().unwrap_or(face) * 0.65;
                            if self.iland(n) {
                                let was = Self::in_pen(a.ents[k]);
                                a.ents[k] = n;
                                if !was && Self::in_pen(n) {
                                    gain += 1
                                }
                            }
                        }
                    }
                }
                1 => {
                    if a.ents[0].distance(p) < 0.7 {
                        gain = 1;
                        a.score += 1;
                        a.ents[0] = self.iland_rand();
                        a.life = BEACON_LIFE
                    }
                }
                _ => {
                    if swim && a.next < a.ents.len() && a.ents[a.next].distance(p) < 0.9 {
                        gain = 1;
                        a.score += 1;
                        a.next += 1
                    }
                }
            }
            if gain > 0 {
                self.snd.push(Snd::Blip);
                if let Some(m) = a.mine.iter_mut().find(|m| m.0 == id) {
                    m.1 += gain
                } else {
                    a.mine.push((id, gain))
                }
            }
        }
        if a.done && a.players.contains(&id) && !a.paid.contains(&id) {
            a.paid.push(id);
            let day = self.w.day;
            if self.w.p.task_day == day {
                for t in &mut self.w.p.tasks {
                    if TASKS[t.t].0 == Ev::Activity {
                        t.prog = TASKS[t.t].1
                    }
                }
            }
            let tier = Self::act_tier(a.kind, a.score);
            let name = ACTS[a.kind].0;
            if tier == 0 {
                self.note(format!("{name} over: score {}. No reward this time.", a.score))
            } else if self.w.p.act_day[a.kind] == self.w.day {
                self.note(format!("{name} over: score {}. (Already rewarded for this one today.)", a.score))
            } else {
                let n = ACT_REWARD[tier - 1];
                let mut got = 0;
                for _ in 0..n {
                    if self.add(85) {
                        got += 1
                    }
                }
                for _ in got..n {
                    self.w.p.mail.push(Letter {
                        from: "Captain Pell".into(),
                        text: "Your pockets were full, so here is the Sea Glass you earned.".into(),
                        coins: 0,
                        item: Some(85),
                        read: false,
                        claimed: false,
                    })
                }
                if got < n {
                    self.note(format!("Pockets full: {} Sea Glass sent to your mailbox.", n - got))
                }
                self.w.p.act_day[a.kind] = self.w.day;
                self.snd.push(Snd::Coin);
                self.note(format!(
                    "{name} over: score {} ({}). +{got} Sea Glass!",
                    a.score,
                    ["", "bronze", "silver", "gold"][tier]
                ))
            }
        }
        self.act = Some(a);
    }
    // ---- Gullrock Islet ----
    pub fn refresh_islet(&mut self) {
        self.w.iground.clear();
        let mut free: Vec<(i32, i32)> = (0..IW * IH)
            .filter(|&i| self.imap[i] == SAND)
            .map(|i| ((i % IW) as i32, (i / IW) as i32))
            .filter(|&t| t != ISTALL && t != (7, 11) && t != (6, 11) && t != (7, 10))
            .collect();
        let foreign: Vec<u16> = FRUITS.iter().copied().filter(|&f| f != self.w.native).collect();
        let ff = foreign[self.w.day.rem_euclid(foreign.len() as i64) as usize]; // one non-native fruit washes ashore daily
        for (d, n) in [(82, 7), (84, 4), (75, 2), (ff, 2)] {
            for _ in 0..n {
                if free.is_empty() {
                    break;
                }
                let t = free.remove(gen_range(0, free.len()));
                let uid = self.uid();
                self.w.iground.push(GItem { uid, def: d, x: t.0 as f32 + 0.5, z: t.1 as f32 + 0.5 })
            }
        }
    }
    /// Captain Pell: offers the return trip, explains today's activity, and pays 3 Sea Glass once per player per day.
    pub fn pell(&mut self) {
        let (d, n, label) = islet_task(self.w.day);
        let have: u16 = self.w.p.inv.iter().filter(|s| s.def == d).map(|s| s.qty).sum();
        let mut lines = vec![format!("Ahoy! Today's islet task: {label}.")];
        if self.w.p.islet_done == self.w.day {
            lines.push("You've already done today's task. Come back tomorrow!".into())
        } else if have >= n {
            for _ in 0..n {
                self.remove_one(d);
            }
            let mut got = 0;
            for _ in 0..3 {
                if self.add(85) {
                    got += 1
                }
            }
            for _ in got..3 {
                self.w.p.mail.push(Letter {
                    from: "Captain Pell".into(),
                    text: "Your pockets were full, so here is the Sea Glass you earned.".into(),
                    coins: 0,
                    item: Some(85),
                    read: false,
                    claimed: false,
                })
            }
            self.w.p.islet_done = self.w.day;
            self.snd.push(Snd::Coin);
            lines.push("Splendid work! Here are 3 pieces of Sea Glass.".into())
        } else {
            lines.push(format!("You have {have} of {n} so far."))
        }
        lines.push("Talk to me again after this to sail home (free).".into());
        self.me.ui = Ui::Talk { npc: None, who: "Captain Pell the pelican".into(), lines, i: 0 };
        self.pell_talked = true;
    }
    pub fn istall_buy(&mut self, k: usize) -> Result<(), &'static str> {
        let (d, cost) = ISTALL_ITEMS[k];
        let have: u16 = self.w.p.inv.iter().filter(|s| s.def == 85).map(|s| s.qty).sum();
        if have < cost {
            return Err("Not enough Sea Glass.");
        }
        let frees_slot = self.w.p.inv.iter().filter(|s| s.def == 85).count() == 1 && have == cost;
        if self.w.p.inv.len() >= INV_CAP && !frees_slot {
            return Err("Your pockets are full.");
        }
        for _ in 0..cost {
            self.remove_one(85);
        }
        self.add_furn(d);
        Ok(())
    }

    // ---- community projects ----
    pub fn remaining(p: &Project) -> String {
        let r: Vec<String> =
            p.need.iter().filter(|n| n.2 < n.1).map(|n| format!("{} {}", n.1 - n.2, MAT_NAMES[n.0 as usize])).collect();
        if r.is_empty() {
            "It's ready!".into()
        } else {
            format!("Still needed: {}.", r.join(", "))
        }
    }
    pub fn liveliness(&self) -> i32 {
        let flowers = self.map.iter().filter(|&&c| c == FLOWER).count() as i32 - self.w.trampled.len() as i32
            + 2 * self.w.garden.iter().filter(|p| p.stage == 2).count() as i32;
        (10 * self.w.built.len() as i32 + 2 * self.w.donated.len() as i32 + flowers / 2 - 3 * self.w.weeds.len() as i32)
            .clamp(0, 100)
    }
    pub fn live_word(l: i32) -> &'static str {
        if l < 20 {
            "Quiet"
        } else if l < 45 {
            "Cozy"
        } else if l < 70 {
            "Lively"
        } else {
            "Bustling"
        }
    }
    pub fn board_lines(&self) -> Vec<String> {
        let mut v =
            vec![format!("Town liveliness: {} / 100 ({}).", self.liveliness(), Game::live_word(self.liveliness()))];
        v.push(self.contest_notice());
        if let Some(s) = self.social_notice() {
            v.push(s);
        }
        if let Some(p) = &self.w.project {
            v.push(format!(
                "Current project: a {} proposed by {}. {}",
                PROJECTS[p.kind as usize].0,
                self.res(p.by).name,
                Game::remaining(p)
            ))
        }
        v.push("Funded public projects are started by the town leader at the Town Hall civic desk.".into());
        if let Some(m) = &self.w.mproject {
            v.push(format!(
                "Funded project: a {} ({} / {} coins). It is built the morning after it is fully funded.",
                MPROJECTS[m.idx].0, m.funded, MPROJECTS[m.idx].1
            ))
        }
        v.push(if self.w.built.is_empty() {
            "Built so far: nothing yet.".into()
        } else {
            format!(
                "Built so far: {}.",
                self.w.built.iter().map(|b| PROJECTS[b.kind as usize].0).collect::<Vec<_>>().join(", ")
            )
        });
        v
    }
    /// Front tile where the leader would place a money project.
    pub fn mp_front(&self) -> (i32, i32) {
        tile_of(self.me.pos + self.me.face)
    }
    /// Site check: plain free grass, not a reserved tile, and every building stays reachable.
    pub fn mp_site_ok(&mut self, t: (i32, i32)) -> bool {
        if self.seek_reserved(t) {
            return false;
        }
        if tile_at(&self.map, t.0, t.1) != GRASS
            || !self.walk_tile(t.0, t.1)
            || self.occupied(t.0, t.1)
            || [mailbox_tile(0), mailbox_tile(1), STALL, BOARD, DOCK, tile_of(self.me.pos)].contains(&t)
        {
            return false;
        }
        if self.w.project.as_ref().is_some_and(|p| (p.x, p.z) == t) {
            return false;
        }
        let keep = self.w.mproject.take();
        self.w.mproject = Some(MProject { idx: 0, x: t.0, z: t.1, funded: 0, log: vec![] });
        let ok = (0..BLDS.len()).all(|b| self.bfs(door_tile(SHOP), door_tile(b), true).is_some())
            && self.bfs(door_tile(SHOP), DOCK_APPROACH, true).is_some();
        self.w.mproject = keep;
        ok
    }
    /// Leader starts money project `idx` at the front tile.
    pub fn mp_start(&mut self, idx: usize) -> Result<(), &'static str> {
        if self.me.id != LEADER {
            return Err("Only the town leader can start public projects.");
        }
        if self.w.mproject.is_some() {
            return Err("A funded project is already underway.");
        }
        let t = self.mp_front();
        if !self.mp_site_ok(t) {
            return Err("Can't build there: needs open grass, paths clear.");
        }
        self.w.mproject = Some(MProject { idx, x: t.0, z: t.1, funded: 0, log: vec![] });
        Ok(())
    }
    /// Donate up to `amount` of the active player's own coins (capped by what is still needed).
    pub fn mp_fund(&mut self, amount: u32) -> Result<u32, &'static str> {
        let Some(m) = self.w.mproject.as_mut() else { return Err("No funded project right now.") };
        let left = MPROJECTS[m.idx].1 - m.funded;
        if left == 0 {
            return Err("It's fully funded! It will be built tomorrow morning.");
        }
        let a = amount.min(left).min(self.w.p.money);
        if a == 0 {
            return Err("Not enough coins.");
        }
        self.w.p.money -= a;
        m.funded += a;
        let id = self.me.id;
        if let Some(l) = m.log.iter_mut().find(|l| l.0 == id) {
            l.1 += a
        } else {
            m.log.push((id, a))
        }
        Ok(a)
    }
    /// A resident proposes the next project at the first valid candidate site (never blocking a building route).
    pub fn propose(&mut self) {
        if self.w.project.is_some() {
            return;
        }
        let has_bridge = self.w.built.iter().any(|b| b.kind == PK_BRIDGE);
        for step in 0..4 {
            let kind = ((self.w.built.len() + step) % 4) as u8;
            if kind == PK_BRIDGE && has_bridge {
                continue;
            }
            for &(x, z) in SITES[kind as usize] {
                let ts = ptiles(kind, x, z);
                let ok = if kind == PK_BRIDGE {
                    ts.iter().all(|t| tile_at(&self.map, t.0, t.1) == RIVER)
                } else {
                    ts.iter().all(|t| {
                        tile_at(&self.map, t.0, t.1) == GRASS
                            && self.walk_tile(t.0, t.1)
                            && !self.occupied(t.0, t.1)
                            && *t != mailbox_tile(0)
                            && *t != mailbox_tile(1)
                            && *t != STALL
                    })
                };
                if !ok {
                    continue;
                }
                let by = gen_range(0, 4);
                self.w.project = Some(Project {
                    kind,
                    x,
                    z,
                    need: PROJECTS[kind as usize].1.iter().map(|&(c, q)| (c, q, 0)).collect(),
                    by,
                    log: vec![],
                });
                let routes = kind == PK_BRIDGE
                    || (0..BLDS.len()).all(|b| self.bfs(door_tile(SHOP), door_tile(b), true).is_some());
                if !routes {
                    self.w.project = None;
                    continue;
                }
                let r = self.res(by);
                self.mail_all(
                    r.name,
                    &format!(
                        "Let's build a {} together! I put up a sign where it should go. Bring materials when you can.",
                        PROJECTS[kind as usize].0
                    ),
                );
                return;
            }
        }
    }
    /// Contribute matching pocket items to the active project (never more than still needed); completes it when full.
    pub fn contribute(&mut self) {
        let Some(mut p) = self.w.project.clone() else { return };
        let mut given = 0u16;
        for n in p.need.iter_mut() {
            while n.2 < n.1 {
                let Some(d) = self.w.p.inv.iter().find(|s| s.uid == 0 && mat_ok(n.0, s.def)).map(|s| s.def) else {
                    break;
                };
                self.remove_one(d);
                n.2 += 1;
                given += 1
            }
        }
        if given > 0 {
            if let Some(l) = p.log.iter_mut().find(|l| l.0 == self.me.id) {
                l.1 += given
            } else {
                p.log.push((self.me.id, given))
            }
            self.snd.push(Snd::Coin)
        }
        let name = PROJECTS[p.kind as usize].0;
        if p.need.iter().all(|n| n.2 >= n.1) {
            self.w.built.push(Built { kind: p.kind, x: p.x, z: p.z, day: self.w.day });
            self.w.project = None;
            for (k, n) in self.npcs.iter_mut().enumerate() {
                if !n.inside {
                    n.happy = 3.;
                    let hp = n.pos;
                    for j in 0..4 {
                        self.hearts.push((vec3(hp.x + (j as f32 - 1.5) * 0.12, 1.3, hp.y), -(j as f32) * 0.12))
                    }
                }
                let _ = k;
            }
            self.fx.push((vec3(p.x as f32 + 0.5, 0.05, p.z as f32 + 0.5), 0., 1));
            self.me.ui = Ui::Talk {
                npc: None,
                who: "Project sign".into(),
                lines: vec![
                    format!("You gave {given} item(s). The {name} is finished! Everyone gathers to admire it."),
                    format!("Town liveliness is now {} ({}).", self.liveliness(), Game::live_word(self.liveliness())),
                ],
                i: 0,
            };
        } else {
            let msg = if given > 0 {
                format!("You contributed {given} item(s) to the {name}.")
            } else {
                format!("You have nothing the {name} needs right now.")
            };
            self.me.ui =
                Ui::Talk { npc: None, who: "Project sign".into(), lines: vec![msg, Game::remaining(&p)], i: 0 };
            self.w.project = Some(p);
        }
    }

    // ---- clothing ----
    pub fn buy_cloth(&mut self, d: u16) -> Result<(), &'static str> {
        let p = def(d).buy;
        if self.w.p.money < p {
            return Err("Not enough coins.");
        }
        if self.w.p.inv.len() >= INV_CAP {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= p;
        self.add_furn(d);
        self.ev(Ev::Buy, 1);
        self.w.spent += p;
        Ok(())
    }
    /// Wear a shirt from pockets; the shirt worn before goes back into the same pocket slot (no duplication, no loss).
    pub fn wear(&mut self, uid: u32) {
        let Some(i) = self.w.p.inv.iter().position(|s| s.uid == uid && def(s.def).k == K::Cloth) else { return };
        let d = self.w.p.inv[i].def;
        let slot = if is_hat(d) { &mut self.w.p.hat } else { &mut self.w.p.shirt };
        match slot.replace(d) {
            Some(old) => self.w.p.inv[i].def = old,
            None => {
                self.w.p.inv.remove(i);
            }
        }
        self.note(format!("You put on the {}.", def(d).name));
    }

    // ---- museum ----
    pub fn donated_of(&self, k: K) -> usize {
        self.w.donated.iter().filter(|x| def(x.0).k == k).count()
    }
    /// Curator: identify unknown fossils, then donate every pocket item the museum doesn't have yet (shared, once per species).
    pub fn donate(&mut self) {
        let mut lines = vec![];
        while self.w.p.inv.iter().any(|s| s.def == 47) {
            self.remove_one(47);
            let f = FOSSILS[gen_range(0, FOSSILS.len())];
            if !self.add(f) {
                self.add(47);
                lines.push("Your pockets are too full for me to hand it back. Make some room first.".to_string());
                break;
            }
            lines.push(format!("Let me see... this fossil is a {}!", def(f).name));
        }
        let mut cands: Vec<u16> = self
            .w
            .p
            .inv
            .iter()
            .map(|s| s.def)
            .filter(|&d| matches!(def(d).k, K::Fish | K::Bug | K::Fossil | K::Sea | K::Art) && d != 47)
            .collect();
        cands.dedup();
        for d in cands {
            if self.w.donated.iter().any(|x| x.0 == d) {
                continue;
            }
            if self.remove_one(d) {
                self.w.donated.push((d, self.me.id));
                lines.push(format!("A {}! It will make a splendid exhibit.", def(d).name))
            }
        }
        if lines.is_empty() {
            lines.push(
                "Bring me fish, bugs, fossils or paintings the museum doesn't have yet, and I'll put them on display."
                    .into(),
            )
        } else {
            self.snd.push(Snd::Coin)
        }
        let tot = |k: K| DEFS.iter().filter(|d| d.k == k).count();
        lines.push(format!(
            "Collection: {}/{} fish, {}/{} bugs, {}/{} fossils, {}/{} paintings.",
            self.donated_of(K::Fish),
            tot(K::Fish),
            self.donated_of(K::Bug),
            tot(K::Bug),
            self.donated_of(K::Fossil),
            FOSSILS.len(),
            self.donated_of(K::Art),
            ARTS.len()
        ));
        self.me.ui = Ui::Talk { npc: None, who: "Professor Quill the heron".into(), lines, i: 0 };
    }

    // ---- shop ----
    pub fn sell_slot(&mut self, i: usize) {
        if i >= self.w.p.inv.len() {
            return;
        }
        let s = self.w.p.inv.remove(i);
        let v = def(s.def).sell * s.qty as u32;
        self.w.p.money += v;
        self.ev(Ev::Sell, v);
        self.snd.push(Snd::Coin);
        self.note(format!("Sold {} x{} for {v} coins.", def(s.def).name, s.qty))
    }
    pub fn sell_all(&mut self) {
        let (keep, sell): (Vec<Slot>, Vec<Slot>) =
            self.w.p.inv.iter().copied().partition(|s| def(s.def).k == K::Furn || def(s.def).sell == 0);
        let v: u32 = sell.iter().map(|s| def(s.def).sell * s.qty as u32).sum();
        self.w.p.inv = keep;
        self.w.p.money += v;
        self.ev(Ev::Sell, v);
        if v > 0 {
            self.snd.push(Snd::Coin)
        }
        self.note(format!("Sold all your finds for {v} coins."));
    }
    pub fn buy(&mut self, disp: usize) -> Result<(), &'static str> {
        let m0 = self.w.p.money;
        let r = self.buy_inner(disp);
        if r.is_ok() {
            self.ev(Ev::Buy, 1);
            self.w.spent += m0 - self.w.p.money
        }
        r
    }
    pub fn shop_tier(&self) -> u8 {
        SHOP_TIERS.iter().filter(|&&t| self.w.spent >= t).count() as u8
    }
    /// (item, price, sold) for display index: 0-4 stock, 5-6 tools, 7 extra stock, 8 weekly special.
    pub fn display_item(&self, d: usize) -> (u16, u32, bool) {
        match d {
            5 | 6 => {
                let t = TOOL_DISPLAY[d - 5];
                (t, def(t).buy, self.w.p.tools.contains(&t))
            }
            8 => {
                let wk = week_of(self.w.day);
                let s = SPECIALS[wk.rem_euclid(SPECIALS.len() as i64) as usize];
                (s, def(s).buy, self.w.special_week == wk)
            }
            _ => {
                let i = if d == 7 { 5 } else { d };
                self.w.stock.get(i).map_or((0, 0, true), |s| (s.def, def(s.def).buy, s.sold))
            }
        }
    }
    fn buy_inner(&mut self, disp: usize) -> Result<(), &'static str> {
        if disp == 8 {
            if self.shop_tier() < 1 {
                return Err("Not available yet.");
            }
            let (d, p, sold) = self.display_item(8);
            if sold {
                return Err("Sold out!");
            }
            if self.w.p.money < p {
                return Err("Not enough coins.");
            }
            if !self.add_furn(d) {
                return Err("Your pockets are full.");
            }
            self.w.p.money -= p;
            self.w.special_week = week_of(self.w.day);
            return Ok(());
        }
        let disp = if disp == 7 {
            if self.w.stock.len() <= 5 {
                return Err("Not available yet.");
            }
            5
        } else {
            if disp >= 5 {
                100 + disp
            } else {
                disp
            }
        };
        if disp >= 100 {
            let disp = disp - 100;
            let t = TOOL_DISPLAY[disp - 5];
            if self.w.p.tools.contains(&t) {
                return Err("You already own one.");
            }
            if self.w.p.money < def(t).buy {
                return Err("Not enough coins.");
            }
            self.w.p.money -= def(t).buy;
            self.w.p.tools.push(t);
            self.w.p.equip = Some(t);
            return Ok(());
        }
        let st = self.w.stock[disp];
        if st.sold {
            return Err("Sold out!");
        }
        let p = def(st.def).buy;
        if self.w.p.money < p {
            return Err("Not enough coins.");
        }
        if self.w.p.inv.len() >= INV_CAP {
            return Err("Your pockets are full.");
        }
        self.w.p.money -= p;
        self.w.stock[disp].sold = true;
        self.add_furn(st.def);
        Ok(())
    }

    // ---- furniture ----
    fn use_furn(&mut self, uid: u32, d: u16) {
        let df = def(d);
        if uid == 0 {
            return self.note(format!("A cozy {}. Better not touch someone else's things.", df.name.to_lowercase()));
        }
        let Some(idx) = self.w.p.room.iter().position(|f| f.uid == uid) else { return };
        let f = self.w.p.room[idx];
        match df.act {
            Act::Sit => {
                self.me.stand_from = self.me.ipos;
                let ip = self.me.ipos;
                self.me.ipos = foot(f.def, f.x, f.z, f.rot)
                    .into_iter()
                    .map(tc)
                    .min_by(|a, b| a.distance(ip).total_cmp(&b.distance(ip)))
                    .unwrap();
                self.me.face = dir(f.rot);
                self.me.pose = Pose::Sit(uid);
                self.note("You sit down. Move to stand up.")
            }
            Act::Bed => {
                self.me.stand_from = self.me.ipos;
                self.me.ipos = fcenter(&f);
                self.me.face = dir(f.rot);
                self.me.pose = Pose::Lie(uid);
                self.note("You lie down for a rest. Time keeps flowing as usual.")
            }
            Act::Lamp => {
                self.w.p.room[idx].on = !f.on;
                self.note(if f.on { "Lamp off." } else { "Lamp on." })
            }
            Act::Storage => self.me.ui = Ui::Storage(0, 0),
            Act::None => self.note(format!("Your {} looks great here.", df.name.to_lowercase())),
        }
    }
    pub fn start_place(&mut self, uid: u32) {
        let Some(s) = self.w.p.inv.iter().find(|s| s.uid == uid).copied() else { return };
        let t = tile_of(self.me.ipos + self.me.face);
        let wall = def(s.def).wall;
        self.me.placing = Some(Placing {
            f: Furn { uid, def: s.def, x: t.0, z: if wall { -1 } else { t.1 }, rot: 0, on: false },
            orig: None,
            slot_uid: Some(uid),
        });
    }
    pub fn start_move(&mut self, uid: u32) {
        if let Some(i) = self.w.p.room.iter().position(|f| f.uid == uid) {
            let f = self.w.p.room.remove(i);
            self.me.placing = Some(Placing { f, orig: Some(f), slot_uid: None })
        }
    }
    pub fn place_cancel(&mut self) {
        if let Some(p) = self.me.placing.take() {
            if let Some(o) = p.orig {
                self.w.p.room.push(o)
            }
        }
    }
    pub fn place_ok(&self, f: &Furn) -> Result<(), &'static str> {
        let (w, d) = self.rdims(self.home());
        let df = def(f.def);
        if df.wall {
            if f.z != -1 || f.x < 0 || f.x >= w {
                return Err("Wall items go on the back wall.");
            }
            if self.w.p.room.iter().any(|o| def(o.def).wall && o.x == f.x && o.uid != f.uid) {
                return Err("Something already hangs there.");
            }
            if self.home_stairs(self.home()).iter().any(|s| elev(s.1) > elev(self.w.p.story) && s.0 .0 == f.x) {
                return Err("The stairs are in the way.");
            }
            return Ok(());
        }
        let fp = foot(f.def, f.x, f.z, f.rot);
        if fp.iter().any(|&(x, z)| x < 0 || z < 0 || x >= w || z >= d) {
            return Err("It doesn't fit there.");
        }
        if self
            .home_stairs(self.home())
            .iter()
            .any(|s| fp.iter().any(|&(x, z)| x == s.0 .0 && (z == s.0 .1 || z == s.0 .1 + 1)))
        {
            return Err("Keep the stairs clear.");
        }
        if fp.contains(&(w / 2, d - 1)) {
            return Err("Keep the doorway clear.");
        }
        if fp.iter().any(|&t| self.room_block(self.home(), t.0, t.1, f.uid)) {
            return Err("Something is in the way.");
        }
        let me: Vec<(i32, i32)> = [(-PR, -PR), (PR, -PR), (-PR, PR), (PR, PR)]
            .iter()
            .map(|&(a, b)| tile_of(self.me.ipos + vec2(a, b)))
            .collect();
        if fp.iter().any(|t| me.contains(t)) {
            return Err("You're standing there.");
        }
        if !self.reach(tile_of(self.me.ipos), (w / 2, d - 1), &fp) {
            return Err("That would block your way out.");
        }
        Ok(())
    }
    fn reach(&self, a: (i32, i32), b: (i32, i32), extra: &[(i32, i32)]) -> bool {
        let mut seen = HashSet::from([a]);
        let mut q = VecDeque::from([a]);
        while let Some(c) = q.pop_front() {
            if c == b {
                return true;
            }
            for d in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (c.0 + d.0, c.1 + d.1);
                if !seen.contains(&n) && !self.room_block(self.home(), n.0, n.1, 0) && !extra.contains(&n) {
                    seen.insert(n);
                    q.push_back(n)
                }
            }
        }
        false
    }
    pub fn place_confirm(&mut self) -> Result<(), &'static str> {
        let (f, su) = {
            let p = self.me.placing.as_ref().ok_or("Nothing to place.")?;
            (p.f, p.slot_uid)
        };
        self.place_ok(&f)?;
        if let Some(uid) = su {
            let i = self.w.p.inv.iter().position(|s| s.uid == uid).ok_or("Item missing.")?;
            self.w.p.inv.remove(i);
        }
        let before = self.sets_done();
        self.w.p.room.push(f);
        self.me.placing = None;
        for n in self.sets_done() {
            if !before.contains(&n) {
                self.note(format!("{n} complete!"));
                self.snd.push(Snd::Coin)
            }
        }
        Ok(())
    }
    pub fn pick_furn(&mut self, uid: u32) {
        if self.w.p.inv.len() >= INV_CAP {
            return self.note("Your pockets are full!");
        }
        if let Some(i) = self.w.p.room.iter().position(|f| f.uid == uid) {
            let f = self.w.p.room.remove(i);
            self.w.p.inv.push(Slot { size_mm: 0, def: f.def, qty: 1, uid });
            self.note(format!("Picked up the {}.", def(f.def).name))
        }
    }
    pub fn transfer(&mut self, col: usize, s: usize) {
        let w = &mut self.w;
        let (src, dst, cap) = if col == 0 {
            (&mut w.p.inv, &mut w.p.storage, STORE_CAP)
        } else {
            (&mut w.p.storage, &mut w.p.inv, INV_CAP)
        };
        if s >= src.len() {
            return;
        }
        if dst.len() >= cap {
            return self.note("No room there.");
        }
        let it = src.remove(s);
        dst.push(it);
    }

    // ---- tools ----
    pub fn cycle_tool(&mut self) {
        let mut opts: Vec<Option<u16>> = vec![None];
        let mut ts = self.w.p.tools.clone();
        ts.sort();
        opts.extend(ts.into_iter().map(Some));
        let i = opts.iter().position(|&o| o == self.w.p.equip).unwrap_or(0);
        self.w.p.equip = opts[(i + 1) % opts.len()];
        self.note(match self.w.p.equip {
            Some(t) => format!("Equipped: {}", def(t).name),
            None => "Tools put away.".into(),
        });
    }
    /// Player is in the sea (outdoors). Swimming is entered by walking off the beach (provisional rule).
    pub fn swimming(&self) -> bool {
        match self.me.scene {
            Scene::Town => {
                let t = tile_of(self.me.pos);
                self.tile(t.0, t.1) == SEA
            }
            Scene::Islet => {
                let t = tile_of(self.me.ipos);
                tile_at_i(&self.imap, t.0, t.1) == SEA
            }
            _ => false,
        }
    }
    /// Dive under the nearest bubble spot within reach and collect the sea creature there.
    pub fn dive(&mut self) {
        let p = self.ppos();
        self.fx.push((vec3(p.x, -0.17, p.y), 0., 0));
        self.snd.push(Snd::Splash);
        let islet = self.me.scene == Scene::Islet;
        let list = if islet { &self.ibubbles } else { &self.bubbles };
        let Some(i) = list.iter().position(|b| b.0.distance(p) < 1.3) else {
            return self.note("You dive... nothing down here.");
        };
        let d = list[i].1;
        if self.add(d) {
            if islet {
                self.ibubbles.remove(i);
            } else {
                self.bubbles.remove(i);
            }
            self.note(format!("You found a {} while diving!", def(d).name));
            self.ev(Ev::Dive, 1)
        }
    }
    pub fn use_tool(&mut self) {
        if self.swimming() {
            return self.dive();
        }
        if self.me.scene == Scene::Islet && self.swimming() {
            return self.dive();
        }
        if self.me.scene == Scene::Islet {
            // only the net works on Gullrock (for island bugs)
            if self.w.p.equip != Some(60) {
                return self.note("Only the net is useful on Gullrock.");
            }
            self.me.swing = 0.35;
            let at = self.me.ipos + self.me.face * 0.7;
            self.fx.push((vec3(at.x, 0.6, at.y), 0., 2));
            if let Some(j) = {
                let reach = 0.8 + 0.3 * self.tool_level(60) as f32;
                self.ibugs.iter().position(|b| b.1.distance(at) < reach)
            } {
                let d = self.ibugs.remove(j).0;
                if self.add_catch(d) {
                    self.ev(Ev::Bug, 1)
                }
            } else {
                self.snd.push(Snd::Rustle)
            }
            return;
        }
        if self.me.scene != Scene::Town {
            return;
        }
        if self.me.fish.is_some() {
            return self.reel();
        }
        let (p, f) = (self.me.pos, self.me.face);
        match self.w.p.equip {
            None => self.note("No tool equipped. Press Tab to cycle tools."),
            Some(60) => {
                self.me.swing = 0.35;
                let at = p + f * 0.7;
                self.fx.push((vec3(at.x, 0.6, at.y), 0., 2));
                if self.me.scene == Scene::Islet {
                    if let Some(j) = self.ibugs.iter().position(|b| b.1.distance(at) < 0.8) {
                        let d = self.ibugs.remove(j).0;
                        if self.add_catch(d) {
                            self.ev(Ev::Bug, 1)
                        }
                    } else {
                        self.snd.push(Snd::Rustle)
                    }
                } else if let Some(j) = {
                    let reach = 0.8 + 0.3 * self.tool_level(60) as f32;
                    self.bugs.iter().position(|b| b.1.distance(at) < reach)
                } {
                    let d = self.bugs.remove(j).0;
                    if self.add_catch(d) {
                        self.ev(Ev::Bug, 1)
                    }
                } else {
                    self.snd.push(Snd::Rustle)
                }
            }
            Some(CAN) => {
                self.me.swing = 0.35;
                self.water();
            }
            Some(61) => {
                let spot = [1.6, 2.2, 2.8].iter().map(|&k| p + f * k).find(|&q| {
                    let t = tile_of(q);
                    matches!(self.tile(t.0, t.1), RIVER | SEA)
                });
                let Some(bob) = spot else { return self.note("Face the water to cast.") };
                self.me.swing = 0.35;
                let bt = tile_of(bob);
                let hab = if self.tile(bt.0, bt.1) == SEA { 2 } else { 1 };
                let fish = self
                    .shadows
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| (def(s.def).hab == hab || def(s.def).hab == 3) && s.pos.distance(bob) < 5.)
                    .min_by(|a, b| a.1.pos.distance(bob).total_cmp(&b.1.pos.distance(bob)))
                    .map(|(i, _)| i);
                self.me.fish = Some(Fishing {
                    bob,
                    fish,
                    t: 0.,
                    nib: 0,
                    need: gen_range(1u8, 4 - self.tool_level(61)),
                    bite: false,
                    dip: 0.3,
                });
                self.snd.push(Snd::Splash);
                self.fx.push((vec3(bob.x, -0.17, bob.y), 0., 0));
            }
            Some(_) => {
                self.me.swing = 0.35;
                let t = tile_of(p + f * 0.9);
                self.fx.push((vec3(t.0 as f32 + 0.5, 0.05, t.1 as f32 + 0.5), 0., 1));
                if self.tile(t.0, t.1) == ROCK {
                    self.snd.push(Snd::Dig);
                    if self.w.rocks_hit.contains(&t) {
                        return self.note("This rock has nothing more to give today.");
                    }
                    if self.add(81) {
                        self.w.rocks_hit.push(t);
                        self.ev(Ev::Rock, 1);
                        for _ in 0..self.tool_level(62) {
                            self.add(81);
                        }
                    }
                    return;
                }
                if let Some(i) = self.w.garden.iter().position(|p| (p.x, p.z) == t) {
                    self.w.garden.remove(i);
                    self.snd.push(Snd::Dig);
                    return self.note("You dug up the flower.");
                }
                if let Some(i) = self.w.holes.iter().position(|&h| h == t) {
                    self.w.holes.remove(i);
                    self.snd.push(Snd::Dig);
                    self.note("You filled the hole.")
                } else if let Some(i) = self.w.digs.iter().position(|&h| h == t) {
                    self.w.digs.remove(i);
                    self.w.holes.push(t);
                    self.snd.push(Snd::Dig);
                    let d = [47, 47, 47, 45, 46][gen_range(0, 5)];
                    if self.add(d) {
                        self.ev(Ev::Dig, 1);
                        if self.tool_level(62) >= 2 && gen_range(0, 3) == 0 {
                            let b = [45, 46][gen_range(0, 2)];
                            if self.add(b) {
                                self.note(format!("Your masterwork shovel turned up a {} too!", def(b).name))
                            }
                        }
                    }
                } else if matches!(self.tile(t.0, t.1), GRASS | SAND | PATH)
                    && self.walk_tile(t.0, t.1)
                    && !self.occupied(t.0, t.1)
                    && t != tile_of(p)
                {
                    self.w.holes.push(t);
                    self.snd.push(Snd::Dig);
                    self.note("You dug a hole.")
                } else {
                    self.note("You can't dig there.")
                }
            }
        }
    }
    fn reel(&mut self) {
        let Some(f) = self.me.fish.take() else { return };
        match f.fish {
            Some(si) if f.bite => {
                self.fx.push((vec3(f.bob.x, -0.17, f.bob.y), 0., 0));
                let Some(sh) = self.take_shadow(si) else { return self.note("The fish slipped away.") };
                let d = sh.def;
                self.snd.push(Snd::Splash);
                if self.add_catch(d) {
                    self.note(format!("You caught a {}!", def(d).name));
                    self.ev(Ev::Fish, 1)
                }
            }
            Some(si) => {
                self.take_shadow(si);
                self.note("Too early! The fish swam off.")
            }
            None => self.note("You reel in your line."),
        }
    }

    /// Remove a fish shadow and fix every player's hooked-fish index (shadows are shared, so indices shift).
    fn take_shadow(&mut self, k: usize) -> Option<Shadow> {
        if k >= self.shadows.len() {
            return None;
        }
        let s = self.shadows.remove(k);
        for p in std::iter::once(&mut self.me).chain(self.others.iter_mut()) {
            if let Some(f) = &mut p.fish {
                match f.fish {
                    Some(j) if j == k => f.fish = None,
                    Some(j) if j > k => f.fish = Some(j - 1),
                    _ => {}
                }
            }
        }
        Some(s)
    }
    // ---- simulation ----
    fn sim_fish(&mut self, dt: f32) {
        let Some(mut f) = self.me.fish.take() else { return };
        f.t += dt;
        f.dip = (f.dip - dt * 2.5).max(0.);
        let mut keep = true;
        match f.fish {
            None => {
                if f.t > 6. {
                    keep = false;
                    self.note("Nothing seems interested...")
                }
            }
            Some(si) => {
                if f.bite {
                    if f.t > 0.8 + 0.3 * self.tool_level(61) as f32 {
                        keep = false;
                        self.take_shadow(si);
                        self.note("It got away!")
                    }
                } else if let Some(sh) = self.shadows.get_mut(si) {
                    let to = f.bob - sh.pos;
                    if to.length() > 0.3 {
                        sh.pos += to.normalize() * 0.8 * dt;
                        f.t = 0.
                    } else if f.t > 0.9 {
                        f.t = 0.;
                        f.nib += 1;
                        f.dip = 0.5;
                        self.snd.push(Snd::Blip);
                        if f.nib >= f.need {
                            f.bite = true;
                            f.dip = 1.2;
                            self.snd.push(Snd::Splash)
                        }
                    }
                } else {
                    keep = false;
                    self.note("The fish swam away.")
                }
            }
        }
        if keep {
            self.me.fish = Some(f)
        }
    }
    /// Town bugs available now (Rain Snail only in rain; Gullrock-only bugs excluded).
    pub fn bug_pool(rain: bool, mo: u32, hr: u32) -> Vec<u16> {
        DEFS.iter()
            .filter(|d| d.k == K::Bug && d.hab == 0 && (d.id != 8 || rain) && avail(d, mo, hr))
            .map(|d| d.id)
            .collect()
    }
    pub fn islet_bug_pool(mo: u32, hr: u32) -> Vec<u16> {
        DEFS.iter().filter(|d| d.k == K::Bug && d.hab == 4 && avail(d, mo, hr)).map(|d| d.id).collect()
    }
    /// Fish for a water tile: hab 1 river / 2 sea; river tiles near the sea (`mouth`) also hold hab 3 fish.
    pub fn fish_pool(hab: u8, mouth: bool, mo: u32, hr: u32) -> Vec<u16> {
        DEFS.iter()
            .filter(|d| d.k == K::Fish && (d.hab == hab || (mouth && d.hab == 3)) && avail(d, mo, hr))
            .map(|d| d.id)
            .collect()
    }
    fn sim_life(&mut self, dt: f32) {
        if self.ibubbles.len() < 2
            && std::iter::once(&self.me).chain(self.others.iter()).any(|p| p.scene == Scene::Islet)
            && gen_range(0, 50) == 0
        {
            let t = (gen_range(0, IW as i32), gen_range(0, IH as i32));
            if tile_at_i(&self.imap, t.0, t.1) == SEA && (t.0 - 7).abs() + (t.1 - 6).abs() < 10 {
                self.ibubbles.push((tc(t), [74, 74, 30, 72][gen_range(0, 4)]))
            }
        }
        let on_islet = std::iter::once(&self.me).chain(self.others.iter()).any(|p| p.scene == Scene::Islet);
        if !on_islet {
            self.ibugs.clear()
        } else {
            if self.ibugs.len() < 2 && gen_range(0, 120) == 0 {
                let t = (gen_range(1, IW as i32 - 1), gen_range(1, IH as i32 - 1));
                if tile_at_i(&self.imap, t.0, t.1) == GRASS {
                    let av = Self::islet_bug_pool(self.mo, self.hr);
                    if !av.is_empty() {
                        self.ibugs.push((av[gen_range(0, av.len())], tc(t), gen_range(0., 6.28)))
                    }
                }
            }
            let t = self.t;
            for b in &mut self.ibugs {
                b.1 += vec2((t * 2. + b.2).sin(), (t * 1.3 + b.2).cos()) * 0.4 * dt
            }
        }
        let anchors: Vec<Vec2> = std::iter::once(&self.me)
            .chain(self.others.iter())
            .filter(|p| p.scene == Scene::Town)
            .map(|p| p.pos)
            .collect();
        if anchors.is_empty() {
            return;
        }
        let fishing = std::iter::once(&self.me).chain(self.others.iter()).any(|p| p.fish.is_some());
        let (p, mo, hr, t) = (anchors[gen_range(0, anchors.len())], self.mo, self.hr, self.t);
        let map = &self.map;
        if !fishing {
            self.shadows.retain(|s| anchors.iter().any(|a| s.pos.distance(*a) < 18.))
        }
        self.bugs.retain(|b| anchors.iter().any(|a| b.1.distance(*a) < 16.));
        if self.shadows.len() < 4 && gen_range(0, 30) == 0 {
            let tt = tile_of(p + vec2(gen_range(-11., 11.), gen_range(-11., 11.)));
            let c = tile_at(map, tt.0, tt.1);
            if matches!(c, RIVER | SEA) {
                let hab = if c == SEA { 2 } else { 1 };
                let mouth =
                    c == RIVER && (-3..=3).any(|dx| (-3..=3).any(|dz| tile_at(map, tt.0 + dx, tt.1 + dz) == SEA));
                let av = Self::fish_pool(hab, mouth, mo, hr);
                if !av.is_empty() {
                    let d = loop {
                        let d = av[gen_range(0, av.len())];
                        if def(d).sell < 1000 || gen_range(0, 4) == 0 {
                            break d;
                        }
                    };
                    self.shadows.push(Shadow { pos: tc(tt), def: d, vel: Vec2::ZERO })
                }
            }
        }
        let att: Vec<usize> = std::iter::once(&self.me)
            .chain(self.others.iter())
            .filter_map(|p| p.fish.as_ref().and_then(|f| f.fish))
            .collect();
        for (i, s) in self.shadows.iter_mut().enumerate() {
            if att.contains(&i) {
                continue;
            }
            if gen_range(0, 60) == 0 {
                s.vel = vec2(gen_range(-0.5, 0.5), gen_range(-0.5, 0.5))
            }
            let n = s.pos + s.vel * dt;
            let nt = tile_of(n);
            if matches!(tile_at(map, nt.0, nt.1), RIVER | SEA) {
                s.pos = n
            } else {
                s.vel = -s.vel
            }
        }
        self.bubbles.retain(|b| anchors.iter().any(|a| b.0.distance(*a) < 20.));
        if self.bubbles.len() < 3 && gen_range(0, 40) == 0 {
            let tt = tile_of(p + vec2(gen_range(-9., 9.), gen_range(-6., 6.)));
            if tile_at(map, tt.0, tt.1) == SEA {
                let d = [70, 70, 71, 71, 72, 73][gen_range(0, 6)];
                self.bubbles.push((tc(tt), d))
            }
        }
        if self.bugs.len() < 4 && gen_range(0, if rainy(self.w.day) { 400 } else { 90 }) == 0 {
            let tt = tile_of(p + vec2(gen_range(-9., 9.), gen_range(-9., 9.)));
            if matches!(tile_at(map, tt.0, tt.1), GRASS | FLOWER) {
                let av = Self::bug_pool(rainy(self.w.day), mo, hr);
                if !av.is_empty() {
                    self.bugs.push((av[gen_range(0, av.len())], tc(tt), gen_range(0., 6.28)))
                }
            }
        }
        for b in &mut self.bugs {
            b.1 += vec2((t * 2. + b.2).sin(), (t * 1.3 + b.2).cos()) * 0.5 * dt
        }
    }
    fn sim_npcs(&mut self, dt: f32) {
        let talkers: Vec<(usize, Vec2, bool)> = std::iter::once(&self.me)
            .chain(self.others.iter())
            .filter_map(|p| {
                if let Ui::Talk { npc: Some(j), .. } = p.ui {
                    Some((j, p.pos, p.scene == Scene::Town))
                } else {
                    None
                }
            })
            .collect();
        for i in 0..4 {
            if self.visit.is_some_and(|v| v.0 == i) {
                continue;
            }
            if let Some(spot) = self.seek_hide_position(i) {
                if self.npcs[i].inside {
                    self.npcs[i].inside = false;
                    self.npcs[i].pos = door_stand(HOUSES[i]);
                    self.npcs[i].path.clear();
                }
                self.npcs[i].moving = false;
                if talkers.iter().any(|t| t.0 == i) {
                    continue;
                }
                if self.npcs[i].pos.distance(spot) < 0.05 {
                    self.npcs[i].path.clear();
                    continue;
                }
                let tile = tile_of(spot);
                if self.npcs[i].path.last() != Some(&tile) {
                    self.npcs[i].path = self.bfs(tile_of(self.npcs[i].pos), tile, true).unwrap_or_default();
                }
                self.follow(i, dt);
                continue;
            }
            if let Some(&(a, b, _)) = self.chats.iter().find(|c| c.0 == i || c.1 == i) {
                let o = if a == i { b } else { a };
                let d = self.npcs[o].pos - self.npcs[i].pos;
                if d.length() > 0.01 {
                    self.npcs[i].yaw = yaw_of(d)
                }
                self.npcs[i].moving = false;
                continue;
            }
            let loc = self.loc(i);
            {
                let n = &mut self.npcs[i];
                n.happy = (n.happy - dt).max(0.);
                n.moving = false;
            }
            if let Some(&(_, pp, town)) = talkers.iter().find(|t| t.0 == i) {
                let d = pp - self.npcs[i].pos;
                if town && d.length() > 0.01 {
                    self.npcs[i].yaw = yaw_of(d)
                }
                continue;
            }
            if loc == Loc::Out {
                if self.npcs[i].inside {
                    let n = &mut self.npcs[i];
                    n.inside = false;
                    n.pos = door_stand(HOUSES[i]);
                    n.path.clear();
                    n.wait = 1.;
                    continue;
                }
                if self.npcs[i].wait > 0. {
                    self.npcs[i].wait -= dt;
                    continue;
                }
                if self.npcs[i].path.is_empty() {
                    let cur = tile_of(self.npcs[i].pos);
                    let mut found = None;
                    let buddy = (0..4)
                        .filter(|&j| j != i && !self.npcs[j].inside)
                        .nth(gen_range(0, 3))
                        .filter(|_| gen_range(0, 3) == 0);
                    if let Some(j) = buddy {
                        let bt = tile_of(self.npcs[j].pos);
                        for t in [(bt.0 + 1, bt.1), (bt.0 - 1, bt.1), (bt.0, bt.1 + 1)] {
                            if self.walk_tile(t.0, t.1) {
                                if let Some(p) = self.bfs(cur, t, true) {
                                    found = Some(p);
                                    break;
                                }
                            }
                        }
                    }
                    if found.is_none() {
                        for _ in 0..6 {
                            let t = (cur.0 + gen_range(-6, 7), cur.1 + gen_range(-6, 7));
                            if self.walk_tile(t.0, t.1) {
                                if let Some(p) = self.bfs(cur, t, true) {
                                    found = Some(p);
                                    break;
                                }
                            }
                        }
                    }
                    match found {
                        Some(p) => self.npcs[i].path = p,
                        None => self.npcs[i].wait = 1.,
                    }
                    continue;
                }
                if self.follow(i, dt) {
                    self.npcs[i].wait = gen_range(1.5, 5.)
                } // idle pause between strolls
            } else if !self.npcs[i].inside {
                let door = door_tile(HOUSES[i]);
                let cur = tile_of(self.npcs[i].pos);
                if cur == door {
                    self.npcs[i].inside = true
                } else if self.npcs[i].path.last() != Some(&door) {
                    match self.bfs(cur, door, true) {
                        Some(p) => self.npcs[i].path = p,
                        None => self.npcs[i].inside = true,
                    }
                } else {
                    self.follow(i, dt);
                }
            }
        }
    }
    /// Residents who meet outdoors sometimes stop and chat (speech bubbles), then carry on.
    pub fn sim_chats(&mut self, dt: f32) {
        for c in &mut self.chats {
            c.2 -= dt
        }
        let ended: Vec<(usize, usize, f32)> = self.chats.iter().copied().filter(|c| c.2 <= 0.).collect();
        for c in ended {
            self.npcs[c.0].wait = 1.;
            self.npcs[c.1].wait = 1.;
        }
        self.chats.retain(|c| c.2 > 0.);
        if gen_range(0, 60) != 0 {
            return;
        }
        for a in 0..4 {
            for b in a + 1..4 {
                let (na, nb) = (&self.npcs[a], &self.npcs[b]);
                if self.seek_hide_position(a).is_some()
                    || self.seek_hide_position(b).is_some()
                    || na.inside
                    || nb.inside
                    || self.loc(a) != Loc::Out
                    || self.loc(b) != Loc::Out
                    || self.chats.iter().any(|c| [c.0, c.1].contains(&a) || [c.0, c.1].contains(&b))
                {
                    continue;
                }
                if na.pos.distance(nb.pos) < 2.6 {
                    for k in [a, b] {
                        self.npcs[k].path.clear();
                        self.npcs[k].wait = 5.
                    }
                    self.chats.push((a, b, 5.));
                    return;
                }
            }
        }
    }
    fn follow(&mut self, i: usize, dt: f32) -> bool {
        let Some(&nt) = self.npcs[i].path.first() else { return true };
        if !self.walk_tile(nt.0, nt.1) {
            self.npcs[i].path.clear();
            return true;
        }
        let n = &mut self.npcs[i];
        let to = tc(nt) - n.pos;
        let st = 1.3 * dt;
        if to.length() <= st {
            n.pos = tc(nt);
            n.path.remove(0);
        } else {
            n.pos += to.normalize() * st;
            n.yaw = yaw_of(to)
        }
        n.moving = true;
        n.anim += dt * 7.;
        n.path.is_empty()
    }
    /// 4-neighbour BFS. `town` = outdoor walkability, else the current scene's passability.
    pub fn bfs(&self, a: (i32, i32), b: (i32, i32), town: bool) -> Option<Vec<(i32, i32)>> {
        if a == b {
            return Some(vec![]);
        }
        let pass = |t: (i32, i32)| match (town, self.me.scene) {
            (false, Scene::Room(r)) => !self.room_block(r, t.0, t.1, 0),
            (false, Scene::Islet) => matches!(tile_at_i(&self.imap, t.0, t.1), GRASS | SAND | BRIDGE) && t != ISTALL,
            _ => self.walk_tile(t.0, t.1),
        };
        if !pass(b) {
            return None;
        }
        let mut prev = HashMap::from([(a, a)]);
        let mut q = VecDeque::from([a]);
        while let Some(c) = q.pop_front() {
            if c == b {
                break;
            }
            if prev.len() > 2500 {
                return None;
            }
            for d in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (c.0 + d.0, c.1 + d.1);
                if !prev.contains_key(&n) && pass(n) {
                    prev.insert(n, c);
                    q.push_back(n)
                }
            }
        }
        if !prev.contains_key(&b) {
            return None;
        }
        let mut path = vec![b];
        let mut c = b;
        while prev[&c] != a {
            c = prev[&c];
            path.push(c)
        }
        path.reverse();
        Some(path)
    }

    // ---- per-frame update through the input layer ----
    /// Single-player convenience: shared tick + update of the active player.
    pub fn update(&mut self, inp: &Inp, dt: f32, now: NaiveDateTime) -> bool {
        let f = self.tick(dt, now);
        self.update_player(inp, dt);
        f
    }
    /// Shared per-frame simulation (clock, daily refresh, residents, wildlife, effects). Call once per frame.
    pub fn tick(&mut self, dt: f32, now: NaiveDateTime) -> bool {
        self.clock_ms = now.and_utc().timestamp_millis();
        self.t += dt;
        self.hr = now.hour();
        self.mo = now.month();
        self.dom = now.day();
        self.yr = now.year();
        let fresh = self.catch_up(now);
        for e in &mut self.fx {
            e.1 += dt
        }
        self.fx.retain(|e| e.1 < 0.8);
        for s in &mut self.shake {
            s.1 -= dt
        }
        self.shake.retain(|s| s.1 > 0.);
        for h in &mut self.hearts {
            h.1 += dt;
            if h.1 > 0. {
                h.0.y += dt * 0.8
            }
        }
        self.hearts.retain(|h| h.1 < 1.6);
        self.sim_npcs(dt);
        self.sim_chats(dt);
        self.sim_life(dt);
        self.sim_visit(dt);
        self.sim_act(dt);
        fresh
    }
    /// Per-player update for the active player (`activate` first): its own input, camera, UI, movement, actions.
    pub fn update_player(&mut self, inp: &Inp, dt: f32) {
        self.social_tick();
        self.me.swing = (self.me.swing - dt).max(0.);
        self.act_step();
        if self.w.p.task_day != self.w.day {
            self.assign_tasks()
        }
        if self.w.wtask.is_none() {
            self.assign_weekly()
        }
        if self.me.spd > 0.1 {
            self.me.anim += dt * self.me.spd * 3.2
        }
        self.sim_fish(dt);
        let pp = self.ppos();
        self.me.cam = self.me.cam.lerp(pp, 1. - (-12. * dt).exp());
        if let Some((t, to, done)) = self.me.trans {
            let t = t + dt;
            let mut done = done;
            if t >= 0.22 && !done {
                self.swap(to);
                done = true
            }
            self.me.trans = if t >= 0.45 { None } else { Some((t, to, done)) };
            self.me.spd = 0.;
            return;
        }
        if self.ui_input(inp) {
            self.me.spd = 0.;
            return;
        }
        if self.me.placing.is_some() {
            self.place_input(inp);
            self.me.spd = 0.;
            return;
        }
        if let Some(idx) = self.me.mplace {
            if inp.p(KeyCode::Escape) {
                self.me.mplace = None;
                self.note("Placement cancelled.")
            } else if self.me.scene == Scene::Town && (inp.p(KeyCode::Enter) || inp.p(KeyCode::KpEnter)) {
                match self.mp_start(idx) {
                    Ok(()) => {
                        self.me.mplace = None;
                        self.snd.push(Snd::Blip);
                        self.note(format!("The {} site is set! Anyone can donate at its sign.", MPROJECTS[idx].0))
                    }
                    Err(e) => self.note(e),
                }
            }
        }
        use KeyCode::*;
        let mut d = Vec2::ZERO;
        if inp.d(W) || inp.d(Up) {
            d.y -= 1.
        }
        if inp.d(S) || inp.d(Down) {
            d.y += 1.
        }
        if inp.d(A) || inp.d(Left) {
            d.x -= 1.
        }
        if inp.d(D) || inp.d(Right) {
            d.x += 1.
        }
        self.step_move(d, inp.d(LeftShift) || inp.d(RightShift), dt);
        if self.me.trans.is_some() {
            return;
        }
        if inp.p(E) {
            self.interact()
        }
        if inp.p(Space) {
            self.use_tool()
        }
        if inp.p(Tab) || inp.p(Q) {
            self.cycle_tool()
        }
        if inp.p(I) {
            self.me.ui = Ui::Inv(0)
        }
        if inp.p(J) {
            self.me.ui = Ui::Journal
        }
        if inp.p(F) || inp.p(G) {
            match self.target() {
                Some(Tg::Furn(uid, _)) if self.me.scene == Scene::Room(self.home()) && uid != 0 => {
                    if inp.p(F) {
                        self.start_move(uid)
                    } else {
                        self.pick_furn(uid)
                    }
                }
                _ => self.note("Face your own furniture to move (F) or pick it up (G)."),
            }
        }
        if inp.p(Escape) && self.me.fish.is_some() {
            self.me.fish = None;
            self.note("You reel in your line.")
        }
    }
    fn ui_input(&mut self, inp: &Inp) -> bool {
        use KeyCode::*;
        if self.me.ui == Ui::Play {
            return false;
        }
        let (up, down) = (inp.p(Up) || inp.p(W), inp.p(Down) || inp.p(S));
        let (left, right) = (inp.p(Left) || inp.p(A), inp.p(Right) || inp.p(D));
        let ok = inp.p(Enter) || inp.p(KpEnter) || inp.p(E);
        let esc = inp.p(Escape);
        if let Ui::Talk { npc: Some(j), .. } = self.me.ui {
            if inp.p(X) && self.w.moving[j] != 0 {
                self.w.moving[j] = 0;
                self.w.arrived[j] = self.w.day;
                self.w.p.npcs[j].friend += 1;
                let r = self.res(j);
                self.me.ui = Ui::Talk {
                    npc: Some(j),
                    who: format!("{} the {}", r.name, r.species),
                    lines: vec!["You really want me to stay? ...Okay. I'll stay!".into()],
                    i: 0,
                };
                return true;
            }
        }
        if let Ui::Talk { npc: Some(j), .. } = self.me.ui {
            if inp.p(G) {
                if self.w.p.inv.is_empty() {
                    self.note("You have nothing to give.")
                } else {
                    self.me.ui = Ui::Gift(j, 0)
                }
                return true;
            }
        }
        if let Ui::Talk { lines, i, .. } = &mut self.me.ui {
            if ok || inp.p(Space) {
                *i += 1;
                if *i >= lines.len() {
                    self.me.ui =
                        if self.pell_talked && self.me.scene == Scene::Islet { Ui::Travel(false) } else { Ui::Play };
                    self.pell_talked = false
                }
            } else if esc {
                self.me.ui = Ui::Play
            }
            return true;
        }
        let n = self.w.p.inv.len();
        let mv = |s: usize, len: usize| {
            if up && s > 0 {
                s - 1
            } else if down && s + 1 < len {
                s + 1
            } else {
                s.min(len.saturating_sub(1))
            }
        };
        match self.me.ui {
            Ui::Invitation => {
                if esc {
                    if let Some(v) = &mut self.w.p.invitation {
                        v.state = 4;
                    }
                    self.me.ui = Ui::Play;
                } else if ok {
                    let r = self.accept_social();
                    self.me.ui = Ui::Play;
                    if let Err(e) = r {
                        self.note(e)
                    }
                }
            }
            Ui::Contest(s) => {
                let slots = self.contest_slots();
                let s = mv(s, slots.len());
                self.me.ui = Ui::Contest(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    if let Some(&slot) = slots.get(s) {
                        match self.contest_submit(slot) {
                            Ok(score) => self.note(format!("New best: {score} points. Catch released.")),
                            Err(e) => self.note(e),
                        }
                    } else {
                        self.note("Bring a measured catch. Its size is shown in pockets.")
                    }
                }
            }
            Ui::Inv(s) => {
                let s = mv(s, n);
                self.me.ui = Ui::Inv(s);
                if esc || inp.p(I) {
                    self.me.ui = Ui::Play
                } else if ok && s < n {
                    let sl = self.w.p.inv[s];
                    if def(sl.def).k == crate::data::K::Cloth {
                        self.me.ui = Ui::Play;
                        self.wear(sl.uid)
                    } else if def(sl.def).k != crate::data::K::Furn {
                        self.note(format!("{} sells for {} coins at the shop.", def(sl.def).name, def(sl.def).sell))
                    } else if self.me.scene == Scene::Room(self.home()) {
                        self.me.ui = Ui::Play;
                        self.start_place(sl.uid)
                    } else {
                        self.note("Furniture can be placed inside your home.")
                    }
                }
            }
            Ui::Sell(s) => {
                let s = mv(s, n + 4);
                if ok && s == n + 3 {
                    self.me.ui = Ui::Keeper(0);
                    return true;
                }
                if ok && s == n + 1 {
                    self.me.ui = Ui::Catalog(0);
                    return true;
                }
                if ok && s == n + 2 {
                    self.me.ui = Ui::Tint(0);
                    return true;
                }
                self.me.ui = Ui::Sell(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s < n {
                    self.sell_slot(s)
                } else if ok && s == n {
                    if self.w.p.debt > 0 {
                        self.pay_debt()
                    } else if (self.w.p.home as usize) < EXPAND_COST.len() {
                        self.me.ui = Ui::ConfirmExpand
                    } else {
                        self.note("Your home is already as big as it gets!")
                    }
                } else if inp.p(A) || inp.p(X) {
                    self.sell_all()
                }
            }
            Ui::Confirm(d) => {
                if ok || inp.p(Y) {
                    self.me.ui = Ui::Play;
                    match self.buy(d) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note("Thank you for your purchase!")
                        }
                        Err(e) => self.note(e),
                    }
                } else if esc || inp.p(N) {
                    self.me.ui = Ui::Play
                }
            }
            Ui::Storage(c, s) => {
                let c = if left {
                    0
                } else if right {
                    1
                } else {
                    c
                };
                let len = if c == 0 { n } else { self.w.p.storage.len() };
                let s = mv(s, len);
                self.me.ui = Ui::Storage(c, s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s < len {
                    self.transfer(c, s)
                }
            }
            Ui::Journal => {
                if esc || inp.p(J) {
                    self.me.ui = Ui::Play
                }
                if inp.p(Tab) || inp.p(R) {
                    self.me.jpage = (self.me.jpage + 1) % 6;
                    self.me.jcol = 0;
                }
                if self.me.jpage == 4 {
                    if up {
                        self.me.jcol = self.me.jcol.saturating_sub(1);
                    }
                    if down {
                        self.me.jcol = (self.me.jcol + 1).min(GUIDE.len() as u8 - 1);
                    }
                }
                if self.me.jpage == 0 {
                    if left {
                        self.me.jcol = (self.me.jcol + 2) % 3
                    }
                    if right {
                        self.me.jcol = (self.me.jcol + 1) % 3
                    }
                }
                let (mv0, sv0, mu0) = (self.music_vol, self.sfx_vol, self.muted);
                if self.me.jpage == 2 {
                    if left {
                        self.music_vol = (self.music_vol - 0.1).max(0.)
                    }
                    if right {
                        self.music_vol = (self.music_vol + 0.1).min(1.)
                    }
                    if down {
                        self.sfx_vol = (self.sfx_vol - 0.1).max(0.)
                    }
                    if up {
                        self.sfx_vol = (self.sfx_vol + 0.1).min(1.)
                    }
                    if inp.p(X) {
                        self.muted = !self.muted
                    }
                    self.music_vol = (self.music_vol * 10.).round() / 10.;
                    self.sfx_vol = (self.sfx_vol * 10.).round() / 10.;
                }
                if (mv0, sv0, mu0) != (self.music_vol, self.sfx_vol, self.muted) {
                    self.settings_dirty = true
                }
            }
            Ui::Gift(j, s) => {
                let s = mv(s, n);
                self.me.ui = Ui::Gift(j, s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s < n {
                    self.give_gift(j, s)
                }
            }
            Ui::MPick(s) => {
                let s = mv(s, MPROJECTS.len());
                self.me.ui = Ui::MPick(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.me.ui = Ui::Play;
                    self.me.mplace = Some(s);
                    if self.me.scene != Scene::Town {
                        self.go(Scene::Town)
                    }
                    self.note("Face a spot outside (green = OK), Enter places.")
                }
            }
            Ui::Paint(s) => {
                let s = mv(s, PAINTS.len());
                self.me.ui = Ui::Paint(s);
                if esc {
                    self.me.ui = Ui::Civic(0)
                } else if ok {
                    match self.repaint(s) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.me.ui = Ui::Play;
                            self.note(format!("Your house will be {} by the time you step outside!", PAINTS[s].0))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Civic(s) => {
                let s = mv(s, 4);
                self.me.ui = Ui::Civic(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    match s {
                        0 => self.me.ui = Ui::Policy(0),
                        1 => {
                            if self.me.id != LEADER {
                                self.me.ui = Ui::Play;
                                self.note("Only the town leader can start public projects.")
                            } else if self.w.mproject.is_some() {
                                self.me.ui = Ui::Play;
                                self.note("A funded project is already underway.")
                            } else {
                                self.me.ui = Ui::MPick(0)
                            }
                        }
                        3 => self.me.ui = Ui::Paint(0),
                        _ => {
                            let lines = self.board_lines();
                            self.me.ui = Ui::Talk { npc: None, who: ADMIN.into(), lines, i: 0 }
                        }
                    }
                }
            }
            Ui::Policy(s) => {
                let s = mv(s, POLICIES.len() + 1);
                self.me.ui = Ui::Policy(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.me.ui = Ui::PolicyOk(s)
                }
            }
            Ui::PolicyOk(s) => {
                if esc {
                    self.me.ui = Ui::Policy(s)
                } else if ok {
                    let r = self.enact((s < POLICIES.len()).then_some(s));
                    self.me.ui = Ui::Play;
                    match r {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note(self.pol().map_or("Policy repealed. Normal hours apply.".to_string(), |p| {
                                format!("{} is now in effect.", p.name)
                            }))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Act(s) => {
                let s = mv(s, ACTS.len());
                self.me.ui = Ui::Act(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.me.ui = Ui::Play;
                    match self.act_start(s) {
                        Ok(()) => {
                            self.snd.push(Snd::Blip);
                            self.note(format!("{} started!", ACTS[s].0))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Keeper(s) => {
                let items = self.keeper_items();
                let s = mv(s, items.len());
                self.me.ui = Ui::Keeper(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    let a = items[s].1;
                    match a {
                        0 => {
                            self.me.ui = Ui::Talk {
                                npc: None,
                                who: "Mr. Tumble the tortoise".into(),
                                lines: KEEPER.iter().map(|s| s.to_string()).collect(),
                                i: 0,
                            }
                        }
                        1..=4 => {
                            if self.w.p.money < SEED_PRICE {
                                self.note("Not enough coins.")
                            } else if self.add(SEEDS[a as usize - 1]) {
                                self.w.p.money -= SEED_PRICE;
                                self.w.spent += SEED_PRICE;
                                self.snd.push(Snd::Coin);
                                self.note(format!("Bought {}.", def(SEEDS[a as usize - 1]).name))
                            }
                        }
                        5 => {
                            let p = def(CAN).buy;
                            if self.w.p.money < p {
                                self.note("Not enough coins.")
                            } else {
                                self.w.p.money -= p;
                                self.w.spent += p;
                                self.w.p.tools.push(CAN);
                                self.w.p.equip = Some(CAN);
                                self.snd.push(Snd::Coin);
                                self.note("Bought a Watering Can (equipped). Use it with Space / X.")
                            }
                        }
                        _ => {
                            let t = UPGRADABLE[(a - 10) as usize];
                            match self.upgrade_tool(t) {
                                Ok(lv) => {
                                    self.snd.push(Snd::Coin);
                                    self.note(format!(
                                        "Your {} is now a {}{}!",
                                        def(t).name,
                                        LV_NAME[lv as usize],
                                        def(t).name
                                    ))
                                }
                                Err(e) => self.note(e),
                            }
                        }
                    }
                }
            }
            Ui::Camp(s) => {
                let s = mv(s, 5);
                self.me.ui = Ui::Camp(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    let v = &VISITORS[self.visitor()];
                    let who = format!("{} the {}", v.name, v.species);
                    match s {
                        0..=2 => match self.camp_buy(s) {
                            Ok(n) => {
                                self.snd.push(Snd::Coin);
                                self.note(format!("Bought {n}."))
                            }
                            Err(e) => self.note(e),
                        },
                        3 => match self.camp_favor() {
                            Ok(n) => {
                                self.snd.push(Snd::Coin);
                                self.me.ui = Ui::Talk {
                                    npc: None,
                                    who,
                                    lines: vec![format!("You found them! Please take this {n} as thanks.")],
                                    i: 0,
                                }
                            }
                            Err(e) => self.me.ui = Ui::Talk { npc: None, who, lines: vec![e], i: 0 },
                        },
                        _ => {
                            self.ev(Ev::Camp, 1);
                            let lines = v.lines.iter().map(|l| l.to_string()).collect();
                            self.me.ui = Ui::Talk { npc: None, who, lines, i: 0 }
                        }
                    }
                }
            }
            Ui::Kiosk(s) => {
                let s = mv(s, 4 + KIOSK_ITEMS.len());
                self.me.ui = Ui::Kiosk(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    if s < 4 {
                        match self.claim(s) {
                            Ok(c) => {
                                self.snd.push(Snd::Coin);
                                self.note(format!("Claimed {c} coupons!"))
                            }
                            Err(e) => self.note(e),
                        }
                    } else {
                        match self.kiosk_buy(s - 4) {
                            Ok(n) => {
                                self.snd.push(Snd::Coin);
                                self.note(format!("Traded coupons for {n}."))
                            }
                            Err(e) => self.note(e),
                        }
                    }
                }
            }
            Ui::Title(s) => {
                let s = mv(s, 5);
                self.me.ui = Ui::Title(s);
                if ok {
                    match s {
                        0 => {
                            self.me.ui = Ui::Play;
                            if self.fresh {
                                self.note(
                                    "Welcome to Hollowbrook! Press J for the journal: controls and a guide are inside.",
                                )
                            }
                        }
                        1 => {
                            self.me.ui = Ui::Play;
                            self.net_req = 1
                        }
                        2 => {
                            self.me.ui = Ui::Play;
                            self.net_req = 2
                        }
                        3 => self.me.ui = Ui::Help,
                        _ => self.quit = true,
                    }
                }
            }
            Ui::Help => {
                if ok || esc {
                    self.me.ui = Ui::Title(3)
                }
            }
            Ui::Fund => {
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    match self.mp_fund(DONATE_STEP) {
                        Ok(a) => {
                            self.snd.push(Snd::Coin);
                            self.note(format!("Donated {a} coins."))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::ConfirmExpand => {
                if ok || inp.p(Y) {
                    self.me.ui = Ui::Play;
                    match self.expand_home() {
                        Ok(()) => {
                            self.snd.push(Snd::Door);
                            self.note(format!("Your home was expanded! Balance: {} coins, no deadline.", self.w.p.debt))
                        }
                        Err(e) => self.note(e),
                    }
                } else if esc || inp.p(N) {
                    self.me.ui = Ui::Play
                }
            }
            Ui::Travel(go_out) => {
                if ok || inp.p(Y) {
                    self.me.ui = Ui::Play;
                    if go_out {
                        if self.w.p.ferry > 0 {
                            self.w.p.ferry -= 1;
                            self.note("Ferry Pass used. Enjoy the trip!");
                            self.go(Scene::Islet)
                        } else if self.w.p.money < FARE {
                            self.note("Not enough coins for the fare.")
                        } else {
                            self.w.p.money -= FARE;
                            self.go(Scene::Islet)
                        }
                    } else {
                        self.go(Scene::Town)
                    }
                } else if esc || inp.p(N) {
                    self.me.ui = Ui::Play
                }
            }
            Ui::IStall(s) => {
                let s = mv(s, ISTALL_ITEMS.len());
                self.me.ui = Ui::IStall(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    match self.istall_buy(s) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note(format!("Traded for a {}!", def(ISTALL_ITEMS[s].0).name))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Tint(s) => {
                let len = self.tint_targets().len() + 2;
                let s = mv(s, len);
                self.me.ui = Ui::Tint(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    match self.customize(s) {
                        Ok(m) => {
                            self.snd.push(Snd::Coin);
                            self.note(m)
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Catalog(s) => {
                let s = mv(s, self.w.p.catalog.len());
                self.me.ui = Ui::Catalog(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s < self.w.p.catalog.len() {
                    let d = self.w.p.catalog[s];
                    match self.order(d) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note(format!("Ordered a {}.", def(d).name))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            Ui::Mail(s) => {
                let s = mv(s, self.w.p.mail.len());
                self.me.ui = Ui::Mail(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok {
                    self.open_letter(s)
                }
            }
            Ui::Rack(s) => {
                let s = mv(s, CLOTHES.len() + 1);
                self.me.ui = Ui::Rack(s);
                if esc {
                    self.me.ui = Ui::Play
                } else if ok && s == CLOTHES.len() {
                    if self.w.p.money < HAIR_FEE {
                        self.note("Not enough coins.")
                    } else {
                        self.w.p.money -= HAIR_FEE;
                        self.w.p.hair = (self.w.p.hair + 1) % HAIRS.len() as u8;
                        self.snd.push(Snd::Coin);
                        self.note("New hairstyle!")
                    }
                } else if ok {
                    match self.buy_cloth(CLOTHES[s]) {
                        Ok(()) => {
                            self.snd.push(Snd::Coin);
                            self.note(format!("Bought the {}. Open your pockets to wear it.", def(CLOTHES[s]).name))
                        }
                        Err(e) => self.note(e),
                    }
                }
            }
            _ => {}
        }
        true
    }
    fn place_input(&mut self, inp: &Inp) {
        use KeyCode::*;
        let dims = self.rdims(self.home());
        let Some(p) = self.me.placing.as_mut() else { return };
        let df = def(p.f.def);
        if inp.p(A) || inp.p(Left) {
            p.f.x -= 1
        }
        if inp.p(D) || inp.p(Right) {
            p.f.x += 1
        }
        if !df.wall {
            if inp.p(W) || inp.p(Up) {
                p.f.z -= 1
            }
            if inp.p(S) || inp.p(Down) {
                p.f.z += 1
            }
            if inp.p(R) && df.rot {
                p.f.rot = (p.f.rot + 1) % 4
            }
        }
        let (w, d) = dims;
        p.f.x = p.f.x.clamp(-1, w);
        p.f.z = if df.wall { -1 } else { p.f.z.clamp(-1, d) };
        if inp.p(Enter) || inp.p(KpEnter) {
            match self.place_confirm() {
                Ok(()) => {
                    self.snd.push(Snd::Blip);
                    self.note("Placed!")
                }
                Err(e) => self.note(e),
            }
        } else if inp.p(Escape) {
            self.place_cancel();
            self.note("Cancelled.")
        }
    }
}
