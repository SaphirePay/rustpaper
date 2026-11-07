// Family core: answers to the queries its SDK file makes: levels, chunks, maps, loot, structures.
use crate::game::Game;
use crate::game::maps::{MapData, map_id};
use crate::game::native_server::NativeServerPlayer;
use crate::game::structure_blocks::load_settings;
use horizon_client::items::ItemStack;
use horizon_client::mobs::spawn_rules::SpawnLevel;
use horizon_client::mobs::spawn_selection::Category;
use horizon_world::nbt::Tag;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// Set as the server starts to stop: `Bukkit.isStopping`.
static STOPPING: AtomicBool = AtomicBool::new(false);
/// What the last pass of the server's loop was: `Server.isPaused`.
static PAUSED: AtomicBool = AtomicBool::new(false);

fn number_format_json(format: Option<&horizon_client::scoreboard::NumberFormat>) -> Value {
    use horizon_client::scoreboard::NumberFormat;
    match format {
        Some(NumberFormat::Blank) => json!({ "kind": "blank" }),
        Some(NumberFormat::Fixed(text)) => json!({ "kind": "fixed", "value": text }),
        Some(NumberFormat::Styled(style)) => json!({ "kind": "styled", "value": style }),
        None => Value::Null,
    }
}

/// A stack a plugin built: as `paper_query`'s, components as SNBT.
fn stack_from(value: &Value) -> Option<ItemStack> {
    let mut components = horizon_world::nbt::Compound::new();
    for (key, snbt) in value["components"].as_object().into_iter().flatten() {
        if let Ok((tag, _)) = horizon_world::nbt::parse_snbt(snbt.as_str()?) {
            components.insert(key.as_str().into(), tag);
        }
    }
    let tag = Tag::compound([
        ("id", Tag::String(value["item"].as_str()?.into())),
        ("count", Tag::Int(i32::try_from(value["count"].as_i64().unwrap_or(1)).unwrap_or(1))),
        ("components", Tag::Compound(components)),
    ]);
    horizon_client::save::item_from_nbt(&tag)
}

fn dimension(query: &Value) -> &str {
    query["dimension"].as_str().unwrap_or("minecraft:overworld")
}

fn position(value: &Value) -> Option<[i32; 3]> {
    Some([i32::try_from(value[0].as_i64()?).ok()?, i32::try_from(value[1].as_i64()?).ok()?, i32::try_from(value[2].as_i64()?).ok()?])
}

fn column(value: &Value) -> Option<(i32, i32)> {
    Some((i32::try_from(value[0].as_i64()?).ok()?, i32::try_from(value[1].as_i64()?).ok()?))
}

fn names(value: &Value) -> Vec<String> {
    value.as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect()
}

fn same_level(here: &str, dimension: &str) -> bool {
    here.trim_start_matches("minecraft:") == dimension.trim_start_matches("minecraft:")
}

/// Every rule by name: its value now, its default, and whether it's a number.
fn game_rules(game: &Game<'_>) -> Value {
    let rows: Map<String, Value> = game
        .ui
        .rules
        .by_name()
        .into_iter()
        .filter_map(|(name, value)| {
            let rule = horizon_client::gamerules::rule(&name)?;
            let (default, number) = match rule.default {
                horizon_client::gamerules::Value::Bool(on) => (i32::from(on), false),
                horizon_client::gamerules::Value::Int(number) => (number, true),
            };
            Some((name, json!({ "value": value, "default": default, "number": number })))
        })
        .collect();
    Value::Object(rows)
}

/// `/locate structure` from anywhere, with what it found.
fn locate_structure(game: &mut Game<'_>, query: &Value) -> Value {
    let wanted = names(&query["names"]);
    let (Some(from), radius) = (position(&query["from"]), query["radius"].as_i64().unwrap_or(100) as i32) else {
        return Value::Null;
    };
    let unexplored = query["find_unexplored"].as_bool().unwrap_or(false);
    game.native_in_level(dimension(query), |game| {
        let found = game.end.locator().and_then(|terrain| terrain.locate_for_map(&wanted, from, radius, unexplored));
        found.map_or(Value::Null, |(name, at)| json!({ "name": name, "pos": at }))
    })
    .unwrap_or(Value::Null)
}

/// `/locate biome`, with Bukkit's own radius and steps.
fn locate_biome(game: &mut Game<'_>, query: &Value) -> Value {
    let wanted = names(&query["names"]);
    let Some(from) = position(&query["from"]) else {
        return Value::Null;
    };
    let number = |key: &str, default: i32| query[key].as_i64().map_or(default, |value| value as i32);
    let (radius, horizontal, vertical) = (number("radius", 6400), number("horizontal", 32), number("vertical", 64));
    game.native_in_level(dimension(query), |game| {
        let allowed = |name: &str| wanted.iter().any(|known| known == name);
        let search = horizon_world::worldgen::generator::BiomeSearch { allowed: &allowed, origin: from, radius, horizontal, vertical, min_y: game.world.bottom_y(), max_y: game.world.top_y() - 1 };
        let found = game.end.locator().and_then(|terrain| terrain.find_closest_biome(&search));
        found.map_or(Value::Null, |(name, at)| json!({ "name": name, "pos": at }))
    })
    .unwrap_or(Value::Null)
}

/// `/place feature`, without needing a source: `World.generateTree`.
fn place_feature(game: &mut Game<'_>, query: &Value) -> Value {
    let (Some(at), Some(name)) = (position(&query["pos"]), query["feature"].as_str()) else {
        return json!(false);
    };
    let name = name.to_owned();
    json!(game
        .native_in_level(dimension(query), |game| {
            let pos = horizon_core::position::BlockPos::new(at[0], at[1], at[2]);
            let placed = game.with_level(|behaviours, level, ticks| behaviours.place_feature(level, ticks, &name, pos)).unwrap_or(false);
            game.process_level_events().is_ok() && placed
        })
        .unwrap_or(false))
}

/// Who has been sent a chunk.
fn chunk_viewers(game: &Game<'_>, dimension: &str, key: (i32, i32)) -> Value {
    let Some(host) = game.lan.as_ref() else {
        return json!([]);
    };
    let seen: Vec<String> = host.hub.guests.iter().filter(|guest| same_level(&guest.dimension, dimension) && guest.sent_chunks.contains(&key)).map(|guest| format!("{:032x}", guest.uuid)).collect();
    json!(seen)
}

/// `World.refreshChunk`: forgotten as sent, it goes out again first thing, whole.
fn refresh_chunk(game: &mut Game<'_>, dimension: &str, key: (i32, i32)) -> Value {
    let Some(host) = game.lan.as_mut() else {
        return json!(false);
    };
    let mut any = false;
    for guest in host.hub.guests.iter_mut().filter(|guest| same_level(&guest.dimension, dimension)) {
        if guest.sent_chunks.remove(&key) {
            guest.chunk_queue.insert(0, key);
            any = true;
        }
    }
    json!(any)
}

/// Facts of one chunk of a level.
fn chunk_facts(game: &mut Game<'_>, dimension: &str, key: (i32, i32)) -> Value {
    game.native_in_level(dimension, |game| {
        let loaded = game.world.chunks.contains_key(&key);
        let ticking = loaded && {
            let area = game.entity_ticking_area();
            area(key.0, key.1)
        };
        json!({ "loaded": loaded, "ticking": ticking, "forced": game.streaming.forced.contains(&key), "inhabited": game.world.inhabited.get(&key).copied().unwrap_or(0) })
    })
    .unwrap_or(Value::Null)
}

fn box_json(min: [i32; 3], max: [i32; 3]) -> Value {
    json!([min[0], min[1], min[2], max[0], max[1], max[2]])
}

