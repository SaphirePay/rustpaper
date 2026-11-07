// The game's moments, told to the plugin libraries. They get plain JSON and answer with a number, and what they ask
// of the game waits in a queue for a safe moment, so a plugin needs none of the game's types and keeps working when
// the game is rebuilt.
use crate::game::{
    Game,
    paper_events::{self as api},
    native_server::NativeServerPlayer,
};
use horizon_client::{paper_hub as hub, items::ItemStack, text::Text};
use horizon_mod_runtime as runtime;
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{Arc, Mutex, Once, atomic::Ordering},
};

const ALLOWED: i32 = 0;
const REFUSED: i32 = 2;

fn uuid(player: u128) -> String {
    format!("{player:032x}")
}

fn player_json(p: &NativeServerPlayer) -> Value {
    json!({ "uuid": uuid(p.uuid), "name": p.name, "dimension": p.dimension, "position": p.position, "health": p.health, "level": p.permission, "game_mode": p.game_mode })
}

fn stack_json(stack: &ItemStack) -> Value {
    super::paper_query::stack_json(stack)
}

fn fire(event: &str, payload: Value) -> i32 {
    if runtime::listening() { runtime::fire(event, &payload.to_string()) } else { 0 }
}

/// `fire`, with the game open to the listeners' questions.
fn fire_in(game: &mut Game<'_>, event: &str, mut payload: Value) -> i32 {
    if !runtime::listening() {
        return 0;
    }
    if event == "command"
        && let Some(owners) = super::paper_events::command_route(&mut payload)
    {
        if owners.is_empty() {
            if let Some(player) = payload["player"]["uuid"].as_str().or(payload["player"].as_str()) {
                runtime::push(json!({ "op": "tell", "player": player, "text": "\u{a7}cUnknown or disabled command." }).to_string());
            }
            return 0;
        }
        return runtime::lend(game, || runtime::fire_only(event, &payload.to_string(), &owners));
    }
    runtime::lend(game, || runtime::fire(event, &payload.to_string()))
}

/// `PlayerCommandPreprocessEvent`, for any command a player sends: true refuses it.
pub(in crate::game) fn on_command_preprocess(game: &mut Game<'_>, player: u128, name: &str, line: &str) -> bool {
    if !runtime::listens("command_preprocess") {
        return false;
    }
    // As the player typed it: `//set` (WorldEdit's) keeps both slashes, as on Paper.
    let message = format!("/{line}");
    fire_in(game, "command_preprocess", json!({ "uuid": uuid(player), "name": name, "message": message })) == REFUSED
}

/// `BlockPlaceEvent`, once the block stands: the game has no way back from this moment.
pub(in crate::game) fn on_block_placed(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3]) {
    if !runtime::listens("block_place") {
        return;
    }
    let player = game.native_acting_id();
    fire_in(game, "block_place", json!({ "uuid": uuid(player), "pos": pos, "stack": stack_json(stack) }));
    flush(game, true);
}

/// `PlayerDropItemEvent`, as the stack leaves the player.
pub(in crate::game) fn on_item_dropped(game: &mut Game<'_>, player: u128, item: &str, count: u32) {
    if runtime::listens("item_drop") {
        fire_in(game, "item_drop", json!({ "uuid": uuid(player), "stack": { "item": item, "count": count } }));
        flush(game, true);
    }
}

/// `EntityPickupItemEvent`: a refusal leaves the item on the ground.
pub(in crate::game) fn on_item_picked_up(game: &mut Game<'_>, player: u128, item: &str, count: u32, canceled: &mut bool) {
    if runtime::listens("item_pickup") {
        *canceled |= fire_in(game, "item_pickup", json!({ "uuid": uuid(player), "stack": { "item": item, "count": count } })) == REFUSED;
        flush(game, true);
    }
}

/// `PlayerBucketFillEvent`.
pub(in crate::game) fn on_fill_bucket(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3], canceled: &mut bool) {
    if runtime::listens("bucket_fill") {
        let player = game.native_acting_id();
        *canceled |= fire_in(game, "bucket_fill", json!({ "uuid": uuid(player), "pos": pos, "stack": stack_json(stack) })) == REFUSED;
        flush(game, true);
    }
}

/// `EntityExplodeEvent` and `BlockExplodeEvent`: a refusal keeps every block.
pub(in crate::game) fn on_explosion(game: &mut Game<'_>, dimension: &str, center: [f64; 3], blocks: &mut Vec<[i32; 3]>) {
    if runtime::listens("explosion") {
        let block = center.map(|value| value.floor() as i32);
        if fire_in(game, "explosion", json!({ "level": dimension, "center": center, "block": block, "blocks": blocks })) == REFUSED {
            blocks.clear();
        }
        flush(game, true);
    }
}

/// A mod's say in a piston: Bukkit's events ask before it moves, in `run_asked`.
pub(in crate::game) fn on_piston(_piston: horizon_core::position::BlockPos, _direction: [i32; 3], _push: &[horizon_core::position::BlockPos], _canceled: &mut bool) {}

/// `BlockSpreadEvent`: true refuses it.
pub(in crate::game) fn on_block_spread(_kind: &str, _from: [i32; 3], _to: [i32; 3]) -> bool {
    false
}

/// `PlayerItemConsumeEvent`, as a food or potion finishes.
pub(in crate::game) fn on_item_consumed(game: &mut Game<'_>, stack: &ItemStack) {
    if runtime::listens("item_consume") {
        let player = game.native_acting_id();
        fire_in(game, "item_consume", json!({ "uuid": uuid(player), "stack": stack_json(stack) }));
        flush(game, true);
    }
}

/// `ProjectileHitEvent`.
pub(in crate::game) fn on_projectile_hit(game: &mut Game<'_>, id: u32, kind: &str, at: [f64; 3], guest: Option<u32>) {
    if runtime::listens("projectile_hit") {
        let shooter = guest.and_then(|conn| game.lan.as_ref()?.hub.guest(conn).map(|g| uuid(g.uuid)));
        fire_in(game, "projectile_hit", json!({ "entity": id, "kind": kind, "pos": at, "shooter": shooter }));
        flush(game, true);
    }
}

/// `EntityBreedEvent`: the parents, as the child is about to be made.
pub(in crate::game) fn on_mob_bred(game: &mut Game<'_>, parent: u32, partner: u32) {
    if runtime::listens("mob_bred") {
        fire_in(game, "mob_bred", json!({ "mother": parent, "father": partner }));
        flush(game, true);
    }
}

/// `PlayerFishEvent`, as a rod is cast or reeled in.
pub(in crate::game) fn on_rod_used(game: &mut Game<'_>) {
    if runtime::listens("rod_used") {
        let player = game.native_acting_id();
        fire_in(game, "rod_used", json!({ "uuid": uuid(player) }));
        flush(game, true);
    }
}

