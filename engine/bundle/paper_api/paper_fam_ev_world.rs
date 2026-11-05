// Family ev_world: answers to the queries its SDK file makes.
use crate::game::Game;
use horizon_client::items::ItemStack;
use horizon_mod_runtime as runtime;
use horizon_world::nbt::Tag;
use serde_json::{Value, json};
use std::sync::Mutex;

/// A vehicle hit again with the damage a listener chose: true if it's still there to hit.
fn vehicle_hit(game: &mut Game<'_>, query: &Value) -> bool {
    let id = query["vehicle"].as_u64().unwrap_or(0) as u32;
    let damage = query["damage"].as_f64().unwrap_or(0.0) as f32;
    let creative = query["creative"].as_bool().unwrap_or(false);
    if let Some(i) = game.carts.iter().position(|cart| cart.id == id) {
        game.hurt_cart(i, damage, creative, query["fire"].as_bool().unwrap_or(false));
        return true;
    }
    if let Some(i) = game.boats.iter().position(|boat| boat.id == id) {
        game.hurt_boat(i, damage, creative);
        return true;
    }
    false
}

/// What a listener set for the block about to give experience.
static XP: Mutex<u32> = Mutex::new(0);
/// What a listener left of a break's drops.
static DROPS: Mutex<Vec<ItemStack>> = Mutex::new(Vec::new());
/// What a listener left of a blast's blocks.
static BLAST_BLOCKS: Mutex<Vec<[i32; 3]>> = Mutex::new(Vec::new());

pub(in crate::game) fn take_xp() -> u32 {
    std::mem::take(&mut *XP.lock().unwrap_or_else(|e| e.into_inner()))
}

pub(in crate::game) fn put_drops(drops: &mut Vec<ItemStack>) {
    *drops = std::mem::take(&mut *DROPS.lock().unwrap_or_else(|e| e.into_inner()));
}

pub(in crate::game) fn keep_blocks(blocks: &mut Vec<[i32; 3]>) {
    let kept = std::mem::take(&mut *BLAST_BLOCKS.lock().unwrap_or_else(|e| e.into_inner()));
    blocks.retain(|pos| kept.contains(pos));
}

pub(in crate::game) fn state_text(game: &Game<'_>, state: horizon_world::registry::BlockStateId) -> String {
    let Some(state) = game.pack.assets.registry.state(state) else {
        return "minecraft:air".to_owned();
    };
    if state.properties.is_empty() {
        return state.name.to_string();
    }
    let properties: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
    format!("{}[{}]", state.name, properties.join(","))
}

fn position(value: &Value) -> Option<[i32; 3]> {
    let list = value.as_array()?;
    Some([list.first()?.as_i64()? as i32, list.get(1)?.as_i64()? as i32, list.get(2)?.as_i64()? as i32])
}

/// A stack a plugin built: its item, count and components, each as SNBT.
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

/// The experience the break at `pos` would give, worked out with the game's own roll put back after.
fn break_exp(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let (harvest, silk, name) = break_tool(game, query)?;
    let spawner = &*name == "minecraft:spawner";
    if game.creative || !harvest || (silk && !spawner) {
        return Some(json!(0));
    }
    let saved = game.world.random.clone();
    let xp = game.block_experience(&name);
    game.world.random = saved;
    Some(json!(xp))
}

fn break_tool(game: &Game<'_>, query: &Value) -> Option<(bool, bool, std::sync::Arc<str>)> {
    let pos = position(&query["pos"])?;
    let facts = game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).clone();
    let held = game.held().cloned();
    let harvest = !facts.requires_tool || game.res.items().correct_for_drops(held.as_ref(), &facts.name);
    let silk = held.as_ref().is_some_and(|stack| stack.enchantment_level("minecraft:silk_touch") > 0);
    Some((harvest, silk, facts.name.clone()))
}

/// A break's experience as a listener left it. The game only asks for it with a tool that keeps it:
/// for any other the orbs come now, as the block goes. `true` if the game will ask.
fn xp_next(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let (harvest, silk, name) = break_tool(game, query)?;
    let exp = u32::try_from(query["exp"].as_i64()?).ok()?;
    if harvest && (!silk || &*name == "minecraft:spawner") {
        *XP.lock().unwrap_or_else(|e| e.into_inner()) = exp;
        return Some(json!(true));
    }
    if !game.creative && exp > 0 {
        let pos = position(&query["pos"])?;
        game.spawn_orbs([f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 0.5, f64::from(pos[2]) + 0.5], exp);
    }
    Some(json!(false))
}

/// The raiders of one wave, and its leader.
fn raiders(game: &mut Game<'_>, query: &Value) -> Value {
    let (raid, wave) = (query["raid"].as_i64().unwrap_or(0) as i32, query["wave"].as_i64().unwrap_or(0) as i32);
    use horizon_client::mobs::raider::state;
    let wave_of = |mob: &&horizon_client::mobs::Mob| state(mob).is_some_and(|state| state.raid_id == Some(raid) && state.wave == wave);
    let ids: Vec<u32> = game.mobs.iter().filter(wave_of).map(|mob| mob.id).collect();
    let leader = game.mobs.iter().filter(wave_of).find(|mob| state(mob).is_some_and(|state| state.patrol_leader)).map(|mob| mob.id);
    json!({ "ids": ids, "leader": leader })
}

/// An effect's name as the game keeps it, which must outlive the effect.
fn effect_name(id: &str) -> &'static str {
    static NAMES: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
    let id = id.trim_start_matches("minecraft:");
    let mut names = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(name) = names.iter().find(|name| **name == id) {
        return name;
    }
    let name: &'static str = Box::leak(id.to_owned().into_boxed_str());
    names.push(name);
    name
}

/// What a listener put in the game's place, for the call it refused.
static RESULT: Mutex<Option<ItemStack>> = Mutex::new(None);

pub(in crate::game) fn take_result() -> Option<ItemStack> {
    RESULT.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// Food goes on a campfire for the time a listener chose, as the game would have put it.
fn campfire_place(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    let time = query["time"].as_i64()? as i32;
    let held = game.held().cloned()?;
    let placed = match game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2])) {
        Some(horizon_client::containers::BlockEntity::Campfire(fire)) => fire.place_food(&mut held.clone(), time),
        _ => false,
    };
    if placed {
        game.consume_held();
        game.stats.custom("interact_with_campfire", 1);
        game.mark_block_entity_updated((pos[0], pos[1], pos[2]));
    }
    Some(json!(placed))
}

/// What a campfire just cooked and by which recipe, for the result's hook.
static COOKED: Mutex<(Value, Option<String>)> = Mutex::new((Value::Null, None));

pub(in crate::game) fn note_cooked(game: &Game<'_>, item: &ItemStack) {
    let recipe = game.res.recipes().cook("minecraft:campfire_cooking", item).map(|recipe| format!("minecraft:{}", recipe.id));
    *COOKED.lock().unwrap_or_else(|e| e.into_inner()) = (crate::game::paper_query::stack_json(item), recipe);
}

