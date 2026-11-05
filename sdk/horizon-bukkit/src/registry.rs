//! `org.bukkit.Registry`, `Tag`, and the classes whose entries are the game's own data: enchantments,
//! potion effects, entity types. Answered from the data the game loaded.
use crate::entity::Entity;
use crate::event2::{EntityDamageByEntityEvent, EntityDeathEvent};
use crate::event::PlayerDeathEvent;
use crate::meta_lists::FireworkEffect;
use crate::persistence::NamespacedKey;
use crate::{Color, ItemStack, Material, Player};
use horizon_mod::{Value, query};
use serde_json::json;

fn data_file(path: &str) -> Value {
    query(json!({ "op": "data_file", "path": path }))
}

fn key_of(name: &str) -> String {
    if name.contains(':') { name.to_owned() } else { format!("minecraft:{}", name.to_ascii_lowercase()) }
}

type TagSet = std::sync::Arc<(Vec<String>, std::collections::HashSet<String>)>;

/// A tag's members, kept for a few seconds: a tag changes only when datapacks reload, and a plugin asks the same
/// one for every block of an explosion (CoreProtect's `Tag.ALL_SIGNS.isTagged` per block made one TNT blast cost
/// 1.8 s at ~1 ms a query, ~20 ms off the server thread).
fn tag_members(kind: &str, name: &str) -> TagSet {
    static KEPT: std::sync::Mutex<Option<std::collections::HashMap<String, (std::time::Instant, TagSet)>>> = std::sync::Mutex::new(None);
    let key = format!("{kind}|{name}");
    if let Some((at, set)) = KEPT.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Default::default).get(&key)
        && at.elapsed() < std::time::Duration::from_secs(5)
    {
        return set.clone();
    }
    let list: Vec<String> = query(json!({ "op": "tag", "kind": kind, "name": name })).as_array().into_iter().flatten().filter_map(|id| id.as_str().map(str::to_owned)).collect();
    let set: TagSet = std::sync::Arc::new((list.clone(), list.into_iter().collect()));
    KEPT.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Default::default).insert(key, (std::time::Instant::now(), set.clone()));
    set
}

/// `org.bukkit.Registry`: the entries of a folder of the game's data, or `report:` and one of the
/// game's built-in registries, from its registry report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Registry {
    pub(crate) folder: &'static str,
}

