// Family ev_entity: answers to the queries its SDK file makes, and what its hooks call.
//
// A hook here that needs more than a row's JSON calls one of these from its payload: the function
// fires the event itself on the row's own hook, with whatever must exist while listeners look at
// it, and returns whether one refused. The row then fires once more with `ee_refused`, which the
// SDK's events answer in `intercept`: that is how a refusal reaches the row's `Break`.
use crate::game::{Game, paper_bridge as bridge, paper_query::stack_json};
use horizon_client::items::ItemStack;
use horizon_client::mobs::{Mob, MobEvent, Who, ee};
use horizon_client::projectiles::{Hit, Projectile, ProjectileKind};
use horizon_client::text::Text;
use horizon_core::geometry::{Aabb, Vec3};
use horizon_core::random::RandomSource;
use crate::game::paper_events::server_living_entity_events as living;
use horizon_mod_runtime as runtime;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once};

fn level(game: &Game<'_>) -> String {
    format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"))
}

fn uuid(player: u128) -> String {
    format!("{player:032x}")
}

/// The player a mob means by `Who::Player`: the guest it is bound to, else whoever is acting.
fn player_of_mob(game: &mut Game<'_>, mob: Option<u32>) -> u128 {
    let bound = mob.and_then(|id| {
        let host = game.lan.as_ref()?;
        let conn = host.mob_players.get(&id)?;
        host.hub.guest(*conn).map(|guest| guest.uuid)
    });
    bound.unwrap_or_else(|| game.native_acting_id())
}

/// Someone as the SDK reads them: `{ "id": 7 }` or `{ "uuid": "..." }`.
fn who_json(game: &mut Game<'_>, who: Who, mob: Option<u32>) -> Value {
    match who {
        Who::Mob(id) => json!({ "id": id }),
        Who::Player => {
            let player = player_of_mob(game, mob);
            let eid = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == player).map(|g| g.entity_id));
            json!({ "uuid": uuid(player), "eid": eid })
        }
    }
}

fn owner_json(game: &mut Game<'_>, p: &Projectile) -> Value {
    if let Some(conn) = p.guest
        && let Some(guest) = game.lan.as_ref().and_then(|host| host.hub.guest(conn))
    {
        return json!({ "uuid": uuid(guest.uuid) });
    }
    match p.owner {
        Some(who) => who_json(game, who, None),
        None => Value::Null,
    }
}

fn stack_opt(stack: Option<&ItemStack>) -> Value {
    stack.map_or(Value::Null, stack_json)
}

/// The projectile is real while listeners look at it, then the game adds it, or doesn't.
pub(in crate::game) fn launch(game: &mut Game<'_>, p: &mut Projectile) -> bool {
    // `age` 0 keeps out what comes back from a save or through a gateway; a client only copies.
    if p.age != 0 || game.remote.is_some() {
        return false;
    }
    if p.id == 0 {
        game.next_entity += 1;
        p.id = game.next_entity;
    }
    let owner = owner_json(game, p);
    let speed = (p.velocity.x * p.velocity.x + p.velocity.y * p.velocity.y + p.velocity.z * p.velocity.z).sqrt();
    let mob_weapon = match p.owner {
        Some(Who::Mob(id)) => game.mobs.iter().find(|m| m.id == id).and_then(|m| m.extra.main_hand_stack().cloned().or_else(|| m.extra.main_hand.as_deref().map(|item| ItemStack::new(item, 1)))),
        _ => None,
    };
    let weapon = p.arrow.as_ref().and_then(|arrow| arrow.fired_from_weapon.as_ref()).or(mob_weapon.as_ref());
    let offhand = mob_weapon.is_none() && weapon.is_some_and(|weapon| game.held().is_none_or(|held| held.item != weapon.item));
    // Only a bow draws at the shot: a crossbow's arrows left the inventory as it was loaded.
    let by_bow = mob_weapon.is_none() && weapon.is_some_and(|weapon| &*weapon.item == "minecraft:bow");
    let free = !by_bow || p.arrow.as_ref().is_none_or(|arrow| arrow.pickup != horizon_client::arrows::Pickup::Allowed);
    let force = take_force().or_else(|| match p.owner {
        Some(Who::Mob(id)) => ee::force_of(id),
        _ => None,
    });
    let payload = json!({
        "level": level(game),
        "entity": p.id,
        "kind": p.kind.id(),
        "arrow": p.is_arrow(),
        "owner": owner,
        "at": [p.position.x, p.position.y, p.position.z],
        "speed": speed,
        "weapon": stack_opt(weapon),
        "ammo": stack_opt(p.arrow.as_ref().map(|arrow| &arrow.pickup_item)),
        "consumes": !free,
        "hand": if offhand { "OFF_HAND" } else { "HAND" },
        "force": force,
    });
    game.creatures.projectiles.push(p.clone());
    let refused = bridge::on_refusable(game, "ee_launch", payload);
    let id = p.id;
    game.creatures.projectiles.retain(|other| other.id != id);
    refused
}

/// A cancelled shot's bow doesn't wear: `release_bow` does that last.
static SKIP_WEAR: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn skip_wear() -> bool {
    SKIP_WEAR.swap(false, Ordering::Relaxed)
}

pub(in crate::game) fn face_name(face: [i32; 3]) -> &'static str {
    match face {
        [0, 1, 0] => "UP",
        [0, -1, 0] => "DOWN",
        [0, 0, -1] => "NORTH",
        [0, 0, 1] => "SOUTH",
        [-1, 0, 0] => "WEST",
        [1, 0, 0] => "EAST",
        _ => "SELF",
    }
}

/// What the projectile struck, with the projectile back in the world while listeners look: a
/// flying one has left the list by now, one stuck in a block is still on it.
/// A refused hit: the projectile goes on through, unless it has stayed in the game by sticking.
pub(in crate::game) fn fly_on(game: &mut Game<'_>, p: &Projectile) {
    if !game.creatures.projectiles.iter().any(|other| other.id == p.id) {
        let mut p = p.clone();
        p.previous = p.position;
        p.position = p.position.add_vector(p.velocity);
        game.creatures.projectiles.push(p);
    }
}

pub(in crate::game) fn projectile_hit(game: &mut Game<'_>, p: &Projectile, hit: Hit) -> bool {
    let mut payload = json!({ "level": level(game), "entity": p.id, "kind": p.kind.id(), "shooter": owner_json(game, p) });
    match hit {
        Hit::None => return false,
        Hit::Block { pos, face, at } => {
            payload["block"] = json!(pos);
            payload["face"] = json!(face_name(face));
            payload["at"] = json!([at.x, at.y, at.z]);
        }
        Hit::Entity { who, at } => {
            payload["hit"] = who_json(game, who, None);
            payload["at"] = json!([at.x, at.y, at.z]);
        }
    }
    let lent = !game.creatures.projectiles.iter().any(|other| other.id == p.id);
    if lent {
        game.creatures.projectiles.push(p.clone());
    }
    let refused = bridge::on_refusable(game, "ee_hit", payload);
    if lent {
        let id = p.id;
        game.creatures.projectiles.retain(|other| other.id != id);
    }
    refused
}

fn mob_state(game: &mut Game<'_>, dimension: &str) -> Value {
    game.native_in_level(dimension, |game| {
        let rows: Vec<(Value, Option<Who>, Option<Who>)> = game
            .mobs
            .iter()
            .filter(|m| !m.removed && m.alive())
            .map(|m| (json!({
                "id": m.id,
                "kind": m.kind.id,
                "at": [m.position.x, m.position.y, m.position.z],
                "yaw": m.yaw,
                "pitch": m.head_pitch,
                "mount": m.extra.mount,
                "gliding": m.extra.fall_flying,
                "using": m.extra.using_item,
                "health": m.health,
                "effects": m.extra.effects,
                "wet": m.fluids.eye_in_water,
                "nav": m.ai.nav.target_pos,
                "rider": m.rider.is_some(),
                "warden": m.extra.warden.as_ref().map(|warden| warden.anger.active_anger(m.ai.target)),
                "fire": m.fire_ticks,
                "flag": m.extra.flag,
                "flag2": m.extra.flag2,
                "counter2": m.extra.counter2,
                "spell": m.extra.variant2,
                "rising": m.velocity.y,
                "swell": m.swell,
                "ground": m.on_ground,
                "fall": m.fall_distance,
                "anger": m.extra.anger,
                "sitting": m.extra.sitting,
                "held": m.extra.main_hand.as_deref(),
            }), m.ai.target, m.ai.last_hurt_by))
            .collect();
        let mut out = Vec::with_capacity(rows.len());
        for (mut row, target, hurt_by) in rows {
            let id = row["id"].as_u64().map(|id| id as u32);
            row["target"] = target.map_or(Value::Null, |who| who_json(game, who, id));
            row["hurt_by"] = hurt_by.map_or(Value::Null, |who| who_json(game, who, id));
            out.push(row);
        }
        Value::Array(out)
    })
    .unwrap_or(Value::Null)
}

/// How a mob's id reads: `spawn_mob` takes it with or without the namespace.
pub(in crate::game) fn mob_id(kind: &str) -> String {
    if kind.contains(':') { kind.to_owned() } else { format!("minecraft:{kind}") }
}

pub(in crate::game) fn here(game: &Game<'_>) -> String {
    level(game)
}

pub(in crate::game) fn fight_who(game: &mut Game<'_>, who: crate::game::dragon::FightWho) -> Value {
    use crate::game::dragon::FightWho;
    match who {
        FightWho::Host => json!({ "uuid": uuid(game.native_acting_id()) }),
        FightWho::Guest(conn) => game.lan.as_ref().and_then(|host| host.hub.guest(conn)).map_or(Value::Null, |guest| json!({ "uuid": uuid(guest.uuid) })),
    }
}

/// `PotionSplashEvent` and its kin as a thrown potion breaks. An experience bottle's whole hit is
/// done here, as what it gives is rolled inside the game's: true then skips the game's.
pub(in crate::game) fn splash(game: &mut Game<'_>, stack: &ItemStack, at: Vec3, mob: Option<Vec3>, age: u32, owner: &Option<Text>) -> bool {
    let thrower = owner.as_ref().map(Text::plain);
    let mut payload = json!({ "level": level(game), "stack": stack_json(stack), "at": [at.x, at.y, at.z], "owner": thrower, "entity": dying_id() });
    if &*stack.item == "minecraft:experience_bottle" {
        return false;
    }
    payload["kind"] = json!(if &*stack.item == "minecraft:lingering_potion" { "lingering" } else { "splash" });
    let lingering = &*stack.item == "minecraft:lingering_potion";
    let cloud = lingering.then(|| crate::game::Game::lingering_cloud(stack, mob.unwrap_or(at), owner.clone())).flatten().map(|mut cloud| {
        cloud.id = fresh_id(game);
        let id = cloud.id;
        game.breath_clouds.push(cloud);
        id
    });
    payload["cloud"] = json!(cloud);
    payload["water"] = json!(stack.potion() == Some("minecraft:water"));
    // `splash_distance_sqr`: the potion's box and what reaches it.
    let potion = Aabb::new(Vec3::new(at.x - 0.125, at.y, at.z - 0.125), Vec3::new(at.x + 0.125, at.y + 0.25, at.z + 0.125));
    let margin = ((f64::from(age) - 2.0) / 20.0).clamp(0.0, 0.3);
    let intensity = |bounds: Aabb| {
        if !potion.inflate(Vec3::new(4.0, 2.0, 4.0)).intersects(bounds) {
            return None;
        }
        let d2 = potion.distance_squared_to_box(bounds.inflate(Vec3::new(margin, margin, margin)));
        (d2 < 16.0).then(|| 1.0 - d2.sqrt() / 4.0)
    };
    let mut affected = Vec::new();
    for player in game.fight_players().into_iter().filter(|player| player.alive) {
        if let Some(strength) = intensity(player.bounds) {
            affected.push(json!({ "who": fight_who(game, player.who), "intensity": strength }));
        }
    }
    for mob in game.mobs.iter().filter(|mob| mob.alive()) {
        if let Some(strength) = intensity(mob.bounds()) {
            let sensitive = matches!(&*mob.kind.id, "minecraft:blaze" | "minecraft:enderman" | "minecraft:snow_golem" | "minecraft:strider");
            affected.push(json!({ "who": { "id": mob.id }, "intensity": strength, "kind": mob.kind.id, "fire": mob.fire_ticks, "sensitive": sensitive }));
        }
    }
    payload["affected"] = Value::Array(affected);
    INTENSITIES.lock().unwrap_or_else(|e| e.into_inner()).clear();
    note_cause(game, "POTION_SPLASH");
    if let Some(thrower) = &thrower {
        let row = KNOWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|known| known.values().find(|row| row["name"] == thrower.as_str()).cloned());
        *SOURCE.lock().unwrap_or_else(|e| e.into_inner()) = (row.unwrap_or(Value::Null), game.world.game_time as i64);
    }
    let refused = bridge::on_refusable(game, "ee_splash", payload);
    if let Some(cloud) = cloud {
        if refused {
            game.breath_clouds.retain(|held| held.id != cloud);
        } else {
            CLOUD_OUT.store(true, Ordering::Relaxed);
        }
    }
    refused
}

/// `ExpBottleEvent`: what it gives, and whether it shows.
pub(in crate::game) fn bottle(game: &mut Game<'_>, stack: &ItemStack, at: Vec3, exp: i32) -> (i32, bool) {
    if !bridge::hears("ee_splash") {
        return (exp, true);
    }
    let payload = json!({ "level": level(game), "stack": stack_json(stack), "at": [at.x, at.y, at.z], "kind": "exp", "exp": exp, "shows": true, "entity": dying_id() });
    let (_, after) = fire_lent(game, "ee_splash", payload);
    (after["exp"].as_i64().map_or(exp, |exp| exp as i32), after["shows"] != false)
}

/// What a listener of `PotionSplashEvent` set, by who.
static INTENSITIES: Mutex<Vec<(Value, f64)>> = Mutex::new(Vec::new());
/// `getToDamage`'s other way: one the game wouldn't hurt, that a listener says water hurts.
static WET: Mutex<Vec<u32>> = Mutex::new(Vec::new());

/// What a listener of `PotionSplashEvent` set for one in the splash: `None` leaves it out.
fn strength_of(who: &Value, own: Option<f64>) -> Option<f64> {
    let intensities = INTENSITIES.lock().unwrap_or_else(|e| e.into_inner());
    let set = intensities.iter().find(|(row, _)| row == who).map(|(_, strength)| *strength);
    set.or(own).filter(|strength| *strength > 0.0)
}

/// `splash`'s own loop over the mobs.
pub(in crate::game) fn mob_strength(id: u32, own: Option<f64>) -> Option<f64> {
    strength_of(&json!({ "id": id }), own)
}

/// `splash_on_player`, for the player in hand.
pub(in crate::game) fn player_strength(game: &mut Game<'_>, own: Option<f64>) -> Option<f64> {
    strength_of(&json!({ "uuid": uuid(game.native_acting_id()) }), own)
}

/// Those a listener says water hurts, after the game has done its own.
pub(in crate::game) fn wet_after(game: &mut Game<'_>) {
    for id in std::mem::take(&mut *WET.lock().unwrap_or_else(|e| e.into_inner())) {
        // `applyWater`: `indirectMagic`, asked of the listeners like any hit.
        let source = horizon_client::mobs::Damage::of(horizon_client::mobs::DamageKind::Magic);
        if !crate::game::paper_entity_events::hit_asked(game, id, 1.0, source) {
            continue;
        }
        if let Some(m) = game.mobs.iter_mut().find(|m| m.id == id) {
            let at = [m.position.x, m.position.y, m.position.z];
            m.hurt(1.0, Some(at), &mut game.world.random);
        }
    }
}

/// The world's own spawning: one event a mob, a refused one left out by running the game's own
/// `add_spawned` again over the rest.
static KEPT: Mutex<Vec<usize>> = Mutex::new(Vec::new());
static AGAIN: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn natural(game: &mut Game<'_>, pending: &[Mob]) -> bool {
    if AGAIN.swap(false, Ordering::Relaxed) || !bridge::hears("ee_pre_spawn") {
        return false;
    }
    let mut kept = Vec::with_capacity(pending.len());
    for (i, mob) in pending.iter().enumerate() {
        let payload = json!({ "level": level(game), "kind": mob.kind.id, "at": [mob.position.x, mob.position.y, mob.position.z], "natural": true });
        if !bridge::on_refusable(game, "ee_pre_spawn", payload) {
            kept.push(i);
        }
        if ee::take_abort() {
            kept.clear();
            break;
        }
    }
    let refused = kept.len() != pending.len();
    *KEPT.lock().unwrap_or_else(|e| e.into_inner()) = kept;
    refused
}

