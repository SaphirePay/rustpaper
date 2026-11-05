// Family paper: answers to the queries its SDK file makes.
use crate::game::Game;
use horizon_mod_runtime as runtime;
use horizon_protocol::entity_data::{Particle, ParticleData, PositionSource};
use horizon_protocol::packets::play::{self, Clientbound as C};
use serde_json::{Value, json};

fn triple(value: &Value) -> [f64; 3] {
    let at = |index: usize| value[index].as_f64().unwrap_or_default();
    [at(0), at(1), at(2)]
}

/// A stack as the SDK writes it: its item, count and components, each as SNBT.
fn stack_from(value: &Value) -> Option<horizon_client::items::ItemStack> {
    let mut components = horizon_world::nbt::Compound::new();
    for (key, snbt) in value["components"].as_object().into_iter().flatten() {
        if let Ok((tag, _)) = horizon_world::nbt::parse_snbt(snbt.as_str()?) {
            components.insert(key.as_str().into(), tag);
        }
    }
    let tag = horizon_world::nbt::Tag::compound([
        ("id", horizon_world::nbt::Tag::String(value["item"].as_str()?.into())),
        ("count", horizon_world::nbt::Tag::Int(i32::try_from(value["count"].as_i64().unwrap_or(1)).unwrap_or(1))),
        ("components", horizon_world::nbt::Tag::Compound(components)),
    ]);
    horizon_client::save::item_from_nbt(&tag)
}

/// What a type of particle carries, read from the SDK's JSON. The packet is `ParticleTypes.STREAM_CODEC`'s, so
/// data of the wrong shape would break the client: `None` refuses it, and `Some(None)` is a type with no data.
fn particle_data(game: &mut Game<'_>, name: &str, data: &Value) -> Option<Option<ParticleData>> {
    let int = |key: &str| data[key].as_i64().map(|value| value as i32);
    let float = |key: &str| data[key].as_f64().map(|value| value as f32);
    Some(Some(match name.trim_start_matches("minecraft:") {
        "block" | "block_marker" | "falling_dust" | "dust_pillar" | "block_crumble" => ParticleData::State(super::render_control::parse_state(game.pack.assets.registry, data["block"].as_str()?).ok()?.0 as i32),
        "entity_effect" | "tinted_leaves" | "flash" => ParticleData::Color(int("color")?),
        "dust" => ParticleData::ColorScale(int("color")?, float("scale")?),
        "dust_color_transition" => ParticleData::ColorScale2(int("from")?, int("to")?, float("scale")?),
        "effect" | "instant_effect" => ParticleData::ColorPower(int("color")?, float("power")?),
        "dragon_breath" | "sculk_charge" => ParticleData::Scalar(float("scale")?),
        "shriek" => ParticleData::Delay(int("delay")?),
        "trail" => ParticleData::Trail(triple(data.get("target")?), int("color")?, int("duration")?),
        "item" => {
            let stack = stack_from(&data["item"])?;
            ParticleData::Item(game.lan.as_mut()?.hub.stack(&Some(stack))?)
        }
        "vibration" => {
            let ticks = int("ticks")?;
            let source = match data.get("pos") {
                Some(pos) => PositionSource::Block(triple(pos).map(|value| value.floor() as i32)),
                None => PositionSource::Entity { id: int("entity")?, y_offset: float("offset").unwrap_or(0.0) },
            };
            ParticleData::Vibration { source, ticks }
        }
        "geyser" | "geyser_plume" => ParticleData::Geyser(int("water")?),
        "geyser_base" | "geyser_poof" => ParticleData::GeyserBase(int("water")?, float("impulse")?),
        _ => return data.is_null().then_some(None),
    }))
}

/// `{ "op": "paper_particle", ... }`: `ServerLevel.sendParticles` to the players a plugin named. False
/// where the hub is lent out to a guest's turn, and for a particle that's unknown or has the wrong data.
pub(in crate::game) fn particle(game: &mut Game<'_>, request: &Value) -> Value {
    let Some(name) = request["particle"].as_str() else {
        return json!(false);
    };
    let Some(data) = particle_data(game, name, &request["data"]) else {
        return json!(false);
    };
    let Some(host) = game.lan.as_ref() else {
        return json!(false);
    };
    let Some(id) = host.hub.registry_id("minecraft:particle_type", name) else {
        return json!(false);
    };
    let at = triple(&request["pos"]);
    let force = request["force"].as_bool().unwrap_or(false);
    let packet = C::LevelParticles(play::LevelParticles {
        particle: Particle { id, data },
        override_limiter: force,
        always_show: false,
        pos: at,
        dist: triple(&request["offset"]).map(|v| v as f32),
        max_speed: triple(&request["speed"]).map(|v| v as f32),
        count: request["count"].as_i64().unwrap_or(1) as i32,
        randomization: request["randomization"].as_i64().unwrap_or(0) as i32,
    });
    let range: f64 = if force { 512.0 } else { 32.0 };
    let viewers: Vec<u128> = request["viewers"].as_array().into_iter().flatten().filter_map(|uuid| u128::from_str_radix(uuid.as_str()?, 16).ok()).collect();
    let dimension = request["dimension"].as_str().unwrap_or("minecraft:overworld");
    for guest in host.hub.guests.iter().filter(|guest| guest.placed && guest.dimension == dimension && viewers.contains(&guest.uuid)) {
        let distance: f64 = (0..3).map(|k| (guest.position[k].floor() + 0.5 - at[k]).powi(2)).sum();
        if distance < range * range {
            guest.send(&packet);
        }
    }
    json!(true)
}

/// `{ "op": "paper_registry", "registry": "block" }`: a registry's names. Built in ones from the hub's
/// tables, by network id; the rest are data, and the folder lists them.
fn registry(game: &Game<'_>, request: &Value) -> Value {
    let path = request["registry"].as_str().unwrap_or_default();
    let key = format!("minecraft:{path}");
    if let Some(host) = game.lan.as_ref() {
        let names: Vec<&str> = (0..).map_while(|id| host.hub.registry_name(&key, id)).collect();
        if !names.is_empty() {
            return json!(names);
        }
    }
    super::paper_levels::data_list(game, path)
}

