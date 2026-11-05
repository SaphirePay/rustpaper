//! `BlockData` and `TileState`: what the classes in `blocks.rs` are views of, with the base calls.
use crate::blocks::{Banner, Sign};
use crate::meta_lists::Pattern;
use crate::{Block, Location, Material};
use horizon_mod::{Value, query};
use serde_json::json;
use std::collections::BTreeMap;

/// A class of `BlockData`: `data.class::<WaterloggedData>()` is Java's cast.
pub trait DataClass {
    fn of(data: BlockData) -> Self;
    fn data(&self) -> &BlockData;
}

/// `org.bukkit.block.data.BlockData`: a block and its properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockData {
    name: String,
    properties: BTreeMap<String, String>,
}

impl BlockData {
    /// bukkit: org/bukkit/Bukkit.createBlockData, org/bukkit/Server.createBlockData, org/bukkit/Material.createBlockData
    ///
    /// From what the game writes: `minecraft:oak_stairs[facing=east]`, or a bare name.
    pub fn parse(text: &str) -> Self {
        let (name, rest) = text.split_once('[').unwrap_or((text, ""));
        let properties = rest.trim_end_matches(']').split(',').filter_map(|pair| pair.split_once('=')).map(|(key, value)| (key.to_owned(), value.to_owned())).collect();
        Self { name: if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") }, properties }
    }
    /// The same data seen as a class.
    pub fn class<C: DataClass>(&self) -> C {
        C::of(self.clone())
    }
    /// bukkit: org/bukkit/block/data/BlockData.getAsString
    pub fn get_as_string(&self) -> String {
        if self.properties.is_empty() {
            return self.name.clone();
        }
        let pairs: Vec<String> = self.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
        format!("{}[{}]", self.name, pairs.join(","))
    }
    /// bukkit: org/bukkit/block/data/BlockData.getMaterial
    pub fn get_material(&self) -> Material {
        Material::of(&self.name)
    }
    pub(crate) fn property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }
    pub(crate) fn set_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_owned(), value.to_owned());
    }
    /// The highest a number property of this block goes.
    pub(crate) fn maximum_of(&self, key: &str) -> i32 {
        query(json!({ "op": "property_values", "block": self.name, "key": key })).as_array().into_iter().flatten().filter_map(|value| value.as_str()?.parse().ok()).max().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/BlockData.matches
    pub fn matches(&self, other: &Self) -> bool {
        self.name == other.name && other.properties.iter().all(|(key, value)| self.properties.get(key) == Some(value))
    }
    /// bukkit: org/bukkit/block/data/BlockData.merge
    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = self.clone();
        merged.properties.extend(other.properties.clone());
        merged
    }
    /// bukkit: org/bukkit/block/data/BlockData.copyTo
    pub fn copy_to(&self, other: &mut Self) {
        for (key, value) in &self.properties {
            if other.properties.contains_key(key) {
                other.properties.insert(key.clone(), value.clone());
            }
        }
    }
    /// bukkit: org/bukkit/block/data/BlockData.clone
    pub fn copy(&self) -> Self {
        self.clone()
    }
}