pub(in crate::game) fn kept(pending: &[Mob]) -> Vec<Mob> {
    AGAIN.store(true, Ordering::Relaxed);
    KEPT.lock().unwrap_or_else(|e| e.into_inner()).iter().filter_map(|i| pending.get(*i).cloned()).collect()
}

/// The entity counter as a mob's death starts: what's newer when its loot is down is its loot.
static DEATH_MARK: Mutex<u32> = Mutex::new(0);

pub(in crate::game) fn mark_death(game: &Game<'_>) -> u32 {
    *DEATH_MARK.lock().unwrap_or_else(|e| e.into_inner()) = game.next_entity;
    DEATH_WINDOW.store(bridge::hears("ee_death"), Ordering::Relaxed);
    game.next_entity
}

/// From a mob's death to its `EntityDeathEvent`: Paper holds the loot back that long, so
/// nobody hears of an item that the event may still take away.
static DEATH_WINDOW: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn death_window() -> bool {
    DEATH_WINDOW.load(Ordering::Relaxed)
}

/// Whether `EntityDeathEvent` cancelled this mob's death.
pub(in crate::game) fn was_revived(id: u32) -> bool {
    REVIVED.load(Ordering::Relaxed) == id
}

/// The loot the event left, joining the level as Paper adds it after the event.
fn death_items(game: &mut Game<'_>, mark: u32) {
    DEATH_WINDOW.store(false, Ordering::Relaxed);
    let items: Vec<(u32, [f64; 3])> = game.dropped.iter().filter(|item| item.id > mark).map(|item| (item.id, [item.position.x, item.position.y, item.position.z])).collect();
    for (id, at) in items {
        let refused = bridge::hears("ee_item") && bridge::on_refusable(game, "ee_item", json!({ "level": level(game), "entity": id, "at": at }));
        if refused || !game.native_entity_joins(id, "minecraft:item", at, false) {
            game.dropped.retain(|item| item.id != id);
        }
    }
}

/// What `MobEvent::SpawnKind` is carrying out, by what it makes.
pub(in crate::game) fn spawn_kind_why(kind: &str) -> &'static str {
    match kind {
        "minecraft:zombified_piglin" | "minecraft:witch" => "LIGHTNING",
        "minecraft:vex" => "SPELL",
        "minecraft:cow" => "SHEARED",
        _ => "DEFAULT",
    }
}

/// `CarvedPumpkinBlock` and `WitherSkullBlock`, by what stands up.
pub(in crate::game) fn built_why(kind: &str) -> &'static str {
    match kind.trim_start_matches("minecraft:") {
        "iron_golem" => "BUILD_IRONGOLEM",
        "snow_golem" => "BUILD_SNOWMAN",
        "copper_golem" => "BUILD_COPPERGOLEM",
        "wither" => "BUILD_WITHER",
        _ => "DEFAULT",
    }
}

/// `EntityBreakEvent` for an item frame, broken by a player's blow or for want of support.
pub(in crate::game) fn frame_refused(game: &mut Game<'_>, index: usize, cause: &str) -> bool {
    if !bridge::hears("ee_hanging") {
        return false;
    }
    let id = game.item_frames[index].id;
    let kind = if cause == "ENTITY" { "entity" } else { "other" };
    bridge::on_refusable(game, "ee_hanging", json!({ "level": level(game), "entity": id, "kind": kind, "cause": cause }))
}

/// The boat and the mob it pushes.
pub(in crate::game) fn boat_rows(game: &mut Game<'_>, boat: usize, mob: usize) -> [Value; 2] {
    let (boat, mob) = (game.boats[boat].id, game.mobs[mob].id);
    [row_for(game, boat), row_for(game, mob)]
}

/// `EntityCollideWithEntityEvent` for two minecarts: true if refused.
pub(in crate::game) fn carts_collide(game: &mut Game<'_>, i: usize, j: usize) -> bool {
    if !bridge::hears("ee_collide") {
        return false;
    }
    let (one, two) = (game.carts[i].id, game.carts[j].id);
    let entities = [row_for(game, one), row_for(game, two)];
    bridge::on_refusable(game, "ee_collide", json!({ "entities": entities }))
}

/// `EntityPlaceEvent`, with what's been placed standing: true if refused, and the caller takes it away.
pub(in crate::game) fn placed(game: &mut Game<'_>, id: u32, stack: &ItemStack, hit: Option<([i32; 3], [i32; 3])>) -> bool {
    if !bridge::hears("ee_place") {
        return false;
    }
    let payload = json!({ "level": level(game), "entity": id, "stack": stack_json(stack), "pos": hit.map(|hit| hit.0), "face": hit.map(|hit| face_name(hit.1)), "hand": hand_of(game, stack) });
    bridge::on_refusable(game, "ee_place", payload)
}

/// `TameableDeathMessageEvent`: only for a pet whose owner hears of it, with the rule on.
pub(in crate::game) fn pet_message_refused(game: &mut Game<'_>, m: &Mob) -> bool {
    if !bridge::hears("ee_pet_died") {
        return false;
    }
    let owner = owner_of(m);
    bridge::on_refusable(game, "ee_pet_died", json!({ "level": level(game), "entity": m.id, "owner": owner }))
}

/// `LivingEntity.setArrowCount` for a mob, which keeps no count of its own: (count, ticks to the next falling out).
static MOB_ARROWS: Mutex<Option<HashMap<u32, (i32, i32)>>> = Mutex::new(None);

fn arrow_count(game: &mut Game<'_>, id: u32, old: i32, new: i32) -> i32 {
    let entity = row_for(game, id);
    let (refused, after) = fire_lent(game, "ee_arrows", json!({ "entity": entity, "old": old, "new": new, "reset": false }));
    if refused { old } else { after["new"].as_i64().map_or(new, |new| new as i32) }
}

fn set_mob_arrows(id: u32, count: i32) {
    let mut all = MOB_ARROWS.lock().unwrap_or_else(|e| e.into_inner());
    let all = all.get_or_insert_default();
    if count > 0 {
        all.insert(id, (count, 20 * (30 - count)));
    } else {
        all.remove(&id);
    }
}

/// An arrow that struck a mob stays in it.
pub(in crate::game) fn arrow_stuck(game: &mut Game<'_>, p: &Projectile, who: Who) {
    let Who::Mob(id) = who else {
        return;
    };
    if !p.is_arrow() || !bridge::hears("ee_arrows") {
        return;
    }
    let old = mob_arrow_row(id).0;
    let count = arrow_count(game, id, old, old + 1);
    set_mob_arrows(id, count);
}

fn mob_arrow_row(id: u32) -> (i32, i32) {
    MOB_ARROWS.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|all| all.get(&id).copied()).unwrap_or((0, 0))
}

/// `LivingEntity.tick`: one falls out every `20 * (30 - count)` ticks.
fn arrows_fall(game: &mut Game<'_>) {
    let due: Vec<(u32, i32)> = {
        let mut all = MOB_ARROWS.lock().unwrap_or_else(|e| e.into_inner());
        let Some(all) = all.as_mut() else {
            return;
        };
        all.iter_mut().filter_map(|(id, row)| {
            row.1 -= 1;
            (row.1 <= 0).then_some((*id, row.0))
        }).collect()
    };
    for (id, count) in due {
        if !game.mobs.iter().any(|m| m.id == id) {
            set_mob_arrows(id, 0);
            continue;
        }
        let left = arrow_count(game, id, count, count - 1);
        set_mob_arrows(id, left);
    }
}

/// What each mob last wore, by slot: `LivingEntity.collectEquipmentChanges` looks each tick.
static WORN: Mutex<Option<HashMap<u32, Vec<Option<ItemStack>>>>> = Mutex::new(None);

/// `EntityEquipmentChangedEvent` for the mobs: every slot that changed since the last tick, together.
fn mobs_change_equipment(game: &mut Game<'_>) {
    use horizon_world::player_inventory::EquipmentSlot as Slot;
    const SLOTS: [(Slot, &str); 7] = [(Slot::MainHand, "HAND"), (Slot::OffHand, "OFF_HAND"), (Slot::Feet, "FEET"), (Slot::Legs, "LEGS"), (Slot::Chest, "CHEST"), (Slot::Head, "HEAD"), (Slot::Body, "BODY")];
    if !bridge::hears("ee_equipment") {
        return;
    }
    let held = WORN.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or_default();
    let mut worn = HashMap::with_capacity(game.mobs.len());
    let mut changed = Vec::new();
    for m in game.mobs.iter().filter(|m| m.alive()) {
        let now: Vec<Option<ItemStack>> = SLOTS.iter().map(|(slot, _)| m.item_by_slot(*slot)).collect();
        let was = held.get(&m.id);
        let rows: Vec<Value> = SLOTS.iter().enumerate().filter_map(|(i, (_, name))| {
            let old = was.and_then(|was| was[i].as_ref());
            (old != now[i].as_ref()).then(|| json!({ "slot": name, "old": stack_opt(old), "new": stack_opt(now[i].as_ref()) }))
        }).collect();
        if !rows.is_empty() {
            changed.push((m.id, rows));
        }
        worn.insert(m.id, now);
    }
    *WORN.lock().unwrap_or_else(|e| e.into_inner()) = Some(worn);
    for (id, rows) in changed {
        let entity = row_for(game, id);
        bridge::on_trigger(game, "ee_equipment", json!({ "entity": entity, "changes": rows }));
    }
}

/// `EntityInsideBlockEvent` for items: they tick apart from the mobs. What the block does to one isn't stopped.
fn items_inside(game: &mut Game<'_>) {
    if !bridge::hears("ee_inside") {
        return;
    }
    let inside: Vec<(u32, [i32; 3], String)> = game.dropped.iter().filter_map(|item| {
        let at = [item.position.x.floor() as i32, item.position.y.floor() as i32, item.position.z.floor() as i32];
        let name = game.res.facts().get(game.world.block(at[0], at[1], at[2])).name.to_string();
        ee::acts_on_entities(&name).then_some((item.id, at, name))
    }).collect();
    for (id, at, name) in inside {
        let entity = row_for(game, id);
        let block = json!({ "dimension": entity["dimension"], "pos": at, "name": name });
        bridge::on_trigger(game, "ee_inside", json!({ "entity": entity, "block": block }));
    }
}

/// The block just come loose: a refusal puts it back, as `FallingBlockEntity.fall` leaves it.
pub(in crate::game) fn falling_starts(game: &mut Game<'_>, pos: [i32; 3], state: horizon_world::registry::BlockStateId) {
    if !bridge::hears("ee_mob") {
        return;
    }
    let id = fresh_id(game);
    let Some(falling) = game.falling.last_mut() else {
        return;
    };
    falling.id = id;
    let name = game.res.facts().get(state).name.to_string();
    let here = level(game);
    let entity = json!({ "id": id, "dimension": here, "uuid": "" });
    if bridge::on_refusable(game, "ee_mob", json!({ "kind": "change_block", "level": here, "pos": pos, "old": name, "new": "minecraft:air", "entity": entity })) {
        game.falling.retain(|falling| falling.id != id);
        game.native_set_block(game.dim.id(), pos, state, 3);
    }
}

/// It's about to be a block again: true if refused, and it's gone.
pub(in crate::game) fn falling_lands(game: &mut Game<'_>, id: &mut u32, state: horizon_world::registry::BlockStateId, pos: [i32; 3]) -> bool {
    if !bridge::hears("ee_mob") {
        return false;
    }
    if *id == 0 {
        *id = fresh_id(game);
    }
    let name = game.res.facts().get(state).name.to_string();
    let old = game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name.to_string();
    let here = level(game);
    let at = [f64::from(pos[0]) + 0.5, f64::from(pos[1]), f64::from(pos[2]) + 0.5];
    *DYING.lock().unwrap_or_else(|e| e.into_inner()) = Some((*id, "minecraft:falling_block".to_owned(), at, json!({})));
    let entity = json!({ "id": *id, "dimension": here, "uuid": "" });
    let refused = bridge::on_refusable(game, "ee_mob", json!({ "kind": "change_block", "level": here, "pos": pos, "old": old, "new": name, "entity": entity }));
    dying_over();
    refused
}

/// `ElderGuardianAppearanceEvent`, for a player the game has found in reach.
pub(in crate::game) fn elder_refused(game: &mut Game<'_>, at: [f64; 3]) -> bool {
    if !bridge::hears("ee_elder") {
        return false;
    }
    let entity = nearest(game, "minecraft:elder_guardian", at);
    bridge::on_refusable(game, "ee_elder", json!({ "level": level(game), "at": at, "entity": entity }))
}

/// `PhantomPreSpawnEvent`, past the game's odds.
pub(in crate::game) fn phantom_refused(game: &mut Game<'_>) -> bool {
    bridge::hears("ee_phantom") && bridge::on_refusable(game, "ee_phantom", json!({ "level": level(game) }))
}

/// `EntityDeathEvent`: at the first call after `drop_death_loot`, so its loot and orbs exist.
pub(in crate::game) fn death(game: &mut Game<'_>, m: &Mob) -> bool {
    let mark = *DEATH_MARK.lock().unwrap_or_else(|e| e.into_inner());
    let items: Vec<Value> = game.dropped.iter().filter(|item| item.id > mark).map(|item| json!({ "id": item.id, "stack": stack_json(&item.stack) })).collect();
    let orbs: Vec<u32> = game.orbs.iter().filter(|orb| orb.id > mark).map(|orb| orb.id).collect();
    let exp: u64 = game.orbs.iter().filter(|orb| orb.id > mark).map(|orb| u64::from(orb.value) * u64::from(orb.count.max(1))).sum();
    let damage = m.last_damage.map(|(damage, _)| damage);
    let attacker = damage.and_then(|damage| damage.attacker).map_or(Value::Null, |who| who_json(game, who, Some(m.id)));
    let voice = ee::take_voice(m.id);
    let (sound, volume, pitch) = voice.unwrap_or((horizon_client::mobs::ai::voice_sounds(m)[2], horizon_client::mobs::ai::sound_volume(m), 1.0));
    let category = if m.species() == horizon_client::mobs::Species::Bat { "AMBIENT" } else if m.hostile() { "HOSTILE" } else { "NEUTRAL" };
    let payload = json!({
        "level": level(game),
        "entity": m.id,
        "kind": m.kind.id,
        "at": [m.position.x, m.position.y + 0.5, m.position.z],
        "items": items,
        "orbs": orbs,
        "exp": exp,
        "attacker": attacker,
        "damage_type": damage.map(crate::game::creatures::source_damage_type),
        "from": damage.and_then(|damage| damage.from),
        "sound": format!("minecraft:{sound}"),
        "volume": volume,
        "voice_pitch": pitch,
        "play": voice.is_some(),
        "category": category,
    });
    let (refused, after) = fire_lent(game, "ee_death", payload);
    REVIVED.store(if refused { m.id } else { 0 }, Ordering::Relaxed);
    death_items(game, mark);
    if !refused && after["play"] == true && let Some(sound) = after["sound"].as_str() {
        let at = [m.position.x, m.position.y, m.position.z];
        let volume = after["volume"].as_f64().unwrap_or(1.0) as f32;
        let pitch = after["voice_pitch"].as_f64().unwrap_or(1.0) as f32;
        let source = after["category"].as_str().and_then(sound_source);
        if after["category"] != category && let Some(source) = source {
            game.audio.play(sound, Some(at), volume, pitch);
            let dimension = game.dim.id();
            game.creatures.lan_sounds.push(crate::game::creatures::NativeSound { dimension, sound: sound.to_owned(), at, source, volume, pitch });
        } else if let Err(e) = game.handle_mob_events(vec![MobEvent::SoundId(sound.to_owned(), at, volume, pitch)]) {
            log::warn!("a death's sound: {e}");
        }
    }
    refused
}

