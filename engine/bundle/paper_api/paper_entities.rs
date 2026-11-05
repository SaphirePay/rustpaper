// A plugin's way to any entity or block entity: its data as the game saves it, the same tree `/data`
// reads and writes. Bukkit's hundreds of `Zombie.isBaby` and `Sign.getLine` are one table of paths in
// the SDK over these few functions. A child of `commands`, as a target is private to it.
use super::{Target, data::DataRef};
use crate::game::Game;
use horizon_client::data_cmd::merged;
use horizon_world::nbt::{Tag, parse_snbt};
use serde_json::{Map, Value, json};

/// A tag as JSON. Numbers lose their width, which no reader of these needs.
pub(in crate::game) fn tag_json(tag: &Tag) -> Value {
    match tag {
        Tag::End => Value::Null,
        Tag::Byte(value) => json!(value),
        Tag::Short(value) => json!(value),
        Tag::Int(value) => json!(value),
        Tag::Long(value) => json!(value),
        Tag::Float(bits) => json!(f32::from_bits(*bits)),
        Tag::Double(bits) => json!(f64::from_bits(*bits)),
        Tag::ByteArray(values) => json!(values),
        Tag::String(value) => json!(value.to_string_lossy()),
        Tag::List(values) => Value::Array(values.iter().map(tag_json).collect()),
        Tag::Compound(values) => Value::Object(values.iter().map(|(key, value)| (key.to_string_lossy(), tag_json(value))).collect::<Map<_, _>>()),
        Tag::IntArray(values) => json!(values),
        Tag::LongArray(values) => json!(values),
    }
}

fn target_id(target: Target) -> Option<u32> {
    super::non_player_entity_id(target).and_then(|id| u32::try_from(id).ok())
}

fn same_level(here: &str, dimension: &str) -> bool {
    here.trim_start_matches("minecraft:") == dimension.trim_start_matches("minecraft:")
}

