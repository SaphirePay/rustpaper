//! Written by plugin-tools/tools/sdk_entities.py: do not edit.
//!
//! Each call reads or changes a path in the data the game saves for the entity: see that tool's table.
#![allow(missing_docs)]
use crate::entity::{Entity, EntityClass};

impl Entity {
    /// bukkit: org/bukkit/entity/Entity.getFireTicks
    pub fn get_fire_ticks(&self) -> i32 {
        self.data_at("Fire").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Entity.setFireTicks
    pub fn set_fire_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Fire", &format!("{}", format!("{value}s"))))
    }
    /// bukkit: org/bukkit/entity/Entity.getFreezeTicks
    pub fn get_freeze_ticks(&self) -> i32 {
        self.data_at("TicksFrozen").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Entity.setFreezeTicks
    pub fn set_freeze_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("TicksFrozen", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Entity.isGlowing
    pub fn is_glowing(&self) -> bool {
        crate::data::flag(&self.data_at("Glowing"))
    }
    /// bukkit: org/bukkit/entity/Entity.setGlowing
    pub fn set_glowing(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Glowing", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Entity.isInvulnerable
    pub fn is_invulnerable(&self) -> bool {
        crate::data::flag(&self.data_at("Invulnerable"))
    }
    /// bukkit: org/bukkit/entity/Entity.setInvulnerable
    pub fn set_invulnerable(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Invulnerable", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Entity.isSilent
    pub fn is_silent(&self) -> bool {
        crate::data::flag(&self.data_at("Silent"))
    }
    /// bukkit: org/bukkit/entity/Entity.setSilent
    pub fn set_silent(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Silent", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Entity.hasGravity
    pub fn has_gravity(&self) -> bool {
        !crate::data::flag(&self.data_at("NoGravity"))
    }
    /// bukkit: org/bukkit/entity/Entity.setGravity
    pub fn set_gravity(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("NoGravity", &format!("{}", !value)))
    }
    /// bukkit: org/bukkit/entity/Entity.isCustomNameVisible
    pub fn is_custom_name_visible(&self) -> bool {
        crate::data::flag(&self.data_at("CustomNameVisible"))
    }
    /// bukkit: org/bukkit/entity/Entity.setCustomNameVisible
    pub fn set_custom_name_visible(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("CustomNameVisible", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Entity.getPortalCooldown
    pub fn get_portal_cooldown(&self) -> i32 {
        self.data_at("PortalCooldown").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Entity.setPortalCooldown
    pub fn set_portal_cooldown(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("PortalCooldown", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Entity.getFallDistance
    pub fn get_fall_distance(&self) -> f64 {
        self.data_at("fall_distance").as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/entity/Entity.setFallDistance
    pub fn set_fall_distance(&self, value: f64) -> bool {
        self.merge(&crate::data::patch("fall_distance", &format!("{}", format!("{value:?}d"))))
    }
    /// bukkit: org/bukkit/entity/Entity.isPersistent
    pub fn is_persistent(&self) -> bool {
        crate::data::flag(&self.data_at("PersistenceRequired"))
    }
    /// bukkit: org/bukkit/entity/Entity.setPersistent
    pub fn set_persistent(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("PersistenceRequired", &format!("{}", value)))
    }
}

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getRemainingAir
    pub fn get_remaining_air(&self) -> i32 {
        self.data_at("Air").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setRemainingAir
    pub fn set_remaining_air(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Air", &format!("{}", format!("{value}s"))))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.hasAI
    pub fn has_ai(&self) -> bool {
        !crate::data::flag(&self.data_at("NoAI"))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setAI
    pub fn set_ai(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("NoAI", &format!("{}", !value)))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getRemoveWhenFarAway
    pub fn get_remove_when_far_away(&self) -> bool {
        !crate::data::flag(&self.data_at("PersistenceRequired"))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setRemoveWhenFarAway
    pub fn set_remove_when_far_away(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("PersistenceRequired", &format!("{}", !value)))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getCanPickupItems
    pub fn get_can_pickup_items(&self) -> bool {
        crate::data::flag(&self.data_at("CanPickUpLoot"))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setCanPickupItems
    pub fn set_can_pickup_items(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("CanPickUpLoot", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.isGliding
    pub fn is_gliding(&self) -> bool {
        crate::data::flag(&self.data_at("FallFlying"))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setGliding
    pub fn set_gliding(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("FallFlying", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getAbsorptionAmount
    pub fn get_absorption_amount(&self) -> f32 {
        self.data_at("AbsorptionAmount").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setAbsorptionAmount
    pub fn set_absorption_amount(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("AbsorptionAmount", &format!("{}", format!("{value:?}f"))))
    }
}

impl Entity {
    /// bukkit: org/bukkit/entity/Mob.isLeftHanded
    pub fn is_left_handed(&self) -> bool {
        crate::data::flag(&self.data_at("LeftHanded"))
    }
    /// bukkit: org/bukkit/entity/Mob.setLeftHanded
    pub fn set_left_handed(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("LeftHanded", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.AbstractArrow`.
#[derive(Clone, Debug, PartialEq)]
pub struct AbstractArrow(pub Entity);

impl std::ops::Deref for AbstractArrow {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for AbstractArrow {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl AbstractArrow {
    /// bukkit: org/bukkit/entity/AbstractArrow.getDamage
    pub fn get_damage(&self) -> f64 {
        self.data_at("damage").as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setDamage
    pub fn set_damage(&self, value: f64) -> bool {
        self.merge(&crate::data::patch("damage", &format!("{}", format!("{value:?}d"))))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getPierceLevel
    pub fn get_pierce_level(&self) -> i32 {
        self.data_at("PierceLevel").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setPierceLevel
    pub fn set_pierce_level(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("PierceLevel", &format!("{}", format!("{value}b"))))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.isCritical
    pub fn is_critical(&self) -> bool {
        crate::data::flag(&self.data_at("crit"))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setCritical
    pub fn set_critical(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("crit", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.isInBlock
    pub fn is_in_block(&self) -> bool {
        crate::data::flag(&self.data_at("inGround"))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getPickupStatus
    pub fn get_pickup_status(&self) -> Option<&'static str> {
        self.data_at("pickup").as_i64().and_then(|number| crate::enums::abstract_arrow_pickup_status::values().get(usize::try_from(number).ok()?).copied())
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setPickupStatus
    pub fn set_pickup_status(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("pickup", &format!("{}", crate::enums::abstract_arrow_pickup_status::values().iter().position(|name| *name == value).unwrap_or(0))))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.isShotFromCrossbow
    pub fn is_shot_from_crossbow(&self) -> bool {
        crate::data::flag(&self.data_at("ShotFromCrossbow"))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setShotFromCrossbow
    pub fn set_shot_from_crossbow(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("ShotFromCrossbow", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getItem
    pub fn get_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("item"))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setItem
    pub fn set_item(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("item", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getWeapon
    pub fn get_weapon(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("weapon"))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setWeapon
    pub fn set_weapon(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("weapon", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getLifetimeTicks
    pub fn get_lifetime_ticks(&self) -> i32 {
        self.data_at("life").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setLifetimeTicks
    pub fn set_lifetime_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("life", &format!("{}", format!("{value}s"))))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getHitSound
    pub fn get_hit_sound(&self) -> Option<String> {
        self.data_at("SoundEvent").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setHitSound
    pub fn set_hit_sound(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("SoundEvent", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getItemStack
    pub fn get_item_stack(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("item"))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setItemStack
    pub fn set_item_stack(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("item", &format!("{}", value.to_snbt())))
    }
}

/// `org.bukkit.entity.AbstractHorse`.
#[derive(Clone, Debug, PartialEq)]
pub struct AbstractHorse(pub Entity);

impl std::ops::Deref for AbstractHorse {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for AbstractHorse {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl AbstractHorse {
    /// bukkit: org/bukkit/entity/AbstractHorse.getDomestication
    pub fn get_domestication(&self) -> i32 {
        self.data_at("Temper").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.setDomestication
    pub fn set_domestication(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Temper", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.isEatingHaystack
    pub fn is_eating_haystack(&self) -> bool {
        crate::data::flag(&self.data_at("EatingHaystack"))
    }
}

/// `org.bukkit.entity.Ageable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ageable(pub Entity);

impl std::ops::Deref for Ageable {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Ageable {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Ageable {
    /// bukkit: org/bukkit/entity/Ageable.getAge
    pub fn get_age(&self) -> i32 {
        self.data_at("Age").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Ageable.setAge
    pub fn set_age(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Age", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Ageable.getAgeLock
    pub fn get_age_lock(&self) -> bool {
        crate::data::flag(&self.data_at("AgeLocked"))
    }
    /// bukkit: org/bukkit/entity/Ageable.setAgeLock
    pub fn set_age_lock(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("AgeLocked", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Allay`.
#[derive(Clone, Debug, PartialEq)]
pub struct Allay(pub Entity);

impl std::ops::Deref for Allay {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Allay {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Allay {
    /// bukkit: org/bukkit/entity/Allay.canDuplicate
    pub fn can_duplicate(&self) -> bool {
        crate::data::flag(&self.data_at("CanDuplicate"))
    }
    /// bukkit: org/bukkit/entity/Allay.setCanDuplicate
    pub fn set_can_duplicate(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("CanDuplicate", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Allay.getDuplicationCooldown
    pub fn get_duplication_cooldown(&self) -> i64 {
        self.data_at("DuplicationCooldown").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/entity/Allay.setDuplicationCooldown
    pub fn set_duplication_cooldown(&self, value: i64) -> bool {
        self.merge(&crate::data::patch("DuplicationCooldown", &format!("{}", format!("{value}L"))))
    }
}

/// `org.bukkit.entity.Animals`.
#[derive(Clone, Debug, PartialEq)]
pub struct Animals(pub Entity);

impl std::ops::Deref for Animals {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Animals {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Animals {
    /// bukkit: org/bukkit/entity/Animals.getBreedCause
    pub fn get_breed_cause(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("LoveCause"))
    }
    /// bukkit: org/bukkit/entity/Animals.setBreedCause
    pub fn set_breed_cause(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("LoveCause", &format!("{}", crate::data::uuid_snbt(value).unwrap_or_default())))
    }
    /// bukkit: org/bukkit/entity/Animals.getLoveModeTicks
    pub fn get_love_mode_ticks(&self) -> i32 {
        self.data_at("InLove").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Animals.setLoveModeTicks
    pub fn set_love_mode_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("InLove", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.AreaEffectCloud`.
#[derive(Clone, Debug, PartialEq)]
pub struct AreaEffectCloud(pub Entity);

impl std::ops::Deref for AreaEffectCloud {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for AreaEffectCloud {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl AreaEffectCloud {
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getDuration
    pub fn get_duration(&self) -> i32 {
        self.data_at("Duration").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setDuration
    pub fn set_duration(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Duration", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getWaitTime
    pub fn get_wait_time(&self) -> i32 {
        self.data_at("WaitTime").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setWaitTime
    pub fn set_wait_time(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("WaitTime", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getReapplicationDelay
    pub fn get_reapplication_delay(&self) -> i32 {
        self.data_at("ReapplicationDelay").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setReapplicationDelay
    pub fn set_reapplication_delay(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("ReapplicationDelay", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getDurationOnUse
    pub fn get_duration_on_use(&self) -> i32 {
        self.data_at("DurationOnUse").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setDurationOnUse
    pub fn set_duration_on_use(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("DurationOnUse", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getRadius
    pub fn get_radius(&self) -> f32 {
        self.data_at("Radius").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setRadius
    pub fn set_radius(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("Radius", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getRadiusOnUse
    pub fn get_radius_on_use(&self) -> f32 {
        self.data_at("RadiusOnUse").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setRadiusOnUse
    pub fn set_radius_on_use(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("RadiusOnUse", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getRadiusPerTick
    pub fn get_radius_per_tick(&self) -> f32 {
        self.data_at("RadiusPerTick").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setRadiusPerTick
    pub fn set_radius_per_tick(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("RadiusPerTick", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getOwnerUniqueId
    pub fn get_owner_unique_id(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("Owner"))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setOwnerUniqueId
    pub fn set_owner_unique_id(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Owner", &format!("{}", crate::data::uuid_snbt(value).unwrap_or_default())))
    }
}

/// `org.bukkit.entity.Armadillo`.
#[derive(Clone, Debug, PartialEq)]
pub struct Armadillo(pub Entity);

impl std::ops::Deref for Armadillo {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Armadillo {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Armadillo {
    /// bukkit: org/bukkit/entity/Armadillo.getState
    pub fn get_state(&self) -> Option<&'static str> {
        self.data_at("state").as_str().and_then(|text| crate::enums::armadillo_state::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
}

/// `org.bukkit.entity.ArmorStand`.
#[derive(Clone, Debug, PartialEq)]
pub struct ArmorStand(pub Entity);

impl std::ops::Deref for ArmorStand {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ArmorStand {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ArmorStand {
    /// bukkit: org/bukkit/entity/ArmorStand.hasArms
    pub fn has_arms(&self) -> bool {
        crate::data::flag(&self.data_at("ShowArms"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setArms
    pub fn set_arms(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("ShowArms", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.isSmall
    pub fn is_small(&self) -> bool {
        crate::data::flag(&self.data_at("Small"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setSmall
    pub fn set_small(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Small", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.isMarker
    pub fn is_marker(&self) -> bool {
        crate::data::flag(&self.data_at("Marker"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setMarker
    pub fn set_marker(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Marker", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.hasBasePlate
    pub fn has_base_plate(&self) -> bool {
        !crate::data::flag(&self.data_at("NoBasePlate"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setBasePlate
    pub fn set_base_plate(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("NoBasePlate", &format!("{}", !value)))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.isVisible
    pub fn is_visible(&self) -> bool {
        !crate::data::flag(&self.data_at("Invisible"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setVisible
    pub fn set_visible(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Invisible", &format!("{}", !value)))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getItemInHand
    pub fn get_item_in_hand(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("equipment.mainhand"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setItemInHand
    pub fn set_item_in_hand(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("equipment.mainhand", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getBoots
    pub fn get_boots(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("equipment.feet"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setBoots
    pub fn set_boots(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("equipment.feet", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getLeggings
    pub fn get_leggings(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("equipment.legs"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setLeggings
    pub fn set_leggings(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("equipment.legs", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getChestplate
    pub fn get_chestplate(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("equipment.chest"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setChestplate
    pub fn set_chestplate(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("equipment.chest", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getHelmet
    pub fn get_helmet(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("equipment.head"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setHelmet
    pub fn set_helmet(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("equipment.head", &format!("{}", value.to_snbt())))
    }
}

/// `org.bukkit.entity.Axolotl`.
#[derive(Clone, Debug, PartialEq)]
pub struct Axolotl(pub Entity);

impl std::ops::Deref for Axolotl {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Axolotl {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Axolotl {
    /// bukkit: org/bukkit/entity/Axolotl.getVariant
    pub fn get_variant(&self) -> Option<&'static str> {
        self.data_at("Variant").as_i64().and_then(|number| crate::enums::axolotl_variant::values().get(usize::try_from(number).ok()?).copied())
    }
    /// bukkit: org/bukkit/entity/Axolotl.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Variant", &format!("{}", crate::enums::axolotl_variant::values().iter().position(|name| *name == value).unwrap_or(0))))
    }
}

/// `org.bukkit.entity.Bee`.
#[derive(Clone, Debug, PartialEq)]
pub struct Bee(pub Entity);

impl std::ops::Deref for Bee {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Bee {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Bee {
    /// bukkit: org/bukkit/entity/Bee.getHive
    pub fn get_hive(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("hive_pos"))
    }
    /// bukkit: org/bukkit/entity/Bee.setHive
    pub fn set_hive(&self, value: &crate::Location) -> bool {
        self.merge(&crate::data::patch("hive_pos", &format!("{}", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()))))
    }
    /// bukkit: org/bukkit/entity/Bee.getFlower
    pub fn get_flower(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("flower_pos"))
    }
    /// bukkit: org/bukkit/entity/Bee.setFlower
    pub fn set_flower(&self, value: &crate::Location) -> bool {
        self.merge(&crate::data::patch("flower_pos", &format!("{}", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()))))
    }
    /// bukkit: org/bukkit/entity/Bee.hasNectar
    pub fn has_nectar(&self) -> bool {
        crate::data::flag(&self.data_at("HasNectar"))
    }
    /// bukkit: org/bukkit/entity/Bee.setHasNectar
    pub fn set_has_nectar(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("HasNectar", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Bee.hasStung
    pub fn has_stung(&self) -> bool {
        crate::data::flag(&self.data_at("HasStung"))
    }
    /// bukkit: org/bukkit/entity/Bee.setHasStung
    pub fn set_has_stung(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("HasStung", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Bee.getCannotEnterHiveTicks
    pub fn get_cannot_enter_hive_ticks(&self) -> i32 {
        self.data_at("CannotEnterHiveTicks").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Bee.setCannotEnterHiveTicks
    pub fn set_cannot_enter_hive_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("CannotEnterHiveTicks", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Bee.getCropsGrownSincePollination
    pub fn get_crops_grown_since_pollination(&self) -> i32 {
        self.data_at("CropsGrownSincePollination").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Bee.setCropsGrownSincePollination
    pub fn set_crops_grown_since_pollination(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("CropsGrownSincePollination", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Bee.getTicksSincePollination
    pub fn get_ticks_since_pollination(&self) -> i32 {
        self.data_at("TicksSincePollination").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Bee.setTicksSincePollination
    pub fn set_ticks_since_pollination(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("TicksSincePollination", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.BlockDisplay`.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockDisplay(pub Entity);

impl std::ops::Deref for BlockDisplay {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for BlockDisplay {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl BlockDisplay {
    /// bukkit: org/bukkit/entity/BlockDisplay.getBlockName
    pub fn get_block_name(&self) -> Option<String> {
        self.data_at("block_state.Name").as_str().map(str::to_owned)
    }
}

/// `org.bukkit.entity.Bogged`.
#[derive(Clone, Debug, PartialEq)]
pub struct Bogged(pub Entity);

impl std::ops::Deref for Bogged {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Bogged {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Bogged {
    /// bukkit: org/bukkit/entity/Bogged.isSheared
    pub fn is_sheared(&self) -> bool {
        crate::data::flag(&self.data_at("sheared"))
    }
    /// bukkit: org/bukkit/entity/Bogged.setSheared
    pub fn set_sheared(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("sheared", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Cat`.
#[derive(Clone, Debug, PartialEq)]
pub struct Cat(pub Entity);

impl std::ops::Deref for Cat {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Cat {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Cat {
    /// bukkit: org/bukkit/entity/Cat.getCatType
    pub fn get_cat_type(&self) -> Option<String> {
        self.data_at("variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Cat.setCatType
    pub fn set_cat_type(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("variant", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Cat.getSoundVariant
    pub fn get_sound_variant(&self) -> Option<String> {
        self.data_at("sound_variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Cat.setSoundVariant
    pub fn set_sound_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("sound_variant", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Cat.getCollarColor
    pub fn get_collar_color(&self) -> Option<&'static str> {
        self.data_at("CollarColor").as_i64().and_then(|number| crate::enums::dye_color::values().get(usize::try_from(number).ok()?).copied())
    }
    /// bukkit: org/bukkit/entity/Cat.setCollarColor
    pub fn set_collar_color(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("CollarColor", &format!("{}", crate::enums::dye_color::values().iter().position(|name| *name == value).unwrap_or(0))))
    }
}

/// `org.bukkit.entity.ChestedHorse`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChestedHorse(pub Entity);

impl std::ops::Deref for ChestedHorse {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ChestedHorse {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ChestedHorse {
    /// bukkit: org/bukkit/entity/ChestedHorse.isCarryingChest
    pub fn is_carrying_chest(&self) -> bool {
        crate::data::flag(&self.data_at("ChestedHorse"))
    }
    /// bukkit: org/bukkit/entity/ChestedHorse.setCarryingChest
    pub fn set_carrying_chest(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("ChestedHorse", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Chicken`.
#[derive(Clone, Debug, PartialEq)]
pub struct Chicken(pub Entity);

impl std::ops::Deref for Chicken {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Chicken {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Chicken {
    /// bukkit: org/bukkit/entity/Chicken.getVariant
    pub fn get_variant(&self) -> Option<String> {
        self.data_at("variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Chicken.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("variant", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Chicken.getSoundVariant
    pub fn get_sound_variant(&self) -> Option<String> {
        self.data_at("sound_variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Chicken.setSoundVariant
    pub fn set_sound_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("sound_variant", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Chicken.isChickenJockey
    pub fn is_chicken_jockey(&self) -> bool {
        crate::data::flag(&self.data_at("IsChickenJockey"))
    }
    /// bukkit: org/bukkit/entity/Chicken.setIsChickenJockey
    pub fn set_is_chicken_jockey(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("IsChickenJockey", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Chicken.getEggLayTime
    pub fn get_egg_lay_time(&self) -> i32 {
        self.data_at("EggLayTime").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Chicken.setEggLayTime
    pub fn set_egg_lay_time(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("EggLayTime", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Cow`.
#[derive(Clone, Debug, PartialEq)]
pub struct Cow(pub Entity);

impl std::ops::Deref for Cow {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Cow {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Cow {
    /// bukkit: org/bukkit/entity/Cow.getVariant
    pub fn get_variant(&self) -> Option<String> {
        self.data_at("variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Cow.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("variant", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Cow.getSoundVariant
    pub fn get_sound_variant(&self) -> Option<String> {
        self.data_at("sound_variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Cow.setSoundVariant
    pub fn set_sound_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("sound_variant", &format!("{}", crate::data::quoted(value))))
    }
}

/// `org.bukkit.entity.Creaking`.
#[derive(Clone, Debug, PartialEq)]
pub struct Creaking(pub Entity);

impl std::ops::Deref for Creaking {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Creaking {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Creaking {
    /// bukkit: org/bukkit/entity/Creaking.getHome
    pub fn get_home(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("home_pos"))
    }
}

/// `org.bukkit.entity.Creeper`.
#[derive(Clone, Debug, PartialEq)]
pub struct Creeper(pub Entity);

impl std::ops::Deref for Creeper {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Creeper {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Creeper {
    /// bukkit: org/bukkit/entity/Creeper.isPowered
    pub fn is_powered(&self) -> bool {
        crate::data::flag(&self.data_at("powered"))
    }
    /// bukkit: org/bukkit/entity/Creeper.setPowered
    pub fn set_powered(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("powered", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Creeper.getMaxFuseTicks
    pub fn get_max_fuse_ticks(&self) -> i32 {
        self.data_at("Fuse").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Creeper.getExplosionRadius
    pub fn get_explosion_radius(&self) -> i32 {
        self.data_at("ExplosionRadius").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Creeper.isIgnited
    pub fn is_ignited(&self) -> bool {
        crate::data::flag(&self.data_at("ignited"))
    }
    /// bukkit: org/bukkit/entity/Creeper.setIgnited
    pub fn set_ignited(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("ignited", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Display`.
#[derive(Clone, Debug, PartialEq)]
pub struct Display(pub Entity);

impl std::ops::Deref for Display {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Display {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Display {
    /// bukkit: org/bukkit/entity/Display.getInterpolationDuration
    pub fn get_interpolation_duration(&self) -> i32 {
        self.data_at("interpolation_duration").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Display.setInterpolationDuration
    pub fn set_interpolation_duration(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("interpolation_duration", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Display.getTeleportDuration
    pub fn get_teleport_duration(&self) -> i32 {
        self.data_at("teleport_duration").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Display.setTeleportDuration
    pub fn set_teleport_duration(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("teleport_duration", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Display.getViewRange
    pub fn get_view_range(&self) -> f32 {
        self.data_at("view_range").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Display.setViewRange
    pub fn set_view_range(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("view_range", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/Display.getShadowRadius
    pub fn get_shadow_radius(&self) -> f32 {
        self.data_at("shadow_radius").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Display.setShadowRadius
    pub fn set_shadow_radius(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("shadow_radius", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/Display.getShadowStrength
    pub fn get_shadow_strength(&self) -> f32 {
        self.data_at("shadow_strength").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Display.setShadowStrength
    pub fn set_shadow_strength(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("shadow_strength", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/Display.getDisplayWidth
    pub fn get_display_width(&self) -> f32 {
        self.data_at("width").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Display.setDisplayWidth
    pub fn set_display_width(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("width", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/Display.getDisplayHeight
    pub fn get_display_height(&self) -> f32 {
        self.data_at("height").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Display.setDisplayHeight
    pub fn set_display_height(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("height", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/Display.getInterpolationDelay
    pub fn get_interpolation_delay(&self) -> i32 {
        self.data_at("start_interpolation").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Display.setInterpolationDelay
    pub fn set_interpolation_delay(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("start_interpolation", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Display.getBillboard
    pub fn get_billboard(&self) -> Option<&'static str> {
        self.data_at("billboard").as_str().and_then(|text| crate::enums::display_billboard::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
    /// bukkit: org/bukkit/entity/Display.setBillboard
    pub fn set_billboard(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("billboard", &format!("{}", crate::data::quoted(&value.to_ascii_lowercase()))))
    }
    /// bukkit: org/bukkit/entity/Display.getBlockLight
    pub fn get_block_light(&self) -> i32 {
        self.data_at("brightness.block").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Display.getSkyLight
    pub fn get_sky_light(&self) -> i32 {
        self.data_at("brightness.sky").as_i64().unwrap_or(0) as i32
    }
}

/// `org.bukkit.entity.Dolphin`.
#[derive(Clone, Debug, PartialEq)]
pub struct Dolphin(pub Entity);

impl std::ops::Deref for Dolphin {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Dolphin {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Dolphin {
    /// bukkit: org/bukkit/entity/Dolphin.getMoistness
    pub fn get_moistness(&self) -> i32 {
        self.data_at("Moistness").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Dolphin.setMoistness
    pub fn set_moistness(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Moistness", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Dolphin.hasFish
    pub fn has_fish(&self) -> bool {
        crate::data::flag(&self.data_at("GotFish"))
    }
    /// bukkit: org/bukkit/entity/Dolphin.setHasFish
    pub fn set_has_fish(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("GotFish", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.EnderCrystal`.
#[derive(Clone, Debug, PartialEq)]
pub struct EnderCrystal(pub Entity);

impl std::ops::Deref for EnderCrystal {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for EnderCrystal {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl EnderCrystal {
    /// bukkit: org/bukkit/entity/EnderCrystal.isShowingBottom
    pub fn is_showing_bottom(&self) -> bool {
        crate::data::flag(&self.data_at("ShowBottom"))
    }
    /// bukkit: org/bukkit/entity/EnderCrystal.setShowingBottom
    pub fn set_showing_bottom(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("ShowBottom", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/EnderCrystal.getBeamTarget
    pub fn get_beam_target(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("beam_target"))
    }
    /// bukkit: org/bukkit/entity/EnderCrystal.setBeamTarget
    pub fn set_beam_target(&self, value: &crate::Location) -> bool {
        self.merge(&crate::data::patch("beam_target", &format!("{}", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()))))
    }
}

/// `org.bukkit.entity.EnderSignal`.
#[derive(Clone, Debug, PartialEq)]
pub struct EnderSignal(pub Entity);

impl std::ops::Deref for EnderSignal {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for EnderSignal {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.Enderman`.
#[derive(Clone, Debug, PartialEq)]
pub struct Enderman(pub Entity);

impl std::ops::Deref for Enderman {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Enderman {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.AbstractCubeMob`.
#[derive(Clone, Debug, PartialEq)]
pub struct AbstractCubeMob(pub Entity);

impl std::ops::Deref for AbstractCubeMob {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for AbstractCubeMob {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.Horse`.
#[derive(Clone, Debug, PartialEq)]
pub struct Horse(pub Entity);

impl std::ops::Deref for Horse {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Horse {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.Painting`.
#[derive(Clone, Debug, PartialEq)]
pub struct Painting(pub Entity);

impl std::ops::Deref for Painting {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Painting {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.Slime`.
#[derive(Clone, Debug, PartialEq)]
pub struct Slime(pub Entity);

impl std::ops::Deref for Slime {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Slime {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.TropicalFish`.
#[derive(Clone, Debug, PartialEq)]
pub struct TropicalFish(pub Entity);

impl std::ops::Deref for TropicalFish {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for TropicalFish {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.Mob`.
#[derive(Clone, Debug, PartialEq)]
pub struct Mob(pub Entity);

impl std::ops::Deref for Mob {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Mob {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.Endermite`.
#[derive(Clone, Debug, PartialEq)]
pub struct Endermite(pub Entity);

impl std::ops::Deref for Endermite {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Endermite {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Endermite {
    /// bukkit: org/bukkit/entity/Endermite.getLifetimeTicks
    pub fn get_lifetime_ticks(&self) -> i32 {
        self.data_at("Lifetime").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Endermite.setLifetimeTicks
    pub fn set_lifetime_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Lifetime", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.EvokerFangs`.
#[derive(Clone, Debug, PartialEq)]
pub struct EvokerFangs(pub Entity);

impl std::ops::Deref for EvokerFangs {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for EvokerFangs {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.ExperienceOrb`.
#[derive(Clone, Debug, PartialEq)]
pub struct ExperienceOrb(pub Entity);

impl std::ops::Deref for ExperienceOrb {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ExperienceOrb {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ExperienceOrb {
    /// bukkit: org/bukkit/entity/ExperienceOrb.getExperience
    pub fn get_experience(&self) -> i32 {
        self.data_at("Value").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/ExperienceOrb.setExperience
    pub fn set_experience(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Value", &format!("{}", format!("{value}s"))))
    }
    /// bukkit: org/bukkit/entity/ExperienceOrb.getCount
    pub fn get_count(&self) -> i32 {
        self.data_at("Count").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/ExperienceOrb.setCount
    pub fn set_count(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Count", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Explosive`.
#[derive(Clone, Debug, PartialEq)]
pub struct Explosive(pub Entity);

impl std::ops::Deref for Explosive {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Explosive {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Explosive {
    /// bukkit: org/bukkit/entity/Explosive.getYield
    pub fn get_yield(&self) -> f32 {
        self.data_at("explosion_power").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Explosive.setYield
    pub fn set_yield(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("explosion_power", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/Explosive.isIncendiary
    pub fn is_incendiary(&self) -> bool {
        crate::data::flag(&self.data_at("explosion_incendiary"))
    }
    /// bukkit: org/bukkit/entity/Explosive.setIsIncendiary
    pub fn set_is_incendiary(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("explosion_incendiary", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.FallingBlock`.
#[derive(Clone, Debug, PartialEq)]
pub struct FallingBlock(pub Entity);

impl std::ops::Deref for FallingBlock {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for FallingBlock {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl FallingBlock {
    /// bukkit: org/bukkit/entity/FallingBlock.getDropItem
    pub fn get_drop_item(&self) -> bool {
        crate::data::flag(&self.data_at("DropItem"))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.setDropItem
    pub fn set_drop_item(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("DropItem", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.getCancelDrop
    pub fn get_cancel_drop(&self) -> bool {
        crate::data::flag(&self.data_at("CancelDrop"))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.setCancelDrop
    pub fn set_cancel_drop(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("CancelDrop", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.canHurtEntities
    pub fn can_hurt_entities(&self) -> bool {
        crate::data::flag(&self.data_at("HurtEntities"))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.setHurtEntities
    pub fn set_hurt_entities(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("HurtEntities", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.getDamagePerBlock
    pub fn get_damage_per_block(&self) -> f32 {
        self.data_at("FallHurtAmount").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/FallingBlock.setDamagePerBlock
    pub fn set_damage_per_block(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("FallHurtAmount", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.getMaxDamage
    pub fn get_max_damage(&self) -> i32 {
        self.data_at("FallHurtMax").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/FallingBlock.setMaxDamage
    pub fn set_max_damage(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("FallHurtMax", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Firework`.
#[derive(Clone, Debug, PartialEq)]
pub struct Firework(pub Entity);

impl std::ops::Deref for Firework {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Firework {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Firework {
    /// bukkit: org/bukkit/entity/Firework.getItem
    pub fn get_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("FireworksItem"))
    }
    /// bukkit: org/bukkit/entity/Firework.setItem
    pub fn set_item(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("FireworksItem", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/Firework.isShotAtAngle
    pub fn is_shot_at_angle(&self) -> bool {
        crate::data::flag(&self.data_at("ShotAtAngle"))
    }
    /// bukkit: org/bukkit/entity/Firework.setShotAtAngle
    pub fn set_shot_at_angle(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("ShotAtAngle", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Fox`.
#[derive(Clone, Debug, PartialEq)]
pub struct Fox(pub Entity);

impl std::ops::Deref for Fox {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Fox {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Fox {
    /// bukkit: org/bukkit/entity/Fox.getFoxType
    pub fn get_fox_type(&self) -> Option<&'static str> {
        self.data_at("Type").as_str().and_then(|text| crate::enums::fox_type::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
    /// bukkit: org/bukkit/entity/Fox.setFoxType
    pub fn set_fox_type(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Type", &format!("{}", crate::data::quoted(&value.to_ascii_lowercase()))))
    }
    /// bukkit: org/bukkit/entity/Fox.isCrouching
    pub fn is_crouching(&self) -> bool {
        crate::data::flag(&self.data_at("Crouching"))
    }
    /// bukkit: org/bukkit/entity/Fox.setCrouching
    pub fn set_crouching(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Crouching", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Fox.isSleeping
    pub fn is_sleeping(&self) -> bool {
        crate::data::flag(&self.data_at("Sleeping"))
    }
    /// bukkit: org/bukkit/entity/Fox.setSleeping
    pub fn set_sleeping(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Sleeping", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Frog`.
#[derive(Clone, Debug, PartialEq)]
pub struct Frog(pub Entity);

impl std::ops::Deref for Frog {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Frog {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Frog {
    /// bukkit: org/bukkit/entity/Frog.getVariant
    pub fn get_variant(&self) -> Option<String> {
        self.data_at("variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Frog.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("variant", &format!("{}", crate::data::quoted(value))))
    }
}

/// `org.bukkit.entity.Ghast`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ghast(pub Entity);

impl std::ops::Deref for Ghast {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Ghast {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Ghast {
    /// bukkit: org/bukkit/entity/Ghast.getExplosionPower
    pub fn get_explosion_power(&self) -> i32 {
        self.data_at("ExplosionPower").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Ghast.setExplosionPower
    pub fn set_explosion_power(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("ExplosionPower", &format!("{}", format!("{value}b"))))
    }
}

/// `org.bukkit.entity.GlowSquid`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlowSquid(pub Entity);

impl std::ops::Deref for GlowSquid {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for GlowSquid {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl GlowSquid {
    /// bukkit: org/bukkit/entity/GlowSquid.getDarkTicksRemaining
    pub fn get_dark_ticks_remaining(&self) -> i32 {
        self.data_at("DarkTicksRemaining").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/GlowSquid.setDarkTicksRemaining
    pub fn set_dark_ticks_remaining(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("DarkTicksRemaining", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Goat`.
#[derive(Clone, Debug, PartialEq)]
pub struct Goat(pub Entity);

impl std::ops::Deref for Goat {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Goat {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Goat {
    /// bukkit: org/bukkit/entity/Goat.hasLeftHorn
    pub fn has_left_horn(&self) -> bool {
        crate::data::flag(&self.data_at("HasLeftHorn"))
    }
    /// bukkit: org/bukkit/entity/Goat.setLeftHorn
    pub fn set_left_horn(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("HasLeftHorn", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Goat.hasRightHorn
    pub fn has_right_horn(&self) -> bool {
        crate::data::flag(&self.data_at("HasRightHorn"))
    }
    /// bukkit: org/bukkit/entity/Goat.setRightHorn
    pub fn set_right_horn(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("HasRightHorn", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Goat.isScreaming
    pub fn is_screaming(&self) -> bool {
        crate::data::flag(&self.data_at("IsScreamingGoat"))
    }
    /// bukkit: org/bukkit/entity/Goat.setScreaming
    pub fn set_screaming(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("IsScreamingGoat", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Hoglin`.
#[derive(Clone, Debug, PartialEq)]
pub struct Hoglin(pub Entity);

impl std::ops::Deref for Hoglin {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Hoglin {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Hoglin {
    /// bukkit: org/bukkit/entity/Hoglin.isImmuneToZombification
    pub fn is_immune_to_zombification(&self) -> bool {
        crate::data::flag(&self.data_at("IsImmuneToZombification"))
    }
    /// bukkit: org/bukkit/entity/Hoglin.setImmuneToZombification
    pub fn set_immune_to_zombification(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("IsImmuneToZombification", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Hoglin.isAbleToBeHunted
    pub fn is_able_to_be_hunted(&self) -> bool {
        !crate::data::flag(&self.data_at("CannotBeHunted"))
    }
    /// bukkit: org/bukkit/entity/Hoglin.setIsAbleToBeHunted
    pub fn set_is_able_to_be_hunted(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("CannotBeHunted", &format!("{}", !value)))
    }
}

/// `org.bukkit.entity.Interaction`.
#[derive(Clone, Debug, PartialEq)]
pub struct Interaction(pub Entity);

impl std::ops::Deref for Interaction {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Interaction {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Interaction {
    /// bukkit: org/bukkit/entity/Interaction.getInteractionWidth
    pub fn get_interaction_width(&self) -> f32 {
        self.data_at("width").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Interaction.setInteractionWidth
    pub fn set_interaction_width(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("width", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/Interaction.getInteractionHeight
    pub fn get_interaction_height(&self) -> f32 {
        self.data_at("height").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Interaction.setInteractionHeight
    pub fn set_interaction_height(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("height", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/Interaction.isResponsive
    pub fn is_responsive(&self) -> bool {
        crate::data::flag(&self.data_at("response"))
    }
    /// bukkit: org/bukkit/entity/Interaction.setResponsive
    pub fn set_responsive(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("response", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.IronGolem`.
#[derive(Clone, Debug, PartialEq)]
pub struct IronGolem(pub Entity);

impl std::ops::Deref for IronGolem {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for IronGolem {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl IronGolem {
    /// bukkit: org/bukkit/entity/IronGolem.isPlayerCreated
    pub fn is_player_created(&self) -> bool {
        crate::data::flag(&self.data_at("PlayerCreated"))
    }
    /// bukkit: org/bukkit/entity/IronGolem.setPlayerCreated
    pub fn set_player_created(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("PlayerCreated", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Item`.
#[derive(Clone, Debug, PartialEq)]
pub struct Item(pub Entity);

impl std::ops::Deref for Item {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Item {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Item {
    /// bukkit: org/bukkit/entity/Item.getItemStack
    pub fn get_item_stack(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("Item"))
    }
    /// bukkit: org/bukkit/entity/Item.setItemStack
    pub fn set_item_stack(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("Item", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/Item.getPickupDelay
    pub fn get_pickup_delay(&self) -> i32 {
        self.data_at("PickupDelay").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Item.setPickupDelay
    pub fn set_pickup_delay(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("PickupDelay", &format!("{}", format!("{value}s"))))
    }
    /// bukkit: org/bukkit/entity/Item.getOwner
    pub fn get_owner(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("Owner"))
    }
    /// bukkit: org/bukkit/entity/Item.setOwner
    pub fn set_owner(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Owner", &format!("{}", crate::data::uuid_snbt(value).unwrap_or_default())))
    }
    /// bukkit: org/bukkit/entity/Item.getThrower
    pub fn get_thrower(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("Thrower"))
    }
    /// bukkit: org/bukkit/entity/Item.setThrower
    pub fn set_thrower(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Thrower", &format!("{}", crate::data::uuid_snbt(value).unwrap_or_default())))
    }
    /// bukkit: org/bukkit/entity/Item.getHealth
    pub fn get_health(&self) -> i32 {
        self.data_at("Health").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Item.setHealth
    pub fn set_health(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Health", &format!("{}", format!("{value}s"))))
    }
}

/// `org.bukkit.entity.ItemDisplay`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemDisplay(pub Entity);

impl std::ops::Deref for ItemDisplay {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ItemDisplay {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ItemDisplay {
    /// bukkit: org/bukkit/entity/ItemDisplay.getItemStack
    pub fn get_item_stack(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("item"))
    }
    /// bukkit: org/bukkit/entity/ItemDisplay.setItemStack
    pub fn set_item_stack(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("item", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/ItemDisplay.getItemDisplayTransform
    pub fn get_item_display_transform(&self) -> Option<&'static str> {
        self.data_at("item_display").as_str().and_then(|text| crate::enums::item_display_item_display_transform::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
    /// bukkit: org/bukkit/entity/ItemDisplay.setItemDisplayTransform
    pub fn set_item_display_transform(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("item_display", &format!("{}", crate::data::quoted(&value.to_ascii_lowercase()))))
    }
}

/// `org.bukkit.entity.ItemFrame`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemFrame(pub Entity);

impl std::ops::Deref for ItemFrame {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ItemFrame {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ItemFrame {
    /// bukkit: org/bukkit/entity/ItemFrame.getItem
    pub fn get_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("Item"))
    }
    /// bukkit: org/bukkit/entity/ItemFrame.setItem
    pub fn set_item(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("Item", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/ItemFrame.getItemDropChance
    pub fn get_item_drop_chance(&self) -> f32 {
        self.data_at("ItemDropChance").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/ItemFrame.setItemDropChance
    pub fn set_item_drop_chance(&self, value: f32) -> bool {
        self.merge(&crate::data::patch("ItemDropChance", &format!("{}", format!("{value:?}f"))))
    }
    /// bukkit: org/bukkit/entity/ItemFrame.getRotation
    pub fn get_rotation(&self) -> Option<&'static str> {
        self.data_at("ItemRotation").as_i64().and_then(|number| crate::enums::rotation::values().get(usize::try_from(number).ok()?).copied())
    }
    /// bukkit: org/bukkit/entity/ItemFrame.setRotation
    pub fn set_rotation(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("ItemRotation", &format!("{}", crate::enums::rotation::values().iter().position(|name| *name == value).unwrap_or(0))))
    }
    /// bukkit: org/bukkit/entity/ItemFrame.isVisible
    pub fn is_visible(&self) -> bool {
        !crate::data::flag(&self.data_at("Invisible"))
    }
    /// bukkit: org/bukkit/entity/ItemFrame.setVisible
    pub fn set_visible(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Invisible", &format!("{}", !value)))
    }
    /// bukkit: org/bukkit/entity/ItemFrame.isFixed
    pub fn is_fixed(&self) -> bool {
        crate::data::flag(&self.data_at("Fixed"))
    }
    /// bukkit: org/bukkit/entity/ItemFrame.setFixed
    pub fn set_fixed(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Fixed", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Llama`.
#[derive(Clone, Debug, PartialEq)]
pub struct Llama(pub Entity);

impl std::ops::Deref for Llama {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Llama {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Llama {
    /// bukkit: org/bukkit/entity/Llama.getColor
    pub fn get_color(&self) -> Option<&'static str> {
        self.data_at("Variant").as_i64().and_then(|number| crate::enums::llama_color::values().get(usize::try_from(number).ok()?).copied())
    }
    /// bukkit: org/bukkit/entity/Llama.setColor
    pub fn set_color(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Variant", &format!("{}", crate::enums::llama_color::values().iter().position(|name| *name == value).unwrap_or(0))))
    }
    /// bukkit: org/bukkit/entity/Llama.getStrength
    pub fn get_strength(&self) -> i32 {
        self.data_at("Strength").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Llama.setStrength
    pub fn set_strength(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Strength", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Minecart`.
#[derive(Clone, Debug, PartialEq)]
pub struct Minecart(pub Entity);

impl std::ops::Deref for Minecart {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Minecart {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Minecart {
    /// bukkit: org/bukkit/entity/Minecart.getDisplayBlockOffset
    pub fn get_display_block_offset(&self) -> i32 {
        self.data_at("DisplayOffset").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Minecart.setDisplayBlockOffset
    pub fn set_display_block_offset(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("DisplayOffset", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.MushroomCow`.
#[derive(Clone, Debug, PartialEq)]
pub struct MushroomCow(pub Entity);

impl std::ops::Deref for MushroomCow {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for MushroomCow {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl MushroomCow {
    /// bukkit: org/bukkit/entity/MushroomCow.getVariant
    pub fn get_variant(&self) -> Option<&'static str> {
        self.data_at("Type").as_str().and_then(|text| crate::enums::mushroom_cow_variant::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
    /// bukkit: org/bukkit/entity/MushroomCow.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Type", &format!("{}", crate::data::quoted(&value.to_ascii_lowercase()))))
    }
}

/// `org.bukkit.entity.OminousItemSpawner`.
#[derive(Clone, Debug, PartialEq)]
pub struct OminousItemSpawner(pub Entity);

impl std::ops::Deref for OminousItemSpawner {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for OminousItemSpawner {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl OminousItemSpawner {
    /// bukkit: org/bukkit/entity/OminousItemSpawner.getItem
    pub fn get_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("item"))
    }
    /// bukkit: org/bukkit/entity/OminousItemSpawner.setItem
    pub fn set_item(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("item", &format!("{}", value.to_snbt())))
    }
    /// bukkit: org/bukkit/entity/OminousItemSpawner.getSpawnItemAfterTicks
    pub fn get_spawn_item_after_ticks(&self) -> i64 {
        self.data_at("spawn_item_after_ticks").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/entity/OminousItemSpawner.setSpawnItemAfterTicks
    pub fn set_spawn_item_after_ticks(&self, value: i64) -> bool {
        self.merge(&crate::data::patch("spawn_item_after_ticks", &format!("{}", format!("{value}L"))))
    }
}

/// `org.bukkit.entity.Ocelot`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ocelot(pub Entity);

impl std::ops::Deref for Ocelot {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Ocelot {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Ocelot {
    /// bukkit: org/bukkit/entity/Ocelot.isTrusting
    pub fn is_trusting(&self) -> bool {
        crate::data::flag(&self.data_at("Trusting"))
    }
    /// bukkit: org/bukkit/entity/Ocelot.setTrusting
    pub fn set_trusting(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Trusting", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Panda`.
#[derive(Clone, Debug, PartialEq)]
pub struct Panda(pub Entity);

impl std::ops::Deref for Panda {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Panda {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Panda {
    /// bukkit: org/bukkit/entity/Panda.getMainGene
    pub fn get_main_gene(&self) -> Option<&'static str> {
        self.data_at("MainGene").as_str().and_then(|text| crate::enums::panda_gene::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
    /// bukkit: org/bukkit/entity/Panda.setMainGene
    pub fn set_main_gene(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("MainGene", &format!("{}", crate::data::quoted(&value.to_ascii_lowercase()))))
    }
    /// bukkit: org/bukkit/entity/Panda.getHiddenGene
    pub fn get_hidden_gene(&self) -> Option<&'static str> {
        self.data_at("HiddenGene").as_str().and_then(|text| crate::enums::panda_gene::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
    /// bukkit: org/bukkit/entity/Panda.setHiddenGene
    pub fn set_hidden_gene(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("HiddenGene", &format!("{}", crate::data::quoted(&value.to_ascii_lowercase()))))
    }
}

/// `org.bukkit.entity.Parrot`.
#[derive(Clone, Debug, PartialEq)]
pub struct Parrot(pub Entity);

impl std::ops::Deref for Parrot {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Parrot {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Parrot {
    /// bukkit: org/bukkit/entity/Parrot.getVariant
    pub fn get_variant(&self) -> Option<&'static str> {
        self.data_at("Variant").as_i64().and_then(|number| crate::enums::parrot_variant::values().get(usize::try_from(number).ok()?).copied())
    }
    /// bukkit: org/bukkit/entity/Parrot.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Variant", &format!("{}", crate::enums::parrot_variant::values().iter().position(|name| *name == value).unwrap_or(0))))
    }
}

/// `org.bukkit.entity.Phantom`.
#[derive(Clone, Debug, PartialEq)]
pub struct Phantom(pub Entity);

impl std::ops::Deref for Phantom {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Phantom {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Phantom {
    /// bukkit: org/bukkit/entity/Phantom.getSpawningEntity
    pub fn get_spawning_entity(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("SpawningEntity"))
    }
    /// bukkit: org/bukkit/entity/Phantom.getAnchorLocation
    pub fn get_anchor_location(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("anchor_pos"))
    }
    /// bukkit: org/bukkit/entity/Phantom.setAnchorLocation
    pub fn set_anchor_location(&self, value: &crate::Location) -> bool {
        self.merge(&crate::data::patch("anchor_pos", &format!("{}", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()))))
    }
}

/// `org.bukkit.entity.Pig`.
#[derive(Clone, Debug, PartialEq)]
pub struct Pig(pub Entity);

impl std::ops::Deref for Pig {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Pig {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Pig {
    /// bukkit: org/bukkit/entity/Pig.getVariant
    pub fn get_variant(&self) -> Option<String> {
        self.data_at("variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Pig.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("variant", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Pig.getSoundVariant
    pub fn get_sound_variant(&self) -> Option<String> {
        self.data_at("sound_variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Pig.setSoundVariant
    pub fn set_sound_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("sound_variant", &format!("{}", crate::data::quoted(value))))
    }
}

/// `org.bukkit.entity.PiglinAbstract`.
#[derive(Clone, Debug, PartialEq)]
pub struct PiglinAbstract(pub Entity);

impl std::ops::Deref for PiglinAbstract {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for PiglinAbstract {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl PiglinAbstract {
    /// bukkit: org/bukkit/entity/PiglinAbstract.isImmuneToZombification
    pub fn is_immune_to_zombification(&self) -> bool {
        crate::data::flag(&self.data_at("IsImmuneToZombification"))
    }
    /// bukkit: org/bukkit/entity/PiglinAbstract.setImmuneToZombification
    pub fn set_immune_to_zombification(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("IsImmuneToZombification", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/PiglinAbstract.isBaby
    pub fn is_baby(&self) -> bool {
        crate::data::flag(&self.data_at("IsBaby"))
    }
    /// bukkit: org/bukkit/entity/PiglinAbstract.setBaby
    pub fn set_baby(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("IsBaby", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Piglin`.
#[derive(Clone, Debug, PartialEq)]
pub struct Piglin(pub Entity);

impl std::ops::Deref for Piglin {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Piglin {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Piglin {
    /// bukkit: org/bukkit/entity/Piglin.isAbleToHunt
    pub fn is_able_to_hunt(&self) -> bool {
        !crate::data::flag(&self.data_at("CannotHunt"))
    }
    /// bukkit: org/bukkit/entity/Piglin.setIsAbleToHunt
    pub fn set_is_able_to_hunt(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("CannotHunt", &format!("{}", !value)))
    }
}

/// `org.bukkit.entity.Projectile`.
#[derive(Clone, Debug, PartialEq)]
pub struct Projectile(pub Entity);

impl std::ops::Deref for Projectile {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Projectile {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Projectile {
    /// bukkit: org/bukkit/entity/Projectile.getOwnerUniqueId
    pub fn get_owner_unique_id(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("Owner"))
    }
    /// bukkit: org/bukkit/entity/Projectile.hasLeftShooter
    pub fn has_left_shooter(&self) -> bool {
        crate::data::flag(&self.data_at("LeftOwner"))
    }
    /// bukkit: org/bukkit/entity/Projectile.setHasLeftShooter
    pub fn set_has_left_shooter(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("LeftOwner", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Projectile.hasBeenShot
    pub fn has_been_shot(&self) -> bool {
        crate::data::flag(&self.data_at("HasBeenShot"))
    }
    /// bukkit: org/bukkit/entity/Projectile.setHasBeenShot
    pub fn set_has_been_shot(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("HasBeenShot", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.PufferFish`.
#[derive(Clone, Debug, PartialEq)]
pub struct PufferFish(pub Entity);

impl std::ops::Deref for PufferFish {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for PufferFish {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl PufferFish {
    /// bukkit: org/bukkit/entity/PufferFish.getPuffState
    pub fn get_puff_state(&self) -> i32 {
        self.data_at("PuffState").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/PufferFish.setPuffState
    pub fn set_puff_state(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("PuffState", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Rabbit`.
#[derive(Clone, Debug, PartialEq)]
pub struct Rabbit(pub Entity);

impl std::ops::Deref for Rabbit {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Rabbit {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Rabbit {
    /// bukkit: org/bukkit/entity/Rabbit.getMoreCarrotTicks
    pub fn get_more_carrot_ticks(&self) -> i32 {
        self.data_at("MoreCarrotTicks").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Rabbit.setMoreCarrotTicks
    pub fn set_more_carrot_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("MoreCarrotTicks", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Raider`.
#[derive(Clone, Debug, PartialEq)]
pub struct Raider(pub Entity);

impl std::ops::Deref for Raider {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Raider {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Raider {
    /// bukkit: org/bukkit/entity/Raider.getWave
    pub fn get_wave(&self) -> i32 {
        self.data_at("Wave").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Raider.setWave
    pub fn set_wave(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Wave", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Raider.getPatrolTarget
    pub fn get_patrol_target(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("patrol_target"))
    }
    /// bukkit: org/bukkit/entity/Raider.setPatrolTarget
    pub fn set_patrol_target(&self, value: &crate::Location) -> bool {
        self.merge(&crate::data::patch("patrol_target", &format!("{}", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()))))
    }
    /// bukkit: org/bukkit/entity/Raider.isPatrolLeader
    pub fn is_patrol_leader(&self) -> bool {
        crate::data::flag(&self.data_at("PatrolLeader"))
    }
    /// bukkit: org/bukkit/entity/Raider.setPatrolLeader
    pub fn set_patrol_leader(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("PatrolLeader", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Raider.isCanJoinRaid
    pub fn is_can_join_raid(&self) -> bool {
        crate::data::flag(&self.data_at("CanJoinRaid"))
    }
    /// bukkit: org/bukkit/entity/Raider.setCanJoinRaid
    pub fn set_can_join_raid(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("CanJoinRaid", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Raider.getTicksOutsideRaid
    pub fn get_ticks_outside_raid(&self) -> i32 {
        self.data_at("TicksOutsideRaid").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Raider.setTicksOutsideRaid
    pub fn set_ticks_outside_raid(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("TicksOutsideRaid", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Ravager`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ravager(pub Entity);

impl std::ops::Deref for Ravager {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Ravager {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Ravager {
    /// bukkit: org/bukkit/entity/Ravager.getAttackTicks
    pub fn get_attack_ticks(&self) -> i32 {
        self.data_at("AttackTick").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Ravager.setAttackTicks
    pub fn set_attack_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("AttackTick", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Ravager.getStunnedTicks
    pub fn get_stunned_ticks(&self) -> i32 {
        self.data_at("StunTick").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Ravager.setStunnedTicks
    pub fn set_stunned_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("StunTick", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Ravager.getRoarTicks
    pub fn get_roar_ticks(&self) -> i32 {
        self.data_at("RoarTick").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Ravager.setRoarTicks
    pub fn set_roar_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("RoarTick", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Salmon`.
#[derive(Clone, Debug, PartialEq)]
pub struct Salmon(pub Entity);

impl std::ops::Deref for Salmon {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Salmon {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Salmon {
    /// bukkit: org/bukkit/entity/Salmon.getVariant
    pub fn get_variant(&self) -> Option<&'static str> {
        self.data_at("type").as_str().and_then(|text| crate::enums::salmon_variant::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
    /// bukkit: org/bukkit/entity/Salmon.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("type", &format!("{}", crate::data::quoted(&value.to_ascii_lowercase()))))
    }
}

/// `org.bukkit.entity.Sheep`.
#[derive(Clone, Debug, PartialEq)]
pub struct Sheep(pub Entity);

impl std::ops::Deref for Sheep {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Sheep {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Sheep {
    /// bukkit: org/bukkit/entity/Sheep.isSheared
    pub fn is_sheared(&self) -> bool {
        crate::data::flag(&self.data_at("Sheared"))
    }
    /// bukkit: org/bukkit/entity/Sheep.setSheared
    pub fn set_sheared(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Sheared", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Shulker`.
#[derive(Clone, Debug, PartialEq)]
pub struct Shulker(pub Entity);

impl std::ops::Deref for Shulker {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Shulker {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Shulker {
    /// bukkit: org/bukkit/entity/Shulker.getPeek
    pub fn get_peek(&self) -> i32 {
        self.data_at("Peek").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Shulker.setPeek
    pub fn set_peek(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Peek", &format!("{}", format!("{value}b"))))
    }
}

/// `org.bukkit.entity.Sittable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Sittable(pub Entity);

impl std::ops::Deref for Sittable {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Sittable {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Sittable {
    /// bukkit: org/bukkit/entity/Sittable.isSitting
    pub fn is_sitting(&self) -> bool {
        crate::data::flag(&self.data_at("Sitting"))
    }
    /// bukkit: org/bukkit/entity/Sittable.setSitting
    pub fn set_sitting(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Sitting", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Snowman`.
#[derive(Clone, Debug, PartialEq)]
pub struct Snowman(pub Entity);

impl std::ops::Deref for Snowman {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Snowman {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Snowman {
    /// bukkit: org/bukkit/entity/Snowman.isDerp
    pub fn is_derp(&self) -> bool {
        !crate::data::flag(&self.data_at("Pumpkin"))
    }
    /// bukkit: org/bukkit/entity/Snowman.setDerp
    pub fn set_derp(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Pumpkin", &format!("{}", !value)))
    }
}

/// `org.bukkit.entity.SpectralArrow`.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectralArrow(pub Entity);

impl std::ops::Deref for SpectralArrow {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for SpectralArrow {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl SpectralArrow {
    /// bukkit: org/bukkit/entity/SpectralArrow.getGlowingTicks
    pub fn get_glowing_ticks(&self) -> i32 {
        self.data_at("Duration").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/SpectralArrow.setGlowingTicks
    pub fn set_glowing_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Duration", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Spellcaster`.
#[derive(Clone, Debug, PartialEq)]
pub struct Spellcaster(pub Entity);

impl std::ops::Deref for Spellcaster {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Spellcaster {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.Steerable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Steerable(pub Entity);

impl std::ops::Deref for Steerable {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Steerable {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Steerable {
    /// bukkit: org/bukkit/entity/Steerable.hasSaddle
    pub fn has_saddle(&self) -> bool {
        crate::data::flag(&self.data_at("equipment.saddle"))
    }
}

/// `org.bukkit.entity.Strider`.
#[derive(Clone, Debug, PartialEq)]
pub struct Strider(pub Entity);

impl std::ops::Deref for Strider {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Strider {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.TNTPrimed`.
#[derive(Clone, Debug, PartialEq)]
pub struct TNTPrimed(pub Entity);

impl std::ops::Deref for TNTPrimed {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for TNTPrimed {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl TNTPrimed {
    /// bukkit: org/bukkit/entity/TNTPrimed.getFuseTicks
    pub fn get_fuse_ticks(&self) -> i32 {
        self.data_at("fuse").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/TNTPrimed.setFuseTicks
    pub fn set_fuse_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("fuse", &format!("{}", format!("{value}s"))))
    }
}

/// `org.bukkit.entity.Tadpole`.
#[derive(Clone, Debug, PartialEq)]
pub struct Tadpole(pub Entity);

impl std::ops::Deref for Tadpole {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Tadpole {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Tadpole {
    /// bukkit: org/bukkit/entity/Tadpole.getAge
    pub fn get_age(&self) -> i32 {
        self.data_at("Age").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Tadpole.setAge
    pub fn set_age(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Age", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Tadpole.getAgeLock
    pub fn get_age_lock(&self) -> bool {
        crate::data::flag(&self.data_at("AgeLocked"))
    }
    /// bukkit: org/bukkit/entity/Tadpole.setAgeLock
    pub fn set_age_lock(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("AgeLocked", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Tameable`.
#[derive(Clone, Debug, PartialEq)]
pub struct Tameable(pub Entity);

impl std::ops::Deref for Tameable {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Tameable {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Tameable {
    /// bukkit: org/bukkit/entity/Tameable.isTamed
    pub fn is_tamed(&self) -> bool {
        crate::data::flag(&self.data_at("Tame"))
    }
    /// bukkit: org/bukkit/entity/Tameable.setTamed
    pub fn set_tamed(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Tame", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Tameable.getOwnerUniqueId
    pub fn get_owner_unique_id(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("Owner"))
    }
}

/// `org.bukkit.entity.TextDisplay`.
#[derive(Clone, Debug, PartialEq)]
pub struct TextDisplay(pub Entity);

impl std::ops::Deref for TextDisplay {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for TextDisplay {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl TextDisplay {
    /// bukkit: org/bukkit/entity/TextDisplay.getLineWidth
    pub fn get_line_width(&self) -> i32 {
        self.data_at("line_width").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/TextDisplay.setLineWidth
    pub fn set_line_width(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("line_width", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.getTextOpacity
    pub fn get_text_opacity(&self) -> i32 {
        self.data_at("text_opacity").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/TextDisplay.setTextOpacity
    pub fn set_text_opacity(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("text_opacity", &format!("{}", format!("{value}b"))))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.isShadowed
    pub fn is_shadowed(&self) -> bool {
        crate::data::flag(&self.data_at("shadow"))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.setShadowed
    pub fn set_shadowed(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("shadow", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.isSeeThrough
    pub fn is_see_through(&self) -> bool {
        crate::data::flag(&self.data_at("see_through"))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.setSeeThrough
    pub fn set_see_through(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("see_through", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.isDefaultBackground
    pub fn is_default_background(&self) -> bool {
        crate::data::flag(&self.data_at("default_background"))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.setDefaultBackground
    pub fn set_default_background(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("default_background", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.getAlignment
    pub fn get_alignment(&self) -> Option<&'static str> {
        self.data_at("alignment").as_str().and_then(|text| crate::enums::text_display_text_alignment::value_of(&text.trim_start_matches("minecraft:").to_ascii_uppercase()))
    }
    /// bukkit: org/bukkit/entity/TextDisplay.setAlignment
    pub fn set_alignment(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("alignment", &format!("{}", crate::data::quoted(&value.to_ascii_lowercase()))))
    }
}

/// `org.bukkit.entity.ThrowableProjectile`.
#[derive(Clone, Debug, PartialEq)]
pub struct ThrowableProjectile(pub Entity);

impl std::ops::Deref for ThrowableProjectile {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ThrowableProjectile {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ThrowableProjectile {
    /// bukkit: org/bukkit/entity/ThrowableProjectile.getItem
    pub fn get_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("Item"))
    }
    /// bukkit: org/bukkit/entity/ThrowableProjectile.setItem
    pub fn set_item(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("Item", &format!("{}", value.to_snbt())))
    }
}

/// `org.bukkit.entity.Trident`.
#[derive(Clone, Debug, PartialEq)]
pub struct Trident(pub Entity);

impl std::ops::Deref for Trident {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Trident {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Trident {
    /// bukkit: org/bukkit/entity/Trident.hasDealtDamage
    pub fn has_dealt_damage(&self) -> bool {
        crate::data::flag(&self.data_at("DealtDamage"))
    }
    /// bukkit: org/bukkit/entity/Trident.setHasDealtDamage
    pub fn set_has_dealt_damage(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("DealtDamage", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Turtle`.
#[derive(Clone, Debug, PartialEq)]
pub struct Turtle(pub Entity);

impl std::ops::Deref for Turtle {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Turtle {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Turtle {
    /// bukkit: org/bukkit/entity/Turtle.hasEgg
    pub fn has_egg(&self) -> bool {
        crate::data::flag(&self.data_at("has_egg"))
    }
    /// bukkit: org/bukkit/entity/Turtle.setHasEgg
    pub fn set_has_egg(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("has_egg", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Turtle.getHome
    pub fn get_home(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("home_pos"))
    }
    /// bukkit: org/bukkit/entity/Turtle.setHome
    pub fn set_home(&self, value: &crate::Location) -> bool {
        self.merge(&crate::data::patch("home_pos", &format!("{}", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()))))
    }
}

/// `org.bukkit.entity.Vex`.
#[derive(Clone, Debug, PartialEq)]
pub struct Vex(pub Entity);

impl std::ops::Deref for Vex {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Vex {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Vex {
    /// bukkit: org/bukkit/entity/Vex.getBound
    pub fn get_bound(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("bound_pos"))
    }
    /// bukkit: org/bukkit/entity/Vex.setBound
    pub fn set_bound(&self, value: &crate::Location) -> bool {
        self.merge(&crate::data::patch("bound_pos", &format!("{}", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()))))
    }
    /// bukkit: org/bukkit/entity/Vex.getLifeTicks
    pub fn get_life_ticks(&self) -> i32 {
        self.data_at("life_ticks").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Vex.setLifeTicks
    pub fn set_life_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("life_ticks", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Vex.getLimitedLifetimeTicks
    pub fn get_limited_lifetime_ticks(&self) -> i32 {
        self.data_at("life_ticks").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Vex.setLimitedLifetimeTicks
    pub fn set_limited_lifetime_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("life_ticks", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Villager`.
#[derive(Clone, Debug, PartialEq)]
pub struct Villager(pub Entity);

impl std::ops::Deref for Villager {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Villager {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Villager {
    /// bukkit: org/bukkit/entity/Villager.getProfession
    pub fn get_profession(&self) -> Option<String> {
        self.data_at("VillagerData.profession").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Villager.setProfession
    pub fn set_profession(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("VillagerData.profession", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Villager.getVillagerType
    pub fn get_villager_type(&self) -> Option<String> {
        self.data_at("VillagerData.type").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Villager.setVillagerType
    pub fn set_villager_type(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("VillagerData.type", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Villager.getVillagerLevel
    pub fn get_villager_level(&self) -> i32 {
        self.data_at("VillagerData.level").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Villager.setVillagerLevel
    pub fn set_villager_level(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("VillagerData.level", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Villager.getVillagerExperience
    pub fn get_villager_experience(&self) -> i32 {
        self.data_at("Xp").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Villager.setVillagerExperience
    pub fn set_villager_experience(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Xp", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Villager.getRestocksToday
    pub fn get_restocks_today(&self) -> i32 {
        self.data_at("RestocksToday").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Villager.setRestocksToday
    pub fn set_restocks_today(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("RestocksToday", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Vindicator`.
#[derive(Clone, Debug, PartialEq)]
pub struct Vindicator(pub Entity);

impl std::ops::Deref for Vindicator {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Vindicator {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Vindicator {
    /// bukkit: org/bukkit/entity/Vindicator.isJohnny
    pub fn is_johnny(&self) -> bool {
        crate::data::flag(&self.data_at("Johnny"))
    }
    /// bukkit: org/bukkit/entity/Vindicator.setJohnny
    pub fn set_johnny(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Johnny", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.WanderingTrader`.
#[derive(Clone, Debug, PartialEq)]
pub struct WanderingTrader(pub Entity);

impl std::ops::Deref for WanderingTrader {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for WanderingTrader {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl WanderingTrader {
    /// bukkit: org/bukkit/entity/WanderingTrader.getDespawnDelay
    pub fn get_despawn_delay(&self) -> i32 {
        self.data_at("DespawnDelay").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/WanderingTrader.setDespawnDelay
    pub fn set_despawn_delay(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("DespawnDelay", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/WanderingTrader.getWanderingTowards
    pub fn get_wandering_towards(&self) -> Option<crate::Location> {
        self.location_of(&self.data_at("wander_target"))
    }
    /// bukkit: org/bukkit/entity/WanderingTrader.setWanderingTowards
    pub fn set_wandering_towards(&self, value: &crate::Location) -> bool {
        self.merge(&crate::data::patch("wander_target", &format!("{}", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()))))
    }
}

/// `org.bukkit.entity.WitherSkull`.
#[derive(Clone, Debug, PartialEq)]
pub struct WitherSkull(pub Entity);

impl std::ops::Deref for WitherSkull {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for WitherSkull {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl WitherSkull {
    /// bukkit: org/bukkit/entity/WitherSkull.isCharged
    pub fn is_charged(&self) -> bool {
        crate::data::flag(&self.data_at("dangerous"))
    }
    /// bukkit: org/bukkit/entity/WitherSkull.setCharged
    pub fn set_charged(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("dangerous", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Wither`.
#[derive(Clone, Debug, PartialEq)]
pub struct Wither(pub Entity);

impl std::ops::Deref for Wither {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Wither {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Wither {
    /// bukkit: org/bukkit/entity/Wither.getInvulnerableTicks
    pub fn get_invulnerable_ticks(&self) -> i32 {
        self.data_at("Invul").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Wither.setInvulnerableTicks
    pub fn set_invulnerable_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Invul", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Wither.getInvulnerabilityTicks
    pub fn get_invulnerability_ticks(&self) -> i32 {
        self.data_at("Invul").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Wither.setInvulnerabilityTicks
    pub fn set_invulnerability_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Invul", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Wolf`.
#[derive(Clone, Debug, PartialEq)]
pub struct Wolf(pub Entity);

impl std::ops::Deref for Wolf {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Wolf {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Wolf {
    /// bukkit: org/bukkit/entity/Wolf.getCollarColor
    pub fn get_collar_color(&self) -> Option<&'static str> {
        self.data_at("CollarColor").as_i64().and_then(|number| crate::enums::dye_color::values().get(usize::try_from(number).ok()?).copied())
    }
    /// bukkit: org/bukkit/entity/Wolf.setCollarColor
    pub fn set_collar_color(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("CollarColor", &format!("{}", crate::enums::dye_color::values().iter().position(|name| *name == value).unwrap_or(0))))
    }
    /// bukkit: org/bukkit/entity/Wolf.getVariant
    pub fn get_variant(&self) -> Option<String> {
        self.data_at("variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Wolf.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("variant", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/Wolf.getSoundVariant
    pub fn get_sound_variant(&self) -> Option<String> {
        self.data_at("sound_variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Wolf.setSoundVariant
    pub fn set_sound_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("sound_variant", &format!("{}", crate::data::quoted(value))))
    }
}

/// `org.bukkit.entity.Zombie`.
#[derive(Clone, Debug, PartialEq)]
pub struct Zombie(pub Entity);

impl std::ops::Deref for Zombie {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Zombie {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Zombie {
    /// bukkit: org/bukkit/entity/Zombie.isBaby
    pub fn is_baby(&self) -> bool {
        crate::data::flag(&self.data_at("IsBaby"))
    }
    /// bukkit: org/bukkit/entity/Zombie.setBaby
    pub fn set_baby(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("IsBaby", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/Zombie.canBreakDoors
    pub fn can_break_doors(&self) -> bool {
        crate::data::flag(&self.data_at("CanBreakDoors"))
    }
    /// bukkit: org/bukkit/entity/Zombie.setCanBreakDoors
    pub fn set_can_break_doors(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("CanBreakDoors", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.Zoglin`.
#[derive(Clone, Debug, PartialEq)]
pub struct Zoglin(pub Entity);

impl std::ops::Deref for Zoglin {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Zoglin {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Zoglin {
    /// bukkit: org/bukkit/entity/Zoglin.isBaby
    pub fn is_baby(&self) -> bool {
        crate::data::flag(&self.data_at("IsBaby"))
    }
    /// bukkit: org/bukkit/entity/Zoglin.setBaby
    pub fn set_baby(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("IsBaby", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.ZombieVillager`.
#[derive(Clone, Debug, PartialEq)]
pub struct ZombieVillager(pub Entity);

impl std::ops::Deref for ZombieVillager {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ZombieVillager {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ZombieVillager {
    /// bukkit: org/bukkit/entity/ZombieVillager.getVillagerProfession
    pub fn get_villager_profession(&self) -> Option<String> {
        self.data_at("VillagerData.profession").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/ZombieVillager.setVillagerProfession
    pub fn set_villager_profession(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("VillagerData.profession", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/ZombieVillager.getVillagerType
    pub fn get_villager_type(&self) -> Option<String> {
        self.data_at("VillagerData.type").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/ZombieVillager.setVillagerType
    pub fn set_villager_type(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("VillagerData.type", &format!("{}", crate::data::quoted(value))))
    }
    /// bukkit: org/bukkit/entity/ZombieVillager.getConversionTime
    pub fn get_conversion_time(&self) -> i32 {
        self.data_at("ConversionTime").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/ZombieVillager.setConversionTime
    pub fn set_conversion_time(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("ConversionTime", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/ZombieVillager.getConversionPlayer
    pub fn get_conversion_player(&self) -> Option<String> {
        crate::data::uuid(&self.data_at("ConversionPlayer"))
    }
    /// bukkit: org/bukkit/entity/ZombieVillager.setConversionPlayer
    pub fn set_conversion_player(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("ConversionPlayer", &format!("{}", crate::data::uuid_snbt(value).unwrap_or_default())))
    }
}

/// `org.bukkit.entity.minecart.CommandMinecart`.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandMinecart(pub Entity);

impl std::ops::Deref for CommandMinecart {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for CommandMinecart {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl CommandMinecart {
    /// bukkit: org/bukkit/entity/minecart/CommandMinecart.getCommand
    pub fn get_command(&self) -> Option<String> {
        self.data_at("Command").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/minecart/CommandMinecart.setCommand
    pub fn set_command(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("Command", &format!("{}", crate::data::quoted(value))))
    }
}

/// `org.bukkit.entity.minecart.ExplosiveMinecart`.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplosiveMinecart(pub Entity);

impl std::ops::Deref for ExplosiveMinecart {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ExplosiveMinecart {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ExplosiveMinecart {
    /// bukkit: org/bukkit/entity/minecart/ExplosiveMinecart.getFuseTicks
    pub fn get_fuse_ticks(&self) -> i32 {
        self.data_at("fuse").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/minecart/ExplosiveMinecart.setFuseTicks
    pub fn set_fuse_ticks(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("fuse", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.minecart.HopperMinecart`.
#[derive(Clone, Debug, PartialEq)]
pub struct HopperMinecart(pub Entity);

impl std::ops::Deref for HopperMinecart {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for HopperMinecart {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl HopperMinecart {
    /// bukkit: org/bukkit/entity/minecart/HopperMinecart.isEnabled
    pub fn is_enabled(&self) -> bool {
        crate::data::flag(&self.data_at("Enabled"))
    }
    /// bukkit: org/bukkit/entity/minecart/HopperMinecart.setEnabled
    pub fn set_enabled(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("Enabled", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.minecart.PoweredMinecart`.
#[derive(Clone, Debug, PartialEq)]
pub struct PoweredMinecart(pub Entity);

impl std::ops::Deref for PoweredMinecart {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for PoweredMinecart {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl PoweredMinecart {
    /// bukkit: org/bukkit/entity/minecart/PoweredMinecart.getFuel
    pub fn get_fuel(&self) -> i32 {
        self.data_at("Fuel").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/minecart/PoweredMinecart.setFuel
    pub fn set_fuel(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("Fuel", &format!("{}", format!("{value}s"))))
    }
}

/// `org.bukkit.entity.ThrownPotion`.
#[derive(Clone, Debug, PartialEq)]
pub struct ThrownPotion(pub Entity);

impl std::ops::Deref for ThrownPotion {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ThrownPotion {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ThrownPotion {
    /// bukkit: org/bukkit/entity/ThrownPotion.getItem
    pub fn get_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("Item"))
    }
    /// bukkit: org/bukkit/entity/ThrownPotion.setItem
    pub fn set_item(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("Item", &format!("{}", value.to_snbt())))
    }
}

/// `org.bukkit.entity.Bat`.
#[derive(Clone, Debug, PartialEq)]
pub struct Bat(pub Entity);

impl std::ops::Deref for Bat {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Bat {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Bat {
    /// bukkit: org/bukkit/entity/Bat.isAwake
    pub fn is_awake(&self) -> bool {
        !crate::data::flag(&self.data_at("BatFlags"))
    }
    /// bukkit: org/bukkit/entity/Bat.setAwake
    pub fn set_awake(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("BatFlags", &format!("{}", !value)))
    }
}

/// `org.bukkit.entity.SizedFireball`.
#[derive(Clone, Debug, PartialEq)]
pub struct SizedFireball(pub Entity);

impl std::ops::Deref for SizedFireball {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for SizedFireball {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl SizedFireball {
    /// bukkit: org/bukkit/entity/SizedFireball.getDisplayItem
    pub fn get_display_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.data_at("Item"))
    }
    /// bukkit: org/bukkit/entity/SizedFireball.setDisplayItem
    pub fn set_display_item(&self, value: &crate::ItemStack) -> bool {
        self.merge(&crate::data::patch("Item", &format!("{}", value.to_snbt())))
    }
}

/// `org.bukkit.entity.ZombieNautilus`.
#[derive(Clone, Debug, PartialEq)]
pub struct ZombieNautilus(pub Entity);

impl std::ops::Deref for ZombieNautilus {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for ZombieNautilus {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl ZombieNautilus {
    /// bukkit: org/bukkit/entity/ZombieNautilus.getVariant
    pub fn get_variant(&self) -> Option<String> {
        self.data_at("variant").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/ZombieNautilus.setVariant
    pub fn set_variant(&self, value: &str) -> bool {
        self.merge(&crate::data::patch("variant", &format!("{}", crate::data::quoted(value))))
    }
}

/// `org.bukkit.entity.Husk`.
#[derive(Clone, Debug, PartialEq)]
pub struct Husk(pub Entity);

impl std::ops::Deref for Husk {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Husk {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

/// `org.bukkit.entity.SkeletonHorse`.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonHorse(pub Entity);

impl std::ops::Deref for SkeletonHorse {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for SkeletonHorse {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl SkeletonHorse {
    /// bukkit: org/bukkit/entity/SkeletonHorse.isTrapped
    pub fn is_trapped(&self) -> bool {
        crate::data::flag(&self.data_at("SkeletonTrap"))
    }
    /// bukkit: org/bukkit/entity/SkeletonHorse.setTrapped
    pub fn set_trapped(&self, value: bool) -> bool {
        self.merge(&crate::data::patch("SkeletonTrap", &format!("{}", value)))
    }
    /// bukkit: org/bukkit/entity/SkeletonHorse.getTrapTime
    pub fn get_trap_time(&self) -> i32 {
        self.data_at("SkeletonTrapTime").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/SkeletonHorse.setTrapTime
    pub fn set_trap_time(&self, value: i32) -> bool {
        self.merge(&crate::data::patch("SkeletonTrapTime", &format!("{}", value)))
    }
}

/// `org.bukkit.entity.CopperGolem`.
#[derive(Clone, Debug, PartialEq)]
pub struct CopperGolem(pub Entity);

impl std::ops::Deref for CopperGolem {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for CopperGolem {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl CopperGolem {
    /// bukkit: org/bukkit/entity/CopperGolem.getWeatheringStateName
    pub fn get_weathering_state_name(&self) -> Option<String> {
        self.data_at("weather_state").as_str().map(str::to_owned)
    }
}