pub(in crate::game) fn cooked() -> (Value, Option<String>) {
    COOKED.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Loot a listener changed: the table it stands for, and what it gives instead.
static LOOT_NEXT: Mutex<Option<(String, Vec<ItemStack>)>> = Mutex::new(None);

pub(in crate::game) fn loot_next(table: &str) -> Option<Vec<ItemStack>> {
    LOOT_NEXT.lock().unwrap_or_else(|e| e.into_inner()).take_if(|(wanted, _)| wanted == table).map(|(_, loot)| loot)
}

/// What a mob's shearing rolls: `(parent, table)`, as `Game::shear_entity` picks them.
pub(in crate::game) fn shearable(mob: &horizon_client::mobs::Mob) -> Option<(String, String)> {
    use horizon_client::mobs::Species;
    if !mob.alive() {
        return None;
    }
    let same = |table: &str| Some((table.to_owned(), table.to_owned()));
    match mob.species() {
        Species::Sheep if !mob.extra.sheared && !mob.is_baby() => {
            let color = horizon_client::mobs::render::DYE_NAMES[mob.extra.color as usize & 15];
            Some(("minecraft:shearing/sheep".to_owned(), format!("minecraft:shearing/sheep/{color}")))
        }
        Species::Bogged if !mob.extra.sheared => same("minecraft:shearing/bogged"),
        Species::SnowGolem if !mob.extra.sheared => same("minecraft:shearing/snow_golem"),
        Species::Mooshroom if !mob.is_baby() => {
            let variant = if mob.extra.variant == 1 { "brown" } else { "red" };
            Some(("minecraft:shearing/mooshroom".to_owned(), format!("minecraft:shearing/mooshroom/{variant}")))
        }
        _ => None,
    }
}

/// A look at a loot table's roll, with the game's dice put back after it.
fn loot_preview(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let tool = if query["held"].as_bool().unwrap_or(false) { game.held().cloned() } else { Some(ItemStack::new("minecraft:shears", 1)) };
    let (sequences, random) = (game.rand_sequences.clone(), game.world.random.clone());
    let rolled = game.sequence_loot(query["parent"].as_str()?, query["table"].as_str()?, tool.as_ref());
    game.rand_sequences = sequences;
    game.world.random = random;
    Some(Value::Array(rolled.iter().map(crate::game::paper_query::stack_json).collect()))
}

/// A dispenser's velocity a listener chose, for the item about to leave.
static VELOCITY: Mutex<Option<[f64; 3]>> = Mutex::new(None);

pub(in crate::game) fn dispense_velocity() -> Option<[f64; 3]> {
    VELOCITY.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// `SculkBloomEvent`: what the catalyst takes, as listeners leave it.
pub(in crate::game) fn bloom_charge(game: &mut Game<'_>, key: (i32, i32, i32), xp: u32) -> u32 {
    if xp == 0 || !crate::game::paper_bridge::hears("ew_bloom_charge") {
        return xp;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let payload = json!({ "uuid": Value::Null, "level": level, "pos": [key.0, key.1, key.2], "charge": xp });
    *CHARGE.lock().unwrap_or_else(|e| e.into_inner()) = None;
    if crate::game::paper_bridge::on_refusable(game, "ew_bloom_charge", payload) {
        return 0;
    }
    CHARGE.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or(xp)
}

static CHARGE: Mutex<Option<u32>> = Mutex::new(None);
static RADIUS: Mutex<Option<i32>> = Mutex::new(None);

pub(in crate::game) fn take_radius() -> Option<i32> {
    RADIUS.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// Who a bell makes glow, as `BellResonateEvent` and `BellRevealRaiderEvent` leave it.
pub(in crate::game) fn bell_glow(game: &mut Game<'_>, pos: [i32; 3], glow: Vec<u64>) -> Vec<u64> {
    use crate::game::paper_bridge::{hears, on_refusable, on_trigger};
    let mut glow = glow;
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    if hears("ew_bell_resonate") {
        on_trigger(game, "ew_bell_resonate", json!({ "uuid": Value::Null, "level": level, "pos": pos, "raiders": glow }));
        if let Some(kept) = BELL.lock().unwrap_or_else(|e| e.into_inner()).take() {
            glow = kept;
        }
    }
    if hears("ew_bell_reveal") {
        let everyone = std::mem::take(&mut glow);
        for id in everyone {
            if !on_refusable(game, "ew_bell_reveal", json!({ "uuid": Value::Null, "level": level, "pos": pos, "entity": id })) {
                glow.push(id);
            }
        }
    }
    glow
}

static BELL: Mutex<Option<Vec<u64>>> = Mutex::new(None);

/// A suggestion's tooltip, as a listener gave it.
static TOOLTIPS: Mutex<Vec<(String, Value)>> = Mutex::new(Vec::new());
static SUGGEST_NONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(in crate::game) fn tooltip(suggestion: &str) -> Option<Tag> {
    let tooltips = TOOLTIPS.lock().unwrap_or_else(|e| e.into_inner());
    tooltips.iter().find(|(text, _)| text == suggestion).map(|(_, tooltip)| horizon_client::lan::data::component_nbt(tooltip))
}

/// True if a listener cancelled the completion: the player gets nothing, the game's own included.
pub(in crate::game) fn suggestions_cancelled() -> bool {
    TOOLTIPS.lock().unwrap_or_else(|e| e.into_inner()).clear();
    SUGGEST_NONE.swap(false, std::sync::atomic::Ordering::Relaxed)
}

/// Whether the console line about to run came from RCON.
static REMOTE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(in crate::game) fn note_remote(remote: bool) {
    REMOTE.store(remote, std::sync::atomic::Ordering::Relaxed);
}

pub(in crate::game) fn remote_line() -> bool {
    REMOTE.swap(false, std::sync::atomic::Ordering::Relaxed)
}

/// Chunks a listener doesn't want saved as they go.
static UNSAVED: Mutex<Vec<(i32, i32)>> = Mutex::new(Vec::new());

pub(in crate::game) fn chunk_saved(chunk: (i32, i32)) -> bool {
    let mut unsaved = UNSAVED.lock().unwrap_or_else(|e| e.into_inner());
    let at = unsaved.iter().position(|known| *known == chunk);
    at.map(|at| unsaved.swap_remove(at)).is_none()
}

/// Vehicles that ran into a block this tick, while something is listening.
static VEHICLE_HITS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
static VEHICLE_WATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(in crate::game) fn note_vehicle_hit(id: u32, at: [f64; 3], tried: [f64; 3], went: [f64; 2]) {
    if VEHICLE_WATCH.load(std::sync::atomic::Ordering::Relaxed) {
        // The block in the way: one past the vehicle, on the axis it was stopped on.
        let (x, z) = if (went[0] - tried[0]).abs() > 1.0e-7 { (tried[0].signum(), 0.0) } else { (0.0, tried[2].signum()) };
        let pos = [(at[0] + x).floor() as i32, at[1].floor() as i32, (at[2] + z).floor() as i32];
        VEHICLE_HITS.lock().unwrap_or_else(|e| e.into_inner()).push(json!({ "vehicle": id, "pos": pos, "velocity": tried }));
    }
}

/// A mob a boat is about to take on: `true` if a listener refused it.
pub(in crate::game) fn boarding_refused(boat: u32, mob: u32) -> bool {
    runtime::listens("ew_boat_board") && runtime::fire("ew_boat_board", &json!({ "uuid": Value::Null, "vehicle": boat, "entity": mob, "boards": true }).to_string()) == 2
}

// ---- The GS4 query port: `enable-query` and `query.port` ----

/// Answers one GS4 packet: a challenge for a handshake, a status for a known token.
fn query_reply(packet: &[u8], from: std::net::SocketAddr, tokens: &mut Vec<(std::net::SocketAddr, i32, std::time::Instant)>, shared: &horizon_client::lan::Shared, port: u16, map: &str) -> Option<Vec<u8>> {
    if packet.len() < 7 || packet[0] != 0xFE || packet[1] != 0xFD {
        return None;
    }
    let (kind, session) = (packet[2], &packet[3..7]);
    tokens.retain(|(_, _, at)| at.elapsed() < std::time::Duration::from_secs(30));
    let mut out = vec![kind];
    out.extend_from_slice(session);
    if kind == 9 {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |t| t.subsec_nanos()) as i32;
        let token = stamp.rem_euclid(9_999_999) + 1;
        tokens.retain(|(known, _, _)| *known != from);
        tokens.push((from, token, std::time::Instant::now()));
        out.extend_from_slice(token.to_string().as_bytes());
        out.push(0);
        return Some(out);
    }
    if kind != 0 || packet.len() < 11 {
        return None;
    }
    let token = i32::from_be_bytes([packet[7], packet[8], packet[9], packet[10]]);
    if !tokens.iter().any(|(known, value, _)| *known == from && *value == token) {
        return None;
    }
    let full = packet.len() >= 15;
    let question = json!({
        "address": from.ip().to_string(),
        "full": full,
        "motd": shared.motd(),
        "server_version": "Horizon",
        "plugins": [],
        "map": map,
        "current_players": shared.online(),
        "max_players": shared.max_players(),
        "players": shared.online_names(),
        "hostname": from.ip().is_loopback().then_some("127.0.0.1").unwrap_or("0.0.0.0"),
        "game_version": shared.game_version(),
        "port": port,
    });
    QUERY_NOW.set(None);
    super::paper_bridge::fire_free("ew_query", &question);
    let answer = QUERY_NOW.take().unwrap_or(question);
    let words = |key: &str| answer[key].as_str().unwrap_or_default().as_bytes().to_vec();
    let number = |key: &str| answer[key].as_i64().unwrap_or(0).to_string().into_bytes();
    let field = |out: &mut Vec<u8>, value: &[u8]| {
        out.extend_from_slice(value);
        out.push(0);
    };
    if full {
        out.extend_from_slice(b"splitnum\0\x80\0");
        let plugins: Vec<String> = answer["plugins"].as_array().into_iter().flatten().map(|row| format!("{} {}", row["name"].as_str().unwrap_or_default(), row["version"].as_str().unwrap_or_default())).collect();
        let plugin_line = if plugins.is_empty() { String::new() } else { format!("{}: {}", answer["server_version"].as_str().unwrap_or_default(), plugins.join("; ")) };
        let rows: [(&str, Vec<u8>); 10] = [("hostname", words("motd")), ("gametype", b"SMP".to_vec()), ("game_id", b"MINECRAFT".to_vec()), ("version", words("game_version")), ("plugins", plugin_line.into_bytes()), ("map", words("map")), ("numplayers", number("current_players")), ("maxplayers", number("max_players")), ("hostport", number("port")), ("hostip", words("hostname"))];
        for (key, value) in rows {
            field(&mut out, key.as_bytes());
            field(&mut out, &value);
        }
        out.push(0);
        out.extend_from_slice(b"\x01player_\0\0");
        for name in answer["players"].as_array().into_iter().flatten() {
            field(&mut out, name.as_str().unwrap_or_default().as_bytes());
        }
        out.push(0);
    } else {
        for value in [words("motd"), b"SMP".to_vec(), words("map"), number("current_players"), number("max_players")] {
            field(&mut out, &value);
        }
        out.extend_from_slice(&(answer["port"].as_i64().unwrap_or(0) as u16).to_le_bytes());
        field(&mut out, &words("hostname"));
    }
    Some(out)
}

/// Starts the query listener once the server has its network, if `enable-query` says so.
pub(in crate::game) fn start_query(game: &Game<'_>) {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    let Some(host) = game.lan.as_ref() else {
        return;
    };
    if STARTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let path = std::path::Path::new("server.properties");
    let Ok(properties) = horizon_server_files::properties::ServerProperties::from_file(path, &mut horizon_server_files::properties::SystemEntropy::default()) else {
        return;
    };
    if !properties.enable_query {
        return;
    }
    let port = properties.query_port as u16;
    let map = properties.level_name.clone();
    let shared = host.hub.lan.shared.clone();
    let spawned = std::thread::Builder::new().name("Query Listener".into()).spawn(move || {
        let Ok(socket) = std::net::UdpSocket::bind(("0.0.0.0", port)) else {
            log::warn!("Horizon: the query port {port} can't be bound");
            return;
        };
        let mut tokens = Vec::new();
        let mut packet = [0u8; 1460];
        while let Ok((length, from)) = socket.recv_from(&mut packet) {
            if let Some(reply) = query_reply(&packet[..length], from, &mut tokens, &shared, port, &map) {
                let _ = socket.send_to(&reply, from);
            }
        }
    });
    if let Err(error) = spawned {
        log::warn!("Horizon: the query thread didn't start: {error}");
    }
}

// ---- What the level does with no game to lend: these fire as it happens, and a listener can't ask ----

/// Every block state's text, by id, as a level's moment has no registry to read.
static STATES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
/// A state a listener showed as a destroy effect.
static EFFECT: Mutex<Option<String>> = Mutex::new(None);
/// What a listener left of a block's drops, or of a sponge's water.
static FREE_DROPS: Mutex<Option<Vec<ItemStack>>> = Mutex::new(None);
static FREE_KEPT: Mutex<Option<Vec<[i32; 3]>>> = Mutex::new(None);

fn level_name(dimension: horizon_world::level::Dimension) -> &'static str {
    match dimension {
        horizon_world::level::Dimension::Overworld => "minecraft:overworld",
        horizon_world::level::Dimension::Nether => "minecraft:the_nether",
        horizon_world::level::Dimension::End => "minecraft:the_end",
    }
}

/// A block's drops, unless a piston broke it: those wait for `BlockBreakBlockEvent`.
pub(in crate::game) fn broke_block(dimension: crate::game::LevelDimension, pos: [i32; 3], drops: Vec<ItemStack>) -> Vec<ItemStack> {
    if TAKE_DROPS.get() {
        TAKEN_DROPS.set(drops);
        return Vec::new();
    }
    let source = {
        let mut broken = horizon_world::level::pistons::BROKEN_BY.lock().unwrap_or_else(|e| e.into_inner());
        broken.iter().position(|(place, _)| *place == pos).map(|at| broken.swap_remove(at).1)
    };
    let Some(source) = source.filter(|_| runtime::listens("ew_block_broke")) else {
        return drops;
    };
    BROKE.lock().unwrap_or_else(|e| e.into_inner()).push((dimension, pos, source, drops));
    horizon_world::level::ASK_WAITING.store(true, std::sync::atomic::Ordering::Relaxed);
    Vec::new()
}

// ---- The moments a plugin's lifecycle handlers wait for ----

/// The game's own data, once a world has said where it is.
static DATA_ROOT: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);
/// The moments that have been.
static LIFE_PASSED: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
static PACKS_ASKED: std::sync::Once = std::sync::Once::new();

/// Before a world's data packs are first read: what plugins put there goes in now.
fn before_packs(resources: &std::path::Path) {
    PACKS_ASKED.call_once(|| {
        *DATA_ROOT.lock().unwrap_or_else(|e| e.into_inner()) = Some(resources.to_path_buf());
        for moment in ["ew_life_discovery", "ew_life_compose", "ew_life_entries", "ew_life_tags_pre", "ew_life_tags_post"] {
            LIFE_PASSED.lock().unwrap_or_else(|e| e.into_inner()).push(moment);
            super::paper_bridge::fire_free(moment, &json!({ "cause": "INITIAL" }));
        }
    });
}

/// Handlers that can register commands: they go in the tree the moment they're done.
fn commands_moment(game: &mut Game<'_>, cause: &str) {
    LIFE_PASSED.lock().unwrap_or_else(|e| e.into_inner()).push("ew_life_commands");
    if super::paper_bridge::hears("ew_life_commands") {
        let payload = json!({ "cause": cause }).to_string();
        runtime::lend(game, || runtime::fire("ew_life_commands", &payload));
        super::paper_bridge::take_commands();
        super::paper_bridge::on_trigger(game, "ew_life_done", Value::Null);
    }
}

/// At startup, with no world yet.
pub(in crate::game) fn install_free() {
    let _ = horizon_client::datapack::BEFORE_DISCOVERY.set(before_packs);
    let _ = horizon_world::datafix::BLOCK_ENTITY_DROPPED.set(block_entity_dropped);
    let _ = horizon_client::vanillagen::LOCATE_HOOK.set(locate_hook);
    let _ = horizon_client::paper_hub::PING_HOOK.set((ping_listened, ping_hook));
}

/// The server starts: the world family's own hooks, and the first `LifecycleEvents.COMMANDS`.
pub(in crate::game) fn on_server_starting(game: &mut Game<'_>) {
    install(game);
    commands_moment(game, "INITIAL");
}

/// A reload's end: `LifecycleEvents.COMMANDS` again.
pub(in crate::game) fn on_reload_end(game: &mut Game<'_>) {
    commands_moment(game, "RELOAD");
}

/// A reload's start: the tag moments, as Paper's lifecycle has them.
pub(in crate::game) fn on_reload_start(game: &mut Game<'_>) {
    for moment in ["ew_life_tags_pre", "ew_life_tags_post"] {
        if super::paper_bridge::hears(moment) {
            super::paper_bridge::on_trigger(game, moment, json!({ "cause": "RELOAD" }));
        }
    }
}

/// `CommandRegisteredEvent`, for each word of the mods' as the tree for players is built.
pub(in crate::game) fn sync_commands(game: &mut Game<'_>, commands: &mut Vec<crate::game::native_server::NativeCommand>) {
    if !super::paper_bridge::hears("ew_command_registered") {
        return;
    }
    let labels = crate::game::paper_events::mod_words();
    let word_of = |usage: &str| usage.split(' ').next().unwrap_or_default().to_owned();
    for label in &labels {
        let usages: Vec<String> = commands.iter().filter(|command| word_of(&command.usage) == *label).map(|command| command.usage.clone()).collect();
        let payload = json!({ "uuid": null, "label": label, "labels": labels, "usages": usages }).to_string();
        let (answer, carried) = runtime::lend(game, || runtime::fire_carried("ew_command_registered", &payload));
        if answer == 2 {
            commands.retain(|command| word_of(&command.usage) != *label);
            continue;
        }
        let Some(carried) = carried.and_then(|text| serde_json::from_str::<Value>(&text).ok()) else {
            continue;
        };
        if let Some(new) = carried["usages"].as_array() {
            commands.retain(|command| word_of(&command.usage) != *label);
            commands.extend(new.iter().filter_map(Value::as_str).map(crate::game::native_server::NativeCommand::new));
        }
        if let Some(by) = carried["taken"].as_str() {
            runtime::fire("ew_command_taken", &json!({ "label": label, "by": by }).to_string());
        }
    }
    super::paper_bridge::take_commands();
}

thread_local! {
    /// A ping or a query, as the listeners so far have left it.
    static PING_NOW: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    static QUERY_NOW: std::cell::RefCell<Option<Value>> = const { std::cell::RefCell::new(None) };
}

fn ping_listened() -> bool {
    super::paper_bridge::hears("ew_ping")
}

/// `PaperServerListPingEvent`, on the connection's thread.
fn ping_hook(id: u32, json: String) -> String {
    PING_NOW.set(json);
    super::paper_bridge::fire_free("ew_ping", &json!({ "uuid": null, "id": id }));
    PING_NOW.take()
}

/// `PortalCreateEvent` for the platform that meets a traveller to the End: false leaves it unbuilt.
pub(in crate::game) fn end_platform(game: &mut Game<'_>, planned: &[([i32; 3], &str)]) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_portal") {
        return true;
    }
    let rows: Vec<Value> = planned.iter().map(|(at, name)| json!([at[0], at[1], at[2], name])).collect();
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let acting = format!("{:032x}", game.native_acting_id());
    !on_refusable(game, "ew_portal", json!({ "uuid": acting, "level": level, "blocks": rows, "reason": "END_PLATFORM" }))
}

/// What `BlockLockCheckEvent`'s listeners said of the lock about to be checked.
static LOCK_SAID: Mutex<Option<((i32, i32, i32), bool, Option<ItemStack>)>> = Mutex::new(None);

/// For the check at `key`: whether a listener let them in, and the key a listener put in their hand.
pub(in crate::game) fn lock_said(key: (i32, i32, i32)) -> (bool, Option<ItemStack>) {
    match LOCK_SAID.lock().unwrap_or_else(|e| e.into_inner()).take() {
        Some((at, allow, stack)) if at == key => (allow, stack),
        _ => (false, None),
    }
}

/// `HangingPlaceEvent`, with the entity up and nothing yet paid for it: false takes it down.
pub(in crate::game) fn hanging_placed(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3], stack: &ItemStack, entity: u32) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_hanging_place") {
        return true;
    }
    let off_hand = game.hand == horizon_client::first_person::Hand::Off;
    !on_refusable(game, "ew_hanging_place", json!({ "pos": pos, "face": face, "stack": crate::game::paper_query::stack_json(stack), "off_hand": off_hand, "entity": entity }))
}