/// The structures that reach a chunk, by the starts its references name.
fn chunk_structures(game: &mut Game<'_>, dimension: &str, key: (i32, i32)) -> Value {
    game.native_in_level(dimension, |game| {
        let (Some(chunk), Some(locator)) = (game.world.chunks.get(&key), game.end.locator()) else {
            return json!([]);
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut out = Vec::new();
        for (id, (x, z)) in &chunk.structure_refs {
            for start in locator.starts_at(*x, *z) {
                if *start.name != **id || !seen.insert((start.name.clone(), *x, *z)) {
                    continue;
                }
                let pieces: Vec<Value> = start.pieces.iter().map(|piece| piece.bounding_box()).map(|bounds| box_json(bounds.min, bounds.max)).collect();
                out.push(json!({ "name": start.name, "box": box_json(start.bounding_box.min, start.bounding_box.max), "pieces": pieces }));
            }
        }
        Value::Array(out)
    })
    .unwrap_or(Value::Null)
}

/// Every point of interest in loaded chunks within `radius` of `from`, nearest first.
fn points_of_interest(game: &mut Game<'_>, query: &Value) -> Value {
    let Some(from) = position(&query["from"]) else {
        return Value::Null;
    };
    let radius = query["radius"].as_i64().unwrap_or(256) as i32;
    game.native_in_level(dimension(query), |game| {
        let mut kinds: std::collections::HashMap<u32, Option<&'static str>> = std::collections::HashMap::new();
        let mut found: Vec<(i64, [i32; 3], &'static str)> = Vec::new();
        let bottom = game.world.bottom_y();
        let registry = game.pack.assets.registry;
        for cx in (from[0] - radius) >> 4..=(from[0] + radius) >> 4 {
            for cz in (from[2] - radius) >> 4..=(from[2] + radius) >> 4 {
                let Some(chunk) = game.world.chunks.get(&(cx, cz)) else {
                    continue;
                };
                for (i, section) in chunk.sections.iter().enumerate() {
                    let mut wanted = |state: u32| {
                        kinds
                            .entry(state)
                            .or_insert_with(|| registry.state(horizon_world::registry::BlockStateId(state)).and_then(horizon_client::save::poi_type).map(|(kind, _)| kind))
                            .is_some()
                    };
                    if !section.blocks.maybe_has(&mut wanted) {
                        continue;
                    }
                    let base = bottom + 16 * i as i32;
                    for j in 0..4096 {
                        let Some(kind) = kinds.get(&section.get(j)).copied().flatten() else {
                            continue;
                        };
                        let at = [cx * 16 + (j & 15) as i32, base + (j >> 8) as i32, cz * 16 + ((j >> 4) & 15) as i32];
                        let distance = (0..3).map(|axis| i64::from(at[axis] - from[axis]).pow(2)).sum::<i64>();
                        if distance <= i64::from(radius) * i64::from(radius) {
                            found.push((distance, at, kind));
                        }
                    }
                }
            }
        }
        found.sort_by_key(|row| row.0);
        Value::Array(found.into_iter().map(|(_, at, kind)| json!({ "pos": at, "kind": kind })).collect())
    })
    .unwrap_or(Value::Null)
}

/// Whether a chunk is in its level's region file.
fn chunk_saved(game: &Game<'_>, dimension: &str, key: (i32, i32)) -> Value {
    let id = format!("minecraft:{}", dimension.trim_start_matches("minecraft:"));
    let folder = horizon_client::save::dimension_dir(&game.world_root, &id);
    let file = folder.join(format!("region/r.{}.{}.mca", key.0 >> 5, key.1 >> 5));
    json!(horizon_world::region::RegionFile::open(&file).is_ok_and(|region| region.chunks().any(|(chunk, _)| chunk.x == key.0 && chunk.z == key.1)))
}

// ---- Maps ----

fn map_json(id: i32, map: &MapData, colors: bool) -> Value {
    let mut value = json!({ "id": id, "center": map.center, "scale": map.scale, "dimension": map.dimension, "locked": map.locked, "tracking": map.tracking, "unlimited": map.unlimited });
    if colors {
        value["colors"] = json!(map.colors);
        value["decorations"] = map.decorations.iter().map(|mark| json!({ "type": mark.sprite, "x": mark.x, "y": mark.y, "direction": mark.rot })).collect();
    }
    value
}

/// Maps to save again and to send whole to who holds them, the next time `core_map_tick` runs:
/// what marks that is private to `maps`.
static MAPS_TOUCHED: Mutex<BTreeSet<i32>> = Mutex::new(BTreeSet::new());

pub(in crate::game) fn take_touched_maps() -> BTreeSet<i32> {
    std::mem::take(&mut *MAPS_TOUCHED.lock().unwrap_or_else(|e| e.into_inner()))
}

/// A map's fields changed by a plugin, on the map itself: its frames and who holds it stay.
fn map_change(game: &mut Game<'_>, query: &Value) -> Value {
    let Some(id) = query["id"].as_i64().and_then(|id| i32::try_from(id).ok()) else {
        return Value::Null;
    };
    let Some(map) = game.map(id) else {
        return Value::Null;
    };
    let number = |key: &str| query[key].as_i64().and_then(|value| i32::try_from(value).ok());
    if let Some(scale) = number("scale") {
        map.scale = scale.clamp(0, 4) as u8;
    }
    map.center = [number("center_x").unwrap_or(map.center[0]), number("center_z").unwrap_or(map.center[1])];
    if let Some(dimension) = query["dimension"].as_str() {
        map.dimension = dimension.to_owned();
    }
    map.locked = query["locked"].as_bool().unwrap_or(map.locked);
    map.tracking = query["tracking"].as_bool().unwrap_or(map.tracking);
    map.unlimited = query["unlimited"].as_bool().unwrap_or(map.unlimited);
    let answer = map_json(id, map, false);
    MAPS_TOUCHED.lock().unwrap_or_else(|e| e.into_inner()).insert(id);
    answer
}

/// What a plugin's renderer drew, to one player: their own view of the map.
fn map_send(game: &mut Game<'_>, query: &Value) -> Value {
    use horizon_protocol::packets::play::{self, Clientbound as C};
    let (Some(uuid), Some(id)) = (query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok()), query["id"].as_i64().and_then(|id| i32::try_from(id).ok())) else {
        return json!(false);
    };
    let colors: Vec<u8> = query["colors"].as_array().into_iter().flatten().filter_map(|color| color.as_u64().map(|color| color as u8)).collect();
    let Some((scale, locked)) = game.map(id).map(|map| (map.scale, map.locked)) else {
        return json!(false);
    };
    let Some(host) = game.lan.as_ref() else {
        return json!(false);
    };
    let Some(guest) = host.hub.guests.iter().find(|guest| guest.uuid == uuid) else {
        return json!(false);
    };
    let decorations = query["cursors"].as_array().map(|cursors| {
        cursors
            .iter()
            .filter_map(|cursor| {
                let kind = host.hub.registry_id("minecraft:map_decoration_type", cursor["type"].as_str()?)?;
                let name = cursor.get("caption").filter(|caption| !caption.is_null()).map(|caption| horizon_client::lan::data::component_nbt(&horizon_client_render::packs::component(caption).component()));
                Some(play::MapDecoration { kind, x: cursor["x"].as_i64()? as i8, y: cursor["y"].as_i64()? as i8, rot: cursor["direction"].as_i64()? as i8, name })
            })
            .collect()
    });
    let patch = (colors.len() == 128 * 128).then_some(play::MapPatch { start_x: 0, start_y: 0, width: 128, height: 128, colors });
    guest.send(&C::MapItemData(play::MapItemData { map_id: id, scale: scale as i8, locked, decorations, patch }));
    json!(true)
}

/// Who carries a map, or sees it in a frame of the level the frames are of.
fn map_holders(game: &Game<'_>, id: i32) -> Value {
    let Some(host) = game.lan.as_ref() else {
        return json!([]);
    };
    let framed = game.item_frames.iter().any(|frame| frame.item.as_ref().and_then(map_id) == Some(id));
    let holders: Vec<String> = host
        .hub
        .guests
        .iter()
        .filter(|guest| guest.inventory.iter().chain(std::iter::once(&guest.offhand)).flatten().filter_map(map_id).any(|held| held == id) || (framed && host.hub.here(guest)))
        .map(|guest| format!("{:032x}", guest.uuid))
        .collect();
    json!(holders)
}

// ---- Structures ----

/// `Structure.fill`: the area's blocks and entities, into a template the game keeps in memory.
fn structure_fill(game: &mut Game<'_>, query: &Value) -> Value {
    let (Some(name), Some(origin), Some(size)) = (query["template"].as_str().map(str::to_owned), position(&query["origin"]), position(&query["size"])) else {
        return json!(false);
    };
    let entities = query["entities"].as_bool().unwrap_or(true);
    json!(game.native_in_level(dimension(query), |game| game.save_structure_area(&name, origin, size, !entities, "?", false, false)).unwrap_or(false))
}

thread_local! {
    /// Entities a planned placement would spawn, kept whole: a plugin sees them as text.
    static ENTITY_PLAN: std::cell::RefCell<EntityPlan> = const { std::cell::RefCell::new(EntityPlan::Off) };
    /// `/execute ... run` of a plugin's word: the words, and who ran it, in JSON.
    static PROXIED: std::cell::RefCell<Option<(String, Value)>> = const { std::cell::RefCell::new(None) };
}

enum EntityPlan {
    Off,
    Asking(Vec<horizon_world::nbt::Compound>),
    Answered(Vec<Value>),
}

/// What `place_template_entry` spawns of what a template holds.
pub(in crate::game) fn planned_entities(entities: Vec<horizon_world::nbt::Compound>) -> Vec<horizon_world::nbt::Compound> {
    ENTITY_PLAN.with(|plan| match &mut *plan.borrow_mut() {
        EntityPlan::Off => entities,
        EntityPlan::Asking(kept) => {
            *kept = entities;
            Vec::new()
        }
        EntityPlan::Answered(verdicts) => entities
            .into_iter()
            .zip(verdicts.iter())
            .filter_map(|(entity, verdict)| match verdict {
                Value::Null => None,
                Value::String(snbt) => match horizon_world::nbt::parse_snbt(snbt) {
                    Ok((Tag::Compound(changed), _)) => Some(changed),
                    _ => Some(entity),
                },
                _ => Some(entity),
            })
            .collect(),
    })
}

/// What a template would do, with nothing placed.
fn structure_plan(game: &mut Game<'_>, query: &Value) -> Value {
    use horizon_world::worldgen::structure::processor::{TEMPLATE_PLAN, TemplatePlan};
    TEMPLATE_PLAN.with(|plan| *plan.borrow_mut() = TemplatePlan::Asking(Vec::new()));
    ENTITY_PLAN.with(|plan| *plan.borrow_mut() = EntityPlan::Asking(Vec::new()));
    let ran = structure_place_plain(game, query);
    let blocks = match TEMPLATE_PLAN.with(|plan| std::mem::take(&mut *plan.borrow_mut())) {
        TemplatePlan::Asking(blocks) => blocks,
        _ => Vec::new(),
    };
    let entities = match ENTITY_PLAN.with(|plan| std::mem::replace(&mut *plan.borrow_mut(), EntityPlan::Off)) {
        EntityPlan::Asking(entities) => entities,
        _ => Vec::new(),
    };
    if ran != json!(true) {
        return Value::Null;
    }
    let rows: Vec<Value> = blocks.iter().map(|(pos, state)| json!([pos[0], pos[1], pos[2], crate::game::paper_fam_ev_world::state_text(game, *state)])).collect();
    let others: Vec<Value> = entities
        .iter()
        .map(|entity| {
            let data = crate::game::commands::paper_entities::tag_json(&Tag::Compound(entity.clone()));
            let at = [0, 1, 2].map(|side| data["Pos"][side].as_f64().unwrap_or(0.0).floor() as i32);
            json!({ "pos": at, "data": data })
        })
        .collect();
    json!({ "blocks": rows, "entities": others })
}

/// What transformers have been shown so far of a template: the level, where, and what's really there.
static PREVIEW: Mutex<Vec<(String, [i32; 3], horizon_world::registry::BlockStateId)>> = Mutex::new(Vec::new());

/// Back to the world as it was, newest first.
fn undo_preview(game: &mut Game<'_>) {
    let shown = std::mem::take(&mut *PREVIEW.lock().unwrap_or_else(|e| e.into_inner()));
    for (level, at, state) in shown.into_iter().rev() {
        game.native_in_level(&level, |game| game.world.set_quietly(at[0], at[1], at[2], state));
    }
}

/// A template placed: with what a plugin made of its plan, if it has one.
fn structure_place(game: &mut Game<'_>, query: &Value) -> Value {
    use horizon_world::worldgen::structure::processor::{TEMPLATE_PLAN, TemplatePlan};
    undo_preview(game);
    let Some(changes) = query["changes"].as_array() else {
        return structure_place_plain(game, query);
    };
    let registry = game.pack.assets.registry;
    let answers = changes
        .iter()
        .filter_map(|row| {
            let at = [row[0].as_i64()? as i32, row[1].as_i64()? as i32, row[2].as_i64()? as i32];
            Some((at, crate::game::render_control::parse_state(registry, row[3].as_str()?).ok()))
        })
        .collect();
    TEMPLATE_PLAN.with(|plan| *plan.borrow_mut() = TemplatePlan::Answered(answers));
    let verdicts = query["verdicts"].as_array().cloned().unwrap_or_default();
    ENTITY_PLAN.with(|plan| *plan.borrow_mut() = EntityPlan::Answered(verdicts));
    let ran = structure_place_plain(game, query);
    TEMPLATE_PLAN.with(|plan| *plan.borrow_mut() = TemplatePlan::Off);
    ENTITY_PLAN.with(|plan| *plan.borrow_mut() = EntityPlan::Off);
    ran
}

thread_local! {
    static GAME_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Who the line RCON is running came from, while it runs.
static REMOTE_PEER: Mutex<Option<String>> = Mutex::new(None);

/// A control request's line: `remote <address> <line>` says who, which `rcon.rs` knows.
pub(in crate::game) fn remote_line(remote: bool, line: &str) -> &str {
    let (peer, rest) = if remote { line.split_once(' ').map_or((None, line), |(peer, rest)| (Some(peer), rest)) } else { (None, line) };
    *REMOTE_PEER.lock().unwrap_or_else(|e| e.into_inner()) = peer.filter(|peer| *peer != "-").map(str::to_owned);
    rest
}

/// Text without its `§` colour codes, for the log.
pub(in crate::game) fn strip_section_codes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{a7}' {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}

/// A console or RCON line for a word a plugin took: the plugin hears it, with who sent it. The
/// game's tree hasn't got those words, so it can't get there any other way.
pub(in crate::game) fn console_word(game: &mut Game<'_>, line: &str) -> bool {
    let word = line.split(' ').next().unwrap_or_default();
    // The engine's own words, from the console too (a player's run through its command tree).
    if let Some(lines) = crate::game::paper_bridge::console_engine_word(line) {
        for text in lines {
            log::info!("{}", strip_section_codes(&text));
        }
        return true;
    }
    if !game.lan.as_ref().and_then(|host| host.native_commands.as_ref()).is_some_and(|words| words.contains_key(word)) {
        return false;
    }
    let remote = REMOTE_PEER.lock().unwrap_or_else(|e| e.into_inner()).clone();
    crate::game::paper_bridge::on_trigger(game, "command", json!({ "line": line, "player": Value::Null, "remote": remote }));
    true
}

/// `f` with the hub in `game.lan`, where a command reaches every player. None where this frame's
/// callers hold it themselves and haven't lent it: a player's packet, outside its turn.
pub(in crate::game) fn with_hub<'g, R>(game: &mut Game<'g>, f: impl FnOnce(&mut Game<'g>) -> R) -> Option<R> {
    if game.lan.is_some() {
        return Some(f(game));
    }
    horizon_mod_runtime::take_lent(|host: crate::game::lan::LanHost| {
        game.lan = Some(host);
        let result = f(game);
        (game.lan.take().expect("a command puts the hub back"), result)
    })
}

/// A line of the SDK's own, now: whatever a plugin took, this is the game's command. None if the
/// hub is out of reach here.
pub(in crate::game) fn run_at_once(game: &mut Game<'_>, line: &str) -> Option<bool> {
    with_hub(game, |game| {
        let held = GAME_ONLY.replace(true);
        let ran = game.console_run(line);
        GAME_ONLY.set(held);
        ran
    })
}

/// A line of the SDK's own: run at once, or as soon as the hub is back.
pub(in crate::game) fn run_game_only(game: &mut Game<'_>, line: &str) -> bool {
    run_at_once(game, line).unwrap_or_else(|| {
        wait_with_line(None, line);
        true
    })
}

/// Lines the hub was out of reach for, and who for: `None` is the console. They run as soon as it's
/// back, which is the end of the packet, where a player's own commands run.
static WAITING_LINES: Mutex<Vec<(Option<u128>, String)>> = Mutex::new(Vec::new());

/// Work that needs the hub.
type HubWork = Box<dyn for<'a, 'b> FnOnce(&'a mut Game<'b>)>;

thread_local! {
    static HUB_WORK: std::cell::RefCell<Vec<HubWork>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// `work`, as soon as the hub is back: it runs with the waiting lines.
pub(in crate::game) fn when_hub_back(work: HubWork) {
    HUB_WORK.with(|all| all.borrow_mut().push(work));
    LINES_WAIT.store(true, Ordering::Relaxed);
}

static LINES_WAIT: AtomicBool = AtomicBool::new(false);

fn wait_with_line(player: Option<u128>, line: &str) {
    WAITING_LINES.lock().unwrap_or_else(|e| e.into_inner()).push((player, line.to_owned()));
    LINES_WAIT.store(true, Ordering::Relaxed);
}

pub(in crate::game) fn lines_waiting() -> bool {
    LINES_WAIT.load(Ordering::Relaxed)
}

pub(in crate::game) fn run_waiting_lines(game: &mut Game<'_>) {
    LINES_WAIT.store(false, Ordering::Relaxed);
    for work in HUB_WORK.with(|all| std::mem::take(&mut *all.borrow_mut())) {
        work(game);
    }
    let lines = std::mem::take(&mut *WAITING_LINES.lock().unwrap_or_else(|e| e.into_inner()));
    for (player, line) in lines {
        match player {
            Some(player) => {
                player_line(game, player, &line);
            }
            None => {
                run_game_only(game, &line);
            }
        }
    }
}

/// `Player.performCommand`, now: what they'd run, as they'd run it, without
/// `PlayerCommandPreprocessEvent`, which is only for what a client sends. None if the hub is out of reach.
fn player_line(game: &mut Game<'_>, player: u128, line: &str) -> Option<bool> {
    with_hub(game, |game| {
        if game.lan_acting_uuid == Some(player) {
            // Their own turn: they're in place already, and can't be swapped in again.
            let (conn, permission) = (game.lan_acting?, game.lan_acting_permission?);
            let level = std::mem::replace(&mut game.ui.registries.permission, permission);
            let (vanilla_only, line) = match line.strip_prefix("minecraft:") {
                Some(rest) => (true, rest),
                None => (false, crate::game::paper_bridge::unprefixed(line)),
            };
            let ran = (!vanilla_only && game.perform_native_command(conn, permission, line)) || game.run_command(line);
            game.ui.registries.permission = level;
            return Some(ran);
        }
        let host = game.lan.as_ref()?;
        let guest = host.hub.guests.iter().find(|guest| guest.uuid == player)?;
        let (conn, permission) = (guest.conn, guest.permission);
        Some(game.run_guest_command(crate::game::lan_actions::GuestCommand { conn, command: line.to_owned(), permission, integrated: false, quiet: false }))
    })
    .map(|ran| ran.unwrap_or(false))
}

/// `/execute ... run <word>` with a word a plugin owns: the game's tree hasn't got it, so the line
/// goes through with a word it has, and `proxied_run` runs the plugin's for each source.
pub(in crate::game) fn proxied_line(game: &mut Game<'_>, command: &str) -> Option<bool> {
    let line = command.trim_start_matches('/');
    if !GAME_ONLY.get() && horizon_client::lan::commands::GAME_WORDS_OFF.load(Ordering::Relaxed) && !matches!(line.split(' ').next(), Some("timings" | "bukkit:timings" | "bukkit:reload" | "bukkit:rl")) {
        let word = line.split(' ').next().unwrap_or_default();
        if !game.lan.as_ref().and_then(|host| host.native_commands.as_ref()).is_some_and(|words| words.contains_key(word)) {
            game.system_message(horizon_client::text::Text::translate("command.unknown.command", &[]));
            return Some(false);
        }
    }
    if GAME_ONLY.get() || !line.starts_with("execute ") || PROXIED.with(|held| held.borrow().is_some()) {
        return None;
    }
    let host = game.lan.as_ref()?;
    let natives = host.native_commands.as_ref()?;
    let cut = line.match_indices(" run ").map(|(at, _)| at).filter(|at| line[at + 5..].split(' ').next().is_some_and(|word| natives.contains_key(word))).last()?;
    let caller = game.lan_acting.and_then(|conn| host.hub.guest(conn)).map_or(Value::Null, |guest| json!(format!("{:032x}", guest.uuid)));
    PROXIED.with(|held| *held.borrow_mut() = Some((line[cut + 5..].to_owned(), caller)));
    let ran = game.run_command(&format!("{} run seed", &line[..cut]));
    PROXIED.with(|held| *held.borrow_mut() = None);
    Some(ran)
}

/// One source of that line: the plugin's command, as whoever the source is.
pub(in crate::game) fn proxied_run(game: &mut Game<'_>) -> Option<i32> {
    use crate::game::commands::Target;
    let (line, caller) = PROXIED.with(|held| held.borrow().clone())?;
    let source = game.command_source?;
    let host = game.lan.as_ref();
    let guest = |conn: u32| host.and_then(|host| host.hub.guest(conn)).map(|guest| format!("{:032x}", guest.uuid));
    let (player, entity) = match source.entity {
        Some(Target::Player) => (game.lan_acting.and_then(guest), None),
        Some(Target::Guest(conn)) => (guest(conn), None),
        Some(Target::Mob(id) | Target::Item(id) | Target::Orb(id) | Target::Projectile(id) | Target::Stand(id) | Target::Frame(id) | Target::Painting(id) | Target::Boat(id) | Target::Cart(id) | Target::Other(id) | Target::Crystal(id) | Target::Dragon(id)) => (None, Some(id)),
        None => (None, None),
    };
    let payload = json!({ "line": line, "player": Value::Null, "proxied": { "caller": caller, "player": player, "entity": entity, "dimension": source.dimension.id() } });
    // Out while it runs: every `/execute` comes here, and the plugin's own would run the line again, for ever.
    PROXIED.with(|held| *held.borrow_mut() = None);
    crate::game::paper_bridge::on_trigger(game, "command", payload);
    PROXIED.with(|held| *held.borrow_mut() = Some((line, caller)));
    Some(1)
}

/// `Structure.place`.
fn structure_place_plain(game: &mut Game<'_>, query: &Value) -> Value {
    use horizon_world::worldgen::structure::{Mirror, Rotation};
    let (Some(name), Some(at)) = (query["template"].as_str(), position(&query["pos"])) else {
        return json!(false);
    };
    let Some(entry) = game.template_entry(name) else {
        return json!(false);
    };
    let rotation = query["rotation"].as_str().and_then(Rotation::from_name).unwrap_or(Rotation::None);
    let mirror = match query["mirror"].as_str() {
        Some("LEFT_RIGHT") => Mirror::LeftRight,
        Some("FRONT_BACK") => Mirror::FrontBack,
        _ => Mirror::None,
    };
    let integrity = query["integrity"].as_f64().unwrap_or(1.0) as f32;
    let mut settings = load_settings(mirror, rotation, integrity, query["seed"].as_i64().unwrap_or(0), false);
    settings.ignore_entities = !query["entities"].as_bool().unwrap_or(true);
    let pos = horizon_core::position::BlockPos::new(at[0], at[1], at[2]);
    json!(game.native_in_level(dimension(query), |game| game.place_template_entry(&entry, pos, settings, mirror, rotation)).unwrap_or(false))
}

// ---- Data ----

fn walk_data(root: &std::path::Path, prefix: &str, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(root).into_iter().flatten().flatten() {
        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if path.is_dir() {
            walk_data(&path, &format!("{prefix}{stem}/"), out);
        } else if path.extension().is_some_and(|extension| extension == "json") {
            out.push(format!("minecraft:{prefix}{stem}"));
        }
    }
}

/// Every entry of a registry folder, its sub-folders too: advancements, loot tables.
fn data_tree(game: &Game<'_>, path: &str) -> Value {
    if path.contains("..") {
        return Value::Null;
    }
    let mut out = Vec::new();
    walk_data(&game.options.resources.join(format!("data/minecraft/{path}")), "", &mut out);
    out.sort();
    json!(out)
}

/// Bukkit's recipes over the crafting grid: the nine slots, row by row.
fn craft(game: &Game<'_>, query: &Value) -> Value {
    let grid: Vec<Option<horizon_client::items::ItemStack>> = query["grid"].as_array().into_iter().flatten().map(stack_from).collect();
    if grid.len() != 9 && grid.len() != 4 {
        return Value::Null;
    }
    let size = if grid.len() == 9 { 3 } else { 2 };
    game.res.recipes().craft(&grid, size).map_or(Value::Null, |stack| crate::game::paper_query::stack_json(&stack))
}

/// Light and biomes of a loaded chunk, for a `ChunkSnapshot` that has them.
fn chunk_light(game: &mut Game<'_>, dimension: &str, key: (i32, i32)) -> Value {
    game.native_in_level(dimension, |game| {
        let Some(chunk) = game.world.chunks.get(&key).cloned() else {
            return Value::Null;
        };
        let (bottom, top) = (game.world.bottom_y(), game.world.top_y());
        let biomes = game.res.biomes();
        let mut names = Vec::new();
        let mut climate = Vec::new();
        for y in (bottom..top).step_by(4) {
            for z in (0..16).step_by(4) {
                for x in (0..16).step_by(4) {
                    let biome = &biomes.list[chunk.biome(key.0 * 16 + x, y, key.1 * 16 + z) as usize];
                    names.push(biome.name.clone());
                    climate.push(biome.temperature);
                }
            }
        }
        let view = game.mob_world_base(&[]);
        let mut sky = Vec::with_capacity(((top - bottom) * 256) as usize);
        let mut block = Vec::with_capacity(((top - bottom) * 256) as usize);
        for y in bottom..top {
            for z in 0..16 {
                for x in 0..16 {
                    sky.push(view.sky_light(key.0 * 16 + x, y, key.1 * 16 + z));
                    block.push(view.block_light(key.0 * 16 + x, y, key.1 * 16 + z));
                }
            }
        }
        json!({ "bottom": bottom, "biomes": names, "temperature": climate, "sky": sky, "block": block })
    })
    .unwrap_or(Value::Null)
}

/// A level's raids, as the game saves them: it saves what changed first.
fn raids(game: &mut Game<'_>, dimension: &str) -> Value {
    game.save_raids();
    let id = format!("minecraft:{}", dimension.trim_start_matches("minecraft:"));
    let file = horizon_client::save::dimension_dir(&game.world_root, &id).join("data/minecraft/raids.dat");
    horizon_client::save::read_dat(&file).and_then(|tag| tag.get("data").map(crate::game::commands::paper_entities::tag_json)).unwrap_or(Value::Null)
}

// ---- The server's own loop ----

struct TickLog {
    began: Option<std::time::Instant>,
    rows: VecDeque<(std::time::Instant, u64)>,
    count: u64,
}

static TICKS: Mutex<TickLog> = Mutex::new(TickLog { began: None, rows: VecDeque::new(), count: 0 });
/// Paper's three windows: as many ticks as fit in the longest, at 20 a second.
const TICKS_KEPT: usize = 15 * 60 * 20;

pub(in crate::game) fn tick_began() {
    TICKS.lock().unwrap_or_else(|e| e.into_inner()).began = Some(std::time::Instant::now());
}

/// Which plugins hold which chunk: level, x, z.
static TICKETS: Mutex<BTreeMap<(String, i32, i32), BTreeSet<String>>> = Mutex::new(BTreeMap::new());

/// The chunks of a level that plugins hold. They load and tick as forced chunks do, without being
/// `/forceload`'s: nothing lists or saves them, as Java's tickets end with the server.
pub(in crate::game) fn ticket_chunks(level: &str) -> Vec<(i32, i32)> {
    let level = level.trim_start_matches("minecraft:");
    let tickets = TICKETS.lock().unwrap_or_else(|e| e.into_inner());
    tickets.keys().filter(|(held, _, _)| held.trim_start_matches("minecraft:") == level).map(|(_, x, z)| (*x, *z)).collect()
}

/// What `core_recipes_clear` waits for: the game's tables can't change under an event.
static RECIPES_STALE: AtomicBool = AtomicBool::new(false);

/// `/reload` read the game's recipes again, as Java's does; the SDK's own reloads leave a clear as it is.
pub(in crate::game) fn real_reload() {
    if !GAME_ONLY.get() {
        *horizon_world::loot::CLEARED_RECIPES.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

/// `World.setSpawnFlags`: monsters, and everything else, by level.
static SPAWN_FLAGS: Mutex<BTreeMap<String, (bool, bool)>> = Mutex::new(BTreeMap::new());
/// `World.setSpawnLimit`: by level and category.
static LEVEL_LIMITS: Mutex<BTreeMap<(String, String), i32>> = Mutex::new(BTreeMap::new());

fn own_spawn_file(game: &Game<'_>, dimension: &str) -> std::path::PathBuf {
    horizon_client::save::dimension_dir(&game.world_root, dimension).join("data/paper/level_overrides.dat")
}

/// A level's own spawn: the game's, for the respawn world; else the file Paper keeps it in.
fn own_spawn(game: &Game<'_>, dimension: &str) -> Option<([i32; 3], f32, f32)> {
    if dimension == horizon_client::save::spawn_dimension() {
        let [yaw, pitch] = game.level.world_spawn_angle;
        return Some((game.level.world_spawn?, yaw, pitch));
    }
    let tag = horizon_client::save::read_dat(&own_spawn_file(game, dimension))?;
    let spawn = crate::game::commands::paper_entities::tag_json(&tag)["data"]["spawn"].clone();
    Some((position(&spawn["pos"])?, spawn["yaw"].as_f64().unwrap_or(0.0) as f32, spawn["pitch"].as_f64().unwrap_or(0.0) as f32))
}

fn save_own_spawn(game: &Game<'_>, dimension: &str, at: [i32; 3], yaw: f32, pitch: f32) {
    let file = own_spawn_file(game, dimension);
    let spawn = Tag::compound([("pos", Tag::IntArray(at.to_vec())), ("yaw", Tag::Float(yaw.to_bits())), ("pitch", Tag::Float(pitch.to_bits()))]);
    let mut root = horizon_client::save::read_dat(&file).unwrap_or_else(|| Tag::compound([("DataVersion", Tag::Int(horizon_world::DATA_VERSION))]));
    if let Tag::Compound(top) = &mut root {
        match top.get_mut(&"data".into()) {
            Some(Tag::Compound(data)) => {
                data.insert("spawn".into(), spawn);
            }
            _ => {
                top.insert("data".into(), Tag::compound([("spawn", spawn)]));
            }
        }
    }
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(error) = horizon_client::save::write_dat(&file, root) {
        log::error!("[paper_fam_core] couldn't save the spawn of {dimension}: {error}");
    }
}

/// Whose permission manager is in force: `PluginManager.overridePermissionManager`.
static PERMISSION_OWNER: Mutex<Option<String>> = Mutex::new(None);

/// A library is coming in through `loadPlugin`: it stops short of `on_enable`.
static LOADING_LATE: AtomicBool = AtomicBool::new(false);
/// What a plugin answered `core_command_call` with.
static REPLY: Mutex<Option<Value>> = Mutex::new(None);
const LOAD_FOLDER: &str = "plugins/.load";

/// `open_folder` opens all a folder holds, so each library gets a folder of its own.
/// A library, started as far as `on_load`: the runtime lets one in during an event. What it's
/// called, or nothing if it wouldn't load.
fn load_library(path: &std::path::Path, index: u128) -> Option<String> {
    load_library_described(path, index).map(|plugin| plugin.name)
}

/// `load_library`, with what the library says of itself.
fn load_library_described(path: &std::path::Path, index: u128) -> Option<crate::game::paper_plugins::Description> {
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |time| time.as_millis());
    let folder = std::path::Path::new(LOAD_FOLDER).join(format!("{stamp}-{index}"));
    let copied = std::fs::create_dir_all(&folder).and_then(|()| std::fs::copy(path, folder.join(path.file_name().unwrap_or_default())));
    if let Err(error) = copied {
        log::error!("[paper_fam_core] couldn't load {}: {error}", path.display());
        return None;
    }
    let (manifests, errors) = horizon_mod_runtime::open_folder(&folder);
    for error in errors {
        log::error!("[paper_fam_core] {error}");
    }
    // What the code that asked has queued stays queued: only what the library declares is taken.
    let waiting = horizon_mod_runtime::take_actions();
    let mut name = None;
    for (manifest, file) in manifests {
        let plugin = match crate::game::paper_plugins::Description::parse(&manifest, &file) {
            Ok(plugin) => plugin,
            Err(error) => {
                log::error!("[PluginLoader] {error}");
                continue;
            }
        };
        match horizon_mod_runtime::init(&plugin.name, &manifest) {
            Ok(()) => name = Some(plugin.clone()),
            Err(error) => log::error!("[paper_fam_core] {error}"),
        }
    }
    // What it declared while it started: words for the game's tree.
    crate::game::paper_bridge::take_commands();
    TREE_STALE.store(true, Ordering::Relaxed);
    for action in waiting {
        horizon_mod_runtime::push(action);
    }
    name
}

/// A library opened by `horizon_mod_runtime::preload` (hot reload, off the server thread) starts as
/// `load_library_described` starts one it opened itself.
fn start_preloaded(library: horizon_mod_runtime::Preloaded, file: &str) -> Option<crate::game::paper_plugins::Description> {
    let manifest = library.manifest().to_owned();
    let plugin = match crate::game::paper_plugins::Description::parse(&manifest, file) {
        Ok(plugin) => plugin,
        Err(error) => {
            log::error!("[PluginLoader] {error}");
            return None;
        }
    };
    horizon_mod_runtime::adopt(library);
    let waiting = horizon_mod_runtime::take_actions();
    let started = match horizon_mod_runtime::init(&plugin.name, &manifest) {
        Ok(()) => Some(plugin),
        Err(error) => {
            log::error!("[paper_fam_core] {error}");
            None
        }
    };
    crate::game::paper_bridge::take_commands();
    TREE_STALE.store(true, Ordering::Relaxed);
    for action in waiting {
        horizon_mod_runtime::push(action);
    }
    started
}

type VoidRule = (bool, f32, f64);
const VOID_DEFAULT: VoidRule = (true, 4.0, -64.0);
static VOID_RULES: Mutex<BTreeMap<String, VoidRule>> = Mutex::new(BTreeMap::new());

fn void_rule_of(dimension: &str) -> VoidRule {
    VOID_RULES.lock().unwrap_or_else(|e| e.into_inner()).get(dimension).copied().unwrap_or(VOID_DEFAULT)
}

/// As a level's entities or a guest tick: that level's rule is the one in force.
pub(in crate::game) fn void_rule_here(dimension: &str) -> i32 {
    *horizon_client::motion::VOID_RULE.lock().unwrap_or_else(|e| e.into_inner()) = void_rule_of(dimension);
    0
}

static SIGN_LINES: Mutex<Option<([i32; 3], bool, Vec<String>)>> = Mutex::new(None);

/// What the listeners of a `SignChangeEvent` wrote on this sign, once the game writes it.
pub(in crate::game) fn take_sign_lines(pos: [i32; 3], front: bool) -> Option<Vec<String>> {
    let mut held = SIGN_LINES.lock().unwrap_or_else(|e| e.into_inner());
    if held.as_ref().is_some_and(|(at, side, _)| *at == pos && *side == front) {
        return held.take().map(|(.., lines)| lines);
    }
    None
}

/// `DamageSource` contexts by the entity hurt, with the tick they were dealt.
static DAMAGE_CONTEXTS: Mutex<BTreeMap<String, (i64, Value)>> = Mutex::new(BTreeMap::new());

fn uuid_words(uuid: u128) -> [i32; 4] {
    [(uuid >> 96) as u32 as i32, (uuid >> 64) as u32 as i32, (uuid >> 32) as u32 as i32, uuid as u32 as i32]
}

type BarChange = (String, [i32; 4], bool);
/// Changes to bars a plugin made this tick: `/bossbar add` waits for the tick's end like any command.
static BAR_CHANGES: Mutex<Vec<(i64, BarChange)>> = Mutex::new(Vec::new());

fn change_bar(game: &mut Game<'_>, (id, words, add): &BarChange) -> bool {
    let Some(bar) = game.bosses.bars.get_mut(id) else {
        return false;
    };
    if !*add {
        bar.players.retain(|known| known != words);
    } else if !bar.players.contains(words) {
        bar.players.push(*words);
    }
    true
}

/// `Bukkit.reload`, for the end of the tick: no listener can be running when libraries go.
static RELOAD_ALL: AtomicBool = AtomicBool::new(false);

fn reload_all(game: &mut Game<'_>) {
    let names: Vec<String> = SHARED.lock().unwrap_or_else(|e| e.into_inner()).plugins.keys().cloned().collect();
    for name in names {
        crate::game::paper_bridge::fire_custom(game, "core_plugin_disable", json!({ "name": name }));
    }
    if let Some(folder) = crate::game::paper_plugins::get().map(crate::game::paper_plugins::Plugins::folder) {
        crate::game::paper_bridge::reload(game, &folder);
    }
    game.reload_world_tables();
}

type Explosion = (String, [f64; 3], f32, bool, bool);
/// Explosions a plugin asked for: blocks and entities can't change under an event.
static EXPLOSIONS: Mutex<Vec<Explosion>> = Mutex::new(Vec::new());

/// Whether the last explosion got past its listeners.
static EXPLOSION_WENT: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn explosion_went() {
    EXPLOSION_WENT.store(true, Ordering::Relaxed);
}

/// One explosion, and whether a listener let it: `Explosion.wasCanceled`.
fn explode(game: &mut Game<'_>, (level, at, power, fire, breaks): Explosion) -> bool {
    use crate::game::explosion::BlockInteraction;
    let blocks = if breaks { BlockInteraction::Destroy } else { BlockInteraction::Keep };
    EXPLOSION_WENT.store(false, Ordering::Relaxed);
    game.native_in_level(&level, |game| {
        if let Err(error) = game.server_explode(at, power, blocks, fire) {
            log::error!("[paper_fam_core] an explosion failed: {error}");
        }
    });
    EXPLOSION_WENT.load(Ordering::Relaxed)
}

fn settle_explosions(game: &mut Game<'_>) {
    let waiting = std::mem::take(&mut *EXPLOSIONS.lock().unwrap_or_else(|e| e.into_inner()));
    for order in waiting {
        explode(game, order);
    }
}

/// What `World.setViewDistance` and its like said, by level without `minecraft:` and by which:
/// `view`, `send` or `simulation`.
static LEVEL_DISTANCES: Mutex<BTreeMap<(String, &'static str), i32>> = Mutex::new(BTreeMap::new());
/// Whether any was ever set: until then every player's are the server's.
static DISTANCES_SET: AtomicBool = AtomicBool::new(false);
/// What each connection was last told: its chunk radius and its simulation distance.
static TOLD_DISTANCES: Mutex<BTreeMap<u32, (i32, i32)>> = Mutex::new(BTreeMap::new());

/// A level's distance, or `otherwise` where it has none of its own.
pub(in crate::game) fn level_distance(level: &str, which: &'static str, otherwise: i32) -> i32 {
    if !DISTANCES_SET.load(Ordering::Relaxed) {
        return otherwise;
    }
    let key = (level.trim_start_matches("minecraft:").to_owned(), which);
    LEVEL_DISTANCES.lock().unwrap_or_else(|e| e.into_inner()).get(&key).copied().unwrap_or(otherwise)
}

/// Every player's distances are their level's, as `RegionizedPlayerChunkLoader` has them: however
/// they came to be off, by changing level or by telling the server their own view again.
fn settle_distances(game: &mut Game<'_>) {
    use horizon_protocol::packets::play::{self, Clientbound as C};
    if !DISTANCES_SET.load(Ordering::Relaxed) {
        return;
    }
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    let (server_view, server_simulation) = (host.hub.config.view_distance, host.hub.config.simulation_distance);
    let mut told = TOLD_DISTANCES.lock().unwrap_or_else(|e| e.into_inner());
    told.retain(|conn, _| host.hub.guest(*conn).is_some());
    for index in 0..host.hub.guests.len() {
        let level = host.hub.guests[index].dimension.clone();
        let view = level_distance(&level, "view", server_view);
        let send = level_distance(&level, "send", view).min(view);
        let simulation = level_distance(&level, "simulation", server_simulation);
        let radius = host.hub.radius_for(send);
        let first = (host.hub.radius_for(server_view), server_simulation);
        let guest = &mut host.hub.guests[index];
        let told = told.entry(guest.conn).or_insert(first);
        let wanted = i32::from(guest.info.view_distance).clamp(2, send);
        if guest.view_distance != wanted {
            guest.view_distance = wanted;
            guest.center = None;
        }
        if told.0 != radius {
            told.0 = radius;
            guest.send(&C::SetChunkCacheRadius(play::Distance { distance: radius }));
        }
        if told.1 != simulation {
            told.1 = simulation;
            guest.send(&C::SetSimulationDistance(play::Distance { distance: simulation }));
        }
    }
}

/// Raids whose bar a plugin asked for, by id: the level, and who was near last tick.
static RAID_BARS: Mutex<BTreeMap<i32, (String, Vec<[i32; 4]>)>> = Mutex::new(BTreeMap::new());
/// Whether the bars an earlier run made have gone: the file keeps every bar.
static RAID_BARS_SWEPT: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn raid_bar_owned(id: i32) -> bool {
    RAID_BARS.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&id)
}

fn raid_bar_name(id: i32) -> String {
    format!("horizon_bukkit:raid_{id}")
}

/// `Raid.getBossBar`: the raid's bar, made a bar of the game's the first time a plugin asks for it.
/// From then on `settle_raid_bars` keeps its title, progress and who sees it, as the raid's own
/// `ServerBossEvent`; the rest is the plugin's.
fn own_raid_bar(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let id = query["id"].as_i64()? as i32;
    let level = dimension(query).to_owned();
    game.native_in_level(&level, |game| {
        game.raiding.raids.get(id)?;
        let bar = raid_bar_name(id);
        if game.bosses.bars.get(&bar).is_none() {
            game.bosses.bars.add(&bar, json!({ "translate": "event.minecraft.raid" }));
            let made = game.bosses.bars.get_mut(&bar)?;
            made.color = "red".to_owned();
            made.style = "notched_10".to_owned();
            made.max = 1_000_000;
        }
        RAID_BARS.lock().unwrap_or_else(|e| e.into_inner()).entry(id).or_insert_with(|| (level.clone(), Vec::new()));
        Some(json!(bar))
    })
    .flatten()
}

fn settle_raid_bars(game: &mut Game<'_>) {
    if !RAID_BARS_SWEPT.swap(true, Ordering::Relaxed) {
        let stale: Vec<String> = game.bosses.bars.ids().into_iter().filter(|id| id.starts_with("horizon_bukkit:raid_")).collect();
        for id in stale {
            game.bosses.bars.remove(&id);
        }
    }
    let owned: Vec<(i32, String, Vec<[i32; 4]>)> = RAID_BARS.lock().unwrap_or_else(|e| e.into_inner()).iter().map(|(id, (level, seen))| (*id, level.clone(), seen.clone())).collect();
    for (id, level, seen) in owned {
        let near = game
            .native_in_level(&level, |game| {
                let bar = raid_bar_name(id);
                if game.raiding.raids.get(id).is_none_or(|raid| raid.is_stopped()) {
                    game.bosses.bars.remove(&bar);
                    return None;
                }
                let viewers = game.boss_viewers();
                let rows: Vec<([i32; 4], Tag, f32)> = viewers.iter().filter(|viewer| viewer.here).filter_map(|viewer| game.raid_boss_at(viewer.at, viewer.alive).filter(|row| row.0 == id).map(|row| (viewer.ints, row.1, row.2))).collect();
                let near: Vec<[i32; 4]> = rows.iter().map(|row| row.0).collect();
                let made = game.bosses.bars.get_mut(&bar)?;
                if let Some((_, name, progress)) = rows.first() {
                    made.name = crate::game::commands::paper_entities::tag_json(name);
                    made.value = (progress * made.max as f32) as i32;
                }
                // Who came near and who left. A player the plugin added or took off itself stays as it left them.
                for viewer in near.iter().filter(|viewer| !seen.contains(viewer)) {
                    if !made.players.contains(viewer) {
                        made.players.push(*viewer);
                    }
                }
                for gone in seen.iter().filter(|viewer| !near.contains(viewer)) {
                    made.players.retain(|player| player != gone);
                }
                Some(near)
            })
            .flatten();
        let mut all = RAID_BARS.lock().unwrap_or_else(|e| e.into_inner());
        match near {
            Some(near) => {
                if let Some(row) = all.get_mut(&id) {
                    row.1 = near;
                }
            }
            None => {
                all.remove(&id);
            }
        }
    }
}

/// `DragonBattle.getBossBar`'s bar, once a plugin has asked for it, and who was near last tick.
static DRAGON_BAR: Mutex<Option<Vec<[i32; 4]>>> = Mutex::new(None);
const DRAGON_BAR_NAME: &str = "horizon_bukkit:dragon_battle";

pub(in crate::game) fn dragon_bar_owned() -> bool {
    DRAGON_BAR.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}

/// The battle's bar: from then on the game goes on setting its progress and who sees it, dragon or
/// no dragon, and the rest is the plugin's.
fn own_dragon_bar(game: &mut Game<'_>) -> Option<Value> {
    if game.bosses.bars.get(DRAGON_BAR_NAME).is_none() {
        game.bosses.bars.add(DRAGON_BAR_NAME, json!({ "translate": "entity.minecraft.ender_dragon" }));
        let made = game.bosses.bars.get_mut(DRAGON_BAR_NAME)?;
        made.color = "pink".to_owned();
        made.style = "progress".to_owned();
        made.max = 1_000_000;
        made.music = true;
        made.fog = true;
    }
    DRAGON_BAR.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default();
    Some(json!(DRAGON_BAR_NAME))
}

fn settle_dragon_bar(game: &mut Game<'_>) {
    let Some(seen) = DRAGON_BAR.lock().unwrap_or_else(|e| e.into_inner()).clone() else {
        return;
    };
    let near = game.native_in_level("minecraft:the_end", |game| {
        let viewers = game.boss_viewers();
        let near: Vec<[i32; 4]> = viewers.iter().filter(|viewer| viewer.here && game.dragon_boss_progress(viewer.at, viewer.alive).is_some()).map(|viewer| viewer.ints).collect();
        let progress = game.dragon.as_ref().map(|dragon| (dragon.health / 200.0).clamp(0.0, 1.0));
        if let Some(made) = game.bosses.bars.get_mut(DRAGON_BAR_NAME) {
            if let Some(progress) = progress {
                made.value = (progress * made.max as f32) as i32;
            }
            for viewer in near.iter().filter(|viewer| !seen.contains(viewer)) {
                if !made.players.contains(viewer) {
                    made.players.push(*viewer);
                }
            }
            for gone in seen.iter().filter(|viewer| !near.contains(viewer)) {
                made.players.retain(|player| player != gone);
            }
        }
        near
    });
    if let Some(near) = near {
        *DRAGON_BAR.lock().unwrap_or_else(|e| e.into_inner()) = Some(near);
    }
}

/// Who's looking at a board a plugin made, and so not at the game's.
static OWN_BOARDS: Mutex<BTreeSet<u128>> = Mutex::new(BTreeSet::new());

pub(in crate::game) fn on_own_board(player: u128) -> bool {
    OWN_BOARDS.lock().unwrap_or_else(|e| e.into_inner()).contains(&player)
}

/// `teamkill.*` and `killedByTeam.*` as they fall: how many there have been, and the last few.
static TEAM_KILLS: Mutex<(u64, VecDeque<(String, String)>)> = Mutex::new((0, VecDeque::new()));

pub(in crate::game) fn team_kill_score(criteria: &str, owner: &str) {
    let mut log = TEAM_KILLS.lock().unwrap_or_else(|e| e.into_inner());
    log.0 += 1;
    log.1.push_back((criteria.to_owned(), owner.to_owned()));
    if log.1.len() > 256 {
        log.1.pop_front();
    }
}

/// `settings.allow-end` of `bukkit.yml`, read as the server starts, as levels are made then.
pub(in crate::game) fn allow_end() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| setting("bukkit.yml", "allow-end:"))
}

/// `misc.enable-nether` of `config/paper-global.yml`, as `allow_end`.
pub(in crate::game) fn allow_nether() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| setting("config/paper-global.yml", "enable-nether:"))
}

