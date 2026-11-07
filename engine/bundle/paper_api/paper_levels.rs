// What a plugin asks of a level and the blocks in it, past what a command can tell it.
use crate::game::Game;
use horizon_client::mobs::spawn_rules::SpawnLevel;
use serde_json::{Value, json};

/// `{ "op": "level", "dimension": ... }`: the facts of `org.bukkit.World`.
pub(in crate::game) fn level(game: &mut Game<'_>, dimension: &str) -> Value {
    game.native_in_level(dimension, |game| {
        let weather = &game.world.weather;
        let border = &game.border;
        json!({
            "seed": game.level.seed,
            "spawn": game.level.world_spawn,
            "spawn_angle": game.level.world_spawn_angle,
            "difficulty": game.ui.difficulty,
            "hardcore": game.level.hardcore,
            "day_time": game.world.day_time,
            "game_time": game.world.game_time,
            "raining": weather.raining,
            "thundering": weather.thundering,
            "clear_time": weather.clear_time,
            "rain_time": weather.rain_time,
            "thunder_time": weather.thunder_time,
            "min_y": game.world.bottom_y(),
            "max_y": game.world.top_y(),
            "chunks": game.world.chunks.keys().map(|(x, z)| [*x, *z]).collect::<Vec<_>>(),
            "forced": game.streaming.forced.iter().copied().map(|(x, z)| [x, z]).collect::<Vec<_>>(),
            "border": {
                "center": border.center(),
                "size": border.size(),
                "target_size": border.target_size(),
                "remaining_ticks": border.remaining_ticks(),
                "damage_per_block": border.damage_per_block,
                "safe_zone": border.safe_zone,
                "warning_distance": border.warning_distance,
                "warning_time": border.warning_time,
                "absolute_max": border.absolute_max_size(),
            },
        })
    })
    .unwrap_or(Value::Null)
}

/// `{ "op": "block_facts", ... }`: what `org.bukkit.block.Block` asks beyond its type.
pub(in crate::game) fn block_facts(game: &mut Game<'_>, dimension: &str, pos: [i32; 3]) -> Value {
    game.native_in_level(dimension, |game| {
        let state = game.world.block(pos[0], pos[1], pos[2]);
        let block_pos = horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]);
        let signal = game.with_level(|behaviours, level, ticks| behaviours.best_neighbor_signal(level, ticks, block_pos));
        let climate = game.world.chunks.get(&(pos[0] >> 4, pos[2] >> 4)).map(|chunk| {
            let biome = &game.res.biomes().list[chunk.biome(pos[0], pos[1], pos[2]) as usize];
            (biome.temperature, biome.downfall)
        });
        let facts = game.res.facts().get(state);
        json!({
            "signal": signal,
            "temperature": climate.map(|climate| climate.0),
            "downfall": climate.map(|climate| climate.1),
            "name": &*facts.name,
            "item": &*facts.item,
            "emission": facts.emission,
            "hardness": facts.hardness,
            "explosion_resistance": facts.explosion_resistance,
            "friction": facts.friction,
            "requires_tool": facts.requires_tool,
            "replaceable": facts.replaceable,
            "random_ticks": facts.random_ticks,
            "block_entity": facts.block_entity,
            "full_collision": facts.full_collision,
            "solid": facts.legacy_solid,
            "liquid": facts.fluid != horizon_client::blocks::Fluid::None,
            "waterlogged": facts.waterlogged,
            "occluding": facts.can_occlude,
            "map_color": facts.map_color,
            "shape": &*facts.outline,
            "biome": game.world.chunks.get(&(pos[0] >> 4, pos[2] >> 4)).map(|chunk| game.res.biomes().list[chunk.biome(pos[0], pos[1], pos[2]) as usize].name.clone()),
            "sky_light": game.mob_world_base(&[]).sky_light(pos[0], pos[1], pos[2]),
            "block_light": game.mob_world_base(&[]).block_light(pos[0], pos[1], pos[2]),
        })
    })
    .unwrap_or(Value::Null)
}

/// `{ "op": "game_rule", "name": ... }`, as `/gamerule` reads it.
pub(in crate::game) fn game_rule(game: &Game<'_>, name: &str) -> Value {
    let name = name.trim_start_matches("minecraft:");
    game.ui.rules.get(name).map_or(Value::Null, |value| json!(value.to_string()))
}

/// `{ "op": "tick_times" }`: milliseconds of the last ticks, for `getTPS` and `getAverageTickTime`.
pub(in crate::game) fn tick_times(game: &Game<'_>) -> Value {
    json!({ "average": game.debug_charts.smoothed_tick_ms, "ticks": game.ticks })
}

/// The values one property of a block takes, in the order the game lists them.
pub(in crate::game) fn property_values(game: &Game<'_>, block: &str, key: &str) -> Value {
    let name = block.trim_start_matches("minecraft:");
    let registry = &game.pack.assets.registry;
    let mut values: Vec<String> = Vec::new();
    for state in (0..registry.len()).filter_map(|id| registry.state(horizon_world::registry::BlockStateId(id as _))) {
        if state.name.trim_start_matches("minecraft:") == name
            && let Some(value) = state.properties.get(key)
            && !values.contains(value)
        {
            values.push(value.clone());
        }
    }
    json!(values)
}

/// `{ "op": "scoreboard" }`: objectives, scores, display slots and teams, as commands would list them.
pub(in crate::game) fn scoreboard(game: &Game<'_>) -> Value {
    let board = &game.score.board;
    json!({
        "objectives": board.objectives.iter().map(|objective| json!({ "name": objective.name, "criteria": objective.criteria, "display_name": objective.display_name, "display_auto_update": objective.display_auto_update })).collect::<Vec<_>>(),
        "scores": board.scores.iter().map(|(entry, scores)| (entry.clone(), scores.iter().map(|(objective, score)| (objective.clone(), json!({ "value": score.value, "locked": score.locked }))).collect::<serde_json::Map<_, _>>())).collect::<std::collections::BTreeMap<_, _>>(),
        "display": board.display,
        "teams": board.teams.iter().map(|team| json!({ "name": team.name, "display_name": team.display_name, "color": team.color, "allow_friendly_fire": team.allow_friendly_fire, "see_friendly_invisibles": team.see_friendly_invisibles, "prefix": team.prefix, "suffix": team.suffix, "players": team.players.iter().collect::<Vec<_>>() })).collect::<Vec<_>>(),
    })
}

/// `{ "op": "boss_bar", "id": ... }`.
pub(in crate::game) fn boss_bar(game: &Game<'_>, id: &str) -> Value {
    let id = if id.contains(':') { id.to_owned() } else { format!("minecraft:{id}") };
    game.bosses.bars.get(&id).map_or(Value::Null, |bar| json!({ "name": bar.name, "color": bar.color, "style": bar.style, "value": bar.value, "max": bar.max, "visible": bar.visible, "players": bar.players.iter().map(|words| format!("{:08x}{:08x}{:08x}{:08x}", words[0] as u32, words[1] as u32, words[2] as u32, words[3] as u32)).collect::<Vec<_>>() }))
}