/// `InventoryOpenEvent` for a block's own menu.
pub(in crate::game) fn on_container_opened(game: &mut Game<'_>, conn: u32, pos: (i32, i32, i32)) {
    if runtime::listens("container_open") {
        let player = game.lan.as_ref().and_then(|host| host.hub.guest(conn)).map(|g| uuid(g.uuid));
        fire_in(game, "container_open", json!({ "uuid": player, "pos": [pos.0, pos.1, pos.2] }));
        flush(game, true);
    }
}

/// What a listener said can come next, for the request being answered.
static SUGGESTIONS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// `{ "op": "suggest", "matches": [...] }`, from a plugin's tab completer.
pub(in crate::game) fn add_suggestions(matches: &Value) {
    SUGGESTIONS.lock().unwrap_or_else(|e| e.into_inner()).extend(matches.as_array().into_iter().flatten().filter_map(|text| text.as_str().map(str::to_owned)));
}

/// A client's request to complete a command: true if a plugin's completer answered it.
pub(in crate::game) fn on_suggestions(game: &mut Game<'_>, host: &mut crate::game::lan::LanHost, conn: u32, payload: &[u8]) -> bool {
    if !runtime::listens("tab_complete") {
        return false;
    }
    let mut reader = horizon_protocol::buf::Reader::new(payload);
    let (Ok(id), Ok(text)) = (reader.varint(), reader.string(32500)) else {
        return false;
    };
    let Some((player, name)) = host.hub.guest(conn).map(|g| (g.uuid, g.name.clone())) else {
        return false;
    };
    SUGGESTIONS.lock().unwrap_or_else(|e| e.into_inner()).clear();
    fire_in(game, "tab_complete", json!({ "uuid": uuid(player), "name": name, "line": text }));
    let matches = std::mem::take(&mut *SUGGESTIONS.lock().unwrap_or_else(|e| e.into_inner()));
    if matches.is_empty() {
        return super::paper_fam_ev_world::suggestions_cancelled();
    }
    let Some(matches) = super::paper_fam_ev_world::tab_matches(game, player, &name, &text, matches) else {
        return true;
    };
    let start = text.rfind(' ').map_or(usize::from(text.starts_with('/')), |space| space + 1);
    let utf16 = |upto: usize| text[..upto].encode_utf16().count() as i32;
    let mut out = horizon_protocol::buf::Writer::new();
    out.varint(id);
    out.varint(utf16(start));
    out.varint(utf16(text.len()) - utf16(start));
    out.varint(matches.len() as i32);
    for suggestion in &matches {
        out.string(suggestion);
        match super::paper_fam_ev_world::tooltip(suggestion) {
            Some(tooltip) => {
                out.bool(true);
                out.nbt(&tooltip);
            }
            None => out.bool(false),
        }
    }
    host.hub.send_to(conn, &horizon_protocol::packets::play::Clientbound::Unknown(horizon_protocol::ids::play::clientbound::COMMAND_SUGGESTIONS, out.data));
    true
}

/// `SignChangeEvent`: true refuses what was typed.
pub(in crate::game) fn on_sign_update(game: &mut Game<'_>, host: &crate::game::lan::LanHost, conn: u32, pos: [i32; 3], lines: &[String; 4], front: bool) -> bool {
    if !runtime::listens("sign_change") {
        return false;
    }
    let Some(player) = host.hub.guest(conn).map(|g| (g.uuid, g.name.clone())) else {
        return false;
    };
    fire_in(game, "sign_change", json!({ "uuid": uuid(player.0), "name": player.1, "pos": pos, "lines": lines, "front": front })) == REFUSED
}

/// An event of a player and the stack they used.
pub(in crate::game) fn on_player_item(game: &mut Game<'_>, event: &str, stack: &ItemStack) {
    if runtime::listens(event) {
        let player = game.native_acting_id();
        fire_in(game, event, json!({ "uuid": uuid(player), "stack": stack_json(stack) }));
        flush(game, true);
    }
}

/// An event of a player alone.
pub(in crate::game) fn on_rod_like(game: &mut Game<'_>, event: &str) {
    if runtime::listens(event) {
        let player = game.native_acting_id();
        fire_in(game, event, json!({ "uuid": uuid(player) }));
        flush(game, true);
    }
}

/// A player dropping what they hold, or swapping hands: true refuses it, and their inventory is
/// sent again to undo what their client already did.
pub(in crate::game) fn on_player_action(game: &mut Game<'_>, host: &mut crate::game::lan::LanHost, conn: u32, action: i32) -> bool {
    let event = if action == 6 { "swap_hands" } else { "drop_request" };
    if !runtime::listens(event) {
        return false;
    }
    let Some(index) = host.hub.guests.iter().position(|guest| guest.conn == conn) else {
        return false;
    };
    let guest = &host.hub.guests[index];
    let held = guest.inventory.get(guest.selected).cloned().flatten();
    let payload = json!({
        "uuid": uuid(guest.uuid),
        "name": guest.name,
        "stack": held.as_ref().map(stack_json),
        "off_hand": guest.offhand.as_ref().map(stack_json),
        "all": action == 3,
    });
    let refused = fire_in(game, event, payload) == REFUSED;
    if refused {
        host.hub.send_inventory(index);
    }
    refused
}

/// Whether a plugin listens: a trigger's payload is only made for those that do.
pub(in crate::game) fn hears(event: &str) -> bool {
    runtime::listens(event)
}

/// The id of the player on a connection, for an event's JSON.
pub(in crate::game) fn uuid_of(host: &crate::game::lan::LanHost, conn: u32) -> Value {
    host.hub.guest(conn).map_or(Value::Null, |guest| json!(uuid(guest.uuid)))
}

/// Who joined, by id: in a guest's own turn the game has no list of players to ask.
static NAMES: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<u128, String>>> = std::sync::LazyLock::new(Default::default);

fn name_of(player: u128) -> Value {
    NAMES.lock().unwrap_or_else(|e| e.into_inner()).get(&player).map_or(Value::Null, |name| json!(name))
}

/// The acting player on a payload that doesn't name one, with their name, so a listener can always say who.
fn name_payload(game: &mut Game<'_>, payload: &mut Value) {
    if payload.get("uuid").is_none() {
        payload["uuid"] = json!(uuid(game.native_acting_id()));
    }
    if payload.get("name").is_none()
        && let Some(player) = payload["uuid"].as_str().and_then(|id| u128::from_str_radix(id, 16).ok())
    {
        payload["name"] = name_of(player);
    }
}

/// An engine moment of the acting player: the game's own moment for many of Bukkit's events.
pub(in crate::game) fn on_trigger(game: &mut Game<'_>, event: &str, mut payload: Value) {
    name_payload(game, &mut payload);
    fire_in(game, event, payload);
    flush(game, true);
}

/// `on_trigger` for what a listener can stop: true if one did.
pub(in crate::game) fn on_refusable(game: &mut Game<'_>, event: &str, mut payload: Value) -> bool {
    name_payload(game, &mut payload);
    let refused = fire_in(game, event, payload) == REFUSED;
    flush(game, true);
    refused
}

