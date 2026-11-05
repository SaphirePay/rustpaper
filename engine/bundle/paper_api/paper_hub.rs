// The plugin layer's part of the network hub, below the game binary. The hub has no `Game`, so what needs one waits
// in a queue for the end of the tick; what the plugins say on chat and system lines comes through the hooks the
// bridge sets once a plugin loads (`set_chat_hooks`, `set_decorator`), as this crate can't see the binary's.
use crate::mobs::Mob;
use horizon_world::nbt::Tag;
use std::{
    collections::HashMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

/// A game message: its English, the component a client gets, and whether it's for the action bar.
pub struct GameMessage<'a> {
    pub text: &'a str,
    pub component: &'a Tag,
    pub overlay: bool,
}

/// `AsyncPlayerChatEvent` (false holds the game's line back) and a system line's say (false keeps it in).
type ChatHooks = (fn(u128, &str, &str) -> bool, fn(&GameMessage<'_>) -> bool);
static CHAT_HOOKS: OnceLock<ChatHooks> = OnceLock::new();
/// `AsyncChatDecorateEvent`: the sender and the text so far.
static DECORATOR: OnceLock<fn(Option<u128>, Tag) -> Tag> = OnceLock::new();

/// Set once a plugin loads.
pub fn set_chat_hooks(chat: fn(u128, &str, &str) -> bool, game: fn(&GameMessage<'_>) -> bool) {
    let _ = CHAT_HOOKS.set((chat, game));
}

/// Set once the player family hears its first packet.
pub fn set_decorator(decorate: fn(Option<u128>, Tag) -> Tag) {
    let _ = DECORATOR.set(decorate);
}

thread_local! {
    /// The guest a host's menu engine is working for: `Menu` itself doesn't know.
    static MENU_PLAYER: std::cell::Cell<Option<u128>> = const { std::cell::Cell::new(None) };
}

pub fn set_menu_player(player: Option<u128>) {
    MENU_PLAYER.with(|cell| cell.set(player));
}

pub fn menu_player() -> Option<u128> {
    MENU_PLAYER.with(std::cell::Cell::get)
}

/// The chat text with its decorators run: `None` when nothing changed it.
pub fn decorate(sender: u128, content: &str) -> Option<Tag> {
    decorate_as(Some(sender), content)
}

/// `ChatDecorator.decorate`: `None` as the sender is the console's, or a mob's.
pub fn decorate_as(sender: Option<u128>, content: &str) -> Option<Tag> {
    let decorate = DECORATOR.get()?;
    let original = horizon_protocol::text::literal(content);
    let decorated = decorate(sender, original.clone());
    (decorated != original).then_some(decorated)
}

/// What happened in the hub this tick, for the game's end of tick.
pub enum Pending {
    Respawned { player: u128, name: String },
    ChangedLevel { player: u128, name: String, from: String, to: String },
    EffectAdded { effect: String, amplifier: i32 },
    EffectCleared { effect: Option<String> },
    EntityChangedLevel { entity: u32, kind: String, from: String, to: String },
}

static PENDING: Mutex<Vec<Pending>> = Mutex::new(Vec::new());

/// A message the chain took and the filter passed: whether it goes to everyone.
pub fn allow_chat(player: u128, name: &str, message: &str) -> bool {
    CHAT_HOOKS.get().is_none_or(|(chat, _)| chat(player, name, message))
}

/// A system line about to reach everyone: whether it goes.
static SPAWN_LIMITS: std::sync::Mutex<Option<HashMap<String, i32>>> = std::sync::Mutex::new(None);
static SPAWN_LIMITS_SET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `World.setSpawnLimit`: a mob category's cap, from now on. Below zero goes back to the game's own.
pub fn set_spawn_limit(category: &str, limit: i32) {
    let mut limits = SPAWN_LIMITS.lock().unwrap_or_else(|e| e.into_inner());
    let limits = limits.get_or_insert_default();
    if limit < 0 {
        limits.remove(category);
    } else {
        limits.insert(category.to_owned(), limit);
    }
    SPAWN_LIMITS_SET.store(!limits.is_empty(), std::sync::atomic::Ordering::Relaxed);
}

/// What `Category::max` answers.
pub fn spawn_limit(category: impl std::fmt::Debug, value: i32) -> i32 {
    if !SPAWN_LIMITS_SET.load(std::sync::atomic::Ordering::Relaxed) {
        return value;
    }
    SPAWN_LIMITS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|limits| limits.get(&format!("{category:?}")).copied()).unwrap_or(value)
}

/// Join, quit, death: which of the server's lines a plugin may change.
pub const LINE_JOIN: usize = 0;
pub const LINE_QUIT: usize = 1;
pub const LINE_DEATH: usize = 2;

#[derive(Default)]
pub struct Line {
    /// The game's own line, until its event has been heard.
    pub held: Option<Tag>,
    /// What a plugin asked for: `Some(None)` says nothing.
    pub changed: Option<Option<Tag>>,
    /// The event was heard: the line goes out at the end of the tick.
    pub ready: bool,
    /// Ticks held with no event, so a line is never lost.
    pub waited: u32,
}

pub static LINES: Mutex<[Line; 3]> = Mutex::new([Line { held: None, changed: None, ready: false, waited: 0 }, Line { held: None, changed: None, ready: false, waited: 0 }, Line { held: None, changed: None, ready: false, waited: 0 }]);

/// Set while held lines go out.
pub static LINES_LEAVING: AtomicBool = AtomicBool::new(false);

/// The game is about to say a line: true if it's held for its event instead.
pub fn hold_line(english: &str, content: &Tag) -> bool {
    // The line on its way out after its event comes through here too.
    if LINES_LEAVING.load(Ordering::Relaxed) {
        return false;
    }
    let kind = if english.ends_with(" joined the game") {
        LINE_JOIN
    } else if english.ends_with(" left the game") {
        LINE_QUIT
    } else {
        return false;
    };
    let mut lines = LINES.lock().unwrap_or_else(|e| e.into_inner());
    lines[kind].held = Some(content.clone());
    lines[kind].waited = 0;
    true
}

/// A death's line, which comes after its event: what a plugin asked for, if it did.
pub fn take_death_line() -> Option<Option<Tag>> {
    LINES.lock().unwrap_or_else(|e| e.into_inner())[LINE_DEATH].changed.take()
}

/// A created world's dimension type, which is its kind's: a client knows only the registry's.
pub static WORLD_KINDS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

pub fn dimension_type_of(id: &str) -> &str {
    match WORLD_KINDS.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|world| world.0 == id).map(|world| world.1.as_str()) {
        Some("minecraft:the_nether") => "minecraft:the_nether",
        Some("minecraft:the_end") => "minecraft:the_end",
        Some(_) => "minecraft:overworld",
        None => id,
    }
}

/// `Player.hidePlayer`, by player: (who doesn't see, who is hidden). It outlives the hidden player's
/// connection, and goes with the viewer's, as Bukkit's map does.
pub static HIDDEN: Mutex<Vec<(u128, u128)>> = Mutex::new(Vec::new());
/// The same pairs by entity id, for the tracker's range checks.
static HIDDEN_IDS: std::sync::RwLock<Vec<(i32, i32)>> = std::sync::RwLock::new(Vec::new());
static ANY_HIDDEN: AtomicBool = AtomicBool::new(false);
/// `Entity.setVisibleByDefault` and `Player.hideEntity`: (viewer, entity), by id.
static UNSEEN_IDS: std::sync::RwLock<Vec<(i32, i32)>> = std::sync::RwLock::new(Vec::new());
static ANY_UNSEEN: AtomicBool = AtomicBool::new(false);

/// Entities by what a plugin set on them, for the library's code, which has no `persist.raw`: 0, no mob takes this
/// item (`Item.setCanMobPickup`); 1, no friction on it (`Frictional`); 2, this player takes fall damage while allowed
/// to fly (`Player.setFlyingFallDamage`); 3, this player has no part in spawning (`Player.setAffectsSpawning`).
static PLUGIN_MARKS: [std::sync::RwLock<Vec<u32>>; 5] = [const { std::sync::RwLock::new(Vec::new()) }; 5];
/// A player's pose held by `Entity.setPose(pose, true)`, by entity id.
static FIXED_POSES: std::sync::RwLock<Vec<(i32, i32)>> = std::sync::RwLock::new(Vec::new());

pub fn set_fixed_pose(entity: i32, pose: Option<i32>) {
    let mut poses = FIXED_POSES.write().unwrap_or_else(|e| e.into_inner());
    poses.retain(|held| held.0 != entity);
    if let Some(pose) = pose {
        poses.push((entity, pose));
    }
}

/// `Player.setSimulationDistance`, by entity id.
static SIMULATION: std::sync::RwLock<Vec<(i32, i32)>> = std::sync::RwLock::new(Vec::new());

pub fn set_simulation(entity: i32, distance: i32) {
    let mut held = SIMULATION.write().unwrap_or_else(|e| e.into_inner());
    held.retain(|row| row.0 != entity);
    held.push((entity, distance));
}

pub fn simulation_of(entity: i32) -> Option<i32> {
    SIMULATION.read().unwrap_or_else(|e| e.into_inner()).iter().find(|row| row.0 == entity).map(|row| row.1)
}

/// `LivingEntity.setCollidable(false)` of a player, by UUID: what a mob's `pushEntities` asks.
static NO_COLLIDE: std::sync::RwLock<Vec<[i32; 4]>> = std::sync::RwLock::new(Vec::new());

pub fn set_no_collide(uuid: [i32; 4], off: bool) {
    let mut held = NO_COLLIDE.write().unwrap_or_else(|e| e.into_inner());
    held.retain(|known| *known != uuid);
    if off {
        held.push(uuid);
    }
}

pub fn no_collide(uuid: [i32; 4]) -> bool {
    NO_COLLIDE.read().unwrap_or_else(|e| e.into_inner()).contains(&uuid)
}

/// `LivingEntity.setMaximumAir` of a player, by entity id.
static MAX_AIR: std::sync::RwLock<Vec<(i32, i32)>> = std::sync::RwLock::new(Vec::new());

pub fn set_max_air(entity: i32, ticks: i32) {
    let mut held = MAX_AIR.write().unwrap_or_else(|e| e.into_inner());
    held.retain(|row| row.0 != entity);
    held.push((entity, ticks));
}

pub fn max_air(entity: i32) -> i32 {
    MAX_AIR.read().unwrap_or_else(|e| e.into_inner()).iter().find(|row| row.0 == entity).map_or(300, |row| row.1)
}

pub fn fixed_pose(entity: i32) -> Option<i32> {
    FIXED_POSES.read().unwrap_or_else(|e| e.into_inner()).iter().find(|held| held.0 == entity).map(|held| held.1)
}
static ANY_MARK: AtomicBool = AtomicBool::new(false);

pub fn plugin_mark(kind: usize, id: u32, on: bool) {
    let mut marks = PLUGIN_MARKS[kind].write().unwrap_or_else(|e| e.into_inner());
    marks.retain(|marked| *marked != id);
    if on {
        marks.push(id);
        ANY_MARK.store(true, Ordering::Relaxed);
    }
}

pub fn plugin_marked(kind: usize, id: u32) -> bool {
    ANY_MARK.load(Ordering::Relaxed) && PLUGIN_MARKS[kind].read().unwrap_or_else(|e| e.into_inner()).contains(&id)
}

pub fn set_unseen_ids(ids: Vec<(i32, i32)>) {
    ANY_UNSEEN.store(!ids.is_empty(), Ordering::Relaxed);
    *UNSEEN_IDS.write().unwrap_or_else(|e| e.into_inner()) = ids;
}

pub fn set_hidden_ids(ids: Vec<(i32, i32)>) {
    ANY_HIDDEN.store(!ids.is_empty(), Ordering::Relaxed);
    *HIDDEN_IDS.write().unwrap_or_else(|e| e.into_inner()) = ids;
}

/// The tracker's question, asked for every pair of viewer and entity: nothing to look at when nobody is hidden.
pub fn hidden_from(viewer: i32, target: i32) -> bool {
    (ANY_HIDDEN.load(Ordering::Relaxed) && HIDDEN_IDS.read().unwrap_or_else(|e| e.into_inner()).contains(&(viewer, target)))
        || (ANY_UNSEEN.load(Ordering::Relaxed) && UNSEEN_IDS.read().unwrap_or_else(|e| e.into_inner()).contains(&(viewer, target)))
}

/// `openWorkbench(null, true)` and the like: the menu a player has open where no block is, which
/// the game's check would close at once.
pub static FORCED_MENUS: Mutex<Vec<(u128, [i32; 3])>> = Mutex::new(Vec::new());

pub fn forced_menu(player: u128, pos: [i32; 3]) -> bool {
    FORCED_MENUS.lock().unwrap_or_else(|e| e.into_inner()).contains(&(player, pos))
}

pub fn allow_game(text: &str, component: &Tag, overlay: bool) -> bool {
    let message = GameMessage { text, component, overlay };
    CHAT_HOOKS.get().is_none_or(|(_, game)| game(&message))
}

thread_local! {
    /// What `allow_command` made of a line, for the code that sends it.
    static COMMAND_DECORATED: std::cell::RefCell<Option<Tag>> = const { std::cell::RefCell::new(None) };
}

pub fn take_decorated() -> Option<Tag> {
    COMMAND_DECORATED.with(|held| held.borrow_mut().take())
}

/// A line `/say` or `/me` is about to send to everyone: whether it goes. `broadcastChatMessage` for a
/// command source: decorated, then the command's events, then for a player the chat's too.
pub fn allow_command(kind: &str, text: &str, sender: Option<u128>, name: &str) -> bool {
    let decorated = decorate_as(sender, text);
    let _ = kind;
    if let Some(player) = sender
        && !allow_chat(player, name, text)
    {
        return false;
    }
    COMMAND_DECORATED.with(|held| *held.borrow_mut() = decorated);
    true
}

/// A fatal hit waiting for `ALLOW_DEATH`. The hit lands deep in the combat code, which has no
/// `Game`: it stops short of dying the way a totem stops it, and the game asks at the end of the
/// player's turn.
pub struct HeldDeath {
    pub source: String,
    pub amount: f32,
    /// `/kill` and the void: a totem can't stop these, a listener can.
    pub bypass: bool,
}

static DEATH_GUARD: AtomicBool = AtomicBool::new(false);
static HELD_DEATH: Mutex<Option<HeldDeath>> = Mutex::new(None);
/// The last damage asked for, in bits: `ALLOW_DEATH` reports it.
static LAST_AMOUNT: AtomicU32 = AtomicU32::new(0);

/// What a sleeping mob did in its turn. `LivingEntity` has no `Game` in hand, so the game tells it at the turn's end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobSleep {
    Started([i32; 3]),
    Stopped([i32; 3]),
    /// `checkBedExists`: the place, and the game's own answer.
    BedChecked([i32; 3], bool),
}

