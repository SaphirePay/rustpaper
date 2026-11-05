//! Family ev_world: see plugin-tools/tools/family.
//!
//! The events of blocks, worlds, the server, vehicles, hangings, weather and raids that `events.rs`
//! and `event2.rs` don't cover. An event here with the name of one there replaces it: it has more
//! hooks and getters, and those files' structs can't be reached from here.
#![allow(missing_docs)]
use crate::data::snbt;
use crate::entity::Entity;
use crate::event::{Event, HandlerList};
use crate::events::{
    BlockFromToEvent, BlockPhysicsEvent, ServerLoadEvent, ServerResourcesReloadedEvent,
    ServerTickStartEvent,
};
use crate::{Block, BlockState, CommandSender, Component, ItemStack, Location, Material, Player, PotionEffect, World};
use horizon_mod::{Answer, Value, query};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;

fn world_of(value: &Value) -> World {
    World::of(value["level"].as_str().or(value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
}

fn block_at(world: &World, pos: &Value) -> Option<Block> {
    let at = |i: usize| pos[i].as_i64().map(|v| v as i32);
    Some(world.get_block_at(at(0)?, at(1)?, at(2)?))
}

fn player_of(value: &Value) -> Option<Player> {
    if value["uuid"].is_null() && value["player"].is_null() {
        return None;
    }
    Player::read(&value["player"]).or_else(|| Player::read(value)).or_else(|| Player::read(&query(json!({ "op": "player", "uuid": value["uuid"] }))))
}

fn entity_at(value: &Value, key: &str) -> Option<Entity> {
    value[key].as_u64().map(|id| Entity::of(id, &world_of(value)))
}

/// A player as an entity, for the getters that return either.
fn player_entity(player: &Player) -> Option<Entity> {
    let state = query(json!({ "op": "player", "uuid": player.uuid() }));
    Entity::read(&json!({ "id": state["entity_id"], "dimension": state["dimension"], "uuid": player.uuid() }))
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_default()
}

fn face_name(face: &Value) -> &'static str {
    let at = |i: usize| face[i].as_i64().unwrap_or(0);
    match (at(0), at(1), at(2)) {
        (1, _, _) => "EAST",
        (-1, _, _) => "WEST",
        (_, 1, _) => "UP",
        (_, -1, _) => "DOWN",
        (_, _, 1) => "SOUTH",
        _ => "NORTH",
    }
}

/// `pos` + `face`, as a JSON position.
fn beside(pos: &Value, face: &Value) -> Value {
    let at = |list: &Value, i: usize| list[i].as_i64().unwrap_or(0);
    json!([at(pos, 0) + at(face, 0), at(pos, 1) + at(face, 1), at(pos, 2) + at(face, 2)])
}

fn hand_of(player: &Player, stack: Option<&ItemStack>) -> &'static str {
    let held = query(json!({ "op": "equipment", "uuid": player.uuid() }));
    let item = stack.map(|stack| stack.get_type().key().to_owned());
    if item.is_some() && held["main_hand"]["item"].as_str() != item.as_deref() && held["off_hand"]["item"].as_str() == item.as_deref() { "OFF_HAND" } else { "HAND" }
}

/// The hand of a use, as the game's hook says it.
fn hand_used(value: &Value) -> &'static str {
    if value["off_hand"] == true { "OFF_HAND" } else { "HAND" }
}

/// An event whose fields are the hook's JSON and a cancel.
macro_rules! hooked {
    ($name:ident, [$hook:literal $(, $more:literal)*] $(, $when:expr)?) => {
        impl Event for $name {
            const HOOK: &'static str = $hook;
            fn more_hooks() -> &'static [&'static str] {
                &[$($more),*]
            }
            fn read(value: &Value) -> Option<Self> {
                $(let when: fn(&Value) -> bool = $when;
                if !when(value) {
                    return None;
                })?
                Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
            }
            fn answer(&self) -> Answer {
                if self.cancelled { Answer::Fail } else { Answer::Pass }
            }
        }
    };
}

// ---- What Java's abstract classes declare, for every event under them ----

/// `org.bukkit.event.block.BlockPistonEvent`.
pub trait BlockPistonEvent {
    /// bukkit: org/bukkit/event/block/BlockPistonEvent.getDirection
    fn get_direction(&self) -> &str;
    /// bukkit: org/bukkit/event/block/BlockPistonEvent.isSticky
    fn is_sticky(&self) -> bool;
    /// bukkit: org/bukkit/event/block/BlockPistonEvent.isCancelled
    fn is_cancelled(&self) -> bool;
    /// bukkit: org/bukkit/event/block/BlockPistonEvent.setCancelled
    fn set_cancelled(&mut self, cancelled: bool);
}

/// bukkit: org/bukkit/event/block/BlockPistonExtendEvent
#[derive(Clone, Debug)]
pub struct BlockPistonExtendEvent {
    value: Value,
    cancelled: bool,
}

impl BlockPistonExtendEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockPistonExtendEvent.getBlocks
    pub fn get_blocks(&self) -> Vec<Block> {
        let world = world_of(&self.value);
        self.value["push"].as_array().into_iter().flatten().filter_map(|pos| block_at(&world, pos)).collect()
    }
}

impl BlockPistonEvent for BlockPistonExtendEvent {
    fn get_direction(&self) -> &str {
        text(&self.value, "direction")
    }
    fn is_sticky(&self) -> bool {
        self.value["sticky"].as_bool().unwrap_or(false)
    }
    fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}

impl Event for BlockPistonExtendEvent {
    const HOOK: &'static str = "piston";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "extend").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/block/BlockPistonRetractEvent
#[derive(Clone, Debug)]
pub struct BlockPistonRetractEvent {
    value: Value,
    cancelled: bool,
}

impl BlockPistonRetractEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockPistonRetractEvent.getBlocks
    pub fn get_blocks(&self) -> Vec<Block> {
        let world = world_of(&self.value);
        self.value["push"].as_array().into_iter().flatten().filter_map(|pos| block_at(&world, pos)).collect()
    }
}

impl BlockPistonEvent for BlockPistonRetractEvent {
    fn get_direction(&self) -> &str {
        text(&self.value, "direction")
    }
    fn is_sticky(&self) -> bool {
        self.value["sticky"].as_bool().unwrap_or(false)
    }
    fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
}

impl Event for BlockPistonRetractEvent {
    const HOOK: &'static str = "piston";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "retract").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

impl BlockPistonExtendEvent {
    /// bukkit: org/bukkit/event/block/BlockPistonExtendEvent.getLength
    pub fn get_length(&self) -> usize {
        self.get_blocks().len()
    }
}

impl BlockPistonRetractEvent {
    /// bukkit: org/bukkit/event/block/BlockPistonRetractEvent.getRetractLocation
    pub fn get_retract_location(&self) -> Option<Location> {
        let block = self.get_block()?;
        let (x, y, z) = match BlockPistonEvent::get_direction(self) {
            "EAST" => (1, 0, 0),
            "WEST" => (-1, 0, 0),
            "UP" => (0, 1, 0),
            "DOWN" => (0, -1, 0),
            "SOUTH" => (0, 0, 1),
            _ => (0, 0, -1),
        };
        Some(block.get_relative(x * 2, y * 2, z * 2).get_location())
    }
}

/// `org.bukkit.event.hanging.HangingEvent`.
pub trait HangingEvent {
    /// bukkit: org/bukkit/event/hanging/HangingEvent.getEntity
    fn get_entity(&self) -> Option<Entity>;
}

/// `org.bukkit.event.vehicle.VehicleEvent`.
pub trait VehicleEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleEvent.getVehicle
    fn get_vehicle(&self) -> Option<Entity>;
}

/// `org.bukkit.event.weather.WeatherEvent`.
pub trait WeatherEvent {
    /// bukkit: org/bukkit/event/weather/WeatherEvent.getWorld
    fn get_world(&self) -> World;
}


/// `org.bukkit.event.world.ChunkEvent`.
pub trait ChunkEvent {
    /// bukkit: org/bukkit/event/world/ChunkEvent.getChunk
    fn get_chunk(&self) -> &crate::entity::Chunk;
}

impl ChunkEvent for crate::event2::ChunkLoadEvent {
    fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
}

impl ChunkEvent for ChunkUnloadEvent {
    fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
}

/// bukkit: org/bukkit/event/world/ChunkUnloadEvent
#[derive(Clone, Debug)]
pub struct ChunkUnloadEvent {
    chunk: crate::entity::Chunk,
    at: Value,
    save: bool,
}

impl ChunkUnloadEvent {
    pub fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
    /// bukkit: org/bukkit/event/world/ChunkUnloadEvent.isSaveChunk
    pub fn is_save_chunk(&self) -> bool {
        self.save
    }
    /// bukkit: org/bukkit/event/world/ChunkUnloadEvent.setSaveChunk
    pub fn set_save_chunk(&mut self, save: bool) {
        self.save = save;
    }
}

impl Event for ChunkUnloadEvent {
    const HOOK: &'static str = "chunk_unload";
    fn read(value: &Value) -> Option<Self> {
        let at = |i: usize| value["chunk"][i].as_i64().map(|v| v as i32);
        let save = query(json!({ "op": "ev_world_chunk_dirty", "chunk": value["chunk"] })).as_bool().unwrap_or(true);
        Some(Self { chunk: world_of(value).get_chunk_at(at(0)?, at(1)?), at: value["chunk"].clone(), save })
    }
    fn finish(&self) {
        if !self.save {
            query(json!({ "op": "ev_world_chunk_unsaved", "chunk": self.at }));
        }
    }
}

// ---- More of what the bridge's events already carry ----

impl ServerLoadEvent {
    /// bukkit: org/bukkit/event/server/ServerLoadEvent.getType
    ///
    /// `STARTUP`: Bukkit's reload of plugins doesn't exist here, so nothing else fires this.
    pub fn get_type(&self) -> &'static str {
        crate::enums::server_load_event_load_type::STARTUP
    }
}

impl ServerTickStartEvent {
    /// bukkit: com/destroystokyo/paper/event/server/ServerTickStartEvent.getTickNumber
    pub fn get_tick_number(&self) -> i64 {
        query(json!({ "op": "tick_times" }))["ticks"].as_i64().unwrap_or(0)
    }
}


impl ServerResourcesReloadedEvent {
    /// bukkit: io/papermc/paper/event/server/ServerResourcesReloadedEvent.getCause
    pub fn get_cause(&self) -> &'static str {
        use crate::enums::server_resources_reloaded_event_cause as cause;
        if query(json!({ "op": "ev_world_free_reload_was_plugin" })) == true { cause::PLUGIN } else { cause::COMMAND }
    }
}

/// bukkit: org/bukkit/event/world/EntitiesLoadEvent
#[derive(Clone, Debug)]
pub struct EntitiesLoadEvent {
    chunk: crate::entity::Chunk,
    entities: Vec<Entity>,
}

impl ChunkEvent for EntitiesLoadEvent {
    fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
}

impl EntitiesLoadEvent {
    pub fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
    /// bukkit: org/bukkit/event/world/EntitiesLoadEvent.getEntities
    pub fn get_entities(&self) -> &[Entity] {
        &self.entities
    }
}

fn chunk_of(value: &Value) -> Option<crate::entity::Chunk> {
    let at = |i: usize| value["chunk"][i].as_i64().map(|v| v as i32);
    Some(world_of(value).get_chunk_at(at(0)?, at(1)?))
}

impl Event for EntitiesLoadEvent {
    const HOOK: &'static str = "ew_entities_load";
    fn read(value: &Value) -> Option<Self> {
        let chunk = chunk_of(value)?;
        let entities = chunk.get_entities();
        (!entities.is_empty()).then_some(Self { chunk, entities })
    }
}

/// bukkit: org/bukkit/event/world/EntitiesUnloadEvent
#[derive(Clone, Debug)]
pub struct EntitiesUnloadEvent {
    chunk: crate::entity::Chunk,
    entities: Vec<Entity>,
}

impl ChunkEvent for EntitiesUnloadEvent {
    fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
}

impl EntitiesUnloadEvent {
    pub fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
    /// bukkit: org/bukkit/event/world/EntitiesUnloadEvent.getEntities
    pub fn get_entities(&self) -> &[Entity] {
        &self.entities
    }
}

impl Event for EntitiesUnloadEvent {
    const HOOK: &'static str = "ew_entities_unload";
    fn read(value: &Value) -> Option<Self> {
        let chunk = chunk_of(value)?;
        let entities = chunk.get_entities();
        (!entities.is_empty()).then_some(Self { chunk, entities })
    }
}

impl BlockPhysicsEvent {
    /// bukkit: org/bukkit/event/block/BlockPhysicsEvent.getChangedType
    pub fn get_changed_type(&self) -> Material {
        Material::of(self.get_changed_block_data().split('[').next().unwrap_or("minecraft:air"))
    }
    /// bukkit~: org/bukkit/event/block/BlockPhysicsEvent.getSourceBlock | the block that changed: the neighbour that made it isn't known
    pub fn get_source_block(&self) -> Option<Block> {
        self.get_block()
    }
}

impl BlockFromToEvent {
    /// bukkit: org/bukkit/event/block/BlockFromToEvent.getFace
    pub fn get_face(&self) -> Option<&'static str> {
        self.get_block()?.get_face(&self.get_to_block()?)
    }
}

/// bukkit: org/bukkit/event/block/BlockDamageAbortEvent
#[derive(Clone, Debug)]
pub struct BlockDamageAbortEvent {
    value: Value,
}

impl BlockDamageAbortEvent {
    /// bukkit: org/bukkit/event/block/BlockDamageAbortEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.get_player()?.get_world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockDamageAbortEvent.getItemInHand
    pub fn get_item_in_hand(&self) -> Option<ItemStack> {
        ItemStack::read(&query(json!({ "op": "equipment", "uuid": self.get_player()?.uuid() }))["main_hand"])
    }
}

impl Event for BlockDamageAbortEvent {
    const HOOK: &'static str = "ew_dig_abort";
    fn read(value: &Value) -> Option<Self> {
        value["pos"].is_array().then(|| Self { value: value.clone() })
    }
}


/// The fire or lava that could have lit a place: fire reaches this far, and lava three above.
fn fire_near(block: &Block, names: &[&str]) -> Option<Block> {
    for (y, x, z) in [(-1, 0, 0), (1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)].into_iter().chain((-1..=4).flat_map(|y| (-1..=1).flat_map(move |x| (-1..=1).map(move |z| (y, x, z))))) {
        if (x, y, z) == (0, 0, 0) {
            continue;
        }
        let near = block.get_relative(x, y, z);
        if names.contains(&near.get_type().key()) {
            return Some(near);
        }
    }
    None
}


// ---- A block a player puts down ----

/// Where a player last used an item on a block, and what stood there, for the placing that follows.
#[derive(Clone, Debug)]
struct Click {
    hand: &'static str,
    pos: Value,
    face: Value,
    at: String,
    next: String,
    /// The blocks around where it may go, and the tick.
    around: Vec<(Value, String)>,
    tick: u64,
}

static CLICK: Mutex<Option<Click>> = Mutex::new(None);

/// bukkit: org/bukkit/event/block/BlockPlaceEvent
///
/// After the block stands, as in Paper's capture: a cancel puts back what was there, and the item.
#[derive(Clone, Debug)]
pub struct BlockPlaceEvent {
    pub player: Player,
    pub block: Block,
    pub item: Option<ItemStack>,
    click: Option<Click>,
    pos: Value,
    build: bool,
    cancelled: bool,
}

