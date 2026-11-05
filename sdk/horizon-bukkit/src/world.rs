//! The rest of `org.bukkit.World`, its border, chunks and blocks: answered by the level or by what a
//! command does.
use crate::entity::{Chunk, Entity};
use crate::util::{BoundingBox, RayTraceResult, Vector};
use crate::{Block, ItemStack, Location, Material, World, get_online_players};
use horizon_mod::{Value, query, run_command};
use serde_json::json;

impl World {
    fn level(&self) -> Value {
        query(json!({ "op": "level", "dimension": self.dimension() }))
    }
    fn in_here(&self, command: &str) {
        run_command(&format!("execute in {} run {command}", self.dimension()));
    }
    /// bukkit: org/bukkit/World.getSpawnLocation
    pub fn get_spawn_location(&self) -> Location {
        let own = query(json!({ "op": "core_world_spawn", "dimension": self.dimension() }));
        let level = if own["pos"].is_array() { json!({ "spawn": own["pos"], "spawn_angle": [own["yaw"], own["pitch"]] }) } else { self.level() };
        let at = |i: usize| level["spawn"][i].as_f64().unwrap_or(0.0);
        let angle = |i: usize| level["spawn_angle"][i].as_f64().unwrap_or(0.0) as f32;
        Location::new(self.clone(), at(0), at(1), at(2), angle(0), angle(1))
    }
    /// bukkit: org/bukkit/World.setSpawnLocation
    pub fn set_spawn_location(&self, at: &Location) -> bool {
        let pos = [at.get_block_x(), at.get_block_y(), at.get_block_z()];
        // The world players respawn in has the game's own spawn; the others keep theirs as Paper does.
        if !query(json!({ "op": "core_world_spawn_set", "dimension": self.dimension(), "pos": pos, "yaw": at.yaw, "pitch": at.pitch })).as_bool().unwrap_or(false) {
            self.in_here(&format!("setworldspawn {} {} {} {}", pos[0], pos[1], pos[2], at.yaw));
        }
        true
    }
    /// bukkit: org/bukkit/World.getSeed
    pub fn get_seed(&self) -> i64 {
        crate::worlds::created().iter().find(|row| row["id"] == self.dimension()).and_then(|row| row["seed"].as_i64()).unwrap_or_else(|| self.level()["seed"].as_i64().unwrap_or(0))
    }
    /// bukkit: org/bukkit/World.setFullTime
    pub fn set_full_time(&self, time: i64) {
        self.in_here(&format!("time set {time}"));
    }
    /// bukkit: org/bukkit/World.isDayTime
    pub fn is_day_time(&self) -> bool {
        self.get_time() % 24000 < 12000
    }
    /// bukkit: org/bukkit/World.getWeatherDuration
    pub fn get_weather_duration(&self) -> i32 {
        self.level()["rain_time"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/World.setWeatherDuration
    pub fn set_weather_duration(&self, ticks: i32) {
        horizon_mod::query(serde_json::json!({ "op": "ev_world_free_weather_plugin" }));
        self.in_here(&format!("weather {} {ticks}t", if self.has_storm() { "rain" } else { "clear" }));
    }
    /// bukkit: org/bukkit/World.isThundering
    pub fn is_thundering(&self) -> bool {
        self.level()["thundering"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/World.setThundering
    pub fn set_thundering(&self, thundering: bool) {
        horizon_mod::query(serde_json::json!({ "op": "ev_world_free_weather_plugin" }));
        self.in_here(if thundering { "weather thunder" } else { "weather rain" });
    }
    /// bukkit: org/bukkit/World.getThunderDuration
    pub fn get_thunder_duration(&self) -> i32 {
        self.level()["thunder_time"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/World.setThunderDuration
    pub fn set_thunder_duration(&self, ticks: i32) {
        self.in_here(&format!("weather thunder {ticks}t"));
    }
    /// bukkit: org/bukkit/World.isClearWeather
    pub fn is_clear_weather(&self) -> bool {
        !self.has_storm() && !self.is_thundering()
    }
    /// bukkit: org/bukkit/World.getClearWeatherDuration
    pub fn get_clear_weather_duration(&self) -> i32 {
        self.level()["clear_time"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/World.setClearWeatherDuration
    pub fn set_clear_weather_duration(&self, ticks: i32) {
        horizon_mod::query(serde_json::json!({ "op": "ev_world_free_weather_plugin" }));
        self.in_here(&format!("weather clear {ticks}t"));
    }
    /// bukkit: org/bukkit/World.getDifficulty
    pub fn get_difficulty(&self) -> &'static str {
        crate::enums::difficulty::values().get(self.level()["difficulty"].as_u64().unwrap_or(0) as usize).copied().unwrap_or("NORMAL")
    }
    /// bukkit: org/bukkit/World.setDifficulty
    pub fn set_difficulty(&self, difficulty: &str) {
        run_command(&format!("difficulty {}", difficulty.to_ascii_lowercase()));
    }
    /// bukkit: org/bukkit/World.isHardcore
    pub fn is_hardcore(&self) -> bool {
        let dimension = self.dimension().to_owned();
        if let Some(row) = crate::worlds::created().into_iter().find(|row| row["id"] == dimension.as_str()) {
            return row["hardcore"].as_bool().unwrap_or(false);
        }
        self.level()["hardcore"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/World.getMinHeight, org/bukkit/RegionAccessor.getMinHeight, org/bukkit/generator/WorldInfo.getMinHeight
    pub fn get_min_height(&self) -> i32 {
        self.level()["min_y"].as_i64().unwrap_or(-64) as i32
    }
    /// bukkit: org/bukkit/World.getMaxHeight, org/bukkit/RegionAccessor.getMaxHeight, org/bukkit/generator/WorldInfo.getMaxHeight
    pub fn get_max_height(&self) -> i32 {
        self.level()["max_y"].as_i64().unwrap_or(320) as i32
    }
    /// bukkit: org/bukkit/World.getLogicalHeight
    pub fn get_logical_height(&self) -> i32 {
        if self.dimension().ends_with("nether") { 256 } else { self.get_max_height() - self.get_min_height() }
    }
    /// bukkit: org/bukkit/World.getSeaLevel
    pub fn get_sea_level(&self) -> i32 {
        match self.dimension() {
            dimension if dimension.ends_with("nether") => 32,
            dimension if dimension.ends_with("end") => 0,
            _ => 63,
        }
    }
    /// bukkit: org/bukkit/World.isNatural
    pub fn is_natural(&self) -> bool {
        self.dimension().ends_with("overworld")
    }
    /// bukkit: org/bukkit/World.isBedWorks
    pub fn is_bed_works(&self) -> bool {
        self.dimension().ends_with("overworld")
    }
    /// bukkit: org/bukkit/World.hasSkyLight
    pub fn has_sky_light(&self) -> bool {
        self.dimension().ends_with("overworld")
    }
    /// bukkit: org/bukkit/World.hasCeiling
    pub fn has_ceiling(&self) -> bool {
        self.dimension().ends_with("nether")
    }
    /// bukkit: org/bukkit/World.isPiglinSafe
    pub fn is_piglin_safe(&self) -> bool {
        self.dimension().ends_with("nether")
    }
    /// bukkit: org/bukkit/World.isRespawnAnchorWorks
    pub fn is_respawn_anchor_works(&self) -> bool {
        self.dimension().ends_with("nether")
    }
    /// bukkit: org/bukkit/World.hasRaids
    pub fn has_raids(&self) -> bool {
        self.dimension().ends_with("overworld")
    }
    /// bukkit: org/bukkit/World.isUltraWarm
    pub fn is_ultra_warm(&self) -> bool {
        self.dimension().ends_with("nether")
    }
    /// bukkit: org/bukkit/World.getEnvironment, org/bukkit/generator/WorldInfo.getEnvironment
    pub fn get_environment(&self) -> &'static str {
        match self.dimension() {
            "minecraft:the_nether" => "NETHER",
            "minecraft:the_end" => "THE_END",
            "minecraft:overworld" => "NORMAL",
            created => match crate::worlds::created().iter().find(|row| row["id"] == created).and_then(|row| row["environment"].as_str().map(str::to_owned)).as_deref() {
                Some("NETHER") => "NETHER",
                Some("THE_END") => "THE_END",
                _ => "NORMAL",
            },
        }
    }

    // ---- Game rules ----

    /// bukkit: org/bukkit/World.getGameRuleValue
    pub fn get_game_rule_value(&self, rule: &str) -> Option<String> {
        match query(json!({ "op": "game_rule", "name": rule })) {
            Value::Null => None,
            Value::String(value) => Some(value),
            value => Some(value.to_string()),
        }
    }
    /// bukkit: org/bukkit/World.setGameRuleValue, org/bukkit/World.setGameRule
    pub fn set_game_rule(&self, rule: &str, value: &str) -> bool {
        run_command(&format!("gamerule {} {value}", rule.trim_start_matches("minecraft:")));
        true
    }
    /// bukkit: org/bukkit/World.isGameRule
    pub fn is_game_rule(&self, rule: &str) -> bool {
        self.get_game_rule_value(rule).is_some()
    }
    /// bukkit: org/bukkit/World.getPVP
    pub fn get_pvp(&self) -> bool {
        self.get_game_rule_value("pvp").is_none_or(|value| value == "true")
    }
    /// bukkit: org/bukkit/World.setPVP
    pub fn set_pvp(&self, pvp: bool) {
        self.set_game_rule("pvp", if pvp { "true" } else { "false" });
    }

    // ---- Who and what is here ----

    /// bukkit: org/bukkit/World.getPlayerCount
    pub fn get_player_count(&self) -> usize {
        self.get_players().len()
    }
    /// bukkit: org/bukkit/World.getEntityCount
    pub fn get_entity_count(&self) -> usize {
        self.get_entities().len() + self.get_player_count()
    }
    /// bukkit: org/bukkit/World.getNearbyPlayers
    pub fn get_nearby_players(&self, at: &Location, radius: f64) -> Vec<crate::Player> {
        self.get_players().into_iter().filter(|player| player.get_location().distance_squared(at) <= radius * radius).collect()
    }
    /// bukkit: org/bukkit/World.getNearbyLivingEntities
    pub fn get_nearby_living_entities(&self, at: &Location, radius: f64) -> Vec<Entity> {
        self.get_nearby_entities(at, radius, radius, radius).into_iter().filter(|entity| !entity.data_at("Health").is_null()).collect()
    }
    /// bukkit: org/bukkit/World.getEntity, org/bukkit/Bukkit.getEntity, org/bukkit/Server.getEntity
    pub fn get_entity(&self, uuid: &str) -> Option<Entity> {
        self.get_entities().into_iter().find(|entity| entity.get_unique_id() == uuid)
    }
    /// bukkit: org/bukkit/World.dropItem, org/bukkit/RegionAccessor.dropItem
    pub fn drop_item(&self, at: &Location, stack: &ItemStack) -> Option<crate::entities::Item> {
        let snbt = format!("{{id:\"minecraft:item\",Pos:[{:?}d,{:?}d,{:?}d],Item:{},PickupDelay:10s}}", at.x, at.y, at.z, stack.to_snbt());
        Entity::read(&query(json!({ "op": "spawn", "dimension": self.dimension(), "snbt": snbt }))).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/World.dropItemNaturally
    pub fn drop_item_naturally(&self, at: &Location, stack: &ItemStack) -> Option<crate::entities::Item> {
        let [x, y, z] = crate::fam_core::natural_spread();
        let mut there = at.clone();
        there.add(x, y, z);
        self.drop_item(&there, stack)
    }
    /// bukkit: org/bukkit/World.strikeLightning
    pub fn strike_lightning(&self, at: &Location) {
        query(json!({ "op": "ev_world_bolt_cause", "cause": "CUSTOM" }));
        self.in_here(&format!("summon minecraft:lightning_bolt {} {} {}", at.x, at.y, at.z));
    }
    /// bukkit: org/bukkit/World.strikeLightningEffect
    pub fn strike_lightning_effect(&self, at: &Location) {
        query(json!({ "op": "ev_world_bolt_cause", "cause": "CUSTOM" }));
        self.in_here(&format!("summon minecraft:lightning_bolt {} {} {} {{visual_only:true}}", at.x, at.y, at.z));
    }
    /// `create_explosion_with`, breaking blocks and starting no fires, as Java's shortest form.
    pub fn create_explosion(&self, at: &Location, power: f32) -> bool {
        self.create_explosion_with(at, power, false, true)
    }
    /// bukkit: org/bukkit/World.playSound
    pub fn play_sound(&self, at: &Location, sound: &str, volume: f32, pitch: f32) {
        self.in_here(&format!("playsound {sound} master @a {} {} {} {volume} {pitch}", at.x, at.y, at.z));
    }
    /// bukkit: org/bukkit/World.playSound
    ///
    /// With a `SoundCategory`.
    pub fn play_sound_in(&self, at: &Location, sound: &str, category: &str, volume: f32, pitch: f32) {
        self.in_here(&format!("playsound {sound} {} @a {} {} {} {volume} {pitch}", crate::sound_source(category), at.x, at.y, at.z));
    }
    /// `spawn_particle_forced`, with no force: seen from as far as the game normally shows one.
    pub fn spawn_particle(&self, particle: &str, at: &Location, count: u32, dx: f64, dy: f64, dz: f64, speed: f64) {
        let particle = crate::enums::particle::value_of(particle).unwrap_or(particle);
        self.in_here(&format!("particle {particle} {} {} {} {dx} {dy} {dz} {speed} {count}", at.x, at.y, at.z));
    }
    /// bukkit: org/bukkit/World.save
    pub fn save(&self) {
        run_command("save-all");
    }
    /// bukkit: org/bukkit/World.getWorldBorder
    pub fn get_world_border(&self) -> WorldBorder {
        WorldBorder { world: self.clone() }
    }

    // ---- Chunks ----

    /// bukkit: org/bukkit/World.getLoadedChunks
    pub fn get_loaded_chunks(&self) -> Vec<Chunk> {
        self.level()["chunks"].as_array().into_iter().flatten().map(|chunk| self.get_chunk_at(chunk[0].as_i64().unwrap_or(0) as i32, chunk[1].as_i64().unwrap_or(0) as i32)).collect()
    }
    /// bukkit: org/bukkit/World.getForceLoadedChunks
    pub fn get_force_loaded_chunks(&self) -> Vec<Chunk> {
        self.level()["forced"].as_array().into_iter().flatten().map(|chunk| self.get_chunk_at(chunk[0].as_i64().unwrap_or(0) as i32, chunk[1].as_i64().unwrap_or(0) as i32)).collect()
    }
    /// bukkit: org/bukkit/World.isChunkForceLoaded
    pub fn is_chunk_force_loaded(&self, x: i32, z: i32) -> bool {
        self.get_force_loaded_chunks().iter().any(|chunk| chunk.get_x() == x && chunk.get_z() == z)
    }
    /// bukkit: org/bukkit/World.setChunkForceLoaded
    pub fn set_chunk_force_loaded(&self, x: i32, z: i32, forced: bool) {
        self.in_here(&format!("forceload {} {} {}", if forced { "add" } else { "remove" }, x * 16, z * 16));
    }
    /// bukkit: org/bukkit/World.getHighestBlockAt, org/bukkit/RegionAccessor.getHighestBlockAt
    pub fn get_highest_block_at(&self, x: i32, z: i32) -> Block {
        let (bottom, top) = (self.get_min_height(), self.get_max_height());
        let y = (bottom..top).rev().find(|y| !self.get_block_at(x, *y, z).is_empty()).unwrap_or(bottom);
        self.get_block_at(x, y, z)
    }
    /// bukkit: org/bukkit/World.getHighestBlockYAt, org/bukkit/RegionAccessor.getHighestBlockYAt
    pub fn get_highest_block_y_at(&self, x: i32, z: i32) -> i32 {
        self.get_highest_block_at(x, z).get_y()
    }
    /// bukkit: org/bukkit/World.getHighestBlockYAt, org/bukkit/RegionAccessor.getHighestBlockYAt
    ///
    /// At a location's column: `getHighestBlockYAt(Location)`.
    pub fn get_highest_block_y_at_location(&self, at: &Location) -> i32 {
        self.get_highest_block_y_at(at.get_block_x(), at.get_block_z())
    }
    /// bukkit: org/bukkit/World.getHighestBlockAt, org/bukkit/RegionAccessor.getHighestBlockAt
    ///
    /// At a location's column: `getHighestBlockAt(Location)`.
    pub fn get_highest_block_at_location(&self, at: &Location) -> Block {
        self.get_highest_block_at(at.get_block_x(), at.get_block_z())
    }
    /// bukkit: org/bukkit/RegionAccessor.getType
    pub fn get_type(&self, at: &Location) -> Material {
        at.get_block().get_type()
    }
    /// bukkit: org/bukkit/World.rayTraceBlocks
    ///
    /// Steps along the ray a block at a time, and meets a block's whole cell.
    pub fn ray_trace_blocks(&self, start: &Location, direction: &Vector, distance: f64) -> Option<RayTraceResult> {
        let origin = start.to_vector();
        let mut step = *direction;
        step.normalize();
        let mut travelled = 0.0;
        while travelled <= distance {
            let (x, y, z) = (origin.x + step.x * travelled, origin.y + step.y * travelled, origin.z + step.z * travelled);
            let block = self.get_block_at(x.floor() as i32, y.floor() as i32, z.floor() as i32);
            if block.get_facts()["full_collision"].as_bool().unwrap_or(false) {
                let (bx, by, bz) = (f64::from(block.get_x()), f64::from(block.get_y()), f64::from(block.get_z()));
                let cell = BoundingBox::new(bx, by, bz, bx + 1.0, by + 1.0, bz + 1.0);
                if let Some(mut hit) = cell.ray_trace(&origin, &step, distance) {
                    hit.block = Some(block);
                    return Some(hit);
                }
            }
            travelled += 0.1;
        }
        None
    }
    /// bukkit: org/bukkit/World.rayTraceEntities
    pub fn ray_trace_entities(&self, start: &Location, direction: &Vector, distance: f64) -> Option<RayTraceResult> {
        let origin = start.to_vector();
        self.get_entities()
            .into_iter()
            .filter_map(|entity| {
                let at = entity.get_location()?;
                let bounds = BoundingBox::new(at.x - 0.3, at.y, at.z - 0.3, at.x + 0.3, at.y + 1.8, at.z + 0.3);
                let mut hit = bounds.ray_trace(&origin, direction, distance)?;
                hit.entity = Some(entity);
                Some(hit)
            })
            .min_by(|first, second| first.position.distance_squared(&origin).total_cmp(&second.position.distance_squared(&origin)))
    }
}

impl World {
    /// bukkit: org/bukkit/World.getEntitiesByClass, org/bukkit/RegionAccessor.getEntitiesByClass
    ///
    /// By the game's id: `minecraft:zombie`.
    pub fn get_entities_by_type(&self, kind: &str) -> Vec<Entity> {
        self.get_entities().into_iter().filter(|entity| entity.get_type() == kind).collect()
    }
    /// bukkit: org/bukkit/World.getViewDistance
    pub fn get_view_distance(&self) -> i32 {
        self.level_distance("view")
    }
    /// bukkit: org/bukkit/World.getSimulationDistance
    pub fn get_simulation_distance(&self) -> i32 {
        self.level_distance("simulation")
    }
    /// bukkit: org/bukkit/World.isAutoSave
    pub fn is_auto_save(&self) -> bool {
        query(json!({ "op": "core_auto_save", "dimension": self.dimension() })).as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/World.setAutoSave
    pub fn set_auto_save(&self, on: bool) {
        query(json!({ "op": "core_auto_save", "dimension": self.dimension(), "on": on }));
    }
    /// bukkit: org/bukkit/World.loadChunk
    pub fn load_chunk(&self, x: i32, z: i32) {
        self.set_chunk_force_loaded(x, z, true);
    }
    /// bukkit: org/bukkit/World.unloadChunk, org/bukkit/World.unloadChunkRequest
    pub fn unload_chunk(&self, x: i32, z: i32) -> bool {
        self.set_chunk_force_loaded(x, z, false);
        true
    }
    /// bukkit: org/bukkit/World.addPluginChunkTicket
    pub fn add_plugin_chunk_ticket(&self, x: i32, z: i32) -> bool {
        self.get_chunk_at(x, z).add_plugin_chunk_ticket()
    }
    /// bukkit: org/bukkit/World.removePluginChunkTicket
    pub fn remove_plugin_chunk_ticket(&self, x: i32, z: i32) -> bool {
        self.get_chunk_at(x, z).remove_plugin_chunk_ticket()
    }
    /// bukkit: org/bukkit/World.spawnArrow
    pub fn spawn_arrow(&self, at: &Location, direction: &crate::util::Vector, speed: f32) -> Option<Entity> {
        let mut velocity = *direction;
        velocity.normalize();
        velocity.multiply(f64::from(speed));
        let entity = self.spawn(at, "minecraft:arrow")?;
        entity.set_velocity(velocity.x, velocity.y, velocity.z);
        Some(entity)
    }
    /// bukkit: org/bukkit/World.spawnFallingBlock
    pub fn spawn_falling_block(&self, at: &Location, block: &crate::block_data::BlockData) -> Option<crate::entities::FallingBlock> {
        let entity: crate::entities::FallingBlock = self.spawn(at, "minecraft:falling_block")?.cast();
        entity.set_block_data(block);
        Some(entity)
    }
}

impl Chunk {
    /// bukkit: org/bukkit/Chunk.getTileEntities
    pub fn get_tile_entities(&self) -> Vec<crate::block_data::TileState> {
        let found = query(json!({ "op": "chunk_entities", "dimension": self.get_world().dimension(), "chunk": [self.get_x(), self.get_z()] }));
        found.as_array().into_iter().flatten().map(|pos| self.get_world().get_block_at(pos[0].as_i64().unwrap_or(0) as i32, pos[1].as_i64().unwrap_or(0) as i32, pos[2].as_i64().unwrap_or(0) as i32).tile()).collect()
    }
    /// bukkit: org/bukkit/Chunk.load
    pub fn load(&self) -> bool {
        self.get_world().load_chunk(self.get_x(), self.get_z());
        true
    }
    /// bukkit: org/bukkit/Chunk.unload
    pub fn unload(&self) -> bool {
        self.get_world().unload_chunk(self.get_x(), self.get_z())
    }
    /// bukkit: org/bukkit/Chunk.getChunkKey
    pub fn get_chunk_key(&self) -> i64 {
        (i64::from(self.get_z()) << 32) | (i64::from(self.get_x()) & 0xffff_ffff)
    }
    /// bukkit: org/bukkit/Chunk.getEntities
    pub fn get_entities(&self) -> Vec<Entity> {
        self.get_world()
            .get_entities()
            .into_iter()
            .filter(|entity| {
                let Some(at) = entity.get_location() else {
                    return false;
                };
                at.get_block_x() >> 4 == self.get_x() && at.get_block_z() >> 4 == self.get_z()
            })
            .collect()
    }
    /// bukkit: org/bukkit/Chunk.isForceLoaded
    pub fn is_force_loaded(&self) -> bool {
        self.get_world().is_chunk_force_loaded(self.get_x(), self.get_z())
    }
    /// bukkit: org/bukkit/Chunk.setForceLoaded
    pub fn set_force_loaded(&self, forced: bool) {
        self.get_world().set_chunk_force_loaded(self.get_x(), self.get_z(), forced);
    }
    /// bukkit: org/bukkit/Chunk.isSlimeChunk
    pub fn is_slime_chunk(&self) -> bool {
        let (x, z) = (i64::from(self.get_x()), i64::from(self.get_z()));
        let seed = self.get_world().get_seed().wrapping_add(x.wrapping_mul(x).wrapping_mul(4_987_142)).wrapping_add(x.wrapping_mul(5_947_611)).wrapping_add(z.wrapping_mul(z).wrapping_mul(4_392_871)).wrapping_add(z.wrapping_mul(389_711)) ^ 987_234_911;
        // java.util.Random(seed).nextInt(10) == 0.
        let mut state = (seed ^ 0x5_DEEC_E66D) & ((1 << 48) - 1);
        loop {
            state = state.wrapping_mul(0x5_DEEC_E66D).wrapping_add(0xB) & ((1 << 48) - 1);
            let bits = (state >> 17) as i32;
            let value = bits % 10;
            if bits.wrapping_sub(value).wrapping_add(9) >= 0 {
                return value == 0;
            }
        }
    }
}

/// `org.bukkit.WorldBorder`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldBorder {
    world: World,
}

impl WorldBorder {
    fn state(&self) -> Value {
        self.world.level()["border"].take()
    }
    fn in_here(&self, command: &str) {
        self.world.in_here(&format!("worldborder {command}"));
    }
    /// bukkit: org/bukkit/WorldBorder.getWorld
    pub fn get_world(&self) -> &World {
        &self.world
    }
    /// bukkit: org/bukkit/WorldBorder.getSize
    pub fn get_size(&self) -> f64 {
        self.state()["size"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/WorldBorder.setSize
    pub fn set_size(&self, size: f64) {
        self.in_here(&format!("set {size}"));
    }
    /// bukkit: org/bukkit/WorldBorder.changeSize
    pub fn change_size(&self, size: f64, ticks: i64) {
        self.in_here(&format!("set {size} {}", ticks as f64 / 20.0));
    }
    /// bukkit: org/bukkit/WorldBorder.getCenter
    pub fn get_center(&self) -> Location {
        let state = self.state();
        Location::new(self.world.clone(), state["center"][0].as_f64().unwrap_or(0.0), 0.0, state["center"][1].as_f64().unwrap_or(0.0), 0.0, 0.0)
    }
    /// bukkit: org/bukkit/WorldBorder.setCenter
    pub fn set_center(&self, x: f64, z: f64) {
        self.in_here(&format!("center {x} {z}"));
    }
    /// bukkit: org/bukkit/WorldBorder.getDamageBuffer
    pub fn get_damage_buffer(&self) -> f64 {
        self.state()["safe_zone"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/WorldBorder.setDamageBuffer
    pub fn set_damage_buffer(&self, blocks: f64) {
        self.in_here(&format!("damage buffer {blocks}"));
    }
    /// bukkit: org/bukkit/WorldBorder.getDamageAmount
    pub fn get_damage_amount(&self) -> f64 {
        self.state()["damage_per_block"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/WorldBorder.setDamageAmount
    pub fn set_damage_amount(&self, amount: f64) {
        self.in_here(&format!("damage amount {amount}"));
    }
    /// bukkit: org/bukkit/WorldBorder.getWarningTime
    pub fn get_warning_time(&self) -> i32 {
        self.state()["warning_time"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/WorldBorder.setWarningTime
    pub fn set_warning_time(&self, seconds: i32) {
        self.in_here(&format!("warning time {seconds}"));
    }
    /// bukkit: org/bukkit/WorldBorder.getWarningTimeTicks
    pub fn get_warning_time_ticks(&self) -> i32 {
        self.get_warning_time() * 20
    }
    /// bukkit: org/bukkit/WorldBorder.setWarningTimeTicks
    pub fn set_warning_time_ticks(&self, ticks: i32) {
        self.set_warning_time(ticks / 20);
    }
    /// bukkit: org/bukkit/WorldBorder.getWarningDistance
    pub fn get_warning_distance(&self) -> i32 {
        self.state()["warning_distance"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/WorldBorder.setWarningDistance
    pub fn set_warning_distance(&self, blocks: i32) {
        self.in_here(&format!("warning distance {blocks}"));
    }
    /// bukkit: org/bukkit/WorldBorder.isInside
    pub fn is_inside(&self, at: &Location) -> bool {
        let center = self.get_center();
        let half = self.get_size() / 2.0;
        (at.x - center.x).abs() <= half && (at.z - center.z).abs() <= half
    }
    /// bukkit: org/bukkit/WorldBorder.getMaxSize
    pub fn get_max_size(&self) -> f64 {
        self.state()["absolute_max"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/WorldBorder.getMaxCenterCoordinate
    pub fn get_max_center_coordinate(&self) -> f64 {
        2.9999984e7
    }
    /// bukkit: org/bukkit/WorldBorder.reset
    pub fn reset(&self) {
        self.set_size(5.9999968e7);
        self.set_center(0.0, 0.0);
        self.set_damage_amount(0.2);
        self.set_damage_buffer(5.0);
        self.set_warning_distance(5);
        self.set_warning_time(15);
    }
}

impl Block {
    /// What the game knows of this block: light, solidity, hardness, biome.
    pub fn get_facts(&self) -> Value {
        query(json!({ "op": "block_facts", "dimension": self.get_world().dimension(), "pos": [self.get_x(), self.get_y(), self.get_z()] }))
    }
    fn fact(&self, key: &str) -> bool {
        self.get_facts()[key].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/block/Block.getLightFromSky
    pub fn get_light_from_sky(&self) -> i32 {
        self.get_facts()["sky_light"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Block.getLightFromBlocks
    pub fn get_light_from_blocks(&self) -> i32 {
        self.get_facts()["block_light"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Block.getLightLevel
    pub fn get_light_level(&self) -> i32 {
        self.get_light_from_sky().max(self.get_light_from_blocks())
    }
    /// bukkit: org/bukkit/block/Block.getBlockKey
    pub fn get_block_key(&self) -> i64 {
        (i64::from(self.get_x()) & 0x7ff_ffff) | ((i64::from(self.get_z()) & 0x7ff_ffff) << 27) | (i64::from(self.get_y()) << 54)
    }
    /// bukkit: org/bukkit/block/Block.getChunk
    pub fn get_chunk(&self) -> Chunk {
        self.get_world().get_chunk_at(self.get_x() >> 4, self.get_z() >> 4)
    }
    /// bukkit: org/bukkit/block/Block.getBiome, org/bukkit/block/Block.getComputedBiome, org/bukkit/RegionAccessor.getBiome, org/bukkit/World.getBiome
    pub fn get_biome(&self) -> Option<String> {
        self.get_facts()["biome"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Block.isLiquid
    pub fn is_liquid(&self) -> bool {
        self.fact("liquid")
    }
    /// bukkit: org/bukkit/block/Block.isSolid
    pub fn is_solid(&self) -> bool {
        self.fact("solid")
    }
    /// bukkit: org/bukkit/block/Block.isReplaceable
    pub fn is_replaceable(&self) -> bool {
        self.fact("replaceable")
    }
    /// bukkit: org/bukkit/block/Block.isPassable
    pub fn is_passable(&self) -> bool {
        !self.fact("full_collision")
    }
    /// bukkit: org/bukkit/block/Block.isCollidable
    pub fn is_collidable(&self) -> bool {
        self.fact("full_collision")
    }
    /// bukkit: org/bukkit/block/Block.isSuffocating
    pub fn is_suffocating(&self) -> bool {
        self.fact("occluding") && self.fact("full_collision")
    }
    /// bukkit: org/bukkit/block/Block.getFace
    pub fn get_face(&self, other: &Self) -> Option<&'static str> {
        let (x, y, z) = (other.get_x() - self.get_x(), other.get_y() - self.get_y(), other.get_z() - self.get_z());
        crate::enums::block_face::values().iter().copied().find(|face| Self::offset(face) == (x, y, z))
    }
    fn offset(face: &str) -> (i32, i32, i32) {
        match face {
            "NORTH" => (0, 0, -1),
            "EAST" => (1, 0, 0),
            "SOUTH" => (0, 0, 1),
            "WEST" => (-1, 0, 0),
            "UP" => (0, 1, 0),
            "DOWN" => (0, -1, 0),
            _ => (0, 0, 0),
        }
    }
    /// bukkit: org/bukkit/block/Block.getBoundingBox
    pub fn get_bounding_box(&self) -> BoundingBox {
        let (x, y, z) = (f64::from(self.get_x()), f64::from(self.get_y()), f64::from(self.get_z()));
        let facts = self.get_facts();
        let boxes = facts["shape"].as_array().cloned().unwrap_or_default();
        let mut bounds: Option<BoundingBox> = None;
        for shape in boxes {
            let side = |i: usize| shape[i].as_f64().unwrap_or(0.0);
            let piece = BoundingBox::new(x + side(0), y + side(1), z + side(2), x + side(3), y + side(4), z + side(5));
            bounds = Some(bounds.map_or(piece, |mut all| *all.union_box(&piece)));
        }
        bounds.unwrap_or_else(|| BoundingBox::new(x, y, z, x, y, z))
    }
    /// bukkit: org/bukkit/block/Block.tick
    pub fn tick(&self) {
        self.get_world().in_here(&format!("tick step 0 {} {} {}", self.get_x(), self.get_y(), self.get_z()));
    }
}

impl crate::BlockState {
    /// bukkit: org/bukkit/block/BlockState.getLocation
    pub fn get_location(&self) -> Location {
        self.get_block().get_location()
    }
    /// bukkit: org/bukkit/block/BlockState.getWorld
    pub fn get_world(&self) -> World {
        self.get_block().get_world().clone()
    }
    /// bukkit: org/bukkit/block/BlockState.getX
    pub fn get_x(&self) -> i32 {
        self.get_block().get_x()
    }
    /// bukkit: org/bukkit/block/BlockState.getY
    pub fn get_y(&self) -> i32 {
        self.get_block().get_y()
    }
    /// bukkit: org/bukkit/block/BlockState.getZ
    pub fn get_z(&self) -> i32 {
        self.get_block().get_z()
    }
    /// bukkit: org/bukkit/block/BlockState.getChunk
    pub fn get_chunk(&self) -> Chunk {
        self.get_block().get_chunk()
    }
    /// bukkit: org/bukkit/block/BlockState.getLightLevel
    pub fn get_light_level(&self) -> i32 {
        self.get_block().get_light_level()
    }
    /// bukkit: org/bukkit/block/BlockState.setType
    pub fn set_type(&mut self, material: &Material) {
        self.set_block_data(material.key());
    }
    /// bukkit: org/bukkit/block/BlockState.copy
    pub fn copy(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/block/BlockState.isPlaced
    pub fn is_placed(&self) -> bool {
        self.placed()
    }
}

/// bukkit: org/bukkit/Bukkit.matchPlayer, org/bukkit/Server.matchPlayer
pub fn match_player(name: &str) -> Vec<crate::Player> {
    let lower = name.to_ascii_lowercase();
    let everyone = get_online_players();
    if let Some(exact) = everyone.iter().find(|player| player.get_name().eq_ignore_ascii_case(name)) {
        return vec![exact.clone()];
    }
    everyone.into_iter().filter(|player| player.get_name().to_ascii_lowercase().contains(&lower)).collect()
}
