// What a run-time mod may ask the game, and be answered at once. A plugin reads the world all the
// time, and these answers can't wait for the end of a tick as the bridge's actions do. A mod changes
// the world through the game's own commands; only what no command does is here, as GUIs are.
use crate::game::{Game, native_server::NativeServerPlayer};
use horizon_client::items::ItemStack;
use horizon_world::nbt::Tag;
use serde_json::{Value, json};

fn uuid(value: &Value) -> Option<u128> {
    u128::from_str_radix(value.as_str()?, 16).ok()
}

fn player_json(p: &NativeServerPlayer) -> Value {
    json!({
        "uuid": format!("{:032x}", p.uuid),
        "name": p.name,
        "permission": p.permission,
        "dimension": p.dimension,
        "position": p.position,
        "game_mode": p.game_mode,
        "health": p.health,
        "yaw": p.yaw,
        "pitch": p.pitch,
    })
}

fn position(value: &Value) -> Option<[i32; 3]> {
    let list = value.as_array()?;
    Some([list.first()?.as_i64()? as i32, list.get(1)?.as_i64()? as i32, list.get(2)?.as_i64()? as i32])
}

/// A stack as a plugin reads it, each component as JSON.
pub(in crate::game) fn stack_json(stack: &horizon_client::items::ItemStack) -> Value {
    let mut value = json!({ "item": &*stack.item, "count": stack.count, "damage": stack.damage });
    if let Some(components) = horizon_client::item_data::encode_components(stack) {
        value["components"] = super::commands::paper_entities::tag_json(&components);
    }
    value
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

/// A play packet by name with the fields a plugin gave (`[field, value]` pairs): an entity's type by name, item stacks
/// as `{ "item": ..., "count": ..., "components": { ... } }` in entity data (`[index, 7, stack]`) and equipment
/// (`slots: [[slot, stack], ...]`).
fn build_packet(game: &mut Game<'_>, name: &str, fields: &Value) -> Result<horizon_protocol::packets::play::Clientbound, String> {
    use horizon_protocol::packets::play::Clientbound;
    // A blank packet of each name made once (`blank` reads 64 KB of zeroes) and copied after; entity data can't be
    // read from zeroes (its list ends at 0xFF), so that one is made empty here.
    thread_local! {
        static BLANKS: std::cell::RefCell<std::collections::HashMap<String, Clientbound>> = std::cell::RefCell::new(std::collections::HashMap::new());
    }
    let cached = BLANKS.with(|b| b.borrow().get(name).cloned());
    let mut packet = match cached {
        Some(p) => p,
        None => {
            let made = match name {
                "SetEntityData" => Clientbound::SetEntityData(horizon_protocol::packets::play::SetEntityData { id: 0, values: Vec::new() }),
                _ => Clientbound::blank(name).ok_or_else(|| format!("no packet named {name}"))?,
            };
            BLANKS.with(|b| b.borrow_mut().insert(name.to_owned(), made.clone()));
            made
        }
    };
    let mut items: Vec<(u8, Value)> = Vec::new();
    let mut slots: Option<Vec<(u8, Value)>> = None;
    for pair in fields.as_array().into_iter().flatten() {
        let field = pair[0].as_str().unwrap_or_default();
        let value = match (&pair[1], name, field) {
            (Value::String(kind), "AddEntity", "kind") => game.lan.as_ref().and_then(|host| host.hub.registry_id("minecraft:entity_type", kind)).map_or(Value::Null, |id| json!(id)),
            (Value::Array(entries), "SetEntityData", "values") => {
                // Item entries aside: the rest are plain values.
                let (stacks, plain): (Vec<&Value>, Vec<&Value>) = entries.iter().partition(|e| e[1].as_i64() == Some(7));
                items.extend(stacks.into_iter().map(|e| (e[0].as_u64().unwrap_or(0) as u8, e[2].clone())));
                Value::Array(plain.into_iter().cloned().collect())
            }
            (Value::Array(entries), "SetEquipment", "slots") => {
                slots = Some(entries.iter().map(|e| (e[0].as_u64().unwrap_or(0) as u8, e[1].clone())).collect());
                continue;
            }
            (other, _, _) => other.clone(),
        };
        packet.set_field(field, &value);
    }
    if !items.is_empty() || slots.is_some() {
        let Some(host) = game.lan.as_mut() else { return Ok(packet) };
        match &mut packet {
            Clientbound::SetEntityData(p) => {
                for (index, stack) in items {
                    let stack = if stack.is_null() { None } else { stack_from(&stack) };
                    p.values.push((index, horizon_protocol::entity_data::serializer::ITEM_STACK, horizon_protocol::entity_data::Value::Item(host.hub.stack(&stack))));
                }
            }
            Clientbound::SetEquipment(p) => {
                for (slot, stack) in slots.unwrap_or_default() {
                    let stack = if stack.is_null() { None } else { stack_from(&stack) };
                    p.slots.push((slot, host.hub.stack(&stack)));
                }
            }
            _ => {}
        }
    }
    Ok(packet)
}

/// Each player's sending half, for packets sent from a plugin's own thread (`packets_free`): set again from the
/// game thread whenever a plugin sends with it.
static SENDERS: std::sync::RwLock<Vec<(u128, horizon_client::lan::PlayerSender)>> = std::sync::RwLock::new(Vec::new());

fn refresh_senders(game: &Game<'_>) {
    let Some(host) = game.lan.as_ref() else { return };
    let now: Vec<(u128, horizon_client::lan::PlayerSender)> = host.hub.guests.iter().map(|g| (g.uuid, g.sender.clone())).collect();
    *SENDERS.write().unwrap_or_else(std::sync::PoisonError::into_inner) = now;
}

/// `{"op":"packets_free","packets":[{"to":[uuid],"raw":hex}]}` from any thread, without the game: the packets
/// handed to each player's writer at once. The answer: how many went, and the players not known yet (a
/// `packets_send` from the game thread makes them known).
pub(in crate::game) fn free_packets(request: &str) -> Option<String> {
    if !request.contains(r#""op":"packets_free""#) {
        return None;
    }
    let query: Value = serde_json::from_str(request).ok()?;
    let senders = SENDERS.read().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (mut sent, mut missing) = (0, Vec::new());
    let mut touched: Vec<usize> = Vec::new();
    let holding = horizon_client::lan::Holding::start();
    for entry in query["packets"].as_array().into_iter().flatten() {
        let bodies = raw_packets(entry["raw"].as_str().unwrap_or_default());
        for to in entry["to"].as_array().into_iter().flatten() {
            let Some(player) = uuid(to) else { continue };
            match senders.iter().position(|(u, _)| *u == player) {
                Some(i) => {
                    for body in &bodies {
                        senders[i].1.send_quiet(body.clone());
                        sent += 1;
                    }
                    if !touched.contains(&i) {
                        touched.push(i);
                    }
                }
                None => {
                    if !missing.contains(to) {
                        missing.push(to.clone());
                    }
                }
            }
        }
    }
    drop(holding);
    for i in touched {
        senders[i].1.flush();
    }
    Some(json!({ "sent": sent, "missing": missing }).to_string())
}

/// `packets_send`'s `raw`: hex of packets one after another, each its VarInt length then its id and body.
fn raw_packets(hex: &str) -> Vec<Vec<u8>> {
    let b = hex.as_bytes();
    let nib = |c: u8| match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    };
    let bytes: Vec<u8> = b.chunks_exact(2).map(|p| nib(p[0]) << 4 | nib(p[1])).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let (mut len, mut shift) = (0usize, 0);
        while i < bytes.len() {
            let byte = bytes[i];
            i += 1;
            len |= usize::from(byte & 0x7F) << shift;
            shift += 7;
            if byte & 0x80 == 0 || shift > 28 {
                break;
            }
        }
        if len == 0 || i + len > bytes.len() {
            break;
        }
        out.push(bytes[i..i + len].to_vec());
        i += len;
    }
    out
}

