// What a plugin does to one player: the packets only they get, and the state no command reaches.
use crate::game::Game;
use horizon_client::{paper_hub as hub, lan::data::component_nbt};
use horizon_protocol::packets::{configuration, play, play::Clientbound as C};
use horizon_world::attribute_map::ValueKind;
use horizon_world::nbt::Tag;
use serde_json::{Value, json};

fn uuid(request: &Value) -> Option<u128> {
    u128::from_str_radix(request["uuid"].as_str()?, 16).ok()
}

pub(in crate::game) fn text(value: &Value) -> Tag {
    component_nbt(&horizon_client_render::packs::component(value).component())
}

fn pos(value: &Value) -> Option<[i32; 3]> {
    let at = |i: usize| value.get(i)?.as_i64().map(|v| v as i32);
    Some([at(0)?, at(1)?, at(2)?])
}

/// One player's line of a list update, with only `fill`'s field in it.
fn info(uuid: u128, fill: impl FnOnce(&mut play::PlayerInfo)) -> play::PlayerInfo {
    let mut entry = play::PlayerInfo { uuid, name: None, properties: Vec::new(), chat_session: None, game_mode: None, listed: None, latency: None, display_name: None, list_order: None, show_hat: None };
    fill(&mut entry);
    entry
}

/// The packet a request names.
fn packet(game: &Game<'_>, request: &Value) -> Option<C> {
    let int = |key: &str| request[key].as_i64().unwrap_or(0);
    let float = |key: &str| request[key].as_f64().unwrap_or(0.0) as f32;
    let string = |key: &str| request[key].as_str().unwrap_or_default().to_owned();
    Some(match request["packet"].as_str()? {
        "tab_list" => C::TabList(play::TabList { header: text(&request["header"]), footer: text(&request["footer"]) }),
        "stop_sound" => C::StopSound(play::StopSound { source: request["source"].as_i64().map(|source| source as i32), sound: request["sound"].as_str().map(str::to_owned) }),
        "block_destruction" => C::BlockDestruction(play::BlockDestruction { id: int("id") as i32, pos: pos(&request["pos"])?, progress: int("progress") as u8 }),
        "block_update" => C::BlockUpdate(play::BlockUpdate { pos: pos(&request["pos"])?, state: super::render_control::parse_state(game.pack.assets.registry, request["block"].as_str()?).ok()?.0 as i32 }),
        "game_event" => C::GameEvent(play::GameEvent { event: int("event") as u8, param: float("param") }),
        "level_event" => C::LevelEvent(play::LevelEvent { kind: int("kind") as i32, pos: pos(&request["pos"])?, data: int("data") as i32, global: false }),
        "transfer" => C::Transfer(configuration::Transfer { host: string("host"), port: int("port") as i32 }),
        "store_cookie" => C::StoreCookie(configuration::StoreCookie { key: string("key"), payload: request["data"].as_array()?.iter().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect() }),
        "resource_pack_push" => C::ResourcePackPush(configuration::ResourcePackPush {
            id: u128::from_str_radix(request["id"].as_str()?, 16).ok()?,
            url: string("url"),
            hash: string("hash"),
            required: request["required"].as_bool().unwrap_or(false),
            prompt: request.get("prompt").filter(|prompt| !prompt.is_null()).map(text),
        }),
        "resource_pack_pop" => C::ResourcePackPop(configuration::ResourcePackPop { id: request["id"].as_str().and_then(|id| u128::from_str_radix(id, 16).ok()) }),
        "remove_entities" => C::RemoveEntities(play::RemoveEntities { ids: request["ids"].as_array()?.iter().filter_map(|id| id.as_i64().map(|id| id as i32)).collect() }),
        "sb_objective" => C::SetObjective(play::SetObjective { name: string("name"), method: int("method") as i8, display: (int("method") != 1).then(|| (text(&request["display"]), int("render") as i32, None)) }),
        "sb_display" => C::SetDisplayObjective(play::SetDisplayObjective { slot: int("slot") as i32, objective: string("objective") }),
        "sb_score" => C::SetScore(play::SetScore { owner: string("owner"), objective: string("objective"), score: int("score") as i32, display: None, number_format: None }),
        "sb_reset" => C::ResetScore(play::ResetScore { owner: string("owner"), objective: None }),
        "entity_event" => C::EntityEvent(play::EntityEvent { id: int("id") as i32, event: int("event") as i8 }),
        "hurt_animation" => C::HurtAnimation(play::HurtAnimation { id: int("id") as i32, yaw: float("yaw") }),
        "open_sign" => C::OpenSignEditor(play::OpenSignEditor { pos: pos(&request["pos"])?, front: request["front"].as_bool().unwrap_or(true) }),
        "cooldown" => C::Cooldown(play::Cooldown { group: string("group"), duration: int("ticks") as i32 }),
        "chat_completions" => C::CustomChatCompletions(play::CustomChatCompletions { action: int("action") as i32, entries: request["entries"].as_array()?.iter().filter_map(|entry| entry.as_str().map(str::to_owned)).collect() }),
        "experience" => C::SetExperience(play::SetExperience { progress: float("progress"), level: int("level") as i32, total: int("total") as i32 }),
        "health" => C::SetHealth(play::SetHealth { health: float("health"), food: int("food") as i32, saturation: float("saturation") }),
        "abilities" => C::PlayerAbilities(play::PlayerAbilities { flags: int("flags") as i8, flying_speed: float("fly"), walking_speed: float("walk") }),
        "spawn_position" => C::SetDefaultSpawnPosition(play::SetDefaultSpawnPosition { pos: play::GlobalPos { dimension: string("dimension"), pos: pos(&request["pos"])? }, yaw: 0.0, pitch: 0.0 }),
        "list_name" => C::PlayerInfoUpdate(play::PlayerInfoUpdate { actions: 32, entries: vec![info(u128::from_str_radix(request["target"].as_str()?, 16).ok()?, |entry| entry.display_name = Some(request.get("name").filter(|name| !name.is_null()).map(text)))] }),
        "listed" => C::PlayerInfoUpdate(play::PlayerInfoUpdate { actions: 8, entries: vec![info(u128::from_str_radix(request["target"].as_str()?, 16).ok()?, |entry| entry.listed = request["listed"].as_bool())] }),
        "list_order" => C::PlayerInfoUpdate(play::PlayerInfoUpdate { actions: 64, entries: vec![info(u128::from_str_radix(request["target"].as_str()?, 16).ok()?, |entry| entry.list_order = Some(int("order") as i32))] }),
        "player_time" => C::SetTime(play::SetTime { game_time: request["game_time"].as_i64()?, clocks: vec![play::ClockState { clock: 0, total_ticks: request["time"].as_i64()?, partial_tick: 0.0, rate: if request["relative"].as_bool().unwrap_or(true) { 1.0 } else { 0.0 } }] }),
        "held_slot" => C::SetHeldSlot(play::SetHeldSlot { slot: int("slot") as i32 }),
        "rotation" => C::PlayerRotation(play::PlayerRotation { yaw: float("yaw"), relative_yaw: false, pitch: float("pitch"), relative_pitch: false }),
        _ => return None,
    })
}

