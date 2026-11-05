//! The entity calls that aren't one path: poses, block states, bits of a number, owners.
use crate::block_data::BlockData;
use crate::data::{flag, quoted, snbt};
use crate::entities::{AbstractCubeMob, AbstractHorse, ArmorStand, BlockDisplay, Enderman, EvokerFangs, FallingBlock, Horse, Mob, Painting, Phantom, Rabbit, Sheep, Shulker, Slime, Tameable, TextDisplay, TropicalFish, Villager, Wolf, Zombie};
use crate::entity::Entity;
use crate::util::EulerAngle;
use crate::{Color, Component, OfflinePlayer, Player, PotionEffect, adventure, get_offline_player};
use horizon_mod::{Value, run_command};
use serde_json::json;

pub(crate) fn block_state(value: &Value) -> Option<BlockData> {
    let name = value["Name"].as_str()?;
    let properties: Vec<String> = value["Properties"].as_object().into_iter().flatten().filter_map(|(key, value)| Some(format!("{key}={}", value.as_str()?))).collect();
    Some(BlockData::parse(&if properties.is_empty() { name.to_owned() } else { format!("{name}[{}]", properties.join(",")) }))
}

pub(crate) fn block_state_snbt(data: &BlockData) -> String {
    let text = data.get_as_string();
    let (name, rest) = text.split_once('[').unwrap_or((&text, ""));
    let properties: Vec<String> = rest.trim_end_matches(']').split(',').filter_map(|pair| pair.split_once('=')).map(|(key, value)| format!("{key}:{}", quoted(value))).collect();
    format!("{{Name:{},Properties:{{{}}}}}", quoted(name), properties.join(","))
}

macro_rules! poses {
    ($(($getter:ident, $setter:ident, $key:literal, $java_get:literal, $java_set:literal)),* $(,)?) => {
        impl ArmorStand {
            $(
                pub fn $getter(&self) -> EulerAngle {
                    let pose = self.data_at(concat!("Pose.", $key));
                    let angle = |i: usize| pose[i].as_f64().unwrap_or(0.0).to_radians();
                    EulerAngle::new(angle(0), angle(1), angle(2))
                }
                pub fn $setter(&self, angle: &EulerAngle) -> bool {
                    self.merge(&format!(concat!("{{Pose:{{", $key, ":[{:?}f,{:?}f,{:?}f]}}}}"), angle.x.to_degrees() as f32, angle.y.to_degrees() as f32, angle.z.to_degrees() as f32))
                }
            )*
        }
    };
}

/// bukkit: org/bukkit/entity/ArmorStand.getBodyPose, org/bukkit/entity/ArmorStand.setBodyPose
/// bukkit: org/bukkit/entity/ArmorStand.getHeadPose, org/bukkit/entity/ArmorStand.setHeadPose
/// bukkit: org/bukkit/entity/ArmorStand.getLeftArmPose, org/bukkit/entity/ArmorStand.setLeftArmPose
/// bukkit: org/bukkit/entity/ArmorStand.getRightArmPose, org/bukkit/entity/ArmorStand.setRightArmPose
/// bukkit: org/bukkit/entity/ArmorStand.getLeftLegPose, org/bukkit/entity/ArmorStand.setLeftLegPose
/// bukkit: org/bukkit/entity/ArmorStand.getRightLegPose, org/bukkit/entity/ArmorStand.setRightLegPose
#[rustfmt::skip]
const _: () = ();
poses!(
    (get_body_pose, set_body_pose, "Body", "getBodyPose", "setBodyPose"),
    (get_head_pose, set_head_pose, "Head", "getHeadPose", "setHeadPose"),
    (get_left_arm_pose, set_left_arm_pose, "LeftArm", "getLeftArmPose", "setLeftArmPose"),
    (get_right_arm_pose, set_right_arm_pose, "RightArm", "getRightArmPose", "setRightArmPose"),
    (get_left_leg_pose, set_left_leg_pose, "LeftLeg", "getLeftLegPose", "setLeftLegPose"),
    (get_right_leg_pose, set_right_leg_pose, "RightLeg", "getRightLegPose", "setRightLegPose"),
);

impl ArmorStand {
    /// bukkit: org/bukkit/entity/ArmorStand.isSlotDisabled
    ///
    /// `DisabledSlots` holds three bits per slot: this asks the first, which stops taking from it.
    pub fn is_slot_disabled(&self, slot: u32) -> bool {
        self.data_at("DisabledSlots").as_i64().unwrap_or(0) & (1 << slot) != 0
    }
}