static SLEEP_GUARD: AtomicBool = AtomicBool::new(false);

thread_local! {
    static MOB_SLEEP: std::cell::RefCell<Vec<(u32, MobSleep)>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub fn guard_sleep(listening: bool) {
    SLEEP_GUARD.store(listening, Ordering::Relaxed);
}

/// Whether anything listens to a mob's sleep.
pub fn sleep_guarded() -> bool {
    SLEEP_GUARD.load(Ordering::Relaxed)
}

pub fn mob_sleep(mob: u32, note: MobSleep) {
    if sleep_guarded() {
        MOB_SLEEP.with(|notes| notes.borrow_mut().push((mob, note)));
    }
}

pub fn mob_sleep_waiting() -> bool {
    MOB_SLEEP.with(|notes| !notes.borrow().is_empty())
}

pub fn take_mob_sleep() -> Vec<(u32, MobSleep)> {
    MOB_SLEEP.with(|notes| std::mem::take(&mut *notes.borrow_mut()))
}

pub fn guard_deaths(listening: bool) {
    DEATH_GUARD.store(listening, Ordering::Relaxed);
}

pub fn deaths_guarded() -> bool {
    DEATH_GUARD.load(Ordering::Relaxed)
}

pub fn note_amount(amount: f32) {
    LAST_AMOUNT.store(amount.to_bits(), Ordering::Relaxed);
}

pub fn hold_death(source: &str, bypass: bool) {
    let amount = f32::from_bits(LAST_AMOUNT.load(Ordering::Relaxed));
    *HELD_DEATH.lock().unwrap_or_else(|e| e.into_inner()) = Some(HeldDeath { source: source.to_owned(), amount, bypass });
}

pub fn take_death() -> Option<HeldDeath> {
    HELD_DEATH.lock().unwrap_or_else(|e| e.into_inner()).take()
}

pub fn push(event: Pending) {
    PENDING.lock().unwrap_or_else(|e| e.into_inner()).push(event);
}

pub fn take() -> Vec<Pending> {
    std::mem::take(&mut *PENDING.lock().unwrap_or_else(|e| e.into_inner()))
}







/// What a plugin set for one player's flight; the game knows only of game modes.
#[derive(Clone, Copy, Debug)]
pub struct Flight {
    pub allowed: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
    /// What the client last asked for this tick.
    pub wanted: Option<bool>,
}

pub static FLIGHT: Mutex<std::collections::BTreeMap<u128, Flight>> = Mutex::new(std::collections::BTreeMap::new());

fn flight() -> std::sync::MutexGuard<'static, std::collections::BTreeMap<u128, Flight>> {
    FLIGHT.lock().unwrap_or_else(|e| e.into_inner())
}

/// `Guest::ability_flags`, with a plugin's permission to fly.
pub fn ability_flags(player: u128, value: i8) -> i8 {
    if flight().get(&player).is_some_and(|flight| flight.allowed) { value | 4 } else { value }
}

/// A client's request to fly or land, seen before the game refuses it for not being in creative.
pub fn flight_wanted(player: u128, flying: bool) {
    if let Some(flight) = flight().get_mut(&player).filter(|flight| flight.allowed) {
        flight.wanted = Some(flying);
    }
}

/// True if the three toggle packets go to the game to be heard first: only what changes a flag does.
pub fn toggle_held(changes: bool) -> bool {
    changes && horizon_mod_runtime::listens("pl_packet")
}

/// True if a plugin lets this player fly.
pub fn flight_allowed(player: u128) -> bool {
    flight().get(&player).is_some_and(|flight| flight.allowed)
}

/// A line through `ChatProcessor`: what a listener did to it, then what's held for the hub to send.
type ChatDone = (u128, serde_json::Value, Box<dyn std::any::Any + Send>);

static CHAT_CHAINS: Mutex<Option<HashMap<u128, std::thread::JoinHandle<()>>>> = Mutex::new(None);
static CHAT_DONE: Mutex<Vec<ChatDone>> = Mutex::new(Vec::new());
/// Lines between the packet and the answer: the game keeps a copy of who's who for them.
pub static CHATS_RUNNING: AtomicU32 = AtomicU32::new(0);
/// `PlayerChatEvent`: the game's thread runs it, and chat waits, as Paper's `Waitable` does.
pub static CHAT_MAIN: Mutex<Vec<(String, std::sync::mpsc::Sender<String>)>> = Mutex::new(Vec::new());
/// A listener's question from a chat thread, for the game's next turn.
pub static CHAT_ASKS: Mutex<Vec<(String, std::sync::mpsc::Sender<String>)>> = Mutex::new(Vec::new());
/// Each player's state as of the last turn, by uuid. Chat threads read it as Paper's read a live player.
pub static CHAT_PLAYERS: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