fn tag_file(game: &Game<'_>, kind: &str, name: &str) -> Vec<String> {
    let name = name.trim_start_matches('#').trim_start_matches("minecraft:");
    let path = game.options.resources.join(format!("data/minecraft/tags/{kind}/{name}.json"));
    let file = serde_json::from_slice::<Value>(&std::fs::read(path).unwrap_or_default()).unwrap_or_default();
    let mut out = Vec::new();
    for entry in file["values"].as_array().into_iter().flatten() {
        let text = entry.as_str().or_else(|| entry["id"].as_str()).unwrap_or_default();
        if text.starts_with('#') {
            out.extend(tag_file(game, kind, text));
        } else {
            out.push(text.to_owned());
        }
    }
    out
}

/// `{ "op": "tag", "kind": "block", "name": ... }`: the ids in a tag, with the tags inside it opened.
pub(in crate::game) fn tag_members(game: &Game<'_>, kind: &str, name: &str) -> Value {
    json!(tag_file(game, kind, name))
}

/// `{ "op": "tags", "kind": "block" }`: every tag of a kind.
pub(in crate::game) fn tag_names(game: &Game<'_>, kind: &str) -> Value {
    let folder = game.options.resources.join(format!("data/minecraft/tags/{kind}"));
    let names: Vec<String> = std::fs::read_dir(folder).into_iter().flatten().flatten().filter_map(|file| Some(format!("minecraft:{}", file.path().file_stem()?.to_str()?))).collect();
    json!(names)
}

/// `{ "op": "data_file", "path": "enchantment/sharpness" }`: a file of the game's own data: what a
/// registry's entries say about themselves.
pub(in crate::game) fn data_file(game: &Game<'_>, path: &str) -> Value {
    if path.contains("..") {
        return Value::Null;
    }
    let file = game.options.resources.join(format!("data/minecraft/{path}.json"));
    serde_json::from_slice(&std::fs::read(file).unwrap_or_default()).unwrap_or(Value::Null)
}

/// `{ "op": "data_list", "path": "enchantment" }`: the entries of a registry folder.
pub(in crate::game) fn data_list(game: &Game<'_>, path: &str) -> Value {
    if path.contains("..") {
        return Value::Null;
    }
    let folder = game.options.resources.join(format!("data/minecraft/{path}"));
    let names: Vec<String> = std::fs::read_dir(folder).into_iter().flatten().flatten().filter_map(|file| Some(format!("minecraft:{}", file.path().file_stem()?.to_str()?))).collect();
    json!(names)
}

/// `{ "op": "set_block", ... }`: at once, as `Block.setBlockData` is, with neighbours told.
pub(in crate::game) fn set_block(game: &mut Game<'_>, dimension: &str, pos: [i32; 3], block: &str) -> bool {
    let Ok(state) = super::render_control::parse_state(game.pack.assets.registry, block) else {
        return false;
    };
    game.native_in_level(dimension, |game| game.native_set_block(game.dim.id(), pos, state, 3)).unwrap_or(false)
}

/// `{ "op": "bone_meal", ... }`: `Block.applyBoneMeal`, as a dispenser uses it.
pub(in crate::game) fn bone_meal(game: &mut Game<'_>, dimension: &str, pos: [i32; 3]) -> bool {
    let block_pos = horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]);
    game.native_in_level(dimension, |game| game.with_level(|behaviours, level, ticks| behaviours.use_bone_meal(level, ticks, block_pos, horizon_core::geometry::Direction::Up))).unwrap_or(false)
}

/// `{ "op": "tick_state" }`, as `/tick query` reads it.
pub(in crate::game) fn tick_state(game: &Game<'_>) -> Value {
    json!({ "rate": game.clock.rate(), "sprinting": game.clock.sprinting(), "frozen": game.clock.frozen(), "stepping": game.clock.stepping() })
}

/// `{ "op": "state_names" }`: every block state's text by its id (`minecraft:oak_stairs[facing=north,...]`), for a
/// plugin that reads and writes blocks in bulk by id (FastAsyncWorldEdit's chunk queue).
pub(in crate::game) fn state_names(game: &Game<'_>) -> Value {
    let registry = game.pack.assets.registry;
    let names: Vec<String> = (0..registry.len() as u32)
        .map(|id| {
            registry.state(horizon_world::registry::BlockStateId(id)).map_or_else(String::new, |state| {
                if state.properties.is_empty() {
                    return state.name.clone();
                }
                let properties: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
                format!("{}[{}]", state.name, properties.join(","))
            })
        })
        .collect();
    json!(names)
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 (with `=` padding) of bytes: how bulk block data crosses the JSON query in one string.
pub(in crate::game) fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

pub(in crate::game) fn base64_decode(text: &str) -> Vec<u8> {
    let mut table = [255u8; 256];
    for (i, c) in B64.iter().enumerate() {
        table[*c as usize] = i as u8;
    }
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let (mut buffer, mut bits) = (0u32, 0u32);
    for c in text.bytes() {
        let v = table[c as usize];
        if v == 255 {
            continue;
        }
        buffer = (buffer << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    out
}

/// `{ "op": "blocks_write", "dimension", "data" }`: `data` is base64 of packed little-endian records `[x, y, z,
/// state id]` (i32, i32, i32, u32), written as the world's own storage writes (`World::set`: remesh, relight, saving
/// and the players' section updates follow; no neighbour updates, as FAWE's fast mode). One query for a whole edit
/// instead of one JSON query per block. Answers how many blocks changed.
pub(in crate::game) fn blocks_write(game: &mut Game<'_>, dimension: &str, data: &str) -> Value {
    let bytes = base64_decode(data);
    let states = game.pack.assets.registry.len() as u32;
    game.native_in_level(dimension, |game| {
        let mut changed = 0u64;
        for record in bytes.chunks_exact(16) {
            let word = |i: usize| u32::from_le_bytes([record[i], record[i + 1], record[i + 2], record[i + 3]]);
            let (x, y, z, state) = (word(0) as i32, word(4) as i32, word(8) as i32, word(12));
            if state >= states {
                continue;
            }
            if game.world.block(x, y, z).0 != state && game.world.set(x, y, z, horizon_world::registry::BlockStateId(state)).is_ok() {
                changed += 1;
            }
        }
        json!(changed)
    })
    .unwrap_or(json!(0))
}

/// `{ "op": "blocks_read", "dimension", "chunk": [x, z] }`: every block state id of a loaded chunk, bottom up (y,
/// then z, then x), as `{ bottom, height, data }` with `data` base64 of little-endian u32 ids; null when the chunk
/// isn't loaded.
pub(in crate::game) fn blocks_read(game: &mut Game<'_>, dimension: &str, chunk: [i32; 2]) -> Value {
    game.native_in_level(dimension, |game| {
        if !game.world.loaded(chunk[0] << 4, chunk[1] << 4) {
            return Value::Null;
        }
        let (bottom, top) = (game.world.bottom_y(), game.world.top_y());
        let mut bytes = Vec::with_capacity(((top - bottom) * 256 * 4) as usize);
        for y in bottom..top {
            for z in 0..16 {
                for x in 0..16 {
                    bytes.extend_from_slice(&game.world.block((chunk[0] << 4) + x, y, (chunk[1] << 4) + z).0.to_le_bytes());
                }
            }
        }
        json!({ "bottom": bottom, "height": top - bottom, "data": base64_encode(&bytes) })
    })
    .unwrap_or(Value::Null)
}

/// The chunks of a box resident now (FAWE edits far from players): the missing ones forced and loaded as vanilla's
/// `getChunk` does (`load_test_chunks`), then released, so they unload again in their own time. False if they
/// didn't all arrive within the deadline.
fn resident(game: &mut Game<'_>, min: [i32; 3], max: [i32; 3]) -> bool {
    let (x0, x1, z0, z1) = (min[0] >> 4, max[0] >> 4, min[2] >> 4, max[2] >> 4);
    let missing = |g: &Game<'_>| (x0..=x1).flat_map(|x| (z0..=z1).map(move |z| (x, z))).filter(|c| !g.world.chunks.contains_key(c)).collect::<Vec<_>>();
    let wanted = missing(game);
    if wanted.is_empty() {
        return true;
    }
    // A region's thread holds only its region's chunks and can't load others: waiting here (a minute) for one
    // that another thread holds took the whole server down on the watchdog. Not there is the answer.
    if game.region_worker {
        return false;
    }
    let added: Vec<(i32, i32)> = wanted.iter().copied().filter(|c| game.streaming.forced.insert(*c)).collect();
    game.streaming.rank_forced(x0, z0, x1, z1);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !missing(game).is_empty() && std::time::Instant::now() < deadline {
        let _ = game.streaming.update(&mut game.world);
        game.streaming.wait(std::time::Duration::from_millis(2));
    }
    for chunk in added {
        game.streaming.forced.remove(&chunk);
    }
    missing(game).is_empty()
}

fn corners(query: &Value) -> Option<([i32; 3], [i32; 3])> {
    let at = |v: &Value| -> Option<[i32; 3]> { Some([v[0].as_i64()? as i32, v[1].as_i64()? as i32, v[2].as_i64()? as i32]) };
    let (a, b) = (at(&query["min"])?, at(&query["max"])?);
    Some(([a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])], [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])]))
}

