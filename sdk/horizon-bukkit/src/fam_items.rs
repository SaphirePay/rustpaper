//! Family items: see plugin-tools/tools/family. What's left of `org.bukkit.block`, `org.bukkit.inventory`,
//! `org.bukkit.potion` and Paper's data components, most used first.
#![allow(missing_docs, clippy::wrong_self_convention)]
use crate::block_data::{BlockData, DataClass};
use crate::blocks;
use crate::core2::VoxelShape;
use crate::persistence::NamespacedKey;
use crate::util::{RayTraceResult, Vector};
use crate::{Block, BlockState, Color, ItemStack, Location, Material, Player, World};
use horizon_mod::{Value, query, run_command};
use serde_json::{Map, json};
use std::collections::BTreeMap;

// ---- A block's facts ----

/// The height of the block behind a state that isn't placed.
pub(crate) const UNPLACED_Y: i32 = -100_000;

/// `BlockBehaviour.getInteractionShape` of the four blocks that have one of their own.
fn interaction_boxes(state: &str) -> Option<Vec<[f64; 6]>> {
    const SIXTEENTH: f64 = 1.0 / 16.0;
    let name = state.split('[').next().unwrap_or_default().trim_start_matches("minecraft:");
    if name == "composter" || name == "scaffolding" {
        return Some(vec![[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]]);
    }
    if name.ends_with("cauldron") {
        return Some(vec![[2.0 * SIXTEENTH, 4.0 * SIXTEENTH, 2.0 * SIXTEENTH, 14.0 * SIXTEENTH, 1.0, 14.0 * SIXTEENTH]]);
    }
    if name != "hopper" {
        return None;
    }
    let mut boxes = vec![[2.0 * SIXTEENTH, 11.0 * SIXTEENTH, 2.0 * SIXTEENTH, 14.0 * SIXTEENTH, 1.0, 14.0 * SIXTEENTH]];
    let facing = state.split("facing=").nth(1).unwrap_or("down").split([',', ']']).next().unwrap_or("down");
    let (y0, y1) = (8.0 * SIXTEENTH, 10.0 * SIXTEENTH);
    let (near, far, low, high) = (0.0, 4.0 * SIXTEENTH, 6.0 * SIXTEENTH, 10.0 * SIXTEENTH);
    match facing {
        "north" => boxes.push([low, y0, near, high, y1, far]),
        "south" => boxes.push([low, y0, 1.0 - far, high, y1, 1.0 - near]),
        "west" => boxes.push([near, y0, low, far, y1, high]),
        "east" => boxes.push([1.0 - far, y0, low, 1.0 - near, y1, high]),
        _ => {}
    }
    Some(boxes)
}

/// `FlowingFluid.getOwnHeight`: a source and a falling fluid stand 8/9 high, a flowing one by its level.
fn fluid_height(facts: &Value) -> f64 {
    let level = facts["fluid_level"].as_f64().unwrap_or(0.0);
    (if flag(&facts["fluid_falling"]) || level == 0.0 { 8.0 } else { 8.0 - level }) / 9.0
}

/// Answers of the game that never change while it runs (a material's or a block state's facts), asked once each:
/// a query from a plugin's own thread waits a tick for the game, and CoreProtect's writer asked these for every row
/// (26 s of every 30 s waiting, every lookup "Database busy"). An answer that is an error is not kept.
pub(crate) fn fixed_answer(cache: &'static std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, Value>>>, key: &str, ask: impl FnOnce() -> Value) -> Value {
    let map = cache.get_or_init(Default::default);
    if let Some(known) = map.lock().unwrap_or_else(|e| e.into_inner()).get(key) {
        return known.clone();
    }
    let answer = ask();
    if !answer.is_null() && answer.get("error").is_none() {
        map.lock().unwrap_or_else(|e| e.into_inner()).insert(key.to_owned(), answer.clone());
    }
    answer
}

/// What the game knows of a block state, wherever it stands.
fn facts_of(block: &str) -> Value {
    static FACTS: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, Value>>> = std::sync::OnceLock::new();
    fixed_answer(&FACTS, block, || query(json!({ "op": "items_block", "block": block })))
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_owned()
}

fn number(value: &Value) -> f32 {
    value.as_f64().unwrap_or(0.0) as f32
}

fn flag(value: &Value) -> bool {
    value.as_bool().unwrap_or(false)
}

fn sound_key(id: &str) -> String {
    id.trim_start_matches("minecraft:").replace('.', "_").to_ascii_uppercase()
}

/// `org.bukkit.SoundGroup` and Paper's `BlockSoundGroup`: a block's sounds, by their game ids.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundGroup {
    volume: f32,
    pitch: f32,
    break_sound: String,
    step_sound: String,
    place_sound: String,
    hit_sound: String,
    fall_sound: String,
}

impl SoundGroup {
    fn of(facts: &Value) -> Self {
        Self {
            volume: number(&facts["volume"]),
            pitch: number(&facts["pitch"]),
            break_sound: text(&facts["break_sound"]),
            step_sound: text(&facts["step_sound"]),
            place_sound: text(&facts["place_sound"]),
            hit_sound: text(&facts["hit_sound"]),
            fall_sound: text(&facts["fall_sound"]),
        }
    }
    /// bukkit: org/bukkit/SoundGroup.getVolume
    pub fn get_volume(&self) -> f32 {
        self.volume
    }
    /// bukkit: org/bukkit/SoundGroup.getPitch
    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }
    /// bukkit: org/bukkit/SoundGroup.getBreakSound, com/destroystokyo/paper/block/BlockSoundGroup.getBreakSound
    pub fn get_break_sound(&self) -> String {
        sound_key(&self.break_sound)
    }
    /// bukkit: org/bukkit/SoundGroup.getStepSound, com/destroystokyo/paper/block/BlockSoundGroup.getStepSound
    pub fn get_step_sound(&self) -> String {
        sound_key(&self.step_sound)
    }
    /// bukkit: org/bukkit/SoundGroup.getPlaceSound, com/destroystokyo/paper/block/BlockSoundGroup.getPlaceSound
    pub fn get_place_sound(&self) -> String {
        sound_key(&self.place_sound)
    }
    /// bukkit: org/bukkit/SoundGroup.getHitSound, com/destroystokyo/paper/block/BlockSoundGroup.getHitSound
    pub fn get_hit_sound(&self) -> String {
        sound_key(&self.hit_sound)
    }
    /// bukkit: org/bukkit/SoundGroup.getFallSound, com/destroystokyo/paper/block/BlockSoundGroup.getFallSound
    pub fn get_fall_sound(&self) -> String {
        sound_key(&self.fall_sound)
    }
}

fn voxel_shape(facts: &Value) -> VoxelShape {
    let boxes = facts["shape"].as_array().cloned().unwrap_or_default();
    VoxelShape(boxes.iter().filter_map(|row| {
        let at = |i: usize| row[i].as_f64();
        Some([at(0)?, at(1)?, at(2)?, at(3)?, at(4)?, at(5)?])
    }).collect())
}

fn stacks_of(value: &Value) -> Vec<ItemStack> {
    value.as_array().into_iter().flatten().filter_map(ItemStack::read).collect()
}

/// Paper's `speed + level * level + 1` for an efficient tool that has a speed at all.
fn destroy_speed(block: &str, tool: &ItemStack, enchants: bool) -> f32 {
    let answer = query(json!({ "op": "items_block_tool", "block": block, "tool": tool.to_json() }));
    let speed = number(&answer["speed"]);
    let level = answer["efficiency"].as_i64().unwrap_or(0) as f32;
    if enchants && speed > 1.0 && level > 0.0 { speed + level * level + 1.0 } else { speed }
}

/// `Material::of` for an empty answer: the game's name for no item.
fn material_of(value: &Value) -> Option<Material> {
    value.as_str().filter(|id| *id != "minecraft:air").map(Material::of)
}