/// A yes-or-no in a settings file, on unless it says otherwise, as both default.
fn setting(file: &str, key: &str) -> bool {
    let text = std::fs::read_to_string(file).unwrap_or_default();
    text.lines().find_map(|line| line.trim().strip_prefix(key)).is_none_or(|value| value.trim() != "false")
}

/// Whether `dimension` is a level this server was told not to have.
pub(in crate::game) fn level_off(dimension: &str) -> bool {
    match dimension.trim_start_matches("minecraft:") {
        "the_end" => !allow_end(),
        "the_nether" => !allow_nether(),
        _ => false,
    }
}

/// Levels `World.setAutoSave` took off, without `minecraft:`.
static NO_SAVE: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
/// `/save-off`: every level, as `SaveOffCommand` sets `noSave` on each.
static NO_SAVE_ANYWHERE: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn no_save(level: &str) -> bool {
    NO_SAVE_ANYWHERE.load(Ordering::Relaxed) || NO_SAVE.lock().unwrap_or_else(|e| e.into_inner()).contains(level.trim_start_matches("minecraft:"))
}

pub(in crate::game) fn no_save_anywhere(off: bool) {
    NO_SAVE_ANYWHERE.store(off, Ordering::Relaxed);
    if !off {
        NO_SAVE.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
}

/// What each client said it listens on: `Player.getListeningPluginChannels`.
static CLIENT_CHANNELS: Mutex<BTreeMap<u128, BTreeSet<String>>> = Mutex::new(BTreeMap::new());

pub(in crate::game) fn note_channels(host: &crate::game::lan::LanHost, conn: u32, packet: &horizon_protocol::packets::play::Serverbound) {
    use horizon_protocol::packets::play::Serverbound as S;
    let S::CustomPayload(payload) = packet else {
        return;
    };
    let adding = match payload.channel.as_str() {
        "minecraft:register" => true,
        "minecraft:unregister" => false,
        _ => return,
    };
    let Some(guest) = host.hub.guest(conn) else {
        return;
    };
    let mut all = CLIENT_CHANNELS.lock().unwrap_or_else(|e| e.into_inner());
    heard_configuring(&mut all, guest.uuid);
    let known = all.entry(guest.uuid).or_default();
    for name in payload.data.split(|byte| *byte == 0).filter(|name| !name.is_empty()) {
        let name = String::from_utf8_lossy(name).into_owned();
        if adding {
            known.insert(name);
        } else {
            known.remove(&name);
        }
    }
}

/// What a client said before it was in the game, in with the rest.
fn heard_configuring(all: &mut BTreeMap<u128, BTreeSet<String>>, uuid: u128) {
    let said = horizon_client::lan::conn::CONFIG_CHANNELS.lock().unwrap_or_else(|e| e.into_inner()).remove(&uuid);
    let known = all.entry(uuid).or_default();
    for (adding, name) in said.into_iter().flatten() {
        if adding {
            known.insert(name);
        } else {
            known.remove(&name);
        }
    }
}

/// A client's channels go with it.
fn settle_channels(game: &mut Game<'_>) {
    if let Some(host) = game.lan.as_ref() {
        CLIENT_CHANNELS.lock().unwrap_or_else(|e| e.into_inner()).retain(|uuid, _| host.hub.guests.iter().any(|guest| guest.uuid == *uuid));
    }
}

/// A player's file, whoever they are.
fn player_path(game: &Game<'_>, uuid: u128) -> std::path::PathBuf {
    game.world_root.join(format!("players/data/{}.dat", horizon_protocol::crypto::uuid_string(uuid)))
}

/// A plugin added a word since the tree went out.
static TREE_STALE: AtomicBool = AtomicBool::new(false);

fn settle_commands(game: &mut Game<'_>) {
    if TREE_STALE.load(Ordering::Relaxed) && let Some(host) = game.lan.as_mut() {
        TREE_STALE.store(false, Ordering::Relaxed);
        // `register_native_commands`, at the head of the next `tick_lan`, builds it again.
        host.native_commands = None;
    }
}

static SILENT_BOLTS: Mutex<Vec<[f64; 3]>> = Mutex::new(Vec::new());

/// Whether a plugin asked for this bolt, at the moment it would thunder, to make no sound.
pub(in crate::game) fn silent_bolt(at: [f64; 3]) -> bool {
    let mut bolts = SILENT_BOLTS.lock().unwrap_or_else(|e| e.into_inner());
    let found = bolts.iter().position(|bolt| (0..3).all(|side| (bolt[side] - at[side]).abs() < 0.01));
    found.map(|index| bolts.swap_remove(index)).is_some()
}

fn settle_bars(game: &mut Game<'_>) {
    let waiting = std::mem::take(&mut *BAR_CHANGES.lock().unwrap_or_else(|e| e.into_inner()));
    let now = game.world.game_time;
    let kept: Vec<(i64, BarChange)> = waiting.into_iter().filter(|(at, change)| !change_bar(game, change) && now - *at < 2).collect();
    BAR_CHANGES.lock().unwrap_or_else(|e| e.into_inner()).extend(kept);
}

fn tick_ended() {
    let mut log = TICKS.lock().unwrap_or_else(|e| e.into_inner());
    let Some(began) = log.began.take() else {
        return;
    };
    let now = std::time::Instant::now();
    log.rows.push_back((now, now.duration_since(began).as_nanos() as u64));
    log.count += 1;
    if log.rows.len() > TICKS_KEPT {
        log.rows.pop_front();
    }
}

/// Ticks a second over a window of the last `seconds`: all of them, while the server is younger.
fn tps_over(log: &TickLog, seconds: f64) -> f64 {
    let Some((last, _)) = log.rows.back() else {
        return 20.0;
    };
    let in_window: Vec<&(std::time::Instant, u64)> = log.rows.iter().rev().take_while(|(at, _)| last.duration_since(*at).as_secs_f64() < seconds).collect();
    let spent = in_window.last().map_or(0.0, |(first, _)| last.duration_since(*first).as_secs_f64());
    // Paper's averages start full of 20s (`RollingAverage` is filled at construction): a window the server
    // hasn't run for yet counts as 20 ticks a second, so one slow tick at the start doesn't read as 0.1.
    let ticks = in_window.len().saturating_sub(1) as f64;
    if spent >= seconds { ticks / spent } else { (ticks + 20.0 * (seconds - spent)) / seconds }
}

/// Average, least and most milliseconds of the ticks of the last `seconds`.
fn mspt_over(log: &TickLog, seconds: f64) -> [f64; 3] {
    let Some((last, _)) = log.rows.back() else {
        return [0.0; 3];
    };
    let (mut total, mut least, mut most, mut count) = (0_u128, u64::MAX, 0_u64, 0_u64);
    for (_, nanos) in log.rows.iter().rev().take_while(|(at, _)| last.duration_since(*at).as_secs_f64() < seconds) {
        total += u128::from(*nanos);
        least = least.min(*nanos);
        most = most.max(*nanos);
        count += 1;
    }
    if count == 0 {
        return [0.0; 3];
    }
    [total as f64 / count as f64 / 1e6, least as f64 / 1e6, most as f64 / 1e6]
}

fn tick_stats() -> Value {
    let log = TICKS.lock().unwrap_or_else(|e| e.into_inner());
    let times: Vec<u64> = log.rows.iter().rev().take(100).map(|(_, nanos)| *nanos).collect();
    json!({ "times": times, "tps": [tps_over(&log, 60.0), tps_over(&log, 300.0), tps_over(&log, 900.0)], "count": log.count })
}

/// A line to whoever ran a command, in their turn or not.
fn say(game: &mut Game<'_>, player: u128, lines: &[String]) {
    for text in lines {
        horizon_mod_runtime::push(json!({ "op": "tell", "player": format!("{player:032x}"), "text": text }).to_string());
    }
    crate::game::paper_bridge::on_trigger(game, "core_said", json!({}));
}

fn tps_text(tps: f64) -> String {
    let color = if tps > 18.0 {
        "\u{a7}a"
    } else if tps > 16.0 {
        "\u{a7}e"
    } else {
        "\u{a7}c"
    };
    format!("{color}{}{}", if tps > 21.0 { "*" } else { "" }, ((tps * 100.0).round() / 100.0).min(20.0))
}

/// `/tps`, as Paper prints it.
fn tps_command(game: &mut Game<'_>, player: &NativeServerPlayer, _line: &str) {
    let tps = {
        let log = TICKS.lock().unwrap_or_else(|e| e.into_inner());
        [tps_over(&log, 60.0), tps_over(&log, 300.0), tps_over(&log, 900.0)]
    };
    let list = tps.map(tps_text).join("\u{a7}7, ");
    say(game, player.uuid, &[format!("\u{a7}6TPS from last 1m, 5m, 15m: {list}")]);
}

fn mspt_text(ms: f64) -> String {
    let color = if ms < 40.0 {
        "\u{a7}a"
    } else if ms < 50.0 {
        "\u{a7}e"
    } else {
        "\u{a7}c"
    };
    format!("{color}{:.1}", ms)
}

/// `/mspt`, as Paper prints it.
fn mspt_command(game: &mut Game<'_>, player: &NativeServerPlayer, _line: &str) {
    let rows = {
        let log = TICKS.lock().unwrap_or_else(|e| e.into_inner());
        [mspt_over(&log, 5.0), mspt_over(&log, 10.0), mspt_over(&log, 60.0)]
    };
    let cell = |row: [f64; 3]| row.map(mspt_text).join("\u{a7}7/");
    say(game, player.uuid, &["\u{a7}6Server tick times \u{a7}e(\u{a7}7avg\u{a7}e/\u{a7}7min\u{a7}e/\u{a7}7max\u{a7}e)\u{a7}6 from last 5s\u{a7}7,\u{a7}6 10s\u{a7}7,\u{a7}6 1m\u{a7}e:".to_owned(), format!("\u{a7}6\u{25f4} {}\u{a7}7, {}\u{a7}7, {}", cell(rows[0]), cell(rows[1]), cell(rows[2]))]);
}

/// `/version`, `/ver`, `/about`: the server, or with words each plugin they name, as Bukkit prints them.
/// The server fork, as `/version`, `/plugins` and the console name it: Horizon's Paper engine (Paper's plugin
/// API in Rust), its version and build (the engine's commit and the day, set by tools/build_engine.py), the
/// Minecraft version, the Bukkit/Paper API it implements and whether plugins hot reload.
pub(in crate::game) fn engine_lines() -> [String; 2] {
    let minecraft = horizon_protocol::GAME_VERSION;
    let version = env!("CARGO_PKG_VERSION");
    let commit = option_env!("HORIZON_ENGINE_COMMIT").unwrap_or("local");
    let built = option_env!("HORIZON_ENGINE_BUILT").unwrap_or("unknown day");
    let horizon = option_env!("HORIZON_SOURCE_COMMIT").unwrap_or("?");
    let hot = if hot_reload::on() { "on" } else { "off" };
    [
        format!("\u{a7}fThis server is running \u{a7}bHorizon Paper engine\u{a7}f {version} (Paper API in Rust, build \u{a7}a{commit}\u{a7}f, {built}, Horizon {horizon})"),
        format!("\u{a7}fMinecraft {minecraft}, implementing API version {minecraft}-R0.1-SNAPSHOT; plugin hot reload \u{a7}a{hot}"),
    ]
}

fn version_command(game: &mut Game<'_>, player: &NativeServerPlayer, line: &str) {
    let wanted = line.split_whitespace().skip(1).collect::<Vec<_>>().join(" ").to_ascii_lowercase();
    if wanted.is_empty() {
        say(game, player.uuid, &engine_lines());
        return;
    }
    let rows: Vec<Value> = SHARED.lock().unwrap_or_else(|e| e.into_inner()).plugins.iter().filter(|(name, _)| name.contains(&wanted)).map(|(_, row)| row.clone()).collect();
    if rows.is_empty() {
        say(game, player.uuid, &["\u{a7}fThis server is not running any plugin by that name.".to_owned(), "\u{a7}fUse /plugins to get a list of plugins.".to_owned()]);
        return;
    }
    let mut lines = Vec::new();
    for row in rows {
        let yaml = &row["yaml"];
        let text = |key: &str| yaml[key].as_str().map(str::to_owned).or_else(|| yaml[key].as_f64().map(|value| value.to_string()));
        lines.push(format!("\u{a7}a{} \u{a7}fversion \u{a7}a{}", row["name"].as_str().unwrap_or_default(), text("version").unwrap_or_default()));
        if let Some(description) = text("description") {
            lines.push(format!("\u{a7}f{description}"));
        }
        if let Some(website) = text("website") {
            lines.push(format!("\u{a7}fWebsite: \u{a7}a{website}"));
        }
        let authors: Vec<String> = yaml["authors"].as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).chain(text("author")).collect();
        match authors.as_slice() {
            [] => {}
            [one] => lines.push(format!("\u{a7}fAuthor: \u{a7}a{one}")),
            many => lines.push(format!("\u{a7}fAuthors: \u{a7}a{}", many.join("\u{a7}f, \u{a7}a"))),
        }
    }
    say(game, player.uuid, &lines);
}

