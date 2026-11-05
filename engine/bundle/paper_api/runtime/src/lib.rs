//! Loads mods from a folder while the game runs: each is a native library with a small C interface,
//! so it builds in seconds, on its own, and survives a rebuild of the game. The game itself forbids
//! `unsafe`, so what needs it lives here. Everything crosses as bytes of JSON, copied at once.
//!
//! A mod exports `horizon_mod_abi`, `horizon_mod_manifest` and `horizon_mod_init`. Mods run with the
//! game's own rights: this is no sandbox.
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex, RwLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

/// Raised only when the interface below changes shape.
pub const ABI: u32 = 3;

pub type EventFn = extern "C" fn(*const u8, usize, *const u8, usize) -> i32;

/// What the game hands a mod. It lives for as long as the mod is loaded.
#[repr(C)]
pub struct HostApi {
    pub abi: u32,
    pub subscribe: extern "C" fn(*const u8, usize, EventFn),
    pub call: extern "C" fn(*const u8, usize),
    /// Asks the game something: the answer's bytes last until this thread's next query.
    pub query: extern "C" fn(*const u8, usize, *mut usize) -> *const u8,
}

static HOST: HostApi = HostApi { abi: ABI, subscribe, call, query };

static ANSWERER: std::sync::RwLock<Option<fn(&str) -> String>> = std::sync::RwLock::new(None);