impl Game<'_> {
    /// Runs `f` in a level, by its name; `None` if there's no such level. One of the game's three that
    /// nobody has been to opens now: in Java they're all there from the start.
    pub(in crate::game) fn native_in_level<R>(&mut self, dimension: &str, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        if same_level(self.dim.id(), dimension) {
            return Some(f(self));
        }
        let parked = self.levels.parked_dims().into_iter().find(|dim| same_level(dim.id(), dimension));
        let dim = parked.or_else(|| {
            let name = dimension.trim_start_matches("minecraft:");
            (matches!(name, "overworld" | "the_nether" | "the_end") && !crate::game::paper_fam_core::level_off(name)).then(|| crate::game::Dim::from_id(dimension))
        })?;
        let back = self.enter_command_level(dim);
        let answer = f(self);
        if let Some(back) = back {
            self.enter_command_level(back);
        }
        Some(answer)
    }

    /// The entity a request names, by `uuid` or by `id` in any loaded level.
    fn native_target(&mut self, request: &Value) -> Option<Target> {
        if let Some(uuid) = request["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok()) {
            return self.target_by_uuid(uuid);
        }
        let id = i32::try_from(request["id"].as_i64()?).ok()?;
        if let Some(target) = self.level_entity(id) {
            return Some(target);
        }
        for dim in self.levels.parked_dims() {
            let back = self.enter_command_level(dim);
            let found = self.level_entity(id);
            if let Some(back) = back {
                self.enter_command_level(back);
            }
            if found.is_some() {
                return found;
            }
        }
        None
    }

    fn native_entity_json(&mut self, target: Target) -> Value {
        let at = self.target_position(target).unwrap_or_default();
        let [yaw, pitch] = self.target_rotation(target);
        let mut value = json!({ "kind": self.target_type(target), "dimension": format!("minecraft:{}", self.dim.id().trim_start_matches("minecraft:")), "pos": at, "yaw": yaw, "pitch": pitch });
        if let Some(id) = target_id(target) {
            let uuid = self.uuid_for(id).iter().fold(0_u128, |all, word| (all << 32) | u128::from(*word as u32));
            value["id"] = json!(id);
            value["uuid"] = json!(format!("{uuid:032x}"));
        }
        value
    }

    /// Everything in a level but its players.
    pub(in crate::game) fn native_entities(&mut self, dimension: &str) -> Value {
        self.native_in_level(dimension, |game| {
            let targets = game.level_candidates();
            Value::Array(targets.into_iter().map(|target| game.native_entity_json(target)).collect())
        })
        .unwrap_or(Value::Null)
    }

    /// One entity: where it is, and `data` for the tree it saves.
    pub(in crate::game) fn native_entity(&mut self, request: &Value) -> Value {
        let Some(target) = self.native_target(request) else {
            return Value::Null;
        };
        let wants_data = request["data"].as_bool().unwrap_or(true);
        self.in_target_level(target, |game| {
            let mut value = game.native_entity_json(target);
            if wants_data {
                value["data"] = game.data_of(&DataRef::Entity(target)).map_or(Value::Null, |tag| tag_json(&tag));
            }
            value
        })
        .unwrap_or(Value::Null)
    }

    /// `{ "op": "mob_watch" }`: the saved tree of each living entity a listener wants, by kind. A
    /// `kind` of "" is every mob, so it costs a save of each: only what's asked for is read.
    pub(in crate::game) fn native_mob_watch(&mut self, request: &Value) -> Value {
        let dimension = request["dimension"].as_str().unwrap_or("minecraft:overworld");
        let kinds: Vec<String> = request["wanted"].as_array().into_iter().flatten().filter_map(|row| row["kind"].as_str().map(str::to_owned)).collect();
        self.native_in_level(dimension, |game| {
            let every = kinds.iter().any(String::is_empty);
            let rows: Vec<(u32, String)> = game.mobs.iter().map(|mob| (mob.id, mob.kind.id.clone())).filter(|(_, kind)| every || kinds.contains(kind)).collect();
            let mut out = Vec::with_capacity(rows.len());
            for (id, kind) in rows {
                let Some(target) = i32::try_from(id).ok().and_then(|id| game.level_entity(id)) else {
                    continue;
                };
                let data = game.data_of(&DataRef::Entity(target)).map_or(Value::Null, |tag| tag_json(&tag));
                out.push(json!({ "id": id, "kind": kind, "data": data }));
            }
            Value::Array(out)
        })
        .unwrap_or(Value::Null)
    }

    /// `/data merge entity`.
    pub(in crate::game) fn native_entity_merge(&mut self, request: &Value) -> bool {
        let (Some(target), Some(Ok((patch, _)))) = (self.native_target(request), request["snbt"].as_str().map(parse_snbt)) else {
            return false;
        };
        self.in_target_level(target, |game| {
            let reference = DataRef::Entity(target);
            game.data_of(&reference).is_ok_and(|old| game.set_data_of(&reference, merged(&old, &patch)).is_ok())
        })
        .unwrap_or(false)
    }

    /// `/summon`, with the entity to show for it: its data has `id` and `Pos`.
    pub(in crate::game) fn native_spawn(&mut self, dimension: &str, snbt: &str) -> Value {
        let Ok((tag, _)) = parse_snbt(snbt) else {
            return Value::Null;
        };
        self.native_in_level(dimension, |game| game.spawn_entity_from_nbt(tag).map_or(Value::Null, |target| game.native_entity_json(target))).unwrap_or(Value::Null)
    }

    /// A block entity's data.
    pub(in crate::game) fn native_block_data(&mut self, dimension: &str, pos: [i32; 3]) -> Value {
        self.native_in_level(dimension, |game| game.data_of(&DataRef::Block(pos)).map_or(Value::Null, |tag| tag_json(&tag))).unwrap_or(Value::Null)
    }

    /// `/data merge block`.
    pub(in crate::game) fn native_block_merge(&mut self, dimension: &str, pos: [i32; 3], snbt: &str) -> bool {
        let Ok((patch, _)) = parse_snbt(snbt) else {
            return false;
        };
        self.native_in_level(dimension, |game| {
            let reference = DataRef::Block(pos);
            game.data_of(&reference).is_ok_and(|old| game.set_data_of(&reference, merged(&old, &patch)).is_ok())
        })
        .unwrap_or(false)
    }

    /// A value in named storage, the same tree `/data storage` has: what `get_data` of a world holds.
    pub(in crate::game) fn native_storage(&mut self, id: &str) -> Value {
        self.data_of(&DataRef::Storage(id.to_owned())).map_or(Value::Null, |tag| tag_json(&tag))
    }
}