/// `SoundSource`'s number, which is the order of Bukkit's `SoundCategory`.
fn sound_source(name: &str) -> Option<i32> {
    ["MASTER", "MUSIC", "RECORDS", "WEATHER", "BLOCKS", "HOSTILE", "NEUTRAL", "PLAYERS", "AMBIENT", "VOICE", "UI"].iter().position(|row| *row == name).map(|at| at as i32)
}

/// The mob a listener of `EntityDeathEvent` just brought back: it doesn't split.
static REVIVED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn death_apply(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let ids = |key: &str| -> Vec<u32> { query[key].as_array().into_iter().flatten().filter_map(|row| row["id"].as_u64().or(row.as_u64())).map(|id| id as u32).collect() };
    let at = [query["at"][0].as_f64()?, query["at"][1].as_f64()?, query["at"][2].as_f64()?];
    if let Some(stacks) = query["items"].as_array() {
        let gone = ids("item_ids");
        game.dropped.retain(|item| !gone.contains(&item.id));
        for stack in stacks.iter().filter_map(stack_of) {
            game.spawn_item(stack, at, crate::game::dispenser::new_item_motion(), 10);
        }
    }
    if let Some(exp) = query["exp"].as_u64() {
        let gone = ids("orb_ids");
        game.orbs.retain(|orb| !gone.contains(&orb.id));
        game.spawn_orbs(at, u32::try_from(exp).unwrap_or(u32::MAX));
    }
    if let (Some(health), Some(id)) = (query["revive"].as_f64(), query["id"].as_u64()) {
        let id = id as u32;
        game.died.remove(&id);
        if let Some(m) = game.mobs.iter_mut().find(|m| m.id == id) {
            m.health = (health as f32).min(m.attrs.max_health);
            m.death_time = 0;
            m.needs_sync = true;
        }
    }
    Some(json!(true))
}

/// What a listener of `PlayerDeathEvent` asked of the death it's hearing.
#[derive(Default)]
struct DeathAsked {
    keep: bool,
    no_exp: bool,
    drops: Option<Vec<ItemStack>>,
    revive: Option<f32>,
}

static DEATH_ASKED: Mutex<Option<DeathAsked>> = Mutex::new(None);
/// Death screens a listener rewrote, sent once the game's own has gone.
static SCREENS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
/// `keep_inventory` is on for the rest of a death whose listener asked for it.
static KEEP_RULE: AtomicBool = AtomicBool::new(false);

fn set_keep_rule(game: &mut Game<'_>, on: bool) {
    game.ui.rules.set("keep_inventory", horizon_client::gamerules::Value::Bool(on));
}

/// `PlayerDeathEvent` for a server's player: the head of `guest_die`, which runs however they are,
/// so only for one falling now. `game.lan` is out of the game here, so listeners can't ask it of
/// any player, and all they need comes in the event. True stops the death.
pub(in crate::game) fn player_dying(game: &mut Game<'_>, host: &mut crate::game::lan::LanHost, gi: usize, cause: Option<String>) -> bool {
    let g = &host.hub.guests[gi];
    if g.health > 0.0 || g.dead {
        return false;
    }
    let mut drops: Vec<Value> = g.inventory.iter().flatten().map(stack_json).collect();
    drops.extend(g.armor.iter().rev().flatten().map(stack_json));
    drops.extend(g.offhand.iter().map(stack_json));
    let keep = game.ui.rules.bool("keep_inventory");
    let at = g.position;
    let eyes = [at[0], at[1] + g.eye_height() - 0.3, at[2]];
    let payload = json!({
        "ee_dying": true,
        "uuid": uuid(g.uuid),
        "name": g.name,
        "level": g.dimension,
        "at": at,
        "cause": cause,
        "drops": if keep { Vec::new() } else { drops },
        "keep": keep,
        "show": game.ui.rules.bool("show_death_messages"),
        "xp_level": g.xp_level,
        "xp_progress": g.xp_progress,
    });
    *DEATH_ASKED.lock().unwrap_or_else(|e| e.into_inner()) = None;
    bridge::on_refusable(game, "ee_player_dying", payload);
    let Some(asked) = DEATH_ASKED.lock().unwrap_or_else(|e| e.into_inner()).take() else {
        return false;
    };
    let g = &mut host.hub.guests[gi];
    if let Some(health) = asked.revive {
        g.health = health;
        g.sent_health = None;
        return true;
    }
    if asked.no_exp {
        (g.xp_level, g.xp_progress, g.xp_total) = (0, 0.0, 0);
    }
    let mut keep_rule = asked.keep;
    let reward = horizon_client::entities::Experience { level: g.xp_level, progress: g.xp_progress, total: g.xp_total }.death_reward(false);
    if let Some(stacks) = asked.drops {
        // What falls is what the listener left: the game's own fall is off, and its slots with it.
        g.inventory.iter_mut().for_each(|slot| *slot = None);
        g.armor.iter_mut().for_each(|slot| *slot = None);
        g.offhand = None;
        host.hub.send_inventory(gi);
        for stack in stacks {
            game.spawn_item(stack, eyes, crate::game::dispenser::new_item_motion(), 40);
        }
        keep_rule = true;
    }
    if keep_rule && !keep {
        set_keep_rule(game, true);
        KEEP_RULE.store(true, Ordering::Relaxed);
        // The rule takes the experience with it, which this death doesn't ask for.
        if !asked.no_exp {
            game.spawn_orbs(at, reward);
        }
    }
    false
}

pub(in crate::game) fn breeder(game: &mut Game<'_>, parent: u32, partner: u32) -> Value {
    match game.love_cause.get(&parent).or(game.love_cause.get(&partner)).copied() {
        Some(Some(conn)) => game.lan.as_ref().and_then(|host| host.hub.guest(conn)).map_or(Value::Null, |guest| json!({ "uuid": uuid(guest.uuid) })),
        Some(None) => json!({ "uuid": uuid(game.native_acting_id()) }),
        None => Value::Null,
    }
}

pub(in crate::game) fn lays_eggs(game: &Game<'_>, parent: u32) -> bool {
    game.mobs.iter().any(|m| m.id == parent && matches!(&*m.kind.id, "minecraft:turtle" | "minecraft:frog" | "minecraft:sniffer"))
}

/// The doors a mob is cracking, by where: `BreakDoor` doesn't name the mob.
static CRACKING: Mutex<Vec<([i32; 3], u32)>> = Mutex::new(Vec::new());
static REFUSED: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// What a mob's tick asks of the game, before it's done. A refused one goes by running the game's
/// own `handle_mob_events` again over the rest: `events` can't be changed where it is.
/// `EntityChangeBlockEvent` for what a mob's tick asks of a block: true if a listener refused.
fn block_refused(game: &mut Game<'_>, event: &MobEvent, actor: Option<u32>) -> bool {
    use horizon_world::registry::BlockStateId;
    let registry = game.pack.assets.registry;
    let (pos, state, mob): ([i32; 3], Option<BlockStateId>, Option<u32>) = match event {
        MobEvent::SetBlock { pos, state } => (*pos, Some(*state), actor),
        MobEvent::SetBlockName { pos, name, .. } => (*pos, registry.default_state(name), actor),
        MobEvent::BreakBlock { pos, .. } => (*pos, Some(game.world.air), actor),
        MobEvent::CropRaided { pos, state, mob, .. } => (*pos, Some(*state), Some(*mob)),
        MobEvent::Infest { pos, infested, .. } => {
            let old = game.world.block(pos[0], pos[1], pos[2]);
            let (from, to) = if *infested { ("minecraft:", "minecraft:infested_") } else { ("minecraft:infested_", "minecraft:") };
            let state = registry.state(old).cloned().and_then(|mut state| {
                state.name = state.name.replacen(from, to, 1);
                registry.resolve(&state).ok()
            });
            (*pos, state, actor)
        }
        _ => return false,
    };
    let (Some(state), Some(mob)) = (state, mob) else {
        return false;
    };
    let Some(m) = game.mobs.iter().find(|m| m.id == mob) else {
        return false;
    };
    let kind = m.kind.id.to_string();
    // A snow golem's trail is `EntityBlockFormEvent`.
    if kind == "minecraft:snow_golem" {
        return false;
    }
    let text = |id: BlockStateId| registry.state(id).map_or_else(String::new, |state| {
        let mut parts: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
        parts.sort();
        if parts.is_empty() { state.name.clone() } else { format!("{}[{}]", state.name, parts.join(",")) }
    });
    let (old, new) = (text(game.world.block(pos[0], pos[1], pos[2])), text(state));
    let mut entity = row_for(game, mob);
    entity["kind"] = json!(kind);
    let refused = bridge::on_refusable(game, "ee_mob", json!({ "kind": "change_block", "level": level(game), "pos": pos, "old": old, "new": new, "entity": entity }));
    // An enderman's arms changed as it asked: they go back with the block.
    if refused && kind == "minecraft:enderman" && let MobEvent::SetBlock { state, .. } = event {
        let air = game.world.air;
        if let Some(m) = game.mobs.iter_mut().find(|m| m.id == mob) {
            m.extra.carried = (*state != air).then_some(*state);
        }
    }
    refused
}

pub(in crate::game) fn mob_events(game: &mut Game<'_>, events: &[MobEvent]) -> bool {
    if AGAIN_EVENTS.swap(false, Ordering::Relaxed) {
        return false;
    }
    let here = level(game);
    let mut refused = Vec::new();
    let mut changed = Vec::new();
    let mut actor = None;
    for (i, event) in events.iter().enumerate() {
        if let MobEvent::Actor(mob) = event {
            actor = Some(*mob);
            continue;
        }
        *MOB_CHANGE.lock().unwrap_or_else(|e| e.into_inner()) = Value::Null;
        if block_refused(game, event, actor) {
            refused.push(i);
            continue;
        }
        let payload = match event {
            MobEvent::Projectile(p) => {
                let ProjectileKind::Potion { effect, .. } = p.kind else {
                    continue;
                };
                let Some(Who::Mob(owner)) = p.owner else {
                    continue;
                };
                let Some(target) = game.mobs.iter().find(|m| m.id == owner && &*m.kind.id == "minecraft:witch").map(|m| m.ai.target) else {
                    continue;
                };
                let target = target.map_or(Value::Null, |who| who_json(game, who, Some(owner)));
                json!({ "kind": "witch_throw", "level": here, "entity": owner, "target": target, "effect": effect })
            }
            MobEvent::SetBlock { pos, state } => {
                let registry = game.pack.assets.registry;
                let Some(block) = registry.state(*state).filter(|block| block.name == "minecraft:turtle_egg") else {
                    continue;
                };
                let eggs = block.properties.get("eggs").and_then(|eggs| eggs.parse::<i32>().ok()).unwrap_or(1);
                let at = [f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2])];
                json!({ "kind": "turtle_egg", "level": here, "pos": pos, "count": eggs, "entity": actor.or_else(|| nearest(game, "minecraft:turtle", at)) })
            }
            MobEvent::Tamed { mob } => {
                HORSE_TAMED.store(true, Ordering::Relaxed);
                json!({ "kind": "tame", "level": here, "entity": mob, "owner": uuid(player_of_mob(game, Some(*mob))) })
            }
            MobEvent::BreakDoor { pos } => {
                let mut cracking = CRACKING.lock().unwrap_or_else(|e| e.into_inner());
                let mob = actor.or_else(|| cracking.iter().find(|(at, _)| at == pos).map(|(_, mob)| *mob));
                cracking.retain(|(at, _)| at != pos);
                drop(cracking);
                json!({ "kind": "break_door", "level": here, "pos": pos, "entity": mob })
            }
            MobEvent::ExplosionSource { mob, .. } => {
                exploder(*mob, "minecraft:creeper");
                continue;
            }
            MobEvent::Explode(.., species) => {
                if EXPLODER.lock().unwrap_or_else(|e| e.into_inner()).is_none() {
                    exploder(0, &format!("minecraft:{}", format!("{species:?}").to_lowercase()));
                }
                continue;
            }
            MobEvent::DestroyProgress { mob, pos, .. } => {
                let mut cracking = CRACKING.lock().unwrap_or_else(|e| e.into_inner());
                cracking.retain(|(at, _)| at != pos);
                cracking.truncate(63);
                cracking.push((*pos, *mob));
                continue;
            }
            _ => continue,
        };
        if bridge::on_refusable(game, "ee_mob", payload) {
            refused.push(i);
        } else if let Some(new) = mob_change(game, event) {
            changed.push((i, new));
        }
    }
    let any = !refused.is_empty() || !changed.is_empty();
    *REFUSED.lock().unwrap_or_else(|e| e.into_inner()) = refused;
    CHANGED.with(|slot| *slot.borrow_mut() = changed);
    any
}

static MOB_CHANGE: Mutex<Value> = Mutex::new(Value::Null);

pub(in crate::game) fn nearest(game: &Game<'_>, kind: &str, at: [f64; 3]) -> Option<u32> {
    let distance = |m: &Mob| (m.position.x - at[0]).powi(2) + (m.position.y - at[1]).powi(2) + (m.position.z - at[2]).powi(2);
    game.mobs.iter().filter(|m| &*m.kind.id == kind && m.alive()).min_by(|a, b| distance(a).total_cmp(&distance(b))).map(|m| m.id)
}

/// The event as a listener rewrote it: a witch's other potion, a turtle's other number of eggs.
fn mob_change(game: &Game<'_>, event: &MobEvent) -> Option<MobEvent> {
    let change = std::mem::take(&mut *MOB_CHANGE.lock().unwrap_or_else(|e| e.into_inner()));
    match event {
        MobEvent::Projectile(p) => {
            let id = change["potion"].as_str()?;
            let def = horizon_client::brewing::potion(id)?;
            let (effect, ticks, amplifier) = *def.effects.first()?;
            let mut p = p.clone();
            p.kind = ProjectileKind::Potion { effect, ticks, amplifier, color: horizon_client::brewing::potion_color(Some(id)) };
            Some(MobEvent::Projectile(p))
        }
        MobEvent::SetBlock { pos, state } => {
            let eggs = change["eggs"].as_i64()?;
            let registry = game.pack.assets.registry;
            let mut block = registry.state(*state)?.clone();
            block.properties.insert("eggs".to_owned(), eggs.to_string());
            Some(MobEvent::SetBlock { pos: *pos, state: registry.id(&block)? })
        }
        _ => None,
    }
}

static AGAIN_EVENTS: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn kept_events(events: &[MobEvent]) -> Vec<MobEvent> {
    AGAIN_EVENTS.store(true, Ordering::Relaxed);
    let refused = REFUSED.lock().unwrap_or_else(|e| e.into_inner());
    CHANGED.with(|changed| {
        let changed = changed.borrow();
        events
            .iter()
            .enumerate()
            .filter(|(i, _)| !refused.contains(i))
            .map(|(i, event)| changed.iter().find(|(at, _)| *at == i).map_or_else(|| event.clone(), |(_, new)| new.clone()))
            .collect()
    })
}

/// What a listener of `ExplosionPrimeEvent` left: refused, the radius and the fire.
static EXPLOSION: Mutex<Option<(bool, f32, bool)>> = Mutex::new(None);
static AGAIN_EXPLOSION: AtomicBool = AtomicBool::new(false);

/// True goes to the row's own `Break`: it runs the explosion again with what's asked, or not at all.
pub(in crate::game) fn explode(game: &mut Game<'_>, center: [f64; 3], radius: f32, fire: bool, interaction: crate::game::explosion::BlockInteraction) -> bool {
    use crate::game::explosion::BlockInteraction as Blocks;
    *BLAST_KIND.lock().unwrap_or_else(|e| e.into_inner()) = match interaction {
        Blocks::Keep => ("KEEP", 1.0),
        Blocks::Destroy => ("DESTROY", 1.0),
        Blocks::DestroyWithDecay => ("DESTROY_WITH_DECAY", 1.0 / radius.max(0.001)),
    };
    if AGAIN_EXPLOSION.swap(false, Ordering::Relaxed) {
        return false;
    }
    *EXPLOSION.lock().unwrap_or_else(|e| e.into_inner()) = None;
    {
        let mut who = EXPLODER.lock().unwrap_or_else(|e| e.into_inner());
        if who.is_none()
            && let Some((id, _)) = PRIMED.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|(_, at)| at[0] == center[0] && at[2] == center[2])
        {
            *who = Some((*id, "minecraft:tnt".to_owned()));
        }
    }
    let who = exploder_now();
    // What blows up has mostly left the world: a listener still reads what it was and where.
    if let Some((id, kind)) = &who {
        *DYING.lock().unwrap_or_else(|e| e.into_inner()) = Some((*id, kind.clone(), center, json!({})));
    }
    bridge::on_trigger(game, "ee_explode", json!({ "level": level(game), "at": center, "radius": radius, "fire": fire, "entity": who.as_ref().map(|(id, _)| *id).filter(|id| *id != 0), "entity_kind": who.map(|(_, kind)| kind) }));
    EXPLOSION.lock().unwrap_or_else(|e| e.into_inner()).is_some_and(|(refused, wanted, burns)| refused || wanted != radius || burns != fire)
}