/// `{ "op": "paper_snbt", "text": ... }`: SNBT as the JSON a plugin reads.
fn snbt(request: &Value) -> Value {
    horizon_world::nbt::parse_snbt(request["text"].as_str().unwrap_or_default()).map_or(Value::Null, |(tag, _)| super::commands::paper_entities::tag_json(&tag))
}

/// `{ "op": "paper_select", "selector": "@e[...]" }`: `EntitySelector.findEntities`, from where the game's
/// commands stand now. Players by uuid, the rest by id and level.
fn select(game: &mut Game<'_>, request: &Value) -> Value {
    use super::commands::Target as T;
    let text = request["selector"].as_str().unwrap_or_default();
    let mut reader = horizon_client::command::Reader::new(text);
    let Ok(selector) = horizon_client::command::selector::parse(&mut reader, &game.ui.registries, true) else {
        return json!({ "players": [], "entities": [] });
    };
    let mut players = Vec::new();
    let mut entities = Vec::new();
    for target in game.select(&selector) {
        let id = match target {
            T::Player => {
                let acting = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| Some(guest.conn) == game.lan_acting));
                players.extend(acting.map(|guest| format!("{:032x}", guest.uuid)).or_else(|| request["source"].as_str().map(str::to_owned)));
                continue;
            }
            T::Guest(conn) => {
                let guest = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| guest.conn == conn));
                players.extend(guest.map(|guest| format!("{:032x}", guest.uuid)));
                continue;
            }
            T::Mob(id) | T::Item(id) | T::Orb(id) | T::Projectile(id) | T::Stand(id) | T::Frame(id) | T::Painting(id) | T::Boat(id) | T::Cart(id) | T::Other(id) | T::Crystal(id) | T::Dragon(id) => id,
        };
        entities.push(json!({ "id": id, "dimension": game.target_level(target).id() }));
    }
    json!({ "players": players, "entities": entities })
}

/// The packs a world has: the built-in one, then its folder's, each with `pack.mcmeta` when it's a folder.
fn datapacks(game: &mut Game<'_>) -> Value {
    let stack = super::functions::pack_stack(&game.world_root, &game.options.resources, game.cmds.vanilla_disabled, &game.level.enabled_features);
    let mut rows = Vec::new();
    if stack.builtin.is_some() {
        rows.push(json!({ "id": "vanilla", "enabled": true, "meta": { "pack": { "description": "The default data for Minecraft" } } }));
    }
    for pack in &stack.packs {
        let meta = pack.read("pack.mcmeta").and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok()).unwrap_or_else(|| json!({ "pack": { "description": pack.description() } }));
        let plugins = horizon_client::datapack::PLUGIN_PACKS.lock().unwrap_or_else(|e| e.into_inner());
        let plugin = plugins.iter().find(|known| known.id == pack.id);
        rows.push(json!({ "id": pack.id, "enabled": stack.enabled.contains(&pack.id), "meta": meta, "title": plugin.map(|plugin| &plugin.title), "required": plugin.is_some_and(|plugin| plugin.required), "plugin": plugin.is_some() }));
    }
    json!({ "packs": rows, "format": horizon_core::version::DATA_PACK_FORMAT.0 })
}

fn guest_of<'a>(game: &'a Game<'_>, request: &Value) -> Option<&'a horizon_client::lan::guest::Guest> {
    let player = u128::from_str_radix(request["uuid"].as_str()?, 16).ok()?;
    game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == player)
}

fn component_tag(value: &Value) -> horizon_world::nbt::Tag {
    horizon_client::lan::data::component_nbt(&horizon_client_render::packs::component(value).component())
}

/// `{ "op": "paper_links", ... }`: `ClientboundServerLinksPacket` to one player.
fn links(game: &Game<'_>, request: &Value) -> Value {
    use horizon_protocol::packets::configuration::ServerLinks;
    let Some(guest) = guest_of(game, request) else {
        return json!(false);
    };
    guest.send(&C::ServerLinks(ServerLinks { links: server_links(request) }));
    json!(true)
}

fn server_links(request: &Value) -> Vec<horizon_protocol::packets::configuration::ServerLink> {
    use horizon_protocol::packets::configuration::{ServerLink, ServerLinkKind};
    request["links"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| ServerLink {
            kind: row["known"].as_i64().map_or_else(|| ServerLinkKind::Custom(component_tag(&row["label"])), |known| ServerLinkKind::Known(known as i32)),
            link: row["url"].as_str().unwrap_or_default().to_owned(),
        })
        .collect()
}

/// `{ "op": "paper_report_details", ... }`: `ClientboundCustomReportDetailsPacket`, which the protocol crate
/// has an id for but no struct: a map of up to 32 strings.
fn report_details(game: &Game<'_>, request: &Value) -> Value {
    let Some(guest) = guest_of(game, request) else {
        return json!(false);
    };
    let mut body = horizon_protocol::buf::Writer::new();
    body.varint(horizon_protocol::ids::play::clientbound::CUSTOM_REPORT_DETAILS);
    write_report_details(&mut body, request);
    guest.sender.send(body.data);
    json!(true)
}

fn write_report_details(body: &mut horizon_protocol::buf::Writer, request: &Value) {
    let details: Vec<(&String, &str)> = request["details"].as_object().into_iter().flatten().filter_map(|(key, value)| Some((key, value.as_str()?))).filter(|(key, value)| key.len() <= 128 && value.len() <= 4096).take(32).collect();
    body.varint(details.len() as i32);
    for (key, value) in details {
        body.string(key);
        body.string(value);
    }
}

/// `{ "op": "paper_profile_lookup", "id": hex or null, "name": ..., "wait": ms }`: the session servers through
/// the game's own cache, which fetches off this thread. `{ "pending": true }` while it's on its way.
fn profile_lookup(game: &Game<'_>, request: &Value) -> Value {
    use horizon_client::skins::Resolve;
    use horizon_client_render::chunk::block_entity::NameOrId;
    let key = match request["id"].as_str().and_then(|id| u128::from_str_radix(id, 16).ok()) {
        Some(id) => NameOrId::Id(id),
        None => NameOrId::Name(request["name"].as_str().unwrap_or_default().to_owned()),
    };
    let until = std::time::Instant::now() + std::time::Duration::from_millis(request["wait"].as_u64().unwrap_or(0));
    loop {
        let state = game.skin_profiles.borrow_mut().state(&key);
        match state {
            Resolve::Pending if std::time::Instant::now() < until => std::thread::sleep(std::time::Duration::from_millis(20)),
            Resolve::Pending => return json!({ "pending": true }),
            Resolve::Done(None) => return Value::Null,
            Resolve::Done(Some(profile)) => {
                let properties: Vec<Value> = profile.properties.iter().map(|(name, value, signature)| json!({ "name": name, "value": value, "signature": signature })).collect();
                return json!({ "id": horizon_protocol::crypto::uuid_string(profile.id), "name": profile.name, "properties": properties });
            }
        }
    }
}