/// Every block of a box in (y, z, x) order, the world's height clamped.
fn each_in(min: [i32; 3], max: [i32; 3], bottom: i32, top: i32, mut f: impl FnMut(i32, i32, i32)) {
    for y in min[1].max(bottom)..=max[1].min(top - 1) {
        for z in min[2]..=max[2] {
            for x in min[0]..=max[0] {
                f(x, y, z);
            }
        }
    }
}

/// Writes a box chunk by chunk: each chunk copied on write once, its blocks set in place, then what `World::set`
/// does per block done per chunk and section (players' section updates, block entities, watched blocks, saving,
/// remeshing, light). `pick(was, i)` is the state for the box's `i`th block ((y, z, x) order, height clamped), or
/// None to leave it; `before` gets every block of the box as it was. Answers how many blocks changed.
fn write_box(game: &mut Game<'_>, min: [i32; 3], max: [i32; 3], mut pick: impl FnMut(u32, usize) -> Option<u32>, mut before: Option<&mut Vec<u32>>) -> u64 {
    let (bottom, top) = (game.world.bottom_y(), game.world.top_y());
    let (y0, y1) = (min[1].max(bottom), max[1].min(top - 1));
    if y0 > y1 {
        return 0;
    }
    let (dx, dz) = ((max[0] - min[0] + 1) as usize, (max[2] - min[2] + 1) as usize);
    if let Some(before) = before.as_deref_mut() {
        before.clear();
        before.resize(dx * dz * (y1 - y0 + 1) as usize, 0);
    }
    let mut total = 0u64;
    let mut resend: Vec<(i32, i32)> = Vec::new();
    for cx in (min[0] >> 4)..=(max[0] >> 4) {
        for cz in (min[2] >> 4)..=(max[2] >> 4) {
            let (xa, xb) = (min[0].max(cx << 4), max[0].min((cx << 4) + 15));
            let (za, zb) = (min[2].max(cz << 4), max[2].min((cz << 4) + 15));
            let mut changes: Vec<((i32, i32, i32), u32, u32)> = Vec::new();
            let (lit, min_y, sections) = {
                let Some(shared) = game.world.chunks.get_mut(&(cx, cz)) else { continue };
                let lit = shared.lit;
                let chunk = std::sync::Arc::make_mut(shared);
                let (min_y, sections) = (chunk.min_y(), chunk.sections.len() as i32);
                for y in y0..=y1 {
                    for z in za..=zb {
                        for x in xa..=xb {
                            let index = ((y - y0) as usize * dz + (z - min[2]) as usize) * dx + (x - min[0]) as usize;
                            let was = chunk.get(x & 15, y, z & 15);
                            if let Some(before) = before.as_deref_mut() {
                                before[index] = was;
                            }
                            if let Some(new) = pick(was, index) {
                                if new != was {
                                    chunk.set(x & 15, y, z & 15, new);
                                    changes.push(((x, y, z), was, new));
                                }
                            }
                        }
                    }
                }
                (lit, min_y, sections)
            };
            if changes.is_empty() {
                continue;
            }
            total += changes.len() as u64;
            // A chunk changed in bulk is sent again whole (as FAWE does) instead of block by block, and mobs aren't
            // told of each block's collision (FAWE's edits don't tell them either). Measured 11-04 on 1M blocks:
            // per-block logs cost ~0.4 s a tick (pathfinding, sorting the log); section updates reached the client
            // later (0.3-0.48 s) than whole chunks (0.15-0.27 s, Java FAWE 0.33-0.51 s).
            let bulk = changes.len() > BULK;
            if bulk {
                resend.push((cx, cz));
            }
            let world = &mut game.world;
            let mut touched: Vec<i32> = Vec::new();
            for &(pos, was, new) in &changes {
                world.block_entities.state_changed(pos, was, new);
                if world.watched.get(was as usize).copied().unwrap_or(false) || world.watched.get(new as usize).copied().unwrap_or(false) {
                    world.watched_writes.push((pos, was, new));
                }
                if !bulk {
                    if let Some(log) = &mut world.block_log {
                        log.push(pos);
                    }
                    if let Some(writes) = &mut world.path_writes {
                        writes.push((pos, was, new));
                    }
                }
                let section = (pos.1 - min_y) >> 4;
                if touched.last() != Some(&section) && !touched.contains(&section) {
                    touched.push(section);
                }
            }
            world.revision = world.revision.wrapping_add(changes.len() as u64);
            world.modified.insert((cx, cz));
            // The sections written, the ones above and below, and the same ones of the columns around (the mesher's
            // 3x3x3 neighbourhood): more than a border write needs, never less.
            for &sy in &touched {
                for (ox, oz) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)] {
                    for s in [sy - 1, sy, sy + 1] {
                        if s >= 0 && s < sections {
                            world.dirty_sections.insert((cx + ox, s, cz + oz));
                        }
                    }
                }
            }
            // A few blocks are relit one by one, as `World::set` does; a whole chunk's worth at once is cheaper
            // relit as a column.
            if lit && changes.len() <= 4096 {
                world.light_edits.extend(changes.iter().copied());
            } else {
                for ox in -1..=1 {
                    for oz in -1..=1 {
                        world.dirty_light.insert((cx + ox, cz + oz));
                    }
                }
            }
        }
    }
    if !resend.is_empty()
        && let Some(host) = game.lan.as_mut()
    {
        let here: Vec<usize> = (0..host.hub.guests.len()).filter(|&i| host.hub.here(&host.hub.guests[i])).collect();
        for i in here {
            let guest = &mut host.hub.guests[i];
            for key in &resend {
                if guest.sent_chunks.remove(key) && !guest.chunk_queue.contains(key) {
                    guest.chunk_queue.insert(0, *key);
                }
            }
        }
    }
    total
}

