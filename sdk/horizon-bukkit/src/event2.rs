//! More of `org.bukkit.event`. Those on a bridge hook are the game's own moment; those marked as
//! changes are read from a player's state between one tick and the next, so they arrive a tick
//! late and can't be cancelled.
use crate::entity::Entity;
use crate::event::Event;
use crate::{Block, Player, World, get_online_players};
use horizon_mod::{Answer, Value, query};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn player_of(value: &Value) -> Option<Player> {
    Player::read(&value["player"]).or_else(|| Player::read(value)).or_else(|| Player::read(&query(json!({ "op": "player", "uuid": value["uuid"] }))))
}

fn level_of(value: &Value) -> World {
    World::of(value["level"].as_str().unwrap_or("minecraft:overworld"))
}

fn block_at(world: &World, pos: &Value) -> Option<Block> {
    let at = |i: usize| pos[i].as_i64().map(|v| v as i32);
    Some(world.get_block_at(at(0)?, at(1)?, at(2)?))
}

// ---- What a player's state did since last tick ----

#[derive(Clone, PartialEq)]
struct Seen {
    sneaking: bool,
    sprinting: bool,
    flying: bool,
    gliding: bool,
    selected: i64,
    food: i64,
    level: i64,
    game_mode: i64,
}

struct Change {
    player: Player,
    before: Seen,
    now: Seen,
}

#[derive(Default)]
struct Changes {
    now: u64,
    seen: HashMap<String, Seen>,
    changes: Arc<Vec<Change>>,
}

static CHANGES: Mutex<Option<Changes>> = Mutex::new(None);

fn changes() -> Arc<Vec<Change>> {
    let mut all = CHANGES.lock().unwrap_or_else(|e| e.into_inner());
    let all = all.get_or_insert_default();
    let now = crate::scheduler::now();
    if all.now == now {
        return all.changes.clone();
    }
    all.now = now;
    let mut changes = Vec::new();
    let mut seen = HashMap::new();
    for player in get_online_players() {
        let state = query(json!({ "op": "player", "uuid": player.uuid() }));
        let flag = |key: &str| state[key].as_bool().unwrap_or(false);
        let number = |key: &str| state[key].as_i64().unwrap_or(0);
        let current = Seen {
            sneaking: flag("sneaking"),
            sprinting: flag("sprinting"),
            flying: flag("flying"),
            gliding: flag("gliding"),
            selected: number("selected"),
            food: number("food"),
            level: number("level"),
            game_mode: number("game_mode"),
        };
        if let Some(before) = all.seen.get(player.uuid())
            && *before != current
        {
            changes.push(Change { player: player.clone(), before: before.clone(), now: current.clone() });
        }
        seen.insert(player.uuid().to_owned(), current);
    }
    all.seen = seen;
    all.changes = Arc::new(changes);
    all.changes.clone()
}

// These have one version now, in `fam_ev_player`.
pub use crate::fam_ev_player::{PlayerInteractEntityEvent, PlayerItemHeldEvent, PlayerToggleFlightEvent, PlayerBedEnterEvent, PlayerBedLeaveEvent, PlayerGameModeChangeEvent, PlayerToggleSneakEvent, PlayerToggleSprintEvent};

