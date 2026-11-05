//! `org.bukkit.entity.Entity` for what isn't a player, `org.bukkit.Chunk`, and what sits beside them.
use crate::{Location, World};
use horizon_mod::{Value, query, run_command};
use serde_json::json;

/// What every class of entity is: `entity.cast::<Zombie>()` is Java's `(Zombie) entity`.
pub trait EntityClass {
    fn of(entity: Entity) -> Self;
}

/// bukkit: org/bukkit/entity/Entity, org/bukkit/entity/LivingEntity, org/bukkit/entity/Mob, org/bukkit/entity/Damageable
///
/// Anything in a level but a player, by its id; a mob's calls are here too.
#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    id: u64,
    dimension: String,
    uuid: String,
}

impl Entity {
    pub(crate) fn read(value: &Value) -> Option<Self> {
        Some(Self { id: value["id"].as_u64()?, dimension: value["dimension"].as_str().unwrap_or("minecraft:overworld").to_owned(), uuid: value["uuid"].as_str().unwrap_or_default().to_owned() })
    }
    /// An entity by the id an event gave.
    pub fn of(id: u64, world: &World) -> Self {
        let uuid = query(json!({ "op": "entity", "id": id, "data": false }))["uuid"].as_str().unwrap_or_default().to_owned();
        Self { id, dimension: world.dimension().to_owned(), uuid }
    }
    /// Another view of the same entity, as a Java cast.
    pub fn cast<C: EntityClass>(&self) -> C {
        C::of(self.clone())
    }
    fn state(&self) -> Value {
        Some(query(json!({ "op": "entity", "id": self.id, "data": true }))).filter(|state| !state.is_null()).unwrap_or_else(|| query(json!({ "op": "entity_kept", "id": self.id, "dimension": self.dimension })))
    }
    fn mob_state(&self) -> Value {
        query(json!({ "op": "mob", "dimension": self.dimension, "id": self.id }))
    }
    /// The tree the game saves for this entity.
    pub fn get_data(&self) -> Value {
        self.state()["data"].take()
    }
    /// One path of `get_data`.
    pub fn data_at(&self, path: &str) -> Value {
        crate::data::at(&self.get_data(), path).clone()
    }
    /// `/data merge entity`: at once, unlike a command in a player's event.
    pub fn merge(&self, snbt: &str) -> bool {
        query(json!({ "op": "entity_merge", "id": self.id, "snbt": snbt })).as_bool().unwrap_or(false) || query(json!({ "op": "entity_merge_kept", "id": self.id, "dimension": self.dimension, "snbt": snbt })).as_bool().unwrap_or(false)
    }
    pub(crate) fn location_of(&self, value: &Value) -> Option<Location> {
        let at = |i: usize| value[i].as_f64();
        Some(Location::new(self.get_world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    fn act(&self, action: &str, value: Value) -> bool {
        query(json!({ "op": "mob_action", "action": action, "dimension": self.dimension, "id": self.id, "value": value })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Entity.getEntityId
    pub fn get_entity_id(&self) -> u64 {
        self.id
    }
    /// bukkit: org/bukkit/entity/Entity.getUniqueId
    pub fn get_unique_id(&self) -> String {
        crate::dashed(&self.uuid)
    }
    /// bukkit: org/bukkit/entity/Entity.getNearbyEntities
    ///
    /// In a box `x y z` each way around it, itself left out, as Bukkit's.
    pub fn get_nearby_entities_around(&self, x: f64, y: f64, z: f64) -> Vec<Entity> {
        let Some(at) = self.get_location() else { return Vec::new() };
        let me = self.get_unique_id();
        at.get_nearby_entities(x, y, z).into_iter().filter(|other| other.get_unique_id() != me).collect()
    }
    /// bukkit: org/bukkit/entity/Entity.getType
    ///
    /// The game's id: `minecraft:zombie`.
    pub fn get_type(&self) -> String {
        self.state()["kind"].as_str().unwrap_or_default().to_owned()
    }
    /// bukkit: org/bukkit/entity/Entity.isValid
    pub fn is_valid(&self) -> bool {
        !self.state().is_null()
    }
    /// bukkit: org/bukkit/entity/Entity.isDead, org/bukkit/entity/LivingEntity.isDead
    pub fn is_dead(&self) -> bool {
        self.state().is_null()
    }
    /// bukkit: org/bukkit/entity/Entity.getWorld
    pub fn get_world(&self) -> World {
        World::of(&self.dimension)
    }
    /// bukkit: org/bukkit/entity/Entity.getLocation
    pub fn get_location(&self) -> Option<Location> {
        let state = self.state();
        let at = |i: usize| state["pos"][i].as_f64();
        Some(Location::new(self.get_world(), at(0)?, at(1)?, at(2)?, state["yaw"].as_f64().unwrap_or(0.0) as f32, state["pitch"].as_f64().unwrap_or(0.0) as f32))
    }
    /// bukkit: org/bukkit/entity/Entity.getX
    pub fn get_x(&self) -> f64 {
        self.get_location().map_or(0.0, |at| at.x)
    }
    /// bukkit: org/bukkit/entity/Entity.getY
    pub fn get_y(&self) -> f64 {
        self.get_location().map_or(0.0, |at| at.y)
    }
    /// bukkit: org/bukkit/entity/Entity.getZ
    pub fn get_z(&self) -> f64 {
        self.get_location().map_or(0.0, |at| at.z)
    }
    /// bukkit: org/bukkit/entity/Entity.teleport, org/bukkit/entity/Entity.teleportAsync
    pub fn teleport(&self, to: &Location) -> bool {
        if self.act("teleport", json!([to.x, to.y, to.z])) {
            return true;
        }
        self.merge(&format!("{{Pos:[{:?}d,{:?}d,{:?}d],Rotation:[{:?}f,{:?}f]}}", to.x, to.y, to.z, to.yaw, to.pitch))
    }
    /// bukkit: org/bukkit/entity/Entity.setVelocity
    pub fn set_velocity(&self, x: f64, y: f64, z: f64) -> bool {
        self.act("push", json!([x, y, z])) || self.merge(&format!("{{Motion:[{x:?}d,{y:?}d,{z:?}d]}}"))
    }
    /// bukkit: org/bukkit/entity/Entity.getVelocity
    pub fn get_velocity(&self) -> [f64; 3] {
        let motion = self.data_at("Motion");
        [0, 1, 2].map(|i| motion[i].as_f64().unwrap_or(0.0))
    }
    /// bukkit: org/bukkit/entity/Entity.isOnGround
    pub fn is_on_ground(&self) -> bool {
        crate::data::flag(&self.data_at("OnGround"))
    }
    /// bukkit: org/bukkit/entity/Entity.getScoreboardTags
    pub fn get_scoreboard_tags(&self) -> Vec<String> {
        self.data_at("Tags").as_array().into_iter().flatten().filter_map(|tag| tag.as_str().map(str::to_owned)).collect()
    }
    fn write_tags(&self, tags: &[String]) -> bool {
        self.merge(&format!("{{Tags:[{}]}}", tags.iter().map(|tag| crate::data::quoted(tag)).collect::<Vec<_>>().join(",")))
    }
    /// bukkit: org/bukkit/entity/Entity.addScoreboardTag
    pub fn add_scoreboard_tag(&self, tag: &str) -> bool {
        let mut tags = self.get_scoreboard_tags();
        if tags.iter().any(|known| known == tag) {
            return false;
        }
        tags.push(tag.to_owned());
        self.write_tags(&tags)
    }
    /// bukkit: org/bukkit/entity/Entity.removeScoreboardTag
    pub fn remove_scoreboard_tag(&self, tag: &str) -> bool {
        let mut tags = self.get_scoreboard_tags();
        let before = tags.len();
        tags.retain(|known| known != tag);
        tags.len() != before && self.write_tags(&tags)
    }
    /// bukkit: org/bukkit/entity/Entity.customName, org/bukkit/entity/Nameable.customName
    pub fn custom_name(&self) -> Option<crate::Component> {
        let name = self.data_at("CustomName");
        (!name.is_null()).then_some(crate::Component(name))
    }
    /// bukkit: org/bukkit/entity/Entity.customName, org/bukkit/Nameable.customName
    pub fn set_custom_name_component(&self, name: &crate::Component) -> bool {
        self.merge(&format!("{{CustomName:{}}}", crate::data::snbt(&name.0)))
    }
    /// bukkit: org/bukkit/Nameable.getCustomName
    pub fn get_custom_name(&self) -> Option<String> {
        self.custom_name().map(|name| name.to_legacy())
    }
    /// bukkit: org/bukkit/Nameable.setCustomName
    pub fn set_custom_name(&self, name: &str) -> bool {
        self.set_custom_name_component(&crate::adventure::legacy(name))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getHealth, org/bukkit/entity/Damageable.getHealth
    pub fn get_health(&self) -> f64 {
        self.mob_state()["health"].as_f64().or(self.data_at("Health").as_f64()).unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getMaxHealth, org/bukkit/entity/Damageable.getMaxHealth
    pub fn get_max_health(&self) -> f64 {
        self.mob_state()["max_health"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setHealth, org/bukkit/entity/Damageable.setHealth
    pub fn set_health(&self, health: f64) -> bool {
        self.act("health", json!(health))
    }
    /// bukkit: org/bukkit/entity/Entity.remove
    pub fn remove(&self) -> bool {
        self.act("remove", Value::Null)
    }
    /// bukkit: org/bukkit/entity/Damageable.kill, org/bukkit/entity/LivingEntity.kill
    pub fn kill(&self) -> bool {
        self.act("kill", Value::Null)
    }
    /// bukkit~: org/bukkit/entity/Damageable.damage | a command, so it lands at the end of the tick inside a player's event
    pub fn damage(&self, amount: f64) {
        run_command(&format!("damage {} {amount}", crate::dashed(&self.uuid)));
    }
    /// bukkit: org/bukkit/entity/Damageable.heal
    pub fn heal(&self, amount: f64) -> bool {
        self.set_health((self.get_health() + amount).min(self.get_max_health()))
    }
}

impl World {
    /// bukkit: org/bukkit/World.getEntities, org/bukkit/World.getLivingEntities
    pub fn get_entities(&self) -> Vec<Entity> {
        let mut entities = self.entities_but_players();
        entities.extend(self.player_entities(None));
        entities
    }
    fn entities_but_players(&self) -> Vec<Entity> {
        query(json!({ "op": "entities", "dimension": self.dimension() })).as_array().into_iter().flatten().filter_map(Entity::read).chain(query(json!({ "op": "entity_kept_list", "dimension": self.dimension() })).as_array().into_iter().flatten().filter_map(Entity::read)).collect()
    }
    /// bukkit: org/bukkit/World.getNearbyEntities
    pub fn get_nearby_entities(&self, at: &Location, x: f64, y: f64, z: f64) -> Vec<Entity> {
        let mut entities = self.nearby_but_players(at, x, y, z);
        entities.extend(self.player_entities(Some((at, x, y, z))));
        entities
    }
    fn nearby_but_players(&self, at: &Location, x: f64, y: f64, z: f64) -> Vec<Entity> {
        query(json!({ "op": "entities", "dimension": self.dimension() }))
            .as_array()
            .into_iter()
            .flatten()
            .filter(|entity| {
                let near = |i: usize, center: f64, range: f64| entity["pos"][i].as_f64().is_some_and(|value| (value - center).abs() <= range);
                near(0, at.x, x) && near(1, at.y, y) && near(2, at.z, z)
            })
            .filter_map(Entity::read)
            .collect()
    }
    /// bukkit: org/bukkit/World.spawn, org/bukkit/RegionAccessor.spawn, org/bukkit/World.spawnEntity, org/bukkit/RegionAccessor.spawnEntity
    pub fn spawn(&self, at: &Location, kind: &str) -> Option<Entity> {
        let id = if kind.contains(':') { kind.to_owned() } else { format!("minecraft:{}", kind.to_ascii_lowercase()) };
        let snbt = format!("{{id:{},Pos:[{:?}d,{:?}d,{:?}d],Rotation:[{:?}f,{:?}f]}}", crate::data::quoted(&id), at.x, at.y, at.z, at.yaw, at.pitch);
        let spawned = Entity::read(&query(json!({ "op": "spawn", "dimension": self.dimension(), "snbt": snbt })))?;
        if id == "minecraft:experience_orb" {
            crate::fam_entity::orb_spawned_by_plugin(&spawned);
        }
        Some(spawned)
    }
    /// bukkit: org/bukkit/World.getChunkAt
    pub fn get_chunk_at(&self, x: i32, z: i32) -> Chunk {
        Chunk { world: self.clone(), x, z }
    }
    /// bukkit: org/bukkit/World.isChunkLoaded
    pub fn is_chunk_loaded(&self, x: i32, z: i32) -> bool {
        self.get_chunk_at(x, z).is_loaded()
    }
}

/// `org.bukkit.Chunk`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    world: World,
    x: i32,
    z: i32,
}

impl Chunk {
    /// bukkit: org/bukkit/Chunk.getX
    pub fn get_x(&self) -> i32 {
        self.x
    }
    /// bukkit: org/bukkit/Chunk.getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
    /// bukkit: org/bukkit/Chunk.getWorld
    pub fn get_world(&self) -> &World {
        &self.world
    }
    /// bukkit: org/bukkit/Chunk.isLoaded
    pub fn is_loaded(&self) -> bool {
        query(json!({ "op": "chunk_loaded", "dimension": self.world.dimension(), "chunk": [self.x, self.z] })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/Chunk.getBlock
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> crate::Block {
        self.world.get_block_at(self.x * 16 + x, y, self.z * 16 + z)
    }
}

impl Location {
    /// bukkit: org/bukkit/Location.getChunk
    pub fn get_chunk(&self) -> Chunk {
        self.world.get_chunk_at(self.get_block_x() >> 4, self.get_block_z() >> 4)
    }
}
