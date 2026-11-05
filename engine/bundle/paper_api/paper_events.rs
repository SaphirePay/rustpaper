// The Paper engine's events: each of the game's hook points told straight to the plugin layer, as Paper calls its
// event handlers. Nothing here costs a thing on a server with no plugin: the bridge is installed only once one loads
// (`paper_bridge::installed`), and the core family once the server starts (`paper_fam_core::started`).
//
// The modules keep the names of what they tell (`server_tick_events::end_server_tick`) so the families' rows read as
// before; each is a plain function now, called by the hook that has the moment.
use crate::game::{
    Game,
    native_hooks::NativeLootDrops,
    native_lifecycle::{ServerTickPhase, WorldPhase},
    native_server::{NativeCommand, NativeServerPlayer},
};
use crate::game::{paper_bridge as bridge, paper_fam_core as core, paper_fam_ev_player as ev_player, paper_fam_ev_world as ev_world};
use horizon_client::{paper_hub as hub, paper_net as net, items::ItemStack};
use horizon_world::registry::BlockStateId;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, Once, RwLock},
};

/// The engine's version, for `Bukkit.getVersion` and friends.
pub(in crate::game) const VERSION: &str = env!("CARGO_PKG_VERSION");

const LEVELS: [&str; 3] = ["minecraft:overworld", "minecraft:the_nether", "minecraft:the_end"];

/// What a plugin's answer to an action is: leave it to the game, take it, or refuse it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game) enum Decision {
    Pass,
    Success,
    Fail,
}

impl Decision {
    /// A plugin took or refused it: the game's own handling stops.
    pub(in crate::game) fn decided(self) -> bool {
        self != Self::Pass
    }

    /// Whether it goes ahead, `default` when nobody said.
    pub(in crate::game) fn allow_action(self, default: bool) -> bool {
        match self {
            Self::Pass => default,
            Self::Success => true,
            Self::Fail => false,
        }
    }
}

/// Whether anything hears the bridge's events: a plugin is loaded.
fn heard() -> bool {
    bridge::installed()
}

pub(in crate::game) mod server_lifecycle_events {
    use super::*;

    pub(in crate::game) fn server_starting(game: &mut Game<'_>) {
        if heard() {
            ev_world::on_server_starting(game);
        }
    }

    pub(in crate::game) fn server_started(game: &mut Game<'_>) {
        if heard() {
            bridge::ev_server_started(game);
        }
    }

    pub(in crate::game) fn server_stopping(game: &mut Game<'_>) {
        if heard() {
            bridge::ev_server_stopping(game);
        }
        if core::started() {
            core::on_stopping();
        }
    }

    pub(in crate::game) fn start_data_pack_reload(game: &mut Game<'_>) {
        if heard() {
            ev_world::on_reload_start(game);
        }
    }

    pub(in crate::game) fn end_data_pack_reload(game: &mut Game<'_>, success: bool) {
        if heard() {
            bridge::ev_reload_end(game, success);
            ev_world::on_reload_end(game);
        }
    }

    pub(in crate::game) fn before_save(game: &mut Game<'_>, wait: bool) {
        if heard() {
            bridge::ev_save_before(game, wait);
        }
    }
}

pub(in crate::game) mod server_tick_events {
    use super::*;

    pub(in crate::game) fn start_server_tick(game: &mut Game<'_>) {
        if heard() {
            bridge::ev_tick_start(game);
        }
        if core::started() {
            core::tick_began();
        }
    }

    pub(in crate::game) fn end_server_tick(game: &mut Game<'_>) {
        if heard() {
            bridge::ev_tick(game);
        }
        if core::started() {
            core::on_tick_end(game);
        }
    }
}

pub(in crate::game) mod entity_sleep_events {
    use super::*;

    /// What stops a player lying down, and what they're told. `Other` says nothing.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(in crate::game) enum BedProblem {
        Other,
    }

    /// `PlayerBedEnterEvent` for what the game's own checks would say: the first problem stops the player.
    pub(in crate::game) fn allow_sleeping(game: &mut Game<'_>, player: &NativeServerPlayer, pos: [i32; 3]) -> Option<BedProblem> {
        if ev_player::bed_heard() { ev_player::bed_allow_sleeping(game, player, pos) } else { None }
    }

    /// `PlayerBedEnterEvent` for monsters near: `safe` is the game's answer.
    pub(in crate::game) fn allow_nearby_monsters(game: &mut Game<'_>, player: &NativeServerPlayer, pos: [i32; 3], safe: bool) -> Decision {
        if ev_player::bed_heard() { ev_player::bed_nearby_monsters(game, player, pos, safe) } else { Decision::Pass }
    }
}

