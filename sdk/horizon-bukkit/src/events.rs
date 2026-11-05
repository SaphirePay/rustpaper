//! Written by plugin-tools/tools/sdk_events.py: do not edit.
//!
//! Each event is a hook of the bridge and what its JSON says: see that tool's table.
#![allow(missing_docs, dead_code)]
use crate::entity::Entity;
use crate::event::Event;
use crate::{Block, Location, Player, World};
use horizon_mod::{Answer, Value, query};
use serde_json::json;

fn player_of(value: &Value) -> Option<Player> {
    Player::read(&value["player"]).or_else(|| Player::read(value)).or_else(|| Player::read(&query(json!({ "op": "player", "uuid": value["uuid"] }))))
}

fn block_at(world: &World, pos: &Value) -> Option<Block> {
    let at = |i: usize| pos[i].as_i64().map(|v| v as i32);
    Some(world.get_block_at(at(0)?, at(1)?, at(2)?))
}

/// bukkit: org/bukkit/event/entity/EntityDamageEvent$DamageCause.values, org/bukkit/event/entity/EntityDamageEvent$DamageCause.valueOf
///
/// The game's damage type as Bukkit's `DamageCause`.
pub fn damage_cause(damage_type: &str) -> &'static str {
    match damage_type.trim_start_matches("minecraft:") {
        "in_fire" | "campfire" => "FIRE",
        "on_fire" => "FIRE_TICK",
        "lava" => "LAVA",
        "hot_floor" => "HOT_FLOOR",
        "lightning_bolt" => "LIGHTNING",
        "fall" | "stalagmite" => "FALL",
        "falling_block" | "falling_anvil" | "falling_stalactite" => "FALLING_BLOCK",
        "drown" => "DROWNING",
        "in_wall" => "SUFFOCATION",
        "cramming" => "CRAMMING",
        "starve" => "STARVATION",
        "cactus" | "sweet_berry_bush" => "CONTACT",
        "fly_into_wall" => "FLY_INTO_WALL",
        "out_of_world" => "VOID",
        "generic_kill" => "KILL",
        "magic" | "indirect_magic" => "MAGIC",
        "wither" | "wither_skull" => "WITHER",
        "dragon_breath" => "DRAGON_BREATH",
        "freeze" => "FREEZE",
        "sonic_boom" => "SONIC_BOOM",
        "thorns" => "THORNS",
        "explosion" | "player_explosion" | "fireworks" | "bad_respawn_point" => "ENTITY_EXPLOSION",
        "arrow" | "trident" | "thrown" | "fireball" | "unattributed_fireball" | "mob_projectile" | "wind_charge" | "spit" => "PROJECTILE",
        "player_attack" | "mob_attack" | "mob_attack_no_aggro" | "sting" | "mace_smash" => "ENTITY_ATTACK",
        _ => "CUSTOM",
    }
}