/// `{ "op": "paper_command_source" }`: who and where the game's command source is, while `/execute` set one.
fn command_source(game: &Game<'_>) -> Value {
    use super::commands::Target as T;
    let Some(source) = game.command_source.as_ref() else {
        return Value::Null;
    };
    let mut answer = json!({ "pos": source.pos, "rot": source.rot, "dimension": source.dimension.id() });
    match source.entity {
        Some(T::Player) => {
            let acting = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| Some(guest.conn) == game.lan_acting));
            answer["player"] = json!(acting.map(|guest| format!("{:032x}", guest.uuid)));
        }
        Some(T::Guest(conn)) => {
            let guest = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| guest.conn == conn));
            answer["player"] = json!(guest.map(|guest| format!("{:032x}", guest.uuid)));
        }
        Some(T::Mob(id) | T::Item(id) | T::Orb(id) | T::Projectile(id) | T::Stand(id) | T::Frame(id) | T::Painting(id) | T::Boat(id) | T::Cart(id) | T::Other(id) | T::Crystal(id) | T::Dragon(id)) => answer["entity"] = json!(id),
        None => {}
    }
    answer
}

/// `{ "op": "paper_connection_asks" }`: the connections a listener can still answer. What last tick's listeners
/// had goes out first, so every plugin's listeners see one connection in the same tick.
fn connection_asks(game: &Game<'_>) -> Value {
    use horizon_client::paper_hub as hub;
    *hub::CONNECTIONS_POLLED.lock().unwrap_or_else(|e| e.into_inner()) = Some(std::time::Instant::now());
    let tick = game.ticks as i64 + 1;
    let mut asks = hub::CONNECTION_ASKS.lock().unwrap_or_else(|e| e.into_inner());
    asks.retain_mut(|ask| {
        if ask.seen == 0 || ask.seen == tick {
            return true;
        }
        let _ = ask.answer.send(hub::ConnectionAnswer { kick: ask.kick.take(), packets: std::mem::take(&mut ask.packets) });
        false
    });
    Value::Array(
        asks.iter_mut()
            .map(|ask| {
                ask.seen = tick;
                let mut row = ask.payload.clone();
                row["ask"] = json!(ask.id);
                row["kind"] = json!(ask.kind);
                row["kick"] = ask.kick.clone().unwrap_or(Value::Null);
                row
            })
            .collect(),
    )
}

/// `{ "op": "paper_connection_answer", "ask": id, "held": id, "kick": component or null }`.
fn connection_answer(request: &Value) -> Value {
    use horizon_client::paper_hub as hub;
    {
        let mut asks = hub::CONNECTION_ASKS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ask) = asks.iter_mut().find(|ask| Some(u64::from(ask.id)) == request["ask"].as_u64()) {
            ask.kick = request.get("kick").filter(|kick| !kick.is_null()).cloned();
            return json!(true);
        }
    }
    json!(request["held"].as_u64().is_some_and(|id| hub::held_kick(id as u32, &request["kick"])))
}

const BUILD_TIME_MS: u64 = 0;

static SIGNED_ARGUMENTS: std::sync::Mutex<Vec<(String, Value)>> = std::sync::Mutex::new(Vec::new());

/// The arguments of the line about to run, as the sender's chain let them through.
pub(in crate::game) fn note_signed_arguments(signed: Option<&[(String, horizon_client::lan::hub::chat::ChatMessage)]>) {
    let rows = signed.into_iter().flatten().map(|(name, message)| {
        (name.clone(), json!({ "message": message.content, "timestamp": message.timestamp, "salt": message.salt, "signature": message.signature.as_ref().map(|signature| signature.to_vec()), "uuid": format!("{:032x}", message.sender), "sender": format!("{:032x}", message.sender) }))
    });
    *SIGNED_ARGUMENTS.lock().unwrap_or_else(|e| e.into_inner()) = rows.collect();
}

/// `SignableCommand.of` for a plugin's command: what the game's dispatcher can't parse, as the tree's in the plugin.
pub(in crate::game) fn plugin_signable(game: &mut Game<'_>, command: &str) -> Vec<(String, String)> {
    if !runtime::listens("paper_signable") {
        return Vec::new();
    }
    let payload = json!({ "line": command }).to_string();
    let (_, carried) = runtime::lend(game, || runtime::fire_carried("paper_signable", &payload));
    let value: Value = carried.and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null);
    value["signable"].as_array().into_iter().flatten().filter_map(|row| Some((row[0].as_str()?.to_owned(), row[1].as_str()?.to_owned()))).collect()
}

/// A packet of the configuration phase, by name.
fn configuration_packet(request: &Value) -> Option<Vec<u8>> {
    use horizon_protocol::packets::{Empty, configuration as cfg};
    let body = match request["packet"].as_str().unwrap_or_default() {
        "clear_chat" => cfg::Clientbound::ResetChat(Empty),
        "clear_dialog" => cfg::Clientbound::ClearDialog(Empty),
        "dialog" => cfg::Clientbound::ShowDialog(cfg::ShowDialog { dialog: horizon_world::nbt::parse_snbt(request["snbt"].as_str().unwrap_or_default()).ok()?.0 }),
        "transfer" => cfg::Clientbound::Transfer(cfg::Transfer { host: request["host"].as_str().unwrap_or_default().to_owned(), port: request["port"].as_i64().unwrap_or(25565) as i32 }),
        "store_cookie" => cfg::Clientbound::StoreCookie(cfg::StoreCookie { key: request["key"].as_str().unwrap_or_default().to_owned(), payload: request["data"].as_array().into_iter().flatten().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect() }),
        "cookie_request" => cfg::Clientbound::CookieRequest(cfg::CookieRequest { key: request["key"].as_str().unwrap_or_default().to_owned() }),
        "pack_push" => cfg::Clientbound::ResourcePackPush(cfg::ResourcePackPush {
            id: u128::from_str_radix(request["id"].as_str()?, 16).ok()?,
            url: request["url"].as_str()?.to_owned(),
            hash: request["hash"].as_str().unwrap_or_default().to_owned(),
            required: request["required"].as_bool().unwrap_or(false),
            prompt: request.get("prompt").filter(|prompt| !prompt.is_null()).map(component_tag),
        }),
        "pack_pop" => cfg::Clientbound::ResourcePackPop(cfg::ResourcePackPop { id: request["id"].as_str().and_then(|id| u128::from_str_radix(id, 16).ok()) }),
        "links" => cfg::Clientbound::ServerLinks(cfg::ServerLinks { links: server_links(request) }),
        "report_details" => {
            let mut body = horizon_protocol::buf::Writer::new();
            body.varint(horizon_protocol::ids::configuration::clientbound::CUSTOM_REPORT_DETAILS);
            write_report_details(&mut body, request);
            return Some(body.data);
        }
        _ => return None,
    };
    Some(body.encode())
}