pub(in crate::game) mod block_events {
    use super::*;

    /// What a plugin answers a block use: an `InteractionResult`, or `Default`, which leaves the block to it. `Fail`
    /// and `Pass` skip the block and go on to the item's own use, as in `ServerPlayerGameMode.useItemOn`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(in crate::game) enum Use {
        Default,
        Success,
        SuccessServer,
        Consume,
        Fail,
        Pass,
        TryEmptyHand,
    }

    /// The `BlockHitResult` and `InteractionHand`. `isInside` is `point` being the player's eye.
    #[derive(Clone, Copy, Debug)]
    pub(in crate::game) struct Click {
        pub(in crate::game) pos: [i32; 3],
        pub(in crate::game) face: [i32; 3],
        pub(in crate::game) point: [f64; 3],
        pub(in crate::game) off_hand: bool,
    }

    pub(in crate::game) fn heard() -> bool {
        super::heard()
    }

    /// The block's step of a click with an item (`minecraft:air` for an empty hand).
    pub(in crate::game) fn use_item_on(game: &mut Game<'_>, player: &NativeServerPlayer, stack: &ItemStack, click: &Click) -> Use {
        if heard() { bridge::ev_block_use_item(game, player, stack, click) } else { Use::Default }
    }

    /// The block's step of a click with nothing that uses it.
    pub(in crate::game) fn use_without_item(game: &mut Game<'_>, player: &NativeServerPlayer, click: &Click) -> Use {
        if heard() { bridge::ev_block_use(game, player, click) } else { Use::Default }
    }
}

pub(in crate::game) mod player_block_break_events {
    /// The block as it was, for a break that went ahead.
    pub(in crate::game) struct Was {
        pub(in crate::game) entity: Option<horizon_client::containers::BlockEntity>,
        /// The saved form, for what Horizon keeps only as that.
        pub(in crate::game) saved: Option<horizon_world::nbt::Tag>,
    }
}

pub(in crate::game) mod server_player_events {
    use super::*;

    /// The player, their name, and `alive`: false after a death, true coming back from the End.
    pub(in crate::game) fn after_respawn(game: &mut Game<'_>, player: u128, name: &str, alive: bool) {
        let _ = alive;
        if heard() {
            bridge::ev_respawn(game, player, name);
        }
    }
}

pub(in crate::game) mod server_entity_level_change_events {
    use super::*;

    pub(in crate::game) fn after_player_change_level(game: &mut Game<'_>, player: u128, name: &str, from: &str, to: &str) {
        if heard() {
            bridge::ev_change_level(game, player, name, from, to);
        }
    }

    /// The entity's old id, its new one, its type, and the two levels, as it joins the new level.
    pub(in crate::game) fn after_entity_change_level(game: &mut Game<'_>, entity: u32, new: u32, kind: &str, from: &str, to: &str) {
        if heard() {
            bridge::ev_entity_change_level(game, entity, new, kind, from, to);
        }
    }
}

pub(in crate::game) mod server_living_entity_events {
    use super::*;