/// Answers one request, `{ "op": ..., ... }`, with JSON: `null` for what isn't there.
pub(in crate::game) fn answer(game: &mut Game<'_>, request: &str) -> String {
    let Ok(query) = serde_json::from_str::<Value>(request) else {
        return json!({ "error": "not JSON" }).to_string();
    };
    let answer = match query["op"].as_str().unwrap_or_default() {
        "players" => Value::Array(game.native_server_players().iter().map(player_json).collect()),
        "player" => {
            let wanted = uuid(&query["uuid"]);
            let name = query["name"].as_str();
            let found = game.native_server_players().into_iter().find(|p| Some(p.uuid) == wanted || name.is_some_and(|name| p.name.eq_ignore_ascii_case(name))).or_else(|| game.native_acting_player().filter(|p| Some(p.uuid) == wanted || name.is_some_and(|name| p.name.eq_ignore_ascii_case(name))));
            match found {
                Some(player) => {
                    let mut state = player_json(&player);
                    let guest = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == player.uuid));
                    // In a player's own command their state is swapped into the game.
                    let acting = guest.is_some_and(|g| game.lan_acting == Some(g.conn));
                    if let Some(g) = guest {
                        state["sneaking"] = json!(g.sneaking);
                        state["sprinting"] = json!(g.sprinting);
                        state["flying"] = json!(g.flying);
                        state["gliding"] = json!(g.fall_flying);
                        state["on_ground"] = json!(g.on_ground);
                        state["sleeping"] = json!(g.sleeping);
                        state["in_vehicle"] = json!(g.vehicle.is_some());
                        state["dead"] = json!(g.dead);
                        state["selected"] = json!(if acting { game.selected } else { g.selected });
                        state["food"] = json!(g.food);
                        state["saturation"] = json!(g.saturation);
                        state["level"] = json!(g.xp_level);
                        state["exp"] = json!(g.xp_progress);
                        state["total_exp"] = json!(g.xp_total);
                        state["entity_id"] = json!(g.entity_id);
                        state["brand"] = json!(g.brand);
                        state["address"] = json!(g.address.ip().to_string());
                        state["view_distance"] = json!(g.info.view_distance);
                        super::paper_players::describe(&mut state, g);
                        if let Some(body) = game.player.as_ref().filter(|_| acting) {
                            state["fire_ticks"] = json!(body.fire_ticks);
                            state["air"] = json!(body.air);
                            state["fall_distance"] = json!(body.fall_distance);
                            state["freeze_ticks"] = json!(body.ticks_frozen);
                        }
                    }
                    state
                }
                None => Value::Null,
            }
        }
        "effects" => {
            let player = uuid(&query["uuid"]);
            let guest = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| Some(g.uuid) == player));
            let effect = |e: &horizon_client_render::effects::EffectInstance| json!({ "id": e.id, "duration": e.duration, "amplifier": e.amplifier, "ambient": e.ambient, "visible": e.visible });
            match guest {
                Some(g) if game.lan_acting == Some(g.conn) => Value::Array(game.potions.effects.list.iter().map(effect).collect()),
                Some(g) => Value::Array(g.effects.list.iter().map(effect).collect()),
                None => Value::Null,
            }
        }
        "equipment" => {
            let guest = uuid(&query["uuid"]).and_then(|player| game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == player)));
            let stack = |stack: &Option<ItemStack>| stack.as_ref().map_or(Value::Null, stack_json);
            match guest {
                Some(g) if game.lan_acting == Some(g.conn) => json!({
                    "main_hand": game.inventory.get(game.selected).map_or(Value::Null, |held| stack(held)),
                    "off_hand": stack(&game.offhand),
                    "armor": game.armor.iter().map(stack).collect::<Vec<_>>(),
                }),
                Some(g) => json!({
                    "main_hand": g.inventory.get(g.selected).map_or(Value::Null, |held| stack(held)),
                    "off_hand": stack(&g.offhand),
                    "armor": g.armor.iter().map(stack).collect::<Vec<_>>(),
                    "cursor": stack(&g.carried),
                    "ender_chest": g.ender_items.iter().map(stack).collect::<Vec<_>>(),
                }),
                None => Value::Null,
            }
        }
        "block" => match position(&query["pos"]) {
            Some(pos) => {
                let dimension = query["dimension"].as_str().unwrap_or("minecraft:overworld");
                game.native_in_level(dimension, |game| {
                    let state = game.world.block(pos[0], pos[1], pos[2]);
                    let properties = game.pack.assets.registry.state(state).map_or(Value::Null, |state| json!(state.properties));
                    json!({ "name": game.native_block(pos), "properties": properties })
                })
                .unwrap_or(Value::Null)
            }
            None => Value::Null,
        },
        "block_property" => match (position(&query["pos"]), query["key"].as_str()) {
            (Some(pos), Some(key)) => json!(game.native_block_property(pos, key)),
            _ => Value::Null,
        },
        "inventory" => {
            let player = uuid(&query["uuid"]);
            let acting = game.lan.as_ref().is_some_and(|host| host.hub.guests.iter().any(|g| Some(g.uuid) == player && game.lan_acting == Some(g.conn)));
            let stacks = if acting { Some(game.inventory.to_vec()) } else { player.and_then(|player| game.native_player_stacks(player)) };
            match stacks {
                Some(stacks) => Value::Array(stacks.iter().map(|stack| stack.as_ref().map_or(Value::Null, stack_json)).collect()),
                None => Value::Null,
            }
        }
        "mobs" => {
            let dimension = query["dimension"].as_str().unwrap_or("minecraft:overworld");
            let mobs = game.native_mobs(dimension);
            Value::Array(mobs.iter().map(|m| json!({ "id": m.id, "dimension": m.dimension, "kind": m.kind, "pos": m.pos, "yaw": m.yaw, "health": m.health, "max_health": m.max_health })).collect())
        }
        "open_menu" => {
            let items = query["items"].as_array().into_iter().flatten().map(stack_from).collect();
            let title = horizon_client_render::packs::component(&query["title"]);
            match uuid(&query["uuid"]) {
                Some(player) => json!(game.native_open_menu(player, query["rows"].as_u64().unwrap_or(3) as u8, title, items)),
                None => Value::Null,
            }
        }
        "set_menu" => {
            let items = query["items"].as_array().into_iter().flatten().map(stack_from).collect();
            if let Some(player) = uuid(&query["uuid"]) {
                game.native_set_menu_items(player, items);
            }
            Value::Null
        }
        "close_menu" => {
            if let Some(player) = uuid(&query["uuid"]) {
                game.native_close_menu(player);
                // A table, a station or a chest is the game's own: it closes when the player's turn is over.
                super::paper_bridge::push_action(json!({ "op": "close_all", "player": query["uuid"] }).to_string());
            }
            Value::Null
        }
        "world" => json!({ "time": game.world.day_time, "game_time": game.world.game_time, "raining": game.world.weather.levels(1.0).0 > 0.2 }),
        "mob" => {
            let dimension = query["dimension"].as_str().unwrap_or("minecraft:overworld");
            let id = query["id"].as_u64().unwrap_or(0) as u32;
            game.native_mob(dimension, id).map_or(Value::Null, |m| json!({ "id": m.id, "dimension": m.dimension, "kind": m.kind, "pos": m.pos, "yaw": m.yaw, "health": m.health, "max_health": m.max_health }))
        }
        "mob_action" => {
            let dimension = query["dimension"].as_str().unwrap_or("minecraft:overworld");
            let id = query["id"].as_u64().unwrap_or(0) as u32;
            let vector = |value: &Value| Some([value.get(0)?.as_f64()?, value.get(1)?.as_f64()?, value.get(2)?.as_f64()?]);
            let done = match (query["action"].as_str().unwrap_or_default(), vector(&query["value"])) {
                ("teleport", Some(at)) => game.native_teleport_mob(dimension, id, at),
                ("push", Some(velocity)) => game.native_push_mob(dimension, id, velocity),
                ("health", _) => game.native_set_mob_health(dimension, id, query["value"].as_f64().unwrap_or(0.0) as f32),
                ("kill", _) => game.native_kill_mob(dimension, id),
                ("remove", _) => game.native_remove_entity(dimension, id) || super::paper_levels::discard_entity(game, dimension, id),
                _ => false,
            };
            json!(done)
        }
        "item_info" => json!({ "max_stack": game.res.items().get(query["item"].as_str().unwrap_or("minecraft:air")).map_or(64, |item| item.max_stack) }),
        "property_values" => super::paper_levels::property_values(game, query["block"].as_str().unwrap_or_default(), query["key"].as_str().unwrap_or_default()),
        // Bulk block I/O by state id, packed and base64 in one string (FAWE): see paper_levels.
        "state_names" => super::paper_levels::state_names(game),
        "blocks_write" => super::paper_levels::blocks_write(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), query["data"].as_str().unwrap_or_default()),
        "blocks_read" => match (query["chunk"][0].as_i64(), query["chunk"][1].as_i64()) {
            (Some(x), Some(z)) => super::paper_levels::blocks_read(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), [x as i32, z as i32]),
            _ => Value::Null,
        },
        "box_read" => super::paper_levels::box_read(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "box_fill" => super::paper_levels::box_fill(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "box_write" => super::paper_levels::box_write(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "box_entities" => super::paper_levels::box_entities(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "box_undo" => super::paper_levels::box_replay(game, &query, true),
        "box_redo" => super::paper_levels::box_replay(game, &query, false),
        "box_forget" => super::paper_levels::box_forget(&query),
        "box_regen" => super::paper_levels::box_regen(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "box_biomes" => super::paper_levels::box_biomes(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "box_set_biomes" => super::paper_levels::box_set_biomes(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "box_mobs" => super::paper_levels::box_mobs(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "box_summon" => super::paper_levels::box_summon(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), &query),
        "set_block" => match (position(&query["pos"]), query["block"].as_str()) {
            (Some(pos), Some(block)) => json!(super::paper_levels::set_block(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), pos, block)),
            _ => json!(false),
        },
        "suggest" => {
            super::paper_bridge::add_suggestions(&query["matches"]);
            Value::Null
        }
        "chunk_entities" => match (query["chunk"][0].as_i64(), query["chunk"][1].as_i64()) {
            (Some(x), Some(z)) => super::paper_levels::chunk_block_entities(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), [x as i32, z as i32]),
            _ => Value::Null,
        },
        "scoreboard" => super::paper_levels::scoreboard(game),
        "tag" => super::paper_levels::tag_members(game, query["kind"].as_str().unwrap_or("block"), query["name"].as_str().unwrap_or_default()),
        // A block state's name with its properties, by its number in a packet (ProtocolLib's WrappedBlockData).
        "state_name" => json!(super::paper_fam_ev_world::state_name(horizon_world::registry::BlockStateId(query["state"].as_u64().unwrap_or(0) as u32))),
        // ProtocolLib's createPacket: a play packet of this name, its fields blank.
        "packet_blank" => {
            let name = query["name"].as_str().unwrap_or_default();
            horizon_protocol::packets::play::Clientbound::blank(name).map_or(Value::Null, |mut packet| {
                Value::Array(packet.fields().into_iter().map(|(field, kind, value)| json!([field, kind, value])).collect())
            })
        }
        // ProtocolLib's sendServerPacket: the packet a plugin filled, to one player. It doesn't pass the
        // plugins' own packet listeners again (a listener that sends would loop).
        "packet_send" => {
            let name = query["name"].as_str().unwrap_or_default();
            match build_packet(game, name, &query["fields"]) {
                Err(error) => json!({ "error": error }),
                Ok(packet) => {
                    let guest = uuid(&query["uuid"]).and_then(|player| game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == player)));
                    match guest {
                        Some(guest) => {
                            guest.send_quietly(&packet);
                            json!(true)
                        }
                        None => json!(false),
                    }
                }
            }
        }
        // Many packets in one query: `{ "packets": [{ "to": [uuid...], "name": ..., "fields": [...] }, ...] }`, each
        // built once and sent to each player it names, in order. Answers how many were sent.
        "packets_send" => {
            refresh_senders(game);
            // Held while they're made and handed to each player's writer together after: one wake-up per player,
            // not one per packet (each a system call).
            let mut sent = 0;
            let mut touched: Vec<u128> = Vec::new();
            let holding = horizon_client::lan::Holding::start();
            for entry in query["packets"].as_array().into_iter().flatten() {
                // Packets the plugin made itself: `raw` is hex of (VarInt length, id and body) after each other.
                if let Some(hex) = entry["raw"].as_str() {
                    let Some(host) = game.lan.as_ref() else { break };
                    let bodies = raw_packets(hex);
                    for to in entry["to"].as_array().into_iter().flatten() {
                        if let Some(player) = uuid(to)
                            && let Some(guest) = host.hub.guests.iter().find(|g| g.uuid == player)
                        {
                            for body in &bodies {
                                guest.sender.send_quiet(body.clone());
                                sent += 1;
                            }
                            if !touched.contains(&player) {
                                touched.push(player);
                            }
                        }
                    }
                    continue;
                }
                let Ok(packet) = build_packet(game, entry["name"].as_str().unwrap_or_default(), &entry["fields"]) else { continue };
                let Some(host) = game.lan.as_ref() else { break };
                for to in entry["to"].as_array().into_iter().flatten() {
                    if let Some(player) = uuid(to)
                        && let Some(guest) = host.hub.guests.iter().find(|g| g.uuid == player)
                    {
                        guest.send_quietly(&packet);
                        if !touched.contains(&player) {
                            touched.push(player);
                        }
                        sent += 1;
                    }
                }
            }
            drop(holding);
            if let Some(host) = game.lan.as_ref() {
                for guest in host.hub.guests.iter().filter(|g| touched.contains(&g.uuid)) {
                    guest.sender.flush();
                }
            }
            json!(sent)
        }
        // The ids of packets by name, for plugins that make packets themselves (`packets_send` with `raw`).
        "packet_ids" => {
            refresh_senders(game);
            let mut ids = serde_json::Map::new();
            for name in query["names"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                if let Ok(p) = build_packet(game, name, &json!([])) {
                    ids.insert(name.to_owned(), json!(p.id()));
                }
            }
            Value::Object(ids)
        }
        "tags" => super::paper_levels::tag_names(game, query["kind"].as_str().unwrap_or("block")),
        "data_file" => super::paper_levels::data_file(game, query["path"].as_str().unwrap_or_default()),
        "data_list" => super::paper_levels::data_list(game, query["path"].as_str().unwrap_or_default()),
        "boss_bar" => super::paper_levels::boss_bar(game, query["id"].as_str().unwrap_or_default()),
        "level" => super::paper_levels::level(game, query["dimension"].as_str().unwrap_or("minecraft:overworld")),
        "block_facts" => match position(&query["pos"]) {
            Some(pos) => super::paper_levels::block_facts(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), pos),
            None => Value::Null,
        },
        "game_rule" => super::paper_levels::game_rule(game, query["name"].as_str().unwrap_or_default()),
        "tick_times" => super::paper_levels::tick_times(game),
        "resend_commands" => {
            if let Some(host) = game.lan.as_mut() {
                host.hub.resend_commands();
            }
            Value::Null
        }
        "attribute" => super::paper_players::attribute(game, &query),
        "attribute_set" => json!(super::paper_players::attribute_set(game, &query)),
        "player_send" => json!(super::paper_players::send(game, &query)),
        "player_set" => json!(super::paper_players::set(game, &query)),
        "entities" => game.native_entities(query["dimension"].as_str().unwrap_or("minecraft:overworld")),
        "entity" => game.native_entity(&query),
        "mob_watch" => game.native_mob_watch(&query),
        "entity_merge" => json!(game.native_entity_merge(&query)),
        "spawn" => game.native_spawn(query["dimension"].as_str().unwrap_or("minecraft:overworld"), query["snbt"].as_str().unwrap_or_default()),
        "block_data" => match position(&query["pos"]) {
            Some(pos) => game.native_block_data(query["dimension"].as_str().unwrap_or("minecraft:overworld"), pos),
            None => Value::Null,
        },
        "block_merge" => match position(&query["pos"]) {
            Some(pos) => json!(game.native_block_merge(query["dimension"].as_str().unwrap_or("minecraft:overworld"), pos, query["snbt"].as_str().unwrap_or_default())),
            None => json!(false),
        },
        "storage" => game.native_storage(query["id"].as_str().unwrap_or_default()),
        "chunk_loaded" => {
            let (x, z) = (query["chunk"][0].as_i64().unwrap_or(0) as i32, query["chunk"][1].as_i64().unwrap_or(0) as i32);
            let dimension = query["dimension"].as_str().unwrap_or("minecraft:overworld");
            json!(game.native_in_level(dimension, |game| game.world.chunks.contains_key(&(x, z))).unwrap_or(false))
        }
        "plugin_message" => {
            let bytes: Vec<u8> = query["data"].as_array().into_iter().flatten().filter_map(|b| b.as_u64().map(|b| b as u8)).collect();
            match (uuid(&query["uuid"]), query["channel"].as_str()) {
                (Some(player), Some(channel)) => json!(game.native_net_send_to_player(player, channel, &bytes).is_ok()),
                _ => json!(false),
            }
        }
        "listen_channel" => {
            if let Some(channel) = query["channel"].as_str() {
                crate::game::paper_bridge::listen_channel(channel);
            }
            Value::Null
        }
        "version" => json!({
            "minecraft": "26.3",
            "engine": super::paper_events::VERSION,
            "data_version": horizon_world::DATA_VERSION,
            "protocol_version": horizon_protocol::PROTOCOL_VERSION,
            // The server fork, for a plugin that prints it (`/version`, `/plugins` in the core plugin).
            "fork": "Horizon Paper engine",
            "build": option_env!("HORIZON_ENGINE_COMMIT").unwrap_or("local"),
            "built": option_env!("HORIZON_ENGINE_BUILT").unwrap_or(""),
            "horizon": option_env!("HORIZON_SOURCE_COMMIT").unwrap_or(""),
            "api_version": "26.3-R0.1-SNAPSHOT",
            "hot_reload": crate::game::paper_fam_core::hot_reload::on(),
        }),
        "tick_state" => super::paper_levels::tick_state(game),
        "event_patch" => {
            super::paper_bridge::carry(query["key"].as_str().unwrap_or_default(), query["value"].as_str().unwrap_or("null"));
            json!(true)
        }
        "line_set" => {
            let line = (!query["line"].is_null()).then(|| super::paper_players::text(&query["line"]));
            super::paper_bridge::line_set(query["kind"].as_u64().unwrap_or(9) as usize, line);
            json!(true)
        }
        "lead_motd" => match game.lan.as_ref() {
            Some(host) => {
                horizon_client::lan::paper_lan::set_motd(&host.hub.lan.shared, query["motd"].as_str().unwrap_or_default());
                json!(true)
            }
            None => json!(false),
        },
        "lead_max_players" => match game.lan.as_ref() {
            Some(host) => {
                horizon_client::lan::paper_lan::set_max_players(&host.hub.lan.shared, query["count"].as_u64().unwrap_or(0) as usize);
                json!(true)
            }
            None => json!(false),
        },
        "lead_material" => {
            let id = query["id"].as_str().unwrap_or_default();
            json!({ "block": game.pack.assets.registry.default_state(id).is_some(), "item": game.res.items().get(id).is_some() })
        }
        "lead_mods" => super::paper_bridge::mod_ids(),
        "lead_fire" => json!(super::paper_bridge::fire_custom(game, query["event"].as_str().unwrap_or_default(), query["payload"].clone())),
        "lead_spawn_limit" => {
            horizon_client::paper_hub::set_spawn_limit(query["category"].as_str().unwrap_or_default(), query["limit"].as_i64().unwrap_or(-1) as i32);
            json!(true)
        }
        "bone_meal" => match position(&query["pos"]) {
            Some(pos) => json!(super::paper_levels::bone_meal(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), pos)),
            None => json!(false),
        },
        "mob_type" => game.res.mob_types().get(query["id"].as_str().unwrap_or_default()).map_or(Value::Null, |kind| json!({ "category": kind.category, "width": kind.width, "height": kind.height, "eye_height": kind.eye_height })),
        "no_drops" => {
            super::paper_bridge::no_drops(game);
            json!(true)
        }
        "chunk_snapshot" => match (query["chunk"][0].as_i64(), query["chunk"][1].as_i64()) {
            (Some(x), Some(z)) => super::paper_levels::chunk_snapshot(game, query["dimension"].as_str().unwrap_or("minecraft:overworld"), [x as i32, z as i32]),
            _ => Value::Null,
        },
        other => {
            if let Some(answer) = super::paper_fam_ev_entity::query(game, other, &query) {
                return answer.to_string();
            }
            if let Some(answer) = super::paper_fam_ev_player::query(game, other, &query) {
                return answer.to_string();
            }
            if let Some(answer) = super::paper_fam_ev_world::query(game, other, &query) {
                return answer.to_string();
            }
            if let Some(answer) = super::paper_fam_entity::query(game, other, &query) {
                return answer.to_string();
            }
            if let Some(answer) = super::paper_fam_core::query(game, other, &query) {
                return answer.to_string();
            }
            if let Some(answer) = super::paper_fam_items::query(game, other, &query) {
                return answer.to_string();
            }
            if let Some(answer) = super::paper_fam_paper::query(game, other, &query) {
                return answer.to_string();
            }
            if let Some(answer) = super::paper_fam_ess::query(game, other, &query) {
                return answer.to_string();
            }
            if let Some(answer) = super::paper_worlds::query(game, other, &query) {
                return answer.to_string();
            }
            // Ops another bundle registered (`add_queries`), tried in the order they came.
            let extra = EXTRA.lock().unwrap_or_else(|e| e.into_inner()).clone();
            for f in extra {
                if let Some(answer) = f(game, other, &query) {
                    return answer.to_string();
                }
            }
            return json!({ "error": format!("no such query: {other}") }).to_string();
        }
    };
    answer.to_string()
}

/// A query handler another bundle adds: `Some(answer)` for an op it knows.
type ExtraQuery = fn(&mut Game<'_>, &str, &Value) -> Option<Value>;

static EXTRA: std::sync::Mutex<Vec<ExtraQuery>> = std::sync::Mutex::new(Vec::new());

/// Ops of another bundle (applied after this one), answered after every op of this one; once per function.
pub(in crate::game) fn add_queries(f: ExtraQuery) {
    let mut extra = EXTRA.lock().unwrap_or_else(|e| e.into_inner());
    if !extra.contains(&f) {
        extra.push(f);
    }
}