// ---- What the world's own ticks did to blocks ----

/// True while the level runs its block ticks: a change then is the world's, not a player's.
static BLOCK_TICK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
type BlockChange = (crate::game::LevelDimension, [i32; 3], horizon_world::registry::BlockStateId, horizon_world::registry::BlockStateId, bool);
static BLOCK_CHANGES: std::sync::Mutex<Vec<BlockChange>> = std::sync::Mutex::new(Vec::new());

/// The end of a moment of block changes outside the block ticks: bone meal's.
pub(in crate::game) fn on_block_tick_pause() {
    BONE_MEAL.store(false, std::sync::atomic::Ordering::Relaxed);
    BLOCK_TICK.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// True while bone meal works: what it does isn't asked about first, so it's told after.
static BONE_MEAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(in crate::game) fn on_bone_meal_begin() {
    BONE_MEAL.store(true, std::sync::atomic::Ordering::Relaxed);
    on_block_tick_begin();
}

/// Whether block changes are being noted; off while a change that's been asked about is made.
pub(in crate::game) fn noting_changes(on: bool) -> bool {
    BLOCK_TICK.swap(on, std::sync::atomic::Ordering::Relaxed)
}

pub(in crate::game) fn on_block_tick_begin() {
    BLOCK_TICK.store(runtime::listens("block_change"), std::sync::atomic::Ordering::Relaxed);
}

/// True while mobs tick: a change then is `EntityChangeBlockEvent`.
static MOB_TICK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(in crate::game) fn on_mob_tick_begin() {
    let listening = runtime::listens("block_change");
    MOB_TICK.store(listening, std::sync::atomic::Ordering::Relaxed);
    BLOCK_TICK.store(listening, std::sync::atomic::Ordering::Relaxed);
}

/// A block just changed. Listeners hear of it after the block ticks, when the game can answer them.
pub(in crate::game) fn on_block_changed(dimension: crate::game::LevelDimension, pos: horizon_core::position::BlockPos, old: horizon_world::registry::BlockStateId, new: horizon_world::registry::BlockStateId) {
    if old != new && BLOCK_TICK.load(std::sync::atomic::Ordering::Relaxed) {
        BLOCK_CHANGES.lock().unwrap_or_else(|e| e.into_inner()).push((dimension, [pos.x, pos.y, pos.z], old, new, BONE_MEAL.load(std::sync::atomic::Ordering::Relaxed)));
    }
}

fn state_text(state: &horizon_world::chunk::BlockState) -> String {
    if state.properties.is_empty() {
        return state.name.clone();
    }
    let properties: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
    format!("{}[{}]", state.name, properties.join(","))
}

/// Which of Bukkit's block events a change of the world's is.
fn block_change_kind(old: &horizon_world::chunk::BlockState, new: &horizon_world::chunk::BlockState) -> &'static str {
    let (from, to) = (old.name.trim_start_matches("minecraft:"), new.name.trim_start_matches("minecraft:"));
    let number = |state: &horizon_world::chunk::BlockState, key: &str| state.properties.get(key).and_then(|value| value.parse::<i32>().ok());
    if from == to {
        if from == "farmland" {
            return "moisture";
        }
        if from == "water" || from == "lava" {
            return "fluid";
        }
        if from.ends_with("cauldron") {
            return "cauldron";
        }
        if ["power", "powered", "lit"].iter().any(|key| old.properties.get(*key) != new.properties.get(*key)) {
            // Levers and buttons ask as they move; the rest of what changes these has no event.
            let event = from == "redstone_wire" || from.ends_with("pressure_plate") || from == "daylight_detector" || from.ends_with("sculk_sensor") || from == "target" || from == "trapped_chest";
            return if event { "redstone" } else { "other" };
        }
        if number(new, "age") > number(old, "age") || number(new, "stage") > number(old, "stage") {
            return "grow";
        }
        return "other";
    }
    let empty = |name: &str| matches!(name, "air" | "cave_air" | "void_air");
    if from.ends_with("_leaves") && empty(to) {
        return "decay";
    }
    if to == "fire" || to == "soul_fire" {
        return if empty(from) { "ignite" } else { "burn" };
    }
    if from.ends_with("cauldron") || to.ends_with("cauldron") {
        return "cauldron";
    }
    if matches!(from, "ice" | "frosted_ice" | "snow" | "snow_block" | "fire" | "soul_fire") && (empty(to) || to == "water")
        || to.strip_prefix("dead_").is_some_and(|alive| alive == from)
        || (from == "farmland" && to == "dirt")
    {
        return "fade";
    }
    if matches!(to, "ice" | "frosted_ice" | "snow" | "obsidian" | "cobblestone" | "stone" | "basalt") && (empty(from) || from == "water" || from == "lava")
        || (from.ends_with("_concrete_powder") && to.ends_with("_concrete"))
    {
        return "form";
    }
    if from.ends_with("_sapling") || from == "mangrove_propagule" || from.ends_with("_fungus") || from.ends_with("mushroom") {
        return "structure";
    }
    if empty(from) || from == "water" {
        return "grow";
    }
    "other"
}

/// The block ticks are over: each change goes to the listeners, and what they refuse is put back.
pub(in crate::game) fn on_block_tick_end(game: &mut Game<'_>) {
    super::paper_fam_ev_world::run_asked(game);
    BLOCK_TICK.store(false, std::sync::atomic::Ordering::Relaxed);
    let by_mobs = MOB_TICK.swap(false, std::sync::atomic::Ordering::Relaxed);
    let changes = std::mem::take(&mut *BLOCK_CHANGES.lock().unwrap_or_else(|e| e.into_inner()));
    for (dimension, pos, old, new, told_after) in changes {
        let level = match dimension {
            crate::game::LevelDimension::Overworld => "minecraft:overworld",
            crate::game::LevelDimension::Nether => "minecraft:the_nether",
            crate::game::LevelDimension::End => "minecraft:the_end",
        };
        let registry = game.pack.assets.registry;
        let (Some(before), Some(after)) = (registry.state(old), registry.state(new)) else {
            continue;
        };
        let number = |state: &horizon_world::chunk::BlockState, key: &str| state.properties.get(key).and_then(|value| value.parse::<i32>().ok()).unwrap_or(0);
        let on = |state: &horizon_world::chunk::BlockState| {
            let powered = state.properties.get("powered").is_some_and(|v| v == "true") || state.properties.get("lit").is_some_and(|v| v == "true");
            number(state, "power").max(if powered { 15 } else { 0 })
        };
        let kind = if by_mobs { "entity_change" } else { block_change_kind(before, after) };
        // These are asked about as they happen, all but bone meal's: what's left is `BlockPhysicsEvent`'s.
        let kind = if !told_after && matches!(kind, "grow" | "form" | "fade" | "decay" | "moisture" | "fluid" | "burn" | "ignite") { "other" } else { kind };
        let payload = json!({
            "kind": kind,
            "level": level,
            "pos": pos,
            "old": state_text(before),
            "new": state_text(after),
            "old_level": number(before, "level"),
            "new_level": number(after, "level"),
            "old_current": on(before),
            "new_current": on(after),
        });
        if fire_in(game, "block_change", payload) == REFUSED {
            game.native_in_level(level, |game| game.native_set_block(game.dim.id(), pos, old, 3));
        }
        flush(game, false);
    }
}