    /// What's being hurt and by what. `mob` is the mob as it stands, in `game.mobs` as ever; `player` a player.
    pub(in crate::game) struct Damage<'a> {
        pub(in crate::game) entity: i32,
        pub(in crate::game) kind: &'a str,
        pub(in crate::game) damage_type: &'a str,
        pub(in crate::game) causing: i32,
        pub(in crate::game) direct: i32,
        pub(in crate::game) amount: f32,
        pub(in crate::game) player: Option<u128>,
        pub(in crate::game) mob: Option<&'a horizon_client::mobs::Mob>,
    }

    /// `AFTER_DEATH`: what died, and of what. `damage_type` is `None` for a death no hit led to.
    pub(in crate::game) struct Death<'a> {
        pub(in crate::game) entity: i32,
        pub(in crate::game) kind: &'a str,
        pub(in crate::game) player: Option<u128>,
        pub(in crate::game) mob: Option<&'a horizon_client::mobs::Mob>,
        pub(in crate::game) damage_type: Option<&'a str>,
        pub(in crate::game) causing: i32,
        pub(in crate::game) direct: i32,
        pub(in crate::game) killer: Option<u128>,
        /// A player's death message key.
        pub(in crate::game) cause: Option<&'a str>,
    }

    /// The mob that was, its type, the new mob, its type, and whether the equipment moved over. `old` is the mob as
    /// it was; the new one is in `game.mobs`, finished, and not yet in the level.
    pub(in crate::game) struct Conversion<'a> {
        pub(in crate::game) old: &'a horizon_client::mobs::Mob,
        pub(in crate::game) new: u32,
        pub(in crate::game) new_kind: &'a str,
        pub(in crate::game) keep_equipment: bool,
        pub(in crate::game) preserve_can_pick_up_loot: bool,
        /// `ConversionType.SPLIT_ON_DEATH`, not `SINGLE`.
        pub(in crate::game) split: bool,
    }

    /// `EntityDamageEvent` in the middle of the hit: false refuses it.
    pub(in crate::game) fn allow_damage_mid(damage: &Damage<'_>) -> bool {
        // The entity family notes the hit first, for what its events ask of it later in the tick.
        if crate::game::paper_fam_ev_entity::damage_noted() {
            crate::game::paper_fam_ev_entity::damage_heard(damage);
        }
        !heard() || bridge::ev_allow_damage(damage)
    }

    pub(in crate::game) fn mob_conversion(game: &mut Game<'_>, conversion: &Conversion<'_>) {
        if heard() {
            bridge::ev_mob_conversion(game, conversion);
        }
    }

    pub(in crate::game) fn after_player_death(game: &mut Game<'_>, death: &Death<'_>) {
        if heard() {
            bridge::ev_player_death(game, death);
        }
    }

    pub(in crate::game) fn after_mob_death(game: &mut Game<'_>, death: &Death<'_>) {
        if heard() {
            bridge::ev_mob_death(game, death);
        }
    }
}

pub(in crate::game) mod server_entity_events {
    use super::*;

    /// `onChange`: who, which slot, and what was and is there. A mob by its entity id.
    pub(in crate::game) struct EquipmentChange<'a> {
        pub(in crate::game) player: Option<&'a NativeServerPlayer>,
        pub(in crate::game) mob: Option<u32>,
        pub(in crate::game) slot: &'a str,
        pub(in crate::game) previous: Option<&'a ItemStack>,
        pub(in crate::game) current: Option<&'a ItemStack>,
    }
}

// ---- Commands: a plugin's word in the server's tree, with its handler by first word ----

