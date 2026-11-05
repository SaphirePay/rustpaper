//! `org.bukkit.WorldCreator` and `org.bukkit.generator`: worlds beyond the three, and terrain from a plugin.
//!
//! A generator runs on the game's chunk workers, as on Paper: it may read what it's given and nothing of the live
//! game. A populator runs on the server thread once the chunk stands, so it can write through its `LimitedRegion`.
use crate::block_data::BlockData;
use crate::fam_core::WorldInfo;
use crate::{Location, Material, World};
use horizon_mod::{Answer, Host, Value, query};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// `org.bukkit.World.Environment`.
fn environment_of(world: &World) -> String {
    let dimension = world.dimension().to_owned();
    match crate::world_of(&dimension) {
        "world_nether" => "NETHER".to_owned(),
        "world_the_end" => "THE_END".to_owned(),
        "world" if dimension == "minecraft:overworld" => "NORMAL".to_owned(),
        _ => created().iter().find(|row| row["id"] == dimension.as_str()).and_then(|row| row["environment"].as_str().map(str::to_owned)).unwrap_or_else(|| "NORMAL".to_owned()),
    }
}

/// What the game has open of created worlds. Each plugin has its own statics, so the game keeps this.
pub(crate) fn created() -> Vec<Value> {
    query(json!({ "op": "world_list" })).as_array().cloned().unwrap_or_default()
}

/// A created world's dimension: `None` for a name that isn't one.
pub(crate) fn dimension_of_created(name: &str) -> Option<&'static str> {
    static KNOWN: Mutex<Option<HashMap<String, &'static str>>> = Mutex::new(None);
    let mut known = KNOWN.lock().unwrap_or_else(|e| e.into_inner());
    let known = known.get_or_insert_default();
    if let Some(found) = known.get(name) {
        return Some(found);
    }
    let row = created().into_iter().find(|row| row["name"] == name || row["id"] == name)?;
    let id: &'static str = Box::leak(row["id"].as_str()?.to_owned().into_boxed_str());
    known.insert(name.to_owned(), id);
    Some(id)
}

/// A created world's name, from its dimension.
pub(crate) fn name_of_created(dimension: &str) -> Option<&'static str> {
    static KNOWN: Mutex<Option<HashMap<String, &'static str>>> = Mutex::new(None);
    let mut known = KNOWN.lock().unwrap_or_else(|e| e.into_inner());
    let known = known.get_or_insert_default();
    if let Some(found) = known.get(dimension) {
        return Some(found);
    }
    let row = created().into_iter().find(|row| row["id"] == dimension)?;
    let name: &'static str = Box::leak(row["name"].as_str()?.to_owned().into_boxed_str());
    known.insert(dimension.to_owned(), name);
    Some(name)
}

static GENERATORS: Mutex<Option<HashMap<String, Arc<dyn ChunkGenerator>>>> = Mutex::new(None);
static PROVIDERS: Mutex<Option<HashMap<String, Arc<dyn BiomeProvider>>>> = Mutex::new(None);

fn generator_of(id: &str) -> Option<Arc<dyn ChunkGenerator>> {
    GENERATORS.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(id).cloned()
}

fn provider_of(id: &str) -> Option<Arc<dyn BiomeProvider>> {
    PROVIDERS.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(id).cloned()
}

/// `org.bukkit.WorldType`.
pub mod world_type {
    pub const NORMAL: &str = "NORMAL";
    pub const FLAT: &str = "FLAT";
    pub const LARGE_BIOMES: &str = "LARGE_BIOMES";
    pub const AMPLIFIED: &str = "AMPLIFIED";
}

/// bukkit: org/bukkit/WorldCreator
///
/// What a world is to be made from.
#[derive(Clone)]
pub struct WorldCreator {
    name: String,
    key: String,
    seed: i64,
    seeded: bool,
    environment: String,
    kind: String,
    generate_structures: bool,
    hardcore: bool,
    bonus_chest: bool,
    keep_spawn_loaded: Option<bool>,
    settings: String,
    generator: Option<Arc<dyn ChunkGenerator>>,
    provider: Option<Arc<dyn BiomeProvider>>,
    forced_spawn: Option<(Location, f32, f32)>,
}