/// `/version` is a word of the game's, and a mod's command of a name the game has is left out of
/// the tree, so a player's goes on to `/ver`, as Paper's answers it, and to everyone, as Paper's.
pub(in crate::game) fn version_line(host: &mut crate::game::lan::LanHost, conn: u32, command: &str) -> bool {
    let line = command.trim_start_matches('/');
    let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
    if word != "version" {
        return false;
    }
    host.commands.push(crate::game::lan_actions::GuestCommand { conn, command: format!("ver {rest}").trim_end().to_owned(), permission: 0, integrated: false, quiet: false });
    true
}

/// The core family's work at the end of each tick, once the server has started.
pub(in crate::game) fn on_tick_end(game: &mut Game<'_>) {
    tick_ended();
    settle_bars(game);
    settle_commands(game);
    settle_explosions(game);
    if lines_waiting() {
        run_waiting_lines(game);
    }
    settle_distances(game);
    settle_dragon_bar(game);
    settle_raid_bars(game);
    undo_preview(game);
    settle_channels(game);
    if RELOAD_ALL.swap(false, Ordering::Relaxed) {
        reload_all(game);
    }
    hot_reload::tick(game);
    if RECIPES_STALE.swap(false, Ordering::Relaxed) {
        game.reload_world_tables();
    }
}

/// `Bukkit.isStopping` from here on.
pub(in crate::game) fn on_stopping() {
    STOPPING.store(true, Ordering::Relaxed);
}

static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the server's start has reached `start`.
pub(in crate::game) fn started() -> bool {
    STARTED.load(Ordering::Relaxed)
}

/// What only runs if the game's own start reaches it: `paper_events::startup`.
pub(in crate::game) fn start() {
    use crate::game::paper_events::register_command;
    STARTED.store(true, Ordering::Relaxed);
    // What an earlier run loaded is held by nothing now.
    let _ = std::fs::remove_dir_all(LOAD_FOLDER);
    for word in ["ver", "about"] {
        register_command(&format!("{word} [plugin:text]"), 0, std::sync::Arc::new(version_command));
    }
    register_command("tps", 2, std::sync::Arc::new(tps_command));
    register_command("mspt", 2, std::sync::Arc::new(mspt_command));
}

static PAUSE_SECONDS: AtomicI32 = AtomicI32::new(-1);
static PAUSE_HELD: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
static IDLE_MINUTES: AtomicI32 = AtomicI32::new(-1);
static EMPTY_TICKS: AtomicI32 = AtomicI32::new(0);
static IN_PASS: AtomicBool = AtomicBool::new(false);

/// The head of each pass of the server's loop: what the loop keeps in its own locals. True if this
/// call ran the pass the way a plugin wants it, so the loop's own is skipped.
pub(in crate::game) fn server_pass(game: &mut Game<'_>, paused: bool) -> bool {
    if IN_PASS.load(Ordering::Relaxed) {
        return false;
    }
    restore_spawn_limits();
    settle_dispensed(game);
    send_links(game);
    let empty = game.lan.as_ref().is_none_or(|host| host.hub.guests.is_empty());
    if !empty {
        EMPTY_TICKS.store(0, Ordering::Relaxed);
    }
    let ticks = if empty { EMPTY_TICKS.fetch_add(1, Ordering::Relaxed) + 1 } else { 0 };
    let seconds = PAUSE_SECONDS.load(Ordering::Relaxed);
    let mut wanted = if seconds >= 0 { seconds > 0 && ticks >= seconds.saturating_mul(20) } else { paused };
    if !PAUSE_HELD.lock().unwrap_or_else(|e| e.into_inner()).is_empty() {
        wanted = false;
    }
    PAUSED.store(wanted, Ordering::Relaxed);
    let minutes = IDLE_MINUTES.load(Ordering::Relaxed);
    if minutes > 0 && let Some(host) = game.lan.as_mut() {
        let limit = minutes as u64 * 1200;
        let idle: Vec<u32> = host.hub.guests.iter().filter(|guest| host.hub.ticks.saturating_sub(guest.last_action_tick) > limit).map(|guest| guest.conn).collect();
        for conn in idle {
            host.hub.kick(conn, horizon_protocol::text::translatable("multiplayer.disconnect.idling", &[]));
        }
    }
    if wanted == paused {
        return false;
    }
    IN_PASS.store(true, Ordering::Relaxed);
    if let Err(error) = game.tick_server(wanted) {
        log::error!("[paper_fam_core] a server tick failed: {error}");
    }
    IN_PASS.store(false, Ordering::Relaxed);
    true
}

/// Whether a plugin took the idle kick over, so the loop's own stands down.
pub(in crate::game) fn idle_overridden() -> bool {
    IDLE_MINUTES.load(Ordering::Relaxed) >= 0
}

// ---- Spawn timing ----

static TICKS_PER: Mutex<BTreeMap<String, i64>> = Mutex::new(BTreeMap::new());
static HELD_LIMITS: Mutex<Vec<(String, i32)>> = Mutex::new(Vec::new());
static IN_SPAWN: AtomicBool = AtomicBool::new(false);

/// The caps a spawn pass held at zero go back to what they were.
fn restore_spawn_limits() {
    let held = std::mem::take(&mut *HELD_LIMITS.lock().unwrap_or_else(|e| e.into_inner()));
    for (key, limit) in held {
        horizon_client::paper_hub::set_spawn_limit(&key, limit);
    }
}

fn hold_spawn_limit(category: Category) {
    hold_spawn_limit_at(category, 0);
}

fn hold_spawn_limit_at(category: Category, limit: i32) {
    let key = format!("{category:?}");
    let mut held = HELD_LIMITS.lock().unwrap_or_else(|e| e.into_inner());
    if !held.iter().any(|(known, _)| *known == key) {
        held.push((key.clone(), category.max()));
    }
    horizon_client::paper_hub::set_spawn_limit(&key, limit);
}

/// `World.setTicksPerSpawns`, at the head of `spawn_natural`: a category that isn't due has no room
/// this pass. Animals are the game's one kind on a timer of its own (every 400 ticks), so to come
/// more often than that they get a pass of their own, with the clock on a multiple of 400.
pub(in crate::game) fn spawn_pass(game: &mut Game<'_>) -> i32 {
    if IN_SPAWN.load(Ordering::Relaxed) {
        return 0;
    }
    restore_spawn_limits();
    let here = game.dim.id();
    for ((level, name), limit) in LEVEL_LIMITS.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        if level == here {
            hold_spawn_limit_at(Category::from_name(name), *limit);
        }
    }
    // `ChunkMap`'s split: monsters on one side, everything else on the other.
    if let Some((monsters, animals)) = SPAWN_FLAGS.lock().unwrap_or_else(|e| e.into_inner()).get(here).copied() {
        for category in Category::SPAWNING {
            if if category == Category::Monster { !monsters } else { !animals } {
                hold_spawn_limit(category);
            }
        }
    }
    let rows = TICKS_PER.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if rows.is_empty() {
        return 0;
    }
    let now = game.world.game_time;
    let due = |category: Category| rows.get(category.name()).is_none_or(|ticks| *ticks > 0 && now.rem_euclid(*ticks) == 0);
    if rows.contains_key(Category::Creature.name()) && due(Category::Creature) && now.rem_euclid(400) != 0 {
        for category in Category::SPAWNING.into_iter().filter(|category| *category != Category::Creature) {
            hold_spawn_limit(category);
        }
        let inhabited = game.world.inhabited.clone();
        game.world.game_time = now - now.rem_euclid(400);
        IN_SPAWN.store(true, Ordering::Relaxed);
        game.spawn_natural();
        IN_SPAWN.store(false, Ordering::Relaxed);
        game.world.game_time = now;
        game.world.inhabited = inhabited;
        restore_spawn_limits();
    }
    for category in Category::SPAWNING {
        if rows.contains_key(category.name()) && !due(category) {
            hold_spawn_limit(category);
        }
    }
    0
}

