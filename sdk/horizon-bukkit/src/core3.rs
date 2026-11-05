//! Block facts, potions and small value classes, by how often real plugins call them.
#![allow(missing_docs)]
use crate::core2::VoxelShape;
use crate::{Block, ItemStack, Material, PotionEffect};
use horizon_mod::{Value, query};
use serde_json::json;

impl Block {
    fn number(&self, key: &str) -> f64 {
        self.get_facts()[key].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/block/Block.getCollisionShape
    pub fn get_collision_shape(&self) -> VoxelShape {
        crate::fam_items::collision_of(self)
    }
    /// bukkit: org/bukkit/block/Block.getTemperature
    pub fn get_temperature(&self) -> f64 {
        self.number("temperature")
    }
    /// bukkit: org/bukkit/block/Block.getHumidity
    pub fn get_humidity(&self) -> f64 {
        self.number("downfall")
    }
    /// bukkit: org/bukkit/block/Block.getBlockPower
    pub fn get_block_power(&self) -> i32 {
        self.get_block_power_from(None)
    }
    /// bukkit: org/bukkit/block/Block.applyBoneMeal
    pub fn apply_bone_meal(&self) -> bool {
        query(json!({ "op": "bone_meal", "dimension": self.get_world().dimension(), "pos": [self.get_x(), self.get_y(), self.get_z()] })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/block/Block.getTranslationKey
    pub fn get_translation_key(&self) -> String {
        format!("block.minecraft.{}", self.get_type().key().trim_start_matches("minecraft:"))
    }
}

impl PotionEffect {
    /// bukkit: org/bukkit/potion/PotionEffect.hasIcon
    pub fn has_icon(&self) -> bool {
        self.icon
    }
}

/// `org.bukkit.potion.PotionData`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PotionData {
    kind: String,
    extended: bool,
    upgraded: bool,
}

impl PotionData {
    pub fn new(kind: &str, extended: bool, upgraded: bool) -> Self {
        Self { kind: kind.to_owned(), extended, upgraded }
    }
    /// A `potion_contents` id: `minecraft:long_swiftness`.
    pub fn from_id(id: &str) -> Self {
        let name = id.trim_start_matches("minecraft:");
        let kind = name.trim_start_matches("long_").trim_start_matches("strong_");
        Self::new(kind, name.starts_with("long_"), name.starts_with("strong_"))
    }
    /// bukkit: org/bukkit/potion/PotionData.getType
    pub fn get_type(&self) -> &str {
        &self.kind
    }
    /// bukkit: org/bukkit/potion/PotionData.isExtended
    pub fn is_extended(&self) -> bool {
        self.extended
    }
    /// bukkit: org/bukkit/potion/PotionData.isUpgraded
    pub fn is_upgraded(&self) -> bool {
        self.upgraded
    }
}

/// bukkit: org/bukkit/inventory/ItemType.createItemStack, org/bukkit/inventory/ItemType$Typed.createItemStack
pub fn create_item_stack(item: &str, amount: i32) -> ItemStack {
    ItemStack::new(Material::of(item), amount)
}

/// bukkit: org/bukkit/inventory/EquipmentSlotGroup.getByName
pub fn equipment_slot_group(name: &str) -> Option<&'static str> {
    ["any", "mainhand", "offhand", "hand", "feet", "legs", "chest", "head", "armor", "body", "saddle"].into_iter().find(|group| group.eq_ignore_ascii_case(name))
}

/// bukkit: org/bukkit/Particle.getKey
pub fn particle_key(name: &str) -> String {
    format!("minecraft:{}", name.to_ascii_lowercase())
}

pub use crate::worlds::unload_world;

/// bukkit: org/bukkit/inventory/meta/trim/TrimMaterial.create
///
/// A definition held in the component, for `ArmorMeta.set_trim_inline`.
pub fn create_trim_material(build: impl FnOnce(crate::registry_data::TrimMaterialRegistryEntryBuilder) -> crate::registry_data::TrimMaterialRegistryEntryBuilder) -> Value {
    build(Default::default()).build().into()
}

/// bukkit: org/bukkit/inventory/meta/trim/TrimPattern.create
///
/// As `create_trim_material`.
pub fn create_trim_pattern(build: impl FnOnce(crate::registry_data::TrimPatternRegistryEntryBuilder) -> crate::registry_data::TrimPatternRegistryEntryBuilder) -> Value {
    build(Default::default()).build().into()
}

/// bukkit: org/bukkit/inventory/meta/trim/ArmorTrim.getMaterial
///
/// The material of a `minecraft:trim` component.
pub fn trim_material(component: &Value) -> Option<&str> {
    component["material"].as_str()
}

/// bukkit: org/bukkit/inventory/meta/trim/ArmorTrim.getPattern
pub fn trim_pattern(component: &Value) -> Option<&str> {
    component["pattern"].as_str()
}

/// bukkit: org/bukkit/inventory/meta/trim/TrimMaterial.getKey, org/bukkit/inventory/meta/trim/TrimPattern.getKey
pub fn trim_key(id: &str) -> crate::persistence::NamespacedKey {
    crate::persistence::NamespacedKey::from_string(id).unwrap_or_else(|| crate::persistence::NamespacedKey::from_string("minecraft:empty").expect("a valid key"))
}