impl WorldCreator {
    /// bukkit: org/bukkit/WorldCreator.ofNameAndKey, org/bukkit/WorldCreator.name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            key: format!("minecraft:{}", name.to_lowercase()),
            seed: 0,
            seeded: false,
            environment: "NORMAL".to_owned(),
            kind: world_type::NORMAL.to_owned(),
            generate_structures: true,
            hardcore: false,
            bonus_chest: false,
            keep_spawn_loaded: None,
            settings: String::new(),
            generator: None,
            provider: None,
            forced_spawn: None,
        }
    }
    /// bukkit: org/bukkit/WorldCreator.ofKey
    pub fn of_key(key: &str) -> Self {
        let mut creator = Self::new(key.rsplit(':').next().unwrap_or(key));
        creator.key = key.to_owned();
        creator
    }
    /// bukkit: org/bukkit/WorldCreator.ofNameAndKey
    pub fn of_name_and_key(name: &str, key: &str) -> Self {
        let mut creator = Self::new(name);
        creator.key = key.to_owned();
        creator
    }
    /// bukkit: org/bukkit/WorldCreator.copy
    pub fn copy(&mut self, world: &World) -> &mut Self {
        self.seed = world.get_seed();
        self.seeded = true;
        self.environment = environment_of(world);
        self.generator = world.get_generator();
        self.provider = world.get_biome_provider();
        self
    }
    /// bukkit: org/bukkit/WorldCreator.name
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/WorldCreator.key
    pub fn key(&self) -> &str {
        &self.key
    }
    /// bukkit: org/bukkit/WorldCreator.seed
    pub fn seed(&mut self, seed: i64) -> &mut Self {
        self.seed = seed;
        self.seeded = true;
        self
    }
    /// bukkit: org/bukkit/WorldCreator.seed
    pub fn get_seed(&self) -> i64 {
        self.seed
    }
    /// bukkit: org/bukkit/WorldCreator.environment
    pub fn environment(&mut self, environment: &str) -> &mut Self {
        self.environment = environment.to_owned();
        self
    }
    /// bukkit: org/bukkit/WorldCreator.environment
    pub fn get_environment(&self) -> &str {
        &self.environment
    }
    /// bukkit: org/bukkit/WorldCreator.type
    pub fn world_type(&mut self, kind: &str) -> &mut Self {
        self.kind = kind.to_owned();
        self
    }
    /// bukkit: org/bukkit/WorldCreator.type
    pub fn get_type(&self) -> &str {
        &self.kind
    }
    /// bukkit: org/bukkit/WorldCreator.generator
    pub fn generator(&mut self, generator: Arc<dyn ChunkGenerator>) -> &mut Self {
        self.generator = Some(generator);
        self
    }
    /// bukkit: org/bukkit/WorldCreator.generator
    pub fn get_generator(&self) -> Option<Arc<dyn ChunkGenerator>> {
        self.generator.clone()
    }
    /// bukkit: org/bukkit/WorldCreator.biomeProvider
    pub fn biome_provider(&mut self, provider: Arc<dyn BiomeProvider>) -> &mut Self {
        self.provider = Some(provider);
        self
    }
    /// bukkit: org/bukkit/WorldCreator.biomeProvider
    pub fn get_biome_provider(&self) -> Option<Arc<dyn BiomeProvider>> {
        self.provider.clone()
    }
    /// bukkit: org/bukkit/WorldCreator.generatorSettings
    ///
    /// For a FLAT world, the JSON of its settings: `{"layers":[{"block":"stone","height":3}],"biome":"plains"}`.
    pub fn generator_settings(&mut self, settings: &str) -> &mut Self {
        self.settings = settings.to_owned();
        self
    }
    /// bukkit: org/bukkit/WorldCreator.generatorSettings
    pub fn get_generator_settings(&self) -> &str {
        &self.settings
    }
    /// bukkit: org/bukkit/WorldCreator.generateStructures
    pub fn generate_structures(&mut self, generate: bool) -> &mut Self {
        self.generate_structures = generate;
        self
    }
    /// bukkit: org/bukkit/WorldCreator.generateStructures
    pub fn get_generate_structures(&self) -> bool {
        self.generate_structures
    }
    /// bukkit: org/bukkit/WorldCreator.hardcore
    pub fn hardcore(&mut self, hardcore: bool) -> &mut Self {
        self.hardcore = hardcore;
        self
    }
    /// bukkit: org/bukkit/WorldCreator.hardcore
    pub fn get_hardcore(&self) -> bool {
        self.hardcore
    }
    /// bukkit: org/bukkit/WorldCreator.bonusChest
    pub fn bonus_chest(&mut self, bonus_chest: bool) -> &mut Self {
        self.bonus_chest = bonus_chest;
        self
    }
    /// bukkit: org/bukkit/WorldCreator.bonusChest
    pub fn get_bonus_chest(&self) -> bool {
        self.bonus_chest
    }
    /// bukkit: org/bukkit/WorldCreator.keepSpawnLoaded
    pub fn keep_spawn_loaded(&mut self, keep: Option<bool>) -> &mut Self {
        self.keep_spawn_loaded = keep;
        self
    }
    /// bukkit: org/bukkit/WorldCreator.keepSpawnLoaded
    pub fn get_keep_spawn_loaded(&self) -> Option<bool> {
        self.keep_spawn_loaded
    }
    /// bukkit: org/bukkit/WorldCreator.forcedSpawnPosition
    pub fn forced_spawn_position(&mut self, at: Location, yaw: f32, pitch: f32) -> &mut Self {
        self.forced_spawn = Some((at, yaw, pitch));
        self
    }
    /// bukkit: org/bukkit/WorldCreator.clearForcedSpawnPosition
    pub fn clear_forced_spawn_position(&mut self) -> &mut Self {
        self.forced_spawn = None;
        self
    }
    /// bukkit: org/bukkit/WorldCreator.forcedSpawnYaw
    pub fn forced_spawn_yaw(&self) -> Option<f32> {
        self.forced_spawn.as_ref().map(|spawn| spawn.1)
    }
    /// bukkit: org/bukkit/WorldCreator.forcedSpawnPitch
    pub fn forced_spawn_pitch(&self) -> Option<f32> {
        self.forced_spawn.as_ref().map(|spawn| spawn.2)
    }
    /// bukkit: org/bukkit/WorldCreator.createWorld, org/bukkit/Bukkit.createWorld, org/bukkit/Server.createWorld
    pub fn create_world(&self) -> Option<World> {
        let wants = |ask: fn(&dyn ChunkGenerator) -> bool| self.generator.as_deref().is_none_or(ask);
        let (noise, surface, caves) = (wants(|generator| generator.should_generate_noise()), wants(|generator| generator.should_generate_surface()), wants(|generator| generator.should_generate_caves()));
        let (decorations, structures) = (wants(|generator| generator.should_generate_decorations()), wants(|generator| generator.should_generate_structures()));
        let vanilla = self.generator.is_some() && (noise || surface || caves || decorations || wants(|generator| generator.should_generate_mobs()) || structures);
        let left_out = u8::from(!noise) | u8::from(!surface) << 1 | u8::from(!caves) << 2;
        let provider = self.provider.clone().or_else(|| self.generator.as_ref().and_then(|generator| generator.get_default_biome_provider()));
        let generated = self.generator.is_some() || provider.is_some();
        let answer = query(json!({
            "op": "world_create", "name": self.name, "id": self.key, "environment": self.environment, "seed": self.seeded.then_some(self.seed),
            "flat": self.kind == world_type::FLAT, "kind": self.kind, "settings": self.settings, "provider": provider.is_some(), "hardcore": self.hardcore, "mobs": wants(|generator| generator.should_generate_mobs()), "structures": self.generate_structures && structures, "decorations": decorations, "stepped": vanilla, "left_out": left_out, "generated": generated, "vanilla": vanilla || (self.generator.is_none() && generated),
        }));
        answer["id"].as_str()?;
        if let Some(generator) = &self.generator {
            match generator.named() {
                // Another plugin's: it runs there, so it's told which world it has.
                Some((plugin, id)) => {
                    ask_plugin(&plugin, "bind_generator", &id, &self.key);
                }
                None => {
                    GENERATORS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(self.key.clone(), generator.clone());
                }
            }
        }
        if let Some(provider) = provider {
            match provider.named() {
                Some((plugin, id)) => {
                    ask_plugin(&plugin, "bind_provider", &id, &self.key);
                }
                None => {
                    PROVIDERS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(self.key.clone(), provider);
                }
            }
        }
        let world = World::of(&self.key);
        if let Some((at, yaw, pitch)) = &self.forced_spawn {
            let mut at = at.clone();
            at.yaw = *yaw;
            at.pitch = *pitch;
            world.set_spawn_location(&at);
        }
        if self.keep_spawn_loaded == Some(true) {
            let spawn = world.get_spawn_location();
            let (x, z) = (spawn.x.floor() as i32 >> 4, spawn.z.floor() as i32 >> 4);
            for dx in -2..=2 {
                for dz in -2..=2 {
                    world.add_plugin_chunk_ticket(x + dx, z + dz);
                }
            }
        }
        if self.bonus_chest {
            let spawn = world.get_spawn_location();
            crate::run_command(&format!("execute in {} positioned {} {} {} run place feature minecraft:bonus_chest", self.key, spawn.x.floor(), world.get_highest_block_y_at(spawn.x.floor() as i32, spawn.z.floor() as i32) + 1, spawn.z.floor()));
        }
        Some(world)
    }
}