pub use crate::fam_ev_world::AnvilDamagedEvent;
pub use crate::fam_ev_entity::AreaEffectCloudApplyEvent;
pub use crate::fam_ev_entity::ArrowBodyCountChangeEvent;
pub use crate::fam_ev_player::AsyncChatCommandDecorateEvent;
pub use crate::fam_ev_player::AsyncChatDecorateEvent;
pub use crate::fam_ev_player::AsyncChatEvent;
pub use crate::fam_ev_player::AsyncPlayerChatEvent;
pub use crate::fam_ev_player::AsyncPlayerChatPreviewEvent;
pub use crate::fam_paper::AsyncPlayerConnectionConfigureEvent;
pub use crate::fam_ev_player::AsyncPlayerPreLoginEvent;
pub use crate::fam_ev_world::AsyncPlayerSendCommandsEvent;
pub use crate::fam_ev_world::AsyncPlayerSendSuggestionsEvent;
pub use crate::fam_ev_player::AsyncPlayerSpawnLocationEvent;
pub use crate::fam_ev_world::AsyncServerDataFixerRemoveBlockEntityEvent;
pub use crate::fam_ev_world::AsyncStructureGenerateEvent;
pub use crate::fam_ev_world::AsyncStructureSpawnEvent;
pub use crate::fam_ev_world::AsyncTabCompleteEvent;
pub use crate::fam_ev_entity::BatToggleSleepEvent;
pub use crate::fam_ev_world::BeaconActivatedEvent;
pub use crate::fam_ev_world::BeaconDeactivatedEvent;
pub use crate::fam_ev_world::BeaconEffectEvent;
pub use crate::fam_ev_world::BellResonateEvent;
pub use crate::fam_ev_world::BellRevealRaiderEvent;
pub use crate::fam_ev_world::BellRingEvent;
pub use crate::fam_ev_world::BlockBreakBlockEvent;
pub use crate::fam_ev_world::BlockBreakEvent;
pub use crate::fam_ev_world::BlockBreakProgressUpdateEvent;
pub use crate::fam_ev_world::BlockBurnEvent;
pub use crate::fam_ev_world::BlockCanBuildEvent;
pub use crate::fam_ev_world::BlockCookEvent;
pub use crate::fam_ev_world::BlockDamageAbortEvent;
pub use crate::fam_ev_world::BlockDamageEvent;
pub use crate::fam_ev_world::BlockDestroyEvent;
pub use crate::fam_ev_world::BlockDispenseArmorEvent;
pub use crate::fam_ev_world::BlockDispenseEvent;
pub use crate::fam_ev_world::BlockDispenseLootEvent;
pub use crate::fam_ev_world::BlockDropItemEvent;
pub use crate::fam_ev_world::BlockExpEvent;
pub use crate::fam_ev_world::BlockExplodeEvent;
pub use crate::fam_ev_world::BlockFailedDispenseEvent;
pub use crate::fam_ev_world::BlockFertilizeEvent;
pub use crate::fam_ev_world::BlockIgniteEvent;
pub use crate::fam_ev_world::BlockLockCheckEvent;
pub use crate::fam_ev_world::BlockMultiPlaceEvent;
pub use crate::fam_ev_world::BlockPistonExtendEvent;
pub use crate::fam_ev_world::BlockPistonRetractEvent;
pub use crate::fam_ev_world::BlockPlaceEvent;
pub use crate::fam_ev_world::BlockPreDispenseEvent;
pub use crate::fam_ev_world::BlockReceiveGameEvent;
pub use crate::fam_ev_world::BlockRedstoneEvent;
pub use crate::fam_ev_world::BlockShearEntityEvent;
pub use crate::fam_ev_player::BrewEvent;
pub use crate::fam_ev_player::BrewingStandFuelEvent;
pub use crate::fam_ev_world::BrewingStartEvent;
pub use crate::fam_ev_world::BroadcastMessageEvent;
pub use crate::fam_ev_world::CampfireStartEvent;
pub use crate::fam_ev_player::CartographyItemEvent;
pub use crate::fam_ev_world::CauldronLevelChangeEvent;
pub use crate::fam_ev_world::ChunkPopulateEvent;
pub use crate::fam_ev_world::ChunkUnloadEvent;
pub use crate::fam_ev_world::ClockTimeSkipEvent;
pub use crate::fam_ev_world::CommandRegisteredEvent;
pub use crate::fam_ev_world::CompostItemEvent;
pub use crate::fam_core::ConversationAbandonedEvent;
pub use crate::fam_ev_player::CraftItemEvent;
pub use crate::fam_ev_world::CrafterCraftEvent;
pub use crate::fam_ev_entity::CreatureSpawnEvent;
pub use crate::fam_ev_entity::CreeperIgniteEvent;
pub use crate::fam_ev_entity::CreeperPowerEvent;
pub use crate::fam_ev_world::DragonEggFormEvent;
pub use crate::fam_ev_player::EnchantItemEvent;
pub use crate::fam_ev_entity::EnderDragonChangePhaseEvent;
pub use crate::fam_ev_entity::EnderDragonFireballHitEvent;
pub use crate::fam_ev_entity::EnderDragonFlameEvent;
pub use crate::fam_ev_entity::EnderDragonShootFireballEvent;
pub use crate::fam_ev_entity::EndermanAttackPlayerEvent;
pub use crate::fam_ev_entity::EndermanEscapeEvent;
pub use crate::fam_ev_world::EntitiesLoadEvent;
pub use crate::fam_ev_world::EntitiesUnloadEvent;
pub use crate::fam_ev_entity::EntityAirChangeEvent;
pub use crate::fam_ev_entity::EntityAttemptSmashAttackEvent;
pub use crate::fam_ev_entity::EntityAttemptSpinAttackEvent;
pub use crate::fam_ev_world::EntityBlockFormEvent;
pub use crate::fam_ev_entity::EntityBreakDoorEvent;
pub use crate::fam_ev_entity::EntityBreedEvent;
pub use crate::fam_ev_entity::EntityChangeBlockEvent;
pub use crate::fam_ev_entity::EntityCollideWithEntityEvent;
pub use crate::fam_ev_entity::EntityCombustByBlockEvent;
pub use crate::fam_ev_entity::EntityCombustByEntityEvent;
pub use crate::fam_ev_entity::EntityCombustEvent;
pub use crate::fam_ev_entity::EntityCompostItemEvent;
pub use crate::fam_ev_entity::EntityDamageByBlockEvent;
pub use crate::fam_ev_entity::EntityDamageByEntityEvent;
pub use crate::fam_ev_entity::EntityDamageEvent;
pub use crate::fam_ev_entity::EntityDamageItemEvent;
pub use crate::fam_ev_entity::EntityDeathEvent;
pub use crate::fam_ev_entity::EntityDismountEvent;
pub use crate::fam_ev_entity::EntityDropItemEvent;
pub use crate::fam_ev_entity::EntityDyeEvent;
pub use crate::fam_ev_entity::EntityEnterLoveModeEvent;
pub use crate::fam_ev_entity::EntityEquipmentChangedEvent;
pub use crate::fam_ev_entity::EntityExhaustionEvent;
pub use crate::fam_ev_entity::EntityExplodeEvent;
pub use crate::fam_ev_entity::EntityFertilizeEggEvent;
pub use crate::fam_ev_entity::EntityIgniteEvent;
pub use crate::fam_ev_entity::EntityInsideBlockEvent;
pub use crate::fam_ev_entity::EntityInteractEvent;
pub use crate::fam_ev_entity::EntityJumpEvent;
pub use crate::fam_ev_entity::EntityKnockbackByEntityEvent;
pub use crate::fam_ev_entity::EntityKnockbackEvent;
pub use crate::fam_ev_entity::EntityLandEvent;
pub use crate::fam_ev_entity::EntityLoadCrossbowEvent;
pub use crate::fam_ev_entity::EntityLungeEvent;
pub use crate::fam_ev_entity::EntityMountEvent;
pub use crate::fam_ev_entity::EntityMoveEvent;
pub use crate::fam_ev_entity::EntityPathfindEvent;
pub use crate::fam_ev_entity::EntityPickupItemEvent;
pub use crate::fam_ev_entity::EntityPlaceEvent;
pub use crate::fam_ev_entity::EntityPortalEnterEvent;
pub use crate::fam_ev_entity::EntityPortalEvent;
pub use crate::fam_ev_entity::EntityPortalExitEvent;
pub use crate::fam_ev_entity::EntityPortalReadyEvent;
pub use crate::fam_ev_entity::EntityPoseChangeEvent;
pub use crate::fam_ev_entity::EntityPotentSulfurLaunchEvent;
pub use crate::fam_ev_entity::EntityPotionEffectEvent;
pub use crate::fam_ev_entity::EntityPushedByEntityAttackEvent;
pub use crate::fam_ev_entity::EntityRegainHealthEvent;
pub use crate::fam_ev_entity::EntityResurrectEvent;
pub use crate::fam_ev_entity::EntityShootBowEvent;
pub use crate::fam_ev_entity::EntitySpawnEvent;
pub use crate::fam_ev_entity::EntitySpellCastEvent;
pub use crate::fam_ev_entity::EntityTameEvent;
pub use crate::fam_ev_entity::EntityTargetEvent;
pub use crate::fam_ev_entity::EntityTargetLivingEntityEvent;
pub use crate::fam_ev_entity::EntityTeleportEvent;
pub use crate::fam_ev_entity::EntityToggleGlideEvent;
pub use crate::fam_ev_entity::EntityToggleSitEvent;
pub use crate::fam_ev_entity::EntityToggleSwimEvent;
pub use crate::fam_ev_entity::EntityTransformEvent;
pub use crate::fam_ev_entity::EntityUnleashEvent;
pub use crate::fam_ev_entity::EntityZapEvent;
pub use crate::fam_ev_entity::ExpBottleEvent;
pub use crate::fam_ev_entity::ExperienceOrbMergeEvent;
pub use crate::fam_ev_entity::ExplosionPrimeEvent;
pub use crate::fam_ev_world::FillProfileEvent;
pub use crate::fam_ev_entity::FishHookStateChangeEvent;
pub use crate::fam_ev_world::FluidLevelChangeEvent;
pub use crate::fam_ev_entity::FoodLevelChangeEvent;
pub use crate::fam_ev_player::FurnaceBurnEvent;
pub use crate::fam_ev_player::FurnaceExtractEvent;
pub use crate::fam_ev_player::FurnaceStartSmeltEvent;
pub use crate::fam_ev_world::GS4QueryEvent;
pub use crate::fam_core::GameEvent;
pub use crate::fam_ev_world::GenericGameEvent;
pub use crate::fam_ev_world::HangingBreakByEntityEvent;
pub use crate::fam_ev_world::HangingBreakEvent;
pub use crate::fam_ev_world::HangingPlaceEvent;
pub use crate::fam_ev_player::HopperInventorySearchEvent;
pub use crate::fam_ev_entity::HorseJumpEvent;
pub use crate::fam_ev_player::IllegalPacketEvent;
pub use crate::fam_ev_player::InventoryClickEvent;
pub use crate::fam_ev_player::InventoryCloseEvent;
pub use crate::fam_ev_player::InventoryCreativeEvent;
pub use crate::fam_ev_player::InventoryDragEvent;
pub use crate::fam_ev_player::InventoryMoveItemEvent;
pub use crate::fam_ev_player::InventoryOpenEvent;
pub use crate::fam_ev_player::InventoryPickupItemEvent;
pub use crate::fam_ev_player::ItemCraftedEvent;
pub use crate::fam_ev_entity::ItemDespawnEvent;
pub use crate::fam_ev_entity::ItemMergeEvent;
pub use crate::fam_ev_entity::ItemSpawnEvent;
pub use crate::fam_ev_entity::ItemTransportingEntitySortEvent;
pub use crate::fam_ev_entity::ItemTransportingEntityValidateTargetEvent;
pub use crate::fam_ev_world::LightningStrikeEvent;
pub use crate::fam_ev_entity::LingeringPotionSplashEvent;
pub use crate::fam_ev_world::LookupProfileEvent;
pub use crate::fam_ev_world::LootGenerateEvent;
pub use crate::fam_paper::LootableInventoryReplenishEvent;
pub use crate::fam_ev_world::MapInitializeEvent;
pub use crate::fam_ev_world::NotePlayEvent;
pub use crate::fam_ev_world::PaperServerListPingEvent;
pub use crate::fam_ev_entity::PigZapEvent;
pub use crate::fam_ev_entity::PigZombieAngerEvent;
pub use crate::fam_ev_entity::PiglinBarterEvent;
pub use crate::fam_ev_player::PlayerAdvancementCriterionGrantEvent;
pub use crate::fam_ev_player::PlayerAdvancementDoneEvent;
pub use crate::fam_ev_player::PlayerAnimationEvent;
pub use crate::fam_ev_player::PlayerArmorChangeEvent;
pub use crate::fam_ev_player::PlayerArmorStandManipulateEvent;
pub use crate::fam_ev_player::PlayerAttackEntityCooldownResetEvent;
pub use crate::fam_ev_player::PlayerAttemptPickupItemEvent;
pub use crate::fam_ev_player::PlayerBedEnterEvent;
pub use crate::fam_ev_player::PlayerBedFailEnterEvent;
pub use crate::fam_ev_player::PlayerBedLeaveEvent;
pub use crate::fam_ev_player::PlayerBucketEmptyEvent;
pub use crate::fam_ev_player::PlayerBucketEntityEvent;
pub use crate::fam_ev_player::PlayerBucketFillEvent;
pub use crate::fam_ev_player::PlayerBundleItemSelectEvent;
pub use crate::fam_ev_player::PlayerChangeBeaconEffectEvent;
pub use crate::fam_ev_player::PlayerChangedMainHandEvent;
pub use crate::fam_ev_player::PlayerChangedWorldEvent;
pub use crate::fam_ev_player::PlayerChatEvent;
pub use crate::fam_ev_player::PlayerChatTabCompleteEvent;
pub use crate::fam_ev_player::PlayerChunkLoadEvent;
pub use crate::fam_ev_player::PlayerChunkUnloadEvent;
pub use crate::fam_ev_player::PlayerClientLoadedWorldEvent;
pub use crate::fam_ev_player::PlayerClientOptionsChangeEvent;
pub use crate::fam_ev_player::PlayerCodeOfConductSendEvent;
pub use crate::fam_ev_player::PlayerCommandSendEvent;
pub use crate::fam_ev_player::PlayerConnectionCloseEvent;
pub use crate::fam_paper::PlayerConnectionInitialConfigureEvent;
pub use crate::fam_ev_player::PlayerConnectionReconfigureEvent;
pub use crate::fam_paper::PlayerConnectionValidateLoginEvent;
pub use crate::fam_ev_player::PlayerCustomClickEvent;
pub use crate::fam_ev_entity::PlayerDeathEvent;
pub use crate::fam_ev_player::PlayerDeepSleepEvent;
pub use crate::fam_ev_player::PlayerDropItemEvent;
pub use crate::fam_ev_player::PlayerEditBookEvent;
pub use crate::fam_ev_player::PlayerEggThrowEvent;
pub use crate::fam_ev_player::PlayerElytraBoostEvent;
pub use crate::fam_ev_player::PlayerExpChangeEvent;
pub use crate::fam_ev_player::PlayerExpCooldownChangeEvent;
pub use crate::fam_ev_player::PlayerFailMoveEvent;
pub use crate::fam_ev_player::PlayerFishEvent;
pub use crate::fam_ev_player::PlayerFlowerPotManipulateEvent;
pub use crate::fam_ev_player::PlayerGameModeChangeEvent;
pub use crate::fam_ev_player::PlayerHandshakeEvent;
pub use crate::fam_ev_player::PlayerHarvestBlockEvent;
pub use crate::fam_ev_player::PlayerHideEntityEvent;
pub use crate::fam_ev_player::PlayerInputEvent;
pub use crate::fam_ev_player::PlayerInsertLecternBookEvent;
pub use crate::fam_ev_player::PlayerInteractEntityEvent;
pub use crate::fam_ev_player::PlayerInteractEvent;
pub use crate::fam_ev_player::PlayerInventorySlotChangeEvent;
pub use crate::fam_ev_player::PlayerItemBreakEvent;
pub use crate::fam_ev_player::PlayerItemConsumeEvent;
pub use crate::fam_ev_player::PlayerItemDamageEvent;
pub use crate::fam_ev_player::PlayerItemFrameChangeEvent;
pub use crate::fam_ev_player::PlayerItemGroupCooldownEvent;
pub use crate::fam_ev_player::PlayerItemHeldEvent;
pub use crate::fam_ev_player::PlayerItemMendEvent;
pub use crate::fam_ev_player::PlayerJumpEvent;
pub use crate::fam_ev_player::PlayerKickEvent;
pub use crate::fam_ev_player::PlayerLaunchProjectileEvent;
pub use crate::fam_ev_entity::PlayerLeashEntityEvent;
pub use crate::fam_ev_player::PlayerLecternPageChangeEvent;
pub use crate::fam_ev_player::PlayerLinksSendEvent;
pub use crate::fam_ev_player::PlayerLocaleChangeEvent;
pub use crate::fam_ev_player::PlayerLoomPatternSelectEvent;
pub use crate::fam_ev_player::PlayerMapFilledEvent;
pub use crate::fam_ev_player::PlayerMoveEvent;
pub use crate::fam_ev_player::PlayerNameEntityEvent;
pub use crate::fam_ev_entity::PlayerNaturallySpawnCreaturesEvent;
pub use crate::fam_ev_player::PlayerOpenSignEvent;
pub use crate::fam_ev_player::PlayerPickBlockEvent;
pub use crate::fam_ev_player::PlayerPickEntityEvent;
pub use crate::fam_ev_player::PlayerPickupArrowEvent;
pub use crate::fam_ev_player::PlayerPickupExperienceEvent;
pub use crate::fam_ev_player::PlayerPickupItemEvent;
pub use crate::fam_ev_player::PlayerPortalEvent;
pub use crate::fam_ev_player::PlayerPostRespawnEvent;
pub use crate::fam_ev_player::PlayerPurchaseEvent;
pub use crate::fam_ev_player::PlayerReadyArrowEvent;
pub use crate::fam_ev_player::PlayerRecipeBookClickEvent;
pub use crate::fam_ev_player::PlayerRecipeBookSettingsChangeEvent;
pub use crate::fam_ev_player::PlayerRecipeDiscoverEvent;
pub use crate::fam_ev_player::PlayerRegisterChannelEvent;
pub use crate::fam_ev_player::PlayerResourcePackStatusEvent;
pub use crate::fam_ev_player::PlayerRespawnEvent;
pub use crate::fam_ev_player::PlayerRiptideEvent;
pub use crate::fam_ev_player::PlayerServerFullCheckEvent;
pub use crate::fam_ev_player::PlayerSetSpawnEvent;
pub use crate::fam_ev_world::PlayerShearBlockEvent;
pub use crate::fam_ev_player::PlayerShearEntityEvent;
pub use crate::fam_ev_player::PlayerShieldDisableEvent;
pub use crate::fam_ev_player::PlayerShowEntityEvent;
pub use crate::fam_ev_player::PlayerSignCommandPreprocessEvent;
pub use crate::fam_ev_player::PlayerSignOpenEvent;
pub use crate::fam_ev_player::PlayerSpawnChangeEvent;
pub use crate::fam_ev_player::PlayerStartSpectatingEntityEvent;
pub use crate::fam_ev_player::PlayerStatisticIncrementEvent;
pub use crate::fam_ev_player::PlayerStonecutterRecipeSelectEvent;
pub use crate::fam_ev_player::PlayerStopSpectatingEntityEvent;
pub use crate::fam_ev_player::PlayerStopUsingItemEvent;
pub use crate::fam_ev_player::PlayerSwapHandItemsEvent;
pub use crate::fam_ev_player::PlayerSwapWithEquipmentSlotEvent;
pub use crate::fam_ev_player::PlayerTakeLecternBookEvent;
pub use crate::fam_ev_player::PlayerTeleportEndGatewayEvent;
pub use crate::fam_ev_player::PlayerTeleportEvent;
pub use crate::fam_ev_player::PlayerToggleEntityAgeLockEvent;
pub use crate::fam_ev_player::PlayerToggleFlightEvent;
pub use crate::fam_ev_player::PlayerToggleSneakEvent;
pub use crate::fam_ev_player::PlayerToggleSprintEvent;
pub use crate::fam_ev_player::PlayerTrackEntityEvent;
pub use crate::fam_ev_player::PlayerTradeEvent;
pub use crate::fam_ev_player::PlayerUnleashEntityEvent;
pub use crate::fam_ev_player::PlayerUnregisterChannelEvent;
pub use crate::fam_ev_player::PlayerUseUnknownEntityEvent;
pub use crate::fam_ev_player::PlayerVelocityEvent;
pub use crate::fam_ev_world::PluginDisableEvent;
pub use crate::fam_ev_world::PluginEnableEvent;
pub use crate::fam_ev_world::PortalCreateEvent;
pub use crate::fam_ev_entity::PotionSplashEvent;
pub use crate::fam_ev_entity::PreCreatureSpawnEvent;
pub use crate::fam_ev_world::PreFillProfileEvent;
pub use crate::fam_ev_world::PreLookupProfileEvent;
pub use crate::fam_ev_player::PrePlayerAttackEntityEvent;
pub use crate::fam_ev_entity::PreSpawnerSpawnEvent;
pub use crate::fam_ev_player::PrepareAnvilEvent;
pub use crate::fam_ev_player::PrepareGrindstoneEvent;
pub use crate::fam_ev_player::PrepareHopperTransferEvent;
pub use crate::fam_ev_player::PrepareInventoryResultEvent;
pub use crate::fam_ev_player::PrepareItemCraftEvent;
pub use crate::fam_ev_player::PrepareItemEnchantEvent;
pub use crate::fam_ev_player::PrepareSmithingEvent;
pub use crate::fam_ev_world::ProfileWhitelistVerifyEvent;
pub use crate::fam_ev_entity::ProjectileCollideEvent;
pub use crate::fam_ev_entity::ProjectileHitEvent;
pub use crate::fam_ev_entity::ProjectileLaunchEvent;
pub use crate::fam_ev_entity::PufferFishStateChangeEvent;
pub use crate::fam_ev_world::RaidFinishEvent;
pub use crate::fam_ev_world::RaidSpawnWaveEvent;
pub use crate::fam_ev_world::RaidStopEvent;
pub use crate::fam_paper::RegistrarEvent;
pub use crate::fam_paper::RegistryComposeEvent;
pub use crate::fam_paper::RegistryEntryAddEvent;
pub use crate::fam_ev_world::RemoteServerCommandEvent;
pub use crate::fam_ev_world::SculkBloomEvent;
pub use crate::fam_ev_world::ServerCommandEvent;
pub use crate::fam_ev_world::ServerTickEndEvent;
pub use crate::fam_ev_world::ServiceRegisterEvent;
pub use crate::fam_ev_world::ServiceUnregisterEvent;
pub use crate::fam_ev_entity::SheepDyeWoolEvent;
pub use crate::fam_ev_entity::SheepRegrowWoolEvent;
pub use crate::fam_ev_entity::ShulkerDuplicateEvent;
pub use crate::fam_ev_world::SignChangeEvent;
pub use crate::fam_ev_entity::SlimeChangeDirectionEvent;
pub use crate::fam_ev_entity::SlimePathfindEvent;
pub use crate::fam_ev_entity::SlimeSplitEvent;
pub use crate::fam_ev_entity::SlimeSwimEvent;
pub use crate::fam_ev_entity::SlimeTargetLivingEntityEvent;
pub use crate::fam_ev_entity::SlimeWanderEvent;
pub use crate::fam_ev_player::SmithItemEvent;
pub use crate::fam_ev_world::SpawnChangeEvent;
pub use crate::fam_ev_entity::SpawnerSpawnEvent;
pub use crate::fam_ev_world::SpongeAbsorbEvent;
pub use crate::fam_ev_entity::StriderTemperatureChangeEvent;
pub use crate::fam_ev_world::StructureGrowEvent;
pub use crate::fam_ev_world::StructuresLocateEvent;
pub use crate::fam_ev_entity::SulfurCubeSwallowItemEvent;
pub use crate::fam_ev_world::TNTPrimeEvent;
pub use crate::fam_ev_world::TabCompleteEvent;
pub use crate::fam_ev_entity::TameableDeathMessageEvent;
pub use crate::fam_ev_world::TargetHitEvent;
pub use crate::fam_ev_entity::ThrownEggHatchEvent;
pub use crate::fam_ev_world::ThunderChangeEvent;
pub use crate::fam_ev_world::TimeSkipEvent;
pub use crate::fam_ev_player::TradeSelectEvent;
pub use crate::fam_ev_entity::TrialSpawnerSpawnEvent;
pub use crate::fam_ev_entity::TurtleGoHomeEvent;
pub use crate::fam_ev_entity::TurtleLayEggEvent;
pub use crate::fam_ev_entity::TurtleStartDiggingEvent;
pub use crate::fam_ev_player::UncheckedSignChangeEvent;
pub use crate::fam_ev_world::UnknownCommandEvent;
pub use crate::fam_ev_world::VaultChangeStateEvent;
pub use crate::fam_ev_world::VaultDisplayItemEvent;
pub use crate::fam_ev_world::VehicleBlockCollisionEvent;
pub use crate::fam_ev_world::VehicleCreateEvent;
pub use crate::fam_ev_world::VehicleDamageEvent;
pub use crate::fam_ev_world::VehicleDestroyEvent;
pub use crate::fam_ev_world::VehicleEnterEvent;
pub use crate::fam_ev_world::VehicleEntityCollisionEvent;
pub use crate::fam_ev_world::VehicleExitEvent;
pub use crate::fam_ev_world::VehicleMoveEvent;
pub use crate::fam_ev_world::VehicleUpdateEvent;
pub use crate::fam_ev_entity::VillagerAcquireTradeEvent;
pub use crate::fam_ev_entity::VillagerCareerChangeEvent;
pub use crate::fam_ev_entity::VillagerReplenishTradeEvent;
pub use crate::fam_ev_entity::WardenAngerChangeEvent;
pub use crate::fam_ev_entity::WaterBottleSplashEvent;
pub use crate::fam_ev_world::WeatherChangeEvent;
pub use crate::fam_ev_world::WhitelistStateUpdateEvent;
pub use crate::fam_ev_world::WhitelistToggleEvent;
pub use crate::fam_ev_entity::WitchConsumePotionEvent;
pub use crate::fam_ev_entity::WitchReadyPotionEvent;
pub use crate::fam_ev_entity::WitchThrowPotionEvent;
pub use crate::fam_ev_world::WorldBorderBoundsChangeEvent;
pub use crate::fam_ev_world::WorldBorderBoundsChangeFinishEvent;
pub use crate::fam_ev_world::WorldBorderCenterChangeEvent;
pub use crate::fam_ev_world::WorldDifficultyChangeEvent;
pub use crate::fam_ev_world::WorldGameRuleChangeEvent;
pub use crate::fam_ev_world::WorldUnloadEvent;

