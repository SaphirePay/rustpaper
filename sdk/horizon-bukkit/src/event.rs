//! `org.bukkit.event`: `register_event::<PlayerJoinEvent>(|event| ...)`, in the place of a class
//! with `@EventHandler` methods. `set_cancelled` refuses what the game would have done.
use crate::{Block, Inventory, ItemStack, Location, OPEN_INVENTORIES, Plugin, Player, World, get_online_players, get_player};
use horizon_mod::{Answer, Value, query};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;

/// bukkit: org/bukkit/event/Cancellable, org/bukkit/event/Cancellable.setCancelled, org/bukkit/event/Cancellable.isCancelled, org/bukkit/event/Event.getEventName, org/bukkit/event/Event.isAsynchronous, org/bukkit/event/player/PlayerEvent.getPlayer, org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/block/BlockEvent.getBlock, org/bukkit/event/world/WorldEvent.getWorld, org/bukkit/event/inventory/InventoryEvent.getInventory
///
/// What an event is on the bridge.
pub trait Event: Sized + 'static {
    /// The bridge's event this one is read from.
    const HOOK: &'static str;
    /// `None` skips it: another of the hook's cases.
    fn read(value: &Value) -> Option<Self>;
    fn answer(&self) -> Answer {
        Answer::Pass
    }
    /// More bridge events this one is read from.
    fn more_hooks() -> &'static [&'static str] {
        &[]
    }
    /// Every event one firing of the hook makes: more than one for a tick's moves.
    fn read_all(value: &Value) -> Vec<Self> {
        Self::read(value).into_iter().collect()
    }
    /// After the listeners, for what isn't an answer to the game.
    fn finish(&self) {}
    /// The event's class, as Java's `getEventName` gives it.
    fn get_event_name(&self) -> &'static str {
        std::any::type_name::<Self>().rsplit("::").next().unwrap_or_default()
    }
    /// Every event here runs on the server thread.
    fn is_asynchronous(&self) -> bool {
        false
    }
    /// Before anything is read: an answer for what isn't an event of this class.
    fn intercept(_value: &Value) -> Option<Answer> {
        None
    }
}

fn player_of(value: &Value) -> Option<Player> {
    Player::read(&value["player"]).or_else(|| Player::read(value)).or_else(|| {
        if let Some(uuid) = value["uuid"].as_str() {
            return Player::read(&query(json!({ "op": "player", "uuid": uuid })));
        }
        value["name"].as_str().and_then(get_player)
    })
}

pub(crate) fn block_of(player: &Player, pos: &Value) -> Option<Block> {
    let at = |i: usize| pos[i].as_i64().map(|v| v as i32);
    Some(World::get_block_at(&player.get_world(), at(0)?, at(1)?, at(2)?))
}

/// Everything this plugin listens to is off.
static HANDLERS_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
type HandlerFlags = HashMap<std::any::TypeId, std::sync::Arc<std::sync::atomic::AtomicBool>>;
static HANDLER_FLAGS: Mutex<Option<HandlerFlags>> = Mutex::new(None);

fn handler_flag<E: Event>() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    HANDLER_FLAGS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().entry(std::any::TypeId::of::<E>()).or_default().clone()
}

/// `org.bukkit.event.HandlerList`: a plugin's own listeners, as closures have no other name.
#[derive(Clone, Copy, Debug, Default)]
pub struct HandlerList;

pub use crate::fam_ev_world::BlockBreakEvent;

/// bukkit: org/bukkit/plugin/RegisteredListener
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredListener {
    hook: String,
    name: String,
    plugin: String,
    priority: &'static str,
    ignoring_cancelled: bool,
}

impl RegisteredListener {
    /// bukkit: org/bukkit/plugin/RegisteredListener.getPlugin
    pub fn get_plugin(&self) -> Option<crate::fam_core::LoadedPlugin> {
        crate::fam_core::PluginManager.get_plugin(&self.plugin)
    }
    /// The plugin's name, which holds whether it's in `get_plugin`'s list yet or not.
    pub fn get_plugin_name(&self) -> &str {
        &self.plugin
    }
    /// bukkit: org/bukkit/plugin/RegisteredListener.getPriority
    pub fn get_priority(&self) -> &'static str {
        self.priority
    }
    /// bukkit: org/bukkit/plugin/RegisteredListener.isIgnoringCancelled
    pub fn is_ignoring_cancelled(&self) -> bool {
        self.ignoring_cancelled
    }
    /// bukkit: org/bukkit/plugin/RegisteredListener.callEvent
    ///
    /// This listener alone, in whichever plugin it lives: `event` is back as it left it.
    pub fn call_event<E: crate::fam_core::RaisedEvent>(&self, event: &mut E) {
        let mut payload = event.raised_payload();
        payload["raised"] = json!(true);
        let answer = query(json!({ "op": "event_call", "hook": self.hook, "name": self.name, "payload": payload }));
        let carried = if answer["payload"].is_null() { &payload } else { &answer["payload"] };
        if let Some(fresh) = E::read(carried) {
            *event = fresh;
        }
        let code = answer["answer"].as_i64().unwrap_or(0);
        event.set_raised_cancelled(code == 1 || code == 2);
    }
}

