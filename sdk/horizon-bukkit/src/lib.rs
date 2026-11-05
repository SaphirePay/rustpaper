//! Bukkit's API in Rust, so that porting a plugin is a line by line rewrite: the same names in
//! snake_case, the same classes, the same behaviour. A port is an ordinary run-time mod.
//!
//! Each item says what it ports in a `bukkit:` line. `tools/sdk_map.py` reads those lines into the
//! plugin scanner, so the coverage of every plugin follows this file by itself. `bukkit~:` marks a
//! call that isn't exact: the scanner counts it as partial and shows the note to whoever ports with it.
//!
//! Reads are answered by the game at once. Writes go through the game's own commands, so they do
//! what the command does, permissions and all. Inside a player's event a command runs at the end
//! of the tick: `bukkit~` notes where a plugin can see that.
// A plugin's build shows its own warnings only: the SDK's would bury them.
#![allow(warnings)]
pub mod prelude;
pub mod adventure;
pub mod block_data;
pub mod blocks;
pub mod brigadier;
pub mod components;
pub mod config;
pub mod config2;
pub mod core2;
pub mod core3;
pub mod data;
pub mod dialog;
pub mod entities;
pub mod entities2;
pub mod entities3;
pub mod entity;
pub mod enums;
pub mod event;
pub mod event2;
pub mod events;
pub mod extra;
pub mod fam_ev_entity;
pub mod fam_ev_player;
pub mod fam_ev_world;
pub mod fam_entity;
pub mod fam_ess;
pub mod worlds;
pub mod fam_core;
pub mod fam_items;
pub mod fam_paper;
pub mod generated;
pub mod inventory;
pub mod lead;
pub mod lead2;
pub mod meta;
pub mod misc;
pub mod meta_lists;
pub mod persistence;
pub mod player;
pub mod pure;
pub mod pure2;
pub mod registry;
pub mod registry_data;
pub mod scheduler;
pub mod scoreboard;
pub mod server;
pub mod util;
pub mod watch;
pub mod world;

pub use adventure::{Component, mini_message};
pub use horizon_mod::{self, Answer, Value, log};
/// serde_json, for plugins: `json!` and `Map` without adding the crate (the prelude brings `json!`).
pub use serde_json;
use horizon_mod::{Host, query, run_command};
use serde_json::{Map, json};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// ---- Colors ----

/// `org.bukkit.ChatColor`: the section sign codes the game draws.
pub mod chat_color {
    pub const BLACK: &str = "\u{a7}0";
    pub const DARK_BLUE: &str = "\u{a7}1";
    pub const DARK_GREEN: &str = "\u{a7}2";
    pub const DARK_AQUA: &str = "\u{a7}3";
    pub const DARK_RED: &str = "\u{a7}4";
    pub const DARK_PURPLE: &str = "\u{a7}5";
    pub const GOLD: &str = "\u{a7}6";
    pub const GRAY: &str = "\u{a7}7";
    pub const DARK_GRAY: &str = "\u{a7}8";
    pub const BLUE: &str = "\u{a7}9";
    pub const GREEN: &str = "\u{a7}a";
    pub const AQUA: &str = "\u{a7}b";
    pub const RED: &str = "\u{a7}c";
    pub const LIGHT_PURPLE: &str = "\u{a7}d";
    pub const YELLOW: &str = "\u{a7}e";
    pub const WHITE: &str = "\u{a7}f";
    pub const BOLD: &str = "\u{a7}l";
    pub const ITALIC: &str = "\u{a7}o";
    pub const UNDERLINE: &str = "\u{a7}n";
    pub const STRIKETHROUGH: &str = "\u{a7}m";
    pub const MAGIC: &str = "\u{a7}k";
    pub const RESET: &str = "\u{a7}r";

    /// bukkit: org/bukkit/ChatColor.translateAlternateColorCodes
    pub fn translate_alternate_color_codes(code: char, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::with_capacity(text.len());
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == code && chars.get(i + 1).is_some_and(|c| "0123456789AaBbCcDdEeFfKkLlMmNnOoRrXx".contains(*c)) {
                out.push('\u{a7}');
                out.push(chars[i + 1].to_ascii_lowercase());
                i += 2;
            } else {
                out.push(chars[i]);
                i += 1;
            }
        }
        out
    }

    /// bukkit: org/bukkit/ChatColor.stripColor
    pub fn strip_color(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\u{a7}' {
                chars.next();
            } else {
                out.push(c);
            }
        }
        out
    }
}

// ---- Worlds and positions ----

fn sound_source(category: &str) -> &'static str {
    // Bukkit's SoundCategory names to the game's sound sources (the player's volume sliders).
    match category.to_ascii_uppercase().as_str() {
        "MUSIC" => "music",
        "RECORDS" | "RECORD" => "record",
        "WEATHER" => "weather",
        "BLOCKS" | "BLOCK" => "block",
        "HOSTILE" => "hostile",
        "NEUTRAL" => "neutral",
        "PLAYERS" | "PLAYER" => "player",
        "AMBIENT" => "ambient",
        "VOICE" => "voice",
        _ => "master",
    }
}

fn dimension_of(world: &str) -> &str {
    match world {
        "world_nether" | "minecraft:the_nether" | "the_nether" => "minecraft:the_nether",
        "world_the_end" | "minecraft:the_end" | "the_end" => "minecraft:the_end",
        "world" | "minecraft:overworld" | "overworld" | "" => "minecraft:overworld",
        created => worlds::dimension_of_created(created).unwrap_or("minecraft:overworld"),
    }
}

pub(crate) fn world_of(dimension: &str) -> &'static str {
    match dimension {
        "minecraft:the_nether" | "the_nether" => "world_nether",
        "minecraft:the_end" | "the_end" => "world_the_end",
        "minecraft:overworld" | "overworld" | "" => "world",
        created => worlds::name_of_created(created).unwrap_or("world"),
    }
}

/// `org.bukkit.World`: Bukkit's three default names stand for the three levels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    name: String,
}