impl Registry {
    pub const ENCHANTMENT: Self = Self { folder: "enchantment" };
    pub const BIOME: Self = Self { folder: "worldgen/biome" };
    pub const DAMAGE_TYPE: Self = Self { folder: "damage_type" };
    pub const INSTRUMENT: Self = Self { folder: "instrument" };
    pub const JUKEBOX_SONG: Self = Self { folder: "jukebox_song" };
    pub const PAINTING_VARIANT: Self = Self { folder: "painting_variant" };
    pub const TRIM_MATERIAL: Self = Self { folder: "trim_material" };
    pub const TRIM_PATTERN: Self = Self { folder: "trim_pattern" };
    pub const BANNER_PATTERN: Self = Self { folder: "banner_pattern" };
    pub const WOLF_VARIANT: Self = Self { folder: "wolf_variant" };
    pub const CAT_VARIANT: Self = Self { folder: "cat_variant" };
    pub const FROG_VARIANT: Self = Self { folder: "frog_variant" };
    pub const STRUCTURE: Self = Self { folder: "worldgen/structure" };
    pub const LOOT_TABLES: Self = Self { folder: "loot_table" };
    pub const WOLF_SOUND_VARIANT: Self = Self { folder: "wolf_sound_variant" };
    pub const CAT_SOUND_VARIANT: Self = Self { folder: "cat_sound_variant" };
    pub const CHICKEN_VARIANT: Self = Self { folder: "chicken_variant" };
    pub const CHICKEN_SOUND_VARIANT: Self = Self { folder: "chicken_sound_variant" };
    pub const COW_VARIANT: Self = Self { folder: "cow_variant" };
    pub const COW_SOUND_VARIANT: Self = Self { folder: "cow_sound_variant" };
    pub const PIG_VARIANT: Self = Self { folder: "pig_variant" };
    pub const PIG_SOUND_VARIANT: Self = Self { folder: "pig_sound_variant" };
    pub const ZOMBIE_NAUTILUS_VARIANT: Self = Self { folder: "zombie_nautilus_variant" };
    pub const SULFUR_CUBE_ARCHETYPE: Self = Self { folder: "sulfur_cube_archetype" };
    pub const DIALOG: Self = Self { folder: "dialog" };
    pub const GAME_EVENT: Self = Self { folder: "report:minecraft:game_event" };
    pub const STRUCTURE_TYPE: Self = Self { folder: "report:minecraft:structure_type" };
    pub const MOB_EFFECT: Self = Self { folder: "report:minecraft:mob_effect" };
    pub const BLOCK: Self = Self { folder: "report:minecraft:block" };
    pub const ITEM: Self = Self { folder: "report:minecraft:item" };
    pub const VILLAGER_PROFESSION: Self = Self { folder: "report:minecraft:villager_profession" };
    pub const VILLAGER_TYPE: Self = Self { folder: "report:minecraft:villager_type" };
    pub const MAP_DECORATION_TYPE: Self = Self { folder: "report:minecraft:map_decoration_type" };
    pub const MENU: Self = Self { folder: "report:minecraft:menu" };
    pub const ATTRIBUTE: Self = Self { folder: "report:minecraft:attribute" };
    pub const FLUID: Self = Self { folder: "report:minecraft:fluid" };
    pub const SOUND_EVENT: Self = Self { folder: "report:minecraft:sound_event" };
    pub const DATA_COMPONENT_TYPE: Self = Self { folder: "report:minecraft:data_component_type" };
    pub const GAME_RULE: Self = Self { folder: "report:minecraft:game_rule" };
    pub const POINT_OF_INTEREST_TYPE: Self = Self { folder: "report:minecraft:point_of_interest_type" };
    pub const ENTITY_TYPE: Self = Self { folder: "report:minecraft:entity_type" };
    pub const PARTICLE_TYPE: Self = Self { folder: "report:minecraft:particle_type" };
    pub const POTION: Self = Self { folder: "report:minecraft:potion" };
    pub const MEMORY_MODULE_TYPE: Self = Self { folder: "report:minecraft:memory_module_type" };

    /// bukkit: org/bukkit/Registry.keyStream, org/bukkit/Registry.iterator
    pub fn keys(&self) -> Vec<String> {
        let names = match self.folder.strip_prefix("report:") {
            Some(registry) => query(json!({ "op": "core_registry", "registry": registry })),
            None => query(json!({ "op": "data_list", "path": self.folder })),
        };
        names.as_array().into_iter().flatten().filter_map(|key| key.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/Registry.get
    ///
    /// The key, if the registry has it, as these entries are named by their keys here.
    pub fn get(&self, key: &str) -> Option<String> {
        let key = key_of(key);
        self.keys().into_iter().find(|known| *known == key)
    }
    /// bukkit: org/bukkit/Registry.getOrThrow
    ///
    /// # Panics
    /// As Java throws, for a key that isn't there.
    pub fn get_or_throw(&self, key: &str) -> String {
        self.get(key).unwrap_or_else(|| panic!("no {key} in {}", self.folder))
    }
    /// bukkit: org/bukkit/Registry.match
    pub fn match_key(&self, text: &str) -> Option<String> {
        self.get(&text.trim().to_ascii_lowercase().replace(' ', "_"))
    }
    /// bukkit: org/bukkit/Registry.size
    pub fn size(&self) -> usize {
        self.keys().len()
    }
    /// bukkit: org/bukkit/Registry.stream
    pub fn stream(&self) -> impl Iterator<Item = String> {
        self.keys().into_iter()
    }
    /// bukkit: org/bukkit/Registry.getKey, org/bukkit/Registry.getKeyOrThrow
    pub fn get_key(&self, value: &str) -> Option<NamespacedKey> {
        self.get(value).and_then(|key| NamespacedKey::from_string(&key))
    }
}

/// `org.bukkit.Tag`: a name in one of the game's tag folders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    kind: &'static str,
    name: String,
}

impl Tag {
    /// bukkit: org/bukkit/Bukkit.getTag, org/bukkit/Server.getTag
    ///
    /// `kind` is the game's folder: `block`, `item`, `entity_type`, `fluid`.
    pub fn get(kind: &'static str, name: &str) -> Self {
        Self { kind, name: name.to_owned() }
    }
    /// bukkit: org/bukkit/Bukkit.getTags, org/bukkit/Server.getTags
    pub fn all(kind: &'static str) -> Vec<Self> {
        query(json!({ "op": "tags", "kind": kind })).as_array().into_iter().flatten().filter_map(|name| Some(Self { kind, name: name.as_str()?.to_owned() })).collect()
    }
    /// bukkit: org/bukkit/Tag.getValues
    pub fn get_values(&self) -> Vec<String> {
        tag_members(self.kind, &self.name).0.clone()
    }
    /// bukkit: org/bukkit/Tag.isTagged
    pub fn is_tagged(&self, id: &str) -> bool {
        tag_members(self.kind, &self.name).1.contains(&key_of(id))
    }
    /// bukkit: org/bukkit/Keyed.getKey
    pub fn get_key(&self) -> Option<NamespacedKey> {
        NamespacedKey::from_string(&key_of(&self.name))
    }
}

impl Material {
    /// Whether a tag holds this material.
    pub fn is_tagged(&self, tag: &Tag) -> bool {
        tag.is_tagged(self.key())
    }
}

// ---- Enchantments ----

/// `org.bukkit.enchantments.Enchantment`, by its key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enchantment {
    key: String,
}

