//! Family ev_entity: `org.bukkit.event.entity` and Paper's entity events.
//!
//! A class written out here replaces its row in `events`, which then re-exports it. `EntityDeathEvent`,
//! `EntitySpawnEvent`, `PlayerDeathEvent`, `EntityToggleGlideEvent`, `FoodLevelChangeEvent` and
//! `EntityDamageByEntityEvent` also have an older, narrower cousin of the same name in `event` or
//! `event2`: the ones here are the fuller ones.
//!
//! An event in Java says `LivingEntity` for what may be a player or a mob: here that is an [`Actor`].
#![allow(missing_docs)]
use crate::entity::{Entity, EntityClass};
use crate::event::Event;
use crate::{Block, ItemStack, Location, Player, World};
use horizon_mod::{Answer, Value, query};
use serde_json::json;
use std::sync::{Arc, Mutex};

fn world_of(value: &Value) -> World {
    World::of(value["level"].as_str().unwrap_or("minecraft:overworld"))
}

fn block_at(world: &World, pos: &Value) -> Option<Block> {
    let at = |i: usize| pos[i].as_i64().map(|v| v as i32);
    Some(world.get_block_at(at(0)?, at(1)?, at(2)?))
}

fn location_at(world: &World, at: &Value) -> Option<Location> {
    Some(Location::new(world.clone(), at[0].as_f64()?, at[1].as_f64()?, at[2].as_f64()?, 0.0, 0.0))
}

fn player_by_uuid(uuid: &str) -> Option<Player> {
    Player::read(&query(json!({ "op": "player", "uuid": uuid })))
}

/// The row's last word on a firing: see `paper_fam_ev_entity.rs`.
fn verdict(value: &Value) -> Option<Answer> {
    value["ee_refused"].as_bool().map(|refused| if refused { Answer::Fail } else { Answer::Pass })
}

fn answer(cancelled: bool) -> Answer {
    if cancelled { Answer::Fail } else { Answer::Pass }
}

/// Who an event is about: Java's `LivingEntity`, which a player is too.
#[derive(Clone, Debug, PartialEq)]
pub enum Actor {
    Player(Player),
    Entity(Entity),
}

impl Actor {
    /// `{ "id": 7 }` or `{ "uuid": "..." }`, as the engine sends them.
    fn read(value: &Value, world: &World) -> Option<Self> {
        if let Some(id) = value["id"].as_u64() {
            return Some(Self::Entity(Entity::of(id, world)));
        }
        value["uuid"].as_str().and_then(player_by_uuid).map(Self::Player)
    }
    fn to_json(&self) -> Value {
        match self {
            Self::Player(player) => json!({ "uuid": player.uuid() }),
            Self::Entity(entity) => json!({ "id": entity.get_entity_id() }),
        }
    }
    pub fn as_entity(&self) -> Option<&Entity> {
        match self {
            Self::Entity(entity) => Some(entity),
            Self::Player(_) => None,
        }
    }
    pub fn as_player(&self) -> Option<&Player> {
        match self {
            Self::Player(player) => Some(player),
            Self::Entity(_) => None,
        }
    }
    /// A view of the entity as a Java cast: `None` for a player.
    pub fn cast<C: EntityClass>(&self) -> Option<C> {
        self.as_entity().map(Entity::cast)
    }
    pub fn get_location(&self) -> Option<Location> {
        match self {
            Self::Player(player) => Some(player.get_location()),
            Self::Entity(entity) => entity.get_location(),
        }
    }
}

// ---- Projectiles ----

/// bukkit: org/bukkit/event/entity/EntityShootBowEvent
#[derive(Clone, Debug)]
pub struct EntityShootBowEvent {
    value: Value,
    cancelled: bool,
    replaced: Option<Entity>,
    consume: bool,
}