/// `{ "op": "paper_connection_send", "ask": id, "held": id, "packet": ... }`: to a listener's connection this
/// tick, or to whoever's held in configuration.
fn connection_send(request: &Value) -> Value {
    use horizon_client::paper_hub as hub;
    let Some(body) = configuration_packet(request) else {
        return json!(false);
    };
    {
        let mut asks = hub::CONNECTION_ASKS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ask) = asks.iter_mut().find(|ask| Some(u64::from(ask.id)) == request["ask"].as_u64()) {
            ask.packets.push(body);
            return json!(true);
        }
    }
    json!(request["held"].as_u64().is_some_and(|id| hub::held_packet(id as u32, body)))
}

/// `allowed_symlinks.txt`, by its `[prefix]` rules: no other can be read here.
fn symlink_allowed(link: &std::path::Path) -> bool {
    let Ok(target) = std::fs::read_link(link) else {
        return true;
    };
    let target = target.to_string_lossy().into_owned();
    std::fs::read_to_string("allowed_symlinks.txt").is_ok_and(|rules| rules.lines().filter_map(|rule| rule.trim().strip_prefix("[prefix]")).any(|prefix| target.starts_with(prefix)))
}

/// `DirectoryValidator`: every link under a pack has to be allowed.
fn links_allowed(folder: &std::path::Path) -> bool {
    std::fs::read_dir(folder).is_ok_and(|entries| {
        entries.flatten().all(|entry| {
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_symlink() => symlink_allowed(&path),
                Ok(kind) if kind.is_dir() => links_allowed(&path),
                _ => true,
            }
        })
    })
}

/// A zip's folder as files: `Path.of` a `jar:` URI, which a plugin here has no file system mounted for.
fn unpack(file: &str, inner: &str, to: &std::path::Path) -> Option<()> {
    let zip = horizon_client_render::zip::Zip::open(std::path::Path::new(file)).ok()?;
    let inner = inner.trim_matches('/');
    let prefix = if inner.is_empty() { String::new() } else { format!("{inner}/") };
    let names: Vec<String> = zip.names().filter(|name| name.starts_with(&prefix) && !name.ends_with('/')).map(str::to_owned).collect();
    for name in names {
        let target = to.join(name.strip_prefix(&prefix)?);
        if target.components().any(|part| matches!(part, std::path::Component::ParentDir)) {
            return None;
        }
        std::fs::create_dir_all(target.parent()?).ok()?;
        std::fs::write(target, zip.read(&name).ok()?).ok()?;
    }
    Some(())
}

/// `{ "op": "paper_free_pack_add", ... }`: `PaperDatapackRegistrar.discoverPack`, to be read at the next reload.
fn pack_add(request: &Value) -> Option<Value> {
    use horizon_client::datapack;
    let id = request["id"].as_str()?;
    let mut path = std::path::PathBuf::from(request["path"].as_str()?);
    if let Some(inner) = request["inner"].as_str() {
        let to = std::path::PathBuf::from(request["unpack"].as_str()?);
        unpack(path.to_str()?, inner, &to)?;
        path = to;
    }
    if path.is_symlink() {
        if !symlink_allowed(&path) {
            return None;
        }
        path = std::fs::read_link(&path).ok()?;
    }
    if path.is_dir() && !links_allowed(&path) {
        return None;
    }
    let pack = datapack::DataPack::at(id, &path)?;
    let meta: Value = serde_json::from_slice(&pack.read("pack.mcmeta")?).ok()?;
    meta.get("pack")?;
    let required = request["required"].as_bool().unwrap_or(false);
    let added = datapack::PluginPack { id: id.to_owned(), path, title: request["title"].clone(), required, top: request["top"].as_bool().unwrap_or(true), fixed: request["fixed"].as_bool().unwrap_or(false) };
    let mut packs = datapack::PLUGIN_PACKS.lock().unwrap_or_else(|e| e.into_inner());
    packs.retain(|known| known.id != id);
    packs.push(added);
    drop(packs);
    datapack::HIDDEN_PACKS.lock().unwrap_or_else(|e| e.into_inner()).retain(|known| known != id);
    Some(json!({ "id": id, "enabled": false, "meta": meta, "title": request["title"], "required": required, "plugin": true }))
}

/// What needs no game, so a thread with none can ask: a held connection's own thread does the rest.
pub(in crate::game) fn free_query(request: &str) -> Option<String> {
    use horizon_client::paper_hub as hub;
    if !request.contains("paper_connection_") && !request.contains("paper_free_") {
        return None;
    }
    let request: Value = serde_json::from_str(request).ok()?;
    let answer = match request["op"].as_str()? {
        "paper_free_pack_add" => pack_add(&request).unwrap_or(Value::Null),
        "paper_free_pack_remove" => {
            use horizon_client::datapack;
            let id = request["id"].as_str()?;
            datapack::PLUGIN_PACKS.lock().unwrap_or_else(|e| e.into_inner()).retain(|known| known.id != id);
            datapack::HIDDEN_PACKS.lock().unwrap_or_else(|e| e.into_inner()).push(id.to_owned());
            json!(true)
        }
        "paper_free_build_time" => json!(BUILD_TIME_MS),
        "paper_free_signed_argument" => SIGNED_ARGUMENTS.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|(name, _)| Some(name.as_str()) == request["name"].as_str()).map_or(Value::Null, |(_, row)| row.clone()),
        "paper_connection_answer" => connection_answer(&request),
        "paper_connection_send" => connection_send(&request),
        "paper_connection_open" => json!(request["held"].as_u64().is_some_and(|id| hub::held_open(id as u32))),
        "paper_connection_complete" => {
            hub::release_reconfiguration(u128::from_str_radix(request["uuid"].as_str()?, 16).ok()?);
            json!(true)
        }
        _ => return None,
    };
    Some(answer.to_string())
}

