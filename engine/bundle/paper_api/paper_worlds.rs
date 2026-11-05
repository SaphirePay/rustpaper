// Worlds a plugin creates, and terrain a plugin generates. `Dim::Custom` and `CREATED_WORLDS` come from
// tools/dim_patch.py, as a mixin can't add to an enum.
use super::render_control::parse_state;
use super::{CREATED_WORLDS, CreatedWorld, Dim, Game};
use horizon_client::chunk::{Chunk, LevelShape, SavedProto, block_entity::BlockEntity};
use horizon_client::save::world_gen_settings;
use horizon_client::section_mask::SectionMask;
use horizon_client::streaming::{ChunkGenerator, LodSampler, Proto};
use horizon_client::vanillagen::VanillaTerrain;
use horizon_world::nbt::{Tag, parse_snbt};
use serde_json::{Value, json};
use std::{cell::RefCell, path::Path, sync::Arc};

/// Services by name, across plugins: each library has its own statics, so the game keeps the list.
static SERVICES: std::sync::Mutex<Vec<Value>> = std::sync::Mutex::new(Vec::new());

/// `WorldType.FLAT`: `FlatLevelGeneratorSettings.getDefault`.
const FLAT: &str = r#"{dimensions:{"minecraft:overworld":{generator:{type:"minecraft:flat",settings:{biome:"minecraft:plains",lakes:0b,features:0b,layers:[{block:"minecraft:bedrock",height:1},{block:"minecraft:dirt",height:2},{block:"minecraft:grass_block",height:1}],structure_overrides:[]}}}}}"#;

/// The world generation settings a level reads: the server's own for its levels, its creator's for a created one.
pub(in crate::game) fn settings_of(dim: Dim, world_root: &Path) -> Option<Tag> {
    match dim.created() {
        Some(world) if world.flat => flat_settings(world.layers).and_then(|text| parse_snbt(&text).ok()).or_else(|| parse_snbt(FLAT).ok()).map(|(settings, _)| settings),
        Some(_) => None,
        None => world_gen_settings(world_root),
    }
}

