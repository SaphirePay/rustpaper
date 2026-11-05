//! The everyday calls that used to be counted through a tag on the whole class: Material, Entity,
//! LivingEntity and the plugin manager.
#![allow(missing_docs)]
use crate::entity::Entity;
use crate::fam_items::{BlockType, ItemType};
use crate::util::{BoundingBox, RayTraceResult, Vector};
use crate::{Location, Material, Plugin, PotionEffect};
use horizon_mod::{Value, query, run_command};
use serde_json::json;

impl Material {
    fn item(&self) -> ItemType {
        ItemType::of(self.key())
    }
    fn block(&self) -> BlockType {
        BlockType::of(self.key())
    }
    fn facts(&self) -> Value {
        // Fixed for the game's run: asked once per material (crate::fam_items::fixed_answer).
        static FACTS: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, Value>>> = std::sync::OnceLock::new();
        crate::fam_items::fixed_answer(&FACTS, self.key(), || query(json!({ "op": "lead_material", "id": self.key() })))
    }
    /// bukkit: org/bukkit/Material.createBlockData
    ///
    /// The block's default state, as the game has it (`minecraft:water[level=0]`).
    pub fn create_block_data(&self) -> crate::block_data::BlockData {
        self.block().create_block_data(None)
    }
    /// bukkit: org/bukkit/Material.createBlockData
    ///
    /// The default state with the properties written (`[facing=east,half=top]`).
    pub fn create_block_data_with(&self, data: &str) -> crate::block_data::BlockData {
        self.block().create_block_data(Some(data))
    }
    /// bukkit: org/bukkit/Material.isBlock
    pub fn is_block(&self) -> bool {
        self.facts()["block"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/Material.isItem
    pub fn is_item(&self) -> bool {
        self.facts()["item"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/Material.getMaxDurability
    pub fn get_max_durability(&self) -> i16 {
        self.item().get_max_durability()
    }
    /// bukkit: org/bukkit/Material.isEdible
    pub fn is_edible(&self) -> bool {
        self.item().is_edible()
    }
    /// bukkit: org/bukkit/Material.isRecord
    pub fn is_record(&self) -> bool {
        self.item().is_record()
    }
    /// bukkit: org/bukkit/Material.isFuel
    pub fn is_fuel(&self) -> bool {
        self.item().is_fuel()
    }
    /// bukkit: org/bukkit/Material.isCompostable
    pub fn is_compostable(&self) -> bool {
        self.item().is_compostable()
    }
    /// bukkit: org/bukkit/Material.getCompostChance
    pub fn get_compost_chance(&self) -> f32 {
        self.item().get_compost_chance()
    }
    /// bukkit: org/bukkit/Material.getCreativeCategory
    pub fn get_creative_category(&self) -> &'static str {
        self.item().get_creative_category()
    }
    /// bukkit: org/bukkit/Material.getItemRarity
    pub fn get_item_rarity(&self) -> String {
        self.item().get_item_rarity()
    }
    /// bukkit: org/bukkit/Material.getCraftingRemainingItem
    pub fn get_crafting_remaining_item(&self) -> Option<Self> {
        self.item().get_crafting_remaining_item().map(|item| item.as_material())
    }
    /// bukkit: org/bukkit/Material.isSolid
    pub fn is_solid(&self) -> bool {
        self.is_block() && self.block().is_solid()
    }
    /// bukkit: org/bukkit/Material.isFlammable
    pub fn is_flammable(&self) -> bool {
        self.is_block() && self.block().is_flammable()
    }
    /// bukkit: org/bukkit/Material.isBurnable
    pub fn is_burnable(&self) -> bool {
        self.is_block() && self.block().is_burnable()
    }
    /// bukkit: org/bukkit/Material.isOccluding
    pub fn is_occluding(&self) -> bool {
        self.is_block() && self.block().is_occluding()
    }
    /// bukkit: org/bukkit/Material.hasGravity
    pub fn has_gravity(&self) -> bool {
        self.is_block() && self.block().has_gravity()
    }
    /// bukkit: org/bukkit/Material.isCollidable
    pub fn is_collidable(&self) -> bool {
        self.is_block() && self.block().has_collision()
    }
    /// bukkit: org/bukkit/Material.getHardness
    pub fn get_hardness(&self) -> f32 {
        self.block().get_hardness()
    }
    /// bukkit: org/bukkit/Material.getBlastResistance
    pub fn get_blast_resistance(&self) -> f32 {
        self.block().get_blast_resistance()
    }
    /// bukkit: org/bukkit/Material.getSlipperiness
    pub fn get_slipperiness(&self) -> f32 {
        self.block().get_slipperiness()
    }
    /// bukkit: org/bukkit/Material.isEmpty, org/bukkit/Material.isAir
    pub fn is_empty(&self) -> bool {
        self.block().is_air()
    }
    /// bukkit: org/bukkit/Material.asItemType
    pub fn as_item_type(&self) -> Option<ItemType> {
        self.is_item().then(|| self.item())
    }
    /// bukkit: org/bukkit/Material.asBlockType
    pub fn as_block_type(&self) -> Option<BlockType> {
        self.is_block().then(|| self.block())
    }
    /// bukkit: org/bukkit/Material.getKey
    pub fn get_key(&self) -> &str {
        self.key()
    }
    /// bukkit: org/bukkit/Material.translationKey, org/bukkit/Material.getTranslationKey
    pub fn translation_key(&self) -> String {
        if self.is_block() { self.block().get_translation_key() } else { self.item().get_translation_key() }
    }
    /// bukkit: org/bukkit/Material.getBlockTranslationKey
    pub fn get_block_translation_key(&self) -> Option<String> {
        self.is_block().then(|| self.block().get_translation_key())
    }
    /// bukkit: org/bukkit/Material.getItemTranslationKey
    pub fn get_item_translation_key(&self) -> Option<String> {
        self.is_item().then(|| self.item().get_translation_key())
    }
    /// bukkit: org/bukkit/Material.hasDefaultData
    pub fn has_default_data(&self, component: &str) -> bool {
        self.item().has_default_data(component)
    }
    /// bukkit: org/bukkit/Material.getDefaultData
    pub fn get_default_data(&self, component: &str) -> Option<Value> {
        self.item().get_default_data(component)
    }
    /// bukkit: org/bukkit/Material.getDefaultDataTypes
    pub fn get_default_data_types(&self) -> Vec<String> {
        self.item().get_default_data_types()
    }
}

impl Entity {
    fn kind_facts(&self) -> Value {
        query(json!({ "op": "mob_type", "id": self.get_type() }))
    }
    fn saved_bool(&self, key: &str) -> bool {
        let value = self.data_at(key);
        value.as_bool().unwrap_or(value.as_i64().unwrap_or(0) != 0)
    }
    /// bukkit: org/bukkit/entity/Entity.getHeight
    pub fn get_height(&self) -> f64 {
        crate::fam_entity::live_size(self).map_or_else(|| self.kind_facts()["height"].as_f64().unwrap_or(0.0), |size| size.1)
    }
    /// bukkit: org/bukkit/entity/Entity.getWidth
    pub fn get_width(&self) -> f64 {
        crate::fam_entity::live_size(self).map_or_else(|| self.kind_facts()["width"].as_f64().unwrap_or(0.0), |size| size.0)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getEyeHeight
    pub fn get_eye_height(&self) -> f64 {
        crate::fam_entity::live_size(self).map_or_else(|| self.kind_facts()["eye_height"].as_f64().unwrap_or(0.0), |size| size.2)
    }
    /// bukkit: org/bukkit/entity/Entity.getBoundingBox
    pub fn get_bounding_box(&self) -> Option<BoundingBox> {
        let at = self.get_location()?;
        let half = self.get_width() / 2.0;
        Some(BoundingBox::new(at.get_x() - half, at.get_y(), at.get_z() - half, at.get_x() + half, at.get_y() + self.get_height(), at.get_z() + half))
    }
    /// bukkit: org/bukkit/entity/Entity.isInWater
    pub fn is_in_water(&self) -> bool {
        self.get_location().is_some_and(|at| at.get_block().get_type().key() == "minecraft:water")
    }
    /// bukkit: org/bukkit/entity/Entity.setVisualFire
    pub fn set_visual_fire(&self, on: bool) -> bool {
        self.merge(&format!("{{HasVisualFire:{on}}}"))
    }
    /// bukkit: org/bukkit/entity/Entity.isVisualFire
    pub fn is_visual_fire(&self) -> bool {
        self.saved_bool("HasVisualFire")
    }
    /// bukkit: org/bukkit/entity/Entity.getMaxFreezeTicks
    pub fn get_max_freeze_ticks(&self) -> i32 {
        140
    }
    /// bukkit: org/bukkit/entity/Entity.isFrozen
    pub fn is_frozen(&self) -> bool {
        self.data_at("TicksFrozen").as_i64().unwrap_or(0) >= 140
    }
    /// bukkit: org/bukkit/entity/Entity.getPassenger
    pub fn get_passenger(&self) -> Option<Value> {
        self.get_passengers().into_iter().next()
    }
    /// bukkit: org/bukkit/entity/Entity.setPassenger
    pub fn set_passenger(&self, rider: &Self) -> bool {
        self.add_passenger(rider);
        true
    }
    /// bukkit: org/bukkit/entity/Entity.removePassenger
    pub fn remove_passenger(&self, rider: &Self) -> bool {
        rider.leave_vehicle();
        true
    }
    /// bukkit: org/bukkit/entity/Entity.getFacing
    pub fn get_facing(&self) -> Option<&'static str> {
        let yaw = self.get_location()?.get_yaw().rem_euclid(360.0);
        Some(["SOUTH", "WEST", "NORTH", "EAST"][(((yaw + 45.0) / 90.0) as usize) % 4])
    }
    /// bukkit: org/bukkit/entity/LivingEntity.isLeashed
    pub fn is_leashed(&self) -> bool {
        !self.data_at("leash").is_null()
    }
    /// bukkit: org/bukkit/entity/LivingEntity.clearActivePotionEffects
    pub fn clear_active_potion_effects(&self) -> bool {
        run_command(&format!("effect clear {}", self.get_unique_id()));
        true
    }
    /// bukkit: org/bukkit/entity/LivingEntity.addPotionEffects
    pub fn add_potion_effects(&self, effects: &[PotionEffect]) -> bool {
        effects.iter().for_each(|effect| self.add_potion_effect(effect));
        true
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getLastDamage
    pub fn get_last_damage(&self) -> f64 {
        self.last_hurt()
    }
    /// bukkit: org/bukkit/entity/LivingEntity.rayTraceBlocks
    ///
    /// From its eyes, along where it looks. `fluids` is a `FluidCollisionMode` name.
    pub fn ray_trace_blocks_with(&self, distance: f64, fluids: &str) -> Option<RayTraceResult> {
        let at = self.get_location()?;
        let (yaw, pitch) = (f64::from(at.get_yaw()).to_radians(), f64::from(at.get_pitch()).to_radians());
        let direction = Vector::new(-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos());
        let eyes = Location::new(at.get_world().clone(), at.get_x(), at.get_y() + self.get_eye_height(), at.get_z(), 0.0, 0.0);
        at.get_world().ray_trace(&eyes, &direction, distance, fluids, false, 0.0, |_| false)
    }
    /// `ray_trace_blocks_with`, past fluids.
    pub fn ray_trace_blocks(&self, distance: f64) -> Option<RayTraceResult> {
        self.ray_trace_blocks_with(distance, "NEVER")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getTargetBlockExact
    pub fn get_target_block_exact(&self, distance: i32, fluids: &str) -> Option<crate::Block> {
        self.ray_trace_blocks_with(f64::from(distance), fluids)?.get_hit_block().cloned()
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getTargetBlock
    ///
    /// The last of `get_line_of_sight`, as `getLineOfSight(transparent, range, 1)` keeps.
    pub fn get_target_block(&self, transparent: &[crate::Material], range: i32) -> Option<crate::Block> {
        self.get_line_of_sight(transparent, range).pop()
    }
}

// ---- The plugin manager ----

/// Whether a mod of this id is loaded: `fam_core::PluginManager` has the plugins themselves.
pub fn is_plugin_enabled(name: &str) -> bool {
    query(json!({ "op": "lead_mods" })).as_array().is_some_and(|mods| mods.iter().any(|id| id.as_str().is_some_and(|id| id.eq_ignore_ascii_case(name))))
}

/// The ids of the loaded mods.
pub fn get_plugins() -> Vec<String> {
    query(json!({ "op": "lead_mods" })).as_array().into_iter().flatten().filter_map(|id| id.as_str().map(str::to_owned)).collect()
}

impl Plugin {
    /// bukkit: org/bukkit/plugin/PluginManager.callEvent
    ///
    /// A plugin's own event, by the `HOOK` its `Event` type names. `true` if a listener refused it.
    pub fn call_event(&self, hook: &str, payload: &Value) -> bool {
        query(json!({ "op": "lead_fire", "event": hook, "payload": payload })).as_bool().unwrap_or(false)
    }
}