pub(in crate::game) fn ping_client_json(id: u64) -> Value {
    horizon_client::paper_hub::ping_client(id as u32).map_or(Value::Null, |handshake| handshake_json(&handshake))
}

fn handshake_json(handshake: &horizon_client::paper_hub::Handshake) -> Value {
    json!({ "address": handshake.address.to_string(), "protocol": handshake.protocol, "host": format!("{}:{}", handshake.host, handshake.port), "transferred": handshake.transferred })
}

/// One row of a combat tracker, on its way in.
pub(in crate::game) struct CombatRow {
    pub damage_type: String,
    pub damage: f32,
    pub fall_location: Option<&'static str>,
    pub fall_distance: f32,
    pub attacker: Option<(u32, String)>,
}

pub(in crate::game) enum CombatChange {
    Read,
    Reset,
    Set(Vec<CombatRow>),
    Add(CombatRow),
}

static COMBAT_CHANGE: std::sync::Mutex<Option<CombatChange>> = std::sync::Mutex::new(None);
static COMBAT_READ: std::sync::Mutex<Option<Value>> = std::sync::Mutex::new(None);
/// Each guest connection's fight: when it began and ended, and whether it's on. The tracker keeps neither time.
static PLAYER_TICKS: std::sync::Mutex<Vec<(Option<u32>, i64)>> = std::sync::Mutex::new(Vec::new());

/// `Entity.tickCount` of a player, which the game keeps no count of: one for each tick `record_combat` sees them.
pub(in crate::game) fn combat_tick(conn: Option<u32>) {
    let mut ticks = PLAYER_TICKS.lock().unwrap_or_else(|e| e.into_inner());
    match ticks.iter_mut().find(|row| row.0 == conn) {
        Some(row) => row.1 += 1,
        None => ticks.push((conn, 0)),
    }
}

/// A respawned player is a new entity to vanilla, counting from nothing.
pub(in crate::game) fn player_respawned(conn: u32) {
    PLAYER_TICKS.lock().unwrap_or_else(|e| e.into_inner()).retain(|row| row.0 != Some(conn));
}

pub(in crate::game) fn combat_wanted() -> Option<CombatChange> {
    COMBAT_CHANGE.lock().unwrap_or_else(|e| e.into_inner()).take()
}

pub(in crate::game) fn combat_read(entries: Vec<Value>, in_combat: bool, last_damage: i64, duration: i64) {
    *COMBAT_READ.lock().unwrap_or_else(|e| e.into_inner()) = Some(json!({ "entries": entries, "in_combat": in_combat, "last_damage": last_damage, "duration": duration }));
}

const FALL_TYPES: [&str; 8] = ["generic", "ladder", "vines", "weeping_vines", "twisting_vines", "scaffolding", "other_climbable", "water"];

fn combat_row(row: &Value) -> CombatRow {
    CombatRow {
        damage_type: row["damage_type"].as_str().unwrap_or("minecraft:generic").trim_start_matches("minecraft:").to_owned(),
        damage: row["damage"].as_f64().unwrap_or(0.0) as f32,
        fall_location: row["fall_location"].as_str().and_then(|fall| FALL_TYPES.iter().copied().find(|known| *known == fall)),
        fall_distance: row["fall_distance"].as_f64().unwrap_or(0.0) as f32,
        attacker: row["attacker"].as_u64().map(|id| (id as u32, row["attacker_name"].as_str().unwrap_or_default().to_owned())),
    }
}

/// `{ "op": "paper_combat", "uuid": ..., "change": ..., "rows": [...] }`: a player's tracker, read through a call of
/// `record_combat` with no events: its hook sees the private fields.
fn player_combat(game: &mut Game<'_>, request: &Value) -> Value {
    let rows = || request["rows"].as_array().into_iter().flatten().map(combat_row).collect::<Vec<_>>();
    let change = match request["change"].as_str().unwrap_or_default() {
        "reset" => CombatChange::Reset,
        "set" => CombatChange::Set(rows()),
        "add" => rows().pop().map_or(CombatChange::Read, CombatChange::Add),
        _ => CombatChange::Read,
    };
    let Some(conn) = guest_of(game, request).map(|guest| guest.conn) else {
        return Value::Null;
    };
    let mine = game.lan_acting == Some(conn);
    let mut tracker = if mine { Default::default() } else { game.guest_combat.remove(&conn).unwrap_or_default() };
    if !mine {
        std::mem::swap(&mut game.combat, &mut tracker);
    }
    *COMBAT_CHANGE.lock().unwrap_or_else(|e| e.into_inner()) = Some(change);
    game.record_combat(&[]);
    let mut answer = COMBAT_READ.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or(Value::Null);
    if request["death"] == true {
        let me = game.target_name(super::commands::Target::Guest(conn));
        let message = game.combat_death_message_for(me);
        answer["death"] = message.component();
    }
    if !mine {
        std::mem::swap(&mut game.combat, &mut tracker);
        game.guest_combat.insert(conn, tracker);
    }
    // The tracker goes by the world's clock: said in the player's own ticks, as Java's are.
    let ticks = PLAYER_TICKS.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|row| row.0 == Some(conn)).map_or(0, |row| row.1);
    let last_damage = answer["last_damage"].as_i64().unwrap_or(0);
    answer["last_damage"] = json!(if last_damage == 0 { 0 } else { (ticks - (game.world.game_time - last_damage)).max(0) });
    answer
}