macro_rules! change_event {
    ($(#[$doc:meta])* $name:ident, $field:ident, $value:ty, $getter:ident, $previous:ident $(; $undo:expr)?) => {
        $(#[$doc])*
        pub struct $name {
            pub player: Player,
            before: $value,
            now: $value,
            cancelled: bool,
        }
        impl $name {
            pub fn get_player(&self) -> &Player {
                &self.player
            }
            pub fn $getter(&self) -> $value {
                self.now
            }
            pub fn $previous(&self) -> $value {
                self.before
            }
        }
        impl Event for $name {
            const HOOK: &'static str = "tick";
            fn read(_: &Value) -> Option<Self> {
                None
            }
            fn read_all(_: &Value) -> Vec<Self> {
                changes()
                    .iter()
                    .filter(|change| change.before.$field != change.now.$field)
                    .map(|change| Self { player: change.player.clone(), before: change.before.$field, now: change.now.$field, cancelled: crate::event::prior_cancelled() })
                    .collect()
            }
            fn finish(&self) {
                $(if self.cancelled {
                    let undo: fn(&Player, $value) = $undo;
                    undo(&self.player, self.before);
                })?
            }
        }
    };
}

change_event!(
    /// bukkit~: org/bukkit/event/entity/EntityToggleGlideEvent | players only, a tick late, and can't be cancelled
    EntityToggleGlideEvent, gliding, bool, is_gliding, was_gliding
);
change_event!(
    /// bukkit~: org/bukkit/event/entity/FoodLevelChangeEvent | players only, a tick late; a cancel puts the food back
    FoodLevelChangeEvent, food, i64, get_food_level, get_previous_food_level; |player, before| player.set_food_level(before as i32)
);

impl FoodLevelChangeEvent {
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.setCancelled
    pub fn set_cancelled(&mut self, cancelled: bool) {
        self.cancelled = cancelled;
    }
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.isCancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.setFoodLevel
    pub fn set_food_level(&self, food: i32) {
        self.player.set_food_level(food);
    }
    /// bukkit: org/bukkit/event/entity/FoodLevelChangeEvent.getEntity
    pub fn get_entity(&self) -> &Player {
        &self.player
    }
}
change_event!(
    /// bukkit: org/bukkit/event/player/PlayerLevelChangeEvent, org/bukkit/event/player/PlayerLevelChangeEvent.getOldLevel, org/bukkit/event/player/PlayerLevelChangeEvent.getNewLevel
    PlayerLevelChangeEvent, level, i64, get_new_level, get_old_level
);

// ---- The world's state between ticks ----

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Sky {
    raining: bool,
    thundering: bool,
}

static SKIES: Mutex<Option<HashMap<String, Sky>>> = Mutex::new(None);

/// A level whose sky changed since last tick: what it was, and what it is.
fn sky_changes() -> Vec<(World, Sky, Sky)> {
    let mut seen = SKIES.lock().unwrap_or_else(|e| e.into_inner());
    let seen = seen.get_or_insert_default();
    let mut changes = Vec::new();
    for world in crate::get_worlds() {
        let now = Sky { raining: world.has_storm(), thundering: world.is_thundering() };
        if let Some(before) = seen.insert(world.get_name().to_owned(), now)
            && before != now
        {
            changes.push((world, before, now));
        }
    }
    changes
}







// ---- On a bridge hook ----




macro_rules! block_player_event {
    ($(#[$doc:meta])* $name:ident, $hook:literal) => {
        $(#[$doc])*
        pub struct $name {
            pub player: Player,
            pub block: Option<Block>,
        }
        impl $name {
            pub fn get_player(&self) -> &Player {
                &self.player
            }
            pub fn get_bed(&self) -> Option<&Block> {
                self.block.as_ref()
            }
        }
        impl Event for $name {
            const HOOK: &'static str = $hook;
            fn read(value: &Value) -> Option<Self> {
                let player = player_of(value)?;
                let block = block_at(&player.get_world(), &value["pos"]);
                Some(Self { player, block })
            }
        }
    };
}


/// bukkit: org/bukkit/event/world/ChunkLoadEvent
pub struct ChunkLoadEvent {
    pub chunk: crate::entity::Chunk,
    pub new_chunk: bool,
}

impl ChunkLoadEvent {
    pub fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
    /// bukkit: org/bukkit/event/world/ChunkLoadEvent.isNewChunk
    pub fn is_new_chunk(&self) -> bool {
        self.new_chunk
    }
}

impl Event for ChunkLoadEvent {
    const HOOK: &'static str = "chunk_load";
    fn read(value: &Value) -> Option<Self> {
        let at = |i: usize| value["chunk"][i].as_i64().map(|v| v as i32);
        Some(Self { chunk: level_of(value).get_chunk_at(at(0)?, at(1)?), new_chunk: value["new"].as_bool().unwrap_or(false) })
    }
}

/// bukkit: org/bukkit/event/world/ChunkUnloadEvent
pub struct ChunkUnloadEvent {
    pub chunk: crate::entity::Chunk,
}

impl ChunkUnloadEvent {
    pub fn get_chunk(&self) -> &crate::entity::Chunk {
        &self.chunk
    }
}

impl Event for ChunkUnloadEvent {
    const HOOK: &'static str = "chunk_unload";
    fn read(value: &Value) -> Option<Self> {
        let at = |i: usize| value["chunk"][i].as_i64().map(|v| v as i32);
        Some(Self { chunk: level_of(value).get_chunk_at(at(0)?, at(1)?) })
    }
}

/// bukkit~: org/bukkit/event/entity/EntityDeathEvent | after it died: drops and experience can't be changed
pub struct EntityDeathEvent {
    pub entity_id: u64,
    pub kind: String,
    pub killer: Option<Player>,
}

impl EntityDeathEvent {
    pub fn get_entity_id(&self) -> u64 {
        self.entity_id
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    pub fn get_entity_type(&self) -> &str {
        &self.kind
    }
    pub fn get_killer(&self) -> Option<&Player> {
        self.killer.as_ref()
    }
}

impl Event for EntityDeathEvent {
    const HOOK: &'static str = "mob_death";
    fn read(value: &Value) -> Option<Self> {
        let killer = value["killer"].as_str().and_then(|uuid| Player::read(&query(json!({ "op": "player", "uuid": uuid }))));
        Some(Self { entity_id: value["entity"].as_u64()?, kind: value["kind"].as_str().unwrap_or_default().to_owned(), killer })
    }
}

/// bukkit~: org/bukkit/event/entity/EntitySpawnEvent, org/bukkit/event/entity/CreatureSpawnEvent | as it joins the level, with no spawn reason, and can't be cancelled yet
pub struct EntitySpawnEvent {
    pub entity: Entity,
    pub kind: String,
}

impl EntitySpawnEvent {
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntity, org/bukkit/event/hanging/HangingEvent.getEntity
    pub fn get_entity(&self) -> &Entity {
        &self.entity
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    pub fn get_entity_type(&self) -> &str {
        &self.kind
    }
}

/// What the game was last doing that spawns things.
static SPAWN_REASON: Mutex<&'static str> = Mutex::new("DEFAULT");

fn known_reason(reason: &str) -> &'static str {
    ["NATURAL", "SPAWNER_EGG", "BREEDING", "SPAWNER", "VILLAGE_DEFENSE", "RAID", "CONVERSION"].into_iter().find(|known| *known == reason).unwrap_or("DEFAULT")
}

impl EntitySpawnEvent {
    /// bukkit~: org/bukkit/event/entity/CreatureSpawnEvent.getSpawnReason | what the game last did that spawns, so a plugin's own spawn right after can read as that
    pub fn get_spawn_reason(&self) -> &'static str {
        *SPAWN_REASON.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Event for EntitySpawnEvent {
    fn more_hooks() -> &'static [&'static str] {
        &["why_natural", "why_egg", "why_breed", "why_spawner", "why_golem", "why_raid", "why_phantom", "why_convert", "why_trader"]
    }
    fn intercept(value: &Value) -> Option<Answer> {
        let reason = value["reason"].as_str()?;
        *SPAWN_REASON.lock().unwrap_or_else(|e| e.into_inner()) = known_reason(reason);
        Some(Answer::Pass)
    }
    const HOOK: &'static str = "entity_load";
    fn read(value: &Value) -> Option<Self> {
        let world = level_of(value);
        Some(Self { entity: Entity::of(value["entity"].as_u64()?, &world), kind: value["kind"].as_str().unwrap_or_default().to_owned() })
    }
}

/// bukkit~: org/bukkit/event/entity/EntityDamageByEntityEvent | a player hitting something: other attackers don't reach this yet
pub struct EntityDamageByEntityEvent {
    pub damager: Player,
    pub entity_id: u64,
    pub kind: String,
    cancelled: bool,
}

impl EntityDamageByEntityEvent {
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.getDamager
    pub fn get_damager(&self) -> &Player {
        &self.damager
    }
    pub fn get_entity_id(&self) -> u64 {
        self.entity_id
    }
    /// bukkit: org/bukkit/event/entity/EntityEvent.getEntityType
    pub fn get_entity_type(&self) -> &str {
        &self.kind
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

impl Event for EntityDamageByEntityEvent {
    const HOOK: &'static str = "attack_entity";
    fn read(value: &Value) -> Option<Self> {
        Some(Self { damager: player_of(value)?, entity_id: value["entity"].as_u64()?, kind: value["kind"].as_str().unwrap_or_default().to_owned(), cancelled: crate::event::prior_cancelled() })
    }
    fn answer(&self) -> Answer {
        if self.cancelled { Answer::Fail } else { Answer::Pass }
    }
}

