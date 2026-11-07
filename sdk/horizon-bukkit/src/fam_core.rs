//! Family core: see plugin-tools/tools/family.
//!
//! The rest of `org.bukkit`: worlds, chunks, maps, loot, structures, commands, configuration, and the
//! small value classes. What the game can answer goes through `core_*` queries, whose answers are
//! `bundle/paper_api/paper_fam_core.rs`. A method missing here is one Horizon can't do yet: nothing
//! here pretends.
#![allow(missing_docs)]
use crate::entity::{Chunk, Entity};
use crate::persistence::NamespacedKey;
use crate::pure::Position;
use crate::server::server_folder;
use crate::util::{BoundingBox, RayTraceResult, Vector};
use crate::{Block, BossBar, Color, CommandSender, Component, ItemStack, Location, Material, OfflinePlayer, Player, Plugin, World, adventure, get_online_players, get_player};
use horizon_mod::{Value, query, run_command};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

fn key_of(name: &str) -> String {
    if name.contains(':') { name.to_owned() } else { format!("minecraft:{}", name.to_ascii_lowercase()) }
}

fn int(value: &Value) -> i32 {
    value.as_i64().unwrap_or(0) as i32
}

fn at(value: &Value, world: &World) -> Option<Location> {
    let side = |i: usize| value[i].as_f64();
    Some(Location::new(world.clone(), side(0)?, side(1)?, side(2)?, 0.0, 0.0))
}

fn file_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_default()).unwrap_or(Value::Null)
}

fn property(key: &str) -> Option<String> {
    let text = std::fs::read_to_string(server_folder().join("server.properties")).unwrap_or_default();
    text.lines().filter(|line| !line.starts_with('#')).filter_map(|line| line.split_once('=')).find(|(name, _)| name.trim() == key).map(|(_, value)| value.trim().to_owned())
}

/// `bukkit.yml`, read as Paper reads it: a key the file doesn't have has Paper's default.
fn bukkit_yml() -> crate::config::YamlConfiguration {
    crate::config::YamlConfiguration::load_configuration(&server_folder().join("bukkit.yml"))
}

// ---- Worlds ----

pub use crate::worlds::{BiomeProvider, BlockPopulator, ChunkGenerator};

/// bukkit: org/bukkit/World$ChunkLoadCallback, org/bukkit/World$ChunkLoadCallback.onLoad, org/bukkit/World$ChunkLoadCallback.accept
pub type ChunkLoadCallback = Box<dyn FnOnce(Chunk) + Send>;

/// `SpawnCategory` to the game's `MobCategory`.
fn mob_category(category: &str) -> &'static str {
    match category {
        "MONSTER" => "monster",
        "ANIMAL" => "creature",
        "WATER_ANIMAL" => "water_creature",
        "WATER_AMBIENT" => "water_ambient",
        "WATER_UNDERGROUND_CREATURE" => "underground_water_creature",
        "AXOLOTL" => "axolotls",
        "AMBIENT" => "ambient",
        _ => "misc",
    }
}

fn spawn_limit_of(category: &str) -> i32 {
    int(&query(json!({ "op": "core_spawn_limits" }))[mob_category(category)]["limit"])
}

fn ticks_per_spawns_of(category: &str) -> i64 {
    if let Some(set) = query(json!({ "op": "core_ticks_per", "category": mob_category(category) })).as_i64() {
        return set;
    }
    // `NaturalSpawner`: a persistent kind tries every 400 ticks, any other every tick.
    if query(json!({ "op": "core_spawn_limits" }))[mob_category(category)]["persistent"].as_bool().unwrap_or(false) { 400 } else { 1 }
}

/// The game's feature for a `TreeType`, as `CraftWorld` names them.
fn tree_feature(tree: &str) -> Option<&'static str> {
    Some(match tree {
        "TREE" => "minecraft:oak",
        "BIG_TREE" => "minecraft:fancy_oak",
        "REDWOOD" => "minecraft:spruce",
        "TALL_REDWOOD" => "minecraft:pine",
        "BIRCH" => "minecraft:birch",
        "JUNGLE" => "minecraft:mega_jungle_tree",
        "SMALL_JUNGLE" => "minecraft:jungle_tree_no_vine",
        "COCOA_TREE" => "minecraft:jungle_tree",
        "JUNGLE_BUSH" => "minecraft:jungle_bush",
        "RED_MUSHROOM" => "minecraft:huge_red_mushroom",
        "BROWN_MUSHROOM" => "minecraft:huge_brown_mushroom",
        "SWAMP" => "minecraft:swamp_oak",
        "ACACIA" => "minecraft:acacia",
        "DARK_OAK" => "minecraft:dark_oak",
        "MEGA_REDWOOD" => "minecraft:mega_spruce",
        "MEGA_PINE" => "minecraft:mega_pine",
        "TALL_BIRCH" => "minecraft:super_birch_bees_0002",
        "CHORUS_PLANT" => "minecraft:chorus_plant",
        "CRIMSON_FUNGUS" => "minecraft:crimson_fungus_planted",
        "WARPED_FUNGUS" => "minecraft:warped_fungus_planted",
        "AZALEA" => "minecraft:azalea_tree",
        "MANGROVE" => "minecraft:mangrove",
        "TALL_MANGROVE" => "minecraft:tall_mangrove",
        "CHERRY" => "minecraft:cherry",
        "PALE_OAK" => "minecraft:pale_oak",
        "PALE_OAK_CREAKING" => "minecraft:pale_oak_creaking",
        "RED_POPLAR" => "minecraft:red_poplar",
        "ORANGE_POPLAR" => "minecraft:orange_poplar",
        "YELLOW_POPLAR" => "minecraft:yellow_poplar",
        _ => return None,
    })
}

/// The cells a ray crosses, in order, to `distance`.
fn cells_along(start: &Vector, way: &Vector, distance: f64) -> Vec<(i32, i32, i32)> {
    let mut cell = [start.x.floor(), start.y.floor(), start.z.floor()];
    let origin = [start.x, start.y, start.z];
    let dir = [way.x, way.y, way.z];
    let mut cross = [f64::INFINITY; 3];
    let mut each = [f64::INFINITY; 3];
    for axis in 0..3 {
        if dir[axis] != 0.0 {
            let next = if dir[axis] > 0.0 { cell[axis] + 1.0 } else { cell[axis] };
            cross[axis] = (next - origin[axis]) / dir[axis];
            each[axis] = 1.0 / dir[axis].abs();
        }
    }
    let mut out = vec![(cell[0] as i32, cell[1] as i32, cell[2] as i32)];
    loop {
        let axis = if cross[0] <= cross[1] && cross[0] <= cross[2] {
            0
        } else if cross[1] <= cross[2] {
            1
        } else {
            2
        };
        if cross[axis] > distance {
            return out;
        }
        cell[axis] += dir[axis].signum();
        cross[axis] += each[axis];
        out.push((cell[0] as i32, cell[1] as i32, cell[2] as i32));
    }
}

/// How wide and tall an entity's box is, by the game's data; a player's when it doesn't say.
fn entity_size(kind: &str, sizes: &mut HashMap<String, (f64, f64)>) -> (f64, f64) {
    *sizes.entry(kind.to_owned()).or_insert_with(|| {
        let facts = query(json!({ "op": "mob_type", "id": kind }));
        (facts["width"].as_f64().unwrap_or(0.6), facts["height"].as_f64().unwrap_or(1.8))
    })
}

impl World {
    fn ask(&self, op: &str, more: Value) -> Value {
        let mut request = json!({ "op": op, "dimension": self.dimension() });
        if let (Some(request), Some(more)) = (request.as_object_mut(), more.as_object()) {
            request.extend(more.iter().map(|(key, value)| (key.clone(), value.clone())));
        }
        query(request)
    }
    fn counts(&self) -> Value {
        self.ask("core_level_counts", json!({}))
    }
    /// bukkit: org/bukkit/World.getKeepSpawnInMemory
    ///
    /// False, as on Paper: the game has had no spawn chunks since 1.21.9.
    pub fn get_keep_spawn_in_memory(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/World.setKeepSpawnInMemory
    ///
    /// Does nothing, as `World`'s own default in Paper's API.
    pub fn set_keep_spawn_in_memory(&self, _keep: bool) {}
    /// bukkit: org/bukkit/World.regenerateChunk
    ///
    /// Nothing regenerates a chunk, on Paper either: `false` where it throws.
    pub fn regenerate_chunk(&self, _x: i32, _z: i32) -> bool {
        false
    }
    /// bukkit: org/bukkit/World.refreshChunk
    pub fn refresh_chunk(&self, x: i32, z: i32) -> bool {
        self.ask("core_refresh_chunk", json!({ "chunk": [x, z] })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/World.generateTree, org/bukkit/RegionAccessor.generateTree
    ///
    /// `tree` is a `TreeType` name: the game's own feature for it, at `at`, in a loaded area.
    pub fn generate_tree(&self, at: &Location, tree: &str) -> bool {
        tree_feature(tree).is_some_and(|feature| self.ask("core_place_feature", json!({ "pos": [at.get_block_x(), at.get_block_y(), at.get_block_z()], "feature": feature })).as_bool().unwrap_or(false))
    }
    /// bukkit: org/bukkit/World.getWorldType
    pub fn get_world_type(&self) -> &'static str {
        match crate::server::get_world_type().trim_start_matches("minecraft:") {
            "flat" => "FLAT",
            "large_biomes" => "LARGE_BIOMES",
            "amplified" => "AMPLIFIED",
            "single_biome_surface" => "SINGLE_BIOME_SURFACE",
            "debug_all_block_states" => "DEBUG_ALL_BLOCK_STATES",
            _ => "NORMAL",
        }
    }
    /// bukkit: org/bukkit/World.getSpawnLimit
    ///
    /// `category` is a `SpawnCategory` name. The cap the game spawns to.
    pub fn get_spawn_limit(&self, category: &str) -> i32 {
        if let Some(limit) = query(json!({ "op": "core_level_limit", "dimension": self.dimension(), "category": mob_category(category) })).as_i64() {
            return limit as i32;
        }
        get_spawn_limit(category)
    }
    /// bukkit: org/bukkit/World.getTicksPerSpawns
    pub fn get_ticks_per_spawns(&self, category: &str) -> i64 {
        ticks_per_spawns_of(category)
    }
    /// bukkit: org/bukkit/World.getMonsterSpawnLimit
    pub fn get_monster_spawn_limit(&self) -> i32 {
        spawn_limit_of("MONSTER")
    }
    /// bukkit: org/bukkit/World.getAnimalSpawnLimit
    pub fn get_animal_spawn_limit(&self) -> i32 {
        spawn_limit_of("ANIMAL")
    }
    /// bukkit: org/bukkit/World.getWaterAnimalSpawnLimit
    pub fn get_water_animal_spawn_limit(&self) -> i32 {
        spawn_limit_of("WATER_ANIMAL")
    }
    /// bukkit: org/bukkit/World.getWaterAmbientSpawnLimit
    pub fn get_water_ambient_spawn_limit(&self) -> i32 {
        spawn_limit_of("WATER_AMBIENT")
    }
    /// bukkit: org/bukkit/World.getWaterUndergroundCreatureSpawnLimit
    pub fn get_water_underground_creature_spawn_limit(&self) -> i32 {
        spawn_limit_of("WATER_UNDERGROUND_CREATURE")
    }
    /// bukkit: org/bukkit/World.getAmbientSpawnLimit
    pub fn get_ambient_spawn_limit(&self) -> i32 {
        spawn_limit_of("AMBIENT")
    }
    /// bukkit: org/bukkit/World.getTicksPerAnimalSpawns
    pub fn get_ticks_per_animal_spawns(&self) -> i64 {
        ticks_per_spawns_of("ANIMAL")
    }
    /// bukkit: org/bukkit/World.getTicksPerMonsterSpawns
    pub fn get_ticks_per_monster_spawns(&self) -> i64 {
        ticks_per_spawns_of("MONSTER")
    }
    /// bukkit: org/bukkit/World.getTicksPerWaterSpawns
    pub fn get_ticks_per_water_spawns(&self) -> i64 {
        ticks_per_spawns_of("WATER_ANIMAL")
    }
    /// bukkit: org/bukkit/World.getTicksPerWaterAmbientSpawns
    pub fn get_ticks_per_water_ambient_spawns(&self) -> i64 {
        ticks_per_spawns_of("WATER_AMBIENT")
    }
    /// bukkit: org/bukkit/World.getTicksPerWaterUndergroundCreatureSpawns
    pub fn get_ticks_per_water_underground_creature_spawns(&self) -> i64 {
        ticks_per_spawns_of("WATER_UNDERGROUND_CREATURE")
    }
    /// bukkit: org/bukkit/World.getTicksPerAmbientSpawns
    pub fn get_ticks_per_ambient_spawns(&self) -> i64 {
        ticks_per_spawns_of("AMBIENT")
    }
    /// bukkit: org/bukkit/World.getGameRules
    pub fn get_game_rules(&self) -> Vec<String> {
        query(json!({ "op": "core_game_rules" })).as_object().map_or_else(Vec::new, |rules| rules.keys().cloned().collect())
    }
    /// bukkit: org/bukkit/World.getGameRuleDefault
    pub fn get_game_rule_default(&self, rule: &str) -> Option<i64> {
        query(json!({ "op": "core_game_rules" }))[rule.trim_start_matches("minecraft:")]["default"].as_i64()
    }
    /// bukkit: org/bukkit/World.setBiome, org/bukkit/RegionAccessor.setBiome
    ///
    /// The biome cell of the block, four blocks a side, as the game keeps them.
    pub fn set_biome(&self, x: i32, y: i32, z: i32, biome: &str) {
        run_command(&format!("execute in {} run fillbiome {x} {y} {z} {x} {y} {z} {}", self.dimension(), key_of(biome)));
    }
    /// Java's `setBiome(x, z, biome)`: the whole column.
    pub fn set_biome_column(&self, x: i32, z: i32, biome: &str) {
        run_command(&format!("execute in {} run fillbiome {x} {} {z} {x} {} {z} {}", self.dimension(), self.get_min_height(), self.get_max_height() - 1, key_of(biome)));
    }
    /// bukkit: org/bukkit/World.getNearbyEntitiesByType
    ///
    /// By the game's id, in a cube of `radius` around `at`.
    pub fn get_nearby_entities_by_type(&self, kind: &str, at: &Location, radius: f64) -> Vec<Entity> {
        let kind = key_of(kind);
        self.get_nearby_entities(at, radius, radius, radius).into_iter().filter(|entity| entity.get_type() == kind).collect()
    }
    /// bukkit: org/bukkit/World.getEntitiesByClasses, org/bukkit/RegionAccessor.getEntitiesByClasses
    pub fn get_entities_by_classes(&self, kinds: &[&str]) -> Vec<Entity> {
        let kinds: Vec<String> = kinds.iter().map(|kind| key_of(kind)).collect();
        self.get_entities().into_iter().filter(|entity| kinds.contains(&entity.get_type())).collect()
    }
    /// bukkit: org/bukkit/World.audiences
    pub fn audiences(&self) -> Vec<Player> {
        self.get_players()
    }
    /// bukkit: org/bukkit/World.getTemperature
    pub fn get_temperature(&self, x: i32, y: i32, z: i32) -> f64 {
        self.get_block_at(x, y, z).get_temperature()
    }
    /// bukkit: org/bukkit/World.getHumidity
    pub fn get_humidity(&self, x: i32, y: i32, z: i32) -> f64 {
        self.get_block_at(x, y, z).get_humidity()
    }
    /// bukkit: org/bukkit/World.getCoordinateScale
    pub fn get_coordinate_scale(&self) -> f64 {
        if self.dimension().ends_with("nether") { 8.0 } else { 1.0 }
    }
    /// bukkit: org/bukkit/World.isFixedTime
    pub fn is_fixed_time(&self) -> bool {
        !self.dimension().ends_with("overworld")
    }
    /// bukkit: org/bukkit/World.getInfiniburn
    pub fn get_infiniburn(&self) -> Vec<String> {
        let tag = match self.dimension() {
            dimension if dimension.ends_with("nether") => "infiniburn_nether",
            dimension if dimension.ends_with("end") => "infiniburn_end",
            _ => "infiniburn_overworld",
        };
        crate::registry::Tag::get("block", tag).get_values()
    }
    /// bukkit: org/bukkit/World.getChunkCount
    pub fn get_chunk_count(&self) -> i32 {
        int(&self.counts()["chunks"])
    }
    /// bukkit: org/bukkit/World.getTileEntityCount
    pub fn get_tile_entity_count(&self) -> i32 {
        int(&self.counts()["block_entities"])
    }
    /// bukkit: org/bukkit/World.isChunkInUse
    pub fn is_chunk_in_use(&self, x: i32, z: i32) -> bool {
        !self.get_chunk_at(x, z).get_players_seeing_chunk().is_empty()
    }
    /// bukkit: org/bukkit/World.getPlayersSeeingChunk
    pub fn get_players_seeing_chunk(&self, chunk: &Chunk) -> Vec<Player> {
        chunk.get_players_seeing_chunk()
    }
    /// bukkit: org/bukkit/World.hasBonusChest
    pub fn has_bonus_chest(&self) -> bool {
        query(json!({ "op": "core_world_gen" }))["bonus_chest"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/World.canGenerateStructures
    pub fn can_generate_structures(&self) -> bool {
        query(json!({ "op": "core_world_gen" }))["structures"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/World.setHardcore
    ///
    /// Players learn of it as they join, which is when the game tells them.
    pub fn set_hardcore(&self, hardcore: bool) {
        query(json!({ "op": "core_set_hardcore", "hardcore": hardcore }));
    }
    pub(crate) fn level_distance(&self, which: &str) -> i32 {
        int(&self.ask("core_level_distance", json!({ "which": which })))
    }
    fn set_level_distance(&self, which: &str, distance: i32) {
        self.ask("core_level_distance", json!({ "which": which, "distance": distance }));
    }
    /// bukkit: org/bukkit/World.setViewDistance, org/bukkit/World.setNoTickViewDistance
    pub fn set_view_distance(&self, distance: i32) {
        self.set_level_distance("view", distance);
    }
    /// bukkit: org/bukkit/World.setSendViewDistance
    ///
    /// -1 goes back to the view distance. Never past it: chunks past the view aren't kept here.
    pub fn set_send_view_distance(&self, distance: i32) {
        self.set_level_distance("send", distance);
    }
    /// bukkit: org/bukkit/World.setSimulationDistance
    pub fn set_simulation_distance(&self, distance: i32) {
        self.set_level_distance("simulation", distance);
    }
    /// bukkit: org/bukkit/World.getSendViewDistance
    pub fn get_send_view_distance(&self) -> i32 {
        self.level_distance("send")
    }
    /// bukkit: org/bukkit/World.getNoTickViewDistance
    pub fn get_no_tick_view_distance(&self) -> i32 {
        self.get_view_distance()
    }
    /// bukkit: org/bukkit/World.playNote
    ///
    /// `instrument` is an `Instrument` name.
    pub fn play_note(&self, at: &Location, instrument: &str, note: &crate::core2::Note) {
        if let Some(sound) = instrument_sound(instrument) {
            run_command(&format!("execute in {} run playsound {sound} record @a {} {} {} 3 {}", self.dimension(), at.x, at.y, at.z, note.get_pitch()));
        }
    }
    /// bukkit: org/bukkit/World.getIntersectingChunks
    pub fn get_intersecting_chunks(&self, bounds: &BoundingBox) -> Vec<Chunk> {
        let (x0, x1) = ((bounds.get_min_x().floor() as i32) >> 4, ((bounds.get_max_x().ceil() as i32) - 1) >> 4);
        let (z0, z1) = ((bounds.get_min_z().floor() as i32) >> 4, ((bounds.get_max_z().ceil() as i32) - 1) >> 4);
        (x0..=x1).flat_map(|x| (z0..=z1).map(move |z| (x, z))).map(|(x, z)| self.get_chunk_at(x, z)).collect()
    }
    /// bukkit: org/bukkit/World.getBlockAtKey
    pub fn get_block_at_key(&self, key: i64) -> Block {
        self.get_block_at(Block::get_block_key_x(key), Block::get_block_key_y(key), Block::get_block_key_z(key))
    }
    /// bukkit: org/bukkit/World.getLocationAtKey
    pub fn get_location_at_key(&self, key: i64) -> Location {
        Location::new(self.clone(), f64::from(Block::get_block_key_x(key)), f64::from(Block::get_block_key_y(key)), f64::from(Block::get_block_key_z(key)), 0.0, 0.0)
    }
    /// bukkit: org/bukkit/World.isPositionLoaded
    pub fn is_position_loaded(&self, position: &Position) -> bool {
        self.is_chunk_loaded(position.block_x() >> 4, position.block_z() >> 4)
    }
    /// bukkit: org/bukkit/World.getChunkGenerationState, org/bukkit/World.getChunkGenerationStateAsync
    ///
    /// `GENERATED` for what's loaded or saved, `NOT_GENERATED` for the rest; a call on the server thread.
    pub fn get_chunk_generation_state(&self, x: i32, z: i32) -> &'static str {
        if self.get_chunk_at(x, z).is_generated() { "GENERATED" } else { "NOT_GENERATED" }
    }
    /// bukkit: org/bukkit/World.getChunksAtAsync
    ///
    /// Each chunk not yet loaded is held while it loads, then `done` runs on the server thread.
    pub fn get_chunks_at_async(&self, min_x: i32, min_z: i32, max_x: i32, max_z: i32, _urgent: bool, done: impl FnOnce() + Send + 'static) {
        let wanted: Vec<(i32, i32)> = (min_x..=max_x).flat_map(|x| (min_z..=max_z).map(move |z| (x, z))).collect();
        let held: Vec<(i32, i32)> = wanted.iter().copied().filter(|(x, z)| !self.is_chunk_loaded(*x, *z)).collect();
        if held.is_empty() {
            done();
            return;
        }
        held.iter().for_each(|(x, z)| self.set_chunk_force_loaded(*x, *z, true));
        let world = self.clone();
        let mut done = Some(done);
        crate::scheduler::run_task_timer(1, 1, move || {
            if held.iter().all(|(x, z)| world.is_chunk_loaded(*x, *z)) {
                held.iter().for_each(|(x, z)| world.set_chunk_force_loaded(*x, *z, false));
                if let Some(done) = done.take() {
                    done();
                }
                crate::scheduler::cancel_self();
            }
        });
    }
    /// bukkit: org/bukkit/World.rayTrace
    ///
    /// Blocks by their shapes and entities by their boxes, `ray_size` wider; whichever comes first.
    /// `fluids` is a `FluidCollisionMode` name, `filter` says which entities count.
    pub fn ray_trace(&self, start: &Location, direction: &Vector, distance: f64, fluids: &str, ignore_passable: bool, ray_size: f64, filter: impl Fn(&Entity) -> bool) -> Option<RayTraceResult> {
        let origin = start.to_vector();
        let mut way = *direction;
        way.normalize();
        let block = self.ray_trace_cells(&origin, &way, distance, fluids, ignore_passable);
        let reach = block.as_ref().map_or(distance, |hit| hit.position.distance(&origin));
        let mut sizes = HashMap::new();
        let mut best: Option<(f64, RayTraceResult)> = None;
        for entity in self.get_entities().into_iter().filter(|entity| filter(entity)) {
            let Some(at) = entity.get_location() else {
                continue;
            };
            let (width, height) = entity_size(&entity.get_type(), &mut sizes);
            let half = width / 2.0 + ray_size;
            let bounds = BoundingBox::new(at.x - half, at.y - ray_size, at.z - half, at.x + half, at.y + height + ray_size, at.z + half);
            if let Some(mut hit) = bounds.ray_trace(&origin, &way, reach) {
                let along = hit.position.distance(&origin);
                if best.as_ref().is_none_or(|(known, _)| along < *known) {
                    hit.entity = Some(entity);
                    best = Some((along, hit));
                }
            }
        }
        best.map(|(_, hit)| hit).or(block)
    }
    fn ray_trace_cells(&self, origin: &Vector, way: &Vector, distance: f64, fluids: &str, ignore_passable: bool) -> Option<RayTraceResult> {
        for (x, y, z) in cells_along(origin, way, distance) {
            let block = self.get_block_at(x, y, z);
            let facts = block.get_facts();
            let (fx, fy, fz) = (f64::from(x), f64::from(y), f64::from(z));
            let liquid = facts["liquid"].as_bool().unwrap_or(false);
            let wants_fluid = liquid && (fluids == "ALWAYS" || (fluids == "SOURCE_ONLY" && block.get_property("level").is_none_or(|level| level == "0")));
            let mut shapes: Vec<BoundingBox> = Vec::new();
            if wants_fluid {
                shapes.push(BoundingBox::new(fx, fy, fz, fx + 1.0, fy + 1.0, fz + 1.0));
            }
            if !ignore_passable || facts["full_collision"].as_bool().unwrap_or(false) {
                for row in facts["shape"].as_array().into_iter().flatten() {
                    let side = |i: usize| row[i].as_f64().unwrap_or(0.0);
                    shapes.push(BoundingBox::new(fx + side(0), fy + side(1), fz + side(2), fx + side(3), fy + side(4), fz + side(5)));
                }
            }
            let hit = shapes.iter().filter_map(|shape| shape.ray_trace(origin, way, distance)).min_by(|first, second| first.position.distance_squared(origin).total_cmp(&second.position.distance_squared(origin)));
            if let Some(mut hit) = hit {
                hit.block = Some(block);
                return Some(hit);
            }
        }
        None
    }
    /// bukkit: org/bukkit/World.spigot, org/bukkit/World$Spigot.strikeLightning
    pub fn spigot_strike_lightning(&self, at: &Location, silent: bool) {
        if silent {
            query(json!({ "op": "core_silent_bolt", "at": [at.x, at.y, at.z] }));
        }
        self.strike_lightning(at);
    }
    /// bukkit: org/bukkit/World$Spigot.strikeLightningEffect
    pub fn spigot_strike_lightning_effect(&self, at: &Location, silent: bool) {
        if silent {
            query(json!({ "op": "core_silent_bolt", "at": [at.x, at.y, at.z] }));
        }
        self.strike_lightning_effect(at);
    }
    /// bukkit: org/bukkit/World.locateNearestStructure
    ///
    /// `structure` is a structure's key, or a `StructureType` name for all of that kind. `radius` is
    /// in chunks, as `/locate` goes.
    pub fn locate_nearest_structure(&self, origin: &Location, structure: &str, radius: i32, find_unexplored: bool) -> Option<StructureSearchResult> {
        let names = structure_ids(structure);
        let found = self.ask("core_locate_structure", json!({ "names": names, "from": [origin.get_block_x(), origin.get_block_y(), origin.get_block_z()], "radius": radius, "find_unexplored": find_unexplored }));
        let location = at(&found["pos"], self)?;
        Some(StructureSearchResult { structure: found["name"].as_str()?.to_owned(), location })
    }
    /// bukkit: org/bukkit/World.locateNearestBiome
    pub fn locate_nearest_biome(&self, origin: &Location, radius: i32, horizontal: i32, vertical: i32, biomes: &[&str]) -> Option<BiomeSearchResult> {
        let names: Vec<String> = biomes.iter().map(|biome| key_of(biome)).collect();
        let found = self.ask("core_locate_biome", json!({ "names": names, "from": [origin.get_block_x(), origin.get_block_y(), origin.get_block_z()], "radius": radius, "horizontal": horizontal, "vertical": vertical }));
        let location = at(&found["pos"], self)?;
        Some(BiomeSearchResult { biome: found["name"].as_str()?.to_owned(), location })
    }
    /// bukkit: org/bukkit/World.locateNearestPoi
    ///
    /// `poi` is the game's id. Among loaded chunks, as `/locate poi` looks.
    pub fn locate_nearest_poi(&self, origin: &Location, poi: &str, radius: i32) -> Option<Location> {
        self.locate_all_poi_in_range(origin, radius).into_iter().filter(|found| found.get_poi_type() == key_of(poi)).map(|found| found.location).next()
    }
    /// bukkit: org/bukkit/World.locateAllPoiInRange
    ///
    /// Nearest first; `.filter` is Java's predicate.
    pub fn locate_all_poi_in_range(&self, origin: &Location, radius: i32) -> Vec<PoiSearchResult> {
        let found = self.ask("core_pois", json!({ "from": [origin.get_block_x(), origin.get_block_y(), origin.get_block_z()], "radius": radius }));
        found.as_array().into_iter().flatten().filter_map(|row| Some(PoiSearchResult { kind: row["kind"].as_str()?.to_owned(), location: at(&row["pos"], self)? })).collect()
    }
    /// bukkit: org/bukkit/World.findLightningRod
    pub fn find_lightning_rod(&self, near: &Location) -> Option<Location> {
        self.locate_all_poi_in_range(near, 128).into_iter().filter(|found| found.get_poi_type() == "minecraft:lightning_rod").filter(|found| self.get_highest_block_y_at(found.location.get_block_x(), found.location.get_block_z()) == found.location.get_block_y()).map(|found| found.location).next()
    }
    /// bukkit: org/bukkit/World.findLightningTarget
    pub fn find_lightning_target(&self, near: &Location) -> Option<Location> {
        let found = self.ask("core_lightning_target", json!({ "x": near.get_block_x(), "z": near.get_block_z() }));
        let at = |side: usize| found[side].as_f64();
        Some(Location::new(self.clone(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/World.hasStructureAt
    pub fn has_structure_at(&self, position: &Position, structure: &str) -> bool {
        let key = key_of(structure);
        self.structures_at(position).iter().any(|name| *name == key)
    }
    fn structures_at(&self, position: &Position) -> Vec<String> {
        let found = self.ask("core_structures_at", json!({ "pos": [position.block_x(), position.block_y(), position.block_z()] }));
        found.as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/World.getStructures
    ///
    /// The structures that reach a chunk.
    pub fn get_structures(&self, x: i32, z: i32) -> Vec<GeneratedStructure> {
        self.ask("core_chunk_structures", json!({ "chunk": [x, z] })).as_array().into_iter().flatten().filter_map(GeneratedStructure::read).collect()
    }
}

/// bukkit: org/bukkit/World$Environment.getId
pub fn environment_id(environment: &str) -> i32 {
    match environment {
        "NORMAL" => 0,
        "NETHER" => -1,
        "THE_END" => 1,
        _ => -999,
    }
}

/// bukkit: org/bukkit/World$Environment.getEnvironment
pub fn environment_by_id(id: i32) -> Option<&'static str> {
    crate::enums::world_environment::values().iter().copied().find(|environment| environment_id(environment) == id)
}

/// bukkit: org/bukkit/WorldType.getByName
pub fn world_type_by_name(name: &str) -> Option<&'static str> {
    crate::enums::world_type::values().iter().copied().find(|kind| world_type_name(kind).eq_ignore_ascii_case(name))
}

/// bukkit: org/bukkit/WorldType.getName
pub fn world_type_name(kind: &str) -> &str {
    match kind {
        "NORMAL" => "DEFAULT",
        "LARGE_BIOMES" => "LARGEBIOMES",
        "DEBUG_ALL_BLOCK_STATES" => "DEBUG",
        other => other,
    }
}

// ---- What a search finds ----

/// `org.bukkit.util.StructureSearchResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct StructureSearchResult {
    structure: String,
    location: Location,
}

impl StructureSearchResult {
    /// bukkit: org/bukkit/util/StructureSearchResult.getStructure
    pub fn get_structure(&self) -> &str {
        &self.structure
    }
    /// bukkit: org/bukkit/util/StructureSearchResult.getLocation
    pub fn get_location(&self) -> &Location {
        &self.location
    }
}

/// `org.bukkit.util.BiomeSearchResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct BiomeSearchResult {
    biome: String,
    location: Location,
}

impl BiomeSearchResult {
    /// bukkit: org/bukkit/util/BiomeSearchResult.getBiome
    pub fn get_biome(&self) -> &str {
        &self.biome
    }
    /// bukkit: org/bukkit/util/BiomeSearchResult.getLocation
    pub fn get_location(&self) -> &Location {
        &self.location
    }
}

/// `io.papermc.paper.util.PoiSearchResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct PoiSearchResult {
    kind: String,
    location: Location,
}

impl PoiSearchResult {
    pub fn get_poi_type(&self) -> &str {
        &self.kind
    }
    pub fn get_location(&self) -> &Location {
        &self.location
    }
}

/// `org.bukkit.generator.structure.GeneratedStructure`.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedStructure {
    structure: String,
    bounds: BoundingBox,
    pieces: Vec<BoundingBox>,
}

fn saved_box(row: &Value) -> Option<BoundingBox> {
    let side = |i: usize| row[i].as_f64();
    // Both corners are blocks in the game's box; Bukkit's reaches past the last one.
    Some(BoundingBox::new(side(0)?, side(1)?, side(2)?, side(3)? + 1.0, side(4)? + 1.0, side(5)? + 1.0))
}

impl GeneratedStructure {
    fn read(row: &Value) -> Option<Self> {
        Some(Self { structure: row["name"].as_str()?.to_owned(), bounds: saved_box(&row["box"])?, pieces: row["pieces"].as_array()?.iter().filter_map(saved_box).collect() })
    }
    /// bukkit: org/bukkit/generator/structure/GeneratedStructure.getStructure
    pub fn get_structure(&self) -> &str {
        &self.structure
    }
    /// bukkit: org/bukkit/generator/structure/GeneratedStructure.getBoundingBox
    pub fn get_bounding_box(&self) -> &BoundingBox {
        &self.bounds
    }
    /// bukkit: org/bukkit/generator/structure/GeneratedStructure.getPieces
    pub fn get_pieces(&self) -> Vec<StructurePiece> {
        self.pieces.iter().map(|bounds| StructurePiece { bounds: bounds.clone() }).collect()
    }
}

/// `org.bukkit.generator.structure.StructurePiece`.
#[derive(Clone, Debug, PartialEq)]
pub struct StructurePiece {
    bounds: BoundingBox,
}

impl StructurePiece {
    /// bukkit: org/bukkit/generator/structure/StructurePiece.getBoundingBox
    pub fn get_bounding_box(&self) -> &BoundingBox {
        &self.bounds
    }
}

/// `org.bukkit.StructureType`: the old names, each for what the game calls a family of structures.
const STRUCTURE_TYPES: [(&str, &str, &[&str]); 18] = [
    ("MINESHAFT", "RED_X", &["minecraft:mineshaft", "minecraft:mineshaft_mesa"]),
    ("VILLAGE", "MANSION", &["minecraft:village_plains", "minecraft:village_desert", "minecraft:village_savanna", "minecraft:village_snowy", "minecraft:village_taiga"]),
    ("FORTRESS", "RED_X", &["minecraft:fortress"]),
    ("STRONGHOLD", "MANSION", &["minecraft:stronghold"]),
    ("JUNGLE_PYRAMID", "RED_X", &["minecraft:jungle_pyramid"]),
    ("OCEAN_RUIN", "MONUMENT", &["minecraft:ocean_ruin_cold", "minecraft:ocean_ruin_warm"]),
    ("DESERT_PYRAMID", "RED_X", &["minecraft:desert_pyramid"]),
    ("IGLOO", "RED_X", &["minecraft:igloo"]),
    ("SWAMP_HUT", "RED_X", &["minecraft:swamp_hut"]),
    ("MONUMENT", "MONUMENT", &["minecraft:monument"]),
    ("END_CITY", "RED_X", &["minecraft:end_city"]),
    ("MANSION", "MANSION", &["minecraft:mansion"]),
    ("BURIED_TREASURE", "RED_X", &["minecraft:buried_treasure"]),
    ("SHIPWRECK", "RED_X", &["minecraft:shipwreck", "minecraft:shipwreck_beached"]),
    ("PILLAGER_OUTPOST", "RED_X", &["minecraft:pillager_outpost"]),
    ("NETHER_FOSSIL", "RED_X", &["minecraft:nether_fossil"]),
    ("RUINED_PORTAL", "RED_X", &["minecraft:ruined_portal", "minecraft:ruined_portal_desert", "minecraft:ruined_portal_jungle", "minecraft:ruined_portal_mountain", "minecraft:ruined_portal_nether", "minecraft:ruined_portal_ocean", "minecraft:ruined_portal_swamp"]),
    ("BASTION_REMNANT", "RED_X", &["minecraft:bastion_remnant"]),
];

fn structure_ids(name: &str) -> Vec<String> {
    let upper = name.trim_start_matches("minecraft:").to_ascii_uppercase();
    let upper = match upper.as_str() {
        "NETHER_FORTRESS" => "FORTRESS",
        "OCEAN_MONUMENT" => "MONUMENT",
        "WOODLAND_MANSION" => "MANSION",
        other => other,
    }
    .to_owned();
    STRUCTURE_TYPES.iter().find(|(known, ..)| *known == upper).map_or_else(|| vec![key_of(name)], |(_, _, ids)| ids.iter().map(|id| (*id).to_owned()).collect())
}

/// bukkit: org/bukkit/StructureType.getStructureTypes
pub fn get_structure_types() -> Vec<String> {
    STRUCTURE_TYPES.iter().map(|(name, ..)| name.to_ascii_lowercase()).collect()
}

/// bukkit: org/bukkit/StructureType.getName
pub fn structure_type_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// bukkit: org/bukkit/StructureType.getKey
pub fn structure_type_key(name: &str) -> NamespacedKey {
    NamespacedKey::minecraft(&name.to_ascii_lowercase())
}

/// bukkit: org/bukkit/StructureType.getMapIcon
pub fn structure_type_map_icon(name: &str) -> Option<&'static str> {
    let upper = name.to_ascii_uppercase();
    STRUCTURE_TYPES.iter().find(|(known, ..)| *known == upper).map(|(_, icon, _)| *icon)
}

// ---- Chunks ----

/// Tickets this plugin took through `Chunk`, by level and chunk.

impl Chunk {
    fn ask(&self, op: &str) -> Value {
        query(json!({ "op": op, "dimension": self.get_world().dimension(), "chunk": [self.get_x(), self.get_z()] }))
    }
    fn ticket(&self, change: &str) -> Value {
        let plugin = get_own_plugin().map(|own| own.get_name().to_owned()).unwrap_or_default();
        query(json!({ "op": "core_ticket", "change": change, "plugin": plugin, "dimension": self.get_world().dimension(), "chunk": [self.get_x(), self.get_z()] }))
    }
    /// bukkit: org/bukkit/Chunk.addPluginChunkTicket
    ///
    /// Loads and ticks as a forced chunk does, without being one of `/forceload`'s. Gone with the
    /// plugin, or the server.
    pub fn add_plugin_chunk_ticket(&self) -> bool {
        self.ticket("add").as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/Chunk.removePluginChunkTicket
    pub fn remove_plugin_chunk_ticket(&self) -> bool {
        self.ticket("remove").as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/Chunk.getPluginChunkTickets
    ///
    /// Every plugin holding it.
    pub fn get_plugin_chunk_tickets(&self) -> Vec<String> {
        self.ticket("holders").as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect()
    }
    /// Whether this plugin holds it.
    pub fn has_plugin_chunk_ticket(&self) -> bool {
        get_own_plugin().is_some_and(|own| self.get_plugin_chunk_tickets().iter().any(|name| name == own.get_name()))
    }
    /// bukkit: org/bukkit/Chunk.isGenerated
    pub fn is_generated(&self) -> bool {
        self.is_loaded() || self.ask("core_chunk_saved").as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/Chunk.getLoadLevel
    pub fn get_load_level(&self) -> &'static str {
        let facts = self.ask("core_chunk");
        if !facts["loaded"].as_bool().unwrap_or(false) {
            "UNLOADED"
        } else if facts["ticking"].as_bool().unwrap_or(false) {
            "ENTITY_TICKING"
        } else {
            "BORDER"
        }
    }
    /// bukkit: org/bukkit/Chunk.getInhabitedTime
    pub fn get_inhabited_time(&self) -> i64 {
        self.ask("core_chunk")["inhabited"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/Chunk.setInhabitedTime
    pub fn set_inhabited_time(&self, ticks: i64) {
        query(json!({ "op": "core_set_inhabited", "dimension": self.get_world().dimension(), "chunk": [self.get_x(), self.get_z()], "time": ticks }));
    }
    /// bukkit: org/bukkit/Chunk.isEntitiesLoaded
    pub fn is_entities_loaded(&self) -> bool {
        self.is_loaded()
    }
    /// bukkit: org/bukkit/Chunk.contains
    ///
    /// `block` as a state string: `minecraft:stone`, or with its properties.
    pub fn contains(&self, block: &str) -> bool {
        self.get_chunk_snapshot().is_some_and(|snapshot| snapshot.contains(block))
    }
    /// bukkit: org/bukkit/Chunk.getPlayersSeeingChunk
    pub fn get_players_seeing_chunk(&self) -> Vec<Player> {
        self.ask("core_chunk_viewers").as_array().into_iter().flatten().filter_map(|uuid| Player::read(&query(json!({ "op": "player", "uuid": uuid })))).collect()
    }
    /// bukkit: org/bukkit/Chunk.getStructures
    pub fn get_structures(&self) -> Vec<GeneratedStructure> {
        self.get_world().get_structures(self.get_x(), self.get_z())
    }
}

/// bukkit: org/bukkit/World.getPluginChunkTickets
///
/// Every plugin's, in this level, as its name and the chunks.
pub fn get_plugin_chunk_tickets(world: &World) -> BTreeMap<String, Vec<(i32, i32)>> {
    let mut held: BTreeMap<String, Vec<(i32, i32)>> = BTreeMap::new();
    for row in query(json!({ "op": "core_ticket", "change": "all" })).as_array().into_iter().flatten() {
        if row[0].as_str() == Some(world.dimension()) {
            let chunk = (row[1].as_i64().unwrap_or(0) as i32, row[2].as_i64().unwrap_or(0) as i32);
            held.entry(row[3].as_str().unwrap_or_default().to_owned()).or_default().push(chunk);
        }
    }
    held
}

/// bukkit: org/bukkit/World.removePluginChunkTickets
///
/// This plugin's, in every level.
pub fn remove_all_plugin_chunk_tickets() {
    let Some(own) = get_own_plugin() else {
        return;
    };
    for row in query(json!({ "op": "core_ticket", "change": "all" })).as_array().into_iter().flatten() {
        if row[3].as_str() == Some(own.get_name()) {
            World::of(row[0].as_str().unwrap_or_default()).get_chunk_at(row[1].as_i64().unwrap_or(0) as i32, row[2].as_i64().unwrap_or(0) as i32).remove_plugin_chunk_ticket();
        }
    }
}

fn instrument_sound(instrument: &str) -> Option<&'static str> {
    Some(match instrument {
        "PIANO" => "minecraft:block.note_block.harp",
        "BASS_DRUM" => "minecraft:block.note_block.basedrum",
        "SNARE_DRUM" => "minecraft:block.note_block.snare",
        "STICKS" => "minecraft:block.note_block.hat",
        "BASS_GUITAR" => "minecraft:block.note_block.bass",
        "FLUTE" => "minecraft:block.note_block.flute",
        "BELL" => "minecraft:block.note_block.bell",
        "GUITAR" => "minecraft:block.note_block.guitar",
        "CHIME" => "minecraft:block.note_block.chime",
        "XYLOPHONE" => "minecraft:block.note_block.xylophone",
        "IRON_XYLOPHONE" => "minecraft:block.note_block.iron_xylophone",
        "COW_BELL" => "minecraft:block.note_block.cow_bell",
        "DIDGERIDOO" => "minecraft:block.note_block.didgeridoo",
        "BIT" => "minecraft:block.note_block.bit",
        "BANJO" => "minecraft:block.note_block.banjo",
        "PLING" => "minecraft:block.note_block.pling",
        "TRUMPET" => "minecraft:block.note_block.trumpet",
        "TRUMPET_EXPOSED" => "minecraft:block.note_block.trumpet_exposed",
        "TRUMPET_OXIDIZED" => "minecraft:block.note_block.trumpet_oxidized",
        "TRUMPET_WEATHERED" => "minecraft:block.note_block.trumpet_weathered",
        "ZOMBIE" => "minecraft:block.note_block.imitate.zombie",
        "SKELETON" => "minecraft:block.note_block.imitate.skeleton",
        "CREEPER" => "minecraft:block.note_block.imitate.creeper",
        "DRAGON" => "minecraft:block.note_block.imitate.ender_dragon",
        "WITHER_SKELETON" => "minecraft:block.note_block.imitate.wither_skeleton",
        "PIGLIN" => "minecraft:block.note_block.imitate.piglin",
        _ => return None,
    })
}

/// bukkit: org/bukkit/Instrument.getSound
pub fn instrument_get_sound(instrument: &str) -> Option<&'static str> {
    instrument_sound(instrument)
}

/// bukkit: org/bukkit/Instrument.getType
pub fn instrument_get_type(instrument: &str) -> i8 {
    crate::enums::instrument::values().iter().position(|known| *known == instrument).map_or(0, |index| index as i8)
}

/// bukkit: org/bukkit/Instrument.getByType
pub fn instrument_by_type(kind: i8) -> Option<&'static str> {
    usize::try_from(kind).ok().and_then(|index| crate::enums::instrument::values().get(index).copied())
}

// ---- Maps ----

/// `org.bukkit.map.MapRenderer`: what draws on a map, a plugin's own.
pub trait MapRenderer: Send + Sync {
    /// bukkit: org/bukkit/map/MapRenderer.initialize
    fn initialize(&self, _map: &MapView) {}
    /// bukkit: org/bukkit/map/MapRenderer.render
    fn render(&self, map: &MapView, canvas: &mut crate::pure2::MapCanvas, player: &Player);
    /// bukkit: org/bukkit/map/MapRenderer.isContextual
    fn is_contextual(&self) -> bool {
        false
    }
    /// Whose this is, when it lives in another plugin: `MapView::get_renderers` gives those.
    #[doc(hidden)]
    fn foreign(&self) -> Option<(String, usize)> {
        None
    }
}

/// A renderer of another plugin. The work stays there, and a call crosses as JSON.
struct ForeignRenderer {
    plugin: String,
    key: usize,
    contextual: bool,
}

fn ask_about_map(plugin: &str, method: &str, map: i32, key: usize, player: Option<&Player>) -> Value {
    query(json!({ "op": "core_command_call", "plugin": plugin, "kind": "map", "method": method, "map": map, "key": key, "player": player.map(Player::uuid) }))
}

impl MapRenderer for ForeignRenderer {
    fn render(&self, map: &MapView, canvas: &mut crate::pure2::MapCanvas, player: &Player) {
        let drawn = ask_about_map(&self.plugin, "render", map.id, self.key, Some(player));
        for (index, color) in drawn.as_array().into_iter().flatten().enumerate() {
            if let Some(color) = color.as_u64() {
                canvas.set_pixel(index as i32 % 128, index as i32 / 128, color as u8);
            }
        }
    }
    fn is_contextual(&self) -> bool {
        self.contextual
    }
    fn foreign(&self) -> Option<(String, usize)> {
        Some((self.plugin.clone(), self.key))
    }
}

/// The other end of `ForeignRenderer`, and of `MapView::get_renderers`.
fn answer_map(value: &Value) -> Value {
    let map = MapView { id: int(&value["map"]) };
    let key = value["key"].as_u64().unwrap_or(0) as usize;
    let game = game_renderer();
    let list = map.renderers(|list| list.clone());
    let found = list.iter().find(|renderer| renderer_key(renderer) == key).cloned();
    match value["method"].as_str().unwrap_or_default() {
        "list" => list.iter().filter(|renderer| !same_renderer(renderer, &game)).map(|renderer| json!({ "key": renderer_key(renderer), "contextual": renderer.is_contextual() })).collect(),
        "remove" => found.map_or(json!(false), |renderer| json!(map.remove_renderer(&renderer))),
        "game_off" => {
            map.renderers(|list| list.retain(|renderer| !same_renderer(renderer, &game)));
            json!(true)
        }
        "render" => {
            let (Some(renderer), Some(player)) = (found, Player::read(&query(json!({ "op": "player", "uuid": value["player"] })))) else {
                return Value::Null;
            };
            let slot = (renderer_key(&renderer), renderer.is_contextual().then(|| player.get_unique_id()));
            let mut canvas = CANVASES.lock().unwrap_or_else(|e| e.into_inner()).remove(&slot).unwrap_or_default();
            renderer.render(&map, &mut canvas, &player);
            let drawn: Value = canvas.colors().into_iter().map(|color| json!(color)).collect();
            CANVASES.lock().unwrap_or_else(|e| e.into_inner()).insert(slot, canvas);
            drawn
        }
        _ => Value::Null,
    }
}

type Renderers = Vec<std::sync::Arc<dyn MapRenderer>>;

static RENDERERS: Mutex<BTreeMap<i32, Renderers>> = Mutex::new(BTreeMap::new());
/// What each player's client was last sent: picture, the game's own under it, cursors.
static SENT: Mutex<BTreeMap<(i32, String), (Vec<u8>, Vec<u8>, Vec<Value>)>> = Mutex::new(BTreeMap::new());
static RENDERING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The loop that draws maps: tasks go when a plugin is disabled, so it may need starting again.
fn start_rendering() {
    if !RENDERING.swap(true, std::sync::atomic::Ordering::Relaxed) {
        crate::scheduler::run_task_timer(1, 1, render_maps);
    }
}
/// A canvas per renderer, per player if it's contextual: what a renderer drew stays, as in Java,
/// which is how the common draw-once renderer works.
static CANVASES: Mutex<BTreeMap<(usize, Option<String>), crate::pure2::MapCanvas>> = Mutex::new(BTreeMap::new());

thread_local! {
    /// The map being drawn, for `MapCanvas.getMapView`.
    static CURRENT_VIEW: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
    /// The map's own picture and decorations this pass, for the game's renderer.
    static GAME_PICTURE: std::cell::RefCell<(Vec<u8>, Vec<Value>)> = const { std::cell::RefCell::new((Vec::new(), Vec::new())) };
}

/// `CraftMapRenderer`: the game's own picture and markers, first in every map's list.
struct GameMapRenderer;

impl MapRenderer for GameMapRenderer {
    fn render(&self, _map: &MapView, canvas: &mut crate::pure2::MapCanvas, _player: &Player) {
        GAME_PICTURE.with(|picture| {
            let picture = picture.borrow();
            for (index, color) in picture.0.iter().enumerate() {
                canvas.set_pixel((index % 128) as i32, (index / 128) as i32, *color);
            }
            while canvas.cursors.size() > 0 {
                let Some(first) = canvas.cursors.get_cursor(0).cloned() else {
                    break;
                };
                canvas.cursors.remove_cursor(&first);
            }
            for marker in &picture.1 {
                let kind = map_cursor_type_value(marker["type"].as_str().unwrap_or_default().trim_start_matches("minecraft:")).unwrap_or(0);
                canvas.cursors.add_cursor(crate::pure2::MapCursor::new(marker["x"].as_i64().unwrap_or(0) as i8, marker["y"].as_i64().unwrap_or(0) as i8, marker["direction"].as_u64().unwrap_or(0) as u8, kind, true));
            }
        });
    }
}

fn game_renderer() -> std::sync::Arc<dyn MapRenderer> {
    static GAME: std::sync::OnceLock<std::sync::Arc<dyn MapRenderer>> = std::sync::OnceLock::new();
    GAME.get_or_init(|| std::sync::Arc::new(GameMapRenderer)).clone()
}

fn same_renderer(one: &std::sync::Arc<dyn MapRenderer>, other: &std::sync::Arc<dyn MapRenderer>) -> bool {
    std::ptr::addr_eq(std::sync::Arc::as_ptr(one), std::sync::Arc::as_ptr(other))
}

fn renderer_key(renderer: &std::sync::Arc<dyn MapRenderer>) -> usize {
    std::sync::Arc::as_ptr(renderer).cast::<()>() as usize
}

/// Draws every map whose list isn't only the game's own, which the game sends by itself.
fn render_maps() {
    let maps: Vec<(i32, Renderers)> = RENDERERS.lock().unwrap_or_else(|e| e.into_inner()).iter().map(|(id, list)| (*id, list.clone())).collect();
    let game = game_renderer();
    for (id, renderers) in maps {
        if renderers.len() == 1 && same_renderer(&renderers[0], &game) {
            continue;
        }
        let holders = query(json!({ "op": "core_map_holders", "id": id }));
        let players: Vec<Player> = holders.as_array().into_iter().flatten().filter_map(|uuid| Player::read(&query(json!({ "op": "player", "uuid": uuid })))).collect();
        if players.is_empty() {
            continue;
        }
        let state = query(json!({ "op": "core_map", "id": id, "colors": true }));
        let own: Vec<u8> = state["colors"].as_array().into_iter().flatten().filter_map(|color| color.as_u64().map(|color| color as u8)).collect();
        let markers = state["decorations"].as_array().cloned().unwrap_or_default();
        GAME_PICTURE.with(|picture| *picture.borrow_mut() = (own.clone(), markers));
        let view = MapView { id };
        CURRENT_VIEW.with(|current| current.set(Some(id)));
        for player in players {
            let mut colors = vec![0_u8; 128 * 128];
            let mut cursors: Vec<Value> = Vec::new();
            for renderer in &renderers {
                let key = (renderer_key(renderer), renderer.is_contextual().then(|| player.get_unique_id()));
                let mut canvas = CANVASES.lock().unwrap_or_else(|e| e.into_inner()).remove(&key).unwrap_or_default();
                canvas.base.clone_from(&colors);
                renderer.render(&view, &mut canvas, &player);
                // The map palette stops at 207: what's past it is see-through, as in `CraftMapView.render`.
                for (out, drawn) in colors.iter_mut().zip(canvas.colors()) {
                    if drawn < 208 {
                        *out = drawn;
                    }
                }
                cursors.extend((0..canvas.cursors.size()).filter_map(|index| canvas.cursors.get_cursor(index)).filter(|cursor| cursor.is_visible()).map(|cursor| json!({ "type": map_cursor_type_key(cursor.get_raw_type()), "x": cursor.get_x(), "y": cursor.get_y(), "direction": cursor.get_direction(), "caption": cursor.get_caption().map(adventure::legacy).map(|caption| caption.0) })));
                CANVASES.lock().unwrap_or_else(|e| e.into_inner()).insert(key, canvas);
            }
            let key = (id, player.get_unique_id());
            let mut sent = SENT.lock().unwrap_or_else(|e| e.into_inner());
            let last = sent.get(&key);
            // The game's own patches reach the client too, so a change of its picture means sending ours again.
            let picture = last.is_none_or(|(old, old_own, _)| *old != colors || *old_own != own);
            let marked = last.is_none_or(|(_, _, old)| *old != cursors);
            if picture || marked {
                sent.insert(key, (colors.clone(), own.clone(), cursors.clone()));
                drop(sent);
                query(json!({ "op": "core_map_send", "uuid": player.uuid(), "id": id, "colors": if picture { json!(colors) } else { json!([]) }, "cursors": cursors }));
            }
        }
        CURRENT_VIEW.with(|current| current.set(None));
    }
}

/// `org.bukkit.map.MapView`: a map the game saves, by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapView {
    id: i32,
}

/// bukkit: org/bukkit/Bukkit.getMap, org/bukkit/Server.getMap
pub fn get_map(id: i32) -> Option<MapView> {
    (!query(json!({ "op": "core_map", "id": id })).is_null()).then_some(MapView { id })
}

/// bukkit: org/bukkit/Bukkit.createMap, org/bukkit/Server.createMap
pub fn create_map(world: &World) -> Option<MapView> {
    let spawn = world.get_spawn_location();
    let id = query(json!({ "op": "core_map_create", "dimension": world.dimension(), "at": [spawn.x, spawn.z], "scale": 3 })).as_i64()?;
    Some(MapView { id: id as i32 })
}

impl MapView {
    fn state(&self) -> Value {
        query(json!({ "op": "core_map", "id": self.id }))
    }
    fn change(&self, field: &str, value: Value) {
        let mut request = json!({ "op": "core_map_change", "id": self.id });
        request[field] = value;
        query(request);
    }
    /// bukkit: org/bukkit/map/MapView.getId
    pub fn get_id(&self) -> i32 {
        self.id
    }
    /// bukkit: org/bukkit/map/MapView.isVirtual
    ///
    /// Whether what draws first isn't the game's own, as `CraftMapView`.
    pub fn is_virtual(&self) -> bool {
        let game = game_renderer();
        self.renderers(|list| list.first().is_some_and(|first| !same_renderer(first, &game)))
    }
    /// bukkit: org/bukkit/map/MapView.getScale
    pub fn get_scale(&self) -> &'static str {
        ["CLOSEST", "CLOSE", "NORMAL", "FAR", "FARTHEST"].get(self.state()["scale"].as_u64().unwrap_or(3) as usize).copied().unwrap_or("NORMAL")
    }
    /// bukkit: org/bukkit/map/MapView.setScale
    pub fn set_scale(&self, scale: &str) {
        self.change("scale", json!(crate::pure2::map_scale_value(scale)));
    }
    /// bukkit: org/bukkit/map/MapView.getCenterX
    pub fn get_center_x(&self) -> i32 {
        int(&self.state()["center"][0])
    }
    /// bukkit: org/bukkit/map/MapView.getCenterZ
    pub fn get_center_z(&self) -> i32 {
        int(&self.state()["center"][1])
    }
    /// bukkit: org/bukkit/map/MapView.setCenterX
    pub fn set_center_x(&self, x: i32) {
        self.change("center_x", json!(x));
    }
    /// bukkit: org/bukkit/map/MapView.setCenterZ
    pub fn set_center_z(&self, z: i32) {
        self.change("center_z", json!(z));
    }
    /// bukkit: org/bukkit/map/MapView.getWorld
    pub fn get_world(&self) -> Option<World> {
        self.state()["dimension"].as_str().map(World::of)
    }
    /// bukkit: org/bukkit/map/MapView.setWorld
    pub fn set_world(&self, world: &World) {
        self.change("dimension", json!(world.dimension()));
    }
    /// bukkit: org/bukkit/map/MapView.isTrackingPosition
    pub fn is_tracking_position(&self) -> bool {
        self.state()["tracking"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/map/MapView.setTrackingPosition
    pub fn set_tracking_position(&self, on: bool) {
        self.change("tracking", json!(on));
    }
    /// bukkit: org/bukkit/map/MapView.isUnlimitedTracking
    pub fn is_unlimited_tracking(&self) -> bool {
        self.state()["unlimited"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/map/MapView.setUnlimitedTracking
    pub fn set_unlimited_tracking(&self, on: bool) {
        self.change("unlimited", json!(on));
    }
    /// bukkit: org/bukkit/map/MapView.isLocked
    pub fn is_locked(&self) -> bool {
        self.state()["locked"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/map/MapView.setLocked
    pub fn set_locked(&self, on: bool) {
        self.change("locked", json!(on));
    }
    fn renderers<T>(&self, body: impl FnOnce(&mut Renderers) -> T) -> T {
        let mut all = RENDERERS.lock().unwrap_or_else(|e| e.into_inner());
        body(all.entry(self.id).or_insert_with(|| vec![game_renderer()]))
    }
    /// bukkit: org/bukkit/map/MapView.addRenderer
    ///
    /// Draws for whoever holds the map, each tick, over the renderers before it. The `Arc` it
    /// gives back is what `remove_renderer` takes.
    pub fn add_renderer(&self, renderer: impl MapRenderer + 'static) -> std::sync::Arc<dyn MapRenderer> {
        let renderer: std::sync::Arc<dyn MapRenderer> = std::sync::Arc::new(renderer);
        self.renderers(|list| list.push(renderer.clone()));
        renderer.initialize(self);
        start_rendering();
        renderer
    }
    /// bukkit: org/bukkit/map/MapView.getRenderers
    ///
    /// Every plugin's. Another's stays in that plugin: what's here calls it.
    pub fn get_renderers(&self) -> Vec<std::sync::Arc<dyn MapRenderer>> {
        let mut all = self.renderers(|list| list.clone());
        for plugin in other_plugins() {
            for row in ask_about_map(&plugin, "list", self.id, 0, None).as_array().into_iter().flatten() {
                let renderer = ForeignRenderer { plugin: plugin.clone(), key: row["key"].as_u64().unwrap_or(0) as usize, contextual: row["contextual"].as_bool().unwrap_or(false) };
                all.push(std::sync::Arc::new(renderer));
            }
        }
        all
    }
    /// bukkit: org/bukkit/map/MapView.removeRenderer
    ///
    /// What it drew goes with it. With the game's own the only one left, the game sends the map again.
    pub fn remove_renderer(&self, renderer: &std::sync::Arc<dyn MapRenderer>) -> bool {
        if let Some((plugin, key)) = renderer.foreign() {
            return ask_about_map(&plugin, "remove", self.id, key, None).as_bool().unwrap_or(false);
        }
        if same_renderer(renderer, &game_renderer()) {
            for plugin in other_plugins() {
                ask_about_map(&plugin, "game_off", self.id, 0, None);
            }
        }
        let (removed, only_game) = self.renderers(|list| {
            let before = list.len();
            list.retain(|known| !same_renderer(known, renderer));
            (list.len() != before, list.len() == 1 && same_renderer(&list[0], &game_renderer()))
        });
        if !removed {
            return false;
        }
        let key = renderer_key(renderer);
        CANVASES.lock().unwrap_or_else(|e| e.into_inner()).retain(|(known, _), _| *known != key);
        if only_game {
            SENT.lock().unwrap_or_else(|e| e.into_inner()).retain(|(id, _), _| *id != self.id);
            query(json!({ "op": "core_map_touch", "id": self.id }));
        }
        // Even an empty list draws: a map with no renderers is blank, which the loop has to send.
        start_rendering();
        true
    }
}

impl crate::pure2::MapCanvas {
    /// bukkit: org/bukkit/map/MapCanvas.getMapView
    ///
    /// The map being drawn on, inside `render`.
    pub fn get_map_view(&self) -> Option<MapView> {
        CURRENT_VIEW.with(std::cell::Cell::get).map(|id| MapView { id })
    }
    /// bukkit: org/bukkit/map/MapCanvas.setPixelColor
    ///
    /// `None` leaves the pixel to the base, as Java's `null`.
    pub fn set_pixel_color(&mut self, x: i32, y: i32, color: Option<Color>) {
        if let Some(color) = color {
            self.set_pixel(x, y, match_color(color.red, color.green, color.blue));
        }
    }
    /// bukkit: org/bukkit/map/MapCanvas.getPixelColor
    pub fn get_pixel_color(&self, x: i32, y: i32) -> Option<Color> {
        self.get_pixel(x, y).and_then(palette_color)
    }
    /// bukkit: org/bukkit/map/MapCanvas.getBasePixelColor
    pub fn get_base_pixel_color(&self, x: i32, y: i32) -> Option<Color> {
        palette_color(self.get_base_pixel(x, y))
    }
    /// bukkit: org/bukkit/map/MapCanvas.drawText
    ///
    /// `\u{a7}<n>;` sets the colour by palette index; a newline starts a line.
    pub fn draw_text(&mut self, x: i32, y: i32, font: &MapFont, text: &str) {
        let (mut pen_x, mut pen_y, mut color) = (x, y, 44_u8);
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\n' {
                pen_x = x;
                pen_y += font.get_height() + 1;
                continue;
            }
            if ch == '\u{a7}' {
                let mut digits = String::new();
                while let Some(next) = chars.next_if(char::is_ascii_digit) {
                    digits.push(next);
                }
                if chars.next_if_eq(&';').is_some()
                    && let Ok(value) = digits.parse()
                {
                    color = value;
                }
                continue;
            }
            let Some(sprite) = font.get_char(ch) else {
                continue;
            };
            for row in 0..sprite.get_height() {
                for column in 0..sprite.get_width() {
                    if sprite.get(row, column) {
                        self.set_pixel(pen_x + column, pen_y + row, color);
                    }
                }
            }
            pen_x += sprite.get_width() + 1;
        }
    }
    /// bukkit: org/bukkit/map/MapCanvas.drawImage
    ///
    /// `rgba` is what `java.awt.Image` would hold: `width` pixels a row, four bytes each.
    pub fn draw_image(&mut self, x: i32, y: i32, width: usize, rgba: &[u8]) {
        for (index, pixel) in rgba.chunks_exact(4).enumerate() {
            if pixel[3] >= 128 {
                let (column, row) = ((index % width.max(1)) as i32, (index / width.max(1)) as i32);
                self.set_pixel(x + column, y + row, match_color(pixel[0], pixel[1], pixel[2]));
            }
        }
    }
}

/// `org.bukkit.map.MapFont$CharacterSprite`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterSprite {
    width: i32,
    height: i32,
    data: Vec<bool>,
}

impl CharacterSprite {
    /// bukkit: org/bukkit/map/MapFont$CharacterSprite.<init>
    pub fn new(width: i32, height: i32, data: Vec<bool>) -> Self {
        Self { width, height, data }
    }
    /// bukkit: org/bukkit/map/MapFont$CharacterSprite.get
    pub fn get(&self, row: i32, column: i32) -> bool {
        if row < 0 || column < 0 || row >= self.height || column >= self.width {
            return false;
        }
        self.data.get((row * self.width + column) as usize).copied().unwrap_or(false)
    }
    /// bukkit: org/bukkit/map/MapFont$CharacterSprite.getWidth
    pub fn get_width(&self) -> i32 {
        self.width
    }
    /// bukkit: org/bukkit/map/MapFont$CharacterSprite.getHeight
    pub fn get_height(&self) -> i32 {
        self.height
    }
}

/// `org.bukkit.map.MapFont`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapFont {
    chars: HashMap<char, CharacterSprite>,
    height: i32,
}

impl MapFont {
    /// bukkit: org/bukkit/map/MapFont.setChar
    pub fn set_char(&mut self, ch: char, sprite: CharacterSprite) {
        self.height = self.height.max(sprite.height);
        self.chars.insert(ch, sprite);
    }
    /// bukkit: org/bukkit/map/MapFont.getChar
    pub fn get_char(&self, ch: char) -> Option<&CharacterSprite> {
        self.chars.get(&ch)
    }
    /// bukkit: org/bukkit/map/MapFont.getHeight
    pub fn get_height(&self) -> i32 {
        self.height
    }
    /// bukkit: org/bukkit/map/MapFont.isValid
    pub fn is_valid(&self, text: &str) -> bool {
        text.chars().all(|ch| ch == '\u{a7}' || ch == '\n' || self.chars.contains_key(&ch))
    }
    /// bukkit: org/bukkit/map/MapFont.getWidth
    ///
    /// The width of a line, a pixel between letters; `\u{a7}<n>;` takes no room.
    pub fn get_width(&self, text: &str) -> i32 {
        let mut width = 0;
        let mut drawn = 0;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{a7}' {
                while chars.next_if(|next| next.is_ascii_digit()).is_some() {}
                let _ = chars.next_if_eq(&';');
                continue;
            }
            if let Some(sprite) = self.chars.get(&ch) {
                width += sprite.width;
                drawn += 1;
            }
        }
        width + (drawn - 1).max(0)
    }
}

/// The game's map colours: for each base colour its four shades, as the bytes a map holds.
fn palette() -> &'static [(u8, u8, u8)] {
    static PALETTE: std::sync::OnceLock<Vec<(u8, u8, u8)>> = std::sync::OnceLock::new();
    PALETTE.get_or_init(|| {
        let bases = query(json!({ "op": "core_map_palette" }));
        let mut out = Vec::new();
        for base in bases.as_array().into_iter().flatten() {
            let rgb = base.as_u64().unwrap_or(0) as u32;
            for shade in [180_u32, 220, 255, 135] {
                let channel = |shift: u32| (((rgb >> shift) & 255) * shade / 255) as u8;
                out.push((channel(16), channel(8), channel(0)));
            }
        }
        out
    })
}

fn palette_color(index: u8) -> Option<Color> {
    (index >= 4).then(|| palette().get(usize::from(index)).map(|(red, green, blue)| Color::from_rgb(*red, *green, *blue))).flatten()
}

/// bukkit: org/bukkit/map/MapPalette.matchColor
pub fn match_color(red: u8, green: u8, blue: u8) -> u8 {
    if let Some(cache) = COLOR_CACHE.get().filter(|cache| cache.is_cached()) {
        return cache.match_color(Color::from_rgb(red, green, blue));
    }
    let distance = |other: &(u8, u8, u8)| {
        let sum = i32::from(red) + i32::from(other.0);
        let (r, g, b) = (i32::from(red) - i32::from(other.0), i32::from(green) - i32::from(other.1), i32::from(blue) - i32::from(other.2));
        (1024 + sum) * r * r + 2048 * g * g + (1024 + (510 - sum)) * b * b
    };
    palette().iter().enumerate().skip(4).min_by_key(|(_, color)| distance(color)).map_or(0, |(index, _)| index as u8)
}

/// bukkit: org/bukkit/map/MapPalette.getColor
pub fn get_color(index: u8) -> Option<Color> {
    palette_color(index)
}

/// bukkit: org/bukkit/map/MapPalette.getNearestColor
pub fn get_nearest_color(color: Color) -> Option<Color> {
    palette_color(match_color(color.red, color.green, color.blue))
}

/// bukkit: org/bukkit/map/MapPalette.imageToBytes
///
/// `rgba` as in `MapCanvas::draw_image`: a byte of the palette for each pixel.
pub fn image_to_bytes(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4).map(|pixel| if pixel[3] < 128 { 0 } else { match_color(pixel[0], pixel[1], pixel[2]) }).collect()
}

/// bukkit: org/bukkit/map/MapPalette.resizeImage
///
/// Nearest pixels, to the map's 128 by 128.
pub fn resize_image(width: usize, height: usize, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(128 * 128 * 4);
    for y in 0..128 {
        for x in 0..128 {
            let from = (y * height.max(1) / 128 * width + x * width.max(1) / 128) * 4;
            out.extend_from_slice(rgba.get(from..from + 4).unwrap_or(&[0, 0, 0, 0]));
        }
    }
    out
}

/// `org.bukkit.map.MapPalette$MapColorCache`: a plugin's own faster `match_color`.
pub trait MapColorCache: Send + Sync {
    /// bukkit: org/bukkit/map/MapPalette$MapColorCache.isCached
    fn is_cached(&self) -> bool;
    /// bukkit: org/bukkit/map/MapPalette$MapColorCache.matchColor
    fn match_color(&self, color: Color) -> u8;
}

static COLOR_CACHE: std::sync::OnceLock<Box<dyn MapColorCache>> = std::sync::OnceLock::new();

/// bukkit: org/bukkit/map/MapPalette.setMapColorCache
pub fn set_map_color_cache(cache: impl MapColorCache + 'static) {
    assert!(COLOR_CACHE.set(Box::new(cache)).is_ok(), "Map color cache already set");
}

const MAP_CURSOR_TYPES: [&str; 40] = ["player", "frame", "red_marker", "blue_marker", "target_x", "target_point", "player_off_map", "player_off_limits", "mansion", "monument", "banner_white", "banner_orange", "banner_magenta", "banner_light_blue", "banner_yellow", "banner_lime", "banner_pink", "banner_gray", "banner_light_gray", "banner_cyan", "banner_purple", "banner_blue", "banner_brown", "banner_green", "banner_red", "banner_black", "red_x", "village_desert", "village_plains", "village_savanna", "village_snowy", "village_taiga", "jungle_temple", "swamp_hut", "trial_chambers", "abandoned_camp", "ancient_city", "desert_pyramid", "mineshaft", "ocean_ruin_warm"];

/// The registry's numbers for `MapCursor$Type`, which are what a packet carries.
fn map_cursor_ids() -> &'static BTreeMap<&'static str, u8> {
    static IDS: std::sync::OnceLock<BTreeMap<&'static str, u8>> = std::sync::OnceLock::new();
    IDS.get_or_init(|| {
        let found = query(json!({ "op": "core_map_decorations", "names": MAP_CURSOR_TYPES.to_vec() }));
        MAP_CURSOR_TYPES.iter().filter_map(|name| Some((*name, found[*name].as_u64()? as u8))).collect()
    })
}

fn map_cursor_type_key(kind: u8) -> String {
    let name = map_cursor_ids().iter().find(|(_, id)| **id == kind).map_or("player", |(name, _)| *name);
    format!("minecraft:{name}")
}

/// bukkit: org/bukkit/map/MapCursor$Type.getValue
pub fn map_cursor_type_value(name: &str) -> Option<u8> {
    map_cursor_ids().get(name.to_ascii_lowercase().as_str()).copied()
}

/// bukkit: org/bukkit/map/MapCursor$Type.byValue
pub fn map_cursor_type_by_value(value: u8) -> Option<String> {
    map_cursor_ids().iter().find(|(_, id)| **id == value).map(|(name, _)| name.to_ascii_uppercase())
}

// ---- Loot ----

/// `org.bukkit.loot.LootContext` and its builder.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LootContext {
    location: Option<Location>,
    luck: f32,
    looting: i32,
    looted: Option<Entity>,
    killer: Option<Player>,
}

impl LootContext {
    pub(crate) fn builder(location: Option<Location>, looted: Option<Entity>) -> Self { Self { location, luck: 0.0, looting: 0, looted, killer: None } }
    /// bukkit: org/bukkit/loot/LootContext$Builder.<init>, org/bukkit/loot/LootContext$Builder.build
    pub fn new(location: &Location) -> Self {
        Self { location: Some(location.clone()), ..Self::default() }
    }
    /// bukkit: org/bukkit/loot/LootContext$Builder.luck
    pub fn luck(mut self, luck: f32) -> Self {
        self.luck = luck;
        self
    }
    /// bukkit: org/bukkit/loot/LootContext$Builder.lootingModifier
    pub fn looting_modifier(mut self, looting: i32) -> Self {
        self.looting = looting;
        self
    }
    /// bukkit: org/bukkit/loot/LootContext$Builder.lootedEntity
    pub fn looted_entity(mut self, entity: &Entity) -> Self {
        self.looted = Some(entity.clone());
        self
    }
    /// bukkit: org/bukkit/loot/LootContext$Builder.killer
    pub fn killer(mut self, killer: &Player) -> Self {
        self.killer = Some(killer.clone());
        self
    }
    /// bukkit: org/bukkit/loot/LootContext.getLocation
    pub fn get_location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
    /// bukkit: org/bukkit/loot/LootContext.getLuck
    pub fn get_luck(&self) -> f32 {
        self.luck
    }
    /// bukkit: org/bukkit/loot/LootContext.getLootingModifier
    pub fn get_looting_modifier(&self) -> i32 {
        self.looting
    }
    /// bukkit: org/bukkit/loot/LootContext.getLootedEntity
    pub fn get_looted_entity(&self) -> Option<&Entity> {
        self.looted.as_ref()
    }
    /// bukkit: org/bukkit/loot/LootContext.getKiller
    pub fn get_killer(&self) -> Option<&Player> {
        self.killer.as_ref()
    }
}

/// `org.bukkit.loot.LootTable`: one of the game's, by its key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LootTable {
    key: String,
}

/// bukkit: org/bukkit/Bukkit.getLootTable, org/bukkit/Server.getLootTable
pub fn get_loot_table(key: &str) -> Option<LootTable> {
    let key = key_of(key);
    query(json!({ "op": "core_loot_known", "table": key })).as_bool().unwrap_or(false).then_some(LootTable { key })
}

/// bukkit: org/bukkit/loot/LootTables.getLootTable, org/bukkit/loot/LootTables.getKey
///
/// By a `LootTables` name: `SIMPLE_DUNGEON` is `minecraft:chests/simple_dungeon`.
pub fn loot_tables_get(name: &str) -> Option<LootTable> {
    let lower = name.to_ascii_lowercase();
    let known = query(json!({ "op": "core_data_tree", "path": "loot_table" }));
    let key = known.as_array()?.iter().filter_map(Value::as_str).find(|key| key.rsplit('/').next() == Some(&lower) || *key == lower)?.to_owned();
    Some(LootTable { key })
}

impl LootTable {
    /// bukkit: org/bukkit/loot/LootTable.getKey
    pub fn get_key(&self) -> &str {
        &self.key
    }
    /// bukkit: org/bukkit/loot/LootTable.populateLoot
    ///
    /// `seed` stands for Java's `Random`.
    pub fn populate_loot(&self, seed: Option<i64>, context: &LootContext) -> Vec<ItemStack> {
        let request = self.loot_request(seed, context);
        query(request).as_array().into_iter().flatten().filter_map(ItemStack::read).collect()
    }
    /// bukkit: org/bukkit/loot/LootTable.fillInventory
    ///
    /// Spread over the free slots as a chest's loot is, on what's rolled here. A plugin's `Inventory`
    /// is its own copy: `update_inventory` or a menu shows it.
    pub fn fill_inventory(&self, inventory: &mut crate::Inventory, seed: Option<i64>, context: &LootContext) {
        let free: Vec<usize> = (0..inventory.get_size()).filter(|slot| inventory.get_item(*slot).is_none()).collect();
        let mut request = self.loot_request(seed, context);
        request["slots"] = json!(free.len());
        let answer = query(request);
        for (slot, row) in free.iter().zip(answer.as_array().into_iter().flatten()) {
            if !row.is_null() {
                inventory.set_item(*slot, ItemStack::read(row));
            }
        }
    }
    fn loot_request(&self, seed: Option<i64>, context: &LootContext) -> Value {
        let mut request = json!({ "op": "core_loot", "table": self.key, "luck": context.luck, "killer": context.killer.as_ref().map(Player::uuid), "looted": context.looted.as_ref().map(Entity::get_entity_id) });
        if let Some(seed) = seed {
            request["seed"] = json!(seed);
        }
        request
    }
}

// ---- Structures ----

static NEXT_STRUCTURE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

/// `org.bukkit.structure.Structure`: a template the game holds, under a name of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Structure {
    template: String,
}

impl Structure {
    fn unnamed() -> Self {
        Self { template: format!("horizon_bukkit:plugin/s{}", NEXT_STRUCTURE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)) }
    }
    fn tree(&self) -> Value {
        query(json!({ "op": "core_structure_tree", "template": self.template }))
    }
    /// bukkit: org/bukkit/structure/Structure.fill
    ///
    /// Everything between the corners, ends included. `false` where the area isn't loaded here.
    pub fn fill(&self, first: &Location, second: &Location, entities: bool) -> bool {
        let low = [first.get_block_x().min(second.get_block_x()), first.get_block_y().min(second.get_block_y()), first.get_block_z().min(second.get_block_z())];
        let high = [first.get_block_x().max(second.get_block_x()), first.get_block_y().max(second.get_block_y()), first.get_block_z().max(second.get_block_z())];
        let size = [high[0] - low[0] + 1, high[1] - low[1] + 1, high[2] - low[2] + 1];
        query(json!({ "op": "core_structure_fill", "template": self.template, "dimension": first.world.dimension(), "origin": low, "size": size, "entities": entities })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/structure/Structure.place
    ///
    /// `rotation` is a `StructureRotation` name in the game's own words (`clockwise_90`), `mirror` a
    /// `Mirror` name, `seed` Java's `Random`. The game places the template's first palette.
    pub fn place(&self, at: &Location, entities: bool, rotation: &str, mirror: &str, integrity: f32, seed: i64) -> bool {
        query(json!({ "op": "core_structure_place", "template": self.template, "dimension": at.world.dimension(), "pos": [at.get_block_x(), at.get_block_y(), at.get_block_z()], "entities": entities, "rotation": rotation.to_ascii_lowercase(), "mirror": mirror, "integrity": integrity, "seed": seed })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/structure/Structure.getSize
    pub fn get_size(&self) -> Vector {
        let size = self.tree()["size"].clone();
        let side = |i: usize| size[i].as_f64().unwrap_or(0.0);
        Vector::new(side(0), side(1), side(2))
    }
    /// bukkit: org/bukkit/structure/Structure.getPaletteCount
    pub fn get_palette_count(&self) -> usize {
        self.get_palettes().len()
    }
    /// bukkit: org/bukkit/structure/Structure.getPalettes
    pub fn get_palettes(&self) -> Vec<Palette> {
        let tree = self.tree();
        let blocks = tree["blocks"].as_array().cloned().unwrap_or_default();
        let palettes: Vec<Value> = match tree["palettes"].as_array() {
            Some(all) => all.clone(),
            None if tree["palette"].is_array() => vec![tree["palette"].clone()],
            None => Vec::new(),
        };
        palettes.into_iter().map(|palette| Palette { blocks: blocks.clone(), states: palette.as_array().cloned().unwrap_or_default() }).collect()
    }
    /// bukkit: org/bukkit/structure/Structure.getEntityCount
    pub fn get_entity_count(&self) -> usize {
        self.tree()["entities"].as_array().map_or(0, Vec::len)
    }
    /// bukkit: org/bukkit/structure/Structure.getEntities
    ///
    /// Each as the game saves it: `pos`, `blockPos` and `nbt`. They aren't in a level to be `Entity`s.
    pub fn get_entities(&self) -> Vec<Value> {
        self.tree()["entities"].as_array().cloned().unwrap_or_default()
    }
}

/// `org.bukkit.structure.Palette`.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    blocks: Vec<Value>,
    states: Vec<Value>,
}

/// One block of a palette: `org.bukkit.block.BlockState` where there's no world to read it from.
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteBlock {
    pub pos: [i32; 3],
    pub name: String,
    pub properties: Value,
    pub nbt: Value,
}

impl Palette {
    /// bukkit: org/bukkit/structure/Palette.getBlockCount
    pub fn get_block_count(&self) -> usize {
        self.blocks.len()
    }
    /// bukkit: org/bukkit/structure/Palette.getBlocks
    pub fn get_blocks(&self) -> Vec<PaletteBlock> {
        self.blocks
            .iter()
            .filter_map(|row| {
                let state = self.states.get(row["state"].as_u64()? as usize)?;
                Some(PaletteBlock { pos: [int(&row["pos"][0]), int(&row["pos"][1]), int(&row["pos"][2])], name: state["Name"].as_str()?.to_owned(), properties: state["Properties"].clone(), nbt: row["nbt"].clone() })
            })
            .collect()
    }
}

/// `org.bukkit.structure.StructureManager`: the game's templates by key.
#[derive(Clone, Copy, Debug, Default)]
pub struct StructureManager;

static REGISTERED: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// bukkit: org/bukkit/Bukkit.getStructureManager, org/bukkit/Server.getStructureManager
pub fn get_structure_manager() -> StructureManager {
    StructureManager
}

impl StructureManager {
    /// bukkit: org/bukkit/structure/StructureManager.createStructure
    pub fn create_structure(&self) -> Structure {
        Structure::unnamed()
    }
    /// bukkit: org/bukkit/structure/StructureManager.getStructure
    ///
    /// One already held in memory: `load_structure` reads it in.
    pub fn get_structure(&self, key: &str) -> Option<Structure> {
        let template = key_of(key);
        query(json!({ "op": "core_structure_known", "template": template })).as_bool().unwrap_or(false).then_some(Structure { template })
    }
    /// bukkit: org/bukkit/structure/StructureManager.loadStructure
    ///
    /// From the world's `generated` folder, else the game's own.
    pub fn load_structure(&self, key: &str, register: bool) -> Option<Structure> {
        let template = key_of(key);
        if query(json!({ "op": "core_structure_tree", "template": template })).is_null() {
            return None;
        }
        if register {
            REGISTERED.lock().unwrap_or_else(|e| e.into_inner()).insert(template.clone());
        } else {
            query(json!({ "op": "core_structure_forget", "template": template }));
        }
        Some(Structure { template })
    }
    /// bukkit: org/bukkit/structure/StructureManager.registerStructure
    ///
    /// What was registered under the key before, if anything; the template is copied to the key.
    pub fn register_structure(&self, key: &str, structure: &Structure) -> Option<Structure> {
        let template = key_of(key);
        let before = self.get_structure(&template).map(|old| {
            let kept = Structure::unnamed();
            query(json!({ "op": "core_structure_copy", "from": old.template, "to": kept.template }));
            kept
        });
        query(json!({ "op": "core_structure_copy", "from": structure.template, "to": template }));
        REGISTERED.lock().unwrap_or_else(|e| e.into_inner()).insert(template);
        before
    }
    /// bukkit: org/bukkit/structure/StructureManager.unregisterStructure
    pub fn unregister_structure(&self, key: &str) -> Option<Structure> {
        let template = key_of(key);
        REGISTERED.lock().unwrap_or_else(|e| e.into_inner()).remove(&template);
        let kept = self.get_structure(&template).map(|old| {
            let copy = Structure::unnamed();
            query(json!({ "op": "core_structure_copy", "from": old.template, "to": copy.template }));
            copy
        });
        query(json!({ "op": "core_structure_forget", "template": template }));
        kept
    }
    /// bukkit: org/bukkit/structure/StructureManager.getStructures
    ///
    /// What the game holds loaded, as Java's: whatever any plugin or the game has read so far.
    pub fn get_structures(&self) -> BTreeMap<String, Structure> {
        let held = query(json!({ "op": "core_structures" }));
        held.as_array().into_iter().flatten().filter_map(Value::as_str).map(|key| (key.to_owned(), Structure { template: key.to_owned() })).collect()
    }
    /// bukkit: org/bukkit/structure/StructureManager.saveStructure
    ///
    /// To the world's `generated` folder, as a structure block saves.
    pub fn save_structure(&self, key: &str) -> bool {
        query(json!({ "op": "core_structure_save", "template": key_of(key) })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/structure/StructureManager.deleteStructure
    pub fn delete_structure(&self, key: &str) -> bool {
        let template = key_of(key);
        REGISTERED.lock().unwrap_or_else(|e| e.into_inner()).remove(&template);
        query(json!({ "op": "core_structure_delete", "template": template })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/structure/StructureManager.getStructureFile
    pub fn get_structure_file(&self, key: &str) -> PathBuf {
        let template = key_of(key);
        let (space, path) = template.split_once(':').unwrap_or(("minecraft", &template));
        world_folder().join("generated").join(space).join("structures").join(format!("{path}.nbt"))
    }
    /// bukkit: org/bukkit/structure/StructureManager.copy
    pub fn copy(&self, structure: &Structure) -> Structure {
        let copy = Structure::unnamed();
        query(json!({ "op": "core_structure_copy", "from": structure.template, "to": copy.template }));
        copy
    }
}

// ---- The server ----

/// bukkit: org/bukkit/Bukkit.isStopping, org/bukkit/Server.isStopping
pub fn is_stopping() -> bool {
    STOPPING_HERE.load(std::sync::atomic::Ordering::Relaxed) || query(json!({ "op": "core_stopping" })).as_bool().unwrap_or(false)
}

/// bukkit: org/bukkit/Bukkit.getRegistry, org/bukkit/Server.getRegistry
///
/// By the class's simple name, as Rust has no `Class<T>`: `Enchantment`, `Biome`, `Villager.Type`.
/// What's here is what `RegistryKey` names, as in Paper: `Material` has no registry to be asked for.
pub fn get_registry(class: &str) -> Option<crate::registry::Registry> {
    use crate::registry::Registry;
    Some(match class {
        "Enchantment" => Registry::ENCHANTMENT,
        "Biome" => Registry::BIOME,
        "DamageType" => Registry::DAMAGE_TYPE,
        "MusicInstrument" => Registry::INSTRUMENT,
        "JukeboxSong" => Registry::JUKEBOX_SONG,
        "Art" => Registry::PAINTING_VARIANT,
        "TrimMaterial" => Registry::TRIM_MATERIAL,
        "TrimPattern" => Registry::TRIM_PATTERN,
        "PatternType" => Registry::BANNER_PATTERN,
        "Structure" => Registry::STRUCTURE,
        "Wolf.Variant" => Registry::WOLF_VARIANT,
        "Wolf.SoundVariant" => Registry::WOLF_SOUND_VARIANT,
        "Cat.Type" => Registry::CAT_VARIANT,
        "Cat.SoundVariant" => Registry::CAT_SOUND_VARIANT,
        "Frog.Variant" => Registry::FROG_VARIANT,
        "Chicken.Variant" => Registry::CHICKEN_VARIANT,
        "Chicken.SoundVariant" => Registry::CHICKEN_SOUND_VARIANT,
        "Cow.Variant" => Registry::COW_VARIANT,
        "Cow.SoundVariant" => Registry::COW_SOUND_VARIANT,
        "Pig.Variant" => Registry::PIG_VARIANT,
        "Pig.SoundVariant" => Registry::PIG_SOUND_VARIANT,
        "ZombieNautilus.Variant" => Registry::ZOMBIE_NAUTILUS_VARIANT,
        "SulfurCube.Archetype" => Registry::SULFUR_CUBE_ARCHETYPE,
        "Dialog" => Registry::DIALOG,
        "GameEvent" => Registry::GAME_EVENT,
        "StructureType" => Registry::STRUCTURE_TYPE,
        "PotionEffectType" => Registry::MOB_EFFECT,
        "BlockType" => Registry::BLOCK,
        "ItemType" => Registry::ITEM,
        "Villager.Profession" => Registry::VILLAGER_PROFESSION,
        "Villager.Type" => Registry::VILLAGER_TYPE,
        "MapCursor.Type" => Registry::MAP_DECORATION_TYPE,
        "MenuType" => Registry::MENU,
        "Attribute" => Registry::ATTRIBUTE,
        "Fluid" => Registry::FLUID,
        "Sound" => Registry::SOUND_EVENT,
        "DataComponentType" => Registry::DATA_COMPONENT_TYPE,
        "GameRule" => Registry::GAME_RULE,
        "PoiType" => Registry::POINT_OF_INTEREST_TYPE,
        "EntityType" => Registry::ENTITY_TYPE,
        "Particle" => Registry::PARTICLE_TYPE,
        "PotionType" => Registry::POTION,
        "MemoryKey" => Registry::MEMORY_MODULE_TYPE,
        _ => return None,
    })
}

/// bukkit: org/bukkit/Bukkit.getSpawnLimit, org/bukkit/Server.getSpawnLimit
pub fn get_spawn_limit(category: &str) -> i32 {
    spawn_limit_of(category)
}

/// bukkit: org/bukkit/Bukkit.getTicksPerSpawns, org/bukkit/Server.getTicksPerSpawns
pub fn get_ticks_per_spawns(category: &str) -> i64 {
    ticks_per_spawns_of(category)
}

/// bukkit: org/bukkit/Bukkit.getMonsterSpawnLimit, org/bukkit/Server.getMonsterSpawnLimit
pub fn get_monster_spawn_limit() -> i32 {
    spawn_limit_of("MONSTER")
}

/// bukkit: org/bukkit/Bukkit.getAnimalSpawnLimit, org/bukkit/Server.getAnimalSpawnLimit
pub fn get_animal_spawn_limit() -> i32 {
    spawn_limit_of("ANIMAL")
}

/// bukkit: org/bukkit/Bukkit.getAmbientSpawnLimit, org/bukkit/Server.getAmbientSpawnLimit
pub fn get_ambient_spawn_limit() -> i32 {
    spawn_limit_of("AMBIENT")
}

/// bukkit: org/bukkit/Bukkit.getWaterAnimalSpawnLimit, org/bukkit/Server.getWaterAnimalSpawnLimit
pub fn get_water_animal_spawn_limit() -> i32 {
    spawn_limit_of("WATER_ANIMAL")
}

/// bukkit: org/bukkit/Bukkit.getWaterAmbientSpawnLimit, org/bukkit/Server.getWaterAmbientSpawnLimit
pub fn get_water_ambient_spawn_limit() -> i32 {
    spawn_limit_of("WATER_AMBIENT")
}

/// bukkit: org/bukkit/Bukkit.getWaterUndergroundCreatureSpawnLimit, org/bukkit/Server.getWaterUndergroundCreatureSpawnLimit
pub fn get_water_underground_creature_spawn_limit() -> i32 {
    spawn_limit_of("WATER_UNDERGROUND_CREATURE")
}

/// bukkit: org/bukkit/Bukkit.getTicksPerAnimalSpawns, org/bukkit/Server.getTicksPerAnimalSpawns
pub fn get_ticks_per_animal_spawns() -> i64 {
    ticks_per_spawns_of("ANIMAL")
}

/// bukkit: org/bukkit/Bukkit.getTicksPerMonsterSpawns, org/bukkit/Server.getTicksPerMonsterSpawns
pub fn get_ticks_per_monster_spawns() -> i64 {
    ticks_per_spawns_of("MONSTER")
}

/// bukkit: org/bukkit/Bukkit.getTicksPerWaterSpawns, org/bukkit/Server.getTicksPerWaterSpawns
pub fn get_ticks_per_water_spawns() -> i64 {
    ticks_per_spawns_of("WATER_ANIMAL")
}

/// bukkit: org/bukkit/Bukkit.getTicksPerWaterAmbientSpawns, org/bukkit/Server.getTicksPerWaterAmbientSpawns
pub fn get_ticks_per_water_ambient_spawns() -> i64 {
    ticks_per_spawns_of("WATER_AMBIENT")
}

/// bukkit: org/bukkit/Bukkit.getTicksPerWaterUndergroundCreatureSpawns, org/bukkit/Server.getTicksPerWaterUndergroundCreatureSpawns
pub fn get_ticks_per_water_underground_creature_spawns() -> i64 {
    ticks_per_spawns_of("WATER_UNDERGROUND_CREATURE")
}

/// bukkit: org/bukkit/Bukkit.getTicksPerAmbientSpawns, org/bukkit/Server.getTicksPerAmbientSpawns
pub fn get_ticks_per_ambient_spawns() -> i64 {
    ticks_per_spawns_of("AMBIENT")
}

/// bukkit: org/bukkit/Bukkit.getOfflinePlayerIfCached, org/bukkit/Server.getOfflinePlayerIfCached
pub fn get_offline_player_if_cached(name: &str) -> Option<OfflinePlayer> {
    if get_player(name).is_some() {
        return Some(crate::get_offline_player(name));
    }
    let cache = file_json(&server_folder().join("usercache.json"));
    cache.as_array()?.iter().any(|entry| entry["name"].as_str().is_some_and(|known| known.eq_ignore_ascii_case(name))).then(|| crate::get_offline_player(name))
}

/// bukkit: org/bukkit/Bukkit.advancementIterator, org/bukkit/Server.advancementIterator
pub fn advancement_iterator() -> Vec<crate::core2::Advancement> {
    let names = query(json!({ "op": "core_data_tree", "path": "advancement" }));
    names.as_array().into_iter().flatten().filter_map(Value::as_str).filter_map(crate::core2::get_advancement).collect()
}

/// bukkit: org/bukkit/Bukkit.getBossBars, org/bukkit/Server.getBossBars
///
/// Every bar made with a key: the game's own `/bossbar`, and this plugin's.
pub fn get_boss_bars() -> Vec<BossBar> {
    query(json!({ "op": "core_boss_bars" })).as_array().into_iter().flatten().filter_map(|id| Some(BossBar { id: id.as_str()?.to_owned() })).collect()
}

/// bukkit: org/bukkit/Bukkit.getBossBar, org/bukkit/Server.getBossBar
pub fn get_boss_bar(key: &str) -> Option<BossBar> {
    let id = key_of(key);
    (!query(json!({ "op": "boss_bar", "id": id })).is_null()).then_some(BossBar { id })
}

/// bukkit: org/bukkit/Bukkit.removeBossBar, org/bukkit/Server.removeBossBar
pub fn remove_boss_bar(key: &str) -> bool {
    let id = key_of(key);
    let known = !query(json!({ "op": "boss_bar", "id": id })).is_null();
    if known {
        run_command(&format!("bossbar remove {id}"));
    }
    known
}

fn bar_flag_note(flag: &str) -> Option<&'static str> {
    Some(match flag {
        "DARKEN_SKY" => "darken_sky",
        "PLAY_BOSS_MUSIC" => "play_boss_music",
        "CREATE_FOG" => "create_fog",
        _ => return None,
    })
}

impl BossBar {
    /// bukkit: org/bukkit/boss/BossBar.show
    pub fn show(&self) {
        self.set_visible(true);
    }
    /// bukkit: org/bukkit/boss/BossBar.hide
    pub fn hide(&self) {
        self.set_visible(false);
    }
    /// bukkit: org/bukkit/boss/BossBar.hasFlag
    pub fn has_flag(&self, flag: &str) -> bool {
        bar_flag_note(flag).is_some_and(|name| query(json!({ "op": "core_boss_flag", "id": self.id() }))[name].as_bool().unwrap_or(false))
    }
    /// bukkit: org/bukkit/boss/BossBar.addFlag
    pub fn add_flag(&self, flag: &str) {
        self.set_flag(flag, true);
    }
    /// bukkit: org/bukkit/boss/BossBar.removeFlag
    pub fn remove_flag(&self, flag: &str) {
        self.set_flag(flag, false);
    }
    fn set_flag(&self, flag: &str, on: bool) {
        if let Some(name) = bar_flag_note(flag) {
            query(json!({ "op": "core_boss_flag_set", "id": self.id(), "flag": name, "on": on }));
        }
    }
}

/// bukkit: org/bukkit/Bukkit.setWhitelistEnforced, org/bukkit/Server.setWhitelistEnforced
pub fn set_whitelist_enforced(on: bool) {
    query(json!({ "op": "core_set_enforced", "on": on }));
}

/// bukkit: org/bukkit/Bukkit.setSpawnRadius, org/bukkit/Server.setSpawnRadius
pub fn set_spawn_radius(radius: i32) {
    query(json!({ "op": "core_set_spawn_radius", "radius": radius }));
}

/// bukkit: org/bukkit/Bukkit.getAllowEnd, org/bukkit/Server.getAllowEnd
///
/// `settings.allow-end` of `bukkit.yml`. Off, nothing gets to the End: portals stop taking
/// anyone, `/execute in` doesn't know it, and `get_world` doesn't give it.
pub fn get_allow_end() -> bool {
    bukkit_yml().at("settings.allow-end").and_then(Value::as_bool).unwrap_or(true)
}

/// bukkit: org/bukkit/Bukkit.getAllowNether, org/bukkit/Server.getAllowNether
///
/// `misc.enable-nether` of `config/paper-global.yml`. Off, as `get_allow_end`.
pub fn get_allow_nether() -> bool {
    crate::core2::get_server_config("config/paper-global.yml").at("misc.enable-nether").and_then(Value::as_bool).unwrap_or(true)
}

/// bukkit: org/bukkit/Bukkit.forcesDefaultGameMode, org/bukkit/Server.forcesDefaultGameMode
pub fn forces_default_game_mode() -> bool {
    property("force-gamemode").is_some_and(|value| value == "true")
}

/// bukkit: org/bukkit/Bukkit.getInitialEnabledPacks, org/bukkit/Server.getInitialEnabledPacks
pub fn get_initial_enabled_packs() -> Vec<String> {
    property("initial-enabled-packs").unwrap_or_else(|| "vanilla".to_owned()).split(',').filter(|name| !name.is_empty()).map(str::to_owned).collect()
}

/// bukkit: org/bukkit/Bukkit.getInitialDisabledPacks, org/bukkit/Server.getInitialDisabledPacks
pub fn get_initial_disabled_packs() -> Vec<String> {
    property("initial-disabled-packs").unwrap_or_default().split(',').filter(|name| !name.is_empty()).map(str::to_owned).collect()
}

/// `org.bukkit.packs.ResourcePack`: the pack `server.properties` names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcePack {
    id: String,
    url: String,
    hash: String,
    required: bool,
    prompt: String,
}

/// bukkit: org/bukkit/Bukkit.getServerResourcePack, org/bukkit/Server.getServerResourcePack
pub fn get_server_resource_pack() -> Option<ResourcePack> {
    let url = crate::server::get_resource_pack();
    (!url.is_empty()).then(|| ResourcePack { id: property("resource-pack-id").unwrap_or_default(), url, hash: crate::server::get_resource_pack_hash(), required: crate::server::is_resource_pack_required(), prompt: crate::server::get_resource_pack_prompt() })
}

impl ResourcePack {
    /// bukkit: org/bukkit/packs/ResourcePack.getId
    pub fn get_id(&self) -> &str {
        &self.id
    }
    /// bukkit: org/bukkit/packs/ResourcePack.getUrl
    pub fn get_url(&self) -> &str {
        &self.url
    }
    /// bukkit: org/bukkit/packs/ResourcePack.getHash
    pub fn get_hash(&self) -> &str {
        &self.hash
    }
    /// bukkit: org/bukkit/packs/ResourcePack.isRequired
    pub fn is_required(&self) -> bool {
        self.required
    }
    /// bukkit: org/bukkit/packs/ResourcePack.getPrompt
    pub fn get_prompt(&self) -> &str {
        &self.prompt
    }
}

/// bukkit: org/bukkit/Bukkit.getConnectionThrottle, org/bukkit/Server.getConnectionThrottle
///
/// What `bukkit.yml` says, which is what the handshake goes by.
pub fn get_connection_throttle() -> i64 {
    bukkit_yml().at("settings.connection-throttle").and_then(Value::as_i64).unwrap_or(4000)
}

/// bukkit: org/bukkit/Bukkit.getWarningState, org/bukkit/Server.getWarningState
pub fn get_warning_state() -> &'static str {
    warning_state_value(&bukkit_yml().get_string("settings.deprecated-verbose").unwrap_or_else(|| "default".to_owned()))
}

/// bukkit: org/bukkit/Warning$WarningState.value
pub fn warning_state_value(text: &str) -> &'static str {
    match text.to_ascii_lowercase().as_str() {
        "true" | "on" | "yes" => "ON",
        "false" | "off" | "no" => "OFF",
        _ => "DEFAULT",
    }
}

/// bukkit: org/bukkit/Warning$WarningState.printFor
///
/// `warning_on` is `@Warning(true)`'s `value`, if the method has one.
pub fn warning_state_prints_for(state: &str, warning_on: Option<bool>) -> bool {
    match state {
        "DEFAULT" => warning_on.is_none_or(|on| on),
        other => other == "ON",
    }
}

/// bukkit: org/bukkit/Bukkit.getShutdownMessage, org/bukkit/Server.getShutdownMessage
pub fn get_shutdown_message() -> String {
    bukkit_yml().get_string("settings.shutdown-message").unwrap_or_else(|| "Server closed".to_owned())
}

/// bukkit: org/bukkit/Bukkit.shutdownMessage, org/bukkit/Server.shutdownMessage
pub fn shutdown_message() -> Component {
    adventure::legacy(&get_shutdown_message())
}

const DEFAULT_PERMISSION_MESSAGE: &str = "I'm sorry, but you do not have permission to perform this command. Please contact the server administrators if you believe that this is in error.";

/// bukkit: org/bukkit/Bukkit.getPermissionMessage, org/bukkit/Server.getPermissionMessage
pub fn get_permission_message() -> String {
    format!("{}{DEFAULT_PERMISSION_MESSAGE}", crate::chat_color::RED)
}

/// bukkit: org/bukkit/Bukkit.permissionMessage, org/bukkit/Server.permissionMessage
pub fn permission_message() -> Component {
    adventure::legacy(&get_permission_message())
}

/// bukkit: org/bukkit/Bukkit.getVersionMessage
pub fn get_version_message() -> String {
    format!("This server is running {} version {} (Implementing API version {})", crate::get_name(), crate::get_version(), crate::get_bukkit_version())
}

/// bukkit: org/bukkit/Bukkit.suggestPlayerNamesWhenNullTabCompletions, org/bukkit/Server.suggestPlayerNamesWhenNullTabCompletions
///
/// `commands.suggest-player-names-when-null-tab-completions` of `config/paper-global.yml`.
pub fn suggest_player_names_when_null_tab_completions() -> bool {
    crate::core2::get_server_config("config/paper-global.yml").at("commands.suggest-player-names-when-null-tab-completions").and_then(Value::as_bool).unwrap_or(true)
}

/// bukkit: org/bukkit/Bukkit.shouldSendChatPreviews, org/bukkit/Server.shouldSendChatPreviews
pub fn should_send_chat_previews() -> bool {
    false
}

/// bukkit: org/bukkit/Bukkit.getCommandAliases, org/bukkit/Server.getCommandAliases
///
/// What `commands.yml` says, as Bukkit reads it.
pub fn get_command_aliases() -> BTreeMap<String, Vec<String>> {
    let config = crate::config::YamlConfiguration::load_configuration(&server_folder().join("commands.yml"));
    config.get_keys("aliases").into_iter().map(|name| {
        let lines = config.get_string_list(&format!("aliases.{name}"));
        (name, lines)
    }).collect()
}

/// bukkit: org/bukkit/Bukkit.getScoreboardCriteria, org/bukkit/Server.getScoreboardCriteria
pub fn get_scoreboard_criteria(name: &str) -> Criteria {
    Criteria::create(name)
}

/// bukkit: org/bukkit/Bukkit.getCraftingRecipe, org/bukkit/Server.getCraftingRecipe
///
/// `grid` is the nine slots, or four, row by row.
pub fn get_crafting_recipe(grid: &[Option<ItemStack>]) -> Option<crate::core2::GameRecipe> {
    let key = query(json!({ "op": "core_recipe_key", "grid": grid_json(grid) }));
    crate::core2::get_recipe(&NamespacedKey::minecraft(key.as_str()?))
}

fn grid_json(grid: &[Option<ItemStack>]) -> Value {
    Value::Array(grid.iter().map(|stack| stack.as_ref().map_or(Value::Null, ItemStack::to_json)).collect())
}

/// bukkit: org/bukkit/Bukkit.craftItem, org/bukkit/Server.craftItem
///
/// What the grid makes, as Java's says: `craft_item_result` says what's left of it.
pub fn craft_item(grid: &[Option<ItemStack>]) -> Option<ItemStack> {
    ItemStack::read(&query(json!({ "op": "core_craft", "grid": grid_json(grid) })))
}

/// bukkit: org/bukkit/Bukkit.removeRecipes, org/bukkit/Server.removeRecipes
pub fn remove_recipes(keys: &[NamespacedKey]) -> bool {
    keys.iter().fold(false, |any, key| crate::extra::remove_recipe(key) || any)
}

/// bukkit: org/bukkit/Bukkit.resetRecipes, org/bukkit/Server.resetRecipes
///
/// The ones plugins added go, so the game's own are what's left.
pub fn reset_recipes() {
    let _ = std::fs::remove_dir_all(world_folder().join("datapacks/plugin-recipes/data"));
    query(json!({ "op": "core_recipes_back" }));
}

/// bukkit: org/bukkit/Bukkit.clearRecipes, org/bukkit/Server.clearRecipes
///
/// The game's table, at once: clients hear with `update_recipes`, and `/reload` brings them back, as Java's.
pub fn clear_recipes() {
    let _ = std::fs::remove_dir_all(world_folder().join("datapacks/plugin-recipes/data"));
    query(json!({ "op": "core_recipes_clear" }));
}

/// bukkit: org/bukkit/Bukkit.reloadCommandAliases, org/bukkit/Server.reloadCommandAliases
///
/// Nothing is held to reload: `get_command_aliases` reads the file each time.
pub fn reload_command_aliases() -> bool {
    true
}

/// bukkit: org/bukkit/Bukkit.reload, org/bukkit/Server.reload
///
/// At the end of the tick: plugins off, their libraries read again and started, the game's data read again.
pub fn reload() {
    query(json!({ "op": "core_reload_all" }));
}

/// bukkit: org/bukkit/Bukkit.updateRecipes, org/bukkit/Server.updateRecipes
///
/// Every player is sent the recipes and their book again.
pub fn update_recipes() {
    query(json!({ "op": "core_update_resources" }));
}

/// bukkit: org/bukkit/Bukkit.updateResources, org/bukkit/Server.updateResources
///
/// Every player is sent the tags and recipes again.
pub fn update_resources() {
    query(json!({ "op": "core_update_resources" }));
}

// ---- A sender that keeps what it's told ----

/// `org.bukkit.command.BufferedCommandSender`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BufferedCommandSender {
    buffer: String,
}

impl BufferedCommandSender {
    /// bukkit: org/bukkit/command/BufferedCommandSender.sendMessage
    pub fn send_message(&mut self, text: &str) {
        self.buffer.push_str(text);
        self.buffer.push('\n');
    }
    /// bukkit: org/bukkit/command/BufferedCommandSender.getBuffer
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }
    /// bukkit: org/bukkit/command/BufferedCommandSender.reset
    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

// ---- Server icons ----

fn base64(bytes: &[u8]) -> String {
    const LETTERS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for group in bytes.chunks(3) {
        let number = group.iter().fold(0_u32, |all, byte| (all << 8) | u32::from(*byte)) << (8 * (3 - group.len()));
        for index in 0..4 {
            if index <= group.len() {
                out.push(LETTERS[(number >> (18 - 6 * index)) as usize & 63] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// `org.bukkit.util.CachedServerIcon`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedServerIcon {
    data: Option<String>,
}

/// bukkit: org/bukkit/Bukkit.loadServerIcon, org/bukkit/Server.loadServerIcon
pub fn load_server_icon(file: &Path) -> Option<CachedServerIcon> {
    let bytes = std::fs::read(file).ok()?;
    (bytes.starts_with(&[0x89, b'P', b'N', b'G'])).then(|| CachedServerIcon { data: Some(format!("data:image/png;base64,{}", base64(&bytes))) })
}

/// bukkit: org/bukkit/Bukkit.getServerIcon, org/bukkit/Server.getServerIcon
pub fn get_server_icon() -> Option<CachedServerIcon> {
    load_server_icon(&server_folder().join("server-icon.png"))
}

impl CachedServerIcon {
    pub(crate) fn of_data(data: &str) -> Self { Self { data: Some(data.to_owned()) } }
    /// bukkit: org/bukkit/util/CachedServerIcon.getData
    pub fn get_data(&self) -> Option<&str> {
        self.data.as_deref()
    }
    /// bukkit: org/bukkit/util/CachedServerIcon.isEmpty
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }
}

// ---- Score criteria ----

/// `org.bukkit.scoreboard.Criteria`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Criteria {
    name: String,
}

impl Criteria {
    /// bukkit: org/bukkit/scoreboard/Criteria.create
    pub fn create(name: &str) -> Self {
        Self { name: name.to_owned() }
    }
    /// bukkit: org/bukkit/scoreboard/Criteria.statistic
    ///
    /// `kind` is the game's stat type (`mined`), `of` what it counts.
    pub fn statistic(kind: &str, of: &str) -> Self {
        Self::create(&format!("minecraft.{kind}:{}", of.replace(':', ".")))
    }
    /// bukkit: org/bukkit/scoreboard/Criteria.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/scoreboard/Criteria.isReadOnly
    pub fn is_read_only(&self) -> bool {
        matches!(self.name.as_str(), "air" | "armor" | "food" | "health" | "level" | "xp")
    }
    /// bukkit: org/bukkit/scoreboard/Criteria.getDefaultRenderType
    pub fn get_default_render_type(&self) -> &'static str {
        if self.name == "health" { "HEARTS" } else { "INTEGER" }
    }
}

impl crate::scoreboard::Objective {
    /// bukkit: org/bukkit/scoreboard/Objective.isModifiable
    pub fn is_modifiable(&self) -> bool {
        !Criteria::create(&self.get_criteria()).is_read_only()
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getRenderType
    pub fn get_render_type(&self) -> String {
        let board = query(json!({ "op": "core_objective", "name": self.get_name() }));
        board["render_type"].as_str().unwrap_or("integer").to_ascii_uppercase()
    }
}

impl crate::scoreboard::Scoreboard {
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getEntityTeam
    pub fn get_entity_team(&self, entity: &Entity) -> Option<crate::scoreboard::Team> {
        self.get_entry_team(&entity.get_unique_id())
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.resetScoresFor
    pub fn reset_scores_for(&self, entity: &Entity) {
        self.reset_scores(&entity.get_unique_id());
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getScoresFor, org/bukkit/scoreboard/Scoreboard.getScores
    pub fn get_scores(&self, entry: &str) -> Vec<crate::scoreboard::Score> {
        self.get_objectives().into_iter().map(|objective| objective.get_score(entry)).filter(crate::scoreboard::Score::is_score_set).collect()
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getPlayers
    pub fn get_players(&self) -> Vec<OfflinePlayer> {
        self.get_entries().iter().map(|name| crate::get_offline_player(name)).collect()
    }
}

impl crate::scoreboard::Score {
    /// bukkit: org/bukkit/scoreboard/Score.getPlayer
    pub fn get_player(&self) -> OfflinePlayer {
        crate::get_offline_player(self.get_entry())
    }
    /// bukkit: org/bukkit/scoreboard/Score.isTriggerable
    pub fn is_triggerable(&self) -> bool {
        let board = query(json!({ "op": "scoreboard" }));
        !board["scores"][self.get_entry()][self.get_objective().get_name()]["locked"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/scoreboard/Score.setTriggerable
    pub fn set_triggerable(&self, on: bool) {
        query(json!({ "op": "core_score_lock", "entry": self.get_entry(), "objective": self.get_objective().get_name(), "locked": !on }));
    }
}

impl crate::scoreboard::Team {
    /// bukkit: org/bukkit/scoreboard/Team.getOption
    pub fn get_option(&self, option: &str) -> &'static str {
        let key = match option {
            "NAME_TAG_VISIBILITY" => "name_tag_visibility",
            "DEATH_MESSAGE_VISIBILITY" => "death_message_visibility",
            _ => "collision_rule",
        };
        let value = query(json!({ "op": "core_team", "name": self.get_name() }))[key].as_str().unwrap_or("always").to_owned();
        match value.as_str() {
            "Never" => "NEVER",
            "HideForOwnTeam" | "PushOtherTeams" => "FOR_OTHER_TEAMS",
            "HideForOtherTeams" | "PushOwnTeam" => "FOR_OWN_TEAM",
            _ => "ALWAYS",
        }
    }
    /// bukkit: org/bukkit/scoreboard/Team.getNameTagVisibility
    pub fn get_name_tag_visibility(&self) -> &'static str {
        match self.get_option("NAME_TAG_VISIBILITY") {
            "NEVER" => "NEVER",
            "FOR_OTHER_TEAMS" => "HIDE_FOR_OWN_TEAM",
            "FOR_OWN_TEAM" => "HIDE_FOR_OTHER_TEAMS",
            _ => "ALWAYS",
        }
    }
    /// bukkit: org/bukkit/scoreboard/Team.setNameTagVisibility
    pub fn set_name_tag_visibility(&self, visibility: &str) {
        let status = match visibility {
            "NEVER" => "NEVER",
            "HIDE_FOR_OWN_TEAM" => "FOR_OTHER_TEAMS",
            "HIDE_FOR_OTHER_TEAMS" => "FOR_OWN_TEAM",
            _ => "ALWAYS",
        };
        self.set_option("NAME_TAG_VISIBILITY", status);
    }
    /// bukkit: org/bukkit/scoreboard/Team.color
    pub fn color(&self) -> Option<String> {
        self.get_color().map(|color| color.to_ascii_lowercase())
    }
    /// bukkit: org/bukkit/scoreboard/Team.getPlayers
    pub fn get_players(&self) -> Vec<OfflinePlayer> {
        self.get_entries().iter().map(|name| crate::get_offline_player(name)).collect()
    }
}

// ---- Tables, from the API's own enums ----

/// `org.bukkit.Effect`: name, id, whether it is a sound, and the class of its data, if any.
const EFFECTS: [(&str, i32, bool, Option<&str>); 103] = [
    ("DISPENSER_DISPENSE", 1000, true, None),
    ("DISPENSER_FAIL", 1001, true, None),
    ("DISPENSER_PROJECTILE_LAUNCH", 1002, true, None),
    ("DOOR_TOGGLE", 1006, true, None),
    ("IRON_DOOR_TOGGLE", 1005, true, None),
    ("TRAPDOOR_TOGGLE", 1007, true, None),
    ("IRON_TRAPDOOR_TOGGLE", 1037, true, None),
    ("FENCE_GATE_TOGGLE", 1008, true, None),
    ("DOOR_CLOSE", 1012, true, None),
    ("IRON_DOOR_CLOSE", 1011, true, None),
    ("TRAPDOOR_CLOSE", 1013, true, None),
    ("IRON_TRAPDOOR_CLOSE", 1036, true, None),
    ("FENCE_GATE_CLOSE", 1014, true, None),
    ("ENDEREYE_LAUNCH", 1003, true, None),
    ("FIREWORK_SHOOT", 1004, true, None),
    ("EXTINGUISH", 1009, true, Some("Boolean")),
    ("RECORD_STOP", 1011, true, None),
    ("GHAST_SHRIEK", 1015, true, None),
    ("GHAST_SHOOT", 1016, true, None),
    ("ENDER_DRAGON_SHOOT", 1017, true, None),
    ("BLAZE_SHOOT", 1018, true, None),
    ("ZOMBIE_CHEW_WOODEN_DOOR", 1019, true, None),
    ("ZOMBIE_CHEW_IRON_DOOR", 1020, true, None),
    ("ZOMBIE_DESTROY_DOOR", 1021, true, None),
    ("WITHER_BREAK_BLOCK", 1022, true, None),
    ("WITHER_SPAWNED", 1023, true, None),
    ("WITHER_SHOOT", 1024, true, None),
    ("BAT_TAKEOFF", 1025, true, None),
    ("ZOMBIE_INFECT", 1026, true, None),
    ("ZOMBIE_CONVERTED_TO_VILLAGER", 1027, true, None),
    ("ENDER_DRAGON_DEATH", 1028, true, None),
    ("ANVIL_BREAK", 1029, true, None),
    ("ANVIL_USE", 1030, true, None),
    ("ANVIL_LAND", 1031, true, None),
    ("PORTAL_TRAVEL", 1032, true, None),
    ("CHORUS_FLOWER_GROW", 1033, true, None),
    ("CHORUS_FLOWER_DEATH", 1034, true, None),
    ("BREWING_STAND_BREW", 1035, true, None),
    ("END_PORTAL_CREATED_IN_OVERWORLD", 1038, true, None),
    ("PHANTOM_BITE", 1039, true, None),
    ("ZOMBIE_CONVERTED_TO_DROWNED", 1040, true, None),
    ("HUSK_CONVERTED_TO_ZOMBIE", 1041, true, None),
    ("GRINDSTONE_USE", 1042, true, None),
    ("BOOK_PAGE_TURN", 1043, true, None),
    ("SMITHING_TABLE_USE", 1044, true, None),
    ("POINTED_DRIPSTONE_LAND", 1045, true, None),
    ("POINTED_DRIPSTONE_DRIP_LAVA_INTO_CAULDRON", 1046, true, None),
    ("POINTED_DRIPSTONE_DRIP_WATER_INTO_CAULDRON", 1047, true, None),
    ("SKELETON_CONVERTED_TO_STRAY", 1048, true, None),
    ("CRAFTER_CRAFT", 1049, true, None),
    ("CRAFTER_FAIL", 1050, true, None),
    ("WIND_CHARGE_SHOOT", 1051, true, None),
    ("SULFUR_SPIKE_LAND", 1052, true, None),
    ("SPELL_POTION_SPLASH", 1053, true, None),
    ("INSTANT_POTION_SPLASH", 1054, true, None),
    ("COMPOSTER_FILL_ATTEMPT", 1500, false, Some("Boolean")),
    ("LAVA_INTERACT", 1501, false, None),
    ("REDSTONE_TORCH_BURNOUT", 1502, false, None),
    ("END_PORTAL_FRAME_FILL", 1503, false, None),
    ("DRIPPING_DRIPSTONE", 1504, false, None),
    ("BONE_MEAL_USE", 1505, false, Some("Integer")),
    ("SMOKE_SHOOT", 2000, false, Some("BlockFace")),
    ("DESTROY_BLOCK_WITH_SOUND", 2001, false, Some("BlockData")),
    ("POTION_BREAK", 2002, false, Some("Color")),
    ("ENDER_SIGNAL", 2003, false, None),
    ("MOBSPAWNER_FLAMES", 2004, false, None),
    ("ENDER_DRAGON_BREATH", 2006, false, Some("Boolean")),
    ("INSTANT_POTION_BREAK", 2007, false, Some("Color")),
    ("ENDER_DRAGON_DESTROY_BLOCK", 2008, false, None),
    ("SPONGE_DRY", 2009, false, None),
    ("WHITE_SMOKE_SHOOT", 2010, false, Some("BlockFace")),
    ("BEE_GROWTH", 2011, false, Some("Integer")),
    ("TURTLE_EGG_PLACEMENT", 2012, false, Some("Integer")),
    ("SMASH_ATTACK", 2013, false, Some("Integer")),
    ("DESTROY_BLOCK", 2014, false, Some("BlockData")),
    ("ENDER_DRAGON_EGG_TELEPORT", 2015, false, Some("Integer")),
    ("SHULKER_TELEPORT", 2016, false, Some("Integer")),
    ("CONSUME_EFFECT_TELEPORT", 2017, false, Some("Integer")),
    ("ENDERMAN_TELEPORT", 2018, false, Some("Integer")),
    ("DESTROY_PROGRESS", 2019, false, Some("BlockFace")),
    ("DESTROY_PROGRESS_WITH_SOUND", 2020, false, Some("BlockFace")),
    ("END_GATEWAY_SPAWN", 3000, false, None),
    ("ENDER_DRAGON_GROWL", 3001, true, None),
    ("ELECTRIC_SPARK", 3002, false, Some("Axis")),
    ("COPPER_WAX_ON", 3003, false, None),
    ("COPPER_WAX_OFF", 3004, false, None),
    ("OXIDISED_COPPER_SCRAPE", 3005, false, None),
    ("SCULK_CHARGE", 3006, false, Some("Integer")),
    ("SCULK_SHRIEK", 3007, true, None),
    ("BRUSH_BLOCK_COMPLETE", 3008, false, Some("BlockData")),
    ("EGG_CRACK", 3009, false, None),
    ("GUST_DUST", 3010, false, None),
    ("TRIAL_SPAWNER_SPAWN", 3011, false, Some("Boolean")),
    ("TRIAL_SPAWNER_SPAWN_MOB_AT", 3012, false, Some("Boolean")),
    ("TRIAL_SPAWNER_DETECT_PLAYER", 3013, false, Some("Integer")),
    ("TRIAL_SPAWNER_EJECT_ITEM", 3014, false, None),
    ("VAULT_ACTIVATE", 3015, false, Some("Boolean")),
    ("VAULT_DEACTIVATE", 3016, false, Some("Boolean")),
    ("VAULT_EJECT_ITEM", 3017, false, None),
    ("SPAWN_COBWEB", 3018, false, None),
    ("TRIAL_SPAWNER_DETECT_PLAYER_OMINOUS", 3019, false, Some("Integer")),
    ("TRIAL_SPAWNER_BECOME_OMINOUS", 3020, false, Some("Boolean")),
    ("TRIAL_SPAWNER_SPAWN_ITEM", 3021, false, Some("Boolean")),
];

/// `org.bukkit.EntityEffect`: name, the byte the game sends, the classes it applies to.
const ENTITY_EFFECTS: [(&str, u8, &[&str]); 81] = [
    ("ARROW_PARTICLES", 0, &["Arrow"]),
    ("RABBIT_JUMP", 1, &["Rabbit"]),
    ("RESET_SPAWNER_MINECART_DELAY", 1, &["SpawnerMinecart"]),
    ("HIT", 2, &["LivingEntity"]),
    ("HURT", 2, &["LivingEntity"]),
    ("DEATH", 3, &["Entity"]),
    ("EGG_BREAK", 3, &["Egg"]),
    ("SNOWBALL_BREAK", 3, &["Snowball"]),
    ("PROJECTILE_CRACK", 3, &["Egg", "Snowball"]),
    ("ENTITY_DEATH", 3, &["LivingEntity"]),
    ("FANG_ATTACK", 4, &["EvokerFangs"]),
    ("HOGLIN_ATTACK", 4, &["Hoglin"]),
    ("IRON_GOLEN_ATTACK", 4, &["IronGolem"]),
    ("RAVAGER_ATTACK", 4, &["Ravager"]),
    ("WARDEN_ATTACK", 4, &["Warden"]),
    ("ZOGLIN_ATTACK", 4, &["Zoglin"]),
    ("ENTITY_ATTACK", 4, &["EvokerFangs", "Hoglin", "IronGolem", "Ravager", "Warden", "Zoglin", "Creaking"]),
    ("WOLF_SMOKE", 6, &["Tameable"]),
    ("WOLF_HEARTS", 7, &["Tameable"]),
    ("TAMING_FAILED", 6, &["Tameable"]),
    ("TAMING_SUCCEEDED", 7, &["Tameable"]),
    ("WOLF_SHAKE", 8, &["Wolf"]),
    ("SHEEP_EAT", 10, &["Entity"]),
    ("SHEEP_EAT_GRASS", 10, &["Sheep"]),
    ("IRON_GOLEM_ROSE", 11, &["IronGolem"]),
    ("VILLAGER_HEART", 12, &["Villager"]),
    ("VILLAGER_ANGRY", 13, &["Villager"]),
    ("VILLAGER_HAPPY", 14, &["Villager"]),
    ("WITCH_MAGIC", 15, &["Witch"]),
    ("ZOMBIE_TRANSFORM", 16, &["ZombieVillager"]),
    ("FIREWORK_EXPLODE", 17, &["Firework"]),
    ("LOVE_HEARTS", 18, &["Animals", "Allay"]),
    ("SQUID_ROTATE", 19, &["Squid"]),
    ("ENTITY_POOF", 20, &["Mob"]),
    ("GUARDIAN_TARGET", 21, &["Guardian"]),
    ("SHIELD_BLOCK", 29, &["LivingEntity"]),
    ("SHIELD_BREAK", 30, &["LivingEntity"]),
    ("ARMOR_STAND_HIT", 32, &["ArmorStand"]),
    ("THORNS_HURT", 33, &["LivingEntity"]),
    ("IRON_GOLEM_SHEATH", 34, &["IronGolem"]),
    ("TOTEM_RESURRECT", 35, &["LivingEntity"]),
    ("PROTECTED_FROM_DEATH", 35, &["Entity"]),
    ("HURT_DROWN", 36, &["LivingEntity"]),
    ("HURT_EXPLOSION", 37, &["LivingEntity"]),
    ("DOLPHIN_FED", 38, &["Dolphin"]),
    ("RAVAGER_STUNNED", 39, &["Ravager"]),
    ("CAT_TAME_FAIL", 40, &["Cat"]),
    ("CAT_TAME_SUCCESS", 41, &["Cat"]),
    ("TRUSTING_FAILED", 40, &["Ocelot"]),
    ("TRUSTING_SUCCEEDED", 41, &["Ocelot"]),
    ("VILLAGER_SPLASH", 42, &["Villager"]),
    ("PLAYER_BAD_OMEN_RAID", 43, &["Player"]),
    ("HURT_BERRY_BUSH", 44, &["LivingEntity"]),
    ("FOX_CHEW", 45, &["Fox"]),
    ("TELEPORT_ENDER", 46, &["LivingEntity"]),
    ("BREAK_EQUIPMENT_MAIN_HAND", 47, &["LivingEntity"]),
    ("BREAK_EQUIPMENT_OFF_HAND", 48, &["LivingEntity"]),
    ("BREAK_EQUIPMENT_HELMET", 49, &["LivingEntity"]),
    ("BREAK_EQUIPMENT_CHESTPLATE", 50, &["LivingEntity"]),
    ("BREAK_EQUIPMENT_LEGGINGS", 51, &["LivingEntity"]),
    ("BREAK_EQUIPMENT_BOOTS", 52, &["LivingEntity"]),
    ("HONEY_BLOCK_SLIDE_PARTICLES", 53, &["Entity"]),
    ("HONEY_BLOCK_FALL_PARTICLES", 54, &["LivingEntity"]),
    ("SWAP_HAND_ITEMS", 55, &["LivingEntity"]),
    ("WOLF_SHAKE_STOP", 56, &["Wolf"]),
    ("GOAT_LOWER_HEAD", 58, &["Goat"]),
    ("GOAT_RAISE_HEAD", 59, &["Goat"]),
    ("SPAWN_DEATH_SMOKE", 60, &["LivingEntity"]),
    ("WARDEN_TENDRIL_SHAKE", 61, &["Warden"]),
    ("WARDEN_SONIC_ATTACK", 62, &["Warden"]),
    ("SNIFFER_DIG", 63, &["Sniffer"]),
    ("ARMADILLO_PEEK", 64, &["Armadillo"]),
    ("BODY_BREAK", 65, &["LivingEntity"]),
    ("BREAK_EQUIPMENT_BODY", 65, &["LivingEntity"]),
    ("SHAKE", 66, &["Creaking"]),
    ("DROWN_PARTICLES", 67, &["LivingEntity"]),
    ("BREAK_EQUIPMENT_SADDLE", 68, &["LivingEntity"]),
    ("RAVAGER_ROARED", 69, &["Ravager"]),
    ("TNT_MINECART_IGNITE", 70, &["ExplosiveMinecart"]),
    ("BOAT_SINK", 71, &["Boat"]),
    ("BOAT_LAUNCH", 72, &["Boat"]),
];

/// `org.bukkit.Statistic`s that take a block, item or entity: all others are `UNTYPED`.
const TYPED_STATISTICS: [(&str, &str); 8] = [
    ("BREAK_ITEM", "ITEM"),
    ("CRAFT_ITEM", "ITEM"),
    ("DROP", "ITEM"),
    ("KILL_ENTITY", "ENTITY"),
    ("ENTITY_KILLED_BY", "ENTITY"),
    ("MINE_BLOCK", "BLOCK"),
    ("PICKUP", "ITEM"),
    ("USE_ITEM", "ITEM"),
];

/// `org.bukkit.Particle`s that take data, and the class of it: all others take none.
const PARTICLE_DATA: [(&str, &str); 22] = [
    ("EFFECT", "Spell"),
    ("INSTANT_EFFECT", "Spell"),
    ("ENTITY_EFFECT", "Color"),
    ("DUST", "DustOptions"),
    ("ITEM", "ItemStack"),
    ("BLOCK", "BlockData"),
    ("DRAGON_BREATH", "Float"),
    ("FALLING_DUST", "BlockData"),
    ("FLASH", "Color"),
    ("DUST_COLOR_TRANSITION", "DustTransition"),
    ("VIBRATION", "Vibration"),
    ("SCULK_CHARGE", "Float"),
    ("SHRIEK", "Integer"),
    ("TINTED_LEAVES", "Color"),
    ("DUST_PILLAR", "BlockData"),
    ("BLOCK_CRUMBLE", "BlockData"),
    ("TRAIL", "Trail"),
    ("BLOCK_MARKER", "BlockData"),
    ("GEYSER", "Geyser"),
    ("GEYSER_BASE", "GeyserBase"),
    ("GEYSER_PLUME", "Geyser"),
    ("GEYSER_POOF", "GeyserBase"),
];

// ---- Small value classes ----

/// `org.bukkit.GameRule`: a rule of the game, by its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameRule {
    name: String,
}

impl GameRule {
    fn row(&self) -> Value {
        query(json!({ "op": "core_game_rules" }))[self.name.as_str()].clone()
    }
    /// bukkit: org/bukkit/GameRule.getByName
    pub fn get_by_name(name: &str) -> Option<Self> {
        let rule = Self { name: name.trim_start_matches("minecraft:").to_owned() };
        (!rule.row().is_null()).then_some(rule)
    }
    /// bukkit: org/bukkit/GameRule.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/GameRule.getType
    ///
    /// `Integer` or `Boolean`.
    pub fn get_type(&self) -> &'static str {
        if self.row()["number"].as_bool().unwrap_or(false) { "Integer" } else { "Boolean" }
    }
    /// bukkit: org/bukkit/GameRule.getDefaultValue
    ///
    /// As a number: a boolean rule's is 1 or 0.
    pub fn get_default_value(&self) -> i64 {
        self.row()["default"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/GameRule.translationKey
    pub fn translation_key(&self) -> String {
        format!("gamerule.minecraft.{}", self.name)
    }
}

/// `org.bukkit.Particle$DustOptions`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DustOptions {
    color: Color,
    size: f32,
}

impl DustOptions {
    /// bukkit: org/bukkit/Particle$DustOptions.<init>
    pub fn new(color: Color, size: f32) -> Self {
        Self { color, size }
    }
    /// bukkit: org/bukkit/Particle$DustOptions.getColor
    pub fn get_color(&self) -> Color {
        self.color
    }
    /// bukkit: org/bukkit/Particle$DustOptions.getSize
    pub fn get_size(&self) -> f32 {
        self.size
    }
    /// The options as `/particle` reads them.
    pub fn snbt(&self) -> String {
        format!("{{color:{},scale:{:?}f}}", self.color.as_rgb(), self.size)
    }
}

/// `org.bukkit.Particle$DustTransition`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DustTransition {
    from: DustOptions,
    to: Color,
}

impl DustTransition {
    /// bukkit: org/bukkit/Particle$DustTransition.<init>
    pub fn new(from: Color, to: Color, size: f32) -> Self {
        Self { from: DustOptions::new(from, size), to }
    }
    /// The first colour and the size.
    pub fn from_options(&self) -> DustOptions {
        self.from
    }
    /// bukkit: org/bukkit/Particle$DustTransition.getToColor
    pub fn get_to_color(&self) -> Color {
        self.to
    }
}

/// `org.bukkit.Particle$Trail`.
#[derive(Clone, Debug, PartialEq)]
pub struct Trail {
    target: Location,
    color: Color,
    duration: i32,
}

impl Trail {
    /// bukkit: org/bukkit/Particle$Trail.<init>
    pub fn new(target: Location, color: Color, duration: i32) -> Self {
        Self { target, color, duration }
    }
    /// bukkit: org/bukkit/Particle$Trail.getTarget
    pub fn get_target(&self) -> &Location {
        &self.target
    }
    /// bukkit: org/bukkit/Particle$Trail.getColor
    pub fn get_color(&self) -> Color {
        self.color
    }
    /// bukkit: org/bukkit/Particle$Trail.getDuration
    pub fn get_duration(&self) -> i32 {
        self.duration
    }
}

/// `org.bukkit.Particle$Spell`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spell {
    color: Color,
    power: f32,
}

impl Spell {
    /// bukkit: org/bukkit/Particle$Spell.<init>
    pub fn new(color: Color, power: f32) -> Self {
        Self { color, power }
    }
    /// bukkit: org/bukkit/Particle$Spell.getColor
    pub fn get_color(&self) -> Color {
        self.color
    }
    /// bukkit: org/bukkit/Particle$Spell.getPower
    pub fn get_power(&self) -> f32 {
        self.power
    }
}

/// `org.bukkit.Particle$Geyser` and `$GeyserBase`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geyser {
    water_blocks: i32,
    burst_impulse: f32,
}

impl Geyser {
    /// bukkit: org/bukkit/Particle$Geyser.<init>
    pub fn new(water_blocks: i32) -> Self {
        Self { water_blocks, burst_impulse: 0.0 }
    }
    /// bukkit: org/bukkit/Particle$GeyserBase.<init>
    pub fn base(water_blocks: i32, burst_impulse: f32) -> Self {
        Self { water_blocks, burst_impulse }
    }
    /// bukkit: org/bukkit/Particle$AbstractGeyser.getWaterBlocks, org/bukkit/Particle$Geyser.getWaterBlocks, org/bukkit/Particle$GeyserBase.getWaterBlocks
    pub fn get_water_blocks(&self) -> i32 {
        self.water_blocks
    }
    /// bukkit: org/bukkit/Particle$GeyserBase.getBurstImpulse
    pub fn get_burst_impulse(&self) -> f32 {
        self.burst_impulse
    }
}

/// bukkit: org/bukkit/Particle.getDataType
///
/// The simple name of the class of the particle's data; `Void` for none.
pub fn particle_data_type(particle: &str) -> &'static str {
    PARTICLE_DATA.iter().find(|(name, _)| *name == particle).map_or("Void", |(_, class)| class)
}

/// bukkit: org/bukkit/Sound.getKey, org/bukkit/Sound.key
///
/// A sound here is its key already: `generated::sound::ENTITY_ZOMBIE_AMBIENT`.
pub fn sound_key(sound: &str) -> Option<NamespacedKey> {
    NamespacedKey::from_string(&key_of(sound))
}

/// bukkit: org/bukkit/Rotation.rotateClockwise
pub fn rotate_clockwise(rotation: &str) -> &'static str {
    const ROUND: [&str; 8] = ["NONE", "CLOCKWISE_45", "CLOCKWISE", "CLOCKWISE_135", "FLIPPED", "FLIPPED_45", "COUNTER_CLOCKWISE", "COUNTER_CLOCKWISE_45"];
    ROUND[(ROUND.iter().position(|known| *known == rotation).unwrap_or(0) + 1) % 8]
}

/// bukkit: org/bukkit/Rotation.rotateCounterClockwise
pub fn rotate_counter_clockwise(rotation: &str) -> &'static str {
    const ROUND: [&str; 8] = ["NONE", "CLOCKWISE_45", "CLOCKWISE", "CLOCKWISE_135", "FLIPPED", "FLIPPED_45", "COUNTER_CLOCKWISE", "COUNTER_CLOCKWISE_45"];
    ROUND[(ROUND.iter().position(|known| *known == rotation).unwrap_or(0) + 7) % 8]
}

/// bukkit: org/bukkit/GameMode.isInvulnerable
pub fn game_mode_is_invulnerable(mode: &str) -> bool {
    matches!(mode, "CREATIVE" | "SPECTATOR")
}

/// bukkit: org/bukkit/GameMode.translationKey
pub fn game_mode_translation_key(mode: &str) -> String {
    format!("gameMode.{}", mode.to_ascii_lowercase())
}

/// bukkit: org/bukkit/Difficulty.translationKey
pub fn difficulty_translation_key(difficulty: &str) -> String {
    format!("options.difficulty.{}", difficulty.to_ascii_lowercase())
}

/// bukkit: org/bukkit/FireworkEffect$Type.translationKey
pub fn firework_type_translation_key(kind: &str) -> String {
    let id = match kind {
        "BALL" => 0,
        "BALL_LARGE" => 1,
        "STAR" => 2,
        "CREEPER" => 3,
        _ => 4,
    };
    format!("item.minecraft.firework_star.shape.{}", ["small_ball", "large_ball", "star", "creeper", "burst"][id])
}

const DYE_ORDER: [&str; 16] = ["WHITE", "ORANGE", "MAGENTA", "LIGHT_BLUE", "YELLOW", "LIME", "PINK", "GRAY", "LIGHT_GRAY", "CYAN", "PURPLE", "BLUE", "BROWN", "GREEN", "RED", "BLACK"];

/// bukkit: org/bukkit/DyeColor.getWoolData
pub fn dye_wool_data(dye: &str) -> Option<u8> {
    DYE_ORDER.iter().position(|known| *known == dye).map(|index| index as u8)
}

/// bukkit: org/bukkit/DyeColor.getDyeData
pub fn dye_dye_data(dye: &str) -> Option<u8> {
    dye_wool_data(dye).map(|wool| 15 - wool)
}

/// bukkit: org/bukkit/DyeColor.getByWoolData
pub fn dye_by_wool_data(data: u8) -> Option<&'static str> {
    DYE_ORDER.get(usize::from(data)).copied()
}

/// bukkit: org/bukkit/DyeColor.getByDyeData
pub fn dye_by_dye_data(data: u8) -> Option<&'static str> {
    data.checked_sub(0).filter(|data| *data < 16).and_then(|data| DYE_ORDER.get(usize::from(15 - data)).copied())
}

/// bukkit: org/bukkit/DyeColor.legacyValueOf
pub fn dye_legacy_value_of(name: &str) -> Option<&'static str> {
    let modern = if name == "SILVER" { "LIGHT_GRAY" } else { name };
    DYE_ORDER.iter().copied().find(|known| *known == modern)
}

fn data_of(names: &'static [&'static str], name: &str) -> Option<u8> {
    names.iter().position(|known| *known == name).map(|index| index as u8)
}

/// bukkit: org/bukkit/CoalType.getData
pub fn coal_type_data(name: &str) -> Option<u8> {
    data_of(&["COAL", "CHARCOAL"], name)
}

/// bukkit: org/bukkit/CoalType.getByData
pub fn coal_type_by_data(data: u8) -> Option<&'static str> {
    ["COAL", "CHARCOAL"].get(usize::from(data)).copied()
}

/// bukkit: org/bukkit/CropState.getData
pub fn crop_state_data(name: &str) -> Option<u8> {
    data_of(&["SEEDED", "GERMINATED", "VERY_SMALL", "SMALL", "MEDIUM", "TALL", "VERY_TALL", "RIPE"], name)
}

/// bukkit: org/bukkit/CropState.getByData
pub fn crop_state_by_data(data: u8) -> Option<&'static str> {
    ["SEEDED", "GERMINATED", "VERY_SMALL", "SMALL", "MEDIUM", "TALL", "VERY_TALL", "RIPE"].get(usize::from(data)).copied()
}

/// bukkit: org/bukkit/GrassSpecies.getData
pub fn grass_species_data(name: &str) -> Option<u8> {
    data_of(&["DEAD", "NORMAL", "FERN_LIKE"], name)
}

/// bukkit: org/bukkit/GrassSpecies.getByData
pub fn grass_species_by_data(data: u8) -> Option<&'static str> {
    ["DEAD", "NORMAL", "FERN_LIKE"].get(usize::from(data)).copied()
}

/// bukkit: org/bukkit/SandstoneType.getData
pub fn sandstone_type_data(name: &str) -> Option<u8> {
    data_of(&["CRACKED", "GLYPHED", "SMOOTH"], name)
}

/// bukkit: org/bukkit/SandstoneType.getByData
pub fn sandstone_type_by_data(data: u8) -> Option<&'static str> {
    ["CRACKED", "GLYPHED", "SMOOTH"].get(usize::from(data)).copied()
}

/// bukkit: org/bukkit/TreeSpecies.getData
pub fn tree_species_data(name: &str) -> Option<u8> {
    data_of(&["GENERIC", "REDWOOD", "BIRCH", "JUNGLE", "ACACIA", "DARK_OAK"], name)
}

/// bukkit: org/bukkit/TreeSpecies.getByData
pub fn tree_species_by_data(data: u8) -> Option<&'static str> {
    ["GENERIC", "REDWOOD", "BIRCH", "JUNGLE", "ACACIA", "DARK_OAK"].get(usize::from(data)).copied()
}

/// bukkit: org/bukkit/Effect.getId
pub fn effect_id(effect: &str) -> Option<i32> {
    EFFECTS.iter().find(|row| row.0 == effect).map(|row| row.1)
}

/// bukkit: org/bukkit/Effect.getById
pub fn effect_by_id(id: i32) -> Option<&'static str> {
    EFFECTS.iter().find(|row| row.1 == id).map(|row| row.0)
}

/// bukkit: org/bukkit/Effect.getType
pub fn effect_type(effect: &str) -> Option<&'static str> {
    EFFECTS.iter().find(|row| row.0 == effect).map(|row| if row.2 { "SOUND" } else { "VISUAL" })
}

/// bukkit: org/bukkit/Effect.getData
///
/// The simple name of the class of the effect's data, if it takes any.
pub fn effect_data(effect: &str) -> Option<&'static str> {
    EFFECTS.iter().find(|row| row.0 == effect).and_then(|row| row.3)
}

/// bukkit: org/bukkit/Effect.isApplicable
///
/// Whether data of a class (by its simple name) is what the effect takes.
pub fn effect_is_applicable(effect: &str, class: &str) -> bool {
    effect_data(effect) == Some(class)
}

/// bukkit: org/bukkit/EntityEffect.getData
pub fn entity_effect_data(effect: &str) -> Option<u8> {
    ENTITY_EFFECTS.iter().find(|row| row.0 == effect).map(|row| row.1)
}

/// bukkit: org/bukkit/EntityEffect.getApplicableClasses, org/bukkit/EntityEffect.getApplicable
pub fn entity_effect_applicable(effect: &str) -> &'static [&'static str] {
    ENTITY_EFFECTS.iter().find(|row| row.0 == effect).map_or(&[], |row| row.2)
}

/// What each interface of `org.bukkit.entity` extends, there.
const ENTITY_PARENTS: [(&str, &[&str]); 178] = [("AbstractArrow", &["Projectile"]), ("AbstractCow", &["Animals"]), ("AbstractCubeMob", &["Creature"]), ("AbstractHorse", &["Vehicle", "Tameable"]), ("AbstractNautilus", &["Tameable", "Vehicle"]), ("AbstractSkeleton", &["Monster"]), ("AbstractVillager", &["Breedable", "NPC"]), ("AbstractWindCharge", &["Fireball"]), ("Ageable", &["Creature"]), ("Allay", &["Creature"]), ("Ambient", &["Mob"]), ("Animals", &["Breedable"]), ("AreaEffectCloud", &["Entity"]), ("Armadillo", &["Animals"]), ("ArmorStand", &["LivingEntity"]), ("Arrow", &["AbstractArrow"]), ("Axolotl", &["Animals"]), ("Bat", &["Ambient"]), ("Bee", &["Animals"]), ("Blaze", &["Monster"]), ("BlockDisplay", &["Display"]), ("Boat", &["Vehicle"]), ("Bogged", &["AbstractSkeleton"]), ("Boss", &["Entity"]), ("Breedable", &["Ageable"]), ("Breeze", &["Monster"]), ("BreezeWindCharge", &["AbstractWindCharge"]), ("Camel", &["AbstractHorse", "Sittable"]), ("CamelHusk", &["Camel"]), ("Cat", &["Tameable", "Sittable"]), ("CaveSpider", &["Spider"]), ("ChestBoat", &["Boat"]), ("ChestedHorse", &["AbstractHorse"]), ("Chicken", &["Animals"]), ("ComplexEntityPart", &["Entity"]), ("ComplexLivingEntity", &["LivingEntity"]), ("CopperGolem", &["Golem"]), ("Cow", &["AbstractCow"]), ("Creaking", &["Monster"]), ("Creature", &["Mob"]), ("Creeper", &["Monster"]), ("Cushion", &["Entity"]), ("Damageable", &["Entity"]), ("Display", &["Entity"]), ("Dolphin", &["Ageable", "WaterMob"]), ("Donkey", &["ChestedHorse"]), ("DragonFireball", &["Fireball"]), ("Drowned", &["Zombie"]), ("Egg", &["ThrowableProjectile"]), ("ElderGuardian", &["Guardian"]), ("EnderCrystal", &["Entity"]), ("EnderDragon", &["ComplexLivingEntity", "Boss", "Mob", "Enemy"]), ("EnderDragonPart", &["ComplexEntityPart", "Damageable"]), ("EnderPearl", &["ThrowableProjectile"]), ("EnderSignal", &["Entity"]), ("Enderman", &["Monster"]), ("Endermite", &["Monster"]), ("Enemy", &["LivingEntity"]), ("Evoker", &["Spellcaster"]), ("EvokerFangs", &["Entity"]), ("ExperienceOrb", &["Entity"]), ("Explosive", &["Entity"]), ("FallingBlock", &["Entity"]), ("Fireball", &["Projectile", "Explosive"]), ("Firework", &["Projectile"]), ("Fish", &["WaterMob"]), ("FishHook", &["Projectile"]), ("Flying", &["Mob"]), ("Fox", &["Animals", "Sittable"]), ("Frog", &["Animals"]), ("Ghast", &["Flying", "Enemy"]), ("Giant", &["Monster"]), ("GlowItemFrame", &["ItemFrame"]), ("GlowSquid", &["Squid"]), ("Goat", &["Animals"]), ("Golem", &["Creature"]), ("Guardian", &["Monster"]), ("Hanging", &["Entity"]), ("HappyGhast", &["Vehicle", "Animals"]), ("Hoglin", &["Animals", "Enemy"]), ("Horse", &["AbstractHorse"]), ("HumanEntity", &["LivingEntity"]), ("Husk", &["Zombie"]), ("Illager", &["Raider"]), ("Illusioner", &["Spellcaster"]), ("Interaction", &["Entity"]), ("IronGolem", &["Golem"]), ("Item", &["Entity"]), ("ItemDisplay", &["Display"]), ("ItemFrame", &["Hanging"]), ("LargeFireball", &["SizedFireball"]), ("LeashHitch", &["Hanging"]), ("LightningStrike", &["Entity"]), ("LingeringPotion", &["ThrownPotion"]), ("LivingEntity", &["Damageable"]), ("Llama", &["ChestedHorse"]), ("LlamaSpit", &["Projectile"]), ("MagmaCube", &["AbstractCubeMob", "Enemy"]), ("Mannequin", &["LivingEntity"]), ("Marker", &["Entity"]), ("Minecart", &["Vehicle"]), ("Mob", &["LivingEntity"]), ("Monster", &["Creature", "Enemy"]), ("Mule", &["ChestedHorse"]), ("MushroomCow", &["AbstractCow"]), ("NPC", &["Creature"]), ("Nautilus", &["AbstractNautilus"]), ("Ocelot", &["Animals"]), ("OminousItemSpawner", &["Entity"]), ("Painting", &["Hanging"]), ("Panda", &["Animals", "Sittable"]), ("Parched", &["AbstractSkeleton"]), ("Parrot", &["Tameable", "Sittable"]), ("Phantom", &["Flying", "Enemy"]), ("Pig", &["Steerable", "Vehicle"]), ("PigZombie", &["Zombie"]), ("Piglin", &["PiglinAbstract"]), ("PiglinAbstract", &["Monster", "Ageable"]), ("PiglinBrute", &["PiglinAbstract"]), ("Pillager", &["Illager"]), ("Player", &["HumanEntity"]), ("PolarBear", &["Animals"]), ("Projectile", &["Entity"]), ("PufferFish", &["Fish"]), ("Rabbit", &["Animals"]), ("Raider", &["Monster"]), ("Ravager", &["Raider"]), ("Sheep", &["Animals"]), ("Shulker", &["Golem", "Enemy"]), ("ShulkerBullet", &["Projectile"]), ("Silverfish", &["Monster"]), ("Sittable", &["Entity"]), ("SizedFireball", &["Fireball"]), ("Skeleton", &["AbstractSkeleton"]), ("SkeletonHorse", &["AbstractHorse"]), ("Slime", &["AbstractCubeMob", "Enemy"]), ("SmallFireball", &["SizedFireball"]), ("Sniffer", &["Animals"]), ("Snowball", &["ThrowableProjectile"]), ("Snowman", &["Golem"]), ("SpectralArrow", &["AbstractArrow"]), ("Spellcaster", &["Illager"]), ("Spider", &["Monster"]), ("SplashPotion", &["ThrownPotion"]), ("Squid", &["Ageable", "WaterMob"]), ("Steerable", &["Animals"]), ("Stray", &["AbstractSkeleton"]), ("Strider", &["Steerable", "Vehicle"]), ("SulfurCube", &["AbstractCubeMob", "Ageable"]), ("TNTPrimed", &["Explosive"]), ("Tadpole", &["Fish"]), ("Tameable", &["Animals"]), ("TextDisplay", &["Display"]), ("ThrowableProjectile", &["Projectile"]), ("ThrownExpBottle", &["ThrowableProjectile"]), ("ThrownPotion", &["ThrowableProjectile"]), ("TippedArrow", &["Arrow"]), ("TraderLlama", &["Llama"]), ("Trident", &["AbstractArrow", "ThrowableProjectile"]), ("Turtle", &["Animals"]), ("Vehicle", &["Entity"]), ("Vex", &["Monster"]), ("Villager", &["AbstractVillager"]), ("Vindicator", &["Illager"]), ("WanderingTrader", &["AbstractVillager"]), ("Warden", &["Monster"]), ("WaterMob", &["Creature"]), ("WindCharge", &["AbstractWindCharge"]), ("Witch", &["Raider"]), ("Wither", &["Monster", "Boss"]), ("WitherSkeleton", &["AbstractSkeleton"]), ("WitherSkull", &["Fireball"]), ("Wolf", &["Tameable", "Sittable"]), ("Zoglin", &["Monster", "Ageable"]), ("Zombie", &["Monster", "Ageable"]), ("ZombieHorse", &["AbstractHorse"]), ("ZombieNautilus", &["AbstractNautilus"]), ("ZombieVillager", &["Zombie"])];

/// Java's `isAssignableFrom`, by simple names.
fn entity_class_is(class: &str, target: &str) -> bool {
    class == target || ENTITY_PARENTS.iter().find(|row| row.0 == class).is_some_and(|row| row.1.iter().any(|parent| entity_class_is(parent, target)))
}

/// bukkit: org/bukkit/EntityEffect.isApplicableTo
///
/// `class` by its simple name, as Java's `Class`.
pub fn entity_effect_is_applicable_to(effect: &str, class: &str) -> bool {
    entity_effect_applicable(effect).iter().any(|target| entity_class_is(class, target))
}

/// bukkit: org/bukkit/Statistic.getType
pub fn statistic_type(statistic: &str) -> &'static str {
    TYPED_STATISTICS.iter().find(|row| row.0 == statistic).map_or("UNTYPED", |row| row.1)
}

/// bukkit: org/bukkit/Statistic.isSubstatistic
pub fn statistic_is_substatistic(statistic: &str) -> bool {
    statistic_type(statistic) != "UNTYPED"
}

/// bukkit: org/bukkit/Statistic.isBlock
pub fn statistic_is_block(statistic: &str) -> bool {
    statistic_type(statistic) == "BLOCK"
}

/// bukkit: org/bukkit/Statistic.getKey
pub fn statistic_key(statistic: &str) -> NamespacedKey {
    NamespacedKey::minecraft(&statistic.to_ascii_lowercase())
}

/// bukkit: org/bukkit/Note$Tone.getId
pub fn tone_id(tone: &str, sharped: bool) -> Option<u8> {
    let (id, sharpable) = match tone {
        "G" => (1, true),
        "A" => (3, true),
        "B" => (5, false),
        "C" => (6, true),
        "D" => (8, true),
        "E" => (10, false),
        "F" => (11, true),
        _ => return None,
    };
    Some((if sharped && sharpable { id + 1 } else { id }) % 12)
}

/// bukkit: org/bukkit/Note$Tone.isSharpable
pub fn tone_is_sharpable(tone: &str) -> bool {
    !matches!(tone, "B" | "E")
}

/// bukkit: org/bukkit/Note$Tone.isSharped
pub fn tone_is_sharped(tone: &str, id: u8) -> Option<bool> {
    if tone_id(tone, false) == Some(id) {
        Some(false)
    } else if tone_id(tone, true) == Some(id) {
        Some(true)
    } else {
        None
    }
}

/// bukkit: org/bukkit/Note$Tone.getById
pub fn tone_by_id(id: u8) -> Option<&'static str> {
    ["G", "A", "B", "C", "D", "E", "F"].into_iter().find(|tone| tone_id(tone, false) == Some(id) || tone_id(tone, true) == Some(id))
}

/// bukkit: org/bukkit/Art.getId
///
/// A place in `Art.values()`: Paper's own says this number means nothing.
pub fn art_id(name: &str) -> Option<usize> {
    crate::enums::art::values().iter().position(|known| *known == name)
}

/// bukkit: org/bukkit/Art.getById
pub fn art_by_id(id: usize) -> Option<&'static str> {
    crate::enums::art::values().get(id).copied()
}

/// bukkit: org/bukkit/SoundCategory.soundSource
///
/// Adventure's `Sound.Source` of the same name.
pub fn sound_category_source(category: &str) -> &str {
    category
}

/// bukkit: org/bukkit/scoreboard/DisplaySlot.getId
pub fn display_slot_id(slot: &str) -> String {
    match slot {
        "SIDEBAR" => "sidebar".to_owned(),
        "PLAYER_LIST" => "list".to_owned(),
        "BELOW_NAME" => "below_name".to_owned(),
        other => other.to_ascii_lowercase().replace("sidebar_team_", "sidebar.team."),
    }
}

impl Color {
    /// bukkit: org/bukkit/Color.setAlpha
    pub fn set_alpha(self, alpha: u8) -> Self {
        Self { alpha, ..self }
    }
    /// bukkit: org/bukkit/Color.serialize
    pub fn serialize(self) -> Value {
        json!({ "ALPHA": self.alpha, "RED": self.red, "GREEN": self.green, "BLUE": self.blue })
    }
    /// bukkit: org/bukkit/Color.deserialize
    pub fn deserialize(map: &Value) -> Option<Self> {
        let channel = |key: &str| map[key].as_u64().and_then(|value| u8::try_from(value).ok());
        Some(Self { red: channel("RED")?, green: channel("GREEN")?, blue: channel("BLUE")?, alpha: channel("ALPHA").unwrap_or(255) })
    }
    /// bukkit: org/bukkit/Color.mixColors
    pub fn mix_colors(self, colors: &[Self]) -> Self {
        let (mut red, mut green, mut blue) = (i32::from(self.red), i32::from(self.green), i32::from(self.blue));
        let mut most = red.max(green).max(blue);
        for color in colors {
            let (r, g, b) = (i32::from(color.red), i32::from(color.green), i32::from(color.blue));
            red += r;
            green += g;
            blue += b;
            most += r.max(g).max(b);
        }
        let count = colors.len() as i32 + 1;
        let (red, green, blue, most) = ((red / count) as f32, (green / count) as f32, (blue / count) as f32, (most / count) as f32);
        let gain = most / red.max(green).max(blue);
        Self::from_rgb((red * gain) as u8, (green * gain) as u8, (blue * gain) as u8)
    }
    /// bukkit: org/bukkit/Color.mixDyes
    pub fn mix_dyes(self, dyes: &[&str]) -> Self {
        let colors: Vec<Self> = dyes.iter().filter_map(|dye| crate::misc::dye_color(dye)).collect();
        self.mix_colors(&colors)
    }
}

/// `org.bukkit.Vibration`.
#[derive(Clone, Debug, PartialEq)]
pub struct Vibration {
    origin: Option<Location>,
    destination: VibrationDestination,
    arrival_time: i32,
}

/// `org.bukkit.Vibration$Destination`.
#[derive(Clone, Debug, PartialEq)]
pub enum VibrationDestination {
    Entity(Entity),
    Block(Location),
}

impl VibrationDestination {
    /// bukkit: org/bukkit/Vibration$Destination$EntityDestination.getEntity
    pub fn get_entity(&self) -> Option<&Entity> {
        match self {
            Self::Entity(entity) => Some(entity),
            Self::Block(_) => None,
        }
    }
    /// bukkit: org/bukkit/Vibration$Destination$BlockDestination.getLocation
    pub fn get_location(&self) -> Option<&Location> {
        match self {
            Self::Entity(_) => None,
            Self::Block(at) => Some(at),
        }
    }
    /// bukkit: org/bukkit/Vibration$Destination$BlockDestination.getBlock
    pub fn get_block(&self) -> Option<Block> {
        self.get_location().map(Location::get_block)
    }
}

impl Vibration {
    /// bukkit: org/bukkit/Vibration.<init>
    pub fn new(origin: Option<Location>, destination: VibrationDestination, arrival_time: i32) -> Self {
        Self { origin, destination, arrival_time }
    }
    /// bukkit: org/bukkit/Vibration.getOrigin
    pub fn get_origin(&self) -> Option<&Location> {
        self.origin.as_ref()
    }
    /// bukkit: org/bukkit/Vibration.getDestination
    pub fn get_destination(&self) -> &VibrationDestination {
        &self.destination
    }
    /// bukkit: org/bukkit/Vibration.getArrivalTime
    pub fn get_arrival_time(&self) -> i32 {
        self.arrival_time
    }
}

/// bukkit: org/bukkit/plugin/AuthorNagException.<init>, org/bukkit/plugin/AuthorNagException.getMessage
///
/// `org.bukkit.plugin.AuthorNagException`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorNagException {
    message: String,
}

impl AuthorNagException {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_owned() }
    }
    /// bukkit: org/bukkit/plugin/AuthorNagException.getMessage
    pub fn get_message(&self) -> &str {
        &self.message
    }
}

/// bukkit: org/bukkit/util/Consumer, org/bukkit/util/Consumer.accept
pub type Consumer<T> = Box<dyn FnOnce(T) + Send>;

/// bukkit: org/bukkit/util/NumberConversions.checkFinite
///
/// # Panics
/// As Java throws, for a number that isn't finite.
pub fn check_finite(value: f64, message: &str) {
    assert!(value.is_finite(), "{message}");
}

/// bukkit: org/bukkit/util/BoundingBox.deserialize
pub fn bounding_box_deserialize(map: &Value) -> BoundingBox {
    let side = |key: &str| map[key].as_f64().unwrap_or(0.0);
    BoundingBox::new(side("minX"), side("minY"), side("minZ"), side("maxX"), side("maxY"), side("maxZ"))
}

// ---- Advancements ----

impl crate::core2::Advancement {
    /// bukkit: org/bukkit/advancement/Advancement.getChildren
    pub fn get_children(&self) -> Vec<Self> {
        advancement_iterator().into_iter().filter(|other| other.get_parent().is_some_and(|parent| parent.id == self.id)).collect()
    }
    /// bukkit: org/bukkit/advancement/Advancement.displayName
    pub fn display_name(&self) -> Component {
        let title = self.get_display().map(|display| display.get_title().clone()).unwrap_or_default();
        Component(json!({ "text": "[", "extra": [title, { "text": "]" }] }))
    }
}

impl crate::core2::AdvancementDisplay {
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.getX
    pub fn get_x(&self) -> f32 {
        query(json!({ "op": "core_advancement_place", "id": self.0["__key"] }))[0].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.getY
    pub fn get_y(&self) -> f32 {
        query(json!({ "op": "core_advancement_place", "id": self.0["__key"] }))[1].as_f64().unwrap_or(0.0) as f32
    }
}

/// bukkit: org/bukkit/advancement/AdvancementDisplayType.getColor
pub fn advancement_display_type_color(kind: &str) -> &'static str {
    if kind.eq_ignore_ascii_case("challenge") { crate::chat_color::DARK_PURPLE } else { crate::chat_color::GREEN }
}

/// bukkit: org/bukkit/advancement/AdvancementRequirement.isStrict
///
/// Whether a row of `get_requirements` is the only way: one criterion in it.
pub fn advancement_requirement_is_strict(row: &[String]) -> bool {
    row.len() == 1
}

// ---- Registry tags ----

impl crate::registry::Registry {
    fn tag_kind(&self) -> &'static str {
        self.folder
    }
    /// bukkit: org/bukkit/Registry.hasTag
    pub fn has_tag(&self, tag: &str) -> bool {
        let key = key_of(tag.trim_start_matches('#'));
        query(json!({ "op": "tags", "kind": self.tag_kind() })).as_array().is_some_and(|names| names.iter().any(|name| name.as_str() == Some(&key)))
    }
    /// bukkit: org/bukkit/Registry.getTag
    pub fn get_tag(&self, tag: &str) -> Option<crate::registry::Tag> {
        self.has_tag(tag).then(|| crate::registry::Tag::get(self.tag_kind(), tag))
    }
    /// bukkit: org/bukkit/Registry.getTags
    pub fn get_tags(&self) -> Vec<crate::registry::Tag> {
        crate::registry::Tag::all(self.tag_kind())
    }
    /// bukkit: org/bukkit/Registry.getTagValues
    pub fn get_tag_values(&self, tag: &str) -> Vec<String> {
        crate::registry::Tag::get(self.tag_kind(), tag).get_values()
    }
}

// ---- Instruments and songs ----

/// `org.bukkit.MusicInstrument`: an entry of the game's `instrument` data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MusicInstrument {
    key: String,
    /// What `create` made, which has no key.
    inline: Option<Value>,
}

impl MusicInstrument {
    fn data(&self) -> Value {
        self.inline.clone().unwrap_or_else(|| query(json!({ "op": "data_file", "path": format!("instrument/{}", self.key.trim_start_matches("minecraft:")) })))
    }
    /// bukkit: org/bukkit/MusicInstrument.getByKey
    pub fn get_by_key(key: &str) -> Option<Self> {
        let instrument = Self { key: key_of(key), inline: None };
        (!instrument.data().is_null()).then_some(instrument)
    }
    /// bukkit: org/bukkit/MusicInstrument.getKey, org/bukkit/MusicInstrument.key
    pub fn get_key(&self) -> &str {
        &self.key
    }
    /// bukkit: org/bukkit/MusicInstrument.getDuration
    pub fn get_duration(&self) -> f32 {
        self.data()["use_duration"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/MusicInstrument.getRange
    pub fn get_range(&self) -> f32 {
        self.data()["range"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/MusicInstrument.getSound
    pub fn get_sound(&self) -> Option<String> {
        self.data()["sound_event"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/MusicInstrument.description
    pub fn description(&self) -> Component {
        Component(self.data()["description"].take())
    }
    /// bukkit: org/bukkit/MusicInstrument.translationKey
    pub fn translation_key(&self) -> String {
        self.data()["description"]["translate"].as_str().unwrap_or_default().to_owned()
    }
}

/// `org.bukkit.JukeboxSong`: an entry of the game's `jukebox_song` data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JukeboxSong {
    key: String,
}

impl JukeboxSong {
    fn data(&self) -> Value {
        query(json!({ "op": "data_file", "path": format!("jukebox_song/{}", self.key.trim_start_matches("minecraft:")) }))
    }
    pub fn of(key: &str) -> Self {
        Self { key: key_of(key) }
    }
    /// bukkit: org/bukkit/JukeboxSong.getSound
    pub fn get_sound(&self) -> Option<String> {
        self.data()["sound_event"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/JukeboxSong.getLengthInSeconds
    pub fn get_length_in_seconds(&self) -> f32 {
        self.data()["length_in_seconds"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/JukeboxSong.getComparatorOutput
    pub fn get_comparator_output(&self) -> i32 {
        int(&self.data()["comparator_output"])
    }
    /// bukkit: org/bukkit/JukeboxSong.getDescription
    pub fn get_description(&self) -> Component {
        Component(self.data()["description"].take())
    }
    /// bukkit: org/bukkit/JukeboxSong.getTranslationKey
    pub fn get_translation_key(&self) -> String {
        self.data()["description"]["translate"].as_str().unwrap_or_default().to_owned()
    }
}

// ---- Damage types ----

/// `org.bukkit.damage.DamageType`: an entry of the game's `damage_type` data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageType {
    key: String,
}

impl DamageType {
    fn data(&self) -> Value {
        query(json!({ "op": "data_file", "path": format!("damage_type/{}", self.key.trim_start_matches("minecraft:")) }))
    }
    pub fn of(key: &str) -> Self {
        Self { key: key_of(key) }
    }
    /// bukkit: org/bukkit/damage/DamageType.getTranslationKey
    pub fn get_translation_key(&self) -> String {
        self.data()["message_id"].as_str().unwrap_or_default().to_owned()
    }
    /// bukkit: org/bukkit/damage/DamageType.getDamageScaling
    pub fn get_damage_scaling(&self) -> String {
        self.data()["scaling"].as_str().unwrap_or("never").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/damage/DamageType.getDamageEffect
    pub fn get_damage_effect(&self) -> DamageEffect {
        DamageEffect(self.data()["effects"].as_str().unwrap_or("hurt").to_owned())
    }
    /// bukkit: org/bukkit/damage/DamageType.getDeathMessageType
    pub fn get_death_message_type(&self) -> String {
        self.data()["death_message_type"].as_str().unwrap_or("default").to_ascii_uppercase()
    }
}

/// `org.bukkit.damage.DamageEffect`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageEffect(pub String);

impl DamageEffect {
    /// bukkit: org/bukkit/damage/DamageEffect.getSound
    pub fn get_sound(&self) -> &'static str {
        match self.0.as_str() {
            "thorns" => "minecraft:enchant.thorns.hit",
            "drowning" => "minecraft:entity.player.hurt_drown",
            "burning" => "minecraft:entity.player.hurt_on_fire",
            "poking" => "minecraft:entity.player.hurt_sweet_berry_bush",
            "freezing" => "minecraft:entity.player.hurt_freeze",
            _ => "minecraft:entity.player.hurt",
        }
    }
}

impl crate::core2::DamageSource {
    /// bukkit: org/bukkit/damage/DamageSource.getSourceLocation
    pub fn get_source_location(&self) -> Option<Location> {
        self.get_damage_location().cloned().or_else(|| self.get_direct_entity().and_then(Entity::get_location))
    }
}

// ---- Bans ----

impl crate::extra::BanList {
    /// bukkit: org/bukkit/BanList.getBanEntry, org/bukkit/ban/ProfileBanList.getBanEntry
    pub fn get_ban_entry(&self, target: &str) -> Option<crate::core2::BanEntry> {
        crate::core2::get_ban_entries(self.addresses).into_iter().find(|entry| entry.get_target().eq_ignore_ascii_case(target))
    }
}

/// bukkit: org/bukkit/ban/ProfileBanList.isBanned
pub fn profile_is_banned(name: &str) -> bool {
    crate::extra::get_ban_list("PROFILE").is_banned(name)
}

/// bukkit: org/bukkit/ban/ProfileBanList.pardon
pub fn profile_pardon(name: &str) {
    crate::extra::get_ban_list("PROFILE").pardon(name);
}

/// A ban by name, for good, by the server: `OfflinePlayer::ban_until` has `ProfileBanList.addBan`'s date and source.
pub fn profile_add_ban(name: &str, reason: &str) {
    crate::get_offline_player(name).ban_until(Some(reason), None, None);
}

impl OfflinePlayer {
    /// bukkit: org/bukkit/OfflinePlayer.ban, org/bukkit/ban/ProfileBanList.addBan
    ///
    /// `expires` as the file keeps dates, `yyyy-MM-dd HH:mm:ss Z`, or none for good. Whoever it
    /// is keeps playing until they're kicked, as in Java.
    pub fn ban_until(&self, reason: Option<&str>, expires: Option<&str>, source: Option<&str>) -> bool {
        let Some(uuid) = self.get_unique_id() else {
            return false;
        };
        let row = json!({ "uuid": crate::dashed(uuid), "name": self.get_name(), "created": ban_date(), "source": source.unwrap_or("Server"), "expires": expires.unwrap_or("forever"), "reason": reason.unwrap_or("Banned by an operator.") });
        query(json!({ "op": "core_ban_save", "addresses": false, "row": row })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/OfflinePlayer.banPlayer
    ///
    /// `ban_until` with a kick, if they're on, as `kick_if_online` says.
    pub fn ban_player(&self, reason: Option<&str>, expires: Option<&str>, source: Option<&str>, kick_if_online: bool) -> bool {
        let banned = self.ban_until(reason, expires, source);
        if kick_if_online && let Some(player) = self.get_player() {
            player.kick_player(reason.unwrap_or("You are banned from this server."));
        }
        banned
    }
    /// For good, by the console, as `/ban` is.
    pub fn ban(&self, reason: &str) {
        run_command(&format!("ban {} {reason}", self.get_name()));
    }
    fn file(&self) -> Value {
        // Who is on is saved only now and then: the live answer first, where a caller has one.
        query(json!({ "op": "core_player_file", "uuid": crate::dashed(self.get_unique_id().unwrap_or_default()) }))
    }
    fn located(&self, dimension: &Value, pos: [Option<f64>; 3], yaw: f32, pitch: f32) -> Option<Location> {
        Some(Location::new(World::of(dimension.as_str()?), pos[0]?, pos[1]?, pos[2]?, yaw, pitch))
    }
    /// bukkit: org/bukkit/OfflinePlayer.getLocation
    ///
    /// Where the player was last saved.
    pub fn get_location(&self) -> Option<Location> {
        if let Some(player) = self.get_player() {
            return Some(player.get_location());
        }
        let data = self.file();
        let side = |i: usize| data["Pos"][i].as_f64();
        self.located(&data["Dimension"], [side(0), side(1), side(2)], data["Rotation"][0].as_f64().unwrap_or(0.0) as f32, data["Rotation"][1].as_f64().unwrap_or(0.0) as f32)
    }
    /// bukkit: org/bukkit/OfflinePlayer.getRespawnLocation, org/bukkit/OfflinePlayer.getBedSpawnLocation
    pub fn get_respawn_location(&self) -> Option<Location> {
        let data = self.file();
        let respawn = &data["respawn"];
        let side = |i: usize| respawn["pos"][i].as_f64();
        self.located(&respawn["dimension"], [side(0), side(1), side(2)], respawn["yaw"].as_f64().unwrap_or(0.0) as f32, respawn["pitch"].as_f64().unwrap_or(0.0) as f32)
    }
    /// bukkit: org/bukkit/OfflinePlayer.getLastDeathLocation
    pub fn get_last_death_location(&self) -> Option<Location> {
        let data = self.file();
        let death = &data["LastDeathLocation"];
        let side = |i: usize| death["pos"][i].as_f64();
        self.located(&death["dimension"], [side(0), side(1), side(2)], 0.0, 0.0)
    }
    fn stats_file(&self) -> PathBuf {
        world_folder().join(format!("players/stats/{}.json", crate::dashed(self.get_unique_id().unwrap_or_default())))
    }
    /// bukkit: org/bukkit/OfflinePlayer.setStatistic
    ///
    /// `stat_type` and `name` as the stats file has them.
    pub fn set_statistic(&self, stat_type: &str, name: &str, value: i64) -> bool {
        if self.is_online() {
            return query(json!({ "op": "core_stat_set", "uuid": self.get_unique_id().unwrap_or_default().replace('-', ""), "kind": stat_type, "key": name, "value": value })).as_bool().unwrap_or(false);
        }
        let file = self.stats_file();
        let mut stats = file_json(&file);
        if !stats.is_object() {
            stats = json!({ "stats": {}, "DataVersion": crate::core2::get_unsafe().get_data_version() });
        }
        stats["stats"][stat_type][name] = json!(value);
        std::fs::write(file, stats.to_string()).is_ok()
    }
    /// bukkit: org/bukkit/OfflinePlayer.incrementStatistic
    pub fn increment_statistic(&self, stat_type: &str, name: &str, amount: i64) -> bool {
        self.set_statistic(stat_type, name, self.get_statistic(stat_type, name) + amount)
    }
    /// bukkit: org/bukkit/OfflinePlayer.decrementStatistic
    pub fn decrement_statistic(&self, stat_type: &str, name: &str, amount: i64) -> bool {
        self.set_statistic(stat_type, name, self.get_statistic(stat_type, name) - amount)
    }
    /// What the game holds of an online player's statistic, which its file lags behind.
    pub(crate) fn online_statistic(&self, stat_type: &str, name: &str) -> Option<i64> {
        if !self.is_online() {
            return None;
        }
        query(json!({ "op": "core_stat_get", "uuid": self.get_unique_id()?.replace('-', ""), "kind": stat_type, "key": name })).as_i64()
    }
    /// bukkit: org/bukkit/OfflinePlayer.getPersistentDataContainer
    ///
    /// `BukkitValues` of their file. None for someone with no id.
    pub fn get_persistent_data_container(&self) -> Option<crate::persistence::PersistentDataContainer> {
        Some(crate::persistence::PersistentDataContainer::of_player(self.get_unique_id()?.to_owned()))
    }
}

// ---- Help ----

impl crate::core2::HelpTopic {
    /// bukkit: org/bukkit/help/HelpTopic.canSee, org/bukkit/help/GenericCommandHelpTopic.canSee, org/bukkit/help/IndexHelpTopic.canSee
    ///
    /// The console always; else by `amend_can_see`, or by the command the topic is of.
    pub fn can_see(&self, sender: &CommandSender) -> bool {
        if *sender == CommandSender::Console {
            return true;
        }
        if let Some(node) = HELP_PERMISSIONS.lock().unwrap_or_else(|e| e.into_inner()).get(self.get_name()) {
            return sender.has_permission(node);
        }
        command_of(self.get_name().trim_start_matches('/')).is_none_or(|command| command.test_permission_silent(sender))
    }
    /// bukkit: org/bukkit/help/HelpTopic.amendCanSee, org/bukkit/help/IndexHelpTopic.amendCanSee
    pub fn amend_can_see(&self, permission: Option<&str>) {
        let mut all = HELP_PERMISSIONS.lock().unwrap_or_else(|e| e.into_inner());
        match permission {
            Some(node) => {
                all.insert(self.get_name().to_owned(), node.to_owned());
            }
            None => {
                all.remove(self.get_name());
            }
        }
    }
}

static HELP_PERMISSIONS: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

/// bukkit: org/bukkit/help/HelpTopicComparator.compare, org/bukkit/help/HelpTopicComparator.helpTopicComparatorInstance
///
/// Slash-commands sort after plain topics, each by name, ignoring case.
pub fn help_topic_compare(first: &crate::core2::HelpTopic, second: &crate::core2::HelpTopic) -> std::cmp::Ordering {
    topic_name_compare(first.get_name(), second.get_name())
}

/// bukkit: org/bukkit/help/HelpTopicComparator$TopicNameComparator.compare, org/bukkit/help/HelpTopicComparator.topicNameComparatorInstance
pub fn topic_name_compare(first: &str, second: &str) -> std::cmp::Ordering {
    (first.starts_with('/'), first.to_ascii_lowercase()).cmp(&(second.starts_with('/'), second.to_ascii_lowercase()))
}

/// bukkit: org/bukkit/help/HelpMap.getIgnoredPlugins
pub fn help_ignored_plugins() -> Vec<String> {
    crate::config::YamlConfiguration::load_configuration(&server_folder().join("help.yml")).get_string_list("ignore-plugins")
}

// ---- Permissions ----

/// `org.bukkit.permissions.PermissionAttachmentInfo`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionAttachmentInfo {
    player: Player,
    permission: String,
    value: bool,
}

impl PermissionAttachmentInfo {
    /// bukkit: org/bukkit/permissions/PermissionAttachmentInfo.getPermission
    pub fn get_permission(&self) -> &str {
        &self.permission
    }
    /// bukkit: org/bukkit/permissions/PermissionAttachmentInfo.getValue
    pub fn get_value(&self) -> bool {
        self.value
    }
    /// bukkit: org/bukkit/permissions/PermissionAttachmentInfo.getPermissible
    pub fn get_permissible(&self) -> &Player {
        &self.player
    }
}

impl Player {
    /// bukkit: org/bukkit/permissions/Permissible.getEffectivePermissions
    ///
    /// What plugins granted, and what `plugin.yml` declares by default.
    pub fn get_effective_permissions(&self) -> Vec<PermissionAttachmentInfo> {
        let granted = crate::PERMISSION_GRANTS.lock().unwrap_or_else(|e| e.into_inner());
        let mut rows: BTreeMap<String, bool> = granted.iter().filter(|((uuid, _), _)| uuid == self.uuid()).map(|((_, node), value)| (node.clone(), *value)).collect();
        drop(granted);
        let defaults: Vec<String> = crate::PERMISSION_DEFAULTS.lock().unwrap_or_else(|e| e.into_inner()).keys().cloned().collect();
        for node in defaults {
            let value = self.has_permission(&node);
            rows.entry(node).or_insert(value);
        }
        rows.into_iter().map(|(permission, value)| PermissionAttachmentInfo { player: self.clone(), permission, value }).collect()
    }
    /// bukkit: org/bukkit/permissions/Permissible.removeAttachment
    pub fn remove_attachment(&self, attachment: &crate::config2::PermissionAttachment) -> bool {
        attachment.remove()
    }
    /// bukkit: org/bukkit/permissions/Permissible.permissionValue
    ///
    /// `TRUE` or `FALSE` where it's set for them, else `NOT_SET`.
    pub fn permission_value(&self, node: &str) -> &'static str {
        match crate::permission_is_granted(self.uuid(), node) {
            Some(true) => "TRUE",
            Some(false) => "FALSE",
            None => "NOT_SET",
        }
    }
}

/// bukkit: org/bukkit/permissions/PermissionDefault.getByName
pub fn permission_default_by_name(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().replace('_', "").replace(' ', "").as_str() {
        "t" | "true" => Some("TRUE"),
        "f" | "false" => Some("FALSE"),
        "op" | "isop" | "operator" | "isoperator" | "admin" | "isadmin" => Some("OP"),
        "!op" | "notop" | "!operator" | "notoperator" | "!admin" | "notadmin" => Some("NOT_OP"),
        _ => None,
    }
}

impl crate::config2::Permission {
    /// bukkit: org/bukkit/permissions/Permission.addParent
    ///
    /// Makes this node a child of `parent`, which is declared here if it wasn't.
    pub fn add_parent(&self, parent: &str, value: bool) -> Self {
        let mut made = Self::new(parent, "op", "");
        made.children.insert(self.name.clone(), value);
        made
    }
    /// bukkit: org/bukkit/permissions/Permission.loadPermission
    ///
    /// One node of `plugin.yml`'s `permissions`.
    pub fn load_permission(name: &str, data: &Value) -> Self {
        let mut made = Self::new(name, data["default"].as_str().unwrap_or("op"), data["description"].as_str().unwrap_or_default());
        for (child, value) in data["children"].as_object().into_iter().flatten() {
            made.children.insert(child.clone(), value.as_bool().unwrap_or(true));
        }
        made
    }
    /// bukkit: org/bukkit/permissions/Permission.loadPermissions
    pub fn load_permissions(data: &Value) -> Vec<Self> {
        data.as_object().into_iter().flatten().map(|(name, row)| Self::load_permission(name, row)).collect()
    }
    /// bukkit: org/bukkit/permissions/Permission.recalculatePermissibles
    ///
    /// Nothing to do: a permission is worked out as it's asked for.
    pub fn recalculate_permissibles(&self) {}
}

impl crate::config2::PermissionAttachment {
    /// bukkit: org/bukkit/permissions/PermissionAttachment.getPlugin
    pub fn get_plugin_name(&self) -> &str {
        &self.plugin
    }
}

// ---- Plugins and messages ----

/// `org.bukkit.plugin.messaging.PluginMessageListenerRegistration`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginMessageListenerRegistration {
    channel: String,
    plugin: String,
}

impl PluginMessageListenerRegistration {
    /// bukkit: org/bukkit/plugin/messaging/PluginMessageListenerRegistration.getChannel
    pub fn get_channel(&self) -> &str {
        &self.channel
    }
    /// bukkit: org/bukkit/plugin/messaging/PluginMessageListenerRegistration.getPlugin
    pub fn get_plugin(&self) -> &str {
        &self.plugin
    }
    /// bukkit: org/bukkit/plugin/messaging/PluginMessageListenerRegistration.isValid
    pub fn is_valid(&self) -> bool {
        crate::persistence::get_incoming_channels().contains(&self.channel)
    }
}

/// bukkit: org/bukkit/plugin/messaging/Messenger.getIncomingChannelRegistrations
pub fn get_incoming_channel_registrations(plugin: &Plugin) -> Vec<PluginMessageListenerRegistration> {
    crate::persistence::get_incoming_channels().into_iter().map(|channel| PluginMessageListenerRegistration { channel, plugin: plugin.get_name().to_owned() }).collect()
}

/// bukkit: org/bukkit/plugin/messaging/Messenger.isRegistrationValid
pub fn is_registration_valid(registration: &PluginMessageListenerRegistration) -> bool {
    registration.is_valid()
}

impl Plugin {
    /// bukkit: org/bukkit/plugin/PluginBase.namespace
    pub fn namespace(&self) -> String {
        self.get_name().to_ascii_lowercase()
    }
}

/// bukkit: org/bukkit/plugin/messaging/Messenger.dispatchIncomingMessage
///
/// As if it came from the client: every plugin's listener of the channel hears it.
pub fn dispatch_incoming_message(player: &Player, channel: &str, data: &[u8]) {
    query(json!({ "op": "core_plugin_message_in", "uuid": player.uuid(), "name": player.get_name(), "channel": channel, "data": data }));
}

/// bukkit: org/bukkit/plugin/PluginBase.getName
pub fn plugin_base_get_name(plugin: &Plugin) -> &str {
    plugin.get_name()
}

// ---- Persistent data types ----

/// `org.bukkit.persistence.PersistentDataAdapterContext`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PersistentDataAdapterContext;

impl PersistentDataAdapterContext {
    /// bukkit: org/bukkit/persistence/PersistentDataAdapterContext.newPersistentDataContainer
    ///
    /// A container here is a JSON object.
    pub fn new_persistent_data_container(&self) -> serde_json::Map<String, Value> {
        serde_json::Map::new()
    }
}

/// `org.bukkit.persistence.PersistentDataType`: how a value of a plugin's own is kept.
pub trait PersistentDataType {
    type Complex;
    /// bukkit: org/bukkit/persistence/PersistentDataType.getPrimitiveType
    fn get_primitive_type(&self) -> &'static str;
    /// bukkit: org/bukkit/persistence/PersistentDataType.getComplexType
    fn get_complex_type(&self) -> &'static str;
    /// bukkit: org/bukkit/persistence/PersistentDataType.toPrimitive
    fn to_primitive(&self, complex: &Self::Complex, context: &PersistentDataAdapterContext) -> Value;
    /// bukkit: org/bukkit/persistence/PersistentDataType.fromPrimitive
    fn from_primitive(&self, primitive: &Value, context: &PersistentDataAdapterContext) -> Option<Self::Complex>;
    /// What NBT a primitive of this type is kept as: `persistence::typed_snbt`'s letters.
    fn nbt_kind(&self, primitive: &Value) -> Value {
        let letter = match self.get_primitive_type().to_ascii_uppercase().as_str() {
            "BYTE" | "BOOLEAN" => "b",
            "SHORT" => "s",
            "INT" | "INTEGER" => "i",
            "LONG" => "l",
            "FLOAT" => "f",
            "DOUBLE" => "d",
            "STRING" => "t",
            "BYTE[]" | "BYTE_ARRAY" => "B",
            "INT[]" | "INTEGER_ARRAY" => "I",
            "LONG[]" | "LONG_ARRAY" => "L",
            _ => return crate::persistence::natural_kind(primitive),
        };
        json!(letter)
    }
}

/// bukkit: org/bukkit/persistence/PersistentDataType$BooleanPersistentDataType.getPrimitiveType, org/bukkit/persistence/PersistentDataType$BooleanPersistentDataType.getComplexType, org/bukkit/persistence/PersistentDataType$BooleanPersistentDataType.toPrimitive, org/bukkit/persistence/PersistentDataType$BooleanPersistentDataType.fromPrimitive
///
/// `PersistentDataType.BOOLEAN`: a byte, one for true.
#[derive(Clone, Copy, Debug, Default)]
pub struct BooleanPersistentDataType;

impl PersistentDataType for BooleanPersistentDataType {
    type Complex = bool;
    fn get_primitive_type(&self) -> &'static str {
        "Byte"
    }
    fn get_complex_type(&self) -> &'static str {
        "Boolean"
    }
    fn to_primitive(&self, complex: &bool, _context: &PersistentDataAdapterContext) -> Value {
        json!(i8::from(*complex))
    }
    fn from_primitive(&self, primitive: &Value, _context: &PersistentDataAdapterContext) -> Option<bool> {
        primitive.as_i64().map(|value| value != 0)
    }
}

/// bukkit: org/bukkit/persistence/PersistentDataType$PrimitivePersistentDataType.getPrimitiveType, org/bukkit/persistence/PersistentDataType$PrimitivePersistentDataType.getComplexType, org/bukkit/persistence/PersistentDataType$PrimitivePersistentDataType.toPrimitive, org/bukkit/persistence/PersistentDataType$PrimitivePersistentDataType.fromPrimitive
///
/// The primitive types: the value is kept as it is.
#[derive(Clone, Copy, Debug)]
pub struct PrimitivePersistentDataType(pub &'static str);

impl PersistentDataType for PrimitivePersistentDataType {
    type Complex = Value;
    fn get_primitive_type(&self) -> &'static str {
        self.0
    }
    fn get_complex_type(&self) -> &'static str {
        self.0
    }
    fn to_primitive(&self, complex: &Value, _context: &PersistentDataAdapterContext) -> Value {
        complex.clone()
    }
    fn from_primitive(&self, primitive: &Value, _context: &PersistentDataAdapterContext) -> Option<Value> {
        Some(primitive.clone())
    }
}

// ---- Configuration options ----

/// `org.bukkit.configuration.file.YamlConfigurationOptions` and those it extends. What a save does
/// with them is `YamlConfiguration::save_with_options`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigurationOptions {
    separator: char,
    copy_defaults: bool,
    header: Vec<String>,
    footer: Vec<String>,
    parse_comments: bool,
    indent: i32,
    width: i32,
    code_point_limit: i32,
}

impl Default for ConfigurationOptions {
    fn default() -> Self {
        Self { separator: '.', copy_defaults: false, header: Vec::new(), footer: Vec::new(), parse_comments: true, indent: 2, width: 80, code_point_limit: 3 * 1024 * 1024 }
    }
}

impl ConfigurationOptions {
    /// bukkit: org/bukkit/configuration/ConfigurationOptions.pathSeparator
    pub fn path_separator(&self) -> char {
        self.separator
    }
    /// What `set_path_separator` on a configuration takes.
    pub fn set_path_separator(&mut self, separator: char) -> &mut Self {
        self.separator = separator;
        self
    }
    /// bukkit: org/bukkit/configuration/ConfigurationOptions.copyDefaults
    pub fn copy_defaults(&self) -> bool {
        self.copy_defaults
    }
    /// bukkit: org/bukkit/configuration/file/FileConfigurationOptions.getHeader
    pub fn get_header(&self) -> &[String] {
        &self.header
    }
    /// bukkit: org/bukkit/configuration/file/FileConfigurationOptions.header
    pub fn header(&self) -> String {
        self.header.join("\n")
    }
    /// bukkit: org/bukkit/configuration/file/FileConfigurationOptions.setHeader, org/bukkit/configuration/file/YamlConfigurationOptions.setHeader, org/bukkit/configuration/file/YamlConfigurationOptions.header
    pub fn set_header(&mut self, header: &[String]) -> &mut Self {
        header.clone_into(&mut self.header);
        self
    }
    /// bukkit: org/bukkit/configuration/file/FileConfigurationOptions.getFooter
    pub fn get_footer(&self) -> &[String] {
        &self.footer
    }
    /// bukkit: org/bukkit/configuration/file/FileConfigurationOptions.setFooter, org/bukkit/configuration/file/YamlConfigurationOptions.setFooter
    pub fn set_footer(&mut self, footer: &[String]) -> &mut Self {
        footer.clone_into(&mut self.footer);
        self
    }
    /// bukkit: org/bukkit/configuration/file/FileConfigurationOptions.parseComments
    pub fn parse_comments(&self) -> bool {
        self.parse_comments
    }
    /// bukkit: org/bukkit/configuration/file/YamlConfigurationOptions.parseComments
    pub fn set_parse_comments(&mut self, on: bool) -> &mut Self {
        self.parse_comments = on;
        self
    }
    /// bukkit: org/bukkit/configuration/file/FileConfigurationOptions.copyHeader
    pub fn copy_header(&self) -> bool {
        !self.header.is_empty()
    }
    /// bukkit: org/bukkit/configuration/file/YamlConfigurationOptions.copyHeader
    ///
    /// Off drops the header, as Java's does.
    pub fn set_copy_header(&mut self, on: bool) -> &mut Self {
        if !on {
            self.header.clear();
        }
        self
    }
    /// bukkit: org/bukkit/configuration/file/YamlConfigurationOptions.indent
    pub fn indent(&self) -> i32 {
        self.indent
    }
    /// bukkit: org/bukkit/configuration/file/YamlConfigurationOptions.width
    pub fn width(&self) -> i32 {
        self.width
    }
    /// bukkit: org/bukkit/configuration/file/YamlConfigurationOptions.codePointLimit
    pub fn code_point_limit(&self) -> i32 {
        self.code_point_limit
    }
}

impl crate::config::YamlConfiguration {
    /// bukkit: org/bukkit/configuration/file/YamlConfigurationOptions.copyDefaults, org/bukkit/configuration/MemoryConfigurationOptions.copyDefaults
    ///
    /// `options().copyDefaults(true)`: what the defaults say and the file doesn't is copied in.
    pub fn options_copy_defaults(&mut self, on: bool) {
        if on && let Some(defaults) = self.defaults.clone() {
            self.copy_defaults(&defaults);
        }
    }
    /// bukkit: org/bukkit/configuration/file/FileConfiguration.loadFromString, org/bukkit/configuration/file/YamlConfiguration.loadFromString
    pub fn load_from_string(&mut self, text: &str) {
        self.root = match crate::config::parse(text) {
            Value::Object(map) => Value::Object(map),
            _ => Value::Object(serde_json::Map::new()),
        };
    }
    /// bukkit: org/bukkit/configuration/file/FileConfiguration.load
    pub fn load(&mut self, path: &Path) -> std::io::Result<()> {
        let text = std::fs::read_to_string(path)?;
        self.load_from_string(&text);
        self.path = Some(path.to_owned());
        Ok(())
    }
    /// bukkit: org/bukkit/configuration/Configuration.addDefaults, org/bukkit/configuration/MemoryConfiguration.addDefaults
    ///
    /// Another configuration's values as defaults (`addDefaults(YamlConfiguration.loadConfiguration(reader))`).
    pub fn add_defaults_from(&mut self, other: &crate::config::YamlConfiguration) {
        fn walk(into: &mut crate::config::YamlConfiguration, prefix: &str, value: &Value, separator: char) {
            match value.as_object() {
                Some(map) if !map.is_empty() => {
                    for (key, inner) in map {
                        let path = if prefix.is_empty() { key.clone() } else { format!("{prefix}{separator}{key}") };
                        walk(into, &path, inner, separator);
                    }
                }
                _ if !prefix.is_empty() => into.add_default(prefix, value.clone()),
                _ => {}
            }
        }
        let separator = self.separator;
        walk(self, "", &other.root, separator);
    }
    /// bukkit: org/bukkit/configuration/Configuration.addDefaults, org/bukkit/configuration/MemoryConfiguration.addDefaults
    pub fn add_defaults(&mut self, values: &serde_json::Map<String, Value>) {
        for (path, value) in values {
            self.add_default(path, value.clone());
        }
    }
    /// bukkit: org/bukkit/configuration/Configuration.getDefaults, org/bukkit/configuration/MemoryConfiguration.getDefaults
    pub fn get_defaults(&self) -> Option<&Value> {
        self.defaults.as_ref()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getDefaultSection, org/bukkit/configuration/MemorySection.getDefaultSection
    pub fn get_default_section(&self) -> Option<Self> {
        self.defaults.clone().map(|root| {
            let mut section = Self::new();
            section.root = root;
            section
        })
    }
    /// bukkit: org/bukkit/configuration/file/FileConfiguration.save
    ///
    /// `save`, with the header and footer of `options` as comments around the file.
    pub fn save_with_options(&self, path: &Path, options: &ConfigurationOptions) -> std::io::Result<()> {
        let comment = |lines: &[String]| lines.iter().map(|line| if line.is_empty() { "#\n".to_owned() } else { format!("# {line}\n") }).collect::<String>();
        let mut text = comment(&options.header);
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&self.save_to_string());
        text.push_str(&comment(&options.footer));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)
    }
}

// ---- Commands ----

/// `org.bukkit.command.CommandMap`: this plugin's commands, the only ones it can run or change.
#[derive(Clone, Copy, Debug, Default)]
pub struct CommandMap;

/// bukkit: org/bukkit/Bukkit.getCommandMap, org/bukkit/Server.getCommandMap
pub fn get_command_map() -> CommandMap {
    CommandMap
}

/// What a plugin's `plugin.yml` says of a command, as a `Command`.
fn declared_command(word: &str, row: &Value) -> crate::pure2::Command {
    let mut command = crate::pure2::Command::new(word);
    if let Some(usage) = row["usage"].as_str() {
        command.set_usage(usage);
    }
    if let Some(node) = row["permission"].as_str() {
        command.set_permission(node);
    }
    if let Some(message) = row["permission-message"].as_str() {
        command.set_permission_message(message);
    }
    if let Some(text) = row["description"].as_str() {
        command.set_description(text);
    }
    let aliases: Vec<&str> = match &row["aliases"] {
        Value::Array(list) => list.iter().filter_map(Value::as_str).collect(),
        Value::String(one) => vec![one.as_str()],
        _ => Vec::new(),
    };
    command.set_aliases(&aliases);
    command.register();
    command
}

/// The plugin's name and its command for a word, of any plugin: `plugin:word` works too.
fn plugin_command(word: &str) -> Option<(String, crate::pure2::Command)> {
    let lower = word.to_ascii_lowercase();
    let (space, bare) = lower.rsplit_once(':').map_or((None, lower.as_str()), |(space, bare)| (Some(space), bare));
    for row in shared_plugins() {
        let plugin = row["name"].as_str().unwrap_or_default();
        if space.is_some_and(|space| space != plugin.to_ascii_lowercase()) {
            continue;
        }
        for (name, declared) in row["yaml"]["commands"].as_object().into_iter().flatten() {
            let aliases = &declared["aliases"];
            let answers = name.eq_ignore_ascii_case(bare) || aliases.as_str().is_some_and(|alias| alias.eq_ignore_ascii_case(bare)) || aliases.as_array().is_some_and(|list| list.iter().any(|alias| alias.as_str().is_some_and(|alias| alias.eq_ignore_ascii_case(bare))));
            if answers {
                return Some((plugin.to_owned(), declared_command(name, declared)));
            }
        }
    }
    None
}

/// A command this plugin made in code, as `plugin.yml` doesn't list those.
fn own_command(word: &str) -> Option<crate::pure2::Command> {
    let owner = crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).get(&word.to_ascii_lowercase()).cloned()?;
    let mut command = crate::pure2::Command::new(&owner);
    command.register();
    Some(command)
}

fn game_commands() -> Vec<String> {
    query(json!({ "op": "core_commands" })).as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect()
}

fn command_of(name: &str) -> Option<crate::pure2::Command> {
    plugin_command(name).map(|(_, command)| command).or_else(|| own_command(name)).or_else(|| {
        let bare = name.rsplit(':').next().unwrap_or(name).to_ascii_lowercase();
        game_commands().contains(&bare).then(|| {
            let mut command = crate::pure2::Command::new(&bare);
            command.register();
            command
        })
    })
}

impl CommandMap {
    /// bukkit: org/bukkit/command/CommandMap.getCommand
    ///
    /// Of any plugin, or the game's own, by name, alias or `plugin:name`.
    pub fn get_command(&self, name: &str) -> Option<crate::pure2::Command> {
        command_of(name)
    }
    /// bukkit: org/bukkit/command/CommandMap.getKnownCommands
    ///
    /// Every word, plugins' under `plugin:word` too, as Java's map has them.
    pub fn get_known_commands(&self) -> BTreeMap<String, crate::pure2::Command> {
        let mut known = BTreeMap::new();
        for word in game_commands() {
            if let Some(command) = command_of(&word) {
                known.insert(format!("minecraft:{word}"), command.clone());
                known.insert(word, command);
            }
        }
        for row in shared_plugins() {
            let plugin = row["name"].as_str().unwrap_or_default().to_ascii_lowercase();
            for (name, declared) in row["yaml"]["commands"].as_object().into_iter().flatten() {
                let command = declared_command(name, declared);
                for word in std::iter::once(name.clone()).chain(command.get_aliases().iter().cloned()) {
                    known.insert(format!("{plugin}:{word}"), command.clone());
                    known.insert(word, command.clone());
                }
            }
        }
        let own: Vec<String> = crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).keys().cloned().collect();
        for word in own {
            if let Some(command) = own_command(&word) {
                known.entry(word).or_insert(command);
            }
        }
        known
    }
    /// bukkit: org/bukkit/command/CommandMap.register
    ///
    /// At any time: players are sent the new tree on the next tick. `fallback:name` answers too.
    pub fn register(&self, plugin: &Plugin, fallback: &str, command: &crate::pure2::Command) -> bool {
        let busy = crate::commands().try_lock().is_err();
        if busy || crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).contains_key(command.get_name()) {
            return false;
        }
        crate::add_command(plugin.host, command.get_name());
        let prefixed = format!("{}:{}", fallback.to_ascii_lowercase(), command.get_name());
        for alias in command.get_aliases().iter().map(String::as_str).chain([prefixed.as_str()]) {
            command_word(plugin.host, alias);
            crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).insert(alias.to_ascii_lowercase(), command.get_name().to_owned());
        }
        if let Some(entry) = crate::commands().lock().unwrap_or_else(|e| e.into_inner()).get_mut(command.get_name()) {
            command.get_usage().clone_into(&mut entry.usage);
            entry.permission = command.get_permission().map(str::to_owned);
            entry.permission_message = command.get_permission_message().map(str::to_owned);
        }
        true
    }
    /// bukkit: org/bukkit/command/CommandMap.registerAll
    pub fn register_all(&self, plugin: &Plugin, fallback: &str, commands: &[crate::pure2::Command]) {
        for command in commands {
            self.register(plugin, fallback, command);
        }
    }
    /// bukkit: org/bukkit/command/CommandMap.clearCommands
    ///
    /// Every plugin's and the game's, as `SimpleCommandMap` leaves only its own defaults. Players
    /// hear with `update_commands`. What the SDK runs of the game's own keeps working.
    pub fn clear_commands(&self) {
        crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).clear();
        query(json!({ "op": "core_commands_clear" }));
        query(json!({ "op": "core_game_words_off" }));
    }
    /// bukkit~: org/bukkit/command/CommandMap.dispatch | at once, in whichever plugin owns it; one of the game's waits for the end of the packet in the hooks that hold the hub themselves
    pub fn dispatch(&self, sender: &CommandSender, line: &str) -> bool {
        let line = line.trim_start_matches('/');
        let mut words = line.split_whitespace();
        let Some(label) = words.next() else {
            return false;
        };
        let args: Vec<&str> = words.collect();
        if let Some(done) = run_here(sender, label, &args) {
            return done;
        }
        if let Some((plugin, _)) = plugin_command(label) {
            return call_other(&plugin, "run", sender, label, &args).as_bool().unwrap_or(false);
        }
        if command_of(label).is_none() {
            return false;
        }
        match sender {
            CommandSender::Player(player) => {
                player.perform_command(line);
            }
            CommandSender::Console | CommandSender::Proxied(_) | CommandSender::Remote(_) | CommandSender::Entity(_) => {
                run_now(line);
            }
        }
        true
    }
    /// bukkit: org/bukkit/command/CommandMap.tabComplete
    pub fn tab_complete(&self, sender: &CommandSender, line: &str) -> Vec<String> {
        let mut words: Vec<&str> = line.trim_start_matches('/').split(' ').collect();
        let label = words.remove(0);
        if let Some(list) = complete_here(sender, label, &words) {
            return list;
        }
        let Some((plugin, _)) = plugin_command(label) else {
            return query(json!({ "op": "core_game_complete", "line": line.trim_start_matches('/') })).as_array().into_iter().flatten().filter_map(|word| word.as_str().map(str::to_owned)).collect();
        };
        call_other(&plugin, "complete", sender, label, &words).as_array().into_iter().flatten().filter_map(|word| word.as_str().map(str::to_owned)).collect()
    }
}

impl crate::pure2::Command {
    /// bukkit: org/bukkit/command/Command.testPermissionSilent
    pub fn test_permission_silent(&self, sender: &CommandSender) -> bool {
        self.get_permission().is_none_or(|node| node.is_empty() || node.split(';').any(|one| sender.has_permission(one)))
    }
    /// bukkit: org/bukkit/command/Command.testPermission
    ///
    /// Tells the sender if they can't.
    pub fn test_permission(&self, sender: &CommandSender) -> bool {
        if self.test_permission_silent(sender) {
            return true;
        }
        match self.get_permission_message() {
            Some("") => {}
            Some(message) => sender.send_message(message),
            None => sender.send_message(&get_permission_message()),
        }
        false
    }
    /// bukkit: org/bukkit/command/Command.permissionMessage
    pub fn permission_message(&self) -> Option<Component> {
        self.get_permission_message().map(adventure::legacy)
    }
    /// bukkit: org/bukkit/command/Command.execute, org/bukkit/command/PluginCommand.execute
    pub fn execute(&self, sender: &CommandSender, label: &str, args: &[&str]) -> bool {
        CommandMap.dispatch(sender, &format!("{label} {}", args.join(" ")))
    }
    /// bukkit: org/bukkit/command/Command.tabComplete, org/bukkit/command/PluginCommand.tabComplete
    pub fn tab_complete(&self, sender: &CommandSender, label: &str, args: &[&str]) -> Vec<String> {
        CommandMap.tab_complete(sender, &format!("{label} {}", args.join(" ")))
    }
    /// bukkit: org/bukkit/command/Command.broadcastCommandMessage
    pub fn broadcast_command_message(sender: &CommandSender, message: &str) {
        let line = format!("[{}: {message}]", sender.get_name());
        crate::log(&line);
        for player in get_online_players().iter().filter(|player| player.has_permission("bukkit.broadcast.admin")) {
            player.send_message(&format!("{}{}{line}", crate::chat_color::GRAY, crate::chat_color::ITALIC));
        }
    }
    /// bukkit: org/bukkit/command/Command.canBeOverriden
    ///
    /// False: a command here keeps its word.
    pub fn can_be_overriden(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/command/Command.getTimingName
    pub fn get_timing_name(&self) -> &str {
        self.get_name()
    }
}

impl Plugin {
    /// Whether this plugin answers to the word.
    pub fn owns_command(&self, name: &str) -> bool {
        crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&name.to_ascii_lowercase())
    }
}

/// bukkit: org/bukkit/command/PluginCommand.getPlugin, org/bukkit/command/PluginIdentifiableCommand.getPlugin
///
/// The plugin a command belongs to, of any plugin.
pub fn command_plugin(name: &str) -> Option<LoadedPlugin> {
    let (plugin, _) = plugin_command(name)?;
    PluginManager.get_plugin(&plugin)
}

/// bukkit: org/bukkit/command/PluginCommandYamlParser.parse
///
/// The commands `plugin.yml` declares.
pub fn parse_plugin_commands() -> Vec<crate::pure2::Command> {
    let words: BTreeSet<String> = crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).values().cloned().collect();
    words.iter().filter_map(|word| command_of(word)).collect()
}

impl CommandSender {
    /// bukkit: org/bukkit/command/CommandSender.sendPlainMessage
    pub fn send_plain_message(&self, text: &str) {
        self.send_component(&Component::text(text));
    }
    /// bukkit: org/bukkit/command/CommandSender.name
    pub fn name(&self) -> Component {
        Component::text(self.get_name())
    }
    /// bukkit: org/bukkit/conversations/Conversable.sendRawMessage
    pub fn send_raw_message(&self, text: &str) {
        self.send_message(text);
    }
}

// ---- Chunks in full ----

/// `org.bukkit.ChunkSnapshot` with what `Chunk.getChunkSnapshot(true, true, true)` adds: biomes and
/// light, as they were when taken. The blocks are the plain snapshot's, through `Deref`.
#[derive(Clone, Debug, PartialEq)]
pub struct FullChunkSnapshot {
    plain: crate::core2::ChunkSnapshot,
    bottom: i32,
    biomes: Vec<String>,
    climate: Vec<f32>,
    sky: Vec<u8>,
    block: Vec<u8>,
}

impl std::ops::Deref for FullChunkSnapshot {
    type Target = crate::core2::ChunkSnapshot;
    fn deref(&self) -> &Self::Target {
        &self.plain
    }
}

impl Chunk {
    /// bukkit: org/bukkit/Chunk.getChunkSnapshot
    ///
    /// With biomes and light, for a loaded chunk.
    pub fn get_full_snapshot(&self) -> Option<FullChunkSnapshot> {
        let plain = self.get_chunk_snapshot()?;
        let extra = self.ask("core_chunk_light");
        let list = |key: &str| extra[key].as_array().cloned().unwrap_or_default();
        Some(FullChunkSnapshot {
            plain,
            bottom: int(&extra["bottom"]),
            biomes: list("biomes").iter().filter_map(|name| name.as_str().map(str::to_owned)).collect(),
            climate: list("temperature").iter().filter_map(|value| value.as_f64().map(|value| value as f32)).collect(),
            sky: list("sky").iter().filter_map(|value| value.as_u64().map(|value| value as u8)).collect(),
            block: list("block").iter().filter_map(|value| value.as_u64().map(|value| value as u8)).collect(),
        })
    }
}

impl FullChunkSnapshot {
    fn cell(&self, x: i32, y: i32, z: i32) -> usize {
        ((y - self.bottom) * 256 + (z & 15) * 16 + (x & 15)) as usize
    }
    fn quart(&self, x: i32, y: i32, z: i32) -> usize {
        (((y - self.bottom) >> 2) * 16 + ((z & 15) >> 2) * 4 + ((x & 15) >> 2)) as usize
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getBiome
    pub fn get_biome(&self, x: i32, y: i32, z: i32) -> Option<&str> {
        self.biomes.get(self.quart(x, y, z)).map(String::as_str)
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getRawBiomeTemperature
    pub fn get_raw_biome_temperature(&self, x: i32, y: i32, z: i32) -> f64 {
        self.climate.get(self.quart(x, y, z)).map_or(0.0, |value| f64::from(*value))
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getBlockSkyLight
    pub fn get_block_sky_light(&self, x: i32, y: i32, z: i32) -> i32 {
        self.sky.get(self.cell(x, y, z)).map_or(0, |value| i32::from(*value))
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getBlockEmittedLight
    pub fn get_block_emitted_light(&self, x: i32, y: i32, z: i32) -> i32 {
        self.block.get(self.cell(x, y, z)).map_or(0, |value| i32::from(*value))
    }
}

// ---- Raids ----

/// `org.bukkit.Raid`: as the level's raid data last said. The game saves it as it changes.
#[derive(Clone, Debug, PartialEq)]
pub struct Raid {
    world: World,
    id: i32,
    data: Value,
}

impl World {
    /// bukkit: org/bukkit/World.getRaids
    pub fn get_raids(&self) -> Vec<Raid> {
        let data = self.ask("core_raids", json!({}));
        data["raids"].as_array().into_iter().flatten().filter(|raid| raid["status"].as_str() == Some("ongoing") && raid["active"].as_i64().unwrap_or(1) != 0).map(|raid| Raid { world: self.clone(), id: int(&raid["id"]), data: raid.clone() }).collect()
    }
    /// bukkit: org/bukkit/World.getRaid
    pub fn get_raid(&self, id: i32) -> Option<Raid> {
        self.get_raids().into_iter().find(|raid| raid.id == id)
    }
    /// bukkit: org/bukkit/World.locateNearestRaid
    pub fn locate_nearest_raid(&self, at: &Location, radius: i32) -> Option<Raid> {
        let range = f64::from(radius);
        self.get_raids().into_iter().filter_map(|raid| Some((raid.get_location()?.distance_squared(at), raid))).filter(|(distance, _)| *distance <= range * range).min_by(|first, second| first.0.total_cmp(&second.0)).map(|(_, raid)| raid)
    }
}

impl Raid {
    /// bukkit: org/bukkit/Raid.getId
    pub fn get_id(&self) -> i32 {
        self.id
    }
    /// bukkit: org/bukkit/Raid.isStarted
    pub fn is_started(&self) -> bool {
        self.data["started"].as_i64().unwrap_or(0) != 0
    }
    /// bukkit: org/bukkit/Raid.getActiveTicks
    pub fn get_active_ticks(&self) -> i64 {
        self.data["ticks_active"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/Raid.getBadOmenLevel
    pub fn get_bad_omen_level(&self) -> i32 {
        int(&self.data["raid_omen_level"])
    }
    /// bukkit: org/bukkit/Raid.getLocation
    pub fn get_location(&self) -> Option<Location> {
        at(&self.data["center"], &self.world)
    }
    /// bukkit: org/bukkit/Raid.getStatus
    pub fn get_status(&self) -> String {
        self.data["status"].as_str().unwrap_or("ongoing").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/Raid.getSpawnedGroups
    pub fn get_spawned_groups(&self) -> i32 {
        int(&self.data["groups_spawned"])
    }
    /// bukkit: org/bukkit/Raid.getTotalGroups
    pub fn get_total_groups(&self) -> i32 {
        int(&self.data["group_count"])
    }
    /// bukkit: org/bukkit/Raid.getTotalWaves
    pub fn get_total_waves(&self) -> i32 {
        self.get_total_groups()
    }
    /// bukkit: org/bukkit/Raid.getTotalHealth
    pub fn get_total_health(&self) -> f32 {
        self.data["total_health"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/Raid.getHeroes
    pub fn get_heroes(&self) -> Vec<String> {
        self.data["heroes_of_the_village"].as_array().into_iter().flatten().filter_map(|words| {
            let word = |i: usize| words[i].as_i64().map(|value| value as u32);
            Some(crate::dashed(&format!("{:08x}{:08x}{:08x}{:08x}", word(0)?, word(1)?, word(2)?, word(3)?)))
        }).collect()
    }
    /// bukkit: org/bukkit/Raid.getRaiders
    pub fn get_raiders(&self) -> Vec<Entity> {
        self.world.ask("core_raiders", json!({ "raid": self.id })).as_array().into_iter().flatten().filter_map(|id| id.as_u64().map(|id| Entity::of(id, &self.world))).collect()
    }
}

// ---- The dragon fight ----

/// `org.bukkit.boss.DragonBattle`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DragonBattle;

impl World {
    /// bukkit: org/bukkit/World.getEnderDragonBattle
    pub fn get_ender_dragon_battle(&self) -> Option<DragonBattle> {
        self.dimension().ends_with("the_end").then_some(DragonBattle)
    }
}

impl DragonBattle {
    fn state(&self) -> Value {
        query(json!({ "op": "core_dragon" }))
    }
    fn world(&self) -> World {
        World::of("world_the_end")
    }
    /// bukkit: org/bukkit/boss/DragonBattle.getEnderDragon
    pub fn get_ender_dragon(&self) -> Option<Entity> {
        let id = self.state()["dragon"]["id"].as_u64().filter(|id| *id != 0)?;
        Some(Entity::of(id, &self.world()))
    }
    /// bukkit: org/bukkit/boss/DragonBattle.getEndPortalLocation
    pub fn get_end_portal_location(&self) -> Option<Location> {
        at(&self.state()["exit_portal"], &self.world())
    }
    /// bukkit: org/bukkit/boss/DragonBattle.hasBeenPreviouslyKilled
    pub fn has_been_previously_killed(&self) -> bool {
        self.state()["previously_killed"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/boss/DragonBattle.setPreviouslyKilled
    ///
    /// Saved with the fight's next change.
    pub fn set_previously_killed(&self, killed: bool) {
        query(json!({ "op": "core_dragon_killed", "killed": killed }));
    }
    /// bukkit: org/bukkit/boss/DragonBattle.getRespawnPhase
    pub fn get_respawn_phase(&self) -> &'static str {
        let state = self.state();
        match state["respawn"].as_str().and_then(|text| text.split(' ').next()) {
            Some("start") => "START",
            Some("preparing_to_summon_pillars") => "PREPARING_TO_SUMMON_PILLARS",
            Some("summoning_pillars") => "SUMMONING_PILLARS",
            Some("summoning_dragon") => "SUMMONING_DRAGON",
            Some("end") => "END",
            _ => "NONE",
        }
    }
    /// bukkit: org/bukkit/boss/DragonBattle.getGatewayCount
    pub fn get_gateway_count(&self) -> i32 {
        20 - self.state()["gateways_left"].as_i64().map_or(20, |left| left as i32)
    }
    /// bukkit: org/bukkit/boss/DragonBattle.getHealingCrystals
    pub fn get_healing_crystals(&self) -> Vec<Entity> {
        let state = self.state();
        state["crystal_list"].as_array().into_iter().flatten().filter_map(|crystal| crystal["id"].as_u64().filter(|id| *id != 0)).map(|id| Entity::of(id, &self.world())).collect()
    }
}

// ---- Tasks ----

/// `org.bukkit.scheduler.BukkitTask`: a task by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BukkitTask {
    id: u32,
}

impl BukkitTask {
    pub fn of(id: u32) -> Self {
        Self { id }
    }
    /// bukkit: org/bukkit/scheduler/BukkitTask.isCancelled, org/bukkit/scheduler/BukkitRunnable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        CANCELLED_TASKS.lock().unwrap_or_else(|e| e.into_inner()).contains(&self.id)
    }
    /// bukkit: org/bukkit/scheduler/BukkitTask.isSync
    ///
    /// False for what `run_task_asynchronously` started.
    pub fn is_sync(&self) -> bool {
        !crate::scheduler::is_async(self.id)
    }
    /// bukkit: org/bukkit/scheduler/BukkitRunnable.getTaskId
    pub fn get_task_id(&self) -> u32 {
        self.id
    }
    /// bukkit: org/bukkit/scheduler/BukkitTask.cancel, org/bukkit/scheduler/BukkitRunnable.cancel
    pub fn cancel(&self) {
        crate::scheduler::cancel_task(self.id);
    }
    /// bukkit: org/bukkit/scheduler/BukkitTask.getOwner
    ///
    /// A library is one plugin, so every task it can see is its own.
    pub fn get_owner(&self) -> Option<Plugin> {
        get_own_plugin()
    }
}

/// Tasks cancelled so far: one that ran its course isn't one of them.
static CANCELLED_TASKS: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());

pub(crate) fn note_cancelled(id: u32) {
    CANCELLED_TASKS.lock().unwrap_or_else(|e| e.into_inner()).insert(id);
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.isQueued
pub fn is_queued(id: u32) -> bool {
    let state = crate::scheduler::STATE.lock().unwrap_or_else(|e| e.into_inner());
    !state.cancelled.contains(&id) && state.tasks.iter().any(|task| task.id == id)
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.isCurrentlyRunning
pub fn is_currently_running(id: u32) -> bool {
    crate::scheduler::RUNNING.with(std::cell::Cell::get) == id && id != 0
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.cancelTasks
///
/// All of this plugin's, as a library is one plugin.
pub fn cancel_tasks() {
    let ids: Vec<u32> = crate::scheduler::STATE.lock().unwrap_or_else(|e| e.into_inner()).tasks.iter().map(|task| task.id).collect();
    ids.into_iter().for_each(crate::scheduler::cancel_task);
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.getPendingTasks
pub fn get_pending_tasks() -> Vec<BukkitTask> {
    crate::scheduler::STATE.lock().unwrap_or_else(|e| e.into_inner()).tasks.iter().map(|task| BukkitTask { id: task.id }).collect()
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.callSyncMethod
///
/// `task` runs next tick on the server thread; its answer comes through the channel.
pub fn call_sync_method<T: Send + 'static>(task: impl FnOnce() -> T + Send + 'static) -> std::sync::mpsc::Receiver<T> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut task = Some(task);
    crate::scheduler::run_task(move || {
        if let Some(task) = task.take() {
            let _ = sender.send(task());
        }
    });
    receiver
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.scheduleAsyncDelayedTask
pub fn schedule_async_delayed_task(delay: u64, task: impl FnOnce() + Send + 'static) {
    crate::misc::run_task_later_asynchronously(delay, task);
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.scheduleAsyncRepeatingTask
pub fn schedule_async_repeating_task(delay: u64, period: u64, task: impl FnMut() -> bool + Send + 'static) {
    crate::misc::run_task_timer_asynchronously(delay, period, task);
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.getMainThreadExecutor
///
/// A function that runs a closure on the server thread, as Java's `Executor`.
pub fn get_main_thread_executor() -> impl Fn(Box<dyn FnOnce() + Send>) {
    |task| {
        let mut task = Some(task);
        crate::scheduler::run_task(move || {
            if let Some(task) = task.take() {
                task();
            }
        });
    }
}

// ---- Regions ----

impl World {
    /// bukkit: org/bukkit/RegionAccessor.setType
    pub fn set_type(&self, at: &Location, material: &Material) -> bool {
        at.get_block().set_type(material)
    }
    /// bukkit: org/bukkit/RegionAccessor.setBlockData
    pub fn set_block_data(&self, at: &Location, data: &str) -> bool {
        at.get_block().set_block_data(data)
    }
    /// bukkit: org/bukkit/RegionAccessor.getBlockData
    pub fn get_block_data(&self, at: &Location) -> String {
        at.get_block().get_block_data()
    }
    /// bukkit: org/bukkit/RegionAccessor.getBlockState
    pub fn get_block_state(&self, at: &Location) -> crate::BlockState {
        at.get_block().get_state()
    }
    /// bukkit: org/bukkit/RegionAccessor.getComputedBiome
    pub fn get_computed_biome(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.get_block_at(x, y, z).get_biome()
    }
    /// bukkit: org/bukkit/RegionAccessor.getMoonPhase
    pub fn get_moon_phase(&self) -> &'static str {
        let time = self.ask("level", json!({}))["day_time"].as_i64().unwrap_or(0);
        crate::enums::moon_phase::values().get(((time / 24000) % 8) as usize).copied().unwrap_or("FULL_MOON")
    }
    /// bukkit: org/bukkit/RegionAccessor.getKey
    pub fn get_key(&self) -> NamespacedKey {
        NamespacedKey::minecraft(self.dimension().trim_start_matches("minecraft:"))
    }
    /// bukkit: org/bukkit/RegionAccessor.getEntities, org/bukkit/RegionAccessor.getLivingEntities
    ///
    /// As `World.getEntities`: every entity but the players.
    pub fn region_entities(&self) -> Vec<Entity> {
        self.get_entities()
    }
    /// bukkit: org/bukkit/RegionAccessor.lineOfSightExists
    pub fn line_of_sight_exists(&self, from: &Location, to: &Location) -> bool {
        let origin = from.to_vector();
        let mut way = to.to_vector();
        way.subtract(&origin);
        let distance = way.length();
        way.normalize();
        self.ray_trace_cells(&origin, &way, distance, "NEVER", true).is_none()
    }
    /// bukkit: org/bukkit/RegionAccessor.hasCollisionsIn
    pub fn has_collisions_in(&self, bounds: &BoundingBox) -> bool {
        for x in bounds.get_min_x().floor() as i32..bounds.get_max_x().ceil() as i32 {
            for y in bounds.get_min_y().floor() as i32..bounds.get_max_y().ceil() as i32 {
                for z in bounds.get_min_z().floor() as i32..bounds.get_max_z().ceil() as i32 {
                    let block = self.get_block_at(x, y, z);
                    if !block.is_passable() && block.get_bounding_box().overlaps(bounds) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// `org.bukkit.BlockChangeDelegate`: what a tree writes through.
pub trait BlockChangeDelegate {
    /// bukkit: org/bukkit/BlockChangeDelegate.setBlockData
    fn set_block_data(&mut self, x: i32, y: i32, z: i32, data: &str) -> bool;
    /// bukkit: org/bukkit/BlockChangeDelegate.getBlockData
    fn get_block_data(&self, x: i32, y: i32, z: i32) -> String;
    /// bukkit: org/bukkit/BlockChangeDelegate.getHeight
    fn get_height(&self) -> i32;
    /// bukkit: org/bukkit/BlockChangeDelegate.isEmpty
    fn is_empty(&self, x: i32, y: i32, z: i32) -> bool;
}

impl BlockChangeDelegate for World {
    fn set_block_data(&mut self, x: i32, y: i32, z: i32, data: &str) -> bool {
        self.get_block_at(x, y, z).set_block_data(data)
    }
    fn get_block_data(&self, x: i32, y: i32, z: i32) -> String {
        self.get_block_at(x, y, z).get_block_data()
    }
    fn get_height(&self) -> i32 {
        self.get_max_height()
    }
    fn is_empty(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block_at(x, y, z).is_empty()
    }
}

// ---- What the server knows of worlds ----

/// `org.bukkit.generator.WorldInfo`.
pub trait WorldInfo {
    /// bukkit: org/bukkit/generator/WorldInfo.getSeed
    fn get_seed(&self) -> i64;
    /// bukkit: org/bukkit/generator/WorldInfo.getUID
    fn get_uid(&self) -> String;
}

impl WorldInfo for World {
    fn get_seed(&self) -> i64 {
        World::get_seed(self)
    }
    fn get_uid(&self) -> String {
        World::get_uid(self)
    }
}

// ---- Unsafe values ----

impl crate::core2::UnsafeValues {
    /// bukkit: org/bukkit/UnsafeValues.modifyItemStack
    ///
    /// `arguments` as `/give` takes them: `[custom_name="Sword",damage=5]`.
    pub fn modify_item_stack(&self, stack: &ItemStack, arguments: &str) -> ItemStack {
        let mut changed = stack.clone();
        let found = query(json!({ "op": "core_item_components", "text": arguments }));
        for (id, value) in found.as_object().into_iter().flatten() {
            changed.set_component(&key_of(id), value.clone());
        }
        changed
    }
    /// bukkit: org/bukkit/UnsafeValues.fromLegacy
    ///
    /// The same material: there are no legacy ones in this `Material`.
    pub fn from_legacy(&self, material: &Material) -> Material {
        material.clone()
    }
    /// bukkit: org/bukkit/UnsafeValues.toLegacy
    pub fn to_legacy(&self, material: &Material) -> Material {
        material.clone()
    }
    /// bukkit: org/bukkit/UnsafeValues.getMaterial
    pub fn get_material(&self, name: &str, _version: i32) -> Material {
        Material::of(name)
    }
    /// bukkit: org/bukkit/UnsafeValues.getMainLevelName
    pub fn get_main_level_name(&self) -> String {
        property("level-name").unwrap_or_else(|| "world".to_owned())
    }
    /// bukkit: org/bukkit/UnsafeValues.isSupportedApiVersion
    pub fn is_supported_api_version(&self, version: &str) -> bool {
        version.split('.').next().and_then(|major| major.parse::<i32>().ok()).is_some()
    }
    /// bukkit: org/bukkit/UnsafeValues.isLegacyPlugin
    ///
    /// Whether `plugin.yml` has no `api-version`.
    pub fn is_legacy_plugin(&self, plugin: &Plugin) -> bool {
        plugin.get_api_version().is_none()
    }
    /// bukkit: org/bukkit/UnsafeValues.checkSupported
    pub fn check_supported(&self, plugin: &Plugin) -> bool {
        plugin.get_api_version().is_none_or(|version| self.is_supported_api_version(&version))
    }
    /// bukkit: org/bukkit/UnsafeValues.serializeItemAsJson
    pub fn serialize_item_as_json(&self, stack: &ItemStack) -> Value {
        stack.to_json()
    }
    /// bukkit: org/bukkit/UnsafeValues.deserializeItemFromJson, org/bukkit/UnsafeValues.deserializeStack
    pub fn deserialize_item_from_json(&self, data: &Value) -> Option<ItemStack> {
        ItemStack::read(data)
    }
    /// bukkit: org/bukkit/UnsafeValues.serializeEntity
    ///
    /// The entity's saved tree, in the game's binary form.
    pub fn serialize_entity(&self, entity: &Entity) -> Vec<u8> {
        let bytes = query(json!({ "op": "paper_nbt_bytes", "snbt": crate::data::snbt(&entity.get_data()) }));
        bytes.as_array().into_iter().flatten().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect()
    }
    /// bukkit: org/bukkit/UnsafeValues.deserializeEntity
    ///
    /// What `serialize_entity` wrote, spawned where it was saved. `preserve_uuid` keeps its id.
    pub fn deserialize_entity(&self, data: &[u8], world: &World, preserve_uuid: bool) -> Option<Entity> {
        let mut tree = query(json!({ "op": "core_nbt_read", "bytes": data }));
        if !tree.is_object() {
            return None;
        }
        if !preserve_uuid && let Some(fields) = tree.as_object_mut() {
            fields.remove("UUID");
        }
        Entity::read(&query(json!({ "op": "spawn", "dimension": world.dimension(), "snbt": crate::data::snbt(&tree) })))
    }
    /// bukkit~: org/bukkit/UnsafeValues.loadAdvancement | into the plugins' data pack, where Java's has one of its own; the game's tables are read again at once, but in the hooks that hold the hub themselves, at the end of the tick
    pub fn load_advancement(&self, key: &NamespacedKey, json_text: &str) -> Option<crate::core2::Advancement> {
        if crate::core2::get_advancement(key.as_str()).is_some() {
            return None;
        }
        let folder = world_folder().join(format!("datapacks/plugin-recipes/data/{}/advancement", key.get_namespace()));
        let file = folder.join(format!("{}.json", key.get_key()));
        std::fs::create_dir_all(file.parent().unwrap_or(&folder)).ok()?;
        std::fs::write(file, json_text).ok()?;
        run_now("datapack enable \"file/plugin-recipes\"");
        query(json!({ "op": "core_tables_reload" }));
        crate::core2::get_advancement(key.as_str())
    }
    /// bukkit~: org/bukkit/UnsafeValues.removeAdvancement | as `load_advancement`
    pub fn remove_advancement(&self, key: &NamespacedKey) -> bool {
        let file = world_folder().join(format!("datapacks/plugin-recipes/data/{}/advancement/{}.json", key.get_namespace(), key.get_key()));
        let removed = std::fs::remove_file(file).is_ok();
        if removed {
            query(json!({ "op": "core_tables_reload" }));
        }
        removed
    }
}

// ---- Enchantments ----

/// `org.bukkit.enchantments.EnchantmentOffer`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnchantmentOffer {
    enchantment: String,
    level: i32,
    cost: i32,
}

impl EnchantmentOffer {
    /// bukkit: org/bukkit/enchantments/EnchantmentOffer.<init>
    pub fn new(enchantment: &str, level: i32, cost: i32) -> Self {
        Self { enchantment: enchantment.to_owned(), level, cost }
    }
    /// bukkit: org/bukkit/enchantments/EnchantmentOffer.getEnchantment
    pub fn get_enchantment(&self) -> &str {
        &self.enchantment
    }
    /// bukkit: org/bukkit/enchantments/EnchantmentOffer.setEnchantment
    pub fn set_enchantment(&mut self, enchantment: &str) {
        enchantment.clone_into(&mut self.enchantment);
    }
    /// bukkit: org/bukkit/enchantments/EnchantmentOffer.getEnchantmentLevel
    pub fn get_enchantment_level(&self) -> i32 {
        self.level
    }
    /// bukkit: org/bukkit/enchantments/EnchantmentOffer.setEnchantmentLevel
    pub fn set_enchantment_level(&mut self, level: i32) {
        self.level = level;
    }
    /// bukkit: org/bukkit/enchantments/EnchantmentOffer.getCost
    pub fn get_cost(&self) -> i32 {
        self.cost
    }
    /// bukkit: org/bukkit/enchantments/EnchantmentOffer.setCost
    pub fn set_cost(&mut self, cost: i32) {
        self.cost = cost;
    }
}

/// bukkit: org/bukkit/enchantments/EnchantmentTarget.includes, org/bukkit/enchantments/EnchantmentTarget$2.includes, org/bukkit/enchantments/EnchantmentTarget$3.includes, org/bukkit/enchantments/EnchantmentTarget$4.includes, org/bukkit/enchantments/EnchantmentTarget$5.includes, org/bukkit/enchantments/EnchantmentTarget$6.includes, org/bukkit/enchantments/EnchantmentTarget$7.includes, org/bukkit/enchantments/EnchantmentTarget$8.includes, org/bukkit/enchantments/EnchantmentTarget$9.includes
///
/// By the game's own item tags, as Java's.
pub fn enchantment_target_includes(target: &str, material: &Material) -> bool {
    let tag = match target {
        "ARMOR" => "enchantable/armor",
        "ARMOR_FEET" => "enchantable/foot_armor",
        "ARMOR_LEGS" => "enchantable/leg_armor",
        "ARMOR_TORSO" => "enchantable/chest_armor",
        "ARMOR_HEAD" => "enchantable/head_armor",
        "WEAPON" => "enchantable/melee_weapon",
        "TOOL" => "enchantable/mining_loot",
        "BOW" => "enchantable/bow",
        "FISHING_ROD" => "enchantable/fishing",
        "BREAKABLE" => "enchantable/durability",
        "WEARABLE" => "enchantable/equippable",
        "TRIDENT" => "enchantable/trident",
        "CROSSBOW" => "enchantable/crossbow",
        "VANISHABLE" => "enchantable/vanishing",
        "ALL" => return ["ARMOR", "WEAPON", "TOOL", "BOW", "FISHING_ROD", "BREAKABLE", "WEARABLE", "TRIDENT", "CROSSBOW", "VANISHABLE"].iter().any(|one| enchantment_target_includes(one, material)),
        _ => return false,
    };
    crate::registry::Tag::get("item", tag).is_tagged(material.key())
}

impl crate::registry::Enchantment {
    /// bukkit: org/bukkit/enchantments/Enchantment.description
    pub fn description(&self) -> Component {
        Component(self.data()["description"].take())
    }
    fn data(&self) -> Value {
        query(json!({ "op": "data_file", "path": format!("enchantment/{}", self.get_key().trim_start_matches("minecraft:")) }))
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.displayName
    ///
    /// As an item's line shows it: with the level, in grey or red for a curse.
    pub fn display_name(&self, level: i32) -> Component {
        let mut text = json!({ "translate": self.translation_key(), "color": if self.is_cursed() { "red" } else { "gray" } });
        if level != 1 || self.get_max_level() != 1 {
            text = json!({ "text": "", "extra": [text, { "text": " " }, { "translate": format!("enchantment.level.{level}"), "color": if self.is_cursed() { "red" } else { "gray" } }] });
        }
        Component(text)
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getActiveSlotGroups
    ///
    /// By the game's own names for them: `mainhand`, `armor`, `any`.
    pub fn get_active_slot_groups(&self) -> Vec<String> {
        self.data()["slots"].as_array().into_iter().flatten().filter_map(|slot| slot.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getRarity
    pub fn get_rarity(&self) -> &'static str {
        match self.get_weight() {
            0..=1 => "VERY_RARE",
            2..=4 => "RARE",
            5..=9 => "UNCOMMON",
            _ => "COMMON",
        }
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getItemTarget
    ///
    /// `None`: Paper's throws, as enchantments go by item tags now: `can_enchant_item` is the way.
    pub fn get_item_target(&self) -> Option<&'static str> {
        None
    }
}

// ---- Lootables ----

impl crate::block_data::TileState {
    /// bukkit: org/bukkit/loot/Lootable.getLootTable
    pub fn get_loot_table(&self) -> Option<String> {
        self.saved("LootTable").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/loot/Lootable.hasLootTable
    pub fn has_loot_table(&self) -> bool {
        self.get_loot_table().is_some()
    }
    /// bukkit: org/bukkit/loot/Lootable.setLootTable
    ///
    /// With `update`, as a block state is written back.
    pub fn set_loot_table(&mut self, table: &str) {
        self.change("LootTable", crate::data::quoted(&key_of(table)));
    }
    /// bukkit: org/bukkit/loot/Lootable.getSeed
    pub fn get_loot_seed(&self) -> i64 {
        self.saved("LootTableSeed").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/loot/Lootable.setSeed
    pub fn set_loot_seed(&mut self, seed: i64) {
        self.change("LootTableSeed", format!("{seed}L"));
    }
}

// ---- Spawners ----

impl crate::pure::Spawner {
    /// bukkit: org/bukkit/spawner/BaseSpawner.getSpawnedType
    pub fn get_spawned_type(&self) -> Option<String> {
        self.0.get_state_data()["SpawnData"]["entity"]["id"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/spawner/BaseSpawner.setSpawnedType
    pub fn set_spawned_type(&self, kind: &str) -> bool {
        self.0.merge_state(&format!("{{SpawnData:{{entity:{{id:{}}}}}}}", crate::data::quoted(&key_of(kind))))
    }
    /// bukkit: org/bukkit/spawner/Spawner.isActivated
    pub fn is_activated(&self) -> bool {
        let range = f64::from(int(&self.0.get_state_data()["RequiredPlayerRange"]));
        let at = self.0.get_location();
        self.0.get_world().get_players().iter().any(|player| player.get_location().distance_squared(&at) <= range * range)
    }
    /// bukkit: org/bukkit/spawner/Spawner.resetTimer
    ///
    /// A delay between the minimum and the maximum, as the game picks one.
    pub fn reset_timer(&self) -> bool {
        let data = self.0.get_state_data();
        let (low, high) = (int(&data["MinSpawnDelay"]), int(&data["MaxSpawnDelay"]));
        let delay = low + (self.0.get_x().wrapping_mul(31).wrapping_add(crate::scheduler::now() as i32).rem_euclid((high - low).max(1)));
        self.0.merge_state(&format!("{{Delay:{delay}s}}"))
    }
    /// bukkit: org/bukkit/spawner/Spawner.setSpawnedItem
    pub fn set_spawned_item(&self, stack: &ItemStack) -> bool {
        self.0.merge_state(&format!("{{SpawnData:{{entity:{{id:\"minecraft:item\",Item:{}}}}}}}", stack.to_snbt()))
    }
}

impl Block {
    fn get_state_data(&self) -> Value {
        query(json!({ "op": "block_data", "dimension": self.get_world().dimension(), "pos": [self.get_x(), self.get_y(), self.get_z()] }))
    }
    fn merge_state(&self, snbt: &str) -> bool {
        query(json!({ "op": "block_merge", "dimension": self.get_world().dimension(), "pos": [self.get_x(), self.get_y(), self.get_z()], "snbt": snbt })).as_bool().unwrap_or(false)
    }
}

// ---- Attributes ----

impl crate::scoreboard::AttributeModifier {
    /// bukkit: org/bukkit/attribute/AttributeModifier.serialize
    pub fn serialize(&self) -> Value {
        json!({ "key": self.key, "amount": self.amount, "operation": self.operation })
    }
    /// bukkit: org/bukkit/attribute/AttributeModifier.deserialize
    pub fn deserialize(map: &Value) -> Option<Self> {
        Some(Self::new(map["key"].as_str()?, map["amount"].as_f64()?, map["operation"].as_str()?))
    }
}

// ---- Server links ----

/// `org.bukkit.ServerLinks$ServerLink`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerLink {
    kind: Option<String>,
    name: Option<String>,
    url: String,
}

impl ServerLink {
    /// bukkit: org/bukkit/ServerLinks$ServerLink.getType
    pub fn get_type(&self) -> Option<&str> {
        self.kind.as_deref()
    }
    /// bukkit: org/bukkit/ServerLinks$ServerLink.getUrl
    pub fn get_url(&self) -> &str {
        &self.url
    }
    /// bukkit: org/bukkit/ServerLinks$ServerLink.getDisplayName
    pub fn get_display_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// bukkit: org/bukkit/ServerLinks$ServerLink.displayName
    pub fn display_name(&self) -> Option<Component> {
        self.name.as_deref().map(adventure::legacy)
    }
}

// ---- Keys and names, as interfaces ----

/// `org.bukkit.Keyed`.
pub trait Keyed {
    /// bukkit: org/bukkit/Keyed.key
    fn key(&self) -> Option<NamespacedKey>;
}

impl Keyed for crate::registry::Tag {
    fn key(&self) -> Option<NamespacedKey> {
        self.get_key()
    }
}

impl Keyed for crate::registry::Enchantment {
    fn key(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(self.get_key())
    }
}

impl Keyed for Material {
    fn key(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(Material::key(self))
    }
}

/// `org.bukkit.Translatable`.
pub trait Translatable {
    /// bukkit: org/bukkit/Translatable.getTranslationKey
    fn get_translation_key(&self) -> String;
}

impl Translatable for Block {
    fn get_translation_key(&self) -> String {
        Block::get_translation_key(self)
    }
}

impl Translatable for crate::registry::Enchantment {
    fn get_translation_key(&self) -> String {
        self.translation_key()
    }
}

/// bukkit: org/bukkit/command/CommandSender.getServer, org/bukkit/command/MessageCommandSender.getServer
///
/// There's no object to fetch: the server is this crate's functions, as `extra::SERVER_OBJECTS` says.
pub const SERVER_IS_THE_CRATE: () = ();

/// bukkit: org/bukkit/command/CommandExecutor, org/bukkit/command/CommandExecutor.onCommand
pub type CommandExecutor = crate::Executor;

impl CommandSender {
    /// bukkit: org/bukkit/command/CommandSender.spigot, org/bukkit/command/CommandSender$Spigot.sendMessage
    ///
    /// BungeeCord's components are this crate's `Component` here.
    pub fn spigot_send_message(&self, text: &Component) {
        self.send_component(text);
    }
}

/// bukkit: org/bukkit/Server$Spigot.broadcast
pub fn spigot_broadcast(text: &Component) {
    crate::server::broadcast(text);
}

// ---- Status of the server ----

/// bukkit: org/bukkit/Server.isPaused
pub fn is_paused() -> bool {
    query(json!({ "op": "core_paused" })).as_bool().unwrap_or(false)
}

impl World {
    fn void_rule(&self) -> Value {
        self.ask("core_void", json!({}))
    }
    /// bukkit: org/bukkit/World.isVoidDamageEnabled
    pub fn is_void_damage_enabled(&self) -> bool {
        self.void_rule()["enabled"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/World.setVoidDamageEnabled
    pub fn set_void_damage_enabled(&self, on: bool) {
        self.ask("core_void_set", json!({ "enabled": on }));
    }
    /// bukkit: org/bukkit/World.getVoidDamageAmount
    pub fn get_void_damage_amount(&self) -> f32 {
        self.void_rule()["amount"].as_f64().unwrap_or(4.0) as f32
    }
    /// bukkit: org/bukkit/World.setVoidDamageAmount
    pub fn set_void_damage_amount(&self, amount: f32) {
        self.ask("core_void_set", json!({ "amount": amount }));
    }
    /// bukkit: org/bukkit/World.getVoidDamageMinBuildHeightOffset
    pub fn get_void_damage_min_build_height_offset(&self) -> f64 {
        self.void_rule()["offset"].as_f64().unwrap_or(-64.0)
    }
    /// bukkit: org/bukkit/World.setVoidDamageMinBuildHeightOffset
    pub fn set_void_damage_min_build_height_offset(&self, offset: f64) {
        self.ask("core_void_set", json!({ "offset": offset }));
    }
    /// bukkit: org/bukkit/World.getFeatureFlags
    ///
    /// The keys of the data packs' feature flags this world has on.
    pub fn get_feature_flags(&self) -> Vec<String> {
        query(json!({ "op": "core_feature_flags" })).as_array().into_iter().flatten().filter_map(|flag| flag.as_str().map(str::to_owned)).collect()
    }
}

// ---- Maps from a search ----

/// bukkit: org/bukkit/Bukkit.createExplorerMap, org/bukkit/Server.createExplorerMap
///
/// A map of the nearest such structure, with the game's red cross on it.
pub fn create_explorer_map(world: &World, from: &Location, structure: &str) -> Option<ItemStack> {
    let found = world.locate_nearest_structure(from, structure, 100, false)?;
    let spot = found.get_location();
    let id = query(json!({ "op": "core_map_create", "dimension": world.dimension(), "at": [spot.x, spot.z], "scale": 2 })).as_i64()?;
    let mut stack = ItemStack::new(Material::of("minecraft:filled_map"), 1);
    stack.set_component("minecraft:map_id", json!(id));
    stack.set_component("minecraft:map_decorations", json!({ "+": { "type": "minecraft:red_x", "x": spot.x, "z": spot.z, "rotation": 180.0 } }));
    Some(stack)
}

/// `org.bukkit.inventory.ItemCraftResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemCraftResult {
    result: Option<ItemStack>,
    matrix: Vec<Option<ItemStack>>,
    overflow: Vec<ItemStack>,
}

impl ItemCraftResult {
    /// bukkit: org/bukkit/inventory/ItemCraftResult.getResult
    pub fn get_result(&self) -> Option<&ItemStack> {
        self.result.as_ref()
    }
    /// bukkit: org/bukkit/inventory/ItemCraftResult.getResultingMatrix
    pub fn get_resulting_matrix(&self) -> &[Option<ItemStack>] {
        &self.matrix
    }
    /// bukkit: org/bukkit/inventory/ItemCraftResult.getOverflowItems
    pub fn get_overflow_items(&self) -> &[ItemStack] {
        &self.overflow
    }
}

/// bukkit: org/bukkit/Bukkit.craftItemResult, org/bukkit/Server.craftItemResult
pub fn craft_item_result(grid: &[Option<ItemStack>]) -> ItemCraftResult {
    let result = craft_item(grid);
    let (matrix, overflow) = crate::fam_items::craft_rest(grid, result.is_some());
    ItemCraftResult { result, matrix, overflow }
}

// ---- Server links ----

/// bukkit: org/bukkit/Bukkit.getServerLinks, org/bukkit/Server.getServerLinks, org/bukkit/ServerLinks.getLinks
///
/// The links players are shown: `ServerLinks` changes them.
pub fn get_server_links() -> Vec<ServerLink> {
    let kind = |link: &Value| link["known"].as_u64().and_then(|index| LINK_TYPES.get(index as usize)).map(|name| (*name).to_owned());
    query(json!({ "op": "core_links" })).as_array().into_iter().flatten().filter_map(|link| Some(ServerLink { kind: kind(link), name: link["label"].as_str().map(str::to_owned), url: link["url"].as_str()?.to_owned() })).collect()
}

// ---- Bans in full ----

impl crate::core2::BanEntry {
    /// bukkit: org/bukkit/BanEntry.setSource
    pub fn set_source(&mut self, source: &str) {
        self.row["source"] = json!(source);
    }
    /// bukkit: org/bukkit/BanEntry.setReason
    pub fn set_reason(&mut self, reason: &str) {
        self.row["reason"] = json!(reason);
    }
    /// bukkit: org/bukkit/BanEntry.setExpiration
    ///
    /// In the file's own form; `None` is for good.
    pub fn set_expiration(&mut self, when: Option<&str>) {
        self.row["expires"] = json!(when.unwrap_or("forever"));
    }
    /// bukkit: org/bukkit/BanEntry.setCreated
    pub fn set_created(&mut self, when: &str) {
        self.row["created"] = json!(when);
    }
    /// bukkit: org/bukkit/BanEntry.save
    pub fn save(&self) -> bool {
        query(json!({ "op": "core_ban_save", "addresses": self.addresses, "row": self.row })).as_bool().unwrap_or(false)
    }
}

// ---- Scores ----

impl crate::scoreboard::Score {
    /// bukkit: org/bukkit/scoreboard/Score.customName
    pub fn custom_name(&self) -> Option<Component> {
        let score = query(json!({ "op": "core_score", "entry": self.get_entry(), "objective": self.get_objective().get_name() }));
        score.get("display").filter(|name| !name.is_null()).map(|name| Component(name.clone()))
    }
}

impl crate::scoreboard::Team {
    /// bukkit: org/bukkit/scoreboard/Team.addEntities
    pub fn add_entities(&self, entities: &[&Entity]) {
        entities.iter().for_each(|entity| self.add_entity(entity));
    }
    /// bukkit: org/bukkit/scoreboard/Team.removeEntities
    pub fn remove_entities(&self, entities: &[&Entity]) -> bool {
        entities.iter().fold(false, |any, entity| self.remove_entity(entity) || any)
    }
}

// ---- Items of the block world ----

impl crate::block_data::TileState {
    /// bukkit: org/bukkit/loot/Lootable.clearLootTable
    pub fn clear_loot_table(&mut self) {
        self.change("LootTable", "\"\"".to_owned());
    }
}

impl crate::pure::Spawner {
    /// bukkit: org/bukkit/spawner/Spawner.setDelay
    pub fn set_spawn_delay(&self, delay: i32) -> bool {
        self.0.merge_state(&format!("{{Delay:{delay}s}}"))
    }
}

impl crate::core2::ChunkSnapshot {
    /// bukkit: org/bukkit/ChunkSnapshot.getData
    ///
    /// The old magic value, from the block's age, level or power.
    pub fn get_data(&self, x: i32, y: i32, z: i32) -> u8 {
        let data = self.get_block_data(x, y, z);
        let properties = data.split('[').nth(1).unwrap_or_default();
        properties.trim_end_matches(']').split(',').filter_map(|row| row.split_once('=')).find(|(key, _)| matches!(*key, "age" | "level" | "power" | "moisture" | "stage" | "layers")).and_then(|(_, value)| value.parse().ok()).unwrap_or(0)
    }
}

/// A trial spawner's normal or ominous settings, over its saved data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrialSpawnerConfiguration {
    block: Block,
    field: &'static str,
}

impl TrialSpawnerConfiguration {
    /// The settings of a trial spawner block: `ominous` for the other set.
    pub fn of(block: &Block, ominous: bool) -> Self {
        Self { block: block.clone(), field: if ominous { "ominous_config" } else { "normal_config" } }
    }
    /// The settings, inline or, as the game's own spawners have them, by name in `trial_spawner`.
    fn settings(&self) -> Value {
        let data = self.block.get_state_data();
        match data[self.field].as_str() {
            Some(name) => query(json!({ "op": "data_file", "path": format!("trial_spawner/{}", name.trim_start_matches("minecraft:")) })),
            None => data[self.field].clone(),
        }
    }
    /// `TrialSpawnerConfig.DEFAULT`, for what a set of settings leaves out.
    fn number(&self, key: &str) -> f32 {
        let default = match key {
            "total_mobs" => 6.0,
            "simultaneous_mobs" | "total_mobs_added_per_player" => 2.0,
            "simultaneous_mobs_added_per_player" => 1.0,
            _ => 0.0,
        };
        self.settings()[key].as_f64().unwrap_or(default) as f32
    }
    /// The whole set goes on the spawner: a change to a named one makes it this spawner's own.
    fn change(&self, edit: impl FnOnce(&mut Value)) -> bool {
        let mut settings = self.settings();
        if !settings.is_object() {
            settings = json!({});
        }
        edit(&mut settings);
        self.block.merge_state(&format!("{{{}:{}}}", self.field, crate::data::snbt(&settings)))
    }
    fn set_number(&self, key: &str, value: f32) -> bool {
        self.change(|settings| settings[key] = json!(value))
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.getBaseSpawnsBeforeCooldown
    pub fn get_base_spawns_before_cooldown(&self) -> f32 {
        self.number("total_mobs")
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.setBaseSpawnsBeforeCooldown
    pub fn set_base_spawns_before_cooldown(&self, value: f32) -> bool {
        self.set_number("total_mobs", value)
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.getBaseSimultaneousEntities
    pub fn get_base_simultaneous_entities(&self) -> f32 {
        self.number("simultaneous_mobs")
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.setBaseSimultaneousEntities
    pub fn set_base_simultaneous_entities(&self, value: f32) -> bool {
        self.set_number("simultaneous_mobs", value)
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.getAdditionalSpawnsBeforeCooldown
    pub fn get_additional_spawns_before_cooldown(&self) -> f32 {
        self.number("total_mobs_added_per_player")
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.setAdditionalSpawnsBeforeCooldown
    pub fn set_additional_spawns_before_cooldown(&self, value: f32) -> bool {
        self.set_number("total_mobs_added_per_player", value)
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.getAdditionalSimultaneousEntities
    pub fn get_additional_simultaneous_entities(&self) -> f32 {
        self.number("simultaneous_mobs_added_per_player")
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.setAdditionalSimultaneousEntities
    pub fn set_additional_simultaneous_entities(&self, value: f32) -> bool {
        self.set_number("simultaneous_mobs_added_per_player", value)
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.getPossibleRewards
    ///
    /// Each loot table with its weight.
    pub fn get_possible_rewards(&self) -> BTreeMap<String, i32> {
        self.settings()["loot_tables_to_eject"].as_array().into_iter().flatten().filter_map(|row| Some((row["data"].as_str()?.to_owned(), int(&row["weight"])))).collect()
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.setPossibleRewards
    pub fn set_possible_rewards(&self, rewards: &BTreeMap<String, i32>) -> bool {
        let rows: Vec<Value> = rewards.iter().map(|(table, weight)| json!({ "data": key_of(table), "weight": weight })).collect();
        self.change(|settings| settings["loot_tables_to_eject"] = Value::Array(rows))
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.addPossibleReward
    pub fn add_possible_reward(&self, table: &str, weight: i32) -> bool {
        let mut rewards = self.get_possible_rewards();
        rewards.insert(key_of(table), weight);
        self.set_possible_rewards(&rewards)
    }
    /// bukkit: org/bukkit/spawner/TrialSpawnerConfiguration.removePossibleReward
    pub fn remove_possible_reward(&self, table: &str) -> bool {
        let mut rewards = self.get_possible_rewards();
        rewards.remove(&key_of(table));
        self.set_possible_rewards(&rewards)
    }
}

// ---- Structures, as the registry has them ----

/// bukkit: org/bukkit/generator/structure/Structure.getKey, org/bukkit/generator/structure/Structure.key
pub fn generator_structure_key(id: &str) -> Option<NamespacedKey> {
    NamespacedKey::from_string(&key_of(id))
}

/// bukkit: org/bukkit/generator/structure/Structure.getStructureType
///
/// The game's type of a structure: `minecraft:jigsaw`.
pub fn generator_structure_type(id: &str) -> Option<String> {
    let name = key_of(id);
    query(json!({ "op": "data_file", "path": format!("worldgen/structure/{}", name.trim_start_matches("minecraft:")) }))["type"].as_str().map(str::to_owned)
}

// ---- Metadata ----

impl crate::extra::MetadataValue {
    /// bukkit: org/bukkit/metadata/MetadataValue.invalidate, org/bukkit/metadata/FixedMetadataValue.invalidate
    ///
    /// Nothing, as Java's fixed value: it has nothing cached.
    pub fn invalidate(&self) {}
}

/// `org.bukkit.metadata.LazyMetadataValue`: worked out when asked, and kept until `invalidate`.
pub struct LazyMetadataValue {
    compute: Box<dyn Fn() -> Value + Send + Sync>,
    cached: Mutex<Option<Value>>,
}

impl LazyMetadataValue {
    /// bukkit: org/bukkit/metadata/LazyMetadataValue.<init>
    pub fn new(compute: impl Fn() -> Value + Send + Sync + 'static) -> Self {
        Self { compute: Box::new(compute), cached: Mutex::new(None) }
    }
    /// bukkit: org/bukkit/metadata/LazyMetadataValue.value
    pub fn value(&self) -> Value {
        self.cached.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(|| (self.compute)()).clone()
    }
    /// bukkit: org/bukkit/metadata/LazyMetadataValue.invalidate
    pub fn invalidate(&self) {
        *self.cached.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

// ---- Permissions in code ----

/// bukkit: org/bukkit/util/permissions/DefaultPermissions.registerPermission
pub fn register_permission(permission: &crate::config2::Permission) {
    PluginManager.add_permission(permission);
}

/// bukkit: org/bukkit/util/permissions/BroadcastPermissions.registerPermissions
pub fn register_broadcast_permissions(parent: &mut crate::config2::Permission) {
    for (node, default) in [("bukkit.broadcast.admin", "op"), ("bukkit.broadcast.user", "true")] {
        crate::set_permission_default(node, default);
        parent.children.insert(node.to_owned(), true);
    }
}

/// bukkit: org/bukkit/util/permissions/CommandPermissions.registerPermissions
pub fn register_command_permissions(parent: &mut crate::config2::Permission) {
    for (node, default) in [("bukkit.command.plugins", "true"), ("bukkit.command.version", "true"), ("bukkit.command.reload", "op"), ("bukkit.command.help", "true")] {
        crate::set_permission_default(node, default);
        parent.children.insert(node.to_owned(), true);
    }
}

/// bukkit: org/bukkit/util/permissions/DefaultPermissions.registerCorePermissions
pub fn register_core_permissions() {
    let mut root = crate::config2::Permission::new("bukkit", "op", "Gives the user the ability to use all CraftBukkit utilities and commands");
    register_broadcast_permissions(&mut root);
    register_command_permissions(&mut root);
    register_permission(&root);
}

/// What runs when an attachment goes: `org.bukkit.permissions.PermissionRemovedExecutor`.
/// bukkit: org/bukkit/permissions/PermissionRemovedExecutor, org/bukkit/permissions/PermissionRemovedExecutor.attachmentRemoved
pub type PermissionRemovedExecutor = std::sync::Arc<dyn Fn(&crate::config2::PermissionAttachment) + Send + Sync>;

// ---- Tasks run off the server thread ----

/// bukkit: org/bukkit/scheduler/BukkitRunnable.runTaskLaterAsynchronously
pub fn runnable_task_later_asynchronously(delay: u64, task: impl FnOnce() + Send + 'static) {
    crate::misc::run_task_later_asynchronously(delay, task);
}

/// bukkit: org/bukkit/scheduler/BukkitRunnable.runTaskTimerAsynchronously
pub fn runnable_task_timer_asynchronously(delay: u64, period: u64, task: impl FnMut() -> bool + Send + 'static) {
    crate::misc::run_task_timer_asynchronously(delay, period, task);
}

// ---- Registries of things that aren't data files ----

/// bukkit: org/bukkit/Registry$2.get
pub fn registry_boss_bar(key: &str) -> Option<BossBar> {
    get_boss_bar(key)
}

/// bukkit: org/bukkit/Registry$2.iterator
pub fn registry_boss_bars() -> Vec<BossBar> {
    get_boss_bars()
}

/// bukkit: org/bukkit/enchantments/EnchantmentWrapper.getEnchantment
pub fn enchantment_wrapper(key: &str) -> Option<crate::registry::Enchantment> {
    crate::registry::Enchantment::get_by_key(key)
}

impl crate::registry::Enchantment {
    /// bukkit: org/bukkit/enchantments/Enchantment.getActiveSlots
    pub fn get_active_slots(&self) -> Vec<&'static str> {
        let mut slots = Vec::new();
        for group in self.get_active_slot_groups() {
            let names: &[&'static str] = match group.as_str() {
                "mainhand" => &["HAND"],
                "offhand" => &["OFF_HAND"],
                "hand" => &["HAND", "OFF_HAND"],
                "feet" => &["FEET"],
                "legs" => &["LEGS"],
                "chest" => &["CHEST"],
                "head" => &["HEAD"],
                "armor" => &["FEET", "LEGS", "CHEST", "HEAD", "BODY"],
                "body" => &["BODY"],
                "saddle" => &["SADDLE"],
                _ => &["HAND", "OFF_HAND", "FEET", "LEGS", "CHEST", "HEAD", "BODY", "SADDLE"],
            };
            for name in names {
                if !slots.contains(name) {
                    slots.push(*name);
                }
            }
        }
        slots
    }
}

// ---- The old `Attribute` UUID ----

fn md5(data: &[u8]) -> [u8; 16] {
    const SHIFTS: [u32; 64] = [7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21];
    let table: Vec<u32> = (0..64).map(|i| (f64::from(i + 1).sin().abs() * 4_294_967_296.0) as u32).collect();
    let mut state = [0x6745_2301_u32, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&((data.len() as u64).wrapping_mul(8)).to_le_bytes());
    for block in message.chunks_exact(64) {
        let words: Vec<u32> = block.chunks_exact(4).map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]])).collect();
        let [mut a, mut b, mut c, mut d] = state;
        for i in 0..64 {
            let (mix, index) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let next = a.wrapping_add(mix).wrapping_add(table[i]).wrapping_add(words[index]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(next.rotate_left(SHIFTS[i]));
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
    }
    let mut out = [0; 16];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

impl crate::scoreboard::AttributeModifier {
    /// bukkit: org/bukkit/attribute/AttributeModifier.getUniqueId
    ///
    /// The UUID its key is, if it's one; else `UUID.nameUUIDFromBytes` of the key, as Paper's.
    pub fn get_unique_id(&self) -> String {
        let path = self.key.rsplit(':').next().unwrap_or_default();
        if path.len() == 36 && path.bytes().filter(|byte| *byte == b'-').count() == 4 {
            return path.to_owned();
        }
        name_uuid(&self.key)
    }
}

/// `UUID.nameUUIDFromBytes`.
fn name_uuid(name: &str) -> String {
    let mut hash = md5(name.as_bytes());
    hash[6] = (hash[6] & 0x0f) | 0x30;
    hash[8] = (hash[8] & 0x3f) | 0x80;
    crate::dashed(&hash.iter().map(|byte| format!("{byte:02x}")).collect::<String>())
}

// ---- Fireworks ----

impl crate::meta_lists::FireworkEffect {
    /// bukkit: org/bukkit/FireworkEffect.serialize
    pub fn serialize(&self) -> Value {
        let colors = |list: Vec<Color>| list.into_iter().map(Color::serialize).collect::<Vec<_>>();
        let saved = self.saved_value();
        json!({ "flicker": saved["has_twinkle"], "trail": saved["has_trail"], "colors": colors(self.get_colors()), "fade-colors": colors(self.get_fade_colors()), "type": saved["shape"] })
    }
}

// ---- Configuration: sections as paths, comments, objects ----

/// `org.bukkit.configuration.SectionPathData`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SectionPathData {
    data: Option<Value>,
    comments: Vec<String>,
    inline_comments: Vec<String>,
}

impl SectionPathData {
    /// bukkit: org/bukkit/configuration/SectionPathData.getData
    pub fn get_data(&self) -> Option<&Value> {
        self.data.as_ref()
    }
    /// bukkit: org/bukkit/configuration/SectionPathData.setData
    pub fn set_data(&mut self, data: Option<Value>) {
        self.data = data;
    }
    /// bukkit: org/bukkit/configuration/SectionPathData.getComments
    pub fn get_comments(&self) -> &[String] {
        &self.comments
    }
    /// bukkit: org/bukkit/configuration/SectionPathData.setComments
    pub fn set_comments(&mut self, comments: &[String]) {
        comments.clone_into(&mut self.comments);
    }
    /// bukkit: org/bukkit/configuration/SectionPathData.getInlineComments
    pub fn get_inline_comments(&self) -> &[String] {
        &self.inline_comments
    }
    /// bukkit: org/bukkit/configuration/SectionPathData.setInlineComments
    pub fn set_inline_comments(&mut self, comments: &[String]) {
        comments.clone_into(&mut self.inline_comments);
    }
}

/// Comments of each key of a file: above it, and on its line.
type CommentMap = BTreeMap<String, (Vec<String>, Vec<String>)>;

/// What `options()` set on a configuration, and its comments, by file.
#[derive(Default)]
struct FileExtras {
    options: Option<ConfigurationOptions>,
    comments: Option<CommentMap>,
}

static EXTRAS: Mutex<BTreeMap<PathBuf, FileExtras>> = Mutex::new(BTreeMap::new());

/// The dotted key each line of YAML sets, with the comments before it. Lists and block text
/// aren't keys; a comment stays with the next key.
fn scan_comments(text: &str) -> CommentMap {
    let mut out = CommentMap::new();
    let mut parents: Vec<(usize, String)> = Vec::new();
    let mut above: Vec<String> = Vec::new();
    for line in text.lines() {
        let body = line.trim_start();
        let depth = line.len() - body.len();
        if let Some(comment) = body.strip_prefix('#') {
            above.push(comment.strip_prefix(' ').unwrap_or(comment).to_owned());
            continue;
        }
        if body.is_empty() || body.starts_with('-') {
            continue;
        }
        let Some((key, rest)) = body.split_once(':') else {
            continue;
        };
        if key.contains(' ') && !key.starts_with(['"', '\'']) {
            continue;
        }
        while parents.last().is_some_and(|(indent, _)| *indent >= depth) {
            parents.pop();
        }
        let key = key.trim_matches(['"', '\'']).to_owned();
        let mut path: Vec<&str> = parents.iter().map(|(_, name)| name.as_str()).collect();
        path.push(&key);
        let path = path.join(".");
        let inline = rest.find(" #").map(|at| rest[at + 2..].trim_start().to_owned());
        if !above.is_empty() || inline.is_some() {
            out.insert(path, (std::mem::take(&mut above), inline.into_iter().collect()));
        }
        parents.push((depth, key));
    }
    out
}

/// The text of a configuration with its comments put back in.
fn weave_comments(text: &str, comments: &CommentMap) -> String {
    let mut out = String::new();
    let mut parents: Vec<(usize, String)> = Vec::new();
    for line in text.lines() {
        let body = line.trim_start();
        let depth = line.len() - body.len();
        let key = body.split_once(':').map(|(key, _)| key.trim_matches(['"', '\'']).to_owned()).filter(|key| !body.starts_with('-') && !key.contains(' '));
        let Some(key) = key else {
            out.push_str(line);
            out.push('\n');
            continue;
        };
        while parents.last().is_some_and(|(indent, _)| *indent >= depth) {
            parents.pop();
        }
        let mut path: Vec<&str> = parents.iter().map(|(_, name)| name.as_str()).collect();
        path.push(&key);
        let row = comments.get(&path.join("."));
        for comment in row.iter().flat_map(|row| &row.0) {
            out.push_str(&" ".repeat(depth));
            out.push_str(&format!("# {comment}\n").replace("# \n", "#\n"));
        }
        out.push_str(line);
        if let Some(inline) = row.and_then(|row| row.1.first()) {
            out.push_str(&format!(" # {inline}"));
        }
        out.push('\n');
        parents.push((depth, key));
    }
    out
}

/// `options()`: live while it's held, kept by file for the next `save_commented`.
pub struct BoundOptions<'a> {
    config: &'a mut crate::config::YamlConfiguration,
    settings: ConfigurationOptions,
}

impl std::ops::Deref for BoundOptions<'_> {
    type Target = ConfigurationOptions;
    fn deref(&self) -> &ConfigurationOptions {
        &self.settings
    }
}

impl std::ops::DerefMut for BoundOptions<'_> {
    fn deref_mut(&mut self) -> &mut ConfigurationOptions {
        &mut self.settings
    }
}

impl Drop for BoundOptions<'_> {
    fn drop(&mut self) {
        if let Some(path) = self.config.path.clone() {
            EXTRAS.lock().unwrap_or_else(|e| e.into_inner()).entry(path).or_default().options = Some(self.settings.clone());
        }
    }
}

impl BoundOptions<'_> {
    /// bukkit: org/bukkit/configuration/ConfigurationOptions.configuration, org/bukkit/configuration/MemoryConfigurationOptions.configuration, org/bukkit/configuration/file/FileConfigurationOptions.configuration, org/bukkit/configuration/file/YamlConfigurationOptions.configuration
    pub fn configuration(&self) -> &crate::config::YamlConfiguration {
        self.config
    }
    /// bukkit: org/bukkit/configuration/file/YamlConfigurationOptions.copyDefaults, org/bukkit/configuration/file/FileConfigurationOptions.copyDefaults
    ///
    /// On, what the defaults say and the file doesn't is copied in at once.
    pub fn set_copy_defaults(&mut self, on: bool) -> &mut Self {
        self.settings.copy_defaults = on;
        self.config.options_copy_defaults(on);
        self
    }
}

/// `org.bukkit.configuration.ConfigurationSection` as a place in its file, for the calls that
/// need to know where it is. `get_configuration_section` hands a detached copy; this is the path.
#[derive(Clone, Copy, Debug)]
pub struct SectionRef<'a> {
    root: &'a crate::config::YamlConfiguration,
    path: &'a str,
}

impl<'a> SectionRef<'a> {
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getName, org/bukkit/configuration/MemorySection.getName
    pub fn get_name(&self) -> &'a str {
        self.path.rsplit('.').next().unwrap_or_default()
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getCurrentPath, org/bukkit/configuration/MemorySection.getCurrentPath
    pub fn get_current_path(&self) -> &'a str {
        self.path
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getRoot, org/bukkit/configuration/MemorySection.getRoot
    pub fn get_root(&self) -> &'a crate::config::YamlConfiguration {
        self.root
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getParent, org/bukkit/configuration/MemorySection.getParent, org/bukkit/configuration/MemoryConfiguration.getParent
    pub fn get_parent(&self) -> Option<Self> {
        if self.path.is_empty() {
            return None;
        }
        Some(Self { root: self.root, path: self.path.rsplit_once('.').map_or("", |(parent, _)| parent) })
    }
    /// bukkit: org/bukkit/configuration/MemorySection.createPath
    pub fn create_path(&self, key: &str) -> String {
        if self.path.is_empty() { key.to_owned() } else { format!("{}.{key}", self.path) }
    }
}

/// `org.bukkit.configuration.serialization.ConfigurationSerializable`: a value of a plugin's own in
/// a YAML file, under `==` with its alias.
pub trait ConfigurationSerializable: Sized {
    /// `@SerializableAs`, or the class's name.
    fn alias() -> &'static str;
    fn serialize(&self) -> serde_json::Map<String, Value>;
    fn deserialize(map: &serde_json::Map<String, Value>) -> Option<Self>;
}

/// `ConfigurationSerialization`'s own, in its order: alias and class. Each answers to both.
const SERIALIZABLE_BUILT_IN: [(&str, &str); 11] = [
    ("Vector", "org.bukkit.util.Vector"),
    ("BlockVector", "org.bukkit.util.BlockVector"),
    ("org.bukkit.inventory.ItemStack", "org.bukkit.inventory.ItemStack"),
    ("Color", "org.bukkit.Color"),
    ("PotionEffect", "org.bukkit.potion.PotionEffect"),
    ("Firework", "org.bukkit.FireworkEffect"),
    ("Pattern", "org.bukkit.block.banner.Pattern"),
    ("org.bukkit.Location", "org.bukkit.Location"),
    ("org.bukkit.attribute.AttributeModifier", "org.bukkit.attribute.AttributeModifier"),
    ("BoundingBox", "org.bukkit.util.BoundingBox"),
    ("SpawnRule", "org.bukkit.block.spawner.SpawnRule"),
];

/// By alias and by class's name, what each is.
static SERIALIZABLE: Mutex<Option<BTreeMap<String, String>>> = Mutex::new(None);

fn serializable<T>(body: impl FnOnce(&mut BTreeMap<String, String>) -> T) -> T {
    let mut held = SERIALIZABLE.lock().unwrap_or_else(|e| e.into_inner());
    body(held.get_or_insert_with(|| SERIALIZABLE_BUILT_IN.iter().flat_map(|(alias, class)| [((*alias).to_owned(), (*class).to_owned()), ((*class).to_owned(), (*class).to_owned())]).collect()))
}

/// `Class` of a configuration type: all Rust has of one, its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializableClass {
    name: String,
}

impl SerializableClass {
    /// `Class.getName`.
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Whether this is `T`, as a `==` on a `Class` asks.
    pub fn is<T: ConfigurationSerializable>(&self) -> bool {
        self.name == std::any::type_name::<T>()
    }
}

/// bukkit: org/bukkit/configuration/serialization/ConfigurationSerialization.registerClass
pub fn register_class<T: ConfigurationSerializable>() {
    let class = std::any::type_name::<T>().to_owned();
    serializable(|all| {
        all.insert(T::alias().to_owned(), class.clone());
        all.insert(class.clone(), class);
    });
}

/// bukkit: org/bukkit/configuration/serialization/ConfigurationSerialization.unregisterClass
pub fn unregister_class(alias: &str) {
    serializable(|all| {
        all.remove(alias);
    });
}

/// bukkit: org/bukkit/configuration/serialization/ConfigurationSerialization.getAlias
pub fn get_alias<T: ConfigurationSerializable>() -> &'static str {
    T::alias()
}

/// bukkit: org/bukkit/configuration/serialization/ConfigurationSerialization.getClassByAlias
pub fn get_class_by_alias(alias: &str) -> Option<SerializableClass> {
    serializable(|all| all.get(alias).cloned()).map(|name| SerializableClass { name })
}

/// bukkit: org/bukkit/configuration/serialization/ConfigurationSerialization.deserializeObject, org/bukkit/configuration/serialization/ConfigurationSerialization.deserialize
pub fn deserialize_object<T: ConfigurationSerializable>(map: &serde_json::Map<String, Value>) -> Option<T> {
    match map.get("==").and_then(Value::as_str) {
        Some(alias) if alias != T::alias() => None,
        _ => T::deserialize(map),
    }
}

impl crate::config::YamlConfiguration {
    fn extras<R>(&self, read: impl FnOnce(&mut FileExtras, &str) -> R) -> R {
        let path = self.path.clone().unwrap_or_default();
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        read(EXTRAS.lock().unwrap_or_else(|e| e.into_inner()).entry(path).or_default(), &text)
    }
    /// bukkit: org/bukkit/configuration/file/FileConfiguration.options, org/bukkit/configuration/file/YamlConfiguration.options, org/bukkit/configuration/Configuration.options, org/bukkit/configuration/MemoryConfiguration.options
    pub fn options(&mut self) -> BoundOptions<'_> {
        let settings = self.extras(|extras, _| extras.options.clone()).unwrap_or_default();
        BoundOptions { config: self, settings }
    }
    /// A path of this file as a section: `getName`, `getParent` and the like.
    pub fn section_at<'a>(&'a self, path: &'a str) -> SectionRef<'a> {
        SectionRef { root: self, path }
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getComments, org/bukkit/configuration/MemorySection.getComments
    pub fn get_comments(&self, path: &str) -> Vec<String> {
        self.extras(|extras, text| extras.comments.get_or_insert_with(|| scan_comments(text)).get(path).map_or_else(Vec::new, |row| row.0.clone()))
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getInlineComments, org/bukkit/configuration/MemorySection.getInlineComments
    pub fn get_inline_comments(&self, path: &str) -> Vec<String> {
        self.extras(|extras, text| extras.comments.get_or_insert_with(|| scan_comments(text)).get(path).map_or_else(Vec::new, |row| row.1.clone()))
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.setComments, org/bukkit/configuration/MemorySection.setComments
    ///
    /// Written by `save_commented`.
    pub fn set_comments(&self, path: &str, comments: &[String]) {
        self.extras(|extras, text| comments.clone_into(&mut extras.comments.get_or_insert_with(|| scan_comments(text)).entry(path.to_owned()).or_default().0));
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.setInlineComments, org/bukkit/configuration/MemorySection.setInlineComments
    ///
    /// Written by `save_commented`.
    pub fn set_inline_comments(&self, path: &str, comments: &[String]) {
        self.extras(|extras, text| comments.clone_into(&mut extras.comments.get_or_insert_with(|| scan_comments(text)).entry(path.to_owned()).or_default().1));
    }
    /// `save`, keeping what `options()` and the comment calls hold, and the file's own comments.
    pub fn save_commented(&self, path: Option<&Path>) -> std::io::Result<()> {
        let path = path.map(Path::to_path_buf).or_else(|| self.path.clone()).unwrap_or_default();
        let (options, comments) = self.extras(|extras, text| (extras.options.clone().unwrap_or_default(), extras.comments.get_or_insert_with(|| scan_comments(text)).clone()));
        let mut plain = Self::new();
        plain.root = self.root.clone();
        let woven = weave_comments(&plain.save_to_string(), &comments);
        plain.root = Value::Null;
        let comment = |lines: &[String]| lines.iter().map(|line| if line.is_empty() { "#\n".to_owned() } else { format!("# {line}\n") }).collect::<String>();
        let mut text = comment(options.get_header());
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(&woven);
        text.push_str(&comment(options.get_footer()));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getObject, org/bukkit/configuration/MemorySection.getObject, org/bukkit/configuration/ConfigurationSection.getSerializable, org/bukkit/configuration/MemorySection.getSerializable
    pub fn get_object<T: ConfigurationSerializable>(&self, path: &str) -> Option<T> {
        deserialize_object(self.at(path)?.as_object()?)
    }
    /// A value of a plugin's own, under `==` as Bukkit writes it.
    pub fn set_serializable<T: ConfigurationSerializable>(&mut self, path: &str, value: &T) {
        let mut map = value.serialize();
        map.insert("==".to_owned(), json!(T::alias()));
        self.set(path, Value::Object(map));
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getRichMessage
    pub fn get_rich_message(&self, path: &str) -> Option<Component> {
        self.get_string(path).map(|text| crate::adventure::mini_message(&text))
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.setRichMessage
    pub fn set_rich_message(&mut self, path: &str, message: &Component) {
        self.set(path, json!(mini_written(&message.0)));
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.getComponent
    pub fn get_component<C>(&self, path: &str, decoder: impl Fn(&str) -> C) -> Option<C> {
        self.get_string(path).map(|text| decoder(&text))
    }
    /// bukkit: org/bukkit/configuration/ConfigurationSection.setComponent
    pub fn set_component<C>(&mut self, path: &str, encoder: impl Fn(&C) -> String, value: &C) {
        self.set(path, json!(encoder(value)));
    }
}

/// bukkit: org/bukkit/configuration/ConfigurationSection.addDefault, org/bukkit/configuration/MemorySection.addDefault
///
/// `YamlConfiguration::add_default`, for what a section is a place of.
pub fn section_add_default(config: &mut crate::config::YamlConfiguration, path: &str, value: Value) {
    config.add_default(path, value);
}

// ---- Conversations with the whole of Bukkit's API ----

/// `org.bukkit.conversations.ConversationPrefix`.
pub trait ConversationPrefix: Send + Sync {
    /// bukkit: org/bukkit/conversations/ConversationPrefix.getPrefix
    fn get_prefix(&self, context: &crate::pure::ConversationContext) -> String;
}

/// `org.bukkit.conversations.NullConversationPrefix`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullConversationPrefix;

impl ConversationPrefix for NullConversationPrefix {
    /// bukkit: org/bukkit/conversations/NullConversationPrefix.getPrefix
    fn get_prefix(&self, _context: &crate::pure::ConversationContext) -> String {
        String::new()
    }
}

/// `org.bukkit.conversations.PluginNameConversationPrefix`.
#[derive(Clone, Debug)]
pub struct PluginNameConversationPrefix {
    name: String,
}

impl PluginNameConversationPrefix {
    /// bukkit: org/bukkit/conversations/PluginNameConversationPrefix.<init>
    pub fn new(plugin: &Plugin) -> Self {
        Self { name: plugin.get_name().to_owned() }
    }
}

impl ConversationPrefix for PluginNameConversationPrefix {
    /// bukkit: org/bukkit/conversations/PluginNameConversationPrefix.getPrefix
    fn get_prefix(&self, _context: &crate::pure::ConversationContext) -> String {
        format!("{} > ", self.name)
    }
}

/// `org.bukkit.conversations.ConversationCanceller`.
pub trait ConversationCanceller: Send + Sync {
    /// bukkit: org/bukkit/conversations/ConversationCanceller.setConversation
    fn set_conversation(&mut self, _player: &Player) {}
    /// bukkit: org/bukkit/conversations/ConversationCanceller.cancelBasedOnInput
    fn cancel_based_on_input(&mut self, context: &crate::pure::ConversationContext, input: &str) -> bool;
}

/// `org.bukkit.conversations.ExactMatchConversationCanceller`.
#[derive(Clone, Debug)]
pub struct ExactMatchConversationCanceller {
    escape: String,
}

impl ExactMatchConversationCanceller {
    /// bukkit: org/bukkit/conversations/ExactMatchConversationCanceller.<init>
    pub fn new(escape: &str) -> Self {
        Self { escape: escape.to_owned() }
    }
}

impl ConversationCanceller for ExactMatchConversationCanceller {
    /// bukkit: org/bukkit/conversations/ExactMatchConversationCanceller.setConversation
    fn set_conversation(&mut self, _player: &Player) {}
    fn cancel_based_on_input(&mut self, _context: &crate::pure::ConversationContext, input: &str) -> bool {
        input == self.escape
    }
}

/// `org.bukkit.conversations.InactivityConversationCanceller`: silence for so long ends it.
#[derive(Clone, Debug)]
pub struct InactivityConversationCanceller {
    seconds: u64,
}

impl InactivityConversationCanceller {
    /// bukkit: org/bukkit/conversations/InactivityConversationCanceller.<init>
    pub fn new(seconds: u64) -> Self {
        Self { seconds }
    }
}

impl ConversationCanceller for InactivityConversationCanceller {
    /// bukkit: org/bukkit/conversations/InactivityConversationCanceller.setConversation
    fn set_conversation(&mut self, player: &Player) {
        watch_silence(player, self.seconds);
    }
    /// bukkit: org/bukkit/conversations/InactivityConversationCanceller.cancelBasedOnInput
    fn cancel_based_on_input(&mut self, _context: &crate::pure::ConversationContext, _input: &str) -> bool {
        if let Some(player) = conversing_player() {
            watch_silence(&player, self.seconds);
        }
        false
    }
}

/// `org.bukkit.conversations.ManuallyAbandonedConversationCanceller`: what `abandon` counts as.
#[derive(Clone, Copy, Debug, Default)]
pub struct ManuallyAbandonedConversationCanceller;

impl ConversationCanceller for ManuallyAbandonedConversationCanceller {
    /// bukkit: org/bukkit/conversations/ManuallyAbandonedConversationCanceller.setConversation
    fn set_conversation(&mut self, _player: &Player) {}
    /// bukkit: org/bukkit/conversations/ManuallyAbandonedConversationCanceller.cancelBasedOnInput
    fn cancel_based_on_input(&mut self, _context: &crate::pure::ConversationContext, _input: &str) -> bool {
        false
    }
}

/// `org.bukkit.conversations.ConversationAbandonedEvent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConversationAbandonedEvent {
    canceller: Option<&'static str>,
}

impl ConversationAbandonedEvent {
    /// bukkit: org/bukkit/conversations/ConversationAbandonedEvent.gracefulExit
    pub fn graceful_exit(&self) -> bool {
        self.canceller.is_none()
    }
    /// bukkit: org/bukkit/conversations/ConversationAbandonedEvent.getCanceller
    ///
    /// By kind: `input`, `timeout` or `manual`.
    pub fn get_canceller(&self) -> Option<&'static str> {
        self.canceller
    }
    /// bukkit: org/bukkit/conversations/ConversationAbandonedEvent.getContext
    pub fn get_context(&self) -> Option<crate::pure::ConversationContext> {
        CLOSING.with(|context| context.borrow_mut().take())
    }
}

/// bukkit: org/bukkit/conversations/ConversationAbandonedListener, org/bukkit/conversations/ConversationAbandonedListener.conversationAbandoned
pub type ConversationAbandonedListener = std::sync::Arc<dyn Fn(&ConversationAbandonedEvent) + Send + Sync>;

/// `org.bukkit.conversations.MessagePrompt`: says something and moves on, with no input.
pub struct MessagePrompt {
    text: String,
    next: Option<Box<dyn Fn() -> Box<dyn crate::pure::Prompt> + Send + Sync>>,
}

impl MessagePrompt {
    pub fn new(text: &str, next: Option<Box<dyn Fn() -> Box<dyn crate::pure::Prompt> + Send + Sync>>) -> Self {
        Self { text: text.to_owned(), next }
    }
}

impl crate::pure::Prompt for MessagePrompt {
    fn get_prompt_text(&self, _context: &crate::pure::ConversationContext) -> String {
        self.text.clone()
    }
    /// bukkit: org/bukkit/conversations/MessagePrompt.blocksForInput
    fn blocks_for_input(&self, _context: &crate::pure::ConversationContext) -> bool {
        false
    }
    /// bukkit: org/bukkit/conversations/MessagePrompt.acceptInput
    fn accept_input(&self, _context: &mut crate::pure::ConversationContext, _input: &str) -> Option<Box<dyn crate::pure::Prompt>> {
        self.next.as_ref().map(|next| next())
    }
}

/// `org.bukkit.conversations.ValidatingPrompt`: asks again until the input is good.
pub struct ValidatingPrompt {
    text: String,
    valid: Box<dyn Fn(&crate::pure::ConversationContext, &str) -> bool + Send + Sync>,
    accepted: std::sync::Arc<dyn Fn(&mut crate::pure::ConversationContext, &str) -> Option<Box<dyn crate::pure::Prompt>> + Send + Sync>,
    failed: String,
}

impl ValidatingPrompt {
    pub fn new(text: &str, failed: &str, valid: impl Fn(&crate::pure::ConversationContext, &str) -> bool + Send + Sync + 'static, accepted: impl Fn(&mut crate::pure::ConversationContext, &str) -> Option<Box<dyn crate::pure::Prompt>> + Send + Sync + 'static) -> Self {
        Self { text: text.to_owned(), valid: Box::new(valid), accepted: std::sync::Arc::new(accepted), failed: failed.to_owned() }
    }
}

impl crate::pure::Prompt for ValidatingPrompt {
    fn get_prompt_text(&self, _context: &crate::pure::ConversationContext) -> String {
        self.text.clone()
    }
    /// bukkit: org/bukkit/conversations/ValidatingPrompt.blocksForInput
    fn blocks_for_input(&self, _context: &crate::pure::ConversationContext) -> bool {
        true
    }
    /// bukkit: org/bukkit/conversations/ValidatingPrompt.acceptInput
    fn accept_input(&self, context: &mut crate::pure::ConversationContext, input: &str) -> Option<Box<dyn crate::pure::Prompt>> {
        if (self.valid)(context, input) {
            return (self.accepted)(context, input);
        }
        if let Some(player) = conversing_player() {
            player.send_message(&self.failed);
        }
        Some(Box::new(Self { text: self.text.clone(), valid: Box::new(|_, _| false), accepted: self.accepted.clone(), failed: self.failed.clone() }))
    }
}

/// `org.bukkit.conversations.ConversationFactory`.
#[derive(Default)]
pub struct ConversationFactory {
    first: Option<Box<dyn Fn() -> Box<dyn crate::pure::Prompt> + Send + Sync>>,
    prefix: Option<Box<dyn ConversationPrefix>>,
    cancellers: Vec<Box<dyn ConversationCanceller>>,
    timeout: Option<u64>,
    escape: Option<String>,
    initial: Vec<(String, String)>,
    not_player: Option<String>,
    listeners: Vec<ConversationAbandonedListener>,
    echo: bool,
    modal: bool,
}

impl ConversationFactory {
    /// bukkit: org/bukkit/conversations/ConversationFactory.withFirstPrompt
    pub fn with_first_prompt(mut self, first: impl Fn() -> Box<dyn crate::pure::Prompt> + Send + Sync + 'static) -> Self {
        self.first = Some(Box::new(first));
        self
    }
    /// bukkit: org/bukkit/conversations/ConversationFactory.withConversationCanceller
    pub fn with_conversation_canceller(mut self, canceller: impl ConversationCanceller + 'static) -> Self {
        self.cancellers.push(Box::new(canceller));
        self
    }
    /// bukkit: org/bukkit/conversations/ConversationFactory.withTimeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Some(seconds);
        self
    }
    /// bukkit: org/bukkit/conversations/ConversationFactory.withInitialSessionData
    pub fn with_initial_session_data(mut self, data: &[(&str, &str)]) -> Self {
        self.initial = data.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned())).collect();
        self
    }
    /// bukkit: org/bukkit/conversations/ConversationFactory.thatExcludesNonPlayersWithMessage
    pub fn that_excludes_non_players_with_message(mut self, message: &str) -> Self {
        self.not_player = Some(message.to_owned());
        self
    }
    /// bukkit: org/bukkit/conversations/ConversationFactory.addConversationAbandonedListener
    pub fn add_conversation_abandoned_listener(mut self, listener: impl Fn(&ConversationAbandonedEvent) + Send + Sync + 'static) -> Self {
        self.listeners.push(std::sync::Arc::new(listener));
        self
    }
    /// With Bukkit's `withEscapeSequence`.
    pub fn escape_on(mut self, text: &str) -> Self {
        self.escape = Some(text.to_owned());
        self
    }
    /// With Bukkit's `withPrefix`.
    pub fn with_conversation_prefix(mut self, prefix: impl ConversationPrefix + 'static) -> Self {
        self.prefix = Some(Box::new(prefix));
        self
    }
    /// With Bukkit's `withLocalEcho`.
    pub fn local_echo(mut self, on: bool) -> Self {
        self.echo = on;
        self
    }
    /// With Bukkit's `withModality`.
    pub fn modal(mut self, on: bool) -> Self {
        self.modal = on;
        self
    }
    /// bukkit: org/bukkit/conversations/ConversationFactory.buildConversation
    ///
    /// For a console, what `that_excludes_non_players_with_message` set is said instead: `None`.
    pub fn build_conversation(mut self, whom: &CommandSender) -> Option<FullConversation> {
        let Some(player) = whom.as_player().cloned() else {
            if let Some(message) = self.not_player.take() {
                whom.send_message(&message);
            }
            return None;
        };
        let mut context = crate::pure::ConversationContext::default();
        for (key, value) in &self.initial {
            context.set_session_data(key, value);
        }
        if let Some(escape) = &self.escape {
            self.cancellers.push(Box::new(ExactMatchConversationCanceller::new(escape)));
        }
        if let Some(seconds) = self.timeout {
            self.cancellers.push(Box::new(InactivityConversationCanceller::new(seconds)));
        }
        Some(FullConversation { player, prompt: self.first.take().map(|first| first()), context, prefix: self.prefix.take(), cancellers: std::mem::take(&mut self.cancellers), listeners: std::mem::take(&mut self.listeners), echo: self.echo, state: "UNSTARTED", modal: self.modal })
    }
}

/// `org.bukkit.conversations.Conversation`, with prefix objects, cancellers and listeners.
pub struct FullConversation {
    player: Player,
    prompt: Option<Box<dyn crate::pure::Prompt>>,
    context: crate::pure::ConversationContext,
    prefix: Option<Box<dyn ConversationPrefix>>,
    cancellers: Vec<Box<dyn ConversationCanceller>>,
    listeners: Vec<ConversationAbandonedListener>,
    echo: bool,
    state: &'static str,
    modal: bool,
}

static FULL_TALKS: Mutex<BTreeMap<String, FullConversation>> = Mutex::new(BTreeMap::new());
static FULL_LISTENING: std::sync::OnceLock<()> = std::sync::OnceLock::new();
static SILENCE: Mutex<BTreeMap<String, u32>> = Mutex::new(BTreeMap::new());

thread_local! {
    static CONVERSING: std::cell::RefCell<Option<Player>> = const { std::cell::RefCell::new(None) };
    static CLOSING: std::cell::RefCell<Option<crate::pure::ConversationContext>> = const { std::cell::RefCell::new(None) };
}

fn conversing_player() -> Option<Player> {
    CONVERSING.with(|player| player.borrow().clone())
}

fn with_conversing<R>(player: &Player, run: impl FnOnce() -> R) -> R {
    CONVERSING.with(|current| *current.borrow_mut() = Some(player.clone()));
    let answer = run();
    CONVERSING.with(|current| *current.borrow_mut() = None);
    answer
}

fn watch_silence(player: &Player, seconds: u64) {
    let uuid = player.uuid().to_owned();
    let stamp = {
        let mut all = SILENCE.lock().unwrap_or_else(|e| e.into_inner());
        let stamp = all.entry(uuid.clone()).or_insert(0);
        *stamp += 1;
        *stamp
    };
    crate::scheduler::run_task_later(seconds * 20, move || {
        if SILENCE.lock().unwrap_or_else(|e| e.into_inner()).get(&uuid) == Some(&stamp) {
            close_conversation(&uuid, Some("timeout"));
        }
    });
}

/// Ends a conversation and tells its listeners.
fn close_conversation(uuid: &str, canceller: Option<&'static str>) {
    let talk = FULL_TALKS.lock().unwrap_or_else(|e| e.into_inner()).remove(uuid);
    if let Some(mut talk) = talk {
        talk.state = "ABANDONED";
        let event = ConversationAbandonedEvent { canceller };
        let listeners = std::mem::take(&mut talk.listeners);
        CLOSING.with(|closing| *closing.borrow_mut() = Some(std::mem::take(&mut talk.context)));
        for listener in &listeners {
            listener(&event);
        }
    }
}

impl FullConversation {
    /// bukkit: org/bukkit/conversations/Conversation.getForWhom, org/bukkit/conversations/ConversationContext.getForWhom
    pub fn get_for_whom(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/conversations/Conversation.getContext
    pub fn get_context(&self) -> &crate::pure::ConversationContext {
        &self.context
    }
    /// bukkit: org/bukkit/conversations/Conversation.getState
    pub fn get_state(&self) -> &'static str {
        self.state
    }
    /// bukkit: org/bukkit/conversations/Conversation.isLocalEchoEnabled
    pub fn is_local_echo_enabled(&self) -> bool {
        self.echo
    }
    /// bukkit: org/bukkit/conversations/Conversation.setLocalEchoEnabled
    pub fn set_local_echo_enabled(&mut self, on: bool) {
        self.echo = on;
    }
    /// bukkit: org/bukkit/conversations/Conversation.getPrefix
    pub fn get_prefix(&self) -> String {
        self.prefix.as_ref().map_or_else(String::new, |prefix| prefix.get_prefix(&self.context))
    }
    /// bukkit: org/bukkit/conversations/Conversation.getCancellers
    pub fn get_cancellers(&self) -> &[Box<dyn ConversationCanceller>] {
        &self.cancellers
    }
    /// bukkit: org/bukkit/conversations/Conversation.addConversationAbandonedListener
    pub fn add_conversation_abandoned_listener(&mut self, listener: impl Fn(&ConversationAbandonedEvent) + Send + Sync + 'static) {
        self.listeners.push(std::sync::Arc::new(listener));
    }
    /// bukkit: org/bukkit/conversations/Conversation.removeConversationAbandonedListener
    ///
    /// By its place in the list, in the order they were added.
    pub fn remove_conversation_abandoned_listener(&mut self, index: usize) {
        if index < self.listeners.len() {
            self.listeners.remove(index);
        }
    }
    /// bukkit: org/bukkit/conversations/Conversation.outputNextPrompt
    ///
    /// Says the prompt, and goes on by itself past any that don't wait for input.
    pub fn output_next_prompt(&mut self) -> bool {
        let player = self.player.clone();
        with_conversing(&player, || loop {
            let Some(prompt) = self.prompt.as_ref() else {
                return false;
            };
            player.send_message(&format!("{}{}", self.prefix.as_ref().map_or_else(String::new, |prefix| prefix.get_prefix(&self.context)), prompt.get_prompt_text(&self.context)));
            if prompt.blocks_for_input(&self.context) {
                return true;
            }
            let next = prompt.accept_input(&mut self.context, "");
            self.prompt = next;
        })
    }
    /// bukkit: org/bukkit/conversations/Conversation.acceptInput, org/bukkit/conversations/Conversable.acceptConversationInput
    ///
    /// False once it's over.
    pub fn accept_input(&mut self, input: &str) -> bool {
        let player = self.player.clone();
        let cancelled = with_conversing(&player, || self.cancellers.iter_mut().any(|canceller| canceller.cancel_based_on_input(&self.context, input)));
        if cancelled {
            self.state = "ABANDONED";
            return false;
        }
        if self.echo {
            player.send_message(input);
        }
        let Some(prompt) = self.prompt.take() else {
            return false;
        };
        let next = with_conversing(&player, || prompt.accept_input(&mut self.context, input));
        self.prompt = next;
        self.output_next_prompt()
    }
    /// bukkit: org/bukkit/conversations/Conversation.begin, org/bukkit/conversations/ConversationFactory$NotPlayerMessagePrompt.getPromptText
    ///
    /// What the player types next goes to the prompt, not to chat.
    pub fn begin(mut self, plugin: &Plugin) {
        self.state = "STARTED";
        let player = self.player.clone();
        for canceller in &mut self.cancellers {
            canceller.set_conversation(&player);
        }
        if !self.output_next_prompt() {
            close_conversation(player.uuid(), None);
            return;
        }
        FULL_TALKS.lock().unwrap_or_else(|e| e.into_inner()).insert(player.uuid().to_owned(), self);
        FULL_LISTENING.get_or_init(|| {
            plugin.host.on("allow_chat", |value| {
                let (Some(uuid), Some(message)) = (value["uuid"].as_str(), value["message"].as_str()) else {
                    return horizon_mod::Answer::Pass;
                };
                let taken = FULL_TALKS.lock().unwrap_or_else(|e| e.into_inner()).remove(uuid);
                let Some(mut talk) = taken else {
                    return horizon_mod::Answer::Pass;
                };
                let going = talk.accept_input(message);
                let manual = talk.state == "ABANDONED";
                FULL_TALKS.lock().unwrap_or_else(|e| e.into_inner()).insert(uuid.to_owned(), talk);
                if !going {
                    close_conversation(uuid, manual.then_some("input"));
                }
                horizon_mod::Answer::Fail
            });
        });
    }
}

/// bukkit: org/bukkit/conversations/Conversation.abandon
pub fn abandon_conversation_of(player: &Player) {
    close_conversation(player.uuid(), Some("manual"));
}

impl crate::pure::ConversationContext {
    /// bukkit: org/bukkit/conversations/ConversationContext.getPlugin
    pub fn get_plugin<'a>(&self, plugin: &'a Plugin) -> &'a Plugin {
        plugin
    }
    /// A `Conversable` inside a prompt of a `FullConversation`: Java's `getForWhom`.
    pub fn for_whom(&self) -> Option<Player> {
        conversing_player()
    }
}

// ---- Aliases from `commands.yml` ----

/// `org.bukkit.command.MultipleCommandAlias`: one word for several commands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipleCommandAlias {
    commands: Vec<String>,
}

impl MultipleCommandAlias {
    pub fn new(commands: &[&str]) -> Self {
        Self { commands: commands.iter().map(|line| (*line).to_owned()).collect() }
    }
    /// bukkit: org/bukkit/command/MultipleCommandAlias.getCommands
    pub fn get_commands(&self) -> &[String] {
        &self.commands
    }
    /// bukkit: org/bukkit/command/MultipleCommandAlias.execute
    pub fn execute(&self, sender: &CommandSender, args: &[&str]) -> bool {
        self.commands.iter().fold(false, |any, line| CommandMap.dispatch(sender, &format!("{line} {}", args.join(" "))) || any)
    }
}

/// `org.bukkit.command.FormattedCommandAlias`: `$1` and `$$1` in its lines, as `commands.yml` has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormattedCommandAlias {
    alias: String,
    formats: Vec<String>,
}

impl FormattedCommandAlias {
    pub fn new(alias: &str, formats: &[&str]) -> Self {
        Self { alias: alias.to_owned(), formats: formats.iter().map(|line| (*line).to_owned()).collect() }
    }
    fn build(format: &str, args: &[&str]) -> Option<String> {
        let mut out = String::new();
        let mut chars = format.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch != '$' {
                out.push(ch);
                continue;
            }
            let required = chars.next_if_eq(&'$').is_some();
            let mut digits = String::new();
            while let Some(digit) = chars.next_if(char::is_ascii_digit) {
                digits.push(digit);
            }
            let rest = chars.next_if_eq(&'-').is_some();
            let Ok(index) = digits.parse::<usize>() else {
                out.push('$');
                continue;
            };
            let found = args.get(index.saturating_sub(1));
            match found {
                Some(_) if rest => out.push_str(&args[index - 1..].join(" ")),
                Some(word) => out.push_str(word),
                None if required => return None,
                None => {}
            }
        }
        Some(out)
    }
    /// bukkit: org/bukkit/command/FormattedCommandAlias.execute
    pub fn execute(&self, sender: &CommandSender, args: &[&str]) -> bool {
        let mut lines = Vec::new();
        for format in &self.formats {
            match Self::build(format, args) {
                Some(line) => lines.push(line),
                None => return false,
            }
        }
        lines.iter().fold(false, |any, line| CommandMap.dispatch(sender, line) || any)
    }
    /// bukkit: org/bukkit/command/FormattedCommandAlias.getTimingName
    pub fn get_timing_name(&self) -> &str {
        &self.alias
    }
}

// ---- Skins ----

impl crate::player::PlayerProfile {
    /// bukkit: org/bukkit/profile/PlayerTextures.getSkinModel
    pub fn get_skin_model(&self) -> &'static str {
        if self.textures()["textures"]["SKIN"]["metadata"]["model"].as_str() == Some("slim") { "SLIM" } else { "CLASSIC" }
    }
    /// bukkit: org/bukkit/profile/PlayerTextures.getTimestamp
    pub fn get_timestamp(&self) -> i64 {
        self.textures()["timestamp"].as_i64().unwrap_or(0)
    }
}

// ---- Noise ----

const SIMPLEX_GRADIENTS: [[f64; 3]; 12] = [[1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0], [0.0, 1.0, 1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, -1.0, -1.0]];

/// `org.bukkit.util.noise.SimplexNoiseGenerator`: over Perlin's table, as Bukkit's unseeded one.
#[derive(Clone, Copy, Debug, Default)]
pub struct SimplexNoiseGenerator;

impl SimplexNoiseGenerator {
    /// bukkit: org/bukkit/util/noise/SimplexNoiseGenerator.getInstance
    pub fn get_instance() -> Self {
        Self
    }
    fn hash(i: i32) -> usize {
        usize::from(crate::pure::PERMUTATION[(i & 255) as usize])
    }
    /// bukkit: org/bukkit/util/noise/SimplexNoiseGenerator.getNoise, org/bukkit/util/noise/SimplexNoiseGenerator.noise
    ///
    /// In three dimensions, -1 to 1; `y` and `z` of 0 for fewer.
    pub fn noise(x: f64, y: f64, z: f64) -> f64 {
        const F3: f64 = 1.0 / 3.0;
        const G3: f64 = 1.0 / 6.0;
        let skew = (x + y + z) * F3;
        let (i, j, k) = ((x + skew).floor() as i32, (y + skew).floor() as i32, (z + skew).floor() as i32);
        let unskew = f64::from(i + j + k) * G3;
        let first = [x - (f64::from(i) - unskew), y - (f64::from(j) - unskew), z - (f64::from(k) - unskew)];
        let (i1, j1, k1, i2, j2, k2) = if first[0] >= first[1] {
            if first[1] >= first[2] {
                (1, 0, 0, 1, 1, 0)
            } else if first[0] >= first[2] {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if first[1] < first[2] {
            (0, 0, 1, 0, 1, 1)
        } else if first[0] < first[2] {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };
        let corners = [(0, 0, 0, 0.0), (i1, j1, k1, G3), (i2, j2, k2, 2.0 * G3), (1, 1, 1, 3.0 * G3)];
        let mut total = 0.0;
        for (a, b, c, g) in corners {
            let at = [first[0] - f64::from(a) + g, first[1] - f64::from(b) + g, first[2] - f64::from(c) + g];
            let fall = 0.6 - at[0] * at[0] - at[1] * at[1] - at[2] * at[2];
            if fall > 0.0 {
                let hash = Self::hash(i + a + Self::hash(j + b + Self::hash(k + c) as i32) as i32) % 12;
                let gradient = SIMPLEX_GRADIENTS[hash];
                total += fall * fall * fall * fall * (gradient[0] * at[0] + gradient[1] * at[1] + gradient[2] * at[2]);
            }
        }
        32.0 * total
    }
}

/// `org.bukkit.util.noise.SimplexOctaveGenerator`.
#[derive(Clone, Copy, Debug)]
pub struct SimplexOctaveGenerator {
    octaves: i32,
    x_scale: f64,
    y_scale: f64,
    z_scale: f64,
    w_scale: f64,
}

impl SimplexOctaveGenerator {
    pub fn new(octaves: i32) -> Self {
        Self { octaves, x_scale: 1.0, y_scale: 1.0, z_scale: 1.0, w_scale: 1.0 }
    }
    /// bukkit: org/bukkit/util/noise/SimplexOctaveGenerator.setScale
    pub fn set_scale(&mut self, scale: f64) {
        self.x_scale = scale;
        self.y_scale = scale;
        self.z_scale = scale;
        self.w_scale = scale;
    }
    /// bukkit: org/bukkit/util/noise/SimplexOctaveGenerator.getWScale
    pub fn get_w_scale(&self) -> f64 {
        self.w_scale
    }
    /// bukkit: org/bukkit/util/noise/SimplexOctaveGenerator.setWScale
    pub fn set_w_scale(&mut self, scale: f64) {
        self.w_scale = scale;
    }
    /// bukkit: org/bukkit/util/noise/SimplexOctaveGenerator.noise
    pub fn noise(&self, x: f64, y: f64, z: f64, frequency: f64, amplitude: f64) -> f64 {
        let (mut result, mut wave, mut height) = (0.0, 1.0, 1.0);
        for _ in 0..self.octaves {
            result += SimplexNoiseGenerator::noise(x * self.x_scale * wave, y * self.y_scale * wave, z * self.z_scale * wave) * height;
            wave *= frequency;
            height *= amplitude;
        }
        result
    }
}

// ---- The last of the small ones ----

/// A modifier, with the slot group the item gave it.
pub(crate) fn slotted(slot: Option<&str>, mut modifier: crate::scoreboard::AttributeModifier) -> crate::scoreboard::AttributeModifier {
    if let Some(slot) = slot {
        slot.clone_into(&mut modifier.slot);
    }
    modifier
}

impl crate::scoreboard::AttributeModifier {
    /// bukkit: org/bukkit/attribute/AttributeModifier.getSlotGroup
    ///
    /// The game's name for it: `any`, `mainhand`, `armor`.
    pub fn get_slot_group(&self) -> &str {
        &self.slot
    }
    /// bukkit: org/bukkit/attribute/AttributeModifier.getSlot
    ///
    /// The one `EquipmentSlot`, for a group of one; `None` for `any` and the wider ones, as Java's.
    pub fn get_slot(&self) -> Option<&'static str> {
        Some(match self.slot.as_str() {
            "mainhand" => "HAND",
            "offhand" => "OFF_HAND",
            "feet" => "FEET",
            "legs" => "LEGS",
            "chest" => "CHEST",
            "head" => "HEAD",
            "body" => "BODY",
            "saddle" => "SADDLE",
            _ => return None,
        })
    }
}

/// bukkit: org/bukkit/plugin/ServicesManager.getRegistrations
pub fn get_registrations(plugin: &Plugin) -> Vec<crate::persistence::ServiceProvider> {
    crate::persistence::plugin_services(plugin.get_name())
}

/// bukkit: org/bukkit/plugin/EventExecutor, org/bukkit/plugin/EventExecutor.execute
///
/// A listener here is its closure, called with the event.
pub type EventExecutor<E> = Box<dyn Fn(&mut E) + Send + Sync>;

impl CommandSender {
    /// bukkit: org/bukkit/command/BlockCommandSender.getBlock
    ///
    /// The command block a command came from, inside that command.
    pub fn get_block(&self) -> Option<Block> {
        if !matches!(self, Self::Console) {
            return None;
        }
        let found = query(json!({ "op": "core_command_block" }));
        Some(World::of(found["dimension"].as_str()?).get_block_at(int(&found["pos"][0]), int(&found["pos"][1]), int(&found["pos"][2])))
    }
}

/// bukkit: org/bukkit/command/MessageCommandSender.sendMessage
///
/// Each line, as Java's default does.
pub fn send_messages(sender: &mut BufferedCommandSender, lines: &[&str]) {
    lines.iter().for_each(|line| sender.send_message(line));
}

/// bukkit: org/bukkit/Bukkit.reloadPermissions, org/bukkit/Server.reloadPermissions
///
/// Reads `permissions.yml`, as Bukkit does, into what this plugin's `has_permission` goes by.
pub fn reload_permissions() {
    let config = crate::config::YamlConfiguration::load_configuration(&server_folder().join("permissions.yml"));
    for permission in crate::config2::Permission::load_permissions(&config.root) {
        register_permission(&permission);
    }
}

/// bukkit: org/bukkit/help/IndexHelpTopic.getFullText
///
/// What a list of topics shows `sender`: one line for each it may see.
pub fn index_full_text(sender: &CommandSender, topics: &[crate::core2::HelpTopic]) -> String {
    topics.iter().filter(|topic| topic.can_see(sender)).map(|topic| format!("{}{}: {}{}", crate::chat_color::GOLD, topic.get_name(), crate::chat_color::WHITE, topic.get_short_text())).collect::<Vec<_>>().join("\n")
}

impl crate::player::PlayerProfile {
    /// bukkit: org/bukkit/profile/PlayerTextures.isSigned
    pub fn is_signed(&self) -> bool {
        self.properties.iter().any(|(name, _, signature)| name == "textures" && signature.is_some())
    }
    /// bukkit: org/bukkit/profile/PlayerTextures.clear
    pub fn clear_textures(&mut self) {
        self.properties.retain(|(name, ..)| name != "textures");
    }
}

/// bukkit: org/bukkit/persistence/ListPersistentDataType, org/bukkit/persistence/ListPersistentDataType.elementType
///
/// A list of `T`, each kept as `T`'s type keeps it.
#[derive(Clone, Copy, Debug)]
pub struct ListPersistentDataType<T>(pub T);

impl<T> ListPersistentDataType<T> {
    /// bukkit: org/bukkit/persistence/ListPersistentDataType.elementType
    pub fn element_type(&self) -> &T {
        &self.0
    }
}

impl<T: PersistentDataType> PersistentDataType for ListPersistentDataType<T> {
    type Complex = Vec<T::Complex>;
    fn get_primitive_type(&self) -> &'static str {
        "List"
    }
    fn get_complex_type(&self) -> &'static str {
        "List"
    }
    fn to_primitive(&self, complex: &Self::Complex, context: &PersistentDataAdapterContext) -> Value {
        Value::Array(complex.iter().map(|one| self.0.to_primitive(one, context)).collect())
    }
    fn from_primitive(&self, primitive: &Value, context: &PersistentDataAdapterContext) -> Option<Self::Complex> {
        primitive.as_array()?.iter().map(|one| self.0.from_primitive(one, context)).collect()
    }
    fn nbt_kind(&self, primitive: &Value) -> Value {
        Value::Array(primitive.as_array().into_iter().flatten().map(|one| self.0.nbt_kind(one)).collect())
    }
}

impl crate::pure::OctaveGenerator {
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.getOctaves
    ///
    /// How many: each is the one Perlin generator, at its own scale.
    pub fn get_octaves(&self) -> i32 {
        self.octaves
    }
}

/// bukkit: org/bukkit/FireworkEffect.deserialize
pub fn firework_effect_deserialize(map: &Value) -> Option<crate::meta_lists::FireworkEffect> {
    let colors = |key: &str| -> Vec<Color> { map[key].as_array().into_iter().flatten().filter_map(Color::deserialize).collect() };
    let flag = |key: &str| map[key].as_bool().or_else(|| map[key].as_i64().map(|value| value != 0)).unwrap_or(false);
    Some(crate::meta_lists::FireworkEffect::new(map["type"].as_str()?, &colors("colors"), &colors("fade-colors"), flag("flicker"), flag("trail")))
}

impl crate::core2::UnsafeValues {
    /// bukkit: org/bukkit/UnsafeValues.loadAdvancements
    pub fn load_advancements(&self, advancements: &[(NamespacedKey, String)]) -> Vec<NamespacedKey> {
        advancements.iter().filter(|(key, text)| self.load_advancement(key, text).is_some()).map(|(key, _)| key.clone()).collect()
    }
}

impl crate::registry::Enchantment {
    /// bukkit: org/bukkit/enchantments/Enchantment.getDamageIncrease
    ///
    /// `category` is an `EntityCategory` name. From the enchantment's own `minecraft:damage` effects.
    pub fn get_damage_increase(&self, level: i32, category: &str) -> f32 {
        let wanted = match category {
            "UNDEAD" => "sensitive_to_smite",
            "ARTHROPOD" => "sensitive_to_bane_of_arthropods",
            "WATER" | "AQUATIC" => "sensitive_to_impaling",
            _ => "",
        };
        let data = self.data();
        let mut total = 0.0;
        for row in data["effects"]["minecraft:damage"].as_array().into_iter().flatten() {
            let needs = row["requirements"]["predicate"]["type"].as_str().or_else(|| row["requirements"]["terms"][0]["predicate"]["type"].as_str());
            if needs.is_some_and(|tag| wanted.is_empty() || !tag.ends_with(wanted)) {
                continue;
            }
            let amount = &row["effect"]["value"];
            total += match (amount.as_f64(), amount["base"].as_f64()) {
                (Some(plain), _) => plain,
                (None, Some(base)) => base + amount["per_level_above_first"].as_f64().unwrap_or(0.0) * f64::from(level - 1),
                _ => 0.0,
            };
        }
        total as f32
    }
}

/// `org.bukkit.GameEvent`: the game's event by its key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameEvent {
    key: String,
}

impl GameEvent {
    /// bukkit: org/bukkit/GameEvent.getByKey
    pub fn get_by_key(key: &str) -> Option<Self> {
        let key = key_of(key);
        let name = key.trim_start_matches("minecraft:").to_ascii_uppercase();
        crate::enums::game_event::value_of(&name).map(|_| Self { key })
    }
    fn facts(&self) -> Value {
        query(json!({ "op": "core_game_event", "name": self.key }))
    }
    /// bukkit: org/bukkit/GameEvent.getRange
    pub fn get_range(&self) -> i32 {
        int(&self.facts()["range"])
    }
    /// bukkit: org/bukkit/GameEvent.getVibrationLevel
    pub fn get_vibration_level(&self) -> i32 {
        int(&self.facts()["frequency"])
    }
}

impl World {
    /// bukkit: org/bukkit/World.sendGameEvent
    ///
    /// `source` is an entity, with no one to answer for it if `None`. A player's goes by `send_game_event_as`.
    pub fn send_game_event(&self, source: Option<&Entity>, event: &GameEvent, position: &Vector) -> bool {
        self.ask("core_game_event_send", json!({ "name": event.key, "pos": [position.x, position.y, position.z], "source": source.map(Entity::get_entity_id) })).as_bool().unwrap_or(false)
    }
    /// `send_game_event` with a player as the source, as `Entity` is never one here.
    pub fn send_game_event_as(&self, source: &Player, event: &GameEvent, position: &Vector) -> bool {
        self.ask("core_game_event_send", json!({ "name": event.key, "pos": [position.x, position.y, position.z], "player": source.uuid().replace('-', "") })).as_bool().unwrap_or(false)
    }
}

// ---- Who an attachment was, once it goes ----

static REMOVAL_CALLBACKS: Mutex<BTreeMap<(String, String), PermissionRemovedExecutor>> = Mutex::new(BTreeMap::new());

fn callback_key(attachment: &crate::config2::PermissionAttachment) -> (String, String) {
    (attachment.get_permissible().uuid().to_owned(), attachment.plugin.clone())
}

impl crate::config2::PermissionAttachment {
    /// bukkit: org/bukkit/permissions/PermissionAttachment.setRemovalCallback
    pub fn set_removal_callback(&self, callback: impl Fn(&Self) + Send + Sync + 'static) {
        REMOVAL_CALLBACKS.lock().unwrap_or_else(|e| e.into_inner()).insert(callback_key(self), std::sync::Arc::new(callback));
    }
    /// bukkit: org/bukkit/permissions/PermissionAttachment.getRemovalCallback
    pub fn get_removal_callback(&self) -> Option<PermissionRemovedExecutor> {
        REMOVAL_CALLBACKS.lock().unwrap_or_else(|e| e.into_inner()).get(&callback_key(self)).cloned()
    }
}

pub(crate) fn attachment_removed(attachment: &crate::config2::PermissionAttachment) {
    let callback = REMOVAL_CALLBACKS.lock().unwrap_or_else(|e| e.into_inner()).remove(&callback_key(attachment));
    if let Some(callback) = callback {
        callback(attachment);
    }
}

impl PermissionAttachmentInfo {
    /// bukkit: org/bukkit/permissions/PermissionAttachmentInfo.getAttachment
    ///
    /// None for what comes of a default.
    pub fn get_attachment(&self) -> Option<crate::config2::PermissionAttachment> {
        crate::config2::attachment_granting(&self.player, &self.permission)
    }
}

// ---- Bukkit's own commands ----

/// bukkit: org/bukkit/command/defaults/VersionCommand.execute
pub fn version_command(sender: &CommandSender) -> bool {
    sender.send_message(&get_version_message());
    true
}

/// bukkit: org/bukkit/command/defaults/PluginsCommand.tabComplete
pub fn no_completions() -> Vec<String> {
    Vec::new()
}

/// bukkit: org/bukkit/command/defaults/ReloadCommand.tabComplete
pub fn reload_completions() -> Vec<String> {
    vec!["permissions".to_owned(), "commands".to_owned()]
}

/// bukkit: org/bukkit/command/defaults/VersionCommand.tabComplete
///
/// The plugins whose names start with the one word it takes.
pub fn version_completions(args: &[&str]) -> Vec<String> {
    let [word] = args else {
        return Vec::new();
    };
    PluginManager.get_plugins().iter().map(|plugin| plugin.get_name()).filter(|name| crate::extra::starts_with_ignore_case(name, word)).collect()
}

/// bukkit: org/bukkit/command/defaults/PluginsCommand.execute
pub fn plugins_command(sender: &CommandSender) -> bool {
    let plugins = PluginManager.get_plugins();
    let names: Vec<String> = plugins.iter().map(|plugin| format!("{}{}", if plugin.is_enabled() { crate::chat_color::GREEN } else { crate::chat_color::RED }, plugin.get_name())).collect();
    sender.send_message(&format!("{}Plugins ({}): {}", crate::chat_color::WHITE, plugins.len(), names.join(&format!("{}, ", crate::chat_color::WHITE))));
    true
}

/// bukkit: org/bukkit/command/defaults/ReloadCommand.execute
pub fn reload_command(sender: &CommandSender) -> bool {
    sender.send_message(&format!("{}Please note that this command is not supported and may cause issues.", crate::chat_color::RED));
    reload();
    sender.send_message(&format!("{}Reload complete.", crate::chat_color::GREEN));
    true
}

/// bukkit: org/bukkit/command/defaults/HelpCommand.execute
///
/// The topic named, or the list.
pub fn help_command(sender: &CommandSender, args: &[&str]) -> bool {
    let topics = crate::core2::get_help_topics();
    match args.first().and_then(|name| crate::core2::get_help_topic(name)) {
        Some(topic) if topic.can_see(sender) => sender.send_message(topic.get_full_text()),
        Some(_) => sender.send_message(&get_permission_message()),
        None => sender.send_message(&index_full_text(sender, &topics)),
    }
    true
}

/// bukkit: org/bukkit/command/defaults/HelpCommand.tabComplete
pub fn help_tab_complete(sender: &CommandSender, args: &[&str]) -> Vec<String> {
    let start = args.last().copied().unwrap_or_default().to_ascii_lowercase();
    crate::core2::get_help_topics().iter().filter(|topic| topic.can_see(sender)).map(|topic| topic.get_name().to_owned()).filter(|name| name.to_ascii_lowercase().starts_with(&start)).collect()
}

// ---- A spawner's entity ----

impl crate::pure::Spawner {
    /// bukkit: org/bukkit/spawner/BaseSpawner.getSpawnedEntity
    pub fn get_spawned_entity(&self) -> Option<crate::entities3::EntitySnapshot> {
        let entity = self.0.get_state_data()["SpawnData"]["entity"].clone();
        (!entity.is_null()).then(|| crate::entities3::create_entity_snapshot(&crate::data::snbt(&entity)))
    }
    /// bukkit: org/bukkit/spawner/BaseSpawner.setSpawnedEntity
    pub fn set_spawned_entity(&self, entity: &crate::entities3::EntitySnapshot) -> bool {
        self.0.merge_state(&format!("{{SpawnData:{{entity:{}}}}}", entity.get_as_string()))
    }
}

// ---- Materials ----

/// The bits of the process's own random hash keys: what a fresh id here is drawn from.
fn fresh_bits() -> u128 {
    use std::hash::{BuildHasher, Hasher};
    let word = || std::collections::hash_map::RandomState::new().build_hasher().finish();
    (u128::from(word()) << 64) | u128::from(word())
}

impl Material {
    /// bukkit: org/bukkit/Material.getMaterial
    ///
    /// By the constant's exact name; `None` for a name that isn't one, as Java's `null`.
    pub fn get_material(name: &str) -> Option<Self> {
        crate::enums::material::value_of(name).filter(|known| !known.starts_with("LEGACY_")).map(Self::of)
    }
    /// bukkit: org/bukkit/Material.isLegacy
    pub fn is_legacy(&self) -> bool {
        self.key().trim_start_matches("minecraft:").starts_with("legacy_")
    }
    /// bukkit: org/bukkit/Material.getId
    ///
    /// `None` where Java throws, which is every modern material: a number here means nothing.
    pub fn get_id(&self) -> Option<i32> {
        None
    }
    /// bukkit: org/bukkit/Material.isInteractable
    pub fn is_interactable(&self) -> bool {
        INTERACTABLE.binary_search(&self.key().trim_start_matches("minecraft:")).is_ok()
    }
    /// bukkit: org/bukkit/Material.getEquipmentSlot
    ///
    /// Where the item goes, by its own `equippable` component; else the hand.
    pub fn get_equipment_slot(&self) -> &'static str {
        let data = query(json!({ "op": "core_item_default", "id": self.key(), "component": "minecraft:equippable" }));
        match data["slot"].as_str() {
            Some("head") => "HEAD",
            Some("chest") => "CHEST",
            Some("legs") => "LEGS",
            Some("feet") => "FEET",
            Some("body") => "BODY",
            Some("saddle") => "SADDLE",
            Some("offhand") => "OFF_HAND",
            _ if self.key() == "minecraft:shield" => "OFF_HAND",
            _ => "HAND",
        }
    }
    /// bukkit: org/bukkit/Material.getDefaultAttributeModifiers, org/bukkit/Material.getItemAttributes
    ///
    /// Attribute keys with their modifiers; `slot` is the game's group name, or all of them.
    pub fn get_default_attribute_modifiers(&self, slot: Option<&str>) -> Vec<(String, crate::scoreboard::AttributeModifier)> {
        crate::fam_items::ItemType::of(self.key()).get_default_attribute_modifiers(slot)
    }
}

// ---- Chat colours ----

const CHAT_COLORS: [(&str, char); 22] = [("BLACK", '0'), ("DARK_BLUE", '1'), ("DARK_GREEN", '2'), ("DARK_AQUA", '3'), ("DARK_RED", '4'), ("DARK_PURPLE", '5'), ("GOLD", '6'), ("GRAY", '7'), ("DARK_GRAY", '8'), ("BLUE", '9'), ("GREEN", 'a'), ("AQUA", 'b'), ("RED", 'c'), ("LIGHT_PURPLE", 'd'), ("YELLOW", 'e'), ("WHITE", 'f'), ("MAGIC", 'k'), ("BOLD", 'l'), ("STRIKETHROUGH", 'm'), ("UNDERLINE", 'n'), ("ITALIC", 'o'), ("RESET", 'r')];

/// bukkit: org/bukkit/ChatColor.getByChar
///
/// The constant's name for a code letter.
pub fn chat_color_by_char(code: char) -> Option<&'static str> {
    CHAT_COLORS.iter().find(|(_, known)| *known == code).map(|(name, _)| *name)
}

/// bukkit: org/bukkit/ChatColor.getChar
///
/// The code letter of a constant, by its name.
pub fn chat_color_char(name: &str) -> Option<char> {
    CHAT_COLORS.iter().find(|(known, _)| *known == name).map(|(_, code)| *code)
}

/// bukkit: org/bukkit/ChatColor.isColor
pub fn chat_color_is_color(name: &str) -> bool {
    chat_color_char(name).is_some_and(|code| code != 'r' && !chat_color_code_is_format(code))
}

/// bukkit: org/bukkit/ChatColor.isFormat
pub fn chat_color_is_format(name: &str) -> bool {
    chat_color_char(name).is_some_and(chat_color_code_is_format)
}

fn chat_color_code_is_format(code: char) -> bool {
    matches!(code, 'k' | 'l' | 'm' | 'n' | 'o')
}

/// bukkit: org/bukkit/ChatColor.getLastColors
///
/// The codes in force at the end of `text`: a colour and the formats after it, as Java keeps them.
pub fn get_last_colors(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    for index in (0..chars.len().saturating_sub(1)).rev() {
        if chars[index] != '\u{a7}' {
            continue;
        }
        let code = chars[index + 1].to_ascii_lowercase();
        if chat_color_by_char(code).is_none() {
            continue;
        }
        out.insert_str(0, &format!("\u{a7}{code}"));
        if code == 'r' || !chat_color_code_is_format(code) {
            break;
        }
    }
    out
}

// ---- Locations ----

impl Location {
    /// bukkit: org/bukkit/Location.x
    pub fn x(&self) -> f64 {
        self.x
    }
    /// bukkit: org/bukkit/Location.y
    pub fn y(&self) -> f64 {
        self.y
    }
    /// bukkit: org/bukkit/Location.z
    pub fn z(&self) -> f64 {
        self.z
    }
    /// bukkit: org/bukkit/Location.set
    pub fn set(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }
    /// bukkit: org/bukkit/Location.getRotation
    pub fn get_rotation(&self) -> crate::pure::Rotation {
        crate::pure::Rotation::rotation(self.yaw, self.pitch)
    }
    /// bukkit: org/bukkit/Location.setRotation
    pub fn set_rotation(&mut self, yaw: f32, pitch: f32) -> &mut Self {
        self.yaw = yaw;
        self.pitch = pitch;
        self
    }
    /// bukkit: org/bukkit/Location.addRotation
    pub fn add_rotation(&mut self, yaw: f32, pitch: f32) -> &mut Self {
        self.yaw += yaw;
        self.pitch += pitch;
        self
    }
    /// bukkit: org/bukkit/Location.subtractRotation
    pub fn subtract_rotation(&mut self, yaw: f32, pitch: f32) -> &mut Self {
        self.yaw -= yaw;
        self.pitch -= pitch;
        self
    }
    /// bukkit: org/bukkit/Location.toBlockKey
    pub fn to_block_key(&self) -> i64 {
        self.get_block().get_block_key()
    }
    /// bukkit: org/bukkit/Location.isGenerated
    pub fn is_generated(&self) -> bool {
        self.get_chunk().is_generated()
    }
    /// bukkit: org/bukkit/Location.isFinite
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.yaw.is_finite() && self.pitch.is_finite()
    }
    /// bukkit: org/bukkit/Location.toLocation
    pub fn to_location(&self, world: &World) -> Self {
        Self::new(world.clone(), self.x, self.y, self.z, self.yaw, self.pitch)
    }
    /// bukkit: org/bukkit/Location.toHighestLocation
    pub fn to_highest_location(&self) -> Self {
        let mut at = self.clone();
        at.y = f64::from(self.world.get_highest_block_y_at(self.get_block_x(), self.get_block_z()));
        at
    }
    /// bukkit: org/bukkit/Location.createExplosion
    pub fn create_explosion(&self, power: f32) -> bool {
        self.world.create_explosion(self, power)
    }
    /// bukkit: org/bukkit/Location.getNearbyEntities
    pub fn get_nearby_entities(&self, x: f64, y: f64, z: f64) -> Vec<Entity> {
        self.world.get_nearby_entities(self, x, y, z)
    }
    /// bukkit: org/bukkit/Location.getNearbyEntitiesByType
    ///
    /// By the game's id, in a cube of `radius` around here.
    pub fn get_nearby_entities_by_type(&self, kind: &str, radius: f64) -> Vec<Entity> {
        self.world.get_nearby_entities_by_type(kind, self, radius)
    }
    /// bukkit: org/bukkit/Location.getNearbyLivingEntities
    ///
    pub fn get_nearby_living_entities(&self, radius: f64) -> Vec<Entity> {
        self.get_nearby_entities(radius, radius, radius).into_iter().filter(|entity| !entity.data_at("Health").is_null()).collect()
    }
    /// bukkit: org/bukkit/Location.getNearbyPlayers
    pub fn get_nearby_players(&self, radius: f64) -> Vec<Player> {
        self.world.get_players().into_iter().filter(|player| {
            let at = player.get_location();
            (at.x - self.x).abs() <= radius && (at.y - self.y).abs() <= radius && (at.z - self.z).abs() <= radius
        }).collect()
    }
    /// bukkit: org/bukkit/Location.deserialize
    pub fn deserialize(map: &Value) -> Option<Self> {
        let side = |key: &str| map[key].as_f64();
        Some(Self::new(crate::get_world(map["world"].as_str()?)?, side("x")?, side("y")?, side("z")?, side("yaw").unwrap_or(0.0) as f32, side("pitch").unwrap_or(0.0) as f32))
    }
}

// ---- Keys and vectors ----

impl NamespacedKey {
    /// bukkit: org/bukkit/NamespacedKey.asString
    pub fn as_string(&self) -> String {
        format!("{}:{}", self.get_namespace(), self.get_key())
    }
    /// bukkit: org/bukkit/NamespacedKey.value
    pub fn value(&self) -> &str {
        self.get_key()
    }
    /// bukkit: org/bukkit/NamespacedKey.namespace
    pub fn namespace(&self) -> &str {
        self.get_namespace()
    }
    /// bukkit: org/bukkit/NamespacedKey.randomKey
    pub fn random_key() -> Self {
        let text = format!("{:032x}", fresh_bits());
        Self::from_string(&format!("bukkit:{}", crate::dashed(&text))).unwrap_or_else(|| Self::minecraft("random"))
    }
}

impl Vector {
    /// bukkit: org/bukkit/util/Vector.deserialize
    pub fn deserialize(map: &Value) -> Self {
        let side = |key: &str| map[key].as_f64().unwrap_or(0.0);
        Self::new(side("x"), side("y"), side("z"))
    }
    /// bukkit: org/bukkit/util/Vector.getRandom
    ///
    /// Each side from 0 to 1.
    pub fn get_random() -> Self {
        let side = || (fresh_bits() >> 75) as f64 / (1_u64 << 53) as f64;
        Self::new(side(), side(), side())
    }
}

/// bukkit: org/bukkit/util/BlockVector.deserialize
pub fn block_vector_deserialize(map: &Value) -> crate::util::BlockVector {
    let side = |key: &str| map[key].as_f64().unwrap_or(0.0);
    crate::util::BlockVector::new(side("x"), side("y"), side("z"))
}

// ---- Profiles ----

impl crate::player::PlayerProfile {
    /// bukkit: org/bukkit/profile/PlayerProfile.getTextures, com/destroystokyo/paper/profile/PlayerProfile.getTextures
    ///
    /// The textures live on the profile here, so what reads them is a copy of it.
    pub fn get_textures(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/profile/PlayerProfile.setTextures, com/destroystokyo/paper/profile/PlayerProfile.setTextures
    ///
    /// `None` clears them, as Java's `null`.
    pub fn set_textures(&mut self, textures: Option<&Self>) {
        self.properties.retain(|(name, ..)| name != "textures");
        if let Some(source) = textures {
            self.properties.extend(source.properties.iter().filter(|(name, ..)| name == "textures").cloned());
        }
    }
    /// bukkit: org/bukkit/profile/PlayerProfile.isComplete, com/destroystokyo/paper/profile/PlayerProfile.isComplete
    pub fn is_complete(&self) -> bool {
        !self.name.is_empty() && !self.id.is_empty()
    }
    /// bukkit: org/bukkit/profile/PlayerTextures.setCape
    ///
    /// `None` takes it off. The textures aren't signed afterwards, as Java's aren't.
    pub fn set_cape(&mut self, url: Option<&str>) {
        let mut textures = self.textures();
        if !textures.is_object() {
            textures = json!({ "timestamp": 0, "profileId": self.id.replace('-', ""), "profileName": self.name, "textures": {} });
        }
        match url {
            Some(url) => textures["textures"]["CAPE"] = json!({ "url": url }),
            None => {
                if let Some(all) = textures["textures"].as_object_mut() {
                    all.remove("CAPE");
                }
            }
        }
        self.properties.retain(|(name, ..)| name != "textures");
        self.properties.push(("textures".to_owned(), base64(textures.to_string().as_bytes()), None));
    }
}

// ---- Attributes ----

/// bukkit: org/bukkit/attribute/Attribute.getDefaultValue
///
/// `attribute` as the game names it. `None` if it isn't one.
pub fn attribute_default_value(attribute: &str) -> Option<f64> {
    query(json!({ "op": "core_attribute", "id": key_of(attribute) }))["default"].as_f64()
}

impl crate::scoreboard::AttributeInstance {
    /// bukkit: org/bukkit/attribute/AttributeInstance.getDefaultValue
    pub fn get_default_value(&self) -> f64 {
        attribute_default_value(self.get_attribute()).unwrap_or(0.0)
    }
}

impl Entity {
    /// bukkit: org/bukkit/attribute/Attributable.registerAttribute
    ///
    /// Gives the entity the attribute at its default, if it hasn't got it.
    pub fn register_attribute(&self, attribute: &str) -> bool {
        let id = key_of(attribute);
        let mut list = self.data_at("attributes").as_array().cloned().unwrap_or_default();
        if list.iter().any(|row| row["id"] == id.as_str()) {
            return false;
        }
        list.push(json!({ "id": id, "base": attribute_default_value(&id).unwrap_or(0.0) }));
        self.merge(&format!("{{attributes:{}}}", crate::data::snbt(&Value::Array(list))))
    }
}

// ---- Number formats ----

fn number_format_of(saved: &Value) -> Option<crate::fam_paper::NumberFormat> {
    use crate::fam_paper::NumberFormat;
    match saved["kind"].as_str()? {
        "blank" => Some(NumberFormat::Blank),
        "fixed" => Some(NumberFormat::Fixed(Component(saved["value"].clone()))),
        "styled" => Some(NumberFormat::Styled(crate::fam_paper::Style(saved["value"].clone()))),
        _ => None,
    }
}

impl crate::scoreboard::Objective {
    /// bukkit: org/bukkit/scoreboard/Objective.numberFormat
    pub fn number_format(&self) -> Option<crate::fam_paper::NumberFormat> {
        number_format_of(&query(json!({ "op": "core_objective", "name": self.get_name() }))["number_format"])
    }
}

impl crate::scoreboard::Score {
    /// bukkit: org/bukkit/scoreboard/Score.numberFormat
    pub fn number_format(&self) -> Option<crate::fam_paper::NumberFormat> {
        number_format_of(&query(json!({ "op": "core_score", "entry": self.get_entry(), "objective": self.get_objective().get_name() }))["number_format"])
    }
}

// ---- Plugins across libraries ----
//
// Each plugin is its own DLL with its own copy of this crate, so what they share is kept by the
// engine (`core_plugin*` and `core_perm*`). While the mods start no question reaches the game, so
// each also leaves its `plugin.yml` in a file with the process's id, for what `on_enable` asks of
// the plugins before it: what it depends on has started by then.

/// `org.bukkit.plugin.PluginDescriptionFile`: `plugin.yml`.
#[derive(Clone, Debug, PartialEq)]
pub struct PluginDescriptionFile(pub Value);

impl PluginDescriptionFile {
    fn text(&self, key: &str) -> Option<String> {
        crate::config::scalar(&self.0[key])
    }
    fn list(&self, key: &str) -> Vec<String> {
        match &self.0[key] {
            Value::Array(rows) => rows.iter().filter_map(crate::config::scalar).collect(),
            Value::Null => Vec::new(),
            other => crate::config::scalar(other).into_iter().collect(),
        }
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getName, org/bukkit/plugin/PluginDescriptionFile.getRawName
    pub fn get_name(&self) -> String {
        self.text("name").unwrap_or_default()
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getProvides, org/bukkit/plugin/PluginDescriptionFile.getProvidedPlugins
    pub fn get_provides(&self) -> Vec<String> {
        self.list("provides")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getMainClass
    pub fn get_main_class(&self) -> Option<String> {
        self.text("main")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getLoadOrder
    pub fn get_load_order(&self) -> String {
        self.text("load").unwrap_or_else(|| "POSTWORLD".to_owned()).to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getLoggerPrefix
    pub fn get_logger_prefix(&self) -> Option<String> {
        self.text("prefix")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getPluginDependencies
    pub fn get_plugin_dependencies(&self) -> Vec<String> {
        self.list("depend")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getPluginSoftDependencies
    pub fn get_plugin_soft_dependencies(&self) -> Vec<String> {
        self.list("softdepend")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getLoadBeforePlugins
    pub fn get_load_before_plugins(&self) -> Vec<String> {
        self.list("loadbefore")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getLibraries
    pub fn get_libraries(&self) -> Vec<String> {
        self.list("libraries")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getPermissionDefault
    pub fn get_permission_default(&self) -> &'static str {
        self.text("default-permission").as_deref().and_then(permission_default_by_name).unwrap_or("OP")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getAwareness
    pub fn get_awareness(&self) -> Vec<String> {
        self.list("awareness")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getClassLoaderOf
    pub fn get_class_loader_of(&self) -> Option<String> {
        self.text("class-loader-of")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getPaperPluginLoader
    pub fn get_paper_plugin_loader(&self) -> Option<String> {
        self.text("paper-plugin-loader")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.save
    pub fn save(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_all(serde_yaml::to_string(&self.0).unwrap_or_default().as_bytes())
    }
}

/// `org.bukkit.plugin.Plugin`, of any library: what the others can know of a plugin.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadedPlugin {
    description: PluginDescriptionFile,
    enabled: bool,
}

impl LoadedPlugin {
    fn read(row: &Value) -> Self {
        Self { description: PluginDescriptionFile(row["yaml"].clone()), enabled: row["enabled"].as_bool().unwrap_or(true) }
    }
    pub fn get_name(&self) -> String {
        self.description.get_name()
    }
    pub fn get_description(&self) -> &PluginDescriptionFile {
        &self.description
    }
    pub fn get_version(&self) -> String {
        self.description.text("version").unwrap_or_default()
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    pub fn get_data_folder(&self) -> PathBuf {
        PathBuf::from("plugins").join(self.get_name())
    }
}

struct Own {
    plugin: Plugin,
    enable: fn(Plugin),
    disable: fn(Plugin),
}

static OWN: Mutex<Option<Own>> = Mutex::new(None);
static DISABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static STOPPING_HERE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static NAGGABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static SHARED_YET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PENDING_PERMISSIONS: Mutex<Vec<Value>> = Mutex::new(Vec::new());

fn snapshot_file() -> PathBuf {
    server_folder().join("plugins/.horizon-plugins.json")
}

/// What the plugins that have started so far left, in this run of the server.
fn snapshot() -> Vec<Value> {
    let saved = file_json(&snapshot_file());
    if saved["pid"].as_u64() != Some(u64::from(std::process::id())) {
        return Vec::new();
    }
    saved["plugins"].as_array().cloned().unwrap_or_default()
}

fn write_snapshot(row: Value) {
    let mut rows = snapshot();
    rows.retain(|known| known["name"] != row["name"]);
    rows.push(row);
    let _ = std::fs::create_dir_all(server_folder().join("plugins"));
    let _ = std::fs::write(snapshot_file(), json!({ "pid": std::process::id(), "plugins": rows }).to_string());
}

fn own_row() -> Option<Value> {
    let own = OWN.lock().unwrap_or_else(|e| e.into_inner());
    let plugin = own.as_ref()?.plugin;
    let mut yaml = crate::DESCRIPTION.get()?.clone();
    // Commands made in code are part of what other plugins can see, and `plugin.yml` doesn't list them.
    if let Ok(list) = crate::commands().try_lock() {
        let aliases = crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner());
        for (name, entry) in list.iter() {
            if yaml["commands"][name].is_null() {
                let words: Vec<&String> = aliases.iter().filter(|(word, owner)| *owner == name && *word != name).map(|(word, _)| word).collect();
                yaml["commands"][name] = json!({ "usage": entry.usage, "permission": entry.permission, "permission-message": entry.permission_message, "aliases": words });
            }
        }
    }
    Some(json!({ "name": plugin.get_name(), "yaml": yaml, "enabled": !DISABLED.load(std::sync::atomic::Ordering::Relaxed) }))
}

/// Tells the engine who this plugin is, and what it declared while no question could reach it.
fn share() {
    if !SHARED_YET.load(std::sync::atomic::Ordering::Relaxed)
        && let Some(row) = own_row()
        && query(json!({ "op": "core_plugin_add", "plugin": row })).as_bool().unwrap_or(false)
    {
        SHARED_YET.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    crate::persistence::raise_waiting_services();
    // Asked only while the help topics are stale: `core_plugins` is every plugin's row, and every plugin
    // asking it every tick cost the square of the plugin count (62 plugins: 200 ms a tick, 5 TPS, nobody on).
    if HELP_STALE.load(std::sync::atomic::Ordering::Relaxed) && !query(json!({ "op": "core_plugins" })).is_null() {
        HELP_STALE.store(false, std::sync::atomic::Ordering::Relaxed);
        make_help_topics();
    }
    if PERMISSION_OWNER_WAITS.load(std::sync::atomic::Ordering::Relaxed) {
        tell_permission_owner();
    }
    if SHARED_YET.load(std::sync::atomic::Ordering::Relaxed) {
        let pending = std::mem::take(&mut *PENDING_PERMISSIONS.lock().unwrap_or_else(|e| e.into_inner()));
        for permission in pending {
            query(json!({ "op": "core_perm_add", "permission": permission }));
        }
    }
}

/// What `bukkit_plugin!` calls before `on_enable`.
pub(crate) fn own_plugin() -> Option<Plugin> {
    OWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|own| own.plugin)
}

pub fn install(plugin: Plugin, enable: fn(Plugin), disable: fn(Plugin)) {
    *OWN.lock().unwrap_or_else(|e| e.into_inner()) = Some(Own { plugin, enable, disable });
    if let Some(row) = own_row() {
        write_snapshot(row);
    }
    let host = plugin.host;
    host.on("tick", |_| {
        share();
        horizon_mod::Answer::Pass
    });
    host.on("core_plugin_disable", move |value| {
        if value["name"].as_str().is_some_and(|name| name.eq_ignore_ascii_case(plugin.get_name())) {
            disable_self();
        }
        horizon_mod::Answer::Pass
    });
    host.on("core_plugin_enable", move |value| {
        if value["name"].as_str().is_some_and(|name| name.eq_ignore_ascii_case(plugin.get_name())) {
            enable_self();
        }
        horizon_mod::Answer::Pass
    });
    host.on("core_commands_clear", |_| {
        crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).clear();
        horizon_mod::Answer::Pass
    });
    host.on("core_command_call", |value| {
        command_called(value);
        horizon_mod::Answer::Pass
    });
    host.on("server_stopping", |_| {
        disable_at_stop();
        horizon_mod::Answer::Pass
    });
    // The hooks whose payload is the work: a hook runs only if someone listens.
    for hook in HOOKS_OF_THE_ENGINE {
        host.on(hook, |_| horizon_mod::Answer::Pass);
    }
    host.on("core_tick_server", |_| {
        crate::lead::keep_up();
        read_trades();
        horizon_mod::Answer::Pass
    });
    for hook in ["core_tick_server", "core_idle", "core_version_line"] {
        host.on(hook, |value| if value["skip"].as_bool().unwrap_or(false) { horizon_mod::Answer::Fail } else { horizon_mod::Answer::Pass });
    }
    // Paper enables a plugin once the worlds are loaded: the engine says when, one plugin at a time in
    // load order, with the game at hand (see `enable_later`).
    host.on("core_enable_plugin", |value| {
        let own = OWN_ID.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if own.is_some_and(|own| value["id"].as_str() == Some(own.as_str())) && WAITING.swap(false, std::sync::atomic::Ordering::Relaxed) {
            let own = OWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|own| (own.plugin, own.enable));
            if let Some((plugin, enable)) = own {
                enable(plugin);
            }
        }
        horizon_mod::Answer::Pass
    });
    // What `on_enable` adds from here on is what going off takes away.
    OWN_LISTENERS.store(horizon_mod::listener_mark(), std::sync::atomic::Ordering::Relaxed);
}

const HOOKS_OF_THE_ENGINE: [&str; 9] = ["core_map_tick", "core_raid_tick", "core_dragon_tick", "core_end_tick", "core_spawn_pass", "core_tick_command", "core_dispense", "core_void_mobs", "core_void_guest"];
/// Every plugin but this one.
fn other_plugins() -> Vec<String> {
    let own = get_own_plugin().map(|own| own.get_name().to_owned());
    shared_plugins().iter().filter_map(|row| row["name"].as_str().map(str::to_owned)).filter(|name| Some(name) != own.as_ref()).collect()
}

/// How many of this library's listeners are its own, as they stood before `on_enable`.
static OWN_LISTENERS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
/// The words `clear_commands` took, to give back.
static WORDS_OFF: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

/// A word of this plugin's while it is disabled. Paper still runs its PluginCommand, which throws
/// "plugin is disabled", so the sender gets the error line.
pub(crate) fn word_is_off(label: &str) -> bool {
    WORDS_OFF.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&label.to_lowercase())
}

fn disable_self() {
    if DISABLED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let own = OWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|own| (own.plugin, own.disable));
    if let Some((plugin, disable)) = own {
        disable(plugin);
    }
    cancel_tasks();
    remove_all_plugin_chunk_tickets();
    if let Some((plugin, _)) = own {
        crate::persistence::unregister_all_services(&plugin);
    }
    crate::event::HandlerList::unregister_all();
    horizon_mod::drop_listeners_from(OWN_LISTENERS.load(std::sync::atomic::Ordering::Relaxed));
    *WORDS_OFF.lock().unwrap_or_else(|e| e.into_inner()) = std::mem::take(&mut *crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()));
    if let Some(row) = own_row() {
        write_snapshot(row.clone());
        query(json!({ "op": "core_plugin_add", "plugin": row }));
    }
}

/// `on_enable` again, for a plugin that was disabled. Its own statics stay, as a Java plugin's
/// fields do; its listeners and tasks went when it did, so nothing runs twice.
fn enable_self() {
    if !DISABLED.swap(false, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    crate::event::handlers_back();
    RENDERING.store(false, std::sync::atomic::Ordering::Relaxed);
    let words = std::mem::take(&mut *WORDS_OFF.lock().unwrap_or_else(|e| e.into_inner()));
    *crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()) = words;
    let own = OWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|own| (own.plugin, own.enable));
    if let Some((plugin, enable)) = own {
        enable(plugin);
    }
    if let Some(row) = own_row() {
        write_snapshot(row.clone());
        query(json!({ "op": "core_plugin_add", "plugin": row }));
    }
}

/// What `bukkit_plugin!` calls as the server stops: `on_disable`, once.
pub fn disable_at_stop() {
    STOPPING_HERE.store(true, std::sync::atomic::Ordering::Relaxed);
    disable_self();
}

/// This library's mod id, which the engine's `core_enable_plugin` names.
static OWN_ID: Mutex<Option<String>> = Mutex::new(None);
/// `on_enable` is waiting for the engine's `core_enable_plugin`.
static WAITING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What `bukkit_plugin!` asks before `on_enable`: true when the engine enables plugins as Paper does, after
/// the worlds load, so `on_enable` waits for it. A plugin's queries in `on_enable` (its worlds, a material's
/// facts, the services it registers) need the game, and at launch there is none yet. An engine that doesn't
/// know the question says no, and `on_enable` runs at once, as before.
pub fn enable_later(manifest: &str) -> bool {
    if !query(json!({ "op": "core_enable_later" })).as_bool().unwrap_or(false) {
        return false;
    }
    // The engine knows a plugin by its name as Bukkit keeps it, with `_` for a space.
    let Some(id) = crate::config::scalar(&crate::config::parse(manifest)["name"]).map(|name| name.replace(' ', "_")) else {
        return false;
    };
    *OWN_ID.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
    WAITING.store(true, std::sync::atomic::Ordering::Relaxed);
    true
}

/// What `bukkit_plugin!` asks before `on_enable`: a library `load_plugin` brought in waits for `enable_plugin`.
pub fn starting_enabled() -> bool {
    let late = query(json!({ "op": "core_loading_late" })).as_bool().unwrap_or(false);
    if late {
        DISABLED.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(row) = own_row() {
            write_snapshot(row.clone());
            query(json!({ "op": "core_plugin_add", "plugin": row }));
        }
    }
    !late
}

/// bukkit: org/bukkit/plugin/Plugin.isEnabled, org/bukkit/plugin/java/JavaPlugin.isEnabled
pub fn plugin_is_enabled() -> bool {
    !DISABLED.load(std::sync::atomic::Ordering::Relaxed)
}

/// `Plugin::is_naggable`'s answer.
pub fn plugin_is_naggable() -> bool {
    NAGGABLE.load(std::sync::atomic::Ordering::Relaxed)
}

/// The engine's number for the plugin list (`core_plugins_version`): it moves whenever a plugin is added,
/// removed or swapped. `None` on an engine without it. A plugin that keeps something made from the list
/// (EssentialsX's alternative commands) can ask this instead of reading the whole list again.
pub fn plugins_version() -> Option<u64> {
    query(json!({ "op": "core_plugins_version" })).as_u64()
}

/// The plugin list as last read, and the engine's number for it (`core_plugins_version`).
static PLUGIN_LIST: Mutex<Option<(u64, Vec<Value>)>> = Mutex::new(None);

/// Every plugin's row. Read again only when the engine says the list changed: `getPlugin`, `isPluginEnabled`
/// and command lookups ask it many times a tick, and reading it each time (the snapshot file from disk, then
/// every row with its plugin.yml from the engine) grew with the plugin count until a join froze the server
/// (31 plugins on the Linux server, 11-03). An engine without the number: read each time, as before.
fn shared_plugins() -> Vec<Value> {
    share();
    let version = query(json!({ "op": "core_plugins_version" })).as_u64();
    if let Some(version) = version {
        if let Some((known, rows)) = PLUGIN_LIST.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            if *known == version {
                return rows.clone();
            }
        }
    }
    let mut rows = snapshot();
    for live in query(json!({ "op": "core_plugins" })).as_array().into_iter().flatten() {
        rows.retain(|known| known["name"] != live["name"]);
        rows.push(live.clone());
    }
    if let Some(version) = version {
        *PLUGIN_LIST.lock().unwrap_or_else(|e| e.into_inner()) = Some((version, rows.clone()));
    }
    rows
}

/// `org.bukkit.plugin.PluginManager`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PluginManager;

/// bukkit: org/bukkit/Bukkit.getPluginManager, org/bukkit/Server.getPluginManager
pub fn get_plugin_manager() -> PluginManager {
    PluginManager
}

impl PluginManager {
    /// bukkit: org/bukkit/plugin/PluginManager.getPlugins
    pub fn get_plugins(&self) -> Vec<LoadedPlugin> {
        shared_plugins().iter().map(LoadedPlugin::read).collect()
    }
    /// bukkit: org/bukkit/plugin/PluginManager.getPlugin
    ///
    /// By the name in `plugin.yml`, or one it `provides`. Case counts, as Java's.
    pub fn get_plugin(&self, name: &str) -> Option<LoadedPlugin> {
        let plugins = self.get_plugins();
        let found = plugins.iter().position(|plugin| plugin.get_name() == name).or_else(|| plugins.iter().position(|plugin| plugin.description.get_provides().iter().any(|known| known == name)))?;
        plugins.into_iter().nth(found)
    }
    /// bukkit: org/bukkit/plugin/PluginManager.isPluginEnabled
    pub fn is_plugin_enabled(&self, name: &str) -> bool {
        self.get_plugin(name).is_some_and(|plugin| plugin.enabled)
    }
    /// bukkit: org/bukkit/plugin/PluginManager.disablePlugin
    ///
    /// Its `on_disable` runs, its tasks are cancelled and its listeners and commands stop. A plugin
    /// can do this to itself at any time, and to another once the server is running.
    pub fn disable_plugin(&self, name: &str) -> bool {
        let mine = OWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|own| own.plugin.get_name().eq_ignore_ascii_case(name));
        if mine {
            disable_self();
            return true;
        }
        query(json!({ "op": "core_plugin_disable", "name": name })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/plugin/PluginManager.enablePlugin
    ///
    /// Of any plugin, by name. False if no plugin has that name.
    pub fn enable_plugin(&self, name: &str) -> bool {
        query(json!({ "op": "core_plugin_enable", "name": name })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/plugin/PluginManager.disablePlugins
    pub fn disable_plugins(&self) {
        for plugin in self.get_plugins() {
            self.disable_plugin(&plugin.get_name());
        }
    }
    /// bukkit: org/bukkit/plugin/PluginManager.clearPlugins
    ///
    /// Disables them all, and forgets them with their permissions.
    pub fn clear_plugins(&self) {
        self.disable_plugins();
        query(json!({ "op": "core_plugins_clear" }));
        let _ = std::fs::remove_file(snapshot_file());
    }
    /// bukkit: org/bukkit/plugin/PluginManager.useTimings
    ///
    /// False: nothing here times event calls.
    pub fn use_timings(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/plugin/PluginManager.isTransitiveDependency
    ///
    /// Whether `of` depends on `dependency`, by `depend`, `softdepend` or a `provides`, at any depth.
    pub fn is_transitive_dependency(&self, of: &PluginDescriptionFile, dependency: &PluginDescriptionFile) -> bool {
        let plugins = self.get_plugins();
        let wanted = dependency.get_name();
        let aliases = dependency.get_provides();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut waiting: Vec<String> = of.get_plugin_dependencies().into_iter().chain(of.get_plugin_soft_dependencies()).collect();
        while let Some(name) = waiting.pop() {
            if name == wanted || aliases.contains(&name) {
                return true;
            }
            if seen.insert(name.clone())
                && let Some(next) = plugins.iter().find(|plugin| plugin.get_name() == name)
            {
                waiting.extend(next.description.get_plugin_dependencies().into_iter().chain(next.description.get_plugin_soft_dependencies()));
            }
        }
        false
    }
    fn permission_json(permission: &crate::config2::Permission) -> Value {
        json!({ "name": permission.name, "default": permission.default, "description": permission.description, "children": permission.children })
    }
    fn permission_of(row: &Value) -> Option<crate::config2::Permission> {
        let name = row["name"].as_str()?;
        Some(crate::config2::Permission::load_permission(name, row))
    }
    /// bukkit: org/bukkit/plugin/PluginManager.overridePermissionManager
    ///
    /// For every plugin: the manager stays in this one, and theirs call it by JSON.
    ///
    /// `None` goes back to the engine's.
    pub fn override_permission_manager(&self, manager: Option<Box<dyn PermissionManagerOverride>>) {
        *PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner()) = manager;
        tell_permission_owner();
    }
    /// bukkit: org/bukkit/plugin/PluginManager.addPermission
    ///
    /// For every plugin. False if the name is taken, where Java throws.
    pub fn add_permission(&self, permission: &crate::config2::Permission) -> bool {
        if let Some(manager) = PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            return manager.add_permission(permission);
        }
        if let Some(answer) = foreign_manager("add_permission", json!({ "permission": Self::permission_json(permission) })) {
            return answer.as_bool().unwrap_or(false);
        }
        crate::set_permission_default(&permission.name, &permission.default);
        let row = Self::permission_json(permission);
        match query(json!({ "op": "core_perm_add", "permission": row })) {
            Value::Bool(added) => added,
            _ => {
                let mut pending = PENDING_PERMISSIONS.lock().unwrap_or_else(|e| e.into_inner());
                let fresh = !pending.iter().any(|known| known["name"] == row["name"]);
                if fresh {
                    pending.push(row);
                }
                fresh
            }
        }
    }
    /// bukkit: org/bukkit/plugin/PluginManager.getPermission
    pub fn get_permission(&self, name: &str) -> Option<crate::config2::Permission> {
        if let Some(manager) = PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            return manager.get_permission(name);
        }
        if let Some(answer) = foreign_manager("get_permission", json!({ "name": name })) {
            return Self::permission_of(&answer);
        }
        let row = query(json!({ "op": "core_perm_get", "name": name.to_ascii_lowercase() }));
        Self::permission_of(&row).or_else(|| PENDING_PERMISSIONS.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|known| known["name"] == name).and_then(Self::permission_of))
    }
    /// bukkit: org/bukkit/plugin/PluginManager.removePermission
    pub fn remove_permission(&self, name: &str) -> bool {
        if let Some(manager) = PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            return manager.remove_permission(name);
        }
        if let Some(answer) = foreign_manager("remove_permission", json!({ "name": name })) {
            return answer.as_bool().unwrap_or(false);
        }
        query(json!({ "op": "core_perm_remove", "name": name.to_ascii_lowercase() })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/plugin/PluginManager.getPermissions
    pub fn get_permissions(&self) -> Vec<crate::config2::Permission> {
        if let Some(manager) = PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            return manager.get_permissions();
        }
        if let Some(answer) = foreign_manager("get_permissions", json!({})) {
            return answer.as_array().into_iter().flatten().filter_map(Self::permission_of).collect();
        }
        query(json!({ "op": "core_perm_list" })).as_array().into_iter().flatten().filter_map(Self::permission_of).collect()
    }
    /// bukkit: org/bukkit/plugin/PluginManager.getDefaultPermissions
    ///
    /// Those on by default for an operator, or for anyone else.
    pub fn get_default_permissions(&self, operator: bool) -> Vec<crate::config2::Permission> {
        if let Some(manager) = PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            return manager.get_default_permissions(operator);
        }
        if let Some(answer) = foreign_manager("get_default_permissions", json!({ "operator": operator })) {
            return answer.as_array().into_iter().flatten().filter_map(Self::permission_of).collect();
        }
        self.get_permissions().into_iter().filter(|permission| crate::config2::permission_default_value(&permission.default, operator)).collect()
    }
    /// bukkit: org/bukkit/plugin/PluginManager.recalculatePermissionDefaults
    ///
    /// Nothing to redo: the engine works a player's permissions out as they're asked for.
    pub fn recalculate_permission_defaults(&self, _permission: &crate::config2::Permission) {}
    /// bukkit: org/bukkit/plugin/PluginManager.subscribeToPermission
    pub fn subscribe_to_permission(&self, node: &str, player: &Player) {
        query(json!({ "op": "core_perm_subscribe", "node": node.to_ascii_lowercase(), "uuid": player.uuid(), "on": true }));
    }
    /// bukkit: org/bukkit/plugin/PluginManager.unsubscribeFromPermission
    pub fn unsubscribe_from_permission(&self, node: &str, player: &Player) {
        query(json!({ "op": "core_perm_subscribe", "node": node.to_ascii_lowercase(), "uuid": player.uuid(), "on": false }));
    }
    /// bukkit: org/bukkit/plugin/PluginManager.getPermissionSubscriptions
    pub fn get_permission_subscriptions(&self, node: &str) -> Vec<Player> {
        subscribers(&query(json!({ "op": "core_perm_subscribers", "node": node.to_ascii_lowercase() })))
    }
    /// bukkit: org/bukkit/plugin/PluginManager.subscribeToDefaultPerms
    pub fn subscribe_to_default_perms(&self, operator: bool, player: &Player) {
        query(json!({ "op": "core_perm_subscribe", "node": default_node(operator), "uuid": player.uuid(), "on": true }));
    }
    /// bukkit: org/bukkit/plugin/PluginManager.unsubscribeFromDefaultPerms
    pub fn unsubscribe_from_default_perms(&self, operator: bool, player: &Player) {
        query(json!({ "op": "core_perm_subscribe", "node": default_node(operator), "uuid": player.uuid(), "on": false }));
    }
    /// bukkit: org/bukkit/plugin/PluginManager.getDefaultPermSubscriptions
    pub fn get_default_perm_subscriptions(&self, operator: bool) -> Vec<Player> {
        subscribers(&query(json!({ "op": "core_perm_subscribers", "node": default_node(operator) })))
    }
}

fn default_node(operator: bool) -> &'static str {
    if operator { "\u{0}defaults-op" } else { "\u{0}defaults" }
}

fn subscribers(rows: &Value) -> Vec<Player> {
    rows.as_array().into_iter().flatten().filter_map(|uuid| Player::read(&query(json!({ "op": "player", "uuid": uuid })))).collect()
}

/// What `permitted` asks first: the shared table's answer, `None` if it doesn't say or can't be asked.
pub(crate) fn shared_permission(uuid: &str, operator: bool, node: &str) -> Option<bool> {
    query(json!({ "op": "core_perm_check", "uuid": uuid, "op_status": operator, "node": node.to_ascii_lowercase() })).as_bool()
}

/// What `grant_permission` tells the engine, for the other plugins.
pub(crate) fn share_grant(uuid: &str, node: &str, value: Option<bool>) {
    query(json!({ "op": "core_perm_grant", "uuid": uuid, "node": node.to_ascii_lowercase(), "value": value }));
}

/// `org.bukkit.Server`, as `getServer()` hands it: the crate's functions, and these for what a port
/// writes as `plugin.getServer().x()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Server;

impl Server {
    /// bukkit: org/bukkit/Server.getOnlinePlayers
    pub fn get_online_players(&self) -> Vec<Player> {
        get_online_players()
    }
    /// bukkit: org/bukkit/Server.getPlayer
    pub fn get_player(&self, name: &str) -> Option<Player> {
        get_player(name)
    }
    /// bukkit: org/bukkit/Server.getWorld
    pub fn get_world(&self, name: &str) -> Option<World> {
        crate::get_world(name)
    }
    /// bukkit: org/bukkit/Server.getWorlds
    pub fn get_worlds(&self) -> Vec<World> {
        crate::get_worlds()
    }
    /// bukkit: org/bukkit/Server.broadcastMessage
    pub fn broadcast_message(&self, text: &str) {
        crate::broadcast_message(text);
    }
    pub fn dispatch_command(&self, line: &str) -> bool {
        crate::dispatch_command(line)
    }
    /// bukkit: org/bukkit/Server.getPluginManager
    pub fn get_plugin_manager(&self) -> PluginManager {
        PluginManager
    }
    /// bukkit: org/bukkit/Server.getCommandMap
    pub fn get_command_map(&self) -> CommandMap {
        CommandMap
    }
    /// bukkit: org/bukkit/Server.getConsoleSender
    pub fn get_console_sender(&self) -> CommandSender {
        CommandSender::Console
    }
    /// bukkit: org/bukkit/Server.getName
    pub fn get_name(&self) -> &'static str {
        crate::get_name()
    }
    /// bukkit: org/bukkit/Server.getVersion
    pub fn get_version(&self) -> String {
        crate::get_version()
    }
    /// bukkit: org/bukkit/Server.getMaxPlayers
    pub fn get_max_players(&self) -> i32 {
        crate::server::get_max_players()
    }
    /// bukkit: org/bukkit/Server.shutdown
    pub fn shutdown(&self) {
        crate::server::shutdown();
    }
}

impl CommandSender {
    /// bukkit: org/bukkit/command/CommandSender.getServer
    pub fn get_server(&self) -> Server {
        Server
    }
}

static RESOURCES: Mutex<BTreeMap<String, &'static [u8]>> = Mutex::new(BTreeMap::new());

/// bukkit: org/bukkit/plugin/java/JavaPlugin.getProvidingPlugin, org/bukkit/plugin/java/JavaPlugin.getPlugin
///
/// The plugin of this library: a Rust plugin's own classes are all in its one DLL.
pub fn get_own_plugin() -> Option<Plugin> {
    OWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|own| own.plugin)
}

impl Plugin {
    /// bukkit: org/bukkit/plugin/Plugin.getServer, org/bukkit/plugin/java/JavaPlugin.getServer
    pub fn get_server(&self) -> Server {
        Server
    }
    /// bukkit: org/bukkit/plugin/Plugin.getDataPath
    pub fn get_data_path(&self) -> PathBuf {
        self.get_data_folder()
    }
    /// bukkit: org/bukkit/plugin/Plugin.setNaggable, org/bukkit/plugin/java/JavaPlugin.setNaggable
    pub fn set_naggable(&self, on: bool) {
        NAGGABLE.store(on, std::sync::atomic::Ordering::Relaxed);
    }
    /// A file to hand out through `get_resource`, as Rust has no jar: `include_bytes!` at the call.
    pub fn add_resource(&self, name: &str, bytes: &'static [u8]) {
        RESOURCES.lock().unwrap_or_else(|e| e.into_inner()).insert(name.to_owned(), bytes);
    }
    /// bukkit: org/bukkit/plugin/Plugin.getResource, org/bukkit/plugin/java/JavaPlugin.getResource
    ///
    /// A file the plugin was built with, which `embed_resources!` puts in the library, or the data folder's copy.
    pub fn get_resource(&self, name: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
        let embedded = RESOURCES.lock().unwrap_or_else(|e| e.into_inner()).get(name).copied();
        embedded.map(std::borrow::Cow::Borrowed).or_else(|| std::fs::read(self.get_data_folder().join(name)).ok().map(std::borrow::Cow::Owned))
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.setEnabled
    pub fn set_enabled(&self, on: bool) {
        if on {
            enable_self();
        } else {
            disable_self();
        }
    }
    pub fn get_plugin_description(&self) -> PluginDescriptionFile {
        PluginDescriptionFile(crate::DESCRIPTION.get().cloned().unwrap_or(Value::Null))
    }
    /// bukkit: org/bukkit/plugin/Plugin.getLifecycleManager, org/bukkit/plugin/java/JavaPlugin.getLifecycleManager
    pub fn get_lifecycle_manager(&self) -> crate::fam_paper::LifecycleEventManager {
        crate::fam_paper::LifecycleEventManager::of(self)
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.registerCommand
    pub fn register_basic_command(&self, label: &str, aliases: &[&str], command: impl crate::fam_paper::BasicCommand) -> BTreeSet<String> {
        crate::fam_paper::Commands::of(self).register_basic(label, aliases, command)
    }
}

/// `org.bukkit.plugin.java.JavaPlugin`: a plugin's class, with what Bukkit calls on it. `java_plugin!`
/// is its entry point.
pub trait JavaPlugin: Send + Sync + 'static {
    /// bukkit: org/bukkit/plugin/Plugin.onLoad, org/bukkit/plugin/java/JavaPlugin.onLoad
    fn on_load(&self, _plugin: Plugin) {}
    /// bukkit: org/bukkit/plugin/Plugin.onEnable, org/bukkit/plugin/java/JavaPlugin.onEnable
    fn on_enable(&self, _plugin: Plugin) {}
    /// bukkit: org/bukkit/plugin/Plugin.onDisable, org/bukkit/plugin/java/JavaPlugin.onDisable
    fn on_disable(&self, _plugin: Plugin) {}
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.onCommand
    ///
    /// For a command of `plugin.yml` with no executor of its own, as Java's.
    fn on_command(&self, _sender: &CommandSender, _command: &str, _label: &str, _args: &[&str]) -> bool {
        false
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.onTabComplete
    ///
    /// `None` leaves it to the default: the names of who's on.
    fn on_tab_complete(&self, _sender: &CommandSender, _command: &str, _label: &str, _args: &[&str]) -> Option<Vec<String>> {
        None
    }
}

/// What `java_plugin!` does before `on_enable` runs: commands with neither go to the plugin's class.
pub fn bind_commands<T: JavaPlugin>(plugin: Plugin, main: &'static T) {
    let names: Vec<String> = crate::commands().lock().unwrap_or_else(|e| e.into_inner()).keys().cloned().collect();
    for name in names {
        let word = name.clone();
        plugin.set_executor(&name, move |sender, label, args| main.on_command(sender, &word, label, args));
        let word = name.clone();
        plugin.set_tab_completer(&name, move |sender, label, args| main.on_tab_complete(sender, &word, label, args).unwrap_or_else(|| crate::default_completion(args)));
    }
}

/// The entry of a plugin written as a class: `java_plugin!(MyPlugin::default())` beside `plugin.yml`.
#[macro_export]
macro_rules! java_plugin {
    ($main:expr) => {
        fn horizon_java_main() -> &'static impl $crate::fam_core::JavaPlugin {
            static MAIN: ::std::sync::OnceLock<Box<dyn ::std::any::Any + Send + Sync>> = ::std::sync::OnceLock::new();
            fn build() -> impl $crate::fam_core::JavaPlugin {
                $main
            }
            fn typed<T: 'static>(_: fn() -> T, any: &'static (dyn ::std::any::Any + Send + Sync)) -> &'static T {
                any.downcast_ref().expect("the plugin's own type")
            }
            typed(build, &**MAIN.get_or_init(|| Box::new(build())))
        }
        fn horizon_java_enable(plugin: $crate::Plugin) {
            use $crate::fam_core::JavaPlugin as _;
            // Enabling again isn't loading again.
            static LOADED: ::std::sync::Once = ::std::sync::Once::new();
            $crate::fam_core::bind_commands(plugin, horizon_java_main());
            LOADED.call_once(|| horizon_java_main().on_load(plugin));
            horizon_java_main().on_enable(plugin);
        }
        fn horizon_java_disable(plugin: $crate::Plugin) {
            use $crate::fam_core::JavaPlugin as _;
            horizon_java_main().on_disable(plugin);
        }
        $crate::bukkit_plugin!(horizon_java_enable, horizon_java_disable);
    };
}

// ---- Permissions by file ----

impl crate::persistence::PersistentDataContainer {
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.readFromBytes
    pub fn read_from_bytes(&self, bytes: &[u8], clear: bool) -> bool {
        let Value::Object(incoming) = query(json!({ "op": "core_nbt_read", "bytes": bytes })) else {
            return false;
        };
        let (mut values, mut kinds) = if clear { (serde_json::Map::new(), serde_json::Map::new()) } else { (self.values(), self.kinds()) };
        values.extend(incoming);
        if let Value::Object(read) = query(json!({ "op": "paper_nbt_kinds", "bytes": bytes })) {
            kinds.extend(read);
        }
        self.keep_kinds(values, kinds);
        true
    }
}

// ---- The server's loop, from a plugin ----

/// What a query of the engine says of a line of `server.properties` that plugins can change.
pub(crate) fn live_property(key: &str) -> Option<String> {
    query(json!({ "op": "core_live_property", "key": key })).as_str().map(str::to_owned)
}

/// bukkit: org/bukkit/Bukkit.getTickTimes, org/bukkit/Server.getTickTimes
///
/// Nanoseconds of the last hundred ticks, newest first.
pub fn get_tick_times() -> Vec<i64> {
    query(json!({ "op": "core_tick_stats" }))["times"].as_array().into_iter().flatten().filter_map(Value::as_i64).collect()
}

/// The ticks a second of the last 1, 5 and 15 minutes.
pub(crate) fn tick_rates() -> [f64; 3] {
    let stats = query(json!({ "op": "core_tick_stats" }));
    [0, 1, 2].map(|i| stats["tps"][i].as_f64().unwrap_or(20.0))
}

/// bukkit: org/bukkit/Bukkit.setIdleTimeout, org/bukkit/Server.setIdleTimeout
///
/// Minutes; 0 never kicks.
pub fn set_idle_timeout(minutes: i32) {
    query(json!({ "op": "core_set_idle", "minutes": minutes.max(0) }));
}

/// bukkit: org/bukkit/Bukkit.setPauseWhenEmptyTime, org/bukkit/Server.setPauseWhenEmptyTime
///
/// Seconds; 0 never pauses.
pub fn set_pause_when_empty_time(seconds: i32) {
    query(json!({ "op": "core_set_pause_time", "seconds": seconds.max(0) }));
}

/// bukkit: org/bukkit/Server.allowPausing
pub fn allow_pausing(plugin: &Plugin, allow: bool) {
    query(json!({ "op": "core_allow_pausing", "plugin": plugin.get_name(), "allow": allow }));
}

impl crate::core2::ServerTickManager {
    /// bukkit: org/bukkit/ServerTickManager.getFrozenTicksToRun
    pub fn get_frozen_ticks_to_run(&self) -> i32 {
        int(&query(json!({ "op": "core_frozen_ticks" })))
    }
}

impl World {
    /// bukkit: org/bukkit/World.setTicksPerSpawns
    ///
    /// `category` is a `SpawnCategory` name. 0 stops that kind spawning, below 0 is the game's own again.
    pub fn set_ticks_per_spawns(&self, category: &str, ticks: i32) {
        query(json!({ "op": "core_set_ticks_per", "category": mob_category(category), "ticks": ticks }));
    }
    /// bukkit: org/bukkit/World.setTicksPerAnimalSpawns
    pub fn set_ticks_per_animal_spawns(&self, ticks: i32) {
        self.set_ticks_per_spawns("ANIMAL", ticks);
    }
    /// bukkit: org/bukkit/World.setTicksPerMonsterSpawns
    pub fn set_ticks_per_monster_spawns(&self, ticks: i32) {
        self.set_ticks_per_spawns("MONSTER", ticks);
    }
    /// bukkit: org/bukkit/World.setTicksPerWaterSpawns
    pub fn set_ticks_per_water_spawns(&self, ticks: i32) {
        self.set_ticks_per_spawns("WATER_ANIMAL", ticks);
    }
    /// bukkit: org/bukkit/World.setTicksPerWaterAmbientSpawns
    pub fn set_ticks_per_water_ambient_spawns(&self, ticks: i32) {
        self.set_ticks_per_spawns("WATER_AMBIENT", ticks);
    }
    /// bukkit: org/bukkit/World.setTicksPerWaterUndergroundCreatureSpawns
    pub fn set_ticks_per_water_underground_creature_spawns(&self, ticks: i32) {
        self.set_ticks_per_spawns("WATER_UNDERGROUND_CREATURE", ticks);
    }
    /// bukkit: org/bukkit/World.setTicksPerAmbientSpawns
    pub fn set_ticks_per_ambient_spawns(&self, ticks: i32) {
        self.set_ticks_per_spawns("AMBIENT", ticks);
    }
    /// bukkit: org/bukkit/RegionAccessor.getFluidData
    pub fn get_fluid_data(&self, x: i32, y: i32, z: i32) -> crate::fam_paper::FluidData {
        crate::fam_paper::FluidData::of(&self.get_block_at(x, y, z))
    }
    /// bukkit: org/bukkit/RegionAccessor.createEntity
    ///
    /// An entity not yet in the level: `add_entity` puts it there. Its data can be changed in between.
    pub fn create_entity(&self, at: &Location, kind: &str) -> crate::entities3::EntitySnapshot {
        crate::entities3::create_entity_snapshot(&format!("{{id:{},Pos:[{:?}d,{:?}d,{:?}d],Rotation:[{:?}f,{:?}f]}}", crate::data::quoted(&key_of(kind)), at.x, at.y, at.z, at.yaw, at.pitch))
    }
    /// bukkit: org/bukkit/RegionAccessor.addEntity
    pub fn add_entity(&self, entity: &crate::entities3::EntitySnapshot) -> Option<Entity> {
        entity.create_entity(self).map(|id| Entity::of(id, self))
    }
}

// ---- Bars of the game's own ----

impl Raid {
    /// bukkit: org/bukkit/Raid.setBadOmenLevel
    pub fn set_bad_omen_level(&self, level: i32) -> bool {
        self.world.ask("core_raid_omen", json!({ "id": self.id, "level": level.clamp(0, 5) })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/Raid.setTotalWaves
    ///
    /// From the raid's next tick.
    pub fn set_total_waves(&self, waves: i32) {
        self.world.ask("core_raid_waves", json!({ "id": self.id, "waves": waves }));
    }
    /// bukkit: org/bukkit/Raid.getBossBar
    ///
    /// A bar like any other from here on: the game goes on setting its title, progress and who
    /// sees it, as on Paper. None once the raid has gone.
    pub fn get_boss_bar(&self) -> Option<crate::BossBar> {
        let id = self.world.ask("core_raid_bar_own", json!({ "id": self.id }));
        Some(crate::BossBar { id: id.as_str()?.to_owned() })
    }
}

impl DragonBattle {
    fn order(&self, order: &str) {
        query(json!({ "op": "core_dragon_order", "order": order }));
    }
    /// bukkit: org/bukkit/boss/DragonBattle.generateEndPortal
    ///
    /// On the End's next tick. False if a portal is there already.
    pub fn generate_end_portal(&self, with_portals: bool) -> bool {
        if !self.state()["exit_portal"].is_null() {
            return false;
        }
        query(json!({ "op": "core_dragon_portal", "active": with_portals })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/boss/DragonBattle.initiateRespawn
    ///
    /// On the End's next tick, if four crystals stand on the portal, as the game wants them.
    pub fn initiate_respawn(&self) {
        self.order("respawn");
    }
    /// bukkit: org/bukkit/boss/DragonBattle.resetCrystals
    pub fn reset_crystals(&self) {
        self.order("reset");
    }
    /// bukkit: org/bukkit/boss/DragonBattle.spawnNewGateway
    ///
    /// On the End's next tick. False if all twenty are out.
    pub fn spawn_new_gateway(&self) -> bool {
        let left = self.state()["gateways_left"].as_u64().unwrap_or(0) > 0;
        if left {
            self.order("gateway");
        }
        left
    }
    /// bukkit: org/bukkit/boss/DragonBattle.setRespawnPhase
    ///
    /// A `RespawnPhase` name. False with no respawn under way, as Java's.
    pub fn set_respawn_phase(&self, phase: &str) -> bool {
        if self.get_respawn_phase() == "NONE" {
            return false;
        }
        self.order(&phase.to_ascii_lowercase());
        true
    }
    /// bukkit: org/bukkit/boss/DragonBattle.getRespawnCrystals
    pub fn get_respawn_crystals(&self) -> Vec<Entity> {
        let state = self.state();
        let spots = state["respawn_crystals"].as_array().cloned().unwrap_or_default();
        let near = |one: &Value, other: &Value| (0..3).all(|i| (one[i].as_f64().unwrap_or(0.0) - other[i].as_f64().unwrap_or(9e9)).abs() < 1.0);
        state["crystal_list"].as_array().into_iter().flatten().filter(|crystal| spots.iter().any(|spot| near(spot, &crystal["position"]))).filter_map(|crystal| crystal["id"].as_u64().filter(|id| *id != 0)).map(|id| Entity::of(id, &self.world())).collect()
    }
    /// bukkit: org/bukkit/boss/DragonBattle.getBossBar
    ///
    /// The battle's, dragon or no dragon: a bar like any other from here on, as on Paper, with the
    /// game setting its progress and who sees it.
    pub fn get_boss_bar(&self) -> Option<crate::BossBar> {
        let id = query(json!({ "op": "core_dragon_bar" }));
        Some(crate::BossBar { id: id.as_str()?.to_owned() })
    }
}

// ---- Commands that answer in text ----

/// `Bukkit.createCommandSender`'s sender: what a command it runs says goes to the closure.
pub struct FeedbackCommandSender {
    feedback: Box<dyn Fn(Component) + Send + Sync>,
}

/// bukkit: org/bukkit/Bukkit.createCommandSender, org/bukkit/Server.createCommandSender
pub fn create_command_sender(feedback: impl Fn(Component) + Send + Sync + 'static) -> FeedbackCommandSender {
    FeedbackCommandSender { feedback: Box::new(feedback) }
}

impl FeedbackCommandSender {
    /// `Bukkit.dispatchCommand` with this sender: with the console's rights, at once, outside a
    /// player's own event. False if it failed or the moment is one the game can't run it in.
    pub fn dispatch(&self, line: &str) -> bool {
        let answer = query(json!({ "op": "core_run_captured", "line": line.trim_start_matches('/') }));
        for said in answer["lines"].as_array().into_iter().flatten() {
            (self.feedback)(Component(said.clone()));
        }
        answer["ran"].as_bool().unwrap_or(false)
    }
}

// ---- Links ----

pub(crate) const LINK_TYPES: [&str; 10] = ["REPORT_BUG", "COMMUNITY_GUIDELINES", "SUPPORT", "STATUS", "FEEDBACK", "COMMUNITY", "WEBSITE", "FORUMS", "NEWS", "ANNOUNCEMENTS"];

/// `org.bukkit.ServerLinks`: the list players are shown. `get_server_links` reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerLinks {
    links: Vec<ServerLink>,
}

impl ServerLinks {
    /// The list as it is now.
    pub fn current() -> Self {
        Self { links: get_server_links() }
    }
    fn send(&self) {
        let rows: Vec<Value> = self
            .links
            .iter()
            .map(|link| match (link.kind.as_deref().and_then(|kind| LINK_TYPES.iter().position(|known| *known == kind)), &link.name) {
                (Some(index), _) => json!({ "known": index, "url": link.url }),
                (None, name) => json!({ "label": adventure::legacy(name.as_deref().unwrap_or_default()).0, "url": link.url }),
            })
            .collect();
        query(json!({ "op": "core_links_set", "links": rows }));
    }
    /// bukkit: org/bukkit/ServerLinks.getLink
    pub fn get_link(&self, kind: &str) -> Option<&ServerLink> {
        self.links.iter().find(|link| link.kind.as_deref() == Some(kind))
    }
    /// bukkit: org/bukkit/ServerLinks.setLink
    ///
    /// Everyone sees it at once, and whoever joins next.
    pub fn set_link(&mut self, kind: &str, url: &str) {
        self.links.retain(|link| link.kind.as_deref() != Some(kind));
        self.links.push(ServerLink { kind: Some(kind.to_owned()), name: None, url: url.to_owned() });
        self.send();
    }
    /// bukkit: org/bukkit/ServerLinks.addLink
    pub fn add_link(&mut self, display_name: &str, url: &str) {
        self.links.push(ServerLink { kind: None, name: Some(display_name.to_owned()), url: url.to_owned() });
        self.send();
    }
    /// bukkit: org/bukkit/ServerLinks.removeLink
    pub fn remove_link(&mut self, link: &ServerLink) -> bool {
        let before = self.links.len();
        self.links.retain(|known| known != link);
        let removed = self.links.len() != before;
        if removed {
            self.send();
        }
        removed
    }
    /// bukkit: org/bukkit/ServerLinks.copy
    pub fn copy(&self) -> Self {
        self.clone()
    }
}

// ---- What's left of the small ones ----

impl crate::core2::UnsafeValues {
    /// bukkit: org/bukkit/UnsafeValues.nextEntityId
    pub fn next_entity_id(&self) -> i32 {
        int(&query(json!({ "op": "core_next_entity_id" })))
    }
    /// bukkit: org/bukkit/UnsafeValues.deserializeItemHover
    ///
    /// An item from a hover event's `show_item` contents: `id`, `count` and `components`.
    pub fn deserialize_item_hover(&self, hover: &Value) -> Option<ItemStack> {
        ItemStack::read(hover)
    }
}

/// bukkit: org/bukkit/enchantments/EnchantmentOffer.rollEnchantmentOffers
///
/// What an enchanting table offers for `stack`: `None` in a slot with nothing, as Java's `null`.
pub fn roll_enchantment_offers(stack: &ItemStack, seed: i32, bookshelves: i32, count: usize) -> Vec<Option<EnchantmentOffer>> {
    let rows = query(json!({ "op": "core_enchant_offers", "stack": stack.to_json(), "seed": seed, "bookshelves": bookshelves }));
    let mut offers: Vec<Option<EnchantmentOffer>> = rows.as_array().into_iter().flatten().map(|row| Some(EnchantmentOffer::new(row["enchantment"].as_str()?, int(&row["level"]), int(&row["cost"])))).collect();
    offers.resize(count, None);
    offers
}

impl crate::entities::Projectile {
    /// bukkit: org/bukkit/projectiles/BlockProjectileSource.getBlock
    ///
    /// The dispenser it came out of, if one shot it.
    pub fn get_block_shooter(&self) -> Option<Block> {
        let found = query(json!({ "op": "core_projectile_block", "id": self.0.get_entity_id() }));
        Some(World::of(found["dimension"].as_str()?).get_block_at(int(&found["pos"][0]), int(&found["pos"][1]), int(&found["pos"][2])))
    }
}

impl crate::pure::Spawner {
    /// bukkit: org/bukkit/spawner/BaseSpawner.getPotentialSpawns
    pub fn get_potential_spawns(&self) -> Vec<crate::fam_items::SpawnerEntry> {
        self.0.get_state_data()["SpawnPotentials"].as_array().into_iter().flatten().map(|row| crate::fam_items::SpawnerEntry(row.clone())).collect()
    }
    /// bukkit: org/bukkit/spawner/BaseSpawner.setPotentialSpawns
    pub fn set_potential_spawns(&self, entries: &[crate::fam_items::SpawnerEntry]) -> bool {
        let rows: Vec<Value> = entries.iter().map(|entry| entry.0.clone()).collect();
        self.0.merge_state(&format!("{{SpawnPotentials:{}}}", crate::data::snbt(&Value::Array(rows))))
    }
    /// bukkit: org/bukkit/spawner/BaseSpawner.addPotentialSpawn
    pub fn add_potential_spawn(&self, entity: &crate::entities3::EntitySnapshot, weight: i32) -> bool {
        self.0.merge_state(&format!("{{SpawnPotentials:[{}{{data:{{entity:{}}},weight:{weight}}}]}}", self.get_potential_spawns().iter().map(|entry| format!("{},", crate::data::snbt(&entry.0))).collect::<String>(), entity.get_as_string()))
    }
}

/// `org.bukkit.scheduler.BukkitWorker`: a thread running an async task.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BukkitWorker {
    id: u32,
    thread: std::thread::ThreadId,
}

static WORKERS: Mutex<Vec<BukkitWorker>> = Mutex::new(Vec::new());
static NEXT_WORKER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);

/// While this lives, the thread is one of `get_active_workers`.
pub(crate) struct WorkerGuard(BukkitWorker);

pub(crate) fn worker_started() -> WorkerGuard {
    let worker = BukkitWorker { id: NEXT_WORKER.fetch_sub(1, std::sync::atomic::Ordering::Relaxed), thread: std::thread::current().id() };
    WORKERS.lock().unwrap_or_else(|e| e.into_inner()).push(worker);
    WorkerGuard(worker)
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        WORKERS.lock().unwrap_or_else(|e| e.into_inner()).retain(|worker| worker.id != self.0.id);
    }
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.getActiveWorkers
pub fn get_active_workers() -> Vec<BukkitWorker> {
    WORKERS.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

impl BukkitWorker {
    /// bukkit: org/bukkit/scheduler/BukkitWorker.getTaskId
    pub fn get_task_id(&self) -> u32 {
        self.id
    }
    /// bukkit: org/bukkit/scheduler/BukkitWorker.getThread
    pub fn get_thread(&self) -> std::thread::ThreadId {
        self.thread
    }
    /// bukkit: org/bukkit/scheduler/BukkitWorker.getOwner
    ///
    /// The plugin of this library: every task it can see is its own.
    pub fn get_owner(&self) -> Option<Plugin> {
        get_own_plugin()
    }
}

// ---- Commands, across plugins ----

/// A command of this plugin, as `PluginCommand.execute` runs it. `None` if it has no such word, or
/// a completer of its own is holding the list.
fn run_here(sender: &CommandSender, label: &str, args: &[&str]) -> Option<bool> {
    // `other:word` names another plugin: its command, not this plugin's word of the same name.
    if let Some((space, _)) = label.rsplit_once(':') {
        let mine = crate::DESCRIPTION.get().and_then(|d| d["name"].as_str().map(str::to_ascii_lowercase));
        if mine.as_deref() != Some(space.to_ascii_lowercase().as_str()) {
            return None;
        }
    }
    let word = label.rsplit(':').next().unwrap_or(label).to_lowercase();
    let owner = crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).get(&word).cloned()?;
    if !plugin_is_enabled() {
        return Some(false);
    }
    let (executor, usage) = {
        let list = crate::commands().try_lock().ok()?;
        let entry = list.get(&owner)?;
        if let Some(node) = &entry.permission
            && !sender.has_permission(node)
        {
            match entry.permission_message.as_deref() {
                Some("") => {}
                Some(message) => sender.send_message(message),
                None => sender.send_message(&get_permission_message()),
            }
            return Some(true);
        }
        (entry.executor.clone(), entry.usage.clone())
    };
    let done = executor.is_some_and(|executor| executor(sender, label, args));
    if !done && !usage.is_empty() {
        sender.send_message(&usage.replace("<command>", label));
    }
    Some(done)
}

/// What a command of this plugin completes to, for `sender`.
fn complete_here(sender: &CommandSender, label: &str, args: &[&str]) -> Option<Vec<String>> {
    let word = label.rsplit(':').next().unwrap_or(label).to_lowercase();
    let owner = crate::ALIASES.lock().unwrap_or_else(|e| e.into_inner()).get(&word).cloned()?;
    let completer = {
        let list = crate::commands().try_lock().ok()?;
        let entry = list.get(&owner)?;
        if entry.permission.as_ref().is_some_and(|node| !sender.has_permission(node)) {
            return Some(Vec::new());
        }
        entry.completer.clone()
    };
    Some(completer.map_or_else(|| crate::default_completion(args), |completer| completer(sender, label, args)))
}

/// Asks the plugin that owns a command, in its own DLL, by way of the engine.
fn call_other(plugin: &str, kind: &str, sender: &CommandSender, label: &str, args: &[&str]) -> Value {
    let uuid = match sender {
        CommandSender::Player(player) => json!(player.uuid()),
        CommandSender::Console | CommandSender::Proxied(_) | CommandSender::Remote(_) | CommandSender::Entity(_) => Value::Null,
    };
    query(json!({ "op": "core_command_call", "plugin": plugin, "kind": kind, "sender": uuid, "label": label, "args": args }))
}

/// The other end of `call_other`: every plugin hears it, and the one it names answers.
fn command_called(value: &Value) {
    let named = value["plugin"].as_str().unwrap_or_default();
    if !get_own_plugin().is_some_and(|own| own.get_name().eq_ignore_ascii_case(named)) {
        return;
    }
    if matches!(value["kind"].as_str(), Some("generator" | "provider" | "bind_generator" | "bind_provider")) {
        query(json!({ "op": "core_reply", "value": crate::worlds::answer_named(value) }));
        return;
    }
    if value["kind"] == "service" {
        query(json!({ "op": "core_reply", "value": crate::persistence::answer_service(value) }));
        return;
    }
    if value["kind"] == "permission_manager" {
        query(json!({ "op": "core_reply", "value": answer_permission_manager(value) }));
        return;
    }
    if value["kind"] == "map" {
        query(json!({ "op": "core_reply", "value": answer_map(value) }));
        return;
    }
    let sender = match value["sender"].as_str() {
        Some(uuid) => match Player::read(&query(json!({ "op": "player", "uuid": uuid }))) {
            Some(player) => CommandSender::Player(player),
            None => return,
        },
        None => CommandSender::Console,
    };
    let label = value["label"].as_str().unwrap_or_default();
    let args: Vec<&str> = value["args"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    let reply = if value["kind"] == "complete" { json!(complete_here(&sender, label, &args).unwrap_or_default()) } else { json!(run_here(&sender, label, &args).unwrap_or(false)) };
    query(json!({ "op": "core_reply", "value": reply }));
}

/// `org.bukkit.command.PluginCommand`: a command of any plugin.
#[derive(Clone, Debug)]
pub struct PluginCommand {
    plugin: String,
    command: crate::pure2::Command,
}

/// bukkit: org/bukkit/Bukkit.getPluginCommand, org/bukkit/Server.getPluginCommand
///
/// Of any plugin, by name, alias or `plugin:name`.
pub fn get_plugin_command(name: &str) -> Option<PluginCommand> {
    if let Some((plugin, command)) = plugin_command(name) {
        return Some(PluginCommand { plugin, command });
    }
    let own = get_own_plugin()?;
    own_command(name).map(|command| PluginCommand { plugin: own.get_name().to_owned(), command })
}

impl PluginCommand {
    /// bukkit: org/bukkit/command/PluginCommand.getPlugin
    pub fn get_plugin(&self) -> Option<LoadedPlugin> {
        PluginManager.get_plugin(&self.plugin)
    }
    /// The plugin's name, which holds whether it's in `get_plugin`'s list yet or not.
    pub fn get_plugin_name(&self) -> &str {
        &self.plugin
    }
    /// Name, usage, aliases and the rest: `org.bukkit.command.Command`.
    pub fn get_command(&self) -> &crate::pure2::Command {
        &self.command
    }
    /// bukkit: org/bukkit/command/PluginCommand.execute
    ///
    /// In whichever plugin owns it, at once, with what it answers.
    pub fn execute(&self, sender: &CommandSender, label: &str, args: &[&str]) -> bool {
        if let Some(done) = self.is_own().then(|| run_here(sender, label, args)).flatten() {
            return done;
        }
        call_other(&self.plugin, "run", sender, label, args).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/command/PluginCommand.tabComplete
    pub fn tab_complete(&self, sender: &CommandSender, label: &str, args: &[&str]) -> Vec<String> {
        if let Some(list) = self.is_own().then(|| complete_here(sender, label, args)).flatten() {
            return list;
        }
        call_other(&self.plugin, "complete", sender, label, args).as_array().into_iter().flatten().filter_map(|word| word.as_str().map(str::to_owned)).collect()
    }
    fn is_own(&self) -> bool {
        get_own_plugin().is_some_and(|own| own.get_name().eq_ignore_ascii_case(&self.plugin))
    }
}

impl Plugin {
    /// bukkit: org/bukkit/command/PluginCommand.getExecutor
    ///
    /// Of this plugin's commands: another plugin's is run with `PluginCommand::execute`.
    pub fn get_executor(&self, command: &str) -> Option<crate::Executor> {
        crate::commands().lock().unwrap_or_else(|e| e.into_inner()).get(command)?.executor.clone()
    }
    /// bukkit: org/bukkit/command/PluginCommand.getTabCompleter
    ///
    /// Of this plugin's commands: another plugin's is asked with `PluginCommand::tab_complete`.
    pub fn get_tab_completer(&self, command: &str) -> Option<crate::Completer> {
        crate::commands().lock().unwrap_or_else(|e| e.into_inner()).get(command)?.completer.clone()
    }
}

// ---- Events a plugin raises ----

/// An event `PluginManager::call_event` can raise: other plugins' listeners hear it as they would
/// the game's, in priority order, each seeing what those before it did.
pub trait RaisedEvent: crate::event::Event {
    /// The hook's payload for this event, as the game would give it.
    fn raised_payload(&self) -> Value;
    fn set_raised_cancelled(&mut self, cancelled: bool);
}

impl PluginManager {
    /// bukkit: org/bukkit/plugin/PluginManager.callEvent
    ///
    /// Back here, `event` is what the listeners made of it. Nothing happens to the game: as in
    /// Java, what's raised is a question, and the caller does the work if the answer lets it.
    pub fn call_event<E: RaisedEvent>(&self, event: &mut E) {
        let mut payload = event.raised_payload();
        payload["raised"] = json!(true);
        let answer = query(json!({ "op": "core_raise", "event": E::HOOK, "payload": payload }));
        let carried = if answer["payload"].is_null() { &payload } else { &answer["payload"] };
        if let Some(fresh) = E::read(carried) {
            *event = fresh;
        }
        let code = answer["answer"].as_i64().unwrap_or(0);
        event.set_raised_cancelled(code == 1 || code == 2);
    }
}

fn block_pos(block: &Block) -> Value {
    json!([block.get_x(), block.get_y(), block.get_z()])
}

impl crate::fam_ev_world::BlockBreakEvent {
    /// bukkit: org/bukkit/event/block/BlockBreakEvent.<init>
    pub fn new(block: &Block, player: &Player) -> Option<Self> {
        let mut event = <Self as crate::event::Event>::read(&json!({ "uuid": player.uuid(), "pos": block_pos(block), "raised": true }))?;
        event.set_cancelled(false);
        Some(event)
    }
}

impl RaisedEvent for crate::fam_ev_world::BlockBreakEvent {
    fn raised_payload(&self) -> Value {
        json!({ "uuid": self.get_player().uuid(), "pos": block_pos(self.get_block()) })
    }
    fn set_raised_cancelled(&mut self, cancelled: bool) {
        self.set_cancelled(cancelled);
    }
}

impl crate::fam_ev_world::SignChangeEvent {
    /// bukkit: org/bukkit/event/block/SignChangeEvent.<init>
    ///
    /// `front` is the side: Java's `Side`.
    pub fn new(block: &Block, player: &Player, lines: &[String], front: bool) -> Option<Self> {
        let mut event = <Self as crate::event::Event>::read(&json!({ "uuid": player.uuid(), "pos": block_pos(block), "level": block.get_world().dimension(), "front": front, "lines": lines, "raised": true }))?;
        event.set_cancelled(false);
        Some(event)
    }
}

impl RaisedEvent for crate::fam_ev_world::SignChangeEvent {
    fn raised_payload(&self) -> Value {
        let block = self.get_block();
        json!({ "uuid": self.get_player().map(|player| player.uuid().to_owned()), "pos": block.as_ref().map(block_pos), "level": block.as_ref().map(|block| block.get_world().dimension().to_owned()), "front": self.get_side() == "FRONT", "lines": self.get_lines() })
    }
    fn set_raised_cancelled(&mut self, cancelled: bool) {
        self.set_cancelled(cancelled);
    }
}

// ---- The process's memory ----

/// What `/gc` shows. Bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeMemory {
    /// `Runtime.maxMemory`: the machine's RAM, as nothing caps a Rust process the way `-Xmx` caps a JVM.
    pub max: u64,
    /// `Runtime.totalMemory`: what this process holds of it now.
    pub total: u64,
    /// `Runtime.freeMemory`: what the machine has left. A Rust process has no heap of its own that
    /// it could say the free part of, which is what Java's figure is.
    pub free: u64,
}

/// The numbers `Runtime.getRuntime()` gives a Java plugin. `None` on a system this can't ask.
#[cfg(windows)]
pub fn runtime_memory() -> Option<RuntimeMemory> {
    #[repr(C)]
    struct MemoryStatus {
        length: u32,
        load: u32,
        total_physical: u64,
        available_physical: u64,
        total_page_file: u64,
        available_page_file: u64,
        total_virtual: u64,
        available_virtual: u64,
        available_extended: u64,
    }
    #[repr(C)]
    #[derive(Default)]
    struct ProcessCounters {
        size: u32,
        page_faults: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged: usize,
        quota_paged: usize,
        quota_peak_non_paged: usize,
        quota_non_paged: usize,
        page_file: usize,
        peak_page_file: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn K32GetProcessMemoryInfo(process: *mut core::ffi::c_void, counters: *mut ProcessCounters, size: u32) -> i32;
    }
    let mut status = MemoryStatus { length: size_of::<MemoryStatus>() as u32, load: 0, total_physical: 0, available_physical: 0, total_page_file: 0, available_page_file: 0, total_virtual: 0, available_virtual: 0, available_extended: 0 };
    let mut counters = ProcessCounters { size: size_of::<ProcessCounters>() as u32, ..ProcessCounters::default() };
    // SAFETY: both are the structs these calls document, sized as they ask, and live for the calls.
    let read = unsafe { GlobalMemoryStatusEx(&raw mut status) != 0 && K32GetProcessMemoryInfo(GetCurrentProcess(), &raw mut counters, counters.size) != 0 };
    read.then_some(RuntimeMemory { max: status.total_physical, total: counters.working_set as u64, free: status.available_physical })
}

/// The numbers `Runtime.getRuntime()` gives a Java plugin. `None` on a system this can't ask.
#[cfg(target_os = "linux")]
pub fn runtime_memory() -> Option<RuntimeMemory> {
    let field = |text: &str, name: &str| text.lines().find_map(|line| line.strip_prefix(name)?.strip_prefix(':')?.split_whitespace().next()?.parse::<u64>().ok());
    let machine = std::fs::read_to_string("/proc/meminfo").ok()?;
    let process = std::fs::read_to_string("/proc/self/status").ok()?;
    Some(RuntimeMemory { max: field(&machine, "MemTotal")? * 1024, total: field(&process, "VmRSS")? * 1024, free: field(&machine, "MemAvailable")? * 1024 })
}

/// The numbers `Runtime.getRuntime()` gives a Java plugin. `None` on a system this can't ask.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn runtime_memory() -> Option<RuntimeMemory> {
    None
}

// ---- Loading a plugin while the server runs ----

/// bukkit: org/bukkit/plugin/PluginManager.loadPlugin
///
/// Loaded and not enabled: `enable_plugin` does that. The library is copied first, so the file can
/// be rebuilt while it runs.
pub fn load_plugin(path: &std::path::Path) -> Option<LoadedPlugin> {
    let name = query(json!({ "op": "core_load_plugin", "path": path.to_string_lossy() }));
    share();
    PluginManager.get_plugin(name.as_str()?)
}

/// bukkit: org/bukkit/plugin/PluginManager.loadPlugins
///
/// Each library in the folder. Loading runs none of a plugin's code, so the order is no matter.
pub fn load_plugins(folder: &std::path::Path) -> Vec<LoadedPlugin> {
    let is_library = |path: &std::path::Path| path.extension().is_some_and(|kind| kind == "dll" || kind == "so" || kind == "dylib");
    std::fs::read_dir(folder).into_iter().flatten().flatten().map(|entry| entry.path()).filter(|path| is_library(path)).filter_map(|path| load_plugin(&path)).collect()
}

// ---- A border of no world ----

/// What `create_world_border` makes: a border only the players it's shown to see.
#[derive(Clone, Debug, PartialEq)]
pub struct VirtualBorder {
    center: (f64, f64),
    size: f64,
    warning_time: i32,
    warning_distance: i32,
    damage_buffer: f64,
    damage_amount: f64,
    viewers: Vec<String>,
}

/// bukkit: org/bukkit/Bukkit.createWorldBorder, org/bukkit/Server.createWorldBorder
///
/// Its own type, as it has no world: `show_to` is `Player.setWorldBorder` with one of these.
pub fn create_world_border() -> VirtualBorder {
    VirtualBorder { center: (0.0, 0.0), size: 5.9999968e7, warning_time: 15, warning_distance: 5, damage_buffer: 5.0, damage_amount: 0.2, viewers: Vec::new() }
}

impl VirtualBorder {
    fn send(&self, uuid: &str) {
        self.send_changing(uuid, self.size, 0);
    }
    fn send_changing(&self, uuid: &str, from: f64, ticks: i64) {
        query(json!({ "op": "core_border_send", "uuid": uuid, "x": self.center.0, "z": self.center.1, "from": from, "size": self.size, "ticks": ticks, "warning_time": self.warning_time, "warning_blocks": self.warning_distance }));
    }
    fn changed(&self) {
        for uuid in &self.viewers {
            self.send(uuid);
        }
    }
    /// `Player.setWorldBorder` with this one. From then on, what changes here goes to them too.
    pub fn show_to(&mut self, player: &Player) {
        self.send(player.uuid());
        if !self.viewers.iter().any(|known| known == player.uuid()) {
            self.viewers.push(player.uuid().to_owned());
        }
    }
    pub fn get_size(&self) -> f64 {
        self.size
    }
    pub fn set_size(&mut self, size: f64) {
        self.size = size.clamp(1.0, 5.9999968e7);
        self.changed();
    }
    /// `changeSize`: grows or shrinks to `size` over `ticks`.
    pub fn change_size(&mut self, size: f64, ticks: i64) {
        let from = self.size;
        self.size = size.clamp(1.0, 5.9999968e7);
        for uuid in &self.viewers {
            self.send_changing(uuid, from, ticks);
        }
    }
    /// The centre's x and z.
    pub fn get_center(&self) -> (f64, f64) {
        self.center
    }
    pub fn set_center(&mut self, x: f64, z: f64) {
        self.center = (x.clamp(-2.9999984e7, 2.9999984e7), z.clamp(-2.9999984e7, 2.9999984e7));
        self.changed();
    }
    pub fn get_warning_time(&self) -> i32 {
        self.warning_time
    }
    pub fn set_warning_time(&mut self, seconds: i32) {
        self.warning_time = seconds;
        self.changed();
    }
    pub fn get_warning_distance(&self) -> i32 {
        self.warning_distance
    }
    pub fn set_warning_distance(&mut self, blocks: i32) {
        self.warning_distance = blocks;
        self.changed();
    }
    pub fn is_inside(&self, at: &Location) -> bool {
        let half = self.size / 2.0;
        (at.x - self.center.0).abs() <= half && (at.z - self.center.1).abs() <= half
    }
    /// `WorldBorder.getWorld`: none, for a border of no world.
    pub fn get_world(&self) -> Option<World> {
        None
    }
    /// `WorldBorder.reset`.
    pub fn reset(&mut self) {
        let viewers = std::mem::take(&mut self.viewers);
        *self = Self { viewers, ..create_world_border() };
        self.changed();
    }
    /// `WorldBorder.setSize(size, seconds)`.
    pub fn set_size_over(&mut self, size: f64, seconds: i64) {
        self.change_size(size, seconds * 20);
    }
    pub fn get_damage_buffer(&self) -> f64 {
        self.damage_buffer
    }
    pub fn set_damage_buffer(&mut self, blocks: f64) {
        self.damage_buffer = blocks;
    }
    pub fn get_damage_amount(&self) -> f64 {
        self.damage_amount
    }
    pub fn set_damage_amount(&mut self, damage: f64) {
        self.damage_amount = damage;
    }
    /// `WorldBorder.getMaxSize`.
    pub fn get_max_size(&self) -> f64 {
        5.9999968e7
    }
    /// `WorldBorder.getMaxCenterCoordinate`.
    pub fn get_max_center_coordinate(&self) -> f64 {
        2.9999984e7
    }
}

// ---- Help topics from a factory ----

/// bukkit: org/bukkit/help/HelpTopicFactory, org/bukkit/help/HelpTopicFactory.createTopic
pub trait HelpTopicFactory {
    /// `None` for no topic.
    fn create_topic(&self, command: &crate::pure2::Command) -> Option<crate::core2::HelpTopic>;
}

type HelpFactory = (Box<dyn Fn(&crate::pure2::Command) -> bool + Send + Sync>, Box<dyn HelpTopicFactory + Send + Sync>);

static HELP_FACTORIES: Mutex<Vec<HelpFactory>> = Mutex::new(Vec::new());
/// A factory is waiting for every plugin's commands.
static HELP_STALE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn make_help_topics() {
    let factories = HELP_FACTORIES.lock().unwrap_or_else(|e| e.into_inner());
    for (word, command) in CommandMap.get_known_commands() {
        if word.contains(':') {
            continue;
        }
        if let Some((_, factory)) = factories.iter().find(|(wanted, _)| wanted(&command)) {
            if let Some(topic) = factory.create_topic(&command) {
                crate::core2::add_help_topic(topic);
            }
        }
    }
}

/// bukkit: org/bukkit/help/HelpMap.registerHelpTopicFactory
///
/// Every plugin's commands go through the first factory that wants them, as the server finishes
/// starting. Rust has no `Class` to choose them by: `wanted` asks the same of a command, and
/// `|_| true` is `Command.class`.
pub fn register_help_topic_factory(wanted: impl Fn(&crate::pure2::Command) -> bool + Send + Sync + 'static, factory: impl HelpTopicFactory + Send + Sync + 'static) {
    HELP_FACTORIES.lock().unwrap_or_else(|e| e.into_inner()).push((Box::new(wanted), Box::new(factory)));
    HELP_STALE.store(true, std::sync::atomic::Ordering::Relaxed);
}

// ---- Who listens on what ----

type ChannelListener = std::sync::Arc<dyn Fn(&str, &Player, &[u8]) + Send + Sync>;

static CHANNEL_LISTENERS: Mutex<BTreeMap<String, ChannelListener>> = Mutex::new(BTreeMap::new());

pub(crate) fn note_channel_listener(channel: &str, listener: ChannelListener) {
    CHANNEL_LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).insert(channel.to_owned(), listener);
}

impl PluginMessageListenerRegistration {
    /// bukkit: org/bukkit/plugin/messaging/PluginMessageListenerRegistration.getListener
    pub fn get_listener(&self) -> Option<ChannelListener> {
        CHANNEL_LISTENERS.lock().unwrap_or_else(|e| e.into_inner()).get(&self.channel).filter(|_| self.is_valid()).cloned()
    }
}

impl Player {
    /// bukkit: org/bukkit/plugin/messaging/PluginMessageRecipient.getListeningPluginChannels, org/bukkit/entity/Player.getListeningPluginChannels
    pub fn get_listening_plugin_channels(&self) -> BTreeSet<String> {
        query(json!({ "op": "core_client_channels", "uuid": self.uuid() })).as_array().into_iter().flatten().filter_map(|channel| channel.as_str().map(str::to_owned)).collect()
    }
}

/// bukkit: org/bukkit/Server.getListeningPluginChannels, org/bukkit/Bukkit.getListeningPluginChannels
pub fn get_listening_plugin_channels() -> BTreeSet<String> {
    crate::persistence::get_incoming_channels().into_iter().collect()
}

// ---- A goat horn of no registry ----

impl MusicInstrument {
    /// bukkit: org/bukkit/MusicInstrument.create
    ///
    /// An instrument of its own, with no entry in the game's data: `apply_to` puts it on a horn.
    pub fn create(sound_event: &str, use_duration: f32, range: f32, description: &Component) -> Self {
        Self { key: String::new(), inline: Some(json!({ "sound_event": key_of(sound_event), "use_duration": use_duration, "range": range, "description": description.0 })) }
    }
    /// `MusicInstrumentMeta.setInstrument`, for either kind.
    pub fn apply_to(&self, meta: &mut crate::meta::MusicInstrumentMeta) {
        match &self.inline {
            Some(inline) => meta.0.set_component_at("minecraft:instrument", "", inline.clone()),
            None => meta.set_instrument(&self.key),
        }
    }
}

/// The time as the ban files write it: `yyyy-MM-dd HH:mm:ss Z`, in UTC.
fn ban_date() -> String {
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |time| time.as_secs()) as i64;
    let (days, rest) = (seconds.div_euclid(86_400), seconds.rem_euclid(86_400));
    // Howard Hinnant's `civil_from_days`.
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let of_era = shifted.rem_euclid(146_097);
    let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * of_year + 2) / 153;
    let day = of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} +0000", rest / 3600, rest / 60 % 60, rest % 60)
}

impl crate::config::YamlConfiguration {
    /// bukkit: org/bukkit/configuration/file/FileConfigurationOptions.pathSeparator, org/bukkit/configuration/file/YamlConfigurationOptions.pathSeparator, org/bukkit/configuration/MemoryConfigurationOptions.pathSeparator
    pub fn set_path_separator(&mut self, separator: char) -> &mut Self {
        self.separator = separator;
        self
    }
}

// ---- Shooting ----

/// The game's four words of a UUID, as an entity's `Owner` is saved.
fn uuid_words(uuid: &str) -> Option<[i32; 4]> {
    let id = u128::from_str_radix(&uuid.replace('-', ""), 16).ok()?;
    Some([(id >> 96) as u32 as i32, (id >> 64) as u32 as i32, (id >> 32) as u32 as i32, id as u32 as i32])
}

/// `Projectile.shootFromRotation`: tilt in degrees and speed, by what's shot. Fireballs carry their
/// push in `acceleration_power` instead.
fn shot_of(kind: &str) -> Option<(f64, f64)> {
    Some(match kind {
        "snowball" | "egg" | "ender_pearl" | "wind_charge" => (0.0, 1.5),
        "experience_bottle" => (-20.0, 0.7),
        "splash_potion" | "lingering_potion" => (-20.0, 0.5),
        "arrow" | "spectral_arrow" => (0.0, 3.0),
        "trident" => (0.0, 2.5),
        "fireball" | "small_fireball" | "dragon_fireball" | "wither_skull" => (0.0, 0.1),
        _ => return None,
    })
}

fn launch(owner: &str, world: &World, eye: [f64; 3], (yaw, pitch): (f64, f64), carried: [f64; 3], kind: &str) -> Option<u64> {
    let kind = kind.trim_start_matches("minecraft:");
    let (tilt, speed) = shot_of(kind)?;
    let (yaw, pitch) = (yaw.to_radians(), pitch.to_radians());
    let aim = [-yaw.sin() * pitch.cos(), -(pitch + tilt.to_radians()).sin(), yaw.cos() * pitch.cos()];
    let length = aim.iter().map(|side| side * side).sum::<f64>().sqrt().max(1e-9);
    let fireball = speed < 0.2;
    // `RandomSource.triangle`, with an inaccuracy of one.
    let random = || (fresh_bits() as u64 as f64) / (u64::MAX as f64);
    let spread = |side: f64| if fireball { side / length } else { side / length + 0.017_227_5 * (random() - random()) };
    let motion = [0, 1, 2].map(|side| spread(aim[side]) * speed + carried[side]);
    let words = uuid_words(owner)?;
    let push = if fireball { ",acceleration_power:0.1d" } else { "" };
    let snbt = format!("{{id:\"minecraft:{kind}\",Pos:[{:?}d,{:?}d,{:?}d],Motion:[{:?}d,{:?}d,{:?}d],Owner:[I;{},{},{},{}]{push}}}", eye[0], eye[1] - 0.1, eye[2], motion[0], motion[1], motion[2], words[0], words[1], words[2], words[3]);
    crate::entities3::create_entity_snapshot(&snbt).create_entity(world)
}

impl Entity {
    /// bukkit: org/bukkit/projectiles/ProjectileSource.launchProjectile, org/bukkit/entity/LivingEntity.launchProjectile
    ///
    /// `kind` as the game names it: `snowball`, `arrow`. `None` for what the game doesn't shoot this way.
    pub fn launch_projectile(&self, kind: &str) -> Option<Entity> {
        let at = self.get_location()?;
        let moving = self.get_velocity();
        let carried = [moving[0], if self.is_on_ground() { 0.0 } else { moving[1] }, moving[2]];
        let id = launch(&self.get_unique_id(), &self.get_world(), [at.x, at.y + self.get_eye_height(), at.z], (f64::from(at.yaw), f64::from(at.pitch)), carried, kind)?;
        Some(Entity::of(id, &self.get_world()))
    }
}

impl Player {
    /// bukkit: org/bukkit/projectiles/ProjectileSource.launchProjectile, org/bukkit/entity/LivingEntity.launchProjectile
    ///
    /// `kind` as the game names it: `snowball`, `arrow`. `None` for what the game doesn't shoot this way.
    pub fn launch_projectile(&self, kind: &str) -> Option<Entity> {
        let at = self.get_eye_location();
        let id = launch(self.uuid(), &self.get_world(), [at.x, at.y, at.z], (f64::from(at.yaw), f64::from(at.pitch)), [0.0; 3], kind)?;
        Some(Entity::of(id, &self.get_world()))
    }
}

// ---- Registries and the last of the server ----

/// `Registry.MEMORY_MODULE_TYPE`: the keys of a mob's memories.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryKeyRegistry;

impl MemoryKeyRegistry {
    /// bukkit: org/bukkit/Registry$3.size
    pub fn size(&self) -> usize {
        crate::enums::memory_key::values().len()
    }
    /// bukkit: org/bukkit/Registry$3.get
    pub fn get(&self, key: &NamespacedKey) -> Option<crate::fam_entity::MemoryKey> {
        crate::fam_entity::MemoryKey::get_by_key(&key.as_string())
    }
    /// bukkit: org/bukkit/Registry$3.iterator
    pub fn iterator(&self) -> impl Iterator<Item = crate::fam_entity::MemoryKey> {
        crate::enums::memory_key::values().iter().map(|name| crate::fam_entity::MemoryKey((*name).to_owned()))
    }
}

/// bukkit: org/bukkit/Bukkit.getPotionBrewer, org/bukkit/Server.getPotionBrewer
pub fn get_potion_brewer() -> crate::fam_items::PotionBrewer {
    crate::fam_items::PotionBrewer
}

impl World {
    /// bukkit: org/bukkit/World.getTickableTileEntityCount
    pub fn get_tickable_tile_entity_count(&self) -> i32 {
        int(&self.ask("core_level_counts", json!({}))["tickers"])
    }
}

/// bukkit: org/bukkit/attribute/Attribute.getSentiment
///
/// `Attribute.Sentiment`'s name, as the game's item tooltips colour the attribute.
pub fn attribute_sentiment(attribute: &str) -> &'static str {
    match key_of(attribute).trim_start_matches("minecraft:") {
        "burning_time" | "fall_damage_multiplier" => "NEGATIVE",
        "gravity" | "scale" | "waypoint_transmit_range" | "waypoint_receive_range" => "NEUTRAL",
        _ => "POSITIVE",
    }
}

// ---- A merchant of no mob ----

/// What `create_merchant` makes: trades of a plugin's own, with no villager behind them.
#[derive(Clone, Debug)]
pub struct VirtualMerchant {
    title: Option<Component>,
    recipes: std::sync::Arc<Mutex<Vec<crate::core2::MerchantRecipe>>>,
}

impl PartialEq for VirtualMerchant {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.recipes, &other.recipes)
    }
}

/// Who's in a screen of which merchant: its trades count there, and come back here.
static TRADING: Mutex<Vec<(Player, std::sync::Weak<Mutex<Vec<crate::core2::MerchantRecipe>>>)>> = Mutex::new(Vec::new());

/// The uses each open screen has reached. From `get_recipes`, and each tick, for a screen about to close.
pub(crate) fn read_trades() {
    TRADING.lock().unwrap_or_else(|e| e.into_inner()).retain(|(player, recipes)| {
        let Some(recipes) = recipes.upgrade() else {
            return false;
        };
        let open = query(json!({ "op": "items_menu", "uuid": player.uuid() }));
        let Some(offers) = open["offers"].as_array().filter(|_| open["kind"] == "Merchant" && open["mob"].is_null()) else {
            return false;
        };
        let mut held = recipes.lock().unwrap_or_else(|e| e.into_inner());
        for (recipe, offer) in held.iter_mut().zip(offers) {
            recipe.set_uses(int(&offer["uses"]));
        }
        true
    });
}

/// bukkit: org/bukkit/Bukkit.createMerchant, org/bukkit/Server.createMerchant
///
/// Its own type, as it has no mob: `open_for` is `Player.openMerchant` with one of these. A trade's
/// uses come back here while the screen is open.
pub fn create_merchant(title: Option<&Component>) -> VirtualMerchant {
    VirtualMerchant { title: title.cloned(), recipes: std::sync::Arc::default() }
}

impl VirtualMerchant {
    fn held(&self) -> std::sync::MutexGuard<'_, Vec<crate::core2::MerchantRecipe>> {
        self.recipes.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn get_recipes(&self) -> Vec<crate::core2::MerchantRecipe> {
        read_trades();
        self.held().clone()
    }
    pub fn set_recipes(&mut self, recipes: Vec<crate::core2::MerchantRecipe>) {
        *self.held() = recipes;
    }
    pub fn get_recipe(&self, index: usize) -> Option<crate::core2::MerchantRecipe> {
        self.get_recipes().get(index).cloned()
    }
    /// False past the end, where Java's throws.
    pub fn set_recipe(&mut self, index: usize, recipe: crate::core2::MerchantRecipe) -> bool {
        let mut held = self.held();
        let Some(slot) = held.get_mut(index) else {
            return false;
        };
        *slot = recipe;
        true
    }
    pub fn get_recipe_count(&self) -> usize {
        self.held().len()
    }
    /// `Player.openMerchant` with this one.
    pub fn open_for(&self, player: &Player) -> bool {
        let recipes = self.held().clone();
        let mut builder = crate::fam_items::MenuType("minecraft:merchant".to_owned()).builder().merchant(crate::fam_items::MerchantSource::Offers(recipes));
        if let Some(title) = &self.title {
            builder = builder.title(title);
        }
        let opened = builder.build(player).open();
        if opened {
            TRADING.lock().unwrap_or_else(|e| e.into_inner()).push((player.clone(), std::sync::Arc::downgrade(&self.recipes)));
        }
        opened
    }
}

/// What `override_permission_manager` takes: `io.papermc.paper.plugin.PermissionManager`, as a plugin writes one.
pub trait PermissionManagerOverride: Send + Sync {
    fn add_permission(&self, permission: &crate::config2::Permission) -> bool;
    fn get_permission(&self, name: &str) -> Option<crate::config2::Permission>;
    fn remove_permission(&self, name: &str) -> bool;
    fn get_permissions(&self) -> Vec<crate::config2::Permission>;
    fn get_default_permissions(&self, operator: bool) -> Vec<crate::config2::Permission>;
}

static PERMISSION_MANAGER: Mutex<Option<Box<dyn PermissionManagerOverride>>> = Mutex::new(None);
/// The game wasn't there to hear who has the manager: as a plugin starts.
static PERMISSION_OWNER_WAITS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Tells every plugin whose manager is in force.
fn tell_permission_owner() {
    let mine = PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner()).is_some();
    let name = get_own_plugin().filter(|_| mine).map(|own| own.get_name().to_owned());
    let told = !query(json!({ "op": "core_perm_manager_set", "plugin": name, "was": get_own_plugin().map(|own| own.get_name().to_owned()) })).is_null();
    PERMISSION_OWNER_WAITS.store(!told, std::sync::atomic::Ordering::Relaxed);
}

/// A manager another plugin has in force, asked. `None` if there isn't one.
fn foreign_manager(method: &str, args: Value) -> Option<Value> {
    if PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner()).is_some() {
        return None;
    }
    let owner = query(json!({ "op": "core_perm_manager" }));
    let owner = owner.as_str()?;
    Some(query(json!({ "op": "core_command_call", "plugin": owner, "kind": "permission_manager", "method": method, "call": args })))
}

/// The other end of `foreign_manager`.
fn answer_permission_manager(value: &Value) -> Value {
    let held = PERMISSION_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    let (Some(manager), call) = (held.as_ref(), &value["call"]) else {
        return Value::Null;
    };
    let name = call["name"].as_str().unwrap_or_default();
    let list = |rows: Vec<crate::config2::Permission>| Value::Array(rows.iter().map(PluginManager::permission_json).collect());
    match value["method"].as_str().unwrap_or_default() {
        "add_permission" => PluginManager::permission_of(&call["permission"]).map_or(json!(false), |permission| json!(manager.add_permission(&permission))),
        "get_permission" => manager.get_permission(name).as_ref().map_or(Value::Null, PluginManager::permission_json),
        "remove_permission" => json!(manager.remove_permission(name)),
        "get_permissions" => list(manager.get_permissions()),
        "get_default_permissions" => list(manager.get_default_permissions(call["operator"].as_bool().unwrap_or(false))),
        _ => Value::Null,
    }
}

/// A game command now, as Java's run. What's said is the console's. In the hooks that hold the hub
/// themselves it runs when the packet ends, and the answer is true.
pub(crate) fn run_now(line: &str) -> bool {
    query(json!({ "op": "core_run_now", "line": line })).as_bool().unwrap_or(true)
}

/// The world's own folder: `level-name`, which isn't always `world`.
pub(crate) fn world_folder() -> PathBuf {
    server_folder().join(crate::server::property("level-name").filter(|name| !name.is_empty()).unwrap_or_else(|| "world".to_owned()))
}

// ---- What a plugin says about its damage ----

impl crate::core2::DamageSource {
    /// bukkit: org/bukkit/damage/DamageSource.getDamageContext
    ///
    /// What `with_damage_context` put in: Adventure's `Pointers`, by each pointer's key.
    pub fn get_damage_context(&self) -> &BTreeMap<String, Value> {
        &self.context
    }
    /// bukkit: org/bukkit/damage/DamageSource$Builder.withDamageContext
    pub fn with_damage_context(mut self, fill: impl FnOnce(&mut BTreeMap<String, Value>)) -> Self {
        fill(&mut self.context);
        self
    }
    /// The `/damage` words after the target and amount.
    fn command_words(&self) -> String {
        let mut words = key_of(self.get_damage_type());
        match (self.get_causing_entity(), self.get_direct_entity(), self.get_damage_location()) {
            (Some(causer), direct, _) => {
                words += &format!(" by {}", crate::dashed(&causer.get_unique_id()));
                if let Some(direct) = direct {
                    words += &format!(" from {}", crate::dashed(&direct.get_unique_id()));
                }
            }
            (None, _, Some(at)) => words += &format!(" at {:?} {:?} {:?}", at.x, at.y, at.z),
            _ => {}
        }
        words
    }
    fn store_context(&self, uuid: &str) {
        if !self.context.is_empty() {
            query(json!({ "op": "core_damage_context", "uuid": uuid.replace('-', ""), "context": self.context }));
        }
    }
}

/// What the `EntityDamageEvent` of a hit reads back: the context of the damage dealt to this
/// entity this tick. Nothing else in the game carries one.
pub(crate) fn stored_damage_context(mut source: crate::core2::DamageSource, victim: &Value) -> crate::core2::DamageSource {
    let Some(uuid) = victim["uuid"].as_str() else {
        return source;
    };
    if let Some(context) = query(json!({ "op": "core_damage_context_of", "uuid": uuid.replace('-', "") })).as_object() {
        source.context = context.iter().map(|(key, value)| (key.clone(), value.clone())).collect();
    }
    source
}

impl Entity {
    /// bukkit~: org/bukkit/entity/Damageable.damage | a command, so it lands at the end of the tick inside a player's event
    pub fn damage_by(&self, amount: f64, source: &crate::core2::DamageSource) {
        source.store_context(&self.get_unique_id());
        run_command(&format!("damage {} {amount} {}", crate::dashed(&self.get_unique_id()), source.command_words()));
    }
}

impl Player {
    /// bukkit~: org/bukkit/entity/Damageable.damage | a command, so it lands at the end of the tick inside a player's event
    pub fn damage_by(&self, amount: f64, source: &crate::core2::DamageSource) {
        source.store_context(self.uuid());
        run_command(&format!("damage {} {amount} {}", self.get_name(), source.command_words()));
    }
}


// ---- `/execute as ... run` ----

/// Who `/execute as` ran a plugin's command as.
#[derive(Clone, Debug, PartialEq)]
pub enum ProxiedCallee {
    Player(Player),
    Entity(Entity),
}

impl Eq for ProxiedCallee {}

/// `org.bukkit.command.ProxiedCommandSender`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProxiedCommandSender {
    caller: CommandSender,
    callee: ProxiedCallee,
    name: String,
}

impl ProxiedCommandSender {
    /// bukkit: org/bukkit/command/ProxiedCommandSender.getCaller
    pub fn get_caller(&self) -> &CommandSender {
        &self.caller
    }
    /// bukkit: org/bukkit/command/ProxiedCommandSender.getCallee
    pub fn get_callee(&self) -> CommandSender {
        match &self.callee {
            ProxiedCallee::Player(player) => CommandSender::Player(player.clone()),
            ProxiedCallee::Entity(entity) => CommandSender::Entity(entity.clone()),
        }
    }
    /// bukkit: org/bukkit/command/ProxiedCommandSender.audience
    ///
    /// The caller, who hears what the command says.
    pub fn audience(&self) -> &CommandSender {
        &self.caller
    }
    /// bukkit: org/bukkit/command/ProxiedCommandSender.sendMessage
    pub fn send_message(&self, text: &Component) {
        self.caller.send_component(text);
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// `org.bukkit.command.RemoteConsoleCommandSender`: whoever sent a line by RCON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteConsoleCommandSender {
    address: String,
}

impl RemoteConsoleCommandSender {
    /// bukkit: org/bukkit/command/RemoteConsoleCommandSender.getAddress
    pub fn get_address(&self) -> Option<std::net::SocketAddr> {
        self.address.parse().ok()
    }
    /// What the command says goes back to the client, with the rest of its output.
    pub fn send_message(&self, text: &str) {
        query(json!({ "op": "core_remote_say", "text": text }));
    }
}

/// The sender of the line RCON is running now, if it's RCON's.
pub(crate) fn remote_sender() -> Option<CommandSender> {
    let address = query(json!({ "op": "core_remote_peer" })).as_str()?.to_owned();
    Some(CommandSender::Remote(RemoteConsoleCommandSender { address }))
}

/// The sender of a `command` event from RCON, or from `/execute` when someone else is who it ran as.
pub(crate) fn event_sender(event: &Value) -> Option<CommandSender> {
    if let Some(address) = event["remote"].as_str() {
        return Some(CommandSender::Remote(RemoteConsoleCommandSender { address: address.to_owned() }));
    }
    let proxied = &event["proxied"];
    if !proxied.is_object() {
        return None;
    }
    let player = |key: &str| proxied[key].as_str().and_then(|uuid| Player::read(&query(json!({ "op": "player", "uuid": uuid }))));
    let caller = player("caller").map_or(CommandSender::Console, CommandSender::Player);
    if let Some(callee) = player("player") {
        if caller.as_player() == Some(&callee) {
            return Some(caller);
        }
        let name = callee.get_name().to_owned();
        return Some(CommandSender::Proxied(Box::new(ProxiedCommandSender { caller, callee: ProxiedCallee::Player(callee), name })));
    }
    let world = World::of(proxied["dimension"].as_str().unwrap_or("minecraft:overworld"));
    let Some(id) = proxied["entity"].as_u64() else {
        return Some(caller);
    };
    let entity = Entity::of(id, &world);
    let name = entity.get_name();
    Some(CommandSender::Proxied(Box::new(ProxiedCommandSender { caller, callee: ProxiedCallee::Entity(entity), name })))
}

// ---- Transformers ----

/// bukkit: org/bukkit/util/BlockTransformer$TransformationState
#[derive(Clone, Debug, PartialEq)]
pub struct TransformationState {
    original: crate::block_data::BlockData,
    world: crate::block_data::BlockData,
}

impl TransformationState {
    pub(crate) fn of(original: crate::block_data::BlockData, world: crate::block_data::BlockData) -> Self {
        Self { original, world }
    }
    /// bukkit: org/bukkit/util/BlockTransformer$TransformationState.getOriginal
    ///
    /// What the structure has for this place.
    pub fn get_original(&self) -> &crate::block_data::BlockData {
        &self.original
    }
    /// bukkit: org/bukkit/util/BlockTransformer$TransformationState.getWorld
    ///
    /// What's in the world there now.
    pub fn get_world(&self) -> &crate::block_data::BlockData {
        &self.world
    }
}

/// bukkit: org/bukkit/util/BlockTransformer
pub trait BlockTransformer {
    /// bukkit~: org/bukkit/util/BlockTransformer.transform | with `BlockData`, so a block entity's own data isn't offered: what's answered is in the world and `region` while the next is asked, as Java's are placed by then
    fn transform(&self, region: &crate::worlds::LimitedRegion, x: i32, y: i32, z: i32, current: crate::block_data::BlockData, state: &TransformationState) -> crate::block_data::BlockData;
}

/// bukkit: org/bukkit/util/EntityTransformer
pub trait EntityTransformer {
    /// bukkit~: org/bukkit/util/EntityTransformer.transform | an `EntitySnapshot`, as the entity isn't in the world yet to be an `Entity`
    fn transform(&self, region: &crate::worlds::LimitedRegion, x: i32, y: i32, z: i32, entity: &mut crate::entities3::EntitySnapshot, allowed_to_spawn: bool) -> bool;
}

impl Structure {
    /// bukkit: org/bukkit/structure/Structure.place
    ///
    /// `place`, each block and entity going through the transformers first, in order.
    #[allow(clippy::too_many_arguments)]
    pub fn place_transformed(&self, at: &Location, entities: bool, rotation: &str, mirror: &str, integrity: f32, seed: i64, blocks: &[&dyn BlockTransformer], with_entities: &[&dyn EntityTransformer]) -> bool {
        let dimension = at.world.dimension().to_owned();
        let pos = [at.get_block_x(), at.get_block_y(), at.get_block_z()];
        let request = json!({ "template": self.template, "dimension": dimension, "pos": pos, "entities": entities, "rotation": rotation.to_ascii_lowercase(), "mirror": mirror, "integrity": integrity, "seed": seed });
        let mut asking = request.clone();
        asking["op"] = json!("core_structure_plan");
        let plan = query(asking);
        let Some(planned) = plan["blocks"].as_array() else {
            return false;
        };
        let region = |x: i32, z: i32| crate::worlds::LimitedRegion::of(&at.world, x >> 4, z >> 4);
        let mut changes = Vec::new();
        for row in planned {
            let (x, y, z) = (int(&row[0]), int(&row[1]), int(&row[2]));
            let original = row[3].as_str().unwrap_or_default();
            let world = crate::block_data::BlockData::parse(&at.world.get_block_at(x, y, z).get_block_data());
            let state = TransformationState { original: crate::block_data::BlockData::parse(original), world };
            let result = blocks.iter().fold(state.original.clone(), |current, transformer| transformer.transform(&region(x, z), x, y, z, current, &state));
            query(json!({ "op": "core_structure_preview", "dimension": dimension, "pos": [x, y, z], "state": result.get_as_string() }));
            if result != state.original {
                changes.push(json!([x, y, z, result.get_as_string()]));
            }
        }
        let mut verdicts = Vec::new();
        for row in plan["entities"].as_array().into_iter().flatten() {
            let (x, y, z) = (int(&row["pos"][0]), int(&row["pos"][1]), int(&row["pos"][2]));
            let before = crate::data::snbt(&row["data"]);
            let mut snapshot = crate::entities3::create_entity_snapshot(&before);
            let keep = with_entities.iter().fold(true, |keep, transformer| transformer.transform(&region(x, z), x, y, z, &mut snapshot, keep));
            verdicts.push(match (keep, snapshot.get_as_string() == before) {
                (false, _) => Value::Null,
                (true, true) => json!(true),
                (true, false) => json!(snapshot.get_as_string()),
            });
        }
        let mut placing = request;
        placing["op"] = json!("core_structure_place");
        placing["changes"] = json!(changes);
        placing["verdicts"] = json!(verdicts);
        query(placing).as_bool().unwrap_or(false)
    }
}

/// A word of the game's command tree for this plugin: while it starts as every mod's, and later
/// through the engine, which sends players the new tree.
pub(crate) fn command_word(host: horizon_mod::Host, word: &str) {
    command_usage(host, &format!("{word} [args:text]"));
}

pub(crate) fn command_usage(host: horizon_mod::Host, usage: &str) {
    // The command's own word in lower case, as Bukkit's command map keeps it.
    let (word, rest) = usage.split_once(' ').unwrap_or((usage, ""));
    let lowered = if word.chars().any(|c| c.is_ascii_uppercase()) { format!("{} {rest}", word.to_lowercase()).trim_end().to_owned() } else { usage.to_owned() };
    let usage = lowered.as_str();
    if !query(json!({ "op": "core_command_add", "text": usage, "level": 0 })).as_bool().unwrap_or(false) {
        host.command(usage, 0);
    }
}

/// The blocks whose class in 26.3 answers a click, sorted: Paper's `CraftBlockType.isInteractable` run over the game's registry.
const INTERACTABLE: [&str; 338] = ["acacia_button", "acacia_door", "acacia_fence", "acacia_fence_gate", "acacia_hanging_sign", "acacia_shelf", "acacia_sign", "acacia_trapdoor", "acacia_wall_hanging_sign", "acacia_wall_sign", "anvil", "bamboo_button", "bamboo_door", "bamboo_fence", "bamboo_fence_gate", "bamboo_hanging_sign", "bamboo_shelf", "bamboo_sign", "bamboo_trapdoor", "bamboo_wall_hanging_sign", "bamboo_wall_sign", "barrel", "beacon", "bee_nest", "beehive", "bell", "birch_button", "birch_door", "birch_fence", "birch_fence_gate", "birch_hanging_sign", "birch_shelf", "birch_sign", "birch_trapdoor", "birch_wall_hanging_sign", "birch_wall_sign", "black_bed", "black_candle", "black_candle_cake", "black_shulker_box", "blast_furnace", "blue_bed", "blue_candle", "blue_candle_cake", "blue_shulker_box", "brewing_stand", "brown_bed", "brown_candle", "brown_candle_cake", "brown_shulker_box", "cake", "campfire", "candle", "candle_cake", "cartography_table", "cauldron", "cave_vines", "cave_vines_plant", "chain_command_block", "cherry_button", "cherry_door", "cherry_fence", "cherry_fence_gate", "cherry_hanging_sign", "cherry_shelf", "cherry_sign", "cherry_trapdoor", "cherry_wall_hanging_sign", "cherry_wall_sign", "chest", "chipped_anvil", "chiseled_bookshelf", "command_block", "comparator", "composter", "copper_chest", "copper_door", "copper_golem_statue", "copper_trapdoor", "crafter", "crafting_table", "crimson_button", "crimson_door", "crimson_fence", "crimson_fence_gate", "crimson_hanging_sign", "crimson_shelf", "crimson_sign", "crimson_trapdoor", "crimson_wall_hanging_sign", "crimson_wall_sign", "cyan_bed", "cyan_candle", "cyan_candle_cake", "cyan_shulker_box", "damaged_anvil", "dark_oak_button", "dark_oak_door", "dark_oak_fence", "dark_oak_fence_gate", "dark_oak_hanging_sign", "dark_oak_shelf", "dark_oak_sign", "dark_oak_trapdoor", "dark_oak_wall_hanging_sign", "dark_oak_wall_sign", "daylight_detector", "decorated_pot", "deepslate_redstone_ore", "dispenser", "dragon_egg", "dropper", "enchanting_table", "ender_chest", "exposed_copper_chest", "exposed_copper_door", "exposed_copper_golem_statue", "exposed_copper_trapdoor", "flower_pot", "furnace", "gray_bed", "gray_candle", "gray_candle_cake", "gray_shulker_box", "green_bed", "green_candle", "green_candle_cake", "green_shulker_box", "grindstone", "hopper", "iron_door", "iron_trapdoor", "jigsaw", "jukebox", "jungle_button", "jungle_door", "jungle_fence", "jungle_fence_gate", "jungle_hanging_sign", "jungle_shelf", "jungle_sign", "jungle_trapdoor", "jungle_wall_hanging_sign", "jungle_wall_sign", "lava_cauldron", "lectern", "lever", "light", "light_blue_bed", "light_blue_candle", "light_blue_candle_cake", "light_blue_shulker_box", "light_gray_bed", "light_gray_candle", "light_gray_candle_cake", "light_gray_shulker_box", "lime_bed", "lime_candle", "lime_candle_cake", "lime_shulker_box", "loom", "magenta_bed", "magenta_candle", "magenta_candle_cake", "magenta_shulker_box", "mangrove_button", "mangrove_door", "mangrove_fence", "mangrove_fence_gate", "mangrove_hanging_sign", "mangrove_shelf", "mangrove_sign", "mangrove_trapdoor", "mangrove_wall_hanging_sign", "mangrove_wall_sign", "moving_piston", "nether_brick_fence", "note_block", "oak_button", "oak_door", "oak_fence", "oak_fence_gate", "oak_hanging_sign", "oak_shelf", "oak_sign", "oak_trapdoor", "oak_wall_hanging_sign", "oak_wall_sign", "orange_bed", "orange_candle", "orange_candle_cake", "orange_shulker_box", "oxidized_copper_chest", "oxidized_copper_door", "oxidized_copper_golem_statue", "oxidized_copper_trapdoor", "pale_oak_button", "pale_oak_door", "pale_oak_fence", "pale_oak_fence_gate", "pale_oak_hanging_sign", "pale_oak_shelf", "pale_oak_sign", "pale_oak_trapdoor", "pale_oak_wall_hanging_sign", "pale_oak_wall_sign", "pink_bed", "pink_candle", "pink_candle_cake", "pink_shulker_box", "polished_blackstone_button", "poplar_button", "poplar_door", "poplar_fence", "poplar_fence_gate", "poplar_hanging_sign", "poplar_shelf", "poplar_sign", "poplar_trapdoor", "poplar_wall_hanging_sign", "poplar_wall_sign", "potted_acacia_sapling", "potted_allium", "potted_azalea_bush", "potted_azure_bluet", "potted_bamboo", "potted_birch_sapling", "potted_blue_orchid", "potted_brown_mushroom", "potted_cactus", "potted_cherry_sapling", "potted_closed_eyeblossom", "potted_cornflower", "potted_crimson_fungus", "potted_crimson_roots", "potted_dandelion", "potted_dark_oak_sapling", "potted_dead_bush", "potted_fern", "potted_flowering_azalea_bush", "potted_golden_dandelion", "potted_jungle_sapling", "potted_lily_of_the_valley", "potted_mangrove_propagule", "potted_oak_sapling", "potted_open_eyeblossom", "potted_orange_tulip", "potted_oxeye_daisy", "potted_pale_oak_sapling", "potted_pink_tulip", "potted_poplar_sapling", "potted_poppy", "potted_red_mushroom", "potted_red_tulip", "potted_spruce_sapling", "potted_torchflower", "potted_warped_fungus", "potted_warped_roots", "potted_white_tulip", "potted_wither_rose", "powder_snow_cauldron", "pumpkin", "purple_bed", "purple_candle", "purple_candle_cake", "purple_shulker_box", "red_bed", "red_candle", "red_candle_cake", "red_shulker_box", "redstone_ore", "redstone_wire", "repeater", "repeating_command_block", "respawn_anchor", "shulker_box", "smithing_table", "smoker", "soul_campfire", "spruce_button", "spruce_door", "spruce_fence", "spruce_fence_gate", "spruce_hanging_sign", "spruce_shelf", "spruce_sign", "spruce_trapdoor", "spruce_wall_hanging_sign", "spruce_wall_sign", "stone_button", "stonecutter", "straw_bed", "structure_block", "sweet_berry_bush", "test_block", "test_instance_block", "tnt", "trapped_chest", "vault", "warped_button", "warped_door", "warped_fence", "warped_fence_gate", "warped_hanging_sign", "warped_shelf", "warped_sign", "warped_trapdoor", "warped_wall_hanging_sign", "warped_wall_sign", "water_cauldron", "waxed_copper_chest", "waxed_copper_door", "waxed_copper_golem_statue", "waxed_copper_trapdoor", "waxed_exposed_copper_chest", "waxed_exposed_copper_door", "waxed_exposed_copper_golem_statue", "waxed_exposed_copper_trapdoor", "waxed_oxidized_copper_chest", "waxed_oxidized_copper_door", "waxed_oxidized_copper_golem_statue", "waxed_oxidized_copper_trapdoor", "waxed_weathered_copper_chest", "waxed_weathered_copper_door", "waxed_weathered_copper_golem_statue", "waxed_weathered_copper_trapdoor", "weathered_copper_chest", "weathered_copper_door", "weathered_copper_golem_statue", "weathered_copper_trapdoor", "white_bed", "white_candle", "white_candle_cake", "white_shulker_box", "yellow_bed", "yellow_candle", "yellow_candle_cake", "yellow_shulker_box"];

/// bukkit: org/bukkit/Bukkit.dispatchCommand, org/bukkit/Server.dispatchCommand
///
/// As `sender`: a plugin's command at once, whichever plugin owns it. False for no such command.
pub fn dispatch_command_as(sender: &CommandSender, line: &str) -> bool {
    CommandMap.dispatch(sender, line)
}

/// The id `getOfflinePlayer` gives someone never seen: Mojang's on an online server, else Bukkit's offline one.
pub(crate) fn uuid_of_stranger(name: &str) -> Option<String> {
    let mut profile = crate::extra::create_player_profile("", name);
    profile.complete_from_cache(true, crate::server::property("online-mode").is_none_or(|mode| mode != "false"));
    (!profile.id.is_empty()).then(|| profile.id.replace('-', ""))
}

/// `CraftWorld.dropItemNaturally`'s offset: a quarter block each way, and half an item's height down.
pub(crate) fn natural_spread() -> [f64; 3] {
    let random = || (fresh_bits() as u64 as f64) / (u64::MAX as f64) * 0.5 - 0.25;
    [random(), random() - 0.125, random()]
}

impl World {
    /// bukkit: org/bukkit/World.spawnParticle
    ///
    /// `force` shows it from 512 blocks, past a player's particle setting, as Java's does.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_particle_forced(&self, particle: &str, at: &Location, count: u32, dx: f64, dy: f64, dz: f64, speed: f64, force: bool) {
        let particle = crate::enums::particle::value_of(particle).unwrap_or(particle);
        run_command(&format!("execute in {} run particle {particle} {} {} {} {dx} {dy} {dz} {speed} {count} {}", self.dimension(), at.x, at.y, at.z, if force { "force" } else { "normal" }));
    }
    fn spawn_flags(&self) -> Value {
        query(json!({ "op": "core_spawn_flags", "dimension": self.dimension() }))
    }
    /// bukkit: org/bukkit/World.getAllowMonsters
    pub fn get_allow_monsters(&self) -> bool {
        self.spawn_flags()["monsters"].as_bool().unwrap_or_else(|| self.get_game_rule_value("spawn_monsters").is_none_or(|value| value != "0" && value != "false"))
    }
    /// bukkit: org/bukkit/World.getAllowAnimals
    pub fn get_allow_animals(&self) -> bool {
        self.spawn_flags()["animals"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/World.setSpawnFlags
    pub fn set_spawn_flags(&self, monsters: bool, animals: bool) {
        query(json!({ "op": "core_spawn_flags_set", "dimension": self.dimension(), "monsters": monsters, "animals": animals }));
    }
    /// bukkit: org/bukkit/World.setAllowMonsterSpawning
    pub fn set_allow_monster_spawning(&self, allow: bool) {
        self.set_spawn_flags(allow, self.get_allow_animals());
    }
    /// bukkit: org/bukkit/World.setSpawnLimit
    ///
    /// For this level; below 0 goes back to the server's.
    pub fn set_spawn_limit(&self, category: &str, limit: i32) {
        query(json!({ "op": "core_level_limit_set", "dimension": self.dimension(), "category": mob_category(category), "limit": limit }));
    }
    /// bukkit: org/bukkit/World.setMonsterSpawnLimit
    pub fn set_monster_spawn_limit(&self, limit: i32) {
        self.set_spawn_limit("MONSTER", limit);
    }
    /// bukkit: org/bukkit/World.setAnimalSpawnLimit
    pub fn set_animal_spawn_limit(&self, limit: i32) {
        self.set_spawn_limit("ANIMAL", limit);
    }
    /// bukkit: org/bukkit/World.setWaterAnimalSpawnLimit
    pub fn set_water_animal_spawn_limit(&self, limit: i32) {
        self.set_spawn_limit("WATER_ANIMAL", limit);
    }
    /// bukkit: org/bukkit/World.setWaterAmbientSpawnLimit
    pub fn set_water_ambient_spawn_limit(&self, limit: i32) {
        self.set_spawn_limit("WATER_AMBIENT", limit);
    }
    /// bukkit: org/bukkit/World.setWaterUndergroundCreatureSpawnLimit
    pub fn set_water_underground_creature_spawn_limit(&self, limit: i32) {
        self.set_spawn_limit("WATER_UNDERGROUND_CREATURE", limit);
    }
    /// bukkit: org/bukkit/World.setAmbientSpawnLimit
    pub fn set_ambient_spawn_limit(&self, limit: i32) {
        self.set_spawn_limit("AMBIENT", limit);
    }
}

/// bukkit: org/bukkit/Bukkit.setRespawnWorld, org/bukkit/Server.setRespawnWorld
pub fn set_respawn_world(world: &World) {
    query(json!({ "op": "core_set_respawn_world", "dimension": world.dimension() }));
}

impl World {
    /// The players here as `Entity`s, by their ids, in a box if one's given: what `getEntities` has.
    pub(crate) fn player_entities(&self, within: Option<(&Location, f64, f64, f64)>) -> Vec<Entity> {
        self.get_players()
            .iter()
            .filter(|player| {
                let at = player.get_location();
                within.is_none_or(|(center, x, y, z)| (at.x - center.x).abs() <= x && (at.y - center.y).abs() <= y && (at.z - center.z).abs() <= z)
            })
            .filter_map(|player| Entity::read(&json!({ "id": player.get_entity_id(), "dimension": self.dimension(), "uuid": player.uuid() })))
            .collect()
    }
}

impl Entity {
    /// Java's `entity instanceof Player`: the player this is, if it's one.
    pub fn as_player(&self) -> Option<Player> {
        Player::read(&query(json!({ "op": "player", "uuid": self.get_unique_id() })))
    }
}

impl World {
    /// bukkit~: org/bukkit/World.createExplosion | at once, and false if a listener stopped it; in the hooks that hold the hub themselves it waits for the end of the tick, and true means asked for
    pub fn create_explosion_with(&self, at: &Location, power: f32, fire: bool, break_blocks: bool) -> bool {
        query(json!({ "op": "core_explode", "dimension": self.dimension(), "at": [at.x, at.y, at.z], "power": power, "fire": fire, "breaks": break_blocks })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/World.createExplosion
    ///
    /// By coordinates: `createExplosion(x, y, z, power, setFire, breakBlocks)`.
    #[allow(clippy::too_many_arguments)]
    pub fn create_explosion_at(&self, x: f64, y: f64, z: f64, power: f32, fire: bool, break_blocks: bool) -> bool {
        query(json!({ "op": "core_explode", "dimension": self.dimension(), "at": [x, y, z], "power": power, "fire": fire, "breaks": break_blocks })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/World.createExplosion
    ///
    /// `createExplosion(x, y, z, power)`: no fire, blocks break, as Bukkit's default.
    pub fn create_explosion_xyz(&self, x: f64, y: f64, z: f64, power: f32) -> bool {
        self.create_explosion_at(x, y, z, power, false, true)
    }
    /// bukkit: org/bukkit/World.createExplosion
    ///
    /// `createExplosion(x, y, z, power, setFire)`: blocks break, as Bukkit's default.
    pub fn create_explosion_xyz_fire(&self, x: f64, y: f64, z: f64, power: f32, fire: bool) -> bool {
        self.create_explosion_at(x, y, z, power, fire, true)
    }
}

/// A component in MiniMessage's words, as `mini_message` reads them.
fn mini_written(value: &Value) -> String {
    fn escaped(text: &str) -> String {
        text.replace('\\', "\\\\").replace('<', "\\<")
    }
    match value {
        Value::String(text) => escaped(text),
        Value::Array(parts) => parts.iter().map(mini_written).collect(),
        Value::Object(fields) => {
            let mut open = Vec::new();
            if let Some(color) = fields.get("color").and_then(Value::as_str) {
                open.push((color.to_owned(), color.to_owned()));
            }
            for name in ["bold", "italic", "underlined", "strikethrough", "obfuscated"] {
                match fields.get(name).and_then(Value::as_bool) {
                    Some(true) => open.push((name.to_owned(), name.to_owned())),
                    Some(false) => open.push((format!("!{name}"), format!("!{name}"))),
                    None => {}
                }
            }
            if let Some(click) = fields.get("click_event") {
                let action = click["action"].as_str().unwrap_or_default();
                let target = click["command"].as_str().or(click["url"].as_str()).or(click["value"].as_str()).unwrap_or_default();
                open.push((format!("click:{action}:'{}'", target.replace('\'', "\\'")), "click".to_owned()));
            }
            if let Some(hover) = fields.get("hover_event").filter(|hover| hover["action"] == "show_text") {
                open.push((format!("hover:show_text:'{}'", mini_written(&hover["value"]).replace('\'', "\\'")), "hover".to_owned()));
            }
            if let Some(word) = fields.get("insertion").and_then(Value::as_str) {
                open.push((format!("insert:'{word}'"), "insert".to_owned()));
            }
            let mut out: String = open.iter().map(|(tag, _)| format!("<{tag}>")).collect();
            out += &match fields.get("translate").and_then(Value::as_str) {
                Some(key) => format!("<lang:{key}>"),
                None => fields.get("text").and_then(Value::as_str).map(escaped).unwrap_or_default(),
            };
            out += &fields.get("extra").map(mini_written).unwrap_or_default();
            for (_, name) in open.iter().rev() {
                out += &format!("</{name}>");
            }
            out
        }
        _ => String::new(),
    }
}

/// `org.bukkit.util.noise.PerlinNoiseGenerator`, seeded: its own shuffled table and offsets.
#[derive(Clone, Debug)]
pub struct SeededPerlinNoise {
    table: Vec<i32>,
    offset: [f64; 3],
}

impl SeededPerlinNoise {
    /// bukkit: org/bukkit/util/noise/PerlinNoiseGenerator.<init>
    pub fn new(random: &mut crate::worlds::JavaRandom) -> Self {
        let offset = [random.next_double() * 256.0, random.next_double() * 256.0, random.next_double() * 256.0];
        let mut table = vec![0; 512];
        for slot in table.iter_mut().take(256) {
            *slot = random.next_int_bound(256);
        }
        for index in 0..256 {
            let other = (random.next_int_bound(256 - index as i32) + index as i32) as usize;
            table.swap(index, other);
            table[index + 256] = table[index];
        }
        Self { table, offset }
    }
    /// bukkit: org/bukkit/util/noise/NoiseGenerator.noise
    pub fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        let (x, y, z) = (x + self.offset[0], y + self.offset[1], z + self.offset[2]);
        let (floor_x, floor_y, floor_z) = (crate::pure::noise_floor(x), crate::pure::noise_floor(y), crate::pure::noise_floor(z));
        let (cell_x, cell_y, cell_z) = (floor_x & 255, floor_y & 255, floor_z & 255);
        let (x, y, z) = (x - f64::from(floor_x), y - f64::from(floor_y), z - f64::from(floor_z));
        let (u, v, w) = (crate::pure::fade(x), crate::pure::fade(y), crate::pure::fade(z));
        let at = |index: i32| self.table[index as usize];
        let a = at(cell_x) + cell_y;
        let (aa, ab) = (at(a) + cell_z, at(a + 1) + cell_z);
        let b = at(cell_x + 1) + cell_y;
        let (ba, bb) = (at(b) + cell_z, at(b + 1) + cell_z);
        let grad = |index: i32, x: f64, y: f64, z: f64| crate::pure::grad(at(index) as u8, x, y, z);
        let lerp = crate::pure::lerp;
        lerp(
            w,
            lerp(v, lerp(u, grad(aa, x, y, z), grad(ba, x - 1.0, y, z)), lerp(u, grad(ab, x, y - 1.0, z), grad(bb, x - 1.0, y - 1.0, z))),
            lerp(v, lerp(u, grad(aa + 1, x, y, z - 1.0), grad(ba + 1, x - 1.0, y, z - 1.0)), lerp(u, grad(ab + 1, x, y - 1.0, z - 1.0), grad(bb + 1, x - 1.0, y - 1.0, z - 1.0))),
        )
    }
}

/// `org.bukkit.util.noise.PerlinOctaveGenerator`: a seeded generator an octave.
#[derive(Clone, Debug)]
pub struct PerlinOctaveGenerator {
    octaves: Vec<SeededPerlinNoise>,
    scale: [f64; 3],
}

impl PerlinOctaveGenerator {
    /// bukkit: org/bukkit/util/noise/PerlinOctaveGenerator.<init>
    pub fn new(seed: i64, octaves: usize) -> Self {
        let mut random = crate::worlds::JavaRandom::new(seed);
        Self { octaves: (0..octaves).map(|_| SeededPerlinNoise::new(&mut random)).collect(), scale: [1.0; 3] }
    }
    /// `setXScale`, `setYScale` and `setZScale` together.
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = [scale; 3];
    }
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.noise
    pub fn noise(&self, x: f64, y: f64, z: f64, frequency: f64, amplitude: f64, normalized: bool) -> f64 {
        let (x, y, z) = (x * self.scale[0], y * self.scale[1], z * self.scale[2]);
        let (mut total, mut freq, mut amp, mut most) = (0.0, 1.0, 1.0, 0.0);
        for octave in &self.octaves {
            total += octave.noise(x * freq, y * freq, z * freq) * amp;
            most += amp;
            freq *= frequency;
            amp *= amplitude;
        }
        if normalized { total / most } else { total }
    }
}