/// The explosion a listener changed, or nothing for one it stopped.
pub(in crate::game) fn explosion_again(game: &mut Game<'_>, center: [f64; 3], interaction: crate::game::explosion::BlockInteraction, damage_type: &str, resists: Option<([i32; 3], f32)>) -> horizon_client::Result<()> {
    let Some((refused, radius, fire)) = EXPLOSION.lock().unwrap_or_else(|e| e.into_inner()).take() else {
        return Ok(());
    };
    if refused {
        return Ok(());
    }
    AGAIN_EXPLOSION.store(true, Ordering::Relaxed);
    game.server_explode_as(center, radius, interaction, fire, damage_type, resists)
}

/// A lead's break, as a listener left it: refused, and whether its item drops.
static LEAD_DROP: Mutex<Option<(bool, bool)>> = Mutex::new(None);
static AGAIN_LEAD: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn unleash(game: &mut Game<'_>, id: u32, drop: bool) -> bool {
    let reason = ee::take_unleash_reason();
    if AGAIN_LEAD.swap(false, Ordering::Relaxed) {
        return false;
    }
    *LEAD_DROP.lock().unwrap_or_else(|e| e.into_inner()) = None;
    bridge::on_trigger(game, "ee_unleash", json!({ "level": level(game), "entity": id, "drop": drop, "reason": reason }));
    LEAD_DROP.lock().unwrap_or_else(|e| e.into_inner()).is_some_and(|(refused, wanted)| refused || wanted != drop)
}

/// Whether the lead's item drops when it breaks after all: `None` for a break that was refused.
pub(in crate::game) fn lead_drop(drop: bool) -> Option<bool> {
    let (refused, wanted) = LEAD_DROP.lock().unwrap_or_else(|e| e.into_inner()).unwrap_or((false, drop));
    if refused {
        return None;
    }
    AGAIN_LEAD.store(true, Ordering::Relaxed);
    Some(wanted)
}

/// A swallowed block a listener changed: the game's own goes in first.
static HEADS: Mutex<Vec<(u64, String, String)>> = Mutex::new(Vec::new());

/// Who is who in the tick, for what's decided with the game out of reach: the middle of a hit.
static KNOWN: Mutex<Option<HashMap<i32, Value>>> = Mutex::new(None);
static INSTALLED: Once = Once::new();

/// Whether hits are noted, which starts at the first level tick a listener wants.
pub(in crate::game) fn damage_noted() -> bool {
    INSTALLED.is_completed()
}

/// At each level's tick, when a listener wants it.
pub(in crate::game) fn tick_note(game: &mut Game<'_>) -> bool {
    // From now on each hit's middle is noted first, before the plugins' say on it (`paper_events::allow_damage_mid`).
    INSTALLED.call_once(|| {});
    for (id, dimension, item) in std::mem::take(&mut *HEADS.lock().unwrap_or_else(|e| e.into_inner())) {
        let request = json!({ "id": id, "dimension": dimension });
        with_mob(game, &request, |m| m.extra.head = Some(item.as_str().into()));
    }
    for i in 0..game.breath_clouds.len() {
        if game.breath_clouds[i].id == 0 {
            game.breath_clouds[i].id = fresh_id(game);
        }
    }
    for i in 0..game.dragon_fireballs.len() {
        if game.dragon_fireballs[i].id == 0 {
            game.dragon_fireballs[i].id = fresh_id(game);
        }
    }
    let here = level(game);
    ee::set_here(&here);
    ee::effect_cause("UNKNOWN");
    ee::why("");
    let ids: Vec<(u32, String, [f64; 3])> = game.mobs.iter().map(|m| (m.id, m.kind.id.to_string(), [m.position.x, m.position.y, m.position.z])).collect();
    let mut rows = Vec::with_capacity(ids.len());
    for (id, kind, at) in ids {
        let uuid = game.uuid_for(id).iter().fold(0_u128, |all, word| (all << 32) | u128::from(*word as u32));
        rows.push((id as i32, json!({ "id": id, "dimension": here, "uuid": uuid_text(uuid), "kind": kind, "at": at, "pos": at, "data": {} })));
    }
    let (host_id, host_name) = (game.native_acting_id(), game.acting_player_name().to_owned());
    let host_body = game.player.as_ref().map(|player| player.entity_id);
    let players = game.native_server_players();
    ee::hold_rows(Vec::new());
    let mut known = KNOWN.lock().unwrap_or_else(|e| e.into_inner());
    let known = known.get_or_insert_default();
    known.retain(|_, row| row["dimension"] != here.as_str());
    known.extend(rows);
    let mut host = json!({ "uuid": uuid(host_id), "name": host_name, "player": true, "dimension": here });
    if let Some(player) = players.iter().find(|player| player.uuid == host_id) {
        host["position"] = json!(player.position);
        host["at"] = json!(player.position);
        host["health"] = json!(player.health);
        host["game_mode"] = json!(player.game_mode);
        host["permission"] = json!(player.permission);
        host["yaw"] = json!(player.yaw);
        host["pitch"] = json!(player.pitch);
    }
    if let Some(id) = host_body {
        host["id"] = json!(id);
        known.insert(id, host.clone());
    }
    known.insert(ee::HOST, host);
    if let Some(host) = game.lan.as_ref() {
        for g in host.hub.guests.iter().filter(|g| g.placed) {
            let player = players.iter().find(|player| player.uuid == g.uuid);
            let row = player.map_or(Value::Null, |player| json!({ "permission": player.permission, "position": player.position, "game_mode": player.game_mode, "health": player.health, "yaw": player.yaw, "pitch": player.pitch }));
            let mut guest = json!({ "id": g.entity_id, "dimension": g.dimension, "uuid": uuid_text(g.uuid), "name": g.name, "player": true, "at": g.position });
            if let (Some(guest), Some(row)) = (guest.as_object_mut(), row.as_object()) {
                guest.extend(row.clone());
            }
            known.insert(g.entity_id, guest);
        }
    }
    true
}

fn uuid_text(value: u128) -> String {
    uuid(value)
}

/// The side of `mobs::ee` that lives here.
fn free_fire(hook: &str, payload: &Value) -> (i32, Value) {
    let (code, text) = runtime::fire_carried(hook, &payload.to_string());
    (code, text.and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_else(|| payload.clone()))
}

fn known_row(id: i32) -> Value {
    KNOWN.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|known| known.get(&id)).cloned().unwrap_or(Value::Null)
}

fn player_row(id: &str) -> Value {
    let known = KNOWN.lock().unwrap_or_else(|e| e.into_inner());
    known.as_ref().and_then(|known| known.values().find(|row| row["player"] == true && row["uuid"] == id)).cloned().unwrap_or(Value::Null)
}

/// A listener's question with no game lent, inside `ee::ask`: what the event's own rows can say.
pub(in crate::game) fn free_answer(request: &str) -> Option<String> {
    let query: Value = serde_json::from_str(request).ok()?;
    let row = match query["op"].as_str()? {
        "entity" | "entity_kept" | "mob" => dying_row(query["id"].as_u64()? as u32)
            .map(|(kind, at, data)| json!({ "id": query["id"], "kind": kind, "dimension": query["dimension"], "pos": at, "yaw": 0.0, "pitch": 0.0, "uuid": "", "data": data }))
            .or_else(|| ee::row_of(query["id"].as_u64().unwrap_or(0), query["data"] == true)),
        "block" => ee::block_of(&query["pos"]),
        "player" => ee::player_of(query["uuid"].as_str()?),
        "entities" => {
            let known = KNOWN.lock().unwrap_or_else(|e| e.into_inner());
            let rows = known.iter().flat_map(|known| known.values()).filter(|row| row["dimension"] == query["dimension"] && row["player"] != true);
            Some(Value::Array(rows.cloned().collect()))
        }
        _ => None,
    };
    row.map(|row| row.to_string()).or_else(|| keep_for_later(query["op"].as_str()?, request))
}

/// What listeners asked of the game while it couldn't be had: its turn comes with `after_mobs`.
static ASKED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// `reads`: a late read tells nobody anything.
fn keep_for_later(op: &str, request: &str) -> Option<String> {
    let reads = op.ends_with("_get") || op.ends_with("_list") || op.ends_with("_known") || matches!(op, "entity" | "mob" | "mobs" | "entities" | "block" | "block_data" | "block_facts" | "player" | "players" | "inventory" | "equipment" | "effects" | "tag" | "tags" | "version" | "level" | "world" | "item_info" | "chunk_loaded" | "chunk_snapshot" | "chunk_entities" | "attribute" | "mob_type" | "tick_state" | "tick_times" | "data_list" | "ee_mobs" | "ee_players" | "ee_items" | "ee_mob_at");
    if reads {
        return None;
    }
    let mut asked = ASKED.lock().unwrap_or_else(|e| e.into_inner());
    if asked.len() < 4096 {
        asked.push(request.to_owned());
    }
    // These two have a fallback a yes turns off: the game would hear them twice.
    matches!(op, "mob_action" | "entity_merge").then(|| "true".to_owned())
}

/// The tree a mob saves, from the mob itself.
fn mob_tree(m: &Mob, uuid: &str) -> Value {
    let whole = u128::from_str_radix(&uuid.replace('-', ""), 16).unwrap_or(0);
    let words = [(whole >> 96) as u32 as i32, (whole >> 64) as u32 as i32, (whole >> 32) as u32 as i32, whole as u32 as i32];
    crate::game::commands::paper_entities::tag_json(&horizon_client::entity_nbt::mob_to_nbt(m, words, None, None))
}

/// An event with the game lent: whether a listener refused, and the payload as they left it.
fn fire_lent(game: &mut Game<'_>, hook: &str, payload: Value) -> (bool, Value) {
    let text = payload.to_string();
    let (code, text) = runtime::lend(game, || runtime::fire_carried(hook, &text));
    // Nobody hears this one: what the listeners asked of the game goes out on its way through.
    bridge::on_trigger(game, "ee_flush", json!({}));
    (code == 2, text.and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(payload))
}

static BRIDGED: Once = Once::new();

/// The mobs are back: what their ticks kept for the listeners to look at.
pub(in crate::game) fn after_mobs(game: &mut Game<'_>) {
    BRIDGED.call_once(|| ee::install(ee::Bridge { fire: free_fire, hears: runtime::listens, known: known_row, player: player_row, stack: stack_json, tree: mob_tree }));
    ee::bind(None);
    ee::hold_actor(None);
    dying_over();
    arrows_fall(game);
    mobs_change_equipment(game);
    items_inside(game);
    for ee::Later { mob, hook, payload, apply } in ee::take_later() {
        let (refused, after) = fire_lent(game, hook, payload);
        if !refused && let Some(m) = game.mobs.iter_mut().find(|m| m.id == mob) {
            apply(m, &after);
        }
    }
    let asked = std::mem::take(&mut *ASKED.lock().unwrap_or_else(|e| e.into_inner()));
    if !asked.is_empty() {
        runtime::lend(game, || {
            for request in &asked {
                runtime::borrowed(|game: &mut Game<'static>| crate::game::paper_query::answer(game, request));
            }
        });
        bridge::on_trigger(game, "ee_flush", json!({}));
    }
}

/// What `EntityPortalEvent` and `EntityPortalReadyEvent` left for the plan the game works out next.
static PORTAL: Mutex<Option<(Value, crate::game::Dim, horizon_core::position::BlockPos)>> = Mutex::new(None);

/// A mob's, item's or vehicle's way through a portal, before it goes: true if it stays.
pub(in crate::game) fn portal(game: &mut Game<'_>, id: u32, entry: horizon_core::position::BlockPos, plan: crate::game::nether_portal::Plan) -> bool {
    let to = plan.approximate;
    let to_nether = plan.dim.kind() == crate::game::Dim::Nether;
    let entity = row_for(game, id);
    let payload = json!({
        "level": level(game),
        "entity": entity,
        "pos": [entry.x, entry.y, entry.z],
        "portal": if plan.end { "ENDER" } else { "NETHER" },
        "to": [to.x, to.y, to.z],
        "to_level": plan.dim.id(),
        "target": plan.dim.id(),
        "search": if to_nether { 16 } else { 128 },
        "creation": 16,
        "can_create": true,
    });
    let (refused, after) = fire_lent(game, "ee_portal", payload);
    if refused || after["target"].is_null() || after["to"].is_null() {
        return true;
    }
    *PORTAL.lock().unwrap_or_else(|e| e.into_inner()) = Some((after, game.dim, entry));
    false
}

/// `entity_plan`: the plan as the listeners left it.
pub(in crate::game) fn portal_answer(plan: &mut crate::game::nether_portal::Plan) {
    let Some((after, from, entry)) = PORTAL.lock().unwrap_or_else(|e| e.into_inner()).take() else {
        return;
    };
    let own = (plan.dim, plan.approximate);
    if let Some(target) = after["target"].as_str() {
        plan.dim = crate::game::Dim::from_id(target);
    }
    if let (Some(world), Some(x), Some(y), Some(z)) = (after["to_level"].as_str(), after["to"][0].as_f64(), after["to"][1].as_f64(), after["to"][2].as_f64()) {
        let to = horizon_core::position::BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32);
        if world != own.0.id() {
            plan.dim = crate::game::Dim::from_id(world);
        }
        plan.approximate = to;
    }
    plan.rules.search = after["search"].as_i64().map(|radius| radius as i32);
    plan.rules.creation = after["creation"].as_i64().map(|radius| radius as i32);
    plan.rules.no_portal = after["can_create"] == false;
    plan.rules.exact = plan.end && plan.approximate != own.1;
    plan.rules.back = Some((from, entry));
}

/// `EntityPortalExitEvent` as it arrives: where it lands, and how fast.
pub(in crate::game) fn portal_exit(game: &mut Game<'_>, tag: &horizon_world::nbt::Tag, rules: crate::game::nether_portal::PortalRules, at: Vec3, motion: Vec3) -> (Vec3, Vec3) {
    if !bridge::hears("ee_portal_exit") {
        return (at, motion);
    }
    use horizon_client::entity_nbt as saved;
    let before = saved::motion(tag);
    let from = saved::position(tag).unwrap_or(at);
    let (from_level, entry) = rules.back.map_or((game.dim.id(), None), |(dim, entry)| (dim.id(), Some([entry.x, entry.y, entry.z])));
    let payload = json!({
        "level": level(game),
        "from_level": from_level,
        "kind": saved::kind(tag),
        "uuid": saved::uuid(tag).map(|ints| uuid(horizon_client::lan::playerdata::ints_uuid(ints))),
        "entry": entry,
        "from": [from.x, from.y, from.z],
        "to": [at.x, at.y, at.z],
        "before": [before.x, before.y, before.z],
        "after": [motion.x, motion.y, motion.z],
    });
    let (refused, left) = fire_lent(game, "ee_portal_exit", payload);
    if refused {
        return (at, motion);
    }
    let vector = |value: &Value, own: Vec3| match (value[0].as_f64(), value[1].as_f64(), value[2].as_f64()) {
        (Some(x), Some(y), Some(z)) => Vec3::new(x, y, z),
        _ => own,
    };
    (vector(&left["to"], at), vector(&left["after"], motion))
}

/// `EntityIgniteEvent`: a TNT minecart's fuse, or `None` if refused.
pub(in crate::game) fn cart_ignite(game: &mut Game<'_>, id: u32, fuse: i32) -> Option<i32> {
    if !bridge::hears("ee_cart_ignite") {
        return Some(fuse);
    }
    let entity = row_for(game, id);
    let (refused, after) = fire_lent(game, "ee_cart_ignite", json!({ "entity": entity, "fuse": fuse }));
    (!refused).then(|| after["fuse"].as_i64().map_or(fuse, |fuse| fuse as i32))
}