/// More changed blocks than this in one chunk: the chunk is sent again whole.
const BULK: usize = 1024;

/// One native edit kept for undo and redo (FAWE's history of a //set or //replace).
struct BoxEdit {
    dimension: String,
    min: [i32; 3],
    max: [i32; 3],
    state: u32,
    only: Option<std::collections::HashSet<u32>>,
    except: bool,
    before: Vec<u32>,
    /// A written box's blocks (`box_write` with `keep`), which redo writes again; a fill has none.
    data: Option<Vec<u32>>,
    /// The box's block entities as they were (undo puts them back: a chest's items, a sign's text).
    entities_before: Entities,
    /// The block entities the write placed (redo places them again).
    entities_after: Entities,
}

type Entities = Vec<((i32, i32, i32), horizon_world::nbt::Tag)>;

/// The block entities of a box with their data, as `/data get block` gives it.
fn entities_in(game: &mut Game<'_>, min: [i32; 3], max: [i32; 3]) -> Entities {
    let keys: Vec<(i32, i32, i32)> = game
        .world
        .block_entities
        .keys()
        .filter(|(x, y, z)| *x >= min[0] && *x <= max[0] && *y >= min[1] && *y <= max[1] && *z >= min[2] && *z <= max[2])
        .copied()
        .collect();
    keys.into_iter()
        .filter_map(|k| game.data_of(&crate::game::commands::data::DataRef::Block([k.0, k.1, k.2])).ok().map(|tag| (k, tag)))
        .collect()
}

/// Block entities given their data where the block has one, as `/data merge block` does. Answers how many.
fn entities_put(game: &mut Game<'_>, list: &Entities) -> u64 {
    let mut placed = 0;
    for (k, tag) in list {
        if game.world.block_entities.get(k).is_none() {
            continue;
        }
        let mut tag = tag.clone();
        if let horizon_world::nbt::Tag::Compound(c) = &mut tag {
            c.insert("x".into(), horizon_world::nbt::Tag::Int(k.0));
            c.insert("y".into(), horizon_world::nbt::Tag::Int(k.1));
            c.insert("z".into(), horizon_world::nbt::Tag::Int(k.2));
        }
        if game.set_data_of(&crate::game::commands::data::DataRef::Block([k.0, k.1, k.2]), tag).is_ok() {
            placed += 1;
        }
    }
    placed
}

/// A tag as base64 of named binary NBT (the file format; a plugin reads it as a schematic's).
fn nbt_encode(tag: &horizon_world::nbt::Tag) -> String {
    let named = horizon_world::nbt::NamedTag { name: "".into(), tag: tag.clone() };
    base64_encode(&horizon_world::nbt::write_named(&named, horizon_world::nbt::Limits::default()).unwrap_or_default())
}

fn nbt_decode(text: &str) -> Option<horizon_world::nbt::Tag> {
    horizon_world::nbt::read_named(&base64_decode(text), horizon_world::nbt::Limits::default()).ok().map(|(named, _)| named.tag)
}

/// `"entities": [{ "pos": [x, y, z], "nbt": base64 }]` of a query.
fn entities_of(value: &Value) -> Entities {
    value
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|e| {
                    let pos = e["pos"].as_array()?;
                    let at = (pos.first()?.as_i64()? as i32, pos.get(1)?.as_i64()? as i32, pos.get(2)?.as_i64()? as i32);
                    Some((at, nbt_decode(e["nbt"].as_str()?)?))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Base64 of `(id, count)` little-endian u32 pairs: a box's blocks as runs, a few bytes for what is mostly one block.
fn runs_encode(ids: &[u32]) -> String {
    let mut bytes = Vec::new();
    let mut i = 0;
    while i < ids.len() {
        let id = ids[i];
        let mut n = 1;
        while i + n < ids.len() && ids[i + n] == id {
            n += 1;
        }
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes.extend_from_slice(&(n as u32).to_le_bytes());
        i += n;
    }
    base64_encode(&bytes)
}

fn runs_decode(text: &str) -> Vec<u32> {
    let bytes = base64_decode(text);
    let mut out = Vec::new();
    for pair in bytes.chunks_exact(8) {
        let id = u32::from_le_bytes([pair[0], pair[1], pair[2], pair[3]]);
        let n = u32::from_le_bytes([pair[4], pair[5], pair[6], pair[7]]) as usize;
        out.resize(out.len() + n.min(1 << 28), id);
    }
    out
}

static BOX_EDITS: std::sync::Mutex<Option<std::collections::HashMap<u64, BoxEdit>>> = std::sync::Mutex::new(None);
static NEXT_BOX_EDIT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// The answer a request with this `token` already got: a plugin's request from another thread can reach the game
/// twice, and an edit must happen once.
static ANSWERED: std::sync::Mutex<std::collections::VecDeque<(String, Value)>> = std::sync::Mutex::new(std::collections::VecDeque::new());

fn once(query: &Value, work: impl FnOnce() -> Value) -> Value {
    let Some(token) = query["token"].as_str().map(str::to_owned) else { return work() };
    if let Some((_, answer)) = ANSWERED.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|(t, _)| *t == token) {
        return answer.clone();
    }
    let answer = work();
    let mut answered = ANSWERED.lock().unwrap_or_else(|e| e.into_inner());
    answered.push_back((token, answer.clone()));
    while answered.len() > 64 {
        answered.pop_front();
    }
    answer
}

fn fill(game: &mut Game<'_>, edit: &BoxEdit, before: Option<&mut Vec<u32>>, entities: Option<&mut Entities>) -> Option<u64> {
    let (state, only, except) = (edit.state, &edit.only, edit.except);
    let states = game.pack.assets.registry.len() as u32;
    game.native_in_level(&edit.dimension, |game| {
        if !resident(game, edit.min, edit.max) {
            return None;
        }
        if let Some(entities) = entities {
            *entities = entities_in(game, edit.min, edit.max);
        }
        if let Some(data) = &edit.data {
            let changed = write_box(game, edit.min, edit.max, |_, i| data.get(i).copied().filter(|id| *id < states), before);
            entities_put(game, &edit.entities_after);
            return Some(changed);
        }
        Some(write_box(game, edit.min, edit.max, |was, _| only.as_ref().is_none_or(|set| set.contains(&was) != except).then_some(state), before))
    })
    .flatten()
}

/// `{ "op": "box_fill", "dimension", "min", "max", "state", "only": [ids]?, "except": bool?, "keep": bool?, "token"? }`:
/// the box set to one state (FAWE's //set), or only where the block is (or, `except`, isn't) one of `only`
/// (//replace). With `keep` the blocks as they were stay here for `box_undo` / `box_redo` (`edit` in the answer).
/// Answers `changed` and `micros`.
pub(in crate::game) fn box_fill(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    once(query, || {
        let started = std::time::Instant::now();
        let Some((min, max)) = corners(query) else { return Value::Null };
        let state = query["state"].as_u64().unwrap_or(0) as u32;
        if state as usize >= game.pack.assets.registry.len() {
            return json!({ "error": "no such state" });
        }
        let only = query["only"].as_array().map(|ids| ids.iter().filter_map(|i| i.as_u64()).map(|i| i as u32).collect());
        let mut edit = BoxEdit { dimension: dimension.to_owned(), min, max, state, only, except: query["except"].as_bool().unwrap_or(false), before: Vec::new(), data: None, entities_before: Vec::new(), entities_after: Vec::new() };
        let keep = query["keep"].as_bool().unwrap_or(false);
        let (mut before, mut entities) = (Vec::new(), Vec::new());
        let Some(changed) = fill(game, &edit, keep.then_some(&mut before), keep.then_some(&mut entities)) else { return json!({ "error": "chunks did not load" }) };
        let mut answer = json!({ "changed": changed, "micros": started.elapsed().as_micros() as u64 });
        if keep {
            edit.before = before;
            edit.entities_before = entities;
            let id = NEXT_BOX_EDIT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            BOX_EDITS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Default::default).insert(id, edit);
            answer["edit"] = json!(id);
        }
        answer
    })
}