/// `{ "op": "player_send", "uuid": ..., "packet": ..., ... }`.
pub(in crate::game) fn send(game: &mut Game<'_>, request: &Value) -> bool {
    let (Some(player), Some(packet)) = (uuid(request), packet(game, request)) else {
        return false;
    };
    game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| guest.uuid == player)).map(|guest| guest.send(&packet)).is_some()
}

/// Flight: `allow_flight`, `flying`, `fly_speed` and `walk_speed`, each sent to the client at once.
fn set_flight(game: &mut Game<'_>, player: u128, field: &str, value: &Value) -> bool {
    let Some(host) = game.lan.as_mut() else {
        return false;
    };
    let Some(guest) = host.hub.guests.iter_mut().find(|guest| guest.uuid == player) else {
        return false;
    };
    let flight = {
        let mut all = hub::FLIGHT.lock().unwrap_or_else(|e| e.into_inner());
        let flight = all.entry(player).or_insert(hub::Flight { allowed: guest.creative() || guest.spectator(), fly_speed: 0.05, walk_speed: 0.1, wanted: None });
        match field {
            "allow_flight" => flight.allowed = value.as_bool().unwrap_or(false),
            "fly_speed" => flight.fly_speed = value.as_f64().unwrap_or(0.05) as f32,
            "walk_speed" => flight.walk_speed = value.as_f64().unwrap_or(0.1) as f32,
            _ => {}
        }
        *flight
    };
    match field {
        "flying" => guest.flying = value.as_bool().unwrap_or(false) && flight.allowed,
        "allow_flight" if !flight.allowed => guest.flying = false,
        _ => {}
    }
    if let Some(motion) = guest.motion.as_mut() {
        motion.may_fly = flight.allowed;
        motion.flying = guest.flying;
    }
    guest.send(&C::PlayerAbilities(play::PlayerAbilities { flags: guest.ability_flags(), flying_speed: flight.fly_speed, walking_speed: flight.walk_speed }));
    true
}

