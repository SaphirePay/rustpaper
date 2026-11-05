//! Paper's own classes outside events, entities and data components: particles, registries, tags,
//! Brigadier arguments, connections, datapacks, and the small value classes. Family paper: see
//! plugin-tools/tools/family. What only Java can have is listed in `family/paper_java_only.txt`.
#![allow(missing_docs, clippy::wrong_self_convention, clippy::too_many_arguments, clippy::should_implement_trait)]
use crate::adventure::Component;
use crate::brigadier::{ArgumentType, CommandContext, CommandNode};
use crate::config2::Permission;
use crate::data::snbt;
use crate::dialog::{Dialog, DialogAction, DialogBase, DialogInput, DialogType};
use crate::entity::Entity;
use crate::extra::create_player_profile;
use crate::persistence::{NamespacedKey, PersistentDataContainer};
use crate::player::PlayerProfile;
use crate::pure::{Angle, Position, Rotation};
use crate::scheduler;
use crate::util::Vector;
use crate::{Block, Color, CommandSender, ItemStack, Location, Material, Player, Plugin, World};
use horizon_mod::{Value, query, run_command};
use serde_json::{Map, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// ---- Particles ----

/// `com.destroystokyo.paper.ParticleBuilder`.
///
/// Data is the particle's own, as JSON: `{ "color": argb }`, `{ "color": rgb, "scale": s }`,
/// `{ "from": rgb, "to": rgb, "scale": s }`, `{ "color": rgb, "power": p }`, `{ "block": "minecraft:stone" }`,
/// `{ "scale": s }`, `{ "delay": ticks }`, `{ "target": [x, y, z], "color": argb, "duration": ticks }`,
/// `{ "item": stack's to_json }`, `{ "pos": [x, y, z], "ticks": t }` or `{ "entity": id, "offset": y, "ticks": t }`,
/// `{ "water": blocks }` or `{ "water": blocks, "impulse": f }`.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleBuilder {
    particle: String,
    receivers: Option<Vec<Player>>,
    source: Option<Player>,
    location: Option<Location>,
    count: i32,
    offset: [f64; 3],
    speed: [f64; 3],
    data: Option<Value>,
    force: bool,
    randomization: &'static str,
}

fn particle_key(name: &str) -> String {
    if name.contains(':') { name.to_owned() } else { format!("minecraft:{}", name.to_ascii_lowercase()) }
}

/// The types Java refuses without data, as the packet would be wrong.
fn particle_needs_data(particle: &str) -> bool {
    matches!(particle.trim_start_matches("minecraft:"), "block" | "block_marker" | "falling_dust" | "dust_pillar" | "block_crumble" | "entity_effect" | "tinted_leaves" | "flash" | "dust" | "dust_color_transition" | "effect" | "instant_effect" | "dragon_breath" | "sculk_charge" | "shriek" | "trail" | "item" | "vibration" | "geyser" | "geyser_plume" | "geyser_base" | "geyser_poof")
}