/// `DamageSources` by the name `/damage` takes, as the mobs' own kinds have it.
fn mob_damage_kind(name: &str) -> horizon_client::mobs::DamageKind {
    use horizon_client::mobs::DamageKind as K;
    match name.trim_start_matches("minecraft:") {
        "player_attack" => K::PlayerAttack,
        "mace_smash" => K::MaceSmash,
        "spear" => K::Spear,
        "mob_attack" | "mob_attack_no_aggro" => K::MobAttack,
        "arrow" => K::Arrow,
        "thrown" | "trident" | "mob_projectile" => K::Thrown,
        "fireball" | "unattributed_fireball" => K::Fireball,
        "wither_skull" => K::WitherSkull,
        "explosion" | "player_explosion" | "bad_respawn_point" => K::Explosion,
        "fall" | "ender_pearl" | "fly_into_wall" => K::Fall,
        "on_fire" => K::OnFire,
        "in_fire" => K::InFire,
        "hot_floor" => K::HotFloor,
        "campfire" => K::Campfire,
        "lava" => K::Lava,
        "drown" => K::Drown,
        "dry_out" => K::DryOut,
        "magic" | "dragon_breath" => K::Magic,
        "indirect_magic" => K::IndirectMagic,
        "sting" => K::Sting,
        "cactus" => K::Cactus,
        "sweet_berry_bush" => K::SweetBerryBush,
        "freeze" => K::Freeze,
        "in_wall" => K::InWall,
        "thorns" => K::Thorns,
        "cramming" => K::Cramming,
        "falling_anvil" => K::FallingAnvil,
        "falling_block" => K::FallingBlock,
        "falling_stalactite" => K::FallingStalactite,
        "fireworks" => K::Fireworks,
        "generic_kill" => K::GenericKill,
        "lightning_bolt" => K::LightningBolt,
        "out_of_world" => K::OutOfWorld,
        "outside_border" => K::OutsideBorder,
        "sonic_boom" => K::SonicBoom,
        "spit" => K::Spit,
        "stalagmite" => K::Stalagmite,
        "starve" => K::Starve,
        "sulfur_cube_hot" => K::SulfurCubeHot,
        "wind_charge" => K::WindCharge,
        "wither" => K::Wither,
        _ => K::Generic,
    }
}

/// `MobAttack` as the file writes it: `mob_attack`.
fn mob_damage_name(kind: horizon_client::mobs::DamageKind) -> String {
    let mut out = String::from("minecraft:");
    for (at, letter) in format!("{kind:?}").char_indices() {
        if letter.is_ascii_uppercase() && at > 0 {
            out.push('_');
        }
        out.push(letter.to_ascii_lowercase());
    }
    out
}

/// `{ "op": "paper_combat", "id": ..., "dimension": ... }`: a mob's tracker, which is public.
fn mob_combat(game: &mut Game<'_>, request: &Value) -> Value {
    use horizon_client::mobs::{Damage, Who};
    let id = request["id"].as_u64().unwrap_or(0) as u32;
    let dimension = request["dimension"].as_str().unwrap_or("minecraft:overworld").to_owned();
    game.native_in_level(&dimension, |game| {
        let Some(at) = game.mobs.iter().position(|mob| mob.id == id) else {
            return Value::Null;
        };
        let row_of = |row: &Value| {
            let row = combat_row(row);
            horizon_client::mobs::combat::Entry {
                source: Damage { kind: mob_damage_kind(&row.damage_type), attacker: row.attacker.map(|(id, _)| Who::Mob(id)), from: None },
                damage: row.damage,
                in_water: row.fall_location == Some("water"),
                fall_distance: row.fall_distance,
            }
        };
        match request["change"].as_str().unwrap_or_default() {
            "reset" => {
                let tick = game.mobs[at].age;
                game.mobs[at].combat.reset(tick);
            }
            "set" => game.mobs[at].combat.entries = request["rows"].as_array().into_iter().flatten().map(row_of).collect(),
            "add" => {
                if let Some(row) = request["rows"].get(0) {
                    let entry = row_of(row);
                    let tick = game.mobs[at].age;
                    game.mobs[at].combat.record(entry, tick);
                }
            }
            _ => {}
        }
        let mob = game.mobs[at].clone();
        let entries: Vec<Value> = mob
            .combat
            .entries
            .iter()
            .map(|entry| {
                let attacker = match entry.source.attacker {
                    Some(Who::Mob(id)) => Some(id),
                    _ => None,
                };
                json!({ "damage_type": mob_damage_name(entry.source.kind), "damage": entry.damage, "fall_location": entry.in_water.then_some("water"), "fall_distance": entry.fall_distance, "attacker": attacker })
            })
            .collect();
        let mut answer = json!({ "entries": entries, "in_combat": mob.combat.in_combat, "last_damage": mob.combat.last_damage_time(), "duration": mob.combat.duration(mob.age) });
        if request["death"] == true {
            let message = game.mob_death_message(&mob, None);
            answer["death"] = message.component();
        }
        answer
    })
    .unwrap_or(Value::Null)
}

/// What Paper's `PaperLootableInventoryData` keeps: per container, when it was filled and who has looted it.
fn loot_file(game: &Game<'_>) -> std::path::PathBuf {
    game.world_root.join("paper-lootables.json")
}

fn loot_data(game: &Game<'_>) -> Value {
    std::fs::read(loot_file(game)).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_else(|| json!({}))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |gone| gone.as_millis() as u64)
}

fn note_loot(game: &mut Game<'_>, key: String) {
    let mut data = loot_data(game);
    let now = now_ms();
    let opener = game.lan_acting.is_some().then(|| format!("{:032x}", game.native_acting_id()));
    data[&key]["filled"] = json!(now);
    if let Some(opener) = opener {
        data[&key]["looted"][&opener] = json!(now);
    }
    let _ = std::fs::write(loot_file(game), data.to_string());
}

/// The head of `unpack_opened_loot`: a container someone opens that still has a table.
pub(in crate::game) fn loot_opened(game: &mut Game<'_>, key: (i32, i32, i32)) {
    let held = game.world.chunks.get(&(key.0 >> 4, key.2 >> 4)).and_then(|chunk| chunk.saved.as_ref()).is_some_and(|saved| saved.rolled_loot.iter().any(|rolled| rolled.pos == key));
    if held {
        let name = format!("{}/{}/{}/{}", game.dim.id().trim_start_matches("minecraft:"), key.0, key.1, key.2);
        note_loot(game, name);
    }
}