static NAMED: Mutex<Option<HashMap<String, Arc<dyn ChunkGenerator>>>> = Mutex::new(None);
static NAMED_PROVIDERS: Mutex<Option<HashMap<String, Arc<dyn BiomeProvider>>>> = Mutex::new(None);

/// What Bukkit asks `Plugin.getDefaultWorldGenerator` for: a generator this plugin answers to, by its id.
pub fn register_default_generator(id: &str, generator: Arc<dyn ChunkGenerator>) {
    NAMED.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(id.to_owned(), generator);
}

/// What Bukkit asks `Plugin.getDefaultBiomeProvider` for: a provider this plugin answers to, by its id.
pub fn register_default_biome_provider(id: &str, provider: Arc<dyn BiomeProvider>) {
    NAMED_PROVIDERS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(id.to_owned(), provider);
}

/// One plugin asking another about a generator or provider: a Rust type can't cross libraries, but the work can stay where it is.
fn ask_plugin(plugin: &str, kind: &str, id: &str, world: &str) -> Value {
    query(json!({ "op": "core_command_call", "plugin": plugin, "kind": kind, "id": id, "world": world }))
}

/// The other end of `ask_plugin`.
pub(crate) fn answer_named(value: &Value) -> Value {
    let (id, world) = (value["id"].as_str().unwrap_or_default(), value["world"].as_str().unwrap_or_default());
    let generator = || NAMED.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(id).cloned());
    let provider = || NAMED_PROVIDERS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(id).cloned());
    match value["kind"].as_str() {
        Some("generator") => generator().map_or(Value::Null, |generator| {
            json!({ "noise": generator.should_generate_noise(), "surface": generator.should_generate_surface(), "caves": generator.should_generate_caves(), "decorations": generator.should_generate_decorations(), "mobs": generator.should_generate_mobs(), "structures": generator.should_generate_structures() })
        }),
        Some("provider") => provider().map_or(Value::Null, |_| json!(true)),
        Some("bind_generator") => generator().map_or(json!(false), |generator| {
            GENERATORS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(world.to_owned(), generator);
            json!(true)
        }),
        _ => provider().map_or(json!(false), |provider| {
            PROVIDERS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(world.to_owned(), provider);
            json!(true)
        }),
    }
}

/// A generator that lives in another plugin: what it wants of the game, and whose it is.
struct NamedGenerator {
    plugin: String,
    id: String,
    wants: Value,
}

impl ChunkGenerator for NamedGenerator {
    fn named(&self) -> Option<(String, String)> {
        Some((self.plugin.clone(), self.id.clone()))
    }
    fn should_generate_noise(&self) -> bool {
        self.wants["noise"] == true
    }
    fn should_generate_surface(&self) -> bool {
        self.wants["surface"] == true
    }
    fn should_generate_caves(&self) -> bool {
        self.wants["caves"] == true
    }
    fn should_generate_decorations(&self) -> bool {
        self.wants["decorations"] == true
    }
    fn should_generate_mobs(&self) -> bool {
        self.wants["mobs"] == true
    }
    fn should_generate_structures(&self) -> bool {
        self.wants["structures"] == true
    }
}

/// A provider that lives in another plugin: only `create_world` reads this, and the plugin does the work.
struct NamedProvider {
    plugin: String,
    id: String,
}

impl BiomeProvider for NamedProvider {
    fn named(&self) -> Option<(String, String)> {
        Some((self.plugin.clone(), self.id.clone()))
    }
    fn get_biome(&self, _world: &GeneratingWorld, _x: i32, _y: i32, _z: i32) -> String {
        "minecraft:plains".to_owned()
    }
    fn get_biomes(&self, _world: &GeneratingWorld) -> Vec<String> {
        Vec::new()
    }
}

/// bukkit: org/bukkit/WorldCreator.getGeneratorForName
///
/// `Plugin:id`, in any plugin. Another plugin's stands in here for `WorldCreator::generator`, and runs in that plugin.
pub fn get_generator_for_name(name: &str) -> Option<Arc<dyn ChunkGenerator>> {
    let (plugin, id) = name.split_once(':').unwrap_or((name, ""));
    if crate::fam_core::get_own_plugin().is_some_and(|own| own.get_name().eq_ignore_ascii_case(plugin)) {
        return NAMED.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(id).cloned();
    }
    let wants = ask_plugin(plugin, "generator", id, "");
    (!wants.is_null()).then(|| Arc::new(NamedGenerator { plugin: plugin.to_owned(), id: id.to_owned(), wants }) as Arc<dyn ChunkGenerator>)
}

/// bukkit: org/bukkit/WorldCreator.getBiomeProviderForName
pub fn get_biome_provider_for_name(_world: &str, name: &str, _output: &crate::CommandSender) -> Option<Arc<dyn BiomeProvider>> {
    let (plugin, id) = name.split_once(':').unwrap_or((name, ""));
    if crate::fam_core::get_own_plugin().is_some_and(|own| own.get_name().eq_ignore_ascii_case(plugin)) {
        return NAMED_PROVIDERS.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(id).cloned();
    }
    (ask_plugin(plugin, "provider", id, "") == true).then(|| Arc::new(NamedProvider { plugin: plugin.to_owned(), id: id.to_owned() }) as Arc<dyn BiomeProvider>)
}