impl Enchantment {
    /// bukkit: org/bukkit/enchantments/Enchantment.getByKey
    pub fn get_by_key(key: &str) -> Option<Self> {
        Registry::ENCHANTMENT.get(key).map(|key| Self { key })
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getByName
    ///
    /// By the old names too: `DAMAGE_ALL` is `SHARPNESS`.
    pub fn get_by_name(name: &str) -> Option<Self> {
        let modern = match name {
            "PROTECTION_ENVIRONMENTAL" => "protection",
            "PROTECTION_FIRE" => "fire_protection",
            "PROTECTION_FALL" => "feather_falling",
            "PROTECTION_EXPLOSIONS" => "blast_protection",
            "PROTECTION_PROJECTILE" => "projectile_protection",
            "OXYGEN" => "respiration",
            "WATER_WORKER" => "aqua_affinity",
            "DAMAGE_ALL" => "sharpness",
            "DAMAGE_UNDEAD" => "smite",
            "DAMAGE_ARTHROPODS" => "bane_of_arthropods",
            "LOOT_BONUS_MOBS" => "looting",
            "SWEEPING_EDGE" => "sweeping_edge",
            "DIG_SPEED" => "efficiency",
            "DURABILITY" => "unbreaking",
            "LOOT_BONUS_BLOCKS" => "fortune",
            "ARROW_DAMAGE" => "power",
            "ARROW_KNOCKBACK" => "punch",
            "ARROW_FIRE" => "flame",
            "ARROW_INFINITE" => "infinity",
            "LUCK" => "luck_of_the_sea",
            other => return Self::get_by_key(&other.to_ascii_lowercase()),
        };
        Self::get_by_key(modern)
    }
    fn entry(&self) -> Value {
        data_file(&format!("enchantment/{}", self.key.trim_start_matches("minecraft:")))
    }
    /// bukkit: org/bukkit/Keyed.getKey, org/bukkit/enchantments/Enchantment.getKey
    pub fn get_key(&self) -> &str {
        &self.key
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getName
    pub fn get_name(&self) -> String {
        self.key.trim_start_matches("minecraft:").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getMaxLevel
    pub fn get_max_level(&self) -> i32 {
        self.entry()["max_level"].as_i64().unwrap_or(1) as i32
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getStartLevel
    pub fn get_start_level(&self) -> i32 {
        1
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getWeight
    pub fn get_weight(&self) -> i32 {
        self.entry()["weight"].as_i64().unwrap_or(1) as i32
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getAnvilCost
    pub fn get_anvil_cost(&self) -> i32 {
        self.entry()["anvil_cost"].as_i64().unwrap_or(1) as i32
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getMinModifiedCost
    pub fn get_min_modified_cost(&self, level: i32) -> i32 {
        let cost = &self.entry()["min_cost"];
        (cost["base"].as_i64().unwrap_or(0) + cost["per_level_above_first"].as_i64().unwrap_or(0) * i64::from(level - 1)) as i32
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getMaxModifiedCost
    pub fn get_max_modified_cost(&self, level: i32) -> i32 {
        let cost = &self.entry()["max_cost"];
        (cost["base"].as_i64().unwrap_or(0) + cost["per_level_above_first"].as_i64().unwrap_or(0) * i64::from(level - 1)) as i32
    }
    fn in_tag(&self, tag: &str) -> bool {
        Tag::get("enchantment", tag).is_tagged(&self.key)
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.isTreasure
    pub fn is_treasure(&self) -> bool {
        self.in_tag("treasure")
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.isCursed
    pub fn is_cursed(&self) -> bool {
        self.in_tag("curse")
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.isTradeable
    pub fn is_tradeable(&self) -> bool {
        self.in_tag("tradeable")
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.isDiscoverable
    pub fn is_discoverable(&self) -> bool {
        self.in_tag("on_random_loot") || self.in_tag("in_enchanting_table")
    }
    fn items(&self, key: &str) -> Vec<String> {
        let entry = self.entry();
        match entry[key].as_str() {
            Some(text) if text.starts_with('#') => Tag::get("item", text).get_values(),
            Some(text) => vec![text.to_owned()],
            None => Vec::new(),
        }
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getSupportedItems
    pub fn get_supported_items(&self) -> Vec<String> {
        self.items("supported_items")
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getPrimaryItems
    pub fn get_primary_items(&self) -> Vec<String> {
        self.items("primary_items")
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.canEnchantItem
    pub fn can_enchant_item(&self, stack: &ItemStack) -> bool {
        self.get_supported_items().iter().any(|item| item == stack.get_type().key())
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.getExclusiveWith
    pub fn get_exclusive_with(&self) -> Vec<String> {
        let entry = self.entry();
        match &entry["exclusive_set"] {
            Value::String(text) if text.starts_with('#') => Tag::get("enchantment", &text.replace("exclusive_set/", "exclusive_set/")).get_values(),
            Value::String(text) => vec![text.clone()],
            Value::Array(keys) => keys.iter().filter_map(|key| key.as_str().map(str::to_owned)).collect(),
            _ => Vec::new(),
        }
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.conflictsWith
    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.key == other.key || self.get_exclusive_with().contains(&other.key)
    }
    /// bukkit: org/bukkit/enchantments/Enchantment.translationKey
    pub fn translation_key(&self) -> String {
        format!("enchantment.minecraft.{}", self.key.trim_start_matches("minecraft:"))
    }
}

// ---- Potion effects ----

/// bukkit: org/bukkit/potion/PotionEffectType.getByName
///
/// The key of an effect by Bukkit's name, the old ones too.
pub fn potion_effect_type_by_name(name: &str) -> Option<String> {
    let modern = match name.to_ascii_uppercase().as_str() {
        "SLOW" => "slowness",
        "FAST_DIGGING" => "haste",
        "SLOW_DIGGING" => "mining_fatigue",
        "INCREASE_DAMAGE" => "strength",
        "HEAL" => "instant_health",
        "HARM" => "instant_damage",
        "JUMP" => "jump_boost",
        "CONFUSION" => "nausea",
        "DAMAGE_RESISTANCE" => "resistance",
        other => return potion_effect_key(&other.to_ascii_lowercase()),
    };
    potion_effect_key(modern)
}

fn potion_effect_key(name: &str) -> Option<String> {
    crate::enums::potion_effect_type::values().iter().find(|known| known.eq_ignore_ascii_case(name)).map(|_| format!("minecraft:{name}"))
}

/// bukkit: org/bukkit/potion/PotionEffectType.getByKey
pub fn potion_effect_type_by_key(key: &str) -> Option<String> {
    potion_effect_key(key.trim_start_matches("minecraft:"))
}

/// bukkit: org/bukkit/potion/PotionEffectType.createEffect
pub fn create_effect(effect: &str, duration: i32, amplifier: i32) -> crate::PotionEffect {
    crate::PotionEffect::new(effect, duration, amplifier, true, true)
}

impl crate::PotionEffect {
    /// bukkit: org/bukkit/potion/PotionEffect.withDuration
    pub fn with_duration(&self, duration: i32) -> Self {
        Self::new(self.get_type(), duration, self.get_amplifier(), self.is_ambient(), self.has_particles()).with_icon(self.icon)
    }
    /// bukkit: org/bukkit/potion/PotionEffect.withAmplifier
    pub fn with_amplifier(&self, amplifier: i32) -> Self {
        Self::new(self.get_type(), self.get_duration(), amplifier, self.is_ambient(), self.has_particles()).with_icon(self.icon)
    }
    /// bukkit: org/bukkit/potion/PotionEffect.withAmbient
    pub fn with_ambient(&self, ambient: bool) -> Self {
        Self::new(self.get_type(), self.get_duration(), self.get_amplifier(), ambient, self.has_particles()).with_icon(self.icon)
    }
    /// bukkit: org/bukkit/potion/PotionEffect.withParticles
    pub fn with_particles(&self, particles: bool) -> Self {
        Self::new(self.get_type(), self.get_duration(), self.get_amplifier(), self.is_ambient(), particles).with_icon(self.icon)
    }
    /// bukkit: org/bukkit/potion/PotionEffect.withType
    pub fn with_type(&self, effect: &str) -> Self {
        Self::new(effect, self.get_duration(), self.get_amplifier(), self.is_ambient(), self.has_particles()).with_icon(self.icon)
    }
    /// bukkit: org/bukkit/potion/PotionEffect.isInfinite
    pub fn is_infinite(&self) -> bool {
        self.get_duration() == -1
    }
    /// bukkit: org/bukkit/potion/PotionEffect.isShorterThan
    pub fn is_shorter_than(&self, other: &Self) -> bool {
        !self.is_infinite() && (other.is_infinite() || self.get_duration() < other.get_duration())
    }
}

// ---- Entity types ----

/// bukkit: org/bukkit/entity/EntityType.getKey
pub fn entity_type_key(name: &str) -> NamespacedKey {
    NamespacedKey::minecraft(&name.trim_start_matches("minecraft:").to_ascii_lowercase())
}

/// bukkit: org/bukkit/entity/EntityType.getName
pub fn entity_type_name(key: &str) -> String {
    key.trim_start_matches("minecraft:").to_ascii_lowercase()
}

/// bukkit: org/bukkit/entity/EntityType.fromName
pub fn entity_type_from_name(name: &str) -> Option<&'static str> {
    crate::enums::entity_type::value_of(&name.to_ascii_uppercase())
}

/// bukkit: org/bukkit/entity/EntityType.translationKey, org/bukkit/entity/EntityType.getTranslationKey
pub fn entity_type_translation_key(key: &str) -> String {
    format!("entity.minecraft.{}", entity_type_name(key))
}

impl Entity {
    /// bukkit: org/bukkit/entity/Entity.getName
    pub fn get_name(&self) -> String {
        self.get_custom_name().unwrap_or_else(|| entity_type_name(&self.get_type()))
    }
}

// ---- Colors ----

impl Color {
    /// bukkit: org/bukkit/Color.fromARGB
    pub fn from_argb(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue, alpha }
    }
    /// bukkit: org/bukkit/Color.fromBGR
    pub fn from_bgr(blue: u8, green: u8, red: u8) -> Self {
        Self { red, green, blue, alpha: 255 }
    }
    /// bukkit: org/bukkit/Color.getRed, org/bukkit/Color.red
    pub fn get_red(self) -> u8 {
        self.red
    }
    /// bukkit: org/bukkit/Color.getGreen, org/bukkit/Color.green
    pub fn get_green(self) -> u8 {
        self.green
    }
    /// bukkit: org/bukkit/Color.getBlue, org/bukkit/Color.blue
    pub fn get_blue(self) -> u8 {
        self.blue
    }
    /// bukkit: org/bukkit/Color.getAlpha, org/bukkit/Color.alpha
    pub fn get_alpha(self) -> u8 {
        self.alpha
    }
    /// bukkit: org/bukkit/Color.setRed
    pub fn set_red(self, red: u8) -> Self {
        Self { red, ..self }
    }
    /// bukkit: org/bukkit/Color.setGreen
    pub fn set_green(self, green: u8) -> Self {
        Self { green, ..self }
    }
    /// bukkit: org/bukkit/Color.setBlue
    pub fn set_blue(self, blue: u8) -> Self {
        Self { blue, ..self }
    }
    /// bukkit: org/bukkit/Color.asARGB
    pub fn as_argb(self) -> u32 {
        (u32::from(self.alpha) << 24) | self.as_rgb()
    }
    /// bukkit: org/bukkit/Color.asBGR
    pub fn as_bgr(self) -> u32 {
        (u32::from(self.blue) << 16) | (u32::from(self.green) << 8) | u32::from(self.red)
    }
}

// ---- Fireworks ----

/// `org.bukkit.FireworkEffect$Builder`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FireworkEffectBuilder {
    shape: String,
    colors: Vec<Color>,
    fades: Vec<Color>,
    flicker: bool,
    trail: bool,
}

impl FireworkEffectBuilder {
    /// bukkit: org/bukkit/FireworkEffect$Builder.with
    pub fn with(mut self, shape: &str) -> Self {
        shape.clone_into(&mut self.shape);
        self
    }
    /// bukkit: org/bukkit/FireworkEffect$Builder.withColor
    pub fn with_color(mut self, color: Color) -> Self {
        self.colors.push(color);
        self
    }
    /// bukkit: org/bukkit/FireworkEffect$Builder.withFade
    pub fn with_fade(mut self, color: Color) -> Self {
        self.fades.push(color);
        self
    }
    /// bukkit: org/bukkit/FireworkEffect$Builder.withFlicker, org/bukkit/FireworkEffect$Builder.flicker
    pub fn flicker(mut self, on: bool) -> Self {
        self.flicker = on;
        self
    }
    /// bukkit: org/bukkit/FireworkEffect$Builder.withTrail, org/bukkit/FireworkEffect$Builder.trail
    pub fn trail(mut self, on: bool) -> Self {
        self.trail = on;
        self
    }
    /// bukkit: org/bukkit/FireworkEffect$Builder.build
    pub fn build(self) -> FireworkEffect {
        let shape = if self.shape.is_empty() { "BALL" } else { &self.shape };
        let shape = match shape {
            "BALL" => "small_ball",
            "BALL_LARGE" => "large_ball",
            "STAR" => "star",
            "BURST" => "burst",
            "CREEPER" => "creeper",
            other => other,
        };
        FireworkEffect::new(shape, &self.colors, &self.fades, self.flicker, self.trail)
    }
}

/// bukkit: org/bukkit/FireworkEffect.builder
pub fn firework_effect_builder() -> FireworkEffectBuilder {
    FireworkEffectBuilder::default()
}

// ---- What events were missing a Java name for ----

impl PlayerDeathEvent {
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getEntity, org/bukkit/event/entity/PlayerDeathEvent.getPlayer
    pub fn get_entity(&self) -> &Player {
        &self.player
    }
}

impl EntityDamageByEntityEvent {
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.getEntity, org/bukkit/event/entity/EntityEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.get_entity_id(), &self.get_damager().get_world())
    }
}

impl EntityDeathEvent {
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getEntity
    pub fn get_entity(&self, world: &crate::World) -> Entity {
        Entity::of(self.get_entity_id(), world)
    }
}