/// `EntityPortalEnterEvent`, each tick it's in a portal: true if a listener keeps it out.
pub(in crate::game) fn portal_enter(game: &mut Game<'_>, id: u32, pos: horizon_core::position::BlockPos) -> bool {
    if !bridge::hears("ee_portal_enter") {
        return false;
    }
    let portal = if game.pack.assets.registry.state(game.world.block(pos.x, pos.y, pos.z)).is_some_and(|state| state.name == "minecraft:end_portal") { "ENDER" } else { "NETHER" };
    let entity = row_for(game, id);
    fire_lent(game, "ee_portal_enter", json!({ "level": level(game), "entity": entity, "pos": [pos.x, pos.y, pos.z], "portal": portal })).0
}

/// The same for the player in hand, each tick they stand in a nether portal.
pub(in crate::game) fn player_portal_enter(game: &mut Game<'_>, pos: horizon_core::position::BlockPos) -> bool {
    if !bridge::hears("ee_portal_enter") {
        return false;
    }
    let (player, name) = (game.native_acting_id(), game.acting_player_name().to_owned());
    let entity = json!({ "uuid": uuid(player), "name": name, "player": true, "dimension": level(game) });
    fire_lent(game, "ee_portal_enter", json!({ "level": level(game), "entity": entity, "pos": [pos.x, pos.y, pos.z], "portal": "NETHER" })).0
}

/// `PlayerLeashEntityEvent`, as the lead goes on: true if refused.
pub(in crate::game) fn leash_refused(game: &mut Game<'_>, index: usize, item: &str) -> bool {
    if !bridge::hears("ee_leash") {
        return false;
    }
    let hand = if game.held().is_some_and(|held| &*held.item == item) { "HAND" } else { "OFF_HAND" };
    let payload = json!({ "level": level(game), "entity": game.mobs[index].id, "hand": hand });
    bridge::on_refusable(game, "ee_leash", payload)
}

/// `EntityPickupItemEvent` for a mob, with the item still on the ground: true if refused.
pub(in crate::game) fn mob_pickup(game: &mut Game<'_>, mob: u32, item: usize) -> bool {
    if !bridge::hears("ee_pickup") {
        return false;
    }
    let mut entity = row_for(game, mob);
    entity["kind"] = json!(game.mobs.iter().find(|m| m.id == mob).map(|m| m.kind.id.to_string()));
    let dropped = &game.dropped[item];
    let payload = json!({ "level": level(game), "entity": entity, "item_id": dropped.id, "stack": stack_json(&dropped.stack), "remaining": 0 });
    fire_lent(game, "ee_pickup", payload).0
}

/// `EntityPotionEffectEvent` for an effect the player in hand is losing: true if refused.
pub(in crate::game) fn effect_gone(game: &mut Game<'_>, id: &str, cause: &str, action: &str) -> bool {
    if !bridge::hears("ee_effect") {
        return false;
    }
    let old = game.potions.effects.list.iter().find(|effect| effect.id == id).map(|effect| json!({ "amplifier": effect.amplifier, "duration": effect.duration }));
    let Some(old) = old else {
        return false;
    };
    let amplifier = old["amplifier"].clone();
    bridge::on_refusable(game, "ee_effect", json!({ "effect": id, "amplifier": amplifier, "duration": 0, "cause": cause, "action": action, "old": old, "ended": true }))
}

/// All of the player's effects going at once: those a listener keeps stay.
pub(in crate::game) fn effects_clear(game: &mut Game<'_>, cause: &str) {
    if !bridge::hears("ee_effect") {
        return game.potions.effects.clear();
    }
    let list = game.potions.effects.list.clone();
    let kept: Vec<_> = list.into_iter().filter(|effect| effect_gone(game, effect.id, cause, "CLEARED")).collect();
    game.potions.effects.clear();
    game.potions.effects.list = kept;
}

/// What the player has before their effects tick, if anyone wants to hear what runs out.
pub(in crate::game) fn effects_held(effects: &horizon_client::effects::PlayerEffects) -> Vec<horizon_client::effects::EffectInstance> {
    if bridge::hears("ee_effect") { effects.list.clone() } else { Vec::new() }
}

/// What ran out in the tick: a refused one comes back for a tick.
pub(in crate::game) fn effects_expired(game: &mut Game<'_>, had: Vec<horizon_client::effects::EffectInstance>) {
    for mut effect in had {
        if game.potions.effects.list.iter().any(|now| now.id == effect.id) {
            continue;
        }
        let old = json!({ "amplifier": effect.amplifier, "duration": effect.duration });
        let payload = json!({ "effect": effect.id, "amplifier": effect.amplifier, "duration": 0, "cause": "EXPIRATION", "action": "REMOVED", "old": old, "ended": true });
        if bridge::on_refusable(game, "ee_effect", payload) {
            effect.duration = 1;
            game.potions.effects.list.push(effect);
        }
    }
}

static LUNGE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);

/// `post_piercing_attack`: Lunge's level, or what a listener made it.
pub(in crate::game) fn lunge_power(id: &str, level: u32) -> u32 {
    if id != "minecraft:lunge" {
        return level;
    }
    u32::try_from(LUNGE.swap(-1, Ordering::Relaxed)).unwrap_or(level)
}

/// Each player's spawn radius this round: a listener's, or none at all for one refused.
static REACH: Mutex<Option<HashMap<i32, i32>>> = Mutex::new(None);

pub(in crate::game) fn spawn_reach(player: i32) -> i32 {
    REACH.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|reach| reach.get(&player)).copied().unwrap_or(8)
}

fn row_for(game: &mut Game<'_>, id: u32) -> Value {
    let uuid = game.uuid_for(id).iter().fold(0_u128, |all, word| (all << 32) | u128::from(*word as u32));
    json!({ "id": id, "dimension": level(game), "uuid": uuid_text(uuid) })
}

/// `EntityDamageItemEvent` on what a mob wears, with its wear worked out: what it really loses.
pub(in crate::game) fn item_wear(game: &mut Game<'_>, id: u32, stack: &ItemStack, loss: u32) -> u32 {
    if loss == 0 || !bridge::hears("ee_item_wear") {
        return loss;
    }
    let entity = row_for(game, id);
    let (refused, after) = fire_lent(game, "ee_item_wear", json!({ "entity": entity, "item": stack_json(stack), "damage": loss }));
    if refused { 0 } else { after["damage"].as_u64().map_or(loss, |damage| damage as u32) }
}

/// `ALLOW_DAMAGE` at its first phase, so that listeners of the hit have this before they hear it.
pub(in crate::game) fn damage_heard(damage: &living::Damage<'_>) {
    if !runtime::listens("ee_damage_info") {
        return;
    }
    // `subject_row` reads `KNOWN` itself: holding it here froze the server at the first mob hurt.
    let row = |id: i32| ee::subject_row(id as u64).unwrap_or_else(|| known_row(id));
    // A player's blow on a mob has no causing id: the mob's own damage says who.
    let causer = Some(row(damage.causing)).filter(|row| !row.is_null()).unwrap_or_else(ee::hit_by_row);
    *LAST_HIT.lock().unwrap_or_else(|e| e.into_inner()) = causer.clone();
    ee::hold_rows([row(damage.entity), causer.clone(), row(damage.direct)].into_iter().filter(|row| row["dimension"].is_string()).collect());
    let payload = json!({ "ee_info": true, "entity": damage.entity, "victim": row(damage.entity), "causer": causer, "source": row(damage.direct) });
    runtime::fire("ee_damage_info", &payload.to_string());
}

/// A mob's kin at the moment of its change: what a cancel of `EntityTransformEvent` brings back.
thread_local! {
    static CONVERTING: RefCell<Option<Box<Mob>>> = const { RefCell::new(None) };
    static CHANGED: RefCell<Vec<(usize, MobEvent)>> = const { RefCell::new(Vec::new()) };
}

pub(in crate::game) fn note_convert(from: &Mob) -> bool {
    // It has left the list by the time its event is heard: this is what a listener reads of it.
    if runtime::listens("mob_conversion") {
        let row = known_row(from.id as i32);
        let data = mob_tree(from, row["uuid"].as_str().unwrap_or_default());
        *DYING.lock().unwrap_or_else(|e| e.into_inner()) = Some((from.id, from.kind.id.to_string(), [from.position.x, from.position.y, from.position.z], data));
    }
    CONVERTING.with(|slot| *slot.borrow_mut() = Some(Box::new(from.clone())));
    true
}

/// `HAND` or `OFF_HAND`, by whether the main hand holds this stack's item.
pub(in crate::game) fn hand_of(game: &Game<'_>, stack: &ItemStack) -> &'static str {
    if game.held().is_some_and(|held| held.item == stack.item) { "HAND" } else { "OFF_HAND" }
}

/// A horse's taming heard ahead of the game's own trigger: that one is no second event.
static HORSE_TAMED: AtomicBool = AtomicBool::new(false);

/// `EntityTameEvent` for an animal a player's hand tamed: it has happened, and a refusal undoes it.
pub(in crate::game) fn tamed(game: &mut Game<'_>, name: &str, kind: &str, at: [f64; 3]) -> bool {
    if name != "minecraft:tame_animal" || HORSE_TAMED.swap(false, Ordering::Relaxed) || ee::take_tame_heard() {
        return false;
    }
    let Some(id) = nearest(game, kind, at) else {
        return false;
    };
    let payload = json!({ "kind": "tame", "level": level(game), "entity": id, "owner": uuid(game.native_acting_id()) });
    let refused = bridge::on_refusable(game, "ee_mob", payload);
    if refused && let Some(m) = game.mobs.iter_mut().find(|m| m.id == id) {
        m.extra.tame = false;
        m.extra.owner = None;
        m.extra.sitting = false;
    }
    refused
}

/// What's giving effects this tick: (the cause, the tick).
static CAUSE: Mutex<(&'static str, i64)> = Mutex::new(("UNKNOWN", 0));

pub(in crate::game) fn note_cause(game: &Game<'_>, cause: &'static str) -> bool {
    ee::effect_cause(cause);
    *CAUSE.lock().unwrap_or_else(|e| e.into_inner()) = (cause, game.world.game_time as i64);
    true
}

/// A mob's hit gives an effect: `ATTACK`, by whoever just hit.
pub(in crate::game) fn note_attack(game: &Game<'_>) -> bool {
    let told = take_cause_next();
    let from = ee::take_effect_from();
    let by = if from.is_null() && told.is_none() { LAST_HIT.lock().unwrap_or_else(|e| e.into_inner()).clone() } else { from };
    *SOURCE.lock().unwrap_or_else(|e| e.into_inner()) = (by, game.world.game_time as i64);
    note_cause(game, told.unwrap_or("ATTACK"))
}

/// Who gave the effect going on now: `EntityPotionEffectEvent.getSource`.
static SOURCE: Mutex<(Value, i64)> = Mutex::new((Value::Null, 0));
/// Who last hit anything, this tick.
static LAST_HIT: Mutex<Value> = Mutex::new(Value::Null);

pub(in crate::game) fn effect_source(game: &Game<'_>) -> Value {
    let source = SOURCE.lock().unwrap_or_else(|e| e.into_inner());
    if source.1 == game.world.game_time as i64 { source.0.clone() } else { Value::Null }
}

pub(in crate::game) fn effect_cause(game: &Game<'_>) -> &'static str {
    let (cause, tick) = *CAUSE.lock().unwrap_or_else(|e| e.into_inner());
    if tick == game.world.game_time as i64 { cause } else { "UNKNOWN" }
}

/// What a tick's `EntityEffectTickEvent` is, by what the effect does: the game's tick is told no more.
pub(in crate::game) fn effect_of(action: &horizon_client::effects::EffectAction) -> &'static str {
    use horizon_client::effects::EffectAction as Action;
    match action {
        Action::Heal(_) => "regeneration",
        Action::Hurt(kind, _) if kind.ends_with("wither") => "wither",
        Action::Hurt(..) => "instant_damage",
        Action::Poison => "poison",
        Action::Exhaust(_) => "hunger",
        Action::Eat(..) => "saturation",
    }
}

/// `ThrownEggHatchEvent`, with the game's own roll: how many hatch, and what.
pub(in crate::game) fn egg_hatch(game: &mut Game<'_>, id: u32, hatching: bool, count: i32) -> (i32, String) {
    let own = (count, "minecraft:chicken".to_owned());
    if !bridge::hears("ee_egg") {
        return own;
    }
    let (_, after) = fire_lent(game, "ee_egg", json!({ "level": level(game), "entity": id, "hatching": hatching, "count": if hatching { count } else { 1 }, "kind": own.1 }));
    let hatches = after["hatching"] == true;
    (if hatches { after["count"].as_i64().map_or(count, |count| count as i32) } else { 0 }, after["kind"].as_str().map_or(own.1, str::to_owned))
}

/// `SlimeSplitEvent`, with the game's own roll: how many it splits into.
pub(in crate::game) fn slime_count(game: &mut Game<'_>, m: &Mob, count: i32) -> i32 {
    if REVIVED.load(Ordering::Relaxed) == m.id {
        return 0;
    }
    if !bridge::hears("ee_slime") {
        return count;
    }
    let (refused, after) = fire_lent(game, "ee_slime", json!({ "level": level(game), "entity": m.id, "count": count }));
    if refused { 0 } else { after["count"].as_i64().map_or(count, |count| count as i32) }
}

/// `PiglinBarterEvent`, with the game's own roll: what it throws, or `None` if refused.
pub(in crate::game) fn barter_outcome(game: &mut Game<'_>, mob: u32, input: &str, items: Vec<ItemStack>) -> Option<Vec<ItemStack>> {
    if !bridge::hears("ee_barter") {
        return Some(items);
    }
    let outcome: Vec<Value> = items.iter().map(stack_json).collect();
    let (refused, after) = fire_lent(game, "ee_barter", json!({ "level": level(game), "entity": mob, "input": { "item": input, "count": 1 }, "outcome": outcome }));
    if refused {
        return None;
    }
    Some(after["outcome"].as_array().map_or(items, |rows| rows.iter().filter_map(stack_of).collect()))
}

/// The world's spawning round, one event a player; true if every one refused.
pub(in crate::game) fn spawn_round(game: &mut Game<'_>) -> bool {
    let players: Vec<u128> = game.native_server_players().iter().map(|player| player.uuid).collect();
    let mut refused = !players.is_empty();
    let mut reach = HashMap::new();
    for player in players {
        let id = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == player).map(|g| g.entity_id)).unwrap_or(0);
        let (no, after) = fire_lent(game, "ee_spawn_round", json!({ "player": { "uuid": uuid(player) }, "radius": 8 }));
        refused &= no;
        reach.insert(id, if no { -1 } else { after["radius"].as_i64().map_or(8, |radius| radius as i32) });
    }
    *REACH.lock().unwrap_or_else(|e| e.into_inner()) = Some(reach);
    refused
}

/// A pattern about to go: what `EntityConstructEvent` lists, and what a cancel puts back.
static PATTERN: Mutex<Vec<([i32; 3], horizon_world::registry::BlockStateId)>> = Mutex::new(Vec::new());

pub(in crate::game) fn note_pattern(game: &Game<'_>, cells: &[[i32; 3]]) -> bool {
    *PATTERN.lock().unwrap_or_else(|e| e.into_inner()) = cells.iter().map(|p| (*p, game.world.block(p[0], p[1], p[2]))).collect();
    true
}

pub(in crate::game) fn pattern() -> Vec<[i32; 3]> {
    PATTERN.lock().unwrap_or_else(|e| e.into_inner()).iter().map(|(at, _)| *at).collect()
}

pub(in crate::game) fn restore_pattern(game: &mut Game<'_>) {
    let cells = std::mem::take(&mut *PATTERN.lock().unwrap_or_else(|e| e.into_inner()));
    let here = level(game);
    for (at, state) in cells {
        game.native_in_level(&here, |game| game.native_set_block(game.dim.id(), at, state, 3));
    }
}

/// The breeding's experience, rolled here so that a listener can have it: (amount, tick).
static BREED_XP: Mutex<Option<(u32, i64)>> = Mutex::new(None);

pub(in crate::game) fn breed_exp(game: &mut Game<'_>) -> u32 {
    let exp = 1 + game.world.random.next_int_bound(7).unwrap_or(0) as u32;
    *BREED_XP.lock().unwrap_or_else(|e| e.into_inner()) = Some((exp, game.world.game_time as i64));
    exp
}

