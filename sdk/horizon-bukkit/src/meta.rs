//! Written by plugin-tools/tools/sdk_meta.py: do not edit.
#![allow(missing_docs)]
use crate::ItemStack;
use horizon_mod::Value;
use serde_json::json;

/// A class of `ItemMeta`: a view of a copy of a stack.
pub trait MetaClass {
    fn of(stack: ItemStack) -> Self;
    fn stack(&self) -> &ItemStack;
}

impl ItemStack {
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getEnchantable
    pub fn get_enchantable(&self) -> Option<i32> {
        self.component_at("minecraft:enchantable", "value").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setEnchantable
    pub fn set_enchantable(&mut self, value: i32) {
        self.set_component_at("minecraft:enchantable", "value", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasEnchantable
    pub fn has_enchantable(&self) -> bool {
        self.component_at("minecraft:enchantable", "value").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getTooltipStyle
    pub fn get_tooltip_style(&self) -> Option<String> {
        self.component_at("minecraft:tooltip_style", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setTooltipStyle
    pub fn set_tooltip_style(&mut self, value: &str) {
        self.set_component_at("minecraft:tooltip_style", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasTooltipStyle
    pub fn has_tooltip_style(&self) -> bool {
        self.component_at("minecraft:tooltip_style", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getItemModel
    pub fn get_item_model(&self) -> Option<String> {
        self.component_at("minecraft:item_model", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setItemModel
    pub fn set_item_model(&mut self, value: &str) {
        self.set_component_at("minecraft:item_model", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasItemModel
    pub fn has_item_model(&self) -> bool {
        self.component_at("minecraft:item_model", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getEnchantmentGlintOverride
    pub fn get_enchantment_glint_override(&self) -> Option<bool> {
        self.component_at("minecraft:enchantment_glint_override", "").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setEnchantmentGlintOverride
    pub fn set_enchantment_glint_override(&mut self, value: bool) {
        self.set_component_at("minecraft:enchantment_glint_override", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasEnchantmentGlintOverride
    pub fn has_enchantment_glint_override(&self) -> bool {
        self.component_at("minecraft:enchantment_glint_override", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getMaxStackSizeOverride
    pub fn get_max_stack_size_override(&self) -> Option<i32> {
        self.component_at("minecraft:max_stack_size", "").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setMaxStackSize
    pub fn set_max_stack_size(&mut self, value: i32) {
        self.set_component_at("minecraft:max_stack_size", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasMaxStackSize
    pub fn has_max_stack_size(&self) -> bool {
        self.component_at("minecraft:max_stack_size", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getRarity, org/bukkit/inventory/ItemStack.getRarity
    ///
    /// `None` where Java throws: the stack has no rarity of its own.
    pub fn get_rarity(&self) -> Option<String> {
        self.component_at("minecraft:rarity", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setRarity
    pub fn set_rarity(&mut self, value: &str) {
        self.set_component_at("minecraft:rarity", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasRarity
    pub fn has_rarity(&self) -> bool {
        self.component_at("minecraft:rarity", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.isGlider
    pub fn is_glider(&self) -> bool {
        self.get_component("minecraft:glider").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setGlider
    pub fn set_glider(&mut self, value: bool) {
        if value {
            self.set_component("minecraft:glider", json!({}));
        } else {
            self.remove_component("minecraft:glider");
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.isFireResistant
    pub fn is_fire_resistant(&self) -> bool {
        self.get_component("minecraft:damage_resistant").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setFireResistant
    pub fn set_fire_resistant(&mut self, value: bool) {
        if value {
            self.set_component("minecraft:damage_resistant", json!({ "types": "#minecraft:is_fire" }));
        } else {
            self.remove_component("minecraft:damage_resistant");
        }
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.itemName
    pub fn item_name(&self) -> Option<crate::Component> {
        self.component_at("minecraft:item_name", "").cloned().map(crate::Component)
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setItemNameComponent
    pub fn set_item_name_component(&mut self, value: &crate::Component) {
        self.set_component_at("minecraft:item_name", "", value.0.clone());
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasItemName
    pub fn has_item_name(&self) -> bool {
        self.component_at("minecraft:item_name", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/Damageable.getMaxDamage
    pub fn get_max_damage(&self) -> Option<i32> {
        self.component_at("minecraft:max_damage", "").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/Damageable.setMaxDamage
    pub fn set_max_damage(&mut self, value: i32) {
        self.set_component_at("minecraft:max_damage", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/Damageable.hasMaxDamage
    pub fn has_max_damage(&self) -> bool {
        self.component_at("minecraft:max_damage", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/Repairable.getRepairCost
    pub fn get_repair_cost(&self) -> Option<i32> {
        self.component_at("minecraft:repair_cost", "").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/Repairable.setRepairCost
    pub fn set_repair_cost(&mut self, value: i32) {
        self.set_component_at("minecraft:repair_cost", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/Repairable.hasRepairCost
    pub fn has_repair_cost(&self) -> bool {
        self.component_at("minecraft:repair_cost", "").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/LeatherArmorMeta
#[derive(Clone, Debug, PartialEq)]
pub struct LeatherArmorMeta(pub ItemStack);

impl MetaClass for LeatherArmorMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl LeatherArmorMeta {
    /// bukkit: org/bukkit/inventory/meta/LeatherArmorMeta.getColor
    pub fn get_color(&self) -> Option<crate::Color> {
        self.0.component_at("minecraft:dyed_color", "").and_then(Value::as_i64).map(|rgb| crate::Color::from_rgb(((rgb >> 16) & 255) as u8, ((rgb >> 8) & 255) as u8, (rgb & 255) as u8))
    }
    /// bukkit: org/bukkit/inventory/meta/LeatherArmorMeta.setColor
    pub fn set_color(&mut self, value: &crate::Color) {
        self.0.set_component_at("minecraft:dyed_color", "", json!(value.as_rgb()));
    }
    /// bukkit: org/bukkit/inventory/meta/LeatherArmorMeta.isDyed
    pub fn is_dyed(&self) -> bool {
        self.0.component_at("minecraft:dyed_color", "").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/ColorableArmorMeta
#[derive(Clone, Debug, PartialEq)]
pub struct ColorableArmorMeta(pub ItemStack);

impl MetaClass for ColorableArmorMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl ColorableArmorMeta {
    /// bukkit: org/bukkit/inventory/meta/ColorableArmorMeta.getColor
    pub fn get_color(&self) -> Option<crate::Color> {
        self.0.component_at("minecraft:dyed_color", "").and_then(Value::as_i64).map(|rgb| crate::Color::from_rgb(((rgb >> 16) & 255) as u8, ((rgb >> 8) & 255) as u8, (rgb & 255) as u8))
    }
    /// bukkit: org/bukkit/inventory/meta/ColorableArmorMeta.setColor
    pub fn set_color(&mut self, value: &crate::Color) {
        self.0.set_component_at("minecraft:dyed_color", "", json!(value.as_rgb()));
    }
    /// bukkit: org/bukkit/inventory/meta/ColorableArmorMeta.isDyed
    pub fn is_dyed(&self) -> bool {
        self.0.component_at("minecraft:dyed_color", "").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/MapMeta
#[derive(Clone, Debug, PartialEq)]
pub struct MapMeta(pub ItemStack);

impl MetaClass for MapMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl MapMeta {
    /// bukkit: org/bukkit/inventory/meta/MapMeta.getMapId
    pub fn get_map_id(&self) -> Option<i32> {
        self.0.component_at("minecraft:map_id", "").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.setMapId
    pub fn set_map_id(&mut self, value: i32) {
        self.0.set_component_at("minecraft:map_id", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.hasMapId
    pub fn has_map_id(&self) -> bool {
        self.0.component_at("minecraft:map_id", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.getColor
    pub fn get_color(&self) -> Option<crate::Color> {
        self.0.component_at("minecraft:map_color", "").and_then(Value::as_i64).map(|rgb| crate::Color::from_rgb(((rgb >> 16) & 255) as u8, ((rgb >> 8) & 255) as u8, (rgb & 255) as u8))
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.setColor
    pub fn set_color(&mut self, value: &crate::Color) {
        self.0.set_component_at("minecraft:map_color", "", json!(value.as_rgb()));
    }
    /// bukkit: org/bukkit/inventory/meta/MapMeta.hasColor
    pub fn has_color(&self) -> bool {
        self.0.component_at("minecraft:map_color", "").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/PotionMeta
#[derive(Clone, Debug, PartialEq)]
pub struct PotionMeta(pub ItemStack);

impl MetaClass for PotionMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl PotionMeta {
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.getColor
    pub fn get_color(&self) -> Option<crate::Color> {
        self.0.component_at("minecraft:potion_contents", "custom_color").and_then(Value::as_i64).map(|rgb| crate::Color::from_rgb(((rgb >> 16) & 255) as u8, ((rgb >> 8) & 255) as u8, (rgb & 255) as u8))
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.setColor
    pub fn set_color(&mut self, value: &crate::Color) {
        self.0.set_component_at("minecraft:potion_contents", "custom_color", json!(value.as_rgb()));
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.hasColor
    pub fn has_color(&self) -> bool {
        self.0.component_at("minecraft:potion_contents", "custom_color").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.getBasePotionType
    pub fn get_base_potion_type(&self) -> Option<String> {
        self.0.component_at("minecraft:potion_contents", "potion").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.setBasePotionType
    pub fn set_base_potion_type(&mut self, value: &str) {
        self.0.set_component_at("minecraft:potion_contents", "potion", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.hasBasePotionType
    pub fn has_base_potion_type(&self) -> bool {
        self.0.component_at("minecraft:potion_contents", "potion").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.getCustomPotionName
    pub fn get_custom_potion_name(&self) -> Option<String> {
        self.0.component_at("minecraft:potion_contents", "custom_name").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.setCustomPotionName
    pub fn set_custom_potion_name(&mut self, value: &str) {
        self.0.set_component_at("minecraft:potion_contents", "custom_name", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.hasCustomPotionName
    pub fn has_custom_potion_name(&self) -> bool {
        self.0.component_at("minecraft:potion_contents", "custom_name").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.getCustomEffectsJson
    pub fn get_custom_effects_json(&self) -> Option<&Value> {
        self.0.component_at("minecraft:potion_contents", "custom_effects")
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.hasCustomEffects
    pub fn has_custom_effects(&self) -> bool {
        self.0.component_at("minecraft:potion_contents", "custom_effects").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/BookMeta
#[derive(Clone, Debug, PartialEq)]
pub struct BookMeta(pub ItemStack);

impl MetaClass for BookMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl BookMeta {
    /// bukkit: org/bukkit/inventory/meta/BookMeta.getTitle
    pub fn get_title(&self) -> Option<String> {
        self.0.component_at("minecraft:written_book_content", "title.raw").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.setTitle
    pub fn set_title(&mut self, value: &str) {
        self.0.set_component_at("minecraft:written_book_content", "title.raw", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.hasTitle
    pub fn has_title(&self) -> bool {
        self.0.component_at("minecraft:written_book_content", "title.raw").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.getAuthor
    pub fn get_author(&self) -> Option<String> {
        self.0.component_at("minecraft:written_book_content", "author").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.setAuthor
    pub fn set_author(&mut self, value: &str) {
        self.0.set_component_at("minecraft:written_book_content", "author", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.hasAuthor
    pub fn has_author(&self) -> bool {
        self.0.component_at("minecraft:written_book_content", "author").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.getGenerationNumber
    pub fn get_generation_number(&self) -> Option<i32> {
        self.0.component_at("minecraft:written_book_content", "generation").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.setGenerationNumber
    pub fn set_generation_number(&mut self, value: i32) {
        self.0.set_component_at("minecraft:written_book_content", "generation", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.hasGeneration
    pub fn has_generation(&self) -> bool {
        self.0.component_at("minecraft:written_book_content", "generation").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/SkullMeta
#[derive(Clone, Debug, PartialEq)]
pub struct SkullMeta(pub ItemStack);

impl MetaClass for SkullMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl SkullMeta {
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.getOwner
    pub fn get_owner(&self) -> Option<String> {
        self.0.component_at("minecraft:profile", "name").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.setOwner
    pub fn set_owner(&mut self, value: &str) {
        self.0.set_component_at("minecraft:profile", "name", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.hasOwner
    pub fn has_owner(&self) -> bool {
        self.0.component_at("minecraft:profile", "name").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.getNoteBlockSound
    pub fn get_note_block_sound(&self) -> Option<String> {
        self.0.component_at("minecraft:note_block_sound", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/SkullMeta.setNoteBlockSound
    pub fn set_note_block_sound(&mut self, value: &str) {
        self.0.set_component_at("minecraft:note_block_sound", "", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/CompassMeta
#[derive(Clone, Debug, PartialEq)]
pub struct CompassMeta(pub ItemStack);

impl MetaClass for CompassMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl CompassMeta {
    /// bukkit: org/bukkit/inventory/meta/CompassMeta.isLodestoneTracked
    pub fn is_lodestone_tracked(&self) -> Option<bool> {
        self.0.component_at("minecraft:lodestone_tracker", "tracked").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/CompassMeta.setLodestoneTracked
    pub fn set_lodestone_tracked(&mut self, value: bool) {
        self.0.set_component_at("minecraft:lodestone_tracker", "tracked", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/FireworkMeta
#[derive(Clone, Debug, PartialEq)]
pub struct FireworkMeta(pub ItemStack);

impl MetaClass for FireworkMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl FireworkMeta {
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.getPower
    pub fn get_power(&self) -> Option<i32> {
        self.0.component_at("minecraft:fireworks", "flight_duration").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.setPower
    pub fn set_power(&mut self, value: i32) {
        self.0.set_component_at("minecraft:fireworks", "flight_duration", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.hasPower
    pub fn has_power(&self) -> bool {
        self.0.component_at("minecraft:fireworks", "flight_duration").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/ShieldMeta
#[derive(Clone, Debug, PartialEq)]
pub struct ShieldMeta(pub ItemStack);

impl MetaClass for ShieldMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl ShieldMeta {
    /// bukkit: org/bukkit/inventory/meta/ShieldMeta.getBaseColorName
    pub fn get_base_color_name(&self) -> Option<String> {
        self.0.component_at("minecraft:base_color", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/ShieldMeta.setBaseColorName
    pub fn set_base_color_name(&mut self, value: &str) {
        self.0.set_component_at("minecraft:base_color", "", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/ArmorMeta
#[derive(Clone, Debug, PartialEq)]
pub struct ArmorMeta(pub ItemStack);

impl MetaClass for ArmorMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl ArmorMeta {
    /// bukkit: org/bukkit/inventory/meta/ArmorMeta.getTrimMaterial
    pub fn get_trim_material(&self) -> Option<String> {
        self.0.component_at("minecraft:trim", "material").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/ArmorMeta.setTrimMaterial
    pub fn set_trim_material(&mut self, value: &str) {
        self.0.set_component_at("minecraft:trim", "material", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/ArmorMeta.hasTrim
    pub fn has_trim(&self) -> bool {
        self.0.component_at("minecraft:trim", "material").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ArmorMeta.getTrimPattern
    pub fn get_trim_pattern(&self) -> Option<String> {
        self.0.component_at("minecraft:trim", "pattern").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/ArmorMeta.setTrimPattern
    pub fn set_trim_pattern(&mut self, value: &str) {
        self.0.set_component_at("minecraft:trim", "pattern", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/SpawnEggMeta
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnEggMeta(pub ItemStack);

impl MetaClass for SpawnEggMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl SpawnEggMeta {
    /// bukkit: org/bukkit/inventory/meta/SpawnEggMeta.getCustomSpawnedType
    pub fn get_custom_spawned_type(&self) -> Option<String> {
        self.0.component_at("minecraft:entity_data", "id").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/SpawnEggMeta.setCustomSpawnedType
    pub fn set_custom_spawned_type(&mut self, value: &str) {
        self.0.set_component_at("minecraft:entity_data", "id", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/OminousBottleMeta
#[derive(Clone, Debug, PartialEq)]
pub struct OminousBottleMeta(pub ItemStack);

impl MetaClass for OminousBottleMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl OminousBottleMeta {
    /// bukkit: org/bukkit/inventory/meta/OminousBottleMeta.getAmplifier
    pub fn get_amplifier(&self) -> Option<i32> {
        self.0.component_at("minecraft:ominous_bottle_amplifier", "").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/OminousBottleMeta.setAmplifier
    pub fn set_amplifier(&mut self, value: i32) {
        self.0.set_component_at("minecraft:ominous_bottle_amplifier", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/OminousBottleMeta.hasAmplifier
    pub fn has_amplifier(&self) -> bool {
        self.0.component_at("minecraft:ominous_bottle_amplifier", "").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/MusicInstrumentMeta
#[derive(Clone, Debug, PartialEq)]
pub struct MusicInstrumentMeta(pub ItemStack);

impl MetaClass for MusicInstrumentMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl MusicInstrumentMeta {
    /// bukkit: org/bukkit/inventory/meta/MusicInstrumentMeta.getInstrument
    pub fn get_instrument(&self) -> Option<String> {
        self.0.component_at("minecraft:instrument", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/MusicInstrumentMeta.setInstrument
    pub fn set_instrument(&mut self, value: &str) {
        self.0.set_component_at("minecraft:instrument", "", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/AxolotlBucketMeta
#[derive(Clone, Debug, PartialEq)]
pub struct AxolotlBucketMeta(pub ItemStack);

impl MetaClass for AxolotlBucketMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl AxolotlBucketMeta {
    /// bukkit: org/bukkit/inventory/meta/AxolotlBucketMeta.getVariant
    pub fn get_variant(&self) -> Option<String> {
        self.0.component_at("minecraft:axolotl/variant", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/AxolotlBucketMeta.setVariant
    pub fn set_variant(&mut self, value: &str) {
        self.0.set_component_at("minecraft:axolotl/variant", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/AxolotlBucketMeta.hasVariant
    pub fn has_variant(&self) -> bool {
        self.0.component_at("minecraft:axolotl/variant", "").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta
#[derive(Clone, Debug, PartialEq)]
pub struct TropicalFishBucketMeta(pub ItemStack);

impl MetaClass for TropicalFishBucketMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl TropicalFishBucketMeta {
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.getPattern
    pub fn get_pattern(&self) -> Option<String> {
        self.0.component_at("minecraft:tropical_fish/pattern", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.setPattern
    pub fn set_pattern(&mut self, value: &str) {
        self.0.set_component_at("minecraft:tropical_fish/pattern", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.hasPattern
    pub fn has_pattern(&self) -> bool {
        self.0.component_at("minecraft:tropical_fish/pattern", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.getBodyColor
    pub fn get_body_color(&self) -> Option<String> {
        self.0.component_at("minecraft:tropical_fish/base_color", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.setBodyColor
    pub fn set_body_color(&mut self, value: &str) {
        self.0.set_component_at("minecraft:tropical_fish/base_color", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.hasBodyColor
    pub fn has_body_color(&self) -> bool {
        self.0.component_at("minecraft:tropical_fish/base_color", "").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.getPatternColor
    pub fn get_pattern_color(&self) -> Option<String> {
        self.0.component_at("minecraft:tropical_fish/pattern_color", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.setPatternColor
    pub fn set_pattern_color(&mut self, value: &str) {
        self.0.set_component_at("minecraft:tropical_fish/pattern_color", "", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/TropicalFishBucketMeta.hasPatternColor
    pub fn has_pattern_color(&self) -> bool {
        self.0.component_at("minecraft:tropical_fish/pattern_color", "").is_some()
    }
}

/// bukkit: org/bukkit/inventory/meta/components/FoodComponent
#[derive(Clone, Debug, PartialEq)]
pub struct FoodComponent(pub ItemStack);

impl MetaClass for FoodComponent {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl FoodComponent {
    /// bukkit: org/bukkit/inventory/meta/components/FoodComponent.getNutrition
    pub fn get_nutrition(&self) -> Option<i32> {
        self.0.component_at("minecraft:food", "nutrition").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/components/FoodComponent.setNutrition
    pub fn set_nutrition(&mut self, value: i32) {
        self.0.set_component_at("minecraft:food", "nutrition", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/FoodComponent.getSaturation
    pub fn get_saturation(&self) -> Option<f32> {
        self.0.component_at("minecraft:food", "saturation").and_then(Value::as_f64).map(|value| value as f32)
    }
    /// bukkit: org/bukkit/inventory/meta/components/FoodComponent.setSaturation
    pub fn set_saturation(&mut self, value: f32) {
        self.0.set_component_at("minecraft:food", "saturation", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/FoodComponent.canAlwaysEat
    pub fn can_always_eat(&self) -> Option<bool> {
        self.0.component_at("minecraft:food", "can_always_eat").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/components/FoodComponent.setCanAlwaysEat
    pub fn set_can_always_eat(&mut self, value: bool) {
        self.0.set_component_at("minecraft:food", "can_always_eat", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/components/ToolComponent
#[derive(Clone, Debug, PartialEq)]
pub struct ToolComponent(pub ItemStack);

impl MetaClass for ToolComponent {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl ToolComponent {
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent.getDefaultMiningSpeed
    pub fn get_default_mining_speed(&self) -> Option<f32> {
        self.0.component_at("minecraft:tool", "default_mining_speed").and_then(Value::as_f64).map(|value| value as f32)
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent.setDefaultMiningSpeed
    pub fn set_default_mining_speed(&mut self, value: f32) {
        self.0.set_component_at("minecraft:tool", "default_mining_speed", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent.getDamagePerBlock
    pub fn get_damage_per_block(&self) -> Option<i32> {
        self.0.component_at("minecraft:tool", "damage_per_block").and_then(Value::as_i64).map(|value| value as i32)
    }
    /// bukkit: org/bukkit/inventory/meta/components/ToolComponent.setDamagePerBlock
    pub fn set_damage_per_block(&mut self, value: i32) {
        self.0.set_component_at("minecraft:tool", "damage_per_block", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/components/UseCooldownComponent
#[derive(Clone, Debug, PartialEq)]
pub struct UseCooldownComponent(pub ItemStack);

impl MetaClass for UseCooldownComponent {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl UseCooldownComponent {
    /// bukkit: org/bukkit/inventory/meta/components/UseCooldownComponent.getCooldownSeconds
    pub fn get_cooldown_seconds(&self) -> Option<f32> {
        self.0.component_at("minecraft:use_cooldown", "seconds").and_then(Value::as_f64).map(|value| value as f32)
    }
    /// bukkit: org/bukkit/inventory/meta/components/UseCooldownComponent.setCooldownSeconds
    pub fn set_cooldown_seconds(&mut self, value: f32) {
        self.0.set_component_at("minecraft:use_cooldown", "seconds", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/UseCooldownComponent.getCooldownGroup
    pub fn get_cooldown_group(&self) -> Option<String> {
        self.0.component_at("minecraft:use_cooldown", "cooldown_group").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/components/UseCooldownComponent.setCooldownGroup
    pub fn set_cooldown_group(&mut self, value: &str) {
        self.0.set_component_at("minecraft:use_cooldown", "cooldown_group", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/components/JukeboxPlayableComponent
#[derive(Clone, Debug, PartialEq)]
pub struct JukeboxPlayableComponent(pub ItemStack);

impl MetaClass for JukeboxPlayableComponent {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl JukeboxPlayableComponent {
    /// bukkit: org/bukkit/inventory/meta/components/JukeboxPlayableComponent.getSongKey
    pub fn get_song_key(&self) -> Option<String> {
        self.0.component_at("minecraft:jukebox_playable", "").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/components/JukeboxPlayableComponent.setSongKey
    pub fn set_song_key(&mut self, value: &str) {
        self.0.set_component_at("minecraft:jukebox_playable", "", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/components/EquippableComponent
#[derive(Clone, Debug, PartialEq)]
pub struct EquippableComponent(pub ItemStack);

impl MetaClass for EquippableComponent {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

impl EquippableComponent {
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.getSlot
    pub fn get_slot(&self) -> Option<String> {
        self.0.component_at("minecraft:equippable", "slot").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setSlot
    pub fn set_slot(&mut self, value: &str) {
        self.0.set_component_at("minecraft:equippable", "slot", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.getEquipSound
    pub fn get_equip_sound(&self) -> Option<String> {
        self.0.component_at("minecraft:equippable", "equip_sound").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setEquipSound
    pub fn set_equip_sound(&mut self, value: &str) {
        self.0.set_component_at("minecraft:equippable", "equip_sound", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.getModel
    pub fn get_model(&self) -> Option<String> {
        self.0.component_at("minecraft:equippable", "asset_id").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setModel
    pub fn set_model(&mut self, value: &str) {
        self.0.set_component_at("minecraft:equippable", "asset_id", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.getCameraOverlay
    pub fn get_camera_overlay(&self) -> Option<String> {
        self.0.component_at("minecraft:equippable", "camera_overlay").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setCameraOverlay
    pub fn set_camera_overlay(&mut self, value: &str) {
        self.0.set_component_at("minecraft:equippable", "camera_overlay", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.isDispensable
    pub fn is_dispensable(&self) -> Option<bool> {
        self.0.component_at("minecraft:equippable", "dispensable").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setDispensable
    pub fn set_dispensable(&mut self, value: bool) {
        self.0.set_component_at("minecraft:equippable", "dispensable", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.isSwappable
    pub fn is_swappable(&self) -> Option<bool> {
        self.0.component_at("minecraft:equippable", "swappable").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setSwappable
    pub fn set_swappable(&mut self, value: bool) {
        self.0.set_component_at("minecraft:equippable", "swappable", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.isDamageOnHurt
    pub fn is_damage_on_hurt(&self) -> Option<bool> {
        self.0.component_at("minecraft:equippable", "damage_on_hurt").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setDamageOnHurt
    pub fn set_damage_on_hurt(&mut self, value: bool) {
        self.0.set_component_at("minecraft:equippable", "damage_on_hurt", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.isEquipOnInteract
    pub fn is_equip_on_interact(&self) -> Option<bool> {
        self.0.component_at("minecraft:equippable", "equip_on_interact").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setEquipOnInteract
    pub fn set_equip_on_interact(&mut self, value: bool) {
        self.0.set_component_at("minecraft:equippable", "equip_on_interact", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.canBeSheared
    pub fn can_be_sheared(&self) -> Option<bool> {
        self.0.component_at("minecraft:equippable", "can_be_sheared").map(crate::data::flag)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setCanBeSheared
    pub fn set_can_be_sheared(&mut self, value: bool) {
        self.0.set_component_at("minecraft:equippable", "can_be_sheared", json!(value));
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.getShearingSound
    pub fn get_shearing_sound(&self) -> Option<String> {
        self.0.component_at("minecraft:equippable", "shearing_sound").and_then(Value::as_str).map(str::to_owned)
    }
    /// bukkit: org/bukkit/inventory/meta/components/EquippableComponent.setShearingSound
    pub fn set_shearing_sound(&mut self, value: &str) {
        self.0.set_component_at("minecraft:equippable", "shearing_sound", json!(value));
    }
}

/// bukkit: org/bukkit/inventory/meta/BannerMeta
#[derive(Clone, Debug, PartialEq)]
pub struct BannerMeta(pub ItemStack);

impl MetaClass for BannerMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/BlockDataMeta
#[derive(Clone, Debug, PartialEq)]
pub struct BlockDataMeta(pub ItemStack);

impl MetaClass for BlockDataMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/BlockStateMeta
#[derive(Clone, Debug, PartialEq)]
pub struct BlockStateMeta(pub ItemStack);

impl MetaClass for BlockStateMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/BundleMeta
#[derive(Clone, Debug, PartialEq)]
pub struct BundleMeta(pub ItemStack);

impl MetaClass for BundleMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/CrossbowMeta
#[derive(Clone, Debug, PartialEq)]
pub struct CrossbowMeta(pub ItemStack);

impl MetaClass for CrossbowMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/EnchantmentStorageMeta
#[derive(Clone, Debug, PartialEq)]
pub struct EnchantmentStorageMeta(pub ItemStack);

impl MetaClass for EnchantmentStorageMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/FireworkEffectMeta
#[derive(Clone, Debug, PartialEq)]
pub struct FireworkEffectMeta(pub ItemStack);

impl MetaClass for FireworkEffectMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/KnowledgeBookMeta
#[derive(Clone, Debug, PartialEq)]
pub struct KnowledgeBookMeta(pub ItemStack);

impl MetaClass for KnowledgeBookMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/SuspiciousStewMeta
#[derive(Clone, Debug, PartialEq)]
pub struct SuspiciousStewMeta(pub ItemStack);

impl MetaClass for SuspiciousStewMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/WritableBookMeta
#[derive(Clone, Debug, PartialEq)]
pub struct WritableBookMeta(pub ItemStack);

impl MetaClass for WritableBookMeta {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}

/// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent
#[derive(Clone, Debug, PartialEq)]
pub struct CustomModelDataComponent(pub ItemStack);

impl MetaClass for CustomModelDataComponent {
    fn of(stack: ItemStack) -> Self {
        Self(stack)
    }
    fn stack(&self) -> &ItemStack {
        &self.0
    }
}
