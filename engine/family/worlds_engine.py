# The game's climate at any place, for a plugin's BiomeProvider: `Climate.Sampler.sample`.
EDITS = [
    ('horizon-world/src/worldgen/generator.rs', """    fn noise_biomes(&self, worker: &mut WorkerState, quarts: &[[i32; 3]]) -> Vec<Option<String>>;
""", """    fn noise_biomes(&self, worker: &mut WorkerState, quarts: &[[i32; 3]]) -> Vec<Option<String>>;
    fn climate_points(&self, worker: &mut WorkerState, quarts: &[[i32; 3]]) -> Vec<Option<[i64; 6]>>;
"""),
    ('horizon-world/src/worldgen/generator.rs', """    fn noise_biomes(&self, worker: &mut WorkerState, quarts: &[[i32; 3]]) -> Vec<Option<String>> {
        let mut resolver""", """    fn climate_points(&self, worker: &mut WorkerState, quarts: &[[i32; 3]]) -> Vec<Option<[i64; 6]>> {
        quarts
            .iter()
            .map(|&quart| {
                self.climate.sample(&mut worker.carver_context, quart).ok().map(|point| {
                    [point.temperature, point.humidity, point.continentalness, point.erosion, point.depth, point.weirdness]
                })
            })
            .collect()
    }
    fn noise_biomes(&self, worker: &mut WorkerState, quarts: &[[i32; 3]]) -> Vec<Option<String>> {
        let mut resolver"""),
    ('horizon-world/src/worldgen/generator.rs', """    pub fn noise_biomes(&mut self, quarts: &[[i32; 3]]) -> Vec<Option<String>> {
        self.pipeline.noise_biomes(&mut self.state, quarts)
    }
""", """    pub fn noise_biomes(&mut self, quarts: &[[i32; 3]]) -> Vec<Option<String>> {
        self.pipeline.noise_biomes(&mut self.state, quarts)
    }
    /// `Climate.Sampler.sample` at quart positions, quantized as `Climate.TargetPoint` is.
    pub fn climate_points(&mut self, quarts: &[[i32; 3]]) -> Vec<Option<[i64; 6]>> {
        self.pipeline.climate_points(&mut self.state, quarts)
    }
"""),
    ('horizon-world/src/worldgen/generator.rs', """pub struct ProtoChunk {
    pub terrain:""", """thread_local! {
    /// A plugin's generator, on the thread making its world's chunks: the game's steps it leaves out
    /// (1 noise, 2 surface, 4 caves), and what it writes after each step.
    pub static STEPS_LEFT_OUT: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
    pub static AFTER_STEP: std::cell::Cell<Option<fn(&'static str, &mut TerrainChunk<'static, GenBlockStates>)>> = const { std::cell::Cell::new(None) };
}
fn after_step(step: &'static str, chunk: &mut TerrainChunk<'static, GenBlockStates>) {
    if let Some(hook) = AFTER_STEP.get() {
        hook(step, chunk);
    }
}
pub struct ProtoChunk {
    pub terrain:"""),
    ('horizon-world/src/worldgen/generator.rs', """        fill_noise(
            &mut chunk,
            volume,
            self.router.get(RouterField::FinalDensity).as_ref(),
            context,
            aquifer.as_mut(),
            self.settings.default_block,
        )
        .map_err(stage("noise"))?;
        let generation = self.generation();
""", """        if STEPS_LEFT_OUT.get() & 1 == 0 {
            fill_noise(
                &mut chunk,
                volume,
                self.router.get(RouterField::FinalDensity).as_ref(),
                context,
                aquifer.as_mut(),
                self.settings.default_block,
            )
            .map_err(stage("noise"))?;
        }
        after_step("noise", &mut chunk);
        let generation = self.generation();
"""),
    ('horizon-world/src/worldgen/generator.rs', """        if !self.identity_end_surface(pos) {
            build_surface(""", """        if !self.identity_end_surface(pos) && STEPS_LEFT_OUT.get() & 2 == 0 {
            build_surface("""),
    ('horizon-world/src/worldgen/generator.rs', """        worker.timings.surface += lap(&mut clock);
        let carvers = &data.biome_carvers;
""", """        worker.timings.surface += lap(&mut clock);
        after_step("surface", &mut chunk);
        after_step("bedrock", &mut chunk);
        let carvers = &data.biome_carvers;
"""),
    ('horizon-world/src/worldgen/generator.rs', """        if !mask.is_empty() {
            let mut resolver = self.source.resolver(
                &self.climate,
                self.router.get(RouterField::Erosion).as_ref(),
            );
            let mut biome_context = SamplerContext::default();
            apply_carving_mask(""", """        if !mask.is_empty() && STEPS_LEFT_OUT.get() & 4 == 0 {
            let mut resolver = self.source.resolver(
                &self.climate,
                self.router.get(RouterField::Erosion).as_ref(),
            );
            let mut biome_context = SamplerContext::default();
            apply_carving_mask("""),
    ('horizon-world/src/worldgen/generator.rs', """        worker.timings.carvers += lap(&mut clock);
        chunk
            .prime_heightmaps(&[
                HeightmapType::MotionBlocking,
                HeightmapType::MotionBlockingNoLeaves,
                HeightmapType::OceanFloor,
                HeightmapType::WorldSurface,
            ])
            .map_err(stage("heightmaps"))?;
        chunk.set_generation_stage(PreLightChunkStage::Terrain);
""", """        worker.timings.carvers += lap(&mut clock);
        after_step("caves", &mut chunk);
        chunk
            .prime_heightmaps(&[
                HeightmapType::MotionBlocking,
                HeightmapType::MotionBlockingNoLeaves,
                HeightmapType::OceanFloor,
                HeightmapType::WorldSurface,
            ])
            .map_err(stage("heightmaps"))?;
        chunk.set_generation_stage(PreLightChunkStage::Terrain);
"""),
    ('horizon-client/src/game/natural_spawning.rs', """        if !self.world.chunks.contains_key(&(cx, cz)) || self.dim.kind() == Dim::End {
            return;
        }
""", """        if !self.world.chunks.contains_key(&(cx, cz)) || self.dim.kind() == Dim::End || self.dim.created().is_some_and(|world| !world.mobs) {
            return;
        }
"""),
    ('horizon-client/src/game/lan_actions.rs', """            let c = host.commands.remove(0);
            self.run_guest_command(c);
""", """            let c = host.commands.remove(0);
            // A marker that came in during a command: what it's for is done here, not run as a command.
            if crate::game::paper_fam_ev_player::is_mark(&c.command) {
                crate::game::paper_fam_ev_player::run_held(self);
                continue;
            }
            self.run_guest_command(c);
"""),
    ('horizon-client/src/game/end.rs', """    areas: Vec<(i32, i32)>,
""", """    pub(super) areas: Vec<(i32, i32)>,
"""),
    ('horizon-world/src/worldgen/biome.rs', """impl<'a> BiomeResolver<'a> {
    pub fn sample(
""", """impl<'a> BiomeResolver<'a> {
    /// `Climate.Sampler.sample`, for a plugin's `BiomeProvider`.
    pub fn point(&self, context: &mut SamplerContext, quart: [i32; 3]) -> Option<super::climate::TargetPoint> {
        self.climate.sample(context, quart).ok()
    }
    pub fn sample(
"""),
    ('horizon-world/src/worldgen/biome_source.rs', """pub use runtime::{BiomeSourceError, BiomeSourceResolver, ResolvedBiomeSource};""", """pub use runtime::{BIOME_HOOK, BiomeHook, BiomeSourceError, BiomeSourceResolver, ResolvedBiomeSource};"""),
    ('horizon-world/src/worldgen/biome_source/runtime.rs', """#[derive(Debug)]
pub enum BiomeSourceResolver<'a> {
""", """/// Quart, the game's biome there and its climate: the biome to use, or none for the game's.
pub type BiomeHook = fn([i32; 3], &str, &mut dyn FnMut() -> [i64; 6]) -> Option<&'static str>;
thread_local! {
    /// A plugin's `BiomeProvider`, on the thread making its world's chunks.
    pub static BIOME_HOOK: std::cell::Cell<Option<BiomeHook>> = const { std::cell::Cell::new(None) };
}
fn point_array(point: super::super::climate::TargetPoint) -> [i64; 6] {
    [point.temperature, point.humidity, point.continentalness, point.erosion, point.depth, point.weirdness]
}
#[derive(Debug)]
pub enum BiomeSourceResolver<'a> {
"""),
    ('horizon-world/src/worldgen/biome_source/runtime.rs', """        Ok(match self {
            Self::Fixed(source) => source.biome.as_str(),
            Self::Checkerboard(source) => source.noise_biome(quart)?.as_str(),
            Self::End(source, erosion) => source.noise_biome(quart, *erosion, context)?.as_str(),
            Self::MultiNoise(source) => source.sample(context, quart)?,
        })
    }
""", """        let biome: &'a str = match self {
            Self::Fixed(source) => source.biome.as_str(),
            Self::Checkerboard(source) => source.noise_biome(quart)?.as_str(),
            Self::End(source, erosion) => source.noise_biome(quart, *erosion, context)?.as_str(),
            Self::MultiNoise(source) => source.sample(context, quart)?,
        };
        let Some(hook) = BIOME_HOOK.get() else {
            return Ok(biome);
        };
        let this = &*self;
        let mut point = || match this {
            Self::MultiNoise(source) => source.point(context, quart).map_or([0; 6], point_array),
            Self::End(_, erosion) => point_array(super::super::climate::TargetPoint::from_values([0., 0., 0., erosion.sample_value(context, quart.map(|at| at.wrapping_shl(2))).unwrap_or(0.), 0., 0.])),
            _ => [0; 6],
        };
        Ok(hook(quart, biome, &mut point).unwrap_or(biome))
    }
"""),
    ('horizon-world/src/worldgen/biome_source/runtime.rs', """                return Ok(source.sample_prepared(volume, quart)?);
""", """                let biome: &'a str = source.sample_prepared(volume, quart)?;
                return Ok(match BIOME_HOOK.get() {
                    Some(hook) => hook(quart, biome, &mut || volume.target(quart).map_or([0; 6], point_array)).unwrap_or(biome),
                    None => biome,
                });
"""),
    ('horizon-client/src/vanillagen.rs', """    pub fn noise_biomes(&self, quarts: &[[i32; 3]]) -> Vec<Option<String>> {
""", """    pub fn climate_points(&self, quarts: &[[i32; 3]]) -> Vec<Option<[i64; 6]>> {
        let mut worker = self.query.lock().unwrap_or_else(|e| e.into_inner());
        let worker = worker.get_or_insert_with(|| self.generator.worker());
        worker.climate_points(quarts)
    }
    pub fn noise_biomes(&self, quarts: &[[i32; 3]]) -> Vec<Option<String>> {
"""),
]