impl crate::Plugin {
    /// bukkit: org/bukkit/plugin/Plugin.getDefaultWorldGenerator, org/bukkit/plugin/java/JavaPlugin.getDefaultWorldGenerator
    ///
    /// What the plugin gave `register_default_generator` for this id: Rust has no override of this method.
    pub fn get_default_world_generator(&self, _world: &str, id: &str) -> Option<Arc<dyn ChunkGenerator>> {
        NAMED.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(id).cloned()
    }
    /// bukkit: org/bukkit/plugin/Plugin.getDefaultBiomeProvider, org/bukkit/plugin/java/JavaPlugin.getDefaultBiomeProvider
    pub fn get_default_biome_provider(&self, _world: &str, id: &str) -> Option<Arc<dyn BiomeProvider>> {
        NAMED_PROVIDERS.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(id).cloned()
    }
}

/// bukkit: org/bukkit/Bukkit.createChunkData, org/bukkit/Server.createChunkData
pub fn create_chunk_data(world: &World) -> ChunkData {
    let min_y = world.get_min_height();
    ChunkData::new(min_y, world.get_max_height() - min_y)
}

/// bukkit: org/bukkit/Bukkit.createWorld, org/bukkit/Server.createWorld
pub fn create_world(creator: &WorldCreator) -> Option<World> {
    creator.create_world()
}

/// bukkit: org/bukkit/Bukkit.unloadWorld, org/bukkit/Server.unloadWorld
///
/// False for the three levels, and for a world with a player in it, as on Bukkit.
pub fn unload_world(name: &str, save: bool) -> bool {
    let Some(id) = dimension_of_created(name) else {
        return false;
    };
    let done = query(json!({ "op": "world_unload", "id": id, "save": save })).as_bool().unwrap_or(false);
    if done {
        GENERATORS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().remove(id);
        PROVIDERS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().remove(id);
    }
    done
}

/// `java.util.Random`, as a generator is handed: the same numbers for the same seed.
#[derive(Clone, Debug)]
pub struct JavaRandom {
    seed: i64,
}

impl JavaRandom {
    const MULTIPLIER: i64 = 0x5_DEEC_E66D;
    const MASK: i64 = (1 << 48) - 1;

    pub fn new(seed: i64) -> Self {
        Self { seed: (seed ^ Self::MULTIPLIER) & Self::MASK }
    }
    pub fn set_seed(&mut self, seed: i64) {
        *self = Self::new(seed);
    }
    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(0xB) & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }
    pub fn next_int(&mut self) -> i32 {
        self.next(32)
    }
    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        if bound & (bound - 1) == 0 {
            return ((i64::from(bound) * i64::from(self.next(31))) >> 31) as i32;
        }
        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }
    pub fn next_long(&mut self) -> i64 {
        (i64::from(self.next(32)) << 32).wrapping_add(i64::from(self.next(32)))
    }
    pub fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }
    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1 << 24) as f32
    }
    pub fn next_double(&mut self) -> f64 {
        ((i64::from(self.next(26)) << 27) + i64::from(self.next(27))) as f64 / (1_i64 << 53) as f64
    }
}

/// bukkit: org/bukkit/generator/WorldInfo
///
/// What a generator knows of its world: nothing here asks the game.
#[derive(Clone, Debug)]
pub struct GeneratingWorld {
    name: String,
    seed: i64,
    environment: String,
    min_height: i32,
    max_height: i32,
}

impl GeneratingWorld {
    /// bukkit: org/bukkit/generator/WorldInfo.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/generator/WorldInfo.getEnvironment
    pub fn get_environment(&self) -> &str {
        &self.environment
    }
    /// bukkit: org/bukkit/generator/WorldInfo.getMinHeight
    pub fn get_min_height(&self) -> i32 {
        self.min_height
    }
    /// bukkit: org/bukkit/generator/WorldInfo.getMaxHeight
    pub fn get_max_height(&self) -> i32 {
        self.max_height
    }
    /// bukkit: org/bukkit/generator/WorldInfo.vanillaBiomeProvider
    ///
    /// The game's own biomes of this world: for a `BiomeProvider` to fall back on, while a chunk generates.
    pub fn vanilla_biome_provider(&self) -> Arc<dyn BiomeProvider> {
        Arc::new(VanillaBiomes)
    }
}

/// `WorldInfo.vanillaBiomeProvider`.
struct VanillaBiomes;

thread_local! {
    /// The place the game is asking a provider about, and the game's own biome there.
    static ASKED: std::cell::RefCell<Option<([i32; 3], String)>> = const { std::cell::RefCell::new(None) };
}

impl BiomeProvider for VanillaBiomes {
    fn get_biome(&self, _world: &GeneratingWorld, x: i32, y: i32, z: i32) -> String {
        let known = ASKED.with(|asked| asked.borrow().as_ref().filter(|(quart, _)| *quart == [x >> 2, y >> 2, z >> 2]).map(|(_, biome)| biome.clone()));
        if let Some(biome) = known {
            return biome;
        }
        query(json!({ "op": "world_biomes", "quarts": [[x >> 2, y >> 2, z >> 2]] }))[0].as_str().unwrap_or("minecraft:plains").to_owned()
    }
    fn get_biomes(&self, _world: &GeneratingWorld) -> Vec<String> {
        query(json!({ "op": "world_biomes", "all": true })).as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect()
    }
}

/// bukkit: org/bukkit/generator/BiomeParameterPoint
///
/// The game's climate where a biome is asked for. On Paper each `get_min_` and `get_max_` is the value itself,
/// read from a live server on every kind of world, so these are too.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BiomeParameterPoint {
    values: [f64; 6],
}