pub(in crate::game) fn breed_orb(game: &Game<'_>) -> bool {
    BREED_XP.lock().unwrap_or_else(|e| e.into_inner()).is_some_and(|(_, tick)| tick == game.world.game_time as i64)
}

pub(in crate::game) fn breed_amount(amount: u32) -> u32 {
    BREED_XP.lock().unwrap_or_else(|e| e.into_inner()).take().map_or(amount, |(exp, _)| exp)
}

/// What each mob was last fed, by id.
static FED: Mutex<Option<HashMap<u32, Value>>> = Mutex::new(None);

pub(in crate::game) fn note_feed(game: &Game<'_>) -> bool {
    if let Some(mob) = game.mob_hit().and_then(|(i, _)| game.mobs.get(i)) {
        FED.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(mob.id, stack_opt(game.held()));
    }
    true
}

pub(in crate::game) fn fed(parent: u32, partner: u32) -> Value {
    let mut fed = FED.lock().unwrap_or_else(|e| e.into_inner());
    let fed = fed.get_or_insert_default();
    let item = fed.remove(&parent).or(fed.remove(&partner));
    item.unwrap_or(Value::Null)
}

/// When each trial spawner ticked, and the entity counter then.

/// `TrialSpawnerSpawnEvent`, as the mob stands in the world: true if a listener took it away.
pub(in crate::game) fn trial_spawned(game: &mut Game<'_>, id: u32, spawner: [i32; 3]) -> bool {
    if !bridge::hears("ee_trial_spawn") {
        return false;
    }
    let entity = row_for(game, id);
    let (refused, _) = fire_lent(game, "ee_trial_spawn", json!({ "level": level(game), "entity": entity, "pos": spawner }));
    if refused {
        game.mobs.retain(|m| m.id != id);
        game.forget_entity(id);
    }
    refused
}

/// `EnderDragonFlameEvent` or `EnderDragonFireballHitEvent`, for the cloud about to come: true if refused.
pub(in crate::game) fn dragon_cloud(game: &mut Game<'_>, hook: &str) {
    if !bridge::hears(hook) {
        return;
    }
    let id = fresh_id(game);
    let dragon = game.dragon.as_ref().map(|dragon| dragon.id);
    let Some(cloud) = game.breath_clouds.last_mut() else {
        return;
    };
    cloud.id = id;
    let (at, radius, duration) = (cloud.at, cloud.radius, cloud.duration);
    if fire_lent(game, hook, json!({ "level": level(game), "dragon": dragon, "cloud": id, "at": at, "radius": radius, "duration": duration, "ball": dying_id() })).0 {
        game.breath_clouds.retain(|cloud| cloud.id != id);
    }
}

/// `EnderDragonShootFireballEvent`, with the fireball in flight: true if refused, and it's gone.
pub(in crate::game) fn fireball_shot(game: &mut Game<'_>) -> bool {
    if !bridge::hears("ee_dragon_fire") {
        return false;
    }
    let id = fresh_id(game);
    let Some(ball) = game.dragon_fireballs.last_mut() else {
        return false;
    };
    ball.id = id;
    let at = [ball.position.x, ball.position.y, ball.position.z];
    let dragon = game.dragon.as_ref().map(|dragon| dragon.id);
    let refused = fire_lent(game, "ee_dragon_fire", json!({ "level": level(game), "entity": dragon, "ball": id, "at": at })).0;
    if refused {
        game.dragon_fireballs.retain(|ball| ball.id != id);
    }
    refused
}

/// `ProjectileLaunchEvent` with the potion in flight: a refusal takes it out again.
pub(in crate::game) fn potion_launched(game: &mut Game<'_>) {
    if !bridge::hears("ee_launch_potion") {
        return;
    }
    let Some(potion) = game.potions.thrown.last() else {
        return;
    };
    let id = potion.id;
    let kind = match &*potion.stack.item {
        "minecraft:lingering_potion" => "minecraft:lingering_potion",
        "minecraft:experience_bottle" => "minecraft:experience_bottle",
        _ => "minecraft:splash_potion",
    };
    let (stack, at) = (stack_json(&potion.stack), [potion.position.x, potion.position.y, potion.position.z]);
    if bridge::on_refusable(game, "ee_launch_potion", json!({ "level": level(game), "kind": kind, "stack": stack, "at": at, "entity": id })) {
        game.potions.thrown.retain(|potion| potion.id != id);
    }
}

/// The child `EntityBreedEvent` names: it stands for the event, not yet joined.
static CHILD: Mutex<Option<u32>> = Mutex::new(None);
/// A mob made for an event, which joins the level once that's been heard.
static HOLD_JOIN: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn hold_join(on: bool) {
    HOLD_JOIN.store(on, Ordering::Relaxed);
}

fn take_force() -> Option<f32> {
    FORCE.lock().unwrap_or_else(|e| e.into_inner()).take()
}

fn take_dropper() -> Option<Value> {
    DROPPER.lock().unwrap_or_else(|e| e.into_inner()).take()
}

fn take_cause_next() -> Option<&'static str> {
    CAUSE_NEXT.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// Who drops the next item, for `EntityDropItemEvent`.
static DROPPER: Mutex<Option<Value>> = Mutex::new(None);

/// `EntityBreedEvent`, with the child made as `Animal.getBreedOffspring` makes it. A refusal
/// leaves no child and takes the love off both, as `spawnChildFromBreeding` does.
pub(in crate::game) fn breed(game: &mut Game<'_>, parent: u32, partner: u32) -> bool {
    let breeder = breeder(game, parent, partner);
    let eggs = lays_eggs(game, parent);
    let exp = breed_exp(game);
    let item = fed(parent, partner);
    let mother = game.mobs.iter().find(|m| m.id == parent).map(|m| (m.kind.id.to_string(), [m.position.x, m.position.y, m.position.z]));
    let mate = game.mobs.iter().find(|m| m.id == partner).map(|m| m.kind.id.to_string());
    let child = mother.filter(|_| !eggs).and_then(|(kind, at)| {
        let kind = match (kind.as_str(), mate.as_deref()) {
            ("minecraft:horse", Some("minecraft:donkey")) | ("minecraft:donkey", Some("minecraft:horse")) => "minecraft:mule".to_owned(),
            _ => kind,
        };
        HOLD_JOIN.store(true, Ordering::Relaxed);
        let child = game.spawn_mob(&kind, at, true);
        HOLD_JOIN.store(false, Ordering::Relaxed);
        if let Some(mob) = child.and_then(|id| game.mobs.iter_mut().find(|m| m.id == id)) {
            mob.extra.age = -24000;
        }
        child
    });
    let payload = json!({ "level": level(game), "mother": parent, "father": partner, "breeder": breeder, "egg": eggs, "exp": exp, "item": item, "child": child });
    let refused = bridge::on_refusable(game, "ee_breed", payload);
    if refused {
        game.mobs.retain(|m| Some(m.id) != child);
        for m in game.mobs.iter_mut().filter(|m| m.id == parent || m.id == partner) {
            m.extra.in_love = 0;
        }
    } else {
        *CHILD.lock().unwrap_or_else(|e| e.into_inner()) = child;
    }
    refused
}

/// The child the event made, joining the level now: `Some(None)` if a listener kept it out.
pub(in crate::game) fn take_child(game: &mut Game<'_>) -> Option<Option<u32>> {
    let id = CHILD.lock().unwrap_or_else(|e| e.into_inner()).take()?;
    let index = game.mobs.iter().position(|m| m.id == id)?;
    if mob_joins(game, index) {
        Some(Some(id))
    } else {
        game.mobs.remove(index);
        Some(None)
    }
}

/// `CreatureSpawnEvent`: false if a listener keeps it out.
pub(in crate::game) fn mob_joins(game: &mut Game<'_>, index: usize) -> bool {
    if HOLD_JOIN.load(Ordering::Relaxed) {
        return true;
    }
    let mob = &game.mobs[index];
    let (id, kind, at) = (mob.id, mob.kind.id.to_string(), [mob.position.x, mob.position.y, mob.position.z]);
    JOINING_MOB.store(true, Ordering::Relaxed);
    let joins = game.native_entity_joins(id, &kind, at, false);
    JOINING_MOB.store(false, Ordering::Relaxed);
    joins
}

/// What's joining the level is a mob: `CreatureSpawnEvent` rather than its parent.
static JOINING_MOB: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn joining_mob() -> bool {
    JOINING_MOB.load(Ordering::Relaxed)
}

/// `ShulkerDuplicateEvent` with the new one made: true if refused.
pub(in crate::game) fn shulker_born(game: &mut Game<'_>) -> bool {
    let Some(new) = game.mobs.last().filter(|m| &*m.kind.id == "minecraft:shulker" && bridge::hears("ee_mob")) else {
        return false;
    };
    let (id, at) = (new.id, new.position);
    let near = |m: &&Mob| (m.position.x - at.x).powi(2) + (m.position.y - at.y).powi(2) + (m.position.z - at.z).powi(2);
    let parent = ee::actor_by()["id"].as_u64().or_else(|| {
        game.mobs.iter().filter(|m| &*m.kind.id == "minecraft:shulker" && m.id != id).min_by(|a, b| near(a).total_cmp(&near(b))).map(|m| u64::from(m.id))
    });
    bridge::on_refusable(game, "ee_mob", json!({ "kind": "shulker_new", "level": level(game), "entity": parent, "new": id }))
}

/// What a bolt turns a pig or a villager into is made before it's asked about, and joins after.
pub(in crate::game) fn zap_ahead(kind: &str) -> bool {
    let zap = matches!(kind, "minecraft:zombified_piglin" | "minecraft:witch") && bridge::hears("ee_mob");
    HOLD_JOIN.store(zap, Ordering::Relaxed);
    zap
}

/// `PigZapEvent` and `EntityZapEvent`: a refusal takes the new mob away and keeps the old one.
pub(in crate::game) fn zapped(game: &mut Game<'_>, zap: bool, id: u32, at: [f64; 3], kind: &str) {
    if !zap {
        return;
    }
    HOLD_JOIN.store(false, Ordering::Relaxed);
    let old = game.mobs.iter().find(|m| m.removed && m.position.x == at[0] && m.position.y == at[1] && m.position.z == at[2]).map(|m| (m.id, m.kind.id.to_string()));
    let refused = old.as_ref().is_some_and(|(old, was)| bridge::on_refusable(game, "ee_mob", json!({ "kind": "zap", "level": level(game), "entity": old, "mob": was, "new": kind, "replacement": id })));
    if refused {
        game.mobs.retain(|m| m.id != id);
        if let Some(m) = old.and_then(|(old, _)| game.mobs.iter_mut().find(|m| m.id == old)) {
            m.removed = false;
        }
    } else if let Some(index) = game.mobs.iter().position(|m| m.id == id)
        && !mob_joins(game, index)
    {
        game.mobs.remove(index);
    }
}

/// The next item to land is this mob's own drop.
pub(in crate::game) fn dropping() {
    if bridge::hears("ee_mob") {
        *DROPPER.lock().unwrap_or_else(|e| e.into_inner()) = Some(ee::actor_by());
    }
}

/// `EntityDropItemEvent`, with the item on the ground: true if refused, and the caller takes it up.
pub(in crate::game) fn drop_refused(game: &mut Game<'_>) -> bool {
    let dropper = take_dropper();
    let (Some(dropper), Some(item)) = (dropper, game.dropped.last()) else {
        return false;
    };
    let (id, at, stack) = (item.id, [item.position.x, item.position.y, item.position.z], stack_json(&item.stack));
    bridge::on_refusable(game, "ee_mob", json!({ "kind": "drop", "level": level(game), "at": at, "entity": dropper["id"], "item": id, "stack": stack }))
}

/// `AreaEffectCloudApplyEvent` for the dragon's breath: who it goes on to harm.
pub(in crate::game) fn breath_apply(game: &mut Game<'_>, c: &mut crate::game::dragon::Breath, near: Vec<crate::game::dragon::FightWho>) -> Vec<crate::game::dragon::FightWho> {
    if near.is_empty() || !bridge::hears("ee_cloud") {
        return near;
    }
    if c.id == 0 {
        c.id = fresh_id(game);
    }
    let id = c.id;
    let rows: Vec<Value> = near.iter().map(|who| fight_who(game, *who)).collect();
    *SPARED.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let payload = json!({ "level": level(game), "cloud": id, "at": c.at, "radius": c.radius, "duration": c.duration, "affected": rows });
    let held = std::mem::replace(c, crate::game::dragon::Breath { id: 0, at: [0.0; 3], radius: 0.0, age: 0, duration: 0, from_dragon: false, potion: None });
    game.breath_clouds.push(held);
    let refused = bridge::on_refusable(game, "ee_cloud", payload);
    if let Some(i) = game.breath_clouds.iter().position(|cloud| cloud.id == id) {
        *c = game.breath_clouds.remove(i);
    }
    let spared = SPARED.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or_default();
    if refused {
        return Vec::new();
    }
    near.into_iter().zip(rows).filter(|(_, row)| !spared.contains(row)).map(|(who, _)| who).collect()
}

/// What a mob gives a player with no hit: dolphins and nautili have causes of their own.
pub(in crate::game) fn mob_gives(game: &Game<'_>, effect: &str) {
    let cause = match effect {
        "minecraft:dolphins_grace" => "DOLPHIN",
        "minecraft:breath_of_the_nautilus" => "NAUTILUS",
        _ => "ATTACK",
    };
    ee::effect_from(ee::actor_by()["id"].as_u64().map(|id| Who::Mob(id as u32)));
    cause_next(game, cause);
}

/// `give_player_effect` calls it `ATTACK` unless told otherwise, here, for the next one.
static CAUSE_NEXT: Mutex<Option<&'static str>> = Mutex::new(None);

pub(in crate::game) fn cause_next(_: &Game<'_>, cause: &'static str) {
    *CAUSE_NEXT.lock().unwrap_or_else(|e| e.into_inner()) = Some(cause);
}

/// `EntityTeleportEvent` for `/tp`: where it goes, `None` if refused. A player has their own event.
pub(in crate::game) fn command_teleport(game: &mut Game<'_>, target: crate::game::commands::Target, to: [f64; 3]) -> Option<[f64; 3]> {
    use crate::game::commands::Target;
    let id = match target {
        Target::Player | Target::Guest(_) => return Some(to),
        Target::Mob(id) | Target::Item(id) | Target::Orb(id) | Target::Projectile(id) | Target::Stand(id) | Target::Frame(id) | Target::Painting(id) | Target::Boat(id) | Target::Cart(id) | Target::Other(id) | Target::Crystal(id) | Target::Dragon(id) => id,
    };
    if !bridge::hears("ee_teleport") {
        return Some(to);
    }
    let from = game.target_position(target).unwrap_or_default();
    let entity = row_for(game, id);
    let (refused, after) = fire_lent(game, "ee_teleport", json!({ "entity": entity, "from": from, "to": to }));
    let at = |i: usize| after["to"][i].as_f64();
    if refused { None } else { Some([at(0)?, at(1)?, at(2)?]) }
}

/// A bow's draw as it lets go, for the shot it's about to make.
static FORCE: Mutex<Option<f32>> = Mutex::new(None);

pub(in crate::game) fn note_force(power: f32) {
    *FORCE.lock().unwrap_or_else(|e| e.into_inner()) = Some(power);
}

/// An id for what the game only numbers once a server's players track it.
fn fresh_id(game: &mut Game<'_>) -> u32 {
    game.next_entity += 1;
    game.next_entity
}

/// What's leaving the level as its event is heard: the list no longer holds it, and a listener still reads it.
static DYING: Mutex<Option<(u32, String, [f64; 3], Value)>> = Mutex::new(None);
/// A lingering potion's cloud went out ahead of the splash.
static CLOUD_OUT: AtomicBool = AtomicBool::new(false);

fn dying_id() -> Option<u32> {
    DYING.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|dying| dying.0)
}

fn dying_row(id: u32) -> Option<(String, [f64; 3], Value)> {
    DYING.lock().unwrap_or_else(|e| e.into_inner()).as_ref().filter(|dying| dying.0 == id).map(|dying| (dying.1.clone(), dying.2, dying.3.clone()))
}

