//! Written by plugin-tools/tools/sdk_watch.py: do not edit.
//!
//! Events for what an entity's saved data shows changed: they come a tick late, and a cancel writes
//! the old value back.
#![allow(missing_docs, dead_code)]
use crate::data::snbt;
use crate::entity::Entity;
use crate::event::Event;
use crate::World;
use horizon_mod::{Value, query};
use serde_json::json;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

/// A tag that moved.
#[derive(Clone, Debug)]
pub struct Moved {
    pub id: u64,
    pub kind: String,
    pub level: String,
    pub path: String,
    pub old: Value,
    pub new: Value,
}

#[derive(Default)]
struct Watch {
    now: u64,
    /// What listeners asked for: (kind or empty for any, tag path).
    wanted: BTreeSet<(String, String)>,
    seen: HashMap<(u64, String), Value>,
    moved: Arc<Vec<Moved>>,
}

static WATCH: Mutex<Option<Watch>> = Mutex::new(None);

fn tag_true(value: &Value) -> bool {
    value.as_bool().unwrap_or(value.as_i64().unwrap_or(0) != 0)
}

fn at<'a>(data: &'a Value, path: &str) -> &'a Value {
    path.split('.').fold(data, |value, key| &value[key])
}

/// This tick's moved tags, worked out once however many listeners ask.
fn moved(kinds: &[&str], path: &str) -> Arc<Vec<Moved>> {
    let mut all = WATCH.lock().unwrap_or_else(|e| e.into_inner());
    let all = all.get_or_insert_default();
    if kinds.is_empty() {
        all.wanted.insert((String::new(), path.to_owned()));
    }
    for kind in kinds {
        all.wanted.insert(((*kind).to_owned(), path.to_owned()));
    }
    let now = crate::scheduler::now();
    if all.now == now {
        return all.moved.clone();
    }
    all.now = now;
    let mut changes = Vec::new();
    let wanted: Vec<Value> = all.wanted.iter().map(|(kind, path)| json!({ "kind": kind, "path": path })).collect();
    let mut seen = HashMap::new();
    for world in crate::get_worlds() {
        let level = world.dimension().to_owned();
        let rows = query(json!({ "op": "mob_watch", "dimension": level, "wanted": wanted }));
        for row in rows.as_array().into_iter().flatten() {
            let (Some(id), Some(kind)) = (row["id"].as_u64(), row["kind"].as_str()) else {
                continue;
            };
            for (want, path) in &all.wanted {
                if !want.is_empty() && want != kind {
                    continue;
                }
                let value = at(&row["data"], path).clone();
                if value.is_null() {
                    continue;
                }
                let key = (id, path.clone());
                if let Some(before) = all.seen.get(&key)
                    && *before != value
                {
                    changes.push(Moved { id, kind: kind.to_owned(), level: level.clone(), path: path.clone(), old: before.clone(), new: value.clone() });
                }
                seen.insert(key, value);
            }
        }
    }
    all.seen = seen;
    all.moved = Arc::new(changes);
    all.moved.clone()
}

/// A cancelled change: the old value goes back.
fn put_back(moved: &Moved) {
    let mut keys: Vec<&str> = moved.path.split('.').collect();
    let last = keys.pop().unwrap_or_default();
    let mut text = format!("{{{}:{}}}", last, snbt(&moved.old));
    for key in keys.iter().rev() {
        text = format!("{{{key}:{text}}}");
    }
    query(json!({ "op": "entity_merge", "id": moved.id, "snbt": text }));
}

pub use crate::fam_ev_entity::BatToggleSleepEvent;
pub use crate::fam_ev_entity::CreeperPowerEvent;
pub use crate::fam_ev_entity::EnderDragonChangePhaseEvent;
pub use crate::fam_ev_entity::EntityAirChangeEvent;
pub use crate::fam_ev_entity::EntityCombustEvent;
pub use crate::fam_ev_entity::EntityDyeEvent;
pub use crate::fam_ev_entity::EntityEnterLoveModeEvent;
pub use crate::fam_ev_entity::EntityRegainHealthEvent;
pub use crate::fam_ev_entity::EntityToggleSitEvent;
pub use crate::fam_ev_entity::PigZombieAngerEvent;
pub use crate::fam_ev_entity::PufferFishStateChangeEvent;
pub use crate::fam_ev_entity::SheepRegrowWoolEvent;
pub use crate::fam_ev_entity::VillagerAcquireTradeEvent;
pub use crate::fam_ev_entity::VillagerCareerChangeEvent;