/// bukkit: org/bukkit/event/block/BlockSpreadEvent
#[derive(Clone, Debug)]
pub struct BlockSpreadEvent {
    value: Value,
    cancelled: bool,
}

impl BlockSpreadEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["to"])
    }
    /// bukkit: org/bukkit/event/block/BlockSpreadEvent.getSource
    pub fn get_source(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["from"])
    }
    /// bukkit: org/bukkit/event/block/BlockSpreadEvent.getNewState, org/bukkit/event/block/BlockFormEvent.getNewState
    ///
    /// What the block becomes: the hook's `new` when it says, else what spreads (the source's block, as fire,
    /// grass and vines copy themselves).
    pub fn get_new_state(&self) -> Option<crate::BlockState> {
        let block = self.get_block()?;
        let new = match self.value["new"].as_str() {
            Some(new) => new.to_owned(),
            None => self.get_source().map_or_else(|| "minecraft:air".to_owned(), |source| source.get_block_data()),
        };
        Some(crate::BlockState::snapshot(block, &new))
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockSpreadEvent {
    const HOOK: &'static str = "block_spread";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/block/BlockFromToEvent
#[derive(Clone, Debug)]
pub struct BlockFromToEvent {
    value: Value,
    cancelled: bool,
}

impl BlockFromToEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["from"])
    }
    /// bukkit: org/bukkit/event/block/BlockFromToEvent.getToBlock
    pub fn get_to_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["to"])
    }
    /// bukkit: org/bukkit/event/block/BlockFromToEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockFromToEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockFromToEvent {
    const HOOK: &'static str = "fluid_flow";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/entity/EntityRemoveEvent