/// Each tick, for those a plugin lets fly: the game sets this from the game mode.
pub(in crate::game) fn keep_flight(game: &mut Game<'_>) {
    let all = hub::FLIGHT.lock().unwrap_or_else(|e| e.into_inner());
    if all.is_empty() {
        return;
    }
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    for guest in &mut host.hub.guests {
        if let (Some(flight), Some(motion)) = (all.get(&guest.uuid), guest.motion.as_mut()) {
            motion.may_fly = flight.allowed;
            motion.flying = guest.flying;
        }
    }
}

/// What the `player` query says of flight.
fn flight_of(guest: &horizon_client::lan::guest::Guest) -> (bool, f32, f32) {
    hub::FLIGHT.lock().unwrap_or_else(|e| e.into_inner()).get(&guest.uuid).map_or((guest.creative() || guest.spectator(), 0.05, 0.1), |flight| (flight.allowed, flight.fly_speed, flight.walk_speed))
}

/// `{ "op": "player_set", "uuid": ..., "field": ..., "value": ... }`: the player's own state.
pub(in crate::game) fn set(game: &mut Game<'_>, request: &Value) -> bool {
    if let (Some(player), Some(field @ ("allow_flight" | "flying" | "fly_speed" | "walk_speed"))) = (uuid(request), request["field"].as_str()) {
        return set_flight(game, player, field, &request["value"]);
    }
    let Some(target) = uuid(request).and_then(|player| game.target_by_uuid(player)) else {
        return false;
    };
    let (field, value) = (request["field"].as_str().unwrap_or_default(), &request["value"]);
    let (int, float) = (value.as_i64().unwrap_or(0) as i32, value.as_f64().unwrap_or(0.0) as f32);
    game.for_player(target, |game| {
        match field {
            "exp_progress" => game.experience.progress = float,
            "exp_total" => game.experience.total = int,
            // `setGameMode` tells nobody, where `/gamemode` tells the player.
            "game_mode" => {
                use horizon_client::command::args::GameMode;
                let mode = match value.as_str().unwrap_or_default() {
                    "SURVIVAL" => GameMode::Survival,
                    "CREATIVE" => GameMode::Creative,
                    "ADVENTURE" => GameMode::Adventure,
                    "SPECTATOR" => GameMode::Spectator,
                    _ => return false,
                };
                return game.set_game_mode(mode);
            }
            "time_since_rest" => {
                let now = game.stats.get("custom", "minecraft:time_since_rest") as i32;
                game.stats.custom("time_since_rest", (int - now).into());
            }
            _ => {
                let Some(player) = game.player.as_mut() else {
                    return false;
                };
                match field {
                    "food" => player.health.food.food = int,
                    "saturation" => player.health.food.saturation = float,
                    "exhaustion" => player.health.food.exhaustion = float,
                    "fire_ticks" => player.set_remaining_fire_ticks(int),
                    "air" => player.air = int,
                    "freeze_ticks" => player.ticks_frozen = int,
                    "fall_distance" => player.fall_distance = value.as_f64().unwrap_or(0.0),
                    "health" => {
                        let _ = player.set_health(float, false);
                    }
                    _ => return false,
                }
            }
        }
        true
    })
    .unwrap_or(false)
}