pub(in crate::game) fn cart_loot_opened(game: &mut Game<'_>, id: u32) {
    let uuid = game.uuid_for(id).iter().fold(0_u128, |all, word| (all << 32) | u128::from(*word as u32));
    note_loot(game, format!("{uuid:032x}"));
}

/// `{ "op": "paper_lootable", "key": ..., "write": { ... } }`: one container's record.
fn lootable(game: &Game<'_>, request: &Value) -> Value {
    let key = request["key"].as_str().unwrap_or_default();
    let mut data = loot_data(game);
    if let Some(looted) = request.get("looted") {
        let player = looted["player"].as_str().unwrap_or_default();
        if looted["on"] == true {
            data[key]["looted"][player] = json!(now_ms());
        } else if let Some(all) = data[key]["looted"].as_object_mut() {
            all.remove(player);
        }
        let _ = std::fs::write(loot_file(game), data.to_string());
    }
    if let Some(next) = request.get("next") {
        data[key]["next"] = next.clone();
        let _ = std::fs::write(loot_file(game), data.to_string());
    }
    data[key].take()
}

static NO_LOAD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(in crate::game) fn take_no_load() -> bool {
    NO_LOAD.swap(false, std::sync::atomic::Ordering::Relaxed)
}

/// `{ "op": "paper_item_test", "predicate": ..., "stack": ... }`: `ItemPredicateArgument`, the game's own test.
fn item_test(game: &mut Game<'_>, request: &Value) -> Value {
    let text = request["predicate"].as_str().unwrap_or_default();
    let mut reader = horizon_client::command::Reader::new(text);
    let Ok(predicate) = horizon_client::command::item::parse_predicate(&mut reader, &game.ui.registries) else {
        return json!(false);
    };
    if request.get("stack").is_none() {
        return json!(true);
    }
    json!(stack_from(&request["stack"]).is_some_and(|stack| game.stack_matches(&predicate, &stack)))
}

/// `FlowingFluid.getFlow` for a block, with what the game's tables know of its neighbours.
struct FlowView<'a, 'g> {
    game: &'a Game<'g>,
}

impl horizon_world::fluid_flow::FluidFlowWorld for FlowView<'_, '_> {
    type Error = std::io::Error;
    type Block = horizon_world::registry::BlockStateId;
    fn fluid(&mut self, p: horizon_core::position::BlockPos) -> std::io::Result<horizon_world::fluid::FluidState> {
        let state = self.game.world.block(p.x, p.y, p.z);
        horizon_world::fluid::FluidState::from_block(self.game.pack.assets.registry.state(state).ok_or_else(|| std::io::Error::other("no such state"))?)
    }
    fn block(&mut self, p: horizon_core::position::BlockPos) -> std::io::Result<Self::Block> {
        Ok(self.game.world.block(p.x, p.y, p.z))
    }
    fn blocks_fluid_flow(&self, s: Self::Block) -> bool {
        self.game.pack.assets.registry.state(s).is_some_and(|state| self.game.pack.assets.blocks_fluid_flow.contains(&state.name))
    }
    fn is_ice_class(&self, s: Self::Block) -> bool {
        self.game.res.facts().get(s).classes.iter().any(|class| class.as_ref() == "IceBlock")
    }
    fn is_face_sturdy(&mut self, s: Self::Block, _: horizon_core::position::BlockPos, d: horizon_core::geometry::Direction) -> std::io::Result<bool> {
        Ok(self.game.res.facts().get(s).sturdy_full & (1 << d as u8) != 0)
    }
}

/// `{ "op": "paper_fluid_flow", "dimension": ..., "pos": [x, y, z] }`.
fn fluid_flow(game: &mut Game<'_>, request: &Value) -> Value {
    let at = triple(&request["pos"]).map(|value| value as i32);
    let dimension = request["dimension"].as_str().unwrap_or("minecraft:overworld").to_owned();
    game.native_in_level(&dimension, |game| {
        let position = horizon_core::position::BlockPos::new(at[0], at[1], at[2]);
        let mut view = FlowView { game };
        use horizon_world::fluid_flow::FluidFlowWorld as _;
        let flow = view.fluid(position).and_then(|state| horizon_world::fluid_flow::flow_at(&mut view, state, position));
        flow.map_or(Value::Null, |flow| json!([flow.x, flow.y, flow.z]))
    })
    .unwrap_or(Value::Null)
}

/// `{ "op": "paper_set_pack", "name": ..., "enabled": ... }`: `/datapack enable|disable`, at once.
fn set_pack(game: &mut Game<'_>, request: &Value) -> Value {
    let name = request["name"].as_str().unwrap_or_default();
    let enabled = request["enabled"].as_bool().unwrap_or(false);
    if name == "vanilla" {
        game.cmds.vanilla_disabled = !enabled;
    } else {
        let file = game.world_root.join("datapacks.json");
        let mut saved: Value = std::fs::read(&file).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_else(|| json!({ "enabled": [], "disabled": [] }));
        let list = |value: &Value| value.as_array().cloned().unwrap_or_default();
        let (mut on, mut off) = (list(&saved["enabled"]), list(&saved["disabled"]));
        on.retain(|known| known != name);
        off.retain(|known| known != name);
        if enabled {
            on.push(json!(name));
        } else {
            off.push(json!(name));
        }
        saved["enabled"] = json!(on);
        saved["disabled"] = json!(off);
        if std::fs::write(file, saved.to_string()).is_err() {
            return json!(false);
        }
    }
    game.reload_datapacks(true);
    json!(true)
}

/// `{ "op": "paper_resolve", "component": ..., "player": uuid or null, "entity": id or null }`:
/// `ComponentUtils.updateForEntity`, as a command's text is resolved, from where the context stands.
fn resolve_component(game: &mut Game<'_>, request: &Value) -> Value {
    use horizon_client::command::{Context, ParsedArg, args::Arg};
    use super::commands::Target as T;
    let json_text = request["component"].to_string();
    let context = Context { root: 0, nodes: Vec::new(), args: vec![ParsedArg { name: "message".to_owned(), value: Arg::Str(json_text), range: (0, 0) }], command: None, range: (0, 0), child: None, literals: Vec::new() };
    let saved = game.command_source;
    let mut source = game.server_source();
    if let Some(conn) = guest_of(game, request).map(|guest| guest.conn) {
        source.entity = Some(T::Guest(conn));
        if let Some(at) = game.target_position(T::Guest(conn)) {
            source.pos = at;
        }
        source.rot = game.target_rotation(T::Guest(conn));
    }
    // `scoreboardSubject`: who `*` in a score means.
    if let Some(id) = request["subject"].as_u64() {
        source.entity = Some(T::Mob(id as u32));
    } else if let Some(uuid) = request["subject_player"].as_str() {
        let guest = guest_of(game, &json!({ "uuid": uuid })).map(|guest| guest.conn);
        source.entity = guest.map(T::Guest).or(source.entity);
    }
    game.command_source = Some(source);
    let text = game.component_arg(&context, "message");
    game.command_source = saved;
    text.ok().map_or(Value::Null, |text| text.component())
}