impl World {
    pub fn of(name: &str) -> Self {
        Self { name: world_of(dimension_of(name)).to_owned() }
    }
    /// bukkit: org/bukkit/World.getName, org/bukkit/WorldInfo.getName, org/bukkit/generator/WorldInfo.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub(crate) fn dimension(&self) -> &str {
        dimension_of(&self.name)
    }
    /// bukkit: org/bukkit/World.getBlockAt
    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> Block {
        Block { world: self.clone(), x, y, z }
    }
    /// bukkit: org/bukkit/World.getPlayers
    pub fn get_players(&self) -> Vec<Player> {
        get_online_players().into_iter().filter(|player| player.get_world() == *self).collect()
    }
    /// bukkit: org/bukkit/World.getTime
    pub fn get_time(&self) -> i64 {
        query(json!({ "op": "world" }))["time"].as_i64().unwrap_or(0) % 24000
    }
    /// bukkit: org/bukkit/World.getFullTime, org/bukkit/World.getGameTime
    pub fn get_full_time(&self) -> i64 {
        query(json!({ "op": "world" }))["game_time"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/World.hasStorm
    pub fn has_storm(&self) -> bool {
        query(json!({ "op": "level", "dimension": self.dimension() }))["raining"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/World.setTime
    pub fn set_time(&self, time: i64) {
        query(json!({ "op": "core_set_time", "dimension": self.dimension(), "time": time }));
    }
    /// bukkit: org/bukkit/World.setStorm
    pub fn set_storm(&self, storm: bool) {
        query(json!({ "op": "ev_world_free_weather_plugin" }));
        query(json!({ "op": "core_set_storm", "dimension": self.dimension(), "storm": storm }));
    }
    /// bukkit: org/bukkit/World.spawnEntity
    ///
    /// By the game's id, at once, as `/summon` makes it.
    pub fn spawn_entity(&self, location: &Location, kind: &str) -> Option<entity::Entity> {
        let key = if kind.contains(':') { kind.to_owned() } else { format!("minecraft:{}", kind.to_ascii_lowercase()) };
        let tree = format!("{{id:{},Pos:[{:?}d,{:?}d,{:?}d],Rotation:[{:?}f,{:?}f]}}", data::quoted(&key), location.x, location.y, location.z, location.yaw, location.pitch);
        entity::Entity::read(&query(json!({ "op": "spawn", "dimension": self.dimension(), "snbt": tree })))
    }
}

/// bukkit: org/bukkit/Bukkit.getWorld, org/bukkit/Server.getWorld
pub fn get_world(name: &str) -> Option<World> {
    let built_in = match name {
        "world" => true,
        "world_nether" => fam_core::get_allow_nether(),
        "world_the_end" => fam_core::get_allow_end(),
        _ => false,
    };
    (built_in || worlds::dimension_of_created(name).is_some()).then(|| World::of(name))
}

/// bukkit: org/bukkit/Bukkit.getWorlds, org/bukkit/Server.getWorlds
pub fn get_worlds() -> Vec<World> {
    let mut all: Vec<World> = ["world", "world_nether", "world_the_end"].iter().filter_map(|name| get_world(name)).collect();
    all.extend(worlds::created().iter().filter_map(|row| row["id"].as_str()).map(World::of));
    all
}

/// `org.bukkit.Location`.
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub world: World,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl Location {
    /// bukkit: org/bukkit/Location.<init>
    pub fn new(world: World, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Self {
        Self { world, x, y, z, yaw, pitch }
    }
    /// bukkit: org/bukkit/Location.getWorld
    pub fn get_world(&self) -> &World {
        &self.world
    }
    /// bukkit: org/bukkit/Location.getX
    pub fn get_x(&self) -> f64 {
        self.x
    }
    /// bukkit: org/bukkit/Location.getY
    pub fn get_y(&self) -> f64 {
        self.y
    }
    /// bukkit: org/bukkit/Location.getZ
    pub fn get_z(&self) -> f64 {
        self.z
    }
    /// bukkit: org/bukkit/Location.getYaw
    pub fn get_yaw(&self) -> f32 {
        self.yaw
    }
    /// bukkit: org/bukkit/Location.getPitch
    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }
    /// bukkit: org/bukkit/Location.getBlockX
    pub fn get_block_x(&self) -> i32 {
        self.x.floor() as i32
    }
    /// bukkit: org/bukkit/Location.getBlockY
    pub fn get_block_y(&self) -> i32 {
        self.y.floor() as i32
    }
    /// bukkit: org/bukkit/Location.getBlockZ
    pub fn get_block_z(&self) -> i32 {
        self.z.floor() as i32
    }
    /// bukkit: org/bukkit/Location.getBlock
    pub fn get_block(&self) -> Block {
        self.world.get_block_at(self.get_block_x(), self.get_block_y(), self.get_block_z())
    }
    /// bukkit: org/bukkit/Location.add
    pub fn add(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self
    }
    /// bukkit: org/bukkit/Location.distanceSquared
    pub fn distance_squared(&self, other: &Self) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)
    }
    /// bukkit: org/bukkit/Location.distance
    pub fn distance(&self, other: &Self) -> f64 {
        self.distance_squared(other).sqrt()
    }
}

/// `org.bukkit.Material`, as the game's item and block id. `Material::of("DIAMOND_SWORD")` and
/// `Material::of("minecraft:diamond_sword")` are the same.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Material(String);

impl Material {
    /// bukkit: org/bukkit/Material.valueOf, org/bukkit/Material.matchMaterial
    pub fn of(name: &str) -> Self {
        let lower = name.to_ascii_lowercase();
        Self(if lower.contains(':') { lower } else { format!("minecraft:{lower}") })
    }
    /// The id the game uses.
    pub fn key(&self) -> &str {
        &self.0
    }
    /// bukkit: org/bukkit/Material.name
    pub fn name(&self) -> String {
        self.0.trim_start_matches("minecraft:").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/Material.getMaxStackSize
    pub fn get_max_stack_size(&self) -> i32 {
        // Fixed for the game's run: asked once per material.
        static INFO: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, Value>>> = std::sync::OnceLock::new();
        crate::fam_items::fixed_answer(&INFO, &self.0, || query(json!({ "op": "item_info", "item": self.0 })))["max_stack"].as_i64().unwrap_or(64) as i32
    }
    /// bukkit: org/bukkit/Material.isAir
    pub fn is_air(&self) -> bool {
        matches!(self.0.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
    }
}

/// `org.bukkit.block.Block`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    world: World,
    x: i32,
    y: i32,
    z: i32,
}

impl Block {
    /// bukkit: org/bukkit/block/Block.getType
    pub fn get_type(&self) -> Material {
        let answer = query(json!({ "op": "block", "dimension": self.world.dimension(), "pos": [self.x, self.y, self.z] }));
        Material::of(answer["name"].as_str().unwrap_or("minecraft:air"))
    }
    /// bukkit: org/bukkit/block/Block.setType
    pub fn set_type(&self, material: &Material) -> bool {
        self.set_block_data(material.key())
    }
    fn read(&self) -> Value {
        query(json!({ "op": "block", "dimension": self.world.dimension(), "pos": [self.x, self.y, self.z] }))
    }
    /// bukkit: org/bukkit/block/Block.getBlockData, org/bukkit/block/data/BlockData.getAsString
    ///
    /// As the game writes it: `minecraft:oak_stairs[facing=east,half=bottom]`.
    pub fn get_block_data(&self) -> String {
        let block = self.read();
        let name = block["name"].as_str().unwrap_or("minecraft:air");
        let properties: Vec<String> = block["properties"].as_object().into_iter().flatten().map(|(key, value)| format!("{key}={}", value.as_str().unwrap_or_default())).collect();
        if properties.is_empty() { name.to_owned() } else { format!("{name}[{}]", properties.join(",")) }
    }
    /// One property of the block: Java's `BlockData` subclasses, `Directional.getFacing` and so on.
    pub fn get_property(&self, key: &str) -> Option<String> {
        self.read()["properties"][key].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Block.setBlockData
    pub fn set_block_data(&self, data: &str) -> bool {
        query(json!({ "op": "set_block", "dimension": self.world.dimension(), "pos": [self.x, self.y, self.z], "block": data })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/block/Block.breakNaturally
    pub fn break_naturally(&self) -> bool {
        self.break_naturally_with(None, false, false, false)
    }
    /// bukkit: org/bukkit/block/Block.getState
    pub fn get_state(&self) -> BlockState {
        BlockState { block: self.clone(), data: self.get_block_data() }
    }
    /// bukkit: org/bukkit/block/Block.getRelative
    pub fn get_relative(&self, x: i32, y: i32, z: i32) -> Self {
        Self { world: self.world.clone(), x: self.x + x, y: self.y + y, z: self.z + z }
    }
    /// bukkit: org/bukkit/block/Block.getX
    pub fn get_x(&self) -> i32 {
        self.x
    }
    /// bukkit: org/bukkit/block/Block.getY
    pub fn get_y(&self) -> i32 {
        self.y
    }
    /// bukkit: org/bukkit/block/Block.getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
    /// bukkit: org/bukkit/block/Block.getWorld
    pub fn get_world(&self) -> &World {
        &self.world
    }
    /// bukkit: org/bukkit/block/Block.getLocation
    pub fn get_location(&self) -> Location {
        Location::new(self.world.clone(), f64::from(self.x), f64::from(self.y), f64::from(self.z), 0.0, 0.0)
    }
    /// bukkit: org/bukkit/block/Block.isEmpty
    pub fn is_empty(&self) -> bool {
        self.get_type().is_air()
    }
}

/// `org.bukkit.potion.PotionEffect`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PotionEffect {
    id: String,
    duration: i32,
    amplifier: i32,
    ambient: bool,
    particles: bool,
    pub(crate) icon: bool,
    pub(crate) hidden: Option<Box<Self>>,
}

impl PotionEffect {
    fn read(value: &Value) -> Option<Self> {
        Some(Self {
            id: value["id"].as_str()?.to_owned(),
            duration: value["duration"].as_i64().unwrap_or(0) as i32,
            amplifier: value["amplifier"].as_i64().unwrap_or(0) as i32,
            ambient: value["ambient"].as_bool().unwrap_or(false),
            particles: value["visible"].as_bool().unwrap_or(true),
            icon: value["visible"].as_bool().unwrap_or(true),
            hidden: value.get("hidden").and_then(Self::read).map(Box::new),
        })
    }
    /// bukkit: org/bukkit/potion/PotionEffect.getType, org/bukkit/potion/PotionEffectType.getName, org/bukkit/potion/PotionEffectType.getKey
    pub fn get_type(&self) -> &str {
        &self.id
    }
    pub(crate) fn is(&self, effect: &str) -> bool {
        self.id.trim_start_matches("minecraft:") == effect.trim_start_matches("minecraft:")
    }
    /// bukkit: org/bukkit/potion/PotionEffect.<init>
    pub fn new(id: &str, duration: i32, amplifier: i32, ambient: bool, particles: bool) -> Self {
        Self { id: id.to_owned(), duration, amplifier, ambient, particles, icon: particles, hidden: None }
    }
    /// `new PotionEffect(type, duration, amplifier)`: Bukkit's defaults, not ambient, with particles.
    /// bukkit: org/bukkit/potion/PotionEffect.<init>
    pub fn of(id: &str, duration: i32, amplifier: i32) -> Self {
        Self::new(id, duration, amplifier, false, true)
    }
    /// `new PotionEffect(type, duration, amplifier, ambient)`.
    /// bukkit: org/bukkit/potion/PotionEffect.<init>
    pub fn of_ambient(id: &str, duration: i32, amplifier: i32, ambient: bool) -> Self {
        Self::new(id, duration, amplifier, ambient, true)
    }
    /// `new PotionEffect(type, duration, amplifier, ambient, particles, icon)`.
    /// bukkit: org/bukkit/potion/PotionEffect.<init>
    pub fn of_full(id: &str, duration: i32, amplifier: i32, ambient: bool, particles: bool, icon: bool) -> Self {
        Self { icon, ..Self::new(id, duration, amplifier, ambient, particles) }
    }
    /// An effect as items and entities save it.
    pub(crate) fn from_saved(value: &Value) -> Option<Self> {
        Some(Self {
            id: value["id"].as_str()?.to_owned(),
            duration: value["duration"].as_i64().unwrap_or(1) as i32,
            amplifier: value["amplifier"].as_i64().unwrap_or(0) as i32,
            ambient: data::flag(&value["ambient"]),
            particles: value.get("show_particles").is_none_or(data::flag),
            icon: value.get("show_icon").or_else(|| value.get("show_particles")).is_none_or(data::flag),
            hidden: value.get("hidden_effect").and_then(Self::from_saved).map(Box::new),
        })
    }
    pub(crate) fn to_saved(&self) -> Value {
        json!({ "id": self.id, "duration": self.duration, "amplifier": self.amplifier, "ambient": self.ambient, "show_particles": self.particles })
    }
    /// bukkit: org/bukkit/potion/PotionEffect.getDuration
    pub fn get_duration(&self) -> i32 {
        self.duration
    }
    /// bukkit: org/bukkit/potion/PotionEffect.getAmplifier
    pub fn get_amplifier(&self) -> i32 {
        self.amplifier
    }
    /// bukkit: org/bukkit/potion/PotionEffect.isAmbient
    pub fn is_ambient(&self) -> bool {
        self.ambient
    }
    /// bukkit: org/bukkit/potion/PotionEffect.hasParticles
    pub fn has_particles(&self) -> bool {
        self.particles
    }
}

/// Who each boss bar shows to: `/bossbar set players` replaces the list.

/// `org.bukkit.block.BlockState`: the block as it was, written back by `update`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockState {
    block: Block,
    data: String,
}

impl BlockState {
    /// A state the block isn't in, for an event's "what it will be".
    pub(crate) fn snapshot(block: Block, data: &str) -> Self {
        Self { block, data: data.to_owned() }
    }
    /// bukkit: org/bukkit/block/BlockState.getBlock
    ///
    /// # Panics
    /// As Java throws, for a state that isn't placed.
    pub fn get_block(&self) -> &Block {
        assert!(self.placed(), "The blockState must be placed to call this method");
        &self.block
    }
    pub(crate) fn placed(&self) -> bool {
        self.block.y != fam_items::UNPLACED_Y
    }
    /// bukkit: org/bukkit/block/BlockState.getBlockData
    pub fn get_block_data(&self) -> &str {
        &self.data
    }
    /// bukkit: org/bukkit/block/BlockState.setBlockData
    pub fn set_block_data(&mut self, data: &str) {
        data.clone_into(&mut self.data);
    }
    /// bukkit: org/bukkit/block/BlockState.getType
    pub fn get_type(&self) -> Material {
        Material::of(self.data.split('[').next().unwrap_or("minecraft:air"))
    }
    /// bukkit: org/bukkit/block/BlockState.update
    ///
    /// False, and no change, when the block is of another type now. `TileState` carries a block
    /// entity's contents; this state is the block's alone.
    pub fn update(&self) -> bool {
        !self.placed() || (self.block.get_type() == self.get_type() && self.block.set_block_data(&self.data))
    }
    /// bukkit: org/bukkit/block/BlockState.update
    ///
    /// `force` writes whatever the block is now. Neighbours are always told: `physics` as Java's `true`.
    pub fn update_forced(&self) -> bool {
        !self.placed() || self.block.set_block_data(&self.data)
    }
}

/// `org.bukkit.inventory.ItemStack`, with its `ItemMeta`: the game's components, by id, as JSON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStack {
    material: Material,
    amount: i32,
    components: BTreeMap<String, Value>,
    pub(crate) legacy: fam_items::LegacySlot,
}

impl ItemStack {
    /// bukkit: org/bukkit/inventory/ItemStack.<init>, org/bukkit/inventory/ItemStack.of
    pub fn new(material: Material, amount: i32) -> Self {
        Self { material, amount, components: BTreeMap::new(), legacy: Default::default() }
    }
    /// `new ItemStack(material)`: one of it. Paper's `ItemStack.of(material)` too.
    /// bukkit: org/bukkit/inventory/ItemStack.<init>, org/bukkit/inventory/ItemStack.of
    pub fn of_one(material: Material) -> Self {
        Self::new(material, 1)
    }
    /// A component, by its id: the way to what the typed calls don't reach.
    pub fn get_component(&self, id: &str) -> Option<&Value> {
        self.components.get(id)
    }
    /// Every component the stack carries.
    pub fn all_components(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.components.iter()
    }
    /// bukkit: io/papermc/paper/datacomponent/DataComponentHolder.setData
    pub fn set_component(&mut self, id: &str, value: Value) {
        self.components.insert(id.to_owned(), value);
    }
    /// bukkit: io/papermc/paper/datacomponent/DataComponentHolder.unsetData
    pub fn remove_component(&mut self, id: &str) -> Option<Value> {
        self.components.remove(id)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasDisplayName
    pub fn has_display_name(&self) -> bool {
        self.components.contains_key("minecraft:custom_name")
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasLore
    pub fn has_lore(&self) -> bool {
        self.components.contains_key("minecraft:lore")
    }
    fn enchants(&self) -> Option<&Map<String, Value>> {
        let value = self.components.get("minecraft:enchantments")?;
        value.get("levels").unwrap_or(value).as_object()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getEnchants, org/bukkit/inventory/ItemStack.getEnchantments
    pub fn get_enchants(&self) -> BTreeMap<String, u32> {
        self.enchants().into_iter().flatten().map(|(key, level)| (key.clone(), level.as_u64().unwrap_or(0) as u32)).collect()
    }
    fn write_enchants(&mut self, levels: BTreeMap<String, u32>) {
        if levels.is_empty() {
            self.components.remove("minecraft:enchantments");
        } else {
            self.components.insert("minecraft:enchantments".to_owned(), json!(levels));
        }
    }
    fn key(enchantment: &str) -> String {
        if enchantment.contains(':') { enchantment.to_owned() } else { format!("minecraft:{enchantment}") }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.addEnchant, org/bukkit/inventory/ItemStack.addEnchantment, org/bukkit/inventory/ItemStack.addUnsafeEnchantment
    pub fn add_enchant(&mut self, enchantment: &str, level: u32) {
        let mut levels = self.get_enchants();
        levels.insert(Self::key(enchantment), level);
        self.write_enchants(levels);
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.removeEnchant
    pub fn remove_enchant(&mut self, enchantment: &str) -> bool {
        let mut levels = self.get_enchants();
        let had = levels.remove(&Self::key(enchantment)).is_some();
        self.write_enchants(levels);
        had
    }
    /// bukkit: org/bukkit/inventory/ItemStack.removeEnchantment
    pub fn remove_enchantment(&mut self, enchantment: &str) -> u32 {
        let level = self.get_enchant_level(enchantment);
        self.remove_enchant(enchantment);
        level
    }
    /// bukkit: org/bukkit/inventory/ItemStack.removeEnchantments, org/bukkit/inventory/meta/ItemMeta.removeEnchantments
    pub fn remove_enchantments(&mut self) {
        self.components.remove("minecraft:enchantments");
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasEnchant, org/bukkit/inventory/ItemStack.containsEnchantment
    pub fn has_enchant(&self, enchantment: &str) -> bool {
        self.get_enchants().contains_key(&Self::key(enchantment))
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getEnchantLevel, org/bukkit/inventory/ItemStack.getEnchantmentLevel
    pub fn get_enchant_level(&self, enchantment: &str) -> u32 {
        self.get_enchants().get(&Self::key(enchantment)).copied().unwrap_or(0)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasEnchants
    pub fn has_enchants(&self) -> bool {
        self.enchants().is_some_and(|levels| !levels.is_empty())
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setCustomModelData
    pub fn set_custom_model_data(&mut self, data: Option<i32>) {
        match data {
            Some(data) => self.set_component("minecraft:custom_model_data", json!({ "floats": [data] })),
            None => {
                self.components.remove("minecraft:custom_model_data");
            }
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getCustomModelData
    pub fn get_custom_model_data(&self) -> Option<i32> {
        self.components.get("minecraft:custom_model_data")?["floats"][0].as_f64().map(|data| data as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasCustomModelData
    pub fn has_custom_model_data(&self) -> bool {
        self.get_custom_model_data().is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/Damageable.getDamage
    pub fn get_damage(&self) -> i32 {
        self.components.get("minecraft:damage").and_then(Value::as_i64).unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/inventory/meta/Damageable.setDamage
    pub fn set_damage(&mut self, damage: i32) {
        self.set_component("minecraft:damage", json!(damage));
    }
    /// bukkit: org/bukkit/inventory/meta/Damageable.hasDamage
    pub fn has_damage(&self) -> bool {
        self.get_damage() > 0
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setUnbreakable
    pub fn set_unbreakable(&mut self, unbreakable: bool) {
        if unbreakable {
            self.set_component("minecraft:unbreakable", json!({}));
        } else {
            self.components.remove("minecraft:unbreakable");
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.isUnbreakable
    pub fn is_unbreakable(&self) -> bool {
        self.components.contains_key("minecraft:unbreakable")
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setHideTooltip
    pub fn hide_tooltip(&mut self, hidden: bool) {
        self.set_component("minecraft:tooltip_display", json!({ "hide_tooltip": hidden }));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.isHideTooltip
    pub fn is_hide_tooltip(&self) -> bool {
        self.components.get("minecraft:tooltip_display").is_some_and(|display| display["hide_tooltip"].as_bool() == Some(true) || display["hide_tooltip"].as_i64() == Some(1))
    }
    /// The stack as the game's commands write it: `minecraft:stone[minecraft:damage=3]`.
    pub fn to_command(&self) -> String {
        let parts: Vec<String> = self.components.iter().map(|(id, value)| if id.starts_with('!') { id.clone() } else { format!("{id}={}", data::snbt(value)) }).collect();
        if parts.is_empty() { self.material.key().to_owned() } else { format!("{}[{}]", self.material.key(), parts.join(",")) }
    }
    /// The stack as entities and block entities save it.
    pub fn to_snbt(&self) -> String {
        let parts: Vec<String> = self.components.iter().map(|(id, value)| format!("\"{id}\":{}", data::snbt(value))).collect();
        format!("{{id:\"{}\",count:{},components:{{{}}}}}", self.material.key(), self.amount, parts.join(","))
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setDisplayName, org/bukkit/inventory/ItemStack.getItemMeta, org/bukkit/inventory/ItemStack.setItemMeta
    pub fn set_display_name(&mut self, name: &str) {
        self.set_component("minecraft:custom_name", adventure::legacy(name).0);
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.displayName
    /// bukkit: org/bukkit/inventory/ItemStack.displayName
    pub fn display_name(&self) -> Option<Component> {
        self.components.get("minecraft:custom_name").map(|name| Component(name.clone()))
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setDisplayName
    pub fn set_display_name_component(&mut self, name: &Component) {
        self.set_component("minecraft:custom_name", name.0.clone());
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getDisplayName
    pub fn get_display_name(&self) -> Option<String> {
        self.display_name().map(|name| name.to_legacy())
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setLore, org/bukkit/inventory/ItemStack.setLore
    pub fn set_lore(&mut self, lore: Vec<String>) {
        if lore.is_empty() {
            self.components.remove("minecraft:lore");
        } else {
            self.set_component("minecraft:lore", Value::Array(lore.iter().map(|line| adventure::legacy(line).0).collect()));
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.lore, org/bukkit/inventory/ItemStack.lore
    pub fn lore(&self) -> Vec<Component> {
        self.components.get("minecraft:lore").and_then(Value::as_array).into_iter().flatten().map(|line| Component(line.clone())).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getLore, org/bukkit/inventory/ItemStack.getLore
    pub fn get_lore(&self) -> Vec<String> {
        self.lore().iter().map(Component::to_legacy).collect()
    }
    /// A stack for the game to read: each component as SNBT.
    pub(crate) fn to_json(&self) -> Value {
        let components: Map<String, Value> = self.components.iter().map(|(id, value)| (id.clone(), json!(data::snbt(value)))).collect();
        json!({ "item": self.material.key(), "count": self.amount, "components": components })
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getType
    pub fn get_type(&self) -> &Material {
        &self.material
    }
    /// bukkit: org/bukkit/inventory/ItemStack.setType
    pub fn set_type(&mut self, material: Material) {
        self.material = material;
        self.legacy = Default::default();
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getAmount
    pub fn get_amount(&self) -> i32 {
        self.amount
    }
    /// bukkit: org/bukkit/inventory/ItemStack.setAmount
    pub fn set_amount(&mut self, amount: i32) {
        self.amount = amount;
    }
    /// bukkit: org/bukkit/inventory/ItemStack.getMaxStackSize
    pub fn get_max_stack_size(&self) -> i32 {
        self.components.get("minecraft:max_stack_size").and_then(Value::as_i64).unwrap_or_else(|| i64::from(self.material.get_max_stack_size())) as i32
    }
    /// bukkit: org/bukkit/inventory/ItemStack.isSimilar
    pub fn is_similar(&self, other: &Self) -> bool {
        self.material == other.material && self.components == other.components
    }
    /// bukkit: org/bukkit/inventory/ItemStack.hasItemMeta
    pub fn has_item_meta(&self) -> bool {
        !self.components.is_empty()
    }
    /// bukkit: org/bukkit/inventory/ItemStack.clone
    pub fn copy(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/inventory/ItemStack.asOne
    pub fn as_one(&self) -> Self {
        self.as_quantity(1)
    }
    /// bukkit: org/bukkit/inventory/ItemStack.asQuantity
    pub fn as_quantity(&self, amount: i32) -> Self {
        let mut stack = self.clone();
        stack.amount = amount;
        stack
    }
    /// bukkit: org/bukkit/inventory/ItemStack.add
    pub fn add(&mut self, amount: i32) -> &mut Self {
        self.amount += amount;
        self
    }
    /// bukkit: org/bukkit/inventory/ItemStack.subtract
    pub fn subtract(&mut self, amount: i32) -> &mut Self {
        self.amount -= amount;
        self
    }
    /// bukkit: org/bukkit/inventory/ItemStack.isEmpty
    pub fn is_empty(&self) -> bool {
        self.amount <= 0 || self.material.is_air()
    }
    /// bukkit: org/bukkit/inventory/ItemStack.empty
    pub fn empty() -> Self {
        Self::new(Material::of("AIR"), 0)
    }
    pub(crate) fn read(value: &Value) -> Option<Self> {
        let mut components: BTreeMap<String, Value> = value["components"].as_object().into_iter().flatten().map(|(id, value)| (id.clone(), value.clone())).collect();
        if let Some(damage) = value["damage"].as_i64().filter(|damage| *damage > 0) {
            components.entry("minecraft:damage".to_owned()).or_insert(json!(damage));
        }
        Some(Self { material: Material::of(value["item"].as_str().or(value["id"].as_str())?), amount: value["count"].as_i64().unwrap_or(1) as i32, components, legacy: Default::default() })
    }
}

/// `org.bukkit.inventory.PlayerInventory`.
#[derive(Clone, Debug)]
pub struct PlayerInventory {
    player: Player,
}

fn slot_name(slot: i32) -> String {
    if slot < 9 { format!("hotbar.{slot}") } else { format!("inventory.{}", slot - 9) }
}

impl PlayerInventory {
    /// bukkit: org/bukkit/inventory/Inventory.getContents, org/bukkit/inventory/PlayerInventory.getContents
    pub fn get_contents(&self) -> Vec<Option<ItemStack>> {
        let answer = query(json!({ "op": "inventory", "uuid": self.player.uuid }));
        answer.as_array().into_iter().flatten().map(ItemStack::read).collect()
    }
    /// bukkit: org/bukkit/inventory/Inventory.getItem, org/bukkit/inventory/PlayerInventory.getItem
    pub fn get_item(&self, slot: i32) -> Option<ItemStack> {
        self.get_contents().get(usize::try_from(slot).ok()?).cloned().flatten()
    }
    fn equipment(&self) -> Value {
        query(json!({ "op": "equipment", "uuid": self.player.uuid }))
    }
    pub(crate) fn owner(&self) -> &Player {
        &self.player
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getItemInMainHand
    pub fn get_item_in_main_hand(&self) -> Option<ItemStack> {
        ItemStack::read(&self.equipment()["main_hand"])
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getItemInOffHand
    pub fn get_item_in_off_hand(&self) -> Option<ItemStack> {
        ItemStack::read(&self.equipment()["off_hand"])
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getHelmet
    pub fn get_helmet(&self) -> Option<ItemStack> {
        ItemStack::read(&self.equipment()["armor"][0])
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getChestplate
    pub fn get_chestplate(&self) -> Option<ItemStack> {
        ItemStack::read(&self.equipment()["armor"][1])
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getLeggings
    pub fn get_leggings(&self) -> Option<ItemStack> {
        ItemStack::read(&self.equipment()["armor"][2])
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getBoots
    pub fn get_boots(&self) -> Option<ItemStack> {
        ItemStack::read(&self.equipment()["armor"][3])
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getArmorContents
    pub fn get_armor_contents(&self) -> Vec<Option<ItemStack>> {
        let equipment = self.equipment();
        equipment["armor"].as_array().into_iter().flatten().rev().map(ItemStack::read).collect()
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.getHeldItemSlot
    pub fn get_held_item_slot(&self) -> i32 {
        self.player.state()["selected"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setItemInMainHand
    pub fn set_item_in_main_hand(&self, stack: &ItemStack) {
        run_command(&format!("item replace entity {} weapon.mainhand with {} {}", self.player.name, stack.to_command(), stack.amount));
    }
    /// bukkit: org/bukkit/inventory/PlayerInventory.setItemInOffHand
    pub fn set_item_in_off_hand(&self, stack: &ItemStack) {
        run_command(&format!("item replace entity {} weapon.offhand with {} {}", self.player.name, stack.to_command(), stack.amount));
    }
}

/// `org.bukkit.inventory.Inventory` made with `Bukkit.createInventory`: a chest GUI.
#[derive(Clone, Debug)]
pub struct Inventory {
    id: u32,
    title: String,
    contents: Vec<Option<ItemStack>>,
}

static NEXT_INVENTORY: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
/// Who has which GUI open, for the click events.
pub(crate) static OPEN_INVENTORIES: Mutex<BTreeMap<String, Inventory>> = Mutex::new(BTreeMap::new());

impl Inventory {
    /// bukkit: org/bukkit/Bukkit.createInventory, org/bukkit/Server.createInventory
    pub fn create(size: usize, title: &str) -> Self {
        Self { id: NEXT_INVENTORY.fetch_add(1, std::sync::atomic::Ordering::Relaxed), title: title.to_owned(), contents: vec![None; size] }
    }
    /// bukkit: org/bukkit/inventory/Inventory.getSize
    pub fn get_size(&self) -> usize {
        self.contents.len()
    }
    /// The same GUI, whichever copy of it this is: Java's `inventory == this.gui`.
    pub fn same_as(&self, other: &Self) -> bool {
        self.id == other.id
    }
    /// bukkit: org/bukkit/inventory/Inventory.getType
    pub fn get_type(&self) -> &'static str {
        fam_items::inventory_kind(self.id)
    }
    /// bukkit: org/bukkit/inventory/Inventory.getContents, org/bukkit/inventory/Inventory.getStorageContents
    pub fn get_contents(&self) -> &[Option<ItemStack>] {
        &self.contents
    }
    /// bukkit: org/bukkit/inventory/Inventory.firstEmpty
    pub fn first_empty(&self) -> i32 {
        self.contents.iter().position(Option::is_none).map_or(-1, |slot| slot as i32)
    }
    /// bukkit: org/bukkit/inventory/Inventory.clear
    pub fn clear(&mut self) {
        self.contents.fill(None);
        fam_items::gui_cleared(self.id);
    }
    /// bukkit: org/bukkit/inventory/InventoryView.getTitle
    pub fn get_title(&self) -> &str {
        &self.title
    }
    /// bukkit: org/bukkit/inventory/Inventory.setItem, org/bukkit/inventory/Inventory.getItem
    pub fn set_item(&mut self, slot: usize, stack: Option<ItemStack>) {
        if let Some(place) = self.contents.get_mut(slot) {
            place.clone_from(&stack);
            fam_items::gui_slot(self.id, slot, stack.as_ref());
        }
    }
    /// bukkit: org/bukkit/inventory/Inventory.getItem
    pub fn get_item(&self, slot: usize) -> Option<&ItemStack> {
        self.contents.get(slot)?.as_ref()
    }
    fn items(&self) -> Value {
        Value::Array(self.contents.iter().map(|stack| stack.as_ref().map_or(Value::Null, ItemStack::to_json)).collect())
    }
}

// ---- Permissions ----

/// `plugin.yml`'s defaults: `true`, `false`, `op`, `not op`.
static PERMISSION_DEFAULTS: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());
static PERMISSION_GRANTS: Mutex<BTreeMap<(String, String), bool>> = Mutex::new(BTreeMap::new());

pub(crate) fn set_permission_default(node: &str, default: &str) {
    PERMISSION_DEFAULTS.lock().unwrap_or_else(|e| e.into_inner()).insert(node.to_owned(), default.to_ascii_lowercase());
}

pub(crate) fn permission_is_granted(uuid: &str, node: &str) -> Option<bool> {
    PERMISSION_GRANTS.lock().unwrap_or_else(|e| e.into_inner()).get(&(uuid.to_owned(), node.to_owned())).copied()
}

pub(crate) fn grant_permission(uuid: &str, node: &str, value: Option<bool>) {
    fam_core::share_grant(uuid, node, value);
    let mut grants = PERMISSION_GRANTS.lock().unwrap_or_else(|e| e.into_inner());
    match value {
        Some(value) => grants.insert((uuid.to_owned(), node.to_owned()), value),
        None => grants.remove(&(uuid.to_owned(), node.to_owned())),
    };
}

fn permitted(uuid: Option<&str>, operator: bool, node: &str) -> bool {
    if let Some(answer) = uuid.and_then(|uuid| fam_core::shared_permission(uuid, operator, node)) {
        return answer;
    }
    if let Some(uuid) = uuid
        && let Some(granted) = PERMISSION_GRANTS.lock().unwrap_or_else(|e| e.into_inner()).get(&(uuid.to_owned(), node.to_owned()))
    {
        return *granted;
    }
    match PERMISSION_DEFAULTS.lock().unwrap_or_else(|e| e.into_inner()).get(node).map(String::as_str) {
        Some("true") => true,
        Some("false") => false,
        Some("not op") => !operator,
        // Bukkit's default for a node nobody declared, and `default: op`.
        _ => operator,
    }
}

/// A query from code that knows it's on the main thread, in a tick's task.
pub fn query_from_main(request: Value) -> Value {
    query(request)
}

// ---- Players ----

/// `org.bukkit.entity.Player`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    uuid: String,
    name: String,
}

impl Player {
    pub(crate) fn uuid(&self) -> &str {
        &self.uuid
    }
    pub(crate) fn read(value: &Value) -> Option<Self> {
        Some(Self { uuid: value["uuid"].as_str()?.to_owned(), name: value["name"].as_str()?.to_owned() })
    }
    /// A player the engine names by UUID (32 hex digits) and name, who may still be logging in: the one a
    /// packet goes to, which ProtocolLib's `PacketEvent.getPlayer()` gives before the join event.
    pub fn of_ids(uuid: &str, name: &str) -> Self {
        Self { uuid: uuid.to_owned(), name: name.to_owned() }
    }
    fn state(&self) -> Value {
        query(json!({ "op": "player", "uuid": self.uuid }))
    }
    pub(crate) fn player_state(&self) -> Value {
        self.state()
    }
    pub(crate) fn equipment_state(&self) -> Value {
        query(json!({ "op": "equipment", "uuid": self.uuid }))
    }
    pub(crate) fn send_held_slot(&self, slot: i32) {
        query(json!({ "op": "player_send", "uuid": self.uuid, "packet": "held_slot", "slot": slot }));
    }
    /// bukkit: org/bukkit/entity/Player.getName, org/bukkit/entity/AnimalTamer.getName, org/bukkit/entity/HumanEntity.getName, org/bukkit/OfflinePlayer.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/entity/Player.getUniqueId, org/bukkit/entity/AnimalTamer.getUniqueId, org/bukkit/entity/Entity.getUniqueId, org/bukkit/OfflinePlayer.getUniqueId
    pub fn get_unique_id(&self) -> String {
        dashed(&self.uuid)
    }
    /// bukkit: org/bukkit/entity/Player.sendMessage, net/kyori/adventure/audience/Audience.sendMessage
    pub fn send_component(&self, text: &Component) {
        run_command(&format!("tellraw {} {}", self.name, text.0));
    }
    /// bukkit: org/bukkit/entity/Player.showTitle, net/kyori/adventure/audience/Audience.showTitle, net/kyori/adventure/title/Title.title, net/kyori/adventure/title/Title$Times.times
    pub fn show_title(&self, title: &Component, subtitle: &Component, fade_in: u32, stay: u32, fade_out: u32) {
        run_command(&format!("title {} times {fade_in} {stay} {fade_out}", self.name));
        run_command(&format!("title {} subtitle {}", self.name, subtitle.0));
        run_command(&format!("title {} title {}", self.name, title.0));
    }
    /// bukkit~: org/bukkit/entity/Player.openInventory, org/bukkit/entity/HumanEntity.openInventory | a window of `create_of_type` moves stacks as a chest does: a furnace's fuel or result slot has no rule of its own
    ///
    /// The GUI holds what's put in it, as a chest does, unless a listener cancels the click.
    pub fn open_inventory(&self, inventory: &Inventory) {
        let inventory = &inventory.live();
        if fam_items::open_typed(self, inventory) {
            return;
        }
        let answer = query(json!({ "op": "open_menu", "uuid": self.uuid, "rows": inventory.contents.len() / 9, "title": adventure::legacy(&inventory.title).0, "items": inventory.items() }));
        if !answer.is_null() {
            OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner()).insert(self.uuid.clone(), inventory.clone());
            fam_items::hold_gui(self, &answer);
        }
    }
    /// bukkit: org/bukkit/entity/Player.updateInventory
    pub fn update_inventory(&self, inventory: &Inventory) {
        query(json!({ "op": "set_menu", "uuid": self.uuid, "items": inventory.items() }));
        OPEN_INVENTORIES.lock().unwrap_or_else(|e| e.into_inner()).insert(self.uuid.clone(), inventory.clone());
    }
    /// bukkit: org/bukkit/entity/Player.closeInventory, org/bukkit/entity/HumanEntity.closeInventory
    pub fn close_inventory(&self) {
        query(json!({ "op": "close_menu", "uuid": self.uuid }));
    }
    /// bukkit: org/bukkit/entity/Player.sendTitle
    pub fn send_title(&self, title: &str, subtitle: &str, fade_in: u32, stay: u32, fade_out: u32) {
        self.show_title(&adventure::legacy(title), &adventure::legacy(subtitle), fade_in, stay, fade_out);
    }
    /// bukkit: org/bukkit/entity/Player.resetTitle
    pub fn reset_title(&self) {
        run_command(&format!("title {} reset", self.name));
    }
    /// bukkit: org/bukkit/entity/Player.sendActionBar, net/kyori/adventure/audience/Audience.sendActionBar
    pub fn send_action_bar(&self, text: &Component) {
        run_command(&format!("title {} actionbar {}", self.name, text.0));
    }
    /// bukkit: org/bukkit/entity/Player.playSound
    pub fn play_sound(&self, at: &Location, sound: &str, volume: f32, pitch: f32) {
        run_command(&format!("execute in {} run playsound {sound} master {} {} {} {} {volume} {pitch}", at.world.dimension(), self.name, at.x, at.y, at.z));
    }
    /// bukkit: org/bukkit/entity/Player.playSound
    ///
    /// With a `SoundCategory` (`player.playSound(at, sound, SoundCategory.MASTER, volume, pitch)`).
    pub fn play_sound_in(&self, at: &Location, sound: &str, category: &str, volume: f32, pitch: f32) {
        run_command(&format!("execute in {} run playsound {sound} {} {} {} {} {} {volume} {pitch}", at.world.dimension(), sound_source(category), self.name, at.x, at.y, at.z));
    }
    /// bukkit: org/bukkit/entity/Player.sendMessage, org/bukkit/command/CommandSender.sendMessage
    ///
    /// Each line of the text is its own chat message, as CraftBukkit splits it, the colours carried on.
    pub fn send_message(&self, text: &str) {
        let mut carried = String::new();
        // A text ending in a newline (a YAML `|` block) makes no empty last message.
        let text = if text.len() > 1 { text.strip_suffix('\n').unwrap_or(text) } else { text };
        for line in text.split('\n') {
            let line = format!("{carried}{line}");
            self.send_component(&adventure::legacy(&line));
            carried = fam_core::get_last_colors(&line);
        }
    }
    fn flag(&self, key: &str) -> bool {
        self.state()[key].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Player.isSneaking, org/bukkit/entity/Entity.isSneaking
    pub fn is_sneaking(&self) -> bool {
        self.flag("sneaking")
    }
    /// bukkit: org/bukkit/entity/Player.isSprinting
    pub fn is_sprinting(&self) -> bool {
        self.flag("sprinting")
    }
    /// bukkit: org/bukkit/entity/Player.isFlying
    pub fn is_flying(&self) -> bool {
        self.flag("flying")
    }
    /// bukkit: org/bukkit/entity/Player.isGliding, org/bukkit/entity/LivingEntity.isGliding
    pub fn is_gliding(&self) -> bool {
        self.flag("gliding")
    }
    /// bukkit: org/bukkit/entity/Entity.isOnGround, org/bukkit/entity/Player.isOnGround
    pub fn is_on_ground(&self) -> bool {
        self.flag("on_ground")
    }
    /// bukkit: org/bukkit/entity/Player.isSleeping, org/bukkit/entity/HumanEntity.isSleeping, org/bukkit/entity/LivingEntity.isSleeping
    pub fn is_sleeping(&self) -> bool {
        self.flag("sleeping")
    }
    /// bukkit: org/bukkit/entity/Entity.isInsideVehicle
    pub fn is_inside_vehicle(&self) -> bool {
        self.flag("in_vehicle")
    }
    /// bukkit: org/bukkit/entity/Entity.isDead, org/bukkit/entity/LivingEntity.isDead
    pub fn is_dead(&self) -> bool {
        self.flag("dead")
    }
    /// bukkit: org/bukkit/entity/Entity.getEntityId
    pub fn get_entity_id(&self) -> i64 {
        self.state()["entity_id"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/entity/Player.getFoodLevel, org/bukkit/entity/HumanEntity.getFoodLevel
    pub fn get_food_level(&self) -> i32 {
        self.state()["food"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.getSaturation, org/bukkit/entity/HumanEntity.getSaturation
    pub fn get_saturation(&self) -> f32 {
        self.state()["saturation"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Player.getLevel
    pub fn get_level(&self) -> i32 {
        self.state()["level"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.getExp
    pub fn get_exp(&self) -> f32 {
        self.state()["exp"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Player.getTotalExperience
    pub fn get_total_experience(&self) -> i32 {
        self.state()["total_exp"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.getGameMode, org/bukkit/entity/HumanEntity.getGameMode
    pub fn get_game_mode(&self) -> &'static str {
        match self.state()["game_mode"].as_i64() {
            Some(1) => "CREATIVE",
            Some(2) => "ADVENTURE",
            Some(3) => "SPECTATOR",
            _ => "SURVIVAL",
        }
    }
    /// bukkit: org/bukkit/entity/Player.getClientBrandName
    pub fn get_client_brand_name(&self) -> Option<String> {
        self.state()["brand"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Player.getAddress
    ///
    /// `get_socket_address`, as text.
    pub fn get_address(&self) -> Option<String> {
        self.get_socket_address().map(|address| address.to_string())
    }
    /// bukkit: org/bukkit/entity/Player.getClientViewDistance
    pub fn get_client_view_distance(&self) -> i32 {
        self.state()["view_distance"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.getEyeLocation, org/bukkit/entity/LivingEntity.getEyeLocation
    pub fn get_eye_location(&self) -> Location {
        let mut at = self.get_location();
        // Java's eye heights are floats, widened: 1.62 isn't 1.62 as a double.
        at.y += f64::from(if self.is_sneaking() { 1.27_f32 } else { 1.62_f32 });
        at
    }
    /// bukkit: org/bukkit/entity/Player.setHealth, org/bukkit/entity/LivingEntity.setHealth, org/bukkit/entity/Damageable.setHealth
    pub fn set_health(&self, health: f64) {
        if health <= 0.0 {
            query(json!({ "op": "ev_player_plain_kill" }));
            // At once, as `setHealth` is: what the caller says next comes after the death.
            fam_core::run_now(&format!("kill {}", self.name));
        } else {
            query(json!({ "op": "player_set", "uuid": self.uuid, "field": "health", "value": health }));
        }
    }
    /// bukkit: org/bukkit/entity/Player.addPotionEffect, org/bukkit/entity/LivingEntity.addPotionEffect
    pub fn give_potion_effect(&self, effect: &PotionEffect) -> bool {
        let request = json!({
            "op": "items_effect_add", "entity": self.get_entity_id(), "uuid": self.uuid.replace('-', ""), "dimension": self.get_world().dimension(), "id": effect.get_type(),
            "duration": effect.get_duration(), "amplifier": effect.get_amplifier(), "ambient": effect.is_ambient(), "particles": effect.has_particles(),
        });
        query(request).as_bool().unwrap_or(false)
    }
    /// `give_potion_effect`, in the words of `/effect give`.
    pub fn add_potion_effect(&self, effect: &str, seconds: u32, amplifier: u8) -> bool {
        self.give_potion_effect(&registry::create_effect(effect, i32::try_from(seconds).unwrap_or(i32::MAX / 20) * 20, i32::from(amplifier)))
    }
    /// bukkit: org/bukkit/entity/Player.removePotionEffect, org/bukkit/entity/LivingEntity.removePotionEffect
    pub fn remove_potion_effect(&self, effect: &str) {
        run_command(&format!("effect clear {} {effect}", self.name));
    }
    /// bukkit: org/bukkit/entity/Player.spawnParticle
    ///
    /// For this player alone, as `ServerLevel.sendParticles` to them. `data` is what the particle
    /// takes, and `force` is 512 blocks instead of 32. False for a particle that's unknown, or
    /// has the wrong data.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_particle_with(&self, particle: &str, at: &Location, count: i32, offset: [f64; 3], speed: f64, data: &Value, force: bool) -> bool {
        let particle = crate::enums::particle::value_of(particle).unwrap_or(particle);
        let name = if particle.contains(':') { particle.to_owned() } else { format!("minecraft:{}", particle.to_ascii_lowercase()) };
        let request = json!({
            "op": "core_particle_for", "particle": name, "data": data, "dimension": at.world.dimension(), "pos": [at.x, at.y, at.z], "offset": offset,
            "speed": [speed, speed, speed], "count": count, "force": force, "viewers": [self.uuid.replace('-', "")],
        });
        query(request).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Player.spawnParticle
    ///
    /// `spawn_particle_with` at its simplest.
    pub fn spawn_particle(&self, particle: &str, at: &Location, count: u32) {
        self.spawn_particle_with(particle, at, i32::try_from(count).unwrap_or(i32::MAX), [0.0; 3], 0.0, &Value::Null, true);
    }
    /// bukkit: org/bukkit/entity/Player.spawnParticle
    ///
    /// Spread over `dx dy dz` around `at`: Java's `spawnParticle(particle, at, count, dx, dy, dz)`.
    pub fn spawn_particle_spread(&self, particle: &str, at: &Location, count: i32, dx: f64, dy: f64, dz: f64) {
        self.spawn_particle_with(particle, at, count, [dx, dy, dz], 0.0, &Value::Null, true);
    }
    /// bukkit: org/bukkit/entity/Player.spawnParticle
    ///
    /// With `extra`, the particle's speed: Java's `spawnParticle(particle, at, count, dx, dy, dz, extra)`.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_particle_extra(&self, particle: &str, at: &Location, count: i32, dx: f64, dy: f64, dz: f64, extra: f64) {
        self.spawn_particle_with(particle, at, count, [dx, dy, dz], extra, &Value::Null, true);
    }
    /// bukkit: org/bukkit/entity/Player.getActivePotionEffects, org/bukkit/entity/LivingEntity.getActivePotionEffects
    pub fn get_active_potion_effects(&self) -> Vec<PotionEffect> {
        query(json!({ "op": "effects", "uuid": self.uuid })).as_array().into_iter().flatten().filter_map(PotionEffect::read).collect()
    }
    /// bukkit: org/bukkit/entity/Player.hasPotionEffect, org/bukkit/entity/LivingEntity.hasPotionEffect
    pub fn has_potion_effect(&self, effect: &str) -> bool {
        self.get_active_potion_effects().iter().any(|active| active.is(effect))
    }
    /// bukkit: org/bukkit/entity/Player.getPotionEffect, org/bukkit/entity/LivingEntity.getPotionEffect
    pub fn get_potion_effect(&self, effect: &str) -> Option<PotionEffect> {
        self.get_active_potion_effects().into_iter().find(|active| active.is(effect))
    }
    /// bukkit: org/bukkit/entity/Player.getDisplayName
    pub fn get_display_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/entity/Player.canSee
    pub fn can_see(&self, other: &Player) -> bool {
        query(json!({ "op": "ess_can_see", "viewer": self.uuid, "target": other.uuid })).as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/entity/Player.hidePlayer
    pub fn hide_player(&self, other: &Player) {
        if query(json!({ "op": "ess_hide", "viewer": self.uuid, "target": other.uuid, "hide": true })) == true {
            crate::fam_ev_player::visibility_changed(self, other.get_entity_id() as u64, true);
        }
    }
    /// bukkit: org/bukkit/entity/Player.showPlayer
    pub fn show_player(&self, other: &Player) {
        if query(json!({ "op": "ess_hide", "viewer": self.uuid, "target": other.uuid, "hide": false })) == true {
            crate::fam_ev_player::visibility_changed(self, other.get_entity_id() as u64, false);
        }
    }
    /// bukkit: org/bukkit/entity/Player.hidePlayer
    ///
    /// The form with the plugin that hides them (Bukkit keeps a hider per plugin; here one hiding is kept).
    pub fn hide_player_for(&self, _plugin: &Plugin, other: &Player) {
        self.hide_player(other);
    }
    /// bukkit: org/bukkit/entity/Player.showPlayer
    pub fn show_player_for(&self, _plugin: &Plugin, other: &Player) {
        self.show_player(other);
    }
    /// bukkit: org/bukkit/entity/Player.isOnline, org/bukkit/OfflinePlayer.isOnline
    pub fn is_online(&self) -> bool {
        !self.state().is_null()
    }
    /// bukkit: org/bukkit/entity/Player.getLocation, org/bukkit/entity/Entity.getLocation
    pub fn get_location(&self) -> Location {
        let state = self.state();
        let at = |i: usize| state["position"][i].as_f64().unwrap_or(0.0);
        let angle = |key: &str| state[key].as_f64().unwrap_or(0.0) as f32;
        Location::new(World::of(state["dimension"].as_str().unwrap_or("")), at(0), at(1), at(2), angle("yaw"), angle("pitch"))
    }
    /// bukkit: org/bukkit/entity/Player.getWorld, org/bukkit/entity/Entity.getWorld
    pub fn get_world(&self) -> World {
        World::of(self.state()["dimension"].as_str().unwrap_or(""))
    }
    /// bukkit~: org/bukkit/entity/Player.teleport, org/bukkit/entity/Entity.teleport | at once, but to another level as the console's goes, at the next tick's start, and at the end of the packet in the hooks that hold the hub themselves
    pub fn teleport(&self, to: &Location) -> bool {
        query(json!({ "op": "ev_player_tp_plugin" }));
        fam_core::run_now(&format!("execute in {} run tp {} {} {} {} {} {}", to.world.dimension(), self.name, to.x, to.y, to.z, to.yaw, to.pitch))
    }
    /// bukkit: org/bukkit/entity/Player.isOp, org/bukkit/permissions/ServerOperator.isOp, org/bukkit/OfflinePlayer.isOp
    pub fn is_op(&self) -> bool {
        self.state()["permission"].as_u64().unwrap_or(0) >= 2
    }
    /// bukkit: org/bukkit/entity/Player.hasPermission, org/bukkit/permissions/Permissible.hasPermission
    pub fn has_permission(&self, node: &str) -> bool {
        permitted(Some(&self.uuid), self.is_op(), node)
    }
    /// bukkit: org/bukkit/entity/Player.getInventory, org/bukkit/entity/HumanEntity.getInventory
    pub fn get_inventory(&self) -> PlayerInventory {
        PlayerInventory { player: self.clone() }
    }
    /// bukkit: org/bukkit/entity/Player.getHealth, org/bukkit/entity/LivingEntity.getHealth, org/bukkit/entity/Damageable.getHealth
    pub fn get_health(&self) -> f64 {
        self.state()["health"].as_f64().unwrap_or(0.0)
    }
    /// bukkit~: org/bukkit/entity/Player.kickPlayer, org/bukkit/entity/Player.kick | at once, but at the end of the packet in the hooks that hold the hub themselves
    pub fn kick_player(&self, reason: &str) {
        query(json!({ "op": "ev_player_kick_plugin" }));
        fam_core::run_now(&format!("kick {} {reason}", self.name));
    }
    /// bukkit: org/bukkit/entity/Player.performCommand
    ///
    /// What they'd run, at once. `PlayerCommandPreprocessEvent` isn't heard, as Java's isn't: that's for what a client sends.
    pub fn perform_command(&self, line: &str) -> bool {
        query(json!({ "op": "core_run_as_now", "uuid": self.uuid, "line": line.trim_start_matches('/') })).as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/entity/Player.setGameMode, org/bukkit/entity/HumanEntity.setGameMode
    pub fn set_game_mode(&self, mode: &str) {
        query(json!({ "op": "ev_player_mode_plugin", "name": self.name }));
        query(json!({ "op": "player_set", "uuid": self.uuid, "field": "game_mode", "value": mode.to_ascii_uppercase() }));
    }
}

fn dashed(uuid: &str) -> String {
    if uuid.len() != 32 {
        return uuid.to_owned();
    }
    format!("{}-{}-{}-{}-{}", &uuid[..8], &uuid[8..12], &uuid[12..16], &uuid[16..20], &uuid[20..])
}

/// `org.bukkit.OfflinePlayer`: someone by name, from who is on and the server's `usercache.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfflinePlayer {
    name: String,
    uuid: Option<String>,
    online: bool,
}

impl OfflinePlayer {
    /// bukkit: org/bukkit/OfflinePlayer.getUniqueId
    pub fn get_unique_id(&self) -> Option<&str> {
        self.uuid.as_deref()
    }
    /// bukkit: org/bukkit/OfflinePlayer.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/OfflinePlayer.isOnline
    pub fn is_online(&self) -> bool {
        self.online
    }
    /// bukkit: org/bukkit/OfflinePlayer.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        get_player(&self.name)
    }
    /// bukkit: org/bukkit/OfflinePlayer.hasPlayedBefore
    pub fn has_played_before(&self) -> bool {
        self.online || self.uuid.as_ref().is_some_and(|uuid| fam_core::world_folder().join(format!("players/data/{}.dat", dashed(uuid))).is_file())
    }
}

/// bukkit: org/bukkit/Bukkit.getOfflinePlayer, org/bukkit/Server.getOfflinePlayer
///
/// Someone never seen gets the offline id Bukkit gives them, with Mojang asked first on an online server.
pub fn get_offline_player(name: &str) -> OfflinePlayer {
    // `getOfflinePlayer(UUID)`: a UUID is its text here, so one shaped like an id is looked up as one, its
    // name from who is on or `usercache.json`.
    let bare: String = name.chars().filter(|c| *c != '-').collect();
    if bare.len() == 32 && bare.chars().all(|c| c.is_ascii_hexdigit()) && (name.len() == 32 || name.len() == 36) {
        let same = |other: &str| other.replace('-', "").eq_ignore_ascii_case(&bare);
        if let Some(player) = get_online_players().into_iter().find(|player| same(&player.get_unique_id())) {
            return OfflinePlayer { name: player.name.clone(), uuid: Some(player.get_unique_id()), online: true };
        }
        let cache: Value = std::fs::read("usercache.json").ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or(Value::Null);
        let known = cache.as_array().into_iter().flatten().find(|entry| entry["uuid"].as_str().is_some_and(same)).and_then(|entry| entry["name"].as_str()).unwrap_or_default();
        return OfflinePlayer { name: known.to_owned(), uuid: Some(name.to_owned()), online: false };
    }
    if let Some(player) = get_player(name) {
        return OfflinePlayer { name: player.name.clone(), uuid: Some(player.get_unique_id()), online: true };
    }
    let cache: Value = std::fs::read("usercache.json").ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or(Value::Null);
    let found = cache.as_array().into_iter().flatten().find(|entry| entry["name"].as_str().is_some_and(|known| known.eq_ignore_ascii_case(name)));
    let uuid = found.and_then(|entry| entry["uuid"].as_str()).map(str::to_owned).or_else(|| fam_core::uuid_of_stranger(name));
    OfflinePlayer { name: name.to_owned(), uuid, online: false }
}

/// `org.bukkit.boss.BossBar`, on the game's own `/bossbar`.
#[derive(Clone, Debug)]
pub struct BossBar {
    id: String,
}

static NEXT_BAR: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

impl BossBar {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }
    /// bukkit: org/bukkit/Bukkit.createBossBar, org/bukkit/Server.createBossBar, org/bukkit/boss/BarColor.valueOf
    pub fn create(title: &str, color: &str) -> Self {
        // Each plugin counts its own, and the game's bars are one list.
        let owner: String = NAME.get().map_or("plugin", String::as_str).to_ascii_lowercase().chars().map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' }).collect();
        let id = format!("horizon_bukkit:{owner}_bar{}", NEXT_BAR.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        run_command(&format!("bossbar add {id} {}", adventure::legacy(title).0));
        run_command(&format!("bossbar set {id} color {}", color.to_ascii_lowercase()));
        Self { id }
    }
    /// bukkit: org/bukkit/boss/BossBar.setTitle
    pub fn set_title(&self, title: &str) {
        run_command(&format!("bossbar set {} name {}", self.id, adventure::legacy(title).0));
    }
    /// bukkit: org/bukkit/boss/BossBar.setProgress
    pub fn set_progress(&self, progress: f64) {
        run_command(&format!("bossbar set {} value {}", self.id, (progress.clamp(0.0, 1.0) * 100.0).round() as i32));
    }
    /// bukkit: org/bukkit/boss/BossBar.addPlayer
    pub fn add_player(&self, player: &Player) {
        query(json!({ "op": "core_bossbar_player", "id": self.id, "uuid": player.uuid(), "add": true }));
    }
    /// bukkit: org/bukkit/boss/BossBar.setVisible
    pub fn set_visible(&self, visible: bool) {
        run_command(&format!("bossbar set {} visible {visible}", self.id));
    }
    /// bukkit: org/bukkit/boss/BossBar.removeAll
    pub fn remove_all(&self) {
        run_command(&format!("bossbar remove {}", self.id));
    }
}

/// bukkit: org/bukkit/Bukkit.getOnlinePlayers, org/bukkit/Server.getOnlinePlayers
pub fn get_online_players() -> Vec<Player> {
    query(json!({ "op": "players" })).as_array().into_iter().flatten().filter_map(Player::read).collect()
}

/// bukkit: org/bukkit/Bukkit.getPlayer, org/bukkit/Server.getPlayer, org/bukkit/Bukkit.getPlayerExact, org/bukkit/Server.getPlayerExact
pub fn get_player(name: &str) -> Option<Player> {
    Player::read(&query(json!({ "op": "player", "name": name })))
}

/// bukkit: org/bukkit/Bukkit.getName, org/bukkit/Server.getName
pub fn get_name() -> &'static str {
    "Horizon"
}

/// bukkit: org/bukkit/Bukkit.getConsoleSender, org/bukkit/Server.getConsoleSender
pub fn get_console_sender() -> CommandSender {
    CommandSender::Console
}

/// bukkit: org/bukkit/Bukkit.isPrimaryThread, org/bukkit/Server.isPrimaryThread
///
/// Plugin code runs on the server's thread, unless a task was started with `run_task_asynchronously`.
pub fn is_primary_thread() -> bool {
    !scheduler::in_async_task()
}

/// How many are online: Java's `getOnlinePlayers().size()`.
pub fn get_online_count() -> usize {
    get_online_players().len()
}

/// What Java's `"" + value` prints, where it differs from Rust: `20.0`, not `20`.
pub trait JavaString {
    fn java_string(&self) -> String;
}

/// `Double.toString`: plain between 10^-3 and 10^7, and `1.0E7` past them.
fn java_scientific(value: f64, plain: String) -> String {
    let size = value.abs();
    if size == 0.0 || !size.is_finite() || (0.001..10_000_000.0).contains(&size) {
        return plain;
    }
    let text = format!("{plain:e}", plain = plain.parse::<f64>().unwrap_or(value));
    let (digits, power) = text.split_once('e').unwrap_or((&text, "0"));
    let digits = if digits.contains('.') { digits.to_owned() } else { format!("{digits}.0") };
    format!("{digits}E{power}")
}

macro_rules! java_float {
    ($($kind:ty),*) => {
        $(impl JavaString for $kind {
            fn java_string(&self) -> String {
                java_scientific(f64::from(*self), format!("{self:?}"))
            }
        })*
    };
}

macro_rules! java_plain {
    ($($kind:ty),*) => {
        $(impl JavaString for $kind {
            fn java_string(&self) -> String {
                self.to_string()
            }
        })*
    };
}

java_float!(f32, f64);
java_plain!(bool, i8, i16, i32, i64, u8, u16, u32, u64, usize, String, &str);

/// `org.bukkit.Color`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    /// bukkit: org/bukkit/Color.fromRGB
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue, alpha: 255 }
    }
    /// bukkit: org/bukkit/Color.asRGB
    pub fn as_rgb(self) -> u32 {
        (u32::from(self.red) << 16) | (u32::from(self.green) << 8) | u32::from(self.blue)
    }
}

/// bukkit: org/bukkit/Bukkit.broadcastMessage, org/bukkit/Server.broadcastMessage
///
/// Who heard it: the players allowed `bukkit.broadcast.user`, and the console.
pub fn broadcast_message(text: &str) -> usize {
    let heard = get_online_players().iter().filter(|player| player.has_permission("bukkit.broadcast.user")).count();
    if crate::fam_ev_world::broadcast_goes_out(text) {
        run_command(&format!("tellraw @a {}", adventure::legacy(text).0));
    }
    log(&chat_color::strip_color(text));
    heard + 1
}

/// bukkit: org/bukkit/Bukkit.getVersion, org/bukkit/Server.getVersion
///
/// Paper's shape, `<build> (MC: <version>)`: plugins read the version out of the brackets.
pub fn get_version() -> String {
    let mc = query(json!({ "op": "version" }))["minecraft"].as_str().unwrap_or("26.3").to_owned();
    format!("{mc}-Horizon (MC: {mc})")
}

/// bukkit: org/bukkit/Bukkit.getBukkitVersion, org/bukkit/Server.getBukkitVersion
///
/// `<major>.<minor>.<patch>-R0.1-SNAPSHOT`. 26.3 has no patch number, but older plugins split this on dots
/// and read three numbers (`split[2]`), so a missing patch is given as `.0`.
pub fn get_bukkit_version() -> String {
    let mc = query(json!({ "op": "version" }))["minecraft"].as_str().unwrap_or("26.3").to_owned();
    let mc = if mc.split('.').count() < 3 { format!("{mc}.0") } else { mc };
    format!("{mc}-R0.1-SNAPSHOT")
}

/// A line as the console: `fam_core::dispatch_command_as` has Java's other argument.
pub fn dispatch_command(line: &str) -> bool {
    fam_core::dispatch_command_as(&CommandSender::Console, line)
}

// ---- Commands ----

/// `org.bukkit.command.CommandSender`.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandSender {
    Player(Player),
    Console,
    /// Any other entity, which is a sender in Java: `/execute as` a mob.
    Entity(entity::Entity),
    /// Java's `ProxiedCommandSender`: `/execute as` someone else.
    Proxied(Box<fam_core::ProxiedCommandSender>),
    /// Java's `RemoteConsoleCommandSender`: a line that came by RCON.
    Remote(fam_core::RemoteConsoleCommandSender),
}

impl CommandSender {
    /// bukkit: org/bukkit/command/CommandSender.sendMessage
    pub fn send_message(&self, text: &str) {
        match self {
            Self::Player(player) => player.send_message(text),
            Self::Console => log(text),
            Self::Entity(_) => {}
            Self::Proxied(proxied) => proxied.get_caller().send_message(text),
            Self::Remote(remote) => remote.send_message(text),
        }
    }
    /// bukkit: org/bukkit/command/CommandSender.getName
    pub fn get_name(&self) -> &str {
        match self {
            Self::Player(player) => player.get_name(),
            Self::Console => "CONSOLE",
            Self::Entity(_) => "",
            Self::Proxied(proxied) => proxied.name(),
            Self::Remote(_) => "Rcon",
        }
    }
    /// bukkit: org/bukkit/command/CommandSender.hasPermission
    pub fn has_permission(&self, node: &str) -> bool {
        match self {
            Self::Player(player) => player.has_permission(node),
            Self::Console | Self::Remote(_) => true,
            Self::Entity(_) => false,
            Self::Proxied(proxied) => proxied.get_caller().has_permission(node),
        }
    }
    /// bukkit: org/bukkit/command/CommandSender.sendMessage, net/kyori/adventure/audience/Audience.sendMessage
    pub fn send_component(&self, text: &Component) {
        match self {
            Self::Player(player) => player.send_component(text),
            Self::Console => log(&text.plain()),
            Self::Entity(_) => {}
            Self::Proxied(proxied) => proxied.get_caller().send_component(text),
            Self::Remote(remote) => remote.send_message(&text.plain()),
        }
    }
    /// Java's `sender instanceof Player`.
    pub fn as_player(&self) -> Option<&Player> {
        match self {
            Self::Player(player) => Some(player),
            Self::Console | Self::Proxied(_) | Self::Remote(_) | Self::Entity(_) => None,
        }
    }
}

/// `CommandExecutor.onCommand`: the sender, the name it was called by, and the words after it.
/// False prints the command's usage, as Bukkit does.
pub type Executor = std::sync::Arc<dyn Fn(&CommandSender, &str, &[&str]) -> bool + Send + Sync>;

struct CommandEntry {
    usage: String,
    permission: Option<String>,
    permission_message: Option<String>,
    executor: Option<Executor>,
    completer: Option<Completer>,
}

/// bukkit: org/bukkit/command/TabCompleter, org/bukkit/command/TabCompleter.onTabComplete
pub type Completer = std::sync::Arc<dyn Fn(&CommandSender, &str, &[&str]) -> Vec<String> + Send + Sync>;

static COMMANDS: OnceLock<Mutex<HashMap<String, CommandEntry>>> = OnceLock::new();
/// Every word a command answers to, and the name it has in `plugin.yml`.
static ALIASES: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

/// The `plugin.yml` name a plugin means by `getCommand(name)`: as written, else by any case or alias.
fn command_name(name: &str) -> String {
    if commands().lock().unwrap_or_else(|e| e.into_inner()).contains_key(name) {
        return name.to_owned();
    }
    ALIASES.lock().unwrap_or_else(|e| e.into_inner()).get(&name.to_lowercase()).cloned().unwrap_or_else(|| name.to_owned())
}

/// A command made in code: the game learns of the word, and it runs like one from `plugin.yml`.
pub(crate) fn add_command(host: Host, name: &str) {
    add_command_usage(host, name, &format!("{name} [args:text]"));
}

pub(crate) fn add_command_usage(host: Host, name: &str, usage: &str) {
    fam_core::command_usage(host, usage);
    ALIASES.lock().unwrap_or_else(|e| e.into_inner()).insert(name.to_lowercase(), name.to_owned());
    commands().lock().unwrap_or_else(|e| e.into_inner()).insert(name.to_owned(), CommandEntry { usage: String::new(), permission: None, permission_message: None, executor: None, completer: None });
}

fn commands() -> &'static Mutex<HashMap<String, CommandEntry>> {
    COMMANDS.get_or_init(Default::default)
}

// ---- The plugin ----

/// `org.bukkit.plugin.java.JavaPlugin`: what `plugin.yml` says, and where its files are.
#[derive(Clone, Copy)]
pub struct Plugin {
    pub(crate) host: Host,
}

static NAME: OnceLock<String> = OnceLock::new();
/// The `config.yml` the plugin ships, which `getConfig` falls back to.
static JAR_CONFIG: Mutex<Option<String>> = Mutex::new(None);
static VERSION: OnceLock<String> = OnceLock::new();
/// `plugin.yml`, as JSON.
static DESCRIPTION: OnceLock<Value> = OnceLock::new();

impl Plugin {
    /// bukkit: org/bukkit/plugin/Plugin.getName, org/bukkit/plugin/java/JavaPlugin.getName
    pub fn get_name(&self) -> &str {
        NAME.get().map_or("plugin", String::as_str)
    }
    fn described(&self, key: &str) -> Value {
        DESCRIPTION.get().map_or(Value::Null, |yaml| yaml[key].clone())
    }
    fn described_list(&self, key: &str) -> Vec<String> {
        match self.described(key) {
            Value::Array(values) => values.iter().filter_map(config::scalar).collect(),
            Value::Null => Vec::new(),
            other => config::scalar(&other).into_iter().collect(),
        }
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getDescription, org/bukkit/plugin/PluginDescriptionFile.getDescription
    pub fn get_description(&self) -> Option<String> {
        config::scalar(&self.described("description"))
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getAuthors, org/bukkit/plugin/PluginDescriptionFile.getAuthors
    pub fn get_authors(&self) -> Vec<String> {
        let mut authors = self.described_list("author");
        authors.extend(self.described_list("authors"));
        authors
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getContributors, org/bukkit/plugin/PluginDescriptionFile.getContributors
    pub fn get_contributors(&self) -> Vec<String> {
        self.described_list("contributors")
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getWebsite, org/bukkit/plugin/PluginDescriptionFile.getWebsite
    pub fn get_website(&self) -> Option<String> {
        config::scalar(&self.described("website"))
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getMainClass, org/bukkit/plugin/PluginDescriptionFile.getMain
    pub fn get_main_class(&self) -> Option<String> {
        config::scalar(&self.described("main"))
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getAPIVersion, org/bukkit/plugin/PluginDescriptionFile.getAPIVersion
    pub fn get_api_version(&self) -> Option<String> {
        config::scalar(&self.described("api-version"))
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getLoggerPrefix, org/bukkit/plugin/PluginDescriptionFile.getPrefix
    pub fn get_prefix(&self) -> Option<String> {
        config::scalar(&self.described("prefix"))
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getPluginDependencies, org/bukkit/plugin/PluginDescriptionFile.getDepend
    pub fn get_depend(&self) -> Vec<String> {
        self.described_list("depend")
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getPluginSoftDependencies, org/bukkit/plugin/PluginDescriptionFile.getSoftDepend
    pub fn get_soft_depend(&self) -> Vec<String> {
        self.described_list("softdepend")
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getLoadBeforePlugins, org/bukkit/plugin/PluginDescriptionFile.getLoadBefore
    pub fn get_load_before(&self) -> Vec<String> {
        self.described_list("loadbefore")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getCommands
    pub fn get_commands(&self) -> Value {
        self.described("commands")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getPermissions
    pub fn get_permissions(&self) -> Value {
        self.described("permissions")
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getLoad, io/papermc/paper/plugin/configuration/PluginMeta.getLoadOrder
    pub fn get_load(&self) -> String {
        config::scalar(&self.described("load")).unwrap_or_else(|| "POSTWORLD".to_owned())
    }
    /// bukkit: org/bukkit/plugin/Plugin.isEnabled
    pub fn is_enabled(&self) -> bool {
        fam_core::plugin_is_enabled()
    }
    /// bukkit: org/bukkit/plugin/Plugin.isNaggable, org/bukkit/plugin/PluginBase.isNaggable
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.isNaggable
    pub fn is_naggable(&self) -> bool {
        fam_core::plugin_is_naggable()
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getDisplayName, org/bukkit/plugin/PluginDescriptionFile.getFullName
    pub fn get_full_name(&self) -> String {
        format!("{} v{}", self.get_name(), self.get_version())
    }
    /// bukkit: org/bukkit/plugin/PluginDescriptionFile.getVersion, io/papermc/paper/plugin/configuration/PluginMeta.getVersion
    pub fn get_version(&self) -> &str {
        VERSION.get().map_or("0", String::as_str)
    }
    /// bukkit: org/bukkit/plugin/Plugin.getDataFolder, org/bukkit/plugin/java/JavaPlugin.getDataFolder
    ///
    /// `plugins/<Name>`, relative to the server's folder. Made if it isn't there, so a file can be written
    /// into it straight away.
    pub fn get_data_folder(&self) -> PathBuf {
        let folder = PathBuf::from("plugins").join(self.get_name());
        let _ = std::fs::create_dir_all(&folder);
        folder
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.getCommand, org/bukkit/command/PluginCommand.setExecutor
    ///
    /// The name as Bukkit's `getCommand` takes it: any case, or an alias (`getCommand("Spawn")` for `spawn:`).
    pub fn set_executor(&self, command: &str, executor: impl Fn(&CommandSender, &str, &[&str]) -> bool + Send + Sync + 'static) {
        let command = &command_name(command);
        if let Some(entry) = commands().lock().unwrap_or_else(|e| e.into_inner()).get_mut(command) {
            entry.executor = Some(std::sync::Arc::new(executor));
        } else {
            log(&format!("no command {command} in plugin.yml"));
        }
    }
    /// bukkit: org/bukkit/command/PluginCommand.setTabCompleter, org/bukkit/command/TabExecutor, org/bukkit/command/TabCompleter
    pub fn set_tab_completer(&self, command: &str, completer: impl Fn(&CommandSender, &str, &[&str]) -> Vec<String> + Send + Sync + 'static) {
        let command = &command_name(command);
        if let Some(entry) = commands().lock().unwrap_or_else(|e| e.into_inner()).get_mut(command) {
            entry.completer = Some(std::sync::Arc::new(completer));
        } else {
            log(&format!("no command {command} in plugin.yml"));
        }
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.saveResource, org/bukkit/plugin/Plugin.saveResource
    pub fn save_resource(&self, name: &str, text: &str) {
        let path = self.get_data_folder().join(name);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, text);
        }
    }
    /// bukkit: org/bukkit/plugin/Plugin.getLogger, org/bukkit/plugin/java/JavaPlugin.getLogger, java/util/logging/Logger.info
    pub fn info(&self, text: &str) {
        log(&format!("[{}] {text}", self.get_name()));
    }
    /// bukkit: java/util/logging/Logger.warning
    pub fn warning(&self, text: &str) {
        log(&format!("[{}] WARNING: {text}", self.get_name()));
    }
    /// bukkit: java/util/logging/Logger.severe
    pub fn severe(&self, text: &str) {
        log(&format!("[{}] SEVERE: {text}", self.get_name()));
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.getConfig, org/bukkit/plugin/Plugin.getConfig
    pub fn get_config(&self) -> config::YamlConfiguration {
        let mut config = config::YamlConfiguration::load_configuration(&self.get_data_folder().join("config.yml"));
        if let Some(defaults) = JAR_CONFIG.lock().unwrap_or_else(|e| e.into_inner()).as_deref() {
            config.set_defaults(config::parse(defaults));
        }
        config
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.saveConfig, org/bukkit/plugin/Plugin.saveConfig
    pub fn save_config(&self, config: &config::YamlConfiguration) {
        if let Err(error) = config.save(Some(&self.get_data_folder().join("config.yml"))) {
            self.severe(&format!("Could not save config.yml: {error}"));
        }
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.reloadConfig, org/bukkit/plugin/Plugin.reloadConfig
    pub fn reload_config(&self) -> config::YamlConfiguration {
        self.get_config()
    }
    /// bukkit: org/bukkit/plugin/java/JavaPlugin.saveDefaultConfig, org/bukkit/plugin/Plugin.saveDefaultConfig
    ///
    /// Java copies config.yml from the jar; here you pass its text, usually
    /// `plugin.save_default_config(include_str!("../config.yml"))` with config.yml beside plugin.yml.
    /// Written to the data folder only if no config.yml is there yet; it is also what `get_config` falls back
    /// to for a key the file lacks. `get_config()` reads the file anew each call: call it again after a reload.
    pub fn save_default_config(&self, text: &str) {
        *JAR_CONFIG.lock().unwrap_or_else(|e| e.into_inner()) = Some(text.to_owned());
        self.save_resource("config.yml", text);
    }
}

impl Plugin {
    /// bukkit: org/bukkit/scheduler/BukkitScheduler.runTask, org/bukkit/scheduler/BukkitRunnable.runTask
    ///
    /// `task` once, on the server thread, next tick. Times are in ticks: 20 a second.
    pub fn run_task(&self, task: impl FnMut() + Send + 'static) -> fam_core::BukkitTask {
        fam_core::BukkitTask::of(scheduler::run_task(task))
    }
    /// bukkit: org/bukkit/scheduler/BukkitScheduler.runTaskLater, org/bukkit/scheduler/BukkitRunnable.runTaskLater
    ///
    /// `task` once, `delay` ticks from now (20 a second), on the server thread.
    pub fn run_task_later(&self, delay: u64, task: impl FnMut() + Send + 'static) -> fam_core::BukkitTask {
        fam_core::BukkitTask::of(scheduler::run_task_later(delay, task))
    }
    /// bukkit: org/bukkit/scheduler/BukkitScheduler.runTaskTimer, org/bukkit/scheduler/BukkitRunnable.runTaskTimer
    ///
    /// `task` every `period` ticks (20 a second), the first after `delay` ticks, on the server thread, until the
    /// returned task's `cancel()` (or the plugin's disabling, which cancels all its tasks).
    pub fn run_task_timer(&self, delay: u64, period: u64, task: impl FnMut() + Send + 'static) -> fam_core::BukkitTask {
        fam_core::BukkitTask::of(scheduler::run_task_timer(delay, period, task))
    }
    /// bukkit: org/bukkit/scheduler/BukkitScheduler.runTaskAsynchronously, org/bukkit/scheduler/BukkitRunnable.runTaskAsynchronously
    ///
    /// `task` on a thread of its own: no game calls from it but the thread-safe ones (messages, files).
    pub fn run_task_asynchronously(&self, task: impl FnOnce() + Send + 'static) -> fam_core::BukkitTask {
        scheduler::run_task_asynchronously(task)
    }
}

/// What Bukkit does for a command with no completer of its own: the names of who is on.
fn default_completion(args: &[&str]) -> Vec<String> {
    if !fam_core::suggest_player_names_when_null_tab_completions() {
        return Vec::new();
    }
    let last = args.last().copied().unwrap_or_default();
    get_online_players().iter().map(|player| player.get_name().to_owned()).filter(|name| extra::starts_with_ignore_case(name, last)).collect()
}

fn run_tab_complete(event: &Value) -> Answer {
    let line = event["line"].as_str().unwrap_or_default().trim_start_matches('/');
    let mut words: Vec<&str> = line.split(' ').collect();
    if words.is_empty() {
        return Answer::Pass;
    }
    let label = words.remove(0);
    let name = ALIASES.lock().unwrap_or_else(|e| e.into_inner()).get(&label.to_lowercase()).cloned();
    let list = commands().lock().unwrap_or_else(|e| e.into_inner());
    let Some(entry) = name.and_then(|name| list.get(&name)) else {
        return Answer::Pass;
    };
    let sender = Player::read(event).map_or(CommandSender::Console, CommandSender::Player);
    if entry.permission.as_ref().is_some_and(|node| !sender.has_permission(node)) {
        return Answer::Pass;
    }
    let matches = match &entry.completer {
        Some(completer) => completer(&sender, label, &words),
        None => default_completion(&words),
    };
    query(json!({ "op": "suggest", "matches": matches }));
    Answer::Pass
}

fn run_command_event(event: &Value) -> Answer {
    let raw = event["line"].as_str().unwrap_or_default();
    // A word with a slash of its own is its own command when a plugin took it so (WorldEdit's `//set` is `/set`).
    let first = raw.split_whitespace().next().unwrap_or_default().to_lowercase();
    let line = if first.len() > 1 && first.starts_with('/') && ALIASES.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&first) {
        raw
    } else {
        raw.trim_start_matches('/')
    };
    let mut words = line.split_whitespace();
    let Some(label) = words.next() else {
        return Answer::Pass;
    };
    let args: Vec<&str> = words.collect();
    let sender = fam_core::event_sender(event).unwrap_or_else(|| Player::read(&event["player"]).map_or(CommandSender::Console, CommandSender::Player));
    let name = ALIASES.lock().unwrap_or_else(|e| e.into_inner()).get(&label.to_lowercase()).cloned();
    let list = commands().lock().unwrap_or_else(|e| e.into_inner());
    let Some(entry) = name.as_ref().and_then(|name| list.get(name)) else {
        if name.is_none() && fam_core::word_is_off(label) {
            sender.send_message(&format!("{}An unexpected error occurred while trying to execute that command", chat_color::RED));
        }
        return Answer::Pass;
    };
    if let Some(node) = &entry.permission
        && !sender.has_permission(node)
    {
        let text = entry.permission_message.as_deref().unwrap_or("I'm sorry, but you do not have permission to perform this command.");
        sender.send_message(&format!("{}{text}", chat_color::RED));
        return Answer::Pass;
    }
    let (executor, usage) = (entry.executor.clone(), entry.usage.clone());
    // Not held while it runs: a command can run another of the plugin's.
    drop(list);
    if let Some(executor) = executor
        && !executor(&sender, label, &args)
        && !usage.is_empty()
    {
        sender.send_message(&usage.replace("<command>", label));
    }
    Answer::Pass
}

/// Reads `plugin.yml`, as Bukkit does before `onEnable`: name, commands, aliases, permissions.
/// What the engine reads of a plugin before anything of it runs, as Paper's loader reads `plugin.yml`:
/// its name and version, and the load rules (`depend`, `softdepend`, `loadbefore`, `provides`), as JSON.
/// A word where a list belongs counts as a list of one, as in Bukkit.
pub fn plugin_manifest(description: &str) -> String {
    let yaml = config::parse(description);
    let list = |key: &str| -> Vec<String> {
        match &yaml[key] {
            Value::Array(items) => items.iter().filter_map(config::scalar).collect(),
            Value::Null => Vec::new(),
            other => config::scalar(other).into_iter().collect(),
        }
    };
    let mut authors = list("authors");
    if let Some(author) = config::scalar(&yaml["author"]) {
        authors.insert(0, author);
    }
    let (mut depend, mut softdepend, mut loadbefore) = (list("depend"), list("softdepend"), list("loadbefore"));
    // `paper-plugin.yml`'s `dependencies.server`: `required` (true unless it says false) and `load` (`BEFORE`: it
    // starts first, `AFTER`: this one does, `OMIT`: either), as Paper's loader reads them.
    if let Some(server) = yaml["dependencies"]["server"].as_object() {
        for (name, entry) in server {
            let required = entry["required"].as_bool().unwrap_or(true);
            let load = config::scalar(&entry["load"]).unwrap_or_else(|| "OMIT".to_owned()).to_ascii_uppercase();
            match (required, load.as_str()) {
                (_, "AFTER") => loadbefore.push(name.clone()),
                (true, _) => depend.push(name.clone()),
                (false, "BEFORE") => softdepend.push(name.clone()),
                (false, _) => {}
            }
        }
    }
    serde_json::json!({
        "name": config::scalar(&yaml["name"]),
        "version": config::scalar(&yaml["version"]),
        "api-version": config::scalar(&yaml["api-version"]),
        "description": config::scalar(&yaml["description"]),
        "authors": authors,
        "depend": depend,
        "softdepend": softdepend,
        "loadbefore": loadbefore,
        "provides": list("provides"),
    })
    .to_string()
}

pub fn load_description(host: Host, description: &str) -> Plugin {
    let yaml = config::parse(description);
    // `PluginDescriptionFile` keeps a name with `_` for each space: the data folder and every lookup use it.
    let _ = NAME.set(config::scalar(&yaml["name"]).unwrap_or_else(|| "plugin".to_owned()).replace(' ', "_"));
    let _ = VERSION.set(config::scalar(&yaml["version"]).unwrap_or_default());
    let _ = DESCRIPTION.set(yaml.clone());
    if let Some(list) = yaml["permissions"].as_object() {
        let mut defaults = PERMISSION_DEFAULTS.lock().unwrap_or_else(|e| e.into_inner());
        for (node, value) in list {
            if let Some(default) = config::scalar(&value["default"]) {
                defaults.insert(node.clone(), default.to_ascii_lowercase());
            }
        }
    }
    if let Some(list) = yaml["commands"].as_object() {
        for (name, value) in list {
            let mut names = vec![name.clone()];
            match &value["aliases"] {
                Value::String(alias) => names.push(alias.clone()),
                Value::Array(aliases) => names.extend(aliases.iter().filter_map(|alias| alias.as_str().map(str::to_owned))),
                _ => {}
            }
            for word in names {
                // Bukkit's command map keeps labels in lower case (`KaiCore` is typed `/kaicore`, any case).
                let word = word.to_lowercase();
                host.command(&format!("{word} [args:text]"), 0);
                ALIASES.lock().unwrap_or_else(|e| e.into_inner()).insert(word, name.clone());
            }
            commands().lock().unwrap_or_else(|e| e.into_inner()).insert(name.clone(), CommandEntry {
                usage: value["usage"].as_str().unwrap_or_default().to_owned(),
                permission: value["permission"].as_str().map(str::to_owned),
                permission_message: value["permission-message"].as_str().map(str::to_owned),
                executor: None,
                completer: None,
            });
        }
    }
    host.on("command", run_command_event);
    host.on("tab_complete", run_tab_complete);
    worlds::listen(host);
    host.on("chat_final", fam_ev_player::chat_final);
    scheduler::install(host);
    fam_items::install(host);
    fam_entity::install(host);
    Plugin { host }
}

/// The library's entry: `bukkit_plugin!(on_enable, on_disable);` beside `plugin.yml` and
/// `fabric.mod.json`.
/// Files a plugin is built with, by their paths in its crate: what `Plugin::get_resource` gives, as a jar's are.
#[macro_export]
macro_rules! embed_resources {
    ($plugin:expr, $($name:literal),* $(,)?) => {
        $($plugin.add_resource($name, include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $name)));)*
    };
}

impl Eq for CommandSender {}

#[macro_export]
macro_rules! bukkit_plugin {
    ($enable:path, $disable:path) => {
        fn horizon_bukkit_start(host: $crate::horizon_mod::Host) {
            let plugin = $crate::load_description(host, include_str!("../plugin.yml"));
            plugin.add_resource("plugin.yml", include_bytes!("../plugin.yml"));
            $crate::fam_core::install(plugin, |plugin| $enable(plugin), |plugin| $disable(plugin));
            // A mod's events and commands are collected while it starts: onEnable runs here, or once the
            // worlds are loaded when the engine says so (`enable_later`).
            if $crate::fam_core::starting_enabled() && !$crate::fam_core::enable_later(include_str!("../plugin.yml")) {
                $enable(plugin);
            }
        }
        $crate::horizon_mod::mod_entry!(horizon_bukkit_start, $crate::plugin_manifest(include_str!("../plugin.yml")));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plugin_yml_describes_the_plugin_as_paper_reads_it() {
        let yml = "name: Shop\nversion: 1.2\nauthor: me\nauthors: [you]\ndepend: Vault\nsoftdepend: [Essentials, LuckPerms]\nloadbefore:\n  - Other\nprovides: [Store]\n";
        let manifest: Value = serde_json::from_str(&plugin_manifest(yml)).unwrap();
        assert_eq!(manifest["name"], "Shop");
        assert_eq!(manifest["version"], "1.2");
        assert_eq!(manifest["authors"], json!(["me", "you"]));
        assert_eq!(manifest["depend"], json!(["Vault"]));
        assert_eq!(manifest["softdepend"], json!(["Essentials", "LuckPerms"]));
        assert_eq!(manifest["loadbefore"], json!(["Other"]));
        assert_eq!(manifest["provides"], json!(["Store"]));
    }

    #[test]
    fn a_paper_plugin_yml_s_dependencies_order_the_load() {
        let yml = "name: P\nversion: '1'\ndependencies:\n  server:\n    Vault:\n      load: BEFORE\n    Maps:\n      load: BEFORE\n      required: false\n    Late:\n      load: AFTER\n      required: false\n    Optional:\n      required: false\n";
        let manifest: Value = serde_json::from_str(&plugin_manifest(yml)).unwrap();
        assert_eq!(manifest["depend"], json!(["Vault"]));
        assert_eq!(manifest["softdepend"], json!(["Maps"]));
        assert_eq!(manifest["loadbefore"], json!(["Late"]));
    }

    #[test]
    fn color_codes_translate_as_bukkit_does() {
        assert_eq!(chat_color::translate_alternate_color_codes('&', "&aHi &zthere &&c!"), "\u{a7}aHi &zthere &\u{a7}c!");
        assert_eq!(chat_color::strip_color("\u{a7}aHi\u{a7}r"), "Hi");
    }

    #[test]
    fn worlds_map_to_the_three_levels() {
        assert_eq!(World::of("minecraft:the_nether").get_name(), "world_nether");
        assert_eq!(World::of("world").dimension(), "minecraft:overworld");
        assert!(get_world("nowhere").is_none());
    }

    #[test]
    fn materials_read_both_ways() {
        assert_eq!(Material::of("DIAMOND_SWORD"), Material::of("minecraft:diamond_sword"));
        assert_eq!(Material::of("minecraft:stone").name(), "STONE");
    }

    #[test]
    fn locations_floor_negative_coordinates() {
        let at = Location::new(World::of("world"), -0.5, 64.0, 1.5, 0.0, 0.0);
        assert_eq!((at.get_block_x(), at.get_block_y(), at.get_block_z()), (-1, 64, 1));
    }

    #[test]
    fn permission_defaults_follow_plugin_yml() {
        PERMISSION_DEFAULTS.lock().unwrap().insert("a.yes".into(), "true".into());
        PERMISSION_DEFAULTS.lock().unwrap().insert("a.not".into(), "not op".into());
        assert!(permitted(None, false, "a.yes"));
        assert!(permitted(None, false, "a.not") && !permitted(None, true, "a.not"));
        assert!(!permitted(None, false, "a.undeclared") && permitted(None, true, "a.undeclared"));
    }
}