impl Block {
    /// A typed view of the block's data.
    pub fn block_data(&self) -> BlockData {
        BlockData::parse(&self.get_block_data())
    }
    /// bukkit: org/bukkit/block/Block.setBlockData
    pub fn set_data(&self, data: &BlockData) {
        self.set_block_data(&data.get_as_string());
    }
    /// bukkit: org/bukkit/block/Block.setBlockData
    ///
    /// `applyPhysics` false is Java's quiet write; the engine's block write always tells the neighbours, as
    /// `BlockState.update(force, physics)` does here (CoreProtect's rollback puts blocks back with false).
    pub fn set_data_physics(&self, data: &BlockData, _apply_physics: bool) {
        self.set_block_data(&data.get_as_string());
    }
    /// bukkit: org/bukkit/block/Block.setType
    ///
    /// `applyPhysics` as `setBlockData`'s: the neighbours are told either way.
    pub fn set_type_physics(&self, material: &Material, _apply_physics: bool) {
        self.set_block_data(material.key());
    }
    /// bukkit: org/bukkit/block/Block.getRelative
    ///
    /// The block next to this one on that face (`BlockFace.NORTH`, `UP`, ...).
    pub fn get_relative_face(&self, face: &str) -> Block {
        self.get_relative_face_by(face, 1)
    }
    /// bukkit: org/bukkit/block/Block.getRelative
    ///
    /// `distance` blocks away on that face.
    pub fn get_relative_face_by(&self, face: &str, distance: i32) -> Block {
        let (x, y, z) = face_offset(face);
        self.get_relative(x * distance, y * distance, z * distance)
    }
    /// A typed view of the block entity here.
    pub fn tile_state<C: StateClass>(&self) -> C {
        C::of(self.tile())
    }
    /// bukkit: org/bukkit/block/Block.getState
    ///
    /// With the block entity's data, for `update` to put back.
    pub fn tile(&self) -> TileState {
        let data = query(json!({ "op": "block_data", "dimension": self.get_world().dimension(), "pos": [self.get_x(), self.get_y(), self.get_z()] }));
        let state = self.get_block_data();
        TileState { block: self.clone(), data, changes: Vec::new(), held: Default::default(), state }
    }
}

impl crate::World {
    /// bukkit: org/bukkit/World.getBlockAt
    ///
    /// The block a location is in (its block coordinates, in this world).
    pub fn get_block_at_location(&self, location: &Location) -> Block {
        self.get_block_at(location.get_block_x(), location.get_block_y(), location.get_block_z())
    }
}

/// A class of block state: `block.tile_state::<Furnace>()` is Java's cast of `getState()`.
pub trait StateClass {
    fn of(state: TileState) -> Self;
}

/// bukkit: org/bukkit/block/TileState
///
/// A block entity as it was when read; `update` writes the changes back.
#[derive(Clone, Debug, PartialEq)]
pub struct TileState {
    pub(crate) block: Block,
    pub(crate) data: Value,
    pub(crate) changes: Vec<(String, String)>,
    /// For a state of no block: what stands in for the world.
    pub(crate) held: crate::fam_items::Held,
    /// The block as it was when the state was taken (`minecraft:oak_stairs[facing=east]`), as Java's snapshot:
    /// read later, even on another thread, it is still what was there (CoreProtect logs a broken block from it).
    /// Empty for a state of no block.
    pub(crate) state: String,
}