// ---- Raids and the dragon: what only code beside them can reach ----

static RAID_WAVES: Mutex<Vec<(i32, i32)>> = Mutex::new(Vec::new());
static DRAGON_ORDERS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static PORTAL_ORDER: Mutex<Option<bool>> = Mutex::new(None);
static DRAGON_NOTE: Mutex<Option<Value>> = Mutex::new(None);

/// What `core_raid_tick` applies to the raids.
pub(in crate::game) fn take_raid_waves() -> Vec<(i32, i32)> {
    std::mem::take(&mut *RAID_WAVES.lock().unwrap_or_else(|e| e.into_inner()))
}

pub(in crate::game) fn take_dragon_orders() -> Vec<String> {
    std::mem::take(&mut *DRAGON_ORDERS.lock().unwrap_or_else(|e| e.into_inner()))
}

pub(in crate::game) fn take_portal_order() -> Option<bool> {
    PORTAL_ORDER.lock().unwrap_or_else(|e| e.into_inner()).take()
}

pub(in crate::game) fn note_dragon(gateways: usize, crystals: Vec<[f64; 3]>) {
    *DRAGON_NOTE.lock().unwrap_or_else(|e| e.into_inner()) = Some(json!({ "gateways_left": gateways, "respawn_crystals": crystals }));
}

// ---- Steps of a frozen game ----

static STEP: Mutex<(u64, i64)> = Mutex::new((0, 0));

/// `/tick step`, as it starts: `ServerTickManager.getFrozenTicksToRun` counts down from it.
pub(in crate::game) fn note_step(id: &str, time: i32) -> i32 {
    if id == "tick.step" {
        let count = TICKS.lock().unwrap_or_else(|e| e.into_inner()).count;
        *STEP.lock().unwrap_or_else(|e| e.into_inner()) = (count, i64::from(time));
    }
    0
}

// ---- What a dispenser shot ----

static DISPENSED: Mutex<Vec<(u32, [i32; 3], String)>> = Mutex::new(Vec::new());
static SHOT_BY: Mutex<BTreeMap<u32, ([i32; 3], String)>> = Mutex::new(BTreeMap::new());

/// As a dispenser fires: every entity made from here to the next pass came out of it.
pub(in crate::game) fn note_dispense(game: &Game<'_>, pos: [i32; 3]) -> i32 {
    DISPENSED.lock().unwrap_or_else(|e| e.into_inner()).push((game.next_entity, pos, game.dim.id().to_owned()));
    0
}

fn settle_dispensed(game: &Game<'_>) {
    let pending = std::mem::take(&mut *DISPENSED.lock().unwrap_or_else(|e| e.into_inner()));
    if pending.is_empty() {
        return;
    }
    let mut shot = SHOT_BY.lock().unwrap_or_else(|e| e.into_inner());
    for (index, (from, pos, dimension)) in pending.iter().enumerate() {
        let to = pending.get(index + 1).map_or(game.next_entity, |next| next.0);
        for projectile in game.creatures.projectiles.iter().filter(|projectile| projectile.id > *from && projectile.id <= to) {
            shot.insert(projectile.id, (*pos, dimension.clone()));
        }
    }
    while shot.len() > 1024 {
        shot.pop_first();
    }
}

/// A live property of the server: what plugins changed since it read `server.properties`.
fn live_property(game: &Game<'_>, key: &str) -> Value {
    let Some(host) = game.lan.as_ref() else {
        return Value::Null;
    };
    match key {
        "max-players" => json!(host.hub.lan.shared.max_players().to_string()),
        "motd" => json!(host.hub.lan.shared.motd()),
        "view-distance" => json!(host.hub.config.view_distance.to_string()),
        "simulation-distance" => json!(host.hub.config.simulation_distance.to_string()),
        "spawn-protection" => json!(host.hub.spawn_protection.to_string()),
        "player-idle-timeout" => match IDLE_MINUTES.load(Ordering::Relaxed) {
            minutes if minutes >= 0 => json!(minutes.to_string()),
            _ => Value::Null,
        },
        "pause-when-empty-seconds" => match PAUSE_SECONDS.load(Ordering::Relaxed) {
            seconds if seconds >= 0 => json!(seconds.to_string()),
            _ => Value::Null,
        },
        "enforce-whitelist" => game.server_lists.as_ref().map_or(Value::Null, |lists| json!(lists.lock().unwrap_or_else(|e| e.into_inner()).enforce_whitelist.to_string())),
        "force-gamemode" => json!(host.hub.config.force_game_mode.to_string()),
        _ => Value::Null,
    }
}

/// Server links a plugin changed: the live list, sent to every player now and to each as they join.
static LINKS: Mutex<Option<Vec<Value>>> = Mutex::new(None);
static LINKS_SENT: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());

fn links_packet(rows: &[Value]) -> horizon_protocol::packets::play::Clientbound {
    use horizon_protocol::packets::configuration::{ServerLink, ServerLinkKind, ServerLinks};
    use horizon_protocol::packets::play::Clientbound as C;
    let links = rows
        .iter()
        .filter_map(|row| {
            let kind = match row["known"].as_i64() {
                Some(index) => ServerLinkKind::Known(index as i32),
                None => ServerLinkKind::Custom(horizon_client::lan::data::component_nbt(&row["label"])),
            };
            Some(ServerLink { kind, link: row["url"].as_str()?.to_owned() })
        })
        .collect();
    C::ServerLinks(ServerLinks { links })
}

/// The list a plugin made, to whoever hasn't had it: the configuration phase of a join sends the file's.
fn send_links(game: &Game<'_>) {
    let rows = LINKS.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let (Some(rows), Some(host)) = (rows, game.lan.as_ref()) else {
        return;
    };
    let mut sent = LINKS_SENT.lock().unwrap_or_else(|e| e.into_inner());
    sent.retain(|conn| host.hub.guests.iter().any(|guest| guest.conn == *conn));
    for guest in &host.hub.guests {
        if sent.insert(guest.conn) {
            guest.send(&links_packet(&rows));
        }
    }
}

/// Runs a line as the console, with what it says kept for the caller.
fn run_captured(game: &mut Game<'_>, line: &str) -> Value {
    let before = game.chat_capture.replace(Vec::new());
    let ran = game.native_run_command(line);
    let said = std::mem::replace(&mut game.chat_capture, before).unwrap_or_default();
    json!({ "ran": ran, "lines": said.iter().map(horizon_client::text::Text::component).collect::<Vec<_>>() })
}

/// The links the server was started with, as `core_links` gives them.
fn configured_links(game: &Game<'_>) -> Value {
    use horizon_protocol::packets::configuration::ServerLinkKind;
    let Some(host) = game.lan.as_ref() else {
        return json!([]);
    };
    Value::Array(
        host.hub
            .lan
            .shared
            .config
            .server_links
            .iter()
            .map(|link| match &link.kind {
                ServerLinkKind::Known(index) => json!({ "url": link.link, "known": index }),
                ServerLinkKind::Custom(label) => json!({ "url": link.link, "label": crate::game::commands::paper_entities::tag_json(label) }),
            })
            .collect(),
    )
}

/// What every plugin's DLL shares: each has its own copy of the SDK's statics.
struct Shared {
    plugins: BTreeMap<String, Value>,
    permissions: BTreeMap<String, Value>,
    grants: BTreeMap<(String, String), bool>,
    subscribers: BTreeMap<String, BTreeSet<String>>,
    /// Each player's worked-out permissions (by uuid and op status), kept until a permission or one of
    /// their grants changes. Working them out walks every permission every plugin declared (EssentialsX
    /// alone has about a thousand): done per check, a join's few thousand checks froze the server ~3 s.
    effective: BTreeMap<(String, bool), BTreeMap<String, bool>>,
}

/// Moves each time the shared plugin list changes (`core_plugins_version`).
static PLUGINS_VERSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
static SHARED: Mutex<Shared> = Mutex::new(Shared { plugins: BTreeMap::new(), permissions: BTreeMap::new(), grants: BTreeMap::new(), subscribers: BTreeMap::new(), effective: BTreeMap::new() });

/// `PermissibleBase.calculatePermissions`: the player's defaults, then what plugins granted, each with
/// its children. `None` if nothing here says anything of `node`.
fn effective_permission(shared: &mut Shared, uuid: &str, operator: bool, node: &str) -> Option<bool> {
    let key = (uuid.to_owned(), operator);
    if !shared.effective.contains_key(&key) {
        let all = all_permissions(shared, uuid, operator);
        shared.effective.insert(key.clone(), all);
    }
    shared.effective.get(&key).and_then(|all| all.get(node).copied())
}

/// What `effective_permission` keeps: the player's defaults, then what plugins granted, each with its children.
fn all_permissions(shared: &Shared, uuid: &str, operator: bool) -> BTreeMap<String, bool> {
    fn spread(shared: &Shared, name: &str, value: bool, out: &mut BTreeMap<String, bool>) {
        let lower = name.to_ascii_lowercase();
        out.insert(lower.clone(), value);
        for (child, flag) in shared.permissions.get(&lower).and_then(|row| row["children"].as_object()).into_iter().flatten() {
            let flag = flag.as_bool().unwrap_or(true);
            if !out.contains_key(&child.to_ascii_lowercase()) {
                spread(shared, child, flag == value, out);
            }
        }
    }
    let mut out = BTreeMap::new();
    for (name, row) in &shared.permissions {
        // `default: false` in a `plugin.yml` comes as a boolean, `default: op` as a word.
        let default = row["default"].as_bool().map_or_else(|| row["default"].as_str().unwrap_or("op").to_owned(), |flag| flag.to_string());
        let applies = match default.to_ascii_lowercase().replace(['_', ' '], "").as_str() {
            "true" => true,
            "false" => false,
            "notop" => !operator,
            _ => operator,
        };
        if applies {
            spread(shared, name, true, &mut out);
        }
    }
    let mut grants = BTreeMap::new();
    for ((who, name), value) in &shared.grants {
        if who == uuid {
            spread(shared, name, *value, &mut grants);
        }
    }
    out.extend(grants);
    out
}