pub(in crate::game) type CommandHandler = dyn Fn(&mut Game<'_>, &NativeServerPlayer, &str) + Send + Sync;

static COMMANDS: RwLock<Vec<(NativeCommand, String, Arc<CommandHandler>)>> = RwLock::new(Vec::new());
static STARTED: Once = Once::new();

fn word(line: &str) -> &str {
    line.split_whitespace().next().unwrap_or_default()
}

/// A command every player whose level allows it may run; `level` 0 for all.
pub(in crate::game) fn register_command(usage: &str, level: u8, handler: Arc<CommandHandler>) {
    let command = if level == 0 { NativeCommand::new(usage) } else { NativeCommand::op(usage, level) };
    COMMANDS.write().unwrap_or_else(|e| e.into_inner()).push((command, word(usage).to_owned(), handler));
}

/// A plugin's command: it takes the word from whatever had it before, the engine's own `/plugins`, `/pl`
/// or `/version` (a core plugin prints them its way) or the plugin's older self (a hot swap).
pub(in crate::game) fn register_plugin_command(usage: &str, level: u8, handler: Arc<CommandHandler>) {
    let name = word(usage).to_owned();
    COMMANDS.write().unwrap_or_else(|e| e.into_inner()).retain(|(_, taken, _)| *taken != name);
    PLUGIN_WORDS.write().unwrap_or_else(|e| e.into_inner()).insert(name);
    register_command(usage, level, handler);
}

static PLUGIN_WORDS: RwLock<std::collections::BTreeSet<String>> = RwLock::new(std::collections::BTreeSet::new());

/// Whether a plugin took `word` (the console then leaves it to the plugin, not the engine's own).
pub(in crate::game) fn plugin_word(word: &str) -> bool {
    PLUGIN_WORDS.read().unwrap_or_else(|e| e.into_inner()).contains(word)
}

/// The words plugins have in the tree.
pub(in crate::game) fn mod_words() -> Vec<String> {
    let mut words: Vec<String> = COMMANDS.read().unwrap_or_else(|e| e.into_inner()).iter().map(|(_, name, _)| name.clone()).collect();
    words.sort();
    words.dedup();
    words
}

pub(in crate::game) fn on_register_commands(game: &mut Game<'_>, commands: &mut Vec<NativeCommand>) {
    let _ = game;
    startup();
    commands.extend(COMMANDS.read().unwrap_or_else(|e| e.into_inner()).iter().map(|(command, ..)| command.clone()));
}

pub(in crate::game) fn on_perform_command(game: &mut Game<'_>, player: &NativeServerPlayer, line: &str, ran: &mut bool) {
    if *ran {
        return;
    }
    let first = word(line);
    let handler = COMMANDS.read().unwrap_or_else(|e| e.into_inner()).iter().find(|(_, name, _)| name == first).map(|(.., handler)| handler.clone());
    if let Some(handler) = handler {
        handler(game, player, line);
        *ran = true;
    }
}

// ---- Plugin channels: Bukkit's `Messenger` ----
//
// A plugin's incoming channel is announced to every client in `minecraft:register`, as Paper does, and what a client
// sends on it goes to the plugin. Which channels each client registered is what `Player.getListeningPluginChannels`
// reads.

pub(in crate::game) type Receiver = dyn Fn(&mut Game<'_>, u128, &[u8]) + Send + Sync;

static RECEIVERS: RwLock<Vec<(String, Arc<Receiver>)>> = RwLock::new(Vec::new());
/// Which channels each player's client said it listens on.
static SENDABLE: Mutex<Option<HashMap<u128, HashSet<String>>>> = Mutex::new(None);
/// A channel that came or went once the first list went out: told to whoever is in, at the next tick.
static CHANNEL_NEWS: Mutex<Vec<(bool, String)>> = Mutex::new(Vec::new());

/// `Messenger.registerIncomingPluginChannel`.
pub(in crate::game) fn register_global_receiver(channel: &str, receiver: Arc<Receiver>) {
    CHANNEL_NEWS.lock().unwrap_or_else(|e| e.into_inner()).push((true, channel.to_owned()));
    RECEIVERS.write().unwrap_or_else(|e| e.into_inner()).push((channel.to_owned(), receiver));
}

/// `Messenger.unregisterIncomingPluginChannel`.
pub(in crate::game) fn unregister_global_receiver(channel: &str) -> bool {
    let mut receivers = RECEIVERS.write().unwrap_or_else(|e| e.into_inner());
    let before = receivers.len();
    receivers.retain(|(name, _)| name != channel);
    let removed = receivers.len() != before;
    drop(receivers);
    if removed {
        CHANNEL_NEWS.lock().unwrap_or_else(|e| e.into_inner()).push((false, channel.to_owned()));
    }
    removed
}

/// Whether this player's client listens on `channel`.
pub(in crate::game) fn can_send(player: u128, channel: &str) -> bool {
    SENDABLE.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(&player)).is_some_and(|set| set.contains(channel))
}

/// `Player.sendPluginMessage`.
pub(in crate::game) fn send(game: &mut Game<'_>, player: u128, channel: &str, data: &[u8]) -> Result<(), String> {
    game.native_net_send_to_player(player, channel, data)
}

fn news_payload(added: bool, channel: String) -> horizon_protocol::packets::configuration::CustomPayload {
    horizon_protocol::packets::configuration::CustomPayload { channel: if added { net::REGISTER } else { net::UNREGISTER }.to_owned(), data: net::channel_list([channel]) }
}

fn tell_channel_news(game: &mut Game<'_>) {
    use horizon_protocol::packets::play::Clientbound as C;
    let news = std::mem::take(&mut *CHANNEL_NEWS.lock().unwrap_or_else(|e| e.into_inner()));
    if let Some(host) = game.lan.as_ref() {
        for (added, channel) in news {
            let packet = C::CustomPayload(news_payload(added, channel));
            for guest in host.hub.guests.iter().filter(|guest| guest.placed) {
                guest.send(&packet);
            }
        }
    }
}

/// The plugins' channels to a client that just joined, as `minecraft:register`. Nothing for an empty list.
fn tell_channels(game: &mut Game<'_>, player: u128) {
    use horizon_protocol::packets::{configuration::CustomPayload, play::Clientbound as C};
    let receivers = RECEIVERS.read().unwrap_or_else(|e| e.into_inner());
    if receivers.is_empty() {
        return;
    }
    let data = net::channel_list(receivers.iter().map(|(channel, _)| channel.clone()));
    drop(receivers);
    if let Some(host) = game.lan.as_ref()
        && let Some(guest) = host.hub.guests.iter().find(|guest| guest.uuid == player)
    {
        guest.send(&C::CustomPayload(CustomPayload { channel: net::REGISTER.to_owned(), data }));
    }
}

/// A client's `minecraft:register` or `unregister`: the set it updates, and whether it was one of those.
fn apply_channels(set: &mut HashSet<String>, channel: &str, data: &[u8]) -> bool {
    let channels = net::read_channels(data);
    match channel {
        net::REGISTER => set.extend(channels),
        net::UNREGISTER => channels.iter().for_each(|name| {
            set.remove(name);
        }),
        _ => return false,
    }
    true
}

pub(in crate::game) fn on_net_from_client(game: &mut Game<'_>, player: u128, channel: &str, data: &[u8]) {
    let registration = apply_channels(SENDABLE.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().entry(player).or_default(), channel, data);
    if registration {
        return;
    }
    let receivers: Vec<Arc<Receiver>> = RECEIVERS.read().unwrap_or_else(|e| e.into_inner()).iter().filter(|(name, _)| name == channel).map(|(_, receiver)| receiver.clone()).collect();
    for receiver in receivers {
        receiver(game, player, data);
    }
}

// ---- The server's start, its worlds and its ticks ----

/// What the launch needs before plugins load: nothing yet beyond the loader.
pub(in crate::game) fn at_launch() {}

/// What runs once the server starts, plugins or not: the core family's own commands and turns.
pub(in crate::game) fn startup() {
    STARTED.call_once(core::start);
}

pub(in crate::game) fn on_server_starting(game: &mut Game<'_>, dedicated: bool) {
    startup();
    log::info!("[PaperEngine] {VERSION}, dedicated: {dedicated}");
    server_lifecycle_events::server_starting(game);
}

pub(in crate::game) fn on_world_lifecycle(game: &mut Game<'_>, phase: WorldPhase) {
    match phase {
        WorldPhase::WorldOpenReady => {
            for level in LEVELS {
                level_loaded(game, level);
            }
            for level in crate::game::paper_worlds::open_levels() {
                level_loaded(game, &level);
            }
            // Paper's order: the worlds load, the plugins enable, then ServerLoadEvent (`server_started`).
            bridge::enable_waiting(game);
            server_lifecycle_events::server_started(game);
            // What hosts and tools wait for: the world is open and the plugins are on.
            let plugins = crate::game::paper_plugins::get().map_or(0, |p| p.loaded.len());
            log::info!("[PaperEngine] server started: {plugins} plugins enabled");
        }
        WorldPhase::WorldCloseFinal | WorldPhase::WorldCloseFailed => {
            for level in crate::game::paper_worlds::open_levels() {
                level_unloaded(game, &level);
            }
            for level in LEVELS {
                level_unloaded(game, level);
            }
        }
        WorldPhase::WorldOpenFailed => {}
    }
}

/// The levels `WorldLoadEvent` has told of.
static LOADED_LEVELS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// `WorldLoadEvent`, once for a level, however it came to exist.
pub(in crate::game) fn level_loaded(game: &mut Game<'_>, level: &str) {
    {
        let mut loaded = LOADED_LEVELS.lock().unwrap_or_else(|e| e.into_inner());
        if loaded.iter().any(|known| known == level) {
            return;
        }
        loaded.push(level.to_owned());
    }
    if heard() {
        bridge::ev_level_load(game, level);
    }
}

/// A level closes: the next time it opens, it's told again.
pub(in crate::game) fn level_unloaded(_game: &mut Game<'_>, level: &str) {
    LOADED_LEVELS.lock().unwrap_or_else(|e| e.into_inner()).retain(|known| known != level);
}

pub(in crate::game) fn on_server_stopping(game: &mut Game<'_>) {
    server_lifecycle_events::server_stopping(game);
}

pub(in crate::game) fn on_server_tick(game: &mut Game<'_>, phase: ServerTickPhase) {
    match phase {
        ServerTickPhase::ServerTickStart { .. } => {
            crate::game::paper_entity_events::hear_effects(game);
            horizon_client::block_entities::hear_changes(heard());
            server_tick_events::start_server_tick(game);
        }
        ServerTickPhase::ServerTickEnd { .. } => {
            if heard() {
                bridge::flush_lines(game);
            }
            hub::take();
            tell_channel_news(game);
            fire_tracking(game);
            crate::game::paper_entity_events::deaths_now(game);
            server_tick_events::end_server_tick(game);
        }
    }
}

/// What each player tracked last tick, to hear what they stopped tracking.
static TRACKED: Mutex<Option<HashMap<u128, HashSet<i32>>>> = Mutex::new(None);

/// The game keeps each client's tracked set: what's gone since last tick is what stopped being tracked.
fn fire_tracking(game: &mut Game<'_>) {
    let (bridge_hears, family_hears) = (heard(), ev_player::tracking_heard());
    if !bridge_hears && !family_hears {
        return;
    }
    let Some(host) = &game.lan else {
        return;
    };
    let now: HashMap<u128, HashSet<i32>> = host.hub.guests.iter().map(|g| (g.uuid, g.tracker.ids().into_iter().collect())).collect();
    let before = TRACKED.lock().unwrap_or_else(|e| e.into_inner()).replace(now.clone()).unwrap_or_default();
    for (player, seen) in &before {
        let new = now.get(player);
        for entity in seen.iter().filter(|id| new.is_none_or(|n| !n.contains(id))) {
            if bridge_hears {
                bridge::ev_tracking_stop(game, *entity, *player);
            }
            if family_hears {
                ev_player::on_tracking_stop(game, *entity, *player);
            }
        }
    }
}

pub(in crate::game) fn on_level_tick(game: &mut Game<'_>, dimension: &str, end: bool) {
    crate::game::paper_entity_events::deaths_now(game);
    if end && heard() {
        fire_block_entities(game, dimension);
        fire_mob_equipment(game);
    }
}

/// What went from this level, in order, with what it was: the map's writers write it down.
fn fire_block_entities(game: &mut Game<'_>, dimension: &str) {
    for (pos, gone) in game.world.block_entities.take_notes() {
        if let Some(entity) = gone {
            bridge::ev_block_entity_unload(game, dimension, [pos.0, pos.1, pos.2], &entity);
        }
    }
}

// ---- Players coming and going ----

/// `PlayerJoinEvent`, once the player is in, then this server's plugin channels.
pub(in crate::game) fn on_joined(game: &mut Game<'_>, player: u128, name: &str) {
    if heard() {
        bridge::ev_join(game, player, name);
    }
    tell_channels(game, player);
}

pub(in crate::game) fn on_left(game: &mut Game<'_>, player: u128, name: &str) {
    if let Some(all) = SENDABLE.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        all.remove(&player);
    }
    if heard() {
        if !net::left_for_configuration(player) {
            bridge::ev_disconnect(game, player, name);
        }
        bridge::ev_player_leave(game, player, name);
    }
}