/// `WorldCreator.generatorSettings`: the JSON of `FlatLevelGeneratorSettings`, ids with or without their `minecraft:`.
fn flat_settings(text: &str) -> Option<String> {
    let named = |name: &str| if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") };
    let mut value: Value = serde_json::from_str(text).ok()?;
    let object = value.as_object_mut()?;
    for layer in object.get_mut("layers")?.as_array_mut()? {
        let block = named(layer["block"].as_str()?);
        layer["block"] = json!(block);
    }
    let biome = named(object.get("biome").and_then(Value::as_str).unwrap_or("plains"));
    object.insert("biome".to_owned(), json!(biome));
    object.entry("structure_overrides").or_insert(json!([]));
    let flag = |object: &mut serde_json::Map<String, Value>, name: &str| if object.remove(name).and_then(|on| on.as_bool()) == Some(true) { "1b" } else { "0b" };
    let (lakes, features) = (flag(object, "lakes"), flag(object, "features"));
    let rest = value.to_string();
    let rest = rest.strip_prefix('{')?;
    Some(format!(r#"{{dimensions:{{"minecraft:overworld":{{generator:{{type:"minecraft:flat",settings:{{lakes:{lakes},features:{features},{rest}}}}}}}}}"#))
}

thread_local! {
    /// What a plugin's generator answered, on the worker that asked.
    static DELIVERED: RefCell<Option<Value>> = const { RefCell::new(None) };
    /// The game's terrain of the world this worker is finishing a chunk of.
    static SAMPLING: RefCell<Option<Arc<VanillaTerrain>>> = const { RefCell::new(None) };
    /// The world this worker is generating for a plugin's `BiomeProvider`: its id, seed, lowest block, height, environment.
    static CHOOSING: std::cell::Cell<Option<(&'static str, i64, i32, i32, &'static str)>> = const { std::cell::Cell::new(None) };
    /// What the provider said for the place asked about.
    static CHOSEN: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
    /// The chunk a plugin's generator is writing over, during one of its steps, and the blocks it can name.
    static STEPPING: std::cell::Cell<Option<&'static horizon_world::registry::BlockRegistry>> = const { std::cell::Cell::new(None) };
    /// What a plugin's generator reads: the chunk as the step before left it, layer by layer.
    static READING: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    /// Its height maps then, by map, column by column.
    static HEIGHTS: RefCell<Vec<[i32; 256]>> = const { RefCell::new(Vec::new()) };
}

type Stepped = horizon_world::worldgen::terrain::TerrainChunk<'static, horizon_world::worldgen::generator::GenBlockStates>;

/// `CustomChunkGenerator`: the plugin's step after the game's, on the chunk as it stands.
fn plugin_step(step: &'static str, chunk: &mut Stepped) {
    let (Some((world, seed, min_y, height, environment)), Some(registry)) = (CHOOSING.get(), STEPPING.get()) else {
        return;
    };
    let at = chunk.position();
    let (from_x, from_z) = (at.x.wrapping_shl(4), at.z.wrapping_shl(4));
    READING.with(|cells| {
        let mut cells = cells.borrow_mut();
        cells.clear();
        for y in min_y..min_y + height {
            for z in 0..16 {
                for x in 0..16 {
                    cells.push(chunk.block(from_x + x, y, from_z + z).0);
                }
            }
        }
    });
    {
        use horizon_world::heightmap::HeightmapType;
        const MAPS: [HeightmapType; 4] = [HeightmapType::WorldSurface, HeightmapType::OceanFloor, HeightmapType::MotionBlocking, HeightmapType::MotionBlockingNoLeaves];
        let _ = chunk.prime_heightmaps(&MAPS);
        HEIGHTS.with(|heights| {
            let mut heights = heights.borrow_mut();
            heights.clear();
            for kind in MAPS {
                let mut columns = [min_y; 256];
                if let Some(map) = chunk.heightmap(kind) {
                    for (index, column) in columns.iter_mut().enumerate() {
                        *column = map.first_available((index % 16) as i32, (index / 16) as i32).unwrap_or(min_y) - 1;
                    }
                }
                heights.push(columns);
            }
        });
    }
    DELIVERED.with(|slot| *slot.borrow_mut() = None);
    let hook = horizon_world::worldgen::biome_source::BIOME_HOOK.replace(None);
    super::paper_bridge::fire_free("world_generate", &json!({ "world": world, "x": at.x, "z": at.z, "seed": seed, "min_y": min_y, "height": height, "environment": environment, "step": step, "biomes_done": true }));
    horizon_world::worldgen::biome_source::BIOME_HOOK.set(hook);
    let Some(answer) = DELIVERED.with(|slot| slot.borrow_mut().take()) else {
        return;
    };
    let palette: Vec<Option<u32>> = answer["palette"].as_array().into_iter().flatten().map(|text| text.as_str().filter(|text| !text.is_empty()).and_then(|text| parse_state(registry, text).ok()).map(|state| state.0)).collect();
    let (mut index, area) = (0_usize, 16 * 16);
    for run in answer["runs"].as_array().into_iter().flatten() {
        let (value, count) = (run[0].as_u64().unwrap_or(0) as usize, run[1].as_u64().unwrap_or(0) as usize);
        if let Some(Some(state)) = palette.get(value) {
            for cell in index..index + count {
                let (y, rest) = (cell / area, cell % area);
                let _ = chunk.set_block_raw(at.x.wrapping_shl(4) + (rest % 16) as i32, min_y + y as i32, at.z.wrapping_shl(4) + (rest / 16) as i32, horizon_world::registry::BlockStateId(*state));
            }
        }
        index += count;
    }
}

/// Biome names are few: each is kept once.
fn interned(name: &str) -> &'static str {
    static NAMES: std::sync::Mutex<std::collections::BTreeSet<&'static str>> = std::sync::Mutex::new(std::collections::BTreeSet::new());
    let mut names = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(known) = names.get(name) {
        return known;
    }
    let kept: &'static str = Box::leak(name.to_owned().into_boxed_str());
    names.insert(kept);
    kept
}

/// `CustomWorldChunkManager.getNoiseBiome`: the plugin's answer goes into the generator, so its surface and features follow.
fn provider_biome(quart: [i32; 3], vanilla: &str, climate: &mut dyn FnMut() -> [i64; 6]) -> Option<&'static str> {
    use horizon_world::worldgen::biome_source::BIOME_HOOK;
    use std::sync::atomic::{AtomicU64, Ordering};
    /// The answers so far, as structures ask about the same few places from every worker: at most this many are kept.
    const KEPT: usize = 1 << 22;
    type Answers = std::collections::HashMap<(&'static str, [i32; 3]), &'static str>;
    static ANSWERS: std::sync::LazyLock<std::sync::RwLock<Answers>> = std::sync::LazyLock::new(Default::default);
    static ASKED: AtomicU64 = AtomicU64::new(0);
    let (world, seed, min_y, height, environment) = CHOOSING.get()?;
    if let Some(known) = ANSWERS.read().unwrap_or_else(|e| e.into_inner()).get(&(world, quart)) {
        return Some(known);
    }
    let asked = ASKED.fetch_add(1, Ordering::Relaxed);
    if asked % (1 << 18) == 0 {
        log::info!("[paper_worlds] a plugin's BiomeProvider asked {asked} times, now on {:?}", std::thread::current().name());
    }
    let climate = climate();
    // The provider can ask for the game's own biomes: those mustn't come back here.
    let hook = BIOME_HOOK.replace(None);
    CHOSEN.set(None);
    super::paper_bridge::fire_free("world_biome", &json!({ "world": world, "seed": seed, "min_y": min_y, "height": height, "environment": environment, "quart": quart, "vanilla": vanilla, "climate": climate }));
    BIOME_HOOK.set(hook);
    let chosen = CHOSEN.take()?;
    let mut answers = ANSWERS.write().unwrap_or_else(|e| e.into_inner());
    if answers.len() >= KEPT {
        answers.clear();
    }
    answers.insert((world, quart), chosen);
    Some(chosen)
}

/// A `BiomeProvider` asking for the game's climate or biomes, on a chunk worker: no game to lend.
pub(in crate::game) fn free_query(request: &str) -> Option<String> {
    let climate = request.contains(r#""op":"world_climate""#);
    let chosen = request.contains(r#""op":"world_biome_name""#);
    if request.contains(r#""op":"world_legacy_data""#) {
        let value: Value = serde_json::from_str(request).ok()?;
        return Some(json!(super::paper_fam_items::legacy_byte(value["block"].as_str()?).unwrap_or(0)).to_string());
    }
    if request.contains(r#""op":"world_proto_height""#) {
        let value: Value = serde_json::from_str(request).ok()?;
        let (map, x, z) = (value["map"].as_u64()? as usize, value["x"].as_u64()? as usize, value["z"].as_u64()? as usize);
        return Some(HEIGHTS.with(|heights| heights.borrow().get(map).and_then(|columns| columns.get(z * 16 + x).copied())).map_or_else(|| "null".to_owned(), |height| height.to_string()));
    }
    if request.contains(r#""op":"world_proto_block""#) {
        let value: Value = serde_json::from_str(request).ok()?;
        let (x, y, z) = (value["x"].as_i64()?, value["y"].as_i64()?, value["z"].as_i64()?);
        let (_, _, min_y, _, _) = CHOOSING.get()?;
        let index = usize::try_from((y - i64::from(min_y)) * 256 + z * 16 + x).ok()?;
        let state = READING.with(|cells| cells.borrow().get(index).copied())?;
        let text = STEPPING.get().and_then(|registry| registry.state(horizon_world::registry::BlockStateId(state))).map(|state| {
            let properties: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
            if properties.is_empty() { state.name.to_string() } else { format!("{}[{}]", state.name, properties.join(",")) }
        });
        return Some(json!(text).to_string());
    }
    if !climate && !chosen && !request.contains(r#""op":"world_biomes""#) {
        return None;
    }
    let value: Value = serde_json::from_str(request).ok()?;
    if chosen {
        CHOSEN.set(value["name"].as_str().map(interned));
        return Some("true".to_owned());
    }
    let quarts: Vec<[i32; 3]> = value["quarts"].as_array().into_iter().flatten().filter_map(|quart| Some([quart[0].as_i64()? as i32, quart[1].as_i64()? as i32, quart[2].as_i64()? as i32])).collect();
    Some(SAMPLING.with(|slot| match slot.borrow().as_ref() {
        None => "null".to_owned(),
        Some(terrain) if climate => json!(terrain.climate_points(&quarts)).to_string(),
        Some(terrain) if value["all"].as_bool() == Some(true) => json!(terrain.generator_data().biome_names().iter().map(|name| name.to_string()).collect::<Vec<_>>()).to_string(),
        Some(terrain) => json!(terrain.noise_biomes(&quarts)).to_string(),
    }))
}

/// A chunk from a plugin comes on a worker with no game to lend: this takes it before the query does.
pub(in crate::game) fn take_chunk(request: &str) -> bool {
    if !request.contains(r#""op":"world_chunk""#) {
        return false;
    }
    if let Ok(value) = serde_json::from_str::<Value>(request) {
        DELIVERED.with(|slot| *slot.borrow_mut() = Some(value));
    }
    true
}

/// An empty column for a plugin to fill: no `shouldGenerate...` asked for the game's own.
pub(in crate::game) struct Blank {
    pub(in crate::game) shape: LevelShape,
    pub(in crate::game) biome: u32,
}

impl ChunkGenerator for Blank {
    type Worker = ();
    fn worker(&self) {}
    fn proto(&self, _: &mut (), x: i32, z: i32) -> Proto {
        let mut chunk = Chunk::with_layout(x, z, self.shape.column());
        for section in &mut chunk.sections {
            Arc::make_mut(section).biomes = horizon_core::paletted::PalettedContainer::new(horizon_client::chunk::BIOME_STRATEGY, self.biome);
        }
        Box::new(chunk)
    }
    fn finish(&self, proto: Proto) -> Chunk {
        *proto.downcast::<Chunk>().expect("blank terrain proto")
    }
}

/// `CustomChunkGenerator`: `inner` makes the chunk, then the plugin writes over it.
pub(in crate::game) struct Overlay<G> {
    inner: Arc<G>,
    vanilla: Option<Arc<VanillaTerrain>>,
    /// The generator itself asks the plugin's `BiomeProvider`.
    provider: bool,
    /// What a plugin's `ChunkGenerator` wants of the game's steps: none for a world the plugin writes alone.
    created: Option<CreatedWorld>,
    world: &'static str,
    seed: i64,
    shape: LevelShape,
    environment: &'static str,
    assets: Arc<horizon_client::assets::Assets>,
    biomes: Arc<horizon_client::biome::Biomes>,
}

impl<G> Overlay<G> {
    pub(in crate::game) fn new(inner: Arc<G>, vanilla: Option<Arc<VanillaTerrain>>, created: Option<CreatedWorld>, dim: Dim, seed: i64, shape: LevelShape, assets: Arc<horizon_client::assets::Assets>, biomes: Arc<horizon_client::biome::Biomes>) -> Self {
        let environment = match dim.kind() {
            Dim::Nether => "NETHER",
            Dim::End => "THE_END",
            _ => "NORMAL",
        };
        Self { inner, vanilla, provider: created.is_some_and(|world| world.provider), created, world: dim.id(), seed, shape, environment, assets, biomes }
    }
    /// `work`, with the plugin's `BiomeProvider` standing in for the game's biome source.
    fn choosing<R>(&self, work: impl FnOnce() -> R) -> R {
        use horizon_world::worldgen::biome_source::BIOME_HOOK;
        use horizon_world::worldgen::generator::{AFTER_STEP, STEPS_LEFT_OUT};
        let stepped = self.created.filter(|world| world.stepped);
        if !self.provider && stepped.is_none() {
            return work();
        }
        CHOOSING.set(Some((self.world, self.seed, self.shape.min_y, self.shape.height as i32, self.environment)));
        SAMPLING.with(|slot| slot.borrow_mut().clone_from(&self.vanilla));
        if self.provider {
            BIOME_HOOK.set(Some(provider_biome));
        }
        if let Some(world) = stepped {
            STEPS_LEFT_OUT.set(world.left_out);
            AFTER_STEP.set(Some(plugin_step));
            STEPPING.set(Some(self.assets.registry));
        }
        let done = work();
        STEPPING.set(None);
        AFTER_STEP.set(None);
        STEPS_LEFT_OUT.set(0);
        BIOME_HOOK.set(None);
        SAMPLING.with(|slot| *slot.borrow_mut() = None);
        CHOOSING.set(None);
        done
    }
    fn write(&self, chunk: &mut Chunk, answer: &Value) {
        let palette: Vec<u32> = answer["palette"].as_array().into_iter().flatten().map(|text| text.as_str().filter(|text| !text.is_empty()).and_then(|text| parse_state(self.assets.registry, text).ok()).map_or(u32::MAX, |state| state.0)).collect();
        let (mut at, area) = (0_usize, 16 * 16);
        for run in answer["runs"].as_array().into_iter().flatten() {
            let (value, count) = (run[0].as_u64().unwrap_or(0) as usize, run[1].as_u64().unwrap_or(0) as usize);
            if let Some(&state) = palette.get(value).filter(|state| **state != u32::MAX) {
                for index in at..at + count {
                    let (y, rest) = (index / area, index % area);
                    chunk.set((rest % 16) as i32, self.shape.min_y + y as i32, (rest / 16) as i32, state);
                }
            }
            at += count;
        }
        let biomes: Vec<Option<u32>> = answer["biome_palette"].as_array().into_iter().flatten().map(|name| name.as_str().map(|name| u32::from(self.biomes.id(name)))).collect();
        let (mut at, cells) = (0_usize, 4 * 4 * 4);
        for run in answer["biome_runs"].as_array().into_iter().flatten() {
            let (value, count) = (run[0].as_u64().unwrap_or(0) as usize, run[1].as_u64().unwrap_or(0) as usize);
            if let Some(Some(biome)) = biomes.get(value) {
                for index in at..at + count {
                    if let Some(section) = chunk.sections.get_mut(index / cells) {
                        Arc::make_mut(section).biomes.set(index % cells, *biome);
                    }
                }
            }
            at += count;
        }
        for section in &mut chunk.sections {
            Arc::make_mut(section).compact();
        }
    }
}

impl<G: ChunkGenerator> ChunkGenerator for Overlay<G> {
    type Worker = G::Worker;
    fn worker(&self) -> G::Worker {
        self.inner.worker()
    }
    fn lod_sampler(self: Arc<Self>) -> Box<dyn LodSampler> {
        ChunkGenerator::lod_sampler(self.inner.clone())
    }
    fn load(&self, x: i32, z: i32) -> Option<Chunk> {
        self.inner.load(x, z)
    }
    fn prepare_loaded(&self, worker: &mut G::Worker, chunk: &Chunk) {
        self.inner.prepare_loaded(worker, chunk);
    }
    fn idle(&self, worker: &mut G::Worker) {
        self.inner.idle(worker);
    }
    fn proto(&self, worker: &mut G::Worker, x: i32, z: i32) -> Proto {
        self.choosing(|| self.inner.proto(worker, x, z))
    }
    fn decorates(&self) -> bool {
        self.inner.decorates() && self.created.is_none_or(|world| world.decorations)
    }
    fn decorate(&self, worker: &mut G::Worker, region: &mut [Proto; 9]) {
        self.choosing(|| self.inner.decorate(worker, region));
    }
    fn decorates_beside_final(&self) -> bool {
        self.inner.decorates_beside_final()
    }
    fn proto_beside_final(&self, worker: &mut G::Worker, snapshot: &Chunk) -> Option<Proto> {
        self.choosing(|| self.inner.proto_beside_final(worker, snapshot))
    }
    fn final_writes(&self, snapshot: &Chunk, proto: &Proto) -> Vec<([i32; 3], u32)> {
        self.inner.final_writes(snapshot, proto)
    }
    fn final_block_entities(&self, proto: &Proto, writes: &[([i32; 3], u32)]) -> Vec<((i32, i32, i32), BlockEntity)> {
        self.inner.final_block_entities(proto, writes)
    }
    fn finish(&self, proto: Proto) -> Chunk {
        let mut chunk = self.inner.finish(proto);
        let payload = json!({ "world": self.world, "x": chunk.x, "z": chunk.z, "seed": self.seed, "min_y": self.shape.min_y, "height": self.shape.height, "environment": self.environment, "biomes_done": self.provider, "steps_done": self.created.is_some_and(|world| world.stepped) });
        DELIVERED.with(|slot| *slot.borrow_mut() = None);
        SAMPLING.with(|slot| slot.borrow_mut().clone_from(&self.vanilla));
        super::paper_bridge::fire_free("world_generate", &payload);
        SAMPLING.with(|slot| *slot.borrow_mut() = None);
        if let Some(answer) = DELIVERED.with(|slot| slot.borrow_mut().take()) {
            self.write(&mut chunk, &answer);
        }
        chunk
    }
    fn is_decorated(&self, proto: &Proto) -> bool {
        self.inner.is_decorated(proto)
    }
    fn light_view(&self, proto: &Proto) -> Option<Chunk> {
        self.inner.light_view(proto)
    }
    fn non_air_sections(&self, proto: &Proto) -> Option<SectionMask> {
        self.inner.non_air_sections(proto)
    }
    fn saved_proto(&self, proto: &Proto) -> Option<SavedProto> {
        self.inner.saved_proto(proto)
    }
    fn restore_proto(&self, proto: &mut Proto, saved: SavedProto) {
        self.inner.restore_proto(proto, saved);
    }
    fn save_protos(&self, protos: &[&Proto]) {
        self.inner.save_protos(protos);
    }
    fn save_finished(&self, chunk: Chunk) {
        self.inner.save_finished(chunk);
    }
}

/// The levels plugins have open now.
pub(in crate::game) fn open_levels() -> Vec<String> {
    CREATED_WORLDS.read().unwrap_or_else(|e| e.into_inner()).iter().filter(|world| world.open).map(|world| world.id.to_owned()).collect()
}

fn described(world: &CreatedWorld) -> Value {
    json!({ "id": world.id, "name": world.name, "environment": environment(world.kind), "seed": world.seed, "generated": world.generated, "open": world.open, "hardcore": world.hardcore })
}

fn environment(kind: Dim) -> &'static str {
    match kind {
        Dim::Nether => "NETHER",
        Dim::End => "THE_END",
        _ => "NORMAL",
    }
}

/// The level goes: saved first if asked.
pub(in crate::game) fn close(game: &mut Game<'_>, id: &str, save: bool) {
    super::paper_events::level_unloaded(game, id);
    if let Err(error) = game.close_created_level(Dim::from_id(id), save) {
        log::warn!("[paper_worlds] closing {id}: {error}");
    }
}

pub(in crate::game) fn query(game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    match op {
        "world_create" => {
            let name = query["name"].as_str()?;
            let id = query["id"].as_str()?;
            let kind = match query["environment"].as_str() {
                Some("NETHER") => Dim::Nether,
                Some("THE_END") => Dim::End,
                _ => Dim::Overworld,
            };
            let (seed, flat) = (query["seed"].as_i64(), query["flat"].as_bool().unwrap_or(false));
            let (structures, generated, vanilla) = (query["structures"].as_bool().unwrap_or(true), query["generated"].as_bool().unwrap_or(false), query["vanilla"].as_bool().unwrap_or(false));
            let mut worlds = CREATED_WORLDS.write().unwrap_or_else(|e| e.into_inner());
            if let Some(world) = worlds.iter_mut().find(|world| world.id == id) {
                world.open = true;
                return Some(described(world));
            }
            let (id, name): (&'static str, &'static str) = (Box::leak(id.to_owned().into_boxed_str()), Box::leak(name.to_owned().into_boxed_str()));
            let preset = match query["kind"].as_str() {
                Some("AMPLIFIED") => "minecraft:amplified",
                Some("LARGE_BIOMES") => "minecraft:large_biomes",
                _ => "minecraft:normal",
            };
            let layers: &'static str = Box::leak(query["settings"].as_str().unwrap_or_default().to_owned().into_boxed_str());
            let provider = query["provider"].as_bool().unwrap_or(false);
            if generated {
                super::paper_fam_items::legacy_ready(game);
            }
            let (stepped, decorations) = (query["stepped"].as_bool().unwrap_or(false), query["decorations"].as_bool().unwrap_or(true));
            let left_out = query["left_out"].as_u64().unwrap_or(0) as u8;
            let (mobs, hardcore) = (query["mobs"].as_bool().unwrap_or(true), query["hardcore"].as_bool().unwrap_or(false));
            let world = CreatedWorld { id, name, kind, seed, flat, structures, preset, layers, provider, stepped, left_out, decorations, mobs, hardcore, generated, vanilla, open: true };
            worlds.push(world);
            horizon_client::paper_hub::WORLD_KINDS.lock().unwrap_or_else(|e| e.into_inner()).push((id.to_owned(), kind.id().to_owned()));
            let told = described(&world);
            drop(worlds);
            super::paper_events::level_loaded(game, id);
            Some(told)
        }
        "service_add" => {
            let mut services = SERVICES.lock().unwrap_or_else(|e| e.into_inner());
            services.retain(|row| row["name"] != query["name"] || row["plugin"] != query["plugin"]);
            services.push(json!({ "name": query["name"], "plugin": query["plugin"], "priority": query["priority"] }));
            // `RegisteredServiceProvider.compareTo`: the highest first, in the order they came.
            services.sort_by_key(|row| std::cmp::Reverse(["Lowest", "Low", "Normal", "High", "Highest"].iter().position(|name| row["priority"].as_str().is_some_and(|priority| priority.eq_ignore_ascii_case(name)))));
            Some(json!(true))
        }
        "service_remove" => {
            SERVICES.lock().unwrap_or_else(|e| e.into_inner()).retain(|row| row["name"] != query["name"] || row["plugin"] != query["plugin"]);
            Some(json!(true))
        }
        "service_list" => Some(Value::Array(SERVICES.lock().unwrap_or_else(|e| e.into_inner()).iter().filter(|row| query["name"].is_null() || row["name"] == query["name"]).cloned().collect())),
        "event_listeners" => Some(json!(super::paper_bridge::listeners_of(query["hook"].as_str()?))),
        "event_hooks" => Some(json!(super::paper_bridge::listened())),
        "event_call" => {
            let (answer, carried) = super::paper_bridge::call_listener(game, query["hook"].as_str()?, query["name"].as_str()?, &query["payload"].to_string());
            Some(json!({ "answer": answer, "payload": carried.and_then(|text| serde_json::from_str::<Value>(&text).ok()) }))
        }
        "world_list" => Some(Value::Array(CREATED_WORLDS.read().unwrap_or_else(|e| e.into_inner()).iter().filter(|world| world.open).map(described).collect())),
        "world_unload" => {
            let id = query["id"].as_str()?;
            let dim = Dim::from_id(id);
            if !matches!(dim, Dim::Custom(_)) || game.lan.as_ref().is_some_and(|host| host.hub.guests.iter().any(|guest| guest.dimension == id)) {
                return Some(json!(false));
            }
            if let Dim::Custom(index) = dim {
                CREATED_WORLDS.write().unwrap_or_else(|e| e.into_inner())[usize::from(index)].open = false;
            }
            super::paper_bridge::push_action(json!({ "op": "world_close", "text": id, "save": query["save"].as_bool().unwrap_or(true) }).to_string());
            Some(json!(true))
        }
        _ => None,
    }
}