/// `None` for an op that isn't this family's.
pub(in crate::game) fn query(game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    Some(match op {
        "core_game_rules" => game_rules(game),
        "core_spawn_limits" => Value::Object(Category::ALL.iter().map(|category| (category.name().to_owned(), json!({ "limit": category.max(), "persistent": category.persistent() }))).collect()),
        "core_level_counts" => game.native_in_level(dimension(query), |game| json!({ "chunks": game.world.chunks.len(), "block_entities": game.world.block_entities.len(), "tickers": game.world.block_entity_tickers().len() })).unwrap_or(Value::Null),
        "core_locate_structure" => locate_structure(game, query),
        "core_locate_biome" => locate_biome(game, query),
        "core_place_feature" => place_feature(game, query),
        "core_chunk_viewers" => column(&query["chunk"]).map_or(Value::Null, |key| chunk_viewers(game, dimension(query), key)),
        "core_refresh_chunk" => column(&query["chunk"]).map_or(json!(false), |key| refresh_chunk(game, dimension(query), key)),
        "core_chunk" => column(&query["chunk"]).map_or(Value::Null, |key| chunk_facts(game, dimension(query), key)),
        "core_chunk_saved" => column(&query["chunk"]).map_or(json!(false), |key| chunk_saved(game, dimension(query), key)),
        "core_chunk_structures" => column(&query["chunk"]).map_or(Value::Null, |key| chunk_structures(game, dimension(query), key)),
        "core_structures_at" => position(&query["pos"]).map_or(Value::Null, |at| game.native_in_level(dimension(query), |game| json!(game.structures_reaching(at).into_iter().map(|(name, _)| name).collect::<Vec<_>>())).unwrap_or(Value::Null)),
        "core_pois" => points_of_interest(game, query),
        "core_world_gen" => horizon_client::save::WorldGen::read(&game.world_root).map_or(Value::Null, |settings| json!({ "structures": settings.structures, "bonus_chest": settings.bonus_chest })),
        "core_set_hardcore" => {
            let hardcore = query["hardcore"].as_bool().unwrap_or(false);
            game.level.hardcore = hardcore;
            if let Some(host) = game.lan.as_mut() {
                host.hub.config.hardcore = hardcore;
            }
            json!(true)
        }
        "core_distances" => game.lan.as_ref().map_or(Value::Null, |host| json!({ "view": host.hub.config.view_distance, "simulation": host.hub.config.simulation_distance })),
        "core_level_distance" => {
            let which = match query["which"].as_str()? {
                "view" => "view",
                "send" => "send",
                _ => "simulation",
            };
            let level = dimension(query).trim_start_matches("minecraft:").to_owned();
            if let Some(distance) = query["distance"].as_i64() {
                let mut all = LEVEL_DISTANCES.lock().unwrap_or_else(|e| e.into_inner());
                if distance < 0 {
                    all.remove(&(level.clone(), which));
                } else {
                    all.insert((level.clone(), which), (distance as i32).clamp(2, 32));
                }
                DISTANCES_SET.store(true, Ordering::Relaxed);
            }
            let Some(host) = game.lan.as_ref() else {
                return Some(Value::Null);
            };
            let view = level_distance(&level, "view", host.hub.config.view_distance);
            json!(match which {
                "view" => view,
                "send" => level_distance(&level, "send", view),
                _ => level_distance(&level, "simulation", host.hub.config.simulation_distance),
            })
        }
        "core_map" => {
            let id = query["id"].as_i64().and_then(|id| i32::try_from(id).ok())?;
            let colors = query["colors"].as_bool().unwrap_or(false);
            game.map(id).map_or(Value::Null, |map| map_json(id, map, colors))
        }
        "core_map_touch" => {
            MAPS_TOUCHED.lock().unwrap_or_else(|e| e.into_inner()).insert(query["id"].as_i64()? as i32);
            json!(true)
        }
        "core_map_create" => {
            let name = dimension(query).to_owned();
            let name = if name.contains(':') { name } else { format!("minecraft:{name}") };
            let at = |i: usize| query["at"][i].as_f64().unwrap_or(0.0);
            let id = game.new_map_id(MapData::fresh(at(0), at(1), query["scale"].as_u64().unwrap_or(3) as u8, &name));
            json!(id)
        }
        "core_map_change" => map_change(game, query),
        "core_map_send" => map_send(game, query),
        "core_map_holders" => {
            let id = query["id"].as_i64().and_then(|id| i32::try_from(id).ok())?;
            map_holders(game, id)
        }
        "core_map_palette" => json!(horizon_client::blocks::MAP_COLORS.to_vec()),
        "core_map_decorations" => {
            let Some(host) = game.lan.as_ref() else {
                return Some(Value::Null);
            };
            Value::Object(names(&query["names"]).into_iter().filter_map(|name| Some((name.clone(), json!(host.hub.registry_id("minecraft:map_decoration_type", &format!("minecraft:{name}"))?)))).collect())
        }
        "core_structure_fill" => structure_fill(game, query),
        "core_structure_place" => structure_place(game, query),
        "core_structure_tree" => {
            let name = query["template"].as_str()?;
            game.template_entry(name).map_or(Value::Null, |entry| crate::game::commands::paper_entities::tag_json(&entry.tag))
        }
        "core_structure_known" => {
            let name = query["template"].as_str()?;
            json!(game.structure_templates.get(name).is_some_and(Option::is_some))
        }
        "core_structure_copy" => {
            let (Some(from), Some(to)) = (query["from"].as_str(), query["to"].as_str()) else {
                return Some(json!(false));
            };
            let entry = game.template_entry(from);
            let known = entry.is_some();
            game.structure_templates.insert(to.to_owned(), entry);
            json!(known)
        }
        "core_structure_forget" => {
            let name = query["template"].as_str()?;
            json!(game.structure_templates.remove(name).is_some_and(|entry| entry.is_some()))
        }
        "core_structure_save" => {
            let name = query["template"].as_str()?;
            json!(game.template_entry(name).is_some() && game.save_structure_area(name, [0; 3], [0; 3], true, "?", true, false))
        }
        "core_structure_delete" => {
            let name = query["template"].as_str()?;
            game.structure_templates.remove(name);
            let (space, path) = name.split_once(':').unwrap_or(("minecraft", name));
            let file = game.world_root.join("generated").join(space).join("structures").join(format!("{path}.nbt"));
            json!(!path.contains("..") && !space.contains("..") && std::fs::remove_file(file).is_ok())
        }
        "core_loot" => {
            let table = query["table"].as_str()?;
            let Some(loot) = horizon_client::structure_blocks::loot_data(&game.options.resources, &game.pack_files()) else {
                return Some(Value::Null);
            };
            if !loot.has_table(table) {
                return Some(Value::Null);
            }
            let mut params = horizon_world::loot::EntityLootParams { luck: query["luck"].as_f64().unwrap_or(0.0) as f32, ..Default::default() };
            let looted = query["looted"].as_u64().and_then(|id| game.mobs.iter().find(|mob| u64::from(mob.id) == id));
            if let Some(mob) = looted {
                params.this = horizon_client::mobs::drops::loot_entity(mob, None);
            }
            if let Some(uuid) = query["killer"].as_str().and_then(|uuid| u128::from_str_radix(&uuid.replace('-', ""), 16).ok()) {
                let mut killer = horizon_world::loot::LootEntity::new("minecraft:player");
                let held = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| guest.uuid == uuid)).and_then(|guest| guest.inventory.get(guest.selected).cloned().flatten());
                if let Some(held) = held {
                    killer.equipment.push(("mainhand".to_owned(), crate::game::creatures::to_loot(&held)));
                }
                params.player = Some(killer);
            }
            let mut random = horizon_core::random::LegacyRandom::new(query["seed"].as_i64().unwrap_or_else(|| game.world.game_time));
            let drops = loot.roll_entity(table, &mut random, &params);
            if let Some(slots) = query["slots"].as_u64() {
                let spread = horizon_world::loot::spread_loot(drops, slots as usize, &mut random);
                return Some(spread.iter().map(|stack| stack.as_ref().map_or(Value::Null, |stack| crate::game::paper_query::stack_json(&crate::game::creatures::from_loot(stack)))).collect());
            }
            Value::Array(drops.iter().filter(|stack| !stack.is_empty()).map(|stack| crate::game::paper_query::stack_json(&crate::game::creatures::from_loot(stack))).collect())
        }
        "core_loot_known" => json!(game.items().has_loot_table(query["table"].as_str()?)),
        "core_data_tree" => data_tree(game, query["path"].as_str()?),
        "core_craft" => craft(game, query),
        "core_recipe_key" => {
            let grid: Vec<Option<horizon_client::items::ItemStack>> = query["grid"].as_array().into_iter().flatten().map(stack_from).collect();
            let size = if grid.len() == 9 { 3 } else { 2 };
            game.res.recipes().find(&grid, size).map_or(Value::Null, |(key, _)| json!(key))
        }
        "core_boss_bars" => json!(game.bosses.bars.ids()),
        "core_player_file" => {
            let uuid = query["uuid"].as_str()?;
            let file = game.world_root.join(format!("players/data/{uuid}.dat"));
            horizon_client::save::read_dat(&file).map_or(Value::Null, |tag| crate::game::commands::paper_entities::tag_json(&tag))
        }
        "core_boss_flag" => game.bosses.bars.get(query["id"].as_str()?).map_or(Value::Null, |bar| json!({ "darken_sky": bar.darken, "play_boss_music": bar.music, "create_fog": bar.fog })),
        "core_boss_flag_set" => {
            let on = query["on"].as_bool().unwrap_or(false);
            let flag = query["flag"].as_str()?.to_owned();
            let Some(bar) = game.bosses.bars.get_mut(query["id"].as_str()?) else {
                return Some(json!(false));
            };
            match flag.as_str() {
                "darken_sky" => bar.darken = on,
                "play_boss_music" => bar.music = on,
                _ => bar.fog = on,
            }
            json!(true)
        }
        "core_objective" => {
            let name = query["name"].as_str()?;
            game.score.board.objectives.iter().find(|objective| objective.name == name).map_or(Value::Null, |objective| json!({ "render_type": format!("{:?}", objective.render_type).to_ascii_lowercase(), "number_format": number_format_json(objective.number_format.as_ref()) }))
        }
        "core_team" => {
            let name = query["name"].as_str()?;
            game.score.board.teams.iter().find(|team| team.name == name).map_or(Value::Null, |team| json!({ "name_tag_visibility": format!("{:?}", team.nametag_visibility), "death_message_visibility": format!("{:?}", team.death_message_visibility), "collision_rule": format!("{:?}", team.collision_rule) }))
        }
        "core_item_components" => match horizon_client::item_data::command_components(query["text"].as_str()?) {
            Ok(tag) => crate::game::commands::paper_entities::tag_json(&tag),
            Err(_) => Value::Null,
        },
        "core_stopping" => json!(STOPPING.load(Ordering::Relaxed)),
        "core_set_enforced" => {
            let on = query["on"].as_bool().unwrap_or(false);
            let Some(lists) = game.server_lists.clone() else {
                return Some(json!(false));
            };
            lists.lock().unwrap_or_else(|e| e.into_inner()).enforce_whitelist = on;
            game.store_property("enforce-whitelist", on.to_string());
            game.kick_unlisted_players(&lists);
            json!(true)
        }
        "core_set_spawn_radius" => {
            let Some(host) = game.lan.as_mut() else {
                return Some(json!(false));
            };
            host.hub.spawn_protection = query["radius"].as_i64().unwrap_or(16) as i32;
            json!(true)
        }
        "core_chunk_light" => column(&query["chunk"]).map_or(Value::Null, |key| chunk_light(game, dimension(query), key)),
        "core_raids" => raids(game, dimension(query)),
        "core_raiders" => {
            let raid = query["raid"].as_i64().and_then(|id| i32::try_from(id).ok())?;
            game.native_in_level(dimension(query), |game| json!(game.mobs.iter().filter(|mob| mob.alive() && horizon_client::mobs::raider::state(mob).is_some_and(|state| state.raid_id == Some(raid))).map(|mob| mob.id).collect::<Vec<_>>())).unwrap_or(Value::Null)
        }
        "core_dragon" => {
            // Crystals get their id when a player first sees them: asked for, one is due now.
            game.native_in_level("minecraft:the_end", |game| {
                let mut next = game.next_entity;
                for crystal in game.crystals.iter_mut().filter(|crystal| crystal.id == 0) {
                    next += 1;
                    crystal.id = next;
                }
                game.next_entity = next;
            });
            let Some(mut status) = game.native_in_level("minecraft:the_end", |game| game.dragon_status()) else {
                return Some(Value::Null);
            };
            status["previously_killed"] = json!(game.fight.previously_killed);
            let file = horizon_client::save::dimension_dir(&game.world_root, "minecraft:the_end").join("data/minecraft/ender_dragon_fight.dat");
            let saved = horizon_client::save::read_dat(&file);
            status["gateways_left"] = saved.as_ref().and_then(|tag| tag.get("data")).and_then(|data| data.get("gateways")).map_or(Value::Null, |gateways| crate::game::commands::paper_entities::tag_json(gateways).as_array().map_or(Value::Null, |list| json!(list.len())));
            status
        }
        "core_dragon_killed" => {
            game.fight.previously_killed = query["killed"].as_bool().unwrap_or(false);
            json!(true)
        }
        "core_paused" => json!(PAUSED.load(Ordering::Relaxed)),
        "core_feature_flags" => {
            let tag = horizon_client::save::read_dat(&game.world_root.join("level.dat"));
            tag.as_ref().and_then(|tag| tag.get("Data")).and_then(|data| data.get("enabled_features")).map_or_else(|| json!(["minecraft:vanilla"]), crate::game::commands::paper_entities::tag_json)
        }
        "core_ban_save" => {
            let Some(lists) = game.server_lists.clone() else {
                return Some(json!(false));
            };
            let addresses = query["addresses"].as_bool().unwrap_or(false);
            let file = if addresses { "banned-ips.json" } else { "banned-players.json" };
            let row = &query["row"];
            let key = if addresses { "ip" } else { "uuid" };
            let mut rows: Vec<Value> = std::fs::read(file).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default();
            rows.retain(|known| known[key] != row[key]);
            rows.push(row.clone());
            let written = std::fs::write(file, Value::Array(rows).to_string()).is_ok();
            let mut lists = lists.lock().unwrap_or_else(|e| e.into_inner());
            let loaded = if addresses { lists.ip_bans.load().is_ok() } else { lists.bans.load().is_ok() };
            json!(written && loaded)
        }
        "core_score" => {
            let (Some(entry), Some(objective)) = (query["entry"].as_str(), query["objective"].as_str()) else {
                return Some(Value::Null);
            };
            let score = game.score.board.scores.get(entry).and_then(|scores| scores.get(objective));
            score.map_or(Value::Null, |score| json!({ "display": score.display, "number_format": number_format_json(score.number_format.as_ref()) }))
        }
        "core_score_lock" => {
            let (Some(entry), Some(objective)) = (query["entry"].as_str(), query["objective"].as_str()) else {
                return Some(json!(false));
            };
            let locked = query["locked"].as_bool().unwrap_or(true);
            match game.score.board.scores.get_mut(entry).and_then(|scores| scores.get_mut(objective)) {
                Some(score) => {
                    score.locked = locked;
                    json!(true)
                }
                None => json!(false),
            }
        }
        "core_game_event" => {
            let name = query["name"].as_str()?;
            json!({ "range": horizon_client::sculk::notification_radius(name), "frequency": horizon_client::sculk::frequency(name) })
        }
        "core_game_event_send" => {
            let (Some(name), Some(at)) = (query["name"].as_str(), query["pos"].as_array()) else {
                return Some(json!(false));
            };
            let pos = [at[0].as_f64().unwrap_or(0.0), at[1].as_f64().unwrap_or(0.0), at[2].as_f64().unwrap_or(0.0)];
            // The game's events have 'static names: a table of them, kept, is bounded by how many there are.
            static NAMES: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
            let name: &'static str = {
                let mut names = NAMES.lock().unwrap_or_else(|e| e.into_inner());
                match names.iter().find(|known| **known == name) {
                    Some(known) => known,
                    None => {
                        let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
                        names.push(leaked);
                        leaked
                    }
                }
            };
            if let Some(uuid) = query["player"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok()) {
                let Some(target) = game.target_by_uuid(uuid) else {
                    return Some(json!(false));
                };
                game.for_player(target, |game| {
                    let context = game.player_context(None);
                    game.post_game_event(name, pos, context);
                });
                return Some(json!(true));
            }
            let source = query["source"].as_u64().map(|id| id as u32);
            json!(game.native_in_level(dimension(query), |game| {
                let context = source.map_or_else(horizon_client::sculk::Context::default, |id| game.mob_context(id, None));
                game.post_game_event(name, pos, context);
            }).is_some())
        }
        "core_command_block" => {
            let Some(source) = game.command_source.filter(|source| source.block.is_some()) else {
                return Some(Value::Null);
            };
            json!({ "pos": source.block.map(|block| [block.0, block.1, block.2]), "dimension": format!("minecraft:{}", source.dimension.id().trim_start_matches("minecraft:")) })
        }
        "core_attribute" => {
            let motion = game.res.motion();
            let attributes = &motion.attributes;
            attributes.id(query["id"].as_str()?).map_or(Value::Null, |id| json!({ "default": attributes.attribute(id).definition.default_value() }))
        }
        "core_item_default" => {
            let id = query["id"].as_str()?;
            let stack = stack_from(&json!({ "item": id, "count": 1 }));
            let encoded = stack.as_ref().and_then(horizon_client::item_data::encode_components);
            encoded.as_ref().and_then(|tag| tag.get(query["component"].as_str().unwrap_or_default())).map_or(Value::Null, crate::game::commands::paper_entities::tag_json)
        }
        "core_plugin_add" => {
            let row = query["plugin"].clone();
            let name = row["name"].as_str()?.to_ascii_lowercase();
            let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
            for (node, data) in row["yaml"]["permissions"].as_object().into_iter().flatten() {
                let mut permission = data.clone();
                permission["name"] = json!(node);
                shared.permissions.entry(node.to_ascii_lowercase()).or_insert(permission);
            }
            shared.effective.clear();
            shared.plugins.insert(name, row);
            PLUGINS_VERSION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            json!(true)
        }
        // What changes when the plugin list does: a plugin asks this (cheap) before the whole list (each row
        // with its plugin.yml), which it then keeps until the number moves. With many plugins, each asking for
        // the list many times a tick, reading it every time froze the server as a player joined.
        "core_plugins_version" => json!(PLUGINS_VERSION.load(std::sync::atomic::Ordering::Relaxed)),
        "core_plugins" => Value::Array(SHARED.lock().unwrap_or_else(|e| e.into_inner()).plugins.values().cloned().collect()),
        "core_plugins_clear" => {
            PLUGINS_VERSION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
            shared.plugins.clear();
            shared.permissions.clear();
            shared.effective.clear();
            json!(true)
        }
        "core_plugin_disable" => {
            let name = query["name"].as_str()?.to_owned();
            let known = SHARED.lock().unwrap_or_else(|e| e.into_inner()).plugins.contains_key(&name.to_ascii_lowercase());
            if known {
                crate::game::paper_bridge::fire_custom(game, "core_plugin_disable", json!({ "name": name }));
            }
            json!(known)
        }
        "core_perm_add" => {
            let row = query["permission"].clone();
            let name = row["name"].as_str()?.to_ascii_lowercase();
            let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
            if shared.permissions.contains_key(&name) {
                return Some(json!(false));
            }
            shared.permissions.insert(name, row);
            shared.effective.clear();
            json!(true)
        }
        "core_perm_get" => SHARED.lock().unwrap_or_else(|e| e.into_inner()).permissions.get(query["name"].as_str()?).cloned().unwrap_or(Value::Null),
        "core_perm_remove" => {
            let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
            let removed = shared.permissions.remove(query["name"].as_str()?).is_some();
            shared.effective.clear();
            json!(removed)
        }
        "core_perm_list" => Value::Array(SHARED.lock().unwrap_or_else(|e| e.into_inner()).permissions.values().cloned().collect()),
        "core_perm_grant" => {
            let key = (query["uuid"].as_str()?.to_owned(), query["node"].as_str()?.to_owned());
            let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
            shared.effective.retain(|(who, _), _| *who != key.0);
            match query["value"].as_bool() {
                Some(value) => shared.grants.insert(key, value),
                None => shared.grants.remove(&key),
            };
            json!(true)
        }
        "core_perm_check" => {
            let (uuid, node) = (query["uuid"].as_str()?, query["node"].as_str()?);
            let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
            effective_permission(&mut shared, uuid, query["op_status"].as_bool().unwrap_or(false), node).map_or(Value::Null, |value| json!(value))
        }
        "core_perm_manager" => PERMISSION_OWNER.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map_or(Value::Null, |owner| json!(owner)),
        "core_perm_manager_set" => {
            let mut owner = PERMISSION_OWNER.lock().unwrap_or_else(|e| e.into_inner());
            // Going back to the engine's is up to whoever had it.
            if query["plugin"].is_string() || *owner == query["was"].as_str().map(str::to_owned) {
                *owner = query["plugin"].as_str().map(str::to_owned);
            }
            json!(true)
        }
        "core_perm_subscribe" => {
            let (node, uuid) = (query["node"].as_str()?, query["uuid"].as_str()?);
            let mut shared = SHARED.lock().unwrap_or_else(|e| e.into_inner());
            let list = shared.subscribers.entry(node.to_owned()).or_default();
            if query["on"].as_bool().unwrap_or(false) {
                list.insert(uuid.to_owned());
            } else {
                list.remove(uuid);
            }
            json!(true)
        }
        "core_perm_subscribers" => json!(SHARED.lock().unwrap_or_else(|e| e.into_inner()).subscribers.get(query["node"].as_str()?).cloned().unwrap_or_default()),
        "core_commands" => {
            let usage = horizon_client::command::vanilla::game_dispatcher().smart_usage(0, 4);
            let mut names: Vec<String> = usage.into_iter().filter_map(|(_, line)| line.split_whitespace().next().map(str::to_owned)).collect();
            names.sort();
            names.dedup();
            json!(names)
        }
        "core_plugin_message_in" => {
            json!(crate::game::paper_bridge::fire_custom(game, "plugin_message", json!({ "uuid": query["uuid"], "name": query["name"], "channel": query["channel"], "data": query["data"] })))
        }
        "core_tick_stats" => tick_stats(),
        "core_client_channels" => {
            let uuid = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            let mut all = CLIENT_CHANNELS.lock().unwrap_or_else(|e| e.into_inner());
            heard_configuring(&mut all, uuid);
            all.get(&uuid).map_or(json!([]), |channels| json!(channels))
        }
        "core_registry" => {
            static REPORT: std::sync::OnceLock<Option<horizon_protocol::registry::Builtin>> = std::sync::OnceLock::new();
            let report = REPORT.get_or_init(|| horizon_protocol::registry::Builtin::find(&game.options.resources).and_then(|path| horizon_protocol::registry::Builtin::load(&path).ok()));
            report.as_ref().and_then(|report| report.get(query["registry"].as_str()?)).map_or(json!([]), |ids| json!(ids.names))
        }
        "core_player_values" => {
            let uuid = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            let written = horizon_client::lan::playerdata::BUKKIT_VALUES.lock().unwrap_or_else(|e| e.into_inner()).get(&uuid).cloned();
            let online = || game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == uuid)?.saved.as_ref()?.get("BukkitValues").cloned();
            let saved = || horizon_client::save::read_dat(&player_path(game, uuid))?.get("BukkitValues").cloned();
            written.or_else(online).or_else(saved).map_or(json!({}), |tag| if query["kinds"] == true { crate::game::paper_fam_paper::tag_kinds(&tag) } else { crate::game::commands::paper_entities::tag_json(&tag) })
        }
        "core_player_values_set" => {
            let uuid = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            let Ok((values, _)) = horizon_world::nbt::parse_snbt(query["values"].as_str()?) else {
                return Some(json!(false));
            };
            horizon_client::lan::playerdata::BUKKIT_VALUES.lock().unwrap_or_else(|e| e.into_inner()).insert(uuid, values.clone());
            // Someone on saves with the rest of their file; someone away has only the file.
            if game.lan.as_ref().is_some_and(|host| host.hub.guests.iter().any(|guest| guest.uuid == uuid)) {
                return Some(json!(true));
            }
            let path = player_path(game, uuid);
            let Some(mut file) = horizon_client::save::read_dat(&path) else {
                return Some(json!(false));
            };
            if let Tag::Compound(map) = &mut file {
                map.insert(horizon_world::nbt::JavaString::from("BukkitValues"), values);
            }
            json!(horizon_client::save::write_dat(&path, file).is_ok())
        }
        "core_commands_clear" => {
            crate::game::paper_bridge::fire_custom(game, "core_commands_clear", json!({}));
            json!(true)
        }
        "core_game_complete" => {
            let line = query["line"].as_str()?;
            let d = horizon_command::vanilla::game_dispatcher();
            let parse = d.parse(line, &game.ui.registries);
            let (_, suggestions) = d.suggestions(&parse, line.len(), &game.ui.registries);
            suggestions.iter().map(|suggestion| json!(suggestion.text)).collect()
        }
        "core_reload_all" => {
            RELOAD_ALL.store(true, Ordering::Relaxed);
            json!(true)
        }
        "core_nbt_read" => {
            let bytes: Vec<u8> = query["bytes"].as_array()?.iter().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect();
            horizon_world::nbt::read_named(&bytes, horizon_world::nbt::Limits::default()).ok().map_or(Value::Null, |(named, _)| crate::game::commands::paper_entities::tag_json(&named.tag))
        }
        "core_border_send" => {
            use horizon_protocol::packets::play::{self, Clientbound as C};
            let uuid = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            let packet = play::InitializeBorder {
                center_x: query["x"].as_f64()?,
                center_z: query["z"].as_f64()?,
                old_size: query["from"].as_f64()?,
                new_size: query["size"].as_f64()?,
                lerp_time: query["ticks"].as_i64().unwrap_or(0) * 50,
                absolute_max_size: 29_999_984,
                warning_blocks: query["warning_blocks"].as_i64().unwrap_or(5) as i32,
                warning_time: query["warning_time"].as_i64().unwrap_or(15) as i32,
            };
            let guest = game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == uuid)?;
            guest.send(&C::InitializeBorder(packet));
            json!(true)
        }
        "core_explode" => {
            let at = query["at"].as_array()?;
            let order = (dimension(query).to_owned(), [at.first()?.as_f64()?, at.get(1)?.as_f64()?, at.get(2)?.as_f64()?], query["power"].as_f64()? as f32, query["fire"].as_bool().unwrap_or(false), query["breaks"].as_bool().unwrap_or(true));
            let copy = order.clone();
            match with_hub(game, |game| explode(game, copy)) {
                Some(went) => json!(went),
                None => {
                    EXPLOSIONS.lock().unwrap_or_else(|e| e.into_inner()).push(order);
                    json!(true)
                }
            }
        }
        "core_particle_for" => with_hub(game, |game| crate::game::paper_fam_paper::particle(game, query)).unwrap_or(json!(false)),
        "core_adv_live" => {
            let uuid = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            with_hub(game, |game| {
                if game.lan_acting_uuid == Some(uuid) {
                    return Some(game.adv.progress.to_json(&game.adv.tree, horizon_world::DATA_VERSION));
                }
                let guest = game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == uuid && guest.advancements_loaded)?;
                Some(guest.advancements.to_json(&game.adv.tree, horizon_world::DATA_VERSION))
            })
            .flatten()
            .unwrap_or(Value::Null)
        }
        "core_weather_own" => {
            let uuid = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            let mut personal = horizon_client::lan::hub::PERSONAL_WEATHER.lock().unwrap_or_else(|e| e.into_inner());
            match query["downfall"].as_bool() {
                Some(downfall) => personal.entry(uuid).or_insert((downfall, 0.0)).0 = downfall,
                None => {
                    personal.remove(&uuid);
                }
            }
            json!(true)
        }
        "core_auto_save" => {
            let level = dimension(query).to_owned();
            if let Some(on) = query["on"].as_bool() {
                let mut off = NO_SAVE.lock().unwrap_or_else(|e| e.into_inner());
                let name = level.trim_start_matches("minecraft:").to_owned();
                if on {
                    off.remove(&name);
                } else {
                    off.insert(name);
                }
            }
            json!(!no_save(&level))
        }
        "core_run_now" => {
            let line = query["line"].as_str()?;
            run_at_once(game, line).map_or_else(
                || {
                    wait_with_line(None, line);
                    Value::Null
                },
                |ran| json!(ran),
            )
        }
        "core_run_as_now" => {
            let (player, line) = (u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?, query["line"].as_str()?);
            player_line(game, player, line).map_or_else(
                || {
                    wait_with_line(Some(player), line);
                    Value::Null
                },
                |ran| json!(ran),
            )
        }
        "core_tables_reload" => {
            if with_hub(game, |game| game.reload_world_tables()).is_none() {
                RECIPES_STALE.store(true, Ordering::Relaxed);
            }
            json!(true)
        }
        "core_set_time" => {
            // A plugin often does this in a player's turn, where a command can't run: it reads back at once, and
            // the game's own command tells the clients when the turn is over.
            let time = query["time"].as_i64()?;
            let level = dimension(query).to_owned();
            let ok = game.native_in_level(&level, |game| game.world.day_time = game.world.day_time - game.world.day_time.rem_euclid(24000) + time).is_some();
            crate::game::paper_bridge::push_action(json!({ "op": "run_command", "text": format!("execute in {level} run time set {time}") }).to_string());
            json!(ok)
        }
        "core_set_storm" => {
            let storm = query["storm"].as_bool()?;
            json!(game.native_in_level(dimension(query), |game| {
                let weather = &mut game.world.weather;
                weather.raining = storm;
                weather.rain_time = 0;
                weather.clear_time = 0;
            }).is_some())
        }
        "core_spawn_flags" => SPAWN_FLAGS.lock().unwrap_or_else(|e| e.into_inner()).get(dimension(query)).map_or(Value::Null, |(monsters, animals)| json!({ "monsters": monsters, "animals": animals })),
        "core_spawn_flags_set" => {
            SPAWN_FLAGS.lock().unwrap_or_else(|e| e.into_inner()).insert(dimension(query).to_owned(), (query["monsters"].as_bool()?, query["animals"].as_bool()?));
            json!(true)
        }
        "core_level_limit" => LEVEL_LIMITS.lock().unwrap_or_else(|e| e.into_inner()).get(&(dimension(query).to_owned(), query["category"].as_str()?.to_owned())).map_or(Value::Null, |limit| json!(limit)),
        "core_level_limit_set" => {
            let key = (dimension(query).to_owned(), query["category"].as_str()?.to_owned());
            let limit = query["limit"].as_i64()? as i32;
            let mut limits = LEVEL_LIMITS.lock().unwrap_or_else(|e| e.into_inner());
            if limit < 0 {
                limits.remove(&key);
            } else {
                limits.insert(key, limit);
            }
            json!(true)
        }
        "core_world_spawn" => own_spawn(game, dimension(query)).map_or(Value::Null, |(pos, yaw, pitch)| json!({ "pos": pos, "yaw": yaw, "pitch": pitch })),
        "core_world_spawn_set" => {
            if dimension(query) == horizon_client::save::spawn_dimension() {
                return Some(json!(false));
            }
            let at = position(&query["pos"])?;
            crate::game::paper_bridge::on_trigger(game, "ew_spawn_set", json!({ "uuid": Value::Null, "level": dimension(query) }));
            save_own_spawn(game, dimension(query), at, query["yaw"].as_f64().unwrap_or(0.0) as f32, query["pitch"].as_f64().unwrap_or(0.0) as f32);
            json!(true)
        }
        "core_update_resources" => {
            if game.lan.is_none() {
                return Some(json!(false));
            }
            let host = game.lan.as_mut()?;
            let sync = host.hub.lan.shared.sync();
            host.hub.reload_resources(sync);
            json!(true)
        }
        "core_remote_peer" => REMOTE_PEER.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map_or(Value::Null, |peer| json!(peer)),
        "core_remote_say" => {
            game.console_reply(horizon_client::text::Text::literal(query["text"].as_str()?.to_owned()));
            json!(true)
        }
        "core_command_add" => {
            let usage = query["text"].as_str().unwrap_or_default();
            let word = usage.split_whitespace().next().unwrap_or_default().to_owned();
            // Who registered it: one plugin answers a word several have (command-owners.yml, `/commands`). The
            // game's tree takes no `plugin:word` words; the console and `dispatchCommand` still reach a plugin so.
            if !word.is_empty() && !word.contains(':') {
                crate::game::paper_events::note_word(&word, None);
            }
            horizon_mod_runtime::push(json!({ "op": "register_command", "text": query["text"], "level": query["level"] }).to_string());
            crate::game::paper_bridge::take_commands();
            TREE_STALE.store(true, Ordering::Relaxed);
            json!(true)
        }
        "core_lightning_target" => {
            let (x, z) = (query["x"].as_i64()? as i32, query["z"].as_i64()? as i32);
            game.native_in_level(dimension(query), |game| game.lightning_target(x, z)).flatten().map_or(Value::Null, |at| json!([f64::from(at[0]), f64::from(at[1]), f64::from(at[2])]))
        }
        "core_silent_bolt" => {
            let at = query["at"].as_array()?;
            SILENT_BOLTS.lock().unwrap_or_else(|e| e.into_inner()).push([at.first()?.as_f64()?, at.get(1)?.as_f64()?, at.get(2)?.as_f64()?]);
            json!(true)
        }
        "core_respawn_world" => json!(horizon_client::save::spawn_dimension()),
        "core_set_respawn_world" => {
            // The game's one `spawn` is the respawn world's: the world it was keeps its own, as Paper's do.
            let wanted = dimension(query).to_owned();
            let old = horizon_client::save::spawn_dimension();
            if wanted != old {
                if let Some(at) = game.level.world_spawn {
                    let [yaw, pitch] = game.level.world_spawn_angle;
                    save_own_spawn(game, &old, at, yaw, pitch);
                }
                if let Some((pos, yaw, pitch)) = own_spawn(game, &wanted) {
                    game.level.world_spawn = Some(pos);
                    game.level.world_spawn_angle = [yaw, pitch];
                }
                horizon_client::save::set_spawn_dimension(&wanted);
            }
            json!(true)
        }
        "core_recipes_clear" => {
            // What the world's packs hold now goes too: what's added later has its own id.
            let packs: BTreeSet<String> = game.pack_files().kind("recipe").map(|(id, _)| id.to_owned()).collect();
            *horizon_world::loot::CLEARED_RECIPES.lock().unwrap_or_else(|e| e.into_inner()) = Some(packs);
            game.world_tables = game.build_world_tables();
            json!(true)
        }
        "core_recipes_back" => {
            *horizon_world::loot::CLEARED_RECIPES.lock().unwrap_or_else(|e| e.into_inner()) = None;
            game.world_tables = game.build_world_tables();
            json!(true)
        }
        "core_structure_plan" => structure_plan(game, query),
        "core_structure_preview" => {
            let Some(at) = position(&query["pos"]) else {
                return Some(json!(false));
            };
            let Ok(state) = crate::game::render_control::parse_state(game.pack.assets.registry, query["state"].as_str()?) else {
                return Some(json!(false));
            };
            let level = dimension(query).to_owned();
            let before = game.native_in_level(&level, |game| game.world.set_quietly(at[0], at[1], at[2], state)).flatten();
            let shown = before.is_some();
            if let Some(before) = before {
                PREVIEW.lock().unwrap_or_else(|e| e.into_inner()).push((level, at, before));
            }
            json!(shown)
        }
        "core_void" => json!({ "enabled": void_rule_of(dimension(query)).0, "amount": void_rule_of(dimension(query)).1, "offset": void_rule_of(dimension(query)).2 }),
        "core_void_set" => {
            let name = dimension(query).to_owned();
            let mut rule = void_rule_of(&name);
            rule.0 = query["enabled"].as_bool().unwrap_or(rule.0);
            rule.1 = query["amount"].as_f64().map_or(rule.1, |amount| amount as f32);
            rule.2 = query["offset"].as_f64().unwrap_or(rule.2);
            VOID_RULES.lock().unwrap_or_else(|e| e.into_inner()).insert(name, rule);
            json!(true)
        }
        "core_sign_lines" => {
            let pos = query["pos"].as_array()?;
            let at = [pos.first()?.as_i64()? as i32, pos.get(1)?.as_i64()? as i32, pos.get(2)?.as_i64()? as i32];
            let lines = query["lines"].as_array().map(|lines| lines.iter().filter_map(|line| line.as_str().map(str::to_owned)).collect());
            *SIGN_LINES.lock().unwrap_or_else(|e| e.into_inner()) = lines.map(|lines| (at, query["front"].as_bool().unwrap_or(true), lines));
            json!(true)
        }
        "core_damage_context" => {
            let now = game.world.game_time;
            DAMAGE_CONTEXTS.lock().unwrap_or_else(|e| e.into_inner()).insert(query["uuid"].as_str()?.to_owned(), (now, query["context"].clone()));
            json!(true)
        }
        "core_damage_context_of" => {
            // The tick it was dealt or the next: `/damage` runs at the end of the tick that asked.
            let now = game.world.game_time;
            let mut contexts = DAMAGE_CONTEXTS.lock().unwrap_or_else(|e| e.into_inner());
            contexts.retain(|_, (at, _)| now - *at <= 1);
            contexts.get(query["uuid"].as_str()?).map_or(Value::Null, |(_, context)| context.clone())
        }
        "core_bossbar_player" => {
            let uuid = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            let change = (query["id"].as_str()?.to_owned(), uuid_words(uuid), query["add"].as_bool().unwrap_or(true));
            if !change_bar(game, &change) {
                BAR_CHANGES.lock().unwrap_or_else(|e| e.into_inner()).push((game.world.game_time, change));
            }
            json!(true)
        }
        "core_plugin_enable" => {
            let name = query["name"].as_str()?.to_owned();
            let known = SHARED.lock().unwrap_or_else(|e| e.into_inner()).plugins.contains_key(&name.to_ascii_lowercase());
            if known {
                crate::game::paper_bridge::fire_custom(game, "core_plugin_enable", json!({ "name": name }));
            }
            json!(known)
        }
        "core_structures" => game.structure_templates.iter().filter(|(_, entry)| entry.is_some()).map(|(name, _)| json!(name)).collect(),
        "core_advancement_place" => {
            let tree = game.advancement_tree();
            let index = *tree.by_id.get(query["id"].as_str()?)?;
            tree.layout().get(index).copied().flatten().map_or(Value::Null, |(x, y)| json!([x, y]))
        }
        "core_stat_get" | "core_stat_set" => {
            let uuid = u128::from_str_radix(query["uuid"].as_str()?, 16).ok()?;
            let target = game.target_by_uuid(uuid)?;
            let kind = query["kind"].as_str()?.trim_start_matches("minecraft:").to_owned();
            let key = query["key"].as_str()?.to_owned();
            let value = query["value"].as_i64();
            let setting = op == "core_stat_set";
            let held = game.for_player(target, |game| {
                let now = game.stats.get(&kind, &key) as i64;
                if setting && let Some(value) = value {
                    game.stats.add(&kind, &key, (value - now) as i32);
                    return value;
                }
                now
            });
            if setting { json!(true) } else { json!(held) }
        }
        "core_ticket" => {
            let mut tickets = TICKETS.lock().unwrap_or_else(|e| e.into_inner());
            let change = query["change"].as_str()?;
            if change == "all" {
                return Some(tickets.iter().flat_map(|((level, x, z), holders)| holders.iter().map(move |plugin| json!([level, x, z, plugin]))).collect());
            }
            let key = (query["dimension"].as_str()?.to_owned(), query["chunk"][0].as_i64()? as i32, query["chunk"][1].as_i64()? as i32);
            let plugin = query["plugin"].as_str()?.to_owned();
            match change {
                "add" => json!(tickets.entry(key).or_default().insert(plugin)),
                "remove" => {
                    let Some(holders) = tickets.get_mut(&key) else {
                        return Some(json!(false));
                    };
                    let changed = holders.remove(&plugin);
                    if holders.is_empty() {
                        tickets.remove(&key);
                    }
                    json!(changed)
                }
                _ => tickets.get(&key).map_or(json!([]), |holders| json!(holders)),
            }
        }
        "core_raise" => {
            let payload = query["payload"].to_string();
            let (answer, carried) = horizon_mod_runtime::lend(game, || horizon_mod_runtime::fire_carried(query["event"].as_str().unwrap_or_default(), &payload));
            json!({ "answer": answer, "payload": carried.and_then(|text| serde_json::from_str::<Value>(&text).ok()) })
        }
        "core_command_call" => {
            let outer = REPLY.lock().unwrap_or_else(|e| e.into_inner()).take();
            let payload = query.to_string();
            horizon_mod_runtime::lend(game, || horizon_mod_runtime::fire("core_command_call", &payload));
            std::mem::replace(&mut *REPLY.lock().unwrap_or_else(|e| e.into_inner()), outer).unwrap_or(Value::Null)
        }
        "core_reply" => {
            *REPLY.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["value"].clone());
            json!(true)
        }
        "core_load_plugin" => {
            let path = std::path::PathBuf::from(query["path"].as_str()?);
            let name = path.file_name()?.to_string_lossy().into_owned();
            let held = horizon_mod_runtime::loaded().iter().any(|(_, file)| file.file_name().is_some_and(|file| file.to_string_lossy().ends_with(&format!("-{name}"))));
            if held || !path.is_file() {
                return Some(Value::Null);
            }
            static LOADS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let index = u128::from(LOADS.fetch_add(1, Ordering::Relaxed));
            LOADING_LATE.store(true, Ordering::Relaxed);
            let name = horizon_mod_runtime::lend(game, || load_library(&path, index));
            LOADING_LATE.store(false, Ordering::Relaxed);
            json!(name)
        }
        "core_loading_late" => json!(LOADING_LATE.load(Ordering::Relaxed)),
        "core_live_property" => live_property(game, query["key"].as_str()?),
        "core_set_idle" => {
            IDLE_MINUTES.store(query["minutes"].as_i64().unwrap_or(0) as i32, Ordering::Relaxed);
            json!(true)
        }
        "core_set_pause_time" => {
            PAUSE_SECONDS.store(query["seconds"].as_i64().unwrap_or(0) as i32, Ordering::Relaxed);
            json!(true)
        }
        "core_allow_pausing" => {
            let plugin = query["plugin"].as_str()?.to_owned();
            let mut held = PAUSE_HELD.lock().unwrap_or_else(|e| e.into_inner());
            if query["allow"].as_bool().unwrap_or(true) {
                held.remove(&plugin);
            } else {
                held.insert(plugin);
            }
            json!(true)
        }
        "core_ticks_per" => json!(TICKS_PER.lock().unwrap_or_else(|e| e.into_inner()).get(query["category"].as_str()?).copied()),
        "core_set_ticks_per" => {
            let category = query["category"].as_str()?.to_owned();
            let ticks = query["ticks"].as_i64().unwrap_or(-1);
            let mut rows = TICKS_PER.lock().unwrap_or_else(|e| e.into_inner());
            if ticks < 0 {
                rows.remove(&category);
            } else {
                rows.insert(category, ticks);
            }
            json!(true)
        }
        "core_raid_omen" => {
            let (Some(id), Some(level)) = (query["id"].as_i64(), query["level"].as_i64()) else {
                return Some(json!(false));
            };
            json!(game.native_in_level(dimension(query), |game| game.raid_set_omen(id as i32, level as i32)).is_some())
        }
        "core_raid_waves" => {
            RAID_WAVES.lock().unwrap_or_else(|e| e.into_inner()).push((query["id"].as_i64()? as i32, query["waves"].as_i64()? as i32));
            json!(true)
        }
        "core_raid_bar_own" => own_raid_bar(game, query).unwrap_or(Value::Null),
        "core_dragon_bar" => own_dragon_bar(game).unwrap_or(Value::Null),
        "core_board_own" => {
            let player = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            let own = query["own"].as_bool().unwrap_or(true);
            let mut boards = OWN_BOARDS.lock().unwrap_or_else(|e| e.into_inner());
            if !own {
                boards.remove(&player);
                return Some(json!(true));
            }
            boards.insert(player);
            drop(boards);
            // What it had of the game's goes first: a name can be on both boards.
            json!(with_hub(game, |game| {
                let nothing = horizon_client::lan::scoreboard::View::default();
                if let Some(guest) = game.lan.as_mut().and_then(|host| host.hub.guests.iter_mut().find(|guest| guest.uuid == player)) {
                    guest.send_all(&guest.scoreboard.changes(&nothing));
                    guest.scoreboard = nothing;
                }
            })
            .is_some())
        }
        "core_board_team" => {
            use horizon_protocol::packets::play::{self, Clientbound as C};
            let player = u128::from_str_radix(&query["uuid"].as_str()?.replace('-', ""), 16).ok()?;
            let method = query["method"].as_i64()? as i8;
            let parameters = matches!(method, 0 | 2).then(|| play::TeamParameters {
                display_name: horizon_client::scoreboard::json_to_tag(&query["display"]),
                prefix: horizon_client::scoreboard::json_to_tag(&query["prefix"]),
                suffix: horizon_client::scoreboard::json_to_tag(&query["suffix"]),
                name_tag_visibility: query["visibility"].as_i64().unwrap_or(0) as i32,
                collision_rule: query["collision"].as_i64().unwrap_or(0) as i32,
                color: query["color"].as_i64().map(|color| color as i32),
                options: query["options"].as_i64().unwrap_or(0) as i8,
            });
            let players = query["players"].as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect();
            let packet = C::SetPlayerTeam(play::SetPlayerTeam { name: query["name"].as_str()?.to_owned(), method, parameters, players });
            with_hub(game, |game| {
                if let Some(guest) = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| guest.uuid == player)) {
                    guest.send(&packet);
                }
            });
            json!(true)
        }
        "core_board_kills" => {
            let log = TEAM_KILLS.lock().unwrap_or_else(|e| e.into_inner());
            let from = query["from"].as_u64().unwrap_or(u64::MAX);
            let first = log.0 - log.1.len() as u64;
            let rows: Vec<Value> = log.1.iter().skip(from.saturating_sub(first) as usize).map(|(criteria, owner)| json!([criteria, owner])).collect();
            json!({ "next": log.0, "rows": rows })
        }
        "core_game_words_off" => {
            horizon_client::lan::commands::GAME_WORDS_OFF.store(true, Ordering::Relaxed);
            json!(true)
        }
        "core_dragon_order" => {
            DRAGON_ORDERS.lock().unwrap_or_else(|e| e.into_inner()).push(query["order"].as_str()?.to_owned());
            json!(true)
        }
        "core_dragon_portal" => {
            *PORTAL_ORDER.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["active"].as_bool().unwrap_or(true));
            json!(true)
        }
        "core_frozen_ticks" => {
            if !game.clock.stepping() {
                return Some(json!(0));
            }
            let (started, time) = *STEP.lock().unwrap_or_else(|e| e.into_inner());
            let count = TICKS.lock().unwrap_or_else(|e| e.into_inner()).count;
            json!((time - (count - started) as i64).max(0))
        }
        "core_next_entity_id" => {
            game.next_entity += 1;
            json!(game.next_entity)
        }
        "core_projectile_block" => {
            let id = query["id"].as_u64()? as u32;
            SHOT_BY.lock().unwrap_or_else(|e| e.into_inner()).get(&id).map_or(Value::Null, |(pos, dimension)| json!({ "pos": pos, "dimension": dimension }))
        }
        "core_enchant_offers" => {
            let stack = stack_from(&query["stack"]);
            let offers = horizon_client::stations::enchant_offers(&game.stations().enchantments, game.items(), query["seed"].as_i64().unwrap_or(0) as i32, query["bookshelves"].as_i64().unwrap_or(0) as i32, stack.as_ref());
            Value::Array((0..3).map(|slot| offers.clues[slot].as_ref().filter(|_| offers.costs[slot] > 0).map_or(Value::Null, |(id, level)| json!({ "enchantment": &**id, "level": level, "cost": offers.costs[slot] }))).collect())
        }
        "core_run_captured" => run_captured(game, query["line"].as_str()?),
        "core_links" => {
            let saved = LINKS.lock().unwrap_or_else(|e| e.into_inner()).clone();
            match saved {
                Some(rows) => Value::Array(rows),
                None => configured_links(game),
            }
        }
        "core_links_set" => {
            let rows = query["links"].as_array()?.clone();
            *LINKS.lock().unwrap_or_else(|e| e.into_inner()) = Some(rows);
            LINKS_SENT.lock().unwrap_or_else(|e| e.into_inner()).clear();
            json!(true)
        }
        "core_set_inhabited" => {
            let (Some(key), Some(time)) = (column(&query["chunk"]), query["time"].as_i64()) else {
                return Some(json!(false));
            };
            json!(game
                .native_in_level(dimension(query), |game| {
                    game.world.inhabited.insert(key, time);
                    game.world.modified.insert(key);
                })
                .is_some())
        }
        _ => return None,
    })
}