impl HandlerList {
    /// bukkit: org/bukkit/event/HandlerList.getRegisteredListeners
    ///
    /// Everyone listening to an event, from every plugin, in the order they run.
    pub fn get_registered_listeners<E: Event>() -> Vec<RegisteredListener> {
        let names = query(json!({ "op": "event_listeners", "hook": E::HOOK }));
        names.as_array().into_iter().flatten().filter_map(Value::as_str).filter_map(|name| {
            let parts: Vec<&str> = name.split('#').collect();
            let priority = crate::enums::event_priority::values().get(parts.get(1)?.parse::<usize>().ok()?)?;
            Some(RegisteredListener { hook: E::HOOK.to_owned(), name: name.to_owned(), plugin: (*parts.get(4)?).to_owned(), priority, ignoring_cancelled: *parts.get(3)? == "i" })
        }).collect()
    }
    /// bukkit~: org/bukkit/event/HandlerList.getHandlerLists | each event anyone listens to, by its hook: a list here is `get_registered_listeners::<E>()`, by the event's type
    pub fn get_handler_lists() -> Vec<String> {
        query(json!({ "op": "event_hooks" })).as_array().into_iter().flatten().filter_map(|hook| hook.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/event/HandlerList.unregisterAll
    pub fn unregister_all() {
        HANDLERS_OFF.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    /// bukkit: org/bukkit/event/HandlerList.unregister
    pub fn unregister<E: Event>() {
        handler_flag::<E>().store(true, std::sync::atomic::Ordering::Relaxed);
    }
    /// bukkit: org/bukkit/event/HandlerList.hasListeners
    pub fn has_listeners<E: Event>() -> bool {
        !HANDLERS_OFF.load(std::sync::atomic::Ordering::Relaxed) && HANDLER_FLAGS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|all| all.get(&std::any::TypeId::of::<E>()).is_some_and(|off| !off.load(std::sync::atomic::Ordering::Relaxed)))
    }
}

thread_local! {
    /// The listener about to run comes after a cancel.
    static PRIOR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The event just read took that cancel on, so a `Pass` from it means `setCancelled(false)`.
    static SEEDED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// What `cancelled` starts as, in every event's `read`: a listener sees what those before it did.
pub(crate) fn prior_cancelled() -> bool {
    SEEDED.set(true);
    PRIOR.get()
}

/// Tells the listeners after this one that a field of the event now reads as `value`.
pub(crate) fn carry(key: &str, value: Value) {
    query(json!({ "op": "event_patch", "key": key, "value": value.to_string() }));
}

static LISTENER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// A plugin enabled again: what it was switched off for no longer holds.
pub(crate) fn handlers_back() {
    HANDLERS_OFF.store(false, std::sync::atomic::Ordering::Relaxed);
    *HANDLER_FLAGS.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

impl Plugin {
    /// bukkit: org/bukkit/plugin/PluginManager.registerEvents, org/bukkit/event/Listener, org/bukkit/event/EventHandler
    pub fn register_event<E: Event>(&self, listener: impl Fn(&mut E) + Send + Sync + 'static) {
        self.register_event_with(crate::enums::event_priority::NORMAL, false, listener);
    }
    /// bukkit: org/bukkit/plugin/PluginManager.registerEvent
    ///
    /// `@EventHandler(priority = ..., ignoreCancelled = ...)`: `priority` is a name from `event_priority`.
    pub fn register_event_with<E: Event>(&self, priority: &str, ignore_cancelled: bool, listener: impl Fn(&mut E) + Send + Sync + 'static) {
        let slot = crate::enums::event_priority::values().iter().position(|name| *name == priority).unwrap_or(2);
        let listener = std::sync::Arc::new(listener);
        let off = handler_flag::<E>();
        for hook in std::iter::once(E::HOOK).chain(E::more_hooks().iter().copied()) {
            let listener = listener.clone();
            let off = off.clone();
            // A name of its own: the game runs each listener apart, so each can see the last one's answer.
            let number = LISTENER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let ignoring = if ignore_cancelled { "i" } else { "-" };
            // Listed so that the game runs that step while someone wants this event, not as this event.
            let only_keeps_it_running = matches!(hook, "pl_moves" | "pl_move_turn" | "pl_packet") && hook != E::HOOK;
            self.host.on(&format!("{hook}#{slot}#{number}#{ignoring}#{}", self.get_name()), move |value| {
                if only_keeps_it_running || HANDLERS_OFF.load(std::sync::atomic::Ordering::Relaxed) || off.load(std::sync::atomic::Ordering::Relaxed) {
                    return Answer::Pass;
                }
                if let Some(answer) = E::intercept(value) {
                    return answer;
                }
                let prior = value["__answer"].as_i64().unwrap_or(0);
                let cancelled = prior == 1 || prior == 2;
                if cancelled && ignore_cancelled {
                    return Answer::Pass;
                }
                // An amount a listener set is what the next one reads.
                let amended;
                let value = if prior >= 3 {
                    let mut copy = value.clone();
                    copy["amount"] = json!((prior - 3) as f64 / 1000.0);
                    amended = copy;
                    &amended
                } else {
                    value
                };
                PRIOR.set(cancelled);
                SEEDED.set(false);
                let mut answer = Answer::Pass;
                for mut event in E::read_all(value) {
                    listener(&mut event);
                    let said = event.answer();
                    // Whoever cancelled it first has done what a cancel does.
                    if !cancelled || said == Answer::Pass {
                        event.finish();
                    }
                    answer = if cancelled && SEEDED.get() && said == Answer::Pass { Answer::Allow } else { said };
                }
                PRIOR.set(false);
                answer
            });
        }
    }
}

/// bukkit: org/bukkit/event/player/PlayerCommandPreprocessEvent
pub struct PlayerCommandPreprocessEvent {
    pub player: Player,
    pub message: String,
    cancelled: bool,
}

impl PlayerCommandPreprocessEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/player/PlayerCommandPreprocessEvent.getMessage
    pub fn get_message(&self) -> &str {
        &self.message
    }
    /// bukkit: org/bukkit/event/player/PlayerCommandPreprocessEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/player/PlayerCommandPreprocessEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PlayerCommandPreprocessEvent {
    const HOOK: &'static str = "command_preprocess";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { player: player_of(value)?, message: value["message"].as_str()?.to_owned(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

// These have one version now, in `fam_ev_player`: fired before the game acts, and for every menu.
pub use crate::fam_ev_player::{AsyncPlayerChatEvent, PlayerInteractEvent, PlayerRespawnEvent, InventoryClickEvent, InventoryCloseEvent, InventoryDragEvent, PlayerChangedWorldEvent, PlayerMoveEvent, PlayerPortalEvent, PlayerTeleportEvent};

macro_rules! player_event {
    ($(#[$doc:meta])* $name:ident, $hook:literal) => {
        $(#[$doc])*
        pub struct $name {
            pub player: Player,
        }
        impl $name {
            pub fn get_player(&self) -> &Player {
                &self.player
            }
        }
        impl Event for $name {
            const HOOK: &'static str = $hook;
            fn read(value: &Value) -> Option<Self> {
                Some(Self { player: player_of(value)? })
            }
        }
    };
}

/// Asks the server for one of its lines: it keeps them, as each plugin has its own copy of this code.
pub(crate) fn line_set(kind: u8, text: Option<&str>) {
    query(json!({ "op": "line_set", "kind": kind, "line": text.map(|text| crate::adventure::legacy(text).0) }));
}

/// bukkit: org/bukkit/event/player/PlayerJoinEvent, org/bukkit/event/player/PlayerJoinEvent.getJoinMessage
pub struct PlayerJoinEvent {
    pub player: Player,
    message: Option<Option<String>>,
}

impl PlayerJoinEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/player/PlayerJoinEvent.getJoinMessage
    pub fn get_join_message(&self) -> String {
        match &self.message {
            Some(text) => text.clone().unwrap_or_default(),
            None => format!("\u{a7}e{} joined the game", self.player.get_name()),
        }
    }
    /// bukkit: org/bukkit/event/player/PlayerJoinEvent.setJoinMessage
    ///
    /// `None` says nothing.
    pub fn set_join_message(&mut self, text: Option<&str>) {
        line_set(0, text);
        carry("join_message", json!(text));
        self.message = Some(text.map(str::to_owned));
    }
    /// bukkit: org/bukkit/event/player/PlayerJoinEvent.joinMessage
    ///
    /// Paper's form: the message as a component, read back as one.
    pub fn join_message(&self) -> crate::Component {
        crate::adventure::legacy(&self.get_join_message())
    }
    /// bukkit: org/bukkit/event/player/PlayerJoinEvent.joinMessage
    ///
    /// Paper's setter: `null` says nothing.
    pub fn set_join_message_component(&mut self, message: Option<&crate::Component>) {
        let text = message.map(crate::Component::to_legacy);
        self.set_join_message(text.as_deref());
    }
}

impl Event for PlayerJoinEvent {
    const HOOK: &'static str = "join";
    fn read(value: &Value) -> Option<Self> {
        let message = value.get("join_message").map(|text| text.as_str().map(str::to_owned));
        Some(Self { player: player_of(value)?, message })
    }
}

/// bukkit: org/bukkit/event/player/PlayerQuitEvent, org/bukkit/event/player/PlayerQuitEvent.getQuitMessage
pub struct PlayerQuitEvent {
    pub player: Player,
    message: Option<Option<String>>,
}

impl PlayerQuitEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/player/PlayerQuitEvent.getQuitMessage
    pub fn get_quit_message(&self) -> String {
        match &self.message {
            Some(text) => text.clone().unwrap_or_default(),
            None => format!("\u{a7}e{} left the game", self.player.get_name()),
        }
    }
    /// bukkit: org/bukkit/event/player/PlayerQuitEvent.setQuitMessage
    ///
    /// `None` says nothing.
    pub fn set_quit_message(&mut self, text: Option<&str>) {
        line_set(1, text);
        carry("quit_message", json!(text));
        self.message = Some(text.map(str::to_owned));
    }
    /// bukkit: org/bukkit/event/player/PlayerQuitEvent.quitMessage
    pub fn quit_message(&self) -> crate::Component {
        crate::adventure::legacy(&self.get_quit_message())
    }
    /// bukkit: org/bukkit/event/player/PlayerQuitEvent.quitMessage
    pub fn set_quit_message_component(&mut self, message: Option<&crate::Component>) {
        let text = message.map(crate::Component::to_legacy);
        self.set_quit_message(text.as_deref());
    }
}

impl Event for PlayerQuitEvent {
    const HOOK: &'static str = "disconnect";
    fn read(value: &Value) -> Option<Self> {
        let message = value.get("quit_message").map(|text| text.as_str().map(str::to_owned));
        Some(Self { player: player_of(value)?, message })
    }
}






/// bukkit~: org/bukkit/event/player/PlayerDeathEvent, org/bukkit/event/entity/PlayerDeathEvent | after the death: drops and the message can't be changed
pub struct PlayerDeathEvent {
    pub player: Player,
    pub cause: Option<String>,
}



impl PlayerDeathEvent {
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getDeathMessage
    ///
    /// The cause, as the game words it.
    pub fn get_death_message(&self) -> Option<&str> {
        self.cause.as_deref()
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setDeathMessage
    ///
    /// What the server says of this death: `None` says nothing.
    pub fn set_death_message(&mut self, text: Option<&str>) {
        line_set(2, text);
        carry("cause", json!(text));
        self.cause = text.map(str::to_owned);
    }
}

impl Event for PlayerDeathEvent {
    const HOOK: &'static str = "player_death";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { player: player_of(value)?, cause: value["cause"].as_str().map(str::to_owned) })
    }
}




/// bukkit: org/bukkit/event/player/PlayerLoginEvent
///
/// Before the game takes them: a refusal disconnects them without their ever joining. Most of a
/// `Player` can't be read yet, as on Paper.
pub struct PlayerLoginEvent {
    pub player: Player,
    pub(crate) value: Value,
    refused: Option<String>,
}

impl PlayerLoginEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    pub fn get_name(&self) -> &str {
        self.player.get_name()
    }
    pub fn get_unique_id(&self) -> String {
        self.player.get_unique_id()
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.getResult
    ///
    /// `ALLOWED`, or `KICK_OTHER` once refused.
    pub fn get_result(&self) -> &'static str {
        if self.refused.is_some() { "KICK_OTHER" } else { "ALLOWED" }
    }
    pub fn get_login_result(&self) -> &'static str {
        self.get_result()
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.setResult
    pub fn set_result(&mut self, result: &str) {
        if result == "ALLOWED" {
            self.refused = None;
        } else {
            self.refused.get_or_insert_default();
        }
    }
    pub fn set_login_result(&mut self, result: &str) {
        self.set_result(result);
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.disallow
    pub fn disallow(&mut self, _result: &str, message: &str) {
        self.refused = Some(message.to_owned());
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.allow
    pub fn allow(&mut self) {
        self.refused = None;
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.setKickMessage
    pub fn set_kick_message(&mut self, message: &str) {
        self.refused = Some(message.to_owned());
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent.getKickMessage
    pub fn get_kick_message(&self) -> &str {
        self.refused.as_deref().unwrap_or_default()
    }
}

impl Event for PlayerLoginEvent {
    const HOOK: &'static str = "pl_login";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { player: player_of(value)?, value: value.clone(), refused: None })
    }
    fn answer(&self) -> Answer {
        if self.refused.is_some() { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let Some(message) = &self.refused {
            query(json!({ "op": "ev_player_login_line", "line": message }));
        }
    }
}