impl BiomeParameterPoint {
    fn at(point: &Value) -> Self {
        let mut values = [0.0; 6];
        for (index, value) in values.iter_mut().enumerate() {
            *value = point[index].as_f64().unwrap_or(0.0) / 10_000.0;
        }
        Self { values }
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getTemperature
    pub fn get_temperature(&self) -> f64 {
        self.values[0]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getHumidity
    pub fn get_humidity(&self) -> f64 {
        self.values[1]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getContinentalness
    pub fn get_continentalness(&self) -> f64 {
        self.values[2]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getErosion
    pub fn get_erosion(&self) -> f64 {
        self.values[3]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getDepth
    pub fn get_depth(&self) -> f64 {
        self.values[4]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getWeirdness
    pub fn get_weirdness(&self) -> f64 {
        self.values[5]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMinTemperature
    pub fn get_min_temperature(&self) -> f64 {
        self.values[0]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMaxTemperature
    pub fn get_max_temperature(&self) -> f64 {
        self.values[0]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMinHumidity
    pub fn get_min_humidity(&self) -> f64 {
        self.values[1]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMaxHumidity
    pub fn get_max_humidity(&self) -> f64 {
        self.values[1]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMinContinentalness
    pub fn get_min_continentalness(&self) -> f64 {
        self.values[2]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMaxContinentalness
    pub fn get_max_continentalness(&self) -> f64 {
        self.values[2]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMinErosion
    pub fn get_min_erosion(&self) -> f64 {
        self.values[3]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMaxErosion
    pub fn get_max_erosion(&self) -> f64 {
        self.values[3]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMinDepth
    pub fn get_min_depth(&self) -> f64 {
        self.values[4]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMaxDepth
    pub fn get_max_depth(&self) -> f64 {
        self.values[4]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMinWeirdness
    pub fn get_min_weirdness(&self) -> f64 {
        self.values[5]
    }
    /// bukkit: org/bukkit/generator/BiomeParameterPoint.getMaxWeirdness
    pub fn get_max_weirdness(&self) -> f64 {
        self.values[5]
    }
}

impl WorldInfo for GeneratingWorld {
    fn get_seed(&self) -> i64 {
        self.seed
    }
    fn get_uid(&self) -> String {
        World::of(&self.name).get_uid()
    }
}

/// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData
///
/// One chunk's blocks as a plugin writes them. Blocks it leaves alone stay as the game made them.
pub struct ChunkData {
    min_y: i32,
    height: i32,
    palette: Vec<String>,
    cells: Vec<u16>,
    biomes: Vec<String>,
    biome_cells: Vec<u16>,
}

impl ChunkData {
    fn new(min_y: i32, height: i32) -> Self {
        Self { min_y, height, palette: vec![String::new()], cells: vec![0; 256 * height.max(0) as usize], biomes: Vec::new(), biome_cells: Vec::new() }
    }
    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        ((0..16).contains(&x) && (0..16).contains(&z) && y >= self.min_y && y < self.min_y + self.height).then(|| ((y - self.min_y) * 256 + z * 16 + x) as usize)
    }
    fn state(&mut self, text: &str) -> u16 {
        match self.palette.iter().position(|known| known == text) {
            Some(index) => index as u16,
            None => {
                self.palette.push(text.to_owned());
                (self.palette.len() - 1) as u16
            }
        }
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.getMinHeight
    pub fn get_min_height(&self) -> i32 {
        self.min_y
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.getMaxHeight
    pub fn get_max_height(&self) -> i32 {
        self.min_y + self.height
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.setBlock
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, material: &Material) {
        let state = self.state(material.key());
        if let Some(index) = self.index(x, y, z) {
            self.cells[index] = state;
        }
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.setBlock
    pub fn set_block_data(&mut self, x: i32, y: i32, z: i32, data: &BlockData) {
        let state = self.state(&data.get_as_string());
        if let Some(index) = self.index(x, y, z) {
            self.cells[index] = state;
        }
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.setRegion
    ///
    /// From the first corner to before the second, as Bukkit's.
    pub fn set_region(&mut self, from: [i32; 3], to: [i32; 3], material: &Material) {
        let state = self.state(material.key());
        for y in from[1]..to[1] {
            for z in from[2]..to[2] {
                for x in from[0]..to[0] {
                    if let Some(index) = self.index(x, y, z) {
                        self.cells[index] = state;
                    }
                }
            }
        }
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.setRegion
    pub fn set_region_data(&mut self, from: [i32; 3], to: [i32; 3], data: &BlockData) {
        let state = self.state(&data.get_as_string());
        for y in from[1]..to[1] {
            for z in from[2]..to[2] {
                for x in from[0]..to[0] {
                    if let Some(index) = self.index(x, y, z) {
                        self.cells[index] = state;
                    }
                }
            }
        }
    }
    fn text(&self, x: i32, y: i32, z: i32) -> &str {
        self.index(x, y, z).map_or("", |index| &self.palette[usize::from(self.cells[index])])
    }
    /// What's there: what this generator wrote, else what the game's steps so far have made.
    fn read(&self, x: i32, y: i32, z: i32) -> String {
        let text = self.text(x, y, z);
        if !text.is_empty() {
            return text.to_owned();
        }
        query(json!({ "op": "world_proto_block", "x": x, "y": y, "z": z })).as_str().unwrap_or("minecraft:air").to_owned()
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.getType
    pub fn get_type(&self, x: i32, y: i32, z: i32) -> Material {
        let text = self.read(x, y, z);
        Material::of(text.split('[').next().unwrap_or(&text))
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.getBlockData
    pub fn get_block_data(&self, x: i32, y: i32, z: i32) -> BlockData {
        BlockData::parse(&self.read(x, y, z))
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.getBiome
    ///
    /// What this world's `BiomeProvider` said for the cell, by name: plains with no provider.
    pub fn get_biome(&self, x: i32, y: i32, z: i32) -> String {
        let section = ((y - self.min_y).max(0) / 16) as usize;
        let index = section * 64 + ((y & 15) / 4) as usize * 16 + ((z & 15) / 4) as usize * 4 + ((x & 15) / 4) as usize;
        self.biome_cells.get(index).and_then(|cell| self.biomes.get(usize::from(*cell))).cloned().unwrap_or_else(|| "minecraft:plains".to_owned())
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.getData
    ///
    /// The 1.12 data value of what's there, from the game's own flattening table.
    pub fn get_data(&self, x: i32, y: i32, z: i32) -> u8 {
        query(json!({ "op": "world_legacy_data", "block": self.read(x, y, z) })).as_u64().unwrap_or(0) as u8
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$ChunkData.getHeight
    ///
    /// By `height_map`, over what the game's steps have made so far and what this generator wrote since.
    pub fn get_height(&self, height_map: &str, x: i32, z: i32) -> i32 {
        let map = match height_map {
            "OCEAN_FLOOR" | "OCEAN_FLOOR_WG" => 1,
            "MOTION_BLOCKING" => 2,
            "MOTION_BLOCKING_NO_LEAVES" => 3,
            _ => 0,
        };
        let game = query(json!({ "op": "world_proto_height", "map": map, "x": x, "z": z })).as_i64().map_or(self.min_y, |height| height as i32);
        let written = (self.min_y..self.min_y + self.height).rev().find(|y| {
            let text = self.text(x, *y, z);
            !text.is_empty() && !text.ends_with("air")
        });
        written.map_or(game, |written| written.max(game))
    }
    fn runs(cells: &[u16]) -> Value {
        let mut runs: Vec<[u64; 2]> = Vec::new();
        for &cell in cells {
            match runs.last_mut() {
                Some(run) if run[0] == u64::from(cell) => run[1] += 1,
                _ => runs.push([u64::from(cell), 1]),
            }
        }
        json!(runs)
    }
}

/// bukkit: org/bukkit/generator/ChunkGenerator$BiomeGrid
///
/// The old way to set a chunk's biomes, whole columns at a time, for `generate_chunk_data`.
pub struct BiomeGrid {
    columns: Vec<Option<String>>,
}

impl BiomeGrid {
    /// bukkit: org/bukkit/generator/ChunkGenerator$BiomeGrid.getBiome
    pub fn get_biome(&self, x: i32, z: i32) -> Option<&str> {
        self.columns.get(((z & 15) * 16 + (x & 15)) as usize)?.as_deref()
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator$BiomeGrid.setBiome
    pub fn set_biome(&mut self, x: i32, z: i32, biome: &str) {
        if let Some(column) = self.columns.get_mut(((z & 15) * 16 + (x & 15)) as usize) {
            *column = Some(biome.to_owned());
        }
    }
}

/// bukkit: org/bukkit/generator/ChunkGenerator
pub trait ChunkGenerator: Send + Sync {
    /// Whose this is, when it lives in another plugin: `get_generator_for_name` gives those.
    #[doc(hidden)]
    fn named(&self) -> Option<(String, String)> {
        None
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.generateChunkData
    ///
    /// The old whole-chunk way, run after the four steps when a generator overrides it.
    fn generate_chunk_data(&self, _world: &GeneratingWorld, _random: &mut JavaRandom, _chunk_x: i32, _chunk_z: i32, _biomes: &mut BiomeGrid, _data: &mut ChunkData) {}
    /// bukkit: org/bukkit/generator/ChunkGenerator.generateNoise
    fn generate_noise(&self, _world: &GeneratingWorld, _random: &mut JavaRandom, _chunk_x: i32, _chunk_z: i32, _data: &mut ChunkData) {}
    /// bukkit: org/bukkit/generator/ChunkGenerator.generateSurface
    fn generate_surface(&self, _world: &GeneratingWorld, _random: &mut JavaRandom, _chunk_x: i32, _chunk_z: i32, _data: &mut ChunkData) {}
    /// bukkit: org/bukkit/generator/ChunkGenerator.generateBedrock
    fn generate_bedrock(&self, _world: &GeneratingWorld, _random: &mut JavaRandom, _chunk_x: i32, _chunk_z: i32, _data: &mut ChunkData) {}
    /// bukkit: org/bukkit/generator/ChunkGenerator.generateCaves
    fn generate_caves(&self, _world: &GeneratingWorld, _random: &mut JavaRandom, _chunk_x: i32, _chunk_z: i32, _data: &mut ChunkData) {}
    /// bukkit: org/bukkit/generator/ChunkGenerator.getDefaultBiomeProvider
    fn get_default_biome_provider(&self) -> Option<Arc<dyn BiomeProvider>> {
        None
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.getBaseHeight
    fn get_base_height(&self, _world: &GeneratingWorld, _random: &mut JavaRandom, _x: i32, _z: i32, _height_map: &str) -> i32 {
        0
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.canSpawn
    fn can_spawn(&self, _world: &World, _x: i32, _z: i32) -> bool {
        true
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.getDefaultPopulators
    fn get_default_populators(&self, _world: &World) -> Vec<Arc<dyn BlockPopulator>> {
        Vec::new()
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.getFixedSpawnLocation
    fn get_fixed_spawn_location(&self, _world: &World, _random: &mut JavaRandom) -> Option<Location> {
        None
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.isParallelCapable
    fn is_parallel_capable(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.shouldGenerateNoise
    fn should_generate_noise(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.shouldGenerateSurface
    fn should_generate_surface(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.shouldGenerateBedrock
    ///
    /// Has no effect, by Paper's own doc: bedrock is part of the surface step.
    fn should_generate_bedrock(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.shouldGenerateCaves
    fn should_generate_caves(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.shouldGenerateDecorations
    fn should_generate_decorations(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.shouldGenerateMobs
    fn should_generate_mobs(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/generator/ChunkGenerator.shouldGenerateStructures
    fn should_generate_structures(&self) -> bool {
        false
    }
}

/// bukkit: org/bukkit/generator/BiomeProvider
pub trait BiomeProvider: Send + Sync {
    /// Whose this is, when it lives in another plugin.
    #[doc(hidden)]
    fn named(&self) -> Option<(String, String)> {
        None
    }
    /// bukkit: org/bukkit/generator/BiomeProvider.getBiome
    ///
    /// By a biome's name, `minecraft:plains`, at a block of a corner of a 4x4x4 cell.
    fn get_biome(&self, world: &GeneratingWorld, x: i32, y: i32, z: i32) -> String;
    /// bukkit: org/bukkit/generator/BiomeProvider.getBiome
    ///
    /// With the game's climate at the place: Java's overload, which by default ignores it.
    fn get_biome_at(&self, world: &GeneratingWorld, x: i32, y: i32, z: i32, _point: &BiomeParameterPoint) -> String {
        self.get_biome(world, x, y, z)
    }
    /// bukkit: org/bukkit/generator/BiomeProvider.getBiomes
    fn get_biomes(&self, world: &GeneratingWorld) -> Vec<String>;
}

/// Populators with no generator of the plugin's own.
struct NoTerrain;

impl ChunkGenerator for NoTerrain {}

static POPULATORS: Mutex<Option<HashMap<String, Vec<Arc<dyn BlockPopulator>>>>> = Mutex::new(None);

fn added_populators(id: &str) -> Vec<Arc<dyn BlockPopulator>> {
    POPULATORS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(id).cloned()).unwrap_or_default()
}

/// bukkit: org/bukkit/generator/BlockPopulator
pub trait BlockPopulator: Send + Sync {
    /// bukkit: org/bukkit/generator/BlockPopulator.populate
    fn populate(&self, _world: &World, _random: &mut JavaRandom, _chunk_x: i32, _chunk_z: i32, _region: &LimitedRegion) {}
}

/// bukkit: org/bukkit/generator/LimitedRegion
///
/// The chunk being populated and its neighbours, written through the live world once the chunk stands.
#[derive(Clone, Debug)]
pub struct LimitedRegion {
    world: World,
    chunk_x: i32,
    chunk_z: i32,
}

impl LimitedRegion {
    /// The chunk at `chunk_x`, `chunk_z` and its neighbours.
    pub(crate) fn of(world: &World, chunk_x: i32, chunk_z: i32) -> Self {
        Self { world: world.clone(), chunk_x, chunk_z }
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.getBuffer
    pub fn get_buffer(&self) -> i32 {
        16
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.getCenterChunkX
    pub fn get_center_chunk_x(&self) -> i32 {
        self.chunk_x
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.getCenterChunkZ
    pub fn get_center_chunk_z(&self) -> i32 {
        self.chunk_z
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.getCenterBlockX
    pub fn get_center_block_x(&self) -> i32 {
        self.chunk_x * 16 + 8
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.getCenterBlockZ
    pub fn get_center_block_z(&self) -> i32 {
        self.chunk_z * 16 + 8
    }
    /// bukkit: org/bukkit/RegionAccessor.getWorld, org/bukkit/generator/LimitedRegion.getWorld
    pub fn get_world(&self) -> &World {
        &self.world
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.getTileEntities
    pub fn get_tile_entities(&self) -> Vec<crate::block_data::TileState> {
        let mut all = Vec::new();
        for x in self.chunk_x - 1..=self.chunk_x + 1 {
            for z in self.chunk_z - 1..=self.chunk_z + 1 {
                all.extend(self.world.get_chunk_at(x, z).get_tile_entities());
            }
        }
        all
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.scheduleBlockUpdate
    pub fn schedule_block_update(&self, x: i32, y: i32, z: i32) {
        self.world.get_block_at(x, y, z).tick();
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.scheduleFluidUpdate
    pub fn schedule_fluid_update(&self, x: i32, y: i32, z: i32) {
        self.world.get_block_at(x, y, z).fluid_tick();
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.setBlockState
    pub fn set_block_state(&self, x: i32, y: i32, z: i32, state: &crate::BlockState) {
        self.world.get_block_at(x, y, z).set_block_data(&state.get_block_data());
    }
    /// bukkit: org/bukkit/generator/LimitedRegion.isInRegion
    pub fn is_in_region(&self, x: i32, y: i32, z: i32) -> bool {
        let _ = y;
        let (from_x, from_z) = (self.chunk_x * 16 - 16, self.chunk_z * 16 - 16);
        x >= from_x && x < from_x + 48 && z >= from_z && z < from_z + 48
    }
    /// bukkit: org/bukkit/RegionAccessor.getType
    pub fn get_type(&self, x: i32, y: i32, z: i32) -> Material {
        self.world.get_block_at(x, y, z).get_type()
    }
    /// bukkit: org/bukkit/RegionAccessor.setType
    pub fn set_type(&self, x: i32, y: i32, z: i32, material: &Material) {
        self.world.get_block_at(x, y, z).set_type(material);
    }
    /// bukkit: org/bukkit/RegionAccessor.getBlockData
    pub fn get_block_data(&self, x: i32, y: i32, z: i32) -> BlockData {
        BlockData::parse(&self.world.get_block_at(x, y, z).get_block_data())
    }
    /// bukkit: org/bukkit/RegionAccessor.setBlockData
    pub fn set_block_data(&self, x: i32, y: i32, z: i32, data: &BlockData) {
        self.world.get_block_at(x, y, z).set_block_data(&data.get_as_string());
    }
}

/// The generator asking a plugin's `BiomeProvider` about one cell, on a chunk worker.
fn on_biome(value: &Value) -> Answer {
    let Some(provider) = value["world"].as_str().and_then(provider_of) else {
        return Answer::Pass;
    };
    let id = value["world"].as_str().unwrap_or_default();
    let number = |key: &str| value[key].as_i64().unwrap_or(0);
    let (min_y, height) = (number("min_y") as i32, number("height") as i32);
    let world = GeneratingWorld { name: id.rsplit(':').next().unwrap_or(id).to_owned(), seed: number("seed"), environment: value["environment"].as_str().unwrap_or("NORMAL").to_owned(), min_height: min_y, max_height: min_y + height };
    let quart = |index: usize| value["quart"][index].as_i64().unwrap_or(0) as i32;
    let (x, y, z) = (quart(0), quart(1), quart(2));
    ASKED.with(|asked| *asked.borrow_mut() = Some(([x, y, z], value["vanilla"].as_str().unwrap_or_default().to_owned())));
    let point = BiomeParameterPoint::at(&value["climate"]);
    let biome = provider.get_biome_at(&world, x << 2, y << 2, z << 2, &point);
    query(json!({ "op": "world_biome_name", "name": biome }));
    Answer::Success
}

fn on_generate(value: &Value) -> Answer {
    let Some(id) = value["world"].as_str() else {
        return Answer::Pass;
    };
    // With the game's own terrain the generator asked the provider itself, cell by cell.
    let (generator, provider) = (generator_of(id), provider_of(id).filter(|_| value["biomes_done"] != true));
    // Not parallel-capable: one chunk at a time, as Bukkit's lock does.
    static SERIAL: Mutex<()> = Mutex::new(());
    let _serial = generator.as_ref().filter(|generator| !generator.is_parallel_capable()).map(|_| SERIAL.lock().unwrap_or_else(|e| e.into_inner()));
    // The game's own steps ran: each of the plugin's follows the one it goes with.
    if let Some(step) = value["step"].as_str() {
        let Some(generator) = &generator else {
            return Answer::Pass;
        };
        let number = |key: &str| value[key].as_i64().unwrap_or(0);
        let (x, z, seed, min_y, height) = (number("x") as i32, number("z") as i32, number("seed"), number("min_y") as i32, number("height") as i32);
        let world = GeneratingWorld { name: id.rsplit(':').next().unwrap_or(id).to_owned(), seed, environment: value["environment"].as_str().unwrap_or("NORMAL").to_owned(), min_height: min_y, max_height: min_y + height };
        let mut data = ChunkData::new(min_y, height);
        let mut random = JavaRandom::new(seed);
        random.set_seed(i64::from(x).wrapping_mul(341_873_128_712).wrapping_add(i64::from(z).wrapping_mul(132_897_987_541)));
        match step {
            "noise" => generator.generate_noise(&world, &mut random, x, z, &mut data),
            "surface" => generator.generate_surface(&world, &mut random, x, z, &mut data),
            "bedrock" => generator.generate_bedrock(&world, &mut random, x, z, &mut data),
            _ => generator.generate_caves(&world, &mut random, x, z, &mut data),
        }
        query(json!({ "op": "world_chunk", "palette": data.palette, "runs": ChunkData::runs(&data.cells) }));
        return Answer::Success;
    }
    if generator.is_none() && provider.is_none() {
        return Answer::Pass;
    }
    let number = |key: &str| value[key].as_i64().unwrap_or(0);
    let (x, z, seed, min_y, height) = (number("x") as i32, number("z") as i32, number("seed"), number("min_y") as i32, number("height") as i32);
    let world = GeneratingWorld { name: id.rsplit(':').next().unwrap_or(id).to_owned(), seed, environment: value["environment"].as_str().unwrap_or("NORMAL").to_owned(), min_height: min_y, max_height: min_y + height };
    let mut data = ChunkData::new(min_y, height);
    let mut random = JavaRandom::new(seed);
    random.set_seed(i64::from(x).wrapping_mul(341_873_128_712).wrapping_add(i64::from(z).wrapping_mul(132_897_987_541)));
    // The biomes first: a generator can read them through `ChunkData::get_biome`.
    if let Some(provider) = &provider {
        let sections = height / 16;
        let mut quarts = Vec::with_capacity(sections as usize * 64);
        for section in 0..sections {
            for cell_y in 0..4 {
                for cell_z in 0..4 {
                    for cell_x in 0..4 {
                        quarts.push([x * 4 + cell_x, (min_y + section * 16 + cell_y * 4) >> 2, z * 4 + cell_z]);
                    }
                }
            }
        }
        let climate = query(json!({ "op": "world_climate", "quarts": quarts }));
        let mut cell = 0;
        for section in 0..sections {
            for cell_y in 0..4 {
                for cell_z in 0..4 {
                    for cell_x in 0..4 {
                        let point = BiomeParameterPoint::at(&climate[cell]);
                        cell += 1;
                        let biome = provider.get_biome_at(&world, x * 16 + cell_x * 4, min_y + section * 16 + cell_y * 4, z * 16 + cell_z * 4, &point);
                        let index = match data.biomes.iter().position(|known| *known == biome) {
                            Some(index) => index,
                            None => {
                                data.biomes.push(biome);
                                data.biomes.len() - 1
                            }
                        };
                        data.biome_cells.push(index as u16);
                    }
                }
            }
        }
    }
    if let Some(generator) = &generator {
        if value["steps_done"] != true {
            generator.generate_noise(&world, &mut random, x, z, &mut data);
            generator.generate_surface(&world, &mut random, x, z, &mut data);
            generator.generate_bedrock(&world, &mut random, x, z, &mut data);
            generator.generate_caves(&world, &mut random, x, z, &mut data);
        }
        let mut grid = BiomeGrid { columns: vec![None; 256] };
        generator.generate_chunk_data(&world, &mut random, x, z, &mut grid, &mut data);
        // Whole columns, as the old call means them: they win over a provider, as it came last.
        if grid.columns.iter().any(Option::is_some) {
            let sections = (height / 16) as usize;
            data.biome_cells.resize(sections * 64, 0);
            if data.biomes.is_empty() {
                data.biomes.push("minecraft:plains".to_owned());
            }
            for cell in 0..sections * 64 {
                let (cell_x, cell_z) = ((cell & 3) * 4, ((cell >> 2) & 3) * 4);
                if let Some(biome) = &grid.columns[cell_z * 16 + cell_x] {
                    let index = match data.biomes.iter().position(|known| known == biome) {
                        Some(index) => index,
                        None => {
                            data.biomes.push(biome.clone());
                            data.biomes.len() - 1
                        }
                    };
                    data.biome_cells[cell] = index as u16;
                }
            }
        }
    }
    query(json!({ "op": "world_chunk", "palette": data.palette, "runs": ChunkData::runs(&data.cells), "biome_palette": data.biomes, "biome_runs": ChunkData::runs(&data.biome_cells) }));
    Answer::Success
}

fn on_chunk_load(value: &Value) -> Answer {
    if value["new"].as_bool() != Some(true) && value["new_chunk"].as_bool() != Some(true) {
        return Answer::Pass;
    }
    let Some(id) = value["level"].as_str().or(value["dimension"].as_str()) else {
        return Answer::Pass;
    };
    let generator = generator_of(id).or_else(|| (!added_populators(id).is_empty()).then(|| Arc::new(NoTerrain) as Arc<dyn ChunkGenerator>));
    let Some(generator) = generator else {
        return Answer::Pass;
    };
    let (Some(x), Some(z)) = (value["chunk"][0].as_i64(), value["chunk"][1].as_i64()) else {
        return Answer::Pass;
    };
    let world = World::of(id);
    let (x, z) = (x as i32, z as i32);
    let mut random = JavaRandom::new(world.get_seed());
    random.set_seed(i64::from(x).wrapping_mul(341_873_128_712).wrapping_add(i64::from(z).wrapping_mul(132_897_987_541)));
    let region = LimitedRegion { world: world.clone(), chunk_x: x, chunk_z: z };
    for populator in generator.get_default_populators(&world).into_iter().chain(added_populators(id)) {
        populator.populate(&world, &mut random, x, z, &region);
    }
    Answer::Pass
}

pub(crate) fn listen(host: Host) {
    host.on("world_generate", on_generate);
    host.on("world_biome", on_biome);
    host.on("chunk_load", on_chunk_load);
}

impl World {
    /// bukkit: org/bukkit/World.getGenerator
    pub fn get_generator(&self) -> Option<Arc<dyn ChunkGenerator>> {
        generator_of(self.dimension())
    }
    /// bukkit: org/bukkit/World.getBiomeProvider
    pub fn get_biome_provider(&self) -> Option<Arc<dyn BiomeProvider>> {
        provider_of(self.dimension())
    }
    /// bukkit: org/bukkit/World.getPopulators
    ///
    /// The generator's and any added: `add_populator` and `remove_populators` are what Java does to this list.
    pub fn get_populators(&self) -> Vec<Arc<dyn BlockPopulator>> {
        let mut all = generator_of(self.dimension()).map_or_else(Vec::new, |generator| generator.get_default_populators(self));
        all.extend(added_populators(self.dimension()));
        all
    }
    /// Java's `getPopulators().add`.
    pub fn add_populator(&self, populator: Arc<dyn BlockPopulator>) {
        POPULATORS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().entry(self.dimension().to_owned()).or_default().push(populator);
    }
    /// Java's `getPopulators().clear`, of what was added.
    pub fn remove_populators(&self) {
        if let Some(all) = POPULATORS.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            all.remove(self.dimension());
        }
    }
}