thread_local! {
    static REPLY: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

thread_local! {
    /// What `lend` holds out on this thread.
    static LENT: std::cell::Cell<*mut ()> = const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// Runs `listeners` with `value` open to `borrowed` on this thread.
pub fn lend<T, R>(value: &mut T, listeners: impl FnOnce() -> R) -> R {
    let before = LENT.with(|lent| lent.replace(std::ptr::from_mut(value).cast()));
    let result = listeners();
    LENT.with(|lent| lent.set(before));
    result
}

thread_local! {
    /// What `lend_movable` holds out on this thread.
    static MOVABLE: std::cell::Cell<*mut ()> = const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// Runs `body` with `value` open to `take_lent` on this thread. `value` is the caller's own `&mut`,
/// passed down to here, so what `body` does to it through `take_lent` is what the caller sees.
pub fn lend_movable<T, R>(value: &mut T, body: impl FnOnce() -> R) -> R {
    let before = MOVABLE.with(|lent| lent.replace(std::ptr::from_mut(value).cast()));
    let result = body();
    MOVABLE.with(|lent| lent.set(before));
    result
}

/// The value `lend_movable` holds out, moved out for `visit`, which hands it back. A `T` by the one
/// caller of each, the game with its hub. `None` outside `lend_movable`, or while it's out already.
pub fn take_lent<T, R>(visit: impl FnOnce(T) -> (T, R)) -> Option<R> {
    /// Past a panic in `visit` there's no value to put back, and the frames above read one.
    struct Abort;
    impl Drop for Abort {
        fn drop(&mut self) {
            std::process::abort();
        }
    }
    let slot = MOVABLE.with(|lent| lent.replace(std::ptr::null_mut())).cast::<T>();
    if slot.is_null() {
        return None;
    }
    let guard = Abort;
    // SAFETY: `lend_movable` took this from a `&mut T` it holds for `body`'s whole run, and the
    // frames above it don't touch the value until `body` returns. It's written back below before
    // anything can, and the guard ends the process if it can't be.
    let (value, result) = visit(unsafe { slot.read() });
    unsafe { slot.write(value) };
    std::mem::forget(guard);
    MOVABLE.with(|lent| lent.set(slot.cast()));
    Some(result)
}

/// What `lend` is holding out, which must be a `T` (the one caller of each is the bridge, with the
/// game). `None` outside `lend`.
pub fn borrowed<T, R>(visit: impl FnOnce(&mut T) -> R) -> Option<R> {
    let value = LENT.with(std::cell::Cell::get).cast::<T>();
    if value.is_null() {
        return None;
    }
    // SAFETY: `lend` took this from a `&mut T` it holds for the listeners' whole run, and the
    // caller under it doesn't touch the value until they return.
    Some(visit(unsafe { &mut *value }))
}

/// The game's side of a mod's questions.
pub fn set_answerer(answerer: fn(&str) -> String) {
    *ANSWERER.write().unwrap_or_else(|e| e.into_inner()) = Some(answerer);
}

// ---- Packets a plugin listens to (ProtocolLib): which, and what it changed in the one being sent ----

static PACKETS_LISTENED: RwLock<Vec<String>> = RwLock::new(Vec::new());
thread_local! {
    static PACKET_EDITS: std::cell::RefCell<Option<Vec<(String, serde_json::Value)>>> = const { std::cell::RefCell::new(None) };
}

/// Whether any plugin listens to packets at all: the one check every packet pays.
pub fn any_packet_listened() -> bool {
    PACKETS_LISTENED.read().is_ok_and(|listened| !listened.is_empty())
}

/// Whether a plugin listens to the play packet of this name (`BlockUpdate`).
pub fn packet_listened(name: &str) -> bool {
    let listened = PACKETS_LISTENED.read().unwrap_or_else(|e| e.into_inner());
    !listened.is_empty() && listened.iter().any(|known| known == name || known == "*")
}

/// Before `fire("pl_send", ..)`: the listeners' changes to the packet are kept from here.
pub fn begin_packet() {
    PACKET_EDITS.with(|edits| *edits.borrow_mut() = Some(Vec::new()));
}

/// After it: the fields the listeners changed, in order.
pub fn take_packet_edits() -> Vec<(String, serde_json::Value)> {
    PACKET_EDITS.with(|edits| edits.borrow_mut().take()).unwrap_or_default()
}

// ---- Bukkit's services, which plugins register in onEnable, before any game is lent ----

static SERVICES: Mutex<Vec<serde_json::Value>> = Mutex::new(Vec::new());
/// What a provider answered a call made while no game was lent (`core_reply`).
static UNLENT_REPLY: Mutex<Option<serde_json::Value>> = Mutex::new(None);
thread_local! {
    static UNLENT_CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn game_lent() -> bool {
    !LENT.with(std::cell::Cell::get).is_null() || !MOVABLE.with(std::cell::Cell::get).is_null()
}

/// The questions that need no game: answered here, on any thread, at any time (a service registered
/// in onEnable at startup was lost when it went to the game, which wasn't there yet).
fn packet_answer(request: &str) -> Option<String> {
    let interesting = ["\"packet_listen\"", "\"packet_edit\"", "\"service_add\"", "\"service_remove\"", "\"service_list\"", "\"core_command_call\"", "\"core_reply\""];
    if !interesting.iter().any(|op| request.contains(op)) {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(request).ok()?;
    match value["op"].as_str()? {
        "service_add" => {
            let mut services = SERVICES.lock().unwrap_or_else(|e| e.into_inner());
            services.retain(|row| row["name"] != value["name"] || row["plugin"] != value["plugin"]);
            services.push(serde_json::json!({ "name": value["name"], "plugin": value["plugin"], "priority": value["priority"] }));
            // `RegisteredServiceProvider.compareTo`: the highest first, in the order they came.
            let rank = |row: &serde_json::Value| ["Lowest", "Low", "Normal", "High", "Highest"].iter().position(|name| row["priority"].as_str().is_some_and(|p| p.eq_ignore_ascii_case(name))).unwrap_or(2);
            services.sort_by_key(|row| std::cmp::Reverse(rank(row)));
            Some("true".to_owned())
        }
        "service_remove" => {
            SERVICES.lock().unwrap_or_else(|e| e.into_inner()).retain(|row| row["name"] != value["name"] || row["plugin"] != value["plugin"]);
            Some("true".to_owned())
        }
        "service_list" => {
            let services = SERVICES.lock().unwrap_or_else(|e| e.into_inner());
            Some(serde_json::Value::Array(services.iter().filter(|row| value["name"].is_null() || row["name"] == value["name"]).cloned().collect()).to_string())
        }
        // A call to another plugin while no game is lent (in onEnable): straight to it. With a game, the
        // game lends itself to the provider (the engine's own answer).
        "core_command_call" if !game_lent() => {
            let outer = UNLENT_REPLY.lock().unwrap_or_else(|e| e.into_inner()).take();
            UNLENT_CALLS.with(|calls| calls.set(calls.get() + 1));
            fire("core_command_call", request);
            UNLENT_CALLS.with(|calls| calls.set(calls.get() - 1));
            let answer = std::mem::replace(&mut *UNLENT_REPLY.lock().unwrap_or_else(|e| e.into_inner()), outer);
            Some(answer.unwrap_or(serde_json::Value::Null).to_string())
        }
        "core_reply" if UNLENT_CALLS.with(std::cell::Cell::get) > 0 && !game_lent() => {
            *UNLENT_REPLY.lock().unwrap_or_else(|e| e.into_inner()) = Some(value["value"].clone());
            Some("true".to_owned())
        }
        "packet_listen" => {
            let name = value["name"].as_str()?.to_owned();
            let mut listened = PACKETS_LISTENED.write().unwrap_or_else(|e| e.into_inner());
            if !listened.contains(&name) {
                listened.push(name);
            }
            Some("true".to_owned())
        }
        "packet_edit" => {
            let field = value["field"].as_str()?.to_owned();
            let kept = PACKET_EDITS.with(|edits| edits.borrow_mut().as_mut().map(|list| list.push((field, value["value"].clone()))).is_some());
            Some(kept.to_string())
        }
        _ => None,
    }
}

extern "C" fn query(json: *const u8, length: usize, reply_length: *mut usize) -> *const u8 {
    // SAFETY: the SDK passes readable bytes for this call, and a writable length.
    let request = std::str::from_utf8(unsafe { std::slice::from_raw_parts(json, length) }).unwrap_or("");
    let answerer = *ANSWERER.read().unwrap_or_else(|e| e.into_inner());
    let started = profile::on().then(std::time::Instant::now);
    let answer = packet_answer(request).unwrap_or_else(|| answerer.map_or_else(|| "null".to_owned(), |answerer| answerer(request)));
    if let Some(started) = started {
        profile::query(request, started, answer.len());
    }
    REPLY.with(|reply| {
        let mut reply = reply.borrow_mut();
        *reply = answer;
        // SAFETY: as above; the String stays in the thread local until the next query.
        unsafe { *reply_length = reply.len() };
        reply.as_ptr()
    })
}

struct Loaded {
    id: String,
    manifest: String,
    path: PathBuf,
    subscriptions: Vec<(String, EventFn)>,
    // Last: a library outlives the function pointers taken from it.
    library: Library,
}

/// One listener of an event. A mod's own `host.on` is one name for the whole mod, and its answer
/// ends the run, as a plain listener's does. A Bukkit listener has `event#priority#number`, and all run.
struct Route {
    slot: u8,
    bukkit: bool,
    name: String,
    callback: EventFn,
    /// The mod the listener belongs to, for `HORIZON_PLUGIN_PROFILE`.
    owner: String,
    /// HorizonsX: its mod says `folia-supported: true` (Folia's `plugin.yml` key: the plugin is safe on several
    /// region threads at once), so the game's threads run it without taking turns, as Folia does.
    free: bool,
}

/// Whether a mod's manifest (its `plugin.yml`, or a mod's JSON) says it is safe on Folia's region threads.
fn folia_supported(manifest: &str) -> bool {
    manifest.lines().any(|line| {
        let line = line.trim().trim_start_matches(['{', ',', ' ']).replace(['"', ' '], "");
        line.starts_with("folia-supported:true") || line.starts_with("folia_supported:true")
    })
}

static MODS: RwLock<Vec<Loaded>> = RwLock::new(Vec::new());
/// Each event's listeners in the order they run, across every mod.
static ROUTES: RwLock<Option<HashMap<String, std::sync::Arc<Vec<Route>>>>> = RwLock::new(None);
/// Held by each run of listeners: the libraries stay loaded while their functions are in use. A
/// library can come in during a run, as `PluginManager.loadPlugin` has it, as nothing else is held.
static IN_USE: RwLock<()> = RwLock::new(());
/// Subscriptions of the mod being initialised, which holds no lock on `MODS` meanwhile.
static STARTING: Mutex<Vec<(String, EventFn)>> = Mutex::new(Vec::new());
static ACTIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SUBSCRIBED: AtomicUsize = AtomicUsize::new(0);

fn text<'a>(pointer: *const u8, length: usize) -> &'a str {
    if pointer.is_null() {
        return "";
    }
    // SAFETY: the caller promises `length` readable bytes, for the length of this call.
    std::str::from_utf8(unsafe { std::slice::from_raw_parts(pointer, length) }).unwrap_or("")
}

extern "C" fn subscribe(name: *const u8, length: usize, callback: EventFn) {
    let name = text(name, length).to_owned();
    if INITIALIZING.load(Ordering::Relaxed) {
        STARTING.lock().unwrap_or_else(|e| e.into_inner()).push((name, callback));
    } else {
        // `registerEvents` at any time, not only as a plugin starts: the plugin whose code is running
        // owns it, so a hot swap takes it out with that plugin.
        let owner = CALLING.with(|calling| {
            let route = calling.get();
            // SAFETY: set only for the length of a listener's call, while its route list is held.
            if route.is_null() { "late".to_owned() } else { unsafe { (*route).owner.clone() } }
        });
        LATE.lock().unwrap_or_else(|e| e.into_inner()).push((owner, name, callback));
        LATE_WAITING.store(true, Ordering::Relaxed);
    }
}

thread_local! {
    /// The listener this thread is running, for whose `registerEvents` a late subscription is.
    static CALLING: std::cell::Cell<*const Route> = const { std::cell::Cell::new(std::ptr::null()) };
}

/// Runs `call` as `route`'s listener: what it subscribes to is its owner's.
fn calling<T>(route: &Route, call: impl FnOnce() -> T) -> T {
    let outer = CALLING.with(|calling| calling.replace(route));
    let out = call();
    CALLING.with(|calling| calling.set(outer));
    out
}

/// A mod is starting: what it subscribes to is its own.
static INITIALIZING: AtomicBool = AtomicBool::new(false);
/// Subscriptions made after a mod started: the owner, the event, the callback.
static LATE: Mutex<Vec<(String, String, EventFn)>> = Mutex::new(Vec::new());
/// Hot reload: libraries of plugins that were swapped out. They stay loaded, so a thread of theirs still
/// running (an async task finishing) never runs code that is gone; none of their listeners is called.
static RETIRED: Mutex<Vec<Loaded>> = Mutex::new(Vec::new());
static LATE_WAITING: AtomicBool = AtomicBool::new(false);

/// Puts listeners added since the last call in the order. It waits if an event is running, on this thread or another.
pub fn adopt_late() {
    if !LATE_WAITING.swap(false, Ordering::Relaxed) {
        return;
    }
    let (Ok(mods), Ok(mut routes)) = (MODS.try_read(), ROUTES.try_write()) else {
        LATE_WAITING.store(true, Ordering::Relaxed);
        return;
    };
    SUBSCRIBED.fetch_add(1, Ordering::Relaxed);
    *routes = Some(routed(&mods));
}

extern "C" fn call(json: *const u8, length: usize) {
    push(text(json, length).to_owned());
}

/// Queues an action for the game to carry out at a safe moment.
pub fn push(action: String) {
    ACTIONS.lock().unwrap_or_else(|e| e.into_inner()).push(action);
}

/// The actions the mods have asked for, oldest first.
pub fn take_actions() -> Vec<String> {
    std::mem::take(&mut *ACTIONS.lock().unwrap_or_else(|e| e.into_inner()))
}

fn is_library(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "dll" || e == "so" || e == "dylib")
}

fn open(path: &Path) -> Result<Loaded, String> {
    // SAFETY: a mod is trusted native code: loading one runs whatever it holds.
    let library = unsafe { Library::new(path) }.map_err(|e| format!("{}: {e}", path.display()))?;
    // SAFETY: these symbols are the mod interface's, with these exact signatures.
    let (abi, manifest) = unsafe {
        let abi: Symbol<extern "C" fn() -> u32> = library.get(b"horizon_mod_abi\0").map_err(|e| format!("{}: {e}", path.display()))?;
        let manifest: Symbol<extern "C" fn(*mut usize) -> *const u8> = library.get(b"horizon_mod_manifest\0").map_err(|e| format!("{}: {e}", path.display()))?;
        let mut length = 0;
        let pointer = manifest(&mut length);
        (abi(), text(pointer, length).to_owned())
    };
    if abi != ABI {
        let why = if abi == 2 { "a Fabric engine library: rebuild it on the Paper SDK".to_owned() } else { format!("built for plugin interface {abi}, this engine has {ABI}") };
        return Err(format!("{}: {why}", path.display()));
    }
    Ok(Loaded { id: String::new(), manifest, path: path.to_owned(), subscriptions: Vec::new(), library })
}

/// Opens every library in `folder` and gives back each one's description with its file name, or why it
/// couldn't. Nothing runs yet: the loader checks dependencies first. A copy is what's opened, so the
/// file in `folder` can be rebuilt while the game holds its copy.
pub fn open_folder(folder: &Path) -> (Vec<(String, String)>, Vec<String>) {
    let (mut manifests, mut errors) = (Vec::new(), Vec::new());
    let Ok(entries) = fs::read_dir(folder) else {
        return (manifests, errors);
    };
    let loaded = folder.join(".loaded");
    let generation = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis());
    if let Ok(old) = fs::read_dir(&loaded) {
        for file in old.flatten() {
            let _ = fs::remove_file(file.path());
        }
    }
    let _ = fs::create_dir_all(&loaded);
    let mut mods = MODS.write().unwrap_or_else(|e| e.into_inner());
    for path in entries.flatten().map(|e| e.path()).filter(|p| is_library(p)) {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let copy = loaded.join(format!("{generation}-{name}"));
        if let Err(e) = fs::copy(&path, &copy) {
            errors.push(format!("{}: {e}", path.display()));
            continue;
        }
        match open(&copy) {
            Ok(module) => {
                manifests.push((module.manifest.clone(), name));
                mods.push(module);
            }
            Err(e) => errors.push(e),
        }
    }
    (manifests, errors)
}

/// Starts the mod whose manifest is `manifest`, once its dependencies have started.
pub fn init(id: &str, manifest: &str) -> Result<(), String> {
    let init_fn = {
        let mods = MODS.read().unwrap_or_else(|e| e.into_inner());
        let Some(module) = mods.iter().find(|m| m.manifest == manifest) else {
            return Ok(());
        };
        // SAFETY: the mod interface's own symbol and signature. The library stays in `MODS`.
        *unsafe { module.library.get::<extern "C" fn(*const HostApi)>(b"horizon_mod_init\0") }.map_err(|e| format!("{id}: {e}"))?
    };
    // Named before it starts, for the watchdog's report of a start that hangs.
    if let Some(module) = MODS.write().unwrap_or_else(|e| e.into_inner()).iter_mut().find(|m| m.manifest == manifest) {
        module.id = id.to_owned();
    }
    INITIALIZING.store(true, Ordering::Relaxed);
    watch::mark_server_thread();
    let watched = watch::enter(|| watch::Doing::Enabling(id.to_owned()));
    init_fn(&HOST);
    watch::leave(watched);
    INITIALIZING.store(false, Ordering::Relaxed);
    let subscriptions = std::mem::take(&mut *STARTING.lock().unwrap_or_else(|e| e.into_inner()));
    SUBSCRIBED.fetch_add(subscriptions.len(), Ordering::Relaxed);
    let mut mods = MODS.write().unwrap_or_else(|e| e.into_inner());
    if let Some(module) = mods.iter_mut().find(|m| m.manifest == manifest) {
        module.id = id.to_owned();
        module.subscriptions = subscriptions;
    }
    *ROUTES.write().unwrap_or_else(|e| e.into_inner()) = Some(routed(&mods));
    Ok(())
}

/// Reads each subscription's name into the order `fire` runs them.
fn routed(mods: &[Loaded]) -> HashMap<String, std::sync::Arc<Vec<Route>>> {
    let mut routes: HashMap<String, Vec<Route>> = HashMap::new();
    let late = LATE.lock().unwrap_or_else(|e| e.into_inner());
    let free: std::collections::HashSet<&str> =
        mods.iter().filter(|module| folia_supported(&module.manifest)).map(|module| module.id.as_str()).collect();
    let owned = mods.iter().flat_map(|module| module.subscriptions.iter().map(|(name, callback)| (name, *callback, module.id.as_str())));
    for (name, callback, owner) in owned.chain(late.iter().map(|(owner, name, callback)| (name, *callback, owner.as_str()))) {
        let mut parts = name.split('#');
        let event = parts.next().unwrap_or_default();
        let priority = parts.next().and_then(|slot| slot.parse().ok());
        routes.entry(event.to_owned()).or_default().push(Route { slot: priority.unwrap_or(NORMAL), bukkit: priority.is_some(), name: name.clone(), callback, owner: owner.to_owned(), free: free.contains(owner) });
    }
    for list in routes.values_mut() {
        list.sort_by_key(|route| (route.slot, route.bukkit));
    }
    routes.into_iter().map(|(event, list)| (event, std::sync::Arc::new(list))).collect()
}

/// `EventPriority.NORMAL`, where a mod's own listeners run.
const NORMAL: u8 = 2;
/// A listener that took a cancel back: `setCancelled(false)`.
const ALLOWED: i32 = -1;

thread_local! {
    /// What listeners changed for the ones after them: fields of the payload, as JSON.
    static PATCHES: std::cell::RefCell<Vec<(String, String)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// For the next listener of the event running: `key` reads as `value`.
pub fn patch(key: &str, value: &str) {
    PATCHES.with(|patches| {
        let mut patches = patches.borrow_mut();
        patches.retain(|(known, _)| known != key);
        patches.push((key.to_owned(), value.to_owned()));
    });
}

/// The payload as the next listener reads it: later keys of a JSON object win.
fn carried(payload: &str, answer: i32, patches: &[(String, String)]) -> String {
    let head = payload.trim_end().strip_suffix('}').unwrap_or("{");
    let mut out = head.to_owned();
    if out.trim_end() != "{" {
        out.push(',');
    }
    out.push_str(&format!("\"__answer\":{answer}"));
    for (key, value) in patches {
        out.push_str(&format!(",\"{key}\":{value}"));
    }
    out.push('}');
    out
}

/// Hot reload: plugin `id` leaves the event routes (its own listeners and the ones it added later), and its
/// library is kept loaded, retired (see `RETIRED`). Its `onDisable` ran before (`core_plugin_disable`).
/// False when an event is running on some thread: try again at the next tick's end.
pub fn retire(id: &str) -> Result<bool, String> {
    let Ok(_unused) = IN_USE.try_write() else {
        return Err("an event is running".to_owned());
    };
    let mut mods = MODS.write().unwrap_or_else(|e| e.into_inner());
    let Some(at) = mods.iter().position(|m| m.id == id) else {
        return Ok(false);
    };
    let module = mods.remove(at);
    let mut late = LATE.lock().unwrap_or_else(|e| e.into_inner());
    late.retain(|(owner, _, _)| owner != id);
    let count = mods.iter().map(|m| m.subscriptions.len()).sum::<usize>() + late.len();
    drop(late);
    SUBSCRIBED.store(count, Ordering::Relaxed);
    *ROUTES.write().unwrap_or_else(|e| e.into_inner()) = Some(routed(&mods));
    RETIRED.lock().unwrap_or_else(|e| e.into_inner()).push(module);
    Ok(true)
}

/// Hot reload: a library copied and opened off the server thread, waiting to be adopted.
pub struct Preloaded(Loaded);

impl Preloaded {
    /// What the library says of itself (`plugin.yml` as JSON).
    pub fn manifest(&self) -> &str {
        &self.0.manifest
    }
}

/// Copies `path` into `folder` under a fresh name and opens the copy: the slow part of loading a plugin
/// (a large library's file copy and its load), for a thread that isn't the server's. Nothing runs yet.
pub fn preload(path: &Path, folder: &Path) -> Result<Preloaded, String> {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    let _ = fs::create_dir_all(folder);
    let copy = folder.join(format!("{stamp}-{name}"));
    fs::copy(path, &copy).map_err(|e| format!("{}: {e}", path.display()))?;
    open(&copy).map(Preloaded)
}

/// A preloaded library joins the mods, ready for `init` by its manifest.
pub fn adopt(library: Preloaded) {
    MODS.write().unwrap_or_else(|e| e.into_inner()).push(library.0);
}

/// How many swapped-out libraries are kept loaded.
pub fn retired() -> usize {
    RETIRED.lock().unwrap_or_else(|e| e.into_inner()).len()
}

/// Lets every mod go. The libraries unload, and the next `open_folder` reads the files again.
pub fn close_all() {
    let _unused = IN_USE.write().unwrap_or_else(|e| e.into_inner());
    SUBSCRIBED.store(0, Ordering::Relaxed);
    *ROUTES.write().unwrap_or_else(|e| e.into_inner()) = None;
    LATE.lock().unwrap_or_else(|e| e.into_inner()).clear();
    MODS.write().unwrap_or_else(|e| e.into_inner()).clear();
}

/// HorizonsX's region threads (Folia's): several threads tick the world and fire events at once, each with its
/// own region's game lent (`lend`, per thread). The plugins' code was written for one game thread: the game's
/// threads (the server thread and the region threads, the `horizonsx-N` pool) take turns in it, so a plugin
/// never sees two of them at once. Re-entrant: a listener's own events run inside its turn. Other threads
/// (a connection's pre-login, a plugin's own async tasks) don't take turns, as before region threads: a
/// listener may wait on them (LuckPerms' pre-login waits for its loader, which fires events itself).
mod turns {
    use std::sync::{Condvar, Mutex};
    static OWNER: Mutex<Option<std::thread::ThreadId>> = Mutex::new(None);
    static FREE: Condvar = Condvar::new();
    thread_local! {
        static DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    }
    pub(crate) struct Turn(bool);
    /// A game thread: the server's, or one of HorizonsX's region threads (its pool's names).
    fn game_thread() -> bool {
        thread_local! {
            static REGION: bool = std::thread::current().name().is_some_and(|n| n.starts_with("horizonsx-"));
        }
        REGION.with(|r| *r) || super::watch::on_server_thread()
    }
    pub(crate) fn take() -> Turn {
        if !game_thread() {
            return Turn(false);
        }
        if DEPTH.with(|d| d.get()) == 0 {
            let me = std::thread::current().id();
            let mut owner = OWNER.lock().unwrap_or_else(|e| e.into_inner());
            while owner.is_some_and(|o| o != me) {
                owner = FREE.wait(owner).unwrap_or_else(|e| e.into_inner());
            }
            *owner = Some(me);
        }
        DEPTH.with(|d| d.set(d.get() + 1));
        Turn(true)
    }
    impl Drop for Turn {
        fn drop(&mut self) {
            if !self.0 {
                return;
            }
            let left = DEPTH.with(|d| {
                d.set(d.get() - 1);
                d.get()
            });
            if left == 0 {
                *OWNER.lock().unwrap_or_else(|e| e.into_inner()) = None;
                FREE.notify_one();
            }
        }
    }
}

/// Whether any mod listens at all: an event nobody hears builds no JSON.
pub fn listening() -> bool {
    SUBSCRIBED.load(Ordering::Relaxed) > 0
}

/// The names of an event's listeners, in the order they run.
pub fn listeners(event: &str) -> Vec<String> {
    let routes = ROUTES.read().unwrap_or_else(|e| e.into_inner());
    routes.as_ref().and_then(|routes| routes.get(event)).map_or_else(Vec::new, |list| list.iter().map(|route| route.name.clone()).collect())
}

/// Every event anyone listens to.
pub fn listened() -> Vec<String> {
    ROUTES.read().unwrap_or_else(|e| e.into_inner()).as_ref().map_or_else(Vec::new, |routes| routes.keys().cloned().collect())
}

/// One listener of `event`, by its name in `listeners`: `RegisteredListener.callEvent`.
pub fn fire_listener(event: &str, name: &str, payload: &str) -> (i32, Option<String>) {
    let _in_use = IN_USE.read().unwrap_or_else(|e| e.into_inner());
    let list = ROUTES.read().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|routes| routes.get(event).cloned());
    let Some(route) = list.as_ref().and_then(|list| list.iter().find(|route| route.name == name)) else {
        return (0, None);
    };
    // A Folia-supported plugin's listener needs no turn (Folia runs it on every region thread at once).
    let _turn = (!route.free).then(turns::take);
    let outer = PATCHES.with(|patches| std::mem::take(&mut *patches.borrow_mut()));
    let watched = list.as_ref().and_then(|list| list.iter().position(|route| route.name == name).map(|at| watch::enter(|| watch::Doing::Listener(list.clone(), at))));
    let said = calling(route, || (route.callback)(route.name.as_ptr(), route.name.len(), payload.as_ptr(), payload.len()));
    watch::leave(watched.unwrap_or(false));
    let answer = if said == ALLOWED { 0 } else { said };
    let text = PATCHES.with(|patches| (!patches.borrow().is_empty()).then(|| carried(payload, answer, &patches.borrow())));
    PATCHES.with(|patches| *patches.borrow_mut() = outer);
    (answer, text)
}

/// Whether a mod subscribed to `event`.
pub fn listens(event: &str) -> bool {
    listening() && ROUTES.read().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|routes| routes.contains_key(event))
}

/// Tells the mods that subscribed to `event`, in priority order. A mod's own listener ends the run
/// with its answer. Bukkit listeners all run, each seeing the answer so far: the last one stands.
pub fn fire(event: &str, payload: &str) -> i32 {
    fire_carried(event, payload).0
}

/// `fire`, with the payload as the listeners left it, if any of them changed it.
pub fn fire_carried(event: &str, payload: &str) -> (i32, Option<String>) {
    // The libraries stay loaded for as long as their function pointers are in use.
    let _in_use = IN_USE.read().unwrap_or_else(|e| e.into_inner());
    let list = ROUTES.read().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|routes| routes.get(event).cloned());
    let Some(list) = list else {
        return (0, None);
    };
    // Turns only while a listener of a plugin that isn't Folia-supported is in the run: Folia-supported ones
    // run on every region thread at once, as on Folia.
    let _turn = (!list.iter().all(|route| route.free)).then(turns::take);
    // A listener may fire another event: this run's changes are kept apart from that one's.
    let outer = PATCHES.with(|patches| std::mem::take(&mut *patches.borrow_mut()));
    let (mut answer, mut text) = (0, None::<String>);
    let profiling = profile::on();
    for (at, route) in list.iter().enumerate() {
        let now = text.as_deref().unwrap_or(payload);
        let started = profiling.then(|| profile::enter(&route.owner));
        let watched = watch::enter(|| watch::Doing::Listener(list.clone(), at));
        let said = calling(route, || (route.callback)(route.name.as_ptr(), route.name.len(), now.as_ptr(), now.len()));
        watch::leave(watched);
        if let Some(started) = started {
            profile::leave(event, started);
        }
        if !route.bukkit && said != 0 {
            answer = said;
            break;
        }
        let before = answer;
        match said {
            0 => {}
            ALLOWED => answer = 0,
            said => answer = said,
        }
        let changed = PATCHES.with(|patches| !patches.borrow().is_empty());
        if answer != before || changed {
            text = Some(PATCHES.with(|patches| carried(payload, answer, &patches.borrow())));
        }
    }
    PATCHES.with(|patches| *patches.borrow_mut() = outer);
    (answer, text)
}

/// What the server thread is running in the plugins, for the watchdog's crash report: Paper's
/// watchdog prints the stuck thread's Java stack, and this is ours. The engine knows the plugin and
/// the listener; a plugin converted by jport answers with its Java calls when asked
/// (`horizon_java_stack_ask` / `horizon_java_stack_take`, exported by jrt).
pub mod watch {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    pub(crate) enum Doing {
        Listener(Arc<Vec<super::Route>>, usize),
        Enabling(String),
    }

    static NOW: Mutex<Vec<(Doing, Instant)>> = Mutex::new(Vec::new());
    /// The server thread: the one plugins start on (`init`); "main" on a dedicated server.
    static SERVER_ID: std::sync::OnceLock<std::thread::ThreadId> = std::sync::OnceLock::new();

    thread_local! {
        /// Whether this is the server thread, once that is known.
        static SERVER: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    }

    pub(crate) fn mark_server_thread() {
        let _ = SERVER_ID.set(std::thread::current().id());
    }

    pub(crate) fn on_server_thread() -> bool {
        SERVER.with(|known| {
            if let Some(known) = known.get() {
                return known;
            }
            let Some(id) = SERVER_ID.get() else { return false };
            let here = *id == std::thread::current().id();
            known.set(Some(here));
            here
        })
    }

    /// The server thread goes into a plugin. Other threads aren't watched.
    pub(crate) fn enter(doing: impl FnOnce() -> Doing) -> bool {
        if !on_server_thread() {
            return false;
        }
        NOW.lock().unwrap_or_else(|e| e.into_inner()).push((doing(), Instant::now()));
        true
    }

    pub(crate) fn leave(entered: bool) {
        if entered {
            NOW.lock().unwrap_or_else(|e| e.into_inner()).pop();
        }
    }

    /// The plugin calls the server thread is inside, outermost first: "DonutShards: listener tick#2#5, 59.8 s".
    pub fn frames() -> Vec<String> {
        let Ok(now) = NOW.try_lock() else {
            return vec!["(the plugin calls could not be read)".to_owned()];
        };
        now.iter()
            .map(|(doing, since)| {
                let took = since.elapsed().as_secs_f32();
                match doing {
                    Doing::Listener(list, at) => list.get(*at).map_or_else(String::new, |route| format!("{}: listener {}, for {took:.1} s", route.owner, route.name)),
                    Doing::Enabling(id) => format!("{id}: onEnable, for {took:.1} s"),
                }
            })
            .collect()
    }

    /// The plugins the server thread is inside.
    fn owners() -> Vec<String> {
        let Ok(now) = NOW.try_lock() else {
            return Vec::new();
        };
        now.iter()
            .filter_map(|(doing, _)| match doing {
                Doing::Listener(list, at) => list.get(*at).map(|route| route.owner.clone()),
                Doing::Enabling(id) => Some(id.clone()),
            })
            .collect()
    }

    /// The crash report's stack, one frame a line, innermost first: each converted plugin's Java
    /// calls, as the server thread answers within `wait` (it answers at its next Java call: one stuck
    /// in the game or in a lock doesn't, and a line says so), then the plugin calls the engine made.
    pub fn report(wait: Duration) -> String {
        let frames = frames();
        if frames.is_empty() {
            return "(the server thread was not in a plugin)".to_owned();
        }
        let mut out = String::new();
        let mut engine_side = String::new();
        for frame in frames.iter().rev() {
            engine_side.push_str(&format!("plugin {frame}\n"));
        }
        type Ask = extern "C" fn();
        type Take = extern "C" fn(*mut usize) -> *const u8;
        let asked: Vec<(String, Take)> = {
            let Ok(mods) = super::MODS.try_read() else {
                out.push_str("(the plugins could not be asked for their Java calls)\n");
                out.push_str(&engine_side);
                return out;
            };
            let owners = owners();
            mods.iter()
                .filter(|module| owners.contains(&module.id))
                .filter_map(|module| {
                    // SAFETY: jrt's own symbols and signatures; the libraries stay loaded (`MODS`) until the process ends.
                    let ask = unsafe { module.library.get::<Ask>(b"horizon_java_stack_ask\0") };
                    let take = unsafe { module.library.get::<Take>(b"horizon_java_stack_take\0") };
                    let (Ok(ask), Ok(take)) = (ask, take) else {
                        out.push_str(&format!("({} can't tell its Java calls: it was converted before jport could, or isn't a converted plugin)\n", module.id));
                        return None;
                    };
                    ask();
                    Some((module.id.clone(), *take))
                })
                .collect()
        };
        let until = Instant::now() + wait;
        let mut waiting: Vec<(String, Take)> = asked;
        let mut answered = Vec::new();
        while !waiting.is_empty() && Instant::now() < until {
            waiting.retain(|(id, take)| {
                let mut length = 0usize;
                let text = take(&mut length);
                if text.is_null() {
                    return true;
                }
                // SAFETY: jrt keeps the answer's bytes until its next ask.
                let bytes = unsafe { std::slice::from_raw_parts(text, length) };
                answered.push((id.clone(), String::from_utf8_lossy(bytes).into_owned()));
                false
            });
            std::thread::sleep(Duration::from_millis(20));
        }
        for (id, stack) in answered {
            for call in stack.lines() {
                out.push_str(&format!("{id} {call}\n"));
            }
        }
        let silent: Vec<&str> = waiting.iter().map(|(id, _)| id.as_str()).filter(|id| !id.is_empty()).collect();
        if !silent.is_empty() {
            out.push_str(&format!(
                "(no Java calls from {} in {:.1} s: it waits on a lock, the game or the network, or isn't the one stuck)\n",
                silent.join(", "),
                wait.as_secs_f32()
            ));
        }
        out.push_str(&engine_side);
        out
    }
}

/// `HORIZON_PLUGIN_PROFILE=1`: where the game thread's time goes in the plugins. Every 10 s, to
/// stderr: each plugin's time per event (its own listeners, the questions it asked included) and the
/// questions asked, by plugin and op, with their count, time and answer size. Off: one check an event.
mod profile {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    #[derive(Default)]
    struct Tally {
        listeners: HashMap<(String, String), (u64, Duration)>,
        queries: HashMap<(String, String), (u64, Duration, u64)>,
        since: Option<Instant>,
    }

    static TALLY: Mutex<Option<Tally>> = Mutex::new(None);
    thread_local! {
        static CURRENT: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    pub fn on() -> bool {
        static ON: OnceLock<bool> = OnceLock::new();
        *ON.get_or_init(|| std::env::var("HORIZON_PLUGIN_PROFILE").is_ok_and(|v| v == "1"))
    }

    pub fn enter(owner: &str) -> Instant {
        CURRENT.with(|c| c.borrow_mut().push(owner.to_owned()));
        Instant::now()
    }

    pub fn leave(event: &str, started: Instant) {
        let spent = started.elapsed();
        let owner = CURRENT.with(|c| c.borrow_mut().pop()).unwrap_or_default();
        let mut tally = TALLY.lock().unwrap_or_else(|e| e.into_inner());
        let tally = tally.get_or_insert_with(Tally::default);
        let row = tally.listeners.entry((owner, event.to_owned())).or_default();
        row.0 += 1;
        row.1 += spent;
        report(tally);
    }

    pub fn query(request: &str, started: Instant, answer: usize) {
        let spent = started.elapsed();
        // `"op":"name"` without parsing the whole request.
        let op = request.split("\"op\"").nth(1).and_then(|rest| rest.split('"').nth(1)).unwrap_or("?").to_owned();
        let owner = CURRENT.with(|c| c.borrow().last().cloned()).unwrap_or_else(|| "outside an event".to_owned());
        let mut tally = TALLY.lock().unwrap_or_else(|e| e.into_inner());
        let tally = tally.get_or_insert_with(Tally::default);
        let row = tally.queries.entry((owner, op)).or_default();
        row.0 += 1;
        row.1 += spent;
        row.2 += answer as u64;
    }

    fn report(tally: &mut Tally) {
        let since = *tally.since.get_or_insert_with(Instant::now);
        let window = since.elapsed();
        if window < Duration::from_secs(10) {
            return;
        }
        let per_s = |d: Duration| d.as_secs_f64() * 1000.0 / window.as_secs_f64();
        let mut owners: HashMap<&str, Duration> = HashMap::new();
        for ((owner, _), (_, spent)) in &tally.listeners {
            *owners.entry(owner.as_str()).or_default() += *spent;
        }
        let mut owners: Vec<_> = owners.into_iter().collect();
        owners.sort_by(|a, b| b.1.cmp(&a.1));
        let mut out = format!("[plugin-profile] {:.0} s window, ms of game thread per second:\n", window.as_secs_f64());
        for (owner, spent) in owners.iter().take(15) {
            out += &format!("  {owner:<24} {:8.1} ms/s\n", per_s(*spent));
        }
        let mut listeners: Vec<_> = tally.listeners.iter().collect();
        listeners.sort_by(|a, b| b.1.1.cmp(&a.1.1));
        out += "  top plugin x event (calls/s, ms/s):\n";
        for ((owner, event), (calls, spent)) in listeners.iter().take(15) {
            out += &format!("    {owner:<24} {event:<28} {:8.1} {:8.1}\n", *calls as f64 / window.as_secs_f64(), per_s(*spent));
        }
        let mut queries: Vec<_> = tally.queries.iter().collect();
        queries.sort_by(|a, b| b.1.1.cmp(&a.1.1));
        out += "  top plugin x question (calls/s, ms/s, KB/s answered):\n";
        for ((owner, op), (calls, spent, bytes)) in queries.iter().take(20) {
            out += &format!("    {owner:<24} {op:<28} {:8.1} {:8.1} {:8.1}\n", *calls as f64 / window.as_secs_f64(), per_s(*spent), *bytes as f64 / 1024.0 / window.as_secs_f64());
        }
        eprint!("{out}");
        *tally = Tally { since: Some(Instant::now()), ..Tally::default() };
    }
}

/// The loaded mods' ids and the libraries they came from, for a host to read.
pub fn loaded() -> Vec<(String, PathBuf)> {
    MODS.read().unwrap_or_else(|e| e.into_inner()).iter().map(|m| (m.id.clone(), m.path.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_come_out_in_order_and_once() {
        push("a".into());
        push("b".into());
        assert_eq!(take_actions(), ["a", "b"]);
        assert!(take_actions().is_empty());
    }

    #[test]
    fn a_folder_that_isnt_there_is_no_error() {
        let (manifests, errors) = open_folder(Path::new("no-such-folder-here"));
        assert!(manifests.is_empty() && errors.is_empty());
    }

    #[test]
    fn nobody_listening_answers_zero() {
        assert_eq!(fire("nothing", "{}"), 0);
    }

    extern "C" fn quiet(_: *const u8, _: usize, _: *const u8, _: usize) -> i32 {
        0
    }

    extern "C" fn subscribes_late(_: *const u8, _: usize, _: *const u8, _: usize) -> i32 {
        let name = "late_event#2#1";
        subscribe(name.as_ptr(), name.len(), quiet);
        0
    }

    #[test]
    fn a_late_subscription_is_its_listeners_owners() {
        let route = Route { slot: NORMAL, bukkit: true, name: "tick#2#1".into(), callback: subscribes_late, owner: "Shop".into(), free: false };
        calling(&route, || (route.callback)(std::ptr::null(), 0, std::ptr::null(), 0));
        let late = LATE.lock().unwrap();
        assert!(late.iter().any(|(owner, name, _)| owner == "Shop" && name == "late_event#2#1"));
        drop(late);
        // Retiring a plugin the runtime never loaded changes nothing; its late listener goes with a real one.
        assert_eq!(retire("NoSuchPlugin"), Ok(false));
    }

    #[test]
    fn the_watchdog_sees_the_server_threads_plugin_calls() {
        let list = std::sync::Arc::new(vec![Route { slot: NORMAL, bukkit: true, name: "tick#2#5".into(), callback: quiet, owner: "DonutShards".into(), free: false }]);
        let report = std::thread::Builder::new()
            .name("main".into())
            .spawn(move || {
                watch::mark_server_thread();
                let outer = watch::enter(|| watch::Doing::Enabling("Other".into()));
                let inner = watch::enter(|| watch::Doing::Listener(list.clone(), 0));
                let report = watch::report(std::time::Duration::from_millis(50));
                watch::leave(inner);
                watch::leave(outer);
                assert!(watch::frames().is_empty());
                report
            })
            .unwrap()
            .join()
            .unwrap();
        let lines: Vec<&str> = report.lines().collect();
        assert!(lines[0].starts_with("plugin DonutShards: listener tick#2#5, for "), "{report}");
        assert!(lines[1].starts_with("plugin Other: onEnable, for "), "{report}");
        // Another thread's calls aren't the server thread's.
        assert!(!watch::enter(|| watch::Doing::Enabling("x".into())));
    }
}