#[derive(Clone, Debug)]
pub struct EntityRemoveEvent {
    value: Value,
    cancelled: bool,
}

impl EntityRemoveEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
}

impl Event for EntityRemoveEvent {
    const HOOK: &'static str = "entity_unload";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/EntityRemoveFromWorldEvent
#[derive(Clone, Debug)]
pub struct EntityRemoveFromWorldEvent {
    value: Value,
    cancelled: bool,
}

impl EntityRemoveFromWorldEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
}

impl Event for EntityRemoveFromWorldEvent {
    const HOOK: &'static str = "entity_unload";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/EntityAddToWorldEvent
#[derive(Clone, Debug)]
pub struct EntityAddToWorldEvent {
    value: Value,
    cancelled: bool,
}

impl EntityAddToWorldEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
}

impl Event for EntityAddToWorldEvent {
    const HOOK: &'static str = "entity_load";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerUntrackEntityEvent
#[derive(Clone, Debug)]
pub struct PlayerUntrackEntityEvent {
    value: Value,
    cancelled: bool,
}

impl PlayerUntrackEntityEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/player/PlayerUntrackEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
}

impl Event for PlayerUntrackEntityEvent {
    const HOOK: &'static str = "tracking_stop";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: org/bukkit/event/world/WorldLoadEvent
#[derive(Clone, Debug)]
pub struct WorldLoadEvent {
    value: Value,
    cancelled: bool,
}

impl WorldLoadEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        World::of(self.value["level"].as_str().unwrap_or("minecraft:overworld"))
    }
}

