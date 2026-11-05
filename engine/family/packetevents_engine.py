# PacketEvents on Horizon (owner 11-03: "add in that plugin so the plugins that need it work"). Two engine
# pieces beside ProtocolLib's (protocol_engine.py): an entity's data entries as values a plugin reads and
# writes (a text display a plugin spawns for one player: its text, scale, background), and the play packets a
# player sends passing the plugins that listen to them (PacketEvents' PacketReceiveEvent), which may cancel
# them or change their fields.
EDITS = [
    # Entity data entries as `[index, serializer, value]`: numbers, text, booleans, vectors, positions, UUIDs and
    # text components (as their JSON). An item, particle or profile reads as null and isn't written.
    ('horizon-protocol/src/entity_data.rs', """pub mod serializer {""", """impl Default for Value {
    fn default() -> Self {
        Value::Byte(0)
    }
}

fn tag_json(tag: &Tag) -> serde_json::Value {
    match tag {
        Tag::End => serde_json::Value::Null,
        Tag::Byte(v) => serde_json::json!(v),
        Tag::Short(v) => serde_json::json!(v),
        Tag::Int(v) => serde_json::json!(v),
        Tag::Long(v) => serde_json::json!(v),
        Tag::Float(bits) => serde_json::json!(f32::from_bits(*bits)),
        Tag::Double(bits) => serde_json::json!(f64::from_bits(*bits)),
        Tag::String(s) => serde_json::json!(s.to_string_lossy()),
        Tag::List(items) => serde_json::Value::Array(items.iter().map(tag_json).collect()),
        Tag::Compound(map) => serde_json::Value::Object(map.iter().map(|(k, v)| (k.to_string_lossy(), tag_json(v))).collect()),
        Tag::ByteArray(v) => serde_json::json!(v),
        Tag::IntArray(v) => serde_json::json!(v),
        Tag::LongArray(v) => serde_json::json!(v),
    }
}

/// A text component's JSON as the NBT the client reads (`{"text": "hi"}`, or a plain string).
fn json_tag(value: &serde_json::Value) -> Tag {
    match value {
        serde_json::Value::Null => Tag::compound(Vec::<(&str, Tag)>::new()),
        serde_json::Value::Bool(b) => Tag::Byte(i8::from(*b)),
        serde_json::Value::Number(n) => match n.as_i64() {
            Some(i) => i32::try_from(i).map_or(Tag::Long(i), Tag::Int),
            None => Tag::Double(n.as_f64().unwrap_or(0.0).to_bits()),
        },
        serde_json::Value::String(s) => Tag::String(horizon_world::nbt::JavaString::from(s.as_str())),
        serde_json::Value::Array(items) => Tag::List(items.iter().map(json_tag).collect()),
        serde_json::Value::Object(map) => Tag::compound(map.iter().map(|(k, v)| (k.as_str(), json_tag(v)))),
    }
}

fn value_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Byte(v) => serde_json::json!(v),
        Value::Int(v) | Value::Id(v) => serde_json::json!(v),
        Value::Long(v) => serde_json::json!(v),
        Value::Float(v) => serde_json::json!(v),
        Value::String(v) => serde_json::json!(v),
        Value::Text(tag) => tag_json(tag),
        Value::OptText(tag) => tag.as_ref().map_or(serde_json::Value::Null, tag_json),
        Value::Bool(v) => serde_json::json!(v),
        Value::Floats(v) => serde_json::json!(v),
        Value::Ids(v) => serde_json::json!(v),
        Value::Pos(p) => serde_json::json!(p),
        Value::OptPos(p) => p.map_or(serde_json::Value::Null, |p| serde_json::json!(p)),
        Value::OptUuid(u) => u.map_or(serde_json::Value::Null, |u| serde_json::json!(format!("{u:032x}"))),
        _ => serde_json::Value::Null,
    }
}

/// The value a serializer carries, from its JSON; None for what a plugin can't write here.
fn json_value(kind: i32, value: &serde_json::Value) -> Option<Value> {
    use serializer::*;
    let floats = || value.as_array().map(|items| items.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect::<Vec<f32>>());
    let pos = || value.as_array().filter(|items| items.len() == 3).map(|items| [0, 1, 2].map(|i| items[i].as_i64().unwrap_or(0) as i32));
    Some(match kind {
        BYTE => Value::Byte(value.as_i64().or_else(|| value.as_bool().map(i64::from))? as i8),
        INT => Value::Int(value.as_i64()? as i32),
        LONG => Value::Long(value.as_i64()?),
        FLOAT => Value::Float(value.as_f64()? as f32),
        STRING => Value::String(value.as_str()?.to_owned()),
        COMPONENT => Value::Text(json_tag(value)),
        OPTIONAL_COMPONENT => Value::OptText((!value.is_null()).then(|| json_tag(value))),
        BOOLEAN => Value::Bool(value.as_bool()?),
        ROTATIONS | VECTOR3 | QUATERNION => Value::Floats(floats()?),
        BLOCK_POS => Value::Pos(pos()?),
        OPTIONAL_BLOCK_POS => Value::OptPos(if value.is_null() { None } else { Some(pos()?) }),
        OPTIONAL_LIVING_ENTITY_REFERENCE => Value::OptUuid(value.as_str().and_then(|v| u128::from_str_radix(&v.replace('-', ""), 16).ok())),
        VILLAGER_DATA => Value::Ids(value.as_array()?.iter().map(|v| v.as_i64().unwrap_or(0) as i32).collect()),
        DIRECTION | BLOCK_STATE | OPTIONAL_BLOCK_STATE | OPTIONAL_UNSIGNED_INT | POSE | CAT_VARIANT..=ZOMBIE_NAUTILUS_VARIANT | PAINTING_VARIANT..=WEATHERING_COPPER_STATE | HUMANOID_ARM | DYE_COLOR => Value::Id(value.as_i64()? as i32),
        _ => return None,
    })
}

impl crate::buf::FieldValue for (u8, i32, Value) {
    const KIND: &'static str = "entity_data";
    fn field_json(&self) -> serde_json::Value {
        serde_json::json!([self.0, self.1, value_json(&self.2)])
    }
    fn field_set(&mut self, value: &serde_json::Value) -> bool {
        let Some(entry) = value.as_array().filter(|e| e.len() == 3) else {
            return false;
        };
        let (Some(index), Some(kind)) = (entry[0].as_u64(), entry[1].as_i64()) else {
            return false;
        };
        match json_value(kind as i32, &entry[2]) {
            Some(v) => {
                *self = (index as u8, kind as i32, v);
                true
            }
            None => false,
        }
    }
}

pub mod serializer {"""),
    # The play packets a player sends pass the plugins listening to them (as `in:<name>`), on the game thread,
    # before the server handles them: cancelled ones go no further, changed ones go on changed.
    ('horizon-client/src/lan/hub.rs', """                    if !g.left_reason.is_empty() {
                        continue;
                    }
                    g.inbox.push_back((*packet, bytes));""", """                    if !g.left_reason.is_empty() {
                        continue;
                    }
                    let Some(packet) = heard_from_player(g.uuid, &g.name, *packet) else {
                        continue;
                    };
                    g.inbox.push_back((packet, bytes));"""),
    ('horizon-client/src/lan/hub.rs', """/// The packets of one player handled in a tick; the rest wait for the next.""", """/// A packet from a player: the plugins that listen to its kind (`in:<name>`) read it, change it or
/// cancel it (None).
fn heard_from_player(uuid: u128, name: &str, mut packet: horizon_protocol::packets::play::Serverbound) -> Option<horizon_protocol::packets::play::Serverbound> {
    if !horizon_mod_runtime::any_packet_listened() || !horizon_mod_runtime::packet_listened(&format!("in:{}", packet.name())) {
        return Some(packet);
    }
    let fields: Vec<serde_json::Value> = packet.fields().into_iter().map(|(field, kind, value)| serde_json::json!([field, kind, value])).collect();
    let payload = serde_json::json!({ "uuid": format!("{uuid:032x}"), "player": name, "packet": packet.name(), "fields": fields });
    horizon_mod_runtime::begin_packet();
    let code = horizon_mod_runtime::fire("pl_receive", &payload.to_string());
    let edits = horizon_mod_runtime::take_packet_edits();
    if code == 2 {
        return None;
    }
    for (field, value) in &edits {
        packet.set_field(field, value);
    }
    Some(packet)
}

/// The packets of one player handled in a tick; the rest wait for the next."""),
]