/// `{ "op": "paper_english", "component": ... }`: a component read out, translations and all.
fn english(game: &Game<'_>, request: &Value) -> Value {
    let Some(host) = game.lan.as_ref() else {
        return Value::Null;
    };
    json!(horizon_client::paper_hub::english(&host.hub, &component_tag(&request["component"])))
}

/// `{ "op": "paper_nbt_kinds", "bytes": ... }`: `tag_kinds` of what `NbtIo.read` gives.
fn nbt_kinds(request: &Value) -> Value {
    let bytes: Vec<u8> = request["bytes"].as_array().into_iter().flatten().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect();
    horizon_world::nbt::read_named(&bytes, horizon_world::nbt::Limits::default()).ok().map_or(Value::Null, |(named, _)| tag_kinds(&named.tag))
}

/// What each tag is, as `persistence::typed_snbt` in the SDK reads it: a letter, or the same shape as a list or compound.
pub(in crate::game) fn tag_kinds(tag: &horizon_world::nbt::Tag) -> Value {
    use horizon_world::nbt::Tag;
    match tag {
        Tag::End => Value::Null,
        Tag::Byte(_) => json!("b"),
        Tag::Short(_) => json!("s"),
        Tag::Int(_) => json!("i"),
        Tag::Long(_) => json!("l"),
        Tag::Float(_) => json!("f"),
        Tag::Double(_) => json!("d"),
        Tag::String(_) => json!("t"),
        Tag::ByteArray(_) => json!("B"),
        Tag::IntArray(_) => json!("I"),
        Tag::LongArray(_) => json!("L"),
        Tag::List(values) => Value::Array(values.iter().map(tag_kinds).collect()),
        Tag::Compound(values) => Value::Object(values.iter().map(|(key, value)| (key.to_string_lossy(), tag_kinds(value))).collect()),
    }
}

/// `{ "op": "paper_nbt_bytes", "snbt": ... }`: `NbtIo.write` of a compound.
fn nbt_bytes(request: &Value) -> Value {
    let Ok((horizon_world::nbt::Tag::Compound(compound), _)) = horizon_world::nbt::parse_snbt(request["snbt"].as_str().unwrap_or_default()) else {
        return Value::Null;
    };
    let root = horizon_world::nbt::NamedTag { name: "".into(), tag: horizon_world::nbt::Tag::Compound(compound) };
    horizon_world::nbt::write_named(&root, horizon_world::nbt::Limits::default()).map_or(Value::Null, |bytes| json!(bytes))
}

/// `None` for an op that isn't this family's.
pub(in crate::game) fn query(game: &mut Game<'_>, op: &str, request: &Value) -> Option<Value> {
    match op {
        "paper_particle" => Some(particle(game, request)),
        "paper_registry" => Some(registry(game, request)),
        "paper_snbt" => Some(snbt(request)),
        "paper_profile_lookup" => Some(profile_lookup(game, request)),
        "paper_offline_uuid" => Some(json!(horizon_protocol::crypto::uuid_string(horizon_protocol::crypto::offline_uuid(request["name"].as_str().unwrap_or_default())))),
        "paper_command_source" => Some(command_source(game)),
        "paper_select" => Some(select(game, request)),
        "paper_datapacks" => Some(datapacks(game)),
        "paper_features" => Some(json!(game.level.enabled_features)),
        "paper_datapacks_refresh" => {
            game.reload_datapacks(false);
            NO_LOAD.store(true, std::sync::atomic::Ordering::Relaxed);
            game.ensure_datapacks();
            Some(json!(true))
        }
        "paper_address" => Some(guest_of(game, request).map_or(Value::Null, |guest| horizon_client::paper_hub::handshake_of(guest.address).map_or_else(|| json!({ "address": guest.address.to_string() }), |handshake| handshake_json(&handshake)))),
        "paper_connection_asks" => Some(connection_asks(game)),
        "paper_restrict" => {
            horizon_client::paper_hub::restrict_command(request["path"].as_array().into_iter().flatten().filter_map(|word| word.as_str().map(str::to_owned)).collect());
            if let Some(host) = game.lan.as_mut() {
                host.hub.resend_commands();
            }
            Some(json!(true))
        }
        "paper_combat" => Some(if request.get("uuid").is_some() { player_combat(game, request) } else { mob_combat(game, request) }),
        "paper_lootable" => Some(lootable(game, request)),
        "paper_item_test" => Some(item_test(game, request)),
        "paper_fluid_flow" => Some(fluid_flow(game, request)),
        "paper_set_pack" => Some(set_pack(game, request)),
        "paper_resolve" => Some(resolve_component(game, request)),
        "paper_english" => Some(english(game, request)),
        "paper_nbt_bytes" => Some(nbt_bytes(request)),
        "paper_nbt_kinds" => Some(nbt_kinds(request)),
        "paper_ping_client" => Some(request["id"].as_u64().and_then(|id| horizon_client::paper_hub::ping_client(id as u32)).map_or(Value::Null, |handshake| handshake_json(&handshake))),
        "paper_disconnect" => Some(json!(guest_of(game, request).map(|guest| guest.sender.disconnect(component_tag(&request["reason"]))).is_some())),
        "paper_links" => Some(links(game, request)),
        "paper_report_details" => Some(report_details(game, request)),
        "paper_cookie_request" => Some(json!(guest_of(game, request).map(|guest| guest.send(&C::CookieRequest(horizon_protocol::packets::configuration::CookieRequest { key: request["key"].as_str().unwrap_or_default().to_owned() }))).is_some())),
        _ => None,
    }
}