impl Event for WorldLoadEvent {
    const HOOK: &'static str = "level_load";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: org/bukkit/event/world/WorldInitEvent
#[derive(Clone, Debug)]
pub struct WorldInitEvent {
    value: Value,
    cancelled: bool,
}

impl WorldInitEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/world/WorldEvent.getWorld
    pub fn get_world(&self) -> World {
        World::of(self.value["level"].as_str().unwrap_or("minecraft:overworld"))
    }
}

impl Event for WorldInitEvent {
    const HOOK: &'static str = "level_load";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: org/bukkit/event/world/WorldSaveEvent
#[derive(Clone, Debug)]
pub struct WorldSaveEvent {
    value: Value,
    cancelled: bool,
}

impl WorldSaveEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
}

impl Event for WorldSaveEvent {
    const HOOK: &'static str = "save_before";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: org/bukkit/event/server/ServerLoadEvent
#[derive(Clone, Debug)]
pub struct ServerLoadEvent {
    value: Value,
    cancelled: bool,
}

impl ServerLoadEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
}

impl Event for ServerLoadEvent {
    const HOOK: &'static str = "server_started";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: com/destroystokyo/paper/event/server/ServerTickStartEvent
#[derive(Clone, Debug)]
pub struct ServerTickStartEvent {
    value: Value,
    cancelled: bool,
}

impl ServerTickStartEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
}