impl Block {
    fn at(&self) -> Value {
        json!({ "dimension": self.get_world().dimension(), "pos": [self.get_x(), self.get_y(), self.get_z()] })
    }
    fn asked(&self, op: &str, mut request: Value) -> Value {
        request["op"] = json!(op);
        request["dimension"] = json!(self.get_world().dimension());
        request["pos"] = json!([self.get_x(), self.get_y(), self.get_z()]);
        query(request)
    }
    /// What the game knows of the state here.
    fn state_facts(&self) -> Value {
        self.asked("items_block", json!({}))
    }
    /// bukkit: org/bukkit/block/Block.getDrops
    pub fn get_drops(&self, tool: Option<&ItemStack>) -> Vec<ItemStack> {
        self.get_drops_for(tool, None)
    }
    /// bukkit: org/bukkit/block/Block.getDrops
    ///
    /// With `entity` as the one breaking it: `LootContextParams.THIS_ENTITY`.
    pub fn get_drops_for(&self, tool: Option<&ItemStack>, entity: Option<&crate::entity::Entity>) -> Vec<ItemStack> {
        stacks_of(&self.asked("items_block_drops", json!({ "tool": tool.map(ItemStack::to_json), "entity": entity.is_some() })))
    }
    /// bukkit: org/bukkit/block/Block.getPistonMoveReaction
    pub fn get_piston_move_reaction(&self) -> String {
        text(&self.state_facts()["push"])
    }
    /// bukkit: org/bukkit/block/Block.randomTick
    pub fn random_tick(&self) {
        self.asked("items_block_tick", json!({ "kind": "random" }));
    }
    /// bukkit: org/bukkit/block/Block.fluidTick
    pub fn fluid_tick(&self) {
        self.asked("items_block_tick", json!({ "kind": "fluid" }));
    }
    /// bukkit: org/bukkit/block/Block.getBlockSoundGroup, org/bukkit/block/Block.getSoundGroup
    pub fn get_block_sound_group(&self) -> SoundGroup {
        SoundGroup::of(&self.state_facts())
    }
    /// bukkit: org/bukkit/block/Block.rayTrace
    ///
    /// `BlockGetter.clip` at this block: `NEVER`, `SOURCE_ONLY` or `ALWAYS` for its fluid.
    pub fn ray_trace(&self, start: &Location, direction: &Vector, distance: f64, fluid_mode: &str) -> Option<RayTraceResult> {
        let facts = self.state_facts();
        let mut block = self.clip_boxes(start, direction, distance, &voxel_shape(&facts).0);
        // `clipWithInteractionOverride`: a nearer hit on the interaction shape tells the face.
        if let (Some(hit), Some(boxes)) = (block.as_mut(), interaction_boxes(&self.get_block_data())) {
            let origin = start.to_vector();
            if let Some(inner) = self.clip_boxes(start, direction, distance, &boxes)
                && inner.get_hit_position().distance_squared(&origin) < hit.get_hit_position().distance_squared(&origin)
            {
                hit.face = inner.face;
            }
        }
        let picks = match fluid_mode {
            "ALWAYS" => !facts["fluid"].is_null(),
            "SOURCE_ONLY" => !facts["fluid"].is_null() && !flag(&facts["fluid_falling"]) && facts["fluid_level"].as_i64().unwrap_or(0) == 0,
            _ => false,
        };
        let fluid = picks.then(|| {
            let above = self.get_world().get_block_at(self.get_x(), self.get_y() + 1, self.get_z()).state_facts();
            let height = if above["fluid"] == facts["fluid"] { 1.0 } else { fluid_height(&facts) };
            self.clip_boxes(start, direction, distance, &[[0.0, 0.0, 0.0, 1.0, height, 1.0]])
        }).flatten();
        let origin = start.to_vector();
        let far = |hit: &Option<RayTraceResult>| hit.as_ref().map_or(f64::MAX, |hit| hit.get_hit_position().distance_squared(&origin));
        if far(&block) <= far(&fluid) { block } else { fluid }
    }
    /// `VoxelShape.clip`: a ray that starts inside hits where it starts, on the face it came from.
    fn clip_boxes(&self, start: &Location, direction: &Vector, distance: f64, boxes: &[[f64; 6]]) -> Option<RayTraceResult> {
        let (x, y, z) = (f64::from(self.get_x()), f64::from(self.get_y()), f64::from(self.get_z()));
        let origin = start.to_vector();
        let length = direction.length();
        if boxes.is_empty() || length == 0.0 || distance * distance < 1.0e-7 {
            return None;
        }
        let scale = distance / length;
        let reach = [direction.get_x() * scale, direction.get_y() * scale, direction.get_z() * scale];
        let probe = [origin.get_x() + reach[0] * 0.001 - x, origin.get_y() + reach[1] * 0.001 - y, origin.get_z() + reach[2] * 0.001 - z];
        if boxes.iter().any(|[x0, y0, z0, x1, y1, z1]| (*x0..*x1).contains(&probe[0]) && (*y0..*y1).contains(&probe[1]) && (*z0..*z1).contains(&probe[2])) {
            let axis = (0..3).max_by(|a, b| reach[*a].abs().total_cmp(&reach[*b].abs())).unwrap_or(0);
            let face = match (axis, reach[axis] > 0.0) {
                (0, true) => "WEST",
                (0, false) => "EAST",
                (1, true) => "DOWN",
                (1, false) => "UP",
                (_, true) => "NORTH",
                (_, false) => "SOUTH",
            };
            return Some(RayTraceResult { position: Vector::new(probe[0] + x, probe[1] + y, probe[2] + z), block: Some(self.clone()), face: Some(face), entity: None });
        }
        let mut best: Option<RayTraceResult> = None;
        for [x0, y0, z0, x1, y1, z1] in boxes {
            let shape = crate::util::BoundingBox::new(x + x0, y + y0, z + z0, x + x1, y + y1, z + z1);
            let Some(mut hit) = shape.ray_trace(&origin, direction, distance) else {
                continue;
            };
            let far = |result: &RayTraceResult| result.get_hit_position().distance_squared(&origin);
            if best.as_ref().is_none_or(|known| far(&hit) < far(known)) {
                hit.block = Some(self.clone());
                best = Some(hit);
            }
        }
        best
    }
    /// bukkit: org/bukkit/block/Block.setBiome
    ///
    /// Into the chunk's cell, if it's loaded. Clients see it when they next load the chunk, as in Java.
    pub fn set_biome(&self, biome: &str) {
        self.asked("items_set_biome", json!({ "biome": biome }));
    }
    /// bukkit: org/bukkit/block/Block.getDestroySpeed
    pub fn get_destroy_speed(&self, tool: &ItemStack, consider_enchants: bool) -> f32 {
        destroy_speed(&self.get_block_data(), tool, consider_enchants)
    }
    /// bukkit: org/bukkit/block/Block.getBreakSpeed
    pub fn get_break_speed(&self, player: &Player) -> f32 {
        number(&self.asked("items_break_speed", json!({ "uuid": player.uuid() })))
    }
    /// bukkit: org/bukkit/block/Block.isValidTool, org/bukkit/block/Block.isPreferredTool
    pub fn is_preferred_tool(&self, tool: &ItemStack) -> bool {
        flag(&query(json!({ "op": "items_block_tool", "block": self.get_block_data(), "tool": tool.to_json() }))["harvests"])
    }
    /// bukkit: org/bukkit/block/Block.isBurnable
    pub fn is_burnable(&self) -> bool {
        flag(&self.state_facts()["ignited_by_lava"])
    }
    /// bukkit: org/bukkit/block/Block.isBuildable
    pub fn is_buildable(&self) -> bool {
        flag(&self.state_facts()["solid"])
    }
    /// bukkit: org/bukkit/block/Block.isBlockFacePowered
    pub fn is_block_face_powered(&self, face: &str) -> bool {
        flag(&self.asked("items_power", json!({ "kind": "face", "face": face })))
    }
    /// bukkit: org/bukkit/block/Block.isBlockFaceIndirectlyPowered
    pub fn is_block_face_indirectly_powered(&self, face: &str) -> bool {
        flag(&self.asked("items_power", json!({ "kind": "face_indirect", "face": face })))
    }
    /// bukkit: org/bukkit/block/Block.isBlockPowered
    pub fn is_block_powered(&self) -> bool {
        flag(&self.asked("items_power", json!({ "kind": "direct" })))
    }
    /// bukkit: org/bukkit/block/Block.isBlockIndirectlyPowered
    pub fn is_block_indirectly_powered(&self) -> bool {
        flag(&self.asked("items_power", json!({ "kind": "indirect" })))
    }
    /// bukkit: org/bukkit/block/Block.getBlockPower
    ///
    /// From `face`, or `None` for `BlockFace.SELF`: all six.
    pub fn get_block_power_from(&self, face: Option<&str>) -> i32 {
        self.asked("items_power", json!({ "kind": "level", "face": face })).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Block.getData
    ///
    /// The 1.12 data value of the state, from the game's own flattening table.
    pub fn get_data_value(&self) -> u8 {
        legacy_of(self.get_block_data().as_str()).1
    }
    /// bukkit: org/bukkit/block/Block.canPlace
    pub fn can_place(&self, data: &BlockData) -> bool {
        data.is_supported(self)
    }
}

impl BlockState {
    /// bukkit: org/bukkit/block/BlockState.getDrops
    pub fn get_drops(&self, tool: Option<&ItemStack>) -> Vec<ItemStack> {
        self.get_drops_for(tool, None)
    }
    /// bukkit: org/bukkit/block/BlockState.getDrops
    ///
    /// With `entity` as the one breaking it.
    pub fn get_drops_for(&self, tool: Option<&ItemStack>, entity: Option<&crate::entity::Entity>) -> Vec<ItemStack> {
        let block = self.get_block();
        stacks_of(&query(json!({ "op": "items_block_drops", "block": self.get_block_data(), "dimension": block.get_world().dimension(), "pos": [block.get_x(), block.get_y(), block.get_z()], "tool": tool.map(ItemStack::to_json), "entity": entity.is_some() })))
    }
    /// bukkit: org/bukkit/block/BlockState.isCollidable
    pub fn is_collidable(&self) -> bool {
        flag(&facts_of(self.get_block_data())["collides"])
    }
    /// bukkit: org/bukkit/block/BlockState.isSuffocating
    pub fn is_suffocating(&self) -> bool {
        let facts = facts_of(self.get_block_data());
        flag(&facts["occluding"]) && flag(&facts["full_collision"])
    }
    /// bukkit: org/bukkit/block/BlockState.getRawData
    pub fn get_raw_data(&self) -> u8 {
        legacy_of(self.get_block_data()).1
    }
    /// bukkit: org/bukkit/block/BlockState.setRawData
    pub fn set_raw_data(&mut self, value: u8) {
        let state = query(json!({ "op": "items_legacy_state", "block": self.get_block_data(), "data": value }));
        self.set_block_data(&text(&state));
    }
    /// bukkit: org/bukkit/block/BlockState.getData
    pub fn get_material_data(&self) -> MaterialData {
        let (id, data) = legacy_of(self.get_block_data());
        MaterialData { item_type: Material::of(LEGACY_NAMES[id as usize]), data }
    }
}

/// `CraftLegacy.toLegacy(state)`: the old block id and data value.
fn legacy_of(state: &str) -> (i64, u8) {
    let answer = query(json!({ "op": "items_legacy_data", "block": state }));
    (answer[0].as_i64().unwrap_or(0), answer[1].as_u64().unwrap_or(0) as u8)
}

/// The `MaterialData` a plugin set on a stack: not part of what makes two stacks equal, as in Java.
#[derive(Clone, Debug, Default)]
pub struct LegacySlot(pub Option<MaterialData>);

impl PartialEq for LegacySlot {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for LegacySlot {}

impl ItemStack {
    /// bukkit: org/bukkit/inventory/ItemStack.setData
    pub fn set_material_data(&mut self, data: Option<MaterialData>) {
        self.legacy = LegacySlot(data);
    }
    /// What `set_material_data` gave.
    pub fn get_material_data(&self) -> Option<&MaterialData> {
        self.legacy.0.as_ref()
    }
}

/// The slots of a state of no block: Java's snapshot inventory, which every copy of the state shares.
#[derive(Clone, Debug)]
pub struct HeldSlots(std::sync::Arc<std::sync::Mutex<Vec<Option<ItemStack>>>>);

impl PartialEq for HeldSlots {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for HeldSlots {}

impl HeldSlots {
    fn get(&self) -> Vec<Option<ItemStack>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

/// What a state of no block has in the world's place: a `BlockStateMeta`'s.
#[derive(Clone, Debug, Default)]
pub struct Held {
    pub(crate) slots: Option<HeldSlots>,
    pub(crate) kind: Option<Material>,
}

impl PartialEq for Held {
    fn eq(&self, other: &Self) -> bool {
        self.slots == other.slots && self.kind == other.kind
    }
}

/// `org.bukkit.material.MaterialData`, as the old API's calls hand it around: the old material and its data value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialData {
    pub item_type: Material,
    pub data: u8,
}

/// `Material`'s old constants of blocks, by id. 253 and 254 were never used.
const LEGACY_NAMES: [&str; 256] = ["LEGACY_AIR", "LEGACY_STONE", "LEGACY_GRASS", "LEGACY_DIRT", "LEGACY_COBBLESTONE", "LEGACY_WOOD", "LEGACY_SAPLING", "LEGACY_BEDROCK", "LEGACY_WATER", "LEGACY_STATIONARY_WATER", "LEGACY_LAVA", "LEGACY_STATIONARY_LAVA", "LEGACY_SAND", "LEGACY_GRAVEL", "LEGACY_GOLD_ORE", "LEGACY_IRON_ORE", "LEGACY_COAL_ORE", "LEGACY_LOG", "LEGACY_LEAVES", "LEGACY_SPONGE", "LEGACY_GLASS", "LEGACY_LAPIS_ORE", "LEGACY_LAPIS_BLOCK", "LEGACY_DISPENSER", "LEGACY_SANDSTONE", "LEGACY_NOTE_BLOCK", "LEGACY_BED_BLOCK", "LEGACY_POWERED_RAIL", "LEGACY_DETECTOR_RAIL", "LEGACY_PISTON_STICKY_BASE", "LEGACY_WEB", "LEGACY_LONG_GRASS", "LEGACY_DEAD_BUSH", "LEGACY_PISTON_BASE", "LEGACY_PISTON_EXTENSION", "LEGACY_WOOL", "LEGACY_PISTON_MOVING_PIECE", "LEGACY_YELLOW_FLOWER", "LEGACY_RED_ROSE", "LEGACY_BROWN_MUSHROOM", "LEGACY_RED_MUSHROOM", "LEGACY_GOLD_BLOCK", "LEGACY_IRON_BLOCK", "LEGACY_DOUBLE_STEP", "LEGACY_STEP", "LEGACY_BRICK", "LEGACY_TNT", "LEGACY_BOOKSHELF", "LEGACY_MOSSY_COBBLESTONE", "LEGACY_OBSIDIAN", "LEGACY_TORCH", "LEGACY_FIRE", "LEGACY_MOB_SPAWNER", "LEGACY_WOOD_STAIRS", "LEGACY_CHEST", "LEGACY_REDSTONE_WIRE", "LEGACY_DIAMOND_ORE", "LEGACY_DIAMOND_BLOCK", "LEGACY_WORKBENCH", "LEGACY_CROPS", "LEGACY_SOIL", "LEGACY_FURNACE", "LEGACY_BURNING_FURNACE", "LEGACY_SIGN_POST", "LEGACY_WOODEN_DOOR", "LEGACY_LADDER", "LEGACY_RAILS", "LEGACY_COBBLESTONE_STAIRS", "LEGACY_WALL_SIGN", "LEGACY_LEVER", "LEGACY_STONE_PLATE", "LEGACY_IRON_DOOR_BLOCK", "LEGACY_WOOD_PLATE", "LEGACY_REDSTONE_ORE", "LEGACY_GLOWING_REDSTONE_ORE", "LEGACY_REDSTONE_TORCH_OFF", "LEGACY_REDSTONE_TORCH_ON", "LEGACY_STONE_BUTTON", "LEGACY_SNOW", "LEGACY_ICE", "LEGACY_SNOW_BLOCK", "LEGACY_CACTUS", "LEGACY_CLAY", "LEGACY_SUGAR_CANE_BLOCK", "LEGACY_JUKEBOX", "LEGACY_FENCE", "LEGACY_PUMPKIN", "LEGACY_NETHERRACK", "LEGACY_SOUL_SAND", "LEGACY_GLOWSTONE", "LEGACY_PORTAL", "LEGACY_JACK_O_LANTERN", "LEGACY_CAKE_BLOCK", "LEGACY_DIODE_BLOCK_OFF", "LEGACY_DIODE_BLOCK_ON", "LEGACY_STAINED_GLASS", "LEGACY_TRAP_DOOR", "LEGACY_MONSTER_EGGS", "LEGACY_SMOOTH_BRICK", "LEGACY_HUGE_MUSHROOM_1", "LEGACY_HUGE_MUSHROOM_2", "LEGACY_IRON_FENCE", "LEGACY_THIN_GLASS", "LEGACY_MELON_BLOCK", "LEGACY_PUMPKIN_STEM", "LEGACY_MELON_STEM", "LEGACY_VINE", "LEGACY_FENCE_GATE", "LEGACY_BRICK_STAIRS", "LEGACY_SMOOTH_STAIRS", "LEGACY_MYCEL", "LEGACY_WATER_LILY", "LEGACY_NETHER_BRICK", "LEGACY_NETHER_FENCE", "LEGACY_NETHER_BRICK_STAIRS", "LEGACY_NETHER_WARTS", "LEGACY_ENCHANTMENT_TABLE", "LEGACY_BREWING_STAND", "LEGACY_CAULDRON", "LEGACY_ENDER_PORTAL", "LEGACY_ENDER_PORTAL_FRAME", "LEGACY_ENDER_STONE", "LEGACY_DRAGON_EGG", "LEGACY_REDSTONE_LAMP_OFF", "LEGACY_REDSTONE_LAMP_ON", "LEGACY_WOOD_DOUBLE_STEP", "LEGACY_WOOD_STEP", "LEGACY_COCOA", "LEGACY_SANDSTONE_STAIRS", "LEGACY_EMERALD_ORE", "LEGACY_ENDER_CHEST", "LEGACY_TRIPWIRE_HOOK", "LEGACY_TRIPWIRE", "LEGACY_EMERALD_BLOCK", "LEGACY_SPRUCE_WOOD_STAIRS", "LEGACY_BIRCH_WOOD_STAIRS", "LEGACY_JUNGLE_WOOD_STAIRS", "LEGACY_COMMAND", "LEGACY_BEACON", "LEGACY_COBBLE_WALL", "LEGACY_FLOWER_POT", "LEGACY_CARROT", "LEGACY_POTATO", "LEGACY_WOOD_BUTTON", "LEGACY_SKULL", "LEGACY_ANVIL", "LEGACY_TRAPPED_CHEST", "LEGACY_GOLD_PLATE", "LEGACY_IRON_PLATE", "LEGACY_REDSTONE_COMPARATOR_OFF", "LEGACY_REDSTONE_COMPARATOR_ON", "LEGACY_DAYLIGHT_DETECTOR", "LEGACY_REDSTONE_BLOCK", "LEGACY_QUARTZ_ORE", "LEGACY_HOPPER", "LEGACY_QUARTZ_BLOCK", "LEGACY_QUARTZ_STAIRS", "LEGACY_ACTIVATOR_RAIL", "LEGACY_DROPPER", "LEGACY_STAINED_CLAY", "LEGACY_STAINED_GLASS_PANE", "LEGACY_LEAVES_2", "LEGACY_LOG_2", "LEGACY_ACACIA_STAIRS", "LEGACY_DARK_OAK_STAIRS", "LEGACY_SLIME_BLOCK", "LEGACY_BARRIER", "LEGACY_IRON_TRAPDOOR", "LEGACY_PRISMARINE", "LEGACY_SEA_LANTERN", "LEGACY_HAY_BLOCK", "LEGACY_CARPET", "LEGACY_HARD_CLAY", "LEGACY_COAL_BLOCK", "LEGACY_PACKED_ICE", "LEGACY_DOUBLE_PLANT", "LEGACY_STANDING_BANNER", "LEGACY_WALL_BANNER", "LEGACY_DAYLIGHT_DETECTOR_INVERTED", "LEGACY_RED_SANDSTONE", "LEGACY_RED_SANDSTONE_STAIRS", "LEGACY_DOUBLE_STONE_SLAB2", "LEGACY_STONE_SLAB2", "LEGACY_SPRUCE_FENCE_GATE", "LEGACY_BIRCH_FENCE_GATE", "LEGACY_JUNGLE_FENCE_GATE", "LEGACY_DARK_OAK_FENCE_GATE", "LEGACY_ACACIA_FENCE_GATE", "LEGACY_SPRUCE_FENCE", "LEGACY_BIRCH_FENCE", "LEGACY_JUNGLE_FENCE", "LEGACY_DARK_OAK_FENCE", "LEGACY_ACACIA_FENCE", "LEGACY_SPRUCE_DOOR", "LEGACY_BIRCH_DOOR", "LEGACY_JUNGLE_DOOR", "LEGACY_ACACIA_DOOR", "LEGACY_DARK_OAK_DOOR", "LEGACY_END_ROD", "LEGACY_CHORUS_PLANT", "LEGACY_CHORUS_FLOWER", "LEGACY_PURPUR_BLOCK", "LEGACY_PURPUR_PILLAR", "LEGACY_PURPUR_STAIRS", "LEGACY_PURPUR_DOUBLE_SLAB", "LEGACY_PURPUR_SLAB", "LEGACY_END_BRICKS", "LEGACY_BEETROOT_BLOCK", "LEGACY_GRASS_PATH", "LEGACY_END_GATEWAY", "LEGACY_COMMAND_REPEATING", "LEGACY_COMMAND_CHAIN", "LEGACY_FROSTED_ICE", "LEGACY_MAGMA", "LEGACY_NETHER_WART_BLOCK", "LEGACY_RED_NETHER_BRICK", "LEGACY_BONE_BLOCK", "LEGACY_STRUCTURE_VOID", "LEGACY_OBSERVER", "LEGACY_WHITE_SHULKER_BOX", "LEGACY_ORANGE_SHULKER_BOX", "LEGACY_MAGENTA_SHULKER_BOX", "LEGACY_LIGHT_BLUE_SHULKER_BOX", "LEGACY_YELLOW_SHULKER_BOX", "LEGACY_LIME_SHULKER_BOX", "LEGACY_PINK_SHULKER_BOX", "LEGACY_GRAY_SHULKER_BOX", "LEGACY_SILVER_SHULKER_BOX", "LEGACY_CYAN_SHULKER_BOX", "LEGACY_PURPLE_SHULKER_BOX", "LEGACY_BLUE_SHULKER_BOX", "LEGACY_BROWN_SHULKER_BOX", "LEGACY_GREEN_SHULKER_BOX", "LEGACY_RED_SHULKER_BOX", "LEGACY_BLACK_SHULKER_BOX", "LEGACY_WHITE_GLAZED_TERRACOTTA", "LEGACY_ORANGE_GLAZED_TERRACOTTA", "LEGACY_MAGENTA_GLAZED_TERRACOTTA", "LEGACY_LIGHT_BLUE_GLAZED_TERRACOTTA", "LEGACY_YELLOW_GLAZED_TERRACOTTA", "LEGACY_LIME_GLAZED_TERRACOTTA", "LEGACY_PINK_GLAZED_TERRACOTTA", "LEGACY_GRAY_GLAZED_TERRACOTTA", "LEGACY_SILVER_GLAZED_TERRACOTTA", "LEGACY_CYAN_GLAZED_TERRACOTTA", "LEGACY_PURPLE_GLAZED_TERRACOTTA", "LEGACY_BLUE_GLAZED_TERRACOTTA", "LEGACY_BROWN_GLAZED_TERRACOTTA", "LEGACY_GREEN_GLAZED_TERRACOTTA", "LEGACY_RED_GLAZED_TERRACOTTA", "LEGACY_BLACK_GLAZED_TERRACOTTA", "LEGACY_CONCRETE", "LEGACY_CONCRETE_POWDER", "LEGACY_AIR", "LEGACY_AIR", "LEGACY_STRUCTURE_BLOCK"];

impl BlockData {
    fn facts(&self) -> Value {
        facts_of(&self.get_as_string())
    }
    /// bukkit: org/bukkit/block/data/BlockData.getSoundGroup
    pub fn get_sound_group(&self) -> SoundGroup {
        SoundGroup::of(&self.facts())
    }
    /// bukkit: org/bukkit/block/data/BlockData.createBlockState
    ///
    /// Not placed: `update` does nothing and `get_block` panics, as Java's throws.
    pub fn create_block_state(&self) -> BlockState {
        BlockState::snapshot(World::of("world").get_block_at(0, UNPLACED_Y, 0), &self.get_as_string())
    }
    /// bukkit: org/bukkit/block/data/BlockData.getPlacementMaterial
    pub fn get_placement_material(&self) -> Material {
        material_of(&self.facts()["item"]).unwrap_or_else(|| Material::of("AIR"))
    }
    /// bukkit: org/bukkit/block/data/BlockData.getLightEmission
    pub fn get_light_emission(&self) -> i32 {
        self.facts()["emission"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/data/BlockData.isRandomlyTicked
    pub fn is_randomly_ticked(&self) -> bool {
        flag(&self.facts()["random_ticks"])
    }
    /// bukkit: org/bukkit/block/data/BlockData.isPreferredTool
    pub fn is_preferred_tool(&self, tool: &ItemStack) -> bool {
        flag(&query(json!({ "op": "items_block_tool", "block": self.get_as_string(), "tool": tool.to_json() }))["harvests"])
    }
    /// bukkit: org/bukkit/block/data/BlockData.isOccluding
    pub fn is_occluding(&self) -> bool {
        flag(&self.facts()["occluding"])
    }
    /// bukkit: org/bukkit/block/data/BlockData.isFaceSturdy
    pub fn is_face_sturdy(&self, face: &str, support: &str) -> bool {
        flag(&query(json!({ "op": "items_block_sturdy", "block": self.get_as_string(), "face": face, "support": support })))
    }
    /// bukkit: org/bukkit/block/data/BlockData.rotate
    ///
    /// `rotation` is `NONE`, `CLOCKWISE_90`, `CLOCKWISE_180` or `COUNTERCLOCKWISE_90`.
    pub fn rotate(&mut self, rotation: &str) {
        let name = match rotation {
            "CLOCKWISE_90" => "clockwise_90",
            "CLOCKWISE_180" => "180",
            "COUNTERCLOCKWISE_90" => "counterclockwise_90",
            _ => "none",
        };
        if let Some(changed) = query(json!({ "op": "items_block_transform", "block": self.get_as_string(), "rotation": name })).as_str() {
            *self = Self::parse(changed);
        }
    }
    /// bukkit: org/bukkit/block/data/BlockData.mirror
    ///
    /// `mirror` is `NONE`, `LEFT_RIGHT` or `FRONT_BACK`.
    pub fn mirror(&mut self, mirror: &str) {
        if let Some(changed) = query(json!({ "op": "items_block_transform", "block": self.get_as_string(), "mirror": mirror })).as_str() {
            *self = Self::parse(changed);
        }
    }
    /// bukkit: org/bukkit/block/data/BlockData.getDestroySpeed
    pub fn get_destroy_speed(&self, tool: &ItemStack, consider_enchants: bool) -> f32 {
        destroy_speed(&self.get_as_string(), tool, consider_enchants)
    }
    /// bukkit: org/bukkit/block/data/BlockData.requiresCorrectToolForDrops
    pub fn requires_correct_tool_for_drops(&self) -> bool {
        flag(&self.facts()["requires_tool"])
    }
    /// bukkit: org/bukkit/block/data/BlockData.isSupported
    pub fn is_supported(&self, block: &Block) -> bool {
        flag(&block.asked("items_block_survives", json!({ "block": self.get_as_string() })))
    }
    /// bukkit: org/bukkit/block/data/BlockData.isReplaceable
    pub fn is_replaceable(&self) -> bool {
        flag(&self.facts()["replaceable"])
    }
    /// bukkit: org/bukkit/block/data/BlockData.getCollisionShape
    ///
    /// This state's, with the world around `location`, whatever stands there.
    pub fn get_collision_shape(&self, location: &Location) -> VoxelShape {
        let block = location.get_block();
        let boxes = query(json!({ "op": "items_collision", "block": self.get_as_string(), "dimension": block.get_world().dimension(), "pos": [block.get_x(), block.get_y(), block.get_z()] }));
        VoxelShape(boxes.as_array().into_iter().flatten().filter_map(|row| {
            let at = |index: usize| row[index].as_f64();
            Some([at(0)?, at(1)?, at(2)?, at(3)?, at(4)?, at(5)?])
        }).collect())
    }
    /// bukkit: org/bukkit/block/data/BlockData.getPistonMoveReaction
    pub fn get_piston_move_reaction(&self) -> String {
        text(&self.facts()["push"])
    }
    /// bukkit: org/bukkit/block/data/BlockData.getMapColor
    pub fn get_map_color(&self) -> Color {
        let rgb = self.facts()["map_color"].as_i64().unwrap_or(0);
        Color::from_rgb(((rgb >> 16) & 255) as u8, ((rgb >> 8) & 255) as u8, (rgb & 255) as u8)
    }
}

/// `org.bukkit.block.PistonMoveReaction.getId`.
pub const PISTON_REACTIONS: [&str; 5] = ["MOVE", "BREAK", "BLOCK", "IGNORE", "PUSH_ONLY"];

/// bukkit: org/bukkit/block/PistonMoveReaction.getId
pub fn piston_reaction_id(reaction: &str) -> i32 {
    PISTON_REACTIONS.iter().position(|name| *name == reaction).map_or(-1, |id| id as i32)
}

/// bukkit: org/bukkit/block/PistonMoveReaction.getById
pub fn piston_reaction_by_id(id: i32) -> Option<&'static str> {
    usize::try_from(id).ok().and_then(|id| PISTON_REACTIONS.get(id).copied())
}

// ---- Block types ----

/// `org.bukkit.block.BlockType`, by the game's id. `BlockType$Typed` is the same thing here, as a data
/// class is chosen with `create_block_data().class::<C>()`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockType(pub String);

impl BlockType {
    /// The block of this id.
    pub fn of(id: &str) -> Self {
        Self(Material::of(id).key().to_owned())
    }
    fn facts(&self) -> Value {
        facts_of(&self.0)
    }
    /// bukkit: org/bukkit/block/BlockType.typed
    pub fn typed(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/block/BlockType.asMaterial
    pub fn as_material(&self) -> Material {
        Material::of(&self.0)
    }
    /// bukkit: org/bukkit/block/BlockType.hasItemType
    pub fn has_item_type(&self) -> bool {
        material_of(&self.facts()["item"]).is_some()
    }
    /// bukkit: org/bukkit/block/BlockType.getItemType
    pub fn get_item_type(&self) -> Option<ItemType> {
        material_of(&self.facts()["item"]).map(|item| ItemType(item.key().to_owned()))
    }
    /// bukkit: org/bukkit/block/BlockType.createBlockData, org/bukkit/block/BlockType$Typed.createBlockData
    ///
    /// The block's default state, with `data` (`[facing=east]`) over it. Java's `Consumer` form is the
    /// same data changed by the caller.
    pub fn create_block_data(&self, data: Option<&str>) -> BlockData {
        let state = text(&self.facts()["state"]);
        let base = BlockData::parse(if state.is_empty() { &self.0 } else { &state });
        match data {
            Some(data) => base.merge(&BlockData::parse(&format!("{}{data}", self.0))),
            None => base,
        }
    }
    /// bukkit: org/bukkit/block/BlockType.createBlockDataStates, org/bukkit/block/BlockType$Typed.createBlockDataStates
    pub fn create_block_data_states(&self) -> Vec<BlockData> {
        query(json!({ "op": "items_block_states", "block": self.0 })).as_array().into_iter().flatten().filter_map(Value::as_str).map(BlockData::parse).collect()
    }
    /// bukkit: org/bukkit/block/BlockType.isSolid
    pub fn is_solid(&self) -> bool {
        flag(&self.facts()["solid"]) && !matches!(self.0.as_str(), "minecraft:cobweb" | "minecraft:bamboo_sapling")
    }
    /// bukkit: org/bukkit/block/BlockType.isFlammable
    pub fn is_flammable(&self) -> bool {
        flag(&self.facts()["ignited_by_lava"])
    }
    /// bukkit: org/bukkit/block/BlockType.isBurnable
    pub fn is_burnable(&self) -> bool {
        self.facts()["ignite_odds"].as_i64().unwrap_or(0) > 0
    }
    /// bukkit: org/bukkit/block/BlockType.isEnabledByFeature
    ///
    /// No block of this game needs a feature flag.
    pub fn is_enabled_by_feature(&self, _world: &World) -> bool {
        !self.facts().is_null()
    }
    /// bukkit: org/bukkit/block/BlockType.isOccluding
    pub fn is_occluding(&self) -> bool {
        flag(&self.facts()["occluding"])
    }
    /// bukkit: org/bukkit/block/BlockType.hasGravity
    pub fn has_gravity(&self) -> bool {
        self.facts()["classes"].as_array().is_some_and(|classes| classes.iter().any(|class| class == "FallingBlock"))
    }
    /// bukkit: org/bukkit/block/BlockType.getHardness
    pub fn get_hardness(&self) -> f32 {
        number(&self.facts()["hardness"])
    }
    /// bukkit: org/bukkit/block/BlockType.getBlastResistance
    pub fn get_blast_resistance(&self) -> f32 {
        number(&self.facts()["explosion_resistance"])
    }
    /// bukkit: org/bukkit/block/BlockType.getSlipperiness
    pub fn get_slipperiness(&self) -> f32 {
        number(&self.facts()["friction"])
    }
    /// bukkit: org/bukkit/block/BlockType.isAir
    pub fn is_air(&self) -> bool {
        Material::of(&self.0).is_air()
    }
    /// bukkit: org/bukkit/block/BlockType.getTranslationKey
    pub fn get_translation_key(&self) -> String {
        format!("block.{}", self.0.replace(':', "."))
    }
    /// bukkit: org/bukkit/block/BlockType.hasCollision
    pub fn has_collision(&self) -> bool {
        flag(&self.facts()["collides"])
    }
    /// bukkit: org/bukkit/block/BlockType.getInstrument
    ///
    /// The name of `org.bukkit.Instrument`, the game's own names changed to Bukkit's.
    pub fn get_instrument(&self) -> String {
        instrument_name(&text(&self.facts()["instrument"])).to_owned()
    }
}

/// The game's names of the note block's instruments.
fn horizon_instruments() -> [&'static str; 27] {
    ["harp", "basedrum", "snare", "hat", "bass", "flute", "bell", "guitar", "chime", "xylophone", "iron_xylophone", "cow_bell", "didgeridoo", "bit", "banjo", "pling", "trumpet", "trumpet_exposed", "trumpet_oxidized", "trumpet_weathered", "zombie", "skeleton", "creeper", "dragon", "wither_skeleton", "piglin", "custom_head"]
}

/// Bukkit's name of a note block instrument, from the game's.
fn instrument_name(id: &str) -> &'static str {
    match id {
    "harp" => "PIANO",
    "basedrum" => "BASS_DRUM",
    "snare" => "SNARE_DRUM",
    "hat" => "STICKS",
    "bass" => "BASS_GUITAR",
    "bell" => "BELL",
    "flute" => "FLUTE",
    "guitar" => "GUITAR",
    "chime" => "CHIME",
    "xylophone" => "XYLOPHONE",
    "iron_xylophone" => "IRON_XYLOPHONE",
    "cow_bell" => "COW_BELL",
    "didgeridoo" => "DIDGERIDOO",
    "bit" => "BIT",
    "banjo" => "BANJO",
    "pling" => "PLING",
    "trumpet" => "TRUMPET",
    "trumpet_exposed" => "TRUMPET_EXPOSED",
    "trumpet_oxidized" => "TRUMPET_OXIDIZED",
    "trumpet_weathered" => "TRUMPET_WEATHERED",
    "zombie" => "ZOMBIE",
    "skeleton" => "SKELETON",
    "creeper" => "CREEPER",
    "dragon" => "DRAGON",
    "wither_skeleton" => "WITHER_SKELETON",
    "piglin" => "PIGLIN",
    _ => "CUSTOM_HEAD",
    }
}

// ---- Item types ----

/// `org.bukkit.inventory.ItemType`, by the game's id. `ItemType$Typed` is the same thing here.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemType(pub String);

impl ItemType {
    /// The item of this id.
    pub fn of(id: &str) -> Self {
        Self(Material::of(id).key().to_owned())
    }
    fn info(&self) -> Value {
        query(json!({ "op": "items_item", "item": self.0 }))
    }
    fn default_component(&self, id: &str) -> Value {
        self.info()["components"][id].clone()
    }
    /// bukkit: org/bukkit/inventory/ItemType.typed
    pub fn typed(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/inventory/ItemType.asMaterial
    pub fn as_material(&self) -> Material {
        Material::of(&self.0)
    }
    /// bukkit: org/bukkit/inventory/ItemType.hasBlockType
    pub fn has_block_type(&self) -> bool {
        self.info()["block"].is_string()
    }
    /// bukkit: org/bukkit/inventory/ItemType.getBlockType
    pub fn get_block_type(&self) -> Option<BlockType> {
        self.info()["block"].as_str().map(|state| BlockType::of(state.split('[').next().unwrap_or(state)))
    }
    /// bukkit: org/bukkit/inventory/ItemType.getMaxStackSize
    pub fn get_max_stack_size(&self) -> i32 {
        self.info()["max_stack"].as_i64().unwrap_or(64) as i32
    }
    /// bukkit: org/bukkit/inventory/ItemType.getMaxDurability
    pub fn get_max_durability(&self) -> i16 {
        self.info()["max_damage"].as_i64().unwrap_or(0) as i16
    }
    /// bukkit: org/bukkit/inventory/ItemType.isEdible
    pub fn is_edible(&self) -> bool {
        !self.default_component("minecraft:food").is_null()
    }
    /// bukkit: org/bukkit/inventory/ItemType.isRecord
    pub fn is_record(&self) -> bool {
        !self.default_component("minecraft:jukebox_playable").is_null()
    }
    /// bukkit: org/bukkit/inventory/ItemType.isFuel
    pub fn is_fuel(&self) -> bool {
        !self.info()["fuel"].is_null()
    }
    /// bukkit: org/bukkit/inventory/ItemType.getBurnDuration
    ///
    /// In ticks, in a plain furnace.
    pub fn get_burn_duration(&self) -> i32 {
        query(json!({ "op": "items_burn_time", "item": self.0 })).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/inventory/ItemType.isCompostable
    pub fn is_compostable(&self) -> bool {
        !self.info()["compostable"].is_null()
    }
    /// bukkit: org/bukkit/inventory/ItemType.getCompostChance
    pub fn get_compost_chance(&self) -> f32 {
        match self.info()["compostable"].as_str() {
            Some("minecraft:compostable/low") => 0.3,
            Some("minecraft:compostable/low_medium") => 0.5,
            Some("minecraft:compostable/medium") => 0.65,
            Some("minecraft:compostable/medium_high") => 0.85,
            Some("minecraft:compostable/always_add_one") => 1.0,
            _ => 0.0,
        }
    }
    /// bukkit: org/bukkit/inventory/ItemType.getCreativeCategory
    ///
    /// Java's Javadoc: no longer kept, always this one.
    pub fn get_creative_category(&self) -> &'static str {
        "BUILDING_BLOCKS"
    }
    /// bukkit: org/bukkit/inventory/ItemType.getCraftingRemainingItem
    pub fn get_crafting_remaining_item(&self) -> Option<Self> {
        self.info()["crafting_remainder"].as_str().map(Self::of)
    }
    /// bukkit: org/bukkit/inventory/ItemType.getItemRarity
    ///
    /// The name of `org.bukkit.inventory.ItemRarity`.
    pub fn get_item_rarity(&self) -> String {
        text(&self.default_component("minecraft:rarity")).to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/inventory/ItemType.getTranslationKey
    pub fn get_translation_key(&self) -> String {
        format!("item.{}", self.0.replace(':', "."))
    }
    /// bukkit: org/bukkit/inventory/ItemType.hasDefaultData
    pub fn has_default_data(&self, component: &str) -> bool {
        !self.default_component(component).is_null()
    }
    /// bukkit: org/bukkit/inventory/ItemType.getDefaultData
    ///
    /// By the component's id, as its JSON: `io.papermc.paper.datacomponent.item` wraps it.
    pub fn get_default_data(&self, component: &str) -> Option<Value> {
        Some(self.default_component(component)).filter(|value| !value.is_null())
    }
    /// bukkit: org/bukkit/inventory/ItemType.getDefaultDataTypes
    pub fn get_default_data_types(&self) -> Vec<String> {
        self.info()["components"].as_object().into_iter().flatten().map(|(id, _)| id.clone()).collect()
    }
    /// bukkit: org/bukkit/inventory/ItemType.getDefaultAttributeModifiers
    ///
    /// Attribute keys with their modifiers, only those of `slot` when it's given: `mainhand`, `chest`...
    pub fn get_default_attribute_modifiers(&self, slot: Option<&str>) -> Vec<(String, crate::scoreboard::AttributeModifier)> {
        modifiers_of(&self.default_component("minecraft:attribute_modifiers"), slot)
    }
    /// bukkit: org/bukkit/inventory/ItemType.isEnabledByFeature
    ///
    /// No item of this game needs a feature flag.
    pub fn is_enabled_by_feature(&self, _world: &World) -> bool {
        !self.info().is_null()
    }
}

fn modifiers_of(list: &Value, slot: Option<&str>) -> Vec<(String, crate::scoreboard::AttributeModifier)> {
    let fits = |row: &Value| slot.is_none_or(|slot| row["slot"].as_str().unwrap_or("any") == slot);
    list.as_array().into_iter().flatten().filter(|row| fits(row)).filter_map(|row| Some((row["type"].as_str()?.to_owned(), crate::scoreboard::AttributeModifier::new(row["id"].as_str()?, row["amount"].as_f64()?, row["operation"].as_str()?)))).collect()
}

// ---- BlockData classes over one property each ----

/// The lowest a number property of this block goes.
fn minimum_of(data: &BlockData, key: &str) -> i32 {
    let block = data.get_material();
    query(json!({ "op": "property_values", "block": block.key(), "key": key })).as_array().into_iter().flatten().filter_map(|value| value.as_str()?.parse().ok()).min().unwrap_or(0)
}

/// The values one property of this block takes, in upper case.
fn allowed_values(data: &BlockData, key: &str) -> Vec<String> {
    let block = data.get_material();
    query(json!({ "op": "property_values", "block": block.key(), "key": key })).as_array().into_iter().flatten().filter_map(|value| value.as_str().map(str::to_ascii_uppercase)).collect()
}

/// `org.bukkit.block.data.type.RespawnAnchor`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RespawnAnchorData(pub BlockData);

impl DataClass for RespawnAnchorData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl RespawnAnchorData {
    /// bukkit: org/bukkit/block/data/type/RespawnAnchor.getCharges
    pub fn get_charges(&self) -> i32 {
        self.0.property("charges").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/RespawnAnchor.setCharges
    pub fn set_charges(&mut self, value: i32) {
        self.0.set_property("charges", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/RespawnAnchor.getMaximumCharges
    pub fn get_maximum_charges(&self) -> i32 {
        self.0.maximum_of("charges")
    }
}

/// `org.bukkit.block.data.type.SeaPickle`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeaPickleData(pub BlockData);

impl DataClass for SeaPickleData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SeaPickleData {
    /// bukkit: org/bukkit/block/data/type/SeaPickle.getPickles
    pub fn get_pickles(&self) -> i32 {
        self.0.property("pickles").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/SeaPickle.setPickles
    pub fn set_pickles(&mut self, value: i32) {
        self.0.set_property("pickles", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/SeaPickle.getMaximumPickles
    pub fn get_maximum_pickles(&self) -> i32 {
        self.0.maximum_of("pickles")
    }
    /// bukkit: org/bukkit/block/data/type/SeaPickle.getMinimumPickles
    pub fn get_minimum_pickles(&self) -> i32 {
        minimum_of(&self.0, "pickles")
    }
}

/// `org.bukkit.block.data.type.Stairs`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StairsData(pub BlockData);

impl DataClass for StairsData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl StairsData {
    /// bukkit: org/bukkit/block/data/type/Stairs.getShape
    pub fn get_shape(&self) -> String {
        self.0.property("shape").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Stairs.setShape
    pub fn set_shape(&mut self, value: &str) {
        self.0.set_property("shape", &value.to_ascii_lowercase());
    }
}

/// `org.bukkit.block.data.type.Slab`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlabData(pub BlockData);

impl DataClass for SlabData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SlabData {
    /// bukkit: org/bukkit/block/data/type/Slab.getType
    pub fn get_type(&self) -> String {
        self.0.property("type").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Slab.setType
    pub fn set_type(&mut self, value: &str) {
        self.0.set_property("type", &value.to_ascii_lowercase());
    }
}

/// `org.bukkit.block.data.type.Snow`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnowData(pub BlockData);

impl DataClass for SnowData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SnowData {
    /// bukkit: org/bukkit/block/data/type/Snow.getLayers
    pub fn get_layers(&self) -> i32 {
        self.0.property("layers").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Snow.setLayers
    pub fn set_layers(&mut self, value: i32) {
        self.0.set_property("layers", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Snow.getMaximumLayers
    pub fn get_maximum_layers(&self) -> i32 {
        self.0.maximum_of("layers")
    }
    /// bukkit: org/bukkit/block/data/type/Snow.getMinimumLayers
    pub fn get_minimum_layers(&self) -> i32 {
        minimum_of(&self.0, "layers")
    }
}

/// `org.bukkit.block.data.type.Repeater`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeaterData(pub BlockData);

impl DataClass for RepeaterData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl RepeaterData {
    /// bukkit: org/bukkit/block/data/type/Repeater.getDelay
    pub fn get_delay(&self) -> i32 {
        self.0.property("delay").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Repeater.setDelay
    pub fn set_delay(&mut self, value: i32) {
        self.0.set_property("delay", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Repeater.getMaximumDelay
    pub fn get_maximum_delay(&self) -> i32 {
        self.0.maximum_of("delay")
    }
    /// bukkit: org/bukkit/block/data/type/Repeater.getMinimumDelay
    pub fn get_minimum_delay(&self) -> i32 {
        minimum_of(&self.0, "delay")
    }
    /// bukkit: org/bukkit/block/data/type/Repeater.isLocked
    pub fn is_locked(&self) -> bool {
        self.0.property("locked") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Repeater.setLocked
    pub fn set_locked(&mut self, value: bool) {
        self.0.set_property("locked", if value { "true" } else { "false" });
    }
}

/// `org.bukkit.block.data.type.Scaffolding`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaffoldingData(pub BlockData);

impl DataClass for ScaffoldingData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl ScaffoldingData {
    /// bukkit: org/bukkit/block/data/type/Scaffolding.getDistance
    pub fn get_distance(&self) -> i32 {
        self.0.property("distance").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Scaffolding.setDistance
    pub fn set_distance(&mut self, value: i32) {
        self.0.set_property("distance", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Scaffolding.getMaximumDistance
    pub fn get_maximum_distance(&self) -> i32 {
        self.0.maximum_of("distance")
    }
    /// bukkit: org/bukkit/block/data/type/Scaffolding.isBottom
    pub fn is_bottom(&self) -> bool {
        self.0.property("bottom") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Scaffolding.setBottom
    pub fn set_bottom(&mut self, value: bool) {
        self.0.set_property("bottom", if value { "true" } else { "false" });
    }
}

/// `org.bukkit.block.data.type.SculkShrieker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SculkShriekerData(pub BlockData);

impl DataClass for SculkShriekerData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SculkShriekerData {
    /// bukkit: org/bukkit/block/data/type/SculkShrieker.isShrieking
    pub fn is_shrieking(&self) -> bool {
        self.0.property("shrieking") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/SculkShrieker.setShrieking
    pub fn set_shrieking(&mut self, value: bool) {
        self.0.set_property("shrieking", if value { "true" } else { "false" });
    }
    /// bukkit: org/bukkit/block/data/type/SculkShrieker.isCanSummon
    pub fn is_can_summon(&self) -> bool {
        self.0.property("can_summon") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/SculkShrieker.setCanSummon
    pub fn set_can_summon(&mut self, value: bool) {
        self.0.set_property("can_summon", if value { "true" } else { "false" });
    }
}

/// `org.bukkit.block.data.type.TechnicalPiston`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TechnicalPistonData(pub BlockData);

impl DataClass for TechnicalPistonData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl TechnicalPistonData {
    /// bukkit: org/bukkit/block/data/type/TechnicalPiston.getType
    pub fn get_type(&self) -> String {
        self.0.property("type").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/TechnicalPiston.setType
    pub fn set_type(&mut self, value: &str) {
        self.0.set_property("type", &value.to_ascii_lowercase());
    }
}

/// `org.bukkit.block.data.type.TrialSpawner`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrialSpawnerData(pub BlockData);

impl DataClass for TrialSpawnerData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl TrialSpawnerData {
    /// bukkit: org/bukkit/block/data/type/TrialSpawner.getTrialSpawnerState
    pub fn get_trial_spawner_state(&self) -> String {
        self.0.property("trial_spawner_state").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/TrialSpawner.setTrialSpawnerState
    pub fn set_trial_spawner_state(&mut self, value: &str) {
        self.0.set_property("trial_spawner_state", &value.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/block/data/type/TrialSpawner.isOminous
    pub fn is_ominous(&self) -> bool {
        self.0.property("ominous") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/TrialSpawner.setOminous
    pub fn set_ominous(&mut self, value: bool) {
        self.0.set_property("ominous", if value { "true" } else { "false" });
    }
}

/// `org.bukkit.block.data.type.Vault`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VaultData(pub BlockData);

impl DataClass for VaultData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl VaultData {
    /// bukkit: org/bukkit/block/data/type/Vault.getVaultState, org/bukkit/block/data/type/Vault.getTrialSpawnerState
    pub fn get_vault_state(&self) -> String {
        self.0.property("vault_state").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Vault.setVaultState, org/bukkit/block/data/type/Vault.setTrialSpawnerState
    pub fn set_vault_state(&mut self, value: &str) {
        self.0.set_property("vault_state", &value.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/block/data/type/Vault.isOminous
    pub fn is_ominous(&self) -> bool {
        self.0.property("ominous") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Vault.setOminous
    pub fn set_ominous(&mut self, value: bool) {
        self.0.set_property("ominous", if value { "true" } else { "false" });
    }
}

/// `org.bukkit.block.data.type.SculkCatalyst`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SculkCatalystData(pub BlockData);

impl DataClass for SculkCatalystData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SculkCatalystData {
    /// bukkit: org/bukkit/block/data/type/SculkCatalyst.isBloom
    pub fn is_bloom(&self) -> bool {
        self.0.property("bloom") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/SculkCatalyst.setBloom
    pub fn set_bloom(&mut self, value: bool) {
        self.0.set_property("bloom", if value { "true" } else { "false" });
    }
}

/// `org.bukkit.block.data.type.SculkSensor`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SculkSensorData(pub BlockData);

impl DataClass for SculkSensorData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SculkSensorData {
    /// bukkit: org/bukkit/block/data/type/SculkSensor.getSculkSensorPhase, org/bukkit/block/data/type/SculkSensor.getPhase
    pub fn get_sculk_sensor_phase(&self) -> String {
        self.0.property("sculk_sensor_phase").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/SculkSensor.setSculkSensorPhase, org/bukkit/block/data/type/SculkSensor.setPhase
    pub fn set_sculk_sensor_phase(&mut self, value: &str) {
        self.0.set_property("sculk_sensor_phase", &value.to_ascii_lowercase());
    }
}

/// `org.bukkit.block.data.type.Speleothem`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeleothemData(pub BlockData);

impl DataClass for SpeleothemData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SpeleothemData {
    /// bukkit: org/bukkit/block/data/type/Speleothem.getThickness
    pub fn get_thickness(&self) -> String {
        self.0.property("thickness").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Speleothem.setThickness
    pub fn set_thickness(&mut self, value: &str) {
        self.0.set_property("thickness", &value.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/block/data/type/Speleothem.getVerticalDirection
    pub fn get_vertical_direction(&self) -> String {
        self.0.property("vertical_direction").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Speleothem.setVerticalDirection
    pub fn set_vertical_direction(&mut self, value: &str) {
        self.0.set_property("vertical_direction", &value.to_ascii_lowercase());
    }
}

/// `org.bukkit.block.data.type.StructureBlock`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureBlockData(pub BlockData);

impl DataClass for StructureBlockData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl StructureBlockData {
    /// bukkit: org/bukkit/block/data/type/StructureBlock.getMode
    pub fn get_mode(&self) -> String {
        self.0.property("mode").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/StructureBlock.setMode
    pub fn set_mode(&mut self, value: &str) {
        self.0.set_property("mode", &value.to_ascii_lowercase());
    }
}

/// `org.bukkit.block.data.type.Tripwire`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TripwireData(pub BlockData);

impl DataClass for TripwireData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl TripwireData {
    /// bukkit: org/bukkit/block/data/type/Tripwire.isDisarmed
    pub fn is_disarmed(&self) -> bool {
        self.0.property("disarmed") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Tripwire.setDisarmed
    pub fn set_disarmed(&mut self, value: bool) {
        self.0.set_property("disarmed", if value { "true" } else { "false" });
    }
}

/// `org.bukkit.block.data.type.TurtleEgg`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurtleEggData(pub BlockData);

impl DataClass for TurtleEggData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl TurtleEggData {
    /// bukkit: org/bukkit/block/data/type/TurtleEgg.getEggs
    pub fn get_eggs(&self) -> i32 {
        self.0.property("eggs").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/TurtleEgg.setEggs
    pub fn set_eggs(&mut self, value: i32) {
        self.0.set_property("eggs", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/TurtleEgg.getMaximumEggs
    pub fn get_maximum_eggs(&self) -> i32 {
        self.0.maximum_of("eggs")
    }
    /// bukkit: org/bukkit/block/data/type/TurtleEgg.getMinimumEggs
    pub fn get_minimum_eggs(&self) -> i32 {
        minimum_of(&self.0, "eggs")
    }
}

/// `org.bukkit.block.data.type.Sapling`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaplingData(pub BlockData);

impl DataClass for SaplingData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SaplingData {
    /// bukkit: org/bukkit/block/data/type/Sapling.getStage
    pub fn get_stage(&self) -> i32 {
        self.0.property("stage").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Sapling.setStage
    pub fn set_stage(&mut self, value: i32) {
        self.0.set_property("stage", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Sapling.getMaximumStage
    pub fn get_maximum_stage(&self) -> i32 {
        self.0.maximum_of("stage")
    }
}

/// `org.bukkit.block.data.type.TNT`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TNTData(pub BlockData);

impl DataClass for TNTData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl TNTData {
    /// bukkit: org/bukkit/block/data/type/TNT.isUnstable
    pub fn is_unstable(&self) -> bool {
        self.0.property("unstable") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/TNT.setUnstable
    pub fn set_unstable(&mut self, value: bool) {
        self.0.set_property("unstable", if value { "true" } else { "false" });
    }
}

/// `org.bukkit.block.data.type.TestBlock`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestBlockData(pub BlockData);

impl DataClass for TestBlockData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl TestBlockData {
    /// bukkit: org/bukkit/block/data/type/TestBlock.getMode
    pub fn get_mode(&self) -> String {
        self.0.property("mode").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/TestBlock.setMode
    pub fn set_mode(&mut self, value: &str) {
        self.0.set_property("mode", &value.to_ascii_lowercase());
    }
}

/// `org.bukkit.block.data.type.Wall`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WallData(pub BlockData);

impl DataClass for WallData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl WallData {
    /// bukkit: org/bukkit/block/data/type/Wall.isUp
    pub fn is_up(&self) -> bool {
        self.0.property("up") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Wall.setUp
    pub fn set_up(&mut self, value: bool) {
        self.0.set_property("up", if value { "true" } else { "false" });
    }
}

const FACES: [&str; 6] = ["NORTH", "EAST", "SOUTH", "WEST", "UP", "DOWN"];

fn truthy(data: &BlockData, key: &str) -> bool {
    data.property(key) == Some("true")
}

fn set_flag(data: &mut BlockData, key: &str, value: bool) {
    data.set_property(key, if value { "true" } else { "false" });
}

/// `org.bukkit.block.data.MultipleFacing`: fences, panes, vines and the like.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultipleFacingData(pub BlockData);

impl DataClass for MultipleFacingData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl MultipleFacingData {
    /// bukkit: org/bukkit/block/data/MultipleFacing.hasFace
    pub fn has_face(&self, face: &str) -> bool {
        truthy(&self.0, &face.to_ascii_lowercase())
    }
    /// bukkit: org/bukkit/block/data/MultipleFacing.setFace
    pub fn set_face(&mut self, face: &str, value: bool) {
        set_flag(&mut self.0, &face.to_ascii_lowercase(), value);
    }
    /// bukkit: org/bukkit/block/data/MultipleFacing.getFaces
    pub fn get_faces(&self) -> Vec<&'static str> {
        FACES.into_iter().filter(|face| self.has_face(face)).collect()
    }
    /// bukkit: org/bukkit/block/data/MultipleFacing.getAllowedFaces
    pub fn get_allowed_faces(&self) -> Vec<&'static str> {
        FACES.into_iter().filter(|face| self.0.property(&face.to_ascii_lowercase()).is_some()).collect()
    }
}

/// `org.bukkit.block.data.type.ChiseledBookshelf`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChiseledBookshelfData(pub BlockData);

impl DataClass for ChiseledBookshelfData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl ChiseledBookshelfData {
    /// bukkit: org/bukkit/block/data/type/ChiseledBookshelf.isSlotOccupied
    pub fn is_slot_occupied(&self, slot: usize) -> bool {
        truthy(&self.0, &format!("slot_{slot}_occupied"))
    }
    /// bukkit: org/bukkit/block/data/type/ChiseledBookshelf.setSlotOccupied
    pub fn set_slot_occupied(&mut self, slot: usize, value: bool) {
        set_flag(&mut self.0, &format!("slot_{slot}_occupied"), value);
    }
    /// bukkit: org/bukkit/block/data/type/ChiseledBookshelf.getOccupiedSlots
    pub fn get_occupied_slots(&self) -> Vec<usize> {
        (0..6).filter(|slot| self.is_slot_occupied(*slot)).collect()
    }
    /// bukkit: org/bukkit/block/data/type/ChiseledBookshelf.getMaximumOccupiedSlots
    pub fn get_maximum_occupied_slots(&self) -> usize {
        6
    }
}

/// `org.bukkit.block.data.type.BrewingStand`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrewingStandData(pub BlockData);

impl DataClass for BrewingStandData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BrewingStandData {
    /// bukkit: org/bukkit/block/data/type/BrewingStand.hasBottle
    pub fn has_bottle(&self, bottle: usize) -> bool {
        truthy(&self.0, &format!("has_bottle_{bottle}"))
    }
    /// bukkit: org/bukkit/block/data/type/BrewingStand.setBottle
    pub fn set_bottle(&mut self, bottle: usize, value: bool) {
        set_flag(&mut self.0, &format!("has_bottle_{bottle}"), value);
    }
    /// bukkit: org/bukkit/block/data/type/BrewingStand.getBottles
    pub fn get_bottles(&self) -> Vec<usize> {
        (0..3).filter(|bottle| self.has_bottle(*bottle)).collect()
    }
    /// bukkit: org/bukkit/block/data/type/BrewingStand.getMaximumBottles
    pub fn get_maximum_bottles(&self) -> usize {
        3
    }
}

/// `org.bukkit.block.data.type.RedstoneWire`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedstoneWireData(pub BlockData);

impl DataClass for RedstoneWireData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl RedstoneWireData {
    /// bukkit: org/bukkit/block/data/type/RedstoneWire.getFace
    pub fn get_face(&self, face: &str) -> String {
        self.0.property(&face.to_ascii_lowercase()).unwrap_or("none").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/RedstoneWire.setFace
    pub fn set_face(&mut self, face: &str, connection: &str) {
        self.0.set_property(&face.to_ascii_lowercase(), &connection.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/block/data/type/RedstoneWire.getAllowedFaces
    pub fn get_allowed_faces(&self) -> Vec<&'static str> {
        ["NORTH", "EAST", "SOUTH", "WEST"].to_vec()
    }
}

impl WallData {
    /// bukkit: org/bukkit/block/data/type/Wall.getHeight
    pub fn get_height(&self, face: &str) -> String {
        self.0.property(&face.to_ascii_lowercase()).unwrap_or("none").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Wall.setHeight
    pub fn set_height(&mut self, face: &str, height: &str) {
        self.0.set_property(&face.to_ascii_lowercase(), &height.to_ascii_lowercase());
    }
}

/// `org.bukkit.block.data.type.Switch`: buttons and levers, with `AttachableDirectional`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchData(pub BlockData);

impl DataClass for SwitchData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SwitchData {
    /// bukkit: org/bukkit/block/data/type/Switch.getFace, org/bukkit/block/data/type/Switch.getAttachedFace
    pub fn get_face(&self) -> String {
        self.0.property("face").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Switch.setFace, org/bukkit/block/data/type/Switch.setAttachedFace
    pub fn set_face(&mut self, face: &str) {
        self.0.set_property("face", &face.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/block/data/AttachableDirectional.getAttachedBlockFace
    pub fn get_attached_block_face(&self) -> &'static str {
        match self.get_face().as_str() {
            "FLOOR" => "DOWN",
            "CEILING" => "UP",
            _ => crate::block_data::block_face_opposite(&self.0.property("facing").unwrap_or_default().to_ascii_uppercase()),
        }
    }
}

impl blocks::DirectionalData {
    /// bukkit: org/bukkit/block/data/Directional.getFaces
    pub fn get_faces(&self) -> Vec<String> {
        allowed_values(&self.0, "facing")
    }
}

impl blocks::LevelledData {
    /// bukkit: org/bukkit/block/data/Levelled.getMinimumLevel
    pub fn get_minimum_level(&self) -> i32 {
        minimum_of(&self.0, "level")
    }
}

impl blocks::OrientableData {
    /// bukkit: org/bukkit/block/data/Orientable.getAxes
    pub fn get_axes(&self) -> Vec<String> {
        allowed_values(&self.0, "axis")
    }
}

impl blocks::RailData {
    /// bukkit: org/bukkit/block/data/Rail.getShapes
    pub fn get_shapes(&self) -> Vec<String> {
        allowed_values(&self.0, "shape")
    }
}

impl blocks::SegmentableData {
    /// bukkit: org/bukkit/block/data/Segmentable.getMinimumSegmentAmount
    pub fn get_minimum_segment_amount(&self) -> i32 {
        minimum_of(&self.0, "segment_amount")
    }
}

impl blocks::CandleData {
    /// bukkit: org/bukkit/block/data/type/Candle.getMinimumCandles
    pub fn get_minimum_candles(&self) -> i32 {
        minimum_of(&self.0, "candles")
    }
}

impl blocks::FlowerBedData {
    /// bukkit: org/bukkit/block/data/type/FlowerBed.getMinimumFlowerAmount
    pub fn get_minimum_flower_amount(&self) -> i32 {
        minimum_of(&self.0, "flower_amount")
    }
}

impl blocks::LeavesData {
    /// bukkit: org/bukkit/block/data/type/Leaves.getMinimumDistance
    pub fn get_minimum_distance(&self) -> i32 {
        minimum_of(&self.0, "distance")
    }
}

impl blocks::CreakingHeartData {
    /// bukkit: org/bukkit/block/data/type/CreakingHeart.isActive
    pub fn is_active(&self) -> bool {
        self.0.property("creaking_heart_state") == Some("awake")
    }
    /// bukkit: org/bukkit/block/data/type/CreakingHeart.setActive
    pub fn set_active(&mut self, active: bool) {
        self.0.set_property("creaking_heart_state", if active { "awake" } else { "uprooted" });
    }
}

impl SpeleothemData {
    /// bukkit: org/bukkit/block/data/type/Speleothem.getVerticalDirections
    pub fn get_vertical_directions(&self) -> Vec<String> {
        allowed_values(&self.0, "vertical_direction")
    }
}

impl blocks::CaveVinesPlantData {
    /// bukkit: org/bukkit/block/data/type/CaveVinesPlant.hasBerries
    pub fn has_berries(&self) -> bool {
        truthy(&self.0, "berries")
    }
}

impl blocks::MossyCarpetData {
    /// bukkit: org/bukkit/block/data/type/MossyCarpet.getHeight
    pub fn get_height(&self, face: &str) -> String {
        self.0.property(&face.to_ascii_lowercase()).unwrap_or("none").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/MossyCarpet.setHeight
    pub fn set_height(&mut self, face: &str, height: &str) {
        self.0.set_property(&face.to_ascii_lowercase(), &height.to_ascii_lowercase());
    }
}

impl blocks::RotatableData {
    /// bukkit: org/bukkit/block/data/Rotatable.getRotation
    ///
    /// The sixteen `rotation` steps as Bukkit's faces, clockwise from south.
    pub fn get_rotation(&self) -> &'static str {
        ROTATIONS[self.get_rotation_number().rem_euclid(16) as usize]
    }
    /// bukkit: org/bukkit/block/data/Rotatable.setRotation
    pub fn set_rotation(&mut self, face: &str) {
        if let Some(step) = ROTATIONS.iter().position(|known| *known == face) {
            self.set_rotation_number(step as i32);
        }
    }
}

const ROTATIONS: [&str; 16] = [
    "SOUTH", "SOUTH_SOUTH_WEST", "SOUTH_WEST", "WEST_SOUTH_WEST", "WEST", "WEST_NORTH_WEST", "NORTH_WEST", "NORTH_NORTH_WEST",
    "NORTH", "NORTH_NORTH_EAST", "NORTH_EAST", "EAST_NORTH_EAST", "EAST", "EAST_SOUTH_EAST", "SOUTH_EAST", "SOUTH_SOUTH_EAST",
];

impl blocks::NoteBlockData {
    /// bukkit: org/bukkit/block/data/type/NoteBlock.getInstrument
    pub fn get_instrument(&self) -> String {
        instrument_name(self.0.property("instrument").unwrap_or("harp")).to_owned()
    }
    /// bukkit: org/bukkit/block/data/type/NoteBlock.setInstrument
    ///
    /// By Bukkit's name.
    pub fn set_instrument(&mut self, instrument: &str) {
        if let Some(id) = horizon_instruments().iter().find(|id| instrument_name(id) == instrument) {
            self.0.set_property("instrument", id);
        }
    }
    /// bukkit: org/bukkit/block/data/type/NoteBlock.getNote
    pub fn get_note(&self) -> crate::core2::Note {
        crate::core2::Note(self.get_note_number() as u8)
    }
    /// bukkit: org/bukkit/block/data/type/NoteBlock.setNote
    pub fn set_note(&mut self, note: &crate::core2::Note) {
        self.set_note_number(i32::from(note.0));
    }
}

// ---- ItemMeta: every call here is on the stack, whose components are the meta ----

/// A holder set as the game saves it: an id, a `#tag` or a list of ids.
fn holder_list(value: &Value) -> Vec<String> {
    match value {
        Value::String(id) => vec![id.clone()],
        Value::Array(ids) => ids.iter().filter_map(|id| id.as_str().map(str::to_owned)).collect(),
        _ => Vec::new(),
    }
}

/// A holder set's ids, with each tag opened: `kind` is the tag folder.
fn holder_members(value: &Value, kind: &'static str) -> Vec<String> {
    holder_list(value).into_iter().flat_map(|id| if id.starts_with('#') { crate::registry::Tag::get(kind, &id).get_values() } else { vec![id] }).collect()
}

/// The game's name of an attribute modifier's operation, from Bukkit's or its own.
fn operation_name(operation: &str) -> &'static str {
    match operation {
        "ADD_SCALAR" | "add_multiplied_base" => "add_multiplied_base",
        "MULTIPLY_SCALAR_1" | "add_multiplied_total" => "add_multiplied_total",
        _ => "add_value",
    }
}

fn predicate_blocks(stack: &ItemStack, id: &str) -> Vec<String> {
    match stack.get_component(id) {
        Some(Value::Array(rows)) => rows.iter().flat_map(|row| holder_list(&row["blocks"])).collect(),
        Some(row) => holder_list(&row["blocks"]),
        None => Vec::new(),
    }
}

fn set_predicate_blocks(stack: &mut ItemStack, id: &str, blocks: &[String]) {
    if blocks.is_empty() {
        stack.remove_component(id);
    } else {
        stack.set_component(id, json!([{ "blocks": blocks }]));
    }
}

impl ItemStack {
    fn take_component(&mut self, id: &str, from: Option<&Self>) {
        match from.and_then(|from| from.get_component(id)) {
            Some(value) => self.set_component(id, value.clone()),
            None => {
                self.remove_component(id);
            }
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setAttributeModifiers
    ///
    /// Attribute, modifier and slot group for each; `None` takes them all off.
    pub fn set_attribute_modifiers(&mut self, modifiers: Option<&[(String, crate::scoreboard::AttributeModifier, String)]>) {
        match modifiers {
            Some(rows) => {
                let saved: Vec<Value> = rows.iter().map(|(attribute, modifier, slot)| json!({ "type": attribute, "id": modifier.get_key(), "amount": modifier.get_amount(), "operation": operation_name(modifier.get_operation()), "slot": slot })).collect();
                self.set_component("minecraft:attribute_modifiers", Value::Array(saved));
            }
            None => {
                self.remove_component("minecraft:attribute_modifiers");
            }
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasFood
    pub fn has_food(&self) -> bool {
        self.get_component("minecraft:food").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getFood
    pub fn get_food(&self) -> crate::meta::FoodComponent {
        self.meta()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setFood
    pub fn set_food(&mut self, food: Option<&crate::meta::FoodComponent>) {
        self.take_component("minecraft:food", food.map(|food| &food.0));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasEquippable
    pub fn has_equippable(&self) -> bool {
        self.get_component("minecraft:equippable").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getEquippable
    pub fn get_equippable(&self) -> crate::meta::EquippableComponent {
        self.meta()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setEquippable
    pub fn set_equippable(&mut self, equippable: Option<&crate::meta::EquippableComponent>) {
        self.take_component("minecraft:equippable", equippable.map(|equippable| &equippable.0));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasUseCooldown
    pub fn has_use_cooldown(&self) -> bool {
        self.get_component("minecraft:use_cooldown").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getUseCooldown
    pub fn get_use_cooldown(&self) -> crate::meta::UseCooldownComponent {
        self.meta()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setUseCooldown
    pub fn set_use_cooldown(&mut self, cooldown: Option<&crate::meta::UseCooldownComponent>) {
        self.take_component("minecraft:use_cooldown", cooldown.map(|cooldown| &cooldown.0));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasTool
    pub fn has_tool(&self) -> bool {
        self.get_component("minecraft:tool").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getTool
    pub fn get_tool(&self) -> crate::meta::ToolComponent {
        self.meta()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setTool
    pub fn set_tool(&mut self, tool: Option<&crate::meta::ToolComponent>) {
        self.take_component("minecraft:tool", tool.map(|tool| &tool.0));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasJukeboxPlayable
    pub fn has_jukebox_playable(&self) -> bool {
        self.get_component("minecraft:jukebox_playable").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getJukeboxPlayable
    pub fn get_jukebox_playable(&self) -> crate::meta::JukeboxPlayableComponent {
        self.meta()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setJukeboxPlayable
    pub fn set_jukebox_playable(&mut self, playable: Option<&crate::meta::JukeboxPlayableComponent>) {
        self.take_component("minecraft:jukebox_playable", playable.map(|playable| &playable.0));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasDamageResistant
    pub fn has_damage_resistant(&self) -> bool {
        self.get_component("minecraft:damage_resistant").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getDamageResistant, org/bukkit/inventory/meta/ItemMeta.getDamageResistantTypes
    ///
    /// The damage type tag, `#minecraft:is_fire`, or the types by id.
    pub fn get_damage_resistant(&self) -> Option<Value> {
        self.component_at("minecraft:damage_resistant", "types").cloned()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setDamageResistant, org/bukkit/inventory/meta/ItemMeta.setDamageResistantTypes
    ///
    /// A tag as `#minecraft:is_fire`, or a list of ids. `None` takes the resistance off.
    pub fn set_damage_resistant(&mut self, types: Option<Value>) {
        match types {
            Some(types) => self.set_component("minecraft:damage_resistant", json!({ "types": types })),
            None => {
                self.remove_component("minecraft:damage_resistant");
            }
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getMaxStackSize
    ///
    /// What this stack was set to: `None` where Java throws.
    pub fn get_meta_max_stack_size(&self) -> Option<i32> {
        self.get_max_stack_size_override()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getAsComponentString
    pub fn get_as_component_string(&self) -> String {
        let parts: Vec<String> = self.all_components().map(|(id, value)| format!("{id}={}", crate::data::snbt(value))).collect();
        format!("[{}]", parts.join(","))
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getAsString
    pub fn get_as_string(&self) -> String {
        let parts: Vec<String> = self.all_components().map(|(id, value)| format!("{}:{}", crate::data::quoted(id), crate::data::snbt(value))).collect();
        format!("{{{}}}", parts.join(","))
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasUseRemainder
    pub fn has_use_remainder(&self) -> bool {
        self.get_component("minecraft:use_remainder").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getUseRemainder
    pub fn get_use_remainder(&self) -> Option<Self> {
        self.get_component("minecraft:use_remainder").and_then(Self::read)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setUseRemainder
    pub fn set_use_remainder(&mut self, remainder: Option<&Self>) {
        match remainder {
            Some(stack) => {
                let components: Map<String, Value> = stack.all_components().map(|(id, value)| (id.clone(), value.clone())).collect();
                self.set_component("minecraft:use_remainder", json!({ "id": stack.get_type().key(), "count": stack.get_amount(), "components": components }));
            }
            None => {
                self.remove_component("minecraft:use_remainder");
            }
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getLoreComponents
    ///
    /// Bungee's `BaseComponent[]` is one `Component` here.
    pub fn get_lore_components(&self) -> Vec<crate::Component> {
        self.lore()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setLoreComponents
    pub fn set_lore_components(&mut self, lore: &[crate::Component]) {
        if lore.is_empty() {
            self.remove_component("minecraft:lore");
        } else {
            self.set_component("minecraft:lore", Value::Array(lore.iter().map(|line| line.0.clone()).collect()));
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setDisplayNameComponent
    pub fn set_display_name_components(&mut self, name: &crate::Component) {
        self.set_display_name_component(name);
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getDisplayNameComponent
    pub fn get_display_name_component(&self) -> Option<crate::Component> {
        self.display_name()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasLocalizedName
    ///
    /// A display name that's a translation key, as the method's successor reads it.
    pub fn has_localized_name(&self) -> bool {
        self.component_at("minecraft:custom_name", "translate").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getLocalizedName
    pub fn get_localized_name(&self) -> Option<String> {
        self.component_at("minecraft:custom_name", "translate").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setLocalizedName
    pub fn set_localized_name(&mut self, key: Option<&str>) {
        match key {
            Some(key) => self.set_component("minecraft:custom_name", json!({ "translate": key })),
            None => {
                self.remove_component("minecraft:custom_name");
            }
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasPlaceableKeys
    pub fn has_placeable_keys(&self) -> bool {
        !predicate_blocks(self, "minecraft:can_place_on").is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getPlaceableKeys
    ///
    /// Block ids and `#tags`.
    pub fn get_placeable_keys(&self) -> Vec<String> {
        predicate_blocks(self, "minecraft:can_place_on")
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setPlaceableKeys
    pub fn set_placeable_keys(&mut self, keys: &[String]) {
        set_predicate_blocks(self, "minecraft:can_place_on", keys);
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasDestroyableKeys
    pub fn has_destroyable_keys(&self) -> bool {
        !predicate_blocks(self, "minecraft:can_break").is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getDestroyableKeys
    pub fn get_destroyable_keys(&self) -> Vec<String> {
        predicate_blocks(self, "minecraft:can_break")
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setDestroyableKeys
    pub fn set_destroyable_keys(&mut self, keys: &[String]) {
        set_predicate_blocks(self, "minecraft:can_break", keys);
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getCanPlaceOn
    pub fn get_can_place_on(&self) -> Vec<Material> {
        holder_members(&json!(self.get_placeable_keys()), "block").iter().map(|id| Material::of(id)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setCanPlaceOn
    pub fn set_can_place_on(&mut self, materials: &[Material]) {
        self.set_placeable_keys(&materials.iter().map(|material| material.key().to_owned()).collect::<Vec<_>>());
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getCanDestroy
    pub fn get_can_destroy(&self) -> Vec<Material> {
        holder_members(&json!(self.get_destroyable_keys()), "block").iter().map(|id| Material::of(id)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setCanDestroy
    pub fn set_can_destroy(&mut self, materials: &[Material]) {
        self.set_destroyable_keys(&materials.iter().map(|material| material.key().to_owned()).collect::<Vec<_>>());
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasConflictingEnchant
    pub fn has_conflicting_enchant(&self, enchantment: &crate::registry::Enchantment) -> bool {
        self.get_enchants().keys().filter_map(|key| crate::registry::Enchantment::get_by_key(key)).any(|other| other.get_key() != enchantment.get_key() && other.conflicts_with(enchantment))
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getCustomTagContainer
    pub fn get_custom_tag_container(&mut self) -> CustomItemTagContainer<'_> {
        CustomItemTagContainer(self)
    }
}

/// `org.bukkit.inventory.meta.tags.CustomItemTagContainer`: the old face of a stack's persistent data.
/// The value's Rust type stands for `ItemTagType`.
pub struct CustomItemTagContainer<'a>(&'a mut ItemStack);

impl CustomItemTagContainer<'_> {
    /// bukkit: org/bukkit/inventory/meta/tags/CustomItemTagContainer.setCustomTag
    pub fn set_custom_tag(&mut self, key: &NamespacedKey, value: Value) {
        self.0.pdc_set(key, value);
    }
    /// bukkit: org/bukkit/inventory/meta/tags/CustomItemTagContainer.hasCustomTag
    pub fn has_custom_tag(&self, key: &NamespacedKey) -> bool {
        self.0.pdc_has(key)
    }
    /// bukkit: org/bukkit/inventory/meta/tags/CustomItemTagContainer.getCustomTag
    pub fn get_custom_tag(&self, key: &NamespacedKey) -> Option<Value> {
        self.0.pdc_get(key)
    }
    /// bukkit: org/bukkit/inventory/meta/tags/CustomItemTagContainer.removeCustomTag
    pub fn remove_custom_tag(&mut self, key: &NamespacedKey) {
        self.0.pdc_remove(key);
    }
    /// bukkit: org/bukkit/inventory/meta/tags/CustomItemTagContainer.isEmpty
    pub fn is_empty(&self) -> bool {
        self.0.pdc_is_empty()
    }
}

/// `org.bukkit.inventory.meta.components.ToolComponent$ToolRule`, as the component saves it.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolRule(pub Value);

impl ToolRule {
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent$ToolRule.getBlocks
    pub fn get_blocks(&self) -> Vec<Material> {
        holder_members(&self.0["blocks"], "block").iter().map(|id| Material::of(id)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent$ToolRule.setBlocks
    ///
    /// Ids, or one `#tag`.
    pub fn set_blocks(&mut self, blocks: &[&str]) {
        self.0["blocks"] = match blocks {
            [one] => json!(one),
            many => json!(many),
        };
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent$ToolRule.getSpeed
    pub fn get_speed(&self) -> Option<f32> {
        self.0["speed"].as_f64().map(|speed| speed as f32)
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent$ToolRule.setSpeed
    pub fn set_speed(&mut self, speed: Option<f32>) {
        self.0["speed"] = json!(speed);
        if speed.is_none() {
            self.0.as_object_mut().map(|rule| rule.remove("speed"));
        }
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent$ToolRule.isCorrectForDrops
    pub fn is_correct_for_drops(&self) -> Option<bool> {
        self.0.get("correct_for_drops").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent$ToolRule.setCorrectForDrops
    pub fn set_correct_for_drops(&mut self, correct: Option<bool>) {
        self.0["correct_for_drops"] = json!(correct);
        if correct.is_none() {
            self.0.as_object_mut().map(|rule| rule.remove("correct_for_drops"));
        }
    }
}

impl crate::meta::ToolComponent {
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent.getRules
    pub fn get_rules(&self) -> Vec<ToolRule> {
        self.0.component_at("minecraft:tool", "rules").and_then(Value::as_array).into_iter().flatten().cloned().map(ToolRule).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent.setRules
    pub fn set_rules(&mut self, rules: &[ToolRule]) {
        self.0.set_component_at("minecraft:tool", "rules", Value::Array(rules.iter().map(|rule| rule.0.clone()).collect()));
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent.addRule
    ///
    /// For block ids, or one `#tag`.
    pub fn add_rule(&mut self, blocks: &[&str], speed: Option<f32>, correct: Option<bool>) -> ToolRule {
        let mut rule = ToolRule(json!({}));
        rule.set_blocks(blocks);
        rule.set_speed(speed);
        rule.set_correct_for_drops(correct);
        let mut rules = self.get_rules();
        rules.push(rule.clone());
        self.set_rules(&rules);
        rule
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent.removeRule
    pub fn remove_rule(&mut self, rule: &ToolRule) -> bool {
        let mut rules = self.get_rules();
        let before = rules.len();
        rules.retain(|known| known != rule);
        let removed = rules.len() != before;
        self.set_rules(&rules);
        removed
    }
}

impl crate::meta::EquippableComponent {
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.getAllowedEntities
    pub fn get_allowed_entities(&self) -> Vec<String> {
        self.0.component_at("minecraft:equippable", "allowed_entities").map_or_else(Vec::new, |allowed| holder_members(allowed, "entity_type"))
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setAllowedEntities
    ///
    /// Entity ids, or one `#tag`. Empty lets every entity wear it.
    pub fn set_allowed_entities(&mut self, entities: &[&str]) {
        if entities.is_empty() {
            let mut component = self.0.get_component("minecraft:equippable").cloned().unwrap_or_else(|| json!({}));
            component.as_object_mut().map(|fields| fields.remove("allowed_entities"));
            self.0.set_component("minecraft:equippable", component);
            return;
        }
        let saved = match entities {
            [one] => json!(one),
            many => json!(many),
        };
        self.0.set_component_at("minecraft:equippable", "allowed_entities", saved);
    }
}

impl crate::meta::JukeboxPlayableComponent {
    /// bukkit: org/bukkit/inventory/meta/components/JukeboxPlayableComponent.getSong
    pub fn get_song(&self) -> Option<String> {
        self.get_song_key()
    }
    /// bukkit: org/bukkit/inventory/meta/components/JukeboxPlayableComponent.setSong
    pub fn set_song(&mut self, song: &str) {
        self.set_song_key(song);
    }
    fn hidden(&self) -> Vec<Value> {
        self.0.component_at("minecraft:tooltip_display", "hidden_components").and_then(Value::as_array).cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/inventory/meta/components/JukeboxPlayableComponent.isShowInTooltip
    pub fn is_show_in_tooltip(&self) -> bool {
        !self.hidden().iter().any(|id| id == "minecraft:jukebox_playable")
    }
    /// bukkit: org/bukkit/inventory/meta/components/JukeboxPlayableComponent.setShowInTooltip
    pub fn set_show_in_tooltip(&mut self, show: bool) {
        let mut hidden = self.hidden();
        hidden.retain(|id| id != "minecraft:jukebox_playable");
        if !show {
            hidden.push(json!("minecraft:jukebox_playable"));
        }
        self.0.set_component_at("minecraft:tooltip_display", "hidden_components", Value::Array(hidden));
    }
}

// ---- The meta classes' own calls that weren't there ----

impl crate::meta::BookMeta {
    /// bukkit: org/bukkit/inventory/meta/BookMeta.title
    pub fn title(&self) -> Option<crate::Component> {
        self.get_title().map(|title| crate::adventure::legacy(&title))
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.author
    pub fn author(&self) -> Option<crate::Component> {
        self.get_author().map(|author| crate::adventure::legacy(&author))
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.page, org/bukkit/inventory/meta/BookMeta$Spigot.getPage
    ///
    /// Pages count from 1, as in Java.
    pub fn page(&self, page: usize) -> Option<crate::Component> {
        self.pages().get(page.checked_sub(1)?).cloned()
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta$Spigot.setPages
    ///
    /// Bungee's `BaseComponent[]` is one `Component` here, so `spigot()` is this class itself.
    pub fn set_page_components(&mut self, pages: &[crate::Component]) {
        let saved: Vec<Value> = pages.iter().map(|page| json!({ "raw": page.0 })).collect();
        self.0.set_component_at("minecraft:written_book_content", "pages", Value::Array(saved));
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta$Spigot.addPage
    pub fn add_page_component(&mut self, page: crate::Component) {
        let mut pages = self.pages();
        pages.push(page);
        self.set_page_components(&pages);
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta$Spigot.setPage
    pub fn set_page_component(&mut self, page: usize, text: crate::Component) {
        let mut pages = self.pages();
        if let Some(place) = page.checked_sub(1).and_then(|index| pages.get_mut(index)) {
            *place = text;
        }
        self.set_page_components(&pages);
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta$Spigot.getPages
    pub fn get_page_components(&self) -> Vec<crate::Component> {
        self.pages()
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.getGeneration
    ///
    /// The name of `BookMeta$Generation`.
    pub fn get_generation(&self) -> Option<&'static str> {
        GENERATIONS.get(usize::try_from(self.get_generation_number()?).ok()?).copied()
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.setGeneration
    pub fn set_generation(&mut self, generation: &str) {
        if let Some(number) = GENERATIONS.iter().position(|name| *name == generation) {
            self.set_generation_number(number as i32);
        }
    }
}

const GENERATIONS: [&str; 4] = ["ORIGINAL", "COPY_OF_ORIGINAL", "COPY_OF_COPY", "TATTERED"];

impl crate::meta::SkullMeta {
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.getPlayerProfile, org/bukkit/inventory/meta/SkullMeta.getOwnerProfile
    pub fn get_player_profile(&self) -> Option<crate::player::PlayerProfile> {
        let profile = self.0.get_component("minecraft:profile")?;
        let id = profile.get("id").and_then(crate::data::uuid).unwrap_or_default();
        Some(crate::extra::create_player_profile(&id, profile["name"].as_str().unwrap_or_default()))
    }
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.setOwningPlayer, org/bukkit/inventory/meta/SkullMeta.setPlayerProfile, org/bukkit/inventory/meta/SkullMeta.setOwnerProfile
    ///
    /// By the player's id and name: a head finds its skin by them.
    pub fn set_profile(&mut self, id: Option<&str>, name: Option<&str>) {
        let mut profile = Map::new();
        if let Some(name) = name {
            profile.insert("name".to_owned(), json!(name));
        }
        if let Some(words) = id.and_then(uuid_words) {
            profile.insert("id".to_owned(), json!(words));
        }
        self.0.set_component("minecraft:profile", Value::Object(profile));
    }
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.setOwningPlayer
    ///
    /// The player's head, by the player's id and name (`meta.setOwningPlayer(player)`; a Player is an OfflinePlayer).
    pub fn set_owning_player(&mut self, player: &crate::OfflinePlayer) -> bool {
        self.set_profile(player.get_unique_id(), Some(player.get_name()));
        true
    }
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.getOwningPlayer
    pub fn get_owning_player(&self) -> Option<crate::OfflinePlayer> {
        let name = self.get_owner()?;
        Some(crate::get_offline_player(&name))
    }
}

impl crate::ItemStack {
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.setOwnerProfile, org/bukkit/inventory/meta/SkullMeta.setPlayerProfile
    ///
    /// A whole profile on a head's meta (which is its stack here): id, name and properties (a skin's
    /// `textures`), as the game's `profile` component keeps them. `None` takes it off, as Java's `null`.
    pub fn set_owner_profile(&mut self, profile: Option<&crate::player::PlayerProfile>) {
        let Some(profile) = profile else {
            self.remove_component("minecraft:profile");
            return;
        };
        let mut out = Map::new();
        if !profile.name.is_empty() {
            out.insert("name".to_owned(), json!(profile.name));
        }
        if let Some(words) = uuid_words(&profile.id) {
            out.insert("id".to_owned(), json!(words));
        }
        let properties: Vec<Value> = profile
            .properties
            .iter()
            .map(|(name, value, signature)| match signature {
                Some(signature) => json!({ "name": name, "value": value, "signature": signature }),
                None => json!({ "name": name, "value": value }),
            })
            .collect();
        if !properties.is_empty() {
            out.insert("properties".to_owned(), Value::Array(properties));
        }
        self.set_component("minecraft:profile", Value::Object(out));
    }
}

/// SNBT as JSON, read by the game.
pub(crate) fn snbt_json(snbt: &str) -> Value {
    query(json!({ "op": "items_snbt", "snbt": snbt }))
}

/// A UUID as the four ints it's saved in, as JSON keeps them.
fn uuid_words(uuid: &str) -> Option<[i32; 4]> {
    let value = u128::from_str_radix(&uuid.replace('-', ""), 16).ok()?;
    let word = |shift: u32| (value >> shift) as u32 as i32;
    Some([word(96), word(64), word(32), word(0)])
}

impl crate::meta::CompassMeta {
    /// bukkit: org/bukkit/inventory/meta/CompassMeta.isLodestoneCompass
    pub fn is_lodestone_compass(&self) -> bool {
        self.0.get_component("minecraft:lodestone_tracker").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/CompassMeta.hasLodestone
    pub fn has_lodestone(&self) -> bool {
        self.0.component_at("minecraft:lodestone_tracker", "target").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/CompassMeta.getLodestone
    pub fn get_lodestone(&self) -> Option<Location> {
        let target = self.0.component_at("minecraft:lodestone_tracker", "target")?;
        let at = |i: usize| target["pos"][i].as_f64();
        Some(Location::new(World::of(target["dimension"].as_str()?), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/inventory/meta/CompassMeta.setLodestone
    pub fn set_lodestone(&mut self, location: Option<&Location>) {
        match location {
            Some(location) => {
                let dimension = location.get_world().dimension();
                self.0.set_component_at("minecraft:lodestone_tracker", "target", json!({ "dimension": dimension, "pos": [location.get_block_x(), location.get_block_y(), location.get_block_z()] }));
            }
            None => self.clear_lodestone(),
        }
    }
    /// bukkit: org/bukkit/inventory/meta/CompassMeta.clearLodestone
    pub fn clear_lodestone(&mut self) {
        let Some(mut tracker) = self.0.get_component("minecraft:lodestone_tracker").cloned() else {
            return;
        };
        tracker.as_object_mut().map(|fields| fields.remove("target"));
        self.0.set_component("minecraft:lodestone_tracker", tracker);
    }
}

impl crate::meta::MapMeta {
    /// bukkit: org/bukkit/inventory/meta/MapMeta.hasLocationName
    pub fn has_location_name(&self) -> bool {
        self.0.has_item_name()
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.getLocationName
    pub fn get_location_name(&self) -> Option<String> {
        self.0.get_item_name()
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.setLocationName
    pub fn set_location_name(&mut self, name: Option<&str>) {
        match name {
            Some(name) => self.0.set_item_name(name),
            None => {
                self.0.remove_component("minecraft:item_name");
            }
        }
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.isScaling
    pub fn is_scaling(&self) -> bool {
        self.0.component_at("minecraft:map_post_processing", "") == Some(&json!("scale"))
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.setScaling
    pub fn set_scaling(&mut self, scaling: bool) {
        if scaling {
            self.0.set_component("minecraft:map_post_processing", json!("scale"));
        } else {
            self.0.remove_component("minecraft:map_post_processing");
        }
    }
}

impl crate::meta::ShieldMeta {
    /// bukkit: org/bukkit/inventory/meta/ShieldMeta.getBaseColor
    pub fn get_base_color(&self) -> Option<String> {
        self.get_base_color_name().map(|color| color.to_ascii_uppercase())
    }
    /// bukkit: org/bukkit/inventory/meta/ShieldMeta.setBaseColor
    pub fn set_base_color(&mut self, color: Option<&str>) {
        match color {
            Some(color) => self.set_base_color_name(&color.to_ascii_lowercase()),
            None => {
                self.0.remove_component("minecraft:base_color");
            }
        }
    }
}

impl crate::meta::ArmorMeta {
    /// bukkit: org/bukkit/inventory/meta/ArmorMeta.getTrim
    ///
    /// The component, for `core3::trim_material` and `trim_pattern` to read.
    pub fn get_trim(&self) -> Option<&Value> {
        self.0.get_component("minecraft:trim")
    }
    /// bukkit: org/bukkit/inventory/meta/ArmorMeta.setTrim
    pub fn set_trim(&mut self, material: &str, pattern: &str) {
        self.0.set_component("minecraft:trim", json!({ "material": material, "pattern": pattern }));
    }
    /// A trim from `create_trim_material` and `create_trim_pattern`, or ids: each is what the component holds.
    pub fn set_trim_inline(&mut self, material: Value, pattern: Value) {
        self.0.set_component("minecraft:trim", json!({ "material": material, "pattern": pattern }));
    }
}

impl crate::meta::SpawnEggMeta {
    /// bukkit: org/bukkit/inventory/meta/SpawnEggMeta.getSpawnedType
    pub fn get_spawned_type(&self) -> String {
        self.get_custom_spawned_type().unwrap_or_else(|| self.0.get_type().key().trim_end_matches("_spawn_egg").to_owned())
    }
    /// bukkit: org/bukkit/inventory/meta/SpawnEggMeta.setSpawnedType
    pub fn set_spawned_type(&mut self, kind: &str) {
        self.set_custom_spawned_type(kind);
    }
    /// bukkit: org/bukkit/inventory/meta/SpawnEggMeta.getSpawnedEntity
    pub fn get_spawned_entity(&self) -> Option<crate::entities3::EntitySnapshot> {
        self.0.get_component("minecraft:entity_data").map(|data| crate::entities3::create_entity_snapshot(&crate::data::snbt(data)))
    }
    /// bukkit: org/bukkit/inventory/meta/SpawnEggMeta.setSpawnedEntity
    pub fn set_spawned_entity(&mut self, snapshot: &crate::entities3::EntitySnapshot) -> bool {
        let value = snbt_json(&snapshot.0);
        let read = !value.is_null();
        if read {
            self.0.set_component("minecraft:entity_data", value);
        }
        read
    }
}

impl crate::meta::EnchantmentStorageMeta {
    /// bukkit: org/bukkit/inventory/meta/EnchantmentStorageMeta.hasConflictingStoredEnchant
    pub fn has_conflicting_stored_enchant(&self, enchantment: &crate::registry::Enchantment) -> bool {
        self.get_stored_enchants().keys().filter_map(|key| crate::registry::Enchantment::get_by_key(key)).any(|other| other.get_key() != enchantment.get_key() && other.conflicts_with(enchantment))
    }
}

impl crate::meta::BlockDataMeta {
    /// bukkit: org/bukkit/inventory/meta/BlockDataMeta.hasBlockData
    pub fn has_block_data(&self) -> bool {
        self.0.get_component("minecraft:block_state").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/BlockDataMeta.getBlockData
    pub fn get_block_data(&self, material: &Material) -> BlockData {
        let properties: Vec<String> = self.0.get_component("minecraft:block_state").and_then(Value::as_object).into_iter().flatten().filter_map(|(key, value)| Some(format!("{key}={}", value.as_str()?))).collect();
        BlockData::parse(&format!("{}[{}]", material.key(), properties.join(",")))
    }
    /// bukkit: org/bukkit/inventory/meta/BlockDataMeta.setBlockData
    pub fn set_block_data(&mut self, data: &BlockData) {
        let text = data.get_as_string();
        let inside = text.split_once('[').map_or("", |(_, rest)| rest.trim_end_matches(']'));
        let properties: Map<String, Value> = inside.split(',').filter_map(|pair| pair.split_once('=')).map(|(key, value)| (key.to_owned(), json!(value))).collect();
        self.0.set_component("minecraft:block_state", Value::Object(properties));
    }
}

impl crate::meta::BlockStateMeta {
    /// bukkit: org/bukkit/inventory/meta/BlockStateMeta.hasBlockState
    pub fn has_block_state(&self) -> bool {
        self.0.get_component("minecraft:block_entity_data").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/BlockStateMeta.clearBlockState
    pub fn clear_block_state(&mut self) {
        self.0.remove_component("minecraft:block_entity_data");
    }
    /// The block entity's saved data as the stack has it.
    pub fn get_block_state(&self) -> Option<&Value> {
        self.0.get_component("minecraft:block_entity_data")
    }
    /// The block entity's saved data, with its `id`, as the game wants it.
    pub fn set_block_state(&mut self, data: Value) {
        self.0.set_component("minecraft:block_entity_data", data);
    }
    /// bukkit: org/bukkit/inventory/meta/BlockStateMeta.getBlockState
    ///
    /// `meta.state::<Chest>()` is Java's cast. A state of no block: `get_inventory` is the snapshot's, as
    /// in Java, and what needs the world panics where Java throws.
    pub fn state<C: StateClass>(&self) -> C {
        let mut data = self.get_block_state().cloned().unwrap_or_else(|| json!({}));
        let kind = self.0.get_type().clone();
        let mut slots = vec![None; slots_of(&kind)];
        if let Some(rows) = self.0.get_component("minecraft:container").and_then(Value::as_array) {
            let saved: Vec<Value> = rows.iter().map(|row| {
                let mut item = row["item"].clone();
                item["Slot"] = row["slot"].clone();
                item
            }).collect();
            slots = items_of(&json!(saved), slots.len());
            data["Items"] = Value::Array(saved);
        }
        let held = Held { slots: Some(HeldSlots(std::sync::Arc::new(std::sync::Mutex::new(slots)))), kind: Some(kind) };
        C::of(TileState { block: World::of("world").get_block_at(0, UNPLACED_Y, 0), data, changes: Vec::new(), held, state: String::new() })
    }
    /// bukkit: org/bukkit/inventory/meta/BlockStateMeta.setBlockState
    ///
    /// What the state was changed to goes on the stack: its slots as the `container` component, the rest
    /// as `block_entity_data`, and what the game's block entity doesn't keep under `horizon:extras`, which
    /// placing the item puts back.
    pub fn set_state(&mut self, state: &TileState) {
        let mut data = state.data.clone();
        let mut snapshot = false;
        for (path, value) in &state.changes {
            if let Some(field) = path.strip_prefix("extra:") {
                data["horizon:extras"][field] = serde_json::from_str(value).unwrap_or(Value::Null);
                continue;
            }
            let path = path.trim_start_matches('=');
            snapshot |= path == "Items";
            let parsed = snbt_json(value);
            let mut place = &mut data;
            let mut keys = path.split('.').map(|key| key.trim_matches('"')).peekable();
            while let Some(key) = keys.next() {
                if keys.peek().is_none() {
                    place[key] = parsed;
                    break;
                }
                place = &mut place[key];
            }
        }
        let rows = data.as_object_mut().and_then(|fields| fields.remove("Items"));
        let slots: Option<Vec<Value>> = if snapshot {
            rows.as_ref().and_then(Value::as_array).map(|rows| rows.iter().map(|row| {
                let mut item = row.clone();
                let slot = item.as_object_mut().and_then(|fields| fields.remove("Slot")).unwrap_or(json!(0));
                json!({ "slot": slot, "item": item })
            }).collect())
        } else {
            state.held.slots.as_ref().map(|slots| slots.get().iter().enumerate().filter_map(|(slot, stack)| Some(json!({ "slot": slot, "item": stack.as_ref()?.saved_row() }))).collect())
        };
        match slots {
            Some(rows) if rows.is_empty() => {
                self.0.remove_component("minecraft:container");
            }
            Some(rows) => self.0.set_component("minecraft:container", Value::Array(rows)),
            None => {}
        }
        if data.as_object().is_some_and(|fields| fields.keys().any(|key| key != "id")) {
            if data.get("id").is_none() {
                data["id"] = query(json!({ "op": "items_be_id", "block": self.0.get_type().key() }));
            }
            self.0.set_component("minecraft:block_entity_data", data);
        }
    }
}

impl crate::meta::TropicalFishBucketMeta {
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.hasVariant
    pub fn has_variant(&self) -> bool {
        self.has_pattern() || self.has_body_color() || self.has_pattern_color()
    }
}

impl crate::meta::PotionMeta {
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.hasCustomName
    pub fn has_custom_name(&self) -> bool {
        self.has_custom_potion_name()
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.getCustomName
    pub fn get_custom_name(&self) -> Option<String> {
        self.get_custom_potion_name()
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.setCustomName
    pub fn set_custom_name(&mut self, name: &str) {
        self.set_custom_potion_name(name);
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.getBasePotionData
    pub fn get_base_potion_data(&self) -> Option<crate::core3::PotionData> {
        self.get_base_potion_type().map(|id| crate::core3::PotionData::from_id(&id))
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.setBasePotionData
    pub fn set_base_potion_data(&mut self, data: &crate::core3::PotionData) {
        let prefix = if data.is_extended() { "long_" } else if data.is_upgraded() { "strong_" } else { "" };
        self.set_base_potion_type(&format!("minecraft:{prefix}{}", data.get_type().to_ascii_lowercase()));
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.getAllEffects
    pub fn get_all_effects(&self) -> Vec<crate::PotionEffect> {
        let mut effects = self.get_base_potion_type().map_or_else(Vec::new, |id| potion_effects(&id));
        effects.extend(self.get_custom_effects());
        effects
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.setMainEffect
    pub fn set_main_effect(&mut self, effect: &str) -> bool {
        let mut effects = self.get_custom_effects();
        let Some(index) = effects.iter().position(|known| known.is(effect)) else {
            return false;
        };
        let moved = effects.remove(index);
        effects.insert(0, moved);
        self.0.set_component_at("minecraft:potion_contents", "custom_effects", Value::Array(effects.iter().map(crate::PotionEffect::to_saved).collect()));
        index != 0
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.computeEffectiveColor
    pub fn compute_effective_color(&self) -> Color {
        if let Some(color) = self.get_color() {
            return color;
        }
        let saved: Vec<Value> = self.get_all_effects().iter().map(crate::PotionEffect::to_saved).collect();
        let rgb = query(json!({ "op": "items_potion_color", "effects": saved })).as_i64().unwrap_or(0x385DC6);
        Color::from_rgb(((rgb >> 16) & 255) as u8, ((rgb >> 8) & 255) as u8, (rgb & 255) as u8)
    }
}

/// The effects a potion id gives, from the game's own table.
fn potion_effects(id: &str) -> Vec<crate::PotionEffect> {
    query(json!({ "op": "items_potion", "id": id }))["effects"].as_array().into_iter().flatten().filter_map(crate::PotionEffect::from_saved).collect()
}

// ---- ItemStack's own calls ----

impl ItemStack {
    /// bukkit: org/bukkit/inventory/ItemStack.of
    pub fn of(material: &Material, amount: i32) -> Self {
        Self::new(material.clone(), amount)
    }
    /// bukkit: org/bukkit/inventory/ItemStack.withType
    pub fn with_type(&self, material: &Material) -> Self {
        let mut stack = self.clone();
        stack.set_type(material.clone());
        stack
    }
    /// bukkit: org/bukkit/inventory/ItemStack.addEnchantments, org/bukkit/inventory/ItemStack.addUnsafeEnchantments
    pub fn add_enchantments(&mut self, enchantments: &BTreeMap<String, u32>) {
        for (key, level) in enchantments {
            self.add_enchant(key, *level);
        }
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getDurability
    pub fn get_durability(&self) -> i16 {
        self.get_damage() as i16
    }
    /// bukkit: org/bukkit/inventory/ItemStack.setDurability
    pub fn set_durability(&mut self, durability: i16) {
        self.set_damage(i32::from(durability));
    }
    /// bukkit: org/bukkit/inventory/ItemStack.damage
    ///
    /// `livingEntity.damageItemStack(this, amount)`: Unbreaking rolls, the entity's item damage event can
    /// change or cancel it, and a player's break event fires; `Self::empty()` when it breaks.
    pub fn damage(&self, amount: i32, entity: &crate::entity::Entity) -> Self {
        if self.is_unbreakable() {
            return self.clone();
        }
        let max = self.get_max_damage().unwrap_or_else(|| ItemType(self.get_type().key().to_owned()).get_max_durability().into());
        let player = crate::get_online_players().into_iter().find(|player| player.get_entity_id() as u64 == entity.get_entity_id());
        Self::read(&query(json!({ "op": "items_stack_damage", "stack": self.to_json(), "amount": amount, "max": max, "entity": entity.get_entity_id(), "uuid": player.as_ref().map(Player::uuid) }))).unwrap_or_else(Self::empty)
    }
    /// bukkit: org/bukkit/inventory/ItemStack.editMeta
    ///
    /// The meta class goes in by the closure's argument: `stack.edit_meta(|meta: &mut BookMeta| ...)`.
    pub fn edit_meta<M: crate::meta::MetaClass>(&mut self, edit: impl FnOnce(&mut M)) -> bool {
        let mut meta: M = self.meta();
        edit(&mut meta);
        self.set_meta(&meta);
        true
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getPersistentDataContainer, org/bukkit/inventory/ItemStack.editPersistentDataContainer
    ///
    /// `pdc_*` are the container's calls: this runs them on the stack.
    pub fn edit_persistent_data_container(&mut self, edit: impl FnOnce(&mut Self)) -> bool {
        edit(self);
        true
    }
    /// bukkit: org/bukkit/inventory/ItemStack.effectiveName
    pub fn effective_name(&self) -> crate::Component {
        self.display_name().or_else(|| self.item_name()).unwrap_or_else(|| crate::Component(ItemType(self.get_type().key().to_owned()).default_component("minecraft:item_name")))
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getTranslationKey, org/bukkit/inventory/ItemStack.translationKey
    pub fn translation_key(&self) -> String {
        ItemType(self.get_type().key().to_owned()).get_translation_key()
    }
    /// bukkit: org/bukkit/inventory/ItemStack.ensureServerConversions
    ///
    /// A stack here is the game's form already.
    pub fn ensure_server_conversions(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/inventory/ItemStack.isRepairableBy
    pub fn is_repairable_by(&self, material: &Self) -> bool {
        let saved = self.get_component("minecraft:repairable").cloned().or_else(|| ItemType(self.get_type().key().to_owned()).get_default_data("minecraft:repairable"));
        saved.is_some_and(|repairable| holder_members(&repairable["items"], "item").iter().any(|id| id == material.get_type().key()))
    }
    /// bukkit: org/bukkit/inventory/ItemStack.canRepair
    pub fn can_repair(&self, damaged: &Self) -> bool {
        damaged.is_repairable_by(self)
    }
    /// bukkit: org/bukkit/inventory/ItemStack.copyDataFrom
    ///
    /// The stack's components that `filter` lets through.
    pub fn copy_data_from(&mut self, source: &Self, filter: impl Fn(&str) -> bool) {
        for (id, value) in source.all_components() {
            if filter(id) {
                self.set_component(id, value.clone());
            }
        }
    }
    /// bukkit: org/bukkit/inventory/ItemStack.matchesWithoutData
    pub fn matches_without_data(&self, other: &Self, excluded: &[&str]) -> bool {
        let kept = |stack: &Self| stack.all_components().filter(|(id, _)| !excluded.contains(&id.as_str())).map(|(id, value)| (id.clone(), value.clone())).collect::<BTreeMap<_, _>>();
        self.get_type() == other.get_type() && kept(self) == kept(other)
    }
    /// bukkit: org/bukkit/inventory/ItemStack.hasData, io/papermc/paper/datacomponent/DataComponentView.hasData
    ///
    /// On the stack, or in its type's defaults.
    pub fn has_data(&self, component: &str) -> bool {
        self.get_data(component).is_some()
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getData, io/papermc/paper/datacomponent/DataComponentView.getData
    ///
    /// By id, as JSON: the classes in `components.rs` wrap it.
    pub fn get_data(&self, component: &str) -> Option<Value> {
        if self.get_component(&format!("!{component}")).is_some() {
            return None;
        }
        self.get_component(component).cloned().or_else(|| ItemType(self.get_type().key().to_owned()).get_default_data(component))
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getDataOrDefault, io/papermc/paper/datacomponent/DataComponentView.getDataOrDefault
    pub fn get_data_or_default(&self, component: &str, fallback: Value) -> Value {
        self.get_data(component).unwrap_or(fallback)
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getDataTypes
    pub fn get_data_types(&self) -> Vec<String> {
        let mut types = ItemType(self.get_type().key().to_owned()).get_default_data_types();
        types.extend(self.all_components().map(|(id, _)| id.clone()));
        types.sort();
        types.dedup();
        types.retain(|id| !id.starts_with('!') && self.get_component(&format!("!{id}")).is_none());
        types
    }
    /// bukkit: org/bukkit/inventory/ItemStack.isDataOverridden
    pub fn is_data_overridden(&self, component: &str) -> bool {
        self.get_component(component).is_some() || self.get_component(&format!("!{component}")).is_some()
    }
    /// bukkit~: org/bukkit/inventory/ItemStack.unsetData | the stack, its saved form and clients have it gone; the engine's own rules (eating, tool speed, stack size...) read the item's table by id at many sites, so they still see the default
    pub fn unset_data(&mut self, component: &str) {
        self.remove_component(component);
        self.set_component(&format!("!{component}"), json!({}));
    }
    /// bukkit: org/bukkit/inventory/ItemStack.resetData
    pub fn reset_data(&mut self, component: &str) {
        self.remove_component(component);
        self.remove_component(&format!("!{component}"));
    }
    /// A stack as a saved row: its components as JSON.
    fn saved_row(&self) -> Value {
        let components: Map<String, Value> = self.all_components().map(|(id, value)| (id.clone(), value.clone())).collect();
        json!({ "id": self.get_type().key(), "count": self.get_amount(), "components": components })
    }
    /// bukkit: org/bukkit/inventory/ItemStack.serialize
    pub fn serialize(&self) -> BTreeMap<String, Value> {
        let version = query(json!({ "op": "items_data_version" }));
        let mut saved = BTreeMap::from([("DataVersion".to_owned(), version), ("schema_version".to_owned(), json!(1))]);
        if self.get_type().is_air() || self.get_amount() < 1 {
            saved.insert("id".to_owned(), json!("minecraft:air"));
            return saved;
        }
        saved.insert("id".to_owned(), json!(self.get_type().key()));
        saved.insert("count".to_owned(), json!(self.get_amount()));
        let components: Map<String, Value> = self.all_components().map(|(id, value)| (id.clone(), json!(crate::data::snbt(value)))).collect();
        if !components.is_empty() {
            saved.insert("components".to_owned(), Value::Object(components));
        }
        saved
    }
    /// bukkit: org/bukkit/inventory/ItemStack.deserialize
    ///
    /// A map of `serialize`, components as SNBT, or Bukkit's old `type`, `amount`, `damage`.
    pub fn deserialize(saved: &BTreeMap<String, Value>) -> Option<Self> {
        if let Some(id) = saved.get("id").and_then(Value::as_str) {
            let count = saved.get("count").and_then(Value::as_i64).unwrap_or(1) as i32;
            let mut stack = Self::new(Material::of(id), count);
            for (key, value) in saved.get("components").and_then(Value::as_object).into_iter().flatten() {
                stack.set_component(key, value.as_str().map_or_else(|| value.clone(), snbt_json));
            }
            return Some(stack);
        }
        let kind = saved.get("type")?.as_str()?;
        let mut stack = Self::new(Material::of(kind), saved.get("amount").and_then(Value::as_i64).unwrap_or(1) as i32);
        if let Some(damage) = saved.get("damage").and_then(Value::as_i64).filter(|damage| *damage != 0) {
            stack.set_damage(damage as i32);
        }
        Some(stack)
    }
    /// bukkit: org/bukkit/inventory/ItemStack.serializeAsBytes
    ///
    /// # Panics
    /// As Java throws, for an empty stack.
    pub fn serialize_as_bytes(&self) -> Vec<u8> {
        assert!(!self.get_type().is_air() && self.get_amount() > 0, "Cannot serialize empty ItemStack");
        query(json!({ "op": "items_stack_bytes", "stack": self.to_json() })).as_array().into_iter().flatten().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect()
    }
    /// bukkit: org/bukkit/inventory/ItemStack.deserializeBytes
    pub fn deserialize_bytes(bytes: &[u8]) -> Option<Self> {
        Self::read(&query(json!({ "op": "items_stack_from_bytes", "bytes": bytes })))
    }
    /// bukkit: org/bukkit/inventory/ItemStack.serializeItemsAsBytes
    ///
    /// A format byte, the count, then each stack's length and bytes, 0 for an empty slot, all big endian.
    pub fn serialize_items_as_bytes(stacks: &[Option<Self>]) -> Vec<u8> {
        let mut out = vec![1];
        out.extend_from_slice(&(stacks.len() as i32).to_be_bytes());
        for stack in stacks {
            let bytes = stack.as_ref().filter(|stack| !stack.get_type().is_air() && stack.get_amount() > 0).map(Self::serialize_as_bytes).unwrap_or_default();
            out.extend_from_slice(&(bytes.len() as i32).to_be_bytes());
            out.extend_from_slice(&bytes);
        }
        out
    }
    /// bukkit: org/bukkit/inventory/ItemStack.deserializeItemsFromBytes
    pub fn deserialize_items_from_bytes(bytes: &[u8]) -> Vec<Option<Self>> {
        let mut out = Vec::new();
        let word = |at: usize| Some(i32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
        if bytes.first() != Some(&1) {
            return out;
        }
        let mut at = 5;
        for _ in 0..word(1).unwrap_or(0) {
            let Some(length) = word(at).and_then(|length| usize::try_from(length).ok()) else {
                break;
            };
            at += 4;
            out.push(if length == 0 { None } else { bytes.get(at..at + length).and_then(Self::deserialize_bytes) });
            at += length;
        }
        out
    }
    /// bukkit: org/bukkit/inventory/ItemStack.asHoverEvent
    ///
    /// The `hover_event` of a text component, as the game writes it.
    pub fn as_hover_event(&self) -> Value {
        let components: Map<String, Value> = self.all_components().map(|(id, value)| (id.clone(), value.clone())).collect();
        json!({ "action": "show_item", "id": self.get_type().key(), "count": self.get_amount(), "components": components })
    }
}

/// `org.bukkit.inventory.ItemFactory`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemFactory;

impl ItemFactory {
    /// bukkit: org/bukkit/inventory/ItemFactory.getItemMeta
    ///
    /// A stack is its own meta here: a fresh one, for `meta::<C>()`.
    pub fn get_item_meta(&self, material: &Material) -> ItemStack {
        ItemStack::new(material.clone(), 1)
    }
    /// bukkit: org/bukkit/inventory/ItemFactory.isApplicable
    ///
    /// A stack is its own meta here: of the class its item has. The plain class fits every item but air,
    /// any other only the items of that class.
    pub fn is_applicable(&self, meta: &ItemStack, stack: &ItemStack) -> bool {
        let class = meta_class_of(meta.get_type());
        !stack.get_type().is_air() && (class == "ItemMeta" || class == meta_class_of(stack.get_type()))
    }
    /// bukkit: org/bukkit/inventory/ItemFactory.asMetaFor
    pub fn as_meta_for(&self, meta: &ItemStack, material: &Material) -> ItemStack {
        meta.with_type(material)
    }
    /// bukkit: org/bukkit/inventory/ItemFactory.ensureServerConversions
    pub fn ensure_server_conversions(&self, stack: &ItemStack) -> ItemStack {
        stack.ensure_server_conversions()
    }
    /// bukkit: org/bukkit/inventory/ItemFactory.displayName
    pub fn display_name(&self, stack: &ItemStack) -> crate::Component {
        stack.effective_name()
    }
    /// bukkit: org/bukkit/inventory/ItemFactory.asHoverEvent
    pub fn as_hover_event(&self, stack: &ItemStack) -> Value {
        stack.as_hover_event()
    }
    /// bukkit: org/bukkit/inventory/ItemFactory.hoverContentOf
    pub fn hover_content_of(&self, stack: &ItemStack) -> Value {
        stack.as_hover_event()
    }
}

// ---- Block entities: what holds items, read and written live as `Container.getInventory` is ----

use crate::block_data::{StateClass, TileState};

/// `org.bukkit.inventory.InventoryHolder`.
pub trait InventoryHolder {
    type Inventory;
    /// bukkit: org/bukkit/inventory/InventoryHolder.getInventory
    fn get_inventory(&self) -> Self::Inventory;
}

impl InventoryHolder for Player {
    type Inventory = crate::PlayerInventory;
    fn get_inventory(&self) -> crate::PlayerInventory {
        Self::get_inventory(self)
    }
}

/// The saved list of a block entity's slots.
fn items_of(value: &Value, size: usize) -> Vec<Option<ItemStack>> {
    let mut stacks = vec![None; size];
    for row in value.as_array().into_iter().flatten() {
        if let (Some(slot), Some(stack)) = (row["Slot"].as_u64().map(|slot| slot as usize), ItemStack::read(row))
            && let Some(place) = stacks.get_mut(slot)
        {
            *place = Some(stack);
        }
    }
    stacks
}

fn items_snbt(stacks: &[Option<ItemStack>]) -> String {
    let rows: Vec<String> = stacks.iter().enumerate().filter_map(|(slot, stack)| Some(stack.as_ref()?.to_snbt().replacen('{', &format!("{{Slot:{slot}b,"), 1))).collect();
    format!("[{}]", rows.join(","))
}

fn block_data_of(block: &Block) -> Value {
    query(json!({ "op": "block_data", "dimension": block.get_world().dimension(), "pos": [block.get_x(), block.get_y(), block.get_z()] }))
}

fn block_merge(block: &Block, snbt: &str) -> bool {
    query(json!({ "op": "block_merge", "dimension": block.get_world().dimension(), "pos": [block.get_x(), block.get_y(), block.get_z()], "snbt": snbt })).as_bool().unwrap_or(false)
}

/// How many slots a block entity's container has, by the block.
fn container_size(block: &Block) -> usize {
    slots_of(&block.get_type())
}

fn slots_of(material: &Material) -> usize {
    match material.key().trim_start_matches("minecraft:") {
        "dispenser" | "dropper" | "crafter" => 9,
        "hopper" => 5,
        "furnace" | "blast_furnace" | "smoker" | "shelf" => 3,
        "brewing_stand" => 5,
        "campfire" | "soul_campfire" => 4,
        "chiseled_bookshelf" => 6,
        "decorated_pot" | "jukebox" | "lectern" => 1,
        name if name.ends_with("_shelf") => 3,
        _ => 27,
    }
}

/// `org.bukkit.inventory.Inventory` of a block: `Container.getInventory`, live: each write reaches the
/// block entity at once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockInventory {
    block: Block,
    size: usize,
    /// The other half of a double chest, whose slots come after `block`'s.
    second: Option<Block>,
    /// A state of no block's own slots: Java hands out the snapshot inventory for one.
    held: Option<HeldSlots>,
}

impl BlockInventory {
    /// The inventory of the block entity at `block`.
    pub fn of(block: &Block) -> Self {
        Self { block: block.clone(), size: container_size(block), second: None, held: None }
    }
    /// bukkit: org/bukkit/inventory/Inventory.getSize
    pub fn get_size(&self) -> usize {
        self.size
    }
    /// bukkit: org/bukkit/inventory/Inventory.getContents, org/bukkit/inventory/Inventory.getStorageContents
    pub fn get_contents(&self) -> Vec<Option<ItemStack>> {
        if let Some(held) = &self.held {
            return held.get();
        }
        let Some(second) = &self.second else {
            return items_of(&block_data_of(&self.block)["Items"], self.size);
        };
        let half = self.size / 2;
        let mut stacks = items_of(&block_data_of(&self.block)["Items"], half);
        stacks.extend(items_of(&block_data_of(second)["Items"], half));
        stacks
    }
    /// bukkit: org/bukkit/inventory/Inventory.getItem
    pub fn get_item(&self, slot: usize) -> Option<ItemStack> {
        self.get_contents().get(slot).cloned().flatten()
    }
    /// bukkit: org/bukkit/inventory/Inventory.setContents, org/bukkit/inventory/Inventory.setStorageContents
    pub fn set_contents(&self, stacks: &[Option<ItemStack>]) -> bool {
        if let Some(held) = &self.held {
            let mut slots = stacks.to_vec();
            slots.resize(self.size, None);
            *held.0.lock().unwrap_or_else(|e| e.into_inner()) = slots;
            return true;
        }
        let Some(second) = &self.second else {
            return block_merge(&self.block, &format!("{{Items:{}}}", items_snbt(stacks)));
        };
        let half = self.size / 2;
        let (first, rest) = stacks.split_at(half.min(stacks.len()));
        block_merge(&self.block, &format!("{{Items:{}}}", items_snbt(first))) && block_merge(second, &format!("{{Items:{}}}", items_snbt(rest)))
    }
    /// bukkit: org/bukkit/inventory/Inventory.setItem
    pub fn set_item(&self, slot: usize, stack: Option<ItemStack>) -> bool {
        let mut stacks = self.get_contents();
        let Some(place) = stacks.get_mut(slot) else {
            return false;
        };
        *place = stack;
        self.set_contents(&stacks)
    }
    /// bukkit: org/bukkit/inventory/Inventory.getMaxStackSize
    ///
    /// A container's, as Bukkit gives it (99); an item's own limit applies on top (CoreProtect's hopper logging asks it).
    pub fn get_max_stack_size(&self) -> i32 {
        99
    }
    /// bukkit: org/bukkit/inventory/Inventory.addItem
    ///
    /// Into the stacks that have room, then the empty slots, in one read and one write; what didn't fit comes
    /// back. (A chest's inventory had no addItem: CoreProtect's rollback couldn't put a chest's items back.)
    pub fn add_item(&self, stack: &ItemStack) -> Option<ItemStack> {
        let mut stacks = self.get_contents();
        let mut left = stack.get_amount();
        let limit = stack.get_max_stack_size();
        for place in stacks.iter_mut() {
            if let Some(held) = place.as_mut().filter(|held| held.is_similar(stack)) {
                let moved = (limit - held.get_amount()).clamp(0, left);
                left -= moved;
                *held = held.as_quantity(held.get_amount() + moved);
            }
        }
        for place in stacks.iter_mut() {
            if left == 0 {
                break;
            }
            if place.as_ref().is_none_or(|held| held.get_type().is_air()) {
                let moved = left.min(limit);
                left -= moved;
                *place = Some(stack.as_quantity(moved));
            }
        }
        self.set_contents(&stacks);
        (left > 0).then(|| stack.as_quantity(left))
    }
    /// bukkit: org/bukkit/inventory/Inventory.removeItem
    ///
    /// From the similar stacks, first to last; what couldn't be found comes back.
    pub fn remove_item(&self, stack: &ItemStack) -> Option<ItemStack> {
        let mut stacks = self.get_contents();
        let mut left = stack.get_amount();
        for place in stacks.iter_mut() {
            if left == 0 {
                break;
            }
            if let Some(held) = place.clone().filter(|held| held.is_similar(stack)) {
                let taken = held.get_amount().min(left);
                left -= taken;
                *place = (held.get_amount() > taken).then(|| held.as_quantity(held.get_amount() - taken));
            }
        }
        self.set_contents(&stacks);
        (left > 0).then(|| stack.as_quantity(left))
    }
    /// bukkit: org/bukkit/inventory/Inventory.clear
    pub fn clear(&self) -> bool {
        self.set_contents(&[])
    }
    /// bukkit: org/bukkit/inventory/Inventory.getLocation
    pub fn get_location(&self) -> Location {
        self.block.get_location()
    }
    /// bukkit: org/bukkit/inventory/Inventory.getHolder, org/bukkit/inventory/BlockInventoryHolder.getBlock
    pub fn get_block(&self) -> &Block {
        &self.block
    }
}

impl InventoryHolder for Block {
    type Inventory = BlockInventory;
    fn get_inventory(&self) -> BlockInventory {
        let Some(double) = DoubleChest::of(self) else {
            return BlockInventory::of(self);
        };
        let mut whole = BlockInventory::of(&double.left);
        whole.size *= 2;
        whole.second = Some(double.right);
        whole
    }
}

/// The block entity as read at `tile`, as `Container.getSnapshotInventory` keeps it: a copy, for `update`.
pub struct SnapshotInventory<'a> {
    tile: &'a mut TileState,
    size: usize,
}

impl SnapshotInventory<'_> {
    /// The copy's slots.
    pub fn get_contents(&self) -> Vec<Option<ItemStack>> {
        items_of(&self.tile.saved("Items"), self.size)
    }
    /// A slot of the copy: `update` writes it to the block.
    pub fn set_contents(&mut self, stacks: &[Option<ItemStack>]) {
        self.tile.change("Items", items_snbt(stacks));
    }
}

/// `org.bukkit.inventory.FurnaceInventory`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FurnaceInventory(pub BlockInventory);

impl FurnaceInventory {
    /// bukkit: org/bukkit/inventory/FurnaceInventory.getSmelting
    pub fn get_smelting(&self) -> Option<ItemStack> {
        self.0.get_item(0)
    }
    /// bukkit: org/bukkit/inventory/FurnaceInventory.setSmelting
    pub fn set_smelting(&self, stack: Option<ItemStack>) {
        self.0.set_item(0, stack);
    }
    /// bukkit: org/bukkit/inventory/FurnaceInventory.getFuel
    pub fn get_fuel(&self) -> Option<ItemStack> {
        self.0.get_item(1)
    }
    /// bukkit: org/bukkit/inventory/FurnaceInventory.setFuel
    pub fn set_fuel(&self, stack: Option<ItemStack>) {
        self.0.set_item(1, stack);
    }
    /// bukkit: org/bukkit/inventory/FurnaceInventory.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.0.get_item(2)
    }
    /// bukkit: org/bukkit/inventory/FurnaceInventory.setResult
    pub fn set_result(&self, stack: Option<ItemStack>) {
        self.0.set_item(2, stack);
    }
    /// bukkit: org/bukkit/inventory/FurnaceInventory.isFuel
    pub fn is_fuel(&self, stack: &ItemStack) -> bool {
        ItemType(stack.get_type().key().to_owned()).is_fuel()
    }
    /// bukkit: org/bukkit/inventory/FurnaceInventory.canSmelt
    pub fn can_smelt(&self, stack: &ItemStack) -> bool {
        flag(&query(json!({ "op": "items_can_smelt", "block": self.0.block.get_type().key(), "stack": stack.to_json() })))
    }
    /// bukkit: org/bukkit/inventory/FurnaceInventory.getHolder
    pub fn get_holder(&self) -> blocks::Furnace {
        self.0.block.tile_state()
    }
}

/// `org.bukkit.inventory.BrewerInventory`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrewerInventory(pub BlockInventory);

impl BrewerInventory {
    /// bukkit: org/bukkit/inventory/BrewerInventory.getIngredient
    pub fn get_ingredient(&self) -> Option<ItemStack> {
        self.0.get_item(3)
    }
    /// bukkit: org/bukkit/inventory/BrewerInventory.setIngredient
    pub fn set_ingredient(&self, stack: Option<ItemStack>) {
        self.0.set_item(3, stack);
    }
    /// bukkit: org/bukkit/inventory/BrewerInventory.getFuel
    pub fn get_fuel(&self) -> Option<ItemStack> {
        self.0.get_item(4)
    }
    /// bukkit: org/bukkit/inventory/BrewerInventory.setFuel
    pub fn set_fuel(&self, stack: Option<ItemStack>) {
        self.0.set_item(4, stack);
    }
    /// bukkit: org/bukkit/inventory/BrewerInventory.getHolder
    pub fn get_holder(&self) -> blocks::BrewingStand {
        self.0.block.tile_state()
    }
}

/// `org.bukkit.inventory.JukeboxInventory`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JukeboxInventory(pub Block);

impl JukeboxInventory {
    /// bukkit: org/bukkit/inventory/JukeboxInventory.getRecord
    pub fn get_record(&self) -> Option<ItemStack> {
        ItemStack::read(&block_data_of(&self.0)["RecordItem"])
    }
    /// bukkit: org/bukkit/inventory/JukeboxInventory.setRecord
    pub fn set_record(&self, stack: &ItemStack) {
        block_merge(&self.0, &format!("{{RecordItem:{}}}", stack.to_snbt()));
    }
    /// bukkit: org/bukkit/inventory/JukeboxInventory.getHolder
    pub fn get_holder(&self) -> blocks::Jukebox {
        self.0.tile_state()
    }
}

impl blocks::Jukebox {
    /// bukkit: org/bukkit/block/Jukebox.getInventory, org/bukkit/block/Jukebox.getSnapshotInventory
    pub fn get_inventory(&self) -> JukeboxInventory {
        JukeboxInventory(self.0.get_block().clone())
    }
    /// bukkit: org/bukkit/block/Jukebox.hasRecord
    pub fn has_record(&self) -> bool {
        !self.0.saved("RecordItem").is_null()
    }
    /// bukkit: org/bukkit/block/Jukebox.getPlaying
    pub fn get_playing(&self) -> Material {
        self.get_record().map_or_else(|| Material::of("AIR"), |record| record.get_type().clone())
    }
    /// bukkit: org/bukkit/block/Jukebox.isPlaying
    pub fn is_playing(&self) -> bool {
        flag(&self.0.get_block().asked("items_jukebox", json!({}))["playing"])
    }
}

/// `org.bukkit.inventory.LecternInventory`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LecternInventory(pub Block);

impl LecternInventory {
    /// bukkit: org/bukkit/inventory/LecternInventory.getBook
    pub fn get_book(&self) -> Option<ItemStack> {
        ItemStack::read(&block_data_of(&self.0)["Book"])
    }
    /// bukkit: org/bukkit/inventory/LecternInventory.setBook
    pub fn set_book(&self, stack: &ItemStack) {
        block_merge(&self.0, &format!("{{Book:{}}}", stack.to_snbt()));
    }
    /// bukkit: org/bukkit/inventory/LecternInventory.getHolder
    pub fn get_holder(&self) -> blocks::Lectern {
        self.0.tile_state()
    }
}

/// `org.bukkit.block.Chest`: `Container.getInventory` is the chest with its other half.
#[derive(Clone, Debug, PartialEq)]
pub struct Chest(pub TileState);

impl StateClass for Chest {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl Chest {
    /// bukkit: org/bukkit/block/Chest.getBlockInventory
    pub fn get_block_inventory(&self) -> BlockInventory {
        self.0.inventory()
    }
    /// bukkit: org/bukkit/block/DoubleChest.getInventory
    pub fn get_double_chest(&self) -> Option<DoubleChest> {
        DoubleChest::of(self.0.get_block())
    }
    /// bukkit: org/bukkit/block/Chest.isBlocked, org/bukkit/block/EnderChest.isBlocked
    pub fn is_blocked(&self) -> bool {
        let above = self.0.get_block().get_relative(0, 1, 0);
        flag(&above.state_facts()["occluding"])
    }
}

/// `org.bukkit.block.DoubleChest` and `org.bukkit.inventory.DoubleChestInventory`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoubleChest {
    /// The half the game counts first: its `type` is `right`.
    left: Block,
    right: Block,
}

impl DoubleChest {
    /// bukkit: org/bukkit/inventory/DoubleChestInventory.getHolder
    ///
    /// The two halves, for a block of a double chest.
    pub fn of(block: &Block) -> Option<Self> {
        let data = block.block_data();
        let kind = data.property("type")?;
        let turn = |facing: &str, clockwise: bool| -> Option<(i32, i32)> {
            Some(match (facing, clockwise) {
                ("north", true) | ("south", false) => (1, 0),
                ("south", true) | ("north", false) => (-1, 0),
                ("east", true) | ("west", false) => (0, 1),
                ("west", true) | ("east", false) => (0, -1),
                _ => return None,
            })
        };
        // ChestBlock.getConnectedDirection: the other half stands beside this one.
        let (x, z) = turn(data.property("facing")?, kind == "left")?;
        let other = block.get_relative(x, 0, z);
        match kind {
            "right" => Some(Self { left: block.clone(), right: other }),
            "left" => Some(Self { left: other, right: block.clone() }),
            _ => None,
        }
    }
    /// bukkit: org/bukkit/block/DoubleChest.getLeftSide, org/bukkit/inventory/DoubleChestInventory.getLeftSide
    pub fn get_left_side(&self) -> BlockInventory {
        BlockInventory::of(&self.left)
    }
    /// bukkit: org/bukkit/block/DoubleChest.getRightSide, org/bukkit/inventory/DoubleChestInventory.getRightSide
    pub fn get_right_side(&self) -> BlockInventory {
        BlockInventory::of(&self.right)
    }
    /// bukkit: org/bukkit/block/DoubleChest.getInventory
    ///
    /// All 54 slots, the left half's first, as the game's `CompoundContainer` numbers them.
    pub fn get_contents(&self) -> Vec<Option<ItemStack>> {
        let mut stacks = self.get_left_side().get_contents();
        stacks.extend(self.get_right_side().get_contents());
        stacks
    }
    /// bukkit: org/bukkit/block/DoubleChest.getLocation
    pub fn get_location(&self) -> Location {
        let (a, b) = (self.left.get_location(), self.right.get_location());
        Location::new(self.left.get_world().clone(), f64::midpoint(a.x, b.x), a.y, f64::midpoint(a.z, b.z), 0.0, 0.0)
    }
    /// bukkit: org/bukkit/block/DoubleChest.getX
    pub fn get_x(&self) -> f64 {
        self.get_location().x
    }
    /// bukkit: org/bukkit/block/DoubleChest.getY
    pub fn get_y(&self) -> f64 {
        self.get_location().y
    }
    /// bukkit: org/bukkit/block/DoubleChest.getZ
    pub fn get_z(&self) -> f64 {
        self.get_location().z
    }
    /// bukkit: org/bukkit/block/DoubleChest.getWorld
    pub fn get_world(&self) -> &World {
        self.left.get_world()
    }
}

fn dye_of(block: &Block, suffix: &str) -> Option<String> {
    let name = block.get_type().name();
    name.strip_suffix(suffix).map(str::to_owned)
}

/// `org.bukkit.block.ShulkerBox`.
#[derive(Clone, Debug, PartialEq)]
pub struct ShulkerBox(pub TileState);

impl StateClass for ShulkerBox {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl ShulkerBox {
    /// bukkit: org/bukkit/block/ShulkerBox.getColor
    pub fn get_color(&self) -> Option<String> {
        dye_of(self.0.get_block(), "_SHULKER_BOX")
    }
}

/// `org.bukkit.block.Bed`.
#[derive(Clone, Debug, PartialEq)]
pub struct Bed(pub TileState);

impl StateClass for Bed {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl Bed {
    /// bukkit: org/bukkit/block/Bed.getColor
    pub fn get_color(&self) -> Option<String> {
        dye_of(self.0.get_block(), "_BED")
    }
    /// bukkit: org/bukkit/block/Bed.setColor
    ///
    /// # Panics
    /// Always, as Java throws: a bed's colour is its block type.
    pub fn set_color(&self, _color: &str) {
        panic!("Must set block type to appropriate bed colour");
    }
}

/// `org.bukkit.block.Skull`.
#[derive(Clone, Debug, PartialEq)]
pub struct Skull(pub TileState);

impl StateClass for Skull {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl Skull {
    /// bukkit: org/bukkit/block/Skull.hasOwner
    pub fn has_owner(&self) -> bool {
        !self.0.saved("profile").is_null()
    }
    /// bukkit: org/bukkit/block/Skull.getOwner
    pub fn get_owner(&self) -> Option<String> {
        self.0.saved("profile.name").as_str().map(str::to_owned)
    }
    fn write_profile(&mut self, id: Option<&str>, name: Option<&str>) {
        let mut fields = vec!["properties:[]".to_owned()];
        if let Some(name) = name {
            fields.push(format!("name:{}", crate::data::quoted(name)));
        }
        if let Some(words) = id.and_then(crate::data::uuid_snbt) {
            fields.push(format!("id:{words}"));
        }
        self.0.change("profile", format!("{{{}}}", fields.join(",")));
    }
    /// bukkit: org/bukkit/block/Skull.setOwner
    pub fn set_owner(&mut self, name: &str) -> bool {
        self.write_profile(None, Some(name));
        true
    }
    /// bukkit: org/bukkit/block/Skull.setOwningPlayer
    pub fn set_owning_player(&mut self, player: &crate::OfflinePlayer) {
        self.write_profile(player.get_unique_id(), Some(player.get_name()));
    }
    /// bukkit: org/bukkit/block/Skull.getOwningPlayer
    pub fn get_owning_player(&self) -> Option<crate::OfflinePlayer> {
        self.get_owner().map(|name| crate::get_offline_player(&name))
    }
    /// bukkit: org/bukkit/block/Skull.setPlayerProfile, org/bukkit/block/Skull.setOwnerProfile, org/bukkit/block/Skull.setProfile
    pub fn set_player_profile(&mut self, id: Option<&str>, name: Option<&str>) {
        self.write_profile(id, name);
    }
    /// bukkit: org/bukkit/block/Skull.getPlayerProfile, org/bukkit/block/Skull.getOwnerProfile, org/bukkit/block/Skull.getProfile
    pub fn get_player_profile(&self) -> Option<crate::player::PlayerProfile> {
        let profile = self.0.saved("profile");
        if profile.is_null() {
            return None;
        }
        let id = crate::data::uuid(&profile["id"]).unwrap_or_default();
        Some(crate::extra::create_player_profile(&id, profile["name"].as_str().unwrap_or_default()))
    }
    /// bukkit: org/bukkit/block/Skull.getNoteBlockSound
    pub fn get_note_block_sound(&self) -> Option<String> {
        self.0.saved("note_block_sound").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Skull.setNoteBlockSound
    pub fn set_note_block_sound(&mut self, sound: &str) {
        self.0.change("note_block_sound", crate::data::quoted(sound));
    }
    /// bukkit: org/bukkit/block/Skull.customName
    pub fn custom_name(&self) -> Option<crate::Component> {
        let name = self.0.saved("custom_name");
        (!name.is_null()).then(|| crate::Component(name))
    }
    /// bukkit: org/bukkit/block/Skull.getSkullType
    pub fn get_skull_type(&self) -> &'static str {
        let name = self.0.get_block().get_type().name();
        match name.split("_WALL").next().unwrap_or_default().trim_end_matches("_HEAD").trim_end_matches("_SKULL") {
            "SKELETON" => "SKELETON",
            "WITHER_SKELETON" => "WITHER",
            "ZOMBIE" => "ZOMBIE",
            "PLAYER" => "PLAYER",
            "CREEPER" => "CREEPER",
            "DRAGON" => "DRAGON",
            _ => "PIGLIN",
        }
    }
    /// bukkit: org/bukkit/block/Skull.setSkullType
    ///
    /// # Panics
    /// Always, as Java throws: the block's type is the skull's.
    pub fn set_skull_type(&self, _kind: &str) {
        panic!("change the block's type instead");
    }
    /// bukkit: org/bukkit/block/Skull.getRotation
    pub fn get_rotation(&self) -> &'static str {
        let data = self.0.get_block().block_data();
        match (data.property("rotation").and_then(|step| step.parse::<usize>().ok()), data.property("facing")) {
            (Some(step), _) => ROTATIONS[step % 16],
            (None, Some("north")) => "NORTH",
            (None, Some("east")) => "EAST",
            (None, Some("south")) => "SOUTH",
            (None, Some("west")) => "WEST",
            _ => "SELF",
        }
    }
    /// bukkit: org/bukkit/block/Skull.setRotation
    ///
    /// On the block's data, at once, for a head standing on the ground.
    pub fn set_rotation(&self, face: &str) {
        let block = self.0.get_block();
        let mut data = block.block_data();
        if let Some(step) = ROTATIONS.iter().position(|known| *known == face) {
            data.set_property("rotation", &step.to_string());
            block.set_data(&data);
        }
    }
}

/// `org.bukkit.block.Bell`.
#[derive(Clone, Debug, PartialEq)]
pub struct Bell(pub TileState);

impl StateClass for Bell {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl Bell {
    fn live(&self) -> Value {
        self.0.get_block().asked("items_bell", json!({}))
    }
    /// bukkit: org/bukkit/block/Bell.isShaking
    pub fn is_shaking(&self) -> bool {
        flag(&self.live()["shaking"])
    }
    /// bukkit: org/bukkit/block/Bell.getShakingTicks
    pub fn get_shaking_ticks(&self) -> i32 {
        self.live()["ticks"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Bell.isResonating
    pub fn is_resonating(&self) -> bool {
        flag(&self.live()["resonating"])
    }
    /// bukkit: org/bukkit/block/Bell.getResonatingTicks
    pub fn get_resonating_ticks(&self) -> i32 {
        self.live()["resonation_ticks"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Bell.ring
    ///
    /// From `face`, or the bell's facing; `entity` is who rang it, for the game event.
    pub fn ring(&self, face: Option<&str>, entity: Option<&crate::entity::Entity>) -> bool {
        flag(&self.0.get_block().asked("items_bell_ring", json!({ "face": face, "entity": entity.map(crate::entity::Entity::get_entity_id) })))
    }
}

/// `org.bukkit.block.Campfire`.
#[derive(Clone, Debug, PartialEq)]
pub struct Campfire(pub TileState);

impl StateClass for Campfire {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl Campfire {
    fn numbers(&self, key: &str) -> [i32; 4] {
        let saved = self.0.saved(key);
        std::array::from_fn(|slot| saved[slot].as_i64().unwrap_or(0) as i32)
    }
    fn write_numbers(&mut self, key: &str, slot: usize, value: i32) {
        let mut numbers = self.numbers(key);
        if let Some(place) = numbers.get_mut(slot) {
            *place = value;
        }
        self.0.change(key, format!("[I;{},{},{},{}]", numbers[0], numbers[1], numbers[2], numbers[3]));
    }
    /// bukkit: org/bukkit/block/Campfire.getSize
    pub fn get_size(&self) -> usize {
        4
    }
    /// bukkit: org/bukkit/block/Campfire.getItem
    pub fn get_item(&self, slot: usize) -> Option<ItemStack> {
        items_of(&self.0.saved("Items"), 4).get(slot).cloned().flatten()
    }
    /// bukkit: org/bukkit/block/Campfire.setItem
    pub fn set_item(&mut self, slot: usize, stack: Option<ItemStack>) {
        let mut stacks = items_of(&self.0.saved("Items"), 4);
        if let Some(place) = stacks.get_mut(slot) {
            *place = stack;
        }
        self.0.change("Items", items_snbt(&stacks));
    }
    /// bukkit: org/bukkit/block/Campfire.getCookTime
    pub fn get_cook_time(&self, slot: usize) -> i32 {
        self.numbers("CookingTimes").get(slot).copied().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/Campfire.setCookTime
    pub fn set_cook_time(&mut self, slot: usize, ticks: i32) {
        self.write_numbers("CookingTimes", slot, ticks);
    }
    /// bukkit: org/bukkit/block/Campfire.getCookTimeTotal
    pub fn get_cook_time_total(&self, slot: usize) -> i32 {
        self.numbers("CookingTotalTimes").get(slot).copied().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/Campfire.setCookTimeTotal
    pub fn set_cook_time_total(&mut self, slot: usize, ticks: i32) {
        self.write_numbers("CookingTotalTimes", slot, ticks);
    }
}

/// `org.bukkit.block.ChiseledBookshelf`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChiseledBookshelf(pub TileState);

impl StateClass for ChiseledBookshelf {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl ChiseledBookshelf {
    /// bukkit: org/bukkit/block/ChiseledBookshelf.getInventory, org/bukkit/inventory/ChiseledBookshelfInventory.getHolder
    pub fn get_inventory(&self) -> BlockInventory {
        self.0.inventory()
    }
    /// bukkit: org/bukkit/block/ChiseledBookshelf.getSnapshotInventory
    pub fn get_snapshot_inventory(&mut self) -> SnapshotInventory<'_> {
        SnapshotInventory { tile: &mut self.0, size: 6 }
    }
    /// bukkit: org/bukkit/block/ChiseledBookshelf.getLastInteractedSlot
    pub fn get_last_interacted_slot(&self) -> i32 {
        self.0.saved("last_interacted_slot").as_i64().unwrap_or(-1) as i32
    }
    /// bukkit: org/bukkit/block/ChiseledBookshelf.setLastInteractedSlot
    pub fn set_last_interacted_slot(&mut self, slot: i32) {
        self.0.change("last_interacted_slot", slot.to_string());
    }
    /// bukkit: org/bukkit/block/ChiseledBookshelf.getSlot
    ///
    /// The slot a point on the block's front falls on, `position` being within the block; -1 off the front.
    pub fn get_slot(&self, position: &Vector) -> i32 {
        self.0.get_block().asked("items_bookshelf_slot", json!({ "at": [position.x, position.y, position.z] })).as_i64().unwrap_or(-1) as i32
    }
}

/// `org.bukkit.block.DecoratedPot`. Its saved data lives with the chunk, not with the block entities
/// `/data` reaches: `Block::tile_state` has nothing for it.
#[derive(Clone, Debug, PartialEq)]
pub struct DecoratedPot(pub Block);

const POT_SIDES: [(&str, &str); 4] = [("BACK", "back"), ("LEFT", "left"), ("RIGHT", "right"), ("FRONT", "front")];

impl DecoratedPot {
    fn saved(&self) -> Value {
        self.0.asked("items_saved", json!({}))
    }
    /// bukkit: org/bukkit/block/DecoratedPot.getSherd
    ///
    /// Bricks where nothing is set, as a new pot has.
    pub fn get_sherd(&self, side: &str) -> Material {
        let key = POT_SIDES.iter().find(|(java, _)| *java == side).map_or("front", |(_, saved)| saved);
        Material::of(self.saved()["sherds"][key].as_str().unwrap_or("minecraft:brick"))
    }
    /// bukkit: org/bukkit/block/DecoratedPot.getSherds
    pub fn get_sherds(&self) -> BTreeMap<&'static str, Material> {
        POT_SIDES.iter().map(|(java, _)| (*java, self.get_sherd(java))).collect()
    }
    /// bukkit: org/bukkit/block/DecoratedPot.getShards
    ///
    /// Back, left, right, front.
    pub fn get_shards(&self) -> Vec<Material> {
        POT_SIDES.iter().map(|(java, _)| self.get_sherd(java)).collect()
    }
    /// bukkit: org/bukkit/block/DecoratedPot.setSherd
    pub fn set_sherd(&self, side: &str, sherd: &Material) -> bool {
        POT_SIDES.iter().find(|(java, _)| *java == side).is_some_and(|(_, saved)| flag(&self.0.asked("items_saved_merge", json!({ "snbt": format!("{{sherds:{{{saved}:{}}}}}", crate::data::quoted(sherd.key())) }))))
    }
    /// bukkit: org/bukkit/block/DecoratedPot.getInventory, org/bukkit/block/DecoratedPot.getSnapshotInventory, org/bukkit/inventory/DecoratedPotInventory.getHolder
    pub fn get_inventory(&self) -> DecoratedPotInventory {
        DecoratedPotInventory(self.0.clone())
    }
    /// bukkit: org/bukkit/block/DecoratedPot.startWobble
    ///
    /// `POSITIVE` or `NEGATIVE`.
    pub fn start_wobble(&self, style: &str) -> bool {
        flag(&self.0.asked("items_pot_wobble", json!({ "style": style })))
    }
}

/// `org.bukkit.inventory.DecoratedPotInventory`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecoratedPotInventory(pub Block);

impl DecoratedPotInventory {
    /// bukkit: org/bukkit/inventory/DecoratedPotInventory.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.0.asked("items_saved", json!({}))["item"])
    }
    /// bukkit: org/bukkit/inventory/DecoratedPotInventory.setItem
    pub fn set_item(&self, stack: &ItemStack) -> bool {
        flag(&self.0.asked("items_saved_merge", json!({ "snbt": format!("{{item:{}}}", stack.to_snbt()) })))
    }
}

/// `org.bukkit.block.Shelf`.
#[derive(Clone, Debug, PartialEq)]
pub struct Shelf(pub TileState);

impl StateClass for Shelf {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl Shelf {
    /// bukkit: org/bukkit/block/Shelf.getInventory, org/bukkit/inventory/ShelfInventory.getHolder
    pub fn get_inventory(&self) -> BlockInventory {
        self.0.inventory()
    }
    /// bukkit: org/bukkit/block/Shelf.getSnapshotInventory
    pub fn get_snapshot_inventory(&mut self) -> SnapshotInventory<'_> {
        SnapshotInventory { tile: &mut self.0, size: 3 }
    }
}

/// `org.bukkit.block.SculkShrieker`.
#[derive(Clone, Debug, PartialEq)]
pub struct SculkShrieker(pub TileState);

impl StateClass for SculkShrieker {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl SculkShrieker {
    /// bukkit: org/bukkit/block/SculkShrieker.getWarningLevel
    pub fn get_warning_level(&self) -> i32 {
        self.0.saved("warning_level").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/SculkShrieker.setWarningLevel
    pub fn set_warning_level(&mut self, level: i32) {
        self.0.change("warning_level", level.to_string());
    }
}

impl blocks::SculkSensor {
    /// bukkit: org/bukkit/block/SculkSensor.getListenerRange
    ///
    /// The game's range, 8 for a sensor and 16 for a calibrated one, or the one `set_listener_range` gave.
    pub fn get_listener_range(&self) -> i32 {
        self.0.extra("listener_range").as_i64().map_or_else(|| if self.0.get_type().key().contains("calibrated") { 16 } else { 8 }, |range| range as i32)
    }
}

/// `org.bukkit.block.EntityBlockStorage`, for a beehive: the bees in its saved data.
impl blocks::Beehive {
    /// bukkit: org/bukkit/block/EntityBlockStorage.getEntityCount
    pub fn get_entity_count(&self) -> usize {
        self.0.saved("bees").as_array().map_or(0, Vec::len)
    }
    /// bukkit: org/bukkit/block/EntityBlockStorage.getMaxEntities
    ///
    /// `BeehiveBlockEntity.MAX_OCCUPANTS`, or the limit `set_max_entities` gave.
    pub fn get_max_entities(&self) -> usize {
        self.0.extra("max_bees").as_u64().map_or(3, |limit| limit as usize)
    }
    /// bukkit: org/bukkit/block/EntityBlockStorage.isFull
    pub fn is_full(&self) -> bool {
        self.get_entity_count() >= self.get_max_entities()
    }
    /// bukkit: org/bukkit/block/EntityBlockStorage.clearEntities
    pub fn clear_entities(&mut self) {
        self.0.change("bees", "[]".to_owned());
    }
    /// bukkit: org/bukkit/block/Beehive.isSedated
    ///
    /// Smoke from a campfire within five blocks below, as the game's bees read it.
    pub fn is_sedated(&self) -> bool {
        flag(&self.0.get_block().asked("items_hive_sedated", json!({})))
    }
}

impl blocks::Banner {
    /// bukkit: org/bukkit/block/Banner.removePattern
    pub fn remove_pattern(&mut self, index: usize) -> Option<crate::meta_lists::Pattern> {
        let mut patterns = self.get_patterns();
        let removed = (index < patterns.len()).then(|| patterns.remove(index));
        self.set_patterns(&patterns);
        removed
    }
    /// bukkit: org/bukkit/block/Banner.setPattern
    pub fn set_pattern(&mut self, index: usize, pattern: crate::meta_lists::Pattern) {
        let mut patterns = self.get_patterns();
        if let Some(place) = patterns.get_mut(index) {
            *place = pattern;
        }
        self.set_patterns(&patterns);
    }
    /// bukkit: org/bukkit/block/Banner.setBaseColor
    ///
    /// On this state only, as in Java: a banner's colour in the world is its block type, and only a
    /// shield's pseudo banner goes by this.
    pub fn set_base_color(&mut self, color: &str) {
        self.0.data["BaseColor"] = json!(color.to_ascii_uppercase());
    }
}

impl blocks::Beacon {
    /// bukkit: org/bukkit/block/Beacon.getPrimaryEffect
    pub fn get_primary_effect(&self) -> Option<crate::PotionEffect> {
        self.beacon_effect(self.get_primary_effect_id(), true)
    }
    /// bukkit: org/bukkit/block/Beacon.getSecondaryEffect
    pub fn get_secondary_effect(&self) -> Option<crate::PotionEffect> {
        self.beacon_effect(self.get_secondary_effect_id(), false)
    }
    /// The effect at the pyramid's strength: II for a primary power that's also the secondary.
    fn beacon_effect(&self, id: Option<String>, primary: bool) -> Option<crate::PotionEffect> {
        let id = id?;
        let doubled = primary && self.get_tier() >= 4 && self.get_secondary_effect_id().as_deref() == Some(id.as_str());
        Some(crate::PotionEffect::new(&id, (9 + self.get_tier() * 2) * 20, i32::from(doubled), true, true))
    }
    /// bukkit: org/bukkit/block/Beacon.getEffectRange
    ///
    /// `BeaconBlockEntity.applyEffects`: 10 blocks a level over 10, or the range `set_effect_range` gave.
    pub fn get_effect_range(&self) -> f64 {
        self.0.extra("effect_range").as_f64().unwrap_or_else(|| f64::from(self.get_tier() * 10 + 10))
    }
    /// bukkit: org/bukkit/block/Beacon.getEntitiesInRange
    ///
    /// The players the beacon reaches: in a box of the range around it, down to the bottom and up to the height of the world.
    pub fn get_entities_in_range(&self) -> Vec<Player> {
        let block = self.0.get_block();
        let range = self.get_effect_range();
        let (x, z) = (f64::from(block.get_x()) + 0.5, f64::from(block.get_z()) + 0.5);
        block.get_world().get_players().into_iter().filter(|player| {
            let at = player.get_location();
            (at.x - x).abs() <= range + 0.5 && (at.z - z).abs() <= range + 0.5
        }).collect()
    }
}

impl blocks::BrewingStand {
    /// bukkit: org/bukkit/block/BrewingStand.getRecipeBrewTime
    pub fn get_recipe_brew_time(&self) -> i32 {
        self.0.saved("total_brew_time").as_i64().unwrap_or(400) as i32
    }
    /// bukkit: org/bukkit/block/BrewingStand.setRecipeBrewTime
    pub fn set_recipe_brew_time(&mut self, ticks: i32) {
        self.0.change("total_brew_time", ticks.to_string());
    }
}

impl blocks::CommandBlock {
    /// bukkit: org/bukkit/block/CommandBlock.name
    pub fn name(&self) -> Option<crate::Component> {
        let name = self.0.saved("CustomName");
        (!name.is_null()).then(|| crate::Component(name))
    }
    /// bukkit: org/bukkit/block/CommandBlock.getName
    pub fn get_name(&self) -> String {
        self.name().map_or_else(|| "@".to_owned(), |name| name.to_legacy())
    }
    /// bukkit: org/bukkit/block/CommandBlock.setName
    pub fn set_name(&mut self, name: &str) {
        self.0.change("CustomName", crate::data::snbt(&crate::adventure::legacy(name).0));
    }
}

impl blocks::Crafter {
    /// bukkit: org/bukkit/block/Crafter.isSlotDisabled
    pub fn is_slot_disabled(&self, slot: i32) -> bool {
        self.0.saved("disabled_slots").as_array().is_some_and(|slots| slots.iter().any(|known| known.as_i64() == Some(i64::from(slot))))
    }
    /// bukkit: org/bukkit/block/Crafter.setSlotDisabled
    pub fn set_slot_disabled(&mut self, slot: i32, disabled: bool) {
        let mut slots: Vec<i64> = self.0.saved("disabled_slots").as_array().into_iter().flatten().filter_map(Value::as_i64).filter(|known| *known != i64::from(slot)).collect();
        if disabled {
            slots.push(i64::from(slot));
        }
        slots.sort_unstable();
        let saved: Vec<String> = slots.iter().map(i64::to_string).collect();
        self.0.change("disabled_slots", format!("[I;{}]", saved.join(",")));
    }
}

impl blocks::Furnace {
    /// bukkit: org/bukkit/block/Furnace.getCookSpeedMultiplier
    pub fn get_cook_speed_multiplier(&self) -> f64 {
        self.0.saved("speed_multiplier").as_f64().unwrap_or(1.0)
    }
    /// bukkit: org/bukkit/block/Furnace.setCookSpeedMultiplier
    pub fn set_cook_speed_multiplier(&mut self, multiplier: f64) {
        self.0.change("speed_multiplier", format!("{multiplier:?}f"));
    }
    /// bukkit: org/bukkit/block/Furnace.getRecipesUsed
    ///
    /// Recipe ids with how many times each was cooked.
    pub fn get_recipes_used(&self) -> BTreeMap<String, i32> {
        self.0.saved("RecipesUsed").as_object().into_iter().flatten().map(|(id, count)| (id.clone(), count.as_i64().unwrap_or(0) as i32)).collect()
    }
    /// bukkit: org/bukkit/block/Furnace.getRecipeUsedCount
    pub fn get_recipe_used_count(&self, recipe: &NamespacedKey) -> i32 {
        self.get_recipes_used().get(recipe.as_str()).copied().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/Furnace.hasRecipeUsedCount
    pub fn has_recipe_used_count(&self, recipe: &NamespacedKey) -> bool {
        self.get_recipes_used().contains_key(recipe.as_str())
    }
    /// bukkit: org/bukkit/block/Furnace.setRecipeUsedCount
    ///
    /// A count of 0 or less takes the recipe out.
    pub fn set_recipe_used_count(&mut self, recipe: &NamespacedKey, count: i32) {
        let mut used = self.get_recipes_used();
        if count > 0 {
            used.insert(recipe.as_str().to_owned(), count);
        } else {
            used.remove(recipe.as_str());
        }
        self.set_recipes_used(&used);
    }
    /// bukkit: org/bukkit/block/Furnace.setRecipesUsed
    pub fn set_recipes_used(&mut self, used: &BTreeMap<String, i32>) {
        let rows: Vec<String> = used.iter().map(|(id, count)| format!("{}:{count}", crate::data::quoted(id))).collect();
        self.0.change("=RecipesUsed", format!("{{{}}}", rows.join(",")));
    }
}

const STRUCTURE_SIDES: [&str; 3] = ["X", "Y", "Z"];

impl blocks::Structure {
    fn triple(&self, prefix: &str) -> Vector {
        let value = |axis: &str| self.0.saved(&format!("{prefix}{axis}")).as_f64().unwrap_or(0.0);
        Vector::new(value("X"), value("Y"), value("Z"))
    }
    fn write_triple(&mut self, prefix: &str, value: &Vector) {
        for (axis, number) in STRUCTURE_SIDES.into_iter().zip([value.x, value.y, value.z]) {
            self.0.change(&format!("{prefix}{axis}"), (number as i32).to_string());
        }
    }
    /// bukkit: org/bukkit/block/Structure.getRelativePosition
    pub fn get_relative_position(&self) -> Vector {
        self.triple("pos")
    }
    /// bukkit: org/bukkit/block/Structure.setRelativePosition
    pub fn set_relative_position(&mut self, position: &Vector) {
        self.write_triple("pos", position);
    }
    /// bukkit: org/bukkit/block/Structure.getStructureSize
    pub fn get_structure_size(&self) -> Vector {
        self.triple("size")
    }
    /// bukkit: org/bukkit/block/Structure.setStructureSize
    pub fn set_structure_size(&mut self, size: &Vector) {
        self.write_triple("size", size);
    }
    /// bukkit: org/bukkit/block/Structure.getMirror
    pub fn get_mirror(&self) -> String {
        text(&self.0.saved("mirror"))
    }
    /// bukkit: org/bukkit/block/Structure.setMirror
    pub fn set_mirror(&mut self, mirror: &str) {
        self.0.change("mirror", crate::data::quoted(mirror));
    }
    /// bukkit: org/bukkit/block/Structure.getRotation
    pub fn get_rotation(&self) -> String {
        text(&self.0.saved("rotation"))
    }
    /// bukkit: org/bukkit/block/Structure.setRotation
    pub fn set_rotation(&mut self, rotation: &str) {
        self.0.change("rotation", crate::data::quoted(rotation));
    }
    /// bukkit: org/bukkit/block/Structure.getUsageMode
    pub fn get_usage_mode(&self) -> String {
        text(&self.0.saved("mode"))
    }
    /// bukkit: org/bukkit/block/Structure.setUsageMode
    pub fn set_usage_mode(&mut self, mode: &str) {
        self.0.change("mode", crate::data::quoted(mode));
    }
}

impl blocks::Vault {
    fn uuids(&self, path: &str) -> Vec<String> {
        self.0.saved(path).as_array().into_iter().flatten().filter_map(crate::data::uuid).collect()
    }
    /// bukkit: org/bukkit/block/Vault.getRewardedPlayers
    pub fn get_rewarded_players(&self) -> Vec<String> {
        self.uuids("server_data.rewarded_players")
    }
    /// bukkit: org/bukkit/block/Vault.hasRewardedPlayer
    pub fn has_rewarded_player(&self, uuid: &str) -> bool {
        self.get_rewarded_players().iter().any(|known| known == uuid)
    }
    /// bukkit: org/bukkit/block/Vault.addRewardedPlayer
    pub fn add_rewarded_player(&mut self, uuid: &str) -> bool {
        self.change_rewarded(uuid, true)
    }
    /// bukkit: org/bukkit/block/Vault.removeRewardedPlayer
    pub fn remove_rewarded_player(&mut self, uuid: &str) -> bool {
        self.change_rewarded(uuid, false)
    }
    fn change_rewarded(&mut self, uuid: &str, add: bool) -> bool {
        let mut players = self.get_rewarded_players();
        let had = players.iter().any(|known| known == uuid);
        players.retain(|known| known != uuid);
        if add {
            players.push(uuid.to_owned());
        }
        let saved: Vec<String> = players.iter().filter_map(|id| crate::data::uuid_snbt(id)).collect();
        self.0.change("server_data.rewarded_players", format!("[{}]", saved.join(",")));
        had != add
    }
    /// bukkit: org/bukkit/block/Vault.getConnectedPlayers
    pub fn get_connected_players(&self) -> Vec<String> {
        self.uuids("shared_data.connected_players")
    }
    /// bukkit: org/bukkit/block/Vault.hasConnectedPlayer
    pub fn has_connected_player(&self, uuid: &str) -> bool {
        self.get_connected_players().iter().any(|known| known == uuid)
    }
    /// bukkit: org/bukkit/block/Vault.getLootTable
    pub fn get_loot_table(&self) -> Option<String> {
        self.0.saved("config.loot_table").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Vault.setLootTable
    pub fn set_loot_table(&mut self, table: &str) {
        self.0.change("config.loot_table", crate::data::quoted(table));
    }
    /// bukkit: org/bukkit/block/Vault.getDisplayedLootTable
    pub fn get_displayed_loot_table(&self) -> Option<String> {
        self.0.saved("config.override_loot_table_to_display").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Vault.setDisplayedLootTable
    pub fn set_displayed_loot_table(&mut self, table: &str) {
        self.0.change("config.override_loot_table_to_display", crate::data::quoted(table));
    }
}

impl blocks::TrialSpawner {
    /// bukkit: org/bukkit/block/TrialSpawner.isOminous
    pub fn is_ominous(&self) -> bool {
        self.0.get_block().block_data().property("ominous") == Some("true")
    }
    /// bukkit: org/bukkit/block/TrialSpawner.setOminous
    pub fn set_ominous(&self, ominous: bool) {
        let block = self.0.get_block();
        let mut data = block.block_data();
        set_flag(&mut data, "ominous", ominous);
        block.set_data(&data);
    }
    fn tracked(&self, key: &str) -> Vec<String> {
        self.0.saved(key).as_array().into_iter().flatten().filter_map(crate::data::uuid).collect()
    }
    /// bukkit: org/bukkit/block/TrialSpawner.getTrackedPlayers
    pub fn get_tracked_players(&self) -> Vec<String> {
        self.tracked("registered_players")
    }
    /// bukkit: org/bukkit/block/TrialSpawner.isTrackingPlayer
    pub fn is_tracking_player(&self, uuid: &str) -> bool {
        self.get_tracked_players().iter().any(|known| known == uuid)
    }
    /// bukkit: org/bukkit/block/TrialSpawner.getTrackedEntities
    pub fn get_tracked_entities(&self) -> Vec<String> {
        self.tracked("current_mobs")
    }
    /// bukkit: org/bukkit/block/TrialSpawner.isTrackingEntity
    pub fn is_tracking_entity(&self, uuid: &str) -> bool {
        self.get_tracked_entities().iter().any(|known| known == uuid)
    }
    fn change_tracked(&mut self, key: &str, uuid: &str, add: bool) {
        let mut known = self.tracked(key);
        known.retain(|id| id != uuid);
        if add {
            known.push(uuid.to_owned());
        }
        let saved: Vec<String> = known.iter().filter_map(|id| crate::data::uuid_snbt(id)).collect();
        self.0.change(key, format!("[{}]", saved.join(",")));
    }
    /// bukkit: org/bukkit/block/TrialSpawner.startTrackingPlayer
    pub fn start_tracking_player(&mut self, uuid: &str) {
        self.change_tracked("registered_players", uuid, true);
    }
    /// bukkit: org/bukkit/block/TrialSpawner.stopTrackingPlayer
    pub fn stop_tracking_player(&mut self, uuid: &str) {
        self.change_tracked("registered_players", uuid, false);
    }
    /// bukkit: org/bukkit/block/TrialSpawner.startTrackingEntity
    pub fn start_tracking_entity(&mut self, uuid: &str) {
        self.change_tracked("current_mobs", uuid, true);
    }
    /// bukkit: org/bukkit/block/TrialSpawner.stopTrackingEntity
    pub fn stop_tracking_entity(&mut self, uuid: &str) {
        self.change_tracked("current_mobs", uuid, false);
    }
}

/// `org.bukkit.block.Sign`'s two sides: `front_text` and `back_text`.
impl blocks::Sign {
    /// bukkit: org/bukkit/block/Sign.getSide
    pub fn get_side(&mut self, side: &str) -> SignSide<'_> {
        let path = if side == "BACK" { "back_text" } else { "front_text" };
        SignSide { sign: self, path }
    }
    /// bukkit: org/bukkit/block/Sign.isEditable
    pub fn is_editable(&self) -> bool {
        !self.is_waxed()
    }
    /// bukkit: org/bukkit/block/Sign.getColor
    pub fn get_color(&self) -> String {
        self.0.saved("front_text.color").as_str().unwrap_or("black").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/Sign.setColor
    pub fn set_color(&mut self, color: &str) {
        self.0.change("front_text.color", crate::data::quoted(&color.to_ascii_lowercase()));
    }
}

/// `org.bukkit.block.sign.SignSide`.
pub struct SignSide<'a> {
    sign: &'a mut blocks::Sign,
    path: &'static str,
}

impl SignSide<'_> {
    /// bukkit: org/bukkit/block/sign/SignSide.isGlowingText
    pub fn is_glowing_text(&self) -> bool {
        crate::data::flag(&self.sign.0.saved(&format!("{}.has_glowing_text", self.path)))
    }
    /// bukkit: org/bukkit/block/sign/SignSide.setGlowingText
    pub fn set_glowing_text(&mut self, glowing: bool) {
        self.sign.0.change(&format!("{}.has_glowing_text", self.path), glowing.to_string());
    }
    /// bukkit: org/bukkit/block/sign/SignSide.line
    pub fn line(&self, index: usize) -> Option<crate::Component> {
        let lines = self.sign.0.saved(&format!("{}.messages", self.path));
        lines.get(index).cloned().map(crate::Component)
    }
    /// The side's color, by Bukkit's name.
    pub fn get_color(&self) -> String {
        self.sign.0.saved(&format!("{}.color", self.path)).as_str().unwrap_or("black").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/sign/SignSide.setLine
    ///
    /// The side's line, as legacy text.
    pub fn set_line(&mut self, index: usize, line: &str) {
        let mut lines: Vec<crate::Component> = self.sign.0.saved(&format!("{}.messages", self.path)).as_array().into_iter().flatten().cloned().map(crate::Component).collect();
        lines.resize(4, crate::Component::empty());
        if let Some(place) = lines.get_mut(index) {
            *place = crate::adventure::legacy(line);
        }
        let saved: Vec<String> = lines.iter().map(|line| crate::data::snbt(&line.0)).collect();
        self.sign.0.change(&format!("{}.messages", self.path), format!("[{}]", saved.join(",")));
    }
}

/// `org.bukkit.block.Dispenser`.
#[derive(Clone, Debug, PartialEq)]
pub struct Dispenser(pub TileState);

impl StateClass for Dispenser {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl Dispenser {
    /// bukkit: org/bukkit/block/Dispenser.dispense, org/bukkit/block/Dropper.drop
    pub fn dispense(&self) -> bool {
        flag(&self.0.get_block().asked("items_dispense", json!({})))
    }
}

/// `org.bukkit.block.CreatureSpawner`: the old name of what it spawns.
impl crate::pure::Spawner {
    /// bukkit: org/bukkit/block/CreatureSpawner.getCreatureTypeName
    pub fn get_creature_type_name(&self) -> Option<String> {
        block_data_of(&self.0)["SpawnData"]["entity"]["id"].as_str().map(|id| id.trim_start_matches("minecraft:").to_owned())
    }
    /// bukkit: org/bukkit/block/CreatureSpawner.setCreatureTypeByName
    pub fn set_creature_type_by_name(&self, name: &str) -> bool {
        block_merge(&self.0, &format!("{{SpawnData:{{entity:{{id:{}}}}},SpawnPotentials:[]}}", crate::data::quoted(&Material::of(name).key().to_owned())))
    }
}

/// bukkit: org/bukkit/block/Biome.translationKey
pub fn biome_translation_key(biome: &str) -> String {
    format!("biome.{}", biome.replace(':', "."))
}

// ---- Potions ----

/// `org.bukkit.potion.PotionType`, by the game's id: `minecraft:long_swiftness`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PotionType(pub String);

impl PotionType {
    /// A potion by its id or Bukkit's name.
    pub fn of(name: &str) -> Self {
        Self(Material::of(name).key().to_owned())
    }
    fn info(&self) -> Value {
        query(json!({ "op": "items_potion", "id": self.0 }))
    }
    /// bukkit: org/bukkit/potion/PotionType.getKey
    pub fn get_key(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(&self.0)
    }
    /// bukkit: org/bukkit/potion/PotionType.getPotionEffects
    pub fn get_potion_effects(&self) -> Vec<crate::PotionEffect> {
        potion_effects(&self.0)
    }
    /// bukkit: org/bukkit/potion/PotionType.getEffectType
    pub fn get_effect_type(&self) -> Option<String> {
        self.get_potion_effects().first().map(|effect| effect.get_type().to_owned())
    }
    /// bukkit: org/bukkit/potion/PotionType.isInstant
    pub fn is_instant(&self) -> bool {
        self.get_effect_type().is_some_and(|effect| flag(&query(json!({ "op": "items_effect", "id": effect }))["instant"]))
    }
    /// bukkit: org/bukkit/potion/PotionType.isUpgradeable
    pub fn is_upgradeable(&self) -> bool {
        flag(&self.info()["upgradeable"])
    }
    /// bukkit: org/bukkit/potion/PotionType.isExtendable
    pub fn is_extendable(&self) -> bool {
        flag(&self.info()["extendable"])
    }
    /// bukkit: org/bukkit/potion/PotionType.getMaxLevel
    pub fn get_max_level(&self) -> i32 {
        if self.is_upgradeable() { 2 } else { 1 }
    }
    /// bukkit: org/bukkit/potion/PotionType.getByEffect
    pub fn get_by_effect(effect: &str) -> Option<Self> {
        query(json!({ "op": "items_potion_by_effect", "id": effect })).as_str().map(Self::of)
    }
}

/// `org.bukkit.potion.PotionEffectType`, by the game's id: `minecraft:speed`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PotionEffectType(pub String);

impl PotionEffectType {
    /// An effect by its id.
    pub fn of(id: &str) -> Self {
        Self(Material::of(id).key().to_owned())
    }
    fn info(&self) -> Value {
        query(json!({ "op": "items_effect", "id": self.0 }))
    }
    /// bukkit: org/bukkit/potion/PotionEffectType.getById
    ///
    /// Bukkit's numbers: the registry's order from 1.
    pub fn get_by_id(id: i32) -> Option<Self> {
        query(json!({ "op": "items_effect_by_number", "id": id })).as_str().map(Self::of)
    }
    /// bukkit: org/bukkit/potion/PotionEffectType.getId
    pub fn get_id(&self) -> i32 {
        self.info()["number"].as_i64().unwrap_or(-1) as i32
    }
    /// bukkit: org/bukkit/potion/PotionEffectType.isInstant
    pub fn is_instant(&self) -> bool {
        flag(&self.info()["instant"])
    }
    /// bukkit: org/bukkit/potion/PotionEffectType.getColor
    pub fn get_color(&self) -> Color {
        let rgb = self.info()["color"].as_i64().unwrap_or(0);
        Color::from_rgb(((rgb >> 16) & 255) as u8, ((rgb >> 8) & 255) as u8, (rgb & 255) as u8)
    }
    /// bukkit: org/bukkit/potion/PotionEffectType.getCategory, org/bukkit/potion/PotionEffectType.getEffectCategory
    ///
    /// The name of `PotionEffectTypeCategory`: `BENEFICIAL`, `HARMFUL` or `NEUTRAL`.
    pub fn get_category(&self) -> String {
        text(&self.info()["category"])
    }
}

/// bukkit: org/bukkit/potion/PotionEffectType$Category.getColor
///
/// The text colour of a category's effects in a tooltip.
pub fn potion_category_color(category: &str) -> &'static str {
    match category {
        "BENEFICIAL" => "blue",
        "HARMFUL" => "red",
        _ => "blue",
    }
}

impl crate::PotionEffect {
    /// bukkit: org/bukkit/potion/PotionEffect.apply
    pub fn apply(&self, target: &crate::entity::Entity) -> bool {
        flag(&query(json!({ "op": "items_effect_add", "entity": target.get_entity_id(), "uuid": target.get_unique_id().to_string().replace('-', ""), "dimension": target.get_world().dimension(), "id": self.get_type(), "duration": self.get_duration(), "amplifier": self.get_amplifier(), "ambient": self.is_ambient(), "particles": self.has_particles() })))
    }
    /// bukkit: org/bukkit/potion/PotionEffect.serialize
    pub fn serialize(&self) -> BTreeMap<String, Value> {
        BTreeMap::from([("effect".to_owned(), json!(self.get_type())), ("duration".to_owned(), json!(self.get_duration())), ("amplifier".to_owned(), json!(self.get_amplifier())), ("ambient".to_owned(), json!(self.is_ambient())), ("has-particles".to_owned(), json!(self.has_particles())), ("has-icon".to_owned(), json!(self.has_icon()))])
    }
    /// bukkit: org/bukkit/potion/PotionEffect.getColor
    pub fn get_color(&self) -> Color {
        PotionEffectType::of(self.get_type()).get_color()
    }
}

/// `org.bukkit.potion.PotionBrewer`'s reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PotionBrewer;

impl PotionBrewer {
    /// bukkit: org/bukkit/potion/PotionBrewer.createEffect
    pub fn create_effect(&self, effect: &str, duration: i32, amplifier: i32) -> crate::PotionEffect {
        crate::registry::create_effect(effect, duration, amplifier)
    }
    /// bukkit: org/bukkit/potion/PotionBrewer.getEffectsFromDamage
    ///
    /// Non-functional in Java too: nothing.
    pub fn get_effects_from_damage(&self, _damage: i32) -> Vec<crate::PotionEffect> {
        Vec::new()
    }
    /// bukkit: org/bukkit/potion/PotionBrewer.getEffects
    pub fn get_effects(&self, potion: &PotionType, upgraded: bool, extended: bool) -> Vec<crate::PotionEffect> {
        let name = potion.0.trim_start_matches("minecraft:").trim_start_matches("long_").trim_start_matches("strong_").to_owned();
        let id = if upgraded { format!("minecraft:strong_{name}") } else if extended { format!("minecraft:long_{name}") } else { format!("minecraft:{name}") };
        potion_effects(&id)
    }
}

// ---- Open menus ----

/// bukkit: org/bukkit/inventory/InventoryView$Property.getId
pub fn inventory_property_id(property: &str) -> i32 {
    match property {
        "BREW_TIME" | "BURN_TIME" | "ENCHANT_BUTTON1" | "LEVELS" | "REPAIR_COST" | "BOOK_PAGE" | "STONECUTTER_RECIPE" | "LOOM_PATTERN" => 0,
        "FUEL_TIME" | "TICKS_FOR_CURRENT_FUEL" | "ENCHANT_BUTTON2" | "PRIMARY_EFFECT" => 1,
        "COOK_TIME" | "ENCHANT_BUTTON3" | "SECONDARY_EFFECT" => 2,
        "TICKS_FOR_CURRENT_SMELTING" | "ENCHANT_XP_SEED" => 3,
        "ENCHANT_ID1" => 4,
        "ENCHANT_ID2" => 5,
        "ENCHANT_ID3" => 6,
        "ENCHANT_LEVEL1" => 7,
        "ENCHANT_LEVEL2" => 8,
        "ENCHANT_LEVEL3" => 9,
        _ => -1,
    }
}

/// bukkit: org/bukkit/inventory/InventoryView$Property.getType
pub fn inventory_property_type(property: &str) -> &'static str {
    match property {
        "BREW_TIME" | "FUEL_TIME" => "BREWING",
        "BURN_TIME" | "TICKS_FOR_CURRENT_FUEL" | "COOK_TIME" | "TICKS_FOR_CURRENT_SMELTING" => "FURNACE",
        "LEVELS" | "PRIMARY_EFFECT" | "SECONDARY_EFFECT" => "BEACON",
        "REPAIR_COST" => "ANVIL",
        "BOOK_PAGE" => "LECTERN",
        "STONECUTTER_RECIPE" => "STONECUTTER",
        "LOOM_PATTERN" => "LOOM",
        _ => "ENCHANTING",
    }
}

/// Every plugin hears the hooks that keep menus readable: see `family/items.py`.
pub(crate) fn install(host: horizon_mod::Host) {
    for hook in ["items_station", "items_stations_tick", "items_station_closed", "items_lectern", "items_table", "items_tables_tick", "items_table_closed", "items_merchant", "items_merchant_closed", "items_campfire", "items_sculk"] {
        host.on(hook, |_| horizon_mod::Answer::Pass);
    }
    host.on("items_gui_changed", gui_changed);
    // These answer: the engine's side worked out the refusal, and only a listener's answer can give it.
    for hook in ["items_lock", "items_lock_station", "items_hive_enter", "items_beacon"] {
        host.on(hook, |value| if value["refused"] == true { horizon_mod::Answer::Fail } else { horizon_mod::Answer::Pass });
    }
}

/// What a player has open, as the game runs it: a workstation, a crafting table, a merchant's screen, a
/// container, or only their own 2x2 grid. A write is done the next time the game sends the menu, within
/// the tick, and later reads in the same event see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenMenu {
    player: Player,
}

impl OpenMenu {
    /// The menu `player` has open now.
    pub fn of(player: &Player) -> Self {
        Self { player: player.clone() }
    }
    /// All the game says of it: `Value::Null` for a player who isn't here.
    pub fn state(&self) -> Value {
        query(json!({ "op": "items_menu", "uuid": self.player.uuid() }))
    }
    /// The engine's menu kind: `Anvil`, `Crafting`, `Merchant`, `Block`...
    pub fn kind(&self) -> String {
        text(&self.state()["kind"])
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    fn slot(&self, slot: usize) -> Option<ItemStack> {
        ItemStack::read(&self.state()["slots"][slot])
    }
    fn set_slot(&self, slot: usize, stack: Option<&ItemStack>) {
        query(json!({ "op": "items_menu_set", "uuid": self.player.uuid(), "slot": slot, "stack": stack.map(ItemStack::to_json) }));
    }
    fn data(&self, index: usize) -> i32 {
        self.state()["data"][index].as_i64().unwrap_or(0) as i32
    }
    fn set_data(&self, index: usize, value: i32) {
        query(json!({ "op": "items_menu_set", "uuid": self.player.uuid(), "data": index, "value": value }));
    }
    /// The block of a block's menu: a furnace's, a beacon's.
    fn block(&self) -> Option<Block> {
        let state = self.state();
        let at = |i: usize| state["pos"][i].as_i64().map(|value| value as i32);
        Some(self.player.get_world().get_block_at(at(0)?, at(1)?, at(2)?))
    }
}

macro_rules! menu_views {
    ($($name:ident: $java:literal),* $(,)?) => {
        $(
            #[doc = concat!("`", $java, "`.")]
            #[derive(Clone, Debug, PartialEq, Eq)]
            pub struct $name(pub OpenMenu);

            impl $name {
                /// The menu `player` has open, seen as this class: Java's cast.
                pub fn of(player: &Player) -> Self {
                    Self(OpenMenu::of(player))
                }
            }
        )*
    };
}

menu_views!(
    CraftingInventory: "org.bukkit.inventory.CraftingInventory",
    AnvilInventory: "org.bukkit.inventory.AnvilInventory",
    SmithingInventory: "org.bukkit.inventory.SmithingInventory",
    GrindstoneInventory: "org.bukkit.inventory.GrindstoneInventory",
    StonecutterInventory: "org.bukkit.inventory.StonecutterInventory",
    EnchantingInventory: "org.bukkit.inventory.EnchantingInventory",
    BeaconInventory: "org.bukkit.inventory.BeaconInventory",
    CartographyInventory: "org.bukkit.inventory.CartographyInventory",
    MerchantInventory: "org.bukkit.inventory.MerchantInventory",
    AnvilView: "org.bukkit.inventory.view.AnvilView",
    BeaconView: "org.bukkit.inventory.view.BeaconView",
    BrewingStandView: "org.bukkit.inventory.view.BrewingStandView",
    CrafterView: "org.bukkit.inventory.view.CrafterView",
    EnchantmentView: "org.bukkit.inventory.view.EnchantmentView",
    FurnaceView: "org.bukkit.inventory.view.FurnaceView",
    LecternView: "org.bukkit.inventory.view.LecternView",
    LoomView: "org.bukkit.inventory.view.LoomView",
    MerchantView: "org.bukkit.inventory.view.MerchantView",
    StonecutterView: "org.bukkit.inventory.view.StonecutterView",
);

impl CraftingInventory {
    /// The grid's first slot: a crafting table's nine, or the inventory's four, follow the result.
    fn grid(&self) -> Vec<usize> {
        let slots = self.0.state()["slots"].as_array().map_or(0, Vec::len);
        (1..slots).collect()
    }
    /// bukkit: org/bukkit/inventory/CraftingInventory.getMatrix
    pub fn get_matrix(&self) -> Vec<Option<ItemStack>> {
        self.grid().into_iter().map(|slot| self.0.slot(slot)).collect()
    }
    /// bukkit: org/bukkit/inventory/CraftingInventory.setMatrix
    pub fn set_matrix(&self, stacks: &[Option<ItemStack>]) {
        for (index, slot) in self.grid().into_iter().enumerate() {
            self.0.set_slot(slot, stacks.get(index).and_then(Option::as_ref));
        }
    }
    /// bukkit: org/bukkit/inventory/CraftingInventory.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.0.slot(0)
    }
    /// bukkit: org/bukkit/inventory/CraftingInventory.setResult
    pub fn set_result(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(0, stack);
    }
}

impl AnvilInventory {
    /// bukkit: org/bukkit/inventory/AnvilInventory.getFirstItem
    pub fn get_first_item(&self) -> Option<ItemStack> {
        self.0.slot(0)
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.setFirstItem
    pub fn set_first_item(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(0, stack);
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.getSecondItem
    pub fn get_second_item(&self) -> Option<ItemStack> {
        self.0.slot(1)
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.setSecondItem
    pub fn set_second_item(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(1, stack);
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.0.slot(2)
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.setResult
    pub fn set_result(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(2, stack);
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.getRenameText, org/bukkit/inventory/view/AnvilView.getRenameText
    pub fn get_rename_text(&self) -> Option<String> {
        self.0.state()["rename"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.getRepairCost, org/bukkit/inventory/view/AnvilView.getRepairCost
    pub fn get_repair_cost(&self) -> i32 {
        self.0.data(0)
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.setRepairCost, org/bukkit/inventory/view/AnvilView.setRepairCost
    pub fn set_repair_cost(&self, levels: i32) {
        self.0.set_data(0, levels);
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.getRepairCostAmount, org/bukkit/inventory/view/AnvilView.getRepairItemCountCost
    pub fn get_repair_cost_amount(&self) -> i32 {
        self.0.state()["repair_count"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.setRepairCostAmount, org/bukkit/inventory/view/AnvilView.setRepairItemCountCost
    pub fn set_repair_cost_amount(&self, amount: i32) {
        query(json!({ "op": "items_menu_set", "uuid": self.0.player.uuid(), "repair_count": amount }));
    }
    /// bukkit: org/bukkit/inventory/AnvilInventory.getMaximumRepairCost, org/bukkit/inventory/view/AnvilView.getMaximumRepairCost
    ///
    /// The game's own limit, 40, past which only a creative player may take the result, or a plugin's.
    pub fn get_maximum_repair_cost(&self) -> i32 {
        query(json!({ "op": "items_anvil_state", "uuid": self.0.player.uuid() }))["max"].as_i64().unwrap_or(40) as i32
    }
}

impl AnvilView {
    /// bukkit: org/bukkit/inventory/view/AnvilView.getTopInventory
    pub fn get_top_inventory(&self) -> AnvilInventory {
        AnvilInventory(self.0.clone())
    }
}

impl SmithingInventory {
    /// bukkit: org/bukkit/inventory/SmithingInventory.getInputTemplate
    pub fn get_input_template(&self) -> Option<ItemStack> {
        self.0.slot(0)
    }
    /// bukkit: org/bukkit/inventory/SmithingInventory.setInputTemplate
    pub fn set_input_template(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(0, stack);
    }
    /// bukkit: org/bukkit/inventory/SmithingInventory.getInputEquipment
    pub fn get_input_equipment(&self) -> Option<ItemStack> {
        self.0.slot(1)
    }
    /// bukkit: org/bukkit/inventory/SmithingInventory.setInputEquipment
    pub fn set_input_equipment(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(1, stack);
    }
    /// bukkit: org/bukkit/inventory/SmithingInventory.getInputMineral
    pub fn get_input_mineral(&self) -> Option<ItemStack> {
        self.0.slot(2)
    }
    /// bukkit: org/bukkit/inventory/SmithingInventory.setInputMineral
    pub fn set_input_mineral(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(2, stack);
    }
    /// bukkit: org/bukkit/inventory/SmithingInventory.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.0.slot(3)
    }
    /// bukkit: org/bukkit/inventory/SmithingInventory.setResult
    pub fn set_result(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(3, stack);
    }
}

impl GrindstoneInventory {
    /// bukkit: org/bukkit/inventory/GrindstoneInventory.getUpperItem
    pub fn get_upper_item(&self) -> Option<ItemStack> {
        self.0.slot(0)
    }
    /// bukkit: org/bukkit/inventory/GrindstoneInventory.setUpperItem
    pub fn set_upper_item(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(0, stack);
    }
    /// bukkit: org/bukkit/inventory/GrindstoneInventory.getLowerItem
    pub fn get_lower_item(&self) -> Option<ItemStack> {
        self.0.slot(1)
    }
    /// bukkit: org/bukkit/inventory/GrindstoneInventory.setLowerItem
    pub fn set_lower_item(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(1, stack);
    }
    /// bukkit: org/bukkit/inventory/GrindstoneInventory.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.0.slot(2)
    }
    /// bukkit: org/bukkit/inventory/GrindstoneInventory.setResult
    pub fn set_result(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(2, stack);
    }
}

impl StonecutterInventory {
    /// bukkit: org/bukkit/inventory/StonecutterInventory.getInputItem
    pub fn get_input_item(&self) -> Option<ItemStack> {
        self.0.slot(0)
    }
    /// bukkit: org/bukkit/inventory/StonecutterInventory.setInputItem
    pub fn set_input_item(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(0, stack);
    }
    /// bukkit: org/bukkit/inventory/StonecutterInventory.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.0.slot(1)
    }
    /// bukkit: org/bukkit/inventory/StonecutterInventory.setResult
    pub fn set_result(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(1, stack);
    }
}

impl StonecutterView {
    /// bukkit: org/bukkit/inventory/view/StonecutterView.getTopInventory
    pub fn get_top_inventory(&self) -> StonecutterInventory {
        StonecutterInventory(self.0.clone())
    }
    /// bukkit: org/bukkit/inventory/view/StonecutterView.getSelectedRecipeIndex
    pub fn get_selected_recipe_index(&self) -> i32 {
        self.0.data(0)
    }
    /// bukkit: org/bukkit/inventory/view/StonecutterView.getRecipeAmount
    pub fn get_recipe_amount(&self) -> usize {
        self.0.state()["recipes"].as_u64().unwrap_or(0) as usize
    }
}

impl LoomView {
    /// bukkit: org/bukkit/inventory/view/LoomView.getSelectedPatternIndex
    pub fn get_selected_pattern_index(&self) -> i32 {
        self.0.data(0)
    }
    /// bukkit: org/bukkit/inventory/view/LoomView.getSelectablePatterns
    ///
    /// The patterns by id.
    pub fn get_selectable_patterns(&self) -> Vec<String> {
        self.0.state()["patterns"].as_array().into_iter().flatten().filter_map(|id| id.as_str().map(str::to_owned)).collect()
    }
}

impl EnchantingInventory {
    /// bukkit: org/bukkit/inventory/EnchantingInventory.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        self.0.slot(0)
    }
    /// bukkit: org/bukkit/inventory/EnchantingInventory.setItem
    pub fn set_item(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(0, stack);
    }
    /// bukkit: org/bukkit/inventory/EnchantingInventory.getSecondary
    pub fn get_secondary(&self) -> Option<ItemStack> {
        self.0.slot(1)
    }
    /// bukkit: org/bukkit/inventory/EnchantingInventory.setSecondary
    pub fn set_secondary(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(1, stack);
    }
}

impl EnchantmentView {
    /// bukkit: org/bukkit/inventory/view/EnchantmentView.getTopInventory
    pub fn get_top_inventory(&self) -> EnchantingInventory {
        EnchantingInventory(self.0.clone())
    }
    /// bukkit: org/bukkit/inventory/view/EnchantmentView.getEnchantmentSeed
    pub fn get_enchantment_seed(&self) -> i32 {
        self.0.data(3)
    }
    /// bukkit: org/bukkit/inventory/view/EnchantmentView.getOffers
    ///
    /// The three offers, each its enchantment's key with its level and level cost; `None` for an empty one.
    pub fn get_offers(&self) -> [Option<(String, i32, i32)>; 3] {
        let state = self.0.state();
        std::array::from_fn(|index| {
            let cost = state["data"][index].as_i64()? as i32;
            let name = state["names"][index].as_str()?;
            (cost > 0).then(|| (name.to_owned(), state["data"][7 + index].as_i64().unwrap_or(0) as i32, cost))
        })
    }
}

impl BeaconInventory {
    /// bukkit: org/bukkit/inventory/BeaconInventory.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        self.0.slot(0)
    }
    /// bukkit: org/bukkit/inventory/BeaconInventory.setItem
    pub fn set_item(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(0, stack);
    }
}

impl BeaconView {
    /// bukkit: org/bukkit/inventory/view/BeaconView.getTopInventory
    pub fn get_top_inventory(&self) -> BeaconInventory {
        BeaconInventory(self.0.clone())
    }
    /// bukkit: org/bukkit/inventory/view/BeaconView.getTier
    pub fn get_tier(&self) -> i32 {
        self.0.data(0)
    }
    /// bukkit: org/bukkit/inventory/view/BeaconView.getPrimaryEffect
    pub fn get_primary_effect(&self) -> Option<String> {
        self.0.state()["names"][0].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/view/BeaconView.getSecondaryEffect
    pub fn get_secondary_effect(&self) -> Option<String> {
        self.0.state()["names"][1].as_str().map(str::to_owned)
    }
    fn beacon(&self) -> Option<blocks::Beacon> {
        Some(self.0.block()?.tile_state())
    }
    /// bukkit: org/bukkit/inventory/view/BeaconView.setPrimaryEffect
    ///
    /// On the block, whose powers the menu follows.
    pub fn set_primary_effect(&self, effect: &str) {
        if let Some(mut beacon) = self.beacon() {
            beacon.set_primary_effect(effect);
            beacon.update();
        }
    }
    /// bukkit: org/bukkit/inventory/view/BeaconView.setSecondaryEffect
    pub fn set_secondary_effect(&self, effect: &str) {
        if let Some(mut beacon) = self.beacon() {
            beacon.set_secondary_effect(effect);
            beacon.update();
        }
    }
}

impl CartographyInventory {
    /// bukkit: org/bukkit/inventory/CartographyInventory.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        self.0.slot(2)
    }
    /// bukkit: org/bukkit/inventory/CartographyInventory.setResult
    pub fn set_result(&self, stack: Option<&ItemStack>) {
        self.0.set_slot(2, stack);
    }
}

impl LecternView {
    fn lectern(&self) -> Option<blocks::Lectern> {
        Some(self.0.block()?.tile_state())
    }
    /// bukkit: org/bukkit/inventory/view/LecternView.getTopInventory
    pub fn get_top_inventory(&self) -> Option<LecternInventory> {
        self.0.block().map(LecternInventory)
    }
    /// bukkit: org/bukkit/inventory/view/LecternView.getPage
    pub fn get_page(&self) -> i32 {
        self.lectern().map_or(0, |lectern| lectern.get_page())
    }
    /// bukkit: org/bukkit/inventory/view/LecternView.setPage
    pub fn set_page(&self, page: i32) -> bool {
        self.lectern().is_some_and(|mut lectern| {
            lectern.set_page(page);
            lectern.update()
        })
    }
}

impl FurnaceView {
    fn furnace(&self) -> Option<blocks::Furnace> {
        Some(self.0.block()?.tile_state())
    }
    /// bukkit: org/bukkit/inventory/view/FurnaceView.getTopInventory
    pub fn get_top_inventory(&self) -> Option<FurnaceInventory> {
        self.0.block().map(|block| FurnaceInventory(BlockInventory::of(&block)))
    }
    /// bukkit: org/bukkit/inventory/view/FurnaceView.getCookTime
    ///
    /// Progress from 0 to 1.
    pub fn get_cook_time(&self) -> f32 {
        self.furnace().map_or(0.0, |furnace| {
            let total = furnace.get_cook_time_total();
            if total > 0 { (furnace.get_cook_time() as f32 / total as f32).clamp(0.0, 1.0) } else { 0.0 }
        })
    }
    /// bukkit: org/bukkit/inventory/view/FurnaceView.getBurnTime
    ///
    /// Fuel left from 0 to 1.
    pub fn get_burn_time(&self) -> f32 {
        self.furnace().map_or(0.0, |furnace| {
            let total = furnace.0.saved("lit_total_time").as_i64().unwrap_or(0) as f32;
            if total > 0.0 { (furnace.get_burn_time() as f32 / total).clamp(0.0, 1.0) } else { 0.0 }
        })
    }
    /// bukkit: org/bukkit/inventory/view/FurnaceView.isBurning
    pub fn is_burning(&self) -> bool {
        self.furnace().is_some_and(|furnace| furnace.get_burn_time() > 0)
    }
    /// bukkit: org/bukkit/inventory/view/FurnaceView.setCookTime
    pub fn set_cook_time(&self, spent: i32, total: i32) {
        if let Some(mut furnace) = self.furnace() {
            furnace.set_cook_time(spent);
            furnace.set_cook_time_total(total);
            furnace.update();
        }
    }
    /// bukkit: org/bukkit/inventory/view/FurnaceView.setBurnTime
    pub fn set_burn_time(&self, remaining: i32, total: i32) {
        if let Some(mut furnace) = self.furnace() {
            furnace.set_burn_time(remaining);
            furnace.0.change("lit_total_time", total.to_string());
            furnace.update();
        }
    }
}

impl BrewingStandView {
    fn stand(&self) -> Option<blocks::BrewingStand> {
        Some(self.0.block()?.tile_state())
    }
    /// bukkit: org/bukkit/inventory/view/BrewingStandView.getTopInventory
    pub fn get_top_inventory(&self) -> Option<BrewerInventory> {
        self.0.block().map(|block| BrewerInventory(BlockInventory::of(&block)))
    }
    /// bukkit: org/bukkit/inventory/view/BrewingStandView.getFuelLevel
    pub fn get_fuel_level(&self) -> i32 {
        self.stand().map_or(0, |stand| stand.get_fuel_level())
    }
    /// bukkit: org/bukkit/inventory/view/BrewingStandView.setFuelLevel
    pub fn set_fuel_level(&self, level: i32) {
        if let Some(mut stand) = self.stand() {
            stand.set_fuel_level(level);
            stand.update();
        }
    }
    /// bukkit: org/bukkit/inventory/view/BrewingStandView.getBrewingTicks
    pub fn get_brewing_ticks(&self) -> i32 {
        self.stand().map_or(0, |stand| stand.get_brewing_time())
    }
    /// bukkit: org/bukkit/inventory/view/BrewingStandView.setBrewingTicks
    pub fn set_brewing_ticks(&self, ticks: i32) {
        if let Some(mut stand) = self.stand() {
            stand.set_brewing_time(ticks);
            stand.update();
        }
    }
    /// bukkit: org/bukkit/inventory/view/BrewingStandView.getRecipeBrewTime
    pub fn get_recipe_brew_time(&self) -> i32 {
        self.stand().map_or(400, |stand| stand.get_recipe_brew_time())
    }
    /// bukkit: org/bukkit/inventory/view/BrewingStandView.setRecipeBrewTime
    pub fn set_recipe_brew_time(&self, ticks: i32) {
        if let Some(mut stand) = self.stand() {
            stand.set_recipe_brew_time(ticks);
            stand.update();
        }
    }
}

impl CrafterView {
    fn crafter(&self) -> Option<blocks::Crafter> {
        Some(self.0.block()?.tile_state())
    }
    /// bukkit: org/bukkit/inventory/view/CrafterView.isSlotDisabled
    pub fn is_slot_disabled(&self, slot: i32) -> bool {
        self.crafter().is_some_and(|crafter| crafter.is_slot_disabled(slot))
    }
    /// bukkit: org/bukkit/inventory/view/CrafterView.setSlotDisabled
    pub fn set_slot_disabled(&self, slot: i32, disabled: bool) {
        if let Some(mut crafter) = self.crafter() {
            crafter.set_slot_disabled(slot, disabled);
            crafter.update();
        }
    }
    /// bukkit: org/bukkit/inventory/view/CrafterView.isPowered
    pub fn is_powered(&self) -> bool {
        self.0.block().is_some_and(|block| block.block_data().property("triggered") == Some("true"))
    }
}

impl MerchantInventory {
    /// bukkit: org/bukkit/inventory/MerchantInventory.getSelectedRecipeIndex
    pub fn get_selected_recipe_index(&self) -> i32 {
        self.0.state()["selected"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/inventory/MerchantInventory.getSelectedRecipe
    pub fn get_selected_recipe(&self) -> Option<crate::core2::MerchantRecipe> {
        let state = self.0.state();
        let index = usize::try_from(state["selected"].as_i64()?).ok()?;
        state["offers"].get(index).cloned().map(crate::core2::MerchantRecipe)
    }
    /// bukkit: org/bukkit/inventory/MerchantInventory.getMerchant, org/bukkit/inventory/view/MerchantView.getMerchant
    pub fn get_merchant(&self) -> Option<Merchant> {
        let state = self.0.state();
        let id = state["mob"].as_u64()?;
        self.0.player.get_world().get_entities().into_iter().find(|entity| entity.get_entity_id() == id).map(Merchant)
    }
}

impl MerchantView {
    /// bukkit: org/bukkit/inventory/view/MerchantView.getTopInventory
    pub fn get_top_inventory(&self) -> MerchantInventory {
        MerchantInventory(self.0.clone())
    }
}

/// `org.bukkit.inventory.Merchant`: a villager or wandering trader, over its saved offers.
#[derive(Clone, Debug, PartialEq)]
pub struct Merchant(pub crate::entity::Entity);

impl Merchant {
    /// bukkit: org/bukkit/inventory/Merchant.getRecipes
    pub fn get_recipes(&self) -> Vec<crate::core2::MerchantRecipe> {
        let flags = self.discount_flags();
        let row = |(index, mut row): (usize, Value)| {
            row["ignoreDiscounts"] = json!(flags[index].as_bool().unwrap_or(false));
            crate::core2::MerchantRecipe(row)
        };
        self.0.data_at("Offers.Recipes").as_array().into_iter().flatten().cloned().enumerate().map(row).collect()
    }
    fn discount_key(&self) -> Value {
        query(json!({ "op": "items_entity_key", "mob": self.0.get_entity_id() }))
    }
    fn discount_flags(&self) -> Value {
        query(json!({ "op": "items_extra", "key": self.discount_key() }))["ignore_discounts"].clone()
    }
    /// bukkit: org/bukkit/inventory/Merchant.getRecipeCount
    pub fn get_recipe_count(&self) -> usize {
        self.get_recipes().len()
    }
    /// bukkit: org/bukkit/inventory/Merchant.getRecipe
    pub fn get_recipe(&self, index: usize) -> Option<crate::core2::MerchantRecipe> {
        self.get_recipes().get(index).cloned()
    }
    /// bukkit: org/bukkit/inventory/Merchant.setRecipes
    pub fn set_recipes(&self, recipes: &[crate::core2::MerchantRecipe]) -> bool {
        let rows: Vec<String> = recipes.iter().map(|recipe| crate::data::snbt(&recipe_saved(&recipe.0))).collect();
        let flags: Vec<bool> = recipes.iter().map(crate::core2::MerchantRecipe::should_ignore_discounts).collect();
        query(json!({ "op": "items_extra_set", "key": self.discount_key(), "field": "ignore_discounts", "value": flags }));
        self.0.merge(&format!("{{Offers:{{Recipes:[{}]}}}}", rows.join(",")))
    }
    /// bukkit: org/bukkit/inventory/Merchant.setRecipe
    pub fn set_recipe(&self, index: usize, recipe: crate::core2::MerchantRecipe) -> bool {
        let mut recipes = self.get_recipes();
        let Some(place) = recipes.get_mut(index) else {
            return false;
        };
        *place = recipe;
        self.set_recipes(&recipes)
    }
    /// bukkit: org/bukkit/inventory/Merchant.getTrader
    ///
    /// The id of the player trading with it, for `get_player_by_uuid`.
    pub fn get_trader(&self) -> Option<String> {
        query(json!({ "op": "items_trader", "mob": self.0.get_entity_id() })).as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/Merchant.isTrading
    pub fn is_trading(&self) -> bool {
        self.get_trader().is_some()
    }
}

/// An offer's stacks as a saved offer writes them: `item` from the SDK's JSON is `id`.
fn recipe_saved(recipe: &Value) -> Value {
    let mut saved = recipe.clone();
    if let Some(row) = saved.as_object_mut() {
        row.remove("ignoreDiscounts");
    }
    for key in ["buy", "buyB", "sell"] {
        if let Some(stack) = saved.get(key).and_then(ItemStack::read) {
            saved[key] = stack.saved_row();
        }
    }
    saved
}

impl crate::core2::MerchantRecipe {
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getAdjustedIngredient1
    ///
    /// The first ingredient at the price the game would ask: `getModifiedCostCount`.
    pub fn get_adjusted_ingredient1(&self) -> Option<ItemStack> {
        let first = self.get_ingredients().into_iter().next()?;
        let base = first.get_amount();
        let demand = ((f64::from(base) * self.get_demand() as f64 * self.get_price_multiplier()).floor() as i32).max(0);
        let price = (base + demand + self.get_special_price() as i32).clamp(1, first.get_max_stack_size());
        Some(first.as_quantity(price))
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.adjust
    pub fn adjust(&self, stack: &mut ItemStack) {
        if let Some(adjusted) = self.get_adjusted_ingredient1() {
            stack.set_amount(adjusted.get_amount());
        }
    }
}

impl crate::inventory::InventoryView {
    fn menu(&self) -> OpenMenu {
        OpenMenu::of(self.get_player())
    }
    /// bukkit: org/bukkit/inventory/InventoryView.setCursor
    pub fn set_cursor(&self, stack: Option<&ItemStack>) -> bool {
        flag(&query(json!({ "op": "items_cursor", "uuid": self.get_player().uuid(), "stack": stack.map(ItemStack::to_json) })))
    }
    /// bukkit: org/bukkit/inventory/InventoryView.setTitle
    pub fn set_title(&self, title: &crate::Component) -> bool {
        flag(&query(json!({ "op": "items_menu_title", "uuid": self.get_player().uuid(), "title": title.0 })))
    }
    /// bukkit: org/bukkit/inventory/InventoryView.setProperty
    pub fn set_property(&self, property: &str, value: i32) -> bool {
        let id = inventory_property_id(property);
        id >= 0 && flag(&query(json!({ "op": "items_menu_property", "uuid": self.get_player().uuid(), "id": id, "value": value })))
    }
    /// bukkit: org/bukkit/inventory/InventoryView.setItem
    pub fn set_item(&self, raw_slot: usize, stack: Option<&ItemStack>) {
        let size = self.top_size();
        if raw_slot < size {
            if let Some(mut gui) = self.get_top_inventory().cloned() {
                gui.set_item(raw_slot, stack.cloned());
            } else {
                self.menu().set_slot(raw_slot, stack);
            }
        } else if let Some(stack) = stack {
            let below = raw_slot - size;
            self.get_player().get_inventory().set_item(if below < 27 { below as i32 + 9 } else { below as i32 - 27 }, stack);
        }
    }
    fn top_size(&self) -> usize {
        self.get_top_inventory().map_or_else(|| self.menu().state()["slots"].as_array().map_or(0, Vec::len), crate::Inventory::get_size)
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getSlotType
    pub fn get_slot_type(&self, raw_slot: i32) -> &'static str {
        let size = self.top_size() as i32;
        let menu = self.get_menu_type().unwrap_or_default();
        let own_grid = menu.is_empty() && self.get_top_inventory().is_none();
        if raw_slot < 0 {
            return "OUTSIDE";
        }
        if own_grid {
            return match raw_slot {
                0 => "RESULT",
                1..=4 => "CRAFTING",
                5..=8 => "ARMOR",
                36.. => "QUICKBAR",
                _ => "CONTAINER",
            };
        }
        if raw_slot < size {
            return match (menu.as_str(), raw_slot) {
                ("FURNACE" | "BLAST_FURNACE" | "SMOKER", 2) | ("CRAFTING", 0) | ("ANVIL" | "CARTOGRAPHY_TABLE" | "GRINDSTONE" | "MERCHANT", 2) | ("STONECUTTER", 1) | ("LOOM" | "SMITHING", 3) => "RESULT",
                ("FURNACE" | "BLAST_FURNACE" | "SMOKER", 1) | ("BREWING_STAND", 3) => "FUEL",
                ("FURNACE" | "BLAST_FURNACE" | "SMOKER" | "BREWING_STAND" | "ENCHANTMENT" | "CRAFTING" | "BEACON" | "ANVIL" | "CARTOGRAPHY_TABLE" | "GRINDSTONE" | "MERCHANT" | "STONECUTTER" | "LOOM" | "SMITHING", _) => "CRAFTING",
                _ => "CONTAINER",
            };
        }
        if menu == "CRAFTER_3X3" {
            return match raw_slot {
                45 => "RESULT",
                36.. => "QUICKBAR",
                _ => "CONTAINER",
            };
        }
        if raw_slot >= size + 27 { "QUICKBAR" } else { "CONTAINER" }
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getMenuType
    ///
    /// Bukkit's constant, `ANVIL`, `GENERIC_9X3`, `CRAFTING`; `None` for a player's own inventory, as Java's `null`.
    pub fn get_menu_type(&self) -> Option<String> {
        let name = query(json!({ "op": "items_menu_type", "uuid": self.get_player().uuid() }));
        name.as_str().map(|name| name.trim_start_matches("minecraft:").to_ascii_uppercase())
    }
}

/// bukkit: org/bukkit/inventory/EquipmentSlot.isHand
pub fn equipment_slot_is_hand(slot: &str) -> bool {
    matches!(slot, "HAND" | "OFF_HAND")
}

/// bukkit: org/bukkit/inventory/EquipmentSlot.isArmor
pub fn equipment_slot_is_armor(slot: &str) -> bool {
    matches!(slot, "HEAD" | "CHEST" | "LEGS" | "FEET")
}

/// bukkit: org/bukkit/inventory/EquipmentSlot.getOppositeHand
pub fn equipment_slot_opposite_hand(slot: &str) -> Option<&'static str> {
    match slot {
        "HAND" => Some("OFF_HAND"),
        "OFF_HAND" => Some("HAND"),
        _ => None,
    }
}

/// bukkit: org/bukkit/inventory/EquipmentSlot.getGroup
///
/// The game's group of a slot: `mainhand`, `offhand`, `head`...
pub fn equipment_slot_group(slot: &str) -> &'static str {
    match slot {
        "HAND" => "mainhand",
        "OFF_HAND" => "offhand",
        "HEAD" => "head",
        "CHEST" => "chest",
        "LEGS" => "legs",
        "FEET" => "feet",
        "BODY" => "body",
        "SADDLE" => "saddle",
        _ => "any",
    }
}

/// bukkit: org/bukkit/inventory/EquipmentSlotGroup.test
pub fn equipment_group_test(group: &str, slot: &str) -> bool {
    let name = equipment_slot_group(slot);
    match group {
        "any" => true,
        "hand" => matches!(name, "mainhand" | "offhand"),
        "armor" => matches!(name, "head" | "chest" | "legs" | "feet" | "body"),
        other => other == name,
    }
}

/// bukkit: org/bukkit/inventory/EquipmentSlotGroup.getExample
pub fn equipment_group_example(group: &str) -> &'static str {
    match group {
        "offhand" => "OFF_HAND",
        "feet" => "FEET",
        "legs" => "LEGS",
        "chest" | "armor" => "CHEST",
        "head" => "HEAD",
        "body" => "BODY",
        "saddle" => "SADDLE",
        _ => "HAND",
    }
}

impl crate::inventory::EntityEquipment {
    /// bukkit: org/bukkit/inventory/EntityEquipment.setArmorContents
    ///
    /// In Bukkit's order: boots, leggings, chestplate, helmet.
    pub fn set_armor_contents(&self, stacks: &[ItemStack; 4]) -> bool {
        ["FEET", "LEGS", "CHEST", "HEAD"].into_iter().zip(stacks).all(|(slot, stack)| self.set_item(slot, stack))
    }
}

impl crate::PlayerInventory {
    /// bukkit: org/bukkit/inventory/PlayerInventory.setExtraContents
    ///
    /// The one extra slot: the off hand.
    pub fn set_extra_contents(&self, stacks: &[Option<ItemStack>]) {
        if let Some(Some(stack)) = stacks.first() {
            self.set_item_in_off_hand(stack);
        } else {
            run_command(&format!("item replace entity {} weapon.offhand with minecraft:air", self.owner().get_name()));
        }
    }
}

/// bukkit: org/bukkit/inventory/CreativeCategory.translationKey
pub fn creative_category_translation_key(category: &str) -> String {
    format!("itemGroup.{}", category.to_ascii_lowercase())
}

/// bukkit: org/bukkit/inventory/ItemRarity.color
///
/// The name of the rarity's text colour.
pub fn item_rarity_color(rarity: &str) -> &'static str {
    match rarity {
        "UNCOMMON" => "yellow",
        "RARE" => "aqua",
        "EPIC" => "light_purple",
        _ => "white",
    }
}

// ---- Recipes' choices ----

/// `org.bukkit.inventory.RecipeChoice`: what a slot of a recipe takes.
#[derive(Clone, Debug, PartialEq)]
pub enum RecipeChoice {
    /// `EmptyRecipeChoice`: nothing fits but nothing.
    Empty,
    /// `RecipeChoice$MaterialChoice`, `RecipeChoice$ItemTypeChoice`.
    Materials(Vec<Material>),
    /// `RecipeChoice$ExactChoice`: stacks with the same components.
    Exact(Vec<ItemStack>),
}

impl RecipeChoice {
    /// bukkit: org/bukkit/inventory/RecipeChoice.empty
    pub fn empty() -> Self {
        Self::Empty
    }
    /// bukkit: org/bukkit/inventory/RecipeChoice.itemType
    pub fn item_type(items: &[ItemType]) -> Self {
        Self::Materials(items.iter().map(ItemType::as_material).collect())
    }
    /// bukkit: org/bukkit/inventory/RecipeChoice.exactChoice
    pub fn exact_choice(stacks: &[ItemStack]) -> Self {
        Self::Exact(stacks.to_vec())
    }
    /// bukkit: org/bukkit/inventory/RecipeChoice.test, org/bukkit/inventory/RecipeChoice$MaterialChoice.test, org/bukkit/inventory/RecipeChoice$ExactChoice.test, org/bukkit/inventory/EmptyRecipeChoice.test
    pub fn test(&self, stack: &ItemStack) -> bool {
        match self {
            Self::Empty => stack.is_empty(),
            Self::Materials(materials) => materials.contains(stack.get_type()),
            Self::Exact(stacks) => stacks.iter().any(|known| known.is_similar(stack)),
        }
    }
    /// bukkit: org/bukkit/inventory/RecipeChoice.getItemStack, org/bukkit/inventory/RecipeChoice$MaterialChoice.getItemStack, org/bukkit/inventory/RecipeChoice$ExactChoice.getItemStack, org/bukkit/inventory/EmptyRecipeChoice.getItemStack
    pub fn get_item_stack(&self) -> ItemStack {
        match self {
            Self::Empty => ItemStack::empty(),
            Self::Materials(materials) => materials.first().map_or_else(ItemStack::empty, |material| ItemStack::new(material.clone(), 1)),
            Self::Exact(stacks) => stacks.first().cloned().unwrap_or_else(ItemStack::empty),
        }
    }
    /// bukkit: org/bukkit/inventory/RecipeChoice.validate, org/bukkit/inventory/RecipeChoice$MaterialChoice.validate, org/bukkit/inventory/RecipeChoice$ExactChoice.validate, org/bukkit/inventory/EmptyRecipeChoice.validate
    ///
    /// # Panics
    /// As Java throws, for a choice with nothing in it where an ingredient must have something.
    pub fn validate(self, allow_empty: bool) -> Self {
        assert!(allow_empty || self != Self::Empty, "empty choice");
        self
    }
    /// bukkit: org/bukkit/inventory/RecipeChoice$MaterialChoice.getChoices, org/bukkit/inventory/RecipeChoice$ExactChoice.getChoices
    pub fn get_choices(&self) -> Vec<ItemStack> {
        match self {
            Self::Empty => Vec::new(),
            Self::Materials(materials) => materials.iter().map(|material| ItemStack::new(material.clone(), 1)).collect(),
            Self::Exact(stacks) => stacks.clone(),
        }
    }
}

/// The materials of a recipe's ingredient list, when `set_input_choice` gave one.
fn listed_input(extra: &Map<String, Value>) -> Option<RecipeChoice> {
    let ingredient = extra.get("ingredient")?;
    if let Some(rows) = ingredient["stacks"].as_array() {
        return Some(RecipeChoice::Exact(rows.iter().filter_map(|row| {
            let saved: BTreeMap<String, Value> = row.as_object()?.iter().map(|(key, value)| (key.clone(), value.clone())).collect();
            let mut stack = ItemStack::new(Material::of(saved.get("id")?.as_str()?), saved.get("count").and_then(Value::as_i64).unwrap_or(1) as i32);
            for (id, value) in saved.get("components").and_then(Value::as_object).into_iter().flatten() {
                stack.set_component(id, value.clone());
            }
            Some(stack)
        }).collect()));
    }
    Some(RecipeChoice::Materials(ingredient.as_array()?.iter().filter_map(Value::as_str).map(Material::of).collect()))
}

impl RecipeChoice {
    /// The choice as a recipe file has it: items, or stacks by item and components.
    fn ingredient(&self) -> Value {
        if let Self::Exact(stacks) = self {
            return json!({ "type": "horizon:exact", "stacks": stacks.iter().map(ItemStack::saved_row).collect::<Vec<_>>() });
        }
        Value::Array(self.get_choices().iter().map(|stack| json!(stack.get_type().key())).collect())
    }
}

impl crate::extra::CookingRecipe {
    /// bukkit: org/bukkit/inventory/CookingRecipe.getInputChoice
    pub fn get_input_choice(&self) -> RecipeChoice {
        listed_input(&self.extra).unwrap_or_else(|| RecipeChoice::Materials(vec![self.get_input().clone()]))
    }
}

impl crate::extra::StonecuttingRecipe {
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.getInputChoice
    pub fn get_input_choice(&self) -> RecipeChoice {
        listed_input(&self.extra).unwrap_or_else(|| RecipeChoice::Materials(vec![self.get_input().clone()]))
    }
}

/// A spawner's saved `SpawnPotentials` row: `org.bukkit.block.spawner.SpawnerEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnerEntry(pub Value);

impl SpawnerEntry {
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry.getSpawnWeight
    pub fn get_spawn_weight(&self) -> i32 {
        self.0["weight"].as_i64().unwrap_or(1) as i32
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry.setSpawnWeight
    pub fn set_spawn_weight(&mut self, weight: i32) {
        self.0["weight"] = json!(weight);
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry.getSnapshot
    pub fn get_snapshot(&self) -> crate::entities3::EntitySnapshot {
        crate::entities3::create_entity_snapshot(&crate::data::snbt(&self.0["data"]["entity"]))
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry.setSnapshot
    pub fn set_snapshot(&mut self, snapshot: &crate::entities3::EntitySnapshot) {
        self.0["data"]["entity"] = snbt_json(&snapshot.0);
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry.getSpawnRule
    pub fn get_spawn_rule(&self) -> Option<SpawnRule> {
        let rule = &self.0["data"]["custom_spawn_rules"];
        (!rule.is_null()).then(|| SpawnRule(rule.clone()))
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry.setSpawnRule
    pub fn set_spawn_rule(&mut self, rule: &SpawnRule) {
        self.0["data"]["custom_spawn_rules"] = rule.0.clone();
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry.getEquipment
    ///
    /// The saved `equipment` of the entry: its loot table and drop chances.
    pub fn get_equipment(&self) -> Option<SpawnerEquipment> {
        let equipment = &self.0["data"]["equipment"];
        (!equipment.is_null()).then(|| SpawnerEquipment(equipment.clone()))
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry.setEquipment
    pub fn set_equipment(&mut self, equipment: &SpawnerEquipment) {
        self.0["data"]["equipment"] = equipment.0.clone();
    }
}

/// `org.bukkit.block.spawner.SpawnerEntry$Equipment`, as saved.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnerEquipment(pub Value);

impl SpawnerEquipment {
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry$Equipment.getEquipmentLootTable
    pub fn get_equipment_loot_table(&self) -> Option<String> {
        self.0["loot_table"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry$Equipment.setEquipmentLootTable
    pub fn set_equipment_loot_table(&mut self, table: &str) {
        self.0["loot_table"] = json!(table);
    }
    /// bukkit: org/bukkit/block/spawner/SpawnerEntry$Equipment.getDropChances
    pub fn get_drop_chances(&self) -> BTreeMap<String, f32> {
        self.0["slot_drop_chances"].as_object().into_iter().flatten().map(|(slot, chance)| (slot.clone(), number(chance))).collect()
    }
}

/// `org.bukkit.block.spawner.SpawnRule`, as a spawner saves `custom_spawn_rules`.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnRule(pub Value);

impl SpawnRule {
    /// bukkit: org/bukkit/block/spawner/SpawnRule.<init>
    pub fn new(min_block: i32, max_block: i32, min_sky: i32, max_sky: i32) -> Self {
        Self(json!({ "block_light_limit": [min_block, max_block], "sky_light_limit": [min_sky, max_sky] }))
    }
    fn limit(&self, key: &str, index: usize, fallback: i32) -> i32 {
        self.0[key][index].as_i64().map_or(fallback, |value| value as i32)
    }
    fn set_limit(&mut self, key: &str, index: usize, value: i32) {
        let mut pair = [self.limit(key, 0, 0), self.limit(key, 1, 15)];
        pair[index] = value;
        self.0[key] = json!(pair);
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.getMinBlockLight
    pub fn get_min_block_light(&self) -> i32 {
        self.limit("block_light_limit", 0, 0)
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.setMinBlockLight
    pub fn set_min_block_light(&mut self, light: i32) {
        self.set_limit("block_light_limit", 0, light);
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.getMaxBlockLight
    pub fn get_max_block_light(&self) -> i32 {
        self.limit("block_light_limit", 1, 15)
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.setMaxBlockLight
    pub fn set_max_block_light(&mut self, light: i32) {
        self.set_limit("block_light_limit", 1, light);
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.getMinSkyLight
    pub fn get_min_sky_light(&self) -> i32 {
        self.limit("sky_light_limit", 0, 0)
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.setMinSkyLight
    pub fn set_min_sky_light(&mut self, light: i32) {
        self.set_limit("sky_light_limit", 0, light);
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.getMaxSkyLight
    pub fn get_max_sky_light(&self) -> i32 {
        self.limit("sky_light_limit", 1, 15)
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.setMaxSkyLight
    pub fn set_max_sky_light(&mut self, light: i32) {
        self.set_limit("sky_light_limit", 1, light);
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.serialize
    pub fn serialize(&self) -> BTreeMap<String, Value> {
        let range = |low: i32, high: i32| json!({ "min": low, "max": high });
        BTreeMap::from([("block".to_owned(), range(self.get_min_block_light(), self.get_max_block_light())), ("sky".to_owned(), range(self.get_min_sky_light(), self.get_max_sky_light()))])
    }
    /// bukkit: org/bukkit/block/spawner/SpawnRule.deserialize
    pub fn deserialize(saved: &BTreeMap<String, Value>) -> Self {
        let at = |key: &str, end: &str, fallback: i32| saved.get(key).and_then(|range| range[end].as_i64()).map_or(fallback, |value| value as i32);
        Self::new(at("block", "min", 0), at("block", "max", 15), at("sky", "min", 0), at("sky", "max", 15))
    }
}

/// Banner pattern and trim helpers by name, as Bukkit's classes give them.
///
/// bukkit: org/bukkit/block/banner/PatternType.getIdentifier
pub fn pattern_type_identifier(key: &str) -> String {
    key.trim_start_matches("minecraft:").to_owned()
}

/// bukkit: org/bukkit/block/banner/PatternType.getByIdentifier
pub fn pattern_type_by_identifier(identifier: &str) -> Option<String> {
    let key = Material::of(identifier).key().to_owned();
    crate::registry::Registry::BANNER_PATTERN.get(&key)
}

/// bukkit: org/bukkit/block/banner/PatternType.getKey, org/bukkit/block/banner/PatternType.key
pub fn pattern_type_key(pattern: &str) -> Option<NamespacedKey> {
    NamespacedKey::from_string(pattern)
}

impl crate::meta_lists::Pattern {
    /// bukkit: org/bukkit/block/banner/Pattern.getPattern
    pub fn get_pattern(&self) -> &str {
        &self.pattern
    }
    /// bukkit: org/bukkit/block/banner/Pattern.getColor
    pub fn get_color(&self) -> String {
        self.color.to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/banner/Pattern.serialize
    pub fn serialize(&self) -> BTreeMap<String, Value> {
        BTreeMap::from([("color".to_owned(), json!(self.get_color())), ("pattern".to_owned(), json!(self.pattern))])
    }
}

/// bukkit: org/bukkit/inventory/meta/trim/TrimMaterial.key, org/bukkit/inventory/meta/trim/TrimPattern.key
pub fn trim_entry_key(id: &str) -> Option<NamespacedKey> {
    NamespacedKey::from_string(id)
}

/// bukkit: org/bukkit/inventory/meta/trim/TrimMaterial.getTranslationKey, org/bukkit/inventory/meta/trim/TrimMaterial.description
pub fn trim_material_translation_key(id: &str) -> String {
    format!("trim_material.{}", id.replace(':', "."))
}

/// bukkit: org/bukkit/inventory/meta/trim/TrimPattern.getTranslationKey, org/bukkit/inventory/meta/trim/TrimPattern.description
pub fn trim_pattern_translation_key(id: &str) -> String {
    format!("trim_pattern.{}", id.replace(':', "."))
}

// ---- What's left of blocks, recipes and mounts ----

/// `org.bukkit.block.Lidded`: chests, barrels and shulker boxes, held open by a plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lidded(pub Block);

impl Lidded {
    /// bukkit: org/bukkit/block/Lidded.open
    pub fn open(&self) -> bool {
        flag(&self.0.asked("items_lid", json!({ "open": true })))
    }
    /// bukkit: org/bukkit/block/Lidded.close
    pub fn close(&self) -> bool {
        flag(&self.0.asked("items_lid", json!({ "open": false })))
    }
    /// bukkit: org/bukkit/block/Lidded.isOpen
    pub fn is_open(&self) -> bool {
        flag(&self.0.asked("items_lid_open", json!({})))
    }
}

impl blocks::Beehive {
    /// bukkit: org/bukkit/block/EntityBlockStorage.releaseEntities
    pub fn release_entities(&self) -> Vec<crate::entity::Entity> {
        let released = self.0.get_block().asked("items_hive_release", json!({}));
        let ids: Vec<u64> = released.as_array().into_iter().flatten().filter_map(Value::as_u64).collect();
        self.0.get_block().get_world().get_entities().into_iter().filter(|entity| ids.contains(&entity.get_entity_id())).collect()
    }
    /// bukkit: org/bukkit/block/EntityBlockStorage.addEntity
    pub fn add_entity(&self, bee: &crate::entity::Entity) -> bool {
        flag(&self.0.get_block().asked("items_hive_add", json!({ "id": bee.get_entity_id() })))
    }
}

/// `org.bukkit.block.Conduit`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conduit(pub Block);

impl Conduit {
    fn live(&self) -> Value {
        self.0.asked("items_conduit", json!({}))
    }
    /// bukkit: org/bukkit/block/Conduit.isActive
    pub fn is_active(&self) -> bool {
        flag(&self.live()["active"])
    }
    /// bukkit: org/bukkit/block/Conduit.getFrameBlockCount
    pub fn get_frame_block_count(&self) -> i32 {
        self.live()["frames"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Conduit.getRange
    ///
    /// 16 blocks a full ring of seven frame blocks, as `ConduitBlockEntity.applyEffects`.
    pub fn get_range(&self) -> i32 {
        self.get_frame_block_count() / 7 * 16
    }
    /// bukkit: org/bukkit/block/Conduit.isHunting
    pub fn is_hunting(&self) -> bool {
        self.get_frame_block_count() >= 42
    }
    /// bukkit: org/bukkit/block/Conduit.hasTarget
    pub fn has_target(&self) -> bool {
        self.live()["target"].is_number()
    }
    /// bukkit: org/bukkit/block/Conduit.getTarget
    pub fn get_target(&self) -> Option<crate::entity::Entity> {
        let id = self.live()["target"].as_u64()?;
        self.0.get_world().get_entities().into_iter().find(|entity| entity.get_entity_id() == id)
    }
    /// bukkit: org/bukkit/block/Conduit.setTarget
    pub fn set_target(&self, target: Option<&crate::entity::Entity>) -> bool {
        flag(&self.0.asked("items_conduit_target", json!({ "id": target.map(crate::entity::Entity::get_entity_id) })))
    }
    /// bukkit: org/bukkit/block/Conduit.getHuntingArea
    pub fn get_hunting_area(&self) -> crate::util::BoundingBox {
        let (x, y, z) = (f64::from(self.0.get_x()), f64::from(self.0.get_y()), f64::from(self.0.get_z()));
        crate::util::BoundingBox::new(x - 8.0, y - 8.0, z - 8.0, x + 9.0, y + 9.0, z + 9.0)
    }
}

/// `org.bukkit.inventory.AbstractHorseInventory` and its kin, over the mount's saved equipment.
#[derive(Clone, Debug, PartialEq)]
pub struct MountInventory(pub crate::entity::Entity);

impl MountInventory {
    fn equipped(&self, slot: &str) -> Option<ItemStack> {
        ItemStack::read(&self.0.data_at(&format!("equipment.{slot}")))
    }
    fn equip(&self, slot: &str, stack: &ItemStack) -> bool {
        self.0.merge(&format!("{{equipment:{{{slot}:{}}}}}", stack.to_snbt()))
    }
    /// bukkit: org/bukkit/inventory/AbstractHorseInventory.getSaddle, org/bukkit/inventory/SaddledMountInventory.getSaddle
    pub fn get_saddle(&self) -> Option<ItemStack> {
        self.equipped("saddle")
    }
    /// bukkit: org/bukkit/inventory/AbstractHorseInventory.setSaddle, org/bukkit/inventory/SaddledMountInventory.setSaddle
    pub fn set_saddle(&self, stack: &ItemStack) -> bool {
        self.equip("saddle", stack)
    }
    /// bukkit: org/bukkit/inventory/ArmoredHorseInventory.getArmor, org/bukkit/inventory/ArmoredMountInventory.getArmor, org/bukkit/inventory/LlamaInventory.getDecor
    pub fn get_armor(&self) -> Option<ItemStack> {
        self.equipped("body")
    }
    /// bukkit: org/bukkit/inventory/ArmoredHorseInventory.setArmor, org/bukkit/inventory/ArmoredMountInventory.setArmor, org/bukkit/inventory/LlamaInventory.setDecor
    pub fn set_armor(&self, stack: &ItemStack) -> bool {
        self.equip("body", stack)
    }
}

/// A recipe's ingredient as its file has it: an id, a `#tag` or a list.
fn recipe_choice(value: &Value) -> RecipeChoice {
    RecipeChoice::Materials(holder_members(value, "item").iter().map(|id| Material::of(id)).collect())
}

impl crate::core2::GameRecipe {
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getKey, org/bukkit/inventory/SmithingRecipe.getKey
    pub fn get_key(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(&self.key)
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getResult, org/bukkit/inventory/SmithingRecipe.getResult
    pub fn get_recipe_result(&self) -> Option<ItemStack> {
        self.get_result()
    }
    /// bukkit: org/bukkit/inventory/SmithingRecipe.getBase
    pub fn get_base(&self) -> RecipeChoice {
        recipe_choice(&self.data["base"])
    }
    /// bukkit: org/bukkit/inventory/SmithingRecipe.getAddition
    pub fn get_addition(&self) -> RecipeChoice {
        recipe_choice(&self.data["addition"])
    }
    /// bukkit: org/bukkit/inventory/SmithingTransformRecipe.getTemplate, org/bukkit/inventory/SmithingTrimRecipe.getTemplate
    pub fn get_template(&self) -> RecipeChoice {
        recipe_choice(&self.data["template"])
    }
    /// bukkit: org/bukkit/inventory/SmithingTrimRecipe.getTrimPattern
    pub fn get_trim_pattern(&self) -> Option<String> {
        self.data["pattern"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/SmithingRecipe.willCopyNbt, org/bukkit/inventory/SmithingRecipe.willCopyDataComponents
    ///
    /// A recipe of the game's data always does.
    pub fn will_copy_data_components(&self) -> bool {
        true
    }
    /// bukkit: org/bukkit/inventory/TransmuteRecipe.getInput
    pub fn get_input(&self) -> RecipeChoice {
        recipe_choice(&self.data["input"])
    }
    /// bukkit: org/bukkit/inventory/TransmuteRecipe.getMaterial
    pub fn get_material(&self) -> RecipeChoice {
        recipe_choice(&self.data["material"])
    }
}

impl CraftingInventory {
    /// bukkit: org/bukkit/inventory/CraftingInventory.getRecipe
    ///
    /// The game's recipe the grid makes now.
    pub fn get_recipe(&self) -> Option<crate::core2::GameRecipe> {
        let grid: Vec<Value> = self.get_matrix().iter().map(|stack| stack.as_ref().map_or(Value::Null, ItemStack::to_json)).collect();
        let id = query(json!({ "op": "items_grid_recipe", "grid": grid }));
        crate::core2::get_recipe(&NamespacedKey::from_string(&Material::of(id.as_str()?).key().to_owned())?)
    }
}

impl StonecutterView {
    /// bukkit: org/bukkit/inventory/view/StonecutterView.getRecipes
    pub fn get_recipes(&self) -> Vec<crate::core2::GameRecipe> {
        self.0.state()["recipes"].as_array().into_iter().flatten().filter_map(Value::as_str).filter_map(|id| NamespacedKey::from_string(&Material::of(id).key().to_owned())).filter_map(|key| crate::core2::get_recipe(&key)).collect()
    }
}

impl LoomView {
    /// bukkit: org/bukkit/inventory/view/LoomView.getTopInventory
    pub fn get_top_inventory(&self) -> OpenMenu {
        self.0.clone()
    }
}

impl CrafterView {
    /// bukkit: org/bukkit/inventory/view/CrafterView.getTopInventory
    pub fn get_top_inventory(&self) -> Option<BlockInventory> {
        self.0.block().map(|block| BlockInventory::of(&block))
    }
}

impl EnchantmentView {
    /// bukkit: org/bukkit/inventory/view/EnchantmentView.setOffers
    ///
    /// Each offer's enchantment, level and level cost; `None` clears it.
    pub fn set_offers(&self, offers: &[Option<(String, i32, i32)>; 3]) {
        for (index, offer) in offers.iter().enumerate() {
            let (key, level, cost) = offer.as_ref().map_or((None, -1, 0), |(key, level, cost)| (Some(key.as_str()), *level, *cost));
            query(json!({ "op": "items_menu_set", "uuid": self.0.player.uuid(), "offer": index, "key": key, "level": level, "cost": cost }));
        }
    }
    /// bukkit: org/bukkit/inventory/view/EnchantmentView.setEnchantmentSeed
    pub fn set_enchantment_seed(&self, seed: i32) {
        self.0.set_data(3, seed);
    }
}

impl RecipeChoice {
    /// bukkit: org/bukkit/inventory/RecipeChoice$ItemTypeChoice.itemTypes
    pub fn item_types(&self) -> Vec<ItemType> {
        self.get_choices().iter().map(|stack| ItemType::of(stack.get_type().key())).collect()
    }
}

/// bukkit: org/bukkit/potion/PotionEffectTypeWrapper.getType
pub fn potion_effect_type_wrapper(wrapped: &PotionEffectType) -> PotionEffectType {
    wrapped.clone()
}

/// An item tag's conversion: `org.bukkit.inventory.meta.tags.ItemTagType`.
pub trait ItemTagType {
    type Primitive;
    type Complex;
    /// bukkit: org/bukkit/inventory/meta/tags/ItemTagType.toPrimitive
    fn to_primitive(&self, complex: Self::Complex, context: &ItemTagAdapterContext) -> Self::Primitive;
    /// bukkit: org/bukkit/inventory/meta/tags/ItemTagType.fromPrimitive
    fn from_primitive(&self, primitive: Self::Primitive, context: &ItemTagAdapterContext) -> Self::Complex;
}

/// `org.bukkit.inventory.meta.tags.ItemTagType$PrimitiveTagType`: a value that is its own primitive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveTagType<T>(std::marker::PhantomData<T>);

impl<T> ItemTagType for PrimitiveTagType<T> {
    type Primitive = T;
    type Complex = T;
    /// bukkit: org/bukkit/inventory/meta/tags/ItemTagType$PrimitiveTagType.toPrimitive
    fn to_primitive(&self, complex: T, _context: &ItemTagAdapterContext) -> T {
        complex
    }
    /// bukkit: org/bukkit/inventory/meta/tags/ItemTagType$PrimitiveTagType.fromPrimitive
    fn from_primitive(&self, primitive: T, _context: &ItemTagAdapterContext) -> T {
        primitive
    }
}

/// `org.bukkit.inventory.meta.tags.ItemTagAdapterContext`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemTagAdapterContext;

impl ItemTagAdapterContext {
    /// bukkit: org/bukkit/inventory/meta/tags/ItemTagAdapterContext.newTagContainer
    ///
    /// A container needs a stack to live on: an empty one, which `CustomItemTagContainer` writes to.
    pub fn new_tag_container(&self) -> ItemStack {
        ItemStack::new(Material::of("STONE"), 1)
    }
}

impl CustomItemTagContainer<'_> {
    /// bukkit: org/bukkit/inventory/meta/tags/CustomItemTagContainer.getAdapterContext
    pub fn get_adapter_context(&self) -> ItemTagAdapterContext {
        ItemTagAdapterContext
    }
}

/// bukkit: io/papermc/paper/datacomponent/DataComponentType.isPersistent
///
/// All but the two the game keeps only in memory.
pub fn data_component_is_persistent(id: &str) -> bool {
    !matches!(id, "minecraft:creative_slot_lock" | "minecraft:map_post_processing")
}

/// A component class's builder: `io.papermc.paper.datacomponent.DataComponentBuilder`.
pub trait DataComponentBuilder {
    type Built;
    /// bukkit: io/papermc/paper/datacomponent/DataComponentBuilder.build
    fn build_component(self) -> Self::Built;
}

/// `io.papermc.paper.datacomponent.BuildableDataComponent`.
pub trait BuildableDataComponent {
    type Builder;
    /// bukkit: io/papermc/paper/datacomponent/BuildableDataComponent.toBuilder
    fn to_builder(&self) -> Self::Builder;
}

macro_rules! buildable {
    ($($class:ident, $builder:ident;)*) => {
        $(
            impl DataComponentBuilder for crate::components::$builder {
                type Built = crate::components::$class;
                fn build_component(self) -> crate::components::$class {
                    self.build()
                }
            }

            impl BuildableDataComponent for crate::components::$class {
                type Builder = crate::components::$builder;
                fn to_builder(&self) -> crate::components::$builder {
                    crate::components::$builder::from_built(&self.0)
                }
            }
        )*
    };
}

buildable!(
    AttackRange, AttackRangeBuilder;
    BannerPatternLayers, BannerPatternLayersBuilder;
    BlocksAttacks, BlocksAttacksBuilder;
    BundleContents, BundleContentsBuilder;
    ChargedProjectiles, ChargedProjectilesBuilder;
    Consumable, ConsumableBuilder;
    CustomModelData, CustomModelDataBuilder;
    DeathProtection, DeathProtectionBuilder;
    DyedItemColor, DyedItemColorBuilder;
    Equippable, EquippableBuilder;
    Fireworks, FireworksBuilder;
    FoodProperties, FoodPropertiesBuilder;
    ItemAdventurePredicate, ItemAdventurePredicateBuilder;
    ItemArmorTrim, ItemArmorTrimBuilder;
    ItemAttributeModifiers, ItemAttributeModifiersBuilder;
    ItemContainerContents, ItemContainerContentsBuilder;
    ItemEnchantments, ItemEnchantmentsBuilder;
    ItemLore, ItemLoreBuilder;
    JukeboxPlayable, JukeboxPlayableBuilder;
    KineticWeapon, KineticWeaponBuilder;
    LodestoneTracker, LodestoneTrackerBuilder;
    MapDecorations, MapDecorationsBuilder;
    PiercingWeapon, PiercingWeaponBuilder;
    PotDecorations, PotDecorationsBuilder;
    PotionContents, PotionContentsBuilder;
    ResolvableProfile, ResolvableProfileBuilder;
    SeededContainerLoot, SeededContainerLootBuilder;
    SignText, SignTextBuilder;
    SuspiciousStewEffects, SuspiciousStewEffectsBuilder;
    SwingAnimation, SwingAnimationBuilder;
    Tool, ToolBuilder;
    TooltipDisplay, TooltipDisplayBuilder;
    UseCooldown, UseCooldownBuilder;
    UseEffects, UseEffectsBuilder;
    Weapon, WeaponBuilder;
    WritableBookContent, WritableBookContentBuilder;
    WrittenBookContent, WrittenBookContentBuilder;
);

impl ItemFactory {
    /// bukkit: org/bukkit/inventory/ItemFactory.enchantWithLevels
    ///
    /// `seed` stands for Java's `Random`: `None` draws from the level's own.
    pub fn enchant_with_levels(&self, stack: &ItemStack, levels: i32, treasure: bool, seed: Option<i64>) -> Option<ItemStack> {
        stack.enchant_with_levels(levels, treasure, seed)
    }
    /// bukkit: org/bukkit/inventory/ItemFactory.enchantItem
    ///
    /// Java draws from the entity's random: one more stream of the same game's numbers.
    pub fn enchant_item(&self, _entity: &crate::entity::Entity, stack: &ItemStack, levels: i32, treasure: bool) -> Option<ItemStack> {
        stack.enchant_with_levels(levels, treasure, None)
    }
}

impl blocks::Sign {
    /// bukkit: org/bukkit/block/Sign.getInteractableSideFor
    ///
    /// The side a point at `x`, `z` sees, as `SignBlockEntity.isFacingFrontText`.
    pub fn get_interactable_side_for(&self, x: f64, z: f64) -> &'static str {
        let block = self.0.get_block();
        let data = block.block_data();
        let degrees = match data.property("rotation").and_then(|step| step.parse::<f64>().ok()) {
            Some(step) => step * 22.5,
            None => match data.property("facing") {
                Some("north") => 180.0,
                Some("east") => -90.0,
                Some("west") => 90.0,
                _ => 0.0,
            },
        };
        let radians = degrees.to_radians();
        let (front_x, front_z) = (-radians.sin(), radians.cos());
        let (dx, dz) = (x - (f64::from(block.get_x()) + 0.5), z - (f64::from(block.get_z()) + 0.5));
        if dx * front_x + dz * front_z >= 0.0 { "FRONT" } else { "BACK" }
    }
    /// bukkit: org/bukkit/block/Sign.getTargetSide
    pub fn get_target_side(&self, player: &Player) -> &'static str {
        let at = player.get_location();
        self.get_interactable_side_for(at.x, at.z)
    }
}

impl blocks::Furnace {
    /// bukkit: org/bukkit/block/Furnace.getInventory
    pub fn get_inventory(&self) -> BlockInventory {
        self.0.inventory()
    }
    /// bukkit: org/bukkit/block/Furnace.getSnapshotInventory
    pub fn get_snapshot_inventory(&mut self) -> SnapshotInventory<'_> {
        let size = container_size(self.0.get_block());
        SnapshotInventory { tile: &mut self.0, size }
    }
}

impl blocks::BrewingStand {
    /// bukkit: org/bukkit/block/BrewingStand.getInventory
    pub fn get_inventory(&self) -> BlockInventory {
        self.0.inventory()
    }
    /// bukkit: org/bukkit/block/BrewingStand.getSnapshotInventory
    pub fn get_snapshot_inventory(&mut self) -> SnapshotInventory<'_> {
        let size = container_size(self.0.get_block());
        SnapshotInventory { tile: &mut self.0, size }
    }
}

impl blocks::Jukebox {
    /// bukkit: org/bukkit/block/Jukebox.eject
    pub fn eject(&self) -> bool {
        flag(&self.0.get_block().asked("items_jukebox_eject", json!({})))
    }
}

impl ItemStack {
    /// bukkit: org/bukkit/inventory/ItemStack.getMaxItemUseDuration
    ///
    /// `Item.getUseDuration`: the same for any entity, a crossbow's charge being another number.
    pub fn get_max_item_use_duration(&self) -> i32 {
        if let Some(consumable) = self.get_data("minecraft:consumable") {
            return (consumable["consume_seconds"].as_f64().unwrap_or(1.6) * 20.0) as i32;
        }
        if self.has_data("minecraft:blocks_attacks") || self.has_data("minecraft:kinetic_weapon") {
            return 72_000;
        }
        match self.get_type().key() {
            "minecraft:bow" | "minecraft:crossbow" | "minecraft:trident" => 72_000,
            "minecraft:spyglass" => 1200,
            "minecraft:goat_horn" => query(json!({ "op": "items_instrument_ticks", "stack": self.to_json() })).as_i64().unwrap_or(0) as i32,
            "minecraft:brush" | "minecraft:bundle" => 200,
            name if name.ends_with("_bundle") => 200,
            _ => 0,
        }
    }
    /// bukkit~: org/bukkit/inventory/ItemStack.computeTooltipLines | the game's lines for these flags, styled, but as text in the server's language where Java keeps translatable keys for each client, and attribute lines as the acting player's: the engine builds them as strings
    pub fn compute_tooltip_lines(&self, advanced: bool, creative: bool) -> Vec<crate::Component> {
        query(json!({ "op": "items_tooltip_styled", "stack": self.to_json(), "advanced": advanced, "creative": creative })).as_array().into_iter().flatten().cloned().map(crate::Component).collect()
    }
}

impl crate::meta::MapMeta {
    /// bukkit: org/bukkit/inventory/meta/MapMeta.hasMapView
    pub fn has_map_view(&self) -> bool {
        self.has_map_id()
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.getMapView
    pub fn get_map_view(&self) -> Option<crate::fam_core::MapView> {
        crate::fam_core::get_map(self.get_map_id()?)
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.setMapView
    pub fn set_map_view(&mut self, map: &crate::fam_core::MapView) {
        self.set_map_id(map.get_id());
    }
}

/// What `InventoryView::get_inventory` finds at a raw slot.
#[derive(Clone, Debug)]
pub enum ViewInventory {
    /// The GUI or block's menu above.
    Top(OpenMenu),
    /// The player's own rows below it.
    Bottom(crate::PlayerInventory),
}

impl crate::inventory::InventoryView {
    /// bukkit: org/bukkit/inventory/InventoryView.getInventory
    ///
    /// `None` outside the window, as Java's `null`.
    pub fn get_inventory(&self, raw_slot: i32) -> Option<ViewInventory> {
        let raw = usize::try_from(raw_slot).ok()?;
        let top = self.top_size();
        if raw < top {
            return Some(ViewInventory::Top(self.menu()));
        }
        (raw < top + 36).then(|| ViewInventory::Bottom(self.get_bottom_inventory()))
    }
    /// bukkit~: org/bukkit/inventory/InventoryView.open | a plugin's GUI, or the menu still open, which goes out to the client again: this crate's view is the player's open menu by name, not Java's menu object, so a block's menu that has closed has nothing to reopen
    pub fn open(&self) -> bool {
        if let Some(inventory) = self.get_top_inventory() {
            self.get_player().open_inventory(inventory);
            return true;
        }
        self.get_menu_type().is_some() && self.set_title(&self.title())
    }
}

impl crate::extra::ShapedRecipe {
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getKey
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getResult
    pub fn get_result(&self) -> &ItemStack {
        &self.result
    }
}

impl crate::extra::ShapelessRecipe {
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getKey
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getResult
    pub fn get_result(&self) -> &ItemStack {
        &self.result
    }
}

impl crate::extra::CookingRecipe {
    /// bukkit: org/bukkit/inventory/CookingRecipe.setInputChoice, org/bukkit/inventory/FurnaceRecipe.setInputChoice
    pub fn set_input_choice(&mut self, choice: &RecipeChoice) {
        if let Some(stack) = choice.get_choices().first() {
            self.set_input(stack.get_type().clone());
        }
        self.extra.insert("ingredient".to_owned(), choice.ingredient());
    }
}

/// `org.bukkit.block.SculkCatalyst`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SculkCatalyst(pub Block);

impl SculkCatalyst {
    /// bukkit: org/bukkit/block/SculkCatalyst.bloom
    pub fn bloom(&self, at: &Block, charge: i32) -> bool {
        flag(&self.0.asked("items_catalyst_bloom", json!({ "at": [at.get_x(), at.get_y(), at.get_z()], "charge": charge })))
    }
}

// ---- What Paper keeps on a block entity and the game's own don't ----

/// A write of a `TileState` that isn't a merge into the block entity: `update` brings them here.
pub(crate) fn special_change(block: &Block, path: &str, value: &str) -> Option<bool> {
    if let Some(field) = path.strip_prefix("extra:") {
        let parsed: Value = serde_json::from_str(value).unwrap_or(Value::Null);
        return Some(flag(&block.asked("items_extra_set", json!({ "field": field, "value": parsed }))));
    }
    path.strip_prefix('=').map(|field| flag(&block.asked("items_block_put", json!({ "field": field, "snbt": value }))))
}

impl TileState {
    /// `Container.getInventory`: the block's, or a state of no block's own.
    pub(crate) fn inventory(&self) -> BlockInventory {
        match &self.held.slots {
            Some(slots) => BlockInventory { block: self.block.clone(), size: self.held.kind.as_ref().map_or(27, slots_of), second: None, held: Some(slots.clone()) },
            None => BlockInventory::of(&self.block),
        }
    }
    /// bukkit: org/bukkit/block/Container.getInventory, org/bukkit/inventory/BlockInventoryHolder.getInventory, org/bukkit/block/Container.getSnapshotInventory
    ///
    /// `Block.getState()` gives a TileState; cast to `Container` in Java, it is the block's inventory
    /// (CoreProtect reads a broken chest's items from it). jbridge says which states are containers.
    pub fn get_inventory(&self) -> BlockInventory {
        self.inventory()
    }
    /// bukkit: org/bukkit/block/Chest.getBlockInventory
    ///
    /// This chest's own half, as a TileState cast to `Chest`.
    pub fn get_block_inventory(&self) -> BlockInventory {
        self.inventory()
    }
    /// bukkit: org/bukkit/block/BlockState.getType
    pub fn get_type(&self) -> Material {
        if let Some(kind) = &self.held.kind {
            return kind.clone();
        }
        // The snapshot's type, as Java's BlockState: not the block's now.
        match self.state.split('[').next() {
            Some(name) if !name.is_empty() => Material::of(name),
            _ => self.block.get_type(),
        }
    }
    /// A field kept beside the block, as this state sees it: what it changed, then the world's.
    fn extra(&self, field: &str) -> Value {
        let wanted = format!("extra:{field}");
        if let Some((_, value)) = self.changes.iter().rev().find(|(path, _)| *path == wanted) {
            return serde_json::from_str(value).unwrap_or(Value::Null);
        }
        if self.held.kind.is_some() {
            return self.data["horizon:extras"][field].clone();
        }
        self.get_block().asked("items_extra", json!({}))[field].clone()
    }
    fn set_extra(&mut self, field: &str, value: &Value) {
        self.change(&format!("extra:{field}"), value.to_string());
    }
    /// bukkit: org/bukkit/block/TileState.getPersistentDataContainer
    pub fn get_persistent_data_container(&mut self) -> TileData<'_> {
        TileData(self)
    }
    /// bukkit: org/bukkit/block/Lockable.isLocked
    pub fn is_locked(&self) -> bool {
        !self.extra("lock").is_null()
    }
    /// bukkit: org/bukkit/block/Lockable.getLock
    ///
    /// The name a key must have, or nothing for a lock of another kind.
    pub fn get_lock(&self) -> String {
        text(&self.extra("lock")["name"])
    }
    /// bukkit: org/bukkit/block/Lockable.setLock
    ///
    /// An empty name takes the lock off.
    pub fn set_lock(&mut self, key: &str) {
        let lock = if key.is_empty() { Value::Null } else { json!({ "name": key }) };
        self.set_extra("lock", &lock);
    }
    /// bukkit: org/bukkit/block/Lockable.setLockItem
    pub fn set_lock_item(&mut self, key: Option<&ItemStack>) {
        let lock = key.map_or(Value::Null, |key| {
            let components: Map<String, Value> = key.all_components().map(|(id, value)| (id.clone(), value.clone())).collect();
            json!({ "item": key.get_type().key(), "components": components })
        });
        self.set_extra("lock", &lock);
    }
}

/// A block entity's `PersistentDataContainer`: written by `update`, as in Java.
pub struct TileData<'a>(&'a mut TileState);

impl TileData<'_> {
    fn all(&self) -> Map<String, Value> {
        self.0.extra("pdc").as_object().cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.get
    pub fn get(&self, key: &NamespacedKey) -> Option<Value> {
        self.all().get(key.as_str()).cloned()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.has
    pub fn has(&self, key: &NamespacedKey) -> bool {
        self.all().contains_key(key.as_str())
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.set
    pub fn set(&mut self, key: &NamespacedKey, value: Value) {
        let mut all = self.all();
        all.insert(key.as_str().to_owned(), value);
        self.0.set_extra("pdc", &Value::Object(all));
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.remove
    pub fn remove(&mut self, key: &NamespacedKey) {
        let mut all = self.all();
        all.remove(key.as_str());
        let value = if all.is_empty() { Value::Null } else { Value::Object(all) };
        self.0.set_extra("pdc", &value);
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainerView.getKeys
    pub fn get_keys(&self) -> Vec<String> {
        self.all().keys().cloned().collect()
    }
}

impl blocks::Beacon {
    /// bukkit: org/bukkit/block/Beacon.setEffectRange
    pub fn set_effect_range(&mut self, range: f64) {
        self.0.set_extra("effect_range", &json!(range));
    }
    /// bukkit: org/bukkit/block/Beacon.resetEffectRange
    pub fn reset_effect_range(&mut self) {
        self.0.set_extra("effect_range", &Value::Null);
    }
}

impl blocks::Beehive {
    /// bukkit: org/bukkit/block/EntityBlockStorage.setMaxEntities
    pub fn set_max_entities(&mut self, limit: i32) {
        assert!(limit > 0, "Max bees must be more than 0");
        self.0.set_extra("max_bees", &json!(limit));
    }
}

impl Campfire {
    fn stopped(&self) -> [bool; 4] {
        let held = self.0.extra("stop_cooking");
        std::array::from_fn(|slot| held[slot].as_bool().unwrap_or(false))
    }
    /// bukkit: org/bukkit/block/Campfire.isCookingDisabled
    pub fn is_cooking_disabled(&self, slot: usize) -> bool {
        self.stopped()[slot]
    }
    fn hold(&mut self, slot: Option<usize>, held: bool) -> bool {
        let mut flags = self.stopped();
        let before = slot.is_some_and(|slot| flags[slot]);
        match slot {
            Some(slot) => flags[slot] = held,
            None => flags = [held; 4],
        }
        let value = if flags.contains(&true) { json!(flags) } else { Value::Null };
        self.0.set_extra("stop_cooking", &value);
        before
    }
    /// bukkit: org/bukkit/block/Campfire.stopCooking
    ///
    /// One slot, or all four. Whether it was stopped before.
    pub fn stop_cooking(&mut self, slot: Option<usize>) -> bool {
        self.hold(slot, true)
    }
    /// bukkit: org/bukkit/block/Campfire.startCooking
    pub fn start_cooking(&mut self, slot: Option<usize>) -> bool {
        self.hold(slot, false)
    }
}

impl blocks::SculkSensor {
    /// bukkit: org/bukkit/block/SculkSensor.setListenerRange
    pub fn set_listener_range(&mut self, range: i32) {
        assert!(range > 0, "Vibration listener range must be greater than 0");
        self.0.set_extra("listener_range", &json!(range));
    }
}

impl blocks::Sign {
    /// bukkit: org/bukkit/block/Sign.setEditable
    pub fn set_editable(&mut self, editable: bool) {
        self.set_waxed(!editable);
    }
    /// bukkit: org/bukkit/block/Sign.getAllowedEditorUniqueId
    pub fn get_allowed_editor_unique_id(&self) -> Option<String> {
        self.0.get_block().asked("items_sign_editor", json!({})).as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Sign.setAllowedEditorUniqueId
    ///
    /// By the player's id without dashes, or nobody.
    pub fn set_allowed_editor_unique_id(&self, uuid: Option<&str>) -> bool {
        flag(&self.0.get_block().asked("items_sign_editor_set", json!({ "uuid": uuid.map(|id| id.replace('-', "")) })))
    }
    /// bukkit: org/bukkit/block/Sign.getAllowedEditor
    pub fn get_allowed_editor(&self) -> Option<Player> {
        let id = self.get_allowed_editor_unique_id()?;
        crate::get_online_players().into_iter().find(|player| player.uuid() == id)
    }
}

impl blocks::Jukebox {
    /// bukkit: org/bukkit/block/Jukebox.setPlaying
    ///
    /// The record, by `update`; `None` or air empties it.
    pub fn set_playing(&mut self, record: Option<&Material>) {
        match record.filter(|record| !record.is_air()) {
            Some(record) => self.set_record(&ItemStack::new(record.clone(), 1)),
            None => self.0.change("RecordItem", "{}".to_owned()),
        }
    }
    /// bukkit: org/bukkit/block/Jukebox.startPlaying
    pub fn start_playing(&self) -> bool {
        flag(&self.0.get_block().asked("items_jukebox_set", json!({ "start": true })))
    }
    /// bukkit: org/bukkit/block/Jukebox.stopPlaying
    pub fn stop_playing(&self) {
        self.0.get_block().asked("items_jukebox_set", json!({ "start": false }));
    }
}

impl Dispenser {
    /// bukkit: org/bukkit/block/Dispenser.getBlockProjectileSource
    pub fn get_block_projectile_source(&self) -> Option<BlockProjectileSource> {
        (self.0.get_block().get_type().key() == "minecraft:dispenser").then(|| BlockProjectileSource(self.0.get_block().clone()))
    }
}

/// `org.bukkit.projectiles.BlockProjectileSource`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockProjectileSource(pub Block);

impl BlockProjectileSource {
    /// bukkit: org/bukkit/projectiles/BlockProjectileSource.getBlock
    pub fn get_block(&self) -> &Block {
        &self.0
    }
}

/// `org.bukkit.block.CreakingHeart`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreakingHeart(pub Block);

impl CreakingHeart {
    fn entity(&self, id: Option<u64>) -> Option<crate::entity::Entity> {
        let id = id?;
        self.0.get_world().get_entities().into_iter().find(|entity| entity.get_entity_id() == id)
    }
    /// bukkit: org/bukkit/block/CreakingHeart.getCreaking
    pub fn get_creaking(&self) -> Option<crate::entity::Entity> {
        self.entity(self.0.asked("items_heart_creaking", json!({})).as_u64())
    }
    /// bukkit: org/bukkit/block/CreakingHeart.setCreaking
    pub fn set_creaking(&self, creaking: Option<&crate::entity::Entity>) -> bool {
        flag(&self.0.asked("items_heart_set", json!({ "id": creaking.map(crate::entity::Entity::get_entity_id) })))
    }
    /// bukkit: org/bukkit/block/CreakingHeart.spreadResin
    pub fn spread_resin(&self) -> Option<Location> {
        let at = self.0.asked("items_heart_resin", json!({}));
        let spot = |index: usize| at[index].as_f64();
        Some(Location::new(self.0.get_world().clone(), spot(0)?, spot(1)?, spot(2)?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/block/CreakingHeart.spawnCreaking
    pub fn spawn_creaking(&self) -> Option<crate::entity::Entity> {
        self.entity(self.0.asked("items_heart_spawn", json!({})).as_u64())
    }
}

impl SculkShrieker {
    /// bukkit: org/bukkit/block/SculkShrieker.tryShriek
    ///
    /// At the next sculk tick, this tick's if the sculk hasn't run yet. No player, no shriek, as in Java.
    pub fn try_shriek(&self, player: Option<&Player>) -> bool {
        player.is_some() && flag(&self.0.get_block().asked("items_try_shriek", json!({})))
    }
}

impl blocks::TrialSpawner {
    /// bukkit: org/bukkit/block/TrialSpawner.getNormalConfiguration
    pub fn get_normal_configuration(&self) -> crate::fam_core::TrialSpawnerConfiguration {
        crate::fam_core::TrialSpawnerConfiguration::of(self.0.get_block(), false)
    }
    /// bukkit: org/bukkit/block/TrialSpawner.getOminousConfiguration
    pub fn get_ominous_configuration(&self) -> crate::fam_core::TrialSpawnerConfiguration {
        crate::fam_core::TrialSpawnerConfiguration::of(self.0.get_block(), true)
    }
}

// ---- Stacks as bytes, and as maps ----

impl ItemStack {
    /// bukkit: org/bukkit/inventory/ItemStack.getI18NDisplayName
    pub fn get_i18n_display_name(&self) -> String {
        let mut plain = self.clone();
        plain.remove_component("minecraft:custom_name");
        query(json!({ "op": "items_tooltip", "stack": plain.to_json() }))[0].as_str().unwrap_or_default().to_owned()
    }
    /// bukkit: org/bukkit/inventory/ItemStack.setData
    pub fn set_data(&mut self, component: &str, value: Value) {
        self.remove_component(&format!("!{component}"));
        self.set_component(component, value);
    }
    /// bukkit: org/bukkit/inventory/ItemStack.enchantWithLevels
    ///
    /// `seed` stands for Java's `Random`: `None` draws from the level's own.
    pub fn enchant_with_levels(&self, levels: i32, treasure: bool, seed: Option<i64>) -> Option<Self> {
        Self::read(&query(json!({ "op": "items_enchant_seeded", "stack": self.to_json(), "levels": levels, "treasure": treasure, "seed": seed })))
    }
}

// ---- Inventories of a kind, and live views ----

static KINDS: std::sync::Mutex<BTreeMap<u32, &'static str>> = std::sync::Mutex::new(BTreeMap::new());
static LIMITS: std::sync::Mutex<BTreeMap<u32, i32>> = std::sync::Mutex::new(BTreeMap::new());

/// An `InventoryType`'s default size and the game's menu for it.
fn inventory_type_menu(kind: &str) -> Option<(usize, &'static str)> {
    Some(match kind {
        "CHEST" | "ENDER_CHEST" | "BARREL" => (27, "minecraft:generic_9x3"),
        "DISPENSER" | "DROPPER" => (9, "minecraft:generic_3x3"),
        "FURNACE" => (3, "minecraft:furnace"),
        "WORKBENCH" => (10, "minecraft:crafting"),
        "ENCHANTING" => (2, "minecraft:enchantment"),
        "BREWING" => (5, "minecraft:brewing_stand"),
        "MERCHANT" => (3, "minecraft:merchant"),
        "ANVIL" => (3, "minecraft:anvil"),
        "SMITHING" => (4, "minecraft:smithing"),
        "BEACON" => (1, "minecraft:beacon"),
        "HOPPER" => (5, "minecraft:hopper"),
        "SHULKER_BOX" => (27, "minecraft:shulker_box"),
        "BLAST_FURNACE" => (3, "minecraft:blast_furnace"),
        "LECTERN" => (1, "minecraft:lectern"),
        "SMOKER" => (3, "minecraft:smoker"),
        "LOOM" => (4, "minecraft:loom"),
        "CARTOGRAPHY" => (3, "minecraft:cartography_table"),
        "GRINDSTONE" => (3, "minecraft:grindstone"),
        "STONECUTTER" => (2, "minecraft:stonecutter"),
        "CRAFTER" => (9, "minecraft:crafter_3x3"),
        _ => return None,
    })
}

/// The type a plugin's GUI was made as.
pub(crate) fn inventory_kind(id: u32) -> &'static str {
    KINDS.lock().unwrap_or_else(|e| e.into_inner()).get(&id).copied().unwrap_or("CHEST")
}

/// The limit a plugin set on its GUI, or the game's.
pub(crate) fn inventory_limit(id: u32) -> i32 {
    LIMITS.lock().unwrap_or_else(|e| e.into_inner()).get(&id).copied().unwrap_or(99)
}

/// Opens a GUI made as a type a chest menu can't show. False for a chest, which `open_inventory` does.
pub(crate) fn open_typed(player: &Player, inventory: &crate::Inventory) -> bool {
    let kind = inventory_kind(inventory.id);
    if kind == "CHEST" {
        return false;
    }
    let Some((_, menu)) = inventory_type_menu(kind) else {
        return false;
    };
    let opened = query(json!({ "op": "items_open_gui", "uuid": player.uuid(), "menu": menu, "title": crate::adventure::legacy(inventory.get_title()).0, "items": inventory.items() }));
    if flag(&opened) {
        crate::OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner()).insert(player.uuid().to_owned(), inventory.clone());
    }
    true
}

impl crate::Inventory {
    /// bukkit~: org/bukkit/Bukkit.createInventory, org/bukkit/Server.createInventory | that type's slots and window, moving stacks as a chest does: a furnace's fuel or a crafting result has no rule of its own, as the game's rules live with its block
    ///
    /// By `InventoryType`: `HOPPER`, `WORKBENCH`, `ANVIL`... `None` for a type a plugin's GUI can't be.
    pub fn create_of_type(kind: &str, title: &str) -> Option<Self> {
        let (size, _) = inventory_type_menu(kind)?;
        let inventory = Self::create(size, title);
        if let Some(known) = ["ENDER_CHEST", "BARREL", "DISPENSER", "DROPPER", "FURNACE", "WORKBENCH", "ENCHANTING", "BREWING", "MERCHANT", "ANVIL", "SMITHING", "BEACON", "HOPPER", "SHULKER_BOX", "BLAST_FURNACE", "LECTERN", "SMOKER", "LOOM", "CARTOGRAPHY", "GRINDSTONE", "STONECUTTER", "CRAFTER"].into_iter().find(|known| *known == kind) {
            KINDS.lock().unwrap_or_else(|e| e.into_inner()).insert(inventory.id, known);
        }
        Some(inventory)
    }
    /// bukkit: org/bukkit/inventory/Inventory.setMaxStackSize
    pub fn set_max_stack_size(&self, limit: i32) {
        LIMITS.lock().unwrap_or_else(|e| e.into_inner()).insert(self.id, limit);
    }
}

/// `org.bukkit.inventory.Inventory` of type `ENDER_CHEST`: a player's own, live: each call reads or
/// writes it where the game keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnderChest {
    player: Player,
}

impl Player {
    /// bukkit: org/bukkit/entity/HumanEntity.getEnderChest
    pub fn ender_chest(&self) -> EnderChest {
        EnderChest { player: self.clone() }
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openInventory
    ///
    /// Another player's inventory (`PLAYER`) or ender chest as a window on the real thing: what this
    /// player moves, the owner has, and what the owner does shows here, within the tick. `Inventory`'s
    /// order holds: hotbar, the rest, boots to helmet, off hand.
    pub fn open_live(&self, target: &Player, ender: bool, title: &crate::Component) -> bool {
        flag(&query(json!({ "op": "items_open_live", "uuid": self.uuid(), "target": target.uuid(), "ender": ender, "title": title.0 })))
    }
}

impl EnderChest {
    /// bukkit: org/bukkit/inventory/Inventory.getHolder
    pub fn get_holder(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/inventory/Inventory.getType
    pub fn get_type(&self) -> &'static str {
        "ENDER_CHEST"
    }
    /// bukkit: org/bukkit/inventory/Inventory.getSize
    pub fn get_size(&self) -> usize {
        27
    }
    /// bukkit: org/bukkit/inventory/Inventory.getContents, org/bukkit/inventory/Inventory.getStorageContents
    pub fn get_contents(&self) -> Vec<Option<ItemStack>> {
        let mut stacks: Vec<Option<ItemStack>> = query(json!({ "op": "items_ender", "uuid": self.player.uuid() })).as_array().into_iter().flatten().map(ItemStack::read).collect();
        stacks.resize(27, None);
        stacks
    }
    /// bukkit: org/bukkit/inventory/Inventory.getItem
    pub fn get_item(&self, slot: usize) -> Option<ItemStack> {
        self.get_contents().get(slot).cloned().flatten()
    }
    /// bukkit: org/bukkit/inventory/Inventory.setItem
    pub fn set_item(&self, slot: usize, stack: Option<&ItemStack>) -> bool {
        flag(&query(json!({ "op": "items_ender_set", "uuid": self.player.uuid(), "slot": slot, "stack": stack.map(ItemStack::to_json) })))
    }
    /// bukkit: org/bukkit/inventory/Inventory.setContents
    pub fn set_contents(&self, stacks: &[Option<ItemStack>]) {
        for slot in 0..27 {
            self.set_item(slot, stacks.get(slot).and_then(Option::as_ref));
        }
    }
    /// bukkit: org/bukkit/inventory/Inventory.clear
    pub fn clear(&self) {
        self.set_contents(&[]);
    }
    /// bukkit: org/bukkit/inventory/Inventory.firstEmpty
    pub fn first_empty(&self) -> i32 {
        self.get_contents().iter().position(Option::is_none).map_or(-1, |slot| slot as i32)
    }
    /// bukkit: org/bukkit/inventory/Inventory.addItem
    ///
    /// Into the stacks that have room, then the empty slots; what didn't fit comes back.
    pub fn add_item(&self, stack: &ItemStack) -> Option<ItemStack> {
        let mut left = stack.get_amount();
        let limit = stack.get_max_stack_size();
        for (slot, held) in self.get_contents().into_iter().enumerate() {
            let Some(held) = held.filter(|held| held.is_similar(stack)) else {
                continue;
            };
            let moved = (limit - held.get_amount()).clamp(0, left);
            left -= moved;
            self.set_item(slot, Some(&held.as_quantity(held.get_amount() + moved)));
        }
        while left > 0 {
            let Ok(slot) = usize::try_from(self.first_empty()) else {
                return Some(stack.as_quantity(left));
            };
            let moved = left.min(limit);
            left -= moved;
            self.set_item(slot, Some(&stack.as_quantity(moved)));
        }
        None
    }
    /// bukkit: org/bukkit/inventory/Inventory.contains
    pub fn contains(&self, material: &Material) -> bool {
        self.get_contents().iter().flatten().any(|stack| stack.get_type() == material)
    }
}

impl crate::PlayerInventory {
    /// bukkit: org/bukkit/inventory/PlayerInventory.getType, org/bukkit/inventory/Inventory.getType
    pub fn get_type(&self) -> &'static str {
        "PLAYER"
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getSize, org/bukkit/inventory/Inventory.getSize
    ///
    /// The 36 rows, the four armor slots and the off hand.
    pub fn get_size(&self) -> usize {
        41
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.firstEmpty, org/bukkit/inventory/Inventory.firstEmpty
    pub fn first_empty(&self) -> i32 {
        self.get_contents().iter().take(36).position(Option::is_none).map_or(-1, |slot| slot as i32)
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getStorageContents, org/bukkit/inventory/Inventory.getStorageContents
    pub fn get_storage_contents(&self) -> Vec<Option<ItemStack>> {
        let mut stacks = self.get_contents();
        stacks.resize(36, None);
        stacks
    }
}

// ---- Views a plugin builds ----

/// `org.bukkit.inventory.MenuType`, by the game's id: `minecraft:anvil`, `minecraft:generic_9x3`.
/// `MenuType$Typed` is the same thing here: Rust's views are chosen by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MenuType(pub String);

impl MenuType {
    /// A menu by its id or Bukkit's constant: `ANVIL`, `GENERIC_9X3`.
    pub fn of(name: &str) -> Self {
        Self(Material::of(name).key().to_owned())
    }
    /// bukkit: org/bukkit/inventory/MenuType.typed
    pub fn typed(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/inventory/MenuType$Typed.builder
    pub fn builder(&self) -> InventoryViewBuilder {
        InventoryViewBuilder { menu: self.clone(), title: None, location: None, check_reachable: false, merchant: None }
    }
    /// bukkit: org/bukkit/inventory/MenuType.create, org/bukkit/inventory/MenuType$Typed.create
    pub fn create(&self, player: &Player, title: Option<&crate::Component>) -> BuiltView {
        let mut builder = self.builder();
        if let Some(title) = title {
            builder = builder.title(title);
        }
        builder.build(player)
    }
}

/// What a merchant's screen trades with.
#[derive(Clone, Debug, PartialEq)]
pub enum MerchantSource {
    /// A villager or wandering trader in the world.
    Entity(Merchant),
    /// `Bukkit.createMerchant`: offers of the plugin's own.
    Offers(Vec<crate::core2::MerchantRecipe>),
}

/// `org.bukkit.inventory.view.builder.InventoryViewBuilder` with its location and merchant kin.
#[derive(Clone, Debug, PartialEq)]
pub struct InventoryViewBuilder {
    menu: MenuType,
    title: Option<crate::Component>,
    location: Option<Location>,
    check_reachable: bool,
    merchant: Option<MerchantSource>,
}

impl InventoryViewBuilder {
    /// bukkit: org/bukkit/inventory/view/builder/InventoryViewBuilder.copy, org/bukkit/inventory/view/builder/LocationInventoryViewBuilder.copy, org/bukkit/inventory/view/builder/MerchantInventoryViewBuilder.copy
    pub fn copy(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/inventory/view/builder/InventoryViewBuilder.title, org/bukkit/inventory/view/builder/LocationInventoryViewBuilder.title, org/bukkit/inventory/view/builder/MerchantInventoryViewBuilder.title
    pub fn title(mut self, title: &crate::Component) -> Self {
        self.title = Some(title.clone());
        self
    }
    /// bukkit: org/bukkit/inventory/view/builder/LocationInventoryViewBuilder.location
    pub fn location(mut self, location: &Location) -> Self {
        self.location = Some(location.clone());
        self
    }
    /// bukkit: org/bukkit/inventory/view/builder/LocationInventoryViewBuilder.checkReachable, org/bukkit/inventory/view/builder/MerchantInventoryViewBuilder.checkReachable
    pub fn check_reachable(mut self, check: bool) -> Self {
        self.check_reachable = check;
        self
    }
    /// bukkit: org/bukkit/inventory/view/builder/MerchantInventoryViewBuilder.merchant
    pub fn merchant(mut self, merchant: MerchantSource) -> Self {
        self.merchant = Some(merchant);
        self
    }
    /// bukkit: org/bukkit/inventory/view/builder/InventoryViewBuilder.build
    pub fn build(&self, player: &Player) -> BuiltView {
        BuiltView { builder: self.clone(), player: player.clone() }
    }
}

/// A view made and not yet opened: `InventoryView` before `open`.
#[derive(Clone, Debug, PartialEq)]
pub struct BuiltView {
    builder: InventoryViewBuilder,
    player: Player,
}

impl BuiltView {
    /// bukkit~: org/bukkit/inventory/InventoryView.open | a view built and not yet opened; see `InventoryView::open` for one that was
    pub fn open(&self) -> bool {
        let (menu, player) = (self.builder.menu.0.as_str(), &self.player);
        let at = self.builder.location.as_ref();
        let force = !self.builder.check_reachable;
        let opened = match menu {
            "minecraft:crafting" => player.open_workbench(at, force),
            "minecraft:anvil" => player.open_anvil(at, force),
            "minecraft:grindstone" => player.open_grindstone(at, force),
            "minecraft:cartography_table" => player.open_cartography_table(at, force),
            "minecraft:loom" => player.open_loom(at, force),
            "minecraft:smithing" => player.open_smithing_table(at, force),
            "minecraft:stonecutter" => player.open_stonecutter(at, force),
            "minecraft:enchantment" => player.open_enchanting(at, force),
            "minecraft:merchant" => return self.open_merchant(),
            other => return self.open_gui(other),
        };
        if let (true, Some(title)) = (opened, &self.builder.title) {
            player.get_open_inventory().set_title(title);
        }
        opened
    }
    fn open_gui(&self, menu: &str) -> bool {
        let size = match menu.trim_start_matches("minecraft:") {
            "generic_9x1" => 9,
            "generic_9x2" => 18,
            "generic_9x3" | "shulker_box" => 27,
            "generic_9x4" => 36,
            "generic_9x5" => 45,
            "generic_9x6" => 54,
            "generic_3x3" | "crafter_3x3" => 9,
            "hopper" | "brewing_stand" => 5,
            "furnace" | "blast_furnace" | "smoker" => 3,
            "beacon" | "lectern" => 1,
            _ => return false,
        };
        let title = self.builder.title.clone().unwrap_or_else(crate::Component::empty);
        let items = vec![Value::Null; size];
        flag(&query(json!({ "op": "items_open_gui", "uuid": self.player.uuid(), "menu": menu, "title": title.0, "items": items })))
    }
    fn open_merchant(&self) -> bool {
        let title = self.builder.title.as_ref().map(crate::Component::to_legacy);
        match &self.builder.merchant {
            Some(MerchantSource::Entity(merchant)) => flag(&query(json!({ "op": "items_open_merchant", "uuid": self.player.uuid(), "mob": merchant.0.get_entity_id() }))),
            Some(MerchantSource::Offers(recipes)) => {
                let rows: Vec<String> = recipes.iter().map(|recipe| crate::data::snbt(&recipe_saved(&recipe.0))).collect();
                flag(&query(json!({ "op": "items_open_merchant", "uuid": self.player.uuid(), "title": title, "offers": format!("{{Recipes:[{}]}}", rows.join(",")) })))
            }
            None => false,
        }
    }
}

// ---- Potion mixes, as the game's brewing recipes in the plugin's data pack ----

/// `io.papermc.paper.potion.PotionMix`, and `org.bukkit.inventory.BrewingRecipe`: a recipe of a brewing
/// stand, in the game's own form: an item or `#tag` list each side, by `RecipeChoice`.
#[derive(Clone, Debug, PartialEq)]
pub struct PotionMix {
    key: NamespacedKey,
    result: ItemStack,
    input: RecipeChoice,
    ingredient: RecipeChoice,
}

impl PotionMix {
    /// bukkit: org/bukkit/inventory/BrewingRecipe.<init>, io/papermc/paper/potion/PotionMix.<init>
    pub fn new(key: NamespacedKey, result: ItemStack, input: RecipeChoice, ingredient: RecipeChoice) -> Self {
        Self { key, result, input, ingredient }
    }
    /// bukkit: org/bukkit/inventory/BrewingRecipe.getKey, io/papermc/paper/potion/PotionMix.getKey
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
    /// bukkit: org/bukkit/inventory/BrewingRecipe.getResult, io/papermc/paper/potion/PotionMix.getResult
    pub fn get_result(&self) -> &ItemStack {
        &self.result
    }
    /// bukkit: org/bukkit/inventory/BrewingRecipe.getInput, io/papermc/paper/potion/PotionMix.getInput
    pub fn get_input(&self) -> &RecipeChoice {
        &self.input
    }
    /// bukkit: org/bukkit/inventory/BrewingRecipe.getIngredient, io/papermc/paper/potion/PotionMix.getIngredient
    pub fn get_ingredient(&self) -> &RecipeChoice {
        &self.ingredient
    }
}

fn mix_side(choice: &RecipeChoice) -> Value {
    if let RecipeChoice::Exact(stacks) = choice {
        return json!({ "stacks": stacks.iter().map(ItemStack::to_json).collect::<Vec<_>>() });
    }
    json!({ "items": choice.get_choices().iter().map(|stack| stack.get_type().key().to_owned()).collect::<Vec<_>>() })
}

impl PotionBrewer {
    /// bukkit: org/bukkit/potion/PotionBrewer.addPotionMix
    ///
    /// `Server.addRecipe` of a `BrewingRecipe`: at once, after the data's, and for this run of the server.
    ///
    /// # Panics
    /// As Java throws, for a key that's taken.
    pub fn add_potion_mix(&self, mix: &PotionMix) {
        let added = query(json!({ "op": "items_mix_add", "key": mix.key.as_str(), "input": mix_side(&mix.input), "reagent": mix_side(&mix.ingredient), "output": mix.result.to_json() }));
        assert!(flag(&added), "Duplicate recipe ignored with ID {}", mix.key.as_str());
    }
    /// bukkit: org/bukkit/potion/PotionBrewer.removePotionMix
    pub fn remove_potion_mix(&self, key: &NamespacedKey) {
        query(json!({ "op": "items_mix_remove", "key": key.as_str() }));
    }
    /// bukkit: org/bukkit/potion/PotionBrewer.resetPotionMixes
    pub fn reset_potion_mixes(&self) {
        query(json!({ "op": "items_mix_reset" }));
    }
}

impl PotionEffectType {
    /// bukkit: org/bukkit/potion/PotionEffectType.getDurationModifier
    ///
    /// Unused, and always this in Java: its Javadoc says so.
    pub fn get_duration_modifier(&self) -> f64 {
        1.0
    }
    /// bukkit: org/bukkit/potion/PotionEffectType.getEffectAttributes
    ///
    /// Attribute keys with the modifier the effect adds at its first level.
    pub fn get_effect_attributes(&self) -> Vec<(String, crate::scoreboard::AttributeModifier)> {
        let row = query(json!({ "op": "items_effect_attribute", "id": self.0 }));
        row["attribute"].as_str().map(|attribute| (attribute.to_owned(), crate::scoreboard::AttributeModifier::new(row["key"].as_str().unwrap_or_default(), row["amount"].as_f64().unwrap_or(0.0), row["operation"].as_str().unwrap_or_default()))).into_iter().collect()
    }
    /// bukkit: org/bukkit/potion/PotionEffectType.getAttributeModifierAmount
    ///
    /// # Panics
    /// As Java throws, for an attribute the effect doesn't change, or a negative amplifier.
    pub fn get_attribute_modifier_amount(&self, attribute: &str, amplifier: i32) -> f64 {
        assert!(amplifier >= 0, "amplifier must be positive");
        let key = Material::of(attribute).key().to_owned();
        let modifiers = self.get_effect_attributes();
        let (_, modifier) = modifiers.iter().find(|(known, _)| *known == key).unwrap_or_else(|| panic!("{attribute} isn't changed by this effect"));
        modifier.get_amount() * f64::from(amplifier + 1)
    }
}

impl crate::PotionEffect {
    /// bukkit: org/bukkit/potion/PotionEffect.withIcon
    pub fn with_icon(&self, icon: bool) -> Self {
        let mut effect = self.clone();
        effect.icon = icon;
        effect
    }
    /// bukkit: org/bukkit/potion/PotionEffect.getHiddenPotionEffect
    pub fn get_hidden_potion_effect(&self) -> Option<&Self> {
        self.hidden.as_deref()
    }
}

impl crate::core2::MerchantRecipe {
    /// bukkit: org/bukkit/inventory/MerchantRecipe.shouldIgnoreDiscounts
    pub fn should_ignore_discounts(&self) -> bool {
        self.0["ignoreDiscounts"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setIgnoreDiscounts
    ///
    /// `Merchant::set_recipes` hands it to the game, which then keeps gossip, cures and the hero's
    /// effect off this offer's price.
    pub fn set_ignore_discounts(&mut self, ignore: bool) {
        self.0["ignoreDiscounts"] = json!(ignore);
    }
}

impl SmithingInventory {
    /// bukkit: org/bukkit/inventory/SmithingInventory.getRecipe
    pub fn get_recipe(&self) -> Option<crate::core2::GameRecipe> {
        let slots: Vec<Value> = (0..3).map(|slot| self.0.slot(slot).map_or(Value::Null, |stack| stack.to_json())).collect();
        let id = query(json!({ "op": "items_smithing_recipe", "slots": slots }));
        crate::core2::get_recipe(&NamespacedKey::from_string(id.as_str()?)?)
    }
}

impl AnvilInventory {
    /// bukkit: org/bukkit/inventory/AnvilInventory.setMaximumRepairCost, org/bukkit/inventory/view/AnvilView.setMaximumRepairCost
    pub fn set_maximum_repair_cost(&self, levels: i32) {
        query(json!({ "op": "items_anvil_rule", "uuid": self.0.player.uuid(), "max": levels }));
    }
}

impl AnvilView {
    /// bukkit: org/bukkit/inventory/view/AnvilView.bypassEnchantmentLevelRestriction
    pub fn bypass_enchantment_level_restriction(&self, bypass: bool) {
        query(json!({ "op": "items_anvil_rule", "uuid": self.0.player.uuid(), "bypass": bypass }));
    }
    /// bukkit: org/bukkit/inventory/view/AnvilView.bypassesEnchantmentLevelRestriction
    pub fn bypasses_enchantment_level_restriction(&self) -> bool {
        flag(&query(json!({ "op": "items_anvil_state", "uuid": self.0.player.uuid() }))["bypass"])
    }
}

/// `org.bukkit.block.BlockType.isInteractable`: the block classes that answer a click, as Paper finds
/// them on the block's class chain.
const INTERACTABLE: [&str; 64] = [
    "AbstractBedBlock", "AbstractCauldronBlock", "AbstractFurnaceBlock", "AnvilBlock", "BarrelBlock", "BeaconBlock", "BeehiveBlock", "BellBlock", "BrewingStandBlock", "ButtonBlock", "CakeBlock", "CampfireBlock", "CandleBlock", "CandleCakeBlock", "CartographyTableBlock", "CaveVinesBlock",
    "CaveVinesPlantBlock", "CeilingHangingSignBlock", "ChestBlock", "ChiseledBookShelfBlock", "CommandBlock", "ComparatorBlock", "ComposterBlock", "CopperGolemStatueBlock", "CrafterBlock", "CraftingTableBlock", "DaylightDetectorBlock", "DecoratedPotBlock", "DispenserBlock", "DoorBlock", "DragonEggBlock", "EnchantingTableBlock",
    "EnderChestBlock", "FenceBlock", "FenceGateBlock", "FlowerPotBlock", "GrindstoneBlock", "HopperBlock", "JigsawBlock", "JukeboxBlock", "LecternBlock", "LeverBlock", "LightBlock", "LoomBlock", "NoteBlock", "PumpkinBlock", "RedStoneOreBlock", "RedstoneWireBlock",
    "RepeaterBlock", "RespawnAnchorBlock", "ShelfBlock", "ShulkerBoxBlock", "SignBlock", "SmithingTableBlock", "StonecutterBlock", "StructureBlock", "SweetBerryBushBlock", "TestBlock", "TestInstanceBlock", "TntBlock", "TrapDoorBlock", "VaultBlock", "WallHangingSignBlock", "WeatheringCopperGolemStatueBlock",
];

impl BlockType {
    /// bukkit: org/bukkit/block/BlockType.isInteractable
    pub fn is_interactable(&self) -> bool {
        self.facts()["classes"].as_array().is_some_and(|classes| classes.iter().any(|class| class.as_str().is_some_and(|class| INTERACTABLE.contains(&class))))
    }
}

impl Conduit {
    /// bukkit: org/bukkit/block/Conduit.getFrameBlocks
    pub fn get_frame_blocks(&self) -> Vec<Block> {
        self.live()["blocks"].as_array().into_iter().flatten().filter_map(|at| Some(self.0.get_world().get_block_at(at[0].as_i64()? as i32, at[1].as_i64()? as i32, at[2].as_i64()? as i32))).collect()
    }
}

/// `org.bukkit.inventory.RecipeChoice$PredicateChoice`: any test of the plugin's. A recipe in the game's
/// data can't hold one, as in Java it's only for a recipe's own checks.
#[derive(Clone)]
pub struct PredicateChoice {
    test: std::sync::Arc<dyn Fn(&ItemStack) -> bool + Send + Sync>,
    example: ItemStack,
}

impl PredicateChoice {
    /// bukkit: org/bukkit/inventory/RecipeChoice.predicateChoice
    pub fn new(test: impl Fn(&ItemStack) -> bool + Send + Sync + 'static, example: ItemStack) -> Self {
        Self { test: std::sync::Arc::new(test), example }
    }
    /// The choice's own test.
    pub fn test(&self, stack: &ItemStack) -> bool {
        (self.test)(stack)
    }
    /// The stack it shows.
    pub fn get_item_stack(&self) -> &ItemStack {
        &self.example
    }
}

/// What a craft leaves in the grid and the remainders with no room there, as `craftItemResult` works them out.
pub(crate) fn craft_rest(grid: &[Option<ItemStack>], crafted: bool) -> (Vec<Option<ItemStack>>, Vec<ItemStack>) {
    if !crafted {
        return (grid.to_vec(), Vec::new());
    }
    let slots: Vec<Value> = grid.iter().map(|stack| stack.as_ref().map_or(Value::Null, ItemStack::to_json)).collect();
    let rests = query(json!({ "op": "items_craft_rest", "grid": slots }));
    let mut matrix = Vec::with_capacity(grid.len());
    let mut overflow = Vec::new();
    for (index, stack) in grid.iter().enumerate() {
        let left = stack.as_ref().filter(|stack| stack.get_amount() > 1).map(|stack| stack.as_quantity(stack.get_amount() - 1));
        match (left, ItemStack::read(&rests[index])) {
            (left, None) => matrix.push(left),
            (None, Some(rest)) => matrix.push(Some(rest)),
            (Some(left), Some(rest)) if left.is_similar(&rest) => matrix.push(Some(rest.as_quantity(rest.get_amount() + left.get_amount()))),
            (Some(left), Some(rest)) => {
                matrix.push(Some(left));
                overflow.push(rest);
            }
        }
    }
    (matrix, overflow)
}

// ---- Recipe book groups and categories, written with the recipe ----

fn recipe_text(extra: &Map<String, Value>, field: &str, fallback: &str) -> String {
    extra.get(field).and_then(Value::as_str).unwrap_or(fallback).to_owned()
}

impl crate::extra::ShapedRecipe {
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getCategory
    pub fn get_category(&self) -> String {
        recipe_text(&self.extra, "category", "misc").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.setCategory
    pub fn set_category(&mut self, category: &str) {
        self.extra.insert("category".to_owned(), json!(category.to_ascii_lowercase()));
    }
}

impl crate::extra::ShapelessRecipe {
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getCategory
    pub fn get_category(&self) -> String {
        recipe_text(&self.extra, "category", "misc").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.setCategory
    pub fn set_category(&mut self, category: &str) {
        self.extra.insert("category".to_owned(), json!(category.to_ascii_lowercase()));
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getGroup
    pub fn get_group(&self) -> String {
        recipe_text(&self.extra, "group", "")
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.setGroup
    pub fn set_group(&mut self, group: &str) {
        self.extra.insert("group".to_owned(), json!(group));
    }
}

impl crate::extra::CookingRecipe {
    /// bukkit: org/bukkit/inventory/CookingRecipe.getCategory
    pub fn get_category(&self) -> String {
        recipe_text(&self.extra, "category", "misc").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.setCategory
    pub fn set_category(&mut self, category: &str) {
        self.extra.insert("category".to_owned(), json!(category.to_ascii_lowercase()));
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.getGroup
    pub fn get_group(&self) -> String {
        recipe_text(&self.extra, "group", "")
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.setGroup
    pub fn set_group(&mut self, group: &str) {
        self.extra.insert("group".to_owned(), json!(group));
    }
}

impl crate::extra::StonecuttingRecipe {
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.getGroup
    pub fn get_group(&self) -> String {
        recipe_text(&self.extra, "group", "")
    }
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.setGroup
    pub fn set_group(&mut self, group: &str) {
        self.extra.insert("group".to_owned(), json!(group));
    }
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.setInput
    pub fn set_input(mut self, input: Material) -> Self {
        self.input = input;
        self.extra.remove("ingredient");
        self
    }
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.setInputChoice
    pub fn set_input_choice(mut self, choice: &RecipeChoice) -> Self {
        if let Some(first) = choice.get_choices().first() {
            self.input = first.get_type().clone();
        }
        self.extra.insert("ingredient".to_owned(), choice.ingredient());
        self
    }
}

impl crate::meta::BookMeta {
    /// bukkit: org/bukkit/inventory/meta/BookMeta.spigot
    ///
    /// `BookMeta.Spigot`'s pages are on the meta itself here.
    pub fn spigot(&mut self) -> &mut Self {
        self
    }
}

// ---- A plugin's inventory holds what's put in it ----

/// What each GUI last held: a player's clicks change the one the game has, and a plugin's `Inventory`
/// value is a copy. The oldest go first, as Java's collector would have them.
static LATEST: std::sync::Mutex<BTreeMap<u32, Vec<Option<ItemStack>>>> = std::sync::Mutex::new(BTreeMap::new());

fn remember(id: u32, contents: &[Option<ItemStack>]) {
    let mut latest = LATEST.lock().unwrap_or_else(|e| e.into_inner());
    latest.insert(id, contents.to_vec());
    while latest.len() > 4096 {
        latest.pop_first();
    }
}

fn push_viewers(viewers: Vec<(String, Value)>) {
    for (uuid, items) in viewers {
        query(json!({ "op": "set_menu", "uuid": uuid, "items": items }));
    }
}

/// `open_menu` answered with the window: the game now moves stacks in it.
pub(crate) fn hold_gui(player: &Player, window: &Value) {
    query(json!({ "op": "items_gui_hold", "uuid": player.uuid(), "window": window }));
}

/// A player moved stacks in their GUI: everyone with the same inventory open sees it.
fn gui_changed(value: &Value) -> horizon_mod::Answer {
    let (Some(uuid), Some(items)) = (value["uuid"].as_str(), value["items"].as_array()) else {
        return horizon_mod::Answer::Pass;
    };
    let stacks: Vec<Option<ItemStack>> = items.iter().map(ItemStack::read).collect();
    let mut viewers = Vec::new();
    {
        let mut open = crate::OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner());
        let Some(id) = open.iter().find(|(viewer, _)| viewer.replace('-', "") == uuid).map(|(_, gui)| gui.id) else {
            return horizon_mod::Answer::Pass;
        };
        remember(id, &stacks);
        for (viewer, gui) in open.iter_mut().filter(|(_, gui)| gui.id == id) {
            gui.contents.clone_from(&stacks);
            if viewer.replace('-', "") != uuid {
                viewers.push((viewer.clone(), gui.items()));
            }
        }
    }
    push_viewers(viewers);
    horizon_mod::Answer::Pass
}

/// `Inventory.setItem`, on what's open now.
pub(crate) fn gui_slot(id: u32, slot: usize, stack: Option<&ItemStack>) {
    let mut viewers = Vec::new();
    {
        let mut open = crate::OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner());
        for (viewer, gui) in open.iter_mut().filter(|(_, gui)| gui.id == id) {
            if let Some(place) = gui.contents.get_mut(slot) {
                *place = stack.cloned();
                viewers.push((viewer.clone(), gui.items()));
            }
        }
    }
    if let Some(place) = LATEST.lock().unwrap_or_else(|e| e.into_inner()).get_mut(&id).and_then(|known| known.get_mut(slot)) {
        *place = stack.cloned();
    }
    push_viewers(viewers);
}

/// `Inventory.clear`, on what's open now.
pub(crate) fn gui_cleared(id: u32) {
    let mut viewers = Vec::new();
    {
        let mut open = crate::OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner());
        for (viewer, gui) in open.iter_mut().filter(|(_, gui)| gui.id == id) {
            gui.contents.fill(None);
            viewers.push((viewer.clone(), gui.items()));
        }
    }
    if let Some(known) = LATEST.lock().unwrap_or_else(|e| e.into_inner()).get_mut(&id) {
        known.fill(None);
    }
    push_viewers(viewers);
}

impl crate::Inventory {
    /// The same GUI as it holds now: Java has one object, this crate a value per holder, so a plugin
    /// that kept its `Inventory` while players moved stacks asks here. Events hand over the live one.
    pub fn live(&self) -> Self {
        let mut live = self.clone();
        if let Some(known) = LATEST.lock().unwrap_or_else(|e| e.into_inner()).get(&self.id) {
            live.contents.clone_from(known);
        }
        live
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/HumanEntity.openInventory
    ///
    /// `player.openInventory(target.getInventory())`: `/invsee`. A window on the real thing, both ways.
    pub fn open_player_inventory(&self, inventory: &crate::PlayerInventory) -> bool {
        self.open_live(inventory.owner(), false, &crate::adventure::legacy("Player"))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openInventory
    ///
    /// `player.openInventory(target.getEnderChest())`: `/enderchest`, of anyone. Both ways.
    pub fn open_ender_chest(&self, chest: &EnderChest) -> bool {
        self.open_live(chest.get_holder(), true, &crate::Component(json!({ "translate": "container.enderchest" })))
    }
}

/// A block's collision boxes where it stands: `BlockBehaviour.getCollisionShape` with its world around it.
pub(crate) fn collision_of(block: &Block) -> VoxelShape {
    VoxelShape(block.asked("items_collision", json!({})).as_array().into_iter().flatten().filter_map(|row| {
        let at = |index: usize| row[index].as_f64();
        Some([at(0)?, at(1)?, at(2)?, at(3)?, at(4)?, at(5)?])
    }).collect())
}

/// The class of `ItemMeta` Java hands out for an item, as `CraftItemMetas` picks it.
fn meta_class_of(material: &Material) -> &'static str {
    let name = material.key().trim_start_matches("minecraft:");
    META_CLASSES.iter().find(|(_, items)| items.contains(&name)).map_or_else(|| meta_class_by_ending(name), |(class, _)| class)
}

/// The items with a meta class of their own, as `CraftItemMetas` lists them.
const META_CLASSES: [(&str, &[&str]); 21] = [
    ("BookMeta", &["written_book"]),
    ("WritableBookMeta", &["writable_book"]),
    ("SkullMeta", &["creeper_head", "dragon_head", "piglin_head", "player_head", "skeleton_skull", "wither_skeleton_skull", "zombie_head"]),
    ("ArmorMeta", &["chainmail_helmet", "chainmail_chestplate", "chainmail_leggings", "chainmail_boots", "diamond_helmet", "diamond_chestplate", "diamond_leggings", "diamond_boots", "golden_helmet", "golden_chestplate", "golden_leggings", "golden_boots", "iron_helmet", "iron_chestplate", "iron_leggings", "iron_boots", "netherite_helmet", "netherite_chestplate", "netherite_leggings", "netherite_boots", "copper_helmet", "copper_chestplate", "copper_leggings", "copper_boots", "turtle_helmet"]),
    ("ColorableArmorMeta", &["leather_helmet", "leather_chestplate", "leather_leggings", "leather_boots", "wolf_armor"]),
    ("LeatherArmorMeta", &["leather_horse_armor"]),
    ("PotionMeta", &["potion", "splash_potion", "lingering_potion", "tipped_arrow"]),
    ("FireworkMeta", &["firework_rocket"]),
    ("FireworkEffectMeta", &["firework_star"]),
    ("EnchantmentStorageMeta", &["enchanted_book"]),
    ("ArmorStandMeta", &["armor_stand"]),
    ("KnowledgeBookMeta", &["knowledge_book"]),
    ("ShieldMeta", &["shield"]),
    ("TropicalFishBucketMeta", &["tropical_fish_bucket"]),
    ("AxolotlBucketMeta", &["axolotl_bucket"]),
    ("CrossbowMeta", &["crossbow"]),
    ("SuspiciousStewMeta", &["suspicious_stew"]),
    ("EntityTagMeta", &["cod_bucket", "pufferfish_bucket", "tadpole_bucket", "salmon_bucket", "sulfur_cube_bucket", "item_frame", "glow_item_frame", "painting"]),
    ("CompassMeta", &["compass"]),
    ("MusicInstrumentMeta", &["goat_horn"]),
    ("OminousBottleMeta", &["ominous_bottle"]),
];

/// `CraftItemMetas`' classes that go by the item's or the block's own kind.
fn meta_class_by_ending(name: &str) -> &'static str {
    if name == "filled_map" {
        return "MapMeta";
    }
    if name.ends_with("_banner") {
        return "BannerMeta";
    }
    if name.ends_with("_spawn_egg") {
        return "SpawnEggMeta";
    }
    if name == "bundle" || name.ends_with("_bundle") {
        return "BundleMeta";
    }
    if flag(&facts_of(&format!("minecraft:{name}"))["block_entity"]) {
        return "BlockStateMeta";
    }
    "ItemMeta"
}

impl crate::PlayerInventory {
    /// The game's slots directly: `/item` and `/give` wait for the end of the tick.
    fn write_slot(&self, slot: usize, stack: Option<&ItemStack>) -> bool {
        flag(&query(json!({ "op": "items_player_set", "uuid": self.owner().uuid(), "slot": slot, "stack": stack.map(ItemStack::to_json) })))
    }
    /// bukkit: org/bukkit/inventory/Inventory.addItem
    ///
    /// Into the stacks that have room, then the empty slots; what didn't fit comes back.
    pub fn add_item(&self, stack: &ItemStack) -> Option<ItemStack> {
        let left = query(json!({ "op": "items_player_add", "uuid": self.owner().uuid(), "stack": stack.to_json() })).as_i64().unwrap_or_else(|| i64::from(stack.get_amount())) as i32;
        (left > 0).then(|| stack.as_quantity(left))
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setItem, org/bukkit/inventory/Inventory.setItem
    ///
    /// Slots 0-35, then boots, leggings, chestplate and helmet, then the off hand.
    pub fn set_item(&self, slot: i32, stack: &ItemStack) {
        if let Ok(slot) = usize::try_from(slot) {
            self.write_slot(slot, Some(stack));
        }
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setItem, org/bukkit/inventory/Inventory.setItem
    ///
    /// Java's `setItem(slot, null)`.
    pub fn clear_slot(&self, slot: i32) {
        if let Ok(slot) = usize::try_from(slot) {
            self.write_slot(slot, None);
        }
    }
    /// bukkit: org/bukkit/inventory/Inventory.clear, org/bukkit/inventory/PlayerInventory.clear
    pub fn clear(&self) {
        for slot in 0..41 {
            self.write_slot(slot, None);
        }
    }
}

impl Block {
    /// bukkit: org/bukkit/block/Block.breakNaturally
    ///
    /// With `tool`, what it harvests drops; the break's particles with `effect`, or always with
    /// `force_effect`; the block's experience with `experience`. Whether it broke and dropped.
    pub fn break_naturally_with(&self, tool: Option<&ItemStack>, effect: bool, experience: bool, force_effect: bool) -> bool {
        flag(&self.asked("items_block_break", json!({ "tool": tool.map(ItemStack::to_json), "effect": effect, "experience": experience, "force": force_effect })))
    }
}