impl TextDisplay {
    /// bukkit: org/bukkit/entity/TextDisplay.text
    pub fn text(&self) -> Component {
        Component(self.data_at("text"))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.getText
    pub fn get_text(&self) -> String {
        self.text().to_legacy()
    }
    /// bukkit: org/bukkit/entity/TextDisplay.setText
    pub fn set_text(&self, text: &str) -> bool {
        self.merge(&format!("{{text:{}}}", snbt(&adventure::legacy(text).0)))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.getBackgroundColor
    pub fn get_background_color(&self) -> Option<Color> {
        let argb = self.data_at("background").as_i64()?;
        Some(Color::from_rgb(((argb >> 16) & 255) as u8, ((argb >> 8) & 255) as u8, (argb & 255) as u8))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.setBackgroundColor
    pub fn set_background_color(&self, color: &Color) -> bool {
        self.merge(&format!("{{background:{}}}", color.as_argb() as i32))
    }
}

impl BlockDisplay {
    /// bukkit: org/bukkit/entity/BlockDisplay.getBlock
    pub fn get_block(&self) -> Option<BlockData> {
        block_state(&self.data_at("block_state"))
    }
    /// bukkit: org/bukkit/entity/BlockDisplay.setBlock
    pub fn set_block(&self, data: &BlockData) -> bool {
        self.merge(&format!("{{block_state:{}}}", block_state_snbt(data)))
    }
}

impl FallingBlock {
    /// bukkit: org/bukkit/entity/FallingBlock.getBlockData
    pub fn get_block_data(&self) -> Option<BlockData> {
        block_state(&self.data_at("BlockState"))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.setBlockData
    pub fn set_block_data(&self, data: &BlockData) -> bool {
        self.merge(&format!("{{BlockState:{}}}", block_state_snbt(data)))
    }
}

impl Enderman {
    /// bukkit: org/bukkit/entity/Enderman.getCarriedBlock
    pub fn get_carried_block(&self) -> Option<BlockData> {
        block_state(&self.data_at("carriedBlockState"))
    }
    /// bukkit: org/bukkit/entity/Enderman.setCarriedBlock
    pub fn set_carried_block(&self, data: &BlockData) -> bool {
        self.merge(&format!("{{carriedBlockState:{}}}", block_state_snbt(data)))
    }
}

impl Painting {
    /// bukkit: org/bukkit/entity/Painting.getArt
    pub fn get_art(&self) -> Option<String> {
        self.data_at("variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Painting.setArt
    pub fn set_art(&self, art: &str) -> bool {
        self.merge(&format!("{{variant:{}}}", quoted(art)))
    }
}

macro_rules! sized {
    ($($name:ty, $key:literal, $offset:literal, $get:ident, $set:ident);* $(;)?) => {
        $(impl $name {
            pub fn $get(&self) -> i32 {
                self.data_at($key).as_i64().unwrap_or(0) as i32 + $offset
            }
            pub fn $set(&self, size: i32) -> bool {
                self.merge(&format!(concat!("{{", $key, ":{}}}"), size - $offset))
            }
        })*
    };
}

/// bukkit: org/bukkit/entity/Slime.setSize, org/bukkit/entity/MagmaCube.setSize, org/bukkit/entity/AbstractCubeMob.getSize, org/bukkit/entity/AbstractCubeMob.setSize
/// bukkit: org/bukkit/entity/Phantom.getSize, org/bukkit/entity/Phantom.setSize
#[rustfmt::skip]
const _: () = ();
sized!(
    Slime, "Size", 1, get_size, set_size;
    AbstractCubeMob, "Size", 1, get_size, set_size;
    Phantom, "size", 0, get_size, set_size;
);

const HORSE_COLORS: [&str; 7] = ["WHITE", "CREAMY", "CHESTNUT", "BROWN", "BLACK", "GRAY", "DARK_BROWN"];
const HORSE_STYLES: [&str; 5] = ["NONE", "WHITE", "WHITEFIELD", "WHITE_DOTS", "BLACK_DOTS"];

impl Horse {
    fn bits(&self) -> i64 {
        self.data_at("Variant").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/entity/Horse.getColor
    pub fn get_color(&self) -> &'static str {
        HORSE_COLORS[(self.bits() & 255) as usize % 7]
    }
    /// bukkit: org/bukkit/entity/Horse.setColor
    pub fn set_color(&self, color: &str) -> bool {
        let index = HORSE_COLORS.iter().position(|known| *known == color).unwrap_or(0) as i64;
        self.merge(&format!("{{Variant:{}}}", (self.bits() & !255) | index))
    }
    /// bukkit: org/bukkit/entity/Horse.getStyle
    pub fn get_style(&self) -> &'static str {
        HORSE_STYLES[((self.bits() >> 8) & 255) as usize % 5]
    }
    /// bukkit: org/bukkit/entity/Horse.setStyle
    pub fn set_style(&self, style: &str) -> bool {
        let index = HORSE_STYLES.iter().position(|known| *known == style).unwrap_or(0) as i64;
        self.merge(&format!("{{Variant:{}}}", (self.bits() & 255) | (index << 8)))
    }
}

impl AbstractHorse {
    /// bukkit: org/bukkit/entity/AbstractHorse.getJumpStrength
    pub fn get_jump_strength(&self) -> f64 {
        self.0.get_attribute("minecraft:jump_strength").get_value()
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.setJumpStrength
    pub fn set_jump_strength(&self, strength: f64) {
        self.0.get_attribute("minecraft:jump_strength").set_base_value(strength);
    }
}

impl Rabbit {
    /// bukkit: org/bukkit/entity/Rabbit.getRabbitType
    pub fn get_rabbit_type(&self) -> &'static str {
        match self.data_at("RabbitType").as_i64() {
            Some(1) => "WHITE",
            Some(2) => "BLACK",
            Some(3) => "BLACK_AND_WHITE",
            Some(4) => "GOLD",
            Some(5) => "SALT_AND_PEPPER",
            Some(99) => "THE_KILLER_BUNNY",
            _ => "BROWN",
        }
    }
    /// bukkit: org/bukkit/entity/Rabbit.setRabbitType
    pub fn set_rabbit_type(&self, kind: &str) -> bool {
        let number = match kind {
            "WHITE" => 1,
            "BLACK" => 2,
            "BLACK_AND_WHITE" => 3,
            "GOLD" => 4,
            "SALT_AND_PEPPER" => 5,
            "THE_KILLER_BUNNY" => 99,
            _ => 0,
        };
        self.merge(&format!("{{RabbitType:{number}}}"))
    }
}

impl TropicalFish {
    fn bits(&self) -> i64 {
        self.data_at("Variant").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/entity/TropicalFish.getBodyColor
    pub fn get_body_color(&self) -> &'static str {
        crate::enums::dye_color::values().get(((self.bits() >> 16) & 255) as usize).copied().unwrap_or("WHITE")
    }
    /// bukkit: org/bukkit/entity/TropicalFish.getPatternColor
    pub fn get_pattern_color(&self) -> &'static str {
        crate::enums::dye_color::values().get(((self.bits() >> 24) & 255) as usize).copied().unwrap_or("WHITE")
    }
    /// bukkit: org/bukkit/entity/TropicalFish.setBodyColor
    pub fn set_body_color(&self, color: &str) -> bool {
        let index = crate::enums::dye_color::values().iter().position(|known| *known == color).unwrap_or(0) as i64;
        self.merge(&format!("{{Variant:{}}}", (self.bits() & !(255 << 16)) | (index << 16)))
    }
    /// bukkit: org/bukkit/entity/TropicalFish.setPatternColor
    pub fn set_pattern_color(&self, color: &str) -> bool {
        let index = crate::enums::dye_color::values().iter().position(|known| *known == color).unwrap_or(0) as i64;
        self.merge(&format!("{{Variant:{}}}", (self.bits() & !(255 << 24)) | (index << 24)))
    }
}

impl Sheep {
    /// bukkit: org/bukkit/material/Colorable.getColor
    pub fn get_color(&self) -> &'static str {
        crate::enums::dye_color::values().get(self.data_at("Color").as_u64().unwrap_or(0) as usize).copied().unwrap_or("WHITE")
    }
    /// bukkit: org/bukkit/material/Colorable.setColor
    pub fn set_color(&self, color: &str) -> bool {
        let index = crate::enums::dye_color::values().iter().position(|known| *known == color).unwrap_or(0);
        self.merge(&format!("{{Color:{index}b}}"))
    }
}

impl Shulker {
    /// bukkit: org/bukkit/entity/Shulker.getAttachedFace
    pub fn get_attached_face(&self) -> &'static str {
        match self.data_at("AttachFace").as_i64() {
            Some(1) => "UP",
            Some(2) => "NORTH",
            Some(3) => "SOUTH",
            Some(4) => "WEST",
            Some(5) => "EAST",
            _ => "DOWN",
        }
    }
    /// bukkit: org/bukkit/entity/Shulker.setAttachedFace
    pub fn set_attached_face(&self, face: &str) -> bool {
        let number = ["DOWN", "UP", "NORTH", "SOUTH", "WEST", "EAST"].iter().position(|known| *known == face).unwrap_or(0);
        self.merge(&format!("{{AttachFace:{number}b}}"))
    }
}

impl Tameable {
    /// bukkit: org/bukkit/entity/Tameable.getOwner
    pub fn get_owner(&self) -> Option<OfflinePlayer> {
        let id = crate::data::uuid(&self.data_at("Owner"))?;
        crate::get_online_players().iter().find(|player| player.get_unique_id() == id).map(|player| get_offline_player(player.get_name()))
    }
    /// bukkit: org/bukkit/entity/Tameable.setOwner
    pub fn set_owner(&self, owner: &Player) -> bool {
        self.merge(&format!("{{Owner:{},Tame:true}}", crate::data::uuid_snbt(&owner.get_unique_id()).unwrap_or_default()))
    }
}

impl Wolf {
    /// bukkit: org/bukkit/entity/Wolf.isAngry
    pub fn is_angry(&self) -> bool {
        self.data_at("anger_end_time").as_i64().unwrap_or(0) > self.0.get_world().get_full_time()
    }
    /// bukkit: org/bukkit/entity/Wolf.setAngry
    pub fn set_angry(&self, angry: bool) -> bool {
        let until = if angry { self.0.get_world().get_full_time() + 400 } else { 0 };
        self.merge(&format!("{{anger_end_time:{until}L}}"))
    }
}

impl Zombie {
    /// bukkit: org/bukkit/entity/Zombie.isConverting
    pub fn is_converting(&self) -> bool {
        crate::fam_entity::zombie_converting(&self.0)
    }
}

impl Villager {
    /// bukkit: org/bukkit/entity/AbstractVillager.getRecipes, org/bukkit/entity/Villager.getRecipes
    pub fn get_recipes(&self) -> Vec<Value> {
        self.data_at("Offers.Recipes").as_array().cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/entity/AbstractVillager.getRecipeCount
    pub fn get_recipe_count(&self) -> usize {
        self.get_recipes().len()
    }
    /// bukkit: org/bukkit/entity/Villager.zombify
    pub fn zombify(&self) {
        run_command(&format!("execute at {0} run summon minecraft:zombie_villager ~ ~ ~", self.get_unique_id()));
        self.remove();
    }
}

impl Mob {
    /// bukkit: org/bukkit/entity/Mob.getTarget
    pub fn get_target(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("angry_at"))
    }
    /// bukkit: org/bukkit/entity/Mob.setTarget
    pub fn set_target(&self, target: &Entity) -> bool {
        self.merge(&format!("{{angry_at:{}}}", crate::data::uuid_snbt(&target.get_unique_id()).unwrap_or_default()))
    }
    /// bukkit: org/bukkit/entity/Mob.isAware
    pub fn is_aware(&self) -> bool {
        !flag(&self.data_at("NoAI"))
    }
    /// bukkit: org/bukkit/entity/Mob.setAware
    pub fn set_aware(&self, aware: bool) -> bool {
        self.merge(&format!("{{NoAI:{}}}", !aware))
    }
}

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getActivePotionEffects
    pub fn get_active_potion_effects(&self) -> Vec<PotionEffect> {
        self.data_at("active_effects").as_array().into_iter().flatten().filter_map(PotionEffect::from_saved).collect()
    }
    /// bukkit: org/bukkit/entity/LivingEntity.addPotionEffect
    pub fn add_potion_effect(&self, effect: &PotionEffect) {
        effect.apply(self);
    }
    /// bukkit: org/bukkit/entity/LivingEntity.removePotionEffect
    pub fn remove_potion_effect(&self, effect: &str) {
        run_command(&format!("effect clear {} {effect}", self.get_unique_id()));
    }
    /// bukkit: org/bukkit/entity/LivingEntity.hasPotionEffect
    pub fn has_potion_effect(&self, effect: &str) -> bool {
        self.get_active_potion_effects().iter().any(|known| known.is(effect))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getPotionEffect
    pub fn get_potion_effect(&self, effect: &str) -> Option<PotionEffect> {
        self.get_active_potion_effects().into_iter().find(|known| known.is(effect))
    }
    /// bukkit: org/bukkit/entity/Entity.addPassenger
    pub fn add_passenger(&self, rider: &Self) {
        run_command(&format!("ride {} mount {}", rider.get_unique_id(), self.get_unique_id()));
    }
    /// bukkit: org/bukkit/entity/Entity.eject, org/bukkit/entity/Entity.leaveVehicle
    pub fn leave_vehicle(&self) {
        run_command(&format!("ride {} dismount", self.get_unique_id()));
    }
    /// bukkit: org/bukkit/entity/Entity.getPassengers
    pub fn get_passengers(&self) -> Vec<Value> {
        self.data_at("Passengers").as_array().cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/entity/Entity.isEmpty
    pub fn is_empty(&self) -> bool {
        self.get_passengers().is_empty()
    }
}

/// Keeps `json!` for the odd macro use above honest under `deny(unused)`.
pub fn empty_compound() -> Value {
    json!({})
}