thread_local! {
    static ON_CHAT_THREAD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// True if a plugin hears chat.
pub fn chat_listened() -> bool {
    use horizon_mod_runtime::listens;
    listens("pl_chat_old") || listens("pl_chat_main") || listens("pl_chat")
}

fn chat_stage(event: &str, text: &mut String) {
    if let (_, Some(carried)) = horizon_mod_runtime::fire_carried(event, text) {
        *text = carried;
    }
}

/// `ChatProcessor.process`, on a thread of the player's own: their lines keep their order, and a slow
/// listener holds only them.
pub fn chat_heard(player: u128, payload: serde_json::Value, held: Box<dyn std::any::Any + Send>) {
    CHATS_RUNNING.fetch_add(1, Ordering::Relaxed);
    let mut chains = CHAT_CHAINS.lock().unwrap_or_else(|e| e.into_inner());
    let chains = chains.get_or_insert_default();
    let before = chains.remove(&player);
    let thread = std::thread::spawn(move || {
        if let Some(before) = before {
            let _ = before.join();
        }
        ON_CHAT_THREAD.set(true);
        let mut text = payload.to_string();
        chat_stage("pl_chat_old", &mut text);
        if horizon_mod_runtime::listens("pl_chat_main") {
            let (answer, reply) = std::sync::mpsc::channel();
            CHAT_MAIN.lock().unwrap_or_else(|e| e.into_inner()).push((text.clone(), answer));
            if let Ok(done) = reply.recv_timeout(std::time::Duration::from_secs(30)) {
                text = done;
            }
        }
        chat_stage("pl_chat", &mut text);
        chat_stage("chat_final", &mut text);
        let said = serde_json::from_str(&text).unwrap_or_default();
        CHAT_DONE.lock().unwrap_or_else(|e| e.into_inner()).push((player, said, held));
        CHATS_RUNNING.fetch_sub(1, Ordering::Relaxed);
    });
    chains.insert(player, thread);
}

/// `Player.chat`: the same events, on the caller's thread with the game in hand.
pub fn chat_compelled(player: u128, mut payload: serde_json::Value, held: Box<dyn std::any::Any + Send>) {
    payload["compelled"] = serde_json::json!(true);
    let mut text = payload.to_string();
    for event in ["pl_chat_old", "pl_chat_main", "pl_chat", "chat_final"] {
        chat_stage(event, &mut text);
    }
    let said = serde_json::from_str(&text).unwrap_or_default();
    CHAT_DONE.lock().unwrap_or_else(|e| e.into_inner()).push((player, said, held));
}

/// What came of this player's lines, for the hub to send.
pub fn chat_answers(player: u128) -> Vec<(serde_json::Value, Box<dyn std::any::Any + Send>)> {
    // HorizonsX: a region thread's hub holds only its region's players; a chat line goes to everyone, so it
    // waits for the server thread's turn at the player's inbox (the same tick).
    thread_local! {
        static REGION: bool = std::thread::current().name().is_some_and(|n| n.starts_with("horizonsx-"));
    }
    if REGION.with(|r| *r) {
        return Vec::new();
    }
    let mut done = CHAT_DONE.lock().unwrap_or_else(|e| e.into_inner());
    let mut mine = Vec::new();
    let mut k = 0;
    while k < done.len() {
        if done[k].0 == player {
            let (_, said, held) = done.remove(k);
            mine.push((said, held));
        } else {
            k += 1;
        }
    }
    mine
}

/// A chat thread's question: `None` on any other thread. A player's state comes from the game's copy;
/// anything else waits for its next turn.
pub fn chat_ask(request: &str) -> Option<String> {
    if !ON_CHAT_THREAD.get() {
        return None;
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(request)
        && value["op"] == "player"
        && let Some(known) = CHAT_PLAYERS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(value["uuid"].as_str()?).cloned())
    {
        return Some(known);
    }
    let (answer, reply) = std::sync::mpsc::channel();
    CHAT_ASKS.lock().unwrap_or_else(|e| e.into_inner()).push((request.to_owned(), answer));
    Some(reply.recv_timeout(std::time::Duration::from_secs(5)).unwrap_or_else(|_| r#"{"error":"the game didn't answer"}"#.to_owned()))
}

/// A line's component as the client wants it.
pub fn chat_tag(value: &serde_json::Value) -> Tag {
    crate::lan::data::component_nbt(&horizon_client_render::packs::component(value).component())
}

/// A connection in configuration, asking the game where this player stands.
pub static SPAWN_ASKS: Mutex<Vec<(u128, std::sync::mpsc::Sender<serde_json::Value>)>> = Mutex::new(Vec::new());
static SPAWN_CHOSEN: Mutex<Vec<(u128, String, [f64; 3], f32, f32)>> = Mutex::new(Vec::new());

/// `AsyncPlayerSpawnLocationEvent`, on the connection's thread, as configuration ends.
pub fn spawn_event(uuid: u128) {
    if !horizon_mod_runtime::listens("pl_spawn_location") {
        return;
    }
    let (answer, reply) = std::sync::mpsc::channel();
    SPAWN_ASKS.lock().unwrap_or_else(|e| e.into_inner()).push((uuid, answer));
    let Ok(facts) = reply.recv_timeout(std::time::Duration::from_secs(5)) else {
        return;
    };
    let mut payload = CONNECTION.with(|connection| connection.borrow().clone());
    for key in ["level", "at", "yaw", "pitch", "new"] {
        payload[key] = facts[key].clone();
    }
    let value = carried_value("pl_spawn_location", payload.clone()).unwrap_or(payload);
    let moved = value["to"].as_array().zip(value["to_level"].as_str());
    let number = |i: usize| moved.and_then(|(to, _)| to.get(i)).and_then(serde_json::Value::as_f64).unwrap_or(0.0);
    let (level, at) = match moved {
        Some((_, level)) => {
            SPAWN_CHOSEN.lock().unwrap_or_else(|e| e.into_inner()).push((uuid, level.to_owned(), [number(0), number(1), number(2)], number(3) as f32, number(4) as f32));
            (level.to_owned(), [number(0), number(2)])
        }
        None => (facts["level"].as_str().unwrap_or_default().to_owned(), [facts["at"][0].as_f64().unwrap_or(0.0), facts["at"][2].as_f64().unwrap_or(0.0)]),
    };
    // Paper's player stays in configuration till the spawn's chunk has loaded.
    let (done, loaded) = std::sync::mpsc::channel();
    let chunk = ((at[0].floor() as i32) >> 4, (at[1].floor() as i32) >> 4);
    SPAWN_LOADS.lock().unwrap_or_else(|e| e.into_inner()).push(SpawnLoad { level, chunk, asked: false, done });
    let _ = loaded.recv_timeout(std::time::Duration::from_secs(15));
}

/// A connection waiting for its spawn's chunk.
pub struct SpawnLoad {
    pub level: String,
    pub chunk: (i32, i32),
    pub asked: bool,
    pub done: std::sync::mpsc::Sender<()>,
}

pub static SPAWN_LOADS: Mutex<Vec<SpawnLoad>> = Mutex::new(Vec::new());

/// Someone joining waits on the game: their spawn, its chunk, a listener's answer, their held
/// configuration or `ServerPlayConnectionEvents.INIT`. Only a ticking game answers these, so an empty
/// server that has paused must tick while they wait: paused, INIT alone ran out its 5 s every join.
pub fn logins_waiting() -> bool {
    fn any<T>(queue: &Mutex<Vec<T>>) -> bool {
        !queue.lock().unwrap_or_else(|e| e.into_inner()).is_empty()
    }
    any(&SPAWN_ASKS) || any(&SPAWN_LOADS) || any(&CONNECTION_ASKS) || any(&ASKS) || any(&HELD)
}

/// Where a listener put this player, once.
pub fn spawn_chosen(uuid: u128) -> Option<(String, [f64; 3], f32, f32)> {
    let mut all = SPAWN_CHOSEN.lock().unwrap_or_else(|e| e.into_inner());
    let found = all.iter().position(|row| row.0 == uuid)?;
    let row = all.remove(found);
    Some((row.1, row.2, row.3, row.4))
}

/// Blocks a player's motion would change: they wait for `PlayerInteractEvent` on the game's turn.
pub static STEPS_HELD: Mutex<Vec<([i32; 3], Option<horizon_world::registry::BlockStateId>)>> = Mutex::new(Vec::new());

/// True if a plugin hears of this step first: the change then waits for its answer.
pub fn step_held(pos: [i32; 3], next: Option<horizon_world::registry::BlockStateId>) -> bool {
    if !horizon_mod_runtime::listens("pl_step") {
        return false;
    }
    STEPS_HELD.lock().unwrap_or_else(|e| e.into_inner()).push((pos, next));
    true
}

/// A move the hub would turn down, held for `PlayerFailMoveEvent`.
pub struct FailedMove {
    pub uuid: u128,
    pub reason: &'static str,
    pub from: [f64; 3],
    pub to: [f64; 3],
    pub rot: (f32, f32),
    pub flags: u8,
    pub line: String,
    /// Sent back, unless allowed.
    pub refused: bool,
    pub back: Option<[f64; 3]>,
}

pub static FAILED_MOVES: Mutex<Vec<FailedMove>> = Mutex::new(Vec::new());
static LET_THROUGH: Mutex<Vec<u128>> = Mutex::new(Vec::new());
static ANY_LET_THROUGH: AtomicBool = AtomicBool::new(false);

/// True if a plugin hears of this move, which then waits for its answer.
pub fn move_failed(row: FailedMove) -> bool {
    if !horizon_mod_runtime::listens("pl_fail_move") {
        return false;
    }
    FAILED_MOVES.lock().unwrap_or_else(|e| e.into_inner()).push(row);
    true
}

/// `setAllowed(true)`: this player's next move isn't checked.
pub fn let_move_through(player: u128) {
    LET_THROUGH.lock().unwrap_or_else(|e| e.into_inner()).push(player);
    ANY_LET_THROUGH.store(true, Ordering::Relaxed);
}

pub fn move_let_through(player: u128) -> bool {
    if !ANY_LET_THROUGH.load(Ordering::Relaxed) {
        return false;
    }
    let mut all = LET_THROUGH.lock().unwrap_or_else(|e| e.into_inner());
    let found = all.iter().position(|who| *who == player);
    found.map(|at| all.remove(at)).is_some()
}

/// What a client's request comes to, for someone a plugin lets fly.
pub fn flight_answer(player: u128) -> Option<bool> {
    flight().get_mut(&player)?.wanted.take()
}

/// The argument of a plugin's command (`name [args:text]`) asks the server what can come next, as
/// Bukkit's commands do, so a plugin's tab completer gets asked.
pub fn ask_server_for_plugins(mut commands: horizon_protocol::packets::play::Commands) -> horizon_protocol::packets::play::Commands {
    for node in &mut commands.nodes {
        if node.name == "args" && node.kind == 2 && node.suggestions.is_none() {
            node.suggestions = Some("minecraft:ask_server".to_owned());
        }
    }
    commands
}

/// Whether a plugin may want what clients send: set when a DLL mod is loaded.
pub static WATCH_PACKETS: AtomicBool = AtomicBool::new(false);
static PACKETS: Mutex<Vec<(u128, &'static str, serde_json::Value)>> = Mutex::new(Vec::new());

/// A client's packet that Bukkit makes an event of. The hub has no `Game`, so it waits for the end of
/// the tick, where the bridge fires it: these tell, they can't refuse.
pub fn note_packet(player: u128, packet: &horizon_protocol::packets::play::Serverbound) {
    use horizon_protocol::packets::play::Serverbound as S;
    use serde_json::json;
    if !WATCH_PACKETS.load(Ordering::Relaxed) {
        return;
    }
    let event = match packet {
        S::PlayerInput(p) => ("packet_input", json!({ "flags": p.flags })),
        S::ClientInformation(p) => ("packet_options", json!({ "locale": p.language, "view_distance": p.view_distance, "chat_visibility": p.chat_visibility, "chat_colors": p.chat_colors, "skin_parts": p.model_customisation, "main_hand": p.main_hand, "text_filtering": p.text_filtering, "allows_listing": p.allows_listing, "particle_status": p.particle_status })),
        S::SelectBundleItem(p) => ("packet_bundle", json!({ "slot": p.slot, "index": p.index })),
        S::SetBeacon(p) => ("packet_beacon", json!({ "primary": p.primary, "secondary": p.secondary })),
        S::PickItemFromBlock(p) => ("packet_pick", json!({ "pos": p.pos, "include_data": p.include_data })),
        S::PickItemFromEntity(p) => ("packet_pick", json!({ "entity": p.id, "include_data": p.include_data })),
        S::ContainerButtonClick(p) => ("packet_button", json!({ "container": p.container, "button": p.button })),
        S::SetCreativeModeSlot(p) => ("packet_creative", json!({ "slot": p.slot })),
        S::RenameItem(p) => ("packet_rename", json!({ "name": p.name })),
        S::MoveVehicle(p) => ("packet_vehicle", json!({ "position": p.position, "yaw": p.yaw, "pitch": p.pitch })),
        S::PaddleBoat(p) => ("packet_paddle", json!({ "left": p.left, "right": p.right })),
        S::PlayerCommand(p) => ("packet_command", json!({ "action": p.action, "data": p.data })),
        S::ClientCommand(p) => ("packet_client", json!({ "action": p.action })),
        S::ClientTickEnd(_) => ("packet_tick_end", json!({})),
        S::ResourcePack(p) => ("packet_pack", json!({ "id": format!("{:032x}", p.id), "action": p.action })),
        _ => return,
    };
    PACKETS.lock().unwrap_or_else(|e| e.into_inner()).push((player, event.0, event.1));
}

/// What clients sent since the last call.
pub fn take_packets() -> Vec<(u128, &'static str, serde_json::Value)> {
    std::mem::take(&mut *PACKETS.lock().unwrap_or_else(|e| e.into_inner()))
}

// ---- Server list pings ----

/// Whether anyone listens to a ping, and what they make of it: the connection's thread asks them itself.
pub static PING_HOOK: std::sync::OnceLock<(fn() -> bool, fn(u32, String) -> String)> = std::sync::OnceLock::new();
static NEXT_PING: AtomicU32 = AtomicU32::new(1);

thread_local! {
    static BUILDING_STATUS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The status a listener answered with, or `None` to send the game's own. `status` is the game's own,
/// built by coming back through the hook, which then lets it pass.
pub fn status_ping(status: impl FnOnce() -> String) -> Option<String> {
    let hook = PING_HOOK.get()?;
    if BUILDING_STATUS.get() || !(hook.0)() {
        return None;
    }
    BUILDING_STATUS.set(true);
    let json = status();
    BUILDING_STATUS.set(false);
    let id = NEXT_PING.fetch_add(1, Ordering::Relaxed);
    note_ping_client(id);
    Some((hook.1)(id, json))
}

// ---- Connections before they are players ----

/// What a client said as it connected.
#[derive(Clone, Debug)]
pub struct Handshake {
    pub address: std::net::SocketAddr,
    pub protocol: i32,
    pub host: String,
    pub port: u16,
    pub transferred: bool,
}

thread_local! {
    static ADDRESS: std::cell::Cell<Option<std::net::SocketAddr>> = const { std::cell::Cell::new(None) };
    /// The connection this thread serves, as a listener reads it.
    static CONNECTION: std::cell::RefCell<serde_json::Value> = const { std::cell::RefCell::new(serde_json::Value::Null) };
}

/// The handshakes of the latest connections, by address.
static HANDSHAKES: Mutex<Vec<Handshake>> = Mutex::new(Vec::new());
/// Who pinged, for `PaperServerListPingEvent.getClient`.
static PING_CLIENTS: Mutex<Vec<(u32, Handshake)>> = Mutex::new(Vec::new());
const KEPT: usize = 1024;

fn handshake_here() -> Option<Handshake> {
    use horizon_core::paper_wire::connection;
    let (protocol, host, port, intent) = connection::handshake()?;
    Some(Handshake { address: ADDRESS.get()?, protocol, host: host.split('\0').next().unwrap_or_default().to_owned(), port, transferred: intent == horizon_protocol::packets::handshake::TRANSFER })
}

/// The head of `serve_connection`: the next packet on this thread is the handshake.
pub fn connection_begins(address: std::net::SocketAddr) {
    ADDRESS.set(Some(address));
    horizon_core::paper_wire::connection::expect_handshake();
}

/// The head of `serve_login`: this address is a player now, and what they said can be found by it.
pub fn login_begins() {
    let Some(handshake) = handshake_here() else {
        return;
    };
    let mut all = HANDSHAKES.lock().unwrap_or_else(|e| e.into_inner());
    all.retain(|known| known.address != handshake.address);
    if all.len() >= KEPT {
        all.remove(0);
    }
    all.push(handshake);
}

pub fn handshake_of(address: std::net::SocketAddr) -> Option<Handshake> {
    HANDSHAKES.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|known| known.address == address).cloned()
}

fn note_ping_client(id: u32) {
    let Some(handshake) = handshake_here() else {
        return;
    };
    let mut all = PING_CLIENTS.lock().unwrap_or_else(|e| e.into_inner());
    if all.len() >= KEPT {
        all.remove(0);
    }
    all.push((id, handshake));
}

pub fn ping_client(id: u32) -> Option<Handshake> {
    PING_CLIENTS.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|(known, _)| *known == id).map(|(_, handshake)| handshake.clone())
}

/// What listeners made of a connection.
#[derive(Default)]
pub struct ConnectionAnswer {
    pub kick: Option<serde_json::Value>,
    pub packets: Vec<Vec<u8>>,
}

/// A connection waiting on the game, as its thread has none and a plugin needs one.
pub struct ConnectionAsk {
    pub id: u32,
    pub kind: &'static str,
    pub payload: serde_json::Value,
    pub seen: i64,
    pub kick: Option<serde_json::Value>,
    pub packets: Vec<Vec<u8>>,
    pub answer: std::sync::mpsc::Sender<ConnectionAnswer>,
}

pub static CONNECTION_ASKS: Mutex<Vec<ConnectionAsk>> = Mutex::new(Vec::new());
/// When a listener last looked: with nobody looking, a connection doesn't wait.
pub static CONNECTIONS_POLLED: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static NEXT_ASK: AtomicU32 = AtomicU32::new(1);

fn connections_watched() -> bool {
    CONNECTIONS_POLLED.lock().unwrap_or_else(|e| e.into_inner()).is_some_and(|at| at.elapsed() < std::time::Duration::from_secs(1))
}

fn ask_connection(kind: &'static str, kick: Option<serde_json::Value>) -> Option<ConnectionAnswer> {
    if !connections_watched() {
        return None;
    }
    let payload = CONNECTION.with(|connection| connection.borrow().clone());
    let (answer, reply) = std::sync::mpsc::channel();
    let id = NEXT_ASK.fetch_add(1, Ordering::Relaxed);
    CONNECTION_ASKS.lock().unwrap_or_else(|e| e.into_inner()).push(ConnectionAsk { id, kind, payload, seen: 0, kick, packets: Vec::new(), answer });
    reply.recv_timeout(std::time::Duration::from_secs(5)).ok()
}

fn reason_tag(reason: &serde_json::Value) -> Tag {
    crate::lan::data::component_nbt(&horizon_client_render::packs::component(reason).component())
}

fn connection_json(uuid: u128, name: &str, properties: &[horizon_protocol::packets::login::Property]) {
    use horizon_core::paper_wire::connection;
    let properties: Vec<serde_json::Value> = properties.iter().map(|property| serde_json::json!({ "name": property.name, "value": property.value, "signature": property.signature })).collect();
    let handshake = handshake_here();
    let hello = connection::hello();
    CONNECTION.with(|connection| {
        *connection.borrow_mut() = serde_json::json!({
            "uuid": format!("{uuid:032x}"),
            "name": name,
            "properties": properties,
            "address": handshake.as_ref().map(|handshake| handshake.address.to_string()),
            "protocol": handshake.as_ref().map(|handshake| handshake.protocol),
            "host": handshake.as_ref().map(|handshake| format!("{}:{}", handshake.host, handshake.port)),
            "transferred": handshake.as_ref().is_some_and(|handshake| handshake.transferred),
            "hello": hello.map(|(name, uuid)| serde_json::json!({ "name": name, "uuid": format!("{uuid:032x}") })),
        });
    });
}

/// `PlayerConnectionValidateLoginEvent` for who the game let through: the reason if a listener refuses them.
pub fn validate_login(uuid: u128, name: &str, properties: &[horizon_protocol::packets::login::Property]) -> Option<Tag> {
    connection_json(uuid, name, properties);
    ask_connection("validate", None)?.kick.as_ref().map(reason_tag)
}

thread_local! {
    static ADMITTING: std::cell::RefCell<Vec<horizon_protocol::packets::login::Property>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The profile of whoever `try_admit` is about to weigh up.
pub fn admitting(properties: &[horizon_protocol::packets::login::Property]) {
    ADMITTING.with(|admitting| *admitting.borrow_mut() = properties.to_vec());
}

/// The same for one the game refuses: `Some(None)` if a listener lets them in, `Some(reason)` for another reason.
pub fn validate_refused(uuid: u128, name: &str, refusal: &Tag) -> Option<Option<Tag>> {
    if !connections_watched() {
        return None;
    }
    let properties = ADMITTING.with(std::cell::RefCell::take);
    connection_json(uuid, name, &properties);
    let reason = serde_json::from_str(&crate::lan::conn::component_json(refusal)).ok();
    Some(ask_connection("validate", reason)?.kick.as_ref().map(reason_tag))
}

/// What's waiting to go to someone in configuration, from whichever thread a plugin is on.
struct Held {
    id: u32,
    packets: Vec<Vec<u8>>,
    kick: Option<Vec<u8>>,
    /// `AsyncPlayerConnectionConfigureEvent`'s listeners haven't all returned.
    listening: bool,
}

static HELD: Mutex<Vec<Held>> = Mutex::new(Vec::new());

fn held<R>(id: u32, visit: impl FnOnce(&mut Held) -> R) -> Option<R> {
    HELD.lock().unwrap_or_else(|e| e.into_inner()).iter_mut().find(|held| held.id == id).map(visit)
}

/// One connection's row, gone when configuration is over or its thread is.
struct HeldRow(u32);

impl Drop for HeldRow {
    fn drop(&mut self) {
        HELD.lock().unwrap_or_else(|e| e.into_inner()).retain(|held| held.id != self.0);
    }
}

thread_local! {
    static HELD_ROW: std::cell::RefCell<Option<HeldRow>> = const { std::cell::RefCell::new(None) };
}

/// A packet for someone in configuration: `false` once they aren't there.
pub fn held_packet(id: u32, body: Vec<u8>) -> bool {
    held(id, |held| held.packets.push(body)).is_some()
}

/// `PlayerConnection.disconnect` on someone in configuration.
pub fn held_kick(id: u32, reason: &serde_json::Value) -> bool {
    let body = config_kick(reason);
    held(id, |held| held.kick = Some(body)).is_some()
}

/// `PlayerConnection.isConnected` of someone in configuration.
pub fn held_open(id: u32) -> bool {
    held(id, |_| ()).is_some()
}

/// What's gone out of `configure` since last asked: its packets, the kick that ends it, and whether listeners are still running.
pub fn held_take(id: u32) -> (Vec<Vec<u8>>, Option<Vec<u8>>, bool) {
    held(id, |held| (std::mem::take(&mut held.packets), held.kick.take(), held.listening)).unwrap_or_default()
}

fn held_here() -> Option<u32> {
    HELD_ROW.with(|row| row.borrow().as_ref().map(|row| row.0))
}

/// This thread's connection in `held_take`.
pub fn held_id() -> u32 {
    held_here().unwrap_or(0)
}

/// Whether the client has to wait: listeners to run, or something a plugin already sent.
pub fn configuration_held() -> bool {
    held_here().is_some_and(|id| horizon_mod_runtime::listens("paper_configure") || held(id, |held| !held.packets.is_empty() || held.kick.is_some()) == Some(true))
}

/// The plugin's say is over: what comes after isn't configuration.
pub fn configuring_ends() {
    let row = HELD_ROW.with(|row| row.borrow_mut().take());
    drop(row);
}

/// `AsyncPlayerConnectionConfigureEvent`, on a thread of its own as Paper's is: a listener may block for
/// as long as it likes while `hold_configuration` reads and keeps the client alive.
pub fn configure_event() {
    let Some(id) = held_here().filter(|_| horizon_mod_runtime::listens("paper_configure")) else {
        return;
    };
    let payload = CONNECTION.with(|connection| connection.borrow().to_string());
    held(id, |held| held.listening = true);
    let listeners = std::thread::Builder::new().name(format!("Configuration thread {id}")).spawn(move || {
        horizon_mod_runtime::fire("paper_configure", &payload);
        held(id, |held| held.listening = false);
    });
    if listeners.is_err() {
        held(id, |held| held.listening = false);
    }
}

/// `Shared::sync` on a login's thread: configuration begins, with somewhere for a plugin's packets to go.
pub fn configuration_begins() {
    if CONNECTION.with(|connection| connection.borrow().is_null()) {
        return;
    }
    let id = NEXT_ASK.fetch_add(1, Ordering::Relaxed);
    HELD.lock().unwrap_or_else(|e| e.into_inner()).push(Held { id, packets: Vec::new(), kick: None, listening: false });
    let last = HELD_ROW.with(|row| row.borrow_mut().replace(HeldRow(id)));
    drop(last);
    CONNECTION.with(|connection| connection.borrow_mut()["held"] = serde_json::json!(id));
    if let Some(answer) = (!RECONFIGURING.get()).then(|| ask_connection("configure_initial", None)).flatten() {
        answer.packets.into_iter().for_each(horizon_core::paper_wire::wire::push_out);
        if let Some(reason) = &answer.kick {
            horizon_core::paper_wire::wire::push_out(config_kick(reason));
        }
    }
}

fn config_kick(reason: &serde_json::Value) -> Vec<u8> {
    use horizon_protocol::packets::configuration as cfg;
    cfg::Clientbound::Disconnect(cfg::Disconnect { reason: reason_tag(reason) }).encode()
}

/// A tag as the mods read one.
fn tag_value(tag: &Tag) -> serde_json::Value {
    use serde_json::{Value, json};
    match tag {
        Tag::End => Value::Null,
        Tag::Byte(value) => json!(value),
        Tag::Short(value) => json!(value),
        Tag::Int(value) => json!(value),
        Tag::Long(value) => json!(value),
        Tag::Float(bits) => json!(f32::from_bits(*bits)),
        Tag::Double(bits) => json!(f64::from_bits(*bits)),
        Tag::ByteArray(values) => json!(values),
        Tag::String(value) => json!(value.to_string_lossy()),
        Tag::List(values) => Value::Array(values.iter().map(tag_value).collect()),
        Tag::Compound(values) => Value::Object(values.iter().map(|(key, value)| (key.to_string_lossy(), tag_value(value))).collect()),
        Tag::IntArray(values) => json!(values),
        Tag::LongArray(values) => json!(values),
    }
}

/// `decode_config`: a client's settings and brand for the connection's options, and what it says back to a
/// plugin: a dialog's button, a cookie. The listeners hold no game here, as this is the connection's thread.
pub fn config_packet_seen(id: i32, payload: &[u8]) {
    use horizon_protocol::packets::configuration as cfg;
    let packet = cfg::Serverbound::decode(id, payload);
    match &packet {
        Ok(cfg::Serverbound::CustomClickAction(click)) if horizon_mod_runtime::listens("pl_packet") => {
            let mut event = CONNECTION.with(|connection| connection.borrow().clone());
            if !event.is_null() {
                event["kind"] = serde_json::json!("custom_click");
                event["configuring"] = serde_json::json!(true);
                event["id"] = serde_json::json!(click.id);
                event["tag"] = click.payload.as_ref().map_or(serde_json::Value::Null, tag_value);
                horizon_mod_runtime::fire("pl_packet", &event.to_string());
            }
        }
        Ok(cfg::Serverbound::CookieResponse(cookie)) if horizon_mod_runtime::listens("paper_cookie") => {
            let uuid = CONNECTION.with(|connection| connection.borrow()["uuid"].clone());
            horizon_mod_runtime::fire("paper_cookie", &serde_json::json!({ "uuid": uuid, "cookie": { "key": cookie.key, "payload": cookie.payload } }).to_string());
        }
        _ => {}
    }
    match packet {
        Ok(cfg::Serverbound::ClientInformation(info)) => CONNECTION.with(|connection| {
            let mut connection = connection.borrow_mut();
            if !connection.is_null() {
                connection["info"] = serde_json::json!({ "locale": info.language, "view_distance": info.view_distance, "chat_visibility": info.chat_visibility, "chat_colors": info.chat_colors, "skin_parts": info.model_customisation, "main_hand": info.main_hand, "text_filtering": info.text_filtering, "allows_listing": info.allows_listing, "particle_status": info.particle_status });
            }
        }),
        Ok(cfg::Serverbound::CustomPayload(packet)) if packet.channel == "minecraft:brand" => CONNECTION.with(|connection| {
            let mut connection = connection.borrow_mut();
            if !connection.is_null() {
                let mut reader = horizon_protocol::buf::Reader::new(&packet.data);
                connection["brand"] = serde_json::json!(reader.string(32767).ok());
            }
        }),
        _ => {}
    }
}

// ---- Commands.restricted ----

/// Paths of literal words that ask for confirmation when run from a click.
static RESTRICTED: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());

pub fn restrict_command(path: Vec<String>) {
    RESTRICTED.lock().unwrap_or_else(|e| e.into_inner()).push(path);
}

/// The tree a client is sent, with what plugins restricted.
pub fn mark_restricted(mut commands: horizon_protocol::packets::play::Commands) -> horizon_protocol::packets::play::Commands {
    for path in RESTRICTED.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        let mut at = commands.root as usize;
        let mut found = true;
        for word in path {
            let next = commands.nodes[at].children.iter().copied().find(|child| commands.nodes.get(*child as usize).is_some_and(|node| node.name == *word));
            match next {
                Some(next) => at = next as usize,
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found {
            commands.nodes[at].restricted = true;
        }
    }
    commands
}

/// A component read out in the server's language.
pub fn english(hub: &crate::lan::hub::Hub, tag: &Tag) -> String {
    hub.english(tag)
}

// ---- ev_player: a connection's thread, as Paper's netty thread, and what the hub leaves for the tick ----

fn carried_value(event: &str, payload: serde_json::Value) -> Option<serde_json::Value> {
    let (_, carried) = horizon_mod_runtime::fire_carried(event, &payload.to_string());
    carried.and_then(|text| serde_json::from_str(&text).ok())
}

thread_local! {
    /// What `PlayerHandshakeEvent` said of this connection's profile.
    static SPOOFED: std::cell::RefCell<Option<(Option<u128>, Vec<horizon_protocol::packets::login::Property>)>> = const { std::cell::RefCell::new(None) };
}

/// `PlayerHandshakeEvent`, with no game to lend, as on Paper: what to refuse them with, if a listener
/// failed the handshake, and the address they're to have. It starts cancelled, as Paper's does with
/// no BungeeCord set up, and only a listener that lifts that has any say.
pub fn player_handshake(host: &str, port: u16, protocol: i32, intent: i32, address: std::net::SocketAddr) -> (Option<Tag>, std::net::SocketAddr) {
    if !horizon_mod_runtime::listens("pl_handshake") {
        return (None, address);
    }
    let payload = serde_json::json!({ "handshake": host, "remote": address.ip().to_string(), "cancelled": true });
    let Some(value) = carried_value("pl_handshake", payload).filter(|value| value["cancelled"] == false) else {
        return (None, address);
    };
    if value["failed"] == true {
        return (Some(reason_tag(&value["fail_message"])), address);
    }
    if let Some(host) = value["server_hostname"].as_str() {
        horizon_core::paper_wire::connection::note_handshake(protocol, host.to_owned(), port, intent);
    }
    let address = value["socket_hostname"].as_str().and_then(|ip| ip.parse().ok()).map_or(address, |ip| std::net::SocketAddr::new(ip, address.port()));
    ADDRESS.set(Some(address));
    let uuid = value["unique_id"].as_str().and_then(|uuid| u128::from_str_radix(&uuid.replace('-', ""), 16).ok());
    let rows: Vec<serde_json::Value> = value["properties"].as_str().and_then(|json| serde_json::from_str(json).ok()).unwrap_or_default();
    let properties = rows
        .iter()
        .filter_map(|row| Some(horizon_protocol::packets::login::Property { name: row["name"].as_str()?.to_owned(), value: row["value"].as_str()?.to_owned(), signature: row["signature"].as_str().map(str::to_owned) }))
        .collect();
    SPOOFED.with(|spoofed| *spoofed.borrow_mut() = Some((uuid, properties)));
    (None, address)
}

/// An offline login's profile: the game's own, unless `PlayerHandshakeEvent` gave another.
pub fn handshake_profile(offline: u128) -> (u128, Vec<horizon_protocol::packets::login::Property>) {
    match SPOOFED.with(|spoofed| spoofed.borrow_mut().take()) {
        Some((uuid, properties)) => (uuid.unwrap_or(offline), properties),
        None => (offline, Vec::new()),
    }
}

/// `PlayerCodeOfConductSendEvent`: the text the client gets, if any.
pub fn code_of_conduct(text: Option<&str>) -> Option<String> {
    if !horizon_mod_runtime::listens("pl_conduct") {
        return text.map(str::to_owned);
    }
    let mut payload = CONNECTION.with(|connection| connection.borrow().clone());
    if payload.is_null() {
        payload = serde_json::json!({});
    }
    payload["code_of_conduct"] = serde_json::json!(text);
    match carried_value("pl_conduct", payload) {
        Some(value) => value["code_of_conduct"].as_str().map(str::to_owned),
        None => text.map(str::to_owned),
    }
}

/// `IllegalPacketEvent` for a packet that can't be read: `None` if a listener keeps them on, else the
/// reason for the log and what the client is told.
pub fn illegal_packet(id: i32, error: &str) -> Option<(String, Option<Tag>)> {
    let reason = format!("Invalid packet {id}: {error}");
    if !horizon_mod_runtime::listens("pl_illegal") {
        return Some((reason, None));
    }
    let mut payload = CONNECTION.with(|connection| connection.borrow().clone());
    if payload.is_null() {
        payload = serde_json::json!({});
    }
    payload["type"] = serde_json::json!(id.to_string());
    payload["exception"] = serde_json::json!(error);
    let Some(value) = carried_value("pl_illegal", payload) else {
        return Some((reason, None));
    };
    if value["should_kick"] == false {
        return None;
    }
    Some((reason, value["kick_message"].as_str().map(horizon_protocol::text::literal)))
}

/// Whether a listener wants to be asked before a player gets out of bed.
pub static HOLD_WAKES: AtomicBool = AtomicBool::new(false);
/// True while the game wakes one it has asked about.
pub static WAKING: AtomicBool = AtomicBool::new(false);
static WAKES: Mutex<Vec<u128>> = Mutex::new(Vec::new());

/// `Hub::wake`, which has no game to ask: whether this one waits for the tick.
pub fn wake_held(player: u128, sleeping: bool) -> bool {
    if !sleeping || !HOLD_WAKES.load(Ordering::Relaxed) || WAKING.load(Ordering::Relaxed) {
        return false;
    }
    let mut wakes = WAKES.lock().unwrap_or_else(|e| e.into_inner());
    if !wakes.contains(&player) {
        wakes.push(player);
    }
    true
}

pub fn take_wakes() -> Vec<u128> {
    std::mem::take(&mut *WAKES.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Whether a listener wants to hear of the commands a player is sent.
pub static WATCH_COMMANDS: AtomicBool = AtomicBool::new(false);
static COMMANDS_SENT: Mutex<Vec<u128>> = Mutex::new(Vec::new());

/// `Hub::send_commands`, which has no game to ask: whether this one waits for the turn.
pub fn commands_sent(player: u128) -> bool {
    if !WATCH_COMMANDS.load(Ordering::Relaxed) {
        return false;
    }
    let mut sent = COMMANDS_SENT.lock().unwrap_or_else(|e| e.into_inner());
    if !sent.contains(&player) {
        sent.push(player);
    }
    true
}

pub fn take_commands_sent() -> Vec<u128> {
    std::mem::take(&mut *COMMANDS_SENT.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Whether a listener wants to be asked before an entity is shown to a player.
pub static WATCH_TRACKS: AtomicBool = AtomicBool::new(false);
/// Pairs of (viewer, entity), by id, that may be tracked.
static TRACK_ALLOWED: std::sync::RwLock<Vec<(i32, i32)>> = std::sync::RwLock::new(Vec::new());
static TRACK_ASKED: Mutex<Vec<(i32, i32)>> = Mutex::new(Vec::new());

/// The tracker's last word on a pair it would show: nothing to look at unless a listener wants asking.
pub fn track_allowed(viewer: i32, target: i32) -> bool {
    if !WATCH_TRACKS.load(Ordering::Relaxed) || TRACK_ALLOWED.read().unwrap_or_else(|e| e.into_inner()).contains(&(viewer, target)) {
        return true;
    }
    let mut asked = TRACK_ASKED.lock().unwrap_or_else(|e| e.into_inner());
    if !asked.contains(&(viewer, target)) {
        asked.push((viewer, target));
    }
    false
}

pub fn take_track_asked() -> Vec<(i32, i32)> {
    std::mem::take(&mut *TRACK_ASKED.lock().unwrap_or_else(|e| e.into_inner()))
}

pub fn set_track_allowed(pair: (i32, i32), allowed: bool) {
    let mut all = TRACK_ALLOWED.write().unwrap_or_else(|e| e.into_inner());
    all.retain(|known| *known != pair);
    if allowed {
        all.push(pair);
    }
}

/// `AsyncPlayerPreLoginEvent`, on the connection's thread as on Paper: the profile they go on with,
/// or why they don't.
pub fn pre_login(uuid: u128, name: &mut String, properties: Vec<horizon_protocol::packets::login::Property>) -> Result<(u128, Vec<horizon_protocol::packets::login::Property>), Tag> {
    use horizon_protocol::packets::login::Property;
    if !horizon_mod_runtime::listens("pl_pre_login") {
        return Ok((uuid, properties));
    }
    connection_json(uuid, name, &properties);
    let payload = CONNECTION.with(|connection| connection.borrow().clone());
    let Some(value) = carried_value("pl_pre_login", payload) else {
        return Ok((uuid, properties));
    };
    if value["refused"] == true {
        return Err(reason_tag(&value["kick_message"]));
    }
    let profile = &value["profile"];
    if profile.is_null() {
        return Ok((uuid, properties));
    }
    if let Some(changed) = profile["name"].as_str() {
        *name = changed.to_owned();
    }
    let uuid = profile["id"].as_str().and_then(|id| u128::from_str_radix(&id.replace('-', ""), 16).ok()).unwrap_or(uuid);
    let properties = profile["properties"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| Some(Property { name: row["name"].as_str()?.to_owned(), value: row["value"].as_str()?.to_owned(), signature: row["signature"].as_str().map(str::to_owned) }))
        .collect();
    Ok((uuid, properties))
}

/// Whether a listener wants to be asked before a kick.
pub static HOLD_KICKS: AtomicBool = AtomicBool::new(false);
/// True while the game does a kick it has asked about.
pub static KICKING: AtomicBool = AtomicBool::new(false);
static KICKS: Mutex<Vec<(u32, Tag)>> = Mutex::new(Vec::new());

/// `Hub::kick`, which has no game to ask: whether this one waits for the turn.
pub fn kick_held(conn: u32, reason: &Tag) -> bool {
    if !HOLD_KICKS.load(Ordering::Relaxed) || KICKING.load(Ordering::Relaxed) {
        return false;
    }
    KICKS.lock().unwrap_or_else(|e| e.into_inner()).push((conn, reason.clone()));
    true
}

pub fn take_kicks() -> Vec<(u32, Tag)> {
    std::mem::take(&mut *KICKS.lock().unwrap_or_else(|e| e.into_inner()))
}

/// `PlayerServerFullCheckEvent`, with no game to lend, as on Paper: what to refuse them with, if anything.
pub fn server_full(uuid: u128, name: &str, full: bool) -> Option<Tag> {
    let refusal = || horizon_protocol::text::translatable("multiplayer.disconnect.server_full", &[]);
    if !horizon_mod_runtime::listens("pl_full") {
        return full.then(refusal);
    }
    let payload = serde_json::json!({ "uuid": format!("{uuid:032x}"), "name": name, "allowed": !full });
    let Some(value) = carried_value("pl_full", payload) else {
        return full.then(refusal);
    };
    if value["allowed"] == true {
        return None;
    }
    Some(if value["kick_message"].is_null() { refusal() } else { reason_tag(&value["kick_message"]) })
}

/// `PlayerLinksSendEvent`: the links this connection gets.
pub fn server_links(links: &[horizon_protocol::packets::configuration::ServerLink]) -> Vec<horizon_protocol::packets::configuration::ServerLink> {
    use horizon_protocol::packets::configuration::{ServerLink, ServerLinkKind};
    if !horizon_mod_runtime::listens("pl_links") {
        return links.to_vec();
    }
    let rows: Vec<serde_json::Value> = links
        .iter()
        .map(|link| match &link.kind {
            ServerLinkKind::Known(index) => serde_json::json!({ "known": index, "url": link.link }),
            ServerLinkKind::Custom(label) => serde_json::json!({ "label": serde_json::from_str::<serde_json::Value>(&crate::lan::conn::component_json(label)).unwrap_or_default(), "url": link.link }),
        })
        .collect();
    let mut payload = CONNECTION.with(|connection| connection.borrow().clone());
    if payload.is_null() {
        payload = serde_json::json!({});
    }
    payload["links"] = serde_json::json!(rows);
    let Some(value) = carried_value("pl_links", payload) else {
        return links.to_vec();
    };
    value["links"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let kind = match row["known"].as_i64() {
                Some(index) => ServerLinkKind::Known(index as i32),
                None => ServerLinkKind::Custom(reason_tag(&row["label"])),
            };
            Some(ServerLink { kind, link: row["url"].as_str()?.to_owned() })
        })
        .collect()
}

/// `PlayerConnectionCloseEvent` for a login that never got to play.
pub fn connection_closed(uuid: u128, name: &str, address: std::net::SocketAddr) {
    if horizon_mod_runtime::listens("pl_conn_close") {
        horizon_mod_runtime::fire("pl_conn_close", &serde_json::json!({ "uuid": format!("{uuid:032x}"), "name": name, "address": address.to_string() }).to_string());
    }
}

thread_local! {
    /// This connection is back from play: not its initial configuration.
    static RECONFIGURING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
static RECONFIGURED: Mutex<Vec<u128>> = Mutex::new(Vec::new());

pub fn reconfiguring(on: bool) {
    RECONFIGURING.set(on);
}

/// `PlayerConnectionReconfigureEvent`, on the connection's thread.
pub fn reconfigure_event(uuid: u128) {
    if !horizon_mod_runtime::listens("pl_reconfigure") {
        return;
    }
    let payload = CONNECTION.with(|connection| connection.borrow().clone());
    if carried_value("pl_reconfigure", payload).is_some_and(|value| value["complete"] == true) {
        release_reconfiguration(uuid);
    }
}

/// `PlayerConfigurationConnection.completeReconfiguration`.
pub fn release_reconfiguration(uuid: u128) {
    RECONFIGURED.lock().unwrap_or_else(|e| e.into_inner()).push(uuid);
}

/// Whether this player has been let go: once.
pub fn reconfiguration_released(uuid: u128) -> bool {
    let mut released = RECONFIGURED.lock().unwrap_or_else(|e| e.into_inner());
    match released.iter().position(|known| *known == uuid) {
        Some(at) => {
            released.remove(at);
            true
        }
        None => false,
    }
}

// ---- What a thread with no game waits on: asked at the next tick ----

/// A question for the listeners: of what kind, what it says, the tick they first saw it and where the answer goes.
pub struct Ask {
    pub id: u32,
    pub kind: &'static str,
    pub json: String,
    pub seen: i64,
    pub answer: std::sync::mpsc::Sender<String>,
}

pub static ASKS: Mutex<Vec<Ask>> = Mutex::new(Vec::new());
/// When a listener last looked at a kind: with nobody looking, nothing waits.
pub static ASKS_POLLED: Mutex<Vec<(&'static str, std::time::Instant)>> = Mutex::new(Vec::new());
static NEXT_GAME_ASK: AtomicU32 = AtomicU32::new(1);

/// What listeners answered, or `None` with none of them there or too slow.
pub fn ask_game(kind: &'static str, json: String, wait: std::time::Duration) -> Option<String> {
    let looked = ASKS_POLLED.lock().unwrap_or_else(|e| e.into_inner()).iter().any(|(known, at)| *known == kind && at.elapsed() < std::time::Duration::from_secs(1));
    if !looked {
        return None;
    }
    let (answer, reply) = std::sync::mpsc::channel();
    let id = NEXT_GAME_ASK.fetch_add(1, Ordering::Relaxed);
    ASKS.lock().unwrap_or_else(|e| e.into_inner()).push(Ask { id, kind, json, seen: 0, answer });
    reply.recv_timeout(wait).ok()
}

/// `ProfileWhitelistVerifyEvent`: whether the player is let in, and the words if not.
pub fn whitelist_ask(name: &str, id: u128, enabled: bool, op: bool, listed: bool) -> Option<(bool, Option<String>)> {
    let question = serde_json::json!({ "name": name, "uuid": format!("{id:032x}"), "enabled": enabled, "op": op, "whitelisted": listed }).to_string();
    let answer: serde_json::Value = serde_json::from_str(&ask_game("whitelist", question, std::time::Duration::from_millis(400))?).ok()?;
    Some((answer["whitelisted"].as_bool()?, answer["kick"].as_str().map(str::to_owned)))
}

/// Whether nobody has this mark.
pub fn plugin_marks_empty(kind: usize) -> bool {
    PLUGIN_MARKS[kind].read().unwrap_or_else(|e| e.into_inner()).is_empty()
}

/// `Player.setHealthScale`: what a player's bar counts to, by entity id.
static HEALTH_SCALES: std::sync::RwLock<Vec<(i32, f64)>> = std::sync::RwLock::new(Vec::new());
static ANY_SCALE: AtomicBool = AtomicBool::new(false);

pub fn set_health_scale(entity: i32, scale: Option<f64>) {
    let mut all = HEALTH_SCALES.write().unwrap_or_else(|e| e.into_inner());
    all.retain(|(id, _)| *id != entity);
    if let Some(scale) = scale {
        all.push((entity, scale));
    }
    ANY_SCALE.store(!all.is_empty(), std::sync::atomic::Ordering::Relaxed);
}

pub fn health_scale(entity: i32) -> Option<f64> {
    if !ANY_SCALE.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    HEALTH_SCALES.read().unwrap_or_else(|e| e.into_inner()).iter().find(|(id, _)| *id == entity).map(|(_, scale)| *scale)
}

/// The registry's number for `minecraft:max_health`, for the packets that name it.
static MAX_HEALTH_ATTRIBUTE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);

pub fn note_max_health_attribute(id: Option<i32>) {
    MAX_HEALTH_ATTRIBUTE.store(id.unwrap_or(-1), std::sync::atomic::Ordering::Relaxed);
}

pub fn max_health_attribute() -> Option<i32> {
    Some(MAX_HEALTH_ATTRIBUTE.load(std::sync::atomic::Ordering::Relaxed)).filter(|id| *id >= 0)
}

/// `CraftPlayer.getScaledHealth`: what the client is told.
pub fn shown_health(entity: i32, health: f32, most: f64) -> f32 {
    match health_scale(entity) {
        Some(scale) => (f64::from(health) * scale / most.max(1.0)) as f32,
        None => health,
    }
}

/// `Entity.setSneaking` and `setInvisible` on what has no state of its own: the shared flags, by entity id.
static SHARED_FLAGS: std::sync::RwLock<Vec<(i32, i8)>> = std::sync::RwLock::new(Vec::new());
static ANY_FLAGS: AtomicBool = AtomicBool::new(false);

/// Stays listed at nothing, so that what was sent goes back to its default.
pub fn set_shared_flag(entity: i32, bit: i8, on: bool) {
    let mut all = SHARED_FLAGS.write().unwrap_or_else(|e| e.into_inner());
    if !all.iter().any(|(id, _)| *id == entity) {
        all.push((entity, 0));
    }
    for (id, flags) in all.iter_mut() {
        if *id == entity {
            *flags = if on { *flags | bit } else { *flags & !bit };
        }
    }
    ANY_FLAGS.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn shared_flags(entity: i32) -> Option<i8> {
    if !ANY_FLAGS.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    SHARED_FLAGS.read().unwrap_or_else(|e| e.into_inner()).iter().find(|(id, _)| *id == entity).map(|(_, flags)| *flags)
}

/// Players whose client is to rebuild its own player at the hub's next tick, by entity id.
static REFRESH: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());
static ANY_REFRESH: AtomicBool = AtomicBool::new(false);

pub fn ask_refresh(entity: i32) {
    REFRESH.lock().unwrap_or_else(|e| e.into_inner()).push(entity);
    ANY_REFRESH.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn take_refresh(entity: i32) -> bool {
    if !ANY_REFRESH.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }
    let mut all = REFRESH.lock().unwrap_or_else(|e| e.into_inner());
    let found = all.iter().position(|id| *id == entity).map(|at| all.remove(at));
    ANY_REFRESH.store(!all.is_empty(), std::sync::atomic::Ordering::Relaxed);
    found.is_some()
}

/// Which part of `ServerMobEffectEvents` a mob's effect has reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectStep {
    AllowAdd,
    BeforeAdd,
    AfterAdd,
    AllowRemove,
    BeforeRemove,
    AfterRemove,
}

/// A mob's effect: its id, the ticks left and the amplifier.
pub type MobEffectRow = (&'static str, i32, i32);

/// What's to become of a mob's effects. The mob's own code has no `Game` and a listener must have it, so
/// while anyone listens the change waits here for the first place that can lend it: the end of that mob's
/// turn in its tick, or the end of the work that made it. Nothing else runs in between, and it's the change
/// itself that waits, so `ALLOW_ADD` can still refuse and `BEFORE_ADD` still comes first.
#[derive(Clone, Debug)]
pub enum HeldEffect {
    /// `ALLOW_ADD` alone: the game turned the mob away.
    Asked { mob: u32, effect: MobEffectRow },
    Add { mob: u32, effect: MobEffectRow, ambient: bool, visible: bool, cause: &'static str, from: serde_json::Value },
    Remove { mob: u32, effect: MobEffectRow },
    Clear { mob: u32, cause: String },
    /// Ran out in the mob's tick, `BEFORE_REMOVE` said ahead of the tick.
    Gone { mob: u32, effect: MobEffectRow, ambient: bool, visible: bool },
    /// Ended itself in the mob's tick, and it's still on.
    Ending { mob: u32, effect: MobEffectRow },
    /// A hit made where the game can't be had and nothing reads its result.
    Hit { mob: u32, amount: f32, source: crate::mobs::Damage },
}

static EFFECTS_HEARD: AtomicBool = AtomicBool::new(false);

thread_local! {
    static HELD_EFFECTS: std::cell::RefCell<Vec<HeldEffect>> = const { std::cell::RefCell::new(Vec::new()) };
    /// A held change going through, the listeners told.
    static PASSING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Effects whose `BEFORE_REMOVE` came ahead of the tick that ends them.
    static TOLD: std::cell::RefCell<Vec<(u32, &'static str)>> = const { std::cell::RefCell::new(Vec::new()) };
}

static HITS_HEARD: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Hits on mobs wait: a mob's turn in its tick, or a site that doesn't read the result.
    static HOLDING_HITS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn hear_hit_asking(heard: bool) {
    HITS_HEARD.store(heard, std::sync::atomic::Ordering::Relaxed);
}

pub fn holding_hits(on: bool) {
    HOLDING_HITS.set(on);
}

/// `hurtServer` where the game can't be had: true if the hit waits for it.
pub fn hit_held(mob: u32, amount: f32, source: crate::mobs::Damage) -> bool {
    let held = HOLDING_HITS.get() && HITS_HEARD.load(std::sync::atomic::Ordering::Relaxed);
    if held {
        hold_effect(HeldEffect::Hit { mob, amount, source });
    }
    held
}

pub fn hear_effects(heard: bool) {
    EFFECTS_HEARD.store(heard, std::sync::atomic::Ordering::Relaxed);
}

/// Whether a change to a mob's effects waits for the game.
pub fn effects_heard() -> bool {
    EFFECTS_HEARD.load(std::sync::atomic::Ordering::Relaxed) && !PASSING.get()
}

pub fn passing() -> bool {
    PASSING.get()
}

/// Runs a held change: the mob's code does it and asks no one.
pub fn passing_effects<R>(run: impl FnOnce() -> R) -> R {
    let before = PASSING.replace(true);
    let result = run();
    PASSING.set(before);
    result
}

pub fn hold_effect(change: HeldEffect) {
    HELD_EFFECTS.with(|held| held.borrow_mut().push(change));
}

pub fn hold_add(mob: u32, effect: &'static str, ticks: i32, amplifier: i32, ambient: bool, visible: bool) {
    let cause = crate::mobs::ee::effect_cause_now();
    let from = crate::mobs::ee::take_effect_from();
    hold_effect(HeldEffect::Add { mob, effect: (effect, ticks, amplifier), ambient, visible, cause, from });
}

pub fn take_held() -> Vec<HeldEffect> {
    HELD_EFFECTS.with(|held| std::mem::take(&mut *held.borrow_mut()))
}

pub fn effects_waiting() -> bool {
    HELD_EFFECTS.with(|held| !held.borrow().is_empty())
}

pub fn effect_told(mob: u32, effect: &'static str) {
    TOLD.with(|told| told.borrow_mut().push((mob, effect)));
}

/// The effects `tickEffects` just ended on `mob`: those that ended themselves stay on for the game to take, and
/// come back here.
pub fn effects_ended(mob: &Mob, ended: &[MobEffectRow]) -> Vec<&'static str> {
    let mut stay = Vec::new();
    if !effects_heard() {
        return stay;
    }
    TOLD.with(|told| {
        let mut told = told.borrow_mut();
        for effect in ended {
            match told.iter().position(|row| *row == (mob.id, effect.0)) {
                Some(at) => {
                    told.remove(at);
                    let ambient = mob.extra.ambient_effects.contains(&effect.0);
                    let visible = !mob.extra.hidden_effects.contains(&effect.0);
                    hold_effect(HeldEffect::Gone { mob: mob.id, effect: *effect, ambient, visible });
                }
                None => {
                    stay.push(effect.0);
                    hold_effect(HeldEffect::Ending { mob: mob.id, effect: *effect });
                }
            }
        }
        told.retain(|row| row.0 != mob.id);
    });
    stay
}

/// `ALLOW_ADD` where the game turns a mob away without ever reaching `add_effect`.
pub fn mob_asked(mob: &Mob, effect: MobEffectRow) {
    if effects_heard() {
        hold_effect(HeldEffect::Asked { mob: mob.id, effect });
    }
}

type HitLanded = fn(i32, f32, bool, bool);
type MayDie = fn(&mut Mob, f32, bool);
static LIVING_GATES: std::sync::OnceLock<(HitLanded, MayDie)> = std::sync::OnceLock::new();

pub fn hear_hits(landed: HitLanded, may_die: MayDie) {
    let _ = LIVING_GATES.set((landed, may_die));
}

/// The end of `hurtServer`, for anything: what's left of the hit before armour, whether something blocked it, and
/// whether what it hit lives.
pub fn hit_landed(entity: i32, taken: f32, blocked: bool, alive: bool) {
    if let Some((landed, _)) = LIVING_GATES.get() {
        landed(entity, taken, blocked, alive);
    }
}

/// A fatal hit waiting for `ALLOW_DEATH`, which comes with `die` where the game is in hand.
#[derive(Clone, Debug)]
pub struct DeathAsked {
    pub damage_type: String,
    pub causing: i32,
    pub direct: i32,
    pub amount: f32,
    /// The hit goes past a totem.
    pub bypass: bool,
}

static DEATHS_HEARD: AtomicBool = AtomicBool::new(false);

pub fn hear_deaths(heard: bool) {
    DEATHS_HEARD.store(heard, std::sync::atomic::Ordering::Relaxed);
}

/// A mob's fatal hit. With anyone listening, `ALLOW_DEATH`, the totem and `die` all wait for the game, and the
/// mob lies at no health until then: true when they don't.
pub fn mob_may_die(mob: &mut Mob, taken: f32, bypass: bool) -> bool {
    DEATHS.store(true, std::sync::atomic::Ordering::Relaxed);
    if !DEATHS_HEARD.load(std::sync::atomic::Ordering::Relaxed) {
        return true;
    }
    mob.health = 0.0;
    if let Some((_, may_die)) = LIVING_GATES.get() {
        may_die(mob, taken, bypass);
    }
    false
}

/// A mob has a death waiting for `die`.
static DEATHS: AtomicBool = AtomicBool::new(false);

pub fn take_deaths() -> bool {
    DEATHS.swap(false, std::sync::atomic::Ordering::Relaxed)
}

type TagSet = std::collections::HashSet<String>;
/// The entity tags `canBeAffected` asks, as of this tick: a mob's own code can't reach them.
static EFFECT_TAGS: std::sync::RwLock<[Option<TagSet>; 3]> = std::sync::RwLock::new([None, None, None]);

pub fn effect_tags(infested: Option<&TagSet>, oozing: Option<&TagSet>, ignores: Option<&TagSet>) {
    let size = |set: Option<&TagSet>| set.map_or(0, TagSet::len);
    let held = EFFECT_TAGS.read().unwrap_or_else(|e| e.into_inner());
    let same = [infested, oozing, ignores].iter().zip(held.iter()).all(|(new, old)| size(*new) == size(old.as_ref()));
    drop(held);
    if !same {
        *EFFECT_TAGS.write().unwrap_or_else(|e| e.into_inner()) = [infested.cloned(), oozing.cloned(), ignores.cloned()];
    }
}

/// `LivingEntity.canBeAffected` with the mobs' overrides.
pub fn mob_affected(kind: &str, effect: &str) -> bool {
    let effect = effect.trim_start_matches("minecraft:");
    let overridden = match kind.trim_start_matches("minecraft:") {
        "wither" | "wither_skeleton" => effect == "wither",
        "spider" | "cave_spider" | "nautilus" | "zombie_nautilus" => effect == "poison",
        "parched" => effect == "weakness",
        _ => false,
    };
    if overridden {
        return false;
    }
    let tags = EFFECT_TAGS.read().unwrap_or_else(|e| e.into_inner());
    let holds = |at: usize| tags[at].as_ref().is_some_and(|set| set.contains(kind));
    if holds(0) {
        effect != "infested"
    } else if holds(1) {
        effect != "oozing"
    } else {
        !holds(2) || (effect != "regeneration" && effect != "poison")
    }
}