/// Hot reload (Horizon V2): a plugin library changed in `plugins/` is swapped while the server runs, with the
/// players on. Every second the folder is looked at; a file whose size and time stayed the same for two looks
/// (its copy is done) is swapped at the tick's end, when no listener runs: the plugin and every plugin that
/// depends on it (`depend`, `softdepend`, which may hold its objects) go off in reverse load order
/// (`onDisable`, their tasks, listeners and words with them), leave the event routes, and come back from a
/// fresh copy of their files in load order (`onLoad`, `onEnable`). A new file is loaded, a removed one turned
/// off. The libraries swapped out stay loaded (`horizon_mod_runtime::retire`): an async task of theirs that is
/// still finishing never runs code that is gone. `HORIZON_HOT_RELOAD=0` turns the watching off; `/hotswap
/// NAME` (operators) swaps one plugin now.
///
/// `/plugin disable NAME` turns a plugin off (and, first, every plugin that depends on it) and moves its file
/// aside to `FILE.disabled`, so it stays off after a restart too; `/plugin enable NAME` moves it back and starts
/// it, with the dependents it took down. Which plugin each set-aside file holds is kept in
/// `.disabled-plugins.json` in the plugins folder.
pub(in crate::game) mod hot_reload {
    use super::*;
    use std::collections::HashMap;
    use std::time::SystemTime;