/// Whether shears on this block would take from a full hive.
pub(in crate::game) fn full_hive(game: &Game<'_>, pos: [i32; 3]) -> bool {
    let state = game.world.block(pos[0], pos[1], pos[2]);
    let Some(state) = game.pack.assets.registry.state(state) else {
        return false;
    };
    matches!(state.name.as_str(), "minecraft:beehive" | "minecraft:bee_nest") && state.properties.get("honey_level").is_some_and(|level| level.parse::<i32>().ok() == Some(5))
}

thread_local! {
    /// A search and a note, as the listeners so far have left them.
    static LOCATE_NOW: std::cell::RefCell<Option<Value>> = const { std::cell::RefCell::new(None) };
    static NOTE_NOW: std::cell::RefCell<Option<(u64, String)>> = const { std::cell::RefCell::new(None) };
}

/// `StructuresLocateEvent`, before any search: an answer ends the search with it.
fn locate_hook(level: &str, names: &mut Vec<String>, from: [i32; 3], radius: &mut i32, unexplored: &mut bool) -> Option<Option<(String, [i32; 3])>> {
    if !runtime::listens("ew_locate") {
        return None;
    }
    LOCATE_NOW.set(None);
    let level = format!("minecraft:{}", level.trim_start_matches("minecraft:"));
    let payload = json!({ "uuid": null, "level": level, "structures": names, "origin": from, "radius": *radius, "unexplored": *unexplored });
    let refused = super::paper_bridge::fire_free("ew_locate", &payload) == 2;
    let said = LOCATE_NOW.take();
    if let Some(row) = said.as_ref().and_then(|said| said["result"].as_array()) {
        let at = |i: usize| row.get(i).and_then(Value::as_f64).unwrap_or(0.0).floor() as i32;
        return Some(Some((row.first().and_then(Value::as_str).unwrap_or_default().to_owned(), [at(1), at(2), at(3)])));
    }
    if refused {
        return Some(None);
    }
    if let Some(said) = said {
        *names = said["structures"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect();
        *radius = said["radius"].as_i64().unwrap_or(i64::from(*radius)) as i32;
        *unexplored = said["unexplored"] == true;
    }
    None
}

thread_local! {
    /// What a listener left of a tree.
    static GROW_KEPT: std::cell::RefCell<Option<Vec<[i32; 3]>>> = const { std::cell::RefCell::new(None) };
    static GROW_REFUSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `BlockIgniteEvent` for what the game does, with the game to lend: false leaves it unlit.
fn ignited(game: &mut Game<'_>, at: [i32; 3], cause: &str, player: bool, entity: Option<u32>, block: Option<[i32; 3]>) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_ignite") {
        return true;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let mut payload = json!({ "level": level, "pos": at, "cause": cause, "entity": entity, "block": block });
    if !player {
        payload["uuid"] = Value::Null;
    }
    !on_refusable(game, "ew_ignite", payload)
}

pub(in crate::game) fn ignited_by_item(game: &mut Game<'_>, at: [i32; 3], fire_charge: bool) -> bool {
    ignited(game, at, if fire_charge { "FIREBALL" } else { "FLINT_AND_STEEL" }, true, None, None)
}

pub(in crate::game) fn ignited_by_block(game: &mut Game<'_>, at: [i32; 3], cause: &str, source: [i32; 3]) -> bool {
    ignited(game, at, cause, false, None, Some(source))
}

pub(in crate::game) fn ignited_by_entity(game: &mut Game<'_>, at: [i32; 3], cause: &str, entity: Option<u32>) -> bool {
    ignited(game, at, cause, false, entity.filter(|id| *id != 0), None)
}

/// The arrow at `at`, as an arrow's events don't name theirs.
pub(in crate::game) fn arrow_at(game: &Game<'_>, at: [f64; 3]) -> Option<u32> {
    game.creatures.projectiles.iter().find(|p| p.arrow.is_some() && (p.position.x - at[0]).abs() < 0.001 && (p.position.y - at[1]).abs() < 0.001 && (p.position.z - at[2]).abs() < 0.001).map(|p| p.id)
}

/// What the level's own run has left for its listeners: it can't lend the game from inside, so it stops
/// short of the thing, and it's done here once they've spoken.
static ASKED: Mutex<Vec<(horizon_world::level::Dimension, horizon_world::level::Asked, Value)>> = Mutex::new(Vec::new());

/// True if a listener wants a say: the level stops short, and `run_asked` does the rest.
fn ask_hook(dimension: horizon_world::level::Dimension, asked: &horizon_world::level::Asked) -> bool {
    use horizon_world::level::Asked;
    let event = match asked {
        Asked::Ignite { .. } => "ew_ignite",
        Asked::Burn { .. } => "ew_burn",
        Asked::Tnt { .. } => "ew_tnt",
        Asked::Cauldron { .. } => "ew_cauldron",
        Asked::Note { .. } => "ew_note",
        Asked::Bell { .. } => "ew_bell",
        Asked::Portal { .. } => "ew_portal",
        Asked::Grow { .. } => "ew_grow",
        Asked::Change { kind: "spread", .. } => "block_spread",
        Asked::Change { .. } => "block_change",
        Asked::Destroy { .. } => "ew_destroy",
        Asked::Piston { .. } => "piston",
        Asked::Sponge { .. } => "ew_sponge",
        Asked::Flow { .. } => if runtime::listens("ew_block_broke") { "ew_block_broke" } else { "fluid_flow" },
    };
    if !runtime::listens(event) {
        return false;
    }
    let ringer = if matches!(asked, Asked::Bell { .. }) { BELL_RINGER.lock().unwrap_or_else(|e| e.into_inner()).take() } else { None };
    ASKED.lock().unwrap_or_else(|e| e.into_inner()).push((dimension, asked.clone(), ringer.unwrap_or(Value::Null)));
    true
}

fn set_now(game: &mut Game<'_>, pos: horizon_core::position::BlockPos, state: horizon_world::registry::BlockStateId, flags: i32) {
    game.with_level(|b, level, ticks| b.set_block(level, ticks, pos, state, flags));
}

/// What the level left, one by one, in the level it came from.
pub(in crate::game) fn run_asked(game: &mut Game<'_>) {
    if !horizon_world::level::ASK_WAITING.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    // What's done here has had its say, whichever way the level's changes are being noted.
    let noting = crate::game::paper_bridge::noting_changes(false);
    while horizon_world::level::ASK_WAITING.swap(false, std::sync::atomic::Ordering::Relaxed) {
        run_broke(game);
        let asked = std::mem::take(&mut *ASKED.lock().unwrap_or_else(|e| e.into_inner()));
        for (dimension, asked, ringer) in asked {
            run_one(game, dimension, asked, ringer);
        }
    }
    crate::game::paper_bridge::noting_changes(noting);
}

/// A block's change in the words of `block_change`.
fn change_payload(game: &Game<'_>, kind: &str, level: &str, pos: [i32; 3], new: horizon_world::registry::BlockStateId) -> Value {
    let registry = game.pack.assets.registry;
    let old = game.world.block(pos[0], pos[1], pos[2]);
    let number = |state: horizon_world::registry::BlockStateId, key: &str| registry.state(state).and_then(|state| state.properties.get(key)).and_then(|value| value.parse::<i32>().ok()).unwrap_or(0);
    let powered = |state: horizon_world::registry::BlockStateId| registry.state(state).is_some_and(|state| state.properties.get("powered").is_some_and(|value| value == "true"));
    let current = |state: horizon_world::registry::BlockStateId| if powered(state) { 15 } else { 0 };
    json!({ "kind": kind, "level": level, "pos": pos, "old": state_name(old), "new": state_name(new), "old_level": number(old, "level"), "new_level": number(new, "level"), "old_current": current(old), "new_current": current(new), "binary": kind == "redstone" })
}

fn run_one(game: &mut Game<'_>, dimension: horizon_world::level::Dimension, asked: horizon_world::level::Asked, ringer: Value) {
    use horizon_core::position::BlockPos;
    use horizon_world::level::Asked;
    {
        let level = level_name(dimension);
        game.native_in_level(level, |game| {
            use crate::game::paper_bridge::{hears, on_refusable};
            let at = |pos: BlockPos| [pos.x, pos.y, pos.z];
            match asked {
                Asked::Change { kind, at: pos, source, state, doing } => {
                    let refused = match (kind, source) {
                        ("spread", Some(source)) => on_refusable(game, "block_spread", json!({ "kind": kind, "level": level, "from": at(source), "to": at(pos), "new": state_name(state) })),
                        _ => {
                            let payload = change_payload(game, kind, level, at(pos), state);
                            on_refusable(game, "block_change", payload)
                        }
                    };
                    if !refused {
                        game.with_level(|b, level, ticks| b.run_then(level, ticks, &doing));
                    }
                }
                Asked::Destroy { at: pos, state, drop, limit } => {
                    if game.world.block(pos.x, pos.y, pos.z) != state {
                        return;
                    }
                    let payload = json!({ "level": level, "pos": at(pos), "state": state_name(state), "new": "minecraft:air", "drop": drop }).to_string();
                    let code = runtime::lend(game, || runtime::fire("ew_destroy", &payload));
                    if code == 2 {
                        return;
                    }
                    let flags = (code - 3).max(0);
                    let shown = EFFECT.lock().unwrap_or_else(|e| e.into_inner()).take();
                    let effect = match (flags & 2 != 0, flags & 4 != 0) {
                        (true, _) => None,
                        (false, true) => shown.and_then(|data| state_of_text(&data)).or(Some(state)),
                        _ => Some(state),
                    };
                    game.with_level(|b, level, ticks| b.destroy_told(level, ticks, pos, state, drop && flags & 1 == 0, effect, limit));
                }
                Asked::Sponge { at: pos, blocks } => {
                    let places: Vec<[i32; 3]> = blocks.iter().map(|(place, _)| at(*place)).collect();
                    let states: Vec<String> = blocks.iter().map(|(_, state)| state_name(*state)).collect();
                    *FREE_KEPT.lock().unwrap_or_else(|e| e.into_inner()) = None;
                    let refused = on_refusable(game, "ew_sponge", json!({ "level": level, "pos": at(pos), "blocks": places, "states": states }));
                    let kept = FREE_KEPT.lock().unwrap_or_else(|e| e.into_inner()).take();
                    if refused && kept.as_ref().is_none_or(Vec::is_empty) {
                        return;
                    }
                    let kept: Option<Vec<BlockPos>> = kept.map(|kept| kept.iter().map(|place| BlockPos::new(place[0], place[1], place[2])).collect());
                    game.with_level(|b, level, ticks| b.sponge_absorbs(level, ticks, pos, kept.as_deref()));
                }
                Asked::Piston { kind, at: pos, state, a, param_b, direction, sticky, blocks } => {
                    let name = match direction.normal() {
                        [1, 0, 0] => "EAST",
                        [-1, 0, 0] => "WEST",
                        [0, 1, 0] => "UP",
                        [0, -1, 0] => "DOWN",
                        [0, 0, 1] => "SOUTH",
                        _ => "NORTH",
                    };
                    let push: Vec<[i32; 3]> = blocks.iter().map(|place| at(*place)).collect();
                    if !on_refusable(game, "piston", json!({ "kind": kind, "level": level, "pos": at(pos), "direction": name, "sticky": sticky, "push": push })) {
                        game.with_level(|b, level, ticks| b.piston_told(level, ticks, pos, state, a, param_b));
                    }
                }
                Asked::Flow { kind, to, state, direction, target, rest } => {
                    let step = direction.normal();
                    let from = [to.x - step[0], to.y - step[1], to.z - step[2]];
                    if game.world.block(to.x, to.y, to.z) != state {
                        return;
                    }
                    if on_refusable(game, "fluid_flow", json!({ "fluid": format!("{kind:?}"), "level": level, "from": from, "to": at(to) })) {
                        return;
                    }
                    // Water breaks what it takes the place of, with the block still there to see.
                    let breaks = matches!(kind, horizon_world::fluid::FluidKind::Water | horizon_world::fluid::FluidKind::FlowingWater) && !game.res.facts().is_air(state);
                    if breaks && hears("ew_block_broke") {
                        TAKE_DROPS.set(true);
                        game.with_level(|_, level, _| level.drop_resources_now(to, state));
                        TAKE_DROPS.set(false);
                        let drops = TAKEN_DROPS.take();
                        let kept = broke_said(game, level, at(to), from, drops);
                        for stack in kept {
                            if &*stack.item != "minecraft:air" && stack.count > 0 {
                                game.pop_resource(at(to), stack);
                            }
                        }
                    }
                    let dropped = breaks && hears("ew_block_broke");
                    game.with_level(|b, level, ticks| b.fluid_flows(level, ticks, kind, to, state, direction, target, rest, dropped));
                }
                Asked::Ignite { cause, at: pos, source, state } => {
                    if !on_refusable(game, "ew_ignite", json!({ "uuid": null, "level": level, "pos": at(pos), "cause": cause, "block": at(source) }))
                        && (cause != "SPREAD" || !on_refusable(game, "block_spread", json!({ "kind": "fire", "level": level, "from": at(source), "to": at(pos), "new": state_name(state) })))
                        && {
                            let now = game.world.block(pos.x, pos.y, pos.z);
                            game.with_level(|b, _, _| b.is_air(now))
                        }
                    {
                        set_now(game, pos, state, 3);
                    }
                }
                Asked::Burn { at: pos, source, was, to, tnt } => {
                    if game.world.block(pos.x, pos.y, pos.z) != was {
                        return;
                    }
                    if on_refusable(game, "ew_burn", json!({ "uuid": null, "level": level, "pos": at(pos), "block": at(source) })) {
                        return;
                    }
                    if tnt && !tnt_primes(game, at(pos), "FIRE", false, None, Some(at(source))) {
                        return;
                    }
                    match to {
                        Some(state) => set_now(game, pos, state, 3),
                        None => {
                            game.with_level(|b, level, ticks| b.remove_block(level, ticks, pos, false));
                        }
                    }
                    if tnt {
                        game.spawn_primed_tnt(pos, false);
                    }
                }
                Asked::Tnt { cause, at: pos, remove } => {
                    if tnt_primes(game, at(pos), cause, false, None, None) {
                        game.spawn_primed_tnt(pos, false);
                        if remove {
                            game.with_level(|b, level, ticks| b.remove_block(level, ticks, pos, false));
                        }
                    }
                }
                Asked::Cauldron { at: pos, old, new, event } => {
                    let mut payload = cauldron_payload(level.to_owned(), at(pos), &state_name(old), &state_name(new), "NATURAL_FILL");
                    payload["uuid"] = Value::Null;
                    CAULDRON_NOW.set(None);
                    if on_refusable(game, "ew_cauldron", payload) {
                        return;
                    }
                    let state = CAULDRON_NOW.take().and_then(|text| state_of_text(&text)).unwrap_or(new);
                    set_now(game, pos, state, 3);
                    game.post_block_game_event("minecraft:block_change", at(pos), None, Some(state));
                    if let Some(event) = event {
                        game.level_event_at(event, at(pos), 0);
                    }
                }
                Asked::Note { at: pos, state } => {
                    NOTE_NOW.set(None);
                    if on_refusable(game, "ew_note", json!({ "uuid": null, "level": level, "pos": at(pos), "state": state_name(state) })) {
                        return;
                    }
                    let sounds = NOTE_NOW.take().and_then(|(note, instrument)| {
                        let (name, mut properties) = state_parts(&state_name(state));
                        properties.insert("note".to_owned(), note.to_string());
                        properties.insert("instrument".to_owned(), instrument);
                        STATE_IDS.get()?.get(&state_key(&name, &properties)).map(|id| horizon_world::registry::BlockStateId(*id as _))
                    });
                    game.with_level(|b, level, ticks| b.note_sounds(level, ticks, pos, state, sounds));
                }
                Asked::Bell { at: pos, state, direction } => {
                    let payload = json!({ "ring": true, "uuid": ringer["uuid"], "entity": ringer["entity"], "level": level, "pos": at(pos), "face": direction.normal() });
                    if !on_refusable(game, "ew_bell", payload) {
                        game.with_level(|b, level, ticks| b.bell_rings(level, ticks, pos, state, direction));
                    }
                }
                Asked::Portal { reason, rows } => {
                    let blocks: Vec<Value> = rows.iter().map(|(place, state, _)| json!([place.x, place.y, place.z, state_name(*state)])).collect();
                    if !on_refusable(game, "ew_portal", json!({ "uuid": null, "level": level, "blocks": blocks, "reason": reason })) {
                        for (place, state, flags) in rows {
                            set_now(game, place, state, flags);
                        }
                    }
                }
                Asked::Grow { at: pos, feature, writes, stood } => {
                    let rows: Vec<Value> = writes.iter().map(|(place, _, new)| json!([place[0], place[1], place[2], state_name(*new)])).collect();
                    GROW_KEPT.set(None);
                    let refused = on_refusable(game, "ew_grow", json!({ "uuid": null, "level": level, "pos": at(pos), "feature": feature, "blocks": rows }));
                    let kept = GROW_KEPT.take();
                    GROW_REFUSED.set(refused);
                    for (place, old, _) in writes.into_iter().rev() {
                        if refused || kept.as_ref().is_some_and(|kept| !kept.contains(&place)) {
                            set_now(game, BlockPos::new(place[0], place[1], place[2]), old, 0);
                        }
                    }
                    if refused {
                        for (place, state) in stood {
                            set_now(game, place, state, 0);
                        }
                    }
                }
            }
        });
    }
}

thread_local! {
    /// While set, a block's drops are kept for its event, which comes before the block goes.
    static TAKE_DROPS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static TAKEN_DROPS: std::cell::RefCell<Vec<ItemStack>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A piston's breaks, each with its drops, which wait here for a game to lend.
static BROKE: Mutex<Vec<(crate::game::LevelDimension, [i32; 3], [i32; 3], Vec<ItemStack>)>> = Mutex::new(Vec::new());

/// `BlockBreakBlockEvent`: what's left to drop.
fn broke_said(game: &mut Game<'_>, level: &str, pos: [i32; 3], source: [i32; 3], drops: Vec<ItemStack>) -> Vec<ItemStack> {
    let items: Vec<Value> = drops.iter().map(crate::game::paper_query::stack_json).collect();
    if crate::game::paper_bridge::on_refusable(game, "ew_block_broke", json!({ "level": level, "pos": pos, "source": source, "items": items })) {
        return FREE_DROPS.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or_default();
    }
    drops
}

fn run_broke(game: &mut Game<'_>) {
    let broke = std::mem::take(&mut *BROKE.lock().unwrap_or_else(|e| e.into_inner()));
    for (dimension, pos, source, drops) in broke {
        let level = match dimension {
            crate::game::LevelDimension::Overworld => "minecraft:overworld",
            crate::game::LevelDimension::Nether => "minecraft:the_nether",
            crate::game::LevelDimension::End => "minecraft:the_end",
        };
        game.native_in_level(level, |game| {
            for stack in broke_said(game, level, pos, source, drops) {
                if &*stack.item != "minecraft:air" && stack.count > 0 {
                    game.pop_resource(pos, stack);
                }
            }
        });
    }
}

/// A portal for a traveller: asked of its listeners with the plan in hand, as the traveller is waiting.
pub(in crate::game) fn portal_planned(game: &mut Game<'_>, rows: Vec<([i32; 3], horizon_world::registry::BlockStateId)>) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_portal") {
        return true;
    }
    let blocks: Vec<Value> = rows.iter().map(|(at, state)| json!([at[0], at[1], at[2], state_name(*state)])).collect();
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let traveller = PORTAL_TRAVELLER.lock().unwrap_or_else(|e| e.into_inner()).take();
    !on_refusable(game, "ew_portal", json!({ "uuid": traveller, "level": level, "blocks": blocks, "reason": "NETHER_PAIR" }))
}

/// `BlockCanBuildEvent`, with what the check said: what the placing is to take for an answer.
pub(in crate::game) fn can_build(game: &mut Game<'_>, clicked: [i32; 3], state: horizon_world::registry::BlockStateId, fits: bool) -> bool {
    use crate::game::paper_bridge::{hears, on_trigger};
    if !hears("ew_can_build") {
        return fits;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let off_hand = game.hand == horizon_client::first_person::Hand::Off;
    CAN_BUILD.set(None);
    on_trigger(game, "ew_can_build", json!({ "level": level, "pos": clicked, "state": state_name(state), "buildable": fits, "off_hand": off_hand }));
    CAN_BUILD.take().unwrap_or(fits)
}

fn block_state_text(state: &horizon_world::chunk::BlockState) -> String {
    if state.properties.is_empty() {
        return state.name.to_string();
    }
    let properties: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
    format!("{}[{}]", state.name, properties.join(","))
}

fn cauldron_level(text: &str) -> i64 {
    let (name, properties) = state_parts(text);
    match name.as_str() {
        "minecraft:cauldron" => 0,
        "minecraft:lava_cauldron" => 3,
        _ => properties.get("level").and_then(|level| level.parse().ok()).unwrap_or(0),
    }
}

fn cauldron_payload(level: String, pos: [i32; 3], old: &str, new: &str, reason: &str) -> Value {
    json!({ "level": level, "pos": pos, "old": old, "new": new, "old_level": cauldron_level(old), "new_level": cauldron_level(new), "reason": reason })
}

thread_local! {
    static CAULDRON_NOW: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// `CauldronLevelChangeEvent` for a player: `None` to stay, else the state to go to.
pub(in crate::game) fn cauldron_changes(game: &mut Game<'_>, pos: [i32; 3], old: &str, new: &str, reason: &str) -> Option<String> {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_cauldron") {
        return Some(new.to_owned());
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    CAULDRON_NOW.set(None);
    let payload = cauldron_payload(level, pos, old, new, reason);
    if on_refusable(game, "ew_cauldron", payload) {
        return None;
    }
    Some(CAULDRON_NOW.take().unwrap_or_else(|| new.to_owned()))
}

/// What a cauldron use would do, tried on a copy of the hand: `Some(None)` if listeners refused it,
/// `Some(Some(id))` if they changed what it becomes, `None` to go on.
pub(in crate::game) fn cauldron_asked(game: &mut Game<'_>, pos: [i32; 3], state: &horizon_world::chunk::BlockState, under_water: bool, removes_dye: bool) -> Option<Option<horizon_world::registry::BlockStateId>> {
    use horizon_client::cauldron::{self, Outcome};
    struct Trial {
        hand: Option<ItemStack>,
        creative: bool,
    }
    impl cauldron::Player for Trial {
        fn creative(&self) -> bool {
            self.creative
        }
        fn hand(&mut self) -> &mut Option<ItemStack> {
            &mut self.hand
        }
        fn add(&mut self, stack: &mut ItemStack) -> bool {
            stack.count = 0;
            true
        }
        fn contains(&self, _: &ItemStack) -> bool {
            true
        }
        fn drop_stack(&mut self, _: ItemStack) {}
    }
    if !super::paper_bridge::hears("ew_cauldron") {
        return None;
    }
    let hand = game.held().cloned();
    let item = hand.as_ref()?.item.to_string();
    let mut trial = Trial { hand, creative: game.creative };
    let result = cauldron::interact(state, under_water, removes_dye, &mut trial);
    if result.outcome == Outcome::Pass {
        return None;
    }
    let block = result.block?;
    let reason = match item.as_str() {
        "minecraft:bucket" => "BUCKET_FILL",
        "minecraft:glass_bottle" => "BOTTLE_FILL",
        "minecraft:potion" => "BOTTLE_EMPTY",
        name if name.ends_with("_bucket") => "BUCKET_EMPTY",
        name if name.ends_with("_banner") => "BANNER_WASH",
        name if name.ends_with("shulker_box") => "SHULKER_WASH",
        _ => "ARMOR_WASH",
    };
    let new = block_state_text(&block);
    match cauldron_changes(game, pos, &block_state_text(state), &new, reason) {
        None => Some(None),
        Some(said) if said == new => None,
        Some(said) => state_of_text(&said).map(Some),
    }
}

/// A burning player in a cauldron: the state it goes to, or `None` to stay.
pub(in crate::game) fn cauldron_extinguishes(game: &mut Game<'_>, pos: [i32; 3], old: &horizon_world::chunk::BlockState, new: horizon_world::registry::BlockStateId) -> Option<horizon_world::registry::BlockStateId> {
    let said = cauldron_changes(game, pos, &block_state_text(old), &state_name(new), "EXTINGUISH")?;
    Some(state_of_text(&said).unwrap_or(new))
}

/// Who a bell's next ring belongs to, for what the level's run doesn't say.
static BELL_RINGER: Mutex<Option<Value>> = Mutex::new(None);

pub(in crate::game) fn bell_by_mob(mob: u32) {
    *BELL_RINGER.lock().unwrap_or_else(|e| e.into_inner()) = Some(json!({ "entity": mob }));
}

/// A shot that may ring a bell: its shooter rings it, if that's a player.
pub(in crate::game) fn bell_shot(game: &mut Game<'_>, owner: Option<horizon_client::mobs::Who>, guest: Option<u32>) -> bool {
    use horizon_client::mobs::Who;
    let player = match (owner, guest) {
        (Some(Who::Player), Some(conn)) => game.lan.as_ref().and_then(|host| host.hub.guest(conn)).map(|guest| guest.uuid),
        (Some(Who::Player), None) => Some(game.native_acting_id()),
        _ => None,
    };
    *BELL_RINGER.lock().unwrap_or_else(|e| e.into_inner()) = Some(player.map_or(Value::Null, |player| json!({ "uuid": format!("{player:032x}") })));
    true
}

/// `ServerCommandEvent` for a command block's line, as the block: what to run, or `None`.
pub(in crate::game) fn server_command(game: &mut Game<'_>, line: String, source: crate::game::commands::execute::Source) -> Option<String> {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_console") {
        return Some(line);
    }
    let payload = json!({ "uuid": null, "line": line });
    let refused = game.with_source(source, |game| on_refusable(game, "ew_console", payload));
    let changed = console_line();
    if refused && changed.is_none() {
        return None;
    }
    Some(changed.unwrap_or(line))
}

/// What the event said of the smelt about to happen, and the furnace as it was.
pub(in crate::game) struct Smelt {
    refused: bool,
    result: Option<ItemStack>,
    before: horizon_client::containers::Furnace,
}

static FURNACE_RESULT: Mutex<Option<ItemStack>> = Mutex::new(None);

/// `BlockCookEvent`, if this tick finishes a cook: tried on a copy, so what fires is what's about to happen.
pub(in crate::game) fn furnace_smelt(game: &mut Game<'_>, pos: (i32, i32, i32)) -> Option<Smelt> {
    use horizon_client::furnace::FurnaceCooking;
    use horizon_client::containers::BlockEntity;
    let hears = super::paper_bridge::hears("ew_furnace_done");
    if !hears {
        return None;
    }
    let Some(BlockEntity::Furnace(furnace)) = game.world.block_entities.get(&pos) else {
        return None;
    };
    let before = furnace.clone();
    let mut trial = before.clone();
    trial.tick(world_recipes!(game), world_items!(game), game.res.providers());
    if trial.recipes_used == before.recipes_used {
        return None;
    }
    let input = before.items[0].clone()?;
    let recipe = world_recipes!(game).cook(before.kind.recipe_type(), &input)?;
    let (stack, id) = (crate::game::paper_query::stack_json(&recipe.result), recipe.id.clone());
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let payload = json!({ "uuid": null, "furnace": true, "level": level, "pos": [pos.0, pos.1, pos.2], "source": crate::game::paper_query::stack_json(&input), "stack": stack, "recipe": id });
    *FURNACE_RESULT.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let said = super::paper_bridge::on_refusable(game, "ew_furnace_done", payload);
    let result = FURNACE_RESULT.lock().unwrap_or_else(|e| e.into_inner()).take();
    Some(Smelt { refused: said && result.is_none(), result, before })
}

/// What the event left, over the furnace's tick.
pub(in crate::game) fn furnace_smelted(game: &mut Game<'_>, pos: (i32, i32, i32), smelt: Option<Smelt>) {
    use horizon_client::containers::BlockEntity;
    let Some(smelt) = smelt else {
        return;
    };
    if !smelt.refused && smelt.result.is_none() {
        return;
    }
    let Some(BlockEntity::Furnace(furnace)) = game.world.block_entities.get_mut(&pos) else {
        return;
    };
    furnace.items[2] = smelt.before.items[2].clone();
    if smelt.refused {
        furnace.items[0] = smelt.before.items[0].clone();
        furnace.recipes_used = smelt.before.recipes_used.clone();
        if smelt.before.items[1].as_ref().is_some_and(|fuel| &*fuel.item == "minecraft:bucket") {
            furnace.items[1] = smelt.before.items[1].clone();
        }
        return;
    }
    if let Some(result) = smelt.result {
        match &mut furnace.items[2] {
            Some(out) if out.same_item_same_components(&result) => out.count += result.count,
            Some(_) => {}
            slot @ None => *slot = Some(result),
        }
    }
}

/// A brewing stand has started.
pub(in crate::game) fn brewing_started(game: &mut Game<'_>, pos: (i32, i32, i32)) {
    use horizon_client::containers::BlockEntity;
    if !super::paper_bridge::hears("ew_brewing") {
        return;
    }
    let Some(BlockEntity::Brewing(stand)) = game.world.block_entities.get(&pos) else {
        return;
    };
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let payload = json!({ "uuid": null, "level": level, "pos": [pos.0, pos.1, pos.2], "stack": stand.items[3].as_ref().map(crate::game::paper_query::stack_json), "brew_time": stand.brew_time, "total": stand.total_brew_time });
    super::paper_bridge::on_trigger(game, "ew_brewing", payload);
}

/// Which beacons stand lit, by level and place, for the one that's gone.
static BEACONS_LIT: Mutex<Vec<(String, (i32, i32, i32))>> = Mutex::new(Vec::new());
static BEACON_TURNED: Mutex<Option<bool>> = Mutex::new(None);

pub(in crate::game) fn beacon_turned(on: bool) {
    *BEACON_TURNED.lock().unwrap_or_else(|e| e.into_inner()) = Some(on);
}

/// A beacon's tick is over and it's back in the level, for a listener to read.
pub(in crate::game) fn beacon_ticked(game: &mut Game<'_>, key: (i32, i32, i32)) {
    use horizon_client::containers::BlockEntity;
    let turned = BEACON_TURNED.lock().unwrap_or_else(|e| e.into_inner()).take();
    if !super::paper_bridge::hears("ew_beacon") {
        return;
    }
    let Some(BlockEntity::Beacon(beacon)) = game.world.block_entities.get(&key) else {
        return;
    };
    let lit = beacon.levels > 0;
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    {
        let mut all = BEACONS_LIT.lock().unwrap_or_else(|e| e.into_inner());
        let row = all.iter().position(|(known, at)| *known == level && *at == key);
        match (row, lit) {
            (None, true) => all.push((level.clone(), key)),
            (Some(row), false) => {
                all.swap_remove(row);
            }
            _ => {}
        }
    }
    if let Some(on) = turned {
        super::paper_bridge::on_trigger(game, "ew_beacon", json!({ "uuid": null, "level": level, "pos": [key.0, key.1, key.2], "on": on }));
    }
}

pub(in crate::game) fn vault_listened() -> bool {
    super::paper_bridge::hears("ew_dispense_loot")
}

/// `BlockDispenseLootEvent`, for a vault's roll: what to give, or `None` to give nothing and take no key.
pub(in crate::game) fn vault_loot(game: &mut Game<'_>, pos: [i32; 3], rolled: Vec<ItemStack>) -> Option<Vec<ItemStack>> {
    let acting = format!("{:032x}", game.native_acting_id());
    dispense_loot(game, pos, acting.into(), None, rolled)
}

fn dispense_loot(game: &mut Game<'_>, pos: [i32; 3], who: Value, table: Option<String>, rolled: Vec<ItemStack>) -> Option<Vec<ItemStack>> {
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let ejects: Vec<Value> = rolled.iter().map(crate::game::paper_query::stack_json).collect();
    DROPS.lock().unwrap_or_else(|e| e.into_inner()).clear();
    LOOT_LISTED.store(false, std::sync::atomic::Ordering::Relaxed);
    let payload = json!({ "uuid": who, "level": level, "pos": pos, "table": table, "ejects": ejects });
    if super::paper_bridge::on_refusable(game, "ew_dispense_loot", payload) {
        return None;
    }
    if LOOT_LISTED.swap(false, std::sync::atomic::Ordering::Relaxed) {
        return Some(std::mem::take(&mut *DROPS.lock().unwrap_or_else(|e| e.into_inner())));
    }
    Some(rolled)
}

/// Whether a listener left a list of its own for `ev_world_set_drops` to have set.
static LOOT_LISTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A trial spawner's reward, as it's about to be thrown out.
pub(in crate::game) fn trial_loot(game: &mut Game<'_>, pos: [i32; 3], table: Option<String>, ejected: Vec<(ItemStack, [f64; 3])>) -> Vec<(ItemStack, [f64; 3])> {
    if !super::paper_bridge::hears("ew_dispense_loot") || ejected.is_empty() {
        return ejected;
    }
    let throw = ejected.first().map_or([0.0, 0.2, 0.0], |row| row.1);
    let rolled: Vec<ItemStack> = ejected.iter().map(|row| row.0.clone()).collect();
    match dispense_loot(game, pos, Value::Null, table, rolled.clone()) {
        None => Vec::new(),
        Some(items) if items == rolled => ejected,
        Some(items) => items.into_iter().map(|stack| (stack, throw)).collect(),
    }
}

static VAULT_SHOWS: Mutex<Option<Option<ItemStack>>> = Mutex::new(None);

/// What a vault shows now, for after its tick.
pub(in crate::game) fn vault_shown(game: &Game<'_>, key: (i32, i32, i32)) -> Option<Option<ItemStack>> {
    use horizon_client::containers::BlockEntity;
    if !super::paper_bridge::hears("ew_vault_display") {
        return None;
    }
    match game.world.block_entities.get(&key) {
        Some(BlockEntity::Vault(vault)) => Some(vault.display_item.clone()),
        _ => None,
    }
}

/// `VaultDisplayItemEvent`, if the tick chose another item from the vault's table.
pub(in crate::game) fn vault_display(game: &mut Game<'_>, key: (i32, i32, i32), before: Option<Option<ItemStack>>) {
    use horizon_client::containers::BlockEntity;
    let Some(before) = before else {
        return;
    };
    let Some(BlockEntity::Vault(vault)) = game.world.block_entities.get(&key) else {
        return;
    };
    if vault.display_item == before || !vault.items_to_eject.is_empty() {
        return;
    }
    let Some(chosen) = vault.display_item.clone() else {
        return;
    };
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let payload = json!({ "uuid": null, "level": level, "pos": [key.0, key.1, key.2], "stack": crate::game::paper_query::stack_json(&chosen) });
    *VAULT_SHOWS.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let refused = super::paper_bridge::on_refusable(game, "ew_vault_display", payload);
    let said = VAULT_SHOWS.lock().unwrap_or_else(|e| e.into_inner()).take();
    let shows = if refused { Some(before) } else { said };
    if let (Some(shows), Some(BlockEntity::Vault(vault))) = (shows, game.world.block_entities.get_mut(&key)) {
        vault.display_item = shows;
        game.mark_block_entity_updated(key);
    }
}

/// `RaidStopEvent`.
pub(in crate::game) fn raid_stopped(game: &mut Game<'_>, raid: i32, reason: &str) {
    if super::paper_bridge::hears("ew_raid_stop") {
        let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
        super::paper_bridge::on_trigger(game, "ew_raid_stop", json!({ "uuid": null, "level": level, "raid": raid, "reason": reason }));
    }
}

/// `RaidSpawnWaveEvent`: its raiders are in the level.
pub(in crate::game) fn raid_wave_done(game: &mut Game<'_>, raid: i32, pos: [i32; 3], wave: i32) {
    if super::paper_bridge::hears("ew_raid_wave") {
        let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
        super::paper_bridge::on_trigger(game, "ew_raid_wave", json!({ "uuid": null, "level": level, "raid": raid, "pos": pos, "wave": wave }));
    }
}

thread_local! {
    static PROFILE_SAID: std::cell::RefCell<Option<Value>> = const { std::cell::RefCell::new(None) };
}

/// An event of the profile family, fired on the thread that's asking, and what listeners left.
fn profile_fire(query: &Value) -> Option<String> {
    let event = query["event"].as_str()?;
    let outer = PROFILE_SAID.take();
    super::paper_bridge::fire_free(event, &query["payload"]);
    let said = PROFILE_SAID.replace(outer);
    Some(said.unwrap_or(Value::Null).to_string())
}

/// A block entity the data fixer is taking out of a chunk.
fn block_entity_dropped(dimension: &str, kind: &str, pos: [i32; 3], data: Option<&Tag>) {
    if !runtime::listens("ew_data_fixer_removed") {
        return;
    }
    let level = format!("minecraft:{}", dimension.trim_start_matches("minecraft:"));
    let data = data.map_or(Value::Null, crate::game::commands::paper_entities::tag_json);
    super::paper_bridge::fire_free("ew_data_fixer_removed", &json!({ "level": level, "kind": kind, "pos": pos, "data": data }));
}

/// `VehicleEnterEvent`: false keeps them out. `mob` is who's getting in, if not the player in hand.
pub(in crate::game) fn vehicle_enters(game: &mut Game<'_>, vehicle: u32, mob: Option<u32>) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_vehicle_enter") {
        return true;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let mut payload = json!({ "level": level, "vehicle": vehicle, "entity": mob });
    if mob.is_some() {
        payload["uuid"] = Value::Null;
    }
    !on_refusable(game, "ew_vehicle_enter", payload)
}

/// `VehicleExitEvent` for a mob: false keeps it in, if the vehicle stays.
pub(in crate::game) fn mob_exits(game: &mut Game<'_>, vehicle: u32, mob: u32, stays: bool) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_vehicle_exit") {
        return true;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    !on_refusable(game, "ew_vehicle_exit", json!({ "uuid": null, "level": level, "vehicle": vehicle, "entity": mob, "cancellable": stays })) || !stays
}

/// `TNTPrimeEvent` for what the game does.
pub(in crate::game) fn tnt_primes(game: &mut Game<'_>, pos: [i32; 3], cause: &str, player: bool, entity: Option<u32>, block: Option<[i32; 3]>) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_tnt") {
        return true;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let mut payload = json!({ "level": level, "pos": pos, "cause": cause, "entity": entity, "block": block });
    if !player {
        payload["uuid"] = Value::Null;
    }
    !on_refusable(game, "ew_tnt", payload)
}

/// A `BlockExplodeEvent` listener's chance of a block dropping, for the blast under way.
static BLAST_YIELD: Mutex<Option<f32>> = Mutex::new(None);
/// What a block's own blast was before it went: a bed or an anchor.
static EXPLODED: Mutex<Option<String>> = Mutex::new(None);

pub(in crate::game) fn blast_begins() -> bool {
    *BLAST_YIELD.lock().unwrap_or_else(|e| e.into_inner()) = None;
    true
}

/// The radius the loot tables roll against, which is one over the chance of a drop.
pub(in crate::game) fn blast_decay(decays: bool, radius: f32) -> Option<f32> {
    match *BLAST_YIELD.lock().unwrap_or_else(|e| e.into_inner()) {
        Some(chance) => Some(1.0 / chance.max(f32::MIN_POSITIVE)),
        None => decays.then_some(radius),
    }
}

/// A bed or an anchor about to blow: what it is, for the event to say.
pub(in crate::game) fn block_blows(game: &Game<'_>, pos: [i32; 3]) {
    let state = game.world.block(pos[0], pos[1], pos[2]);
    *EXPLODED.lock().unwrap_or_else(|e| e.into_inner()) = Some(state_text(game, state));
}

pub(in crate::game) fn exploded_block() -> Option<String> {
    EXPLODED.lock().unwrap_or_else(|e| e.into_inner()).take()
}

static RELOAD_BY_PLUGIN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `TabCompleteEvent`, on what the completers said: what's to go out, or `None` if it was cancelled.
pub(in crate::game) fn tab_matches(game: &mut Game<'_>, player: u128, name: &str, line: &str, matches: Vec<String>) -> Option<Vec<String>> {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_tab_complete") {
        return Some(matches);
    }
    *TAB_MATCHES.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let payload = json!({ "uuid": format!("{player:032x}"), "name": name, "line": line, "matches": matches });
    if on_refusable(game, "ew_tab_complete", payload) {
        return None;
    }
    Some(TAB_MATCHES.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or(matches))
}

static TAB_MATCHES: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// `SpawnChangeEvent`.
pub(in crate::game) fn spawn_changed(game: &mut Game<'_>, previous: Option<[i32; 3]>) {
    if super::paper_bridge::hears("ew_spawn_changed") {
        let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
        super::paper_bridge::on_trigger(game, "ew_spawn_changed", json!({ "uuid": null, "level": level, "previous": previous }));
    }
}

thread_local! {
    static CAN_BUILD: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// `VehicleCreateEvent`, with the vehicle in the level: false to take it out.
pub(in crate::game) fn vehicle_created(game: &mut Game<'_>, vehicle: u32) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_vehicle_created") {
        return true;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    !on_refusable(game, "ew_vehicle_created", json!({ "uuid": null, "level": level, "vehicle": vehicle }))
}

/// `BlockFertilizeEvent`, on what the bone meal wrote: false if it's all been put back.
pub(in crate::game) fn fertilized(game: &mut Game<'_>, pos: [i32; 3], writes: Vec<horizon_world::worldgen::structure::start::StructureWrite>) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if writes.is_empty() || !hears("ew_fertilize") {
        return true;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let rows: Vec<Value> = writes.iter().map(|(at, _, new)| json!([at[0], at[1], at[2], state_text(game, *new)])).collect();
    // A growth its own event stopped comes here stopped, for a listener to let through.
    let stopped = GROW_REFUSED.take();
    GROW_KEPT.set(None);
    let refused = on_refusable(game, "ew_fertilize", json!({ "level": level, "pos": pos, "blocks": rows, "cancelled": stopped }));
    let kept = GROW_KEPT.take();
    for (at, old, new) in writes.into_iter().rev() {
        if refused || kept.as_ref().is_some_and(|kept| !kept.contains(&at)) {
            let _ = game.level_set(at, old);
        } else if stopped {
            let _ = game.level_set(at, new);
        }
    }
    !refused
}

/// `EntitiesLoadEvent` or `EntitiesUnloadEvent`, for each of these chunks.
pub(in crate::game) fn entities_moved(game: &mut Game<'_>, event: &str, chunks: &[(i32, i32)]) {
    if chunks.is_empty() || !super::paper_bridge::hears(event) {
        return;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    for (x, z) in chunks {
        super::paper_bridge::on_trigger(game, event, json!({ "uuid": null, "level": level, "chunk": [x, z] }));
    }
}

static WEATHER_BY_PLUGIN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The sky has just changed from `before`: `WeatherChangeEvent` and `ThunderChangeEvent` with the old sky
/// still up, and what they refuse stays as it was.
pub(in crate::game) fn weather_moved(game: &mut Game<'_>, before: (bool, bool), cause: &str) {
    use crate::game::paper_bridge::{hears, on_refusable};
    let after = (game.world.weather.raining, game.world.weather.thundering);
    if after == before {
        return;
    }
    let by_plugin = cause == "COMMAND" && WEATHER_BY_PLUGIN.swap(false, std::sync::atomic::Ordering::Relaxed);
    let cause = if by_plugin { "PLUGIN" } else { cause };
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let mut result = after;
    (game.world.weather.raining, game.world.weather.thundering) = before;
    if after.1 != before.1 && hears("ew_thunder") && on_refusable(game, "ew_thunder", json!({ "uuid": null, "level": level, "thundering": after.1, "cause": cause })) {
        result.1 = before.1;
    }
    if after.0 != before.0 && hears("ew_weather") && on_refusable(game, "ew_weather", json!({ "uuid": null, "level": level, "raining": after.0, "cause": cause })) {
        result.0 = before.0;
    }
    (game.world.weather.raining, game.world.weather.thundering) = result;
}

/// Where each vehicle was at the end of the last tick.
static VEHICLES_AT: Mutex<Option<std::collections::HashMap<u32, [f64; 5]>>> = Mutex::new(None);

/// `VehicleUpdateEvent` for every vehicle that ticked, `VehicleMoveEvent` for those that moved.
pub(in crate::game) fn vehicles_ticked(game: &mut Game<'_>) {
    use crate::game::paper_bridge::{hears, on_trigger};
    if !hears("ew_vehicle_tick") {
        return;
    }
    let ticking = game.entity_ticking_area();
    let boats = game.boats.iter().map(|boat| (boat.id, [boat.position.x, boat.position.y, boat.position.z, f64::from(boat.yaw), 0.0]));
    let carts = game.carts.iter().map(|cart| (cart.id, [cart.position.x, cart.position.y, cart.position.z, f64::from(cart.yaw), 0.0]));
    let vehicles: Vec<(u32, [f64; 5])> = boats.chain(carts).filter(|(_, to)| ticking(to[0].floor() as i32, to[2].floor() as i32)).collect();
    drop(ticking);
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    for (id, to) in vehicles {
        let from = VEHICLES_AT.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(id, to).unwrap_or(to);
        on_trigger(game, "ew_vehicle_tick", json!({ "uuid": null, "level": level, "vehicle": id, "from": from, "to": to, "moved": from != to }));
    }
}

/// `WhitelistStateUpdateEvent` for one name: true keeps the list as it is.
pub(in crate::game) fn whitelist_refused(game: &mut Game<'_>, add: bool, name: &str, id: u128) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    hears("ew_whitelist_one") && on_refusable(game, "ew_whitelist_one", json!({ "uuid": null, "add": add, "name": name, "id": format!("{id:032x}") }))
}

/// `MapInitializeEvent`, with the map there to read.
pub(in crate::game) fn map_made(game: &mut Game<'_>, id: i32) {
    use crate::game::paper_bridge::{hears, on_trigger};
    if hears("ew_map_new") {
        on_trigger(game, "ew_map_new", json!({ "uuid": null, "map": id }));
    }
}

/// `BlockReceiveGameEvent` for what a sculk block just heard, cancelled if it wouldn't take it: true ignores it.
pub(in crate::game) fn vibration_refused(game: &mut Game<'_>, key: (i32, i32, i32), event: &str, source: Option<horizon_client::mobs::Who>, ignored: bool) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    use horizon_client::mobs::Who;
    if !hears("ew_heard") {
        return ignored;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let (uuid, entity) = match source {
        Some(Who::Mob(id)) => (None, Some(id)),
        Some(Who::Player) => (Some(format!("{:032x}", game.native_acting_id())), None),
        _ => (None, None),
    };
    on_refusable(game, "ew_heard", json!({ "uuid": uuid, "entity": entity, "level": level, "pos": [key.0, key.1, key.2], "heard": event, "ignored": ignored }))
}

static TARGET_SAID: Mutex<Option<i32>> = Mutex::new(None);

/// `TargetHitEvent`: what the block's to give, `None` if it's stopped.
pub(in crate::game) fn target_said(game: &mut Game<'_>, projectile: u32, pos: [i32; 3], face: [i32; 3], strength: i32) -> Option<i32> {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_target_hit") {
        return Some(strength);
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    *TARGET_SAID.lock().unwrap_or_else(|e| e.into_inner()) = None;
    if on_refusable(game, "ew_target_hit", json!({ "uuid": null, "level": level, "pos": pos, "face": face, "strength": strength, "entity": projectile })) {
        return None;
    }
    Some(TARGET_SAID.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or(strength))
}

static GRIND_XP: Mutex<Option<i32>> = Mutex::new(None);

/// `BlockExpEvent` for a grindstone's result: the experience that's given.
pub(in crate::game) fn grind_xp(game: &mut Game<'_>, pos: [i32; 3], xp: i32) -> i32 {
    use crate::game::paper_bridge::{hears, on_trigger};
    if !hears("ew_grindstone_xp") {
        return xp;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    *GRIND_XP.lock().unwrap_or_else(|e| e.into_inner()) = None;
    on_trigger(game, "ew_grindstone_xp", json!({ "uuid": null, "level": level, "pos": pos, "grindstone": xp }));
    GRIND_XP.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or(xp)
}

/// `BlockDispenseArmorEvent`, for the one who's to wear it.
pub(in crate::game) fn armor_refused(game: &mut Game<'_>, pos: (i32, i32, i32), stack: &ItemStack, entity: Option<i64>, player: bool) -> bool {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_dispense_armor") {
        return false;
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let uuid = player.then(|| format!("{:032x}", game.native_acting_id()));
    on_refusable(game, "ew_dispense_armor", json!({ "uuid": null, "level": level, "pos": [pos.0, pos.1, pos.2], "stack": crate::game::paper_query::stack_json(stack), "target": entity, "target_uuid": uuid }))
}

static BOLT_CAUSE: Mutex<Option<String>> = Mutex::new(None);

/// What the next bolt is said to come from.
pub(in crate::game) fn set_bolt_cause(cause: &str) {
    *BOLT_CAUSE.lock().unwrap_or_else(|e| e.into_inner()) = Some(cause.to_owned());
}

pub(in crate::game) fn bolt_cause() -> Option<String> {
    BOLT_CAUSE.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// What a command that didn't parse says: the failure, then the line up to where it failed.
pub(in crate::game) fn command_failed(message: String, input: Option<&str>, cursor: usize) -> String {
    let Some(input) = input else {
        return message;
    };
    let cursor = cursor.min(input.len());
    let start = (0..=cursor.saturating_sub(10)).rev().find(|&at| input.is_char_boundary(at)).unwrap_or(0);
    let dots = if cursor > 10 { "..." } else { "" };
    format!("{message}\n{dots}{}{}<--[HERE]", &input[start..cursor], &input[cursor..])
}

static DROPS_KEPT: Mutex<Option<Vec<u64>>> = Mutex::new(None);

/// `BlockDropItemEvent` for a creative player's break, which drops nothing and is told so.
pub(in crate::game) fn creative_drops(game: &mut Game<'_>, pos: [i32; 3], state: horizon_world::registry::BlockStateId) {
    let first = game.dropped.len();
    drops_fell(game, pos, state, first);
}

/// `BlockDropItemEvent`, with what a player's break let fall from `first` on: what it keeps out goes.
pub(in crate::game) fn drops_fell(game: &mut Game<'_>, pos: [i32; 3], state: horizon_world::registry::BlockStateId, first: usize) {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_drops_fell") {
        return;
    }
    let items: Vec<u32> = game.dropped.iter().skip(first).map(|item| item.id).collect();
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let uuid = format!("{:032x}", game.native_acting_id());
    *DROPS_KEPT.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let refused = on_refusable(game, "ew_drops_fell", json!({ "uuid": uuid, "level": level, "pos": pos, "state": state_text(game, state), "items": items }));
    if refused {
        let kept = DROPS_KEPT.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or_default();
        game.dropped.retain(|item| !items.contains(&item.id) || kept.contains(&u64::from(item.id)));
    }
}

/// `LootGenerateEvent` for a container's loot, which was rolled with its chunk and is opened now.
pub(in crate::game) fn chest_loot(game: &mut Game<'_>, key: (i32, i32, i32), table: &str) {
    use crate::game::paper_bridge::{hears, on_refusable};
    if !hears("ew_loot") {
        return;
    }
    let Some(entity) = game.world.block_entities.get(&key) else {
        return;
    };
    let items: Vec<Value> = entity.slots().iter().flatten().map(crate::game::paper_query::stack_json).collect();
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    let origin = [f64::from(key.0) + 0.5, f64::from(key.1) + 0.5, f64::from(key.2) + 0.5];
    let acting = if game.lan_acting.is_some() { json!(format!("{:032x}", game.native_acting_id())) } else { Value::Null };
    let payload = json!({ "uuid": acting, "level": level, "table": table, "origin": origin, "pos": [key.0, key.1, key.2], "container": true, "items": items });
    if !on_refusable(game, "ew_loot", payload) {
        return;
    }
    let mut kept = std::mem::take(&mut *DROPS.lock().unwrap_or_else(|e| e.into_inner())).into_iter();
    if let Some(entity) = game.world.block_entities.get_mut(&key) {
        for slot in entity.slots_mut() {
            *slot = kept.next();
        }
    }
    game.world.modified.insert((key.0 >> 4, key.2 >> 4));
}

/// The player a portal is about to be made for.
static PORTAL_TRAVELLER: Mutex<Option<String>> = Mutex::new(None);

pub(in crate::game) fn portal_traveller(player: Option<u128>) {
    *PORTAL_TRAVELLER.lock().unwrap_or_else(|e| e.into_inner()) = player.map(|player| format!("{player:032x}"));
}

/// `player`, if no one has been named: a guest's trip names its guest.
pub(in crate::game) fn portal_traveller_or(player: u128) {
    PORTAL_TRAVELLER.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(|| format!("{player:032x}"));
}

/// The bolts `LightningStrikeEvent`'s listeners are looking at, which are already in the level.
static BOLTS_AHEAD: Mutex<Vec<u32>> = Mutex::new(Vec::new());

pub(in crate::game) fn bolt_ahead(id: u32) {
    BOLTS_AHEAD.lock().unwrap_or_else(|e| e.into_inner()).push(id);
}

/// The last of them, done with.
pub(in crate::game) fn take_bolt_ahead() -> Option<u32> {
    BOLTS_AHEAD.lock().unwrap_or_else(|e| e.into_inner()).pop()
}

// ---- Structures being placed ----

/// The texts of the block states, the other way.
static STATE_IDS: std::sync::OnceLock<std::collections::HashMap<String, u32>> = std::sync::OnceLock::new();
/// Each block's default state.
static DEFAULT_STATES: std::sync::OnceLock<std::collections::HashMap<String, u32>> = std::sync::OnceLock::new();
static STRUCTURE_TOKENS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// What a transformer's thread asks of the thread placing the structure, which has the region.
enum RegionAsk {
    Read([i32; 3], std::sync::mpsc::Sender<Option<horizon_world::registry::BlockStateId>>),
    Write([i32; 3], horizon_world::registry::BlockStateId, std::sync::mpsc::Sender<bool>),
}

thread_local! {
    /// The structure this thread is placing, and the keys of its transformers once a listener set some.
    static STRUCTURE_ASKED: std::cell::RefCell<(u64, Option<Value>)> = const { std::cell::RefCell::new((0, None)) };
    /// On a thread running transformers: what's been placed, as they've left it.
    static STRUCTURE_BATCH: std::cell::RefCell<Option<Value>> = const { std::cell::RefCell::new(None) };
    static REGION_LINK: std::cell::RefCell<Option<std::sync::mpsc::Sender<RegionAsk>>> = const { std::cell::RefCell::new(None) };
}

fn state_parts(text: &str) -> (String, std::collections::BTreeMap<String, String>) {
    let (name, rest) = text.split_once('[').unwrap_or((text, ""));
    let properties = rest.trim_end_matches(']').split(',').filter_map(|pair| pair.split_once('=')).map(|(key, value)| (key.to_owned(), value.to_owned())).collect();
    (if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") }, properties)
}

fn state_key(name: &str, properties: &std::collections::BTreeMap<String, String>) -> String {
    let pairs: Vec<String> = properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
    format!("{name}[{}]", pairs.join(","))
}

/// The state a text names, with the block's defaults for what it leaves out.
fn state_of_text(text: &str) -> Option<horizon_world::registry::BlockStateId> {
    let (name, given) = state_parts(text);
    let default = *DEFAULT_STATES.get()?.get(&name)?;
    let (_, mut properties) = state_parts(STATES.get()?.get(default as usize)?);
    properties.extend(given);
    STATE_IDS.get()?.get(&state_key(&name, &properties)).map(|id| horizon_world::registry::BlockStateId(*id as _))
}

pub(in crate::game) fn state_name(id: horizon_world::registry::BlockStateId) -> String {
    STATES.get().and_then(|states| states.get(id.0 as usize)).cloned().unwrap_or_else(|| "minecraft:air".to_owned())
}

type Region<'a> = dyn horizon_world::worldgen::structure::start::StructureRegion + 'a;

/// Answers a listener's thread until it's done.
fn serve_region(region: &mut Region<'_>, questions: std::sync::mpsc::Receiver<RegionAsk>) {
    for question in questions {
        match question {
            RegionAsk::Read(at, reply) => {
                let _ = reply.send(region.read(at));
            }
            RegionAsk::Write(at, state, reply) => {
                let _ = reply.send(region.write(at, state));
            }
        }
    }
}

/// `AsyncStructureGenerateEvent`, from the thread placing the structure.
/// `AsyncStructureSpawnEvent`: whether the start stands.
fn structure_spawn_hook(level: &str, name: &str, min: [i32; 3], max: [i32; 3], chunk: [i32; 2]) -> bool {
    !super::paper_bridge::hears("ew_structure_spawn")
        || super::paper_bridge::fire_free("ew_structure_spawn", &json!({ "level": level, "structure": name, "min": min, "max": max, "chunk": chunk })) != 2
}

fn structure_hook(level: &str, name: &str, min: [i32; 3], max: [i32; 3], chunk: [i32; 2], cause: &str) -> u8 {
    if !super::paper_bridge::hears("ew_structure_generate") {
        return 1;
    }
    let token = STRUCTURE_TOKENS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    STRUCTURE_ASKED.set((token, None));
    let payload = json!({ "level": level, "structure": name, "min": min, "max": max, "chunk": chunk, "cause": cause, "token": token });
    let refused = super::paper_bridge::fire_free("ew_structure_generate", &payload) == 2;
    let transformed = STRUCTURE_ASKED.with_borrow(|asked| asked.1.is_some());
    if refused {
        if transformed {
            // They're held until the structure is placed, and it won't be.
            super::paper_bridge::fire_free("ew_structure_blocks", &json!({ "placed": token, "cancelled": true }));
        }
        return 0;
    }
    if transformed { 2 } else { 1 }
}

/// A structure's transformers, on a thread of their own: this one keeps the region, and answers for it.
fn structure_writes(
    region: &mut dyn horizon_world::worldgen::structure::start::StructureRegion,
    writes: Vec<horizon_world::worldgen::structure::start::StructureWrite>,
    entities: Vec<horizon_world::nbt::Compound>,
) -> horizon_world::worldgen::structure::start::StructureChanges {
    use horizon_world::worldgen::structure::start::EntityVerdict;
    let (token, keys) = STRUCTURE_ASKED.take();
    let keys = keys.unwrap_or(Value::Null);
    let places: Vec<[i32; 3]> = writes.iter().map(|write| write.0).collect();
    let rows: Vec<Value> = writes.iter().map(|(at, old, new)| json!([at[0], at[1], at[2], state_name(*old), state_name(*new)])).collect();
    let brought: Vec<Value> = entities
        .iter()
        .map(|entity| {
            let data = crate::game::commands::paper_entities::tag_json(&Tag::Compound(entity.clone()));
            let at = [0, 1, 2].map(|side| data["Pos"][side].as_f64().unwrap_or(0.0).floor() as i32);
            json!({ "pos": at, "data": data, "kept": true })
        })
        .collect();
    let level = horizon_world::worldgen::structure::start::STRUCTURE_LEVEL.with_borrow(Clone::clone);
    let batch = json!({ "level": level, "blocks": rows, "entities": brought, "block_keys": keys["blocks"], "entity_keys": keys["entities"] });
    let (sender, questions) = std::sync::mpsc::channel();
    let answer = std::thread::scope(|scope| {
        let runner = scope.spawn(move || {
            REGION_LINK.set(Some(sender));
            STRUCTURE_BATCH.set(Some(batch));
            super::paper_bridge::fire_free("ew_structure_blocks", &json!({ "placed": token }));
            REGION_LINK.set(None);
            STRUCTURE_BATCH.take()
        });
        serve_region(&mut *region, questions);
        runner.join().ok().flatten()
    });
    let Some(answer) = answer else {
        return (Vec::new(), Vec::new());
    };
    let mut changes = Vec::new();
    for (row, place) in answer["blocks"].as_array().into_iter().flatten().zip(places.iter().zip(&writes)) {
        let (at, write) = place;
        if let Some(state) = row[4].as_str().and_then(state_of_text)
            && state != write.2
        {
            changes.push((*at, state));
        }
    }
    let verdicts = answer["entities"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| {
            if row["kept"] != true {
                return EntityVerdict::Drop;
            }
            match row["snbt"].as_str().map(horizon_world::nbt::parse_snbt) {
                Some(Ok((Tag::Compound(changed), _))) => EntityVerdict::Replace(changed),
                _ => EntityVerdict::Keep,
            }
        })
        .collect();
    (changes, verdicts)
}

/// A transformer's `LimitedRegion`: the blocks around the structure, which the placing thread has.
fn region_query(request: &str) -> Option<String> {
    let link = REGION_LINK.with_borrow(Clone::clone)?;
    let query: Value = serde_json::from_str(request).ok()?;
    let at = [query["pos"][0].as_i64()? as i32, query["pos"][1].as_i64()? as i32, query["pos"][2].as_i64()? as i32];
    match query["op"].as_str()? {
        "block" => {
            let (reply, answer) = std::sync::mpsc::channel();
            link.send(RegionAsk::Read(at, reply)).ok()?;
            let (name, properties) = state_parts(&state_name(answer.recv().ok()??));
            Some(json!({ "name": name, "properties": properties }).to_string())
        }
        "set_block" => {
            let state = state_of_text(query["block"].as_str()?)?;
            let (reply, answer) = std::sync::mpsc::channel();
            link.send(RegionAsk::Write(at, state, reply)).ok()?;
            Some(json!(answer.recv().ok()?).to_string())
        }
        _ => None,
    }
}

/// The game's own data files, with or without a game.
fn data_query(query: &Value) -> Option<String> {
    let root = DATA_ROOT.lock().unwrap_or_else(|e| e.into_inner()).clone();
    match query["op"].as_str()? {
        "ev_world_free_locate_now" => Some(LOCATE_NOW.with_borrow(Clone::clone).unwrap_or(Value::Null).to_string()),
        "ev_world_free_locate_set" => {
            LOCATE_NOW.set(Some(query.clone()));
            Some("true".to_owned())
        }
        "ev_world_free_note" => {
            NOTE_NOW.set(Some((query["note"].as_u64()?, query["instrument"].as_str()?.to_owned())));
            Some("true".to_owned())
        }
        "ev_world_free_grow_kept" => {
            GROW_KEPT.set(Some(query["blocks"].as_array()?.iter().filter_map(position).collect()));
            Some("true".to_owned())
        }
        "ev_world_free_cauldron" => {
            CAULDRON_NOW.set(Some(query["state"].as_str()?.to_owned()));
            Some("true".to_owned())
        }
        "ev_world_free_profile_fire" => profile_fire(query),
        "ev_world_free_profile_set" => {
            PROFILE_SAID.set(Some(query["said"].clone()));
            Some("true".to_owned())
        }
        "ev_world_free_reload_plugin" => {
            RELOAD_BY_PLUGIN.store(true, std::sync::atomic::Ordering::Relaxed);
            Some("true".to_owned())
        }
        "ev_world_free_reload_was_plugin" => Some(RELOAD_BY_PLUGIN.swap(false, std::sync::atomic::Ordering::Relaxed).to_string()),
        "ev_world_free_can_build" => {
            CAN_BUILD.set(query["buildable"].as_bool());
            Some("true".to_owned())
        }
        "ev_world_free_ping_now" => Some(json!(PING_NOW.with_borrow(Clone::clone)).to_string()),
        "ev_world_free_ping_set" => {
            PING_NOW.set(query["json"].as_str()?.to_owned());
            Some("true".to_owned())
        }
        "ev_world_free_ping_client" => Some(super::paper_fam_paper::ping_client_json(query["id"].as_u64()?).to_string()),
        "ev_world_free_query_now" => Some(QUERY_NOW.with_borrow(Clone::clone).unwrap_or(Value::Null).to_string()),
        "ev_world_free_query_set" => {
            QUERY_NOW.set(Some(query["answer"].clone()));
            Some("true".to_owned())
        }
        "ev_world_free_weather_plugin" => {
            WEATHER_BY_PLUGIN.store(true, std::sync::atomic::Ordering::Relaxed);
            Some("true".to_owned())
        }
        "ev_world_free_life_passed" => {
            let moment = query["moment"].as_str()?;
            Some(json!(LIFE_PASSED.lock().unwrap_or_else(|e| e.into_inner()).contains(&moment)).to_string())
        }
        "ev_world_free_data_file" => {
            let path = query["path"].as_str()?;
            if path.contains("..") {
                return Some("null".to_owned());
            }
            let file = root?.join(format!("data/minecraft/{path}.json"));
            Some(std::fs::read_to_string(file).ok().filter(|text| serde_json::from_str::<Value>(text).is_ok()).unwrap_or_else(|| "null".to_owned()))
        }
        "ev_world_free_data_list" => {
            let path = query["path"].as_str()?;
            if path.contains("..") {
                return Some("[]".to_owned());
            }
            let folder = root?.join(format!("data/minecraft/{path}"));
            let names: Vec<String> = std::fs::read_dir(folder).into_iter().flatten().flatten().filter_map(|entry| entry.file_name().to_string_lossy().strip_suffix(".json").map(str::to_owned)).collect();
            Some(json!(names).to_string())
        }
        _ => None,
    }
}

/// What a listener of `AsyncStructureGenerateEvent` left on a structure, and what its transformers made of it.
fn structure_query(query: &Value) -> Option<String> {
    match query["op"].as_str()? {
        "ev_world_free_transformers" => {
            STRUCTURE_ASKED.with_borrow_mut(|asked| asked.1 = Some(json!({ "blocks": query["blocks"], "entities": query["entities"] })));
            Some("true".to_owned())
        }
        "ev_world_free_transformer_keys" => Some(STRUCTURE_ASKED.with_borrow(|asked| asked.1.clone()).unwrap_or(Value::Null).to_string()),
        "ev_world_free_structure" => Some(STRUCTURE_BATCH.with_borrow(Clone::clone).unwrap_or(Value::Null).to_string()),
        "ev_world_free_structure_set" => {
            STRUCTURE_BATCH.with_borrow_mut(|batch| {
                let Some(batch) = batch else {
                    return;
                };
                for change in query["blocks"].as_array().into_iter().flatten() {
                    batch["blocks"][change[0].as_u64().unwrap_or(0) as usize][4] = change[1].clone();
                }
                for change in query["entities"].as_array().into_iter().flatten() {
                    let row = &mut batch["entities"][change[0].as_u64().unwrap_or(0) as usize];
                    row["kept"] = change[1].clone();
                    row["snbt"] = change[2].clone();
                }
            });
            Some("true".to_owned())
        }
        _ => None,
    }
}

/// Puts the moments of the level and world generation in place, for the listeners asking.
fn install(game: &Game<'_>) {
    let registry = &game.pack.assets.registry;
    STATES.get_or_init(|| (0..registry.len()).map(|id| registry.state(horizon_world::registry::BlockStateId(id as _)).map_or_else(String::new, |state| {
        if state.properties.is_empty() {
            return state.name.to_string();
        }
        let properties: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
        format!("{}[{}]", state.name, properties.join(","))
    })).collect());
    let _ = horizon_world::level::ASK_HOOK.set(ask_hook);
    let states = STATES.get().map_or(&[][..], Vec::as_slice);
    STATE_IDS.get_or_init(|| states.iter().enumerate().map(|(id, text)| {
        let (name, properties) = state_parts(text);
        (state_key(&name, &properties), id as u32)
    }).collect());
    DEFAULT_STATES.get_or_init(|| {
        let mut defaults = std::collections::HashMap::new();
        for text in states {
            let (name, _) = state_parts(text);
            if let Some(default) = registry.default_state(&name) {
                defaults.insert(name, default.0 as u32);
            }
        }
        defaults
    });
    let _ = horizon_world::worldgen::structure::start::STRUCTURE_HOOK.set(structure_hook);
    let _ = horizon_world::worldgen::structure::start::STRUCTURE_SPAWN_HOOK.set(structure_spawn_hook);
    let _ = horizon_world::worldgen::structure::start::STRUCTURE_WRITES_HOOK.set(structure_writes);
}

/// The questions waiting of one kind, with what last tick's listeners said sent back first.
fn asks(game: &Game<'_>, kind: &str) -> Value {
    use horizon_client::paper_hub as hub;
    let kind: &'static str = match kind {
        "whitelist" => "whitelist",
        "query" => "query",
        _ => "structure",
    };
    {
        let mut polled = hub::ASKS_POLLED.lock().unwrap_or_else(|e| e.into_inner());
        polled.retain(|(known, _)| *known != kind);
        polled.push((kind, std::time::Instant::now()));
    }
    let tick = game.ticks as i64 + 1;
    let mut asks = hub::ASKS.lock().unwrap_or_else(|e| e.into_inner());
    asks.retain(|ask| {
        if ask.kind != kind || ask.seen == 0 || ask.seen == tick {
            return true;
        }
        let _ = ask.answer.send(ask.json.clone());
        false
    });
    Value::Array(
        asks.iter_mut()
            .filter(|ask| ask.kind == kind)
            .map(|ask| {
                ask.seen = tick;
                json!({ "id": ask.id, "json": ask.json })
            })
            .collect(),
    )
}

/// What a listener may say with no game lent, as a level's moment fires.
pub(in crate::game) fn free_query(request: &str) -> Option<String> {
    if !request.contains("ev_world_free_") {
        return region_query(request);
    }
    let query: Value = serde_json::from_str(request).ok()?;
    if let Some(answer) = structure_query(&query) {
        return Some(answer);
    }
    if let Some(answer) = data_query(&query) {
        return Some(answer);
    }
    match query["op"].as_str()? {
        "ev_world_free_effect" => *EFFECT.lock().unwrap_or_else(|e| e.into_inner()) = query["data"].as_str().map(str::to_owned),
        "ev_world_free_drops" => *FREE_DROPS.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["items"].as_array()?.iter().filter_map(stack_from).collect()),
        "ev_world_free_sponge" => *FREE_KEPT.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["blocks"].as_array()?.iter().filter_map(position).collect()),
        _ => return None,
    }
    Some("true".to_owned())
}

/// What a listener left of the suggestions going to a player.
static SUGGESTIONS_LEFT: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// `AsyncPlayerSendSuggestionsEvent`. `None` leaves them as they are; an empty list sends nothing.
pub(in crate::game) fn sent_suggestions(game: &mut Game<'_>, player: u128, buffer: &str, words: Vec<String>) -> Option<Vec<String>> {
    if !crate::game::paper_bridge::hears("ew_suggestions") {
        return None;
    }
    crate::game::paper_bridge::on_trigger(game, "ew_suggestions", json!({ "uuid": format!("{player:032x}"), "buffer": buffer, "suggestions": words }));
    SUGGESTIONS_LEFT.lock().unwrap_or_else(|e| e.into_inner()).take()
}

static SLOT: Mutex<usize> = Mutex::new(0);
static EMPTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(in crate::game) fn note_slot(slot: usize) {
    *SLOT.lock().unwrap_or_else(|e| e.into_inner()) = slot;
}

pub(in crate::game) fn dispense_slot() -> usize {
    *SLOT.lock().unwrap_or_else(|e| e.into_inner())
}

pub(in crate::game) fn note_empty() {
    EMPTY.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub(in crate::game) fn dispenser_empty() -> bool {
    EMPTY.swap(false, std::sync::atomic::Ordering::Relaxed)
}

/// The console line a listener put in place of the one it refused.
static CONSOLE_LINE: Mutex<Option<String>> = Mutex::new(None);

pub(in crate::game) fn console_line() -> Option<String> {
    CONSOLE_LINE.lock().unwrap_or_else(|e| e.into_inner()).take()
}

static SLEEP_SKIP: Mutex<Option<i64>> = Mutex::new(None);

/// `TimeSkipEvent`, for the night a bed skips: how far time goes, `None` if a listener keeps it still.
pub(in crate::game) fn sleep_skip(game: &mut Game<'_>, skip: i64) -> Option<i64> {
    if !crate::game::paper_bridge::hears("ew_time") {
        return Some(skip);
    }
    let level = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    if crate::game::paper_bridge::on_refusable(game, "ew_time", json!({ "uuid": Value::Null, "level": level, "id": "time.sleep", "time": skip })) {
        return None;
    }
    Some(SLEEP_SKIP.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or(skip))
}

/// `None` for an op that isn't this family's.
pub(in crate::game) fn query(game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    match op {
        "ev_world_vehicle_hit" => Some(json!(vehicle_hit(game, query))),
        "ev_world_riding" => {
            let wanted = u128::from_str_radix(query["uuid"].as_str()?, 16).ok()?;
            let guest = game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == wanted)?;
            Some(json!(guest.vehicle))
        }
        "ev_world_portal_shape" => {
            let list = query["pos"].as_array()?;
            let pos = [list.first()?.as_i64()? as i32, list.get(1)?.as_i64()? as i32, list.get(2)?.as_i64()? as i32];
            if !matches!(game.dim, crate::game::Dim::Overworld | crate::game::Dim::Nether) {
                return Some(Value::Null);
            }
            for x_axis in [true, false] {
                if let Some((origin, width, height)) = game.portal_shape(pos, x_axis) {
                    let step = if x_axis { [1, 0] } else { [0, 1] };
                    let blocks: Vec<[i32; 3]> = (0..height).flat_map(|y| (0..width).map(move |w| [origin[0] + step[0] * w, origin[1] + y, origin[2] + step[1] * w])).collect();
                    return Some(json!({ "axis": if x_axis { "x" } else { "z" }, "blocks": blocks }));
                }
            }
            Some(Value::Null)
        }
        "ev_world_break_exp" => break_exp(game, query),
        "ev_world_xp_next" => xp_next(game, query),
        "ev_world_vault_shows" => {
            *VAULT_SHOWS.lock().unwrap_or_else(|e| e.into_inner()) = Some(stack_from(&query["item"]));
            Some(json!(true))
        }
        "ev_world_beacon_gone" => {
            let at = position(&query["pos"])?;
            let level = format!("minecraft:{}", query["level"].as_str()?.trim_start_matches("minecraft:"));
            let mut all = BEACONS_LIT.lock().unwrap_or_else(|e| e.into_inner());
            let row = all.iter().position(|(known, key)| *known == level && *key == (at[0], at[1], at[2]));
            Some(json!(row.map(|row| all.swap_remove(row)).is_some()))
        }
        "ev_world_set_drops" => {
            LOOT_LISTED.store(true, std::sync::atomic::Ordering::Relaxed);
            *DROPS.lock().unwrap_or_else(|e| e.into_inner()) = query["items"].as_array()?.iter().filter_map(stack_from).collect();
            Some(json!(true))
        }
        "ev_world_keep_blocks" => {
            *BLAST_BLOCKS.lock().unwrap_or_else(|e| e.into_inner()) = query["blocks"].as_array()?.iter().filter_map(position).collect();
            Some(json!(true))
        }
        "ev_world_raiders" => Some(raiders(game, query)),
        "ev_world_beacon_effect" => {
            let (duration, amplifier) = (query["duration"].as_i64()? as i32, query["amplifier"].as_i64()? as i32);
            let effect = horizon_client_render::effects::EffectInstance::new(effect_name(query["id"].as_str()?), duration, amplifier);
            game.potions.effects.add(horizon_client_render::effects::EffectInstance { ambient: true, ..effect });
            Some(json!(true))
        }
        "ev_world_no_hardness" => {
            let pos = position(&query["pos"])?;
            Some(json!(game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).hardness == 0.0))
        }
        "ev_world_insta_break" => {
            let pos = position(&query["pos"])?;
            Some(json!(game.break_block(pos).is_ok()))
        }
        "ev_world_compost" => {
            let pos = position(&query["pos"])?;
            let levels = query["levels"].as_i64()? as i32;
            let at = horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]);
            game.with_level(|b, level, ticks| b.composter_insert(level, ticks, at, levels));
            game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
            let _ = game.process_level_events();
            Some(json!(true))
        }
        "ev_world_no_egg" => {
            game.fight.previously_killed = true;
            Some(json!(true))
        }
        "ev_world_brewing" => {
            let pos = position(&query["pos"])?;
            let (brewing, recipe) = (query["brewing"].as_i64()? as i32, query["recipe"].as_i64()? as i32);
            let done = game.native_in_level(query["dimension"].as_str()?, |game| match game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2])) {
                Some(horizon_client::containers::BlockEntity::Brewing(stand)) => {
                    stand.brew_time = brewing;
                    stand.total_brew_time = recipe;
                    true
                }
                _ => false,
            });
            Some(json!(done == Some(true)))
        }
        "ev_world_result" => {
            *RESULT.lock().unwrap_or_else(|e| e.into_inner()) = stack_from(&query["stack"]);
            Some(json!(true))
        }
        "ev_world_campfire" => campfire_place(game, query),
        "ev_world_grind_xp" => {
            *GRIND_XP.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["exp"].as_i64()? as i32);
            Some(json!(true))
        }
        "ev_world_drops_kept" => {
            *DROPS_KEPT.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["items"].as_array()?.iter().filter_map(Value::as_u64).collect());
            Some(json!(true))
        }
        "ev_world_vehicle_worn" => {
            let id = query["vehicle"].as_u64()? as u32;
            if let Some(boat) = game.boats.iter_mut().find(|boat| boat.id == id) {
                boat.damage = 40.0;
            } else if let Some(cart) = game.carts.iter_mut().find(|cart| cart.id == id) {
                cart.damage = 40.0;
            }
            Some(json!(true))
        }
        "ev_world_bolt_cause" => {
            set_bolt_cause(query["cause"].as_str()?);
            Some(json!(true))
        }
        "ev_world_lockable" => {
            let pos = position(&query["pos"])?;
            let state = game.world.block(pos[0], pos[1], pos[2]);
            let name = game.pack.assets.registry.state(state)?.name.trim_start_matches("minecraft:").to_owned();
            let chest = name.ends_with("chest") && name != "ender_chest";
            Some(json!(chest || name.ends_with("shulker_box") || name.ends_with("furnace") || matches!(name.as_str(), "barrel" | "beacon" | "brewing_stand" | "crafter" | "dispenser" | "dropper" | "hopper" | "smoker")))
        }
        "ev_world_broadcast" => {
            let message = query["message"].as_str()?.to_owned();
            Some(json!(!crate::game::paper_bridge::on_refusable(game, "ew_broadcast", json!({ "uuid": null, "message": message }))))
        }
        "ev_world_target_power" => {
            *TARGET_SAID.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["strength"].as_i64()? as i32);
            Some(json!(true))
        }
        "ev_world_loot_preview" => loot_preview(game, query),
        "ev_world_loot_next" => {
            let items = query["items"].as_array()?.iter().filter_map(stack_from).collect();
            *LOOT_NEXT.lock().unwrap_or_else(|e| e.into_inner()) = Some((query["table"].as_str()?.to_owned(), items));
            Some(json!(true))
        }
        "ev_world_velocity" => {
            let at = |i: usize| query["velocity"][i].as_f64();
            *VELOCITY.lock().unwrap_or_else(|e| e.into_inner()) = Some([at(0)?, at(1)?, at(2)?]);
            Some(json!(true))
        }
        "ev_world_charge" => {
            *CHARGE.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["charge"].as_u64()? as u32);
            Some(json!(true))
        }
        "ev_world_radius" => {
            *RADIUS.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["radius"].as_i64()? as i32);
            Some(json!(true))
        }
        "ev_world_furnace_result" => {
            *FURNACE_RESULT.lock().unwrap_or_else(|e| e.into_inner()) = stack_from(&query["item"]);
            Some(json!(true))
        }
        "ev_world_blast_yield" => {
            *BLAST_YIELD.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["chance"].as_f64()? as f32);
            Some(json!(true))
        }
        "ev_world_tab_matches" => {
            *TAB_MATCHES.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["matches"].as_array()?.iter().filter_map(Value::as_str).map(str::to_owned).collect());
            Some(json!(true))
        }
        "ev_world_lock" => {
            let at = position(&query["pos"])?;
            *LOCK_SAID.lock().unwrap_or_else(|e| e.into_inner()) = Some(((at[0], at[1], at[2]), query["allow"] == true, stack_from(&query["key"])));
            Some(json!(true))
        }
        "ev_world_bell" => {
            *BELL.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["raiders"].as_array()?.iter().filter_map(Value::as_u64).collect());
            Some(json!(true))
        }
        "ev_world_tooltips" => {
            *TOOLTIPS.lock().unwrap_or_else(|e| e.into_inner()) = query["tooltips"].as_array()?.iter().filter_map(|row| Some((row[0].as_str()?.to_owned(), row[1].clone()))).collect();
            Some(json!(true))
        }
        "ev_world_suggest_none" => {
            SUGGEST_NONE.store(true, std::sync::atomic::Ordering::Relaxed);
            Some(json!(true))
        }
        "ev_world_asks" => Some(asks(game, query["kind"].as_str()?)),
        "ev_world_ask_set" => {
            let id = query["id"].as_u64()? as u32;
            let mut asks = horizon_client::paper_hub::ASKS.lock().unwrap_or_else(|e| e.into_inner());
            query["json"].as_str()?.clone_into(&mut asks.iter_mut().find(|ask| ask.id == id)?.json);
            Some(json!(true))
        }
        "ev_world_install" => {
            install(game);
            Some(json!(true))
        }
        "ev_world_vehicle_hits" => {
            VEHICLE_WATCH.store(true, std::sync::atomic::Ordering::Relaxed);
            Some(Value::Array(std::mem::take(&mut *VEHICLE_HITS.lock().unwrap_or_else(|e| e.into_inner()))))
        }
        "ev_world_chunk_dirty" => Some(json!(game.world.modified.contains(&(query["chunk"][0].as_i64()? as i32, query["chunk"][1].as_i64()? as i32)))),
        "ev_world_chunk_unsaved" => {
            UNSAVED.lock().unwrap_or_else(|e| e.into_inner()).push((query["chunk"][0].as_i64()? as i32, query["chunk"][1].as_i64()? as i32));
            Some(json!(true))
        }
        "ev_world_suggestions" => {
            *SUGGESTIONS_LEFT.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["left"].as_array()?.iter().filter_map(|word| word.as_str().map(str::to_owned)).collect());
            Some(json!(true))
        }
        "ev_world_console_line" => {
            *CONSOLE_LINE.lock().unwrap_or_else(|e| e.into_inner()) = query["line"].as_str().map(str::to_owned);
            Some(json!(true))
        }
        "ev_world_sleep_skip" => {
            *SLEEP_SKIP.lock().unwrap_or_else(|e| e.into_inner()) = query["skip"].as_i64();
            Some(json!(true))
        }
        "ev_world_is_block" => Some(json!(game.pack.assets.registry.default_state(query["name"].as_str()?).is_some())),
        _ => None,
    }
}