impl ParticleBuilder {
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.<init>, org/bukkit/Particle.builder
    pub fn new(particle: &str) -> Self {
        Self { particle: particle_key(particle), receivers: None, source: None, location: None, count: 1, offset: [0.0; 3], speed: [1.0; 3], data: None, force: true, randomization: "DEFAULT" }
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.spawn
    ///
    /// # Panics
    /// As Java throws, with no location set.
    pub fn spawn(&self) -> &Self {
        let at = self.location.as_ref().expect("Please specify location for this particle");
        assert!(self.data.is_some() || !particle_needs_data(&self.particle), "missing required data for {}", self.particle);
        let world = at.get_world();
        let mut viewers = self.receivers.clone().unwrap_or_else(|| world.get_players());
        if let Some(source) = &self.source {
            viewers.retain(|viewer| source.can_see(viewer));
        }
        let randomization = match self.randomization {
            "ALTERNATIVE" => 1,
            "ALTERNATIVE_WITH_SPEED" => 2,
            _ => 0,
        };
        let uuids: Vec<&str> = viewers.iter().map(Player::uuid).collect();
        let sent = query(json!({ "op": "paper_particle", "particle": self.particle, "data": self.data, "dimension": world.dimension(), "pos": [at.get_x(), at.get_y(), at.get_z()], "offset": self.offset, "speed": self.speed, "count": self.count, "force": self.force, "randomization": randomization, "viewers": uuids }));
        // Where the game is lent out to a guest's turn, `/particle` still reaches them at the tick's end.
        if sent.as_bool() != Some(true) {
            let mode = if self.force { "force" } else { "normal" };
            for viewer in &viewers {
                run_command(&format!("execute in {} run particle {} {} {} {} {} {} {} {} {} {mode} {}", world.dimension(), self.command_spec(), at.get_x(), at.get_y(), at.get_z(), self.offset[0], self.offset[1], self.offset[2], self.speed[0], self.count, viewer.get_name()));
            }
        }
        self
    }
    fn command_spec(&self) -> String {
        let channel = |rgb: i64, shift: i64| ((rgb >> shift) & 255) as f64 / 255.0;
        let list = |rgb: i64| format!("[{:?}f,{:?}f,{:?}f]", channel(rgb, 16), channel(rgb, 8), channel(rgb, 0));
        let Some(data) = &self.data else {
            return self.particle.clone();
        };
        if let (Some(from), Some(to)) = (data["from"].as_i64(), data["to"].as_i64()) {
            return format!("{}{{from_color:{},to_color:{},scale:{:?}f}}", self.particle, list(from), list(to), data["scale"].as_f64().unwrap_or(1.0));
        }
        if let Some(block) = data["block"].as_str() {
            return format!("{}{{block_state:{}}}", self.particle, crate::data::quoted(block));
        }
        if let (Some(color), Some(scale)) = (data["color"].as_i64(), data["scale"].as_f64()) {
            return format!("{}{{color:{},scale:{scale:?}f}}", self.particle, list(color));
        }
        if let Some(color) = data["color"].as_i64() {
            return format!("{}{{color:{color}}}", self.particle);
        }
        self.particle.clone()
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.particle
    pub fn particle(&self) -> &str {
        &self.particle
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.particle
    pub fn with_particle(mut self, particle: &str) -> Self {
        self.particle = particle_key(particle);
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.receivers
    pub fn receivers(&self) -> Option<&[Player]> {
        self.receivers.as_deref()
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.hasReceivers
    pub fn has_receivers(&self) -> bool {
        match &self.receivers {
            Some(list) => !list.is_empty(),
            None => self.location.as_ref().is_some_and(|at| !at.get_world().get_players().is_empty()),
        }
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.allPlayers
    pub fn all_players(mut self) -> Self {
        self.receivers = None;
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.receivers
    pub fn with_receivers(mut self, receivers: Option<Vec<Player>>) -> Self {
        self.receivers = receivers;
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.receivers
    ///
    /// Those in a box around the location, as `receivers(x, y, z)`.
    ///
    /// # Panics
    /// As Java throws, with no location set.
    pub fn receivers_near(mut self, x: f64, y: f64, z: f64) -> Self {
        let at = self.location.clone().expect("Please set location first");
        let inside = |other: &Location| (other.x - at.x).abs() <= x && (other.y - at.y).abs() <= y && (other.z - at.z).abs() <= z;
        self.receivers = Some(at.get_world().get_players().into_iter().filter(|player| inside(&player.get_location())).collect());
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.receivers
    ///
    /// Those within a distance of the location, as `receivers(radius, true)`.
    ///
    /// # Panics
    /// As Java throws, with no location set.
    pub fn receivers_within(mut self, radius: f64) -> Self {
        let at = self.location.clone().expect("Please set location first");
        self.receivers = Some(at.get_world().get_players().into_iter().filter(|player| player.get_location().distance(&at) <= radius).collect());
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.source
    pub fn source(&self) -> Option<&Player> {
        self.source.as_ref()
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.source
    pub fn with_source(mut self, source: Option<Player>) -> Self {
        self.source = source;
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.location
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.location
    pub fn with_location(mut self, location: &Location) -> Self {
        self.location = Some(location.clone());
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.count
    pub fn count(&self) -> i32 {
        self.count
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.count
    pub fn with_count(mut self, count: i32) -> Self {
        self.count = count;
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.offsetX
    pub fn offset_x(&self) -> f64 {
        self.offset[0]
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.offsetY
    pub fn offset_y(&self) -> f64 {
        self.offset[1]
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.offsetZ
    pub fn offset_z(&self) -> f64 {
        self.offset[2]
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.offset
    pub fn offset(mut self, x: f64, y: f64, z: f64) -> Self {
        self.offset = [x, y, z];
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.extra, com/destroystokyo/paper/ParticleBuilder.speedX
    pub fn extra(&self) -> f64 {
        self.speed[0]
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.speedY
    pub fn speed_y(&self) -> f64 {
        self.speed[1]
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.speedZ
    pub fn speed_z(&self) -> f64 {
        self.speed[2]
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.speed, com/destroystokyo/paper/ParticleBuilder.extra
    pub fn speed(mut self, x: f64, y: f64, z: f64) -> Self {
        self.speed = [x, y, z];
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.data
    pub fn data(&self) -> Option<&Value> {
        self.data.as_ref()
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.data
    pub fn with_data(mut self, data: Option<Value>) -> Self {
        self.data = data;
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.force
    pub fn force(&self) -> bool {
        self.force
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.force
    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.randomizationType
    pub fn randomization_type(&self) -> &'static str {
        self.randomization
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.randomizationType
    pub fn with_randomization_type(mut self, kind: &'static str) -> Self {
        self.randomization = kind;
        self
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.color
    pub fn color(self, color: Option<Color>) -> Self {
        self.color_sized(color, 1.0)
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.color
    pub fn color_sized(self, color: Option<Color>, value: f32) -> Self {
        let Some(color) = color else {
            return self.with_data(None);
        };
        let rgb = color.as_rgb();
        let data = match self.particle.as_str() {
            "minecraft:dust" => json!({ "color": rgb, "scale": value }),
            "minecraft:effect" | "minecraft:instant_effect" => json!({ "color": rgb, "power": value }),
            _ => json!({ "color": 0xFF00_0000_u32 | rgb }),
        };
        self.with_data(Some(data))
    }
    /// bukkit: com/destroystokyo/paper/ParticleBuilder.colorTransition
    pub fn color_transition(self, from: Color, to: Color, size: f32) -> Self {
        self.with_data(Some(json!({ "from": from.as_rgb(), "to": to.as_rgb(), "scale": size })))
    }
}

// ---- Registry keys ----

/// `io.papermc.paper.registry.RegistryKey`: a registry by the game's own path, as `worldgen/biome`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegistryKey(pub &'static str);

macro_rules! registry_keys {
    ($($name:ident = $path:literal,)*) => {
        impl RegistryKey {
            $(pub const $name: Self = Self($path);)*
        }
    };
}

registry_keys! {
    GAME_EVENT = "game_event", STRUCTURE_TYPE = "worldgen/structure_type", MOB_EFFECT = "mob_effect", BLOCK = "block", ITEM = "item",
    VILLAGER_PROFESSION = "villager_profession", POINT_OF_INTEREST_TYPE = "point_of_interest_type", VILLAGER_TYPE = "villager_type",
    MAP_DECORATION_TYPE = "map_decoration_type", MENU = "menu", ATTRIBUTE = "attribute", FLUID = "fluid", SOUND_EVENT = "sound_event",
    DATA_COMPONENT_TYPE = "data_component_type", GAME_RULE = "game_rule", BIOME = "worldgen/biome", STRUCTURE = "worldgen/structure",
    TRIM_MATERIAL = "trim_material", TRIM_PATTERN = "trim_pattern", DAMAGE_TYPE = "damage_type", WOLF_VARIANT = "wolf_variant",
    WOLF_SOUND_VARIANT = "wolf_sound_variant", ENCHANTMENT = "enchantment", JUKEBOX_SONG = "jukebox_song", BANNER_PATTERN = "banner_pattern",
    DECORATED_POT_PATTERN = "decorated_pot_pattern", PAINTING_VARIANT = "painting_variant", INSTRUMENT = "instrument", CAT_VARIANT = "cat_variant",
    CAT_SOUND_VARIANT = "cat_sound_variant", FROG_VARIANT = "frog_variant", CHICKEN_VARIANT = "chicken_variant",
    CHICKEN_SOUND_VARIANT = "chicken_sound_variant", COW_VARIANT = "cow_variant", COW_SOUND_VARIANT = "cow_sound_variant", PIG_VARIANT = "pig_variant",
    PIG_SOUND_VARIANT = "pig_sound_variant", ZOMBIE_NAUTILUS_VARIANT = "zombie_nautilus_variant", SULFUR_CUBE_ARCHETYPE = "sulfur_cube_archetype",
    DIALOG = "dialog", ENTITY_TYPE = "entity_type", PARTICLE_TYPE = "particle_type", POTION = "potion", MEMORY_MODULE_TYPE = "memory_module_type",
}

impl RegistryKey {
    /// bukkit: io/papermc/paper/registry/RegistryKey.typedKey, io/papermc/paper/registry/TypedKey.create, io/papermc/paper/registry/keys/BannerPatternKeys.create, io/papermc/paper/registry/keys/BiomeKeys.create, io/papermc/paper/registry/keys/CatSoundVariantKeys.create, io/papermc/paper/registry/keys/CatVariantKeys.create, io/papermc/paper/registry/keys/ChickenSoundVariantKeys.create, io/papermc/paper/registry/keys/ChickenVariantKeys.create, io/papermc/paper/registry/keys/CowSoundVariantKeys.create, io/papermc/paper/registry/keys/CowVariantKeys.create, io/papermc/paper/registry/keys/DamageTypeKeys.create, io/papermc/paper/registry/keys/DecoratedPotPatternKeys.create, io/papermc/paper/registry/keys/DialogKeys.create, io/papermc/paper/registry/keys/EnchantmentKeys.create, io/papermc/paper/registry/keys/FrogVariantKeys.create, io/papermc/paper/registry/keys/GameEventKeys.create, io/papermc/paper/registry/keys/InstrumentKeys.create, io/papermc/paper/registry/keys/JukeboxSongKeys.create, io/papermc/paper/registry/keys/PaintingVariantKeys.create, io/papermc/paper/registry/keys/PigSoundVariantKeys.create, io/papermc/paper/registry/keys/PigVariantKeys.create, io/papermc/paper/registry/keys/StructureKeys.create, io/papermc/paper/registry/keys/SulfurCubeArchetypeKeys.create, io/papermc/paper/registry/keys/TrimMaterialKeys.create, io/papermc/paper/registry/keys/TrimPatternKeys.create, io/papermc/paper/registry/keys/WolfSoundVariantKeys.create, io/papermc/paper/registry/keys/WolfVariantKeys.create, io/papermc/paper/registry/keys/ZombieNautilusVariantKeys.create
    pub fn typed_key(self, key: &NamespacedKey) -> RegistryTypedKey {
        RegistryTypedKey { registry: self, key: key.clone() }
    }
    /// bukkit: io/papermc/paper/registry/RegistryKey.tagKey, io/papermc/paper/registry/tag/TagKey.create, io/papermc/paper/registry/keys/tags/BannerPatternTagKeys.create, io/papermc/paper/registry/keys/tags/BiomeTagKeys.create, io/papermc/paper/registry/keys/tags/BlockTypeTagKeys.create, io/papermc/paper/registry/keys/tags/DamageTypeTagKeys.create, io/papermc/paper/registry/keys/tags/DialogTagKeys.create, io/papermc/paper/registry/keys/tags/EnchantmentTagKeys.create, io/papermc/paper/registry/keys/tags/EntityTypeTagKeys.create, io/papermc/paper/registry/keys/tags/FluidTagKeys.create, io/papermc/paper/registry/keys/tags/GameEventTagKeys.create, io/papermc/paper/registry/keys/tags/InstrumentTagKeys.create, io/papermc/paper/registry/keys/tags/ItemTypeTagKeys.create, io/papermc/paper/registry/keys/tags/PaintingVariantTagKeys.create, io/papermc/paper/registry/keys/tags/PoiTypeTagKeys.create, io/papermc/paper/registry/keys/tags/PotionTypeTagKeys.create, io/papermc/paper/registry/keys/tags/StructureTagKeys.create
    pub fn tag_key(self, key: &NamespacedKey) -> TagKey {
        TagKey { registry: self, key: key.clone() }
    }
    /// The game's folder for this registry's tags.
    fn tag_folder(self) -> &'static str {
        self.0
    }
}

/// `io.papermc.paper.registry.TypedKey`: a key and the registry it is in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RegistryTypedKey {
    registry: RegistryKey,
    key: NamespacedKey,
}

impl RegistryTypedKey {
    /// bukkit: io/papermc/paper/registry/TypedKey.key, io/papermc/paper/registry/holder/RegistryHolder$Reference.key
    pub fn key(&self) -> &NamespacedKey {
        &self.key
    }
    /// bukkit: io/papermc/paper/registry/TypedKey.registryKey
    pub fn registry_key(&self) -> RegistryKey {
        self.registry
    }
}

impl From<RegistryTypedKey> for Value {
    fn from(key: RegistryTypedKey) -> Self {
        json!(key.key.as_str())
    }
}

impl From<NamespacedKey> for Value {
    fn from(key: NamespacedKey) -> Self {
        json!(key.as_str())
    }
}

impl From<Component> for Value {
    fn from(component: Component) -> Self {
        component.0
    }
}

/// `io.papermc.paper.registry.tag.TagKey`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagKey {
    registry: RegistryKey,
    key: NamespacedKey,
}

impl TagKey {
    /// bukkit: io/papermc/paper/registry/tag/TagKey.registryKey
    pub fn registry_key(&self) -> RegistryKey {
        self.registry
    }
    pub fn key(&self) -> &NamespacedKey {
        &self.key
    }
    /// The tag as the game holds it: `io.papermc.paper.registry.tag.Tag`.
    pub fn resolve(&self) -> RegistryTag {
        let names = query(json!({ "op": "tag", "kind": self.registry.tag_folder(), "name": self.key.as_str() }));
        let keys = names.as_array().into_iter().flatten().filter_map(|name| NamespacedKey::from_string(name.as_str()?)).map(|key| self.registry.typed_key(&key)).collect();
        RegistryTag { key: self.clone(), set: RegistryKeySet { registry: self.registry, keys } }
    }
}

impl From<TagKey> for Value {
    fn from(key: TagKey) -> Self {
        json!(format!("#{}", key.key.as_str()))
    }
}

/// `io.papermc.paper.registry.tag.Tag`: a tag and what is in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryTag {
    key: TagKey,
    pub set: RegistryKeySet,
}

impl RegistryTag {
    /// bukkit: io/papermc/paper/registry/tag/Tag.tagKey
    pub fn tag_key(&self) -> &TagKey {
        &self.key
    }
}

/// bukkit: io/papermc/paper/registry/RegistryAccess.registryAccess
pub fn registry_access() -> RegistryAccess {
    RegistryAccess
}

/// `io.papermc.paper.registry.RegistryAccess`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegistryAccess;

impl RegistryAccess {
    /// bukkit: io/papermc/paper/registry/RegistryAccess.getRegistry
    pub fn get_registry(&self, key: RegistryKey) -> GameRegistry {
        GameRegistry(key)
    }
}

/// `org.bukkit.Registry` as `RegistryAccess` gives it: any of the game's registries, built in or from data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameRegistry(pub RegistryKey);

impl GameRegistry {
    /// Every key, each as `minecraft:stone`.
    pub fn keys(&self) -> Vec<String> {
        query(json!({ "op": "paper_registry", "registry": self.0.0 })).as_array().into_iter().flatten().filter_map(|key| key.as_str().map(str::to_owned)).collect()
    }
    pub fn get(&self, key: &NamespacedKey) -> Option<String> {
        self.keys().into_iter().find(|known| known == key.as_str())
    }
    /// # Panics
    /// As Java throws, for a key that isn't there.
    pub fn get_or_throw(&self, key: &NamespacedKey) -> String {
        self.get(key).unwrap_or_else(|| panic!("no {} in {}", key.as_str(), self.0.0))
    }
    pub fn size(&self) -> usize {
        self.keys().len()
    }
    /// The file behind an entry, for the registries the game reads from data.
    pub fn entry(&self, key: &NamespacedKey) -> Value {
        entry_file(self.0, key)
    }
}

fn entry_file(registry: RegistryKey, key: &NamespacedKey) -> Value {
    if registry == RegistryKey::GAME_EVENT {
        // `GameEvent.DEFAULT_NOTIFICATION_RADIUS`, and the few the game gives another.
        let range = match key.as_str() {
            "minecraft:jukebox_play" | "minecraft:jukebox_stop_play" => 10,
            "minecraft:shriek" => 32,
            _ => 16,
        };
        return json!({ "range": range });
    }
    if registry == RegistryKey::SOUND_EVENT {
        return json!({ "sound_id": key.as_str() });
    }
    if key.get_namespace() == "minecraft" {
        let path = format!("{}/{}", registry.0, key.get_key());
        let entry = query(json!({ "op": "ev_world_free_data_file", "path": path }));
        return if entry.is_null() { query(json!({ "op": "data_file", "path": path })) } else { entry };
    }
    let file = crate::fam_core::world_folder().join("datapacks/plugin-registry/data").join(key.get_namespace()).join(registry.0).join(format!("{}.json", key.get_key()));
    std::fs::read(file).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or(Value::Null)
}

/// `io.papermc.paper.registry.Reference`: an entry by its key, looked up when it's wanted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    registry: GameRegistry,
    key: NamespacedKey,
}

impl Reference {
    /// bukkit: io/papermc/paper/registry/Reference.create
    pub fn create(registry: GameRegistry, key: &NamespacedKey) -> Self {
        Self { registry, key: key.clone() }
    }
    /// bukkit: io/papermc/paper/registry/Reference.value
    ///
    /// # Panics
    /// As Java throws, for an entry that isn't there.
    pub fn value(&self) -> String {
        self.registry.get_or_throw(&self.key)
    }
    /// bukkit: io/papermc/paper/registry/Reference.valueOrNull
    pub fn value_or_null(&self) -> Option<String> {
        self.registry.get(&self.key)
    }
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
}

/// `io.papermc.paper.registry.set.RegistryKeySet`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryKeySet {
    registry: RegistryKey,
    keys: Vec<RegistryTypedKey>,
}

impl RegistryKeySet {
    /// bukkit: io/papermc/paper/registry/set/RegistrySet.keySet
    pub fn key_set(registry: RegistryKey, keys: &[RegistryTypedKey]) -> Self {
        Self { registry, keys: keys.to_vec() }
    }
    /// bukkit: io/papermc/paper/registry/set/RegistrySet.keySetFromValues
    pub fn key_set_from_values(registry: RegistryKey, values: &[NamespacedKey]) -> Self {
        Self { registry, keys: values.iter().map(|key| registry.typed_key(key)).collect() }
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryKeySet.values
    pub fn values(&self) -> &[RegistryTypedKey] {
        &self.keys
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryKeySet.iterator
    pub fn iter(&self) -> std::slice::Iter<'_, RegistryTypedKey> {
        self.keys.iter()
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryKeySet.resolve
    pub fn resolve(&self, registry: &GameRegistry) -> Vec<String> {
        let known = registry.keys();
        self.keys.iter().filter(|key| known.iter().any(|name| name == key.key.as_str())).map(|key| key.key.as_str().to_owned()).collect()
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryKeySet.contains
    pub fn contains(&self, key: &RegistryTypedKey) -> bool {
        self.keys.contains(key)
    }
    /// bukkit: io/papermc/paper/registry/set/RegistrySet.size, io/papermc/paper/registry/set/RegistryKeySet.size
    pub fn size(&self) -> usize {
        self.keys.len()
    }
    /// bukkit: io/papermc/paper/registry/set/RegistrySet.isEmpty
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
    /// bukkit: io/papermc/paper/registry/set/RegistrySet.registryKey
    pub fn registry_key(&self) -> RegistryKey {
        self.registry
    }
    /// As data files write a list of a registry's entries.
    fn to_json(&self) -> Value {
        Value::Array(self.keys.iter().map(|key| json!(key.key.as_str())).collect())
    }
}

impl From<RegistryKeySet> for Value {
    fn from(set: RegistryKeySet) -> Self {
        set.to_json()
    }
}

/// A tag in a data file is `#` and its key.
impl From<RegistryTag> for Value {
    fn from(tag: RegistryTag) -> Self {
        tag.key.into()
    }
}

/// `io.papermc.paper.registry.set.RegistryValueSet`: entries written out, not named.
#[derive(Clone, Debug, PartialEq)]
pub struct RegistryValueSet<T> {
    registry: RegistryKey,
    values: Vec<T>,
}

impl<T: Clone> RegistryValueSet<T> {
    /// bukkit: io/papermc/paper/registry/set/RegistrySet.valueSet
    pub fn value_set(registry: RegistryKey, values: &[T]) -> Self {
        Self { registry, values: values.to_vec() }
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryValueSet.values
    pub fn values(&self) -> &[T] {
        &self.values
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryValueSet.iterator
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.values.iter()
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryValueSet.size
    pub fn size(&self) -> usize {
        self.values.len()
    }
    pub fn registry_key(&self) -> RegistryKey {
        self.registry
    }
}

/// `io.papermc.paper.registry.RegistryBuilderFactory`: where an entry's builder comes from.
pub struct RegistryBuilderFactory<B> {
    registry: RegistryKey,
    make: fn(Map<String, Value>) -> B,
}

impl<B> RegistryBuilderFactory<B> {
    pub fn new(registry: RegistryKey, make: fn(Map<String, Value>) -> B) -> Self {
        Self { registry, make }
    }
    /// bukkit: io/papermc/paper/registry/RegistryBuilderFactory.empty
    pub fn empty(&self) -> B {
        (self.make)(Map::new())
    }
    /// bukkit: io/papermc/paper/registry/RegistryBuilderFactory.copyFrom
    pub fn copy_from(&self, key: &RegistryTypedKey) -> B {
        let entry = entry_file(self.registry, key.key());
        (self.make)(entry.as_object().cloned().unwrap_or_default())
    }
}

/// `io.papermc.paper.registry.set.RegistryValueSetBuilder`.
pub struct RegistryValueSetBuilder<B> {
    factory: RegistryBuilderFactory<B>,
    values: Vec<B>,
}

impl<B: Clone> RegistryValueSetBuilder<B> {
    pub fn new(factory: RegistryBuilderFactory<B>) -> Self {
        Self { factory, values: Vec::new() }
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryValueSetBuilder.add
    pub fn add(mut self, build: impl FnOnce(&RegistryBuilderFactory<B>) -> B) -> Self {
        let value = build(&self.factory);
        self.values.push(value);
        self
    }
    /// bukkit: io/papermc/paper/registry/set/RegistryValueSetBuilder.build
    pub fn build(self) -> RegistryValueSet<B> {
        RegistryValueSet { registry: self.factory.registry, values: self.values }
    }
}

/// `io.papermc.paper.registry.holder.RegistryHolder`: an entry by key, or written out in place.
#[derive(Clone, Debug, PartialEq)]
pub enum RegistryHolder<E> {
    Reference(RegistryTypedKey),
    Inlined(E),
}

impl<E> RegistryHolder<E> {
    /// bukkit: io/papermc/paper/registry/holder/RegistryHolder$Inlined.entry
    pub fn entry(&self) -> Option<&E> {
        match self {
            Self::Inlined(entry) => Some(entry),
            Self::Reference(_) => None,
        }
    }
    pub fn key(&self) -> Option<&RegistryTypedKey> {
        match self {
            Self::Reference(key) => Some(key),
            Self::Inlined(_) => None,
        }
    }
}

// ---- The server's build ----

/// `io.papermc.paper.ServerBuildInfo`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ServerBuildInfo;

/// bukkit: io/papermc/paper/ServerBuildInfo.buildInfo
pub fn server_build_info() -> ServerBuildInfo {
    ServerBuildInfo
}

impl ServerBuildInfo {
    /// bukkit: io/papermc/paper/ServerBuildInfo.brandId
    pub fn brand_id(&self) -> NamespacedKey {
        NamespacedKey::from_string("horizon:horizon").expect("a valid key")
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.brandName
    pub fn brand_name(&self) -> &'static str {
        crate::get_name()
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.isBrandCompatible
    ///
    /// Horizon's own brand, and Paper's: this API is what a check for Paper is after.
    pub fn is_brand_compatible(&self, brand: &NamespacedKey) -> bool {
        *brand == self.brand_id() || brand.as_str() == "papermc:paper"
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.minecraftVersionId
    pub fn minecraft_version_id(&self) -> String {
        crate::server::get_minecraft_version()
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.minecraftVersionName
    pub fn minecraft_version_name(&self) -> String {
        crate::server::get_minecraft_version()
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.buildNumber
    ///
    /// Horizon's builds aren't numbered, which Java's empty `OptionalInt` says.
    pub fn build_number(&self) -> Option<i32> {
        None
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.buildTime
    ///
    /// When the sources of this build were put together.
    pub fn build_time(&self) -> Option<SystemTime> {
        let millis = query(json!({ "op": "paper_free_build_time" })).as_u64()?;
        Some(SystemTime::UNIX_EPOCH + Duration::from_millis(millis))
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.gitBranch
    ///
    /// A build doesn't record its branch, as Java's empty `Optional` allows.
    pub fn git_branch(&self) -> Option<String> {
        None
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.gitCommit
    ///
    /// A build doesn't record its commit, as Java's empty `Optional` allows.
    pub fn git_commit(&self) -> Option<String> {
        None
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo.asString
    ///
    /// `VERSION_SIMPLE` or `VERSION_FULL`.
    pub fn as_string(&self, representation: &str) -> String {
        match representation {
            "VERSION_FULL" => format!("{} {} ({})", self.brand_name(), self.minecraft_version_id(), self.brand_id().as_str()),
            _ => format!("{} {}", self.brand_name(), self.minecraft_version_id()),
        }
    }
}

/// `io.papermc.paper.configuration.ServerConfiguration`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ServerConfiguration;

impl ServerConfiguration {
    /// bukkit: io/papermc/paper/configuration/ServerConfiguration.isProxyEnabled
    ///
    /// Velocity's forwarding: what `forwarding.secret` turns on, on a server not in online mode.
    pub fn is_proxy_enabled(&self) -> bool {
        let secret = std::fs::read(crate::server::server_folder().join("forwarding.secret")).unwrap_or_default();
        !crate::server::get_online_mode() && !secret.trim_ascii().is_empty()
    }
    /// bukkit: io/papermc/paper/configuration/ServerConfiguration.isProxyOnlineMode
    ///
    /// A proxy with a forwarding secret authenticates its players.
    pub fn is_proxy_online_mode(&self) -> bool {
        crate::server::get_online_mode() || self.is_proxy_enabled()
    }
}

// ---- Dialogs ----

/// `io.papermc.paper.dialog.DialogResponseView`: what a player filled in, from the payload of their click.
#[derive(Clone, Debug, PartialEq)]
pub struct DialogResponseView(pub Value);

impl DialogResponseView {
    /// bukkit: io/papermc/paper/dialog/DialogResponseView.payload
    ///
    /// As SNBT, as Java's `BinaryTagHolder` holds it.
    pub fn payload(&self) -> String {
        snbt(&self.0)
    }
    /// bukkit: io/papermc/paper/dialog/DialogResponseView.getText
    pub fn get_text(&self, key: &str) -> Option<String> {
        self.0.get(key)?.as_str().map(str::to_owned)
    }
    /// bukkit: io/papermc/paper/dialog/DialogResponseView.getBoolean
    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        let value = self.0.get(key)?;
        value.as_bool().or_else(|| value.as_i64().filter(|number| matches!(number, 0 | 1)).map(|number| number == 1))
    }
    /// bukkit: io/papermc/paper/dialog/DialogResponseView.getFloat
    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.0.get(key)?.as_f64().map(|number| number as f32)
    }
}

/// `io.papermc.paper.registry.data.dialog.DialogRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DialogBuilder {
    fields: Map<String, Value>,
}

impl DialogBuilder {
    pub fn base(mut self, base: DialogBase) -> Self {
        self.fields.extend(base.0);
        self
    }
    pub fn kind(mut self, kind: DialogType) -> Self {
        if let Value::Object(fields) = kind.0 {
            self.fields.extend(fields);
        }
        self
    }
}

impl Dialog {
    /// bukkit: io/papermc/paper/dialog/Dialog.create
    pub fn build(make: impl FnOnce(&RegistryBuilderFactory<DialogBuilder>) -> DialogBuilder) -> Self {
        let factory = RegistryBuilderFactory::new(RegistryKey::DIALOG, |fields| DialogBuilder { fields });
        Self(Value::Object(make(&factory).fields))
    }
    /// bukkit: io/papermc/paper/dialog/Dialog.getKey, io/papermc/paper/dialog/Dialog.key
    ///
    /// The key of a dialog from the registry, by its contents, or `None` for one made in place.
    pub fn get_key(&self) -> Option<NamespacedKey> {
        let registry = GameRegistry(RegistryKey::DIALOG);
        registry.keys().iter().filter_map(|key| NamespacedKey::from_string(key)).find(|key| registry.entry(key) == self.0)
    }
}

impl DialogInput {
    /// bukkit: io/papermc/paper/registry/data/dialog/input/DialogInput.key
    pub fn key(&self) -> &str {
        self.0["key"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/BooleanDialogInput.onTrue
    pub fn get_on_true(&self) -> &str {
        self.0["on_true"].as_str().unwrap_or("true")
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/BooleanDialogInput.onFalse
    pub fn get_on_false(&self) -> &str {
        self.0["on_false"].as_str().unwrap_or("false")
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput.width, io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput.width, io/papermc/paper/registry/data/dialog/input/TextDialogInput.width
    pub fn get_width(&self) -> i64 {
        self.0["width"].as_i64().unwrap_or(200)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput.start
    pub fn get_start(&self) -> f32 {
        self.0["start"].as_f64().unwrap_or_default() as f32
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput.end
    pub fn get_end(&self) -> f32 {
        self.0["end"].as_f64().unwrap_or_default() as f32
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput.labelFormat
    pub fn get_label_format(&self) -> &str {
        self.0["label_format"].as_str().unwrap_or("options.generic_value")
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput.step
    pub fn get_step(&self) -> Option<f32> {
        self.0["step"].as_f64().map(|step| step as f32)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput.labelVisible, io/papermc/paper/registry/data/dialog/input/TextDialogInput.labelVisible
    pub fn get_label_visible(&self) -> bool {
        self.0["label_visible"].as_bool().unwrap_or(true)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput.entries
    pub fn entries(&self) -> Vec<crate::dialog::OptionEntry> {
        self.0["options"].as_array().into_iter().flatten().cloned().map(crate::dialog::OptionEntry).collect()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/TextDialogInput.maxLength
    pub fn get_max_length(&self) -> i64 {
        self.0["max_length"].as_i64().unwrap_or(32)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/TextDialogInput.multiline
    pub fn get_multiline(&self) -> Option<MultilineOptions> {
        self.0.get("multiline").map(|options| MultilineOptions(options.clone()))
    }
}

/// `io.papermc.paper.registry.data.dialog.input.TextDialogInput$MultilineOptions`.
#[derive(Clone, Debug, PartialEq)]
pub struct MultilineOptions(pub Value);

impl MultilineOptions {
    /// bukkit: io/papermc/paper/registry/data/dialog/input/TextDialogInput$MultilineOptions.maxLines
    pub fn max_lines(&self) -> Option<i64> {
        self.0["max_lines"].as_i64()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/TextDialogInput$MultilineOptions.height
    pub fn height(&self) -> Option<i64> {
        self.0["height"].as_i64()
    }
}

// ---- Brigadier arguments ----

/// An argument of the words after a command: how many it may take, what's a good value of them, and
/// what to offer while it's typed. Words stand in for Brigadier's string reader.
#[derive(Clone)]
pub struct CustomArgument {
    name: &'static str,
    /// What the client knows it as, for one the game's parsers have.
    client: Option<&'static str>,
    most: usize,
    check: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    suggest: Arc<dyn Fn() -> Vec<String> + Send + Sync>,
}

impl std::fmt::Debug for CustomArgument {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "CustomArgument({})", self.name)
    }
}

impl PartialEq for CustomArgument {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && Arc::ptr_eq(&self.check, &other.check)
    }
}

impl CustomArgument {
    /// How many of these words make a good value, the fewest that do.
    pub(crate) fn take(&self, words: &[&str]) -> Option<usize> {
        if self.is_message() {
            return (!words.is_empty()).then_some(words.len());
        }
        (1..=self.most.min(words.len())).find(|count| (self.check)(&words[..*count].join(" ")))
    }
    pub(crate) fn suggestions(&self) -> Vec<String> {
        (self.suggest)()
    }
    /// `MessageArgument`: the rest of the line, which a client signs.
    pub(crate) fn is_message(&self) -> bool {
        self.client == Some("message")
    }
}

/// What a client signs on a line, from `plugin_signable` in `paper_fam_paper.rs`.
struct SignableLine {
    line: String,
}

impl crate::event::Event for SignableLine {
    const HOOK: &'static str = "paper_signable";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { line: value["line"].as_str()?.to_owned() })
    }
}

static SIGNING_TREES: Mutex<Vec<Arc<CommandNode>>> = Mutex::new(Vec::new());
static SIGNING_LISTENER: std::sync::Once = std::sync::Once::new();

/// A tree with a message in it: the game has to be told what's signed on its lines.
pub(crate) fn note_signing_tree(plugin: &Plugin, tree: &Arc<CommandNode>) {
    SIGNING_LISTENER.call_once(|| {
        plugin.register_event::<SignableLine>(|event| {
            let words: Vec<&str> = event.line.split(' ').filter(|word| !word.is_empty()).collect();
            let Some((label, rest)) = words.split_first() else {
                return;
            };
            let trees = locked(&SIGNING_TREES).clone();
            for tree in trees.iter().filter(|tree| tree.name == *label) {
                let mut found = Vec::new();
                if tree.signable(rest, &mut found) && !found.is_empty() {
                    crate::event::carry("signable", json!(found));
                    return;
                }
            }
        });
    });
    locked(&SIGNING_TREES).push(tree.clone());
}

fn custom(name: &'static str, most: usize, check: impl Fn(&str) -> bool + Send + Sync + 'static) -> ArgumentType {
    ArgumentType::Custom(CustomArgument { name, client: None, most, check: Arc::new(check), suggest: Arc::new(Vec::new) })
}

fn on_client(name: &'static str, client: &'static str, most: usize, check: impl Fn(&str) -> bool + Send + Sync + 'static) -> ArgumentType {
    ArgumentType::Custom(CustomArgument { name, client: Some(client), most, check: Arc::new(check), suggest: Arc::new(Vec::new) })
}

fn listed(name: &'static str, options: &'static [&'static str]) -> ArgumentType {
    ArgumentType::Custom(CustomArgument { name, client: None, most: 1, check: Arc::new(move |word| options.contains(&word)), suggest: Arc::new(move || options.iter().map(|option| (*option).to_owned()).collect()) })
}

fn is_coordinate(word: &str, whole: bool) -> bool {
    let number = word.strip_prefix(['~', '^']).unwrap_or(word);
    let relative = number.len() != word.len();
    (relative && number.is_empty()) || (!whole || relative) && number.parse::<f64>().is_ok_and(f64::is_finite) || number.parse::<i32>().is_ok()
}

fn coordinates(count: usize, whole: bool) -> impl Fn(&str) -> bool + Send + Sync + 'static {
    move |text| {
        let words: Vec<&str> = text.split(' ').collect();
        words.len() == count && words.iter().all(|word| is_coordinate(word, whole))
    }
}

fn is_key(text: &str) -> bool {
    NamespacedKey::from_string(text).is_some()
}

/// Whether brackets and quotes close, so a value with spaces has been read to its end.
fn balanced(text: &str) -> bool {
    let (mut depth, mut quote, mut escaped) = (0_i32, None, false);
    for letter in text.chars() {
        match (quote, letter) {
            (Some(_), _) if escaped => escaped = false,
            (Some(_), '\\') => escaped = true,
            (Some(open), _) if open == letter => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(letter),
            (None, '[' | '{') => depth += 1,
            (None, ']' | '}') => depth -= 1,
            _ => {}
        }
    }
    depth == 0 && quote.is_none()
}

fn is_selector(text: &str) -> bool {
    match text.strip_prefix('@') {
        Some(rest) => matches!(rest.chars().next(), Some('a' | 'e' | 'n' | 'p' | 'r' | 's')) && (rest.len() == 1 || rest[1..].starts_with('[') && rest.ends_with(']')) && balanced(text),
        None => !text.is_empty() && !text.contains(' '),
    }
}

fn is_uuid(text: &str) -> bool {
    let groups: Vec<&str> = text.split('-').collect();
    groups.len() == 5 && groups.iter().all(|group| !group.is_empty() && group.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

/// A value with its brackets: `id`, `id[...]`, `id{...}`, `#tag`.
fn is_id_form(text: &str, tags: bool) -> bool {
    let id = text.split(['[', '{']).next().unwrap_or(text);
    let id = if tags { id.strip_prefix('#').unwrap_or(id) } else { id };
    is_key(id) && balanced(text)
}

fn time_ticks(text: &str) -> Option<i32> {
    let (number, scale) = match text.chars().last()? {
        'd' => (&text[..text.len() - 1], 24000.0),
        's' => (&text[..text.len() - 1], 20.0),
        't' => (&text[..text.len() - 1], 1.0),
        _ => (text, 1.0),
    };
    let value = number.parse::<f32>().ok().filter(|value| *value >= 0.0)? * scale;
    Some(value.round() as i32)
}

fn range_of<T: std::str::FromStr + Copy>(text: &str) -> Option<(Option<T>, Option<T>)> {
    let bound = |word: &str| if word.is_empty() { Some(None) } else { word.parse().ok().map(Some) };
    match text.split_once("..") {
        Some(("", "")) => None,
        Some((low, high)) => Some((bound(low)?, bound(high)?)),
        None => {
            let value = text.parse().ok()?;
            Some((Some(value), Some(value)))
        }
    }
}

fn hex_color(text: &str) -> Option<Color> {
    let digits = text.strip_prefix('#').unwrap_or(text);
    let rgb = (digits.len() == 6).then(|| u32::from_str_radix(digits, 16).ok()).flatten()?;
    Some(Color::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8))
}

const NAMED_COLORS: &[&str] = &["black", "dark_blue", "dark_green", "dark_aqua", "dark_red", "dark_purple", "gold", "gray", "dark_gray", "blue", "green", "aqua", "red", "light_purple", "yellow", "white"];

/// `io.papermc.paper.command.brigadier.argument.ArgumentTypes`, as `ArgumentType`s. `CommandContext`'s
/// `get_...` calls read them, as `getArgument(name, X.class)` does.
pub mod argument_types {
    use super::*;

    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.uuid
    pub fn uuid() -> ArgumentType {
        custom("uuid", 1, is_uuid)
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.blockPosition
    pub fn block_position() -> ArgumentType {
        on_client("block_position", "block_pos", 3, coordinates(3, true))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.finePosition
    pub fn fine_position() -> ArgumentType {
        on_client("fine_position", "vec3", 3, coordinates(3, false))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.columnBlockPosition
    pub fn column_block_position() -> ArgumentType {
        on_client("column_block_position", "column_pos", 2, coordinates(2, true))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.columnFinePosition
    pub fn column_fine_position() -> ArgumentType {
        on_client("column_fine_position", "vec2", 2, coordinates(2, false))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.rotation
    pub fn rotation() -> ArgumentType {
        on_client("rotation", "rotation", 2, coordinates(2, false))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.angle
    pub fn angle() -> ArgumentType {
        custom("angle", 1, |word| !word.contains('^') && is_coordinate(word, false))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.namespacedKey
    pub fn namespaced_key() -> ArgumentType {
        custom("namespaced_key", 1, is_key)
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.key
    pub fn key() -> ArgumentType {
        custom("key", 1, is_key)
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.world
    pub fn world() -> ArgumentType {
        listed("world", &["minecraft:overworld", "minecraft:the_nether", "minecraft:the_end", "overworld", "the_nether", "the_end", "world", "world_nether", "world_the_end"])
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.gameMode
    pub fn game_mode() -> ArgumentType {
        listed("game_mode", &["survival", "creative", "adventure", "spectator"])
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.heightMap
    pub fn height_map() -> ArgumentType {
        listed("height_map", &["motion_blocking", "motion_blocking_no_leaves", "ocean_floor", "ocean_floor_wg", "world_surface", "world_surface_wg"])
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.templateMirror
    pub fn template_mirror() -> ArgumentType {
        listed("template_mirror", &["none", "front_back", "left_right"])
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.templateRotation
    pub fn template_rotation() -> ArgumentType {
        listed("template_rotation", &["none", "clockwise_90", "180", "counterclockwise_90"])
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.entityAnchor
    pub fn entity_anchor() -> ArgumentType {
        listed("entity_anchor", &["feet", "eyes"])
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.namedColor
    pub fn named_color() -> ArgumentType {
        listed("named_color", NAMED_COLORS)
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.hexColor
    pub fn hex_color() -> ArgumentType {
        custom("hex_color", 1, |word| super::hex_color(word).is_some())
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.scoreboardDisplaySlot
    pub fn scoreboard_display_slot() -> ArgumentType {
        custom("display_slot", 1, |word| matches!(word, "list" | "sidebar" | "below_name") || word.strip_prefix("sidebar.team.").is_some_and(|color| NAMED_COLORS.contains(&color)))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.objectiveCriteria
    pub fn objective_criteria() -> ArgumentType {
        custom("objective_criteria", 1, |word| !word.is_empty())
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.axes
    pub fn axes() -> ArgumentType {
        custom("axes", 1, |word| !word.is_empty() && word.len() <= 3 && word.chars().all(|axis| matches!(axis, 'x' | 'y' | 'z')) && word.chars().collect::<BTreeSet<_>>().len() == word.len())
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.time
    ///
    /// Not less than `least` ticks.
    pub fn time(least: i32) -> ArgumentType {
        custom("time", 1, move |word| time_ticks(word).is_some_and(|ticks| ticks >= least))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.integerRange
    pub fn integer_range() -> ArgumentType {
        custom("integer_range", 1, |word| range_of::<i32>(word).is_some())
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.doubleRange
    pub fn double_range() -> ArgumentType {
        custom("double_range", 1, |word| range_of::<f64>(word).is_some())
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.entity
    pub fn entity() -> ArgumentType {
        custom("entity", usize::MAX, is_selector)
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.entities
    pub fn entities() -> ArgumentType {
        custom("entities", usize::MAX, is_selector)
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.playerProfiles
    pub fn player_profiles() -> ArgumentType {
        custom("player_profiles", usize::MAX, is_selector)
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.itemStack
    pub fn item_stack() -> ArgumentType {
        custom("item_stack", usize::MAX, |text| is_id_form(text, false))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.itemPredicate
    pub fn item_predicate() -> ArgumentType {
        custom("item_predicate", usize::MAX, |text| query(json!({ "op": "paper_item_test", "predicate": text })).as_bool() == Some(true))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.blockState
    pub fn block_state() -> ArgumentType {
        custom("block_state", usize::MAX, |text| is_id_form(text, false))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.blockInWorldPredicate
    pub fn block_in_world_predicate() -> ArgumentType {
        custom("block_in_world_predicate", usize::MAX, |text| is_id_form(text, true))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.component
    pub fn component() -> ArgumentType {
        custom("component", usize::MAX, |text| balanced(text) && (text.starts_with(['{', '[', '"', '\'']) || !text.contains(' ')))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.style
    pub fn style() -> ArgumentType {
        custom("style", usize::MAX, |text| text.starts_with('{') && balanced(text))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.signedMessage
    pub fn signed_message() -> ArgumentType {
        on_client("signed_message", "message", 1, |_| true)
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.resource, io/papermc/paper/command/brigadier/argument/ArgumentTypes.resourceKey
    pub fn resource(registry: RegistryKey) -> ArgumentType {
        let keys = GameRegistry(registry);
        let names = move |word: &str| NamespacedKey::from_string(word).is_some_and(|key| keys.get(&key).is_some());
        ArgumentType::Custom(CustomArgument { name: "resource", client: None, most: 1, check: Arc::new(names), suggest: Arc::new(move || keys.keys()) })
    }
}

/// `io.papermc.paper.command.brigadier.argument.predicate.BlockInWorldPredicate$Result`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockTestResult {
    True,
    False,
    UnloadedChunk,
}

impl BlockTestResult {
    /// bukkit: io/papermc/paper/command/brigadier/argument/predicate/BlockInWorldPredicate$Result.asBoolean
    pub fn as_boolean(self) -> bool {
        self == Self::True
    }
}

/// `NbtUtils.compareNbt` with partial lists: everything in `wanted` is in `saved`.
fn nbt_contains(saved: &Value, wanted: &Value) -> bool {
    match (saved, wanted) {
        (Value::Object(saved), Value::Object(wanted)) => wanted.iter().all(|(key, value)| saved.get(key).is_some_and(|there| nbt_contains(there, value))),
        (Value::Array(saved), Value::Array(wanted)) => wanted.iter().all(|value| saved.iter().any(|there| nbt_contains(there, value))),
        (saved, wanted) => saved == wanted,
    }
}

/// `io.papermc.paper.command.brigadier.argument.predicate.BlockInWorldPredicate`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockInWorldPredicate(pub String);

impl BlockInWorldPredicate {
    /// bukkit: io/papermc/paper/command/brigadier/argument/predicate/BlockInWorldPredicate.testBlock
    pub fn test_block(&self, block: &Block) -> BlockTestResult {
        let at = block.get_location();
        if !at.get_world().is_chunk_loaded(block.get_x() >> 4, block.get_z() >> 4) {
            return BlockTestResult::UnloadedChunk;
        }
        let (head, wanted) = self.0.find('{').map_or((self.0.as_str(), None), |at| (&self.0[..at], Some(&self.0[at..])));
        let (id, properties) = head.split_once('[').map_or((head, ""), |(id, rest)| (id, rest.trim_end_matches(']')));
        let kind = block.get_type();
        let found = match id.strip_prefix('#') {
            Some(tag) => crate::registry::Tag::get("block", tag).is_tagged(kind.key()),
            None => NamespacedKey::from_string(id).is_some_and(|id| id.as_str() == kind.key()),
        };
        let fits = properties.split(',').filter_map(|pair| pair.split_once('=')).all(|(key, value)| block.get_property(key.trim()).as_deref() == Some(value.trim()));
        let saved = wanted.is_none_or(|snbt| nbt_contains(&query(json!({ "op": "block_data", "dimension": at.get_world().dimension(), "pos": [block.get_x(), block.get_y(), block.get_z()] })), &parsed_snbt(snbt)));
        if found && fits && saved { BlockTestResult::True } else { BlockTestResult::False }
    }
}

/// `io.papermc.paper.command.brigadier.argument.predicate.ItemStackPredicate`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStackPredicate(pub String);

impl ItemStackPredicate {
    /// Java's `Predicate.test`, by the game's own `ItemPredicateArgument`.
    pub fn test(&self, stack: &ItemStack) -> bool {
        query(json!({ "op": "paper_item_test", "predicate": self.0, "stack": stack.to_json() })).as_bool() == Some(true)
    }
}

/// `io.papermc.paper.command.brigadier.argument.position.ColumnBlockPosition`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnBlockPosition {
    x: i32,
    z: i32,
}

impl ColumnBlockPosition {
    /// bukkit: io/papermc/paper/command/brigadier/argument/position/ColumnBlockPosition.blockX
    pub fn block_x(&self) -> i32 {
        self.x
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/position/ColumnBlockPosition.blockZ
    pub fn block_z(&self) -> i32 {
        self.z
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/position/ColumnBlockPosition.toPosition
    pub fn to_position(&self, y: i32) -> Position {
        Position::block(self.x, y, self.z)
    }
}

/// `io.papermc.paper.command.brigadier.argument.position.ColumnFinePosition`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnFinePosition {
    x: f64,
    z: f64,
}

impl ColumnFinePosition {
    /// bukkit: io/papermc/paper/command/brigadier/argument/position/ColumnFinePosition.x
    pub fn x(&self) -> f64 {
        self.x
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/position/ColumnFinePosition.z
    pub fn z(&self) -> f64 {
        self.z
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/position/ColumnFinePosition.toPosition
    pub fn to_position(&self, y: f64) -> Position {
        Position::fine(self.x, y, self.z)
    }
}

/// `io.papermc.paper.command.brigadier.argument.SignedMessageResolver`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedMessageResolver(pub String);

impl SignedMessageResolver {
    /// bukkit: io/papermc/paper/command/brigadier/argument/SignedMessageResolver.content
    pub fn content(&self) -> &str {
        &self.0
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/SignedMessageResolver.resolveSignedMessage
    ///
    /// The chain has checked it by the time a command runs, so Java's future is done already. With nothing
    /// signed it's a system message, as Java's is.
    pub fn resolve_signed_message(&self, argument_name: &str, _context: &CommandContext) -> crate::fam_ev_player::SignedChatMessage {
        let row = query(json!({ "op": "paper_free_signed_argument", "name": argument_name }));
        if !row.is_null() {
            return crate::fam_ev_player::SignedChatMessage::of(row);
        }
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |since| since.as_millis() as i64);
        crate::fam_ev_player::SignedChatMessage::of(json!({ "message": self.0, "timestamp": now, "salt": 0 }))
    }
}

/// `net.kyori.adventure.text.format.Style`, as a component's style fields.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style(pub Value);

/// The numbers of a position against where a command runs: absolute, `~` or `^`.
fn resolve_coordinates(words: &[&str], from: &Location) -> Option<[f64; 3]> {
    let [x, y, z] = words else {
        return None;
    };
    let read = |word: &str| word.trim_start_matches(['~', '^']).parse::<f64>().or(if word.len() == 1 { Ok(0.0) } else { Err(()) }).ok();
    if [x, y, z].iter().any(|word| word.starts_with('^')) {
        // LocalCoordinates.getPosition: left, up and forward of the facing.
        let (left, up, forward) = (read(x)?, read(y)?, read(z)?);
        let (yaw, pitch) = (f64::from(from.get_yaw()).to_radians(), f64::from(from.get_pitch()).to_radians());
        let (yaw_cos, yaw_sin) = ((yaw + std::f64::consts::FRAC_PI_2).cos(), (yaw + std::f64::consts::FRAC_PI_2).sin());
        let (pitch_cos, pitch_sin) = (pitch.cos(), (-pitch).sin());
        let (up_cos, up_sin) = ((pitch - std::f64::consts::FRAC_PI_2).cos(), (std::f64::consts::FRAC_PI_2 - pitch).sin());
        let front = [yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos];
        let top = [yaw_cos * up_cos, up_sin, yaw_sin * up_cos];
        let side = [front[1] * top[2] - front[2] * top[1], front[2] * top[0] - front[0] * top[2], front[0] * top[1] - front[1] * top[0]].map(|value: f64| -value);
        return Some([from.get_x() + front[0] * forward + top[0] * up + side[0] * left, from.get_y() + front[1] * forward + top[1] * up + side[1] * left, from.get_z() + front[2] * forward + top[2] * up + side[2] * left]);
    }
    let one = |word: &str, base: f64| Some(if word.starts_with('~') { base + read(word)? } else { read(word)? });
    Some([one(x, from.get_x())?, one(y, from.get_y())?, one(z, from.get_z())?])
}

fn parsed_snbt(text: &str) -> Value {
    query(json!({ "op": "paper_snbt", "text": text }))
}

/// A stack as commands write it: `minecraft:stone[custom_name="x",...]`.
fn stack_from_text(text: &str) -> Option<ItemStack> {
    let (id, rest) = text.split_once('[').map_or((text, ""), |(id, rest)| (id, rest.strip_suffix(']').unwrap_or(rest)));
    let key = NamespacedKey::from_string(id)?;
    let mut stack = ItemStack::new(Material::of(key.as_str()), 1);
    let (mut depth, mut quote, mut start) = (0_i32, None, 0);
    let mut parts = Vec::new();
    for (at, letter) in rest.char_indices() {
        match (quote, letter) {
            (Some(open), _) if open == letter => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(letter),
            (None, '[' | '{') => depth += 1,
            (None, ']' | '}') => depth -= 1,
            (None, ',') if depth == 0 => {
                parts.push(&rest[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    parts.push(&rest[start..]);
    for part in parts.into_iter().filter(|part| !part.trim().is_empty()) {
        let (name, value) = part.split_once('=')?;
        let name = name.trim().trim_start_matches("minecraft:");
        stack.set_component(&format!("minecraft:{name}"), parsed_snbt(value.trim()));
    }
    Some(stack)
}

impl CommandContext {
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.getLocation
    pub fn get_location(&self) -> Location {
        if let Some(at) = &self.location {
            return at.clone();
        }
        match self.get_executor() {
            Some(player) => player.get_location(),
            None => World::of("world").get_spawn_location(),
        }
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.withLocation
    pub fn with_location(&self, location: &Location) -> Self {
        let mut copy = self.clone();
        copy.location = Some(location.clone());
        copy
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.getPlayerOrThrow
    ///
    /// `Err` is Java's `CommandSyntaxException`.
    pub fn get_player_or_throw(&self) -> Result<&Player, String> {
        self.get_executor().ok_or_else(|| "A player is required to run this command here".to_owned())
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.sendReply
    pub fn send_reply(&self, message: &Component) {
        self.get_sender().send_component(message);
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.sendRichReply
    pub fn send_rich_reply(&self, message: &str) {
        self.send_reply(&crate::mini_message(message));
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.sendRichSuccess
    pub fn send_rich_success(&self, message: &str) {
        self.send_success(&crate::mini_message(message), true);
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.sendFailure
    pub fn send_failure(&self, message: &Component) {
        let mut red = message.0.clone();
        if red.is_string() {
            red = json!({ "text": red });
        }
        red["color"] = json!("red");
        self.send_reply(&Component(red));
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.sendRichFailure
    pub fn send_rich_failure(&self, message: &str) {
        self.send_failure(&crate::mini_message(message));
    }
    /// bukkit: com/destroystokyo/paper/brigadier/BukkitBrigadierCommandSource.getBukkitSender
    pub fn get_bukkit_sender(&self) -> &CommandSender {
        self.get_sender()
    }
    /// bukkit: com/destroystokyo/paper/brigadier/BukkitBrigadierCommandSource.getBukkitWorld
    pub fn get_bukkit_world(&self) -> World {
        self.get_location().get_world().clone()
    }
    /// bukkit: com/destroystokyo/paper/brigadier/BukkitBrigadierCommandSource.getBukkitLocation
    pub fn get_bukkit_location(&self) -> Location {
        self.get_location()
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/resolvers/ArgumentResolver.resolve
    ///
    /// `BlockPositionResolver`: `get_block_position`. The others are the `get_...` below.
    pub fn get_block_position(&self, name: &str) -> Option<Position> {
        let at = self.get_fine_position(name)?;
        Some(at.to_block())
    }
    pub fn get_fine_position(&self, name: &str) -> Option<Position> {
        let words: Vec<&str> = self.get_string(name)?.split(' ').collect();
        let [x, y, z] = resolve_coordinates(&words, &self.get_location())?;
        Some(Position::fine(x, y, z))
    }
    pub fn get_column_block_position(&self, name: &str) -> Option<ColumnBlockPosition> {
        let at = self.get_column_fine_position(name)?;
        Some(ColumnBlockPosition { x: at.x.floor() as i32, z: at.z.floor() as i32 })
    }
    pub fn get_column_fine_position(&self, name: &str) -> Option<ColumnFinePosition> {
        let (x, z) = self.get_string(name)?.split_once(' ')?;
        let [x, _, z] = resolve_coordinates(&[x, "~", z], &self.get_location())?;
        Some(ColumnFinePosition { x, z })
    }
    pub fn get_rotation(&self, name: &str) -> Option<Rotation> {
        let (yaw, pitch) = self.get_string(name)?.split_once(' ')?;
        let from = self.get_location();
        let one = |word: &str, base: f32| Some(if let Some(number) = word.strip_prefix('~') { base + if number.is_empty() { 0.0 } else { number.parse::<f32>().ok()? } } else { word.parse::<f32>().ok()? });
        Some(Rotation::rotation(one(yaw, from.get_yaw())?, one(pitch, from.get_pitch())?))
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/resolvers/AngleResolver.resolve
    pub fn get_angle(&self, name: &str) -> Option<Angle> {
        let word = self.get_string(name)?;
        Some(match word.strip_prefix('~') {
            Some(number) => Angle::relative(if number.is_empty() { 0.0 } else { number.parse().ok()? }),
            None => Angle::absolute(word.parse().ok()?),
        })
    }
    pub fn get_uuid(&self, name: &str) -> Option<String> {
        self.get_string(name).filter(|text| is_uuid(text)).map(str::to_owned)
    }
    pub fn get_namespaced_key(&self, name: &str) -> Option<NamespacedKey> {
        NamespacedKey::from_string(self.get_string(name)?)
    }
    pub fn get_world(&self, name: &str) -> Option<World> {
        let word = self.get_string(name)?;
        let dimension = word.trim_start_matches("minecraft:");
        Some(World::of(match dimension {
            "the_nether" | "world_nether" => "world_nether",
            "the_end" | "world_the_end" => "world_the_end",
            _ => "world",
        }))
    }
    pub fn get_game_mode(&self, name: &str) -> Option<String> {
        self.get_string(name).map(str::to_ascii_uppercase)
    }
    pub fn get_height_map(&self, name: &str) -> Option<String> {
        self.get_string(name).map(str::to_ascii_uppercase)
    }
    pub fn get_mirror(&self, name: &str) -> Option<String> {
        self.get_string(name).map(str::to_ascii_uppercase)
    }
    pub fn get_structure_rotation(&self, name: &str) -> Option<String> {
        self.get_string(name).map(|word| if word == "180" { "CLOCKWISE_180".to_owned() } else { word.to_ascii_uppercase() })
    }
    pub fn get_anchor(&self, name: &str) -> Option<String> {
        self.get_string(name).map(str::to_ascii_uppercase)
    }
    pub fn get_named_color(&self, name: &str) -> Option<String> {
        self.get_string(name).map(str::to_owned)
    }
    pub fn get_hex_color(&self, name: &str) -> Option<Color> {
        hex_color(self.get_string(name)?)
    }
    pub fn get_axes(&self, name: &str) -> Option<BTreeSet<char>> {
        Some(self.get_string(name)?.chars().collect())
    }
    pub fn get_time(&self, name: &str) -> Option<i32> {
        time_ticks(self.get_string(name)?)
    }
    pub fn get_integer_range(&self, name: &str) -> Option<(Option<i32>, Option<i32>)> {
        range_of(self.get_string(name)?)
    }
    pub fn get_double_range(&self, name: &str) -> Option<(Option<f64>, Option<f64>)> {
        range_of(self.get_string(name)?)
    }
    pub fn get_item_stack(&self, name: &str) -> Option<ItemStack> {
        stack_from_text(self.get_string(name)?)
    }
    pub fn get_item_predicate(&self, name: &str) -> Option<ItemStackPredicate> {
        self.get_string(name).map(|text| ItemStackPredicate(text.to_owned()))
    }
    pub fn get_block_state(&self, name: &str) -> Option<String> {
        self.get_string(name).map(str::to_owned)
    }
    pub fn get_block_in_world_predicate(&self, name: &str) -> Option<BlockInWorldPredicate> {
        self.get_string(name).map(|text| BlockInWorldPredicate(text.to_owned()))
    }
    pub fn get_component(&self, name: &str) -> Option<Component> {
        let text = self.get_string(name)?;
        if !text.starts_with(['{', '[', '"', '\'']) {
            return Some(Component::text(text));
        }
        Some(Component(parsed_snbt(text)))
    }
    pub fn get_style(&self, name: &str) -> Option<Style> {
        Some(Style(parsed_snbt(self.get_string(name)?)))
    }
    pub fn get_signed_message(&self, name: &str) -> Option<SignedMessageResolver> {
        self.get_string(name).map(|text| SignedMessageResolver(text.to_owned()))
    }
    /// What a selector or name found, in the order the game gave them: players first.
    pub fn get_entities(&self, name: &str) -> (Vec<Player>, Vec<Entity>) {
        self.get_string(name).map_or_else(|| (Vec::new(), Vec::new()), |text| select(text, self.get_sender().as_player()))
    }
    pub fn get_player_profiles(&self, name: &str) -> Vec<PlayerProfile> {
        let text = self.get_string(name).unwrap_or_default();
        let (players, _) = select(text, self.get_sender().as_player());
        if !players.is_empty() || text.starts_with('@') {
            return players.iter().map(Player::get_player_profile).collect();
        }
        vec![create_player_profile(&crate::get_offline_player(text).get_unique_id().unwrap_or_default().to_owned(), text)]
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/RegistryArgumentExtractor.getTypedKey
    ///
    /// # Panics
    /// As Java throws, for a name with no such key.
    pub fn get_typed_key(&self, registry: RegistryKey, name: &str) -> RegistryTypedKey {
        registry.typed_key(&self.get_namespaced_key(name).expect("a resource key argument"))
    }
    /// An argument of a plugin's own type.
    pub fn get_custom<C: CustomArgumentType>(&self, name: &str, kind: &C) -> Option<C::Value> {
        kind.parse(self.get_string(name)?).ok()
    }
}

/// `@e[...]` and the rest, through the game's own selector. The console has no `@s`.
pub fn select(selector: &str, source: Option<&Player>) -> (Vec<Player>, Vec<Entity>) {
    if !selector.starts_with('@') {
        return (crate::get_player(selector).into_iter().collect(), Vec::new());
    }
    let found = query(json!({ "op": "paper_select", "selector": selector, "source": source.map(Player::uuid) }));
    let players = found["players"].as_array().into_iter().flatten().filter_map(|uuid| Player::read(&query(json!({ "op": "player", "uuid": uuid })))).collect();
    let entities = found["entities"].as_array().into_iter().flatten().filter_map(|row| Some(Entity::of(row["id"].as_u64()?, &World::of(crate::world_of(row["dimension"].as_str()?))))).collect();
    (players, entities)
}

/// `io.papermc.paper.command.brigadier.argument.CustomArgumentType`: `parse` sees the argument's text, and
/// `Err` is Java's `CommandSyntaxException`.
pub trait CustomArgumentType: Send + Sync + 'static {
    type Value;
    /// bukkit: io/papermc/paper/command/brigadier/argument/CustomArgumentType.parse
    fn parse(&self, text: &str) -> Result<Self::Value, String>;
    /// bukkit: io/papermc/paper/command/brigadier/argument/CustomArgumentType.getNativeType
    fn get_native_type(&self) -> ArgumentType;
    /// bukkit: io/papermc/paper/command/brigadier/argument/CustomArgumentType.getExamples
    fn get_examples(&self) -> Vec<String> {
        native_examples(&self.get_native_type())
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/CustomArgumentType.listSuggestions
    fn list_suggestions(&self) -> Vec<String> {
        self.get_native_type().suggestions()
    }
}

fn native_examples(kind: &ArgumentType) -> Vec<String> {
    let rows: &[&str] = match kind {
        ArgumentType::Word => &["word", "words_with_underscores"],
        ArgumentType::Greedy => &["word", "words with spaces", "\"and symbols\""],
        ArgumentType::Integer { .. } => &["0", "123", "-123"],
        ArgumentType::Double { .. } => &["0", "1.2", ".5", "-1", "-.5", "-1234.56"],
        ArgumentType::Bool => &["true", "false"],
        ArgumentType::Player | ArgumentType::Players => &["Player", "0123", "@e", "@e[type=foo]", "dd12be42-52a9-4a91-a8a1-11c01849e498"],
        ArgumentType::Custom(_) => &[],
    };
    rows.iter().map(|row| (*row).to_owned()).collect()
}

/// `io.papermc.paper.command.brigadier.argument.CustomArgumentType$Converted`.
pub trait ConvertedArgumentType: Send + Sync + 'static {
    type Value;
    fn native_type(&self) -> ArgumentType;
    /// bukkit: io/papermc/paper/command/brigadier/argument/CustomArgumentType$Converted.convert
    fn convert(&self, native: &str) -> Result<Self::Value, String>;
    /// bukkit: io/papermc/paper/command/brigadier/argument/CustomArgumentType$Converted.parse
    fn parse_native(&self, text: &str) -> Result<Self::Value, String> {
        if self.native_type().accepts(text) { self.convert(text) } else { Err(format!("Invalid value: {text}")) }
    }
}

impl<C: ConvertedArgumentType> CustomArgumentType for C {
    type Value = C::Value;
    fn parse(&self, text: &str) -> Result<Self::Value, String> {
        self.parse_native(text)
    }
    fn get_native_type(&self) -> ArgumentType {
        self.native_type()
    }
}

/// A plugin's own argument, for `CommandNode::argument`.
pub fn custom_argument<C: CustomArgumentType>(kind: C) -> ArgumentType {
    let kind = Arc::new(kind);
    let suggests = kind.clone();
    ArgumentType::Custom(CustomArgument { name: "custom", client: None, most: usize::MAX, check: Arc::new(move |text| kind.parse(text).is_ok()), suggest: Arc::new(move || suggests.list_suggestions()) })
}

// ---- Commands ----

/// `io.papermc.paper.command.brigadier.BasicCommand`: `Err` is Java's `CommandSyntaxException`.
pub trait BasicCommand: Send + Sync + 'static {
    /// bukkit: io/papermc/paper/command/brigadier/BasicCommand.execute
    fn execute(&self, sender: &CommandSender, args: &[&str]) -> Result<(), String>;
    /// bukkit: io/papermc/paper/command/brigadier/BasicCommand.suggest
    fn suggest(&self, _sender: &CommandSender, _args: &[&str]) -> Vec<String> {
        Vec::new()
    }
    /// bukkit: io/papermc/paper/command/brigadier/BasicCommand.canUse
    fn can_use(&self, sender: &CommandSender) -> bool {
        self.permission().is_none_or(|node| sender.has_permission(node))
    }
    /// bukkit: io/papermc/paper/command/brigadier/BasicCommand.permission
    fn permission(&self) -> Option<&str> {
        None
    }
}

impl CommandNode {
    /// bukkit: io/papermc/paper/command/brigadier/Commands.restricted
    ///
    /// `.requires(Commands.restricted(test))`: running it from a click asks the player first.
    pub fn requires_restricted(self, test: impl Fn(&CommandSender) -> bool + Send + Sync + 'static) -> Self {
        let mut node = self.requires(test);
        node.restricted = true;
        node
    }
}

/// A plugin's tree as the client should see it: each path that's more than words gets a usage of its own,
/// beside the command's text argument, so the client knows its positions; and what's restricted.
pub(crate) fn usages_of(node: &CommandNode) -> Vec<String> {
    let (mut usages, mut restricted) = (vec![format!("{} [args:text]", node.name)], Vec::new());
    walk_tree(node, &mut vec![node.name.clone()], &mut vec![node.name.clone()], &mut usages, &mut restricted, false);
    usages
}

/// A plugin's tree as the client should see it: see `walk_tree`.
pub(crate) fn tell_client(plugin: &Plugin, node: &CommandNode) {
    let mut usages = Vec::new();
    let mut restricted = Vec::new();
    walk_tree(node, &mut vec![node.name.clone()], &mut vec![node.name.clone()], &mut usages, &mut restricted, false);
    for usage in usages {
        plugin.host.command(&usage, 0);
    }
    if !restricted.is_empty() {
        // A plugin registers as it loads, and the game answers once it's running.
        scheduler::run_task(move || {
            for path in &restricted {
                query(json!({ "op": "paper_restrict", "path": path }));
            }
        });
    }
}

fn walk_tree(node: &CommandNode, usage: &mut Vec<String>, names: &mut Vec<String>, usages: &mut Vec<String>, restricted: &mut Vec<Vec<String>>, typed: bool) {
    if node.restricted {
        restricted.push(names.clone());
    }
    if node.run.is_some() && typed {
        usages.push(usage.join(" "));
    }
    for child in &node.children {
        let (word, typed) = match &child.argument {
            None => (child.name.clone(), typed),
            Some(ArgumentType::Custom(custom)) => match custom.client {
                Some(client) => (format!("<{}:{client}>", child.name), true),
                None => continue,
            },
            Some(ArgumentType::Word) => (format!("<{}:word>", child.name), typed),
            Some(ArgumentType::Integer { .. }) => (format!("<{}:integer>", child.name), typed),
            Some(ArgumentType::Double { .. }) => (format!("<{}:number>", child.name), typed),
            Some(ArgumentType::Bool) => (format!("<{}:boolean>", child.name), typed),
            Some(ArgumentType::Player) => (format!("<{}:player>", child.name), typed),
            Some(ArgumentType::Greedy | ArgumentType::Players) => continue,
        };
        usage.push(word);
        names.push(child.name.clone());
        walk_tree(child, usage, names, usages, restricted, typed);
        usage.pop();
        names.pop();
    }
}

/// bukkit: io/papermc/paper/command/brigadier/Commands.unrestricted
pub fn unrestricted(test: impl Fn(&CommandSender) -> bool + Send + Sync + 'static) -> impl Fn(&CommandSender) -> bool + Send + Sync + 'static {
    test
}

/// `io.papermc.paper.command.brigadier.Commands`, for one plugin: call it in `on_enable`, as the game's
/// command tree is made when the mods are loaded.
#[derive(Clone, Copy)]
pub struct Commands {
    plugin: Plugin,
}

/// `com.mojang.brigadier.CommandDispatcher`, of a plugin's trees: running a line, not the game's own tree.
#[derive(Clone, Copy)]
pub struct CommandDispatcher {
    plugin: Plugin,
}

impl CommandDispatcher {
    pub fn register(&self, node: CommandNode) {
        self.plugin.register_command(node);
    }
    /// A line as this sender typed it.
    pub fn execute(&self, line: &str, sender: &CommandSender) -> bool {
        match sender {
            CommandSender::Player(player) => player.perform_command(line),
            CommandSender::Console | CommandSender::Proxied(_) | CommandSender::Remote(_) | CommandSender::Entity(_) => crate::dispatch_command(line),
        }
    }
}

impl Commands {
    pub fn of(plugin: &Plugin) -> Self {
        Self { plugin: *plugin }
    }
    /// bukkit~: io/papermc/paper/command/brigadier/Commands.getDispatcher | it registers and runs lines, the game's commands too, but they are the engine's Rust and not `CommandNode`s, so a plugin can't walk or change the game's nodes through it
    pub fn get_dispatcher(&self) -> CommandDispatcher {
        CommandDispatcher { plugin: self.plugin }
    }
    /// A tree, under its name and each alias. The labels it took.
    pub fn register(&self, node: &CommandNode, aliases: &[&str]) -> BTreeSet<String> {
        let mut labels = BTreeSet::new();
        for alias in aliases {
            let mut copy = node.clone();
            (*alias).clone_into(&mut copy.name);
            labels.insert(copy.name.clone());
            self.plugin.register_command(copy);
        }
        labels.insert(node.name.clone());
        self.plugin.register_command(node.clone());
        labels
    }
    /// bukkit: io/papermc/paper/command/brigadier/Commands.registerWithFlags
    ///
    /// `FLATTEN_ALIASES` is how every alias works here: each is a tree of its own.
    pub fn register_with_flags(&self, node: &CommandNode, aliases: &[&str], _flags: &[&str]) -> BTreeSet<String> {
        self.register(node, aliases)
    }
    /// `register(label, description, aliases, basicCommand)`. Permissions are `can_use`'s.
    pub fn register_basic(&self, label: &str, aliases: &[&str], command: impl BasicCommand) -> BTreeSet<String> {
        let command = Arc::new(command);
        let mut labels = BTreeSet::new();
        for name in std::iter::once(&label).chain(aliases) {
            let (runner, completer) = (command.clone(), command.clone());
            crate::add_command(self.plugin.host, name);
            self.plugin.set_executor(name, move |sender, _, args| {
                if !runner.can_use(sender) {
                    sender.send_message(&format!("{}I'm sorry, but you do not have permission to perform this command.", crate::chat_color::RED));
                } else if let Err(message) = runner.execute(sender, args) {
                    sender.send_message(&format!("{}{message}", crate::chat_color::RED));
                }
                true
            });
            self.plugin.set_tab_completer(name, move |sender, _, args| if completer.can_use(sender) { completer.suggest(sender, args) } else { Vec::new() });
            labels.insert((*name).to_owned());
        }
        labels
    }
}

/// bukkit: io/papermc/paper/command/brigadier/MessageComponentSerializer.message
pub fn message_serializer() -> MessageComponentSerializer {
    MessageComponentSerializer
}

/// `io.papermc.paper.command.brigadier.MessageComponentSerializer`: Brigadier's `Message` is a string here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MessageComponentSerializer;

impl MessageComponentSerializer {
    pub fn serialize(&self, component: &Component) -> String {
        component.plain()
    }
    pub fn deserialize(&self, message: &str) -> Component {
        Component::text(message)
    }
}

/// bukkit: io/papermc/paper/brigadier/PaperBrigadier.message
pub fn brigadier_message(component: &Component) -> String {
    MessageComponentSerializer.serialize(component)
}

/// bukkit: io/papermc/paper/brigadier/PaperBrigadier.componentFromMessage
pub fn component_from_message(message: &str) -> Component {
    MessageComponentSerializer.deserialize(message)
}

// ---- Small values ----

/// `io.papermc.paper.block.BlockPredicate`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockPredicate {
    blocks: Option<RegistryKeySet>,
}

impl BlockPredicate {
    /// bukkit: io/papermc/paper/block/BlockPredicate.predicate
    pub fn predicate() -> Self {
        Self::default()
    }
    /// bukkit: io/papermc/paper/block/BlockPredicate$Builder.blocks
    pub fn with_blocks(mut self, blocks: Option<RegistryKeySet>) -> Self {
        self.blocks = blocks;
        self
    }
    /// bukkit: io/papermc/paper/block/BlockPredicate$Builder.build
    pub fn build(self) -> Self {
        self
    }
    /// bukkit: io/papermc/paper/block/BlockPredicate.blocks
    pub fn blocks(&self) -> Option<&RegistryKeySet> {
        self.blocks.as_ref()
    }
    /// As a data component writes it.
    pub fn to_json(&self) -> Value {
        self.blocks.as_ref().map_or_else(|| json!({}), |blocks| json!({ "blocks": blocks.to_json() }))
    }
}

/// `io.papermc.paper.potion.SuspiciousEffectEntry`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuspiciousEffectEntry {
    effect: String,
    duration: i32,
}

impl SuspiciousEffectEntry {
    /// bukkit: io/papermc/paper/potion/SuspiciousEffectEntry.create
    pub fn create(effect: &str, duration: i32) -> Self {
        Self { effect: effect.to_owned(), duration }
    }
    /// bukkit: io/papermc/paper/potion/SuspiciousEffectEntry.effect
    pub fn effect(&self) -> &str {
        &self.effect
    }
    /// bukkit: io/papermc/paper/potion/SuspiciousEffectEntry.duration
    pub fn duration(&self) -> i32 {
        self.duration
    }
}

/// `io.papermc.paper.inventory.tooltip.TooltipContext`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TooltipContext {
    advanced: bool,
    creative: bool,
}

impl TooltipContext {
    /// bukkit: io/papermc/paper/inventory/tooltip/TooltipContext.create
    pub fn create(advanced: bool, creative: bool) -> Self {
        Self { advanced, creative }
    }
    /// bukkit: io/papermc/paper/inventory/tooltip/TooltipContext.isAdvanced
    pub fn is_advanced(self) -> bool {
        self.advanced
    }
    /// bukkit: io/papermc/paper/inventory/tooltip/TooltipContext.isCreative
    pub fn is_creative(self) -> bool {
        self.creative
    }
    /// bukkit: io/papermc/paper/inventory/tooltip/TooltipContext.asAdvanced
    pub fn as_advanced(self) -> Self {
        Self { advanced: true, ..self }
    }
    /// bukkit: io/papermc/paper/inventory/tooltip/TooltipContext.asCreative
    pub fn as_creative(self) -> Self {
        Self { creative: true, ..self }
    }
}

/// `io.papermc.paper.text.Filtered`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filtered<T> {
    raw: T,
    filtered: Option<T>,
}

impl<T> Filtered<T> {
    /// bukkit: io/papermc/paper/text/Filtered.of
    pub fn of(raw: T, filtered: Option<T>) -> Self {
        Self { raw, filtered }
    }
    /// bukkit: io/papermc/paper/text/Filtered.raw
    pub fn raw(&self) -> &T {
        &self.raw
    }
    /// bukkit: io/papermc/paper/text/Filtered.filtered
    pub fn filtered(&self) -> Option<&T> {
        self.filtered.as_ref()
    }
}

/// `io.papermc.paper.scoreboard.numbers.NumberFormat`.
#[derive(Clone, Debug, PartialEq)]
pub enum NumberFormat {
    Blank,
    Styled(Style),
    Fixed(Component),
}

impl NumberFormat {
    /// bukkit: io/papermc/paper/scoreboard/numbers/NumberFormat.blank
    pub fn blank() -> Self {
        Self::Blank
    }
    /// bukkit: io/papermc/paper/scoreboard/numbers/NumberFormat.noStyle
    pub fn no_style() -> Self {
        Self::Styled(Style::default())
    }
    /// bukkit: io/papermc/paper/scoreboard/numbers/NumberFormat.styled
    pub fn styled(style: Style) -> Self {
        Self::Styled(style)
    }
    /// bukkit: io/papermc/paper/scoreboard/numbers/NumberFormat.fixed
    pub fn fixed(component: &Component) -> Self {
        Self::Fixed(component.clone())
    }
    /// bukkit: io/papermc/paper/scoreboard/numbers/StyledFormat.style
    pub fn style(&self) -> Option<&Style> {
        match self {
            Self::Styled(style) => Some(style),
            _ => None,
        }
    }
    /// bukkit: io/papermc/paper/scoreboard/numbers/FixedFormat.component
    pub fn component(&self) -> Option<&Component> {
        match self {
            Self::Fixed(component) => Some(component),
            _ => None,
        }
    }
    /// What `/scoreboard objectives modify ... numberformat` takes.
    pub fn to_command(&self) -> String {
        match self {
            Self::Blank => "blank".to_owned(),
            Self::Styled(style) => format!("styled {}", snbt(&style.0)),
            Self::Fixed(component) => format!("fixed {}", snbt(&component.0)),
        }
    }
}

type ChatRender = dyn Fn(&Player, &Component, &Component, Option<&Player>) -> Component + Send + Sync;

/// `io.papermc.paper.chat.ChatRenderer$ViewerUnaware`: any closure of the three.
pub trait ViewerUnaware: Send + Sync + 'static {
    /// bukkit: io/papermc/paper/chat/ChatRenderer$ViewerUnaware.render
    fn render(&self, source: &Player, display_name: &Component, message: &Component) -> Component;
}

impl<F: Fn(&Player, &Component, &Component) -> Component + Send + Sync + 'static> ViewerUnaware for F {
    fn render(&self, source: &Player, display_name: &Component, message: &Component) -> Component {
        self(source, display_name, message)
    }
}

/// `io.papermc.paper.chat.ChatRenderer`: `viewer` is `None` for the console.
#[derive(Clone)]
pub struct ChatRenderer(Arc<ChatRender>);

impl ChatRenderer {
    pub fn new(render: impl Fn(&Player, &Component, &Component, Option<&Player>) -> Component + Send + Sync + 'static) -> Self {
        Self(Arc::new(render))
    }
    /// bukkit: io/papermc/paper/chat/ChatRenderer.render
    pub fn render(&self, source: &Player, display_name: &Component, message: &Component, viewer: Option<&Player>) -> Component {
        (self.0)(source, display_name, message, viewer)
    }
    /// bukkit: io/papermc/paper/chat/ChatRenderer.viewerUnaware
    pub fn viewer_unaware(render: impl ViewerUnaware) -> Self {
        Self::new(move |source, name, message, _| render.render(source, name, message))
    }
    /// bukkit: io/papermc/paper/chat/ChatRenderer.defaultRenderer
    pub fn default_renderer() -> Self {
        Self::new(|_, name, message, _| Component(json!({ "translate": "chat.type.text", "with": [name.0, message.0] })))
    }
}

/// `com.destroystokyo.paper.ClientOption`: pass `name` to `Player::get_client_option`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientOption {
    pub name: &'static str,
    kind: &'static str,
}

impl ClientOption {
    pub const SKIN_PARTS: Self = Self { name: "SKIN_PARTS", kind: "SkinParts" };
    pub const CHAT_COLORS_ENABLED: Self = Self { name: "CHAT_COLORS_ENABLED", kind: "Boolean" };
    pub const CHAT_VISIBILITY: Self = Self { name: "CHAT_VISIBILITY", kind: "ChatVisibility" };
    pub const LOCALE: Self = Self { name: "LOCALE", kind: "String" };
    pub const MAIN_HAND: Self = Self { name: "MAIN_HAND", kind: "MainHand" };
    pub const VIEW_DISTANCE: Self = Self { name: "VIEW_DISTANCE", kind: "Integer" };
    pub const TEXT_FILTERING_ENABLED: Self = Self { name: "TEXT_FILTERING_ENABLED", kind: "Boolean" };
    pub const ALLOW_SERVER_LISTINGS: Self = Self { name: "ALLOW_SERVER_LISTINGS", kind: "Boolean" };
    pub const PARTICLE_VISIBILITY: Self = Self { name: "PARTICLE_VISIBILITY", kind: "ParticleVisibility" };
    /// The name of the value's Java class.
    pub fn get_type(&self) -> &'static str {
        self.kind
    }
}

/// bukkit: com/destroystokyo/paper/ClientOption$ChatVisibility.translationKey
///
/// # Panics
/// As Java throws, for `UNKNOWN`.
pub fn chat_visibility_translation_key(visibility: &str) -> &'static str {
    match visibility {
        "FULL" => "options.chat.visibility.full",
        "SYSTEM" => "options.chat.visibility.system",
        "HIDDEN" => "options.chat.visibility.hidden",
        other => panic!("{other} doesn't have a translation key"),
    }
}

/// bukkit: com/destroystokyo/paper/ClientOption$ParticleVisibility.translationKey
pub fn particle_visibility_translation_key(visibility: &str) -> &'static str {
    match visibility {
        "DECREASED" => "options.particles.decreased",
        "MINIMAL" => "options.particles.minimal",
        _ => "options.particles.all",
    }
}

/// `com.destroystokyo.paper.SkinParts`, and `SkinParts$Mutable`: a copy to change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkinParts(pub i32);

impl SkinParts {
    fn has(self, bit: i32) -> bool {
        self.0 & bit != 0
    }
    fn set(&mut self, bit: i32, enabled: bool) {
        if enabled {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.allParts
    pub fn all_parts() -> Self {
        Self(0x7F)
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.getRaw
    pub fn get_raw(self) -> i32 {
        self.0
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.hasCapeEnabled
    pub fn has_cape_enabled(self) -> bool {
        self.has(1)
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.hasJacketEnabled
    pub fn has_jacket_enabled(self) -> bool {
        self.has(2)
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.hasLeftSleeveEnabled
    pub fn has_left_sleeve_enabled(self) -> bool {
        self.has(4)
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.hasRightSleeveEnabled
    pub fn has_right_sleeve_enabled(self) -> bool {
        self.has(8)
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.hasLeftPantsEnabled
    pub fn has_left_pants_enabled(self) -> bool {
        self.has(16)
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.hasRightPantsEnabled
    pub fn has_right_pants_enabled(self) -> bool {
        self.has(32)
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.hasHatsEnabled
    pub fn has_hats_enabled(self) -> bool {
        self.has(64)
    }
    /// bukkit: com/destroystokyo/paper/SkinParts.mutableCopy
    pub fn mutable_copy(self) -> Self {
        self
    }
    /// bukkit: com/destroystokyo/paper/SkinParts$Mutable.immutableCopy
    pub fn immutable_copy(self) -> Self {
        self
    }
    /// bukkit: com/destroystokyo/paper/SkinParts$Mutable.setCapeEnabled
    pub fn set_cape_enabled(&mut self, enabled: bool) {
        self.set(1, enabled);
    }
    /// bukkit: com/destroystokyo/paper/SkinParts$Mutable.setJacketEnabled
    pub fn set_jacket_enabled(&mut self, enabled: bool) {
        self.set(2, enabled);
    }
    /// bukkit: com/destroystokyo/paper/SkinParts$Mutable.setLeftSleeveEnabled
    pub fn set_left_sleeve_enabled(&mut self, enabled: bool) {
        self.set(4, enabled);
    }
    /// bukkit: com/destroystokyo/paper/SkinParts$Mutable.setRightSleeveEnabled
    pub fn set_right_sleeve_enabled(&mut self, enabled: bool) {
        self.set(8, enabled);
    }
    /// bukkit: com/destroystokyo/paper/SkinParts$Mutable.setLeftPantsEnabled
    pub fn set_left_pants_enabled(&mut self, enabled: bool) {
        self.set(16, enabled);
    }
    /// bukkit: com/destroystokyo/paper/SkinParts$Mutable.setRightPantsEnabled
    pub fn set_right_pants_enabled(&mut self, enabled: bool) {
        self.set(32, enabled);
    }
    /// bukkit: com/destroystokyo/paper/SkinParts$Mutable.setHatsEnabled
    pub fn set_hats_enabled(&mut self, enabled: bool) {
        self.set(64, enabled);
    }
}

impl Player {
    /// What `Player.getSkinParts` reads, from the client's own settings.
    pub fn get_skin_parts_value(&self) -> SkinParts {
        SkinParts(self.get_client_option("SKIN_PARTS").as_i64().unwrap_or(0x7F) as i32)
    }
}

/// `com.destroystokyo.paper.Title`.
#[derive(Clone, Debug, PartialEq)]
pub struct Title {
    title: Component,
    subtitle: Option<Component>,
    fade_in: i32,
    stay: i32,
    fade_out: i32,
}

impl Title {
    pub const DEFAULT_FADE_IN: i32 = 20;
    pub const DEFAULT_STAY: i32 = 200;
    pub const DEFAULT_FADE_OUT: i32 = 20;
    /// # Panics
    /// As Java throws, for a negative time.
    pub fn new(title: &Component, subtitle: Option<&Component>, fade_in: i32, stay: i32, fade_out: i32) -> Self {
        assert!(fade_in >= 0 && stay >= 0 && fade_out >= 0, "Negative time");
        Self { title: title.clone(), subtitle: subtitle.cloned(), fade_in, stay, fade_out }
    }
    /// bukkit: com/destroystokyo/paper/Title.getTitle
    pub fn get_title(&self) -> &Component {
        &self.title
    }
    /// bukkit: com/destroystokyo/paper/Title.getSubtitle
    pub fn get_subtitle(&self) -> Option<&Component> {
        self.subtitle.as_ref()
    }
    /// bukkit: com/destroystokyo/paper/Title.getFadeIn
    pub fn get_fade_in(&self) -> i32 {
        self.fade_in
    }
    /// bukkit: com/destroystokyo/paper/Title.getStay
    pub fn get_stay(&self) -> i32 {
        self.stay
    }
    /// bukkit: com/destroystokyo/paper/Title.getFadeOut
    pub fn get_fade_out(&self) -> i32 {
        self.fade_out
    }
    /// bukkit: com/destroystokyo/paper/Title.send
    pub fn send(&self, player: &Player) {
        let subtitle = self.subtitle.clone().unwrap_or_else(Component::empty);
        player.show_title(&self.title, &subtitle, self.fade_in as u32, self.stay as u32, self.fade_out as u32);
    }
    /// bukkit: com/destroystokyo/paper/Title.broadcast
    pub fn broadcast(&self) {
        crate::get_online_players().iter().for_each(|player| self.send(player));
    }
    /// bukkit: com/destroystokyo/paper/Title.builder
    pub fn builder() -> TitleBuilder {
        TitleBuilder { title: None, subtitle: None, fade_in: Self::DEFAULT_FADE_IN, stay: Self::DEFAULT_STAY, fade_out: Self::DEFAULT_FADE_OUT }
    }
}

/// `com.destroystokyo.paper.Title$Builder`.
#[derive(Clone, Debug, PartialEq)]
pub struct TitleBuilder {
    title: Option<Component>,
    subtitle: Option<Component>,
    fade_in: i32,
    stay: i32,
    fade_out: i32,
}

impl TitleBuilder {
    /// bukkit: com/destroystokyo/paper/Title$Builder.title
    pub fn title(mut self, title: &Component) -> Self {
        self.title = Some(title.clone());
        self
    }
    /// bukkit: com/destroystokyo/paper/Title$Builder.subtitle
    pub fn subtitle(mut self, subtitle: Option<&Component>) -> Self {
        self.subtitle = subtitle.cloned();
        self
    }
    /// bukkit: com/destroystokyo/paper/Title$Builder.fadeIn
    pub fn fade_in(mut self, ticks: i32) -> Self {
        self.fade_in = ticks;
        self
    }
    /// bukkit: com/destroystokyo/paper/Title$Builder.stay
    pub fn stay(mut self, ticks: i32) -> Self {
        self.stay = ticks;
        self
    }
    /// bukkit: com/destroystokyo/paper/Title$Builder.fadeOut
    pub fn fade_out(mut self, ticks: i32) -> Self {
        self.fade_out = ticks;
        self
    }
    /// bukkit: com/destroystokyo/paper/Title$Builder.build
    ///
    /// # Panics
    /// As Java throws, with no title set.
    pub fn build(self) -> Title {
        Title::new(&self.title.expect("Title not specified"), self.subtitle.as_ref(), self.fade_in, self.stay, self.fade_out)
    }
}

fn face_offset(face: &str) -> [i32; 3] {
    match face {
        "NORTH" => [0, 0, -1],
        "SOUTH" => [0, 0, 1],
        "EAST" => [1, 0, 0],
        "WEST" => [-1, 0, 0],
        "UP" => [0, 1, 0],
        "DOWN" => [0, -1, 0],
        _ => [0, 0, 0],
    }
}

/// `com.destroystokyo.paper.block.TargetBlockInfo`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetBlockInfo {
    block: Block,
    face: String,
}

impl TargetBlockInfo {
    pub fn new(block: &Block, face: &str) -> Self {
        Self { block: block.clone(), face: face.to_owned() }
    }
    /// bukkit: com/destroystokyo/paper/block/TargetBlockInfo.getBlock
    pub fn get_block(&self) -> &Block {
        &self.block
    }
    /// bukkit: com/destroystokyo/paper/block/TargetBlockInfo.getBlockFace
    pub fn get_block_face(&self) -> &str {
        &self.face
    }
    /// bukkit: com/destroystokyo/paper/block/TargetBlockInfo.getRelativeBlock
    pub fn get_relative_block(&self) -> Block {
        let [x, y, z] = face_offset(&self.face);
        self.block.get_relative(x, y, z)
    }
}

// ---- What the server reports going wrong ----

/// `com.destroystokyo.paper.exception.ServerPluginException`: a plugin's failure, as a value.
#[derive(Clone)]
pub struct ServerPluginException {
    pub message: String,
    plugin: Plugin,
}

impl ServerPluginException {
    pub fn new(message: &str, plugin: &Plugin) -> Self {
        Self { message: message.to_owned(), plugin: *plugin }
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerPluginException.getResponsiblePlugin
    pub fn get_responsible_plugin(&self) -> &Plugin {
        &self.plugin
    }
}

/// `com.destroystokyo.paper.exception.ServerEventException`: `L` is what a port calls its listener, `E` the event.
#[derive(Clone)]
pub struct ServerEventException<L, E> {
    pub base: ServerPluginException,
    listener: L,
    event: E,
}

impl<L, E> ServerEventException<L, E> {
    pub fn new(base: ServerPluginException, listener: L, event: E) -> Self {
        Self { base, listener, event }
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerEventException.getListener
    pub fn get_listener(&self) -> &L {
        &self.listener
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerEventException.getEvent
    pub fn get_event(&self) -> &E {
        &self.event
    }
}

/// `com.destroystokyo.paper.exception.ServerPluginMessageException`.
#[derive(Clone)]
pub struct ServerPluginMessageException {
    pub base: ServerPluginException,
    player: Player,
    channel: String,
    data: Vec<u8>,
}

impl ServerPluginMessageException {
    pub fn new(base: ServerPluginException, player: &Player, channel: &str, data: &[u8]) -> Self {
        Self { base, player: player.clone(), channel: channel.to_owned(), data: data.to_vec() }
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerPluginMessageException.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.player
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerPluginMessageException.getChannel
    pub fn get_channel(&self) -> &str {
        &self.channel
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerPluginMessageException.getData
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

/// `com.destroystokyo.paper.exception.ServerSchedulerException`.
#[derive(Clone)]
pub struct ServerSchedulerException {
    pub base: ServerPluginException,
    task: u32,
}

impl ServerSchedulerException {
    pub fn new(base: ServerPluginException, task: u32) -> Self {
        Self { base, task }
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerSchedulerException.getTask
    pub fn get_task(&self) -> u32 {
        self.task
    }
}

/// `com.destroystokyo.paper.exception.ServerCommandException`.
#[derive(Clone)]
pub struct ServerCommandException {
    pub message: String,
    command: crate::pure2::Command,
    sender: CommandSender,
    arguments: Vec<String>,
}

impl ServerCommandException {
    pub fn new(message: &str, command: &crate::pure2::Command, sender: &CommandSender, arguments: &[&str]) -> Self {
        Self { message: message.to_owned(), command: command.clone(), sender: sender.clone(), arguments: arguments.iter().map(|word| (*word).to_owned()).collect() }
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerCommandException.getCommand
    pub fn get_command(&self) -> &crate::pure2::Command {
        &self.command
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerCommandException.getCommandSender
    pub fn get_command_sender(&self) -> &CommandSender {
        &self.sender
    }
    /// bukkit: com/destroystokyo/paper/exception/ServerCommandException.getArguments
    pub fn get_arguments(&self) -> &[String] {
        &self.arguments
    }
}

// ---- The plugin itself ----

/// `io.papermc.paper.plugin.configuration.PluginMeta`, of this plugin: the rest is on `Plugin`.
#[derive(Clone, Copy)]
pub struct PluginMeta(pub Plugin);

impl PluginMeta {
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getName
    pub fn get_name(&self) -> &str {
        self.0.get_name()
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.namespace
    pub fn namespace(&self) -> String {
        self.0.get_name().to_lowercase()
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getProvidedPlugins
    pub fn get_provided_plugins(&self) -> Vec<String> {
        self.0.described_list("provides")
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getPermissionDefault
    pub fn get_permission_default(&self) -> String {
        self.0.described("default-permission").as_str().unwrap_or("op").to_ascii_uppercase().replace(' ', "_")
    }
    /// bukkit: io/papermc/paper/plugin/configuration/PluginMeta.getPermissions
    pub fn get_permissions(&self) -> Vec<Permission> {
        crate::DESCRIPTION.get().map_or_else(Vec::new, |description| permissions_of(description))
    }
}

impl Plugin {
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/LifecycleEventOwner.getPluginMeta
    pub fn get_plugin_meta(&self) -> PluginMeta {
        PluginMeta(*self)
    }
    /// A player's data, as `Player::get_persistent_data_container`.
    ///
    /// bukkit: io/papermc/paper/persistence/PersistentDataViewHolder.getPersistentDataContainer
    pub fn persistent_data_view(&self, player: &Player) -> PersistentDataContainer {
        self.persistent_data(player)
    }
}

/// `io.papermc.paper.plugin.provider.entrypoint.DependencyContext`, as this plugin's `plugin.yml` has it.
#[derive(Clone, Copy)]
pub struct DependencyContext(pub Plugin);

impl DependencyContext {
    /// bukkit: io/papermc/paper/plugin/provider/entrypoint/DependencyContext.hasDependency
    ///
    /// Whether it's named in `plugin.yml` and loaded.
    pub fn has_dependency(&self, name: &str) -> bool {
        self.0.get_depend().iter().chain(&self.0.get_soft_depend()).any(|known| known == name) && crate::lead2::get_plugins().iter().any(|loaded| loaded.eq_ignore_ascii_case(name))
    }
    /// bukkit: io/papermc/paper/plugin/provider/entrypoint/DependencyContext.isTransitiveDependency
    ///
    /// Through this plugin's own `plugin.yml`: `plugin` is this one, or it depends on this one.
    pub fn is_transitive_dependency(&self, depend: &str) -> bool {
        self.0.get_depend().iter().chain(&self.0.get_soft_depend()).any(|known| known == depend)
    }
}

/// `io.papermc.paper.plugin.lifecycle.event.LifecycleEventManager`: a handler is held and runs at its
/// moment: data packs' handlers before a world reads its packs, commands' as the server starts, and those
/// that reload again at each `/reload`.
#[derive(Clone, Copy)]
pub struct LifecycleEventManager {
    plugin: Plugin,
}

static LIFE_NUMBER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// `INITIAL` or `RELOAD`, as a moment's event says.
fn life_cause(value: &Value) -> &'static str {
    if value["cause"] == "RELOAD" { "RELOAD" } else { "INITIAL" }
}

/// `handler` at each of `moments`, in priority order among every plugin's: a plugin that came after the
/// moment gets it on the next tick.
fn hold_handler(plugin: Plugin, moments: &[&str], priority: i32, monitor: bool, handler: Arc<dyn Fn(&'static str) + Send + Sync>) {
    let slot = if monitor { u8::MAX } else { (priority.clamp(-127, 126) + 128) as u8 };
    let mut late = false;
    for moment in moments {
        let number = LIFE_NUMBER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let run = handler.clone();
        plugin.host.on(&format!("{moment}#{slot}#{number}#-#{}", plugin.get_name()), move |value| {
            run(life_cause(value));
            horizon_mod::Answer::Pass
        });
        late |= query(json!({ "op": "ev_world_free_life_passed", "moment": moment })) == true;
    }
    if late {
        scheduler::run_task(move || handler("INITIAL"));
    }
}

/// `io.papermc.paper.plugin.lifecycle.event.handler.configuration.LifecycleEventHandlerConfiguration`:
/// what `LifecycleEventManager.registerEventHandler` takes.
pub trait LifecycleEventHandlerConfiguration {
    /// Held for its moment.
    fn hold(self, plugin: Plugin);
}

impl LifecycleEventManager {
    pub fn of(plugin: &Plugin) -> Self {
        Self { plugin: *plugin }
    }
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/LifecycleEventManager.registerEventHandler
    pub fn register_event_handler(&self, configuration: impl LifecycleEventHandlerConfiguration) {
        configuration.hold(self.plugin);
    }
    /// `LifecycleEvents.COMMANDS`.
    pub fn register_commands(&self, handler: impl Fn(&Commands) + Send + Sync + 'static) {
        self.register_event_handler(LifecycleEvents::commands().new_handler(move |event| handler(event.registrar())));
    }
    /// `LifecycleEvents.TAGS.preFlatten(key)`.
    pub fn register_pre_flatten_tags(&self, registry: RegistryKey, handler: impl Fn(&PreFlattenTagRegistrar) + Send + Sync + 'static) {
        self.register_event_handler(LifecycleEvents::tags_pre_flatten(registry).new_handler(move |event| handler(event.registrar())));
    }
    /// `LifecycleEvents.TAGS.postFlatten(key)`.
    pub fn register_post_flatten_tags(&self, registry: RegistryKey, handler: impl Fn(&PostFlattenTagRegistrar) + Send + Sync + 'static) {
        self.register_event_handler(LifecycleEvents::tags_post_flatten(registry).new_handler(move |event| handler(event.registrar())));
    }
    /// `LifecycleEvents.DATAPACK_DISCOVERY`.
    pub fn register_datapack_discovery(&self, handler: impl Fn(&DatapackRegistrar) + Send + Sync + 'static) {
        self.register_event_handler(LifecycleEvents::datapack_discovery().new_handler(move |event| handler(event.registrar())));
    }
}

/// `io.papermc.paper.plugin.lifecycle.event.registrar.RegistrarEvent`, and `ReloadableRegistrarEvent`.
pub struct RegistrarEvent<R> {
    registrar: R,
    cause: &'static str,
}

impl<R> RegistrarEvent<R> {
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/registrar/RegistrarEvent.registrar
    pub fn registrar(&self) -> &R {
        &self.registrar
    }
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/registrar/ReloadableRegistrarEvent.cause
    pub fn cause(&self) -> &'static str {
        self.cause
    }
}

/// `io.papermc.paper.plugin.lifecycle.event.types.LifecycleEventType`.
pub struct LifecycleEventType<E> {
    name: String,
    moments: &'static [&'static str],
    make: Arc<dyn Fn(Plugin, &'static str) -> E + Send + Sync>,
}

impl<E: 'static> LifecycleEventType<E> {
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/types/LifecycleEventType.name
    pub fn name(&self) -> &str {
        &self.name
    }
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/types/LifecycleEventType.newHandler
    pub fn new_handler(&self, handler: impl Fn(&mut E) + Send + Sync + 'static) -> PrioritizedLifecycleEventHandlerConfiguration<E> {
        PrioritizedLifecycleEventHandlerConfiguration { moments: self.moments, make: self.make.clone(), handler: Arc::new(handler), priority: 0, monitor: false }
    }
}

/// `io.papermc.paper.plugin.lifecycle.event.handler.configuration.PrioritizedLifecycleEventHandlerConfiguration`,
/// and `MonitorLifecycleEventHandlerConfiguration`.
pub struct PrioritizedLifecycleEventHandlerConfiguration<E> {
    moments: &'static [&'static str],
    make: Arc<dyn Fn(Plugin, &'static str) -> E + Send + Sync>,
    handler: Arc<dyn Fn(&mut E) + Send + Sync>,
    priority: i32,
    monitor: bool,
}

impl<E> PrioritizedLifecycleEventHandlerConfiguration<E> {
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/handler/configuration/PrioritizedLifecycleEventHandlerConfiguration.priority
    ///
    /// Lower runs first, across every plugin's.
    #[must_use]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/handler/configuration/PrioritizedLifecycleEventHandlerConfiguration.monitor, io/papermc/paper/plugin/lifecycle/event/handler/configuration/MonitorLifecycleEventHandlerConfiguration.monitor
    #[must_use]
    pub fn monitor(mut self) -> Self {
        self.monitor = true;
        self
    }
}

impl<E: 'static> LifecycleEventHandlerConfiguration for PrioritizedLifecycleEventHandlerConfiguration<E> {
    fn hold(self, plugin: Plugin) {
        let (make, handler) = (self.make, self.handler);
        hold_handler(plugin, self.moments, self.priority, self.monitor, Arc::new(move |cause: &'static str| handler(&mut make(plugin, cause))));
    }
}

/// `io.papermc.paper.plugin.lifecycle.event.types.LifecycleEvents`.
#[derive(Clone, Copy, Debug, Default)]
pub struct LifecycleEvents;

impl LifecycleEvents {
    /// `LifecycleEvents.COMMANDS`.
    pub fn commands() -> LifecycleEventType<RegistrarEvent<Commands>> {
        LifecycleEventType { name: "commands".to_owned(), moments: &["ew_life_commands"], make: Arc::new(|plugin: Plugin, cause: &'static str| RegistrarEvent { registrar: Commands::of(&plugin), cause }) }
    }
    /// `LifecycleEvents.TAGS.preFlatten(key)`.
    pub fn tags_pre_flatten(registry: RegistryKey) -> LifecycleEventType<RegistrarEvent<PreFlattenTagRegistrar>> {
        LifecycleEventType { name: format!("tags_pre_flatten/{}", registry.0), moments: &["ew_life_tags_pre"], make: Arc::new(move |_: Plugin, cause: &'static str| RegistrarEvent { registrar: PreFlattenTagRegistrar(registry), cause }) }
    }
    /// `LifecycleEvents.TAGS.postFlatten(key)`.
    pub fn tags_post_flatten(registry: RegistryKey) -> LifecycleEventType<RegistrarEvent<PostFlattenTagRegistrar>> {
        LifecycleEventType { name: format!("tags_post_flatten/{}", registry.0), moments: &["ew_life_tags_post"], make: Arc::new(move |_: Plugin, cause: &'static str| RegistrarEvent { registrar: PostFlattenTagRegistrar(registry), cause }) }
    }
    /// `LifecycleEvents.DATAPACK_DISCOVERY`.
    pub fn datapack_discovery() -> LifecycleEventType<RegistrarEvent<DatapackRegistrar>> {
        LifecycleEventType { name: "datapack_discovery".to_owned(), moments: &["ew_life_discovery"], make: Arc::new(|plugin: Plugin, cause: &'static str| RegistrarEvent { registrar: DatapackRegistrar { plugin }, cause }) }
    }
}

// ---- The registries' events ----

/// A registry entry's builder, which the registry events hand a plugin.
pub trait RegistryBuilder: Default + Send + 'static {
    /// The registry it builds for.
    const REGISTRY: RegistryKey;
    /// A builder with these fields.
    fn with_fields(fields: Map<String, Value>) -> Self;
    /// The entry's fields so far.
    fn fields(&self) -> &Map<String, Value>;
    /// Into the plugins' data pack.
    fn write(self, key: &NamespacedKey) -> bool;
}

macro_rules! registry_builders {
    ($($builder:ident = $path:literal,)*) => {
        $(impl RegistryBuilder for crate::registry_data::$builder {
            const REGISTRY: RegistryKey = RegistryKey($path);
            fn with_fields(fields: Map<String, Value>) -> Self {
                Self(fields)
            }
            fn fields(&self) -> &Map<String, Value> {
                &self.0
            }
            fn write(self, key: &NamespacedKey) -> bool {
                self.register(key)
            }
        })*
    };
}

registry_builders! {
    BannerPatternRegistryEntryBuilder = "banner_pattern", CatTypeRegistryEntryBuilder = "cat_variant",
    ChickenVariantRegistryEntryBuilder = "chicken_variant", CowVariantRegistryEntryBuilder = "cow_variant",
    DamageTypeRegistryEntryBuilder = "damage_type", DecoratedPotPatternRegistryEntryBuilder = "decorated_pot_pattern",
    EnchantmentRegistryEntryBuilder = "enchantment", FrogVariantRegistryEntryBuilder = "frog_variant",
    InstrumentRegistryEntryBuilder = "instrument", JukeboxSongRegistryEntryBuilder = "jukebox_song",
    PaintingVariantRegistryEntryBuilder = "painting_variant", PigVariantRegistryEntryBuilder = "pig_variant",
    SulfurCubeArchetypeRegistryEntryBuilder = "sulfur_cube_archetype", TrimMaterialRegistryEntryBuilder = "trim_material",
    TrimPatternRegistryEntryBuilder = "trim_pattern", WolfVariantRegistryEntryBuilder = "wolf_variant",
    ZombieNautilusVariantRegistryEntryBuilder = "zombie_nautilus_variant",
}

/// The entry as it stands: the plugins' data pack over the game's own.
fn current_entry(registry: RegistryKey, key: &NamespacedKey) -> Value {
    let file = crate::fam_core::world_folder().join("datapacks/plugin-registry/data").join(key.get_namespace()).join(registry.0).join(format!("{}.json", key.get_key()));
    match std::fs::read(file).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()) {
        Some(entry) => entry,
        None => entry_file(registry, key),
    }
}

/// The keys of a registry's entries as they stand: the game's own and the plugins'.
fn entry_keys(registry: RegistryKey) -> Vec<NamespacedKey> {
    let mut keys: Vec<NamespacedKey> = query(json!({ "op": "ev_world_free_data_list", "path": registry.0 })).as_array().into_iter().flatten().filter_map(|name| NamespacedKey::from_string(&format!("minecraft:{}", name.as_str()?))).collect();
    let root = crate::fam_core::world_folder().join("datapacks/plugin-registry/data");
    for space in std::fs::read_dir(root).into_iter().flatten().flatten() {
        let namespace = space.file_name().to_string_lossy().into_owned();
        for entry in std::fs::read_dir(space.path().join(registry.0)).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(key) = name.strip_suffix(".json").and_then(|name| NamespacedKey::from_string(&format!("{namespace}:{name}"))) {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
    }
    keys
}

/// What's in a tag before the game has read it: the plugins' data pack and the game's own file.
fn tag_members(key: &TagKey, depth: u32, into: &mut Vec<RegistryTypedKey>) {
    let own = query(json!({ "op": "ev_world_free_data_file", "path": format!("tags/{}/{}", key.registry.tag_folder(), key.key.get_key()) }));
    let written = std::fs::read(tag_path(key)).ok().and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok()).unwrap_or_default();
    let mut names: Vec<String> = Vec::new();
    for file in [&own, &written] {
        for value in file["values"].as_array().into_iter().flatten() {
            if let Some(name) = value.as_str().or(value["id"].as_str()) {
                names.push(name.to_owned());
            }
        }
    }
    for name in names {
        match name.strip_prefix('#') {
            Some(tag) if depth < 8 => {
                if let Some(inner) = NamespacedKey::from_string(tag) {
                    tag_members(&TagKey { registry: key.registry, key: inner }, depth + 1, into);
                }
            }
            Some(_) => {}
            None => {
                if let Some(entry) = NamespacedKey::from_string(&name) {
                    let typed = key.registry.typed_key(&entry);
                    if !into.contains(&typed) {
                        into.push(typed);
                    }
                }
            }
        }
    }
}

/// `getOrCreateTag`: the tag, made empty if nothing has it yet. The registries aren't up, so it's read from the files.
fn tag_of_files(key: &TagKey) -> RegistryTag {
    let mut members = Vec::new();
    tag_members(key, 0, &mut members);
    if members.is_empty() && !tag_path(key).is_file() {
        write_tag(key, &[], false);
    }
    RegistryTag { key: key.clone(), set: RegistryKeySet { registry: key.registry, keys: members } }
}

/// `io.papermc.paper.registry.event.WritableRegistry`: an entry goes into the plugins' data pack, which the
/// game reads as the world opens.
pub struct WritableRegistry<B> {
    marker: std::marker::PhantomData<B>,
}

impl<B: RegistryBuilder> WritableRegistry<B> {
    /// bukkit: io/papermc/paper/registry/event/WritableRegistry.register
    pub fn register(&self, key: &RegistryTypedKey, value: impl FnOnce(B) -> B) -> bool {
        value(B::default()).write(&key.key)
    }
    /// bukkit: io/papermc/paper/registry/event/WritableRegistry.registerWith
    pub fn register_with(&self, key: &RegistryTypedKey, value: impl FnOnce(&RegistryBuilderFactory<B>) -> B) -> bool {
        value(&RegistryBuilderFactory::new(B::REGISTRY, B::with_fields)).write(&key.key)
    }
}

/// `io.papermc.paper.registry.event.RegistryComposeEvent`, with `RegistryEvent`.
pub struct RegistryComposeEvent<B> {
    registry: WritableRegistry<B>,
}

impl<B: RegistryBuilder> RegistryComposeEvent<B> {
    /// bukkit: io/papermc/paper/registry/event/RegistryEvent.registryKey
    pub fn registry_key(&self) -> RegistryKey {
        B::REGISTRY
    }
    /// bukkit: io/papermc/paper/registry/event/RegistryComposeEvent.registry
    pub fn registry(&self) -> &WritableRegistry<B> {
        &self.registry
    }
    /// bukkit: io/papermc/paper/registry/event/RegistryComposeEvent.getOrCreateTag
    pub fn get_or_create_tag(&self, key: &TagKey) -> RegistryTag {
        tag_of_files(key)
    }
}

/// `io.papermc.paper.registry.event.RegistryEntryAddEvent`.
pub struct RegistryEntryAddEvent<B> {
    key: RegistryTypedKey,
    builder: B,
}

impl<B: RegistryBuilder> RegistryEntryAddEvent<B> {
    /// bukkit: io/papermc/paper/registry/event/RegistryEntryAddEvent.key
    pub fn key(&self) -> &RegistryTypedKey {
        &self.key
    }
    /// bukkit: io/papermc/paper/registry/event/RegistryEntryAddEvent.builder
    pub fn builder(&mut self) -> &mut B {
        &mut self.builder
    }
    /// The builder's own calls, which take it and give it back.
    pub fn change(&mut self, change: impl FnOnce(B) -> B) {
        self.builder = change(std::mem::take(&mut self.builder));
    }
    /// bukkit: io/papermc/paper/registry/event/RegistryEntryAddEvent.getOrCreateTag
    pub fn get_or_create_tag(&self, key: &TagKey) -> RegistryTag {
        tag_of_files(key)
    }
    /// The registry the entry is in.
    pub fn registry_key(&self) -> RegistryKey {
        B::REGISTRY
    }
}

/// `io.papermc.paper.registry.event.RegistryEventProvider`.
pub struct RegistryEventProvider<B> {
    marker: std::marker::PhantomData<B>,
}

impl<B: RegistryBuilder> Default for RegistryEventProvider<B> {
    fn default() -> Self {
        Self { marker: std::marker::PhantomData }
    }
}

impl<B: RegistryBuilder> RegistryEventProvider<B> {
    /// The registry's events: `RegistryEvents.ENCHANTMENT` is `RegistryEventProvider::<EnchantmentRegistryEntryBuilder>::default()`.
    pub fn registry_key(&self) -> RegistryKey {
        B::REGISTRY
    }
    pub fn compose(&self) -> LifecycleEventType<RegistryComposeEvent<B>> {
        LifecycleEventType { name: format!("registry_compose/{}", B::REGISTRY.0), moments: &["ew_life_compose"], make: Arc::new(|_: Plugin, _: &'static str| RegistryComposeEvent { registry: WritableRegistry { marker: std::marker::PhantomData } }) }
    }
    /// What `RegistryEntryAddConfiguration`s are made from.
    pub fn entry_add(&self) -> RegistryEntryAddType<B> {
        RegistryEntryAddType { marker: std::marker::PhantomData }
    }
}

/// `io.papermc.paper.registry.event.type.RegistryEntryAddEventType`.
pub struct RegistryEntryAddType<B> {
    marker: std::marker::PhantomData<B>,
}

impl<B: RegistryBuilder> RegistryEntryAddType<B> {
    /// A handler for each entry of the registry, the game's own and plugins'.
    pub fn new_handler(&self, handler: impl Fn(&mut RegistryEntryAddEvent<B>) + Send + Sync + 'static) -> RegistryEntryAddConfiguration<B> {
        RegistryEntryAddConfiguration { handler: Arc::new(handler), filter: None, priority: 0, monitor: false }
    }
}

/// `io.papermc.paper.registry.event.type.RegistryEntryAddConfiguration`.
pub struct RegistryEntryAddConfiguration<B> {
    handler: Arc<dyn Fn(&mut RegistryEntryAddEvent<B>) + Send + Sync>,
    filter: Option<Arc<dyn Fn(&RegistryTypedKey) -> bool + Send + Sync>>,
    priority: i32,
    monitor: bool,
}

impl<B: RegistryBuilder> RegistryEntryAddConfiguration<B> {
    /// bukkit: io/papermc/paper/registry/event/type/RegistryEntryAddConfiguration.priority
    #[must_use]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    /// bukkit: io/papermc/paper/registry/event/type/RegistryEntryAddConfiguration.monitor
    #[must_use]
    pub fn monitor(mut self) -> Self {
        self.monitor = true;
        self
    }
    /// bukkit: io/papermc/paper/registry/event/type/RegistryEntryAddConfiguration.filter
    #[must_use]
    pub fn filter(mut self, filter: impl Fn(&RegistryTypedKey) -> bool + Send + Sync + 'static) -> Self {
        self.filter = Some(Arc::new(filter));
        self
    }
    /// `filter`, of one key.
    #[must_use]
    pub fn filter_key(self, key: &RegistryTypedKey) -> Self {
        let key = key.clone();
        self.filter(move |seen| *seen == key)
    }
}

impl<B: RegistryBuilder> LifecycleEventHandlerConfiguration for RegistryEntryAddConfiguration<B> {
    fn hold(self, plugin: Plugin) {
        let (handler, filter) = (self.handler, self.filter);
        hold_handler(plugin, &["ew_life_entries"], self.priority, self.monitor, Arc::new(move |_: &'static str| {
            for name in entry_keys(B::REGISTRY) {
                let key = B::REGISTRY.typed_key(&name);
                if filter.as_ref().is_some_and(|filter| !filter(&key)) {
                    continue;
                }
                let fields = current_entry(B::REGISTRY, &name).as_object().cloned().unwrap_or_default();
                let mut event = RegistryEntryAddEvent { key, builder: B::with_fields(fields.clone()) };
                handler(&mut event);
                if *event.builder.fields() != fields {
                    event.builder.write(&name);
                }
            }
        }));
    }
}

/// `io.papermc.paper.plugin.bootstrap.BootstrapContext`.
#[derive(Clone, Copy)]
pub struct BootstrapContext {
    plugin: Plugin,
}

impl BootstrapContext {
    /// bukkit: io/papermc/paper/plugin/bootstrap/BootstrapContext.getLifecycleManager
    pub fn get_lifecycle_manager(&self) -> LifecycleEventManager {
        LifecycleEventManager::of(&self.plugin)
    }
    pub fn get_configuration(&self) -> PluginMeta {
        PluginMeta(self.plugin)
    }
    pub fn get_data_directory(&self) -> PathBuf {
        self.plugin.get_data_folder()
    }
}

/// `io.papermc.paper.plugin.bootstrap.PluginBootstrap`: a Rust plugin calls `run_bootstrap` from its
/// `init`, before `on_enable`, which is when Paper would.
pub trait PluginBootstrap {
    /// bukkit: io/papermc/paper/plugin/bootstrap/PluginBootstrap.bootstrap
    fn bootstrap(&self, context: &BootstrapContext);
    /// bukkit: io/papermc/paper/plugin/bootstrap/PluginBootstrap.createPlugin
    fn create_plugin(&self, context: &BootstrapContext) -> Plugin {
        context.plugin
    }
}

/// What Paper does with a `PluginBootstrap`: bootstraps it, then makes the plugin.
pub fn run_bootstrap(plugin: &Plugin, bootstrap: &impl PluginBootstrap) -> Plugin {
    let context = BootstrapContext { plugin: *plugin };
    bootstrap.bootstrap(&context);
    bootstrap.create_plugin(&context)
}

static PERMISSIONS: Mutex<BTreeMap<String, Permission>> = Mutex::new(BTreeMap::new());
static SUBSCRIBERS: Mutex<BTreeMap<String, Vec<Player>>> = Mutex::new(BTreeMap::new());
static DEFAULT_SUBSCRIBERS: Mutex<[Vec<Player>; 2]> = Mutex::new([Vec::new(), Vec::new()]);

fn locked<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

static DECLARED: std::sync::Once = std::sync::Once::new();

/// The manager's permissions, starting with what `plugin.yml` declares.
fn known_permissions() -> std::sync::MutexGuard<'static, BTreeMap<String, Permission>> {
    DECLARED.call_once(|| {
        let declared: Vec<Permission> = crate::DESCRIPTION.get().map_or_else(Vec::new, |description| permissions_of(description));
        let mut all = locked(&PERMISSIONS);
        for permission in declared {
            all.entry(permission.name.to_lowercase()).or_insert(permission);
        }
    });
    locked(&PERMISSIONS)
}

fn permissions_of(description: &Value) -> Vec<Permission> {
    let fallback = description["default-permission"].as_str().unwrap_or("op").to_ascii_uppercase().replace(' ', "_");
    let mut out = Vec::new();
    for (name, value) in description["permissions"].as_object().into_iter().flatten() {
        let mut permission = Permission::new(name, value["default"].as_str().unwrap_or(&fallback), value["description"].as_str().unwrap_or_default());
        for (child, on) in value["children"].as_object().into_iter().flatten() {
            permission.children.insert(child.clone(), on.as_bool().unwrap_or(true));
        }
        out.push(permission);
    }
    out
}

/// `io.papermc.paper.plugin.PermissionManager`: over this plugin's `plugin.yml` and what it adds in code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PermissionManager;

impl PermissionManager {
    /// bukkit: io/papermc/paper/plugin/PermissionManager.getPermission
    pub fn get_permission(&self, name: &str) -> Option<Permission> {
        known_permissions().get(&name.to_lowercase()).cloned()
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.addPermission
    ///
    /// # Panics
    /// As Java throws, for a name that's taken.
    pub fn add_permission(&self, permission: &Permission) {
        let previous = known_permissions().insert(permission.name.to_lowercase(), permission.clone());
        assert!(previous.is_none(), "The permission {} is already defined!", permission.name);
        crate::set_permission_default(&permission.name, &permission.default);
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.addPermissions
    pub fn add_permissions(&self, permissions: &[Permission]) {
        permissions.iter().for_each(|permission| self.add_permission(permission));
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.removePermission
    pub fn remove_permission(&self, name: &str) {
        known_permissions().remove(&name.to_lowercase());
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.getDefaultPermissions
    pub fn get_default_permissions(&self, op: bool) -> Vec<Permission> {
        known_permissions().values().filter(|permission| crate::config2::permission_default_value(&permission.default, op)).cloned().collect()
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.recalculatePermissionDefaults
    pub fn recalculate_permission_defaults(&self, permission: &Permission) {
        crate::set_permission_default(&permission.name, &permission.default);
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.subscribeToPermission
    pub fn subscribe_to_permission(&self, permission: &str, player: &Player) {
        let mut all = locked(&SUBSCRIBERS);
        let list = all.entry(permission.to_lowercase()).or_default();
        if !list.contains(player) {
            list.push(player.clone());
        }
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.unsubscribeFromPermission
    pub fn unsubscribe_from_permission(&self, permission: &str, player: &Player) {
        if let Some(list) = locked(&SUBSCRIBERS).get_mut(&permission.to_lowercase()) {
            list.retain(|known| known != player);
        }
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.getPermissionSubscriptions
    pub fn get_permission_subscriptions(&self, permission: &str) -> Vec<Player> {
        locked(&SUBSCRIBERS).get(&permission.to_lowercase()).cloned().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.subscribeToDefaultPerms
    pub fn subscribe_to_default_perms(&self, op: bool, player: &Player) {
        let mut all = locked(&DEFAULT_SUBSCRIBERS);
        let list = &mut all[usize::from(op)];
        if !list.contains(player) {
            list.push(player.clone());
        }
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.unsubscribeFromDefaultPerms
    pub fn unsubscribe_from_default_perms(&self, op: bool, player: &Player) {
        locked(&DEFAULT_SUBSCRIBERS)[usize::from(op)].retain(|known| known != player);
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.getDefaultPermSubscriptions
    pub fn get_default_perm_subscriptions(&self, op: bool) -> Vec<Player> {
        locked(&DEFAULT_SUBSCRIBERS)[usize::from(op)].clone()
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.getPermissions
    pub fn get_permissions(&self) -> Vec<Permission> {
        known_permissions().values().cloned().collect()
    }
    /// bukkit: io/papermc/paper/plugin/PermissionManager.clearPermissions
    pub fn clear_permissions(&self) {
        known_permissions().clear();
        locked(&SUBSCRIBERS).clear();
        *locked(&DEFAULT_SUBSCRIBERS) = [Vec::new(), Vec::new()];
    }
}

// ---- Persistent data ----

/// `io.papermc.paper.persistence.PersistentDataKey`: `kind` is the data type's name, as `STRING`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersistentDataKey {
    key: NamespacedKey,
    kind: String,
}

impl PersistentDataKey {
    /// bukkit: io/papermc/paper/persistence/PersistentDataKey.of
    pub fn of(key: &NamespacedKey, kind: &str) -> Self {
        Self { key: key.clone(), kind: kind.to_owned() }
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataKey.key
    pub fn key(&self) -> &NamespacedKey {
        &self.key
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataKey.dataType
    pub fn data_type(&self) -> &str {
        &self.kind
    }
}

impl PersistentDataContainer {
    fn holder_values(&self) -> Map<String, Value> {
        self.values()
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataContainerView.getOrDefault
    pub fn get_or_default(&self, key: &NamespacedKey, default: Value) -> Value {
        self.get(key).unwrap_or(default)
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataContainerView.getKeys
    pub fn get_keys(&self) -> Vec<NamespacedKey> {
        self.holder_values().keys().filter_map(|key| NamespacedKey::from_string(key)).collect()
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataContainerView.getSize
    pub fn get_size(&self) -> usize {
        self.holder_values().len()
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataContainerView.isEmpty
    pub fn is_empty(&self) -> bool {
        self.holder_values().is_empty()
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataContainerView.copyTo
    pub fn copy_to(&self, other: &Self, replace: bool) {
        let kinds = self.kinds();
        for (key, value) in self.holder_values() {
            let kind = kinds.get(&key).cloned().unwrap_or(Value::Null);
            if let Some(key) = NamespacedKey::from_string(&key)
                && (replace || !other.has(&key))
            {
                other.set_kind(&key, value, kind);
            }
        }
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataContainerView.serializeToBytes
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let tree = crate::persistence::typed_snbt(&Value::Object(self.holder_values()), &Value::Object(self.kinds()));
        let bytes = query(json!({ "op": "paper_nbt_bytes", "snbt": tree }));
        bytes.as_array().into_iter().flatten().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect()
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataContainerView.getPrimitiveStorageType
    ///
    /// The name of the `PersistentDataType` constant, as Java's does by the tag: `None` for an empty list.
    pub fn get_primitive_storage_type(&self, key: &NamespacedKey) -> Option<&'static str> {
        let value = self.get(key)?;
        let kind = self.kinds().remove(key.as_str()).unwrap_or_else(|| crate::persistence::natural_kind(&value));
        Some(match &kind {
            Value::String(letter) => match letter.as_str() {
                "b" => "BYTE",
                "s" => "SHORT",
                "i" => "INTEGER",
                "l" => "LONG",
                "f" => "FLOAT",
                "d" => "DOUBLE",
                "B" => "BYTE_ARRAY",
                "I" => "INTEGER_ARRAY",
                "L" => "LONG_ARRAY",
                _ => "STRING",
            },
            Value::Array(items) if items.is_empty() => return None,
            Value::Array(_) => "LIST",
            _ => "TAG_CONTAINER",
        })
    }
    /// bukkit: io/papermc/paper/persistence/PersistentDataContainerView.getAdapterContext
    pub fn get_adapter_context(&self) -> crate::fam_core::PersistentDataAdapterContext {
        crate::fam_core::PersistentDataAdapterContext
    }
}

// ---- Tags ----

/// `io.papermc.paper.tag.BaseTag`: a set of a registry's entries, by id. `MaterialSetTag` and `EntitySetTag`
/// are this over items and blocks, and over entity types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseTag {
    key: NamespacedKey,
    registries: Vec<RegistryKey>,
    tagged: BTreeSet<String>,
    locked: bool,
}

/// `com.destroystokyo.paper.MaterialSetTag`.
pub type MaterialSetTag = BaseTag;

impl BaseTag {
    /// `new MaterialSetTag(key, values)`: items and blocks both.
    pub fn material(key: &NamespacedKey, values: &[Material]) -> Self {
        Self { key: key.clone(), registries: vec![RegistryKey::ITEM, RegistryKey::BLOCK], tagged: values.iter().map(|material| material.key().to_owned()).collect(), locked: false }
    }
    /// `new EntitySetTag(key, values)`: `values` are `minecraft:zombie` and the like.
    pub fn entity(key: &NamespacedKey, values: &[&str]) -> Self {
        Self { key: key.clone(), registries: vec![RegistryKey::ENTITY_TYPE], tagged: values.iter().map(|value| (*value).to_owned()).collect(), locked: false }
    }
    fn every_value(&self) -> BTreeSet<String> {
        self.registries.iter().flat_map(|registry| GameRegistry(*registry).keys()).collect()
    }
    fn check_lock(&self) {
        assert!(!self.locked, "Tag ({}) is locked", self.key.as_str());
    }
    fn matching(&self, test: impl Fn(&str) -> bool) -> Vec<String> {
        self.every_value().into_iter().filter(|name| test(name.trim_start_matches("minecraft:"))).collect()
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.lock
    pub fn lock(mut self) -> Self {
        self.locked = true;
        self
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.isLocked
    pub fn is_locked(&self) -> bool {
        self.locked
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.getKey
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.getValues
    pub fn get_values(&self) -> &BTreeSet<String> {
        &self.tagged
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.isTagged
    pub fn is_tagged(&self, id: &str) -> bool {
        self.tagged.contains(id)
    }
    /// bukkit: com/destroystokyo/paper/MaterialSetTag.isTagged
    pub fn is_tagged_block(&self, block: &Block) -> bool {
        self.is_tagged(block.get_type().key())
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.add
    ///
    /// # Panics
    /// As Java throws, on a locked tag.
    pub fn add(mut self, values: &[&str]) -> Self {
        self.check_lock();
        self.tagged.extend(values.iter().map(|value| (*value).to_owned()));
        self
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.add
    ///
    /// # Panics
    /// As Java throws, on a locked tag.
    pub fn add_tag(mut self, tag: &crate::registry::Tag) -> Self {
        self.check_lock();
        self.tagged.extend(tag.get_values());
        self
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.add
    ///
    /// # Panics
    /// As Java throws, on a locked tag.
    pub fn add_filter(mut self, filter: impl Fn(&str) -> bool) -> Self {
        self.check_lock();
        let found = self.matching(filter);
        self.tagged.extend(found);
        self
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.contains
    pub fn contains(self, with: &str) -> Self {
        self.add_filter(|name| name.contains(with))
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.endsWith
    pub fn ends_with(self, with: &str) -> Self {
        self.add_filter(|name| name.ends_with(with))
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.startsWith
    pub fn starts_with(self, with: &str) -> Self {
        self.add_filter(|name| name.starts_with(with))
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.not
    ///
    /// # Panics
    /// As Java throws, on a locked tag.
    pub fn not(mut self, values: &[&str]) -> Self {
        self.check_lock();
        for value in values {
            self.tagged.remove(*value);
        }
        self
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.not
    ///
    /// # Panics
    /// As Java throws, on a locked tag.
    pub fn not_filter(mut self, filter: impl Fn(&str) -> bool) -> Self {
        self.check_lock();
        for name in self.matching(filter) {
            self.tagged.remove(&name);
        }
        self
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.notContains
    pub fn not_contains(self, with: &str) -> Self {
        self.not_filter(|name| name.contains(with))
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.notEndsWith
    pub fn not_ends_with(self, with: &str) -> Self {
        self.not_filter(|name| name.ends_with(with))
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.notStartsWith
    pub fn not_starts_with(self, with: &str) -> Self {
        self.not_filter(|name| name.starts_with(with))
    }
    /// bukkit: io/papermc/paper/tag/BaseTag.ensureSize
    ///
    /// # Panics
    /// As Java throws, for another size.
    pub fn ensure_size(self, label: &str, size: usize) -> Self {
        assert!(self.tagged.len() == size, "{}: {label} - Expected {size} values, got {}", self.key.as_str(), self.tagged.len());
        self
    }
}

/// `io.papermc.paper.tag.TagEntry`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagEntry {
    key: NamespacedKey,
    tag: bool,
    required: bool,
}

impl TagEntry {
    /// bukkit: io/papermc/paper/tag/TagEntry.valueEntry
    pub fn value_entry(key: &RegistryTypedKey, required: bool) -> Self {
        Self { key: key.key.clone(), tag: false, required }
    }
    /// bukkit: io/papermc/paper/tag/TagEntry.tagEntry
    pub fn tag_entry(key: &TagKey, required: bool) -> Self {
        Self { key: key.key.clone(), tag: true, required }
    }
    /// bukkit: io/papermc/paper/tag/TagEntry.isTag
    pub fn is_tag(&self) -> bool {
        self.tag
    }
    /// bukkit: io/papermc/paper/tag/TagEntry.isRequired
    pub fn is_required(&self) -> bool {
        self.required
    }
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
    fn file_form(&self) -> Value {
        let id = if self.tag { format!("#{}", self.key.as_str()) } else { self.key.as_str().to_owned() };
        if self.required { json!(id) } else { json!({ "id": id, "required": false }) }
    }
    fn from_file(value: &Value) -> Option<Self> {
        let id = value.as_str().or_else(|| value["id"].as_str())?;
        let name = id.strip_prefix('#');
        Some(Self { key: NamespacedKey::from_string(name.unwrap_or(id))?, tag: name.is_some(), required: value["required"].as_bool().unwrap_or(true) })
    }
}

/// A tag file of the plugins' data pack, which the game reads as it starts.
fn tag_path(key: &TagKey) -> PathBuf {
    crate::fam_core::world_folder().join("datapacks/plugin-registry/data").join(key.key.get_namespace()).join("tags").join(key.registry.tag_folder()).join(format!("{}.json", key.key.get_key()))
}

fn write_tag(key: &TagKey, values: &[Value], replace: bool) {
    let file = tag_path(key);
    let root = crate::fam_core::world_folder().join("datapacks/plugin-registry");
    let before: Value = std::fs::read(&file).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default();
    let mut all = if replace { Vec::new() } else { before["values"].as_array().cloned().unwrap_or_default() };
    for value in values {
        if !all.contains(value) {
            all.push(value.clone());
        }
    }
    let content = json!({ "replace": replace, "values": all });
    if content == before {
        return;
    }
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(root.join("pack.mcmeta"), json!({ "pack": { "pack_format": 94, "description": "Registry entries of plugins" } }).to_string());
    if std::fs::write(file, content.to_string()).is_ok() && !RELOAD_COMING.swap(true, std::sync::atomic::Ordering::Relaxed) {
        // Once for however many tags a plugin writes at a go.
        scheduler::run_task(|| {
            RELOAD_COMING.store(false, std::sync::atomic::Ordering::Relaxed);
            run_command("reload");
        });
    }
}

static RELOAD_COMING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn plugin_tag_entries(key: &TagKey) -> Vec<TagEntry> {
    let file: Value = std::fs::read(tag_path(key)).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default();
    file["values"].as_array().into_iter().flatten().filter_map(TagEntry::from_file).collect()
}

fn known_tags(registry: RegistryKey) -> Vec<TagKey> {
    let names = query(json!({ "op": "tags", "kind": registry.tag_folder() }));
    names.as_array().into_iter().flatten().filter_map(|name| NamespacedKey::from_string(name.as_str()?)).map(|key| registry.tag_key(&key)).collect()
}

/// `io.papermc.paper.tag.PreFlattenTagRegistrar`: tags as entries, some of them other tags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreFlattenTagRegistrar(pub RegistryKey);

impl PreFlattenTagRegistrar {
    /// bukkit: io/papermc/paper/tag/PreFlattenTagRegistrar.registryKey
    pub fn registry_key(&self) -> RegistryKey {
        self.0
    }
    /// bukkit: io/papermc/paper/tag/PreFlattenTagRegistrar.getAllTags
    pub fn get_all_tags(&self) -> Vec<(TagKey, Vec<TagEntry>)> {
        known_tags(self.0).into_iter().map(|key| (key.clone(), self.get_tag(&key))).collect()
    }
    /// bukkit: io/papermc/paper/tag/PreFlattenTagRegistrar.hasTag
    pub fn has_tag(&self, key: &TagKey) -> bool {
        known_tags(self.0).contains(key) || tag_path(key).is_file()
    }
    /// bukkit: io/papermc/paper/tag/PreFlattenTagRegistrar.getTag
    pub fn get_tag(&self, key: &TagKey) -> Vec<TagEntry> {
        let written = plugin_tag_entries(key);
        if !written.is_empty() {
            return written;
        }
        key.resolve().set.keys.iter().map(|value| TagEntry::value_entry(value, true)).collect()
    }
    /// bukkit: io/papermc/paper/tag/PreFlattenTagRegistrar.addToTag
    pub fn add_to_tag(&self, key: &TagKey, entries: &[TagEntry]) {
        write_tag(key, &entries.iter().map(TagEntry::file_form).collect::<Vec<_>>(), false);
    }
    /// bukkit: io/papermc/paper/tag/PreFlattenTagRegistrar.setTag
    pub fn set_tag(&self, key: &TagKey, entries: &[TagEntry]) {
        write_tag(key, &entries.iter().map(TagEntry::file_form).collect::<Vec<_>>(), true);
    }
}

/// `io.papermc.paper.tag.PostFlattenTagRegistrar`: tags as plain values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostFlattenTagRegistrar(pub RegistryKey);

impl PostFlattenTagRegistrar {
    /// bukkit: io/papermc/paper/tag/PostFlattenTagRegistrar.registryKey
    pub fn registry_key(&self) -> RegistryKey {
        self.0
    }
    /// bukkit: io/papermc/paper/tag/PostFlattenTagRegistrar.getAllTags
    pub fn get_all_tags(&self) -> Vec<(TagKey, Vec<RegistryTypedKey>)> {
        known_tags(self.0).into_iter().map(|key| (key.clone(), self.get_tag(&key))).collect()
    }
    /// bukkit: io/papermc/paper/tag/PostFlattenTagRegistrar.hasTag
    pub fn has_tag(&self, key: &TagKey) -> bool {
        known_tags(self.0).contains(key) || tag_path(key).is_file()
    }
    /// bukkit: io/papermc/paper/tag/PostFlattenTagRegistrar.getTag
    pub fn get_tag(&self, key: &TagKey) -> Vec<RegistryTypedKey> {
        key.resolve().set.keys
    }
    /// bukkit: io/papermc/paper/tag/PostFlattenTagRegistrar.addToTag
    pub fn add_to_tag(&self, key: &TagKey, values: &[RegistryTypedKey]) {
        write_tag(key, &values.iter().map(|value| json!(value.key.as_str())).collect::<Vec<_>>(), false);
    }
    /// bukkit: io/papermc/paper/tag/PostFlattenTagRegistrar.setTag
    pub fn set_tag(&self, key: &TagKey, values: &[RegistryTypedKey]) {
        write_tag(key, &values.iter().map(|value| json!(value.key.as_str())).collect::<Vec<_>>(), true);
    }
}

// ---- Time and bounds ----

/// `io.papermc.paper.util.Tick`, the unit of `std::time`'s 50 ms.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tick;

impl Tick {
    const MILLIS: u64 = 50;
    /// bukkit: io/papermc/paper/util/Tick.tick
    pub fn tick() -> Self {
        Self
    }
    /// bukkit: io/papermc/paper/util/Tick.of
    pub fn of(ticks: u64) -> Duration {
        Duration::from_millis(ticks * Self::MILLIS)
    }
    /// bukkit: io/papermc/paper/util/Tick.fromDuration
    pub fn from_duration(self, duration: Duration) -> i32 {
        (duration.as_millis() / u128::from(Self::MILLIS)) as i32
    }
    /// bukkit: io/papermc/paper/util/Tick.getDuration
    pub fn get_duration(self) -> Duration {
        Duration::from_millis(Self::MILLIS)
    }
    /// bukkit: io/papermc/paper/util/Tick.isDurationEstimated
    pub fn is_duration_estimated(self) -> bool {
        false
    }
    /// bukkit: io/papermc/paper/util/Tick.isDateBased
    pub fn is_date_based(self) -> bool {
        false
    }
    /// bukkit: io/papermc/paper/util/Tick.isTimeBased
    pub fn is_time_based(self) -> bool {
        true
    }
    /// bukkit: io/papermc/paper/util/Tick.addTo
    pub fn add_to(self, time: SystemTime, amount: u64) -> SystemTime {
        time + Self::of(amount)
    }
    /// bukkit: io/papermc/paper/util/Tick.between
    pub fn between(self, start: SystemTime, end: SystemTime) -> i64 {
        match end.duration_since(start) {
            Ok(gone) => (gone.as_millis() / u128::from(Self::MILLIS)) as i64,
            Err(before) => -((before.duration().as_millis() / u128::from(Self::MILLIS)) as i64),
        }
    }
}

/// bukkit: io/papermc/paper/util/BoundChecker.requireRange
///
/// # Panics
/// As Java throws, out of the range.
pub fn require_range<T: PartialOrd + std::fmt::Display>(value: T, field: &str, min: T, max: T) -> T {
    assert!(value >= min && value <= max, "argument {field} must be in [{min}, {max}]: {value}");
    value
}

/// bukkit: io/papermc/paper/util/BoundChecker.requireNonNegative
///
/// # Panics
/// As Java throws, below zero.
pub fn require_non_negative<T: PartialOrd + Default + std::fmt::Display>(value: T, field: &str) -> T {
    assert!(value >= T::default(), "argument {field} must be non-negative: {value}");
    value
}

/// bukkit: io/papermc/paper/util/BoundChecker.requirePositive
///
/// # Panics
/// As Java throws, at zero or below.
pub fn require_positive<T: PartialOrd + Default + std::fmt::Display>(value: T, field: &str) -> T {
    assert!(value > T::default(), "argument {field} must be positive: {value}");
    value
}

/// bukkit: io/papermc/paper/world/MoonPhase.getPhase
pub fn moon_phase_of(day: i64) -> &'static str {
    let phases = crate::enums::moon_phase::values();
    phases[day.rem_euclid(phases.len() as i64) as usize]
}

/// bukkit: io/papermc/paper/enchantments/EnchantmentRarity.getWeight
pub fn enchantment_rarity_weight(rarity: &str) -> i32 {
    match rarity {
        "COMMON" => 10,
        "UNCOMMON" => 5,
        "RARE" => 2,
        _ => 1,
    }
}

/// bukkit: io/papermc/paper/inventory/ItemRarity.getColor
///
/// The colour's name, as a component writes it.
pub fn item_rarity_color(rarity: &str) -> &'static str {
    match rarity {
        "UNCOMMON" => "yellow",
        "RARE" => "aqua",
        "EPIC" => "light_purple",
        _ => "white",
    }
}

/// bukkit: io/papermc/paper/advancement/AdvancementDisplay$Frame.color
pub fn frame_color(frame: &str) -> &'static str {
    if frame.eq_ignore_ascii_case("challenge") { "dark_purple" } else { "green" }
}

/// bukkit: io/papermc/paper/advancement/AdvancementDisplay$Frame.translationKey
pub fn frame_translation_key(frame: &str) -> String {
    format!("advancements.toast.{}", frame.to_ascii_lowercase())
}

impl crate::core2::AdvancementDisplay {
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.frame
    pub fn frame(&self) -> String {
        self.get_type().to_ascii_uppercase()
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.title
    pub fn title(&self) -> Component {
        Component(self.get_title().clone())
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.description
    pub fn description(&self) -> Component {
        Component(self.get_description().clone())
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.icon
    pub fn icon(&self) -> Option<ItemStack> {
        self.get_icon()
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.doesShowToast
    pub fn does_show_toast(&self) -> bool {
        self.should_show_toast()
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.doesAnnounceToChat
    pub fn does_announce_to_chat(&self) -> bool {
        self.should_announce_chat()
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.isHidden
    pub fn hidden(&self) -> bool {
        self.is_hidden()
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.backgroundPath
    pub fn background_path(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(self.0["background"].as_str()?)
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay.displayName
    ///
    /// `Advancement.name`: the title in brackets, in the frame's colour, with its description on hover.
    pub fn display_name(&self) -> Component {
        let color = frame_color(self.get_type());
        let hover = json!({ "text": "", "extra": [{ "text": "", "color": color, "extra": [self.get_title()] }, "\n", self.get_description()] });
        Component(json!({ "translate": "chat.square_brackets", "with": [self.get_title()], "color": color, "hover_event": { "action": "show_text", "value": hover } }))
    }
}

/// bukkit: io/papermc/paper/world/damagesource/FallLocationType.translationKey
pub fn fall_location_translation_key(id: &str) -> String {
    format!("death.fell.accident.{id}")
}

/// bukkit: io/papermc/paper/world/damagesource/FallLocationType.id
///
/// Its id from Bukkit's constant name, `WEEPING_VINES` to `weeping_vines`.
pub fn fall_location_id(name: &str) -> String {
    name.to_ascii_lowercase()
}

// ---- Beds ----

/// `io.papermc.paper.block.bed.BedRuleResult`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BedRuleResult {
    Allowed,
    TooMuchLight,
    Never,
}

impl BedRuleResult {
    /// bukkit: io/papermc/paper/block/bed/BedRuleResult.success
    pub fn success(self) -> bool {
        self == Self::Allowed
    }
}

/// `io.papermc.paper.block.bed.BedEnterProblem`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BedEnterProblem {
    TooFarAway,
    Obstructed,
    NotSafe,
    Explosion,
    Other,
}

impl BedEnterProblem {
    /// bukkit: io/papermc/paper/block/bed/BedEnterProblem.errorMessage
    pub fn error_message(self) -> Option<Component> {
        let key = match self {
            Self::TooFarAway => "block.minecraft.bed.too_far_away",
            Self::Obstructed => "block.minecraft.bed.obstructed",
            Self::NotSafe => "block.minecraft.bed.not_safe",
            Self::Explosion | Self::Other => return None,
        };
        Some(Component(json!({ "translate": key })))
    }
}

/// `io.papermc.paper.block.bed.BedEnterAction`.
#[derive(Clone, Debug, PartialEq)]
pub struct BedEnterAction {
    can_sleep: BedRuleResult,
    can_set_spawn: BedRuleResult,
    problem: Option<BedEnterProblem>,
    error: Option<Component>,
}

impl BedEnterAction {
    pub fn new(can_sleep: BedRuleResult, can_set_spawn: BedRuleResult, problem: Option<BedEnterProblem>, error: Option<Component>) -> Self {
        Self { can_sleep, can_set_spawn, problem, error }
    }
    /// bukkit: io/papermc/paper/block/bed/BedEnterAction.canSleep
    pub fn can_sleep(&self) -> BedRuleResult {
        self.can_sleep
    }
    /// bukkit: io/papermc/paper/block/bed/BedEnterAction.canSetSpawn
    pub fn can_set_spawn(&self) -> BedRuleResult {
        self.can_set_spawn
    }
    /// bukkit: io/papermc/paper/block/bed/BedEnterAction.problem
    pub fn problem(&self) -> Option<BedEnterProblem> {
        self.problem
    }
    /// bukkit: io/papermc/paper/block/bed/BedEnterAction.errorMessage
    pub fn error_message(&self) -> Option<&Component> {
        self.error.as_ref()
    }
}

// ---- Names ----

/// `com.destroystokyo.paper.Namespaced`.
pub trait Namespaced {
    /// bukkit: com/destroystokyo/paper/Namespaced.getNamespace
    fn namespace(&self) -> &str;
    /// bukkit: com/destroystokyo/paper/Namespaced.getKey
    fn key_part(&self) -> &str;
}

impl Namespaced for NamespacedKey {
    fn namespace(&self) -> &str {
        self.get_namespace()
    }
    fn key_part(&self) -> &str {
        self.get_key()
    }
}

/// `com.destroystokyo.paper.NamespacedTag`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamespacedTag {
    namespace: String,
    key: String,
}

impl NamespacedTag {
    pub fn new(namespace: &str, key: &str) -> Self {
        Self { namespace: namespace.to_owned(), key: key.to_owned() }
    }
    /// bukkit: com/destroystokyo/paper/NamespacedTag.getNamespace
    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }
    /// bukkit: com/destroystokyo/paper/NamespacedTag.getKey
    pub fn get_key(&self) -> &str {
        &self.key
    }
    /// bukkit: com/destroystokyo/paper/NamespacedTag.minecraft
    pub fn minecraft(key: &str) -> Self {
        Self::new("minecraft", key)
    }
    /// bukkit: com/destroystokyo/paper/NamespacedTag.randomKey
    pub fn random_key() -> Self {
        use std::hash::{BuildHasher, Hasher};
        let word = || std::collections::hash_map::RandomState::new().build_hasher().finish();
        let (high, low) = (word(), word());
        Self::new("bukkit", &format!("{:08x}-{:04x}-4{:03x}-a{:03x}-{:012x}", high >> 32, (high >> 16) & 0xFFFF, high & 0xFFF, low >> 52, low & 0xFFFF_FFFF_FFFF))
    }
}

impl Namespaced for NamespacedTag {
    fn namespace(&self) -> &str {
        &self.namespace
    }
    fn key_part(&self) -> &str {
        &self.key
    }
}

/// `com.destroystokyo.paper.util.VersionFetcher`.
pub trait VersionFetcher {
    /// bukkit: com/destroystokyo/paper/util/VersionFetcher.getCacheTime
    fn get_cache_time(&self) -> i64;
    /// bukkit: com/destroystokyo/paper/util/VersionFetcher.getVersionMessage
    fn get_version_message(&self) -> Component;
}

/// `com.destroystokyo.paper.util.VersionFetcher$DummyVersionFetcher`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DummyVersionFetcher;

impl DummyVersionFetcher {
    /// bukkit: com/destroystokyo/paper/util/VersionFetcher$DummyVersionFetcher.getCacheTime
    pub fn cache_time(self) -> i64 {
        -1
    }
    /// bukkit: com/destroystokyo/paper/util/VersionFetcher$DummyVersionFetcher.getVersionMessage
    pub fn version_message(self) -> Component {
        Component(json!({ "text": "Unable to check for updates. No version provider set.", "color": "red" }))
    }
}

impl VersionFetcher for DummyVersionFetcher {
    fn get_cache_time(&self) -> i64 {
        self.cache_time()
    }
    fn get_version_message(&self) -> Component {
        self.version_message()
    }
}

/// The name of the list's Java class.
pub fn ban_list_type_class(kind: &str) -> &'static str {
    if kind == "IP" { "IpBanList" } else { "ProfileBanList" }
}

impl Position {
    /// bukkit: io/papermc/paper/math/Position.toVector
    pub fn to_vector(&self) -> Vector {
        Vector::new(self.x(), self.y(), self.z())
    }
    /// bukkit: io/papermc/paper/math/Position.toLocation
    pub fn to_location(&self, world: &World) -> Location {
        Location::new(world.clone(), self.x(), self.y(), self.z(), 0.0, 0.0)
    }
}

/// `com.destroystokyo.paper.inventory.meta.ArmorStandMeta`, over the item's `entity_data`.
#[derive(Clone, Debug, PartialEq)]
pub struct ArmorStandMeta(pub ItemStack);

impl crate::meta::MetaClass for ArmorStandMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl ArmorStandMeta {
    fn flag(&self, name: &str) -> bool {
        self.0.component_at("minecraft:entity_data", name).is_some_and(crate::data::flag)
    }
    fn set_flag(&mut self, name: &str, value: bool) {
        self.0.set_component_at("minecraft:entity_data", "id", json!("minecraft:armor_stand"));
        self.0.set_component_at("minecraft:entity_data", name, json!(value));
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.isInvisible
    pub fn is_invisible(&self) -> bool {
        self.flag("Invisible")
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.hasNoBasePlate
    pub fn has_no_base_plate(&self) -> bool {
        self.flag("NoBasePlate")
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.shouldShowArms
    pub fn should_show_arms(&self) -> bool {
        self.flag("ShowArms")
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.isSmall
    pub fn is_small(&self) -> bool {
        self.flag("Small")
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.isMarker
    pub fn is_marker(&self) -> bool {
        self.flag("Marker")
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.setInvisible
    pub fn set_invisible(&mut self, value: bool) {
        self.set_flag("Invisible", value);
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.setNoBasePlate
    pub fn set_no_base_plate(&mut self, value: bool) {
        self.set_flag("NoBasePlate", value);
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.setShowArms
    pub fn set_show_arms(&mut self, value: bool) {
        self.set_flag("ShowArms", value);
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.setSmall
    pub fn set_small(&mut self, value: bool) {
        self.set_flag("Small", value);
    }
    /// bukkit: com/destroystokyo/paper/inventory/meta/ArmorStandMeta.setMarker
    pub fn set_marker(&mut self, value: bool) {
        self.set_flag("Marker", value);
    }
}

// ---- Datapacks ----

fn pack_state() -> Value {
    query(json!({ "op": "paper_datapacks" }))
}

/// `io.papermc.paper.datapack.Datapack`, and `DiscoveredDatapack`: a pack of the world or the game's own.
#[derive(Clone, Debug, PartialEq)]
pub struct Datapack {
    name: String,
    enabled: bool,
    meta: Value,
    format: i64,
    title: Value,
    required: bool,
    plugin: bool,
}

impl Datapack {
    fn read(row: &Value, format: i64) -> Option<Self> {
        Some(Self { name: row["id"].as_str()?.to_owned(), enabled: row["enabled"].as_bool().unwrap_or(false), meta: row["meta"].clone(), format, title: row["title"].clone(), required: row["required"].as_bool().unwrap_or(false), plugin: row["plugin"].as_bool().unwrap_or(false) })
    }
    /// bukkit: io/papermc/paper/datapack/DiscoveredDatapack.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: io/papermc/paper/datapack/Datapack.isEnabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    /// bukkit: io/papermc/paper/datapack/Datapack.setEnabled
    pub fn set_enabled(&mut self, enabled: bool) {
        if query(json!({ "op": "paper_set_pack", "name": self.name, "enabled": enabled })).as_bool() == Some(true) {
            self.enabled = enabled;
        }
    }
    /// bukkit: io/papermc/paper/datapack/DiscoveredDatapack.getTitle
    pub fn get_title(&self) -> Component {
        if self.title.is_null() {
            return Component::text(self.name.trim_start_matches("file/"));
        }
        Component(self.title.clone())
    }
    /// bukkit: io/papermc/paper/datapack/DiscoveredDatapack.getDescription
    pub fn get_description(&self) -> Component {
        Component(self.meta["pack"]["description"].clone())
    }
    /// bukkit: io/papermc/paper/datapack/DiscoveredDatapack.isRequired
    pub fn is_required(&self) -> bool {
        self.required || self.name == "vanilla"
    }
    /// bukkit: io/papermc/paper/datapack/DiscoveredDatapack.getCompatibility
    ///
    /// `TOO_OLD`, `TOO_NEW` or `COMPATIBLE`, from `pack.mcmeta`'s formats.
    pub fn get_compatibility(&self) -> &'static str {
        let pack = &self.meta["pack"];
        let first = |value: &Value| value.as_i64().or_else(|| value[0].as_i64());
        let low = first(&pack["min_format"]).or_else(|| first(&pack["pack_format"]));
        let high = first(&pack["max_format"]).or_else(|| first(&pack["pack_format"]));
        match (low, high) {
            (_, Some(high)) if high < self.format => "TOO_OLD",
            (Some(low), _) if low > self.format => "TOO_NEW",
            _ => "COMPATIBLE",
        }
    }
    /// bukkit: io/papermc/paper/datapack/DiscoveredDatapack.getRequiredFeatures
    pub fn get_required_features(&self) -> Vec<String> {
        self.meta["features"]["enabled"].as_array().into_iter().flatten().filter_map(|flag| flag.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: io/papermc/paper/datapack/DiscoveredDatapack.getSource
    ///
    /// `DEFAULT` for the game's own, `WORLD` for the world's folder, `FEATURE` for a feature pack, `PLUGIN` for a plugin's.
    pub fn get_source(&self) -> &'static str {
        if self.plugin {
            return "PLUGIN";
        }
        match self.name.as_str() {
            "vanilla" => "DEFAULT",
            name if name.starts_with("file/") => "WORLD",
            _ => "FEATURE",
        }
    }
    /// bukkit: io/papermc/paper/datapack/Datapack.computeDisplayName
    pub fn compute_display_name(&self) -> Component {
        let color = if self.enabled { "green" } else { "red" };
        Component(json!({ "translate": "chat.square_brackets", "with": [self.get_title().0], "color": color, "hover_event": { "action": "show_text", "value": self.get_description().0 } }))
    }
}

/// bukkit: org/bukkit/Bukkit.getDatapackManager, org/bukkit/Server.getDatapackManager
pub fn get_datapack_manager() -> DatapackManager {
    DatapackManager
}

/// `io.papermc.paper.datapack.DatapackManager`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DatapackManager;

impl DatapackManager {
    /// bukkit: io/papermc/paper/datapack/DatapackManager.getPacks
    pub fn get_packs(&self) -> Vec<Datapack> {
        let state = pack_state();
        let format = state["format"].as_i64().unwrap_or_default();
        state["packs"].as_array().into_iter().flatten().filter_map(|row| Datapack::read(row, format)).collect()
    }
    /// bukkit: io/papermc/paper/datapack/DatapackManager.getEnabledPacks
    pub fn get_enabled_packs(&self) -> Vec<Datapack> {
        self.get_packs().into_iter().filter(Datapack::is_enabled).collect()
    }
    /// bukkit: io/papermc/paper/datapack/DatapackManager.getPack
    pub fn get_pack(&self, name: &str) -> Option<Datapack> {
        self.get_packs().into_iter().find(|pack| pack.name == name)
    }
    /// bukkit: io/papermc/paper/datapack/DatapackManager.refreshPacks
    pub fn refresh_packs(&self) {
        query(json!({ "op": "paper_datapacks_refresh" }));
    }
}

/// `io.papermc.paper.datapack.DatapackRegistrar$Configurer`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DatapackConfigurer {
    title: Option<Component>,
    auto_enable: bool,
    position: Option<(bool, String)>,
}

impl DatapackConfigurer {
    /// bukkit: io/papermc/paper/datapack/DatapackRegistrar$Configurer.title
    pub fn title(mut self, title: &Component) -> Self {
        self.title = Some(title.clone());
        self
    }
    /// bukkit: io/papermc/paper/datapack/DatapackRegistrar$Configurer.autoEnableOnServerStart
    pub fn auto_enable_on_server_start(mut self, on: bool) -> Self {
        self.auto_enable = on;
        self
    }
    /// bukkit: io/papermc/paper/datapack/DatapackRegistrar$Configurer.position
    ///
    /// `position` is `TOP` or `BOTTOM`, of `Datapack.Position`.
    pub fn position(mut self, fixed: bool, position: &str) -> Self {
        self.position = Some((fixed, position.to_owned()));
        self
    }
}

static DISCOVERED: Mutex<BTreeMap<String, Datapack>> = Mutex::new(BTreeMap::new());

/// A `file:` URI's path, with what the URI escaped.
fn uri_path(uri: &str) -> Option<String> {
    let rest = uri.strip_prefix("file:")?;
    let rest = rest.strip_prefix("//").unwrap_or(rest);
    let rest = rest.split_once('/').map_or(rest, |(host, _)| if host.is_empty() || host == "localhost" { &rest[host.len()..] } else { rest });
    // `/C:/...` is a Windows path, and needs no first slash.
    let rest = if rest.as_bytes().get(2) == Some(&b':') { &rest[1..] } else { rest };
    let mut bytes = Vec::with_capacity(rest.len());
    let mut from = rest.bytes();
    while let Some(byte) = from.next() {
        if byte == b'%' {
            let hex = [from.next()?, from.next()?];
            bytes.push(u8::from_str_radix(std::str::from_utf8(&hex).ok()?, 16).ok()?);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}

/// `io.papermc.paper.datapack.DatapackRegistrar`: every pack that's been found, the plugins' own among them.
#[derive(Clone, Copy)]
pub struct DatapackRegistrar {
    plugin: Plugin,
}

impl DatapackRegistrar {
    /// bukkit: io/papermc/paper/datapack/DatapackRegistrar.hasPackDiscovered
    pub fn has_pack_discovered(&self, name: &str) -> bool {
        self.get_discovered_packs().contains_key(name)
    }
    /// bukkit: io/papermc/paper/datapack/DatapackRegistrar.getDiscoveredPack
    ///
    /// # Panics
    /// As Java throws, for a name not discovered.
    pub fn get_discovered_pack(&self, name: &str) -> Datapack {
        self.get_discovered_packs().remove(name).unwrap_or_else(|| panic!("No pack with id {name} was discovered"))
    }
    /// bukkit: io/papermc/paper/datapack/DatapackRegistrar.getDiscoveredPacks
    pub fn get_discovered_packs(&self) -> BTreeMap<String, Datapack> {
        let mut packs: BTreeMap<String, Datapack> = get_datapack_manager().get_packs().into_iter().map(|pack| (pack.name.clone(), pack)).collect();
        for (name, pack) in locked(&DISCOVERED).iter() {
            packs.entry(name.clone()).or_insert_with(|| pack.clone());
        }
        packs
    }
    /// bukkit: io/papermc/paper/datapack/DatapackRegistrar.removeDiscoveredPack
    pub fn remove_discovered_pack(&self, name: &str) -> bool {
        let found = self.has_pack_discovered(name);
        if found {
            locked(&DISCOVERED).remove(name);
            query(json!({ "op": "paper_free_pack_remove", "id": name }));
            get_datapack_manager().refresh_packs();
        }
        found
    }
    /// bukkit~: io/papermc/paper/datapack/DatapackRegistrar.discoverPack | a symlink under the pack is let through by `allowed_symlinks.txt`'s `[prefix]` rules only: its `[glob]` and `[regex]` rules need a matcher the engine doesn't link, so what only those allow is refused
    ///
    /// A folder or a zip, read from where it is. As Java does, it fails with `None`.
    pub fn discover_pack(&self, path: &std::path::Path, id: &str, configure: impl FnOnce(DatapackConfigurer) -> DatapackConfigurer) -> Option<Datapack> {
        self.discover_pack_as(&PluginMeta(self.plugin), path, id, configure)
    }
    /// `discoverPack(URI, ...)`: a `file:` URI, or a `jar:` one of a folder in a zip, which goes under the plugin's folder to be read.
    pub fn discover_pack_uri(&self, uri: &str, id: &str, configure: impl FnOnce(DatapackConfigurer) -> DatapackConfigurer) -> Option<Datapack> {
        self.discover_pack_uri_as(&PluginMeta(self.plugin), uri, id, configure)
    }
    /// `discoverPack(PluginMeta, Path, ...)`: `owner` names it.
    pub fn discover_pack_as(&self, owner: &PluginMeta, path: &std::path::Path, id: &str, configure: impl FnOnce(DatapackConfigurer) -> DatapackConfigurer) -> Option<Datapack> {
        self.add_pack(owner, json!({ "path": path.to_string_lossy() }), id, configure)
    }
    /// `discoverPack(PluginMeta, URI, ...)`.
    pub fn discover_pack_uri_as(&self, owner: &PluginMeta, uri: &str, id: &str, configure: impl FnOnce(DatapackConfigurer) -> DatapackConfigurer) -> Option<Datapack> {
        let Some((file, inner)) = uri.strip_prefix("jar:").and_then(|rest| rest.split_once('!')) else {
            return self.add_pack(owner, json!({ "path": uri_path(uri)? }), id, configure);
        };
        let unpack = self.plugin.get_data_folder().join("discovered-packs").join(id);
        let _ = std::fs::remove_dir_all(&unpack);
        self.add_pack(owner, json!({ "path": uri_path(file)?, "inner": inner, "unpack": unpack.to_string_lossy() }), id, configure)
    }
    fn add_pack(&self, owner: &PluginMeta, place: Value, id: &str, configure: impl FnOnce(DatapackConfigurer) -> DatapackConfigurer) -> Option<Datapack> {
        let settings = configure(DatapackConfigurer { title: Some(Component::text(id)), auto_enable: false, position: None });
        let name = format!("{}/{id}", owner.get_name());
        let (fixed, top) = settings.position.as_ref().map_or((false, true), |(fixed, position)| (*fixed, position == "TOP"));
        let mut request = place;
        request["op"] = json!("paper_free_pack_add");
        request["id"] = json!(name);
        request["title"] = settings.title.map_or(Value::Null, |title| title.0);
        request["required"] = json!(settings.auto_enable);
        request["top"] = json!(top);
        request["fixed"] = json!(fixed);
        let pack = Datapack::read(&query(request), pack_state()["format"].as_i64().unwrap_or_default())?;
        locked(&DISCOVERED).insert(name, pack.clone());
        get_datapack_manager().refresh_packs();
        Some(pack)
    }
}

// ---- The connection ----

/// A link for `send_links`: one of `ServerLinks$Type`, or a label of a plugin's own.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerLinkEntry {
    pub kind: Option<String>,
    pub label: Option<Component>,
    pub url: String,
}

/// `io.papermc.paper.connection.PlayerGameConnection`, with `PlayerCommonConnection` and `PlayerConnection`:
/// a player in the game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerGameConnection(pub Player);

impl Player {
    /// bukkit: org/bukkit/entity/Player.getConnection
    pub fn get_connection(&self) -> PlayerGameConnection {
        PlayerGameConnection(self.clone())
    }
}

impl PlayerGameConnection {
    /// bukkit: io/papermc/paper/connection/PlayerGameConnection.getPlayer
    pub fn get_player(&self) -> &Player {
        &self.0
    }
    fn address(&self) -> Value {
        query(json!({ "op": "paper_address", "uuid": self.0.uuid() }))
    }
    /// bukkit: io/papermc/paper/connection/PlayerConnection.disconnect
    pub fn disconnect(&self, reason: &Component) {
        query(json!({ "op": "paper_disconnect", "uuid": self.0.uuid(), "reason": reason.0 }));
    }
    /// bukkit: io/papermc/paper/connection/PlayerConnection.isConnected
    pub fn is_connected(&self) -> bool {
        self.0.is_online()
    }
    /// bukkit: io/papermc/paper/connection/PlayerConnection.getAddress
    pub fn get_address(&self) -> Option<String> {
        self.address()["address"].as_str().map(str::to_owned)
    }
    /// bukkit: io/papermc/paper/connection/PlayerConnection.isTransferred
    pub fn is_transferred(&self) -> bool {
        self.address()["transferred"].as_bool().unwrap_or(false)
    }
    /// bukkit: io/papermc/paper/connection/PlayerConnection.getVirtualHost
    ///
    /// The host and port the client says it connected to.
    pub fn get_virtual_host(&self) -> Option<String> {
        self.address()["host"].as_str().map(str::to_owned)
    }
    /// bukkit: io/papermc/paper/connection/PlayerConnection.getClientAddress
    pub fn get_client_address(&self) -> Option<String> {
        self.get_address()
    }
    /// bukkit: io/papermc/paper/connection/PlayerConnection.getHAProxyAddress
    ///
    /// Horizon doesn't speak the PROXY protocol, so there's never one.
    pub fn get_ha_proxy_address(&self) -> Option<String> {
        None
    }
    /// bukkit: io/papermc/paper/connection/PlayerCommonConnection.transfer
    pub fn transfer(&self, host: &str, port: i32) {
        self.0.transfer(host, port);
    }
    /// bukkit: io/papermc/paper/connection/PlayerCommonConnection.sendLinks
    pub fn send_links(&self, links: &[ServerLinkEntry]) {
        query(json!({ "op": "paper_links", "uuid": self.0.uuid(), "links": link_rows(links) }));
    }
    /// bukkit: io/papermc/paper/connection/PlayerCommonConnection.sendReportDetails
    ///
    /// What the client's crash reports should say of this server: 32 details at most.
    pub fn send_report_details(&self, details: &BTreeMap<String, String>) {
        query(json!({ "op": "paper_report_details", "uuid": self.0.uuid(), "details": details }));
    }
    /// bukkit: io/papermc/paper/connection/PlayerCommonConnection.getClientOption
    pub fn get_client_option(&self, option: &ClientOption) -> Value {
        self.0.get_client_option(option.name)
    }
    /// bukkit: io/papermc/paper/connection/PlayerCommonConnection.getClientBrandName
    pub fn get_client_brand_name(&self) -> Option<String> {
        self.0.get_client_brand_name()
    }
    /// bukkit: io/papermc/paper/connection/WritablePlayerCookieConnection.storeCookie
    pub fn store_cookie(&self, key: &NamespacedKey, value: &[u8]) {
        self.0.store_cookie(key.as_str(), value);
    }
}

type CookieCallback = Box<dyn FnOnce(Option<Vec<u8>>) + Send>;

static COOKIE_WAITERS: Mutex<Vec<(String, String, CookieCallback)>> = Mutex::new(Vec::new());
static COOKIE_LISTENER: std::sync::Once = std::sync::Once::new();

/// A client's `ServerboundCookieResponsePacket`, from hook `paper_cookie`.
struct CookieAnswer {
    uuid: String,
    key: String,
    payload: Option<Vec<u8>>,
}

impl crate::event::Event for CookieAnswer {
    const HOOK: &'static str = "paper_cookie";
    fn read(value: &Value) -> Option<Self> {
        let cookie = value.get("cookie").filter(|cookie| !cookie.is_null())?;
        let payload = cookie["payload"].as_array().map(|bytes| bytes.iter().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect());
        Some(Self { uuid: value["uuid"].as_str()?.to_owned(), key: cookie["key"].as_str()?.to_owned(), payload })
    }
}

impl PlayerGameConnection {
    /// bukkit: io/papermc/paper/connection/ReadablePlayerCookieConnection.retrieveCookie
    ///
    /// Java's future is `callback`, on the server's next tick: `None` for a cookie the client doesn't have.
    pub fn retrieve_cookie(&self, plugin: &Plugin, key: &NamespacedKey, callback: impl FnOnce(Option<Vec<u8>>) + Send + 'static) {
        wait_for_cookie(plugin, self.0.uuid(), key, callback);
        query(json!({ "op": "paper_cookie_request", "uuid": self.0.uuid(), "key": key.as_str() }));
    }
}

fn wait_for_cookie(plugin: &Plugin, uuid: &str, key: &NamespacedKey, callback: impl FnOnce(Option<Vec<u8>>) + Send + 'static) {
    COOKIE_LISTENER.call_once(|| {
        plugin.register_event::<CookieAnswer>(|answer| {
            let mut waiters = locked(&COOKIE_WAITERS);
            let Some(at) = waiters.iter().position(|(uuid, key, _)| *uuid == answer.uuid && *key == answer.key) else {
                return;
            };
            let mut waiter = Some(waiters.remove(at).2);
            drop(waiters);
            let mut payload = answer.payload.take();
            scheduler::run_task(move || {
                if let Some(callback) = waiter.take() {
                    callback(payload.take());
                }
            });
        });
    });
    locked(&COOKIE_WAITERS).push((uuid.to_owned(), key.as_str().to_owned(), Box::new(callback)));
}

fn link_rows(links: &[ServerLinkEntry]) -> Vec<Value> {
    links
        .iter()
        .map(|link| {
            let known = link.kind.as_deref().and_then(|kind| crate::enums::server_links_type::values().iter().position(|name| *name == kind));
            json!({ "known": known, "label": link.label.as_ref().map(|label| &label.0), "url": link.url })
        })
        .collect()
}

/// `PlayerCustomClickEvent.getCommonConnection`.
#[derive(Clone, Debug, PartialEq)]
pub enum PlayerCommonConnection {
    Game(PlayerGameConnection),
    Configuration(PlayerConfigurationConnection),
}

// ---- Folia's tasks ----

const IDLE: u8 = 0;
const RUNNING: u8 = 1;
const FINISHED: u8 = 2;
const CANCELLED: u8 = 3;
const CANCELLED_RUNNING: u8 = 4;

/// `io.papermc.paper.threadedregions.scheduler.ScheduledTask`.
#[derive(Clone)]
pub struct ScheduledTask {
    id: Option<u32>,
    plugin: Plugin,
    repeating: bool,
    state: Arc<std::sync::atomic::AtomicU8>,
}

impl ScheduledTask {
    fn new(plugin: &Plugin, repeating: bool) -> Self {
        Self { id: None, plugin: *plugin, repeating, state: Arc::new(std::sync::atomic::AtomicU8::new(IDLE)) }
    }
    fn load(&self) -> u8 {
        self.state.load(std::sync::atomic::Ordering::SeqCst)
    }
    fn store(&self, state: u8) {
        self.state.store(state, std::sync::atomic::Ordering::SeqCst);
    }
    /// One run: `RUNNING`, the work, then back to `IDLE`, or `FINISHED` for a single task. False if it's cancelled.
    fn step(&self, work: &mut impl FnMut(&Self)) -> bool {
        if self.load() == CANCELLED {
            return false;
        }
        self.store(RUNNING);
        work(self);
        match self.load() {
            CANCELLED_RUNNING => self.store(CANCELLED),
            _ => self.store(if self.repeating { IDLE } else { FINISHED }),
        }
        self.load() == IDLE
    }
    /// Through the server's scheduler, as `RegionScheduler` and the rest do here.
    pub fn schedule(plugin: &Plugin, delay: u64, period: Option<u64>, mut work: impl FnMut(&Self) + Send + 'static) -> Self {
        let mut task = Self::new(plugin, period.is_some());
        let handle = task.clone();
        let run = move || {
            handle.step(&mut work);
        };
        task.id = Some(match period {
            Some(period) => scheduler::run_task_timer(delay, period, run),
            None => scheduler::run_task_later(delay, run),
        });
        task
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask.getOwningPlugin
    pub fn get_owning_plugin(&self) -> &Plugin {
        &self.plugin
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask.isRepeatingTask
    pub fn is_repeating_task(&self) -> bool {
        self.repeating
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask.getExecutionState
    pub fn get_execution_state(&self) -> &'static str {
        match self.load() {
            IDLE => "IDLE",
            RUNNING => "RUNNING",
            FINISHED => "FINISHED",
            CANCELLED => "CANCELLED",
            _ => "CANCELLED_RUNNING",
        }
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask.isCancelled
    pub fn is_cancelled(&self) -> bool {
        matches!(self.load(), CANCELLED | CANCELLED_RUNNING)
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask.cancel
    pub fn cancel(&self) -> &'static str {
        let answer = match self.load() {
            IDLE => "CANCELLED_BY_CALLER",
            RUNNING if self.repeating => "NEXT_RUNS_CANCELLED",
            RUNNING => "RUNNING",
            FINISHED => "ALREADY_EXECUTED",
            CANCELLED_RUNNING => "NEXT_RUNS_CANCELLED_ALREADY",
            _ => "CANCELLED_ALREADY",
        };
        match self.load() {
            IDLE => self.store(CANCELLED),
            RUNNING if self.repeating => self.store(CANCELLED_RUNNING),
            _ => {}
        }
        if let Some(id) = self.id {
            scheduler::cancel_task(id);
        }
        answer
    }
}

static ASYNC_TASKS: Mutex<Vec<ScheduledTask>> = Mutex::new(Vec::new());

impl crate::brigadier::AsyncScheduler {
    /// bukkit: io/papermc/paper/threadedregions/scheduler/AsyncScheduler.runAtFixedRate
    ///
    /// On a thread of its own, which ends when the task is cancelled.
    pub fn run_at_fixed_rate(&self, plugin: &Plugin, mut work: impl FnMut(&ScheduledTask) + Send + 'static, delay: Duration, period: Duration) -> ScheduledTask {
        let task = ScheduledTask::new(plugin, true);
        locked(&ASYNC_TASKS).push(task.clone());
        let handle = task.clone();
        scheduler::run_task_asynchronously(move || {
            let start = std::time::Instant::now() + delay;
            let mut next = start;
            loop {
                std::thread::sleep(next.saturating_duration_since(std::time::Instant::now()));
                if !handle.step(&mut work) {
                    return;
                }
                next += period;
            }
        });
        task
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/AsyncScheduler.cancelTasks
    ///
    /// A plugin's library holds only its own tasks, so all of these.
    pub fn cancel_tasks(&self, _plugin: &Plugin) {
        for task in locked(&ASYNC_TASKS).drain(..) {
            task.cancel();
        }
    }
}

// ---- Blocks ----

fn fluid_of(block: &Block) -> Option<(&'static str, i32)> {
    let name = block.get_type();
    let level = || block.get_property("level").and_then(|level| level.parse::<i32>().ok()).unwrap_or(0);
    match name.key() {
        "minecraft:water" => Some(("minecraft:water", level())),
        "minecraft:lava" => Some(("minecraft:lava", level())),
        "minecraft:bubble_column" | "minecraft:kelp" | "minecraft:kelp_plant" | "minecraft:seagrass" | "minecraft:tall_seagrass" => Some(("minecraft:water", 0)),
        _ if block.get_property("waterlogged").as_deref() == Some("true") => Some(("minecraft:water", 0)),
        _ => None,
    }
}

/// `io.papermc.paper.block.fluid.FluidData`, and `FallingFluidData`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FluidData {
    fluid: Option<&'static str>,
    /// The `level` property: 0 a source, 1 to 7 flowing, 8 and up falling.
    level: i32,
}

impl FluidData {
    /// What's in a block: `World.getFluidData`.
    pub fn of(block: &Block) -> Self {
        fluid_of(block).map_or(Self { fluid: None, level: 0 }, |(fluid, level)| Self { fluid: Some(fluid), level })
    }
    /// bukkit: io/papermc/paper/block/fluid/FluidData.getFluidType
    pub fn get_fluid_type(&self) -> &'static str {
        match (self.fluid, self.level) {
            (None, _) => "minecraft:empty",
            (Some("minecraft:water"), 0) => "minecraft:water",
            (Some("minecraft:water"), _) => "minecraft:flowing_water",
            (_, 0) => "minecraft:lava",
            _ => "minecraft:flowing_lava",
        }
    }
    /// bukkit: io/papermc/paper/block/fluid/FluidData.getLevel
    pub fn get_level(&self) -> i32 {
        match (self.fluid, self.level) {
            (None, _) => 0,
            (_, 0) => 8,
            (_, level) if level >= 8 => 8,
            (_, level) => 8 - level,
        }
    }
    /// bukkit: io/papermc/paper/block/fluid/FluidData.isSource
    pub fn is_source(&self) -> bool {
        self.fluid.is_some() && self.level == 0
    }
    /// bukkit: io/papermc/paper/block/fluid/type/FallingFluidData.isFalling
    pub fn is_falling(&self) -> bool {
        self.fluid.is_some() && self.level >= 8
    }
    fn own_height(&self) -> f32 {
        self.get_level() as f32 / 9.0
    }
    /// bukkit: io/papermc/paper/block/fluid/FluidData.computeHeight
    ///
    /// `FlowingFluid.getHeight`: full under the same fluid.
    pub fn compute_height(&self, at: &Location) -> f32 {
        let above = Self::of(&at.get_block().get_relative(0, 1, 0));
        if self.fluid.is_some() && above.fluid == self.fluid { 1.0 } else { self.own_height() }
    }
    /// bukkit: io/papermc/paper/block/fluid/FluidData.computeFlowDirection
    pub fn compute_flow_direction(&self, at: &Location) -> Vector {
        let flow = query(json!({ "op": "paper_fluid_flow", "dimension": at.get_world().dimension(), "pos": [at.get_block_x(), at.get_block_y(), at.get_block_z()] }));
        Vector::new(flow[0].as_f64().unwrap_or(0.0), flow[1].as_f64().unwrap_or(0.0), flow[2].as_f64().unwrap_or(0.0))
    }
}

/// `io.papermc.paper.command.CommandBlockHolder`: a command block or a command minecart.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandBlockHolder {
    Block(Block),
    Minecart(Entity),
}

impl CommandBlockHolder {
    fn saved(&self, name: &str) -> Value {
        match self {
            Self::Block(block) => block.tile().saved(name),
            Self::Minecart(cart) => cart.data_at(name),
        }
    }
    fn change(&self, name: &str, snbt: &str) -> bool {
        match self {
            Self::Block(block) => {
                let mut state = block.tile();
                state.change(name, snbt.to_owned());
                state.update()
            }
            Self::Minecart(cart) => cart.merge(&format!("{{{name}:{snbt}}}")),
        }
    }
    /// bukkit: io/papermc/paper/command/CommandBlockHolder.getCommand
    pub fn get_command(&self) -> String {
        self.saved("Command").as_str().unwrap_or_default().to_owned()
    }
    /// bukkit: io/papermc/paper/command/CommandBlockHolder.setCommand
    pub fn set_command(&self, command: Option<&str>) -> bool {
        self.change("Command", &crate::data::quoted(command.unwrap_or_default()))
    }
    /// bukkit: io/papermc/paper/command/CommandBlockHolder.lastOutput
    pub fn last_output(&self) -> Component {
        let saved = self.saved("LastOutput");
        if saved.is_null() { Component::empty() } else { Component(saved) }
    }
    /// bukkit: io/papermc/paper/command/CommandBlockHolder.lastOutput
    pub fn set_last_output(&self, output: Option<&Component>) -> bool {
        self.change("LastOutput", &output.map_or_else(|| "\"\"".to_owned(), |text| snbt(&text.0)))
    }
    /// bukkit: io/papermc/paper/command/CommandBlockHolder.getSuccessCount
    pub fn get_success_count(&self) -> i32 {
        self.saved("SuccessCount").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/command/CommandBlockHolder.setSuccessCount
    pub fn set_success_count(&self, count: i32) -> bool {
        self.change("SuccessCount", &count.to_string())
    }
}

/// `io.papermc.paper.block.MovingPiston`: a block in the middle of a piston's move.
#[derive(Clone, Debug, PartialEq)]
pub struct MovingPiston(pub crate::block_data::TileState);

impl crate::block_data::StateClass for MovingPiston {
    fn of(state: crate::block_data::TileState) -> Self {
        Self(state)
    }
}

impl MovingPiston {
    /// bukkit: io/papermc/paper/block/MovingPiston.getMovingBlock
    pub fn get_moving_block(&self) -> crate::block_data::BlockData {
        let state = self.0.saved("blockState");
        let name = state["Name"].as_str().unwrap_or("minecraft:air");
        let properties: Vec<String> = state["Properties"].as_object().into_iter().flatten().filter_map(|(key, value)| Some(format!("{key}={}", value.as_str()?))).collect();
        crate::block_data::BlockData::parse(&if properties.is_empty() { name.to_owned() } else { format!("{name}[{}]", properties.join(",")) })
    }
    /// bukkit: io/papermc/paper/block/MovingPiston.getDirection
    pub fn get_direction(&self) -> &'static str {
        match self.0.saved("facing").as_i64().unwrap_or(0) {
            0 => "DOWN",
            1 => "UP",
            2 => "NORTH",
            3 => "SOUTH",
            4 => "WEST",
            _ => "EAST",
        }
    }
    /// bukkit: io/papermc/paper/block/MovingPiston.isExtending
    pub fn is_extending(&self) -> bool {
        crate::data::flag(&self.0.saved("extending"))
    }
    /// bukkit: io/papermc/paper/block/MovingPiston.isPistonHead
    pub fn is_piston_head(&self) -> bool {
        crate::data::flag(&self.0.saved("source"))
    }
}

/// `io.papermc.paper.world.damagesource.CombatEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct CombatEntry {
    source: crate::core2::DamageSource,
    damage: f32,
    fall_type: Option<String>,
    fall_distance: f32,
}

impl CombatEntry {
    /// bukkit: io/papermc/paper/world/damagesource/CombatEntry.combatEntry
    pub fn combat_entry(source: &crate::core2::DamageSource, damage: f32, fall_type: Option<&str>, fall_distance: f32) -> Self {
        Self { source: source.clone(), damage, fall_type: fall_type.map(str::to_owned), fall_distance }
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatEntry.getDamageSource
    pub fn get_damage_source(&self) -> &crate::core2::DamageSource {
        &self.source
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatEntry.getDamage
    pub fn get_damage(&self) -> f32 {
        self.damage
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatEntry.getFallLocationType
    pub fn get_fall_location_type(&self) -> Option<&str> {
        self.fall_type.as_deref()
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatEntry.getFallDistance
    pub fn get_fall_distance(&self) -> f32 {
        self.fall_distance
    }
}

/// bukkit: io/papermc/paper/world/damagesource/CombatTracker.calculateFallLocationType
///
/// `CombatTracker.getFallLocation`, from the block at someone's feet.
pub fn calculate_fall_location_type(at: &Location) -> Option<&'static str> {
    let block = at.get_block();
    let climbable = crate::registry::Tag::get("block", "climbable");
    let kind = block.get_type();
    Some(match kind.key() {
        "minecraft:ladder" => "ladder",
        "minecraft:vine" => "vines",
        "minecraft:weeping_vines" | "minecraft:weeping_vines_plant" => "weeping_vines",
        "minecraft:twisting_vines" | "minecraft:twisting_vines_plant" => "twisting_vines",
        "minecraft:scaffolding" => "scaffolding",
        key if climbable.is_tagged(key) => "other_climbable",
        _ if FluidData::of(&block).get_fluid_type().contains("water") => "water",
        _ => return None,
    })
}

// ---- Rays ----

/// `io.papermc.paper.raytracing.PositionedRayTraceConfigurationBuilder`.
pub struct PositionedRayTraceConfigurationBuilder {
    start: Option<Location>,
    direction: Option<Vector>,
    max_distance: f64,
    fluids: String,
    blocks: String,
    ignore_passable: bool,
    size: f64,
    entity_filter: Option<Box<dyn Fn(&Entity) -> bool>>,
    block_filter: Option<Box<dyn Fn(&Block) -> bool>>,
    targets: Vec<String>,
}

impl Default for PositionedRayTraceConfigurationBuilder {
    fn default() -> Self {
        Self { start: None, direction: None, max_distance: 0.0, fluids: "NEVER".to_owned(), blocks: "COLLIDER".to_owned(), ignore_passable: false, size: 0.0, entity_filter: None, block_filter: None, targets: Vec::new() }
    }
}

impl PositionedRayTraceConfigurationBuilder {
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.start
    pub fn start(mut self, start: &Location) -> Self {
        self.start = Some(start.clone());
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.direction
    pub fn direction(mut self, direction: &Vector) -> Self {
        self.direction = Some(*direction);
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.maxDistance
    pub fn max_distance(mut self, distance: f64) -> Self {
        self.max_distance = distance;
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.fluidCollisionMode
    pub fn fluid_collision_mode(mut self, mode: &str) -> Self {
        mode.clone_into(&mut self.fluids);
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.blockCollisionMode
    ///
    /// `COLLIDER` tests what a block stops, `OUTLINE` its whole outline.
    pub fn block_collision_mode(mut self, mode: &str) -> Self {
        mode.clone_into(&mut self.blocks);
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.ignorePassableBlocks
    pub fn ignore_passable_blocks(mut self, ignore: bool) -> Self {
        self.ignore_passable = ignore;
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.raySize
    pub fn ray_size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.entityFilter
    pub fn entity_filter(mut self, filter: impl Fn(&Entity) -> bool + 'static) -> Self {
        self.entity_filter = Some(Box::new(filter));
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.blockFilter
    pub fn block_filter(mut self, filter: impl Fn(&Block) -> bool + 'static) -> Self {
        self.block_filter = Some(Box::new(filter));
        self
    }
    /// bukkit: io/papermc/paper/raytracing/PositionedRayTraceConfigurationBuilder.targets
    ///
    /// `ENTITY` and `BLOCK`, as many as are wanted.
    pub fn targets(mut self, targets: &[&str]) -> Self {
        self.targets = targets.iter().map(|target| (*target).to_owned()).collect();
        self
    }
    /// The boxes of a block this ray can hit.
    fn shapes(&self, block: &Block) -> Vec<[f64; 6]> {
        if self.block_filter.as_ref().is_some_and(|filter| !filter(block)) {
            return Vec::new();
        }
        let (x, y, z) = (f64::from(block.get_x()), f64::from(block.get_y()), f64::from(block.get_z()));
        let fluid = FluidData::of(block);
        let wet = match self.fluids.as_str() {
            "ALWAYS" => fluid.fluid.is_some(),
            "SOURCE_ONLY" => fluid.is_source(),
            _ => false,
        };
        let mut boxes = if wet { vec![[0.0, 0.0, 0.0, 1.0, f64::from(fluid.own_height()), 1.0]] } else { Vec::new() };
        let solid = block.get_facts()["full_collision"].as_bool().unwrap_or(false);
        if solid || !self.ignore_passable {
            if self.blocks == "OUTLINE" {
                boxes.extend(block.get_collision_shape().0);
            } else {
                let solid_boxes = query(json!({ "op": "items_collision", "dimension": block.get_world().dimension(), "pos": [block.get_x(), block.get_y(), block.get_z()] }));
                boxes.extend(solid_boxes.as_array().into_iter().flatten().filter_map(|row| Some([row[0].as_f64()?, row[1].as_f64()?, row[2].as_f64()?, row[3].as_f64()?, row[4].as_f64()?, row[5].as_f64()?])));
            }
        }
        boxes.iter().map(|[a, b, c, d, e, f]| [x + a, y + b, z + c, x + d, y + e, z + f]).collect()
    }
}

impl World {
    /// `World.rayTrace(builder)`: the nearest of what `targets` asks for. Entities are boxes of a player's size.
    pub fn ray_trace_configured(&self, configure: impl FnOnce(PositionedRayTraceConfigurationBuilder) -> PositionedRayTraceConfigurationBuilder) -> Option<crate::util::RayTraceResult> {
        let ray = configure(PositionedRayTraceConfigurationBuilder::default());
        let (start, direction) = (ray.start.as_ref()?, ray.direction?);
        let origin = start.to_vector();
        let mut step = direction;
        step.normalize();
        let wanted = |target: &str| ray.targets.is_empty() || ray.targets.iter().any(|known| known == target);
        let mut best: Option<(f64, crate::util::RayTraceResult)> = None;
        if wanted("BLOCK") {
            let mut travelled = 0.0;
            while travelled <= ray.max_distance {
                let (x, y, z) = (origin.x + step.x * travelled, origin.y + step.y * travelled, origin.z + step.z * travelled);
                let block = self.get_block_at(x.floor() as i32, y.floor() as i32, z.floor() as i32);
                let nearest = ray
                    .shapes(&block)
                    .into_iter()
                    .filter_map(|[a, b, c, d, e, f]| crate::util::BoundingBox::new(a, b, c, d, e, f).ray_trace(&origin, &step, ray.max_distance))
                    .min_by(|one, other| one.position.distance(&origin).total_cmp(&other.position.distance(&origin)));
                if let Some(mut hit) = nearest {
                    hit.block = Some(block);
                    best = Some((hit.position.distance(&origin), hit));
                    break;
                }
                travelled += 0.1;
            }
        }
        if wanted("ENTITY") {
            for entity in self.get_entities() {
                if ray.entity_filter.as_ref().is_some_and(|filter| !filter(&entity)) {
                    continue;
                }
                let Some(at) = entity.get_location() else {
                    continue;
                };
                let grow = ray.size;
                let bounds = crate::util::BoundingBox::new(at.x - 0.3 - grow, at.y - grow, at.z - 0.3 - grow, at.x + 0.3 + grow, at.y + 1.8 + grow, at.z + 0.3 + grow);
                if let Some(mut hit) = bounds.ray_trace(&origin, &step, ray.max_distance) {
                    let distance = hit.position.distance(&origin);
                    if best.as_ref().is_none_or(|(nearest, _)| distance < *nearest) {
                        hit.entity = Some(entity);
                        best = Some((distance, hit));
                    }
                }
            }
        }
        best.map(|(_, hit)| hit)
    }
}

// ---- Text ----

/// `net.kyori.adventure`'s serializers, as `PaperComponents` hands them out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentSerializer {
    Gson,
    GsonDownsampled,
    Plain,
    LegacySection,
}

/// bukkit: io/papermc/paper/text/PaperComponents.gsonSerializer
pub fn gson_serializer() -> ComponentSerializer {
    ComponentSerializer::Gson
}

/// bukkit: io/papermc/paper/text/PaperComponents.colorDownsamplingGsonSerializer
pub fn color_downsampling_gson_serializer() -> ComponentSerializer {
    ComponentSerializer::GsonDownsampled
}

const NAMED_RGB: [(&str, u32); 16] = [("black", 0x000000), ("dark_blue", 0x0000AA), ("dark_green", 0x00AA00), ("dark_aqua", 0x00AAAA), ("dark_red", 0xAA0000), ("dark_purple", 0xAA00AA), ("gold", 0xFFAA00), ("gray", 0xAAAAAA), ("dark_gray", 0x555555), ("blue", 0x5555FF), ("green", 0x55FF55), ("aqua", 0x55FFFF), ("red", 0xFF5555), ("light_purple", 0xFF55FF), ("yellow", 0xFFFF55), ("white", 0xFFFFFF)];

fn hsv(rgb: u32) -> [f32; 3] {
    let [r, g, b] = [(rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255].map(|channel| channel as f32 / 255.0);
    let (high, low) = (r.max(g).max(b), r.min(g).min(b));
    let delta = high - low;
    let hue = if delta == 0.0 {
        0.0
    } else if high == r {
        ((g - b) / delta).rem_euclid(6.0) / 6.0
    } else if high == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };
    [hue, if high == 0.0 { 0.0 } else { delta / high }, high]
}

/// `NamedTextColor.nearestTo`: by hue, saturation and value, hue counting three times.
fn nearest_named(rgb: u32) -> &'static str {
    let [hue, saturation, value] = hsv(rgb);
    let distance = |known: u32| {
        let [other_hue, other_saturation, other_value] = hsv(known);
        let apart = (hue - other_hue).abs();
        let hue_distance = 3.0 * apart.min(1.0 - apart);
        hue_distance * hue_distance + (saturation - other_saturation).powi(2) + (value - other_value).powi(2)
    };
    NAMED_RGB.iter().min_by(|one, other| distance(one.1).total_cmp(&distance(other.1))).map_or("white", |row| row.0)
}

fn downsampled(value: &Value) -> Value {
    match value {
        Value::Array(parts) => Value::Array(parts.iter().map(downsampled).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, field)| match (key.as_str(), field.as_str().and_then(|color| color.strip_prefix('#'))) {
                    ("color", Some(hex)) => (key.clone(), json!(u32::from_str_radix(hex, 16).map_or("white", nearest_named))),
                    _ => (key.clone(), downsampled(field)),
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

/// bukkit: io/papermc/paper/text/PaperComponents.plainTextSerializer
pub fn plain_text_serializer() -> ComponentSerializer {
    ComponentSerializer::Plain
}

/// bukkit: io/papermc/paper/text/PaperComponents.legacySectionSerializer
pub fn legacy_section_serializer() -> ComponentSerializer {
    ComponentSerializer::LegacySection
}

impl ComponentSerializer {
    pub fn serialize(self, component: &Component) -> String {
        match self {
            Self::Gson => component.0.to_string(),
            Self::GsonDownsampled => downsampled(&component.0).to_string(),
            Self::Plain => component.plain(),
            Self::LegacySection => component.to_legacy(),
        }
    }
    pub fn deserialize(self, text: &str) -> Component {
        match self {
            Self::Gson | Self::GsonDownsampled => Component(serde_json::from_str(text).unwrap_or_else(|_| json!(text))),
            Self::Plain => Component::text(text),
            Self::LegacySection => crate::adventure::legacy(text),
        }
    }
}

/// bukkit: io/papermc/paper/text/PaperComponents.flattener
///
/// To plain text in the server's English, as Paper's flattener reads translated parts.
pub fn flatten_component(component: &Component) -> String {
    query(json!({ "op": "paper_english", "component": component.0 })).as_str().map_or_else(|| component.plain(), str::to_owned)
}

// ---- Potions ----

type StackTest = Arc<dyn Fn(&ItemStack) -> bool + Send + Sync>;

/// `io.papermc.paper.potion.PredicateRecipeChoice`.
#[derive(Clone)]
pub struct PredicateRecipeChoice(StackTest);

impl PredicateRecipeChoice {
    /// bukkit: io/papermc/paper/potion/PotionMix.createPredicateChoice
    pub fn create(test: impl Fn(&ItemStack) -> bool + Send + Sync + 'static) -> Self {
        Self(Arc::new(test))
    }
    /// bukkit: io/papermc/paper/potion/PredicateRecipeChoice.test
    pub fn test(&self, stack: &ItemStack) -> bool {
        (self.0)(stack)
    }
    /// bukkit: io/papermc/paper/potion/PredicateRecipeChoice.itemStackPredicate
    pub fn item_stack_predicate(&self) -> impl Fn(&ItemStack) -> bool + '_ {
        |stack| (self.0)(stack)
    }
    /// bukkit: io/papermc/paper/potion/PredicateRecipeChoice.getItemStack
    ///
    /// # Panics
    /// As Java throws: this choice has no stack.
    pub fn get_item_stack(&self) -> ItemStack {
        panic!("PredicateRecipeChoice does not support this")
    }
}

/// `io.papermc.paper.potion.PotionMix`, generic over the recipe choice a port uses.
#[derive(Clone)]
pub struct PotionMix<C> {
    key: NamespacedKey,
    result: ItemStack,
    input: C,
    ingredient: C,
}

impl<C> PotionMix<C> {
    pub fn new(key: &NamespacedKey, result: &ItemStack, input: C, ingredient: C) -> Self {
        Self { key: key.clone(), result: result.clone(), input, ingredient }
    }
    /// bukkit: io/papermc/paper/potion/PotionMix.getKey
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
    /// bukkit: io/papermc/paper/potion/PotionMix.getResult
    pub fn get_result(&self) -> &ItemStack {
        &self.result
    }
    /// bukkit: io/papermc/paper/potion/PotionMix.getInput
    pub fn get_input(&self) -> &C {
        &self.input
    }
    /// bukkit: io/papermc/paper/potion/PotionMix.getIngredient
    pub fn get_ingredient(&self) -> &C {
        &self.ingredient
    }
}

// ---- Features ----

/// bukkit: io/papermc/paper/world/flag/FeatureFlagSetHolder.getFeatureFlags
///
/// The features of the world, as `level.dat` has them.
pub fn get_feature_flags() -> Vec<String> {
    query(json!({ "op": "paper_features" })).as_array().into_iter().flatten().filter_map(|flag| flag.as_str().map(str::to_owned)).collect()
}

/// bukkit: io/papermc/paper/world/flag/FeatureDependant.requiredFeatures
///
/// In the game's own tables only the rule `max_minecart_speed` needs a feature but `minecraft:vanilla`.
pub fn required_features(registry: RegistryKey, key: &NamespacedKey) -> Vec<String> {
    let gated = registry == RegistryKey::GAME_RULE && key.as_str() == "minecraft:max_minecart_speed";
    vec![if gated { "minecraft:minecart_improvements" } else { "minecraft:vanilla" }.to_owned()]
}

/// bukkit: io/papermc/paper/world/flag/FeatureFlagSetHolder.isEnabled
///
/// Takes `required_features`.
pub fn is_feature_enabled(required: &[String]) -> bool {
    let enabled = get_feature_flags();
    required.iter().all(|flag| enabled.contains(flag))
}

// ---- Dialog callbacks ----

/// `io.papermc.paper.registry.data.dialog.action.DialogActionCallback`: any closure of the two.
pub trait DialogActionCallback: Send + Sync + 'static {
    /// bukkit: io/papermc/paper/registry/data/dialog/action/DialogActionCallback.accept
    fn accept(&self, response: &DialogResponseView, audience: &DialogAudience);
}

impl<F: Fn(&DialogResponseView, &DialogAudience) + Send + Sync + 'static> DialogActionCallback for F {
    fn accept(&self, response: &DialogResponseView, audience: &DialogAudience) {
        self(response, audience);
    }
}

/// Who clicked: a player, or someone a dialog holds in configuration.
#[derive(Clone, Debug, PartialEq)]
pub enum DialogAudience {
    Player(Player),
    Configuring(ConfigurationAudience),
}

static DIALOG_CALLBACKS: Mutex<BTreeMap<String, Arc<dyn DialogActionCallback>>> = Mutex::new(BTreeMap::new());
static DIALOG_LISTENER: std::sync::Once = std::sync::Once::new();

/// `DialogAction.customClick(callback, options)`: the callback hears `PlayerCustomClickEvent` for an id of its
/// own. `ClickCallback.Options` aren't taken, so it stays for as long as the plugin runs and any number of uses.
pub fn dialog_callback_action(plugin: &Plugin, callback: impl DialogActionCallback) -> DialogAction {
    DIALOG_LISTENER.call_once(|| {
        plugin.register_event::<crate::fam_ev_player::PlayerCustomClickEvent>(|event| {
            let found = event.get_identifier().and_then(|id| locked(&DIALOG_CALLBACKS).get(id.as_str()).cloned());
            let audience = match event.get_common_connection() {
                Some(PlayerCommonConnection::Game(connection)) => Some(DialogAudience::Player(connection.get_player().clone())),
                Some(PlayerCommonConnection::Configuration(connection)) => Some(DialogAudience::Configuring(connection.get_audience())),
                None => None,
            };
            if let (Some(callback), Some(audience)) = (found, audience) {
                callback.accept(&event.get_dialog_response_view().unwrap_or_else(|| DialogResponseView(json!({}))), &audience);
            }
        });
    });
    let mut callbacks = locked(&DIALOG_CALLBACKS);
    let id = format!("{}:dialog_callback/{}", PluginMeta(*plugin).namespace(), callbacks.len());
    callbacks.insert(id.clone(), Arc::new(callback));
    DialogAction::custom_click(&id, None)
}

// ---- Resolving text ----

/// bukkit: io/papermc/paper/text/PaperComponents.resolveWithContext
///
/// As a command's text is: selectors, scores and `nbt` parts, from where `context` stands.
pub fn resolve_with_context(component: &Component, context: Option<&CommandSender>, subject: Option<&CommandExecutor>) -> Component {
    let player = context.and_then(CommandSender::as_player).map(Player::uuid);
    let mut request = json!({ "op": "paper_resolve", "component": component.0, "uuid": player });
    match subject {
        Some(CommandExecutor::Player(player)) => request["subject_player"] = json!(player.uuid()),
        Some(CommandExecutor::Entity(entity)) => request["subject"] = json!(entity.get_entity_id()),
        None => {}
    }
    let resolved = query(request);
    if resolved.is_null() { component.clone() } else { Component(resolved) }
}

// ---- Block inventories ----

/// `io.papermc.paper.block.TileStateInventoryHolder`: a block state with an inventory.
#[derive(Clone, Debug, PartialEq)]
pub struct TileStateInventoryHolder(pub crate::blocks::Furnace);

impl crate::block_data::StateClass for TileStateInventoryHolder {
    fn of(state: crate::block_data::TileState) -> Self {
        Self(crate::block_data::StateClass::of(state))
    }
}

impl TileStateInventoryHolder {
    /// bukkit: io/papermc/paper/block/TileStateInventoryHolder.getInventory
    pub fn get_inventory(&self) -> crate::fam_items::BlockInventory {
        self.0.get_inventory()
    }
    /// bukkit: io/papermc/paper/block/TileStateInventoryHolder.getSnapshotInventory
    pub fn get_snapshot_inventory(&mut self) -> crate::fam_items::SnapshotInventory<'_> {
        self.0.get_snapshot_inventory()
    }
}

// ---- Profiles ----

/// An event of the profile family, on every plugin: what its listeners left.
fn profile_event(event: &str, payload: Value) -> Value {
    query(json!({ "op": "ev_world_free_profile_fire", "event": event, "payload": payload }))
}

fn looked_up(answer: &Value) -> Option<(String, String, Vec<(String, String, Option<String>)>)> {
    let id = answer["id"].as_str()?.to_owned();
    let name = answer["name"].as_str()?.to_owned();
    let property = |row: &Value| Some((row["name"].as_str()?.to_owned(), row["value"].as_str()?.to_owned(), row["signature"].as_str().map(str::to_owned)));
    Some((id, name, answer["properties"].as_array().into_iter().flatten().filter_map(property).collect()))
}

impl PlayerProfile {
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.setProperties
    pub fn set_properties(&mut self, properties: &[crate::misc::ProfileProperty]) {
        for property in properties {
            self.set_property(property.get_name(), property.get_value(), property.get_signature());
        }
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.removeProperties
    pub fn remove_properties(&mut self, properties: &[crate::misc::ProfileProperty]) -> bool {
        properties.iter().fold(false, |any, property| self.remove_property(property.get_name()) || any)
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.setId
    ///
    /// The id it had.
    pub fn set_id(&mut self, id: Option<&str>) -> Option<String> {
        let previous = std::mem::replace(&mut self.id, id.unwrap_or_default().to_owned());
        (!previous.is_empty()).then_some(previous)
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.setName
    ///
    /// The name it had.
    pub fn set_name(&mut self, name: Option<&str>) -> Option<String> {
        let previous = std::mem::replace(&mut self.name, name.unwrap_or_default().to_owned());
        (!previous.is_empty()).then_some(previous)
    }
    fn fill_from(&mut self, name: &str, properties: Vec<(String, String, Option<String>)>) {
        if self.name.is_empty() {
            name.clone_into(&mut self.name);
        }
        for property in properties {
            if !self.has_property(&property.0) {
                self.properties.push(property);
            }
        }
    }
    /// `PreLookupProfileEvent` for a name, or `PreFillProfileEvent` for an id: what the listeners already
    /// know, in the session servers' form, if that saves asking them.
    fn before_services(&self) -> Option<Value> {
        let id = self.id.replace('-', "");
        let properties: Vec<Value> = self.properties.iter().map(|(name, value, signature)| json!({ "name": name, "value": value, "signature": signature })).collect();
        if id.is_empty() {
            let said = profile_event("ew_profile_prelookup", json!({ "uuid": null, "name": self.name, "properties": properties }));
            let found = said["uuid"].as_str()?;
            return Some(json!({ "id": found, "name": self.name, "properties": said["properties"] }));
        }
        let said = profile_event("ew_profile_prefill", json!({ "uuid": null, "id": self.id, "name": self.name, "properties": properties }));
        let filled = said["properties"].as_array()?.iter().any(|row| row["name"] == "textures");
        filled.then(|| json!({ "id": self.id, "name": self.name, "properties": said["properties"] }))
    }
    /// `LookupProfileEvent`, if it was a name, and `FillProfileEvent`, on the session servers' answer.
    fn after_services(&self, answer: &Value) {
        let Some((id, name, _)) = looked_up(answer) else {
            return;
        };
        let payload = json!({ "uuid": null, "id": id, "name": name, "properties": answer["properties"] });
        if self.id.is_empty() {
            profile_event("ew_profile_lookup", payload.clone());
        }
        profile_event("ew_profile_fill", payload);
    }
    /// The whole of a blocking lookup, with its events.
    fn services(&self, wait: u32) -> Value {
        if let Some(known) = self.before_services() {
            return known;
        }
        let answer = self.lookup(wait);
        self.after_services(&answer);
        answer
    }
    fn lookup(&self, wait: u32) -> Value {
        let id = self.id.replace('-', "");
        query(json!({ "op": "paper_profile_lookup", "id": (!id.is_empty()).then_some(id), "name": self.name, "wait": wait }))
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.completeFromCache
    ///
    /// From who is on and `usercache.json`. `lookup_uuid` asks the session servers for an id the cache
    /// hasn't got; `online_mode` of false gives the offline id instead, as `completeFromCache(false, false)`.
    pub fn complete_from_cache(&mut self, lookup_uuid: bool, online_mode: bool) -> bool {
        if self.id.is_empty() && !self.name.is_empty() {
            // Who is online or in the user cache. `get_player_unique_id` asks `get_offline_player`, which for a
            // name nobody has seen comes back here, for ever: a stack overflow that takes the server down.
            let cached = crate::fam_core::get_offline_player_if_cached(&self.name).and_then(|player| player.get_unique_id().map(str::to_owned));
            if let Some(id) = cached.filter(|_| online_mode) {
                self.id = id;
            } else if !online_mode {
                query(json!({ "op": "paper_offline_uuid", "name": self.name })).as_str().unwrap_or_default().clone_into(&mut self.id);
            } else if lookup_uuid && let Some((id, name, properties)) = looked_up(&self.services(5000)) {
                self.id = id;
                self.fill_from(&name, properties);
            }
        }
        if (self.name.is_empty() || !self.has_property("textures")) && !self.id.is_empty() {
            let wanted = self.id.clone();
            if let Some(player) = crate::get_online_players().into_iter().find(|player| player.get_unique_id() == wanted) {
                let found = player.get_player_profile();
                self.fill_from(&found.name, found.properties);
            } else if self.name.is_empty() {
                let known = crate::server::get_offline_players().into_iter().find(|player| player.get_unique_id() == Some(wanted.as_str()));
                if let Some(player) = known {
                    player.get_name().clone_into(&mut self.name);
                }
            }
        }
        self.is_complete()
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.complete
    ///
    /// Asks the session servers for what's missing, waiting up to five seconds as Java's call blocks: don't
    /// make it while players are waiting on the tick, `update` is for that.
    pub fn complete(&mut self, textures: bool, online_mode: bool) -> bool {
        let done = |profile: &Self| profile.is_complete() && (!textures || profile.has_property("textures"));
        self.complete_from_cache(online_mode, online_mode);
        if !done(self) && online_mode && let Some((id, name, properties)) = looked_up(&self.services(5000)) {
            if self.id.is_empty() {
                self.id = id;
            }
            self.fill_from(&name, properties);
        }
        done(self)
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.update, org/bukkit/profile/PlayerProfile.update
    ///
    /// Java's future is `callback`, on the server thread: a new profile with the official values, or this one
    /// if no such player exists.
    pub fn update(&self, callback: impl FnOnce(PlayerProfile) + Send + 'static) {
        let profile = self.clone();
        let mut callback = Some(callback);
        let mut known = None;
        let mut asked = false;
        scheduler::run_task_timer(1, 1, move || {
            if !asked {
                asked = true;
                known = profile.before_services();
            }
            let answer = known.clone().unwrap_or_else(|| profile.lookup(0));
            if answer["pending"] == true {
                return;
            }
            if known.is_none() {
                profile.after_services(&answer);
            }
            scheduler::cancel_self();
            let mut updated = profile.clone();
            if let Some((id, name, properties)) = looked_up(&answer) {
                updated.id = id;
                updated.name = name;
                updated.properties = properties;
            }
            if let Some(callback) = callback.take() {
                callback(updated);
            }
        });
    }
    fn profile_fields(&self) -> Value {
        let mut fields = Map::new();
        let id = self.id.replace('-', "");
        let words: Vec<i64> = (0..4).filter_map(|word| u32::from_str_radix(id.get(word * 8..(word + 1) * 8)?, 16).ok()).map(|word| i64::from(word as i32)).collect();
        if self.properties.is_empty() {
            if words.len() == 4 {
                fields.insert("id".to_owned(), json!(words));
            } else if !self.name.is_empty() {
                fields.insert("name".to_owned(), json!(self.name));
            }
            return Value::Object(fields);
        }
        if words.len() == 4 {
            fields.insert("id".to_owned(), json!(words));
        }
        if !self.name.is_empty() {
            fields.insert("name".to_owned(), json!(self.name));
        }
        let properties: Vec<Value> = self
            .properties
            .iter()
            .map(|(name, value, signature)| match signature {
                Some(signature) => json!({ "name": name, "value": value, "signature": signature }),
                None => json!({ "name": name, "value": value }),
            })
            .collect();
        fields.insert("properties".to_owned(), json!(properties));
        Value::Object(fields)
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.applySkinToPlayerHeadContents, org/bukkit/entity/Player.applySkinToPlayerHeadContents, org/bukkit/OfflinePlayer.applySkinToPlayerHeadContents
    ///
    /// Into a `player` object component being built.
    pub fn apply_skin_to_player_head_contents(&self, builder: &mut Map<String, Value>) {
        builder.insert("player".to_owned(), self.profile_fields());
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.asObjectContents, org/bukkit/entity/Player.asObjectContents
    ///
    /// A component that draws this player's head in text.
    pub fn as_object_contents(&self) -> Component {
        let mut contents = Map::new();
        contents.insert("type".to_owned(), json!("object"));
        contents.insert("object".to_owned(), json!("player"));
        self.apply_skin_to_player_head_contents(&mut contents);
        contents.insert("hat".to_owned(), json!(true));
        Component(Value::Object(contents))
    }
}

// ---- Folia's schedulers ----

static GLOBAL_TASKS: Mutex<Vec<ScheduledTask>> = Mutex::new(Vec::new());

/// bukkit: org/bukkit/Bukkit.getGlobalRegionScheduler, org/bukkit/Server.getGlobalRegionScheduler
pub fn get_global_region_scheduler() -> GlobalRegionScheduler {
    GlobalRegionScheduler
}

/// `io.papermc.paper.threadedregions.scheduler.GlobalRegionScheduler`: on Paper this runs on the server
/// thread, as here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlobalRegionScheduler;

impl GlobalRegionScheduler {
    fn keep(task: ScheduledTask) -> ScheduledTask {
        let mut tasks = locked(&GLOBAL_TASKS);
        tasks.retain(|known| matches!(known.load(), IDLE | RUNNING));
        tasks.push(task.clone());
        task
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/GlobalRegionScheduler.execute
    pub fn execute(&self, plugin: &Plugin, run: impl FnOnce() + Send + 'static) {
        let mut run = Some(run);
        self.run(plugin, move |_| {
            if let Some(run) = run.take() {
                run();
            }
        });
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/GlobalRegionScheduler.run
    pub fn run(&self, plugin: &Plugin, task: impl FnMut(&ScheduledTask) + Send + 'static) -> ScheduledTask {
        Self::keep(ScheduledTask::schedule(plugin, 1, None, task))
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/GlobalRegionScheduler.runDelayed
    ///
    /// # Panics
    /// As Java throws, for a delay under one.
    pub fn run_delayed(&self, plugin: &Plugin, task: impl FnMut(&ScheduledTask) + Send + 'static, delay: u64) -> ScheduledTask {
        assert!(delay >= 1, "Delay ticks must be > 0");
        Self::keep(ScheduledTask::schedule(plugin, delay, None, task))
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/GlobalRegionScheduler.runAtFixedRate
    ///
    /// # Panics
    /// As Java throws, for a delay or period under one.
    pub fn run_at_fixed_rate(&self, plugin: &Plugin, task: impl FnMut(&ScheduledTask) + Send + 'static, delay: u64, period: u64) -> ScheduledTask {
        assert!(delay >= 1 && period >= 1, "Delay and period must be > 0");
        Self::keep(ScheduledTask::schedule(plugin, delay, Some(period), task))
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/GlobalRegionScheduler.cancelTasks
    ///
    /// A plugin's library holds only its own tasks, so all of these.
    pub fn cancel_tasks(&self, _plugin: &Plugin) {
        for task in locked(&GLOBAL_TASKS).drain(..) {
            task.cancel();
        }
    }
}

/// Who an `EntityScheduler` follows.
#[derive(Clone, Debug, PartialEq)]
enum Followed {
    Player(Player),
    Entity(Entity),
}

impl Followed {
    fn alive(&self) -> bool {
        match self {
            Self::Player(player) => player.is_online(),
            Self::Entity(entity) => entity.is_valid(),
        }
    }
}

/// `io.papermc.paper.threadedregions.scheduler.EntityScheduler`: a task follows its entity, and `retired`
/// runs in its place once the entity is gone.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityScheduler {
    followed: Followed,
}

impl Entity {
    /// bukkit: org/bukkit/entity/Entity.getScheduler
    pub fn get_scheduler(&self) -> EntityScheduler {
        EntityScheduler { followed: Followed::Entity(self.clone()) }
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/Entity.getScheduler
    pub fn get_scheduler(&self) -> EntityScheduler {
        EntityScheduler { followed: Followed::Player(self.clone()) }
    }
}

impl EntityScheduler {
    fn start(&self, plugin: &Plugin, delay: u64, period: Option<u64>, mut task: impl FnMut(&ScheduledTask) + Send + 'static, retired: Option<Box<dyn FnOnce() + Send>>) -> Option<ScheduledTask> {
        if !self.followed.alive() {
            return None;
        }
        let followed = self.followed.clone();
        let mut retired = retired;
        Some(ScheduledTask::schedule(plugin, delay, period, move |scheduled| {
            if followed.alive() {
                task(scheduled);
                return;
            }
            scheduled.cancel();
            if let Some(retired) = retired.take() {
                retired();
            }
        }))
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/EntityScheduler.execute
    ///
    /// False if the entity is already gone.
    pub fn execute(&self, plugin: &Plugin, run: impl FnOnce() + Send + 'static, retired: Option<Box<dyn FnOnce() + Send>>, delay: u64) -> bool {
        let mut run = Some(run);
        let once = move |_: &ScheduledTask| {
            if let Some(run) = run.take() {
                run();
            }
        };
        self.start(plugin, delay.max(1), None, once, retired).is_some()
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/EntityScheduler.run
    pub fn run(&self, plugin: &Plugin, task: impl FnMut(&ScheduledTask) + Send + 'static, retired: Option<Box<dyn FnOnce() + Send>>) -> Option<ScheduledTask> {
        self.start(plugin, 1, None, task, retired)
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/EntityScheduler.runDelayed
    ///
    /// # Panics
    /// As Java throws, for a delay under one.
    pub fn run_delayed(&self, plugin: &Plugin, task: impl FnMut(&ScheduledTask) + Send + 'static, retired: Option<Box<dyn FnOnce() + Send>>, delay: u64) -> Option<ScheduledTask> {
        assert!(delay >= 1, "Delay ticks must be > 0");
        self.start(plugin, delay, None, task, retired)
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/EntityScheduler.runAtFixedRate
    ///
    /// # Panics
    /// As Java throws, for a delay or period under one.
    pub fn run_at_fixed_rate(&self, plugin: &Plugin, task: impl FnMut(&ScheduledTask) + Send + 'static, retired: Option<Box<dyn FnOnce() + Send>>, delay: u64, period: u64) -> Option<ScheduledTask> {
        assert!(delay >= 1 && period >= 1, "Delay and period must be > 0");
        self.start(plugin, delay, Some(period), task, retired)
    }
}

impl crate::brigadier::AsyncScheduler {
    /// bukkit: io/papermc/paper/threadedregions/scheduler/AsyncScheduler.runNow
    pub fn run_now_task(&self, plugin: &Plugin, mut work: impl FnMut(&ScheduledTask) + Send + 'static) -> ScheduledTask {
        let task = ScheduledTask::new(plugin, false);
        locked(&ASYNC_TASKS).push(task.clone());
        let handle = task.clone();
        scheduler::run_task_asynchronously(move || {
            handle.step(&mut work);
        });
        task
    }
}

// ---- Who runs a command ----

/// `CommandSourceStack.getExecutor`: who a command runs as.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandExecutor {
    Player(Player),
    Entity(Entity),
}

impl CommandContext {
    /// What the game's command source says, for a command `/execute` moved or passed to someone else.
    pub(crate) fn from_source(mut self) -> Self {
        let source = query(json!({ "op": "paper_command_source" }));
        if let Some(at) = source["pos"].as_array().filter(|at| at.len() == 3) {
            let world = World::of(crate::world_of(source["dimension"].as_str().unwrap_or("minecraft:overworld")));
            let yaw = source["rot"][0].as_f64().unwrap_or(0.0) as f32;
            let pitch = source["rot"][1].as_f64().unwrap_or(0.0) as f32;
            self.location = Some(Location::new(world.clone(), at[0].as_f64().unwrap_or(0.0), at[1].as_f64().unwrap_or(0.0), at[2].as_f64().unwrap_or(0.0), yaw, pitch));
            if let Some(uuid) = source["player"].as_str() {
                self.executor = Player::read(&query(json!({ "op": "player", "uuid": uuid }))).map(CommandExecutor::Player);
            } else if let Some(id) = source["entity"].as_u64() {
                self.executor = Some(CommandExecutor::Entity(Entity::of(id, &world)));
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.getExecutor
    pub fn get_executor_entity(&self) -> Option<CommandExecutor> {
        self.executor.clone().or_else(|| self.get_sender().as_player().cloned().map(CommandExecutor::Player))
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.withExecutor
    pub fn with_executor(&self, executor: CommandExecutor) -> Self {
        let mut copy = self.clone();
        copy.executor = Some(executor);
        copy
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.getEntityOrThrow
    ///
    /// `Err` is Java's `CommandSyntaxException`.
    pub fn get_entity_or_throw(&self) -> Result<CommandExecutor, String> {
        self.get_executor_entity().ok_or_else(|| "An entity is required to run this command here".to_owned())
    }
    /// bukkit: com/destroystokyo/paper/brigadier/BukkitBrigadierCommandSource.getBukkitEntity
    pub fn get_bukkit_entity(&self) -> Option<CommandExecutor> {
        self.get_executor_entity()
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.sendSuccess
    ///
    /// `CommandSourceStack.sendSuccess`: to the source, then to the operators and the log as the rules say.
    pub fn send_success(&self, message: &Component, inform_admins: bool) {
        let rule = |name: &str, default: bool| World::of("world").get_game_rule_value(name).map_or(default, |value| value == "true");
        let sender = self.get_sender();
        let console = sender.as_player().is_none();
        if console || rule("send_command_feedback", true) {
            self.send_reply(message);
        }
        if !inform_admins || (console && !rule("broadcast_console_to_ops", true)) {
            return;
        }
        let line = Component(json!({ "translate": "chat.type.admin", "with": [sender.get_name(), message.0], "color": "gray", "italic": true }));
        if rule("send_command_feedback", true) {
            for admin in crate::get_online_players().iter().filter(|player| player.is_op() && Some(*player) != sender.as_player()) {
                admin.send_component(&line);
            }
        }
        if !console && rule("log_admin_commands", true) {
            crate::log(&format!("[{}: {}]", sender.get_name(), message.plain()));
        }
    }
}

impl crate::events::RegionizedServerInitEvent {
    /// bukkit: io/papermc/paper/threadedregions/RegionizedServerInitEvent.getHandlers
    pub fn get_handlers(&self) -> crate::event::HandlerList {
        crate::event::HandlerList
    }
    /// bukkit: io/papermc/paper/threadedregions/RegionizedServerInitEvent.getHandlerList
    pub fn get_handler_list() -> crate::event::HandlerList {
        crate::event::HandlerList
    }
}

// ---- Connections before they are players ----

fn profile_of(row: &Value) -> PlayerProfile {
    let id = row["uuid"].as_str().unwrap_or_default();
    let dashed = if id.len() == 32 { format!("{}-{}-{}-{}-{}", &id[..8], &id[8..12], &id[12..16], &id[16..20], &id[20..]) } else { id.to_owned() };
    let mut profile = create_player_profile(&dashed, row["name"].as_str().unwrap_or_default());
    for property in row["properties"].as_array().into_iter().flatten() {
        profile.set_property(property["name"].as_str().unwrap_or_default(), property["value"].as_str().unwrap_or_default(), property["signature"].as_str());
    }
    profile
}

/// `com.destroystokyo.paper.network.NetworkClient`: someone who's connected, as they said at their handshake.
pub trait NetworkClient {
    fn connection(&self) -> &Value;
    /// bukkit: com/destroystokyo/paper/network/NetworkClient.getAddress, com/destroystokyo/paper/network/NetworkClient.getSocketAddress
    fn get_address(&self) -> Option<String> {
        self.connection()["address"].as_str().map(str::to_owned)
    }
    /// bukkit: com/destroystokyo/paper/network/NetworkClient.getProtocolVersion
    fn get_protocol_version(&self) -> i32 {
        self.connection()["protocol"].as_i64().map_or(-1, |protocol| protocol as i32)
    }
    /// bukkit: com/destroystokyo/paper/network/NetworkClient.getVirtualHost
    fn get_virtual_host(&self) -> Option<String> {
        self.connection()["host"].as_str().map(str::to_owned)
    }
}

/// `com.destroystokyo.paper.network.StatusClient`.
#[derive(Clone, Debug, PartialEq)]
pub struct StatusClient(pub Value);

impl NetworkClient for StatusClient {
    fn connection(&self) -> &Value {
        &self.0
    }
}

impl StatusClient {
    /// bukkit: com/destroystokyo/paper/network/StatusClient.isLegacy
    ///
    /// The game answers the old ping form itself, so what a listener sees never is.
    pub fn is_legacy(&self) -> bool {
        false
    }
}

impl crate::fam_ev_world::PaperServerListPingEvent {
    /// bukkit: com/destroystokyo/paper/event/server/PaperServerListPingEvent.getClient
    pub fn get_client(&self) -> StatusClient {
        StatusClient(query(json!({ "op": "ev_world_free_ping_client", "id": self.id })))
    }
}

/// `io.papermc.paper.connection.PlayerLoginConnection`, with `PlayerConnection`: someone logging in.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerLoginConnection(pub Value);

impl NetworkClient for PlayerLoginConnection {
    fn connection(&self) -> &Value {
        &self.0
    }
}

impl PlayerLoginConnection {
    /// bukkit: io/papermc/paper/connection/PlayerLoginConnection.getAuthenticatedProfile
    pub fn get_authenticated_profile(&self) -> Option<PlayerProfile> {
        Some(profile_of(&self.0))
    }
    /// bukkit: io/papermc/paper/connection/PlayerLoginConnection.getUnsafeProfile
    ///
    /// The name and id the client sent, before anything checked them.
    pub fn get_unsafe_profile(&self) -> Option<PlayerProfile> {
        let hello = self.0.get("hello").filter(|hello| !hello.is_null())?;
        Some(profile_of(hello))
    }
}

/// `io.papermc.paper.connection.PlayerConfigurationConnection`: someone in the configuration phase. Its packets
/// go out from any thread, and at once, while they're held there.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerConfigurationConnection(pub Value);

impl NetworkClient for PlayerConfigurationConnection {
    fn connection(&self) -> &Value {
        &self.0
    }
}

/// `PlayerConfigurationConnection.getAudience`: what can reach someone still configuring.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigurationAudience {
    connection: PlayerConfigurationConnection,
}

impl ConfigurationAudience {
    pub fn show_dialog(&self, dialog: &Dialog) -> bool {
        self.connection.send(json!({ "packet": "dialog", "snbt": snbt(&dialog.0) }))
    }
    pub fn close_dialog(&self) -> bool {
        self.connection.send(json!({ "packet": "clear_dialog" }))
    }
    /// A pack by id, as `Player::add_resource_pack` has it.
    pub fn send_resource_pack(&self, id: &str, url: &str, hash: &str, required: bool, prompt: Option<&Component>) -> bool {
        self.connection.send(json!({ "packet": "pack_push", "id": id.replace('-', ""), "url": url, "hash": hash, "required": required, "prompt": prompt.map(|prompt| &prompt.0) }))
    }
    pub fn remove_resource_pack(&self, id: &str) -> bool {
        self.connection.send(json!({ "packet": "pack_pop", "id": id.replace('-', "") }))
    }
    pub fn clear_resource_packs(&self) -> bool {
        self.connection.send(json!({ "packet": "pack_pop", "id": null }))
    }
    /// What `pointers` holds: `Identity.NAME`, `Identity.UUID` and `Identity.LOCALE`.
    pub fn get_name(&self) -> Option<&str> {
        self.connection.0["name"].as_str()
    }
    pub fn get_unique_id(&self) -> String {
        crate::dashed(self.connection.0["uuid"].as_str().unwrap_or_default())
    }
    pub fn get_locale(&self) -> Option<&str> {
        self.connection.0["info"]["locale"].as_str()
    }
}

impl PlayerConfigurationConnection {
    fn send(&self, packet: Value) -> bool {
        let mut request = packet;
        request["op"] = json!("paper_connection_send");
        request["ask"] = self.0["ask"].clone();
        request["held"] = self.0["held"].clone();
        query(request).as_bool().unwrap_or(false)
    }
    /// bukkit: io/papermc/paper/connection/PlayerConfigurationConnection.getProfile
    pub fn get_profile(&self) -> PlayerProfile {
        profile_of(&self.0)
    }
    /// bukkit: io/papermc/paper/connection/PlayerConfigurationConnection.getAudience
    pub fn get_audience(&self) -> ConfigurationAudience {
        ConfigurationAudience { connection: self.clone() }
    }
    /// bukkit: io/papermc/paper/connection/PlayerConfigurationConnection.clearChat
    pub fn clear_chat(&self) -> bool {
        self.send(json!({ "packet": "clear_chat" }))
    }
    pub fn disconnect(&self, reason: &Component) -> bool {
        query(json!({ "op": "paper_connection_answer", "ask": self.0["ask"], "held": self.0["held"], "kick": reason.0 })).as_bool().unwrap_or(false)
    }
    pub fn is_connected(&self) -> bool {
        query(json!({ "op": "paper_connection_open", "held": self.0["held"] })) == json!(true)
    }
    pub fn send_links(&self, links: &[ServerLinkEntry]) -> bool {
        self.send(json!({ "packet": "links", "links": link_rows(links) }))
    }
    pub fn send_report_details(&self, details: &BTreeMap<String, String>) -> bool {
        self.send(json!({ "packet": "report_details", "details": details }))
    }
    /// As `PlayerGameConnection::retrieve_cookie`.
    pub fn retrieve_cookie(&self, plugin: &Plugin, key: &NamespacedKey, callback: impl FnOnce(Option<Vec<u8>>) + Send + 'static) {
        wait_for_cookie(plugin, self.0["uuid"].as_str().unwrap_or_default(), key, callback);
        self.send(json!({ "packet": "cookie_request", "key": key.as_str() }));
    }
    pub fn transfer(&self, host: &str, port: i32) -> bool {
        self.send(json!({ "packet": "transfer", "host": host, "port": port }))
    }
    pub fn store_cookie(&self, key: &NamespacedKey, value: &[u8]) -> bool {
        self.send(json!({ "packet": "store_cookie", "key": key.as_str(), "data": value }))
    }
    pub fn get_client_brand_name(&self) -> Option<&str> {
        self.0["brand"].as_str()
    }
    pub fn is_transferred(&self) -> bool {
        self.0["transferred"].as_bool().unwrap_or(false)
    }
    /// What the client said of itself: as `Player::get_client_option`, by the same names.
    pub fn get_client_option(&self, option: &ClientOption) -> Value {
        let info = &self.0["info"];
        match option.name {
            "LOCALE" => info["locale"].clone(),
            "VIEW_DISTANCE" => info["view_distance"].clone(),
            "CHAT_COLORS_ENABLED" => info["chat_colors"].clone(),
            "CHAT_VISIBILITY" => info["chat_visibility"].clone(),
            "MAIN_HAND" => json!(if info["main_hand"] == 0 { "LEFT" } else { "RIGHT" }),
            "TEXT_FILTERING_ENABLED" => info["text_filtering"].clone(),
            "ALLOW_SERVER_LISTINGS" => info["allows_listing"].clone(),
            "SKIN_PARTS" => info["skin_parts"].clone(),
            "PARTICLE_VISIBILITY" => info["particle_status"].clone(),
            _ => Value::Null,
        }
    }
}

fn asks_of(kind: &str) -> Vec<Value> {
    query(json!({ "op": "paper_connection_asks" })).as_array().into_iter().flatten().filter(|row| row["kind"] == kind).cloned().collect()
}

/// bukkit: io/papermc/paper/event/connection/PlayerConnectionValidateLoginEvent
pub struct PlayerConnectionValidateLoginEvent {
    connection: PlayerLoginConnection,
    kick: Option<Component>,
}

impl crate::event::Event for PlayerConnectionValidateLoginEvent {
    const HOOK: &'static str = "tick";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        if !value.is_null() {
            return Vec::new();
        }
        asks_of("validate").into_iter().map(|row| Self { kick: (!row["kick"].is_null()).then(|| Component(row["kick"].clone())), connection: PlayerLoginConnection(row) }).collect()
    }
    fn finish(&self) {
        query(json!({ "op": "paper_connection_answer", "ask": self.connection.0["ask"], "kick": self.kick.as_ref().map(|kick| &kick.0) }));
    }
}

impl PlayerConnectionValidateLoginEvent {
    /// bukkit: io/papermc/paper/event/connection/PlayerConnectionValidateLoginEvent.getConnection
    pub fn get_connection(&self) -> &PlayerLoginConnection {
        &self.connection
    }
    /// bukkit: io/papermc/paper/event/connection/PlayerConnectionValidateLoginEvent.allow
    pub fn allow(&mut self) {
        self.kick = None;
    }
    /// bukkit: io/papermc/paper/event/connection/PlayerConnectionValidateLoginEvent.kickMessage
    pub fn kick_message(&mut self, message: &Component) {
        self.kick = Some(message.clone());
    }
    /// bukkit: io/papermc/paper/event/connection/PlayerConnectionValidateLoginEvent.getKickMessage
    pub fn get_kick_message(&self) -> Option<&Component> {
        self.kick.as_ref()
    }
    /// bukkit: io/papermc/paper/event/connection/PlayerConnectionValidateLoginEvent.isAllowed
    pub fn is_allowed(&self) -> bool {
        self.kick.is_none()
    }
}

/// bukkit: io/papermc/paper/event/connection/configuration/PlayerConnectionInitialConfigureEvent
pub struct PlayerConnectionInitialConfigureEvent {
    connection: PlayerConfigurationConnection,
}

impl crate::event::Event for PlayerConnectionInitialConfigureEvent {
    const HOOK: &'static str = "tick";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        if !value.is_null() {
            return Vec::new();
        }
        asks_of("configure_initial").into_iter().map(|row| Self { connection: PlayerConfigurationConnection(row) }).collect()
    }
}

impl PlayerConnectionInitialConfigureEvent {
    /// bukkit: io/papermc/paper/event/connection/configuration/PlayerConnectionInitialConfigureEvent.getConnection
    pub fn get_connection(&self) -> &PlayerConfigurationConnection {
        &self.connection
    }
}

/// bukkit: io/papermc/paper/event/connection/configuration/AsyncPlayerConnectionConfigureEvent
///
/// On a thread of its own for each connection, as on Paper: the client waits in configuration, kept alive, until
/// the listeners return, so one can block on a dialog's answer for as long as it likes.
pub struct AsyncPlayerConnectionConfigureEvent {
    connection: PlayerConfigurationConnection,
}

impl crate::event::Event for AsyncPlayerConnectionConfigureEvent {
    const HOOK: &'static str = "paper_configure";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { connection: PlayerConfigurationConnection(value.clone()) })
    }
    fn is_asynchronous(&self) -> bool {
        true
    }
}

impl AsyncPlayerConnectionConfigureEvent {
    /// bukkit: io/papermc/paper/event/connection/configuration/AsyncPlayerConnectionConfigureEvent.getConnection
    pub fn get_connection(&self) -> &PlayerConfigurationConnection {
        &self.connection
    }
}

// ---- Combat trackers ----

fn entries_of(answer: &Value, world: &World) -> Vec<CombatEntry> {
    answer["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| {
            let mut source = crate::core2::DamageSource::builder(row["damage_type"].as_str().unwrap_or("minecraft:generic"));
            if let Some(id) = row["attacker"].as_u64() {
                source = source.with_causing_entity(&Entity::of(id, world));
            }
            CombatEntry::combat_entry(&source, row["damage"].as_f64().unwrap_or(0.0) as f32, row["fall_location"].as_str(), row["fall_distance"].as_f64().unwrap_or(0.0) as f32)
        })
        .collect()
}

fn entry_row(entry: &CombatEntry) -> Value {
    let attacker = entry.source.get_causing_entity();
    json!({ "damage_type": entry.source.get_damage_type(), "damage": entry.damage, "fall_location": entry.fall_type, "fall_distance": entry.fall_distance, "attacker": attacker.map(Entity::get_entity_id), "attacker_name": attacker.and_then(Entity::get_custom_name).or_else(|| attacker.map(Entity::get_type)) })
}

/// `io.papermc.paper.world.damagesource.CombatTracker`.
#[derive(Clone, Debug, PartialEq)]
pub struct CombatTracker {
    followed: Followed,
}

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getCombatTracker
    pub fn get_combat_tracker(&self) -> CombatTracker {
        CombatTracker { followed: Followed::Entity(self.clone()) }
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/LivingEntity.getCombatTracker
    pub fn get_combat_tracker(&self) -> CombatTracker {
        CombatTracker { followed: Followed::Player(self.clone()) }
    }
}

impl CombatTracker {
    fn ask(&self, change: &str, rows: Vec<Value>, death: bool) -> Value {
        let mut request = json!({ "op": "paper_combat", "change": change, "rows": rows, "death": death });
        match &self.followed {
            Followed::Player(player) => request["uuid"] = json!(player.uuid()),
            Followed::Entity(entity) => {
                request["id"] = json!(entity.get_entity_id());
                request["dimension"] = json!(entity.get_world().dimension());
            }
        }
        query(request)
    }
    fn world(&self) -> World {
        match &self.followed {
            Followed::Player(player) => player.get_world(),
            Followed::Entity(entity) => entity.get_world(),
        }
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.getEntity
    pub fn get_entity(&self) -> CommandExecutor {
        match &self.followed {
            Followed::Player(player) => CommandExecutor::Player(player.clone()),
            Followed::Entity(entity) => CommandExecutor::Entity(entity.clone()),
        }
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.getCombatEntries
    pub fn get_combat_entries(&self) -> Vec<CombatEntry> {
        entries_of(&self.ask("read", Vec::new(), false), &self.world())
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.setCombatEntries
    pub fn set_combat_entries(&self, entries: &[CombatEntry]) {
        self.ask("set", entries.iter().map(entry_row).collect(), false);
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.addCombatEntry
    pub fn add_combat_entry(&self, entry: &CombatEntry) {
        self.ask("add", vec![entry_row(entry)], false);
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.resetCombatState
    pub fn reset_combat_state(&self) {
        self.ask("reset", Vec::new(), false);
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.isInCombat
    pub fn is_in_combat(&self) -> bool {
        self.ask("read", Vec::new(), false)["in_combat"].as_bool().unwrap_or(false)
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.isTakingDamage
    ///
    /// The tracker clears its hits as the moment passes, so anything in it means it's still on.
    pub fn is_taking_damage(&self) -> bool {
        self.ask("read", Vec::new(), false)["entries"].as_array().is_some_and(|entries| !entries.is_empty())
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.getLastDamageTime
    pub fn get_last_damage_time(&self) -> i64 {
        self.ask("read", Vec::new(), false)["last_damage"].as_i64().unwrap_or(0)
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.getCombatDuration
    pub fn get_combat_duration(&self) -> i64 {
        self.ask("read", Vec::new(), false)["duration"].as_i64().unwrap_or(0)
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.getDeathMessage
    pub fn get_death_message(&self) -> Component {
        Component(self.ask("read", Vec::new(), true)["death"].clone())
    }
    /// bukkit: io/papermc/paper/world/damagesource/CombatTracker.computeMostSignificantFall
    ///
    /// `CombatTracker.getMostSignificantFall`.
    pub fn compute_most_significant_fall(&self) -> Option<CombatEntry> {
        let entries = self.get_combat_entries();
        let in_tag = |entry: &CombatEntry, tag: &str| crate::registry::Tag::get("damage_type", tag).is_tagged(entry.source.get_damage_type());
        let (mut fall, mut fallback, mut best_fall, mut best_damage) = (None, None, 0.0_f32, 0.0_f32);
        for (at, entry) in entries.iter().enumerate() {
            let always = in_tag(entry, "always_most_significant_fall");
            let distance = if always { f32::MAX } else { entry.fall_distance };
            if (in_tag(entry, "is_fall") || always) && distance > 0.0 && (fall.is_none() || distance > best_fall) {
                fall = Some(at.saturating_sub(1));
                best_fall = distance;
            }
            if entry.fall_type.is_some() && (fallback.is_none() || entry.damage > best_damage) {
                fallback = Some(at);
                best_damage = entry.damage;
            }
        }
        if best_fall > 5.0 {
            return fall.map(|at| entries[at].clone());
        }
        if best_damage > 5.0 { fallback.map(|at| entries[at].clone()) } else { None }
    }
}

// ---- Lootable inventories ----

/// `com.destroystokyo.paper.loottable.LootableInventory`: Paper's record of a container with a loot table.
/// Paper's refills are off unless a server turns them on, and Horizon has no setting for them, so a container
/// fills once, as vanilla's. The record is kept in the world's `paper-lootables.json`, from when the game first
/// runs this code.
#[derive(Clone, Debug, PartialEq)]
pub enum LootableInventory {
    Block(Block),
    Entity(Entity),
}

impl LootableInventory {
    fn key(&self) -> String {
        match self {
            Self::Block(block) => format!("{}/{}/{}/{}", block.get_world().dimension().trim_start_matches("minecraft:"), block.get_x(), block.get_y(), block.get_z()),
            Self::Entity(entity) => entity.get_unique_id().replace('-', ""),
        }
    }
    fn record(&self) -> Value {
        query(json!({ "op": "paper_lootable", "key": self.key() }))
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableBlockInventory.getBlock
    pub fn get_block(&self) -> Option<&Block> {
        match self {
            Self::Block(block) => Some(block),
            Self::Entity(_) => None,
        }
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableEntityInventory.getEntity
    pub fn get_entity(&self) -> Option<&Entity> {
        match self {
            Self::Entity(entity) => Some(entity),
            Self::Block(_) => None,
        }
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.isRefillEnabled
    ///
    /// Paper's own default, as Horizon has no setting to turn it on.
    pub fn is_refill_enabled(&self) -> bool {
        false
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.hasBeenFilled
    pub fn has_been_filled(&self) -> bool {
        self.get_last_filled() != -1
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.hasPlayerLooted
    pub fn has_player_looted(&self, player: &Player) -> bool {
        self.get_last_looted(player).is_some()
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.canPlayerLoot
    ///
    /// Paper's `restrict-player-reloot`, on by default: someone who has looted this can't again.
    pub fn can_player_loot(&self, player: &Player) -> bool {
        !self.has_player_looted(player)
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.getLastLooted
    pub fn get_last_looted(&self, player: &Player) -> Option<i64> {
        self.record()["looted"][player.uuid()].as_i64()
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.setHasPlayerLooted
    ///
    /// Whether they had.
    pub fn set_has_player_looted(&self, player: &Player, looted: bool) -> bool {
        let had = self.has_player_looted(player);
        query(json!({ "op": "paper_lootable", "key": self.key(), "looted": { "player": player.uuid(), "on": looted } }));
        had
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.hasPendingRefill
    pub fn has_pending_refill(&self) -> bool {
        let next = self.get_next_refill();
        next != -1 && next > self.get_last_filled()
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.getLastFilled
    pub fn get_last_filled(&self) -> i64 {
        self.record()["filled"].as_i64().unwrap_or(-1)
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.getNextRefill
    pub fn get_next_refill(&self) -> i64 {
        self.record()["next"].as_i64().unwrap_or(-1)
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventory.setNextRefill
    ///
    /// Kept. Nothing refills at that time, as on a Paper with `lootables.auto-replenish` at its default of off.
    pub fn set_next_refill(&self, at: i64) -> i64 {
        let previous = self.get_next_refill();
        query(json!({ "op": "paper_lootable", "key": self.key(), "next": at }));
        previous
    }
}

/// `com.destroystokyo.paper.loottable.LootableInventoryReplenishEvent`: a refill, which never comes on Horizon
/// any more than on a Paper with them off, as they are by default, so a listener never hears one.
pub struct LootableInventoryReplenishEvent {
    inventory: LootableInventory,
    cancelled: bool,
}

impl crate::event::Event for LootableInventoryReplenishEvent {
    const HOOK: &'static str = "paper_loot_replenish";
    fn read(_: &Value) -> Option<Self> {
        None
    }
}

impl LootableInventoryReplenishEvent {
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventoryReplenishEvent.getInventory
    pub fn get_inventory(&self) -> &LootableInventory {
        &self.inventory
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventoryReplenishEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventoryReplenishEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventoryReplenishEvent.getHandlers
    pub fn get_handlers(&self) -> crate::event::HandlerList {
        crate::event::HandlerList
    }
    /// bukkit: com/destroystokyo/paper/loottable/LootableInventoryReplenishEvent.getHandlerList
    pub fn get_handler_list() -> crate::event::HandlerList {
        crate::event::HandlerList
    }
}
