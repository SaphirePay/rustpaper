//! The interface between the Horizon Paper engine and a plugin library: a plugin loads at run time,
//! builds in seconds, needs none of the game's code, and keeps working when the game is rebuilt.
//! Plugins use it through `horizon-bukkit`, whose `bukkit_plugin!` exports it from `plugin.yml`.
//!
//! ```ignore
//! fn start(host: horizon_mod::Host) {
//!     host.on("join", |event| {
//!         horizon_mod::tell(event["uuid"].as_str().unwrap_or_default(), "Welcome!");
//!         horizon_mod::Answer::Pass
//!     });
//! }
//! horizon_mod::mod_entry!(start, r#"{"name":"Welcome","version":"1.0"}"#);
//! ```
pub use serde_json::Value;
use serde_json::json;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Mutex, OnceLock},
};

/// The interface this SDK speaks. The game refuses a library built for another: 2 was the Fabric
/// engine's (a `fabric.mod.json` manifest), 3 is the Paper engine's (a plugin description as JSON).
pub const ABI: u32 = 3;

pub type RawEventFn = extern "C" fn(*const u8, usize, *const u8, usize) -> i32;

#[repr(C)]
pub struct HostApi {
    pub abi: u32,
    pub subscribe: extern "C" fn(*const u8, usize, RawEventFn),
    pub call: extern "C" fn(*const u8, usize),
    pub query: extern "C" fn(*const u8, usize, *mut usize) -> *const u8,
}

/// What a listener decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Leave it to the next listener and the game.
    Pass,
    /// Take the action, as handled.
    Success,
    /// Refuse it.
    Fail,
    /// Go on, with this amount in thousandths in place of the game's.
    Amount(i32),
    /// Take back what a listener before this one refused: `setCancelled(false)`.
    Allow,
}

type Listener = std::sync::Arc<dyn Fn(&Value) -> Answer + Send + Sync>;

static LISTENERS: Mutex<Vec<(String, Listener)>> = Mutex::new(Vec::new());
/// The events the game has been told of, which outlive their listeners.
static SUBSCRIBED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Where the list of listeners has got to: `drop_listeners_from` goes back to it.
pub fn listener_mark() -> usize {
    LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).len()
}

/// Every listener added since `mark` stops hearing anything.
pub fn drop_listeners_from(mark: usize) {
    LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).truncate(mark);
}
static HOST: OnceLock<&'static HostApi> = OnceLock::new();

/// A panic here would cross into the game: it is caught, logged and answers Pass.
extern "C" fn dispatch(event: *const u8, event_length: usize, payload: *const u8, payload_length: usize) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the game passes readable bytes that last for this call.
        let (event, payload) = unsafe { (std::slice::from_raw_parts(event, event_length), std::slice::from_raw_parts(payload, payload_length)) };
        let (event, payload) = (std::str::from_utf8(event).unwrap_or(""), serde_json::from_slice(payload).unwrap_or(Value::Null));
        // Out of the lock: a listener may do anything, and may reach the list.
        let listeners: Vec<Listener> = LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).iter().filter(|(name, _)| name == event).map(|(_, listener)| listener.clone()).collect();
        for listener in listeners {
            match listener(&payload) {
                Answer::Pass => {}
                Answer::Success => return 1,
                Answer::Fail => return 2,
                Answer::Amount(thousandths) => return 3 + thousandths.max(0),
                Answer::Allow => return -1,
            }
        }
        0
    }));
    result.unwrap_or_else(|_| {
        log("a listener panicked");
        0
    })
}

fn send(action: Value) {
    if let Some(host) = HOST.get() {
        let text = action.to_string();
        (host.call)(text.as_ptr(), text.len());
    }
}

/// Asks the game something, inside an event or a command: `Value::Null` for what isn't there.
pub fn query(request: Value) -> Value {
    let Some(host) = HOST.get() else {
        return Value::Null;
    };
    let asked = std::time::Instant::now();
    let text = request.to_string();
    let mut length = 0;
    let answer = (host.query)(text.as_ptr(), text.len(), &raw mut length);
    // SAFETY: the game's answer is readable until this thread's next query, and it is read here.
    let bytes = unsafe { std::slice::from_raw_parts(answer, length) };
    let value: Value = serde_json::from_slice(bytes).unwrap_or(Value::Null);
    if value["error"].as_str() == Some(NO_GAME) {
        if let Some(hop) = OFF_THREAD.get() {
            let started = std::time::Instant::now();
            let op = request["op"].as_str().unwrap_or("?").to_owned();
            let answer = hop(request);
            query_timing(&op, started, true);
            return answer;
        }
    }
    query_timing(request["op"].as_str().unwrap_or("?"), asked, false);
    value
}

