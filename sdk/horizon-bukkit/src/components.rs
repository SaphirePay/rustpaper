//! Written by plugin-tools/tools/sdk_components.py from family/items_components.py: do not edit.
//!
//! Each class is the game's component JSON. Values go in as the game has them: ids as strings, text as
//! components, `head` for a slot. `stack.set_component(id, value.0)` puts one on an item.
#![allow(missing_docs, clippy::wrong_self_convention)]
use horizon_mod::Value;
use serde_json::{Map, json};

/// `io.papermc.paper.datacomponent.item.AttackRange`.
#[derive(Clone, Debug, PartialEq)]
pub struct AttackRange(pub Value);

impl AttackRange {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> AttackRangeBuilder {
        AttackRangeBuilder::default()
    }
}

impl AttackRange {
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange.attackRange
    pub fn attack_range() -> AttackRangeBuilder {
        AttackRangeBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange.minReach
    pub fn min_reach(&self) -> &Value {
        &self.0["min_reach"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange.maxReach
    pub fn max_reach(&self) -> &Value {
        &self.0["max_reach"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange.minCreativeReach
    pub fn min_creative_reach(&self) -> &Value {
        &self.0["min_creative_reach"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange.maxCreativeReach
    pub fn max_creative_reach(&self) -> &Value {
        &self.0["max_creative_reach"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange.hitboxMargin
    pub fn hitbox_margin(&self) -> &Value {
        &self.0["hitbox_margin"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange.mobFactor
    pub fn mob_factor(&self) -> &Value {
        &self.0["mob_factor"]
    }
}

/// `io.papermc.paper.datacomponent.item.AttackRange$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttackRangeBuilder(pub Map<String, Value>);

impl AttackRangeBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange$Builder.minReach
    pub fn min_reach(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("min_reach".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange$Builder.maxReach
    pub fn max_reach(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("max_reach".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange$Builder.minCreativeReach
    pub fn min_creative_reach(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("min_creative_reach".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange$Builder.maxCreativeReach
    pub fn max_creative_reach(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("max_creative_reach".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange$Builder.hitboxMargin
    pub fn hitbox_margin(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("hitbox_margin".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange$Builder.mobFactor
    pub fn mob_factor(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("mob_factor".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/AttackRange$Builder.build
    pub fn build(self) -> AttackRange {
        AttackRange(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.BannerPatternLayers`.
#[derive(Clone, Debug, PartialEq)]
pub struct BannerPatternLayers(pub Value);

impl BannerPatternLayers {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> BannerPatternLayersBuilder {
        BannerPatternLayersBuilder::default()
    }
}

impl BannerPatternLayers {
    /// bukkit: io/papermc/paper/datacomponent/item/BannerPatternLayers.bannerPatternLayers
    pub fn banner_pattern_layers(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BannerPatternLayers.patterns
    pub fn patterns(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.BannerPatternLayers$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BannerPatternLayersBuilder(pub Map<String, Value>);

impl BannerPatternLayersBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BannerPatternLayers$Builder.add
    pub fn add(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "value", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BannerPatternLayers$Builder.addAll
    pub fn add_all(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "value", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BannerPatternLayers$Builder.build
    pub fn build(mut self) -> BannerPatternLayers {
        BannerPatternLayers(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.BundleContents`.
#[derive(Clone, Debug, PartialEq)]
pub struct BundleContents(pub Value);

impl BundleContents {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> BundleContentsBuilder {
        BundleContentsBuilder::default()
    }
}

impl BundleContents {
    /// bukkit: io/papermc/paper/datacomponent/item/BundleContents.bundleContents
    pub fn bundle_contents(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BundleContents.contents
    pub fn contents(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.BundleContents$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BundleContentsBuilder(pub Map<String, Value>);

impl BundleContentsBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BundleContents$Builder.add
    pub fn add(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "value", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BundleContents$Builder.addAll
    pub fn add_all(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "value", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BundleContents$Builder.build
    pub fn build(mut self) -> BundleContents {
        BundleContents(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.ChargedProjectiles`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChargedProjectiles(pub Value);

impl ChargedProjectiles {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ChargedProjectilesBuilder {
        ChargedProjectilesBuilder::default()
    }
}

impl ChargedProjectiles {
    /// bukkit: io/papermc/paper/datacomponent/item/ChargedProjectiles.chargedProjectiles
    pub fn charged_projectiles(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ChargedProjectiles.projectiles
    pub fn projectiles(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.ChargedProjectiles$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChargedProjectilesBuilder(pub Map<String, Value>);

impl ChargedProjectilesBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ChargedProjectiles$Builder.add
    pub fn add(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "value", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ChargedProjectiles$Builder.addAll
    pub fn add_all(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "value", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ChargedProjectiles$Builder.build
    pub fn build(mut self) -> ChargedProjectiles {
        ChargedProjectiles(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.SuspiciousStewEffects`.
#[derive(Clone, Debug, PartialEq)]
pub struct SuspiciousStewEffects(pub Value);

impl SuspiciousStewEffects {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> SuspiciousStewEffectsBuilder {
        SuspiciousStewEffectsBuilder::default()
    }
}

impl SuspiciousStewEffects {
    /// bukkit: io/papermc/paper/datacomponent/item/SuspiciousStewEffects.suspiciousStewEffects
    pub fn suspicious_stew_effects(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SuspiciousStewEffects.effects
    pub fn effects(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.SuspiciousStewEffects$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SuspiciousStewEffectsBuilder(pub Map<String, Value>);

impl SuspiciousStewEffectsBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SuspiciousStewEffects$Builder.add
    pub fn add(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "value", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SuspiciousStewEffects$Builder.addAll
    pub fn add_all(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "value", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SuspiciousStewEffects$Builder.build
    pub fn build(mut self) -> SuspiciousStewEffects {
        SuspiciousStewEffects(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.BlockItemDataProperties`.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockItemDataProperties(pub Value);

impl BlockItemDataProperties {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> BlockItemDataPropertiesBuilder {
        BlockItemDataPropertiesBuilder::default()
    }
}

impl BlockItemDataProperties {
    /// bukkit: io/papermc/paper/datacomponent/item/BlockItemDataProperties.blockItemStateProperties
    pub fn block_item_state_properties() -> BlockItemDataPropertiesBuilder {
        BlockItemDataPropertiesBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlockItemDataProperties.applyTo
    ///
    /// The properties this block has of the component's.
    pub fn apply_to(&self, data: &mut crate::block_data::BlockData) {
        for (key, value) in self.0.as_object().into_iter().flatten() {
            if data.property(key).is_some() {
                data.set_property(key, value.as_str().unwrap_or_default());
            }
        }
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlockItemDataProperties.createBlockData
    pub fn create_block_data(&self, block: &str) -> crate::block_data::BlockData {
        let mut data = crate::block_data::BlockData::parse(block);
        self.apply_to(&mut data);
        data
    }
}

/// `io.papermc.paper.datacomponent.item.BlockItemDataProperties$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockItemDataPropertiesBuilder(pub Map<String, Value>);

impl BlockItemDataPropertiesBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlockItemDataProperties$Builder.build
    pub fn build(self) -> BlockItemDataProperties {
        BlockItemDataProperties(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.BlocksAttacks`.
#[derive(Clone, Debug, PartialEq)]
pub struct BlocksAttacks(pub Value);

impl BlocksAttacks {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> BlocksAttacksBuilder {
        BlocksAttacksBuilder::default()
    }
}

impl BlocksAttacks {
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks.blocksAttacks
    pub fn blocks_attacks() -> BlocksAttacksBuilder {
        BlocksAttacksBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks.disableSound
    pub fn disable_sound(&self) -> &Value {
        &self.0["disabled_sound"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks.blockDelaySeconds
    pub fn block_delay_seconds(&self) -> &Value {
        &self.0["block_delay_seconds"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks.disableCooldownScale
    pub fn disable_cooldown_scale(&self) -> &Value {
        &self.0["disable_cooldown_scale"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks.damageReductions
    pub fn damage_reductions(&self) -> &Value {
        &self.0["damage_reductions"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks.itemDamage
    pub fn item_damage(&self) -> &Value {
        &self.0["item_damage"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks.bypassedBy
    pub fn bypassed_by(&self) -> &Value {
        &self.0["bypassed_by"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks.blockSound
    pub fn block_sound(&self) -> &Value {
        &self.0["block_sound"]
    }
}

/// `io.papermc.paper.datacomponent.item.BlocksAttacks$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlocksAttacksBuilder(pub Map<String, Value>);

impl BlocksAttacksBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.disableSound
    pub fn disable_sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("disabled_sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.blockDelaySeconds
    pub fn block_delay_seconds(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("block_delay_seconds".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.disableCooldownScale
    pub fn disable_cooldown_scale(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("disable_cooldown_scale".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.damageReductions
    pub fn damage_reductions(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("damage_reductions".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.itemDamage
    pub fn item_damage(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("item_damage".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.bypassedBy
    pub fn bypassed_by(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("bypassed_by".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.blockSound
    pub fn block_sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("block_sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.addDamageReduction
    pub fn add_damage_reduction(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "damage_reductions", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/BlocksAttacks$Builder.build
    pub fn build(self) -> BlocksAttacks {
        BlocksAttacks(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.Consumable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Consumable(pub Value);

impl Consumable {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ConsumableBuilder {
        ConsumableBuilder::default()
    }
}

impl Consumable {
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable.consumable
    pub fn consumable() -> ConsumableBuilder {
        ConsumableBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable.consumeEffects
    pub fn consume_effects(&self) -> &Value {
        &self.0["on_consume_effects"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable.consumeSeconds
    pub fn consume_seconds(&self) -> &Value {
        &self.0["consume_seconds"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable.animation
    pub fn animation(&self) -> &Value {
        &self.0["animation"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable.sound
    pub fn sound(&self) -> &Value {
        &self.0["sound"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable.hasConsumeParticles
    pub fn has_consume_particles(&self) -> &Value {
        &self.0["has_consume_particles"]
    }
}

/// `io.papermc.paper.datacomponent.item.Consumable$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConsumableBuilder(pub Map<String, Value>);

impl ConsumableBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable$Builder.effects
    pub fn effects(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("on_consume_effects".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable$Builder.consumeSeconds
    pub fn consume_seconds(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("consume_seconds".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable$Builder.animation
    pub fn animation(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("animation".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable$Builder.sound
    pub fn sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable$Builder.hasConsumeParticles
    pub fn has_consume_particles(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("has_consume_particles".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable$Builder.addEffect
    pub fn add_effect(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "on_consume_effects", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable$Builder.addEffects
    pub fn add_effects(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "on_consume_effects", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Consumable$Builder.build
    pub fn build(self) -> Consumable {
        Consumable(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.CustomModelData`.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomModelData(pub Value);

impl CustomModelData {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> CustomModelDataBuilder {
        CustomModelDataBuilder::default()
    }
}

impl CustomModelData {
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData.customModelData
    pub fn custom_model_data() -> CustomModelDataBuilder {
        CustomModelDataBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData.floats
    pub fn floats(&self) -> &Value {
        &self.0["floats"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData.flags
    pub fn flags(&self) -> &Value {
        &self.0["flags"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData.strings
    pub fn strings(&self) -> &Value {
        &self.0["strings"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData.colors
    pub fn colors(&self) -> &Value {
        &self.0["colors"]
    }
}

/// `io.papermc.paper.datacomponent.item.CustomModelData$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomModelDataBuilder(pub Map<String, Value>);

impl CustomModelDataBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.addFloat
    pub fn add_float(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "floats", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.addFloats
    pub fn add_floats(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "floats", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.addFlag
    pub fn add_flag(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "flags", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.addFlags
    pub fn add_flags(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "flags", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.addString
    pub fn add_string(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "strings", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.addStrings
    pub fn add_strings(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "strings", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.addColor
    pub fn add_color(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "colors", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.addColors
    pub fn add_colors(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "colors", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/CustomModelData$Builder.build
    pub fn build(self) -> CustomModelData {
        CustomModelData(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.DamageResistant`.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageResistant(pub Value);

impl DamageResistant {
    /// bukkit: io/papermc/paper/datacomponent/item/DamageResistant.damageResistant
    pub fn damage_resistant(types: impl Into<Value>) -> Self {
        Self(json!({ "types": types.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/DamageResistant.types
    pub fn types(&self) -> &Value {
        &self.0["types"]
    }
}

/// `io.papermc.paper.datacomponent.item.DeathProtection`.
#[derive(Clone, Debug, PartialEq)]
pub struct DeathProtection(pub Value);

impl DeathProtection {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> DeathProtectionBuilder {
        DeathProtectionBuilder::default()
    }
}

impl DeathProtection {
    /// bukkit: io/papermc/paper/datacomponent/item/DeathProtection.deathProtection
    pub fn death_protection(death_effects: impl Into<Value>) -> Self {
        Self(json!({ "death_effects": death_effects.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/DeathProtection.deathEffects
    pub fn death_effects(&self) -> &Value {
        &self.0["death_effects"]
    }
}

/// `io.papermc.paper.datacomponent.item.DeathProtection$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeathProtectionBuilder(pub Map<String, Value>);

impl DeathProtectionBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/DeathProtection$Builder.addEffect
    pub fn add_effect(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "death_effects", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/DeathProtection$Builder.addEffects
    pub fn add_effects(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "death_effects", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/DeathProtection$Builder.build
    pub fn build(self) -> DeathProtection {
        DeathProtection(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.DyedItemColor`.
#[derive(Clone, Debug, PartialEq)]
pub struct DyedItemColor(pub Value);

impl DyedItemColor {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> DyedItemColorBuilder {
        DyedItemColorBuilder::default()
    }
}

impl DyedItemColor {
    /// bukkit: io/papermc/paper/datacomponent/item/DyedItemColor.dyedItemColor
    pub fn dyed_item_color(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/DyedItemColor.color
    pub fn color(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.DyedItemColor$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DyedItemColorBuilder(pub Map<String, Value>);

impl DyedItemColorBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/DyedItemColor$Builder.color
    pub fn color(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("value".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/DyedItemColor$Builder.build
    pub fn build(mut self) -> DyedItemColor {
        DyedItemColor(self.0.remove("value").unwrap_or_else(|| json!(null)))
    }
}

/// `io.papermc.paper.datacomponent.item.Enchantable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Enchantable(pub Value);

impl Enchantable {
    /// bukkit: io/papermc/paper/datacomponent/item/Enchantable.enchantable
    pub fn enchantable(value: impl Into<Value>) -> Self {
        Self(json!({ "value": value.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Enchantable.value
    pub fn value(&self) -> &Value {
        &self.0["value"]
    }
}

/// `io.papermc.paper.datacomponent.item.Equippable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Equippable(pub Value);

impl Equippable {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> EquippableBuilder {
        EquippableBuilder::default()
    }
}

impl Equippable {
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.equippable
    pub fn equippable(slot: impl Into<Value>) -> EquippableBuilder {
        let mut built = Map::new();
        built.insert("slot".to_owned(), slot.into());
        EquippableBuilder(built)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.shearSound
    pub fn shear_sound(&self) -> &Value {
        &self.0["shearing_sound"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.slot
    pub fn slot(&self) -> &Value {
        &self.0["slot"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.assetId
    pub fn asset_id(&self) -> &Value {
        &self.0["asset_id"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.cameraOverlay
    pub fn camera_overlay(&self) -> &Value {
        &self.0["camera_overlay"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.allowedEntities
    pub fn allowed_entities(&self) -> &Value {
        &self.0["allowed_entities"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.dispensable
    pub fn dispensable(&self) -> &Value {
        &self.0["dispensable"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.swappable
    pub fn swappable(&self) -> &Value {
        &self.0["swappable"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.damageOnHurt
    pub fn damage_on_hurt(&self) -> &Value {
        &self.0["damage_on_hurt"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.equipOnInteract
    pub fn equip_on_interact(&self) -> &Value {
        &self.0["equip_on_interact"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.canBeSheared
    pub fn can_be_sheared(&self) -> &Value {
        &self.0["can_be_sheared"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable.equipSound
    pub fn equip_sound(&self) -> &Value {
        &self.0["equip_sound"]
    }
}

/// `io.papermc.paper.datacomponent.item.Equippable$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EquippableBuilder(pub Map<String, Value>);

impl EquippableBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.shearSound
    pub fn shear_sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("shearing_sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.assetId
    pub fn asset_id(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.cameraOverlay
    pub fn camera_overlay(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("camera_overlay".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.allowedEntities
    pub fn allowed_entities(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("allowed_entities".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.dispensable
    pub fn dispensable(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("dispensable".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.swappable
    pub fn swappable(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("swappable".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.damageOnHurt
    pub fn damage_on_hurt(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("damage_on_hurt".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.equipOnInteract
    pub fn equip_on_interact(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("equip_on_interact".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.canBeSheared
    pub fn can_be_sheared(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("can_be_sheared".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.equipSound
    pub fn equip_sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("equip_sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Equippable$Builder.build
    pub fn build(self) -> Equippable {
        Equippable(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.Fireworks`.
#[derive(Clone, Debug, PartialEq)]
pub struct Fireworks(pub Value);

impl Fireworks {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> FireworksBuilder {
        FireworksBuilder::default()
    }
}

impl Fireworks {
    /// bukkit: io/papermc/paper/datacomponent/item/Fireworks.fireworks
    pub fn fireworks(explosions: impl Into<Value>, flight_duration: impl Into<Value>) -> Self {
        Self(json!({ "explosions": explosions.into(), "flight_duration": flight_duration.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Fireworks.effects
    pub fn effects(&self) -> &Value {
        &self.0["explosions"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Fireworks.flightDuration
    pub fn flight_duration(&self) -> &Value {
        &self.0["flight_duration"]
    }
}

/// `io.papermc.paper.datacomponent.item.Fireworks$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FireworksBuilder(pub Map<String, Value>);

impl FireworksBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Fireworks$Builder.flightDuration
    pub fn flight_duration(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("flight_duration".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Fireworks$Builder.addEffect
    pub fn add_effect(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "explosions", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Fireworks$Builder.addEffects
    pub fn add_effects(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "explosions", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Fireworks$Builder.build
    pub fn build(self) -> Fireworks {
        Fireworks(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.FoodProperties`.
#[derive(Clone, Debug, PartialEq)]
pub struct FoodProperties(pub Value);

impl FoodProperties {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> FoodPropertiesBuilder {
        FoodPropertiesBuilder::default()
    }
}

impl FoodProperties {
    /// bukkit: io/papermc/paper/datacomponent/item/FoodProperties.food
    pub fn food() -> FoodPropertiesBuilder {
        FoodPropertiesBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/FoodProperties.nutrition
    pub fn nutrition(&self) -> &Value {
        &self.0["nutrition"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/FoodProperties.saturation
    pub fn saturation(&self) -> &Value {
        &self.0["saturation"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/FoodProperties.canAlwaysEat
    pub fn can_always_eat(&self) -> &Value {
        &self.0["can_always_eat"]
    }
}

/// `io.papermc.paper.datacomponent.item.FoodProperties$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FoodPropertiesBuilder(pub Map<String, Value>);

impl FoodPropertiesBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/FoodProperties$Builder.nutrition
    pub fn nutrition(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("nutrition".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/FoodProperties$Builder.saturation
    pub fn saturation(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("saturation".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/FoodProperties$Builder.canAlwaysEat
    pub fn can_always_eat(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("can_always_eat".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/FoodProperties$Builder.build
    pub fn build(self) -> FoodProperties {
        FoodProperties(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.ItemAdventurePredicate`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemAdventurePredicate(pub Value);

impl ItemAdventurePredicate {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ItemAdventurePredicateBuilder {
        ItemAdventurePredicateBuilder::default()
    }
}

impl ItemAdventurePredicate {
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAdventurePredicate.itemAdventurePredicate
    pub fn item_adventure_predicate(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAdventurePredicate.predicates
    pub fn predicates(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.ItemAdventurePredicate$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemAdventurePredicateBuilder(pub Map<String, Value>);

impl ItemAdventurePredicateBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAdventurePredicate$Builder.addPredicate
    pub fn add_predicate(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "value", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAdventurePredicate$Builder.addPredicates
    pub fn add_predicates(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "value", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAdventurePredicate$Builder.build
    pub fn build(mut self) -> ItemAdventurePredicate {
        ItemAdventurePredicate(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.ItemArmorTrim`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemArmorTrim(pub Value);

impl ItemArmorTrim {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ItemArmorTrimBuilder {
        ItemArmorTrimBuilder::default()
    }
}

impl ItemArmorTrim {
    /// bukkit: io/papermc/paper/datacomponent/item/ItemArmorTrim.itemArmorTrim
    pub fn item_armor_trim(armor_trim: impl Into<Value>) -> ItemArmorTrimBuilder {
        let mut built = Map::new();
        built.insert("value".to_owned(), armor_trim.into());
        ItemArmorTrimBuilder(built)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemArmorTrim.armorTrim
    pub fn armor_trim(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.ItemArmorTrim$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemArmorTrimBuilder(pub Map<String, Value>);

impl ItemArmorTrimBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemArmorTrim$Builder.armorTrim
    pub fn armor_trim(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("value".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemArmorTrim$Builder.build
    pub fn build(mut self) -> ItemArmorTrim {
        ItemArmorTrim(self.0.remove("value").unwrap_or_else(|| json!(null)))
    }
}

/// `io.papermc.paper.datacomponent.item.ItemAttributeModifiers`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemAttributeModifiers(pub Value);

impl ItemAttributeModifiers {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ItemAttributeModifiersBuilder {
        ItemAttributeModifiersBuilder::default()
    }
}

impl ItemAttributeModifiers {
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAttributeModifiers.itemAttributes
    pub fn item_attributes() -> ItemAttributeModifiersBuilder {
        ItemAttributeModifiersBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAttributeModifiers.modifiers
    pub fn modifiers(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.ItemAttributeModifiers$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemAttributeModifiersBuilder(pub Map<String, Value>);

impl ItemAttributeModifiersBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAttributeModifiers$Builder.addModifier
    pub fn add_modifier(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "value", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAttributeModifiers$Builder.build
    pub fn build(mut self) -> ItemAttributeModifiers {
        ItemAttributeModifiers(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.ItemAttributeModifiers$Entry`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemAttributeModifiersEntry(pub Value);

impl ItemAttributeModifiersEntry {
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAttributeModifiers$Entry.attribute
    pub fn attribute(&self) -> &Value {
        &self.0["type"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAttributeModifiers$Entry.getGroup
    pub fn get_group(&self) -> &Value {
        &self.0["slot"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAttributeModifiers$Entry.display
    pub fn display(&self) -> &Value {
        &self.0["display"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemAttributeModifiers$Entry.modifier
    pub fn modifier(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.ItemContainerContents`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemContainerContents(pub Value);

impl ItemContainerContents {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ItemContainerContentsBuilder {
        ItemContainerContentsBuilder::default()
    }
}

impl ItemContainerContents {
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents.containerContents
    ///
    /// The stacks in slot order, from slot 0.
    pub fn container_contents(stacks: impl Into<Value>) -> Self {
        let rows: Vec<Value> = match stacks.into() {
            Value::Array(stacks) => stacks.into_iter().enumerate().filter(|(_, stack)| !stack.is_null()).map(|(slot, item)| json!({ "slot": slot, "item": item })).collect(),
            _ => Vec::new(),
        };
        Self(Value::Array(rows))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents.contents
    ///
    /// The stacks by slot, up to the last that has one; `Null` for an empty slot.
    pub fn contents(&self) -> Vec<Value> {
        let rows = self.0.as_array().cloned().unwrap_or_default();
        let size = rows.iter().filter_map(|row| row["slot"].as_u64()).max().map_or(0, |last| last as usize + 1);
        let mut stacks = vec![Value::Null; size];
        for row in rows {
            if let Some(place) = row["slot"].as_u64().and_then(|slot| stacks.get_mut(slot as usize)) {
                *place = row["item"].clone();
            }
        }
        stacks
    }
}

/// `io.papermc.paper.datacomponent.item.ItemContainerContents$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemContainerContentsBuilder(pub Map<String, Value>);

impl ItemContainerContentsBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    fn put(&mut self, item: Value) {
        let slot = self.0.get("value").and_then(Value::as_array).map_or(0, Vec::len);
        push(&mut self.0, "value", json!({ "slot": slot, "item": item }));
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents$Builder.add
    pub fn add(mut self, value: impl Into<Value>) -> Self {
        self.put(value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents$Builder.addAll
    pub fn add_all(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(stacks) = value.into() {
            for stack in stacks {
                self.put(stack);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemContainerContents$Builder.build
    pub fn build(mut self) -> ItemContainerContents {
        ItemContainerContents(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.ItemEnchantments`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemEnchantments(pub Value);

impl ItemEnchantments {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ItemEnchantmentsBuilder {
        ItemEnchantmentsBuilder::default()
    }
}

impl ItemEnchantments {
    /// bukkit: io/papermc/paper/datacomponent/item/ItemEnchantments.itemEnchantments
    pub fn item_enchantments(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemEnchantments.enchantments
    pub fn enchantments(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.ItemEnchantments$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemEnchantmentsBuilder(pub Map<String, Value>);

impl ItemEnchantmentsBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemEnchantments$Builder.add
    pub fn add(mut self, enchantment: &str, level: i32) -> Self {
        self.0.insert(enchantment.to_owned(), json!(level));
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemEnchantments$Builder.addAll
    pub fn add_all(mut self, enchantments: impl Into<Value>) -> Self {
        if let Value::Object(rows) = enchantments.into() {
            self.0.extend(rows);
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemEnchantments$Builder.build
    pub fn build(self) -> ItemEnchantments {
        ItemEnchantments(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.ItemLore`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemLore(pub Value);

impl ItemLore {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ItemLoreBuilder {
        ItemLoreBuilder::default()
    }
}

impl ItemLore {
    /// bukkit: io/papermc/paper/datacomponent/item/ItemLore.lore
    pub fn lore(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemLore.lines
    pub fn lines(&self) -> &Value {
        &self.0
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemLore.styledLines
    ///
    /// The lines under lore's own style, dark purple italic, which a line's own styles beat.
    pub fn styled_lines(&self) -> Vec<Value> {
        self.0.as_array().into_iter().flatten().map(|line| {
            let mut styled = if line.is_object() { line.clone() } else { json!({ "text": line }) };
            for (key, value) in [("color", json!("dark_purple")), ("italic", json!(true))] {
                if styled.get(key).is_none() {
                    styled[key] = value;
                }
            }
            styled
        }).collect()
    }
}

/// `io.papermc.paper.datacomponent.item.ItemLore$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemLoreBuilder(pub Map<String, Value>);

impl ItemLoreBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemLore$Builder.addLine
    pub fn add_line(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "value", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemLore$Builder.addLines
    pub fn add_lines(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "value", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemLore$Builder.lines
    pub fn lines(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("value".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ItemLore$Builder.build
    pub fn build(mut self) -> ItemLore {
        ItemLore(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.JukeboxPlayable`.
#[derive(Clone, Debug, PartialEq)]
pub struct JukeboxPlayable(pub Value);

impl JukeboxPlayable {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> JukeboxPlayableBuilder {
        JukeboxPlayableBuilder::default()
    }
}

impl JukeboxPlayable {
    /// bukkit: io/papermc/paper/datacomponent/item/JukeboxPlayable.jukeboxPlayable
    pub fn jukebox_playable(jukebox_song: impl Into<Value>) -> JukeboxPlayableBuilder {
        let mut built = Map::new();
        built.insert("value".to_owned(), jukebox_song.into());
        JukeboxPlayableBuilder(built)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/JukeboxPlayable.jukeboxSong
    pub fn jukebox_song(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.JukeboxPlayable$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JukeboxPlayableBuilder(pub Map<String, Value>);

impl JukeboxPlayableBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/JukeboxPlayable$Builder.jukeboxSong
    pub fn jukebox_song(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("value".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/JukeboxPlayable$Builder.build
    pub fn build(mut self) -> JukeboxPlayable {
        JukeboxPlayable(self.0.remove("value").unwrap_or_else(|| json!(null)))
    }
}

/// `io.papermc.paper.datacomponent.item.KineticWeapon`.
#[derive(Clone, Debug, PartialEq)]
pub struct KineticWeapon(pub Value);

impl KineticWeapon {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> KineticWeaponBuilder {
        KineticWeaponBuilder::default()
    }
}

impl KineticWeapon {
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.kineticWeapon
    pub fn kinetic_weapon() -> KineticWeaponBuilder {
        KineticWeaponBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.contactCooldownTicks
    pub fn contact_cooldown_ticks(&self) -> &Value {
        &self.0["contact_cooldown_ticks"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.delayTicks
    pub fn delay_ticks(&self) -> &Value {
        &self.0["delay_ticks"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.dismountConditions
    pub fn dismount_conditions(&self) -> &Value {
        &self.0["dismount_conditions"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.knockbackConditions
    pub fn knockback_conditions(&self) -> &Value {
        &self.0["knockback_conditions"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.damageConditions
    pub fn damage_conditions(&self) -> &Value {
        &self.0["damage_conditions"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.forwardMovement
    pub fn forward_movement(&self) -> &Value {
        &self.0["forward_movement"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.damageMultiplier
    pub fn damage_multiplier(&self) -> &Value {
        &self.0["damage_multiplier"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.sound
    pub fn sound(&self) -> &Value {
        &self.0["sound"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.hitSound
    pub fn hit_sound(&self) -> &Value {
        &self.0["hit_sound"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon.condition
    pub fn condition(max_duration_ticks: i32, min_speed: f32, min_relative_speed: f32) -> KineticWeaponCondition {
        KineticWeaponCondition(json!({ "max_duration_ticks": max_duration_ticks, "min_speed": min_speed, "min_relative_speed": min_relative_speed }))
    }
}

/// `io.papermc.paper.datacomponent.item.KineticWeapon$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KineticWeaponBuilder(pub Map<String, Value>);

impl KineticWeaponBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.contactCooldownTicks
    pub fn contact_cooldown_ticks(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("contact_cooldown_ticks".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.delayTicks
    pub fn delay_ticks(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("delay_ticks".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.dismountConditions
    pub fn dismount_conditions(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("dismount_conditions".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.knockbackConditions
    pub fn knockback_conditions(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("knockback_conditions".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.damageConditions
    pub fn damage_conditions(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("damage_conditions".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.forwardMovement
    pub fn forward_movement(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("forward_movement".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.damageMultiplier
    pub fn damage_multiplier(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("damage_multiplier".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.sound
    pub fn sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.hitSound
    pub fn hit_sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("hit_sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Builder.build
    pub fn build(self) -> KineticWeapon {
        KineticWeapon(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.KineticWeapon$Condition`.
#[derive(Clone, Debug, PartialEq)]
pub struct KineticWeaponCondition(pub Value);

impl KineticWeaponCondition {
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Condition.maxDurationTicks
    pub fn max_duration_ticks(&self) -> &Value {
        &self.0["max_duration_ticks"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Condition.minSpeed
    pub fn min_speed(&self) -> &Value {
        &self.0["min_speed"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/KineticWeapon$Condition.minRelativeSpeed
    pub fn min_relative_speed(&self) -> &Value {
        &self.0["min_relative_speed"]
    }
}

/// `io.papermc.paper.datacomponent.item.LodestoneTracker`.
#[derive(Clone, Debug, PartialEq)]
pub struct LodestoneTracker(pub Value);

impl LodestoneTracker {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> LodestoneTrackerBuilder {
        LodestoneTrackerBuilder::default()
    }
}

impl LodestoneTracker {
    /// bukkit: io/papermc/paper/datacomponent/item/LodestoneTracker.lodestoneTracker
    pub fn lodestone_tracker(target: impl Into<Value>, tracked: impl Into<Value>) -> Self {
        Self(json!({ "target": target.into(), "tracked": tracked.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/LodestoneTracker.location
    pub fn location(&self) -> &Value {
        &self.0["target"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/LodestoneTracker.tracked
    pub fn tracked(&self) -> &Value {
        &self.0["tracked"]
    }
}

/// `io.papermc.paper.datacomponent.item.LodestoneTracker$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LodestoneTrackerBuilder(pub Map<String, Value>);

impl LodestoneTrackerBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/LodestoneTracker$Builder.location
    pub fn location(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("target".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/LodestoneTracker$Builder.tracked
    pub fn tracked(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("tracked".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/LodestoneTracker$Builder.build
    pub fn build(self) -> LodestoneTracker {
        LodestoneTracker(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.MapDecorations`.
#[derive(Clone, Debug, PartialEq)]
pub struct MapDecorations(pub Value);

impl MapDecorations {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> MapDecorationsBuilder {
        MapDecorationsBuilder::default()
    }
}

impl MapDecorations {
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations.mapDecorations
    pub fn map_decorations(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations.decorations
    pub fn decorations(&self) -> &Value {
        &self.0
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations.decoration
    pub fn decoration(&self, id: &str) -> Option<&Value> {
        self.0.get(id)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations.decorationEntry
    pub fn decoration_entry(kind: impl Into<Value>, x: f64, z: f64, rotation: f32) -> MapDecorationsDecorationEntry {
        MapDecorationsDecorationEntry(json!({ "type": kind.into(), "x": x, "z": z, "rotation": rotation }))
    }
}

/// `io.papermc.paper.datacomponent.item.MapDecorations$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapDecorationsBuilder(pub Map<String, Value>);

impl MapDecorationsBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$Builder.put
    pub fn put(mut self, id: &str, entry: impl Into<Value>) -> Self {
        self.0.insert(id.to_owned(), entry.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$Builder.putAll
    pub fn put_all(mut self, entries: impl Into<Value>) -> Self {
        if let Value::Object(rows) = entries.into() {
            self.0.extend(rows);
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$Builder.build
    pub fn build(self) -> MapDecorations {
        MapDecorations(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.MapDecorations$DecorationEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct MapDecorationsDecorationEntry(pub Value);

impl MapDecorationsDecorationEntry {
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$DecorationEntry.type
    pub fn type_(&self) -> &Value {
        &self.0["type"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$DecorationEntry.rotation
    pub fn rotation(&self) -> &Value {
        &self.0["rotation"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$DecorationEntry.x
    pub fn x(&self) -> &Value {
        &self.0["x"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapDecorations$DecorationEntry.z
    pub fn z(&self) -> &Value {
        &self.0["z"]
    }
}

/// `io.papermc.paper.datacomponent.item.MapId`.
#[derive(Clone, Debug, PartialEq)]
pub struct MapId(pub Value);

impl MapId {
    /// bukkit: io/papermc/paper/datacomponent/item/MapId.mapId
    pub fn map_id(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MapId.id
    pub fn id(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.MobVisibility`.
#[derive(Clone, Debug, PartialEq)]
pub struct MobVisibility(pub Value);

impl MobVisibility {
    /// bukkit: io/papermc/paper/datacomponent/item/MobVisibility.mobVisibility
    pub fn mob_visibility(targeting_entity_types: impl Into<Value>, visibility: impl Into<Value>) -> Self {
        Self(json!({ "targeting_entity_types": targeting_entity_types.into(), "visibility": visibility.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MobVisibility.targetingEntityTypes
    pub fn targeting_entity_types(&self) -> &Value {
        &self.0["targeting_entity_types"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/MobVisibility.visibility
    pub fn visibility(&self) -> &Value {
        &self.0["visibility"]
    }
}

/// `io.papermc.paper.datacomponent.item.OminousBottleAmplifier`.
#[derive(Clone, Debug, PartialEq)]
pub struct OminousBottleAmplifier(pub Value);

impl OminousBottleAmplifier {
    /// bukkit: io/papermc/paper/datacomponent/item/OminousBottleAmplifier.amplifier
    pub fn amplifier(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/OminousBottleAmplifier.amplifier
    pub fn value(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.PiercingWeapon`.
#[derive(Clone, Debug, PartialEq)]
pub struct PiercingWeapon(pub Value);

impl PiercingWeapon {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> PiercingWeaponBuilder {
        PiercingWeaponBuilder::default()
    }
}

impl PiercingWeapon {
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon.piercingWeapon
    pub fn piercing_weapon() -> PiercingWeaponBuilder {
        PiercingWeaponBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon.dealsKnockback
    pub fn deals_knockback(&self) -> &Value {
        &self.0["deals_knockback"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon.dismounts
    pub fn dismounts(&self) -> &Value {
        &self.0["dismounts"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon.sound
    pub fn sound(&self) -> &Value {
        &self.0["sound"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon.hitSound
    pub fn hit_sound(&self) -> &Value {
        &self.0["hit_sound"]
    }
}

/// `io.papermc.paper.datacomponent.item.PiercingWeapon$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PiercingWeaponBuilder(pub Map<String, Value>);

impl PiercingWeaponBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon$Builder.dealsKnockback
    pub fn deals_knockback(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("deals_knockback".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon$Builder.dismounts
    pub fn dismounts(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("dismounts".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon$Builder.sound
    pub fn sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon$Builder.hitSound
    pub fn hit_sound(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("hit_sound".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PiercingWeapon$Builder.build
    pub fn build(self) -> PiercingWeapon {
        PiercingWeapon(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.PotDecorations`.
#[derive(Clone, Debug, PartialEq)]
pub struct PotDecorations(pub Value);

impl PotDecorations {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> PotDecorationsBuilder {
        PotDecorationsBuilder::default()
    }
}

impl PotDecorations {
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.potDecorations
    ///
    /// The game keeps the four as a list; `Null` is the plain side, a brick's.
    pub fn pot_decorations(back: Value, left: Value, right: Value, front: Value) -> Self {
        let side = |value: Value| if value.is_null() { json!("minecraft:brick") } else { value };
        Self(json!([side(back), side(left), side(right), side(front)]))
    }
    fn side(&self, index: usize) -> Option<&Value> {
        self.0.get(index).filter(|side| side.as_str() != Some("minecraft:brick"))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.back
    pub fn back(&self) -> Option<&Value> {
        self.side(0)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.left
    pub fn left(&self) -> Option<&Value> {
        self.side(1)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.right
    pub fn right(&self) -> Option<&Value> {
        self.side(2)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations.front
    pub fn front(&self) -> Option<&Value> {
        self.side(3)
    }
}

/// `io.papermc.paper.datacomponent.item.PotDecorations$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PotDecorationsBuilder(pub Map<String, Value>);

impl PotDecorationsBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(Map::from_iter([("value".to_owned(), built.clone())]))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations$Builder.back
    pub fn back(mut self, value: impl Into<Value>) -> Self {
        let sides = self.0.entry("value".to_owned()).or_insert_with(|| json!(["minecraft:brick", "minecraft:brick", "minecraft:brick", "minecraft:brick"]));
        let value = value.into();
        sides[0] = if value.is_null() { json!("minecraft:brick") } else { value };
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations$Builder.left
    pub fn left(mut self, value: impl Into<Value>) -> Self {
        let sides = self.0.entry("value".to_owned()).or_insert_with(|| json!(["minecraft:brick", "minecraft:brick", "minecraft:brick", "minecraft:brick"]));
        let value = value.into();
        sides[1] = if value.is_null() { json!("minecraft:brick") } else { value };
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations$Builder.right
    pub fn right(mut self, value: impl Into<Value>) -> Self {
        let sides = self.0.entry("value".to_owned()).or_insert_with(|| json!(["minecraft:brick", "minecraft:brick", "minecraft:brick", "minecraft:brick"]));
        let value = value.into();
        sides[2] = if value.is_null() { json!("minecraft:brick") } else { value };
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations$Builder.front
    pub fn front(mut self, value: impl Into<Value>) -> Self {
        let sides = self.0.entry("value".to_owned()).or_insert_with(|| json!(["minecraft:brick", "minecraft:brick", "minecraft:brick", "minecraft:brick"]));
        let value = value.into();
        sides[3] = if value.is_null() { json!("minecraft:brick") } else { value };
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotDecorations$Builder.build
    pub fn build(mut self) -> PotDecorations {
        PotDecorations(self.0.remove("value").unwrap_or_else(|| json!([])))
    }
}

/// `io.papermc.paper.datacomponent.item.PotionContents`.
#[derive(Clone, Debug, PartialEq)]
pub struct PotionContents(pub Value);

impl PotionContents {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> PotionContentsBuilder {
        PotionContentsBuilder::default()
    }
}

impl PotionContents {
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.potionContents
    pub fn potion_contents() -> PotionContentsBuilder {
        PotionContentsBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.potion
    pub fn potion(&self) -> &Value {
        &self.0["potion"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.customColor
    pub fn custom_color(&self) -> &Value {
        &self.0["custom_color"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.customEffects
    pub fn custom_effects(&self) -> &Value {
        &self.0["custom_effects"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.customName
    pub fn custom_name(&self) -> &Value {
        &self.0["custom_name"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.allEffects
    ///
    /// The potion's own effects, then the custom ones.
    pub fn all_effects(&self) -> Vec<Value> {
        let own = self.0["potion"].as_str().map_or(Value::Null, |potion| horizon_mod::query(json!({ "op": "items_potion", "id": potion })));
        own["effects"].as_array().into_iter().chain(self.0["custom_effects"].as_array()).flatten().cloned().collect()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents.computeEffectiveColor
    ///
    /// `PotionContents.getColor`: the custom colour, or the visible effects' colours by level.
    pub fn compute_effective_color(&self) -> crate::Color {
        if let Some(color) = self.0["custom_color"].as_i64() {
            return crate::Color::from_rgb(((color >> 16) & 255) as u8, ((color >> 8) & 255) as u8, (color & 255) as u8);
        }
        let (mut sum, mut total) = ([0_i64; 3], 0_i64);
        for effect in self.all_effects() {
            if effect["show_particles"] == false || effect["visible"] == false {
                continue;
            }
            let Some(id) = effect["id"].as_str() else {
                continue;
            };
            let color = crate::fam_items::PotionEffectType::of(id).get_color();
            let level = effect["amplifier"].as_i64().unwrap_or(0) + 1;
            sum[0] += level * i64::from(color.get_red());
            sum[1] += level * i64::from(color.get_green());
            sum[2] += level * i64::from(color.get_blue());
            total += level;
        }
        if total == 0 {
            // `BASE_POTION_COLOR`.
            return crate::Color::from_rgb(0x38, 0x5D, 0xC6);
        }
        crate::Color::from_rgb((sum[0] / total) as u8, (sum[1] / total) as u8, (sum[2] / total) as u8)
    }
}

/// `io.papermc.paper.datacomponent.item.PotionContents$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PotionContentsBuilder(pub Map<String, Value>);

impl PotionContentsBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents$Builder.potion
    pub fn potion(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("potion".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents$Builder.customColor
    pub fn custom_color(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("custom_color".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents$Builder.customName
    pub fn custom_name(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("custom_name".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents$Builder.addCustomEffect
    pub fn add_custom_effect(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "custom_effects", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents$Builder.addCustomEffects
    pub fn add_custom_effects(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "custom_effects", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/PotionContents$Builder.build
    pub fn build(self) -> PotionContents {
        PotionContents(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.Repairable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Repairable(pub Value);

impl Repairable {
    /// bukkit: io/papermc/paper/datacomponent/item/Repairable.repairable
    pub fn repairable(items: impl Into<Value>) -> Self {
        Self(json!({ "items": items.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Repairable.types
    pub fn types(&self) -> &Value {
        &self.0["items"]
    }
}

/// `io.papermc.paper.datacomponent.item.ResolvableProfile`.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvableProfile(pub Value);

impl ResolvableProfile {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ResolvableProfileBuilder {
        ResolvableProfileBuilder::default()
    }
}

impl ResolvableProfile {
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.resolvableProfile
    pub fn resolvable_profile(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.uuid
    pub fn uuid(&self) -> &Value {
        &self.0["id"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.name
    pub fn name(&self) -> &Value {
        &self.0["name"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.properties
    pub fn properties(&self) -> &Value {
        &self.0["properties"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.dynamic
    ///
    /// A name or an id alone, no properties: the game looks it up each time it's shown.
    pub fn dynamic(&self) -> bool {
        let empty = self.0["properties"].as_array().is_none_or(Vec::is_empty) && self.0["properties"].as_object().is_none_or(Map::is_empty);
        empty && self.0["name"].is_string() != !self.0["id"].is_null()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.resolve
    ///
    /// Java's future is `callback`, on the server thread. A dynamic profile is looked up, as `PlayerProfile.update`
    /// does; a static one comes as it is, with an empty name or the offline id of its name in place of what it lacks.
    pub fn resolve(&self, callback: impl FnOnce(crate::player::PlayerProfile) + Send + 'static) {
        let id = crate::data::uuid(&self.0["id"]).unwrap_or_default();
        let name = self.0["name"].as_str().unwrap_or_default().to_owned();
        let rows = self.0["properties"].as_array().cloned().unwrap_or_default();
        let properties = rows.iter().map(|row| (row["name"].as_str().unwrap_or_default().to_owned(), row["value"].as_str().unwrap_or_default().to_owned(), row["signature"].as_str().map(str::to_owned))).collect();
        let mut profile = crate::player::PlayerProfile { name, id, properties };
        if self.dynamic() {
            profile.update(callback);
            return;
        }
        if profile.id.is_empty() {
            profile.id = horizon_mod::query(json!({ "op": "paper_offline_uuid", "name": profile.name })).as_str().unwrap_or_default().to_owned();
        }
        callback(profile);
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile.skinPatch
    pub fn skin_patch(&self) -> ResolvableProfileSkinPatch {
        let mut patch = Map::new();
        for key in ["texture", "cape", "elytra", "model"] {
            if let Some(value) = self.0.get(key) {
                patch.insert(key.to_owned(), value.clone());
            }
        }
        ResolvableProfileSkinPatch(Value::Object(patch))
    }
}

/// `io.papermc.paper.datacomponent.item.ResolvableProfile$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvableProfileBuilder(pub Map<String, Value>);

impl ResolvableProfileBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$Builder.uuid
    pub fn uuid(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$Builder.name
    pub fn name(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("name".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$Builder.addProperty
    pub fn add_property(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "properties", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$Builder.addProperties
    pub fn add_properties(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "properties", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$Builder.skinPatch
    pub fn skin_patch(mut self, patch: &ResolvableProfileSkinPatch) -> Self {
        for (key, value) in patch.0.as_object().into_iter().flatten() {
            self.0.insert(key.clone(), value.clone());
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$Builder.build
    pub fn build(self) -> ResolvableProfile {
        ResolvableProfile(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.ResolvableProfile$SkinPatch`.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvableProfileSkinPatch(pub Value);

impl ResolvableProfileSkinPatch {
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.body
    pub fn body(&self) -> &Value {
        &self.0["texture"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.cape
    pub fn cape(&self) -> &Value {
        &self.0["cape"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.elytra
    pub fn elytra(&self) -> &Value {
        &self.0["elytra"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.model
    pub fn model(&self) -> &Value {
        &self.0["model"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.empty
    pub fn empty() -> Self {
        Self(json!({}))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.skinPatch
    pub fn skin_patch() -> ResolvableProfileSkinPatchBuilder {
        ResolvableProfileSkinPatchBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatch.isEmpty
    pub fn is_empty(&self) -> bool {
        self.0.as_object().is_none_or(Map::is_empty)
    }
}

/// `io.papermc.paper.datacomponent.item.ResolvableProfile$SkinPatchBuilder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvableProfileSkinPatchBuilder(pub Map<String, Value>);

impl ResolvableProfileSkinPatchBuilder {
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatchBuilder.body
    pub fn body(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("texture".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatchBuilder.cape
    pub fn cape(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("cape".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatchBuilder.elytra
    pub fn elytra(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("elytra".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/ResolvableProfile$SkinPatchBuilder.model
    pub fn model(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("model".to_owned(), value.into());
        self
    }
    /// The patch built.
    pub fn build(self) -> ResolvableProfileSkinPatch {
        ResolvableProfileSkinPatch(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.SeededContainerLoot`.
#[derive(Clone, Debug, PartialEq)]
pub struct SeededContainerLoot(pub Value);

impl SeededContainerLoot {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> SeededContainerLootBuilder {
        SeededContainerLootBuilder::default()
    }
}

impl SeededContainerLoot {
    /// bukkit: io/papermc/paper/datacomponent/item/SeededContainerLoot.seededContainerLoot
    pub fn seeded_container_loot(loot_table: impl Into<Value>, seed: impl Into<Value>) -> Self {
        Self(json!({ "loot_table": loot_table.into(), "seed": seed.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SeededContainerLoot.lootTable
    pub fn loot_table(&self) -> &Value {
        &self.0["loot_table"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SeededContainerLoot.seed
    pub fn seed(&self) -> &Value {
        &self.0["seed"]
    }
}

/// `io.papermc.paper.datacomponent.item.SeededContainerLoot$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SeededContainerLootBuilder(pub Map<String, Value>);

impl SeededContainerLootBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SeededContainerLoot$Builder.lootTable
    pub fn loot_table(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("loot_table".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SeededContainerLoot$Builder.seed
    pub fn seed(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("seed".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SeededContainerLoot$Builder.build
    pub fn build(self) -> SeededContainerLoot {
        SeededContainerLoot(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.SignText`.
#[derive(Clone, Debug, PartialEq)]
pub struct SignText(pub Value);

impl SignText {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> SignTextBuilder {
        SignTextBuilder::default()
    }
}

impl SignText {
    /// bukkit: io/papermc/paper/datacomponent/item/SignText.signText
    pub fn sign_text(lines: impl Into<Value>) -> Self {
        Self(json!({ "messages": lines.into() }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SignText.lines
    pub fn lines(&self) -> &Value {
        &self.0["messages"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SignText.color
    pub fn color(&self) -> &Value {
        &self.0["color"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SignText.hasGlowingText
    pub fn has_glowing_text(&self) -> &Value {
        &self.0["has_glowing_text"]
    }
}

/// `io.papermc.paper.datacomponent.item.SignText$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignTextBuilder(pub Map<String, Value>);

impl SignTextBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SignText$Builder.lines
    pub fn lines(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("messages".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SignText$Builder.color
    pub fn color(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("color".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SignText$Builder.hasGlowingText
    pub fn has_glowing_text(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("has_glowing_text".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SignText$Builder.line
    pub fn line(mut self, index: usize, line: impl Into<Value>) -> Self {
        let lines = self.0.entry("messages".to_owned()).or_insert_with(|| json!(["", "", "", ""]));
        lines[index] = line.into();
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SignText$Builder.build
    pub fn build(self) -> SignText {
        SignText(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.SulfurCubeContent`.
#[derive(Clone, Debug, PartialEq)]
pub struct SulfurCubeContent(pub Value);

impl SulfurCubeContent {
    /// bukkit: io/papermc/paper/datacomponent/item/SulfurCubeContent.sulfurCubeContent
    pub fn sulfur_cube_content(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SulfurCubeContent.absorbedItem
    pub fn absorbed_item(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.SwingAnimation`.
#[derive(Clone, Debug, PartialEq)]
pub struct SwingAnimation(pub Value);

impl SwingAnimation {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> SwingAnimationBuilder {
        SwingAnimationBuilder::default()
    }
}

impl SwingAnimation {
    /// bukkit: io/papermc/paper/datacomponent/item/SwingAnimation.swingAnimation
    pub fn swing_animation() -> SwingAnimationBuilder {
        SwingAnimationBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SwingAnimation.type
    pub fn type_(&self) -> &Value {
        &self.0["type"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SwingAnimation.duration
    pub fn duration(&self) -> &Value {
        &self.0["duration"]
    }
}

/// `io.papermc.paper.datacomponent.item.SwingAnimation$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwingAnimationBuilder(pub Map<String, Value>);

impl SwingAnimationBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SwingAnimation$Builder.type
    pub fn type_(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("type".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SwingAnimation$Builder.duration
    pub fn duration(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("duration".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SwingAnimation$Builder.build
    pub fn build(self) -> SwingAnimation {
        SwingAnimation(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.Tool`.
#[derive(Clone, Debug, PartialEq)]
pub struct Tool(pub Value);

impl Tool {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ToolBuilder {
        ToolBuilder::default()
    }
}

impl Tool {
    /// bukkit: io/papermc/paper/datacomponent/item/Tool.tool
    pub fn tool() -> ToolBuilder {
        ToolBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool.rules
    pub fn rules(&self) -> &Value {
        &self.0["rules"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool.defaultMiningSpeed
    pub fn default_mining_speed(&self) -> &Value {
        &self.0["default_mining_speed"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool.damagePerBlock
    pub fn damage_per_block(&self) -> &Value {
        &self.0["damage_per_block"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool.canDestroyBlocksInCreative
    pub fn can_destroy_blocks_in_creative(&self) -> &Value {
        &self.0["can_destroy_blocks_in_creative"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool.rule
    pub fn rule(blocks: impl Into<Value>, speed: impl Into<Value>, correct_for_drops: impl Into<Value>) -> ToolRule {
        ToolRule(json!({ "blocks": blocks.into(), "speed": speed.into(), "correct_for_drops": correct_for_drops.into() }))
    }
}

/// `io.papermc.paper.datacomponent.item.Tool$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolBuilder(pub Map<String, Value>);

impl ToolBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Builder.defaultMiningSpeed
    pub fn default_mining_speed(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("default_mining_speed".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Builder.damagePerBlock
    pub fn damage_per_block(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("damage_per_block".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Builder.canDestroyBlocksInCreative
    pub fn can_destroy_blocks_in_creative(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("can_destroy_blocks_in_creative".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Builder.addRule
    pub fn add_rule(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "rules", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Builder.addRules
    pub fn add_rules(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "rules", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Builder.build
    pub fn build(self) -> Tool {
        Tool(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.Tool$Rule`.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolRule(pub Value);

impl ToolRule {
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Rule.blocks
    pub fn blocks(&self) -> &Value {
        &self.0["blocks"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Rule.speed
    pub fn speed(&self) -> &Value {
        &self.0["speed"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Tool$Rule.correctForDrops
    pub fn correct_for_drops(&self) -> &Value {
        &self.0["correct_for_drops"]
    }
}

/// `io.papermc.paper.datacomponent.item.TooltipDisplay`.
#[derive(Clone, Debug, PartialEq)]
pub struct TooltipDisplay(pub Value);

impl TooltipDisplay {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> TooltipDisplayBuilder {
        TooltipDisplayBuilder::default()
    }
}

impl TooltipDisplay {
    /// bukkit: io/papermc/paper/datacomponent/item/TooltipDisplay.tooltipDisplay
    pub fn tooltip_display() -> TooltipDisplayBuilder {
        TooltipDisplayBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/TooltipDisplay.hideTooltip
    pub fn hide_tooltip(&self) -> &Value {
        &self.0["hide_tooltip"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/TooltipDisplay.hiddenComponents
    pub fn hidden_components(&self) -> &Value {
        &self.0["hidden_components"]
    }
}

/// `io.papermc.paper.datacomponent.item.TooltipDisplay$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TooltipDisplayBuilder(pub Map<String, Value>);

impl TooltipDisplayBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/TooltipDisplay$Builder.hideTooltip
    pub fn hide_tooltip(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("hide_tooltip".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/TooltipDisplay$Builder.hiddenComponents
    pub fn hidden_components(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("hidden_components".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/TooltipDisplay$Builder.addHiddenComponents
    pub fn add_hidden_components(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "hidden_components", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/TooltipDisplay$Builder.build
    pub fn build(self) -> TooltipDisplay {
        TooltipDisplay(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.UseCooldown`.
#[derive(Clone, Debug, PartialEq)]
pub struct UseCooldown(pub Value);

impl UseCooldown {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> UseCooldownBuilder {
        UseCooldownBuilder::default()
    }
}

impl UseCooldown {
    /// bukkit: io/papermc/paper/datacomponent/item/UseCooldown.useCooldown
    pub fn use_cooldown(seconds: impl Into<Value>) -> UseCooldownBuilder {
        let mut built = Map::new();
        built.insert("seconds".to_owned(), seconds.into());
        UseCooldownBuilder(built)
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseCooldown.seconds
    pub fn seconds(&self) -> &Value {
        &self.0["seconds"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseCooldown.cooldownGroup
    pub fn cooldown_group(&self) -> &Value {
        &self.0["cooldown_group"]
    }
}

/// `io.papermc.paper.datacomponent.item.UseCooldown$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UseCooldownBuilder(pub Map<String, Value>);

impl UseCooldownBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseCooldown$Builder.cooldownGroup
    pub fn cooldown_group(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("cooldown_group".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseCooldown$Builder.build
    pub fn build(self) -> UseCooldown {
        UseCooldown(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.UseEffects`.
#[derive(Clone, Debug, PartialEq)]
pub struct UseEffects(pub Value);

impl UseEffects {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> UseEffectsBuilder {
        UseEffectsBuilder::default()
    }
}

impl UseEffects {
    /// bukkit: io/papermc/paper/datacomponent/item/UseEffects.useEffects
    pub fn use_effects() -> UseEffectsBuilder {
        UseEffectsBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseEffects.canSprint
    pub fn can_sprint(&self) -> &Value {
        &self.0["can_sprint"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseEffects.interactVibrations
    pub fn interact_vibrations(&self) -> &Value {
        &self.0["interact_vibrations"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseEffects.speedMultiplier
    pub fn speed_multiplier(&self) -> &Value {
        &self.0["speed_multiplier"]
    }
}

/// `io.papermc.paper.datacomponent.item.UseEffects$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UseEffectsBuilder(pub Map<String, Value>);

impl UseEffectsBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseEffects$Builder.canSprint
    pub fn can_sprint(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("can_sprint".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseEffects$Builder.interactVibrations
    pub fn interact_vibrations(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("interact_vibrations".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseEffects$Builder.speedMultiplier
    pub fn speed_multiplier(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("speed_multiplier".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseEffects$Builder.build
    pub fn build(self) -> UseEffects {
        UseEffects(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.UseRemainder`.
#[derive(Clone, Debug, PartialEq)]
pub struct UseRemainder(pub Value);

impl UseRemainder {
    /// bukkit: io/papermc/paper/datacomponent/item/UseRemainder.useRemainder
    pub fn use_remainder(value: impl Into<Value>) -> Self {
        Self(value.into())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/UseRemainder.transformInto
    pub fn transform_into(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.datacomponent.item.VillagerFood`.
#[derive(Clone, Debug, PartialEq)]
pub struct VillagerFood(pub Value);

impl VillagerFood {
    /// bukkit: io/papermc/paper/datacomponent/item/VillagerFood.villagerFood
    pub fn villager_food(nutrition: impl Into<Value>) -> Self {
        Self(json!({ "nutrition": nutrition.into(), }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/VillagerFood.nutrition
    pub fn nutrition(&self) -> &Value {
        &self.0["nutrition"]
    }
}

/// `io.papermc.paper.datacomponent.item.Weapon`.
#[derive(Clone, Debug, PartialEq)]
pub struct Weapon(pub Value);

impl Weapon {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> WeaponBuilder {
        WeaponBuilder::default()
    }
}

impl Weapon {
    /// bukkit: io/papermc/paper/datacomponent/item/Weapon.weapon
    pub fn weapon() -> WeaponBuilder {
        WeaponBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Weapon.itemDamagePerAttack
    pub fn item_damage_per_attack(&self) -> &Value {
        &self.0["item_damage_per_attack"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Weapon.disableBlockingForSeconds
    pub fn disable_blocking_for_seconds(&self) -> &Value {
        &self.0["disable_blocking_for_seconds"]
    }
}

/// `io.papermc.paper.datacomponent.item.Weapon$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeaponBuilder(pub Map<String, Value>);

impl WeaponBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Weapon$Builder.itemDamagePerAttack
    pub fn item_damage_per_attack(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("item_damage_per_attack".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Weapon$Builder.disableBlockingForSeconds
    pub fn disable_blocking_for_seconds(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("disable_blocking_for_seconds".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/Weapon$Builder.build
    pub fn build(self) -> Weapon {
        Weapon(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.WritableBookContent`.
#[derive(Clone, Debug, PartialEq)]
pub struct WritableBookContent(pub Value);

impl WritableBookContent {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> WritableBookContentBuilder {
        WritableBookContentBuilder::default()
    }
}

impl WritableBookContent {
    /// bukkit: io/papermc/paper/datacomponent/item/WritableBookContent.writeableBookContent
    pub fn writeable_book_content() -> WritableBookContentBuilder {
        WritableBookContentBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WritableBookContent.pages
    pub fn pages(&self) -> &Value {
        &self.0["pages"]
    }
}

/// `io.papermc.paper.datacomponent.item.WritableBookContent$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WritableBookContentBuilder(pub Map<String, Value>);

impl WritableBookContentBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WritableBookContent$Builder.addPage
    pub fn add_page(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "pages", json!({ "raw": value.into() }));
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WritableBookContent$Builder.addPages
    pub fn add_pages(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "pages", json!({ "raw": value }));
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WritableBookContent$Builder.addFilteredPage
    pub fn add_filtered_page(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "pages", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WritableBookContent$Builder.addFilteredPages
    pub fn add_filtered_pages(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "pages", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WritableBookContent$Builder.build
    pub fn build(self) -> WritableBookContent {
        WritableBookContent(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.WrittenBookContent`.
#[derive(Clone, Debug, PartialEq)]
pub struct WrittenBookContent(pub Value);

impl WrittenBookContent {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> WrittenBookContentBuilder {
        WrittenBookContentBuilder::default()
    }
}

impl WrittenBookContent {
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent.writtenBookContent
    pub fn written_book_content() -> WrittenBookContentBuilder {
        WrittenBookContentBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent.title
    pub fn title(&self) -> &Value {
        &self.0["title"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent.author
    pub fn author(&self) -> &Value {
        &self.0["author"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent.generation
    pub fn generation(&self) -> &Value {
        &self.0["generation"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent.pages
    pub fn pages(&self) -> &Value {
        &self.0["pages"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent.resolved
    pub fn resolved(&self) -> &Value {
        &self.0["resolved"]
    }
}

/// `io.papermc.paper.datacomponent.item.WrittenBookContent$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WrittenBookContentBuilder(pub Map<String, Value>);

impl WrittenBookContentBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.filteredTitle
    pub fn filtered_title(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("title".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.author
    pub fn author(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("author".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.generation
    pub fn generation(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("generation".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.resolved
    pub fn resolved(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("resolved".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.addPage
    pub fn add_page(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "pages", json!({ "raw": value.into() }));
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.addPages
    pub fn add_pages(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "pages", json!({ "raw": value }));
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.addFilteredPage
    pub fn add_filtered_page(mut self, value: impl Into<Value>) -> Self {
        push(&mut self.0, "pages", value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.addFilteredPages
    pub fn add_filtered_pages(mut self, value: impl Into<Value>) -> Self {
        if let Value::Array(rows) = value.into() {
            for value in rows {
                push(&mut self.0, "pages", value);
            }
        }
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.title
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.insert("title".to_owned(), json!({ "raw": title.into() }));
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/WrittenBookContent$Builder.build
    pub fn build(self) -> WrittenBookContent {
        WrittenBookContent(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.attribute.AttributeModifierDisplay`.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeModifierDisplay(pub Value);

impl AttributeModifierDisplay {
    /// bukkit: io/papermc/paper/datacomponent/item/attribute/AttributeModifierDisplay.reset
    pub fn reset() -> Self {
        Self(json!({ "type": "default", }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/attribute/AttributeModifierDisplay.hidden
    pub fn hidden() -> Self {
        Self(json!({ "type": "hidden", }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/attribute/AttributeModifierDisplay.override
    pub fn override_(text: impl Into<Value>) -> AttributeModifierDisplayOverrideText {
        AttributeModifierDisplayOverrideText(json!({ "type": "override", "value": text.into() }))
    }
}

/// `io.papermc.paper.datacomponent.item.attribute.AttributeModifierDisplay$OverrideText`.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeModifierDisplayOverrideText(pub Value);

impl AttributeModifierDisplayOverrideText {
    /// bukkit: io/papermc/paper/datacomponent/item/attribute/AttributeModifierDisplay$OverrideText.text
    pub fn text(&self) -> &Value {
        &self.0["value"]
    }
}

/// `io.papermc.paper.datacomponent.item.blocksattacks.DamageReduction`.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageReduction(pub Value);

impl DamageReduction {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> DamageReductionBuilder {
        DamageReductionBuilder::default()
    }
}

impl DamageReduction {
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction.damageReduction
    pub fn damage_reduction() -> DamageReductionBuilder {
        DamageReductionBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction.type
    pub fn type_(&self) -> &Value {
        &self.0["type"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction.base
    pub fn base(&self) -> &Value {
        &self.0["base"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction.factor
    pub fn factor(&self) -> &Value {
        &self.0["factor"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction.horizontalBlockingAngle
    pub fn horizontal_blocking_angle(&self) -> &Value {
        &self.0["horizontal_blocking_angle"]
    }
}

/// `io.papermc.paper.datacomponent.item.blocksattacks.DamageReduction$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageReductionBuilder(pub Map<String, Value>);

impl DamageReductionBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction$Builder.type
    pub fn type_(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("type".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction$Builder.base
    pub fn base(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("base".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction$Builder.factor
    pub fn factor(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("factor".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction$Builder.horizontalBlockingAngle
    pub fn horizontal_blocking_angle(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("horizontal_blocking_angle".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/DamageReduction$Builder.build
    pub fn build(self) -> DamageReduction {
        DamageReduction(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.blocksattacks.ItemDamageFunction`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemDamageFunction(pub Value);

impl ItemDamageFunction {
    /// A builder of this component, as Java's no-argument factory gives.
    pub fn builder() -> ItemDamageFunctionBuilder {
        ItemDamageFunctionBuilder::default()
    }
}

impl ItemDamageFunction {
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction.itemDamageFunction
    pub fn item_damage_function() -> ItemDamageFunctionBuilder {
        ItemDamageFunctionBuilder::default()
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction.threshold
    pub fn threshold(&self) -> &Value {
        &self.0["threshold"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction.base
    pub fn base(&self) -> &Value {
        &self.0["base"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction.factor
    pub fn factor(&self) -> &Value {
        &self.0["factor"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction.damageToApply
    pub fn damage_to_apply(&self, blocked: f32) -> i32 {
        let number = |key: &str| self.0[key].as_f64().unwrap_or(0.0) as f32;
        if blocked < number("threshold") { 0 } else { (number("base") + number("factor") * blocked).floor() as i32 }
    }
}

/// `io.papermc.paper.datacomponent.item.blocksattacks.ItemDamageFunction$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemDamageFunctionBuilder(pub Map<String, Value>);

impl ItemDamageFunctionBuilder {
    /// `BuildableDataComponent.toBuilder`: from the built value.
    pub fn from_built(built: &Value) -> Self {
        Self(built.as_object().cloned().unwrap_or_default())
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction$Builder.threshold
    pub fn threshold(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("threshold".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction$Builder.base
    pub fn base(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("base".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction$Builder.factor
    pub fn factor(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("factor".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/datacomponent/item/blocksattacks/ItemDamageFunction$Builder.build
    pub fn build(self) -> ItemDamageFunction {
        ItemDamageFunction(Value::Object(self.0))
    }
}

/// `io.papermc.paper.datacomponent.item.consumable.ConsumeEffect`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumeEffect(pub Value);

impl ConsumeEffect {
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.applyStatusEffects
    pub fn apply_status_effects(effects: impl Into<Value>, probability: f32) -> ConsumeEffectApplyStatusEffects {
        ConsumeEffectApplyStatusEffects(json!({ "type": "minecraft:apply_effects", "effects": effects.into(), "probability": probability }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.removeEffects
    pub fn remove_effects(effects: impl Into<Value>) -> ConsumeEffectRemoveStatusEffects {
        ConsumeEffectRemoveStatusEffects(json!({ "type": "minecraft:remove_effects", "effects": effects.into() }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.clearAllStatusEffects
    pub fn clear_all_status_effects() -> Self {
        Self(json!({ "type": "minecraft:clear_all_effects" }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.teleportRandomlyEffect
    pub fn teleport_randomly_effect(diameter: f32) -> ConsumeEffectTeleportRandomly {
        ConsumeEffectTeleportRandomly(json!({ "type": "minecraft:teleport_randomly", "diameter": diameter }))
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect.playSoundConsumeEffect
    pub fn play_sound_consume_effect(sound: impl Into<Value>) -> ConsumeEffectPlaySound {
        ConsumeEffectPlaySound(json!({ "type": "minecraft:play_sound", "sound": sound.into() }))
    }
}

/// `io.papermc.paper.datacomponent.item.consumable.ConsumeEffect$ApplyStatusEffects`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumeEffectApplyStatusEffects(pub Value);

impl ConsumeEffectApplyStatusEffects {
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect$ApplyStatusEffects.effects
    pub fn effects(&self) -> &Value {
        &self.0["effects"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect$ApplyStatusEffects.probability
    pub fn probability(&self) -> &Value {
        &self.0["probability"]
    }
}

/// `io.papermc.paper.datacomponent.item.consumable.ConsumeEffect$PlaySound`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumeEffectPlaySound(pub Value);

impl ConsumeEffectPlaySound {
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect$PlaySound.sound
    pub fn sound(&self) -> &Value {
        &self.0["sound"]
    }
}

/// `io.papermc.paper.datacomponent.item.consumable.ConsumeEffect$RemoveStatusEffects`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumeEffectRemoveStatusEffects(pub Value);

impl ConsumeEffectRemoveStatusEffects {
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect$RemoveStatusEffects.removeEffects
    pub fn remove_effects(&self) -> &Value {
        &self.0["effects"]
    }
}

/// `io.papermc.paper.datacomponent.item.consumable.ConsumeEffect$TeleportRandomly`.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsumeEffectTeleportRandomly(pub Value);

impl ConsumeEffectTeleportRandomly {
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect$TeleportRandomly.diameter
    pub fn diameter(&self) -> &Value {
        &self.0["diameter"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ConsumeEffect$TeleportRandomly.directionalParticles
    pub fn directional_particles(&self) -> &Value {
        &self.0["directional_particles"]
    }
}

fn push(fields: &mut Map<String, Value>, key: &str, value: Value) {
    let list = fields.entry(key.to_owned()).or_insert_with(|| json!([]));
    if let Some(list) = list.as_array_mut() {
        list.push(value);
    }
}