impl Event for ServerTickStartEvent {
    const HOOK: &'static str = "tick_start";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: io/papermc/paper/event/server/ServerResourcesReloadedEvent
#[derive(Clone, Debug)]
pub struct ServerResourcesReloadedEvent {
    value: Value,
    cancelled: bool,
}

impl ServerResourcesReloadedEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
}

impl Event for ServerResourcesReloadedEvent {
    const HOOK: &'static str = "reload_end";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit~: io/papermc/paper/event/player/ChatEvent | the message can be refused but not changed
#[derive(Clone, Debug)]
pub struct ChatEvent {
    value: Value,
    cancelled: bool,
}

impl ChatEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    pub fn message(&self) -> &str {
        self.value["message"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ChatEvent {
    const HOOK: &'static str = "allow_chat";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/entity/FireworkExplodeEvent
#[derive(Clone, Debug)]
pub struct FireworkExplodeEvent {
    value: Value,
    cancelled: bool,
}

impl FireworkExplodeEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/FireworkExplodeEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
    /// bukkit: org/bukkit/event/entity/FireworkExplodeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/FireworkExplodeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for FireworkExplodeEvent {
    const HOOK: &'static str = "rocket_explode";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: io/papermc/paper/event/player/PlayerPickItemEvent
#[derive(Clone, Debug)]
pub struct PlayerPickItemEvent {
    value: Value,
    cancelled: bool,
}

impl PlayerPickItemEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
}

impl Event for PlayerPickItemEvent {
    const HOOK: &'static str = "packet_pick";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: io/papermc/paper/event/packet/ClientTickEndEvent
#[derive(Clone, Debug)]
pub struct ClientTickEndEvent {
    value: Value,
    cancelled: bool,
}

impl ClientTickEndEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
}

impl Event for ClientTickEndEvent {
    const HOOK: &'static str = "packet_tick_end";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}

/// bukkit: org/bukkit/event/block/BlockGrowEvent
#[derive(Clone, Debug)]
pub struct BlockGrowEvent {
    value: Value,
    cancelled: bool,
}

impl BlockGrowEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockGrowEvent.getNewState
    pub fn get_new_state(&self) -> Option<crate::BlockState> {
        block_at(&self.world(), &self.value["pos"]).map(|block| crate::BlockState::snapshot(block, self.value["new"].as_str().unwrap_or("minecraft:air")))
    }
    /// bukkit: org/bukkit/event/block/BlockGrowEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockGrowEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockGrowEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "grow").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/block/BlockFadeEvent
#[derive(Clone, Debug)]
pub struct BlockFadeEvent {
    value: Value,
    cancelled: bool,
}

impl BlockFadeEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockFadeEvent.getNewState
    pub fn get_new_state(&self) -> Option<crate::BlockState> {
        block_at(&self.world(), &self.value["pos"]).map(|block| crate::BlockState::snapshot(block, self.value["new"].as_str().unwrap_or("minecraft:air")))
    }
    /// bukkit: org/bukkit/event/block/BlockFadeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockFadeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockFadeEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "fade").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/block/BlockFormEvent
#[derive(Clone, Debug)]
pub struct BlockFormEvent {
    value: Value,
    cancelled: bool,
}

impl BlockFormEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockFormEvent.getNewState
    pub fn get_new_state(&self) -> Option<crate::BlockState> {
        block_at(&self.world(), &self.value["pos"]).map(|block| crate::BlockState::snapshot(block, self.value["new"].as_str().unwrap_or("minecraft:air")))
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockFormEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "form").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/block/LeavesDecayEvent
#[derive(Clone, Debug)]
pub struct LeavesDecayEvent {
    value: Value,
    cancelled: bool,
}

impl LeavesDecayEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/LeavesDecayEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/LeavesDecayEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for LeavesDecayEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "decay").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/block/MoistureChangeEvent
#[derive(Clone, Debug)]
pub struct MoistureChangeEvent {
    value: Value,
    cancelled: bool,
}

impl MoistureChangeEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/MoistureChangeEvent.getNewState
    pub fn get_new_state(&self) -> Option<crate::BlockState> {
        block_at(&self.world(), &self.value["pos"]).map(|block| crate::BlockState::snapshot(block, self.value["new"].as_str().unwrap_or("minecraft:air")))
    }
    /// bukkit: org/bukkit/event/block/MoistureChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/MoistureChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for MoistureChangeEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "moisture").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit~: org/bukkit/event/block/BlockPhysicsEvent | every change of the world's block ticks
#[derive(Clone, Debug)]
pub struct BlockPhysicsEvent {
    value: Value,
    cancelled: bool,
}

impl BlockPhysicsEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/block/BlockPhysicsEvent.getChangedBlockData
    pub fn get_changed_block_data(&self) -> &str {
        self.value["new"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/block/BlockPhysicsEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/block/BlockPhysicsEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BlockPhysicsEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/raid/RaidTriggerEvent
#[derive(Clone, Debug)]
pub struct RaidTriggerEvent {
    value: Value,
    cancelled: bool,
}

impl RaidTriggerEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/raid/RaidTriggerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    pub fn get_raid_location(&self) -> Option<Location> {
        let at = |i: usize| self.value["pos"][i].as_f64();
        Some(Location::new(self.world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    pub fn get_amplifier(&self) -> i64 {
        self.value["amplifier"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/raid/RaidTriggerEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/raid/RaidTriggerEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for RaidTriggerEvent {
    const HOOK: &'static str = "raid_trigger";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: io/papermc/paper/event/entity/ElderGuardianAppearanceEvent
#[derive(Clone, Debug)]
pub struct ElderGuardianAppearanceEvent {
    value: Value,
    cancelled: bool,
}

impl ElderGuardianAppearanceEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: io/papermc/paper/event/entity/ElderGuardianAppearanceEvent.getAffectedPlayer
    pub fn get_affected_player(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/entity/ElderGuardianAppearanceEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
    /// bukkit: io/papermc/paper/event/entity/ElderGuardianAppearanceEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/ElderGuardianAppearanceEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ElderGuardianAppearanceEvent {
    const HOOK: &'static str = "ee_elder";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityConstructEvent
#[derive(Clone, Debug)]
pub struct EntityConstructEvent {
    value: Value,
    cancelled: bool,
}

impl EntityConstructEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    pub fn get_entity_kind(&self) -> &str {
        self.value["mob"].as_str().unwrap_or_default()
    }
    pub fn get_location(&self) -> Option<Location> {
        let at = |i: usize| self.value["at"][i].as_f64();
        Some(Location::new(self.world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityConstructEvent.getBlocks
    pub fn get_blocks(&self) -> Vec<Block> {
        self.value["blocks"].as_array().into_iter().flatten().filter_map(|pos| block_at(&self.world(), pos)).collect()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityConstructEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityConstructEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityConstructEvent {
    const HOOK: &'static str = "ee_construct";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_pattern"]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "construct").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityEffectTickEvent | players, by what the tick does: a mob ticks inside its own move
#[derive(Clone, Debug)]
pub struct EntityEffectTickEvent {
    value: Value,
    cancelled: bool,
}

impl EntityEffectTickEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityEffectTickEvent.getEntity
    pub fn get_entity(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityEffectTickEvent.getType
    pub fn get_type(&self) -> &str {
        self.value["effect"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityEffectTickEvent.getAmplifier
    pub fn get_amplifier(&self) -> i64 {
        self.value["amplifier"].as_i64().unwrap_or(0)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityEffectTickEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityEffectTickEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityEffectTickEvent {
    const HOOK: &'static str = "ee_effect_tick";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit~: com/destroystokyo/paper/event/entity/EntityTeleportEndGatewayEvent | mobs and items: where they land is not given
#[derive(Clone, Debug)]
pub struct EntityTeleportEndGatewayEvent {
    value: Value,
    cancelled: bool,
}

impl EntityTeleportEndGatewayEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityTeleportEndGatewayEvent.getGateway
    pub fn get_gateway(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityTeleportEndGatewayEvent {
    const HOOK: &'static str = "ee_gateway";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/entity/EntityCreatePortalEvent
#[derive(Clone, Debug)]
pub struct EntityCreatePortalEvent {
    value: Value,
    cancelled: bool,
}

impl EntityCreatePortalEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/EntityCreatePortalEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
    /// bukkit: org/bukkit/event/entity/EntityCreatePortalEvent.getPortalType
    pub fn get_portal_type(&self) -> &str {
        self.value["portal"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/entity/EntityCreatePortalEvent.getBlocks
    pub fn get_blocks(&self) -> Vec<Block> {
        self.value["cells"].as_array().into_iter().flatten().filter_map(|pos| block_at(&self.world(), pos)).collect()
    }
    /// bukkit: org/bukkit/event/entity/EntityCreatePortalEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityCreatePortalEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityCreatePortalEvent {
    const HOOK: &'static str = "ee_create_portal";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityBreakEvent | paintings and item frames: a leash knot or a cushion breaking is not heard
#[derive(Clone, Debug)]
pub struct EntityBreakEvent {
    value: Value,
    cancelled: bool,
}

impl EntityBreakEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityBreakEvent.getCause
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityBreakEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityBreakEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityBreakEvent {
    const HOOK: &'static str = "ee_hanging";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityBreakByEntityEvent | paintings, by a player
#[derive(Clone, Debug)]
pub struct EntityBreakByEntityEvent {
    value: Value,
    cancelled: bool,
}

impl EntityBreakByEntityEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityBreakByEntityEvent.getRemover
    pub fn get_remover(&self) -> Option<Player> {
        player_of(&self.value)
    }
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityBreakByEntityEvent {
    const HOOK: &'static str = "ee_hanging";
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "entity").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/entity/EntityEnterBlockEvent
#[derive(Clone, Debug)]
pub struct EntityEnterBlockEvent {
    value: Value,
    cancelled: bool,
}

impl EntityEnterBlockEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
    /// bukkit: org/bukkit/event/entity/EntityEnterBlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&self.world(), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/entity/EntityEnterBlockEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityEnterBlockEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityEnterBlockEvent {
    const HOOK: &'static str = "ee_enter_block";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/SkeletonHorseTrapEvent
#[derive(Clone, Debug)]
pub struct SkeletonHorseTrapEvent {
    value: Value,
    cancelled: bool,
}

impl SkeletonHorseTrapEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/SkeletonHorseTrapEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &self.world()))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/SkeletonHorseTrapEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/SkeletonHorseTrapEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for SkeletonHorseTrapEvent {
    const HOOK: &'static str = "ee_trap";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/PhantomPreSpawnEvent
#[derive(Clone, Debug)]
pub struct PhantomPreSpawnEvent {
    value: Value,
    cancelled: bool,
}

impl PhantomPreSpawnEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PhantomPreSpawnEvent.getSpawningEntity
    pub fn get_spawning_entity(&self) -> Option<Player> {
        player_of(&self.value)
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PhantomPreSpawnEvent {
    const HOOK: &'static str = "ee_phantom";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

/// bukkit: io/papermc/paper/threadedregions/RegionizedServerInitEvent
#[derive(Clone, Debug)]
pub struct RegionizedServerInitEvent {
    value: Value,
    cancelled: bool,
}

impl RegionizedServerInitEvent {
    fn world(&self) -> World {
        World::of(self.value["level"].as_str().or(self.value["dimension"].as_str()).unwrap_or("minecraft:overworld"))
    }
}

impl Event for RegionizedServerInitEvent {
    const HOOK: &'static str = "server_started";
    fn read(value: &Value) -> Option<Self> {
        (true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
}