/// Queries that had to wait for the game (no game lent on this thread): every 30 s, the ones that cost
/// more than 300 ms go to the log, so a slow plugin shows where its time goes.
fn query_timing(op: &str, started: std::time::Instant, hopped: bool) {
    type Table = Option<(std::time::Instant, std::collections::HashMap<String, (u64, u64, std::time::Duration)>)>;
    static TABLE: Mutex<Table> = Mutex::new(None);
    let spent = started.elapsed();
    let mut held = TABLE.lock().unwrap_or_else(|e| e.into_inner());
    let (since, table) = held.get_or_insert_with(|| (std::time::Instant::now(), std::collections::HashMap::new()));
    let row = table.entry(op.to_owned()).or_insert((0, 0, std::time::Duration::ZERO));
    row.0 += 1;
    row.1 += u64::from(hopped);
    row.2 += spent;
    if since.elapsed() >= std::time::Duration::from_secs(30) {
        let mut rows: Vec<_> = table.drain().filter(|(_, r)| r.2 >= std::time::Duration::from_millis(300)).collect();
        *since = std::time::Instant::now();
        drop(held);
        rows.sort_by(|a, b| b.1.2.cmp(&a.1.2));
        for (op, (count, hopped, total)) in rows.into_iter().take(5) {
            log(&format!("[queries] {op}: {:.0} ms in 30 s over {count} queries, {hopped} waited for the game", total.as_secs_f64() * 1000.0));
        }
    }
}

const NO_GAME: &str = "a query needs an event that has the game";
static OFF_THREAD: std::sync::OnceLock<fn(Value) -> Value> = std::sync::OnceLock::new();

/// What a query does when this thread has no game: Java plugins read the server from any thread.
pub fn on_query_without_game(hop: fn(Value) -> Value) {
    let _ = OFF_THREAD.set(hop);
}

/// The handle `start` gets, to listen with.
#[derive(Clone, Copy)]
pub struct Host;

impl Host {
    /// Listens to `event`: `join`, `disconnect`, `tick`, `break_before`, `break_after`, `use_block`,
    /// `attack_block`, `use_item`, `use_entity`, `attack_entity`, `allow_damage`, `player_death`,
    /// `respawn`, `change_level`, `allow_chat`, `command`.
    pub fn on(&self, event: &str, listener: impl Fn(&Value) -> Answer + Send + Sync + 'static) {
        LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).push((event.to_owned(), std::sync::Arc::new(listener)));
        // `dispatch` runs every listener of the event, so the event is subscribed once: a second
        // subscription would run them all twice.
        let mut subscribed = SUBSCRIBED.lock().unwrap_or_else(|e| e.into_inner());
        let first = !subscribed.iter().any(|name| name == event);
        if first {
            subscribed.push(event.to_owned());
        }
        drop(subscribed);
        if first && let Some(host) = HOST.get() {
            (host.subscribe)(event.as_ptr(), event.len(), dispatch);
        }
    }

    /// Adds a command, by the game's usage text: `home <name:word>`. `level` 0 is every player.
    pub fn command(&self, usage: &str, level: u8) {
        send(json!({ "op": "register_command", "text": usage, "level": level }));
    }
}

/// A chat line to one player.
pub fn tell(player: &str, text: &str) {
    send(json!({ "op": "tell", "player": player, "text": text }));
}

/// A line over one player's hotbar.
pub fn action_bar(player: &str, text: &str) {
    send(json!({ "op": "action_bar", "player": player, "text": text }));
}

/// A line to everyone and the log.
pub fn broadcast(text: &str) {
    send(json!({ "op": "broadcast", "text": text }));
}

/// The answer to the command being run.
pub fn feedback(text: &str) {
    send(json!({ "op": "feedback", "text": text }));
}

/// One of the game's own commands, run as the server's console at the next safe moment. A plugin that
/// took the word doesn't hear it: this is how the SDK does its work, so `tp` here stays the game's `tp`.
pub fn run_command(line: &str) {
    send(json!({ "op": "run_command", "text": line }));
}

/// A line as the console typed it, at the next safe moment: a word a plugin took is that plugin's.
pub fn dispatch_command(line: &str) {
    send(json!({ "op": "dispatch_command", "text": line }));
}

/// A line in the server's log.
pub fn log(text: &str) {
    send(json!({ "op": "log", "text": text }));
}

#[doc(hidden)]
pub fn startup(host: *const HostApi, start: fn(Host)) {
    // SAFETY: the game's table lives for as long as this library is loaded.
    if let Some(host) = unsafe { host.as_ref() } {
        let _ = HOST.set(host);
        start(Host);
    }
}

/// Exports what the game looks for: your `start` function and your description, the JSON form of
/// `plugin.yml` (`name`, `version`, `depend`, `softdepend`, `loadbefore`, `provides`, ...). The
/// description is worked out once, the first time the game asks.
#[macro_export]
macro_rules! mod_entry {
    ($start:path, $manifest:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn horizon_mod_abi() -> u32 {
            $crate::ABI
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn horizon_mod_manifest(length: *mut usize) -> *const u8 {
            static MANIFEST: ::std::sync::OnceLock<String> = ::std::sync::OnceLock::new();
            let text: &'static str = MANIFEST.get_or_init(|| ::std::string::String::from($manifest));
            // SAFETY: the game passes a valid place for the length.
            unsafe { *length = text.len() };
            text.as_ptr()
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn horizon_mod_init(host: *const $crate::HostApi) {
            $crate::startup(host, $start);
        }
    };
}