/// `{ "op": "box_undo" | "box_redo", "edit", "token"? }`: a kept edit's box back as it was, or the edit again.
pub(in crate::game) fn box_replay(game: &mut Game<'_>, query: &Value, undo: bool) -> Value {
    once(query, || {
        let started = std::time::Instant::now();
        let id = query["edit"].as_u64().unwrap_or(0);
        let Some(edit) = BOX_EDITS.lock().unwrap_or_else(|e| e.into_inner()).as_mut().and_then(|edits| edits.remove(&id)) else {
            return json!({ "error": "no such edit" });
        };
        let changed = if undo {
            let before = &edit.before;
            let entities = &edit.entities_before;
            game.native_in_level(&edit.dimension, |game| {
                resident(game, edit.min, edit.max).then(|| {
                    let changed = write_box(game, edit.min, edit.max, |_, i| before.get(i).copied(), None);
                    entities_put(game, entities);
                    changed
                })
            })
            .flatten()
        } else {
            fill(game, &edit, None, None)
        };
        BOX_EDITS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Default::default).insert(id, edit);
        json!({ "changed": changed, "micros": started.elapsed().as_micros() as u64 })
    })
}

/// `{ "op": "box_forget", "edit" }`: a kept edit dropped (FAWE let go of that history).
pub(in crate::game) fn box_forget(query: &Value) -> Value {
    let id = query["edit"].as_u64().unwrap_or(0);
    json!(BOX_EDITS.lock().unwrap_or_else(|e| e.into_inner()).as_mut().and_then(|edits| edits.remove(&id)).is_some())
}

/// `{ "op": "box_read", "dimension", "min", "max" }`: every state id of a box, (y, z, x) order with the world's
/// height clamped, as base64 of little-endian u32s (FAWE's copy reads whole boxes this way).
pub(in crate::game) fn box_read(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    let Some((min, max)) = corners(query) else { return Value::Null };
    game.native_in_level(dimension, |game| {
        if !resident(game, min, max) {
            return json!({ "error": "chunks did not load" });
        }
        let (bottom, top) = (game.world.bottom_y(), game.world.top_y());
        let mut ids = Vec::new();
        each_in(min, max, bottom, top, |x, y, z| ids.push(game.world.block(x, y, z).0));
        let mut answer = json!({ "bottom": min[1].max(bottom), "top": max[1].min(top - 1) });
        // `"format": "runs"`: as (id, count) runs, which a world mostly of a few blocks shrinks to almost nothing.
        if query["format"] == "runs" {
            answer["runs"] = json!(runs_encode(&ids));
        } else {
            let bytes: Vec<u8> = ids.iter().flat_map(|id| id.to_le_bytes()).collect();
            answer["data"] = json!(base64_encode(&bytes));
        }
        answer
    })
    .unwrap_or(Value::Null)
}

/// `{ "op": "box_entities", "dimension", "min", "max" }`: the block entities of a box with their data (a chest's
/// items, a sign's text): `{ entities: [{ pos: [x, y, z], nbt: base64 of named binary NBT }] }`.
pub(in crate::game) fn box_entities(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    let Some((min, max)) = corners(query) else { return Value::Null };
    game.native_in_level(dimension, |game| {
        if !resident(game, min, max) {
            return json!({ "error": "chunks did not load" });
        }
        let list: Vec<Value> = entities_in(game, min, max).iter().map(|(k, tag)| json!({ "pos": [k.0, k.1, k.2], "nbt": nbt_encode(tag) })).collect();
        json!({ "entities": list })
    })
    .unwrap_or(Value::Null)
}

/// `{ "op": "box_write", "dimension", "min", "max", "data" | "runs", "skip": id?, "keep": bool?, "token"? }`: a box's
/// blocks from `data` (as box_read gives them) or `runs` (box_read's runs). An id of `skip`, or one the game doesn't
/// have, leaves the block alone. With `keep` the blocks as they were stay here for `box_undo` / `box_redo` (`edit`
/// in the answer), so a paste is one query. Answers `changed` and `micros`.
pub(in crate::game) fn box_write(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    once(query, || {
        let started = std::time::Instant::now();
        let Some((min, max)) = corners(query) else { return Value::Null };
        let ids: Vec<u32> = match query["runs"].as_str() {
            Some(runs) => runs_decode(runs),
            None => {
                let bytes = base64_decode(query["data"].as_str().unwrap_or_default());
                bytes.chunks_exact(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect()
            }
        };
        let skip = query["skip"].as_u64().map(|s| s as u32);
        let states = game.pack.assets.registry.len() as u32;
        let keep = query["keep"].as_bool().unwrap_or(false);
        let placed = entities_of(&query["entities"]);
        let (mut before, mut overwritten) = (Vec::new(), Vec::new());
        let changed = game.native_in_level(dimension, |game| {
            if !resident(game, min, max) {
                return None;
            }
            if keep {
                overwritten = entities_in(game, min, max);
            }
            let changed = write_box(game, min, max, |_, i| ids.get(i).copied().filter(|id| Some(*id) != skip && *id < states), keep.then_some(&mut before));
            entities_put(game, &placed);
            Some(changed)
        });
        let Some(Some(changed)) = changed else { return json!({ "error": "chunks did not load" }) };
        let mut answer = json!({ "changed": changed, "micros": started.elapsed().as_micros() as u64 });
        if keep {
            let data = ids.into_iter().map(|id| if Some(id) == skip { u32::MAX } else { id }).collect();
            let edit = BoxEdit { dimension: dimension.to_owned(), min, max, state: 0, only: None, except: false, before, data: Some(data), entities_before: overwritten, entities_after: placed };
            let id = NEXT_BOX_EDIT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            BOX_EDITS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Default::default).insert(id, edit);
            answer["edit"] = json!(id);
        }
        answer
    })
}