// ---- Blocks, items and entities, as a player acts on them ----

pub(in crate::game) fn on_destroy_block(game: &mut Game<'_>, pos: [i32; 3], _state: BlockStateId, canceled: &mut bool) {
    if !heard() {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    if !bridge::ev_break_before(game, &player, pos) {
        *canceled = true;
    }
}

pub(in crate::game) fn on_left_click_block(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    if !heard() {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    *canceled |= bridge::ev_attack_block(game, &player, pos, face).decided();
}

pub(in crate::game) fn on_right_click_block(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    if !heard() {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    *canceled |= bridge::ev_use_block(game, &player, pos, face).decided();
}

pub(in crate::game) fn on_use_item(game: &mut Game<'_>, stack: &ItemStack, claimed: &mut bool) {
    if !heard() {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    *claimed |= bridge::ev_use_item(game, &player, stack).decided();
}

pub(in crate::game) fn on_use_item_on(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3], face: [i32; 3], claimed: &mut bool) {
    if !heard() {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    *claimed |= bridge::ev_item_use_on(game, &player, stack, pos, face).decided();
}

pub(in crate::game) fn on_block_changed(dimension: crate::game::LevelDimension, pos: horizon_core::position::BlockPos, old: BlockStateId, new: BlockStateId) {
    bridge::on_block_changed(dimension, pos, old, new);
}

pub(in crate::game) fn on_block_spread(_dimension: crate::game::LevelDimension, kind: &str, from: [i32; 3], to: [i32; 3], canceled: &mut bool) {
    if bridge::on_block_spread(kind, from, to) {
        *canceled = true;
    }
}

pub(in crate::game) fn on_interact_entity(game: &mut Game<'_>, entity: i32, kind: &str, stack: Option<&ItemStack>, claimed: &mut bool) {
    let _ = stack;
    if !heard() {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    *claimed |= bridge::ev_use_entity(game, &player, entity, kind).decided();
}

pub(in crate::game) fn on_attack_entity(game: &mut Game<'_>, entity: i32, kind: &str, pos: [f64; 3], canceled: &mut bool) {
    if !heard() {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    *canceled |= bridge::ev_attack_entity(game, &player, entity, kind, pos).decided();
}

// ---- Hits, deaths and what joins or leaves a level ----

pub(in crate::game) fn on_incoming_damage(entity: i32, kind: &str, damage_type: &str, causing: i32, direct: i32, changing: &mut f32, canceled: &mut bool) {
    let amount = *changing;
    hub::note_amount(amount);
    let damage = server_living_entity_events::Damage { entity, kind, damage_type, causing, direct, amount, player: None, mob: None };
    if !server_living_entity_events::allow_damage_mid(&damage) {
        *canceled = true;
    }
    if let Some(amount) = bridge::take_damage() {
        *changing = amount;
    }
    if !*canceled {
        crate::game::paper_entity_events::hit_coming(entity, kind, damage_type, causing, direct, *changing);
    }
}

pub(in crate::game) fn on_player_died(game: &mut Game<'_>, player: u128, cause: Option<&str>) {
    crate::game::paper_entity_events::player_dead(game, player, cause);
}

/// The player who killed the mob now dying, for its death at the end of `die`.
static KILLER: Mutex<Option<(u32, u128)>> = Mutex::new(None);

pub(in crate::game) fn on_mob_killed(_game: &mut Game<'_>, player: u128, entity: u32, _kind: &str) {
    *KILLER.lock().unwrap_or_else(|e| e.into_inner()) = Some((entity, player));
}

/// The end of `LivingEntity.die`, whatever killed the mob.
pub(in crate::game) fn on_mob_died(game: &mut Game<'_>, m: &horizon_client::mobs::Mob) {
    let killer = KILLER.lock().unwrap_or_else(|e| e.into_inner()).take().filter(|(id, _)| *id == m.id).map(|(_, player)| player);
    crate::game::paper_entity_events::mob_dead(game, m, killer);
}

pub(in crate::game) fn on_entity_join(game: &mut Game<'_>, dimension: &str, entity: u32, kind: &str, pos: [f64; 3], from_disk: bool, canceled: &mut bool) {
    if heard() {
        if !bridge::ev_entity_allow_load(game, dimension, entity, kind, pos, from_disk) {
            *canceled = true;
            return;
        }
        bridge::ev_entity_load(game, dimension, entity, kind, pos, from_disk);
    }
    crate::game::paper_entity_events::entity_arrived(game, entity, kind, pos);
}

pub(in crate::game) fn on_entity_leave(game: &mut Game<'_>, dimension: &str, entity: u32, kind: &str) {
    if heard() {
        bridge::ev_entity_unload(game, dimension, entity, kind);
    }
}

pub(in crate::game) fn on_chunk_load(game: &mut Game<'_>, dimension: &str, chunk: [i32; 2], new: bool) {
    if heard() {
        bridge::ev_chunk_load(game, dimension, chunk, new);
    }
}

pub(in crate::game) fn on_chunk_unload(game: &mut Game<'_>, dimension: &str, chunk: [i32; 2]) {
    if heard() {
        bridge::ev_chunk_unload(game, dimension, chunk);
    }
}

pub(in crate::game) fn on_loot_drops(game: &mut Game<'_>, _context: &NativeLootDrops<'_>, drops: &mut Vec<ItemStack>) {
    if heard() && bridge::take_no_drops(game) {
        drops.clear();
    }
}

pub(in crate::game) fn on_converted(game: &mut Game<'_>, index: usize) {
    crate::game::paper_entity_events::on_converted(game, index);
}

pub(in crate::game) fn on_effects_ticked(game: &mut Game<'_>) {
    crate::game::paper_entity_events::effects_ticked(game);
}

// ---- Equipment ----

/// What each player last wore, for `previous`: the game's hook tells only of what's new.
static WORN: Mutex<Option<HashMap<(u128, String), ItemStack>>> = Mutex::new(None);

pub(in crate::game) fn on_equip(game: &mut Game<'_>, slot: &str, new: Option<&ItemStack>) {
    use server_entity_events::EquipmentChange;
    if !heard() {
        return;
    }
    if let Some(player) = game.native_acting_player() {
        let key = (player.uuid, slot.to_owned());
        let previous = {
            let mut worn = WORN.lock().unwrap_or_else(|e| e.into_inner());
            let worn = worn.get_or_insert_with(HashMap::new);
            match new {
                Some(stack) => worn.insert(key, stack.clone()),
                None => worn.remove(&key),
            }
        };
        let change = EquipmentChange { player: Some(&player), mob: None, slot, previous: previous.as_ref(), current: new };
        bridge::ev_equipment_change(game, &change);
    }
}

/// `collectEquipmentChanges` for the mobs of this level: each compares with what it wore last tick.
fn fire_mob_equipment(game: &mut Game<'_>) {
    use server_entity_events::EquipmentChange;
    type Worn = [Option<ItemStack>; 6];
    static LAST: Mutex<Option<HashMap<u32, Worn>>> = Mutex::new(None);
    const SLOTS: [&str; 6] = ["mainhand", "offhand", "feet", "legs", "chest", "head"];
    let now: Vec<(u32, Worn)> = game
        .mobs
        .iter()
        .map(|m| {
            let e = &m.extra;
            let of = |item: &Option<std::sync::Arc<str>>, stack: Option<&ItemStack>| stack.cloned().or_else(|| item.as_ref().map(|item| ItemStack::new(item, 1)));
            (m.id, [of(&e.main_hand, e.main_hand_stack()), of(&e.off_hand, None), of(&e.feet, e.armor[3].as_ref()), of(&e.legs, e.armor[2].as_ref()), of(&e.chest, e.armor[1].as_ref()), of(&e.head, e.armor[0].as_ref())])
        })
        .collect();
    let same = |a: &Option<ItemStack>, b: &Option<ItemStack>| a.as_ref().map(|s| (s.item.clone(), s.count, s.damage)) == b.as_ref().map(|s| (s.item.clone(), s.count, s.damage));
    for (id, worn) in now {
        let last = LAST.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).insert(id, worn.clone()).unwrap_or_default();
        for (index, slot) in SLOTS.iter().enumerate() {
            if !same(&last[index], &worn[index]) {
                let change = EquipmentChange { player: None, mob: Some(id), slot, previous: last[index].as_ref(), current: worn[index].as_ref() };
                bridge::ev_equipment_change(game, &change);
            }
        }
    }
}

// ---- Saves and reloads ----

thread_local! {
    /// A save under way on this thread: the hook runs the save itself, which comes back through it.
    static SAVING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `saveAllChunks` at its head: whether the hook is to run the save and say when it's done.
pub(in crate::game) fn save_begins(game: &mut Game<'_>, wait: bool) -> bool {
    if SAVING.replace(true) {
        return false;
    }
    server_lifecycle_events::before_save(game, wait);
    true
}

/// `saveAllChunks` at its tail.
pub(in crate::game) fn save_ended(_game: &mut Game<'_>) {
    SAVING.set(false);
}

/// A closing world's save runs apart from `save_game_with`, on other threads: `WorldSaveEvent` once, before it.
pub(in crate::game) fn closing_save_begins(game: &mut Game<'_>, wait: bool) {
    static TOLD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if game.pending_save.is_none() && !TOLD.swap(true, std::sync::atomic::Ordering::Relaxed) {
        server_lifecycle_events::before_save(game, wait);
    }
}

pub(in crate::game) fn on_reload(game: &mut Game<'_>) {
    server_lifecycle_events::start_data_pack_reload(game);
}

/// The tail of a reload: the new tables are in and the players have them.
pub(in crate::game) fn on_reload_done(game: &mut Game<'_>, success: bool) {
    // A reload makes a new `Commands`, and the plugins' words go into it at the next turn.
    core::with_hub(game, |game| {
        if let Some(host) = game.lan.as_mut() {
            host.native_commands = None;
        }
    });
    server_lifecycle_events::end_data_pack_reload(game, success);
}

// ---- Beds: `PlayerBedEnterEvent` ----

pub(in crate::game) fn on_try_sleep(game: &mut Game<'_>, pos: [i32; 3]) -> bool {
    let Some(player) = game.native_acting_player() else {
        return true;
    };
    entity_sleep_events::allow_sleeping(game, &player, pos).is_none()
}

pub(in crate::game) fn on_nearby_monsters(game: &mut Game<'_>, pos: [i32; 3], monsters: bool) -> bool {
    let Some(player) = game.native_acting_player().filter(|_| !game.creative) else {
        return monsters;
    };
    let safe = !monsters;
    !entity_sleep_events::allow_nearby_monsters(game, &player, pos, safe).allow_action(safe)
}