/// A listener's `setDamage`, for the hit being decided.
static DAMAGE: std::sync::Mutex<Option<f32>> = std::sync::Mutex::new(None);

pub(in crate::game) fn take_damage() -> Option<f32> {
    DAMAGE.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// `BlockBreakEvent.setDropItems(false)`: the game tick it was asked in.
static NO_DROPS: std::sync::Mutex<Option<u64>> = std::sync::Mutex::new(None);

pub(in crate::game) fn no_drops(game: &Game<'_>) {
    *NO_DROPS.lock().unwrap_or_else(|e| e.into_inner()) = Some(game.world.game_time as u64);
}

pub(in crate::game) fn take_no_drops(game: &Game<'_>) -> bool {
    NO_DROPS.lock().unwrap_or_else(|e| e.into_inner()).take() == Some(game.world.game_time as u64)
}

/// `PluginManager.callEvent`: true if a listener refused.
pub(in crate::game) fn fire_custom(game: &mut Game<'_>, event: &str, payload: Value) -> bool {
    fire_in(game, event, payload) == REFUSED
}

/// The names of the plugins the server loaded, in load order.
pub(in crate::game) fn mod_ids() -> Value {
    json!(crate::game::paper_plugins::get().map_or_else(Vec::new, |plugins| plugins.loaded.iter().map(|p| p.name.clone()).collect::<Vec<_>>()))
}

/// `{ "op": "line_set" }`: `setJoinMessage`, `setQuitMessage`, `setDeathMessage`. Null says nothing.
pub(in crate::game) fn line_set(kind: usize, line: Option<horizon_world::nbt::Tag>) {
    if let Some(slot) = hub::LINES.lock().unwrap_or_else(|e| e.into_inner()).get_mut(kind) {
        slot.changed = Some(line);
    }
}

/// The end of a tick: the lines whose events have been heard go out, once, whoever changed them.
pub(in crate::game) fn flush_lines(game: &mut Game<'_>) {
    // Out of the hub's reach a line would go nowhere: it stays held for the next look.
    if game.lan.is_none() {
        return;
    }
    let mut out = Vec::new();
    {
        let mut lines = hub::LINES.lock().unwrap_or_else(|e| e.into_inner());
        for line in lines.iter_mut().take(2) {
            if line.held.is_none() {
                line.ready = false;
                line.changed = None;
                continue;
            }
            line.waited += 1;
            if line.ready || line.waited > 40 {
                let held = line.held.take();
                out.push(line.changed.take().unwrap_or(held));
                line.ready = false;
            }
        }
    }
    if let Some(host) = game.lan.as_ref() {
        hub::LINES_LEAVING.store(true, std::sync::atomic::Ordering::Relaxed);
        for line in out.into_iter().flatten() {
            host.hub.system_message(line);
        }
        hub::LINES_LEAVING.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

/// What clients sent this tick, for the events made of it.
fn fire_packets(game: &mut Game<'_>) {
    for (player, event, mut payload) in hub::take_packets() {
        if runtime::listens(event) {
            payload["uuid"] = json!(uuid(player));
            fire_in(game, event, payload);
        }
    }
}

/// A click in a menu a mod opened: `InventoryClickEvent`.
pub(in crate::game) fn on_menu_clicked(game: &mut Game<'_>, player: u128, window: i32, slot: i32, button: i32, mode: i32) {
    if runtime::listens("menu_click") {
        fire_in(game, "menu_click", json!({ "uuid": uuid(player), "window": window, "slot": slot, "button": button, "mode": mode }));
        flush(game, false);
    }
}

/// `InventoryCloseEvent`.
pub(in crate::game) fn on_menu_closed(game: &mut Game<'_>, player: u128, window: i32) {
    if runtime::listens("menu_close") {
        fire_in(game, "menu_close", json!({ "uuid": uuid(player), "window": window }));
        flush(game, false);
    }
}

/// `Messenger.registerIncomingPluginChannel`: what arrives on `channel` goes to the mods.
pub(in crate::game) fn listen_channel(channel: &str) {
    let name = channel.to_owned();
    api::register_global_receiver(channel, Arc::new(move |game, player, data| {
        fire_in(game, "plugin_message", json!({ "uuid": uuid(player), "name": "", "channel": name, "data": data }));
        flush(game, false);
    }));
}

fn answer_query(request: &str) -> String {
    if crate::game::paper_worlds::take_chunk(request) {
        return "true".to_owned();
    }
    if let Some(answer) = crate::game::paper_worlds::free_query(request) {
        return answer;
    }
    if let Some(answer) = super::paper_fam_ev_world::free_query(request) {
        return answer;
    }
    if let Some(answer) = crate::game::paper_query::free_packets(request) {
        return answer;
    }
    if request.contains(r#""op":"core_enable_later""#) {
        return enables_later().to_string();
    }
    // A listener's change to pass on: chat fires with no game to lend.
    if request.contains(r#""op":"event_patch""#) {
        if let Ok(value) = serde_json::from_str::<Value>(request) {
            runtime::patch(value["key"].as_str().unwrap_or_default(), value["value"].as_str().unwrap_or("null"));
        }
        return "true".to_owned();
    }
    runtime::borrowed(|game: &mut Game<'static>| crate::game::paper_query::answer(game, request))
        .unwrap_or_else(|| r#"{"error":"a query needs an event that has the game"}"#.to_owned())
}

/// A mod's answer to a block use: 0, 1 and 2 as `answer`, then the rest of `InteractionResult`.
fn use_answer(code: i32) -> crate::game::paper_events::block_events::Use {
    use crate::game::paper_events::block_events::Use;
    match code {
        0 => Use::Default,
        1 => Use::Success,
        3 => Use::SuccessServer,
        4 => Use::Consume,
        5 => Use::Pass,
        6 => Use::TryEmptyHand,
        _ => Use::Fail,
    }
}

fn answer(code: i32) -> api::Decision {
    match code {
        0 => api::Decision::Pass,
        1 => api::Decision::Success,
        _ => api::Decision::Fail,
    }
}

/// Carries out what the mods asked. Inside a player's hook a console command waits for the tick.
fn flush(game: &mut Game<'_>, inside_player: bool) {
    runtime::adopt_late();
    for action in runtime::take_actions() {
        let Ok(value) = serde_json::from_str::<Value>(&action) else {
            log::warn!("[paper_bridge] not JSON: {action}");
            continue;
        };
        let text = value["text"].as_str().unwrap_or_default();
        let player = value["player"].as_str().and_then(|p| u128::from_str_radix(p, 16).ok());
        match (value["op"].as_str().unwrap_or_default(), player) {
            ("tell" | "action_bar", Some(_)) if game.lan.is_none() && game.lan_acting.is_some() => runtime::push(action),
            ("world_close", _) if inside_player || game.lan.is_none() || game.lan_acting.is_some() => runtime::push(action),
            ("world_close", _) => super::paper_worlds::close(game, text, value["save"].as_bool().unwrap_or(true)),
            ("open_station", _) | ("close_all", Some(_)) if inside_player || game.lan.is_none() || game.lan_acting.is_some() => runtime::push(action),
            ("open_station", _) => super::paper_fam_entity::open_station_now(game, &value["request"]),
            ("close_all", Some(player)) => {
                if let Some(mut host) = game.lan.take() {
                    if let Some(conn) = host.hub.guests.iter().find(|guest| guest.uuid == player).map(|guest| guest.conn) {
                        game.guest_close_all(&mut host, conn);
                    }
                    game.lan = Some(host);
                }
            }
            ("run_as" | "hide_sync", Some(_)) if inside_player || game.lan.is_none() || game.lan_acting.is_some() => runtime::push(action),
            ("run_as", Some(player)) => super::paper_fam_ess::run_as(game, player, text),
            ("hide_sync", Some(player)) => {
                if let Some(target) = value["target"].as_str().and_then(|target| u128::from_str_radix(target, 16).ok()) {
                    super::paper_fam_ess::hide_sync(game, player, target, value["hide"].as_bool().unwrap_or(true));
                }
            }
            ("tell", Some(player)) => {
                game.native_tell(player, Text::literal(text));
            }
            ("action_bar", Some(player)) => {
                game.native_action_bar(player, Text::literal(text));
            }
            ("broadcast", _) => game.native_broadcast(Text::literal(text)),
            ("feedback", _) => game.native_feedback(Text::literal(text)),
            ("run_command" | "dispatch_command", _) if inside_player => runtime::push(action),
            ("run_command", _) => {
                super::paper_fam_core::run_game_only(game, text);
            }
            ("dispatch_command", _) => dispatch_line(game, text),
            // As Paper prints a plugin's logger: the SDK puts the plugin's name in front.
            ("log", _) => log::info!("{text}"),
            ("register_command", _) => log::warn!("[paper_bridge] commands register while a mod starts: {text}"),
            (op, _) => log::warn!("[paper_bridge] unknown action {op}"),
        }
    }
}

/// `Bukkit.dispatchCommand`: the plugin that took the word, or the game.
fn dispatch_line(game: &mut Game<'_>, text: &str) {
    let text = text.trim_start_matches('/');
    if let Some(line) = text.strip_prefix("minecraft:") {
        super::paper_fam_core::run_game_only(game, line);
        return;
    }
    let line = unprefixed(text);
    let word = line.split(' ').next().unwrap_or_default();
    if game.lan.as_ref().and_then(|host| host.native_commands.as_ref()).is_some_and(|words| words.contains_key(word)) {
        on_trigger(game, "command", json!({ "line": line, "player": Value::Null }));
    } else {
        super::paper_fam_core::run_game_only(game, line);
    }
}

/// Commands a mod registered as it started: one word of the tree each, answered by event.
fn take_commands() {
    for action in runtime::take_actions() {
        let Ok(value) = serde_json::from_str::<Value>(&action) else {
            continue;
        };
        if value["op"] != "register_command" {
            runtime::push(action);
            continue;
        }
        let usage = value["text"].as_str().unwrap_or_default();
        let level = u8::try_from(value["level"].as_u64().unwrap_or(0)).unwrap_or(4);
        if let Some(word) = usage.split_whitespace().next().filter(|w| !w.contains(':')) {
            super::paper_events::note_word(word, value["owner"].as_str());
        }
        api::register_plugin_command(usage, level, Arc::new(|game, player, line| {
            fire_in(game, "command", json!({ "player": player_json(player), "line": line }));
            flush(game, true);
        }));
    }
}

fn maybe_uuid(player: Option<u128>) -> Value {
    player.map_or(Value::Null, |player| json!(uuid(player)))
}

/// Events a mod only hears: each goes out as `name` with its fields.
/// An event the plugins hear by name, with the game open to them, and what they asked carried out after.
fn told(game: &mut Game<'_>, name: &str, payload: impl FnOnce() -> Value) {
    if runtime::listens(name) {
        fire_in(game, name, payload());
        flush(game, false);
    }
}

// ---- What `paper_events` tells the plugins: one function per moment, called by the hook that has it ----

pub(in crate::game) fn ev_server_started(game: &mut Game<'_>) {
    told(game, "server_started", || Value::Null);
}

pub(in crate::game) fn ev_server_stopping(game: &mut Game<'_>) {
    told(game, "server_stopping", || Value::Null);
}

pub(in crate::game) fn ev_reload_end(game: &mut Game<'_>, success: bool) {
    told(game, "reload_end", || json!({ "success": success }));
}

pub(in crate::game) fn ev_save_before(game: &mut Game<'_>, wait: bool) {
    told(game, "save_before", || json!({ "wait": wait }));
}

pub(in crate::game) fn ev_tick_start(game: &mut Game<'_>) {
    told(game, "tick_start", || json!({ "server_tick": true }));
}

pub(in crate::game) fn ev_level_load(game: &mut Game<'_>, level: &str) {
    told(game, "level_load", || json!({ "level": level }));
}

pub(in crate::game) fn ev_player_leave(game: &mut Game<'_>, player: u128, name: &str) {
    told(game, "player_leave", || json!({ "uuid": uuid(player), "name": name }));
}

pub(in crate::game) fn ev_tracking_stop(game: &mut Game<'_>, entity: i32, player: u128) {
    told(game, "tracking_stop", || json!({ "entity": entity, "uuid": uuid(player) }));
}

pub(in crate::game) fn ev_block_entity_unload(game: &mut Game<'_>, level: &str, pos: [i32; 3], _entity: &horizon_client::containers::BlockEntity) {
    told(game, "block_entity_unload", || json!({ "level": level, "pos": pos }));
}

pub(in crate::game) fn ev_chunk_load(game: &mut Game<'_>, level: &str, chunk: [i32; 2], new: bool) {
    told(game, "chunk_load", || json!({ "level": level, "chunk": chunk, "new": new }));
}

pub(in crate::game) fn ev_chunk_unload(game: &mut Game<'_>, level: &str, chunk: [i32; 2]) {
    told(game, "chunk_unload", || json!({ "level": level, "chunk": chunk }));
}

pub(in crate::game) fn ev_entity_load(game: &mut Game<'_>, level: &str, entity: u32, kind: &str, pos: [f64; 3], from_disk: bool) {
    told(game, "entity_load", || json!({ "level": level, "entity": entity, "kind": kind, "pos": pos, "from_disk": from_disk }));
}

pub(in crate::game) fn ev_entity_unload(game: &mut Game<'_>, level: &str, entity: u32, kind: &str) {
    told(game, "entity_unload", || json!({ "level": level, "entity": entity, "kind": kind }));
}

pub(in crate::game) fn ev_mob_death(game: &mut Game<'_>, death: &api::server_living_entity_events::Death<'_>) {
    told(game, "mob_death", || json!({ "killer": maybe_uuid(death.killer), "entity": death.entity, "kind": death.kind }));
}

pub(in crate::game) fn ev_entity_change_level(game: &mut Game<'_>, entity: u32, new: u32, kind: &str, from: &str, to: &str) {
    told(game, "entity_change_level", || json!({ "entity": entity, "new": new, "kind": kind, "from": from, "to": to }));
}

pub(in crate::game) fn ev_mob_conversion(game: &mut Game<'_>, c: &api::server_living_entity_events::Conversion<'_>) {
    told(game, "mob_conversion", || {
        json!({ "from": c.old.id, "kind": &*c.old.kind.id, "to": c.new, "new_kind": c.new_kind, "keep_equipment": c.keep_equipment, "preserve_can_pick_up_loot": c.preserve_can_pick_up_loot, "split": c.split })
    });
}

/// `EntitySpawnEvent` and the like, before the entity is in: false keeps it out.
pub(in crate::game) fn ev_entity_allow_load(game: &mut Game<'_>, level: &str, entity: u32, kind: &str, pos: [f64; 3], from_disk: bool) -> bool {
    if !runtime::listens("entity_allow_load") {
        return true;
    }
    // With the game lent, and whether a mob joins: `CreatureSpawnEvent` rather than its parent.
    let code = fire_in(game, "entity_allow_load", json!({ "level": level, "entity": entity, "kind": kind, "pos": pos, "from_disk": from_disk, "mob": super::paper_fam_ev_entity::joining_mob() }));
    flush(game, true);
    code != REFUSED
}

pub(in crate::game) fn ev_item_use_on(game: &mut Game<'_>, player: &NativeServerPlayer, stack: &ItemStack, pos: [i32; 3], face: [i32; 3]) -> api::Decision {
    let off_hand = game.hand == horizon_client::first_person::Hand::Off;
    let code = fire_in(game, "item_use_on", json!({ "player": player_json(player), "stack": stack_json(stack), "pos": pos, "face": face, "off_hand": off_hand }));
    flush(game, true);
    answer(code)
}

pub(in crate::game) fn ev_block_use(game: &mut Game<'_>, player: &NativeServerPlayer, click: &api::block_events::Click) -> api::block_events::Use {
    let code = fire_in(game, "block_use", json!({ "player": player_json(player), "pos": click.pos, "face": click.face, "point": click.point, "off_hand": click.off_hand }));
    flush(game, true);
    use_answer(code)
}

pub(in crate::game) fn ev_block_use_item(game: &mut Game<'_>, player: &NativeServerPlayer, stack: &ItemStack, click: &api::block_events::Click) -> api::block_events::Use {
    let code = fire_in(game, "block_use_item", json!({ "player": player_json(player), "stack": stack_json(stack), "pos": click.pos, "face": click.face, "point": click.point, "off_hand": click.off_hand }));
    flush(game, true);
    use_answer(code)
}

pub(in crate::game) fn ev_equipment_change(game: &mut Game<'_>, change: &api::server_entity_events::EquipmentChange<'_>) {
    if let Some(player) = change.player {
        fire_in(game, "equipment_change", json!({ "player": player_json(player), "slot": change.slot, "stack": change.current.map(stack_json) }));
        flush(game, true);
    }
}

pub(in crate::game) fn ev_join(game: &mut Game<'_>, player: u128, name: &str) {
    NAMES.lock().unwrap_or_else(|e| e.into_inner()).insert(player, name.to_owned());
    super::paper_fam_ess::on_join(game, player);
    fire_in(game, "join", json!({ "uuid": uuid(player), "name": name }));
    hub::LINES.lock().unwrap_or_else(|e| e.into_inner())[hub::LINE_JOIN].ready = true;
    flush(game, false);
}

pub(in crate::game) fn ev_disconnect(game: &mut Game<'_>, player: u128, name: &str) {
    fire_in(game, "disconnect", json!({ "uuid": uuid(player), "name": name }));
    super::paper_fam_ess::on_quit(game, player);
    hub::LINES.lock().unwrap_or_else(|e| e.into_inner())[hub::LINE_QUIT].ready = true;
}

pub(in crate::game) fn ev_tick(game: &mut Game<'_>) {
    super::paper_players::keep_flight(game);
    fire_packets(game);
    fire_in(game, "tick", Value::Null);
    flush(game, false);
}

/// `BlockBreakEvent`: false refuses the break.
pub(in crate::game) fn ev_break_before(game: &mut Game<'_>, player: &NativeServerPlayer, pos: [i32; 3]) -> bool {
    let code = fire_in(game, "break_before", json!({ "player": player_json(player), "pos": pos }));
    flush(game, true);
    code != REFUSED
}

pub(in crate::game) fn ev_use_block(game: &mut Game<'_>, player: &NativeServerPlayer, pos: [i32; 3], face: [i32; 3]) -> api::Decision {
    let code = fire_in(game, "use_block", json!({ "player": player_json(player), "pos": pos, "face": face }));
    flush(game, true);
    answer(code)
}

pub(in crate::game) fn ev_attack_block(game: &mut Game<'_>, player: &NativeServerPlayer, pos: [i32; 3], face: [i32; 3]) -> api::Decision {
    let code = fire_in(game, "attack_block", json!({ "player": player_json(player), "pos": pos, "face": face, "attack": true }));
    flush(game, true);
    answer(code)
}

pub(in crate::game) fn ev_use_item(game: &mut Game<'_>, player: &NativeServerPlayer, stack: &ItemStack) -> api::Decision {
    let code = fire_in(game, "use_item", json!({ "player": player_json(player), "stack": stack_json(stack) }));
    flush(game, true);
    answer(code)
}

pub(in crate::game) fn ev_use_entity(game: &mut Game<'_>, player: &NativeServerPlayer, entity: i32, kind: &str) -> api::Decision {
    let code = fire_in(game, "use_entity", json!({ "player": player_json(player), "entity": entity, "kind": kind }));
    flush(game, true);
    answer(code)
}

pub(in crate::game) fn ev_attack_entity(game: &mut Game<'_>, player: &NativeServerPlayer, entity: i32, kind: &str, pos: [f64; 3]) -> api::Decision {
    let code = fire_in(game, "attack_entity", json!({ "player": player_json(player), "entity": entity, "kind": kind, "pos": pos }));
    flush(game, true);
    answer(code)
}

/// `EntityDamageEvent` mid-hit, where the game can't be lent: false refuses it, and an amount a plugin set is
/// taken next by `take_damage`.
pub(in crate::game) fn ev_allow_damage(damage: &api::server_living_entity_events::Damage<'_>) -> bool {
    let code = fire("allow_damage", json!({ "entity": damage.entity, "kind": damage.kind, "damage_type": damage.damage_type, "amount": damage.amount }));
    if code >= 3 {
        *DAMAGE.lock().unwrap_or_else(|e| e.into_inner()) = Some((code - 3) as f32 / 1000.0);
    }
    code != REFUSED
}

pub(in crate::game) fn ev_player_death(game: &mut Game<'_>, death: &api::server_living_entity_events::Death<'_>) {
    let Some(player) = death.player else {
        return;
    };
    fire_in(game, "player_death", json!({ "uuid": uuid(player), "name": name_of(player), "cause": death.cause }));
    flush(game, false);
}

pub(in crate::game) fn ev_respawn(game: &mut Game<'_>, player: u128, name: &str) {
    fire_in(game, "respawn", json!({ "uuid": uuid(player), "name": name }));
    flush(game, false);
}

pub(in crate::game) fn ev_change_level(game: &mut Game<'_>, player: u128, name: &str, from: &str, to: &str) {
    fire_in(game, "change_level", json!({ "uuid": uuid(player), "name": name, "from": from, "to": to }));
    flush(game, false);
}

/// `AsyncPlayerChatEvent`, from the hub (which can't see this crate): false holds the game's own line back.
fn allow_chat(player: u128, name: &str, message: &str) -> bool {
    if !runtime::listening() {
        return true;
    }
    let payload = json!({ "uuid": uuid(player), "name": name, "message": message }).to_string();
    let (code, carried) = runtime::fire_carried("allow_chat", &payload);
    // What listeners made of the line goes out once, after the last of them: the game's own stays in.
    if let Some(carried) = carried.filter(|text| code == 0 && text.contains("\"chat_")) {
        runtime::fire("chat_final", &carried);
        return false;
    }
    code != ALLOWED + REFUSED
}

/// A system line about to go out, from the hub: false keeps it in.
fn allow_game_message(message: &hub::GameMessage<'_>) -> bool {
    !runtime::listens("allow_game_message")
        || fire("allow_game_message", json!({ "message": message.text, "component": super::commands::paper_entities::tag_json(message.component), "overlay": message.overlay })) != REFUSED
}

/// Who listens to `event`, for `HandlerList`.
pub(in crate::game) fn listeners_of(event: &str) -> Vec<String> {
    runtime::listeners(event)
}

/// Every event anyone listens to.
pub(in crate::game) fn listened() -> Vec<String> {
    runtime::listened()
}

/// `RegisteredListener.callEvent`.
pub(in crate::game) fn call_listener(game: &mut Game<'_>, event: &str, name: &str, payload: &str) -> (i32, Option<String>) {
    runtime::lend(game, || runtime::fire_listener(event, name, payload))
}

/// A command line without a plugin's prefix: `essentials:tp` is `tp`.
pub(in crate::game) fn unprefixed(line: &str) -> &str {
    let word = line.split(' ').next().unwrap_or_default();
    match word.split_once(':') {
        Some((prefix, _)) if matches!(prefix, "bukkit" | "paper" | "spigot") || runtime::loaded().iter().any(|(id, _)| id.eq_ignore_ascii_case(prefix)) => line.split_once(':').map_or(line, |split| split.1),
        _ => line,
    }
}

/// What a listener changed, for the listeners after it.
pub(in crate::game) fn carry(key: &str, value: &str) {
    runtime::patch(key, value);
}

/// An event from a chunk worker, with no game to lend.
pub(in crate::game) fn fire_free(event: &str, payload: &Value) -> i32 {
    runtime::fire(event, &payload.to_string())
}

/// For a family module, which doesn't see `runtime`.
pub(in crate::game) fn push_action(action: String) {
    runtime::push(action);
}

static INSTALLED: Once = Once::new();

/// A plugin is loaded: the bridge's work starts, and `paper_events` tells it every moment from now on.
fn install() {
    INSTALLED.call_once(|| {
        hub::WATCH_PACKETS.store(true, std::sync::atomic::Ordering::Relaxed);
        runtime::set_answerer(answer_query);
        super::paper_fam_ev_world::install_free();
        hub::set_chat_hooks(allow_chat, allow_game_message);
    });
}

/// Whether a plugin is loaded, so that anything hears the bridge.
pub(in crate::game) fn installed() -> bool {
    INSTALLED.is_completed()
}

/// The mods in the order the loader resolved them: the order Bukkit plugins enable in.
static LOAD_ORDER: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// The worlds are open: a plugin starting from now on enables at once.
static WORLDS_READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `core_enable_later`: a dedicated server enables its Bukkit plugins as Paper does, once its worlds are
/// loaded and the game can answer them (`enable_waiting`), not at launch, where there is no game yet.
fn enables_later() -> bool {
    // Converted mods in single player (code from `.hmod` files) enable at world open too, with the game lent.
    !WORLDS_READY.load(Ordering::Relaxed)
        && crate::game::paper_plugins::get().is_some_and(|plugins| plugins.dedicated || !plugins.loaded.is_empty())
}

/// The worlds are loaded: each Bukkit plugin that waited runs its `onEnable`, in load order, with the
/// game lent; the commands they add join the tree the players are sent.
pub(in crate::game) fn enable_waiting(game: &mut Game<'_>) {
    if WORLDS_READY.swap(true, Ordering::Relaxed) || !runtime::listens("core_enable_plugin") {
        return;
    }
    let order = LOAD_ORDER.lock().unwrap_or_else(|e| e.into_inner()).clone();
    for id in order {
        // Paper's line as each plugin's `onEnable` runs, in load order.
        if let Some(plugin) = crate::game::paper_plugins::get().and_then(|plugins| plugins.find(&id)) {
            log::info!("[{}] Enabling {}", plugin.name, plugin.full_name());
        }
        fire_in(game, "core_enable_plugin", json!({ "id": id }));
        runtime::adopt_late();
    }
    take_commands();
    if let Some(host) = game.lan.as_mut() {
        host.native_commands = None;
    }
    flush(game, false);
}

/// Called once at launch with what the loader resolved, in dependency order.
pub(in crate::game) fn start_mods(mods: &[(String, String)]) {
    // With no DLL mod the bridge stays off the events, which keeps them at no listeners: the
    // walks they guard, of block entities and tracked sets, then don't run at all.
    if !mods.is_empty() {
        install();
    }
    *LOAD_ORDER.lock().unwrap_or_else(|e| e.into_inner()) = mods.iter().map(|(id, _)| id.clone()).collect();
    for (id, manifest) in mods {
        if let Err(error) = runtime::init(id, manifest) {
            log::error!("[PluginLoader] {error}");
        }
    }
    take_commands();
    // `/plugins`, as Paper prints it: in a guest's turn a line waits in the queue for the hub.
    // Hot reload: `/hotswap NAME` (operators) swaps a plugin at the tick's end (paper_fam_core::hot_reload).
    api::register_command("hotswap", 4, Arc::new(|game, player, line| {
        let name = line.split_whitespace().nth(1).unwrap_or_default().to_owned();
        let text = if name.is_empty() {
            "Usage: /hotswap <plugin>".to_owned()
        } else {
            crate::game::paper_fam_core::hot_reload::ask(&name);
            format!("Swapping {name} at the end of this tick.")
        };
        runtime::push(json!({ "op": "tell", "player": uuid(player.uuid), "text": text }).to_string());
        flush(game, true);
    }));
    // `/plugin list|enable|disable|reload NAME` (operators): turn plugins on and off without a restart.
    api::register_command("plugin [args:text]", 4, Arc::new(|game, player, line| {
        for text in plugin_word(line, Some(player.uuid)) {
            runtime::push(json!({ "op": "tell", "player": uuid(player.uuid), "text": text }).to_string());
        }
        flush(game, true);
    }));
    // `/commands ...` (operators): which plugin answers a command several have, and commands kept off.
    api::register_command("commands [args:text]", 4, Arc::new(|game, player, line| {
        for text in super::paper_events::commands_word(line) {
            runtime::push(json!({ "op": "tell", "player": uuid(player.uuid), "text": text }).to_string());
        }
        flush(game, true);
    }));
    for word in ["plugins", "pl"] {
        api::register_command(word, 0, Arc::new(|game, player, _line| {
            for text in plugins_lines() {
                runtime::push(json!({ "op": "tell", "player": uuid(player.uuid), "text": text }).to_string());
            }
            flush(game, true);
        }));
    }
}

/// `/plugin ...` (a player's or the console's): what to answer now. Enable and disable happen at the tick's end
/// and tell `who` (None: the log) how it went.
fn plugin_word(line: &str, who: Option<u128>) -> Vec<String> {
    let mut words = line.trim_start_matches('/').split_whitespace().skip(1);
    let (action, name) = (words.next().unwrap_or("list").to_ascii_lowercase(), words.next());
    use crate::game::paper_fam_core::hot_reload;
    match (action.as_str(), name) {
        ("list" | "ls", _) => hot_reload::listing(),
        ("enable" | "on", Some(name)) => {
            hot_reload::ask_toggle(true, name, who);
            vec![]
        }
        ("disable" | "off", Some(name)) => {
            hot_reload::ask_toggle(false, name, who);
            vec![]
        }
        ("reload" | "swap", Some(name)) => {
            hot_reload::ask(name);
            vec![format!("\u{a7}aReloading {name} from its file at the end of this tick.")]
        }
        _ => vec![
            "\u{a7}f/plugin list \u{a7}7- what runs and what is disabled".to_owned(),
            "\u{a7}f/plugin disable <name> \u{a7}7- turn a plugin off (it stays off after a restart)".to_owned(),
            "\u{a7}f/plugin enable <name> \u{a7}7- turn it back on".to_owned(),
            "\u{a7}f/plugin reload <name> \u{a7}7- load it again from its file".to_owned(),
        ],
    }
}

/// A line for one player (None: the log only), sent at once: what a `/plugin` toggle did.
pub(in crate::game) fn tell_now(game: &mut Game<'_>, who: Option<u128>, text: &str) {
    match who {
        Some(player) => {
            runtime::push(json!({ "op": "tell", "player": uuid(player), "text": text }).to_string());
            flush(game, false);
        }
        None => {
            let (mut plain, mut code) = (String::new(), false);
            for c in text.chars() {
                match (code, c) {
                    (true, _) => code = false,
                    (false, '\u{a7}') => code = true,
                    _ => plain.push(c),
                }
            }
            log::info!("{plain}");
        }
    }
}

/// `/plugins` as Paper prints it, after the line naming the server fork (`engine_lines`).
fn plugins_lines() -> Vec<String> {
    let mut names: Vec<String> = runtime::loaded().into_iter().map(|(id, _)| id).collect();
    names.sort_by_key(|name| name.to_lowercase());
    let list = names.iter().map(|name| format!("\u{a7}a{name}")).collect::<Vec<_>>().join("\u{a7}f, ");
    let [fork, _] = crate::game::paper_fam_core::engine_lines();
    vec![fork, format!("\u{a7}f\u{2139} Server Plugins ({}):", names.len()), "\u{a7}9Horizon Plugins:".to_owned(), format!(" \u{a7}8- {list}")]
}

/// The engine's own words from the server console (`horizon-server cmd ...`), where there is no player to
/// tell: the lines to log, or None for any other word. `hotswap NAME` swaps at the tick's end, as a
/// player's `/hotswap` does.
pub(in crate::game) fn console_engine_word(line: &str) -> Option<Vec<String>> {
    let mut words = line.trim_start_matches('/').split_whitespace();
    let first = words.next()?;
    if api::plugin_word(first) {
        return None;
    }
    match first {
        "plugins" | "pl" => Some(plugins_lines()),
        "version" | "ver" | "about" if words.clone().next().is_none() => {
            Some(crate::game::paper_fam_core::engine_lines().to_vec())
        }
        "plugin" => {
            let lines = plugin_word(line, None);
            Some(if lines.is_empty() { vec!["Done at the end of this tick: see the log.".to_owned()] } else { lines })
        }
        "commands" => Some(super::paper_events::commands_word(line).into_iter().map(|l| crate::game::paper_fam_core::strip_section_codes(&l)).collect()),
        "hotswap" => Some(vec![match words.next() {
            Some(name) => {
                crate::game::paper_fam_core::hot_reload::ask(name);
                format!("Swapping {name} at the end of this tick.")
            }
            None => "Usage: hotswap <plugin>".to_owned(),
        }]),
        _ => None,
    }
}

/// `Bukkit.reload`: the plugins folder, read again and ordered again, with no restart and no rebuild.
pub(in crate::game) fn reload(game: &mut Game<'_>, folder: &Path) {
    install();
    runtime::close_all();
    let (found, read_errors) = crate::game::paper_plugins::read_folder(folder);
    let (ordered, order_errors) = crate::game::paper_plugins::order(found);
    for error in read_errors.iter().chain(&order_errors) {
        log::error!("[PluginLoader] {error}");
        game.native_feedback(Text::literal(error.clone()));
    }
    let mut count = 0;
    for plugin in &ordered {
        match runtime::init(&plugin.name, &plugin.manifest) {
            Ok(()) => count += 1,
            Err(error) => log::error!("[PluginLoader] {error}"),
        }
    }
    *LOAD_ORDER.lock().unwrap_or_else(|e| e.into_inner()) = ordered.iter().map(|p| p.name.clone()).collect();
    // A plugin's commands are in the tree from the first load: a new word needs a restart.
    runtime::take_actions().retain(|_| false);
    game.native_feedback(Text::literal(format!("Reloaded {count} plugins.")));
}
