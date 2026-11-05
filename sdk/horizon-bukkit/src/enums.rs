//! Written by plugin-tools/tools/sdk_gen.py from the Paper API: do not edit.
//!
//! An enum here is its constants' names, as the rest of the SDK passes them.
#![allow(missing_docs, non_upper_case_globals)]

/// `com.destroystokyo.paper.ClientOption$ChatVisibility`.
pub mod client_option_chat_visibility {
    /// bukkit: com/destroystokyo/paper/ClientOption$ChatVisibility.values
    pub fn values() -> &'static [&'static str] {
        &["FULL", "SYSTEM", "HIDDEN", "UNKNOWN"]
    }
    /// bukkit: com/destroystokyo/paper/ClientOption$ChatVisibility.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FULL: &str = "FULL";
    pub const SYSTEM: &str = "SYSTEM";
    pub const HIDDEN: &str = "HIDDEN";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `com.destroystokyo.paper.ClientOption$ParticleVisibility`.
pub mod client_option_particle_visibility {
    /// bukkit: com/destroystokyo/paper/ClientOption$ParticleVisibility.values
    pub fn values() -> &'static [&'static str] {
        &["ALL", "DECREASED", "MINIMAL"]
    }
    /// bukkit: com/destroystokyo/paper/ClientOption$ParticleVisibility.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALL: &str = "ALL";
    pub const DECREASED: &str = "DECREASED";
    pub const MINIMAL: &str = "MINIMAL";
}

/// `com.destroystokyo.paper.block.TargetBlockInfo$FluidMode`.
pub mod target_block_info_fluid_mode {
    /// bukkit: com/destroystokyo/paper/block/TargetBlockInfo$FluidMode.values
    pub fn values() -> &'static [&'static str] {
        &["NEVER", "SOURCE_ONLY", "ALWAYS"]
    }
    /// bukkit: com/destroystokyo/paper/block/TargetBlockInfo$FluidMode.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NEVER: &str = "NEVER";
    pub const SOURCE_ONLY: &str = "SOURCE_ONLY";
    pub const ALWAYS: &str = "ALWAYS";
}

/// `com.destroystokyo.paper.entity.ai.GoalType`.
pub mod goal_type {
    /// bukkit: com/destroystokyo/paper/entity/ai/GoalType.values
    pub fn values() -> &'static [&'static str] {
        &["MOVE", "LOOK", "JUMP", "TARGET", "UNKNOWN_BEHAVIOR"]
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/GoalType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LOOK: &str = "LOOK";
    pub const JUMP: &str = "JUMP";
    pub const TARGET: &str = "TARGET";
    pub const UNKNOWN_BEHAVIOR: &str = "UNKNOWN_BEHAVIOR";
}

/// `com.destroystokyo.paper.entity.villager.ReputationType`.
pub mod reputation_type {
    /// bukkit: com/destroystokyo/paper/entity/villager/ReputationType.values
    pub fn values() -> &'static [&'static str] {
        &["MAJOR_NEGATIVE", "MINOR_NEGATIVE", "MINOR_POSITIVE", "MAJOR_POSITIVE", "TRADING"]
    }
    /// bukkit: com/destroystokyo/paper/entity/villager/ReputationType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const MAJOR_NEGATIVE: &str = "MAJOR_NEGATIVE";
    pub const MINOR_NEGATIVE: &str = "MINOR_NEGATIVE";
    pub const MINOR_POSITIVE: &str = "MINOR_POSITIVE";
    pub const MAJOR_POSITIVE: &str = "MAJOR_POSITIVE";
    pub const TRADING: &str = "TRADING";
}

/// `com.destroystokyo.paper.event.block.AnvilDamagedEvent$DamageState`.
pub mod anvil_damaged_event_damage_state {
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent$DamageState.values
    pub fn values() -> &'static [&'static str] {
        &["FULL", "CHIPPED", "DAMAGED", "BROKEN"]
    }
    /// bukkit: com/destroystokyo/paper/event/block/AnvilDamagedEvent$DamageState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FULL: &str = "FULL";
    pub const CHIPPED: &str = "CHIPPED";
    pub const DAMAGED: &str = "DAMAGED";
    pub const BROKEN: &str = "BROKEN";
}

/// `com.destroystokyo.paper.event.block.TNTPrimeEvent$PrimeReason`.
pub mod tnt_prime_event_prime_reason {
    /// bukkit: com/destroystokyo/paper/event/block/TNTPrimeEvent$PrimeReason.values
    pub fn values() -> &'static [&'static str] {
        &["EXPLOSION", "FIRE", "ITEM", "PROJECTILE", "REDSTONE"]
    }
    /// bukkit: com/destroystokyo/paper/event/block/TNTPrimeEvent$PrimeReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const FIRE: &str = "FIRE";
    pub const ITEM: &str = "ITEM";
    pub const PROJECTILE: &str = "PROJECTILE";
    pub const REDSTONE: &str = "REDSTONE";
}

/// `com.destroystokyo.paper.event.entity.EndermanEscapeEvent$Reason`.
pub mod enderman_escape_event_reason {
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanEscapeEvent$Reason.values
    pub fn values() -> &'static [&'static str] {
        &["RUNAWAY", "INDIRECT", "CRITICAL_HIT", "STARE", "DROWN"]
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanEscapeEvent$Reason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const RUNAWAY: &str = "RUNAWAY";
    pub const INDIRECT: &str = "INDIRECT";
    pub const CRITICAL_HIT: &str = "CRITICAL_HIT";
    pub const STARE: &str = "STARE";
    pub const DROWN: &str = "DROWN";
}

/// `com.destroystokyo.paper.event.player.PlayerArmorChangeEvent$SlotType`.
pub mod player_armor_change_event_slot_type {
    /// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent$SlotType.values
    pub fn values() -> &'static [&'static str] {
        &["HEAD", "CHEST", "LEGS", "FEET"]
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerArmorChangeEvent$SlotType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const HEAD: &str = "HEAD";
    pub const CHEST: &str = "CHEST";
    pub const LEGS: &str = "LEGS";
    pub const FEET: &str = "FEET";
}

/// `com.destroystokyo.paper.event.player.PlayerSetSpawnEvent$Cause`.
pub mod player_set_spawn_event_cause {
    /// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["BED", "RESPAWN_ANCHOR", "PLAYER_RESPAWN", "COMMAND", "PLUGIN", "UNKNOWN"]
    }
    /// bukkit: com/destroystokyo/paper/event/player/PlayerSetSpawnEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BED: &str = "BED";
    pub const RESPAWN_ANCHOR: &str = "RESPAWN_ANCHOR";
    pub const PLAYER_RESPAWN: &str = "PLAYER_RESPAWN";
    pub const COMMAND: &str = "COMMAND";
    pub const PLUGIN: &str = "PLUGIN";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `com.destroystokyo.paper.event.server.GS4QueryEvent$QueryType`.
pub mod gs_4_query_event_query_type {
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryType.values
    pub fn values() -> &'static [&'static str] {
        &["BASIC", "FULL"]
    }
    /// bukkit: com/destroystokyo/paper/event/server/GS4QueryEvent$QueryType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BASIC: &str = "BASIC";
    pub const FULL: &str = "FULL";
}

/// `io.papermc.paper.ServerBuildInfo$StringRepresentation`.
pub mod server_build_info_string_representation {
    /// bukkit: io/papermc/paper/ServerBuildInfo$StringRepresentation.values
    pub fn values() -> &'static [&'static str] {
        &["VERSION_SIMPLE", "VERSION_FULL"]
    }
    /// bukkit: io/papermc/paper/ServerBuildInfo$StringRepresentation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const VERSION_SIMPLE: &str = "VERSION_SIMPLE";
    pub const VERSION_FULL: &str = "VERSION_FULL";
}

/// `io.papermc.paper.advancement.AdvancementDisplay$Frame`.
pub mod advancement_display_frame {
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay$Frame.values
    pub fn values() -> &'static [&'static str] {
        &["CHALLENGE", "GOAL", "TASK"]
    }
    /// bukkit: io/papermc/paper/advancement/AdvancementDisplay$Frame.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CHALLENGE: &str = "CHALLENGE";
    pub const GOAL: &str = "GOAL";
    pub const TASK: &str = "TASK";
}

/// `io.papermc.paper.command.brigadier.CommandRegistrationFlag`.
pub mod command_registration_flag {
    /// bukkit: io/papermc/paper/command/brigadier/CommandRegistrationFlag.values
    pub fn values() -> &'static [&'static str] {
        &["FLATTEN_ALIASES", "SERVER_ONLY"]
    }
    /// bukkit: io/papermc/paper/command/brigadier/CommandRegistrationFlag.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FLATTEN_ALIASES: &str = "FLATTEN_ALIASES";
    pub const SERVER_ONLY: &str = "SERVER_ONLY";
}

/// `io.papermc.paper.command.brigadier.argument.predicate.BlockInWorldPredicate$Result`.
pub mod block_in_world_predicate_result {
    /// bukkit: io/papermc/paper/command/brigadier/argument/predicate/BlockInWorldPredicate$Result.values
    pub fn values() -> &'static [&'static str] {
        &["TRUE", "FALSE", "UNLOADED_CHUNK"]
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/predicate/BlockInWorldPredicate$Result.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UNLOADED_CHUNK: &str = "UNLOADED_CHUNK";
}

/// `io.papermc.paper.datacomponent.item.SwingAnimation$Animation`.
pub mod swing_animation_animation {
    /// bukkit: io/papermc/paper/datacomponent/item/SwingAnimation$Animation.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "WHACK", "STAB"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/SwingAnimation$Animation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const WHACK: &str = "WHACK";
    pub const STAB: &str = "STAB";
}

/// `io.papermc.paper.datacomponent.item.consumable.ItemUseAnimation`.
pub mod item_use_animation {
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ItemUseAnimation.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "EAT", "DRINK", "BLOCK", "BOW", "TRIDENT", "CROSSBOW", "SPYGLASS", "TOOT_HORN", "BRUSH", "BUNDLE", "SPEAR"]
    }
    /// bukkit: io/papermc/paper/datacomponent/item/consumable/ItemUseAnimation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const EAT: &str = "EAT";
    pub const DRINK: &str = "DRINK";
    pub const BLOCK: &str = "BLOCK";
    pub const BOW: &str = "BOW";
    pub const TRIDENT: &str = "TRIDENT";
    pub const CROSSBOW: &str = "CROSSBOW";
    pub const SPYGLASS: &str = "SPYGLASS";
    pub const TOOT_HORN: &str = "TOOT_HORN";
    pub const BRUSH: &str = "BRUSH";
    pub const BUNDLE: &str = "BUNDLE";
    pub const SPEAR: &str = "SPEAR";
}

/// `io.papermc.paper.datapack.Datapack$Compatibility`.
pub mod datapack_compatibility {
    /// bukkit: io/papermc/paper/datapack/Datapack$Compatibility.values
    pub fn values() -> &'static [&'static str] {
        &["TOO_OLD", "TOO_NEW", "COMPATIBLE"]
    }
    /// bukkit: io/papermc/paper/datapack/Datapack$Compatibility.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TOO_OLD: &str = "TOO_OLD";
    pub const TOO_NEW: &str = "TOO_NEW";
    pub const COMPATIBLE: &str = "COMPATIBLE";
}

/// `io.papermc.paper.datapack.Datapack$Position`.
pub mod datapack_position {
    /// bukkit: io/papermc/paper/datapack/Datapack$Position.values
    pub fn values() -> &'static [&'static str] {
        &["TOP", "BOTTOM"]
    }
    /// bukkit: io/papermc/paper/datapack/Datapack$Position.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TOP: &str = "TOP";
    pub const BOTTOM: &str = "BOTTOM";
}

/// `io.papermc.paper.enchantments.EnchantmentRarity`.
pub mod enchantment_rarity {
    /// bukkit: io/papermc/paper/enchantments/EnchantmentRarity.values
    pub fn values() -> &'static [&'static str] {
        &["COMMON", "UNCOMMON", "RARE", "VERY_RARE"]
    }
    /// bukkit: io/papermc/paper/enchantments/EnchantmentRarity.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMON: &str = "COMMON";
    pub const UNCOMMON: &str = "UNCOMMON";
    pub const RARE: &str = "RARE";
    pub const VERY_RARE: &str = "VERY_RARE";
}

/// `io.papermc.paper.entity.EntitySerializationFlag`.
pub mod entity_serialization_flag {
    /// bukkit: io/papermc/paper/entity/EntitySerializationFlag.values
    pub fn values() -> &'static [&'static str] {
        &["FORCE", "MISC", "PASSENGERS", "PLAYER"]
    }
    /// bukkit: io/papermc/paper/entity/EntitySerializationFlag.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FORCE: &str = "FORCE";
    pub const MISC: &str = "MISC";
    pub const PASSENGERS: &str = "PASSENGERS";
    pub const PLAYER: &str = "PLAYER";
}

/// `io.papermc.paper.entity.LookAnchor`.
pub mod look_anchor {
    /// bukkit: io/papermc/paper/entity/LookAnchor.values
    pub fn values() -> &'static [&'static str] {
        &["FEET", "EYES"]
    }
    /// bukkit: io/papermc/paper/entity/LookAnchor.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FEET: &str = "FEET";
    pub const EYES: &str = "EYES";
}

/// `io.papermc.paper.entity.RemovalReason`.
pub mod removal_reason {
    /// bukkit: io/papermc/paper/entity/RemovalReason.values
    pub fn values() -> &'static [&'static str] {
        &["KILLED", "DISCARDED", "UNLOADED_TO_CHUNK", "UNLOADED_WITH_PLAYER", "CHANGED_DIMENSION"]
    }
    /// bukkit: io/papermc/paper/entity/RemovalReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const KILLED: &str = "KILLED";
    pub const DISCARDED: &str = "DISCARDED";
    pub const UNLOADED_TO_CHUNK: &str = "UNLOADED_TO_CHUNK";
    pub const UNLOADED_WITH_PLAYER: &str = "UNLOADED_WITH_PLAYER";
    pub const CHANGED_DIMENSION: &str = "CHANGED_DIMENSION";
}

/// `io.papermc.paper.entity.TeleportFlag$EntityState`.
pub mod teleport_flag_entity_state {
    /// bukkit: io/papermc/paper/entity/TeleportFlag$EntityState.values
    pub fn values() -> &'static [&'static str] {
        &["RETAIN_PASSENGERS", "RETAIN_VEHICLE", "RETAIN_OPEN_INVENTORY"]
    }
    /// bukkit: io/papermc/paper/entity/TeleportFlag$EntityState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const RETAIN_PASSENGERS: &str = "RETAIN_PASSENGERS";
    pub const RETAIN_VEHICLE: &str = "RETAIN_VEHICLE";
    pub const RETAIN_OPEN_INVENTORY: &str = "RETAIN_OPEN_INVENTORY";
}

/// `io.papermc.paper.entity.TeleportFlag$Relative`.
pub mod teleport_flag_relative {
    /// bukkit: io/papermc/paper/entity/TeleportFlag$Relative.values
    pub fn values() -> &'static [&'static str] {
        &["VELOCITY_X", "VELOCITY_Y", "VELOCITY_Z", "VELOCITY_ROTATION", "X", "Y", "Z", "YAW", "PITCH"]
    }
    /// bukkit: io/papermc/paper/entity/TeleportFlag$Relative.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const VELOCITY_X: &str = "VELOCITY_X";
    pub const VELOCITY_Y: &str = "VELOCITY_Y";
    pub const VELOCITY_Z: &str = "VELOCITY_Z";
    pub const VELOCITY_ROTATION: &str = "VELOCITY_ROTATION";
    pub const X: &str = "X";
    pub const Y: &str = "Y";
    pub const Z: &str = "Z";
    pub const YAW: &str = "YAW";
    pub const PITCH: &str = "PITCH";
}

/// `io.papermc.paper.event.entity.EntityBreakEvent$RemoveCause`.
pub mod entity_break_event_remove_cause {
    /// bukkit: io/papermc/paper/event/entity/EntityBreakEvent$RemoveCause.values
    pub fn values() -> &'static [&'static str] {
        &["ENTITY", "EXPLOSION", "OBSTRUCTION", "PHYSICS", "DEFAULT"]
    }
    /// bukkit: io/papermc/paper/event/entity/EntityBreakEvent$RemoveCause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ENTITY: &str = "ENTITY";
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const OBSTRUCTION: &str = "OBSTRUCTION";
    pub const PHYSICS: &str = "PHYSICS";
    pub const DEFAULT: &str = "DEFAULT";
}

/// `io.papermc.paper.event.entity.EntityKnockbackEvent$Cause`.
pub mod entity_knockback_event_cause {
    /// bukkit: io/papermc/paper/event/entity/EntityKnockbackEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["DAMAGE", "ENTITY_ATTACK", "EXPLOSION", "SHIELD_BLOCK", "SWEEP_ATTACK", "PUSH", "UNKNOWN"]
    }
    /// bukkit: io/papermc/paper/event/entity/EntityKnockbackEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DAMAGE: &str = "DAMAGE";
    pub const ENTITY_ATTACK: &str = "ENTITY_ATTACK";
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const SHIELD_BLOCK: &str = "SHIELD_BLOCK";
    pub const SWEEP_ATTACK: &str = "SWEEP_ATTACK";
    pub const PUSH: &str = "PUSH";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `io.papermc.paper.event.inventory.PlayerBundleItemSelectEvent$Direction`.
pub mod player_bundle_item_select_event_direction {
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent$Direction.values
    pub fn values() -> &'static [&'static str] {
        &["FORWARD", "BACKWARD", "UNKNOWN"]
    }
    /// bukkit: io/papermc/paper/event/inventory/PlayerBundleItemSelectEvent$Direction.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FORWARD: &str = "FORWARD";
    pub const BACKWARD: &str = "BACKWARD";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `io.papermc.paper.event.player.PlayerBedFailEnterEvent$FailReason`.
pub mod player_bed_fail_enter_event_fail_reason {
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent$FailReason.values
    pub fn values() -> &'static [&'static str] {
        &["NOT_POSSIBLE_HERE", "NOT_POSSIBLE_NOW", "TOO_FAR_AWAY", "OBSTRUCTED", "OTHER_PROBLEM", "NOT_SAFE", "EXPLOSION"]
    }
    /// bukkit: io/papermc/paper/event/player/PlayerBedFailEnterEvent$FailReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NOT_POSSIBLE_HERE: &str = "NOT_POSSIBLE_HERE";
    pub const NOT_POSSIBLE_NOW: &str = "NOT_POSSIBLE_NOW";
    pub const TOO_FAR_AWAY: &str = "TOO_FAR_AWAY";
    pub const OBSTRUCTED: &str = "OBSTRUCTED";
    pub const OTHER_PROBLEM: &str = "OTHER_PROBLEM";
    pub const NOT_SAFE: &str = "NOT_SAFE";
    pub const EXPLOSION: &str = "EXPLOSION";
}

/// `io.papermc.paper.event.player.PlayerFailMoveEvent$FailReason`.
pub mod player_fail_move_event_fail_reason {
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent$FailReason.values
    pub fn values() -> &'static [&'static str] {
        &["MOVED_INTO_UNLOADED_CHUNK", "MOVED_TOO_QUICKLY", "MOVED_WRONGLY", "CLIPPED_INTO_BLOCK"]
    }
    /// bukkit: io/papermc/paper/event/player/PlayerFailMoveEvent$FailReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const MOVED_INTO_UNLOADED_CHUNK: &str = "MOVED_INTO_UNLOADED_CHUNK";
    pub const MOVED_TOO_QUICKLY: &str = "MOVED_TOO_QUICKLY";
    pub const MOVED_WRONGLY: &str = "MOVED_WRONGLY";
    pub const CLIPPED_INTO_BLOCK: &str = "CLIPPED_INTO_BLOCK";
}

/// `io.papermc.paper.event.player.PlayerItemFrameChangeEvent$ItemFrameChangeAction`.
pub mod player_item_frame_change_event_item_frame_change_action {
    /// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent$ItemFrameChangeAction.values
    pub fn values() -> &'static [&'static str] {
        &["PLACE", "REMOVE", "ROTATE"]
    }
    /// bukkit: io/papermc/paper/event/player/PlayerItemFrameChangeEvent$ItemFrameChangeAction.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PLACE: &str = "PLACE";
    pub const REMOVE: &str = "REMOVE";
    pub const ROTATE: &str = "ROTATE";
}

/// `io.papermc.paper.event.player.PlayerLecternPageChangeEvent$PageChangeDirection`.
pub mod player_lectern_page_change_event_page_change_direction {
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent$PageChangeDirection.values
    pub fn values() -> &'static [&'static str] {
        &["LEFT", "RIGHT"]
    }
    /// bukkit: io/papermc/paper/event/player/PlayerLecternPageChangeEvent$PageChangeDirection.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LEFT: &str = "LEFT";
    pub const RIGHT: &str = "RIGHT";
}

/// `io.papermc.paper.event.player.PlayerOpenSignEvent$Cause`.
pub mod player_open_sign_event_cause {
    /// bukkit: io/papermc/paper/event/player/PlayerOpenSignEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["PLACE", "INTERACT", "PLUGIN", "UNKNOWN"]
    }
    /// bukkit: io/papermc/paper/event/player/PlayerOpenSignEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PLACE: &str = "PLACE";
    pub const INTERACT: &str = "INTERACT";
    pub const PLUGIN: &str = "PLUGIN";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `io.papermc.paper.event.server.ServerResourcesReloadedEvent$Cause`.
pub mod server_resources_reloaded_event_cause {
    /// bukkit: io/papermc/paper/event/server/ServerResourcesReloadedEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["COMMAND", "PLUGIN"]
    }
    /// bukkit: io/papermc/paper/event/server/ServerResourcesReloadedEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMAND: &str = "COMMAND";
    pub const PLUGIN: &str = "PLUGIN";
}

/// `io.papermc.paper.event.server.WhitelistStateUpdateEvent$WhitelistStatus`.
pub mod whitelist_state_update_event_whitelist_status {
    /// bukkit: io/papermc/paper/event/server/WhitelistStateUpdateEvent$WhitelistStatus.values
    pub fn values() -> &'static [&'static str] {
        &["ADDED", "REMOVED"]
    }
    /// bukkit: io/papermc/paper/event/server/WhitelistStateUpdateEvent$WhitelistStatus.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ADDED: &str = "ADDED";
    pub const REMOVED: &str = "REMOVED";
}

/// `io.papermc.paper.event.world.border.WorldBorderBoundsChangeEvent$Type`.
pub mod world_border_bounds_change_event_type {
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent$Type.values
    pub fn values() -> &'static [&'static str] {
        &["STARTED_MOVE", "INSTANT_MOVE"]
    }
    /// bukkit: io/papermc/paper/event/world/border/WorldBorderBoundsChangeEvent$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const STARTED_MOVE: &str = "STARTED_MOVE";
    pub const INSTANT_MOVE: &str = "INSTANT_MOVE";
}

/// `io.papermc.paper.inventory.ItemRarity`.
pub mod item_rarity {
    /// bukkit: io/papermc/paper/inventory/ItemRarity.values
    pub fn values() -> &'static [&'static str] {
        &["COMMON", "UNCOMMON", "RARE", "EPIC"]
    }
    /// bukkit: io/papermc/paper/inventory/ItemRarity.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMON: &str = "COMMON";
    pub const UNCOMMON: &str = "UNCOMMON";
    pub const RARE: &str = "RARE";
    pub const EPIC: &str = "EPIC";
}

/// `io.papermc.paper.item.MapPostProcessing`.
pub mod map_post_processing {
    /// bukkit: io/papermc/paper/item/MapPostProcessing.values
    pub fn values() -> &'static [&'static str] {
        &["LOCK", "SCALE"]
    }
    /// bukkit: io/papermc/paper/item/MapPostProcessing.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LOCK: &str = "LOCK";
    pub const SCALE: &str = "SCALE";
}

/// `io.papermc.paper.plugin.lifecycle.event.registrar.ReloadableRegistrarEvent$Cause`.
pub mod reloadable_registrar_event_cause {
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/registrar/ReloadableRegistrarEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["INITIAL", "RELOAD"]
    }
    /// bukkit: io/papermc/paper/plugin/lifecycle/event/registrar/ReloadableRegistrarEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const INITIAL: &str = "INITIAL";
    pub const RELOAD: &str = "RELOAD";
}

/// `io.papermc.paper.raytracing.BlockCollisionMode`.
pub mod block_collision_mode {
    /// bukkit: io/papermc/paper/raytracing/BlockCollisionMode.values
    pub fn values() -> &'static [&'static str] {
        &["COLLIDER", "OUTLINE", "VISUAL"]
    }
    /// bukkit: io/papermc/paper/raytracing/BlockCollisionMode.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COLLIDER: &str = "COLLIDER";
    pub const OUTLINE: &str = "OUTLINE";
    pub const VISUAL: &str = "VISUAL";
}

/// `io.papermc.paper.raytracing.RayTraceTarget`.
pub mod ray_trace_target {
    /// bukkit: io/papermc/paper/raytracing/RayTraceTarget.values
    pub fn values() -> &'static [&'static str] {
        &["ENTITY", "BLOCK"]
    }
    /// bukkit: io/papermc/paper/raytracing/RayTraceTarget.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ENTITY: &str = "ENTITY";
    pub const BLOCK: &str = "BLOCK";
}

/// `io.papermc.paper.registry.data.ChickenVariantRegistryEntry$Model`.
pub mod chicken_variant_registry_entry_model {
    /// bukkit: io/papermc/paper/registry/data/ChickenVariantRegistryEntry$Model.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "COLD"]
    }
    /// bukkit: io/papermc/paper/registry/data/ChickenVariantRegistryEntry$Model.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const COLD: &str = "COLD";
}

/// `io.papermc.paper.registry.data.CowVariantRegistryEntry$Model`.
pub mod cow_variant_registry_entry_model {
    /// bukkit: io/papermc/paper/registry/data/CowVariantRegistryEntry$Model.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "COLD", "WARM"]
    }
    /// bukkit: io/papermc/paper/registry/data/CowVariantRegistryEntry$Model.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const COLD: &str = "COLD";
    pub const WARM: &str = "WARM";
}

/// `io.papermc.paper.registry.data.PigVariantRegistryEntry$Model`.
pub mod pig_variant_registry_entry_model {
    /// bukkit: io/papermc/paper/registry/data/PigVariantRegistryEntry$Model.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "COLD"]
    }
    /// bukkit: io/papermc/paper/registry/data/PigVariantRegistryEntry$Model.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const COLD: &str = "COLD";
}

/// `io.papermc.paper.registry.data.ZombieNautilusVariantRegistryEntry$Model`.
pub mod zombie_nautilus_variant_registry_entry_model {
    /// bukkit: io/papermc/paper/registry/data/ZombieNautilusVariantRegistryEntry$Model.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "WARM"]
    }
    /// bukkit: io/papermc/paper/registry/data/ZombieNautilusVariantRegistryEntry$Model.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const WARM: &str = "WARM";
}

/// `io.papermc.paper.registry.data.dialog.DialogBase$DialogAfterAction`.
pub mod dialog_base_dialog_after_action {
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase$DialogAfterAction.values
    pub fn values() -> &'static [&'static str] {
        &["CLOSE", "NONE", "WAIT_FOR_RESPONSE"]
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase$DialogAfterAction.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CLOSE: &str = "CLOSE";
    pub const NONE: &str = "NONE";
    pub const WAIT_FOR_RESPONSE: &str = "WAIT_FOR_RESPONSE";
}

/// `io.papermc.paper.threadedregions.scheduler.ScheduledTask$CancelledState`.
pub mod scheduled_task_cancelled_state {
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask$CancelledState.values
    pub fn values() -> &'static [&'static str] {
        &["CANCELLED_BY_CALLER", "CANCELLED_ALREADY", "RUNNING", "ALREADY_EXECUTED", "NEXT_RUNS_CANCELLED", "NEXT_RUNS_CANCELLED_ALREADY"]
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask$CancelledState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CANCELLED_BY_CALLER: &str = "CANCELLED_BY_CALLER";
    pub const CANCELLED_ALREADY: &str = "CANCELLED_ALREADY";
    pub const RUNNING: &str = "RUNNING";
    pub const ALREADY_EXECUTED: &str = "ALREADY_EXECUTED";
    pub const NEXT_RUNS_CANCELLED: &str = "NEXT_RUNS_CANCELLED";
    pub const NEXT_RUNS_CANCELLED_ALREADY: &str = "NEXT_RUNS_CANCELLED_ALREADY";
}

/// `io.papermc.paper.threadedregions.scheduler.ScheduledTask$ExecutionState`.
pub mod scheduled_task_execution_state {
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask$ExecutionState.values
    pub fn values() -> &'static [&'static str] {
        &["IDLE", "RUNNING", "FINISHED", "CANCELLED", "CANCELLED_RUNNING"]
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask$ExecutionState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const IDLE: &str = "IDLE";
    pub const RUNNING: &str = "RUNNING";
    pub const FINISHED: &str = "FINISHED";
    pub const CANCELLED: &str = "CANCELLED";
    pub const CANCELLED_RUNNING: &str = "CANCELLED_RUNNING";
}

/// `io.papermc.paper.world.MoonPhase`.
pub mod moon_phase {
    /// bukkit: io/papermc/paper/world/MoonPhase.values
    pub fn values() -> &'static [&'static str] {
        &["FULL_MOON", "WANING_GIBBOUS", "LAST_QUARTER", "WANING_CRESCENT", "NEW_MOON", "WAXING_CRESCENT", "FIRST_QUARTER", "WAXING_GIBBOUS"]
    }
    /// bukkit: io/papermc/paper/world/MoonPhase.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FULL_MOON: &str = "FULL_MOON";
    pub const WANING_GIBBOUS: &str = "WANING_GIBBOUS";
    pub const LAST_QUARTER: &str = "LAST_QUARTER";
    pub const WANING_CRESCENT: &str = "WANING_CRESCENT";
    pub const NEW_MOON: &str = "NEW_MOON";
    pub const WAXING_CRESCENT: &str = "WAXING_CRESCENT";
    pub const FIRST_QUARTER: &str = "FIRST_QUARTER";
    pub const WAXING_GIBBOUS: &str = "WAXING_GIBBOUS";
}

/// `io.papermc.paper.world.WeatheringCopperState`.
pub mod weathering_copper_state {
    /// bukkit: io/papermc/paper/world/WeatheringCopperState.values
    pub fn values() -> &'static [&'static str] {
        &["UNAFFECTED", "EXPOSED", "WEATHERED", "OXIDIZED"]
    }
    /// bukkit: io/papermc/paper/world/WeatheringCopperState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UNAFFECTED: &str = "UNAFFECTED";
    pub const EXPOSED: &str = "EXPOSED";
    pub const WEATHERED: &str = "WEATHERED";
    pub const OXIDIZED: &str = "OXIDIZED";
}

/// `io.papermc.paper.world.chunk.ChunkGenerationState`.
pub mod chunk_generation_state {
    /// bukkit: io/papermc/paper/world/chunk/ChunkGenerationState.values
    pub fn values() -> &'static [&'static str] {
        &["NOT_GENERATED", "PARTIAL", "OUTDATED", "GENERATED"]
    }
    /// bukkit: io/papermc/paper/world/chunk/ChunkGenerationState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NOT_GENERATED: &str = "NOT_GENERATED";
    pub const PARTIAL: &str = "PARTIAL";
    pub const OUTDATED: &str = "OUTDATED";
    pub const GENERATED: &str = "GENERATED";
}

/// `org.bukkit.Art`.
pub mod art {
    /// bukkit: org/bukkit/Art.values
    pub fn values() -> &'static [&'static str] {
        &["ALBAN", "AZTEC", "AZTEC2", "BACKYARD", "BAROQUE", "BOMB", "BOUQUET", "BURNING_SKULL", "BUST", "CAVEBIRD", "CHANGING", "COTAN", "COURBET", "CREEBET", "DENNIS", "DONKEY_KONG", "EARTH", "ENDBOSS", "FERN", "FIGHTERS", "FINDING", "FIRE", "GRAHAM", "HUMBLE", "KEBAB", "LOWMIST", "MATCH", "MEDITATIVE", "ORB", "OWLEMONS", "PASSAGE", "PIGSCENE", "PLANT", "POINTER", "POND", "POOL", "PRAIRIE_RIDE", "SEA", "SKELETON", "SKULL_AND_ROSES", "STAGE", "SUNFLOWERS", "SUNSET", "TIDES", "UNPACKED", "VOID", "WANDERER", "WASTELAND", "WATER", "WIND", "WITHER"]
    }
    /// bukkit: org/bukkit/Art.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALBAN: &str = "minecraft:alban";
    pub const AZTEC: &str = "minecraft:aztec";
    pub const AZTEC2: &str = "minecraft:aztec2";
    pub const BACKYARD: &str = "minecraft:backyard";
    pub const BAROQUE: &str = "minecraft:baroque";
    pub const BOMB: &str = "minecraft:bomb";
    pub const BOUQUET: &str = "minecraft:bouquet";
    pub const BURNING_SKULL: &str = "minecraft:burning_skull";
    pub const BUST: &str = "minecraft:bust";
    pub const CAVEBIRD: &str = "minecraft:cavebird";
    pub const CHANGING: &str = "minecraft:changing";
    pub const COTAN: &str = "minecraft:cotan";
    pub const COURBET: &str = "minecraft:courbet";
    pub const CREEBET: &str = "minecraft:creebet";
    pub const DENNIS: &str = "minecraft:dennis";
    pub const DONKEY_KONG: &str = "minecraft:donkey_kong";
    pub const EARTH: &str = "minecraft:earth";
    pub const ENDBOSS: &str = "minecraft:endboss";
    pub const FERN: &str = "minecraft:fern";
    pub const FIGHTERS: &str = "minecraft:fighters";
    pub const FINDING: &str = "minecraft:finding";
    pub const FIRE: &str = "minecraft:fire";
    pub const GRAHAM: &str = "minecraft:graham";
    pub const HUMBLE: &str = "minecraft:humble";
    pub const KEBAB: &str = "minecraft:kebab";
    pub const LOWMIST: &str = "minecraft:lowmist";
    pub const MEDITATIVE: &str = "minecraft:meditative";
    pub const ORB: &str = "minecraft:orb";
    pub const OWLEMONS: &str = "minecraft:owlemons";
    pub const PASSAGE: &str = "minecraft:passage";
    pub const PIGSCENE: &str = "minecraft:pigscene";
    pub const PLANT: &str = "minecraft:plant";
    pub const POINTER: &str = "minecraft:pointer";
    pub const POND: &str = "minecraft:pond";
    pub const POOL: &str = "minecraft:pool";
    pub const PRAIRIE_RIDE: &str = "minecraft:prairie_ride";
    pub const SEA: &str = "minecraft:sea";
    pub const SKELETON: &str = "minecraft:skeleton";
    pub const SKULL_AND_ROSES: &str = "minecraft:skull_and_roses";
    pub const STAGE: &str = "minecraft:stage";
    pub const SUNFLOWERS: &str = "minecraft:sunflowers";
    pub const SUNSET: &str = "minecraft:sunset";
    pub const TIDES: &str = "minecraft:tides";
    pub const UNPACKED: &str = "minecraft:unpacked";
    pub const VOID: &str = "minecraft:void";
    pub const WANDERER: &str = "minecraft:wanderer";
    pub const WASTELAND: &str = "minecraft:wasteland";
    pub const WATER: &str = "minecraft:water";
    pub const WIND: &str = "minecraft:wind";
    pub const WITHER: &str = "minecraft:wither";
}

/// `org.bukkit.Axis`.
pub mod axis {
    /// bukkit: org/bukkit/Axis.values
    pub fn values() -> &'static [&'static str] {
        &["X", "Y", "Z"]
    }
    /// bukkit: org/bukkit/Axis.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const X: &str = "X";
    pub const Y: &str = "Y";
    pub const Z: &str = "Z";
}

/// `org.bukkit.BanList$Type`.
pub mod ban_list_type {
    /// bukkit: org/bukkit/BanList$Type.values
    pub fn values() -> &'static [&'static str] {
        &["NAME", "IP", "PROFILE"]
    }
    /// bukkit: org/bukkit/BanList$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NAME: &str = "NAME";
    pub const IP: &str = "IP";
    pub const PROFILE: &str = "PROFILE";
}

/// `org.bukkit.ChatColor`.
pub mod chat_color {
    /// bukkit: org/bukkit/ChatColor.values
    pub fn values() -> &'static [&'static str] {
        &["BLACK", "DARK_BLUE", "DARK_GREEN", "DARK_AQUA", "DARK_RED", "DARK_PURPLE", "GOLD", "GRAY", "DARK_GRAY", "BLUE", "GREEN", "AQUA", "RED", "LIGHT_PURPLE", "YELLOW", "WHITE", "MAGIC", "BOLD", "STRIKETHROUGH", "UNDERLINE", "ITALIC", "RESET"]
    }
    /// bukkit: org/bukkit/ChatColor.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BLACK: &str = "BLACK";
    pub const DARK_BLUE: &str = "DARK_BLUE";
    pub const DARK_GREEN: &str = "DARK_GREEN";
    pub const DARK_AQUA: &str = "DARK_AQUA";
    pub const DARK_RED: &str = "DARK_RED";
    pub const DARK_PURPLE: &str = "DARK_PURPLE";
    pub const GOLD: &str = "GOLD";
    pub const GRAY: &str = "GRAY";
    pub const DARK_GRAY: &str = "DARK_GRAY";
    pub const BLUE: &str = "BLUE";
    pub const GREEN: &str = "GREEN";
    pub const AQUA: &str = "AQUA";
    pub const RED: &str = "RED";
    pub const LIGHT_PURPLE: &str = "LIGHT_PURPLE";
    pub const YELLOW: &str = "YELLOW";
    pub const WHITE: &str = "WHITE";
    pub const MAGIC: &str = "MAGIC";
    pub const BOLD: &str = "BOLD";
    pub const STRIKETHROUGH: &str = "STRIKETHROUGH";
    pub const UNDERLINE: &str = "UNDERLINE";
    pub const ITALIC: &str = "ITALIC";
    pub const RESET: &str = "RESET";
}

/// `org.bukkit.Chunk$LoadLevel`.
pub mod chunk_load_level {
    /// bukkit: org/bukkit/Chunk$LoadLevel.values
    pub fn values() -> &'static [&'static str] {
        &["INACCESSIBLE", "BORDER", "TICKING", "ENTITY_TICKING", "UNLOADED"]
    }
    /// bukkit: org/bukkit/Chunk$LoadLevel.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const INACCESSIBLE: &str = "INACCESSIBLE";
    pub const BORDER: &str = "BORDER";
    pub const TICKING: &str = "TICKING";
    pub const ENTITY_TICKING: &str = "ENTITY_TICKING";
    pub const UNLOADED: &str = "UNLOADED";
}

/// `org.bukkit.CoalType`.
pub mod coal_type {
    /// bukkit: org/bukkit/CoalType.values
    pub fn values() -> &'static [&'static str] {
        &["COAL", "CHARCOAL"]
    }
    /// bukkit: org/bukkit/CoalType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COAL: &str = "COAL";
    pub const CHARCOAL: &str = "CHARCOAL";
}

/// `org.bukkit.CropState`.
pub mod crop_state {
    /// bukkit: org/bukkit/CropState.values
    pub fn values() -> &'static [&'static str] {
        &["SEEDED", "GERMINATED", "VERY_SMALL", "SMALL", "MEDIUM", "TALL", "VERY_TALL", "RIPE"]
    }
    /// bukkit: org/bukkit/CropState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SEEDED: &str = "SEEDED";
    pub const GERMINATED: &str = "GERMINATED";
    pub const VERY_SMALL: &str = "VERY_SMALL";
    pub const SMALL: &str = "SMALL";
    pub const MEDIUM: &str = "MEDIUM";
    pub const TALL: &str = "TALL";
    pub const VERY_TALL: &str = "VERY_TALL";
    pub const RIPE: &str = "RIPE";
}

/// `org.bukkit.Difficulty`.
pub mod difficulty {
    /// bukkit: org/bukkit/Difficulty.values
    pub fn values() -> &'static [&'static str] {
        &["PEACEFUL", "EASY", "NORMAL", "HARD"]
    }
    /// bukkit: org/bukkit/Difficulty.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PEACEFUL: &str = "PEACEFUL";
    pub const EASY: &str = "EASY";
    pub const NORMAL: &str = "NORMAL";
    pub const HARD: &str = "HARD";
}

/// `org.bukkit.DyeColor`.
pub mod dye_color {
    /// bukkit: org/bukkit/DyeColor.values
    pub fn values() -> &'static [&'static str] {
        &["WHITE", "ORANGE", "MAGENTA", "LIGHT_BLUE", "YELLOW", "LIME", "PINK", "GRAY", "LIGHT_GRAY", "CYAN", "PURPLE", "BLUE", "BROWN", "GREEN", "RED", "BLACK"]
    }
    /// bukkit: org/bukkit/DyeColor.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const WHITE: &str = "WHITE";
    pub const ORANGE: &str = "ORANGE";
    pub const MAGENTA: &str = "MAGENTA";
    pub const LIGHT_BLUE: &str = "LIGHT_BLUE";
    pub const YELLOW: &str = "YELLOW";
    pub const LIME: &str = "LIME";
    pub const PINK: &str = "PINK";
    pub const GRAY: &str = "GRAY";
    pub const LIGHT_GRAY: &str = "LIGHT_GRAY";
    pub const CYAN: &str = "CYAN";
    pub const PURPLE: &str = "PURPLE";
    pub const BLUE: &str = "BLUE";
    pub const BROWN: &str = "BROWN";
    pub const GREEN: &str = "GREEN";
    pub const RED: &str = "RED";
    pub const BLACK: &str = "BLACK";
}

/// `org.bukkit.Effect$Type`.
pub mod effect_type {
    /// bukkit: org/bukkit/Effect$Type.values
    pub fn values() -> &'static [&'static str] {
        &["SOUND", "VISUAL"]
    }
    /// bukkit: org/bukkit/Effect$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SOUND: &str = "SOUND";
    pub const VISUAL: &str = "VISUAL";
}

/// `org.bukkit.Effect`.
pub mod effect {
    /// bukkit: org/bukkit/Effect.values
    pub fn values() -> &'static [&'static str] {
        &["DISPENSER_DISPENSE", "DISPENSER_FAIL", "DISPENSER_PROJECTILE_LAUNCH", "DOOR_TOGGLE", "IRON_DOOR_TOGGLE", "TRAPDOOR_TOGGLE", "IRON_TRAPDOOR_TOGGLE", "FENCE_GATE_TOGGLE", "DOOR_CLOSE", "IRON_DOOR_CLOSE", "TRAPDOOR_CLOSE", "IRON_TRAPDOOR_CLOSE", "FENCE_GATE_CLOSE", "ENDEREYE_LAUNCH", "FIREWORK_SHOOT", "EXTINGUISH", "RECORD_PLAY", "RECORD_STOP", "GHAST_SHRIEK", "GHAST_SHOOT", "ENDER_DRAGON_SHOOT", "BLAZE_SHOOT", "ZOMBIE_CHEW_WOODEN_DOOR", "ZOMBIE_CHEW_IRON_DOOR", "ZOMBIE_DESTROY_DOOR", "WITHER_BREAK_BLOCK", "WITHER_SPAWNED", "WITHER_SHOOT", "BAT_TAKEOFF", "ZOMBIE_INFECT", "ZOMBIE_CONVERTED_TO_VILLAGER", "ENDER_DRAGON_DEATH", "ANVIL_BREAK", "ANVIL_USE", "ANVIL_LAND", "PORTAL_TRAVEL", "CHORUS_FLOWER_GROW", "CHORUS_FLOWER_DEATH", "BREWING_STAND_BREW", "END_PORTAL_CREATED_IN_OVERWORLD", "PHANTOM_BITE", "ZOMBIE_CONVERTED_TO_DROWNED", "HUSK_CONVERTED_TO_ZOMBIE", "GRINDSTONE_USE", "BOOK_PAGE_TURN", "SMITHING_TABLE_USE", "POINTED_DRIPSTONE_LAND", "POINTED_DRIPSTONE_DRIP_LAVA_INTO_CAULDRON", "POINTED_DRIPSTONE_DRIP_WATER_INTO_CAULDRON", "SKELETON_CONVERTED_TO_STRAY", "CRAFTER_CRAFT", "CRAFTER_FAIL", "WIND_CHARGE_SHOOT", "SULFUR_SPIKE_LAND", "SPELL_POTION_SPLASH", "INSTANT_POTION_SPLASH", "COMPOSTER_FILL_ATTEMPT", "LAVA_INTERACT", "REDSTONE_TORCH_BURNOUT", "END_PORTAL_FRAME_FILL", "DRIPPING_DRIPSTONE", "BONE_MEAL_USE", "SMOKE_SHOOT", "STEP_SOUND", "DESTROY_BLOCK_WITH_SOUND", "POTION_BREAK", "ENDER_SIGNAL", "MOBSPAWNER_FLAMES", "ENDER_DRAGON_BREATH", "INSTANT_POTION_BREAK", "ENDER_DRAGON_DESTROY_BLOCK", "SPONGE_DRY", "WHITE_SMOKE_SHOOT", "BEE_GROWTH", "TURTLE_EGG_PLACEMENT", "SMASH_ATTACK", "DESTROY_BLOCK", "ENDER_DRAGON_EGG_TELEPORT", "SHULKER_TELEPORT", "CONSUME_EFFECT_TELEPORT", "ENDERMAN_TELEPORT", "DESTROY_PROGRESS", "DESTROY_PROGRESS_WITH_SOUND", "END_GATEWAY_SPAWN", "ENDER_DRAGON_GROWL", "ELECTRIC_SPARK", "COPPER_WAX_ON", "COPPER_WAX_OFF", "OXIDISED_COPPER_SCRAPE", "SCULK_CHARGE", "SCULK_SHRIEK", "BRUSH_BLOCK_COMPLETE", "EGG_CRACK", "GUST_DUST", "TRIAL_SPAWNER_SPAWN", "TRIAL_SPAWNER_SPAWN_MOB_AT", "TRIAL_SPAWNER_DETECT_PLAYER", "TRIAL_SPAWNER_EJECT_ITEM", "VAULT_ACTIVATE", "VAULT_DEACTIVATE", "VAULT_EJECT_ITEM", "SPAWN_COBWEB", "TRIAL_SPAWNER_DETECT_PLAYER_OMINOUS", "TRIAL_SPAWNER_BECOME_OMINOUS", "TRIAL_SPAWNER_SPAWN_ITEM", "CLICK2", "CLICK1", "BOW_FIRE", "SOUND_STOP_JUKEBOX_SONG", "ENDERDRAGON_SHOOT", "ZOMBIE_CONVERTED_VILLAGER", "SOUND_WITH_CHARGE_SHOT", "DRAGON_BREATH", "VILLAGER_PLANT_GROW", "SMOKE", "SHOOT_WHITE_SMOKE", "ENDERDRAGON_GROWL", "PARTICLES_SCULK_CHARGE", "PARTICLES_SCULK_SHRIEK", "PARTICLES_AND_SOUND_BRUSH_BLOCK_COMPLETE", "PARTICLES_EGG_CRACK"]
    }
    /// bukkit: org/bukkit/Effect.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DISPENSER_DISPENSE: &str = "DISPENSER_DISPENSE";
    pub const DISPENSER_FAIL: &str = "DISPENSER_FAIL";
    pub const DISPENSER_PROJECTILE_LAUNCH: &str = "DISPENSER_PROJECTILE_LAUNCH";
    pub const DOOR_TOGGLE: &str = "DOOR_TOGGLE";
    pub const IRON_DOOR_TOGGLE: &str = "IRON_DOOR_TOGGLE";
    pub const TRAPDOOR_TOGGLE: &str = "TRAPDOOR_TOGGLE";
    pub const IRON_TRAPDOOR_TOGGLE: &str = "IRON_TRAPDOOR_TOGGLE";
    pub const FENCE_GATE_TOGGLE: &str = "FENCE_GATE_TOGGLE";
    pub const DOOR_CLOSE: &str = "DOOR_CLOSE";
    pub const IRON_DOOR_CLOSE: &str = "IRON_DOOR_CLOSE";
    pub const TRAPDOOR_CLOSE: &str = "TRAPDOOR_CLOSE";
    pub const IRON_TRAPDOOR_CLOSE: &str = "IRON_TRAPDOOR_CLOSE";
    pub const FENCE_GATE_CLOSE: &str = "FENCE_GATE_CLOSE";
    pub const ENDEREYE_LAUNCH: &str = "ENDEREYE_LAUNCH";
    pub const FIREWORK_SHOOT: &str = "FIREWORK_SHOOT";
    pub const EXTINGUISH: &str = "EXTINGUISH";
    pub const RECORD_PLAY: &str = "RECORD_PLAY";
    pub const RECORD_STOP: &str = "RECORD_STOP";
    pub const GHAST_SHRIEK: &str = "GHAST_SHRIEK";
    pub const GHAST_SHOOT: &str = "GHAST_SHOOT";
    pub const ENDER_DRAGON_SHOOT: &str = "ENDER_DRAGON_SHOOT";
    pub const BLAZE_SHOOT: &str = "BLAZE_SHOOT";
    pub const ZOMBIE_CHEW_WOODEN_DOOR: &str = "ZOMBIE_CHEW_WOODEN_DOOR";
    pub const ZOMBIE_CHEW_IRON_DOOR: &str = "ZOMBIE_CHEW_IRON_DOOR";
    pub const ZOMBIE_DESTROY_DOOR: &str = "ZOMBIE_DESTROY_DOOR";
    pub const WITHER_BREAK_BLOCK: &str = "WITHER_BREAK_BLOCK";
    pub const WITHER_SPAWNED: &str = "WITHER_SPAWNED";
    pub const WITHER_SHOOT: &str = "WITHER_SHOOT";
    pub const BAT_TAKEOFF: &str = "BAT_TAKEOFF";
    pub const ZOMBIE_INFECT: &str = "ZOMBIE_INFECT";
    pub const ZOMBIE_CONVERTED_TO_VILLAGER: &str = "ZOMBIE_CONVERTED_TO_VILLAGER";
    pub const ENDER_DRAGON_DEATH: &str = "ENDER_DRAGON_DEATH";
    pub const ANVIL_BREAK: &str = "ANVIL_BREAK";
    pub const ANVIL_USE: &str = "ANVIL_USE";
    pub const ANVIL_LAND: &str = "ANVIL_LAND";
    pub const PORTAL_TRAVEL: &str = "PORTAL_TRAVEL";
    pub const CHORUS_FLOWER_GROW: &str = "CHORUS_FLOWER_GROW";
    pub const CHORUS_FLOWER_DEATH: &str = "CHORUS_FLOWER_DEATH";
    pub const BREWING_STAND_BREW: &str = "BREWING_STAND_BREW";
    pub const END_PORTAL_CREATED_IN_OVERWORLD: &str = "END_PORTAL_CREATED_IN_OVERWORLD";
    pub const PHANTOM_BITE: &str = "PHANTOM_BITE";
    pub const ZOMBIE_CONVERTED_TO_DROWNED: &str = "ZOMBIE_CONVERTED_TO_DROWNED";
    pub const HUSK_CONVERTED_TO_ZOMBIE: &str = "HUSK_CONVERTED_TO_ZOMBIE";
    pub const GRINDSTONE_USE: &str = "GRINDSTONE_USE";
    pub const BOOK_PAGE_TURN: &str = "BOOK_PAGE_TURN";
    pub const SMITHING_TABLE_USE: &str = "SMITHING_TABLE_USE";
    pub const POINTED_DRIPSTONE_LAND: &str = "POINTED_DRIPSTONE_LAND";
    pub const POINTED_DRIPSTONE_DRIP_LAVA_INTO_CAULDRON: &str = "POINTED_DRIPSTONE_DRIP_LAVA_INTO_CAULDRON";
    pub const POINTED_DRIPSTONE_DRIP_WATER_INTO_CAULDRON: &str = "POINTED_DRIPSTONE_DRIP_WATER_INTO_CAULDRON";
    pub const SKELETON_CONVERTED_TO_STRAY: &str = "SKELETON_CONVERTED_TO_STRAY";
    pub const CRAFTER_CRAFT: &str = "CRAFTER_CRAFT";
    pub const CRAFTER_FAIL: &str = "CRAFTER_FAIL";
    pub const WIND_CHARGE_SHOOT: &str = "WIND_CHARGE_SHOOT";
    pub const SULFUR_SPIKE_LAND: &str = "SULFUR_SPIKE_LAND";
    pub const SPELL_POTION_SPLASH: &str = "SPELL_POTION_SPLASH";
    pub const INSTANT_POTION_SPLASH: &str = "INSTANT_POTION_SPLASH";
    pub const COMPOSTER_FILL_ATTEMPT: &str = "COMPOSTER_FILL_ATTEMPT";
    pub const LAVA_INTERACT: &str = "LAVA_INTERACT";
    pub const REDSTONE_TORCH_BURNOUT: &str = "REDSTONE_TORCH_BURNOUT";
    pub const END_PORTAL_FRAME_FILL: &str = "END_PORTAL_FRAME_FILL";
    pub const DRIPPING_DRIPSTONE: &str = "DRIPPING_DRIPSTONE";
    pub const BONE_MEAL_USE: &str = "BONE_MEAL_USE";
    pub const SMOKE_SHOOT: &str = "SMOKE_SHOOT";
    pub const STEP_SOUND: &str = "STEP_SOUND";
    pub const DESTROY_BLOCK_WITH_SOUND: &str = "DESTROY_BLOCK_WITH_SOUND";
    pub const POTION_BREAK: &str = "POTION_BREAK";
    pub const ENDER_SIGNAL: &str = "ENDER_SIGNAL";
    pub const MOBSPAWNER_FLAMES: &str = "MOBSPAWNER_FLAMES";
    pub const ENDER_DRAGON_BREATH: &str = "ENDER_DRAGON_BREATH";
    pub const INSTANT_POTION_BREAK: &str = "INSTANT_POTION_BREAK";
    pub const ENDER_DRAGON_DESTROY_BLOCK: &str = "ENDER_DRAGON_DESTROY_BLOCK";
    pub const SPONGE_DRY: &str = "SPONGE_DRY";
    pub const WHITE_SMOKE_SHOOT: &str = "WHITE_SMOKE_SHOOT";
    pub const BEE_GROWTH: &str = "BEE_GROWTH";
    pub const TURTLE_EGG_PLACEMENT: &str = "TURTLE_EGG_PLACEMENT";
    pub const SMASH_ATTACK: &str = "SMASH_ATTACK";
    pub const DESTROY_BLOCK: &str = "DESTROY_BLOCK";
    pub const ENDER_DRAGON_EGG_TELEPORT: &str = "ENDER_DRAGON_EGG_TELEPORT";
    pub const SHULKER_TELEPORT: &str = "SHULKER_TELEPORT";
    pub const CONSUME_EFFECT_TELEPORT: &str = "CONSUME_EFFECT_TELEPORT";
    pub const ENDERMAN_TELEPORT: &str = "ENDERMAN_TELEPORT";
    pub const DESTROY_PROGRESS: &str = "DESTROY_PROGRESS";
    pub const DESTROY_PROGRESS_WITH_SOUND: &str = "DESTROY_PROGRESS_WITH_SOUND";
    pub const END_GATEWAY_SPAWN: &str = "END_GATEWAY_SPAWN";
    pub const ENDER_DRAGON_GROWL: &str = "ENDER_DRAGON_GROWL";
    pub const ELECTRIC_SPARK: &str = "ELECTRIC_SPARK";
    pub const COPPER_WAX_ON: &str = "COPPER_WAX_ON";
    pub const COPPER_WAX_OFF: &str = "COPPER_WAX_OFF";
    pub const OXIDISED_COPPER_SCRAPE: &str = "OXIDISED_COPPER_SCRAPE";
    pub const SCULK_CHARGE: &str = "SCULK_CHARGE";
    pub const SCULK_SHRIEK: &str = "SCULK_SHRIEK";
    pub const BRUSH_BLOCK_COMPLETE: &str = "BRUSH_BLOCK_COMPLETE";
    pub const EGG_CRACK: &str = "EGG_CRACK";
    pub const GUST_DUST: &str = "GUST_DUST";
    pub const TRIAL_SPAWNER_SPAWN: &str = "TRIAL_SPAWNER_SPAWN";
    pub const TRIAL_SPAWNER_SPAWN_MOB_AT: &str = "TRIAL_SPAWNER_SPAWN_MOB_AT";
    pub const TRIAL_SPAWNER_DETECT_PLAYER: &str = "TRIAL_SPAWNER_DETECT_PLAYER";
    pub const TRIAL_SPAWNER_EJECT_ITEM: &str = "TRIAL_SPAWNER_EJECT_ITEM";
    pub const VAULT_ACTIVATE: &str = "VAULT_ACTIVATE";
    pub const VAULT_DEACTIVATE: &str = "VAULT_DEACTIVATE";
    pub const VAULT_EJECT_ITEM: &str = "VAULT_EJECT_ITEM";
    pub const SPAWN_COBWEB: &str = "SPAWN_COBWEB";
    pub const TRIAL_SPAWNER_DETECT_PLAYER_OMINOUS: &str = "TRIAL_SPAWNER_DETECT_PLAYER_OMINOUS";
    pub const TRIAL_SPAWNER_BECOME_OMINOUS: &str = "TRIAL_SPAWNER_BECOME_OMINOUS";
    pub const TRIAL_SPAWNER_SPAWN_ITEM: &str = "TRIAL_SPAWNER_SPAWN_ITEM";
    pub const CLICK2: &str = "CLICK2";
    pub const CLICK1: &str = "CLICK1";
    pub const BOW_FIRE: &str = "BOW_FIRE";
    pub const SOUND_STOP_JUKEBOX_SONG: &str = "SOUND_STOP_JUKEBOX_SONG";
    pub const ENDERDRAGON_SHOOT: &str = "ENDERDRAGON_SHOOT";
    pub const ZOMBIE_CONVERTED_VILLAGER: &str = "ZOMBIE_CONVERTED_VILLAGER";
    pub const SOUND_WITH_CHARGE_SHOT: &str = "SOUND_WITH_CHARGE_SHOT";
    pub const DRAGON_BREATH: &str = "DRAGON_BREATH";
    pub const VILLAGER_PLANT_GROW: &str = "VILLAGER_PLANT_GROW";
    pub const SMOKE: &str = "SMOKE";
    pub const SHOOT_WHITE_SMOKE: &str = "SHOOT_WHITE_SMOKE";
    pub const ENDERDRAGON_GROWL: &str = "ENDERDRAGON_GROWL";
    pub const PARTICLES_SCULK_CHARGE: &str = "PARTICLES_SCULK_CHARGE";
    pub const PARTICLES_SCULK_SHRIEK: &str = "PARTICLES_SCULK_SHRIEK";
    pub const PARTICLES_AND_SOUND_BRUSH_BLOCK_COMPLETE: &str = "PARTICLES_AND_SOUND_BRUSH_BLOCK_COMPLETE";
    pub const PARTICLES_EGG_CRACK: &str = "PARTICLES_EGG_CRACK";
}

/// `org.bukkit.EntityEffect`.
pub mod entity_effect {
    /// bukkit: org/bukkit/EntityEffect.values
    pub fn values() -> &'static [&'static str] {
        &["ARROW_PARTICLES", "RABBIT_JUMP", "RESET_SPAWNER_MINECART_DELAY", "HIT", "HURT", "DEATH", "EGG_BREAK", "SNOWBALL_BREAK", "PROJECTILE_CRACK", "ENTITY_DEATH", "FANG_ATTACK", "HOGLIN_ATTACK", "IRON_GOLEN_ATTACK", "RAVAGER_ATTACK", "WARDEN_ATTACK", "ZOGLIN_ATTACK", "ENTITY_ATTACK", "WOLF_SMOKE", "WOLF_HEARTS", "TAMING_FAILED", "TAMING_SUCCEEDED", "WOLF_SHAKE", "SHEEP_EAT", "SHEEP_EAT_GRASS", "IRON_GOLEM_ROSE", "VILLAGER_HEART", "VILLAGER_ANGRY", "VILLAGER_HAPPY", "WITCH_MAGIC", "ZOMBIE_TRANSFORM", "FIREWORK_EXPLODE", "LOVE_HEARTS", "SQUID_ROTATE", "ENTITY_POOF", "GUARDIAN_TARGET", "SHIELD_BLOCK", "SHIELD_BREAK", "ARMOR_STAND_HIT", "THORNS_HURT", "IRON_GOLEM_SHEATH", "TOTEM_RESURRECT", "PROTECTED_FROM_DEATH", "HURT_DROWN", "HURT_EXPLOSION", "DOLPHIN_FED", "RAVAGER_STUNNED", "CAT_TAME_FAIL", "CAT_TAME_SUCCESS", "TRUSTING_FAILED", "TRUSTING_SUCCEEDED", "VILLAGER_SPLASH", "PLAYER_BAD_OMEN_RAID", "HURT_BERRY_BUSH", "FOX_CHEW", "TELEPORT_ENDER", "BREAK_EQUIPMENT_MAIN_HAND", "BREAK_EQUIPMENT_OFF_HAND", "BREAK_EQUIPMENT_HELMET", "BREAK_EQUIPMENT_CHESTPLATE", "BREAK_EQUIPMENT_LEGGINGS", "BREAK_EQUIPMENT_BOOTS", "HONEY_BLOCK_SLIDE_PARTICLES", "HONEY_BLOCK_FALL_PARTICLES", "SWAP_HAND_ITEMS", "WOLF_SHAKE_STOP", "GOAT_LOWER_HEAD", "GOAT_RAISE_HEAD", "SPAWN_DEATH_SMOKE", "WARDEN_TENDRIL_SHAKE", "WARDEN_SONIC_ATTACK", "SNIFFER_DIG", "ARMADILLO_PEEK", "BODY_BREAK", "BREAK_EQUIPMENT_BODY", "SHAKE", "DROWN_PARTICLES", "BREAK_EQUIPMENT_SADDLE", "RAVAGER_ROARED", "TNT_MINECART_IGNITE", "BOAT_SINK", "BOAT_LAUNCH"]
    }
    /// bukkit: org/bukkit/EntityEffect.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ARROW_PARTICLES: &str = "ARROW_PARTICLES";
    pub const RABBIT_JUMP: &str = "RABBIT_JUMP";
    pub const RESET_SPAWNER_MINECART_DELAY: &str = "RESET_SPAWNER_MINECART_DELAY";
    pub const HIT: &str = "HIT";
    pub const HURT: &str = "HURT";
    pub const DEATH: &str = "DEATH";
    pub const EGG_BREAK: &str = "EGG_BREAK";
    pub const SNOWBALL_BREAK: &str = "SNOWBALL_BREAK";
    pub const PROJECTILE_CRACK: &str = "PROJECTILE_CRACK";
    pub const ENTITY_DEATH: &str = "ENTITY_DEATH";
    pub const FANG_ATTACK: &str = "FANG_ATTACK";
    pub const HOGLIN_ATTACK: &str = "HOGLIN_ATTACK";
    pub const IRON_GOLEN_ATTACK: &str = "IRON_GOLEN_ATTACK";
    pub const RAVAGER_ATTACK: &str = "RAVAGER_ATTACK";
    pub const WARDEN_ATTACK: &str = "WARDEN_ATTACK";
    pub const ZOGLIN_ATTACK: &str = "ZOGLIN_ATTACK";
    pub const ENTITY_ATTACK: &str = "ENTITY_ATTACK";
    pub const WOLF_SMOKE: &str = "WOLF_SMOKE";
    pub const WOLF_HEARTS: &str = "WOLF_HEARTS";
    pub const TAMING_FAILED: &str = "TAMING_FAILED";
    pub const TAMING_SUCCEEDED: &str = "TAMING_SUCCEEDED";
    pub const WOLF_SHAKE: &str = "WOLF_SHAKE";
    pub const SHEEP_EAT: &str = "SHEEP_EAT";
    pub const SHEEP_EAT_GRASS: &str = "SHEEP_EAT_GRASS";
    pub const IRON_GOLEM_ROSE: &str = "IRON_GOLEM_ROSE";
    pub const VILLAGER_HEART: &str = "VILLAGER_HEART";
    pub const VILLAGER_ANGRY: &str = "VILLAGER_ANGRY";
    pub const VILLAGER_HAPPY: &str = "VILLAGER_HAPPY";
    pub const WITCH_MAGIC: &str = "WITCH_MAGIC";
    pub const ZOMBIE_TRANSFORM: &str = "ZOMBIE_TRANSFORM";
    pub const FIREWORK_EXPLODE: &str = "FIREWORK_EXPLODE";
    pub const LOVE_HEARTS: &str = "LOVE_HEARTS";
    pub const SQUID_ROTATE: &str = "SQUID_ROTATE";
    pub const ENTITY_POOF: &str = "ENTITY_POOF";
    pub const GUARDIAN_TARGET: &str = "GUARDIAN_TARGET";
    pub const SHIELD_BLOCK: &str = "SHIELD_BLOCK";
    pub const SHIELD_BREAK: &str = "SHIELD_BREAK";
    pub const ARMOR_STAND_HIT: &str = "ARMOR_STAND_HIT";
    pub const THORNS_HURT: &str = "THORNS_HURT";
    pub const IRON_GOLEM_SHEATH: &str = "IRON_GOLEM_SHEATH";
    pub const TOTEM_RESURRECT: &str = "TOTEM_RESURRECT";
    pub const PROTECTED_FROM_DEATH: &str = "PROTECTED_FROM_DEATH";
    pub const HURT_DROWN: &str = "HURT_DROWN";
    pub const HURT_EXPLOSION: &str = "HURT_EXPLOSION";
    pub const DOLPHIN_FED: &str = "DOLPHIN_FED";
    pub const RAVAGER_STUNNED: &str = "RAVAGER_STUNNED";
    pub const CAT_TAME_FAIL: &str = "CAT_TAME_FAIL";
    pub const CAT_TAME_SUCCESS: &str = "CAT_TAME_SUCCESS";
    pub const TRUSTING_FAILED: &str = "TRUSTING_FAILED";
    pub const TRUSTING_SUCCEEDED: &str = "TRUSTING_SUCCEEDED";
    pub const VILLAGER_SPLASH: &str = "VILLAGER_SPLASH";
    pub const PLAYER_BAD_OMEN_RAID: &str = "PLAYER_BAD_OMEN_RAID";
    pub const HURT_BERRY_BUSH: &str = "HURT_BERRY_BUSH";
    pub const FOX_CHEW: &str = "FOX_CHEW";
    pub const TELEPORT_ENDER: &str = "TELEPORT_ENDER";
    pub const BREAK_EQUIPMENT_MAIN_HAND: &str = "BREAK_EQUIPMENT_MAIN_HAND";
    pub const BREAK_EQUIPMENT_OFF_HAND: &str = "BREAK_EQUIPMENT_OFF_HAND";
    pub const BREAK_EQUIPMENT_HELMET: &str = "BREAK_EQUIPMENT_HELMET";
    pub const BREAK_EQUIPMENT_CHESTPLATE: &str = "BREAK_EQUIPMENT_CHESTPLATE";
    pub const BREAK_EQUIPMENT_LEGGINGS: &str = "BREAK_EQUIPMENT_LEGGINGS";
    pub const BREAK_EQUIPMENT_BOOTS: &str = "BREAK_EQUIPMENT_BOOTS";
    pub const HONEY_BLOCK_SLIDE_PARTICLES: &str = "HONEY_BLOCK_SLIDE_PARTICLES";
    pub const HONEY_BLOCK_FALL_PARTICLES: &str = "HONEY_BLOCK_FALL_PARTICLES";
    pub const SWAP_HAND_ITEMS: &str = "SWAP_HAND_ITEMS";
    pub const WOLF_SHAKE_STOP: &str = "WOLF_SHAKE_STOP";
    pub const GOAT_LOWER_HEAD: &str = "GOAT_LOWER_HEAD";
    pub const GOAT_RAISE_HEAD: &str = "GOAT_RAISE_HEAD";
    pub const SPAWN_DEATH_SMOKE: &str = "SPAWN_DEATH_SMOKE";
    pub const WARDEN_TENDRIL_SHAKE: &str = "WARDEN_TENDRIL_SHAKE";
    pub const WARDEN_SONIC_ATTACK: &str = "WARDEN_SONIC_ATTACK";
    pub const SNIFFER_DIG: &str = "SNIFFER_DIG";
    pub const ARMADILLO_PEEK: &str = "ARMADILLO_PEEK";
    pub const BODY_BREAK: &str = "BODY_BREAK";
    pub const BREAK_EQUIPMENT_BODY: &str = "BREAK_EQUIPMENT_BODY";
    pub const SHAKE: &str = "SHAKE";
    pub const DROWN_PARTICLES: &str = "DROWN_PARTICLES";
    pub const BREAK_EQUIPMENT_SADDLE: &str = "BREAK_EQUIPMENT_SADDLE";
    pub const RAVAGER_ROARED: &str = "RAVAGER_ROARED";
    pub const TNT_MINECART_IGNITE: &str = "TNT_MINECART_IGNITE";
    pub const BOAT_SINK: &str = "BOAT_SINK";
    pub const BOAT_LAUNCH: &str = "BOAT_LAUNCH";
}

/// `org.bukkit.ExplosionResult`.
pub mod explosion_result {
    /// bukkit: org/bukkit/ExplosionResult.values
    pub fn values() -> &'static [&'static str] {
        &["KEEP", "DESTROY", "DESTROY_WITH_DECAY", "TRIGGER_BLOCK"]
    }
    /// bukkit: org/bukkit/ExplosionResult.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const KEEP: &str = "KEEP";
    pub const DESTROY: &str = "DESTROY";
    pub const DESTROY_WITH_DECAY: &str = "DESTROY_WITH_DECAY";
    pub const TRIGGER_BLOCK: &str = "TRIGGER_BLOCK";
}

/// `org.bukkit.FireworkEffect$Type`.
pub mod firework_effect_type {
    /// bukkit: org/bukkit/FireworkEffect$Type.values
    pub fn values() -> &'static [&'static str] {
        &["BALL", "BALL_LARGE", "STAR", "BURST", "CREEPER"]
    }
    /// bukkit: org/bukkit/FireworkEffect$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BALL: &str = "BALL";
    pub const BALL_LARGE: &str = "BALL_LARGE";
    pub const STAR: &str = "STAR";
    pub const BURST: &str = "BURST";
    pub const CREEPER: &str = "CREEPER";
}

/// `org.bukkit.Fluid`.
pub mod fluid {
    /// bukkit: org/bukkit/Fluid.values
    pub fn values() -> &'static [&'static str] {
        &["EMPTY", "FLOWING_LAVA", "FLOWING_WATER", "LAVA", "WATER"]
    }
    /// bukkit: org/bukkit/Fluid.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const EMPTY: &str = "minecraft:empty";
    pub const FLOWING_LAVA: &str = "minecraft:flowing_lava";
    pub const FLOWING_WATER: &str = "minecraft:flowing_water";
    pub const LAVA: &str = "minecraft:lava";
    pub const WATER: &str = "minecraft:water";
}

/// `org.bukkit.FluidCollisionMode`.
pub mod fluid_collision_mode {
    /// bukkit: org/bukkit/FluidCollisionMode.values
    pub fn values() -> &'static [&'static str] {
        &["NEVER", "SOURCE_ONLY", "ALWAYS"]
    }
    /// bukkit: org/bukkit/FluidCollisionMode.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NEVER: &str = "NEVER";
    pub const SOURCE_ONLY: &str = "SOURCE_ONLY";
    pub const ALWAYS: &str = "ALWAYS";
}

/// `org.bukkit.GameEvent`.
pub mod game_event {
    /// bukkit: org/bukkit/GameEvent.values
    pub fn values() -> &'static [&'static str] {
        &["BLOCK_ACTIVATE", "BLOCK_ATTACH", "BLOCK_CHANGE", "BLOCK_CLOSE", "BLOCK_DEACTIVATE", "BLOCK_DESTROY", "BLOCK_DETACH", "BLOCK_OPEN", "BLOCK_PLACE", "BOUNCE", "CONTAINER_CLOSE", "CONTAINER_OPEN", "DRINK", "EAT", "ELYTRA_GLIDE", "ENTITY_ACTION", "ENTITY_DAMAGE", "ENTITY_DIE", "ENTITY_DISMOUNT", "ENTITY_INTERACT", "ENTITY_MOUNT", "ENTITY_PLACE", "EQUIP", "EXPLODE", "FLAP", "FLUID_PICKUP", "FLUID_PLACE", "HIT_GROUND", "INSTRUMENT_PLAY", "ITEM_INTERACT_FINISH", "ITEM_INTERACT_START", "JUKEBOX_PLAY", "JUKEBOX_STOP_PLAY", "LIGHTNING_STRIKE", "NOTE_BLOCK_PLAY", "PRIME_FUSE", "PROJECTILE_LAND", "PROJECTILE_SHOOT", "RESONATE_1", "RESONATE_2", "RESONATE_3", "RESONATE_4", "RESONATE_5", "RESONATE_6", "RESONATE_7", "RESONATE_8", "RESONATE_9", "RESONATE_10", "RESONATE_11", "RESONATE_12", "RESONATE_13", "RESONATE_14", "RESONATE_15", "SCULK_SENSOR_TENDRILS_CLICKING", "SHEAR", "SHRIEK", "SPLASH", "STEP", "SWIM", "TELEPORT", "UNEQUIP", "BLOCK_PRESS", "BLOCK_SWITCH", "BLOCK_UNPRESS", "BLOCK_UNSWITCH", "DISPENSE_FAIL", "DRINKING_FINISH", "ELYTRA_FREE_FALL", "ENTITY_DAMAGED", "ENTITY_DYING", "ENTITY_KILLED", "ENTITY_ROAR", "ENTITY_SHAKE", "MOB_INTERACT", "PISTON_CONTRACT", "PISTON_EXTEND", "RAVAGER_ROAR", "RING_BELL", "SHULKER_CLOSE", "SHULKER_OPEN", "WOLF_SHAKING"]
    }
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BLOCK_ACTIVATE: &str = "minecraft:block_activate";
    pub const BLOCK_ATTACH: &str = "minecraft:block_attach";
    pub const BLOCK_CHANGE: &str = "minecraft:block_change";
    pub const BLOCK_CLOSE: &str = "minecraft:block_close";
    pub const BLOCK_DEACTIVATE: &str = "minecraft:block_deactivate";
    pub const BLOCK_DESTROY: &str = "minecraft:block_destroy";
    pub const BLOCK_DETACH: &str = "minecraft:block_detach";
    pub const BLOCK_OPEN: &str = "minecraft:block_open";
    pub const BLOCK_PLACE: &str = "minecraft:block_place";
    pub const BOUNCE: &str = "minecraft:bounce";
    pub const CONTAINER_CLOSE: &str = "minecraft:container_close";
    pub const CONTAINER_OPEN: &str = "minecraft:container_open";
    pub const DRINK: &str = "minecraft:drink";
    pub const EAT: &str = "minecraft:eat";
    pub const ELYTRA_GLIDE: &str = "minecraft:elytra_glide";
    pub const ENTITY_ACTION: &str = "minecraft:entity_action";
    pub const ENTITY_DAMAGE: &str = "minecraft:entity_damage";
    pub const ENTITY_DIE: &str = "minecraft:entity_die";
    pub const ENTITY_DISMOUNT: &str = "minecraft:entity_dismount";
    pub const ENTITY_INTERACT: &str = "minecraft:entity_interact";
    pub const ENTITY_MOUNT: &str = "minecraft:entity_mount";
    pub const ENTITY_PLACE: &str = "minecraft:entity_place";
    pub const EQUIP: &str = "minecraft:equip";
    pub const EXPLODE: &str = "minecraft:explode";
    pub const FLAP: &str = "minecraft:flap";
    pub const FLUID_PICKUP: &str = "minecraft:fluid_pickup";
    pub const FLUID_PLACE: &str = "minecraft:fluid_place";
    pub const HIT_GROUND: &str = "minecraft:hit_ground";
    pub const INSTRUMENT_PLAY: &str = "minecraft:instrument_play";
    pub const ITEM_INTERACT_FINISH: &str = "minecraft:item_interact_finish";
    pub const ITEM_INTERACT_START: &str = "minecraft:item_interact_start";
    pub const JUKEBOX_PLAY: &str = "minecraft:jukebox_play";
    pub const JUKEBOX_STOP_PLAY: &str = "minecraft:jukebox_stop_play";
    pub const LIGHTNING_STRIKE: &str = "minecraft:lightning_strike";
    pub const NOTE_BLOCK_PLAY: &str = "minecraft:note_block_play";
    pub const PRIME_FUSE: &str = "minecraft:prime_fuse";
    pub const PROJECTILE_LAND: &str = "minecraft:projectile_land";
    pub const PROJECTILE_SHOOT: &str = "minecraft:projectile_shoot";
    pub const RESONATE_1: &str = "minecraft:resonate_1";
    pub const RESONATE_2: &str = "minecraft:resonate_2";
    pub const RESONATE_3: &str = "minecraft:resonate_3";
    pub const RESONATE_4: &str = "minecraft:resonate_4";
    pub const RESONATE_5: &str = "minecraft:resonate_5";
    pub const RESONATE_6: &str = "minecraft:resonate_6";
    pub const RESONATE_7: &str = "minecraft:resonate_7";
    pub const RESONATE_8: &str = "minecraft:resonate_8";
    pub const RESONATE_9: &str = "minecraft:resonate_9";
    pub const RESONATE_10: &str = "minecraft:resonate_10";
    pub const RESONATE_11: &str = "minecraft:resonate_11";
    pub const RESONATE_12: &str = "minecraft:resonate_12";
    pub const RESONATE_13: &str = "minecraft:resonate_13";
    pub const RESONATE_14: &str = "minecraft:resonate_14";
    pub const RESONATE_15: &str = "minecraft:resonate_15";
    pub const SCULK_SENSOR_TENDRILS_CLICKING: &str = "minecraft:sculk_sensor_tendrils_clicking";
    pub const SHEAR: &str = "minecraft:shear";
    pub const SHRIEK: &str = "minecraft:shriek";
    pub const SPLASH: &str = "minecraft:splash";
    pub const STEP: &str = "minecraft:step";
    pub const SWIM: &str = "minecraft:swim";
    pub const TELEPORT: &str = "minecraft:teleport";
    pub const UNEQUIP: &str = "minecraft:unequip";
    pub const BLOCK_PRESS: &str = "minecraft:block_press";
    pub const BLOCK_SWITCH: &str = "minecraft:block_switch";
    pub const BLOCK_UNPRESS: &str = "minecraft:block_unpress";
    pub const BLOCK_UNSWITCH: &str = "minecraft:block_unswitch";
    pub const DISPENSE_FAIL: &str = "minecraft:dispense_fail";
    pub const DRINKING_FINISH: &str = "minecraft:drinking_finish";
    pub const ELYTRA_FREE_FALL: &str = "minecraft:elytra_free_fall";
    pub const ENTITY_DAMAGED: &str = "minecraft:entity_damaged";
    pub const ENTITY_DYING: &str = "minecraft:entity_dying";
    pub const ENTITY_KILLED: &str = "minecraft:entity_killed";
    pub const ENTITY_ROAR: &str = "minecraft:entity_roar";
    pub const ENTITY_SHAKE: &str = "minecraft:entity_shake";
    pub const MOB_INTERACT: &str = "minecraft:mob_interact";
    pub const PISTON_CONTRACT: &str = "minecraft:piston_contract";
    pub const PISTON_EXTEND: &str = "minecraft:piston_extend";
    pub const RAVAGER_ROAR: &str = "minecraft:ravager_roar";
    pub const RING_BELL: &str = "minecraft:ring_bell";
    pub const SHULKER_CLOSE: &str = "minecraft:shulker_close";
    pub const SHULKER_OPEN: &str = "minecraft:shulker_open";
    pub const WOLF_SHAKING: &str = "minecraft:wolf_shaking";
}

/// `org.bukkit.GameMode`.
pub mod game_mode {
    /// bukkit: org/bukkit/GameMode.values
    pub fn values() -> &'static [&'static str] {
        &["CREATIVE", "SURVIVAL", "ADVENTURE", "SPECTATOR"]
    }
    /// bukkit: org/bukkit/GameMode.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CREATIVE: &str = "CREATIVE";
    pub const SURVIVAL: &str = "SURVIVAL";
    pub const ADVENTURE: &str = "ADVENTURE";
    pub const SPECTATOR: &str = "SPECTATOR";
}

/// `org.bukkit.GameRule`.
pub mod game_rule {
    /// bukkit: org/bukkit/GameRule.values
    pub fn values() -> &'static [&'static str] {
        &["ANNOUNCE_ADVANCEMENTS", "COMMAND_BLOCK_OUTPUT", "DISABLE_PLAYER_MOVEMENT_CHECK", "DISABLE_ELYTRA_MOVEMENT_CHECK", "DO_DAYLIGHT_CYCLE", "DO_ENTITY_DROPS", "DO_FIRE_TICK", "DO_LIMITED_CRAFTING", "PROJECTILES_CAN_BREAK_BLOCKS", "DO_MOB_LOOT", "DO_MOB_SPAWNING", "DO_TILE_DROPS", "DO_WEATHER_CYCLE", "KEEP_INVENTORY", "LOG_ADMIN_COMMANDS", "MOB_GRIEFING", "NATURAL_REGENERATION", "REDUCED_DEBUG_INFO", "SEND_COMMAND_FEEDBACK", "SHOW_DEATH_MESSAGES", "SPECTATORS_GENERATE_CHUNKS", "DISABLE_RAIDS", "DO_INSOMNIA", "DO_IMMEDIATE_RESPAWN", "DROWNING_DAMAGE", "FALL_DAMAGE", "FIRE_DAMAGE", "FREEZE_DAMAGE", "DO_PATROL_SPAWNING", "DO_TRADER_SPAWNING", "DO_WARDEN_SPAWNING", "FORGIVE_DEAD_PLAYERS", "UNIVERSAL_ANGER", "BLOCK_EXPLOSION_DROP_DECAY", "MOB_EXPLOSION_DROP_DECAY", "TNT_EXPLOSION_DROP_DECAY", "WATER_SOURCE_CONVERSION", "LAVA_SOURCE_CONVERSION", "GLOBAL_SOUND_EVENTS", "DO_VINES_SPREAD", "ENDER_PEARLS_VANISH_ON_DEATH", "ALLOW_FIRE_TICKS_AWAY_FROM_PLAYER", "TNT_EXPLODES", "LOCATOR_BAR", "PVP", "SPAWN_MONSTERS", "ALLOW_ENTERING_NETHER_USING_PORTALS", "COMMAND_BLOCKS_ENABLED", "SPAWNER_BLOCKS_ENABLED", "RANDOM_TICK_SPEED", "SPAWN_RADIUS", "MAX_ENTITY_CRAMMING", "MAX_COMMAND_CHAIN_LENGTH", "MAX_COMMAND_FORK_COUNT", "COMMAND_MODIFICATION_BLOCK_LIMIT", "PLAYERS_SLEEPING_PERCENTAGE", "SNOW_ACCUMULATION_HEIGHT", "PLAYERS_NETHER_PORTAL_DEFAULT_DELAY", "PLAYERS_NETHER_PORTAL_CREATIVE_DELAY", "MINECART_MAX_SPEED"]
    }
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ANNOUNCE_ADVANCEMENTS: &str = "minecraft:announce_advancements";
    pub const COMMAND_BLOCK_OUTPUT: &str = "minecraft:command_block_output";
    pub const DISABLE_PLAYER_MOVEMENT_CHECK: &str = "minecraft:disable_player_movement_check";
    pub const DISABLE_ELYTRA_MOVEMENT_CHECK: &str = "minecraft:disable_elytra_movement_check";
    pub const DO_DAYLIGHT_CYCLE: &str = "minecraft:do_daylight_cycle";
    pub const DO_ENTITY_DROPS: &str = "minecraft:do_entity_drops";
    pub const DO_FIRE_TICK: &str = "minecraft:do_fire_tick";
    pub const DO_LIMITED_CRAFTING: &str = "minecraft:do_limited_crafting";
    pub const PROJECTILES_CAN_BREAK_BLOCKS: &str = "minecraft:projectiles_can_break_blocks";
    pub const DO_MOB_LOOT: &str = "minecraft:do_mob_loot";
    pub const DO_MOB_SPAWNING: &str = "minecraft:do_mob_spawning";
    pub const DO_TILE_DROPS: &str = "minecraft:do_tile_drops";
    pub const DO_WEATHER_CYCLE: &str = "minecraft:do_weather_cycle";
    pub const KEEP_INVENTORY: &str = "minecraft:keep_inventory";
    pub const LOG_ADMIN_COMMANDS: &str = "minecraft:log_admin_commands";
    pub const MOB_GRIEFING: &str = "minecraft:mob_griefing";
    pub const NATURAL_REGENERATION: &str = "minecraft:natural_regeneration";
    pub const REDUCED_DEBUG_INFO: &str = "minecraft:reduced_debug_info";
    pub const SEND_COMMAND_FEEDBACK: &str = "minecraft:send_command_feedback";
    pub const SHOW_DEATH_MESSAGES: &str = "minecraft:show_death_messages";
    pub const SPECTATORS_GENERATE_CHUNKS: &str = "minecraft:spectators_generate_chunks";
    pub const DISABLE_RAIDS: &str = "minecraft:disable_raids";
    pub const DO_INSOMNIA: &str = "minecraft:do_insomnia";
    pub const DO_IMMEDIATE_RESPAWN: &str = "minecraft:do_immediate_respawn";
    pub const DROWNING_DAMAGE: &str = "minecraft:drowning_damage";
    pub const FALL_DAMAGE: &str = "minecraft:fall_damage";
    pub const FIRE_DAMAGE: &str = "minecraft:fire_damage";
    pub const FREEZE_DAMAGE: &str = "minecraft:freeze_damage";
    pub const DO_PATROL_SPAWNING: &str = "minecraft:do_patrol_spawning";
    pub const DO_TRADER_SPAWNING: &str = "minecraft:do_trader_spawning";
    pub const DO_WARDEN_SPAWNING: &str = "minecraft:do_warden_spawning";
    pub const FORGIVE_DEAD_PLAYERS: &str = "minecraft:forgive_dead_players";
    pub const UNIVERSAL_ANGER: &str = "minecraft:universal_anger";
    pub const BLOCK_EXPLOSION_DROP_DECAY: &str = "minecraft:block_explosion_drop_decay";
    pub const MOB_EXPLOSION_DROP_DECAY: &str = "minecraft:mob_explosion_drop_decay";
    pub const TNT_EXPLOSION_DROP_DECAY: &str = "minecraft:tnt_explosion_drop_decay";
    pub const WATER_SOURCE_CONVERSION: &str = "minecraft:water_source_conversion";
    pub const LAVA_SOURCE_CONVERSION: &str = "minecraft:lava_source_conversion";
    pub const GLOBAL_SOUND_EVENTS: &str = "minecraft:global_sound_events";
    pub const DO_VINES_SPREAD: &str = "minecraft:do_vines_spread";
    pub const ENDER_PEARLS_VANISH_ON_DEATH: &str = "minecraft:ender_pearls_vanish_on_death";
    pub const ALLOW_FIRE_TICKS_AWAY_FROM_PLAYER: &str = "minecraft:allow_fire_ticks_away_from_player";
    pub const TNT_EXPLODES: &str = "minecraft:tnt_explodes";
    pub const LOCATOR_BAR: &str = "minecraft:locator_bar";
    pub const PVP: &str = "minecraft:pvp";
    pub const SPAWN_MONSTERS: &str = "minecraft:spawn_monsters";
    pub const ALLOW_ENTERING_NETHER_USING_PORTALS: &str = "minecraft:allow_entering_nether_using_portals";
    pub const COMMAND_BLOCKS_ENABLED: &str = "minecraft:command_blocks_enabled";
    pub const SPAWNER_BLOCKS_ENABLED: &str = "minecraft:spawner_blocks_enabled";
    pub const RANDOM_TICK_SPEED: &str = "minecraft:random_tick_speed";
    pub const SPAWN_RADIUS: &str = "minecraft:spawn_radius";
    pub const MAX_ENTITY_CRAMMING: &str = "minecraft:max_entity_cramming";
    pub const MAX_COMMAND_CHAIN_LENGTH: &str = "minecraft:max_command_chain_length";
    pub const MAX_COMMAND_FORK_COUNT: &str = "minecraft:max_command_fork_count";
    pub const COMMAND_MODIFICATION_BLOCK_LIMIT: &str = "minecraft:command_modification_block_limit";
    pub const PLAYERS_SLEEPING_PERCENTAGE: &str = "minecraft:players_sleeping_percentage";
    pub const SNOW_ACCUMULATION_HEIGHT: &str = "minecraft:snow_accumulation_height";
    pub const PLAYERS_NETHER_PORTAL_DEFAULT_DELAY: &str = "minecraft:players_nether_portal_default_delay";
    pub const PLAYERS_NETHER_PORTAL_CREATIVE_DELAY: &str = "minecraft:players_nether_portal_creative_delay";
    pub const MINECART_MAX_SPEED: &str = "minecraft:minecart_max_speed";
}

/// `org.bukkit.GrassSpecies`.
pub mod grass_species {
    /// bukkit: org/bukkit/GrassSpecies.values
    pub fn values() -> &'static [&'static str] {
        &["DEAD", "NORMAL", "FERN_LIKE"]
    }
    /// bukkit: org/bukkit/GrassSpecies.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DEAD: &str = "DEAD";
    pub const NORMAL: &str = "NORMAL";
    pub const FERN_LIKE: &str = "FERN_LIKE";
}

/// `org.bukkit.HeightMap`.
pub mod height_map {
    /// bukkit: org/bukkit/HeightMap.values
    pub fn values() -> &'static [&'static str] {
        &["MOTION_BLOCKING", "MOTION_BLOCKING_NO_LEAVES", "OCEAN_FLOOR", "OCEAN_FLOOR_WG", "WORLD_SURFACE", "WORLD_SURFACE_WG"]
    }
    /// bukkit: org/bukkit/HeightMap.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const MOTION_BLOCKING: &str = "MOTION_BLOCKING";
    pub const MOTION_BLOCKING_NO_LEAVES: &str = "MOTION_BLOCKING_NO_LEAVES";
    pub const OCEAN_FLOOR: &str = "OCEAN_FLOOR";
    pub const OCEAN_FLOOR_WG: &str = "OCEAN_FLOOR_WG";
    pub const WORLD_SURFACE: &str = "WORLD_SURFACE";
    pub const WORLD_SURFACE_WG: &str = "WORLD_SURFACE_WG";
}

/// `org.bukkit.Instrument`.
pub mod instrument {
    /// bukkit: org/bukkit/Instrument.values
    pub fn values() -> &'static [&'static str] {
        &["PIANO", "BASS_DRUM", "SNARE_DRUM", "STICKS", "BASS_GUITAR", "FLUTE", "BELL", "GUITAR", "CHIME", "XYLOPHONE", "IRON_XYLOPHONE", "COW_BELL", "DIDGERIDOO", "BIT", "BANJO", "PLING", "TRUMPET", "TRUMPET_EXPOSED", "TRUMPET_OXIDIZED", "TRUMPET_WEATHERED", "ZOMBIE", "SKELETON", "CREEPER", "DRAGON", "WITHER_SKELETON", "PIGLIN", "CUSTOM_HEAD"]
    }
    /// bukkit: org/bukkit/Instrument.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PIANO: &str = "PIANO";
    pub const BASS_DRUM: &str = "BASS_DRUM";
    pub const SNARE_DRUM: &str = "SNARE_DRUM";
    pub const STICKS: &str = "STICKS";
    pub const BASS_GUITAR: &str = "BASS_GUITAR";
    pub const FLUTE: &str = "FLUTE";
    pub const BELL: &str = "BELL";
    pub const GUITAR: &str = "GUITAR";
    pub const CHIME: &str = "CHIME";
    pub const XYLOPHONE: &str = "XYLOPHONE";
    pub const IRON_XYLOPHONE: &str = "IRON_XYLOPHONE";
    pub const COW_BELL: &str = "COW_BELL";
    pub const DIDGERIDOO: &str = "DIDGERIDOO";
    pub const BIT: &str = "BIT";
    pub const BANJO: &str = "BANJO";
    pub const PLING: &str = "PLING";
    pub const TRUMPET: &str = "TRUMPET";
    pub const TRUMPET_EXPOSED: &str = "TRUMPET_EXPOSED";
    pub const TRUMPET_OXIDIZED: &str = "TRUMPET_OXIDIZED";
    pub const TRUMPET_WEATHERED: &str = "TRUMPET_WEATHERED";
    pub const ZOMBIE: &str = "ZOMBIE";
    pub const SKELETON: &str = "SKELETON";
    pub const CREEPER: &str = "CREEPER";
    pub const DRAGON: &str = "DRAGON";
    pub const WITHER_SKELETON: &str = "WITHER_SKELETON";
    pub const PIGLIN: &str = "PIGLIN";
    pub const CUSTOM_HEAD: &str = "CUSTOM_HEAD";
}

/// `org.bukkit.Material`.
pub mod material {
    /// bukkit: org/bukkit/Material.values
    pub fn values() -> &'static [&'static str] {
        &["ABANDONED_CAMP_MAP", "ACACIA_BOAT", "ACACIA_CHEST_BOAT", "AIR", "ALLAY_SPAWN_EGG", "AMETHYST_SHARD", "ANGLER_POTTERY_SHERD", "APPLE", "ARCHER_POTTERY_SHERD", "ARMADILLO_SCUTE", "ARMADILLO_SPAWN_EGG", "ARMOR_STAND", "ARMS_UP_POTTERY_SHERD", "ARROW", "AXOLOTL_BUCKET", "AXOLOTL_SPAWN_EGG", "BAKED_POTATO", "BAMBOO_CHEST_RAFT", "BAMBOO_RAFT", "BAT_SPAWN_EGG", "BEE_SPAWN_EGG", "BEEF", "BEETROOT", "BEETROOT_SEEDS", "BEETROOT_SOUP", "BIRCH_BOAT", "BIRCH_CHEST_BOAT", "BLACK_BUNDLE", "BLACK_CUSHION", "BLACK_DYE", "BLACK_HARNESS", "BLADE_POTTERY_SHERD", "BLAZE_POWDER", "BLAZE_ROD", "BLAZE_SPAWN_EGG", "BLUE_BUNDLE", "BLUE_CUSHION", "BLUE_DYE", "BLUE_EGG", "BLUE_HARNESS", "BOGGED_SPAWN_EGG", "BOLT_ARMOR_TRIM_SMITHING_TEMPLATE", "BONE", "BONE_MEAL", "BOOK", "BORDURE_INDENTED_BANNER_PATTERN", "BOW", "BOWL", "BREAD", "BREEZE_ROD", "BREEZE_SPAWN_EGG", "BREWER_POTTERY_SHERD", "BRICK", "BROWN_BUNDLE", "BROWN_CUSHION", "BROWN_DYE", "BROWN_EGG", "BROWN_HARNESS", "BRUSH", "BUCKET", "BUNDLE", "BURIED_ANCIENT_CITY_MAP", "BURIED_MINESHAFT_MAP", "BURIED_TREASURE_MAP", "BURIED_TRIAL_CHAMBERS_MAP", "BURN_POTTERY_SHERD", "CAMEL_HUSK_SPAWN_EGG", "CAMEL_SPAWN_EGG", "CARROT", "CARROT_ON_A_STICK", "CAT_SPAWN_EGG", "CAVE_SPIDER_SPAWN_EGG", "CHAINMAIL_BOOTS", "CHAINMAIL_CHESTPLATE", "CHAINMAIL_HELMET", "CHAINMAIL_LEGGINGS", "CHARCOAL", "CHERRY_BOAT", "CHERRY_CHEST_BOAT", "CHEST_MINECART", "CHICKEN", "CHICKEN_SPAWN_EGG", "CHORUS_FRUIT", "CLAY_BALL", "CLOCK", "COAL", "COAST_ARMOR_TRIM_SMITHING_TEMPLATE", "COCOA_BEANS", "COD", "COD_BUCKET", "COD_SPAWN_EGG", "COMMAND_BLOCK_MINECART", "COMPASS", "COOKED_BEEF", "COOKED_CHICKEN", "COOKED_COD", "COOKED_MUTTON", "COOKED_PORKCHOP", "COOKED_RABBIT", "COOKED_SALMON", "COOKIE", "COPPER_AXE", "COPPER_BOOTS", "COPPER_CHESTPLATE", "COPPER_GOLEM_SPAWN_EGG", "COPPER_HELMET", "COPPER_HOE", "COPPER_HORSE_ARMOR", "COPPER_INGOT", "COPPER_LEGGINGS", "COPPER_NAUTILUS_ARMOR", "COPPER_NUGGET", "COPPER_PICKAXE", "COPPER_SHOVEL", "COPPER_SPEAR", "COPPER_SWORD", "COW_SPAWN_EGG", "CREAKING_SPAWN_EGG", "CREEPER_BANNER_PATTERN", "CREEPER_SPAWN_EGG", "CROSSBOW", "CYAN_BUNDLE", "CYAN_CUSHION", "CYAN_DYE", "CYAN_HARNESS", "DANGER_POTTERY_SHERD", "DARK_OAK_BOAT", "DARK_OAK_CHEST_BOAT", "DEBUG_STICK", "DESERT_PYRAMID_MAP", "DESERT_VILLAGE_MAP", "DIAMOND", "DIAMOND_AXE", "DIAMOND_BOOTS", "DIAMOND_CHESTPLATE", "DIAMOND_HELMET", "DIAMOND_HOE", "DIAMOND_HORSE_ARMOR", "DIAMOND_LEGGINGS", "DIAMOND_NAUTILUS_ARMOR", "DIAMOND_PICKAXE", "DIAMOND_SHOVEL", "DIAMOND_SPEAR", "DIAMOND_SWORD", "DISC_FRAGMENT_5", "DOLPHIN_SPAWN_EGG", "DONKEY_SPAWN_EGG", "DRAGON_BREATH", "DRIED_KELP", "DROWNED_SPAWN_EGG", "DUNE_ARMOR_TRIM_SMITHING_TEMPLATE", "ECHO_SHARD", "EGG", "ELDER_GUARDIAN_SPAWN_EGG", "ELYTRA", "EMERALD", "ENCHANTED_BOOK", "ENCHANTED_GOLDEN_APPLE", "END_CRYSTAL", "ENDER_DRAGON_SPAWN_EGG", "ENDER_EYE", "ENDER_PEARL", "ENDERMAN_SPAWN_EGG", "ENDERMITE_SPAWN_EGG", "EVOKER_SPAWN_EGG", "EXPERIENCE_BOTTLE", "EXPLORER_POTTERY_SHERD", "EYE_ARMOR_TRIM_SMITHING_TEMPLATE", "FEATHER", "FERMENTED_SPIDER_EYE", "FIELD_MASONED_BANNER_PATTERN", "FILLED_MAP", "FIRE_CHARGE", "FIREWORK_ROCKET", "FIREWORK_STAR", "FISHING_ROD", "FLINT", "FLINT_AND_STEEL", "FLOW_ARMOR_TRIM_SMITHING_TEMPLATE", "FLOW_BANNER_PATTERN", "FLOW_POTTERY_SHERD", "FLOWER_BANNER_PATTERN", "FOX_SPAWN_EGG", "FRIEND_POTTERY_SHERD", "FROG_SPAWN_EGG", "FURNACE_MINECART", "GHAST_SPAWN_EGG", "GHAST_TEAR", "GLASS_BOTTLE", "GLISTERING_MELON_SLICE", "GLOBE_BANNER_PATTERN", "GLOW_BERRIES", "GLOW_INK_SAC", "GLOW_ITEM_FRAME", "GLOW_SQUID_SPAWN_EGG", "GLOWSTONE_DUST", "GOAT_HORN", "GOAT_SPAWN_EGG", "GOLD_INGOT", "GOLD_NUGGET", "GOLDEN_APPLE", "GOLDEN_AXE", "GOLDEN_BOOTS", "GOLDEN_CARROT", "GOLDEN_CHESTPLATE", "GOLDEN_HELMET", "GOLDEN_HOE", "GOLDEN_HORSE_ARMOR", "GOLDEN_LEGGINGS", "GOLDEN_NAUTILUS_ARMOR", "GOLDEN_PICKAXE", "GOLDEN_SHOVEL", "GOLDEN_SPEAR", "GOLDEN_SWORD", "GRAY_BUNDLE", "GRAY_CUSHION", "GRAY_DYE", "GRAY_HARNESS", "GREEN_BUNDLE", "GREEN_CUSHION", "GREEN_DYE", "GREEN_HARNESS", "GUARDIAN_SPAWN_EGG", "GUNPOWDER", "GUSTER_BANNER_PATTERN", "GUSTER_POTTERY_SHERD", "HAPPY_GHAST_SPAWN_EGG", "HEART_OF_THE_SEA", "HEART_POTTERY_SHERD", "HEARTBREAK_POTTERY_SHERD", "HOGLIN_SPAWN_EGG", "HONEY_BOTTLE", "HONEYCOMB", "HOPPER_MINECART", "HORSE_SPAWN_EGG", "HOST_ARMOR_TRIM_SMITHING_TEMPLATE", "HOWL_POTTERY_SHERD", "HUSK_SPAWN_EGG", "INK_SAC", "IRON_AXE", "IRON_BOOTS", "IRON_CHESTPLATE", "IRON_GOLEM_SPAWN_EGG", "IRON_HELMET", "IRON_HOE", "IRON_HORSE_ARMOR", "IRON_INGOT", "IRON_LEGGINGS", "IRON_NAUTILUS_ARMOR", "IRON_NUGGET", "IRON_PICKAXE", "IRON_SHOVEL", "IRON_SPEAR", "IRON_SWORD", "ITEM_FRAME", "JUNGLE_BOAT", "JUNGLE_CHEST_BOAT", "JUNGLE_PYRAMID_MAP", "KNOWLEDGE_BOOK", "LAPIS_LAZULI", "LAVA_BUCKET", "LEAD", "LEATHER", "LEATHER_BOOTS", "LEATHER_CHESTPLATE", "LEATHER_HELMET", "LEATHER_HORSE_ARMOR", "LEATHER_LEGGINGS", "LIGHT_BLUE_BUNDLE", "LIGHT_BLUE_CUSHION", "LIGHT_BLUE_DYE", "LIGHT_BLUE_HARNESS", "LIGHT_GRAY_BUNDLE", "LIGHT_GRAY_CUSHION", "LIGHT_GRAY_DYE", "LIGHT_GRAY_HARNESS", "LIME_BUNDLE", "LIME_CUSHION", "LIME_DYE", "LIME_HARNESS", "LINGERING_POTION", "LLAMA_SPAWN_EGG", "MACE", "MAGENTA_BUNDLE", "MAGENTA_CUSHION", "MAGENTA_DYE", "MAGENTA_HARNESS", "MAGMA_CREAM", "MAGMA_CUBE_SPAWN_EGG", "MANGROVE_BOAT", "MANGROVE_CHEST_BOAT", "MAP", "MELON_SEEDS", "MELON_SLICE", "MILK_BUCKET", "MINECART", "MINER_POTTERY_SHERD", "MOJANG_BANNER_PATTERN", "MOOSHROOM_SPAWN_EGG", "MOURNER_POTTERY_SHERD", "MULE_SPAWN_EGG", "MUSHROOM_STEW", "MUSIC_DISC_5", "MUSIC_DISC_11", "MUSIC_DISC_13", "MUSIC_DISC_BLOCKS", "MUSIC_DISC_BOUNCE", "MUSIC_DISC_CAT", "MUSIC_DISC_CHIRP", "MUSIC_DISC_CREATOR", "MUSIC_DISC_CREATOR_MUSIC_BOX", "MUSIC_DISC_FAR", "MUSIC_DISC_LAVA_CHICKEN", "MUSIC_DISC_MALL", "MUSIC_DISC_MELLOHI", "MUSIC_DISC_OTHERSIDE", "MUSIC_DISC_PIGSTEP", "MUSIC_DISC_PRECIPICE", "MUSIC_DISC_RELIC", "MUSIC_DISC_STAL", "MUSIC_DISC_STRAD", "MUSIC_DISC_TEARS", "MUSIC_DISC_WAIT", "MUSIC_DISC_WARD", "MUTTON", "NAME_TAG", "NAUTILUS_SHELL", "NAUTILUS_SPAWN_EGG", "NETHER_BRICK", "NETHER_STAR", "NETHERITE_AXE", "NETHERITE_BOOTS", "NETHERITE_CHESTPLATE", "NETHERITE_HELMET", "NETHERITE_HOE", "NETHERITE_HORSE_ARMOR", "NETHERITE_INGOT", "NETHERITE_LEGGINGS", "NETHERITE_NAUTILUS_ARMOR", "NETHERITE_PICKAXE", "NETHERITE_SCRAP", "NETHERITE_SHOVEL", "NETHERITE_SPEAR", "NETHERITE_SWORD", "NETHERITE_UPGRADE_SMITHING_TEMPLATE", "OAK_BOAT", "OAK_CHEST_BOAT", "OCEAN_MONUMENT_MAP", "OCELOT_SPAWN_EGG", "OMINOUS_BOTTLE", "OMINOUS_TRIAL_KEY", "ORANGE_BUNDLE", "ORANGE_CUSHION", "ORANGE_DYE", "ORANGE_HARNESS", "PAINTING", "PALE_OAK_BOAT", "PALE_OAK_CHEST_BOAT", "PANDA_SPAWN_EGG", "PAPER", "PARCHED_SPAWN_EGG", "PARROT_SPAWN_EGG", "PHANTOM_MEMBRANE", "PHANTOM_SPAWN_EGG", "PIG_SPAWN_EGG", "PIGLIN_BANNER_PATTERN", "PIGLIN_BRUTE_SPAWN_EGG", "PIGLIN_SPAWN_EGG", "PILLAGER_SPAWN_EGG", "PINK_BUNDLE", "PINK_CUSHION", "PINK_DYE", "PINK_HARNESS", "PITCHER_POD", "PLAINS_VILLAGE_MAP", "PLENTY_POTTERY_SHERD", "POISONOUS_POTATO", "POLAR_BEAR_SPAWN_EGG", "POPLAR_BOAT", "POPLAR_CHEST_BOAT", "POPPED_CHORUS_FRUIT", "PORKCHOP", "POTATO", "POTION", "POWDER_SNOW_BUCKET", "PRISMARINE_CRYSTALS", "PRISMARINE_SHARD", "PRIZE_POTTERY_SHERD", "PUFFERFISH", "PUFFERFISH_BUCKET", "PUFFERFISH_SPAWN_EGG", "PUMPKIN_PIE", "PUMPKIN_SEEDS", "PURPLE_BUNDLE", "PURPLE_CUSHION", "PURPLE_DYE", "PURPLE_HARNESS", "QUARTZ", "RABBIT", "RABBIT_FOOT", "RABBIT_HIDE", "RABBIT_SPAWN_EGG", "RABBIT_STEW", "RAISER_ARMOR_TRIM_SMITHING_TEMPLATE", "RAVAGER_SPAWN_EGG", "RAW_COPPER", "RAW_GOLD", "RAW_IRON", "RECOVERY_COMPASS", "RED_BUNDLE", "RED_CUSHION", "RED_DYE", "RED_HARNESS", "REDSTONE", "RESIN_BRICK", "RIB_ARMOR_TRIM_SMITHING_TEMPLATE", "ROTTEN_FLESH", "SADDLE", "SALMON", "SALMON_BUCKET", "SALMON_SPAWN_EGG", "SAVANNA_VILLAGE_MAP", "SCRAPE_POTTERY_SHERD", "SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE", "SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE", "SHEAF_POTTERY_SHERD", "SHEARS", "SHEEP_SPAWN_EGG", "SHELTER_POTTERY_SHERD", "SHIELD", "SHULKER_SHELL", "SHULKER_SPAWN_EGG", "SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE", "SILVERFISH_SPAWN_EGG", "SKELETON_HORSE_SPAWN_EGG", "SKELETON_SPAWN_EGG", "SKULL_BANNER_PATTERN", "SKULL_POTTERY_SHERD", "SLIME_BALL", "SLIME_SPAWN_EGG", "SNIFFER_SPAWN_EGG", "SNORT_POTTERY_SHERD", "SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE", "SNOW_GOLEM_SPAWN_EGG", "SNOWBALL", "SNOWY_VILLAGE_MAP", "SPECTRAL_ARROW", "SPIDER_EYE", "SPIDER_SPAWN_EGG", "SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE", "SPLASH_POTION", "SPRUCE_BOAT", "SPRUCE_CHEST_BOAT", "SPYGLASS", "SQUID_SPAWN_EGG", "STICK", "STONE_AXE", "STONE_HOE", "STONE_PICKAXE", "STONE_SHOVEL", "STONE_SPEAR", "STONE_SWORD", "STRAY_SPAWN_EGG", "STRIDER_SPAWN_EGG", "STRING", "SUGAR", "SULFUR_CUBE_BUCKET", "SULFUR_CUBE_SPAWN_EGG", "SUSPICIOUS_STEW", "SWAMP_HUT_MAP", "SWEET_BERRIES", "TADPOLE_BUCKET", "TADPOLE_SPAWN_EGG", "TAIGA_VILLAGE_MAP", "TIDE_ARMOR_TRIM_SMITHING_TEMPLATE", "TIPPED_ARROW", "TNT_MINECART", "TORCHFLOWER_SEEDS", "TOTEM_OF_UNDYING", "TRADER_LLAMA_SPAWN_EGG", "TRIAL_KEY", "TRIDENT", "TROPICAL_FISH", "TROPICAL_FISH_BUCKET", "TROPICAL_FISH_SPAWN_EGG", "TURTLE_HELMET", "TURTLE_SCUTE", "TURTLE_SPAWN_EGG", "VEX_ARMOR_TRIM_SMITHING_TEMPLATE", "VEX_SPAWN_EGG", "VILLAGER_SPAWN_EGG", "VINDICATOR_SPAWN_EGG", "WANDERING_TRADER_SPAWN_EGG", "WARD_ARMOR_TRIM_SMITHING_TEMPLATE", "WARDEN_SPAWN_EGG", "WARM_OCEAN_RUINS_MAP", "WARPED_FUNGUS_ON_A_STICK", "WATER_BUCKET", "WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE", "WHEAT_SEEDS", "WHITE_BUNDLE", "WHITE_CUSHION", "WHITE_DYE", "WHITE_HARNESS", "WILD_ARMOR_TRIM_SMITHING_TEMPLATE", "WIND_CHARGE", "WITCH_SPAWN_EGG", "WITHER_SKELETON_SPAWN_EGG", "WITHER_SPAWN_EGG", "WOLF_ARMOR", "WOLF_SPAWN_EGG", "WOODEN_AXE", "WOODEN_HOE", "WOODEN_PICKAXE", "WOODEN_SHOVEL", "WOODEN_SPEAR", "WOODEN_SWORD", "WOODLAND_MANSION_MAP", "WRITABLE_BOOK", "WRITTEN_BOOK", "YELLOW_BUNDLE", "YELLOW_CUSHION", "YELLOW_DYE", "YELLOW_HARNESS", "ZOGLIN_SPAWN_EGG", "ZOMBIE_HORSE_SPAWN_EGG", "ZOMBIE_NAUTILUS_SPAWN_EGG", "ZOMBIE_SPAWN_EGG", "ZOMBIE_VILLAGER_SPAWN_EGG", "ZOMBIFIED_PIGLIN_SPAWN_EGG", "ACACIA_BUTTON", "ACACIA_DOOR", "ACACIA_FENCE", "ACACIA_FENCE_GATE", "ACACIA_HANGING_SIGN", "ACACIA_LEAVES", "ACACIA_LOG", "ACACIA_PLANKS", "ACACIA_PRESSURE_PLATE", "ACACIA_SAPLING", "ACACIA_SHELF", "ACACIA_SIGN", "ACACIA_SLAB", "ACACIA_STAIRS", "ACACIA_TRAPDOOR", "ACACIA_WALL_HANGING_SIGN", "ACACIA_WALL_SIGN", "ACACIA_WOOD", "ACTIVATOR_RAIL", "ALLIUM", "AMETHYST_BLOCK", "AMETHYST_CLUSTER", "ANCIENT_DEBRIS", "ANDESITE", "ANDESITE_SLAB", "ANDESITE_STAIRS", "ANDESITE_WALL", "ANVIL", "ATTACHED_MELON_STEM", "ATTACHED_PUMPKIN_STEM", "AZALEA", "AZALEA_LEAVES", "AZURE_BLUET", "BAMBOO", "BAMBOO_BLOCK", "BAMBOO_BUTTON", "BAMBOO_DOOR", "BAMBOO_FENCE", "BAMBOO_FENCE_GATE", "BAMBOO_HANGING_SIGN", "BAMBOO_MOSAIC", "BAMBOO_MOSAIC_SLAB", "BAMBOO_MOSAIC_STAIRS", "BAMBOO_PLANKS", "BAMBOO_PRESSURE_PLATE", "BAMBOO_SAPLING", "BAMBOO_SHELF", "BAMBOO_SIGN", "BAMBOO_SLAB", "BAMBOO_STAIRS", "BAMBOO_TRAPDOOR", "BAMBOO_WALL_HANGING_SIGN", "BAMBOO_WALL_SIGN", "BARREL", "BARRIER", "BASALT", "BEACON", "BEDROCK", "BEE_NEST", "BEEHIVE", "BEETROOTS", "BELL", "BIG_DRIPLEAF", "BIG_DRIPLEAF_STEM", "BIRCH_BUTTON", "BIRCH_DOOR", "BIRCH_FENCE", "BIRCH_FENCE_GATE", "BIRCH_HANGING_SIGN", "BIRCH_LEAVES", "BIRCH_LOG", "BIRCH_PLANKS", "BIRCH_PRESSURE_PLATE", "BIRCH_SAPLING", "BIRCH_SHELF", "BIRCH_SIGN", "BIRCH_SLAB", "BIRCH_STAIRS", "BIRCH_TRAPDOOR", "BIRCH_WALL_HANGING_SIGN", "BIRCH_WALL_SIGN", "BIRCH_WOOD", "BLACK_BANNER", "BLACK_BED", "BLACK_CANDLE", "BLACK_CANDLE_CAKE", "BLACK_CARPET", "BLACK_CONCRETE", "BLACK_CONCRETE_POWDER", "BLACK_CONCRETE_SLAB", "BLACK_CONCRETE_STAIRS", "BLACK_GLAZED_TERRACOTTA", "BLACK_SHULKER_BOX", "BLACK_STAINED_GLASS", "BLACK_STAINED_GLASS_PANE", "BLACK_TERRACOTTA", "BLACK_WALL_BANNER", "BLACK_WOOL", "BLACK_WOOL_SLAB", "BLACK_WOOL_STAIRS", "BLACKSTONE", "BLACKSTONE_SLAB", "BLACKSTONE_STAIRS", "BLACKSTONE_WALL", "BLAST_FURNACE", "BLUE_BANNER", "BLUE_BED", "BLUE_CANDLE", "BLUE_CANDLE_CAKE", "BLUE_CARPET", "BLUE_CONCRETE", "BLUE_CONCRETE_POWDER", "BLUE_CONCRETE_SLAB", "BLUE_CONCRETE_STAIRS", "BLUE_GLAZED_TERRACOTTA", "BLUE_ICE", "BLUE_ORCHID", "BLUE_SHULKER_BOX", "BLUE_STAINED_GLASS", "BLUE_STAINED_GLASS_PANE", "BLUE_TERRACOTTA", "BLUE_WALL_BANNER", "BLUE_WOOL", "BLUE_WOOL_SLAB", "BLUE_WOOL_STAIRS", "BONE_BLOCK", "BOOKSHELF", "BRAIN_CORAL", "BRAIN_CORAL_BLOCK", "BRAIN_CORAL_FAN", "BRAIN_CORAL_WALL_FAN", "BREWING_STAND", "BRICK_SLAB", "BRICK_STAIRS", "BRICK_WALL", "BRICKS", "BROWN_BANNER", "BROWN_BED", "BROWN_CANDLE", "BROWN_CANDLE_CAKE", "BROWN_CARPET", "BROWN_CONCRETE", "BROWN_CONCRETE_POWDER", "BROWN_CONCRETE_SLAB", "BROWN_CONCRETE_STAIRS", "BROWN_GLAZED_TERRACOTTA", "BROWN_MUSHROOM", "BROWN_MUSHROOM_BLOCK", "BROWN_SHULKER_BOX", "BROWN_STAINED_GLASS", "BROWN_STAINED_GLASS_PANE", "BROWN_TERRACOTTA", "BROWN_WALL_BANNER", "BROWN_WOOL", "BROWN_WOOL_SLAB", "BROWN_WOOL_STAIRS", "BUBBLE_COLUMN", "BUBBLE_CORAL", "BUBBLE_CORAL_BLOCK", "BUBBLE_CORAL_FAN", "BUBBLE_CORAL_WALL_FAN", "BUDDING_AMETHYST", "BUSH", "CACTUS", "CACTUS_FLOWER", "CAKE", "CALCITE", "CALIBRATED_SCULK_SENSOR", "CAMPFIRE", "CANDLE", "CANDLE_CAKE", "CARROTS", "CARTOGRAPHY_TABLE", "CARVED_PUMPKIN", "CAULDRON", "CAVE_AIR", "CAVE_VINES", "CAVE_VINES_PLANT", "CHAIN_COMMAND_BLOCK", "CHERRY_BUTTON", "CHERRY_DOOR", "CHERRY_FENCE", "CHERRY_FENCE_GATE", "CHERRY_HANGING_SIGN", "CHERRY_LEAVES", "CHERRY_LOG", "CHERRY_PLANKS", "CHERRY_PRESSURE_PLATE", "CHERRY_SAPLING", "CHERRY_SHELF", "CHERRY_SIGN", "CHERRY_SLAB", "CHERRY_STAIRS", "CHERRY_TRAPDOOR", "CHERRY_WALL_HANGING_SIGN", "CHERRY_WALL_SIGN", "CHERRY_WOOD", "CHEST", "CHIPPED_ANVIL", "CHISELED_BOOKSHELF", "CHISELED_CINNABAR", "CHISELED_COPPER", "CHISELED_DEEPSLATE", "CHISELED_NETHER_BRICKS", "CHISELED_POLISHED_BLACKSTONE", "CHISELED_QUARTZ_BLOCK", "CHISELED_RED_SANDSTONE", "CHISELED_RESIN_BRICKS", "CHISELED_SANDSTONE", "CHISELED_STONE_BRICKS", "CHISELED_SULFUR", "CHISELED_TUFF", "CHISELED_TUFF_BRICKS", "CHORUS_FLOWER", "CHORUS_PLANT", "CINNABAR", "CINNABAR_BRICK_SLAB", "CINNABAR_BRICK_STAIRS", "CINNABAR_BRICK_WALL", "CINNABAR_BRICKS", "CINNABAR_SLAB", "CINNABAR_STAIRS", "CINNABAR_WALL", "CLAY", "CLOSED_EYEBLOSSOM", "COAL_BLOCK", "COAL_ORE", "COARSE_DIRT", "COBBLED_DEEPSLATE", "COBBLED_DEEPSLATE_SLAB", "COBBLED_DEEPSLATE_STAIRS", "COBBLED_DEEPSLATE_WALL", "COBBLESTONE", "COBBLESTONE_SLAB", "COBBLESTONE_STAIRS", "COBBLESTONE_WALL", "COBWEB", "COCOA", "COMMAND_BLOCK", "COMPARATOR", "COMPOSTER", "CONDUIT", "COPPER_BARS", "COPPER_BLOCK", "COPPER_BULB", "COPPER_CHAIN", "COPPER_CHEST", "COPPER_DOOR", "COPPER_GOLEM_STATUE", "COPPER_GRATE", "COPPER_LANTERN", "COPPER_ORE", "COPPER_TORCH", "COPPER_TRAPDOOR", "COPPER_WALL_TORCH", "CORNFLOWER", "CRACKED_DEEPSLATE_BRICKS", "CRACKED_DEEPSLATE_TILES", "CRACKED_NETHER_BRICKS", "CRACKED_POLISHED_BLACKSTONE_BRICKS", "CRACKED_STONE_BRICKS", "CRAFTER", "CRAFTING_TABLE", "CREAKING_HEART", "CREEPER_HEAD", "CREEPER_WALL_HEAD", "CRIMSON_BUTTON", "CRIMSON_DOOR", "CRIMSON_FENCE", "CRIMSON_FENCE_GATE", "CRIMSON_FUNGUS", "CRIMSON_HANGING_SIGN", "CRIMSON_HYPHAE", "CRIMSON_NYLIUM", "CRIMSON_PLANKS", "CRIMSON_PRESSURE_PLATE", "CRIMSON_ROOTS", "CRIMSON_SHELF", "CRIMSON_SIGN", "CRIMSON_SLAB", "CRIMSON_STAIRS", "CRIMSON_STEM", "CRIMSON_TRAPDOOR", "CRIMSON_WALL_HANGING_SIGN", "CRIMSON_WALL_SIGN", "CRYING_OBSIDIAN", "CUT_COPPER", "CUT_COPPER_SLAB", "CUT_COPPER_STAIRS", "CUT_RED_SANDSTONE", "CUT_RED_SANDSTONE_SLAB", "CUT_SANDSTONE", "CUT_SANDSTONE_SLAB", "CYAN_BANNER", "CYAN_BED", "CYAN_CANDLE", "CYAN_CANDLE_CAKE", "CYAN_CARPET", "CYAN_CONCRETE", "CYAN_CONCRETE_POWDER", "CYAN_CONCRETE_SLAB", "CYAN_CONCRETE_STAIRS", "CYAN_GLAZED_TERRACOTTA", "CYAN_SHULKER_BOX", "CYAN_STAINED_GLASS", "CYAN_STAINED_GLASS_PANE", "CYAN_TERRACOTTA", "CYAN_WALL_BANNER", "CYAN_WOOL", "CYAN_WOOL_SLAB", "CYAN_WOOL_STAIRS", "DAMAGED_ANVIL", "DANDELION", "DARK_OAK_BUTTON", "DARK_OAK_DOOR", "DARK_OAK_FENCE", "DARK_OAK_FENCE_GATE", "DARK_OAK_HANGING_SIGN", "DARK_OAK_LEAVES", "DARK_OAK_LOG", "DARK_OAK_PLANKS", "DARK_OAK_PRESSURE_PLATE", "DARK_OAK_SAPLING", "DARK_OAK_SHELF", "DARK_OAK_SIGN", "DARK_OAK_SLAB", "DARK_OAK_STAIRS", "DARK_OAK_TRAPDOOR", "DARK_OAK_WALL_HANGING_SIGN", "DARK_OAK_WALL_SIGN", "DARK_OAK_WOOD", "DARK_PRISMARINE", "DARK_PRISMARINE_SLAB", "DARK_PRISMARINE_STAIRS", "DAYLIGHT_DETECTOR", "DEAD_BRAIN_CORAL", "DEAD_BRAIN_CORAL_BLOCK", "DEAD_BRAIN_CORAL_FAN", "DEAD_BRAIN_CORAL_WALL_FAN", "DEAD_BUBBLE_CORAL", "DEAD_BUBBLE_CORAL_BLOCK", "DEAD_BUBBLE_CORAL_FAN", "DEAD_BUBBLE_CORAL_WALL_FAN", "DEAD_BUSH", "DEAD_FIRE_CORAL", "DEAD_FIRE_CORAL_BLOCK", "DEAD_FIRE_CORAL_FAN", "DEAD_FIRE_CORAL_WALL_FAN", "DEAD_HORN_CORAL", "DEAD_HORN_CORAL_BLOCK", "DEAD_HORN_CORAL_FAN", "DEAD_HORN_CORAL_WALL_FAN", "DEAD_TUBE_CORAL", "DEAD_TUBE_CORAL_BLOCK", "DEAD_TUBE_CORAL_FAN", "DEAD_TUBE_CORAL_WALL_FAN", "DECORATED_POT", "DEEPSLATE", "DEEPSLATE_BRICK_SLAB", "DEEPSLATE_BRICK_STAIRS", "DEEPSLATE_BRICK_WALL", "DEEPSLATE_BRICKS", "DEEPSLATE_COAL_ORE", "DEEPSLATE_COPPER_ORE", "DEEPSLATE_DIAMOND_ORE", "DEEPSLATE_EMERALD_ORE", "DEEPSLATE_GOLD_ORE", "DEEPSLATE_IRON_ORE", "DEEPSLATE_LAPIS_ORE", "DEEPSLATE_REDSTONE_ORE", "DEEPSLATE_TILE_SLAB", "DEEPSLATE_TILE_STAIRS", "DEEPSLATE_TILE_WALL", "DEEPSLATE_TILES", "DETECTOR_RAIL", "DIAMOND_BLOCK", "DIAMOND_ORE", "DIORITE", "DIORITE_SLAB", "DIORITE_STAIRS", "DIORITE_WALL", "DIRT", "DIRT_PATH", "DISPENSER", "DRAGON_EGG", "DRAGON_HEAD", "DRAGON_WALL_HEAD", "DRIED_GHAST", "DRIED_KELP_BLOCK", "DRIPSTONE_BLOCK", "DROPPER", "EMERALD_BLOCK", "EMERALD_ORE", "ENCHANTING_TABLE", "END_GATEWAY", "END_PORTAL", "END_PORTAL_FRAME", "END_ROD", "END_STONE", "END_STONE_BRICK_SLAB", "END_STONE_BRICK_STAIRS", "END_STONE_BRICK_WALL", "END_STONE_BRICKS", "ENDER_CHEST", "EXPOSED_CHISELED_COPPER", "EXPOSED_COPPER", "EXPOSED_COPPER_BARS", "EXPOSED_COPPER_BULB", "EXPOSED_COPPER_CHAIN", "EXPOSED_COPPER_CHEST", "EXPOSED_COPPER_DOOR", "EXPOSED_COPPER_GOLEM_STATUE", "EXPOSED_COPPER_GRATE", "EXPOSED_COPPER_LANTERN", "EXPOSED_COPPER_TRAPDOOR", "EXPOSED_CUT_COPPER", "EXPOSED_CUT_COPPER_SLAB", "EXPOSED_CUT_COPPER_STAIRS", "EXPOSED_LIGHTNING_ROD", "FARMLAND", "FERN", "FIRE", "FIRE_CORAL", "FIRE_CORAL_BLOCK", "FIRE_CORAL_FAN", "FIRE_CORAL_WALL_FAN", "FIREFLY_BUSH", "FLETCHING_TABLE", "FLOWER_POT", "FLOWERING_AZALEA", "FLOWERING_AZALEA_LEAVES", "FROGSPAWN", "FROSTED_ICE", "FURNACE", "GILDED_BLACKSTONE", "GLASS", "GLASS_PANE", "GLOW_LICHEN", "GLOWSTONE", "GOLD_BLOCK", "GOLD_ORE", "GOLDEN_DANDELION", "GRANITE", "GRANITE_SLAB", "GRANITE_STAIRS", "GRANITE_WALL", "GRASS_BLOCK", "GRAVEL", "GRAY_BANNER", "GRAY_BED", "GRAY_CANDLE", "GRAY_CANDLE_CAKE", "GRAY_CARPET", "GRAY_CONCRETE", "GRAY_CONCRETE_POWDER", "GRAY_CONCRETE_SLAB", "GRAY_CONCRETE_STAIRS", "GRAY_GLAZED_TERRACOTTA", "GRAY_SHULKER_BOX", "GRAY_STAINED_GLASS", "GRAY_STAINED_GLASS_PANE", "GRAY_TERRACOTTA", "GRAY_WALL_BANNER", "GRAY_WOOL", "GRAY_WOOL_SLAB", "GRAY_WOOL_STAIRS", "GREEN_BANNER", "GREEN_BED", "GREEN_CANDLE", "GREEN_CANDLE_CAKE", "GREEN_CARPET", "GREEN_CONCRETE", "GREEN_CONCRETE_POWDER", "GREEN_CONCRETE_SLAB", "GREEN_CONCRETE_STAIRS", "GREEN_GLAZED_TERRACOTTA", "GREEN_SHULKER_BOX", "GREEN_STAINED_GLASS", "GREEN_STAINED_GLASS_PANE", "GREEN_TERRACOTTA", "GREEN_WALL_BANNER", "GREEN_WOOL", "GREEN_WOOL_SLAB", "GREEN_WOOL_STAIRS", "GRINDSTONE", "HANGING_ROOTS", "HAY_BLOCK", "HEAVY_CORE", "HEAVY_WEIGHTED_PRESSURE_PLATE", "HONEY_BLOCK", "HONEYCOMB_BLOCK", "HOPPER", "HORN_CORAL", "HORN_CORAL_BLOCK", "HORN_CORAL_FAN", "HORN_CORAL_WALL_FAN", "ICE", "INFESTED_CHISELED_STONE_BRICKS", "INFESTED_COBBLESTONE", "INFESTED_CRACKED_STONE_BRICKS", "INFESTED_DEEPSLATE", "INFESTED_MOSSY_STONE_BRICKS", "INFESTED_STONE", "INFESTED_STONE_BRICKS", "IRON_BARS", "IRON_BLOCK", "IRON_CHAIN", "IRON_DOOR", "IRON_ORE", "IRON_TRAPDOOR", "JACK_O_LANTERN", "JIGSAW", "JUKEBOX", "JUNGLE_BUTTON", "JUNGLE_DOOR", "JUNGLE_FENCE", "JUNGLE_FENCE_GATE", "JUNGLE_HANGING_SIGN", "JUNGLE_LEAVES", "JUNGLE_LOG", "JUNGLE_PLANKS", "JUNGLE_PRESSURE_PLATE", "JUNGLE_SAPLING", "JUNGLE_SHELF", "JUNGLE_SIGN", "JUNGLE_SLAB", "JUNGLE_STAIRS", "JUNGLE_TRAPDOOR", "JUNGLE_WALL_HANGING_SIGN", "JUNGLE_WALL_SIGN", "JUNGLE_WOOD", "KELP", "KELP_PLANT", "LADDER", "LANTERN", "LAPIS_BLOCK", "LAPIS_ORE", "LARGE_AMETHYST_BUD", "LARGE_FERN", "LAVA", "LAVA_CAULDRON", "LEAF_LITTER", "LECTERN", "LEVER", "LIGHT", "LIGHT_BLUE_BANNER", "LIGHT_BLUE_BED", "LIGHT_BLUE_CANDLE", "LIGHT_BLUE_CANDLE_CAKE", "LIGHT_BLUE_CARPET", "LIGHT_BLUE_CONCRETE", "LIGHT_BLUE_CONCRETE_POWDER", "LIGHT_BLUE_CONCRETE_SLAB", "LIGHT_BLUE_CONCRETE_STAIRS", "LIGHT_BLUE_GLAZED_TERRACOTTA", "LIGHT_BLUE_SHULKER_BOX", "LIGHT_BLUE_STAINED_GLASS", "LIGHT_BLUE_STAINED_GLASS_PANE", "LIGHT_BLUE_TERRACOTTA", "LIGHT_BLUE_WALL_BANNER", "LIGHT_BLUE_WOOL", "LIGHT_BLUE_WOOL_SLAB", "LIGHT_BLUE_WOOL_STAIRS", "LIGHT_GRAY_BANNER", "LIGHT_GRAY_BED", "LIGHT_GRAY_CANDLE", "LIGHT_GRAY_CANDLE_CAKE", "LIGHT_GRAY_CARPET", "LIGHT_GRAY_CONCRETE", "LIGHT_GRAY_CONCRETE_POWDER", "LIGHT_GRAY_CONCRETE_SLAB", "LIGHT_GRAY_CONCRETE_STAIRS", "LIGHT_GRAY_GLAZED_TERRACOTTA", "LIGHT_GRAY_SHULKER_BOX", "LIGHT_GRAY_STAINED_GLASS", "LIGHT_GRAY_STAINED_GLASS_PANE", "LIGHT_GRAY_TERRACOTTA", "LIGHT_GRAY_WALL_BANNER", "LIGHT_GRAY_WOOL", "LIGHT_GRAY_WOOL_SLAB", "LIGHT_GRAY_WOOL_STAIRS", "LIGHT_WEIGHTED_PRESSURE_PLATE", "LIGHTNING_ROD", "LILAC", "LILY_OF_THE_VALLEY", "LILY_PAD", "LIME_BANNER", "LIME_BED", "LIME_CANDLE", "LIME_CANDLE_CAKE", "LIME_CARPET", "LIME_CONCRETE", "LIME_CONCRETE_POWDER", "LIME_CONCRETE_SLAB", "LIME_CONCRETE_STAIRS", "LIME_GLAZED_TERRACOTTA", "LIME_SHULKER_BOX", "LIME_STAINED_GLASS", "LIME_STAINED_GLASS_PANE", "LIME_TERRACOTTA", "LIME_WALL_BANNER", "LIME_WOOL", "LIME_WOOL_SLAB", "LIME_WOOL_STAIRS", "LODESTONE", "LOOM", "MAGENTA_BANNER", "MAGENTA_BED", "MAGENTA_CANDLE", "MAGENTA_CANDLE_CAKE", "MAGENTA_CARPET", "MAGENTA_CONCRETE", "MAGENTA_CONCRETE_POWDER", "MAGENTA_CONCRETE_SLAB", "MAGENTA_CONCRETE_STAIRS", "MAGENTA_GLAZED_TERRACOTTA", "MAGENTA_SHULKER_BOX", "MAGENTA_STAINED_GLASS", "MAGENTA_STAINED_GLASS_PANE", "MAGENTA_TERRACOTTA", "MAGENTA_WALL_BANNER", "MAGENTA_WOOL", "MAGENTA_WOOL_SLAB", "MAGENTA_WOOL_STAIRS", "MAGMA_BLOCK", "MANGROVE_BUTTON", "MANGROVE_DOOR", "MANGROVE_FENCE", "MANGROVE_FENCE_GATE", "MANGROVE_HANGING_SIGN", "MANGROVE_LEAVES", "MANGROVE_LOG", "MANGROVE_PLANKS", "MANGROVE_PRESSURE_PLATE", "MANGROVE_PROPAGULE", "MANGROVE_ROOTS", "MANGROVE_SHELF", "MANGROVE_SIGN", "MANGROVE_SLAB", "MANGROVE_STAIRS", "MANGROVE_TRAPDOOR", "MANGROVE_WALL_HANGING_SIGN", "MANGROVE_WALL_SIGN", "MANGROVE_WOOD", "MEDIUM_AMETHYST_BUD", "MELON", "MELON_STEM", "MOSS_BLOCK", "MOSS_CARPET", "MOSSY_COBBLESTONE", "MOSSY_COBBLESTONE_SLAB", "MOSSY_COBBLESTONE_STAIRS", "MOSSY_COBBLESTONE_WALL", "MOSSY_STONE_BRICK_SLAB", "MOSSY_STONE_BRICK_STAIRS", "MOSSY_STONE_BRICK_WALL", "MOSSY_STONE_BRICKS", "MOVING_PISTON", "MUD", "MUD_BRICK_SLAB", "MUD_BRICK_STAIRS", "MUD_BRICK_WALL", "MUD_BRICKS", "MUDDY_MANGROVE_ROOTS", "MUSHROOM_STEM", "MYCELIUM", "NETHER_BRICK_FENCE", "NETHER_BRICK_SLAB", "NETHER_BRICK_STAIRS", "NETHER_BRICK_WALL", "NETHER_BRICKS", "NETHER_GOLD_ORE", "NETHER_PORTAL", "NETHER_QUARTZ_ORE", "NETHER_SPROUTS", "NETHER_WART", "NETHER_WART_BLOCK", "NETHERITE_BLOCK", "NETHERRACK", "NOTE_BLOCK", "OAK_BUTTON", "OAK_DOOR", "OAK_FENCE", "OAK_FENCE_GATE", "OAK_HANGING_SIGN", "OAK_LEAVES", "OAK_LOG", "OAK_PLANKS", "OAK_PRESSURE_PLATE", "OAK_SAPLING", "OAK_SHELF", "OAK_SIGN", "OAK_SLAB", "OAK_STAIRS", "OAK_TRAPDOOR", "OAK_WALL_HANGING_SIGN", "OAK_WALL_SIGN", "OAK_WOOD", "OBSERVER", "OBSIDIAN", "OCHRE_FROGLIGHT", "OPEN_EYEBLOSSOM", "ORANGE_BANNER", "ORANGE_BED", "ORANGE_CANDLE", "ORANGE_CANDLE_CAKE", "ORANGE_CARPET", "ORANGE_CONCRETE", "ORANGE_CONCRETE_POWDER", "ORANGE_CONCRETE_SLAB", "ORANGE_CONCRETE_STAIRS", "ORANGE_GLAZED_TERRACOTTA", "ORANGE_POPLAR_LEAVES", "ORANGE_SHULKER_BOX", "ORANGE_STAINED_GLASS", "ORANGE_STAINED_GLASS_PANE", "ORANGE_TERRACOTTA", "ORANGE_TULIP", "ORANGE_WALL_BANNER", "ORANGE_WOOL", "ORANGE_WOOL_SLAB", "ORANGE_WOOL_STAIRS", "OXEYE_DAISY", "OXIDIZED_CHISELED_COPPER", "OXIDIZED_COPPER", "OXIDIZED_COPPER_BARS", "OXIDIZED_COPPER_BULB", "OXIDIZED_COPPER_CHAIN", "OXIDIZED_COPPER_CHEST", "OXIDIZED_COPPER_DOOR", "OXIDIZED_COPPER_GOLEM_STATUE", "OXIDIZED_COPPER_GRATE", "OXIDIZED_COPPER_LANTERN", "OXIDIZED_COPPER_TRAPDOOR", "OXIDIZED_CUT_COPPER", "OXIDIZED_CUT_COPPER_SLAB", "OXIDIZED_CUT_COPPER_STAIRS", "OXIDIZED_LIGHTNING_ROD", "PACKED_ICE", "PACKED_MUD", "PALE_HANGING_MOSS", "PALE_MOSS_BLOCK", "PALE_MOSS_CARPET", "PALE_OAK_BUTTON", "PALE_OAK_DOOR", "PALE_OAK_FENCE", "PALE_OAK_FENCE_GATE", "PALE_OAK_HANGING_SIGN", "PALE_OAK_LEAVES", "PALE_OAK_LOG", "PALE_OAK_PLANKS", "PALE_OAK_PRESSURE_PLATE", "PALE_OAK_SAPLING", "PALE_OAK_SHELF", "PALE_OAK_SIGN", "PALE_OAK_SLAB", "PALE_OAK_STAIRS", "PALE_OAK_TRAPDOOR", "PALE_OAK_WALL_HANGING_SIGN", "PALE_OAK_WALL_SIGN", "PALE_OAK_WOOD", "PEARLESCENT_FROGLIGHT", "PEONY", "PETRIFIED_OAK_SLAB", "PIGLIN_HEAD", "PIGLIN_WALL_HEAD", "PINK_BANNER", "PINK_BED", "PINK_CANDLE", "PINK_CANDLE_CAKE", "PINK_CARPET", "PINK_CONCRETE", "PINK_CONCRETE_POWDER", "PINK_CONCRETE_SLAB", "PINK_CONCRETE_STAIRS", "PINK_GLAZED_TERRACOTTA", "PINK_PETALS", "PINK_SHULKER_BOX", "PINK_STAINED_GLASS", "PINK_STAINED_GLASS_PANE", "PINK_TERRACOTTA", "PINK_TULIP", "PINK_WALL_BANNER", "PINK_WOOL", "PINK_WOOL_SLAB", "PINK_WOOL_STAIRS", "PISTON", "PISTON_HEAD", "PITCHER_CROP", "PITCHER_PLANT", "PLAYER_HEAD", "PLAYER_WALL_HEAD", "PODZOL", "POINTED_DRIPSTONE", "POLISHED_ANDESITE", "POLISHED_ANDESITE_SLAB", "POLISHED_ANDESITE_STAIRS", "POLISHED_BASALT", "POLISHED_BLACKSTONE", "POLISHED_BLACKSTONE_BRICK_SLAB", "POLISHED_BLACKSTONE_BRICK_STAIRS", "POLISHED_BLACKSTONE_BRICK_WALL", "POLISHED_BLACKSTONE_BRICKS", "POLISHED_BLACKSTONE_BUTTON", "POLISHED_BLACKSTONE_PRESSURE_PLATE", "POLISHED_BLACKSTONE_SLAB", "POLISHED_BLACKSTONE_STAIRS", "POLISHED_BLACKSTONE_WALL", "POLISHED_CINNABAR", "POLISHED_CINNABAR_SLAB", "POLISHED_CINNABAR_STAIRS", "POLISHED_CINNABAR_WALL", "POLISHED_DEEPSLATE", "POLISHED_DEEPSLATE_SLAB", "POLISHED_DEEPSLATE_STAIRS", "POLISHED_DEEPSLATE_WALL", "POLISHED_DIORITE", "POLISHED_DIORITE_SLAB", "POLISHED_DIORITE_STAIRS", "POLISHED_GRANITE", "POLISHED_GRANITE_SLAB", "POLISHED_GRANITE_STAIRS", "POLISHED_SULFUR", "POLISHED_SULFUR_SLAB", "POLISHED_SULFUR_STAIRS", "POLISHED_SULFUR_WALL", "POLISHED_TUFF", "POLISHED_TUFF_SLAB", "POLISHED_TUFF_STAIRS", "POLISHED_TUFF_WALL", "POPLAR_BUTTON", "POPLAR_DOOR", "POPLAR_FENCE", "POPLAR_FENCE_GATE", "POPLAR_HANGING_SIGN", "POPLAR_LOG", "POPLAR_PLANKS", "POPLAR_PRESSURE_PLATE", "POPLAR_SAPLING", "POPLAR_SHELF", "POPLAR_SIGN", "POPLAR_SLAB", "POPLAR_STAIRS", "POPLAR_TRAPDOOR", "POPLAR_WALL_HANGING_SIGN", "POPLAR_WALL_SIGN", "POPLAR_WOOD", "POPPY", "POTATOES", "POTENT_SULFUR", "POTTED_ACACIA_SAPLING", "POTTED_ALLIUM", "POTTED_AZALEA_BUSH", "POTTED_AZURE_BLUET", "POTTED_BAMBOO", "POTTED_BIRCH_SAPLING", "POTTED_BLUE_ORCHID", "POTTED_BROWN_MUSHROOM", "POTTED_CACTUS", "POTTED_CHERRY_SAPLING", "POTTED_CLOSED_EYEBLOSSOM", "POTTED_CORNFLOWER", "POTTED_CRIMSON_FUNGUS", "POTTED_CRIMSON_ROOTS", "POTTED_DANDELION", "POTTED_DARK_OAK_SAPLING", "POTTED_DEAD_BUSH", "POTTED_FERN", "POTTED_FLOWERING_AZALEA_BUSH", "POTTED_GOLDEN_DANDELION", "POTTED_JUNGLE_SAPLING", "POTTED_LILY_OF_THE_VALLEY", "POTTED_MANGROVE_PROPAGULE", "POTTED_OAK_SAPLING", "POTTED_OPEN_EYEBLOSSOM", "POTTED_ORANGE_TULIP", "POTTED_OXEYE_DAISY", "POTTED_PALE_OAK_SAPLING", "POTTED_PINK_TULIP", "POTTED_POPLAR_SAPLING", "POTTED_POPPY", "POTTED_RED_MUSHROOM", "POTTED_RED_TULIP", "POTTED_SPRUCE_SAPLING", "POTTED_TORCHFLOWER", "POTTED_WARPED_FUNGUS", "POTTED_WARPED_ROOTS", "POTTED_WHITE_TULIP", "POTTED_WITHER_ROSE", "POWDER_SNOW", "POWDER_SNOW_CAULDRON", "POWERED_RAIL", "PRISMARINE", "PRISMARINE_BRICK_SLAB", "PRISMARINE_BRICK_STAIRS", "PRISMARINE_BRICKS", "PRISMARINE_SLAB", "PRISMARINE_STAIRS", "PRISMARINE_WALL", "PUMPKIN", "PUMPKIN_STEM", "PURPLE_BANNER", "PURPLE_BED", "PURPLE_CANDLE", "PURPLE_CANDLE_CAKE", "PURPLE_CARPET", "PURPLE_CONCRETE", "PURPLE_CONCRETE_POWDER", "PURPLE_CONCRETE_SLAB", "PURPLE_CONCRETE_STAIRS", "PURPLE_GLAZED_TERRACOTTA", "PURPLE_SHULKER_BOX", "PURPLE_STAINED_GLASS", "PURPLE_STAINED_GLASS_PANE", "PURPLE_TERRACOTTA", "PURPLE_WALL_BANNER", "PURPLE_WOOL", "PURPLE_WOOL_SLAB", "PURPLE_WOOL_STAIRS", "PURPUR_BLOCK", "PURPUR_PILLAR", "PURPUR_SLAB", "PURPUR_STAIRS", "QUARTZ_BLOCK", "QUARTZ_BRICKS", "QUARTZ_PILLAR", "QUARTZ_SLAB", "QUARTZ_STAIRS", "RAIL", "RAW_COPPER_BLOCK", "RAW_GOLD_BLOCK", "RAW_IRON_BLOCK", "RED_BANNER", "RED_BED", "RED_CANDLE", "RED_CANDLE_CAKE", "RED_CARPET", "RED_CONCRETE", "RED_CONCRETE_POWDER", "RED_CONCRETE_SLAB", "RED_CONCRETE_STAIRS", "RED_GLAZED_TERRACOTTA", "RED_MUSHROOM", "RED_MUSHROOM_BLOCK", "RED_NETHER_BRICK_SLAB", "RED_NETHER_BRICK_STAIRS", "RED_NETHER_BRICK_WALL", "RED_NETHER_BRICKS", "RED_POPLAR_LEAVES", "RED_SAND", "RED_SANDSTONE", "RED_SANDSTONE_SLAB", "RED_SANDSTONE_STAIRS", "RED_SANDSTONE_WALL", "RED_SHRUB", "RED_SHULKER_BOX", "RED_STAINED_GLASS", "RED_STAINED_GLASS_PANE", "RED_TERRACOTTA", "RED_TULIP", "RED_WALL_BANNER", "RED_WOOL", "RED_WOOL_SLAB", "RED_WOOL_STAIRS", "REDSTONE_BLOCK", "REDSTONE_LAMP", "REDSTONE_ORE", "REDSTONE_TORCH", "REDSTONE_WALL_TORCH", "REDSTONE_WIRE", "REINFORCED_DEEPSLATE", "REPEATER", "REPEATING_COMMAND_BLOCK", "RESIN_BLOCK", "RESIN_BRICK_SLAB", "RESIN_BRICK_STAIRS", "RESIN_BRICK_WALL", "RESIN_BRICKS", "RESIN_CLUMP", "RESPAWN_ANCHOR", "ROOTED_DIRT", "ROSE_BUSH", "SAND", "SANDSTONE", "SANDSTONE_SLAB", "SANDSTONE_STAIRS", "SANDSTONE_WALL", "SCAFFOLDING", "SCULK", "SCULK_CATALYST", "SCULK_SENSOR", "SCULK_SHRIEKER", "SCULK_VEIN", "SEA_LANTERN", "SEA_PICKLE", "SEAGRASS", "SHELF_MUSHROOM", "SHORT_DRY_GRASS", "SHORT_GRASS", "SHROOMLIGHT", "SHULKER_BOX", "SKELETON_SKULL", "SKELETON_WALL_SKULL", "SLIME_BLOCK", "SMALL_AMETHYST_BUD", "SMALL_DRIPLEAF", "SMITHING_TABLE", "SMOKER", "SMOOTH_BASALT", "SMOOTH_QUARTZ", "SMOOTH_QUARTZ_SLAB", "SMOOTH_QUARTZ_STAIRS", "SMOOTH_RED_SANDSTONE", "SMOOTH_RED_SANDSTONE_SLAB", "SMOOTH_RED_SANDSTONE_STAIRS", "SMOOTH_SANDSTONE", "SMOOTH_SANDSTONE_SLAB", "SMOOTH_SANDSTONE_STAIRS", "SMOOTH_STONE", "SMOOTH_STONE_SLAB", "SNIFFER_EGG", "SNOW", "SNOW_BLOCK", "SOUL_CAMPFIRE", "SOUL_FIRE", "SOUL_LANTERN", "SOUL_SAND", "SOUL_SOIL", "SOUL_TORCH", "SOUL_WALL_TORCH", "SPAWNER", "SPONGE", "SPORE_BLOSSOM", "SPRUCE_BUTTON", "SPRUCE_DOOR", "SPRUCE_FENCE", "SPRUCE_FENCE_GATE", "SPRUCE_HANGING_SIGN", "SPRUCE_LEAVES", "SPRUCE_LOG", "SPRUCE_PLANKS", "SPRUCE_PRESSURE_PLATE", "SPRUCE_SAPLING", "SPRUCE_SHELF", "SPRUCE_SIGN", "SPRUCE_SLAB", "SPRUCE_STAIRS", "SPRUCE_TRAPDOOR", "SPRUCE_WALL_HANGING_SIGN", "SPRUCE_WALL_SIGN", "SPRUCE_WOOD", "STICKY_PISTON", "STONE", "STONE_BRICK_SLAB", "STONE_BRICK_STAIRS", "STONE_BRICK_WALL", "STONE_BRICKS", "STONE_BUTTON", "STONE_PRESSURE_PLATE", "STONE_SLAB", "STONE_STAIRS", "STONECUTTER", "STRAW_BED", "STRIPPED_ACACIA_LOG", "STRIPPED_ACACIA_WOOD", "STRIPPED_BAMBOO_BLOCK", "STRIPPED_BIRCH_LOG", "STRIPPED_BIRCH_WOOD", "STRIPPED_CHERRY_LOG", "STRIPPED_CHERRY_WOOD", "STRIPPED_CRIMSON_HYPHAE", "STRIPPED_CRIMSON_STEM", "STRIPPED_DARK_OAK_LOG", "STRIPPED_DARK_OAK_WOOD", "STRIPPED_JUNGLE_LOG", "STRIPPED_JUNGLE_WOOD", "STRIPPED_MANGROVE_LOG", "STRIPPED_MANGROVE_WOOD", "STRIPPED_OAK_LOG", "STRIPPED_OAK_WOOD", "STRIPPED_PALE_OAK_LOG", "STRIPPED_PALE_OAK_WOOD", "STRIPPED_POPLAR_LOG", "STRIPPED_POPLAR_WOOD", "STRIPPED_SPRUCE_LOG", "STRIPPED_SPRUCE_WOOD", "STRIPPED_WARPED_HYPHAE", "STRIPPED_WARPED_STEM", "STRUCTURE_BLOCK", "STRUCTURE_VOID", "SUGAR_CANE", "SULFUR", "SULFUR_BRICK_SLAB", "SULFUR_BRICK_STAIRS", "SULFUR_BRICK_WALL", "SULFUR_BRICKS", "SULFUR_SLAB", "SULFUR_SPIKE", "SULFUR_STAIRS", "SULFUR_WALL", "SUNFLOWER", "SUSPICIOUS_GRAVEL", "SUSPICIOUS_SAND", "SWEET_BERRY_BUSH", "TALL_DRY_GRASS", "TALL_GRASS", "TALL_SEAGRASS", "TARGET", "TERRACOTTA", "TEST_BLOCK", "TEST_INSTANCE_BLOCK", "TINTED_GLASS", "TNT", "TORCH", "TORCHFLOWER", "TORCHFLOWER_CROP", "TRAPPED_CHEST", "TRIAL_SPAWNER", "TRIPWIRE", "TRIPWIRE_HOOK", "TUBE_CORAL", "TUBE_CORAL_BLOCK", "TUBE_CORAL_FAN", "TUBE_CORAL_WALL_FAN", "TUFF", "TUFF_BRICK_SLAB", "TUFF_BRICK_STAIRS", "TUFF_BRICK_WALL", "TUFF_BRICKS", "TUFF_SLAB", "TUFF_STAIRS", "TUFF_WALL", "TURTLE_EGG", "TWISTING_VINES", "TWISTING_VINES_PLANT", "VAULT", "VERDANT_FROGLIGHT", "VINE", "VOID_AIR", "WALL_TORCH", "WARPED_BUTTON", "WARPED_DOOR", "WARPED_FENCE", "WARPED_FENCE_GATE", "WARPED_FUNGUS", "WARPED_HANGING_SIGN", "WARPED_HYPHAE", "WARPED_NYLIUM", "WARPED_PLANKS", "WARPED_PRESSURE_PLATE", "WARPED_ROOTS", "WARPED_SHELF", "WARPED_SIGN", "WARPED_SLAB", "WARPED_STAIRS", "WARPED_STEM", "WARPED_TRAPDOOR", "WARPED_WALL_HANGING_SIGN", "WARPED_WALL_SIGN", "WARPED_WART_BLOCK", "WATER", "WATER_CAULDRON", "WAXED_CHISELED_COPPER", "WAXED_COPPER_BARS", "WAXED_COPPER_BLOCK", "WAXED_COPPER_BULB", "WAXED_COPPER_CHAIN", "WAXED_COPPER_CHEST", "WAXED_COPPER_DOOR", "WAXED_COPPER_GOLEM_STATUE", "WAXED_COPPER_GRATE", "WAXED_COPPER_LANTERN", "WAXED_COPPER_TRAPDOOR", "WAXED_CUT_COPPER", "WAXED_CUT_COPPER_SLAB", "WAXED_CUT_COPPER_STAIRS", "WAXED_EXPOSED_CHISELED_COPPER", "WAXED_EXPOSED_COPPER", "WAXED_EXPOSED_COPPER_BARS", "WAXED_EXPOSED_COPPER_BULB", "WAXED_EXPOSED_COPPER_CHAIN", "WAXED_EXPOSED_COPPER_CHEST", "WAXED_EXPOSED_COPPER_DOOR", "WAXED_EXPOSED_COPPER_GOLEM_STATUE", "WAXED_EXPOSED_COPPER_GRATE", "WAXED_EXPOSED_COPPER_LANTERN", "WAXED_EXPOSED_COPPER_TRAPDOOR", "WAXED_EXPOSED_CUT_COPPER", "WAXED_EXPOSED_CUT_COPPER_SLAB", "WAXED_EXPOSED_CUT_COPPER_STAIRS", "WAXED_EXPOSED_LIGHTNING_ROD", "WAXED_LIGHTNING_ROD", "WAXED_OXIDIZED_CHISELED_COPPER", "WAXED_OXIDIZED_COPPER", "WAXED_OXIDIZED_COPPER_BARS", "WAXED_OXIDIZED_COPPER_BULB", "WAXED_OXIDIZED_COPPER_CHAIN", "WAXED_OXIDIZED_COPPER_CHEST", "WAXED_OXIDIZED_COPPER_DOOR", "WAXED_OXIDIZED_COPPER_GOLEM_STATUE", "WAXED_OXIDIZED_COPPER_GRATE", "WAXED_OXIDIZED_COPPER_LANTERN", "WAXED_OXIDIZED_COPPER_TRAPDOOR", "WAXED_OXIDIZED_CUT_COPPER", "WAXED_OXIDIZED_CUT_COPPER_SLAB", "WAXED_OXIDIZED_CUT_COPPER_STAIRS", "WAXED_OXIDIZED_LIGHTNING_ROD", "WAXED_WEATHERED_CHISELED_COPPER", "WAXED_WEATHERED_COPPER", "WAXED_WEATHERED_COPPER_BARS", "WAXED_WEATHERED_COPPER_BULB", "WAXED_WEATHERED_COPPER_CHAIN", "WAXED_WEATHERED_COPPER_CHEST", "WAXED_WEATHERED_COPPER_DOOR", "WAXED_WEATHERED_COPPER_GOLEM_STATUE", "WAXED_WEATHERED_COPPER_GRATE", "WAXED_WEATHERED_COPPER_LANTERN", "WAXED_WEATHERED_COPPER_TRAPDOOR", "WAXED_WEATHERED_CUT_COPPER", "WAXED_WEATHERED_CUT_COPPER_SLAB", "WAXED_WEATHERED_CUT_COPPER_STAIRS", "WAXED_WEATHERED_LIGHTNING_ROD", "WEATHERED_CHISELED_COPPER", "WEATHERED_COPPER", "WEATHERED_COPPER_BARS", "WEATHERED_COPPER_BULB", "WEATHERED_COPPER_CHAIN", "WEATHERED_COPPER_CHEST", "WEATHERED_COPPER_DOOR", "WEATHERED_COPPER_GOLEM_STATUE", "WEATHERED_COPPER_GRATE", "WEATHERED_COPPER_LANTERN", "WEATHERED_COPPER_TRAPDOOR", "WEATHERED_CUT_COPPER", "WEATHERED_CUT_COPPER_SLAB", "WEATHERED_CUT_COPPER_STAIRS", "WEATHERED_LIGHTNING_ROD", "WEEPING_VINES", "WEEPING_VINES_PLANT", "WET_SPONGE", "WHEAT", "WHITE_BANNER", "WHITE_BED", "WHITE_CANDLE", "WHITE_CANDLE_CAKE", "WHITE_CARPET", "WHITE_CONCRETE", "WHITE_CONCRETE_POWDER", "WHITE_CONCRETE_SLAB", "WHITE_CONCRETE_STAIRS", "WHITE_GLAZED_TERRACOTTA", "WHITE_SHULKER_BOX", "WHITE_STAINED_GLASS", "WHITE_STAINED_GLASS_PANE", "WHITE_TERRACOTTA", "WHITE_TULIP", "WHITE_WALL_BANNER", "WHITE_WOOL", "WHITE_WOOL_SLAB", "WHITE_WOOL_STAIRS", "WILDFLOWERS", "WITHER_ROSE", "WITHER_SKELETON_SKULL", "WITHER_SKELETON_WALL_SKULL", "YELLOW_BANNER", "YELLOW_BED", "YELLOW_CANDLE", "YELLOW_CANDLE_CAKE", "YELLOW_CARPET", "YELLOW_CONCRETE", "YELLOW_CONCRETE_POWDER", "YELLOW_CONCRETE_SLAB", "YELLOW_CONCRETE_STAIRS", "YELLOW_GLAZED_TERRACOTTA", "YELLOW_POPLAR_LEAVES", "YELLOW_SHULKER_BOX", "YELLOW_STAINED_GLASS", "YELLOW_STAINED_GLASS_PANE", "YELLOW_TERRACOTTA", "YELLOW_WALL_BANNER", "YELLOW_WOOL", "YELLOW_WOOL_SLAB", "YELLOW_WOOL_STAIRS", "ZOMBIE_HEAD", "ZOMBIE_WALL_HEAD", "LEGACY_AIR", "LEGACY_STONE", "LEGACY_GRASS", "LEGACY_DIRT", "LEGACY_COBBLESTONE", "LEGACY_WOOD", "LEGACY_SAPLING", "LEGACY_BEDROCK", "LEGACY_WATER", "LEGACY_STATIONARY_WATER", "LEGACY_LAVA", "LEGACY_STATIONARY_LAVA", "LEGACY_SAND", "LEGACY_GRAVEL", "LEGACY_GOLD_ORE", "LEGACY_IRON_ORE", "LEGACY_COAL_ORE", "LEGACY_LOG", "LEGACY_LEAVES", "LEGACY_SPONGE", "LEGACY_GLASS", "LEGACY_LAPIS_ORE", "LEGACY_LAPIS_BLOCK", "LEGACY_DISPENSER", "LEGACY_SANDSTONE", "LEGACY_NOTE_BLOCK", "LEGACY_BED_BLOCK", "LEGACY_POWERED_RAIL", "LEGACY_DETECTOR_RAIL", "LEGACY_PISTON_STICKY_BASE", "LEGACY_WEB", "LEGACY_LONG_GRASS", "LEGACY_DEAD_BUSH", "LEGACY_PISTON_BASE", "LEGACY_PISTON_EXTENSION", "LEGACY_WOOL", "LEGACY_PISTON_MOVING_PIECE", "LEGACY_YELLOW_FLOWER", "LEGACY_RED_ROSE", "LEGACY_BROWN_MUSHROOM", "LEGACY_RED_MUSHROOM", "LEGACY_GOLD_BLOCK", "LEGACY_IRON_BLOCK", "LEGACY_DOUBLE_STEP", "LEGACY_STEP", "LEGACY_BRICK", "LEGACY_TNT", "LEGACY_BOOKSHELF", "LEGACY_MOSSY_COBBLESTONE", "LEGACY_OBSIDIAN", "LEGACY_TORCH", "LEGACY_FIRE", "LEGACY_MOB_SPAWNER", "LEGACY_WOOD_STAIRS", "LEGACY_CHEST", "LEGACY_REDSTONE_WIRE", "LEGACY_DIAMOND_ORE", "LEGACY_DIAMOND_BLOCK", "LEGACY_WORKBENCH", "LEGACY_CROPS", "LEGACY_SOIL", "LEGACY_FURNACE", "LEGACY_BURNING_FURNACE", "LEGACY_SIGN_POST", "LEGACY_WOODEN_DOOR", "LEGACY_LADDER", "LEGACY_RAILS", "LEGACY_COBBLESTONE_STAIRS", "LEGACY_WALL_SIGN", "LEGACY_LEVER", "LEGACY_STONE_PLATE", "LEGACY_IRON_DOOR_BLOCK", "LEGACY_WOOD_PLATE", "LEGACY_REDSTONE_ORE", "LEGACY_GLOWING_REDSTONE_ORE", "LEGACY_REDSTONE_TORCH_OFF", "LEGACY_REDSTONE_TORCH_ON", "LEGACY_STONE_BUTTON", "LEGACY_SNOW", "LEGACY_ICE", "LEGACY_SNOW_BLOCK", "LEGACY_CACTUS", "LEGACY_CLAY", "LEGACY_SUGAR_CANE_BLOCK", "LEGACY_JUKEBOX", "LEGACY_FENCE", "LEGACY_PUMPKIN", "LEGACY_NETHERRACK", "LEGACY_SOUL_SAND", "LEGACY_GLOWSTONE", "LEGACY_PORTAL", "LEGACY_JACK_O_LANTERN", "LEGACY_CAKE_BLOCK", "LEGACY_DIODE_BLOCK_OFF", "LEGACY_DIODE_BLOCK_ON", "LEGACY_STAINED_GLASS", "LEGACY_TRAP_DOOR", "LEGACY_MONSTER_EGGS", "LEGACY_SMOOTH_BRICK", "LEGACY_HUGE_MUSHROOM_1", "LEGACY_HUGE_MUSHROOM_2", "LEGACY_IRON_FENCE", "LEGACY_THIN_GLASS", "LEGACY_MELON_BLOCK", "LEGACY_PUMPKIN_STEM", "LEGACY_MELON_STEM", "LEGACY_VINE", "LEGACY_FENCE_GATE", "LEGACY_BRICK_STAIRS", "LEGACY_SMOOTH_STAIRS", "LEGACY_MYCEL", "LEGACY_WATER_LILY", "LEGACY_NETHER_BRICK", "LEGACY_NETHER_FENCE", "LEGACY_NETHER_BRICK_STAIRS", "LEGACY_NETHER_WARTS", "LEGACY_ENCHANTMENT_TABLE", "LEGACY_BREWING_STAND", "LEGACY_CAULDRON", "LEGACY_ENDER_PORTAL", "LEGACY_ENDER_PORTAL_FRAME", "LEGACY_ENDER_STONE", "LEGACY_DRAGON_EGG", "LEGACY_REDSTONE_LAMP_OFF", "LEGACY_REDSTONE_LAMP_ON", "LEGACY_WOOD_DOUBLE_STEP", "LEGACY_WOOD_STEP", "LEGACY_COCOA", "LEGACY_SANDSTONE_STAIRS", "LEGACY_EMERALD_ORE", "LEGACY_ENDER_CHEST", "LEGACY_TRIPWIRE_HOOK", "LEGACY_TRIPWIRE", "LEGACY_EMERALD_BLOCK", "LEGACY_SPRUCE_WOOD_STAIRS", "LEGACY_BIRCH_WOOD_STAIRS", "LEGACY_JUNGLE_WOOD_STAIRS", "LEGACY_COMMAND", "LEGACY_BEACON", "LEGACY_COBBLE_WALL", "LEGACY_FLOWER_POT", "LEGACY_CARROT", "LEGACY_POTATO", "LEGACY_WOOD_BUTTON", "LEGACY_SKULL", "LEGACY_ANVIL", "LEGACY_TRAPPED_CHEST", "LEGACY_GOLD_PLATE", "LEGACY_IRON_PLATE", "LEGACY_REDSTONE_COMPARATOR_OFF", "LEGACY_REDSTONE_COMPARATOR_ON", "LEGACY_DAYLIGHT_DETECTOR", "LEGACY_REDSTONE_BLOCK", "LEGACY_QUARTZ_ORE", "LEGACY_HOPPER", "LEGACY_QUARTZ_BLOCK", "LEGACY_QUARTZ_STAIRS", "LEGACY_ACTIVATOR_RAIL", "LEGACY_DROPPER", "LEGACY_STAINED_CLAY", "LEGACY_STAINED_GLASS_PANE", "LEGACY_LEAVES_2", "LEGACY_LOG_2", "LEGACY_ACACIA_STAIRS", "LEGACY_DARK_OAK_STAIRS", "LEGACY_SLIME_BLOCK", "LEGACY_BARRIER", "LEGACY_IRON_TRAPDOOR", "LEGACY_PRISMARINE", "LEGACY_SEA_LANTERN", "LEGACY_HAY_BLOCK", "LEGACY_CARPET", "LEGACY_HARD_CLAY", "LEGACY_COAL_BLOCK", "LEGACY_PACKED_ICE", "LEGACY_DOUBLE_PLANT", "LEGACY_STANDING_BANNER", "LEGACY_WALL_BANNER", "LEGACY_DAYLIGHT_DETECTOR_INVERTED", "LEGACY_RED_SANDSTONE", "LEGACY_RED_SANDSTONE_STAIRS", "LEGACY_DOUBLE_STONE_SLAB2", "LEGACY_STONE_SLAB2", "LEGACY_SPRUCE_FENCE_GATE", "LEGACY_BIRCH_FENCE_GATE", "LEGACY_JUNGLE_FENCE_GATE", "LEGACY_DARK_OAK_FENCE_GATE", "LEGACY_ACACIA_FENCE_GATE", "LEGACY_SPRUCE_FENCE", "LEGACY_BIRCH_FENCE", "LEGACY_JUNGLE_FENCE", "LEGACY_DARK_OAK_FENCE", "LEGACY_ACACIA_FENCE", "LEGACY_SPRUCE_DOOR", "LEGACY_BIRCH_DOOR", "LEGACY_JUNGLE_DOOR", "LEGACY_ACACIA_DOOR", "LEGACY_DARK_OAK_DOOR", "LEGACY_END_ROD", "LEGACY_CHORUS_PLANT", "LEGACY_CHORUS_FLOWER", "LEGACY_PURPUR_BLOCK", "LEGACY_PURPUR_PILLAR", "LEGACY_PURPUR_STAIRS", "LEGACY_PURPUR_DOUBLE_SLAB", "LEGACY_PURPUR_SLAB", "LEGACY_END_BRICKS", "LEGACY_BEETROOT_BLOCK", "LEGACY_GRASS_PATH", "LEGACY_END_GATEWAY", "LEGACY_COMMAND_REPEATING", "LEGACY_COMMAND_CHAIN", "LEGACY_FROSTED_ICE", "LEGACY_MAGMA", "LEGACY_NETHER_WART_BLOCK", "LEGACY_RED_NETHER_BRICK", "LEGACY_BONE_BLOCK", "LEGACY_STRUCTURE_VOID", "LEGACY_OBSERVER", "LEGACY_WHITE_SHULKER_BOX", "LEGACY_ORANGE_SHULKER_BOX", "LEGACY_MAGENTA_SHULKER_BOX", "LEGACY_LIGHT_BLUE_SHULKER_BOX", "LEGACY_YELLOW_SHULKER_BOX", "LEGACY_LIME_SHULKER_BOX", "LEGACY_PINK_SHULKER_BOX", "LEGACY_GRAY_SHULKER_BOX", "LEGACY_SILVER_SHULKER_BOX", "LEGACY_CYAN_SHULKER_BOX", "LEGACY_PURPLE_SHULKER_BOX", "LEGACY_BLUE_SHULKER_BOX", "LEGACY_BROWN_SHULKER_BOX", "LEGACY_GREEN_SHULKER_BOX", "LEGACY_RED_SHULKER_BOX", "LEGACY_BLACK_SHULKER_BOX", "LEGACY_WHITE_GLAZED_TERRACOTTA", "LEGACY_ORANGE_GLAZED_TERRACOTTA", "LEGACY_MAGENTA_GLAZED_TERRACOTTA", "LEGACY_LIGHT_BLUE_GLAZED_TERRACOTTA", "LEGACY_YELLOW_GLAZED_TERRACOTTA", "LEGACY_LIME_GLAZED_TERRACOTTA", "LEGACY_PINK_GLAZED_TERRACOTTA", "LEGACY_GRAY_GLAZED_TERRACOTTA", "LEGACY_SILVER_GLAZED_TERRACOTTA", "LEGACY_CYAN_GLAZED_TERRACOTTA", "LEGACY_PURPLE_GLAZED_TERRACOTTA", "LEGACY_BLUE_GLAZED_TERRACOTTA", "LEGACY_BROWN_GLAZED_TERRACOTTA", "LEGACY_GREEN_GLAZED_TERRACOTTA", "LEGACY_RED_GLAZED_TERRACOTTA", "LEGACY_BLACK_GLAZED_TERRACOTTA", "LEGACY_CONCRETE", "LEGACY_CONCRETE_POWDER", "LEGACY_STRUCTURE_BLOCK", "LEGACY_IRON_SPADE", "LEGACY_IRON_PICKAXE", "LEGACY_IRON_AXE", "LEGACY_FLINT_AND_STEEL", "LEGACY_APPLE", "LEGACY_BOW", "LEGACY_ARROW", "LEGACY_COAL", "LEGACY_DIAMOND", "LEGACY_IRON_INGOT", "LEGACY_GOLD_INGOT", "LEGACY_IRON_SWORD", "LEGACY_WOOD_SWORD", "LEGACY_WOOD_SPADE", "LEGACY_WOOD_PICKAXE", "LEGACY_WOOD_AXE", "LEGACY_STONE_SWORD", "LEGACY_STONE_SPADE", "LEGACY_STONE_PICKAXE", "LEGACY_STONE_AXE", "LEGACY_DIAMOND_SWORD", "LEGACY_DIAMOND_SPADE", "LEGACY_DIAMOND_PICKAXE", "LEGACY_DIAMOND_AXE", "LEGACY_STICK", "LEGACY_BOWL", "LEGACY_MUSHROOM_SOUP", "LEGACY_GOLD_SWORD", "LEGACY_GOLD_SPADE", "LEGACY_GOLD_PICKAXE", "LEGACY_GOLD_AXE", "LEGACY_STRING", "LEGACY_FEATHER", "LEGACY_SULPHUR", "LEGACY_WOOD_HOE", "LEGACY_STONE_HOE", "LEGACY_IRON_HOE", "LEGACY_DIAMOND_HOE", "LEGACY_GOLD_HOE", "LEGACY_SEEDS", "LEGACY_WHEAT", "LEGACY_BREAD", "LEGACY_LEATHER_HELMET", "LEGACY_LEATHER_CHESTPLATE", "LEGACY_LEATHER_LEGGINGS", "LEGACY_LEATHER_BOOTS", "LEGACY_CHAINMAIL_HELMET", "LEGACY_CHAINMAIL_CHESTPLATE", "LEGACY_CHAINMAIL_LEGGINGS", "LEGACY_CHAINMAIL_BOOTS", "LEGACY_IRON_HELMET", "LEGACY_IRON_CHESTPLATE", "LEGACY_IRON_LEGGINGS", "LEGACY_IRON_BOOTS", "LEGACY_DIAMOND_HELMET", "LEGACY_DIAMOND_CHESTPLATE", "LEGACY_DIAMOND_LEGGINGS", "LEGACY_DIAMOND_BOOTS", "LEGACY_GOLD_HELMET", "LEGACY_GOLD_CHESTPLATE", "LEGACY_GOLD_LEGGINGS", "LEGACY_GOLD_BOOTS", "LEGACY_FLINT", "LEGACY_PORK", "LEGACY_GRILLED_PORK", "LEGACY_PAINTING", "LEGACY_GOLDEN_APPLE", "LEGACY_SIGN", "LEGACY_WOOD_DOOR", "LEGACY_BUCKET", "LEGACY_WATER_BUCKET", "LEGACY_LAVA_BUCKET", "LEGACY_MINECART", "LEGACY_SADDLE", "LEGACY_IRON_DOOR", "LEGACY_REDSTONE", "LEGACY_SNOW_BALL", "LEGACY_BOAT", "LEGACY_LEATHER", "LEGACY_MILK_BUCKET", "LEGACY_CLAY_BRICK", "LEGACY_CLAY_BALL", "LEGACY_SUGAR_CANE", "LEGACY_PAPER", "LEGACY_BOOK", "LEGACY_SLIME_BALL", "LEGACY_STORAGE_MINECART", "LEGACY_POWERED_MINECART", "LEGACY_EGG", "LEGACY_COMPASS", "LEGACY_FISHING_ROD", "LEGACY_WATCH", "LEGACY_GLOWSTONE_DUST", "LEGACY_RAW_FISH", "LEGACY_COOKED_FISH", "LEGACY_INK_SACK", "LEGACY_BONE", "LEGACY_SUGAR", "LEGACY_CAKE", "LEGACY_BED", "LEGACY_DIODE", "LEGACY_COOKIE", "LEGACY_MAP", "LEGACY_SHEARS", "LEGACY_MELON", "LEGACY_PUMPKIN_SEEDS", "LEGACY_MELON_SEEDS", "LEGACY_RAW_BEEF", "LEGACY_COOKED_BEEF", "LEGACY_RAW_CHICKEN", "LEGACY_COOKED_CHICKEN", "LEGACY_ROTTEN_FLESH", "LEGACY_ENDER_PEARL", "LEGACY_BLAZE_ROD", "LEGACY_GHAST_TEAR", "LEGACY_GOLD_NUGGET", "LEGACY_NETHER_STALK", "LEGACY_POTION", "LEGACY_GLASS_BOTTLE", "LEGACY_SPIDER_EYE", "LEGACY_FERMENTED_SPIDER_EYE", "LEGACY_BLAZE_POWDER", "LEGACY_MAGMA_CREAM", "LEGACY_BREWING_STAND_ITEM", "LEGACY_CAULDRON_ITEM", "LEGACY_EYE_OF_ENDER", "LEGACY_SPECKLED_MELON", "LEGACY_MONSTER_EGG", "LEGACY_EXP_BOTTLE", "LEGACY_FIREBALL", "LEGACY_BOOK_AND_QUILL", "LEGACY_WRITTEN_BOOK", "LEGACY_EMERALD", "LEGACY_ITEM_FRAME", "LEGACY_FLOWER_POT_ITEM", "LEGACY_CARROT_ITEM", "LEGACY_POTATO_ITEM", "LEGACY_BAKED_POTATO", "LEGACY_POISONOUS_POTATO", "LEGACY_EMPTY_MAP", "LEGACY_GOLDEN_CARROT", "LEGACY_SKULL_ITEM", "LEGACY_CARROT_STICK", "LEGACY_NETHER_STAR", "LEGACY_PUMPKIN_PIE", "LEGACY_FIREWORK", "LEGACY_FIREWORK_CHARGE", "LEGACY_ENCHANTED_BOOK", "LEGACY_REDSTONE_COMPARATOR", "LEGACY_NETHER_BRICK_ITEM", "LEGACY_QUARTZ", "LEGACY_EXPLOSIVE_MINECART", "LEGACY_HOPPER_MINECART", "LEGACY_PRISMARINE_SHARD", "LEGACY_PRISMARINE_CRYSTALS", "LEGACY_RABBIT", "LEGACY_COOKED_RABBIT", "LEGACY_RABBIT_STEW", "LEGACY_RABBIT_FOOT", "LEGACY_RABBIT_HIDE", "LEGACY_ARMOR_STAND", "LEGACY_IRON_BARDING", "LEGACY_GOLD_BARDING", "LEGACY_DIAMOND_BARDING", "LEGACY_LEASH", "LEGACY_NAME_TAG", "LEGACY_COMMAND_MINECART", "LEGACY_MUTTON", "LEGACY_COOKED_MUTTON", "LEGACY_BANNER", "LEGACY_END_CRYSTAL", "LEGACY_SPRUCE_DOOR_ITEM", "LEGACY_BIRCH_DOOR_ITEM", "LEGACY_JUNGLE_DOOR_ITEM", "LEGACY_ACACIA_DOOR_ITEM", "LEGACY_DARK_OAK_DOOR_ITEM", "LEGACY_CHORUS_FRUIT", "LEGACY_CHORUS_FRUIT_POPPED", "LEGACY_BEETROOT", "LEGACY_BEETROOT_SEEDS", "LEGACY_BEETROOT_SOUP", "LEGACY_DRAGONS_BREATH", "LEGACY_SPLASH_POTION", "LEGACY_SPECTRAL_ARROW", "LEGACY_TIPPED_ARROW", "LEGACY_LINGERING_POTION", "LEGACY_SHIELD", "LEGACY_ELYTRA", "LEGACY_BOAT_SPRUCE", "LEGACY_BOAT_BIRCH", "LEGACY_BOAT_JUNGLE", "LEGACY_BOAT_ACACIA", "LEGACY_BOAT_DARK_OAK", "LEGACY_TOTEM", "LEGACY_SHULKER_SHELL", "LEGACY_IRON_NUGGET", "LEGACY_KNOWLEDGE_BOOK", "LEGACY_GOLD_RECORD", "LEGACY_GREEN_RECORD", "LEGACY_RECORD_3", "LEGACY_RECORD_4", "LEGACY_RECORD_5", "LEGACY_RECORD_6", "LEGACY_RECORD_7", "LEGACY_RECORD_8", "LEGACY_RECORD_9", "LEGACY_RECORD_10", "LEGACY_RECORD_11", "LEGACY_RECORD_12"]
    }
    /// bukkit: org/bukkit/Material.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ABANDONED_CAMP_MAP: &str = "ABANDONED_CAMP_MAP";
    pub const ACACIA_BOAT: &str = "ACACIA_BOAT";
    pub const ACACIA_CHEST_BOAT: &str = "ACACIA_CHEST_BOAT";
    pub const AIR: &str = "AIR";
    pub const ALLAY_SPAWN_EGG: &str = "ALLAY_SPAWN_EGG";
    pub const AMETHYST_SHARD: &str = "AMETHYST_SHARD";
    pub const ANGLER_POTTERY_SHERD: &str = "ANGLER_POTTERY_SHERD";
    pub const APPLE: &str = "APPLE";
    pub const ARCHER_POTTERY_SHERD: &str = "ARCHER_POTTERY_SHERD";
    pub const ARMADILLO_SCUTE: &str = "ARMADILLO_SCUTE";
    pub const ARMADILLO_SPAWN_EGG: &str = "ARMADILLO_SPAWN_EGG";
    pub const ARMOR_STAND: &str = "ARMOR_STAND";
    pub const ARMS_UP_POTTERY_SHERD: &str = "ARMS_UP_POTTERY_SHERD";
    pub const ARROW: &str = "ARROW";
    pub const AXOLOTL_BUCKET: &str = "AXOLOTL_BUCKET";
    pub const AXOLOTL_SPAWN_EGG: &str = "AXOLOTL_SPAWN_EGG";
    pub const BAKED_POTATO: &str = "BAKED_POTATO";
    pub const BAMBOO_CHEST_RAFT: &str = "BAMBOO_CHEST_RAFT";
    pub const BAMBOO_RAFT: &str = "BAMBOO_RAFT";
    pub const BAT_SPAWN_EGG: &str = "BAT_SPAWN_EGG";
    pub const BEE_SPAWN_EGG: &str = "BEE_SPAWN_EGG";
    pub const BEEF: &str = "BEEF";
    pub const BEETROOT: &str = "BEETROOT";
    pub const BEETROOT_SEEDS: &str = "BEETROOT_SEEDS";
    pub const BEETROOT_SOUP: &str = "BEETROOT_SOUP";
    pub const BIRCH_BOAT: &str = "BIRCH_BOAT";
    pub const BIRCH_CHEST_BOAT: &str = "BIRCH_CHEST_BOAT";
    pub const BLACK_BUNDLE: &str = "BLACK_BUNDLE";
    pub const BLACK_CUSHION: &str = "BLACK_CUSHION";
    pub const BLACK_DYE: &str = "BLACK_DYE";
    pub const BLACK_HARNESS: &str = "BLACK_HARNESS";
    pub const BLADE_POTTERY_SHERD: &str = "BLADE_POTTERY_SHERD";
    pub const BLAZE_POWDER: &str = "BLAZE_POWDER";
    pub const BLAZE_ROD: &str = "BLAZE_ROD";
    pub const BLAZE_SPAWN_EGG: &str = "BLAZE_SPAWN_EGG";
    pub const BLUE_BUNDLE: &str = "BLUE_BUNDLE";
    pub const BLUE_CUSHION: &str = "BLUE_CUSHION";
    pub const BLUE_DYE: &str = "BLUE_DYE";
    pub const BLUE_EGG: &str = "BLUE_EGG";
    pub const BLUE_HARNESS: &str = "BLUE_HARNESS";
    pub const BOGGED_SPAWN_EGG: &str = "BOGGED_SPAWN_EGG";
    pub const BOLT_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "BOLT_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const BONE: &str = "BONE";
    pub const BONE_MEAL: &str = "BONE_MEAL";
    pub const BOOK: &str = "BOOK";
    pub const BORDURE_INDENTED_BANNER_PATTERN: &str = "BORDURE_INDENTED_BANNER_PATTERN";
    pub const BOW: &str = "BOW";
    pub const BOWL: &str = "BOWL";
    pub const BREAD: &str = "BREAD";
    pub const BREEZE_ROD: &str = "BREEZE_ROD";
    pub const BREEZE_SPAWN_EGG: &str = "BREEZE_SPAWN_EGG";
    pub const BREWER_POTTERY_SHERD: &str = "BREWER_POTTERY_SHERD";
    pub const BRICK: &str = "BRICK";
    pub const BROWN_BUNDLE: &str = "BROWN_BUNDLE";
    pub const BROWN_CUSHION: &str = "BROWN_CUSHION";
    pub const BROWN_DYE: &str = "BROWN_DYE";
    pub const BROWN_EGG: &str = "BROWN_EGG";
    pub const BROWN_HARNESS: &str = "BROWN_HARNESS";
    pub const BRUSH: &str = "BRUSH";
    pub const BUCKET: &str = "BUCKET";
    pub const BUNDLE: &str = "BUNDLE";
    pub const BURIED_ANCIENT_CITY_MAP: &str = "BURIED_ANCIENT_CITY_MAP";
    pub const BURIED_MINESHAFT_MAP: &str = "BURIED_MINESHAFT_MAP";
    pub const BURIED_TREASURE_MAP: &str = "BURIED_TREASURE_MAP";
    pub const BURIED_TRIAL_CHAMBERS_MAP: &str = "BURIED_TRIAL_CHAMBERS_MAP";
    pub const BURN_POTTERY_SHERD: &str = "BURN_POTTERY_SHERD";
    pub const CAMEL_HUSK_SPAWN_EGG: &str = "CAMEL_HUSK_SPAWN_EGG";
    pub const CAMEL_SPAWN_EGG: &str = "CAMEL_SPAWN_EGG";
    pub const CARROT: &str = "CARROT";
    pub const CARROT_ON_A_STICK: &str = "CARROT_ON_A_STICK";
    pub const CAT_SPAWN_EGG: &str = "CAT_SPAWN_EGG";
    pub const CAVE_SPIDER_SPAWN_EGG: &str = "CAVE_SPIDER_SPAWN_EGG";
    pub const CHAINMAIL_BOOTS: &str = "CHAINMAIL_BOOTS";
    pub const CHAINMAIL_CHESTPLATE: &str = "CHAINMAIL_CHESTPLATE";
    pub const CHAINMAIL_HELMET: &str = "CHAINMAIL_HELMET";
    pub const CHAINMAIL_LEGGINGS: &str = "CHAINMAIL_LEGGINGS";
    pub const CHARCOAL: &str = "CHARCOAL";
    pub const CHERRY_BOAT: &str = "CHERRY_BOAT";
    pub const CHERRY_CHEST_BOAT: &str = "CHERRY_CHEST_BOAT";
    pub const CHEST_MINECART: &str = "CHEST_MINECART";
    pub const CHICKEN: &str = "CHICKEN";
    pub const CHICKEN_SPAWN_EGG: &str = "CHICKEN_SPAWN_EGG";
    pub const CHORUS_FRUIT: &str = "CHORUS_FRUIT";
    pub const CLAY_BALL: &str = "CLAY_BALL";
    pub const CLOCK: &str = "CLOCK";
    pub const COAL: &str = "COAL";
    pub const COAST_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "COAST_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const COCOA_BEANS: &str = "COCOA_BEANS";
    pub const COD: &str = "COD";
    pub const COD_BUCKET: &str = "COD_BUCKET";
    pub const COD_SPAWN_EGG: &str = "COD_SPAWN_EGG";
    pub const COMMAND_BLOCK_MINECART: &str = "COMMAND_BLOCK_MINECART";
    pub const COMPASS: &str = "COMPASS";
    pub const COOKED_BEEF: &str = "COOKED_BEEF";
    pub const COOKED_CHICKEN: &str = "COOKED_CHICKEN";
    pub const COOKED_COD: &str = "COOKED_COD";
    pub const COOKED_MUTTON: &str = "COOKED_MUTTON";
    pub const COOKED_PORKCHOP: &str = "COOKED_PORKCHOP";
    pub const COOKED_RABBIT: &str = "COOKED_RABBIT";
    pub const COOKED_SALMON: &str = "COOKED_SALMON";
    pub const COOKIE: &str = "COOKIE";
    pub const COPPER_AXE: &str = "COPPER_AXE";
    pub const COPPER_BOOTS: &str = "COPPER_BOOTS";
    pub const COPPER_CHESTPLATE: &str = "COPPER_CHESTPLATE";
    pub const COPPER_GOLEM_SPAWN_EGG: &str = "COPPER_GOLEM_SPAWN_EGG";
    pub const COPPER_HELMET: &str = "COPPER_HELMET";
    pub const COPPER_HOE: &str = "COPPER_HOE";
    pub const COPPER_HORSE_ARMOR: &str = "COPPER_HORSE_ARMOR";
    pub const COPPER_INGOT: &str = "COPPER_INGOT";
    pub const COPPER_LEGGINGS: &str = "COPPER_LEGGINGS";
    pub const COPPER_NAUTILUS_ARMOR: &str = "COPPER_NAUTILUS_ARMOR";
    pub const COPPER_NUGGET: &str = "COPPER_NUGGET";
    pub const COPPER_PICKAXE: &str = "COPPER_PICKAXE";
    pub const COPPER_SHOVEL: &str = "COPPER_SHOVEL";
    pub const COPPER_SPEAR: &str = "COPPER_SPEAR";
    pub const COPPER_SWORD: &str = "COPPER_SWORD";
    pub const COW_SPAWN_EGG: &str = "COW_SPAWN_EGG";
    pub const CREAKING_SPAWN_EGG: &str = "CREAKING_SPAWN_EGG";
    pub const CREEPER_BANNER_PATTERN: &str = "CREEPER_BANNER_PATTERN";
    pub const CREEPER_SPAWN_EGG: &str = "CREEPER_SPAWN_EGG";
    pub const CROSSBOW: &str = "CROSSBOW";
    pub const CYAN_BUNDLE: &str = "CYAN_BUNDLE";
    pub const CYAN_CUSHION: &str = "CYAN_CUSHION";
    pub const CYAN_DYE: &str = "CYAN_DYE";
    pub const CYAN_HARNESS: &str = "CYAN_HARNESS";
    pub const DANGER_POTTERY_SHERD: &str = "DANGER_POTTERY_SHERD";
    pub const DARK_OAK_BOAT: &str = "DARK_OAK_BOAT";
    pub const DARK_OAK_CHEST_BOAT: &str = "DARK_OAK_CHEST_BOAT";
    pub const DEBUG_STICK: &str = "DEBUG_STICK";
    pub const DESERT_PYRAMID_MAP: &str = "DESERT_PYRAMID_MAP";
    pub const DESERT_VILLAGE_MAP: &str = "DESERT_VILLAGE_MAP";
    pub const DIAMOND: &str = "DIAMOND";
    pub const DIAMOND_AXE: &str = "DIAMOND_AXE";
    pub const DIAMOND_BOOTS: &str = "DIAMOND_BOOTS";
    pub const DIAMOND_CHESTPLATE: &str = "DIAMOND_CHESTPLATE";
    pub const DIAMOND_HELMET: &str = "DIAMOND_HELMET";
    pub const DIAMOND_HOE: &str = "DIAMOND_HOE";
    pub const DIAMOND_HORSE_ARMOR: &str = "DIAMOND_HORSE_ARMOR";
    pub const DIAMOND_LEGGINGS: &str = "DIAMOND_LEGGINGS";
    pub const DIAMOND_NAUTILUS_ARMOR: &str = "DIAMOND_NAUTILUS_ARMOR";
    pub const DIAMOND_PICKAXE: &str = "DIAMOND_PICKAXE";
    pub const DIAMOND_SHOVEL: &str = "DIAMOND_SHOVEL";
    pub const DIAMOND_SPEAR: &str = "DIAMOND_SPEAR";
    pub const DIAMOND_SWORD: &str = "DIAMOND_SWORD";
    pub const DISC_FRAGMENT_5: &str = "DISC_FRAGMENT_5";
    pub const DOLPHIN_SPAWN_EGG: &str = "DOLPHIN_SPAWN_EGG";
    pub const DONKEY_SPAWN_EGG: &str = "DONKEY_SPAWN_EGG";
    pub const DRAGON_BREATH: &str = "DRAGON_BREATH";
    pub const DRIED_KELP: &str = "DRIED_KELP";
    pub const DROWNED_SPAWN_EGG: &str = "DROWNED_SPAWN_EGG";
    pub const DUNE_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "DUNE_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const ECHO_SHARD: &str = "ECHO_SHARD";
    pub const EGG: &str = "EGG";
    pub const ELDER_GUARDIAN_SPAWN_EGG: &str = "ELDER_GUARDIAN_SPAWN_EGG";
    pub const ELYTRA: &str = "ELYTRA";
    pub const EMERALD: &str = "EMERALD";
    pub const ENCHANTED_BOOK: &str = "ENCHANTED_BOOK";
    pub const ENCHANTED_GOLDEN_APPLE: &str = "ENCHANTED_GOLDEN_APPLE";
    pub const END_CRYSTAL: &str = "END_CRYSTAL";
    pub const ENDER_DRAGON_SPAWN_EGG: &str = "ENDER_DRAGON_SPAWN_EGG";
    pub const ENDER_EYE: &str = "ENDER_EYE";
    pub const ENDER_PEARL: &str = "ENDER_PEARL";
    pub const ENDERMAN_SPAWN_EGG: &str = "ENDERMAN_SPAWN_EGG";
    pub const ENDERMITE_SPAWN_EGG: &str = "ENDERMITE_SPAWN_EGG";
    pub const EVOKER_SPAWN_EGG: &str = "EVOKER_SPAWN_EGG";
    pub const EXPERIENCE_BOTTLE: &str = "EXPERIENCE_BOTTLE";
    pub const EXPLORER_POTTERY_SHERD: &str = "EXPLORER_POTTERY_SHERD";
    pub const EYE_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "EYE_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const FEATHER: &str = "FEATHER";
    pub const FERMENTED_SPIDER_EYE: &str = "FERMENTED_SPIDER_EYE";
    pub const FIELD_MASONED_BANNER_PATTERN: &str = "FIELD_MASONED_BANNER_PATTERN";
    pub const FILLED_MAP: &str = "FILLED_MAP";
    pub const FIRE_CHARGE: &str = "FIRE_CHARGE";
    pub const FIREWORK_ROCKET: &str = "FIREWORK_ROCKET";
    pub const FIREWORK_STAR: &str = "FIREWORK_STAR";
    pub const FISHING_ROD: &str = "FISHING_ROD";
    pub const FLINT: &str = "FLINT";
    pub const FLINT_AND_STEEL: &str = "FLINT_AND_STEEL";
    pub const FLOW_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "FLOW_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const FLOW_BANNER_PATTERN: &str = "FLOW_BANNER_PATTERN";
    pub const FLOW_POTTERY_SHERD: &str = "FLOW_POTTERY_SHERD";
    pub const FLOWER_BANNER_PATTERN: &str = "FLOWER_BANNER_PATTERN";
    pub const FOX_SPAWN_EGG: &str = "FOX_SPAWN_EGG";
    pub const FRIEND_POTTERY_SHERD: &str = "FRIEND_POTTERY_SHERD";
    pub const FROG_SPAWN_EGG: &str = "FROG_SPAWN_EGG";
    pub const FURNACE_MINECART: &str = "FURNACE_MINECART";
    pub const GHAST_SPAWN_EGG: &str = "GHAST_SPAWN_EGG";
    pub const GHAST_TEAR: &str = "GHAST_TEAR";
    pub const GLASS_BOTTLE: &str = "GLASS_BOTTLE";
    pub const GLISTERING_MELON_SLICE: &str = "GLISTERING_MELON_SLICE";
    pub const GLOBE_BANNER_PATTERN: &str = "GLOBE_BANNER_PATTERN";
    pub const GLOW_BERRIES: &str = "GLOW_BERRIES";
    pub const GLOW_INK_SAC: &str = "GLOW_INK_SAC";
    pub const GLOW_ITEM_FRAME: &str = "GLOW_ITEM_FRAME";
    pub const GLOW_SQUID_SPAWN_EGG: &str = "GLOW_SQUID_SPAWN_EGG";
    pub const GLOWSTONE_DUST: &str = "GLOWSTONE_DUST";
    pub const GOAT_HORN: &str = "GOAT_HORN";
    pub const GOAT_SPAWN_EGG: &str = "GOAT_SPAWN_EGG";
    pub const GOLD_INGOT: &str = "GOLD_INGOT";
    pub const GOLD_NUGGET: &str = "GOLD_NUGGET";
    pub const GOLDEN_APPLE: &str = "GOLDEN_APPLE";
    pub const GOLDEN_AXE: &str = "GOLDEN_AXE";
    pub const GOLDEN_BOOTS: &str = "GOLDEN_BOOTS";
    pub const GOLDEN_CARROT: &str = "GOLDEN_CARROT";
    pub const GOLDEN_CHESTPLATE: &str = "GOLDEN_CHESTPLATE";
    pub const GOLDEN_HELMET: &str = "GOLDEN_HELMET";
    pub const GOLDEN_HOE: &str = "GOLDEN_HOE";
    pub const GOLDEN_HORSE_ARMOR: &str = "GOLDEN_HORSE_ARMOR";
    pub const GOLDEN_LEGGINGS: &str = "GOLDEN_LEGGINGS";
    pub const GOLDEN_NAUTILUS_ARMOR: &str = "GOLDEN_NAUTILUS_ARMOR";
    pub const GOLDEN_PICKAXE: &str = "GOLDEN_PICKAXE";
    pub const GOLDEN_SHOVEL: &str = "GOLDEN_SHOVEL";
    pub const GOLDEN_SPEAR: &str = "GOLDEN_SPEAR";
    pub const GOLDEN_SWORD: &str = "GOLDEN_SWORD";
    pub const GRAY_BUNDLE: &str = "GRAY_BUNDLE";
    pub const GRAY_CUSHION: &str = "GRAY_CUSHION";
    pub const GRAY_DYE: &str = "GRAY_DYE";
    pub const GRAY_HARNESS: &str = "GRAY_HARNESS";
    pub const GREEN_BUNDLE: &str = "GREEN_BUNDLE";
    pub const GREEN_CUSHION: &str = "GREEN_CUSHION";
    pub const GREEN_DYE: &str = "GREEN_DYE";
    pub const GREEN_HARNESS: &str = "GREEN_HARNESS";
    pub const GUARDIAN_SPAWN_EGG: &str = "GUARDIAN_SPAWN_EGG";
    pub const GUNPOWDER: &str = "GUNPOWDER";
    pub const GUSTER_BANNER_PATTERN: &str = "GUSTER_BANNER_PATTERN";
    pub const GUSTER_POTTERY_SHERD: &str = "GUSTER_POTTERY_SHERD";
    pub const HAPPY_GHAST_SPAWN_EGG: &str = "HAPPY_GHAST_SPAWN_EGG";
    pub const HEART_OF_THE_SEA: &str = "HEART_OF_THE_SEA";
    pub const HEART_POTTERY_SHERD: &str = "HEART_POTTERY_SHERD";
    pub const HEARTBREAK_POTTERY_SHERD: &str = "HEARTBREAK_POTTERY_SHERD";
    pub const HOGLIN_SPAWN_EGG: &str = "HOGLIN_SPAWN_EGG";
    pub const HONEY_BOTTLE: &str = "HONEY_BOTTLE";
    pub const HONEYCOMB: &str = "HONEYCOMB";
    pub const HOPPER_MINECART: &str = "HOPPER_MINECART";
    pub const HORSE_SPAWN_EGG: &str = "HORSE_SPAWN_EGG";
    pub const HOST_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "HOST_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const HOWL_POTTERY_SHERD: &str = "HOWL_POTTERY_SHERD";
    pub const HUSK_SPAWN_EGG: &str = "HUSK_SPAWN_EGG";
    pub const INK_SAC: &str = "INK_SAC";
    pub const IRON_AXE: &str = "IRON_AXE";
    pub const IRON_BOOTS: &str = "IRON_BOOTS";
    pub const IRON_CHESTPLATE: &str = "IRON_CHESTPLATE";
    pub const IRON_GOLEM_SPAWN_EGG: &str = "IRON_GOLEM_SPAWN_EGG";
    pub const IRON_HELMET: &str = "IRON_HELMET";
    pub const IRON_HOE: &str = "IRON_HOE";
    pub const IRON_HORSE_ARMOR: &str = "IRON_HORSE_ARMOR";
    pub const IRON_INGOT: &str = "IRON_INGOT";
    pub const IRON_LEGGINGS: &str = "IRON_LEGGINGS";
    pub const IRON_NAUTILUS_ARMOR: &str = "IRON_NAUTILUS_ARMOR";
    pub const IRON_NUGGET: &str = "IRON_NUGGET";
    pub const IRON_PICKAXE: &str = "IRON_PICKAXE";
    pub const IRON_SHOVEL: &str = "IRON_SHOVEL";
    pub const IRON_SPEAR: &str = "IRON_SPEAR";
    pub const IRON_SWORD: &str = "IRON_SWORD";
    pub const ITEM_FRAME: &str = "ITEM_FRAME";
    pub const JUNGLE_BOAT: &str = "JUNGLE_BOAT";
    pub const JUNGLE_CHEST_BOAT: &str = "JUNGLE_CHEST_BOAT";
    pub const JUNGLE_PYRAMID_MAP: &str = "JUNGLE_PYRAMID_MAP";
    pub const KNOWLEDGE_BOOK: &str = "KNOWLEDGE_BOOK";
    pub const LAPIS_LAZULI: &str = "LAPIS_LAZULI";
    pub const LAVA_BUCKET: &str = "LAVA_BUCKET";
    pub const LEAD: &str = "LEAD";
    pub const LEATHER: &str = "LEATHER";
    pub const LEATHER_BOOTS: &str = "LEATHER_BOOTS";
    pub const LEATHER_CHESTPLATE: &str = "LEATHER_CHESTPLATE";
    pub const LEATHER_HELMET: &str = "LEATHER_HELMET";
    pub const LEATHER_HORSE_ARMOR: &str = "LEATHER_HORSE_ARMOR";
    pub const LEATHER_LEGGINGS: &str = "LEATHER_LEGGINGS";
    pub const LIGHT_BLUE_BUNDLE: &str = "LIGHT_BLUE_BUNDLE";
    pub const LIGHT_BLUE_CUSHION: &str = "LIGHT_BLUE_CUSHION";
    pub const LIGHT_BLUE_DYE: &str = "LIGHT_BLUE_DYE";
    pub const LIGHT_BLUE_HARNESS: &str = "LIGHT_BLUE_HARNESS";
    pub const LIGHT_GRAY_BUNDLE: &str = "LIGHT_GRAY_BUNDLE";
    pub const LIGHT_GRAY_CUSHION: &str = "LIGHT_GRAY_CUSHION";
    pub const LIGHT_GRAY_DYE: &str = "LIGHT_GRAY_DYE";
    pub const LIGHT_GRAY_HARNESS: &str = "LIGHT_GRAY_HARNESS";
    pub const LIME_BUNDLE: &str = "LIME_BUNDLE";
    pub const LIME_CUSHION: &str = "LIME_CUSHION";
    pub const LIME_DYE: &str = "LIME_DYE";
    pub const LIME_HARNESS: &str = "LIME_HARNESS";
    pub const LINGERING_POTION: &str = "LINGERING_POTION";
    pub const LLAMA_SPAWN_EGG: &str = "LLAMA_SPAWN_EGG";
    pub const MACE: &str = "MACE";
    pub const MAGENTA_BUNDLE: &str = "MAGENTA_BUNDLE";
    pub const MAGENTA_CUSHION: &str = "MAGENTA_CUSHION";
    pub const MAGENTA_DYE: &str = "MAGENTA_DYE";
    pub const MAGENTA_HARNESS: &str = "MAGENTA_HARNESS";
    pub const MAGMA_CREAM: &str = "MAGMA_CREAM";
    pub const MAGMA_CUBE_SPAWN_EGG: &str = "MAGMA_CUBE_SPAWN_EGG";
    pub const MANGROVE_BOAT: &str = "MANGROVE_BOAT";
    pub const MANGROVE_CHEST_BOAT: &str = "MANGROVE_CHEST_BOAT";
    pub const MAP: &str = "MAP";
    pub const MELON_SEEDS: &str = "MELON_SEEDS";
    pub const MELON_SLICE: &str = "MELON_SLICE";
    pub const MILK_BUCKET: &str = "MILK_BUCKET";
    pub const MINECART: &str = "MINECART";
    pub const MINER_POTTERY_SHERD: &str = "MINER_POTTERY_SHERD";
    pub const MOJANG_BANNER_PATTERN: &str = "MOJANG_BANNER_PATTERN";
    pub const MOOSHROOM_SPAWN_EGG: &str = "MOOSHROOM_SPAWN_EGG";
    pub const MOURNER_POTTERY_SHERD: &str = "MOURNER_POTTERY_SHERD";
    pub const MULE_SPAWN_EGG: &str = "MULE_SPAWN_EGG";
    pub const MUSHROOM_STEW: &str = "MUSHROOM_STEW";
    pub const MUSIC_DISC_5: &str = "MUSIC_DISC_5";
    pub const MUSIC_DISC_11: &str = "MUSIC_DISC_11";
    pub const MUSIC_DISC_13: &str = "MUSIC_DISC_13";
    pub const MUSIC_DISC_BLOCKS: &str = "MUSIC_DISC_BLOCKS";
    pub const MUSIC_DISC_BOUNCE: &str = "MUSIC_DISC_BOUNCE";
    pub const MUSIC_DISC_CAT: &str = "MUSIC_DISC_CAT";
    pub const MUSIC_DISC_CHIRP: &str = "MUSIC_DISC_CHIRP";
    pub const MUSIC_DISC_CREATOR: &str = "MUSIC_DISC_CREATOR";
    pub const MUSIC_DISC_CREATOR_MUSIC_BOX: &str = "MUSIC_DISC_CREATOR_MUSIC_BOX";
    pub const MUSIC_DISC_FAR: &str = "MUSIC_DISC_FAR";
    pub const MUSIC_DISC_LAVA_CHICKEN: &str = "MUSIC_DISC_LAVA_CHICKEN";
    pub const MUSIC_DISC_MALL: &str = "MUSIC_DISC_MALL";
    pub const MUSIC_DISC_MELLOHI: &str = "MUSIC_DISC_MELLOHI";
    pub const MUSIC_DISC_OTHERSIDE: &str = "MUSIC_DISC_OTHERSIDE";
    pub const MUSIC_DISC_PIGSTEP: &str = "MUSIC_DISC_PIGSTEP";
    pub const MUSIC_DISC_PRECIPICE: &str = "MUSIC_DISC_PRECIPICE";
    pub const MUSIC_DISC_RELIC: &str = "MUSIC_DISC_RELIC";
    pub const MUSIC_DISC_STAL: &str = "MUSIC_DISC_STAL";
    pub const MUSIC_DISC_STRAD: &str = "MUSIC_DISC_STRAD";
    pub const MUSIC_DISC_TEARS: &str = "MUSIC_DISC_TEARS";
    pub const MUSIC_DISC_WAIT: &str = "MUSIC_DISC_WAIT";
    pub const MUSIC_DISC_WARD: &str = "MUSIC_DISC_WARD";
    pub const MUTTON: &str = "MUTTON";
    pub const NAME_TAG: &str = "NAME_TAG";
    pub const NAUTILUS_SHELL: &str = "NAUTILUS_SHELL";
    pub const NAUTILUS_SPAWN_EGG: &str = "NAUTILUS_SPAWN_EGG";
    pub const NETHER_BRICK: &str = "NETHER_BRICK";
    pub const NETHER_STAR: &str = "NETHER_STAR";
    pub const NETHERITE_AXE: &str = "NETHERITE_AXE";
    pub const NETHERITE_BOOTS: &str = "NETHERITE_BOOTS";
    pub const NETHERITE_CHESTPLATE: &str = "NETHERITE_CHESTPLATE";
    pub const NETHERITE_HELMET: &str = "NETHERITE_HELMET";
    pub const NETHERITE_HOE: &str = "NETHERITE_HOE";
    pub const NETHERITE_HORSE_ARMOR: &str = "NETHERITE_HORSE_ARMOR";
    pub const NETHERITE_INGOT: &str = "NETHERITE_INGOT";
    pub const NETHERITE_LEGGINGS: &str = "NETHERITE_LEGGINGS";
    pub const NETHERITE_NAUTILUS_ARMOR: &str = "NETHERITE_NAUTILUS_ARMOR";
    pub const NETHERITE_PICKAXE: &str = "NETHERITE_PICKAXE";
    pub const NETHERITE_SCRAP: &str = "NETHERITE_SCRAP";
    pub const NETHERITE_SHOVEL: &str = "NETHERITE_SHOVEL";
    pub const NETHERITE_SPEAR: &str = "NETHERITE_SPEAR";
    pub const NETHERITE_SWORD: &str = "NETHERITE_SWORD";
    pub const NETHERITE_UPGRADE_SMITHING_TEMPLATE: &str = "NETHERITE_UPGRADE_SMITHING_TEMPLATE";
    pub const OAK_BOAT: &str = "OAK_BOAT";
    pub const OAK_CHEST_BOAT: &str = "OAK_CHEST_BOAT";
    pub const OCEAN_MONUMENT_MAP: &str = "OCEAN_MONUMENT_MAP";
    pub const OCELOT_SPAWN_EGG: &str = "OCELOT_SPAWN_EGG";
    pub const OMINOUS_BOTTLE: &str = "OMINOUS_BOTTLE";
    pub const OMINOUS_TRIAL_KEY: &str = "OMINOUS_TRIAL_KEY";
    pub const ORANGE_BUNDLE: &str = "ORANGE_BUNDLE";
    pub const ORANGE_CUSHION: &str = "ORANGE_CUSHION";
    pub const ORANGE_DYE: &str = "ORANGE_DYE";
    pub const ORANGE_HARNESS: &str = "ORANGE_HARNESS";
    pub const PAINTING: &str = "PAINTING";
    pub const PALE_OAK_BOAT: &str = "PALE_OAK_BOAT";
    pub const PALE_OAK_CHEST_BOAT: &str = "PALE_OAK_CHEST_BOAT";
    pub const PANDA_SPAWN_EGG: &str = "PANDA_SPAWN_EGG";
    pub const PAPER: &str = "PAPER";
    pub const PARCHED_SPAWN_EGG: &str = "PARCHED_SPAWN_EGG";
    pub const PARROT_SPAWN_EGG: &str = "PARROT_SPAWN_EGG";
    pub const PHANTOM_MEMBRANE: &str = "PHANTOM_MEMBRANE";
    pub const PHANTOM_SPAWN_EGG: &str = "PHANTOM_SPAWN_EGG";
    pub const PIG_SPAWN_EGG: &str = "PIG_SPAWN_EGG";
    pub const PIGLIN_BANNER_PATTERN: &str = "PIGLIN_BANNER_PATTERN";
    pub const PIGLIN_BRUTE_SPAWN_EGG: &str = "PIGLIN_BRUTE_SPAWN_EGG";
    pub const PIGLIN_SPAWN_EGG: &str = "PIGLIN_SPAWN_EGG";
    pub const PILLAGER_SPAWN_EGG: &str = "PILLAGER_SPAWN_EGG";
    pub const PINK_BUNDLE: &str = "PINK_BUNDLE";
    pub const PINK_CUSHION: &str = "PINK_CUSHION";
    pub const PINK_DYE: &str = "PINK_DYE";
    pub const PINK_HARNESS: &str = "PINK_HARNESS";
    pub const PITCHER_POD: &str = "PITCHER_POD";
    pub const PLAINS_VILLAGE_MAP: &str = "PLAINS_VILLAGE_MAP";
    pub const PLENTY_POTTERY_SHERD: &str = "PLENTY_POTTERY_SHERD";
    pub const POISONOUS_POTATO: &str = "POISONOUS_POTATO";
    pub const POLAR_BEAR_SPAWN_EGG: &str = "POLAR_BEAR_SPAWN_EGG";
    pub const POPLAR_BOAT: &str = "POPLAR_BOAT";
    pub const POPLAR_CHEST_BOAT: &str = "POPLAR_CHEST_BOAT";
    pub const POPPED_CHORUS_FRUIT: &str = "POPPED_CHORUS_FRUIT";
    pub const PORKCHOP: &str = "PORKCHOP";
    pub const POTATO: &str = "POTATO";
    pub const POTION: &str = "POTION";
    pub const POWDER_SNOW_BUCKET: &str = "POWDER_SNOW_BUCKET";
    pub const PRISMARINE_CRYSTALS: &str = "PRISMARINE_CRYSTALS";
    pub const PRISMARINE_SHARD: &str = "PRISMARINE_SHARD";
    pub const PRIZE_POTTERY_SHERD: &str = "PRIZE_POTTERY_SHERD";
    pub const PUFFERFISH: &str = "PUFFERFISH";
    pub const PUFFERFISH_BUCKET: &str = "PUFFERFISH_BUCKET";
    pub const PUFFERFISH_SPAWN_EGG: &str = "PUFFERFISH_SPAWN_EGG";
    pub const PUMPKIN_PIE: &str = "PUMPKIN_PIE";
    pub const PUMPKIN_SEEDS: &str = "PUMPKIN_SEEDS";
    pub const PURPLE_BUNDLE: &str = "PURPLE_BUNDLE";
    pub const PURPLE_CUSHION: &str = "PURPLE_CUSHION";
    pub const PURPLE_DYE: &str = "PURPLE_DYE";
    pub const PURPLE_HARNESS: &str = "PURPLE_HARNESS";
    pub const QUARTZ: &str = "QUARTZ";
    pub const RABBIT: &str = "RABBIT";
    pub const RABBIT_FOOT: &str = "RABBIT_FOOT";
    pub const RABBIT_HIDE: &str = "RABBIT_HIDE";
    pub const RABBIT_SPAWN_EGG: &str = "RABBIT_SPAWN_EGG";
    pub const RABBIT_STEW: &str = "RABBIT_STEW";
    pub const RAISER_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "RAISER_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const RAVAGER_SPAWN_EGG: &str = "RAVAGER_SPAWN_EGG";
    pub const RAW_COPPER: &str = "RAW_COPPER";
    pub const RAW_GOLD: &str = "RAW_GOLD";
    pub const RAW_IRON: &str = "RAW_IRON";
    pub const RECOVERY_COMPASS: &str = "RECOVERY_COMPASS";
    pub const RED_BUNDLE: &str = "RED_BUNDLE";
    pub const RED_CUSHION: &str = "RED_CUSHION";
    pub const RED_DYE: &str = "RED_DYE";
    pub const RED_HARNESS: &str = "RED_HARNESS";
    pub const REDSTONE: &str = "REDSTONE";
    pub const RESIN_BRICK: &str = "RESIN_BRICK";
    pub const RIB_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "RIB_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const ROTTEN_FLESH: &str = "ROTTEN_FLESH";
    pub const SADDLE: &str = "SADDLE";
    pub const SALMON: &str = "SALMON";
    pub const SALMON_BUCKET: &str = "SALMON_BUCKET";
    pub const SALMON_SPAWN_EGG: &str = "SALMON_SPAWN_EGG";
    pub const SAVANNA_VILLAGE_MAP: &str = "SAVANNA_VILLAGE_MAP";
    pub const SCRAPE_POTTERY_SHERD: &str = "SCRAPE_POTTERY_SHERD";
    pub const SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const SHEAF_POTTERY_SHERD: &str = "SHEAF_POTTERY_SHERD";
    pub const SHEARS: &str = "SHEARS";
    pub const SHEEP_SPAWN_EGG: &str = "SHEEP_SPAWN_EGG";
    pub const SHELTER_POTTERY_SHERD: &str = "SHELTER_POTTERY_SHERD";
    pub const SHIELD: &str = "SHIELD";
    pub const SHULKER_SHELL: &str = "SHULKER_SHELL";
    pub const SHULKER_SPAWN_EGG: &str = "SHULKER_SPAWN_EGG";
    pub const SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const SILVERFISH_SPAWN_EGG: &str = "SILVERFISH_SPAWN_EGG";
    pub const SKELETON_HORSE_SPAWN_EGG: &str = "SKELETON_HORSE_SPAWN_EGG";
    pub const SKELETON_SPAWN_EGG: &str = "SKELETON_SPAWN_EGG";
    pub const SKULL_BANNER_PATTERN: &str = "SKULL_BANNER_PATTERN";
    pub const SKULL_POTTERY_SHERD: &str = "SKULL_POTTERY_SHERD";
    pub const SLIME_BALL: &str = "SLIME_BALL";
    pub const SLIME_SPAWN_EGG: &str = "SLIME_SPAWN_EGG";
    pub const SNIFFER_SPAWN_EGG: &str = "SNIFFER_SPAWN_EGG";
    pub const SNORT_POTTERY_SHERD: &str = "SNORT_POTTERY_SHERD";
    pub const SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const SNOW_GOLEM_SPAWN_EGG: &str = "SNOW_GOLEM_SPAWN_EGG";
    pub const SNOWBALL: &str = "SNOWBALL";
    pub const SNOWY_VILLAGE_MAP: &str = "SNOWY_VILLAGE_MAP";
    pub const SPECTRAL_ARROW: &str = "SPECTRAL_ARROW";
    pub const SPIDER_EYE: &str = "SPIDER_EYE";
    pub const SPIDER_SPAWN_EGG: &str = "SPIDER_SPAWN_EGG";
    pub const SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const SPLASH_POTION: &str = "SPLASH_POTION";
    pub const SPRUCE_BOAT: &str = "SPRUCE_BOAT";
    pub const SPRUCE_CHEST_BOAT: &str = "SPRUCE_CHEST_BOAT";
    pub const SPYGLASS: &str = "SPYGLASS";
    pub const SQUID_SPAWN_EGG: &str = "SQUID_SPAWN_EGG";
    pub const STICK: &str = "STICK";
    pub const STONE_AXE: &str = "STONE_AXE";
    pub const STONE_HOE: &str = "STONE_HOE";
    pub const STONE_PICKAXE: &str = "STONE_PICKAXE";
    pub const STONE_SHOVEL: &str = "STONE_SHOVEL";
    pub const STONE_SPEAR: &str = "STONE_SPEAR";
    pub const STONE_SWORD: &str = "STONE_SWORD";
    pub const STRAY_SPAWN_EGG: &str = "STRAY_SPAWN_EGG";
    pub const STRIDER_SPAWN_EGG: &str = "STRIDER_SPAWN_EGG";
    pub const STRING: &str = "STRING";
    pub const SUGAR: &str = "SUGAR";
    pub const SULFUR_CUBE_BUCKET: &str = "SULFUR_CUBE_BUCKET";
    pub const SULFUR_CUBE_SPAWN_EGG: &str = "SULFUR_CUBE_SPAWN_EGG";
    pub const SUSPICIOUS_STEW: &str = "SUSPICIOUS_STEW";
    pub const SWAMP_HUT_MAP: &str = "SWAMP_HUT_MAP";
    pub const SWEET_BERRIES: &str = "SWEET_BERRIES";
    pub const TADPOLE_BUCKET: &str = "TADPOLE_BUCKET";
    pub const TADPOLE_SPAWN_EGG: &str = "TADPOLE_SPAWN_EGG";
    pub const TAIGA_VILLAGE_MAP: &str = "TAIGA_VILLAGE_MAP";
    pub const TIDE_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "TIDE_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const TIPPED_ARROW: &str = "TIPPED_ARROW";
    pub const TNT_MINECART: &str = "TNT_MINECART";
    pub const TORCHFLOWER_SEEDS: &str = "TORCHFLOWER_SEEDS";
    pub const TOTEM_OF_UNDYING: &str = "TOTEM_OF_UNDYING";
    pub const TRADER_LLAMA_SPAWN_EGG: &str = "TRADER_LLAMA_SPAWN_EGG";
    pub const TRIAL_KEY: &str = "TRIAL_KEY";
    pub const TRIDENT: &str = "TRIDENT";
    pub const TROPICAL_FISH: &str = "TROPICAL_FISH";
    pub const TROPICAL_FISH_BUCKET: &str = "TROPICAL_FISH_BUCKET";
    pub const TROPICAL_FISH_SPAWN_EGG: &str = "TROPICAL_FISH_SPAWN_EGG";
    pub const TURTLE_HELMET: &str = "TURTLE_HELMET";
    pub const TURTLE_SCUTE: &str = "TURTLE_SCUTE";
    pub const TURTLE_SPAWN_EGG: &str = "TURTLE_SPAWN_EGG";
    pub const VEX_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "VEX_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const VEX_SPAWN_EGG: &str = "VEX_SPAWN_EGG";
    pub const VILLAGER_SPAWN_EGG: &str = "VILLAGER_SPAWN_EGG";
    pub const VINDICATOR_SPAWN_EGG: &str = "VINDICATOR_SPAWN_EGG";
    pub const WANDERING_TRADER_SPAWN_EGG: &str = "WANDERING_TRADER_SPAWN_EGG";
    pub const WARD_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "WARD_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const WARDEN_SPAWN_EGG: &str = "WARDEN_SPAWN_EGG";
    pub const WARM_OCEAN_RUINS_MAP: &str = "WARM_OCEAN_RUINS_MAP";
    pub const WARPED_FUNGUS_ON_A_STICK: &str = "WARPED_FUNGUS_ON_A_STICK";
    pub const WATER_BUCKET: &str = "WATER_BUCKET";
    pub const WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const WHEAT_SEEDS: &str = "WHEAT_SEEDS";
    pub const WHITE_BUNDLE: &str = "WHITE_BUNDLE";
    pub const WHITE_CUSHION: &str = "WHITE_CUSHION";
    pub const WHITE_DYE: &str = "WHITE_DYE";
    pub const WHITE_HARNESS: &str = "WHITE_HARNESS";
    pub const WILD_ARMOR_TRIM_SMITHING_TEMPLATE: &str = "WILD_ARMOR_TRIM_SMITHING_TEMPLATE";
    pub const WIND_CHARGE: &str = "WIND_CHARGE";
    pub const WITCH_SPAWN_EGG: &str = "WITCH_SPAWN_EGG";
    pub const WITHER_SKELETON_SPAWN_EGG: &str = "WITHER_SKELETON_SPAWN_EGG";
    pub const WITHER_SPAWN_EGG: &str = "WITHER_SPAWN_EGG";
    pub const WOLF_ARMOR: &str = "WOLF_ARMOR";
    pub const WOLF_SPAWN_EGG: &str = "WOLF_SPAWN_EGG";
    pub const WOODEN_AXE: &str = "WOODEN_AXE";
    pub const WOODEN_HOE: &str = "WOODEN_HOE";
    pub const WOODEN_PICKAXE: &str = "WOODEN_PICKAXE";
    pub const WOODEN_SHOVEL: &str = "WOODEN_SHOVEL";
    pub const WOODEN_SPEAR: &str = "WOODEN_SPEAR";
    pub const WOODEN_SWORD: &str = "WOODEN_SWORD";
    pub const WOODLAND_MANSION_MAP: &str = "WOODLAND_MANSION_MAP";
    pub const WRITABLE_BOOK: &str = "WRITABLE_BOOK";
    pub const WRITTEN_BOOK: &str = "WRITTEN_BOOK";
    pub const YELLOW_BUNDLE: &str = "YELLOW_BUNDLE";
    pub const YELLOW_CUSHION: &str = "YELLOW_CUSHION";
    pub const YELLOW_DYE: &str = "YELLOW_DYE";
    pub const YELLOW_HARNESS: &str = "YELLOW_HARNESS";
    pub const ZOGLIN_SPAWN_EGG: &str = "ZOGLIN_SPAWN_EGG";
    pub const ZOMBIE_HORSE_SPAWN_EGG: &str = "ZOMBIE_HORSE_SPAWN_EGG";
    pub const ZOMBIE_NAUTILUS_SPAWN_EGG: &str = "ZOMBIE_NAUTILUS_SPAWN_EGG";
    pub const ZOMBIE_SPAWN_EGG: &str = "ZOMBIE_SPAWN_EGG";
    pub const ZOMBIE_VILLAGER_SPAWN_EGG: &str = "ZOMBIE_VILLAGER_SPAWN_EGG";
    pub const ZOMBIFIED_PIGLIN_SPAWN_EGG: &str = "ZOMBIFIED_PIGLIN_SPAWN_EGG";
    pub const ACACIA_BUTTON: &str = "ACACIA_BUTTON";
    pub const ACACIA_DOOR: &str = "ACACIA_DOOR";
    pub const ACACIA_FENCE: &str = "ACACIA_FENCE";
    pub const ACACIA_FENCE_GATE: &str = "ACACIA_FENCE_GATE";
    pub const ACACIA_HANGING_SIGN: &str = "ACACIA_HANGING_SIGN";
    pub const ACACIA_LEAVES: &str = "ACACIA_LEAVES";
    pub const ACACIA_LOG: &str = "ACACIA_LOG";
    pub const ACACIA_PLANKS: &str = "ACACIA_PLANKS";
    pub const ACACIA_PRESSURE_PLATE: &str = "ACACIA_PRESSURE_PLATE";
    pub const ACACIA_SAPLING: &str = "ACACIA_SAPLING";
    pub const ACACIA_SHELF: &str = "ACACIA_SHELF";
    pub const ACACIA_SIGN: &str = "ACACIA_SIGN";
    pub const ACACIA_SLAB: &str = "ACACIA_SLAB";
    pub const ACACIA_STAIRS: &str = "ACACIA_STAIRS";
    pub const ACACIA_TRAPDOOR: &str = "ACACIA_TRAPDOOR";
    pub const ACACIA_WALL_HANGING_SIGN: &str = "ACACIA_WALL_HANGING_SIGN";
    pub const ACACIA_WALL_SIGN: &str = "ACACIA_WALL_SIGN";
    pub const ACACIA_WOOD: &str = "ACACIA_WOOD";
    pub const ACTIVATOR_RAIL: &str = "ACTIVATOR_RAIL";
    pub const ALLIUM: &str = "ALLIUM";
    pub const AMETHYST_BLOCK: &str = "AMETHYST_BLOCK";
    pub const AMETHYST_CLUSTER: &str = "AMETHYST_CLUSTER";
    pub const ANCIENT_DEBRIS: &str = "ANCIENT_DEBRIS";
    pub const ANDESITE: &str = "ANDESITE";
    pub const ANDESITE_SLAB: &str = "ANDESITE_SLAB";
    pub const ANDESITE_STAIRS: &str = "ANDESITE_STAIRS";
    pub const ANDESITE_WALL: &str = "ANDESITE_WALL";
    pub const ANVIL: &str = "ANVIL";
    pub const ATTACHED_MELON_STEM: &str = "ATTACHED_MELON_STEM";
    pub const ATTACHED_PUMPKIN_STEM: &str = "ATTACHED_PUMPKIN_STEM";
    pub const AZALEA: &str = "AZALEA";
    pub const AZALEA_LEAVES: &str = "AZALEA_LEAVES";
    pub const AZURE_BLUET: &str = "AZURE_BLUET";
    pub const BAMBOO: &str = "BAMBOO";
    pub const BAMBOO_BLOCK: &str = "BAMBOO_BLOCK";
    pub const BAMBOO_BUTTON: &str = "BAMBOO_BUTTON";
    pub const BAMBOO_DOOR: &str = "BAMBOO_DOOR";
    pub const BAMBOO_FENCE: &str = "BAMBOO_FENCE";
    pub const BAMBOO_FENCE_GATE: &str = "BAMBOO_FENCE_GATE";
    pub const BAMBOO_HANGING_SIGN: &str = "BAMBOO_HANGING_SIGN";
    pub const BAMBOO_MOSAIC: &str = "BAMBOO_MOSAIC";
    pub const BAMBOO_MOSAIC_SLAB: &str = "BAMBOO_MOSAIC_SLAB";
    pub const BAMBOO_MOSAIC_STAIRS: &str = "BAMBOO_MOSAIC_STAIRS";
    pub const BAMBOO_PLANKS: &str = "BAMBOO_PLANKS";
    pub const BAMBOO_PRESSURE_PLATE: &str = "BAMBOO_PRESSURE_PLATE";
    pub const BAMBOO_SAPLING: &str = "BAMBOO_SAPLING";
    pub const BAMBOO_SHELF: &str = "BAMBOO_SHELF";
    pub const BAMBOO_SIGN: &str = "BAMBOO_SIGN";
    pub const BAMBOO_SLAB: &str = "BAMBOO_SLAB";
    pub const BAMBOO_STAIRS: &str = "BAMBOO_STAIRS";
    pub const BAMBOO_TRAPDOOR: &str = "BAMBOO_TRAPDOOR";
    pub const BAMBOO_WALL_HANGING_SIGN: &str = "BAMBOO_WALL_HANGING_SIGN";
    pub const BAMBOO_WALL_SIGN: &str = "BAMBOO_WALL_SIGN";
    pub const BARREL: &str = "BARREL";
    pub const BARRIER: &str = "BARRIER";
    pub const BASALT: &str = "BASALT";
    pub const BEACON: &str = "BEACON";
    pub const BEDROCK: &str = "BEDROCK";
    pub const BEE_NEST: &str = "BEE_NEST";
    pub const BEEHIVE: &str = "BEEHIVE";
    pub const BEETROOTS: &str = "BEETROOTS";
    pub const BELL: &str = "BELL";
    pub const BIG_DRIPLEAF: &str = "BIG_DRIPLEAF";
    pub const BIG_DRIPLEAF_STEM: &str = "BIG_DRIPLEAF_STEM";
    pub const BIRCH_BUTTON: &str = "BIRCH_BUTTON";
    pub const BIRCH_DOOR: &str = "BIRCH_DOOR";
    pub const BIRCH_FENCE: &str = "BIRCH_FENCE";
    pub const BIRCH_FENCE_GATE: &str = "BIRCH_FENCE_GATE";
    pub const BIRCH_HANGING_SIGN: &str = "BIRCH_HANGING_SIGN";
    pub const BIRCH_LEAVES: &str = "BIRCH_LEAVES";
    pub const BIRCH_LOG: &str = "BIRCH_LOG";
    pub const BIRCH_PLANKS: &str = "BIRCH_PLANKS";
    pub const BIRCH_PRESSURE_PLATE: &str = "BIRCH_PRESSURE_PLATE";
    pub const BIRCH_SAPLING: &str = "BIRCH_SAPLING";
    pub const BIRCH_SHELF: &str = "BIRCH_SHELF";
    pub const BIRCH_SIGN: &str = "BIRCH_SIGN";
    pub const BIRCH_SLAB: &str = "BIRCH_SLAB";
    pub const BIRCH_STAIRS: &str = "BIRCH_STAIRS";
    pub const BIRCH_TRAPDOOR: &str = "BIRCH_TRAPDOOR";
    pub const BIRCH_WALL_HANGING_SIGN: &str = "BIRCH_WALL_HANGING_SIGN";
    pub const BIRCH_WALL_SIGN: &str = "BIRCH_WALL_SIGN";
    pub const BIRCH_WOOD: &str = "BIRCH_WOOD";
    pub const BLACK_BANNER: &str = "BLACK_BANNER";
    pub const BLACK_BED: &str = "BLACK_BED";
    pub const BLACK_CANDLE: &str = "BLACK_CANDLE";
    pub const BLACK_CANDLE_CAKE: &str = "BLACK_CANDLE_CAKE";
    pub const BLACK_CARPET: &str = "BLACK_CARPET";
    pub const BLACK_CONCRETE: &str = "BLACK_CONCRETE";
    pub const BLACK_CONCRETE_POWDER: &str = "BLACK_CONCRETE_POWDER";
    pub const BLACK_CONCRETE_SLAB: &str = "BLACK_CONCRETE_SLAB";
    pub const BLACK_CONCRETE_STAIRS: &str = "BLACK_CONCRETE_STAIRS";
    pub const BLACK_GLAZED_TERRACOTTA: &str = "BLACK_GLAZED_TERRACOTTA";
    pub const BLACK_SHULKER_BOX: &str = "BLACK_SHULKER_BOX";
    pub const BLACK_STAINED_GLASS: &str = "BLACK_STAINED_GLASS";
    pub const BLACK_STAINED_GLASS_PANE: &str = "BLACK_STAINED_GLASS_PANE";
    pub const BLACK_TERRACOTTA: &str = "BLACK_TERRACOTTA";
    pub const BLACK_WALL_BANNER: &str = "BLACK_WALL_BANNER";
    pub const BLACK_WOOL: &str = "BLACK_WOOL";
    pub const BLACK_WOOL_SLAB: &str = "BLACK_WOOL_SLAB";
    pub const BLACK_WOOL_STAIRS: &str = "BLACK_WOOL_STAIRS";
    pub const BLACKSTONE: &str = "BLACKSTONE";
    pub const BLACKSTONE_SLAB: &str = "BLACKSTONE_SLAB";
    pub const BLACKSTONE_STAIRS: &str = "BLACKSTONE_STAIRS";
    pub const BLACKSTONE_WALL: &str = "BLACKSTONE_WALL";
    pub const BLAST_FURNACE: &str = "BLAST_FURNACE";
    pub const BLUE_BANNER: &str = "BLUE_BANNER";
    pub const BLUE_BED: &str = "BLUE_BED";
    pub const BLUE_CANDLE: &str = "BLUE_CANDLE";
    pub const BLUE_CANDLE_CAKE: &str = "BLUE_CANDLE_CAKE";
    pub const BLUE_CARPET: &str = "BLUE_CARPET";
    pub const BLUE_CONCRETE: &str = "BLUE_CONCRETE";
    pub const BLUE_CONCRETE_POWDER: &str = "BLUE_CONCRETE_POWDER";
    pub const BLUE_CONCRETE_SLAB: &str = "BLUE_CONCRETE_SLAB";
    pub const BLUE_CONCRETE_STAIRS: &str = "BLUE_CONCRETE_STAIRS";
    pub const BLUE_GLAZED_TERRACOTTA: &str = "BLUE_GLAZED_TERRACOTTA";
    pub const BLUE_ICE: &str = "BLUE_ICE";
    pub const BLUE_ORCHID: &str = "BLUE_ORCHID";
    pub const BLUE_SHULKER_BOX: &str = "BLUE_SHULKER_BOX";
    pub const BLUE_STAINED_GLASS: &str = "BLUE_STAINED_GLASS";
    pub const BLUE_STAINED_GLASS_PANE: &str = "BLUE_STAINED_GLASS_PANE";
    pub const BLUE_TERRACOTTA: &str = "BLUE_TERRACOTTA";
    pub const BLUE_WALL_BANNER: &str = "BLUE_WALL_BANNER";
    pub const BLUE_WOOL: &str = "BLUE_WOOL";
    pub const BLUE_WOOL_SLAB: &str = "BLUE_WOOL_SLAB";
    pub const BLUE_WOOL_STAIRS: &str = "BLUE_WOOL_STAIRS";
    pub const BONE_BLOCK: &str = "BONE_BLOCK";
    pub const BOOKSHELF: &str = "BOOKSHELF";
    pub const BRAIN_CORAL: &str = "BRAIN_CORAL";
    pub const BRAIN_CORAL_BLOCK: &str = "BRAIN_CORAL_BLOCK";
    pub const BRAIN_CORAL_FAN: &str = "BRAIN_CORAL_FAN";
    pub const BRAIN_CORAL_WALL_FAN: &str = "BRAIN_CORAL_WALL_FAN";
    pub const BREWING_STAND: &str = "BREWING_STAND";
    pub const BRICK_SLAB: &str = "BRICK_SLAB";
    pub const BRICK_STAIRS: &str = "BRICK_STAIRS";
    pub const BRICK_WALL: &str = "BRICK_WALL";
    pub const BRICKS: &str = "BRICKS";
    pub const BROWN_BANNER: &str = "BROWN_BANNER";
    pub const BROWN_BED: &str = "BROWN_BED";
    pub const BROWN_CANDLE: &str = "BROWN_CANDLE";
    pub const BROWN_CANDLE_CAKE: &str = "BROWN_CANDLE_CAKE";
    pub const BROWN_CARPET: &str = "BROWN_CARPET";
    pub const BROWN_CONCRETE: &str = "BROWN_CONCRETE";
    pub const BROWN_CONCRETE_POWDER: &str = "BROWN_CONCRETE_POWDER";
    pub const BROWN_CONCRETE_SLAB: &str = "BROWN_CONCRETE_SLAB";
    pub const BROWN_CONCRETE_STAIRS: &str = "BROWN_CONCRETE_STAIRS";
    pub const BROWN_GLAZED_TERRACOTTA: &str = "BROWN_GLAZED_TERRACOTTA";
    pub const BROWN_MUSHROOM: &str = "BROWN_MUSHROOM";
    pub const BROWN_MUSHROOM_BLOCK: &str = "BROWN_MUSHROOM_BLOCK";
    pub const BROWN_SHULKER_BOX: &str = "BROWN_SHULKER_BOX";
    pub const BROWN_STAINED_GLASS: &str = "BROWN_STAINED_GLASS";
    pub const BROWN_STAINED_GLASS_PANE: &str = "BROWN_STAINED_GLASS_PANE";
    pub const BROWN_TERRACOTTA: &str = "BROWN_TERRACOTTA";
    pub const BROWN_WALL_BANNER: &str = "BROWN_WALL_BANNER";
    pub const BROWN_WOOL: &str = "BROWN_WOOL";
    pub const BROWN_WOOL_SLAB: &str = "BROWN_WOOL_SLAB";
    pub const BROWN_WOOL_STAIRS: &str = "BROWN_WOOL_STAIRS";
    pub const BUBBLE_COLUMN: &str = "BUBBLE_COLUMN";
    pub const BUBBLE_CORAL: &str = "BUBBLE_CORAL";
    pub const BUBBLE_CORAL_BLOCK: &str = "BUBBLE_CORAL_BLOCK";
    pub const BUBBLE_CORAL_FAN: &str = "BUBBLE_CORAL_FAN";
    pub const BUBBLE_CORAL_WALL_FAN: &str = "BUBBLE_CORAL_WALL_FAN";
    pub const BUDDING_AMETHYST: &str = "BUDDING_AMETHYST";
    pub const BUSH: &str = "BUSH";
    pub const CACTUS: &str = "CACTUS";
    pub const CACTUS_FLOWER: &str = "CACTUS_FLOWER";
    pub const CAKE: &str = "CAKE";
    pub const CALCITE: &str = "CALCITE";
    pub const CALIBRATED_SCULK_SENSOR: &str = "CALIBRATED_SCULK_SENSOR";
    pub const CAMPFIRE: &str = "CAMPFIRE";
    pub const CANDLE: &str = "CANDLE";
    pub const CANDLE_CAKE: &str = "CANDLE_CAKE";
    pub const CARROTS: &str = "CARROTS";
    pub const CARTOGRAPHY_TABLE: &str = "CARTOGRAPHY_TABLE";
    pub const CARVED_PUMPKIN: &str = "CARVED_PUMPKIN";
    pub const CAULDRON: &str = "CAULDRON";
    pub const CAVE_AIR: &str = "CAVE_AIR";
    pub const CAVE_VINES: &str = "CAVE_VINES";
    pub const CAVE_VINES_PLANT: &str = "CAVE_VINES_PLANT";
    pub const CHAIN_COMMAND_BLOCK: &str = "CHAIN_COMMAND_BLOCK";
    pub const CHERRY_BUTTON: &str = "CHERRY_BUTTON";
    pub const CHERRY_DOOR: &str = "CHERRY_DOOR";
    pub const CHERRY_FENCE: &str = "CHERRY_FENCE";
    pub const CHERRY_FENCE_GATE: &str = "CHERRY_FENCE_GATE";
    pub const CHERRY_HANGING_SIGN: &str = "CHERRY_HANGING_SIGN";
    pub const CHERRY_LEAVES: &str = "CHERRY_LEAVES";
    pub const CHERRY_LOG: &str = "CHERRY_LOG";
    pub const CHERRY_PLANKS: &str = "CHERRY_PLANKS";
    pub const CHERRY_PRESSURE_PLATE: &str = "CHERRY_PRESSURE_PLATE";
    pub const CHERRY_SAPLING: &str = "CHERRY_SAPLING";
    pub const CHERRY_SHELF: &str = "CHERRY_SHELF";
    pub const CHERRY_SIGN: &str = "CHERRY_SIGN";
    pub const CHERRY_SLAB: &str = "CHERRY_SLAB";
    pub const CHERRY_STAIRS: &str = "CHERRY_STAIRS";
    pub const CHERRY_TRAPDOOR: &str = "CHERRY_TRAPDOOR";
    pub const CHERRY_WALL_HANGING_SIGN: &str = "CHERRY_WALL_HANGING_SIGN";
    pub const CHERRY_WALL_SIGN: &str = "CHERRY_WALL_SIGN";
    pub const CHERRY_WOOD: &str = "CHERRY_WOOD";
    pub const CHEST: &str = "CHEST";
    pub const CHIPPED_ANVIL: &str = "CHIPPED_ANVIL";
    pub const CHISELED_BOOKSHELF: &str = "CHISELED_BOOKSHELF";
    pub const CHISELED_CINNABAR: &str = "CHISELED_CINNABAR";
    pub const CHISELED_COPPER: &str = "CHISELED_COPPER";
    pub const CHISELED_DEEPSLATE: &str = "CHISELED_DEEPSLATE";
    pub const CHISELED_NETHER_BRICKS: &str = "CHISELED_NETHER_BRICKS";
    pub const CHISELED_POLISHED_BLACKSTONE: &str = "CHISELED_POLISHED_BLACKSTONE";
    pub const CHISELED_QUARTZ_BLOCK: &str = "CHISELED_QUARTZ_BLOCK";
    pub const CHISELED_RED_SANDSTONE: &str = "CHISELED_RED_SANDSTONE";
    pub const CHISELED_RESIN_BRICKS: &str = "CHISELED_RESIN_BRICKS";
    pub const CHISELED_SANDSTONE: &str = "CHISELED_SANDSTONE";
    pub const CHISELED_STONE_BRICKS: &str = "CHISELED_STONE_BRICKS";
    pub const CHISELED_SULFUR: &str = "CHISELED_SULFUR";
    pub const CHISELED_TUFF: &str = "CHISELED_TUFF";
    pub const CHISELED_TUFF_BRICKS: &str = "CHISELED_TUFF_BRICKS";
    pub const CHORUS_FLOWER: &str = "CHORUS_FLOWER";
    pub const CHORUS_PLANT: &str = "CHORUS_PLANT";
    pub const CINNABAR: &str = "CINNABAR";
    pub const CINNABAR_BRICK_SLAB: &str = "CINNABAR_BRICK_SLAB";
    pub const CINNABAR_BRICK_STAIRS: &str = "CINNABAR_BRICK_STAIRS";
    pub const CINNABAR_BRICK_WALL: &str = "CINNABAR_BRICK_WALL";
    pub const CINNABAR_BRICKS: &str = "CINNABAR_BRICKS";
    pub const CINNABAR_SLAB: &str = "CINNABAR_SLAB";
    pub const CINNABAR_STAIRS: &str = "CINNABAR_STAIRS";
    pub const CINNABAR_WALL: &str = "CINNABAR_WALL";
    pub const CLAY: &str = "CLAY";
    pub const CLOSED_EYEBLOSSOM: &str = "CLOSED_EYEBLOSSOM";
    pub const COAL_BLOCK: &str = "COAL_BLOCK";
    pub const COAL_ORE: &str = "COAL_ORE";
    pub const COARSE_DIRT: &str = "COARSE_DIRT";
    pub const COBBLED_DEEPSLATE: &str = "COBBLED_DEEPSLATE";
    pub const COBBLED_DEEPSLATE_SLAB: &str = "COBBLED_DEEPSLATE_SLAB";
    pub const COBBLED_DEEPSLATE_STAIRS: &str = "COBBLED_DEEPSLATE_STAIRS";
    pub const COBBLED_DEEPSLATE_WALL: &str = "COBBLED_DEEPSLATE_WALL";
    pub const COBBLESTONE: &str = "COBBLESTONE";
    pub const COBBLESTONE_SLAB: &str = "COBBLESTONE_SLAB";
    pub const COBBLESTONE_STAIRS: &str = "COBBLESTONE_STAIRS";
    pub const COBBLESTONE_WALL: &str = "COBBLESTONE_WALL";
    pub const COBWEB: &str = "COBWEB";
    pub const COCOA: &str = "COCOA";
    pub const COMMAND_BLOCK: &str = "COMMAND_BLOCK";
    pub const COMPARATOR: &str = "COMPARATOR";
    pub const COMPOSTER: &str = "COMPOSTER";
    pub const CONDUIT: &str = "CONDUIT";
    pub const COPPER_BARS: &str = "COPPER_BARS";
    pub const COPPER_BLOCK: &str = "COPPER_BLOCK";
    pub const COPPER_BULB: &str = "COPPER_BULB";
    pub const COPPER_CHAIN: &str = "COPPER_CHAIN";
    pub const COPPER_CHEST: &str = "COPPER_CHEST";
    pub const COPPER_DOOR: &str = "COPPER_DOOR";
    pub const COPPER_GOLEM_STATUE: &str = "COPPER_GOLEM_STATUE";
    pub const COPPER_GRATE: &str = "COPPER_GRATE";
    pub const COPPER_LANTERN: &str = "COPPER_LANTERN";
    pub const COPPER_ORE: &str = "COPPER_ORE";
    pub const COPPER_TORCH: &str = "COPPER_TORCH";
    pub const COPPER_TRAPDOOR: &str = "COPPER_TRAPDOOR";
    pub const COPPER_WALL_TORCH: &str = "COPPER_WALL_TORCH";
    pub const CORNFLOWER: &str = "CORNFLOWER";
    pub const CRACKED_DEEPSLATE_BRICKS: &str = "CRACKED_DEEPSLATE_BRICKS";
    pub const CRACKED_DEEPSLATE_TILES: &str = "CRACKED_DEEPSLATE_TILES";
    pub const CRACKED_NETHER_BRICKS: &str = "CRACKED_NETHER_BRICKS";
    pub const CRACKED_POLISHED_BLACKSTONE_BRICKS: &str = "CRACKED_POLISHED_BLACKSTONE_BRICKS";
    pub const CRACKED_STONE_BRICKS: &str = "CRACKED_STONE_BRICKS";
    pub const CRAFTER: &str = "CRAFTER";
    pub const CRAFTING_TABLE: &str = "CRAFTING_TABLE";
    pub const CREAKING_HEART: &str = "CREAKING_HEART";
    pub const CREEPER_HEAD: &str = "CREEPER_HEAD";
    pub const CREEPER_WALL_HEAD: &str = "CREEPER_WALL_HEAD";
    pub const CRIMSON_BUTTON: &str = "CRIMSON_BUTTON";
    pub const CRIMSON_DOOR: &str = "CRIMSON_DOOR";
    pub const CRIMSON_FENCE: &str = "CRIMSON_FENCE";
    pub const CRIMSON_FENCE_GATE: &str = "CRIMSON_FENCE_GATE";
    pub const CRIMSON_FUNGUS: &str = "CRIMSON_FUNGUS";
    pub const CRIMSON_HANGING_SIGN: &str = "CRIMSON_HANGING_SIGN";
    pub const CRIMSON_HYPHAE: &str = "CRIMSON_HYPHAE";
    pub const CRIMSON_NYLIUM: &str = "CRIMSON_NYLIUM";
    pub const CRIMSON_PLANKS: &str = "CRIMSON_PLANKS";
    pub const CRIMSON_PRESSURE_PLATE: &str = "CRIMSON_PRESSURE_PLATE";
    pub const CRIMSON_ROOTS: &str = "CRIMSON_ROOTS";
    pub const CRIMSON_SHELF: &str = "CRIMSON_SHELF";
    pub const CRIMSON_SIGN: &str = "CRIMSON_SIGN";
    pub const CRIMSON_SLAB: &str = "CRIMSON_SLAB";
    pub const CRIMSON_STAIRS: &str = "CRIMSON_STAIRS";
    pub const CRIMSON_STEM: &str = "CRIMSON_STEM";
    pub const CRIMSON_TRAPDOOR: &str = "CRIMSON_TRAPDOOR";
    pub const CRIMSON_WALL_HANGING_SIGN: &str = "CRIMSON_WALL_HANGING_SIGN";
    pub const CRIMSON_WALL_SIGN: &str = "CRIMSON_WALL_SIGN";
    pub const CRYING_OBSIDIAN: &str = "CRYING_OBSIDIAN";
    pub const CUT_COPPER: &str = "CUT_COPPER";
    pub const CUT_COPPER_SLAB: &str = "CUT_COPPER_SLAB";
    pub const CUT_COPPER_STAIRS: &str = "CUT_COPPER_STAIRS";
    pub const CUT_RED_SANDSTONE: &str = "CUT_RED_SANDSTONE";
    pub const CUT_RED_SANDSTONE_SLAB: &str = "CUT_RED_SANDSTONE_SLAB";
    pub const CUT_SANDSTONE: &str = "CUT_SANDSTONE";
    pub const CUT_SANDSTONE_SLAB: &str = "CUT_SANDSTONE_SLAB";
    pub const CYAN_BANNER: &str = "CYAN_BANNER";
    pub const CYAN_BED: &str = "CYAN_BED";
    pub const CYAN_CANDLE: &str = "CYAN_CANDLE";
    pub const CYAN_CANDLE_CAKE: &str = "CYAN_CANDLE_CAKE";
    pub const CYAN_CARPET: &str = "CYAN_CARPET";
    pub const CYAN_CONCRETE: &str = "CYAN_CONCRETE";
    pub const CYAN_CONCRETE_POWDER: &str = "CYAN_CONCRETE_POWDER";
    pub const CYAN_CONCRETE_SLAB: &str = "CYAN_CONCRETE_SLAB";
    pub const CYAN_CONCRETE_STAIRS: &str = "CYAN_CONCRETE_STAIRS";
    pub const CYAN_GLAZED_TERRACOTTA: &str = "CYAN_GLAZED_TERRACOTTA";
    pub const CYAN_SHULKER_BOX: &str = "CYAN_SHULKER_BOX";
    pub const CYAN_STAINED_GLASS: &str = "CYAN_STAINED_GLASS";
    pub const CYAN_STAINED_GLASS_PANE: &str = "CYAN_STAINED_GLASS_PANE";
    pub const CYAN_TERRACOTTA: &str = "CYAN_TERRACOTTA";
    pub const CYAN_WALL_BANNER: &str = "CYAN_WALL_BANNER";
    pub const CYAN_WOOL: &str = "CYAN_WOOL";
    pub const CYAN_WOOL_SLAB: &str = "CYAN_WOOL_SLAB";
    pub const CYAN_WOOL_STAIRS: &str = "CYAN_WOOL_STAIRS";
    pub const DAMAGED_ANVIL: &str = "DAMAGED_ANVIL";
    pub const DANDELION: &str = "DANDELION";
    pub const DARK_OAK_BUTTON: &str = "DARK_OAK_BUTTON";
    pub const DARK_OAK_DOOR: &str = "DARK_OAK_DOOR";
    pub const DARK_OAK_FENCE: &str = "DARK_OAK_FENCE";
    pub const DARK_OAK_FENCE_GATE: &str = "DARK_OAK_FENCE_GATE";
    pub const DARK_OAK_HANGING_SIGN: &str = "DARK_OAK_HANGING_SIGN";
    pub const DARK_OAK_LEAVES: &str = "DARK_OAK_LEAVES";
    pub const DARK_OAK_LOG: &str = "DARK_OAK_LOG";
    pub const DARK_OAK_PLANKS: &str = "DARK_OAK_PLANKS";
    pub const DARK_OAK_PRESSURE_PLATE: &str = "DARK_OAK_PRESSURE_PLATE";
    pub const DARK_OAK_SAPLING: &str = "DARK_OAK_SAPLING";
    pub const DARK_OAK_SHELF: &str = "DARK_OAK_SHELF";
    pub const DARK_OAK_SIGN: &str = "DARK_OAK_SIGN";
    pub const DARK_OAK_SLAB: &str = "DARK_OAK_SLAB";
    pub const DARK_OAK_STAIRS: &str = "DARK_OAK_STAIRS";
    pub const DARK_OAK_TRAPDOOR: &str = "DARK_OAK_TRAPDOOR";
    pub const DARK_OAK_WALL_HANGING_SIGN: &str = "DARK_OAK_WALL_HANGING_SIGN";
    pub const DARK_OAK_WALL_SIGN: &str = "DARK_OAK_WALL_SIGN";
    pub const DARK_OAK_WOOD: &str = "DARK_OAK_WOOD";
    pub const DARK_PRISMARINE: &str = "DARK_PRISMARINE";
    pub const DARK_PRISMARINE_SLAB: &str = "DARK_PRISMARINE_SLAB";
    pub const DARK_PRISMARINE_STAIRS: &str = "DARK_PRISMARINE_STAIRS";
    pub const DAYLIGHT_DETECTOR: &str = "DAYLIGHT_DETECTOR";
    pub const DEAD_BRAIN_CORAL: &str = "DEAD_BRAIN_CORAL";
    pub const DEAD_BRAIN_CORAL_BLOCK: &str = "DEAD_BRAIN_CORAL_BLOCK";
    pub const DEAD_BRAIN_CORAL_FAN: &str = "DEAD_BRAIN_CORAL_FAN";
    pub const DEAD_BRAIN_CORAL_WALL_FAN: &str = "DEAD_BRAIN_CORAL_WALL_FAN";
    pub const DEAD_BUBBLE_CORAL: &str = "DEAD_BUBBLE_CORAL";
    pub const DEAD_BUBBLE_CORAL_BLOCK: &str = "DEAD_BUBBLE_CORAL_BLOCK";
    pub const DEAD_BUBBLE_CORAL_FAN: &str = "DEAD_BUBBLE_CORAL_FAN";
    pub const DEAD_BUBBLE_CORAL_WALL_FAN: &str = "DEAD_BUBBLE_CORAL_WALL_FAN";
    pub const DEAD_BUSH: &str = "DEAD_BUSH";
    pub const DEAD_FIRE_CORAL: &str = "DEAD_FIRE_CORAL";
    pub const DEAD_FIRE_CORAL_BLOCK: &str = "DEAD_FIRE_CORAL_BLOCK";
    pub const DEAD_FIRE_CORAL_FAN: &str = "DEAD_FIRE_CORAL_FAN";
    pub const DEAD_FIRE_CORAL_WALL_FAN: &str = "DEAD_FIRE_CORAL_WALL_FAN";
    pub const DEAD_HORN_CORAL: &str = "DEAD_HORN_CORAL";
    pub const DEAD_HORN_CORAL_BLOCK: &str = "DEAD_HORN_CORAL_BLOCK";
    pub const DEAD_HORN_CORAL_FAN: &str = "DEAD_HORN_CORAL_FAN";
    pub const DEAD_HORN_CORAL_WALL_FAN: &str = "DEAD_HORN_CORAL_WALL_FAN";
    pub const DEAD_TUBE_CORAL: &str = "DEAD_TUBE_CORAL";
    pub const DEAD_TUBE_CORAL_BLOCK: &str = "DEAD_TUBE_CORAL_BLOCK";
    pub const DEAD_TUBE_CORAL_FAN: &str = "DEAD_TUBE_CORAL_FAN";
    pub const DEAD_TUBE_CORAL_WALL_FAN: &str = "DEAD_TUBE_CORAL_WALL_FAN";
    pub const DECORATED_POT: &str = "DECORATED_POT";
    pub const DEEPSLATE: &str = "DEEPSLATE";
    pub const DEEPSLATE_BRICK_SLAB: &str = "DEEPSLATE_BRICK_SLAB";
    pub const DEEPSLATE_BRICK_STAIRS: &str = "DEEPSLATE_BRICK_STAIRS";
    pub const DEEPSLATE_BRICK_WALL: &str = "DEEPSLATE_BRICK_WALL";
    pub const DEEPSLATE_BRICKS: &str = "DEEPSLATE_BRICKS";
    pub const DEEPSLATE_COAL_ORE: &str = "DEEPSLATE_COAL_ORE";
    pub const DEEPSLATE_COPPER_ORE: &str = "DEEPSLATE_COPPER_ORE";
    pub const DEEPSLATE_DIAMOND_ORE: &str = "DEEPSLATE_DIAMOND_ORE";
    pub const DEEPSLATE_EMERALD_ORE: &str = "DEEPSLATE_EMERALD_ORE";
    pub const DEEPSLATE_GOLD_ORE: &str = "DEEPSLATE_GOLD_ORE";
    pub const DEEPSLATE_IRON_ORE: &str = "DEEPSLATE_IRON_ORE";
    pub const DEEPSLATE_LAPIS_ORE: &str = "DEEPSLATE_LAPIS_ORE";
    pub const DEEPSLATE_REDSTONE_ORE: &str = "DEEPSLATE_REDSTONE_ORE";
    pub const DEEPSLATE_TILE_SLAB: &str = "DEEPSLATE_TILE_SLAB";
    pub const DEEPSLATE_TILE_STAIRS: &str = "DEEPSLATE_TILE_STAIRS";
    pub const DEEPSLATE_TILE_WALL: &str = "DEEPSLATE_TILE_WALL";
    pub const DEEPSLATE_TILES: &str = "DEEPSLATE_TILES";
    pub const DETECTOR_RAIL: &str = "DETECTOR_RAIL";
    pub const DIAMOND_BLOCK: &str = "DIAMOND_BLOCK";
    pub const DIAMOND_ORE: &str = "DIAMOND_ORE";
    pub const DIORITE: &str = "DIORITE";
    pub const DIORITE_SLAB: &str = "DIORITE_SLAB";
    pub const DIORITE_STAIRS: &str = "DIORITE_STAIRS";
    pub const DIORITE_WALL: &str = "DIORITE_WALL";
    pub const DIRT: &str = "DIRT";
    pub const DIRT_PATH: &str = "DIRT_PATH";
    pub const DISPENSER: &str = "DISPENSER";
    pub const DRAGON_EGG: &str = "DRAGON_EGG";
    pub const DRAGON_HEAD: &str = "DRAGON_HEAD";
    pub const DRAGON_WALL_HEAD: &str = "DRAGON_WALL_HEAD";
    pub const DRIED_GHAST: &str = "DRIED_GHAST";
    pub const DRIED_KELP_BLOCK: &str = "DRIED_KELP_BLOCK";
    pub const DRIPSTONE_BLOCK: &str = "DRIPSTONE_BLOCK";
    pub const DROPPER: &str = "DROPPER";
    pub const EMERALD_BLOCK: &str = "EMERALD_BLOCK";
    pub const EMERALD_ORE: &str = "EMERALD_ORE";
    pub const ENCHANTING_TABLE: &str = "ENCHANTING_TABLE";
    pub const END_GATEWAY: &str = "END_GATEWAY";
    pub const END_PORTAL: &str = "END_PORTAL";
    pub const END_PORTAL_FRAME: &str = "END_PORTAL_FRAME";
    pub const END_ROD: &str = "END_ROD";
    pub const END_STONE: &str = "END_STONE";
    pub const END_STONE_BRICK_SLAB: &str = "END_STONE_BRICK_SLAB";
    pub const END_STONE_BRICK_STAIRS: &str = "END_STONE_BRICK_STAIRS";
    pub const END_STONE_BRICK_WALL: &str = "END_STONE_BRICK_WALL";
    pub const END_STONE_BRICKS: &str = "END_STONE_BRICKS";
    pub const ENDER_CHEST: &str = "ENDER_CHEST";
    pub const EXPOSED_CHISELED_COPPER: &str = "EXPOSED_CHISELED_COPPER";
    pub const EXPOSED_COPPER: &str = "EXPOSED_COPPER";
    pub const EXPOSED_COPPER_BARS: &str = "EXPOSED_COPPER_BARS";
    pub const EXPOSED_COPPER_BULB: &str = "EXPOSED_COPPER_BULB";
    pub const EXPOSED_COPPER_CHAIN: &str = "EXPOSED_COPPER_CHAIN";
    pub const EXPOSED_COPPER_CHEST: &str = "EXPOSED_COPPER_CHEST";
    pub const EXPOSED_COPPER_DOOR: &str = "EXPOSED_COPPER_DOOR";
    pub const EXPOSED_COPPER_GOLEM_STATUE: &str = "EXPOSED_COPPER_GOLEM_STATUE";
    pub const EXPOSED_COPPER_GRATE: &str = "EXPOSED_COPPER_GRATE";
    pub const EXPOSED_COPPER_LANTERN: &str = "EXPOSED_COPPER_LANTERN";
    pub const EXPOSED_COPPER_TRAPDOOR: &str = "EXPOSED_COPPER_TRAPDOOR";
    pub const EXPOSED_CUT_COPPER: &str = "EXPOSED_CUT_COPPER";
    pub const EXPOSED_CUT_COPPER_SLAB: &str = "EXPOSED_CUT_COPPER_SLAB";
    pub const EXPOSED_CUT_COPPER_STAIRS: &str = "EXPOSED_CUT_COPPER_STAIRS";
    pub const EXPOSED_LIGHTNING_ROD: &str = "EXPOSED_LIGHTNING_ROD";
    pub const FARMLAND: &str = "FARMLAND";
    pub const FERN: &str = "FERN";
    pub const FIRE: &str = "FIRE";
    pub const FIRE_CORAL: &str = "FIRE_CORAL";
    pub const FIRE_CORAL_BLOCK: &str = "FIRE_CORAL_BLOCK";
    pub const FIRE_CORAL_FAN: &str = "FIRE_CORAL_FAN";
    pub const FIRE_CORAL_WALL_FAN: &str = "FIRE_CORAL_WALL_FAN";
    pub const FIREFLY_BUSH: &str = "FIREFLY_BUSH";
    pub const FLETCHING_TABLE: &str = "FLETCHING_TABLE";
    pub const FLOWER_POT: &str = "FLOWER_POT";
    pub const FLOWERING_AZALEA: &str = "FLOWERING_AZALEA";
    pub const FLOWERING_AZALEA_LEAVES: &str = "FLOWERING_AZALEA_LEAVES";
    pub const FROGSPAWN: &str = "FROGSPAWN";
    pub const FROSTED_ICE: &str = "FROSTED_ICE";
    pub const FURNACE: &str = "FURNACE";
    pub const GILDED_BLACKSTONE: &str = "GILDED_BLACKSTONE";
    pub const GLASS: &str = "GLASS";
    pub const GLASS_PANE: &str = "GLASS_PANE";
    pub const GLOW_LICHEN: &str = "GLOW_LICHEN";
    pub const GLOWSTONE: &str = "GLOWSTONE";
    pub const GOLD_BLOCK: &str = "GOLD_BLOCK";
    pub const GOLD_ORE: &str = "GOLD_ORE";
    pub const GOLDEN_DANDELION: &str = "GOLDEN_DANDELION";
    pub const GRANITE: &str = "GRANITE";
    pub const GRANITE_SLAB: &str = "GRANITE_SLAB";
    pub const GRANITE_STAIRS: &str = "GRANITE_STAIRS";
    pub const GRANITE_WALL: &str = "GRANITE_WALL";
    pub const GRASS_BLOCK: &str = "GRASS_BLOCK";
    pub const GRAVEL: &str = "GRAVEL";
    pub const GRAY_BANNER: &str = "GRAY_BANNER";
    pub const GRAY_BED: &str = "GRAY_BED";
    pub const GRAY_CANDLE: &str = "GRAY_CANDLE";
    pub const GRAY_CANDLE_CAKE: &str = "GRAY_CANDLE_CAKE";
    pub const GRAY_CARPET: &str = "GRAY_CARPET";
    pub const GRAY_CONCRETE: &str = "GRAY_CONCRETE";
    pub const GRAY_CONCRETE_POWDER: &str = "GRAY_CONCRETE_POWDER";
    pub const GRAY_CONCRETE_SLAB: &str = "GRAY_CONCRETE_SLAB";
    pub const GRAY_CONCRETE_STAIRS: &str = "GRAY_CONCRETE_STAIRS";
    pub const GRAY_GLAZED_TERRACOTTA: &str = "GRAY_GLAZED_TERRACOTTA";
    pub const GRAY_SHULKER_BOX: &str = "GRAY_SHULKER_BOX";
    pub const GRAY_STAINED_GLASS: &str = "GRAY_STAINED_GLASS";
    pub const GRAY_STAINED_GLASS_PANE: &str = "GRAY_STAINED_GLASS_PANE";
    pub const GRAY_TERRACOTTA: &str = "GRAY_TERRACOTTA";
    pub const GRAY_WALL_BANNER: &str = "GRAY_WALL_BANNER";
    pub const GRAY_WOOL: &str = "GRAY_WOOL";
    pub const GRAY_WOOL_SLAB: &str = "GRAY_WOOL_SLAB";
    pub const GRAY_WOOL_STAIRS: &str = "GRAY_WOOL_STAIRS";
    pub const GREEN_BANNER: &str = "GREEN_BANNER";
    pub const GREEN_BED: &str = "GREEN_BED";
    pub const GREEN_CANDLE: &str = "GREEN_CANDLE";
    pub const GREEN_CANDLE_CAKE: &str = "GREEN_CANDLE_CAKE";
    pub const GREEN_CARPET: &str = "GREEN_CARPET";
    pub const GREEN_CONCRETE: &str = "GREEN_CONCRETE";
    pub const GREEN_CONCRETE_POWDER: &str = "GREEN_CONCRETE_POWDER";
    pub const GREEN_CONCRETE_SLAB: &str = "GREEN_CONCRETE_SLAB";
    pub const GREEN_CONCRETE_STAIRS: &str = "GREEN_CONCRETE_STAIRS";
    pub const GREEN_GLAZED_TERRACOTTA: &str = "GREEN_GLAZED_TERRACOTTA";
    pub const GREEN_SHULKER_BOX: &str = "GREEN_SHULKER_BOX";
    pub const GREEN_STAINED_GLASS: &str = "GREEN_STAINED_GLASS";
    pub const GREEN_STAINED_GLASS_PANE: &str = "GREEN_STAINED_GLASS_PANE";
    pub const GREEN_TERRACOTTA: &str = "GREEN_TERRACOTTA";
    pub const GREEN_WALL_BANNER: &str = "GREEN_WALL_BANNER";
    pub const GREEN_WOOL: &str = "GREEN_WOOL";
    pub const GREEN_WOOL_SLAB: &str = "GREEN_WOOL_SLAB";
    pub const GREEN_WOOL_STAIRS: &str = "GREEN_WOOL_STAIRS";
    pub const GRINDSTONE: &str = "GRINDSTONE";
    pub const HANGING_ROOTS: &str = "HANGING_ROOTS";
    pub const HAY_BLOCK: &str = "HAY_BLOCK";
    pub const HEAVY_CORE: &str = "HEAVY_CORE";
    pub const HEAVY_WEIGHTED_PRESSURE_PLATE: &str = "HEAVY_WEIGHTED_PRESSURE_PLATE";
    pub const HONEY_BLOCK: &str = "HONEY_BLOCK";
    pub const HONEYCOMB_BLOCK: &str = "HONEYCOMB_BLOCK";
    pub const HOPPER: &str = "HOPPER";
    pub const HORN_CORAL: &str = "HORN_CORAL";
    pub const HORN_CORAL_BLOCK: &str = "HORN_CORAL_BLOCK";
    pub const HORN_CORAL_FAN: &str = "HORN_CORAL_FAN";
    pub const HORN_CORAL_WALL_FAN: &str = "HORN_CORAL_WALL_FAN";
    pub const ICE: &str = "ICE";
    pub const INFESTED_CHISELED_STONE_BRICKS: &str = "INFESTED_CHISELED_STONE_BRICKS";
    pub const INFESTED_COBBLESTONE: &str = "INFESTED_COBBLESTONE";
    pub const INFESTED_CRACKED_STONE_BRICKS: &str = "INFESTED_CRACKED_STONE_BRICKS";
    pub const INFESTED_DEEPSLATE: &str = "INFESTED_DEEPSLATE";
    pub const INFESTED_MOSSY_STONE_BRICKS: &str = "INFESTED_MOSSY_STONE_BRICKS";
    pub const INFESTED_STONE: &str = "INFESTED_STONE";
    pub const INFESTED_STONE_BRICKS: &str = "INFESTED_STONE_BRICKS";
    pub const IRON_BARS: &str = "IRON_BARS";
    pub const IRON_BLOCK: &str = "IRON_BLOCK";
    pub const IRON_CHAIN: &str = "IRON_CHAIN";
    pub const IRON_DOOR: &str = "IRON_DOOR";
    pub const IRON_ORE: &str = "IRON_ORE";
    pub const IRON_TRAPDOOR: &str = "IRON_TRAPDOOR";
    pub const JACK_O_LANTERN: &str = "JACK_O_LANTERN";
    pub const JIGSAW: &str = "JIGSAW";
    pub const JUKEBOX: &str = "JUKEBOX";
    pub const JUNGLE_BUTTON: &str = "JUNGLE_BUTTON";
    pub const JUNGLE_DOOR: &str = "JUNGLE_DOOR";
    pub const JUNGLE_FENCE: &str = "JUNGLE_FENCE";
    pub const JUNGLE_FENCE_GATE: &str = "JUNGLE_FENCE_GATE";
    pub const JUNGLE_HANGING_SIGN: &str = "JUNGLE_HANGING_SIGN";
    pub const JUNGLE_LEAVES: &str = "JUNGLE_LEAVES";
    pub const JUNGLE_LOG: &str = "JUNGLE_LOG";
    pub const JUNGLE_PLANKS: &str = "JUNGLE_PLANKS";
    pub const JUNGLE_PRESSURE_PLATE: &str = "JUNGLE_PRESSURE_PLATE";
    pub const JUNGLE_SAPLING: &str = "JUNGLE_SAPLING";
    pub const JUNGLE_SHELF: &str = "JUNGLE_SHELF";
    pub const JUNGLE_SIGN: &str = "JUNGLE_SIGN";
    pub const JUNGLE_SLAB: &str = "JUNGLE_SLAB";
    pub const JUNGLE_STAIRS: &str = "JUNGLE_STAIRS";
    pub const JUNGLE_TRAPDOOR: &str = "JUNGLE_TRAPDOOR";
    pub const JUNGLE_WALL_HANGING_SIGN: &str = "JUNGLE_WALL_HANGING_SIGN";
    pub const JUNGLE_WALL_SIGN: &str = "JUNGLE_WALL_SIGN";
    pub const JUNGLE_WOOD: &str = "JUNGLE_WOOD";
    pub const KELP: &str = "KELP";
    pub const KELP_PLANT: &str = "KELP_PLANT";
    pub const LADDER: &str = "LADDER";
    pub const LANTERN: &str = "LANTERN";
    pub const LAPIS_BLOCK: &str = "LAPIS_BLOCK";
    pub const LAPIS_ORE: &str = "LAPIS_ORE";
    pub const LARGE_AMETHYST_BUD: &str = "LARGE_AMETHYST_BUD";
    pub const LARGE_FERN: &str = "LARGE_FERN";
    pub const LAVA: &str = "LAVA";
    pub const LAVA_CAULDRON: &str = "LAVA_CAULDRON";
    pub const LEAF_LITTER: &str = "LEAF_LITTER";
    pub const LECTERN: &str = "LECTERN";
    pub const LEVER: &str = "LEVER";
    pub const LIGHT: &str = "LIGHT";
    pub const LIGHT_BLUE_BANNER: &str = "LIGHT_BLUE_BANNER";
    pub const LIGHT_BLUE_BED: &str = "LIGHT_BLUE_BED";
    pub const LIGHT_BLUE_CANDLE: &str = "LIGHT_BLUE_CANDLE";
    pub const LIGHT_BLUE_CANDLE_CAKE: &str = "LIGHT_BLUE_CANDLE_CAKE";
    pub const LIGHT_BLUE_CARPET: &str = "LIGHT_BLUE_CARPET";
    pub const LIGHT_BLUE_CONCRETE: &str = "LIGHT_BLUE_CONCRETE";
    pub const LIGHT_BLUE_CONCRETE_POWDER: &str = "LIGHT_BLUE_CONCRETE_POWDER";
    pub const LIGHT_BLUE_CONCRETE_SLAB: &str = "LIGHT_BLUE_CONCRETE_SLAB";
    pub const LIGHT_BLUE_CONCRETE_STAIRS: &str = "LIGHT_BLUE_CONCRETE_STAIRS";
    pub const LIGHT_BLUE_GLAZED_TERRACOTTA: &str = "LIGHT_BLUE_GLAZED_TERRACOTTA";
    pub const LIGHT_BLUE_SHULKER_BOX: &str = "LIGHT_BLUE_SHULKER_BOX";
    pub const LIGHT_BLUE_STAINED_GLASS: &str = "LIGHT_BLUE_STAINED_GLASS";
    pub const LIGHT_BLUE_STAINED_GLASS_PANE: &str = "LIGHT_BLUE_STAINED_GLASS_PANE";
    pub const LIGHT_BLUE_TERRACOTTA: &str = "LIGHT_BLUE_TERRACOTTA";
    pub const LIGHT_BLUE_WALL_BANNER: &str = "LIGHT_BLUE_WALL_BANNER";
    pub const LIGHT_BLUE_WOOL: &str = "LIGHT_BLUE_WOOL";
    pub const LIGHT_BLUE_WOOL_SLAB: &str = "LIGHT_BLUE_WOOL_SLAB";
    pub const LIGHT_BLUE_WOOL_STAIRS: &str = "LIGHT_BLUE_WOOL_STAIRS";
    pub const LIGHT_GRAY_BANNER: &str = "LIGHT_GRAY_BANNER";
    pub const LIGHT_GRAY_BED: &str = "LIGHT_GRAY_BED";
    pub const LIGHT_GRAY_CANDLE: &str = "LIGHT_GRAY_CANDLE";
    pub const LIGHT_GRAY_CANDLE_CAKE: &str = "LIGHT_GRAY_CANDLE_CAKE";
    pub const LIGHT_GRAY_CARPET: &str = "LIGHT_GRAY_CARPET";
    pub const LIGHT_GRAY_CONCRETE: &str = "LIGHT_GRAY_CONCRETE";
    pub const LIGHT_GRAY_CONCRETE_POWDER: &str = "LIGHT_GRAY_CONCRETE_POWDER";
    pub const LIGHT_GRAY_CONCRETE_SLAB: &str = "LIGHT_GRAY_CONCRETE_SLAB";
    pub const LIGHT_GRAY_CONCRETE_STAIRS: &str = "LIGHT_GRAY_CONCRETE_STAIRS";
    pub const LIGHT_GRAY_GLAZED_TERRACOTTA: &str = "LIGHT_GRAY_GLAZED_TERRACOTTA";
    pub const LIGHT_GRAY_SHULKER_BOX: &str = "LIGHT_GRAY_SHULKER_BOX";
    pub const LIGHT_GRAY_STAINED_GLASS: &str = "LIGHT_GRAY_STAINED_GLASS";
    pub const LIGHT_GRAY_STAINED_GLASS_PANE: &str = "LIGHT_GRAY_STAINED_GLASS_PANE";
    pub const LIGHT_GRAY_TERRACOTTA: &str = "LIGHT_GRAY_TERRACOTTA";
    pub const LIGHT_GRAY_WALL_BANNER: &str = "LIGHT_GRAY_WALL_BANNER";
    pub const LIGHT_GRAY_WOOL: &str = "LIGHT_GRAY_WOOL";
    pub const LIGHT_GRAY_WOOL_SLAB: &str = "LIGHT_GRAY_WOOL_SLAB";
    pub const LIGHT_GRAY_WOOL_STAIRS: &str = "LIGHT_GRAY_WOOL_STAIRS";
    pub const LIGHT_WEIGHTED_PRESSURE_PLATE: &str = "LIGHT_WEIGHTED_PRESSURE_PLATE";
    pub const LIGHTNING_ROD: &str = "LIGHTNING_ROD";
    pub const LILAC: &str = "LILAC";
    pub const LILY_OF_THE_VALLEY: &str = "LILY_OF_THE_VALLEY";
    pub const LILY_PAD: &str = "LILY_PAD";
    pub const LIME_BANNER: &str = "LIME_BANNER";
    pub const LIME_BED: &str = "LIME_BED";
    pub const LIME_CANDLE: &str = "LIME_CANDLE";
    pub const LIME_CANDLE_CAKE: &str = "LIME_CANDLE_CAKE";
    pub const LIME_CARPET: &str = "LIME_CARPET";
    pub const LIME_CONCRETE: &str = "LIME_CONCRETE";
    pub const LIME_CONCRETE_POWDER: &str = "LIME_CONCRETE_POWDER";
    pub const LIME_CONCRETE_SLAB: &str = "LIME_CONCRETE_SLAB";
    pub const LIME_CONCRETE_STAIRS: &str = "LIME_CONCRETE_STAIRS";
    pub const LIME_GLAZED_TERRACOTTA: &str = "LIME_GLAZED_TERRACOTTA";
    pub const LIME_SHULKER_BOX: &str = "LIME_SHULKER_BOX";
    pub const LIME_STAINED_GLASS: &str = "LIME_STAINED_GLASS";
    pub const LIME_STAINED_GLASS_PANE: &str = "LIME_STAINED_GLASS_PANE";
    pub const LIME_TERRACOTTA: &str = "LIME_TERRACOTTA";
    pub const LIME_WALL_BANNER: &str = "LIME_WALL_BANNER";
    pub const LIME_WOOL: &str = "LIME_WOOL";
    pub const LIME_WOOL_SLAB: &str = "LIME_WOOL_SLAB";
    pub const LIME_WOOL_STAIRS: &str = "LIME_WOOL_STAIRS";
    pub const LODESTONE: &str = "LODESTONE";
    pub const LOOM: &str = "LOOM";
    pub const MAGENTA_BANNER: &str = "MAGENTA_BANNER";
    pub const MAGENTA_BED: &str = "MAGENTA_BED";
    pub const MAGENTA_CANDLE: &str = "MAGENTA_CANDLE";
    pub const MAGENTA_CANDLE_CAKE: &str = "MAGENTA_CANDLE_CAKE";
    pub const MAGENTA_CARPET: &str = "MAGENTA_CARPET";
    pub const MAGENTA_CONCRETE: &str = "MAGENTA_CONCRETE";
    pub const MAGENTA_CONCRETE_POWDER: &str = "MAGENTA_CONCRETE_POWDER";
    pub const MAGENTA_CONCRETE_SLAB: &str = "MAGENTA_CONCRETE_SLAB";
    pub const MAGENTA_CONCRETE_STAIRS: &str = "MAGENTA_CONCRETE_STAIRS";
    pub const MAGENTA_GLAZED_TERRACOTTA: &str = "MAGENTA_GLAZED_TERRACOTTA";
    pub const MAGENTA_SHULKER_BOX: &str = "MAGENTA_SHULKER_BOX";
    pub const MAGENTA_STAINED_GLASS: &str = "MAGENTA_STAINED_GLASS";
    pub const MAGENTA_STAINED_GLASS_PANE: &str = "MAGENTA_STAINED_GLASS_PANE";
    pub const MAGENTA_TERRACOTTA: &str = "MAGENTA_TERRACOTTA";
    pub const MAGENTA_WALL_BANNER: &str = "MAGENTA_WALL_BANNER";
    pub const MAGENTA_WOOL: &str = "MAGENTA_WOOL";
    pub const MAGENTA_WOOL_SLAB: &str = "MAGENTA_WOOL_SLAB";
    pub const MAGENTA_WOOL_STAIRS: &str = "MAGENTA_WOOL_STAIRS";
    pub const MAGMA_BLOCK: &str = "MAGMA_BLOCK";
    pub const MANGROVE_BUTTON: &str = "MANGROVE_BUTTON";
    pub const MANGROVE_DOOR: &str = "MANGROVE_DOOR";
    pub const MANGROVE_FENCE: &str = "MANGROVE_FENCE";
    pub const MANGROVE_FENCE_GATE: &str = "MANGROVE_FENCE_GATE";
    pub const MANGROVE_HANGING_SIGN: &str = "MANGROVE_HANGING_SIGN";
    pub const MANGROVE_LEAVES: &str = "MANGROVE_LEAVES";
    pub const MANGROVE_LOG: &str = "MANGROVE_LOG";
    pub const MANGROVE_PLANKS: &str = "MANGROVE_PLANKS";
    pub const MANGROVE_PRESSURE_PLATE: &str = "MANGROVE_PRESSURE_PLATE";
    pub const MANGROVE_PROPAGULE: &str = "MANGROVE_PROPAGULE";
    pub const MANGROVE_ROOTS: &str = "MANGROVE_ROOTS";
    pub const MANGROVE_SHELF: &str = "MANGROVE_SHELF";
    pub const MANGROVE_SIGN: &str = "MANGROVE_SIGN";
    pub const MANGROVE_SLAB: &str = "MANGROVE_SLAB";
    pub const MANGROVE_STAIRS: &str = "MANGROVE_STAIRS";
    pub const MANGROVE_TRAPDOOR: &str = "MANGROVE_TRAPDOOR";
    pub const MANGROVE_WALL_HANGING_SIGN: &str = "MANGROVE_WALL_HANGING_SIGN";
    pub const MANGROVE_WALL_SIGN: &str = "MANGROVE_WALL_SIGN";
    pub const MANGROVE_WOOD: &str = "MANGROVE_WOOD";
    pub const MEDIUM_AMETHYST_BUD: &str = "MEDIUM_AMETHYST_BUD";
    pub const MELON: &str = "MELON";
    pub const MELON_STEM: &str = "MELON_STEM";
    pub const MOSS_BLOCK: &str = "MOSS_BLOCK";
    pub const MOSS_CARPET: &str = "MOSS_CARPET";
    pub const MOSSY_COBBLESTONE: &str = "MOSSY_COBBLESTONE";
    pub const MOSSY_COBBLESTONE_SLAB: &str = "MOSSY_COBBLESTONE_SLAB";
    pub const MOSSY_COBBLESTONE_STAIRS: &str = "MOSSY_COBBLESTONE_STAIRS";
    pub const MOSSY_COBBLESTONE_WALL: &str = "MOSSY_COBBLESTONE_WALL";
    pub const MOSSY_STONE_BRICK_SLAB: &str = "MOSSY_STONE_BRICK_SLAB";
    pub const MOSSY_STONE_BRICK_STAIRS: &str = "MOSSY_STONE_BRICK_STAIRS";
    pub const MOSSY_STONE_BRICK_WALL: &str = "MOSSY_STONE_BRICK_WALL";
    pub const MOSSY_STONE_BRICKS: &str = "MOSSY_STONE_BRICKS";
    pub const MOVING_PISTON: &str = "MOVING_PISTON";
    pub const MUD: &str = "MUD";
    pub const MUD_BRICK_SLAB: &str = "MUD_BRICK_SLAB";
    pub const MUD_BRICK_STAIRS: &str = "MUD_BRICK_STAIRS";
    pub const MUD_BRICK_WALL: &str = "MUD_BRICK_WALL";
    pub const MUD_BRICKS: &str = "MUD_BRICKS";
    pub const MUDDY_MANGROVE_ROOTS: &str = "MUDDY_MANGROVE_ROOTS";
    pub const MUSHROOM_STEM: &str = "MUSHROOM_STEM";
    pub const MYCELIUM: &str = "MYCELIUM";
    pub const NETHER_BRICK_FENCE: &str = "NETHER_BRICK_FENCE";
    pub const NETHER_BRICK_SLAB: &str = "NETHER_BRICK_SLAB";
    pub const NETHER_BRICK_STAIRS: &str = "NETHER_BRICK_STAIRS";
    pub const NETHER_BRICK_WALL: &str = "NETHER_BRICK_WALL";
    pub const NETHER_BRICKS: &str = "NETHER_BRICKS";
    pub const NETHER_GOLD_ORE: &str = "NETHER_GOLD_ORE";
    pub const NETHER_PORTAL: &str = "NETHER_PORTAL";
    pub const NETHER_QUARTZ_ORE: &str = "NETHER_QUARTZ_ORE";
    pub const NETHER_SPROUTS: &str = "NETHER_SPROUTS";
    pub const NETHER_WART: &str = "NETHER_WART";
    pub const NETHER_WART_BLOCK: &str = "NETHER_WART_BLOCK";
    pub const NETHERITE_BLOCK: &str = "NETHERITE_BLOCK";
    pub const NETHERRACK: &str = "NETHERRACK";
    pub const NOTE_BLOCK: &str = "NOTE_BLOCK";
    pub const OAK_BUTTON: &str = "OAK_BUTTON";
    pub const OAK_DOOR: &str = "OAK_DOOR";
    pub const OAK_FENCE: &str = "OAK_FENCE";
    pub const OAK_FENCE_GATE: &str = "OAK_FENCE_GATE";
    pub const OAK_HANGING_SIGN: &str = "OAK_HANGING_SIGN";
    pub const OAK_LEAVES: &str = "OAK_LEAVES";
    pub const OAK_LOG: &str = "OAK_LOG";
    pub const OAK_PLANKS: &str = "OAK_PLANKS";
    pub const OAK_PRESSURE_PLATE: &str = "OAK_PRESSURE_PLATE";
    pub const OAK_SAPLING: &str = "OAK_SAPLING";
    pub const OAK_SHELF: &str = "OAK_SHELF";
    pub const OAK_SIGN: &str = "OAK_SIGN";
    pub const OAK_SLAB: &str = "OAK_SLAB";
    pub const OAK_STAIRS: &str = "OAK_STAIRS";
    pub const OAK_TRAPDOOR: &str = "OAK_TRAPDOOR";
    pub const OAK_WALL_HANGING_SIGN: &str = "OAK_WALL_HANGING_SIGN";
    pub const OAK_WALL_SIGN: &str = "OAK_WALL_SIGN";
    pub const OAK_WOOD: &str = "OAK_WOOD";
    pub const OBSERVER: &str = "OBSERVER";
    pub const OBSIDIAN: &str = "OBSIDIAN";
    pub const OCHRE_FROGLIGHT: &str = "OCHRE_FROGLIGHT";
    pub const OPEN_EYEBLOSSOM: &str = "OPEN_EYEBLOSSOM";
    pub const ORANGE_BANNER: &str = "ORANGE_BANNER";
    pub const ORANGE_BED: &str = "ORANGE_BED";
    pub const ORANGE_CANDLE: &str = "ORANGE_CANDLE";
    pub const ORANGE_CANDLE_CAKE: &str = "ORANGE_CANDLE_CAKE";
    pub const ORANGE_CARPET: &str = "ORANGE_CARPET";
    pub const ORANGE_CONCRETE: &str = "ORANGE_CONCRETE";
    pub const ORANGE_CONCRETE_POWDER: &str = "ORANGE_CONCRETE_POWDER";
    pub const ORANGE_CONCRETE_SLAB: &str = "ORANGE_CONCRETE_SLAB";
    pub const ORANGE_CONCRETE_STAIRS: &str = "ORANGE_CONCRETE_STAIRS";
    pub const ORANGE_GLAZED_TERRACOTTA: &str = "ORANGE_GLAZED_TERRACOTTA";
    pub const ORANGE_POPLAR_LEAVES: &str = "ORANGE_POPLAR_LEAVES";
    pub const ORANGE_SHULKER_BOX: &str = "ORANGE_SHULKER_BOX";
    pub const ORANGE_STAINED_GLASS: &str = "ORANGE_STAINED_GLASS";
    pub const ORANGE_STAINED_GLASS_PANE: &str = "ORANGE_STAINED_GLASS_PANE";
    pub const ORANGE_TERRACOTTA: &str = "ORANGE_TERRACOTTA";
    pub const ORANGE_TULIP: &str = "ORANGE_TULIP";
    pub const ORANGE_WALL_BANNER: &str = "ORANGE_WALL_BANNER";
    pub const ORANGE_WOOL: &str = "ORANGE_WOOL";
    pub const ORANGE_WOOL_SLAB: &str = "ORANGE_WOOL_SLAB";
    pub const ORANGE_WOOL_STAIRS: &str = "ORANGE_WOOL_STAIRS";
    pub const OXEYE_DAISY: &str = "OXEYE_DAISY";
    pub const OXIDIZED_CHISELED_COPPER: &str = "OXIDIZED_CHISELED_COPPER";
    pub const OXIDIZED_COPPER: &str = "OXIDIZED_COPPER";
    pub const OXIDIZED_COPPER_BARS: &str = "OXIDIZED_COPPER_BARS";
    pub const OXIDIZED_COPPER_BULB: &str = "OXIDIZED_COPPER_BULB";
    pub const OXIDIZED_COPPER_CHAIN: &str = "OXIDIZED_COPPER_CHAIN";
    pub const OXIDIZED_COPPER_CHEST: &str = "OXIDIZED_COPPER_CHEST";
    pub const OXIDIZED_COPPER_DOOR: &str = "OXIDIZED_COPPER_DOOR";
    pub const OXIDIZED_COPPER_GOLEM_STATUE: &str = "OXIDIZED_COPPER_GOLEM_STATUE";
    pub const OXIDIZED_COPPER_GRATE: &str = "OXIDIZED_COPPER_GRATE";
    pub const OXIDIZED_COPPER_LANTERN: &str = "OXIDIZED_COPPER_LANTERN";
    pub const OXIDIZED_COPPER_TRAPDOOR: &str = "OXIDIZED_COPPER_TRAPDOOR";
    pub const OXIDIZED_CUT_COPPER: &str = "OXIDIZED_CUT_COPPER";
    pub const OXIDIZED_CUT_COPPER_SLAB: &str = "OXIDIZED_CUT_COPPER_SLAB";
    pub const OXIDIZED_CUT_COPPER_STAIRS: &str = "OXIDIZED_CUT_COPPER_STAIRS";
    pub const OXIDIZED_LIGHTNING_ROD: &str = "OXIDIZED_LIGHTNING_ROD";
    pub const PACKED_ICE: &str = "PACKED_ICE";
    pub const PACKED_MUD: &str = "PACKED_MUD";
    pub const PALE_HANGING_MOSS: &str = "PALE_HANGING_MOSS";
    pub const PALE_MOSS_BLOCK: &str = "PALE_MOSS_BLOCK";
    pub const PALE_MOSS_CARPET: &str = "PALE_MOSS_CARPET";
    pub const PALE_OAK_BUTTON: &str = "PALE_OAK_BUTTON";
    pub const PALE_OAK_DOOR: &str = "PALE_OAK_DOOR";
    pub const PALE_OAK_FENCE: &str = "PALE_OAK_FENCE";
    pub const PALE_OAK_FENCE_GATE: &str = "PALE_OAK_FENCE_GATE";
    pub const PALE_OAK_HANGING_SIGN: &str = "PALE_OAK_HANGING_SIGN";
    pub const PALE_OAK_LEAVES: &str = "PALE_OAK_LEAVES";
    pub const PALE_OAK_LOG: &str = "PALE_OAK_LOG";
    pub const PALE_OAK_PLANKS: &str = "PALE_OAK_PLANKS";
    pub const PALE_OAK_PRESSURE_PLATE: &str = "PALE_OAK_PRESSURE_PLATE";
    pub const PALE_OAK_SAPLING: &str = "PALE_OAK_SAPLING";
    pub const PALE_OAK_SHELF: &str = "PALE_OAK_SHELF";
    pub const PALE_OAK_SIGN: &str = "PALE_OAK_SIGN";
    pub const PALE_OAK_SLAB: &str = "PALE_OAK_SLAB";
    pub const PALE_OAK_STAIRS: &str = "PALE_OAK_STAIRS";
    pub const PALE_OAK_TRAPDOOR: &str = "PALE_OAK_TRAPDOOR";
    pub const PALE_OAK_WALL_HANGING_SIGN: &str = "PALE_OAK_WALL_HANGING_SIGN";
    pub const PALE_OAK_WALL_SIGN: &str = "PALE_OAK_WALL_SIGN";
    pub const PALE_OAK_WOOD: &str = "PALE_OAK_WOOD";
    pub const PEARLESCENT_FROGLIGHT: &str = "PEARLESCENT_FROGLIGHT";
    pub const PEONY: &str = "PEONY";
    pub const PETRIFIED_OAK_SLAB: &str = "PETRIFIED_OAK_SLAB";
    pub const PIGLIN_HEAD: &str = "PIGLIN_HEAD";
    pub const PIGLIN_WALL_HEAD: &str = "PIGLIN_WALL_HEAD";
    pub const PINK_BANNER: &str = "PINK_BANNER";
    pub const PINK_BED: &str = "PINK_BED";
    pub const PINK_CANDLE: &str = "PINK_CANDLE";
    pub const PINK_CANDLE_CAKE: &str = "PINK_CANDLE_CAKE";
    pub const PINK_CARPET: &str = "PINK_CARPET";
    pub const PINK_CONCRETE: &str = "PINK_CONCRETE";
    pub const PINK_CONCRETE_POWDER: &str = "PINK_CONCRETE_POWDER";
    pub const PINK_CONCRETE_SLAB: &str = "PINK_CONCRETE_SLAB";
    pub const PINK_CONCRETE_STAIRS: &str = "PINK_CONCRETE_STAIRS";
    pub const PINK_GLAZED_TERRACOTTA: &str = "PINK_GLAZED_TERRACOTTA";
    pub const PINK_PETALS: &str = "PINK_PETALS";
    pub const PINK_SHULKER_BOX: &str = "PINK_SHULKER_BOX";
    pub const PINK_STAINED_GLASS: &str = "PINK_STAINED_GLASS";
    pub const PINK_STAINED_GLASS_PANE: &str = "PINK_STAINED_GLASS_PANE";
    pub const PINK_TERRACOTTA: &str = "PINK_TERRACOTTA";
    pub const PINK_TULIP: &str = "PINK_TULIP";
    pub const PINK_WALL_BANNER: &str = "PINK_WALL_BANNER";
    pub const PINK_WOOL: &str = "PINK_WOOL";
    pub const PINK_WOOL_SLAB: &str = "PINK_WOOL_SLAB";
    pub const PINK_WOOL_STAIRS: &str = "PINK_WOOL_STAIRS";
    pub const PISTON: &str = "PISTON";
    pub const PISTON_HEAD: &str = "PISTON_HEAD";
    pub const PITCHER_CROP: &str = "PITCHER_CROP";
    pub const PITCHER_PLANT: &str = "PITCHER_PLANT";
    pub const PLAYER_HEAD: &str = "PLAYER_HEAD";
    pub const PLAYER_WALL_HEAD: &str = "PLAYER_WALL_HEAD";
    pub const PODZOL: &str = "PODZOL";
    pub const POINTED_DRIPSTONE: &str = "POINTED_DRIPSTONE";
    pub const POLISHED_ANDESITE: &str = "POLISHED_ANDESITE";
    pub const POLISHED_ANDESITE_SLAB: &str = "POLISHED_ANDESITE_SLAB";
    pub const POLISHED_ANDESITE_STAIRS: &str = "POLISHED_ANDESITE_STAIRS";
    pub const POLISHED_BASALT: &str = "POLISHED_BASALT";
    pub const POLISHED_BLACKSTONE: &str = "POLISHED_BLACKSTONE";
    pub const POLISHED_BLACKSTONE_BRICK_SLAB: &str = "POLISHED_BLACKSTONE_BRICK_SLAB";
    pub const POLISHED_BLACKSTONE_BRICK_STAIRS: &str = "POLISHED_BLACKSTONE_BRICK_STAIRS";
    pub const POLISHED_BLACKSTONE_BRICK_WALL: &str = "POLISHED_BLACKSTONE_BRICK_WALL";
    pub const POLISHED_BLACKSTONE_BRICKS: &str = "POLISHED_BLACKSTONE_BRICKS";
    pub const POLISHED_BLACKSTONE_BUTTON: &str = "POLISHED_BLACKSTONE_BUTTON";
    pub const POLISHED_BLACKSTONE_PRESSURE_PLATE: &str = "POLISHED_BLACKSTONE_PRESSURE_PLATE";
    pub const POLISHED_BLACKSTONE_SLAB: &str = "POLISHED_BLACKSTONE_SLAB";
    pub const POLISHED_BLACKSTONE_STAIRS: &str = "POLISHED_BLACKSTONE_STAIRS";
    pub const POLISHED_BLACKSTONE_WALL: &str = "POLISHED_BLACKSTONE_WALL";
    pub const POLISHED_CINNABAR: &str = "POLISHED_CINNABAR";
    pub const POLISHED_CINNABAR_SLAB: &str = "POLISHED_CINNABAR_SLAB";
    pub const POLISHED_CINNABAR_STAIRS: &str = "POLISHED_CINNABAR_STAIRS";
    pub const POLISHED_CINNABAR_WALL: &str = "POLISHED_CINNABAR_WALL";
    pub const POLISHED_DEEPSLATE: &str = "POLISHED_DEEPSLATE";
    pub const POLISHED_DEEPSLATE_SLAB: &str = "POLISHED_DEEPSLATE_SLAB";
    pub const POLISHED_DEEPSLATE_STAIRS: &str = "POLISHED_DEEPSLATE_STAIRS";
    pub const POLISHED_DEEPSLATE_WALL: &str = "POLISHED_DEEPSLATE_WALL";
    pub const POLISHED_DIORITE: &str = "POLISHED_DIORITE";
    pub const POLISHED_DIORITE_SLAB: &str = "POLISHED_DIORITE_SLAB";
    pub const POLISHED_DIORITE_STAIRS: &str = "POLISHED_DIORITE_STAIRS";
    pub const POLISHED_GRANITE: &str = "POLISHED_GRANITE";
    pub const POLISHED_GRANITE_SLAB: &str = "POLISHED_GRANITE_SLAB";
    pub const POLISHED_GRANITE_STAIRS: &str = "POLISHED_GRANITE_STAIRS";
    pub const POLISHED_SULFUR: &str = "POLISHED_SULFUR";
    pub const POLISHED_SULFUR_SLAB: &str = "POLISHED_SULFUR_SLAB";
    pub const POLISHED_SULFUR_STAIRS: &str = "POLISHED_SULFUR_STAIRS";
    pub const POLISHED_SULFUR_WALL: &str = "POLISHED_SULFUR_WALL";
    pub const POLISHED_TUFF: &str = "POLISHED_TUFF";
    pub const POLISHED_TUFF_SLAB: &str = "POLISHED_TUFF_SLAB";
    pub const POLISHED_TUFF_STAIRS: &str = "POLISHED_TUFF_STAIRS";
    pub const POLISHED_TUFF_WALL: &str = "POLISHED_TUFF_WALL";
    pub const POPLAR_BUTTON: &str = "POPLAR_BUTTON";
    pub const POPLAR_DOOR: &str = "POPLAR_DOOR";
    pub const POPLAR_FENCE: &str = "POPLAR_FENCE";
    pub const POPLAR_FENCE_GATE: &str = "POPLAR_FENCE_GATE";
    pub const POPLAR_HANGING_SIGN: &str = "POPLAR_HANGING_SIGN";
    pub const POPLAR_LOG: &str = "POPLAR_LOG";
    pub const POPLAR_PLANKS: &str = "POPLAR_PLANKS";
    pub const POPLAR_PRESSURE_PLATE: &str = "POPLAR_PRESSURE_PLATE";
    pub const POPLAR_SAPLING: &str = "POPLAR_SAPLING";
    pub const POPLAR_SHELF: &str = "POPLAR_SHELF";
    pub const POPLAR_SIGN: &str = "POPLAR_SIGN";
    pub const POPLAR_SLAB: &str = "POPLAR_SLAB";
    pub const POPLAR_STAIRS: &str = "POPLAR_STAIRS";
    pub const POPLAR_TRAPDOOR: &str = "POPLAR_TRAPDOOR";
    pub const POPLAR_WALL_HANGING_SIGN: &str = "POPLAR_WALL_HANGING_SIGN";
    pub const POPLAR_WALL_SIGN: &str = "POPLAR_WALL_SIGN";
    pub const POPLAR_WOOD: &str = "POPLAR_WOOD";
    pub const POPPY: &str = "POPPY";
    pub const POTATOES: &str = "POTATOES";
    pub const POTENT_SULFUR: &str = "POTENT_SULFUR";
    pub const POTTED_ACACIA_SAPLING: &str = "POTTED_ACACIA_SAPLING";
    pub const POTTED_ALLIUM: &str = "POTTED_ALLIUM";
    pub const POTTED_AZALEA_BUSH: &str = "POTTED_AZALEA_BUSH";
    pub const POTTED_AZURE_BLUET: &str = "POTTED_AZURE_BLUET";
    pub const POTTED_BAMBOO: &str = "POTTED_BAMBOO";
    pub const POTTED_BIRCH_SAPLING: &str = "POTTED_BIRCH_SAPLING";
    pub const POTTED_BLUE_ORCHID: &str = "POTTED_BLUE_ORCHID";
    pub const POTTED_BROWN_MUSHROOM: &str = "POTTED_BROWN_MUSHROOM";
    pub const POTTED_CACTUS: &str = "POTTED_CACTUS";
    pub const POTTED_CHERRY_SAPLING: &str = "POTTED_CHERRY_SAPLING";
    pub const POTTED_CLOSED_EYEBLOSSOM: &str = "POTTED_CLOSED_EYEBLOSSOM";
    pub const POTTED_CORNFLOWER: &str = "POTTED_CORNFLOWER";
    pub const POTTED_CRIMSON_FUNGUS: &str = "POTTED_CRIMSON_FUNGUS";
    pub const POTTED_CRIMSON_ROOTS: &str = "POTTED_CRIMSON_ROOTS";
    pub const POTTED_DANDELION: &str = "POTTED_DANDELION";
    pub const POTTED_DARK_OAK_SAPLING: &str = "POTTED_DARK_OAK_SAPLING";
    pub const POTTED_DEAD_BUSH: &str = "POTTED_DEAD_BUSH";
    pub const POTTED_FERN: &str = "POTTED_FERN";
    pub const POTTED_FLOWERING_AZALEA_BUSH: &str = "POTTED_FLOWERING_AZALEA_BUSH";
    pub const POTTED_GOLDEN_DANDELION: &str = "POTTED_GOLDEN_DANDELION";
    pub const POTTED_JUNGLE_SAPLING: &str = "POTTED_JUNGLE_SAPLING";
    pub const POTTED_LILY_OF_THE_VALLEY: &str = "POTTED_LILY_OF_THE_VALLEY";
    pub const POTTED_MANGROVE_PROPAGULE: &str = "POTTED_MANGROVE_PROPAGULE";
    pub const POTTED_OAK_SAPLING: &str = "POTTED_OAK_SAPLING";
    pub const POTTED_OPEN_EYEBLOSSOM: &str = "POTTED_OPEN_EYEBLOSSOM";
    pub const POTTED_ORANGE_TULIP: &str = "POTTED_ORANGE_TULIP";
    pub const POTTED_OXEYE_DAISY: &str = "POTTED_OXEYE_DAISY";
    pub const POTTED_PALE_OAK_SAPLING: &str = "POTTED_PALE_OAK_SAPLING";
    pub const POTTED_PINK_TULIP: &str = "POTTED_PINK_TULIP";
    pub const POTTED_POPLAR_SAPLING: &str = "POTTED_POPLAR_SAPLING";
    pub const POTTED_POPPY: &str = "POTTED_POPPY";
    pub const POTTED_RED_MUSHROOM: &str = "POTTED_RED_MUSHROOM";
    pub const POTTED_RED_TULIP: &str = "POTTED_RED_TULIP";
    pub const POTTED_SPRUCE_SAPLING: &str = "POTTED_SPRUCE_SAPLING";
    pub const POTTED_TORCHFLOWER: &str = "POTTED_TORCHFLOWER";
    pub const POTTED_WARPED_FUNGUS: &str = "POTTED_WARPED_FUNGUS";
    pub const POTTED_WARPED_ROOTS: &str = "POTTED_WARPED_ROOTS";
    pub const POTTED_WHITE_TULIP: &str = "POTTED_WHITE_TULIP";
    pub const POTTED_WITHER_ROSE: &str = "POTTED_WITHER_ROSE";
    pub const POWDER_SNOW: &str = "POWDER_SNOW";
    pub const POWDER_SNOW_CAULDRON: &str = "POWDER_SNOW_CAULDRON";
    pub const POWERED_RAIL: &str = "POWERED_RAIL";
    pub const PRISMARINE: &str = "PRISMARINE";
    pub const PRISMARINE_BRICK_SLAB: &str = "PRISMARINE_BRICK_SLAB";
    pub const PRISMARINE_BRICK_STAIRS: &str = "PRISMARINE_BRICK_STAIRS";
    pub const PRISMARINE_BRICKS: &str = "PRISMARINE_BRICKS";
    pub const PRISMARINE_SLAB: &str = "PRISMARINE_SLAB";
    pub const PRISMARINE_STAIRS: &str = "PRISMARINE_STAIRS";
    pub const PRISMARINE_WALL: &str = "PRISMARINE_WALL";
    pub const PUMPKIN: &str = "PUMPKIN";
    pub const PUMPKIN_STEM: &str = "PUMPKIN_STEM";
    pub const PURPLE_BANNER: &str = "PURPLE_BANNER";
    pub const PURPLE_BED: &str = "PURPLE_BED";
    pub const PURPLE_CANDLE: &str = "PURPLE_CANDLE";
    pub const PURPLE_CANDLE_CAKE: &str = "PURPLE_CANDLE_CAKE";
    pub const PURPLE_CARPET: &str = "PURPLE_CARPET";
    pub const PURPLE_CONCRETE: &str = "PURPLE_CONCRETE";
    pub const PURPLE_CONCRETE_POWDER: &str = "PURPLE_CONCRETE_POWDER";
    pub const PURPLE_CONCRETE_SLAB: &str = "PURPLE_CONCRETE_SLAB";
    pub const PURPLE_CONCRETE_STAIRS: &str = "PURPLE_CONCRETE_STAIRS";
    pub const PURPLE_GLAZED_TERRACOTTA: &str = "PURPLE_GLAZED_TERRACOTTA";
    pub const PURPLE_SHULKER_BOX: &str = "PURPLE_SHULKER_BOX";
    pub const PURPLE_STAINED_GLASS: &str = "PURPLE_STAINED_GLASS";
    pub const PURPLE_STAINED_GLASS_PANE: &str = "PURPLE_STAINED_GLASS_PANE";
    pub const PURPLE_TERRACOTTA: &str = "PURPLE_TERRACOTTA";
    pub const PURPLE_WALL_BANNER: &str = "PURPLE_WALL_BANNER";
    pub const PURPLE_WOOL: &str = "PURPLE_WOOL";
    pub const PURPLE_WOOL_SLAB: &str = "PURPLE_WOOL_SLAB";
    pub const PURPLE_WOOL_STAIRS: &str = "PURPLE_WOOL_STAIRS";
    pub const PURPUR_BLOCK: &str = "PURPUR_BLOCK";
    pub const PURPUR_PILLAR: &str = "PURPUR_PILLAR";
    pub const PURPUR_SLAB: &str = "PURPUR_SLAB";
    pub const PURPUR_STAIRS: &str = "PURPUR_STAIRS";
    pub const QUARTZ_BLOCK: &str = "QUARTZ_BLOCK";
    pub const QUARTZ_BRICKS: &str = "QUARTZ_BRICKS";
    pub const QUARTZ_PILLAR: &str = "QUARTZ_PILLAR";
    pub const QUARTZ_SLAB: &str = "QUARTZ_SLAB";
    pub const QUARTZ_STAIRS: &str = "QUARTZ_STAIRS";
    pub const RAIL: &str = "RAIL";
    pub const RAW_COPPER_BLOCK: &str = "RAW_COPPER_BLOCK";
    pub const RAW_GOLD_BLOCK: &str = "RAW_GOLD_BLOCK";
    pub const RAW_IRON_BLOCK: &str = "RAW_IRON_BLOCK";
    pub const RED_BANNER: &str = "RED_BANNER";
    pub const RED_BED: &str = "RED_BED";
    pub const RED_CANDLE: &str = "RED_CANDLE";
    pub const RED_CANDLE_CAKE: &str = "RED_CANDLE_CAKE";
    pub const RED_CARPET: &str = "RED_CARPET";
    pub const RED_CONCRETE: &str = "RED_CONCRETE";
    pub const RED_CONCRETE_POWDER: &str = "RED_CONCRETE_POWDER";
    pub const RED_CONCRETE_SLAB: &str = "RED_CONCRETE_SLAB";
    pub const RED_CONCRETE_STAIRS: &str = "RED_CONCRETE_STAIRS";
    pub const RED_GLAZED_TERRACOTTA: &str = "RED_GLAZED_TERRACOTTA";
    pub const RED_MUSHROOM: &str = "RED_MUSHROOM";
    pub const RED_MUSHROOM_BLOCK: &str = "RED_MUSHROOM_BLOCK";
    pub const RED_NETHER_BRICK_SLAB: &str = "RED_NETHER_BRICK_SLAB";
    pub const RED_NETHER_BRICK_STAIRS: &str = "RED_NETHER_BRICK_STAIRS";
    pub const RED_NETHER_BRICK_WALL: &str = "RED_NETHER_BRICK_WALL";
    pub const RED_NETHER_BRICKS: &str = "RED_NETHER_BRICKS";
    pub const RED_POPLAR_LEAVES: &str = "RED_POPLAR_LEAVES";
    pub const RED_SAND: &str = "RED_SAND";
    pub const RED_SANDSTONE: &str = "RED_SANDSTONE";
    pub const RED_SANDSTONE_SLAB: &str = "RED_SANDSTONE_SLAB";
    pub const RED_SANDSTONE_STAIRS: &str = "RED_SANDSTONE_STAIRS";
    pub const RED_SANDSTONE_WALL: &str = "RED_SANDSTONE_WALL";
    pub const RED_SHRUB: &str = "RED_SHRUB";
    pub const RED_SHULKER_BOX: &str = "RED_SHULKER_BOX";
    pub const RED_STAINED_GLASS: &str = "RED_STAINED_GLASS";
    pub const RED_STAINED_GLASS_PANE: &str = "RED_STAINED_GLASS_PANE";
    pub const RED_TERRACOTTA: &str = "RED_TERRACOTTA";
    pub const RED_TULIP: &str = "RED_TULIP";
    pub const RED_WALL_BANNER: &str = "RED_WALL_BANNER";
    pub const RED_WOOL: &str = "RED_WOOL";
    pub const RED_WOOL_SLAB: &str = "RED_WOOL_SLAB";
    pub const RED_WOOL_STAIRS: &str = "RED_WOOL_STAIRS";
    pub const REDSTONE_BLOCK: &str = "REDSTONE_BLOCK";
    pub const REDSTONE_LAMP: &str = "REDSTONE_LAMP";
    pub const REDSTONE_ORE: &str = "REDSTONE_ORE";
    pub const REDSTONE_TORCH: &str = "REDSTONE_TORCH";
    pub const REDSTONE_WALL_TORCH: &str = "REDSTONE_WALL_TORCH";
    pub const REDSTONE_WIRE: &str = "REDSTONE_WIRE";
    pub const REINFORCED_DEEPSLATE: &str = "REINFORCED_DEEPSLATE";
    pub const REPEATER: &str = "REPEATER";
    pub const REPEATING_COMMAND_BLOCK: &str = "REPEATING_COMMAND_BLOCK";
    pub const RESIN_BLOCK: &str = "RESIN_BLOCK";
    pub const RESIN_BRICK_SLAB: &str = "RESIN_BRICK_SLAB";
    pub const RESIN_BRICK_STAIRS: &str = "RESIN_BRICK_STAIRS";
    pub const RESIN_BRICK_WALL: &str = "RESIN_BRICK_WALL";
    pub const RESIN_BRICKS: &str = "RESIN_BRICKS";
    pub const RESIN_CLUMP: &str = "RESIN_CLUMP";
    pub const RESPAWN_ANCHOR: &str = "RESPAWN_ANCHOR";
    pub const ROOTED_DIRT: &str = "ROOTED_DIRT";
    pub const ROSE_BUSH: &str = "ROSE_BUSH";
    pub const SAND: &str = "SAND";
    pub const SANDSTONE: &str = "SANDSTONE";
    pub const SANDSTONE_SLAB: &str = "SANDSTONE_SLAB";
    pub const SANDSTONE_STAIRS: &str = "SANDSTONE_STAIRS";
    pub const SANDSTONE_WALL: &str = "SANDSTONE_WALL";
    pub const SCAFFOLDING: &str = "SCAFFOLDING";
    pub const SCULK: &str = "SCULK";
    pub const SCULK_CATALYST: &str = "SCULK_CATALYST";
    pub const SCULK_SENSOR: &str = "SCULK_SENSOR";
    pub const SCULK_SHRIEKER: &str = "SCULK_SHRIEKER";
    pub const SCULK_VEIN: &str = "SCULK_VEIN";
    pub const SEA_LANTERN: &str = "SEA_LANTERN";
    pub const SEA_PICKLE: &str = "SEA_PICKLE";
    pub const SEAGRASS: &str = "SEAGRASS";
    pub const SHELF_MUSHROOM: &str = "SHELF_MUSHROOM";
    pub const SHORT_DRY_GRASS: &str = "SHORT_DRY_GRASS";
    pub const SHORT_GRASS: &str = "SHORT_GRASS";
    pub const SHROOMLIGHT: &str = "SHROOMLIGHT";
    pub const SHULKER_BOX: &str = "SHULKER_BOX";
    pub const SKELETON_SKULL: &str = "SKELETON_SKULL";
    pub const SKELETON_WALL_SKULL: &str = "SKELETON_WALL_SKULL";
    pub const SLIME_BLOCK: &str = "SLIME_BLOCK";
    pub const SMALL_AMETHYST_BUD: &str = "SMALL_AMETHYST_BUD";
    pub const SMALL_DRIPLEAF: &str = "SMALL_DRIPLEAF";
    pub const SMITHING_TABLE: &str = "SMITHING_TABLE";
    pub const SMOKER: &str = "SMOKER";
    pub const SMOOTH_BASALT: &str = "SMOOTH_BASALT";
    pub const SMOOTH_QUARTZ: &str = "SMOOTH_QUARTZ";
    pub const SMOOTH_QUARTZ_SLAB: &str = "SMOOTH_QUARTZ_SLAB";
    pub const SMOOTH_QUARTZ_STAIRS: &str = "SMOOTH_QUARTZ_STAIRS";
    pub const SMOOTH_RED_SANDSTONE: &str = "SMOOTH_RED_SANDSTONE";
    pub const SMOOTH_RED_SANDSTONE_SLAB: &str = "SMOOTH_RED_SANDSTONE_SLAB";
    pub const SMOOTH_RED_SANDSTONE_STAIRS: &str = "SMOOTH_RED_SANDSTONE_STAIRS";
    pub const SMOOTH_SANDSTONE: &str = "SMOOTH_SANDSTONE";
    pub const SMOOTH_SANDSTONE_SLAB: &str = "SMOOTH_SANDSTONE_SLAB";
    pub const SMOOTH_SANDSTONE_STAIRS: &str = "SMOOTH_SANDSTONE_STAIRS";
    pub const SMOOTH_STONE: &str = "SMOOTH_STONE";
    pub const SMOOTH_STONE_SLAB: &str = "SMOOTH_STONE_SLAB";
    pub const SNIFFER_EGG: &str = "SNIFFER_EGG";
    pub const SNOW: &str = "SNOW";
    pub const SNOW_BLOCK: &str = "SNOW_BLOCK";
    pub const SOUL_CAMPFIRE: &str = "SOUL_CAMPFIRE";
    pub const SOUL_FIRE: &str = "SOUL_FIRE";
    pub const SOUL_LANTERN: &str = "SOUL_LANTERN";
    pub const SOUL_SAND: &str = "SOUL_SAND";
    pub const SOUL_SOIL: &str = "SOUL_SOIL";
    pub const SOUL_TORCH: &str = "SOUL_TORCH";
    pub const SOUL_WALL_TORCH: &str = "SOUL_WALL_TORCH";
    pub const SPAWNER: &str = "SPAWNER";
    pub const SPONGE: &str = "SPONGE";
    pub const SPORE_BLOSSOM: &str = "SPORE_BLOSSOM";
    pub const SPRUCE_BUTTON: &str = "SPRUCE_BUTTON";
    pub const SPRUCE_DOOR: &str = "SPRUCE_DOOR";
    pub const SPRUCE_FENCE: &str = "SPRUCE_FENCE";
    pub const SPRUCE_FENCE_GATE: &str = "SPRUCE_FENCE_GATE";
    pub const SPRUCE_HANGING_SIGN: &str = "SPRUCE_HANGING_SIGN";
    pub const SPRUCE_LEAVES: &str = "SPRUCE_LEAVES";
    pub const SPRUCE_LOG: &str = "SPRUCE_LOG";
    pub const SPRUCE_PLANKS: &str = "SPRUCE_PLANKS";
    pub const SPRUCE_PRESSURE_PLATE: &str = "SPRUCE_PRESSURE_PLATE";
    pub const SPRUCE_SAPLING: &str = "SPRUCE_SAPLING";
    pub const SPRUCE_SHELF: &str = "SPRUCE_SHELF";
    pub const SPRUCE_SIGN: &str = "SPRUCE_SIGN";
    pub const SPRUCE_SLAB: &str = "SPRUCE_SLAB";
    pub const SPRUCE_STAIRS: &str = "SPRUCE_STAIRS";
    pub const SPRUCE_TRAPDOOR: &str = "SPRUCE_TRAPDOOR";
    pub const SPRUCE_WALL_HANGING_SIGN: &str = "SPRUCE_WALL_HANGING_SIGN";
    pub const SPRUCE_WALL_SIGN: &str = "SPRUCE_WALL_SIGN";
    pub const SPRUCE_WOOD: &str = "SPRUCE_WOOD";
    pub const STICKY_PISTON: &str = "STICKY_PISTON";
    pub const STONE: &str = "STONE";
    pub const STONE_BRICK_SLAB: &str = "STONE_BRICK_SLAB";
    pub const STONE_BRICK_STAIRS: &str = "STONE_BRICK_STAIRS";
    pub const STONE_BRICK_WALL: &str = "STONE_BRICK_WALL";
    pub const STONE_BRICKS: &str = "STONE_BRICKS";
    pub const STONE_BUTTON: &str = "STONE_BUTTON";
    pub const STONE_PRESSURE_PLATE: &str = "STONE_PRESSURE_PLATE";
    pub const STONE_SLAB: &str = "STONE_SLAB";
    pub const STONE_STAIRS: &str = "STONE_STAIRS";
    pub const STONECUTTER: &str = "STONECUTTER";
    pub const STRAW_BED: &str = "STRAW_BED";
    pub const STRIPPED_ACACIA_LOG: &str = "STRIPPED_ACACIA_LOG";
    pub const STRIPPED_ACACIA_WOOD: &str = "STRIPPED_ACACIA_WOOD";
    pub const STRIPPED_BAMBOO_BLOCK: &str = "STRIPPED_BAMBOO_BLOCK";
    pub const STRIPPED_BIRCH_LOG: &str = "STRIPPED_BIRCH_LOG";
    pub const STRIPPED_BIRCH_WOOD: &str = "STRIPPED_BIRCH_WOOD";
    pub const STRIPPED_CHERRY_LOG: &str = "STRIPPED_CHERRY_LOG";
    pub const STRIPPED_CHERRY_WOOD: &str = "STRIPPED_CHERRY_WOOD";
    pub const STRIPPED_CRIMSON_HYPHAE: &str = "STRIPPED_CRIMSON_HYPHAE";
    pub const STRIPPED_CRIMSON_STEM: &str = "STRIPPED_CRIMSON_STEM";
    pub const STRIPPED_DARK_OAK_LOG: &str = "STRIPPED_DARK_OAK_LOG";
    pub const STRIPPED_DARK_OAK_WOOD: &str = "STRIPPED_DARK_OAK_WOOD";
    pub const STRIPPED_JUNGLE_LOG: &str = "STRIPPED_JUNGLE_LOG";
    pub const STRIPPED_JUNGLE_WOOD: &str = "STRIPPED_JUNGLE_WOOD";
    pub const STRIPPED_MANGROVE_LOG: &str = "STRIPPED_MANGROVE_LOG";
    pub const STRIPPED_MANGROVE_WOOD: &str = "STRIPPED_MANGROVE_WOOD";
    pub const STRIPPED_OAK_LOG: &str = "STRIPPED_OAK_LOG";
    pub const STRIPPED_OAK_WOOD: &str = "STRIPPED_OAK_WOOD";
    pub const STRIPPED_PALE_OAK_LOG: &str = "STRIPPED_PALE_OAK_LOG";
    pub const STRIPPED_PALE_OAK_WOOD: &str = "STRIPPED_PALE_OAK_WOOD";
    pub const STRIPPED_POPLAR_LOG: &str = "STRIPPED_POPLAR_LOG";
    pub const STRIPPED_POPLAR_WOOD: &str = "STRIPPED_POPLAR_WOOD";
    pub const STRIPPED_SPRUCE_LOG: &str = "STRIPPED_SPRUCE_LOG";
    pub const STRIPPED_SPRUCE_WOOD: &str = "STRIPPED_SPRUCE_WOOD";
    pub const STRIPPED_WARPED_HYPHAE: &str = "STRIPPED_WARPED_HYPHAE";
    pub const STRIPPED_WARPED_STEM: &str = "STRIPPED_WARPED_STEM";
    pub const STRUCTURE_BLOCK: &str = "STRUCTURE_BLOCK";
    pub const STRUCTURE_VOID: &str = "STRUCTURE_VOID";
    pub const SUGAR_CANE: &str = "SUGAR_CANE";
    pub const SULFUR: &str = "SULFUR";
    pub const SULFUR_BRICK_SLAB: &str = "SULFUR_BRICK_SLAB";
    pub const SULFUR_BRICK_STAIRS: &str = "SULFUR_BRICK_STAIRS";
    pub const SULFUR_BRICK_WALL: &str = "SULFUR_BRICK_WALL";
    pub const SULFUR_BRICKS: &str = "SULFUR_BRICKS";
    pub const SULFUR_SLAB: &str = "SULFUR_SLAB";
    pub const SULFUR_SPIKE: &str = "SULFUR_SPIKE";
    pub const SULFUR_STAIRS: &str = "SULFUR_STAIRS";
    pub const SULFUR_WALL: &str = "SULFUR_WALL";
    pub const SUNFLOWER: &str = "SUNFLOWER";
    pub const SUSPICIOUS_GRAVEL: &str = "SUSPICIOUS_GRAVEL";
    pub const SUSPICIOUS_SAND: &str = "SUSPICIOUS_SAND";
    pub const SWEET_BERRY_BUSH: &str = "SWEET_BERRY_BUSH";
    pub const TALL_DRY_GRASS: &str = "TALL_DRY_GRASS";
    pub const TALL_GRASS: &str = "TALL_GRASS";
    pub const TALL_SEAGRASS: &str = "TALL_SEAGRASS";
    pub const TARGET: &str = "TARGET";
    pub const TERRACOTTA: &str = "TERRACOTTA";
    pub const TEST_BLOCK: &str = "TEST_BLOCK";
    pub const TEST_INSTANCE_BLOCK: &str = "TEST_INSTANCE_BLOCK";
    pub const TINTED_GLASS: &str = "TINTED_GLASS";
    pub const TNT: &str = "TNT";
    pub const TORCH: &str = "TORCH";
    pub const TORCHFLOWER: &str = "TORCHFLOWER";
    pub const TORCHFLOWER_CROP: &str = "TORCHFLOWER_CROP";
    pub const TRAPPED_CHEST: &str = "TRAPPED_CHEST";
    pub const TRIAL_SPAWNER: &str = "TRIAL_SPAWNER";
    pub const TRIPWIRE: &str = "TRIPWIRE";
    pub const TRIPWIRE_HOOK: &str = "TRIPWIRE_HOOK";
    pub const TUBE_CORAL: &str = "TUBE_CORAL";
    pub const TUBE_CORAL_BLOCK: &str = "TUBE_CORAL_BLOCK";
    pub const TUBE_CORAL_FAN: &str = "TUBE_CORAL_FAN";
    pub const TUBE_CORAL_WALL_FAN: &str = "TUBE_CORAL_WALL_FAN";
    pub const TUFF: &str = "TUFF";
    pub const TUFF_BRICK_SLAB: &str = "TUFF_BRICK_SLAB";
    pub const TUFF_BRICK_STAIRS: &str = "TUFF_BRICK_STAIRS";
    pub const TUFF_BRICK_WALL: &str = "TUFF_BRICK_WALL";
    pub const TUFF_BRICKS: &str = "TUFF_BRICKS";
    pub const TUFF_SLAB: &str = "TUFF_SLAB";
    pub const TUFF_STAIRS: &str = "TUFF_STAIRS";
    pub const TUFF_WALL: &str = "TUFF_WALL";
    pub const TURTLE_EGG: &str = "TURTLE_EGG";
    pub const TWISTING_VINES: &str = "TWISTING_VINES";
    pub const TWISTING_VINES_PLANT: &str = "TWISTING_VINES_PLANT";
    pub const VAULT: &str = "VAULT";
    pub const VERDANT_FROGLIGHT: &str = "VERDANT_FROGLIGHT";
    pub const VINE: &str = "VINE";
    pub const VOID_AIR: &str = "VOID_AIR";
    pub const WALL_TORCH: &str = "WALL_TORCH";
    pub const WARPED_BUTTON: &str = "WARPED_BUTTON";
    pub const WARPED_DOOR: &str = "WARPED_DOOR";
    pub const WARPED_FENCE: &str = "WARPED_FENCE";
    pub const WARPED_FENCE_GATE: &str = "WARPED_FENCE_GATE";
    pub const WARPED_FUNGUS: &str = "WARPED_FUNGUS";
    pub const WARPED_HANGING_SIGN: &str = "WARPED_HANGING_SIGN";
    pub const WARPED_HYPHAE: &str = "WARPED_HYPHAE";
    pub const WARPED_NYLIUM: &str = "WARPED_NYLIUM";
    pub const WARPED_PLANKS: &str = "WARPED_PLANKS";
    pub const WARPED_PRESSURE_PLATE: &str = "WARPED_PRESSURE_PLATE";
    pub const WARPED_ROOTS: &str = "WARPED_ROOTS";
    pub const WARPED_SHELF: &str = "WARPED_SHELF";
    pub const WARPED_SIGN: &str = "WARPED_SIGN";
    pub const WARPED_SLAB: &str = "WARPED_SLAB";
    pub const WARPED_STAIRS: &str = "WARPED_STAIRS";
    pub const WARPED_STEM: &str = "WARPED_STEM";
    pub const WARPED_TRAPDOOR: &str = "WARPED_TRAPDOOR";
    pub const WARPED_WALL_HANGING_SIGN: &str = "WARPED_WALL_HANGING_SIGN";
    pub const WARPED_WALL_SIGN: &str = "WARPED_WALL_SIGN";
    pub const WARPED_WART_BLOCK: &str = "WARPED_WART_BLOCK";
    pub const WATER: &str = "WATER";
    pub const WATER_CAULDRON: &str = "WATER_CAULDRON";
    pub const WAXED_CHISELED_COPPER: &str = "WAXED_CHISELED_COPPER";
    pub const WAXED_COPPER_BARS: &str = "WAXED_COPPER_BARS";
    pub const WAXED_COPPER_BLOCK: &str = "WAXED_COPPER_BLOCK";
    pub const WAXED_COPPER_BULB: &str = "WAXED_COPPER_BULB";
    pub const WAXED_COPPER_CHAIN: &str = "WAXED_COPPER_CHAIN";
    pub const WAXED_COPPER_CHEST: &str = "WAXED_COPPER_CHEST";
    pub const WAXED_COPPER_DOOR: &str = "WAXED_COPPER_DOOR";
    pub const WAXED_COPPER_GOLEM_STATUE: &str = "WAXED_COPPER_GOLEM_STATUE";
    pub const WAXED_COPPER_GRATE: &str = "WAXED_COPPER_GRATE";
    pub const WAXED_COPPER_LANTERN: &str = "WAXED_COPPER_LANTERN";
    pub const WAXED_COPPER_TRAPDOOR: &str = "WAXED_COPPER_TRAPDOOR";
    pub const WAXED_CUT_COPPER: &str = "WAXED_CUT_COPPER";
    pub const WAXED_CUT_COPPER_SLAB: &str = "WAXED_CUT_COPPER_SLAB";
    pub const WAXED_CUT_COPPER_STAIRS: &str = "WAXED_CUT_COPPER_STAIRS";
    pub const WAXED_EXPOSED_CHISELED_COPPER: &str = "WAXED_EXPOSED_CHISELED_COPPER";
    pub const WAXED_EXPOSED_COPPER: &str = "WAXED_EXPOSED_COPPER";
    pub const WAXED_EXPOSED_COPPER_BARS: &str = "WAXED_EXPOSED_COPPER_BARS";
    pub const WAXED_EXPOSED_COPPER_BULB: &str = "WAXED_EXPOSED_COPPER_BULB";
    pub const WAXED_EXPOSED_COPPER_CHAIN: &str = "WAXED_EXPOSED_COPPER_CHAIN";
    pub const WAXED_EXPOSED_COPPER_CHEST: &str = "WAXED_EXPOSED_COPPER_CHEST";
    pub const WAXED_EXPOSED_COPPER_DOOR: &str = "WAXED_EXPOSED_COPPER_DOOR";
    pub const WAXED_EXPOSED_COPPER_GOLEM_STATUE: &str = "WAXED_EXPOSED_COPPER_GOLEM_STATUE";
    pub const WAXED_EXPOSED_COPPER_GRATE: &str = "WAXED_EXPOSED_COPPER_GRATE";
    pub const WAXED_EXPOSED_COPPER_LANTERN: &str = "WAXED_EXPOSED_COPPER_LANTERN";
    pub const WAXED_EXPOSED_COPPER_TRAPDOOR: &str = "WAXED_EXPOSED_COPPER_TRAPDOOR";
    pub const WAXED_EXPOSED_CUT_COPPER: &str = "WAXED_EXPOSED_CUT_COPPER";
    pub const WAXED_EXPOSED_CUT_COPPER_SLAB: &str = "WAXED_EXPOSED_CUT_COPPER_SLAB";
    pub const WAXED_EXPOSED_CUT_COPPER_STAIRS: &str = "WAXED_EXPOSED_CUT_COPPER_STAIRS";
    pub const WAXED_EXPOSED_LIGHTNING_ROD: &str = "WAXED_EXPOSED_LIGHTNING_ROD";
    pub const WAXED_LIGHTNING_ROD: &str = "WAXED_LIGHTNING_ROD";
    pub const WAXED_OXIDIZED_CHISELED_COPPER: &str = "WAXED_OXIDIZED_CHISELED_COPPER";
    pub const WAXED_OXIDIZED_COPPER: &str = "WAXED_OXIDIZED_COPPER";
    pub const WAXED_OXIDIZED_COPPER_BARS: &str = "WAXED_OXIDIZED_COPPER_BARS";
    pub const WAXED_OXIDIZED_COPPER_BULB: &str = "WAXED_OXIDIZED_COPPER_BULB";
    pub const WAXED_OXIDIZED_COPPER_CHAIN: &str = "WAXED_OXIDIZED_COPPER_CHAIN";
    pub const WAXED_OXIDIZED_COPPER_CHEST: &str = "WAXED_OXIDIZED_COPPER_CHEST";
    pub const WAXED_OXIDIZED_COPPER_DOOR: &str = "WAXED_OXIDIZED_COPPER_DOOR";
    pub const WAXED_OXIDIZED_COPPER_GOLEM_STATUE: &str = "WAXED_OXIDIZED_COPPER_GOLEM_STATUE";
    pub const WAXED_OXIDIZED_COPPER_GRATE: &str = "WAXED_OXIDIZED_COPPER_GRATE";
    pub const WAXED_OXIDIZED_COPPER_LANTERN: &str = "WAXED_OXIDIZED_COPPER_LANTERN";
    pub const WAXED_OXIDIZED_COPPER_TRAPDOOR: &str = "WAXED_OXIDIZED_COPPER_TRAPDOOR";
    pub const WAXED_OXIDIZED_CUT_COPPER: &str = "WAXED_OXIDIZED_CUT_COPPER";
    pub const WAXED_OXIDIZED_CUT_COPPER_SLAB: &str = "WAXED_OXIDIZED_CUT_COPPER_SLAB";
    pub const WAXED_OXIDIZED_CUT_COPPER_STAIRS: &str = "WAXED_OXIDIZED_CUT_COPPER_STAIRS";
    pub const WAXED_OXIDIZED_LIGHTNING_ROD: &str = "WAXED_OXIDIZED_LIGHTNING_ROD";
    pub const WAXED_WEATHERED_CHISELED_COPPER: &str = "WAXED_WEATHERED_CHISELED_COPPER";
    pub const WAXED_WEATHERED_COPPER: &str = "WAXED_WEATHERED_COPPER";
    pub const WAXED_WEATHERED_COPPER_BARS: &str = "WAXED_WEATHERED_COPPER_BARS";
    pub const WAXED_WEATHERED_COPPER_BULB: &str = "WAXED_WEATHERED_COPPER_BULB";
    pub const WAXED_WEATHERED_COPPER_CHAIN: &str = "WAXED_WEATHERED_COPPER_CHAIN";
    pub const WAXED_WEATHERED_COPPER_CHEST: &str = "WAXED_WEATHERED_COPPER_CHEST";
    pub const WAXED_WEATHERED_COPPER_DOOR: &str = "WAXED_WEATHERED_COPPER_DOOR";
    pub const WAXED_WEATHERED_COPPER_GOLEM_STATUE: &str = "WAXED_WEATHERED_COPPER_GOLEM_STATUE";
    pub const WAXED_WEATHERED_COPPER_GRATE: &str = "WAXED_WEATHERED_COPPER_GRATE";
    pub const WAXED_WEATHERED_COPPER_LANTERN: &str = "WAXED_WEATHERED_COPPER_LANTERN";
    pub const WAXED_WEATHERED_COPPER_TRAPDOOR: &str = "WAXED_WEATHERED_COPPER_TRAPDOOR";
    pub const WAXED_WEATHERED_CUT_COPPER: &str = "WAXED_WEATHERED_CUT_COPPER";
    pub const WAXED_WEATHERED_CUT_COPPER_SLAB: &str = "WAXED_WEATHERED_CUT_COPPER_SLAB";
    pub const WAXED_WEATHERED_CUT_COPPER_STAIRS: &str = "WAXED_WEATHERED_CUT_COPPER_STAIRS";
    pub const WAXED_WEATHERED_LIGHTNING_ROD: &str = "WAXED_WEATHERED_LIGHTNING_ROD";
    pub const WEATHERED_CHISELED_COPPER: &str = "WEATHERED_CHISELED_COPPER";
    pub const WEATHERED_COPPER: &str = "WEATHERED_COPPER";
    pub const WEATHERED_COPPER_BARS: &str = "WEATHERED_COPPER_BARS";
    pub const WEATHERED_COPPER_BULB: &str = "WEATHERED_COPPER_BULB";
    pub const WEATHERED_COPPER_CHAIN: &str = "WEATHERED_COPPER_CHAIN";
    pub const WEATHERED_COPPER_CHEST: &str = "WEATHERED_COPPER_CHEST";
    pub const WEATHERED_COPPER_DOOR: &str = "WEATHERED_COPPER_DOOR";
    pub const WEATHERED_COPPER_GOLEM_STATUE: &str = "WEATHERED_COPPER_GOLEM_STATUE";
    pub const WEATHERED_COPPER_GRATE: &str = "WEATHERED_COPPER_GRATE";
    pub const WEATHERED_COPPER_LANTERN: &str = "WEATHERED_COPPER_LANTERN";
    pub const WEATHERED_COPPER_TRAPDOOR: &str = "WEATHERED_COPPER_TRAPDOOR";
    pub const WEATHERED_CUT_COPPER: &str = "WEATHERED_CUT_COPPER";
    pub const WEATHERED_CUT_COPPER_SLAB: &str = "WEATHERED_CUT_COPPER_SLAB";
    pub const WEATHERED_CUT_COPPER_STAIRS: &str = "WEATHERED_CUT_COPPER_STAIRS";
    pub const WEATHERED_LIGHTNING_ROD: &str = "WEATHERED_LIGHTNING_ROD";
    pub const WEEPING_VINES: &str = "WEEPING_VINES";
    pub const WEEPING_VINES_PLANT: &str = "WEEPING_VINES_PLANT";
    pub const WET_SPONGE: &str = "WET_SPONGE";
    pub const WHEAT: &str = "WHEAT";
    pub const WHITE_BANNER: &str = "WHITE_BANNER";
    pub const WHITE_BED: &str = "WHITE_BED";
    pub const WHITE_CANDLE: &str = "WHITE_CANDLE";
    pub const WHITE_CANDLE_CAKE: &str = "WHITE_CANDLE_CAKE";
    pub const WHITE_CARPET: &str = "WHITE_CARPET";
    pub const WHITE_CONCRETE: &str = "WHITE_CONCRETE";
    pub const WHITE_CONCRETE_POWDER: &str = "WHITE_CONCRETE_POWDER";
    pub const WHITE_CONCRETE_SLAB: &str = "WHITE_CONCRETE_SLAB";
    pub const WHITE_CONCRETE_STAIRS: &str = "WHITE_CONCRETE_STAIRS";
    pub const WHITE_GLAZED_TERRACOTTA: &str = "WHITE_GLAZED_TERRACOTTA";
    pub const WHITE_SHULKER_BOX: &str = "WHITE_SHULKER_BOX";
    pub const WHITE_STAINED_GLASS: &str = "WHITE_STAINED_GLASS";
    pub const WHITE_STAINED_GLASS_PANE: &str = "WHITE_STAINED_GLASS_PANE";
    pub const WHITE_TERRACOTTA: &str = "WHITE_TERRACOTTA";
    pub const WHITE_TULIP: &str = "WHITE_TULIP";
    pub const WHITE_WALL_BANNER: &str = "WHITE_WALL_BANNER";
    pub const WHITE_WOOL: &str = "WHITE_WOOL";
    pub const WHITE_WOOL_SLAB: &str = "WHITE_WOOL_SLAB";
    pub const WHITE_WOOL_STAIRS: &str = "WHITE_WOOL_STAIRS";
    pub const WILDFLOWERS: &str = "WILDFLOWERS";
    pub const WITHER_ROSE: &str = "WITHER_ROSE";
    pub const WITHER_SKELETON_SKULL: &str = "WITHER_SKELETON_SKULL";
    pub const WITHER_SKELETON_WALL_SKULL: &str = "WITHER_SKELETON_WALL_SKULL";
    pub const YELLOW_BANNER: &str = "YELLOW_BANNER";
    pub const YELLOW_BED: &str = "YELLOW_BED";
    pub const YELLOW_CANDLE: &str = "YELLOW_CANDLE";
    pub const YELLOW_CANDLE_CAKE: &str = "YELLOW_CANDLE_CAKE";
    pub const YELLOW_CARPET: &str = "YELLOW_CARPET";
    pub const YELLOW_CONCRETE: &str = "YELLOW_CONCRETE";
    pub const YELLOW_CONCRETE_POWDER: &str = "YELLOW_CONCRETE_POWDER";
    pub const YELLOW_CONCRETE_SLAB: &str = "YELLOW_CONCRETE_SLAB";
    pub const YELLOW_CONCRETE_STAIRS: &str = "YELLOW_CONCRETE_STAIRS";
    pub const YELLOW_GLAZED_TERRACOTTA: &str = "YELLOW_GLAZED_TERRACOTTA";
    pub const YELLOW_POPLAR_LEAVES: &str = "YELLOW_POPLAR_LEAVES";
    pub const YELLOW_SHULKER_BOX: &str = "YELLOW_SHULKER_BOX";
    pub const YELLOW_STAINED_GLASS: &str = "YELLOW_STAINED_GLASS";
    pub const YELLOW_STAINED_GLASS_PANE: &str = "YELLOW_STAINED_GLASS_PANE";
    pub const YELLOW_TERRACOTTA: &str = "YELLOW_TERRACOTTA";
    pub const YELLOW_WALL_BANNER: &str = "YELLOW_WALL_BANNER";
    pub const YELLOW_WOOL: &str = "YELLOW_WOOL";
    pub const YELLOW_WOOL_SLAB: &str = "YELLOW_WOOL_SLAB";
    pub const YELLOW_WOOL_STAIRS: &str = "YELLOW_WOOL_STAIRS";
    pub const ZOMBIE_HEAD: &str = "ZOMBIE_HEAD";
    pub const ZOMBIE_WALL_HEAD: &str = "ZOMBIE_WALL_HEAD";
    pub const LEGACY_AIR: &str = "LEGACY_AIR";
    pub const LEGACY_STONE: &str = "LEGACY_STONE";
    pub const LEGACY_GRASS: &str = "LEGACY_GRASS";
    pub const LEGACY_DIRT: &str = "LEGACY_DIRT";
    pub const LEGACY_COBBLESTONE: &str = "LEGACY_COBBLESTONE";
    pub const LEGACY_WOOD: &str = "LEGACY_WOOD";
    pub const LEGACY_SAPLING: &str = "LEGACY_SAPLING";
    pub const LEGACY_BEDROCK: &str = "LEGACY_BEDROCK";
    pub const LEGACY_WATER: &str = "LEGACY_WATER";
    pub const LEGACY_STATIONARY_WATER: &str = "LEGACY_STATIONARY_WATER";
    pub const LEGACY_LAVA: &str = "LEGACY_LAVA";
    pub const LEGACY_STATIONARY_LAVA: &str = "LEGACY_STATIONARY_LAVA";
    pub const LEGACY_SAND: &str = "LEGACY_SAND";
    pub const LEGACY_GRAVEL: &str = "LEGACY_GRAVEL";
    pub const LEGACY_GOLD_ORE: &str = "LEGACY_GOLD_ORE";
    pub const LEGACY_IRON_ORE: &str = "LEGACY_IRON_ORE";
    pub const LEGACY_COAL_ORE: &str = "LEGACY_COAL_ORE";
    pub const LEGACY_LOG: &str = "LEGACY_LOG";
    pub const LEGACY_LEAVES: &str = "LEGACY_LEAVES";
    pub const LEGACY_SPONGE: &str = "LEGACY_SPONGE";
    pub const LEGACY_GLASS: &str = "LEGACY_GLASS";
    pub const LEGACY_LAPIS_ORE: &str = "LEGACY_LAPIS_ORE";
    pub const LEGACY_LAPIS_BLOCK: &str = "LEGACY_LAPIS_BLOCK";
    pub const LEGACY_DISPENSER: &str = "LEGACY_DISPENSER";
    pub const LEGACY_SANDSTONE: &str = "LEGACY_SANDSTONE";
    pub const LEGACY_NOTE_BLOCK: &str = "LEGACY_NOTE_BLOCK";
    pub const LEGACY_BED_BLOCK: &str = "LEGACY_BED_BLOCK";
    pub const LEGACY_POWERED_RAIL: &str = "LEGACY_POWERED_RAIL";
    pub const LEGACY_DETECTOR_RAIL: &str = "LEGACY_DETECTOR_RAIL";
    pub const LEGACY_PISTON_STICKY_BASE: &str = "LEGACY_PISTON_STICKY_BASE";
    pub const LEGACY_WEB: &str = "LEGACY_WEB";
    pub const LEGACY_LONG_GRASS: &str = "LEGACY_LONG_GRASS";
    pub const LEGACY_DEAD_BUSH: &str = "LEGACY_DEAD_BUSH";
    pub const LEGACY_PISTON_BASE: &str = "LEGACY_PISTON_BASE";
    pub const LEGACY_PISTON_EXTENSION: &str = "LEGACY_PISTON_EXTENSION";
    pub const LEGACY_WOOL: &str = "LEGACY_WOOL";
    pub const LEGACY_PISTON_MOVING_PIECE: &str = "LEGACY_PISTON_MOVING_PIECE";
    pub const LEGACY_YELLOW_FLOWER: &str = "LEGACY_YELLOW_FLOWER";
    pub const LEGACY_RED_ROSE: &str = "LEGACY_RED_ROSE";
    pub const LEGACY_BROWN_MUSHROOM: &str = "LEGACY_BROWN_MUSHROOM";
    pub const LEGACY_RED_MUSHROOM: &str = "LEGACY_RED_MUSHROOM";
    pub const LEGACY_GOLD_BLOCK: &str = "LEGACY_GOLD_BLOCK";
    pub const LEGACY_IRON_BLOCK: &str = "LEGACY_IRON_BLOCK";
    pub const LEGACY_DOUBLE_STEP: &str = "LEGACY_DOUBLE_STEP";
    pub const LEGACY_STEP: &str = "LEGACY_STEP";
    pub const LEGACY_BRICK: &str = "LEGACY_BRICK";
    pub const LEGACY_TNT: &str = "LEGACY_TNT";
    pub const LEGACY_BOOKSHELF: &str = "LEGACY_BOOKSHELF";
    pub const LEGACY_MOSSY_COBBLESTONE: &str = "LEGACY_MOSSY_COBBLESTONE";
    pub const LEGACY_OBSIDIAN: &str = "LEGACY_OBSIDIAN";
    pub const LEGACY_TORCH: &str = "LEGACY_TORCH";
    pub const LEGACY_FIRE: &str = "LEGACY_FIRE";
    pub const LEGACY_MOB_SPAWNER: &str = "LEGACY_MOB_SPAWNER";
    pub const LEGACY_WOOD_STAIRS: &str = "LEGACY_WOOD_STAIRS";
    pub const LEGACY_CHEST: &str = "LEGACY_CHEST";
    pub const LEGACY_REDSTONE_WIRE: &str = "LEGACY_REDSTONE_WIRE";
    pub const LEGACY_DIAMOND_ORE: &str = "LEGACY_DIAMOND_ORE";
    pub const LEGACY_DIAMOND_BLOCK: &str = "LEGACY_DIAMOND_BLOCK";
    pub const LEGACY_WORKBENCH: &str = "LEGACY_WORKBENCH";
    pub const LEGACY_CROPS: &str = "LEGACY_CROPS";
    pub const LEGACY_SOIL: &str = "LEGACY_SOIL";
    pub const LEGACY_FURNACE: &str = "LEGACY_FURNACE";
    pub const LEGACY_BURNING_FURNACE: &str = "LEGACY_BURNING_FURNACE";
    pub const LEGACY_SIGN_POST: &str = "LEGACY_SIGN_POST";
    pub const LEGACY_WOODEN_DOOR: &str = "LEGACY_WOODEN_DOOR";
    pub const LEGACY_LADDER: &str = "LEGACY_LADDER";
    pub const LEGACY_RAILS: &str = "LEGACY_RAILS";
    pub const LEGACY_COBBLESTONE_STAIRS: &str = "LEGACY_COBBLESTONE_STAIRS";
    pub const LEGACY_WALL_SIGN: &str = "LEGACY_WALL_SIGN";
    pub const LEGACY_LEVER: &str = "LEGACY_LEVER";
    pub const LEGACY_STONE_PLATE: &str = "LEGACY_STONE_PLATE";
    pub const LEGACY_IRON_DOOR_BLOCK: &str = "LEGACY_IRON_DOOR_BLOCK";
    pub const LEGACY_WOOD_PLATE: &str = "LEGACY_WOOD_PLATE";
    pub const LEGACY_REDSTONE_ORE: &str = "LEGACY_REDSTONE_ORE";
    pub const LEGACY_GLOWING_REDSTONE_ORE: &str = "LEGACY_GLOWING_REDSTONE_ORE";
    pub const LEGACY_REDSTONE_TORCH_OFF: &str = "LEGACY_REDSTONE_TORCH_OFF";
    pub const LEGACY_REDSTONE_TORCH_ON: &str = "LEGACY_REDSTONE_TORCH_ON";
    pub const LEGACY_STONE_BUTTON: &str = "LEGACY_STONE_BUTTON";
    pub const LEGACY_SNOW: &str = "LEGACY_SNOW";
    pub const LEGACY_ICE: &str = "LEGACY_ICE";
    pub const LEGACY_SNOW_BLOCK: &str = "LEGACY_SNOW_BLOCK";
    pub const LEGACY_CACTUS: &str = "LEGACY_CACTUS";
    pub const LEGACY_CLAY: &str = "LEGACY_CLAY";
    pub const LEGACY_SUGAR_CANE_BLOCK: &str = "LEGACY_SUGAR_CANE_BLOCK";
    pub const LEGACY_JUKEBOX: &str = "LEGACY_JUKEBOX";
    pub const LEGACY_FENCE: &str = "LEGACY_FENCE";
    pub const LEGACY_PUMPKIN: &str = "LEGACY_PUMPKIN";
    pub const LEGACY_NETHERRACK: &str = "LEGACY_NETHERRACK";
    pub const LEGACY_SOUL_SAND: &str = "LEGACY_SOUL_SAND";
    pub const LEGACY_GLOWSTONE: &str = "LEGACY_GLOWSTONE";
    pub const LEGACY_PORTAL: &str = "LEGACY_PORTAL";
    pub const LEGACY_JACK_O_LANTERN: &str = "LEGACY_JACK_O_LANTERN";
    pub const LEGACY_CAKE_BLOCK: &str = "LEGACY_CAKE_BLOCK";
    pub const LEGACY_DIODE_BLOCK_OFF: &str = "LEGACY_DIODE_BLOCK_OFF";
    pub const LEGACY_DIODE_BLOCK_ON: &str = "LEGACY_DIODE_BLOCK_ON";
    pub const LEGACY_STAINED_GLASS: &str = "LEGACY_STAINED_GLASS";
    pub const LEGACY_TRAP_DOOR: &str = "LEGACY_TRAP_DOOR";
    pub const LEGACY_MONSTER_EGGS: &str = "LEGACY_MONSTER_EGGS";
    pub const LEGACY_SMOOTH_BRICK: &str = "LEGACY_SMOOTH_BRICK";
    pub const LEGACY_HUGE_MUSHROOM_1: &str = "LEGACY_HUGE_MUSHROOM_1";
    pub const LEGACY_HUGE_MUSHROOM_2: &str = "LEGACY_HUGE_MUSHROOM_2";
    pub const LEGACY_IRON_FENCE: &str = "LEGACY_IRON_FENCE";
    pub const LEGACY_THIN_GLASS: &str = "LEGACY_THIN_GLASS";
    pub const LEGACY_MELON_BLOCK: &str = "LEGACY_MELON_BLOCK";
    pub const LEGACY_PUMPKIN_STEM: &str = "LEGACY_PUMPKIN_STEM";
    pub const LEGACY_MELON_STEM: &str = "LEGACY_MELON_STEM";
    pub const LEGACY_VINE: &str = "LEGACY_VINE";
    pub const LEGACY_FENCE_GATE: &str = "LEGACY_FENCE_GATE";
    pub const LEGACY_BRICK_STAIRS: &str = "LEGACY_BRICK_STAIRS";
    pub const LEGACY_SMOOTH_STAIRS: &str = "LEGACY_SMOOTH_STAIRS";
    pub const LEGACY_MYCEL: &str = "LEGACY_MYCEL";
    pub const LEGACY_WATER_LILY: &str = "LEGACY_WATER_LILY";
    pub const LEGACY_NETHER_BRICK: &str = "LEGACY_NETHER_BRICK";
    pub const LEGACY_NETHER_FENCE: &str = "LEGACY_NETHER_FENCE";
    pub const LEGACY_NETHER_BRICK_STAIRS: &str = "LEGACY_NETHER_BRICK_STAIRS";
    pub const LEGACY_NETHER_WARTS: &str = "LEGACY_NETHER_WARTS";
    pub const LEGACY_ENCHANTMENT_TABLE: &str = "LEGACY_ENCHANTMENT_TABLE";
    pub const LEGACY_BREWING_STAND: &str = "LEGACY_BREWING_STAND";
    pub const LEGACY_CAULDRON: &str = "LEGACY_CAULDRON";
    pub const LEGACY_ENDER_PORTAL: &str = "LEGACY_ENDER_PORTAL";
    pub const LEGACY_ENDER_PORTAL_FRAME: &str = "LEGACY_ENDER_PORTAL_FRAME";
    pub const LEGACY_ENDER_STONE: &str = "LEGACY_ENDER_STONE";
    pub const LEGACY_DRAGON_EGG: &str = "LEGACY_DRAGON_EGG";
    pub const LEGACY_REDSTONE_LAMP_OFF: &str = "LEGACY_REDSTONE_LAMP_OFF";
    pub const LEGACY_REDSTONE_LAMP_ON: &str = "LEGACY_REDSTONE_LAMP_ON";
    pub const LEGACY_WOOD_DOUBLE_STEP: &str = "LEGACY_WOOD_DOUBLE_STEP";
    pub const LEGACY_WOOD_STEP: &str = "LEGACY_WOOD_STEP";
    pub const LEGACY_COCOA: &str = "LEGACY_COCOA";
    pub const LEGACY_SANDSTONE_STAIRS: &str = "LEGACY_SANDSTONE_STAIRS";
    pub const LEGACY_EMERALD_ORE: &str = "LEGACY_EMERALD_ORE";
    pub const LEGACY_ENDER_CHEST: &str = "LEGACY_ENDER_CHEST";
    pub const LEGACY_TRIPWIRE_HOOK: &str = "LEGACY_TRIPWIRE_HOOK";
    pub const LEGACY_TRIPWIRE: &str = "LEGACY_TRIPWIRE";
    pub const LEGACY_EMERALD_BLOCK: &str = "LEGACY_EMERALD_BLOCK";
    pub const LEGACY_SPRUCE_WOOD_STAIRS: &str = "LEGACY_SPRUCE_WOOD_STAIRS";
    pub const LEGACY_BIRCH_WOOD_STAIRS: &str = "LEGACY_BIRCH_WOOD_STAIRS";
    pub const LEGACY_JUNGLE_WOOD_STAIRS: &str = "LEGACY_JUNGLE_WOOD_STAIRS";
    pub const LEGACY_COMMAND: &str = "LEGACY_COMMAND";
    pub const LEGACY_BEACON: &str = "LEGACY_BEACON";
    pub const LEGACY_COBBLE_WALL: &str = "LEGACY_COBBLE_WALL";
    pub const LEGACY_FLOWER_POT: &str = "LEGACY_FLOWER_POT";
    pub const LEGACY_CARROT: &str = "LEGACY_CARROT";
    pub const LEGACY_POTATO: &str = "LEGACY_POTATO";
    pub const LEGACY_WOOD_BUTTON: &str = "LEGACY_WOOD_BUTTON";
    pub const LEGACY_SKULL: &str = "LEGACY_SKULL";
    pub const LEGACY_ANVIL: &str = "LEGACY_ANVIL";
    pub const LEGACY_TRAPPED_CHEST: &str = "LEGACY_TRAPPED_CHEST";
    pub const LEGACY_GOLD_PLATE: &str = "LEGACY_GOLD_PLATE";
    pub const LEGACY_IRON_PLATE: &str = "LEGACY_IRON_PLATE";
    pub const LEGACY_REDSTONE_COMPARATOR_OFF: &str = "LEGACY_REDSTONE_COMPARATOR_OFF";
    pub const LEGACY_REDSTONE_COMPARATOR_ON: &str = "LEGACY_REDSTONE_COMPARATOR_ON";
    pub const LEGACY_DAYLIGHT_DETECTOR: &str = "LEGACY_DAYLIGHT_DETECTOR";
    pub const LEGACY_REDSTONE_BLOCK: &str = "LEGACY_REDSTONE_BLOCK";
    pub const LEGACY_QUARTZ_ORE: &str = "LEGACY_QUARTZ_ORE";
    pub const LEGACY_HOPPER: &str = "LEGACY_HOPPER";
    pub const LEGACY_QUARTZ_BLOCK: &str = "LEGACY_QUARTZ_BLOCK";
    pub const LEGACY_QUARTZ_STAIRS: &str = "LEGACY_QUARTZ_STAIRS";
    pub const LEGACY_ACTIVATOR_RAIL: &str = "LEGACY_ACTIVATOR_RAIL";
    pub const LEGACY_DROPPER: &str = "LEGACY_DROPPER";
    pub const LEGACY_STAINED_CLAY: &str = "LEGACY_STAINED_CLAY";
    pub const LEGACY_STAINED_GLASS_PANE: &str = "LEGACY_STAINED_GLASS_PANE";
    pub const LEGACY_LEAVES_2: &str = "LEGACY_LEAVES_2";
    pub const LEGACY_LOG_2: &str = "LEGACY_LOG_2";
    pub const LEGACY_ACACIA_STAIRS: &str = "LEGACY_ACACIA_STAIRS";
    pub const LEGACY_DARK_OAK_STAIRS: &str = "LEGACY_DARK_OAK_STAIRS";
    pub const LEGACY_SLIME_BLOCK: &str = "LEGACY_SLIME_BLOCK";
    pub const LEGACY_BARRIER: &str = "LEGACY_BARRIER";
    pub const LEGACY_IRON_TRAPDOOR: &str = "LEGACY_IRON_TRAPDOOR";
    pub const LEGACY_PRISMARINE: &str = "LEGACY_PRISMARINE";
    pub const LEGACY_SEA_LANTERN: &str = "LEGACY_SEA_LANTERN";
    pub const LEGACY_HAY_BLOCK: &str = "LEGACY_HAY_BLOCK";
    pub const LEGACY_CARPET: &str = "LEGACY_CARPET";
    pub const LEGACY_HARD_CLAY: &str = "LEGACY_HARD_CLAY";
    pub const LEGACY_COAL_BLOCK: &str = "LEGACY_COAL_BLOCK";
    pub const LEGACY_PACKED_ICE: &str = "LEGACY_PACKED_ICE";
    pub const LEGACY_DOUBLE_PLANT: &str = "LEGACY_DOUBLE_PLANT";
    pub const LEGACY_STANDING_BANNER: &str = "LEGACY_STANDING_BANNER";
    pub const LEGACY_WALL_BANNER: &str = "LEGACY_WALL_BANNER";
    pub const LEGACY_DAYLIGHT_DETECTOR_INVERTED: &str = "LEGACY_DAYLIGHT_DETECTOR_INVERTED";
    pub const LEGACY_RED_SANDSTONE: &str = "LEGACY_RED_SANDSTONE";
    pub const LEGACY_RED_SANDSTONE_STAIRS: &str = "LEGACY_RED_SANDSTONE_STAIRS";
    pub const LEGACY_DOUBLE_STONE_SLAB2: &str = "LEGACY_DOUBLE_STONE_SLAB2";
    pub const LEGACY_STONE_SLAB2: &str = "LEGACY_STONE_SLAB2";
    pub const LEGACY_SPRUCE_FENCE_GATE: &str = "LEGACY_SPRUCE_FENCE_GATE";
    pub const LEGACY_BIRCH_FENCE_GATE: &str = "LEGACY_BIRCH_FENCE_GATE";
    pub const LEGACY_JUNGLE_FENCE_GATE: &str = "LEGACY_JUNGLE_FENCE_GATE";
    pub const LEGACY_DARK_OAK_FENCE_GATE: &str = "LEGACY_DARK_OAK_FENCE_GATE";
    pub const LEGACY_ACACIA_FENCE_GATE: &str = "LEGACY_ACACIA_FENCE_GATE";
    pub const LEGACY_SPRUCE_FENCE: &str = "LEGACY_SPRUCE_FENCE";
    pub const LEGACY_BIRCH_FENCE: &str = "LEGACY_BIRCH_FENCE";
    pub const LEGACY_JUNGLE_FENCE: &str = "LEGACY_JUNGLE_FENCE";
    pub const LEGACY_DARK_OAK_FENCE: &str = "LEGACY_DARK_OAK_FENCE";
    pub const LEGACY_ACACIA_FENCE: &str = "LEGACY_ACACIA_FENCE";
    pub const LEGACY_SPRUCE_DOOR: &str = "LEGACY_SPRUCE_DOOR";
    pub const LEGACY_BIRCH_DOOR: &str = "LEGACY_BIRCH_DOOR";
    pub const LEGACY_JUNGLE_DOOR: &str = "LEGACY_JUNGLE_DOOR";
    pub const LEGACY_ACACIA_DOOR: &str = "LEGACY_ACACIA_DOOR";
    pub const LEGACY_DARK_OAK_DOOR: &str = "LEGACY_DARK_OAK_DOOR";
    pub const LEGACY_END_ROD: &str = "LEGACY_END_ROD";
    pub const LEGACY_CHORUS_PLANT: &str = "LEGACY_CHORUS_PLANT";
    pub const LEGACY_CHORUS_FLOWER: &str = "LEGACY_CHORUS_FLOWER";
    pub const LEGACY_PURPUR_BLOCK: &str = "LEGACY_PURPUR_BLOCK";
    pub const LEGACY_PURPUR_PILLAR: &str = "LEGACY_PURPUR_PILLAR";
    pub const LEGACY_PURPUR_STAIRS: &str = "LEGACY_PURPUR_STAIRS";
    pub const LEGACY_PURPUR_DOUBLE_SLAB: &str = "LEGACY_PURPUR_DOUBLE_SLAB";
    pub const LEGACY_PURPUR_SLAB: &str = "LEGACY_PURPUR_SLAB";
    pub const LEGACY_END_BRICKS: &str = "LEGACY_END_BRICKS";
    pub const LEGACY_BEETROOT_BLOCK: &str = "LEGACY_BEETROOT_BLOCK";
    pub const LEGACY_GRASS_PATH: &str = "LEGACY_GRASS_PATH";
    pub const LEGACY_END_GATEWAY: &str = "LEGACY_END_GATEWAY";
    pub const LEGACY_COMMAND_REPEATING: &str = "LEGACY_COMMAND_REPEATING";
    pub const LEGACY_COMMAND_CHAIN: &str = "LEGACY_COMMAND_CHAIN";
    pub const LEGACY_FROSTED_ICE: &str = "LEGACY_FROSTED_ICE";
    pub const LEGACY_MAGMA: &str = "LEGACY_MAGMA";
    pub const LEGACY_NETHER_WART_BLOCK: &str = "LEGACY_NETHER_WART_BLOCK";
    pub const LEGACY_RED_NETHER_BRICK: &str = "LEGACY_RED_NETHER_BRICK";
    pub const LEGACY_BONE_BLOCK: &str = "LEGACY_BONE_BLOCK";
    pub const LEGACY_STRUCTURE_VOID: &str = "LEGACY_STRUCTURE_VOID";
    pub const LEGACY_OBSERVER: &str = "LEGACY_OBSERVER";
    pub const LEGACY_WHITE_SHULKER_BOX: &str = "LEGACY_WHITE_SHULKER_BOX";
    pub const LEGACY_ORANGE_SHULKER_BOX: &str = "LEGACY_ORANGE_SHULKER_BOX";
    pub const LEGACY_MAGENTA_SHULKER_BOX: &str = "LEGACY_MAGENTA_SHULKER_BOX";
    pub const LEGACY_LIGHT_BLUE_SHULKER_BOX: &str = "LEGACY_LIGHT_BLUE_SHULKER_BOX";
    pub const LEGACY_YELLOW_SHULKER_BOX: &str = "LEGACY_YELLOW_SHULKER_BOX";
    pub const LEGACY_LIME_SHULKER_BOX: &str = "LEGACY_LIME_SHULKER_BOX";
    pub const LEGACY_PINK_SHULKER_BOX: &str = "LEGACY_PINK_SHULKER_BOX";
    pub const LEGACY_GRAY_SHULKER_BOX: &str = "LEGACY_GRAY_SHULKER_BOX";
    pub const LEGACY_SILVER_SHULKER_BOX: &str = "LEGACY_SILVER_SHULKER_BOX";
    pub const LEGACY_CYAN_SHULKER_BOX: &str = "LEGACY_CYAN_SHULKER_BOX";
    pub const LEGACY_PURPLE_SHULKER_BOX: &str = "LEGACY_PURPLE_SHULKER_BOX";
    pub const LEGACY_BLUE_SHULKER_BOX: &str = "LEGACY_BLUE_SHULKER_BOX";
    pub const LEGACY_BROWN_SHULKER_BOX: &str = "LEGACY_BROWN_SHULKER_BOX";
    pub const LEGACY_GREEN_SHULKER_BOX: &str = "LEGACY_GREEN_SHULKER_BOX";
    pub const LEGACY_RED_SHULKER_BOX: &str = "LEGACY_RED_SHULKER_BOX";
    pub const LEGACY_BLACK_SHULKER_BOX: &str = "LEGACY_BLACK_SHULKER_BOX";
    pub const LEGACY_WHITE_GLAZED_TERRACOTTA: &str = "LEGACY_WHITE_GLAZED_TERRACOTTA";
    pub const LEGACY_ORANGE_GLAZED_TERRACOTTA: &str = "LEGACY_ORANGE_GLAZED_TERRACOTTA";
    pub const LEGACY_MAGENTA_GLAZED_TERRACOTTA: &str = "LEGACY_MAGENTA_GLAZED_TERRACOTTA";
    pub const LEGACY_LIGHT_BLUE_GLAZED_TERRACOTTA: &str = "LEGACY_LIGHT_BLUE_GLAZED_TERRACOTTA";
    pub const LEGACY_YELLOW_GLAZED_TERRACOTTA: &str = "LEGACY_YELLOW_GLAZED_TERRACOTTA";
    pub const LEGACY_LIME_GLAZED_TERRACOTTA: &str = "LEGACY_LIME_GLAZED_TERRACOTTA";
    pub const LEGACY_PINK_GLAZED_TERRACOTTA: &str = "LEGACY_PINK_GLAZED_TERRACOTTA";
    pub const LEGACY_GRAY_GLAZED_TERRACOTTA: &str = "LEGACY_GRAY_GLAZED_TERRACOTTA";
    pub const LEGACY_SILVER_GLAZED_TERRACOTTA: &str = "LEGACY_SILVER_GLAZED_TERRACOTTA";
    pub const LEGACY_CYAN_GLAZED_TERRACOTTA: &str = "LEGACY_CYAN_GLAZED_TERRACOTTA";
    pub const LEGACY_PURPLE_GLAZED_TERRACOTTA: &str = "LEGACY_PURPLE_GLAZED_TERRACOTTA";
    pub const LEGACY_BLUE_GLAZED_TERRACOTTA: &str = "LEGACY_BLUE_GLAZED_TERRACOTTA";
    pub const LEGACY_BROWN_GLAZED_TERRACOTTA: &str = "LEGACY_BROWN_GLAZED_TERRACOTTA";
    pub const LEGACY_GREEN_GLAZED_TERRACOTTA: &str = "LEGACY_GREEN_GLAZED_TERRACOTTA";
    pub const LEGACY_RED_GLAZED_TERRACOTTA: &str = "LEGACY_RED_GLAZED_TERRACOTTA";
    pub const LEGACY_BLACK_GLAZED_TERRACOTTA: &str = "LEGACY_BLACK_GLAZED_TERRACOTTA";
    pub const LEGACY_CONCRETE: &str = "LEGACY_CONCRETE";
    pub const LEGACY_CONCRETE_POWDER: &str = "LEGACY_CONCRETE_POWDER";
    pub const LEGACY_STRUCTURE_BLOCK: &str = "LEGACY_STRUCTURE_BLOCK";
    pub const LEGACY_IRON_SPADE: &str = "LEGACY_IRON_SPADE";
    pub const LEGACY_IRON_PICKAXE: &str = "LEGACY_IRON_PICKAXE";
    pub const LEGACY_IRON_AXE: &str = "LEGACY_IRON_AXE";
    pub const LEGACY_FLINT_AND_STEEL: &str = "LEGACY_FLINT_AND_STEEL";
    pub const LEGACY_APPLE: &str = "LEGACY_APPLE";
    pub const LEGACY_BOW: &str = "LEGACY_BOW";
    pub const LEGACY_ARROW: &str = "LEGACY_ARROW";
    pub const LEGACY_COAL: &str = "LEGACY_COAL";
    pub const LEGACY_DIAMOND: &str = "LEGACY_DIAMOND";
    pub const LEGACY_IRON_INGOT: &str = "LEGACY_IRON_INGOT";
    pub const LEGACY_GOLD_INGOT: &str = "LEGACY_GOLD_INGOT";
    pub const LEGACY_IRON_SWORD: &str = "LEGACY_IRON_SWORD";
    pub const LEGACY_WOOD_SWORD: &str = "LEGACY_WOOD_SWORD";
    pub const LEGACY_WOOD_SPADE: &str = "LEGACY_WOOD_SPADE";
    pub const LEGACY_WOOD_PICKAXE: &str = "LEGACY_WOOD_PICKAXE";
    pub const LEGACY_WOOD_AXE: &str = "LEGACY_WOOD_AXE";
    pub const LEGACY_STONE_SWORD: &str = "LEGACY_STONE_SWORD";
    pub const LEGACY_STONE_SPADE: &str = "LEGACY_STONE_SPADE";
    pub const LEGACY_STONE_PICKAXE: &str = "LEGACY_STONE_PICKAXE";
    pub const LEGACY_STONE_AXE: &str = "LEGACY_STONE_AXE";
    pub const LEGACY_DIAMOND_SWORD: &str = "LEGACY_DIAMOND_SWORD";
    pub const LEGACY_DIAMOND_SPADE: &str = "LEGACY_DIAMOND_SPADE";
    pub const LEGACY_DIAMOND_PICKAXE: &str = "LEGACY_DIAMOND_PICKAXE";
    pub const LEGACY_DIAMOND_AXE: &str = "LEGACY_DIAMOND_AXE";
    pub const LEGACY_STICK: &str = "LEGACY_STICK";
    pub const LEGACY_BOWL: &str = "LEGACY_BOWL";
    pub const LEGACY_MUSHROOM_SOUP: &str = "LEGACY_MUSHROOM_SOUP";
    pub const LEGACY_GOLD_SWORD: &str = "LEGACY_GOLD_SWORD";
    pub const LEGACY_GOLD_SPADE: &str = "LEGACY_GOLD_SPADE";
    pub const LEGACY_GOLD_PICKAXE: &str = "LEGACY_GOLD_PICKAXE";
    pub const LEGACY_GOLD_AXE: &str = "LEGACY_GOLD_AXE";
    pub const LEGACY_STRING: &str = "LEGACY_STRING";
    pub const LEGACY_FEATHER: &str = "LEGACY_FEATHER";
    pub const LEGACY_SULPHUR: &str = "LEGACY_SULPHUR";
    pub const LEGACY_WOOD_HOE: &str = "LEGACY_WOOD_HOE";
    pub const LEGACY_STONE_HOE: &str = "LEGACY_STONE_HOE";
    pub const LEGACY_IRON_HOE: &str = "LEGACY_IRON_HOE";
    pub const LEGACY_DIAMOND_HOE: &str = "LEGACY_DIAMOND_HOE";
    pub const LEGACY_GOLD_HOE: &str = "LEGACY_GOLD_HOE";
    pub const LEGACY_SEEDS: &str = "LEGACY_SEEDS";
    pub const LEGACY_WHEAT: &str = "LEGACY_WHEAT";
    pub const LEGACY_BREAD: &str = "LEGACY_BREAD";
    pub const LEGACY_LEATHER_HELMET: &str = "LEGACY_LEATHER_HELMET";
    pub const LEGACY_LEATHER_CHESTPLATE: &str = "LEGACY_LEATHER_CHESTPLATE";
    pub const LEGACY_LEATHER_LEGGINGS: &str = "LEGACY_LEATHER_LEGGINGS";
    pub const LEGACY_LEATHER_BOOTS: &str = "LEGACY_LEATHER_BOOTS";
    pub const LEGACY_CHAINMAIL_HELMET: &str = "LEGACY_CHAINMAIL_HELMET";
    pub const LEGACY_CHAINMAIL_CHESTPLATE: &str = "LEGACY_CHAINMAIL_CHESTPLATE";
    pub const LEGACY_CHAINMAIL_LEGGINGS: &str = "LEGACY_CHAINMAIL_LEGGINGS";
    pub const LEGACY_CHAINMAIL_BOOTS: &str = "LEGACY_CHAINMAIL_BOOTS";
    pub const LEGACY_IRON_HELMET: &str = "LEGACY_IRON_HELMET";
    pub const LEGACY_IRON_CHESTPLATE: &str = "LEGACY_IRON_CHESTPLATE";
    pub const LEGACY_IRON_LEGGINGS: &str = "LEGACY_IRON_LEGGINGS";
    pub const LEGACY_IRON_BOOTS: &str = "LEGACY_IRON_BOOTS";
    pub const LEGACY_DIAMOND_HELMET: &str = "LEGACY_DIAMOND_HELMET";
    pub const LEGACY_DIAMOND_CHESTPLATE: &str = "LEGACY_DIAMOND_CHESTPLATE";
    pub const LEGACY_DIAMOND_LEGGINGS: &str = "LEGACY_DIAMOND_LEGGINGS";
    pub const LEGACY_DIAMOND_BOOTS: &str = "LEGACY_DIAMOND_BOOTS";
    pub const LEGACY_GOLD_HELMET: &str = "LEGACY_GOLD_HELMET";
    pub const LEGACY_GOLD_CHESTPLATE: &str = "LEGACY_GOLD_CHESTPLATE";
    pub const LEGACY_GOLD_LEGGINGS: &str = "LEGACY_GOLD_LEGGINGS";
    pub const LEGACY_GOLD_BOOTS: &str = "LEGACY_GOLD_BOOTS";
    pub const LEGACY_FLINT: &str = "LEGACY_FLINT";
    pub const LEGACY_PORK: &str = "LEGACY_PORK";
    pub const LEGACY_GRILLED_PORK: &str = "LEGACY_GRILLED_PORK";
    pub const LEGACY_PAINTING: &str = "LEGACY_PAINTING";
    pub const LEGACY_GOLDEN_APPLE: &str = "LEGACY_GOLDEN_APPLE";
    pub const LEGACY_SIGN: &str = "LEGACY_SIGN";
    pub const LEGACY_WOOD_DOOR: &str = "LEGACY_WOOD_DOOR";
    pub const LEGACY_BUCKET: &str = "LEGACY_BUCKET";
    pub const LEGACY_WATER_BUCKET: &str = "LEGACY_WATER_BUCKET";
    pub const LEGACY_LAVA_BUCKET: &str = "LEGACY_LAVA_BUCKET";
    pub const LEGACY_MINECART: &str = "LEGACY_MINECART";
    pub const LEGACY_SADDLE: &str = "LEGACY_SADDLE";
    pub const LEGACY_IRON_DOOR: &str = "LEGACY_IRON_DOOR";
    pub const LEGACY_REDSTONE: &str = "LEGACY_REDSTONE";
    pub const LEGACY_SNOW_BALL: &str = "LEGACY_SNOW_BALL";
    pub const LEGACY_BOAT: &str = "LEGACY_BOAT";
    pub const LEGACY_LEATHER: &str = "LEGACY_LEATHER";
    pub const LEGACY_MILK_BUCKET: &str = "LEGACY_MILK_BUCKET";
    pub const LEGACY_CLAY_BRICK: &str = "LEGACY_CLAY_BRICK";
    pub const LEGACY_CLAY_BALL: &str = "LEGACY_CLAY_BALL";
    pub const LEGACY_SUGAR_CANE: &str = "LEGACY_SUGAR_CANE";
    pub const LEGACY_PAPER: &str = "LEGACY_PAPER";
    pub const LEGACY_BOOK: &str = "LEGACY_BOOK";
    pub const LEGACY_SLIME_BALL: &str = "LEGACY_SLIME_BALL";
    pub const LEGACY_STORAGE_MINECART: &str = "LEGACY_STORAGE_MINECART";
    pub const LEGACY_POWERED_MINECART: &str = "LEGACY_POWERED_MINECART";
    pub const LEGACY_EGG: &str = "LEGACY_EGG";
    pub const LEGACY_COMPASS: &str = "LEGACY_COMPASS";
    pub const LEGACY_FISHING_ROD: &str = "LEGACY_FISHING_ROD";
    pub const LEGACY_WATCH: &str = "LEGACY_WATCH";
    pub const LEGACY_GLOWSTONE_DUST: &str = "LEGACY_GLOWSTONE_DUST";
    pub const LEGACY_RAW_FISH: &str = "LEGACY_RAW_FISH";
    pub const LEGACY_COOKED_FISH: &str = "LEGACY_COOKED_FISH";
    pub const LEGACY_INK_SACK: &str = "LEGACY_INK_SACK";
    pub const LEGACY_BONE: &str = "LEGACY_BONE";
    pub const LEGACY_SUGAR: &str = "LEGACY_SUGAR";
    pub const LEGACY_CAKE: &str = "LEGACY_CAKE";
    pub const LEGACY_BED: &str = "LEGACY_BED";
    pub const LEGACY_DIODE: &str = "LEGACY_DIODE";
    pub const LEGACY_COOKIE: &str = "LEGACY_COOKIE";
    pub const LEGACY_MAP: &str = "LEGACY_MAP";
    pub const LEGACY_SHEARS: &str = "LEGACY_SHEARS";
    pub const LEGACY_MELON: &str = "LEGACY_MELON";
    pub const LEGACY_PUMPKIN_SEEDS: &str = "LEGACY_PUMPKIN_SEEDS";
    pub const LEGACY_MELON_SEEDS: &str = "LEGACY_MELON_SEEDS";
    pub const LEGACY_RAW_BEEF: &str = "LEGACY_RAW_BEEF";
    pub const LEGACY_COOKED_BEEF: &str = "LEGACY_COOKED_BEEF";
    pub const LEGACY_RAW_CHICKEN: &str = "LEGACY_RAW_CHICKEN";
    pub const LEGACY_COOKED_CHICKEN: &str = "LEGACY_COOKED_CHICKEN";
    pub const LEGACY_ROTTEN_FLESH: &str = "LEGACY_ROTTEN_FLESH";
    pub const LEGACY_ENDER_PEARL: &str = "LEGACY_ENDER_PEARL";
    pub const LEGACY_BLAZE_ROD: &str = "LEGACY_BLAZE_ROD";
    pub const LEGACY_GHAST_TEAR: &str = "LEGACY_GHAST_TEAR";
    pub const LEGACY_GOLD_NUGGET: &str = "LEGACY_GOLD_NUGGET";
    pub const LEGACY_NETHER_STALK: &str = "LEGACY_NETHER_STALK";
    pub const LEGACY_POTION: &str = "LEGACY_POTION";
    pub const LEGACY_GLASS_BOTTLE: &str = "LEGACY_GLASS_BOTTLE";
    pub const LEGACY_SPIDER_EYE: &str = "LEGACY_SPIDER_EYE";
    pub const LEGACY_FERMENTED_SPIDER_EYE: &str = "LEGACY_FERMENTED_SPIDER_EYE";
    pub const LEGACY_BLAZE_POWDER: &str = "LEGACY_BLAZE_POWDER";
    pub const LEGACY_MAGMA_CREAM: &str = "LEGACY_MAGMA_CREAM";
    pub const LEGACY_BREWING_STAND_ITEM: &str = "LEGACY_BREWING_STAND_ITEM";
    pub const LEGACY_CAULDRON_ITEM: &str = "LEGACY_CAULDRON_ITEM";
    pub const LEGACY_EYE_OF_ENDER: &str = "LEGACY_EYE_OF_ENDER";
    pub const LEGACY_SPECKLED_MELON: &str = "LEGACY_SPECKLED_MELON";
    pub const LEGACY_MONSTER_EGG: &str = "LEGACY_MONSTER_EGG";
    pub const LEGACY_EXP_BOTTLE: &str = "LEGACY_EXP_BOTTLE";
    pub const LEGACY_FIREBALL: &str = "LEGACY_FIREBALL";
    pub const LEGACY_BOOK_AND_QUILL: &str = "LEGACY_BOOK_AND_QUILL";
    pub const LEGACY_WRITTEN_BOOK: &str = "LEGACY_WRITTEN_BOOK";
    pub const LEGACY_EMERALD: &str = "LEGACY_EMERALD";
    pub const LEGACY_ITEM_FRAME: &str = "LEGACY_ITEM_FRAME";
    pub const LEGACY_FLOWER_POT_ITEM: &str = "LEGACY_FLOWER_POT_ITEM";
    pub const LEGACY_CARROT_ITEM: &str = "LEGACY_CARROT_ITEM";
    pub const LEGACY_POTATO_ITEM: &str = "LEGACY_POTATO_ITEM";
    pub const LEGACY_BAKED_POTATO: &str = "LEGACY_BAKED_POTATO";
    pub const LEGACY_POISONOUS_POTATO: &str = "LEGACY_POISONOUS_POTATO";
    pub const LEGACY_EMPTY_MAP: &str = "LEGACY_EMPTY_MAP";
    pub const LEGACY_GOLDEN_CARROT: &str = "LEGACY_GOLDEN_CARROT";
    pub const LEGACY_SKULL_ITEM: &str = "LEGACY_SKULL_ITEM";
    pub const LEGACY_CARROT_STICK: &str = "LEGACY_CARROT_STICK";
    pub const LEGACY_NETHER_STAR: &str = "LEGACY_NETHER_STAR";
    pub const LEGACY_PUMPKIN_PIE: &str = "LEGACY_PUMPKIN_PIE";
    pub const LEGACY_FIREWORK: &str = "LEGACY_FIREWORK";
    pub const LEGACY_FIREWORK_CHARGE: &str = "LEGACY_FIREWORK_CHARGE";
    pub const LEGACY_ENCHANTED_BOOK: &str = "LEGACY_ENCHANTED_BOOK";
    pub const LEGACY_REDSTONE_COMPARATOR: &str = "LEGACY_REDSTONE_COMPARATOR";
    pub const LEGACY_NETHER_BRICK_ITEM: &str = "LEGACY_NETHER_BRICK_ITEM";
    pub const LEGACY_QUARTZ: &str = "LEGACY_QUARTZ";
    pub const LEGACY_EXPLOSIVE_MINECART: &str = "LEGACY_EXPLOSIVE_MINECART";
    pub const LEGACY_HOPPER_MINECART: &str = "LEGACY_HOPPER_MINECART";
    pub const LEGACY_PRISMARINE_SHARD: &str = "LEGACY_PRISMARINE_SHARD";
    pub const LEGACY_PRISMARINE_CRYSTALS: &str = "LEGACY_PRISMARINE_CRYSTALS";
    pub const LEGACY_RABBIT: &str = "LEGACY_RABBIT";
    pub const LEGACY_COOKED_RABBIT: &str = "LEGACY_COOKED_RABBIT";
    pub const LEGACY_RABBIT_STEW: &str = "LEGACY_RABBIT_STEW";
    pub const LEGACY_RABBIT_FOOT: &str = "LEGACY_RABBIT_FOOT";
    pub const LEGACY_RABBIT_HIDE: &str = "LEGACY_RABBIT_HIDE";
    pub const LEGACY_ARMOR_STAND: &str = "LEGACY_ARMOR_STAND";
    pub const LEGACY_IRON_BARDING: &str = "LEGACY_IRON_BARDING";
    pub const LEGACY_GOLD_BARDING: &str = "LEGACY_GOLD_BARDING";
    pub const LEGACY_DIAMOND_BARDING: &str = "LEGACY_DIAMOND_BARDING";
    pub const LEGACY_LEASH: &str = "LEGACY_LEASH";
    pub const LEGACY_NAME_TAG: &str = "LEGACY_NAME_TAG";
    pub const LEGACY_COMMAND_MINECART: &str = "LEGACY_COMMAND_MINECART";
    pub const LEGACY_MUTTON: &str = "LEGACY_MUTTON";
    pub const LEGACY_COOKED_MUTTON: &str = "LEGACY_COOKED_MUTTON";
    pub const LEGACY_BANNER: &str = "LEGACY_BANNER";
    pub const LEGACY_END_CRYSTAL: &str = "LEGACY_END_CRYSTAL";
    pub const LEGACY_SPRUCE_DOOR_ITEM: &str = "LEGACY_SPRUCE_DOOR_ITEM";
    pub const LEGACY_BIRCH_DOOR_ITEM: &str = "LEGACY_BIRCH_DOOR_ITEM";
    pub const LEGACY_JUNGLE_DOOR_ITEM: &str = "LEGACY_JUNGLE_DOOR_ITEM";
    pub const LEGACY_ACACIA_DOOR_ITEM: &str = "LEGACY_ACACIA_DOOR_ITEM";
    pub const LEGACY_DARK_OAK_DOOR_ITEM: &str = "LEGACY_DARK_OAK_DOOR_ITEM";
    pub const LEGACY_CHORUS_FRUIT: &str = "LEGACY_CHORUS_FRUIT";
    pub const LEGACY_CHORUS_FRUIT_POPPED: &str = "LEGACY_CHORUS_FRUIT_POPPED";
    pub const LEGACY_BEETROOT: &str = "LEGACY_BEETROOT";
    pub const LEGACY_BEETROOT_SEEDS: &str = "LEGACY_BEETROOT_SEEDS";
    pub const LEGACY_BEETROOT_SOUP: &str = "LEGACY_BEETROOT_SOUP";
    pub const LEGACY_DRAGONS_BREATH: &str = "LEGACY_DRAGONS_BREATH";
    pub const LEGACY_SPLASH_POTION: &str = "LEGACY_SPLASH_POTION";
    pub const LEGACY_SPECTRAL_ARROW: &str = "LEGACY_SPECTRAL_ARROW";
    pub const LEGACY_TIPPED_ARROW: &str = "LEGACY_TIPPED_ARROW";
    pub const LEGACY_LINGERING_POTION: &str = "LEGACY_LINGERING_POTION";
    pub const LEGACY_SHIELD: &str = "LEGACY_SHIELD";
    pub const LEGACY_ELYTRA: &str = "LEGACY_ELYTRA";
    pub const LEGACY_BOAT_SPRUCE: &str = "LEGACY_BOAT_SPRUCE";
    pub const LEGACY_BOAT_BIRCH: &str = "LEGACY_BOAT_BIRCH";
    pub const LEGACY_BOAT_JUNGLE: &str = "LEGACY_BOAT_JUNGLE";
    pub const LEGACY_BOAT_ACACIA: &str = "LEGACY_BOAT_ACACIA";
    pub const LEGACY_BOAT_DARK_OAK: &str = "LEGACY_BOAT_DARK_OAK";
    pub const LEGACY_TOTEM: &str = "LEGACY_TOTEM";
    pub const LEGACY_SHULKER_SHELL: &str = "LEGACY_SHULKER_SHELL";
    pub const LEGACY_IRON_NUGGET: &str = "LEGACY_IRON_NUGGET";
    pub const LEGACY_KNOWLEDGE_BOOK: &str = "LEGACY_KNOWLEDGE_BOOK";
    pub const LEGACY_GOLD_RECORD: &str = "LEGACY_GOLD_RECORD";
    pub const LEGACY_GREEN_RECORD: &str = "LEGACY_GREEN_RECORD";
    pub const LEGACY_RECORD_3: &str = "LEGACY_RECORD_3";
    pub const LEGACY_RECORD_4: &str = "LEGACY_RECORD_4";
    pub const LEGACY_RECORD_5: &str = "LEGACY_RECORD_5";
    pub const LEGACY_RECORD_6: &str = "LEGACY_RECORD_6";
    pub const LEGACY_RECORD_7: &str = "LEGACY_RECORD_7";
    pub const LEGACY_RECORD_8: &str = "LEGACY_RECORD_8";
    pub const LEGACY_RECORD_9: &str = "LEGACY_RECORD_9";
    pub const LEGACY_RECORD_10: &str = "LEGACY_RECORD_10";
    pub const LEGACY_RECORD_11: &str = "LEGACY_RECORD_11";
    pub const LEGACY_RECORD_12: &str = "LEGACY_RECORD_12";
}

/// `org.bukkit.MinecraftExperimental$Requires`.
pub mod minecraft_experimental_requires {
    /// bukkit: org/bukkit/MinecraftExperimental$Requires.values
    pub fn values() -> &'static [&'static str] {
        &["TRADE_REBALANCE", "REDSTONE_EXPERIMENTS", "MINECART_IMPROVEMENTS"]
    }
    /// bukkit: org/bukkit/MinecraftExperimental$Requires.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TRADE_REBALANCE: &str = "TRADE_REBALANCE";
    pub const REDSTONE_EXPERIMENTS: &str = "REDSTONE_EXPERIMENTS";
    pub const MINECART_IMPROVEMENTS: &str = "MINECART_IMPROVEMENTS";
}

/// `org.bukkit.MusicInstrument`.
pub mod music_instrument {
    /// bukkit: org/bukkit/MusicInstrument.values
    pub fn values() -> &'static [&'static str] {
        &["ADMIRE_GOAT_HORN", "CALL_GOAT_HORN", "DREAM_GOAT_HORN", "FEEL_GOAT_HORN", "PONDER_GOAT_HORN", "SEEK_GOAT_HORN", "SING_GOAT_HORN", "YEARN_GOAT_HORN"]
    }
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ADMIRE_GOAT_HORN: &str = "minecraft:admire_goat_horn";
    pub const CALL_GOAT_HORN: &str = "minecraft:call_goat_horn";
    pub const DREAM_GOAT_HORN: &str = "minecraft:dream_goat_horn";
    pub const FEEL_GOAT_HORN: &str = "minecraft:feel_goat_horn";
    pub const PONDER_GOAT_HORN: &str = "minecraft:ponder_goat_horn";
    pub const SEEK_GOAT_HORN: &str = "minecraft:seek_goat_horn";
    pub const SING_GOAT_HORN: &str = "minecraft:sing_goat_horn";
    pub const YEARN_GOAT_HORN: &str = "minecraft:yearn_goat_horn";
}

/// `org.bukkit.NetherWartsState`.
pub mod nether_warts_state {
    /// bukkit: org/bukkit/NetherWartsState.values
    pub fn values() -> &'static [&'static str] {
        &["SEEDED", "STAGE_ONE", "STAGE_TWO", "RIPE"]
    }
    /// bukkit: org/bukkit/NetherWartsState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SEEDED: &str = "SEEDED";
    pub const STAGE_ONE: &str = "STAGE_ONE";
    pub const STAGE_TWO: &str = "STAGE_TWO";
    pub const RIPE: &str = "RIPE";
}

/// `org.bukkit.Note$Tone`.
pub mod note_tone {
    /// bukkit: org/bukkit/Note$Tone.values
    pub fn values() -> &'static [&'static str] {
        &["G", "A", "B", "C", "D", "E", "F"]
    }
    /// bukkit: org/bukkit/Note$Tone.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const G: &str = "G";
    pub const A: &str = "A";
    pub const B: &str = "B";
    pub const C: &str = "C";
    pub const D: &str = "D";
    pub const E: &str = "E";
    pub const F: &str = "F";
}

/// `org.bukkit.Particle$RandomizationType`.
pub mod particle_randomization_type {
    /// bukkit: org/bukkit/Particle$RandomizationType.values
    pub fn values() -> &'static [&'static str] {
        &["DEFAULT", "ALTERNATIVE", "ALTERNATIVE_WITH_SPEED"]
    }
    /// bukkit: org/bukkit/Particle$RandomizationType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DEFAULT: &str = "DEFAULT";
    pub const ALTERNATIVE: &str = "ALTERNATIVE";
    pub const ALTERNATIVE_WITH_SPEED: &str = "ALTERNATIVE_WITH_SPEED";
}

/// `org.bukkit.Particle`.
pub mod particle {
    /// bukkit: org/bukkit/Particle.values
    pub fn values() -> &'static [&'static str] {
        &["POOF", "EXPLOSION", "EXPLOSION_EMITTER", "FIREWORK", "BUBBLE", "SPLASH", "FISHING", "UNDERWATER", "CRIT", "ENCHANTED_HIT", "SMOKE", "LARGE_SMOKE", "EFFECT", "INSTANT_EFFECT", "ENTITY_EFFECT", "WITCH", "DRIPPING_WATER", "DRIPPING_LAVA", "ANGRY_VILLAGER", "HAPPY_VILLAGER", "MYCELIUM", "NOTE", "PORTAL", "ENCHANT", "FLAME", "LAVA", "CLOUD", "DUST", "ITEM_SNOWBALL", "ITEM_SLIME", "HEART", "ITEM", "BLOCK", "RAIN", "ELDER_GUARDIAN", "DRAGON_BREATH", "END_ROD", "DAMAGE_INDICATOR", "SWEEP_ATTACK", "FALLING_DUST", "TOTEM_OF_UNDYING", "SPIT", "SQUID_INK", "BUBBLE_POP", "CURRENT_DOWN", "BUBBLE_COLUMN_UP", "NAUTILUS", "DOLPHIN", "SNEEZE", "CAMPFIRE_COSY_SMOKE", "CAMPFIRE_SIGNAL_SMOKE", "COMPOSTER", "FLASH", "FALLING_LAVA", "LANDING_LAVA", "FALLING_WATER", "DRIPPING_HONEY", "FALLING_HONEY", "LANDING_HONEY", "FALLING_NECTAR", "SOUL_FIRE_FLAME", "ASH", "CRIMSON_SPORE", "WARPED_SPORE", "SOUL", "DRIPPING_OBSIDIAN_TEAR", "FALLING_OBSIDIAN_TEAR", "LANDING_OBSIDIAN_TEAR", "REVERSE_PORTAL", "WHITE_ASH", "DUST_COLOR_TRANSITION", "VIBRATION", "FALLING_SPORE_BLOSSOM", "SPORE_BLOSSOM_AIR", "SMALL_FLAME", "SNOWFLAKE", "DRIPPING_DRIPSTONE_LAVA", "FALLING_DRIPSTONE_LAVA", "DRIPPING_DRIPSTONE_WATER", "FALLING_DRIPSTONE_WATER", "GLOW_SQUID_INK", "GLOW", "WAX_ON", "WAX_OFF", "ELECTRIC_SPARK", "SCRAPE", "SONIC_BOOM", "SCULK_SOUL", "SCULK_CHARGE", "SCULK_CHARGE_POP", "SHRIEK", "CHERRY_LEAVES", "PALE_OAK_LEAVES", "RED_POPLAR_LEAVES", "ORANGE_POPLAR_LEAVES", "YELLOW_POPLAR_LEAVES", "TINTED_LEAVES", "EGG_CRACK", "DUST_PLUME", "WHITE_SMOKE", "GUST", "SMALL_GUST", "GUST_EMITTER_LARGE", "GUST_EMITTER_SMALL", "TRIAL_SPAWNER_DETECTION", "TRIAL_SPAWNER_DETECTION_OMINOUS", "VAULT_CONNECTION", "INFESTED", "ITEM_COBWEB", "DUST_PILLAR", "BLOCK_CRUMBLE", "FIREFLY", "TRAIL", "OMINOUS_SPAWNING", "RAID_OMEN", "TRIAL_OMEN", "BLOCK_MARKER", "COPPER_FIRE_FLAME", "PAUSE_MOB_GROWTH", "RESET_MOB_GROWTH", "NOXIOUS_GAS", "NOXIOUS_GAS_CLOUD", "SULFUR_CUBE_GOO", "SULFUR_BUBBLES", "GEYSER", "GEYSER_BASE", "GEYSER_PLUME", "GEYSER_POOF"]
    }
    /// bukkit: org/bukkit/Particle.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        // The names Bukkit had before 1.20.5 (`SMOKE_NORMAL`), as Paper still reads them for older plugins.
        let name = match name { "EXPLOSION_NORMAL" => "POOF", "EXPLOSION_LARGE" => "EXPLOSION", "EXPLOSION_HUGE" => "EXPLOSION_EMITTER", "FIREWORKS_SPARK" => "FIREWORK", "WATER_BUBBLE" => "BUBBLE", "WATER_SPLASH" => "SPLASH", "WATER_WAKE" => "FISHING", "SUSPENDED" => "UNDERWATER", "SUSPENDED_DEPTH" => "UNDERWATER", "CRIT_MAGIC" => "ENCHANTED_HIT", "SMOKE_NORMAL" => "SMOKE", "SMOKE_LARGE" => "LARGE_SMOKE", "SPELL" => "EFFECT", "SPELL_INSTANT" => "INSTANT_EFFECT", "SPELL_MOB" => "ENTITY_EFFECT", "SPELL_MOB_AMBIENT" => "ENTITY_EFFECT", "SPELL_WITCH" => "WITCH", "DRIP_WATER" => "DRIPPING_WATER", "DRIP_LAVA" => "DRIPPING_LAVA", "VILLAGER_ANGRY" => "ANGRY_VILLAGER", "VILLAGER_HAPPY" => "HAPPY_VILLAGER", "TOWN_AURA" => "MYCELIUM", "ENCHANTMENT_TABLE" => "ENCHANT", "REDSTONE" => "DUST", "SNOWBALL" => "ITEM_SNOWBALL", "SNOW_SHOVEL" => "ITEM_SNOWBALL", "SLIME" => "ITEM_SLIME", "ITEM_CRACK" => "ITEM", "BLOCK_CRACK" => "BLOCK", "BLOCK_DUST" => "BLOCK", "WATER_DROP" => "RAIN", "MOB_APPEARANCE" => "ELDER_GUARDIAN", "TOTEM" => "TOTEM_OF_UNDYING", "GUST_EMITTER" => "GUST_EMITTER_LARGE", other => other };
        values().iter().copied().find(|value| *value == name)
    }
    pub const POOF: &str = "POOF";
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const EXPLOSION_EMITTER: &str = "EXPLOSION_EMITTER";
    pub const FIREWORK: &str = "FIREWORK";
    pub const BUBBLE: &str = "BUBBLE";
    pub const SPLASH: &str = "SPLASH";
    pub const FISHING: &str = "FISHING";
    pub const UNDERWATER: &str = "UNDERWATER";
    pub const CRIT: &str = "CRIT";
    pub const ENCHANTED_HIT: &str = "ENCHANTED_HIT";
    pub const SMOKE: &str = "SMOKE";
    pub const LARGE_SMOKE: &str = "LARGE_SMOKE";
    pub const EFFECT: &str = "EFFECT";
    pub const INSTANT_EFFECT: &str = "INSTANT_EFFECT";
    pub const ENTITY_EFFECT: &str = "ENTITY_EFFECT";
    pub const WITCH: &str = "WITCH";
    pub const DRIPPING_WATER: &str = "DRIPPING_WATER";
    pub const DRIPPING_LAVA: &str = "DRIPPING_LAVA";
    pub const ANGRY_VILLAGER: &str = "ANGRY_VILLAGER";
    pub const HAPPY_VILLAGER: &str = "HAPPY_VILLAGER";
    pub const MYCELIUM: &str = "MYCELIUM";
    pub const NOTE: &str = "NOTE";
    pub const PORTAL: &str = "PORTAL";
    pub const ENCHANT: &str = "ENCHANT";
    pub const FLAME: &str = "FLAME";
    pub const LAVA: &str = "LAVA";
    pub const CLOUD: &str = "CLOUD";
    pub const DUST: &str = "DUST";
    pub const ITEM_SNOWBALL: &str = "ITEM_SNOWBALL";
    pub const ITEM_SLIME: &str = "ITEM_SLIME";
    pub const HEART: &str = "HEART";
    pub const ITEM: &str = "ITEM";
    pub const BLOCK: &str = "BLOCK";
    pub const RAIN: &str = "RAIN";
    pub const ELDER_GUARDIAN: &str = "ELDER_GUARDIAN";
    pub const DRAGON_BREATH: &str = "DRAGON_BREATH";
    pub const END_ROD: &str = "END_ROD";
    pub const DAMAGE_INDICATOR: &str = "DAMAGE_INDICATOR";
    pub const SWEEP_ATTACK: &str = "SWEEP_ATTACK";
    pub const FALLING_DUST: &str = "FALLING_DUST";
    pub const TOTEM_OF_UNDYING: &str = "TOTEM_OF_UNDYING";
    pub const SPIT: &str = "SPIT";
    pub const SQUID_INK: &str = "SQUID_INK";
    pub const BUBBLE_POP: &str = "BUBBLE_POP";
    pub const CURRENT_DOWN: &str = "CURRENT_DOWN";
    pub const BUBBLE_COLUMN_UP: &str = "BUBBLE_COLUMN_UP";
    pub const NAUTILUS: &str = "NAUTILUS";
    pub const DOLPHIN: &str = "DOLPHIN";
    pub const SNEEZE: &str = "SNEEZE";
    pub const CAMPFIRE_COSY_SMOKE: &str = "CAMPFIRE_COSY_SMOKE";
    pub const CAMPFIRE_SIGNAL_SMOKE: &str = "CAMPFIRE_SIGNAL_SMOKE";
    pub const COMPOSTER: &str = "COMPOSTER";
    pub const FLASH: &str = "FLASH";
    pub const FALLING_LAVA: &str = "FALLING_LAVA";
    pub const LANDING_LAVA: &str = "LANDING_LAVA";
    pub const FALLING_WATER: &str = "FALLING_WATER";
    pub const DRIPPING_HONEY: &str = "DRIPPING_HONEY";
    pub const FALLING_HONEY: &str = "FALLING_HONEY";
    pub const LANDING_HONEY: &str = "LANDING_HONEY";
    pub const FALLING_NECTAR: &str = "FALLING_NECTAR";
    pub const SOUL_FIRE_FLAME: &str = "SOUL_FIRE_FLAME";
    pub const ASH: &str = "ASH";
    pub const CRIMSON_SPORE: &str = "CRIMSON_SPORE";
    pub const WARPED_SPORE: &str = "WARPED_SPORE";
    pub const SOUL: &str = "SOUL";
    pub const DRIPPING_OBSIDIAN_TEAR: &str = "DRIPPING_OBSIDIAN_TEAR";
    pub const FALLING_OBSIDIAN_TEAR: &str = "FALLING_OBSIDIAN_TEAR";
    pub const LANDING_OBSIDIAN_TEAR: &str = "LANDING_OBSIDIAN_TEAR";
    pub const REVERSE_PORTAL: &str = "REVERSE_PORTAL";
    pub const WHITE_ASH: &str = "WHITE_ASH";
    pub const DUST_COLOR_TRANSITION: &str = "DUST_COLOR_TRANSITION";
    pub const VIBRATION: &str = "VIBRATION";
    pub const FALLING_SPORE_BLOSSOM: &str = "FALLING_SPORE_BLOSSOM";
    pub const SPORE_BLOSSOM_AIR: &str = "SPORE_BLOSSOM_AIR";
    pub const SMALL_FLAME: &str = "SMALL_FLAME";
    pub const SNOWFLAKE: &str = "SNOWFLAKE";
    pub const DRIPPING_DRIPSTONE_LAVA: &str = "DRIPPING_DRIPSTONE_LAVA";
    pub const FALLING_DRIPSTONE_LAVA: &str = "FALLING_DRIPSTONE_LAVA";
    pub const DRIPPING_DRIPSTONE_WATER: &str = "DRIPPING_DRIPSTONE_WATER";
    pub const FALLING_DRIPSTONE_WATER: &str = "FALLING_DRIPSTONE_WATER";
    pub const GLOW_SQUID_INK: &str = "GLOW_SQUID_INK";
    pub const GLOW: &str = "GLOW";
    pub const WAX_ON: &str = "WAX_ON";
    pub const WAX_OFF: &str = "WAX_OFF";
    pub const ELECTRIC_SPARK: &str = "ELECTRIC_SPARK";
    pub const SCRAPE: &str = "SCRAPE";
    pub const SONIC_BOOM: &str = "SONIC_BOOM";
    pub const SCULK_SOUL: &str = "SCULK_SOUL";
    pub const SCULK_CHARGE: &str = "SCULK_CHARGE";
    pub const SCULK_CHARGE_POP: &str = "SCULK_CHARGE_POP";
    pub const SHRIEK: &str = "SHRIEK";
    pub const CHERRY_LEAVES: &str = "CHERRY_LEAVES";
    pub const PALE_OAK_LEAVES: &str = "PALE_OAK_LEAVES";
    pub const RED_POPLAR_LEAVES: &str = "RED_POPLAR_LEAVES";
    pub const ORANGE_POPLAR_LEAVES: &str = "ORANGE_POPLAR_LEAVES";
    pub const YELLOW_POPLAR_LEAVES: &str = "YELLOW_POPLAR_LEAVES";
    pub const TINTED_LEAVES: &str = "TINTED_LEAVES";
    pub const EGG_CRACK: &str = "EGG_CRACK";
    pub const DUST_PLUME: &str = "DUST_PLUME";
    pub const WHITE_SMOKE: &str = "WHITE_SMOKE";
    pub const GUST: &str = "GUST";
    pub const SMALL_GUST: &str = "SMALL_GUST";
    pub const GUST_EMITTER_LARGE: &str = "GUST_EMITTER_LARGE";
    pub const GUST_EMITTER_SMALL: &str = "GUST_EMITTER_SMALL";
    pub const TRIAL_SPAWNER_DETECTION: &str = "TRIAL_SPAWNER_DETECTION";
    pub const TRIAL_SPAWNER_DETECTION_OMINOUS: &str = "TRIAL_SPAWNER_DETECTION_OMINOUS";
    pub const VAULT_CONNECTION: &str = "VAULT_CONNECTION";
    pub const INFESTED: &str = "INFESTED";
    pub const ITEM_COBWEB: &str = "ITEM_COBWEB";
    pub const DUST_PILLAR: &str = "DUST_PILLAR";
    pub const BLOCK_CRUMBLE: &str = "BLOCK_CRUMBLE";
    pub const FIREFLY: &str = "FIREFLY";
    pub const TRAIL: &str = "TRAIL";
    pub const OMINOUS_SPAWNING: &str = "OMINOUS_SPAWNING";
    pub const RAID_OMEN: &str = "RAID_OMEN";
    pub const TRIAL_OMEN: &str = "TRIAL_OMEN";
    pub const BLOCK_MARKER: &str = "BLOCK_MARKER";
    pub const COPPER_FIRE_FLAME: &str = "COPPER_FIRE_FLAME";
    pub const PAUSE_MOB_GROWTH: &str = "PAUSE_MOB_GROWTH";
    pub const RESET_MOB_GROWTH: &str = "RESET_MOB_GROWTH";
    pub const NOXIOUS_GAS: &str = "NOXIOUS_GAS";
    pub const NOXIOUS_GAS_CLOUD: &str = "NOXIOUS_GAS_CLOUD";
    pub const SULFUR_CUBE_GOO: &str = "SULFUR_CUBE_GOO";
    pub const SULFUR_BUBBLES: &str = "SULFUR_BUBBLES";
    pub const GEYSER: &str = "GEYSER";
    pub const GEYSER_BASE: &str = "GEYSER_BASE";
    pub const GEYSER_PLUME: &str = "GEYSER_PLUME";
    pub const GEYSER_POOF: &str = "GEYSER_POOF";
}

/// `org.bukkit.PortalType`.
pub mod portal_type {
    /// bukkit: org/bukkit/PortalType.values
    pub fn values() -> &'static [&'static str] {
        &["NETHER", "ENDER", "END_GATEWAY", "CUSTOM"]
    }
    /// bukkit: org/bukkit/PortalType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NETHER: &str = "NETHER";
    pub const ENDER: &str = "ENDER";
    pub const END_GATEWAY: &str = "END_GATEWAY";
    pub const CUSTOM: &str = "CUSTOM";
}

/// `org.bukkit.Raid$RaidStatus`.
pub mod raid_raid_status {
    /// bukkit: org/bukkit/Raid$RaidStatus.values
    pub fn values() -> &'static [&'static str] {
        &["ONGOING", "VICTORY", "LOSS", "STOPPED"]
    }
    /// bukkit: org/bukkit/Raid$RaidStatus.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ONGOING: &str = "ONGOING";
    pub const VICTORY: &str = "VICTORY";
    pub const LOSS: &str = "LOSS";
    pub const STOPPED: &str = "STOPPED";
}

/// `org.bukkit.Rotation`.
pub mod rotation {
    /// bukkit: org/bukkit/Rotation.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "CLOCKWISE_45", "CLOCKWISE", "CLOCKWISE_135", "FLIPPED", "FLIPPED_45", "COUNTER_CLOCKWISE", "COUNTER_CLOCKWISE_45"]
    }
    /// bukkit: org/bukkit/Rotation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const CLOCKWISE_45: &str = "CLOCKWISE_45";
    pub const CLOCKWISE: &str = "CLOCKWISE";
    pub const CLOCKWISE_135: &str = "CLOCKWISE_135";
    pub const FLIPPED: &str = "FLIPPED";
    pub const FLIPPED_45: &str = "FLIPPED_45";
    pub const COUNTER_CLOCKWISE: &str = "COUNTER_CLOCKWISE";
    pub const COUNTER_CLOCKWISE_45: &str = "COUNTER_CLOCKWISE_45";
}

/// `org.bukkit.SandstoneType`.
pub mod sandstone_type {
    /// bukkit: org/bukkit/SandstoneType.values
    pub fn values() -> &'static [&'static str] {
        &["CRACKED", "GLYPHED", "SMOOTH"]
    }
    /// bukkit: org/bukkit/SandstoneType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CRACKED: &str = "CRACKED";
    pub const GLYPHED: &str = "GLYPHED";
    pub const SMOOTH: &str = "SMOOTH";
}

/// `org.bukkit.ServerLinks$Type`.
pub mod server_links_type {
    /// bukkit: org/bukkit/ServerLinks$Type.values
    pub fn values() -> &'static [&'static str] {
        &["REPORT_BUG", "COMMUNITY_GUIDELINES", "SUPPORT", "STATUS", "FEEDBACK", "COMMUNITY", "WEBSITE", "FORUMS", "NEWS", "ANNOUNCEMENTS"]
    }
    /// bukkit: org/bukkit/ServerLinks$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const REPORT_BUG: &str = "REPORT_BUG";
    pub const COMMUNITY_GUIDELINES: &str = "COMMUNITY_GUIDELINES";
    pub const SUPPORT: &str = "SUPPORT";
    pub const STATUS: &str = "STATUS";
    pub const FEEDBACK: &str = "FEEDBACK";
    pub const COMMUNITY: &str = "COMMUNITY";
    pub const WEBSITE: &str = "WEBSITE";
    pub const FORUMS: &str = "FORUMS";
    pub const NEWS: &str = "NEWS";
    pub const ANNOUNCEMENTS: &str = "ANNOUNCEMENTS";
}

/// `org.bukkit.SkullType`.
pub mod skull_type {
    /// bukkit: org/bukkit/SkullType.values
    pub fn values() -> &'static [&'static str] {
        &["SKELETON", "WITHER", "ZOMBIE", "PLAYER", "CREEPER", "DRAGON", "PIGLIN"]
    }
    /// bukkit: org/bukkit/SkullType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SKELETON: &str = "SKELETON";
    pub const WITHER: &str = "WITHER";
    pub const ZOMBIE: &str = "ZOMBIE";
    pub const PLAYER: &str = "PLAYER";
    pub const CREEPER: &str = "CREEPER";
    pub const DRAGON: &str = "DRAGON";
    pub const PIGLIN: &str = "PIGLIN";
}

/// `org.bukkit.Sound`.
pub mod sound {
    /// bukkit: org/bukkit/Sound.values
    pub fn values() -> &'static [&'static str] {
        &["AMBIENT_BASALT_DELTAS_ADDITIONS", "AMBIENT_BASALT_DELTAS_LOOP", "AMBIENT_BASALT_DELTAS_MOOD", "AMBIENT_CAVE", "AMBIENT_CRIMSON_FOREST_ADDITIONS", "AMBIENT_CRIMSON_FOREST_LOOP", "AMBIENT_CRIMSON_FOREST_MOOD", "AMBIENT_NETHER_WASTES_ADDITIONS", "AMBIENT_NETHER_WASTES_LOOP", "AMBIENT_NETHER_WASTES_MOOD", "AMBIENT_SOUL_SAND_VALLEY_ADDITIONS", "AMBIENT_SOUL_SAND_VALLEY_LOOP", "AMBIENT_SOUL_SAND_VALLEY_MOOD", "AMBIENT_UNDERWATER_ENTER", "AMBIENT_UNDERWATER_EXIT", "AMBIENT_UNDERWATER_LOOP", "AMBIENT_UNDERWATER_LOOP_ADDITIONS", "AMBIENT_UNDERWATER_LOOP_ADDITIONS_RARE", "AMBIENT_UNDERWATER_LOOP_ADDITIONS_ULTRA_RARE", "AMBIENT_WARPED_FOREST_ADDITIONS", "AMBIENT_WARPED_FOREST_LOOP", "AMBIENT_WARPED_FOREST_MOOD", "BLOCK_AMETHYST_BLOCK_BREAK", "BLOCK_AMETHYST_BLOCK_CHIME", "BLOCK_AMETHYST_BLOCK_FALL", "BLOCK_AMETHYST_BLOCK_HIT", "BLOCK_AMETHYST_BLOCK_PLACE", "BLOCK_AMETHYST_BLOCK_RESONATE", "BLOCK_AMETHYST_BLOCK_STEP", "BLOCK_AMETHYST_CLUSTER_BREAK", "BLOCK_AMETHYST_CLUSTER_FALL", "BLOCK_AMETHYST_CLUSTER_HIT", "BLOCK_AMETHYST_CLUSTER_PLACE", "BLOCK_AMETHYST_CLUSTER_STEP", "BLOCK_ANCIENT_DEBRIS_BREAK", "BLOCK_ANCIENT_DEBRIS_FALL", "BLOCK_ANCIENT_DEBRIS_HIT", "BLOCK_ANCIENT_DEBRIS_PLACE", "BLOCK_ANCIENT_DEBRIS_STEP", "BLOCK_ANVIL_BREAK", "BLOCK_ANVIL_DESTROY", "BLOCK_ANVIL_FALL", "BLOCK_ANVIL_HIT", "BLOCK_ANVIL_LAND", "BLOCK_ANVIL_PLACE", "BLOCK_ANVIL_STEP", "BLOCK_ANVIL_USE", "BLOCK_AZALEA_BREAK", "BLOCK_AZALEA_FALL", "BLOCK_AZALEA_HIT", "BLOCK_AZALEA_PLACE", "BLOCK_AZALEA_STEP", "BLOCK_AZALEA_LEAVES_BREAK", "BLOCK_AZALEA_LEAVES_FALL", "BLOCK_AZALEA_LEAVES_HIT", "BLOCK_AZALEA_LEAVES_PLACE", "BLOCK_AZALEA_LEAVES_STEP", "BLOCK_BAMBOO_BREAK", "BLOCK_BAMBOO_FALL", "BLOCK_BAMBOO_HIT", "BLOCK_BAMBOO_PLACE", "BLOCK_BAMBOO_STEP", "BLOCK_BAMBOO_SAPLING_BREAK", "BLOCK_BAMBOO_SAPLING_HIT", "BLOCK_BAMBOO_SAPLING_PLACE", "BLOCK_BAMBOO_WOOD_BREAK", "BLOCK_BAMBOO_WOOD_FALL", "BLOCK_BAMBOO_WOOD_HIT", "BLOCK_BAMBOO_WOOD_PLACE", "BLOCK_BAMBOO_WOOD_STEP", "BLOCK_BAMBOO_WOOD_BUTTON_CLICK_OFF", "BLOCK_BAMBOO_WOOD_BUTTON_CLICK_ON", "BLOCK_BAMBOO_WOOD_DOOR_CLOSE", "BLOCK_BAMBOO_WOOD_DOOR_OPEN", "BLOCK_BAMBOO_WOOD_FENCE_GATE_CLOSE", "BLOCK_BAMBOO_WOOD_FENCE_GATE_OPEN", "BLOCK_BAMBOO_WOOD_HANGING_SIGN_BREAK", "BLOCK_BAMBOO_WOOD_HANGING_SIGN_FALL", "BLOCK_BAMBOO_WOOD_HANGING_SIGN_HIT", "BLOCK_BAMBOO_WOOD_HANGING_SIGN_PLACE", "BLOCK_BAMBOO_WOOD_HANGING_SIGN_STEP", "BLOCK_BAMBOO_WOOD_PRESSURE_PLATE_CLICK_OFF", "BLOCK_BAMBOO_WOOD_PRESSURE_PLATE_CLICK_ON", "BLOCK_BAMBOO_WOOD_TRAPDOOR_CLOSE", "BLOCK_BAMBOO_WOOD_TRAPDOOR_OPEN", "BLOCK_BARREL_CLOSE", "BLOCK_BARREL_OPEN", "BLOCK_BASALT_BREAK", "BLOCK_BASALT_FALL", "BLOCK_BASALT_HIT", "BLOCK_BASALT_PLACE", "BLOCK_BASALT_STEP", "BLOCK_BEACON_ACTIVATE", "BLOCK_BEACON_AMBIENT", "BLOCK_BEACON_DEACTIVATE", "BLOCK_BEACON_POWER_SELECT", "BLOCK_BEEHIVE_DRIP", "BLOCK_BEEHIVE_ENTER", "BLOCK_BEEHIVE_EXIT", "BLOCK_BEEHIVE_SHEAR", "BLOCK_BEEHIVE_WORK", "BLOCK_BELL_RESONATE", "BLOCK_BELL_USE", "BLOCK_BIG_DRIPLEAF_BREAK", "BLOCK_BIG_DRIPLEAF_FALL", "BLOCK_BIG_DRIPLEAF_HIT", "BLOCK_BIG_DRIPLEAF_PLACE", "BLOCK_BIG_DRIPLEAF_STEP", "BLOCK_BIG_DRIPLEAF_TILT_DOWN", "BLOCK_BIG_DRIPLEAF_TILT_UP", "BLOCK_BLASTFURNACE_FIRE_CRACKLE", "BLOCK_BONE_BLOCK_BREAK", "BLOCK_BONE_BLOCK_FALL", "BLOCK_BONE_BLOCK_HIT", "BLOCK_BONE_BLOCK_PLACE", "BLOCK_BONE_BLOCK_STEP", "BLOCK_BREWING_STAND_BREW", "BLOCK_BUBBLE_COLUMN_BUBBLE_POP", "BLOCK_BUBBLE_COLUMN_UPWARDS_AMBIENT", "BLOCK_BUBBLE_COLUMN_UPWARDS_INSIDE", "BLOCK_BUBBLE_COLUMN_WHIRLPOOL_AMBIENT", "BLOCK_BUBBLE_COLUMN_WHIRLPOOL_INSIDE", "BLOCK_CACTUS_FLOWER_BREAK", "BLOCK_CACTUS_FLOWER_PLACE", "BLOCK_CAKE_ADD_CANDLE", "BLOCK_CALCITE_BREAK", "BLOCK_CALCITE_FALL", "BLOCK_CALCITE_HIT", "BLOCK_CALCITE_PLACE", "BLOCK_CALCITE_STEP", "BLOCK_CAMPFIRE_CRACKLE", "BLOCK_CANDLE_AMBIENT", "BLOCK_CANDLE_BREAK", "BLOCK_CANDLE_EXTINGUISH", "BLOCK_CANDLE_FALL", "BLOCK_CANDLE_HIT", "BLOCK_CANDLE_PLACE", "BLOCK_CANDLE_STEP", "BLOCK_CAVE_VINES_BREAK", "BLOCK_CAVE_VINES_FALL", "BLOCK_CAVE_VINES_HIT", "BLOCK_CAVE_VINES_PICK_BERRIES", "BLOCK_CAVE_VINES_PLACE", "BLOCK_CAVE_VINES_STEP", "BLOCK_CHAIN_BREAK", "BLOCK_CHAIN_FALL", "BLOCK_CHAIN_HIT", "BLOCK_CHAIN_PLACE", "BLOCK_CHAIN_STEP", "BLOCK_CHERRY_LEAVES_BREAK", "BLOCK_CHERRY_LEAVES_FALL", "BLOCK_CHERRY_LEAVES_HIT", "BLOCK_CHERRY_LEAVES_PLACE", "BLOCK_CHERRY_LEAVES_STEP", "BLOCK_CHERRY_SAPLING_BREAK", "BLOCK_CHERRY_SAPLING_FALL", "BLOCK_CHERRY_SAPLING_HIT", "BLOCK_CHERRY_SAPLING_PLACE", "BLOCK_CHERRY_SAPLING_STEP", "BLOCK_CHERRY_WOOD_BREAK", "BLOCK_CHERRY_WOOD_FALL", "BLOCK_CHERRY_WOOD_HIT", "BLOCK_CHERRY_WOOD_PLACE", "BLOCK_CHERRY_WOOD_STEP", "BLOCK_CHERRY_WOOD_BUTTON_CLICK_OFF", "BLOCK_CHERRY_WOOD_BUTTON_CLICK_ON", "BLOCK_CHERRY_WOOD_DOOR_CLOSE", "BLOCK_CHERRY_WOOD_DOOR_OPEN", "BLOCK_CHERRY_WOOD_FENCE_GATE_CLOSE", "BLOCK_CHERRY_WOOD_FENCE_GATE_OPEN", "BLOCK_CHERRY_WOOD_HANGING_SIGN_BREAK", "BLOCK_CHERRY_WOOD_HANGING_SIGN_FALL", "BLOCK_CHERRY_WOOD_HANGING_SIGN_HIT", "BLOCK_CHERRY_WOOD_HANGING_SIGN_PLACE", "BLOCK_CHERRY_WOOD_HANGING_SIGN_STEP", "BLOCK_CHERRY_WOOD_PRESSURE_PLATE_CLICK_OFF", "BLOCK_CHERRY_WOOD_PRESSURE_PLATE_CLICK_ON", "BLOCK_CHERRY_WOOD_TRAPDOOR_CLOSE", "BLOCK_CHERRY_WOOD_TRAPDOOR_OPEN", "BLOCK_CHEST_CLOSE", "BLOCK_CHEST_LOCKED", "BLOCK_CHEST_OPEN", "BLOCK_CHISELED_BOOKSHELF_BREAK", "BLOCK_CHISELED_BOOKSHELF_FALL", "BLOCK_CHISELED_BOOKSHELF_HIT", "BLOCK_CHISELED_BOOKSHELF_INSERT", "BLOCK_CHISELED_BOOKSHELF_INSERT_ENCHANTED", "BLOCK_CHISELED_BOOKSHELF_PICKUP", "BLOCK_CHISELED_BOOKSHELF_PICKUP_ENCHANTED", "BLOCK_CHISELED_BOOKSHELF_PLACE", "BLOCK_CHISELED_BOOKSHELF_STEP", "BLOCK_CHORUS_FLOWER_DEATH", "BLOCK_CHORUS_FLOWER_GROW", "BLOCK_CINNABAR_BREAK", "BLOCK_CINNABAR_FALL", "BLOCK_CINNABAR_HIT", "BLOCK_CINNABAR_PLACE", "BLOCK_CINNABAR_STEP", "BLOCK_COBWEB_BREAK", "BLOCK_COBWEB_FALL", "BLOCK_COBWEB_HIT", "BLOCK_COBWEB_PLACE", "BLOCK_COBWEB_STEP", "BLOCK_COMPARATOR_CLICK", "BLOCK_COMPOSTER_EMPTY", "BLOCK_COMPOSTER_FILL", "BLOCK_COMPOSTER_FILL_SUCCESS", "BLOCK_COMPOSTER_READY", "BLOCK_CONDUIT_ACTIVATE", "BLOCK_CONDUIT_AMBIENT", "BLOCK_CONDUIT_AMBIENT_SHORT", "BLOCK_CONDUIT_ATTACK_TARGET", "BLOCK_CONDUIT_DEACTIVATE", "BLOCK_COPPER_BREAK", "BLOCK_COPPER_FALL", "BLOCK_COPPER_HIT", "BLOCK_COPPER_PLACE", "BLOCK_COPPER_STEP", "BLOCK_COPPER_BULB_BREAK", "BLOCK_COPPER_BULB_FALL", "BLOCK_COPPER_BULB_HIT", "BLOCK_COPPER_BULB_PLACE", "BLOCK_COPPER_BULB_STEP", "BLOCK_COPPER_BULB_TURN_OFF", "BLOCK_COPPER_BULB_TURN_ON", "BLOCK_COPPER_CHEST_CLOSE", "BLOCK_COPPER_CHEST_OPEN", "BLOCK_COPPER_CHEST_OXIDIZED_CLOSE", "BLOCK_COPPER_CHEST_OXIDIZED_OPEN", "BLOCK_COPPER_CHEST_WEATHERED_CLOSE", "BLOCK_COPPER_CHEST_WEATHERED_OPEN", "BLOCK_COPPER_DOOR_CLOSE", "BLOCK_COPPER_DOOR_OPEN", "BLOCK_COPPER_GOLEM_STATUE_BREAK", "BLOCK_COPPER_GOLEM_STATUE_FALL", "BLOCK_COPPER_GOLEM_STATUE_HIT", "BLOCK_COPPER_GOLEM_STATUE_PLACE", "BLOCK_COPPER_GOLEM_STATUE_STEP", "BLOCK_COPPER_GRATE_BREAK", "BLOCK_COPPER_GRATE_FALL", "BLOCK_COPPER_GRATE_HIT", "BLOCK_COPPER_GRATE_PLACE", "BLOCK_COPPER_GRATE_STEP", "BLOCK_COPPER_TRAPDOOR_CLOSE", "BLOCK_COPPER_TRAPDOOR_OPEN", "BLOCK_CORAL_BLOCK_BREAK", "BLOCK_CORAL_BLOCK_FALL", "BLOCK_CORAL_BLOCK_HIT", "BLOCK_CORAL_BLOCK_PLACE", "BLOCK_CORAL_BLOCK_STEP", "BLOCK_CRAFTER_CRAFT", "BLOCK_CRAFTER_FAIL", "BLOCK_CREAKING_HEART_BREAK", "BLOCK_CREAKING_HEART_FALL", "BLOCK_CREAKING_HEART_HIT", "BLOCK_CREAKING_HEART_HURT", "BLOCK_CREAKING_HEART_IDLE", "BLOCK_CREAKING_HEART_PLACE", "BLOCK_CREAKING_HEART_SPAWN", "BLOCK_CREAKING_HEART_STEP", "BLOCK_CROP_BREAK", "BLOCK_DEADBUSH_IDLE", "BLOCK_DECORATED_POT_BREAK", "BLOCK_DECORATED_POT_FALL", "BLOCK_DECORATED_POT_HIT", "BLOCK_DECORATED_POT_INSERT", "BLOCK_DECORATED_POT_INSERT_FAIL", "BLOCK_DECORATED_POT_PLACE", "BLOCK_DECORATED_POT_SHATTER", "BLOCK_DECORATED_POT_STEP", "BLOCK_DEEPSLATE_BREAK", "BLOCK_DEEPSLATE_FALL", "BLOCK_DEEPSLATE_HIT", "BLOCK_DEEPSLATE_PLACE", "BLOCK_DEEPSLATE_STEP", "BLOCK_DEEPSLATE_BRICKS_BREAK", "BLOCK_DEEPSLATE_BRICKS_FALL", "BLOCK_DEEPSLATE_BRICKS_HIT", "BLOCK_DEEPSLATE_BRICKS_PLACE", "BLOCK_DEEPSLATE_BRICKS_STEP", "BLOCK_DEEPSLATE_TILES_BREAK", "BLOCK_DEEPSLATE_TILES_FALL", "BLOCK_DEEPSLATE_TILES_HIT", "BLOCK_DEEPSLATE_TILES_PLACE", "BLOCK_DEEPSLATE_TILES_STEP", "BLOCK_DISPENSER_DISPENSE", "BLOCK_DISPENSER_FAIL", "BLOCK_DISPENSER_LAUNCH", "BLOCK_DRIED_GHAST_AMBIENT", "BLOCK_DRIED_GHAST_AMBIENT_WATER", "BLOCK_DRIED_GHAST_BREAK", "BLOCK_DRIED_GHAST_FALL", "BLOCK_DRIED_GHAST_PLACE", "BLOCK_DRIED_GHAST_PLACE_IN_WATER", "BLOCK_DRIED_GHAST_STEP", "BLOCK_DRIED_GHAST_TRANSITION", "BLOCK_DRIPSTONE_BLOCK_BREAK", "BLOCK_DRIPSTONE_BLOCK_FALL", "BLOCK_DRIPSTONE_BLOCK_HIT", "BLOCK_DRIPSTONE_BLOCK_PLACE", "BLOCK_DRIPSTONE_BLOCK_STEP", "BLOCK_DRY_GRASS_AMBIENT", "BLOCK_ENCHANTMENT_TABLE_USE", "BLOCK_END_GATEWAY_SPAWN", "BLOCK_END_PORTAL_SPAWN", "BLOCK_END_PORTAL_FRAME_FILL", "BLOCK_ENDER_CHEST_CLOSE", "BLOCK_ENDER_CHEST_OPEN", "BLOCK_EYEBLOSSOM_CLOSE", "BLOCK_EYEBLOSSOM_CLOSE_LONG", "BLOCK_EYEBLOSSOM_IDLE", "BLOCK_EYEBLOSSOM_OPEN", "BLOCK_EYEBLOSSOM_OPEN_LONG", "BLOCK_FENCE_GATE_CLOSE", "BLOCK_FENCE_GATE_OPEN", "BLOCK_FIRE_AMBIENT", "BLOCK_FIRE_EXTINGUISH", "BLOCK_FIREFLY_BUSH_IDLE", "BLOCK_FLOWERING_AZALEA_BREAK", "BLOCK_FLOWERING_AZALEA_FALL", "BLOCK_FLOWERING_AZALEA_HIT", "BLOCK_FLOWERING_AZALEA_PLACE", "BLOCK_FLOWERING_AZALEA_STEP", "BLOCK_FROGLIGHT_BREAK", "BLOCK_FROGLIGHT_FALL", "BLOCK_FROGLIGHT_HIT", "BLOCK_FROGLIGHT_PLACE", "BLOCK_FROGLIGHT_STEP", "BLOCK_FROGSPAWN_BREAK", "BLOCK_FROGSPAWN_FALL", "BLOCK_FROGSPAWN_HATCH", "BLOCK_FROGSPAWN_HIT", "BLOCK_FROGSPAWN_PLACE", "BLOCK_FROGSPAWN_STEP", "BLOCK_FUNGUS_BREAK", "BLOCK_FUNGUS_FALL", "BLOCK_FUNGUS_HIT", "BLOCK_FUNGUS_PLACE", "BLOCK_FUNGUS_STEP", "BLOCK_FURNACE_FIRE_CRACKLE", "BLOCK_GILDED_BLACKSTONE_BREAK", "BLOCK_GILDED_BLACKSTONE_FALL", "BLOCK_GILDED_BLACKSTONE_HIT", "BLOCK_GILDED_BLACKSTONE_PLACE", "BLOCK_GILDED_BLACKSTONE_STEP", "BLOCK_GLASS_BREAK", "BLOCK_GLASS_FALL", "BLOCK_GLASS_HIT", "BLOCK_GLASS_PLACE", "BLOCK_GLASS_STEP", "BLOCK_GRASS_BREAK", "BLOCK_GRASS_FALL", "BLOCK_GRASS_HIT", "BLOCK_GRASS_PLACE", "BLOCK_GRASS_STEP", "BLOCK_GRAVEL_BREAK", "BLOCK_GRAVEL_FALL", "BLOCK_GRAVEL_HIT", "BLOCK_GRAVEL_PLACE", "BLOCK_GRAVEL_STEP", "BLOCK_GRINDSTONE_USE", "BLOCK_GROWING_PLANT_CROP", "BLOCK_HANGING_ROOTS_BREAK", "BLOCK_HANGING_ROOTS_FALL", "BLOCK_HANGING_ROOTS_HIT", "BLOCK_HANGING_ROOTS_PLACE", "BLOCK_HANGING_ROOTS_STEP", "BLOCK_HANGING_SIGN_BREAK", "BLOCK_HANGING_SIGN_FALL", "BLOCK_HANGING_SIGN_HIT", "BLOCK_HANGING_SIGN_PLACE", "BLOCK_HANGING_SIGN_STEP", "BLOCK_HANGING_SIGN_WAXED_INTERACT_FAIL", "BLOCK_HEAVY_CORE_BREAK", "BLOCK_HEAVY_CORE_FALL", "BLOCK_HEAVY_CORE_HIT", "BLOCK_HEAVY_CORE_PLACE", "BLOCK_HEAVY_CORE_STEP", "BLOCK_HONEY_BLOCK_BREAK", "BLOCK_HONEY_BLOCK_FALL", "BLOCK_HONEY_BLOCK_HIT", "BLOCK_HONEY_BLOCK_PLACE", "BLOCK_HONEY_BLOCK_SLIDE", "BLOCK_HONEY_BLOCK_STEP", "BLOCK_IRON_BREAK", "BLOCK_IRON_FALL", "BLOCK_IRON_HIT", "BLOCK_IRON_PLACE", "BLOCK_IRON_STEP", "BLOCK_IRON_DOOR_CLOSE", "BLOCK_IRON_DOOR_OPEN", "BLOCK_IRON_TRAPDOOR_CLOSE", "BLOCK_IRON_TRAPDOOR_OPEN", "BLOCK_LADDER_BREAK", "BLOCK_LADDER_FALL", "BLOCK_LADDER_HIT", "BLOCK_LADDER_PLACE", "BLOCK_LADDER_STEP", "BLOCK_LANTERN_BREAK", "BLOCK_LANTERN_FALL", "BLOCK_LANTERN_HIT", "BLOCK_LANTERN_PLACE", "BLOCK_LANTERN_STEP", "BLOCK_LARGE_AMETHYST_BUD_BREAK", "BLOCK_LARGE_AMETHYST_BUD_PLACE", "BLOCK_LAVA_AMBIENT", "BLOCK_LAVA_EXTINGUISH", "BLOCK_LAVA_POP", "BLOCK_LEAF_LITTER_BREAK", "BLOCK_LEAF_LITTER_FALL", "BLOCK_LEAF_LITTER_HIT", "BLOCK_LEAF_LITTER_PLACE", "BLOCK_LEAF_LITTER_STEP", "BLOCK_LEVER_CLICK", "BLOCK_LILY_PAD_PLACE", "BLOCK_LODESTONE_BREAK", "BLOCK_LODESTONE_FALL", "BLOCK_LODESTONE_HIT", "BLOCK_LODESTONE_PLACE", "BLOCK_LODESTONE_STEP", "BLOCK_MANGROVE_ROOTS_BREAK", "BLOCK_MANGROVE_ROOTS_FALL", "BLOCK_MANGROVE_ROOTS_HIT", "BLOCK_MANGROVE_ROOTS_PLACE", "BLOCK_MANGROVE_ROOTS_STEP", "BLOCK_MEDIUM_AMETHYST_BUD_BREAK", "BLOCK_MEDIUM_AMETHYST_BUD_PLACE", "BLOCK_METAL_BREAK", "BLOCK_METAL_FALL", "BLOCK_METAL_HIT", "BLOCK_METAL_PLACE", "BLOCK_METAL_STEP", "BLOCK_METAL_PRESSURE_PLATE_CLICK_OFF", "BLOCK_METAL_PRESSURE_PLATE_CLICK_ON", "BLOCK_MOSS_BREAK", "BLOCK_MOSS_FALL", "BLOCK_MOSS_HIT", "BLOCK_MOSS_PLACE", "BLOCK_MOSS_STEP", "BLOCK_MOSS_CARPET_BREAK", "BLOCK_MOSS_CARPET_FALL", "BLOCK_MOSS_CARPET_HIT", "BLOCK_MOSS_CARPET_PLACE", "BLOCK_MOSS_CARPET_STEP", "BLOCK_MUD_BREAK", "BLOCK_MUD_FALL", "BLOCK_MUD_HIT", "BLOCK_MUD_PLACE", "BLOCK_MUD_STEP", "BLOCK_MUD_BRICKS_BREAK", "BLOCK_MUD_BRICKS_FALL", "BLOCK_MUD_BRICKS_HIT", "BLOCK_MUD_BRICKS_PLACE", "BLOCK_MUD_BRICKS_STEP", "BLOCK_MUDDY_MANGROVE_ROOTS_BREAK", "BLOCK_MUDDY_MANGROVE_ROOTS_FALL", "BLOCK_MUDDY_MANGROVE_ROOTS_HIT", "BLOCK_MUDDY_MANGROVE_ROOTS_PLACE", "BLOCK_MUDDY_MANGROVE_ROOTS_STEP", "BLOCK_NETHER_BRICKS_BREAK", "BLOCK_NETHER_BRICKS_FALL", "BLOCK_NETHER_BRICKS_HIT", "BLOCK_NETHER_BRICKS_PLACE", "BLOCK_NETHER_BRICKS_STEP", "BLOCK_NETHER_GOLD_ORE_BREAK", "BLOCK_NETHER_GOLD_ORE_FALL", "BLOCK_NETHER_GOLD_ORE_HIT", "BLOCK_NETHER_GOLD_ORE_PLACE", "BLOCK_NETHER_GOLD_ORE_STEP", "BLOCK_NETHER_ORE_BREAK", "BLOCK_NETHER_ORE_FALL", "BLOCK_NETHER_ORE_HIT", "BLOCK_NETHER_ORE_PLACE", "BLOCK_NETHER_ORE_STEP", "BLOCK_NETHER_SPROUTS_BREAK", "BLOCK_NETHER_SPROUTS_FALL", "BLOCK_NETHER_SPROUTS_HIT", "BLOCK_NETHER_SPROUTS_PLACE", "BLOCK_NETHER_SPROUTS_STEP", "BLOCK_NETHER_WART_BREAK", "BLOCK_NETHER_WOOD_BREAK", "BLOCK_NETHER_WOOD_FALL", "BLOCK_NETHER_WOOD_HIT", "BLOCK_NETHER_WOOD_PLACE", "BLOCK_NETHER_WOOD_STEP", "BLOCK_NETHER_WOOD_BUTTON_CLICK_OFF", "BLOCK_NETHER_WOOD_BUTTON_CLICK_ON", "BLOCK_NETHER_WOOD_DOOR_CLOSE", "BLOCK_NETHER_WOOD_DOOR_OPEN", "BLOCK_NETHER_WOOD_FENCE_GATE_CLOSE", "BLOCK_NETHER_WOOD_FENCE_GATE_OPEN", "BLOCK_NETHER_WOOD_HANGING_SIGN_BREAK", "BLOCK_NETHER_WOOD_HANGING_SIGN_FALL", "BLOCK_NETHER_WOOD_HANGING_SIGN_HIT", "BLOCK_NETHER_WOOD_HANGING_SIGN_PLACE", "BLOCK_NETHER_WOOD_HANGING_SIGN_STEP", "BLOCK_NETHER_WOOD_PRESSURE_PLATE_CLICK_OFF", "BLOCK_NETHER_WOOD_PRESSURE_PLATE_CLICK_ON", "BLOCK_NETHER_WOOD_TRAPDOOR_CLOSE", "BLOCK_NETHER_WOOD_TRAPDOOR_OPEN", "BLOCK_NETHERITE_BLOCK_BREAK", "BLOCK_NETHERITE_BLOCK_FALL", "BLOCK_NETHERITE_BLOCK_HIT", "BLOCK_NETHERITE_BLOCK_PLACE", "BLOCK_NETHERITE_BLOCK_STEP", "BLOCK_NETHERRACK_BREAK", "BLOCK_NETHERRACK_FALL", "BLOCK_NETHERRACK_HIT", "BLOCK_NETHERRACK_PLACE", "BLOCK_NETHERRACK_STEP", "BLOCK_NOTE_BLOCK_BANJO", "BLOCK_NOTE_BLOCK_BASEDRUM", "BLOCK_NOTE_BLOCK_BASS", "BLOCK_NOTE_BLOCK_BELL", "BLOCK_NOTE_BLOCK_BIT", "BLOCK_NOTE_BLOCK_CHIME", "BLOCK_NOTE_BLOCK_COW_BELL", "BLOCK_NOTE_BLOCK_DIDGERIDOO", "BLOCK_NOTE_BLOCK_FLUTE", "BLOCK_NOTE_BLOCK_GUITAR", "BLOCK_NOTE_BLOCK_HARP", "BLOCK_NOTE_BLOCK_HAT", "BLOCK_NOTE_BLOCK_IMITATE_CREEPER", "BLOCK_NOTE_BLOCK_IMITATE_ENDER_DRAGON", "BLOCK_NOTE_BLOCK_IMITATE_PIGLIN", "BLOCK_NOTE_BLOCK_IMITATE_SKELETON", "BLOCK_NOTE_BLOCK_IMITATE_WITHER_SKELETON", "BLOCK_NOTE_BLOCK_IMITATE_ZOMBIE", "BLOCK_NOTE_BLOCK_IRON_XYLOPHONE", "BLOCK_NOTE_BLOCK_PLING", "BLOCK_NOTE_BLOCK_SNARE", "BLOCK_NOTE_BLOCK_TRUMPET", "BLOCK_NOTE_BLOCK_TRUMPET_EXPOSED", "BLOCK_NOTE_BLOCK_TRUMPET_OXIDIZED", "BLOCK_NOTE_BLOCK_TRUMPET_WEATHERED", "BLOCK_NOTE_BLOCK_XYLOPHONE", "BLOCK_NYLIUM_BREAK", "BLOCK_NYLIUM_FALL", "BLOCK_NYLIUM_HIT", "BLOCK_NYLIUM_PLACE", "BLOCK_NYLIUM_STEP", "BLOCK_PACKED_MUD_BREAK", "BLOCK_PACKED_MUD_FALL", "BLOCK_PACKED_MUD_HIT", "BLOCK_PACKED_MUD_PLACE", "BLOCK_PACKED_MUD_STEP", "BLOCK_PALE_HANGING_MOSS_IDLE", "BLOCK_PINK_PETALS_BREAK", "BLOCK_PINK_PETALS_FALL", "BLOCK_PINK_PETALS_HIT", "BLOCK_PINK_PETALS_PLACE", "BLOCK_PINK_PETALS_STEP", "BLOCK_PISTON_CONTRACT", "BLOCK_PISTON_EXTEND", "BLOCK_POINTED_DRIPSTONE_BREAK", "BLOCK_POINTED_DRIPSTONE_DRIP_LAVA", "BLOCK_POINTED_DRIPSTONE_DRIP_LAVA_INTO_CAULDRON", "BLOCK_POINTED_DRIPSTONE_DRIP_WATER", "BLOCK_POINTED_DRIPSTONE_DRIP_WATER_INTO_CAULDRON", "BLOCK_POINTED_DRIPSTONE_FALL", "BLOCK_POINTED_DRIPSTONE_HIT", "BLOCK_POINTED_DRIPSTONE_LAND", "BLOCK_POINTED_DRIPSTONE_PLACE", "BLOCK_POINTED_DRIPSTONE_STEP", "BLOCK_POLISHED_DEEPSLATE_BREAK", "BLOCK_POLISHED_DEEPSLATE_FALL", "BLOCK_POLISHED_DEEPSLATE_HIT", "BLOCK_POLISHED_DEEPSLATE_PLACE", "BLOCK_POLISHED_DEEPSLATE_STEP", "BLOCK_POLISHED_TUFF_BREAK", "BLOCK_POLISHED_TUFF_FALL", "BLOCK_POLISHED_TUFF_HIT", "BLOCK_POLISHED_TUFF_PLACE", "BLOCK_POLISHED_TUFF_STEP", "BLOCK_POPLAR_LEAVES_AMBIENT", "BLOCK_POPLAR_LEAVES_BREAK", "BLOCK_POPLAR_LEAVES_FALL", "BLOCK_POPLAR_LEAVES_HIT", "BLOCK_POPLAR_LEAVES_PLACE", "BLOCK_POPLAR_LEAVES_STEP", "BLOCK_PORTAL_AMBIENT", "BLOCK_PORTAL_TRAVEL", "BLOCK_PORTAL_TRIGGER", "BLOCK_POTENT_SULFUR_BREAK", "BLOCK_POTENT_SULFUR_FALL", "BLOCK_POTENT_SULFUR_GEYSER_CONTINUOUS_ERUPTION", "BLOCK_POTENT_SULFUR_GEYSER_CONTINUOUS_ERUPTION_ACTIVE", "BLOCK_POTENT_SULFUR_GEYSER_ERUPTION", "BLOCK_POTENT_SULFUR_GEYSER_ERUPTION_ACTIVE", "BLOCK_POTENT_SULFUR_HIT", "BLOCK_POTENT_SULFUR_NOXIOUS_GAS", "BLOCK_POTENT_SULFUR_PLACE", "BLOCK_POTENT_SULFUR_STEP", "BLOCK_POWDER_SNOW_BREAK", "BLOCK_POWDER_SNOW_FALL", "BLOCK_POWDER_SNOW_HIT", "BLOCK_POWDER_SNOW_PLACE", "BLOCK_POWDER_SNOW_STEP", "BLOCK_PUMPKIN_CARVE", "BLOCK_RED_SHRUB_BREAK", "BLOCK_RED_SHRUB_PLACE", "BLOCK_REDSTONE_TORCH_BURNOUT", "BLOCK_RESIN_BREAK", "BLOCK_RESIN_FALL", "BLOCK_RESIN_PLACE", "BLOCK_RESIN_STEP", "BLOCK_RESIN_BRICKS_BREAK", "BLOCK_RESIN_BRICKS_FALL", "BLOCK_RESIN_BRICKS_HIT", "BLOCK_RESIN_BRICKS_PLACE", "BLOCK_RESIN_BRICKS_STEP", "BLOCK_RESPAWN_ANCHOR_AMBIENT", "BLOCK_RESPAWN_ANCHOR_CHARGE", "BLOCK_RESPAWN_ANCHOR_DEPLETE", "BLOCK_RESPAWN_ANCHOR_SET_SPAWN", "BLOCK_ROOTED_DIRT_BREAK", "BLOCK_ROOTED_DIRT_FALL", "BLOCK_ROOTED_DIRT_HIT", "BLOCK_ROOTED_DIRT_PLACE", "BLOCK_ROOTED_DIRT_STEP", "BLOCK_ROOTS_BREAK", "BLOCK_ROOTS_FALL", "BLOCK_ROOTS_HIT", "BLOCK_ROOTS_PLACE", "BLOCK_ROOTS_STEP", "BLOCK_SAND_BREAK", "BLOCK_SAND_FALL", "BLOCK_SAND_HIT", "BLOCK_SAND_IDLE", "BLOCK_SAND_PLACE", "BLOCK_SAND_STEP", "BLOCK_SCAFFOLDING_BREAK", "BLOCK_SCAFFOLDING_FALL", "BLOCK_SCAFFOLDING_HIT", "BLOCK_SCAFFOLDING_PLACE", "BLOCK_SCAFFOLDING_STEP", "BLOCK_SCULK_BREAK", "BLOCK_SCULK_CHARGE", "BLOCK_SCULK_FALL", "BLOCK_SCULK_HIT", "BLOCK_SCULK_PLACE", "BLOCK_SCULK_SPREAD", "BLOCK_SCULK_STEP", "BLOCK_SCULK_CATALYST_BLOOM", "BLOCK_SCULK_CATALYST_BREAK", "BLOCK_SCULK_CATALYST_FALL", "BLOCK_SCULK_CATALYST_HIT", "BLOCK_SCULK_CATALYST_PLACE", "BLOCK_SCULK_CATALYST_STEP", "BLOCK_SCULK_SENSOR_BREAK", "BLOCK_SCULK_SENSOR_CLICKING", "BLOCK_SCULK_SENSOR_CLICKING_STOP", "BLOCK_SCULK_SENSOR_FALL", "BLOCK_SCULK_SENSOR_HIT", "BLOCK_SCULK_SENSOR_PLACE", "BLOCK_SCULK_SENSOR_STEP", "BLOCK_SCULK_SHRIEKER_BREAK", "BLOCK_SCULK_SHRIEKER_FALL", "BLOCK_SCULK_SHRIEKER_HIT", "BLOCK_SCULK_SHRIEKER_PLACE", "BLOCK_SCULK_SHRIEKER_SHRIEK", "BLOCK_SCULK_SHRIEKER_STEP", "BLOCK_SCULK_VEIN_BREAK", "BLOCK_SCULK_VEIN_FALL", "BLOCK_SCULK_VEIN_HIT", "BLOCK_SCULK_VEIN_PLACE", "BLOCK_SCULK_VEIN_STEP", "BLOCK_SHELF_ACTIVATE", "BLOCK_SHELF_BREAK", "BLOCK_SHELF_DEACTIVATE", "BLOCK_SHELF_FALL", "BLOCK_SHELF_HIT", "BLOCK_SHELF_MULTI_SWAP", "BLOCK_SHELF_PLACE", "BLOCK_SHELF_PLACE_ITEM", "BLOCK_SHELF_SINGLE_SWAP", "BLOCK_SHELF_STEP", "BLOCK_SHELF_TAKE_ITEM", "BLOCK_SHELF_MUSHROOM_BOUNCE", "BLOCK_SHELF_MUSHROOM_BREAK", "BLOCK_SHELF_MUSHROOM_FALL", "BLOCK_SHELF_MUSHROOM_PLACE", "BLOCK_SHELF_MUSHROOM_STEP", "BLOCK_SHROOMLIGHT_BREAK", "BLOCK_SHROOMLIGHT_FALL", "BLOCK_SHROOMLIGHT_HIT", "BLOCK_SHROOMLIGHT_PLACE", "BLOCK_SHROOMLIGHT_STEP", "BLOCK_SHULKER_BOX_CLOSE", "BLOCK_SHULKER_BOX_OPEN", "BLOCK_SIGN_WAXED_INTERACT_FAIL", "BLOCK_SLIME_BLOCK_BREAK", "BLOCK_SLIME_BLOCK_FALL", "BLOCK_SLIME_BLOCK_HIT", "BLOCK_SLIME_BLOCK_PLACE", "BLOCK_SLIME_BLOCK_STEP", "BLOCK_SMALL_AMETHYST_BUD_BREAK", "BLOCK_SMALL_AMETHYST_BUD_PLACE", "BLOCK_SMALL_DRIPLEAF_BREAK", "BLOCK_SMALL_DRIPLEAF_FALL", "BLOCK_SMALL_DRIPLEAF_HIT", "BLOCK_SMALL_DRIPLEAF_PLACE", "BLOCK_SMALL_DRIPLEAF_STEP", "BLOCK_SMITHING_TABLE_USE", "BLOCK_SMOKER_SMOKE", "BLOCK_SNIFFER_EGG_CRACK", "BLOCK_SNIFFER_EGG_HATCH", "BLOCK_SNIFFER_EGG_PLOP", "BLOCK_SNOW_BREAK", "BLOCK_SNOW_FALL", "BLOCK_SNOW_HIT", "BLOCK_SNOW_PLACE", "BLOCK_SNOW_STEP", "BLOCK_SOUL_SAND_BREAK", "BLOCK_SOUL_SAND_FALL", "BLOCK_SOUL_SAND_HIT", "BLOCK_SOUL_SAND_PLACE", "BLOCK_SOUL_SAND_STEP", "BLOCK_SOUL_SOIL_BREAK", "BLOCK_SOUL_SOIL_FALL", "BLOCK_SOUL_SOIL_HIT", "BLOCK_SOUL_SOIL_PLACE", "BLOCK_SOUL_SOIL_STEP", "BLOCK_SPAWNER_BREAK", "BLOCK_SPAWNER_FALL", "BLOCK_SPAWNER_HIT", "BLOCK_SPAWNER_PLACE", "BLOCK_SPAWNER_STEP", "BLOCK_SPONGE_ABSORB", "BLOCK_SPONGE_BREAK", "BLOCK_SPONGE_FALL", "BLOCK_SPONGE_HIT", "BLOCK_SPONGE_PLACE", "BLOCK_SPONGE_STEP", "BLOCK_SPORE_BLOSSOM_BREAK", "BLOCK_SPORE_BLOSSOM_FALL", "BLOCK_SPORE_BLOSSOM_HIT", "BLOCK_SPORE_BLOSSOM_PLACE", "BLOCK_SPORE_BLOSSOM_STEP", "BLOCK_STEM_BREAK", "BLOCK_STEM_FALL", "BLOCK_STEM_HIT", "BLOCK_STEM_PLACE", "BLOCK_STEM_STEP", "BLOCK_STONE_BREAK", "BLOCK_STONE_FALL", "BLOCK_STONE_HIT", "BLOCK_STONE_PLACE", "BLOCK_STONE_STEP", "BLOCK_STONE_BUTTON_CLICK_OFF", "BLOCK_STONE_BUTTON_CLICK_ON", "BLOCK_STONE_PRESSURE_PLATE_CLICK_OFF", "BLOCK_STONE_PRESSURE_PLATE_CLICK_ON", "BLOCK_STRAW_BED_BREAK", "BLOCK_STRAW_BED_BREAK_LEAVE", "BLOCK_STRAW_BED_FALL", "BLOCK_STRAW_BED_HIT", "BLOCK_STRAW_BED_PLACE", "BLOCK_STRAW_BED_STEP", "BLOCK_SULFUR_BREAK", "BLOCK_SULFUR_FALL", "BLOCK_SULFUR_HIT", "BLOCK_SULFUR_PLACE", "BLOCK_SULFUR_STEP", "BLOCK_SULFUR_SPIKE_BREAK", "BLOCK_SULFUR_SPIKE_FALL", "BLOCK_SULFUR_SPIKE_HIT", "BLOCK_SULFUR_SPIKE_LAND", "BLOCK_SULFUR_SPIKE_PLACE", "BLOCK_SULFUR_SPIKE_STEP", "BLOCK_SUSPICIOUS_GRAVEL_BREAK", "BLOCK_SUSPICIOUS_GRAVEL_FALL", "BLOCK_SUSPICIOUS_GRAVEL_HIT", "BLOCK_SUSPICIOUS_GRAVEL_PLACE", "BLOCK_SUSPICIOUS_GRAVEL_STEP", "BLOCK_SUSPICIOUS_SAND_BREAK", "BLOCK_SUSPICIOUS_SAND_FALL", "BLOCK_SUSPICIOUS_SAND_HIT", "BLOCK_SUSPICIOUS_SAND_PLACE", "BLOCK_SUSPICIOUS_SAND_STEP", "BLOCK_SWEET_BERRY_BUSH_BREAK", "BLOCK_SWEET_BERRY_BUSH_PICK_BERRIES", "BLOCK_SWEET_BERRY_BUSH_PLACE", "BLOCK_TRIAL_SPAWNER_ABOUT_TO_SPAWN_ITEM", "BLOCK_TRIAL_SPAWNER_AMBIENT", "BLOCK_TRIAL_SPAWNER_AMBIENT_OMINOUS", "BLOCK_TRIAL_SPAWNER_BREAK", "BLOCK_TRIAL_SPAWNER_CLOSE_SHUTTER", "BLOCK_TRIAL_SPAWNER_DETECT_PLAYER", "BLOCK_TRIAL_SPAWNER_EJECT_ITEM", "BLOCK_TRIAL_SPAWNER_FALL", "BLOCK_TRIAL_SPAWNER_HIT", "BLOCK_TRIAL_SPAWNER_OMINOUS_ACTIVATE", "BLOCK_TRIAL_SPAWNER_OPEN_SHUTTER", "BLOCK_TRIAL_SPAWNER_PLACE", "BLOCK_TRIAL_SPAWNER_SPAWN_ITEM", "BLOCK_TRIAL_SPAWNER_SPAWN_ITEM_BEGIN", "BLOCK_TRIAL_SPAWNER_SPAWN_MOB", "BLOCK_TRIAL_SPAWNER_STEP", "BLOCK_TRIPWIRE_ATTACH", "BLOCK_TRIPWIRE_CLICK_OFF", "BLOCK_TRIPWIRE_CLICK_ON", "BLOCK_TRIPWIRE_DETACH", "BLOCK_TUFF_BREAK", "BLOCK_TUFF_FALL", "BLOCK_TUFF_HIT", "BLOCK_TUFF_PLACE", "BLOCK_TUFF_STEP", "BLOCK_TUFF_BRICKS_BREAK", "BLOCK_TUFF_BRICKS_FALL", "BLOCK_TUFF_BRICKS_HIT", "BLOCK_TUFF_BRICKS_PLACE", "BLOCK_TUFF_BRICKS_STEP", "BLOCK_VAULT_ACTIVATE", "BLOCK_VAULT_AMBIENT", "BLOCK_VAULT_BREAK", "BLOCK_VAULT_CLOSE_SHUTTER", "BLOCK_VAULT_DEACTIVATE", "BLOCK_VAULT_EJECT_ITEM", "BLOCK_VAULT_FALL", "BLOCK_VAULT_HIT", "BLOCK_VAULT_INSERT_ITEM", "BLOCK_VAULT_INSERT_ITEM_FAIL", "BLOCK_VAULT_OPEN_SHUTTER", "BLOCK_VAULT_PLACE", "BLOCK_VAULT_REJECT_REWARDED_PLAYER", "BLOCK_VAULT_STEP", "BLOCK_VINE_BREAK", "BLOCK_VINE_FALL", "BLOCK_VINE_HIT", "BLOCK_VINE_PLACE", "BLOCK_VINE_STEP", "BLOCK_WART_BLOCK_BREAK", "BLOCK_WART_BLOCK_FALL", "BLOCK_WART_BLOCK_HIT", "BLOCK_WART_BLOCK_PLACE", "BLOCK_WART_BLOCK_STEP", "BLOCK_WATER_AMBIENT", "BLOCK_WEEPING_VINES_BREAK", "BLOCK_WEEPING_VINES_FALL", "BLOCK_WEEPING_VINES_HIT", "BLOCK_WEEPING_VINES_PLACE", "BLOCK_WEEPING_VINES_STEP", "BLOCK_WET_GRASS_BREAK", "BLOCK_WET_GRASS_FALL", "BLOCK_WET_GRASS_HIT", "BLOCK_WET_GRASS_PLACE", "BLOCK_WET_GRASS_STEP", "BLOCK_WET_SPONGE_BREAK", "BLOCK_WET_SPONGE_DRIES", "BLOCK_WET_SPONGE_FALL", "BLOCK_WET_SPONGE_HIT", "BLOCK_WET_SPONGE_PLACE", "BLOCK_WET_SPONGE_STEP", "BLOCK_WOOD_BREAK", "BLOCK_WOOD_FALL", "BLOCK_WOOD_HIT", "BLOCK_WOOD_PLACE", "BLOCK_WOOD_STEP", "BLOCK_WOODEN_BUTTON_CLICK_OFF", "BLOCK_WOODEN_BUTTON_CLICK_ON", "BLOCK_WOODEN_DOOR_CLOSE", "BLOCK_WOODEN_DOOR_OPEN", "BLOCK_WOODEN_PRESSURE_PLATE_CLICK_OFF", "BLOCK_WOODEN_PRESSURE_PLATE_CLICK_ON", "BLOCK_WOODEN_TRAPDOOR_CLOSE", "BLOCK_WOODEN_TRAPDOOR_OPEN", "BLOCK_WOOL_BREAK", "BLOCK_WOOL_FALL", "BLOCK_WOOL_HIT", "BLOCK_WOOL_PLACE", "BLOCK_WOOL_STEP", "ENCHANT_THORNS_HIT", "ENTITY_ALLAY_AMBIENT_WITH_ITEM", "ENTITY_ALLAY_AMBIENT_WITHOUT_ITEM", "ENTITY_ALLAY_DEATH", "ENTITY_ALLAY_HURT", "ENTITY_ALLAY_ITEM_GIVEN", "ENTITY_ALLAY_ITEM_TAKEN", "ENTITY_ALLAY_ITEM_THROWN", "ENTITY_ARMADILLO_AMBIENT", "ENTITY_ARMADILLO_BRUSH", "ENTITY_ARMADILLO_DEATH", "ENTITY_ARMADILLO_EAT", "ENTITY_ARMADILLO_HURT", "ENTITY_ARMADILLO_HURT_REDUCED", "ENTITY_ARMADILLO_LAND", "ENTITY_ARMADILLO_PEEK", "ENTITY_ARMADILLO_ROLL", "ENTITY_ARMADILLO_SCUTE_DROP", "ENTITY_ARMADILLO_STEP", "ENTITY_ARMADILLO_UNROLL_FINISH", "ENTITY_ARMADILLO_UNROLL_START", "ENTITY_ARMOR_STAND_BREAK", "ENTITY_ARMOR_STAND_FALL", "ENTITY_ARMOR_STAND_HIT", "ENTITY_ARMOR_STAND_PLACE", "ENTITY_ARROW_HIT", "ENTITY_ARROW_HIT_PLAYER", "ENTITY_ARROW_SHOOT", "ENTITY_AXOLOTL_ATTACK", "ENTITY_AXOLOTL_DEATH", "ENTITY_AXOLOTL_HURT", "ENTITY_AXOLOTL_IDLE_AIR", "ENTITY_AXOLOTL_IDLE_WATER", "ENTITY_AXOLOTL_SPLASH", "ENTITY_AXOLOTL_SWIM", "ENTITY_BABY_CAT_AMBIENT", "ENTITY_BABY_CAT_BEG_FOR_FOOD", "ENTITY_BABY_CAT_DEATH", "ENTITY_BABY_CAT_EAT", "ENTITY_BABY_CAT_HISS", "ENTITY_BABY_CAT_HURT", "ENTITY_BABY_CAT_PURR", "ENTITY_BABY_CAT_PURREOW", "ENTITY_BABY_CAT_STRAY_AMBIENT", "ENTITY_BABY_CHICKEN_AMBIENT", "ENTITY_BABY_CHICKEN_DEATH", "ENTITY_BABY_CHICKEN_HURT", "ENTITY_BABY_CHICKEN_STEP", "ENTITY_BABY_HORSE_AMBIENT", "ENTITY_BABY_HORSE_ANGRY", "ENTITY_BABY_HORSE_BREATHE", "ENTITY_BABY_HORSE_DEATH", "ENTITY_BABY_HORSE_EAT", "ENTITY_BABY_HORSE_HURT", "ENTITY_BABY_HORSE_LAND", "ENTITY_BABY_HORSE_STEP", "ENTITY_BABY_NAUTILUS_AMBIENT", "ENTITY_BABY_NAUTILUS_AMBIENT_LAND", "ENTITY_BABY_NAUTILUS_DEATH", "ENTITY_BABY_NAUTILUS_DEATH_LAND", "ENTITY_BABY_NAUTILUS_EAT", "ENTITY_BABY_NAUTILUS_HURT", "ENTITY_BABY_NAUTILUS_HURT_LAND", "ENTITY_BABY_NAUTILUS_SWIM", "ENTITY_BABY_PIG_AMBIENT", "ENTITY_BABY_PIG_DEATH", "ENTITY_BABY_PIG_EAT", "ENTITY_BABY_PIG_HURT", "ENTITY_BABY_PIG_STEP", "ENTITY_BABY_WOLF_AMBIENT", "ENTITY_BABY_WOLF_DEATH", "ENTITY_BABY_WOLF_GROWL", "ENTITY_BABY_WOLF_HURT", "ENTITY_BABY_WOLF_PANT", "ENTITY_BABY_WOLF_STEP", "ENTITY_BABY_WOLF_WHINE", "ENTITY_BAT_AMBIENT", "ENTITY_BAT_DEATH", "ENTITY_BAT_HURT", "ENTITY_BAT_LOOP", "ENTITY_BAT_TAKEOFF", "ENTITY_BEE_DEATH", "ENTITY_BEE_HURT", "ENTITY_BEE_LOOP", "ENTITY_BEE_LOOP_AGGRESSIVE", "ENTITY_BEE_POLLINATE", "ENTITY_BEE_STING", "ENTITY_BLAZE_AMBIENT", "ENTITY_BLAZE_BURN", "ENTITY_BLAZE_DEATH", "ENTITY_BLAZE_HURT", "ENTITY_BLAZE_SHOOT", "ENTITY_BOAT_PADDLE_LAND", "ENTITY_BOAT_PADDLE_WATER", "ENTITY_BOGGED_AMBIENT", "ENTITY_BOGGED_DEATH", "ENTITY_BOGGED_HURT", "ENTITY_BOGGED_SHEAR", "ENTITY_BOGGED_STEP", "ENTITY_BREEZE_CHARGE", "ENTITY_BREEZE_DEATH", "ENTITY_BREEZE_DEFLECT", "ENTITY_BREEZE_HURT", "ENTITY_BREEZE_IDLE_AIR", "ENTITY_BREEZE_IDLE_GROUND", "ENTITY_BREEZE_INHALE", "ENTITY_BREEZE_JUMP", "ENTITY_BREEZE_LAND", "ENTITY_BREEZE_SHOOT", "ENTITY_BREEZE_SLIDE", "ENTITY_BREEZE_WHIRL", "ENTITY_BREEZE_WIND_BURST", "ENTITY_CAMEL_AMBIENT", "ENTITY_CAMEL_DASH", "ENTITY_CAMEL_DASH_READY", "ENTITY_CAMEL_DEATH", "ENTITY_CAMEL_EAT", "ENTITY_CAMEL_HURT", "ENTITY_CAMEL_SADDLE", "ENTITY_CAMEL_SIT", "ENTITY_CAMEL_STAND", "ENTITY_CAMEL_STEP", "ENTITY_CAMEL_STEP_SAND", "ENTITY_CAMEL_HUSK_AMBIENT", "ENTITY_CAMEL_HUSK_DASH", "ENTITY_CAMEL_HUSK_DASH_READY", "ENTITY_CAMEL_HUSK_DEATH", "ENTITY_CAMEL_HUSK_EAT", "ENTITY_CAMEL_HUSK_HURT", "ENTITY_CAMEL_HUSK_SADDLE", "ENTITY_CAMEL_HUSK_SIT", "ENTITY_CAMEL_HUSK_STAND", "ENTITY_CAMEL_HUSK_STEP", "ENTITY_CAMEL_HUSK_STEP_SAND", "ENTITY_CAT_AMBIENT", "ENTITY_CAT_BEG_FOR_FOOD", "ENTITY_CAT_DEATH", "ENTITY_CAT_EAT", "ENTITY_CAT_HISS", "ENTITY_CAT_HURT", "ENTITY_CAT_PURR", "ENTITY_CAT_PURREOW", "ENTITY_CAT_STRAY_AMBIENT", "ENTITY_CAT_ROYAL_AMBIENT", "ENTITY_CAT_ROYAL_BEG_FOR_FOOD", "ENTITY_CAT_ROYAL_DEATH", "ENTITY_CAT_ROYAL_EAT", "ENTITY_CAT_ROYAL_HISS", "ENTITY_CAT_ROYAL_HURT", "ENTITY_CAT_ROYAL_PURR", "ENTITY_CAT_ROYAL_PURREOW", "ENTITY_CAT_ROYAL_STRAY_AMBIENT", "ENTITY_CHICKEN_AMBIENT", "ENTITY_CHICKEN_DEATH", "ENTITY_CHICKEN_EGG", "ENTITY_CHICKEN_HURT", "ENTITY_CHICKEN_STEP", "ENTITY_CHICKEN_PICKY_AMBIENT", "ENTITY_CHICKEN_PICKY_DEATH", "ENTITY_CHICKEN_PICKY_HURT", "ENTITY_COD_AMBIENT", "ENTITY_COD_DEATH", "ENTITY_COD_FLOP", "ENTITY_COD_HURT", "ENTITY_COPPER_GOLEM_DEATH", "ENTITY_COPPER_GOLEM_HURT", "ENTITY_COPPER_GOLEM_ITEM_DROP", "ENTITY_COPPER_GOLEM_ITEM_NO_DROP", "ENTITY_COPPER_GOLEM_NO_ITEM_GET", "ENTITY_COPPER_GOLEM_NO_ITEM_NO_GET", "ENTITY_COPPER_GOLEM_SHEAR", "ENTITY_COPPER_GOLEM_SPAWN", "ENTITY_COPPER_GOLEM_SPIN", "ENTITY_COPPER_GOLEM_STEP", "ENTITY_COPPER_GOLEM_BECOME_STATUE", "ENTITY_COPPER_GOLEM_OXIDIZED_DEATH", "ENTITY_COPPER_GOLEM_OXIDIZED_HURT", "ENTITY_COPPER_GOLEM_OXIDIZED_SPIN", "ENTITY_COPPER_GOLEM_OXIDIZED_STEP", "ENTITY_COPPER_GOLEM_WEATHERED_DEATH", "ENTITY_COPPER_GOLEM_WEATHERED_HURT", "ENTITY_COPPER_GOLEM_WEATHERED_SPIN", "ENTITY_COPPER_GOLEM_WEATHERED_STEP", "ENTITY_COW_AMBIENT", "ENTITY_COW_DEATH", "ENTITY_COW_HURT", "ENTITY_COW_MILK", "ENTITY_COW_STEP", "ENTITY_COW_MOODY_AMBIENT", "ENTITY_COW_MOODY_DEATH", "ENTITY_COW_MOODY_HURT", "ENTITY_COW_MOODY_STEP", "ENTITY_CREAKING_ACTIVATE", "ENTITY_CREAKING_AMBIENT", "ENTITY_CREAKING_ATTACK", "ENTITY_CREAKING_DEACTIVATE", "ENTITY_CREAKING_DEATH", "ENTITY_CREAKING_FREEZE", "ENTITY_CREAKING_SPAWN", "ENTITY_CREAKING_STEP", "ENTITY_CREAKING_SWAY", "ENTITY_CREAKING_TWITCH", "ENTITY_CREAKING_UNFREEZE", "ENTITY_CREEPER_DEATH", "ENTITY_CREEPER_HURT", "ENTITY_CREEPER_PRIMED", "ENTITY_CUSHION_BREAK", "ENTITY_CUSHION_GET_UP", "ENTITY_CUSHION_PLACE", "ENTITY_CUSHION_SIT", "ENTITY_DOLPHIN_AMBIENT", "ENTITY_DOLPHIN_AMBIENT_WATER", "ENTITY_DOLPHIN_ATTACK", "ENTITY_DOLPHIN_DEATH", "ENTITY_DOLPHIN_EAT", "ENTITY_DOLPHIN_HURT", "ENTITY_DOLPHIN_JUMP", "ENTITY_DOLPHIN_PLAY", "ENTITY_DOLPHIN_SPLASH", "ENTITY_DOLPHIN_SWIM", "ENTITY_DONKEY_AMBIENT", "ENTITY_DONKEY_ANGRY", "ENTITY_DONKEY_CHEST", "ENTITY_DONKEY_DEATH", "ENTITY_DONKEY_EAT", "ENTITY_DONKEY_HURT", "ENTITY_DONKEY_JUMP", "ENTITY_DRAGON_FIREBALL_EXPLODE", "ENTITY_DROWNED_AMBIENT", "ENTITY_DROWNED_AMBIENT_WATER", "ENTITY_DROWNED_DEATH", "ENTITY_DROWNED_DEATH_WATER", "ENTITY_DROWNED_HURT", "ENTITY_DROWNED_HURT_WATER", "ENTITY_DROWNED_SHOOT", "ENTITY_DROWNED_STEP", "ENTITY_DROWNED_SWIM", "ENTITY_EGG_THROW", "ENTITY_ELDER_GUARDIAN_AMBIENT", "ENTITY_ELDER_GUARDIAN_AMBIENT_LAND", "ENTITY_ELDER_GUARDIAN_CURSE", "ENTITY_ELDER_GUARDIAN_DEATH", "ENTITY_ELDER_GUARDIAN_DEATH_LAND", "ENTITY_ELDER_GUARDIAN_FLOP", "ENTITY_ELDER_GUARDIAN_HURT", "ENTITY_ELDER_GUARDIAN_HURT_LAND", "ENTITY_ENDER_DRAGON_AMBIENT", "ENTITY_ENDER_DRAGON_DEATH", "ENTITY_ENDER_DRAGON_FLAP", "ENTITY_ENDER_DRAGON_GROWL", "ENTITY_ENDER_DRAGON_HURT", "ENTITY_ENDER_DRAGON_SHOOT", "ENTITY_ENDER_EYE_DEATH", "ENTITY_ENDER_EYE_LAUNCH", "ENTITY_ENDER_PEARL_THROW", "ENTITY_ENDERMAN_AMBIENT", "ENTITY_ENDERMAN_DEATH", "ENTITY_ENDERMAN_HURT", "ENTITY_ENDERMAN_SCREAM", "ENTITY_ENDERMAN_STARE", "ENTITY_ENDERMAN_TELEPORT", "ENTITY_ENDERMITE_AMBIENT", "ENTITY_ENDERMITE_DEATH", "ENTITY_ENDERMITE_HURT", "ENTITY_ENDERMITE_STEP", "ENTITY_EVOKER_AMBIENT", "ENTITY_EVOKER_CAST_SPELL", "ENTITY_EVOKER_CELEBRATE", "ENTITY_EVOKER_DEATH", "ENTITY_EVOKER_HURT", "ENTITY_EVOKER_PREPARE_ATTACK", "ENTITY_EVOKER_PREPARE_SUMMON", "ENTITY_EVOKER_PREPARE_WOLOLO", "ENTITY_EVOKER_FANGS_ATTACK", "ENTITY_EXPERIENCE_BOTTLE_THROW", "ENTITY_EXPERIENCE_ORB_PICKUP", "ENTITY_FIREWORK_ROCKET_BLAST", "ENTITY_FIREWORK_ROCKET_BLAST_FAR", "ENTITY_FIREWORK_ROCKET_LARGE_BLAST", "ENTITY_FIREWORK_ROCKET_LARGE_BLAST_FAR", "ENTITY_FIREWORK_ROCKET_LAUNCH", "ENTITY_FIREWORK_ROCKET_SHOOT", "ENTITY_FIREWORK_ROCKET_TWINKLE", "ENTITY_FIREWORK_ROCKET_TWINKLE_FAR", "ENTITY_FISH_SWIM", "ENTITY_FISHING_BOBBER_RETRIEVE", "ENTITY_FISHING_BOBBER_SPLASH", "ENTITY_FISHING_BOBBER_THROW", "ENTITY_FOX_AGGRO", "ENTITY_FOX_AMBIENT", "ENTITY_FOX_BITE", "ENTITY_FOX_DEATH", "ENTITY_FOX_EAT", "ENTITY_FOX_HURT", "ENTITY_FOX_SCREECH", "ENTITY_FOX_SLEEP", "ENTITY_FOX_SNIFF", "ENTITY_FOX_SPIT", "ENTITY_FOX_TELEPORT", "ENTITY_FROG_AMBIENT", "ENTITY_FROG_DEATH", "ENTITY_FROG_EAT", "ENTITY_FROG_HURT", "ENTITY_FROG_LAY_SPAWN", "ENTITY_FROG_LONG_JUMP", "ENTITY_FROG_STEP", "ENTITY_FROG_TONGUE", "ENTITY_GENERIC_BIG_FALL", "ENTITY_GENERIC_BURN", "ENTITY_GENERIC_DEATH", "ENTITY_GENERIC_DRINK", "ENTITY_GENERIC_EAT", "ENTITY_GENERIC_EXPLODE", "ENTITY_GENERIC_EXTINGUISH_FIRE", "ENTITY_GENERIC_HURT", "ENTITY_GENERIC_SMALL_FALL", "ENTITY_GENERIC_SPLASH", "ENTITY_GENERIC_SWIM", "ENTITY_GHAST_AMBIENT", "ENTITY_GHAST_DEATH", "ENTITY_GHAST_HURT", "ENTITY_GHAST_SCREAM", "ENTITY_GHAST_SHOOT", "ENTITY_GHAST_WARN", "ENTITY_GHASTLING_AMBIENT", "ENTITY_GHASTLING_DEATH", "ENTITY_GHASTLING_HURT", "ENTITY_GHASTLING_SPAWN", "ENTITY_GLOW_ITEM_FRAME_ADD_ITEM", "ENTITY_GLOW_ITEM_FRAME_BREAK", "ENTITY_GLOW_ITEM_FRAME_PLACE", "ENTITY_GLOW_ITEM_FRAME_REMOVE_ITEM", "ENTITY_GLOW_ITEM_FRAME_ROTATE_ITEM", "ENTITY_GLOW_SQUID_AMBIENT", "ENTITY_GLOW_SQUID_DEATH", "ENTITY_GLOW_SQUID_HURT", "ENTITY_GLOW_SQUID_SQUIRT", "ENTITY_GOAT_AMBIENT", "ENTITY_GOAT_DEATH", "ENTITY_GOAT_EAT", "ENTITY_GOAT_HORN_BREAK", "ENTITY_GOAT_HURT", "ENTITY_GOAT_LONG_JUMP", "ENTITY_GOAT_MILK", "ENTITY_GOAT_PREPARE_RAM", "ENTITY_GOAT_RAM_IMPACT", "ENTITY_GOAT_SCREAMING_AMBIENT", "ENTITY_GOAT_SCREAMING_DEATH", "ENTITY_GOAT_SCREAMING_EAT", "ENTITY_GOAT_SCREAMING_HURT", "ENTITY_GOAT_SCREAMING_LONG_JUMP", "ENTITY_GOAT_SCREAMING_MILK", "ENTITY_GOAT_SCREAMING_PREPARE_RAM", "ENTITY_GOAT_SCREAMING_RAM_IMPACT", "ENTITY_GOAT_STEP", "ENTITY_GUARDIAN_AMBIENT", "ENTITY_GUARDIAN_AMBIENT_LAND", "ENTITY_GUARDIAN_ATTACK", "ENTITY_GUARDIAN_DEATH", "ENTITY_GUARDIAN_DEATH_LAND", "ENTITY_GUARDIAN_FLOP", "ENTITY_GUARDIAN_HURT", "ENTITY_GUARDIAN_HURT_LAND", "ENTITY_HAPPY_GHAST_AMBIENT", "ENTITY_HAPPY_GHAST_DEATH", "ENTITY_HAPPY_GHAST_EQUIP", "ENTITY_HAPPY_GHAST_HARNESS_GOGGLES_DOWN", "ENTITY_HAPPY_GHAST_HARNESS_GOGGLES_UP", "ENTITY_HAPPY_GHAST_HURT", "ENTITY_HAPPY_GHAST_RIDING", "ENTITY_HAPPY_GHAST_UNEQUIP", "ENTITY_HOGLIN_AMBIENT", "ENTITY_HOGLIN_ANGRY", "ENTITY_HOGLIN_ATTACK", "ENTITY_HOGLIN_CONVERTED_TO_ZOMBIFIED", "ENTITY_HOGLIN_DEATH", "ENTITY_HOGLIN_HURT", "ENTITY_HOGLIN_RETREAT", "ENTITY_HOGLIN_STEP", "ENTITY_HORSE_AMBIENT", "ENTITY_HORSE_ANGRY", "ENTITY_HORSE_ARMOR", "ENTITY_HORSE_BREATHE", "ENTITY_HORSE_DEATH", "ENTITY_HORSE_EAT", "ENTITY_HORSE_GALLOP", "ENTITY_HORSE_HURT", "ENTITY_HORSE_JUMP", "ENTITY_HORSE_LAND", "ENTITY_HORSE_SADDLE", "ENTITY_HORSE_STEP", "ENTITY_HORSE_STEP_WOOD", "ENTITY_HOSTILE_BIG_FALL", "ENTITY_HOSTILE_DEATH", "ENTITY_HOSTILE_HURT", "ENTITY_HOSTILE_SMALL_FALL", "ENTITY_HOSTILE_SPLASH", "ENTITY_HOSTILE_SWIM", "ENTITY_HUSK_AMBIENT", "ENTITY_HUSK_CONVERTED_TO_ZOMBIE", "ENTITY_HUSK_DEATH", "ENTITY_HUSK_HURT", "ENTITY_HUSK_STEP", "ENTITY_ILLUSIONER_AMBIENT", "ENTITY_ILLUSIONER_CAST_SPELL", "ENTITY_ILLUSIONER_DEATH", "ENTITY_ILLUSIONER_HURT", "ENTITY_ILLUSIONER_MIRROR_MOVE", "ENTITY_ILLUSIONER_PREPARE_BLINDNESS", "ENTITY_ILLUSIONER_PREPARE_MIRROR", "ENTITY_IRON_GOLEM_ATTACK", "ENTITY_IRON_GOLEM_DAMAGE", "ENTITY_IRON_GOLEM_DEATH", "ENTITY_IRON_GOLEM_HURT", "ENTITY_IRON_GOLEM_REPAIR", "ENTITY_IRON_GOLEM_STEP", "ENTITY_ITEM_BREAK", "ENTITY_ITEM_PICKUP", "ENTITY_ITEM_FRAME_ADD_ITEM", "ENTITY_ITEM_FRAME_BREAK", "ENTITY_ITEM_FRAME_PLACE", "ENTITY_ITEM_FRAME_REMOVE_ITEM", "ENTITY_ITEM_FRAME_ROTATE_ITEM", "ENTITY_LIGHTNING_BOLT_IMPACT", "ENTITY_LIGHTNING_BOLT_THUNDER", "ENTITY_LINGERING_POTION_THROW", "ENTITY_LLAMA_AMBIENT", "ENTITY_LLAMA_ANGRY", "ENTITY_LLAMA_CHEST", "ENTITY_LLAMA_DEATH", "ENTITY_LLAMA_EAT", "ENTITY_LLAMA_HURT", "ENTITY_LLAMA_SPIT", "ENTITY_LLAMA_STEP", "ENTITY_LLAMA_SWAG", "ENTITY_MAGMA_CUBE_DEATH", "ENTITY_MAGMA_CUBE_DEATH_SMALL", "ENTITY_MAGMA_CUBE_HURT", "ENTITY_MAGMA_CUBE_HURT_SMALL", "ENTITY_MAGMA_CUBE_JUMP", "ENTITY_MAGMA_CUBE_SQUISH", "ENTITY_MAGMA_CUBE_SQUISH_SMALL", "ENTITY_MINECART_INSIDE", "ENTITY_MINECART_INSIDE_UNDERWATER", "ENTITY_MINECART_RIDING", "ENTITY_MOOSHROOM_CONVERT", "ENTITY_MOOSHROOM_EAT", "ENTITY_MOOSHROOM_MILK", "ENTITY_MOOSHROOM_SHEAR", "ENTITY_MOOSHROOM_SUSPICIOUS_MILK", "ENTITY_MULE_AMBIENT", "ENTITY_MULE_ANGRY", "ENTITY_MULE_CHEST", "ENTITY_MULE_DEATH", "ENTITY_MULE_EAT", "ENTITY_MULE_HURT", "ENTITY_MULE_JUMP", "ENTITY_NAUTILUS_AMBIENT", "ENTITY_NAUTILUS_AMBIENT_LAND", "ENTITY_NAUTILUS_DASH", "ENTITY_NAUTILUS_DASH_LAND", "ENTITY_NAUTILUS_DASH_READY", "ENTITY_NAUTILUS_DASH_READY_LAND", "ENTITY_NAUTILUS_DEATH", "ENTITY_NAUTILUS_DEATH_LAND", "ENTITY_NAUTILUS_EAT", "ENTITY_NAUTILUS_HURT", "ENTITY_NAUTILUS_HURT_LAND", "ENTITY_NAUTILUS_RIDING", "ENTITY_NAUTILUS_SWIM", "ENTITY_OCELOT_AMBIENT", "ENTITY_OCELOT_DEATH", "ENTITY_OCELOT_HURT", "ENTITY_PAINTING_BREAK", "ENTITY_PAINTING_PLACE", "ENTITY_PANDA_AGGRESSIVE_AMBIENT", "ENTITY_PANDA_AMBIENT", "ENTITY_PANDA_BITE", "ENTITY_PANDA_CANT_BREED", "ENTITY_PANDA_DEATH", "ENTITY_PANDA_EAT", "ENTITY_PANDA_HURT", "ENTITY_PANDA_PRE_SNEEZE", "ENTITY_PANDA_SNEEZE", "ENTITY_PANDA_STEP", "ENTITY_PANDA_WORRIED_AMBIENT", "ENTITY_PARCHED_AMBIENT", "ENTITY_PARCHED_DEATH", "ENTITY_PARCHED_HURT", "ENTITY_PARCHED_STEP", "ENTITY_PARROT_AMBIENT", "ENTITY_PARROT_DEATH", "ENTITY_PARROT_EAT", "ENTITY_PARROT_FLY", "ENTITY_PARROT_HURT", "ENTITY_PARROT_IMITATE_BLAZE", "ENTITY_PARROT_IMITATE_BOGGED", "ENTITY_PARROT_IMITATE_BREEZE", "ENTITY_PARROT_IMITATE_CAMEL_HUSK", "ENTITY_PARROT_IMITATE_CREAKING", "ENTITY_PARROT_IMITATE_CREEPER", "ENTITY_PARROT_IMITATE_DROWNED", "ENTITY_PARROT_IMITATE_ELDER_GUARDIAN", "ENTITY_PARROT_IMITATE_ENDER_DRAGON", "ENTITY_PARROT_IMITATE_ENDERMITE", "ENTITY_PARROT_IMITATE_EVOKER", "ENTITY_PARROT_IMITATE_GHAST", "ENTITY_PARROT_IMITATE_GUARDIAN", "ENTITY_PARROT_IMITATE_HOGLIN", "ENTITY_PARROT_IMITATE_HUSK", "ENTITY_PARROT_IMITATE_ILLUSIONER", "ENTITY_PARROT_IMITATE_MAGMA_CUBE", "ENTITY_PARROT_IMITATE_PARCHED", "ENTITY_PARROT_IMITATE_PHANTOM", "ENTITY_PARROT_IMITATE_PIGLIN", "ENTITY_PARROT_IMITATE_PIGLIN_BRUTE", "ENTITY_PARROT_IMITATE_PILLAGER", "ENTITY_PARROT_IMITATE_RAVAGER", "ENTITY_PARROT_IMITATE_SHULKER", "ENTITY_PARROT_IMITATE_SILVERFISH", "ENTITY_PARROT_IMITATE_SKELETON", "ENTITY_PARROT_IMITATE_SLIME", "ENTITY_PARROT_IMITATE_SPIDER", "ENTITY_PARROT_IMITATE_STRAY", "ENTITY_PARROT_IMITATE_VEX", "ENTITY_PARROT_IMITATE_VINDICATOR", "ENTITY_PARROT_IMITATE_WARDEN", "ENTITY_PARROT_IMITATE_WITCH", "ENTITY_PARROT_IMITATE_WITHER", "ENTITY_PARROT_IMITATE_WITHER_SKELETON", "ENTITY_PARROT_IMITATE_ZOGLIN", "ENTITY_PARROT_IMITATE_ZOMBIE", "ENTITY_PARROT_IMITATE_ZOMBIE_HORSE", "ENTITY_PARROT_IMITATE_ZOMBIE_NAUTILUS", "ENTITY_PARROT_IMITATE_ZOMBIE_VILLAGER", "ENTITY_PARROT_STEP", "ENTITY_PHANTOM_AMBIENT", "ENTITY_PHANTOM_BITE", "ENTITY_PHANTOM_DEATH", "ENTITY_PHANTOM_FLAP", "ENTITY_PHANTOM_HURT", "ENTITY_PHANTOM_SWOOP", "ENTITY_PIG_AMBIENT", "ENTITY_PIG_DEATH", "ENTITY_PIG_EAT", "ENTITY_PIG_HURT", "ENTITY_PIG_SADDLE", "ENTITY_PIG_STEP", "ENTITY_PIG_BIG_AMBIENT", "ENTITY_PIG_BIG_DEATH", "ENTITY_PIG_BIG_EAT", "ENTITY_PIG_BIG_HURT", "ENTITY_PIG_MINI_AMBIENT", "ENTITY_PIG_MINI_DEATH", "ENTITY_PIG_MINI_EAT", "ENTITY_PIG_MINI_HURT", "ENTITY_PIGLIN_ADMIRING_ITEM", "ENTITY_PIGLIN_AMBIENT", "ENTITY_PIGLIN_ANGRY", "ENTITY_PIGLIN_CELEBRATE", "ENTITY_PIGLIN_CONVERTED_TO_ZOMBIFIED", "ENTITY_PIGLIN_DEATH", "ENTITY_PIGLIN_HURT", "ENTITY_PIGLIN_JEALOUS", "ENTITY_PIGLIN_RETREAT", "ENTITY_PIGLIN_STEP", "ENTITY_PIGLIN_BRUTE_AMBIENT", "ENTITY_PIGLIN_BRUTE_ANGRY", "ENTITY_PIGLIN_BRUTE_CONVERTED_TO_ZOMBIFIED", "ENTITY_PIGLIN_BRUTE_DEATH", "ENTITY_PIGLIN_BRUTE_HURT", "ENTITY_PIGLIN_BRUTE_STEP", "ENTITY_PILLAGER_AMBIENT", "ENTITY_PILLAGER_CELEBRATE", "ENTITY_PILLAGER_DEATH", "ENTITY_PILLAGER_HURT", "ENTITY_PLAYER_ATTACK_CRIT", "ENTITY_PLAYER_ATTACK_KNOCKBACK", "ENTITY_PLAYER_ATTACK_NODAMAGE", "ENTITY_PLAYER_ATTACK_STRONG", "ENTITY_PLAYER_ATTACK_SWEEP", "ENTITY_PLAYER_ATTACK_WEAK", "ENTITY_PLAYER_BIG_FALL", "ENTITY_PLAYER_BREATH", "ENTITY_PLAYER_BURP", "ENTITY_PLAYER_DEATH", "ENTITY_PLAYER_HURT", "ENTITY_PLAYER_HURT_DROWN", "ENTITY_PLAYER_HURT_FREEZE", "ENTITY_PLAYER_HURT_ON_FIRE", "ENTITY_PLAYER_HURT_SWEET_BERRY_BUSH", "ENTITY_PLAYER_LEVELUP", "ENTITY_PLAYER_SMALL_FALL", "ENTITY_PLAYER_SPLASH", "ENTITY_PLAYER_SPLASH_HIGH_SPEED", "ENTITY_PLAYER_SWIM", "ENTITY_PLAYER_TELEPORT", "ENTITY_POLAR_BEAR_AMBIENT", "ENTITY_POLAR_BEAR_AMBIENT_BABY", "ENTITY_POLAR_BEAR_DEATH", "ENTITY_POLAR_BEAR_HURT", "ENTITY_POLAR_BEAR_STEP", "ENTITY_POLAR_BEAR_WARNING", "ENTITY_PUFFER_FISH_BLOW_OUT", "ENTITY_PUFFER_FISH_BLOW_UP", "ENTITY_PUFFER_FISH_DEATH", "ENTITY_PUFFER_FISH_FLOP", "ENTITY_PUFFER_FISH_HURT", "ENTITY_PUFFER_FISH_STING", "ENTITY_RABBIT_AMBIENT", "ENTITY_RABBIT_ATTACK", "ENTITY_RABBIT_DEATH", "ENTITY_RABBIT_HURT", "ENTITY_RABBIT_JUMP", "ENTITY_RAVAGER_AMBIENT", "ENTITY_RAVAGER_ATTACK", "ENTITY_RAVAGER_CELEBRATE", "ENTITY_RAVAGER_DEATH", "ENTITY_RAVAGER_HURT", "ENTITY_RAVAGER_ROAR", "ENTITY_RAVAGER_STEP", "ENTITY_RAVAGER_STUNNED", "ENTITY_SALMON_AMBIENT", "ENTITY_SALMON_DEATH", "ENTITY_SALMON_FLOP", "ENTITY_SALMON_HURT", "ENTITY_SHEEP_AMBIENT", "ENTITY_SHEEP_DEATH", "ENTITY_SHEEP_HURT", "ENTITY_SHEEP_SHEAR", "ENTITY_SHEEP_STEP", "ENTITY_SHULKER_AMBIENT", "ENTITY_SHULKER_CLOSE", "ENTITY_SHULKER_DEATH", "ENTITY_SHULKER_HURT", "ENTITY_SHULKER_HURT_CLOSED", "ENTITY_SHULKER_OPEN", "ENTITY_SHULKER_SHOOT", "ENTITY_SHULKER_TELEPORT", "ENTITY_SHULKER_BULLET_HIT", "ENTITY_SHULKER_BULLET_HURT", "ENTITY_SILVERFISH_AMBIENT", "ENTITY_SILVERFISH_DEATH", "ENTITY_SILVERFISH_HURT", "ENTITY_SILVERFISH_STEP", "ENTITY_SKELETON_AMBIENT", "ENTITY_SKELETON_CONVERTED_TO_STRAY", "ENTITY_SKELETON_DEATH", "ENTITY_SKELETON_HURT", "ENTITY_SKELETON_SHOOT", "ENTITY_SKELETON_STEP", "ENTITY_SKELETON_HORSE_AMBIENT", "ENTITY_SKELETON_HORSE_AMBIENT_WATER", "ENTITY_SKELETON_HORSE_DEATH", "ENTITY_SKELETON_HORSE_GALLOP_WATER", "ENTITY_SKELETON_HORSE_HURT", "ENTITY_SKELETON_HORSE_JUMP_WATER", "ENTITY_SKELETON_HORSE_STEP_WATER", "ENTITY_SKELETON_HORSE_SWIM", "ENTITY_SLIME_ATTACK", "ENTITY_SLIME_DEATH", "ENTITY_SLIME_DEATH_SMALL", "ENTITY_SLIME_HURT", "ENTITY_SLIME_HURT_SMALL", "ENTITY_SLIME_JUMP", "ENTITY_SLIME_JUMP_SMALL", "ENTITY_SLIME_SQUISH", "ENTITY_SLIME_SQUISH_SMALL", "ENTITY_SMALL_SULFUR_CUBE_DEATH", "ENTITY_SMALL_SULFUR_CUBE_EAT", "ENTITY_SMALL_SULFUR_CUBE_HURT", "ENTITY_SMALL_SULFUR_CUBE_JUMP", "ENTITY_SMALL_SULFUR_CUBE_SQUISH", "ENTITY_SNIFFER_DEATH", "ENTITY_SNIFFER_DIGGING", "ENTITY_SNIFFER_DIGGING_STOP", "ENTITY_SNIFFER_DROP_SEED", "ENTITY_SNIFFER_EAT", "ENTITY_SNIFFER_HAPPY", "ENTITY_SNIFFER_HURT", "ENTITY_SNIFFER_IDLE", "ENTITY_SNIFFER_SCENTING", "ENTITY_SNIFFER_SEARCHING", "ENTITY_SNIFFER_SNIFFING", "ENTITY_SNIFFER_STEP", "ENTITY_SNOW_GOLEM_AMBIENT", "ENTITY_SNOW_GOLEM_DEATH", "ENTITY_SNOW_GOLEM_HURT", "ENTITY_SNOW_GOLEM_SHEAR", "ENTITY_SNOW_GOLEM_SHOOT", "ENTITY_SNOWBALL_THROW", "ENTITY_SPIDER_AMBIENT", "ENTITY_SPIDER_DEATH", "ENTITY_SPIDER_HURT", "ENTITY_SPIDER_STEP", "ENTITY_SPLASH_POTION_BREAK", "ENTITY_SPLASH_POTION_THROW", "ENTITY_SQUID_AMBIENT", "ENTITY_SQUID_DEATH", "ENTITY_SQUID_HURT", "ENTITY_SQUID_SQUIRT", "ENTITY_STRAY_AMBIENT", "ENTITY_STRAY_DEATH", "ENTITY_STRAY_HURT", "ENTITY_STRAY_STEP", "ENTITY_STRIDER_AMBIENT", "ENTITY_STRIDER_DEATH", "ENTITY_STRIDER_EAT", "ENTITY_STRIDER_HAPPY", "ENTITY_STRIDER_HURT", "ENTITY_STRIDER_RETREAT", "ENTITY_STRIDER_SADDLE", "ENTITY_STRIDER_STEP", "ENTITY_STRIDER_STEP_LAVA", "ENTITY_SULFUR_CUBE_ABSORB", "ENTITY_SULFUR_CUBE_BOUNCE", "ENTITY_SULFUR_CUBE_BOUNCY_HIT", "ENTITY_SULFUR_CUBE_BOUNCY_PUSH", "ENTITY_SULFUR_CUBE_DEATH", "ENTITY_SULFUR_CUBE_EJECT", "ENTITY_SULFUR_CUBE_EXPLOSIVE_HIT", "ENTITY_SULFUR_CUBE_EXPLOSIVE_PUSH", "ENTITY_SULFUR_CUBE_FAST_FLAT_HIT", "ENTITY_SULFUR_CUBE_FAST_FLAT_PUSH", "ENTITY_SULFUR_CUBE_FAST_SLIDING_HIT", "ENTITY_SULFUR_CUBE_FAST_SLIDING_PUSH", "ENTITY_SULFUR_CUBE_HIGH_RESISTANCE_HIT", "ENTITY_SULFUR_CUBE_HIGH_RESISTANCE_PUSH", "ENTITY_SULFUR_CUBE_HOT_HIT", "ENTITY_SULFUR_CUBE_HOT_PUSH", "ENTITY_SULFUR_CUBE_HURT", "ENTITY_SULFUR_CUBE_JUMP", "ENTITY_SULFUR_CUBE_LIGHT_HIT", "ENTITY_SULFUR_CUBE_LIGHT_PUSH", "ENTITY_SULFUR_CUBE_REGULAR_HIT", "ENTITY_SULFUR_CUBE_REGULAR_PUSH", "ENTITY_SULFUR_CUBE_SLOW_BOUNCY_HIT", "ENTITY_SULFUR_CUBE_SLOW_BOUNCY_PUSH", "ENTITY_SULFUR_CUBE_SLOW_FLAT_HIT", "ENTITY_SULFUR_CUBE_SLOW_FLAT_PUSH", "ENTITY_SULFUR_CUBE_SLOW_SLIDING_HIT", "ENTITY_SULFUR_CUBE_SLOW_SLIDING_PUSH", "ENTITY_SULFUR_CUBE_SQUISH", "ENTITY_SULFUR_CUBE_STICKY_HIT", "ENTITY_SULFUR_CUBE_STICKY_PUSH", "ENTITY_TADPOLE_DEATH", "ENTITY_TADPOLE_FLOP", "ENTITY_TADPOLE_GROW_UP", "ENTITY_TADPOLE_HURT", "ENTITY_TNT_PRIMED", "ENTITY_TROPICAL_FISH_AMBIENT", "ENTITY_TROPICAL_FISH_DEATH", "ENTITY_TROPICAL_FISH_FLOP", "ENTITY_TROPICAL_FISH_HURT", "ENTITY_TURTLE_AMBIENT_LAND", "ENTITY_TURTLE_DEATH", "ENTITY_TURTLE_DEATH_BABY", "ENTITY_TURTLE_EGG_BREAK", "ENTITY_TURTLE_EGG_CRACK", "ENTITY_TURTLE_EGG_HATCH", "ENTITY_TURTLE_HURT", "ENTITY_TURTLE_HURT_BABY", "ENTITY_TURTLE_LAY_EGG", "ENTITY_TURTLE_SHAMBLE", "ENTITY_TURTLE_SHAMBLE_BABY", "ENTITY_TURTLE_SWIM", "ENTITY_VEX_AMBIENT", "ENTITY_VEX_CHARGE", "ENTITY_VEX_DEATH", "ENTITY_VEX_HURT", "ENTITY_VILLAGER_AMBIENT", "ENTITY_VILLAGER_CELEBRATE", "ENTITY_VILLAGER_DEATH", "ENTITY_VILLAGER_HURT", "ENTITY_VILLAGER_NO", "ENTITY_VILLAGER_TRADE", "ENTITY_VILLAGER_WORK_ARMORER", "ENTITY_VILLAGER_WORK_BUTCHER", "ENTITY_VILLAGER_WORK_CARTOGRAPHER", "ENTITY_VILLAGER_WORK_CLERIC", "ENTITY_VILLAGER_WORK_FARMER", "ENTITY_VILLAGER_WORK_FISHERMAN", "ENTITY_VILLAGER_WORK_FLETCHER", "ENTITY_VILLAGER_WORK_LEATHERWORKER", "ENTITY_VILLAGER_WORK_LIBRARIAN", "ENTITY_VILLAGER_WORK_MASON", "ENTITY_VILLAGER_WORK_SHEPHERD", "ENTITY_VILLAGER_WORK_TOOLSMITH", "ENTITY_VILLAGER_WORK_WEAPONSMITH", "ENTITY_VILLAGER_YES", "ENTITY_VINDICATOR_AMBIENT", "ENTITY_VINDICATOR_CELEBRATE", "ENTITY_VINDICATOR_DEATH", "ENTITY_VINDICATOR_HURT", "ENTITY_WANDERING_TRADER_AMBIENT", "ENTITY_WANDERING_TRADER_DEATH", "ENTITY_WANDERING_TRADER_DISAPPEARED", "ENTITY_WANDERING_TRADER_DRINK_MILK", "ENTITY_WANDERING_TRADER_DRINK_POTION", "ENTITY_WANDERING_TRADER_HURT", "ENTITY_WANDERING_TRADER_NO", "ENTITY_WANDERING_TRADER_REAPPEARED", "ENTITY_WANDERING_TRADER_TRADE", "ENTITY_WANDERING_TRADER_YES", "ENTITY_WARDEN_AGITATED", "ENTITY_WARDEN_AMBIENT", "ENTITY_WARDEN_ANGRY", "ENTITY_WARDEN_ATTACK_IMPACT", "ENTITY_WARDEN_DEATH", "ENTITY_WARDEN_DIG", "ENTITY_WARDEN_EMERGE", "ENTITY_WARDEN_HEARTBEAT", "ENTITY_WARDEN_HURT", "ENTITY_WARDEN_LISTENING", "ENTITY_WARDEN_LISTENING_ANGRY", "ENTITY_WARDEN_NEARBY_CLOSE", "ENTITY_WARDEN_NEARBY_CLOSER", "ENTITY_WARDEN_NEARBY_CLOSEST", "ENTITY_WARDEN_ROAR", "ENTITY_WARDEN_SNIFF", "ENTITY_WARDEN_SONIC_BOOM", "ENTITY_WARDEN_SONIC_CHARGE", "ENTITY_WARDEN_STEP", "ENTITY_WARDEN_TENDRIL_CLICKS", "ENTITY_WIND_CHARGE_THROW", "ENTITY_WIND_CHARGE_WIND_BURST", "ENTITY_WITCH_AMBIENT", "ENTITY_WITCH_CELEBRATE", "ENTITY_WITCH_DEATH", "ENTITY_WITCH_DRINK", "ENTITY_WITCH_HURT", "ENTITY_WITCH_THROW", "ENTITY_WITHER_AMBIENT", "ENTITY_WITHER_BREAK_BLOCK", "ENTITY_WITHER_DEATH", "ENTITY_WITHER_HURT", "ENTITY_WITHER_SHOOT", "ENTITY_WITHER_SPAWN", "ENTITY_WITHER_SKELETON_AMBIENT", "ENTITY_WITHER_SKELETON_DEATH", "ENTITY_WITHER_SKELETON_HURT", "ENTITY_WITHER_SKELETON_STEP", "ENTITY_WOLF_AMBIENT", "ENTITY_WOLF_DEATH", "ENTITY_WOLF_GROWL", "ENTITY_WOLF_HURT", "ENTITY_WOLF_PANT", "ENTITY_WOLF_SHAKE", "ENTITY_WOLF_STEP", "ENTITY_WOLF_WHINE", "ENTITY_WOLF_ANGRY_AMBIENT", "ENTITY_WOLF_ANGRY_DEATH", "ENTITY_WOLF_ANGRY_GROWL", "ENTITY_WOLF_ANGRY_HURT", "ENTITY_WOLF_ANGRY_PANT", "ENTITY_WOLF_ANGRY_WHINE", "ENTITY_WOLF_BIG_AMBIENT", "ENTITY_WOLF_BIG_DEATH", "ENTITY_WOLF_BIG_GROWL", "ENTITY_WOLF_BIG_HURT", "ENTITY_WOLF_BIG_PANT", "ENTITY_WOLF_BIG_WHINE", "ENTITY_WOLF_CUTE_AMBIENT", "ENTITY_WOLF_CUTE_DEATH", "ENTITY_WOLF_CUTE_GROWL", "ENTITY_WOLF_CUTE_HURT", "ENTITY_WOLF_CUTE_PANT", "ENTITY_WOLF_CUTE_WHINE", "ENTITY_WOLF_GRUMPY_AMBIENT", "ENTITY_WOLF_GRUMPY_DEATH", "ENTITY_WOLF_GRUMPY_GROWL", "ENTITY_WOLF_GRUMPY_HURT", "ENTITY_WOLF_GRUMPY_PANT", "ENTITY_WOLF_GRUMPY_WHINE", "ENTITY_WOLF_PUGLIN_AMBIENT", "ENTITY_WOLF_PUGLIN_DEATH", "ENTITY_WOLF_PUGLIN_GROWL", "ENTITY_WOLF_PUGLIN_HURT", "ENTITY_WOLF_PUGLIN_PANT", "ENTITY_WOLF_PUGLIN_WHINE", "ENTITY_WOLF_SAD_AMBIENT", "ENTITY_WOLF_SAD_DEATH", "ENTITY_WOLF_SAD_GROWL", "ENTITY_WOLF_SAD_HURT", "ENTITY_WOLF_SAD_PANT", "ENTITY_WOLF_SAD_WHINE", "ENTITY_ZOGLIN_AMBIENT", "ENTITY_ZOGLIN_ANGRY", "ENTITY_ZOGLIN_ATTACK", "ENTITY_ZOGLIN_DEATH", "ENTITY_ZOGLIN_HURT", "ENTITY_ZOGLIN_STEP", "ENTITY_ZOMBIE_AMBIENT", "ENTITY_ZOMBIE_ATTACK_IRON_DOOR", "ENTITY_ZOMBIE_ATTACK_WOODEN_DOOR", "ENTITY_ZOMBIE_BREAK_WOODEN_DOOR", "ENTITY_ZOMBIE_CONVERTED_TO_DROWNED", "ENTITY_ZOMBIE_DEATH", "ENTITY_ZOMBIE_DESTROY_EGG", "ENTITY_ZOMBIE_HURT", "ENTITY_ZOMBIE_INFECT", "ENTITY_ZOMBIE_STEP", "ENTITY_ZOMBIE_HORSE_AMBIENT", "ENTITY_ZOMBIE_HORSE_ANGRY", "ENTITY_ZOMBIE_HORSE_DEATH", "ENTITY_ZOMBIE_HORSE_EAT", "ENTITY_ZOMBIE_HORSE_HURT", "ENTITY_ZOMBIE_NAUTILUS_AMBIENT", "ENTITY_ZOMBIE_NAUTILUS_AMBIENT_LAND", "ENTITY_ZOMBIE_NAUTILUS_DASH", "ENTITY_ZOMBIE_NAUTILUS_DASH_LAND", "ENTITY_ZOMBIE_NAUTILUS_DASH_READY", "ENTITY_ZOMBIE_NAUTILUS_DASH_READY_LAND", "ENTITY_ZOMBIE_NAUTILUS_DEATH", "ENTITY_ZOMBIE_NAUTILUS_DEATH_LAND", "ENTITY_ZOMBIE_NAUTILUS_EAT", "ENTITY_ZOMBIE_NAUTILUS_HURT", "ENTITY_ZOMBIE_NAUTILUS_HURT_LAND", "ENTITY_ZOMBIE_NAUTILUS_SWIM", "ENTITY_ZOMBIE_VILLAGER_AMBIENT", "ENTITY_ZOMBIE_VILLAGER_CONVERTED", "ENTITY_ZOMBIE_VILLAGER_CURE", "ENTITY_ZOMBIE_VILLAGER_DEATH", "ENTITY_ZOMBIE_VILLAGER_HURT", "ENTITY_ZOMBIE_VILLAGER_STEP", "ENTITY_ZOMBIFIED_PIGLIN_AMBIENT", "ENTITY_ZOMBIFIED_PIGLIN_ANGRY", "ENTITY_ZOMBIFIED_PIGLIN_DEATH", "ENTITY_ZOMBIFIED_PIGLIN_HURT", "EVENT_MOB_EFFECT_BAD_OMEN", "EVENT_MOB_EFFECT_RAID_OMEN", "EVENT_MOB_EFFECT_TRIAL_OMEN", "EVENT_RAID_HORN", "INTENTIONALLY_EMPTY", "ITEM_ARMOR_EQUIP_CHAIN", "ITEM_ARMOR_EQUIP_COPPER", "ITEM_ARMOR_EQUIP_DIAMOND", "ITEM_ARMOR_EQUIP_ELYTRA", "ITEM_ARMOR_EQUIP_GENERIC", "ITEM_ARMOR_EQUIP_GOLD", "ITEM_ARMOR_EQUIP_IRON", "ITEM_ARMOR_EQUIP_LEATHER", "ITEM_ARMOR_EQUIP_NAUTILUS", "ITEM_ARMOR_EQUIP_NETHERITE", "ITEM_ARMOR_EQUIP_TURTLE", "ITEM_ARMOR_EQUIP_WOLF", "ITEM_ARMOR_UNEQUIP_NAUTILUS", "ITEM_ARMOR_UNEQUIP_WOLF", "ITEM_AXE_SCRAPE", "ITEM_AXE_STRIP", "ITEM_AXE_WAX_OFF", "ITEM_BONE_MEAL_USE", "ITEM_BOOK_PAGE_TURN", "ITEM_BOOK_PUT", "ITEM_BOTTLE_EMPTY", "ITEM_BOTTLE_FILL", "ITEM_BOTTLE_FILL_DRAGONBREATH", "ITEM_BRUSH_BRUSHING_GENERIC", "ITEM_BRUSH_BRUSHING_GRAVEL", "ITEM_BRUSH_BRUSHING_GRAVEL_COMPLETE", "ITEM_BRUSH_BRUSHING_SAND", "ITEM_BRUSH_BRUSHING_SAND_COMPLETE", "ITEM_BUCKET_EMPTY", "ITEM_BUCKET_EMPTY_AXOLOTL", "ITEM_BUCKET_EMPTY_FISH", "ITEM_BUCKET_EMPTY_LAVA", "ITEM_BUCKET_EMPTY_POWDER_SNOW", "ITEM_BUCKET_EMPTY_SULFUR_CUBE", "ITEM_BUCKET_EMPTY_TADPOLE", "ITEM_BUCKET_FILL", "ITEM_BUCKET_FILL_AXOLOTL", "ITEM_BUCKET_FILL_FISH", "ITEM_BUCKET_FILL_LAVA", "ITEM_BUCKET_FILL_POWDER_SNOW", "ITEM_BUCKET_FILL_SULFUR_CUBE", "ITEM_BUCKET_FILL_TADPOLE", "ITEM_BUNDLE_DROP_CONTENTS", "ITEM_BUNDLE_INSERT", "ITEM_BUNDLE_INSERT_FAIL", "ITEM_BUNDLE_REMOVE_ONE", "ITEM_CHORUS_FRUIT_TELEPORT", "ITEM_CROP_PLANT", "ITEM_CROSSBOW_HIT", "ITEM_CROSSBOW_LOADING_END", "ITEM_CROSSBOW_LOADING_MIDDLE", "ITEM_CROSSBOW_LOADING_START", "ITEM_CROSSBOW_QUICK_CHARGE_1", "ITEM_CROSSBOW_QUICK_CHARGE_2", "ITEM_CROSSBOW_QUICK_CHARGE_3", "ITEM_CROSSBOW_SHOOT", "ITEM_DYE_USE", "ITEM_ELYTRA_FLYING", "ITEM_FIRECHARGE_USE", "ITEM_FLINTANDSTEEL_USE", "ITEM_GLOW_INK_SAC_USE", "ITEM_GOAT_HORN_SOUND_0", "ITEM_GOAT_HORN_SOUND_1", "ITEM_GOAT_HORN_SOUND_2", "ITEM_GOAT_HORN_SOUND_3", "ITEM_GOAT_HORN_SOUND_4", "ITEM_GOAT_HORN_SOUND_5", "ITEM_GOAT_HORN_SOUND_6", "ITEM_GOAT_HORN_SOUND_7", "ITEM_GOLDEN_DANDELION_UNUSE", "ITEM_GOLDEN_DANDELION_USE", "ITEM_HOE_TILL", "ITEM_HONEY_BOTTLE_DRINK", "ITEM_HONEYCOMB_WAX_ON", "ITEM_HORSE_ARMOR_UNEQUIP", "ITEM_INK_SAC_USE", "ITEM_LEAD_BREAK", "ITEM_LEAD_TIED", "ITEM_LEAD_UNTIED", "ITEM_LLAMA_CARPET_UNEQUIP", "ITEM_LODESTONE_COMPASS_LOCK", "ITEM_MACE_SMASH_AIR", "ITEM_MACE_SMASH_GROUND", "ITEM_MACE_SMASH_GROUND_HEAVY", "ITEM_NAUTILUS_SADDLE_EQUIP", "ITEM_NAUTILUS_SADDLE_UNDERWATER_EQUIP", "ITEM_NETHER_WART_PLANT", "ITEM_OMINOUS_BOTTLE_DISPOSE", "ITEM_SADDLE_UNEQUIP", "ITEM_SHEARS_SNIP", "ITEM_SHIELD_BLOCK", "ITEM_SHIELD_BREAK", "ITEM_SHOVEL_FLATTEN", "ITEM_SPEAR_ATTACK", "ITEM_SPEAR_HIT", "ITEM_SPEAR_LUNGE_1", "ITEM_SPEAR_LUNGE_2", "ITEM_SPEAR_LUNGE_3", "ITEM_SPEAR_USE", "ITEM_SPEAR_WOOD_ATTACK", "ITEM_SPEAR_WOOD_HIT", "ITEM_SPEAR_WOOD_USE", "ITEM_SPYGLASS_STOP_USING", "ITEM_SPYGLASS_USE", "ITEM_TOTEM_USE", "ITEM_TRIDENT_HIT", "ITEM_TRIDENT_HIT_GROUND", "ITEM_TRIDENT_RETURN", "ITEM_TRIDENT_RIPTIDE_1", "ITEM_TRIDENT_RIPTIDE_2", "ITEM_TRIDENT_RIPTIDE_3", "ITEM_TRIDENT_THROW", "ITEM_TRIDENT_THUNDER", "ITEM_WOLF_ARMOR_BREAK", "ITEM_WOLF_ARMOR_CRACK", "ITEM_WOLF_ARMOR_DAMAGE", "ITEM_WOLF_ARMOR_REPAIR", "MUSIC_CREATIVE", "MUSIC_CREDITS", "MUSIC_DRAGON", "MUSIC_END", "MUSIC_GAME", "MUSIC_MENU", "MUSIC_NETHER_BASALT_DELTAS", "MUSIC_NETHER_CRIMSON_FOREST", "MUSIC_NETHER_NETHER_WASTES", "MUSIC_NETHER_SOUL_SAND_VALLEY", "MUSIC_NETHER_WARPED_FOREST", "MUSIC_OVERWORLD_BADLANDS", "MUSIC_OVERWORLD_BAMBOO_JUNGLE", "MUSIC_OVERWORLD_CHERRY_GROVE", "MUSIC_OVERWORLD_DEEP_DARK", "MUSIC_OVERWORLD_DESERT", "MUSIC_OVERWORLD_DRIPSTONE_CAVES", "MUSIC_OVERWORLD_FLOWER_FOREST", "MUSIC_OVERWORLD_FOREST", "MUSIC_OVERWORLD_FROZEN_PEAKS", "MUSIC_OVERWORLD_GROVE", "MUSIC_OVERWORLD_JAGGED_PEAKS", "MUSIC_OVERWORLD_JUNGLE", "MUSIC_OVERWORLD_LUSH_CAVES", "MUSIC_OVERWORLD_MEADOW", "MUSIC_OVERWORLD_OLD_GROWTH_TAIGA", "MUSIC_OVERWORLD_SNOWY_SLOPES", "MUSIC_OVERWORLD_SPARSE_JUNGLE", "MUSIC_OVERWORLD_STONY_PEAKS", "MUSIC_OVERWORLD_SULFUR_CAVES", "MUSIC_OVERWORLD_SWAMP", "MUSIC_UNDER_WATER", "MUSIC_DISC_11", "MUSIC_DISC_13", "MUSIC_DISC_5", "MUSIC_DISC_BLOCKS", "MUSIC_DISC_BOUNCE", "MUSIC_DISC_CAT", "MUSIC_DISC_CHIRP", "MUSIC_DISC_CREATOR", "MUSIC_DISC_CREATOR_MUSIC_BOX", "MUSIC_DISC_FAR", "MUSIC_DISC_LAVA_CHICKEN", "MUSIC_DISC_MALL", "MUSIC_DISC_MELLOHI", "MUSIC_DISC_OTHERSIDE", "MUSIC_DISC_PIGSTEP", "MUSIC_DISC_PRECIPICE", "MUSIC_DISC_RELIC", "MUSIC_DISC_STAL", "MUSIC_DISC_STRAD", "MUSIC_DISC_TEARS", "MUSIC_DISC_WAIT", "MUSIC_DISC_WARD", "PARTICLE_SOUL_ESCAPE", "UI_BUTTON_CLICK", "UI_CARTOGRAPHY_TABLE_TAKE_RESULT", "UI_HUD_BUBBLE_POP", "UI_LOOM_SELECT_PATTERN", "UI_LOOM_TAKE_RESULT", "UI_STONECUTTER_SELECT_RECIPE", "UI_STONECUTTER_TAKE_RESULT", "UI_TOAST_CHALLENGE_COMPLETE", "UI_TOAST_IN", "UI_TOAST_OUT", "WEATHER_END_FLASH", "WEATHER_RAIN", "WEATHER_RAIN_ABOVE"]
    }
    /// bukkit: org/bukkit/Sound.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const AMBIENT_BASALT_DELTAS_ADDITIONS: &str = "minecraft:ambient_basalt_deltas_additions";
    pub const AMBIENT_BASALT_DELTAS_LOOP: &str = "minecraft:ambient_basalt_deltas_loop";
    pub const AMBIENT_BASALT_DELTAS_MOOD: &str = "minecraft:ambient_basalt_deltas_mood";
    pub const AMBIENT_CAVE: &str = "minecraft:ambient_cave";
    pub const AMBIENT_CRIMSON_FOREST_ADDITIONS: &str = "minecraft:ambient_crimson_forest_additions";
    pub const AMBIENT_CRIMSON_FOREST_LOOP: &str = "minecraft:ambient_crimson_forest_loop";
    pub const AMBIENT_CRIMSON_FOREST_MOOD: &str = "minecraft:ambient_crimson_forest_mood";
    pub const AMBIENT_NETHER_WASTES_ADDITIONS: &str = "minecraft:ambient_nether_wastes_additions";
    pub const AMBIENT_NETHER_WASTES_LOOP: &str = "minecraft:ambient_nether_wastes_loop";
    pub const AMBIENT_NETHER_WASTES_MOOD: &str = "minecraft:ambient_nether_wastes_mood";
    pub const AMBIENT_SOUL_SAND_VALLEY_ADDITIONS: &str = "minecraft:ambient_soul_sand_valley_additions";
    pub const AMBIENT_SOUL_SAND_VALLEY_LOOP: &str = "minecraft:ambient_soul_sand_valley_loop";
    pub const AMBIENT_SOUL_SAND_VALLEY_MOOD: &str = "minecraft:ambient_soul_sand_valley_mood";
    pub const AMBIENT_UNDERWATER_ENTER: &str = "minecraft:ambient_underwater_enter";
    pub const AMBIENT_UNDERWATER_EXIT: &str = "minecraft:ambient_underwater_exit";
    pub const AMBIENT_UNDERWATER_LOOP: &str = "minecraft:ambient_underwater_loop";
    pub const AMBIENT_UNDERWATER_LOOP_ADDITIONS: &str = "minecraft:ambient_underwater_loop_additions";
    pub const AMBIENT_UNDERWATER_LOOP_ADDITIONS_RARE: &str = "minecraft:ambient_underwater_loop_additions_rare";
    pub const AMBIENT_UNDERWATER_LOOP_ADDITIONS_ULTRA_RARE: &str = "minecraft:ambient_underwater_loop_additions_ultra_rare";
    pub const AMBIENT_WARPED_FOREST_ADDITIONS: &str = "minecraft:ambient_warped_forest_additions";
    pub const AMBIENT_WARPED_FOREST_LOOP: &str = "minecraft:ambient_warped_forest_loop";
    pub const AMBIENT_WARPED_FOREST_MOOD: &str = "minecraft:ambient_warped_forest_mood";
    pub const BLOCK_AMETHYST_BLOCK_BREAK: &str = "minecraft:block_amethyst_block_break";
    pub const BLOCK_AMETHYST_BLOCK_CHIME: &str = "minecraft:block_amethyst_block_chime";
    pub const BLOCK_AMETHYST_BLOCK_FALL: &str = "minecraft:block_amethyst_block_fall";
    pub const BLOCK_AMETHYST_BLOCK_HIT: &str = "minecraft:block_amethyst_block_hit";
    pub const BLOCK_AMETHYST_BLOCK_PLACE: &str = "minecraft:block_amethyst_block_place";
    pub const BLOCK_AMETHYST_BLOCK_RESONATE: &str = "minecraft:block_amethyst_block_resonate";
    pub const BLOCK_AMETHYST_BLOCK_STEP: &str = "minecraft:block_amethyst_block_step";
    pub const BLOCK_AMETHYST_CLUSTER_BREAK: &str = "minecraft:block_amethyst_cluster_break";
    pub const BLOCK_AMETHYST_CLUSTER_FALL: &str = "minecraft:block_amethyst_cluster_fall";
    pub const BLOCK_AMETHYST_CLUSTER_HIT: &str = "minecraft:block_amethyst_cluster_hit";
    pub const BLOCK_AMETHYST_CLUSTER_PLACE: &str = "minecraft:block_amethyst_cluster_place";
    pub const BLOCK_AMETHYST_CLUSTER_STEP: &str = "minecraft:block_amethyst_cluster_step";
    pub const BLOCK_ANCIENT_DEBRIS_BREAK: &str = "minecraft:block_ancient_debris_break";
    pub const BLOCK_ANCIENT_DEBRIS_FALL: &str = "minecraft:block_ancient_debris_fall";
    pub const BLOCK_ANCIENT_DEBRIS_HIT: &str = "minecraft:block_ancient_debris_hit";
    pub const BLOCK_ANCIENT_DEBRIS_PLACE: &str = "minecraft:block_ancient_debris_place";
    pub const BLOCK_ANCIENT_DEBRIS_STEP: &str = "minecraft:block_ancient_debris_step";
    pub const BLOCK_ANVIL_BREAK: &str = "minecraft:block_anvil_break";
    pub const BLOCK_ANVIL_DESTROY: &str = "minecraft:block_anvil_destroy";
    pub const BLOCK_ANVIL_FALL: &str = "minecraft:block_anvil_fall";
    pub const BLOCK_ANVIL_HIT: &str = "minecraft:block_anvil_hit";
    pub const BLOCK_ANVIL_LAND: &str = "minecraft:block_anvil_land";
    pub const BLOCK_ANVIL_PLACE: &str = "minecraft:block_anvil_place";
    pub const BLOCK_ANVIL_STEP: &str = "minecraft:block_anvil_step";
    pub const BLOCK_ANVIL_USE: &str = "minecraft:block_anvil_use";
    pub const BLOCK_AZALEA_BREAK: &str = "minecraft:block_azalea_break";
    pub const BLOCK_AZALEA_FALL: &str = "minecraft:block_azalea_fall";
    pub const BLOCK_AZALEA_HIT: &str = "minecraft:block_azalea_hit";
    pub const BLOCK_AZALEA_PLACE: &str = "minecraft:block_azalea_place";
    pub const BLOCK_AZALEA_STEP: &str = "minecraft:block_azalea_step";
    pub const BLOCK_AZALEA_LEAVES_BREAK: &str = "minecraft:block_azalea_leaves_break";
    pub const BLOCK_AZALEA_LEAVES_FALL: &str = "minecraft:block_azalea_leaves_fall";
    pub const BLOCK_AZALEA_LEAVES_HIT: &str = "minecraft:block_azalea_leaves_hit";
    pub const BLOCK_AZALEA_LEAVES_PLACE: &str = "minecraft:block_azalea_leaves_place";
    pub const BLOCK_AZALEA_LEAVES_STEP: &str = "minecraft:block_azalea_leaves_step";
    pub const BLOCK_BAMBOO_BREAK: &str = "minecraft:block_bamboo_break";
    pub const BLOCK_BAMBOO_FALL: &str = "minecraft:block_bamboo_fall";
    pub const BLOCK_BAMBOO_HIT: &str = "minecraft:block_bamboo_hit";
    pub const BLOCK_BAMBOO_PLACE: &str = "minecraft:block_bamboo_place";
    pub const BLOCK_BAMBOO_STEP: &str = "minecraft:block_bamboo_step";
    pub const BLOCK_BAMBOO_SAPLING_BREAK: &str = "minecraft:block_bamboo_sapling_break";
    pub const BLOCK_BAMBOO_SAPLING_HIT: &str = "minecraft:block_bamboo_sapling_hit";
    pub const BLOCK_BAMBOO_SAPLING_PLACE: &str = "minecraft:block_bamboo_sapling_place";
    pub const BLOCK_BAMBOO_WOOD_BREAK: &str = "minecraft:block_bamboo_wood_break";
    pub const BLOCK_BAMBOO_WOOD_FALL: &str = "minecraft:block_bamboo_wood_fall";
    pub const BLOCK_BAMBOO_WOOD_HIT: &str = "minecraft:block_bamboo_wood_hit";
    pub const BLOCK_BAMBOO_WOOD_PLACE: &str = "minecraft:block_bamboo_wood_place";
    pub const BLOCK_BAMBOO_WOOD_STEP: &str = "minecraft:block_bamboo_wood_step";
    pub const BLOCK_BAMBOO_WOOD_BUTTON_CLICK_OFF: &str = "minecraft:block_bamboo_wood_button_click_off";
    pub const BLOCK_BAMBOO_WOOD_BUTTON_CLICK_ON: &str = "minecraft:block_bamboo_wood_button_click_on";
    pub const BLOCK_BAMBOO_WOOD_DOOR_CLOSE: &str = "minecraft:block_bamboo_wood_door_close";
    pub const BLOCK_BAMBOO_WOOD_DOOR_OPEN: &str = "minecraft:block_bamboo_wood_door_open";
    pub const BLOCK_BAMBOO_WOOD_FENCE_GATE_CLOSE: &str = "minecraft:block_bamboo_wood_fence_gate_close";
    pub const BLOCK_BAMBOO_WOOD_FENCE_GATE_OPEN: &str = "minecraft:block_bamboo_wood_fence_gate_open";
    pub const BLOCK_BAMBOO_WOOD_HANGING_SIGN_BREAK: &str = "minecraft:block_bamboo_wood_hanging_sign_break";
    pub const BLOCK_BAMBOO_WOOD_HANGING_SIGN_FALL: &str = "minecraft:block_bamboo_wood_hanging_sign_fall";
    pub const BLOCK_BAMBOO_WOOD_HANGING_SIGN_HIT: &str = "minecraft:block_bamboo_wood_hanging_sign_hit";
    pub const BLOCK_BAMBOO_WOOD_HANGING_SIGN_PLACE: &str = "minecraft:block_bamboo_wood_hanging_sign_place";
    pub const BLOCK_BAMBOO_WOOD_HANGING_SIGN_STEP: &str = "minecraft:block_bamboo_wood_hanging_sign_step";
    pub const BLOCK_BAMBOO_WOOD_PRESSURE_PLATE_CLICK_OFF: &str = "minecraft:block_bamboo_wood_pressure_plate_click_off";
    pub const BLOCK_BAMBOO_WOOD_PRESSURE_PLATE_CLICK_ON: &str = "minecraft:block_bamboo_wood_pressure_plate_click_on";
    pub const BLOCK_BAMBOO_WOOD_TRAPDOOR_CLOSE: &str = "minecraft:block_bamboo_wood_trapdoor_close";
    pub const BLOCK_BAMBOO_WOOD_TRAPDOOR_OPEN: &str = "minecraft:block_bamboo_wood_trapdoor_open";
    pub const BLOCK_BARREL_CLOSE: &str = "minecraft:block_barrel_close";
    pub const BLOCK_BARREL_OPEN: &str = "minecraft:block_barrel_open";
    pub const BLOCK_BASALT_BREAK: &str = "minecraft:block_basalt_break";
    pub const BLOCK_BASALT_FALL: &str = "minecraft:block_basalt_fall";
    pub const BLOCK_BASALT_HIT: &str = "minecraft:block_basalt_hit";
    pub const BLOCK_BASALT_PLACE: &str = "minecraft:block_basalt_place";
    pub const BLOCK_BASALT_STEP: &str = "minecraft:block_basalt_step";
    pub const BLOCK_BEACON_ACTIVATE: &str = "minecraft:block_beacon_activate";
    pub const BLOCK_BEACON_AMBIENT: &str = "minecraft:block_beacon_ambient";
    pub const BLOCK_BEACON_DEACTIVATE: &str = "minecraft:block_beacon_deactivate";
    pub const BLOCK_BEACON_POWER_SELECT: &str = "minecraft:block_beacon_power_select";
    pub const BLOCK_BEEHIVE_DRIP: &str = "minecraft:block_beehive_drip";
    pub const BLOCK_BEEHIVE_ENTER: &str = "minecraft:block_beehive_enter";
    pub const BLOCK_BEEHIVE_EXIT: &str = "minecraft:block_beehive_exit";
    pub const BLOCK_BEEHIVE_SHEAR: &str = "minecraft:block_beehive_shear";
    pub const BLOCK_BEEHIVE_WORK: &str = "minecraft:block_beehive_work";
    pub const BLOCK_BELL_RESONATE: &str = "minecraft:block_bell_resonate";
    pub const BLOCK_BELL_USE: &str = "minecraft:block_bell_use";
    pub const BLOCK_BIG_DRIPLEAF_BREAK: &str = "minecraft:block_big_dripleaf_break";
    pub const BLOCK_BIG_DRIPLEAF_FALL: &str = "minecraft:block_big_dripleaf_fall";
    pub const BLOCK_BIG_DRIPLEAF_HIT: &str = "minecraft:block_big_dripleaf_hit";
    pub const BLOCK_BIG_DRIPLEAF_PLACE: &str = "minecraft:block_big_dripleaf_place";
    pub const BLOCK_BIG_DRIPLEAF_STEP: &str = "minecraft:block_big_dripleaf_step";
    pub const BLOCK_BIG_DRIPLEAF_TILT_DOWN: &str = "minecraft:block_big_dripleaf_tilt_down";
    pub const BLOCK_BIG_DRIPLEAF_TILT_UP: &str = "minecraft:block_big_dripleaf_tilt_up";
    pub const BLOCK_BLASTFURNACE_FIRE_CRACKLE: &str = "minecraft:block_blastfurnace_fire_crackle";
    pub const BLOCK_BONE_BLOCK_BREAK: &str = "minecraft:block_bone_block_break";
    pub const BLOCK_BONE_BLOCK_FALL: &str = "minecraft:block_bone_block_fall";
    pub const BLOCK_BONE_BLOCK_HIT: &str = "minecraft:block_bone_block_hit";
    pub const BLOCK_BONE_BLOCK_PLACE: &str = "minecraft:block_bone_block_place";
    pub const BLOCK_BONE_BLOCK_STEP: &str = "minecraft:block_bone_block_step";
    pub const BLOCK_BREWING_STAND_BREW: &str = "minecraft:block_brewing_stand_brew";
    pub const BLOCK_BUBBLE_COLUMN_BUBBLE_POP: &str = "minecraft:block_bubble_column_bubble_pop";
    pub const BLOCK_BUBBLE_COLUMN_UPWARDS_AMBIENT: &str = "minecraft:block_bubble_column_upwards_ambient";
    pub const BLOCK_BUBBLE_COLUMN_UPWARDS_INSIDE: &str = "minecraft:block_bubble_column_upwards_inside";
    pub const BLOCK_BUBBLE_COLUMN_WHIRLPOOL_AMBIENT: &str = "minecraft:block_bubble_column_whirlpool_ambient";
    pub const BLOCK_BUBBLE_COLUMN_WHIRLPOOL_INSIDE: &str = "minecraft:block_bubble_column_whirlpool_inside";
    pub const BLOCK_CACTUS_FLOWER_BREAK: &str = "minecraft:block_cactus_flower_break";
    pub const BLOCK_CACTUS_FLOWER_PLACE: &str = "minecraft:block_cactus_flower_place";
    pub const BLOCK_CAKE_ADD_CANDLE: &str = "minecraft:block_cake_add_candle";
    pub const BLOCK_CALCITE_BREAK: &str = "minecraft:block_calcite_break";
    pub const BLOCK_CALCITE_FALL: &str = "minecraft:block_calcite_fall";
    pub const BLOCK_CALCITE_HIT: &str = "minecraft:block_calcite_hit";
    pub const BLOCK_CALCITE_PLACE: &str = "minecraft:block_calcite_place";
    pub const BLOCK_CALCITE_STEP: &str = "minecraft:block_calcite_step";
    pub const BLOCK_CAMPFIRE_CRACKLE: &str = "minecraft:block_campfire_crackle";
    pub const BLOCK_CANDLE_AMBIENT: &str = "minecraft:block_candle_ambient";
    pub const BLOCK_CANDLE_BREAK: &str = "minecraft:block_candle_break";
    pub const BLOCK_CANDLE_EXTINGUISH: &str = "minecraft:block_candle_extinguish";
    pub const BLOCK_CANDLE_FALL: &str = "minecraft:block_candle_fall";
    pub const BLOCK_CANDLE_HIT: &str = "minecraft:block_candle_hit";
    pub const BLOCK_CANDLE_PLACE: &str = "minecraft:block_candle_place";
    pub const BLOCK_CANDLE_STEP: &str = "minecraft:block_candle_step";
    pub const BLOCK_CAVE_VINES_BREAK: &str = "minecraft:block_cave_vines_break";
    pub const BLOCK_CAVE_VINES_FALL: &str = "minecraft:block_cave_vines_fall";
    pub const BLOCK_CAVE_VINES_HIT: &str = "minecraft:block_cave_vines_hit";
    pub const BLOCK_CAVE_VINES_PICK_BERRIES: &str = "minecraft:block_cave_vines_pick_berries";
    pub const BLOCK_CAVE_VINES_PLACE: &str = "minecraft:block_cave_vines_place";
    pub const BLOCK_CAVE_VINES_STEP: &str = "minecraft:block_cave_vines_step";
    pub const BLOCK_CHAIN_BREAK: &str = "minecraft:block_chain_break";
    pub const BLOCK_CHAIN_FALL: &str = "minecraft:block_chain_fall";
    pub const BLOCK_CHAIN_HIT: &str = "minecraft:block_chain_hit";
    pub const BLOCK_CHAIN_PLACE: &str = "minecraft:block_chain_place";
    pub const BLOCK_CHAIN_STEP: &str = "minecraft:block_chain_step";
    pub const BLOCK_CHERRY_LEAVES_BREAK: &str = "minecraft:block_cherry_leaves_break";
    pub const BLOCK_CHERRY_LEAVES_FALL: &str = "minecraft:block_cherry_leaves_fall";
    pub const BLOCK_CHERRY_LEAVES_HIT: &str = "minecraft:block_cherry_leaves_hit";
    pub const BLOCK_CHERRY_LEAVES_PLACE: &str = "minecraft:block_cherry_leaves_place";
    pub const BLOCK_CHERRY_LEAVES_STEP: &str = "minecraft:block_cherry_leaves_step";
    pub const BLOCK_CHERRY_SAPLING_BREAK: &str = "minecraft:block_cherry_sapling_break";
    pub const BLOCK_CHERRY_SAPLING_FALL: &str = "minecraft:block_cherry_sapling_fall";
    pub const BLOCK_CHERRY_SAPLING_HIT: &str = "minecraft:block_cherry_sapling_hit";
    pub const BLOCK_CHERRY_SAPLING_PLACE: &str = "minecraft:block_cherry_sapling_place";
    pub const BLOCK_CHERRY_SAPLING_STEP: &str = "minecraft:block_cherry_sapling_step";
    pub const BLOCK_CHERRY_WOOD_BREAK: &str = "minecraft:block_cherry_wood_break";
    pub const BLOCK_CHERRY_WOOD_FALL: &str = "minecraft:block_cherry_wood_fall";
    pub const BLOCK_CHERRY_WOOD_HIT: &str = "minecraft:block_cherry_wood_hit";
    pub const BLOCK_CHERRY_WOOD_PLACE: &str = "minecraft:block_cherry_wood_place";
    pub const BLOCK_CHERRY_WOOD_STEP: &str = "minecraft:block_cherry_wood_step";
    pub const BLOCK_CHERRY_WOOD_BUTTON_CLICK_OFF: &str = "minecraft:block_cherry_wood_button_click_off";
    pub const BLOCK_CHERRY_WOOD_BUTTON_CLICK_ON: &str = "minecraft:block_cherry_wood_button_click_on";
    pub const BLOCK_CHERRY_WOOD_DOOR_CLOSE: &str = "minecraft:block_cherry_wood_door_close";
    pub const BLOCK_CHERRY_WOOD_DOOR_OPEN: &str = "minecraft:block_cherry_wood_door_open";
    pub const BLOCK_CHERRY_WOOD_FENCE_GATE_CLOSE: &str = "minecraft:block_cherry_wood_fence_gate_close";
    pub const BLOCK_CHERRY_WOOD_FENCE_GATE_OPEN: &str = "minecraft:block_cherry_wood_fence_gate_open";
    pub const BLOCK_CHERRY_WOOD_HANGING_SIGN_BREAK: &str = "minecraft:block_cherry_wood_hanging_sign_break";
    pub const BLOCK_CHERRY_WOOD_HANGING_SIGN_FALL: &str = "minecraft:block_cherry_wood_hanging_sign_fall";
    pub const BLOCK_CHERRY_WOOD_HANGING_SIGN_HIT: &str = "minecraft:block_cherry_wood_hanging_sign_hit";
    pub const BLOCK_CHERRY_WOOD_HANGING_SIGN_PLACE: &str = "minecraft:block_cherry_wood_hanging_sign_place";
    pub const BLOCK_CHERRY_WOOD_HANGING_SIGN_STEP: &str = "minecraft:block_cherry_wood_hanging_sign_step";
    pub const BLOCK_CHERRY_WOOD_PRESSURE_PLATE_CLICK_OFF: &str = "minecraft:block_cherry_wood_pressure_plate_click_off";
    pub const BLOCK_CHERRY_WOOD_PRESSURE_PLATE_CLICK_ON: &str = "minecraft:block_cherry_wood_pressure_plate_click_on";
    pub const BLOCK_CHERRY_WOOD_TRAPDOOR_CLOSE: &str = "minecraft:block_cherry_wood_trapdoor_close";
    pub const BLOCK_CHERRY_WOOD_TRAPDOOR_OPEN: &str = "minecraft:block_cherry_wood_trapdoor_open";
    pub const BLOCK_CHEST_CLOSE: &str = "minecraft:block_chest_close";
    pub const BLOCK_CHEST_LOCKED: &str = "minecraft:block_chest_locked";
    pub const BLOCK_CHEST_OPEN: &str = "minecraft:block_chest_open";
    pub const BLOCK_CHISELED_BOOKSHELF_BREAK: &str = "minecraft:block_chiseled_bookshelf_break";
    pub const BLOCK_CHISELED_BOOKSHELF_FALL: &str = "minecraft:block_chiseled_bookshelf_fall";
    pub const BLOCK_CHISELED_BOOKSHELF_HIT: &str = "minecraft:block_chiseled_bookshelf_hit";
    pub const BLOCK_CHISELED_BOOKSHELF_INSERT: &str = "minecraft:block_chiseled_bookshelf_insert";
    pub const BLOCK_CHISELED_BOOKSHELF_INSERT_ENCHANTED: &str = "minecraft:block_chiseled_bookshelf_insert_enchanted";
    pub const BLOCK_CHISELED_BOOKSHELF_PICKUP: &str = "minecraft:block_chiseled_bookshelf_pickup";
    pub const BLOCK_CHISELED_BOOKSHELF_PICKUP_ENCHANTED: &str = "minecraft:block_chiseled_bookshelf_pickup_enchanted";
    pub const BLOCK_CHISELED_BOOKSHELF_PLACE: &str = "minecraft:block_chiseled_bookshelf_place";
    pub const BLOCK_CHISELED_BOOKSHELF_STEP: &str = "minecraft:block_chiseled_bookshelf_step";
    pub const BLOCK_CHORUS_FLOWER_DEATH: &str = "minecraft:block_chorus_flower_death";
    pub const BLOCK_CHORUS_FLOWER_GROW: &str = "minecraft:block_chorus_flower_grow";
    pub const BLOCK_CINNABAR_BREAK: &str = "minecraft:block_cinnabar_break";
    pub const BLOCK_CINNABAR_FALL: &str = "minecraft:block_cinnabar_fall";
    pub const BLOCK_CINNABAR_HIT: &str = "minecraft:block_cinnabar_hit";
    pub const BLOCK_CINNABAR_PLACE: &str = "minecraft:block_cinnabar_place";
    pub const BLOCK_CINNABAR_STEP: &str = "minecraft:block_cinnabar_step";
    pub const BLOCK_COBWEB_BREAK: &str = "minecraft:block_cobweb_break";
    pub const BLOCK_COBWEB_FALL: &str = "minecraft:block_cobweb_fall";
    pub const BLOCK_COBWEB_HIT: &str = "minecraft:block_cobweb_hit";
    pub const BLOCK_COBWEB_PLACE: &str = "minecraft:block_cobweb_place";
    pub const BLOCK_COBWEB_STEP: &str = "minecraft:block_cobweb_step";
    pub const BLOCK_COMPARATOR_CLICK: &str = "minecraft:block_comparator_click";
    pub const BLOCK_COMPOSTER_EMPTY: &str = "minecraft:block_composter_empty";
    pub const BLOCK_COMPOSTER_FILL: &str = "minecraft:block_composter_fill";
    pub const BLOCK_COMPOSTER_FILL_SUCCESS: &str = "minecraft:block_composter_fill_success";
    pub const BLOCK_COMPOSTER_READY: &str = "minecraft:block_composter_ready";
    pub const BLOCK_CONDUIT_ACTIVATE: &str = "minecraft:block_conduit_activate";
    pub const BLOCK_CONDUIT_AMBIENT: &str = "minecraft:block_conduit_ambient";
    pub const BLOCK_CONDUIT_AMBIENT_SHORT: &str = "minecraft:block_conduit_ambient_short";
    pub const BLOCK_CONDUIT_ATTACK_TARGET: &str = "minecraft:block_conduit_attack_target";
    pub const BLOCK_CONDUIT_DEACTIVATE: &str = "minecraft:block_conduit_deactivate";
    pub const BLOCK_COPPER_BREAK: &str = "minecraft:block_copper_break";
    pub const BLOCK_COPPER_FALL: &str = "minecraft:block_copper_fall";
    pub const BLOCK_COPPER_HIT: &str = "minecraft:block_copper_hit";
    pub const BLOCK_COPPER_PLACE: &str = "minecraft:block_copper_place";
    pub const BLOCK_COPPER_STEP: &str = "minecraft:block_copper_step";
    pub const BLOCK_COPPER_BULB_BREAK: &str = "minecraft:block_copper_bulb_break";
    pub const BLOCK_COPPER_BULB_FALL: &str = "minecraft:block_copper_bulb_fall";
    pub const BLOCK_COPPER_BULB_HIT: &str = "minecraft:block_copper_bulb_hit";
    pub const BLOCK_COPPER_BULB_PLACE: &str = "minecraft:block_copper_bulb_place";
    pub const BLOCK_COPPER_BULB_STEP: &str = "minecraft:block_copper_bulb_step";
    pub const BLOCK_COPPER_BULB_TURN_OFF: &str = "minecraft:block_copper_bulb_turn_off";
    pub const BLOCK_COPPER_BULB_TURN_ON: &str = "minecraft:block_copper_bulb_turn_on";
    pub const BLOCK_COPPER_CHEST_CLOSE: &str = "minecraft:block_copper_chest_close";
    pub const BLOCK_COPPER_CHEST_OPEN: &str = "minecraft:block_copper_chest_open";
    pub const BLOCK_COPPER_CHEST_OXIDIZED_CLOSE: &str = "minecraft:block_copper_chest_oxidized_close";
    pub const BLOCK_COPPER_CHEST_OXIDIZED_OPEN: &str = "minecraft:block_copper_chest_oxidized_open";
    pub const BLOCK_COPPER_CHEST_WEATHERED_CLOSE: &str = "minecraft:block_copper_chest_weathered_close";
    pub const BLOCK_COPPER_CHEST_WEATHERED_OPEN: &str = "minecraft:block_copper_chest_weathered_open";
    pub const BLOCK_COPPER_DOOR_CLOSE: &str = "minecraft:block_copper_door_close";
    pub const BLOCK_COPPER_DOOR_OPEN: &str = "minecraft:block_copper_door_open";
    pub const BLOCK_COPPER_GOLEM_STATUE_BREAK: &str = "minecraft:block_copper_golem_statue_break";
    pub const BLOCK_COPPER_GOLEM_STATUE_FALL: &str = "minecraft:block_copper_golem_statue_fall";
    pub const BLOCK_COPPER_GOLEM_STATUE_HIT: &str = "minecraft:block_copper_golem_statue_hit";
    pub const BLOCK_COPPER_GOLEM_STATUE_PLACE: &str = "minecraft:block_copper_golem_statue_place";
    pub const BLOCK_COPPER_GOLEM_STATUE_STEP: &str = "minecraft:block_copper_golem_statue_step";
    pub const BLOCK_COPPER_GRATE_BREAK: &str = "minecraft:block_copper_grate_break";
    pub const BLOCK_COPPER_GRATE_FALL: &str = "minecraft:block_copper_grate_fall";
    pub const BLOCK_COPPER_GRATE_HIT: &str = "minecraft:block_copper_grate_hit";
    pub const BLOCK_COPPER_GRATE_PLACE: &str = "minecraft:block_copper_grate_place";
    pub const BLOCK_COPPER_GRATE_STEP: &str = "minecraft:block_copper_grate_step";
    pub const BLOCK_COPPER_TRAPDOOR_CLOSE: &str = "minecraft:block_copper_trapdoor_close";
    pub const BLOCK_COPPER_TRAPDOOR_OPEN: &str = "minecraft:block_copper_trapdoor_open";
    pub const BLOCK_CORAL_BLOCK_BREAK: &str = "minecraft:block_coral_block_break";
    pub const BLOCK_CORAL_BLOCK_FALL: &str = "minecraft:block_coral_block_fall";
    pub const BLOCK_CORAL_BLOCK_HIT: &str = "minecraft:block_coral_block_hit";
    pub const BLOCK_CORAL_BLOCK_PLACE: &str = "minecraft:block_coral_block_place";
    pub const BLOCK_CORAL_BLOCK_STEP: &str = "minecraft:block_coral_block_step";
    pub const BLOCK_CRAFTER_CRAFT: &str = "minecraft:block_crafter_craft";
    pub const BLOCK_CRAFTER_FAIL: &str = "minecraft:block_crafter_fail";
    pub const BLOCK_CREAKING_HEART_BREAK: &str = "minecraft:block_creaking_heart_break";
    pub const BLOCK_CREAKING_HEART_FALL: &str = "minecraft:block_creaking_heart_fall";
    pub const BLOCK_CREAKING_HEART_HIT: &str = "minecraft:block_creaking_heart_hit";
    pub const BLOCK_CREAKING_HEART_HURT: &str = "minecraft:block_creaking_heart_hurt";
    pub const BLOCK_CREAKING_HEART_IDLE: &str = "minecraft:block_creaking_heart_idle";
    pub const BLOCK_CREAKING_HEART_PLACE: &str = "minecraft:block_creaking_heart_place";
    pub const BLOCK_CREAKING_HEART_SPAWN: &str = "minecraft:block_creaking_heart_spawn";
    pub const BLOCK_CREAKING_HEART_STEP: &str = "minecraft:block_creaking_heart_step";
    pub const BLOCK_CROP_BREAK: &str = "minecraft:block_crop_break";
    pub const BLOCK_DEADBUSH_IDLE: &str = "minecraft:block_deadbush_idle";
    pub const BLOCK_DECORATED_POT_BREAK: &str = "minecraft:block_decorated_pot_break";
    pub const BLOCK_DECORATED_POT_FALL: &str = "minecraft:block_decorated_pot_fall";
    pub const BLOCK_DECORATED_POT_HIT: &str = "minecraft:block_decorated_pot_hit";
    pub const BLOCK_DECORATED_POT_INSERT: &str = "minecraft:block_decorated_pot_insert";
    pub const BLOCK_DECORATED_POT_INSERT_FAIL: &str = "minecraft:block_decorated_pot_insert_fail";
    pub const BLOCK_DECORATED_POT_PLACE: &str = "minecraft:block_decorated_pot_place";
    pub const BLOCK_DECORATED_POT_SHATTER: &str = "minecraft:block_decorated_pot_shatter";
    pub const BLOCK_DECORATED_POT_STEP: &str = "minecraft:block_decorated_pot_step";
    pub const BLOCK_DEEPSLATE_BREAK: &str = "minecraft:block_deepslate_break";
    pub const BLOCK_DEEPSLATE_FALL: &str = "minecraft:block_deepslate_fall";
    pub const BLOCK_DEEPSLATE_HIT: &str = "minecraft:block_deepslate_hit";
    pub const BLOCK_DEEPSLATE_PLACE: &str = "minecraft:block_deepslate_place";
    pub const BLOCK_DEEPSLATE_STEP: &str = "minecraft:block_deepslate_step";
    pub const BLOCK_DEEPSLATE_BRICKS_BREAK: &str = "minecraft:block_deepslate_bricks_break";
    pub const BLOCK_DEEPSLATE_BRICKS_FALL: &str = "minecraft:block_deepslate_bricks_fall";
    pub const BLOCK_DEEPSLATE_BRICKS_HIT: &str = "minecraft:block_deepslate_bricks_hit";
    pub const BLOCK_DEEPSLATE_BRICKS_PLACE: &str = "minecraft:block_deepslate_bricks_place";
    pub const BLOCK_DEEPSLATE_BRICKS_STEP: &str = "minecraft:block_deepslate_bricks_step";
    pub const BLOCK_DEEPSLATE_TILES_BREAK: &str = "minecraft:block_deepslate_tiles_break";
    pub const BLOCK_DEEPSLATE_TILES_FALL: &str = "minecraft:block_deepslate_tiles_fall";
    pub const BLOCK_DEEPSLATE_TILES_HIT: &str = "minecraft:block_deepslate_tiles_hit";
    pub const BLOCK_DEEPSLATE_TILES_PLACE: &str = "minecraft:block_deepslate_tiles_place";
    pub const BLOCK_DEEPSLATE_TILES_STEP: &str = "minecraft:block_deepslate_tiles_step";
    pub const BLOCK_DISPENSER_DISPENSE: &str = "minecraft:block_dispenser_dispense";
    pub const BLOCK_DISPENSER_FAIL: &str = "minecraft:block_dispenser_fail";
    pub const BLOCK_DISPENSER_LAUNCH: &str = "minecraft:block_dispenser_launch";
    pub const BLOCK_DRIED_GHAST_AMBIENT: &str = "minecraft:block_dried_ghast_ambient";
    pub const BLOCK_DRIED_GHAST_AMBIENT_WATER: &str = "minecraft:block_dried_ghast_ambient_water";
    pub const BLOCK_DRIED_GHAST_BREAK: &str = "minecraft:block_dried_ghast_break";
    pub const BLOCK_DRIED_GHAST_FALL: &str = "minecraft:block_dried_ghast_fall";
    pub const BLOCK_DRIED_GHAST_PLACE: &str = "minecraft:block_dried_ghast_place";
    pub const BLOCK_DRIED_GHAST_PLACE_IN_WATER: &str = "minecraft:block_dried_ghast_place_in_water";
    pub const BLOCK_DRIED_GHAST_STEP: &str = "minecraft:block_dried_ghast_step";
    pub const BLOCK_DRIED_GHAST_TRANSITION: &str = "minecraft:block_dried_ghast_transition";
    pub const BLOCK_DRIPSTONE_BLOCK_BREAK: &str = "minecraft:block_dripstone_block_break";
    pub const BLOCK_DRIPSTONE_BLOCK_FALL: &str = "minecraft:block_dripstone_block_fall";
    pub const BLOCK_DRIPSTONE_BLOCK_HIT: &str = "minecraft:block_dripstone_block_hit";
    pub const BLOCK_DRIPSTONE_BLOCK_PLACE: &str = "minecraft:block_dripstone_block_place";
    pub const BLOCK_DRIPSTONE_BLOCK_STEP: &str = "minecraft:block_dripstone_block_step";
    pub const BLOCK_DRY_GRASS_AMBIENT: &str = "minecraft:block_dry_grass_ambient";
    pub const BLOCK_ENCHANTMENT_TABLE_USE: &str = "minecraft:block_enchantment_table_use";
    pub const BLOCK_END_GATEWAY_SPAWN: &str = "minecraft:block_end_gateway_spawn";
    pub const BLOCK_END_PORTAL_SPAWN: &str = "minecraft:block_end_portal_spawn";
    pub const BLOCK_END_PORTAL_FRAME_FILL: &str = "minecraft:block_end_portal_frame_fill";
    pub const BLOCK_ENDER_CHEST_CLOSE: &str = "minecraft:block_ender_chest_close";
    pub const BLOCK_ENDER_CHEST_OPEN: &str = "minecraft:block_ender_chest_open";
    pub const BLOCK_EYEBLOSSOM_CLOSE: &str = "minecraft:block_eyeblossom_close";
    pub const BLOCK_EYEBLOSSOM_CLOSE_LONG: &str = "minecraft:block_eyeblossom_close_long";
    pub const BLOCK_EYEBLOSSOM_IDLE: &str = "minecraft:block_eyeblossom_idle";
    pub const BLOCK_EYEBLOSSOM_OPEN: &str = "minecraft:block_eyeblossom_open";
    pub const BLOCK_EYEBLOSSOM_OPEN_LONG: &str = "minecraft:block_eyeblossom_open_long";
    pub const BLOCK_FENCE_GATE_CLOSE: &str = "minecraft:block_fence_gate_close";
    pub const BLOCK_FENCE_GATE_OPEN: &str = "minecraft:block_fence_gate_open";
    pub const BLOCK_FIRE_AMBIENT: &str = "minecraft:block_fire_ambient";
    pub const BLOCK_FIRE_EXTINGUISH: &str = "minecraft:block_fire_extinguish";
    pub const BLOCK_FIREFLY_BUSH_IDLE: &str = "minecraft:block_firefly_bush_idle";
    pub const BLOCK_FLOWERING_AZALEA_BREAK: &str = "minecraft:block_flowering_azalea_break";
    pub const BLOCK_FLOWERING_AZALEA_FALL: &str = "minecraft:block_flowering_azalea_fall";
    pub const BLOCK_FLOWERING_AZALEA_HIT: &str = "minecraft:block_flowering_azalea_hit";
    pub const BLOCK_FLOWERING_AZALEA_PLACE: &str = "minecraft:block_flowering_azalea_place";
    pub const BLOCK_FLOWERING_AZALEA_STEP: &str = "minecraft:block_flowering_azalea_step";
    pub const BLOCK_FROGLIGHT_BREAK: &str = "minecraft:block_froglight_break";
    pub const BLOCK_FROGLIGHT_FALL: &str = "minecraft:block_froglight_fall";
    pub const BLOCK_FROGLIGHT_HIT: &str = "minecraft:block_froglight_hit";
    pub const BLOCK_FROGLIGHT_PLACE: &str = "minecraft:block_froglight_place";
    pub const BLOCK_FROGLIGHT_STEP: &str = "minecraft:block_froglight_step";
    pub const BLOCK_FROGSPAWN_BREAK: &str = "minecraft:block_frogspawn_break";
    pub const BLOCK_FROGSPAWN_FALL: &str = "minecraft:block_frogspawn_fall";
    pub const BLOCK_FROGSPAWN_HATCH: &str = "minecraft:block_frogspawn_hatch";
    pub const BLOCK_FROGSPAWN_HIT: &str = "minecraft:block_frogspawn_hit";
    pub const BLOCK_FROGSPAWN_PLACE: &str = "minecraft:block_frogspawn_place";
    pub const BLOCK_FROGSPAWN_STEP: &str = "minecraft:block_frogspawn_step";
    pub const BLOCK_FUNGUS_BREAK: &str = "minecraft:block_fungus_break";
    pub const BLOCK_FUNGUS_FALL: &str = "minecraft:block_fungus_fall";
    pub const BLOCK_FUNGUS_HIT: &str = "minecraft:block_fungus_hit";
    pub const BLOCK_FUNGUS_PLACE: &str = "minecraft:block_fungus_place";
    pub const BLOCK_FUNGUS_STEP: &str = "minecraft:block_fungus_step";
    pub const BLOCK_FURNACE_FIRE_CRACKLE: &str = "minecraft:block_furnace_fire_crackle";
    pub const BLOCK_GILDED_BLACKSTONE_BREAK: &str = "minecraft:block_gilded_blackstone_break";
    pub const BLOCK_GILDED_BLACKSTONE_FALL: &str = "minecraft:block_gilded_blackstone_fall";
    pub const BLOCK_GILDED_BLACKSTONE_HIT: &str = "minecraft:block_gilded_blackstone_hit";
    pub const BLOCK_GILDED_BLACKSTONE_PLACE: &str = "minecraft:block_gilded_blackstone_place";
    pub const BLOCK_GILDED_BLACKSTONE_STEP: &str = "minecraft:block_gilded_blackstone_step";
    pub const BLOCK_GLASS_BREAK: &str = "minecraft:block_glass_break";
    pub const BLOCK_GLASS_FALL: &str = "minecraft:block_glass_fall";
    pub const BLOCK_GLASS_HIT: &str = "minecraft:block_glass_hit";
    pub const BLOCK_GLASS_PLACE: &str = "minecraft:block_glass_place";
    pub const BLOCK_GLASS_STEP: &str = "minecraft:block_glass_step";
    pub const BLOCK_GRASS_BREAK: &str = "minecraft:block_grass_break";
    pub const BLOCK_GRASS_FALL: &str = "minecraft:block_grass_fall";
    pub const BLOCK_GRASS_HIT: &str = "minecraft:block_grass_hit";
    pub const BLOCK_GRASS_PLACE: &str = "minecraft:block_grass_place";
    pub const BLOCK_GRASS_STEP: &str = "minecraft:block_grass_step";
    pub const BLOCK_GRAVEL_BREAK: &str = "minecraft:block_gravel_break";
    pub const BLOCK_GRAVEL_FALL: &str = "minecraft:block_gravel_fall";
    pub const BLOCK_GRAVEL_HIT: &str = "minecraft:block_gravel_hit";
    pub const BLOCK_GRAVEL_PLACE: &str = "minecraft:block_gravel_place";
    pub const BLOCK_GRAVEL_STEP: &str = "minecraft:block_gravel_step";
    pub const BLOCK_GRINDSTONE_USE: &str = "minecraft:block_grindstone_use";
    pub const BLOCK_GROWING_PLANT_CROP: &str = "minecraft:block_growing_plant_crop";
    pub const BLOCK_HANGING_ROOTS_BREAK: &str = "minecraft:block_hanging_roots_break";
    pub const BLOCK_HANGING_ROOTS_FALL: &str = "minecraft:block_hanging_roots_fall";
    pub const BLOCK_HANGING_ROOTS_HIT: &str = "minecraft:block_hanging_roots_hit";
    pub const BLOCK_HANGING_ROOTS_PLACE: &str = "minecraft:block_hanging_roots_place";
    pub const BLOCK_HANGING_ROOTS_STEP: &str = "minecraft:block_hanging_roots_step";
    pub const BLOCK_HANGING_SIGN_BREAK: &str = "minecraft:block_hanging_sign_break";
    pub const BLOCK_HANGING_SIGN_FALL: &str = "minecraft:block_hanging_sign_fall";
    pub const BLOCK_HANGING_SIGN_HIT: &str = "minecraft:block_hanging_sign_hit";
    pub const BLOCK_HANGING_SIGN_PLACE: &str = "minecraft:block_hanging_sign_place";
    pub const BLOCK_HANGING_SIGN_STEP: &str = "minecraft:block_hanging_sign_step";
    pub const BLOCK_HANGING_SIGN_WAXED_INTERACT_FAIL: &str = "minecraft:block_hanging_sign_waxed_interact_fail";
    pub const BLOCK_HEAVY_CORE_BREAK: &str = "minecraft:block_heavy_core_break";
    pub const BLOCK_HEAVY_CORE_FALL: &str = "minecraft:block_heavy_core_fall";
    pub const BLOCK_HEAVY_CORE_HIT: &str = "minecraft:block_heavy_core_hit";
    pub const BLOCK_HEAVY_CORE_PLACE: &str = "minecraft:block_heavy_core_place";
    pub const BLOCK_HEAVY_CORE_STEP: &str = "minecraft:block_heavy_core_step";
    pub const BLOCK_HONEY_BLOCK_BREAK: &str = "minecraft:block_honey_block_break";
    pub const BLOCK_HONEY_BLOCK_FALL: &str = "minecraft:block_honey_block_fall";
    pub const BLOCK_HONEY_BLOCK_HIT: &str = "minecraft:block_honey_block_hit";
    pub const BLOCK_HONEY_BLOCK_PLACE: &str = "minecraft:block_honey_block_place";
    pub const BLOCK_HONEY_BLOCK_SLIDE: &str = "minecraft:block_honey_block_slide";
    pub const BLOCK_HONEY_BLOCK_STEP: &str = "minecraft:block_honey_block_step";
    pub const BLOCK_IRON_BREAK: &str = "minecraft:block_iron_break";
    pub const BLOCK_IRON_FALL: &str = "minecraft:block_iron_fall";
    pub const BLOCK_IRON_HIT: &str = "minecraft:block_iron_hit";
    pub const BLOCK_IRON_PLACE: &str = "minecraft:block_iron_place";
    pub const BLOCK_IRON_STEP: &str = "minecraft:block_iron_step";
    pub const BLOCK_IRON_DOOR_CLOSE: &str = "minecraft:block_iron_door_close";
    pub const BLOCK_IRON_DOOR_OPEN: &str = "minecraft:block_iron_door_open";
    pub const BLOCK_IRON_TRAPDOOR_CLOSE: &str = "minecraft:block_iron_trapdoor_close";
    pub const BLOCK_IRON_TRAPDOOR_OPEN: &str = "minecraft:block_iron_trapdoor_open";
    pub const BLOCK_LADDER_BREAK: &str = "minecraft:block_ladder_break";
    pub const BLOCK_LADDER_FALL: &str = "minecraft:block_ladder_fall";
    pub const BLOCK_LADDER_HIT: &str = "minecraft:block_ladder_hit";
    pub const BLOCK_LADDER_PLACE: &str = "minecraft:block_ladder_place";
    pub const BLOCK_LADDER_STEP: &str = "minecraft:block_ladder_step";
    pub const BLOCK_LANTERN_BREAK: &str = "minecraft:block_lantern_break";
    pub const BLOCK_LANTERN_FALL: &str = "minecraft:block_lantern_fall";
    pub const BLOCK_LANTERN_HIT: &str = "minecraft:block_lantern_hit";
    pub const BLOCK_LANTERN_PLACE: &str = "minecraft:block_lantern_place";
    pub const BLOCK_LANTERN_STEP: &str = "minecraft:block_lantern_step";
    pub const BLOCK_LARGE_AMETHYST_BUD_BREAK: &str = "minecraft:block_large_amethyst_bud_break";
    pub const BLOCK_LARGE_AMETHYST_BUD_PLACE: &str = "minecraft:block_large_amethyst_bud_place";
    pub const BLOCK_LAVA_AMBIENT: &str = "minecraft:block_lava_ambient";
    pub const BLOCK_LAVA_EXTINGUISH: &str = "minecraft:block_lava_extinguish";
    pub const BLOCK_LAVA_POP: &str = "minecraft:block_lava_pop";
    pub const BLOCK_LEAF_LITTER_BREAK: &str = "minecraft:block_leaf_litter_break";
    pub const BLOCK_LEAF_LITTER_FALL: &str = "minecraft:block_leaf_litter_fall";
    pub const BLOCK_LEAF_LITTER_HIT: &str = "minecraft:block_leaf_litter_hit";
    pub const BLOCK_LEAF_LITTER_PLACE: &str = "minecraft:block_leaf_litter_place";
    pub const BLOCK_LEAF_LITTER_STEP: &str = "minecraft:block_leaf_litter_step";
    pub const BLOCK_LEVER_CLICK: &str = "minecraft:block_lever_click";
    pub const BLOCK_LILY_PAD_PLACE: &str = "minecraft:block_lily_pad_place";
    pub const BLOCK_LODESTONE_BREAK: &str = "minecraft:block_lodestone_break";
    pub const BLOCK_LODESTONE_FALL: &str = "minecraft:block_lodestone_fall";
    pub const BLOCK_LODESTONE_HIT: &str = "minecraft:block_lodestone_hit";
    pub const BLOCK_LODESTONE_PLACE: &str = "minecraft:block_lodestone_place";
    pub const BLOCK_LODESTONE_STEP: &str = "minecraft:block_lodestone_step";
    pub const BLOCK_MANGROVE_ROOTS_BREAK: &str = "minecraft:block_mangrove_roots_break";
    pub const BLOCK_MANGROVE_ROOTS_FALL: &str = "minecraft:block_mangrove_roots_fall";
    pub const BLOCK_MANGROVE_ROOTS_HIT: &str = "minecraft:block_mangrove_roots_hit";
    pub const BLOCK_MANGROVE_ROOTS_PLACE: &str = "minecraft:block_mangrove_roots_place";
    pub const BLOCK_MANGROVE_ROOTS_STEP: &str = "minecraft:block_mangrove_roots_step";
    pub const BLOCK_MEDIUM_AMETHYST_BUD_BREAK: &str = "minecraft:block_medium_amethyst_bud_break";
    pub const BLOCK_MEDIUM_AMETHYST_BUD_PLACE: &str = "minecraft:block_medium_amethyst_bud_place";
    pub const BLOCK_METAL_BREAK: &str = "minecraft:block_metal_break";
    pub const BLOCK_METAL_FALL: &str = "minecraft:block_metal_fall";
    pub const BLOCK_METAL_HIT: &str = "minecraft:block_metal_hit";
    pub const BLOCK_METAL_PLACE: &str = "minecraft:block_metal_place";
    pub const BLOCK_METAL_STEP: &str = "minecraft:block_metal_step";
    pub const BLOCK_METAL_PRESSURE_PLATE_CLICK_OFF: &str = "minecraft:block_metal_pressure_plate_click_off";
    pub const BLOCK_METAL_PRESSURE_PLATE_CLICK_ON: &str = "minecraft:block_metal_pressure_plate_click_on";
    pub const BLOCK_MOSS_BREAK: &str = "minecraft:block_moss_break";
    pub const BLOCK_MOSS_FALL: &str = "minecraft:block_moss_fall";
    pub const BLOCK_MOSS_HIT: &str = "minecraft:block_moss_hit";
    pub const BLOCK_MOSS_PLACE: &str = "minecraft:block_moss_place";
    pub const BLOCK_MOSS_STEP: &str = "minecraft:block_moss_step";
    pub const BLOCK_MOSS_CARPET_BREAK: &str = "minecraft:block_moss_carpet_break";
    pub const BLOCK_MOSS_CARPET_FALL: &str = "minecraft:block_moss_carpet_fall";
    pub const BLOCK_MOSS_CARPET_HIT: &str = "minecraft:block_moss_carpet_hit";
    pub const BLOCK_MOSS_CARPET_PLACE: &str = "minecraft:block_moss_carpet_place";
    pub const BLOCK_MOSS_CARPET_STEP: &str = "minecraft:block_moss_carpet_step";
    pub const BLOCK_MUD_BREAK: &str = "minecraft:block_mud_break";
    pub const BLOCK_MUD_FALL: &str = "minecraft:block_mud_fall";
    pub const BLOCK_MUD_HIT: &str = "minecraft:block_mud_hit";
    pub const BLOCK_MUD_PLACE: &str = "minecraft:block_mud_place";
    pub const BLOCK_MUD_STEP: &str = "minecraft:block_mud_step";
    pub const BLOCK_MUD_BRICKS_BREAK: &str = "minecraft:block_mud_bricks_break";
    pub const BLOCK_MUD_BRICKS_FALL: &str = "minecraft:block_mud_bricks_fall";
    pub const BLOCK_MUD_BRICKS_HIT: &str = "minecraft:block_mud_bricks_hit";
    pub const BLOCK_MUD_BRICKS_PLACE: &str = "minecraft:block_mud_bricks_place";
    pub const BLOCK_MUD_BRICKS_STEP: &str = "minecraft:block_mud_bricks_step";
    pub const BLOCK_MUDDY_MANGROVE_ROOTS_BREAK: &str = "minecraft:block_muddy_mangrove_roots_break";
    pub const BLOCK_MUDDY_MANGROVE_ROOTS_FALL: &str = "minecraft:block_muddy_mangrove_roots_fall";
    pub const BLOCK_MUDDY_MANGROVE_ROOTS_HIT: &str = "minecraft:block_muddy_mangrove_roots_hit";
    pub const BLOCK_MUDDY_MANGROVE_ROOTS_PLACE: &str = "minecraft:block_muddy_mangrove_roots_place";
    pub const BLOCK_MUDDY_MANGROVE_ROOTS_STEP: &str = "minecraft:block_muddy_mangrove_roots_step";
    pub const BLOCK_NETHER_BRICKS_BREAK: &str = "minecraft:block_nether_bricks_break";
    pub const BLOCK_NETHER_BRICKS_FALL: &str = "minecraft:block_nether_bricks_fall";
    pub const BLOCK_NETHER_BRICKS_HIT: &str = "minecraft:block_nether_bricks_hit";
    pub const BLOCK_NETHER_BRICKS_PLACE: &str = "minecraft:block_nether_bricks_place";
    pub const BLOCK_NETHER_BRICKS_STEP: &str = "minecraft:block_nether_bricks_step";
    pub const BLOCK_NETHER_GOLD_ORE_BREAK: &str = "minecraft:block_nether_gold_ore_break";
    pub const BLOCK_NETHER_GOLD_ORE_FALL: &str = "minecraft:block_nether_gold_ore_fall";
    pub const BLOCK_NETHER_GOLD_ORE_HIT: &str = "minecraft:block_nether_gold_ore_hit";
    pub const BLOCK_NETHER_GOLD_ORE_PLACE: &str = "minecraft:block_nether_gold_ore_place";
    pub const BLOCK_NETHER_GOLD_ORE_STEP: &str = "minecraft:block_nether_gold_ore_step";
    pub const BLOCK_NETHER_ORE_BREAK: &str = "minecraft:block_nether_ore_break";
    pub const BLOCK_NETHER_ORE_FALL: &str = "minecraft:block_nether_ore_fall";
    pub const BLOCK_NETHER_ORE_HIT: &str = "minecraft:block_nether_ore_hit";
    pub const BLOCK_NETHER_ORE_PLACE: &str = "minecraft:block_nether_ore_place";
    pub const BLOCK_NETHER_ORE_STEP: &str = "minecraft:block_nether_ore_step";
    pub const BLOCK_NETHER_SPROUTS_BREAK: &str = "minecraft:block_nether_sprouts_break";
    pub const BLOCK_NETHER_SPROUTS_FALL: &str = "minecraft:block_nether_sprouts_fall";
    pub const BLOCK_NETHER_SPROUTS_HIT: &str = "minecraft:block_nether_sprouts_hit";
    pub const BLOCK_NETHER_SPROUTS_PLACE: &str = "minecraft:block_nether_sprouts_place";
    pub const BLOCK_NETHER_SPROUTS_STEP: &str = "minecraft:block_nether_sprouts_step";
    pub const BLOCK_NETHER_WART_BREAK: &str = "minecraft:block_nether_wart_break";
    pub const BLOCK_NETHER_WOOD_BREAK: &str = "minecraft:block_nether_wood_break";
    pub const BLOCK_NETHER_WOOD_FALL: &str = "minecraft:block_nether_wood_fall";
    pub const BLOCK_NETHER_WOOD_HIT: &str = "minecraft:block_nether_wood_hit";
    pub const BLOCK_NETHER_WOOD_PLACE: &str = "minecraft:block_nether_wood_place";
    pub const BLOCK_NETHER_WOOD_STEP: &str = "minecraft:block_nether_wood_step";
    pub const BLOCK_NETHER_WOOD_BUTTON_CLICK_OFF: &str = "minecraft:block_nether_wood_button_click_off";
    pub const BLOCK_NETHER_WOOD_BUTTON_CLICK_ON: &str = "minecraft:block_nether_wood_button_click_on";
    pub const BLOCK_NETHER_WOOD_DOOR_CLOSE: &str = "minecraft:block_nether_wood_door_close";
    pub const BLOCK_NETHER_WOOD_DOOR_OPEN: &str = "minecraft:block_nether_wood_door_open";
    pub const BLOCK_NETHER_WOOD_FENCE_GATE_CLOSE: &str = "minecraft:block_nether_wood_fence_gate_close";
    pub const BLOCK_NETHER_WOOD_FENCE_GATE_OPEN: &str = "minecraft:block_nether_wood_fence_gate_open";
    pub const BLOCK_NETHER_WOOD_HANGING_SIGN_BREAK: &str = "minecraft:block_nether_wood_hanging_sign_break";
    pub const BLOCK_NETHER_WOOD_HANGING_SIGN_FALL: &str = "minecraft:block_nether_wood_hanging_sign_fall";
    pub const BLOCK_NETHER_WOOD_HANGING_SIGN_HIT: &str = "minecraft:block_nether_wood_hanging_sign_hit";
    pub const BLOCK_NETHER_WOOD_HANGING_SIGN_PLACE: &str = "minecraft:block_nether_wood_hanging_sign_place";
    pub const BLOCK_NETHER_WOOD_HANGING_SIGN_STEP: &str = "minecraft:block_nether_wood_hanging_sign_step";
    pub const BLOCK_NETHER_WOOD_PRESSURE_PLATE_CLICK_OFF: &str = "minecraft:block_nether_wood_pressure_plate_click_off";
    pub const BLOCK_NETHER_WOOD_PRESSURE_PLATE_CLICK_ON: &str = "minecraft:block_nether_wood_pressure_plate_click_on";
    pub const BLOCK_NETHER_WOOD_TRAPDOOR_CLOSE: &str = "minecraft:block_nether_wood_trapdoor_close";
    pub const BLOCK_NETHER_WOOD_TRAPDOOR_OPEN: &str = "minecraft:block_nether_wood_trapdoor_open";
    pub const BLOCK_NETHERITE_BLOCK_BREAK: &str = "minecraft:block_netherite_block_break";
    pub const BLOCK_NETHERITE_BLOCK_FALL: &str = "minecraft:block_netherite_block_fall";
    pub const BLOCK_NETHERITE_BLOCK_HIT: &str = "minecraft:block_netherite_block_hit";
    pub const BLOCK_NETHERITE_BLOCK_PLACE: &str = "minecraft:block_netherite_block_place";
    pub const BLOCK_NETHERITE_BLOCK_STEP: &str = "minecraft:block_netherite_block_step";
    pub const BLOCK_NETHERRACK_BREAK: &str = "minecraft:block_netherrack_break";
    pub const BLOCK_NETHERRACK_FALL: &str = "minecraft:block_netherrack_fall";
    pub const BLOCK_NETHERRACK_HIT: &str = "minecraft:block_netherrack_hit";
    pub const BLOCK_NETHERRACK_PLACE: &str = "minecraft:block_netherrack_place";
    pub const BLOCK_NETHERRACK_STEP: &str = "minecraft:block_netherrack_step";
    pub const BLOCK_NOTE_BLOCK_BANJO: &str = "minecraft:block_note_block_banjo";
    pub const BLOCK_NOTE_BLOCK_BASEDRUM: &str = "minecraft:block_note_block_basedrum";
    pub const BLOCK_NOTE_BLOCK_BASS: &str = "minecraft:block_note_block_bass";
    pub const BLOCK_NOTE_BLOCK_BELL: &str = "minecraft:block_note_block_bell";
    pub const BLOCK_NOTE_BLOCK_BIT: &str = "minecraft:block_note_block_bit";
    pub const BLOCK_NOTE_BLOCK_CHIME: &str = "minecraft:block_note_block_chime";
    pub const BLOCK_NOTE_BLOCK_COW_BELL: &str = "minecraft:block_note_block_cow_bell";
    pub const BLOCK_NOTE_BLOCK_DIDGERIDOO: &str = "minecraft:block_note_block_didgeridoo";
    pub const BLOCK_NOTE_BLOCK_FLUTE: &str = "minecraft:block_note_block_flute";
    pub const BLOCK_NOTE_BLOCK_GUITAR: &str = "minecraft:block_note_block_guitar";
    pub const BLOCK_NOTE_BLOCK_HARP: &str = "minecraft:block_note_block_harp";
    pub const BLOCK_NOTE_BLOCK_HAT: &str = "minecraft:block_note_block_hat";
    pub const BLOCK_NOTE_BLOCK_IMITATE_CREEPER: &str = "minecraft:block_note_block_imitate_creeper";
    pub const BLOCK_NOTE_BLOCK_IMITATE_ENDER_DRAGON: &str = "minecraft:block_note_block_imitate_ender_dragon";
    pub const BLOCK_NOTE_BLOCK_IMITATE_PIGLIN: &str = "minecraft:block_note_block_imitate_piglin";
    pub const BLOCK_NOTE_BLOCK_IMITATE_SKELETON: &str = "minecraft:block_note_block_imitate_skeleton";
    pub const BLOCK_NOTE_BLOCK_IMITATE_WITHER_SKELETON: &str = "minecraft:block_note_block_imitate_wither_skeleton";
    pub const BLOCK_NOTE_BLOCK_IMITATE_ZOMBIE: &str = "minecraft:block_note_block_imitate_zombie";
    pub const BLOCK_NOTE_BLOCK_IRON_XYLOPHONE: &str = "minecraft:block_note_block_iron_xylophone";
    pub const BLOCK_NOTE_BLOCK_PLING: &str = "minecraft:block_note_block_pling";
    pub const BLOCK_NOTE_BLOCK_SNARE: &str = "minecraft:block_note_block_snare";
    pub const BLOCK_NOTE_BLOCK_TRUMPET: &str = "minecraft:block_note_block_trumpet";
    pub const BLOCK_NOTE_BLOCK_TRUMPET_EXPOSED: &str = "minecraft:block_note_block_trumpet_exposed";
    pub const BLOCK_NOTE_BLOCK_TRUMPET_OXIDIZED: &str = "minecraft:block_note_block_trumpet_oxidized";
    pub const BLOCK_NOTE_BLOCK_TRUMPET_WEATHERED: &str = "minecraft:block_note_block_trumpet_weathered";
    pub const BLOCK_NOTE_BLOCK_XYLOPHONE: &str = "minecraft:block_note_block_xylophone";
    pub const BLOCK_NYLIUM_BREAK: &str = "minecraft:block_nylium_break";
    pub const BLOCK_NYLIUM_FALL: &str = "minecraft:block_nylium_fall";
    pub const BLOCK_NYLIUM_HIT: &str = "minecraft:block_nylium_hit";
    pub const BLOCK_NYLIUM_PLACE: &str = "minecraft:block_nylium_place";
    pub const BLOCK_NYLIUM_STEP: &str = "minecraft:block_nylium_step";
    pub const BLOCK_PACKED_MUD_BREAK: &str = "minecraft:block_packed_mud_break";
    pub const BLOCK_PACKED_MUD_FALL: &str = "minecraft:block_packed_mud_fall";
    pub const BLOCK_PACKED_MUD_HIT: &str = "minecraft:block_packed_mud_hit";
    pub const BLOCK_PACKED_MUD_PLACE: &str = "minecraft:block_packed_mud_place";
    pub const BLOCK_PACKED_MUD_STEP: &str = "minecraft:block_packed_mud_step";
    pub const BLOCK_PALE_HANGING_MOSS_IDLE: &str = "minecraft:block_pale_hanging_moss_idle";
    pub const BLOCK_PINK_PETALS_BREAK: &str = "minecraft:block_pink_petals_break";
    pub const BLOCK_PINK_PETALS_FALL: &str = "minecraft:block_pink_petals_fall";
    pub const BLOCK_PINK_PETALS_HIT: &str = "minecraft:block_pink_petals_hit";
    pub const BLOCK_PINK_PETALS_PLACE: &str = "minecraft:block_pink_petals_place";
    pub const BLOCK_PINK_PETALS_STEP: &str = "minecraft:block_pink_petals_step";
    pub const BLOCK_PISTON_CONTRACT: &str = "minecraft:block_piston_contract";
    pub const BLOCK_PISTON_EXTEND: &str = "minecraft:block_piston_extend";
    pub const BLOCK_POINTED_DRIPSTONE_BREAK: &str = "minecraft:block_pointed_dripstone_break";
    pub const BLOCK_POINTED_DRIPSTONE_DRIP_LAVA: &str = "minecraft:block_pointed_dripstone_drip_lava";
    pub const BLOCK_POINTED_DRIPSTONE_DRIP_LAVA_INTO_CAULDRON: &str = "minecraft:block_pointed_dripstone_drip_lava_into_cauldron";
    pub const BLOCK_POINTED_DRIPSTONE_DRIP_WATER: &str = "minecraft:block_pointed_dripstone_drip_water";
    pub const BLOCK_POINTED_DRIPSTONE_DRIP_WATER_INTO_CAULDRON: &str = "minecraft:block_pointed_dripstone_drip_water_into_cauldron";
    pub const BLOCK_POINTED_DRIPSTONE_FALL: &str = "minecraft:block_pointed_dripstone_fall";
    pub const BLOCK_POINTED_DRIPSTONE_HIT: &str = "minecraft:block_pointed_dripstone_hit";
    pub const BLOCK_POINTED_DRIPSTONE_LAND: &str = "minecraft:block_pointed_dripstone_land";
    pub const BLOCK_POINTED_DRIPSTONE_PLACE: &str = "minecraft:block_pointed_dripstone_place";
    pub const BLOCK_POINTED_DRIPSTONE_STEP: &str = "minecraft:block_pointed_dripstone_step";
    pub const BLOCK_POLISHED_DEEPSLATE_BREAK: &str = "minecraft:block_polished_deepslate_break";
    pub const BLOCK_POLISHED_DEEPSLATE_FALL: &str = "minecraft:block_polished_deepslate_fall";
    pub const BLOCK_POLISHED_DEEPSLATE_HIT: &str = "minecraft:block_polished_deepslate_hit";
    pub const BLOCK_POLISHED_DEEPSLATE_PLACE: &str = "minecraft:block_polished_deepslate_place";
    pub const BLOCK_POLISHED_DEEPSLATE_STEP: &str = "minecraft:block_polished_deepslate_step";
    pub const BLOCK_POLISHED_TUFF_BREAK: &str = "minecraft:block_polished_tuff_break";
    pub const BLOCK_POLISHED_TUFF_FALL: &str = "minecraft:block_polished_tuff_fall";
    pub const BLOCK_POLISHED_TUFF_HIT: &str = "minecraft:block_polished_tuff_hit";
    pub const BLOCK_POLISHED_TUFF_PLACE: &str = "minecraft:block_polished_tuff_place";
    pub const BLOCK_POLISHED_TUFF_STEP: &str = "minecraft:block_polished_tuff_step";
    pub const BLOCK_POPLAR_LEAVES_AMBIENT: &str = "minecraft:block_poplar_leaves_ambient";
    pub const BLOCK_POPLAR_LEAVES_BREAK: &str = "minecraft:block_poplar_leaves_break";
    pub const BLOCK_POPLAR_LEAVES_FALL: &str = "minecraft:block_poplar_leaves_fall";
    pub const BLOCK_POPLAR_LEAVES_HIT: &str = "minecraft:block_poplar_leaves_hit";
    pub const BLOCK_POPLAR_LEAVES_PLACE: &str = "minecraft:block_poplar_leaves_place";
    pub const BLOCK_POPLAR_LEAVES_STEP: &str = "minecraft:block_poplar_leaves_step";
    pub const BLOCK_PORTAL_AMBIENT: &str = "minecraft:block_portal_ambient";
    pub const BLOCK_PORTAL_TRAVEL: &str = "minecraft:block_portal_travel";
    pub const BLOCK_PORTAL_TRIGGER: &str = "minecraft:block_portal_trigger";
    pub const BLOCK_POTENT_SULFUR_BREAK: &str = "minecraft:block_potent_sulfur_break";
    pub const BLOCK_POTENT_SULFUR_FALL: &str = "minecraft:block_potent_sulfur_fall";
    pub const BLOCK_POTENT_SULFUR_GEYSER_CONTINUOUS_ERUPTION: &str = "minecraft:block_potent_sulfur_geyser_continuous_eruption";
    pub const BLOCK_POTENT_SULFUR_GEYSER_CONTINUOUS_ERUPTION_ACTIVE: &str = "minecraft:block_potent_sulfur_geyser_continuous_eruption_active";
    pub const BLOCK_POTENT_SULFUR_GEYSER_ERUPTION: &str = "minecraft:block_potent_sulfur_geyser_eruption";
    pub const BLOCK_POTENT_SULFUR_GEYSER_ERUPTION_ACTIVE: &str = "minecraft:block_potent_sulfur_geyser_eruption_active";
    pub const BLOCK_POTENT_SULFUR_HIT: &str = "minecraft:block_potent_sulfur_hit";
    pub const BLOCK_POTENT_SULFUR_NOXIOUS_GAS: &str = "minecraft:block_potent_sulfur_noxious_gas";
    pub const BLOCK_POTENT_SULFUR_PLACE: &str = "minecraft:block_potent_sulfur_place";
    pub const BLOCK_POTENT_SULFUR_STEP: &str = "minecraft:block_potent_sulfur_step";
    pub const BLOCK_POWDER_SNOW_BREAK: &str = "minecraft:block_powder_snow_break";
    pub const BLOCK_POWDER_SNOW_FALL: &str = "minecraft:block_powder_snow_fall";
    pub const BLOCK_POWDER_SNOW_HIT: &str = "minecraft:block_powder_snow_hit";
    pub const BLOCK_POWDER_SNOW_PLACE: &str = "minecraft:block_powder_snow_place";
    pub const BLOCK_POWDER_SNOW_STEP: &str = "minecraft:block_powder_snow_step";
    pub const BLOCK_PUMPKIN_CARVE: &str = "minecraft:block_pumpkin_carve";
    pub const BLOCK_RED_SHRUB_BREAK: &str = "minecraft:block_red_shrub_break";
    pub const BLOCK_RED_SHRUB_PLACE: &str = "minecraft:block_red_shrub_place";
    pub const BLOCK_REDSTONE_TORCH_BURNOUT: &str = "minecraft:block_redstone_torch_burnout";
    pub const BLOCK_RESIN_BREAK: &str = "minecraft:block_resin_break";
    pub const BLOCK_RESIN_FALL: &str = "minecraft:block_resin_fall";
    pub const BLOCK_RESIN_PLACE: &str = "minecraft:block_resin_place";
    pub const BLOCK_RESIN_STEP: &str = "minecraft:block_resin_step";
    pub const BLOCK_RESIN_BRICKS_BREAK: &str = "minecraft:block_resin_bricks_break";
    pub const BLOCK_RESIN_BRICKS_FALL: &str = "minecraft:block_resin_bricks_fall";
    pub const BLOCK_RESIN_BRICKS_HIT: &str = "minecraft:block_resin_bricks_hit";
    pub const BLOCK_RESIN_BRICKS_PLACE: &str = "minecraft:block_resin_bricks_place";
    pub const BLOCK_RESIN_BRICKS_STEP: &str = "minecraft:block_resin_bricks_step";
    pub const BLOCK_RESPAWN_ANCHOR_AMBIENT: &str = "minecraft:block_respawn_anchor_ambient";
    pub const BLOCK_RESPAWN_ANCHOR_CHARGE: &str = "minecraft:block_respawn_anchor_charge";
    pub const BLOCK_RESPAWN_ANCHOR_DEPLETE: &str = "minecraft:block_respawn_anchor_deplete";
    pub const BLOCK_RESPAWN_ANCHOR_SET_SPAWN: &str = "minecraft:block_respawn_anchor_set_spawn";
    pub const BLOCK_ROOTED_DIRT_BREAK: &str = "minecraft:block_rooted_dirt_break";
    pub const BLOCK_ROOTED_DIRT_FALL: &str = "minecraft:block_rooted_dirt_fall";
    pub const BLOCK_ROOTED_DIRT_HIT: &str = "minecraft:block_rooted_dirt_hit";
    pub const BLOCK_ROOTED_DIRT_PLACE: &str = "minecraft:block_rooted_dirt_place";
    pub const BLOCK_ROOTED_DIRT_STEP: &str = "minecraft:block_rooted_dirt_step";
    pub const BLOCK_ROOTS_BREAK: &str = "minecraft:block_roots_break";
    pub const BLOCK_ROOTS_FALL: &str = "minecraft:block_roots_fall";
    pub const BLOCK_ROOTS_HIT: &str = "minecraft:block_roots_hit";
    pub const BLOCK_ROOTS_PLACE: &str = "minecraft:block_roots_place";
    pub const BLOCK_ROOTS_STEP: &str = "minecraft:block_roots_step";
    pub const BLOCK_SAND_BREAK: &str = "minecraft:block_sand_break";
    pub const BLOCK_SAND_FALL: &str = "minecraft:block_sand_fall";
    pub const BLOCK_SAND_HIT: &str = "minecraft:block_sand_hit";
    pub const BLOCK_SAND_IDLE: &str = "minecraft:block_sand_idle";
    pub const BLOCK_SAND_PLACE: &str = "minecraft:block_sand_place";
    pub const BLOCK_SAND_STEP: &str = "minecraft:block_sand_step";
    pub const BLOCK_SCAFFOLDING_BREAK: &str = "minecraft:block_scaffolding_break";
    pub const BLOCK_SCAFFOLDING_FALL: &str = "minecraft:block_scaffolding_fall";
    pub const BLOCK_SCAFFOLDING_HIT: &str = "minecraft:block_scaffolding_hit";
    pub const BLOCK_SCAFFOLDING_PLACE: &str = "minecraft:block_scaffolding_place";
    pub const BLOCK_SCAFFOLDING_STEP: &str = "minecraft:block_scaffolding_step";
    pub const BLOCK_SCULK_BREAK: &str = "minecraft:block_sculk_break";
    pub const BLOCK_SCULK_CHARGE: &str = "minecraft:block_sculk_charge";
    pub const BLOCK_SCULK_FALL: &str = "minecraft:block_sculk_fall";
    pub const BLOCK_SCULK_HIT: &str = "minecraft:block_sculk_hit";
    pub const BLOCK_SCULK_PLACE: &str = "minecraft:block_sculk_place";
    pub const BLOCK_SCULK_SPREAD: &str = "minecraft:block_sculk_spread";
    pub const BLOCK_SCULK_STEP: &str = "minecraft:block_sculk_step";
    pub const BLOCK_SCULK_CATALYST_BLOOM: &str = "minecraft:block_sculk_catalyst_bloom";
    pub const BLOCK_SCULK_CATALYST_BREAK: &str = "minecraft:block_sculk_catalyst_break";
    pub const BLOCK_SCULK_CATALYST_FALL: &str = "minecraft:block_sculk_catalyst_fall";
    pub const BLOCK_SCULK_CATALYST_HIT: &str = "minecraft:block_sculk_catalyst_hit";
    pub const BLOCK_SCULK_CATALYST_PLACE: &str = "minecraft:block_sculk_catalyst_place";
    pub const BLOCK_SCULK_CATALYST_STEP: &str = "minecraft:block_sculk_catalyst_step";
    pub const BLOCK_SCULK_SENSOR_BREAK: &str = "minecraft:block_sculk_sensor_break";
    pub const BLOCK_SCULK_SENSOR_CLICKING: &str = "minecraft:block_sculk_sensor_clicking";
    pub const BLOCK_SCULK_SENSOR_CLICKING_STOP: &str = "minecraft:block_sculk_sensor_clicking_stop";
    pub const BLOCK_SCULK_SENSOR_FALL: &str = "minecraft:block_sculk_sensor_fall";
    pub const BLOCK_SCULK_SENSOR_HIT: &str = "minecraft:block_sculk_sensor_hit";
    pub const BLOCK_SCULK_SENSOR_PLACE: &str = "minecraft:block_sculk_sensor_place";
    pub const BLOCK_SCULK_SENSOR_STEP: &str = "minecraft:block_sculk_sensor_step";
    pub const BLOCK_SCULK_SHRIEKER_BREAK: &str = "minecraft:block_sculk_shrieker_break";
    pub const BLOCK_SCULK_SHRIEKER_FALL: &str = "minecraft:block_sculk_shrieker_fall";
    pub const BLOCK_SCULK_SHRIEKER_HIT: &str = "minecraft:block_sculk_shrieker_hit";
    pub const BLOCK_SCULK_SHRIEKER_PLACE: &str = "minecraft:block_sculk_shrieker_place";
    pub const BLOCK_SCULK_SHRIEKER_SHRIEK: &str = "minecraft:block_sculk_shrieker_shriek";
    pub const BLOCK_SCULK_SHRIEKER_STEP: &str = "minecraft:block_sculk_shrieker_step";
    pub const BLOCK_SCULK_VEIN_BREAK: &str = "minecraft:block_sculk_vein_break";
    pub const BLOCK_SCULK_VEIN_FALL: &str = "minecraft:block_sculk_vein_fall";
    pub const BLOCK_SCULK_VEIN_HIT: &str = "minecraft:block_sculk_vein_hit";
    pub const BLOCK_SCULK_VEIN_PLACE: &str = "minecraft:block_sculk_vein_place";
    pub const BLOCK_SCULK_VEIN_STEP: &str = "minecraft:block_sculk_vein_step";
    pub const BLOCK_SHELF_ACTIVATE: &str = "minecraft:block_shelf_activate";
    pub const BLOCK_SHELF_BREAK: &str = "minecraft:block_shelf_break";
    pub const BLOCK_SHELF_DEACTIVATE: &str = "minecraft:block_shelf_deactivate";
    pub const BLOCK_SHELF_FALL: &str = "minecraft:block_shelf_fall";
    pub const BLOCK_SHELF_HIT: &str = "minecraft:block_shelf_hit";
    pub const BLOCK_SHELF_MULTI_SWAP: &str = "minecraft:block_shelf_multi_swap";
    pub const BLOCK_SHELF_PLACE: &str = "minecraft:block_shelf_place";
    pub const BLOCK_SHELF_PLACE_ITEM: &str = "minecraft:block_shelf_place_item";
    pub const BLOCK_SHELF_SINGLE_SWAP: &str = "minecraft:block_shelf_single_swap";
    pub const BLOCK_SHELF_STEP: &str = "minecraft:block_shelf_step";
    pub const BLOCK_SHELF_TAKE_ITEM: &str = "minecraft:block_shelf_take_item";
    pub const BLOCK_SHELF_MUSHROOM_BOUNCE: &str = "minecraft:block_shelf_mushroom_bounce";
    pub const BLOCK_SHELF_MUSHROOM_BREAK: &str = "minecraft:block_shelf_mushroom_break";
    pub const BLOCK_SHELF_MUSHROOM_FALL: &str = "minecraft:block_shelf_mushroom_fall";
    pub const BLOCK_SHELF_MUSHROOM_PLACE: &str = "minecraft:block_shelf_mushroom_place";
    pub const BLOCK_SHELF_MUSHROOM_STEP: &str = "minecraft:block_shelf_mushroom_step";
    pub const BLOCK_SHROOMLIGHT_BREAK: &str = "minecraft:block_shroomlight_break";
    pub const BLOCK_SHROOMLIGHT_FALL: &str = "minecraft:block_shroomlight_fall";
    pub const BLOCK_SHROOMLIGHT_HIT: &str = "minecraft:block_shroomlight_hit";
    pub const BLOCK_SHROOMLIGHT_PLACE: &str = "minecraft:block_shroomlight_place";
    pub const BLOCK_SHROOMLIGHT_STEP: &str = "minecraft:block_shroomlight_step";
    pub const BLOCK_SHULKER_BOX_CLOSE: &str = "minecraft:block_shulker_box_close";
    pub const BLOCK_SHULKER_BOX_OPEN: &str = "minecraft:block_shulker_box_open";
    pub const BLOCK_SIGN_WAXED_INTERACT_FAIL: &str = "minecraft:block_sign_waxed_interact_fail";
    pub const BLOCK_SLIME_BLOCK_BREAK: &str = "minecraft:block_slime_block_break";
    pub const BLOCK_SLIME_BLOCK_FALL: &str = "minecraft:block_slime_block_fall";
    pub const BLOCK_SLIME_BLOCK_HIT: &str = "minecraft:block_slime_block_hit";
    pub const BLOCK_SLIME_BLOCK_PLACE: &str = "minecraft:block_slime_block_place";
    pub const BLOCK_SLIME_BLOCK_STEP: &str = "minecraft:block_slime_block_step";
    pub const BLOCK_SMALL_AMETHYST_BUD_BREAK: &str = "minecraft:block_small_amethyst_bud_break";
    pub const BLOCK_SMALL_AMETHYST_BUD_PLACE: &str = "minecraft:block_small_amethyst_bud_place";
    pub const BLOCK_SMALL_DRIPLEAF_BREAK: &str = "minecraft:block_small_dripleaf_break";
    pub const BLOCK_SMALL_DRIPLEAF_FALL: &str = "minecraft:block_small_dripleaf_fall";
    pub const BLOCK_SMALL_DRIPLEAF_HIT: &str = "minecraft:block_small_dripleaf_hit";
    pub const BLOCK_SMALL_DRIPLEAF_PLACE: &str = "minecraft:block_small_dripleaf_place";
    pub const BLOCK_SMALL_DRIPLEAF_STEP: &str = "minecraft:block_small_dripleaf_step";
    pub const BLOCK_SMITHING_TABLE_USE: &str = "minecraft:block_smithing_table_use";
    pub const BLOCK_SMOKER_SMOKE: &str = "minecraft:block_smoker_smoke";
    pub const BLOCK_SNIFFER_EGG_CRACK: &str = "minecraft:block_sniffer_egg_crack";
    pub const BLOCK_SNIFFER_EGG_HATCH: &str = "minecraft:block_sniffer_egg_hatch";
    pub const BLOCK_SNIFFER_EGG_PLOP: &str = "minecraft:block_sniffer_egg_plop";
    pub const BLOCK_SNOW_BREAK: &str = "minecraft:block_snow_break";
    pub const BLOCK_SNOW_FALL: &str = "minecraft:block_snow_fall";
    pub const BLOCK_SNOW_HIT: &str = "minecraft:block_snow_hit";
    pub const BLOCK_SNOW_PLACE: &str = "minecraft:block_snow_place";
    pub const BLOCK_SNOW_STEP: &str = "minecraft:block_snow_step";
    pub const BLOCK_SOUL_SAND_BREAK: &str = "minecraft:block_soul_sand_break";
    pub const BLOCK_SOUL_SAND_FALL: &str = "minecraft:block_soul_sand_fall";
    pub const BLOCK_SOUL_SAND_HIT: &str = "minecraft:block_soul_sand_hit";
    pub const BLOCK_SOUL_SAND_PLACE: &str = "minecraft:block_soul_sand_place";
    pub const BLOCK_SOUL_SAND_STEP: &str = "minecraft:block_soul_sand_step";
    pub const BLOCK_SOUL_SOIL_BREAK: &str = "minecraft:block_soul_soil_break";
    pub const BLOCK_SOUL_SOIL_FALL: &str = "minecraft:block_soul_soil_fall";
    pub const BLOCK_SOUL_SOIL_HIT: &str = "minecraft:block_soul_soil_hit";
    pub const BLOCK_SOUL_SOIL_PLACE: &str = "minecraft:block_soul_soil_place";
    pub const BLOCK_SOUL_SOIL_STEP: &str = "minecraft:block_soul_soil_step";
    pub const BLOCK_SPAWNER_BREAK: &str = "minecraft:block_spawner_break";
    pub const BLOCK_SPAWNER_FALL: &str = "minecraft:block_spawner_fall";
    pub const BLOCK_SPAWNER_HIT: &str = "minecraft:block_spawner_hit";
    pub const BLOCK_SPAWNER_PLACE: &str = "minecraft:block_spawner_place";
    pub const BLOCK_SPAWNER_STEP: &str = "minecraft:block_spawner_step";
    pub const BLOCK_SPONGE_ABSORB: &str = "minecraft:block_sponge_absorb";
    pub const BLOCK_SPONGE_BREAK: &str = "minecraft:block_sponge_break";
    pub const BLOCK_SPONGE_FALL: &str = "minecraft:block_sponge_fall";
    pub const BLOCK_SPONGE_HIT: &str = "minecraft:block_sponge_hit";
    pub const BLOCK_SPONGE_PLACE: &str = "minecraft:block_sponge_place";
    pub const BLOCK_SPONGE_STEP: &str = "minecraft:block_sponge_step";
    pub const BLOCK_SPORE_BLOSSOM_BREAK: &str = "minecraft:block_spore_blossom_break";
    pub const BLOCK_SPORE_BLOSSOM_FALL: &str = "minecraft:block_spore_blossom_fall";
    pub const BLOCK_SPORE_BLOSSOM_HIT: &str = "minecraft:block_spore_blossom_hit";
    pub const BLOCK_SPORE_BLOSSOM_PLACE: &str = "minecraft:block_spore_blossom_place";
    pub const BLOCK_SPORE_BLOSSOM_STEP: &str = "minecraft:block_spore_blossom_step";
    pub const BLOCK_STEM_BREAK: &str = "minecraft:block_stem_break";
    pub const BLOCK_STEM_FALL: &str = "minecraft:block_stem_fall";
    pub const BLOCK_STEM_HIT: &str = "minecraft:block_stem_hit";
    pub const BLOCK_STEM_PLACE: &str = "minecraft:block_stem_place";
    pub const BLOCK_STEM_STEP: &str = "minecraft:block_stem_step";
    pub const BLOCK_STONE_BREAK: &str = "minecraft:block_stone_break";
    pub const BLOCK_STONE_FALL: &str = "minecraft:block_stone_fall";
    pub const BLOCK_STONE_HIT: &str = "minecraft:block_stone_hit";
    pub const BLOCK_STONE_PLACE: &str = "minecraft:block_stone_place";
    pub const BLOCK_STONE_STEP: &str = "minecraft:block_stone_step";
    pub const BLOCK_STONE_BUTTON_CLICK_OFF: &str = "minecraft:block_stone_button_click_off";
    pub const BLOCK_STONE_BUTTON_CLICK_ON: &str = "minecraft:block_stone_button_click_on";
    pub const BLOCK_STONE_PRESSURE_PLATE_CLICK_OFF: &str = "minecraft:block_stone_pressure_plate_click_off";
    pub const BLOCK_STONE_PRESSURE_PLATE_CLICK_ON: &str = "minecraft:block_stone_pressure_plate_click_on";
    pub const BLOCK_STRAW_BED_BREAK: &str = "minecraft:block_straw_bed_break";
    pub const BLOCK_STRAW_BED_BREAK_LEAVE: &str = "minecraft:block_straw_bed_break_leave";
    pub const BLOCK_STRAW_BED_FALL: &str = "minecraft:block_straw_bed_fall";
    pub const BLOCK_STRAW_BED_HIT: &str = "minecraft:block_straw_bed_hit";
    pub const BLOCK_STRAW_BED_PLACE: &str = "minecraft:block_straw_bed_place";
    pub const BLOCK_STRAW_BED_STEP: &str = "minecraft:block_straw_bed_step";
    pub const BLOCK_SULFUR_BREAK: &str = "minecraft:block_sulfur_break";
    pub const BLOCK_SULFUR_FALL: &str = "minecraft:block_sulfur_fall";
    pub const BLOCK_SULFUR_HIT: &str = "minecraft:block_sulfur_hit";
    pub const BLOCK_SULFUR_PLACE: &str = "minecraft:block_sulfur_place";
    pub const BLOCK_SULFUR_STEP: &str = "minecraft:block_sulfur_step";
    pub const BLOCK_SULFUR_SPIKE_BREAK: &str = "minecraft:block_sulfur_spike_break";
    pub const BLOCK_SULFUR_SPIKE_FALL: &str = "minecraft:block_sulfur_spike_fall";
    pub const BLOCK_SULFUR_SPIKE_HIT: &str = "minecraft:block_sulfur_spike_hit";
    pub const BLOCK_SULFUR_SPIKE_LAND: &str = "minecraft:block_sulfur_spike_land";
    pub const BLOCK_SULFUR_SPIKE_PLACE: &str = "minecraft:block_sulfur_spike_place";
    pub const BLOCK_SULFUR_SPIKE_STEP: &str = "minecraft:block_sulfur_spike_step";
    pub const BLOCK_SUSPICIOUS_GRAVEL_BREAK: &str = "minecraft:block_suspicious_gravel_break";
    pub const BLOCK_SUSPICIOUS_GRAVEL_FALL: &str = "minecraft:block_suspicious_gravel_fall";
    pub const BLOCK_SUSPICIOUS_GRAVEL_HIT: &str = "minecraft:block_suspicious_gravel_hit";
    pub const BLOCK_SUSPICIOUS_GRAVEL_PLACE: &str = "minecraft:block_suspicious_gravel_place";
    pub const BLOCK_SUSPICIOUS_GRAVEL_STEP: &str = "minecraft:block_suspicious_gravel_step";
    pub const BLOCK_SUSPICIOUS_SAND_BREAK: &str = "minecraft:block_suspicious_sand_break";
    pub const BLOCK_SUSPICIOUS_SAND_FALL: &str = "minecraft:block_suspicious_sand_fall";
    pub const BLOCK_SUSPICIOUS_SAND_HIT: &str = "minecraft:block_suspicious_sand_hit";
    pub const BLOCK_SUSPICIOUS_SAND_PLACE: &str = "minecraft:block_suspicious_sand_place";
    pub const BLOCK_SUSPICIOUS_SAND_STEP: &str = "minecraft:block_suspicious_sand_step";
    pub const BLOCK_SWEET_BERRY_BUSH_BREAK: &str = "minecraft:block_sweet_berry_bush_break";
    pub const BLOCK_SWEET_BERRY_BUSH_PICK_BERRIES: &str = "minecraft:block_sweet_berry_bush_pick_berries";
    pub const BLOCK_SWEET_BERRY_BUSH_PLACE: &str = "minecraft:block_sweet_berry_bush_place";
    pub const BLOCK_TRIAL_SPAWNER_ABOUT_TO_SPAWN_ITEM: &str = "minecraft:block_trial_spawner_about_to_spawn_item";
    pub const BLOCK_TRIAL_SPAWNER_AMBIENT: &str = "minecraft:block_trial_spawner_ambient";
    pub const BLOCK_TRIAL_SPAWNER_AMBIENT_OMINOUS: &str = "minecraft:block_trial_spawner_ambient_ominous";
    pub const BLOCK_TRIAL_SPAWNER_BREAK: &str = "minecraft:block_trial_spawner_break";
    pub const BLOCK_TRIAL_SPAWNER_CLOSE_SHUTTER: &str = "minecraft:block_trial_spawner_close_shutter";
    pub const BLOCK_TRIAL_SPAWNER_DETECT_PLAYER: &str = "minecraft:block_trial_spawner_detect_player";
    pub const BLOCK_TRIAL_SPAWNER_EJECT_ITEM: &str = "minecraft:block_trial_spawner_eject_item";
    pub const BLOCK_TRIAL_SPAWNER_FALL: &str = "minecraft:block_trial_spawner_fall";
    pub const BLOCK_TRIAL_SPAWNER_HIT: &str = "minecraft:block_trial_spawner_hit";
    pub const BLOCK_TRIAL_SPAWNER_OMINOUS_ACTIVATE: &str = "minecraft:block_trial_spawner_ominous_activate";
    pub const BLOCK_TRIAL_SPAWNER_OPEN_SHUTTER: &str = "minecraft:block_trial_spawner_open_shutter";
    pub const BLOCK_TRIAL_SPAWNER_PLACE: &str = "minecraft:block_trial_spawner_place";
    pub const BLOCK_TRIAL_SPAWNER_SPAWN_ITEM: &str = "minecraft:block_trial_spawner_spawn_item";
    pub const BLOCK_TRIAL_SPAWNER_SPAWN_ITEM_BEGIN: &str = "minecraft:block_trial_spawner_spawn_item_begin";
    pub const BLOCK_TRIAL_SPAWNER_SPAWN_MOB: &str = "minecraft:block_trial_spawner_spawn_mob";
    pub const BLOCK_TRIAL_SPAWNER_STEP: &str = "minecraft:block_trial_spawner_step";
    pub const BLOCK_TRIPWIRE_ATTACH: &str = "minecraft:block_tripwire_attach";
    pub const BLOCK_TRIPWIRE_CLICK_OFF: &str = "minecraft:block_tripwire_click_off";
    pub const BLOCK_TRIPWIRE_CLICK_ON: &str = "minecraft:block_tripwire_click_on";
    pub const BLOCK_TRIPWIRE_DETACH: &str = "minecraft:block_tripwire_detach";
    pub const BLOCK_TUFF_BREAK: &str = "minecraft:block_tuff_break";
    pub const BLOCK_TUFF_FALL: &str = "minecraft:block_tuff_fall";
    pub const BLOCK_TUFF_HIT: &str = "minecraft:block_tuff_hit";
    pub const BLOCK_TUFF_PLACE: &str = "minecraft:block_tuff_place";
    pub const BLOCK_TUFF_STEP: &str = "minecraft:block_tuff_step";
    pub const BLOCK_TUFF_BRICKS_BREAK: &str = "minecraft:block_tuff_bricks_break";
    pub const BLOCK_TUFF_BRICKS_FALL: &str = "minecraft:block_tuff_bricks_fall";
    pub const BLOCK_TUFF_BRICKS_HIT: &str = "minecraft:block_tuff_bricks_hit";
    pub const BLOCK_TUFF_BRICKS_PLACE: &str = "minecraft:block_tuff_bricks_place";
    pub const BLOCK_TUFF_BRICKS_STEP: &str = "minecraft:block_tuff_bricks_step";
    pub const BLOCK_VAULT_ACTIVATE: &str = "minecraft:block_vault_activate";
    pub const BLOCK_VAULT_AMBIENT: &str = "minecraft:block_vault_ambient";
    pub const BLOCK_VAULT_BREAK: &str = "minecraft:block_vault_break";
    pub const BLOCK_VAULT_CLOSE_SHUTTER: &str = "minecraft:block_vault_close_shutter";
    pub const BLOCK_VAULT_DEACTIVATE: &str = "minecraft:block_vault_deactivate";
    pub const BLOCK_VAULT_EJECT_ITEM: &str = "minecraft:block_vault_eject_item";
    pub const BLOCK_VAULT_FALL: &str = "minecraft:block_vault_fall";
    pub const BLOCK_VAULT_HIT: &str = "minecraft:block_vault_hit";
    pub const BLOCK_VAULT_INSERT_ITEM: &str = "minecraft:block_vault_insert_item";
    pub const BLOCK_VAULT_INSERT_ITEM_FAIL: &str = "minecraft:block_vault_insert_item_fail";
    pub const BLOCK_VAULT_OPEN_SHUTTER: &str = "minecraft:block_vault_open_shutter";
    pub const BLOCK_VAULT_PLACE: &str = "minecraft:block_vault_place";
    pub const BLOCK_VAULT_REJECT_REWARDED_PLAYER: &str = "minecraft:block_vault_reject_rewarded_player";
    pub const BLOCK_VAULT_STEP: &str = "minecraft:block_vault_step";
    pub const BLOCK_VINE_BREAK: &str = "minecraft:block_vine_break";
    pub const BLOCK_VINE_FALL: &str = "minecraft:block_vine_fall";
    pub const BLOCK_VINE_HIT: &str = "minecraft:block_vine_hit";
    pub const BLOCK_VINE_PLACE: &str = "minecraft:block_vine_place";
    pub const BLOCK_VINE_STEP: &str = "minecraft:block_vine_step";
    pub const BLOCK_WART_BLOCK_BREAK: &str = "minecraft:block_wart_block_break";
    pub const BLOCK_WART_BLOCK_FALL: &str = "minecraft:block_wart_block_fall";
    pub const BLOCK_WART_BLOCK_HIT: &str = "minecraft:block_wart_block_hit";
    pub const BLOCK_WART_BLOCK_PLACE: &str = "minecraft:block_wart_block_place";
    pub const BLOCK_WART_BLOCK_STEP: &str = "minecraft:block_wart_block_step";
    pub const BLOCK_WATER_AMBIENT: &str = "minecraft:block_water_ambient";
    pub const BLOCK_WEEPING_VINES_BREAK: &str = "minecraft:block_weeping_vines_break";
    pub const BLOCK_WEEPING_VINES_FALL: &str = "minecraft:block_weeping_vines_fall";
    pub const BLOCK_WEEPING_VINES_HIT: &str = "minecraft:block_weeping_vines_hit";
    pub const BLOCK_WEEPING_VINES_PLACE: &str = "minecraft:block_weeping_vines_place";
    pub const BLOCK_WEEPING_VINES_STEP: &str = "minecraft:block_weeping_vines_step";
    pub const BLOCK_WET_GRASS_BREAK: &str = "minecraft:block_wet_grass_break";
    pub const BLOCK_WET_GRASS_FALL: &str = "minecraft:block_wet_grass_fall";
    pub const BLOCK_WET_GRASS_HIT: &str = "minecraft:block_wet_grass_hit";
    pub const BLOCK_WET_GRASS_PLACE: &str = "minecraft:block_wet_grass_place";
    pub const BLOCK_WET_GRASS_STEP: &str = "minecraft:block_wet_grass_step";
    pub const BLOCK_WET_SPONGE_BREAK: &str = "minecraft:block_wet_sponge_break";
    pub const BLOCK_WET_SPONGE_DRIES: &str = "minecraft:block_wet_sponge_dries";
    pub const BLOCK_WET_SPONGE_FALL: &str = "minecraft:block_wet_sponge_fall";
    pub const BLOCK_WET_SPONGE_HIT: &str = "minecraft:block_wet_sponge_hit";
    pub const BLOCK_WET_SPONGE_PLACE: &str = "minecraft:block_wet_sponge_place";
    pub const BLOCK_WET_SPONGE_STEP: &str = "minecraft:block_wet_sponge_step";
    pub const BLOCK_WOOD_BREAK: &str = "minecraft:block_wood_break";
    pub const BLOCK_WOOD_FALL: &str = "minecraft:block_wood_fall";
    pub const BLOCK_WOOD_HIT: &str = "minecraft:block_wood_hit";
    pub const BLOCK_WOOD_PLACE: &str = "minecraft:block_wood_place";
    pub const BLOCK_WOOD_STEP: &str = "minecraft:block_wood_step";
    pub const BLOCK_WOODEN_BUTTON_CLICK_OFF: &str = "minecraft:block_wooden_button_click_off";
    pub const BLOCK_WOODEN_BUTTON_CLICK_ON: &str = "minecraft:block_wooden_button_click_on";
    pub const BLOCK_WOODEN_DOOR_CLOSE: &str = "minecraft:block_wooden_door_close";
    pub const BLOCK_WOODEN_DOOR_OPEN: &str = "minecraft:block_wooden_door_open";
    pub const BLOCK_WOODEN_PRESSURE_PLATE_CLICK_OFF: &str = "minecraft:block_wooden_pressure_plate_click_off";
    pub const BLOCK_WOODEN_PRESSURE_PLATE_CLICK_ON: &str = "minecraft:block_wooden_pressure_plate_click_on";
    pub const BLOCK_WOODEN_TRAPDOOR_CLOSE: &str = "minecraft:block_wooden_trapdoor_close";
    pub const BLOCK_WOODEN_TRAPDOOR_OPEN: &str = "minecraft:block_wooden_trapdoor_open";
    pub const BLOCK_WOOL_BREAK: &str = "minecraft:block_wool_break";
    pub const BLOCK_WOOL_FALL: &str = "minecraft:block_wool_fall";
    pub const BLOCK_WOOL_HIT: &str = "minecraft:block_wool_hit";
    pub const BLOCK_WOOL_PLACE: &str = "minecraft:block_wool_place";
    pub const BLOCK_WOOL_STEP: &str = "minecraft:block_wool_step";
    pub const ENCHANT_THORNS_HIT: &str = "minecraft:enchant_thorns_hit";
    pub const ENTITY_ALLAY_AMBIENT_WITH_ITEM: &str = "minecraft:entity_allay_ambient_with_item";
    pub const ENTITY_ALLAY_AMBIENT_WITHOUT_ITEM: &str = "minecraft:entity_allay_ambient_without_item";
    pub const ENTITY_ALLAY_DEATH: &str = "minecraft:entity_allay_death";
    pub const ENTITY_ALLAY_HURT: &str = "minecraft:entity_allay_hurt";
    pub const ENTITY_ALLAY_ITEM_GIVEN: &str = "minecraft:entity_allay_item_given";
    pub const ENTITY_ALLAY_ITEM_TAKEN: &str = "minecraft:entity_allay_item_taken";
    pub const ENTITY_ALLAY_ITEM_THROWN: &str = "minecraft:entity_allay_item_thrown";
    pub const ENTITY_ARMADILLO_AMBIENT: &str = "minecraft:entity_armadillo_ambient";
    pub const ENTITY_ARMADILLO_BRUSH: &str = "minecraft:entity_armadillo_brush";
    pub const ENTITY_ARMADILLO_DEATH: &str = "minecraft:entity_armadillo_death";
    pub const ENTITY_ARMADILLO_EAT: &str = "minecraft:entity_armadillo_eat";
    pub const ENTITY_ARMADILLO_HURT: &str = "minecraft:entity_armadillo_hurt";
    pub const ENTITY_ARMADILLO_HURT_REDUCED: &str = "minecraft:entity_armadillo_hurt_reduced";
    pub const ENTITY_ARMADILLO_LAND: &str = "minecraft:entity_armadillo_land";
    pub const ENTITY_ARMADILLO_PEEK: &str = "minecraft:entity_armadillo_peek";
    pub const ENTITY_ARMADILLO_ROLL: &str = "minecraft:entity_armadillo_roll";
    pub const ENTITY_ARMADILLO_SCUTE_DROP: &str = "minecraft:entity_armadillo_scute_drop";
    pub const ENTITY_ARMADILLO_STEP: &str = "minecraft:entity_armadillo_step";
    pub const ENTITY_ARMADILLO_UNROLL_FINISH: &str = "minecraft:entity_armadillo_unroll_finish";
    pub const ENTITY_ARMADILLO_UNROLL_START: &str = "minecraft:entity_armadillo_unroll_start";
    pub const ENTITY_ARMOR_STAND_BREAK: &str = "minecraft:entity_armor_stand_break";
    pub const ENTITY_ARMOR_STAND_FALL: &str = "minecraft:entity_armor_stand_fall";
    pub const ENTITY_ARMOR_STAND_HIT: &str = "minecraft:entity_armor_stand_hit";
    pub const ENTITY_ARMOR_STAND_PLACE: &str = "minecraft:entity_armor_stand_place";
    pub const ENTITY_ARROW_HIT: &str = "minecraft:entity_arrow_hit";
    pub const ENTITY_ARROW_HIT_PLAYER: &str = "minecraft:entity_arrow_hit_player";
    pub const ENTITY_ARROW_SHOOT: &str = "minecraft:entity_arrow_shoot";
    pub const ENTITY_AXOLOTL_ATTACK: &str = "minecraft:entity_axolotl_attack";
    pub const ENTITY_AXOLOTL_DEATH: &str = "minecraft:entity_axolotl_death";
    pub const ENTITY_AXOLOTL_HURT: &str = "minecraft:entity_axolotl_hurt";
    pub const ENTITY_AXOLOTL_IDLE_AIR: &str = "minecraft:entity_axolotl_idle_air";
    pub const ENTITY_AXOLOTL_IDLE_WATER: &str = "minecraft:entity_axolotl_idle_water";
    pub const ENTITY_AXOLOTL_SPLASH: &str = "minecraft:entity_axolotl_splash";
    pub const ENTITY_AXOLOTL_SWIM: &str = "minecraft:entity_axolotl_swim";
    pub const ENTITY_BABY_CAT_AMBIENT: &str = "minecraft:entity_baby_cat_ambient";
    pub const ENTITY_BABY_CAT_BEG_FOR_FOOD: &str = "minecraft:entity_baby_cat_beg_for_food";
    pub const ENTITY_BABY_CAT_DEATH: &str = "minecraft:entity_baby_cat_death";
    pub const ENTITY_BABY_CAT_EAT: &str = "minecraft:entity_baby_cat_eat";
    pub const ENTITY_BABY_CAT_HISS: &str = "minecraft:entity_baby_cat_hiss";
    pub const ENTITY_BABY_CAT_HURT: &str = "minecraft:entity_baby_cat_hurt";
    pub const ENTITY_BABY_CAT_PURR: &str = "minecraft:entity_baby_cat_purr";
    pub const ENTITY_BABY_CAT_PURREOW: &str = "minecraft:entity_baby_cat_purreow";
    pub const ENTITY_BABY_CAT_STRAY_AMBIENT: &str = "minecraft:entity_baby_cat_stray_ambient";
    pub const ENTITY_BABY_CHICKEN_AMBIENT: &str = "minecraft:entity_baby_chicken_ambient";
    pub const ENTITY_BABY_CHICKEN_DEATH: &str = "minecraft:entity_baby_chicken_death";
    pub const ENTITY_BABY_CHICKEN_HURT: &str = "minecraft:entity_baby_chicken_hurt";
    pub const ENTITY_BABY_CHICKEN_STEP: &str = "minecraft:entity_baby_chicken_step";
    pub const ENTITY_BABY_HORSE_AMBIENT: &str = "minecraft:entity_baby_horse_ambient";
    pub const ENTITY_BABY_HORSE_ANGRY: &str = "minecraft:entity_baby_horse_angry";
    pub const ENTITY_BABY_HORSE_BREATHE: &str = "minecraft:entity_baby_horse_breathe";
    pub const ENTITY_BABY_HORSE_DEATH: &str = "minecraft:entity_baby_horse_death";
    pub const ENTITY_BABY_HORSE_EAT: &str = "minecraft:entity_baby_horse_eat";
    pub const ENTITY_BABY_HORSE_HURT: &str = "minecraft:entity_baby_horse_hurt";
    pub const ENTITY_BABY_HORSE_LAND: &str = "minecraft:entity_baby_horse_land";
    pub const ENTITY_BABY_HORSE_STEP: &str = "minecraft:entity_baby_horse_step";
    pub const ENTITY_BABY_NAUTILUS_AMBIENT: &str = "minecraft:entity_baby_nautilus_ambient";
    pub const ENTITY_BABY_NAUTILUS_AMBIENT_LAND: &str = "minecraft:entity_baby_nautilus_ambient_land";
    pub const ENTITY_BABY_NAUTILUS_DEATH: &str = "minecraft:entity_baby_nautilus_death";
    pub const ENTITY_BABY_NAUTILUS_DEATH_LAND: &str = "minecraft:entity_baby_nautilus_death_land";
    pub const ENTITY_BABY_NAUTILUS_EAT: &str = "minecraft:entity_baby_nautilus_eat";
    pub const ENTITY_BABY_NAUTILUS_HURT: &str = "minecraft:entity_baby_nautilus_hurt";
    pub const ENTITY_BABY_NAUTILUS_HURT_LAND: &str = "minecraft:entity_baby_nautilus_hurt_land";
    pub const ENTITY_BABY_NAUTILUS_SWIM: &str = "minecraft:entity_baby_nautilus_swim";
    pub const ENTITY_BABY_PIG_AMBIENT: &str = "minecraft:entity_baby_pig_ambient";
    pub const ENTITY_BABY_PIG_DEATH: &str = "minecraft:entity_baby_pig_death";
    pub const ENTITY_BABY_PIG_EAT: &str = "minecraft:entity_baby_pig_eat";
    pub const ENTITY_BABY_PIG_HURT: &str = "minecraft:entity_baby_pig_hurt";
    pub const ENTITY_BABY_PIG_STEP: &str = "minecraft:entity_baby_pig_step";
    pub const ENTITY_BABY_WOLF_AMBIENT: &str = "minecraft:entity_baby_wolf_ambient";
    pub const ENTITY_BABY_WOLF_DEATH: &str = "minecraft:entity_baby_wolf_death";
    pub const ENTITY_BABY_WOLF_GROWL: &str = "minecraft:entity_baby_wolf_growl";
    pub const ENTITY_BABY_WOLF_HURT: &str = "minecraft:entity_baby_wolf_hurt";
    pub const ENTITY_BABY_WOLF_PANT: &str = "minecraft:entity_baby_wolf_pant";
    pub const ENTITY_BABY_WOLF_STEP: &str = "minecraft:entity_baby_wolf_step";
    pub const ENTITY_BABY_WOLF_WHINE: &str = "minecraft:entity_baby_wolf_whine";
    pub const ENTITY_BAT_AMBIENT: &str = "minecraft:entity_bat_ambient";
    pub const ENTITY_BAT_DEATH: &str = "minecraft:entity_bat_death";
    pub const ENTITY_BAT_HURT: &str = "minecraft:entity_bat_hurt";
    pub const ENTITY_BAT_LOOP: &str = "minecraft:entity_bat_loop";
    pub const ENTITY_BAT_TAKEOFF: &str = "minecraft:entity_bat_takeoff";
    pub const ENTITY_BEE_DEATH: &str = "minecraft:entity_bee_death";
    pub const ENTITY_BEE_HURT: &str = "minecraft:entity_bee_hurt";
    pub const ENTITY_BEE_LOOP: &str = "minecraft:entity_bee_loop";
    pub const ENTITY_BEE_LOOP_AGGRESSIVE: &str = "minecraft:entity_bee_loop_aggressive";
    pub const ENTITY_BEE_POLLINATE: &str = "minecraft:entity_bee_pollinate";
    pub const ENTITY_BEE_STING: &str = "minecraft:entity_bee_sting";
    pub const ENTITY_BLAZE_AMBIENT: &str = "minecraft:entity_blaze_ambient";
    pub const ENTITY_BLAZE_BURN: &str = "minecraft:entity_blaze_burn";
    pub const ENTITY_BLAZE_DEATH: &str = "minecraft:entity_blaze_death";
    pub const ENTITY_BLAZE_HURT: &str = "minecraft:entity_blaze_hurt";
    pub const ENTITY_BLAZE_SHOOT: &str = "minecraft:entity_blaze_shoot";
    pub const ENTITY_BOAT_PADDLE_LAND: &str = "minecraft:entity_boat_paddle_land";
    pub const ENTITY_BOAT_PADDLE_WATER: &str = "minecraft:entity_boat_paddle_water";
    pub const ENTITY_BOGGED_AMBIENT: &str = "minecraft:entity_bogged_ambient";
    pub const ENTITY_BOGGED_DEATH: &str = "minecraft:entity_bogged_death";
    pub const ENTITY_BOGGED_HURT: &str = "minecraft:entity_bogged_hurt";
    pub const ENTITY_BOGGED_SHEAR: &str = "minecraft:entity_bogged_shear";
    pub const ENTITY_BOGGED_STEP: &str = "minecraft:entity_bogged_step";
    pub const ENTITY_BREEZE_CHARGE: &str = "minecraft:entity_breeze_charge";
    pub const ENTITY_BREEZE_DEATH: &str = "minecraft:entity_breeze_death";
    pub const ENTITY_BREEZE_DEFLECT: &str = "minecraft:entity_breeze_deflect";
    pub const ENTITY_BREEZE_HURT: &str = "minecraft:entity_breeze_hurt";
    pub const ENTITY_BREEZE_IDLE_AIR: &str = "minecraft:entity_breeze_idle_air";
    pub const ENTITY_BREEZE_IDLE_GROUND: &str = "minecraft:entity_breeze_idle_ground";
    pub const ENTITY_BREEZE_INHALE: &str = "minecraft:entity_breeze_inhale";
    pub const ENTITY_BREEZE_JUMP: &str = "minecraft:entity_breeze_jump";
    pub const ENTITY_BREEZE_LAND: &str = "minecraft:entity_breeze_land";
    pub const ENTITY_BREEZE_SHOOT: &str = "minecraft:entity_breeze_shoot";
    pub const ENTITY_BREEZE_SLIDE: &str = "minecraft:entity_breeze_slide";
    pub const ENTITY_BREEZE_WHIRL: &str = "minecraft:entity_breeze_whirl";
    pub const ENTITY_BREEZE_WIND_BURST: &str = "minecraft:entity_breeze_wind_burst";
    pub const ENTITY_CAMEL_AMBIENT: &str = "minecraft:entity_camel_ambient";
    pub const ENTITY_CAMEL_DASH: &str = "minecraft:entity_camel_dash";
    pub const ENTITY_CAMEL_DASH_READY: &str = "minecraft:entity_camel_dash_ready";
    pub const ENTITY_CAMEL_DEATH: &str = "minecraft:entity_camel_death";
    pub const ENTITY_CAMEL_EAT: &str = "minecraft:entity_camel_eat";
    pub const ENTITY_CAMEL_HURT: &str = "minecraft:entity_camel_hurt";
    pub const ENTITY_CAMEL_SADDLE: &str = "minecraft:entity_camel_saddle";
    pub const ENTITY_CAMEL_SIT: &str = "minecraft:entity_camel_sit";
    pub const ENTITY_CAMEL_STAND: &str = "minecraft:entity_camel_stand";
    pub const ENTITY_CAMEL_STEP: &str = "minecraft:entity_camel_step";
    pub const ENTITY_CAMEL_STEP_SAND: &str = "minecraft:entity_camel_step_sand";
    pub const ENTITY_CAMEL_HUSK_AMBIENT: &str = "minecraft:entity_camel_husk_ambient";
    pub const ENTITY_CAMEL_HUSK_DASH: &str = "minecraft:entity_camel_husk_dash";
    pub const ENTITY_CAMEL_HUSK_DASH_READY: &str = "minecraft:entity_camel_husk_dash_ready";
    pub const ENTITY_CAMEL_HUSK_DEATH: &str = "minecraft:entity_camel_husk_death";
    pub const ENTITY_CAMEL_HUSK_EAT: &str = "minecraft:entity_camel_husk_eat";
    pub const ENTITY_CAMEL_HUSK_HURT: &str = "minecraft:entity_camel_husk_hurt";
    pub const ENTITY_CAMEL_HUSK_SADDLE: &str = "minecraft:entity_camel_husk_saddle";
    pub const ENTITY_CAMEL_HUSK_SIT: &str = "minecraft:entity_camel_husk_sit";
    pub const ENTITY_CAMEL_HUSK_STAND: &str = "minecraft:entity_camel_husk_stand";
    pub const ENTITY_CAMEL_HUSK_STEP: &str = "minecraft:entity_camel_husk_step";
    pub const ENTITY_CAMEL_HUSK_STEP_SAND: &str = "minecraft:entity_camel_husk_step_sand";
    pub const ENTITY_CAT_AMBIENT: &str = "minecraft:entity_cat_ambient";
    pub const ENTITY_CAT_BEG_FOR_FOOD: &str = "minecraft:entity_cat_beg_for_food";
    pub const ENTITY_CAT_DEATH: &str = "minecraft:entity_cat_death";
    pub const ENTITY_CAT_EAT: &str = "minecraft:entity_cat_eat";
    pub const ENTITY_CAT_HISS: &str = "minecraft:entity_cat_hiss";
    pub const ENTITY_CAT_HURT: &str = "minecraft:entity_cat_hurt";
    pub const ENTITY_CAT_PURR: &str = "minecraft:entity_cat_purr";
    pub const ENTITY_CAT_PURREOW: &str = "minecraft:entity_cat_purreow";
    pub const ENTITY_CAT_STRAY_AMBIENT: &str = "minecraft:entity_cat_stray_ambient";
    pub const ENTITY_CAT_ROYAL_AMBIENT: &str = "minecraft:entity_cat_royal_ambient";
    pub const ENTITY_CAT_ROYAL_BEG_FOR_FOOD: &str = "minecraft:entity_cat_royal_beg_for_food";
    pub const ENTITY_CAT_ROYAL_DEATH: &str = "minecraft:entity_cat_royal_death";
    pub const ENTITY_CAT_ROYAL_EAT: &str = "minecraft:entity_cat_royal_eat";
    pub const ENTITY_CAT_ROYAL_HISS: &str = "minecraft:entity_cat_royal_hiss";
    pub const ENTITY_CAT_ROYAL_HURT: &str = "minecraft:entity_cat_royal_hurt";
    pub const ENTITY_CAT_ROYAL_PURR: &str = "minecraft:entity_cat_royal_purr";
    pub const ENTITY_CAT_ROYAL_PURREOW: &str = "minecraft:entity_cat_royal_purreow";
    pub const ENTITY_CAT_ROYAL_STRAY_AMBIENT: &str = "minecraft:entity_cat_royal_stray_ambient";
    pub const ENTITY_CHICKEN_AMBIENT: &str = "minecraft:entity_chicken_ambient";
    pub const ENTITY_CHICKEN_DEATH: &str = "minecraft:entity_chicken_death";
    pub const ENTITY_CHICKEN_EGG: &str = "minecraft:entity_chicken_egg";
    pub const ENTITY_CHICKEN_HURT: &str = "minecraft:entity_chicken_hurt";
    pub const ENTITY_CHICKEN_STEP: &str = "minecraft:entity_chicken_step";
    pub const ENTITY_CHICKEN_PICKY_AMBIENT: &str = "minecraft:entity_chicken_picky_ambient";
    pub const ENTITY_CHICKEN_PICKY_DEATH: &str = "minecraft:entity_chicken_picky_death";
    pub const ENTITY_CHICKEN_PICKY_HURT: &str = "minecraft:entity_chicken_picky_hurt";
    pub const ENTITY_COD_AMBIENT: &str = "minecraft:entity_cod_ambient";
    pub const ENTITY_COD_DEATH: &str = "minecraft:entity_cod_death";
    pub const ENTITY_COD_FLOP: &str = "minecraft:entity_cod_flop";
    pub const ENTITY_COD_HURT: &str = "minecraft:entity_cod_hurt";
    pub const ENTITY_COPPER_GOLEM_DEATH: &str = "minecraft:entity_copper_golem_death";
    pub const ENTITY_COPPER_GOLEM_HURT: &str = "minecraft:entity_copper_golem_hurt";
    pub const ENTITY_COPPER_GOLEM_ITEM_DROP: &str = "minecraft:entity_copper_golem_item_drop";
    pub const ENTITY_COPPER_GOLEM_ITEM_NO_DROP: &str = "minecraft:entity_copper_golem_item_no_drop";
    pub const ENTITY_COPPER_GOLEM_NO_ITEM_GET: &str = "minecraft:entity_copper_golem_no_item_get";
    pub const ENTITY_COPPER_GOLEM_NO_ITEM_NO_GET: &str = "minecraft:entity_copper_golem_no_item_no_get";
    pub const ENTITY_COPPER_GOLEM_SHEAR: &str = "minecraft:entity_copper_golem_shear";
    pub const ENTITY_COPPER_GOLEM_SPAWN: &str = "minecraft:entity_copper_golem_spawn";
    pub const ENTITY_COPPER_GOLEM_SPIN: &str = "minecraft:entity_copper_golem_spin";
    pub const ENTITY_COPPER_GOLEM_STEP: &str = "minecraft:entity_copper_golem_step";
    pub const ENTITY_COPPER_GOLEM_BECOME_STATUE: &str = "minecraft:entity_copper_golem_become_statue";
    pub const ENTITY_COPPER_GOLEM_OXIDIZED_DEATH: &str = "minecraft:entity_copper_golem_oxidized_death";
    pub const ENTITY_COPPER_GOLEM_OXIDIZED_HURT: &str = "minecraft:entity_copper_golem_oxidized_hurt";
    pub const ENTITY_COPPER_GOLEM_OXIDIZED_SPIN: &str = "minecraft:entity_copper_golem_oxidized_spin";
    pub const ENTITY_COPPER_GOLEM_OXIDIZED_STEP: &str = "minecraft:entity_copper_golem_oxidized_step";
    pub const ENTITY_COPPER_GOLEM_WEATHERED_DEATH: &str = "minecraft:entity_copper_golem_weathered_death";
    pub const ENTITY_COPPER_GOLEM_WEATHERED_HURT: &str = "minecraft:entity_copper_golem_weathered_hurt";
    pub const ENTITY_COPPER_GOLEM_WEATHERED_SPIN: &str = "minecraft:entity_copper_golem_weathered_spin";
    pub const ENTITY_COPPER_GOLEM_WEATHERED_STEP: &str = "minecraft:entity_copper_golem_weathered_step";
    pub const ENTITY_COW_AMBIENT: &str = "minecraft:entity_cow_ambient";
    pub const ENTITY_COW_DEATH: &str = "minecraft:entity_cow_death";
    pub const ENTITY_COW_HURT: &str = "minecraft:entity_cow_hurt";
    pub const ENTITY_COW_MILK: &str = "minecraft:entity_cow_milk";
    pub const ENTITY_COW_STEP: &str = "minecraft:entity_cow_step";
    pub const ENTITY_COW_MOODY_AMBIENT: &str = "minecraft:entity_cow_moody_ambient";
    pub const ENTITY_COW_MOODY_DEATH: &str = "minecraft:entity_cow_moody_death";
    pub const ENTITY_COW_MOODY_HURT: &str = "minecraft:entity_cow_moody_hurt";
    pub const ENTITY_COW_MOODY_STEP: &str = "minecraft:entity_cow_moody_step";
    pub const ENTITY_CREAKING_ACTIVATE: &str = "minecraft:entity_creaking_activate";
    pub const ENTITY_CREAKING_AMBIENT: &str = "minecraft:entity_creaking_ambient";
    pub const ENTITY_CREAKING_ATTACK: &str = "minecraft:entity_creaking_attack";
    pub const ENTITY_CREAKING_DEACTIVATE: &str = "minecraft:entity_creaking_deactivate";
    pub const ENTITY_CREAKING_DEATH: &str = "minecraft:entity_creaking_death";
    pub const ENTITY_CREAKING_FREEZE: &str = "minecraft:entity_creaking_freeze";
    pub const ENTITY_CREAKING_SPAWN: &str = "minecraft:entity_creaking_spawn";
    pub const ENTITY_CREAKING_STEP: &str = "minecraft:entity_creaking_step";
    pub const ENTITY_CREAKING_SWAY: &str = "minecraft:entity_creaking_sway";
    pub const ENTITY_CREAKING_TWITCH: &str = "minecraft:entity_creaking_twitch";
    pub const ENTITY_CREAKING_UNFREEZE: &str = "minecraft:entity_creaking_unfreeze";
    pub const ENTITY_CREEPER_DEATH: &str = "minecraft:entity_creeper_death";
    pub const ENTITY_CREEPER_HURT: &str = "minecraft:entity_creeper_hurt";
    pub const ENTITY_CREEPER_PRIMED: &str = "minecraft:entity_creeper_primed";
    pub const ENTITY_CUSHION_BREAK: &str = "minecraft:entity_cushion_break";
    pub const ENTITY_CUSHION_GET_UP: &str = "minecraft:entity_cushion_get_up";
    pub const ENTITY_CUSHION_PLACE: &str = "minecraft:entity_cushion_place";
    pub const ENTITY_CUSHION_SIT: &str = "minecraft:entity_cushion_sit";
    pub const ENTITY_DOLPHIN_AMBIENT: &str = "minecraft:entity_dolphin_ambient";
    pub const ENTITY_DOLPHIN_AMBIENT_WATER: &str = "minecraft:entity_dolphin_ambient_water";
    pub const ENTITY_DOLPHIN_ATTACK: &str = "minecraft:entity_dolphin_attack";
    pub const ENTITY_DOLPHIN_DEATH: &str = "minecraft:entity_dolphin_death";
    pub const ENTITY_DOLPHIN_EAT: &str = "minecraft:entity_dolphin_eat";
    pub const ENTITY_DOLPHIN_HURT: &str = "minecraft:entity_dolphin_hurt";
    pub const ENTITY_DOLPHIN_JUMP: &str = "minecraft:entity_dolphin_jump";
    pub const ENTITY_DOLPHIN_PLAY: &str = "minecraft:entity_dolphin_play";
    pub const ENTITY_DOLPHIN_SPLASH: &str = "minecraft:entity_dolphin_splash";
    pub const ENTITY_DOLPHIN_SWIM: &str = "minecraft:entity_dolphin_swim";
    pub const ENTITY_DONKEY_AMBIENT: &str = "minecraft:entity_donkey_ambient";
    pub const ENTITY_DONKEY_ANGRY: &str = "minecraft:entity_donkey_angry";
    pub const ENTITY_DONKEY_CHEST: &str = "minecraft:entity_donkey_chest";
    pub const ENTITY_DONKEY_DEATH: &str = "minecraft:entity_donkey_death";
    pub const ENTITY_DONKEY_EAT: &str = "minecraft:entity_donkey_eat";
    pub const ENTITY_DONKEY_HURT: &str = "minecraft:entity_donkey_hurt";
    pub const ENTITY_DONKEY_JUMP: &str = "minecraft:entity_donkey_jump";
    pub const ENTITY_DRAGON_FIREBALL_EXPLODE: &str = "minecraft:entity_dragon_fireball_explode";
    pub const ENTITY_DROWNED_AMBIENT: &str = "minecraft:entity_drowned_ambient";
    pub const ENTITY_DROWNED_AMBIENT_WATER: &str = "minecraft:entity_drowned_ambient_water";
    pub const ENTITY_DROWNED_DEATH: &str = "minecraft:entity_drowned_death";
    pub const ENTITY_DROWNED_DEATH_WATER: &str = "minecraft:entity_drowned_death_water";
    pub const ENTITY_DROWNED_HURT: &str = "minecraft:entity_drowned_hurt";
    pub const ENTITY_DROWNED_HURT_WATER: &str = "minecraft:entity_drowned_hurt_water";
    pub const ENTITY_DROWNED_SHOOT: &str = "minecraft:entity_drowned_shoot";
    pub const ENTITY_DROWNED_STEP: &str = "minecraft:entity_drowned_step";
    pub const ENTITY_DROWNED_SWIM: &str = "minecraft:entity_drowned_swim";
    pub const ENTITY_EGG_THROW: &str = "minecraft:entity_egg_throw";
    pub const ENTITY_ELDER_GUARDIAN_AMBIENT: &str = "minecraft:entity_elder_guardian_ambient";
    pub const ENTITY_ELDER_GUARDIAN_AMBIENT_LAND: &str = "minecraft:entity_elder_guardian_ambient_land";
    pub const ENTITY_ELDER_GUARDIAN_CURSE: &str = "minecraft:entity_elder_guardian_curse";
    pub const ENTITY_ELDER_GUARDIAN_DEATH: &str = "minecraft:entity_elder_guardian_death";
    pub const ENTITY_ELDER_GUARDIAN_DEATH_LAND: &str = "minecraft:entity_elder_guardian_death_land";
    pub const ENTITY_ELDER_GUARDIAN_FLOP: &str = "minecraft:entity_elder_guardian_flop";
    pub const ENTITY_ELDER_GUARDIAN_HURT: &str = "minecraft:entity_elder_guardian_hurt";
    pub const ENTITY_ELDER_GUARDIAN_HURT_LAND: &str = "minecraft:entity_elder_guardian_hurt_land";
    pub const ENTITY_ENDER_DRAGON_AMBIENT: &str = "minecraft:entity_ender_dragon_ambient";
    pub const ENTITY_ENDER_DRAGON_DEATH: &str = "minecraft:entity_ender_dragon_death";
    pub const ENTITY_ENDER_DRAGON_FLAP: &str = "minecraft:entity_ender_dragon_flap";
    pub const ENTITY_ENDER_DRAGON_GROWL: &str = "minecraft:entity_ender_dragon_growl";
    pub const ENTITY_ENDER_DRAGON_HURT: &str = "minecraft:entity_ender_dragon_hurt";
    pub const ENTITY_ENDER_DRAGON_SHOOT: &str = "minecraft:entity_ender_dragon_shoot";
    pub const ENTITY_ENDER_EYE_DEATH: &str = "minecraft:entity_ender_eye_death";
    pub const ENTITY_ENDER_EYE_LAUNCH: &str = "minecraft:entity_ender_eye_launch";
    pub const ENTITY_ENDER_PEARL_THROW: &str = "minecraft:entity_ender_pearl_throw";
    pub const ENTITY_ENDERMAN_AMBIENT: &str = "minecraft:entity_enderman_ambient";
    pub const ENTITY_ENDERMAN_DEATH: &str = "minecraft:entity_enderman_death";
    pub const ENTITY_ENDERMAN_HURT: &str = "minecraft:entity_enderman_hurt";
    pub const ENTITY_ENDERMAN_SCREAM: &str = "minecraft:entity_enderman_scream";
    pub const ENTITY_ENDERMAN_STARE: &str = "minecraft:entity_enderman_stare";
    pub const ENTITY_ENDERMAN_TELEPORT: &str = "minecraft:entity_enderman_teleport";
    pub const ENTITY_ENDERMITE_AMBIENT: &str = "minecraft:entity_endermite_ambient";
    pub const ENTITY_ENDERMITE_DEATH: &str = "minecraft:entity_endermite_death";
    pub const ENTITY_ENDERMITE_HURT: &str = "minecraft:entity_endermite_hurt";
    pub const ENTITY_ENDERMITE_STEP: &str = "minecraft:entity_endermite_step";
    pub const ENTITY_EVOKER_AMBIENT: &str = "minecraft:entity_evoker_ambient";
    pub const ENTITY_EVOKER_CAST_SPELL: &str = "minecraft:entity_evoker_cast_spell";
    pub const ENTITY_EVOKER_CELEBRATE: &str = "minecraft:entity_evoker_celebrate";
    pub const ENTITY_EVOKER_DEATH: &str = "minecraft:entity_evoker_death";
    pub const ENTITY_EVOKER_HURT: &str = "minecraft:entity_evoker_hurt";
    pub const ENTITY_EVOKER_PREPARE_ATTACK: &str = "minecraft:entity_evoker_prepare_attack";
    pub const ENTITY_EVOKER_PREPARE_SUMMON: &str = "minecraft:entity_evoker_prepare_summon";
    pub const ENTITY_EVOKER_PREPARE_WOLOLO: &str = "minecraft:entity_evoker_prepare_wololo";
    pub const ENTITY_EVOKER_FANGS_ATTACK: &str = "minecraft:entity_evoker_fangs_attack";
    pub const ENTITY_EXPERIENCE_BOTTLE_THROW: &str = "minecraft:entity_experience_bottle_throw";
    pub const ENTITY_EXPERIENCE_ORB_PICKUP: &str = "minecraft:entity_experience_orb_pickup";
    pub const ENTITY_FIREWORK_ROCKET_BLAST: &str = "minecraft:entity_firework_rocket_blast";
    pub const ENTITY_FIREWORK_ROCKET_BLAST_FAR: &str = "minecraft:entity_firework_rocket_blast_far";
    pub const ENTITY_FIREWORK_ROCKET_LARGE_BLAST: &str = "minecraft:entity_firework_rocket_large_blast";
    pub const ENTITY_FIREWORK_ROCKET_LARGE_BLAST_FAR: &str = "minecraft:entity_firework_rocket_large_blast_far";
    pub const ENTITY_FIREWORK_ROCKET_LAUNCH: &str = "minecraft:entity_firework_rocket_launch";
    pub const ENTITY_FIREWORK_ROCKET_SHOOT: &str = "minecraft:entity_firework_rocket_shoot";
    pub const ENTITY_FIREWORK_ROCKET_TWINKLE: &str = "minecraft:entity_firework_rocket_twinkle";
    pub const ENTITY_FIREWORK_ROCKET_TWINKLE_FAR: &str = "minecraft:entity_firework_rocket_twinkle_far";
    pub const ENTITY_FISH_SWIM: &str = "minecraft:entity_fish_swim";
    pub const ENTITY_FISHING_BOBBER_RETRIEVE: &str = "minecraft:entity_fishing_bobber_retrieve";
    pub const ENTITY_FISHING_BOBBER_SPLASH: &str = "minecraft:entity_fishing_bobber_splash";
    pub const ENTITY_FISHING_BOBBER_THROW: &str = "minecraft:entity_fishing_bobber_throw";
    pub const ENTITY_FOX_AGGRO: &str = "minecraft:entity_fox_aggro";
    pub const ENTITY_FOX_AMBIENT: &str = "minecraft:entity_fox_ambient";
    pub const ENTITY_FOX_BITE: &str = "minecraft:entity_fox_bite";
    pub const ENTITY_FOX_DEATH: &str = "minecraft:entity_fox_death";
    pub const ENTITY_FOX_EAT: &str = "minecraft:entity_fox_eat";
    pub const ENTITY_FOX_HURT: &str = "minecraft:entity_fox_hurt";
    pub const ENTITY_FOX_SCREECH: &str = "minecraft:entity_fox_screech";
    pub const ENTITY_FOX_SLEEP: &str = "minecraft:entity_fox_sleep";
    pub const ENTITY_FOX_SNIFF: &str = "minecraft:entity_fox_sniff";
    pub const ENTITY_FOX_SPIT: &str = "minecraft:entity_fox_spit";
    pub const ENTITY_FOX_TELEPORT: &str = "minecraft:entity_fox_teleport";
    pub const ENTITY_FROG_AMBIENT: &str = "minecraft:entity_frog_ambient";
    pub const ENTITY_FROG_DEATH: &str = "minecraft:entity_frog_death";
    pub const ENTITY_FROG_EAT: &str = "minecraft:entity_frog_eat";
    pub const ENTITY_FROG_HURT: &str = "minecraft:entity_frog_hurt";
    pub const ENTITY_FROG_LAY_SPAWN: &str = "minecraft:entity_frog_lay_spawn";
    pub const ENTITY_FROG_LONG_JUMP: &str = "minecraft:entity_frog_long_jump";
    pub const ENTITY_FROG_STEP: &str = "minecraft:entity_frog_step";
    pub const ENTITY_FROG_TONGUE: &str = "minecraft:entity_frog_tongue";
    pub const ENTITY_GENERIC_BIG_FALL: &str = "minecraft:entity_generic_big_fall";
    pub const ENTITY_GENERIC_BURN: &str = "minecraft:entity_generic_burn";
    pub const ENTITY_GENERIC_DEATH: &str = "minecraft:entity_generic_death";
    pub const ENTITY_GENERIC_DRINK: &str = "minecraft:entity_generic_drink";
    pub const ENTITY_GENERIC_EAT: &str = "minecraft:entity_generic_eat";
    pub const ENTITY_GENERIC_EXPLODE: &str = "minecraft:entity_generic_explode";
    pub const ENTITY_GENERIC_EXTINGUISH_FIRE: &str = "minecraft:entity_generic_extinguish_fire";
    pub const ENTITY_GENERIC_HURT: &str = "minecraft:entity_generic_hurt";
    pub const ENTITY_GENERIC_SMALL_FALL: &str = "minecraft:entity_generic_small_fall";
    pub const ENTITY_GENERIC_SPLASH: &str = "minecraft:entity_generic_splash";
    pub const ENTITY_GENERIC_SWIM: &str = "minecraft:entity_generic_swim";
    pub const ENTITY_GHAST_AMBIENT: &str = "minecraft:entity_ghast_ambient";
    pub const ENTITY_GHAST_DEATH: &str = "minecraft:entity_ghast_death";
    pub const ENTITY_GHAST_HURT: &str = "minecraft:entity_ghast_hurt";
    pub const ENTITY_GHAST_SCREAM: &str = "minecraft:entity_ghast_scream";
    pub const ENTITY_GHAST_SHOOT: &str = "minecraft:entity_ghast_shoot";
    pub const ENTITY_GHAST_WARN: &str = "minecraft:entity_ghast_warn";
    pub const ENTITY_GHASTLING_AMBIENT: &str = "minecraft:entity_ghastling_ambient";
    pub const ENTITY_GHASTLING_DEATH: &str = "minecraft:entity_ghastling_death";
    pub const ENTITY_GHASTLING_HURT: &str = "minecraft:entity_ghastling_hurt";
    pub const ENTITY_GHASTLING_SPAWN: &str = "minecraft:entity_ghastling_spawn";
    pub const ENTITY_GLOW_ITEM_FRAME_ADD_ITEM: &str = "minecraft:entity_glow_item_frame_add_item";
    pub const ENTITY_GLOW_ITEM_FRAME_BREAK: &str = "minecraft:entity_glow_item_frame_break";
    pub const ENTITY_GLOW_ITEM_FRAME_PLACE: &str = "minecraft:entity_glow_item_frame_place";
    pub const ENTITY_GLOW_ITEM_FRAME_REMOVE_ITEM: &str = "minecraft:entity_glow_item_frame_remove_item";
    pub const ENTITY_GLOW_ITEM_FRAME_ROTATE_ITEM: &str = "minecraft:entity_glow_item_frame_rotate_item";
    pub const ENTITY_GLOW_SQUID_AMBIENT: &str = "minecraft:entity_glow_squid_ambient";
    pub const ENTITY_GLOW_SQUID_DEATH: &str = "minecraft:entity_glow_squid_death";
    pub const ENTITY_GLOW_SQUID_HURT: &str = "minecraft:entity_glow_squid_hurt";
    pub const ENTITY_GLOW_SQUID_SQUIRT: &str = "minecraft:entity_glow_squid_squirt";
    pub const ENTITY_GOAT_AMBIENT: &str = "minecraft:entity_goat_ambient";
    pub const ENTITY_GOAT_DEATH: &str = "minecraft:entity_goat_death";
    pub const ENTITY_GOAT_EAT: &str = "minecraft:entity_goat_eat";
    pub const ENTITY_GOAT_HORN_BREAK: &str = "minecraft:entity_goat_horn_break";
    pub const ENTITY_GOAT_HURT: &str = "minecraft:entity_goat_hurt";
    pub const ENTITY_GOAT_LONG_JUMP: &str = "minecraft:entity_goat_long_jump";
    pub const ENTITY_GOAT_MILK: &str = "minecraft:entity_goat_milk";
    pub const ENTITY_GOAT_PREPARE_RAM: &str = "minecraft:entity_goat_prepare_ram";
    pub const ENTITY_GOAT_RAM_IMPACT: &str = "minecraft:entity_goat_ram_impact";
    pub const ENTITY_GOAT_SCREAMING_AMBIENT: &str = "minecraft:entity_goat_screaming_ambient";
    pub const ENTITY_GOAT_SCREAMING_DEATH: &str = "minecraft:entity_goat_screaming_death";
    pub const ENTITY_GOAT_SCREAMING_EAT: &str = "minecraft:entity_goat_screaming_eat";
    pub const ENTITY_GOAT_SCREAMING_HURT: &str = "minecraft:entity_goat_screaming_hurt";
    pub const ENTITY_GOAT_SCREAMING_LONG_JUMP: &str = "minecraft:entity_goat_screaming_long_jump";
    pub const ENTITY_GOAT_SCREAMING_MILK: &str = "minecraft:entity_goat_screaming_milk";
    pub const ENTITY_GOAT_SCREAMING_PREPARE_RAM: &str = "minecraft:entity_goat_screaming_prepare_ram";
    pub const ENTITY_GOAT_SCREAMING_RAM_IMPACT: &str = "minecraft:entity_goat_screaming_ram_impact";
    pub const ENTITY_GOAT_STEP: &str = "minecraft:entity_goat_step";
    pub const ENTITY_GUARDIAN_AMBIENT: &str = "minecraft:entity_guardian_ambient";
    pub const ENTITY_GUARDIAN_AMBIENT_LAND: &str = "minecraft:entity_guardian_ambient_land";
    pub const ENTITY_GUARDIAN_ATTACK: &str = "minecraft:entity_guardian_attack";
    pub const ENTITY_GUARDIAN_DEATH: &str = "minecraft:entity_guardian_death";
    pub const ENTITY_GUARDIAN_DEATH_LAND: &str = "minecraft:entity_guardian_death_land";
    pub const ENTITY_GUARDIAN_FLOP: &str = "minecraft:entity_guardian_flop";
    pub const ENTITY_GUARDIAN_HURT: &str = "minecraft:entity_guardian_hurt";
    pub const ENTITY_GUARDIAN_HURT_LAND: &str = "minecraft:entity_guardian_hurt_land";
    pub const ENTITY_HAPPY_GHAST_AMBIENT: &str = "minecraft:entity_happy_ghast_ambient";
    pub const ENTITY_HAPPY_GHAST_DEATH: &str = "minecraft:entity_happy_ghast_death";
    pub const ENTITY_HAPPY_GHAST_EQUIP: &str = "minecraft:entity_happy_ghast_equip";
    pub const ENTITY_HAPPY_GHAST_HARNESS_GOGGLES_DOWN: &str = "minecraft:entity_happy_ghast_harness_goggles_down";
    pub const ENTITY_HAPPY_GHAST_HARNESS_GOGGLES_UP: &str = "minecraft:entity_happy_ghast_harness_goggles_up";
    pub const ENTITY_HAPPY_GHAST_HURT: &str = "minecraft:entity_happy_ghast_hurt";
    pub const ENTITY_HAPPY_GHAST_RIDING: &str = "minecraft:entity_happy_ghast_riding";
    pub const ENTITY_HAPPY_GHAST_UNEQUIP: &str = "minecraft:entity_happy_ghast_unequip";
    pub const ENTITY_HOGLIN_AMBIENT: &str = "minecraft:entity_hoglin_ambient";
    pub const ENTITY_HOGLIN_ANGRY: &str = "minecraft:entity_hoglin_angry";
    pub const ENTITY_HOGLIN_ATTACK: &str = "minecraft:entity_hoglin_attack";
    pub const ENTITY_HOGLIN_CONVERTED_TO_ZOMBIFIED: &str = "minecraft:entity_hoglin_converted_to_zombified";
    pub const ENTITY_HOGLIN_DEATH: &str = "minecraft:entity_hoglin_death";
    pub const ENTITY_HOGLIN_HURT: &str = "minecraft:entity_hoglin_hurt";
    pub const ENTITY_HOGLIN_RETREAT: &str = "minecraft:entity_hoglin_retreat";
    pub const ENTITY_HOGLIN_STEP: &str = "minecraft:entity_hoglin_step";
    pub const ENTITY_HORSE_AMBIENT: &str = "minecraft:entity_horse_ambient";
    pub const ENTITY_HORSE_ANGRY: &str = "minecraft:entity_horse_angry";
    pub const ENTITY_HORSE_ARMOR: &str = "minecraft:entity_horse_armor";
    pub const ENTITY_HORSE_BREATHE: &str = "minecraft:entity_horse_breathe";
    pub const ENTITY_HORSE_DEATH: &str = "minecraft:entity_horse_death";
    pub const ENTITY_HORSE_EAT: &str = "minecraft:entity_horse_eat";
    pub const ENTITY_HORSE_GALLOP: &str = "minecraft:entity_horse_gallop";
    pub const ENTITY_HORSE_HURT: &str = "minecraft:entity_horse_hurt";
    pub const ENTITY_HORSE_JUMP: &str = "minecraft:entity_horse_jump";
    pub const ENTITY_HORSE_LAND: &str = "minecraft:entity_horse_land";
    pub const ENTITY_HORSE_SADDLE: &str = "minecraft:entity_horse_saddle";
    pub const ENTITY_HORSE_STEP: &str = "minecraft:entity_horse_step";
    pub const ENTITY_HORSE_STEP_WOOD: &str = "minecraft:entity_horse_step_wood";
    pub const ENTITY_HOSTILE_BIG_FALL: &str = "minecraft:entity_hostile_big_fall";
    pub const ENTITY_HOSTILE_DEATH: &str = "minecraft:entity_hostile_death";
    pub const ENTITY_HOSTILE_HURT: &str = "minecraft:entity_hostile_hurt";
    pub const ENTITY_HOSTILE_SMALL_FALL: &str = "minecraft:entity_hostile_small_fall";
    pub const ENTITY_HOSTILE_SPLASH: &str = "minecraft:entity_hostile_splash";
    pub const ENTITY_HOSTILE_SWIM: &str = "minecraft:entity_hostile_swim";
    pub const ENTITY_HUSK_AMBIENT: &str = "minecraft:entity_husk_ambient";
    pub const ENTITY_HUSK_CONVERTED_TO_ZOMBIE: &str = "minecraft:entity_husk_converted_to_zombie";
    pub const ENTITY_HUSK_DEATH: &str = "minecraft:entity_husk_death";
    pub const ENTITY_HUSK_HURT: &str = "minecraft:entity_husk_hurt";
    pub const ENTITY_HUSK_STEP: &str = "minecraft:entity_husk_step";
    pub const ENTITY_ILLUSIONER_AMBIENT: &str = "minecraft:entity_illusioner_ambient";
    pub const ENTITY_ILLUSIONER_CAST_SPELL: &str = "minecraft:entity_illusioner_cast_spell";
    pub const ENTITY_ILLUSIONER_DEATH: &str = "minecraft:entity_illusioner_death";
    pub const ENTITY_ILLUSIONER_HURT: &str = "minecraft:entity_illusioner_hurt";
    pub const ENTITY_ILLUSIONER_MIRROR_MOVE: &str = "minecraft:entity_illusioner_mirror_move";
    pub const ENTITY_ILLUSIONER_PREPARE_BLINDNESS: &str = "minecraft:entity_illusioner_prepare_blindness";
    pub const ENTITY_ILLUSIONER_PREPARE_MIRROR: &str = "minecraft:entity_illusioner_prepare_mirror";
    pub const ENTITY_IRON_GOLEM_ATTACK: &str = "minecraft:entity_iron_golem_attack";
    pub const ENTITY_IRON_GOLEM_DAMAGE: &str = "minecraft:entity_iron_golem_damage";
    pub const ENTITY_IRON_GOLEM_DEATH: &str = "minecraft:entity_iron_golem_death";
    pub const ENTITY_IRON_GOLEM_HURT: &str = "minecraft:entity_iron_golem_hurt";
    pub const ENTITY_IRON_GOLEM_REPAIR: &str = "minecraft:entity_iron_golem_repair";
    pub const ENTITY_IRON_GOLEM_STEP: &str = "minecraft:entity_iron_golem_step";
    pub const ENTITY_ITEM_BREAK: &str = "minecraft:entity_item_break";
    pub const ENTITY_ITEM_PICKUP: &str = "minecraft:entity_item_pickup";
    pub const ENTITY_ITEM_FRAME_ADD_ITEM: &str = "minecraft:entity_item_frame_add_item";
    pub const ENTITY_ITEM_FRAME_BREAK: &str = "minecraft:entity_item_frame_break";
    pub const ENTITY_ITEM_FRAME_PLACE: &str = "minecraft:entity_item_frame_place";
    pub const ENTITY_ITEM_FRAME_REMOVE_ITEM: &str = "minecraft:entity_item_frame_remove_item";
    pub const ENTITY_ITEM_FRAME_ROTATE_ITEM: &str = "minecraft:entity_item_frame_rotate_item";
    pub const ENTITY_LIGHTNING_BOLT_IMPACT: &str = "minecraft:entity_lightning_bolt_impact";
    pub const ENTITY_LIGHTNING_BOLT_THUNDER: &str = "minecraft:entity_lightning_bolt_thunder";
    pub const ENTITY_LINGERING_POTION_THROW: &str = "minecraft:entity_lingering_potion_throw";
    pub const ENTITY_LLAMA_AMBIENT: &str = "minecraft:entity_llama_ambient";
    pub const ENTITY_LLAMA_ANGRY: &str = "minecraft:entity_llama_angry";
    pub const ENTITY_LLAMA_CHEST: &str = "minecraft:entity_llama_chest";
    pub const ENTITY_LLAMA_DEATH: &str = "minecraft:entity_llama_death";
    pub const ENTITY_LLAMA_EAT: &str = "minecraft:entity_llama_eat";
    pub const ENTITY_LLAMA_HURT: &str = "minecraft:entity_llama_hurt";
    pub const ENTITY_LLAMA_SPIT: &str = "minecraft:entity_llama_spit";
    pub const ENTITY_LLAMA_STEP: &str = "minecraft:entity_llama_step";
    pub const ENTITY_LLAMA_SWAG: &str = "minecraft:entity_llama_swag";
    pub const ENTITY_MAGMA_CUBE_DEATH: &str = "minecraft:entity_magma_cube_death";
    pub const ENTITY_MAGMA_CUBE_DEATH_SMALL: &str = "minecraft:entity_magma_cube_death_small";
    pub const ENTITY_MAGMA_CUBE_HURT: &str = "minecraft:entity_magma_cube_hurt";
    pub const ENTITY_MAGMA_CUBE_HURT_SMALL: &str = "minecraft:entity_magma_cube_hurt_small";
    pub const ENTITY_MAGMA_CUBE_JUMP: &str = "minecraft:entity_magma_cube_jump";
    pub const ENTITY_MAGMA_CUBE_SQUISH: &str = "minecraft:entity_magma_cube_squish";
    pub const ENTITY_MAGMA_CUBE_SQUISH_SMALL: &str = "minecraft:entity_magma_cube_squish_small";
    pub const ENTITY_MINECART_INSIDE: &str = "minecraft:entity_minecart_inside";
    pub const ENTITY_MINECART_INSIDE_UNDERWATER: &str = "minecraft:entity_minecart_inside_underwater";
    pub const ENTITY_MINECART_RIDING: &str = "minecraft:entity_minecart_riding";
    pub const ENTITY_MOOSHROOM_CONVERT: &str = "minecraft:entity_mooshroom_convert";
    pub const ENTITY_MOOSHROOM_EAT: &str = "minecraft:entity_mooshroom_eat";
    pub const ENTITY_MOOSHROOM_MILK: &str = "minecraft:entity_mooshroom_milk";
    pub const ENTITY_MOOSHROOM_SHEAR: &str = "minecraft:entity_mooshroom_shear";
    pub const ENTITY_MOOSHROOM_SUSPICIOUS_MILK: &str = "minecraft:entity_mooshroom_suspicious_milk";
    pub const ENTITY_MULE_AMBIENT: &str = "minecraft:entity_mule_ambient";
    pub const ENTITY_MULE_ANGRY: &str = "minecraft:entity_mule_angry";
    pub const ENTITY_MULE_CHEST: &str = "minecraft:entity_mule_chest";
    pub const ENTITY_MULE_DEATH: &str = "minecraft:entity_mule_death";
    pub const ENTITY_MULE_EAT: &str = "minecraft:entity_mule_eat";
    pub const ENTITY_MULE_HURT: &str = "minecraft:entity_mule_hurt";
    pub const ENTITY_MULE_JUMP: &str = "minecraft:entity_mule_jump";
    pub const ENTITY_NAUTILUS_AMBIENT: &str = "minecraft:entity_nautilus_ambient";
    pub const ENTITY_NAUTILUS_AMBIENT_LAND: &str = "minecraft:entity_nautilus_ambient_land";
    pub const ENTITY_NAUTILUS_DASH: &str = "minecraft:entity_nautilus_dash";
    pub const ENTITY_NAUTILUS_DASH_LAND: &str = "minecraft:entity_nautilus_dash_land";
    pub const ENTITY_NAUTILUS_DASH_READY: &str = "minecraft:entity_nautilus_dash_ready";
    pub const ENTITY_NAUTILUS_DASH_READY_LAND: &str = "minecraft:entity_nautilus_dash_ready_land";
    pub const ENTITY_NAUTILUS_DEATH: &str = "minecraft:entity_nautilus_death";
    pub const ENTITY_NAUTILUS_DEATH_LAND: &str = "minecraft:entity_nautilus_death_land";
    pub const ENTITY_NAUTILUS_EAT: &str = "minecraft:entity_nautilus_eat";
    pub const ENTITY_NAUTILUS_HURT: &str = "minecraft:entity_nautilus_hurt";
    pub const ENTITY_NAUTILUS_HURT_LAND: &str = "minecraft:entity_nautilus_hurt_land";
    pub const ENTITY_NAUTILUS_RIDING: &str = "minecraft:entity_nautilus_riding";
    pub const ENTITY_NAUTILUS_SWIM: &str = "minecraft:entity_nautilus_swim";
    pub const ENTITY_OCELOT_AMBIENT: &str = "minecraft:entity_ocelot_ambient";
    pub const ENTITY_OCELOT_DEATH: &str = "minecraft:entity_ocelot_death";
    pub const ENTITY_OCELOT_HURT: &str = "minecraft:entity_ocelot_hurt";
    pub const ENTITY_PAINTING_BREAK: &str = "minecraft:entity_painting_break";
    pub const ENTITY_PAINTING_PLACE: &str = "minecraft:entity_painting_place";
    pub const ENTITY_PANDA_AGGRESSIVE_AMBIENT: &str = "minecraft:entity_panda_aggressive_ambient";
    pub const ENTITY_PANDA_AMBIENT: &str = "minecraft:entity_panda_ambient";
    pub const ENTITY_PANDA_BITE: &str = "minecraft:entity_panda_bite";
    pub const ENTITY_PANDA_CANT_BREED: &str = "minecraft:entity_panda_cant_breed";
    pub const ENTITY_PANDA_DEATH: &str = "minecraft:entity_panda_death";
    pub const ENTITY_PANDA_EAT: &str = "minecraft:entity_panda_eat";
    pub const ENTITY_PANDA_HURT: &str = "minecraft:entity_panda_hurt";
    pub const ENTITY_PANDA_PRE_SNEEZE: &str = "minecraft:entity_panda_pre_sneeze";
    pub const ENTITY_PANDA_SNEEZE: &str = "minecraft:entity_panda_sneeze";
    pub const ENTITY_PANDA_STEP: &str = "minecraft:entity_panda_step";
    pub const ENTITY_PANDA_WORRIED_AMBIENT: &str = "minecraft:entity_panda_worried_ambient";
    pub const ENTITY_PARCHED_AMBIENT: &str = "minecraft:entity_parched_ambient";
    pub const ENTITY_PARCHED_DEATH: &str = "minecraft:entity_parched_death";
    pub const ENTITY_PARCHED_HURT: &str = "minecraft:entity_parched_hurt";
    pub const ENTITY_PARCHED_STEP: &str = "minecraft:entity_parched_step";
    pub const ENTITY_PARROT_AMBIENT: &str = "minecraft:entity_parrot_ambient";
    pub const ENTITY_PARROT_DEATH: &str = "minecraft:entity_parrot_death";
    pub const ENTITY_PARROT_EAT: &str = "minecraft:entity_parrot_eat";
    pub const ENTITY_PARROT_FLY: &str = "minecraft:entity_parrot_fly";
    pub const ENTITY_PARROT_HURT: &str = "minecraft:entity_parrot_hurt";
    pub const ENTITY_PARROT_IMITATE_BLAZE: &str = "minecraft:entity_parrot_imitate_blaze";
    pub const ENTITY_PARROT_IMITATE_BOGGED: &str = "minecraft:entity_parrot_imitate_bogged";
    pub const ENTITY_PARROT_IMITATE_BREEZE: &str = "minecraft:entity_parrot_imitate_breeze";
    pub const ENTITY_PARROT_IMITATE_CAMEL_HUSK: &str = "minecraft:entity_parrot_imitate_camel_husk";
    pub const ENTITY_PARROT_IMITATE_CREAKING: &str = "minecraft:entity_parrot_imitate_creaking";
    pub const ENTITY_PARROT_IMITATE_CREEPER: &str = "minecraft:entity_parrot_imitate_creeper";
    pub const ENTITY_PARROT_IMITATE_DROWNED: &str = "minecraft:entity_parrot_imitate_drowned";
    pub const ENTITY_PARROT_IMITATE_ELDER_GUARDIAN: &str = "minecraft:entity_parrot_imitate_elder_guardian";
    pub const ENTITY_PARROT_IMITATE_ENDER_DRAGON: &str = "minecraft:entity_parrot_imitate_ender_dragon";
    pub const ENTITY_PARROT_IMITATE_ENDERMITE: &str = "minecraft:entity_parrot_imitate_endermite";
    pub const ENTITY_PARROT_IMITATE_EVOKER: &str = "minecraft:entity_parrot_imitate_evoker";
    pub const ENTITY_PARROT_IMITATE_GHAST: &str = "minecraft:entity_parrot_imitate_ghast";
    pub const ENTITY_PARROT_IMITATE_GUARDIAN: &str = "minecraft:entity_parrot_imitate_guardian";
    pub const ENTITY_PARROT_IMITATE_HOGLIN: &str = "minecraft:entity_parrot_imitate_hoglin";
    pub const ENTITY_PARROT_IMITATE_HUSK: &str = "minecraft:entity_parrot_imitate_husk";
    pub const ENTITY_PARROT_IMITATE_ILLUSIONER: &str = "minecraft:entity_parrot_imitate_illusioner";
    pub const ENTITY_PARROT_IMITATE_MAGMA_CUBE: &str = "minecraft:entity_parrot_imitate_magma_cube";
    pub const ENTITY_PARROT_IMITATE_PARCHED: &str = "minecraft:entity_parrot_imitate_parched";
    pub const ENTITY_PARROT_IMITATE_PHANTOM: &str = "minecraft:entity_parrot_imitate_phantom";
    pub const ENTITY_PARROT_IMITATE_PIGLIN: &str = "minecraft:entity_parrot_imitate_piglin";
    pub const ENTITY_PARROT_IMITATE_PIGLIN_BRUTE: &str = "minecraft:entity_parrot_imitate_piglin_brute";
    pub const ENTITY_PARROT_IMITATE_PILLAGER: &str = "minecraft:entity_parrot_imitate_pillager";
    pub const ENTITY_PARROT_IMITATE_RAVAGER: &str = "minecraft:entity_parrot_imitate_ravager";
    pub const ENTITY_PARROT_IMITATE_SHULKER: &str = "minecraft:entity_parrot_imitate_shulker";
    pub const ENTITY_PARROT_IMITATE_SILVERFISH: &str = "minecraft:entity_parrot_imitate_silverfish";
    pub const ENTITY_PARROT_IMITATE_SKELETON: &str = "minecraft:entity_parrot_imitate_skeleton";
    pub const ENTITY_PARROT_IMITATE_SLIME: &str = "minecraft:entity_parrot_imitate_slime";
    pub const ENTITY_PARROT_IMITATE_SPIDER: &str = "minecraft:entity_parrot_imitate_spider";
    pub const ENTITY_PARROT_IMITATE_STRAY: &str = "minecraft:entity_parrot_imitate_stray";
    pub const ENTITY_PARROT_IMITATE_VEX: &str = "minecraft:entity_parrot_imitate_vex";
    pub const ENTITY_PARROT_IMITATE_VINDICATOR: &str = "minecraft:entity_parrot_imitate_vindicator";
    pub const ENTITY_PARROT_IMITATE_WARDEN: &str = "minecraft:entity_parrot_imitate_warden";
    pub const ENTITY_PARROT_IMITATE_WITCH: &str = "minecraft:entity_parrot_imitate_witch";
    pub const ENTITY_PARROT_IMITATE_WITHER: &str = "minecraft:entity_parrot_imitate_wither";
    pub const ENTITY_PARROT_IMITATE_WITHER_SKELETON: &str = "minecraft:entity_parrot_imitate_wither_skeleton";
    pub const ENTITY_PARROT_IMITATE_ZOGLIN: &str = "minecraft:entity_parrot_imitate_zoglin";
    pub const ENTITY_PARROT_IMITATE_ZOMBIE: &str = "minecraft:entity_parrot_imitate_zombie";
    pub const ENTITY_PARROT_IMITATE_ZOMBIE_HORSE: &str = "minecraft:entity_parrot_imitate_zombie_horse";
    pub const ENTITY_PARROT_IMITATE_ZOMBIE_NAUTILUS: &str = "minecraft:entity_parrot_imitate_zombie_nautilus";
    pub const ENTITY_PARROT_IMITATE_ZOMBIE_VILLAGER: &str = "minecraft:entity_parrot_imitate_zombie_villager";
    pub const ENTITY_PARROT_STEP: &str = "minecraft:entity_parrot_step";
    pub const ENTITY_PHANTOM_AMBIENT: &str = "minecraft:entity_phantom_ambient";
    pub const ENTITY_PHANTOM_BITE: &str = "minecraft:entity_phantom_bite";
    pub const ENTITY_PHANTOM_DEATH: &str = "minecraft:entity_phantom_death";
    pub const ENTITY_PHANTOM_FLAP: &str = "minecraft:entity_phantom_flap";
    pub const ENTITY_PHANTOM_HURT: &str = "minecraft:entity_phantom_hurt";
    pub const ENTITY_PHANTOM_SWOOP: &str = "minecraft:entity_phantom_swoop";
    pub const ENTITY_PIG_AMBIENT: &str = "minecraft:entity_pig_ambient";
    pub const ENTITY_PIG_DEATH: &str = "minecraft:entity_pig_death";
    pub const ENTITY_PIG_EAT: &str = "minecraft:entity_pig_eat";
    pub const ENTITY_PIG_HURT: &str = "minecraft:entity_pig_hurt";
    pub const ENTITY_PIG_SADDLE: &str = "minecraft:entity_pig_saddle";
    pub const ENTITY_PIG_STEP: &str = "minecraft:entity_pig_step";
    pub const ENTITY_PIG_BIG_AMBIENT: &str = "minecraft:entity_pig_big_ambient";
    pub const ENTITY_PIG_BIG_DEATH: &str = "minecraft:entity_pig_big_death";
    pub const ENTITY_PIG_BIG_EAT: &str = "minecraft:entity_pig_big_eat";
    pub const ENTITY_PIG_BIG_HURT: &str = "minecraft:entity_pig_big_hurt";
    pub const ENTITY_PIG_MINI_AMBIENT: &str = "minecraft:entity_pig_mini_ambient";
    pub const ENTITY_PIG_MINI_DEATH: &str = "minecraft:entity_pig_mini_death";
    pub const ENTITY_PIG_MINI_EAT: &str = "minecraft:entity_pig_mini_eat";
    pub const ENTITY_PIG_MINI_HURT: &str = "minecraft:entity_pig_mini_hurt";
    pub const ENTITY_PIGLIN_ADMIRING_ITEM: &str = "minecraft:entity_piglin_admiring_item";
    pub const ENTITY_PIGLIN_AMBIENT: &str = "minecraft:entity_piglin_ambient";
    pub const ENTITY_PIGLIN_ANGRY: &str = "minecraft:entity_piglin_angry";
    pub const ENTITY_PIGLIN_CELEBRATE: &str = "minecraft:entity_piglin_celebrate";
    pub const ENTITY_PIGLIN_CONVERTED_TO_ZOMBIFIED: &str = "minecraft:entity_piglin_converted_to_zombified";
    pub const ENTITY_PIGLIN_DEATH: &str = "minecraft:entity_piglin_death";
    pub const ENTITY_PIGLIN_HURT: &str = "minecraft:entity_piglin_hurt";
    pub const ENTITY_PIGLIN_JEALOUS: &str = "minecraft:entity_piglin_jealous";
    pub const ENTITY_PIGLIN_RETREAT: &str = "minecraft:entity_piglin_retreat";
    pub const ENTITY_PIGLIN_STEP: &str = "minecraft:entity_piglin_step";
    pub const ENTITY_PIGLIN_BRUTE_AMBIENT: &str = "minecraft:entity_piglin_brute_ambient";
    pub const ENTITY_PIGLIN_BRUTE_ANGRY: &str = "minecraft:entity_piglin_brute_angry";
    pub const ENTITY_PIGLIN_BRUTE_CONVERTED_TO_ZOMBIFIED: &str = "minecraft:entity_piglin_brute_converted_to_zombified";
    pub const ENTITY_PIGLIN_BRUTE_DEATH: &str = "minecraft:entity_piglin_brute_death";
    pub const ENTITY_PIGLIN_BRUTE_HURT: &str = "minecraft:entity_piglin_brute_hurt";
    pub const ENTITY_PIGLIN_BRUTE_STEP: &str = "minecraft:entity_piglin_brute_step";
    pub const ENTITY_PILLAGER_AMBIENT: &str = "minecraft:entity_pillager_ambient";
    pub const ENTITY_PILLAGER_CELEBRATE: &str = "minecraft:entity_pillager_celebrate";
    pub const ENTITY_PILLAGER_DEATH: &str = "minecraft:entity_pillager_death";
    pub const ENTITY_PILLAGER_HURT: &str = "minecraft:entity_pillager_hurt";
    pub const ENTITY_PLAYER_ATTACK_CRIT: &str = "minecraft:entity_player_attack_crit";
    pub const ENTITY_PLAYER_ATTACK_KNOCKBACK: &str = "minecraft:entity_player_attack_knockback";
    pub const ENTITY_PLAYER_ATTACK_NODAMAGE: &str = "minecraft:entity_player_attack_nodamage";
    pub const ENTITY_PLAYER_ATTACK_STRONG: &str = "minecraft:entity_player_attack_strong";
    pub const ENTITY_PLAYER_ATTACK_SWEEP: &str = "minecraft:entity_player_attack_sweep";
    pub const ENTITY_PLAYER_ATTACK_WEAK: &str = "minecraft:entity_player_attack_weak";
    pub const ENTITY_PLAYER_BIG_FALL: &str = "minecraft:entity_player_big_fall";
    pub const ENTITY_PLAYER_BREATH: &str = "minecraft:entity_player_breath";
    pub const ENTITY_PLAYER_BURP: &str = "minecraft:entity_player_burp";
    pub const ENTITY_PLAYER_DEATH: &str = "minecraft:entity_player_death";
    pub const ENTITY_PLAYER_HURT: &str = "minecraft:entity_player_hurt";
    pub const ENTITY_PLAYER_HURT_DROWN: &str = "minecraft:entity_player_hurt_drown";
    pub const ENTITY_PLAYER_HURT_FREEZE: &str = "minecraft:entity_player_hurt_freeze";
    pub const ENTITY_PLAYER_HURT_ON_FIRE: &str = "minecraft:entity_player_hurt_on_fire";
    pub const ENTITY_PLAYER_HURT_SWEET_BERRY_BUSH: &str = "minecraft:entity_player_hurt_sweet_berry_bush";
    pub const ENTITY_PLAYER_LEVELUP: &str = "minecraft:entity_player_levelup";
    pub const ENTITY_PLAYER_SMALL_FALL: &str = "minecraft:entity_player_small_fall";
    pub const ENTITY_PLAYER_SPLASH: &str = "minecraft:entity_player_splash";
    pub const ENTITY_PLAYER_SPLASH_HIGH_SPEED: &str = "minecraft:entity_player_splash_high_speed";
    pub const ENTITY_PLAYER_SWIM: &str = "minecraft:entity_player_swim";
    pub const ENTITY_PLAYER_TELEPORT: &str = "minecraft:entity_player_teleport";
    pub const ENTITY_POLAR_BEAR_AMBIENT: &str = "minecraft:entity_polar_bear_ambient";
    pub const ENTITY_POLAR_BEAR_AMBIENT_BABY: &str = "minecraft:entity_polar_bear_ambient_baby";
    pub const ENTITY_POLAR_BEAR_DEATH: &str = "minecraft:entity_polar_bear_death";
    pub const ENTITY_POLAR_BEAR_HURT: &str = "minecraft:entity_polar_bear_hurt";
    pub const ENTITY_POLAR_BEAR_STEP: &str = "minecraft:entity_polar_bear_step";
    pub const ENTITY_POLAR_BEAR_WARNING: &str = "minecraft:entity_polar_bear_warning";
    pub const ENTITY_PUFFER_FISH_BLOW_OUT: &str = "minecraft:entity_puffer_fish_blow_out";
    pub const ENTITY_PUFFER_FISH_BLOW_UP: &str = "minecraft:entity_puffer_fish_blow_up";
    pub const ENTITY_PUFFER_FISH_DEATH: &str = "minecraft:entity_puffer_fish_death";
    pub const ENTITY_PUFFER_FISH_FLOP: &str = "minecraft:entity_puffer_fish_flop";
    pub const ENTITY_PUFFER_FISH_HURT: &str = "minecraft:entity_puffer_fish_hurt";
    pub const ENTITY_PUFFER_FISH_STING: &str = "minecraft:entity_puffer_fish_sting";
    pub const ENTITY_RABBIT_AMBIENT: &str = "minecraft:entity_rabbit_ambient";
    pub const ENTITY_RABBIT_ATTACK: &str = "minecraft:entity_rabbit_attack";
    pub const ENTITY_RABBIT_DEATH: &str = "minecraft:entity_rabbit_death";
    pub const ENTITY_RABBIT_HURT: &str = "minecraft:entity_rabbit_hurt";
    pub const ENTITY_RABBIT_JUMP: &str = "minecraft:entity_rabbit_jump";
    pub const ENTITY_RAVAGER_AMBIENT: &str = "minecraft:entity_ravager_ambient";
    pub const ENTITY_RAVAGER_ATTACK: &str = "minecraft:entity_ravager_attack";
    pub const ENTITY_RAVAGER_CELEBRATE: &str = "minecraft:entity_ravager_celebrate";
    pub const ENTITY_RAVAGER_DEATH: &str = "minecraft:entity_ravager_death";
    pub const ENTITY_RAVAGER_HURT: &str = "minecraft:entity_ravager_hurt";
    pub const ENTITY_RAVAGER_ROAR: &str = "minecraft:entity_ravager_roar";
    pub const ENTITY_RAVAGER_STEP: &str = "minecraft:entity_ravager_step";
    pub const ENTITY_RAVAGER_STUNNED: &str = "minecraft:entity_ravager_stunned";
    pub const ENTITY_SALMON_AMBIENT: &str = "minecraft:entity_salmon_ambient";
    pub const ENTITY_SALMON_DEATH: &str = "minecraft:entity_salmon_death";
    pub const ENTITY_SALMON_FLOP: &str = "minecraft:entity_salmon_flop";
    pub const ENTITY_SALMON_HURT: &str = "minecraft:entity_salmon_hurt";
    pub const ENTITY_SHEEP_AMBIENT: &str = "minecraft:entity_sheep_ambient";
    pub const ENTITY_SHEEP_DEATH: &str = "minecraft:entity_sheep_death";
    pub const ENTITY_SHEEP_HURT: &str = "minecraft:entity_sheep_hurt";
    pub const ENTITY_SHEEP_SHEAR: &str = "minecraft:entity_sheep_shear";
    pub const ENTITY_SHEEP_STEP: &str = "minecraft:entity_sheep_step";
    pub const ENTITY_SHULKER_AMBIENT: &str = "minecraft:entity_shulker_ambient";
    pub const ENTITY_SHULKER_CLOSE: &str = "minecraft:entity_shulker_close";
    pub const ENTITY_SHULKER_DEATH: &str = "minecraft:entity_shulker_death";
    pub const ENTITY_SHULKER_HURT: &str = "minecraft:entity_shulker_hurt";
    pub const ENTITY_SHULKER_HURT_CLOSED: &str = "minecraft:entity_shulker_hurt_closed";
    pub const ENTITY_SHULKER_OPEN: &str = "minecraft:entity_shulker_open";
    pub const ENTITY_SHULKER_SHOOT: &str = "minecraft:entity_shulker_shoot";
    pub const ENTITY_SHULKER_TELEPORT: &str = "minecraft:entity_shulker_teleport";
    pub const ENTITY_SHULKER_BULLET_HIT: &str = "minecraft:entity_shulker_bullet_hit";
    pub const ENTITY_SHULKER_BULLET_HURT: &str = "minecraft:entity_shulker_bullet_hurt";
    pub const ENTITY_SILVERFISH_AMBIENT: &str = "minecraft:entity_silverfish_ambient";
    pub const ENTITY_SILVERFISH_DEATH: &str = "minecraft:entity_silverfish_death";
    pub const ENTITY_SILVERFISH_HURT: &str = "minecraft:entity_silverfish_hurt";
    pub const ENTITY_SILVERFISH_STEP: &str = "minecraft:entity_silverfish_step";
    pub const ENTITY_SKELETON_AMBIENT: &str = "minecraft:entity_skeleton_ambient";
    pub const ENTITY_SKELETON_CONVERTED_TO_STRAY: &str = "minecraft:entity_skeleton_converted_to_stray";
    pub const ENTITY_SKELETON_DEATH: &str = "minecraft:entity_skeleton_death";
    pub const ENTITY_SKELETON_HURT: &str = "minecraft:entity_skeleton_hurt";
    pub const ENTITY_SKELETON_SHOOT: &str = "minecraft:entity_skeleton_shoot";
    pub const ENTITY_SKELETON_STEP: &str = "minecraft:entity_skeleton_step";
    pub const ENTITY_SKELETON_HORSE_AMBIENT: &str = "minecraft:entity_skeleton_horse_ambient";
    pub const ENTITY_SKELETON_HORSE_AMBIENT_WATER: &str = "minecraft:entity_skeleton_horse_ambient_water";
    pub const ENTITY_SKELETON_HORSE_DEATH: &str = "minecraft:entity_skeleton_horse_death";
    pub const ENTITY_SKELETON_HORSE_GALLOP_WATER: &str = "minecraft:entity_skeleton_horse_gallop_water";
    pub const ENTITY_SKELETON_HORSE_HURT: &str = "minecraft:entity_skeleton_horse_hurt";
    pub const ENTITY_SKELETON_HORSE_JUMP_WATER: &str = "minecraft:entity_skeleton_horse_jump_water";
    pub const ENTITY_SKELETON_HORSE_STEP_WATER: &str = "minecraft:entity_skeleton_horse_step_water";
    pub const ENTITY_SKELETON_HORSE_SWIM: &str = "minecraft:entity_skeleton_horse_swim";
    pub const ENTITY_SLIME_ATTACK: &str = "minecraft:entity_slime_attack";
    pub const ENTITY_SLIME_DEATH: &str = "minecraft:entity_slime_death";
    pub const ENTITY_SLIME_DEATH_SMALL: &str = "minecraft:entity_slime_death_small";
    pub const ENTITY_SLIME_HURT: &str = "minecraft:entity_slime_hurt";
    pub const ENTITY_SLIME_HURT_SMALL: &str = "minecraft:entity_slime_hurt_small";
    pub const ENTITY_SLIME_JUMP: &str = "minecraft:entity_slime_jump";
    pub const ENTITY_SLIME_JUMP_SMALL: &str = "minecraft:entity_slime_jump_small";
    pub const ENTITY_SLIME_SQUISH: &str = "minecraft:entity_slime_squish";
    pub const ENTITY_SLIME_SQUISH_SMALL: &str = "minecraft:entity_slime_squish_small";
    pub const ENTITY_SMALL_SULFUR_CUBE_DEATH: &str = "minecraft:entity_small_sulfur_cube_death";
    pub const ENTITY_SMALL_SULFUR_CUBE_EAT: &str = "minecraft:entity_small_sulfur_cube_eat";
    pub const ENTITY_SMALL_SULFUR_CUBE_HURT: &str = "minecraft:entity_small_sulfur_cube_hurt";
    pub const ENTITY_SMALL_SULFUR_CUBE_JUMP: &str = "minecraft:entity_small_sulfur_cube_jump";
    pub const ENTITY_SMALL_SULFUR_CUBE_SQUISH: &str = "minecraft:entity_small_sulfur_cube_squish";
    pub const ENTITY_SNIFFER_DEATH: &str = "minecraft:entity_sniffer_death";
    pub const ENTITY_SNIFFER_DIGGING: &str = "minecraft:entity_sniffer_digging";
    pub const ENTITY_SNIFFER_DIGGING_STOP: &str = "minecraft:entity_sniffer_digging_stop";
    pub const ENTITY_SNIFFER_DROP_SEED: &str = "minecraft:entity_sniffer_drop_seed";
    pub const ENTITY_SNIFFER_EAT: &str = "minecraft:entity_sniffer_eat";
    pub const ENTITY_SNIFFER_HAPPY: &str = "minecraft:entity_sniffer_happy";
    pub const ENTITY_SNIFFER_HURT: &str = "minecraft:entity_sniffer_hurt";
    pub const ENTITY_SNIFFER_IDLE: &str = "minecraft:entity_sniffer_idle";
    pub const ENTITY_SNIFFER_SCENTING: &str = "minecraft:entity_sniffer_scenting";
    pub const ENTITY_SNIFFER_SEARCHING: &str = "minecraft:entity_sniffer_searching";
    pub const ENTITY_SNIFFER_SNIFFING: &str = "minecraft:entity_sniffer_sniffing";
    pub const ENTITY_SNIFFER_STEP: &str = "minecraft:entity_sniffer_step";
    pub const ENTITY_SNOW_GOLEM_AMBIENT: &str = "minecraft:entity_snow_golem_ambient";
    pub const ENTITY_SNOW_GOLEM_DEATH: &str = "minecraft:entity_snow_golem_death";
    pub const ENTITY_SNOW_GOLEM_HURT: &str = "minecraft:entity_snow_golem_hurt";
    pub const ENTITY_SNOW_GOLEM_SHEAR: &str = "minecraft:entity_snow_golem_shear";
    pub const ENTITY_SNOW_GOLEM_SHOOT: &str = "minecraft:entity_snow_golem_shoot";
    pub const ENTITY_SNOWBALL_THROW: &str = "minecraft:entity_snowball_throw";
    pub const ENTITY_SPIDER_AMBIENT: &str = "minecraft:entity_spider_ambient";
    pub const ENTITY_SPIDER_DEATH: &str = "minecraft:entity_spider_death";
    pub const ENTITY_SPIDER_HURT: &str = "minecraft:entity_spider_hurt";
    pub const ENTITY_SPIDER_STEP: &str = "minecraft:entity_spider_step";
    pub const ENTITY_SPLASH_POTION_BREAK: &str = "minecraft:entity_splash_potion_break";
    pub const ENTITY_SPLASH_POTION_THROW: &str = "minecraft:entity_splash_potion_throw";
    pub const ENTITY_SQUID_AMBIENT: &str = "minecraft:entity_squid_ambient";
    pub const ENTITY_SQUID_DEATH: &str = "minecraft:entity_squid_death";
    pub const ENTITY_SQUID_HURT: &str = "minecraft:entity_squid_hurt";
    pub const ENTITY_SQUID_SQUIRT: &str = "minecraft:entity_squid_squirt";
    pub const ENTITY_STRAY_AMBIENT: &str = "minecraft:entity_stray_ambient";
    pub const ENTITY_STRAY_DEATH: &str = "minecraft:entity_stray_death";
    pub const ENTITY_STRAY_HURT: &str = "minecraft:entity_stray_hurt";
    pub const ENTITY_STRAY_STEP: &str = "minecraft:entity_stray_step";
    pub const ENTITY_STRIDER_AMBIENT: &str = "minecraft:entity_strider_ambient";
    pub const ENTITY_STRIDER_DEATH: &str = "minecraft:entity_strider_death";
    pub const ENTITY_STRIDER_EAT: &str = "minecraft:entity_strider_eat";
    pub const ENTITY_STRIDER_HAPPY: &str = "minecraft:entity_strider_happy";
    pub const ENTITY_STRIDER_HURT: &str = "minecraft:entity_strider_hurt";
    pub const ENTITY_STRIDER_RETREAT: &str = "minecraft:entity_strider_retreat";
    pub const ENTITY_STRIDER_SADDLE: &str = "minecraft:entity_strider_saddle";
    pub const ENTITY_STRIDER_STEP: &str = "minecraft:entity_strider_step";
    pub const ENTITY_STRIDER_STEP_LAVA: &str = "minecraft:entity_strider_step_lava";
    pub const ENTITY_SULFUR_CUBE_ABSORB: &str = "minecraft:entity_sulfur_cube_absorb";
    pub const ENTITY_SULFUR_CUBE_BOUNCE: &str = "minecraft:entity_sulfur_cube_bounce";
    pub const ENTITY_SULFUR_CUBE_BOUNCY_HIT: &str = "minecraft:entity_sulfur_cube_bouncy_hit";
    pub const ENTITY_SULFUR_CUBE_BOUNCY_PUSH: &str = "minecraft:entity_sulfur_cube_bouncy_push";
    pub const ENTITY_SULFUR_CUBE_DEATH: &str = "minecraft:entity_sulfur_cube_death";
    pub const ENTITY_SULFUR_CUBE_EJECT: &str = "minecraft:entity_sulfur_cube_eject";
    pub const ENTITY_SULFUR_CUBE_EXPLOSIVE_HIT: &str = "minecraft:entity_sulfur_cube_explosive_hit";
    pub const ENTITY_SULFUR_CUBE_EXPLOSIVE_PUSH: &str = "minecraft:entity_sulfur_cube_explosive_push";
    pub const ENTITY_SULFUR_CUBE_FAST_FLAT_HIT: &str = "minecraft:entity_sulfur_cube_fast_flat_hit";
    pub const ENTITY_SULFUR_CUBE_FAST_FLAT_PUSH: &str = "minecraft:entity_sulfur_cube_fast_flat_push";
    pub const ENTITY_SULFUR_CUBE_FAST_SLIDING_HIT: &str = "minecraft:entity_sulfur_cube_fast_sliding_hit";
    pub const ENTITY_SULFUR_CUBE_FAST_SLIDING_PUSH: &str = "minecraft:entity_sulfur_cube_fast_sliding_push";
    pub const ENTITY_SULFUR_CUBE_HIGH_RESISTANCE_HIT: &str = "minecraft:entity_sulfur_cube_high_resistance_hit";
    pub const ENTITY_SULFUR_CUBE_HIGH_RESISTANCE_PUSH: &str = "minecraft:entity_sulfur_cube_high_resistance_push";
    pub const ENTITY_SULFUR_CUBE_HOT_HIT: &str = "minecraft:entity_sulfur_cube_hot_hit";
    pub const ENTITY_SULFUR_CUBE_HOT_PUSH: &str = "minecraft:entity_sulfur_cube_hot_push";
    pub const ENTITY_SULFUR_CUBE_HURT: &str = "minecraft:entity_sulfur_cube_hurt";
    pub const ENTITY_SULFUR_CUBE_JUMP: &str = "minecraft:entity_sulfur_cube_jump";
    pub const ENTITY_SULFUR_CUBE_LIGHT_HIT: &str = "minecraft:entity_sulfur_cube_light_hit";
    pub const ENTITY_SULFUR_CUBE_LIGHT_PUSH: &str = "minecraft:entity_sulfur_cube_light_push";
    pub const ENTITY_SULFUR_CUBE_REGULAR_HIT: &str = "minecraft:entity_sulfur_cube_regular_hit";
    pub const ENTITY_SULFUR_CUBE_REGULAR_PUSH: &str = "minecraft:entity_sulfur_cube_regular_push";
    pub const ENTITY_SULFUR_CUBE_SLOW_BOUNCY_HIT: &str = "minecraft:entity_sulfur_cube_slow_bouncy_hit";
    pub const ENTITY_SULFUR_CUBE_SLOW_BOUNCY_PUSH: &str = "minecraft:entity_sulfur_cube_slow_bouncy_push";
    pub const ENTITY_SULFUR_CUBE_SLOW_FLAT_HIT: &str = "minecraft:entity_sulfur_cube_slow_flat_hit";
    pub const ENTITY_SULFUR_CUBE_SLOW_FLAT_PUSH: &str = "minecraft:entity_sulfur_cube_slow_flat_push";
    pub const ENTITY_SULFUR_CUBE_SLOW_SLIDING_HIT: &str = "minecraft:entity_sulfur_cube_slow_sliding_hit";
    pub const ENTITY_SULFUR_CUBE_SLOW_SLIDING_PUSH: &str = "minecraft:entity_sulfur_cube_slow_sliding_push";
    pub const ENTITY_SULFUR_CUBE_SQUISH: &str = "minecraft:entity_sulfur_cube_squish";
    pub const ENTITY_SULFUR_CUBE_STICKY_HIT: &str = "minecraft:entity_sulfur_cube_sticky_hit";
    pub const ENTITY_SULFUR_CUBE_STICKY_PUSH: &str = "minecraft:entity_sulfur_cube_sticky_push";
    pub const ENTITY_TADPOLE_DEATH: &str = "minecraft:entity_tadpole_death";
    pub const ENTITY_TADPOLE_FLOP: &str = "minecraft:entity_tadpole_flop";
    pub const ENTITY_TADPOLE_GROW_UP: &str = "minecraft:entity_tadpole_grow_up";
    pub const ENTITY_TADPOLE_HURT: &str = "minecraft:entity_tadpole_hurt";
    pub const ENTITY_TNT_PRIMED: &str = "minecraft:entity_tnt_primed";
    pub const ENTITY_TROPICAL_FISH_AMBIENT: &str = "minecraft:entity_tropical_fish_ambient";
    pub const ENTITY_TROPICAL_FISH_DEATH: &str = "minecraft:entity_tropical_fish_death";
    pub const ENTITY_TROPICAL_FISH_FLOP: &str = "minecraft:entity_tropical_fish_flop";
    pub const ENTITY_TROPICAL_FISH_HURT: &str = "minecraft:entity_tropical_fish_hurt";
    pub const ENTITY_TURTLE_AMBIENT_LAND: &str = "minecraft:entity_turtle_ambient_land";
    pub const ENTITY_TURTLE_DEATH: &str = "minecraft:entity_turtle_death";
    pub const ENTITY_TURTLE_DEATH_BABY: &str = "minecraft:entity_turtle_death_baby";
    pub const ENTITY_TURTLE_EGG_BREAK: &str = "minecraft:entity_turtle_egg_break";
    pub const ENTITY_TURTLE_EGG_CRACK: &str = "minecraft:entity_turtle_egg_crack";
    pub const ENTITY_TURTLE_EGG_HATCH: &str = "minecraft:entity_turtle_egg_hatch";
    pub const ENTITY_TURTLE_HURT: &str = "minecraft:entity_turtle_hurt";
    pub const ENTITY_TURTLE_HURT_BABY: &str = "minecraft:entity_turtle_hurt_baby";
    pub const ENTITY_TURTLE_LAY_EGG: &str = "minecraft:entity_turtle_lay_egg";
    pub const ENTITY_TURTLE_SHAMBLE: &str = "minecraft:entity_turtle_shamble";
    pub const ENTITY_TURTLE_SHAMBLE_BABY: &str = "minecraft:entity_turtle_shamble_baby";
    pub const ENTITY_TURTLE_SWIM: &str = "minecraft:entity_turtle_swim";
    pub const ENTITY_VEX_AMBIENT: &str = "minecraft:entity_vex_ambient";
    pub const ENTITY_VEX_CHARGE: &str = "minecraft:entity_vex_charge";
    pub const ENTITY_VEX_DEATH: &str = "minecraft:entity_vex_death";
    pub const ENTITY_VEX_HURT: &str = "minecraft:entity_vex_hurt";
    pub const ENTITY_VILLAGER_AMBIENT: &str = "minecraft:entity_villager_ambient";
    pub const ENTITY_VILLAGER_CELEBRATE: &str = "minecraft:entity_villager_celebrate";
    pub const ENTITY_VILLAGER_DEATH: &str = "minecraft:entity_villager_death";
    pub const ENTITY_VILLAGER_HURT: &str = "minecraft:entity_villager_hurt";
    pub const ENTITY_VILLAGER_NO: &str = "minecraft:entity_villager_no";
    pub const ENTITY_VILLAGER_TRADE: &str = "minecraft:entity_villager_trade";
    pub const ENTITY_VILLAGER_WORK_ARMORER: &str = "minecraft:entity_villager_work_armorer";
    pub const ENTITY_VILLAGER_WORK_BUTCHER: &str = "minecraft:entity_villager_work_butcher";
    pub const ENTITY_VILLAGER_WORK_CARTOGRAPHER: &str = "minecraft:entity_villager_work_cartographer";
    pub const ENTITY_VILLAGER_WORK_CLERIC: &str = "minecraft:entity_villager_work_cleric";
    pub const ENTITY_VILLAGER_WORK_FARMER: &str = "minecraft:entity_villager_work_farmer";
    pub const ENTITY_VILLAGER_WORK_FISHERMAN: &str = "minecraft:entity_villager_work_fisherman";
    pub const ENTITY_VILLAGER_WORK_FLETCHER: &str = "minecraft:entity_villager_work_fletcher";
    pub const ENTITY_VILLAGER_WORK_LEATHERWORKER: &str = "minecraft:entity_villager_work_leatherworker";
    pub const ENTITY_VILLAGER_WORK_LIBRARIAN: &str = "minecraft:entity_villager_work_librarian";
    pub const ENTITY_VILLAGER_WORK_MASON: &str = "minecraft:entity_villager_work_mason";
    pub const ENTITY_VILLAGER_WORK_SHEPHERD: &str = "minecraft:entity_villager_work_shepherd";
    pub const ENTITY_VILLAGER_WORK_TOOLSMITH: &str = "minecraft:entity_villager_work_toolsmith";
    pub const ENTITY_VILLAGER_WORK_WEAPONSMITH: &str = "minecraft:entity_villager_work_weaponsmith";
    pub const ENTITY_VILLAGER_YES: &str = "minecraft:entity_villager_yes";
    pub const ENTITY_VINDICATOR_AMBIENT: &str = "minecraft:entity_vindicator_ambient";
    pub const ENTITY_VINDICATOR_CELEBRATE: &str = "minecraft:entity_vindicator_celebrate";
    pub const ENTITY_VINDICATOR_DEATH: &str = "minecraft:entity_vindicator_death";
    pub const ENTITY_VINDICATOR_HURT: &str = "minecraft:entity_vindicator_hurt";
    pub const ENTITY_WANDERING_TRADER_AMBIENT: &str = "minecraft:entity_wandering_trader_ambient";
    pub const ENTITY_WANDERING_TRADER_DEATH: &str = "minecraft:entity_wandering_trader_death";
    pub const ENTITY_WANDERING_TRADER_DISAPPEARED: &str = "minecraft:entity_wandering_trader_disappeared";
    pub const ENTITY_WANDERING_TRADER_DRINK_MILK: &str = "minecraft:entity_wandering_trader_drink_milk";
    pub const ENTITY_WANDERING_TRADER_DRINK_POTION: &str = "minecraft:entity_wandering_trader_drink_potion";
    pub const ENTITY_WANDERING_TRADER_HURT: &str = "minecraft:entity_wandering_trader_hurt";
    pub const ENTITY_WANDERING_TRADER_NO: &str = "minecraft:entity_wandering_trader_no";
    pub const ENTITY_WANDERING_TRADER_REAPPEARED: &str = "minecraft:entity_wandering_trader_reappeared";
    pub const ENTITY_WANDERING_TRADER_TRADE: &str = "minecraft:entity_wandering_trader_trade";
    pub const ENTITY_WANDERING_TRADER_YES: &str = "minecraft:entity_wandering_trader_yes";
    pub const ENTITY_WARDEN_AGITATED: &str = "minecraft:entity_warden_agitated";
    pub const ENTITY_WARDEN_AMBIENT: &str = "minecraft:entity_warden_ambient";
    pub const ENTITY_WARDEN_ANGRY: &str = "minecraft:entity_warden_angry";
    pub const ENTITY_WARDEN_ATTACK_IMPACT: &str = "minecraft:entity_warden_attack_impact";
    pub const ENTITY_WARDEN_DEATH: &str = "minecraft:entity_warden_death";
    pub const ENTITY_WARDEN_DIG: &str = "minecraft:entity_warden_dig";
    pub const ENTITY_WARDEN_EMERGE: &str = "minecraft:entity_warden_emerge";
    pub const ENTITY_WARDEN_HEARTBEAT: &str = "minecraft:entity_warden_heartbeat";
    pub const ENTITY_WARDEN_HURT: &str = "minecraft:entity_warden_hurt";
    pub const ENTITY_WARDEN_LISTENING: &str = "minecraft:entity_warden_listening";
    pub const ENTITY_WARDEN_LISTENING_ANGRY: &str = "minecraft:entity_warden_listening_angry";
    pub const ENTITY_WARDEN_NEARBY_CLOSE: &str = "minecraft:entity_warden_nearby_close";
    pub const ENTITY_WARDEN_NEARBY_CLOSER: &str = "minecraft:entity_warden_nearby_closer";
    pub const ENTITY_WARDEN_NEARBY_CLOSEST: &str = "minecraft:entity_warden_nearby_closest";
    pub const ENTITY_WARDEN_ROAR: &str = "minecraft:entity_warden_roar";
    pub const ENTITY_WARDEN_SNIFF: &str = "minecraft:entity_warden_sniff";
    pub const ENTITY_WARDEN_SONIC_BOOM: &str = "minecraft:entity_warden_sonic_boom";
    pub const ENTITY_WARDEN_SONIC_CHARGE: &str = "minecraft:entity_warden_sonic_charge";
    pub const ENTITY_WARDEN_STEP: &str = "minecraft:entity_warden_step";
    pub const ENTITY_WARDEN_TENDRIL_CLICKS: &str = "minecraft:entity_warden_tendril_clicks";
    pub const ENTITY_WIND_CHARGE_THROW: &str = "minecraft:entity_wind_charge_throw";
    pub const ENTITY_WIND_CHARGE_WIND_BURST: &str = "minecraft:entity_wind_charge_wind_burst";
    pub const ENTITY_WITCH_AMBIENT: &str = "minecraft:entity_witch_ambient";
    pub const ENTITY_WITCH_CELEBRATE: &str = "minecraft:entity_witch_celebrate";
    pub const ENTITY_WITCH_DEATH: &str = "minecraft:entity_witch_death";
    pub const ENTITY_WITCH_DRINK: &str = "minecraft:entity_witch_drink";
    pub const ENTITY_WITCH_HURT: &str = "minecraft:entity_witch_hurt";
    pub const ENTITY_WITCH_THROW: &str = "minecraft:entity_witch_throw";
    pub const ENTITY_WITHER_AMBIENT: &str = "minecraft:entity_wither_ambient";
    pub const ENTITY_WITHER_BREAK_BLOCK: &str = "minecraft:entity_wither_break_block";
    pub const ENTITY_WITHER_DEATH: &str = "minecraft:entity_wither_death";
    pub const ENTITY_WITHER_HURT: &str = "minecraft:entity_wither_hurt";
    pub const ENTITY_WITHER_SHOOT: &str = "minecraft:entity_wither_shoot";
    pub const ENTITY_WITHER_SPAWN: &str = "minecraft:entity_wither_spawn";
    pub const ENTITY_WITHER_SKELETON_AMBIENT: &str = "minecraft:entity_wither_skeleton_ambient";
    pub const ENTITY_WITHER_SKELETON_DEATH: &str = "minecraft:entity_wither_skeleton_death";
    pub const ENTITY_WITHER_SKELETON_HURT: &str = "minecraft:entity_wither_skeleton_hurt";
    pub const ENTITY_WITHER_SKELETON_STEP: &str = "minecraft:entity_wither_skeleton_step";
    pub const ENTITY_WOLF_AMBIENT: &str = "minecraft:entity_wolf_ambient";
    pub const ENTITY_WOLF_DEATH: &str = "minecraft:entity_wolf_death";
    pub const ENTITY_WOLF_GROWL: &str = "minecraft:entity_wolf_growl";
    pub const ENTITY_WOLF_HURT: &str = "minecraft:entity_wolf_hurt";
    pub const ENTITY_WOLF_PANT: &str = "minecraft:entity_wolf_pant";
    pub const ENTITY_WOLF_SHAKE: &str = "minecraft:entity_wolf_shake";
    pub const ENTITY_WOLF_STEP: &str = "minecraft:entity_wolf_step";
    pub const ENTITY_WOLF_WHINE: &str = "minecraft:entity_wolf_whine";
    pub const ENTITY_WOLF_ANGRY_AMBIENT: &str = "minecraft:entity_wolf_angry_ambient";
    pub const ENTITY_WOLF_ANGRY_DEATH: &str = "minecraft:entity_wolf_angry_death";
    pub const ENTITY_WOLF_ANGRY_GROWL: &str = "minecraft:entity_wolf_angry_growl";
    pub const ENTITY_WOLF_ANGRY_HURT: &str = "minecraft:entity_wolf_angry_hurt";
    pub const ENTITY_WOLF_ANGRY_PANT: &str = "minecraft:entity_wolf_angry_pant";
    pub const ENTITY_WOLF_ANGRY_WHINE: &str = "minecraft:entity_wolf_angry_whine";
    pub const ENTITY_WOLF_BIG_AMBIENT: &str = "minecraft:entity_wolf_big_ambient";
    pub const ENTITY_WOLF_BIG_DEATH: &str = "minecraft:entity_wolf_big_death";
    pub const ENTITY_WOLF_BIG_GROWL: &str = "minecraft:entity_wolf_big_growl";
    pub const ENTITY_WOLF_BIG_HURT: &str = "minecraft:entity_wolf_big_hurt";
    pub const ENTITY_WOLF_BIG_PANT: &str = "minecraft:entity_wolf_big_pant";
    pub const ENTITY_WOLF_BIG_WHINE: &str = "minecraft:entity_wolf_big_whine";
    pub const ENTITY_WOLF_CUTE_AMBIENT: &str = "minecraft:entity_wolf_cute_ambient";
    pub const ENTITY_WOLF_CUTE_DEATH: &str = "minecraft:entity_wolf_cute_death";
    pub const ENTITY_WOLF_CUTE_GROWL: &str = "minecraft:entity_wolf_cute_growl";
    pub const ENTITY_WOLF_CUTE_HURT: &str = "minecraft:entity_wolf_cute_hurt";
    pub const ENTITY_WOLF_CUTE_PANT: &str = "minecraft:entity_wolf_cute_pant";
    pub const ENTITY_WOLF_CUTE_WHINE: &str = "minecraft:entity_wolf_cute_whine";
    pub const ENTITY_WOLF_GRUMPY_AMBIENT: &str = "minecraft:entity_wolf_grumpy_ambient";
    pub const ENTITY_WOLF_GRUMPY_DEATH: &str = "minecraft:entity_wolf_grumpy_death";
    pub const ENTITY_WOLF_GRUMPY_GROWL: &str = "minecraft:entity_wolf_grumpy_growl";
    pub const ENTITY_WOLF_GRUMPY_HURT: &str = "minecraft:entity_wolf_grumpy_hurt";
    pub const ENTITY_WOLF_GRUMPY_PANT: &str = "minecraft:entity_wolf_grumpy_pant";
    pub const ENTITY_WOLF_GRUMPY_WHINE: &str = "minecraft:entity_wolf_grumpy_whine";
    pub const ENTITY_WOLF_PUGLIN_AMBIENT: &str = "minecraft:entity_wolf_puglin_ambient";
    pub const ENTITY_WOLF_PUGLIN_DEATH: &str = "minecraft:entity_wolf_puglin_death";
    pub const ENTITY_WOLF_PUGLIN_GROWL: &str = "minecraft:entity_wolf_puglin_growl";
    pub const ENTITY_WOLF_PUGLIN_HURT: &str = "minecraft:entity_wolf_puglin_hurt";
    pub const ENTITY_WOLF_PUGLIN_PANT: &str = "minecraft:entity_wolf_puglin_pant";
    pub const ENTITY_WOLF_PUGLIN_WHINE: &str = "minecraft:entity_wolf_puglin_whine";
    pub const ENTITY_WOLF_SAD_AMBIENT: &str = "minecraft:entity_wolf_sad_ambient";
    pub const ENTITY_WOLF_SAD_DEATH: &str = "minecraft:entity_wolf_sad_death";
    pub const ENTITY_WOLF_SAD_GROWL: &str = "minecraft:entity_wolf_sad_growl";
    pub const ENTITY_WOLF_SAD_HURT: &str = "minecraft:entity_wolf_sad_hurt";
    pub const ENTITY_WOLF_SAD_PANT: &str = "minecraft:entity_wolf_sad_pant";
    pub const ENTITY_WOLF_SAD_WHINE: &str = "minecraft:entity_wolf_sad_whine";
    pub const ENTITY_ZOGLIN_AMBIENT: &str = "minecraft:entity_zoglin_ambient";
    pub const ENTITY_ZOGLIN_ANGRY: &str = "minecraft:entity_zoglin_angry";
    pub const ENTITY_ZOGLIN_ATTACK: &str = "minecraft:entity_zoglin_attack";
    pub const ENTITY_ZOGLIN_DEATH: &str = "minecraft:entity_zoglin_death";
    pub const ENTITY_ZOGLIN_HURT: &str = "minecraft:entity_zoglin_hurt";
    pub const ENTITY_ZOGLIN_STEP: &str = "minecraft:entity_zoglin_step";
    pub const ENTITY_ZOMBIE_AMBIENT: &str = "minecraft:entity_zombie_ambient";
    pub const ENTITY_ZOMBIE_ATTACK_IRON_DOOR: &str = "minecraft:entity_zombie_attack_iron_door";
    pub const ENTITY_ZOMBIE_ATTACK_WOODEN_DOOR: &str = "minecraft:entity_zombie_attack_wooden_door";
    pub const ENTITY_ZOMBIE_BREAK_WOODEN_DOOR: &str = "minecraft:entity_zombie_break_wooden_door";
    pub const ENTITY_ZOMBIE_CONVERTED_TO_DROWNED: &str = "minecraft:entity_zombie_converted_to_drowned";
    pub const ENTITY_ZOMBIE_DEATH: &str = "minecraft:entity_zombie_death";
    pub const ENTITY_ZOMBIE_DESTROY_EGG: &str = "minecraft:entity_zombie_destroy_egg";
    pub const ENTITY_ZOMBIE_HURT: &str = "minecraft:entity_zombie_hurt";
    pub const ENTITY_ZOMBIE_INFECT: &str = "minecraft:entity_zombie_infect";
    pub const ENTITY_ZOMBIE_STEP: &str = "minecraft:entity_zombie_step";
    pub const ENTITY_ZOMBIE_HORSE_AMBIENT: &str = "minecraft:entity_zombie_horse_ambient";
    pub const ENTITY_ZOMBIE_HORSE_ANGRY: &str = "minecraft:entity_zombie_horse_angry";
    pub const ENTITY_ZOMBIE_HORSE_DEATH: &str = "minecraft:entity_zombie_horse_death";
    pub const ENTITY_ZOMBIE_HORSE_EAT: &str = "minecraft:entity_zombie_horse_eat";
    pub const ENTITY_ZOMBIE_HORSE_HURT: &str = "minecraft:entity_zombie_horse_hurt";
    pub const ENTITY_ZOMBIE_NAUTILUS_AMBIENT: &str = "minecraft:entity_zombie_nautilus_ambient";
    pub const ENTITY_ZOMBIE_NAUTILUS_AMBIENT_LAND: &str = "minecraft:entity_zombie_nautilus_ambient_land";
    pub const ENTITY_ZOMBIE_NAUTILUS_DASH: &str = "minecraft:entity_zombie_nautilus_dash";
    pub const ENTITY_ZOMBIE_NAUTILUS_DASH_LAND: &str = "minecraft:entity_zombie_nautilus_dash_land";
    pub const ENTITY_ZOMBIE_NAUTILUS_DASH_READY: &str = "minecraft:entity_zombie_nautilus_dash_ready";
    pub const ENTITY_ZOMBIE_NAUTILUS_DASH_READY_LAND: &str = "minecraft:entity_zombie_nautilus_dash_ready_land";
    pub const ENTITY_ZOMBIE_NAUTILUS_DEATH: &str = "minecraft:entity_zombie_nautilus_death";
    pub const ENTITY_ZOMBIE_NAUTILUS_DEATH_LAND: &str = "minecraft:entity_zombie_nautilus_death_land";
    pub const ENTITY_ZOMBIE_NAUTILUS_EAT: &str = "minecraft:entity_zombie_nautilus_eat";
    pub const ENTITY_ZOMBIE_NAUTILUS_HURT: &str = "minecraft:entity_zombie_nautilus_hurt";
    pub const ENTITY_ZOMBIE_NAUTILUS_HURT_LAND: &str = "minecraft:entity_zombie_nautilus_hurt_land";
    pub const ENTITY_ZOMBIE_NAUTILUS_SWIM: &str = "minecraft:entity_zombie_nautilus_swim";
    pub const ENTITY_ZOMBIE_VILLAGER_AMBIENT: &str = "minecraft:entity_zombie_villager_ambient";
    pub const ENTITY_ZOMBIE_VILLAGER_CONVERTED: &str = "minecraft:entity_zombie_villager_converted";
    pub const ENTITY_ZOMBIE_VILLAGER_CURE: &str = "minecraft:entity_zombie_villager_cure";
    pub const ENTITY_ZOMBIE_VILLAGER_DEATH: &str = "minecraft:entity_zombie_villager_death";
    pub const ENTITY_ZOMBIE_VILLAGER_HURT: &str = "minecraft:entity_zombie_villager_hurt";
    pub const ENTITY_ZOMBIE_VILLAGER_STEP: &str = "minecraft:entity_zombie_villager_step";
    pub const ENTITY_ZOMBIFIED_PIGLIN_AMBIENT: &str = "minecraft:entity_zombified_piglin_ambient";
    pub const ENTITY_ZOMBIFIED_PIGLIN_ANGRY: &str = "minecraft:entity_zombified_piglin_angry";
    pub const ENTITY_ZOMBIFIED_PIGLIN_DEATH: &str = "minecraft:entity_zombified_piglin_death";
    pub const ENTITY_ZOMBIFIED_PIGLIN_HURT: &str = "minecraft:entity_zombified_piglin_hurt";
    pub const EVENT_MOB_EFFECT_BAD_OMEN: &str = "minecraft:event_mob_effect_bad_omen";
    pub const EVENT_MOB_EFFECT_RAID_OMEN: &str = "minecraft:event_mob_effect_raid_omen";
    pub const EVENT_MOB_EFFECT_TRIAL_OMEN: &str = "minecraft:event_mob_effect_trial_omen";
    pub const EVENT_RAID_HORN: &str = "minecraft:event_raid_horn";
    pub const INTENTIONALLY_EMPTY: &str = "minecraft:intentionally_empty";
    pub const ITEM_ARMOR_EQUIP_CHAIN: &str = "minecraft:item_armor_equip_chain";
    pub const ITEM_ARMOR_EQUIP_COPPER: &str = "minecraft:item_armor_equip_copper";
    pub const ITEM_ARMOR_EQUIP_DIAMOND: &str = "minecraft:item_armor_equip_diamond";
    pub const ITEM_ARMOR_EQUIP_ELYTRA: &str = "minecraft:item_armor_equip_elytra";
    pub const ITEM_ARMOR_EQUIP_GENERIC: &str = "minecraft:item_armor_equip_generic";
    pub const ITEM_ARMOR_EQUIP_GOLD: &str = "minecraft:item_armor_equip_gold";
    pub const ITEM_ARMOR_EQUIP_IRON: &str = "minecraft:item_armor_equip_iron";
    pub const ITEM_ARMOR_EQUIP_LEATHER: &str = "minecraft:item_armor_equip_leather";
    pub const ITEM_ARMOR_EQUIP_NAUTILUS: &str = "minecraft:item_armor_equip_nautilus";
    pub const ITEM_ARMOR_EQUIP_NETHERITE: &str = "minecraft:item_armor_equip_netherite";
    pub const ITEM_ARMOR_EQUIP_TURTLE: &str = "minecraft:item_armor_equip_turtle";
    pub const ITEM_ARMOR_EQUIP_WOLF: &str = "minecraft:item_armor_equip_wolf";
    pub const ITEM_ARMOR_UNEQUIP_NAUTILUS: &str = "minecraft:item_armor_unequip_nautilus";
    pub const ITEM_ARMOR_UNEQUIP_WOLF: &str = "minecraft:item_armor_unequip_wolf";
    pub const ITEM_AXE_SCRAPE: &str = "minecraft:item_axe_scrape";
    pub const ITEM_AXE_STRIP: &str = "minecraft:item_axe_strip";
    pub const ITEM_AXE_WAX_OFF: &str = "minecraft:item_axe_wax_off";
    pub const ITEM_BONE_MEAL_USE: &str = "minecraft:item_bone_meal_use";
    pub const ITEM_BOOK_PAGE_TURN: &str = "minecraft:item_book_page_turn";
    pub const ITEM_BOOK_PUT: &str = "minecraft:item_book_put";
    pub const ITEM_BOTTLE_EMPTY: &str = "minecraft:item_bottle_empty";
    pub const ITEM_BOTTLE_FILL: &str = "minecraft:item_bottle_fill";
    pub const ITEM_BOTTLE_FILL_DRAGONBREATH: &str = "minecraft:item_bottle_fill_dragonbreath";
    pub const ITEM_BRUSH_BRUSHING_GENERIC: &str = "minecraft:item_brush_brushing_generic";
    pub const ITEM_BRUSH_BRUSHING_GRAVEL: &str = "minecraft:item_brush_brushing_gravel";
    pub const ITEM_BRUSH_BRUSHING_GRAVEL_COMPLETE: &str = "minecraft:item_brush_brushing_gravel_complete";
    pub const ITEM_BRUSH_BRUSHING_SAND: &str = "minecraft:item_brush_brushing_sand";
    pub const ITEM_BRUSH_BRUSHING_SAND_COMPLETE: &str = "minecraft:item_brush_brushing_sand_complete";
    pub const ITEM_BUCKET_EMPTY: &str = "minecraft:item_bucket_empty";
    pub const ITEM_BUCKET_EMPTY_AXOLOTL: &str = "minecraft:item_bucket_empty_axolotl";
    pub const ITEM_BUCKET_EMPTY_FISH: &str = "minecraft:item_bucket_empty_fish";
    pub const ITEM_BUCKET_EMPTY_LAVA: &str = "minecraft:item_bucket_empty_lava";
    pub const ITEM_BUCKET_EMPTY_POWDER_SNOW: &str = "minecraft:item_bucket_empty_powder_snow";
    pub const ITEM_BUCKET_EMPTY_SULFUR_CUBE: &str = "minecraft:item_bucket_empty_sulfur_cube";
    pub const ITEM_BUCKET_EMPTY_TADPOLE: &str = "minecraft:item_bucket_empty_tadpole";
    pub const ITEM_BUCKET_FILL: &str = "minecraft:item_bucket_fill";
    pub const ITEM_BUCKET_FILL_AXOLOTL: &str = "minecraft:item_bucket_fill_axolotl";
    pub const ITEM_BUCKET_FILL_FISH: &str = "minecraft:item_bucket_fill_fish";
    pub const ITEM_BUCKET_FILL_LAVA: &str = "minecraft:item_bucket_fill_lava";
    pub const ITEM_BUCKET_FILL_POWDER_SNOW: &str = "minecraft:item_bucket_fill_powder_snow";
    pub const ITEM_BUCKET_FILL_SULFUR_CUBE: &str = "minecraft:item_bucket_fill_sulfur_cube";
    pub const ITEM_BUCKET_FILL_TADPOLE: &str = "minecraft:item_bucket_fill_tadpole";
    pub const ITEM_BUNDLE_DROP_CONTENTS: &str = "minecraft:item_bundle_drop_contents";
    pub const ITEM_BUNDLE_INSERT: &str = "minecraft:item_bundle_insert";
    pub const ITEM_BUNDLE_INSERT_FAIL: &str = "minecraft:item_bundle_insert_fail";
    pub const ITEM_BUNDLE_REMOVE_ONE: &str = "minecraft:item_bundle_remove_one";
    pub const ITEM_CHORUS_FRUIT_TELEPORT: &str = "minecraft:item_chorus_fruit_teleport";
    pub const ITEM_CROP_PLANT: &str = "minecraft:item_crop_plant";
    pub const ITEM_CROSSBOW_HIT: &str = "minecraft:item_crossbow_hit";
    pub const ITEM_CROSSBOW_LOADING_END: &str = "minecraft:item_crossbow_loading_end";
    pub const ITEM_CROSSBOW_LOADING_MIDDLE: &str = "minecraft:item_crossbow_loading_middle";
    pub const ITEM_CROSSBOW_LOADING_START: &str = "minecraft:item_crossbow_loading_start";
    pub const ITEM_CROSSBOW_QUICK_CHARGE_1: &str = "minecraft:item_crossbow_quick_charge_1";
    pub const ITEM_CROSSBOW_QUICK_CHARGE_2: &str = "minecraft:item_crossbow_quick_charge_2";
    pub const ITEM_CROSSBOW_QUICK_CHARGE_3: &str = "minecraft:item_crossbow_quick_charge_3";
    pub const ITEM_CROSSBOW_SHOOT: &str = "minecraft:item_crossbow_shoot";
    pub const ITEM_DYE_USE: &str = "minecraft:item_dye_use";
    pub const ITEM_ELYTRA_FLYING: &str = "minecraft:item_elytra_flying";
    pub const ITEM_FIRECHARGE_USE: &str = "minecraft:item_firecharge_use";
    pub const ITEM_FLINTANDSTEEL_USE: &str = "minecraft:item_flintandsteel_use";
    pub const ITEM_GLOW_INK_SAC_USE: &str = "minecraft:item_glow_ink_sac_use";
    pub const ITEM_GOAT_HORN_SOUND_0: &str = "minecraft:item_goat_horn_sound_0";
    pub const ITEM_GOAT_HORN_SOUND_1: &str = "minecraft:item_goat_horn_sound_1";
    pub const ITEM_GOAT_HORN_SOUND_2: &str = "minecraft:item_goat_horn_sound_2";
    pub const ITEM_GOAT_HORN_SOUND_3: &str = "minecraft:item_goat_horn_sound_3";
    pub const ITEM_GOAT_HORN_SOUND_4: &str = "minecraft:item_goat_horn_sound_4";
    pub const ITEM_GOAT_HORN_SOUND_5: &str = "minecraft:item_goat_horn_sound_5";
    pub const ITEM_GOAT_HORN_SOUND_6: &str = "minecraft:item_goat_horn_sound_6";
    pub const ITEM_GOAT_HORN_SOUND_7: &str = "minecraft:item_goat_horn_sound_7";
    pub const ITEM_GOLDEN_DANDELION_UNUSE: &str = "minecraft:item_golden_dandelion_unuse";
    pub const ITEM_GOLDEN_DANDELION_USE: &str = "minecraft:item_golden_dandelion_use";
    pub const ITEM_HOE_TILL: &str = "minecraft:item_hoe_till";
    pub const ITEM_HONEY_BOTTLE_DRINK: &str = "minecraft:item_honey_bottle_drink";
    pub const ITEM_HONEYCOMB_WAX_ON: &str = "minecraft:item_honeycomb_wax_on";
    pub const ITEM_HORSE_ARMOR_UNEQUIP: &str = "minecraft:item_horse_armor_unequip";
    pub const ITEM_INK_SAC_USE: &str = "minecraft:item_ink_sac_use";
    pub const ITEM_LEAD_BREAK: &str = "minecraft:item_lead_break";
    pub const ITEM_LEAD_TIED: &str = "minecraft:item_lead_tied";
    pub const ITEM_LEAD_UNTIED: &str = "minecraft:item_lead_untied";
    pub const ITEM_LLAMA_CARPET_UNEQUIP: &str = "minecraft:item_llama_carpet_unequip";
    pub const ITEM_LODESTONE_COMPASS_LOCK: &str = "minecraft:item_lodestone_compass_lock";
    pub const ITEM_MACE_SMASH_AIR: &str = "minecraft:item_mace_smash_air";
    pub const ITEM_MACE_SMASH_GROUND: &str = "minecraft:item_mace_smash_ground";
    pub const ITEM_MACE_SMASH_GROUND_HEAVY: &str = "minecraft:item_mace_smash_ground_heavy";
    pub const ITEM_NAUTILUS_SADDLE_EQUIP: &str = "minecraft:item_nautilus_saddle_equip";
    pub const ITEM_NAUTILUS_SADDLE_UNDERWATER_EQUIP: &str = "minecraft:item_nautilus_saddle_underwater_equip";
    pub const ITEM_NETHER_WART_PLANT: &str = "minecraft:item_nether_wart_plant";
    pub const ITEM_OMINOUS_BOTTLE_DISPOSE: &str = "minecraft:item_ominous_bottle_dispose";
    pub const ITEM_SADDLE_UNEQUIP: &str = "minecraft:item_saddle_unequip";
    pub const ITEM_SHEARS_SNIP: &str = "minecraft:item_shears_snip";
    pub const ITEM_SHIELD_BLOCK: &str = "minecraft:item_shield_block";
    pub const ITEM_SHIELD_BREAK: &str = "minecraft:item_shield_break";
    pub const ITEM_SHOVEL_FLATTEN: &str = "minecraft:item_shovel_flatten";
    pub const ITEM_SPEAR_ATTACK: &str = "minecraft:item_spear_attack";
    pub const ITEM_SPEAR_HIT: &str = "minecraft:item_spear_hit";
    pub const ITEM_SPEAR_LUNGE_1: &str = "minecraft:item_spear_lunge_1";
    pub const ITEM_SPEAR_LUNGE_2: &str = "minecraft:item_spear_lunge_2";
    pub const ITEM_SPEAR_LUNGE_3: &str = "minecraft:item_spear_lunge_3";
    pub const ITEM_SPEAR_USE: &str = "minecraft:item_spear_use";
    pub const ITEM_SPEAR_WOOD_ATTACK: &str = "minecraft:item_spear_wood_attack";
    pub const ITEM_SPEAR_WOOD_HIT: &str = "minecraft:item_spear_wood_hit";
    pub const ITEM_SPEAR_WOOD_USE: &str = "minecraft:item_spear_wood_use";
    pub const ITEM_SPYGLASS_STOP_USING: &str = "minecraft:item_spyglass_stop_using";
    pub const ITEM_SPYGLASS_USE: &str = "minecraft:item_spyglass_use";
    pub const ITEM_TOTEM_USE: &str = "minecraft:item_totem_use";
    pub const ITEM_TRIDENT_HIT: &str = "minecraft:item_trident_hit";
    pub const ITEM_TRIDENT_HIT_GROUND: &str = "minecraft:item_trident_hit_ground";
    pub const ITEM_TRIDENT_RETURN: &str = "minecraft:item_trident_return";
    pub const ITEM_TRIDENT_RIPTIDE_1: &str = "minecraft:item_trident_riptide_1";
    pub const ITEM_TRIDENT_RIPTIDE_2: &str = "minecraft:item_trident_riptide_2";
    pub const ITEM_TRIDENT_RIPTIDE_3: &str = "minecraft:item_trident_riptide_3";
    pub const ITEM_TRIDENT_THROW: &str = "minecraft:item_trident_throw";
    pub const ITEM_TRIDENT_THUNDER: &str = "minecraft:item_trident_thunder";
    pub const ITEM_WOLF_ARMOR_BREAK: &str = "minecraft:item_wolf_armor_break";
    pub const ITEM_WOLF_ARMOR_CRACK: &str = "minecraft:item_wolf_armor_crack";
    pub const ITEM_WOLF_ARMOR_DAMAGE: &str = "minecraft:item_wolf_armor_damage";
    pub const ITEM_WOLF_ARMOR_REPAIR: &str = "minecraft:item_wolf_armor_repair";
    pub const MUSIC_CREATIVE: &str = "minecraft:music_creative";
    pub const MUSIC_CREDITS: &str = "minecraft:music_credits";
    pub const MUSIC_DRAGON: &str = "minecraft:music_dragon";
    pub const MUSIC_END: &str = "minecraft:music_end";
    pub const MUSIC_GAME: &str = "minecraft:music_game";
    pub const MUSIC_MENU: &str = "minecraft:music_menu";
    pub const MUSIC_NETHER_BASALT_DELTAS: &str = "minecraft:music_nether_basalt_deltas";
    pub const MUSIC_NETHER_CRIMSON_FOREST: &str = "minecraft:music_nether_crimson_forest";
    pub const MUSIC_NETHER_NETHER_WASTES: &str = "minecraft:music_nether_nether_wastes";
    pub const MUSIC_NETHER_SOUL_SAND_VALLEY: &str = "minecraft:music_nether_soul_sand_valley";
    pub const MUSIC_NETHER_WARPED_FOREST: &str = "minecraft:music_nether_warped_forest";
    pub const MUSIC_OVERWORLD_BADLANDS: &str = "minecraft:music_overworld_badlands";
    pub const MUSIC_OVERWORLD_BAMBOO_JUNGLE: &str = "minecraft:music_overworld_bamboo_jungle";
    pub const MUSIC_OVERWORLD_CHERRY_GROVE: &str = "minecraft:music_overworld_cherry_grove";
    pub const MUSIC_OVERWORLD_DEEP_DARK: &str = "minecraft:music_overworld_deep_dark";
    pub const MUSIC_OVERWORLD_DESERT: &str = "minecraft:music_overworld_desert";
    pub const MUSIC_OVERWORLD_DRIPSTONE_CAVES: &str = "minecraft:music_overworld_dripstone_caves";
    pub const MUSIC_OVERWORLD_FLOWER_FOREST: &str = "minecraft:music_overworld_flower_forest";
    pub const MUSIC_OVERWORLD_FOREST: &str = "minecraft:music_overworld_forest";
    pub const MUSIC_OVERWORLD_FROZEN_PEAKS: &str = "minecraft:music_overworld_frozen_peaks";
    pub const MUSIC_OVERWORLD_GROVE: &str = "minecraft:music_overworld_grove";
    pub const MUSIC_OVERWORLD_JAGGED_PEAKS: &str = "minecraft:music_overworld_jagged_peaks";
    pub const MUSIC_OVERWORLD_JUNGLE: &str = "minecraft:music_overworld_jungle";
    pub const MUSIC_OVERWORLD_LUSH_CAVES: &str = "minecraft:music_overworld_lush_caves";
    pub const MUSIC_OVERWORLD_MEADOW: &str = "minecraft:music_overworld_meadow";
    pub const MUSIC_OVERWORLD_OLD_GROWTH_TAIGA: &str = "minecraft:music_overworld_old_growth_taiga";
    pub const MUSIC_OVERWORLD_SNOWY_SLOPES: &str = "minecraft:music_overworld_snowy_slopes";
    pub const MUSIC_OVERWORLD_SPARSE_JUNGLE: &str = "minecraft:music_overworld_sparse_jungle";
    pub const MUSIC_OVERWORLD_STONY_PEAKS: &str = "minecraft:music_overworld_stony_peaks";
    pub const MUSIC_OVERWORLD_SULFUR_CAVES: &str = "minecraft:music_overworld_sulfur_caves";
    pub const MUSIC_OVERWORLD_SWAMP: &str = "minecraft:music_overworld_swamp";
    pub const MUSIC_UNDER_WATER: &str = "minecraft:music_under_water";
    pub const MUSIC_DISC_11: &str = "minecraft:music_disc_11";
    pub const MUSIC_DISC_13: &str = "minecraft:music_disc_13";
    pub const MUSIC_DISC_5: &str = "minecraft:music_disc_5";
    pub const MUSIC_DISC_BLOCKS: &str = "minecraft:music_disc_blocks";
    pub const MUSIC_DISC_BOUNCE: &str = "minecraft:music_disc_bounce";
    pub const MUSIC_DISC_CAT: &str = "minecraft:music_disc_cat";
    pub const MUSIC_DISC_CHIRP: &str = "minecraft:music_disc_chirp";
    pub const MUSIC_DISC_CREATOR: &str = "minecraft:music_disc_creator";
    pub const MUSIC_DISC_CREATOR_MUSIC_BOX: &str = "minecraft:music_disc_creator_music_box";
    pub const MUSIC_DISC_FAR: &str = "minecraft:music_disc_far";
    pub const MUSIC_DISC_LAVA_CHICKEN: &str = "minecraft:music_disc_lava_chicken";
    pub const MUSIC_DISC_MALL: &str = "minecraft:music_disc_mall";
    pub const MUSIC_DISC_MELLOHI: &str = "minecraft:music_disc_mellohi";
    pub const MUSIC_DISC_OTHERSIDE: &str = "minecraft:music_disc_otherside";
    pub const MUSIC_DISC_PIGSTEP: &str = "minecraft:music_disc_pigstep";
    pub const MUSIC_DISC_PRECIPICE: &str = "minecraft:music_disc_precipice";
    pub const MUSIC_DISC_RELIC: &str = "minecraft:music_disc_relic";
    pub const MUSIC_DISC_STAL: &str = "minecraft:music_disc_stal";
    pub const MUSIC_DISC_STRAD: &str = "minecraft:music_disc_strad";
    pub const MUSIC_DISC_TEARS: &str = "minecraft:music_disc_tears";
    pub const MUSIC_DISC_WAIT: &str = "minecraft:music_disc_wait";
    pub const MUSIC_DISC_WARD: &str = "minecraft:music_disc_ward";
    pub const PARTICLE_SOUL_ESCAPE: &str = "minecraft:particle_soul_escape";
    pub const UI_BUTTON_CLICK: &str = "minecraft:ui_button_click";
    pub const UI_CARTOGRAPHY_TABLE_TAKE_RESULT: &str = "minecraft:ui_cartography_table_take_result";
    pub const UI_HUD_BUBBLE_POP: &str = "minecraft:ui_hud_bubble_pop";
    pub const UI_LOOM_SELECT_PATTERN: &str = "minecraft:ui_loom_select_pattern";
    pub const UI_LOOM_TAKE_RESULT: &str = "minecraft:ui_loom_take_result";
    pub const UI_STONECUTTER_SELECT_RECIPE: &str = "minecraft:ui_stonecutter_select_recipe";
    pub const UI_STONECUTTER_TAKE_RESULT: &str = "minecraft:ui_stonecutter_take_result";
    pub const UI_TOAST_CHALLENGE_COMPLETE: &str = "minecraft:ui_toast_challenge_complete";
    pub const UI_TOAST_IN: &str = "minecraft:ui_toast_in";
    pub const UI_TOAST_OUT: &str = "minecraft:ui_toast_out";
    pub const WEATHER_END_FLASH: &str = "minecraft:weather_end_flash";
    pub const WEATHER_RAIN: &str = "minecraft:weather_rain";
    pub const WEATHER_RAIN_ABOVE: &str = "minecraft:weather_rain_above";
}

/// `org.bukkit.SoundCategory`.
pub mod sound_category {
    /// bukkit: org/bukkit/SoundCategory.values
    pub fn values() -> &'static [&'static str] {
        &["MASTER", "MUSIC", "RECORDS", "WEATHER", "BLOCKS", "HOSTILE", "NEUTRAL", "PLAYERS", "AMBIENT", "VOICE", "UI"]
    }
    /// bukkit: org/bukkit/SoundCategory.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const MASTER: &str = "MASTER";
    pub const MUSIC: &str = "MUSIC";
    pub const RECORDS: &str = "RECORDS";
    pub const WEATHER: &str = "WEATHER";
    pub const BLOCKS: &str = "BLOCKS";
    pub const HOSTILE: &str = "HOSTILE";
    pub const NEUTRAL: &str = "NEUTRAL";
    pub const PLAYERS: &str = "PLAYERS";
    pub const AMBIENT: &str = "AMBIENT";
    pub const VOICE: &str = "VOICE";
    pub const UI: &str = "UI";
}

/// `org.bukkit.Statistic$Type`.
pub mod statistic_type {
    /// bukkit: org/bukkit/Statistic$Type.values
    pub fn values() -> &'static [&'static str] {
        &["UNTYPED", "ITEM", "BLOCK", "ENTITY"]
    }
    /// bukkit: org/bukkit/Statistic$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UNTYPED: &str = "UNTYPED";
    pub const ITEM: &str = "ITEM";
    pub const BLOCK: &str = "BLOCK";
    pub const ENTITY: &str = "ENTITY";
}

/// `org.bukkit.Statistic`.
pub mod statistic {
    /// bukkit: org/bukkit/Statistic.values
    pub fn values() -> &'static [&'static str] {
        &["ANIMALS_BRED", "AVIATE_ONE_CM", "BELL_RING", "BOAT_ONE_CM", "ARMOR_CLEANED", "BANNER_CLEANED", "CLEAN_SHULKER_BOX", "CLIMB_ONE_CM", "CROUCH_ONE_CM", "DAMAGE_ABSORBED", "DAMAGE_BLOCKED_BY_SHIELD", "DAMAGE_DEALT", "DAMAGE_DEALT_ABSORBED", "DAMAGE_DEALT_RESISTED", "DAMAGE_RESISTED", "DAMAGE_TAKEN", "DEATHS", "DROP_COUNT", "CAKE_SLICES_EATEN", "ITEM_ENCHANTED", "FALL_ONE_CM", "CAULDRON_FILLED", "FISH_CAUGHT", "FLY_ONE_CM", "HAPPY_GHAST_ONE_CM", "HORSE_ONE_CM", "DISPENSER_INSPECTED", "DROPPER_INSPECTED", "HOPPER_INSPECTED", "INTERACT_WITH_ANVIL", "BEACON_INTERACTION", "INTERACT_WITH_BLAST_FURNACE", "BREWINGSTAND_INTERACTION", "INTERACT_WITH_CAMPFIRE", "INTERACT_WITH_CARTOGRAPHY_TABLE", "CRAFTING_TABLE_INTERACTION", "FURNACE_INTERACTION", "INTERACT_WITH_GRINDSTONE", "INTERACT_WITH_LECTERN", "INTERACT_WITH_LOOM", "INTERACT_WITH_SMITHING_TABLE", "INTERACT_WITH_SMOKER", "INTERACT_WITH_STONECUTTER", "JUMP", "LEAVE_GAME", "MINECART_ONE_CM", "MOB_KILLS", "NAUTILUS_ONE_CM", "OPEN_BARREL", "CHEST_OPENED", "ENDERCHEST_OPENED", "SHULKER_BOX_OPENED", "PIG_ONE_CM", "NOTEBLOCK_PLAYED", "RECORD_PLAYED", "PLAY_ONE_MINUTE", "PLAYER_KILLS", "FLOWER_POTTED", "RAID_TRIGGER", "RAID_WIN", "SLEEP_IN_BED", "SLEEP_IN_STRAW_BED", "SNEAK_TIME", "SPRINT_ONE_CM", "STRIDER_ONE_CM", "SWIM_ONE_CM", "TALKED_TO_VILLAGER", "TARGET_HIT", "TIME_SINCE_DEATH", "TIME_SINCE_REST", "TOTAL_WORLD_TIME", "TRADED_WITH_VILLAGER", "TRAPPED_CHEST_TRIGGERED", "NOTEBLOCK_TUNED", "CAULDRON_USED", "WALK_ON_WATER_ONE_CM", "WALK_ONE_CM", "WALK_UNDER_WATER_ONE_CM", "BREAK_ITEM", "CRAFT_ITEM", "DROP", "KILL_ENTITY", "ENTITY_KILLED_BY", "MINE_BLOCK", "PICKUP", "USE_ITEM"]
    }
    /// bukkit: org/bukkit/Statistic.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ANIMALS_BRED: &str = "ANIMALS_BRED";
    pub const AVIATE_ONE_CM: &str = "AVIATE_ONE_CM";
    pub const BELL_RING: &str = "BELL_RING";
    pub const BOAT_ONE_CM: &str = "BOAT_ONE_CM";
    pub const ARMOR_CLEANED: &str = "ARMOR_CLEANED";
    pub const BANNER_CLEANED: &str = "BANNER_CLEANED";
    pub const CLEAN_SHULKER_BOX: &str = "CLEAN_SHULKER_BOX";
    pub const CLIMB_ONE_CM: &str = "CLIMB_ONE_CM";
    pub const CROUCH_ONE_CM: &str = "CROUCH_ONE_CM";
    pub const DAMAGE_ABSORBED: &str = "DAMAGE_ABSORBED";
    pub const DAMAGE_BLOCKED_BY_SHIELD: &str = "DAMAGE_BLOCKED_BY_SHIELD";
    pub const DAMAGE_DEALT: &str = "DAMAGE_DEALT";
    pub const DAMAGE_DEALT_ABSORBED: &str = "DAMAGE_DEALT_ABSORBED";
    pub const DAMAGE_DEALT_RESISTED: &str = "DAMAGE_DEALT_RESISTED";
    pub const DAMAGE_RESISTED: &str = "DAMAGE_RESISTED";
    pub const DAMAGE_TAKEN: &str = "DAMAGE_TAKEN";
    pub const DEATHS: &str = "DEATHS";
    pub const DROP_COUNT: &str = "DROP_COUNT";
    pub const CAKE_SLICES_EATEN: &str = "CAKE_SLICES_EATEN";
    pub const ITEM_ENCHANTED: &str = "ITEM_ENCHANTED";
    pub const FALL_ONE_CM: &str = "FALL_ONE_CM";
    pub const CAULDRON_FILLED: &str = "CAULDRON_FILLED";
    pub const FISH_CAUGHT: &str = "FISH_CAUGHT";
    pub const FLY_ONE_CM: &str = "FLY_ONE_CM";
    pub const HAPPY_GHAST_ONE_CM: &str = "HAPPY_GHAST_ONE_CM";
    pub const HORSE_ONE_CM: &str = "HORSE_ONE_CM";
    pub const DISPENSER_INSPECTED: &str = "DISPENSER_INSPECTED";
    pub const DROPPER_INSPECTED: &str = "DROPPER_INSPECTED";
    pub const HOPPER_INSPECTED: &str = "HOPPER_INSPECTED";
    pub const INTERACT_WITH_ANVIL: &str = "INTERACT_WITH_ANVIL";
    pub const BEACON_INTERACTION: &str = "BEACON_INTERACTION";
    pub const INTERACT_WITH_BLAST_FURNACE: &str = "INTERACT_WITH_BLAST_FURNACE";
    pub const BREWINGSTAND_INTERACTION: &str = "BREWINGSTAND_INTERACTION";
    pub const INTERACT_WITH_CAMPFIRE: &str = "INTERACT_WITH_CAMPFIRE";
    pub const INTERACT_WITH_CARTOGRAPHY_TABLE: &str = "INTERACT_WITH_CARTOGRAPHY_TABLE";
    pub const CRAFTING_TABLE_INTERACTION: &str = "CRAFTING_TABLE_INTERACTION";
    pub const FURNACE_INTERACTION: &str = "FURNACE_INTERACTION";
    pub const INTERACT_WITH_GRINDSTONE: &str = "INTERACT_WITH_GRINDSTONE";
    pub const INTERACT_WITH_LECTERN: &str = "INTERACT_WITH_LECTERN";
    pub const INTERACT_WITH_LOOM: &str = "INTERACT_WITH_LOOM";
    pub const INTERACT_WITH_SMITHING_TABLE: &str = "INTERACT_WITH_SMITHING_TABLE";
    pub const INTERACT_WITH_SMOKER: &str = "INTERACT_WITH_SMOKER";
    pub const INTERACT_WITH_STONECUTTER: &str = "INTERACT_WITH_STONECUTTER";
    pub const JUMP: &str = "JUMP";
    pub const LEAVE_GAME: &str = "LEAVE_GAME";
    pub const MINECART_ONE_CM: &str = "MINECART_ONE_CM";
    pub const MOB_KILLS: &str = "MOB_KILLS";
    pub const NAUTILUS_ONE_CM: &str = "NAUTILUS_ONE_CM";
    pub const OPEN_BARREL: &str = "OPEN_BARREL";
    pub const CHEST_OPENED: &str = "CHEST_OPENED";
    pub const ENDERCHEST_OPENED: &str = "ENDERCHEST_OPENED";
    pub const SHULKER_BOX_OPENED: &str = "SHULKER_BOX_OPENED";
    pub const PIG_ONE_CM: &str = "PIG_ONE_CM";
    pub const NOTEBLOCK_PLAYED: &str = "NOTEBLOCK_PLAYED";
    pub const RECORD_PLAYED: &str = "RECORD_PLAYED";
    pub const PLAY_ONE_MINUTE: &str = "PLAY_ONE_MINUTE";
    pub const PLAYER_KILLS: &str = "PLAYER_KILLS";
    pub const FLOWER_POTTED: &str = "FLOWER_POTTED";
    pub const RAID_TRIGGER: &str = "RAID_TRIGGER";
    pub const RAID_WIN: &str = "RAID_WIN";
    pub const SLEEP_IN_BED: &str = "SLEEP_IN_BED";
    pub const SLEEP_IN_STRAW_BED: &str = "SLEEP_IN_STRAW_BED";
    pub const SNEAK_TIME: &str = "SNEAK_TIME";
    pub const SPRINT_ONE_CM: &str = "SPRINT_ONE_CM";
    pub const STRIDER_ONE_CM: &str = "STRIDER_ONE_CM";
    pub const SWIM_ONE_CM: &str = "SWIM_ONE_CM";
    pub const TALKED_TO_VILLAGER: &str = "TALKED_TO_VILLAGER";
    pub const TARGET_HIT: &str = "TARGET_HIT";
    pub const TIME_SINCE_DEATH: &str = "TIME_SINCE_DEATH";
    pub const TIME_SINCE_REST: &str = "TIME_SINCE_REST";
    pub const TOTAL_WORLD_TIME: &str = "TOTAL_WORLD_TIME";
    pub const TRADED_WITH_VILLAGER: &str = "TRADED_WITH_VILLAGER";
    pub const TRAPPED_CHEST_TRIGGERED: &str = "TRAPPED_CHEST_TRIGGERED";
    pub const NOTEBLOCK_TUNED: &str = "NOTEBLOCK_TUNED";
    pub const CAULDRON_USED: &str = "CAULDRON_USED";
    pub const WALK_ON_WATER_ONE_CM: &str = "WALK_ON_WATER_ONE_CM";
    pub const WALK_ONE_CM: &str = "WALK_ONE_CM";
    pub const WALK_UNDER_WATER_ONE_CM: &str = "WALK_UNDER_WATER_ONE_CM";
    pub const BREAK_ITEM: &str = "BREAK_ITEM";
    pub const CRAFT_ITEM: &str = "CRAFT_ITEM";
    pub const DROP: &str = "DROP";
    pub const KILL_ENTITY: &str = "KILL_ENTITY";
    pub const ENTITY_KILLED_BY: &str = "ENTITY_KILLED_BY";
    pub const MINE_BLOCK: &str = "MINE_BLOCK";
    pub const PICKUP: &str = "PICKUP";
    pub const USE_ITEM: &str = "USE_ITEM";
}

/// `org.bukkit.TreeSpecies`.
pub mod tree_species {
    /// bukkit: org/bukkit/TreeSpecies.values
    pub fn values() -> &'static [&'static str] {
        &["GENERIC", "REDWOOD", "BIRCH", "JUNGLE", "ACACIA", "DARK_OAK"]
    }
    /// bukkit: org/bukkit/TreeSpecies.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const GENERIC: &str = "GENERIC";
    pub const REDWOOD: &str = "REDWOOD";
    pub const BIRCH: &str = "BIRCH";
    pub const JUNGLE: &str = "JUNGLE";
    pub const ACACIA: &str = "ACACIA";
    pub const DARK_OAK: &str = "DARK_OAK";
}

/// `org.bukkit.TreeType`.
pub mod tree_type {
    /// bukkit: org/bukkit/TreeType.values
    pub fn values() -> &'static [&'static str] {
        &["TREE", "BIG_TREE", "REDWOOD", "TALL_REDWOOD", "BIRCH", "JUNGLE", "SMALL_JUNGLE", "COCOA_TREE", "JUNGLE_BUSH", "RED_MUSHROOM", "BROWN_MUSHROOM", "SWAMP", "ACACIA", "DARK_OAK", "MEGA_REDWOOD", "MEGA_PINE", "TALL_BIRCH", "CHORUS_PLANT", "CRIMSON_FUNGUS", "WARPED_FUNGUS", "AZALEA", "MANGROVE", "TALL_MANGROVE", "CHERRY", "PALE_OAK", "PALE_OAK_CREAKING", "RED_POPLAR", "ORANGE_POPLAR", "YELLOW_POPLAR"]
    }
    /// bukkit: org/bukkit/TreeType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TREE: &str = "TREE";
    pub const BIG_TREE: &str = "BIG_TREE";
    pub const REDWOOD: &str = "REDWOOD";
    pub const TALL_REDWOOD: &str = "TALL_REDWOOD";
    pub const BIRCH: &str = "BIRCH";
    pub const JUNGLE: &str = "JUNGLE";
    pub const SMALL_JUNGLE: &str = "SMALL_JUNGLE";
    pub const COCOA_TREE: &str = "COCOA_TREE";
    pub const JUNGLE_BUSH: &str = "JUNGLE_BUSH";
    pub const RED_MUSHROOM: &str = "RED_MUSHROOM";
    pub const BROWN_MUSHROOM: &str = "BROWN_MUSHROOM";
    pub const SWAMP: &str = "SWAMP";
    pub const ACACIA: &str = "ACACIA";
    pub const DARK_OAK: &str = "DARK_OAK";
    pub const MEGA_REDWOOD: &str = "MEGA_REDWOOD";
    pub const MEGA_PINE: &str = "MEGA_PINE";
    pub const TALL_BIRCH: &str = "TALL_BIRCH";
    pub const CHORUS_PLANT: &str = "CHORUS_PLANT";
    pub const CRIMSON_FUNGUS: &str = "CRIMSON_FUNGUS";
    pub const WARPED_FUNGUS: &str = "WARPED_FUNGUS";
    pub const AZALEA: &str = "AZALEA";
    pub const MANGROVE: &str = "MANGROVE";
    pub const TALL_MANGROVE: &str = "TALL_MANGROVE";
    pub const CHERRY: &str = "CHERRY";
    pub const PALE_OAK: &str = "PALE_OAK";
    pub const PALE_OAK_CREAKING: &str = "PALE_OAK_CREAKING";
    pub const RED_POPLAR: &str = "RED_POPLAR";
    pub const ORANGE_POPLAR: &str = "ORANGE_POPLAR";
    pub const YELLOW_POPLAR: &str = "YELLOW_POPLAR";
}

/// `org.bukkit.Warning$WarningState`.
pub mod warning_warning_state {
    /// bukkit: org/bukkit/Warning$WarningState.values
    pub fn values() -> &'static [&'static str] {
        &["ON", "OFF", "DEFAULT"]
    }
    /// bukkit: org/bukkit/Warning$WarningState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ON: &str = "ON";
    pub const OFF: &str = "OFF";
    pub const DEFAULT: &str = "DEFAULT";
}

/// `org.bukkit.WeatherType`.
pub mod weather_type {
    /// bukkit: org/bukkit/WeatherType.values
    pub fn values() -> &'static [&'static str] {
        &["DOWNFALL", "CLEAR"]
    }
    /// bukkit: org/bukkit/WeatherType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DOWNFALL: &str = "DOWNFALL";
    pub const CLEAR: &str = "CLEAR";
}

/// `org.bukkit.World$Environment`.
pub mod world_environment {
    /// bukkit: org/bukkit/World$Environment.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "NETHER", "THE_END", "CUSTOM"]
    }
    /// bukkit: org/bukkit/World$Environment.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const NETHER: &str = "NETHER";
    pub const THE_END: &str = "THE_END";
    pub const CUSTOM: &str = "CUSTOM";
}

/// `org.bukkit.WorldType`.
pub mod world_type {
    /// bukkit: org/bukkit/WorldType.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "FLAT", "LARGE_BIOMES", "AMPLIFIED", "SINGLE_BIOME_SURFACE", "DEBUG_ALL_BLOCK_STATES"]
    }
    /// bukkit: org/bukkit/WorldType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const FLAT: &str = "FLAT";
    pub const LARGE_BIOMES: &str = "LARGE_BIOMES";
    pub const AMPLIFIED: &str = "AMPLIFIED";
    pub const SINGLE_BIOME_SURFACE: &str = "SINGLE_BIOME_SURFACE";
    pub const DEBUG_ALL_BLOCK_STATES: &str = "DEBUG_ALL_BLOCK_STATES";
}

/// `org.bukkit.advancement.AdvancementDisplayType`.
pub mod advancement_display_type {
    /// bukkit: org/bukkit/advancement/AdvancementDisplayType.values
    pub fn values() -> &'static [&'static str] {
        &["TASK", "CHALLENGE", "GOAL"]
    }
    /// bukkit: org/bukkit/advancement/AdvancementDisplayType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TASK: &str = "TASK";
    pub const CHALLENGE: &str = "CHALLENGE";
    pub const GOAL: &str = "GOAL";
}

/// `org.bukkit.attribute.Attribute$Sentiment`.
pub mod attribute_sentiment {
    /// bukkit: org/bukkit/attribute/Attribute$Sentiment.values
    pub fn values() -> &'static [&'static str] {
        &["POSITIVE", "NEUTRAL", "NEGATIVE"]
    }
    /// bukkit: org/bukkit/attribute/Attribute$Sentiment.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const POSITIVE: &str = "POSITIVE";
    pub const NEUTRAL: &str = "NEUTRAL";
    pub const NEGATIVE: &str = "NEGATIVE";
}

/// `org.bukkit.attribute.Attribute`.
pub mod attribute {
    /// bukkit: org/bukkit/attribute/Attribute.values
    pub fn values() -> &'static [&'static str] {
        &["MAX_HEALTH", "FOLLOW_RANGE", "KNOCKBACK_RESISTANCE", "MOVEMENT_SPEED", "FLYING_SPEED", "ATTACK_DAMAGE", "ATTACK_KNOCKBACK", "ATTACK_SPEED", "ARMOR", "ARMOR_TOUGHNESS", "FALL_DAMAGE_MULTIPLIER", "LUCK", "MAX_ABSORPTION", "SAFE_FALL_DISTANCE", "SCALE", "STEP_HEIGHT", "GRAVITY", "JUMP_STRENGTH", "BURNING_TIME", "CAMERA_DISTANCE", "EXPLOSION_KNOCKBACK_RESISTANCE", "MOVEMENT_EFFICIENCY", "OXYGEN_BONUS", "WATER_MOVEMENT_EFFICIENCY", "TEMPT_RANGE", "BLOCK_INTERACTION_RANGE", "ENTITY_INTERACTION_RANGE", "BLOCK_BREAK_SPEED", "MINING_EFFICIENCY", "SNEAKING_SPEED", "SUBMERGED_MINING_SPEED", "SWEEPING_DAMAGE_RATIO", "SPAWN_REINFORCEMENTS", "WAYPOINT_TRANSMIT_RANGE", "WAYPOINT_RECEIVE_RANGE", "AIR_DRAG_MODIFIER", "FRICTION_MODIFIER", "BOUNCINESS", "BELOW_NAME_DISTANCE", "NAME_TAG_DISTANCE"]
    }
    /// bukkit: org/bukkit/attribute/Attribute.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const MAX_HEALTH: &str = "minecraft:max_health";
    pub const FOLLOW_RANGE: &str = "minecraft:follow_range";
    pub const KNOCKBACK_RESISTANCE: &str = "minecraft:knockback_resistance";
    pub const MOVEMENT_SPEED: &str = "minecraft:movement_speed";
    pub const FLYING_SPEED: &str = "minecraft:flying_speed";
    pub const ATTACK_DAMAGE: &str = "minecraft:attack_damage";
    pub const ATTACK_KNOCKBACK: &str = "minecraft:attack_knockback";
    pub const ATTACK_SPEED: &str = "minecraft:attack_speed";
    pub const ARMOR: &str = "minecraft:armor";
    pub const ARMOR_TOUGHNESS: &str = "minecraft:armor_toughness";
    pub const FALL_DAMAGE_MULTIPLIER: &str = "minecraft:fall_damage_multiplier";
    pub const LUCK: &str = "minecraft:luck";
    pub const MAX_ABSORPTION: &str = "minecraft:max_absorption";
    pub const SAFE_FALL_DISTANCE: &str = "minecraft:safe_fall_distance";
    pub const SCALE: &str = "minecraft:scale";
    pub const STEP_HEIGHT: &str = "minecraft:step_height";
    pub const GRAVITY: &str = "minecraft:gravity";
    pub const JUMP_STRENGTH: &str = "minecraft:jump_strength";
    pub const BURNING_TIME: &str = "minecraft:burning_time";
    pub const CAMERA_DISTANCE: &str = "minecraft:camera_distance";
    pub const EXPLOSION_KNOCKBACK_RESISTANCE: &str = "minecraft:explosion_knockback_resistance";
    pub const MOVEMENT_EFFICIENCY: &str = "minecraft:movement_efficiency";
    pub const OXYGEN_BONUS: &str = "minecraft:oxygen_bonus";
    pub const WATER_MOVEMENT_EFFICIENCY: &str = "minecraft:water_movement_efficiency";
    pub const TEMPT_RANGE: &str = "minecraft:tempt_range";
    pub const BLOCK_INTERACTION_RANGE: &str = "minecraft:block_interaction_range";
    pub const ENTITY_INTERACTION_RANGE: &str = "minecraft:entity_interaction_range";
    pub const BLOCK_BREAK_SPEED: &str = "minecraft:block_break_speed";
    pub const MINING_EFFICIENCY: &str = "minecraft:mining_efficiency";
    pub const SNEAKING_SPEED: &str = "minecraft:sneaking_speed";
    pub const SUBMERGED_MINING_SPEED: &str = "minecraft:submerged_mining_speed";
    pub const SWEEPING_DAMAGE_RATIO: &str = "minecraft:sweeping_damage_ratio";
    pub const SPAWN_REINFORCEMENTS: &str = "minecraft:spawn_reinforcements";
    pub const WAYPOINT_TRANSMIT_RANGE: &str = "minecraft:waypoint_transmit_range";
    pub const WAYPOINT_RECEIVE_RANGE: &str = "minecraft:waypoint_receive_range";
    pub const AIR_DRAG_MODIFIER: &str = "minecraft:air_drag_modifier";
    pub const FRICTION_MODIFIER: &str = "minecraft:friction_modifier";
    pub const BOUNCINESS: &str = "minecraft:bounciness";
    pub const BELOW_NAME_DISTANCE: &str = "minecraft:below_name_distance";
    pub const NAME_TAG_DISTANCE: &str = "minecraft:name_tag_distance";
}

/// `org.bukkit.attribute.AttributeModifier$Operation`.
pub mod attribute_modifier_operation {
    /// bukkit: org/bukkit/attribute/AttributeModifier$Operation.values
    pub fn values() -> &'static [&'static str] {
        &["ADD_NUMBER", "ADD_SCALAR", "MULTIPLY_SCALAR_1"]
    }
    /// bukkit: org/bukkit/attribute/AttributeModifier$Operation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ADD_NUMBER: &str = "ADD_NUMBER";
    pub const ADD_SCALAR: &str = "ADD_SCALAR";
    pub const MULTIPLY_SCALAR_1: &str = "MULTIPLY_SCALAR_1";
}

/// `org.bukkit.block.Biome`.
pub mod biome {
    /// bukkit: org/bukkit/block/Biome.values
    pub fn values() -> &'static [&'static str] {
        &["BADLANDS", "BAMBOO_JUNGLE", "BASALT_DELTAS", "BEACH", "BIRCH_FOREST", "CHERRY_GROVE", "COLD_OCEAN", "CRIMSON_FOREST", "DAPPLED_FOREST", "DARK_FOREST", "DEEP_COLD_OCEAN", "DEEP_DARK", "DEEP_FROZEN_OCEAN", "DEEP_LUKEWARM_OCEAN", "DEEP_OCEAN", "DESERT", "DRIPSTONE_CAVES", "END_BARRENS", "END_HIGHLANDS", "END_MIDLANDS", "ERODED_BADLANDS", "FLOWER_FOREST", "FOREST", "FROZEN_OCEAN", "FROZEN_PEAKS", "FROZEN_RIVER", "GROVE", "ICE_SPIKES", "JAGGED_PEAKS", "JUNGLE", "LUKEWARM_OCEAN", "LUSH_CAVES", "MANGROVE_SWAMP", "MEADOW", "MUSHROOM_FIELDS", "NETHER_WASTES", "OCEAN", "OLD_GROWTH_BIRCH_FOREST", "OLD_GROWTH_PINE_TAIGA", "OLD_GROWTH_SPRUCE_TAIGA", "PALE_GARDEN", "PLAINS", "RIVER", "SAVANNA", "SAVANNA_PLATEAU", "SMALL_END_ISLANDS", "SNOWY_BEACH", "SNOWY_PLAINS", "SNOWY_SLOPES", "SNOWY_TAIGA", "SOUL_SAND_VALLEY", "SPARSE_JUNGLE", "STONY_PEAKS", "STONY_SHORE", "SULFUR_CAVES", "SUNFLOWER_PLAINS", "SWAMP", "TAIGA", "THE_END", "THE_VOID", "WARM_OCEAN", "WARPED_FOREST", "WINDSWEPT_FOREST", "WINDSWEPT_GRAVELLY_HILLS", "WINDSWEPT_HILLS", "WINDSWEPT_SAVANNA", "WOODED_BADLANDS", "CUSTOM"]
    }
    /// bukkit: org/bukkit/block/Biome.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BADLANDS: &str = "minecraft:badlands";
    pub const BAMBOO_JUNGLE: &str = "minecraft:bamboo_jungle";
    pub const BASALT_DELTAS: &str = "minecraft:basalt_deltas";
    pub const BEACH: &str = "minecraft:beach";
    pub const BIRCH_FOREST: &str = "minecraft:birch_forest";
    pub const CHERRY_GROVE: &str = "minecraft:cherry_grove";
    pub const COLD_OCEAN: &str = "minecraft:cold_ocean";
    pub const CRIMSON_FOREST: &str = "minecraft:crimson_forest";
    pub const DAPPLED_FOREST: &str = "minecraft:dappled_forest";
    pub const DARK_FOREST: &str = "minecraft:dark_forest";
    pub const DEEP_COLD_OCEAN: &str = "minecraft:deep_cold_ocean";
    pub const DEEP_DARK: &str = "minecraft:deep_dark";
    pub const DEEP_FROZEN_OCEAN: &str = "minecraft:deep_frozen_ocean";
    pub const DEEP_LUKEWARM_OCEAN: &str = "minecraft:deep_lukewarm_ocean";
    pub const DEEP_OCEAN: &str = "minecraft:deep_ocean";
    pub const DESERT: &str = "minecraft:desert";
    pub const DRIPSTONE_CAVES: &str = "minecraft:dripstone_caves";
    pub const END_BARRENS: &str = "minecraft:end_barrens";
    pub const END_HIGHLANDS: &str = "minecraft:end_highlands";
    pub const END_MIDLANDS: &str = "minecraft:end_midlands";
    pub const ERODED_BADLANDS: &str = "minecraft:eroded_badlands";
    pub const FLOWER_FOREST: &str = "minecraft:flower_forest";
    pub const FOREST: &str = "minecraft:forest";
    pub const FROZEN_OCEAN: &str = "minecraft:frozen_ocean";
    pub const FROZEN_PEAKS: &str = "minecraft:frozen_peaks";
    pub const FROZEN_RIVER: &str = "minecraft:frozen_river";
    pub const GROVE: &str = "minecraft:grove";
    pub const ICE_SPIKES: &str = "minecraft:ice_spikes";
    pub const JAGGED_PEAKS: &str = "minecraft:jagged_peaks";
    pub const JUNGLE: &str = "minecraft:jungle";
    pub const LUKEWARM_OCEAN: &str = "minecraft:lukewarm_ocean";
    pub const LUSH_CAVES: &str = "minecraft:lush_caves";
    pub const MANGROVE_SWAMP: &str = "minecraft:mangrove_swamp";
    pub const MEADOW: &str = "minecraft:meadow";
    pub const MUSHROOM_FIELDS: &str = "minecraft:mushroom_fields";
    pub const NETHER_WASTES: &str = "minecraft:nether_wastes";
    pub const OCEAN: &str = "minecraft:ocean";
    pub const OLD_GROWTH_BIRCH_FOREST: &str = "minecraft:old_growth_birch_forest";
    pub const OLD_GROWTH_PINE_TAIGA: &str = "minecraft:old_growth_pine_taiga";
    pub const OLD_GROWTH_SPRUCE_TAIGA: &str = "minecraft:old_growth_spruce_taiga";
    pub const PALE_GARDEN: &str = "minecraft:pale_garden";
    pub const PLAINS: &str = "minecraft:plains";
    pub const RIVER: &str = "minecraft:river";
    pub const SAVANNA: &str = "minecraft:savanna";
    pub const SAVANNA_PLATEAU: &str = "minecraft:savanna_plateau";
    pub const SMALL_END_ISLANDS: &str = "minecraft:small_end_islands";
    pub const SNOWY_BEACH: &str = "minecraft:snowy_beach";
    pub const SNOWY_PLAINS: &str = "minecraft:snowy_plains";
    pub const SNOWY_SLOPES: &str = "minecraft:snowy_slopes";
    pub const SNOWY_TAIGA: &str = "minecraft:snowy_taiga";
    pub const SOUL_SAND_VALLEY: &str = "minecraft:soul_sand_valley";
    pub const SPARSE_JUNGLE: &str = "minecraft:sparse_jungle";
    pub const STONY_PEAKS: &str = "minecraft:stony_peaks";
    pub const STONY_SHORE: &str = "minecraft:stony_shore";
    pub const SULFUR_CAVES: &str = "minecraft:sulfur_caves";
    pub const SUNFLOWER_PLAINS: &str = "minecraft:sunflower_plains";
    pub const SWAMP: &str = "minecraft:swamp";
    pub const TAIGA: &str = "minecraft:taiga";
    pub const THE_END: &str = "minecraft:the_end";
    pub const THE_VOID: &str = "minecraft:the_void";
    pub const WARM_OCEAN: &str = "minecraft:warm_ocean";
    pub const WARPED_FOREST: &str = "minecraft:warped_forest";
    pub const WINDSWEPT_FOREST: &str = "minecraft:windswept_forest";
    pub const WINDSWEPT_GRAVELLY_HILLS: &str = "minecraft:windswept_gravelly_hills";
    pub const WINDSWEPT_HILLS: &str = "minecraft:windswept_hills";
    pub const WINDSWEPT_SAVANNA: &str = "minecraft:windswept_savanna";
    pub const WOODED_BADLANDS: &str = "minecraft:wooded_badlands";
    pub const CUSTOM: &str = "minecraft:custom";
}

/// `org.bukkit.block.BlockFace`.
pub mod block_face {
    /// bukkit: org/bukkit/block/BlockFace.values
    pub fn values() -> &'static [&'static str] {
        &["NORTH", "EAST", "SOUTH", "WEST", "UP", "DOWN", "NORTH_EAST", "NORTH_WEST", "SOUTH_EAST", "SOUTH_WEST", "WEST_NORTH_WEST", "NORTH_NORTH_WEST", "NORTH_NORTH_EAST", "EAST_NORTH_EAST", "EAST_SOUTH_EAST", "SOUTH_SOUTH_EAST", "SOUTH_SOUTH_WEST", "WEST_SOUTH_WEST", "SELF"]
    }
    /// bukkit: org/bukkit/block/BlockFace.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORTH: &str = "NORTH";
    pub const EAST: &str = "EAST";
    pub const SOUTH: &str = "SOUTH";
    pub const WEST: &str = "WEST";
    pub const UP: &str = "UP";
    pub const DOWN: &str = "DOWN";
    pub const NORTH_EAST: &str = "NORTH_EAST";
    pub const NORTH_WEST: &str = "NORTH_WEST";
    pub const SOUTH_EAST: &str = "SOUTH_EAST";
    pub const SOUTH_WEST: &str = "SOUTH_WEST";
    pub const WEST_NORTH_WEST: &str = "WEST_NORTH_WEST";
    pub const NORTH_NORTH_WEST: &str = "NORTH_NORTH_WEST";
    pub const NORTH_NORTH_EAST: &str = "NORTH_NORTH_EAST";
    pub const EAST_NORTH_EAST: &str = "EAST_NORTH_EAST";
    pub const EAST_SOUTH_EAST: &str = "EAST_SOUTH_EAST";
    pub const SOUTH_SOUTH_EAST: &str = "SOUTH_SOUTH_EAST";
    pub const SOUTH_SOUTH_WEST: &str = "SOUTH_SOUTH_WEST";
    pub const WEST_SOUTH_WEST: &str = "WEST_SOUTH_WEST";
}

/// `org.bukkit.block.BlockSupport`.
pub mod block_support {
    /// bukkit: org/bukkit/block/BlockSupport.values
    pub fn values() -> &'static [&'static str] {
        &["FULL", "CENTER", "RIGID"]
    }
    /// bukkit: org/bukkit/block/BlockSupport.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FULL: &str = "FULL";
    pub const CENTER: &str = "CENTER";
    pub const RIGID: &str = "RIGID";
}

/// `org.bukkit.block.DecoratedPot$Side`.
pub mod decorated_pot_side {
    /// bukkit: org/bukkit/block/DecoratedPot$Side.values
    pub fn values() -> &'static [&'static str] {
        &["BACK", "LEFT", "RIGHT", "FRONT"]
    }
    /// bukkit: org/bukkit/block/DecoratedPot$Side.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BACK: &str = "BACK";
    pub const LEFT: &str = "LEFT";
    pub const RIGHT: &str = "RIGHT";
    pub const FRONT: &str = "FRONT";
}

/// `org.bukkit.block.DecoratedPot$WobbleStyle`.
pub mod decorated_pot_wobble_style {
    /// bukkit: org/bukkit/block/DecoratedPot$WobbleStyle.values
    pub fn values() -> &'static [&'static str] {
        &["POSITIVE", "NEGATIVE"]
    }
    /// bukkit: org/bukkit/block/DecoratedPot$WobbleStyle.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const POSITIVE: &str = "POSITIVE";
    pub const NEGATIVE: &str = "NEGATIVE";
}

/// `org.bukkit.block.Orientation`.
pub mod orientation {
    /// bukkit: org/bukkit/block/Orientation.values
    pub fn values() -> &'static [&'static str] {
        &["DOWN_EAST", "DOWN_NORTH", "DOWN_SOUTH", "DOWN_WEST", "UP_EAST", "UP_NORTH", "UP_SOUTH", "UP_WEST", "WEST_UP", "EAST_UP", "NORTH_UP", "SOUTH_UP"]
    }
    /// bukkit: org/bukkit/block/Orientation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DOWN_EAST: &str = "DOWN_EAST";
    pub const DOWN_NORTH: &str = "DOWN_NORTH";
    pub const DOWN_SOUTH: &str = "DOWN_SOUTH";
    pub const DOWN_WEST: &str = "DOWN_WEST";
    pub const UP_EAST: &str = "UP_EAST";
    pub const UP_NORTH: &str = "UP_NORTH";
    pub const UP_SOUTH: &str = "UP_SOUTH";
    pub const UP_WEST: &str = "UP_WEST";
    pub const WEST_UP: &str = "WEST_UP";
    pub const EAST_UP: &str = "EAST_UP";
    pub const NORTH_UP: &str = "NORTH_UP";
    pub const SOUTH_UP: &str = "SOUTH_UP";
}

/// `org.bukkit.block.PistonMoveReaction`.
pub mod piston_move_reaction {
    /// bukkit: org/bukkit/block/PistonMoveReaction.values
    pub fn values() -> &'static [&'static str] {
        &["MOVE", "PUSH_ONLY", "BREAK", "BLOCK", "IGNORE"]
    }
    /// bukkit: org/bukkit/block/PistonMoveReaction.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PUSH_ONLY: &str = "PUSH_ONLY";
    pub const BLOCK: &str = "BLOCK";
    pub const IGNORE: &str = "IGNORE";
}

/// `org.bukkit.block.banner.PatternType`.
pub mod pattern_type {
    /// bukkit: org/bukkit/block/banner/PatternType.values
    pub fn values() -> &'static [&'static str] {
        &["BASE", "BORDER", "BRICKS", "CIRCLE", "CREEPER", "CROSS", "CURLY_BORDER", "DIAGONAL_LEFT", "DIAGONAL_RIGHT", "DIAGONAL_UP_LEFT", "DIAGONAL_UP_RIGHT", "FLOW", "FLOWER", "GLOBE", "GRADIENT", "GRADIENT_UP", "GUSTER", "HALF_HORIZONTAL", "HALF_HORIZONTAL_BOTTOM", "HALF_VERTICAL", "HALF_VERTICAL_RIGHT", "MOJANG", "PIGLIN", "RHOMBUS", "SKULL", "SMALL_STRIPES", "SQUARE_BOTTOM_LEFT", "SQUARE_BOTTOM_RIGHT", "SQUARE_TOP_LEFT", "SQUARE_TOP_RIGHT", "STRAIGHT_CROSS", "STRIPE_BOTTOM", "STRIPE_CENTER", "STRIPE_DOWNLEFT", "STRIPE_DOWNRIGHT", "STRIPE_LEFT", "STRIPE_MIDDLE", "STRIPE_RIGHT", "STRIPE_TOP", "TRIANGLE_BOTTOM", "TRIANGLE_TOP", "TRIANGLES_BOTTOM", "TRIANGLES_TOP"]
    }
    /// bukkit: org/bukkit/block/banner/PatternType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BASE: &str = "minecraft:base";
    pub const BORDER: &str = "minecraft:border";
    pub const BRICKS: &str = "minecraft:bricks";
    pub const CIRCLE: &str = "minecraft:circle";
    pub const CREEPER: &str = "minecraft:creeper";
    pub const CROSS: &str = "minecraft:cross";
    pub const CURLY_BORDER: &str = "minecraft:curly_border";
    pub const DIAGONAL_LEFT: &str = "minecraft:diagonal_left";
    pub const DIAGONAL_RIGHT: &str = "minecraft:diagonal_right";
    pub const DIAGONAL_UP_LEFT: &str = "minecraft:diagonal_up_left";
    pub const DIAGONAL_UP_RIGHT: &str = "minecraft:diagonal_up_right";
    pub const FLOW: &str = "minecraft:flow";
    pub const FLOWER: &str = "minecraft:flower";
    pub const GLOBE: &str = "minecraft:globe";
    pub const GRADIENT: &str = "minecraft:gradient";
    pub const GRADIENT_UP: &str = "minecraft:gradient_up";
    pub const GUSTER: &str = "minecraft:guster";
    pub const HALF_HORIZONTAL: &str = "minecraft:half_horizontal";
    pub const HALF_HORIZONTAL_BOTTOM: &str = "minecraft:half_horizontal_bottom";
    pub const HALF_VERTICAL: &str = "minecraft:half_vertical";
    pub const HALF_VERTICAL_RIGHT: &str = "minecraft:half_vertical_right";
    pub const MOJANG: &str = "minecraft:mojang";
    pub const PIGLIN: &str = "minecraft:piglin";
    pub const RHOMBUS: &str = "minecraft:rhombus";
    pub const SKULL: &str = "minecraft:skull";
    pub const SMALL_STRIPES: &str = "minecraft:small_stripes";
    pub const SQUARE_BOTTOM_LEFT: &str = "minecraft:square_bottom_left";
    pub const SQUARE_BOTTOM_RIGHT: &str = "minecraft:square_bottom_right";
    pub const SQUARE_TOP_LEFT: &str = "minecraft:square_top_left";
    pub const SQUARE_TOP_RIGHT: &str = "minecraft:square_top_right";
    pub const STRAIGHT_CROSS: &str = "minecraft:straight_cross";
    pub const STRIPE_BOTTOM: &str = "minecraft:stripe_bottom";
    pub const STRIPE_CENTER: &str = "minecraft:stripe_center";
    pub const STRIPE_DOWNLEFT: &str = "minecraft:stripe_downleft";
    pub const STRIPE_DOWNRIGHT: &str = "minecraft:stripe_downright";
    pub const STRIPE_LEFT: &str = "minecraft:stripe_left";
    pub const STRIPE_MIDDLE: &str = "minecraft:stripe_middle";
    pub const STRIPE_RIGHT: &str = "minecraft:stripe_right";
    pub const STRIPE_TOP: &str = "minecraft:stripe_top";
    pub const TRIANGLE_BOTTOM: &str = "minecraft:triangle_bottom";
    pub const TRIANGLE_TOP: &str = "minecraft:triangle_top";
    pub const TRIANGLES_BOTTOM: &str = "minecraft:triangles_bottom";
    pub const TRIANGLES_TOP: &str = "minecraft:triangles_top";
}

/// `org.bukkit.block.data.Bisected$Half`.
pub mod bisected_half {
    /// bukkit: org/bukkit/block/data/Bisected$Half.values
    pub fn values() -> &'static [&'static str] {
        &["TOP", "BOTTOM"]
    }
    /// bukkit: org/bukkit/block/data/Bisected$Half.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TOP: &str = "TOP";
    pub const BOTTOM: &str = "BOTTOM";
}

/// `org.bukkit.block.data.FaceAttachable$AttachedFace`.
pub mod face_attachable_attached_face {
    /// bukkit: org/bukkit/block/data/FaceAttachable$AttachedFace.values
    pub fn values() -> &'static [&'static str] {
        &["FLOOR", "WALL", "CEILING"]
    }
    /// bukkit: org/bukkit/block/data/FaceAttachable$AttachedFace.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FLOOR: &str = "FLOOR";
    pub const WALL: &str = "WALL";
    pub const CEILING: &str = "CEILING";
}

/// `org.bukkit.block.data.Rail$Shape`.
pub mod rail_shape {
    /// bukkit: org/bukkit/block/data/Rail$Shape.values
    pub fn values() -> &'static [&'static str] {
        &["NORTH_SOUTH", "EAST_WEST", "ASCENDING_EAST", "ASCENDING_WEST", "ASCENDING_NORTH", "ASCENDING_SOUTH", "SOUTH_EAST", "SOUTH_WEST", "NORTH_WEST", "NORTH_EAST"]
    }
    /// bukkit: org/bukkit/block/data/Rail$Shape.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORTH_SOUTH: &str = "NORTH_SOUTH";
    pub const EAST_WEST: &str = "EAST_WEST";
    pub const ASCENDING_EAST: &str = "ASCENDING_EAST";
    pub const ASCENDING_WEST: &str = "ASCENDING_WEST";
    pub const ASCENDING_NORTH: &str = "ASCENDING_NORTH";
    pub const ASCENDING_SOUTH: &str = "ASCENDING_SOUTH";
    pub const SOUTH_EAST: &str = "SOUTH_EAST";
    pub const SOUTH_WEST: &str = "SOUTH_WEST";
    pub const NORTH_WEST: &str = "NORTH_WEST";
    pub const NORTH_EAST: &str = "NORTH_EAST";
}

/// `org.bukkit.block.data.SideChaining$ChainPart`.
pub mod side_chaining_chain_part {
    /// bukkit: org/bukkit/block/data/SideChaining$ChainPart.values
    pub fn values() -> &'static [&'static str] {
        &["UNCONNECTED", "RIGHT", "CENTER", "LEFT"]
    }
    /// bukkit: org/bukkit/block/data/SideChaining$ChainPart.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UNCONNECTED: &str = "UNCONNECTED";
    pub const RIGHT: &str = "RIGHT";
    pub const CENTER: &str = "CENTER";
    pub const LEFT: &str = "LEFT";
}

/// `org.bukkit.block.data.type.Bamboo$Leaves`.
pub mod bamboo_leaves {
    /// bukkit: org/bukkit/block/data/type/Bamboo$Leaves.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "SMALL", "LARGE"]
    }
    /// bukkit: org/bukkit/block/data/type/Bamboo$Leaves.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const SMALL: &str = "SMALL";
    pub const LARGE: &str = "LARGE";
}

/// `org.bukkit.block.data.type.Bed$Part`.
pub mod bed_part {
    /// bukkit: org/bukkit/block/data/type/Bed$Part.values
    pub fn values() -> &'static [&'static str] {
        &["HEAD", "FOOT"]
    }
    /// bukkit: org/bukkit/block/data/type/Bed$Part.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const HEAD: &str = "HEAD";
    pub const FOOT: &str = "FOOT";
}

/// `org.bukkit.block.data.type.Bell$Attachment`.
pub mod bell_attachment {
    /// bukkit: org/bukkit/block/data/type/Bell$Attachment.values
    pub fn values() -> &'static [&'static str] {
        &["FLOOR", "CEILING", "SINGLE_WALL", "DOUBLE_WALL"]
    }
    /// bukkit: org/bukkit/block/data/type/Bell$Attachment.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FLOOR: &str = "FLOOR";
    pub const CEILING: &str = "CEILING";
    pub const SINGLE_WALL: &str = "SINGLE_WALL";
    pub const DOUBLE_WALL: &str = "DOUBLE_WALL";
}

/// `org.bukkit.block.data.type.BigDripleaf$Tilt`.
pub mod big_dripleaf_tilt {
    /// bukkit: org/bukkit/block/data/type/BigDripleaf$Tilt.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "UNSTABLE", "PARTIAL", "FULL"]
    }
    /// bukkit: org/bukkit/block/data/type/BigDripleaf$Tilt.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const UNSTABLE: &str = "UNSTABLE";
    pub const PARTIAL: &str = "PARTIAL";
    pub const FULL: &str = "FULL";
}

/// `org.bukkit.block.data.type.Chest$Type`.
pub mod chest_type {
    /// bukkit: org/bukkit/block/data/type/Chest$Type.values
    pub fn values() -> &'static [&'static str] {
        &["SINGLE", "LEFT", "RIGHT"]
    }
    /// bukkit: org/bukkit/block/data/type/Chest$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SINGLE: &str = "SINGLE";
    pub const LEFT: &str = "LEFT";
    pub const RIGHT: &str = "RIGHT";
}

/// `org.bukkit.block.data.type.Comparator$Mode`.
pub mod comparator_mode {
    /// bukkit: org/bukkit/block/data/type/Comparator$Mode.values
    pub fn values() -> &'static [&'static str] {
        &["COMPARE", "SUBTRACT"]
    }
    /// bukkit: org/bukkit/block/data/type/Comparator$Mode.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMPARE: &str = "COMPARE";
    pub const SUBTRACT: &str = "SUBTRACT";
}

/// `org.bukkit.block.data.type.CopperGolemStatue$Pose`.
pub mod copper_golem_statue_pose {
    /// bukkit: org/bukkit/block/data/type/CopperGolemStatue$Pose.values
    pub fn values() -> &'static [&'static str] {
        &["STANDING", "SITTING", "RUNNING", "STAR"]
    }
    /// bukkit: org/bukkit/block/data/type/CopperGolemStatue$Pose.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const STANDING: &str = "STANDING";
    pub const SITTING: &str = "SITTING";
    pub const RUNNING: &str = "RUNNING";
    pub const STAR: &str = "STAR";
}

/// `org.bukkit.block.data.type.Crafter$Orientation`.
pub mod crafter_orientation {
    /// bukkit: org/bukkit/block/data/type/Crafter$Orientation.values
    pub fn values() -> &'static [&'static str] {
        &["DOWN_EAST", "DOWN_NORTH", "DOWN_SOUTH", "DOWN_WEST", "UP_EAST", "UP_NORTH", "UP_SOUTH", "UP_WEST", "WEST_UP", "EAST_UP", "NORTH_UP", "SOUTH_UP"]
    }
    /// bukkit: org/bukkit/block/data/type/Crafter$Orientation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DOWN_EAST: &str = "DOWN_EAST";
    pub const DOWN_NORTH: &str = "DOWN_NORTH";
    pub const DOWN_SOUTH: &str = "DOWN_SOUTH";
    pub const DOWN_WEST: &str = "DOWN_WEST";
    pub const UP_EAST: &str = "UP_EAST";
    pub const UP_NORTH: &str = "UP_NORTH";
    pub const UP_SOUTH: &str = "UP_SOUTH";
    pub const UP_WEST: &str = "UP_WEST";
    pub const WEST_UP: &str = "WEST_UP";
    pub const EAST_UP: &str = "EAST_UP";
    pub const NORTH_UP: &str = "NORTH_UP";
    pub const SOUTH_UP: &str = "SOUTH_UP";
}

/// `org.bukkit.block.data.type.CreakingHeart$State`.
pub mod creaking_heart_state {
    /// bukkit: org/bukkit/block/data/type/CreakingHeart$State.values
    pub fn values() -> &'static [&'static str] {
        &["UPROOTED", "DORMANT", "AWAKE"]
    }
    /// bukkit: org/bukkit/block/data/type/CreakingHeart$State.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UPROOTED: &str = "UPROOTED";
    pub const DORMANT: &str = "DORMANT";
    pub const AWAKE: &str = "AWAKE";
}

/// `org.bukkit.block.data.type.Door$Hinge`.
pub mod door_hinge {
    /// bukkit: org/bukkit/block/data/type/Door$Hinge.values
    pub fn values() -> &'static [&'static str] {
        &["LEFT", "RIGHT"]
    }
    /// bukkit: org/bukkit/block/data/type/Door$Hinge.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LEFT: &str = "LEFT";
    pub const RIGHT: &str = "RIGHT";
}

/// `org.bukkit.block.data.type.Jigsaw$Orientation`.
pub mod jigsaw_orientation {
    /// bukkit: org/bukkit/block/data/type/Jigsaw$Orientation.values
    pub fn values() -> &'static [&'static str] {
        &["DOWN_EAST", "DOWN_NORTH", "DOWN_SOUTH", "DOWN_WEST", "UP_EAST", "UP_NORTH", "UP_SOUTH", "UP_WEST", "WEST_UP", "EAST_UP", "NORTH_UP", "SOUTH_UP"]
    }
    /// bukkit: org/bukkit/block/data/type/Jigsaw$Orientation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DOWN_EAST: &str = "DOWN_EAST";
    pub const DOWN_NORTH: &str = "DOWN_NORTH";
    pub const DOWN_SOUTH: &str = "DOWN_SOUTH";
    pub const DOWN_WEST: &str = "DOWN_WEST";
    pub const UP_EAST: &str = "UP_EAST";
    pub const UP_NORTH: &str = "UP_NORTH";
    pub const UP_SOUTH: &str = "UP_SOUTH";
    pub const UP_WEST: &str = "UP_WEST";
    pub const WEST_UP: &str = "WEST_UP";
    pub const EAST_UP: &str = "EAST_UP";
    pub const NORTH_UP: &str = "NORTH_UP";
    pub const SOUTH_UP: &str = "SOUTH_UP";
}

/// `org.bukkit.block.data.type.MossyCarpet$Height`.
pub mod mossy_carpet_height {
    /// bukkit: org/bukkit/block/data/type/MossyCarpet$Height.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "LOW", "TALL"]
    }
    /// bukkit: org/bukkit/block/data/type/MossyCarpet$Height.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const LOW: &str = "LOW";
    pub const TALL: &str = "TALL";
}

/// `org.bukkit.block.data.type.PotentSulfur$State`.
pub mod potent_sulfur_state {
    /// bukkit: org/bukkit/block/data/type/PotentSulfur$State.values
    pub fn values() -> &'static [&'static str] {
        &["DRY", "WET", "DORMANT", "ERUPTING", "CONTINUOUS"]
    }
    /// bukkit: org/bukkit/block/data/type/PotentSulfur$State.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DRY: &str = "DRY";
    pub const WET: &str = "WET";
    pub const DORMANT: &str = "DORMANT";
    pub const ERUPTING: &str = "ERUPTING";
    pub const CONTINUOUS: &str = "CONTINUOUS";
}

/// `org.bukkit.block.data.type.RedstoneWire$Connection`.
pub mod redstone_wire_connection {
    /// bukkit: org/bukkit/block/data/type/RedstoneWire$Connection.values
    pub fn values() -> &'static [&'static str] {
        &["UP", "SIDE", "NONE"]
    }
    /// bukkit: org/bukkit/block/data/type/RedstoneWire$Connection.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UP: &str = "UP";
    pub const SIDE: &str = "SIDE";
    pub const NONE: &str = "NONE";
}

/// `org.bukkit.block.data.type.SculkSensor$Phase`.
pub mod sculk_sensor_phase {
    /// bukkit: org/bukkit/block/data/type/SculkSensor$Phase.values
    pub fn values() -> &'static [&'static str] {
        &["INACTIVE", "ACTIVE", "COOLDOWN"]
    }
    /// bukkit: org/bukkit/block/data/type/SculkSensor$Phase.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const INACTIVE: &str = "INACTIVE";
    pub const ACTIVE: &str = "ACTIVE";
    pub const COOLDOWN: &str = "COOLDOWN";
}

/// `org.bukkit.block.data.type.Slab$Type`.
pub mod slab_type {
    /// bukkit: org/bukkit/block/data/type/Slab$Type.values
    pub fn values() -> &'static [&'static str] {
        &["TOP", "BOTTOM", "DOUBLE"]
    }
    /// bukkit: org/bukkit/block/data/type/Slab$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TOP: &str = "TOP";
    pub const BOTTOM: &str = "BOTTOM";
    pub const DOUBLE: &str = "DOUBLE";
}

/// `org.bukkit.block.data.type.Speleothem$Thickness`.
pub mod speleothem_thickness {
    /// bukkit: org/bukkit/block/data/type/Speleothem$Thickness.values
    pub fn values() -> &'static [&'static str] {
        &["TIP_MERGE", "TIP", "FRUSTUM", "MIDDLE", "BASE"]
    }
    /// bukkit: org/bukkit/block/data/type/Speleothem$Thickness.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TIP_MERGE: &str = "TIP_MERGE";
    pub const TIP: &str = "TIP";
    pub const FRUSTUM: &str = "FRUSTUM";
    pub const MIDDLE: &str = "MIDDLE";
    pub const BASE: &str = "BASE";
}

/// `org.bukkit.block.data.type.Stairs$Shape`.
pub mod stairs_shape {
    /// bukkit: org/bukkit/block/data/type/Stairs$Shape.values
    pub fn values() -> &'static [&'static str] {
        &["STRAIGHT", "INNER_LEFT", "INNER_RIGHT", "OUTER_LEFT", "OUTER_RIGHT"]
    }
    /// bukkit: org/bukkit/block/data/type/Stairs$Shape.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const STRAIGHT: &str = "STRAIGHT";
    pub const INNER_LEFT: &str = "INNER_LEFT";
    pub const INNER_RIGHT: &str = "INNER_RIGHT";
    pub const OUTER_LEFT: &str = "OUTER_LEFT";
    pub const OUTER_RIGHT: &str = "OUTER_RIGHT";
}

/// `org.bukkit.block.data.type.StructureBlock$Mode`.
pub mod structure_block_mode {
    /// bukkit: org/bukkit/block/data/type/StructureBlock$Mode.values
    pub fn values() -> &'static [&'static str] {
        &["SAVE", "LOAD", "CORNER", "DATA"]
    }
    /// bukkit: org/bukkit/block/data/type/StructureBlock$Mode.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SAVE: &str = "SAVE";
    pub const LOAD: &str = "LOAD";
    pub const CORNER: &str = "CORNER";
    pub const DATA: &str = "DATA";
}

/// `org.bukkit.block.data.type.Switch$Face`.
pub mod switch_face {
    /// bukkit: org/bukkit/block/data/type/Switch$Face.values
    pub fn values() -> &'static [&'static str] {
        &["FLOOR", "WALL", "CEILING"]
    }
    /// bukkit: org/bukkit/block/data/type/Switch$Face.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FLOOR: &str = "FLOOR";
    pub const WALL: &str = "WALL";
    pub const CEILING: &str = "CEILING";
}

/// `org.bukkit.block.data.type.TechnicalPiston$Type`.
pub mod technical_piston_type {
    /// bukkit: org/bukkit/block/data/type/TechnicalPiston$Type.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "STICKY"]
    }
    /// bukkit: org/bukkit/block/data/type/TechnicalPiston$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const STICKY: &str = "STICKY";
}

/// `org.bukkit.block.data.type.TestBlock$Mode`.
pub mod test_block_mode {
    /// bukkit: org/bukkit/block/data/type/TestBlock$Mode.values
    pub fn values() -> &'static [&'static str] {
        &["START", "LOG", "FAIL", "ACCEPT"]
    }
    /// bukkit: org/bukkit/block/data/type/TestBlock$Mode.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const START: &str = "START";
    pub const LOG: &str = "LOG";
    pub const FAIL: &str = "FAIL";
    pub const ACCEPT: &str = "ACCEPT";
}

/// `org.bukkit.block.data.type.TrialSpawner$State`.
pub mod trial_spawner_state {
    /// bukkit: org/bukkit/block/data/type/TrialSpawner$State.values
    pub fn values() -> &'static [&'static str] {
        &["INACTIVE", "WAITING_FOR_PLAYERS", "ACTIVE", "WAITING_FOR_REWARD_EJECTION", "EJECTING_REWARD", "COOLDOWN"]
    }
    /// bukkit: org/bukkit/block/data/type/TrialSpawner$State.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const INACTIVE: &str = "INACTIVE";
    pub const WAITING_FOR_PLAYERS: &str = "WAITING_FOR_PLAYERS";
    pub const ACTIVE: &str = "ACTIVE";
    pub const WAITING_FOR_REWARD_EJECTION: &str = "WAITING_FOR_REWARD_EJECTION";
    pub const EJECTING_REWARD: &str = "EJECTING_REWARD";
    pub const COOLDOWN: &str = "COOLDOWN";
}

/// `org.bukkit.block.data.type.Vault$State`.
pub mod vault_state {
    /// bukkit: org/bukkit/block/data/type/Vault$State.values
    pub fn values() -> &'static [&'static str] {
        &["INACTIVE", "ACTIVE", "UNLOCKING", "EJECTING"]
    }
    /// bukkit: org/bukkit/block/data/type/Vault$State.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const INACTIVE: &str = "INACTIVE";
    pub const ACTIVE: &str = "ACTIVE";
    pub const UNLOCKING: &str = "UNLOCKING";
    pub const EJECTING: &str = "EJECTING";
}

/// `org.bukkit.block.data.type.Wall$Height`.
pub mod wall_height {
    /// bukkit: org/bukkit/block/data/type/Wall$Height.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "LOW", "TALL"]
    }
    /// bukkit: org/bukkit/block/data/type/Wall$Height.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const LOW: &str = "LOW";
    pub const TALL: &str = "TALL";
}

/// `org.bukkit.block.sign.Side`.
pub mod side {
    /// bukkit: org/bukkit/block/sign/Side.values
    pub fn values() -> &'static [&'static str] {
        &["FRONT", "BACK"]
    }
    /// bukkit: org/bukkit/block/sign/Side.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FRONT: &str = "FRONT";
    pub const BACK: &str = "BACK";
}

/// `org.bukkit.block.structure.Mirror`.
pub mod mirror {
    /// bukkit: org/bukkit/block/structure/Mirror.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "LEFT_RIGHT", "FRONT_BACK"]
    }
    /// bukkit: org/bukkit/block/structure/Mirror.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const LEFT_RIGHT: &str = "LEFT_RIGHT";
    pub const FRONT_BACK: &str = "FRONT_BACK";
}

/// `org.bukkit.block.structure.StructureRotation`.
pub mod structure_rotation {
    /// bukkit: org/bukkit/block/structure/StructureRotation.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "CLOCKWISE_90", "CLOCKWISE_180", "COUNTERCLOCKWISE_90"]
    }
    /// bukkit: org/bukkit/block/structure/StructureRotation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const CLOCKWISE_90: &str = "CLOCKWISE_90";
    pub const CLOCKWISE_180: &str = "CLOCKWISE_180";
    pub const COUNTERCLOCKWISE_90: &str = "COUNTERCLOCKWISE_90";
}

/// `org.bukkit.block.structure.UsageMode`.
pub mod usage_mode {
    /// bukkit: org/bukkit/block/structure/UsageMode.values
    pub fn values() -> &'static [&'static str] {
        &["SAVE", "LOAD", "CORNER", "DATA"]
    }
    /// bukkit: org/bukkit/block/structure/UsageMode.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SAVE: &str = "SAVE";
    pub const LOAD: &str = "LOAD";
    pub const CORNER: &str = "CORNER";
    pub const DATA: &str = "DATA";
}

/// `org.bukkit.boss.BarColor`.
pub mod bar_color {
    /// bukkit: org/bukkit/boss/BarColor.values
    pub fn values() -> &'static [&'static str] {
        &["PINK", "BLUE", "RED", "GREEN", "YELLOW", "PURPLE", "WHITE"]
    }
    /// bukkit: org/bukkit/boss/BarColor.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PINK: &str = "PINK";
    pub const BLUE: &str = "BLUE";
    pub const RED: &str = "RED";
    pub const GREEN: &str = "GREEN";
    pub const YELLOW: &str = "YELLOW";
    pub const PURPLE: &str = "PURPLE";
    pub const WHITE: &str = "WHITE";
}

/// `org.bukkit.boss.BarFlag`.
pub mod bar_flag {
    /// bukkit: org/bukkit/boss/BarFlag.values
    pub fn values() -> &'static [&'static str] {
        &["DARKEN_SKY", "PLAY_BOSS_MUSIC", "CREATE_FOG"]
    }
    /// bukkit: org/bukkit/boss/BarFlag.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DARKEN_SKY: &str = "DARKEN_SKY";
    pub const PLAY_BOSS_MUSIC: &str = "PLAY_BOSS_MUSIC";
    pub const CREATE_FOG: &str = "CREATE_FOG";
}

/// `org.bukkit.boss.BarStyle`.
pub mod bar_style {
    /// bukkit: org/bukkit/boss/BarStyle.values
    pub fn values() -> &'static [&'static str] {
        &["SOLID", "SEGMENTED_6", "SEGMENTED_10", "SEGMENTED_12", "SEGMENTED_20"]
    }
    /// bukkit: org/bukkit/boss/BarStyle.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SOLID: &str = "SOLID";
    pub const SEGMENTED_6: &str = "SEGMENTED_6";
    pub const SEGMENTED_10: &str = "SEGMENTED_10";
    pub const SEGMENTED_12: &str = "SEGMENTED_12";
    pub const SEGMENTED_20: &str = "SEGMENTED_20";
}

/// `org.bukkit.boss.DragonBattle$RespawnPhase`.
pub mod dragon_battle_respawn_phase {
    /// bukkit: org/bukkit/boss/DragonBattle$RespawnPhase.values
    pub fn values() -> &'static [&'static str] {
        &["START", "PREPARING_TO_SUMMON_PILLARS", "SUMMONING_PILLARS", "SUMMONING_DRAGON", "END", "NONE"]
    }
    /// bukkit: org/bukkit/boss/DragonBattle$RespawnPhase.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const START: &str = "START";
    pub const PREPARING_TO_SUMMON_PILLARS: &str = "PREPARING_TO_SUMMON_PILLARS";
    pub const SUMMONING_PILLARS: &str = "SUMMONING_PILLARS";
    pub const SUMMONING_DRAGON: &str = "SUMMONING_DRAGON";
    pub const END: &str = "END";
    pub const NONE: &str = "NONE";
}

/// `org.bukkit.conversations.Conversation$ConversationState`.
pub mod conversation_conversation_state {
    /// bukkit: org/bukkit/conversations/Conversation$ConversationState.values
    pub fn values() -> &'static [&'static str] {
        &["UNSTARTED", "STARTED", "ABANDONED"]
    }
    /// bukkit: org/bukkit/conversations/Conversation$ConversationState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UNSTARTED: &str = "UNSTARTED";
    pub const STARTED: &str = "STARTED";
    pub const ABANDONED: &str = "ABANDONED";
}

/// `org.bukkit.damage.DamageScaling`.
pub mod damage_scaling {
    /// bukkit: org/bukkit/damage/DamageScaling.values
    pub fn values() -> &'static [&'static str] {
        &["NEVER", "WHEN_CAUSED_BY_LIVING_NON_PLAYER", "ALWAYS"]
    }
    /// bukkit: org/bukkit/damage/DamageScaling.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NEVER: &str = "NEVER";
    pub const WHEN_CAUSED_BY_LIVING_NON_PLAYER: &str = "WHEN_CAUSED_BY_LIVING_NON_PLAYER";
    pub const ALWAYS: &str = "ALWAYS";
}

/// `org.bukkit.damage.DeathMessageType`.
pub mod death_message_type {
    /// bukkit: org/bukkit/damage/DeathMessageType.values
    pub fn values() -> &'static [&'static str] {
        &["DEFAULT", "FALL_VARIANTS", "INTENTIONAL_GAME_DESIGN"]
    }
    /// bukkit: org/bukkit/damage/DeathMessageType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DEFAULT: &str = "DEFAULT";
    pub const FALL_VARIANTS: &str = "FALL_VARIANTS";
    pub const INTENTIONAL_GAME_DESIGN: &str = "INTENTIONAL_GAME_DESIGN";
}

/// `org.bukkit.enchantments.Enchantment`.
pub mod enchantment {
    /// bukkit: org/bukkit/enchantments/Enchantment.values
    pub fn values() -> &'static [&'static str] {
        &["PROTECTION", "FIRE_PROTECTION", "FEATHER_FALLING", "BLAST_PROTECTION", "PROJECTILE_PROTECTION", "RESPIRATION", "AQUA_AFFINITY", "THORNS", "DEPTH_STRIDER", "FROST_WALKER", "BINDING_CURSE", "SHARPNESS", "SMITE", "BANE_OF_ARTHROPODS", "KNOCKBACK", "FIRE_ASPECT", "LOOTING", "SWEEPING_EDGE", "EFFICIENCY", "SILK_TOUCH", "UNBREAKING", "FORTUNE", "POWER", "PUNCH", "FLAME", "INFINITY", "LUCK_OF_THE_SEA", "LURE", "LOYALTY", "IMPALING", "RIPTIDE", "CHANNELING", "MULTISHOT", "QUICK_CHARGE", "PIERCING", "DENSITY", "BREACH", "WIND_BURST", "MENDING", "VANISHING_CURSE", "SOUL_SPEED", "SWIFT_SNEAK", "LUNGE"]
    }
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PROTECTION: &str = "minecraft:protection";
    pub const FIRE_PROTECTION: &str = "minecraft:fire_protection";
    pub const FEATHER_FALLING: &str = "minecraft:feather_falling";
    pub const BLAST_PROTECTION: &str = "minecraft:blast_protection";
    pub const PROJECTILE_PROTECTION: &str = "minecraft:projectile_protection";
    pub const RESPIRATION: &str = "minecraft:respiration";
    pub const AQUA_AFFINITY: &str = "minecraft:aqua_affinity";
    pub const THORNS: &str = "minecraft:thorns";
    pub const DEPTH_STRIDER: &str = "minecraft:depth_strider";
    pub const FROST_WALKER: &str = "minecraft:frost_walker";
    pub const BINDING_CURSE: &str = "minecraft:binding_curse";
    pub const SHARPNESS: &str = "minecraft:sharpness";
    pub const SMITE: &str = "minecraft:smite";
    pub const BANE_OF_ARTHROPODS: &str = "minecraft:bane_of_arthropods";
    pub const KNOCKBACK: &str = "minecraft:knockback";
    pub const FIRE_ASPECT: &str = "minecraft:fire_aspect";
    pub const LOOTING: &str = "minecraft:looting";
    pub const SWEEPING_EDGE: &str = "minecraft:sweeping_edge";
    pub const EFFICIENCY: &str = "minecraft:efficiency";
    pub const SILK_TOUCH: &str = "minecraft:silk_touch";
    pub const UNBREAKING: &str = "minecraft:unbreaking";
    pub const FORTUNE: &str = "minecraft:fortune";
    pub const POWER: &str = "minecraft:power";
    pub const PUNCH: &str = "minecraft:punch";
    pub const FLAME: &str = "minecraft:flame";
    pub const INFINITY: &str = "minecraft:infinity";
    pub const LUCK_OF_THE_SEA: &str = "minecraft:luck_of_the_sea";
    pub const LURE: &str = "minecraft:lure";
    pub const LOYALTY: &str = "minecraft:loyalty";
    pub const IMPALING: &str = "minecraft:impaling";
    pub const RIPTIDE: &str = "minecraft:riptide";
    pub const CHANNELING: &str = "minecraft:channeling";
    pub const MULTISHOT: &str = "minecraft:multishot";
    pub const QUICK_CHARGE: &str = "minecraft:quick_charge";
    pub const PIERCING: &str = "minecraft:piercing";
    pub const DENSITY: &str = "minecraft:density";
    pub const BREACH: &str = "minecraft:breach";
    pub const WIND_BURST: &str = "minecraft:wind_burst";
    pub const MENDING: &str = "minecraft:mending";
    pub const VANISHING_CURSE: &str = "minecraft:vanishing_curse";
    pub const SOUL_SPEED: &str = "minecraft:soul_speed";
    pub const SWIFT_SNEAK: &str = "minecraft:swift_sneak";
    pub const LUNGE: &str = "minecraft:lunge";
}

/// `org.bukkit.enchantments.EnchantmentTarget`.
pub mod enchantment_target {
    /// bukkit: org/bukkit/enchantments/EnchantmentTarget.values
    pub fn values() -> &'static [&'static str] {
        &["ALL", "ARMOR", "ARMOR_FEET", "ARMOR_LEGS", "ARMOR_TORSO", "ARMOR_HEAD", "WEAPON", "TOOL", "BOW", "FISHING_ROD", "BREAKABLE", "WEARABLE", "TRIDENT", "CROSSBOW", "VANISHABLE"]
    }
    /// bukkit: org/bukkit/enchantments/EnchantmentTarget.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALL: &str = "ALL";
    pub const ARMOR: &str = "ARMOR";
    pub const ARMOR_FEET: &str = "ARMOR_FEET";
    pub const ARMOR_LEGS: &str = "ARMOR_LEGS";
    pub const ARMOR_TORSO: &str = "ARMOR_TORSO";
    pub const ARMOR_HEAD: &str = "ARMOR_HEAD";
    pub const WEAPON: &str = "WEAPON";
    pub const TOOL: &str = "TOOL";
    pub const BOW: &str = "BOW";
    pub const FISHING_ROD: &str = "FISHING_ROD";
    pub const BREAKABLE: &str = "BREAKABLE";
    pub const WEARABLE: &str = "WEARABLE";
    pub const TRIDENT: &str = "TRIDENT";
    pub const CROSSBOW: &str = "CROSSBOW";
    pub const VANISHABLE: &str = "VANISHABLE";
}

/// `org.bukkit.entity.AbstractArrow$PickupRule`.
pub mod abstract_arrow_pickup_rule {
    /// bukkit: org/bukkit/entity/AbstractArrow$PickupRule.values
    pub fn values() -> &'static [&'static str] {
        &["DISALLOWED", "ALLOWED", "CREATIVE_ONLY"]
    }
    /// bukkit: org/bukkit/entity/AbstractArrow$PickupRule.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DISALLOWED: &str = "DISALLOWED";
    pub const ALLOWED: &str = "ALLOWED";
    pub const CREATIVE_ONLY: &str = "CREATIVE_ONLY";
}

/// `org.bukkit.entity.AbstractArrow$PickupStatus`.
pub mod abstract_arrow_pickup_status {
    /// bukkit: org/bukkit/entity/AbstractArrow$PickupStatus.values
    pub fn values() -> &'static [&'static str] {
        &["DISALLOWED", "ALLOWED", "CREATIVE_ONLY"]
    }
    /// bukkit: org/bukkit/entity/AbstractArrow$PickupStatus.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DISALLOWED: &str = "DISALLOWED";
    pub const ALLOWED: &str = "ALLOWED";
    pub const CREATIVE_ONLY: &str = "CREATIVE_ONLY";
}

/// `org.bukkit.entity.Armadillo$State`.
pub mod armadillo_state {
    /// bukkit: org/bukkit/entity/Armadillo$State.values
    pub fn values() -> &'static [&'static str] {
        &["IDLE", "ROLLING", "SCARED", "UNROLLING"]
    }
    /// bukkit: org/bukkit/entity/Armadillo$State.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const IDLE: &str = "IDLE";
    pub const ROLLING: &str = "ROLLING";
    pub const SCARED: &str = "SCARED";
    pub const UNROLLING: &str = "UNROLLING";
}

/// `org.bukkit.entity.ArmorStand$LockType`.
pub mod armor_stand_lock_type {
    /// bukkit: org/bukkit/entity/ArmorStand$LockType.values
    pub fn values() -> &'static [&'static str] {
        &["ADDING_OR_CHANGING", "REMOVING_OR_CHANGING", "ADDING"]
    }
    /// bukkit: org/bukkit/entity/ArmorStand$LockType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ADDING_OR_CHANGING: &str = "ADDING_OR_CHANGING";
    pub const REMOVING_OR_CHANGING: &str = "REMOVING_OR_CHANGING";
    pub const ADDING: &str = "ADDING";
}

/// `org.bukkit.entity.Axolotl$Variant`.
pub mod axolotl_variant {
    /// bukkit: org/bukkit/entity/Axolotl$Variant.values
    pub fn values() -> &'static [&'static str] {
        &["LUCY", "WILD", "GOLD", "CYAN", "BLUE"]
    }
    /// bukkit: org/bukkit/entity/Axolotl$Variant.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LUCY: &str = "LUCY";
    pub const WILD: &str = "WILD";
    pub const GOLD: &str = "GOLD";
    pub const CYAN: &str = "CYAN";
    pub const BLUE: &str = "BLUE";
}

/// `org.bukkit.entity.Boat$Status`.
pub mod boat_status {
    /// bukkit: org/bukkit/entity/Boat$Status.values
    pub fn values() -> &'static [&'static str] {
        &["NOT_IN_WORLD", "IN_WATER", "UNDER_WATER", "UNDER_FLOWING_WATER", "ON_LAND", "IN_AIR"]
    }
    /// bukkit: org/bukkit/entity/Boat$Status.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NOT_IN_WORLD: &str = "NOT_IN_WORLD";
    pub const IN_WATER: &str = "IN_WATER";
    pub const UNDER_WATER: &str = "UNDER_WATER";
    pub const UNDER_FLOWING_WATER: &str = "UNDER_FLOWING_WATER";
    pub const ON_LAND: &str = "ON_LAND";
    pub const IN_AIR: &str = "IN_AIR";
}

/// `org.bukkit.entity.Boat$Type`.
pub mod boat_type {
    /// bukkit: org/bukkit/entity/Boat$Type.values
    pub fn values() -> &'static [&'static str] {
        &["OAK", "SPRUCE", "BIRCH", "JUNGLE", "ACACIA", "CHERRY", "DARK_OAK", "MANGROVE", "BAMBOO"]
    }
    /// bukkit: org/bukkit/entity/Boat$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const OAK: &str = "OAK";
    pub const SPRUCE: &str = "SPRUCE";
    pub const BIRCH: &str = "BIRCH";
    pub const JUNGLE: &str = "JUNGLE";
    pub const ACACIA: &str = "ACACIA";
    pub const CHERRY: &str = "CHERRY";
    pub const DARK_OAK: &str = "DARK_OAK";
    pub const MANGROVE: &str = "MANGROVE";
    pub const BAMBOO: &str = "BAMBOO";
}

/// `org.bukkit.entity.Cat$Type`.
pub mod cat_type {
    /// bukkit: org/bukkit/entity/Cat$Type.values
    pub fn values() -> &'static [&'static str] {
        &["ALL_BLACK", "BLACK", "BRITISH_SHORTHAIR", "CALICO", "JELLIE", "PERSIAN", "RAGDOLL", "RED", "SIAMESE", "TABBY", "WHITE"]
    }
    /// bukkit: org/bukkit/entity/Cat$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALL_BLACK: &str = "minecraft:all_black";
    pub const BLACK: &str = "minecraft:black";
    pub const BRITISH_SHORTHAIR: &str = "minecraft:british_shorthair";
    pub const CALICO: &str = "minecraft:calico";
    pub const JELLIE: &str = "minecraft:jellie";
    pub const PERSIAN: &str = "minecraft:persian";
    pub const RAGDOLL: &str = "minecraft:ragdoll";
    pub const RED: &str = "minecraft:red";
    pub const SIAMESE: &str = "minecraft:siamese";
    pub const TABBY: &str = "minecraft:tabby";
    pub const WHITE: &str = "minecraft:white";
}

/// `org.bukkit.entity.CopperGolem$State`.
pub mod copper_golem_state {
    /// bukkit: org/bukkit/entity/CopperGolem$State.values
    pub fn values() -> &'static [&'static str] {
        &["IDLE", "GETTING_ITEM", "GETTING_NO_ITEM", "DROPPING_ITEM", "DROPPING_NO_ITEM"]
    }
    /// bukkit: org/bukkit/entity/CopperGolem$State.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const IDLE: &str = "IDLE";
    pub const GETTING_ITEM: &str = "GETTING_ITEM";
    pub const GETTING_NO_ITEM: &str = "GETTING_NO_ITEM";
    pub const DROPPING_ITEM: &str = "DROPPING_ITEM";
    pub const DROPPING_NO_ITEM: &str = "DROPPING_NO_ITEM";
}

/// `org.bukkit.entity.Display$Billboard`.
pub mod display_billboard {
    /// bukkit: org/bukkit/entity/Display$Billboard.values
    pub fn values() -> &'static [&'static str] {
        &["FIXED", "VERTICAL", "HORIZONTAL", "CENTER"]
    }
    /// bukkit: org/bukkit/entity/Display$Billboard.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FIXED: &str = "FIXED";
    pub const VERTICAL: &str = "VERTICAL";
    pub const HORIZONTAL: &str = "HORIZONTAL";
    pub const CENTER: &str = "CENTER";
}

/// `org.bukkit.entity.EnderDragon$Phase`.
pub mod ender_dragon_phase {
    /// bukkit: org/bukkit/entity/EnderDragon$Phase.values
    pub fn values() -> &'static [&'static str] {
        &["CIRCLING", "STRAFING", "FLY_TO_PORTAL", "LAND_ON_PORTAL", "LEAVE_PORTAL", "BREATH_ATTACK", "SEARCH_FOR_BREATH_ATTACK_TARGET", "ROAR_BEFORE_ATTACK", "CHARGE_PLAYER", "DYING", "HOVER"]
    }
    /// bukkit: org/bukkit/entity/EnderDragon$Phase.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CIRCLING: &str = "CIRCLING";
    pub const STRAFING: &str = "STRAFING";
    pub const FLY_TO_PORTAL: &str = "FLY_TO_PORTAL";
    pub const LAND_ON_PORTAL: &str = "LAND_ON_PORTAL";
    pub const LEAVE_PORTAL: &str = "LEAVE_PORTAL";
    pub const BREATH_ATTACK: &str = "BREATH_ATTACK";
    pub const SEARCH_FOR_BREATH_ATTACK_TARGET: &str = "SEARCH_FOR_BREATH_ATTACK_TARGET";
    pub const ROAR_BEFORE_ATTACK: &str = "ROAR_BEFORE_ATTACK";
    pub const CHARGE_PLAYER: &str = "CHARGE_PLAYER";
    pub const DYING: &str = "DYING";
    pub const HOVER: &str = "HOVER";
}

/// `org.bukkit.entity.EntityCategory`.
pub mod entity_category {
    /// bukkit: org/bukkit/entity/EntityCategory.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "UNDEAD", "ARTHROPOD", "ILLAGER", "WATER"]
    }
    /// bukkit: org/bukkit/entity/EntityCategory.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const UNDEAD: &str = "UNDEAD";
    pub const ARTHROPOD: &str = "ARTHROPOD";
    pub const ILLAGER: &str = "ILLAGER";
    pub const WATER: &str = "WATER";
}

/// `org.bukkit.entity.EntityType`.
pub mod entity_type {
    /// bukkit: org/bukkit/entity/EntityType.values
    pub fn values() -> &'static [&'static str] {
        &["ACACIA_BOAT", "ACACIA_CHEST_BOAT", "ALLAY", "AREA_EFFECT_CLOUD", "ARMADILLO", "ARMOR_STAND", "ARROW", "AXOLOTL", "BAMBOO_CHEST_RAFT", "BAMBOO_RAFT", "BAT", "BEE", "BIRCH_BOAT", "BIRCH_CHEST_BOAT", "BLAZE", "BLOCK_DISPLAY", "BOGGED", "BREEZE", "BREEZE_WIND_CHARGE", "CAMEL", "CAMEL_HUSK", "CAT", "CAVE_SPIDER", "CHERRY_BOAT", "CHERRY_CHEST_BOAT", "CHEST_MINECART", "CHICKEN", "COD", "COMMAND_BLOCK_MINECART", "COPPER_GOLEM", "COW", "CREAKING", "CREEPER", "CUSHION", "DARK_OAK_BOAT", "DARK_OAK_CHEST_BOAT", "DOLPHIN", "DONKEY", "DRAGON_FIREBALL", "DROWNED", "EGG", "ELDER_GUARDIAN", "END_CRYSTAL", "ENDER_DRAGON", "ENDER_PEARL", "ENDERMAN", "ENDERMITE", "EVOKER", "EVOKER_FANGS", "EXPERIENCE_BOTTLE", "EXPERIENCE_ORB", "EYE_OF_ENDER", "FALLING_BLOCK", "FIREBALL", "FIREWORK_ROCKET", "FISHING_BOBBER", "FOX", "FROG", "FURNACE_MINECART", "GHAST", "GIANT", "GLOW_ITEM_FRAME", "GLOW_SQUID", "GOAT", "GUARDIAN", "HAPPY_GHAST", "HOGLIN", "HOPPER_MINECART", "HORSE", "HUSK", "ILLUSIONER", "INTERACTION", "IRON_GOLEM", "ITEM", "ITEM_DISPLAY", "ITEM_FRAME", "JUNGLE_BOAT", "JUNGLE_CHEST_BOAT", "LEASH_KNOT", "LIGHTNING_BOLT", "LINGERING_POTION", "LLAMA", "LLAMA_SPIT", "MAGMA_CUBE", "MANGROVE_BOAT", "MANGROVE_CHEST_BOAT", "MANNEQUIN", "MARKER", "MINECART", "MOOSHROOM", "MULE", "NAUTILUS", "OAK_BOAT", "OAK_CHEST_BOAT", "OCELOT", "OMINOUS_ITEM_SPAWNER", "PAINTING", "PALE_OAK_BOAT", "PALE_OAK_CHEST_BOAT", "PANDA", "PARCHED", "PARROT", "PHANTOM", "PIG", "PIGLIN", "PIGLIN_BRUTE", "PILLAGER", "PLAYER", "POLAR_BEAR", "POPLAR_BOAT", "POPLAR_CHEST_BOAT", "PUFFERFISH", "RABBIT", "RAVAGER", "SALMON", "SHEEP", "SHULKER", "SHULKER_BULLET", "SILVERFISH", "SKELETON", "SKELETON_HORSE", "SLIME", "SMALL_FIREBALL", "SNIFFER", "SNOW_GOLEM", "SNOWBALL", "SPAWNER_MINECART", "SPECTRAL_ARROW", "SPIDER", "SPLASH_POTION", "SPRUCE_BOAT", "SPRUCE_CHEST_BOAT", "SQUID", "STRAY", "STRIDER", "SULFUR_CUBE", "TADPOLE", "TEXT_DISPLAY", "TNT", "TNT_MINECART", "TRADER_LLAMA", "TRIDENT", "TROPICAL_FISH", "TURTLE", "VEX", "VILLAGER", "VINDICATOR", "WANDERING_TRADER", "WARDEN", "WIND_CHARGE", "WITCH", "WITHER", "WITHER_SKELETON", "WITHER_SKULL", "WOLF", "ZOGLIN", "ZOMBIE", "ZOMBIE_HORSE", "ZOMBIE_NAUTILUS", "ZOMBIE_VILLAGER", "ZOMBIFIED_PIGLIN", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/entity/EntityType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ACACIA_BOAT: &str = "ACACIA_BOAT";
    pub const ACACIA_CHEST_BOAT: &str = "ACACIA_CHEST_BOAT";
    pub const ALLAY: &str = "ALLAY";
    pub const AREA_EFFECT_CLOUD: &str = "AREA_EFFECT_CLOUD";
    pub const ARMADILLO: &str = "ARMADILLO";
    pub const ARMOR_STAND: &str = "ARMOR_STAND";
    pub const ARROW: &str = "ARROW";
    pub const AXOLOTL: &str = "AXOLOTL";
    pub const BAMBOO_CHEST_RAFT: &str = "BAMBOO_CHEST_RAFT";
    pub const BAMBOO_RAFT: &str = "BAMBOO_RAFT";
    pub const BAT: &str = "BAT";
    pub const BEE: &str = "BEE";
    pub const BIRCH_BOAT: &str = "BIRCH_BOAT";
    pub const BIRCH_CHEST_BOAT: &str = "BIRCH_CHEST_BOAT";
    pub const BLAZE: &str = "BLAZE";
    pub const BLOCK_DISPLAY: &str = "BLOCK_DISPLAY";
    pub const BOGGED: &str = "BOGGED";
    pub const BREEZE: &str = "BREEZE";
    pub const BREEZE_WIND_CHARGE: &str = "BREEZE_WIND_CHARGE";
    pub const CAMEL: &str = "CAMEL";
    pub const CAMEL_HUSK: &str = "CAMEL_HUSK";
    pub const CAT: &str = "CAT";
    pub const CAVE_SPIDER: &str = "CAVE_SPIDER";
    pub const CHERRY_BOAT: &str = "CHERRY_BOAT";
    pub const CHERRY_CHEST_BOAT: &str = "CHERRY_CHEST_BOAT";
    pub const CHEST_MINECART: &str = "CHEST_MINECART";
    pub const CHICKEN: &str = "CHICKEN";
    pub const COD: &str = "COD";
    pub const COMMAND_BLOCK_MINECART: &str = "COMMAND_BLOCK_MINECART";
    pub const COPPER_GOLEM: &str = "COPPER_GOLEM";
    pub const COW: &str = "COW";
    pub const CREAKING: &str = "CREAKING";
    pub const CREEPER: &str = "CREEPER";
    pub const CUSHION: &str = "CUSHION";
    pub const DARK_OAK_BOAT: &str = "DARK_OAK_BOAT";
    pub const DARK_OAK_CHEST_BOAT: &str = "DARK_OAK_CHEST_BOAT";
    pub const DOLPHIN: &str = "DOLPHIN";
    pub const DONKEY: &str = "DONKEY";
    pub const DRAGON_FIREBALL: &str = "DRAGON_FIREBALL";
    pub const DROWNED: &str = "DROWNED";
    pub const EGG: &str = "EGG";
    pub const ELDER_GUARDIAN: &str = "ELDER_GUARDIAN";
    pub const END_CRYSTAL: &str = "END_CRYSTAL";
    pub const ENDER_DRAGON: &str = "ENDER_DRAGON";
    pub const ENDER_PEARL: &str = "ENDER_PEARL";
    pub const ENDERMAN: &str = "ENDERMAN";
    pub const ENDERMITE: &str = "ENDERMITE";
    pub const EVOKER: &str = "EVOKER";
    pub const EVOKER_FANGS: &str = "EVOKER_FANGS";
    pub const EXPERIENCE_BOTTLE: &str = "EXPERIENCE_BOTTLE";
    pub const EXPERIENCE_ORB: &str = "EXPERIENCE_ORB";
    pub const EYE_OF_ENDER: &str = "EYE_OF_ENDER";
    pub const FALLING_BLOCK: &str = "FALLING_BLOCK";
    pub const FIREBALL: &str = "FIREBALL";
    pub const FIREWORK_ROCKET: &str = "FIREWORK_ROCKET";
    pub const FISHING_BOBBER: &str = "FISHING_BOBBER";
    pub const FOX: &str = "FOX";
    pub const FROG: &str = "FROG";
    pub const FURNACE_MINECART: &str = "FURNACE_MINECART";
    pub const GHAST: &str = "GHAST";
    pub const GIANT: &str = "GIANT";
    pub const GLOW_ITEM_FRAME: &str = "GLOW_ITEM_FRAME";
    pub const GLOW_SQUID: &str = "GLOW_SQUID";
    pub const GOAT: &str = "GOAT";
    pub const GUARDIAN: &str = "GUARDIAN";
    pub const HAPPY_GHAST: &str = "HAPPY_GHAST";
    pub const HOGLIN: &str = "HOGLIN";
    pub const HOPPER_MINECART: &str = "HOPPER_MINECART";
    pub const HORSE: &str = "HORSE";
    pub const HUSK: &str = "HUSK";
    pub const ILLUSIONER: &str = "ILLUSIONER";
    pub const INTERACTION: &str = "INTERACTION";
    pub const IRON_GOLEM: &str = "IRON_GOLEM";
    pub const ITEM: &str = "ITEM";
    pub const ITEM_DISPLAY: &str = "ITEM_DISPLAY";
    pub const ITEM_FRAME: &str = "ITEM_FRAME";
    pub const JUNGLE_BOAT: &str = "JUNGLE_BOAT";
    pub const JUNGLE_CHEST_BOAT: &str = "JUNGLE_CHEST_BOAT";
    pub const LEASH_KNOT: &str = "LEASH_KNOT";
    pub const LIGHTNING_BOLT: &str = "LIGHTNING_BOLT";
    pub const LINGERING_POTION: &str = "LINGERING_POTION";
    pub const LLAMA: &str = "LLAMA";
    pub const LLAMA_SPIT: &str = "LLAMA_SPIT";
    pub const MAGMA_CUBE: &str = "MAGMA_CUBE";
    pub const MANGROVE_BOAT: &str = "MANGROVE_BOAT";
    pub const MANGROVE_CHEST_BOAT: &str = "MANGROVE_CHEST_BOAT";
    pub const MANNEQUIN: &str = "MANNEQUIN";
    pub const MARKER: &str = "MARKER";
    pub const MINECART: &str = "MINECART";
    pub const MOOSHROOM: &str = "MOOSHROOM";
    pub const MULE: &str = "MULE";
    pub const NAUTILUS: &str = "NAUTILUS";
    pub const OAK_BOAT: &str = "OAK_BOAT";
    pub const OAK_CHEST_BOAT: &str = "OAK_CHEST_BOAT";
    pub const OCELOT: &str = "OCELOT";
    pub const OMINOUS_ITEM_SPAWNER: &str = "OMINOUS_ITEM_SPAWNER";
    pub const PAINTING: &str = "PAINTING";
    pub const PALE_OAK_BOAT: &str = "PALE_OAK_BOAT";
    pub const PALE_OAK_CHEST_BOAT: &str = "PALE_OAK_CHEST_BOAT";
    pub const PANDA: &str = "PANDA";
    pub const PARCHED: &str = "PARCHED";
    pub const PARROT: &str = "PARROT";
    pub const PHANTOM: &str = "PHANTOM";
    pub const PIG: &str = "PIG";
    pub const PIGLIN: &str = "PIGLIN";
    pub const PIGLIN_BRUTE: &str = "PIGLIN_BRUTE";
    pub const PILLAGER: &str = "PILLAGER";
    pub const PLAYER: &str = "PLAYER";
    pub const POLAR_BEAR: &str = "POLAR_BEAR";
    pub const POPLAR_BOAT: &str = "POPLAR_BOAT";
    pub const POPLAR_CHEST_BOAT: &str = "POPLAR_CHEST_BOAT";
    pub const PUFFERFISH: &str = "PUFFERFISH";
    pub const RABBIT: &str = "RABBIT";
    pub const RAVAGER: &str = "RAVAGER";
    pub const SALMON: &str = "SALMON";
    pub const SHEEP: &str = "SHEEP";
    pub const SHULKER: &str = "SHULKER";
    pub const SHULKER_BULLET: &str = "SHULKER_BULLET";
    pub const SILVERFISH: &str = "SILVERFISH";
    pub const SKELETON: &str = "SKELETON";
    pub const SKELETON_HORSE: &str = "SKELETON_HORSE";
    pub const SLIME: &str = "SLIME";
    pub const SMALL_FIREBALL: &str = "SMALL_FIREBALL";
    pub const SNIFFER: &str = "SNIFFER";
    pub const SNOW_GOLEM: &str = "SNOW_GOLEM";
    pub const SNOWBALL: &str = "SNOWBALL";
    pub const SPAWNER_MINECART: &str = "SPAWNER_MINECART";
    pub const SPECTRAL_ARROW: &str = "SPECTRAL_ARROW";
    pub const SPIDER: &str = "SPIDER";
    pub const SPLASH_POTION: &str = "SPLASH_POTION";
    pub const SPRUCE_BOAT: &str = "SPRUCE_BOAT";
    pub const SPRUCE_CHEST_BOAT: &str = "SPRUCE_CHEST_BOAT";
    pub const SQUID: &str = "SQUID";
    pub const STRAY: &str = "STRAY";
    pub const STRIDER: &str = "STRIDER";
    pub const SULFUR_CUBE: &str = "SULFUR_CUBE";
    pub const TADPOLE: &str = "TADPOLE";
    pub const TEXT_DISPLAY: &str = "TEXT_DISPLAY";
    pub const TNT: &str = "TNT";
    pub const TNT_MINECART: &str = "TNT_MINECART";
    pub const TRADER_LLAMA: &str = "TRADER_LLAMA";
    pub const TRIDENT: &str = "TRIDENT";
    pub const TROPICAL_FISH: &str = "TROPICAL_FISH";
    pub const TURTLE: &str = "TURTLE";
    pub const VEX: &str = "VEX";
    pub const VILLAGER: &str = "VILLAGER";
    pub const VINDICATOR: &str = "VINDICATOR";
    pub const WANDERING_TRADER: &str = "WANDERING_TRADER";
    pub const WARDEN: &str = "WARDEN";
    pub const WIND_CHARGE: &str = "WIND_CHARGE";
    pub const WITCH: &str = "WITCH";
    pub const WITHER: &str = "WITHER";
    pub const WITHER_SKELETON: &str = "WITHER_SKELETON";
    pub const WITHER_SKULL: &str = "WITHER_SKULL";
    pub const WOLF: &str = "WOLF";
    pub const ZOGLIN: &str = "ZOGLIN";
    pub const ZOMBIE: &str = "ZOMBIE";
    pub const ZOMBIE_HORSE: &str = "ZOMBIE_HORSE";
    pub const ZOMBIE_NAUTILUS: &str = "ZOMBIE_NAUTILUS";
    pub const ZOMBIE_VILLAGER: &str = "ZOMBIE_VILLAGER";
    pub const ZOMBIFIED_PIGLIN: &str = "ZOMBIFIED_PIGLIN";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.entity.Evoker$Spell`.
pub mod evoker_spell {
    /// bukkit: org/bukkit/entity/Evoker$Spell.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "SUMMON", "FANGS", "WOLOLO", "DISAPPEAR", "BLINDNESS"]
    }
    /// bukkit: org/bukkit/entity/Evoker$Spell.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const SUMMON: &str = "SUMMON";
    pub const FANGS: &str = "FANGS";
    pub const WOLOLO: &str = "WOLOLO";
    pub const DISAPPEAR: &str = "DISAPPEAR";
    pub const BLINDNESS: &str = "BLINDNESS";
}

/// `org.bukkit.entity.ExperienceOrb$SpawnReason`.
pub mod experience_orb_spawn_reason {
    /// bukkit: org/bukkit/entity/ExperienceOrb$SpawnReason.values
    pub fn values() -> &'static [&'static str] {
        &["PLAYER_DEATH", "ENTITY_DEATH", "FURNACE", "BREED", "VILLAGER_TRADE", "FISHING", "BLOCK_BREAK", "CUSTOM", "EXP_BOTTLE", "GRINDSTONE", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/entity/ExperienceOrb$SpawnReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PLAYER_DEATH: &str = "PLAYER_DEATH";
    pub const ENTITY_DEATH: &str = "ENTITY_DEATH";
    pub const FURNACE: &str = "FURNACE";
    pub const BREED: &str = "BREED";
    pub const VILLAGER_TRADE: &str = "VILLAGER_TRADE";
    pub const FISHING: &str = "FISHING";
    pub const BLOCK_BREAK: &str = "BLOCK_BREAK";
    pub const CUSTOM: &str = "CUSTOM";
    pub const EXP_BOTTLE: &str = "EXP_BOTTLE";
    pub const GRINDSTONE: &str = "GRINDSTONE";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.entity.FishHook$HookState`.
pub mod fish_hook_hook_state {
    /// bukkit: org/bukkit/entity/FishHook$HookState.values
    pub fn values() -> &'static [&'static str] {
        &["UNHOOKED", "HOOKED_ENTITY", "BOBBING"]
    }
    /// bukkit: org/bukkit/entity/FishHook$HookState.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UNHOOKED: &str = "UNHOOKED";
    pub const HOOKED_ENTITY: &str = "HOOKED_ENTITY";
    pub const BOBBING: &str = "BOBBING";
}

/// `org.bukkit.entity.Fox$Type`.
pub mod fox_type {
    /// bukkit: org/bukkit/entity/Fox$Type.values
    pub fn values() -> &'static [&'static str] {
        &["RED", "SNOW"]
    }
    /// bukkit: org/bukkit/entity/Fox$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const RED: &str = "RED";
    pub const SNOW: &str = "SNOW";
}

/// `org.bukkit.entity.Frog$Variant`.
pub mod frog_variant {
    /// bukkit: org/bukkit/entity/Frog$Variant.values
    pub fn values() -> &'static [&'static str] {
        &["COLD", "TEMPERATE", "WARM"]
    }
    /// bukkit: org/bukkit/entity/Frog$Variant.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COLD: &str = "minecraft:cold";
    pub const TEMPERATE: &str = "minecraft:temperate";
    pub const WARM: &str = "minecraft:warm";
}

/// `org.bukkit.entity.Horse$Color`.
pub mod horse_color {
    /// bukkit: org/bukkit/entity/Horse$Color.values
    pub fn values() -> &'static [&'static str] {
        &["WHITE", "CREAMY", "CHESTNUT", "BROWN", "BLACK", "GRAY", "DARK_BROWN"]
    }
    /// bukkit: org/bukkit/entity/Horse$Color.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const WHITE: &str = "WHITE";
    pub const CREAMY: &str = "CREAMY";
    pub const CHESTNUT: &str = "CHESTNUT";
    pub const BROWN: &str = "BROWN";
    pub const BLACK: &str = "BLACK";
    pub const GRAY: &str = "GRAY";
    pub const DARK_BROWN: &str = "DARK_BROWN";
}

/// `org.bukkit.entity.Horse$Style`.
pub mod horse_style {
    /// bukkit: org/bukkit/entity/Horse$Style.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "WHITE", "WHITEFIELD", "WHITE_DOTS", "BLACK_DOTS"]
    }
    /// bukkit: org/bukkit/entity/Horse$Style.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const WHITE: &str = "WHITE";
    pub const WHITEFIELD: &str = "WHITEFIELD";
    pub const WHITE_DOTS: &str = "WHITE_DOTS";
    pub const BLACK_DOTS: &str = "BLACK_DOTS";
}

/// `org.bukkit.entity.Horse$Variant`.
pub mod horse_variant {
    /// bukkit: org/bukkit/entity/Horse$Variant.values
    pub fn values() -> &'static [&'static str] {
        &["HORSE", "DONKEY", "MULE", "UNDEAD_HORSE", "SKELETON_HORSE", "LLAMA", "CAMEL"]
    }
    /// bukkit: org/bukkit/entity/Horse$Variant.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const HORSE: &str = "HORSE";
    pub const DONKEY: &str = "DONKEY";
    pub const MULE: &str = "MULE";
    pub const UNDEAD_HORSE: &str = "UNDEAD_HORSE";
    pub const SKELETON_HORSE: &str = "SKELETON_HORSE";
    pub const LLAMA: &str = "LLAMA";
    pub const CAMEL: &str = "CAMEL";
}

/// `org.bukkit.entity.ItemDisplay$ItemDisplayTransform`.
pub mod item_display_item_display_transform {
    /// bukkit: org/bukkit/entity/ItemDisplay$ItemDisplayTransform.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "THIRDPERSON_LEFTHAND", "THIRDPERSON_RIGHTHAND", "FIRSTPERSON_LEFTHAND", "FIRSTPERSON_RIGHTHAND", "HEAD", "GUI", "GROUND", "FIXED"]
    }
    /// bukkit: org/bukkit/entity/ItemDisplay$ItemDisplayTransform.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const THIRDPERSON_LEFTHAND: &str = "THIRDPERSON_LEFTHAND";
    pub const THIRDPERSON_RIGHTHAND: &str = "THIRDPERSON_RIGHTHAND";
    pub const FIRSTPERSON_LEFTHAND: &str = "FIRSTPERSON_LEFTHAND";
    pub const FIRSTPERSON_RIGHTHAND: &str = "FIRSTPERSON_RIGHTHAND";
    pub const HEAD: &str = "HEAD";
    pub const GUI: &str = "GUI";
    pub const GROUND: &str = "GROUND";
    pub const FIXED: &str = "FIXED";
}

/// `org.bukkit.entity.Llama$Color`.
pub mod llama_color {
    /// bukkit: org/bukkit/entity/Llama$Color.values
    pub fn values() -> &'static [&'static str] {
        &["CREAMY", "WHITE", "BROWN", "GRAY"]
    }
    /// bukkit: org/bukkit/entity/Llama$Color.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CREAMY: &str = "CREAMY";
    pub const WHITE: &str = "WHITE";
    pub const BROWN: &str = "BROWN";
    pub const GRAY: &str = "GRAY";
}

/// `org.bukkit.entity.MushroomCow$Variant`.
pub mod mushroom_cow_variant {
    /// bukkit: org/bukkit/entity/MushroomCow$Variant.values
    pub fn values() -> &'static [&'static str] {
        &["RED", "BROWN"]
    }
    /// bukkit: org/bukkit/entity/MushroomCow$Variant.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const RED: &str = "RED";
    pub const BROWN: &str = "BROWN";
}

/// `org.bukkit.entity.Ocelot$Type`.
pub mod ocelot_type {
    /// bukkit: org/bukkit/entity/Ocelot$Type.values
    pub fn values() -> &'static [&'static str] {
        &["WILD_OCELOT", "BLACK_CAT", "RED_CAT", "SIAMESE_CAT"]
    }
    /// bukkit: org/bukkit/entity/Ocelot$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const WILD_OCELOT: &str = "WILD_OCELOT";
    pub const BLACK_CAT: &str = "BLACK_CAT";
    pub const RED_CAT: &str = "RED_CAT";
    pub const SIAMESE_CAT: &str = "SIAMESE_CAT";
}

/// `org.bukkit.entity.Panda$Gene`.
pub mod panda_gene {
    /// bukkit: org/bukkit/entity/Panda$Gene.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "LAZY", "WORRIED", "PLAYFUL", "BROWN", "WEAK", "AGGRESSIVE"]
    }
    /// bukkit: org/bukkit/entity/Panda$Gene.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const LAZY: &str = "LAZY";
    pub const WORRIED: &str = "WORRIED";
    pub const PLAYFUL: &str = "PLAYFUL";
    pub const BROWN: &str = "BROWN";
    pub const WEAK: &str = "WEAK";
    pub const AGGRESSIVE: &str = "AGGRESSIVE";
}

/// `org.bukkit.entity.Parrot$Variant`.
pub mod parrot_variant {
    /// bukkit: org/bukkit/entity/Parrot$Variant.values
    pub fn values() -> &'static [&'static str] {
        &["RED", "BLUE", "GREEN", "CYAN", "GRAY"]
    }
    /// bukkit: org/bukkit/entity/Parrot$Variant.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const RED: &str = "RED";
    pub const BLUE: &str = "BLUE";
    pub const GREEN: &str = "GREEN";
    pub const CYAN: &str = "CYAN";
    pub const GRAY: &str = "GRAY";
}

/// `org.bukkit.entity.Pose`.
pub mod pose {
    /// bukkit: org/bukkit/entity/Pose.values
    pub fn values() -> &'static [&'static str] {
        &["STANDING", "FALL_FLYING", "SLEEPING", "SWIMMING", "SPIN_ATTACK", "SNEAKING", "LONG_JUMPING", "DYING", "CROAKING", "USING_TONGUE", "SITTING", "ROARING", "SNIFFING", "EMERGING", "DIGGING", "SLIDING", "SHOOTING", "INHALING"]
    }
    /// bukkit: org/bukkit/entity/Pose.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const STANDING: &str = "STANDING";
    pub const FALL_FLYING: &str = "FALL_FLYING";
    pub const SLEEPING: &str = "SLEEPING";
    pub const SWIMMING: &str = "SWIMMING";
    pub const SPIN_ATTACK: &str = "SPIN_ATTACK";
    pub const SNEAKING: &str = "SNEAKING";
    pub const LONG_JUMPING: &str = "LONG_JUMPING";
    pub const DYING: &str = "DYING";
    pub const CROAKING: &str = "CROAKING";
    pub const USING_TONGUE: &str = "USING_TONGUE";
    pub const SITTING: &str = "SITTING";
    pub const ROARING: &str = "ROARING";
    pub const SNIFFING: &str = "SNIFFING";
    pub const EMERGING: &str = "EMERGING";
    pub const DIGGING: &str = "DIGGING";
    pub const SLIDING: &str = "SLIDING";
    pub const SHOOTING: &str = "SHOOTING";
    pub const INHALING: &str = "INHALING";
}

/// `org.bukkit.entity.Rabbit$Type`.
pub mod rabbit_type {
    /// bukkit: org/bukkit/entity/Rabbit$Type.values
    pub fn values() -> &'static [&'static str] {
        &["BROWN", "WHITE", "BLACK", "BLACK_AND_WHITE", "GOLD", "SALT_AND_PEPPER", "THE_KILLER_BUNNY"]
    }
    /// bukkit: org/bukkit/entity/Rabbit$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BROWN: &str = "BROWN";
    pub const WHITE: &str = "WHITE";
    pub const BLACK: &str = "BLACK";
    pub const BLACK_AND_WHITE: &str = "BLACK_AND_WHITE";
    pub const GOLD: &str = "GOLD";
    pub const SALT_AND_PEPPER: &str = "SALT_AND_PEPPER";
    pub const THE_KILLER_BUNNY: &str = "THE_KILLER_BUNNY";
}

/// `org.bukkit.entity.Salmon$Variant`.
pub mod salmon_variant {
    /// bukkit: org/bukkit/entity/Salmon$Variant.values
    pub fn values() -> &'static [&'static str] {
        &["SMALL", "MEDIUM", "LARGE"]
    }
    /// bukkit: org/bukkit/entity/Salmon$Variant.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SMALL: &str = "SMALL";
    pub const MEDIUM: &str = "MEDIUM";
    pub const LARGE: &str = "LARGE";
}

/// `org.bukkit.entity.Skeleton$SkeletonType`.
pub mod skeleton_skeleton_type {
    /// bukkit: org/bukkit/entity/Skeleton$SkeletonType.values
    pub fn values() -> &'static [&'static str] {
        &["NORMAL", "WITHER", "STRAY", "BOGGED", "PARCHED"]
    }
    /// bukkit: org/bukkit/entity/Skeleton$SkeletonType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NORMAL: &str = "NORMAL";
    pub const WITHER: &str = "WITHER";
    pub const STRAY: &str = "STRAY";
    pub const BOGGED: &str = "BOGGED";
    pub const PARCHED: &str = "PARCHED";
}

/// `org.bukkit.entity.Sniffer$State`.
pub mod sniffer_state {
    /// bukkit: org/bukkit/entity/Sniffer$State.values
    pub fn values() -> &'static [&'static str] {
        &["IDLING", "FEELING_HAPPY", "SCENTING", "SNIFFING", "SEARCHING", "DIGGING", "RISING"]
    }
    /// bukkit: org/bukkit/entity/Sniffer$State.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const IDLING: &str = "IDLING";
    pub const FEELING_HAPPY: &str = "FEELING_HAPPY";
    pub const SCENTING: &str = "SCENTING";
    pub const SNIFFING: &str = "SNIFFING";
    pub const SEARCHING: &str = "SEARCHING";
    pub const DIGGING: &str = "DIGGING";
    pub const RISING: &str = "RISING";
}

/// `org.bukkit.entity.SpawnCategory`.
pub mod spawn_category {
    /// bukkit: org/bukkit/entity/SpawnCategory.values
    pub fn values() -> &'static [&'static str] {
        &["MONSTER", "ANIMAL", "WATER_ANIMAL", "WATER_AMBIENT", "WATER_UNDERGROUND_CREATURE", "AMBIENT", "AXOLOTL", "MISC"]
    }
    /// bukkit: org/bukkit/entity/SpawnCategory.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const MONSTER: &str = "MONSTER";
    pub const ANIMAL: &str = "ANIMAL";
    pub const WATER_ANIMAL: &str = "WATER_ANIMAL";
    pub const WATER_AMBIENT: &str = "WATER_AMBIENT";
    pub const WATER_UNDERGROUND_CREATURE: &str = "WATER_UNDERGROUND_CREATURE";
    pub const AMBIENT: &str = "AMBIENT";
    pub const AXOLOTL: &str = "AXOLOTL";
    pub const MISC: &str = "MISC";
}

/// `org.bukkit.entity.Spellcaster$Spell`.
pub mod spellcaster_spell {
    /// bukkit: org/bukkit/entity/Spellcaster$Spell.values
    pub fn values() -> &'static [&'static str] {
        &["NONE", "SUMMON_VEX", "FANGS", "WOLOLO", "DISAPPEAR", "BLINDNESS"]
    }
    /// bukkit: org/bukkit/entity/Spellcaster$Spell.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NONE: &str = "NONE";
    pub const SUMMON_VEX: &str = "SUMMON_VEX";
    pub const FANGS: &str = "FANGS";
    pub const WOLOLO: &str = "WOLOLO";
    pub const DISAPPEAR: &str = "DISAPPEAR";
    pub const BLINDNESS: &str = "BLINDNESS";
}

/// `org.bukkit.entity.TextDisplay$TextAlignment`.
pub mod text_display_text_alignment {
    /// bukkit: org/bukkit/entity/TextDisplay$TextAlignment.values
    pub fn values() -> &'static [&'static str] {
        &["CENTER", "LEFT", "RIGHT"]
    }
    /// bukkit: org/bukkit/entity/TextDisplay$TextAlignment.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CENTER: &str = "CENTER";
    pub const LEFT: &str = "LEFT";
    pub const RIGHT: &str = "RIGHT";
}

/// `org.bukkit.entity.TropicalFish$Pattern`.
pub mod tropical_fish_pattern {
    /// bukkit: org/bukkit/entity/TropicalFish$Pattern.values
    pub fn values() -> &'static [&'static str] {
        &["KOB", "SUNSTREAK", "SNOOPER", "DASHER", "BRINELY", "SPOTTY", "FLOPPER", "STRIPEY", "GLITTER", "BLOCKFISH", "BETTY", "CLAYFISH"]
    }
    /// bukkit: org/bukkit/entity/TropicalFish$Pattern.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const KOB: &str = "KOB";
    pub const SUNSTREAK: &str = "SUNSTREAK";
    pub const SNOOPER: &str = "SNOOPER";
    pub const DASHER: &str = "DASHER";
    pub const BRINELY: &str = "BRINELY";
    pub const SPOTTY: &str = "SPOTTY";
    pub const FLOPPER: &str = "FLOPPER";
    pub const STRIPEY: &str = "STRIPEY";
    pub const GLITTER: &str = "GLITTER";
    pub const BLOCKFISH: &str = "BLOCKFISH";
    pub const BETTY: &str = "BETTY";
    pub const CLAYFISH: &str = "CLAYFISH";
}

/// `org.bukkit.entity.Villager$Profession`.
pub mod villager_profession {
    /// bukkit: org/bukkit/entity/Villager$Profession.values
    pub fn values() -> &'static [&'static str] {
        &["ARMORER", "BUTCHER", "CARTOGRAPHER", "CLERIC", "FARMER", "FISHERMAN", "FLETCHER", "LEATHERWORKER", "LIBRARIAN", "MASON", "NITWIT", "NONE", "SHEPHERD", "TOOLSMITH", "WEAPONSMITH"]
    }
    /// bukkit: org/bukkit/entity/Villager$Profession.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ARMORER: &str = "minecraft:armorer";
    pub const BUTCHER: &str = "minecraft:butcher";
    pub const CARTOGRAPHER: &str = "minecraft:cartographer";
    pub const CLERIC: &str = "minecraft:cleric";
    pub const FARMER: &str = "minecraft:farmer";
    pub const FISHERMAN: &str = "minecraft:fisherman";
    pub const FLETCHER: &str = "minecraft:fletcher";
    pub const LEATHERWORKER: &str = "minecraft:leatherworker";
    pub const LIBRARIAN: &str = "minecraft:librarian";
    pub const MASON: &str = "minecraft:mason";
    pub const NITWIT: &str = "minecraft:nitwit";
    pub const NONE: &str = "minecraft:none";
    pub const SHEPHERD: &str = "minecraft:shepherd";
    pub const TOOLSMITH: &str = "minecraft:toolsmith";
    pub const WEAPONSMITH: &str = "minecraft:weaponsmith";
}

/// `org.bukkit.entity.Villager$Type`.
pub mod villager_type {
    /// bukkit: org/bukkit/entity/Villager$Type.values
    pub fn values() -> &'static [&'static str] {
        &["DESERT", "JUNGLE", "PLAINS", "SAVANNA", "SNOW", "SWAMP", "TAIGA"]
    }
    /// bukkit: org/bukkit/entity/Villager$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DESERT: &str = "minecraft:desert";
    pub const JUNGLE: &str = "minecraft:jungle";
    pub const PLAINS: &str = "minecraft:plains";
    pub const SAVANNA: &str = "minecraft:savanna";
    pub const SNOW: &str = "minecraft:snow";
    pub const SWAMP: &str = "minecraft:swamp";
    pub const TAIGA: &str = "minecraft:taiga";
}

/// `org.bukkit.entity.Warden$AngerLevel`.
pub mod warden_anger_level {
    /// bukkit: org/bukkit/entity/Warden$AngerLevel.values
    pub fn values() -> &'static [&'static str] {
        &["CALM", "AGITATED", "ANGRY"]
    }
    /// bukkit: org/bukkit/entity/Warden$AngerLevel.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CALM: &str = "CALM";
    pub const AGITATED: &str = "AGITATED";
    pub const ANGRY: &str = "ANGRY";
}

/// `org.bukkit.entity.Wither$Head`.
pub mod wither_head {
    /// bukkit: org/bukkit/entity/Wither$Head.values
    pub fn values() -> &'static [&'static str] {
        &["CENTER", "LEFT", "RIGHT"]
    }
    /// bukkit: org/bukkit/entity/Wither$Head.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CENTER: &str = "CENTER";
    pub const LEFT: &str = "LEFT";
    pub const RIGHT: &str = "RIGHT";
}

/// `org.bukkit.entity.memory.MemoryKey`.
pub mod memory_key {
    /// bukkit: org/bukkit/entity/memory/MemoryKey.values
    pub fn values() -> &'static [&'static str] {
        &["ADMIRING_DISABLED", "ADMIRING_ITEM", "ANGRY_AT", "ATE_RECENTLY", "ATTACK_COOLING_DOWN", "ATTACK_TARGET_COOLDOWN", "CANT_REACH_WALK_TARGET_SINCE", "CHARGE_COOLDOWN_TICKS", "DANCING", "DANGER_DETECTED_RECENTLY", "DISABLE_WALK_TO_ADMIRE_ITEM", "GAZE_COOLDOWN_TICKS", "GOLEM_DETECTED_RECENTLY", "HAS_HUNTING_COOLDOWN", "HEARD_BELL_TIME", "HIDING_PLACE", "HOME", "HUNTED_RECENTLY", "IS_PANICKING", "ITEM_PICKUP_COOLDOWN_TICKS", "JOB_SITE", "LAST_SLEPT", "LAST_WOKEN", "LAST_WORKED_AT_POI", "LIKED_NOTEBLOCK_POSITION", "LIKED_NOTEBLOCK_COOLDOWN_TICKS", "LIKED_PLAYER", "LONG_JUMP_COOLING_DOWN", "LONG_JUMP_MID_JUMP", "MEETING_POINT", "PACIFIED", "PLAY_DEAD_TICKS", "POTENTIAL_JOB_SITE", "RAM_COOLDOWN_TICKS", "SNIFFER_DIGGING", "SNIFFER_HAPPY", "SPEAR_ENGAGE_TIME", "SPEAR_FLEEING_TIME", "TEMPTATION_COOLDOWN_TICKS", "TIME_TRYING_TO_REACH_ADMIRE_ITEM", "TRANSPORT_ITEMS_COOLDOWN_TICKS", "UNIVERSAL_ANGER", "VISIBLE_ADULT_HOGLIN_COUNT", "VISIBLE_ADULT_PIGLIN_COUNT", "SNIFFER_EXPLORED_POSITIONS"]
    }
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ADMIRING_DISABLED: &str = "minecraft:admiring_disabled";
    pub const ADMIRING_ITEM: &str = "minecraft:admiring_item";
    pub const ANGRY_AT: &str = "minecraft:angry_at";
    pub const ATE_RECENTLY: &str = "minecraft:ate_recently";
    pub const ATTACK_COOLING_DOWN: &str = "minecraft:attack_cooling_down";
    pub const ATTACK_TARGET_COOLDOWN: &str = "minecraft:attack_target_cooldown";
    pub const CANT_REACH_WALK_TARGET_SINCE: &str = "minecraft:cant_reach_walk_target_since";
    pub const CHARGE_COOLDOWN_TICKS: &str = "minecraft:charge_cooldown_ticks";
    pub const DANCING: &str = "minecraft:dancing";
    pub const DANGER_DETECTED_RECENTLY: &str = "minecraft:danger_detected_recently";
    pub const DISABLE_WALK_TO_ADMIRE_ITEM: &str = "minecraft:disable_walk_to_admire_item";
    pub const GAZE_COOLDOWN_TICKS: &str = "minecraft:gaze_cooldown_ticks";
    pub const GOLEM_DETECTED_RECENTLY: &str = "minecraft:golem_detected_recently";
    pub const HAS_HUNTING_COOLDOWN: &str = "minecraft:has_hunting_cooldown";
    pub const HEARD_BELL_TIME: &str = "minecraft:heard_bell_time";
    pub const HIDING_PLACE: &str = "minecraft:hiding_place";
    pub const HOME: &str = "minecraft:home";
    pub const HUNTED_RECENTLY: &str = "minecraft:hunted_recently";
    pub const IS_PANICKING: &str = "minecraft:is_panicking";
    pub const ITEM_PICKUP_COOLDOWN_TICKS: &str = "minecraft:item_pickup_cooldown_ticks";
    pub const JOB_SITE: &str = "minecraft:job_site";
    pub const LAST_SLEPT: &str = "minecraft:last_slept";
    pub const LAST_WOKEN: &str = "minecraft:last_woken";
    pub const LAST_WORKED_AT_POI: &str = "minecraft:last_worked_at_poi";
    pub const LIKED_NOTEBLOCK_POSITION: &str = "minecraft:liked_noteblock_position";
    pub const LIKED_NOTEBLOCK_COOLDOWN_TICKS: &str = "minecraft:liked_noteblock_cooldown_ticks";
    pub const LIKED_PLAYER: &str = "minecraft:liked_player";
    pub const LONG_JUMP_COOLING_DOWN: &str = "minecraft:long_jump_cooling_down";
    pub const LONG_JUMP_MID_JUMP: &str = "minecraft:long_jump_mid_jump";
    pub const MEETING_POINT: &str = "minecraft:meeting_point";
    pub const PACIFIED: &str = "minecraft:pacified";
    pub const PLAY_DEAD_TICKS: &str = "minecraft:play_dead_ticks";
    pub const POTENTIAL_JOB_SITE: &str = "minecraft:potential_job_site";
    pub const RAM_COOLDOWN_TICKS: &str = "minecraft:ram_cooldown_ticks";
    pub const SNIFFER_DIGGING: &str = "minecraft:sniffer_digging";
    pub const SNIFFER_HAPPY: &str = "minecraft:sniffer_happy";
    pub const SPEAR_ENGAGE_TIME: &str = "minecraft:spear_engage_time";
    pub const SPEAR_FLEEING_TIME: &str = "minecraft:spear_fleeing_time";
    pub const TEMPTATION_COOLDOWN_TICKS: &str = "minecraft:temptation_cooldown_ticks";
    pub const TIME_TRYING_TO_REACH_ADMIRE_ITEM: &str = "minecraft:time_trying_to_reach_admire_item";
    pub const TRANSPORT_ITEMS_COOLDOWN_TICKS: &str = "minecraft:transport_items_cooldown_ticks";
    pub const UNIVERSAL_ANGER: &str = "minecraft:universal_anger";
    pub const VISIBLE_ADULT_HOGLIN_COUNT: &str = "minecraft:visible_adult_hoglin_count";
    pub const VISIBLE_ADULT_PIGLIN_COUNT: &str = "minecraft:visible_adult_piglin_count";
    pub const SNIFFER_EXPLORED_POSITIONS: &str = "minecraft:sniffer_explored_positions";
}

/// `org.bukkit.event.Event$Result`.
pub mod event_result {
    /// bukkit: org/bukkit/event/Event$Result.values
    pub fn values() -> &'static [&'static str] {
        &["DENY", "DEFAULT", "ALLOW"]
    }
    /// bukkit: org/bukkit/event/Event$Result.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DENY: &str = "DENY";
    pub const DEFAULT: &str = "DEFAULT";
    pub const ALLOW: &str = "ALLOW";
}

/// `org.bukkit.event.EventPriority`.
pub mod event_priority {
    /// bukkit: org/bukkit/event/EventPriority.values
    pub fn values() -> &'static [&'static str] {
        &["LOWEST", "LOW", "NORMAL", "HIGH", "HIGHEST", "MONITOR"]
    }
    /// bukkit: org/bukkit/event/EventPriority.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LOWEST: &str = "LOWEST";
    pub const LOW: &str = "LOW";
    pub const NORMAL: &str = "NORMAL";
    pub const HIGH: &str = "HIGH";
    pub const HIGHEST: &str = "HIGHEST";
    pub const MONITOR: &str = "MONITOR";
}

/// `org.bukkit.event.block.Action`.
pub mod action {
    /// bukkit: org/bukkit/event/block/Action.values
    pub fn values() -> &'static [&'static str] {
        &["LEFT_CLICK_BLOCK", "RIGHT_CLICK_BLOCK", "LEFT_CLICK_AIR", "RIGHT_CLICK_AIR", "PHYSICAL"]
    }
    /// bukkit: org/bukkit/event/block/Action.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LEFT_CLICK_BLOCK: &str = "LEFT_CLICK_BLOCK";
    pub const RIGHT_CLICK_BLOCK: &str = "RIGHT_CLICK_BLOCK";
    pub const LEFT_CLICK_AIR: &str = "LEFT_CLICK_AIR";
    pub const RIGHT_CLICK_AIR: &str = "RIGHT_CLICK_AIR";
    pub const PHYSICAL: &str = "PHYSICAL";
}

/// `org.bukkit.event.block.BlockIgniteEvent$IgniteCause`.
pub mod block_ignite_event_ignite_cause {
    /// bukkit: org/bukkit/event/block/BlockIgniteEvent$IgniteCause.values
    pub fn values() -> &'static [&'static str] {
        &["LAVA", "FLINT_AND_STEEL", "SPREAD", "LIGHTNING", "FIREBALL", "ENDER_CRYSTAL", "EXPLOSION", "ARROW"]
    }
    /// bukkit: org/bukkit/event/block/BlockIgniteEvent$IgniteCause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LAVA: &str = "LAVA";
    pub const FLINT_AND_STEEL: &str = "FLINT_AND_STEEL";
    pub const SPREAD: &str = "SPREAD";
    pub const LIGHTNING: &str = "LIGHTNING";
    pub const FIREBALL: &str = "FIREBALL";
    pub const ENDER_CRYSTAL: &str = "ENDER_CRYSTAL";
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const ARROW: &str = "ARROW";
}

/// `org.bukkit.event.block.CauldronLevelChangeEvent$ChangeReason`.
pub mod cauldron_level_change_event_change_reason {
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent$ChangeReason.values
    pub fn values() -> &'static [&'static str] {
        &["BUCKET_FILL", "BUCKET_EMPTY", "BOTTLE_FILL", "BOTTLE_EMPTY", "BANNER_WASH", "ARMOR_WASH", "SHULKER_WASH", "EXTINGUISH", "EVAPORATE", "NATURAL_FILL", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/block/CauldronLevelChangeEvent$ChangeReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BUCKET_FILL: &str = "BUCKET_FILL";
    pub const BUCKET_EMPTY: &str = "BUCKET_EMPTY";
    pub const BOTTLE_FILL: &str = "BOTTLE_FILL";
    pub const BOTTLE_EMPTY: &str = "BOTTLE_EMPTY";
    pub const BANNER_WASH: &str = "BANNER_WASH";
    pub const ARMOR_WASH: &str = "ARMOR_WASH";
    pub const SHULKER_WASH: &str = "SHULKER_WASH";
    pub const EXTINGUISH: &str = "EXTINGUISH";
    pub const EVAPORATE: &str = "EVAPORATE";
    pub const NATURAL_FILL: &str = "NATURAL_FILL";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.block.TNTPrimeEvent$PrimeCause`.
pub mod tnt_prime_event_prime_cause {
    /// bukkit: org/bukkit/event/block/TNTPrimeEvent$PrimeCause.values
    pub fn values() -> &'static [&'static str] {
        &["FIRE", "REDSTONE", "PLAYER", "EXPLOSION", "PROJECTILE", "BLOCK_BREAK", "DISPENSER"]
    }
    /// bukkit: org/bukkit/event/block/TNTPrimeEvent$PrimeCause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FIRE: &str = "FIRE";
    pub const REDSTONE: &str = "REDSTONE";
    pub const PLAYER: &str = "PLAYER";
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const PROJECTILE: &str = "PROJECTILE";
    pub const BLOCK_BREAK: &str = "BLOCK_BREAK";
    pub const DISPENSER: &str = "DISPENSER";
}

/// `org.bukkit.event.entity.CreatureSpawnEvent$SpawnReason`.
pub mod creature_spawn_event_spawn_reason {
    /// bukkit: org/bukkit/event/entity/CreatureSpawnEvent$SpawnReason.values
    pub fn values() -> &'static [&'static str] {
        &["NATURAL", "JOCKEY", "CHUNK_GEN", "SPAWNER", "TRIAL_SPAWNER", "EGG", "SPAWNER_EGG", "LIGHTNING", "BUILD_SNOWMAN", "BUILD_IRONGOLEM", "BUILD_COPPERGOLEM", "BUILD_WITHER", "VILLAGE_DEFENSE", "VILLAGE_INVASION", "BREEDING", "SLIME_SPLIT", "REINFORCEMENTS", "NETHER_PORTAL", "DISPENSE_EGG", "INFECTION", "CURED", "OCELOT_BABY", "SILVERFISH_BLOCK", "MOUNT", "TRAP", "ENDER_PEARL", "SHOULDER_ENTITY", "DROWNED", "SHEARED", "EXPLOSION", "RAID", "PATROL", "BEEHIVE", "PIGLIN_ZOMBIFIED", "SPELL", "FROZEN", "METAMORPHOSIS", "DUPLICATION", "COMMAND", "ENCHANTMENT", "OMINOUS_ITEM_SPAWNER", "BUCKET", "POTION_EFFECT", "REANIMATE", "REHYDRATION", "CUSTOM", "DEFAULT"]
    }
    /// bukkit: org/bukkit/event/entity/CreatureSpawnEvent$SpawnReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NATURAL: &str = "NATURAL";
    pub const JOCKEY: &str = "JOCKEY";
    pub const CHUNK_GEN: &str = "CHUNK_GEN";
    pub const SPAWNER: &str = "SPAWNER";
    pub const TRIAL_SPAWNER: &str = "TRIAL_SPAWNER";
    pub const EGG: &str = "EGG";
    pub const SPAWNER_EGG: &str = "SPAWNER_EGG";
    pub const LIGHTNING: &str = "LIGHTNING";
    pub const BUILD_SNOWMAN: &str = "BUILD_SNOWMAN";
    pub const BUILD_IRONGOLEM: &str = "BUILD_IRONGOLEM";
    pub const BUILD_COPPERGOLEM: &str = "BUILD_COPPERGOLEM";
    pub const BUILD_WITHER: &str = "BUILD_WITHER";
    pub const VILLAGE_DEFENSE: &str = "VILLAGE_DEFENSE";
    pub const VILLAGE_INVASION: &str = "VILLAGE_INVASION";
    pub const BREEDING: &str = "BREEDING";
    pub const SLIME_SPLIT: &str = "SLIME_SPLIT";
    pub const REINFORCEMENTS: &str = "REINFORCEMENTS";
    pub const NETHER_PORTAL: &str = "NETHER_PORTAL";
    pub const DISPENSE_EGG: &str = "DISPENSE_EGG";
    pub const INFECTION: &str = "INFECTION";
    pub const CURED: &str = "CURED";
    pub const OCELOT_BABY: &str = "OCELOT_BABY";
    pub const SILVERFISH_BLOCK: &str = "SILVERFISH_BLOCK";
    pub const MOUNT: &str = "MOUNT";
    pub const TRAP: &str = "TRAP";
    pub const ENDER_PEARL: &str = "ENDER_PEARL";
    pub const SHOULDER_ENTITY: &str = "SHOULDER_ENTITY";
    pub const DROWNED: &str = "DROWNED";
    pub const SHEARED: &str = "SHEARED";
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const RAID: &str = "RAID";
    pub const PATROL: &str = "PATROL";
    pub const BEEHIVE: &str = "BEEHIVE";
    pub const PIGLIN_ZOMBIFIED: &str = "PIGLIN_ZOMBIFIED";
    pub const SPELL: &str = "SPELL";
    pub const FROZEN: &str = "FROZEN";
    pub const METAMORPHOSIS: &str = "METAMORPHOSIS";
    pub const DUPLICATION: &str = "DUPLICATION";
    pub const COMMAND: &str = "COMMAND";
    pub const ENCHANTMENT: &str = "ENCHANTMENT";
    pub const OMINOUS_ITEM_SPAWNER: &str = "OMINOUS_ITEM_SPAWNER";
    pub const BUCKET: &str = "BUCKET";
    pub const POTION_EFFECT: &str = "POTION_EFFECT";
    pub const REANIMATE: &str = "REANIMATE";
    pub const REHYDRATION: &str = "REHYDRATION";
    pub const CUSTOM: &str = "CUSTOM";
    pub const DEFAULT: &str = "DEFAULT";
}

/// `org.bukkit.event.entity.CreeperPowerEvent$PowerCause`.
pub mod creeper_power_event_power_cause {
    /// bukkit: org/bukkit/event/entity/CreeperPowerEvent$PowerCause.values
    pub fn values() -> &'static [&'static str] {
        &["LIGHTNING", "SET_ON", "SET_OFF"]
    }
    /// bukkit: org/bukkit/event/entity/CreeperPowerEvent$PowerCause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LIGHTNING: &str = "LIGHTNING";
    pub const SET_ON: &str = "SET_ON";
    pub const SET_OFF: &str = "SET_OFF";
}

/// `org.bukkit.event.entity.EntityDamageEvent$DamageCause`.
pub mod entity_damage_event_damage_cause {
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent$DamageCause.values
    pub fn values() -> &'static [&'static str] {
        &["KILL", "WORLD_BORDER", "CONTACT", "ENTITY_ATTACK", "ENTITY_SWEEP_ATTACK", "PROJECTILE", "SUFFOCATION", "FALL", "FIRE", "FIRE_TICK", "MELTING", "LAVA", "DROWNING", "BLOCK_EXPLOSION", "ENTITY_EXPLOSION", "VOID", "LIGHTNING", "SUICIDE", "STARVATION", "POISON", "MAGIC", "WITHER", "FALLING_BLOCK", "THORNS", "DRAGON_BREATH", "FLY_INTO_WALL", "HOT_FLOOR", "CAMPFIRE", "CRAMMING", "DRYOUT", "FREEZE", "SONIC_BOOM", "CUSTOM"]
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent$DamageCause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const KILL: &str = "KILL";
    pub const WORLD_BORDER: &str = "WORLD_BORDER";
    pub const CONTACT: &str = "CONTACT";
    pub const ENTITY_ATTACK: &str = "ENTITY_ATTACK";
    pub const ENTITY_SWEEP_ATTACK: &str = "ENTITY_SWEEP_ATTACK";
    pub const PROJECTILE: &str = "PROJECTILE";
    pub const SUFFOCATION: &str = "SUFFOCATION";
    pub const FALL: &str = "FALL";
    pub const FIRE: &str = "FIRE";
    pub const FIRE_TICK: &str = "FIRE_TICK";
    pub const MELTING: &str = "MELTING";
    pub const LAVA: &str = "LAVA";
    pub const DROWNING: &str = "DROWNING";
    pub const BLOCK_EXPLOSION: &str = "BLOCK_EXPLOSION";
    pub const ENTITY_EXPLOSION: &str = "ENTITY_EXPLOSION";
    pub const VOID: &str = "VOID";
    pub const LIGHTNING: &str = "LIGHTNING";
    pub const SUICIDE: &str = "SUICIDE";
    pub const STARVATION: &str = "STARVATION";
    pub const POISON: &str = "POISON";
    pub const MAGIC: &str = "MAGIC";
    pub const WITHER: &str = "WITHER";
    pub const FALLING_BLOCK: &str = "FALLING_BLOCK";
    pub const THORNS: &str = "THORNS";
    pub const DRAGON_BREATH: &str = "DRAGON_BREATH";
    pub const FLY_INTO_WALL: &str = "FLY_INTO_WALL";
    pub const HOT_FLOOR: &str = "HOT_FLOOR";
    pub const CAMPFIRE: &str = "CAMPFIRE";
    pub const CRAMMING: &str = "CRAMMING";
    pub const DRYOUT: &str = "DRYOUT";
    pub const FREEZE: &str = "FREEZE";
    pub const SONIC_BOOM: &str = "SONIC_BOOM";
    pub const CUSTOM: &str = "CUSTOM";
}

/// `org.bukkit.event.entity.EntityDamageEvent$DamageModifier`.
pub mod entity_damage_event_damage_modifier {
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent$DamageModifier.values
    pub fn values() -> &'static [&'static str] {
        &["BASE", "INVULNERABILITY_REDUCTION", "FREEZING", "HARD_HAT", "BLOCKING", "ARMOR", "RESISTANCE", "MAGIC", "ABSORPTION"]
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent$DamageModifier.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BASE: &str = "BASE";
    pub const INVULNERABILITY_REDUCTION: &str = "INVULNERABILITY_REDUCTION";
    pub const FREEZING: &str = "FREEZING";
    pub const HARD_HAT: &str = "HARD_HAT";
    pub const BLOCKING: &str = "BLOCKING";
    pub const ARMOR: &str = "ARMOR";
    pub const RESISTANCE: &str = "RESISTANCE";
    pub const MAGIC: &str = "MAGIC";
    pub const ABSORPTION: &str = "ABSORPTION";
}

/// `org.bukkit.event.entity.EntityExhaustionEvent$ExhaustionReason`.
pub mod entity_exhaustion_event_exhaustion_reason {
    /// bukkit: org/bukkit/event/entity/EntityExhaustionEvent$ExhaustionReason.values
    pub fn values() -> &'static [&'static str] {
        &["BLOCK_MINED", "HUNGER_EFFECT", "DAMAGED", "ATTACK", "JUMP_SPRINT", "JUMP", "SWIM", "WALK_UNDERWATER", "WALK_ON_WATER", "SPRINT", "CROUCH", "WALK", "REGEN", "ENCHANTMENT_EFFECT", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/entity/EntityExhaustionEvent$ExhaustionReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BLOCK_MINED: &str = "BLOCK_MINED";
    pub const HUNGER_EFFECT: &str = "HUNGER_EFFECT";
    pub const DAMAGED: &str = "DAMAGED";
    pub const ATTACK: &str = "ATTACK";
    pub const JUMP_SPRINT: &str = "JUMP_SPRINT";
    pub const JUMP: &str = "JUMP";
    pub const SWIM: &str = "SWIM";
    pub const WALK_UNDERWATER: &str = "WALK_UNDERWATER";
    pub const WALK_ON_WATER: &str = "WALK_ON_WATER";
    pub const SPRINT: &str = "SPRINT";
    pub const CROUCH: &str = "CROUCH";
    pub const WALK: &str = "WALK";
    pub const REGEN: &str = "REGEN";
    pub const ENCHANTMENT_EFFECT: &str = "ENCHANTMENT_EFFECT";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.entity.EntityKnockbackEvent$KnockbackCause`.
pub mod entity_knockback_event_knockback_cause {
    /// bukkit: org/bukkit/event/entity/EntityKnockbackEvent$KnockbackCause.values
    pub fn values() -> &'static [&'static str] {
        &["DAMAGE", "ENTITY_ATTACK", "EXPLOSION", "SHIELD_BLOCK", "SWEEP_ATTACK", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/entity/EntityKnockbackEvent$KnockbackCause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DAMAGE: &str = "DAMAGE";
    pub const ENTITY_ATTACK: &str = "ENTITY_ATTACK";
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const SHIELD_BLOCK: &str = "SHIELD_BLOCK";
    pub const SWEEP_ATTACK: &str = "SWEEP_ATTACK";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.entity.EntityPotionEffectEvent$Action`.
pub mod entity_potion_effect_event_action {
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent$Action.values
    pub fn values() -> &'static [&'static str] {
        &["ADDED", "CHANGED", "CLEARED", "REMOVED"]
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent$Action.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ADDED: &str = "ADDED";
    pub const CHANGED: &str = "CHANGED";
    pub const CLEARED: &str = "CLEARED";
    pub const REMOVED: &str = "REMOVED";
}

/// `org.bukkit.event.entity.EntityPotionEffectEvent$Cause`.
pub mod entity_potion_effect_event_cause {
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["AREA_EFFECT_CLOUD", "ARROW", "ATTACK", "AXOLOTL", "BEACON", "COMMAND", "CONDUIT", "CONVERSION", "DEATH", "DOLPHIN", "EXPIRATION", "FOOD", "ILLUSION", "MILK", "NAUTILUS", "PATROL_CAPTAIN", "PLUGIN", "POTION_DRINK", "POTION_SPLASH", "SPIDER_SPAWN", "TOTEM", "TURTLE_HELMET", "UNKNOWN", "VILLAGER_TRADE", "WARDEN", "WITHER_ROSE"]
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const AREA_EFFECT_CLOUD: &str = "AREA_EFFECT_CLOUD";
    pub const ARROW: &str = "ARROW";
    pub const ATTACK: &str = "ATTACK";
    pub const AXOLOTL: &str = "AXOLOTL";
    pub const BEACON: &str = "BEACON";
    pub const COMMAND: &str = "COMMAND";
    pub const CONDUIT: &str = "CONDUIT";
    pub const CONVERSION: &str = "CONVERSION";
    pub const DEATH: &str = "DEATH";
    pub const DOLPHIN: &str = "DOLPHIN";
    pub const EXPIRATION: &str = "EXPIRATION";
    pub const FOOD: &str = "FOOD";
    pub const ILLUSION: &str = "ILLUSION";
    pub const MILK: &str = "MILK";
    pub const NAUTILUS: &str = "NAUTILUS";
    pub const PATROL_CAPTAIN: &str = "PATROL_CAPTAIN";
    pub const PLUGIN: &str = "PLUGIN";
    pub const POTION_DRINK: &str = "POTION_DRINK";
    pub const POTION_SPLASH: &str = "POTION_SPLASH";
    pub const SPIDER_SPAWN: &str = "SPIDER_SPAWN";
    pub const TOTEM: &str = "TOTEM";
    pub const TURTLE_HELMET: &str = "TURTLE_HELMET";
    pub const UNKNOWN: &str = "UNKNOWN";
    pub const VILLAGER_TRADE: &str = "VILLAGER_TRADE";
    pub const WARDEN: &str = "WARDEN";
    pub const WITHER_ROSE: &str = "WITHER_ROSE";
}

/// `org.bukkit.event.entity.EntityRegainHealthEvent$RegainReason`.
pub mod entity_regain_health_event_regain_reason {
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent$RegainReason.values
    pub fn values() -> &'static [&'static str] {
        &["REGEN", "SATIATED", "EATING", "ENDER_CRYSTAL", "MAGIC", "MAGIC_REGEN", "WITHER_SPAWN", "WITHER", "CUSTOM"]
    }
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent$RegainReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const REGEN: &str = "REGEN";
    pub const SATIATED: &str = "SATIATED";
    pub const EATING: &str = "EATING";
    pub const ENDER_CRYSTAL: &str = "ENDER_CRYSTAL";
    pub const MAGIC: &str = "MAGIC";
    pub const MAGIC_REGEN: &str = "MAGIC_REGEN";
    pub const WITHER_SPAWN: &str = "WITHER_SPAWN";
    pub const WITHER: &str = "WITHER";
    pub const CUSTOM: &str = "CUSTOM";
}

/// `org.bukkit.event.entity.EntityRemoveEvent$Cause`.
pub mod entity_remove_event_cause {
    /// bukkit: org/bukkit/event/entity/EntityRemoveEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["DEATH", "DESPAWN", "DROP", "ENTER_BLOCK", "EXPLODE", "HIT", "MERGE", "OUT_OF_WORLD", "PICKUP", "PLAYER_QUIT", "PLUGIN", "TRANSFORMATION", "UNLOAD", "DISCARD"]
    }
    /// bukkit: org/bukkit/event/entity/EntityRemoveEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DEATH: &str = "DEATH";
    pub const DESPAWN: &str = "DESPAWN";
    pub const DROP: &str = "DROP";
    pub const ENTER_BLOCK: &str = "ENTER_BLOCK";
    pub const EXPLODE: &str = "EXPLODE";
    pub const HIT: &str = "HIT";
    pub const MERGE: &str = "MERGE";
    pub const OUT_OF_WORLD: &str = "OUT_OF_WORLD";
    pub const PICKUP: &str = "PICKUP";
    pub const PLAYER_QUIT: &str = "PLAYER_QUIT";
    pub const PLUGIN: &str = "PLUGIN";
    pub const TRANSFORMATION: &str = "TRANSFORMATION";
    pub const UNLOAD: &str = "UNLOAD";
    pub const DISCARD: &str = "DISCARD";
}

/// `org.bukkit.event.entity.EntityTargetEvent$TargetReason`.
pub mod entity_target_event_target_reason {
    /// bukkit: org/bukkit/event/entity/EntityTargetEvent$TargetReason.values
    pub fn values() -> &'static [&'static str] {
        &["TARGET_DIED", "CLOSEST_PLAYER", "TARGET_ATTACKED_ENTITY", "PIG_ZOMBIE_TARGET", "FORGOT_TARGET", "TARGET_ATTACKED_OWNER", "OWNER_ATTACKED_TARGET", "RANDOM_TARGET", "DEFEND_VILLAGE", "TARGET_ATTACKED_NEARBY_ENTITY", "REINFORCEMENT_TARGET", "COLLISION", "CUSTOM", "CLOSEST_ENTITY", "FOLLOW_LEADER", "TEMPT", "TARGET_OTHER_LEVEL", "TARGET_INVALID", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/entity/EntityTargetEvent$TargetReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const TARGET_DIED: &str = "TARGET_DIED";
    pub const CLOSEST_PLAYER: &str = "CLOSEST_PLAYER";
    pub const TARGET_ATTACKED_ENTITY: &str = "TARGET_ATTACKED_ENTITY";
    pub const PIG_ZOMBIE_TARGET: &str = "PIG_ZOMBIE_TARGET";
    pub const FORGOT_TARGET: &str = "FORGOT_TARGET";
    pub const TARGET_ATTACKED_OWNER: &str = "TARGET_ATTACKED_OWNER";
    pub const OWNER_ATTACKED_TARGET: &str = "OWNER_ATTACKED_TARGET";
    pub const RANDOM_TARGET: &str = "RANDOM_TARGET";
    pub const DEFEND_VILLAGE: &str = "DEFEND_VILLAGE";
    pub const TARGET_ATTACKED_NEARBY_ENTITY: &str = "TARGET_ATTACKED_NEARBY_ENTITY";
    pub const REINFORCEMENT_TARGET: &str = "REINFORCEMENT_TARGET";
    pub const COLLISION: &str = "COLLISION";
    pub const CUSTOM: &str = "CUSTOM";
    pub const CLOSEST_ENTITY: &str = "CLOSEST_ENTITY";
    pub const FOLLOW_LEADER: &str = "FOLLOW_LEADER";
    pub const TEMPT: &str = "TEMPT";
    pub const TARGET_OTHER_LEVEL: &str = "TARGET_OTHER_LEVEL";
    pub const TARGET_INVALID: &str = "TARGET_INVALID";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.entity.EntityTransformEvent$TransformReason`.
pub mod entity_transform_event_transform_reason {
    /// bukkit: org/bukkit/event/entity/EntityTransformEvent$TransformReason.values
    pub fn values() -> &'static [&'static str] {
        &["CURED", "FROZEN", "INFECTION", "DROWNED", "SHEARED", "LIGHTNING", "SPLIT", "PIGLIN_ZOMBIFIED", "METAMORPHOSIS", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/entity/EntityTransformEvent$TransformReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CURED: &str = "CURED";
    pub const FROZEN: &str = "FROZEN";
    pub const INFECTION: &str = "INFECTION";
    pub const DROWNED: &str = "DROWNED";
    pub const SHEARED: &str = "SHEARED";
    pub const LIGHTNING: &str = "LIGHTNING";
    pub const SPLIT: &str = "SPLIT";
    pub const PIGLIN_ZOMBIFIED: &str = "PIGLIN_ZOMBIFIED";
    pub const METAMORPHOSIS: &str = "METAMORPHOSIS";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.entity.EntityUnleashEvent$UnleashReason`.
pub mod entity_unleash_event_unleash_reason {
    /// bukkit: org/bukkit/event/entity/EntityUnleashEvent$UnleashReason.values
    pub fn values() -> &'static [&'static str] {
        &["HOLDER_GONE", "PLAYER_UNLEASH", "DISTANCE", "LEASHED_GONE", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/entity/EntityUnleashEvent$UnleashReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const HOLDER_GONE: &str = "HOLDER_GONE";
    pub const PLAYER_UNLEASH: &str = "PLAYER_UNLEASH";
    pub const DISTANCE: &str = "DISTANCE";
    pub const LEASHED_GONE: &str = "LEASHED_GONE";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.entity.VillagerCareerChangeEvent$ChangeReason`.
pub mod villager_career_change_event_change_reason {
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent$ChangeReason.values
    pub fn values() -> &'static [&'static str] {
        &["LOSING_JOB", "EMPLOYED"]
    }
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent$ChangeReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LOSING_JOB: &str = "LOSING_JOB";
    pub const EMPLOYED: &str = "EMPLOYED";
}

/// `org.bukkit.event.hanging.HangingBreakEvent$RemoveCause`.
pub mod hanging_break_event_remove_cause {
    /// bukkit: org/bukkit/event/hanging/HangingBreakEvent$RemoveCause.values
    pub fn values() -> &'static [&'static str] {
        &["ENTITY", "EXPLOSION", "OBSTRUCTION", "PHYSICS", "DEFAULT"]
    }
    /// bukkit: org/bukkit/event/hanging/HangingBreakEvent$RemoveCause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ENTITY: &str = "ENTITY";
    pub const EXPLOSION: &str = "EXPLOSION";
    pub const OBSTRUCTION: &str = "OBSTRUCTION";
    pub const PHYSICS: &str = "PHYSICS";
    pub const DEFAULT: &str = "DEFAULT";
}

/// `org.bukkit.event.inventory.ClickType`.
pub mod click_type {
    /// bukkit: org/bukkit/event/inventory/ClickType.values
    pub fn values() -> &'static [&'static str] {
        &["LEFT", "SHIFT_LEFT", "RIGHT", "SHIFT_RIGHT", "WINDOW_BORDER_LEFT", "WINDOW_BORDER_RIGHT", "MIDDLE", "NUMBER_KEY", "DOUBLE_CLICK", "DROP", "CONTROL_DROP", "CREATIVE", "SWAP_OFFHAND", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/inventory/ClickType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LEFT: &str = "LEFT";
    pub const SHIFT_LEFT: &str = "SHIFT_LEFT";
    pub const RIGHT: &str = "RIGHT";
    pub const SHIFT_RIGHT: &str = "SHIFT_RIGHT";
    pub const WINDOW_BORDER_LEFT: &str = "WINDOW_BORDER_LEFT";
    pub const WINDOW_BORDER_RIGHT: &str = "WINDOW_BORDER_RIGHT";
    pub const MIDDLE: &str = "MIDDLE";
    pub const NUMBER_KEY: &str = "NUMBER_KEY";
    pub const DOUBLE_CLICK: &str = "DOUBLE_CLICK";
    pub const DROP: &str = "DROP";
    pub const CONTROL_DROP: &str = "CONTROL_DROP";
    pub const CREATIVE: &str = "CREATIVE";
    pub const SWAP_OFFHAND: &str = "SWAP_OFFHAND";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.inventory.DragType`.
pub mod drag_type {
    /// bukkit: org/bukkit/event/inventory/DragType.values
    pub fn values() -> &'static [&'static str] {
        &["SINGLE", "EVEN"]
    }
    /// bukkit: org/bukkit/event/inventory/DragType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SINGLE: &str = "SINGLE";
    pub const EVEN: &str = "EVEN";
}

/// `org.bukkit.event.inventory.HopperInventorySearchEvent$ContainerType`.
pub mod hopper_inventory_search_event_container_type {
    /// bukkit: org/bukkit/event/inventory/HopperInventorySearchEvent$ContainerType.values
    pub fn values() -> &'static [&'static str] {
        &["SOURCE", "DESTINATION"]
    }
    /// bukkit: org/bukkit/event/inventory/HopperInventorySearchEvent$ContainerType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SOURCE: &str = "SOURCE";
    pub const DESTINATION: &str = "DESTINATION";
}

/// `org.bukkit.event.inventory.InventoryAction`.
pub mod inventory_action {
    /// bukkit: org/bukkit/event/inventory/InventoryAction.values
    pub fn values() -> &'static [&'static str] {
        &["NOTHING", "PICKUP_ALL", "PICKUP_SOME", "PICKUP_HALF", "PICKUP_ONE", "PLACE_ALL", "PLACE_SOME", "PLACE_ONE", "SWAP_WITH_CURSOR", "DROP_ALL_CURSOR", "DROP_ONE_CURSOR", "DROP_ALL_SLOT", "DROP_ONE_SLOT", "MOVE_TO_OTHER_INVENTORY", "HOTBAR_MOVE_AND_READD", "HOTBAR_SWAP", "CLONE_STACK", "COLLECT_TO_CURSOR", "UNKNOWN", "PICKUP_FROM_BUNDLE", "PICKUP_ALL_INTO_BUNDLE", "PICKUP_SOME_INTO_BUNDLE", "PLACE_FROM_BUNDLE", "PLACE_ALL_INTO_BUNDLE", "PLACE_SOME_INTO_BUNDLE"]
    }
    /// bukkit: org/bukkit/event/inventory/InventoryAction.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NOTHING: &str = "NOTHING";
    pub const PICKUP_ALL: &str = "PICKUP_ALL";
    pub const PICKUP_SOME: &str = "PICKUP_SOME";
    pub const PICKUP_HALF: &str = "PICKUP_HALF";
    pub const PICKUP_ONE: &str = "PICKUP_ONE";
    pub const PLACE_ALL: &str = "PLACE_ALL";
    pub const PLACE_SOME: &str = "PLACE_SOME";
    pub const PLACE_ONE: &str = "PLACE_ONE";
    pub const SWAP_WITH_CURSOR: &str = "SWAP_WITH_CURSOR";
    pub const DROP_ALL_CURSOR: &str = "DROP_ALL_CURSOR";
    pub const DROP_ONE_CURSOR: &str = "DROP_ONE_CURSOR";
    pub const DROP_ALL_SLOT: &str = "DROP_ALL_SLOT";
    pub const DROP_ONE_SLOT: &str = "DROP_ONE_SLOT";
    pub const MOVE_TO_OTHER_INVENTORY: &str = "MOVE_TO_OTHER_INVENTORY";
    pub const HOTBAR_MOVE_AND_READD: &str = "HOTBAR_MOVE_AND_READD";
    pub const HOTBAR_SWAP: &str = "HOTBAR_SWAP";
    pub const CLONE_STACK: &str = "CLONE_STACK";
    pub const COLLECT_TO_CURSOR: &str = "COLLECT_TO_CURSOR";
    pub const UNKNOWN: &str = "UNKNOWN";
    pub const PICKUP_FROM_BUNDLE: &str = "PICKUP_FROM_BUNDLE";
    pub const PICKUP_ALL_INTO_BUNDLE: &str = "PICKUP_ALL_INTO_BUNDLE";
    pub const PICKUP_SOME_INTO_BUNDLE: &str = "PICKUP_SOME_INTO_BUNDLE";
    pub const PLACE_FROM_BUNDLE: &str = "PLACE_FROM_BUNDLE";
    pub const PLACE_ALL_INTO_BUNDLE: &str = "PLACE_ALL_INTO_BUNDLE";
    pub const PLACE_SOME_INTO_BUNDLE: &str = "PLACE_SOME_INTO_BUNDLE";
}

/// `org.bukkit.event.inventory.InventoryCloseEvent$Reason`.
pub mod inventory_close_event_reason {
    /// bukkit: org/bukkit/event/inventory/InventoryCloseEvent$Reason.values
    pub fn values() -> &'static [&'static str] {
        &["UNKNOWN", "TELEPORT", "CANT_USE", "UNLOADED", "OPEN_NEW", "PLAYER", "DISCONNECT", "DEATH", "PLUGIN"]
    }
    /// bukkit: org/bukkit/event/inventory/InventoryCloseEvent$Reason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UNKNOWN: &str = "UNKNOWN";
    pub const TELEPORT: &str = "TELEPORT";
    pub const CANT_USE: &str = "CANT_USE";
    pub const UNLOADED: &str = "UNLOADED";
    pub const OPEN_NEW: &str = "OPEN_NEW";
    pub const PLAYER: &str = "PLAYER";
    pub const DISCONNECT: &str = "DISCONNECT";
    pub const DEATH: &str = "DEATH";
    pub const PLUGIN: &str = "PLUGIN";
}

/// `org.bukkit.event.inventory.InventoryType$SlotType`.
pub mod inventory_type_slot_type {
    /// bukkit: org/bukkit/event/inventory/InventoryType$SlotType.values
    pub fn values() -> &'static [&'static str] {
        &["RESULT", "CRAFTING", "ARMOR", "CONTAINER", "QUICKBAR", "OUTSIDE", "FUEL"]
    }
    /// bukkit: org/bukkit/event/inventory/InventoryType$SlotType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const RESULT: &str = "RESULT";
    pub const CRAFTING: &str = "CRAFTING";
    pub const ARMOR: &str = "ARMOR";
    pub const CONTAINER: &str = "CONTAINER";
    pub const QUICKBAR: &str = "QUICKBAR";
    pub const OUTSIDE: &str = "OUTSIDE";
    pub const FUEL: &str = "FUEL";
}

/// `org.bukkit.event.inventory.InventoryType`.
pub mod inventory_type {
    /// bukkit: org/bukkit/event/inventory/InventoryType.values
    pub fn values() -> &'static [&'static str] {
        &["CHEST", "DISPENSER", "DROPPER", "FURNACE", "WORKBENCH", "CRAFTING", "ENCHANTING", "BREWING", "PLAYER", "CREATIVE", "MERCHANT", "ENDER_CHEST", "ANVIL", "SMITHING", "BEACON", "HOPPER", "SHULKER_BOX", "BARREL", "BLAST_FURNACE", "LECTERN", "SMOKER", "LOOM", "CARTOGRAPHY", "GRINDSTONE", "STONECUTTER", "COMPOSTER", "CHISELED_BOOKSHELF", "SHELF", "JUKEBOX", "DECORATED_POT", "CRAFTER", "SMITHING_NEW"]
    }
    /// bukkit: org/bukkit/event/inventory/InventoryType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CHEST: &str = "CHEST";
    pub const DISPENSER: &str = "DISPENSER";
    pub const DROPPER: &str = "DROPPER";
    pub const FURNACE: &str = "FURNACE";
    pub const WORKBENCH: &str = "WORKBENCH";
    pub const CRAFTING: &str = "CRAFTING";
    pub const ENCHANTING: &str = "ENCHANTING";
    pub const BREWING: &str = "BREWING";
    pub const PLAYER: &str = "PLAYER";
    pub const CREATIVE: &str = "CREATIVE";
    pub const MERCHANT: &str = "MERCHANT";
    pub const ENDER_CHEST: &str = "ENDER_CHEST";
    pub const ANVIL: &str = "ANVIL";
    pub const SMITHING: &str = "SMITHING";
    pub const BEACON: &str = "BEACON";
    pub const HOPPER: &str = "HOPPER";
    pub const SHULKER_BOX: &str = "SHULKER_BOX";
    pub const BARREL: &str = "BARREL";
    pub const BLAST_FURNACE: &str = "BLAST_FURNACE";
    pub const LECTERN: &str = "LECTERN";
    pub const SMOKER: &str = "SMOKER";
    pub const LOOM: &str = "LOOM";
    pub const CARTOGRAPHY: &str = "CARTOGRAPHY";
    pub const GRINDSTONE: &str = "GRINDSTONE";
    pub const STONECUTTER: &str = "STONECUTTER";
    pub const COMPOSTER: &str = "COMPOSTER";
    pub const CHISELED_BOOKSHELF: &str = "CHISELED_BOOKSHELF";
    pub const SHELF: &str = "SHELF";
    pub const JUKEBOX: &str = "JUKEBOX";
    pub const DECORATED_POT: &str = "DECORATED_POT";
    pub const CRAFTER: &str = "CRAFTER";
    pub const SMITHING_NEW: &str = "SMITHING_NEW";
}

/// `org.bukkit.event.player.AsyncPlayerPreLoginEvent$Result`.
pub mod async_player_pre_login_event_result {
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent$Result.values
    pub fn values() -> &'static [&'static str] {
        &["ALLOWED", "KICK_FULL", "KICK_BANNED", "KICK_WHITELIST", "KICK_OTHER"]
    }
    /// bukkit: org/bukkit/event/player/AsyncPlayerPreLoginEvent$Result.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALLOWED: &str = "ALLOWED";
    pub const KICK_FULL: &str = "KICK_FULL";
    pub const KICK_BANNED: &str = "KICK_BANNED";
    pub const KICK_WHITELIST: &str = "KICK_WHITELIST";
    pub const KICK_OTHER: &str = "KICK_OTHER";
}

/// `org.bukkit.event.player.PlayerAnimationType`.
pub mod player_animation_type {
    /// bukkit: org/bukkit/event/player/PlayerAnimationType.values
    pub fn values() -> &'static [&'static str] {
        &["ARM_SWING", "OFF_ARM_SWING"]
    }
    /// bukkit: org/bukkit/event/player/PlayerAnimationType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ARM_SWING: &str = "ARM_SWING";
    pub const OFF_ARM_SWING: &str = "OFF_ARM_SWING";
}

/// `org.bukkit.event.player.PlayerBedEnterEvent$BedEnterResult`.
pub mod player_bed_enter_event_bed_enter_result {
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent$BedEnterResult.values
    pub fn values() -> &'static [&'static str] {
        &["OK", "NOT_POSSIBLE_HERE", "NOT_POSSIBLE_NOW", "TOO_FAR_AWAY", "OBSTRUCTED", "NOT_SAFE", "OTHER_PROBLEM", "EXPLOSION"]
    }
    /// bukkit: org/bukkit/event/player/PlayerBedEnterEvent$BedEnterResult.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const OK: &str = "OK";
    pub const NOT_POSSIBLE_HERE: &str = "NOT_POSSIBLE_HERE";
    pub const NOT_POSSIBLE_NOW: &str = "NOT_POSSIBLE_NOW";
    pub const TOO_FAR_AWAY: &str = "TOO_FAR_AWAY";
    pub const OBSTRUCTED: &str = "OBSTRUCTED";
    pub const NOT_SAFE: &str = "NOT_SAFE";
    pub const OTHER_PROBLEM: &str = "OTHER_PROBLEM";
    pub const EXPLOSION: &str = "EXPLOSION";
}

/// `org.bukkit.event.player.PlayerExpCooldownChangeEvent$ChangeReason`.
pub mod player_exp_cooldown_change_event_change_reason {
    /// bukkit: org/bukkit/event/player/PlayerExpCooldownChangeEvent$ChangeReason.values
    pub fn values() -> &'static [&'static str] {
        &["PICKUP_ORB", "PLUGIN"]
    }
    /// bukkit: org/bukkit/event/player/PlayerExpCooldownChangeEvent$ChangeReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PICKUP_ORB: &str = "PICKUP_ORB";
    pub const PLUGIN: &str = "PLUGIN";
}

/// `org.bukkit.event.player.PlayerFishEvent$State`.
pub mod player_fish_event_state {
    /// bukkit: org/bukkit/event/player/PlayerFishEvent$State.values
    pub fn values() -> &'static [&'static str] {
        &["FISHING", "CAUGHT_FISH", "CAUGHT_ENTITY", "IN_GROUND", "FAILED_ATTEMPT", "REEL_IN", "BITE", "LURED"]
    }
    /// bukkit: org/bukkit/event/player/PlayerFishEvent$State.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FISHING: &str = "FISHING";
    pub const CAUGHT_FISH: &str = "CAUGHT_FISH";
    pub const CAUGHT_ENTITY: &str = "CAUGHT_ENTITY";
    pub const IN_GROUND: &str = "IN_GROUND";
    pub const FAILED_ATTEMPT: &str = "FAILED_ATTEMPT";
    pub const REEL_IN: &str = "REEL_IN";
    pub const BITE: &str = "BITE";
    pub const LURED: &str = "LURED";
}

/// `org.bukkit.event.player.PlayerGameModeChangeEvent$Cause`.
pub mod player_game_mode_change_event_cause {
    /// bukkit: org/bukkit/event/player/PlayerGameModeChangeEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["PLUGIN", "COMMAND", "DEFAULT_GAMEMODE", "HARDCORE_DEATH", "GAMEMODE_SWITCHER", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/player/PlayerGameModeChangeEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PLUGIN: &str = "PLUGIN";
    pub const COMMAND: &str = "COMMAND";
    pub const DEFAULT_GAMEMODE: &str = "DEFAULT_GAMEMODE";
    pub const HARDCORE_DEATH: &str = "HARDCORE_DEATH";
    pub const GAMEMODE_SWITCHER: &str = "GAMEMODE_SWITCHER";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.player.PlayerKickEvent$Cause`.
pub mod player_kick_event_cause {
    /// bukkit: org/bukkit/event/player/PlayerKickEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["PLUGIN", "WHITELIST", "BANNED", "IP_BANNED", "KICKED", "FLYING_PLAYER", "FLYING_VEHICLE", "TIMEOUT", "IDLING", "INVALID_VEHICLE_MOVEMENT", "INVALID_PLAYER_MOVEMENT", "INVALID_ENTITY_ATTACKED", "INVALID_PAYLOAD", "INVALID_COOKIE", "SPAM", "ILLEGAL_ACTION", "ILLEGAL_CHARACTERS", "OUT_OF_ORDER_CHAT", "UNSIGNED_CHAT", "CHAT_VALIDATION_FAILED", "EXPIRED_PROFILE_PUBLIC_KEY", "INVALID_PUBLIC_KEY_SIGNATURE", "TOO_MANY_PENDING_CHATS", "SELF_INTERACTION", "DUPLICATE_LOGIN", "RESOURCE_PACK_REJECTION", "RESTART_COMMAND", "UNKNOWN", "KICK_COMMAND"]
    }
    /// bukkit: org/bukkit/event/player/PlayerKickEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PLUGIN: &str = "PLUGIN";
    pub const WHITELIST: &str = "WHITELIST";
    pub const BANNED: &str = "BANNED";
    pub const IP_BANNED: &str = "IP_BANNED";
    pub const KICKED: &str = "KICKED";
    pub const FLYING_PLAYER: &str = "FLYING_PLAYER";
    pub const FLYING_VEHICLE: &str = "FLYING_VEHICLE";
    pub const TIMEOUT: &str = "TIMEOUT";
    pub const IDLING: &str = "IDLING";
    pub const INVALID_VEHICLE_MOVEMENT: &str = "INVALID_VEHICLE_MOVEMENT";
    pub const INVALID_PLAYER_MOVEMENT: &str = "INVALID_PLAYER_MOVEMENT";
    pub const INVALID_ENTITY_ATTACKED: &str = "INVALID_ENTITY_ATTACKED";
    pub const INVALID_PAYLOAD: &str = "INVALID_PAYLOAD";
    pub const INVALID_COOKIE: &str = "INVALID_COOKIE";
    pub const SPAM: &str = "SPAM";
    pub const ILLEGAL_ACTION: &str = "ILLEGAL_ACTION";
    pub const ILLEGAL_CHARACTERS: &str = "ILLEGAL_CHARACTERS";
    pub const OUT_OF_ORDER_CHAT: &str = "OUT_OF_ORDER_CHAT";
    pub const UNSIGNED_CHAT: &str = "UNSIGNED_CHAT";
    pub const CHAT_VALIDATION_FAILED: &str = "CHAT_VALIDATION_FAILED";
    pub const EXPIRED_PROFILE_PUBLIC_KEY: &str = "EXPIRED_PROFILE_PUBLIC_KEY";
    pub const INVALID_PUBLIC_KEY_SIGNATURE: &str = "INVALID_PUBLIC_KEY_SIGNATURE";
    pub const TOO_MANY_PENDING_CHATS: &str = "TOO_MANY_PENDING_CHATS";
    pub const SELF_INTERACTION: &str = "SELF_INTERACTION";
    pub const DUPLICATE_LOGIN: &str = "DUPLICATE_LOGIN";
    pub const RESOURCE_PACK_REJECTION: &str = "RESOURCE_PACK_REJECTION";
    pub const RESTART_COMMAND: &str = "RESTART_COMMAND";
    pub const UNKNOWN: &str = "UNKNOWN";
    pub const KICK_COMMAND: &str = "KICK_COMMAND";
}

/// `org.bukkit.event.player.PlayerLoginEvent$Result`.
pub mod player_login_event_result {
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent$Result.values
    pub fn values() -> &'static [&'static str] {
        &["ALLOWED", "KICK_FULL", "KICK_BANNED", "KICK_WHITELIST", "KICK_OTHER"]
    }
    /// bukkit: org/bukkit/event/player/PlayerLoginEvent$Result.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALLOWED: &str = "ALLOWED";
    pub const KICK_FULL: &str = "KICK_FULL";
    pub const KICK_BANNED: &str = "KICK_BANNED";
    pub const KICK_WHITELIST: &str = "KICK_WHITELIST";
    pub const KICK_OTHER: &str = "KICK_OTHER";
}

/// `org.bukkit.event.player.PlayerPreLoginEvent$Result`.
pub mod player_pre_login_event_result {
    /// bukkit: org/bukkit/event/player/PlayerPreLoginEvent$Result.values
    pub fn values() -> &'static [&'static str] {
        &["ALLOWED", "KICK_FULL", "KICK_BANNED", "KICK_WHITELIST", "KICK_OTHER"]
    }
    /// bukkit: org/bukkit/event/player/PlayerPreLoginEvent$Result.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALLOWED: &str = "ALLOWED";
    pub const KICK_FULL: &str = "KICK_FULL";
    pub const KICK_BANNED: &str = "KICK_BANNED";
    pub const KICK_WHITELIST: &str = "KICK_WHITELIST";
    pub const KICK_OTHER: &str = "KICK_OTHER";
}

/// `org.bukkit.event.player.PlayerQuitEvent$QuitReason`.
pub mod player_quit_event_quit_reason {
    /// bukkit: org/bukkit/event/player/PlayerQuitEvent$QuitReason.values
    pub fn values() -> &'static [&'static str] {
        &["DISCONNECTED", "KICKED", "TIMED_OUT", "ERRONEOUS_STATE"]
    }
    /// bukkit: org/bukkit/event/player/PlayerQuitEvent$QuitReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DISCONNECTED: &str = "DISCONNECTED";
    pub const KICKED: &str = "KICKED";
    pub const TIMED_OUT: &str = "TIMED_OUT";
    pub const ERRONEOUS_STATE: &str = "ERRONEOUS_STATE";
}

/// `org.bukkit.event.player.PlayerRecipeBookSettingsChangeEvent$RecipeBookType`.
pub mod player_recipe_book_settings_change_event_recipe_book_type {
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookSettingsChangeEvent$RecipeBookType.values
    pub fn values() -> &'static [&'static str] {
        &["CRAFTING", "FURNACE", "BLAST_FURNACE", "SMOKER"]
    }
    /// bukkit: org/bukkit/event/player/PlayerRecipeBookSettingsChangeEvent$RecipeBookType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CRAFTING: &str = "CRAFTING";
    pub const FURNACE: &str = "FURNACE";
    pub const BLAST_FURNACE: &str = "BLAST_FURNACE";
    pub const SMOKER: &str = "SMOKER";
}

/// `org.bukkit.event.player.PlayerResourcePackStatusEvent$Status`.
pub mod player_resource_pack_status_event_status {
    /// bukkit: org/bukkit/event/player/PlayerResourcePackStatusEvent$Status.values
    pub fn values() -> &'static [&'static str] {
        &["SUCCESSFULLY_LOADED", "DECLINED", "FAILED_DOWNLOAD", "ACCEPTED", "DOWNLOADED", "INVALID_URL", "FAILED_RELOAD", "DISCARDED"]
    }
    /// bukkit: org/bukkit/event/player/PlayerResourcePackStatusEvent$Status.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SUCCESSFULLY_LOADED: &str = "SUCCESSFULLY_LOADED";
    pub const DECLINED: &str = "DECLINED";
    pub const FAILED_DOWNLOAD: &str = "FAILED_DOWNLOAD";
    pub const ACCEPTED: &str = "ACCEPTED";
    pub const DOWNLOADED: &str = "DOWNLOADED";
    pub const INVALID_URL: &str = "INVALID_URL";
    pub const FAILED_RELOAD: &str = "FAILED_RELOAD";
    pub const DISCARDED: &str = "DISCARDED";
}

/// `org.bukkit.event.player.PlayerRespawnEvent$RespawnFlag`.
pub mod player_respawn_event_respawn_flag {
    /// bukkit: org/bukkit/event/player/PlayerRespawnEvent$RespawnFlag.values
    pub fn values() -> &'static [&'static str] {
        &["BED_SPAWN", "ANCHOR_SPAWN", "END_PORTAL"]
    }
    /// bukkit: org/bukkit/event/player/PlayerRespawnEvent$RespawnFlag.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BED_SPAWN: &str = "BED_SPAWN";
    pub const ANCHOR_SPAWN: &str = "ANCHOR_SPAWN";
    pub const END_PORTAL: &str = "END_PORTAL";
}

/// `org.bukkit.event.player.PlayerRespawnEvent$RespawnReason`.
pub mod player_respawn_event_respawn_reason {
    /// bukkit: org/bukkit/event/player/PlayerRespawnEvent$RespawnReason.values
    pub fn values() -> &'static [&'static str] {
        &["DEATH", "END_PORTAL", "PLUGIN"]
    }
    /// bukkit: org/bukkit/event/player/PlayerRespawnEvent$RespawnReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const DEATH: &str = "DEATH";
    pub const END_PORTAL: &str = "END_PORTAL";
    pub const PLUGIN: &str = "PLUGIN";
}

/// `org.bukkit.event.player.PlayerSignOpenEvent$Cause`.
pub mod player_sign_open_event_cause {
    /// bukkit: org/bukkit/event/player/PlayerSignOpenEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["INTERACT", "PLACE", "PLUGIN", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/player/PlayerSignOpenEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const INTERACT: &str = "INTERACT";
    pub const PLACE: &str = "PLACE";
    pub const PLUGIN: &str = "PLUGIN";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.player.PlayerSpawnChangeEvent$Cause`.
pub mod player_spawn_change_event_cause {
    /// bukkit: org/bukkit/event/player/PlayerSpawnChangeEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["COMMAND", "BED", "RESPAWN_ANCHOR", "PLUGIN", "RESET", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/player/PlayerSpawnChangeEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMAND: &str = "COMMAND";
    pub const BED: &str = "BED";
    pub const RESPAWN_ANCHOR: &str = "RESPAWN_ANCHOR";
    pub const PLUGIN: &str = "PLUGIN";
    pub const RESET: &str = "RESET";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.player.PlayerTeleportEvent$TeleportCause`.
pub mod player_teleport_event_teleport_cause {
    /// bukkit: org/bukkit/event/player/PlayerTeleportEvent$TeleportCause.values
    pub fn values() -> &'static [&'static str] {
        &["ENDER_PEARL", "COMMAND", "PLUGIN", "NETHER_PORTAL", "END_PORTAL", "SPECTATE", "END_GATEWAY", "CONSUMABLE_EFFECT", "DISMOUNT", "EXIT_BED", "UNKNOWN", "CHORUS_FRUIT"]
    }
    /// bukkit: org/bukkit/event/player/PlayerTeleportEvent$TeleportCause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ENDER_PEARL: &str = "ENDER_PEARL";
    pub const COMMAND: &str = "COMMAND";
    pub const PLUGIN: &str = "PLUGIN";
    pub const NETHER_PORTAL: &str = "NETHER_PORTAL";
    pub const END_PORTAL: &str = "END_PORTAL";
    pub const SPECTATE: &str = "SPECTATE";
    pub const END_GATEWAY: &str = "END_GATEWAY";
    pub const CONSUMABLE_EFFECT: &str = "CONSUMABLE_EFFECT";
    pub const DISMOUNT: &str = "DISMOUNT";
    pub const EXIT_BED: &str = "EXIT_BED";
    pub const UNKNOWN: &str = "UNKNOWN";
    pub const CHORUS_FRUIT: &str = "CHORUS_FRUIT";
}

/// `org.bukkit.event.raid.RaidStopEvent$Reason`.
pub mod raid_stop_event_reason {
    /// bukkit: org/bukkit/event/raid/RaidStopEvent$Reason.values
    pub fn values() -> &'static [&'static str] {
        &["PEACE", "TIMEOUT", "FINISHED", "UNSPAWNABLE", "NOT_IN_VILLAGE"]
    }
    /// bukkit: org/bukkit/event/raid/RaidStopEvent$Reason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PEACE: &str = "PEACE";
    pub const TIMEOUT: &str = "TIMEOUT";
    pub const FINISHED: &str = "FINISHED";
    pub const UNSPAWNABLE: &str = "UNSPAWNABLE";
    pub const NOT_IN_VILLAGE: &str = "NOT_IN_VILLAGE";
}

/// `org.bukkit.event.server.ServerLoadEvent$LoadType`.
pub mod server_load_event_load_type {
    /// bukkit: org/bukkit/event/server/ServerLoadEvent$LoadType.values
    pub fn values() -> &'static [&'static str] {
        &["STARTUP", "RELOAD"]
    }
    /// bukkit: org/bukkit/event/server/ServerLoadEvent$LoadType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const STARTUP: &str = "STARTUP";
    pub const RELOAD: &str = "RELOAD";
}

/// `org.bukkit.event.weather.LightningStrikeEvent$Cause`.
pub mod lightning_strike_event_cause {
    /// bukkit: org/bukkit/event/weather/LightningStrikeEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["COMMAND", "CUSTOM", "SPAWNER", "TRIDENT", "TRAP", "WEATHER", "ENCHANTMENT", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/weather/LightningStrikeEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMAND: &str = "COMMAND";
    pub const CUSTOM: &str = "CUSTOM";
    pub const SPAWNER: &str = "SPAWNER";
    pub const TRIDENT: &str = "TRIDENT";
    pub const TRAP: &str = "TRAP";
    pub const WEATHER: &str = "WEATHER";
    pub const ENCHANTMENT: &str = "ENCHANTMENT";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.weather.ThunderChangeEvent$Cause`.
pub mod thunder_change_event_cause {
    /// bukkit: org/bukkit/event/weather/ThunderChangeEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["COMMAND", "NATURAL", "SLEEP", "PLUGIN", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/weather/ThunderChangeEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMAND: &str = "COMMAND";
    pub const NATURAL: &str = "NATURAL";
    pub const SLEEP: &str = "SLEEP";
    pub const PLUGIN: &str = "PLUGIN";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.weather.WeatherChangeEvent$Cause`.
pub mod weather_change_event_cause {
    /// bukkit: org/bukkit/event/weather/WeatherChangeEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["COMMAND", "NATURAL", "SLEEP", "PLUGIN", "UNKNOWN"]
    }
    /// bukkit: org/bukkit/event/weather/WeatherChangeEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMAND: &str = "COMMAND";
    pub const NATURAL: &str = "NATURAL";
    pub const SLEEP: &str = "SLEEP";
    pub const PLUGIN: &str = "PLUGIN";
    pub const UNKNOWN: &str = "UNKNOWN";
}

/// `org.bukkit.event.world.AsyncStructureGenerateEvent$Cause`.
pub mod async_structure_generate_event_cause {
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent$Cause.values
    pub fn values() -> &'static [&'static str] {
        &["COMMAND", "WORLD_GENERATION", "CUSTOM"]
    }
    /// bukkit: org/bukkit/event/world/AsyncStructureGenerateEvent$Cause.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMAND: &str = "COMMAND";
    pub const WORLD_GENERATION: &str = "WORLD_GENERATION";
    pub const CUSTOM: &str = "CUSTOM";
}

/// `org.bukkit.event.world.ClockTimeSkipEvent$SkipReason`.
pub mod clock_time_skip_event_skip_reason {
    /// bukkit: org/bukkit/event/world/ClockTimeSkipEvent$SkipReason.values
    pub fn values() -> &'static [&'static str] {
        &["COMMAND", "CUSTOM", "NIGHT_SKIP"]
    }
    /// bukkit: org/bukkit/event/world/ClockTimeSkipEvent$SkipReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMAND: &str = "COMMAND";
    pub const CUSTOM: &str = "CUSTOM";
    pub const NIGHT_SKIP: &str = "NIGHT_SKIP";
}

/// `org.bukkit.event.world.PortalCreateEvent$CreateReason`.
pub mod portal_create_event_create_reason {
    /// bukkit: org/bukkit/event/world/PortalCreateEvent$CreateReason.values
    pub fn values() -> &'static [&'static str] {
        &["FIRE", "NETHER_PAIR", "END_PLATFORM"]
    }
    /// bukkit: org/bukkit/event/world/PortalCreateEvent$CreateReason.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FIRE: &str = "FIRE";
    pub const NETHER_PAIR: &str = "NETHER_PAIR";
    pub const END_PLATFORM: &str = "END_PLATFORM";
}

/// `org.bukkit.inventory.CreativeCategory`.
pub mod creative_category {
    /// bukkit: org/bukkit/inventory/CreativeCategory.values
    pub fn values() -> &'static [&'static str] {
        &["BUILDING_BLOCKS", "DECORATIONS", "REDSTONE", "TRANSPORTATION", "MISC", "FOOD", "TOOLS", "COMBAT", "BREWING"]
    }
    /// bukkit: org/bukkit/inventory/CreativeCategory.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BUILDING_BLOCKS: &str = "BUILDING_BLOCKS";
    pub const DECORATIONS: &str = "DECORATIONS";
    pub const REDSTONE: &str = "REDSTONE";
    pub const TRANSPORTATION: &str = "TRANSPORTATION";
    pub const MISC: &str = "MISC";
    pub const FOOD: &str = "FOOD";
    pub const TOOLS: &str = "TOOLS";
    pub const COMBAT: &str = "COMBAT";
    pub const BREWING: &str = "BREWING";
}

/// `org.bukkit.inventory.EquipmentSlot`.
pub mod equipment_slot {
    /// bukkit: org/bukkit/inventory/EquipmentSlot.values
    pub fn values() -> &'static [&'static str] {
        &["HAND", "OFF_HAND", "FEET", "LEGS", "CHEST", "HEAD", "BODY", "SADDLE"]
    }
    /// bukkit: org/bukkit/inventory/EquipmentSlot.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const HAND: &str = "HAND";
    pub const OFF_HAND: &str = "OFF_HAND";
    pub const FEET: &str = "FEET";
    pub const LEGS: &str = "LEGS";
    pub const CHEST: &str = "CHEST";
    pub const HEAD: &str = "HEAD";
    pub const BODY: &str = "BODY";
    pub const SADDLE: &str = "SADDLE";
}

/// `org.bukkit.inventory.InventoryView$Property`.
pub mod inventory_view_property {
    /// bukkit: org/bukkit/inventory/InventoryView$Property.values
    pub fn values() -> &'static [&'static str] {
        &["BREW_TIME", "FUEL_TIME", "BURN_TIME", "TICKS_FOR_CURRENT_FUEL", "COOK_TIME", "TICKS_FOR_CURRENT_SMELTING", "ENCHANT_BUTTON1", "ENCHANT_BUTTON2", "ENCHANT_BUTTON3", "ENCHANT_XP_SEED", "ENCHANT_ID1", "ENCHANT_ID2", "ENCHANT_ID3", "ENCHANT_LEVEL1", "ENCHANT_LEVEL2", "ENCHANT_LEVEL3", "LEVELS", "PRIMARY_EFFECT", "SECONDARY_EFFECT", "REPAIR_COST", "BOOK_PAGE"]
    }
    /// bukkit: org/bukkit/inventory/InventoryView$Property.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BREW_TIME: &str = "BREW_TIME";
    pub const FUEL_TIME: &str = "FUEL_TIME";
    pub const BURN_TIME: &str = "BURN_TIME";
    pub const TICKS_FOR_CURRENT_FUEL: &str = "TICKS_FOR_CURRENT_FUEL";
    pub const COOK_TIME: &str = "COOK_TIME";
    pub const TICKS_FOR_CURRENT_SMELTING: &str = "TICKS_FOR_CURRENT_SMELTING";
    pub const ENCHANT_BUTTON1: &str = "ENCHANT_BUTTON1";
    pub const ENCHANT_BUTTON2: &str = "ENCHANT_BUTTON2";
    pub const ENCHANT_BUTTON3: &str = "ENCHANT_BUTTON3";
    pub const ENCHANT_XP_SEED: &str = "ENCHANT_XP_SEED";
    pub const ENCHANT_ID1: &str = "ENCHANT_ID1";
    pub const ENCHANT_ID2: &str = "ENCHANT_ID2";
    pub const ENCHANT_ID3: &str = "ENCHANT_ID3";
    pub const ENCHANT_LEVEL1: &str = "ENCHANT_LEVEL1";
    pub const ENCHANT_LEVEL2: &str = "ENCHANT_LEVEL2";
    pub const ENCHANT_LEVEL3: &str = "ENCHANT_LEVEL3";
    pub const LEVELS: &str = "LEVELS";
    pub const PRIMARY_EFFECT: &str = "PRIMARY_EFFECT";
    pub const SECONDARY_EFFECT: &str = "SECONDARY_EFFECT";
    pub const REPAIR_COST: &str = "REPAIR_COST";
    pub const BOOK_PAGE: &str = "BOOK_PAGE";
}

/// `org.bukkit.inventory.ItemFlag`.
pub mod item_flag {
    /// bukkit: org/bukkit/inventory/ItemFlag.values
    pub fn values() -> &'static [&'static str] {
        &["HIDE_ENCHANTS", "HIDE_ATTRIBUTES", "HIDE_UNBREAKABLE", "HIDE_DESTROYS", "HIDE_PLACED_ON", "HIDE_ADDITIONAL_TOOLTIP", "HIDE_DYE", "HIDE_ARMOR_TRIM", "HIDE_STORED_ENCHANTS"]
    }
    /// bukkit: org/bukkit/inventory/ItemFlag.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const HIDE_ENCHANTS: &str = "HIDE_ENCHANTS";
    pub const HIDE_ATTRIBUTES: &str = "HIDE_ATTRIBUTES";
    pub const HIDE_UNBREAKABLE: &str = "HIDE_UNBREAKABLE";
    pub const HIDE_DESTROYS: &str = "HIDE_DESTROYS";
    pub const HIDE_PLACED_ON: &str = "HIDE_PLACED_ON";
    pub const HIDE_ADDITIONAL_TOOLTIP: &str = "HIDE_ADDITIONAL_TOOLTIP";
    pub const HIDE_DYE: &str = "HIDE_DYE";
    pub const HIDE_ARMOR_TRIM: &str = "HIDE_ARMOR_TRIM";
    pub const HIDE_STORED_ENCHANTS: &str = "HIDE_STORED_ENCHANTS";
}

/// `org.bukkit.inventory.ItemRarity`.
pub mod inventory_item_rarity {
    /// bukkit: org/bukkit/inventory/ItemRarity.values
    pub fn values() -> &'static [&'static str] {
        &["COMMON", "UNCOMMON", "RARE", "EPIC"]
    }
    /// bukkit: org/bukkit/inventory/ItemRarity.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const COMMON: &str = "COMMON";
    pub const UNCOMMON: &str = "UNCOMMON";
    pub const RARE: &str = "RARE";
    pub const EPIC: &str = "EPIC";
}

/// `org.bukkit.inventory.MainHand`.
pub mod main_hand {
    /// bukkit: org/bukkit/inventory/MainHand.values
    pub fn values() -> &'static [&'static str] {
        &["LEFT", "RIGHT"]
    }
    /// bukkit: org/bukkit/inventory/MainHand.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const LEFT: &str = "LEFT";
    pub const RIGHT: &str = "RIGHT";
}

/// `org.bukkit.inventory.meta.BookMeta$Generation`.
pub mod book_meta_generation {
    /// bukkit: org/bukkit/inventory/meta/BookMeta$Generation.values
    pub fn values() -> &'static [&'static str] {
        &["ORIGINAL", "COPY_OF_ORIGINAL", "COPY_OF_COPY", "TATTERED"]
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta$Generation.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ORIGINAL: &str = "ORIGINAL";
    pub const COPY_OF_ORIGINAL: &str = "COPY_OF_ORIGINAL";
    pub const COPY_OF_COPY: &str = "COPY_OF_COPY";
    pub const TATTERED: &str = "TATTERED";
}

/// `org.bukkit.inventory.recipe.CookingBookCategory`.
pub mod cooking_book_category {
    /// bukkit: org/bukkit/inventory/recipe/CookingBookCategory.values
    pub fn values() -> &'static [&'static str] {
        &["FOOD", "BLOCKS", "MISC"]
    }
    /// bukkit: org/bukkit/inventory/recipe/CookingBookCategory.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const FOOD: &str = "FOOD";
    pub const BLOCKS: &str = "BLOCKS";
    pub const MISC: &str = "MISC";
}

/// `org.bukkit.inventory.recipe.CraftingBookCategory`.
pub mod crafting_book_category {
    /// bukkit: org/bukkit/inventory/recipe/CraftingBookCategory.values
    pub fn values() -> &'static [&'static str] {
        &["BUILDING", "REDSTONE", "EQUIPMENT", "MISC"]
    }
    /// bukkit: org/bukkit/inventory/recipe/CraftingBookCategory.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BUILDING: &str = "BUILDING";
    pub const REDSTONE: &str = "REDSTONE";
    pub const EQUIPMENT: &str = "EQUIPMENT";
    pub const MISC: &str = "MISC";
}

/// `org.bukkit.loot.LootTables`.
pub mod loot_tables {
    /// bukkit: org/bukkit/loot/LootTables.values
    pub fn values() -> &'static [&'static str] {
        &["ABANDONED_MINESHAFT", "BURIED_TREASURE", "DESERT_PYRAMID", "END_CITY_TREASURE", "IGLOO_CHEST", "JUNGLE_TEMPLE", "JUNGLE_TEMPLE_DISPENSER", "NETHER_BRIDGE", "PILLAGER_OUTPOST", "BASTION_TREASURE", "BASTION_OTHER", "BASTION_BRIDGE", "BASTION_HOGLIN_STABLE", "ANCIENT_CITY", "ANCIENT_CITY_ICE_BOX", "RUINED_PORTAL", "TRIAL_CHAMBERS_REWARD", "TRIAL_CHAMBERS_REWARD_COMMON", "TRIAL_CHAMBERS_REWARD_RARE", "TRIAL_CHAMBERS_REWARD_UNIQUE", "TRIAL_CHAMBERS_REWARD_OMINOUS", "TRIAL_CHAMBERS_REWARD_OMINOUS_COMMON", "TRIAL_CHAMBERS_REWARD_OMINOUS_RARE", "TRIAL_CHAMBERS_REWARD_OMINOUS_UNIQUE", "TRIAL_CHAMBERS_SUPPLY", "TRIAL_CHAMBERS_CORRIDOR", "TRIAL_CHAMBERS_INTERSECTION", "TRIAL_CHAMBERS_INTERSECTION_BARREL", "TRIAL_CHAMBERS_ENTRANCE", "TRIAL_CHAMBERS_CORRIDOR_DISPENSER", "TRIAL_CHAMBERS_CHAMBER_DISPENSER", "TRIAL_CHAMBERS_WATER_DISPENSER", "TRIAL_CHAMBERS_CORRIDOR_POT", "EQUIPMENT_TRIAL_CHAMBER", "EQUIPMENT_TRIAL_CHAMBER_RANGED", "EQUIPMENT_TRIAL_CHAMBER_MELEE", "SHIPWRECK_MAP", "SHIPWRECK_SUPPLY", "SHIPWRECK_TREASURE", "SIMPLE_DUNGEON", "SPAWN_BONUS_CHEST", "STRONGHOLD_CORRIDOR", "STRONGHOLD_CROSSING", "STRONGHOLD_LIBRARY", "UNDERWATER_RUIN_BIG", "UNDERWATER_RUIN_SMALL", "VILLAGE_ARMORER", "VILLAGE_BUTCHER", "VILLAGE_CARTOGRAPHER", "VILLAGE_DESERT_HOUSE", "VILLAGE_FISHER", "VILLAGE_FLETCHER", "VILLAGE_MASON", "VILLAGE_PLAINS_HOUSE", "VILLAGE_SAVANNA_HOUSE", "VILLAGE_SHEPHERD", "VILLAGE_SNOWY_HOUSE", "VILLAGE_TAIGA_HOUSE", "VILLAGE_TANNERY", "VILLAGE_TEMPLE", "VILLAGE_TOOLSMITH", "VILLAGE_WEAPONSMITH", "WOODLAND_MANSION", "ABANDONED_CAMP_COMMON_CHEST", "ABANDONED_CAMP_SECRET_CHEST", "ABANDONED_CAMP_BARREL", "ARMOR_STAND", "AXOLOTL", "BAT", "BEE", "BLAZE", "CAT", "CAVE_SPIDER", "CHICKEN", "COD", "COW", "CREEPER", "DOLPHIN", "DONKEY", "DROWNED", "ELDER_GUARDIAN", "ENDER_DRAGON", "ENDERMAN", "ENDERMITE", "EVOKER", "FOX", "GHAST", "GIANT", "GLOW_SQUID", "GOAT", "GUARDIAN", "HOGLIN", "HORSE", "HUSK", "ILLUSIONER", "IRON_GOLEM", "LLAMA", "MAGMA_CUBE", "MOOSHROOM", "MULE", "OCELOT", "PANDA", "PARROT", "PHANTOM", "PIG", "PIGLIN", "PIGLIN_BRUTE", "PILLAGER", "PLAYER", "POLAR_BEAR", "PUFFERFISH", "RABBIT", "RAVAGER", "SALMON", "SHULKER", "SILVERFISH", "SKELETON", "SKELETON_HORSE", "SLIME", "SNOW_GOLEM", "SPIDER", "SQUID", "STRAY", "STRIDER", "TRADER_LLAMA", "TROPICAL_FISH", "TURTLE", "VEX", "VILLAGER", "VINDICATOR", "WANDERING_TRADER", "WITCH", "WITHER", "WITHER_SKELETON", "WOLF", "ZOGLIN", "ZOMBIE", "ZOMBIE_HORSE", "ZOMBIE_VILLAGER", "ZOMBIFIED_PIGLIN", "CHARGED_CREEPER", "CHARGED_CREEPER_PIGLIN", "CHARGED_CREEPER_CREEPER", "CHARGED_CREEPER_SKELETON", "CHARGED_CREEPER_WITHER_SKELETON", "CHARGED_CREEPER_ZOMBIE", "ARMORER_GIFT", "BUTCHER_GIFT", "CARTOGRAPHER_GIFT", "CAT_MORNING_GIFT", "CLERIC_GIFT", "FARMER_GIFT", "FISHERMAN_GIFT", "FISHING", "FISHING_FISH", "FISHING_JUNK", "FISHING_TREASURE", "FLETCHER_GIFT", "LEATHERWORKER_GIFT", "LIBRARIAN_GIFT", "MASON_GIFT", "SHEPHERD_GIFT", "TOOLSMITH_GIFT", "WEAPONSMITH_GIFT", "UNEMPLOYED_GIFT", "BABY_VILLAGER_GIFT", "SNIFFER_DIGGING", "TURTLE_GROW", "PANDA_SNEEZE", "CHICKEN_LAY", "ARMADILLO_SHED", "PIGLIN_BARTERING", "TRIAL_CHAMBER_KEY", "TRIAL_CHAMBER_CONSUMABLES", "OMINOUS_TRIAL_CHAMBER_KEY", "OMINOUS_TRIAL_CHAMBER_CONSUMABLES", "TRIAL_CHAMBER_ITEMS_TO_DROP_WHEN_OMINOUS", "SHEARING_BOGGED", "SHEARING_MOOSHROOM", "SHEARING_MOOSHROOM_RED", "SHEARING_MOOSHROOM_BROWN", "SHEARING_SNOW_GOLEM", "SHEARING_SHEEP", "SHEARING_SHEEP_BLACK", "SHEARING_SHEEP_BLUE", "SHEARING_SHEEP_BROWN", "SHEARING_SHEEP_CYAN", "SHEARING_SHEEP_GRAY", "SHEARING_SHEEP_GREEN", "SHEARING_SHEEP_LIGHT_BLUE", "SHEARING_SHEEP_LIGHT_GRAY", "SHEARING_SHEEP_LIME", "SHEARING_SHEEP_MAGENTA", "SHEARING_SHEEP_ORANGE", "SHEARING_SHEEP_PINK", "SHEARING_SHEEP_PURPLE", "SHEARING_SHEEP_RED", "SHEARING_SHEEP_WHITE", "SHEARING_SHEEP_YELLOW", "TILL_ROOTED_DIRT", "CARVE_PUMPKIN", "HARVEST_CAVE_VINE", "HARVEST_SWEET_BERRY_BUSH", "HARVEST_BEEHIVE", "BRUSH_ARMADILLO", "DESERT_WELL_ARCHAEOLOGY", "DESERT_PYRAMID_ARCHAEOLOGY", "TRAIL_RUINS_ARCHAEOLOGY_COMMON", "TRAIL_RUINS_ARCHAEOLOGY_RARE", "OCEAN_RUIN_WARM_ARCHAEOLOGY", "OCEAN_RUIN_COLD_ARCHAEOLOGY", "SHEEP", "SHEEP_BLACK", "SHEEP_BLUE", "SHEEP_BROWN", "SHEEP_CYAN", "SHEEP_GRAY", "SHEEP_GREEN", "SHEEP_LIGHT_BLUE", "SHEEP_LIGHT_GRAY", "SHEEP_LIME", "SHEEP_MAGENTA", "SHEEP_ORANGE", "SHEEP_PINK", "SHEEP_PURPLE", "SHEEP_RED", "SHEEP_WHITE", "SHEEP_YELLOW"]
    }
    /// bukkit: org/bukkit/loot/LootTables.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ABANDONED_MINESHAFT: &str = "ABANDONED_MINESHAFT";
    pub const BURIED_TREASURE: &str = "BURIED_TREASURE";
    pub const DESERT_PYRAMID: &str = "DESERT_PYRAMID";
    pub const END_CITY_TREASURE: &str = "END_CITY_TREASURE";
    pub const IGLOO_CHEST: &str = "IGLOO_CHEST";
    pub const JUNGLE_TEMPLE: &str = "JUNGLE_TEMPLE";
    pub const JUNGLE_TEMPLE_DISPENSER: &str = "JUNGLE_TEMPLE_DISPENSER";
    pub const NETHER_BRIDGE: &str = "NETHER_BRIDGE";
    pub const PILLAGER_OUTPOST: &str = "PILLAGER_OUTPOST";
    pub const BASTION_TREASURE: &str = "BASTION_TREASURE";
    pub const BASTION_OTHER: &str = "BASTION_OTHER";
    pub const BASTION_BRIDGE: &str = "BASTION_BRIDGE";
    pub const BASTION_HOGLIN_STABLE: &str = "BASTION_HOGLIN_STABLE";
    pub const ANCIENT_CITY: &str = "ANCIENT_CITY";
    pub const ANCIENT_CITY_ICE_BOX: &str = "ANCIENT_CITY_ICE_BOX";
    pub const RUINED_PORTAL: &str = "RUINED_PORTAL";
    pub const TRIAL_CHAMBERS_REWARD: &str = "TRIAL_CHAMBERS_REWARD";
    pub const TRIAL_CHAMBERS_REWARD_COMMON: &str = "TRIAL_CHAMBERS_REWARD_COMMON";
    pub const TRIAL_CHAMBERS_REWARD_RARE: &str = "TRIAL_CHAMBERS_REWARD_RARE";
    pub const TRIAL_CHAMBERS_REWARD_UNIQUE: &str = "TRIAL_CHAMBERS_REWARD_UNIQUE";
    pub const TRIAL_CHAMBERS_REWARD_OMINOUS: &str = "TRIAL_CHAMBERS_REWARD_OMINOUS";
    pub const TRIAL_CHAMBERS_REWARD_OMINOUS_COMMON: &str = "TRIAL_CHAMBERS_REWARD_OMINOUS_COMMON";
    pub const TRIAL_CHAMBERS_REWARD_OMINOUS_RARE: &str = "TRIAL_CHAMBERS_REWARD_OMINOUS_RARE";
    pub const TRIAL_CHAMBERS_REWARD_OMINOUS_UNIQUE: &str = "TRIAL_CHAMBERS_REWARD_OMINOUS_UNIQUE";
    pub const TRIAL_CHAMBERS_SUPPLY: &str = "TRIAL_CHAMBERS_SUPPLY";
    pub const TRIAL_CHAMBERS_CORRIDOR: &str = "TRIAL_CHAMBERS_CORRIDOR";
    pub const TRIAL_CHAMBERS_INTERSECTION: &str = "TRIAL_CHAMBERS_INTERSECTION";
    pub const TRIAL_CHAMBERS_INTERSECTION_BARREL: &str = "TRIAL_CHAMBERS_INTERSECTION_BARREL";
    pub const TRIAL_CHAMBERS_ENTRANCE: &str = "TRIAL_CHAMBERS_ENTRANCE";
    pub const TRIAL_CHAMBERS_CORRIDOR_DISPENSER: &str = "TRIAL_CHAMBERS_CORRIDOR_DISPENSER";
    pub const TRIAL_CHAMBERS_CHAMBER_DISPENSER: &str = "TRIAL_CHAMBERS_CHAMBER_DISPENSER";
    pub const TRIAL_CHAMBERS_WATER_DISPENSER: &str = "TRIAL_CHAMBERS_WATER_DISPENSER";
    pub const TRIAL_CHAMBERS_CORRIDOR_POT: &str = "TRIAL_CHAMBERS_CORRIDOR_POT";
    pub const EQUIPMENT_TRIAL_CHAMBER: &str = "EQUIPMENT_TRIAL_CHAMBER";
    pub const EQUIPMENT_TRIAL_CHAMBER_RANGED: &str = "EQUIPMENT_TRIAL_CHAMBER_RANGED";
    pub const EQUIPMENT_TRIAL_CHAMBER_MELEE: &str = "EQUIPMENT_TRIAL_CHAMBER_MELEE";
    pub const SHIPWRECK_MAP: &str = "SHIPWRECK_MAP";
    pub const SHIPWRECK_SUPPLY: &str = "SHIPWRECK_SUPPLY";
    pub const SHIPWRECK_TREASURE: &str = "SHIPWRECK_TREASURE";
    pub const SIMPLE_DUNGEON: &str = "SIMPLE_DUNGEON";
    pub const SPAWN_BONUS_CHEST: &str = "SPAWN_BONUS_CHEST";
    pub const STRONGHOLD_CORRIDOR: &str = "STRONGHOLD_CORRIDOR";
    pub const STRONGHOLD_CROSSING: &str = "STRONGHOLD_CROSSING";
    pub const STRONGHOLD_LIBRARY: &str = "STRONGHOLD_LIBRARY";
    pub const UNDERWATER_RUIN_BIG: &str = "UNDERWATER_RUIN_BIG";
    pub const UNDERWATER_RUIN_SMALL: &str = "UNDERWATER_RUIN_SMALL";
    pub const VILLAGE_ARMORER: &str = "VILLAGE_ARMORER";
    pub const VILLAGE_BUTCHER: &str = "VILLAGE_BUTCHER";
    pub const VILLAGE_CARTOGRAPHER: &str = "VILLAGE_CARTOGRAPHER";
    pub const VILLAGE_DESERT_HOUSE: &str = "VILLAGE_DESERT_HOUSE";
    pub const VILLAGE_FISHER: &str = "VILLAGE_FISHER";
    pub const VILLAGE_FLETCHER: &str = "VILLAGE_FLETCHER";
    pub const VILLAGE_MASON: &str = "VILLAGE_MASON";
    pub const VILLAGE_PLAINS_HOUSE: &str = "VILLAGE_PLAINS_HOUSE";
    pub const VILLAGE_SAVANNA_HOUSE: &str = "VILLAGE_SAVANNA_HOUSE";
    pub const VILLAGE_SHEPHERD: &str = "VILLAGE_SHEPHERD";
    pub const VILLAGE_SNOWY_HOUSE: &str = "VILLAGE_SNOWY_HOUSE";
    pub const VILLAGE_TAIGA_HOUSE: &str = "VILLAGE_TAIGA_HOUSE";
    pub const VILLAGE_TANNERY: &str = "VILLAGE_TANNERY";
    pub const VILLAGE_TEMPLE: &str = "VILLAGE_TEMPLE";
    pub const VILLAGE_TOOLSMITH: &str = "VILLAGE_TOOLSMITH";
    pub const VILLAGE_WEAPONSMITH: &str = "VILLAGE_WEAPONSMITH";
    pub const WOODLAND_MANSION: &str = "WOODLAND_MANSION";
    pub const ABANDONED_CAMP_COMMON_CHEST: &str = "ABANDONED_CAMP_COMMON_CHEST";
    pub const ABANDONED_CAMP_SECRET_CHEST: &str = "ABANDONED_CAMP_SECRET_CHEST";
    pub const ABANDONED_CAMP_BARREL: &str = "ABANDONED_CAMP_BARREL";
    pub const ARMOR_STAND: &str = "ARMOR_STAND";
    pub const AXOLOTL: &str = "AXOLOTL";
    pub const BAT: &str = "BAT";
    pub const BEE: &str = "BEE";
    pub const BLAZE: &str = "BLAZE";
    pub const CAT: &str = "CAT";
    pub const CAVE_SPIDER: &str = "CAVE_SPIDER";
    pub const CHICKEN: &str = "CHICKEN";
    pub const COD: &str = "COD";
    pub const COW: &str = "COW";
    pub const CREEPER: &str = "CREEPER";
    pub const DOLPHIN: &str = "DOLPHIN";
    pub const DONKEY: &str = "DONKEY";
    pub const DROWNED: &str = "DROWNED";
    pub const ELDER_GUARDIAN: &str = "ELDER_GUARDIAN";
    pub const ENDER_DRAGON: &str = "ENDER_DRAGON";
    pub const ENDERMAN: &str = "ENDERMAN";
    pub const ENDERMITE: &str = "ENDERMITE";
    pub const EVOKER: &str = "EVOKER";
    pub const FOX: &str = "FOX";
    pub const GHAST: &str = "GHAST";
    pub const GIANT: &str = "GIANT";
    pub const GLOW_SQUID: &str = "GLOW_SQUID";
    pub const GOAT: &str = "GOAT";
    pub const GUARDIAN: &str = "GUARDIAN";
    pub const HOGLIN: &str = "HOGLIN";
    pub const HORSE: &str = "HORSE";
    pub const HUSK: &str = "HUSK";
    pub const ILLUSIONER: &str = "ILLUSIONER";
    pub const IRON_GOLEM: &str = "IRON_GOLEM";
    pub const LLAMA: &str = "LLAMA";
    pub const MAGMA_CUBE: &str = "MAGMA_CUBE";
    pub const MOOSHROOM: &str = "MOOSHROOM";
    pub const MULE: &str = "MULE";
    pub const OCELOT: &str = "OCELOT";
    pub const PANDA: &str = "PANDA";
    pub const PARROT: &str = "PARROT";
    pub const PHANTOM: &str = "PHANTOM";
    pub const PIG: &str = "PIG";
    pub const PIGLIN: &str = "PIGLIN";
    pub const PIGLIN_BRUTE: &str = "PIGLIN_BRUTE";
    pub const PILLAGER: &str = "PILLAGER";
    pub const PLAYER: &str = "PLAYER";
    pub const POLAR_BEAR: &str = "POLAR_BEAR";
    pub const PUFFERFISH: &str = "PUFFERFISH";
    pub const RABBIT: &str = "RABBIT";
    pub const RAVAGER: &str = "RAVAGER";
    pub const SALMON: &str = "SALMON";
    pub const SHULKER: &str = "SHULKER";
    pub const SILVERFISH: &str = "SILVERFISH";
    pub const SKELETON: &str = "SKELETON";
    pub const SKELETON_HORSE: &str = "SKELETON_HORSE";
    pub const SLIME: &str = "SLIME";
    pub const SNOW_GOLEM: &str = "SNOW_GOLEM";
    pub const SPIDER: &str = "SPIDER";
    pub const SQUID: &str = "SQUID";
    pub const STRAY: &str = "STRAY";
    pub const STRIDER: &str = "STRIDER";
    pub const TRADER_LLAMA: &str = "TRADER_LLAMA";
    pub const TROPICAL_FISH: &str = "TROPICAL_FISH";
    pub const TURTLE: &str = "TURTLE";
    pub const VEX: &str = "VEX";
    pub const VILLAGER: &str = "VILLAGER";
    pub const VINDICATOR: &str = "VINDICATOR";
    pub const WANDERING_TRADER: &str = "WANDERING_TRADER";
    pub const WITCH: &str = "WITCH";
    pub const WITHER: &str = "WITHER";
    pub const WITHER_SKELETON: &str = "WITHER_SKELETON";
    pub const WOLF: &str = "WOLF";
    pub const ZOGLIN: &str = "ZOGLIN";
    pub const ZOMBIE: &str = "ZOMBIE";
    pub const ZOMBIE_HORSE: &str = "ZOMBIE_HORSE";
    pub const ZOMBIE_VILLAGER: &str = "ZOMBIE_VILLAGER";
    pub const ZOMBIFIED_PIGLIN: &str = "ZOMBIFIED_PIGLIN";
    pub const CHARGED_CREEPER: &str = "CHARGED_CREEPER";
    pub const CHARGED_CREEPER_PIGLIN: &str = "CHARGED_CREEPER_PIGLIN";
    pub const CHARGED_CREEPER_CREEPER: &str = "CHARGED_CREEPER_CREEPER";
    pub const CHARGED_CREEPER_SKELETON: &str = "CHARGED_CREEPER_SKELETON";
    pub const CHARGED_CREEPER_WITHER_SKELETON: &str = "CHARGED_CREEPER_WITHER_SKELETON";
    pub const CHARGED_CREEPER_ZOMBIE: &str = "CHARGED_CREEPER_ZOMBIE";
    pub const ARMORER_GIFT: &str = "ARMORER_GIFT";
    pub const BUTCHER_GIFT: &str = "BUTCHER_GIFT";
    pub const CARTOGRAPHER_GIFT: &str = "CARTOGRAPHER_GIFT";
    pub const CAT_MORNING_GIFT: &str = "CAT_MORNING_GIFT";
    pub const CLERIC_GIFT: &str = "CLERIC_GIFT";
    pub const FARMER_GIFT: &str = "FARMER_GIFT";
    pub const FISHERMAN_GIFT: &str = "FISHERMAN_GIFT";
    pub const FISHING: &str = "FISHING";
    pub const FISHING_FISH: &str = "FISHING_FISH";
    pub const FISHING_JUNK: &str = "FISHING_JUNK";
    pub const FISHING_TREASURE: &str = "FISHING_TREASURE";
    pub const FLETCHER_GIFT: &str = "FLETCHER_GIFT";
    pub const LEATHERWORKER_GIFT: &str = "LEATHERWORKER_GIFT";
    pub const LIBRARIAN_GIFT: &str = "LIBRARIAN_GIFT";
    pub const MASON_GIFT: &str = "MASON_GIFT";
    pub const SHEPHERD_GIFT: &str = "SHEPHERD_GIFT";
    pub const TOOLSMITH_GIFT: &str = "TOOLSMITH_GIFT";
    pub const WEAPONSMITH_GIFT: &str = "WEAPONSMITH_GIFT";
    pub const UNEMPLOYED_GIFT: &str = "UNEMPLOYED_GIFT";
    pub const BABY_VILLAGER_GIFT: &str = "BABY_VILLAGER_GIFT";
    pub const SNIFFER_DIGGING: &str = "SNIFFER_DIGGING";
    pub const TURTLE_GROW: &str = "TURTLE_GROW";
    pub const PANDA_SNEEZE: &str = "PANDA_SNEEZE";
    pub const CHICKEN_LAY: &str = "CHICKEN_LAY";
    pub const ARMADILLO_SHED: &str = "ARMADILLO_SHED";
    pub const PIGLIN_BARTERING: &str = "PIGLIN_BARTERING";
    pub const TRIAL_CHAMBER_KEY: &str = "TRIAL_CHAMBER_KEY";
    pub const TRIAL_CHAMBER_CONSUMABLES: &str = "TRIAL_CHAMBER_CONSUMABLES";
    pub const OMINOUS_TRIAL_CHAMBER_KEY: &str = "OMINOUS_TRIAL_CHAMBER_KEY";
    pub const OMINOUS_TRIAL_CHAMBER_CONSUMABLES: &str = "OMINOUS_TRIAL_CHAMBER_CONSUMABLES";
    pub const TRIAL_CHAMBER_ITEMS_TO_DROP_WHEN_OMINOUS: &str = "TRIAL_CHAMBER_ITEMS_TO_DROP_WHEN_OMINOUS";
    pub const SHEARING_BOGGED: &str = "SHEARING_BOGGED";
    pub const SHEARING_MOOSHROOM: &str = "SHEARING_MOOSHROOM";
    pub const SHEARING_MOOSHROOM_RED: &str = "SHEARING_MOOSHROOM_RED";
    pub const SHEARING_MOOSHROOM_BROWN: &str = "SHEARING_MOOSHROOM_BROWN";
    pub const SHEARING_SNOW_GOLEM: &str = "SHEARING_SNOW_GOLEM";
    pub const SHEARING_SHEEP: &str = "SHEARING_SHEEP";
    pub const SHEARING_SHEEP_BLACK: &str = "SHEARING_SHEEP_BLACK";
    pub const SHEARING_SHEEP_BLUE: &str = "SHEARING_SHEEP_BLUE";
    pub const SHEARING_SHEEP_BROWN: &str = "SHEARING_SHEEP_BROWN";
    pub const SHEARING_SHEEP_CYAN: &str = "SHEARING_SHEEP_CYAN";
    pub const SHEARING_SHEEP_GRAY: &str = "SHEARING_SHEEP_GRAY";
    pub const SHEARING_SHEEP_GREEN: &str = "SHEARING_SHEEP_GREEN";
    pub const SHEARING_SHEEP_LIGHT_BLUE: &str = "SHEARING_SHEEP_LIGHT_BLUE";
    pub const SHEARING_SHEEP_LIGHT_GRAY: &str = "SHEARING_SHEEP_LIGHT_GRAY";
    pub const SHEARING_SHEEP_LIME: &str = "SHEARING_SHEEP_LIME";
    pub const SHEARING_SHEEP_MAGENTA: &str = "SHEARING_SHEEP_MAGENTA";
    pub const SHEARING_SHEEP_ORANGE: &str = "SHEARING_SHEEP_ORANGE";
    pub const SHEARING_SHEEP_PINK: &str = "SHEARING_SHEEP_PINK";
    pub const SHEARING_SHEEP_PURPLE: &str = "SHEARING_SHEEP_PURPLE";
    pub const SHEARING_SHEEP_RED: &str = "SHEARING_SHEEP_RED";
    pub const SHEARING_SHEEP_WHITE: &str = "SHEARING_SHEEP_WHITE";
    pub const SHEARING_SHEEP_YELLOW: &str = "SHEARING_SHEEP_YELLOW";
    pub const TILL_ROOTED_DIRT: &str = "TILL_ROOTED_DIRT";
    pub const CARVE_PUMPKIN: &str = "CARVE_PUMPKIN";
    pub const HARVEST_CAVE_VINE: &str = "HARVEST_CAVE_VINE";
    pub const HARVEST_SWEET_BERRY_BUSH: &str = "HARVEST_SWEET_BERRY_BUSH";
    pub const HARVEST_BEEHIVE: &str = "HARVEST_BEEHIVE";
    pub const BRUSH_ARMADILLO: &str = "BRUSH_ARMADILLO";
    pub const DESERT_WELL_ARCHAEOLOGY: &str = "DESERT_WELL_ARCHAEOLOGY";
    pub const DESERT_PYRAMID_ARCHAEOLOGY: &str = "DESERT_PYRAMID_ARCHAEOLOGY";
    pub const TRAIL_RUINS_ARCHAEOLOGY_COMMON: &str = "TRAIL_RUINS_ARCHAEOLOGY_COMMON";
    pub const TRAIL_RUINS_ARCHAEOLOGY_RARE: &str = "TRAIL_RUINS_ARCHAEOLOGY_RARE";
    pub const OCEAN_RUIN_WARM_ARCHAEOLOGY: &str = "OCEAN_RUIN_WARM_ARCHAEOLOGY";
    pub const OCEAN_RUIN_COLD_ARCHAEOLOGY: &str = "OCEAN_RUIN_COLD_ARCHAEOLOGY";
    pub const SHEEP: &str = "SHEEP";
    pub const SHEEP_BLACK: &str = "SHEEP_BLACK";
    pub const SHEEP_BLUE: &str = "SHEEP_BLUE";
    pub const SHEEP_BROWN: &str = "SHEEP_BROWN";
    pub const SHEEP_CYAN: &str = "SHEEP_CYAN";
    pub const SHEEP_GRAY: &str = "SHEEP_GRAY";
    pub const SHEEP_GREEN: &str = "SHEEP_GREEN";
    pub const SHEEP_LIGHT_BLUE: &str = "SHEEP_LIGHT_BLUE";
    pub const SHEEP_LIGHT_GRAY: &str = "SHEEP_LIGHT_GRAY";
    pub const SHEEP_LIME: &str = "SHEEP_LIME";
    pub const SHEEP_MAGENTA: &str = "SHEEP_MAGENTA";
    pub const SHEEP_ORANGE: &str = "SHEEP_ORANGE";
    pub const SHEEP_PINK: &str = "SHEEP_PINK";
    pub const SHEEP_PURPLE: &str = "SHEEP_PURPLE";
    pub const SHEEP_RED: &str = "SHEEP_RED";
    pub const SHEEP_WHITE: &str = "SHEEP_WHITE";
    pub const SHEEP_YELLOW: &str = "SHEEP_YELLOW";
}

/// `org.bukkit.map.MapCursor$Type`.
pub mod map_cursor_type {
    /// bukkit: org/bukkit/map/MapCursor$Type.values
    pub fn values() -> &'static [&'static str] {
        &["ABANDONED_CAMP", "ANCIENT_CITY", "BANNER_BLACK", "BANNER_BLUE", "BANNER_BROWN", "BANNER_CYAN", "BANNER_GRAY", "BANNER_GREEN", "BANNER_LIGHT_BLUE", "BANNER_LIGHT_GRAY", "BANNER_LIME", "BANNER_MAGENTA", "BANNER_ORANGE", "BANNER_PINK", "BANNER_PURPLE", "BANNER_RED", "BANNER_WHITE", "BANNER_YELLOW", "BLUE_MARKER", "DESERT_PYRAMID", "FRAME", "JUNGLE_TEMPLE", "MANSION", "MINESHAFT", "MONUMENT", "OCEAN_RUIN_WARM", "PLAYER", "PLAYER_OFF_LIMITS", "PLAYER_OFF_MAP", "RED_MARKER", "RED_X", "SWAMP_HUT", "TARGET_POINT", "TARGET_X", "TRIAL_CHAMBERS", "VILLAGE_DESERT", "VILLAGE_PLAINS", "VILLAGE_SAVANNA", "VILLAGE_SNOWY", "VILLAGE_TAIGA"]
    }
    /// bukkit: org/bukkit/map/MapCursor$Type.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ABANDONED_CAMP: &str = "minecraft:abandoned_camp";
    pub const ANCIENT_CITY: &str = "minecraft:ancient_city";
    pub const BANNER_BLACK: &str = "minecraft:banner_black";
    pub const BANNER_BLUE: &str = "minecraft:banner_blue";
    pub const BANNER_BROWN: &str = "minecraft:banner_brown";
    pub const BANNER_CYAN: &str = "minecraft:banner_cyan";
    pub const BANNER_GRAY: &str = "minecraft:banner_gray";
    pub const BANNER_GREEN: &str = "minecraft:banner_green";
    pub const BANNER_LIGHT_BLUE: &str = "minecraft:banner_light_blue";
    pub const BANNER_LIGHT_GRAY: &str = "minecraft:banner_light_gray";
    pub const BANNER_LIME: &str = "minecraft:banner_lime";
    pub const BANNER_MAGENTA: &str = "minecraft:banner_magenta";
    pub const BANNER_ORANGE: &str = "minecraft:banner_orange";
    pub const BANNER_PINK: &str = "minecraft:banner_pink";
    pub const BANNER_PURPLE: &str = "minecraft:banner_purple";
    pub const BANNER_RED: &str = "minecraft:banner_red";
    pub const BANNER_WHITE: &str = "minecraft:banner_white";
    pub const BANNER_YELLOW: &str = "minecraft:banner_yellow";
    pub const BLUE_MARKER: &str = "minecraft:blue_marker";
    pub const DESERT_PYRAMID: &str = "minecraft:desert_pyramid";
    pub const FRAME: &str = "minecraft:frame";
    pub const JUNGLE_TEMPLE: &str = "minecraft:jungle_temple";
    pub const MANSION: &str = "minecraft:mansion";
    pub const MINESHAFT: &str = "minecraft:mineshaft";
    pub const MONUMENT: &str = "minecraft:monument";
    pub const OCEAN_RUIN_WARM: &str = "minecraft:ocean_ruin_warm";
    pub const PLAYER: &str = "minecraft:player";
    pub const PLAYER_OFF_LIMITS: &str = "minecraft:player_off_limits";
    pub const PLAYER_OFF_MAP: &str = "minecraft:player_off_map";
    pub const RED_MARKER: &str = "minecraft:red_marker";
    pub const RED_X: &str = "minecraft:red_x";
    pub const SWAMP_HUT: &str = "minecraft:swamp_hut";
    pub const TARGET_POINT: &str = "minecraft:target_point";
    pub const TARGET_X: &str = "minecraft:target_x";
    pub const TRIAL_CHAMBERS: &str = "minecraft:trial_chambers";
    pub const VILLAGE_DESERT: &str = "minecraft:village_desert";
    pub const VILLAGE_PLAINS: &str = "minecraft:village_plains";
    pub const VILLAGE_SAVANNA: &str = "minecraft:village_savanna";
    pub const VILLAGE_SNOWY: &str = "minecraft:village_snowy";
    pub const VILLAGE_TAIGA: &str = "minecraft:village_taiga";
}

/// `org.bukkit.map.MapView$Scale`.
pub mod map_view_scale {
    /// bukkit: org/bukkit/map/MapView$Scale.values
    pub fn values() -> &'static [&'static str] {
        &["CLOSEST", "CLOSE", "NORMAL", "FAR", "FARTHEST"]
    }
    /// bukkit: org/bukkit/map/MapView$Scale.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CLOSEST: &str = "CLOSEST";
    pub const CLOSE: &str = "CLOSE";
    pub const NORMAL: &str = "NORMAL";
    pub const FAR: &str = "FAR";
    pub const FARTHEST: &str = "FARTHEST";
}

/// `org.bukkit.material.CocoaPlant$CocoaPlantSize`.
pub mod cocoa_plant_cocoa_plant_size {
    /// bukkit: org/bukkit/material/CocoaPlant$CocoaPlantSize.values
    pub fn values() -> &'static [&'static str] {
        &["SMALL", "MEDIUM", "LARGE"]
    }
    /// bukkit: org/bukkit/material/CocoaPlant$CocoaPlantSize.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SMALL: &str = "SMALL";
    pub const MEDIUM: &str = "MEDIUM";
    pub const LARGE: &str = "LARGE";
}

/// `org.bukkit.material.types.MushroomBlockTexture`.
pub mod mushroom_block_texture {
    /// bukkit: org/bukkit/material/types/MushroomBlockTexture.values
    pub fn values() -> &'static [&'static str] {
        &["ALL_PORES", "CAP_NORTH_WEST", "CAP_NORTH", "CAP_NORTH_EAST", "CAP_WEST", "CAP_TOP", "CAP_EAST", "CAP_SOUTH_WEST", "CAP_SOUTH", "CAP_SOUTH_EAST", "STEM_SIDES", "ALL_CAP", "ALL_STEM"]
    }
    /// bukkit: org/bukkit/material/types/MushroomBlockTexture.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALL_PORES: &str = "ALL_PORES";
    pub const CAP_NORTH_WEST: &str = "CAP_NORTH_WEST";
    pub const CAP_NORTH: &str = "CAP_NORTH";
    pub const CAP_NORTH_EAST: &str = "CAP_NORTH_EAST";
    pub const CAP_WEST: &str = "CAP_WEST";
    pub const CAP_TOP: &str = "CAP_TOP";
    pub const CAP_EAST: &str = "CAP_EAST";
    pub const CAP_SOUTH_WEST: &str = "CAP_SOUTH_WEST";
    pub const CAP_SOUTH: &str = "CAP_SOUTH";
    pub const CAP_SOUTH_EAST: &str = "CAP_SOUTH_EAST";
    pub const STEM_SIDES: &str = "STEM_SIDES";
    pub const ALL_CAP: &str = "ALL_CAP";
    pub const ALL_STEM: &str = "ALL_STEM";
}

/// `org.bukkit.metadata.LazyMetadataValue$CacheStrategy`.
pub mod lazy_metadata_value_cache_strategy {
    /// bukkit: org/bukkit/metadata/LazyMetadataValue$CacheStrategy.values
    pub fn values() -> &'static [&'static str] {
        &["CACHE_AFTER_FIRST_EVAL", "NEVER_CACHE", "CACHE_ETERNALLY"]
    }
    /// bukkit: org/bukkit/metadata/LazyMetadataValue$CacheStrategy.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CACHE_AFTER_FIRST_EVAL: &str = "CACHE_AFTER_FIRST_EVAL";
    pub const NEVER_CACHE: &str = "NEVER_CACHE";
    pub const CACHE_ETERNALLY: &str = "CACHE_ETERNALLY";
}

/// `org.bukkit.permissions.PermissionDefault`.
pub mod permission_default {
    /// bukkit: org/bukkit/permissions/PermissionDefault.values
    pub fn values() -> &'static [&'static str] {
        &["TRUE", "FALSE", "OP", "NOT_OP"]
    }
    /// bukkit: org/bukkit/permissions/PermissionDefault.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const OP: &str = "OP";
    pub const NOT_OP: &str = "NOT_OP";
}

/// `org.bukkit.plugin.PluginAwareness$Flags`.
pub mod plugin_awareness_flags {
    /// bukkit: org/bukkit/plugin/PluginAwareness$Flags.values
    pub fn values() -> &'static [&'static str] {
        &["UTF8"]
    }
    /// bukkit: org/bukkit/plugin/PluginAwareness$Flags.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const UTF8: &str = "UTF8";
}

/// `org.bukkit.plugin.PluginLoadOrder`.
pub mod plugin_load_order {
    /// bukkit: org/bukkit/plugin/PluginLoadOrder.values
    pub fn values() -> &'static [&'static str] {
        &["STARTUP", "POSTWORLD"]
    }
    /// bukkit: org/bukkit/plugin/PluginLoadOrder.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const STARTUP: &str = "STARTUP";
    pub const POSTWORLD: &str = "POSTWORLD";
}

/// `org.bukkit.plugin.ServicePriority`.
pub mod service_priority {
    /// bukkit: org/bukkit/plugin/ServicePriority.values
    pub fn values() -> &'static [&'static str] {
        &["Lowest", "Low", "Normal", "High", "Highest"]
    }
    /// bukkit: org/bukkit/plugin/ServicePriority.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const Lowest: &str = "Lowest";
    pub const Low: &str = "Low";
    pub const Normal: &str = "Normal";
    pub const High: &str = "High";
    pub const Highest: &str = "Highest";
}

/// `org.bukkit.plugin.messaging.PluginChannelDirection`.
pub mod plugin_channel_direction {
    /// bukkit: org/bukkit/plugin/messaging/PluginChannelDirection.values
    pub fn values() -> &'static [&'static str] {
        &["INCOMING", "OUTGOING"]
    }
    /// bukkit: org/bukkit/plugin/messaging/PluginChannelDirection.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const INCOMING: &str = "INCOMING";
    pub const OUTGOING: &str = "OUTGOING";
}

/// `org.bukkit.potion.PotionEffectType$Category`.
pub mod potion_effect_type_category {
    /// bukkit: org/bukkit/potion/PotionEffectType$Category.values
    pub fn values() -> &'static [&'static str] {
        &["BENEFICIAL", "HARMFUL", "NEUTRAL"]
    }
    /// bukkit: org/bukkit/potion/PotionEffectType$Category.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BENEFICIAL: &str = "BENEFICIAL";
    pub const HARMFUL: &str = "HARMFUL";
    pub const NEUTRAL: &str = "NEUTRAL";
}

/// `org.bukkit.potion.PotionEffectType`.
pub mod potion_effect_type {
    /// bukkit: org/bukkit/potion/PotionEffectType.values
    pub fn values() -> &'static [&'static str] {
        &["SPEED", "SLOWNESS", "HASTE", "MINING_FATIGUE", "STRENGTH", "INSTANT_HEALTH", "INSTANT_DAMAGE", "JUMP_BOOST", "NAUSEA", "REGENERATION", "RESISTANCE", "FIRE_RESISTANCE", "WATER_BREATHING", "INVISIBILITY", "BLINDNESS", "NIGHT_VISION", "HUNGER", "WEAKNESS", "POISON", "WITHER", "HEALTH_BOOST", "ABSORPTION", "SATURATION", "GLOWING", "LEVITATION", "LUCK", "UNLUCK", "SLOW_FALLING", "CONDUIT_POWER", "DOLPHINS_GRACE", "BAD_OMEN", "HERO_OF_THE_VILLAGE", "DARKNESS", "TRIAL_OMEN", "RAID_OMEN", "WIND_CHARGED", "WEAVING", "OOZING", "INFESTED", "BREATH_OF_THE_NAUTILUS"]
    }
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const SPEED: &str = "minecraft:speed";
    pub const SLOWNESS: &str = "minecraft:slowness";
    pub const HASTE: &str = "minecraft:haste";
    pub const MINING_FATIGUE: &str = "minecraft:mining_fatigue";
    pub const STRENGTH: &str = "minecraft:strength";
    pub const INSTANT_HEALTH: &str = "minecraft:instant_health";
    pub const INSTANT_DAMAGE: &str = "minecraft:instant_damage";
    pub const JUMP_BOOST: &str = "minecraft:jump_boost";
    pub const NAUSEA: &str = "minecraft:nausea";
    pub const REGENERATION: &str = "minecraft:regeneration";
    pub const RESISTANCE: &str = "minecraft:resistance";
    pub const FIRE_RESISTANCE: &str = "minecraft:fire_resistance";
    pub const WATER_BREATHING: &str = "minecraft:water_breathing";
    pub const INVISIBILITY: &str = "minecraft:invisibility";
    pub const BLINDNESS: &str = "minecraft:blindness";
    pub const NIGHT_VISION: &str = "minecraft:night_vision";
    pub const HUNGER: &str = "minecraft:hunger";
    pub const WEAKNESS: &str = "minecraft:weakness";
    pub const POISON: &str = "minecraft:poison";
    pub const WITHER: &str = "minecraft:wither";
    pub const HEALTH_BOOST: &str = "minecraft:health_boost";
    pub const ABSORPTION: &str = "minecraft:absorption";
    pub const SATURATION: &str = "minecraft:saturation";
    pub const GLOWING: &str = "minecraft:glowing";
    pub const LEVITATION: &str = "minecraft:levitation";
    pub const LUCK: &str = "minecraft:luck";
    pub const UNLUCK: &str = "minecraft:unluck";
    pub const SLOW_FALLING: &str = "minecraft:slow_falling";
    pub const CONDUIT_POWER: &str = "minecraft:conduit_power";
    pub const DOLPHINS_GRACE: &str = "minecraft:dolphins_grace";
    pub const BAD_OMEN: &str = "minecraft:bad_omen";
    pub const HERO_OF_THE_VILLAGE: &str = "minecraft:hero_of_the_village";
    pub const DARKNESS: &str = "minecraft:darkness";
    pub const TRIAL_OMEN: &str = "minecraft:trial_omen";
    pub const RAID_OMEN: &str = "minecraft:raid_omen";
    pub const WIND_CHARGED: &str = "minecraft:wind_charged";
    pub const WEAVING: &str = "minecraft:weaving";
    pub const OOZING: &str = "minecraft:oozing";
    pub const INFESTED: &str = "minecraft:infested";
    pub const BREATH_OF_THE_NAUTILUS: &str = "minecraft:breath_of_the_nautilus";
}

/// `org.bukkit.potion.PotionEffectTypeCategory`.
pub mod potion_potion_effect_type_category {
    /// bukkit: org/bukkit/potion/PotionEffectTypeCategory.values
    pub fn values() -> &'static [&'static str] {
        &["BENEFICIAL", "HARMFUL", "NEUTRAL"]
    }
    /// bukkit: org/bukkit/potion/PotionEffectTypeCategory.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const BENEFICIAL: &str = "BENEFICIAL";
    pub const HARMFUL: &str = "HARMFUL";
    pub const NEUTRAL: &str = "NEUTRAL";
}

/// `org.bukkit.potion.PotionType`.
pub mod potion_type {
    /// bukkit: org/bukkit/potion/PotionType.values
    pub fn values() -> &'static [&'static str] {
        &["AWKWARD", "FIRE_RESISTANCE", "HARMING", "HEALING", "INFESTED", "INVISIBILITY", "LEAPING", "LONG_FIRE_RESISTANCE", "LONG_INVISIBILITY", "LONG_LEAPING", "LONG_NIGHT_VISION", "LONG_POISON", "LONG_REGENERATION", "LONG_SLOW_FALLING", "LONG_SLOWNESS", "LONG_STRENGTH", "LONG_SWIFTNESS", "LONG_TURTLE_MASTER", "LONG_WATER_BREATHING", "LONG_WEAKNESS", "LUCK", "MUNDANE", "NIGHT_VISION", "OOZING", "POISON", "REGENERATION", "SLOW_FALLING", "SLOWNESS", "STRENGTH", "STRONG_HARMING", "STRONG_HEALING", "STRONG_LEAPING", "STRONG_POISON", "STRONG_REGENERATION", "STRONG_SLOWNESS", "STRONG_STRENGTH", "STRONG_SWIFTNESS", "STRONG_TURTLE_MASTER", "SWIFTNESS", "THICK", "TURTLE_MASTER", "WATER", "WATER_BREATHING", "WEAKNESS", "WEAVING", "WIND_CHARGED"]
    }
    /// bukkit: org/bukkit/potion/PotionType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const AWKWARD: &str = "AWKWARD";
    pub const FIRE_RESISTANCE: &str = "FIRE_RESISTANCE";
    pub const HARMING: &str = "HARMING";
    pub const HEALING: &str = "HEALING";
    pub const INFESTED: &str = "INFESTED";
    pub const INVISIBILITY: &str = "INVISIBILITY";
    pub const LEAPING: &str = "LEAPING";
    pub const LONG_FIRE_RESISTANCE: &str = "LONG_FIRE_RESISTANCE";
    pub const LONG_INVISIBILITY: &str = "LONG_INVISIBILITY";
    pub const LONG_LEAPING: &str = "LONG_LEAPING";
    pub const LONG_NIGHT_VISION: &str = "LONG_NIGHT_VISION";
    pub const LONG_POISON: &str = "LONG_POISON";
    pub const LONG_REGENERATION: &str = "LONG_REGENERATION";
    pub const LONG_SLOW_FALLING: &str = "LONG_SLOW_FALLING";
    pub const LONG_SLOWNESS: &str = "LONG_SLOWNESS";
    pub const LONG_STRENGTH: &str = "LONG_STRENGTH";
    pub const LONG_SWIFTNESS: &str = "LONG_SWIFTNESS";
    pub const LONG_TURTLE_MASTER: &str = "LONG_TURTLE_MASTER";
    pub const LONG_WATER_BREATHING: &str = "LONG_WATER_BREATHING";
    pub const LONG_WEAKNESS: &str = "LONG_WEAKNESS";
    pub const LUCK: &str = "LUCK";
    pub const MUNDANE: &str = "MUNDANE";
    pub const NIGHT_VISION: &str = "NIGHT_VISION";
    pub const OOZING: &str = "OOZING";
    pub const POISON: &str = "POISON";
    pub const REGENERATION: &str = "REGENERATION";
    pub const SLOW_FALLING: &str = "SLOW_FALLING";
    pub const SLOWNESS: &str = "SLOWNESS";
    pub const STRENGTH: &str = "STRENGTH";
    pub const STRONG_HARMING: &str = "STRONG_HARMING";
    pub const STRONG_HEALING: &str = "STRONG_HEALING";
    pub const STRONG_LEAPING: &str = "STRONG_LEAPING";
    pub const STRONG_POISON: &str = "STRONG_POISON";
    pub const STRONG_REGENERATION: &str = "STRONG_REGENERATION";
    pub const STRONG_SLOWNESS: &str = "STRONG_SLOWNESS";
    pub const STRONG_STRENGTH: &str = "STRONG_STRENGTH";
    pub const STRONG_SWIFTNESS: &str = "STRONG_SWIFTNESS";
    pub const STRONG_TURTLE_MASTER: &str = "STRONG_TURTLE_MASTER";
    pub const SWIFTNESS: &str = "SWIFTNESS";
    pub const THICK: &str = "THICK";
    pub const TURTLE_MASTER: &str = "TURTLE_MASTER";
    pub const WATER: &str = "WATER";
    pub const WATER_BREATHING: &str = "WATER_BREATHING";
    pub const WEAKNESS: &str = "WEAKNESS";
    pub const WEAVING: &str = "WEAVING";
    pub const WIND_CHARGED: &str = "WIND_CHARGED";
}

/// `org.bukkit.profile.PlayerTextures$SkinModel`.
pub mod player_textures_skin_model {
    /// bukkit: org/bukkit/profile/PlayerTextures$SkinModel.values
    pub fn values() -> &'static [&'static str] {
        &["CLASSIC", "SLIM"]
    }
    /// bukkit: org/bukkit/profile/PlayerTextures$SkinModel.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const CLASSIC: &str = "CLASSIC";
    pub const SLIM: &str = "SLIM";
}

/// `org.bukkit.scoreboard.DisplaySlot`.
pub mod display_slot {
    /// bukkit: org/bukkit/scoreboard/DisplaySlot.values
    pub fn values() -> &'static [&'static str] {
        &["PLAYER_LIST", "SIDEBAR", "BELOW_NAME", "SIDEBAR_TEAM_BLACK", "SIDEBAR_TEAM_DARK_BLUE", "SIDEBAR_TEAM_DARK_GREEN", "SIDEBAR_TEAM_DARK_AQUA", "SIDEBAR_TEAM_DARK_RED", "SIDEBAR_TEAM_DARK_PURPLE", "SIDEBAR_TEAM_GOLD", "SIDEBAR_TEAM_GRAY", "SIDEBAR_TEAM_DARK_GRAY", "SIDEBAR_TEAM_BLUE", "SIDEBAR_TEAM_GREEN", "SIDEBAR_TEAM_AQUA", "SIDEBAR_TEAM_RED", "SIDEBAR_TEAM_LIGHT_PURPLE", "SIDEBAR_TEAM_YELLOW", "SIDEBAR_TEAM_WHITE"]
    }
    /// bukkit: org/bukkit/scoreboard/DisplaySlot.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const PLAYER_LIST: &str = "PLAYER_LIST";
    pub const SIDEBAR: &str = "SIDEBAR";
    pub const BELOW_NAME: &str = "BELOW_NAME";
    pub const SIDEBAR_TEAM_BLACK: &str = "SIDEBAR_TEAM_BLACK";
    pub const SIDEBAR_TEAM_DARK_BLUE: &str = "SIDEBAR_TEAM_DARK_BLUE";
    pub const SIDEBAR_TEAM_DARK_GREEN: &str = "SIDEBAR_TEAM_DARK_GREEN";
    pub const SIDEBAR_TEAM_DARK_AQUA: &str = "SIDEBAR_TEAM_DARK_AQUA";
    pub const SIDEBAR_TEAM_DARK_RED: &str = "SIDEBAR_TEAM_DARK_RED";
    pub const SIDEBAR_TEAM_DARK_PURPLE: &str = "SIDEBAR_TEAM_DARK_PURPLE";
    pub const SIDEBAR_TEAM_GOLD: &str = "SIDEBAR_TEAM_GOLD";
    pub const SIDEBAR_TEAM_GRAY: &str = "SIDEBAR_TEAM_GRAY";
    pub const SIDEBAR_TEAM_DARK_GRAY: &str = "SIDEBAR_TEAM_DARK_GRAY";
    pub const SIDEBAR_TEAM_BLUE: &str = "SIDEBAR_TEAM_BLUE";
    pub const SIDEBAR_TEAM_GREEN: &str = "SIDEBAR_TEAM_GREEN";
    pub const SIDEBAR_TEAM_AQUA: &str = "SIDEBAR_TEAM_AQUA";
    pub const SIDEBAR_TEAM_RED: &str = "SIDEBAR_TEAM_RED";
    pub const SIDEBAR_TEAM_LIGHT_PURPLE: &str = "SIDEBAR_TEAM_LIGHT_PURPLE";
    pub const SIDEBAR_TEAM_YELLOW: &str = "SIDEBAR_TEAM_YELLOW";
    pub const SIDEBAR_TEAM_WHITE: &str = "SIDEBAR_TEAM_WHITE";
}

/// `org.bukkit.scoreboard.NameTagVisibility`.
pub mod name_tag_visibility {
    /// bukkit: org/bukkit/scoreboard/NameTagVisibility.values
    pub fn values() -> &'static [&'static str] {
        &["ALWAYS", "NEVER", "HIDE_FOR_OTHER_TEAMS", "HIDE_FOR_OWN_TEAM"]
    }
    /// bukkit: org/bukkit/scoreboard/NameTagVisibility.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALWAYS: &str = "ALWAYS";
    pub const NEVER: &str = "NEVER";
    pub const HIDE_FOR_OTHER_TEAMS: &str = "HIDE_FOR_OTHER_TEAMS";
    pub const HIDE_FOR_OWN_TEAM: &str = "HIDE_FOR_OWN_TEAM";
}

/// `org.bukkit.scoreboard.RenderType`.
pub mod render_type {
    /// bukkit: org/bukkit/scoreboard/RenderType.values
    pub fn values() -> &'static [&'static str] {
        &["INTEGER", "HEARTS"]
    }
    /// bukkit: org/bukkit/scoreboard/RenderType.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const INTEGER: &str = "INTEGER";
    pub const HEARTS: &str = "HEARTS";
}

/// `org.bukkit.scoreboard.Team$Option`.
pub mod team_option {
    /// bukkit: org/bukkit/scoreboard/Team$Option.values
    pub fn values() -> &'static [&'static str] {
        &["NAME_TAG_VISIBILITY", "DEATH_MESSAGE_VISIBILITY", "COLLISION_RULE"]
    }
    /// bukkit: org/bukkit/scoreboard/Team$Option.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const NAME_TAG_VISIBILITY: &str = "NAME_TAG_VISIBILITY";
    pub const DEATH_MESSAGE_VISIBILITY: &str = "DEATH_MESSAGE_VISIBILITY";
    pub const COLLISION_RULE: &str = "COLLISION_RULE";
}

/// `org.bukkit.scoreboard.Team$OptionStatus`.
pub mod team_option_status {
    /// bukkit: org/bukkit/scoreboard/Team$OptionStatus.values
    pub fn values() -> &'static [&'static str] {
        &["ALWAYS", "NEVER", "FOR_OTHER_TEAMS", "FOR_OWN_TEAM"]
    }
    /// bukkit: org/bukkit/scoreboard/Team$OptionStatus.valueOf
    pub fn value_of(name: &str) -> Option<&'static str> {
        values().iter().copied().find(|value| *value == name)
    }
    pub const ALWAYS: &str = "ALWAYS";
    pub const NEVER: &str = "NEVER";
    pub const FOR_OTHER_TEAMS: &str = "FOR_OTHER_TEAMS";
    pub const FOR_OWN_TEAM: &str = "FOR_OWN_TEAM";
}