/// `{ "op": "chunk_snapshot", ... }`: every block of a loaded chunk, as ids into a palette, bottom up.
pub(in crate::game) fn chunk_snapshot(game: &mut Game<'_>, dimension: &str, chunk: [i32; 2]) -> Value {
    game.native_in_level(dimension, |game| {
        if !game.world.loaded(chunk[0] << 4, chunk[1] << 4) {
            return Value::Null;
        }
        let (bottom, top) = (game.world.bottom_y(), game.world.top_y());
        let mut palette: Vec<String> = Vec::new();
        let mut seen = std::collections::HashMap::new();
        let mut blocks = Vec::with_capacity(((top - bottom) * 256) as usize);
        for y in bottom..top {
            for z in 0..16 {
                for x in 0..16 {
                    let state = game.world.block((chunk[0] << 4) + x, y, (chunk[1] << 4) + z);
                    let id = *seen.entry(state).or_insert_with(|| {
                        palette.push(game.pack.assets.registry.state(state).map_or_else(|| "minecraft:air".to_owned(), |state| {
                            if state.properties.is_empty() {
                                return state.name.clone();
                            }
                            let properties: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
                            format!("{}[{}]", state.name, properties.join(","))
                        }));
                        palette.len() - 1
                    });
                    blocks.push(id);
                }
            }
        }
        json!({ "bottom": bottom, "height": top - bottom, "palette": palette, "blocks": blocks, "time": game.world.game_time })
    })
    .unwrap_or(Value::Null)
}

/// `{ "op": "chunk_entities", ... }`: the block entities of a loaded chunk, by position.
pub(in crate::game) fn chunk_block_entities(game: &mut Game<'_>, dimension: &str, chunk: [i32; 2]) -> Value {
    game.native_in_level(dimension, |game| {
        let found: Vec<[i32; 3]> = game.world.block_entities.keys().filter(|(x, _, z)| x >> 4 == chunk[0] && z >> 4 == chunk[1]).map(|(x, y, z)| [*x, *y, *z]).collect();
        json!(found)
    })
    .unwrap_or(Value::Null)
}

/// `Entity.remove()` for what the engine's own removal (`native_remove_entity`, its generic entities) doesn't reach:
/// a mob, gone without dying (no loot, as vanilla's `discard`), or an item. The players' entity trackers send the
/// removal at the next tick; mods hear it left.
pub(in crate::game) fn discard_entity(game: &mut Game<'_>, dimension: &str, id: u32) -> bool {
    game.native_in_level(dimension, |game| {
        if let Some(i) = game.mobs.iter().position(|m| m.id == id) {
            let mob = game.mobs.remove(i);
            game.native_entities_left(mob.kind.id.as_ref(), &[id]);
            return true;
        }
        if let Some(i) = game.dropped.iter().position(|item| item.id == id) {
            game.remove_dropped(i);
            return true;
        }
        false
    })
    .unwrap_or(false)
}

/// `{ "op": "box_regen", "dimension", "min", "max", "biomes"? }`: the box as the level's generator makes it afresh
/// (terrain, carvers, features, structures), in box_read's runs, nothing written (WorldEdit's //regen, which the plugin
/// writes as one undoable edit). Terrain for the chunks and two rings around them is generated on every core, then the
/// chunks and the ring around them are decorated, as the world's own chunks are. With `biomes`, the generated biomes
/// too, as box_biomes answers them.
pub(in crate::game) fn box_regen(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    use horizon_client::streaming::{ChunkGenerator, Proto};
    let Some((min, max)) = corners(query) else { return Value::Null };
    let started = std::time::Instant::now();
    let biomes = query["biomes"].as_bool().unwrap_or(false);
    let seed = query["seed"].as_i64();
    game.native_in_level(dimension, |game| {
        // Another seed (WorldEdit's `//regen <seed>`): the game's terrain made for it, kept for the next regen.
        let terrain = match seed {
            Some(seed) => match seeded_terrain(game, dimension, seed) {
                Ok(terrain) => terrain,
                Err(error) => return json!({ "error": error }),
            },
            _ => match game.end.locator() {
                Some(terrain) => terrain,
                None => return json!({ "error": "this world has no generator to regenerate from (flat or a plugin's)" }),
            },
        };
        let (cx0, cx1, cz0, cz1) = (min[0] >> 4, max[0] >> 4, min[2] >> 4, max[2] >> 4);
        if (cx1 - cx0 + 1) * (cz1 - cz0 + 1) > 1024 {
            return json!({ "error": "too many chunks at once (1024 at most)" });
        }
        let keys: Vec<(i32, i32)> = (cx0 - 2..=cx1 + 2).flat_map(|x| (cz0 - 2..=cz1 + 2).map(move |z| (x, z))).collect();
        let next = std::sync::atomic::AtomicUsize::new(0);
        let made: std::sync::Mutex<std::collections::HashMap<(i32, i32), Proto>> = std::sync::Mutex::new(std::collections::HashMap::new());
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 8).min(keys.len());
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| {
                    let mut worker = terrain.worker();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(&(x, z)) = keys.get(i) else { break };
                        let proto = terrain.proto(&mut worker, x, z);
                        made.lock().unwrap_or_else(|e| e.into_inner()).insert((x, z), proto);
                    }
                });
            }
        });
        let mut protos = made.into_inner().unwrap_or_else(|e| e.into_inner());
        let mut worker = terrain.worker();
        for x in cx0 - 1..=cx1 + 1 {
            for z in cz0 - 1..=cz1 + 1 {
                let mut region: [Proto; 9] = std::array::from_fn(|i| protos.remove(&(x + i as i32 % 3 - 1, z + i as i32 / 3 - 1)).unwrap_or_else(|| Box::new(())));
                terrain.decorate(&mut worker, &mut region);
                for (i, proto) in region.into_iter().enumerate() {
                    protos.insert((x + i as i32 % 3 - 1, z + i as i32 / 3 - 1), proto);
                }
            }
        }
        let mut chunks = std::collections::HashMap::new();
        for x in cx0..=cx1 {
            for z in cz0..=cz1 {
                if let Some(proto) = protos.remove(&(x, z)) {
                    chunks.insert((x, z), terrain.finish(proto));
                }
            }
        }
        let (bottom, top) = (game.world.bottom_y(), game.world.top_y());
        let mut ids = Vec::new();
        each_in(min, max, bottom, top, |x, y, z| ids.push(chunks.get(&(x >> 4, z >> 4)).map_or(0, |c| c.get(x & 15, y, z & 15))));
        let mut answer = json!({ "bottom": min[1].max(bottom), "top": max[1].min(top - 1), "runs": runs_encode(&ids), "micros": started.elapsed().as_micros() as u64 });
        if biomes {
            let (q0, q1) = quarts(min, max, bottom, top);
            let mut cells = Vec::new();
            each_quart(q0, q1, |qx, qy, qz| cells.push(chunks.get(&(qx >> 2, qz >> 2)).map_or(0, |c| u32::from(c.biome(qx << 2, qy << 2, qz << 2)))));
            answer["biomes"] = json!({ "names": biome_names(game), "min": q0, "max": q1, "runs": runs_encode(&cells) });
        }
        answer
    })
    .unwrap_or(Value::Null)
}