    struct Seen {
        stamp: (SystemTime, u64),
        /// The looks it stayed as it is since it last changed.
        steady: u32,
        /// The stamp of the copy running now.
        running: Option<(SystemTime, u64)>,
    }

    struct State {
        files: HashMap<String, Seen>,
        /// The plugins running, in load order (the loader's list, then what swaps changed).
        plugins: Vec<crate::game::paper_plugins::Description>,
        ticks: u32,
        asked: Vec<String>,
        /// Swaps whose libraries are being copied and opened on a thread of their own.
        preparing: Vec<Preparing>,
    }

    struct Preparing {
        files: Vec<String>,
        work: std::thread::JoinHandle<Vec<(String, Result<horizon_mod_runtime::Preloaded, String>)>>,
    }

    static STATE: Mutex<Option<State>> = Mutex::new(None);
    static ASKED: Mutex<Vec<String>> = Mutex::new(Vec::new());
    /// `/plugin enable|disable`: (enable, name, who asked).
    static TOGGLES: Mutex<Vec<(bool, String, Option<u128>)>> = Mutex::new(Vec::new());
    /// The dependents a disable took down: (the plugin they wait for, their file, their name). They start
    /// again with it.
    static WAITING: Mutex<Vec<(String, String, String)>> = Mutex::new(Vec::new());
    const DISABLED: &str = ".disabled-plugins.json";

    /// The set-aside plugins: (name, file without `.disabled`), from the record, then any other `*.disabled`
    /// library in the folder (named by its file).
    fn disabled(folder: &std::path::Path) -> Vec<(String, String)> {
        let record: Vec<(String, String)> = std::fs::read_to_string(folder.join(DISABLED))
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Map<String, Value>>(&text).ok())
            .map(|map| map.into_iter().filter_map(|(name, file)| Some((name, file.as_str()?.to_owned()))).collect())
            .unwrap_or_default();
        let mut out: Vec<(String, String)> = record.into_iter().filter(|(_, file)| folder.join(format!("{file}.disabled")).is_file()).collect();
        for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(file) = name.strip_suffix(".disabled") else { continue };
            let library = [".dll", ".so", ".dylib"].iter().any(|e| file.ends_with(e));
            if library && !out.iter().any(|(_, f)| f == file) {
                let stem = file.rsplit_once('.').map_or(file, |(s, _)| s);
                out.push((stem.trim_start_matches("lib").to_owned(), file.to_owned()));
            }
        }
        out
    }

    fn save_disabled(folder: &std::path::Path, list: &[(String, String)]) {
        let map: serde_json::Map<String, Value> = list.iter().map(|(n, f)| (n.clone(), json!(f))).collect();
        if let Err(why) = std::fs::write(folder.join(DISABLED), serde_json::to_string_pretty(&map).unwrap_or_default()) {
            log::warn!("[HotReload] could not write {DISABLED}: {why}");
        }
    }

    /// `/plugin enable|disable NAME`: done at this tick's end; `who` is told how it went.
    pub(in crate::game) fn ask_toggle(enable: bool, name: &str, who: Option<u128>) {
        TOGGLES.lock().unwrap_or_else(|e| e.into_inner()).push((enable, name.to_owned(), who));
    }

    /// `/plugin list`: what runs (green) and what is set aside (red).
    pub(in crate::game) fn listing() -> Vec<String> {
        let Some(plugins) = crate::game::paper_plugins::get().filter(|p| p.dedicated) else {
            return vec!["\u{a7}cPlugins can only be turned on and off on a dedicated server.".to_owned()];
        };
        let folder = plugins.folder();
        let mut running: Vec<String> = match STATE.try_lock() {
            Ok(guard) => guard.as_ref().map_or_else(|| plugins.loaded.iter().map(|p| p.name.clone()).collect(), |s| s.plugins.iter().map(|p| p.name.clone()).collect()),
            Err(_) => plugins.loaded.iter().map(|p| p.name.clone()).collect(),
        };
        running.sort_by_key(|n| n.to_lowercase());
        let off: Vec<String> = disabled(&folder).into_iter().map(|(n, _)| n).collect();
        let mut lines = vec![format!("\u{a7}fPlugins: \u{a7}a{} on\u{a7}f, \u{a7}c{} off", running.len(), off.len())];
        if !running.is_empty() {
            lines.push(format!(" \u{a7}8- \u{a7}a{}", running.join("\u{a7}f, \u{a7}a")));
        }
        if !off.is_empty() {
            lines.push(format!(" \u{a7}8- \u{a7}c{}", off.join("\u{a7}f, \u{a7}c")));
        }
        for (waits_for, _, name) in WAITING.lock().unwrap_or_else(|e| e.into_inner()).iter() {
            lines.push(format!(" \u{a7}8- \u{a7}e{name} \u{a7}7(off until {waits_for} is enabled)"));
        }
        lines.push("\u{a7}7/plugin enable|disable|reload <name>".to_owned());
        lines
    }

    /// Turns `name` and its dependents off and sets its file aside.
    fn disable(game: &mut Game<'_>, state: &mut State, folder: &std::path::Path, name: &str) -> String {
        let Some(target) = state.plugins.iter().find(|p| p.name.eq_ignore_ascii_case(name)).cloned() else {
            return match disabled(folder).into_iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) {
                Some((n, _)) => format!("\u{a7}e{n} is already disabled."),
                None => format!("\u{a7}cNo plugin named {name} is running. /plugin list shows them."),
            };
        };
        let (ordered, _) = plan(state, std::slice::from_ref(&target.file));
        for plugin in ordered.iter().rev() {
            turn_off(game, &plugin.name);
        }
        state.plugins.retain(|p| !ordered.iter().any(|o| o.name == p.name));
        let dependents: Vec<&crate::game::paper_plugins::Description> = ordered.iter().filter(|p| p.name != target.name).collect();
        {
            let mut waiting = WAITING.lock().unwrap_or_else(|e| e.into_inner());
            for d in &dependents {
                waiting.push((target.name.clone(), d.file.clone(), d.name.clone()));
            }
        }
        state.files.remove(&target.file);
        let aside = folder.join(format!("{}.disabled", target.file));
        let moved = std::fs::rename(folder.join(&target.file), &aside);
        let mut list = disabled(folder);
        if moved.is_ok() {
            list.retain(|(_, f)| *f != target.file);
            list.push((target.name.clone(), target.file.clone()));
            save_disabled(folder, &list);
        }
        log::info!("[HotReload] {} disabled by command", target.full_name());
        let mut text = format!("\u{a7}aDisabled {}.", target.name);
        if !dependents.is_empty() {
            let names: Vec<&str> = dependents.iter().map(|d| d.name.as_str()).collect();
            text += &format!(" \u{a7}eAlso turned off, as they depend on it: {} (back with /plugin enable {}).", names.join(", "), target.name);
        }
        if let Err(why) = moved {
            text += &format!(" \u{a7}cIts file could not be set aside ({why}): it comes back on at the next restart.");
        }
        text
    }

    /// Puts `name`'s file back and starts it, with the dependents its disable took down.
    fn enable(state: &mut State, folder: &std::path::Path, name: &str) -> String {
        if let Some(p) = state.plugins.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
            return format!("\u{a7}e{} is already enabled.", p.name);
        }
        let mut list = disabled(folder);
        let wanted = name.to_lowercase();
        let found = list.iter().position(|(n, _)| n.to_lowercase() == wanted).or_else(|| list.iter().position(|(_, f)| f.to_lowercase().contains(&wanted)));
        let Some(at) = found else {
            return format!("\u{a7}cNo disabled plugin named {name}. /plugin list shows them.");
        };
        let (plugin, file) = list.remove(at);
        if let Err(why) = std::fs::rename(folder.join(format!("{file}.disabled")), folder.join(&file)) {
            return format!("\u{a7}cCould not put {file} back: {why}");
        }
        save_disabled(folder, &list);
        let mut files = vec![file];
        WAITING.lock().unwrap_or_else(|e| e.into_inner()).retain(|(waits_for, dep, _)| {
            let mine = waits_for.eq_ignore_ascii_case(&plugin);
            if mine && folder.join(dep).is_file() && !files.contains(dep) {
                files.push(dep.clone());
            }
            !mine
        });
        // Known to the folder watcher as running, so it doesn't load them a second time.
        for f in &files {
            if let Some(now) = stamp(&folder.join(f)) {
                state.files.insert(f.clone(), Seen { stamp: now, steady: 2, running: Some(now) });
            }
        }
        log::info!("[HotReload] {plugin} enabled by command");
        let more = if files.len() > 1 { format!(" (and {} that depend on it)", files.len() - 1) } else { String::new() };
        prepare(state, folder, files);
        format!("\u{a7}aEnabling {plugin}{more}: it starts in a moment.")
    }

    fn stamp(path: &std::path::Path) -> Option<(SystemTime, u64)> {
        let meta = std::fs::metadata(path).ok()?;
        Some((meta.modified().ok()?, meta.len()))
    }

    pub(in crate::game) fn on() -> bool {
        std::env::var("HORIZON_HOT_RELOAD").map_or(true, |v| v != "0")
    }

    /// `/hotswap NAME`: swap that plugin at this tick's end, changed or not.
    pub(in crate::game) fn ask(name: &str) {
        ASKED.lock().unwrap_or_else(|e| e.into_inner()).push(name.to_owned());
    }

    /// At each tick's end.
    pub(in crate::game) fn tick(game: &mut Game<'_>) {
        let Some(plugins) = crate::game::paper_plugins::get().filter(|p| p.dedicated) else {
            return;
        };
        let folder = plugins.folder();
        let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
        let state = guard.get_or_insert_with(|| {
            let files = plugins
                .loaded
                .iter()
                .filter_map(|p| {
                    let now = stamp(&folder.join(&p.file))?;
                    Some((p.file.clone(), Seen { stamp: now, steady: 2, running: Some(now) }))
                })
                .collect();
            State { files, plugins: plugins.loaded.clone(), ticks: 0, asked: Vec::new(), preparing: Vec::new() }
        });
        state.ticks += 1;
        state.asked.extend(std::mem::take(&mut *ASKED.lock().unwrap_or_else(|e| e.into_inner())));
        let asked: Vec<String> = std::mem::take(&mut state.asked);
        let toggles = std::mem::take(&mut *TOGGLES.lock().unwrap_or_else(|e| e.into_inner()));
        for (on, name, who) in toggles {
            let text = if on { enable(state, &folder, &name) } else { disable(game, state, &folder, &name) };
            crate::game::paper_bridge::tell_now(game, who, &text);
        }
        let mut changed: Vec<String> = asked
            .iter()
            .filter_map(|name| state.plugins.iter().find(|p| p.name.eq_ignore_ascii_case(name)).map(|p| p.file.clone()))
            .collect();
        if on() && state.ticks % 20 == 0 {
            let present: Vec<(String, (SystemTime, u64))> = std::fs::read_dir(&folder)
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    let library = path.is_file() && path.extension().is_some_and(|e| e == "dll" || e == "so" || e == "dylib");
                    if !library {
                        return None;
                    }
                    Some((entry.file_name().to_string_lossy().into_owned(), stamp(&path)?))
                })
                .collect();
            for (file, now) in &present {
                let seen = state.files.entry(file.clone()).or_insert(Seen { stamp: *now, steady: 0, running: None });
                if seen.stamp == *now {
                    seen.steady += 1;
                } else {
                    seen.stamp = *now;
                    seen.steady = 0;
                }
                if seen.steady == 2 && seen.running != Some(*now) && !changed.contains(file) {
                    changed.push(file.clone());
                }
            }
            // A file taken away: its plugin goes off.
            let gone: Vec<String> = state.files.keys().filter(|f| !present.iter().any(|(p, _)| p == *f)).cloned().collect();
            for file in gone {
                state.files.remove(&file);
                if let Some(at) = state.plugins.iter().position(|p| p.file == file) {
                    let plugin = state.plugins.remove(at);
                    log::info!("[HotReload] {} was removed: turning it off", plugin.full_name());
                    turn_off(game, &plugin.name);
                }
            }
        }
        if !changed.is_empty() {
            prepare(state, &folder, changed);
        }
        // A swap whose libraries are open goes in at this tick's end: the server thread only swaps and starts.
        let mut ready = Vec::new();
        let mut index = 0;
        while index < state.preparing.len() {
            if state.preparing[index].work.is_finished() {
                ready.push(state.preparing.remove(index));
            } else {
                index += 1;
            }
        }
        for preparing in ready {
            let libraries = preparing.work.join().unwrap_or_default();
            swap(game, state, &folder, &preparing.files, libraries);
        }
    }

    /// The plugins of the changed files and whoever depends on them, in load order, and the files to load
    /// for them (theirs, then new files).
    fn plan(state: &State, files: &[String]) -> (Vec<crate::game::paper_plugins::Description>, Vec<String>) {
        let mut names: Vec<String> = state.plugins.iter().filter(|p| files.contains(&p.file)).map(|p| p.name.clone()).collect();
        loop {
            let more: Vec<String> = state
                .plugins
                .iter()
                .filter(|p| !names.contains(&p.name) && p.depend.iter().chain(&p.softdepend).any(|d| names.iter().any(|n| n.eq_ignore_ascii_case(d))))
                .map(|p| p.name.clone())
                .collect();
            if more.is_empty() {
                break;
            }
            names.extend(more);
        }
        let ordered: Vec<crate::game::paper_plugins::Description> = state.plugins.iter().filter(|p| names.contains(&p.name)).cloned().collect();
        let mut to_load: Vec<String> = ordered.iter().map(|p| p.file.clone()).collect();
        for file in files {
            if !to_load.contains(file) {
                to_load.push(file.clone());
            }
        }
        (ordered, to_load)
    }

    /// Copies and opens the libraries of a swap on a thread of its own.
    fn prepare(state: &mut State, folder: &std::path::Path, files: Vec<String>) {
        let (_, to_load) = plan(state, &files);
        let (folder, load) = (folder.to_owned(), folder.join(".hot"));
        let work = std::thread::Builder::new().name("horizon-hot-reload".into()).spawn(move || {
            to_load.into_iter().map(|file| {
                let opened = horizon_mod_runtime::preload(&folder.join(&file), &load);
                (file, opened)
            }).collect()
        });
        match work {
            Ok(work) => state.preparing.push(Preparing { files, work }),
            Err(error) => log::error!("[HotReload] couldn't start loading: {error}"),
        }
    }

    fn turn_off(game: &mut Game<'_>, name: &str) {
        log::info!("[{name}] Disabling {name}");
        crate::game::paper_bridge::fire_custom(game, "core_plugin_disable", json!({ "name": name }));
        if let Err(why) = horizon_mod_runtime::retire(name) {
            log::error!("[HotReload] {name} could not leave the event routes: {why}");
        }
    }

    fn swap(game: &mut Game<'_>, state: &mut State, folder: &std::path::Path, files: &[String], libraries: Vec<(String, Result<horizon_mod_runtime::Preloaded, String>)>) {
        let started = std::time::Instant::now();
        let (ordered, _) = plan(state, files);
        for plugin in ordered.iter().rev() {
            turn_off(game, &plugin.name);
        }
        let mut swapped = Vec::new();
        for (file, opened) in libraries {
            let before = ordered.iter().find(|p| p.file == file).map(crate::game::paper_plugins::Description::full_name);
            let library = match opened {
                Ok(library) => library,
                Err(error) => {
                    log::error!("[HotReload] {file} did not load ({error}): its plugin stays off");
                    state.plugins.retain(|p| p.file != file);
                    continue;
                }
            };
            let Some(description) = horizon_mod_runtime::lend(game, || start_preloaded(library, &file)) else {
                log::error!("[HotReload] {file} did not start: its plugin stays off");
                state.plugins.retain(|p| p.file != file);
                continue;
            };
            if let Some(seen) = state.files.get_mut(&file) {
                seen.running = stamp(&folder.join(&file));
            }
            let now = description.full_name();
            swapped.push(match before {
                Some(before) if before != now => format!("{before} -> {now}"),
                Some(_) => now,
                None => format!("{now} (new)"),
            });
            match state.plugins.iter_mut().find(|p| p.name == description.name) {
                Some(slot) => *slot = description,
                None => state.plugins.push(description),
            }
        }
        horizon_mod_runtime::adopt_late();
        log::info!(
            "[HotReload] swapped {} in {} ms on the server thread with {} players on: {}",
            swapped.len(),
            started.elapsed().as_millis(),
            game.lan.as_ref().map_or(0, |host| host.hub.guests.len()),
            swapped.join(", ")
        );
    }
}