impl EntityShootBowEvent {
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        Actor::read(&self.value["owner"], &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.getProjectile
    pub fn get_projectile(&self) -> Option<Entity> {
        self.replaced.clone().or_else(|| self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value))))
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.setProjectile
    pub fn set_projectile(&mut self, projectile: Entity) {
        self.replaced = Some(projectile);
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.getBow
    pub fn get_bow(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["weapon"])
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.getConsumable, org/bukkit/event/entity/EntityShootBowEvent.getArrowItem
    pub fn get_consumable(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["ammo"])
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.getArrowItem
    pub fn get_arrow_item(&self) -> Option<ItemStack> {
        self.get_consumable()
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.getForce
    ///
    /// A bow's draw, a mob's as well: a crossbow always shoots at full.
    pub fn get_force(&self) -> f32 {
        self.value["force"].as_f64().map_or(1.0, |force| force as f32)
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.shouldConsumeItem, org/bukkit/event/entity/EntityShootBowEvent.getConsumeArrow
    pub fn should_consume_item(&self) -> bool {
        self.consume && self.value["consumes"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.getConsumeArrow
    pub fn get_consume_arrow(&self) -> bool {
        self.should_consume_item()
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.setConsumeItem
    pub fn set_consume_item(&mut self, consume: bool) {
        self.consume = consume;
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.setConsumeArrow
    pub fn set_consume_arrow(&mut self, consume: bool) {
        self.consume = consume;
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityShootBowEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityShootBowEvent {
    const HOOK: &'static str = "ee_launch";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_no_wear"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["arrow"].as_bool() == Some(true) && !value["owner"].is_null()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), replaced: None, consume: true })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled || self.replaced.is_some())
    }
    fn finish(&self) {
        let would_use = self.value["consumes"].as_bool().unwrap_or(false);
        let stopped = self.cancelled || self.replaced.is_some();
        if stopped || (would_use && !self.consume) {
            // Paper draws the arrow before it asks, so a cancelled shot has paid for it.
            query(json!({ "op": "ee_keep_ammo", "cancelled": stopped, "refund": would_use && !self.cancelled }));
        }
    }
}

/// bukkit: org/bukkit/event/entity/ProjectileLaunchEvent
#[derive(Clone, Debug)]
pub struct ProjectileLaunchEvent {
    value: Value,
    cancelled: bool,
}

impl ProjectileLaunchEvent {
    /// bukkit: org/bukkit/event/entity/ProjectileLaunchEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// The shooter, Java's `getEntity().getShooter()`.
    pub fn get_shooter(&self) -> Option<Actor> {
        Actor::read(&self.value["owner"], &world_of(&self.value))
    }
    pub fn get_entity_kind(&self) -> &str {
        self.value["kind"].as_str().unwrap_or_default()
    }
    /// What was thrown, for a thrown potion: it has no entity a plugin can reach.
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    /// bukkit: org/bukkit/event/entity/ProjectileLaunchEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/ProjectileLaunchEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ProjectileLaunchEvent {
    const HOOK: &'static str = "ee_launch";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_launch_potion", "ee_launch_rocket"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/ProjectileHitEvent
#[derive(Clone, Debug)]
pub struct ProjectileHitEvent {
    value: Value,
    cancelled: bool,
}

impl ProjectileHitEvent {
    /// bukkit: org/bukkit/event/entity/ProjectileHitEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// The shooter, Java's `getEntity().getShooter()`.
    pub fn get_shooter(&self) -> Option<Actor> {
        Actor::read(&self.value["shooter"], &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/ProjectileHitEvent.getHitEntity
    pub fn get_hit_entity(&self) -> Option<Actor> {
        Actor::read(&self.value["hit"], &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/ProjectileHitEvent.getHitBlock
    pub fn get_hit_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["block"])
    }
    /// bukkit: org/bukkit/event/entity/ProjectileHitEvent.getHitBlockFace
    pub fn get_hit_block_face(&self) -> Option<&str> {
        self.value["face"].as_str()
    }
    /// bukkit~: org/bukkit/event/entity/ProjectileHitEvent.setCancelled | what the hit does is skipped and it flies on through, but an arrow that has stuck in a block stays stuck
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/ProjectileHitEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ProjectileHitEvent {
    const HOOK: &'static str = "ee_hit";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- What a mob's state did between one tick and the next ----

#[derive(Clone, Debug)]
pub(crate) struct MobMove {
    level: World,
    old: Value,
    new: Value,
}

#[derive(Default)]
struct MobStates {
    now: u64,
    seen: std::collections::HashMap<u64, Value>,
    moved: Arc<Vec<MobMove>>,
}

static MOB_STATES: Mutex<Option<MobStates>> = Mutex::new(None);

/// Each mob whose state differs from a tick ago, worked out once however many listeners ask.
fn mob_moves() -> Arc<Vec<MobMove>> {
    let mut all = MOB_STATES.lock().unwrap_or_else(|e| e.into_inner());
    let all = all.get_or_insert_default();
    let now = crate::scheduler::now();
    if all.now == now {
        return all.moved.clone();
    }
    all.now = now;
    let mut moved = Vec::new();
    let mut seen = std::collections::HashMap::new();
    for world in crate::get_worlds() {
        let rows = query(json!({ "op": "ee_mobs", "dimension": world.dimension() }));
        for row in rows.as_array().into_iter().flatten() {
            let Some(id) = row["id"].as_u64() else {
                continue;
            };
            if let Some(old) = all.seen.get(&id)
                && old != row
            {
                moved.push(MobMove { level: world.clone(), old: old.clone(), new: row.clone() });
            }
            seen.insert(id, row.clone());
        }
    }
    all.seen = seen;
    all.moved = Arc::new(moved);
    all.moved.clone()
}

/// bukkit: org/bukkit/event/entity/EntityTargetEvent
#[derive(Clone, Debug)]
pub struct EntityTargetEvent {
    value: Value,
    target: Value,
    cancelled: bool,
}

impl EntityTargetEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        known_entity(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityTargetEvent.getTarget
    pub fn get_target(&self) -> Option<Actor> {
        known_actor(&self.target)
    }
    /// bukkit: org/bukkit/event/entity/EntityTargetEvent.setTarget
    ///
    /// `None` makes it forget its target.
    pub fn set_target(&mut self, target: Option<&Actor>) {
        self.target = target.map_or(Value::Null, target_row);
        crate::event::carry("target", self.target.clone());
    }
    /// bukkit: org/bukkit/event/entity/EntityTargetEvent.getReason
    pub fn get_reason(&self) -> &str {
        self.value["reason"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    ///
    /// The mob's kind, as Java's `getEntityType`.
    pub fn get_entity_type(&self) -> &str {
        self.value["entity"]["kind"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/entity/EntityTargetEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityTargetEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityTargetEvent {
    const HOOK: &'static str = "ee_target";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), target: value["target"].clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityTargetLivingEntityEvent
#[derive(Clone, Debug)]
pub struct EntityTargetLivingEntityEvent {
    value: Value,
    target: Value,
    cancelled: bool,
}

impl EntityTargetLivingEntityEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        known_entity(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityTargetLivingEntityEvent.getTarget
    pub fn get_target(&self) -> Option<Actor> {
        known_actor(&self.target)
    }
    /// bukkit: org/bukkit/event/entity/EntityTargetLivingEntityEvent.setTarget
    ///
    /// `None` makes it forget its target.
    pub fn set_target(&mut self, target: Option<&Actor>) {
        self.target = target.map_or(Value::Null, target_row);
        crate::event::carry("target", self.target.clone());
    }
    pub fn get_reason(&self) -> &str {
        self.value["reason"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    ///
    /// The mob's kind, as Java's `getEntityType`.
    pub fn get_entity_type(&self) -> &str {
        self.value["entity"]["kind"].as_str().unwrap_or_default()
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

impl Event for EntityTargetLivingEntityEvent {
    const HOOK: &'static str = "ee_target";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), target: value["target"].clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// An entity as `ee_target` carries one.
fn target_row(actor: &Actor) -> Value {
    match actor {
        Actor::Player(player) => json!({ "uuid": player.uuid(), "name": player.get_name(), "player": true }),
        Actor::Entity(entity) => json!({ "id": entity.get_entity_id(), "dimension": entity.get_world().dimension(), "uuid": entity.get_unique_id().replace('-', "") }),
    }
}


/// bukkit: io/papermc/paper/event/entity/EntityMoveEvent
#[derive(Clone, Debug)]
pub struct EntityMoveEvent {
    row: MobMove,
    from: Location,
    to: Location,
    moved_to: bool,
    cancelled: bool,
}

fn rowed_location(world: &World, row: &Value) -> Location {
    let number = |i: usize| row["at"][i].as_f64().unwrap_or(0.0);
    Location::new(world.clone(), number(0), number(1), number(2), row["yaw"].as_f64().unwrap_or(0.0) as f32, row["pitch"].as_f64().unwrap_or(0.0) as f32)
}

impl EntityMoveEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.row.new["id"].as_u64().unwrap_or(0), &self.row.level)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.getFrom
    pub fn get_from(&self) -> &Location {
        &self.from
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.setFrom
    pub fn set_from(&mut self, from: Location) {
        self.from = from;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.getTo
    pub fn get_to(&self) -> &Location {
        &self.to
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.setTo
    ///
    /// Where it ends up: it's moved there once the listeners are done.
    pub fn set_to(&mut self, to: Location) {
        self.to = to;
        self.moved_to = true;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.hasChangedPosition
    pub fn has_changed_position(&self) -> bool {
        (self.from.x, self.from.y, self.from.z) != (self.to.x, self.to.y, self.to.z)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.hasExplicitlyChangedPosition
    pub fn has_explicitly_changed_position(&self) -> bool {
        self.has_changed_position()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.hasChangedBlock
    pub fn has_changed_block(&self) -> bool {
        let block = |at: &Location| (at.x.floor(), at.y.floor(), at.z.floor());
        block(&self.from) != block(&self.to)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.hasExplicitlyChangedBlock
    pub fn has_explicitly_changed_block(&self) -> bool {
        self.has_changed_block()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.hasChangedOrientation
    pub fn has_changed_orientation(&self) -> bool {
        (self.from.yaw, self.from.pitch) != (self.to.yaw, self.to.pitch)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityMoveEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityMoveEvent {
    const HOOK: &'static str = "tick";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(_: &Value) -> Vec<Self> {
        mob_moves()
            .iter()
            .filter(|row| (&row.old["at"], &row.old["yaw"], &row.old["pitch"]) != (&row.new["at"], &row.new["yaw"], &row.new["pitch"]))
            .map(|row| Self { row: row.clone(), from: rowed_location(&row.level, &row.old), to: rowed_location(&row.level, &row.new), moved_to: false, cancelled: crate::event::prior_cancelled() })
            .collect()
    }
    fn finish(&self) {
        if self.cancelled {
            self.get_entity().teleport(&self.from);
        } else if self.moved_to {
            self.get_entity().teleport(&self.to);
        }
    }
}

// ---- Items on the ground, between one tick and the next ----

#[derive(Default)]
struct ItemStates {
    now: u64,
    seen: std::collections::HashMap<u64, (World, Value)>,
    changes: Arc<ItemChanges>,
}

#[derive(Default)]
struct ItemChanges {
    despawning: Vec<(World, Value)>,
    /// What's gone and what it went into, with its size before and after.
    merged: Vec<(World, Value, Value)>,
}

static ITEM_STATES: Mutex<Option<ItemStates>> = Mutex::new(None);

/// One tick's changes, worked out once however many listeners ask.
fn item_changes() -> Arc<ItemChanges> {
    let mut all = ITEM_STATES.lock().unwrap_or_else(|e| e.into_inner());
    let all = all.get_or_insert_default();
    let now = crate::scheduler::now();
    if all.now == now {
        return all.changes.clone();
    }
    all.now = now;
    let mut seen = std::collections::HashMap::new();
    let mut changes = ItemChanges::default();
    for world in crate::get_worlds() {
        let rows = query(json!({ "op": "ee_items", "dimension": world.dimension() }));
        for row in rows.as_array().into_iter().flatten() {
            let Some(id) = row["id"].as_u64() else {
                continue;
            };
            // A tick from now, its age reaches the 6000 that ends it.
            if row["age"].as_i64() == Some(5999) {
                changes.despawning.push((world.clone(), row.clone()));
            }
            seen.insert(id, (world.clone(), row.clone()));
        }
    }
    let count = |row: &Value| row["stack"]["count"].as_i64().unwrap_or(0);
    let close = |a: &Value, b: &Value| (0..3).all(|i| (a["at"][i].as_f64().unwrap_or(0.0) - b["at"][i].as_f64().unwrap_or(0.0)).abs() < 1.5);
    for (id, (world, gone)) in &all.seen {
        if seen.contains_key(id) || gone["age"].as_i64() >= Some(5999) {
            continue;
        }
        let into = seen.iter().find(|(other, (here, row))| {
            *other != id && here == world && row["stack"]["item"] == gone["stack"]["item"] && close(row, gone) && all.seen.get(other).is_some_and(|(_, before)| count(before) + count(gone) == count(row))
        });
        if let Some((_, (_, row))) = into {
            changes.merged.push((world.clone(), gone.clone(), row.clone()));
        }
    }
    all.seen = seen;
    all.changes = Arc::new(changes);
    all.changes.clone()
}

/// bukkit: org/bukkit/event/entity/ItemSpawnEvent
#[derive(Clone, Debug)]
pub struct ItemSpawnEvent {
    value: Value,
    cancelled: bool,
}

impl ItemSpawnEvent {
    /// bukkit: org/bukkit/event/entity/ItemSpawnEvent.getEntity
    pub fn get_entity(&self) -> crate::entities::Item {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value)).cast()
    }
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
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

impl Event for ItemSpawnEvent {
    const HOOK: &'static str = "ee_item";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/ItemDespawnEvent
#[derive(Clone, Debug)]
pub struct ItemDespawnEvent {
    world: World,
    row: Value,
    cancelled: bool,
}

impl ItemDespawnEvent {
    /// bukkit: org/bukkit/event/entity/ItemDespawnEvent.getEntity
    pub fn get_entity(&self) -> crate::entities::Item {
        Entity::of(self.row["id"].as_u64().unwrap_or(0), &self.world).cast()
    }
    /// bukkit: org/bukkit/event/entity/ItemDespawnEvent.getLocation
    pub fn get_location(&self) -> Option<Location> {
        location_at(&self.world, &self.row["at"])
    }
    /// bukkit: org/bukkit/event/entity/ItemDespawnEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/ItemDespawnEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ItemDespawnEvent {
    const HOOK: &'static str = "tick";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(_: &Value) -> Vec<Self> {
        item_changes().despawning.iter().map(|(world, row)| Self { world: world.clone(), row: row.clone(), cancelled: crate::event::prior_cancelled() }).collect()
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ee_item_age", "dimension": self.world.dimension(), "id": self.row["id"], "age": 0 }));
        }
    }
}

/// bukkit: org/bukkit/event/entity/ItemMergeEvent
#[derive(Clone, Debug)]
pub struct ItemMergeEvent {
    value: Value,
    cancelled: bool,
}

impl ItemMergeEvent {
    /// bukkit: org/bukkit/event/entity/ItemMergeEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Item> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// What's about to merge.
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["entity"]["stack"])
    }
    /// bukkit: org/bukkit/event/entity/ItemMergeEvent.getTarget
    pub fn get_target(&self) -> Option<crate::entities::Item> {
        known_entity(&self.value["target"]).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/event/entity/ItemMergeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/ItemMergeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ItemMergeEvent {
    const HOOK: &'static str = "ee_item_merge";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- Thrown potions ----

/// bukkit: org/bukkit/event/entity/PotionSplashEvent
#[derive(Clone, Debug)]
pub struct PotionSplashEvent {
    value: Value,
    intensities: Vec<(Value, f64)>,
    cancelled: bool,
}

impl PotionSplashEvent {
    /// bukkit: org/bukkit/event/entity/PotionSplashEvent.getPotion
    pub fn get_potion(&self) -> crate::entities::ThrownPotion {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value)).cast()
    }
    /// bukkit: org/bukkit/event/entity/PotionSplashEvent.getEntity
    pub fn get_entity(&self) -> crate::entities::ThrownPotion {
        self.get_potion()
    }
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    /// bukkit: org/bukkit/event/entity/PotionSplashEvent.getAffectedEntities
    pub fn get_affected_entities(&self) -> Vec<Actor> {
        let world = world_of(&self.value);
        self.value["affected"].as_array().into_iter().flatten().filter_map(|row| Actor::read(&row["who"], &world)).collect()
    }
    /// bukkit: org/bukkit/event/entity/PotionSplashEvent.getIntensity
    ///
    /// How strongly it reaches them, 0 to 1: 0 for one it doesn't reach.
    pub fn get_intensity(&self, entity: &Actor) -> f64 {
        let who = entity.to_json();
        if let Some((_, intensity)) = self.intensities.iter().find(|(row, _)| *row == who) {
            return *intensity;
        }
        self.value["affected"].as_array().into_iter().flatten().find(|row| row["who"]["id"] == who["id"] && row["who"]["uuid"] == who["uuid"]).and_then(|row| row["intensity"].as_f64()).unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/event/entity/PotionSplashEvent.setIntensity
    pub fn set_intensity(&mut self, entity: &Actor, intensity: f64) {
        let who = entity.to_json();
        self.intensities.retain(|(row, _)| *row != who);
        self.intensities.push((who, intensity));
    }
    /// bukkit: org/bukkit/event/entity/PotionSplashEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/PotionSplashEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PotionSplashEvent {
    const HOOK: &'static str = "ee_splash";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_splash_player", "ee_splash_owed"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value).or_else(|| (!value["ee_owed"].is_null()).then_some(Answer::Pass))
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "splash").then(|| Self { value: value.clone(), intensities: Vec::new(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        for (who, intensity) in &self.intensities {
            query(json!({ "op": "ee_intensity", "who": who, "intensity": intensity }));
        }
    }
}

/// bukkit: org/bukkit/event/entity/LingeringPotionSplashEvent
#[derive(Clone, Debug)]
pub struct LingeringPotionSplashEvent {
    value: Value,
    allows_empty: bool,
    cancelled: bool,
}

impl LingeringPotionSplashEvent {
    /// bukkit: org/bukkit/event/entity/LingeringPotionSplashEvent.getEntity
    pub fn get_entity(&self) -> crate::entities::ThrownPotion {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value)).cast()
    }
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    /// bukkit: org/bukkit/event/entity/LingeringPotionSplashEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/LingeringPotionSplashEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for LingeringPotionSplashEvent {
    const HOOK: &'static str = "ee_splash";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "lingering").then(|| Self { value: value.clone(), allows_empty: false, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/ExpBottleEvent
#[derive(Clone, Debug)]
pub struct ExpBottleEvent {
    value: Value,
    experience: i64,
    effect: bool,
}

impl ExpBottleEvent {
    /// bukkit: org/bukkit/event/entity/ExpBottleEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/ExpBottleEvent.getExperience
    pub fn get_experience(&self) -> i64 {
        self.experience
    }
    /// bukkit: org/bukkit/event/entity/ExpBottleEvent.setExperience
    pub fn set_experience(&mut self, experience: i64) {
        self.experience = experience;
        crate::event::carry("exp", json!(experience));
    }
    /// bukkit: org/bukkit/event/entity/ExpBottleEvent.getShowEffect
    pub fn get_show_effect(&self) -> bool {
        self.effect
    }
    /// bukkit: org/bukkit/event/entity/ExpBottleEvent.setShowEffect
    pub fn set_show_effect(&mut self, effect: bool) {
        self.effect = effect;
        crate::event::carry("shows", json!(effect));
    }
}

impl Event for ExpBottleEvent {
    const HOOK: &'static str = "ee_splash";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "exp").then(|| Self { value: value.clone(), experience: value["exp"].as_i64().unwrap_or(0), effect: value["shows"] != false })
    }
}

/// bukkit~: org/bukkit/event/entity/AreaEffectCloudApplyEvent | the dragon's breath names only players, as the game's breath harms nothing else
#[derive(Clone, Debug)]
pub struct AreaEffectCloudApplyEvent {
    value: Value,
    affected: Vec<Actor>,
    cancelled: bool,
}

impl AreaEffectCloudApplyEvent {
    /// bukkit: org/bukkit/event/entity/AreaEffectCloudApplyEvent.getAffectedEntities
    ///
    /// Who it reaches this turn: take one out and it's spared.
    pub fn get_affected_entities(&mut self) -> &mut Vec<Actor> {
        &mut self.affected
    }
    /// Where the cloud is: it has no entity a plugin can reach.
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    /// bukkit: org/bukkit/event/entity/AreaEffectCloudApplyEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/AreaEffectCloudApplyEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for AreaEffectCloudApplyEvent {
    const HOOK: &'static str = "ee_cloud";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        let world = world_of(value);
        let affected = value["affected"].as_array().into_iter().flatten().filter_map(|who| Actor::read(who, &world)).collect();
        Some(Self { value: value.clone(), affected, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        let world = world_of(&self.value);
        let spared: Vec<Value> = self.value["affected"].as_array().into_iter().flatten().filter(|who| Actor::read(who, &world).is_some_and(|actor| !self.affected.contains(&actor))).cloned().collect();
        if !self.cancelled && !spared.is_empty() {
            query(json!({ "op": "ee_cloud_spare", "who": spared }));
        }
    }
}

/// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent
#[derive(Clone, Debug)]
pub struct EntityPotionEffectEvent {
    value: Value,
    overriding: Option<bool>,
    cancelled: bool,
}

impl EntityPotionEffectEvent {
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"]).or_else(|| self.value["uuid"].as_str().and_then(player_by_uuid).map(Actor::Player))
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.getModifiedType
    pub fn get_modified_type(&self) -> &str {
        self.value["effect"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.getNewEffect
    ///
    /// `None` for one that's ending.
    pub fn get_new_effect(&self) -> Option<crate::PotionEffect> {
        let (duration, amplifier) = (self.value["duration"].as_i64().unwrap_or(0), self.value["amplifier"].as_i64().unwrap_or(0));
        (self.value["ended"] != true).then(|| crate::PotionEffect::new(self.get_modified_type(), duration as i32, amplifier as i32, false, true))
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.getOldEffect
    pub fn get_old_effect(&self) -> Option<crate::PotionEffect> {
        let old = &self.value["old"];
        Some(crate::PotionEffect::new(self.get_modified_type(), old["duration"].as_i64()? as i32, old["amplifier"].as_i64()? as i32, false, true))
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.getAction
    pub fn get_action(&self) -> &str {
        self.value["action"].as_str().unwrap_or(if self.value["old"].is_null() { "ADDED" } else { "CHANGED" })
    }
    /// bukkit~: org/bukkit/event/entity/EntityPotionEffectEvent.getSource | who gave it, for a mob's hit, a mob's own gift, an arrow and a shulker bullet; a mob struck by a wither skull or an enchantment doesn't name them
    pub fn get_source(&self) -> Option<Actor> {
        known_actor(&self.value["source"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.getCause
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.setOverride
    pub fn set_override(&mut self, overriding: bool) {
        self.overriding = Some(overriding);
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.isOverride
    ///
    /// Whether the new effect replaces the old, as the game decides unless told.
    pub fn is_override(&self) -> bool {
        if let Some(overriding) = self.overriding {
            return overriding;
        }
        let old = &self.value["old"];
        let (level, was) = (self.value["amplifier"].as_i64(), old["amplifier"].as_i64());
        level > was || level == was && self.value["duration"].as_i64() > old["duration"].as_i64()
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityPotionEffectEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityPotionEffectEvent {
    const HOOK: &'static str = "ee_effect";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_why_drink", "ee_why_food", "ee_why_beacon", "ee_why_conduit", "ee_why_mob", "ee_why_command", "ee_why_totem", "ee_tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        value["ee_why"].is_boolean().then_some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        value["effect"].is_string().then(|| Self { value: value.clone(), overriding: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled || self.overriding == Some(false) && !self.value["old"].is_null())
    }
}

// ---- Spawns ----

/// What the game was last doing that spawns things.
static SPAWN_REASON: Mutex<&'static str> = Mutex::new("DEFAULT");

const SPAWN_HOOKS: &[&str] = &["ee_tick", "why_natural", "why_egg", "why_breed", "why_spawner", "why_golem", "why_raid", "why_phantom", "why_convert", "why_trader", "ee_natural", "ee_pre_spawner"];

fn note_reason(value: &Value) -> Option<Answer> {
    let reason = value["reason"].as_str()?;
    *SPAWN_REASON.lock().unwrap_or_else(|e| e.into_inner()) = ["NATURAL", "SPAWNER_EGG", "BREEDING", "SPAWNER", "VILLAGE_DEFENSE", "RAID", "CONVERSION"].into_iter().find(|known| *known == reason).unwrap_or("DEFAULT");
    Some(Answer::Pass)
}

/// bukkit: com/destroystokyo/paper/event/entity/PreCreatureSpawnEvent
#[derive(Clone, Debug)]
pub struct PreCreatureSpawnEvent {
    value: Value,
    cancelled: bool,
    abort: bool,
}

impl PreCreatureSpawnEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/PreCreatureSpawnEvent.getType
    pub fn get_type(&self) -> &str {
        self.value["kind"].as_str().unwrap_or_default()
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PreCreatureSpawnEvent.getSpawnLocation
    pub fn get_spawn_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PreCreatureSpawnEvent.getReason
    pub fn get_reason(&self) -> String {
        self.value["reason"].as_str().filter(|reason| !reason.is_empty()).map_or_else(spawn_reason, str::to_owned)
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PreCreatureSpawnEvent.shouldAbortSpawn
    pub fn should_abort_spawn(&self) -> bool {
        self.abort
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PreCreatureSpawnEvent.setShouldAbortSpawn
    pub fn set_should_abort_spawn(&mut self, abort: bool) {
        self.abort = abort;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PreCreatureSpawnEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PreCreatureSpawnEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PreCreatureSpawnEvent {
    const HOOK: &'static str = "ee_pre_spawn";
    fn more_hooks() -> &'static [&'static str] {
        SPAWN_HOOKS
    }
    fn intercept(value: &Value) -> Option<Answer> {
        note_reason(value).or_else(|| verdict(value))
    }
    fn read(value: &Value) -> Option<Self> {
        value["kind"].is_string().then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), abort: false })
    }
    fn finish(&self) {
        if self.abort {
            query(json!({ "op": "ee_hit_abort" }));
        }
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled || self.abort)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/PreSpawnerSpawnEvent
#[derive(Clone, Debug)]
pub struct PreSpawnerSpawnEvent {
    value: Value,
    cancelled: bool,
    abort: bool,
}

/// The spawner now working: its own hook comes before its mobs do.
static SPAWNER_AT: Mutex<Option<Value>> = Mutex::new(None);

impl PreSpawnerSpawnEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/PreSpawnerSpawnEvent.getSpawnerLocation
    pub fn get_spawner_location(&self) -> Option<Location> {
        let pos = SPAWNER_AT.lock().unwrap_or_else(|e| e.into_inner()).clone()?;
        location_at(&world_of(&self.value), &pos)
    }
    pub fn get_type(&self) -> &str {
        self.value["kind"].as_str().unwrap_or_default()
    }
    pub fn get_spawn_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    pub fn get_reason(&self) -> &'static str {
        "SPAWNER"
    }
    pub fn should_abort_spawn(&self) -> bool {
        self.abort
    }
    pub fn set_should_abort_spawn(&mut self, abort: bool) {
        self.abort = abort;
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

impl Event for PreSpawnerSpawnEvent {
    const HOOK: &'static str = "ee_pre_spawner";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_spawner_at"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if value["spawner"].is_array() {
            *SPAWNER_AT.lock().unwrap_or_else(|e| e.into_inner()) = Some(value["spawner"].clone());
            return Some(Answer::Pass);
        }
        None
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"].is_string() && value["reason"] == "SPAWNER").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), abort: false })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled || self.abort)
    }
}

/// bukkit: org/bukkit/event/entity/SpawnerSpawnEvent
#[derive(Clone, Debug)]
pub struct SpawnerSpawnEvent {
    value: Value,
    id: u64,
    cancelled: bool,
}

impl SpawnerSpawnEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.id, &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/SpawnerSpawnEvent.getSpawner
    pub fn get_spawner(&self) -> Option<crate::BlockState> {
        block_at(&world_of(&self.value), &self.value["spawner"]).map(|block| block.get_state())
    }
    pub fn get_location(&self) -> Option<Location> {
        self.get_entity().get_location()
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

impl Event for SpawnerSpawnEvent {
    const HOOK: &'static str = "ee_spawner_spawn";
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        value["ids"].as_array().into_iter().flatten().filter_map(Value::as_u64).map(|id| Self { value: value.clone(), id, cancelled: crate::event::prior_cancelled() }).collect()
    }
    fn finish(&self) {
        if self.cancelled {
            self.get_entity().remove();
        }
    }
}

// ---- Deaths ----

/// bukkit: org/bukkit/event/entity/EntityDeathEvent
#[derive(Clone, Debug)]
pub struct EntityDeathEvent {
    value: Value,
    drops: Vec<ItemStack>,
    exp: i64,
    revive: Option<f64>,
    sound: DeathSound,
    cancelled: bool,
}

/// The sound a death is about to make.
#[derive(Clone, Debug)]
struct DeathSound {
    play: bool,
    name: Option<String>,
    category: Option<String>,
    volume: f32,
    pitch: f32,
}

impl EntityDeathEvent {
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    pub fn get_entity_type(&self) -> &str {
        self.value["kind"].as_str().unwrap_or_default()
    }
    /// Who struck the last blow, Java's `getEntity().getKiller()` but for a mob too.
    pub fn get_killer(&self) -> Option<Actor> {
        Actor::read(&self.value["attacker"], &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getDrops
    ///
    /// What it drops: change this and what lies on the ground changes.
    pub fn get_drops(&mut self) -> &mut Vec<ItemStack> {
        &mut self.drops
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getDroppedExp
    pub fn get_dropped_exp(&self) -> i64 {
        self.exp
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setDroppedExp
    pub fn set_dropped_exp(&mut self, exp: i64) {
        self.exp = exp;
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getDamageSource
    pub fn get_damage_source(&self) -> crate::core2::DamageSource {
        let world = world_of(&self.value);
        let mut source = crate::core2::DamageSource::builder(self.value["damage_type"].as_str().unwrap_or("minecraft:generic"));
        match Actor::read(&self.value["attacker"], &world) {
            Some(Actor::Entity(entity)) => source = source.with_causing_entity(&entity),
            Some(Actor::Player(_)) => {
                if let Some(entity) = Entity::read(&json!({ "id": self.value["attacker"]["eid"], "dimension": world.dimension(), "uuid": self.value["attacker"]["uuid"] })) {
                    source = source.with_causing_entity(&entity);
                }
            }
            None => {}
        }
        if let Some(location) = location_at(&world, &self.value["from"]) {
            source = source.with_damage_location(&location);
        }
        source
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getReviveHealth
    ///
    /// What a cancel brings it back with: its full health unless set.
    pub fn get_revive_health(&self) -> f64 {
        self.revive.unwrap_or_else(|| self.get_entity().get_max_health())
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setReviveHealth
    pub fn set_revive_health(&mut self, health: f64) {
        self.revive = Some(health);
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.shouldPlayDeathSound
    pub fn should_play_death_sound(&self) -> bool {
        self.sound.play
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setShouldPlayDeathSound
    pub fn set_should_play_death_sound(&mut self, play: bool) {
        self.sound.play = play;
        crate::event::carry("play", json!(play));
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getDeathSound
    pub fn get_death_sound(&self) -> Option<&str> {
        self.sound.name.as_deref()
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setDeathSound
    pub fn set_death_sound(&mut self, sound: Option<&str>) {
        self.sound.name = sound.map(str::to_owned);
        crate::event::carry("sound", json!(sound));
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getDeathSoundCategory
    pub fn get_death_sound_category(&self) -> Option<&str> {
        self.sound.category.as_deref()
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setDeathSoundCategory
    pub fn set_death_sound_category(&mut self, category: Option<&str>) {
        self.sound.category = category.map(str::to_owned);
        crate::event::carry("category", json!(category));
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getDeathSoundVolume
    pub fn get_death_sound_volume(&self) -> f32 {
        self.sound.volume
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setDeathSoundVolume
    pub fn set_death_sound_volume(&mut self, volume: f32) {
        self.sound.volume = volume;
        crate::event::carry("volume", json!(volume));
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getDeathSoundPitch
    pub fn get_death_sound_pitch(&self) -> f32 {
        self.sound.pitch
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setDeathSoundPitch
    pub fn set_death_sound_pitch(&mut self, pitch: f32) {
        self.sound.pitch = pitch;
        crate::event::carry("voice_pitch", json!(pitch));
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityDeathEvent {
    const HOOK: &'static str = "ee_death";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_death_mark"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if !value["ee_mark"].is_null() {
            return Some(Answer::Pass);
        }
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        let drops = value["items"].as_array().into_iter().flatten().filter_map(|row| ItemStack::read(&row["stack"])).collect();
        Some(Self { value: value.clone(), drops, exp: value["exp"].as_i64().unwrap_or(0), revive: None, sound: DeathSound { play: value["play"] == true, name: value["sound"].as_str().map(str::to_owned), category: value["category"].as_str().map(str::to_owned), volume: value["volume"].as_f64().unwrap_or(1.0) as f32, pitch: value["voice_pitch"].as_f64().unwrap_or(1.0) as f32 }, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        let before: Vec<Value> = self.value["items"].as_array().into_iter().flatten().filter_map(|row| ItemStack::read(&row["stack"])).map(|stack| stack.to_json()).collect();
        let now: Vec<Value> = self.drops.iter().map(ItemStack::to_json).collect();
        let items = if self.cancelled { json!([]) } else if before == now { Value::Null } else { json!(now) };
        let exp = if self.cancelled { json!(0) } else if self.exp == self.value["exp"].as_i64().unwrap_or(0) { Value::Null } else { json!(self.exp) };
        let revive = if self.cancelled { json!(self.get_revive_health()) } else { Value::Null };
        query(json!({ "op": "ee_death_apply", "id": self.value["entity"], "at": self.value["at"], "item_ids": self.value["items"], "orb_ids": self.value["orbs"], "items": items, "exp": exp, "revive": revive }));
    }
}

/// What a listener asked of a player's death.
#[derive(Clone, Debug, Default)]
struct Dying {
    /// The server's line in place of the game's: `Some(None)` says nothing.
    line: Option<Option<String>>,
    new_level: Option<i32>,
    new_exp: Option<i32>,
    old_level: Option<i32>,
    old_bar: f64,
    new_total: Option<i32>,
    keeps: Vec<ItemStack>,
}

static DYING: Mutex<Option<(String, Dying)>> = Mutex::new(None);

/// bukkit: org/bukkit/event/entity/PlayerDeathEvent
#[derive(Clone, Debug)]
pub struct PlayerDeathEvent {
    value: Value,
    keep_inventory: bool,
    keep_level: bool,
    drop_exp: bool,
    show_message: bool,
    drops: Vec<ItemStack>,
    keeps: Vec<ItemStack>,
    screen: Option<String>,
    new_exp: i32,
    new_level: i32,
    new_total_exp: i32,
    line: Option<Option<String>>,
    cancelled: bool,
    revive: f64,
}

impl PlayerDeathEvent {
    /// bukkit~: org/bukkit/event/entity/PlayerDeathEvent.getPlayer, org/bukkit/event/entity/PlayerDeathEvent.getEntity | only their name and id work at this moment: anything else is asked of a player the game has lent out
    pub fn get_player(&self) -> Option<Player> {
        Player::read(&self.value)
    }
    /// bukkit~: org/bukkit/event/entity/PlayerDeathEvent.getEntity | as get_player
    pub fn get_entity(&self) -> Option<Player> {
        self.get_player()
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getLocation
    /// Where they fell.
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getDeathKey
    /// The translation key of how they died, when the game has one.
    pub fn get_death_key(&self) -> Option<&str> {
        self.value["cause"].as_str()
    }
    /// bukkit~: org/bukkit/event/entity/PlayerDeathEvent.deathMessage | said as plain text: its colours are lost
    pub fn death_message(&mut self, message: Option<&crate::Component>) {
        self.line = Some(message.map(crate::Component::plain));
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setDeathMessage
    pub fn set_death_message(&mut self, message: Option<&str>) {
        crate::event::line_set(2, message);
        self.line = Some(message.map(str::to_owned));
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getShowDeathMessages
    pub fn get_show_death_messages(&self) -> bool {
        self.show_message
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setShowDeathMessages
    pub fn set_show_death_messages(&mut self, show: bool) {
        self.show_message = show;
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getKeepInventory
    pub fn get_keep_inventory(&self) -> bool {
        self.keep_inventory
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setKeepInventory
    pub fn set_keep_inventory(&mut self, keep: bool) {
        self.keep_inventory = keep;
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getKeepLevel
    pub fn get_keep_level(&self) -> bool {
        self.keep_level
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setKeepLevel
    pub fn set_keep_level(&mut self, keep: bool) {
        self.keep_level = keep;
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.shouldDropExperience
    pub fn should_drop_experience(&self) -> bool {
        self.drop_exp
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setShouldDropExperience
    pub fn set_should_drop_experience(&mut self, drop: bool) {
        self.drop_exp = drop;
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getDrops
    /// What falls: change this and what falls changes.
    pub fn get_drops(&mut self) -> &mut Vec<ItemStack> {
        &mut self.drops
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getItemsToKeep
    /// What they have again as they respawn.
    pub fn get_items_to_keep(&mut self) -> &mut Vec<ItemStack> {
        &mut self.keeps
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getNewLevel
    pub fn get_new_level(&self) -> i32 {
        self.new_level
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setNewLevel
    pub fn set_new_level(&mut self, level: i32) {
        self.new_level = level;
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getNewExp
    pub fn get_new_exp(&self) -> i32 {
        self.new_exp
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setNewExp
    pub fn set_new_exp(&mut self, exp: i32) {
        self.new_exp = exp;
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.getNewTotalExp
    pub fn get_new_total_exp(&self) -> i32 {
        self.new_total_exp
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setNewTotalExp
    pub fn set_new_total_exp(&mut self, exp: i32) {
        self.new_total_exp = exp;
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.getReviveHealth
    pub fn get_revive_health(&self) -> f64 {
        self.revive
    }
    /// bukkit: org/bukkit/event/entity/EntityDeathEvent.setReviveHealth
    pub fn set_revive_health(&mut self, health: f64) {
        self.revive = health;
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/PlayerDeathEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PlayerDeathEvent {
    const HOOK: &'static str = "ee_player_dying";
    fn more_hooks() -> &'static [&'static str] {
        &["allow_game_message", "tick", "respawn"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        let mut dying = DYING.lock().unwrap_or_else(|e| e.into_inner());
        // The death's line, the first the server says after it.
        if value["message"].is_string() {
            let Some(line) = dying.as_mut().and_then(|(_, asked)| asked.line.take()) else {
                return Some(Answer::Pass);
            };
            if let Some(text) = line {
                crate::broadcast_message(&text);
            }
            return Some(Answer::Fail);
        }
        if value.is_null() {
            if let Some((_, asked)) = dying.as_mut() {
                asked.line = None;
            }
            query(json!({ "op": "ee_keep_done" }));
            return Some(Answer::Pass);
        }
        if !value["ee_refused"].is_null() {
            return verdict(value);
        }
        // Back on their feet: what the event asked of them after it.
        if value["ee_dying"].is_null() {
            if let Some((uuid, asked)) = dying.take() {
                if value["uuid"].as_str() == Some(uuid.as_str()) {
                    let name = value["name"].as_str().unwrap_or_default();
                    if let Some(level) = asked.old_level.or(asked.new_level) {
                        horizon_mod::run_command(&format!("xp set {name} {level} levels"));
                    }
                    if let Some(exp) = asked.new_exp {
                        horizon_mod::run_command(&format!("xp set {name} {exp} points"));
                    }
                    if asked.old_level.is_some() {
                        query(json!({ "op": "player_set", "uuid": uuid, "field": "exp_progress", "value": asked.old_bar }));
                    }
                    if let Some(total) = asked.new_total {
                        query(json!({ "op": "player_set", "uuid": uuid, "field": "exp_total", "value": total }));
                    }
                    if let Some(player) = value["uuid"].as_str().and_then(player_by_uuid) {
                        for stack in &asked.keeps {
                            player.get_inventory().add_item(stack);
                        }
                    }
                } else {
                    *dying = Some((uuid, asked));
                }
            }
            return Some(Answer::Pass);
        }
        None
    }
    fn read(value: &Value) -> Option<Self> {
        (value["ee_dying"].as_bool() == Some(true)).then(|| Self {
            value: value.clone(),
            keep_inventory: value["keep"].as_bool().unwrap_or(false),
            keep_level: false,
            drop_exp: true,
            show_message: value["show"].as_bool().unwrap_or(true),
            drops: value["drops"].as_array().into_iter().flatten().filter_map(ItemStack::read).collect(),
            keeps: Vec::new(),
            screen: None,
            new_exp: 0,
            new_level: 0,
            new_total_exp: 0,
            line: None,
            cancelled: crate::event::prior_cancelled(),
            revive: 20.0,
        })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        let before: Vec<Value> = self.value["drops"].as_array().into_iter().flatten().filter_map(ItemStack::read).map(|stack| stack.to_json()).collect();
        let now: Vec<Value> = self.drops.iter().map(ItemStack::to_json).collect();
        let keep_rule = self.value["keep"].as_bool().unwrap_or(false);
        query(json!({
            "op": "ee_player_dying",
            "keep": self.keep_inventory && !keep_rule,
            "no_exp": !self.drop_exp,
            "drops": if before == now { Value::Null } else { json!(now) },
            "revive": if self.cancelled { json!(self.revive) } else { Value::Null },
        }));
        // With its message off, the death screen has no line to show either.
        let screen = if self.show_message { self.screen.as_deref() } else { Some("") };
        if let Some(text) = screen {
            query(json!({ "op": "ee_death_screen", "uuid": self.value["uuid"], "text": text }));
        }
        let line = if self.show_message { self.line.clone() } else { Some(None) };
        let old_level = self.keep_level.then(|| self.value["xp_level"].as_i64().unwrap_or(0) as i32);
        let new_exp = (self.new_exp != 0).then_some(self.new_exp);
        let new_total = (self.new_total_exp != 0).then_some(self.new_total_exp);
        let new_level = (self.new_level != 0).then_some(self.new_level);
        let asked = Dying { line, new_level, new_exp, old_level, old_bar: self.value["xp_progress"].as_f64().unwrap_or(0.0), new_total, keeps: self.keeps.clone() };
        *DYING.lock().unwrap_or_else(|e| e.into_inner()) = Some((self.value["uuid"].as_str().unwrap_or_default().to_owned(), asked));
    }
}

/// bukkit: org/bukkit/event/entity/EntityTransformEvent
#[derive(Clone, Debug)]
pub struct EntityTransformEvent {
    value: Value,
    cancelled: bool,
}

impl EntityTransformEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["from"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    pub fn get_entity_kind(&self) -> &str {
        self.value["kind"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/entity/EntityTransformEvent.getTransformedEntity
    pub fn get_transformed_entity(&self) -> Option<Entity> {
        self.value["to"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/EntityTransformEvent.getTransformedEntities
    pub fn get_transformed_entities(&self) -> Vec<Entity> {
        self.get_transformed_entity().into_iter().collect()
    }
    /// bukkit: org/bukkit/event/entity/EntityTransformEvent.getTransformReason
    pub fn get_transform_reason(&self) -> &'static str {
        let name = |key: &str| self.value[key].as_str().unwrap_or_default().trim_start_matches("minecraft:").to_owned();
        match (name("kind").as_str(), name("new_kind").as_str()) {
            ("zombie", "drowned") => "DROWNED",
            ("husk", "zombie") => "DROWNED",
            ("skeleton", "stray") => "FROZEN",
            ("villager", "zombie_villager") => "INFECTION",
            ("zombie_villager", "villager") => "CURED",
            ("piglin" | "piglin_brute", "zombified_piglin") => "PIGLIN_ZOMBIFIED",
            ("hoglin", "zoglin") => "PIGLIN_ZOMBIFIED",
            ("tadpole", "frog") => "METAMORPHOSIS",
            ("mooshroom", "cow") => "SHEARED",
            ("pig", "zombified_piglin") | ("villager", "witch") | ("mooshroom", "mooshroom") => "LIGHTNING",
            _ => "UNKNOWN",
        }
    }
    /// bukkit~: org/bukkit/event/entity/EntityTransformEvent.setCancelled | the new mob stands by then: a cancel takes it away and brings the old one back as it was
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityTransformEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityTransformEvent {
    const HOOK: &'static str = "mob_conversion";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_convert"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        (value["ee_from"] == true).then_some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn finish(&self) {
        if self.cancelled {
            query(json!({ "op": "ee_unconvert", "dimension": world_of(&self.value).dimension(), "to": self.value["to"] }));
        }
    }
}

// ---- What a player's body and seat did between one tick and the next ----

#[derive(Clone, Debug)]
struct BodyMove {
    old: Value,
    new: Value,
}

#[derive(Default)]
struct Bodies {
    now: u64,
    seen: std::collections::HashMap<String, Value>,
    moved: Arc<Vec<BodyMove>>,
}

static BODIES: Mutex<Option<Bodies>> = Mutex::new(None);

fn body_moves() -> Arc<Vec<BodyMove>> {
    let mut all = BODIES.lock().unwrap_or_else(|e| e.into_inner());
    let all = all.get_or_insert_default();
    let now = crate::scheduler::now();
    if all.now == now {
        return all.moved.clone();
    }
    all.now = now;
    let mut moved = Vec::new();
    let mut seen = std::collections::HashMap::new();
    for row in query(json!({ "op": "ee_players" })).as_array().into_iter().flatten() {
        let Some(uuid) = row["uuid"].as_str() else {
            continue;
        };
        if let Some(old) = all.seen.get(uuid)
            && old != row
        {
            moved.push(BodyMove { old: old.clone(), new: row.clone() });
        }
        seen.insert(uuid.to_owned(), row.clone());
    }
    all.seen = seen;
    all.moved = Arc::new(moved);
    all.moved.clone()
}

fn rider_command(rider: &str, vehicle: &Value, world: &World, mount: bool) {
    if !mount {
        horizon_mod::run_command(&format!("ride {rider} dismount"));
    } else if let Some(id) = vehicle.as_u64() {
        horizon_mod::run_command(&format!("ride {rider} mount {}", Entity::of(id, world).get_unique_id()));
    }
}

/// A change of seat: a player's or a mob's, the same to a listener.
#[derive(Clone, Debug)]
struct SeatChange {
    level: World,
    rider: Value,
    vehicle: Value,
}

impl SeatChange {
    fn rider_name(&self) -> String {
        match Actor::read(&self.rider, &self.level) {
            Some(Actor::Player(player)) => player.get_name().to_owned(),
            Some(Actor::Entity(entity)) => entity.get_unique_id(),
            None => String::new(),
        }
    }
}

/// Who was heard boarding or leaving ahead of the change: the tick's look at who sits where leaves them out.
static SEATS_HEARD: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());

fn seat_key(rider: &Value) -> String {
    rider["uuid"].as_str().map_or_else(|| rider["id"].to_string(), str::to_owned)
}

fn note_seat(rider: &Value) {
    SEATS_HEARD.lock().unwrap_or_else(|e| e.into_inner()).push((crate::scheduler::now(), seat_key(rider)));
}

fn seat_heard(rider: &Value) -> bool {
    let now = crate::scheduler::now();
    let mut heard = SEATS_HEARD.lock().unwrap_or_else(|e| e.into_inner());
    heard.retain(|(tick, _)| now.saturating_sub(*tick) <= 2);
    let key = seat_key(rider);
    heard.iter().any(|(_, rider)| *rider == key)
}

fn seat_changes(mounting: bool) -> Vec<SeatChange> {
    let mut out = Vec::new();
    let overworld = World::of("minecraft:overworld");
    for row in body_moves().iter() {
        let vehicle = if mounting { &row.new["vehicle"] } else { &row.old["vehicle"] };
        let other = if mounting { &row.old["vehicle"] } else { &row.new["vehicle"] };
        if !vehicle.is_null() && other.is_null() {
            out.push(SeatChange { level: World::of(row.new["dimension"].as_str().unwrap_or("minecraft:overworld")), rider: json!({ "uuid": row.new["uuid"] }), vehicle: vehicle.clone() });
        }
    }
    for row in mob_moves().iter() {
        let vehicle = if mounting { &row.new["mount"] } else { &row.old["mount"] };
        let other = if mounting { &row.old["mount"] } else { &row.new["mount"] };
        if !vehicle.is_null() && other.is_null() {
            out.push(SeatChange { level: row.level.clone(), rider: json!({ "id": row.new["id"] }), vehicle: vehicle.clone() });
        }
    }
    let _ = overworld;
    out
}

/// bukkit: org/bukkit/event/entity/EntityMountEvent
#[derive(Clone, Debug)]
pub struct EntityMountEvent {
    seat: SeatChange,
    asked: bool,
    cancelled: bool,
}

impl EntityMountEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.seat.rider).or_else(|| Actor::read(&self.seat.rider, &self.seat.level))
    }
    /// bukkit: org/bukkit/event/entity/EntityMountEvent.getMount
    pub fn get_mount(&self) -> Option<Entity> {
        self.seat.vehicle.as_u64().map(|id| Entity::of(id, &self.seat.level))
    }
    /// bukkit: org/bukkit/event/entity/EntityMountEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityMountEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityMountEvent {
    const HOOK: &'static str = "tick";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_mount", "ee_tick"]
    }
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        if value["rider"].is_object() {
            let level = world_of(&value["rider"]);
            let vehicle = value["mount"]["id"].clone();
            let rider = value["rider"].clone();
            note_seat(&rider);
            return vec![Self { seat: SeatChange { level, rider, vehicle }, asked: true, cancelled: crate::event::prior_cancelled() }];
        }
        if !value["ee_tick"].is_null() {
            return Vec::new();
        }
        seat_changes(true).into_iter().filter(|seat| !seat_heard(&seat.rider)).map(|seat| Self { seat, asked: false, cancelled: crate::event::prior_cancelled() }).collect()
    }
    fn answer(&self) -> Answer {
        if self.asked { answer(self.cancelled) } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.cancelled && !self.asked {
            rider_command(&self.seat.rider_name(), &self.seat.vehicle, &self.seat.level, false);
        }
    }
}

/// bukkit: org/bukkit/event/entity/EntityDismountEvent
#[derive(Clone, Debug)]
pub struct EntityDismountEvent {
    seat: SeatChange,
    asked: bool,
    cancelled: bool,
}

impl EntityDismountEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.seat.rider).or_else(|| Actor::read(&self.seat.rider, &self.seat.level))
    }
    /// bukkit: org/bukkit/event/entity/EntityDismountEvent.getDismounted
    pub fn get_dismounted(&self) -> Option<Entity> {
        self.seat.vehicle.as_u64().map(|id| Entity::of(id, &self.seat.level))
    }
    /// bukkit: org/bukkit/event/entity/EntityDismountEvent.isCancellable
    ///
    /// Not when the vehicle has gone from under them.
    pub fn is_cancellable(&self) -> bool {
        self.asked
    }
    /// bukkit: org/bukkit/event/entity/EntityDismountEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled && self.asked;
    }
    /// bukkit: org/bukkit/event/entity/EntityDismountEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityDismountEvent {
    const HOOK: &'static str = "tick";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_dismount", "ee_tick"]
    }
    fn read(_: &Value) -> Option<Self> {
        None
    }
    fn read_all(value: &Value) -> Vec<Self> {
        if value["rider"].is_object() {
            let level = world_of(&value["rider"]);
            let vehicle = value["mount"]["id"].clone();
            let rider = value["rider"].clone();
            note_seat(&rider);
            return vec![Self { seat: SeatChange { level, rider, vehicle }, asked: value["forced"] != true, cancelled: crate::event::prior_cancelled() }];
        }
        if !value["ee_tick"].is_null() {
            return Vec::new();
        }
        seat_changes(false).into_iter().filter(|seat| !seat_heard(&seat.rider)).map(|seat| Self { seat, asked: false, cancelled: crate::event::prior_cancelled() }).collect()
    }
    fn answer(&self) -> Answer {
        if self.asked { answer(self.cancelled) } else { Answer::Pass }
    }
    fn finish(&self) {
        if self.cancelled && !self.asked {
            rider_command(&self.seat.rider_name(), &self.seat.vehicle, &self.seat.level, true);
        }
    }
}

/// bukkit: org/bukkit/event/entity/EntityToggleGlideEvent
#[derive(Clone, Debug)]
pub struct EntityToggleGlideEvent {
    value: Value,
    cancelled: bool,
}

impl EntityToggleGlideEvent {
    /// bukkit: org/bukkit/event/entity/EntityToggleGlideEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityToggleGlideEvent.isGliding
    pub fn is_gliding(&self) -> bool {
        self.value["gliding"] == true
    }
    /// bukkit: org/bukkit/event/entity/EntityToggleGlideEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityToggleGlideEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityToggleGlideEvent {
    const HOOK: &'static str = "ee_glide";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityToggleSwimEvent
#[derive(Clone, Debug)]
pub struct EntityToggleSwimEvent {
    value: Value,
    cancelled: bool,
}

impl EntityToggleSwimEvent {
    /// bukkit: org/bukkit/event/entity/EntityToggleSwimEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityToggleSwimEvent.isSwimming
    pub fn is_swimming(&self) -> bool {
        self.value["swimming"] == true
    }
    /// bukkit: org/bukkit/event/entity/EntityToggleSwimEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityToggleSwimEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityToggleSwimEvent {
    const HOOK: &'static str = "ee_swim";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityPoseChangeEvent
#[derive(Clone, Debug)]
pub struct EntityPoseChangeEvent {
    value: Value,
}

impl EntityPoseChangeEvent {
    /// bukkit: org/bukkit/event/entity/EntityPoseChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPoseChangeEvent.getPose
    ///
    /// The new pose, by Java's name of it.
    pub fn get_pose(&self) -> String {
        if let Some(number) = self.value["pose_id"].as_u64() {
            return crate::enums::pose::values().get(number as usize).copied().unwrap_or("STANDING").to_owned();
        }
        let name = self.value["pose"].as_str().unwrap_or("Standing");
        if name == "Crouching" {
            return "SNEAKING".to_owned();
        }
        let mut out = String::new();
        for (at, letter) in name.char_indices() {
            if letter.is_uppercase() && at > 0 {
                out.push('_');
            }
            out.push(letter.to_ascii_uppercase());
        }
        out
    }
}

impl Event for EntityPoseChangeEvent {
    const HOOK: &'static str = "ee_pose";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(),  })
    }
}

// ---- Events made elsewhere, and what they lacked ----

const DRAGON_PHASES: [&str; 11] = ["CIRCLING", "STRAFING", "FLY_TO_PORTAL", "LAND_ON_PORTAL", "LEAVE_PORTAL", "BREATH_ATTACK", "SEARCH_FOR_BREATH_ATTACK_TARGET", "ROAR_BEFORE_ATTACK", "CHARGE_PLAYER", "DYING", "HOVER"];

fn dragon_phase(number: i64) -> &'static str {
    usize::try_from(number).ok().and_then(|index| DRAGON_PHASES.get(index)).copied().unwrap_or("HOVER")
}

const DYE_COLORS: [&str; 16] = ["WHITE", "ORANGE", "MAGENTA", "LIGHT_BLUE", "YELLOW", "LIME", "PINK", "GRAY", "LIGHT_GRAY", "CYAN", "PURPLE", "BLUE", "BROWN", "GREEN", "RED", "BLACK"];

/// bukkit: io/papermc/paper/event/entity/EntityDyeEvent
#[derive(Clone, Debug)]
pub struct EntityDyeEvent {
    value: Value,
    color: String,
    cancelled: bool,
}

impl EntityDyeEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityDyeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDyeEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        Player::read(&self.value["player"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDyeEvent.getColor
    pub fn get_color(&self) -> &str {
        &self.color
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDyeEvent.setColor
    pub fn set_color(&mut self, color: &str) {
        self.color = color.to_owned();
        if let Some(number) = DYE_COLORS.iter().position(|name| *name == color) {
            crate::event::carry("color", json!(number));
        }
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDyeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDyeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

fn dye_name(value: &Value) -> String {
    value["color"].as_u64().and_then(|number| DYE_COLORS.get(number as usize)).copied().unwrap_or("WHITE").to_owned()
}

impl Event for EntityDyeEvent {
    const HOOK: &'static str = "ee_dye";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), color: dye_name(value), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/SheepDyeWoolEvent
#[derive(Clone, Debug)]
pub struct SheepDyeWoolEvent {
    value: Value,
    color: String,
    cancelled: bool,
}

impl SheepDyeWoolEvent {
    /// bukkit: org/bukkit/event/entity/SheepDyeWoolEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        Player::read(&self.value["player"])
    }
    pub fn get_color(&self) -> &str {
        &self.color
    }
    pub fn set_color(&mut self, color: &str) {
        self.color = color.to_owned();
        if let Some(number) = DYE_COLORS.iter().position(|name| *name == color) {
            crate::event::carry("color", json!(number));
        }
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

impl Event for SheepDyeWoolEvent {
    const HOOK: &'static str = "ee_dye";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "minecraft:sheep").then(|| Self { value: value.clone(), color: dye_name(value), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

impl crate::events::EntityRemoveEvent {
    /// bukkit~: org/bukkit/event/entity/EntityRemoveEvent.getCause | `DEATH` for one that has died, else `UNLOAD`: the game doesn't say more
    pub fn get_cause(&self) -> &'static str {
        match self.get_entity() {
            Some(entity) if entity.is_dead() => "DEATH",
            _ => "UNLOAD",
        }
    }
}

impl crate::event2::EntityDamageByEntityEvent {
    /// bukkit~: org/bukkit/event/entity/EntityDamageByEntityEvent.getDamageSource | a player's blow, as this event only hears those: the causing player isn't an `Entity` here, see `get_damager`
    pub fn get_damage_source(&self) -> crate::core2::DamageSource {
        crate::core2::DamageSource::builder("minecraft:player_attack").with_damage_location(&self.get_damager().get_location())
    }
}

// ---- Breeding, on the moment the game decides it ----

/// bukkit: org/bukkit/event/entity/EntityBreedEvent
#[derive(Clone, Debug)]
pub struct EntityBreedEvent {
    value: Value,
    experience: i32,
    cancelled: bool,
}

impl EntityBreedEvent {
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.getEntity
    ///
    /// The child: the mother for those that lay eggs, as their breed has no child yet.
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["child"].as_u64().map(|id| Entity::of(id, &world_of(&self.value))).or_else(|| self.get_mother())
    }
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.getMother
    pub fn get_mother(&self) -> Option<Entity> {
        self.value["mother"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.getFather
    pub fn get_father(&self) -> Option<Entity> {
        self.value["father"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.getBreeder
    pub fn get_breeder(&self) -> Option<Actor> {
        Actor::read(&self.value["breeder"], &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.getBredWith
    ///
    /// What it was fed, as the game gave it: `None` if no player fed it.
    pub fn get_bred_with(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["item"])
    }
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.getExperience
    pub fn get_experience(&self) -> i32 {
        self.experience
    }
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.setExperience
    pub fn set_experience(&mut self, experience: i32) {
        self.experience = experience;
    }
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityBreedEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityBreedEvent {
    const HOOK: &'static str = "ee_breed";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_breed_orb", "ee_add_orb", "ee_feed"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value).or_else(|| (!value["ee_fed"].is_null()).then_some(Answer::Pass))
    }
    fn read(value: &Value) -> Option<Self> {
        value["mother"].is_u64().then(|| Self { value: value.clone(), experience: value["exp"].as_i64().unwrap_or(0) as i32, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        query(json!({ "op": "ee_breed_xp", "exp": self.experience }));
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent
#[derive(Clone, Debug)]
pub struct EntityFertilizeEggEvent {
    value: Value,
    experience: i32,
    cancelled: bool,
}

impl EntityFertilizeEggEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["mother"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.getMother
    pub fn get_mother(&self) -> Option<Entity> {
        self.get_entity()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.getFather
    pub fn get_father(&self) -> Option<Entity> {
        self.value["father"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.getBreeder
    pub fn get_breeder(&self) -> Option<Player> {
        self.value["breeder"]["uuid"].as_str().and_then(player_by_uuid)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.getBredWith
    ///
    /// What it was fed, as the game gave it: `None` if no player fed it.
    pub fn get_bred_with(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["item"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.getExperience
    pub fn get_experience(&self) -> i32 {
        self.experience
    }
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.setExperience
    pub fn set_experience(&mut self, experience: i32) {
        self.experience = experience;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityFertilizeEggEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityFertilizeEggEvent {
    const HOOK: &'static str = "ee_breed";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_breed_orb", "ee_add_orb", "ee_feed"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value).or_else(|| (!value["ee_fed"].is_null()).then_some(Answer::Pass))
    }
    fn read(value: &Value) -> Option<Self> {
        (value["egg"].as_bool() == Some(true)).then(|| Self { value: value.clone(), experience: value["exp"].as_i64().unwrap_or(0) as i32, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        query(json!({ "op": "ee_breed_xp", "exp": self.experience }));
    }
}

// ---- What a mob's tick asks of the game ----

/// bukkit: org/bukkit/event/entity/EntityTameEvent
#[derive(Clone, Debug)]
pub struct EntityTameEvent {
    value: Value,
    cancelled: bool,
}

impl EntityTameEvent {
    /// bukkit: org/bukkit/event/entity/EntityTameEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        known_entity(&self.value["entity"]).or_else(|| self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value))))
    }
    /// bukkit: org/bukkit/event/entity/EntityTameEvent.getOwner
    pub fn get_owner(&self) -> Option<Player> {
        Player::read(&self.value["by"]).or_else(|| self.value["owner"].as_str().or(self.value["uuid"].as_str()).and_then(player_by_uuid))
    }
    /// bukkit: org/bukkit/event/entity/EntityTameEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityTameEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityTameEvent {
    const HOOK: &'static str = "ee_mob";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_tamed", "ee_tame", "ee_tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "tame").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityBreakDoorEvent
#[derive(Clone, Debug)]
pub struct EntityBreakDoorEvent {
    value: Value,
    cancelled: bool,
}

impl EntityBreakDoorEvent {
    /// bukkit: org/bukkit/event/entity/EntityBreakDoorEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
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

impl Event for EntityBreakDoorEvent {
    const HOOK: &'static str = "ee_mob";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "break_door").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityDropItemEvent
#[derive(Clone, Debug)]
pub struct EntityDropItemEvent {
    value: Value,
    cancelled: bool,
}

impl EntityDropItemEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/EntityDropItemEvent.getItemDrop
    pub fn get_item_drop(&self) -> Option<crate::entities::Item> {
        self.value["item"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)).cast())
    }
    /// What it is: the stack of `get_item_drop`.
    pub fn get_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    /// bukkit: org/bukkit/event/entity/EntityDropItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityDropItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityDropItemEvent {
    const HOOK: &'static str = "ee_mob";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "drop").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- Explosions, lightning, portals, leads ----

/// bukkit: org/bukkit/event/entity/ExplosionPrimeEvent
#[derive(Clone, Debug)]
pub struct ExplosionPrimeEvent {
    value: Value,
    radius: f32,
    fire: bool,
    cancelled: bool,
}

impl ExplosionPrimeEvent {
    /// bukkit~: org/bukkit/event/entity/ExplosionPrimeEvent.getEntity | on `EntityExplodeEvent`'s terms: what has left the world by now reads as its id, kind and place
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    pub fn get_entity_type(&self) -> Option<&str> {
        self.value["entity_kind"].as_str()
    }
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["at"])
    }
    /// bukkit: org/bukkit/event/entity/ExplosionPrimeEvent.getRadius
    pub fn get_radius(&self) -> f32 {
        self.radius
    }
    /// bukkit: org/bukkit/event/entity/ExplosionPrimeEvent.setRadius
    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
    /// bukkit: org/bukkit/event/entity/ExplosionPrimeEvent.getFire
    pub fn get_fire(&self) -> bool {
        self.fire
    }
    /// bukkit: org/bukkit/event/entity/ExplosionPrimeEvent.setFire
    pub fn set_fire(&mut self, fire: bool) {
        self.fire = fire;
    }
    /// bukkit: org/bukkit/event/entity/ExplosionPrimeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/ExplosionPrimeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ExplosionPrimeEvent {
    const HOOK: &'static str = "ee_explode";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_mob", "ee_blast_shot", "ee_blast_tnt", "ee_blast_cart", "ee_blast_crystal", "ee_blast_end"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value).or_else(|| value["ee_why"].is_boolean().then_some(Answer::Pass))
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), radius: value["radius"].as_f64().unwrap_or(0.0) as f32, fire: value["fire"].as_bool().unwrap_or(false), cancelled: crate::event::prior_cancelled() })
    }
    fn finish(&self) {
        query(json!({ "op": "ee_explosion", "cancelled": self.cancelled, "radius": self.radius, "fire": self.fire }));
    }
}

/// bukkit: org/bukkit/event/entity/PigZapEvent
#[derive(Clone, Debug)]
pub struct PigZapEvent {
    value: Value,
    cancelled: bool,
}

impl PigZapEvent {
    /// bukkit: org/bukkit/event/entity/PigZapEvent.getEntity
    pub fn get_entity(&self) -> crate::entities::Pig {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value)).cast()
    }
    /// bukkit: org/bukkit/event/entity/PigZapEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/PigZapEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PigZapEvent {
    const HOOK: &'static str = "ee_mob";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "zap" && value["mob"] == "minecraft:pig").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/EntityZapEvent
#[derive(Clone, Debug)]
pub struct EntityZapEvent {
    value: Value,
    cancelled: bool,
}

impl EntityZapEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityZapEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityZapEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityZapEvent {
    const HOOK: &'static str = "ee_mob";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "zap" && value["mob"] == "minecraft:villager").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityPortalEvent
#[derive(Clone, Debug)]
pub struct EntityPortalEvent {
    value: Value,
    cancelled: bool,
}

impl EntityPortalEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        known_entity(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.getPortalType
    pub fn get_portal_type(&self) -> &str {
        self.value["portal"].as_str().unwrap_or("NETHER")
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.setTo
    pub fn set_to(&mut self, to: Option<&Location>) {
        match to {
            Some(to) => {
                self.value["to"] = json!([to.x, to.y, to.z]);
                self.value["to_level"] = json!(to.get_world().dimension());
                crate::event::carry("to", self.value["to"].clone());
                crate::event::carry("to_level", self.value["to_level"].clone());
            }
            None => {
                self.value["to"] = Value::Null;
                crate::event::carry("to", Value::Null);
            }
        }
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.setSearchRadius
    pub fn set_search_radius(&mut self, radius: i32) {
        self.value["search"] = json!(radius);
        crate::event::carry("search", json!(radius));
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.setCreationRadius
    pub fn set_creation_radius(&mut self, radius: i32) {
        self.value["creation"] = json!(radius);
        crate::event::carry("creation", json!(radius));
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.setCanCreatePortal
    pub fn set_can_create_portal(&mut self, can: bool) {
        self.value["can_create"] = json!(can);
        crate::event::carry("can_create", json!(can));
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

impl Event for EntityPortalEvent {
    const HOOK: &'static str = "ee_portal";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityPortalExitEvent
#[derive(Clone, Debug)]
pub struct EntityPortalExitEvent {
    value: Value,
    cancelled: bool,
}

impl EntityPortalExitEvent {
    pub fn get_entity_kind(&self) -> &str {
        self.value["kind"].as_str().unwrap_or_default()
    }
    pub fn get_unique_id(&self) -> String {
        crate::dashed(self.value["uuid"].as_str().unwrap_or_default())
    }
    pub fn get_from(&self) -> Option<Location> {
        location_at(&World::of(self.value["from_level"].as_str()?), &self.value["from"])
    }
    pub fn get_to(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["to"])
    }
    /// Within the level it's landing in.
    pub fn set_to(&mut self, to: &Location) {
        self.value["to"] = json!([to.x, to.y, to.z]);
        crate::event::carry("to", self.value["to"].clone());
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalExitEvent.getBefore
    pub fn get_before(&self) -> [f64; 3] {
        speed(&self.value["before"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalExitEvent.getAfter
    pub fn get_after(&self) -> [f64; 3] {
        speed(&self.value["after"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalExitEvent.setAfter
    pub fn set_after(&mut self, after: [f64; 3]) {
        self.value["after"] = json!(after);
        crate::event::carry("after", self.value["after"].clone());
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

fn speed(value: &Value) -> [f64; 3] {
    [0, 1, 2].map(|at| value[at].as_f64().unwrap_or(0.0))
}

impl Event for EntityPortalExitEvent {
    const HOOK: &'static str = "ee_portal_exit";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityPortalReadyEvent
#[derive(Clone, Debug)]
pub struct EntityPortalReadyEvent {
    value: Value,
    cancelled: bool,
}

impl EntityPortalReadyEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        known_entity(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPortalReadyEvent.getPortalType
    pub fn get_portal_type(&self) -> &str {
        self.value["portal"].as_str().unwrap_or("NETHER")
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPortalReadyEvent.getTargetWorld
    pub fn get_target_world(&self) -> Option<World> {
        self.value["target"].as_str().map(World::of)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPortalReadyEvent.setTargetWorld
    pub fn set_target_world(&mut self, world: Option<&World>) {
        self.value["target"] = world.map_or(Value::Null, |world| json!(world.dimension()));
        crate::event::carry("target", self.value["target"].clone());
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPortalReadyEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPortalReadyEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityPortalReadyEvent {
    const HOOK: &'static str = "ee_portal";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityPortalEnterEvent
#[derive(Clone, Debug)]
pub struct EntityPortalEnterEvent {
    value: Value,
    cancelled: bool,
}

impl EntityPortalEnterEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEnterEvent.getLocation
    pub fn get_location(&self) -> Option<Location> {
        self.get_block().map(|block| block.get_location())
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEnterEvent.getPortalType
    pub fn get_portal_type(&self) -> &str {
        self.value["portal"].as_str().unwrap_or("NETHER")
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEnterEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEnterEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityPortalEnterEvent {
    const HOOK: &'static str = "ee_portal_enter";
    fn more_hooks() -> &'static [&'static str] {
        &[]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["pos"].is_array()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/PlayerLeashEntityEvent
#[derive(Clone, Debug)]
pub struct PlayerLeashEntityEvent {
    value: Value,
    cancelled: bool,
}

impl PlayerLeashEntityEvent {
    /// bukkit: org/bukkit/event/entity/PlayerLeashEntityEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        self.value["uuid"].as_str().and_then(player_by_uuid)
    }
    /// bukkit: org/bukkit/event/entity/PlayerLeashEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/PlayerLeashEntityEvent.getLeashHolder
    pub fn get_leash_holder(&self) -> Option<Player> {
        self.get_player()
    }
    /// bukkit: org/bukkit/event/entity/PlayerLeashEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/PlayerLeashEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PlayerLeashEntityEvent {
    const HOOK: &'static str = "ee_leash";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityUnleashEvent
#[derive(Clone, Debug)]
pub struct EntityUnleashEvent {
    value: Value,
    drop: bool,
    cancelled: bool,
}

impl EntityUnleashEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/EntityUnleashEvent.getReason
    pub fn get_reason(&self) -> &str {
        self.value["reason"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: org/bukkit/event/entity/EntityUnleashEvent.isDropLeash
    pub fn is_drop_leash(&self) -> bool {
        self.drop
    }
    /// bukkit: org/bukkit/event/entity/EntityUnleashEvent.setDropLeash
    pub fn set_drop_leash(&mut self, drop: bool) {
        self.drop = drop;
    }
    /// bukkit: org/bukkit/event/entity/EntityUnleashEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityUnleashEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityUnleashEvent {
    const HOOK: &'static str = "ee_unleash";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), drop: value["drop"].as_bool().unwrap_or(false), cancelled: crate::event::prior_cancelled() })
    }
    fn finish(&self) {
        query(json!({ "op": "ee_lead_drop", "drop": self.drop, "cancelled": self.cancelled }));
    }
}

// ---- More of a mob's state between ticks ----

/// bukkit: org/bukkit/event/entity/EntitySpellCastEvent
#[derive(Clone, Debug)]
pub struct EntitySpellCastEvent {
    value: Value,
    cancelled: bool,
}

impl EntitySpellCastEvent {
    /// bukkit: org/bukkit/event/entity/EntitySpellCastEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Spellcaster> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/event/entity/EntitySpellCastEvent.getSpell
    pub fn get_spell(&self) -> &'static str {
        match self.value["spell"].as_u64() {
            Some(1) => "SUMMON_VEX",
            Some(2) => "FANGS",
            Some(3) => "WOLOLO",
            _ => "NONE",
        }
    }
    /// bukkit: org/bukkit/event/entity/EntitySpellCastEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntitySpellCastEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntitySpellCastEvent {
    const HOOK: &'static str = "ee_spell";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/EntityJumpEvent
#[derive(Clone, Debug)]
pub struct EntityJumpEvent {
    value: Value,
    cancelled: bool,
}

impl EntityJumpEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EntityJumpEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityJumpEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityJumpEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityJumpEvent {
    const HOOK: &'static str = "ee_jump";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityLandEvent
#[derive(Clone, Debug)]
pub struct EntityLandEvent {
    value: Value,
    fall: f64,
    cancelled: bool,
}

impl EntityLandEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityLandEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLandEvent.getFallDistance
    ///
    /// How far it fell.
    pub fn get_fall_distance(&self) -> f64 {
        self.fall
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLandEvent.setFallDistance
    pub fn set_fall_distance(&mut self, fall: f64) {
        self.fall = fall.max(0.0);
        crate::event::carry("fall", json!(self.fall));
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLandEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value["entity"]), &self.value["block"]["pos"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLandEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLandEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityLandEvent {
    const HOOK: &'static str = "ee_land";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), fall: value["fall"].as_f64().unwrap_or(0.0), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityCollideWithEntityEvent | mobs, a boat on a mob and minecarts on each other: a minecart on a mob and `SulfurCube`'s own case aren't heard
#[derive(Clone, Debug)]
pub struct EntityCollideWithEntityEvent {
    value: Value,
    cancelled: bool,
}

impl EntityCollideWithEntityEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityCollideWithEntityEvent.getEntities
    pub fn get_entities(&self) -> Vec<Actor> {
        self.value["entities"].as_array().into_iter().flatten().filter_map(known_actor).collect()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityCollideWithEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityCollideWithEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityCollideWithEntityEvent {
    const HOOK: &'static str = "ee_collide";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_tick", "ee_collide_boat"]
    }
    fn read(value: &Value) -> Option<Self> {
        value["entities"].is_array().then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityDamageItemEvent
#[derive(Clone, Debug)]
pub struct EntityDamageItemEvent {
    value: Value,
    damage: i32,
    cancelled: bool,
}

impl EntityDamageItemEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDamageItemEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["item"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDamageItemEvent.getDamage
    pub fn get_damage(&self) -> i32 {
        self.damage
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDamageItemEvent.setDamage
    pub fn set_damage(&mut self, damage: i32) {
        self.damage = damage;
        crate::event::carry("damage", json!(damage));
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDamageItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityDamageItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityDamageItemEvent {
    const HOOK: &'static str = "ee_item_wear";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["item"].is_object().then(|| Self { value: value.clone(), damage: value["damage"].as_i64().unwrap_or(0) as i32, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/ItemTransportingEntityValidateTargetEvent
#[derive(Clone, Debug)]
pub struct ItemTransportingEntityValidateTargetEvent {
    value: Value,
    allowed: bool,
}

impl ItemTransportingEntityValidateTargetEvent {
    /// bukkit: io/papermc/paper/event/entity/ItemTransportingEntityValidateTargetEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/ItemTransportingEntityValidateTargetEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value["entity"]), &self.value["block"]["pos"])
    }
    /// bukkit: io/papermc/paper/event/entity/ItemTransportingEntityValidateTargetEvent.setAllowed
    pub fn set_allowed(&mut self, allowed: bool) {
        self.allowed = allowed;
        crate::event::carry("allowed", json!(allowed));
    }
    /// bukkit: io/papermc/paper/event/entity/ItemTransportingEntityValidateTargetEvent.isAllowed
    pub fn is_allowed(&self) -> bool {
        self.allowed
    }
}

impl Event for ItemTransportingEntityValidateTargetEvent {
    const HOOK: &'static str = "ee_transport_target";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), allowed: value["allowed"].as_bool().unwrap_or(true) })
    }
}

/// bukkit: io/papermc/paper/event/entity/ItemTransportingEntitySortEvent
#[derive(Clone, Debug)]
pub struct ItemTransportingEntitySortEvent {
    value: Value,
    result: String,
}

impl ItemTransportingEntitySortEvent {
    /// bukkit: io/papermc/paper/event/entity/ItemTransportingEntitySortEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/ItemTransportingEntitySortEvent.getItemStack
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit~: io/papermc/paper/event/entity/ItemTransportingEntitySortEvent.getContainerInventory | a copy of what's in it: this moment has no game to change it through
    pub fn get_container_inventory(&self) -> crate::Inventory {
        let rows = self.value["items"].as_array().cloned().unwrap_or_default();
        let mut inventory = crate::Inventory::create(rows.len(), "");
        for (slot, row) in rows.iter().enumerate() {
            inventory.set_item(slot, ItemStack::read(row));
        }
        inventory
    }
    /// bukkit: io/papermc/paper/event/entity/ItemTransportingEntitySortEvent.setResult
    pub fn set_result(&mut self, result: &str) {
        self.result = result.to_owned();
        crate::event::carry("result", json!(result));
    }
    /// bukkit: io/papermc/paper/event/entity/ItemTransportingEntitySortEvent.getResult
    pub fn get_result(&self) -> &str {
        &self.result
    }
}

impl Event for ItemTransportingEntitySortEvent {
    const HOOK: &'static str = "ee_transport_sort";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), result: value["result"].as_str().unwrap_or("DEFAULT").to_owned() })
    }
}

/// The potion a witch means, as a stack: she holds only its name.
fn witch_potion(row: &Value) -> Option<ItemStack> {
    ItemStack::read(&json!({ "item": "minecraft:potion", "count": 1, "components": { "minecraft:potion_contents": { "potion": row["held"].as_str()? } } }))
}

/// A potion's id from a stack, wherever its component keeps it.
fn potion_id(stack: &ItemStack) -> Option<String> {
    let text = stack.to_json()["components"]["minecraft:potion_contents"].to_string();
    let start = text.find("minecraft:")?;
    let rest = &text[start..];
    let end = rest.find(|letter: char| !(letter.is_alphanumeric() || letter == ':' || letter == '_')).unwrap_or(rest.len());
    Some(rest[..end].to_owned())
}

/// bukkit: com/destroystokyo/paper/event/entity/WitchReadyPotionEvent
#[derive(Clone, Debug)]
pub struct WitchReadyPotionEvent {
    value: Value,
    cancelled: bool,
    potion: Option<ItemStack>,
}

impl WitchReadyPotionEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/WitchReadyPotionEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchReadyPotionEvent.getPotion
    pub fn get_potion(&self) -> Option<&ItemStack> {
        self.potion.as_ref()
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchReadyPotionEvent.setPotion
    pub fn set_potion(&mut self, potion: Option<ItemStack>) {
        crate::event::carry("potion", json!(potion.as_ref().and_then(potion_id)));
        self.potion = potion;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchReadyPotionEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchReadyPotionEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WitchReadyPotionEvent {
    const HOOK: &'static str = "ee_witch_ready";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), potion: witch_potion(&json!({ "held": value["potion"] })) })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/WitchConsumePotionEvent
#[derive(Clone, Debug)]
pub struct WitchConsumePotionEvent {
    value: Value,
    cancelled: bool,
    potion: Option<ItemStack>,
}

impl WitchConsumePotionEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/WitchConsumePotionEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchConsumePotionEvent.getPotion
    pub fn get_potion(&self) -> Option<&ItemStack> {
        self.potion.as_ref()
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchConsumePotionEvent.setPotion
    pub fn set_potion(&mut self, potion: Option<ItemStack>) {
        crate::event::carry("potion", json!(potion.as_ref().and_then(potion_id)));
        self.potion = potion;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchConsumePotionEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchConsumePotionEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WitchConsumePotionEvent {
    const HOOK: &'static str = "ee_witch_drink";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), potion: witch_potion(&json!({ "held": value["potion"] })) })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

impl Actor {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    pub fn get_type(&self) -> String {
        match self {
            Self::Player(_) => "minecraft:player".to_owned(),
            Self::Entity(entity) => entity.get_type(),
        }
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/ProjectileCollideEvent
#[derive(Clone, Debug)]
pub struct ProjectileCollideEvent {
    value: Value,
    cancelled: bool,
}

impl ProjectileCollideEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/ProjectileCollideEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ProjectileCollideEvent.getCollidedWith
    pub fn get_collided_with(&self) -> Option<Actor> {
        Actor::read(&self.value["hit"], &world_of(&self.value))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ProjectileCollideEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ProjectileCollideEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ProjectileCollideEvent {
    const HOOK: &'static str = "ee_hit";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (!value["hit"].is_null()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntitySpawnEvent, org/bukkit/event/entity/CreatureSpawnEvent
#[derive(Clone, Debug)]
pub struct EntitySpawnEvent {
    value: Value,
    cancelled: bool,
}

/// The reason of the spawn going on: what its site said, or what the game last said it was doing.
fn spawn_reason() -> String {
    let told = query(json!({ "op": "ee_hit_why" }));
    match told.as_str() {
        Some(reason) if !reason.is_empty() => reason.to_owned(),
        _ => (*SPAWN_REASON.lock().unwrap_or_else(|e| e.into_inner())).to_owned(),
    }
}

impl EntitySpawnEvent {
    /// bukkit: org/bukkit/event/entity/CreatureSpawnEvent.getSpawnReason
    pub fn get_spawn_reason(&self) -> String {
        spawn_reason()
    }
    /// bukkit: org/bukkit/event/entity/CreatureSpawnEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &World::of(self.value["level"].as_str().unwrap_or("minecraft:overworld"))))
    }
    pub fn get_entity_id(&self) -> u64 {
        self.value["entity"].as_u64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    ///
    /// The game's id for its kind.
    pub fn get_entity_type(&self) -> &str {
        self.value["kind"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/entity/EntitySpawnEvent.getLocation
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/entity/EntitySpawnEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntitySpawnEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// bukkit: org/bukkit/event/entity/CreatureSpawnEvent
#[derive(Clone, Debug)]
pub struct CreatureSpawnEvent {
    spawn: EntitySpawnEvent,
}

impl CreatureSpawnEvent {
    /// bukkit: org/bukkit/event/entity/CreatureSpawnEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.spawn.get_entity()
    }
    pub fn get_location(&self) -> Option<Location> {
        self.spawn.get_location()
    }
    /// bukkit: org/bukkit/event/entity/CreatureSpawnEvent.getSpawnReason
    pub fn get_spawn_reason(&self) -> String {
        spawn_reason()
    }
    /// bukkit: org/bukkit/event/Cancellable.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.spawn.set_cancelled(cancelled);
    }
    /// bukkit: org/bukkit/event/Cancellable.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.spawn.is_cancelled()
    }
}

impl Event for CreatureSpawnEvent {
    const HOOK: &'static str = "entity_allow_load";
    fn read(value: &Value) -> Option<Self> {
        (value["mob"] == true && value["from_disk"].as_bool() != Some(true)).then(|| EntitySpawnEvent::read(value)).flatten().map(|spawn| Self { spawn })
    }
    fn answer(&self) -> Answer {
        self.spawn.answer()
    }
}

impl Event for EntitySpawnEvent {
    const HOOK: &'static str = "entity_allow_load";
    fn read(value: &Value) -> Option<Self> {
        (value["from_disk"].as_bool() != Some(true)).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityTeleportEvent
#[derive(Clone, Debug)]
pub struct EntityTeleportEvent {
    value: Value,
    cancelled: bool,
}

impl EntityTeleportEvent {
    /// bukkit: org/bukkit/event/entity/EntityTeleportEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityTeleportEvent.getFrom
    pub fn get_from(&self) -> Option<Location> {
        location_at(&world_of(&self.value["entity"]), &self.value["from"])
    }
    /// bukkit: org/bukkit/event/entity/EntityTeleportEvent.setFrom
    pub fn set_from(&mut self, from: &Location) {
        self.value["from"] = json!([from.x, from.y, from.z]);
    }
    /// bukkit: org/bukkit/event/entity/EntityTeleportEvent.getTo
    pub fn get_to(&self) -> Option<Location> {
        location_at(&world_of(&self.value["entity"]), &self.value["to"])
    }
    /// bukkit: org/bukkit/event/entity/EntityTeleportEvent.setTo
    ///
    /// `None` keeps it where it is.
    pub fn set_to(&mut self, to: Option<&Location>) {
        self.value["to"] = to.map_or(Value::Null, |to| json!([to.x, to.y, to.z]));
        crate::event::carry("to", self.value["to"].clone());
    }
    /// bukkit: org/bukkit/event/entity/EntityTeleportEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityTeleportEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityTeleportEvent {
    const HOOK: &'static str = "ee_teleport";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/CreeperIgniteEvent
#[derive(Clone, Debug)]
pub struct CreeperIgniteEvent {
    value: Value,
    cancelled: bool,
}

impl CreeperIgniteEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/CreeperIgniteEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Creeper> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/CreeperIgniteEvent.isIgnited
    pub fn is_ignited(&self) -> bool {
        self.value["ignited"] == true
    }
    /// bukkit: com/destroystokyo/paper/event/entity/CreeperIgniteEvent.setIgnited
    pub fn set_ignited(&mut self, ignited: bool) {
        self.value["ignited"] = json!(ignited);
        crate::event::carry("ignited", json!(ignited));
    }
    /// bukkit: com/destroystokyo/paper/event/entity/CreeperIgniteEvent.getFuseTime
    pub fn get_fuse_time(&self) -> i32 {
        self.value["fuse"].as_i64().unwrap_or(30) as i32
    }
    /// bukkit: com/destroystokyo/paper/event/entity/CreeperIgniteEvent.setFuseTime
    pub fn set_fuse_time(&mut self, ticks: i32) {
        self.value["fuse"] = json!(ticks);
        crate::event::carry("fuse", json!(ticks));
    }
    /// bukkit: com/destroystokyo/paper/event/entity/CreeperIgniteEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/CreeperIgniteEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for CreeperIgniteEvent {
    const HOOK: &'static str = "ee_creeper_ignite";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/StriderTemperatureChangeEvent
#[derive(Clone, Debug)]
pub struct StriderTemperatureChangeEvent {
    value: Value,
    cancelled: bool,
}

impl StriderTemperatureChangeEvent {
    /// bukkit: org/bukkit/event/entity/StriderTemperatureChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Strider> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/event/entity/StriderTemperatureChangeEvent.isShivering
    pub fn is_shivering(&self) -> bool {
        self.value["shivering"] == true
    }
    /// bukkit: org/bukkit/event/entity/StriderTemperatureChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/StriderTemperatureChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for StriderTemperatureChangeEvent {
    const HOOK: &'static str = "ee_strider";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/TurtleStartDiggingEvent
#[derive(Clone, Debug)]
pub struct TurtleStartDiggingEvent {
    value: Value,
    cancelled: bool,
}

impl TurtleStartDiggingEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleStartDiggingEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Turtle> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleStartDiggingEvent.getLocation
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value["entity"]), &self.value["at"])
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleStartDiggingEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleStartDiggingEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for TurtleStartDiggingEvent {
    const HOOK: &'static str = "ee_turtle_dig";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/TurtleGoHomeEvent
#[derive(Clone, Debug)]
pub struct TurtleGoHomeEvent {
    value: Value,
    cancelled: bool,
}

impl TurtleGoHomeEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleGoHomeEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Turtle> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleGoHomeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleGoHomeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for TurtleGoHomeEvent {
    const HOOK: &'static str = "ee_turtle_home";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityKnockbackEvent, io/papermc/paper/event/entity/EntityKnockbackEvent
#[derive(Clone, Debug)]
pub struct EntityKnockbackEvent {
    value: Value,
    cancelled: bool,
}

impl EntityKnockbackEvent {
    /// bukkit: org/bukkit/event/entity/EntityKnockbackEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"]).or_else(|| self.value["uuid"].as_str().and_then(player_by_uuid).map(Actor::Player))
    }
    /// bukkit~: org/bukkit/event/entity/EntityKnockbackEvent.getKnockback, io/papermc/paper/event/entity/EntityKnockbackEvent.getKnockback, org/bukkit/event/entity/EntityKnockbackEvent.getFinalKnockback | what the hit adds, worked out as the game will: not capped by what they're already doing
    pub fn get_knockback(&self) -> crate::util::Vector {
        let (power, x, z) = (self.value["power"].as_f64().unwrap_or(0.0), self.value["x"].as_f64().unwrap_or(0.0), self.value["z"].as_f64().unwrap_or(0.0));
        let length = (x * x + z * z).sqrt().max(1.0e-7);
        crate::util::Vector::new(-x / length * power, power.min(0.4), -z / length * power)
    }
    /// bukkit~: org/bukkit/event/entity/EntityKnockbackEvent.getFinalKnockback | as get_knockback
    pub fn get_final_knockback(&self) -> crate::util::Vector {
        self.get_knockback()
    }
    /// bukkit: org/bukkit/event/entity/EntityKnockbackEvent.getForce
    pub fn get_force(&self) -> f64 {
        self.value["power"].as_f64().unwrap_or(0.0)
    }
    /// bukkit~: org/bukkit/event/entity/EntityKnockbackEvent.getCause, io/papermc/paper/event/entity/EntityKnockbackEvent.getCause | `DAMAGE`, the only one that reaches here
    pub fn get_cause(&self) -> &'static str {
        "DAMAGE"
    }
    /// bukkit: io/papermc/paper/event/entity/EntityKnockbackEvent.setCancelled, org/bukkit/event/entity/EntityKnockbackEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityKnockbackEvent.isCancelled, org/bukkit/event/entity/EntityKnockbackEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityKnockbackEvent {
    const HOOK: &'static str = "ee_knock";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_mob_knock", "ee_tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        value["power"].is_number().then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

impl crate::events::EntityAddToWorldEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EntityAddToWorldEvent.getWorld
    pub fn get_world(&self) -> Option<World> {
        self.get_entity().map(|entity| entity.get_world())
    }
}

impl crate::events::EntityRemoveFromWorldEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EntityRemoveFromWorldEvent.getWorld
    pub fn get_world(&self) -> Option<World> {
        self.get_entity().map(|entity| entity.get_world())
    }
}

/// `EntityEquipmentChangedEvent.EquipmentChange`: a slot's stack before and after.
#[derive(Clone, Debug)]
pub struct EquipmentChange {
    old: Option<ItemStack>,
    new: Option<ItemStack>,
}

impl EquipmentChange {
    /// bukkit~: io/papermc/paper/event/entity/EntityEquipmentChangedEvent$EquipmentChange.oldItem | what this plugin last saw in the slot: nothing the first time
    pub fn old_item(&self) -> Option<&ItemStack> {
        self.old.as_ref()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityEquipmentChangedEvent$EquipmentChange.newItem
    pub fn new_item(&self) -> Option<&ItemStack> {
        self.new.as_ref()
    }
}

/// What each player last wore, by (uuid, slot).
static WORN: Mutex<Option<std::collections::HashMap<(String, String), Option<ItemStack>>>> = Mutex::new(None);

/// bukkit~: io/papermc/paper/event/entity/EntityEquipmentChangedEvent | a mob's changes come together; a player's are one slot an event
#[derive(Clone, Debug)]
pub struct EntityEquipmentChangedEvent {
    value: Value,
    change: EquipmentChange,
}

impl EntityEquipmentChangedEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityEquipmentChangedEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"]).or_else(|| Player::read(&self.value["player"]).map(Actor::Player))
    }
    /// The slot of a player's change, as the game names it.
    pub fn get_slot(&self) -> &str {
        self.value["slot"].as_str().unwrap_or_default()
    }
    pub fn get_equipment_changes(&self) -> Vec<(String, EquipmentChange)> {
        if let Some(rows) = self.value["changes"].as_array() {
            return rows.iter().map(|row| (row["slot"].as_str().unwrap_or_default().to_owned(), EquipmentChange { old: ItemStack::read(&row["old"]), new: ItemStack::read(&row["new"]) })).collect();
        }
        vec![(self.get_slot().to_owned(), self.change.clone())]
    }
}

impl Event for EntityEquipmentChangedEvent {
    const HOOK: &'static str = "equipment_change";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_equipment"]
    }
    fn read(value: &Value) -> Option<Self> {
        if value["changes"].is_array() {
            return Some(Self { value: value.clone(), change: EquipmentChange { old: None, new: None } });
        }
        let new = ItemStack::read(&value["stack"]);
        let key = (value["player"]["uuid"].as_str()?.to_owned(), value["slot"].as_str()?.to_owned());
        let old = WORN.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(key, new.clone()).flatten();
        Some(Self { value: value.clone(), change: EquipmentChange { old, new } })
    }
}

impl crate::events::SkeletonHorseTrapEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/SkeletonHorseTrapEvent.getEligibleHumans
    pub fn get_eligible_humans(&self) -> Vec<Player> {
        let Some(at) = self.get_entity().and_then(|entity| entity.get_location()) else {
            return Vec::new();
        };
        crate::get_online_players().into_iter().filter(|player| player.get_location().get_world() == at.get_world() && player.get_location().distance_squared(&at) <= 100.0).collect()
    }
}

// ---- Damage: the game is mid-hit and can't be asked anything, so what a listener needs comes with it ----

/// The hit being decided: `ee_damage_info` comes just before `allow_damage`, from the engine.
static HIT: Mutex<Value> = Mutex::new(Value::Null);
/// Who last hurt what, by entity id: (the tick, who).
static LAST_HIT: Mutex<Option<std::collections::HashMap<u64, (u64, Value)>>> = Mutex::new(None);

const DAMAGE_HOOKS: &[&str] = &["ee_damage_info", "ee_tick"];

fn note_damage(value: &Value) -> Option<Answer> {
    if value["ee_info"] == true {
        if let Some(entity) = value["entity"].as_u64() {
            LAST_HIT.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(entity, (crate::scheduler::now(), value["causer"].clone()));
        }
        *HIT.lock().unwrap_or_else(|e| e.into_inner()) = value.clone();
        return Some(Answer::Pass);
    }
    (!value["ee_tick"].is_null()).then_some(Answer::Pass)
}

fn hit_of(value: &Value) -> Value {
    let hit = HIT.lock().unwrap_or_else(|e| e.into_inner());
    if hit["entity"] == value["entity"] { hit.clone() } else { Value::Null }
}

/// bukkit: org/bukkit/event/entity/EntityCombustEvent
#[derive(Clone, Debug)]
pub struct EntityCombustEvent {
    value: Value,
    cancelled: bool,
}

impl EntityCombustEvent {
    /// bukkit: org/bukkit/event/entity/EntityCombustEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityCombustEvent.getDuration
    pub fn get_duration(&self) -> f32 {
        self.value["duration"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/event/entity/EntityCombustEvent.setDuration
    pub fn set_duration(&mut self, seconds: f32) {
        self.value["duration"] = json!(seconds);
        crate::event::carry("duration", json!(seconds));
    }
    /// bukkit: org/bukkit/event/entity/EntityCombustEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityCombustEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityCombustEvent {
    const HOOK: &'static str = "ee_combust";
    fn more_hooks() -> &'static [&'static str] {
        IN_MOB_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["duration"].is_number()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityEnterLoveModeEvent
#[derive(Clone, Debug)]
pub struct EntityEnterLoveModeEvent {
    value: Value,
    cancelled: bool,
}

impl EntityEnterLoveModeEvent {
    /// bukkit: org/bukkit/event/entity/EntityEnterLoveModeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit~: org/bukkit/event/entity/EntityEnterLoveModeEvent.getHumanEntity | whoever fed it by the game's own `Animal.mobInteract`: a llama's, a horse's, a nautilus's or a panda's feed doesn't name them
    pub fn get_human_entity(&self) -> Option<Player> {
        Player::read(&self.value["human"])
    }
    /// bukkit: org/bukkit/event/entity/EntityEnterLoveModeEvent.getTicksInLove
    pub fn get_ticks_in_love(&self) -> i32 {
        self.value["ticks"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/entity/EntityEnterLoveModeEvent.setTicksInLove
    pub fn set_ticks_in_love(&mut self, ticks: i32) {
        self.value["ticks"] = json!(ticks);
        crate::event::carry("ticks", json!(ticks));
    }
    /// bukkit: org/bukkit/event/entity/EntityEnterLoveModeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityEnterLoveModeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityEnterLoveModeEvent {
    const HOOK: &'static str = "ee_love";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent
#[derive(Clone, Debug)]
pub struct VillagerCareerChangeEvent {
    value: Value,
    cancelled: bool,
}

impl VillagerCareerChangeEvent {
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Villager> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent.getProfessionNumber
    /// The profession it's taking, by the game's number.
    pub fn get_profession_number(&self) -> i64 {
        self.value["profession"].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent.getProfession
    pub fn get_profession(&self) -> &'static str {
        PROFESSIONS.get(self.get_profession_number() as usize).copied().unwrap_or("NONE")
    }
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent.setProfession
    pub fn set_profession(&mut self, profession: &str) {
        if let Some(number) = PROFESSIONS.iter().position(|name| *name == profession) {
            self.value["profession"] = json!(number);
            crate::event::carry("profession", json!(number));
        }
    }
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent.getReason
    pub fn get_reason(&self) -> &'static str {
        if self.get_profession_number() == 0 { "LOSING_JOB" } else { "EMPLOYED" }
    }
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/VillagerCareerChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for VillagerCareerChangeEvent {
    const HOOK: &'static str = "ee_career";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/CreeperPowerEvent
#[derive(Clone, Debug)]
pub struct CreeperPowerEvent {
    value: Value,
    cancelled: bool,
}

impl CreeperPowerEvent {
    /// bukkit: org/bukkit/event/entity/CreeperPowerEvent.getLightning
    pub fn get_lightning(&self) -> Option<crate::fam_entity::LightningStrike> {
        bolt_now(&world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/CreeperPowerEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Creeper> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/event/entity/CreeperPowerEvent.getCause
    pub fn get_cause(&self) -> &str {
        self.value["cause"].as_str().unwrap_or("LIGHTNING")
    }
    /// bukkit: org/bukkit/event/entity/CreeperPowerEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/CreeperPowerEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for CreeperPowerEvent {
    const HOOK: &'static str = "ee_creeper_power";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/PigZombieAngerEvent
#[derive(Clone, Debug)]
pub struct PigZombieAngerEvent {
    value: Value,
    cancelled: bool,
}

impl PigZombieAngerEvent {
    /// bukkit: org/bukkit/event/entity/PigZombieAngerEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/PigZombieAngerEvent.getTarget
    pub fn get_target(&self) -> Option<Actor> {
        known_actor(&self.value["target"])
    }
    /// bukkit: org/bukkit/event/entity/PigZombieAngerEvent.getNewAnger
    pub fn get_new_anger(&self) -> i32 {
        self.value["anger"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/entity/PigZombieAngerEvent.setNewAnger
    pub fn set_new_anger(&mut self, anger: i32) {
        self.value["anger"] = json!(anger);
        crate::event::carry("anger", json!(anger));
    }
    /// bukkit: org/bukkit/event/entity/PigZombieAngerEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/PigZombieAngerEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PigZombieAngerEvent {
    const HOOK: &'static str = "ee_pig_anger";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EnderDragonChangePhaseEvent
#[derive(Clone, Debug)]
pub struct EnderDragonChangePhaseEvent {
    value: Value,
    cancelled: bool,
}

impl EnderDragonChangePhaseEvent {
    /// bukkit: org/bukkit/event/entity/EnderDragonChangePhaseEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EnderDragonChangePhaseEvent.getCurrentPhase
    pub fn get_current_phase(&self) -> &'static str {
        dragon_phase(self.value["old"].as_i64().unwrap_or(10))
    }
    /// bukkit: org/bukkit/event/entity/EnderDragonChangePhaseEvent.getNewPhase
    pub fn get_new_phase(&self) -> &'static str {
        dragon_phase(self.value["new"].as_i64().unwrap_or(10))
    }
    /// bukkit: org/bukkit/event/entity/EnderDragonChangePhaseEvent.setNewPhase
    pub fn set_new_phase(&mut self, phase: &str) {
        if let Some(number) = DRAGON_PHASES.iter().position(|name| *name == phase) {
            self.value["new"] = json!(number);
            crate::event::carry("new", json!(number));
        }
    }
    /// bukkit: org/bukkit/event/entity/EnderDragonChangePhaseEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EnderDragonChangePhaseEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EnderDragonChangePhaseEvent {
    const HOOK: &'static str = "ee_dragon_phase";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityAirChangeEvent
#[derive(Clone, Debug)]
pub struct EntityAirChangeEvent {
    value: Value,
    cancelled: bool,
}

impl EntityAirChangeEvent {
    /// bukkit: org/bukkit/event/entity/EntityAirChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityAirChangeEvent.getAmount
    pub fn get_amount(&self) -> i32 {
        self.value["amount"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/entity/EntityAirChangeEvent.setAmount
    pub fn set_amount(&mut self, amount: i32) {
        self.value["amount"] = json!(amount);
        crate::event::carry("amount", json!(amount));
    }
    /// bukkit: org/bukkit/event/entity/EntityAirChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityAirChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityAirChangeEvent {
    const HOOK: &'static str = "ee_air";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/VillagerReplenishTradeEvent
#[derive(Clone, Debug)]
pub struct VillagerReplenishTradeEvent {
    value: Value,
    cancelled: bool,
}

impl VillagerReplenishTradeEvent {
    /// bukkit: org/bukkit/event/entity/VillagerReplenishTradeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    fn merchant(&self) -> Option<crate::fam_items::Merchant> {
        known_entity(&self.value["entity"]).map(crate::fam_items::Merchant)
    }
    /// bukkit: org/bukkit/event/entity/VillagerReplenishTradeEvent.getRecipe
    pub fn get_recipe(&self) -> Option<crate::core2::MerchantRecipe> {
        self.merchant()?.get_recipe(self.value["index"].as_u64()? as usize)
    }
    /// bukkit: org/bukkit/event/entity/VillagerReplenishTradeEvent.setRecipe
    pub fn set_recipe(&self, recipe: crate::core2::MerchantRecipe) -> bool {
        self.merchant().zip(self.value["index"].as_u64()).is_some_and(|(merchant, index)| merchant.set_recipe(index as usize, recipe))
    }
    /// bukkit: org/bukkit/event/entity/VillagerReplenishTradeEvent.getBonus
    pub fn get_bonus(&self) -> i32 {
        self.get_recipe().map_or(0, |recipe| recipe.get_uses() as i32)
    }
    /// bukkit: org/bukkit/event/entity/VillagerReplenishTradeEvent.setBonus
    ///
    /// Bukkit's own does nothing since 1.14, and says so.
    pub fn set_bonus(&self, _bonus: i32) {}
    /// bukkit: org/bukkit/event/entity/VillagerReplenishTradeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/VillagerReplenishTradeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for VillagerReplenishTradeEvent {
    const HOOK: &'static str = "ee_replenish";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/VillagerAcquireTradeEvent
#[derive(Clone, Debug)]
pub struct VillagerAcquireTradeEvent {
    value: Value,
    cancelled: bool,
}

impl VillagerAcquireTradeEvent {
    /// bukkit: org/bukkit/event/entity/VillagerAcquireTradeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    fn merchant(&self) -> Option<crate::fam_items::Merchant> {
        known_entity(&self.value["entity"]).map(crate::fam_items::Merchant)
    }
    /// bukkit: org/bukkit/event/entity/VillagerAcquireTradeEvent.getRecipe
    pub fn get_recipe(&self) -> Option<crate::core2::MerchantRecipe> {
        self.merchant()?.get_recipe(self.value["index"].as_u64()? as usize)
    }
    /// bukkit: org/bukkit/event/entity/VillagerAcquireTradeEvent.setRecipe
    pub fn set_recipe(&self, recipe: crate::core2::MerchantRecipe) -> bool {
        self.merchant().zip(self.value["index"].as_u64()).is_some_and(|(merchant, index)| merchant.set_recipe(index as usize, recipe))
    }
    /// bukkit: org/bukkit/event/entity/VillagerAcquireTradeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/VillagerAcquireTradeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for VillagerAcquireTradeEvent {
    const HOOK: &'static str = "ee_acquire";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/BatToggleSleepEvent
#[derive(Clone, Debug)]
pub struct BatToggleSleepEvent {
    value: Value,
    cancelled: bool,
}

impl BatToggleSleepEvent {
    /// bukkit: org/bukkit/event/entity/BatToggleSleepEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Bat> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/event/entity/BatToggleSleepEvent.isAwake
    pub fn is_awake(&self) -> bool {
        self.value["awake"] == true
    }
    /// bukkit: org/bukkit/event/entity/BatToggleSleepEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/BatToggleSleepEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for BatToggleSleepEvent {
    const HOOK: &'static str = "ee_bat";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/SheepRegrowWoolEvent
#[derive(Clone, Debug)]
pub struct SheepRegrowWoolEvent {
    value: Value,
    cancelled: bool,
}

impl SheepRegrowWoolEvent {
    /// bukkit: org/bukkit/event/entity/SheepRegrowWoolEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Sheep> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/event/entity/SheepRegrowWoolEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/SheepRegrowWoolEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for SheepRegrowWoolEvent {
    const HOOK: &'static str = "ee_sheep_regrow";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityToggleSitEvent
#[derive(Clone, Debug)]
pub struct EntityToggleSitEvent {
    value: Value,
    cancelled: bool,
}

impl EntityToggleSitEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityToggleSitEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityToggleSitEvent.getSittingState
    pub fn get_sitting_state(&self) -> bool {
        self.value["sitting"] == true
    }
    /// bukkit: io/papermc/paper/event/entity/EntityToggleSitEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityToggleSitEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityToggleSitEvent {
    const HOOK: &'static str = "ee_sit";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/PufferFishStateChangeEvent
#[derive(Clone, Debug)]
pub struct PufferFishStateChangeEvent {
    value: Value,
    cancelled: bool,
}

impl PufferFishStateChangeEvent {
    /// bukkit: io/papermc/paper/event/entity/PufferFishStateChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::PufferFish> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: io/papermc/paper/event/entity/PufferFishStateChangeEvent.getNewPuffState
    pub fn get_new_puff_state(&self) -> i32 {
        self.value["state"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/entity/PufferFishStateChangeEvent.isInflating
    pub fn is_inflating(&self) -> bool {
        self.value["state"].as_i64() > self.value["old"].as_i64()
    }
    /// bukkit: io/papermc/paper/event/entity/PufferFishStateChangeEvent.isDeflating
    pub fn is_deflating(&self) -> bool {
        self.value["state"].as_i64() < self.value["old"].as_i64()
    }
    /// bukkit: io/papermc/paper/event/entity/PufferFishStateChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/PufferFishStateChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PufferFishStateChangeEvent {
    const HOOK: &'static str = "ee_puff";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// What hears from inside a mob's tick also hears the tick, which sets up who's who.
/// `mobs::render::PROFESSIONS`, by Bukkit's names.
const PROFESSIONS: [&str; 15] = ["NONE", "ARMORER", "BUTCHER", "CARTOGRAPHER", "CLERIC", "FARMER", "FISHERMAN", "FLETCHER", "LEATHERWORKER", "LIBRARIAN", "MASON", "NITWIT", "SHEPHERD", "TOOLSMITH", "WEAPONSMITH"];
const IN_TICK: &[&str] = &["ee_tick"];
/// And to know who did what in that tick, the mobs' events after it.
const IN_MOB_TICK: &[&str] = &["ee_tick", "ee_mob"];

/// A row of what the engine knows: `Entity::of` would ask the game, which can't answer in a hit.
fn known_entity(row: &Value) -> Option<Entity> {
    row["id"].as_u64()?;
    Entity::read(&json!({ "id": row["id"], "dimension": row["dimension"], "uuid": row["uuid"] }))
}

fn known_actor(row: &Value) -> Option<Actor> {
    if row["player"] == true {
        return Player::read(row).map(Actor::Player);
    }
    known_entity(row).map(Actor::Entity)
}

fn damage_source_of(value: &Value, hit: &Value) -> crate::core2::DamageSource {
    let mut source = crate::core2::DamageSource::builder(value["damage_type"].as_str().unwrap_or("minecraft:generic"));
    if let Some(entity) = known_entity(&hit["causer"]) {
        source = source.with_causing_entity(&entity);
    }
    if let Some(entity) = known_entity(&hit["source"]) {
        source = source.with_direct_entity(&entity);
    }
    if let Some(location) = location_at(&World::of(hit["victim"]["dimension"].as_str().unwrap_or("minecraft:overworld")), &hit["victim"]["at"]) {
        source = source.with_damage_location(&location);
    }
    crate::fam_core::stored_damage_context(source, &hit["victim"])
}

/// `EntityDamageEvent.DamageModifier`, in Bukkit's order.
const HIT_MODIFIERS: [&str; 9] = ["BASE", "INVULNERABILITY_REDUCTION", "FREEZING", "HARD_HAT", "BLOCKING", "ARMOR", "RESISTANCE", "MAGIC", "ABSORPTION"];

/// What the game's own steps make of a hit of `amount`, or of the hit as it came.
fn hit_steps(amount: Option<f64>) -> Value {
    query(json!({ "op": "ee_hit_steps", "amount": amount }))
}

/// A hit's damage as its listeners see it: the amount, and what they set by hand.
#[derive(Clone, Debug)]
struct HitDamage {
    amount: f64,
    moved: bool,
    set: serde_json::Map<String, Value>,
}

impl HitDamage {
    fn read(value: &Value) -> Self {
        Self { amount: value["amount"].as_f64().unwrap_or(0.0), moved: false, set: value["hit_modifiers"].as_object().cloned().unwrap_or_default() }
    }
    fn of(&self, modifier: &str) -> f64 {
        if modifier == "BASE" {
            return self.amount;
        }
        self.set.get(modifier).and_then(Value::as_f64).unwrap_or_else(|| hit_steps(Some(self.amount))[modifier].as_f64().unwrap_or(0.0))
    }
    /// `setDamage`: what was set by hand keeps its distance from what the game would do.
    fn set_amount(&mut self, damage: f64) {
        let (old, new) = (hit_steps(Some(self.amount)), hit_steps(Some(damage)));
        for (modifier, value) in self.set.iter_mut() {
            let difference = old[modifier].as_f64().unwrap_or(0.0) - new[modifier].as_f64().unwrap_or(0.0);
            let held = value.as_f64().unwrap_or(0.0);
            *value = json!(if held > 0.0 { (held - difference).max(0.0) } else { (held - difference).min(0.0) });
        }
        self.amount = damage;
        self.moved = true;
        self.carry();
    }
    fn set_modifier(&mut self, modifier: &str, damage: f64) {
        if modifier == "BASE" {
            self.amount = damage;
            self.moved = true;
        } else if HIT_MODIFIERS.contains(&modifier) {
            self.set.insert(modifier.to_owned(), json!(damage));
        }
        self.carry();
    }
    fn carry(&self) {
        crate::event::carry("hit_modifiers", Value::Object(self.set.clone()));
    }
    fn final_damage(&self) -> f64 {
        let steps = hit_steps(Some(self.amount));
        HIT_MODIFIERS.iter().map(|modifier| match *modifier {
            "BASE" => self.amount,
            modifier => self.set.get(modifier).and_then(Value::as_f64).unwrap_or_else(|| steps[modifier].as_f64().unwrap_or(0.0)),
        }).sum()
    }
    /// A hit the game has no steps for (the dragon's) takes it all as the amount.
    fn answer(&self, cancelled: bool) -> Answer {
        if cancelled {
            return Answer::Fail;
        }
        if !self.set.is_empty() {
            if hit_steps(None).is_null() {
                return Answer::Amount((self.final_damage() * 1000.0) as i32);
            }
            query(json!({ "op": "ee_hit_final", "damage": self.final_damage(), "absorbed": (-self.of("ABSORPTION")).max(0.0) }));
        }
        if self.moved { Answer::Amount((self.amount * 1000.0) as i32) } else { Answer::Pass }
    }
}

/// bukkit: org/bukkit/event/entity/EntityDamageEvent
#[derive(Clone, Debug)]
pub struct EntityDamageEvent {
    value: Value,
    hit: Value,
    cancelled: bool,
    damage: HitDamage,
}

impl EntityDamageEvent {
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        known_entity(&self.hit["victim"]).or_else(|| self.value["entity"].as_u64().map(|id| Entity::of(id, &World::of("minecraft:overworld"))))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity
    pub fn get_victim(&self) -> Option<Actor> {
        known_actor(&self.hit["victim"])
    }
    pub fn get_damage(&self) -> f64 {
        self.damage.amount
    }
    pub fn set_damage(&mut self, damage: f64) {
        self.damage.set_amount(damage);
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.getDamage
    pub fn get_damage_of(&self, modifier: &str) -> f64 {
        self.damage.of(modifier)
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.setDamage
    pub fn set_damage_of(&mut self, modifier: &str, damage: f64) {
        self.damage.set_modifier(modifier, damage);
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.getFinalDamage
    pub fn get_final_damage(&self) -> f64 {
        self.damage.final_damage()
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.getCause
    pub fn get_cause(&self) -> &'static str {
        crate::events::damage_cause(self.value["damage_type"].as_str().unwrap_or_default())
    }
    /// The game's id of the damage type.
    pub fn get_damage_type(&self) -> &str {
        self.value["damage_type"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.getDamageSource
    pub fn get_damage_source(&self) -> crate::core2::DamageSource {
        damage_source_of(&self.value, &self.hit)
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.getOriginalDamage
    pub fn get_original_damage(&self, modifier: &str) -> f64 {
        hit_steps(None)[modifier].as_f64().unwrap_or(if modifier == "BASE" { self.damage.amount } else { 0.0 })
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.isApplicable
    pub fn is_applicable(&self, modifier: &str) -> bool {
        HIT_MODIFIERS.contains(&modifier)
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityDamageEvent {
    const HOOK: &'static str = "allow_damage";
    fn more_hooks() -> &'static [&'static str] {
        DAMAGE_HOOKS
    }
    fn intercept(value: &Value) -> Option<Answer> {
        note_damage(value)
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), hit: hit_of(value), cancelled: crate::event::prior_cancelled(), damage: HitDamage::read(value) })
    }
    fn answer(&self) -> Answer {
        self.damage.answer(self.cancelled)
    }
}

/// Whether the player's blow now landing is a critical hit.
static CRITICAL: Mutex<bool> = Mutex::new(false);

/// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent
#[derive(Clone, Debug)]
pub struct EntityDamageByEntityEvent {
    value: Value,
    hit: Value,
    critical: bool,
    cancelled: bool,
    damage: HitDamage,
}

impl EntityDamageByEntityEvent {
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.hit["victim"])
    }
    /// bukkit~: org/bukkit/event/entity/EntityDamageByEntityEvent.getDamager | the mob whose turn it is reads whole; a shooter elsewhere has its id, kind and place from the start of the tick, and an empty saved tree
    ///
    /// What struck: an arrow for a shot, whoever shot it being in `get_damage_source`.
    pub fn get_damager(&self) -> Option<Actor> {
        known_actor(&self.hit["source"]).or_else(|| known_actor(&self.hit["causer"]))
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.getDamage
    pub fn get_damage(&self) -> f64 {
        self.damage.amount
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.setDamage
    pub fn set_damage(&mut self, damage: f64) {
        self.damage.set_amount(damage);
    }
    pub fn get_damage_of(&self, modifier: &str) -> f64 {
        self.damage.of(modifier)
    }
    pub fn set_damage_of(&mut self, modifier: &str, damage: f64) {
        self.damage.set_modifier(modifier, damage);
    }
    pub fn get_final_damage(&self) -> f64 {
        self.damage.final_damage()
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.getCause
    pub fn get_cause(&self) -> &'static str {
        crate::events::damage_cause(self.value["damage_type"].as_str().unwrap_or_default())
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.getDamageSource
    pub fn get_damage_source(&self) -> crate::core2::DamageSource {
        damage_source_of(&self.value, &self.hit)
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.isCritical
    pub fn is_critical(&self) -> bool {
        self.critical
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityDamageByEntityEvent {
    const HOOK: &'static str = "allow_damage";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_damage_info", "ee_tick", "ee_melee", "ee_mob"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        if let Some(critical) = value["ee_crit"].as_bool() {
            *CRITICAL.lock().unwrap_or_else(|e| e.into_inner()) = critical;
            return Some(Answer::Pass);
        }
        note_damage(value)
    }
    fn read(value: &Value) -> Option<Self> {
        let hit = hit_of(value);
        if hit["causer"].is_null() && hit["source"].is_null() {
            return None;
        }
        let critical = hit["causer"]["player"] == true && std::mem::take(&mut *CRITICAL.lock().unwrap_or_else(|e| e.into_inner()));
        Some(Self { value: value.clone(), hit, critical, cancelled: crate::event::prior_cancelled(), damage: HitDamage::read(value) })
    }
    fn answer(&self) -> Answer {
        self.damage.answer(self.cancelled)
    }
}

fn block_of_damage(damage_type: &str) -> Option<&'static str> {
    Some(match damage_type.trim_start_matches("minecraft:") {
        "cactus" => "minecraft:cactus",
        "sweet_berry_bush" => "minecraft:sweet_berry_bush",
        "hot_floor" => "minecraft:magma_block",
        "lava" => "minecraft:lava",
        "in_fire" => "minecraft:fire",
        "campfire" => "minecraft:campfire",
        "stalagmite" => "minecraft:pointed_dripstone",
        _ => return None,
    })
}

/// bukkit: org/bukkit/event/entity/EntityDamageByBlockEvent
#[derive(Clone, Debug)]
pub struct EntityDamageByBlockEvent {
    value: Value,
    hit: Value,
    cancelled: bool,
    damage: HitDamage,
}

impl EntityDamageByBlockEvent {
    /// bukkit: org/bukkit/event/entity/EntityDamageByBlockEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.hit["victim"])
    }
    /// bukkit~: org/bukkit/event/entity/EntityDamageByBlockEvent.getDamager | the block at the victim's feet as the tick began: the hit doesn't name it
    pub fn get_damager(&self) -> Option<Block> {
        let victim = &self.hit["victim"];
        let world = World::of(victim["dimension"].as_str().unwrap_or("minecraft:overworld"));
        let at = |i: usize| victim["at"][i].as_f64().map(|v| v.floor() as i32);
        Some(world.get_block_at(at(0)?, at(1)?, at(2)?))
    }
    /// bukkit~: org/bukkit/event/entity/EntityDamageByBlockEvent.getDamagerBlockState | the kind the damage type means, at `get_damager`'s place
    pub fn get_damager_block_state(&self) -> Option<crate::BlockState> {
        let name = block_of_damage(self.value["damage_type"].as_str().unwrap_or_default())?;
        self.get_damager().map(|block| crate::BlockState::snapshot(block, name))
    }
    pub fn get_damage(&self) -> f64 {
        self.damage.amount
    }
    pub fn set_damage(&mut self, damage: f64) {
        self.damage.set_amount(damage);
    }
    pub fn get_damage_of(&self, modifier: &str) -> f64 {
        self.damage.of(modifier)
    }
    pub fn set_damage_of(&mut self, modifier: &str, damage: f64) {
        self.damage.set_modifier(modifier, damage);
    }
    pub fn get_final_damage(&self) -> f64 {
        self.damage.final_damage()
    }
    pub fn get_cause(&self) -> &'static str {
        crate::events::damage_cause(self.value["damage_type"].as_str().unwrap_or_default())
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

impl Event for EntityDamageByBlockEvent {
    const HOOK: &'static str = "allow_damage";
    fn more_hooks() -> &'static [&'static str] {
        DAMAGE_HOOKS
    }
    fn intercept(value: &Value) -> Option<Answer> {
        note_damage(value)
    }
    fn read(value: &Value) -> Option<Self> {
        block_of_damage(value["damage_type"].as_str().unwrap_or_default())?;
        Some(Self { value: value.clone(), hit: hit_of(value), cancelled: crate::event::prior_cancelled(), damage: HitDamage::read(value) })
    }
    fn answer(&self) -> Answer {
        self.damage.answer(self.cancelled)
    }
}

// ---- Explosions, as the blocks are decided ----

/// What tells who's blowing up, as it does.
pub(crate) const BLAST_HOOKS: &[&str] = &["ee_explode", "ee_mob", "ee_hit_block", "ee_blast_shot", "ee_blast_tnt", "ee_blast_cart", "ee_blast_crystal", "ee_blast_end"];

/// bukkit: org/bukkit/event/entity/EntityExplodeEvent
#[derive(Clone, Debug)]
pub struct EntityExplodeEvent {
    value: Value,
    blocks: Vec<Block>,
    power: f32,
    cancelled: bool,
    /// What the listener was given: unchanged, nothing goes back to the engine (one listener per plugin sent the
    /// whole list back for every blast, and the last one's undid an earlier one's edits).
    given: (usize, f32, bool),
}

impl EntityExplodeEvent {
    /// bukkit~: org/bukkit/event/entity/EntityExplodeEvent.getEntity | a creeper, wither, TNT, TNT minecart, fireball, wither skull or end crystal: what has left the world by now reads as its id, kind and place
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    ///
    /// The game's id of what blew up: `None` for what isn't an entity, like a bed.
    pub fn get_entity_type(&self) -> Option<&str> {
        self.value["entity_kind"].as_str()
    }
    /// bukkit: org/bukkit/event/entity/EntityExplodeEvent.getLocation
    pub fn get_location(&self) -> Option<Location> {
        location_at(&world_of(&self.value), &self.value["center"])
    }
    /// bukkit: org/bukkit/event/entity/EntityExplodeEvent.blockList
    ///
    /// What it will break: change this and that's what breaks.
    pub fn block_list(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }
    /// bukkit: org/bukkit/event/entity/EntityExplodeEvent.getYield
    pub fn get_yield(&self) -> f32 {
        self.power
    }
    /// bukkit: org/bukkit/event/entity/EntityExplodeEvent.setYield
    pub fn set_yield(&mut self, power: f32) {
        self.power = power;
    }
    /// bukkit: org/bukkit/event/entity/EntityExplodeEvent.getExplosionResult
    pub fn get_explosion_result(&self) -> &str {
        self.value["result"].as_str().unwrap_or("DESTROY")
    }
    /// bukkit: org/bukkit/event/entity/EntityExplodeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityExplodeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityExplodeEvent {
    const HOOK: &'static str = "ee_detonate";
    fn more_hooks() -> &'static [&'static str] {
        BLAST_HOOKS
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value).or_else(|| value["ee_why"].is_boolean().then_some(Answer::Pass))
    }
    fn read(value: &Value) -> Option<Self> {
        if value["ee_blast"] != true {
            return None;
        }
        let world = world_of(value);
        let blocks: Vec<Block> = value["blocks"].as_array().into_iter().flatten().filter_map(|pos| block_at(&world, pos)).collect();
        let (power, cancelled) = (value["yield"].as_f64().unwrap_or(1.0) as f32, crate::event::prior_cancelled());
        Some(Self { given: (blocks.len(), power, cancelled), value: value.clone(), blocks, power, cancelled })
    }
    fn finish(&self) {
        // Listeners only remove blocks (or cancel, or change the yield): the same count, yield and cancel is the list
        // as it came, and the engine keeps what it has.
        if (self.blocks.len(), self.power, self.cancelled) == self.given {
            return;
        }
        let blocks: Vec<[i32; 3]> = self.blocks.iter().map(|block| [block.get_x(), block.get_y(), block.get_z()]).collect();
        // The listeners after this one start from this list, not the first one.
        crate::event::carry("blocks", json!(blocks));
        crate::event::carry("yield", json!(self.power));
        query(json!({ "op": "ee_blast", "blocks": blocks, "cancelled": self.cancelled, "yield": self.power }));
    }
}

/// bukkit~: org/bukkit/event/entity/EntityResurrectEvent | a player with no totem isn't asked: the game's check is in `PlayerMotion`, which sees only the totem
#[derive(Clone, Debug)]
pub struct EntityResurrectEvent {
    value: Value,
    cancelled: bool,
}

impl EntityResurrectEvent {
    /// bukkit: org/bukkit/event/entity/EntityResurrectEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"]).or_else(|| self.value["uuid"].as_str().and_then(player_by_uuid).map(Actor::Player))
    }
    /// bukkit: org/bukkit/event/entity/EntityResurrectEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: org/bukkit/event/entity/EntityResurrectEvent.setCancelled
    ///
    /// A cancel: no totem, and they die.
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityResurrectEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityResurrectEvent {
    const HOOK: &'static str = "ee_totem";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_mob_totem", "ee_tick"]
    }
    fn read(value: &Value) -> Option<Self> {
        if value["entity"].is_object() {
            // No totem: asked as refused, so a listener can bring it back anyway.
            return Some(Self { value: value.clone(), cancelled: value["hand"].is_null() || crate::event::prior_cancelled() });
        }
        (value["used"] == true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- A player's hunger, between ticks ----

/// What each player last ate: (the tick, the stack).
static ATE: Mutex<Option<std::collections::HashMap<String, (u64, Value)>>> = Mutex::new(None);

/// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent
#[derive(Clone, Debug)]
pub struct FoodLevelChangeEvent {
    value: Value,
    cancelled: bool,
}

impl FoodLevelChangeEvent {
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.getItem
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["item"])
    }
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.getFoodLevel
    pub fn get_food_level(&self) -> i32 {
        self.value["level"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.setFoodLevel
    pub fn set_food_level(&mut self, level: i32) {
        self.value["level"] = json!(level);
        crate::event::carry("level", json!(level));
    }
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for FoodLevelChangeEvent {
    const HOOK: &'static str = "ee_food";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityExhaustionEvent
#[derive(Clone, Debug)]
pub struct EntityExhaustionEvent {
    value: Value,
    cancelled: bool,
}

impl EntityExhaustionEvent {
    /// bukkit: org/bukkit/event/entity/EntityExhaustionEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityExhaustionEvent.getExhaustionReason
    pub fn get_exhaustion_reason(&self) -> &str {
        self.value["reason"].as_str().unwrap_or("UNKNOWN")
    }
    /// bukkit: org/bukkit/event/entity/EntityExhaustionEvent.getExhaustion
    pub fn get_exhaustion(&self) -> f32 {
        self.value["amount"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/event/entity/EntityExhaustionEvent.setExhaustion
    pub fn set_exhaustion(&mut self, amount: f32) {
        self.value["amount"] = json!(amount);
        crate::event::carry("amount", json!(amount));
    }
    /// bukkit: org/bukkit/event/entity/EntityExhaustionEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityExhaustionEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityExhaustionEvent {
    const HOOK: &'static str = "ee_exhaustion";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- Fire ----

fn hit_lately(id: u64) -> Option<Value> {
    let hits = LAST_HIT.lock().unwrap_or_else(|e| e.into_inner());
    let (tick, causer) = hits.as_ref()?.get(&id)?;
    (crate::scheduler::now().saturating_sub(*tick) <= 1 && !causer.is_null()).then(|| causer.clone())
}

/// bukkit: org/bukkit/event/entity/EntityCombustByEntityEvent
#[derive(Clone, Debug)]
pub struct EntityCombustByEntityEvent {
    value: Value,
    cancelled: bool,
}

impl EntityCombustByEntityEvent {
    /// bukkit: org/bukkit/event/entity/EntityCombustByEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    pub fn get_duration(&self) -> f32 {
        self.value["duration"].as_f64().unwrap_or(0.0) as f32
    }
    pub fn set_duration(&mut self, seconds: f32) {
        self.value["duration"] = json!(seconds);
        crate::event::carry("duration", json!(seconds));
    }
    /// bukkit: org/bukkit/event/entity/EntityCombustByEntityEvent.getCombuster
    pub fn get_combuster(&self) -> Option<Actor> {
        known_actor(&self.value["by"])
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

impl Event for EntityCombustByEntityEvent {
    const HOOK: &'static str = "ee_combust";
    fn more_hooks() -> &'static [&'static str] {
        IN_MOB_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["duration"].is_number() && value["by"]["block"].is_null() && !value["by"].is_null()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityCombustByBlockEvent
#[derive(Clone, Debug)]
pub struct EntityCombustByBlockEvent {
    value: Value,
    cancelled: bool,
}

impl EntityCombustByBlockEvent {
    /// bukkit: org/bukkit/event/entity/EntityCombustByBlockEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    pub fn get_duration(&self) -> f32 {
        self.value["duration"].as_f64().unwrap_or(0.0) as f32
    }
    pub fn set_duration(&mut self, seconds: f32) {
        self.value["duration"] = json!(seconds);
        crate::event::carry("duration", json!(seconds));
    }
    /// bukkit: org/bukkit/event/entity/EntityCombustByBlockEvent.getCombuster
    pub fn get_combuster(&self) -> Option<Block> {
        block_at(&world_of(&self.value["by"]), &self.value["by"]["pos"])
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

impl Event for EntityCombustByBlockEvent {
    const HOOK: &'static str = "ee_combust";
    fn more_hooks() -> &'static [&'static str] {
        IN_MOB_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (!value["by"]["block"].is_null()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

impl AreaEffectCloudApplyEvent {
    /// bukkit: org/bukkit/event/entity/AreaEffectCloudApplyEvent.getEntity
    pub fn get_entity(&self) -> crate::entities::AreaEffectCloud {
        Entity::of(self.value["cloud"].as_u64().unwrap_or(0), &world_of(&self.value)).cast()
    }
}

// ---- What a mob's tick asks of the game, part two ----

/// What a thrown potion of a witch's is, as a stack: the game keeps its effect.
fn thrown_potion(effect: &str) -> Option<ItemStack> {
    let potion = match effect.trim_start_matches("minecraft:") {
        "instant_damage" => "harming",
        "instant_health" => "healing",
        other => other,
    };
    ItemStack::read(&json!({ "item": "minecraft:splash_potion", "count": 1, "components": { "minecraft:potion_contents": { "potion": format!("minecraft:{potion}") } } }))
}

/// bukkit~: com/destroystokyo/paper/event/entity/WitchThrowPotionEvent | as she throws; the potion goes by its first effect
#[derive(Clone, Debug)]
pub struct WitchThrowPotionEvent {
    value: Value,
    potion: Option<ItemStack>,
    cancelled: bool,
}

impl WitchThrowPotionEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/WitchThrowPotionEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchThrowPotionEvent.getTarget
    pub fn get_target(&self) -> Option<Actor> {
        Actor::read(&self.value["target"], &world_of(&self.value))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchThrowPotionEvent.getPotion
    pub fn get_potion(&self) -> Option<&ItemStack> {
        self.potion.as_ref()
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchThrowPotionEvent.setPotion
    ///
    /// `None` is what a cancel is.
    pub fn set_potion(&mut self, potion: Option<ItemStack>) {
        self.potion = potion;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchThrowPotionEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/WitchThrowPotionEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WitchThrowPotionEvent {
    const HOOK: &'static str = "ee_mob";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "witch_throw").then(|| Self { value: value.clone(), potion: value["effect"].as_str().and_then(thrown_potion), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled || self.potion.is_none())
    }
    fn finish(&self) {
        if let Some(potion) = self.potion.as_ref().filter(|potion| Some(*potion) != self.value["effect"].as_str().and_then(thrown_potion).as_ref()) {
            query(json!({ "op": "ee_mob_change", "potion": potion_id(potion) }));
        }
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/TurtleLayEggEvent
#[derive(Clone, Debug)]
pub struct TurtleLayEggEvent {
    value: Value,
    eggs: i32,
    cancelled: bool,
}

impl TurtleLayEggEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleLayEggEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Turtle> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)).cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleLayEggEvent.getLocation
    pub fn get_location(&self) -> Option<Location> {
        block_at(&world_of(&self.value), &self.value["pos"]).map(|block| block.get_location())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleLayEggEvent.getEggCount
    pub fn get_egg_count(&self) -> i32 {
        self.eggs
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleLayEggEvent.setEggCount
    pub fn set_egg_count(&mut self, eggs: i32) {
        self.eggs = eggs;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleLayEggEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/TurtleLayEggEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for TurtleLayEggEvent {
    const HOOK: &'static str = "ee_mob";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "turtle_egg").then(|| Self { value: value.clone(), eggs: value["count"].as_i64().unwrap_or(1) as i32, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled || self.eggs < 1)
    }
    fn finish(&self) {
        if i64::from(self.eggs) != self.value["count"].as_i64().unwrap_or(1) {
            query(json!({ "op": "ee_mob_change", "eggs": self.eggs.clamp(1, 4) }));
        }
    }
}

/// bukkit: io/papermc/paper/event/entity/ShulkerDuplicateEvent
#[derive(Clone, Debug)]
pub struct ShulkerDuplicateEvent {
    value: Value,
    cancelled: bool,
}

impl ShulkerDuplicateEvent {
    /// bukkit: io/papermc/paper/event/entity/ShulkerDuplicateEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Shulker> {
        self.value["new"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)).cast())
    }
    /// bukkit: io/papermc/paper/event/entity/ShulkerDuplicateEvent.getParent
    pub fn get_parent(&self) -> Option<crate::entities::Shulker> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)).cast())
    }
    /// bukkit: io/papermc/paper/event/entity/ShulkerDuplicateEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/ShulkerDuplicateEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ShulkerDuplicateEvent {
    const HOOK: &'static str = "ee_mob";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "shulker_new").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- More of a mob's state between ticks, part two ----

/// bukkit: com/destroystokyo/paper/event/entity/EndermanAttackPlayerEvent
#[derive(Clone, Debug)]
pub struct EndermanAttackPlayerEvent {
    value: Value,
    cancelled: bool,
}

impl EndermanAttackPlayerEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanAttackPlayerEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Enderman> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanAttackPlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        Player::read(&self.value["target"])
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanAttackPlayerEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanAttackPlayerEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EndermanAttackPlayerEvent {
    const HOOK: &'static str = "ee_target";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"]["kind"] == "minecraft:enderman" && value["target"]["player"] == true).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/EndermanEscapeEvent
#[derive(Clone, Debug)]
pub struct EndermanEscapeEvent {
    value: Value,
    cancelled: bool,
}

impl EndermanEscapeEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanEscapeEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::Enderman> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanEscapeEvent.getReason
    pub fn get_reason(&self) -> &str {
        self.value["reason"].as_str().unwrap_or("RUNAWAY")
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanEscapeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EndermanEscapeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EndermanEscapeEvent {
    const HOOK: &'static str = "ee_enderman_escape";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/EntityPathfindEvent
#[derive(Clone, Debug)]
pub struct EntityPathfindEvent {
    value: Value,
    cancelled: bool,
}

impl EntityPathfindEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EntityPathfindEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityPathfindEvent.getLoc
    pub fn get_loc(&self) -> Option<Location> {
        location_at(&world_of(&self.value["entity"]), &self.value["at"])
    }
    /// bukkit~: com/destroystokyo/paper/event/entity/EntityPathfindEvent.getTargetEntity | the mob's target, when the path is to an entity: the game's path knows only the position
    pub fn get_target_entity(&self) -> Option<Actor> {
        known_actor(&self.value["target"])
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityPathfindEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityPathfindEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityPathfindEvent {
    const HOOK: &'static str = "ee_pathfind";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

fn is_slime(row: &Value) -> bool {
    row["kind"] == "minecraft:slime" || row["kind"] == "minecraft:magma_cube"
}

/// bukkit: com/destroystokyo/paper/event/entity/SlimePathfindEvent
#[derive(Clone, Debug)]
pub struct SlimePathfindEvent {
    value: Value,
    cancelled: bool,
}

impl SlimePathfindEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/SlimePathfindEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::AbstractCubeMob> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/SlimePathfindEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/SlimePathfindEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for SlimePathfindEvent {
    const HOOK: &'static str = "ee_slime_swim";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_slime_wander", "ee_slime_turn", "ee_target", "ee_tick"]
    }
    fn read(value: &Value) -> Option<Self> {
        (is_slime(&value["entity"]) && (value["reason"].is_null() || !value["target"].is_null())).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/SlimeSwimEvent
#[derive(Clone, Debug)]
pub struct SlimeSwimEvent {
    value: Value,
    cancelled: bool,
}

impl SlimeSwimEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/SlimeSwimEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::AbstractCubeMob> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
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

impl Event for SlimeSwimEvent {
    const HOOK: &'static str = "ee_slime_swim";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/SlimeWanderEvent
#[derive(Clone, Debug)]
pub struct SlimeWanderEvent {
    value: Value,
    cancelled: bool,
}

impl SlimeWanderEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/SlimeWanderEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::AbstractCubeMob> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
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

impl Event for SlimeWanderEvent {
    const HOOK: &'static str = "ee_slime_wander";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/SlimeChangeDirectionEvent
#[derive(Clone, Debug)]
pub struct SlimeChangeDirectionEvent {
    value: Value,
    cancelled: bool,
    yaw: f32,
}

impl SlimeChangeDirectionEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/SlimeChangeDirectionEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::AbstractCubeMob> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/SlimeChangeDirectionEvent.getNewYaw
    pub fn get_new_yaw(&self) -> f32 {
        self.yaw
    }
    /// bukkit: com/destroystokyo/paper/event/entity/SlimeChangeDirectionEvent.setNewYaw
    pub fn set_new_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
        crate::event::carry("yaw", json!(yaw));
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

impl Event for SlimeChangeDirectionEvent {
    const HOOK: &'static str = "ee_slime_turn";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(), yaw: value["yaw"].as_f64().unwrap_or(0.0) as f32 })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/SlimeTargetLivingEntityEvent
#[derive(Clone, Debug)]
pub struct SlimeTargetLivingEntityEvent {
    value: Value,
    cancelled: bool,
}

impl SlimeTargetLivingEntityEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::AbstractCubeMob> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/SlimeTargetLivingEntityEvent.getTarget
    pub fn get_target(&self) -> Option<Actor> {
        known_actor(&self.value["target"])
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

impl Event for SlimeTargetLivingEntityEvent {
    const HOOK: &'static str = "ee_target";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (is_slime(&value["entity"]) && !value["target"].is_null()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/WardenAngerChangeEvent
#[derive(Clone, Debug)]
pub struct WardenAngerChangeEvent {
    value: Value,
    cancelled: bool,
}

impl WardenAngerChangeEvent {
    /// bukkit: io/papermc/paper/event/entity/WardenAngerChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/WardenAngerChangeEvent.getTarget
    pub fn get_target(&self) -> Option<Actor> {
        known_actor(&self.value["target"])
    }
    /// bukkit: io/papermc/paper/event/entity/WardenAngerChangeEvent.getOldAnger
    pub fn get_old_anger(&self) -> i32 {
        self.value["old"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/entity/WardenAngerChangeEvent.getNewAnger
    pub fn get_new_anger(&self) -> i32 {
        self.value["new"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/entity/WardenAngerChangeEvent.setNewAnger
    pub fn set_new_anger(&mut self, anger: i32) {
        self.value["new"] = json!(anger);
        crate::event::carry("new", json!(anger));
    }
    /// bukkit: io/papermc/paper/event/entity/WardenAngerChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/WardenAngerChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for WardenAngerChangeEvent {
    const HOOK: &'static str = "ee_warden_anger";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/ArrowBodyCountChangeEvent
#[derive(Clone, Debug)]
pub struct ArrowBodyCountChangeEvent {
    value: Value,
    cancelled: bool,
}

impl ArrowBodyCountChangeEvent {
    /// bukkit: org/bukkit/event/entity/ArrowBodyCountChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/ArrowBodyCountChangeEvent.isReset
    pub fn is_reset(&self) -> bool {
        self.value["reset"] == true
    }
    /// bukkit: org/bukkit/event/entity/ArrowBodyCountChangeEvent.getOldAmount
    pub fn get_old_amount(&self) -> i32 {
        self.value["old"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/entity/ArrowBodyCountChangeEvent.getNewAmount
    pub fn get_new_amount(&self) -> i32 {
        self.value["new"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/event/entity/ArrowBodyCountChangeEvent.setNewAmount
    pub fn set_new_amount(&mut self, amount: i32) {
        self.value["new"] = json!(amount);
        crate::event::carry("new", json!(amount));
    }
    /// bukkit: org/bukkit/event/entity/ArrowBodyCountChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/ArrowBodyCountChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ArrowBodyCountChangeEvent {
    const HOOK: &'static str = "ee_arrows";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}


/// The bolt striking now.
fn bolt_now(world: &World) -> Option<crate::fam_entity::LightningStrike> {
    query(json!({ "op": "ee_hit_bolt" })).as_u64().map(|id| Entity::of(id, world).cast())
}

impl EntityZapEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EntityZapEvent.getBolt
    pub fn get_bolt(&self) -> Option<crate::fam_entity::LightningStrike> {
        bolt_now(&world_of(&self.value))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityZapEvent.getReplacementEntity
    pub fn get_replacement_entity(&self) -> Option<Entity> {
        self.value["replacement"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
}

impl PigZapEvent {
    /// bukkit: org/bukkit/event/entity/PigZapEvent.getLightning
    pub fn get_lightning(&self) -> Option<crate::fam_entity::LightningStrike> {
        bolt_now(&world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/PigZapEvent.getPigZombie
    pub fn get_pig_zombie(&self) -> Option<Entity> {
        self.value["replacement"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
}

impl crate::events::EntityBreakByEntityEvent {
    /// bukkit~: io/papermc/paper/event/entity/EntityBreakByEntityEvent.getDamageSource | a player's blow: a causing player isn't an `Entity` here, see `get_remover`
    pub fn get_damage_source(&self) -> crate::core2::DamageSource {
        crate::core2::DamageSource::builder("minecraft:player_attack")
    }
}

/// What `EntityPickupItemEvent.getItem()` gives: the picked-up stack and where it was taken. The game's hook doesn't
/// say which item entity it was, so its place is the picker's (a pickup is within a block or so of them).
#[derive(Clone, Debug)]
pub struct PickedItem {
    stack: Option<ItemStack>,
    location: Option<Location>,
}

impl PickedItem {
    /// bukkit: org/bukkit/entity/Item.getItemStack
    pub fn get_item_stack(&self) -> Option<ItemStack> {
        self.stack.clone()
    }
    /// bukkit: org/bukkit/entity/Entity.getLocation
    pub fn get_location(&self) -> Option<Location> {
        self.location.clone()
    }
    /// bukkit: org/bukkit/entity/Entity.getWorld
    pub fn get_world(&self) -> Option<World> {
        self.location.as_ref().map(|location| location.get_world().clone())
    }
}

/// bukkit: org/bukkit/event/entity/EntityPickupItemEvent
#[derive(Clone, Debug)]
pub struct EntityPickupItemEvent {
    value: Value,
    cancelled: bool,
}

impl EntityPickupItemEvent {
    /// bukkit: org/bukkit/event/entity/EntityPickupItemEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"]).or_else(|| self.value["uuid"].as_str().and_then(player_by_uuid).map(Actor::Player))
    }
    /// The picked-up stack.
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit~: org/bukkit/event/entity/EntityPickupItemEvent.getItem | the picked stack, placed where the picker is: a player's pickup doesn't say which item entity it was
    ///
    /// Java's is the Item entity; plugins ask it `getItemStack()` and `getLocation()` (CoreProtect logs pickups so).
    pub fn get_item_entity(&self) -> Option<PickedItem> {
        let location = match self.get_entity()? {
            Actor::Player(player) => Some(player.get_location()),
            Actor::Entity(entity) => entity.get_location(),
        };
        Some(PickedItem { stack: self.get_item(), location })
    }
    /// bukkit~: org/bukkit/event/entity/EntityPickupItemEvent.getRemaining | worked out from what fits in a player's inventory: the game adds it just after this
    pub fn get_remaining(&self) -> i32 {
        let (Some(Actor::Player(player)), Some(stack)) = (self.get_entity(), self.get_item()) else {
            return 0;
        };
        let most = query(json!({ "op": "item_info", "item": stack.get_type().key() }))["max_stack"].as_i64().unwrap_or(64) as i32;
        let inventory = player.get_inventory();
        let room: i32 = (0..36)
            .map(|slot| match inventory.get_item(slot) {
                None => most,
                Some(held) if held.get_type() == stack.get_type() => (most - held.get_amount()).max(0),
                Some(_) => 0,
            })
            .sum();
        (stack.get_amount() - room).max(0)
    }
    /// bukkit: org/bukkit/event/entity/EntityPickupItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityPickupItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityPickupItemEvent {
    const HOOK: &'static str = "item_pickup";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_pickup"]
    }
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- Placing, loading, jumping ----

/// bukkit: org/bukkit/event/entity/EntityPlaceEvent
#[derive(Clone, Debug)]
pub struct EntityPlaceEvent {
    value: Value,
    cancelled: bool,
}

impl EntityPlaceEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/EntityPlaceEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        self.value["uuid"].as_str().and_then(player_by_uuid)
    }
    /// bukkit: org/bukkit/event/entity/EntityPlaceEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPlaceEvent.getBlockFace
    pub fn get_block_face(&self) -> &str {
        self.value["face"].as_str().unwrap_or("SELF")
    }
    /// bukkit: org/bukkit/event/entity/EntityPlaceEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// What it was placed from.
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPlaceEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityPlaceEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityPlaceEvent {
    const HOOK: &'static str = "ee_place";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityLoadCrossbowEvent
#[derive(Clone, Debug)]
pub struct EntityLoadCrossbowEvent {
    value: Value,
    consume: bool,
    cancelled: bool,
}

impl EntityLoadCrossbowEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityLoadCrossbowEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"]).or_else(|| self.value["uuid"].as_str().and_then(player_by_uuid).map(Actor::Player))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLoadCrossbowEvent.getCrossbow
    pub fn get_crossbow(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLoadCrossbowEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLoadCrossbowEvent.shouldConsumeItem
    pub fn should_consume_item(&self) -> bool {
        self.consume
    }
    /// bukkit~: io/papermc/paper/event/entity/EntityLoadCrossbowEvent.setConsumeItem | the arrow is still drawn, then given back, maybe to another slot
    pub fn set_consume_item(&mut self, consume: bool) {
        self.consume = consume;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLoadCrossbowEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLoadCrossbowEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityLoadCrossbowEvent {
    const HOOK: &'static str = "ee_crossbow";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_mob_crossbow", "ee_tick"]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["ready"] == true || value["entity"].is_object()).then(|| Self { value: value.clone(), consume: value["creative"] != true, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        if !self.cancelled && !self.consume && self.value["creative"] != true && self.value["entity"].is_null()
            && let Some(mut stack) = ItemStack::read(&self.value["ammo"]).map(|stack| stack.to_json())
        {
            stack["count"] = json!(1);
            query(json!({ "op": "ee_give", "stack": stack }));
        }
    }
}

/// bukkit~: org/bukkit/event/entity/HorseJumpEvent | a guest's client moves the horse itself: their server hears the jump and its power, and a refusal can't stop it
#[derive(Clone, Debug)]
pub struct HorseJumpEvent {
    value: Value,
    cancelled: bool,
}

impl HorseJumpEvent {
    /// bukkit: org/bukkit/event/entity/HorseJumpEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::entities::AbstractHorse> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: org/bukkit/event/entity/HorseJumpEvent.getPower
    pub fn get_power(&self) -> f32 {
        self.value["power"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/event/entity/HorseJumpEvent.setPower
    pub fn set_power(&mut self, power: f32) {
        self.value["power"] = json!(power);
        crate::event::carry("power", json!(power));
    }
    /// bukkit: org/bukkit/event/entity/HorseJumpEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/HorseJumpEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for HorseJumpEvent {
    const HOOK: &'static str = "ee_horse_jump";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityAttemptSpinAttackEvent | a player's riptide reaching a mob
#[derive(Clone, Debug)]
pub struct EntityAttemptSpinAttackEvent {
    value: Value,
    cancelled: bool,
}

impl EntityAttemptSpinAttackEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityAttemptSpinAttackEvent.getEntity
    pub fn get_entity(&self) -> Option<Player> {
        self.value["uuid"].as_str().and_then(player_by_uuid)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityAttemptSpinAttackEvent.getTarget
    pub fn get_target(&self) -> Option<Entity> {
        self.value["target"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityAttemptSpinAttackEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityAttemptSpinAttackEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityAttemptSpinAttackEvent {
    const HOOK: &'static str = "ee_spin";
    fn read(value: &Value) -> Option<Self> {
        value["target"].is_u64().then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityAttemptSmashAttackEvent | a player's mace, for its fall bonus: `ALLOW` can't make a hit that isn't a smash into one, as only those are heard
#[derive(Clone, Debug)]
pub struct EntityAttemptSmashAttackEvent {
    value: Value,
    result: String,
}

impl EntityAttemptSmashAttackEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Player> {
        self.value["uuid"].as_str().and_then(player_by_uuid)
    }
    /// bukkit~: io/papermc/paper/event/entity/EntityAttemptSmashAttackEvent.getTarget | the victim's kind: this moment only has that
    pub fn get_target(&self) -> &str {
        self.value["victim"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/event/entity/EntityAttemptSmashAttackEvent.getWeapon
    pub fn get_weapon(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["weapon"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityAttemptSmashAttackEvent.getOriginalResult
    pub fn get_original_result(&self) -> bool {
        true
    }
    /// bukkit: io/papermc/paper/event/entity/EntityAttemptSmashAttackEvent.getResult
    pub fn get_result(&self) -> &str {
        &self.result
    }
    /// bukkit: io/papermc/paper/event/entity/EntityAttemptSmashAttackEvent.setResult
    pub fn set_result(&mut self, result: &str) {
        result.clone_into(&mut self.result);
    }
}

impl Event for EntityAttemptSmashAttackEvent {
    const HOOK: &'static str = "ee_smash";
    fn read(value: &Value) -> Option<Self> {
        (value["smash"] == true).then(|| Self { value: value.clone(), result: "DEFAULT".to_owned() })
    }
    fn answer(&self) -> Answer {
        answer(self.result == "DENY")
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityLungeEvent
#[derive(Clone, Debug)]
pub struct EntityLungeEvent {
    value: Value,
    power: i32,
    cancelled: bool,
}

impl EntityLungeEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Player> {
        self.value["uuid"].as_str().and_then(player_by_uuid)
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLungeEvent.getLungePower
    pub fn get_lunge_power(&self) -> i32 {
        self.power
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLungeEvent.setLungePower
    pub fn set_lunge_power(&mut self, power: i32) {
        self.power = power;
        crate::event::carry("power", json!(power));
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLungeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityLungeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityLungeEvent {
    const HOOK: &'static str = "ee_lunge";
    fn read(value: &Value) -> Option<Self> {
        (value["power"].as_i64().unwrap_or(0) > 0).then(|| Self { value: value.clone(), power: value["power"].as_i64().unwrap_or(0) as i32, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        if !self.cancelled {
            query(json!({ "op": "ee_lunge_power", "power": self.power }));
        }
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/PlayerNaturallySpawnCreaturesEvent
#[derive(Clone, Debug)]
pub struct PlayerNaturallySpawnCreaturesEvent {
    value: Value,
    cancelled: bool,
}

impl PlayerNaturallySpawnCreaturesEvent {
    /// bukkit: org/bukkit/event/player/PlayerEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        self.value["player"]["uuid"].as_str().and_then(player_by_uuid)
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PlayerNaturallySpawnCreaturesEvent.getSpawnRadius
    pub fn get_spawn_radius(&self) -> u8 {
        self.value["radius"].as_u64().unwrap_or(8) as u8
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PlayerNaturallySpawnCreaturesEvent.setSpawnRadius
    pub fn set_spawn_radius(&mut self, radius: u8) {
        self.value["radius"] = json!(radius);
        crate::event::carry("radius", json!(radius));
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PlayerNaturallySpawnCreaturesEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/PlayerNaturallySpawnCreaturesEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PlayerNaturallySpawnCreaturesEvent {
    const HOOK: &'static str = "ee_spawn_round";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        value["player"].is_object().then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- Thrown eggs and split slimes: what the game rolls inside, rolled here first ----

/// bukkit: com/destroystokyo/paper/event/entity/ThrownEggHatchEvent
#[derive(Clone, Debug)]
pub struct ThrownEggHatchEvent {
    value: Value,
    hatching: bool,
    count: i32,
    kind: String,
}

impl ThrownEggHatchEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/ThrownEggHatchEvent.getEgg
    pub fn get_egg(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ThrownEggHatchEvent.isHatching
    pub fn is_hatching(&self) -> bool {
        self.hatching
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ThrownEggHatchEvent.setHatching
    pub fn set_hatching(&mut self, hatching: bool) {
        self.hatching = hatching;
        crate::event::carry("hatching", json!(hatching));
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ThrownEggHatchEvent.getNumHatches
    pub fn get_num_hatches(&self) -> i32 {
        self.count
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ThrownEggHatchEvent.setNumHatches
    pub fn set_num_hatches(&mut self, count: i32) {
        self.count = count;
        crate::event::carry("count", json!(count));
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ThrownEggHatchEvent.getHatchingType
    pub fn get_hatching_type(&self) -> &str {
        &self.kind
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ThrownEggHatchEvent.setHatchingType
    pub fn set_hatching_type(&mut self, kind: &str) {
        kind.clone_into(&mut self.kind);
        crate::event::carry("kind", json!(kind));
    }
}

impl Event for ThrownEggHatchEvent {
    const HOOK: &'static str = "ee_egg";
    fn read(value: &Value) -> Option<Self> {
        value["hatching"].is_boolean().then(|| Self { value: value.clone(), hatching: value["hatching"] == true, count: value["count"].as_i64().unwrap_or(1) as i32, kind: value["kind"].as_str().unwrap_or("minecraft:chicken").to_owned() })
    }
}

/// bukkit: org/bukkit/event/entity/SlimeSplitEvent
#[derive(Clone, Debug)]
pub struct SlimeSplitEvent {
    value: Value,
    count: i32,
    cancelled: bool,
}

impl SlimeSplitEvent {
    /// bukkit: org/bukkit/event/entity/SlimeSplitEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: org/bukkit/event/entity/SlimeSplitEvent.getCount
    pub fn get_count(&self) -> i32 {
        self.count
    }
    /// bukkit: org/bukkit/event/entity/SlimeSplitEvent.setCount
    pub fn set_count(&mut self, count: i32) {
        self.count = count;
        crate::event::carry("count", json!(count));
    }
    /// bukkit: org/bukkit/event/entity/SlimeSplitEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/SlimeSplitEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for SlimeSplitEvent {
    const HOOK: &'static str = "ee_slime";
    fn read(value: &Value) -> Option<Self> {
        value["count"].is_i64().then(|| Self { value: value.clone(), count: value["count"].as_i64().unwrap_or(0) as i32, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/PiglinBarterEvent
#[derive(Clone, Debug)]
pub struct PiglinBarterEvent {
    value: Value,
    outcome: Vec<ItemStack>,
    cancelled: bool,
}

impl PiglinBarterEvent {
    /// bukkit: org/bukkit/event/entity/PiglinBarterEvent.getEntity
    pub fn get_entity(&self) -> crate::entities::Piglin {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value)).cast()
    }
    /// bukkit: org/bukkit/event/entity/PiglinBarterEvent.getInput
    pub fn get_input(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["input"])
    }
    /// bukkit: org/bukkit/event/entity/PiglinBarterEvent.getOutcome
    ///
    /// What it will throw: change this and that's what it throws.
    pub fn get_outcome(&mut self) -> &mut Vec<ItemStack> {
        &mut self.outcome
    }
    /// bukkit: org/bukkit/event/entity/PiglinBarterEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/PiglinBarterEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for PiglinBarterEvent {
    const HOOK: &'static str = "ee_barter";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), outcome: value["outcome"].as_array().into_iter().flatten().filter_map(ItemStack::read).collect(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        crate::event::carry("outcome", json!(self.outcome.iter().map(ItemStack::to_json).collect::<Vec<Value>>()));
    }
}

/// bukkit~: io/papermc/paper/event/entity/SulfurCubeSwallowItemEvent | a player handing it a block: what it picks up off the ground on its own isn't heard
#[derive(Clone, Debug)]
pub struct SulfurCubeSwallowItemEvent {
    value: Value,
    new_item: Option<ItemStack>,
    cancelled: bool,
}

impl SulfurCubeSwallowItemEvent {
    /// bukkit: io/papermc/paper/event/entity/SulfurCubeSwallowItemEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.value["entity"].as_u64().unwrap_or(0), &world_of(&self.value))
    }
    /// bukkit: io/papermc/paper/event/entity/SulfurCubeSwallowItemEvent.getPlayer
    pub fn get_player(&self) -> Option<Player> {
        self.value["uuid"].as_str().and_then(player_by_uuid)
    }
    /// bukkit: io/papermc/paper/event/entity/SulfurCubeSwallowItemEvent.getOldItem
    pub fn get_old_item(&self) -> Option<ItemStack> {
        ItemStack::read(&json!({ "item": self.value["old"], "count": 1 }))
    }
    /// bukkit: io/papermc/paper/event/entity/SulfurCubeSwallowItemEvent.getNewItem
    pub fn get_new_item(&self) -> Option<&ItemStack> {
        self.new_item.as_ref()
    }
    /// bukkit: io/papermc/paper/event/entity/SulfurCubeSwallowItemEvent.setNewItem
    pub fn set_new_item(&mut self, stack: ItemStack) {
        self.new_item = Some(stack);
    }
    /// bukkit: io/papermc/paper/event/entity/SulfurCubeSwallowItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/SulfurCubeSwallowItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for SulfurCubeSwallowItemEvent {
    const HOOK: &'static str = "ee_swallow";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_tick"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        (!value["ee_tick"].is_null()).then_some(Answer::Pass)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["takes"] == true).then(|| Self { value: value.clone(), new_item: ItemStack::read(&json!({ "item": value["item"], "count": 1 })), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        if let Some(stack) = self.new_item.as_ref().filter(|stack| !self.cancelled && stack.get_type().key() != self.value["item"].as_str().unwrap_or_default()) {
            query(json!({ "op": "ee_set_head", "dimension": world_of(&self.value).dimension(), "id": self.value["entity"], "item": stack.get_type().key() }));
        }
    }
}

// ---- Clouds, orbs, carts and blocks between ticks ----

#[derive(Default)]
struct Clouds {
    now: u64,
    seen: std::collections::BTreeSet<String>,
    fresh: Arc<Vec<(World, Value)>>,
}

static CLOUDS: Mutex<Option<Clouds>> = Mutex::new(None);

/// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFlameEvent
#[derive(Clone, Debug)]
pub struct EnderDragonFlameEvent {
    value: Value,
    cancelled: bool,
}

impl EnderDragonFlameEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFlameEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["dragon"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFlameEvent.getAreaEffectCloud
    pub fn get_area_effect_cloud(&self) -> Option<crate::entities::AreaEffectCloud> {
        Some(Entity::of(self.value["cloud"].as_u64()?, &world_of(&self.value)).cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFlameEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFlameEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EnderDragonFlameEvent {
    const HOOK: &'static str = "ee_dragon_flame";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFireballHitEvent
#[derive(Clone, Debug)]
pub struct EnderDragonFireballHitEvent {
    value: Value,
    cancelled: bool,
}

impl EnderDragonFireballHitEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFireballHitEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        Some(Entity::of(self.value["ball"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit~: com/destroystokyo/paper/event/entity/EnderDragonFireballHitEvent.getTargets | whoever stands in reach of the cloud: the mobs and items, with no players
    pub fn get_targets(&self) -> Vec<Entity> {
        location_at(&world_of(&self.value), &self.value["at"]).map_or_else(Vec::new, |at| world_of(&self.value).get_nearby_entities(&at, 4.0, 2.0, 4.0))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFireballHitEvent.getAreaEffectCloud
    pub fn get_area_effect_cloud(&self) -> Option<crate::entities::AreaEffectCloud> {
        Some(Entity::of(self.value["cloud"].as_u64()?, &world_of(&self.value)).cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFireballHitEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonFireballHitEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EnderDragonFireballHitEvent {
    const HOOK: &'static str = "ee_dragon_fireball";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

impl LingeringPotionSplashEvent {
    /// bukkit~: org/bukkit/event/entity/LingeringPotionSplashEvent.allowsEmptyCreation | kept for whoever asks: a potion with no effects makes no cloud here, for this to let through
    pub fn set_allows_empty_creation(&mut self, allows: bool) {
        self.allows_empty = allows;
    }
    pub fn allows_empty_creation(&self) -> bool {
        self.allows_empty
    }
    /// bukkit: org/bukkit/event/entity/LingeringPotionSplashEvent.getAreaEffectCloud
    pub fn get_area_effect_cloud(&self) -> Option<crate::entities::AreaEffectCloud> {
        Some(Entity::of(self.value["cloud"].as_u64()?, &world_of(&self.value)).cast())
    }
}

#[derive(Default)]
struct OrbStates {
    now: u64,
    seen: std::collections::HashMap<u64, (World, Value)>,
    merged: Arc<Vec<(World, Value, Value)>>,
}

static ORB_STATES: Mutex<Option<OrbStates>> = Mutex::new(None);

/// bukkit: com/destroystokyo/paper/event/entity/ExperienceOrbMergeEvent
#[derive(Clone, Debug)]
pub struct ExperienceOrbMergeEvent {
    value: Value,
    cancelled: bool,
}

impl ExperienceOrbMergeEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/ExperienceOrbMergeEvent.getMergeSource
    pub fn get_merge_source(&self) -> Option<crate::entities::ExperienceOrb> {
        known_entity(&self.value["source"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ExperienceOrbMergeEvent.getMergeTarget
    pub fn get_merge_target(&self) -> Option<crate::entities::ExperienceOrb> {
        known_entity(&self.value["entity"]).map(|entity| entity.cast())
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ExperienceOrbMergeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/ExperienceOrbMergeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for ExperienceOrbMergeEvent {
    const HOOK: &'static str = "ee_orb_merge";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityIgniteEvent
#[derive(Clone, Debug)]
pub struct EntityIgniteEvent {
    value: Value,
    cancelled: bool,
}

impl EntityIgniteEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityIgniteEvent.getFuseTime
    pub fn get_fuse_time(&self) -> i32 {
        self.value["fuse"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: io/papermc/paper/event/entity/EntityIgniteEvent.setFuseTime
    pub fn set_fuse_time(&mut self, fuse: i32) {
        self.value["fuse"] = json!(fuse);
        crate::event::carry("fuse", json!(fuse));
    }
    /// bukkit: io/papermc/paper/event/entity/EntityIgniteEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityIgniteEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityIgniteEvent {
    const HOOK: &'static str = "ee_cart_ignite";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: io/papermc/paper/event/entity/EntityInsideBlockEvent
#[derive(Clone, Debug)]
pub struct EntityInsideBlockEvent {
    value: Value,
    cancelled: bool,
}

impl EntityInsideBlockEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityInsideBlockEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityInsideBlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value["entity"]), &self.value["block"]["pos"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityInsideBlockEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityInsideBlockEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityInsideBlockEvent {
    const HOOK: &'static str = "ee_inside";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/TrialSpawnerSpawnEvent
#[derive(Clone, Debug)]
pub struct TrialSpawnerSpawnEvent {
    value: Value,
    cancelled: bool,
}

impl TrialSpawnerSpawnEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/TrialSpawnerSpawnEvent.getTrialSpawner
    pub fn get_trial_spawner(&self) -> Option<crate::BlockState> {
        block_at(&world_of(&self.value), &self.value["pos"]).map(|block| block.get_state())
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

impl Event for TrialSpawnerSpawnEvent {
    const HOOK: &'static str = "ee_trial_spawn";
    fn more_hooks() -> &'static [&'static str] {
        &[]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["pos"].is_array()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

impl EntityPortalEvent {
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.getTo
    pub fn get_to(&self) -> Option<Location> {
        location_at(&World::of(self.value["to_level"].as_str()?), &self.value["to"])
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.getSearchRadius
    pub fn get_search_radius(&self) -> i32 {
        self.value["search"].as_i64().unwrap_or(128) as i32
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.getCreationRadius
    pub fn get_creation_radius(&self) -> i32 {
        self.value["creation"].as_i64().unwrap_or(16) as i32
    }
    /// bukkit: org/bukkit/event/entity/EntityPortalEvent.getCanCreatePortal
    pub fn get_can_create_portal(&self) -> bool {
        self.value["can_create"] != false
    }
}

impl PlayerLeashEntityEvent {
    /// bukkit: org/bukkit/event/entity/PlayerLeashEntityEvent.getHand
    pub fn get_hand(&self) -> &str {
        self.value["hand"].as_str().unwrap_or("HAND")
    }
}

// ---- A mob on a block that reacts to it ----

/// bukkit~: org/bukkit/event/entity/EntityInteractEvent | a mob on a pressure plate, a tripwire or farmland, as the block changes: other blocks that react to a mob do it inside its own move
#[derive(Clone, Debug)]
pub struct EntityInteractEvent {
    value: Value,
    entity: u64,
    cancelled: bool,
}

impl EntityInteractEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Entity {
        Entity::of(self.entity, &world_of(&self.value))
    }
    /// bukkit: org/bukkit/event/entity/EntityInteractEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/entity/EntityInteractEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityInteractEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityInteractEvent {
    const HOOK: &'static str = "block_change";
    fn read(value: &Value) -> Option<Self> {
        let (old, new) = (value["old"].as_str()?, value["new"].as_str()?);
        let plate = (old.contains("pressure_plate") || old.contains("tripwire")) && value["kind"] == "redstone" && (new.contains("powered=true") || new.contains("power=") && !new.contains("power=0"));
        let trampled = old.starts_with("minecraft:farmland") && new.starts_with("minecraft:dirt");
        if !plate && !trampled {
            return None;
        }
        let entity = query(json!({ "op": "ee_mob_at", "dimension": value["level"], "pos": value["pos"] })).as_u64()?;
        Some(Self { value: value.clone(), entity, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- Who pushed who ----

fn knocked_by(value: &Value) -> Option<Value> {
    hit_lately(value["eid"].as_u64().or(value["entity"]["id"].as_u64())?)
}

fn push_as(value: &Value, vector: crate::util::Vector) {
    let power = (vector.x * vector.x + vector.z * vector.z).sqrt();
    if value["entity"].is_object() {
        crate::event::carry("power", json!(power));
        crate::event::carry("x", json!(-vector.x));
        crate::event::carry("z", json!(-vector.z));
    } else {
        query(json!({ "op": "ee_knock_set", "power": power, "x": -vector.x, "z": -vector.z }));
    }
}

impl EntityKnockbackEvent {
    /// bukkit~: io/papermc/paper/event/entity/EntityKnockbackEvent.setKnockback, org/bukkit/event/entity/EntityKnockbackEvent.setFinalKnockback | its direction and strength along the ground: the game works out the lift itself
    pub fn set_knockback(&self, knockback: crate::util::Vector) {
        push_as(&self.value, knockback);
    }
    /// bukkit~: org/bukkit/event/entity/EntityKnockbackEvent.setFinalKnockback | as set_knockback
    pub fn set_final_knockback(&self, knockback: crate::util::Vector) {
        push_as(&self.value, knockback);
    }
}

/// bukkit~: com/destroystokyo/paper/event/entity/EntityKnockbackByEntityEvent, org/bukkit/event/entity/EntityKnockbackByEntityEvent | who hit them is who last hurt them, as the push doesn't say
#[derive(Clone, Debug)]
pub struct EntityKnockbackByEntityEvent {
    value: Value,
    by: Value,
    cancelled: bool,
}

impl EntityKnockbackByEntityEvent {
    /// bukkit: org/bukkit/event/entity/EntityKnockbackByEntityEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"]).or_else(|| self.value["uuid"].as_str().and_then(player_by_uuid).map(Actor::Player))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityKnockbackByEntityEvent.getHitBy
    pub fn get_hit_by(&self) -> Option<Actor> {
        known_actor(&self.by)
    }
    /// bukkit: org/bukkit/event/entity/EntityKnockbackByEntityEvent.getSourceEntity
    pub fn get_source_entity(&self) -> Option<Actor> {
        known_actor(&self.by)
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EntityKnockbackByEntityEvent.getKnockbackStrength
    pub fn get_knockback_strength(&self) -> f64 {
        self.value["power"].as_f64().unwrap_or(0.0)
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

impl Event for EntityKnockbackByEntityEvent {
    const HOOK: &'static str = "ee_knock";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_damage_info", "ee_tick", "ee_mob_knock"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        note_damage(value).or_else(|| verdict(value))
    }
    fn read(value: &Value) -> Option<Self> {
        knocked_by(value).map(|by| Self { value: value.clone(), by, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityPushedByEntityAttackEvent | who pushed them is who last hurt them, as the push doesn't say
#[derive(Clone, Debug)]
pub struct EntityPushedByEntityAttackEvent {
    value: Value,
    by: Value,
    cancelled: bool,
}

impl EntityPushedByEntityAttackEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityPushedByEntityAttackEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"]).or_else(|| self.value["uuid"].as_str().and_then(player_by_uuid).map(Actor::Player))
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPushedByEntityAttackEvent.getPushedBy
    pub fn get_pushed_by(&self) -> Option<Actor> {
        known_actor(&self.by)
    }
    /// bukkit~: io/papermc/paper/event/entity/EntityPushedByEntityAttackEvent.getAcceleration | what the push adds, worked out as the game will
    pub fn get_acceleration(&self) -> crate::util::Vector {
        let (power, x, z) = (self.value["power"].as_f64().unwrap_or(0.0), self.value["x"].as_f64().unwrap_or(0.0), self.value["z"].as_f64().unwrap_or(0.0));
        let length = (x * x + z * z).sqrt().max(1.0e-7);
        crate::util::Vector::new(-x / length * power, power.min(0.4), -z / length * power)
    }
    /// bukkit~: io/papermc/paper/event/entity/EntityPushedByEntityAttackEvent.setAcceleration | its direction and strength along the ground: the game works out the lift itself
    pub fn set_acceleration(&self, acceleration: crate::util::Vector) {
        push_as(&self.value, acceleration);
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

impl Event for EntityPushedByEntityAttackEvent {
    const HOOK: &'static str = "ee_knock";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_damage_info", "ee_tick", "ee_mob_knock"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        note_damage(value).or_else(|| verdict(value))
    }
    fn read(value: &Value) -> Option<Self> {
        knocked_by(value).map(|by| Self { value: value.clone(), by, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

// ---- A dead pet, and a player's own death screen ----

/// bukkit: io/papermc/paper/event/entity/TameableDeathMessageEvent
#[derive(Clone, Debug)]
pub struct TameableDeathMessageEvent {
    value: Value,
    message: Option<String>,
    cancelled: bool,
}

impl TameableDeathMessageEvent {
    /// bukkit: io/papermc/paper/event/entity/TameableDeathMessageEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: io/papermc/paper/event/entity/TameableDeathMessageEvent.deathMessage
    ///
    /// The line to the owner in place of the game's.
    pub fn death_message(&mut self, message: &crate::Component) {
        self.message = Some(message.plain());
    }
    /// bukkit: io/papermc/paper/event/entity/TameableDeathMessageEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/TameableDeathMessageEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for TameableDeathMessageEvent {
    const HOOK: &'static str = "ee_pet_died";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), message: None, cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled || self.message.is_some())
    }
    fn finish(&self) {
        if let (false, Some(message), Some(owner)) = (self.cancelled, &self.message, self.value["owner"].as_str().and_then(player_by_uuid)) {
            owner.send_message(message);
        }
    }
}

impl PlayerDeathEvent {
    /// bukkit~: org/bukkit/event/entity/PlayerDeathEvent.deathScreenMessageOverride | as plain text, and sent over the game's own line a moment after: the screen shows that one first
    pub fn death_screen_message_override(&mut self, message: Option<&crate::Component>) {
        self.screen = message.map(crate::Component::plain);
    }
}

// ---- A water bottle, a geyser, a fishing hook ----

/// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent
#[derive(Clone, Debug)]
pub struct WaterBottleSplashEvent {
    value: Value,
    intensities: Vec<(Value, f64)>,
    spared: Vec<Value>,
    added: Vec<Value>,
    cancelled: bool,
}

impl WaterBottleSplashEvent {
    fn who_with(&self, wanted: impl Fn(&Value) -> bool) -> Vec<Actor> {
        let world = world_of(&self.value);
        self.value["affected"].as_array().into_iter().flatten().filter(|_| self.value["water"] == true).filter(|row| wanted(row)).filter_map(|row| Actor::read(&row["who"], &world)).collect()
    }
    /// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent.getAffectedEntities
    pub fn get_affected_entities(&self) -> Vec<Actor> {
        self.who_with(|_| true)
    }
    /// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent.getToDamage
    pub fn get_to_damage(&self) -> Vec<Actor> {
        self.who_with(|row| row["sensitive"] == true && !self.spared.contains(&row["who"]))
    }
    /// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent.getToExtinguish
    pub fn get_to_extinguish(&self) -> Vec<Actor> {
        self.who_with(|row| row["fire"].as_i64().unwrap_or(0) > 0)
    }
    /// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent.getToRehydrate
    pub fn get_to_rehydrate(&self) -> Vec<Actor> {
        self.who_with(|row| row["kind"] == "minecraft:axolotl")
    }
    /// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent.getIntensity
    pub fn get_intensity(&self, entity: &Actor) -> f64 {
        let who = entity.to_json();
        if let Some((_, intensity)) = self.intensities.iter().find(|(row, _)| *row == who) {
            return *intensity;
        }
        self.value["affected"].as_array().into_iter().flatten().find(|row| row["who"]["id"] == who["id"] && row["who"]["uuid"] == who["uuid"]).and_then(|row| row["intensity"].as_f64()).unwrap_or(0.0)
    }
    /// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent.setIntensity
    pub fn set_intensity(&mut self, entity: &Actor, intensity: f64) {
        let who = entity.to_json();
        self.intensities.retain(|(row, _)| *row != who);
        self.intensities.push((who, intensity));
    }
    /// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent.doNotDamageAsWaterSensitive
    pub fn do_not_damage_as_water_sensitive(&mut self, entity: &Actor) {
        self.spared.push(entity.to_json());
    }
    /// bukkit: io/papermc/paper/event/entity/WaterBottleSplashEvent.damageAsWaterSensitive
    pub fn damage_as_water_sensitive(&mut self, entity: &Actor) {
        self.added.push(entity.to_json());
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

impl Event for WaterBottleSplashEvent {
    const HOOK: &'static str = "ee_splash";
    fn more_hooks() -> &'static [&'static str] {
        &["ee_splash_player", "ee_splash_owed"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value).or_else(|| (!value["ee_owed"].is_null()).then_some(Answer::Pass))
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "splash").then(|| Self { value: value.clone(), intensities: Vec::new(), spared: Vec::new(), added: Vec::new(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
    fn finish(&self) {
        for (who, intensity) in &self.intensities {
            query(json!({ "op": "ee_intensity", "who": who, "intensity": intensity }));
        }
        for who in &self.spared {
            query(json!({ "op": "ee_intensity", "who": who, "intensity": 0.0 }));
        }
        for who in &self.added {
            query(json!({ "op": "ee_wet", "who": who }));
        }
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityPotentSulfurLaunchEvent | mobs over a geyser, each tick it lifts them: the items and TNT it lifts aren't heard
#[derive(Clone, Debug)]
pub struct EntityPotentSulfurLaunchEvent {
    value: Value,
    cancelled: bool,
}

impl EntityPotentSulfurLaunchEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPotentSulfurLaunchEvent.getSulfurBlock
    pub fn get_sulfur_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value["entity"]), &self.value["pos"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPotentSulfurLaunchEvent.getSourceLocation
    pub fn get_source_location(&self) -> Option<Location> {
        block_at(&world_of(&self.value["entity"]), &self.value["source"]).map(|block| block.get_location())
    }
    /// bukkit: io/papermc/paper/event/entity/EntityPotentSulfurLaunchEvent.getLaunchVelocity
    ///
    /// The geyser's push: 0.2 upward a tick, as long as it's slower than its limit.
    pub fn get_launch_velocity(&self) -> crate::util::Vector {
        crate::util::Vector::new(0.0, 0.2, 0.0)
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

impl Event for EntityPotentSulfurLaunchEvent {
    const HOOK: &'static str = "ee_sulfur";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_object()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled(),  })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// What each hook last was, by its id.
static HOOKS: Mutex<Option<std::collections::HashMap<u64, Value>>> = Mutex::new(None);

/// bukkit: io/papermc/paper/event/entity/FishHookStateChangeEvent
#[derive(Clone, Debug)]
pub struct FishHookStateChangeEvent {
    value: Value,
}

impl FishHookStateChangeEvent {
    /// bukkit: io/papermc/paper/event/entity/FishHookStateChangeEvent.getEntity
    pub fn get_entity(&self) -> Option<crate::fam_entity::FishHook> {
        player_by_uuid(self.value["uuid"].as_str()?)?.get_fish_hook()
    }
    /// bukkit: io/papermc/paper/event/entity/FishHookStateChangeEvent.getNewHookState
    pub fn get_new_hook_state(&self) -> &'static str {
        let state = self.value["state"].as_str().unwrap_or_default();
        if state == "Bobbing" {
            "BOBBING"
        } else if state.starts_with("Hooked") {
            "HOOKED_ENTITY"
        } else {
            "UNHOOKED"
        }
    }
}

impl Event for FishHookStateChangeEvent {
    const HOOK: &'static str = "ee_fish";
    fn read(value: &Value) -> Option<Self> {
        let hook = value["hook"].as_u64()?;
        let before = HOOKS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(hook, value["state"].clone());
        (before.is_some_and(|before| before != value["state"])).then(|| Self { value: value.clone() })
    }
}

// ---- What EssentialsX Protect needs of a block a mob changes, and of health coming back ----

/// bukkit: org/bukkit/event/entity/EntityChangeBlockEvent
#[derive(Clone, Debug)]
pub struct EntityChangeBlockEvent {
    value: Value,
    cancelled: bool,
}

impl EntityChangeBlockEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        known_entity(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    ///
    /// The game's id of that mob: what Java's `getEntityType` says.
    pub fn get_entity_type(&self) -> &str {
        self.value["entity"]["kind"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/entity/EntityChangeBlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    /// bukkit: org/bukkit/event/entity/EntityChangeBlockEvent.getTo
    pub fn get_to(&self) -> &str {
        let new = self.value["new"].as_str().unwrap_or_default();
        new.split('[').next().unwrap_or(new)
    }
    /// bukkit: org/bukkit/event/entity/EntityChangeBlockEvent.getBlockData
    pub fn get_block_data(&self) -> &str {
        self.value["new"].as_str().unwrap_or_default()
    }
    /// bukkit: org/bukkit/event/entity/EntityChangeBlockEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityChangeBlockEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityChangeBlockEvent {
    const HOOK: &'static str = "ee_mob";
    fn intercept(value: &Value) -> Option<Answer> {
        verdict(value)
    }
    fn read(value: &Value) -> Option<Self> {
        (value["kind"] == "change_block").then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent
#[derive(Clone, Debug)]
pub struct EntityRegainHealthEvent {
    value: Value,
    amount: f64,
    cancelled: bool,
}

impl EntityRegainHealthEvent {
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent.getEntity
    pub fn get_entity(&self) -> Option<Actor> {
        known_actor(&self.value["entity"])
    }
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent.getAmount
    pub fn get_amount(&self) -> f64 {
        self.amount
    }
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent.setAmount
    pub fn set_amount(&mut self, amount: f64) {
        self.amount = amount;
        crate::event::carry("amount", json!(amount));
    }
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent.getRegainReason
    pub fn get_regain_reason(&self) -> &str {
        self.value["reason"].as_str().unwrap_or("CUSTOM")
    }
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent.isFastRegen
    pub fn is_fast_regen(&self) -> bool {
        self.value["fast"] == true
    }
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/EntityRegainHealthEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityRegainHealthEvent {
    const HOOK: &'static str = "ee_regain";
    fn more_hooks() -> &'static [&'static str] {
        IN_TICK
    }
    fn read(value: &Value) -> Option<Self> {
        value["entity"].is_object().then(|| Self { value: value.clone(), amount: value["amount"].as_f64().unwrap_or(0.0), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit: com/destroystokyo/paper/event/entity/EnderDragonShootFireballEvent
#[derive(Clone, Debug)]
pub struct EnderDragonShootFireballEvent {
    value: Value,
    cancelled: bool,
}

impl EnderDragonShootFireballEvent {
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonShootFireballEvent.getEntity
    pub fn get_entity(&self) -> Option<Entity> {
        self.value["entity"].as_u64().map(|id| Entity::of(id, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonShootFireballEvent.getFireball
    pub fn get_fireball(&self) -> Option<Entity> {
        Some(Entity::of(self.value["ball"].as_u64()?, &world_of(&self.value)))
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonShootFireballEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: com/destroystokyo/paper/event/entity/EnderDragonShootFireballEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EnderDragonShootFireballEvent {
    const HOOK: &'static str = "ee_dragon_fire";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}

/// bukkit~: io/papermc/paper/event/entity/EntityCompostItemEvent | a player's item, one by one; a farmer's turn is one event, as the game adds its whole inventory at once
#[derive(Clone, Debug)]
pub struct EntityCompostItemEvent {
    value: Value,
    cancelled: bool,
}

impl EntityCompostItemEvent {
    /// bukkit: io/papermc/paper/event/entity/EntityCompostItemEvent.getEntity
    ///
    /// The farmer, or the player: Java's `getEntity`.
    pub fn get_entity(&self) -> Option<Actor> {
        if let Some(id) = self.value["entity"].as_u64() {
            return Some(Actor::Entity(Entity::of(id, &world_of(&self.value))));
        }
        self.value["uuid"].as_str().and_then(player_by_uuid).map(Actor::Player)
    }
    /// bukkit: org/bukkit/event/block/BlockEvent.getBlock
    pub fn get_block(&self) -> Option<Block> {
        block_at(&world_of(&self.value), &self.value["pos"])
    }
    pub fn get_item(&self) -> Option<ItemStack> {
        ItemStack::read(&self.value["stack"])
    }
    /// bukkit: io/papermc/paper/event/entity/EntityCompostItemEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: io/papermc/paper/event/entity/EntityCompostItemEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

impl Event for EntityCompostItemEvent {
    const HOOK: &'static str = "ee_compost";
    fn more_hooks() -> &'static [&'static str] {
        &["ew_compost"]
    }
    fn read(value: &Value) -> Option<Self> {
        (value["entity"].is_u64() || !value["levels"].is_null()).then(|| Self { value: value.clone(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        answer(self.cancelled)
    }
}