pub(in crate::game) fn potion_dying(id: u32, stack: &ItemStack, at: Vec3, velocity: Vec3) {
    let kind = match &*stack.item {
        "minecraft:lingering_potion" => "minecraft:lingering_potion",
        "minecraft:experience_bottle" => "minecraft:experience_bottle",
        _ => "minecraft:splash_potion",
    };
    let item = crate::game::commands::paper_entities::tag_json(&horizon_client::save::item_to_nbt(stack));
    let data = json!({ "Item": item, "Motion": [velocity.x, velocity.y, velocity.z] });
    *DYING.lock().unwrap_or_else(|e| e.into_inner()) = Some((id, kind.to_owned(), [at.x, at.y, at.z], data));
}

pub(in crate::game) fn ball_dying(id: u32, at: [f64; 3], velocity: [f64; 3]) {
    *DYING.lock().unwrap_or_else(|e| e.into_inner()) = Some((id, "minecraft:dragon_fireball".to_owned(), at, json!({ "Motion": velocity })));
}

/// The list has no bolt while they tick: this one is read from here while it strikes.
pub(in crate::game) fn bolt_strikes(game: &mut Game<'_>, bolt: &mut crate::game::lightning::Bolt) {
    if bolt.id == 0 {
        bolt.id = fresh_id(game);
    }
    let data = json!({ "Life": bolt.life, "Flashes": bolt.flashes, "Effect": bolt.visual_only });
    *DYING.lock().unwrap_or_else(|e| e.into_inner()) = Some((bolt.id, "minecraft:lightning_bolt".to_owned(), bolt.at, data));
    ee::set_bolt(Some(bolt.id));
}

pub(in crate::game) fn dying_over() {
    *DYING.lock().unwrap_or_else(|e| e.into_inner()) = None;
    ee::set_bolt(None);
}

/// True once: the splash's cloud stands already.
pub(in crate::game) fn cloud_out() -> bool {
    CLOUD_OUT.swap(false, Ordering::Relaxed)
}

/// What `entity_kept` doesn't list: a dragon's fireball, a hook, or what's on its way out.
pub(in crate::game) fn kept_row(game: &Game<'_>, id: u32) -> Option<(String, [f64; 3], Value)> {
    if let Some(row) = dying_row(id) {
        return Some(row);
    }
    if let Some(ball) = game.dragon_fireballs.iter().find(|ball| ball.id == id) {
        let v = ball.velocity;
        return Some(("minecraft:dragon_fireball".to_owned(), [ball.position.x, ball.position.y, ball.position.z], json!({ "Motion": [v.x, v.y, v.z] })));
    }
    let hook = game.hook.as_ref().filter(|hook| hook.id == id)?;
    let v = hook.velocity;
    Some(("minecraft:fishing_bobber".to_owned(), [hook.position.x, hook.position.y, hook.position.z], json!({ "Motion": [v.x, v.y, v.z] })))
}

pub(in crate::game) fn kept_ids(game: &Game<'_>) -> Vec<u32> {
    game.dragon_fireballs.iter().map(|ball| ball.id).chain(game.hook.as_ref().map(|hook| hook.id)).collect()
}

/// `/data merge` for those: `None` for what isn't one, false for a field the game doesn't keep.
pub(in crate::game) fn kept_merge(game: &mut Game<'_>, id: u32, patch: &horizon_world::nbt::Tag) -> Option<bool> {
    if let Some(ball) = game.dragon_fireballs.iter_mut().find(|ball| ball.id == id) {
        let moved = patch.get("Motion").is_some();
        if moved {
            ball.velocity = horizon_client::entity_nbt::motion(patch);
        }
        return Some(moved);
    }
    None
}

/// `remove`, `kill` and `teleport` on those: true if it was one.
pub(in crate::game) fn kept_action(game: &mut Game<'_>, query: &Value, id: u32) -> bool {
    let action = query["action"].as_str().unwrap_or_default();
    if id == 0 || !matches!(action, "remove" | "kill" | "teleport") {
        return false;
    }
    let at = (0..3).map(|i| query["value"][i].as_f64()).collect::<Option<Vec<f64>>>().map(|at| Vec3::new(at[0], at[1], at[2]));
    let dimension = query["dimension"].as_str().unwrap_or("minecraft:overworld").to_owned();
    game.native_in_level(&dimension, |game| {
        let hook = game.hook.as_ref().is_some_and(|hook| hook.id == id);
        if action != "teleport" {
            let held = kept_ids(game).contains(&id) || game.potions.thrown.iter().any(|potion| potion.id == id) || game.breath_clouds.iter().any(|cloud| cloud.id == id);
            game.breath_clouds.retain(|cloud| cloud.id != id);
            game.dragon_fireballs.retain(|ball| ball.id != id);
            game.potions.thrown.retain(|potion| potion.id != id);
            if hook {
                game.hook = None;
            }
            return held;
        }
        let Some(at) = at else {
            return false;
        };
        if let Some(cloud) = game.breath_clouds.iter_mut().find(|cloud| cloud.id == id) {
            cloud.at = [at.x, at.y, at.z];
            return true;
        }
        if let Some(ball) = game.dragon_fireballs.iter_mut().find(|ball| ball.id == id) {
            ball.position = at;
            ball.previous = at;
            return true;
        }
        if let Some(potion) = game.potions.thrown.iter_mut().find(|potion| potion.id == id) {
            potion.position = at;
            return true;
        }
        if let Some(hook) = game.hook.as_mut().filter(|hook| hook.id == id) {
            hook.position = at;
            hook.previous = at;
            return true;
        }
        false
    }).unwrap_or(false)
}

/// `VillagerAcquireTradeEvent`, for the offers a villager has just come by, from `from` on.
pub(in crate::game) fn offers_made(game: &mut Game<'_>, index: usize, from: usize) {
    if !bridge::hears("ee_acquire") {
        return;
    }
    let id = game.mobs[index].id;
    let count = game.mobs[index].extra.trade.as_ref().map_or(0, |trade| trade.offers.len());
    let entity = row_for(game, id);
    let mut refused = Vec::new();
    for offer in from..count {
        if fire_lent(game, "ee_acquire", json!({ "entity": entity, "index": offer })).0 {
            refused.push(offer);
        }
    }
    if let Some(trade) = game.mobs.iter_mut().find(|m| m.id == id).and_then(|m| m.extra.trade.as_mut()) {
        for offer in refused.into_iter().rev() {
            trade.offers.remove(offer);
        }
    }
}

/// `EntityMountEvent` for the player in hand: true if a listener keeps them off.
pub(in crate::game) fn mount_refused(game: &mut Game<'_>, vehicle: u32) -> bool {
    seat_refused(game, "ee_mount", vehicle)
}

/// `EntityDismountEvent` for the player in hand: true if a listener keeps them on.
pub(in crate::game) fn dismount_refused(game: &mut Game<'_>, vehicle: u32) -> bool {
    seat_refused(game, "ee_dismount", vehicle)
}

/// `EntityDismountEvent` with the vehicle gone from under them, or them from over it: not to be refused.
pub(in crate::game) fn vehicle_gone(game: &mut Game<'_>) {
    let Some(vehicle) = game.riding.filter(|_| bridge::hears("ee_dismount")) else {
        return;
    };
    let player = game.native_acting_id();
    let name = game.acting_player_name().to_owned();
    let mount = row_for(game, vehicle);
    fire_lent(game, "ee_dismount", json!({ "rider": { "uuid": uuid(player), "name": name, "player": true, "dimension": level(game) }, "mount": mount, "forced": true }));
}

fn seat_refused(game: &mut Game<'_>, hook: &str, vehicle: u32) -> bool {
    if !bridge::hears(hook) {
        return false;
    }
    let player = game.native_acting_id();
    let name = game.acting_player_name().to_owned();
    let mount = row_for(game, vehicle);
    fire_lent(game, hook, json!({ "rider": { "uuid": uuid(player), "name": name, "player": true, "dimension": level(game) }, "mount": mount })).0
}

/// Blocks with something to do with what's inside them.
/// `ItemSpawnEvent`: the game's own `spawn_item` is run here first, so that the item stands while
/// listeners look at it; true then skips the call this came from.
pub(in crate::game) fn item_spawn(game: &mut Game<'_>, stack: &ItemStack, position: [f64; 3], velocity: [f64; 3], pickup_delay: u32) -> bool {
    if AGAIN_ITEM.swap(false, Ordering::Relaxed) || death_window() {
        return false;
    }
    AGAIN_ITEM.store(true, Ordering::Relaxed);
    game.spawn_item(stack.clone(), position, velocity, pickup_delay);
    let id = game.next_entity;
    // `EntityDropItemEvent` comes first, and may have taken it away.
    if !game.dropped.iter().any(|item| item.id == id) {
        return true;
    }
    if bridge::on_refusable(game, "ee_item", json!({ "level": level(game), "entity": id, "at": position })) {
        game.dropped.retain(|item| item.id != id);
    }
    true
}

static AGAIN_ITEM: AtomicBool = AtomicBool::new(false);

/// A totem refused: the blow stands.
pub(in crate::game) fn no_totem(game: &mut Game<'_>) {
    if let Some(p) = game.player.as_mut() {
        p.health.totem_used = false;
        let _ = p.set_health(0.0, false);
    }
}

/// What a listener of `EntityExplodeEvent` left: refused, the blocks, the yield.
static BLAST: Mutex<Option<(bool, Vec<[i32; 3]>, f32)>> = Mutex::new(None);
/// The yield for this explosion's blocks, when a listener changed it.
static YIELD: Mutex<Option<f32>> = Mutex::new(None);
static AGAIN_BLOCK: AtomicBool = AtomicBool::new(false);
/// What's blowing up, as far as the game has said: (its id, its kind).
static EXPLODER: Mutex<Option<(u32, String)>> = Mutex::new(None);
/// The primed TNT as this tick's fuses burn down, by where it is.
static PRIMED: Mutex<Vec<(u32, [f64; 3])>> = Mutex::new(Vec::new());

