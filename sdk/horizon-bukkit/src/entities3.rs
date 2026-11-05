//! Entity calls that take more than one saved tag, and what a player's client is told apart from
//! the world: hidden entities, the player's own clock and list name.
#![allow(missing_docs)]
use crate::entities::Item;
use crate::entity::Entity;
use crate::util::Vector;
use crate::{Location, Plugin, Player, World, get_online_players};
use horizon_mod::{Value, query};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

impl Item {
    /// bukkit: org/bukkit/entity/Item.setUnlimitedLifetime, org/bukkit/entity/Item.setWillAge
    pub fn set_unlimited_lifetime(&self, unlimited: bool) -> bool {
        self.0.merge(if unlimited { "{Age:-32768s}" } else { "{Age:0s}" })
    }
    /// bukkit: org/bukkit/entity/Item.isUnlimitedLifetime
    pub fn is_unlimited_lifetime(&self) -> bool {
        self.0.data_at("Age").as_i64() == Some(-32768)
    }
    /// bukkit: org/bukkit/entity/Item.willAge
    pub fn will_age(&self) -> bool {
        !self.is_unlimited_lifetime()
    }
    /// bukkit: org/bukkit/entity/Item.setCanPlayerPickup
    pub fn set_can_player_pickup(&self, can: bool) -> bool {
        self.0.merge(if can { "{PickupDelay:0s}" } else { "{PickupDelay:32767s}" })
    }
    /// bukkit: org/bukkit/entity/Item.canPlayerPickup
    pub fn can_player_pickup(&self) -> bool {
        self.0.data_at("PickupDelay").as_i64() != Some(32767)
    }
}

impl Entity {
    /// bukkit: org/bukkit/entity/Vehicle.getVelocity
    pub fn get_vehicle_velocity(&self) -> Vector {
        let motion = self.data_at("Motion");
        let at = |i: usize| motion[i].as_f64().unwrap_or(0.0);
        Vector::new(at(0), at(1), at(2))
    }
    /// bukkit: org/bukkit/entity/Vehicle.setVelocity
    pub fn set_vehicle_velocity(&self, velocity: &Vector) -> bool {
        self.merge(&format!("{{Motion:[{}d,{}d,{}d]}}", velocity.get_x(), velocity.get_y(), velocity.get_z()))
    }
}

// ---- Entity types ----

fn mob_type(id: &str) -> Value {
    query(json!({ "op": "mob_type", "id": id }))
}

/// bukkit: org/bukkit/entity/EntityType.isAlive
pub fn entity_type_is_alive(id: &str) -> bool {
    !mob_type(id).is_null()
}

/// bukkit: org/bukkit/entity/EntityType.getSpawnCategory
pub fn entity_type_spawn_category(id: &str) -> Option<String> {
    mob_type(id)["category"].as_str().map(str::to_ascii_uppercase)
}

/// `org.bukkit.entity.EntitySnapshot`: an entity's saved data, to place later.
#[derive(Clone, Debug, PartialEq)]
pub struct EntitySnapshot(pub String);

/// bukkit: org/bukkit/entity/EntityFactory.createEntitySnapshot, org/bukkit/Bukkit.getEntityFactory, org/bukkit/Server.getEntityFactory
pub fn create_entity_snapshot(snbt: &str) -> EntitySnapshot {
    EntitySnapshot(snbt.to_owned())
}

impl EntitySnapshot {
    /// bukkit: org/bukkit/entity/EntitySnapshot.getAsString
    pub fn get_as_string(&self) -> &str {
        &self.0
    }
    /// bukkit: org/bukkit/entity/EntitySnapshot.createEntity
    pub fn create_entity(&self, world: &World) -> Option<u64> {
        query(json!({ "op": "spawn", "dimension": world.dimension(), "snbt": self.0 }))["id"].as_u64()
    }
    /// bukkit: org/bukkit/entity/EntitySnapshot.getEntityType
    pub fn get_entity_type(&self) -> Option<String> {
        let start = self.0.find("id:")? + 3;
        let rest = self.0[start..].trim_start().trim_start_matches('"');
        rest.split(['"', ',', '}']).next().map(str::to_owned)
    }
}

// ---- Hidden entities ----

/// Who each player has hidden, by entity id: `fam_entity` hands the pairs to the tracker.
static HIDDEN: Mutex<Option<HashMap<String, HashSet<u64>>>> = Mutex::new(None);

/// The entities `player` has hidden, for the tracker.
pub(crate) fn hidden_of(player: &Player) -> Vec<u64> {
    HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(player.uuid())).map_or_else(Vec::new, |ids| ids.iter().copied().collect())
}

impl Player {
    /// bukkit: org/bukkit/entity/Player.hideEntity
    pub fn hide_entity(&self, _plugin: &Plugin, entity_id: u64) {
        let changed = HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().entry(self.uuid().to_owned()).or_default().insert(entity_id);
        crate::fam_entity::hidden_changed();
        if changed {
            crate::fam_ev_player::visibility_changed(self, entity_id, true);
        }
    }
    /// bukkit: org/bukkit/entity/Player.showEntity
    pub fn show_entity(&self, entity_id: u64) {
        let changed = HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().get_mut(self.uuid()).is_some_and(|ids| ids.remove(&entity_id));
        crate::fam_entity::hidden_changed();
        if changed {
            crate::fam_ev_player::visibility_changed(self, entity_id, false);
        }
    }
    /// bukkit: org/bukkit/entity/Player$Spigot.getHiddenPlayers
    pub fn get_hidden(&self) -> Vec<u64> {
        HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(self.uuid())).map_or_else(Vec::new, |ids| ids.iter().copied().collect())
    }
    /// bukkit: org/bukkit/entity/Player.breakBlock
    pub fn break_block(&self, location: &Location) -> bool {
        horizon_mod::run_command(&format!("execute at {} run setblock {} {} {} air destroy", self.get_name(), location.get_block_x(), location.get_block_y(), location.get_block_z()));
        true
    }
    /// bukkit: org/bukkit/entity/Player.setExperienceLevelAndProgress
    pub fn set_experience_level_and_progress(&self, total: i32) {
        horizon_mod::run_command(&format!("xp set {} {total} points", self.get_name()));
    }
}