/// The level's terrain generator as it would be with another seed (the world's preset and dimension), the last few
/// kept: building one reads the worldgen data.
fn seeded_terrain(game: &Game<'_>, dimension: &str, seed: i64) -> Result<std::sync::Arc<horizon_client::vanillagen::VanillaTerrain>, String> {
    type Kept = Vec<((i64, String), std::sync::Arc<horizon_client::vanillagen::VanillaTerrain>)>;
    static KEPT: std::sync::Mutex<Kept> = std::sync::Mutex::new(Vec::new());
    let id = match dimension.trim_start_matches("minecraft:") {
        "the_nether" | "nether" => "minecraft:the_nether",
        "the_end" | "end" => "minecraft:the_end",
        _ => "minecraft:overworld",
    };
    let key = (seed, id.to_owned());
    if let Some((_, terrain)) = KEPT.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|(k, _)| *k == key) {
        return Ok(terrain.clone());
    }
    let preset = horizon_client::save::WorldGen::preset(&game.world_root);
    let terrain = horizon_client::vanillagen::VanillaTerrain::for_world(&game.options.resources.join("worldgen"), game.pack.assets.registry, game.res.biomes(), seed, preset, id, true)
        .map(std::sync::Arc::new)
        .map_err(|e| format!("couldn't make the terrain for seed {seed}: {e}"))?;
    let mut kept = KEPT.lock().unwrap_or_else(|e| e.into_inner());
    kept.push((key, terrain.clone()));
    if kept.len() > 3 {
        kept.remove(0);
    }
    Ok(terrain)
}

/// The quart cells (4x4x4 biome cells) a box reaches, the world's height clamped.
fn quarts(min: [i32; 3], max: [i32; 3], bottom: i32, top: i32) -> ([i32; 3], [i32; 3]) {
    ([min[0] >> 2, min[1].max(bottom) >> 2, min[2] >> 2], [max[0] >> 2, max[1].min(top - 1) >> 2, max[2] >> 2])
}

fn each_quart(q0: [i32; 3], q1: [i32; 3], mut f: impl FnMut(i32, i32, i32)) {
    for qy in q0[1]..=q1[1] {
        for qz in q0[2]..=q1[2] {
            for qx in q0[0]..=q1[0] {
                f(qx, qy, qz);
            }
        }
    }
}

fn biome_names(game: &Game<'_>) -> Vec<String> {
    game.res.biomes().list.iter().map(|b| b.name.to_string()).collect()
}

/// `{ "op": "box_biomes", "dimension", "min", "max" }`: the biome of every quart cell (4x4x4) the box reaches, (y, z, x)
/// order over `min`..`max` in quarts, as runs of indices into `names`.
pub(in crate::game) fn box_biomes(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    let Some((min, max)) = corners(query) else { return Value::Null };
    game.native_in_level(dimension, |game| {
        if !resident(game, min, max) {
            return json!({ "error": "chunks did not load" });
        }
        let (q0, q1) = quarts(min, max, game.world.bottom_y(), game.world.top_y());
        let mut cells = Vec::new();
        each_quart(q0, q1, |qx, qy, qz| cells.push(game.world.chunks.get(&(qx >> 2, qz >> 2)).map_or(0, |c| u32::from(c.biome(qx << 2, qy << 2, qz << 2)))));
        json!({ "names": biome_names(game), "min": q0, "max": q1, "runs": runs_encode(&cells) })
    })
    .unwrap_or(Value::Null)
}

/// `{ "op": "box_set_biomes", "dimension", "min", "max", "biome" | "runs" (+ "names") }`: sets the quart cells the box
/// reaches to one biome, or to box_biomes' runs (indices into `names`, which may be another server's list; an index
/// past it, u32::MAX, leaves a cell). The chunks changed go out again whole (a client colours by biome). Answers
/// `changed`.
pub(in crate::game) fn box_set_biomes(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    let Some((min, max)) = corners(query) else { return Value::Null };
    let lookup = |game: &Game<'_>, name: &str| -> Option<u32> {
        let name = if name.contains(':') { name.to_owned() } else { format!("minecraft:{}", name.to_ascii_lowercase()) };
        game.res.biomes().list.iter().position(|b| *b.name == *name).map(|i| i as u32)
    };
    let one = query["biome"].as_str().map(str::to_owned);
    let cells: Vec<u32> = match &one {
        Some(name) => match lookup(game, name) {
            Some(id) => vec![id],
            None => return json!({ "error": format!("unknown biome {name}") }),
        },
        None => {
            // Their names to this server's indices.
            let names: Vec<Option<u32>> = query["names"].as_array().map(|n| n.iter().map(|v| v.as_str().and_then(|s| lookup(game, s))).collect()).unwrap_or_default();
            runs_decode(query["runs"].as_str().unwrap_or_default())
                .into_iter()
                .map(|i| if names.is_empty() { i } else { names.get(i as usize).copied().flatten().unwrap_or(u32::MAX) })
                .collect()
        }
    };
    let one = one.is_some();
    game.native_in_level(dimension, |game| {
        if !resident(game, min, max) {
            return json!({ "error": "chunks did not load" });
        }
        let (bottom, top) = (game.world.bottom_y(), game.world.top_y());
        let (q0, q1) = quarts(min, max, bottom, top);
        let count = game.res.biomes().list.len() as u32;
        let (mut i, mut changed) = (0usize, 0u64);
        let mut touched = std::collections::BTreeSet::new();
        let world = &mut game.world;
        each_quart(q0, q1, |qx, qy, qz| {
            let id = if one { cells[0] } else { cells.get(i).copied().unwrap_or(u32::MAX) };
            i += 1;
            if id >= count {
                return;
            }
            let key = (qx >> 2, qz >> 2);
            let Some(shared) = world.chunks.get_mut(&key) else { return };
            if u32::from(shared.biome(qx << 2, qy << 2, qz << 2)) == id {
                return;
            }
            let cell = (qy - (bottom >> 2)).clamp(0, (top - bottom) / 4 - 1);
            std::sync::Arc::make_mut(shared).set_biome_cell(qx & 3, cell, qz & 3, id as u16);
            changed += 1;
            touched.insert(key);
        });
        for key in &touched {
            game.world.modified.insert(*key);
        }
        resend_chunks(game, touched.into_iter().collect());
        json!({ "changed": changed })
    })
    .unwrap_or(Value::Null)
}