impl TileState {
    /// bukkit: org/bukkit/block/TileState.isSnapshot
    pub fn is_snapshot(&self) -> bool {
        true
    }
    pub(crate) fn saved(&self, path: &str) -> Value {
        // A change made since the state was read comes first, as Java's state reads back what was set on it: four
        // `setLine`s in a row each rebuilt the side from the old text and kept only their own line.
        if let Some((_, snbt)) = self.changes.iter().rev().find(|(changed, _)| changed == path) {
            let value = crate::fam_items::snbt_json(snbt);
            if !value.is_null() {
                return value;
            }
        }
        crate::data::at(&self.data, path).clone()
    }
    pub(crate) fn location_of(&self, value: &Value) -> Option<Location> {
        let at = |i: usize| value[i].as_f64();
        Some(Location::new(self.block.get_world().clone(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    pub(crate) fn change(&mut self, path: &str, snbt: String) {
        self.changes.retain(|(known, _)| known != path);
        self.changes.push((path.to_owned(), snbt));
    }
    /// bukkit: org/bukkit/block/BlockState.getBlock
    ///
    /// The block this state is of.
    pub fn get_block(&self) -> &Block {
        &self.block
    }
    /// bukkit: org/bukkit/block/BlockState.getX
    ///
    /// `Block.getState()` is a TileState here: every state has the BlockState basics, as in Java.
    pub fn get_x(&self) -> i32 {
        self.block.get_x()
    }
    /// bukkit: org/bukkit/block/BlockState.getY
    pub fn get_y(&self) -> i32 {
        self.block.get_y()
    }
    /// bukkit: org/bukkit/block/BlockState.getZ
    pub fn get_z(&self) -> i32 {
        self.block.get_z()
    }
    /// bukkit: org/bukkit/block/BlockState.getWorld
    pub fn get_world(&self) -> crate::World {
        self.block.get_world().clone()
    }
    /// bukkit: org/bukkit/block/BlockState.getLocation
    pub fn get_location(&self) -> Location {
        self.block.get_location()
    }
    /// bukkit: org/bukkit/block/BlockState.getChunk
    pub fn get_chunk(&self) -> crate::entity::Chunk {
        self.block.get_chunk()
    }
    /// bukkit: org/bukkit/block/BlockState.getLightLevel
    pub fn get_light_level(&self) -> i32 {
        self.block.get_light_level()
    }
    /// bukkit: org/bukkit/block/BlockState.getBlockData
    ///
    /// The block's data as the game writes it (`minecraft:oak_stairs[facing=east,...]`).
    pub fn get_block_data(&self) -> String {
        match &self.held.kind {
            Some(kind) => kind.key().to_owned(),
            None if !self.state.is_empty() => self.state.clone(),
            None => self.block.get_block_data(),
        }
    }
    /// bukkit: org/bukkit/block/BlockState.isPlaced
    pub fn is_placed(&self) -> bool {
        self.held.kind.is_none()
    }
    /// bukkit: org/bukkit/block/BlockState.update
    ///
    /// Writes what changed. What the game reads back is what it took.
    pub fn update(&self) -> bool {
        let block = &self.block;
        self.changes.iter().all(|(path, snbt)| {
            if let Some(done) = crate::fam_items::special_change(block, path, snbt) {
                return done;
            }
            let patch = crate::data::patch(path, snbt);
            query(json!({ "op": "block_merge", "dimension": block.get_world().dimension(), "pos": [block.get_x(), block.get_y(), block.get_z()], "snbt": patch })).as_bool().unwrap_or(false)
        })
    }
}

/// How a `Sign`'s lines are saved.
fn sign_text(text: &Value) -> Vec<crate::Component> {
    text["messages"].as_array().into_iter().flatten().map(|line| crate::Component(line.clone())).collect()
}

impl Sign {
    /// bukkit: org/bukkit/block/Sign.lines, org/bukkit/block/sign/SignSide.lines
    pub fn lines(&self) -> Vec<crate::Component> {
        sign_text(&self.0.saved("front_text"))
    }
    /// bukkit: org/bukkit/block/Sign.getLines, org/bukkit/block/sign/SignSide.getLines
    pub fn get_lines(&self) -> Vec<String> {
        self.lines().iter().map(crate::Component::to_legacy).collect()
    }
    /// bukkit: org/bukkit/block/Sign.line
    pub fn line(&self, index: usize) -> Option<crate::Component> {
        self.lines().get(index).cloned()
    }
    /// bukkit: org/bukkit/block/Sign.getLine, org/bukkit/block/sign/SignSide.getLine
    pub fn get_line(&self, index: usize) -> Option<String> {
        self.get_lines().get(index).cloned()
    }
    /// bukkit: org/bukkit/block/Sign.setLine, org/bukkit/block/sign/SignSide.setLine
    pub fn set_line(&mut self, index: usize, text: &str) {
        let mut lines = self.lines();
        lines.resize(4, crate::Component::empty());
        if let Some(line) = lines.get_mut(index) {
            *line = crate::adventure::legacy(text);
        }
        let saved: Vec<String> = lines.iter().map(|line| crate::data::snbt(&line.0)).collect();
        self.0.change("front_text.messages", format!("[{}]", saved.join(",")));
    }
}

impl Banner {
    /// bukkit: org/bukkit/block/Banner.getPatterns
    pub fn get_patterns(&self) -> Vec<Pattern> {
        self.0.saved("patterns").as_array().into_iter().flatten().filter_map(|layer| Some(Pattern { pattern: layer["pattern"].as_str()?.to_owned(), color: layer["color"].as_str()?.to_owned() })).collect()
    }
    /// bukkit: org/bukkit/block/Banner.setPatterns
    pub fn set_patterns(&mut self, patterns: &[Pattern]) {
        let saved: Vec<String> = patterns.iter().map(|layer| format!("{{pattern:{},color:{}}}", crate::data::quoted(&layer.pattern), crate::data::quoted(&layer.color))).collect();
        self.0.change("patterns", format!("[{}]", saved.join(",")));
    }
    /// bukkit: org/bukkit/block/Banner.addPattern
    pub fn add_pattern(&mut self, pattern: Pattern) {
        let mut patterns = self.get_patterns();
        patterns.push(pattern);
        self.set_patterns(&patterns);
    }
    /// bukkit: org/bukkit/block/Banner.numberOfPatterns
    pub fn number_of_patterns(&self) -> usize {
        self.get_patterns().len()
    }
    /// bukkit: org/bukkit/block/Banner.getPattern
    pub fn get_pattern(&self, index: usize) -> Option<Pattern> {
        self.get_patterns().get(index).cloned()
    }
    /// bukkit: org/bukkit/block/Banner.getBaseColor
    pub fn get_base_color(&self) -> String {
        if let Some(color) = self.0.saved("BaseColor").as_str() {
            return color.to_owned();
        }
        self.0.get_block().get_type().name().trim_end_matches("_WALL_BANNER").trim_end_matches("_BANNER").to_owned()
    }
}

/// bukkit: org/bukkit/block/BlockFace.getModX
pub fn block_face_mod_x(face: &str) -> i32 {
    face_offset(face).0
}

/// bukkit: org/bukkit/block/BlockFace.getModY
pub fn block_face_mod_y(face: &str) -> i32 {
    face_offset(face).1
}

/// bukkit: org/bukkit/block/BlockFace.getModZ
pub fn block_face_mod_z(face: &str) -> i32 {
    face_offset(face).2
}

/// bukkit: org/bukkit/block/BlockFace.getOppositeFace
pub fn block_face_opposite(face: &str) -> &'static str {
    let (x, y, z) = face_offset(face);
    crate::enums::block_face::values().iter().copied().find(|other| face_offset(other) == (-x, -y, -z)).unwrap_or("SELF")
}

/// bukkit: org/bukkit/block/BlockFace.isCartesian
pub fn block_face_is_cartesian(face: &str) -> bool {
    matches!(face, "UP" | "DOWN" | "NORTH" | "SOUTH" | "EAST" | "WEST")
}

/// bukkit: org/bukkit/block/BlockFace.getDirection
pub fn block_face_direction(face: &str) -> crate::util::Vector {
    let (x, y, z) = face_offset(face);
    let mut direction = crate::util::Vector::new(f64::from(x), f64::from(y), f64::from(z));
    if !direction.is_zero() {
        direction.normalize();
    }
    direction
}

fn face_offset(face: &str) -> (i32, i32, i32) {
    match face {
        "NORTH" => (0, 0, -1),
        "EAST" => (1, 0, 0),
        "SOUTH" => (0, 0, 1),
        "WEST" => (-1, 0, 0),
        "UP" => (0, 1, 0),
        "DOWN" => (0, -1, 0),
        "NORTH_EAST" => (1, 0, -1),
        "NORTH_WEST" => (-1, 0, -1),
        "SOUTH_EAST" => (1, 0, 1),
        "SOUTH_WEST" => (-1, 0, 1),
        "WEST_NORTH_WEST" => (-2, 0, -1),
        "NORTH_NORTH_WEST" => (-1, 0, -2),
        "NORTH_NORTH_EAST" => (1, 0, -2),
        "EAST_NORTH_EAST" => (2, 0, -1),
        "EAST_SOUTH_EAST" => (2, 0, 1),
        "SOUTH_SOUTH_EAST" => (1, 0, 2),
        "SOUTH_SOUTH_WEST" => (-1, 0, 2),
        "WEST_SOUTH_WEST" => (-2, 0, 1),
        _ => (0, 0, 0),
    }
}