/// What a guest's connection knows of them, for the `player` query.
pub(in crate::game) fn describe(state: &mut Value, guest: &horizon_client::lan::guest::Guest) {
    let (allow_flight, fly_speed, walk_speed) = flight_of(guest);
    state["allow_flight"] = json!(allow_flight);
    state["fly_speed"] = json!(fly_speed);
    state["walk_speed"] = json!(walk_speed);
    if let Some(motion) = guest.motion.as_ref() {
        state["fire_ticks"] = json!(motion.fire_ticks);
        state["air"] = json!(motion.air);
        state["fall_distance"] = json!(motion.fall_distance);
        state["freeze_ticks"] = json!(motion.ticks_frozen);
    }
    state["locale"] = json!(guest.info.language);
    state["main_hand"] = json!(guest.info.main_hand);
    state["ping"] = json!(guest.latency.load(std::sync::atomic::Ordering::Relaxed));
    state["chat_visibility"] = json!(guest.info.chat_visibility);
    state["chat_colors"] = json!(guest.info.chat_colors);
    state["skin_parts"] = json!(guest.info.model_customisation);
    state["text_filtering"] = json!(guest.info.text_filtering);
    state["allows_listing"] = json!(guest.info.allows_listing);
    state["particle_status"] = json!(guest.info.particle_status);
    state["server_view_distance"] = json!(guest.view_distance);
    state["living_flags"] = json!(guest.living_flags);
    state["sleep_timer"] = json!(guest.sleep_timer);
    state["bed"] = json!(guest.bed);
    if let Some((level, pos, ..)) = &guest.respawn_elsewhere {
        state["respawn_level"] = json!(level);
        state["respawn_pos"] = json!(pos);
    }
    state["spawn"] = json!(guest.spawn);
    state["last_action_tick"] = json!(guest.last_action_tick);
    state["sprint_input"] = json!(guest.input);
    state["profile"] = json!(guest.properties.iter().map(|property| json!({ "name": property.name, "value": property.value, "signature": property.signature })).collect::<Vec<_>>());
}

/// `{ "op": "attribute", "uuid": ..., "id": ... }`: a player's attribute as it stands.
pub(in crate::game) fn attribute(game: &mut Game<'_>, request: &Value) -> Value {
    let (Some(target), Some(id)) = (uuid(request).and_then(|player| game.target_by_uuid(player)), request["id"].as_str()) else {
        return Value::Null;
    };
    game.for_player(target, |game| {
        let player = game.player.as_mut()?;
        let base = player.attributes.value(Some(player.attributes.key(id)?), ValueKind::Base).ok()?;
        let value = player.attributes.value(Some(player.attributes.key(id)?), ValueKind::Computed).ok()?;
        Some(json!({ "base": base, "value": value }))
    })
    .flatten()
    .unwrap_or(Value::Null)
}

/// `{ "op": "attribute_set", ... }`: `/attribute` reaches only the player in place, and a plugin's
/// call can be for anyone.
pub(in crate::game) fn attribute_set(game: &mut Game<'_>, request: &Value) -> bool {
    let (Some(target), Some(id)) = (uuid(request).and_then(|player| game.target_by_uuid(player)), request["id"].as_str()) else {
        return false;
    };
    game.for_player(target, |game| {
        let Some(player) = game.player.as_mut() else {
            return false;
        };
        if let Some(modifier) = request["modifier"].as_str() {
            let operation = request["operation"].as_str().unwrap_or("add_value");
            return player.set_modifier(id, modifier, request["amount"].as_f64().unwrap_or(0.0), operation, request["add"].as_bool().unwrap_or(true)).is_ok();
        }
        let Some(handle) = player.attributes.key(id).and_then(|key| player.attributes.get_instance(Some(key)).ok().flatten()) else {
            return false;
        };
        let base = request["base"].as_f64().unwrap_or(0.0);
        player.attributes.edit(&handle, |value, observer| value.set_base_value(base, observer)).is_ok()
    })
    .unwrap_or(false)
}