impl BlockPlaceEvent {
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.getBlockPlaced
    pub fn get_block_placed(&self) -> &Block {
        &self.block
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> &Block {
        &self.block
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.getItemInHand
    pub fn get_item_in_hand(&self) -> Option<&ItemStack> {
        self.item.as_ref()
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.getBlockAgainst
    pub fn get_block_against(&self) -> Block {
        self.click.as_ref().and_then(|click| block_at(&self.block.get_world(), &click.pos)).unwrap_or_else(|| self.block.clone())
    }
    fn replaced(&self) -> String {
        let Some(click) = &self.click else {
            return "minecraft:air".to_owned();
        };
        if self.pos == click.pos {
            return click.at.clone();
        }
        if self.pos == beside(&click.pos, &click.face) { click.next.clone() } else { "minecraft:air".to_owned() }
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.getBlockReplacedState
    pub fn get_block_replaced_state(&self) -> BlockState {
        BlockState::snapshot(self.block.clone(), &self.replaced())
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.getHand
    pub fn get_hand(&self) -> &'static str {
        self.click.as_ref().map_or("HAND", |click| click.hand)
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.canBuild
    pub fn can_build(&self) -> bool {
        self.build
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.setBuild
    pub fn set_build(&mut self, build: bool) {
        self.build = build;
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockPlaceEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockPlaceEvent {
    const HOOK: &'static str = "block_place";
    fn more_hooks() -> &'static [&'static str] {
        &["item_use_on"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        value.get("face")?;
        let player = player_of(value)?;
        let world = player.get_world();
        let (pos, face) = (value["pos"].clone(), value["face"].clone());
        let data = |pos: &Value| block_at(&world, pos).map_or_else(|| "minecraft:air".to_owned(), |block| block.get_block_data());
        let (at, next) = (data(&pos), data(&beside(&pos, &face)));
        let spot = beside(&pos, &face);
        let sides = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
        let around = [&pos, &spot].into_iter().flat_map(|from| sides.iter().map(move |side| beside(from, &json!(side)))).map(|place| {
            let state = data(&place);
            (place, state)
        }).collect();
        *CLICK.lock().unwrap_or_else(|e| e.into_inner()) = Some(Click { hand: hand_used(value), pos, face, at, next, around, tick: crate::scheduler::now() });
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        let player = player_of(value)?;
        let block = block_at(&player.get_world(), &value["pos"])?;
        let click = CLICK.lock().unwrap_or_else(|e| e.into_inner()).clone().filter(|click| click.tick == crate::scheduler::now());
        Some(Self { player, block, item: ItemStack::read(&value["stack"]), click, pos: value["pos"].clone(), build: true, cancelled: crate::event::prior_cancelled() })
    }
    fn finish(&self) {
        if self.cancelled || !self.build {
            self.block.set_block_data(&self.replaced());
            let creative = query(json!({ "op": "player", "uuid": self.player.uuid() }))["game_mode"] == 1;
            if let (Some(item), false) = (&self.item, creative) {
                let mut one = item.clone();
                one.set_amount(1);
                self.player.get_inventory().add_item(&one);
            }
        }
    }
}

/// bukkit: org/bukkit/event/block/SignChangeEvent
#[derive(Clone, Debug)]
pub struct SignChangeEvent {
    value: Value,
    lines: Vec<String>,
    changed: bool,
    cancelled: bool,
}

impl SignChangeEvent {
    /// bukkit: org/bukkit/event/block/SignChangeEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/SignChangeEvent.lines
    pub fn lines(&self) -> Vec<Component> {
        self.lines.iter().map(|line| Component::text(line)).collect()
    }
    /// bukkit: org/bukkit/event/block/SignChangeEvent.line
    pub fn line(&self, index: usize) -> Option<Component> {
        self.lines.get(index).map(|line| Component::text(line))
    }
    /// bukkit: org/bukkit/event/block/SignChangeEvent.getLines
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }
    /// bukkit: org/bukkit/event/block/SignChangeEvent.getLine
    pub fn get_line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(String::as_str)
    }
    /// bukkit: org/bukkit/event/block/SignChangeEvent.setLine
    pub fn set_line(&mut self, index: usize, line: &str) {
        if let Some(old) = self.lines.get_mut(index) {
            line.clone_into(old);
            self.changed = true;
        }
        // For the listeners after this one, who read what it wrote and aren't told it's cancelled.
        crate::event::carry("lines", json!(self.lines));
        crate::event::carry("lines_changed", json!(true));
    }
    /// A plugin's `callEvent`, not the game's sign: the caller does what comes of it.
    fn raised(&self) -> bool {
        self.value["raised"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/event/block/SignChangeEvent.getSide
    pub fn get_side(&self) -> &'static str {
        if self.value["front"].as_bool().unwrap_or(true) { "FRONT" } else { "BACK" }
    }
    /// bukkit: org/bukkit/event/block/SignChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/SignChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for SignChangeEvent {
    const HOOK: &'static str = "sign_change";
    fn read(value: &Value) -> Option<Self> {
        let lines = value["lines"].as_array()?.iter().filter_map(|line| line.as_str().map(str::to_owned)).collect();
        Some(Self { value: value.clone(), lines, changed: value["lines_changed"].as_bool().unwrap_or(false), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    /// The game writes the sign once the listeners are done, and takes these lines if there are any.
    fn finish(&self) {
        if !self.raised() {
            let lines = if self.changed && !self.cancelled { json!(self.lines) } else { Value::Null };
            query(json!({ "op": "core_sign_lines", "pos": self.value["pos"], "front": self.value["front"], "lines": lines }));
        }
    }
}

/// bukkit: org/bukkit/event/block/BlockIgniteEvent
///
/// A fire about to start. Fire spreading and lava wait for the level's run to end, as the game can be lent
/// then, and light after their listeners.
#[derive(Clone, Debug)]
pub struct BlockIgniteEvent {
    value: Value,
    cancelled: bool,
}

hooked!(BlockIgniteEvent, ["ew_ignite"]);

impl BlockIgniteEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockIgniteEvent.getCause
    pub fn get_cause(&self) -> &str {
        text(&self.value, "cause")
    }
    /// bukkit: org/bukkit/event/block/BlockIgniteEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        self.value["uuid"].is_string().then(|| player_of(&self.value)).flatten()
    }
    /// bukkit: org/bukkit/event/block/BlockIgniteEvent.getIgnitingEntity
    pub fn get_igniting_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "entity").or_else(|| player_entity(&self.get_player()?))
    }
    /// bukkit: org/bukkit/event/block/BlockIgniteEvent.getIgnitingBlock
    pub fn get_igniting_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["block"])
    }
    /// bukkit: org/bukkit/event/block/BlockIgniteEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockIgniteEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/block/BlockBurnEvent
///
/// A block about to burn: see `BlockIgniteEvent`.
#[derive(Clone, Debug)]
pub struct BlockBurnEvent {
    value: Value,
    cancelled: bool,
}

hooked!(BlockBurnEvent, ["ew_burn"]);

impl BlockBurnEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockBurnEvent.getIgnitingBlock
    pub fn get_igniting_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["block"])
    }
    /// bukkit: org/bukkit/event/block/BlockBurnEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockBurnEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}

impl<T> Pipe for T {}

/// bukkit: org/bukkit/event/block/BlockFertilizeEvent
///
/// With what the bone meal has written: taking a block off the list puts back what was there, and a cancel
/// puts back all of it and keeps the bone meal.
#[derive(Clone, Debug)]
pub struct BlockFertilizeEvent {
    value: Value,
    blocks: Vec<BlockState>,
    listed: usize,
    cancelled: bool,
}

impl BlockFertilizeEvent {
    /// bukkit: org/bukkit/event/block/BlockFertilizeEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockFertilizeEvent.getBlocks
    pub fn get_blocks(&mut self) -> &mut Vec<BlockState> {
        &mut self.blocks
    }
    /// bukkit: org/bukkit/event/block/BlockFertilizeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockFertilizeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockFertilizeEvent {
    const HOOK: &'static str = "ew_fertilize";
    fn read(value: &Value) -> Option<Self> {
        let world = world_of(value);
        let blocks: Vec<BlockState> = value["blocks"].as_array()?.iter().filter_map(|row| {
            let block = world.get_block_at(row[0].as_i64()? as i32, row[1].as_i64()? as i32, row[2].as_i64()? as i32);
            Some(BlockState::snapshot(block, row[3].as_str()?))
        }).collect();
        Some(Self { value: value.clone(), listed: blocks.len(), blocks, cancelled: value["cancelled"].as_bool() == Some(true) || crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled && self.blocks.len() != self.listed {
            let kept: Vec<[i32; 3]> = self.blocks.iter().map(|state| {
                let block = state.get_block();
                [block.get_x(), block.get_y(), block.get_z()]
            }).collect();
            query(json!({ "op": "ev_world_free_grow_kept", "blocks": kept }));
        }
    }
}

// ---- Dispensers ----

/// bukkit: org/bukkit/event/block/BlockDispenseEvent
#[derive(Clone, Debug)]
pub struct BlockDispenseEvent {
    value: Value,
    item: Option<ItemStack>,
    changed: bool,
    velocity: Option<crate::util::Vector>,
    cancelled: bool,
}

impl Event for BlockDispenseEvent {
    const HOOK: &'static str = "ew_dispense";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_dropper"]
    }
    fn read(value: &Value) -> Option<Self> {
        (!my_result(&value["stack"])).then(|| Self { value: value.clone(), item: ItemStack::read(&value["stack"]), changed: false, velocity: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.cancelled {
            return;
        }
        if let Some(velocity) = &self.velocity {
            query(json!({ "op": "ev_world_velocity", "velocity": [velocity.get_x(), velocity.get_y(), velocity.get_z()] }));
        }
        if let (true, Some(item)) = (self.changed, &self.item) {
            replace_result(item);
        }
    }
}

impl BlockDispenseEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseEvent.getItem
    pub fn get_item(&self) -> Option<&ItemStack> {
        self.item.as_ref()
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseEvent.setItem
    pub fn set_item(&mut self, item: ItemStack) {
        self.item = Some(item);
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseEvent.getVelocity
    pub fn get_velocity(&self) -> crate::util::Vector {
        self.velocity.clone().unwrap_or_else(|| {
            let at = |i: usize| self.value["velocity"][i].as_f64().unwrap_or(0.0);
            crate::util::Vector::new(at(0), at(1), at(2))
        })
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseEvent.setVelocity
    pub fn set_velocity(&mut self, velocity: crate::util::Vector) {
        self.velocity = Some(velocity);
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/block/BlockDispenseArmorEvent
#[derive(Clone, Debug)]
pub struct BlockDispenseArmorEvent {
    value: Value,
    cancelled: bool,
}

hooked!(BlockDispenseArmorEvent, ["ew_dispense_armor"], |value| !value["target"].is_null() || !value["target_uuid"].is_null());

impl BlockDispenseArmorEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseArmorEvent.getTargetEntity
    pub fn get_target_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "target").or_else(|| player_entity(&Player::read(&query(json!({ "op": "player", "uuid": self.value["target_uuid"] })))?))
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/block/BlockPreDispenseEvent
#[derive(Clone, Debug)]
pub struct BlockPreDispenseEvent {
    value: Value,
    cancelled: bool,
}

hooked!(BlockPreDispenseEvent, ["ew_dispense", "ew_dropper"]);

impl BlockPreDispenseEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/BlockPreDispenseEvent.getItemStack
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: io/papermc/paper/event/block/BlockPreDispenseEvent.getSlot
    pub fn get_slot(&self) -> i64 {
        self.value["slot"].as_i64().unwrap_or(0)
    }
    /// bukkit: io/papermc/paper/event/block/BlockPreDispenseEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/block/BlockPreDispenseEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/block/BlockFailedDispenseEvent
#[derive(Clone, Debug)]
pub struct BlockFailedDispenseEvent {
    value: Value,
    effect: bool,
}

impl BlockFailedDispenseEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/BlockFailedDispenseEvent.shouldPlayEffect
    pub fn should_play_effect(&self) -> bool {
        self.effect
    }
    pub fn set_should_play_effect(&mut self, effect: bool) {
        self.effect = effect;
    }
    /// bukkit: io/papermc/paper/event/block/BlockFailedDispenseEvent.callEvent
    ///
    /// What Java's `callEvent` answers: whether the effect plays.
    pub fn call_event(&self) -> bool {
        self.effect
    }
}

impl Event for BlockFailedDispenseEvent {
    const HOOK: &'static str = "ew_dispense_fail";
    fn read(value: &Value) -> Option<Self> {
        (value["id"] == 1001 && value["empty"].as_bool().unwrap_or(false)).then(|| Self { value: value.clone(), effect: true })
    }
    fn answer(&self) -> Answer {
        if self.effect { Answer::Pass } else { Answer::Fail }
    }
}

// ---- The server's console and completions ----

/// bukkit: org/bukkit/event/server/ServerCommandEvent
///
/// The console's line, or a command block's or a command minecart's: `get_sender().get_block()` is then the block.
#[derive(Clone, Debug)]
pub struct ServerCommandEvent {
    command: String,
    changed: bool,
    cancelled: bool,
}

impl ServerCommandEvent {
    /// bukkit: org/bukkit/event/server/ServerCommandEvent.getSender
    pub fn get_sender(&self) -> CommandSender {
        CommandSender::Console
    }
    /// bukkit: org/bukkit/event/server/ServerCommandEvent.getCommand
    pub fn get_command(&self) -> &str {
        &self.command
    }
    /// bukkit: org/bukkit/event/server/ServerCommandEvent.setCommand
    pub fn set_command(&mut self, command: &str) {
        command.clone_into(&mut self.command);
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/server/ServerCommandEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/server/ServerCommandEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ServerCommandEvent {
    const HOOK: &'static str = "ew_console";
    fn read(value: &Value) -> Option<Self> {
        if value["remote"].as_bool().unwrap_or(false) {
            return None;
        }
        Some(Self { command: value["line"].as_str()?.to_owned(), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.changed && !self.cancelled {
            query(json!({ "op": "ev_world_console_line", "line": self.command }));
        }
    }
}

/// `AsyncTabCompleteEvent.Completion`: a suggestion.
#[derive(Clone, Debug, PartialEq)]
pub struct Completion {
    suggestion: String,
    tooltip: Option<Component>,
}

impl Completion {
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent$Completion.completion
    pub fn completion(suggestion: &str) -> Self {
        Self { suggestion: suggestion.to_owned(), tooltip: None }
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent$Completion.suggestion
    pub fn suggestion(&self) -> &str {
        &self.suggestion
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent$Completion.tooltip
    pub fn tooltip(&self) -> Option<&Component> {
        self.tooltip.as_ref()
    }
    pub fn with_tooltip(mut self, tooltip: Component) -> Self {
        self.tooltip = Some(tooltip);
        self
    }
}

/// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent
#[derive(Clone, Debug)]
pub struct AsyncTabCompleteEvent {
    pub player: Option<Player>,
    buffer: String,
    completions: Vec<String>,
    tooltips: Vec<(String, Value)>,
    handled: bool,
    cancelled: bool,
}

impl AsyncTabCompleteEvent {
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.getSender
    pub fn get_sender(&self) -> Option<&Player> {
        self.player.as_ref()
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.getBuffer
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.isCommand
    pub fn is_command(&self) -> bool {
        self.buffer.starts_with('/')
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.getCompletions
    pub fn get_completions(&self) -> &[String] {
        &self.completions
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.setCompletions
    pub fn set_completions(&mut self, completions: &[&str]) {
        self.completions = completions.iter().map(|text| (*text).to_owned()).collect();
        self.handled = !self.completions.is_empty();
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.completions
    pub fn completions(&self) -> Vec<Completion> {
        let tooltip = |text: &String| self.tooltips.iter().find(|(known, _)| known == text).map(|(_, tooltip)| Component(tooltip.clone()));
        self.completions.iter().map(|text| Completion { suggestion: text.clone(), tooltip: tooltip(text) }).collect()
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.completions
    ///
    /// As `set_completions`, with their tooltips.
    pub fn set_completion_list(&mut self, completions: Vec<Completion>) {
        self.tooltips = completions.iter().filter_map(|row| Some((row.suggestion.clone(), row.tooltip.as_ref()?.0.clone()))).collect();
        self.completions = completions.into_iter().map(|row| row.suggestion).collect();
        self.handled = !self.completions.is_empty();
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.isHandled
    pub fn is_handled(&self) -> bool {
        self.handled
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.setHandled
    pub fn set_handled(&mut self, handled: bool) {
        self.handled = handled;
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.getLocation
    ///
    /// The place the sender is looking at, if any is in reach.
    pub fn get_location(&self) -> Option<Location> {
        Some(self.player.as_ref()?.get_target_block(5)?.get_location())
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/server/AsyncTabCompleteEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for AsyncTabCompleteEvent {
    const HOOK: &'static str = "tab_complete";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { player: player_of(value), buffer: text(value, "line").to_owned(), completions: Vec::new(), tooltips: Vec::new(), handled: false, cancelled: crate::event::prior_cancelled() })
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ev_world_suggest_none" }));
        } else if !self.completions.is_empty() {
            let tooltips: Vec<Value> = self.tooltips.iter().map(|(text, tooltip)| json!([text, tooltip])).collect();
            query(json!({ "op": "ev_world_tooltips", "tooltips": tooltips }));
            query(json!({ "op": "suggest", "matches": self.completions }));
        }
    }
}

/// bukkit: org/bukkit/event/server/TabCompleteEvent
///
/// The sender's completion with what plugins' completers made of it, as a player's arguments: a player's
/// command word comes from the tree, as the Javadoc says.
#[derive(Clone, Debug)]
pub struct TabCompleteEvent {
    player: Option<Player>,
    buffer: String,
    completions: Vec<String>,
    listed: Vec<String>,
    cancelled: bool,
}

impl TabCompleteEvent {
    /// bukkit: org/bukkit/event/server/TabCompleteEvent.getSender
    pub fn get_sender(&self) -> Option<&Player> {
        self.player.as_ref()
    }
    /// bukkit: org/bukkit/event/server/TabCompleteEvent.getBuffer
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }
    /// bukkit: org/bukkit/event/server/TabCompleteEvent.isCommand
    pub fn is_command(&self) -> bool {
        self.buffer.starts_with('/')
    }
    /// bukkit: org/bukkit/event/server/TabCompleteEvent.getLocation
    ///
    /// The place the sender is looking at, if any is in reach.
    pub fn get_location(&self) -> Option<Location> {
        Some(self.player.as_ref()?.get_target_block(5)?.get_location())
    }
    /// bukkit: org/bukkit/event/server/TabCompleteEvent.getCompletions
    pub fn get_completions(&self) -> &[String] {
        &self.completions
    }
    /// bukkit: org/bukkit/event/server/TabCompleteEvent.setCompletions
    pub fn set_completions(&mut self, completions: Vec<String>) {
        self.completions = completions;
    }
    /// bukkit: org/bukkit/event/server/TabCompleteEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/server/TabCompleteEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for TabCompleteEvent {
    const HOOK: &'static str = "ew_tab_complete";
    fn read(value: &Value) -> Option<Self> {
        let completions: Vec<String> = value["matches"].as_array()?.iter().filter_map(Value::as_str).map(str::to_owned).collect();
        Some(Self { player: player_of(value), buffer: text(value, "line").to_owned(), listed: completions.clone(), completions, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled && self.completions != self.listed {
            query(json!({ "op": "ev_world_tab_matches", "matches": self.completions }));
            crate::event::carry("matches", json!(self.completions));
        }
    }
}

// ---- Game events ----

/// bukkit: org/bukkit/event/world/GenericGameEvent
#[derive(Clone, Debug)]
pub struct GenericGameEvent {
    value: Value,
    radius: Option<i64>,
    cancelled: bool,
}

impl Event for GenericGameEvent {
    const HOOK: &'static str = "ew_game_event";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), radius: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let (Some(radius), false) = (self.radius, self.cancelled) {
            query(json!({ "op": "ev_world_radius", "radius": radius }));
        }
    }
}

impl GenericGameEvent {
    /// bukkit: org/bukkit/event/world/GenericGameEvent.getEvent
    ///
    /// The game event's key: `minecraft:step`.
    pub fn get_event(&self) -> &str {
        text(&self.value, "event")
    }
    /// bukkit: org/bukkit/event/world/GenericGameEvent.getLocation
    pub fn get_location(&self) -> Option<Location> {
        let at = |i: usize| self.value["at"][i].as_f64();
        Some(Location::new(world_of(&self.value), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/event/world/GenericGameEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "entity").or_else(|| player_entity(&player_of(&self.value)?))
    }
    /// bukkit: org/bukkit/event/world/GenericGameEvent.getRadius
    pub fn get_radius(&self) -> i64 {
        self.radius.unwrap_or_else(|| self.value["radius"].as_i64().unwrap_or(0))
    }
    /// bukkit: org/bukkit/event/world/GenericGameEvent.setRadius
    pub fn set_radius(&mut self, radius: i64) {
        self.radius = Some(radius.max(0));
    }
    /// bukkit: org/bukkit/event/world/GenericGameEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/world/GenericGameEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/block/BlockReceiveGameEvent
#[derive(Clone, Debug)]
pub struct BlockReceiveGameEvent {
    value: Value,
    event: Option<String>,
    cancelled: bool,
}

impl Event for BlockReceiveGameEvent {
    const HOOK: &'static str = "ew_heard";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), event: value["heard"].as_str().map(str::to_owned), cancelled: value["ignored"] == true || crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

impl BlockReceiveGameEvent {
    /// bukkit: org/bukkit/event/block/BlockReceiveGameEvent.getEvent
    ///
    /// The game event's key: `minecraft:step`.
    pub fn get_event(&self) -> Option<&str> {
        self.event.as_deref()
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockReceiveGameEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "entity").or_else(|| player_entity(&player_of(&self.value)?))
    }
    /// bukkit: org/bukkit/event/block/BlockReceiveGameEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockReceiveGameEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/block/SculkBloomEvent
#[derive(Clone, Debug)]
pub struct SculkBloomEvent {
    value: Value,
    charge: i64,
    cancelled: bool,
}

impl SculkBloomEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/SculkBloomEvent.getCharge
    pub fn get_charge(&self) -> i64 {
        self.charge
    }
    /// bukkit: org/bukkit/event/block/SculkBloomEvent.setCharge
    pub fn set_charge(&mut self, charge: i64) {
        self.charge = charge.max(0);
    }
    /// bukkit: org/bukkit/event/block/SculkBloomEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/SculkBloomEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for SculkBloomEvent {
    const HOOK: &'static str = "ew_bloom_charge";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), charge: value["charge"].as_i64()?, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled {
            query(json!({ "op": "ev_world_charge", "charge": self.charge }));
        }
    }
}

// ---- TNT ----


/// bukkit: org/bukkit/event/block/TNTPrimeEvent, com/destroystokyo/paper/event/block/TNTPrimeEvent
///
/// Before the fuse is lit. Fire and redstone: see `BlockIgniteEvent`.
#[derive(Clone, Debug)]
pub struct TNTPrimeEvent {
    value: Value,
    cancelled: bool,
}

hooked!(TNTPrimeEvent, ["ew_tnt"]);

impl TNTPrimeEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/TNTPrimeEvent.getPrimingBlock
    pub fn get_priming_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["block"])
    }
    /// bukkit: org/bukkit/event/block/TNTPrimeEvent.getCause
    pub fn get_cause(&self) -> &str {
        text(&self.value, "cause")
    }
    /// bukkit: com/destroystokyo/paper/event/block/TNTPrimeEvent.getReason
    pub fn get_reason(&self) -> &'static str {
        use crate::enums::tnt_prime_event_prime_reason as reason;
        match self.get_cause() {
            "PLAYER" | "DISPENSER" => reason::ITEM,
            "EXPLOSION" => reason::EXPLOSION,
            "FIRE" => reason::FIRE,
            "PROJECTILE" => reason::PROJECTILE,
            _ => reason::REDSTONE,
        }
    }
    /// bukkit: org/bukkit/event/block/TNTPrimeEvent.getPrimingEntity
    pub fn get_priming_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "entity").or_else(|| player_entity(&self.get_player()?))
    }
    /// bukkit: com/destroystokyo/paper/event/block/TNTPrimeEvent.getPrimerEntity
    pub fn get_primer_entity(&self) -> Option<Entity> {
        self.get_priming_entity()
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        self.value["uuid"].is_string().then(|| player_of(&self.value)).flatten()
    }
    /// bukkit: com/destroystokyo/paper/event/block/TNTPrimeEvent.setCancelled, org/bukkit/event/block/TNTPrimeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/block/TNTPrimeEvent.isCancelled, org/bukkit/event/block/TNTPrimeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Vehicles: minecarts and boats ----

macro_rules! vehicle {
    ($name:ident) => {
        impl VehicleEvent for $name {
            fn get_vehicle(&self) -> Option<Entity> {
                entity_at(&self.value, "vehicle")
            }
        }
    };
}

/// A hit a listener changed, on its way back through the same hook.
static MY_HIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// bukkit~: org/bukkit/event/vehicle/VehicleDamageEvent | a fire-aspect hit doesn't name its attacker; the integrated server's host isn't named either
#[derive(Clone, Debug)]
pub struct VehicleDamageEvent {
    value: Value,
    damage: Option<f64>,
    cancelled: bool,
}

vehicle!(VehicleDamageEvent);

impl VehicleDamageEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleDamageEvent.getAttacker
    pub fn get_attacker(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleDamageEvent.getDamage
    pub fn get_damage(&self) -> f64 {
        self.damage.unwrap_or_else(|| self.value["damage"].as_f64().unwrap_or(0.0))
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleDamageEvent.setDamage
    pub fn set_damage(&mut self, damage: f64) {
        self.damage = Some(damage);
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleDamageEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleDamageEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for VehicleDamageEvent {
    const HOOK: &'static str = "ew_cart_hurt";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_boat_hurt"]
    }
    fn read(value: &Value) -> Option<Self> {
        (!MY_HIT.load(std::sync::atomic::Ordering::Relaxed)).then(|| Self { value: value.clone(), damage: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.damage.is_some() { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let (Some(damage), false) = (self.damage, self.cancelled) {
            MY_HIT.store(true, std::sync::atomic::Ordering::Relaxed);
            query(json!({ "op": "ev_world_vehicle_hit", "vehicle": self.value["vehicle"], "damage": damage, "creative": self.value["creative"], "fire": self.value["fire"] }));
            MY_HIT.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// bukkit: org/bukkit/event/vehicle/VehicleDestroyEvent
#[derive(Clone, Debug)]
pub struct VehicleDestroyEvent {
    value: Value,
    cancelled: bool,
}

vehicle!(VehicleDestroyEvent);

impl Event for VehicleDestroyEvent {
    const HOOK: &'static str = "ew_cart_hurt";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_boat_hurt"]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["breaks"] == true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ev_world_vehicle_worn", "vehicle": self.value["vehicle"] }));
        }
    }
}

impl VehicleDestroyEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleDestroyEvent.getAttacker
    pub fn get_attacker(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleDestroyEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleDestroyEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/vehicle/VehicleEnterEvent
#[derive(Clone, Debug)]
pub struct VehicleEnterEvent {
    value: Value,
    cancelled: bool,
}

hooked!(VehicleEnterEvent, ["ew_vehicle_enter"]);
vehicle!(VehicleEnterEvent);

impl VehicleEnterEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleEnterEvent.getEntered
    pub fn get_entered(&self) -> Option<Entity> {
        entity_at(&self.value, "entity").or_else(|| player_entity(&player_of(&self.value)?))
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleEnterEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleEnterEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/vehicle/VehicleExitEvent
#[derive(Clone, Debug)]
pub struct VehicleExitEvent {
    value: Value,
    cancelled: bool,
}

hooked!(VehicleExitEvent, ["ew_cart_exit", "ew_boat_exit", "ew_vehicle_exit"]);
vehicle!(VehicleExitEvent);

impl VehicleExitEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleExitEvent.getExited
    pub fn get_exited(&self) -> Option<Entity> {
        entity_at(&self.value, "entity").or_else(|| player_entity(&player_of(&self.value)?))
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleExitEvent.isCancellable
    ///
    /// Whether a cancel counts: not if the vehicle is going away.
    pub fn is_cancellable(&self) -> bool {
        self.value["cancellable"] != false
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleExitEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleExitEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/vehicle/VehicleCreateEvent
///
/// Once the vehicle is in the level: a cancel takes it out again.
#[derive(Clone, Debug)]
pub struct VehicleCreateEvent {
    value: Value,
    cancelled: bool,
}

hooked!(VehicleCreateEvent, ["ew_vehicle_created"]);
vehicle!(VehicleCreateEvent);

impl VehicleCreateEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleCreateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleCreateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/vehicle/VehicleMoveEvent
#[derive(Clone, Debug)]
pub struct VehicleMoveEvent {
    value: Value,
    from: Location,
    to: Location,
}

vehicle!(VehicleMoveEvent);

impl VehicleMoveEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleMoveEvent.getFrom
    pub fn get_from(&self) -> &Location {
        &self.from
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleMoveEvent.getTo
    pub fn get_to(&self) -> &Location {
        &self.to
    }
}

impl Event for VehicleMoveEvent {
    const HOOK: &'static str = "ew_vehicle_tick";
    fn read(value: &Value) -> Option<Self> {
        if value["moved"] != true {
            return None;
        }
        let world = world_of(value);
        let place = |key: &str| {
            let at = |i: usize| value[key][i].as_f64();
            Some(Location::new(world.clone(), at(0)?, at(1)?, at(2)?, at(3)? as f32, at(4)? as f32))
        };
        Some(Self { value: value.clone(), from: place("from")?, to: place("to")? })
    }
}

/// bukkit: org/bukkit/event/vehicle/VehicleUpdateEvent
#[derive(Clone, Debug)]
pub struct VehicleUpdateEvent {
    value: Value,
}

vehicle!(VehicleUpdateEvent);

impl Event for VehicleUpdateEvent {
    const HOOK: &'static str = "ew_vehicle_tick";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

// ---- Paintings and item frames ----

macro_rules! hanging {
    ($name:ident) => {
        impl HangingEvent for $name {
            fn get_entity(&self) -> Option<Entity> {
                entity_at(&self.value, "entity")
            }
        }
    };
}

/// bukkit: org/bukkit/event/hanging/HangingPlaceEvent
#[derive(Clone, Debug)]
pub struct HangingPlaceEvent {
    value: Value,
    cancelled: bool,
}

hooked!(HangingPlaceEvent, ["ew_hanging_place"]);
hanging!(HangingPlaceEvent);

impl HangingPlaceEvent {
    /// bukkit: org/bukkit/event/hanging/HangingPlaceEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/hanging/HangingPlaceEvent.getItemStack
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/hanging/HangingPlaceEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.get_player()?.get_world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/hanging/HangingPlaceEvent.getBlockFace
    pub fn get_block_face(&self) -> &'static str {
        face_name(&self.value["face"])
    }
    /// bukkit: org/bukkit/event/hanging/HangingPlaceEvent.getHand
    pub fn get_hand(&self) -> &'static str {
        hand_used(&self.value)
    }
    /// bukkit: org/bukkit/event/hanging/HangingPlaceEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/hanging/HangingPlaceEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit~: org/bukkit/event/hanging/HangingBreakEvent | paintings, and a player taking an empty frame down
#[derive(Clone, Debug)]
pub struct HangingBreakEvent {
    value: Value,
    cancelled: bool,
}

hooked!(HangingBreakEvent, ["ew_painting_break", "ew_frame_attack", "ee_decor", "ee_blast_end"], |value| value["entity"].is_number() && (value["ee_blast"] == true || value["survives"] != true || value["by_player"] == true));
hanging!(HangingBreakEvent);

impl HangingBreakEvent {
    pub fn is_by_player(&self) -> bool {
        self.value["by_player"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/event/hanging/HangingBreakEvent.getCause
    ///
    /// `ENTITY` for a player; a painting that still fits its wall was blown off it, else `PHYSICS`.
    pub fn get_cause(&self) -> &'static str {
        use crate::enums::hanging_break_event_remove_cause as cause;
        if self.is_by_player() {
            cause::ENTITY
        } else if self.value["survives"].as_bool().unwrap_or(false) {
            cause::EXPLOSION
        } else {
            cause::PHYSICS
        }
    }
    /// bukkit: org/bukkit/event/hanging/HangingBreakEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/hanging/HangingBreakEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

fn value_of_remover(value: &Value) -> Option<&Value> {
    (value["ee_blast"] == true).then(|| &value["remover"])
}

/// bukkit~: org/bukkit/event/hanging/HangingBreakByEntityEvent | a player, or what blew it down: a mob or a projectile doesn't break one any other way here
#[derive(Clone, Debug)]
pub struct HangingBreakByEntityEvent {
    value: Value,
    cancelled: bool,
}

hooked!(HangingBreakByEntityEvent, ["ew_painting_break", "ew_frame_attack", "ee_decor", "ee_blast_end", "ee_blast_shot", "ee_blast_tnt", "ee_blast_cart", "ee_blast_crystal", "ee_mob"], |value| value["entity"].is_number() && (value["by_player"].as_bool().unwrap_or(false) || value["ee_blast"] == true && value["remover"].is_object()));
hanging!(HangingBreakByEntityEvent);

impl HangingBreakByEntityEvent {
    /// bukkit: org/bukkit/event/hanging/HangingBreakByEntityEvent.getRemover
    pub fn get_remover(&self) -> Option<Entity> {
        if let Some(remover) = value_of_remover(&self.value) {
            return remover["id"].as_u64().filter(|id| *id != 0).map(|id| Entity::of(id, &world_of(&self.value)));
        }
        player_entity(&player_of(&self.value)?)
    }
    /// The game's id of what blew it down, when something did: most of those have left the world by now.
    pub fn get_remover_type(&self) -> Option<&str> {
        value_of_remover(&self.value)?["kind"].as_str()
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Weather ----

/// bukkit: org/bukkit/event/weather/WeatherChangeEvent
///
/// The sky reads as it was until the listeners are done.
#[derive(Clone, Debug)]
pub struct WeatherChangeEvent {
    world: World,
    raining: bool,
    cause: String,
    cancelled: bool,
}

impl WeatherChangeEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> &World {
        &self.world
    }
    /// bukkit: org/bukkit/event/weather/WeatherChangeEvent.toWeatherState
    pub fn to_weather_state(&self) -> bool {
        self.raining
    }
    /// bukkit: org/bukkit/event/weather/WeatherChangeEvent.getCause
    pub fn get_cause(&self) -> &str {
        &self.cause
    }
    /// bukkit: org/bukkit/event/weather/WeatherChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/weather/WeatherChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WeatherChangeEvent {
    const HOOK: &'static str = "ew_weather";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { world: world_of(value), raining: value["raining"].as_bool()?, cause: text(value, "cause").to_owned(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/weather/ThunderChangeEvent
///
/// The sky reads as it was until the listeners are done.
#[derive(Clone, Debug)]
pub struct ThunderChangeEvent {
    world: World,
    thundering: bool,
    cause: String,
    cancelled: bool,
}

impl ThunderChangeEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> &World {
        &self.world
    }
    /// bukkit: org/bukkit/event/weather/ThunderChangeEvent.toThunderState
    pub fn to_thunder_state(&self) -> bool {
        self.thundering
    }
    /// bukkit: org/bukkit/event/weather/ThunderChangeEvent.getCause
    pub fn get_cause(&self) -> &str {
        &self.cause
    }
    /// bukkit: org/bukkit/event/weather/ThunderChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/weather/ThunderChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ThunderChangeEvent {
    const HOOK: &'static str = "ew_thunder";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { world: world_of(value), thundering: value["thundering"].as_bool()?, cause: text(value, "cause").to_owned(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

// ---- Blocks with a state of their own ----

/// bukkit: io/papermc/paper/event/block/VaultChangeStateEvent
#[derive(Clone, Debug)]
pub struct VaultChangeStateEvent {
    value: Value,
    cancelled: bool,
}

hooked!(VaultChangeStateEvent, ["ew_vault_state"]);

impl VaultChangeStateEvent {
    /// bukkit: io/papermc/paper/event/block/VaultChangeStateEvent.getPlayer
    ///
    /// The player whose key did it: `None` for what the vault's own tick changes.
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/VaultChangeStateEvent.getCurrentState
    pub fn get_current_state(&self) -> String {
        text(&self.value, "old").to_ascii_uppercase()
    }
    /// bukkit: io/papermc/paper/event/block/VaultChangeStateEvent.getNewState
    pub fn get_new_state(&self) -> String {
        text(&self.value, "new").to_ascii_uppercase()
    }
    /// bukkit: io/papermc/paper/event/block/VaultChangeStateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/block/VaultChangeStateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// `com.destroystokyo.paper.event.block.AnvilDamagedEvent$DamageState`, with the block each is.
pub mod anvil_damage_state {
    use crate::Material;

    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent$DamageState.getMaterial
    pub fn get_material(state: &str) -> Material {
        Material::of(match state {
            "FULL" => "anvil",
            "CHIPPED" => "chipped_anvil",
            "DAMAGED" => "damaged_anvil",
            _ => "air",
        })
    }
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent$DamageState.getState
    pub fn get_state(block_data: &str) -> Option<&'static str> {
        match block_data.split('[').next().unwrap_or_default().trim_start_matches("minecraft:") {
            "anvil" => Some("FULL"),
            "chipped_anvil" => Some("CHIPPED"),
            "damaged_anvil" => Some("DAMAGED"),
            _ => None,
        }
    }
}

/// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent
#[derive(Clone, Debug)]
pub struct AnvilDamagedEvent {
    value: Value,
    state: &'static str,
    breaking: bool,
    changed: bool,
    cancelled: bool,
}

impl AnvilDamagedEvent {
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent.getInventory
    pub fn get_inventory(&self) -> Option<crate::fam_items::AnvilInventory> {
        Some(crate::fam_items::AnvilInventory::of(&player_of(&self.value)?))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent.getDamageState
    ///
    /// What the anvil is about to be.
    pub fn get_damage_state(&self) -> &'static str {
        self.state
    }
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent.setDamageState
    pub fn set_damage_state(&mut self, state: &'static str) {
        self.state = state;
        self.breaking = state == "BROKEN";
        self.changed = true;
    }
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent.isBreaking
    pub fn is_breaking(&self) -> bool {
        self.breaking
    }
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent.setBreaking
    pub fn set_breaking(&mut self, breaking: bool) {
        self.breaking = breaking;
        self.changed = true;
    }
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for AnvilDamagedEvent {
    const HOOK: &'static str = "ew_anvil";
    fn read(value: &Value) -> Option<Self> {
        // AnvilMenu's 12% roll, seen before the game makes it.
        if value["creative"].as_bool().unwrap_or(true) || value["roll"].as_f64().unwrap_or(1.0) >= 0.12 {
            return None;
        }
        let block = block_at(&world_of(value), &value["pos"])?;
        let state = match anvil_damage_state::get_state(block.get_type().key())? {
            "FULL" => "CHIPPED",
            "CHIPPED" => "DAMAGED",
            _ => "BROKEN",
        };
        Some(Self { value: value.clone(), state, breaking: state == "BROKEN", changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        let Some(block) = self.get_block().filter(|_| self.changed && !self.cancelled) else {
            return;
        };
        if self.breaking {
            block.set_type(&Material::of("air"));
            return;
        }
        let data = block.get_block_data();
        let properties = data.find('[').map_or("", |at| &data[at..]);
        block.set_block_data(&format!("{}{properties}", anvil_damage_state::get_material(self.state).key()));
    }
}

// ---- World settings a command changes ----

/// A change a listener made, going back through the same command: it isn't an event again.
static MY_COMMAND: Mutex<Option<String>> = Mutex::new(None);

fn mine(line: &str) -> bool {
    MY_COMMAND.lock().unwrap_or_else(|e| e.into_inner()).take_if(|mine| mine == line).is_some()
}

fn run_mine(line: String) {
    horizon_mod::run_command(&line);
    *MY_COMMAND.lock().unwrap_or_else(|e| e.into_inner()) = Some(line);
}

/// bukkit: io/papermc/paper/event/world/WorldGameRuleChangeEvent
#[derive(Clone, Debug)]
pub struct WorldGameRuleChangeEvent {
    value: Value,
    rule: String,
    new: String,
    changed: bool,
    cancelled: bool,
}

impl WorldGameRuleChangeEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/world/WorldGameRuleChangeEvent.getGameRule
    ///
    /// The rule's name.
    pub fn get_game_rule(&self) -> &str {
        &self.rule
    }
    /// bukkit: io/papermc/paper/event/world/WorldGameRuleChangeEvent.getValue
    pub fn get_value(&self) -> &str {
        &self.new
    }
    /// bukkit: io/papermc/paper/event/world/WorldGameRuleChangeEvent.setValue
    pub fn set_value(&mut self, value: &str) {
        value.clone_into(&mut self.new);
        self.changed = true;
    }
    /// bukkit: io/papermc/paper/event/world/WorldGameRuleChangeEvent.getCommandSender
    pub fn get_command_sender(&self) -> Option<CommandSender> {
        Some(player_of(&self.value).map_or(CommandSender::Console, CommandSender::Player))
    }
    /// bukkit: io/papermc/paper/event/world/WorldGameRuleChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/world/WorldGameRuleChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WorldGameRuleChangeEvent {
    const HOOK: &'static str = "ew_game_rule";
    fn read(value: &Value) -> Option<Self> {
        let (rule, new) = (value["rule"].as_str()?, value["value"].as_str()?);
        if mine(&format!("gamerule {rule} {new}")) {
            return None;
        }
        Some(Self { value: value.clone(), rule: rule.to_owned(), new: new.to_owned(), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.changed && !self.cancelled {
            run_mine(format!("gamerule {} {}", self.rule, self.new));
        }
    }
}

/// bukkit: org/bukkit/event/world/SpawnChangeEvent
#[derive(Clone, Debug)]
pub struct SpawnChangeEvent {
    value: Value,
}

impl SpawnChangeEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: org/bukkit/event/world/SpawnChangeEvent.getPreviousLocation
    pub fn get_previous_location(&self) -> Option<Location> {
        let at = |i: usize| self.value["previous"][i].as_f64();
        Some(Location::new(self.get_world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
}

impl Event for SpawnChangeEvent {
    const HOOK: &'static str = "ew_spawn_changed";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

/// bukkit: com/destroystokyo/paper/event/server/WhitelistToggleEvent
#[derive(Clone, Debug)]
pub struct WhitelistToggleEvent {
    value: Value,
    cancelled: bool,
}

hooked!(WhitelistToggleEvent, ["ew_whitelist"]);

impl WhitelistToggleEvent {
    /// bukkit: com/destroystokyo/paper/event/server/WhitelistToggleEvent.isEnabled
    pub fn is_enabled(&self) -> bool {
        self.value["on"].as_bool().unwrap_or(false)
    }
}

// ---- The world border ----

/// `io.papermc.paper.event.world.border.WorldBorderEvent`.
pub trait WorldBorderEvent {
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderEvent.getWorldBorder
    fn get_world_border(&self) -> crate::world::WorldBorder;
}

/// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent
#[derive(Clone, Debug)]
pub struct WorldBorderBoundsChangeEvent {
    value: Value,
    size: f64,
    ticks: i64,
    changed: bool,
    cancelled: bool,
}

impl WorldBorderEvent for WorldBorderBoundsChangeEvent {
    fn get_world_border(&self) -> crate::world::WorldBorder {
        world_of(&self.value).get_world_border()
    }
}

impl WorldBorderBoundsChangeEvent {
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.getType
    pub fn get_type(&self) -> &'static str {
        if self.ticks > 0 { "STARTED_MOVE" } else { "INSTANT_MOVE" }
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.getOldSize
    pub fn get_old_size(&self) -> f64 {
        self.value["old"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.getNewSize
    pub fn get_new_size(&self) -> f64 {
        self.size
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.setNewSize
    pub fn set_new_size(&mut self, size: f64) {
        self.size = size;
        self.changed = true;
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.getDurationTicks
    pub fn get_duration_ticks(&self) -> i64 {
        self.ticks
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.setDurationTicks
    pub fn set_duration_ticks(&mut self, ticks: i64) {
        self.ticks = ticks;
        self.changed = true;
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.getDuration
    ///
    /// In milliseconds.
    pub fn get_duration(&self) -> i64 {
        self.ticks * 50
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.setDuration
    pub fn set_duration(&mut self, millis: i64) {
        self.set_duration_ticks(millis / 50);
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WorldBorderBoundsChangeEvent {
    const HOOK: &'static str = "ew_border";
    fn read(value: &Value) -> Option<Self> {
        let old = value["old"].as_f64()?;
        let distance = value["distance"].as_f64()?;
        let size = match text(value, "id") {
            "worldborder.set" => distance,
            "worldborder.add" => old + distance,
            _ => return None,
        };
        let ticks = value["time"].as_i64().unwrap_or(0);
        if mine(&format!("worldborder set {size} {ticks}")) {
            return None;
        }
        Some(Self { value: value.clone(), size, ticks, changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.changed && !self.cancelled {
            run_mine(format!("worldborder set {} {}", self.size, self.ticks));
        }
    }
}

/// bukkit: io/papermc/paper/event/world/border/WorldBorderCenterChangeEvent
#[derive(Clone, Debug)]
pub struct WorldBorderCenterChangeEvent {
    value: Value,
    new: Option<Location>,
    cancelled: bool,
}

impl WorldBorderEvent for WorldBorderCenterChangeEvent {
    fn get_world_border(&self) -> crate::world::WorldBorder {
        world_of(&self.value).get_world_border()
    }
}

impl WorldBorderCenterChangeEvent {
    fn place(&self, key: &str) -> Option<Location> {
        Some(Location::new(world_of(&self.value), self.value[key][0].as_f64()?, 0.0, self.value[key][1].as_f64()?, 0.0, 0.0))
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderCenterChangeEvent.getOldCenter
    pub fn get_old_center(&self) -> Option<Location> {
        self.place("old_center")
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderCenterChangeEvent.getNewCenter
    pub fn get_new_center(&self) -> Option<Location> {
        self.new.clone().or_else(|| self.place("new_center"))
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderCenterChangeEvent.setNewCenter
    pub fn set_new_center(&mut self, center: &Location) {
        self.new = Some(center.clone());
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderCenterChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderCenterChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WorldBorderCenterChangeEvent {
    const HOOK: &'static str = "ew_border";
    fn read(value: &Value) -> Option<Self> {
        if text(value, "id") != "worldborder.center" {
            return None;
        }
        let at = |i: usize| value["new_center"][i].as_f64().unwrap_or(0.0);
        if mine(&format!("worldborder center {} {}", at(0), at(1))) {
            return None;
        }
        Some(Self { value: value.clone(), new: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.new.is_some() { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let (Some(new), false) = (&self.new, self.cancelled) {
            run_mine(format!("worldborder center {} {}", new.get_x(), new.get_z()));
        }
    }
}

// ---- Raids, plugins and chunks ----

/// bukkit: org/bukkit/event/raid/RaidFinishEvent
#[derive(Clone, Debug)]
pub struct RaidFinishEvent {
    value: Value,
    cancelled: bool,
}

hooked!(RaidFinishEvent, ["ew_raid_won"], |value| value["winners"].is_array());

impl RaidFinishEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: org/bukkit/event/raid/RaidFinishEvent.getWinners
    pub fn get_winners(&self) -> Vec<Player> {
        self.value["winners"].as_array().into_iter().flatten().filter_map(|uuid| Player::read(&query(json!({ "op": "player", "uuid": uuid })))).collect()
    }
}

/// The plugins from this one on, in the order they load: those whose enabling it was there to hear.
fn plugins_from_here() -> Vec<String> {
    let names: Vec<String> = crate::fam_core::get_plugin_manager().get_plugins().iter().map(|plugin| plugin.get_name()).collect();
    let me = crate::fam_core::own_plugin().map(|plugin| plugin.get_name().to_owned()).unwrap_or_default();
    let from = names.iter().position(|name| *name == me).unwrap_or(0);
    names.into_iter().skip(from).collect()
}

/// bukkit: org/bukkit/event/server/PluginEnableEvent
#[derive(Clone, Debug)]
pub struct PluginEnableEvent {
    name: String,
}

impl Event for PluginEnableEvent {
    const HOOK: &'static str = "server_started";
    fn more_hooks() -> &'static [&'static str] {
        &["core_plugin_enable"]
    }
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        match value["name"].as_str() {
            Some(name) => vec![Self { name: name.to_owned() }],
            None => plugins_from_here().into_iter().map(|name| Self { name }).collect(),
        }
    }
}

/// bukkit: org/bukkit/event/server/PluginDisableEvent
#[derive(Clone, Debug)]
pub struct PluginDisableEvent {
    name: String,
}

impl Event for PluginDisableEvent {
    const HOOK: &'static str = "server_stopping";
    fn more_hooks() -> &'static [&'static str] {
        &["core_plugin_disable"]
    }
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        match value["name"].as_str() {
            Some(name) => vec![Self { name: name.to_owned() }],
            None => plugins_from_here().into_iter().rev().map(|name| Self { name }).collect(),
        }
    }
}

/// bukkit: org/bukkit/event/world/ChunkPopulateEvent
#[derive(Clone, Debug)]
pub struct ChunkPopulateEvent {
    chunk: crate::entity::Chunk,
}

impl ChunkEvent for ChunkPopulateEvent {
    fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
}

impl ChunkPopulateEvent {
    pub fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
}

impl Event for ChunkPopulateEvent {
    const HOOK: &'static str = "chunk_load";
    fn read(value: &Value) -> Option<Self> {
        if !value["new"].as_bool().unwrap_or(false) {
            return None;
        }
        let at = |i: usize| value["chunk"][i].as_i64().map(|v| v as i32);
        Some(Self { chunk: world_of(value).get_chunk_at(at(0)?, at(1)?) })
    }
}

/// bukkit: org/bukkit/event/server/BroadcastMessageEvent
///
/// `Server.broadcast`, as Paper's: the game's own lines to everyone aren't broadcasts.
#[derive(Clone, Debug)]
pub struct BroadcastMessageEvent {
    message: String,
    recipients: Vec<CommandSender>,
    listed: usize,
    changed: bool,
    cancelled: bool,
}

/// Whether the line goes out as it is: its listeners have had it, and any change was theirs to send.
pub(crate) fn broadcast_goes_out(message: &str) -> bool {
    query(json!({ "op": "ev_world_broadcast", "message": message })).as_bool().unwrap_or(true)
}

impl BroadcastMessageEvent {
    /// bukkit: org/bukkit/event/server/BroadcastMessageEvent.getRecipients
    pub fn get_recipients(&mut self) -> &mut Vec<CommandSender> {
        &mut self.recipients
    }
    /// bukkit: org/bukkit/event/server/BroadcastMessageEvent.getMessage
    pub fn get_message(&self) -> &str {
        &self.message
    }
    /// bukkit: org/bukkit/event/server/BroadcastMessageEvent.message
    pub fn message(&self) -> Component {
        Component::text(&self.message)
    }
    /// bukkit: org/bukkit/event/server/BroadcastMessageEvent.setMessage
    pub fn set_message(&mut self, message: &str) {
        message.clone_into(&mut self.message);
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/server/BroadcastMessageEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/server/BroadcastMessageEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BroadcastMessageEvent {
    const HOOK: &'static str = "ew_broadcast";
    fn read(value: &Value) -> Option<Self> {
        let message = value["message"].as_str()?;
        let mut recipients: Vec<CommandSender> = crate::get_online_players().into_iter().filter(|player| player.has_permission("bukkit.broadcast.user")).map(CommandSender::Player).collect();
        recipients.push(CommandSender::Console);
        Some(Self { message: message.to_owned(), listed: recipients.len(), recipients, changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed || self.recipients.len() != self.listed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled && (self.changed || self.recipients.len() != self.listed) {
            for recipient in &self.recipients {
                recipient.send_message(&self.message);
            }
        }
    }
}

// ---- The list of listeners ----

/// `org.bukkit.event.EventPriority.getSlot`.
///
/// bukkit: org/bukkit/event/EventPriority.getSlot
pub fn event_priority_slot(priority: &str) -> i32 {
    match priority {
        "LOWEST" => 0,
        "LOW" => 1,
        "HIGH" => 3,
        "HIGHEST" => 4,
        "MONITOR" => 5,
        _ => 2,
    }
}

// ---- Trees and portals ----

/// Who last fed what bone meal, this tick: `(place, tick, player)`.
static FED: Mutex<Vec<([i64; 3], u64, Option<Player>)>> = Mutex::new(Vec::new());

/// bukkit: org/bukkit/event/world/StructureGrowEvent
///
/// As the tree or mushroom has grown, by the game's own writes: taking a block off the list puts back what
/// was there, and a cancel puts back all of it and the sapling.
#[derive(Clone, Debug)]
pub struct StructureGrowEvent {
    value: Value,
    blocks: Vec<BlockState>,
    listed: usize,
    fed: Option<Option<Player>>,
    cancelled: bool,
}

impl StructureGrowEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: org/bukkit/event/world/StructureGrowEvent.getLocation
    pub fn get_location(&self) -> Location {
        let at = |i: usize| self.value["pos"][i].as_f64().unwrap_or(0.0);
        Location::new(self.get_world(), at(0), at(1), at(2), 0.0, 0.0)
    }
    /// bukkit: org/bukkit/event/world/StructureGrowEvent.getBlocks
    ///
    /// What it's made of: take one out and that block isn't changed.
    pub fn get_blocks(&mut self) -> &mut Vec<BlockState> {
        &mut self.blocks
    }
    /// bukkit: org/bukkit/event/world/StructureGrowEvent.getSpecies
    ///
    /// `TreeType`'s name, from what the game grew.
    pub fn get_species(&self) -> &'static str {
        let feature = text(&self.value, "feature").trim_start_matches("minecraft:");
        let feature = feature.split("_bees").next().unwrap_or(feature);
        match feature {
            "fancy_oak" => "BIG_TREE",
            "birch" => "BIRCH",
            "super_birch" => "TALL_BIRCH",
            "spruce" => "REDWOOD",
            "pine" => "TALL_REDWOOD",
            "mega_spruce" => "MEGA_REDWOOD",
            "mega_pine" => "MEGA_PINE",
            "jungle_tree" | "jungle_tree_no_vine" => "SMALL_JUNGLE",
            "mega_jungle_tree" => "JUNGLE",
            "acacia" => "ACACIA",
            "dark_oak" => "DARK_OAK",
            "cherry" => "CHERRY",
            "pale_oak" => "PALE_OAK",
            "pale_oak_creaking" => "PALE_OAK_CREAKING",
            "mangrove" => "MANGROVE",
            "tall_mangrove" => "TALL_MANGROVE",
            "azalea_tree" => "AZALEA",
            "huge_red_mushroom" => "RED_MUSHROOM",
            "huge_brown_mushroom" => "BROWN_MUSHROOM",
            "crimson_fungus_planted" | "crimson_fungus" => "CRIMSON_FUNGUS",
            "warped_fungus_planted" | "warped_fungus" => "WARPED_FUNGUS",
            _ => "TREE",
        }
    }
    /// bukkit: org/bukkit/event/world/StructureGrowEvent.isFromBonemeal
    pub fn is_from_bonemeal(&self) -> bool {
        self.fed.is_some()
    }
    /// bukkit: org/bukkit/event/world/StructureGrowEvent.getPlayer
    pub fn get_player(&self) -> Option<&Player> {
        self.fed.as_ref()?.as_ref()
    }
    /// bukkit: org/bukkit/event/world/StructureGrowEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/world/StructureGrowEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for StructureGrowEvent {
    const HOOK: &'static str = "ew_grow";
    fn more_hooks() -> &'static [&'static str] {
        &["tick", "ew_bone_meal"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if value.is_null() {
            query(json!({ "op": "ev_world_install" }));
            return Some(Answer::Pass);
        }
        let pos = value["pos"].as_array().filter(|_| value.get("stack").is_some())?;
        let at = |i: usize| pos.get(i).and_then(Value::as_i64).unwrap_or(0);
        FED.lock().unwrap_or_else(|e| e.into_inner()).push(([at(0), at(1), at(2)], crate::scheduler::now(), player_of(value)));
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        let world = world_of(value);
        let blocks: Vec<BlockState> = value["blocks"].as_array()?.iter().filter_map(|row| {
            let block = world.get_block_at(row[0].as_i64()? as i32, row[1].as_i64()? as i32, row[2].as_i64()? as i32);
            Some(BlockState::snapshot(block, row[3].as_str()?))
        }).collect();
        let now = crate::scheduler::now();
        let at = |i: usize| value["pos"][i].as_i64().unwrap_or(0);
        let mut fed = FED.lock().unwrap_or_else(|e| e.into_inner());
        fed.retain(|(_, tick, _)| *tick == now);
        let fed = fed.iter().find(|(place, _, _)| *place == [at(0), at(1), at(2)]).map(|(_, _, player)| player.clone());
        Some(Self { value: value.clone(), listed: blocks.len(), blocks, fed, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled && self.blocks.len() != self.listed {
            let kept: Vec<[i32; 3]> = self.blocks.iter().map(|state| {
                let block = state.get_block();
                [block.get_x(), block.get_y(), block.get_z()]
            }).collect();
            query(json!({ "op": "ev_world_free_grow_kept", "blocks": kept }));
        }
    }
}

/// Who just used flint or a fire charge, and the tick: fire lit that way can make a portal.
static IGNITER: Mutex<Option<(Player, u64)>> = Mutex::new(None);

/// bukkit: org/bukkit/event/world/PortalCreateEvent
#[derive(Clone, Debug)]
pub struct PortalCreateEvent {
    value: Value,
    igniter: Option<Player>,
    cancelled: bool,
}

impl PortalCreateEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: org/bukkit/event/world/PortalCreateEvent.getBlocks
    ///
    /// The portal, and for a traveller's its frame and platform, as they will stand.
    pub fn get_blocks(&self) -> Vec<BlockState> {
        let world = self.get_world();
        self.value["blocks"].as_array().into_iter().flatten().filter_map(|row| {
            let block = world.get_block_at(row[0].as_i64()? as i32, row[1].as_i64()? as i32, row[2].as_i64()? as i32);
            Some(BlockState::snapshot(block, row[3].as_str()?))
        }).collect()
    }
    /// bukkit: org/bukkit/event/world/PortalCreateEvent.getReason
    ///
    /// `FIRE`, `NETHER_PAIR` or `END_PLATFORM`.
    pub fn get_reason(&self) -> &str {
        text(&self.value, "reason")
    }
    /// bukkit: org/bukkit/event/world/PortalCreateEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.igniter.as_ref().and_then(player_entity).or_else(|| player_entity(&player_of(&self.value)?))
    }
    /// bukkit: org/bukkit/event/world/PortalCreateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/world/PortalCreateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PortalCreateEvent {
    const HOOK: &'static str = "ew_portal";
    fn more_hooks() -> &'static [&'static str] {
        &["tick", "item_use_on"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if value.is_null() {
            query(json!({ "op": "ev_world_install" }));
            return Some(Answer::Pass);
        }
        if value.get("stack").is_none() {
            return None;
        }
        if matches!(text(&value["stack"], "item"), "minecraft:flint_and_steel" | "minecraft:fire_charge")
            && let Some(player) = player_of(value)
        {
            *IGNITER.lock().unwrap_or_else(|e| e.into_inner()) = Some((player, crate::scheduler::now()));
        }
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        value["blocks"].as_array()?;
        let now = crate::scheduler::now();
        let igniter = IGNITER.lock().unwrap_or_else(|e| e.into_inner()).take().filter(|(_, tick)| *tick == now).map(|(player, _)| player);
        Some(Self { value: value.clone(), igniter, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

// ---- More of a block's moments ----

/// bukkit: org/bukkit/event/block/BlockDropItemEvent
#[derive(Clone, Debug)]
pub struct BlockDropItemEvent {
    value: Value,
    items: Vec<crate::entities::Item>,
    listed: usize,
    cancelled: bool,
}

impl BlockDropItemEvent {
    /// bukkit: org/bukkit/event/block/BlockDropItemEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockDropItemEvent.getBlockState
    pub fn get_block_state(&self) -> Option<BlockState> {
        Some(BlockState::snapshot(self.get_block()?, text(&self.value, "state")))
    }
    /// bukkit: org/bukkit/event/block/BlockDropItemEvent.getItems
    ///
    /// One taken out of the list doesn't drop.
    pub fn get_items(&mut self) -> &mut Vec<crate::entities::Item> {
        &mut self.items
    }
    /// bukkit: org/bukkit/event/block/BlockDropItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockDropItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockDropItemEvent {
    const HOOK: &'static str = "ew_drops_fell";
    fn read(value: &Value) -> Option<Self> {
        let world = world_of(value);
        let items: Vec<crate::entities::Item> = value["items"].as_array()?.iter().filter_map(Value::as_u64).map(|id| crate::entities::Item(Entity::of(id, &world))).collect();
        Some(Self { value: value.clone(), listed: items.len(), items, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.items.len() != self.listed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.cancelled || self.items.len() != self.listed {
            let kept: Vec<u64> = if self.cancelled { Vec::new() } else { self.items.iter().map(|item| item.0.get_entity_id()).collect() };
            query(json!({ "op": "ev_world_drops_kept", "items": kept }));
        }
    }
}

/// bukkit: org/bukkit/event/block/BlockDamageEvent
#[derive(Clone, Debug)]
pub struct BlockDamageEvent {
    value: Value,
    insta: Option<bool>,
    cancelled: bool,
}

impl Event for BlockDamageEvent {
    const HOOK: &'static str = "attack_block";
    fn read(value: &Value) -> Option<Self> {
        if player_of(value).is_some_and(|player| player.get_game_mode() == "CREATIVE") {
            return None;
        }
        Some(Self { value: value.clone(), insta: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.insta == Some(true) { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.insta == Some(true) && !self.cancelled {
            query(json!({ "op": "ev_world_insta_break", "pos": self.value["pos"] }));
        }
    }
}

impl BlockDamageEvent {
    /// bukkit: org/bukkit/event/block/BlockDamageEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.get_player()?.get_world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockDamageEvent.getBlockFace
    pub fn get_block_face(&self) -> &'static str {
        face_name(&self.value["face"])
    }
    /// bukkit: org/bukkit/event/block/BlockDamageEvent.getItemInHand
    pub fn get_item_in_hand(&self) -> Option<ItemStack> {
        ItemStack::read(&query(json!({ "op": "equipment", "uuid": self.get_player()?.uuid() }))["main_hand"])
    }
    /// bukkit: org/bukkit/event/block/BlockDamageEvent.getInstaBreak
    pub fn get_insta_break(&self) -> bool {
        self.insta.unwrap_or_else(|| {
            let (Some(block), Some(player)) = (self.get_block(), player_of(&self.value)) else {
                return false;
            };
            block.get_break_speed(&player) >= 1.0
        })
    }
    /// bukkit: org/bukkit/event/block/BlockDamageEvent.setInstaBreak
    pub fn set_insta_break(&mut self, insta: bool) {
        self.insta = Some(insta);
    }
    /// bukkit: org/bukkit/event/block/BlockDamageEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockDamageEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/block/BlockCanBuildEvent
///
/// As a block's item is checked for placing, with the game's own answer.
#[derive(Clone, Debug)]
pub struct BlockCanBuildEvent {
    value: Value,
    buildable: bool,
}

impl BlockCanBuildEvent {
    /// bukkit: org/bukkit/event/block/BlockCanBuildEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    ///
    /// The block the item goes against.
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockCanBuildEvent.getMaterial
    pub fn get_material(&self) -> Material {
        Material::of(text(&self.value, "state").split('[').next().unwrap_or_default())
    }
    /// bukkit: org/bukkit/event/block/BlockCanBuildEvent.getBlockData
    pub fn get_block_data(&self) -> String {
        text(&self.value, "state").to_owned()
    }
    /// bukkit: org/bukkit/event/block/BlockCanBuildEvent.getHand
    pub fn get_hand(&self) -> &'static str {
        hand_used(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockCanBuildEvent.isBuildable
    pub fn is_buildable(&self) -> bool {
        self.buildable
    }
    /// bukkit: org/bukkit/event/block/BlockCanBuildEvent.setBuildable
    pub fn set_buildable(&mut self, buildable: bool) {
        self.buildable = buildable;
    }
}

impl Event for BlockCanBuildEvent {
    const HOOK: &'static str = "ew_can_build";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), buildable: value["buildable"].as_bool()? })
    }
    fn finish(&self) {
        query(json!({ "op": "ev_world_free_can_build", "buildable": self.buildable }));
        crate::event::carry("buildable", json!(self.buildable));
    }
}

/// bukkit: io/papermc/paper/event/block/PlayerShearBlockEvent
#[derive(Clone, Debug)]
pub struct PlayerShearBlockEvent {
    value: Value,
    drops: Vec<ItemStack>,
    listed: usize,
    cancelled: bool,
}

const PUMPKIN_LOOT: &str = "minecraft:carve/pumpkin";

/// The loot table of what's being sheared.
fn shorn_loot(value: &Value) -> &'static str {
    if value["hive"] == true { "minecraft:harvest/beehive" } else { PUMPKIN_LOOT }
}

/// What a loot table is about to give: the game's dice are put back after the look.
fn loot_preview(parent: &str, table: &str, held: bool) -> Vec<ItemStack> {
    query(json!({ "op": "ev_world_loot_preview", "parent": parent, "table": table, "held": held })).as_array().into_iter().flatten().filter_map(ItemStack::read).collect()
}

fn loot_changed(table: &str, drops: &[ItemStack]) {
    let items: Vec<Value> = drops.iter().map(ItemStack::to_json).collect();
    query(json!({ "op": "ev_world_loot_next", "table": table, "items": items }));
}

impl PlayerShearBlockEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/block/PlayerShearBlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.get_player()?.get_world(), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/PlayerShearBlockEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: io/papermc/paper/event/block/PlayerShearBlockEvent.getHand
    ///
    /// `HAND`: the game only shears with the main hand.
    pub fn get_hand(&self) -> &'static str {
        "HAND"
    }
    /// bukkit: io/papermc/paper/event/block/PlayerShearBlockEvent.getDrops
    ///
    /// What falls: take a stack out and it doesn't.
    pub fn get_drops(&mut self) -> &mut Vec<ItemStack> {
        &mut self.drops
    }
    /// bukkit: io/papermc/paper/event/block/PlayerShearBlockEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/block/PlayerShearBlockEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PlayerShearBlockEvent {
    const HOOK: &'static str = "ew_shear_block";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_shear_hive"]
    }
    fn read(value: &Value) -> Option<Self> {
        if text(&value["stack"], "item") != "minecraft:shears" || !(value["pumpkin"] == true || value["hive"] == true) {
            return None;
        }
        let table = shorn_loot(value);
        let drops = loot_preview(table, table, true);
        Some(Self { value: value.clone(), listed: drops.len(), drops, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled && self.drops.len() != self.listed {
            loot_changed(shorn_loot(&self.value), &self.drops);
        }
    }
}

/// Who last used a block, and the tick, as a bell's own run doesn't say who rang it.
static BELL_USER: Mutex<Option<(Value, u64, Player)>> = Mutex::new(None);

/// bukkit: org/bukkit/event/block/BellRingEvent
///
/// Whoever or whatever rings it: see `BlockIgniteEvent`.
#[derive(Clone, Debug)]
pub struct BellRingEvent {
    value: Value,
    user: Option<Player>,
    cancelled: bool,
}

impl BellRingEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BellRingEvent.getDirection
    pub fn get_direction(&self) -> &'static str {
        face_name(&self.value["face"])
    }
    /// bukkit: org/bukkit/event/block/BellRingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "entity").or_else(|| player_entity(self.user.as_ref()?)).or_else(|| player_entity(&player_of(&self.value)?))
    }
    /// bukkit: org/bukkit/event/block/BellRingEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BellRingEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BellRingEvent {
    const HOOK: &'static str = "ew_bell";
    fn more_hooks() -> &'static [&'static str] {
        &["use_block", "ew_bell_hit"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if value.get("ring").is_some() {
            return None;
        }
        if let (Some(pos), Some(player)) = (value.get("pos"), player_of(value).filter(|_| value.get("entity").is_none())) {
            *BELL_USER.lock().unwrap_or_else(|e| e.into_inner()) = Some((pos.clone(), crate::scheduler::now(), player));
        }
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        let now = crate::scheduler::now();
        let user = BELL_USER.lock().unwrap_or_else(|e| e.into_inner()).take().filter(|(pos, tick, _)| *tick == now && *pos == value["pos"]).map(|(_, _, player)| player);
        Some(Self { value: value.clone(), user, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: com/destroystokyo/paper/event/block/BeaconEffectEvent
#[derive(Clone, Debug)]
pub struct BeaconEffectEvent {
    value: Value,
    effect: PotionEffect,
    primary: bool,
    cancelled: bool,
}

impl BeaconEffectEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: com/destroystokyo/paper/event/block/BeaconEffectEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/block/BeaconEffectEvent.isPrimary
    pub fn is_primary(&self) -> bool {
        self.primary
    }
    /// bukkit: com/destroystokyo/paper/event/block/BeaconEffectEvent.getEffect
    pub fn get_effect(&self) -> &PotionEffect {
        &self.effect
    }
    /// bukkit: com/destroystokyo/paper/event/block/BeaconEffectEvent.setEffect
    pub fn set_effect(&mut self, effect: PotionEffect) {
        self.effect = effect;
    }
    /// bukkit: com/destroystokyo/paper/event/block/BeaconEffectEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/block/BeaconEffectEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BeaconEffectEvent {
    const HOOK: &'static str = "ew_beacon";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        let Some(player) = player_of(value) else {
            return Vec::new();
        };
        let (primary, secondary, levels) = (value["primary"].as_str(), value["secondary"].as_str(), value["levels"].as_i64().unwrap_or(0));
        let range = (levels * 10 + 10) as f64;
        let at = |i: usize| value["pos"][i].as_f64().unwrap_or(0.0);
        let there = player.get_location();
        let Some(primary) = primary.filter(|_| there.get_x() >= at(0) - range && there.get_x() <= at(0) + 1.0 + range && there.get_z() >= at(2) - range && there.get_z() <= at(2) + 1.0 + range && there.get_y() >= at(1) - range) else {
            return Vec::new();
        };
        let ticks = ((9 + levels * 2) * 20) as i32;
        let strong = levels >= 4 && secondary == Some(primary);
        let mut out = vec![Self { value: value.clone(), effect: PotionEffect::new(primary, ticks, i32::from(strong), true, true), primary: true, cancelled: crate::event::prior_cancelled() }];
        if let Some(secondary) = secondary.filter(|secondary| levels >= 4 && *secondary != primary) {
            out.push(Self { value: value.clone(), effect: PotionEffect::new(secondary, ticks, 0, true, true), primary: false, cancelled: crate::event::prior_cancelled() });
        }
        out
    }
    /// The game's own giving is always skipped: each effect goes on by itself in `finish`.
    fn answer(&self) -> Answer {
        Answer::Fail
    }
    fn finish(&self) {
        if !self.cancelled {
            query(json!({ "op": "ev_world_beacon_effect", "id": self.effect.get_type(), "duration": self.effect.get_duration(), "amplifier": self.effect.get_amplifier() }));
        }
    }
}

/// bukkit: org/bukkit/event/block/CampfireStartEvent
#[derive(Clone, Debug)]
pub struct CampfireStartEvent {
    value: Value,
    time: i64,
    cancelled: bool,
}

impl Event for CampfireStartEvent {
    const HOOK: &'static str = "ew_campfire";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), time: value["time"].as_i64()?, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.time != self.value["time"].as_i64().unwrap_or(0) { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.time != self.value["time"].as_i64().unwrap_or(0) {
            query(json!({ "op": "ev_world_campfire", "pos": self.value["pos"], "time": self.time }));
        }
    }
}

/// `org.bukkit.event.block.InventoryBlockStartEvent`.
pub trait InventoryBlockStartEvent {
    /// bukkit: org/bukkit/event/block/InventoryBlockStartEvent.getSource, org/bukkit/event/block/InventoryBlockStartEvent.getHandlers, org/bukkit/event/block/InventoryBlockStartEvent.getHandlerList
    fn get_source(&self) -> Option<ItemStack>;
}

impl InventoryBlockStartEvent for CampfireStartEvent {
    fn get_source(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
}

impl CampfireStartEvent {
    /// bukkit: org/bukkit/event/block/CampfireStartEvent.getRecipe
    pub fn get_recipe(&self) -> Option<crate::core2::GameRecipe> {
        crate::core2::get_recipe(&crate::persistence::NamespacedKey::from_string(self.value["recipe"].as_str()?)?)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&player_of(&self.value)?.get_world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/CampfireStartEvent.getTotalCookTime
    pub fn get_total_cook_time(&self) -> i64 {
        self.time
    }
    /// bukkit: org/bukkit/event/block/CampfireStartEvent.setTotalCookTime
    pub fn set_total_cook_time(&mut self, ticks: i64) {
        self.time = ticks.max(0);
    }
}

// ---- Commands the game didn't know, and time ----

/// bukkit: org/bukkit/event/command/UnknownCommandEvent
#[derive(Clone, Debug)]
pub struct UnknownCommandEvent {
    value: Value,
    message: Option<Option<String>>,
    cancelled: bool,
}

impl UnknownCommandEvent {
    /// bukkit: org/bukkit/event/command/UnknownCommandEvent.getSender
    pub fn get_sender(&self) -> CommandSender {
        player_of(&self.value).map_or(CommandSender::Console, CommandSender::Player)
    }
    /// bukkit: org/bukkit/event/command/UnknownCommandEvent.getCommandLine
    pub fn get_command_line(&self) -> &str {
        text(&self.value, "line")
    }
    /// bukkit: org/bukkit/event/command/UnknownCommandEvent.getMessage
    pub fn get_message(&self) -> Option<&str> {
        match &self.message {
            Some(message) => message.as_deref(),
            None => Some(text(&self.value, "message")),
        }
    }
    /// bukkit: org/bukkit/event/command/UnknownCommandEvent.message
    pub fn message(&self) -> Option<Component> {
        self.get_message().map(Component::text)
    }
    /// bukkit: org/bukkit/event/command/UnknownCommandEvent.setMessage
    ///
    /// `None` says nothing.
    pub fn set_message(&mut self, message: Option<&str>) {
        self.message = Some(message.map(str::to_owned));
    }
}

impl Event for UnknownCommandEvent {
    const HOOK: &'static str = "ew_unknown_command";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), message: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.message.is_some() { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let Some(Some(message)) = &self.message {
            self.get_sender().send_message(message);
        }
    }
}

/// bukkit: org/bukkit/event/world/TimeSkipEvent
#[derive(Clone, Debug)]
pub struct TimeSkipEvent {
    value: Value,
    skip: i64,
    changed: bool,
    cancelled: bool,
}

impl TimeSkipEvent {
    /// bukkit: org/bukkit/event/world/TimeSkipEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    pub fn get_skip_reason(&self) -> &'static str {
        if text(&self.value, "id") == "time.sleep" { "NIGHT_SKIP" } else { "COMMAND" }
    }
    pub fn get_skip_amount(&self) -> i64 {
        self.skip
    }
    pub fn set_skip_amount(&mut self, skip: i64) {
        self.skip = skip;
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for TimeSkipEvent {
    const HOOK: &'static str = "ew_time";
    fn read(value: &Value) -> Option<Self> {
        let time = value["time"].as_i64()?;
        let skip = match text(value, "id") {
            "time.add" | "time.sleep" => time,
            "time.set" => time - value["now"].as_i64()?,
            _ => return None,
        };
        if mine(&format!("time add {skip}")) {
            return None;
        }
        Some(Self { value: value.clone(), skip, changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || (self.changed && text(&self.value, "id") != "time.sleep") { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.changed && !self.cancelled {
            if text(&self.value, "id") == "time.sleep" {
                query(json!({ "op": "ev_world_sleep_skip", "skip": self.skip }));
            } else {
                run_mine(format!("time add {}", self.skip));
            }
        }
    }
}

/// bukkit~: org/bukkit/event/world/ClockTimeSkipEvent | never fires: Paper fires this in `TimeSkipEvent`'s place only with `time.affects-all-worlds`, and each level here keeps its own time
#[derive(Clone, Debug)]
pub struct ClockTimeSkipEvent {
    reason: &'static str,
    skip: i64,
    cancelled: bool,
}

impl ClockTimeSkipEvent {
    /// bukkit: org/bukkit/event/world/ClockTimeSkipEvent.getSkipReason
    pub fn get_skip_reason(&self) -> &'static str {
        self.reason
    }
    /// bukkit: org/bukkit/event/world/ClockTimeSkipEvent.getSkipAmount
    pub fn get_skip_amount(&self) -> i64 {
        self.skip
    }
    /// bukkit: org/bukkit/event/world/ClockTimeSkipEvent.setSkipAmount
    pub fn set_skip_amount(&mut self, skip: i64) {
        self.skip = skip;
    }
    /// bukkit: org/bukkit/event/world/ClockTimeSkipEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/world/ClockTimeSkipEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ClockTimeSkipEvent {
    const HOOK: &'static str = "ew_time";
    fn read(_: &Value) -> Option<Self> {
        None
    }
}

/// bukkit: org/bukkit/event/weather/LightningStrikeEvent
#[derive(Clone, Debug)]
pub struct LightningStrikeEvent {
    value: Value,
    cause: &'static str,
    cancelled: bool,
}

/// What is about to strike, and the tick it said so.
static BOLT: Mutex<Option<(&'static str, u64)>> = Mutex::new(None);

impl LightningStrikeEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    pub fn get_location(&self) -> Option<Location> {
        let at = |i: usize| self.value["at"][i].as_f64();
        Some(Location::new(self.get_world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    pub fn is_effect(&self) -> bool {
        self.value["visual_only"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/event/weather/LightningStrikeEvent.getLightning
    ///
    /// The bolt, which goes again if the event's cancelled.
    pub fn get_lightning(&self) -> Option<crate::fam_entity::LightningStrike> {
        let id = self.value["bolt"].as_u64()?;
        Some(crate::fam_entity::LightningStrike(Entity::of(id, &self.get_world())))
    }
    /// bukkit: org/bukkit/event/weather/LightningStrikeEvent.getCause
    pub fn get_cause(&self) -> &'static str {
        self.cause
    }
    /// bukkit: org/bukkit/event/weather/LightningStrikeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/weather/LightningStrikeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for LightningStrikeEvent {
    const HOOK: &'static str = "ew_bolt";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_bolt_trident", "ew_bolt_command"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        let cause = match value["why"].as_str()? {
            "TRIDENT" => "TRIDENT",
            _ => "COMMAND",
        };
        *BOLT.lock().unwrap_or_else(|e| e.into_inner()) = Some((cause, crate::scheduler::now()));
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        let now = crate::scheduler::now();
        let cause = BOLT.lock().unwrap_or_else(|e| e.into_inner()).take().filter(|(_, tick)| *tick == now).map_or("WEATHER", |(cause, _)| cause);
        let cause = match value["cause"].as_str() {
            Some("CUSTOM") => "CUSTOM",
            Some("TRAP") => "TRAP",
            _ => cause,
        };
        Some(Self { value: value.clone(), cause, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

// ---- The list of listeners ----

impl HandlerList {
    /// bukkit~: org/bukkit/event/HandlerList.register | a closure for one event: Java's `RegisteredListener` has no use here
    pub fn register<E: Event>(plugin: &crate::Plugin, listener: impl Fn(&mut E) + Send + Sync + 'static) {
        plugin.register_event::<E>(listener);
    }
    /// bukkit~: org/bukkit/event/HandlerList.registerAll | closures for one event
    pub fn register_all<E: Event, L: Fn(&mut E) + Send + Sync + 'static>(plugin: &crate::Plugin, listeners: impl IntoIterator<Item = L>) {
        for listener in listeners {
            plugin.register_event::<E>(listener);
        }
    }
}

/// bukkit: org/bukkit/event/block/BlockMultiPlaceEvent
#[derive(Clone, Debug)]
pub struct BlockMultiPlaceEvent {
    pub place: BlockPlaceEvent,
}

impl BlockMultiPlaceEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        self.place.get_player()
    }
    pub fn get_block_placed(&self) -> &Block {
        self.place.get_block_placed()
    }
    /// bukkit: org/bukkit/event/block/BlockMultiPlaceEvent.getReplacedBlockStates
    pub fn get_replaced_block_states(&self) -> Vec<BlockState> {
        let mut states = vec![self.place.get_block_replaced_state()];
        let block = self.place.get_block_placed();
        let kind = block.get_block_data();
        let kind = kind.split('[').next().unwrap_or_default();
        let around = self.place.click.iter().flat_map(|click| click.around.iter());
        for (place, was) in around {
            let Some(other) = block_at(&block.get_world(), place) else {
                continue;
            };
            if (other.get_x() - block.get_x()).abs() + (other.get_y() - block.get_y()).abs() + (other.get_z() - block.get_z()).abs() != 1 {
                continue;
            }
            // The other half: what's of its kind now, and wasn't.
            if other.get_block_data().split('[').next() == Some(kind) && was.split('[').next() != Some(kind) {
                states.push(BlockState::snapshot(other, was));
            }
        }
        states
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.place.set_cancelled(cancelled);
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.place.is_cancelled()
    }
}

impl Event for BlockMultiPlaceEvent {
    const HOOK: &'static str = "block_place";
    fn more_hooks() -> &'static [&'static str] {
        BlockPlaceEvent::more_hooks()
    }
    fn intercept(value: &Value) -> Option<Answer> {
        BlockPlaceEvent::intercept(value)
    }
    fn read(value: &Value) -> Option<Self> {
        let item = text(&value["stack"], "item");
        let tall = ["_bed", "_door"].iter().any(|end| item.ends_with(end)) || matches!(item, "minecraft:sunflower" | "minecraft:lilac" | "minecraft:rose_bush" | "minecraft:peony" | "minecraft:tall_grass" | "minecraft:large_fern" | "minecraft:pitcher_plant" | "minecraft:small_dripleaf" | "minecraft:tall_seagrass");
        tall.then(|| BlockPlaceEvent::read(value)).flatten().map(|place| Self { place })
    }
    fn finish(&self) {
        self.place.finish();
    }
}

/// bukkit: org/bukkit/event/block/BlockShearEntityEvent
#[derive(Clone, Debug)]
pub struct BlockShearEntityEvent {
    value: Value,
    drops: Vec<ItemStack>,
    table: Option<String>,
    changed: bool,
    cancelled: bool,
}

impl BlockShearEntityEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockShearEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "target")
    }
    /// bukkit: org/bukkit/event/block/BlockShearEntityEvent.getTool
    pub fn get_tool(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/block/BlockShearEntityEvent.getDrops
    pub fn get_drops(&self) -> &[ItemStack] {
        &self.drops
    }
    /// bukkit: org/bukkit/event/block/BlockShearEntityEvent.setDrops
    pub fn set_drops(&mut self, drops: Vec<ItemStack>) {
        self.drops = drops;
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/block/BlockShearEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockShearEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockShearEntityEvent {
    const HOOK: &'static str = "ew_shear_entity";
    fn read(value: &Value) -> Option<Self> {
        let parent = value["parent"].as_str()?;
        let table = text(value, "table").to_owned();
        let drops = loot_preview(parent, &table, false);
        Some(Self { value: value.clone(), drops, table: Some(table), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let (true, false, Some(table)) = (self.changed, self.cancelled, &self.table) {
            loot_changed(table, &self.drops);
        }
    }
}

/// `org.bukkit.event.server.PluginEvent`.
pub trait PluginEvent {
    /// bukkit: org/bukkit/event/server/PluginEvent.getPlugin
    fn get_plugin(&self) -> Option<crate::fam_core::LoadedPlugin>;
}

impl PluginEvent for PluginEnableEvent {
    fn get_plugin(&self) -> Option<crate::fam_core::LoadedPlugin> {
        crate::fam_core::get_plugin_manager().get_plugin(&self.name)
    }
}

impl PluginEvent for PluginDisableEvent {
    fn get_plugin(&self) -> Option<crate::fam_core::LoadedPlugin> {
        crate::fam_core::get_plugin_manager().get_plugin(&self.name)
    }
}

macro_rules! weather {
    ($name:ident) => {
        impl WeatherEvent for $name {
            fn get_world(&self) -> World {
                $name::get_world(self).clone()
            }
        }
    };
}

weather!(WeatherChangeEvent);
weather!(ThunderChangeEvent);
weather!(LightningStrikeEvent);

/// bukkit~: org/bukkit/event/block/BlockRedstoneEvent | the world's own block ticks, after the change; a new current is written into the block's state then
#[derive(Clone, Debug)]
pub struct BlockRedstoneEvent {
    value: Value,
    current: Option<i64>,
    cancelled: bool,
}

impl BlockRedstoneEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockRedstoneEvent.getOldCurrent
    pub fn get_old_current(&self) -> i64 {
        self.value["old_current"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/block/BlockRedstoneEvent.getNewCurrent
    pub fn get_new_current(&self) -> i64 {
        self.current.unwrap_or_else(|| self.value["new_current"].as_i64().unwrap_or(0))
    }
    /// bukkit: org/bukkit/event/block/BlockRedstoneEvent.setNewCurrent
    pub fn set_new_current(&mut self, current: i64) {
        self.current = Some(current.clamp(0, 15));
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockRedstoneEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "redstone").then(|| Self { value: value.clone(), current: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        // A lever or a button has two currents: one left where it was doesn't move.
        let stays = self.value["binary"] == true && self.current.is_some_and(|current| (current > 0) == (self.get_old_current() > 0));
        if self.cancelled || stays { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.value["binary"] == true {
            return;
        }
        let (Some(current), Some(block), false) = (self.current, self.get_block(), self.cancelled) else {
            return;
        };
        // `power=` where the block has a strength, else `powered=` or `lit=`.
        let state = text(&self.value, "new");
        let Some((name, properties)) = state.split_once('[') else {
            return;
        };
        let written: Vec<String> = properties
            .trim_end_matches(']')
            .split(',')
            .map(|property| match property.split_once('=') {
                Some(("power", _)) => format!("power={current}"),
                Some((key @ ("powered" | "lit"), _)) => format!("{key}={}", current > 0),
                _ => property.to_owned(),
            })
            .collect();
        block.set_block_data(&format!("{name}[{}]", written.join(",")));
    }
}

/// A border on its way to a new size, by when it should arrive.
#[derive(Clone, Debug)]
struct BorderMove {
    level: String,
    old: f64,
    new: f64,
    ticks: i64,
    end: u64,
}

#[derive(Default)]
struct BorderMoves {
    now: u64,
    going: Vec<BorderMove>,
    done: Vec<BorderMove>,
}

static BORDER_MOVES: Mutex<Option<BorderMoves>> = Mutex::new(None);

/// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeFinishEvent
#[derive(Clone, Debug)]
pub struct WorldBorderBoundsChangeFinishEvent {
    moved: BorderMove,
}

impl WorldBorderEvent for WorldBorderBoundsChangeFinishEvent {
    fn get_world_border(&self) -> crate::world::WorldBorder {
        World::of(&self.moved.level).get_world_border()
    }
}

impl WorldBorderBoundsChangeFinishEvent {
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeFinishEvent.getOldSize
    pub fn get_old_size(&self) -> f64 {
        self.moved.old
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeFinishEvent.getNewSize
    pub fn get_new_size(&self) -> f64 {
        self.moved.new
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeFinishEvent.getDuration
    ///
    /// In milliseconds.
    pub fn get_duration(&self) -> f64 {
        (self.moved.ticks * 50) as f64
    }
}

impl Event for WorldBorderBoundsChangeFinishEvent {
    const HOOK: &'static str = "ew_border";
    fn more_hooks() -> &'static [&'static str] {
        &["tick"]
    }
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        let mut moves = BORDER_MOVES.lock().unwrap_or_else(|e| e.into_inner());
        let moves = moves.get_or_insert_default();
        let now = crate::scheduler::now();
        if value["id"].is_string() {
            let (ticks, old, distance) = (value["time"].as_i64().unwrap_or(0), value["old"].as_f64().unwrap_or(0.0), value["distance"].as_f64().unwrap_or(0.0));
            let new = match text(value, "id") {
                "worldborder.set" => distance,
                "worldborder.add" => old + distance,
                _ => return Vec::new(),
            };
            let end = now + ticks as u64;
            let level = text(value, "level").to_owned();
            if ticks > 0 && !moves.going.iter().any(|going| going.end == end && going.level == level) {
                moves.going.push(BorderMove { level, old, new, ticks, end });
            }
            return Vec::new();
        }
        if moves.now != now {
            moves.now = now;
            let (done, going): (Vec<BorderMove>, Vec<BorderMove>) = std::mem::take(&mut moves.going).into_iter().partition(|moved| moved.end <= now);
            moves.going = going;
            moves.done = done.into_iter().filter(|moved| (World::of(&moved.level).get_world_border().get_size() - moved.new).abs() < 0.001).collect();
        }
        moves.done.iter().map(|moved| Self { moved: moved.clone() }).collect()
    }
}

// ---- The server list ----

/// `PaperServerListPingEvent.ListedPlayerInfo`: a line of the list a ping shows on hover.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedPlayerInfo {
    name: String,
    id: String,
}

impl ListedPlayerInfo {
    pub fn new(name: &str, id: &str) -> Self {
        Self { name: name.to_owned(), id: id.to_owned() }
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent$ListedPlayerInfo.name
    pub fn name(&self) -> &str {
        &self.name
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent$ListedPlayerInfo.id
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// What `PaperServerListPingEvent::iterator` walks: Java's `Iterator<Player>`, with `remove`.
#[derive(Debug)]
pub struct PingPlayers<'a> {
    event: &'a mut PaperServerListPingEvent,
    at: usize,
}

impl PingPlayers<'_> {
    pub fn has_next(&self) -> bool {
        self.at < self.event.players.len()
    }
    /// Takes the player last given off the list, and off the count.
    pub fn remove(&mut self) {
        if self.at > 0 {
            self.at -= 1;
            let gone = self.event.players.remove(self.at);
            self.event.listed.retain(|line| line.name != gone.get_name());
            self.event.number -= 1;
        }
    }
}

impl Iterator for PingPlayers<'_> {
    type Item = Player;
    fn next(&mut self) -> Option<Player> {
        let player = self.event.players.get(self.at).cloned();
        self.at += usize::from(player.is_some());
        player
    }
}

/// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent, org/bukkit/event/server/ServerListPingEvent
///
/// On the connection's own thread, with no game to ask: the players are those online at the end of the last tick.
#[derive(Clone, Debug)]
pub struct PaperServerListPingEvent {
    pub(crate) id: Value,
    status: Value,
    motd: Value,
    number: i64,
    max: i64,
    hide: bool,
    listed: Vec<ListedPlayerInfo>,
    players: Vec<Player>,
    version: String,
    protocol: i64,
    icon: Option<String>,
    cancelled: bool,
}

/// `org.bukkit.event.server.ServerListPingEvent`: the same event here.
pub type ServerListPingEvent = PaperServerListPingEvent;

impl PaperServerListPingEvent {
    /// `ip:port`, as the connection's address reads.
    pub fn get_socket_address(&self) -> Option<String> {
        crate::fam_paper::NetworkClient::get_address(&self.get_client())
    }
    /// The address without its port.
    pub fn get_address(&self) -> Option<String> {
        let socket = self.get_socket_address()?;
        Some(socket.rsplit_once(':').map_or(socket.as_str(), |(ip, _)| ip).trim_matches(|c| c == '[' || c == ']').to_owned())
    }
    /// What the client said it connected to.
    pub fn get_hostname(&self) -> String {
        crate::fam_paper::NetworkClient::get_virtual_host(&self.get_client()).unwrap_or_default()
    }
    pub fn get_motd(&self) -> String {
        self.motd().plain()
    }
    pub fn motd(&self) -> Component {
        Component(self.motd.clone())
    }
    pub fn set_motd(&mut self, motd: &str) {
        self.motd = json!(motd);
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.motd(Component)
    ///
    /// The MOTD as a text component (hex colours, gradients), which a legacy string can't carry.
    pub fn set_motd_component(&mut self, motd: &Component) {
        self.motd = motd.0.clone();
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.getNumPlayers
    pub fn get_num_players(&self) -> i64 {
        if self.hide { -1 } else { self.number }
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.setNumPlayers
    pub fn set_num_players(&mut self, number: i64) {
        self.number = number;
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.getMaxPlayers
    pub fn get_max_players(&self) -> i64 {
        if self.hide { -1 } else { self.max }
    }
    pub fn set_max_players(&mut self, max: i64) {
        self.max = max;
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.shouldHidePlayers
    pub fn should_hide_players(&self) -> bool {
        self.hide
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.setHidePlayers
    pub fn set_hide_players(&mut self, hide: bool) {
        self.hide = hide;
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.getListedPlayers
    pub fn get_listed_players(&mut self) -> &mut Vec<ListedPlayerInfo> {
        &mut self.listed
    }
    /// bukkit~: com/destroystokyo/paper/event/server/PaperServerListPingEvent.getPlayerSample | a copy: change the list through `get_listed_players`
    pub fn get_player_sample(&self) -> Vec<crate::player::PlayerProfile> {
        self.listed.iter().map(|line| crate::player::PlayerProfile { name: line.name.clone(), id: line.id.clone(), properties: Vec::new() }).collect()
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.getVersion
    pub fn get_version(&self) -> &str {
        &self.version
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.setVersion
    pub fn set_version(&mut self, version: &str) {
        version.clone_into(&mut self.version);
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.getProtocolVersion
    pub fn get_protocol_version(&self) -> i64 {
        self.protocol
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.setProtocolVersion
    pub fn set_protocol_version(&mut self, protocol: i64) {
        self.protocol = protocol;
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.getServerIcon
    pub fn get_server_icon(&self) -> Option<crate::fam_core::CachedServerIcon> {
        self.icon.as_ref().map(|data| crate::fam_core::CachedServerIcon::of_data(data))
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.setServerIcon
    pub fn set_server_icon(&mut self, icon: Option<&crate::fam_core::CachedServerIcon>) {
        self.icon = icon.and_then(|icon| icon.get_data()).map(str::to_owned);
    }
    pub fn should_send_chat_previews(&self) -> bool {
        false
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.iterator
    pub fn iterator(&mut self) -> PingPlayers<'_> {
        PingPlayers { event: self, at: 0 }
    }
    /// bukkit~: com/destroystokyo/paper/event/server/PaperServerListPingEvent.setCancelled | an empty answer: the client shows it can't connect, where Paper closes the connection
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    fn answer_text(&self) -> String {
        if self.cancelled {
            return String::new();
        }
        let mut status = self.status.clone();
        status["description"] = self.motd.clone();
        status["version"] = json!({ "name": self.version, "protocol": self.protocol });
        if self.hide {
            if let Some(status) = status.as_object_mut() {
                status.remove("players");
            }
        } else {
            let sample: Vec<Value> = self.listed.iter().map(|line| json!({ "id": line.id, "name": line.name })).collect();
            status["players"] = json!({ "max": self.max, "online": self.number, "sample": sample });
        }
        match &self.icon {
            Some(icon) => status["favicon"] = json!(icon),
            None => {
                if let Some(status) = status.as_object_mut() {
                    status.remove("favicon");
                }
            }
        }
        status.to_string()
    }
}

/// Who was online at the end of the last tick, as a ping has no game to ask.
static HEARERS: Mutex<Vec<Player>> = Mutex::new(Vec::new());

impl Event for PaperServerListPingEvent {
    const HOOK: &'static str = "ew_ping";
    fn more_hooks() -> &'static [&'static str] {
        &["tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if !value.is_null() {
            return None;
        }
        *HEARERS.lock().unwrap_or_else(|e| e.into_inner()) = crate::get_online_players();
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        // What the listener before this one left.
        let text = query(json!({ "op": "ev_world_free_ping_now" }));
        let status: Value = serde_json::from_str(text.as_str()?).ok()?;
        let listed = status["players"]["sample"].as_array().into_iter().flatten().map(|line| ListedPlayerInfo::new(self::text(line, "name"), self::text(line, "id"))).collect();
        Some(Self {
            id: value["id"].clone(),
            motd: status["description"].clone(),
            number: status["players"]["online"].as_i64().unwrap_or(0),
            max: status["players"]["max"].as_i64().unwrap_or(0),
            hide: status.get("players").is_none(),
            listed,
            players: HEARERS.lock().unwrap_or_else(|e| e.into_inner()).clone(),
            version: self::text(&status["version"], "name").to_owned(),
            protocol: status["version"]["protocol"].as_i64().unwrap_or(0),
            icon: status["favicon"].as_str().map(str::to_owned),
            status,
            cancelled: crate::event::prior_cancelled(),
        })
    }
    fn answer(&self) -> Answer {
        Answer::Pass
    }
    fn finish(&self) {
        query(json!({ "op": "ev_world_free_ping_set", "json": self.answer_text() }));
    }
}

// ---- Breaking a block: its experience and its drops ----

/// The tick a break said what it drops, for the game's sum that follows it.
static XP_DUE: Mutex<Option<u64>> = Mutex::new(None);

/// bukkit: org/bukkit/event/block/BlockBreakEvent
#[derive(Clone, Debug)]
pub struct BlockBreakEvent {
    pub player: Player,
    pub block: Block,
    pos: Value,
    exp: i64,
    worked_out: i64,
    drop: bool,
    cancelled: bool,
    /// A plugin's `callEvent`, not the game's break: the caller does what comes of it.
    raised: bool,
}

impl BlockBreakEvent {
    /// bukkit: org/bukkit/event/block/BlockBreakEvent.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> &Block {
        &self.block
    }
    /// bukkit: org/bukkit/event/block/BlockBreakEvent.isDropItems
    pub fn is_drop_items(&self) -> bool {
        self.drop
    }
    /// bukkit: org/bukkit/event/block/BlockBreakEvent.setDropItems
    pub fn set_drop_items(&mut self, drop: bool) {
        self.drop = drop;
    }
    /// bukkit: org/bukkit/event/block/BlockExpEvent.getExpToDrop
    pub fn get_exp_to_drop(&self) -> i64 {
        self.exp
    }
    /// bukkit: org/bukkit/event/block/BlockExpEvent.setExpToDrop
    pub fn set_exp_to_drop(&mut self, exp: i64) {
        self.exp = exp.max(0);
    }
    /// bukkit: org/bukkit/event/block/BlockBreakEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockBreakEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockBreakEvent {
    const HOOK: &'static str = "break_before";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_block_xp"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        value.get("xp_block")?;
        let now = crate::scheduler::now();
        let due = XP_DUE.lock().unwrap_or_else(|e| e.into_inner()).take_if(|tick| *tick == now).is_some();
        Some(if due { Answer::Fail } else { Answer::Pass })
    }
    fn read(value: &Value) -> Option<Self> {
        let player = player_of(value)?;
        let block = block_at(&player.get_world(), &value["pos"])?;
        let raised = value["raised"].as_bool().unwrap_or(false);
        let exp = if raised { 0 } else { query(json!({ "op": "ev_world_break_exp", "pos": value["pos"] })).as_i64().unwrap_or(0) };
        Some(Self { player, block, pos: value["pos"].clone(), exp, worked_out: exp, drop: true, cancelled: crate::event::prior_cancelled(), raised })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.raised {
            return;
        }
        if self.cancelled {
            return;
        }
        if !self.drop {
            query(json!({ "op": "no_drops" }));
        }
        if self.exp > 0 || self.exp != self.worked_out {
            let due = query(json!({ "op": "ev_world_xp_next", "pos": self.pos, "exp": self.exp })).as_bool().unwrap_or(false);
            if due {
                *XP_DUE.lock().unwrap_or_else(|e| e.into_inner()) = Some(crate::scheduler::now());
            }
        }
    }
}

/// bukkit: org/bukkit/event/block/BlockExpEvent
///
/// A block's break, whose listeners share this list in Java, and a grindstone's result taken.
#[derive(Clone, Debug)]
pub struct BlockExpEvent {
    block: Block,
    exp: i64,
    broken: Option<BlockBreakEvent>,
}

impl BlockExpEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> &Block {
        &self.block
    }
    /// bukkit: org/bukkit/event/block/BlockExpEvent.getExpToDrop
    pub fn get_exp_to_drop(&self) -> i64 {
        self.exp
    }
    /// bukkit: org/bukkit/event/block/BlockExpEvent.setExpToDrop
    pub fn set_exp_to_drop(&mut self, exp: i64) {
        self.exp = exp.max(0);
        if let Some(broken) = &mut self.broken {
            broken.set_exp_to_drop(exp);
        }
    }
}

impl Event for BlockExpEvent {
    const HOOK: &'static str = "break_before";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_block_xp", "ew_grindstone_xp"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        BlockBreakEvent::intercept(value)
    }
    fn read(value: &Value) -> Option<Self> {
        if let Some(exp) = value["grindstone"].as_i64() {
            return Some(Self { block: block_at(&world_of(value), &value["pos"])?, exp, broken: None });
        }
        let broken = BlockBreakEvent::read(value)?;
        Some(Self { block: broken.block.clone(), exp: broken.get_exp_to_drop(), broken: Some(broken) })
    }
    fn answer(&self) -> Answer {
        self.broken.as_ref().map_or(Answer::Pass, BlockBreakEvent::answer)
    }
    fn finish(&self) {
        match &self.broken {
            Some(broken) => broken.finish(),
            None => {
                query(json!({ "op": "ev_world_grind_xp", "exp": self.exp }));
            }
        }
    }
}

// ---- Explosions ----

#[derive(Clone, Copy, Debug)]
struct Blast {
    radius: f64,
    result: &'static str,
}

static BLAST: Mutex<Blast> = Mutex::new(Blast { radius: 1.0, result: "DESTROY" });
/// bukkit: org/bukkit/event/block/BlockExplodeEvent
///
/// A blast no entity set off: an entity's is `EntityExplodeEvent`.
#[derive(Clone, Debug)]
pub struct BlockExplodeEvent {
    value: Value,
    blocks: Vec<Block>,
    listed: usize,
    blast: Blast,
    keeps: Option<f64>,
    cancelled: bool,
}

impl BlockExplodeEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["block"])
    }
    /// bukkit: org/bukkit/event/block/BlockExplodeEvent.blockList
    pub fn block_list(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }
    /// bukkit: org/bukkit/event/block/BlockExplodeEvent.getExplodedBlockState
    pub fn get_exploded_block_state(&self) -> Option<BlockState> {
        match self.value["exploded"].as_str() {
            Some(state) => Some(BlockState::snapshot(self.get_block()?, state)),
            None => Some(self.get_block()?.get_state()),
        }
    }
    /// bukkit: org/bukkit/event/block/BlockExplodeEvent.getExplosionResult
    pub fn get_explosion_result(&self) -> &'static str {
        self.blast.result
    }
    fn usual_yield(&self) -> f64 {
        if self.blast.result == "DESTROY_WITH_DECAY" { 1.0 / self.blast.radius.max(0.001) } else { 1.0 }
    }
    /// bukkit: org/bukkit/event/block/BlockExplodeEvent.getYield
    pub fn get_yield(&self) -> f64 {
        self.keeps.unwrap_or_else(|| self.usual_yield())
    }
    /// bukkit: org/bukkit/event/block/BlockExplodeEvent.setYield
    ///
    /// The chance of each block dropping, which the loot tables roll against.
    pub fn set_yield(&mut self, chance: f64) {
        self.keeps = Some(chance.clamp(0.0, 1.0));
    }
    /// bukkit: org/bukkit/event/block/BlockExplodeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockExplodeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// Whether an entity set off the blast under way: `ee_detonate` comes first and says.
static BLAST_BY_ENTITY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl Event for BlockExplodeEvent {
    const HOOK: &'static str = "ew_explosion";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_explode", "ee_detonate"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if value["ee_blast"] == true {
            BLAST_BY_ENTITY.store(!value["entity"].is_null(), std::sync::atomic::Ordering::Relaxed);
            return Some(Answer::Pass);
        }
        if let Some(radius) = value["radius"].as_f64() {
            let result = match text(value, "interaction") {
                "Keep" => "KEEP",
                "DestroyWithDecay" => "DESTROY_WITH_DECAY",
                "TriggerBlock" => "TRIGGER_BLOCK",
                _ => "DESTROY",
            };
            *BLAST.lock().unwrap_or_else(|e| e.into_inner()) = Blast { radius, result };
            return Some(Answer::Pass);
        }
        None
    }
    fn read(value: &Value) -> Option<Self> {
        if BLAST_BY_ENTITY.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
        let world = world_of(value);
        let blocks: Vec<Block> = value["blocks"].as_array().into_iter().flatten().filter_map(|pos| block_at(&world, pos)).collect();
        let blast = *BLAST.lock().unwrap_or_else(|e| e.into_inner());
        Some(Self { value: value.clone(), listed: blocks.len(), blocks, blast, keeps: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.blocks.len() != self.listed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let Some(chance) = self.keeps {
            query(json!({ "op": "ev_world_blast_yield", "chance": chance }));
        }
        if !self.cancelled && self.blocks.len() != self.listed {
            let kept: Vec<[i32; 3]> = self.blocks.iter().map(|block| [block.get_x(), block.get_y(), block.get_z()]).collect();
            query(json!({ "op": "ev_world_keep_blocks", "blocks": kept }));
        }
    }
}

// ---- A block's progress, and level changes a listener sets ----

/// bukkit: io/papermc/paper/event/block/BlockBreakProgressUpdateEvent
#[derive(Clone, Debug)]
pub struct BlockBreakProgressUpdateEvent {
    value: Value,
    cancelled: bool,
}

hooked!(BlockBreakProgressUpdateEvent, ["ew_break_progress"], |value| (0..10).contains(&value["progress"].as_i64().unwrap_or(-1)));

impl BlockBreakProgressUpdateEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/BlockBreakProgressUpdateEvent.getProgress
    pub fn get_progress(&self) -> f64 {
        self.value["progress"].as_f64().unwrap_or(0.0) / 10.0
    }
    /// bukkit: io/papermc/paper/event/block/BlockBreakProgressUpdateEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "breaker")
    }
}

/// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent
///
/// Before the change. Rain and drips: see `BlockIgniteEvent`.
#[derive(Clone, Debug)]
pub struct CauldronLevelChangeEvent {
    value: Value,
    level: Option<i64>,
    cancelled: bool,
}

impl Event for CauldronLevelChangeEvent {
    const HOOK: &'static str = "ew_cauldron";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), level: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let (Some(level), false) = (self.level, self.cancelled) {
            query(json!({ "op": "ev_world_free_cauldron", "state": self.state_at(level) }));
            crate::event::carry("new_level", json!(level));
        }
    }
}

impl CauldronLevelChangeEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent.getNewState
    pub fn get_new_state(&self) -> Option<BlockState> {
        let state = self.level.map_or_else(|| text(&self.value, "new").to_owned(), |level| self.state_at(level));
        Some(BlockState::snapshot(self.get_block()?, &state))
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent.getOldLevel
    pub fn get_old_level(&self) -> i64 {
        self.value["old_level"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent.getNewLevel
    pub fn get_new_level(&self) -> i64 {
        self.level.unwrap_or_else(|| self.value["new_level"].as_i64().unwrap_or(0))
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent.setNewLevel
    pub fn set_new_level(&mut self, level: i64) {
        self.level = Some(level.clamp(0, 3));
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent.getReason
    pub fn get_reason(&self) -> &str {
        text(&self.value, "reason")
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        player_entity(&player_of(&self.value)?)
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    fn state_at(&self, level: i64) -> String {
        let state = text(&self.value, "new");
        let name = state.split('[').next().unwrap_or_default();
        let old = text(&self.value, "old");
        let filled = if name == "minecraft:cauldron" { old.split('[').next().unwrap_or_default() } else { name };
        if level == 0 {
            "minecraft:cauldron".to_owned()
        } else if filled == "minecraft:lava_cauldron" {
            filled.to_owned()
        } else if filled == "minecraft:cauldron" {
            format!("minecraft:water_cauldron[level={level}]")
        } else {
            format!("{filled}[level={level}]")
        }
    }
}

/// bukkit: org/bukkit/event/block/FluidLevelChangeEvent
#[derive(Clone, Debug)]
pub struct FluidLevelChangeEvent {
    value: Value,
    data: Option<String>,
    cancelled: bool,
}

impl Event for FluidLevelChangeEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "fluid").then(|| Self { value: value.clone(), data: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        self.write_data();
    }
}

impl FluidLevelChangeEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/FluidLevelChangeEvent.getNewData
    pub fn get_new_data(&self) -> &str {
        self.data.as_deref().unwrap_or_else(|| text(&self.value, "new"))
    }
    pub fn get_new_data_text(&self) -> &str {
        self.get_new_data()
    }
    /// bukkit: org/bukkit/event/block/FluidLevelChangeEvent.setNewData
    pub fn set_new_data(&mut self, data: &str) {
        self.data = Some(data.to_owned());
    }
    /// bukkit: org/bukkit/event/block/FluidLevelChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/FluidLevelChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    fn write_data(&self) {
        if let (Some(data), Some(block), false) = (&self.data, self.get_block(), self.cancelled) {
            block.set_block_data(data);
        }
    }
}

// ---- Raids ----

/// `org.bukkit.event.raid.RaidEvent`.
pub trait RaidEvent {
    /// bukkit~: org/bukkit/event/raid/RaidEvent.getRaid | not for RaidTriggerEvent, which comes before the raid exists; the raid is as its saved data last said
    fn get_raid(&self) -> Option<crate::fam_core::Raid>;
}

impl RaidEvent for RaidFinishEvent {
    fn get_raid(&self) -> Option<crate::fam_core::Raid> {
        self.get_world().get_raid(self.value["raid"].as_i64()? as i32)
    }
}



/// bukkit: org/bukkit/event/raid/RaidStopEvent
#[derive(Clone, Debug)]
pub struct RaidStopEvent {
    level: String,
    raid: i64,
    reason: String,
}

impl RaidStopEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        World::of(&self.level)
    }
    /// bukkit: org/bukkit/event/raid/RaidStopEvent.getReason
    pub fn get_reason(&self) -> &str {
        &self.reason
    }
}

impl RaidEvent for RaidStopEvent {
    fn get_raid(&self) -> Option<crate::fam_core::Raid> {
        self.get_world().get_raid(self.raid as i32)
    }
}

impl Event for RaidStopEvent {
    const HOOK: &'static str = "ew_raid_stop";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { level: text(value, "level").to_owned(), raid: value["raid"].as_i64()?, reason: text(value, "reason").to_owned() })
    }
}

/// bukkit: org/bukkit/event/raid/RaidSpawnWaveEvent
#[derive(Clone, Debug)]
pub struct RaidSpawnWaveEvent {
    value: Value,
    raiders: Value,
}

impl RaidSpawnWaveEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    pub fn get_raid_location(&self) -> Option<Location> {
        let at = |i: usize| self.value["pos"][i].as_f64();
        Some(Location::new(self.get_world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    pub fn get_wave(&self) -> i64 {
        self.value["wave"].as_i64().unwrap_or(0)
    }
    pub fn get_raid_id(&self) -> i64 {
        self.value["raid"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/raid/RaidSpawnWaveEvent.getRaiders
    pub fn get_raiders(&self) -> Vec<crate::entities::Raider> {
        let world = self.get_world();
        self.raiders["ids"].as_array().into_iter().flatten().filter_map(Value::as_u64).map(|id| crate::entities::Raider(Entity::of(id, &world))).collect()
    }
    /// bukkit: org/bukkit/event/raid/RaidSpawnWaveEvent.getPatrolLeader
    pub fn get_patrol_leader(&self) -> Option<crate::entities::Raider> {
        let id = self.raiders["leader"].as_u64()?;
        Some(crate::entities::Raider(Entity::of(id, &self.get_world())))
    }
}

impl RaidEvent for RaidSpawnWaveEvent {
    fn get_raid(&self) -> Option<crate::fam_core::Raid> {
        self.get_world().get_raid(self.get_raid_id() as i32)
    }
}

impl Event for RaidSpawnWaveEvent {
    const HOOK: &'static str = "ew_raid_wave";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { raiders: query(json!({ "op": "ev_world_raiders", "raid": value["raid"], "wave": value["wave"] })), value: value.clone() })
    }
}

/// bukkit: org/bukkit/event/Event.getHandlers
///
/// The list any event has, as a value: its methods are generic on the event here.
pub fn get_handlers<E: Event>(_: &E) -> HandlerList {
    HandlerList
}

/// bukkit~: org/bukkit/event/Event.callEvent | the event's hook payload is what's called with: an event here is read from it. `true` unless a listener refused
pub fn call_event<E: Event>(payload: &Value) -> bool {
    !crate::Plugin { host: horizon_mod::Host }.call_event(E::HOOK, payload)
}

// ---- Whitelist changes ----

/// bukkit: io/papermc/paper/event/server/WhitelistStateUpdateEvent
#[derive(Clone, Debug)]
pub struct WhitelistStateUpdateEvent {
    add: bool,
    name: String,
    id: String,
    cancelled: bool,
}

impl WhitelistStateUpdateEvent {
    /// bukkit: io/papermc/paper/event/server/WhitelistStateUpdateEvent.getStatus
    pub fn get_status(&self) -> &'static str {
        if self.add { "ADDED" } else { "REMOVED" }
    }
    /// bukkit: io/papermc/paper/event/server/WhitelistStateUpdateEvent.getPlayerProfile
    pub fn get_player_profile(&self) -> crate::player::PlayerProfile {
        crate::player::PlayerProfile { name: self.name.clone(), id: self.id.clone(), properties: Vec::new() }
    }
    /// bukkit: io/papermc/paper/event/server/WhitelistStateUpdateEvent.getPlayer
    pub fn get_player(&self) -> crate::OfflinePlayer {
        crate::get_offline_player(&self.name)
    }
    /// bukkit: io/papermc/paper/event/server/WhitelistStateUpdateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/server/WhitelistStateUpdateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WhitelistStateUpdateEvent {
    const HOOK: &'static str = "ew_whitelist_one";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { add: value["add"] == true, name: text(value, "name").to_owned(), id: text(value, "id").to_owned(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

// ---- Composters ----

/// What a listener made of the composting about to happen, by tick.
static COMPOST: Mutex<Option<(u64, i64)>> = Mutex::new(None);

/// bukkit: io/papermc/paper/event/block/CompostItemEvent
#[derive(Clone, Debug)]
pub struct CompostItemEvent {
    value: Value,
    levels: i64,
    worked_out: i64,
    cancelled: bool,
}

impl CompostItemEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/CompostItemEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: io/papermc/paper/event/block/CompostItemEvent.willRaiseLevel
    pub fn will_raise_level(&self) -> bool {
        self.levels > 0
    }
    /// bukkit: io/papermc/paper/event/block/CompostItemEvent.setWillRaiseLevel
    pub fn set_will_raise_level(&mut self, raise: bool) {
        self.levels = if raise { self.levels.max(1) } else { 0 };
    }
    /// bukkit: io/papermc/paper/event/block/CompostItemEvent.getLevelsToRaise
    pub fn get_levels_to_raise(&self) -> i64 {
        self.levels
    }
    /// bukkit: io/papermc/paper/event/block/CompostItemEvent.setLevelsToRaise
    pub fn set_levels_to_raise(&mut self, levels: i64) {
        self.levels = levels.max(0);
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for CompostItemEvent {
    const HOOK: &'static str = "ew_compost";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_compost_add"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        value.get("adding")?;
        let now = crate::scheduler::now();
        let Some((_, levels)) = COMPOST.lock().unwrap_or_else(|e| e.into_inner()).take().filter(|(tick, _)| *tick == now) else {
            return Some(Answer::Pass);
        };
        query(json!({ "op": "ev_world_compost", "pos": value["pos"], "levels": levels }));
        Some(Answer::Fail)
    }
    fn read(value: &Value) -> Option<Self> {
        let levels = value["levels"].as_i64()?;
        Some(Self { value: value.clone(), levels, worked_out: levels, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled && self.levels != self.worked_out {
            *COMPOST.lock().unwrap_or_else(|e| e.into_inner()) = Some((crate::scheduler::now(), self.levels));
        }
    }
}

// ---- The dragon's egg, maps, and vaults ----

/// bukkit: io/papermc/paper/event/block/DragonEggFormEvent
#[derive(Clone, Debug)]
pub struct DragonEggFormEvent {
    value: Value,
    cancelled: bool,
}

impl DragonEggFormEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    pub fn get_new_state(&self) -> Option<BlockState> {
        Some(BlockState::snapshot(self.get_block()?, "minecraft:dragon_egg"))
    }
    /// bukkit: io/papermc/paper/event/block/DragonEggFormEvent.getDragonBattle
    pub fn get_dragon_battle(&self) -> crate::fam_core::DragonBattle {
        crate::fam_core::DragonBattle
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for DragonEggFormEvent {
    const HOOK: &'static str = "ew_dragon_egg";
    fn read(value: &Value) -> Option<Self> {
        value["egg"].as_bool().unwrap_or(false).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ev_world_no_egg" }));
        }
    }
}


/// bukkit: org/bukkit/event/server/MapInitializeEvent
#[derive(Clone, Debug)]
pub struct MapInitializeEvent {
    map: crate::fam_core::MapView,
}

impl MapInitializeEvent {
    /// bukkit: org/bukkit/event/server/MapInitializeEvent.getMap
    pub fn get_map(&self) -> crate::fam_core::MapView {
        self.map
    }
}

impl Event for MapInitializeEvent {
    const HOOK: &'static str = "ew_map_new";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { map: crate::fam_core::get_map(value["map"].as_i64()? as i32)? })
    }
}

/// bukkit: org/bukkit/event/block/VaultDisplayItemEvent
///
/// As the vault chooses, before any client hears of it.
#[derive(Clone, Debug)]
pub struct VaultDisplayItemEvent {
    value: Value,
    item: Option<ItemStack>,
    changed: bool,
    cancelled: bool,
}

impl VaultDisplayItemEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/VaultDisplayItemEvent.getDisplayItem
    pub fn get_display_item(&self) -> Option<&ItemStack> {
        self.item.as_ref()
    }
    /// bukkit: org/bukkit/event/block/VaultDisplayItemEvent.setDisplayItem
    pub fn set_display_item(&mut self, item: Option<ItemStack>) {
        self.item = item;
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/block/VaultDisplayItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/VaultDisplayItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for VaultDisplayItemEvent {
    const HOOK: &'static str = "ew_vault_display";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), item: ItemStack::read(&value["stack"]), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.changed && !self.cancelled {
            query(json!({ "op": "ev_world_vault_shows", "item": self.item.as_ref().map(ItemStack::to_json) }));
        }
    }
}

/// bukkit: org/bukkit/event/block/BlockDispenseLootEvent
///
/// A vault's whole roll as its key goes in, or a trial spawner's reward.
#[derive(Clone, Debug)]
pub struct BlockDispenseLootEvent {
    value: Value,
    items: Vec<ItemStack>,
    listed: usize,
    cancelled: bool,
}

impl BlockDispenseLootEvent {
    /// bukkit: org/bukkit/event/block/BlockDispenseLootEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        self.value["uuid"].is_string().then(|| player_of(&self.value)).flatten()
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseLootEvent.getDispensedLoot
    pub fn get_dispensed_loot(&self) -> &[ItemStack] {
        &self.items
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseLootEvent.setDispensedLoot
    pub fn set_dispensed_loot(&mut self, items: Vec<ItemStack>) {
        self.items = items;
        self.listed = usize::MAX;
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseLootEvent.getLootTable
    pub fn get_loot_table(&self) -> Option<crate::fam_core::LootTable> {
        if let Some(table) = self.value["table"].as_str() {
            return crate::fam_core::get_loot_table(table);
        }
        let data = query(json!({ "op": "block_data", "dimension": world_of(&self.value).dimension(), "pos": self.value["pos"] }));
        crate::fam_core::get_loot_table(data["config"]["loot_table"].as_str().unwrap_or("minecraft:chests/trial_chambers/reward"))
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseLootEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockDispenseLootEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockDispenseLootEvent {
    const HOOK: &'static str = "ew_dispense_loot";
    fn read(value: &Value) -> Option<Self> {
        let items: Vec<ItemStack> = value["ejects"].as_array()?.iter().filter_map(ItemStack::read).collect();
        Some(Self { value: value.clone(), listed: items.len(), items, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled && self.items.len() != self.listed {
            let kept: Vec<Value> = self.items.iter().map(ItemStack::to_json).collect();
            query(json!({ "op": "ev_world_set_drops", "items": kept }));
        }
    }
}

// ---- Note blocks ----

/// bukkit: org/bukkit/event/block/NotePlayEvent
///
/// A click or redstone, as the note is about to sound: what a listener sets lasts for this note.
#[derive(Clone, Debug)]
pub struct NotePlayEvent {
    value: Value,
    note: u8,
    instrument: String,
    changed: bool,
    cancelled: bool,
}

fn state_property<'a>(state: &'a str, key: &str) -> Option<&'a str> {
    state.split_once('[')?.1.trim_end_matches(']').split(',').find_map(|row| row.split_once('=').filter(|(name, _)| *name == key).map(|(_, value)| value))
}

impl NotePlayEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/NotePlayEvent.getNote
    pub fn get_note(&self) -> crate::core2::Note {
        crate::core2::Note(self.note)
    }
    /// bukkit: org/bukkit/event/block/NotePlayEvent.setNote
    pub fn set_note(&mut self, note: crate::core2::Note) {
        self.note = note.0.min(24);
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/block/NotePlayEvent.getInstrument
    ///
    /// Bukkit's name: `PIANO`, `BASS_DRUM`...
    pub fn get_instrument(&self) -> String {
        match self.instrument.as_str() {
            "harp" => "PIANO".to_owned(),
            "basedrum" => "BASS_DRUM".to_owned(),
            "snare" => "SNARE_DRUM".to_owned(),
            "hat" => "STICKS".to_owned(),
            "bass" => "BASS_GUITAR".to_owned(),
            other => other.to_ascii_uppercase(),
        }
    }
    /// bukkit: org/bukkit/event/block/NotePlayEvent.setInstrument
    ///
    /// Takes Bukkit's name, as `get_instrument` gives it.
    pub fn set_instrument(&mut self, instrument: &str) {
        self.instrument = match instrument {
            "PIANO" => "harp".to_owned(),
            "BASS_DRUM" => "basedrum".to_owned(),
            "SNARE_DRUM" => "snare".to_owned(),
            "STICKS" => "hat".to_owned(),
            "BASS_GUITAR" => "bass".to_owned(),
            other => other.to_ascii_lowercase(),
        };
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/block/NotePlayEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/NotePlayEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for NotePlayEvent {
    const HOOK: &'static str = "ew_note";
    fn more_hooks() -> &'static [&'static str] {
        &["tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if value.is_null() {
            query(json!({ "op": "ev_world_install" }));
            return Some(Answer::Pass);
        }
        None
    }
    fn read(value: &Value) -> Option<Self> {
        let state = text(value, "state");
        let note = state_property(state, "note")?.parse::<u8>().ok()?;
        Some(Self { value: value.clone(), note, instrument: state_property(state, "instrument")?.to_owned(), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.changed && !self.cancelled {
            query(json!({ "op": "ev_world_free_note", "note": self.note, "instrument": self.instrument }));
        }
    }
}

// ---- Worlds going away, loot, and brewing ----

/// bukkit: org/bukkit/event/world/WorldUnloadEvent
#[derive(Clone, Debug)]
pub struct WorldUnloadEvent {
    world: World,
    cancelled: bool,
}

impl WorldUnloadEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> &World {
        &self.world
    }
    /// bukkit: org/bukkit/event/world/WorldUnloadEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/world/WorldUnloadEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WorldUnloadEvent {
    const HOOK: &'static str = "ew_level_close";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        value["closing"].as_array().into_iter().flatten().filter_map(Value::as_str).map(|level| Self { world: World::of(level), cancelled: crate::event::prior_cancelled() }).collect()
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/world/LootGenerateEvent
///
/// A mob's or a block's drops, and a container's loot when it's first opened.
#[derive(Clone, Debug)]
pub struct LootGenerateEvent {
    value: Value,
    loot: Vec<ItemStack>,
    listed: usize,
    cancelled: bool,
}

impl LootGenerateEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.getLootTable
    pub fn get_loot_table(&self) -> Option<crate::fam_core::LootTable> {
        crate::fam_core::get_loot_table(text(&self.value, "table"))
    }
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.getLoot
    pub fn get_loot(&self) -> &[ItemStack] {
        &self.loot
    }
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.setLoot
    pub fn set_loot(&mut self, loot: Vec<ItemStack>) {
        self.loot = loot;
        self.listed = usize::MAX;
    }
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.getEntity
    ///
    /// The mob that dropped it, or the player who broke the block.
    pub fn get_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "entity").or_else(|| player_entity(&player_of(&self.value)?))
    }
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.getInventoryHolder
    ///
    /// The container the loot goes in: a block or a mob isn't one.
    pub fn get_inventory_holder(&self) -> Option<Block> {
        if self.value["container"] != true {
            return None;
        }
        block_at(&self.get_world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.isPlugin
    ///
    /// `false`: a plugin's own loot table roll doesn't reach this event.
    pub fn is_plugin(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for LootGenerateEvent {
    const HOOK: &'static str = "ew_loot";
    fn read(value: &Value) -> Option<Self> {
        if value["container"] != true {
            return None;
        }
        let loot: Vec<ItemStack> = value["items"].as_array()?.iter().filter_map(ItemStack::read).collect();
        Some(Self { value: value.clone(), listed: loot.len(), loot, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.loot.len() != self.listed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.cancelled || self.loot.len() != self.listed {
            let kept: Vec<Value> = if self.cancelled { Vec::new() } else { self.loot.iter().map(ItemStack::to_json).collect() };
            query(json!({ "op": "ev_world_set_drops", "items": kept }));
        }
    }
}


/// bukkit: org/bukkit/event/block/BrewingStartEvent
#[derive(Clone, Debug)]
pub struct BrewingStartEvent {
    value: Value,
    brewing: i64,
    recipe: i64,
    changed: bool,
}

impl BrewingStartEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BrewingStartEvent.getBrewingTime
    pub fn get_brewing_time(&self) -> i64 {
        self.brewing
    }
    /// bukkit: org/bukkit/event/block/BrewingStartEvent.setBrewingTime
    pub fn set_brewing_time(&mut self, ticks: i64) {
        self.brewing = ticks.max(0);
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/block/BrewingStartEvent.getRecipeBrewTime
    pub fn get_recipe_brew_time(&self) -> i64 {
        self.recipe
    }
    /// bukkit: org/bukkit/event/block/BrewingStartEvent.setRecipeBrewTime
    pub fn set_recipe_brew_time(&mut self, ticks: i64) {
        self.recipe = ticks.max(1);
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/block/BrewingStartEvent.getTotalBrewTime
    pub fn get_total_brew_time(&self) -> i64 {
        self.get_recipe_brew_time()
    }
    /// bukkit: org/bukkit/event/block/BrewingStartEvent.setTotalBrewTime
    pub fn set_total_brew_time(&mut self, ticks: i64) {
        self.set_recipe_brew_time(ticks);
    }
}

impl InventoryBlockStartEvent for BrewingStartEvent {
    fn get_source(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
}

impl Event for BrewingStartEvent {
    const HOOK: &'static str = "ew_brewing";
    fn read(value: &Value) -> Option<Self> {
        let brewing = value["brew_time"].as_i64()?;
        Some(Self { value: value.clone(), brewing, recipe: value["total"].as_i64().unwrap_or(brewing), changed: false })
    }
    fn finish(&self) {
        if self.changed {
            query(json!({ "op": "ev_world_brewing", "pos": self.value["pos"], "dimension": self.value["level"], "brewing": self.brewing, "recipe": self.recipe }));
        }
    }
}

// ---- Who hit a vehicle or a hanging, as a damage source ----

fn hit_by(player: Option<Player>, fire: bool) -> crate::core2::DamageSource {
    match (player.as_ref().and_then(player_entity), fire) {
        (Some(attacker), _) => crate::core2::DamageSource::builder("minecraft:player_attack").with_causing_entity(&attacker).with_direct_entity(&attacker),
        (None, true) => crate::core2::DamageSource::builder("minecraft:on_fire"),
        (None, false) => crate::core2::DamageSource::builder("minecraft:generic"),
    }
}

impl VehicleDamageEvent {
    /// bukkit~: org/bukkit/event/vehicle/VehicleDamageEvent.getDamageSource | worked out from who hit it: the game doesn't pass one to a vehicle
    pub fn get_damage_source(&self) -> crate::core2::DamageSource {
        hit_by(self.get_attacker(), self.value["fire"].as_bool().unwrap_or(false))
    }
}

impl VehicleDestroyEvent {
    /// bukkit~: org/bukkit/event/vehicle/VehicleDestroyEvent.getDamageSource | worked out from who hit it: the game doesn't pass one to a vehicle
    pub fn get_damage_source(&self) -> crate::core2::DamageSource {
        hit_by(self.get_attacker(), self.value["fire"].as_bool().unwrap_or(false))
    }
}

impl HangingBreakByEntityEvent {
    /// bukkit~: org/bukkit/event/hanging/HangingBreakByEntityEvent.getDamageSource | worked out from the player: the game doesn't pass one to a hanging
    pub fn get_damage_source(&self) -> crate::core2::DamageSource {
        hit_by(player_of(&self.value), false)
    }
}

// ---- A result a listener replaces ----

/// A stack a listener put in the game's place: the same call comes back through the hook with it.
static MY_RESULT: Mutex<Option<(u64, String)>> = Mutex::new(None);

fn my_result(stack: &Value) -> bool {
    let now = crate::scheduler::now();
    let item = text(stack, "item");
    MY_RESULT.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|(tick, mine)| *tick == now && mine == item)
}

fn replace_result(stack: &ItemStack) {
    *MY_RESULT.lock().unwrap_or_else(|e| e.into_inner()) = Some((crate::scheduler::now(), stack.get_type().key().to_owned()));
    query(json!({ "op": "ev_world_result", "stack": stack.to_json() }));
}

/// bukkit: org/bukkit/event/block/CrafterCraftEvent
#[derive(Clone, Debug)]
pub struct CrafterCraftEvent {
    value: Value,
    result: Option<ItemStack>,
    changed: bool,
    cancelled: bool,
}

impl CrafterCraftEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/CrafterCraftEvent.getResult
    pub fn get_result(&self) -> Option<&ItemStack> {
        self.result.as_ref()
    }
    /// bukkit: org/bukkit/event/block/CrafterCraftEvent.setResult
    pub fn set_result(&mut self, result: ItemStack) {
        self.result = Some(result);
        self.changed = true;
    }
    pub fn get_recipe_key(&self) -> &str {
        text(&self.value, "recipe")
    }
    /// bukkit: org/bukkit/event/block/CrafterCraftEvent.getRecipe
    pub fn get_recipe(&self) -> Option<crate::core2::GameRecipe> {
        crate::core2::get_recipe(&crate::persistence::NamespacedKey::from_string(self.get_recipe_key())?)
    }
    /// bukkit: org/bukkit/event/block/CrafterCraftEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/CrafterCraftEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for CrafterCraftEvent {
    const HOOK: &'static str = "ew_crafter";
    fn read(value: &Value) -> Option<Self> {
        (!my_result(&value["stack"])).then(|| Self { value: value.clone(), result: ItemStack::read(&value["stack"]), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let (true, false, Some(result)) = (self.changed, self.cancelled, &self.result) {
            replace_result(result);
        }
    }
}

/// bukkit: org/bukkit/event/block/BlockCookEvent, org/bukkit/event/inventory/FurnaceSmeltEvent
///
/// A campfire's, a furnace's, a smoker's or a blast furnace's: `FurnaceSmeltEvent` adds nothing to it.
#[derive(Clone, Debug)]
pub struct BlockCookEvent {
    value: Value,
    result: Option<ItemStack>,
    changed: bool,
    cancelled: bool,
}

impl BlockCookEvent {
    /// bukkit: org/bukkit/event/block/BlockCookEvent.getSource
    pub fn get_source(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["source"])
    }
    /// bukkit: org/bukkit/event/block/BlockCookEvent.getRecipe
    pub fn get_recipe(&self) -> Option<crate::core2::GameRecipe> {
        crate::core2::get_recipe(&crate::persistence::NamespacedKey::from_string(self.value["recipe"].as_str()?)?)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockCookEvent.getResult
    pub fn get_result(&self) -> Option<&ItemStack> {
        self.result.as_ref()
    }
    /// bukkit: org/bukkit/event/block/BlockCookEvent.setResult
    pub fn set_result(&mut self, result: ItemStack) {
        self.result = Some(result);
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/block/BlockCookEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockCookEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockCookEvent {
    const HOOK: &'static str = "ew_campfire_done";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_furnace_done"]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["furnace"] == true || !my_result(&value["stack"])).then(|| Self { value: value.clone(), result: ItemStack::read(&value["stack"]), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        let (true, false, Some(result)) = (self.changed, self.cancelled, &self.result) else {
            return;
        };
        if self.value["furnace"] == true {
            query(json!({ "op": "ev_world_furnace_result", "item": result.to_json() }));
        } else {
            replace_result(result);
        }
    }
}

// ---- Forms, targets and destruction ----

/// bukkit~: org/bukkit/event/block/EntityBlockFormEvent | snow golems' trails, as mobs tick: the entity is the nearest golem, as a mob's block change doesn't say whose it was
#[derive(Clone, Debug)]
pub struct EntityBlockFormEvent {
    value: Value,
    cancelled: bool,
}

hooked!(EntityBlockFormEvent, ["block_change"], |value| value["kind"] == "entity_change" && text(value, "new").starts_with("minecraft:snow"));

impl EntityBlockFormEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    pub fn get_new_state(&self) -> Option<BlockState> {
        Some(BlockState::snapshot(self.get_block()?, text(&self.value, "new")))
    }
    /// bukkit: org/bukkit/event/block/EntityBlockFormEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        let world = world_of(&self.value);
        let at = |i: usize| self.value["pos"][i].as_f64().unwrap_or(0.0);
        let mobs = query(json!({ "op": "mobs", "dimension": world.dimension() }));
        let distance = |mob: &Value| (0..3).map(|i| (mob["pos"][i].as_f64().unwrap_or(0.0) - at(i)).powi(2)).sum::<f64>();
        let golem = mobs.as_array()?.iter().filter(|mob| text(mob, "kind").ends_with("snow_golem")).min_by(|a, b| distance(a).total_cmp(&distance(b)))?;
        Some(Entity::of(golem["id"].as_u64()?, &world))
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/block/TargetHitEvent
#[derive(Clone, Debug)]
pub struct TargetHitEvent {
    value: Value,
    strength: Option<i64>,
    cancelled: bool,
}

impl TargetHitEvent {
    /// bukkit: io/papermc/paper/event/block/TargetHitEvent.getSignalStrength
    pub fn get_signal_strength(&self) -> i64 {
        self.strength.unwrap_or_else(|| self.value["strength"].as_i64().unwrap_or(0))
    }
    /// bukkit: io/papermc/paper/event/block/TargetHitEvent.setSignalStrength
    pub fn set_signal_strength(&mut self, strength: i64) {
        self.strength = Some(strength.clamp(0, 15));
    }
    pub fn get_hit_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "entity")
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for TargetHitEvent {
    const HOOK: &'static str = "ew_target_hit";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), strength: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if let (Some(strength), false) = (self.strength, self.cancelled) {
            query(json!({ "op": "ev_world_target_power", "strength": strength }));
        }
    }
}

/// bukkit~: org/bukkit/event/vehicle/VehicleEntityCollisionEvent, org/bukkit/event/vehicle/VehicleCollisionEvent | a boat pushing or taking a mob: a minecart's push runs in one sweep with no moment for each pair
#[derive(Clone, Debug)]
pub struct VehicleEntityCollisionEvent {
    value: Value,
    pickup_cancelled: bool,
    cancelled: bool,
}

impl Event for VehicleEntityCollisionEvent {
    const HOOK: &'static str = "ew_boat_push";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_boat_board"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), pickup_cancelled: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || (self.pickup_cancelled && self.value["boards"].as_bool().unwrap_or(false)) { Answer::Fail } else { Answer::Pass }
    }
}
vehicle!(VehicleEntityCollisionEvent);

impl VehicleEntityCollisionEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleEntityCollisionEvent.isPickupCancelled
    pub fn is_pickup_cancelled(&self) -> bool {
        self.pickup_cancelled
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleEntityCollisionEvent.setPickupCancelled
    pub fn set_pickup_cancelled(&mut self, cancelled: bool) {
        self.pickup_cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleEntityCollisionEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        entity_at(&self.value, "entity")
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleEntityCollisionEvent.isCollisionCancelled
    pub fn is_collision_cancelled(&self) -> bool {
        self.cancelled
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleEntityCollisionEvent.setCollisionCancelled
    pub fn set_collision_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleEntityCollisionEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleEntityCollisionEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: io/papermc/paper/event/world/WorldDifficultyChangeEvent
#[derive(Clone, Debug)]
pub struct WorldDifficultyChangeEvent {
    value: Value,
    cancelled: bool,
}

hooked!(WorldDifficultyChangeEvent, ["ew_difficulty"]);

impl WorldDifficultyChangeEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    pub fn get_difficulty_number(&self) -> i64 {
        self.value["difficulty"].as_i64().unwrap_or(0)
    }
    /// bukkit: io/papermc/paper/event/world/WorldDifficultyChangeEvent.getDifficulty
    pub fn get_difficulty(&self) -> &'static str {
        crate::enums::difficulty::values().get(self.get_difficulty_number() as usize).copied().unwrap_or("NORMAL")
    }
    /// bukkit: io/papermc/paper/event/world/WorldDifficultyChangeEvent.getCommandSource
    pub fn get_command_source(&self) -> crate::brigadier::CommandContext {
        crate::brigadier::CommandContext::of_sender(player_of(&self.value).map_or(CommandSender::Console, CommandSender::Player))
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl UnknownCommandEvent {
    /// bukkit: org/bukkit/event/command/UnknownCommandEvent.getCommandSource
    pub fn get_command_source(&self) -> crate::brigadier::CommandContext {
        crate::brigadier::CommandContext::of_sender(self.get_sender())
    }
}

// ---- Locked containers ----

/// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent
#[derive(Clone, Debug)]
pub struct BlockLockCheckEvent {
    value: Value,
    key: Option<ItemStack>,
    custom: bool,
    result: &'static str,
    message: Option<Component>,
    sound: Option<String>,
}

impl BlockLockCheckEvent {
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.get_player()?.get_world(), &self.value["pos"])
    }
    /// bukkit~: io/papermc/paper/event/block/BlockLockCheckEvent.getBlockState | as a plain block state: a `LockableTileState` cast is `block_data`'s to read
    pub fn get_block_state(&self) -> Option<BlockState> {
        Some(self.get_block()?.get_state())
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.getKeyItem
    pub fn get_key_item(&self) -> Option<&ItemStack> {
        self.key.as_ref()
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.setKeyItem
    pub fn set_key_item(&mut self, stack: ItemStack) {
        self.key = Some(stack);
        self.custom = true;
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.resetKeyItem
    pub fn reset_key_item(&mut self) {
        self.key = self.get_player().and_then(|player| ItemStack::read(&query(json!({ "op": "equipment", "uuid": player.uuid() }))["main_hand"]));
        self.custom = false;
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.isUsingCustomKeyItemStack
    pub fn is_using_custom_key_item_stack(&self) -> bool {
        self.custom
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.getResult
    pub fn get_result(&self) -> &'static str {
        self.result
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.setResult
    pub fn set_result(&mut self, result: &'static str) {
        self.result = result;
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.denyWithMessageAndSound
    pub fn deny_with_message_and_sound(&mut self, message: Option<Component>, sound: Option<&str>) {
        self.result = "DENY";
        self.message = message;
        self.sound = sound.map(str::to_owned);
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.getLockedMessage
    pub fn get_locked_message(&self) -> Option<&Component> {
        self.message.as_ref()
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.setLockedMessage
    pub fn set_locked_message(&mut self, message: Option<Component>) {
        self.message = message;
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.getLockedSound
    pub fn get_locked_sound(&self) -> Option<&str> {
        self.sound.as_deref()
    }
    /// bukkit: io/papermc/paper/event/block/BlockLockCheckEvent.setLockedSound
    pub fn set_locked_sound(&mut self, sound: Option<&str>) {
        self.sound = sound.map(str::to_owned);
    }
}

impl Event for BlockLockCheckEvent {
    const HOOK: &'static str = "open_try";
    fn read(value: &Value) -> Option<Self> {
        if !query(json!({ "op": "ev_world_lockable", "pos": value["pos"] })).as_bool().unwrap_or(false) {
            return None;
        }
        let mut event = Self { value: value.clone(), key: None, custom: false, result: "DEFAULT", message: None, sound: None };
        event.reset_key_item();
        Some(event)
    }
    fn answer(&self) -> Answer {
        if self.result == "DENY" { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.result == "ALLOW" || self.custom {
            query(json!({ "op": "ev_world_lock", "pos": self.value["pos"], "allow": self.result == "ALLOW", "key": self.key.as_ref().map(ItemStack::to_json) }));
        }
        let (true, Some(player)) = (self.result == "DENY", self.get_player()) else {
            return;
        };
        if let Some(message) = &self.message {
            player.send_message(&message.plain());
        }
        if let Some(sound) = &self.sound {
            horizon_mod::run_command(&format!("playsound {sound} block {}", player.get_name()));
        }
    }
}

/// bukkit: io/papermc/paper/event/world/StructuresLocateEvent
///
/// Before any search: `/locate`, an eye of ender's, a dolphin's or a map's, on the thread that's searching.
#[derive(Clone, Debug)]
pub struct StructuresLocateEvent {
    value: Value,
    result: Option<LocateResult>,
    structures: Vec<String>,
    radius: i64,
    unexplored: bool,
    cancelled: bool,
}

impl StructuresLocateEvent {
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.getOrigin
    pub fn get_origin(&self) -> Option<Location> {
        let at = |i: usize| self.value["origin"][i].as_f64();
        Some(Location::new(self.get_world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.getStructures
    ///
    /// The structures' keys.
    pub fn get_structures(&self) -> &[String] {
        &self.structures
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.setStructures
    pub fn set_structures(&mut self, structures: Vec<String>) {
        self.structures = structures;
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.getRadius
    pub fn get_radius(&self) -> i64 {
        self.radius
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.setRadius
    pub fn set_radius(&mut self, radius: i64) {
        self.radius = radius.max(0);
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.shouldFindUnexplored
    pub fn should_find_unexplored(&self) -> bool {
        self.unexplored
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.setFindUnexplored
    ///
    /// Past the structures a map has already pointed at, as an explorer map's search is.
    pub fn set_find_unexplored(&mut self, unexplored: bool) {
        self.unexplored = unexplored;
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.getResult
    pub fn get_result(&self) -> Option<&LocateResult> {
        self.result.as_ref()
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.setResult
    pub fn set_result(&mut self, result: LocateResult) {
        self.result = Some(result);
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for StructuresLocateEvent {
    const HOOK: &'static str = "ew_locate";
    fn more_hooks() -> &'static [&'static str] {
        &["tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if value.is_null() {
            query(json!({ "op": "ev_world_install" }));
            return Some(Answer::Pass);
        }
        None
    }
    fn read(value: &Value) -> Option<Self> {
        // What the listener before this one left.
        let now = query(json!({ "op": "ev_world_free_locate_now" }));
        let asked = if now.is_null() { value } else { &now };
        let structures = asked["structures"].as_array()?.iter().filter_map(Value::as_str).map(str::to_owned).collect();
        let world = world_of(value);
        let result = asked["result"].as_array().and_then(|row| {
            let at = |i: usize| row.get(i).and_then(Value::as_f64);
            Some(LocateResult::new(row.first()?.as_str()?, Location::new(world.clone(), at(1)?, at(2)?, at(3)?, 0.0, 0.0)))
        });
        Some(Self { value: value.clone(), result, structures, radius: asked["radius"].as_i64().unwrap_or(100), unexplored: asked["unexplored"] == true, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        let result = self.result.as_ref().map(|result| json!([result.structure, result.at.get_x(), result.at.get_y(), result.at.get_z()]));
        query(json!({ "op": "ev_world_free_locate_set", "structures": self.structures, "radius": self.radius, "unexplored": self.unexplored, "result": result }));
    }
}

/// When the tick going on began.
static TICK_BEGAN: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// bukkit: com/destroystokyo/paper/event/server/ServerTickEndEvent
#[derive(Clone, Debug)]
pub struct ServerTickEndEvent {
    took: f64,
}

impl ServerTickEndEvent {
    /// bukkit: com/destroystokyo/paper/event/server/ServerTickEndEvent.getTickNumber
    pub fn get_tick_number(&self) -> i64 {
        query(json!({ "op": "tick_times" }))["ticks"].as_i64().unwrap_or(0)
    }
    /// bukkit: com/destroystokyo/paper/event/server/ServerTickEndEvent.getTickDuration
    pub fn get_tick_duration(&self) -> f64 {
        self.took
    }
    /// bukkit: com/destroystokyo/paper/event/server/ServerTickEndEvent.getTimeRemaining
    pub fn get_time_remaining(&self) -> i64 {
        let rate = query(json!({ "op": "tick_state" }))["rate"].as_f64().unwrap_or(20.0).max(0.001);
        ((1000.0 / rate - self.took) * 1_000_000.0) as i64
    }
}

impl Event for ServerTickEndEvent {
    const HOOK: &'static str = "tick";
    fn more_hooks() -> &'static [&'static str] {
        &["tick_start"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        value.get("server_tick")?;
        *TICK_BEGAN.lock().unwrap_or_else(|e| e.into_inner()) = Some(std::time::Instant::now());
        Some(Answer::Pass)
    }
    fn read(_: &Value) -> Option<Self> {
        let began = TICK_BEGAN.lock().unwrap_or_else(|e| e.into_inner()).take();
        Some(Self { took: began.map_or(0.0, |at| at.elapsed().as_secs_f64() * 1000.0) })
    }
}

// ---- Beacons coming on and going off ----



/// 1 for a beacon that came on, -1 for one that went off.
/// bukkit: io/papermc/paper/event/block/BeaconActivatedEvent
#[derive(Clone, Debug)]
pub struct BeaconActivatedEvent {
    value: Value,
}

impl BeaconActivatedEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/BeaconActivatedEvent.getBeacon
    pub fn get_beacon(&self) -> Option<crate::blocks::Beacon> {
        Some(self.get_block()?.tile_state())
    }
}

impl Event for BeaconActivatedEvent {
    const HOOK: &'static str = "ew_beacon";
    fn read(value: &Value) -> Option<Self> {
        (value["on"] == true).then(|| Self { value: value.clone() })
    }
}

/// bukkit: io/papermc/paper/event/block/BeaconDeactivatedEvent
///
/// A base or the beacon itself gone: `get_beacon` is then `None`.
#[derive(Clone, Debug)]
pub struct BeaconDeactivatedEvent {
    value: Value,
}

impl BeaconDeactivatedEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/BeaconDeactivatedEvent.getBeacon
    pub fn get_beacon(&self) -> Option<crate::blocks::Beacon> {
        let block = self.get_block()?;
        (block.get_type().key() == "minecraft:beacon").then(|| block.tile_state())
    }
}

impl Event for BeaconDeactivatedEvent {
    const HOOK: &'static str = "ew_beacon";
    fn more_hooks() -> &'static [&'static str] {
        &["block_entity_unload"]
    }
    fn read(value: &Value) -> Option<Self> {
        let off = value["on"] == false || query(json!({ "op": "ev_world_beacon_gone", "level": value["level"], "pos": value["pos"] })) == true;
        off.then(|| Self { value: value.clone() })
    }
}

/// `StructuresLocateEvent.Result`: where a structure is.
#[derive(Clone, Debug, PartialEq)]
pub struct LocateResult {
    structure: String,
    at: Location,
}

impl LocateResult {
    pub fn new(structure: &str, at: Location) -> Self {
        Self { structure: structure.to_owned(), at }
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent$Result.structure
    pub fn structure(&self) -> &str {
        &self.structure
    }
    /// bukkit: io/papermc/paper/event/world/StructuresLocateEvent$Result.position, io/papermc/paper/event/world/StructuresLocateEvent$Result.pos
    pub fn position(&self) -> &Location {
        &self.at
    }
}

impl LootGenerateEvent {
    /// bukkit: org/bukkit/event/world/LootGenerateEvent.getLootContext
    ///
    /// With the killer's luck: `LootContext.getLootingModifier` is `-1`, the class's own "use the default".
    pub fn get_loot_context(&self) -> crate::fam_core::LootContext {
        let looted = self.get_entity();
        let place = self.value["origin"].as_array().and_then(|at| Some(Location::new(self.get_world(), at.first()?.as_f64()?, at.get(1)?.as_f64()?, at.get(2)?.as_f64()?, 0.0, 0.0)));
        let context = crate::fam_core::LootContext::builder(place, looted);
        match player_of(&self.value) {
            Some(killer) => context.luck(self.value["luck"].as_f64().unwrap_or(0.0) as f32).killer(&killer),
            None => context,
        }
    }
}

// ---- Bells ----

/// bukkit: org/bukkit/event/block/BellResonateEvent
#[derive(Clone, Debug)]
pub struct BellResonateEvent {
    value: Value,
    entities: Vec<Entity>,
    listed: usize,
}

impl BellResonateEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BellResonateEvent.getResonatedEntities
    ///
    /// Who is about to glow: take one out and it doesn't.
    pub fn get_resonated_entities(&mut self) -> &mut Vec<Entity> {
        &mut self.entities
    }
}

impl Event for BellResonateEvent {
    const HOOK: &'static str = "ew_bell_resonate";
    fn read(value: &Value) -> Option<Self> {
        let world = world_of(value);
        let entities: Vec<Entity> = value["raiders"].as_array()?.iter().filter_map(Value::as_u64).map(|id| Entity::of(id, &world)).collect();
        Some(Self { value: value.clone(), listed: entities.len(), entities })
    }
    fn finish(&self) {
        if self.entities.len() != self.listed {
            let kept: Vec<u64> = self.entities.iter().map(Entity::get_entity_id).collect();
            query(json!({ "op": "ev_world_bell", "raiders": kept }));
        }
    }
}

/// bukkit: io/papermc/paper/event/block/BellRevealRaiderEvent
#[derive(Clone, Debug)]
pub struct BellRevealRaiderEvent {
    value: Value,
    cancelled: bool,
}

hooked!(BellRevealRaiderEvent, ["ew_bell_reveal"]);

impl BellRevealRaiderEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/BellRevealRaiderEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Raider> {
        Some(crate::entities::Raider(entity_at(&self.value, "entity")?))
    }
    /// bukkit: io/papermc/paper/event/block/BellRevealRaiderEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/block/BellRevealRaiderEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

// ---- Questions a connection or a worker thread waits on: answered in a tick ----

/// What is waiting of a kind: `(id, what it says)`. Asking also keeps such questions coming.
fn asks(kind: &str) -> Vec<(Value, Value)> {
    let waiting = query(json!({ "op": "ev_world_asks", "kind": kind }));
    waiting.as_array().into_iter().flatten().filter_map(|ask| Some((ask["id"].clone(), serde_json::from_str(ask["json"].as_str()?).ok()?))).collect()
}

fn ask_set(id: &Value, answer: &Value) {
    query(json!({ "op": "ev_world_ask_set", "id": id, "json": answer.to_string() }));
}

/// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent
#[derive(Clone, Debug)]
pub struct ProfileWhitelistVerifyEvent {
    id: Value,
    value: Value,
    whitelisted: bool,
    kick: Option<String>,
}

impl ProfileWhitelistVerifyEvent {
    /// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent.getPlayerProfile
    pub fn get_player_profile(&self) -> crate::player::PlayerProfile {
        crate::player::PlayerProfile { name: text(&self.value, "name").to_owned(), id: text(&self.value, "uuid").to_owned(), properties: Vec::new() }
    }
    /// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent.isWhitelisted
    pub fn is_whitelisted(&self) -> bool {
        self.whitelisted
    }
    /// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent.setWhitelisted
    pub fn set_whitelisted(&mut self, whitelisted: bool) {
        self.whitelisted = whitelisted;
    }
    /// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent.isWhitelistEnabled
    pub fn is_whitelist_enabled(&self) -> bool {
        self.value["enabled"].as_bool().unwrap_or(false)
    }
    /// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent.isOp
    pub fn is_op(&self) -> bool {
        self.value["op"].as_bool().unwrap_or(false)
    }
    /// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent.getKickMessage
    pub fn get_kick_message(&self) -> Option<&str> {
        self.kick.as_deref()
    }
    /// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent.kickMessage
    pub fn kick_message(&self) -> Option<Component> {
        self.kick.as_deref().map(Component::text)
    }
    /// bukkit: com/destroystokyo/paper/event/profile/ProfileWhitelistVerifyEvent.setKickMessage
    pub fn set_kick_message(&mut self, message: &str) {
        self.kick = Some(message.to_owned());
    }
}

impl Event for ProfileWhitelistVerifyEvent {
    const HOOK: &'static str = "tick";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        if !value.is_null() {
            return Vec::new();
        }
        asks("whitelist").into_iter().map(|(id, value)| Self { whitelisted: value["whitelisted"].as_bool().unwrap_or(false), kick: value["kick"].as_str().map(str::to_owned), id, value }).collect()
    }
    fn finish(&self) {
        let mut answer = self.value.clone();
        answer["whitelisted"] = json!(self.whitelisted);
        answer["kick"] = json!(self.kick);
        ask_set(&self.id, &answer);
    }
}

/// `GS4QueryEvent.QueryResponse.PluginInformation`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginInformation {
    name: String,
    version: String,
}

impl PluginInformation {
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$PluginInformation.of
    pub fn of(name: &str, version: &str) -> Self {
        Self { name: name.to_owned(), version: version.to_owned() }
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$PluginInformation.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$PluginInformation.setName
    pub fn set_name(&mut self, name: &str) {
        name.clone_into(&mut self.name);
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$PluginInformation.getVersion
    pub fn get_version(&self) -> &str {
        &self.version
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$PluginInformation.setVersion
    pub fn set_version(&mut self, version: &str) {
        version.clone_into(&mut self.version);
    }
}

/// `GS4QueryEvent.QueryResponse`: what a query answer says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueryResponse {
    motd: String,
    server_version: String,
    plugins: Vec<PluginInformation>,
    map: String,
    current_players: i32,
    max_players: i32,
    players: Vec<String>,
    hostname: String,
    game_version: String,
    port: i32,
}

/// `GS4QueryEvent.QueryResponse.Builder`.
#[derive(Clone, Debug, Default)]
pub struct QueryResponseBuilder(QueryResponse);

impl QueryResponse {
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getMotd
    pub fn get_motd(&self) -> String {
        self.motd.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getServerVersion
    pub fn get_server_version(&self) -> String {
        self.server_version.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getPlugins
    pub fn get_plugins(&self) -> Vec<PluginInformation> {
        self.plugins.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getMap
    pub fn get_map(&self) -> String {
        self.map.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getCurrentPlayers
    pub fn get_current_players(&self) -> i32 {
        self.current_players.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getMaxPlayers
    pub fn get_max_players(&self) -> i32 {
        self.max_players.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getPlayers
    pub fn get_players(&self) -> Vec<String> {
        self.players.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getHostname
    pub fn get_hostname(&self) -> String {
        self.hostname.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getGameVersion
    pub fn get_game_version(&self) -> String {
        self.game_version.clone()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.getPort
    pub fn get_port(&self) -> i32 {
        self.port.clone()
    }
}

impl QueryResponse {
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.builder
    pub fn builder() -> QueryResponseBuilder {
        QueryResponseBuilder::default()
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse.toBuilder
    pub fn to_builder(&self) -> QueryResponseBuilder {
        QueryResponseBuilder(self.clone())
    }
    fn of(value: &Value) -> Self {
        let number = |key: &str| value[key].as_i64().unwrap_or(0) as i32;
        let plugins = value["plugins"].as_array().into_iter().flatten().map(|row| PluginInformation::of(text(row, "name"), text(row, "version"))).collect();
        let players = value["players"].as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect();
        Self {
            motd: text(value, "motd").to_owned(),
            server_version: text(value, "server_version").to_owned(),
            plugins,
            map: text(value, "map").to_owned(),
            current_players: number("current_players"),
            max_players: number("max_players"),
            players,
            hostname: text(value, "hostname").to_owned(),
            game_version: text(value, "game_version").to_owned(),
            port: number("port"),
        }
    }
    fn to_value(&self) -> Value {
        let plugins: Vec<Value> = self.plugins.iter().map(|row| json!({ "name": row.name, "version": row.version })).collect();
        json!({ "motd": self.motd, "server_version": self.server_version, "plugins": plugins, "map": self.map, "current_players": self.current_players, "max_players": self.max_players, "players": self.players, "hostname": self.hostname, "game_version": self.game_version, "port": self.port })
    }
}

impl QueryResponseBuilder {
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.motd
    pub fn motd(mut self, value: &str) -> Self {
        self.0.motd = value.into();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.serverVersion
    pub fn server_version(mut self, value: &str) -> Self {
        self.0.server_version = value.into();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.map
    pub fn map(mut self, value: &str) -> Self {
        self.0.map = value.into();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.currentPlayers
    pub fn current_players(mut self, value: i32) -> Self {
        self.0.current_players = value.into();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.maxPlayers
    pub fn max_players(mut self, value: i32) -> Self {
        self.0.max_players = value.into();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.hostname
    pub fn hostname(mut self, value: &str) -> Self {
        self.0.hostname = value.into();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.gameVersion
    pub fn game_version(mut self, value: &str) -> Self {
        self.0.game_version = value.into();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.port
    pub fn port(mut self, value: i32) -> Self {
        self.0.port = value.into();
        self
    }
}

impl QueryResponseBuilder {
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.plugins
    pub fn plugins(mut self, plugins: Vec<PluginInformation>) -> Self {
        self.0.plugins.extend(plugins);
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.players
    pub fn players(mut self, players: Vec<String>) -> Self {
        self.0.players.extend(players);
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.clearPlugins
    pub fn clear_plugins(mut self) -> Self {
        self.0.plugins.clear();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.clearPlayers
    pub fn clear_players(mut self) -> Self {
        self.0.players.clear();
        self
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryResponse$Builder.build
    pub fn build(self) -> QueryResponse {
        self.0
    }
}

/// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent
///
/// On the query thread, with no game to ask.
#[derive(Clone, Debug)]
pub struct GS4QueryEvent {
    value: Value,
    response: QueryResponse,
}

impl GS4QueryEvent {
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent.getQueryType
    pub fn get_query_type(&self) -> &'static str {
        if self.value["full"].as_bool().unwrap_or(false) { "FULL" } else { "BASIC" }
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent.getQuerierAddress
    pub fn get_querier_address(&self) -> &str {
        text(&self.value, "address")
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent.getResponse
    pub fn get_response(&self) -> &QueryResponse {
        &self.response
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent.setResponse
    pub fn set_response(&mut self, response: QueryResponse) {
        self.response = response;
    }
}

impl Event for GS4QueryEvent {
    const HOOK: &'static str = "ew_query";
    fn read(value: &Value) -> Option<Self> {
        // What the listener before this one left.
        let now = query(json!({ "op": "ev_world_free_query_now" }));
        let value = if now.is_null() { value.clone() } else { now };
        Some(Self { response: QueryResponse::of(&value), value })
    }
    fn finish(&self) {
        let mut answer = self.response.to_value();
        answer["address"] = self.value["address"].clone();
        answer["full"] = self.value["full"].clone();
        query(json!({ "op": "ev_world_free_query_set", "answer": answer }));
    }
}

/// bukkit: org/bukkit/event/server/RemoteServerCommandEvent
#[derive(Clone, Debug)]
pub struct RemoteServerCommandEvent {
    command: String,
    changed: bool,
    cancelled: bool,
}

impl RemoteServerCommandEvent {
    pub fn get_sender(&self) -> CommandSender {
        crate::fam_core::remote_sender().unwrap_or(CommandSender::Console)
    }
    pub fn get_command(&self) -> &str {
        &self.command
    }
    pub fn set_command(&mut self, command: &str) {
        command.clone_into(&mut self.command);
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for RemoteServerCommandEvent {
    const HOOK: &'static str = "ew_console";
    fn read(value: &Value) -> Option<Self> {
        value["remote"].as_bool().unwrap_or(false).then(|| Self { command: text(value, "line").to_owned(), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.changed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.changed && !self.cancelled {
            query(json!({ "op": "ev_world_console_line", "line": self.command }));
        }
    }
}

// ---- Structures, as a world generation thread places them ----

/// bukkit: org/bukkit/event/world/AsyncStructureSpawnEvent
#[derive(Clone, Debug)]
pub struct AsyncStructureSpawnEvent {
    value: Value,
    cancelled: bool,
}

fn structure_box(value: &Value) -> crate::util::BoundingBox {
    let at = |key: &str, i: usize| value[key][i].as_f64().unwrap_or(0.0);
    crate::util::BoundingBox::new(at("min", 0), at("min", 1), at("min", 2), at("max", 0), at("max", 1), at("max", 2))
}

impl AsyncStructureSpawnEvent {
    /// bukkit: org/bukkit/event/world/AsyncStructureSpawnEvent.getStructure
    ///
    /// The structure's key.
    pub fn get_structure(&self) -> &str {
        text(&self.value, "structure")
    }
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureSpawnEvent.getBoundingBox
    pub fn get_bounding_box(&self) -> crate::util::BoundingBox {
        structure_box(&self.value)
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureSpawnEvent.getChunkX
    pub fn get_chunk_x(&self) -> i64 {
        self.value["chunk"][0].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureSpawnEvent.getChunkZ
    pub fn get_chunk_z(&self) -> i64 {
        self.value["chunk"][1].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureSpawnEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureSpawnEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for AsyncStructureSpawnEvent {
    const HOOK: &'static str = "tick";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_structure_spawn"]
    }
    fn read(value: &Value) -> Option<Self> {
        if value.is_null() {
            query(json!({ "op": "ev_world_install" }));
            return None;
        }
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn is_asynchronous(&self) -> bool {
        true
    }
}

/// A block transformer, as held for a structure.
pub type HeldBlockTransformer = std::sync::Arc<dyn crate::fam_core::BlockTransformer + Send + Sync>;
/// An entity transformer, as held for a structure.
pub type HeldEntityTransformer = std::sync::Arc<dyn crate::fam_core::EntityTransformer + Send + Sync>;
type HeldTransformers = (Vec<(crate::persistence::NamespacedKey, HeldBlockTransformer)>, Vec<(crate::persistence::NamespacedKey, HeldEntityTransformer)>);

/// This plugin's transformers on each structure being placed, until it's placed.
static HELD_TRANSFORMERS: Mutex<std::collections::BTreeMap<u64, HeldTransformers>> = Mutex::new(std::collections::BTreeMap::new());

/// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent
///
/// Transformers run once the structure's blocks for the chunk are in, each block and entity in turn, on a
/// thread with no game: a `LimitedRegion` there reads and writes blocks only.
#[derive(Clone)]
pub struct AsyncStructureGenerateEvent {
    value: Value,
    blocks: Vec<(crate::persistence::NamespacedKey, HeldBlockTransformer)>,
    entities: Vec<(crate::persistence::NamespacedKey, HeldEntityTransformer)>,
    /// The keys of every plugin's, in order: a key here that isn't above is another plugin's.
    block_keys: Vec<String>,
    entity_keys: Vec<String>,
    changed: bool,
    cancelled: bool,
}

impl AsyncStructureGenerateEvent {
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.getStructure
    ///
    /// The structure's key.
    pub fn get_structure(&self) -> &str {
        text(&self.value, "structure")
    }
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        world_of(&self.value)
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.getCause
    ///
    /// `COMMAND` or `WORLD_GENERATION`.
    pub fn get_cause(&self) -> &str {
        text(&self.value, "cause")
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.getBoundingBox
    pub fn get_bounding_box(&self) -> crate::util::BoundingBox {
        structure_box(&self.value)
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.getChunkX
    pub fn get_chunk_x(&self) -> i64 {
        self.value["chunk"][0].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.getChunkZ
    pub fn get_chunk_z(&self) -> i64 {
        self.value["chunk"][1].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    /// bukkit~: org/bukkit/event/world/AsyncStructureGenerateEvent.getBlockTransformers | this plugin's: another's can't cross between libraries, though its key is honoured
    pub fn get_block_transformers(&self) -> Vec<(crate::persistence::NamespacedKey, HeldBlockTransformer)> {
        self.blocks.clone()
    }
    /// bukkit~: org/bukkit/event/world/AsyncStructureGenerateEvent.getEntityTransformers | this plugin's: another's can't cross between libraries, though its key is honoured
    pub fn get_entity_transformers(&self) -> Vec<(crate::persistence::NamespacedKey, HeldEntityTransformer)> {
        self.entities.clone()
    }
    /// bukkit~: org/bukkit/event/world/AsyncStructureGenerateEvent.getBlockTransformer | this plugin's: another's can't cross between libraries
    pub fn get_block_transformer(&self, key: &crate::persistence::NamespacedKey) -> Option<HeldBlockTransformer> {
        self.blocks.iter().find(|(held, _)| held == key).map(|(_, transformer)| transformer.clone())
    }
    /// bukkit~: org/bukkit/event/world/AsyncStructureGenerateEvent.getEntityTransformer | this plugin's: another's can't cross between libraries
    pub fn get_entity_transformer(&self, key: &crate::persistence::NamespacedKey) -> Option<HeldEntityTransformer> {
        self.entities.iter().find(|(held, _)| held == key).map(|(_, transformer)| transformer.clone())
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.setBlockTransformer
    pub fn set_block_transformer(&mut self, key: &crate::persistence::NamespacedKey, transformer: impl crate::fam_core::BlockTransformer + Send + Sync + 'static) {
        let transformer: HeldBlockTransformer = std::sync::Arc::new(transformer);
        match self.blocks.iter_mut().find(|(held, _)| held == key) {
            Some(held) => held.1 = transformer,
            None => self.blocks.push((key.clone(), transformer)),
        }
        if !self.block_keys.iter().any(|held| held == key.as_str()) {
            self.block_keys.push(key.as_str().to_owned());
        }
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.setEntityTransformer
    pub fn set_entity_transformer(&mut self, key: &crate::persistence::NamespacedKey, transformer: impl crate::fam_core::EntityTransformer + Send + Sync + 'static) {
        let transformer: HeldEntityTransformer = std::sync::Arc::new(transformer);
        match self.entities.iter_mut().find(|(held, _)| held == key) {
            Some(held) => held.1 = transformer,
            None => self.entities.push((key.clone(), transformer)),
        }
        if !self.entity_keys.iter().any(|held| held == key.as_str()) {
            self.entity_keys.push(key.as_str().to_owned());
        }
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.removeBlockTransformer
    pub fn remove_block_transformer(&mut self, key: &crate::persistence::NamespacedKey) {
        self.blocks.retain(|(held, _)| held != key);
        self.block_keys.retain(|held| held != key.as_str());
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.removeEntityTransformer
    pub fn remove_entity_transformer(&mut self, key: &crate::persistence::NamespacedKey) {
        self.entities.retain(|(held, _)| held != key);
        self.entity_keys.retain(|held| held != key.as_str());
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.clearBlockTransformers
    pub fn clear_block_transformers(&mut self) {
        self.blocks.clear();
        self.block_keys.clear();
        self.changed = true;
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent.clearEntityTransformers
    pub fn clear_entity_transformers(&mut self) {
        self.entities.clear();
        self.entity_keys.clear();
        self.changed = true;
    }
}

fn texts_of(value: &Value) -> Vec<String> {
    value.as_array().into_iter().flatten().filter_map(|text| text.as_str().map(str::to_owned)).collect()
}

/// This plugin's transformers over what a structure placed, leaving the result for the next plugin.
fn run_transformers(token: u64, cancelled: bool) {
    let Some((blocks, entities)) = HELD_TRANSFORMERS.lock().unwrap_or_else(|e| e.into_inner()).remove(&token) else {
        return;
    };
    if cancelled {
        return;
    }
    let batch = query(json!({ "op": "ev_world_free_structure" }));
    let world = world_of(&batch);
    let (block_keys, entity_keys) = (texts_of(&batch["block_keys"]), texts_of(&batch["entity_keys"]));
    let blocks: Vec<HeldBlockTransformer> = blocks.into_iter().filter(|(key, _)| block_keys.iter().any(|held| held == key.as_str())).map(|(_, transformer)| transformer).collect();
    let entities: Vec<HeldEntityTransformer> = entities.into_iter().filter(|(key, _)| entity_keys.iter().any(|held| held == key.as_str())).map(|(_, transformer)| transformer).collect();
    let region = |x: i32, z: i32| crate::worlds::LimitedRegion::of(&world, x >> 4, z >> 4);
    let mut block_changes = Vec::new();
    if !blocks.is_empty() {
        for (row, place) in batch["blocks"].as_array().into_iter().flatten().enumerate() {
            let at = |side: usize| place[side].as_i64().unwrap_or(0) as i32;
            let (x, y, z) = (at(0), at(1), at(2));
            let original = crate::block_data::BlockData::parse(place[4].as_str().unwrap_or_default());
            let state = crate::fam_core::TransformationState::of(original.clone(), crate::block_data::BlockData::parse(place[3].as_str().unwrap_or_default()));
            let result = blocks.iter().fold(original.clone(), |current, transformer| transformer.transform(&region(x, z), x, y, z, current, &state));
            if result != original {
                block_changes.push(json!([row, result.get_as_string()]));
            }
        }
    }
    let mut entity_changes = Vec::new();
    if !entities.is_empty() {
        for (row, place) in batch["entities"].as_array().into_iter().flatten().enumerate() {
            let at = |side: usize| place["pos"][side].as_i64().unwrap_or(0) as i32;
            let (x, y, z) = (at(0), at(1), at(2));
            let before = place["snbt"].as_str().map_or_else(|| snbt(&place["data"]), str::to_owned);
            let mut snapshot = crate::entities3::create_entity_snapshot(&before);
            let was_kept = place["kept"] == true;
            let kept = entities.iter().fold(was_kept, |kept, transformer| transformer.transform(&region(x, z), x, y, z, &mut snapshot, kept));
            let changed = snapshot.get_as_string() != before;
            if kept != was_kept || changed {
                entity_changes.push(json!([row, kept, changed.then(|| snapshot.get_as_string().to_owned()).or_else(|| place["snbt"].as_str().map(str::to_owned))]));
            }
        }
    }
    if !block_changes.is_empty() || !entity_changes.is_empty() {
        query(json!({ "op": "ev_world_free_structure_set", "blocks": block_changes, "entities": entity_changes }));
    }
}

impl Event for AsyncStructureGenerateEvent {
    const HOOK: &'static str = "tick";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_structure_generate", "ew_structure_blocks"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        let token = value.get("placed")?.as_u64()?;
        run_transformers(token, value["cancelled"] == true);
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        if value.is_null() {
            query(json!({ "op": "ev_world_install" }));
            return None;
        }
        let token = value.get("token")?.as_u64()?;
        let (blocks, entities) = HELD_TRANSFORMERS.lock().unwrap_or_else(|e| e.into_inner()).get(&token).cloned().unwrap_or_default();
        let keys = query(json!({ "op": "ev_world_free_transformer_keys" }));
        Some(Self { value: value.clone(), blocks, entities, block_keys: texts_of(&keys["blocks"]), entity_keys: texts_of(&keys["entities"]), changed: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.changed {
            return;
        }
        if let Some(token) = self.value["token"].as_u64() {
            HELD_TRANSFORMERS.lock().unwrap_or_else(|e| e.into_inner()).insert(token, (self.blocks.clone(), self.entities.clone()));
        }
        query(json!({ "op": "ev_world_free_transformers", "blocks": self.block_keys, "entities": self.entity_keys }));
    }
    fn is_asynchronous(&self) -> bool {
        self.value["cause"] != "COMMAND"
    }
}

// ---- What the level does to blocks, with no game to lend ----

/// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent
#[derive(Clone, Debug)]
pub struct BlockDestroyEvent {
    value: Value,
    drop: bool,
    effect: bool,
    effect_block: Option<String>,
    cancelled: bool,
}

impl BlockDestroyEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.getNewState
    ///
    /// The state the block leaves behind.
    pub fn get_new_state(&self) -> &str {
        text(&self.value, "new")
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.willDrop
    pub fn will_drop(&self) -> bool {
        self.drop
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.setWillDrop
    pub fn set_will_drop(&mut self, drop: bool) {
        self.drop = drop;
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.playEffect
    pub fn play_effect(&self) -> bool {
        self.effect
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.setPlayEffect
    pub fn set_play_effect(&mut self, effect: bool) {
        self.effect = effect;
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.getEffectBlock
    pub fn get_effect_block(&self) -> &str {
        self.effect_block.as_deref().unwrap_or_else(|| text(&self.value, "state"))
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.setEffectBlock
    pub fn set_effect_block(&mut self, data: &str) {
        self.effect_block = Some(data.to_owned());
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/block/BlockDestroyEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockDestroyEvent {
    const HOOK: &'static str = "ew_destroy";
    fn more_hooks() -> &'static [&'static str] {
        &["tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if !value.is_null() {
            return None;
        }
        query(json!({ "op": "ev_world_install" }));
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), drop: value["drop"].as_bool().unwrap_or(true), effect: true, effect_block: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled {
            Answer::Fail
        } else {
            Answer::Amount(i32::from(!self.drop) + 2 * i32::from(!self.effect) + 4 * i32::from(self.effect_block.is_some()))
        }
    }
    fn finish(&self) {
        if let Some(data) = &self.effect_block {
            query(json!({ "op": "ev_world_free_effect", "data": data }));
        }
    }
}

/// bukkit: io/papermc/paper/event/block/BlockBreakBlockEvent
#[derive(Clone, Debug)]
pub struct BlockBreakBlockEvent {
    value: Value,
    drops: Vec<ItemStack>,
    listed: usize,
}

impl BlockBreakBlockEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/block/BlockBreakBlockEvent.getSource
    pub fn get_source(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["source"])
    }
    /// bukkit: io/papermc/paper/event/block/BlockBreakBlockEvent.getDrops
    pub fn get_drops(&mut self) -> &mut Vec<ItemStack> {
        &mut self.drops
    }
}

impl Event for BlockBreakBlockEvent {
    const HOOK: &'static str = "ew_block_broke";
    fn more_hooks() -> &'static [&'static str] {
        &["tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if !value.is_null() {
            return None;
        }
        query(json!({ "op": "ev_world_install" }));
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        let drops: Vec<ItemStack> = value["items"].as_array()?.iter().filter_map(ItemStack::read).collect();
        Some(Self { value: value.clone(), listed: drops.len(), drops })
    }
    fn answer(&self) -> Answer {
        if self.drops.len() != self.listed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.drops.len() != self.listed {
            let kept: Vec<Value> = self.drops.iter().map(ItemStack::to_json).collect();
            query(json!({ "op": "ev_world_free_drops", "items": kept }));
        }
    }
}

/// bukkit: org/bukkit/event/block/SpongeAbsorbEvent
#[derive(Clone, Debug)]
pub struct SpongeAbsorbEvent {
    value: Value,
    blocks: Vec<BlockState>,
    listed: usize,
    cancelled: bool,
}

impl SpongeAbsorbEvent {
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/SpongeAbsorbEvent.getBlocks
    pub fn get_blocks(&mut self) -> &mut Vec<BlockState> {
        &mut self.blocks
    }
    /// bukkit: org/bukkit/event/block/SpongeAbsorbEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/SpongeAbsorbEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for SpongeAbsorbEvent {
    const HOOK: &'static str = "ew_sponge";
    fn more_hooks() -> &'static [&'static str] {
        &["tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if !value.is_null() {
            return None;
        }
        query(json!({ "op": "ev_world_install" }));
        Some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        let world = world_of(value);
        let states = value["states"].as_array()?;
        let blocks: Vec<BlockState> = value["blocks"].as_array()?.iter().zip(states).filter_map(|(pos, state)| Some(BlockState::snapshot(block_at(&world, pos)?, state.as_str()?))).collect();
        Some(Self { value: value.clone(), listed: blocks.len(), blocks, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled || self.blocks.len() != self.listed { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.cancelled && self.blocks.len() != self.listed {
            let kept: Vec<[i32; 3]> = self.blocks.iter().map(|state| [state.get_block().get_x(), state.get_block().get_y(), state.get_block().get_z()]).collect();
            query(json!({ "op": "ev_world_free_sponge", "blocks": kept }));
        }
    }
}

// ---- A vehicle running into a block, and services ----

/// bukkit: org/bukkit/event/vehicle/VehicleBlockCollisionEvent
#[derive(Clone, Debug)]
pub struct VehicleBlockCollisionEvent {
    value: Value,
}

vehicle!(VehicleBlockCollisionEvent);

impl VehicleBlockCollisionEvent {
    /// bukkit: org/bukkit/event/vehicle/VehicleBlockCollisionEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/vehicle/VehicleBlockCollisionEvent.getVelocity
    pub fn get_velocity(&self) -> crate::util::Vector {
        let at = |i: usize| self.value["velocity"][i].as_f64().unwrap_or(0.0);
        crate::util::Vector::new(at(0), at(1), at(2))
    }
}

impl Event for VehicleBlockCollisionEvent {
    const HOOK: &'static str = "tick";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        if !value.is_null() {
            return Vec::new();
        }
        query(json!({ "op": "ev_world_vehicle_hits" })).as_array().into_iter().flatten().map(|hit| Self { value: hit.clone() }).collect()
    }
}

/// `org.bukkit.event.server.ServiceEvent`.
pub trait ServiceEvent {
    /// bukkit: org/bukkit/event/server/ServiceEvent.getProvider
    fn get_provider(&self) -> &crate::persistence::ServiceProvider;
}

/// bukkit: org/bukkit/event/server/ServiceRegisterEvent
///
/// For every plugin, as a service comes. One that came while a plugin started is told at the first tick,
/// when the game is there to tell.
#[derive(Clone, Debug)]
pub struct ServiceRegisterEvent {
    provider: crate::persistence::ServiceProvider,
}

impl ServiceEvent for ServiceRegisterEvent {
    fn get_provider(&self) -> &crate::persistence::ServiceProvider {
        &self.provider
    }
}

impl Event for ServiceRegisterEvent {
    const HOOK: &'static str = "service_register";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { provider: crate::persistence::ServiceProvider::read(value)? })
    }
}

/// bukkit: org/bukkit/event/server/ServiceUnregisterEvent
#[derive(Clone, Debug)]
pub struct ServiceUnregisterEvent {
    provider: crate::persistence::ServiceProvider,
}

impl ServiceEvent for ServiceUnregisterEvent {
    fn get_provider(&self) -> &crate::persistence::ServiceProvider {
        &self.provider
    }
}

impl Event for ServiceUnregisterEvent {
    const HOOK: &'static str = "service_unregister";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { provider: crate::persistence::ServiceProvider::read(value)? })
    }
}

// ---- Work the game does elsewhere or not at all: these fire when it does ----

/// bukkit: io/papermc/paper/event/server/AsyncServerDataFixerRemoveBlockEntityEvent
///
/// As a bed's old block entity goes, on the thread loading the chunk.
#[derive(Clone, Debug)]
pub struct AsyncServerDataFixerRemoveBlockEntityEvent {
    value: Value,
}

impl AsyncServerDataFixerRemoveBlockEntityEvent {
    /// bukkit: io/papermc/paper/event/server/AsyncServerDataFixerRemoveBlockEntityEvent.getWorldKey
    pub fn get_world_key(&self) -> &str {
        text(&self.value, "level")
    }
    /// bukkit: io/papermc/paper/event/server/AsyncServerDataFixerRemoveBlockEntityEvent.getBlockEntityType
    pub fn get_block_entity_type(&self) -> &str {
        text(&self.value, "kind")
    }
    /// bukkit: io/papermc/paper/event/server/AsyncServerDataFixerRemoveBlockEntityEvent.getBlockPosition
    pub fn get_block_position(&self) -> Option<Location> {
        let at = |i: usize| self.value["pos"][i].as_f64();
        Some(Location::new(world_of(&self.value), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: io/papermc/paper/event/server/AsyncServerDataFixerRemoveBlockEntityEvent.getPersistentDataContainerView
    pub fn get_persistent_data_container_view(&self) -> &Value {
        &self.value["data"]
    }
}

impl Event for AsyncServerDataFixerRemoveBlockEntityEvent {
    const HOOK: &'static str = "ew_data_fixer_removed";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

fn profile_rows(value: &Value) -> Vec<crate::misc::ProfileProperty> {
    value["properties"].as_array().into_iter().flatten().filter_map(|row| Some(crate::misc::ProfileProperty::new(row["name"].as_str()?, row["value"].as_str()?, row["signature"].as_str()))).collect()
}

fn profile_of(value: &Value) -> crate::player::PlayerProfile {
    let properties = profile_rows(value).into_iter().map(|row| (row.name, row.value, row.signature)).collect();
    crate::player::PlayerProfile { name: text(value, "name").to_owned(), id: text(value, "id").to_owned(), properties }
}

fn profile_said(said: Value) {
    query(json!({ "op": "ev_world_free_profile_set", "said": said }));
}

fn property_rows(properties: &[crate::misc::ProfileProperty]) -> Vec<Value> {
    properties.iter().map(|row| json!({ "name": row.name, "value": row.value, "signature": row.signature })).collect()
}

/// bukkit: com/destroystokyo/paper/event/profile/FillProfileEvent
#[derive(Clone, Debug)]
pub struct FillProfileEvent {
    value: Value,
    properties: Vec<crate::misc::ProfileProperty>,
}

impl FillProfileEvent {
    /// bukkit: com/destroystokyo/paper/event/profile/FillProfileEvent.getPlayerProfile
    pub fn get_player_profile(&self) -> crate::player::PlayerProfile {
        profile_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/profile/FillProfileEvent.getProperties
    pub fn get_properties(&self) -> &[crate::misc::ProfileProperty] {
        &self.properties
    }
}

impl Event for FillProfileEvent {
    const HOOK: &'static str = "ew_profile_fill";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { properties: profile_rows(value), value: value.clone() })
    }
}

/// bukkit: com/destroystokyo/paper/event/profile/PreFillProfileEvent
#[derive(Clone, Debug)]
pub struct PreFillProfileEvent {
    value: Value,
    properties: Vec<crate::misc::ProfileProperty>,
}

impl PreFillProfileEvent {
    /// bukkit: com/destroystokyo/paper/event/profile/PreFillProfileEvent.getPlayerProfile
    pub fn get_player_profile(&self) -> crate::player::PlayerProfile {
        profile_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/profile/PreFillProfileEvent.setProperties
    ///
    /// With `textures` among them, the session servers aren't asked.
    pub fn set_properties(&mut self, properties: Vec<crate::misc::ProfileProperty>) {
        self.properties = properties;
    }
}

impl Event for PreFillProfileEvent {
    const HOOK: &'static str = "ew_profile_prefill";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { properties: profile_rows(value), value: value.clone() })
    }
    fn finish(&self) {
        if !self.properties.is_empty() {
            profile_said(json!({ "properties": property_rows(&self.properties) }));
        }
    }
}

/// bukkit: com/destroystokyo/paper/event/profile/LookupProfileEvent
#[derive(Clone, Debug)]
pub struct LookupProfileEvent {
    value: Value,
}

impl LookupProfileEvent {
    /// bukkit: com/destroystokyo/paper/event/profile/LookupProfileEvent.getPlayerProfile
    pub fn get_player_profile(&self) -> crate::player::PlayerProfile {
        profile_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/profile/LookupProfileEvent.getName
    pub fn get_name(&self) -> &str {
        text(&self.value, "name")
    }
    /// bukkit: com/destroystokyo/paper/event/profile/LookupProfileEvent.getId
    pub fn get_id(&self) -> &str {
        text(&self.value, "id")
    }
}

impl Event for LookupProfileEvent {
    const HOOK: &'static str = "ew_profile_lookup";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone() })
    }
}

/// bukkit: com/destroystokyo/paper/event/profile/PreLookupProfileEvent
#[derive(Clone, Debug)]
pub struct PreLookupProfileEvent {
    value: Value,
    uuid: Option<String>,
    properties: Vec<crate::misc::ProfileProperty>,
}

impl PreLookupProfileEvent {
    pub fn get_player_profile(&self) -> crate::player::PlayerProfile {
        profile_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/profile/PreLookupProfileEvent.getName
    pub fn get_name(&self) -> &str {
        text(&self.value, "name")
    }
    /// bukkit: com/destroystokyo/paper/event/profile/PreLookupProfileEvent.getUUID
    pub fn get_uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }
    /// bukkit: com/destroystokyo/paper/event/profile/PreLookupProfileEvent.setUUID
    pub fn set_uuid(&mut self, id: &str) {
        self.uuid = Some(id.to_owned());
    }
    /// bukkit: com/destroystokyo/paper/event/profile/PreLookupProfileEvent.getProfileProperties
    pub fn get_profile_properties(&self) -> &[crate::misc::ProfileProperty] {
        &self.properties
    }
    /// bukkit: com/destroystokyo/paper/event/profile/PreLookupProfileEvent.setProfileProperties
    pub fn set_profile_properties(&mut self, properties: Vec<crate::misc::ProfileProperty>) {
        self.properties = properties;
    }
    /// bukkit: com/destroystokyo/paper/event/profile/PreLookupProfileEvent.addProfileProperties
    pub fn add_profile_properties(&mut self, properties: Vec<crate::misc::ProfileProperty>) {
        self.properties.extend(properties);
    }
}

impl Event for PreLookupProfileEvent {
    const HOOK: &'static str = "ew_profile_prelookup";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), uuid: value["uuid"].as_str().map(str::to_owned), properties: profile_rows(value) })
    }
    fn finish(&self) {
        if self.uuid.is_some() || !self.properties.is_empty() {
            profile_said(json!({ "uuid": self.uuid, "properties": property_rows(&self.properties) }));
            crate::event::carry("uuid", json!(self.uuid));
            crate::event::carry("properties", json!(property_rows(&self.properties)));
        }
    }
}

// ---- Commands, as they're registered and sent ----

/// This plugin's Brigadier commands since the last tick.
static OWN_COMMANDS: Mutex<Vec<(String, std::sync::Arc<crate::brigadier::CommandNode>)>> = Mutex::new(Vec::new());

/// Called as a command goes in the plugin's tree. True if it's there already: a reload's handler again.
pub(crate) fn note_command(plugin: &crate::Plugin, name: &str, node: &std::sync::Arc<crate::brigadier::CommandNode>) -> bool {
    let mut own = OWN_COMMANDS.lock().unwrap_or_else(|e| e.into_inner());
    if own.is_empty() {
        let me = plugin.get_name().to_owned();
        plugin.host.on("ew_command_taken", move |value| {
            // A listener in another plugin made a raw command of it: it runs there now.
            if text(value, "by") != me {
                let label = text(value, "label");
                OWN_COMMANDS.lock().unwrap_or_else(|e| e.into_inner()).retain(|(name, _)| name != label);
                crate::commands().lock().unwrap_or_else(|e| e.into_inner()).remove(label);
            }
            Answer::Pass
        });
    }
    let known = own.iter().position(|(known, _)| known == name);
    match known {
        Some(row) => own[row].1 = node.clone(),
        None => own.push((name.to_owned(), node.clone())),
    }
    known.is_some()
}

/// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent
///
/// For each plugin command as the tree for players is built.
#[derive(Clone)]
pub struct CommandRegisteredEvent {
    label: String,
    labels: Vec<String>,
    literal: std::sync::Arc<crate::brigadier::CommandNode>,
    changed: bool,
    raw: bool,
    cancelled: bool,
}

impl CommandRegisteredEvent {
    /// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.getCommandLabel
    pub fn get_command_label(&self) -> &str {
        &self.label
    }
    /// bukkit~: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.getCommand | the command's word: a Rust command isn't an object another plugin could be handed
    pub fn get_command(&self) -> &str {
        &self.label
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.getLiteral
    pub fn get_literal(&self) -> &crate::brigadier::CommandNode {
        &self.literal
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.setLiteral
    ///
    /// The literal players' clients get. Unless the command is raw, what they type still runs the plugin's own.
    pub fn set_literal(&mut self, literal: crate::brigadier::CommandNode) {
        self.literal = std::sync::Arc::new(literal);
        self.changed = true;
    }
    /// bukkit~: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.getBrigadierCommand | the literal, which runs the command here
    pub fn get_brigadier_command(&self) -> &crate::brigadier::CommandNode {
        &self.literal
    }
    /// bukkit~: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.getRoot | the words plugins have in the tree, as a plugin's tree can't be handed to another
    pub fn get_root(&self) -> &[String] {
        &self.labels
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.getDefaultArgs
    ///
    /// The `args` a word with no tree of its own is given.
    pub fn get_default_args(&self) -> crate::brigadier::CommandNode {
        crate::brigadier::CommandNode::argument("args", crate::brigadier::ArgumentType::Greedy)
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.isRawCommand
    pub fn is_raw_command(&self) -> bool {
        self.raw
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.setRawCommand
    ///
    /// Raw, a command runs by the literal alone.
    pub fn set_raw_command(&mut self, raw: bool) {
        self.raw = raw;
        self.changed = true;
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/CommandRegisteredEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for CommandRegisteredEvent {
    const HOOK: &'static str = "ew_command_registered";
    fn read(value: &Value) -> Option<Self> {
        let label = value["label"].as_str()?.to_owned();
        let own = OWN_COMMANDS.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|(name, _)| *name == label).map(|(_, node)| node.clone());
        let literal = own.unwrap_or_else(|| {
            std::sync::Arc::new(crate::brigadier::CommandNode::literal(&label).then(crate::brigadier::CommandNode::argument("args", crate::brigadier::ArgumentType::Greedy)))
        });
        let labels = value["labels"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect();
        Some(Self { label, labels, literal, changed: false, raw: value["raw"] == true, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
    fn finish(&self) {
        if !self.changed || self.cancelled {
            return;
        }
        crate::event::carry("raw", json!(self.raw));
        crate::event::carry("usages", json!(crate::fam_paper::usages_of(&self.literal)));
        let Some(plugin) = crate::fam_core::own_plugin() else {
            return;
        };
        let literal = self.literal.clone();
        plugin.set_tab_completer(&self.label, move |sender, _, args| literal.complete(sender, args));
        if self.raw {
            crate::brigadier::run_by(&plugin, &self.label, self.literal.clone());
            crate::event::carry("taken", json!(plugin.get_name()));
        }
    }
}

/// bukkit~: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendCommandsEvent | once a player's commands have gone out, as the names at the root of the tree: the tree goes out as packet bytes, so a node can't be edited
#[derive(Clone, Debug)]
pub struct AsyncPlayerSendCommandsEvent {
    value: Value,
}

impl AsyncPlayerSendCommandsEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendCommandsEvent.hasFiredAsync
    ///
    /// `false`: this fires on the server thread.
    pub fn has_fired_async(&self) -> bool {
        false
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendCommandsEvent.getCommandNode
    pub fn get_command_node(&self) -> Vec<String> {
        self.value["commands"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect()
    }
}

impl Event for AsyncPlayerSendCommandsEvent {
    const HOOK: &'static str = "pl_commands";
    fn read(value: &Value) -> Option<Self> {
        value["commands"].is_array().then(|| Self { value: value.clone() })
    }
}

/// bukkit: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendSuggestionsEvent
#[derive(Clone, Debug)]
pub struct AsyncPlayerSendSuggestionsEvent {
    value: Value,
    suggestions: Vec<String>,
    before: Vec<String>,
    cancelled: bool,
}

impl AsyncPlayerSendSuggestionsEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendSuggestionsEvent.getBuffer
    pub fn get_buffer(&self) -> &str {
        text(&self.value, "buffer")
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendSuggestionsEvent.getSuggestions
    pub fn get_suggestions(&self) -> &[String] {
        &self.suggestions
    }
    /// bukkit~: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendSuggestionsEvent.setSuggestions | words at the place the game's own start: with none of its own, new ones have no place to go
    pub fn set_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendSuggestionsEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/brigadier/AsyncPlayerSendSuggestionsEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for AsyncPlayerSendSuggestionsEvent {
    const HOOK: &'static str = "ew_suggestions";
    fn read(value: &Value) -> Option<Self> {
        let suggestions: Vec<String> = value["suggestions"].as_array()?.iter().filter_map(Value::as_str).map(str::to_owned).collect();
        Some(Self { value: value.clone(), before: suggestions.clone(), suggestions, cancelled: crate::event::prior_cancelled() })
    }
    fn finish(&self) {
        if self.cancelled || self.suggestions != self.before {
            let left: &[String] = if self.cancelled { &[] } else { &self.suggestions };
            query(json!({ "op": "ev_world_suggestions", "left": left }));
        }
    }
}