/// Chunks forgotten as sent to the players here, so they go out again whole first thing.
fn resend_chunks(game: &mut Game<'_>, keys: Vec<(i32, i32)>) {
    if keys.is_empty() {
        return;
    }
    let Some(host) = game.lan.as_mut() else { return };
    let here: Vec<usize> = (0..host.hub.guests.len()).filter(|&i| host.hub.here(&host.hub.guests[i])).collect();
    for i in here {
        let guest = &mut host.hub.guests[i];
        for key in &keys {
            if guest.sent_chunks.remove(key) && !guest.chunk_queue.contains(key) {
                guest.chunk_queue.insert(0, *key);
            }
        }
    }
}

/// The entities of the level in place that a world editor copies (all but players, orbs, projectiles and the dragon).
fn copyable_entities(game: &Game<'_>) -> Vec<crate::game::commands::Target> {
    use crate::game::commands::Target;
    let mut list: Vec<Target> = game.mobs.iter().filter(|m| m.death_time == 0).map(|m| Target::Mob(m.id)).collect();
    list.extend(game.dropped.iter().map(|i| Target::Item(i.id)));
    list.extend(game.armor_stands.iter().map(|a| Target::Stand(a.id)));
    list.extend(game.item_frames.iter().map(|f| Target::Frame(f.id)));
    list.extend(game.paintings.iter().map(|p| Target::Painting(p.id)));
    list.extend(game.boats.iter().map(|b| Target::Boat(b.id)));
    list.extend(game.carts.iter().map(|c| Target::Cart(c.id)));
    list.extend(game.others.iter().map(|o| Target::Other(o.id)));
    list.extend(game.crystals.iter().map(|c| Target::Crystal(c.id)));
    list
}

fn tag_f64(tag: &horizon_world::nbt::Tag) -> Option<f64> {
    use horizon_world::nbt::Tag;
    match tag {
        Tag::Double(bits) => Some(f64::from_bits(*bits)),
        Tag::Float(bits) => Some(f64::from(f32::from_bits(*bits))),
        Tag::Int(v) => Some(f64::from(*v)),
        _ => None,
    }
}

/// `{ "op": "box_mobs", "dimension", "min", "max", "tag"?, "remove"? }`: the entities whose position is in the box
/// (players aside), each `{ kind, pos: [x, y, z], yaw, snbt }` with its saved data (no UUID, position or rotation).
/// `tag`: only those carrying that scoreboard tag. `remove`: they are taken out of the world too, without drops
/// (//cut -e, an undone paste).
pub(in crate::game) fn box_mobs(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    use crate::game::commands::Target;
    use horizon_world::nbt::Tag;
    let Some((min, max)) = corners(query) else { return Value::Null };
    let wanted = query["tag"].as_str().map(str::to_owned);
    let remove = query["remove"].as_bool().unwrap_or(false);
    let dimension = dimension.to_owned();
    game.native_in_level(&dimension.clone(), |game| {
        let mut out = Vec::new();
        for target in copyable_entities(game) {
            let Some(mut tag) = game.entity_nbt(target) else { continue };
            let Some(pos) = (match tag.get("Pos") {
                Some(Tag::List(l)) if l.len() == 3 => (|| Some([tag_f64(&l[0])?, tag_f64(&l[1])?, tag_f64(&l[2])?]))(),
                _ => None,
            }) else {
                continue;
            };
            let inside = (0..3).all(|i| pos[i] >= f64::from(min[i]) && pos[i] < f64::from(max[i]) + 1.0);
            if !inside {
                continue;
            }
            if let Some(wanted) = &wanted {
                let has = matches!(tag.get("Tags"), Some(Tag::List(l)) if l.iter().any(|t| matches!(t, Tag::String(s) if s.to_string_lossy() == *wanted)));
                if !has {
                    continue;
                }
            }
            let kind = match tag.get("id") {
                Some(Tag::String(s)) => s.to_string_lossy(),
                _ => match target {
                    Target::Mob(id) => match game.mobs.iter().find(|m| m.id == id) {
                        Some(m) => m.kind.id.to_string(),
                        None => continue,
                    },
                    _ => continue,
                },
            };
            let yaw = match tag.get("Rotation") {
                Some(Tag::List(l)) => l.first().and_then(tag_f64).unwrap_or(0.0),
                _ => 0.0,
            };
            if let Tag::Compound(c) = &mut tag {
                for key in ["UUID", "Pos", "Rotation", "id", "Motion", "Passengers", "Leash"] {
                    c.remove(&key.into());
                }
            }
            out.push(json!({ "kind": kind, "pos": pos, "yaw": yaw, "snbt": horizon_world::snbt_text::to_snbt(&tag) }));
            if remove {
                match target {
                    Target::Mob(id) | Target::Item(id) => {
                        discard_entity(game, &dimension, id);
                    }
                    other => game.kill_target(other),
                }
            }
        }
        json!({ "entities": out })
    })
    .unwrap_or(Value::Null)
}

/// `{ "op": "box_summon", "dimension", "entities": [{ kind, pos, yaw?, snbt }], "tag"? }`: each entity made again from
/// box_mobs' data where `pos` says (as `/summon` with its data), carrying the scoreboard tag `tag` (so an undo can
/// find them). Answers how many.
pub(in crate::game) fn box_summon(game: &mut Game<'_>, dimension: &str, query: &Value) -> Value {
    use horizon_world::nbt::Tag;
    let tag_name = query["tag"].as_str();
    let mut made = 0;
    for e in query["entities"].as_array().map(Vec::as_slice).unwrap_or_default() {
        let (Some(kind), Some(pos)) = (e["kind"].as_str(), e["pos"].as_array()) else { continue };
        let at: Vec<f64> = pos.iter().filter_map(Value::as_f64).collect();
        if at.len() != 3 || !kind.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '.' | '-' | '/')) {
            continue;
        }
        let mut tag = horizon_world::nbt::parse_snbt(e["snbt"].as_str().unwrap_or("{}")).map(|(t, _)| t).unwrap_or(Tag::Compound(Default::default()));
        if let Tag::Compound(c) = &mut tag {
            if let Some(yaw) = e["yaw"].as_f64() {
                c.insert("Rotation".into(), Tag::List(vec![Tag::Float((yaw as f32).to_bits()), Tag::Float(0f32.to_bits())]));
            }
            if let Some(name) = tag_name {
                let mut tags = match c.remove(&"Tags".into()) {
                    Some(Tag::List(l)) => l,
                    _ => Vec::new(),
                };
                tags.push(Tag::String(name.into()));
                c.insert("Tags".into(), Tag::List(tags));
            }
        }
        let line = format!("execute in {dimension} run summon {kind} {} {} {} {}", at[0], at[1], at[2], horizon_world::snbt_text::to_snbt(&tag));
        super::paper_fam_core::run_game_only(game, &line);
        made += 1;
    }
    json!({ "made": made })
}