fn exploder_now() -> Option<(u32, String)> {
    EXPLODER.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn exploder(id: u32, kind: &str) {
    *EXPLODER.lock().unwrap_or_else(|e| e.into_inner()) = Some((id, kind.to_owned()));
}

pub(in crate::game) fn blast_by_shot(p: &Projectile) -> bool {
    match p.kind {
        ProjectileKind::LargeFireball { .. } | ProjectileKind::SmallFireball | ProjectileKind::WitherSkull { .. } => exploder(p.id, p.kind.id()),
        _ => {}
    }
    true
}

pub(in crate::game) fn blast_by_cart(id: u32) -> bool {
    exploder(id, "minecraft:tnt_minecart");
    true
}

pub(in crate::game) fn blast_by_crystal(id: u32) -> bool {
    exploder(id, "minecraft:end_crystal");
    true
}

pub(in crate::game) fn note_tnt(primed: Vec<(u32, [f64; 3])>) -> bool {
    *PRIMED.lock().unwrap_or_else(|e| e.into_inner()) = primed;
    true
}

/// Who it was, once it's over: nothing else is theirs.
pub(in crate::game) fn blast_over(game: &mut Game<'_>) -> bool {
    *EXPLODER.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let paintings = HELD_PAINTINGS.with(|held| std::mem::take(&mut *held.borrow_mut()));
    game.paintings.extend(paintings);
    let frames = HELD_FRAMES.with(|held| std::mem::take(&mut *held.borrow_mut()));
    game.item_frames.extend(frames);
    true
}

thread_local! {
    /// What a listener saved from a blast: out of the game's loops until the blast is over.
    static HELD_PAINTINGS: RefCell<Vec<horizon_client::entities::Painting>> = const { RefCell::new(Vec::new()) };
    static HELD_FRAMES: RefCell<Vec<horizon_client::entities::ItemFrame>> = const { RefCell::new(Vec::new()) };
}

/// Each painting and frame a blast reaches, ahead of the game's loops: one a listener keeps sits
/// them out. A refused break inside those loops would never end them.
pub(in crate::game) fn decorations(game: &mut Game<'_>, center: [f64; 3], diameter: f64) -> bool {
    use crate::game::explosion::blast_reaches;
    let remover = exploder_now().map_or(Value::Null, |(id, kind)| json!({ "id": id, "kind": kind }));
    let here = level(game);
    let paintings: Vec<u32> = game.paintings.iter().filter(|p| {
        let (lo, hi) = p.bounds();
        blast_reaches(center, diameter, [0, 1, 2].map(|k| (lo[k] + hi[k]) / 2.0))
    }).map(|p| p.id).collect();
    for id in paintings {
        let payload = json!({ "ee_blast": true, "level": here, "entity": id, "by_player": false, "survives": true, "remover": remover });
        if bridge::on_refusable(game, "ee_decor", payload)
            && let Some(i) = game.paintings.iter().position(|p| p.id == id)
        {
            let held = game.paintings.remove(i);
            HELD_PAINTINGS.with(|all| all.borrow_mut().push(held));
        }
    }
    let frames: Vec<u32> = game.item_frames.iter().filter(|f| !f.fixed && blast_reaches(center, diameter, f.pos.map(|v| f64::from(v) + 0.5))).map(|f| f.id).collect();
    for id in frames {
        let payload = json!({ "ee_blast": true, "level": here, "entity": id, "by_player": false, "survives": true, "remover": remover });
        if bridge::on_refusable(game, "ee_decor", payload)
            && let Some(i) = game.item_frames.iter().position(|f| f.id == id)
        {
            let held = game.item_frames.remove(i);
            HELD_FRAMES.with(|all| all.borrow_mut().push(held));
        }
    }
    true
}
/// What the explosion under way does to blocks: `ExplosionResult`, and `Explosion.yield` at the start.
static BLAST_KIND: Mutex<(&'static str, f32)> = Mutex::new(("DESTROY", 1.0));

/// `EntityExplodeEvent`, as the game has its blocks: they're the game's own list, edited in place.
pub(in crate::game) fn detonate(game: &mut Game<'_>, center: [f64; 3], blocks: &mut Vec<[i32; 3]>) -> bool {
    let (result, power) = *BLAST_KIND.lock().unwrap_or_else(|e| e.into_inner());
    let entity = exploder_now();
    *BLAST.lock().unwrap_or_else(|e| e.into_inner()) = None;
    *YIELD.lock().unwrap_or_else(|e| e.into_inner()) = None;
    bridge::on_trigger(game, "ee_detonate", json!({ "ee_blast": true, "level": level(game), "center": center, "blocks": blocks, "entity": entity.as_ref().map(|(id, _)| *id).filter(|id| *id != 0), "entity_kind": entity.map(|(_, kind)| kind), "result": result, "yield": power }));
    if let Some((refused, kept, wanted)) = BLAST.lock().unwrap_or_else(|e| e.into_inner()).take() {
        *blocks = if refused { Vec::new() } else { kept };
        if (wanted - power).abs() > f32::EPSILON {
            *YIELD.lock().unwrap_or_else(|e| e.into_inner()) = Some(wanted);
        }
    }
    true
}

/// Whether this block's drops go by a yield a listener set: true runs it again with that.
pub(in crate::game) fn yield_set() -> bool {
    !AGAIN_BLOCK.swap(false, Ordering::Relaxed) && YIELD.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}

/// The radius that gives a yield: an explosion's drops go at one in `radius`.
pub(in crate::game) fn yield_radius() -> f32 {
    AGAIN_BLOCK.store(true, Ordering::Relaxed);
    let wanted = YIELD.lock().unwrap_or_else(|e| e.into_inner()).unwrap_or(1.0);
    if wanted <= 0.0 { f32::MAX } else { 1.0 / wanted }
}

/// A lingering cloud, on the turns it could reach someone: `tick_potion_cloud`'s own checks, as
/// they'd stand a tick on.
pub(in crate::game) fn cloud(game: &mut Game<'_>, c: &mut crate::game::dragon::Breath, players: &[crate::game::dragon::FightPlayer], able: &[u32]) -> bool {
    const WAIT: i32 = 10;
    note_cause(game, "AREA_EFFECT_CLOUD");
    if AGAIN_CLOUD.swap(false, Ordering::Relaxed) {
        return false;
    }
    *SPARED.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let age = c.age + 1;
    if c.potion.is_none() || age < WAIT || age - WAIT >= c.duration || age % 5 != 0 {
        return false;
    }
    let radius = c.radius - 3.0 / 600.0;
    let r = f64::from(radius);
    let area = Aabb::new(Vec3::new(c.at[0] - r, c.at[1], c.at[2] - r), Vec3::new(c.at[0] + r, c.at[1] + 0.5, c.at[2] + r));
    let near = |x: f64, z: f64| (x - c.at[0]) * (x - c.at[0]) + (z - c.at[2]) * (z - c.at[2]) <= r * r;
    let mut affected = Vec::new();
    for player in players.iter().filter(|player| player.bounds.intersects(area) && near(player.position.x, player.position.z)) {
        affected.push(fight_who(game, player.who));
    }
    affected.extend(game.mobs.iter().filter(|m| m.alive() && able.contains(&m.id) && m.bounds().intersects(area) && near(m.position.x, m.position.z)).map(|m| json!({ "id": m.id })));
    if affected.is_empty() {
        return false;
    }
    // It stands in the list for this, so that what a listener sets on it holds.
    if c.id == 0 {
        c.id = fresh_id(game);
    }
    let id = c.id;
    let payload = json!({ "level": level(game), "cloud": id, "at": c.at, "radius": c.radius, "duration": c.duration, "affected": affected });
    let held = std::mem::replace(c, crate::game::dragon::Breath { id: 0, at: [0.0; 3], radius: 0.0, age: 0, duration: 0, from_dragon: false, potion: None });
    game.breath_clouds.push(held);
    let refused = bridge::on_refusable(game, "ee_cloud", payload);
    if let Some(i) = game.breath_clouds.iter().position(|cloud| cloud.id == id) {
        *c = game.breath_clouds.remove(i);
    }
    if refused {
        *SPARED.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
    refused || SPARED.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}

/// Who a listener took off the cloud's list: `None` for a cloud refused whole.
static SPARED: Mutex<Option<Vec<Value>>> = Mutex::new(None);
static AGAIN_CLOUD: AtomicBool = AtomicBool::new(false);

/// The ids of the mobs a listener spared, and the guests' connections: the row marks them as already
/// reached, and runs the cloud's turn again.
/// The knock a listener asked for in place of the game's: (power, x, z).
static KNOCK: Mutex<Option<(f64, f64, f64)>> = Mutex::new(None);
static AGAIN_KNOCK: AtomicBool = AtomicBool::new(false);

pub(in crate::game) fn knock(game: &mut Game<'_>, power: f64, xd: f64, zd: f64) -> bool {
    if AGAIN_KNOCK.swap(false, Ordering::Relaxed) {
        return false;
    }
    *KNOCK.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let player = game.native_acting_id();
    let eid = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|g| g.uuid == player).map(|g| g.entity_id));
    let refused = bridge::on_refusable(game, "ee_knock", json!({ "power": power, "x": xd, "z": zd, "eid": eid }));
    if refused {
        *KNOCK.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
    refused || KNOCK.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}

pub(in crate::game) fn knock_again(game: &mut Game<'_>) {
    if let Some((power, xd, zd)) = KNOCK.lock().unwrap_or_else(|e| e.into_inner()).take() {
        AGAIN_KNOCK.store(true, Ordering::Relaxed);
        game.knock_back_player(power, xd, zd);
    }
}

pub(in crate::game) fn owner_of(m: &Mob) -> Value {
    m.extra.owner.map_or(Value::Null, |owner| json!(uuid(horizon_client::lan::playerdata::ints_uuid(owner))))
}

pub(in crate::game) fn spared(game: &Game<'_>) -> Option<(Vec<u32>, Vec<u32>)> {
    let list = SPARED.lock().unwrap_or_else(|e| e.into_inner()).take()?;
    AGAIN_CLOUD.store(true, Ordering::Relaxed);
    let mobs = list.iter().filter_map(|who| who["id"].as_u64()).map(|id| id as u32).collect();
    let guests = list.iter().filter_map(|who| who["uuid"].as_str()).filter_map(|player| game.lan.as_ref()?.hub.guests.iter().find(|g| uuid(g.uuid) == player).map(|g| g.conn)).collect();
    Some((mobs, guests))
}

fn players_state(game: &mut Game<'_>) -> Value {
    let Some(host) = game.lan.as_ref() else {
        return Value::Null;
    };
    let rows = host.hub.guests.iter().filter(|g| g.placed).map(|g| json!({ "uuid": uuid(g.uuid), "dimension": g.dimension, "vehicle": g.vehicle, "gliding": g.fall_flying, "pose": pose_name(g.pose()), "food": g.food, "saturation": g.saturation, "health": g.health, "exhaustion": g.motion.as_ref().map(|motion| motion.health.food.exhaustion), "arrows": game.guest_combat.get(&g.conn).map(|combat| combat.stuck.arrows) }));
    Value::Array(rows.collect())
}

fn pose_name(pose: impl std::fmt::Debug) -> String {
    let text = format!("{pose:?}");
    match text.as_str() {
        "FallFlying" => "FALL_FLYING".to_owned(),
        "SpinAttack" => "SPIN_ATTACK".to_owned(),
        "LongJumping" => "LONG_JUMPING".to_owned(),
        "UsingTongue" => "USING_TONGUE".to_owned(),
        "Crouching" => "SNEAKING".to_owned(),
        _ => text.to_uppercase(),
    }
}

fn items(game: &mut Game<'_>, dimension: &str) -> Value {
    game.native_in_level(dimension, |game| Value::Array(game.dropped.iter().map(|item| json!({ "id": item.id, "stack": stack_json(&item.stack), "at": [item.position.x, item.position.y, item.position.z], "age": item.age })).collect()))
        .unwrap_or(Value::Null)
}

fn with_mob<R>(game: &mut Game<'_>, query: &Value, f: impl FnOnce(&mut Mob) -> R) -> Option<R> {
    let id = query["id"].as_u64()? as u32;
    let dimension = query["dimension"].as_str().unwrap_or("minecraft:overworld").to_owned();
    game.native_in_level(&dimension, |game| game.mobs.iter_mut().find(|m| m.id == id).map(f)).flatten()
}

fn who_from(value: &Value) -> Option<Who> {
    if value.is_null() {
        return None;
    }
    Some(value["id"].as_u64().map_or(Who::Player, |id| Who::Mob(id as u32)))
}

/// `None` for an op that isn't this family's.
pub(in crate::game) fn query(game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    let answer = match op {
        "ee_lunge_power" => {
            LUNGE.store(query["power"].as_i64()? as i32, Ordering::Relaxed);
            Some(json!(true))?
        }
        "ee_mobs" => mob_state(game, query["dimension"].as_str().unwrap_or("minecraft:overworld")),
        "ee_blast" => {
            let blocks = query["blocks"].as_array().into_iter().flatten().filter_map(|row| Some([row[0].as_i64()? as i32, row[1].as_i64()? as i32, row[2].as_i64()? as i32])).collect();
            *BLAST.lock().unwrap_or_else(|e| e.into_inner()) = Some((query["cancelled"].as_bool().unwrap_or(false), blocks, query["yield"].as_f64().unwrap_or(1.0) as f32));
            json!(true)
        }
        "ee_unconvert" => {
            let to = query["to"].as_u64()? as u32;
            let dimension = query["dimension"].as_str()?.to_owned();
            let from = CONVERTING.with(|slot| slot.borrow_mut().take())?;
            json!(game.native_in_level(&dimension, |game| {
                game.mobs.retain(|m| m.id != to);
                game.forget_entity(to);
                let mut back = *from;
                back.removed = false;
                let id = back.id;
                for rider in game.mobs.iter_mut() {
                    if rider.extra.mount == Some(to) {
                        rider.extra.mount = Some(id);
                    }
                }
                game.mobs.retain(|m| m.id != id);
                game.mobs.push(back);
            }).is_some())
        }
        "ee_witch" => {
            let (stop, potion) = (query["stop"].as_bool().unwrap_or(false), query["potion"].as_str().map(str::to_owned));
            json!(with_mob(game, query, |m| {
                if stop {
                    m.extra.using_item = 0;
                    m.extra.flag = false;
                    let _ = m.extra.take_main_hand();
                } else if let Some(potion) = potion {
                    m.extra.set_main_hand(Some(potion.into()));
                }
            }).is_some())
        }
        "ee_breed_xp" => {
            let tick = game.world.game_time as i64;
            *BREED_XP.lock().unwrap_or_else(|e| e.into_inner()) = Some((query["exp"].as_u64().unwrap_or(0) as u32, tick));
            json!(true)
        }
        "ee_set_head" => {
            HEADS.lock().unwrap_or_else(|e| e.into_inner()).push((query["id"].as_u64()?, query["dimension"].as_str()?.to_owned(), query["item"].as_str()?.to_owned()));
            json!(true)
        }
        "ee_clouds" => {
            let dimension = query["dimension"].as_str()?.to_owned();
            game.native_in_level(&dimension, |game| {
                let dragon = game.dragon.as_ref().map(|dragon| dragon.id);
                Value::Array(game.breath_clouds.iter().filter(|c| c.potion.is_none()).map(|c| json!({ "at": c.at, "radius": c.radius, "duration": c.duration, "from_dragon": c.from_dragon, "dragon": dragon })).collect())
            }).unwrap_or(Value::Null)
        }
        "ee_cloud_gone" => {
            let dimension = query["dimension"].as_str()?.to_owned();
            let at = [query["at"][0].as_f64()?, query["at"][1].as_f64()?, query["at"][2].as_f64()?];
            json!(game.native_in_level(&dimension, |game| game.breath_clouds.retain(|c| c.at != at)).is_some())
        }
        "ee_orbs" => {
            let dimension = query["dimension"].as_str()?.to_owned();
            game.native_in_level(&dimension, |game| Value::Array(game.orbs.iter().map(|orb| json!({ "id": orb.id, "value": orb.value, "count": orb.count, "at": [orb.position.x, orb.position.y, orb.position.z] })).collect())).unwrap_or(Value::Null)
        }
        "ee_mob_change" => {
            *MOB_CHANGE.lock().unwrap_or_else(|e| e.into_inner()) = query.clone();
            json!(true)
        }
        "ee_arrows" => {
            let host = game.lan.as_ref()?;
            let player = query["uuid"].as_str()?;
            let conn = host.hub.guests.iter().find(|g| uuid(g.uuid) == player)?.conn;
            game.guest_combat.entry(conn).or_default().stuck.arrows = query["count"].as_i64().unwrap_or(0) as i32;
            json!(true)
        }
        "ee_explosion" => {
            *EXPLOSION.lock().unwrap_or_else(|e| e.into_inner()) = Some((query["cancelled"].as_bool().unwrap_or(false), query["radius"].as_f64().unwrap_or(0.0) as f32, query["fire"].as_bool().unwrap_or(false)));
            json!(true)
        }
        "ee_lead_drop" => {
            *LEAD_DROP.lock().unwrap_or_else(|e| e.into_inner()) = Some((query["cancelled"].as_bool().unwrap_or(false), query["drop"].as_bool().unwrap_or(false)));
            json!(true)
        }
        "ee_death_apply" => death_apply(game, query)?,
        "ee_player_dying" => {
            let asked = DeathAsked {
                keep: query["keep"].as_bool().unwrap_or(false),
                no_exp: query["no_exp"].as_bool().unwrap_or(false),
                drops: query["drops"].as_array().map(|stacks| stacks.iter().filter_map(stack_of).collect()),
                revive: query["revive"].as_f64().map(|health| health as f32),
            };
            *DEATH_ASKED.lock().unwrap_or_else(|e| e.into_inner()) = Some(asked);
            json!(true)
        }
        "ee_keep_done" => {
            for (player, text) in std::mem::take(&mut *SCREENS.lock().unwrap_or_else(|e| e.into_inner())) {
                if let Some(host) = game.lan.as_mut()
                    && let Some(guest) = host.hub.guests.iter().find(|g| uuid(g.uuid) == player)
                {
                    let (conn, id) = (guest.conn, guest.entity_id);
                    let message = horizon_client::lan::data::component_nbt(&Text::literal(text).component());
                    host.hub.send_to(conn, &horizon_protocol::packets::play::Clientbound::PlayerCombatKill(horizon_protocol::packets::play::PlayerCombatKill { id, message }));
                }
            }
            if KEEP_RULE.swap(false, Ordering::Relaxed) {
                set_keep_rule(game, false);
            }
            json!(true)
        }
        "ee_players" => players_state(game),
        "ee_items" => items(game, query["dimension"].as_str().unwrap_or("minecraft:overworld")),
        "ee_item_age" => {
            let (id, dimension) = (query["id"].as_u64()? as u32, query["dimension"].as_str()?.to_owned());
            let age = query["age"].as_u64().unwrap_or(0) as u32;
            json!(game.native_in_level(&dimension, |game| game.dropped.iter_mut().find(|item| item.id == id).map(|item| item.age = age)).flatten().is_some())
        }
        "ee_intensity" => {
            let who = query["who"].clone();
            let mut all = INTENSITIES.lock().unwrap_or_else(|e| e.into_inner());
            all.retain(|(row, _)| *row != who);
            all.push((who, query["intensity"].as_f64().unwrap_or(0.0)));
            json!(true)
        }
        "ee_cloud_spare" => {
            *SPARED.lock().unwrap_or_else(|e| e.into_inner()) = Some(query["who"].as_array().cloned().unwrap_or_default());
            json!(true)
        }
        "ee_knock_set" => {
            *KNOCK.lock().unwrap_or_else(|e| e.into_inner()) = Some((query["power"].as_f64()?, query["x"].as_f64()?, query["z"].as_f64()?));
            json!(true)
        }
        "ee_wet" => {
            WET.lock().unwrap_or_else(|e| e.into_inner()).push(query["who"]["id"].as_u64()? as u32);
            json!(true)
        }
        "ee_mob_at" => {
            let dimension = query["dimension"].as_str()?.to_owned();
            let pos = [query["pos"][0].as_i64()? as i32, query["pos"][1].as_i64()? as i32, query["pos"][2].as_i64()? as i32];
            let area = Aabb::new(Vec3::new(f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2])), Vec3::new(f64::from(pos[0] + 1), f64::from(pos[1]) + 1.0, f64::from(pos[2] + 1)));
            game.native_in_level(&dimension, |game| game.mobs.iter().find(|m| m.alive() && m.bounds().intersects(area)).map(|m| m.id)).flatten().map_or(Value::Null, |id| json!(id))
        }
        "ee_death_screen" => {
            SCREENS.lock().unwrap_or_else(|e| e.into_inner()).push((query["uuid"].as_str()?.to_owned(), query["text"].as_str()?.to_owned()));
            json!(true)
        }
        "ee_keep_ammo" => {
            // `release_bow` takes one from this slot after the shot: it finds one more there.
            // A cancelled shot has still drawn its arrow, as `BowItem.draw` does before the event.
            if query["refund"] != false {
                let slot = game.projectile_slot("minecraft:bow")?;
                if let Some(stack) = game.stack_in_mut(slot).as_mut() {
                    stack.count += 1;
                }
            }
            SKIP_WEAR.store(query["cancelled"].as_bool().unwrap_or(false), Ordering::Relaxed);
            json!(true)
        }
        "ee_give" => {
            let stack = stack_of(&query["stack"])?;
            if let Some(left) = game.give(stack) {
                let at = game.player.as_ref().map_or([0.0; 3], |p| [p.position.x, p.position.y, p.position.z]);
                game.spawn_item(left, at, [0.0; 3], 10);
            }
            json!(true)
        }
        _ => return None,
    };
    Some(answer)
}

/// A stack the SDK sent: its item, count and components, each as SNBT or as JSON text.
fn stack_of(value: &Value) -> Option<ItemStack> {
    use horizon_world::nbt::{Compound, Tag, parse_snbt};
    let mut components = Compound::new();
    for (key, snbt) in value["components"].as_object().into_iter().flatten() {
        if let Ok((tag, _)) = parse_snbt(snbt.as_str()?) {
            components.insert(key.as_str().into(), tag);
        }
    }
    let tag = Tag::compound([
        ("id", Tag::String(value["item"].as_str()?.into())),
        ("count", Tag::Int(i32::try_from(value["count"].as_i64().unwrap_or(1)).unwrap_or(1))),
        ("components", Tag::Compound(components)),
    ]);
    horizon_client::save::item_from_nbt(&tag)
}
