//! Family entity: what `Player`, `HumanEntity` and the entity classes do past their saved tags: the live state of
//! a mob (ops `entity_get`, `entity_set`, `entity_act` in `paper_fam_entity.rs`), the entities no `/data` reaches,
//! and the small classes around them. Saved tags are in `entities.rs`.
#![allow(missing_docs)]
use crate::entities::*;
use crate::entity::{Entity, EntityClass};
use crate::util::Vector;
use crate::fam_ev_entity::Actor;
use crate::{Component, ItemStack, Location, OfflinePlayer, Player, World};
use horizon_mod::{Value, query};
use serde_json::json;

fn ask(entity: &Entity, op: &str, key: &str, value: Value) -> Value {
    query(json!({ "op": op, "id": entity.get_entity_id(), "dimension": entity.get_world().dimension(), "key": key, "value": value }))
}

fn get(entity: &Entity, key: &str) -> Value {
    ask(entity, "entity_get", key, Value::Null)
}

fn flag(entity: &Entity, key: &str) -> bool {
    get(entity, key).as_bool().unwrap_or(false)
}

fn number(entity: &Entity, key: &str) -> i32 {
    get(entity, key).as_i64().unwrap_or(0) as i32
}

fn put(entity: &Entity, key: &str, value: Value) -> bool {
    ask(entity, "entity_set", key, value).as_bool().unwrap_or(false)
}

fn act(entity: &Entity, key: &str, fields: Value) -> Value {
    let mut request = fields;
    request["op"] = json!("entity_act");
    request["id"] = json!(entity.get_entity_id());
    request["dimension"] = json!(entity.get_world().dimension());
    request["key"] = json!(key);
    query(request)
}

fn done(entity: &Entity, key: &str, fields: Value) -> bool {
    act(entity, key, fields).as_bool().unwrap_or(false)
}

fn place(entity: &Entity, at: &Value) -> Option<Location> {
    let spot = |i: usize| at.get(i)?.as_f64();
    Some(Location::new(entity.get_world(), spot(0)?, spot(1)?, spot(2)?, 0.0, 0.0))
}

fn block_of(at: &Location) -> Value {
    json!([at.get_block_x(), at.get_block_y(), at.get_block_z()])
}

/// An entity in the same level as `near`, by the id the game gave.
fn neighbour(near: &Entity, value: &Value) -> Option<Entity> {
    Some(Entity::of(value["entity"].as_u64()?, &near.get_world()))
}

fn target(entity: &Entity) -> Value {
    json!({ "entity": entity.get_entity_id() })
}

fn player_target(player: &Player) -> Value {
    json!({ "player": true, "uuid": words(player.uuid()) })
}

/// A UUID as the four words a save holds.
fn words(uuid: &str) -> Value {
    let hex = uuid.replace('-', "");
    let word = |i: usize| hex.get(i * 8..i * 8 + 8).and_then(|text| u32::from_str_radix(text, 16).ok()).map_or(0, |word| word as i32);
    json!([word(0), word(1), word(2), word(3)])
}

fn uuid_text(value: &Value) -> Option<String> {
    let list = value.as_array().filter(|list| list.len() == 4)?;
    let text: String = list.iter().map(|word| format!("{:08x}", word.as_i64().unwrap_or(0) as u32)).collect();
    Some(crate::dashed(&text))
}

/// The player with this UUID, if they're on.
fn online(uuid: &str) -> Option<Player> {
    let plain = uuid.replace('-', "");
    crate::get_online_players().into_iter().find(|player| player.uuid().replace('-', "") == plain)
}

/// An entity by its id: a player's is the `Player`.
fn actor_id(near: &Entity, id: u64) -> Option<Actor> {
    actor_in(&near.get_world(), id)
}

fn actor_in(world: &World, id: u64) -> Option<Actor> {
    if id == 0 {
        return None;
    }
    Some(match crate::get_online_players().into_iter().find(|player| player.get_entity_id() as u64 == id) {
        Some(player) => Actor::Player(player),
        None => Actor::Entity(Entity::of(id, world)),
    })
}

/// What the game's answer names: `{ "entity": id }`, `{ "player": true, "uuid": ... }`, `{ "uuid": ... }` or a bare id.
fn actor(near: &Entity, value: &Value) -> Option<Actor> {
    let uuid = value["uuid"].as_str().map(str::to_owned).or_else(|| uuid_text(&value["uuid"]));
    if let Some(uuid) = uuid {
        return online(&uuid).map(Actor::Player).or_else(|| Some(Actor::Entity(near.get_world().get_entity(&crate::dashed(&uuid.replace('-', "")))?)));
    }
    actor_id(near, value["entity"].as_u64().or_else(|| value.as_u64())?)
}

/// `actor` the other way round, as the game's ops take it.
fn who(actor: &Actor) -> Value {
    match actor {
        Actor::Player(player) => {
            let mut fields = player_target(player);
            fields["entity"] = json!(player.get_entity_id());
            fields
        }
        Actor::Entity(entity) => target(entity),
    }
}

/// Java's `instanceof LivingEntity`.
fn living(entity: &Entity) -> bool {
    !get(entity, "is_mob").is_null() || is_type(entity, &["minecraft:armor_stand"])
}

/// `org.bukkit.entity.AnimalTamer`: what a player and an offline one share.
pub trait AnimalTamer {
    /// bukkit: org/bukkit/entity/AnimalTamer.getUniqueId
    fn tamer_unique_id(&self) -> Option<String>;
    /// bukkit: org/bukkit/entity/AnimalTamer.getName
    fn tamer_name(&self) -> Option<String>;
}

impl AnimalTamer for Player {
    fn tamer_unique_id(&self) -> Option<String> {
        Some(self.get_unique_id())
    }
    fn tamer_name(&self) -> Option<String> {
        Some(self.get_name().to_owned())
    }
}

impl AnimalTamer for OfflinePlayer {
    fn tamer_unique_id(&self) -> Option<String> {
        self.get_unique_id().map(str::to_owned)
    }
    fn tamer_name(&self) -> Option<String> {
        Some(self.get_name().to_owned()).filter(|name| !name.is_empty())
    }
}

/// Someone by UUID, on or not: a name from `usercache.json`, none when they were never seen.
fn offline_by_uuid(uuid: &str) -> OfflinePlayer {
    if let Some(player) = online(uuid) {
        return crate::get_offline_player(player.get_name());
    }
    let wanted = uuid.replace('-', "");
    let cache: Value = std::fs::read("usercache.json").ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default();
    let name = cache.as_array().into_iter().flatten().find(|row| row["uuid"].as_str().is_some_and(|known| known.replace('-', "") == wanted)).and_then(|row| row["name"].as_str());
    OfflinePlayer { name: name.unwrap_or_default().to_owned(), uuid: Some(crate::dashed(&wanted)), online: false }
}

/// `org.bukkit.material.MaterialData`: a block as Bukkit's old calls take it.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialData {
    material: crate::Material,
    data: u8,
}

impl MaterialData {
    pub fn new(material: crate::Material) -> Self {
        Self { material, data: 0 }
    }
    /// The material.
    pub fn get_item_type(&self) -> &crate::Material {
        &self.material
    }
    /// The old data value: nothing has one any more.
    pub fn get_data(&self) -> u8 {
        self.data
    }
}

/// `Player.canSee(Entity)`: nothing hid it from them.
fn sees(player: &Player, id: u64) -> bool {
    let hidden = crate::entities3::hidden_of(player).iter().any(|known| *known as u64 == id);
    !hidden && UNSEEN.lock().unwrap_or_else(|e| e.into_inner()).get(&id).is_none_or(|shown| shown.iter().any(|uuid| uuid == player.uuid()))
}

/// What a player's client shows: the game's board or one a plugin made. Rust has a type for each.
#[derive(Clone)]
pub enum ShownScoreboard {
    Game(crate::scoreboard::Scoreboard),
    Plugin(crate::lead::PlayerScoreboard),
}

/// `net.kyori.adventure.identity.Identity`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity(String);

impl Identity {
    /// Whose it is.
    pub fn uuid(&self) -> &str {
        &self.0
    }
}

/// `LivingEntity.getCollidableExemptions`: a live set of UUIDs, for this entity or player.
#[derive(Clone, Debug, PartialEq)]
pub struct CollidableExemptions(Entity);

impl CollidableExemptions {
    fn list(&self) -> Vec<String> {
        hold(&self.0, "exempt_get", Value::Null).as_array().into_iter().flatten().filter_map(|uuid| Some(crate::dashed(uuid.as_str()?))).collect()
    }
    fn write(&self, list: &[String]) -> bool {
        hold(&self.0, "exempt", json!(list.iter().map(|uuid| uuid.replace('-', "")).collect::<Vec<_>>())).as_bool().unwrap_or(false)
    }
    /// How many.
    pub fn len(&self) -> usize {
        self.list().len()
    }
    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.list().is_empty()
    }
    /// Whether the set holds this UUID.
    pub fn contains(&self, uuid: &str) -> bool {
        let wanted = uuid.replace('-', "");
        self.list().iter().any(|known| known.replace('-', "") == wanted)
    }
    /// Adds a UUID: false when it was there.
    pub fn add(&self, uuid: &str) -> bool {
        let mut list = self.list();
        if self.contains(uuid) {
            return false;
        }
        list.push(uuid.to_owned());
        self.write(&list)
    }
    /// Takes a UUID out: false when it wasn't there.
    pub fn remove(&self, uuid: &str) -> bool {
        let wanted = uuid.replace('-', "");
        let list = self.list();
        let kept: Vec<String> = list.iter().filter(|known| known.replace('-', "") != wanted).cloned().collect();
        kept.len() != list.len() && self.write(&kept)
    }
    /// Empties it.
    pub fn clear(&self) {
        self.write(&[]);
    }
    /// What's in it now.
    pub fn iter(&self) -> std::vec::IntoIter<String> {
        self.list().into_iter()
    }
}

/// `Player.getFireImmuneTicks`.
const PLAYER_FIRE_TICKS: i32 = 20;
/// What isn't a mob and doesn't burn: `EntityTypes`' `fireImmune`.
const FIRE_IMMUNE: &[&str] = &["minecraft:area_effect_cloud", "minecraft:end_crystal", "minecraft:tnt"];
/// The `fall_damage_immune` tag.
const FALL_IMMUNE: &[&str] = &[
    "minecraft:copper_golem", "minecraft:iron_golem", "minecraft:snow_golem", "minecraft:shulker", "minecraft:allay", "minecraft:bat", "minecraft:bee", "minecraft:blaze", "minecraft:cat",
    "minecraft:chicken", "minecraft:ghast", "minecraft:happy_ghast", "minecraft:phantom", "minecraft:magma_cube", "minecraft:ocelot", "minecraft:parrot", "minecraft:wither", "minecraft:breeze",
];

/// Whether the entity's type is one of `ids`.
fn is_type(entity: &Entity, ids: &[&str]) -> bool {
    let kind = entity.get_type();
    ids.iter().any(|id| kind == *id)
}

macro_rules! classes {
    ($($name:ident),* $(,)?) => {$(
        /// One of Bukkit's entity classes: `entity.cast::<...>()`.
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name(pub Entity);

        impl std::ops::Deref for $name {
            type Target = Entity;
            fn deref(&self) -> &Entity {
                &self.0
            }
        }

        impl EntityClass for $name {
            fn of(entity: Entity) -> Self {
                Self(entity)
            }
        }
    )*};
}

classes!(
    AbstractSkeleton, AbstractVillager, AbstractWindCharge, Arrow, Boat, Camel, Evoker, Fireball, Guardian, Hanging, Leashable, PigZombie, PolarBear, RangedEntity, SchoolableFish, Shearable, Skeleton, Sniffer, Warden,
    Witch, Bucketable, ShulkerBullet, SulfurCube,
);

impl Panda {
    /// bukkit: org/bukkit/entity/Panda.isSneezing
    pub fn is_sneezing(&self) -> bool {
        flag(&self.0, "panda_sneezing")
    }
    /// bukkit: org/bukkit/entity/Panda.setSneezing
    pub fn set_sneezing(&self, value: bool) -> bool {
        put(&self.0, "panda_sneezing", json!(value))
    }
    /// bukkit: org/bukkit/entity/Panda.isRolling
    pub fn is_rolling(&self) -> bool {
        flag(&self.0, "panda_rolling")
    }
    /// bukkit: org/bukkit/entity/Panda.setRolling
    pub fn set_rolling(&self, value: bool) -> bool {
        put(&self.0, "panda_rolling", json!(value))
    }
    /// bukkit: org/bukkit/entity/Panda.isOnBack
    pub fn is_on_back(&self) -> bool {
        flag(&self.0, "panda_on_back")
    }
    /// bukkit: org/bukkit/entity/Panda.setOnBack, org/bukkit/entity/Panda.setIsOnBack
    pub fn set_on_back(&self, value: bool) -> bool {
        put(&self.0, "panda_on_back", json!(value))
    }
    /// bukkit: org/bukkit/entity/Panda.isEating
    pub fn is_eating(&self) -> bool {
        flag(&self.0, "panda_eating")
    }
    /// bukkit: org/bukkit/entity/Panda.setEating
    pub fn set_eating(&self, value: bool) -> bool {
        put(&self.0, "panda_eating", json!(value))
    }
    /// bukkit: org/bukkit/entity/Panda.isScared
    pub fn is_scared(&self) -> bool {
        flag(&self.0, "panda_scared")
    }
    /// bukkit: org/bukkit/entity/Panda.getUnhappyTicks
    pub fn get_unhappy_ticks(&self) -> i32 {
        number(&self.0, "panda_unhappy")
    }
    /// bukkit: org/bukkit/entity/Panda.setUnhappyTicks
    pub fn set_unhappy_ticks(&self, value: i32) -> bool {
        put(&self.0, "panda_unhappy", json!(value))
    }
    /// bukkit: org/bukkit/entity/Panda.getSneezeTicks
    pub fn get_sneeze_ticks(&self) -> i32 {
        number(&self.0, "panda_sneeze_ticks")
    }
    /// bukkit: org/bukkit/entity/Panda.setSneezeTicks
    pub fn set_sneeze_ticks(&self, value: i32) -> bool {
        put(&self.0, "panda_sneeze_ticks", json!(value))
    }
    /// bukkit: org/bukkit/entity/Panda.getEatingTicks
    pub fn get_eating_ticks(&self) -> i32 {
        number(&self.0, "panda_eat_ticks")
    }
    /// bukkit: org/bukkit/entity/Panda.setEatingTicks
    pub fn set_eating_ticks(&self, value: i32) -> bool {
        put(&self.0, "panda_eat_ticks", json!(value))
    }
}

impl AbstractHorse {
    /// bukkit: org/bukkit/entity/AbstractHorse.isEatingGrass
    pub fn is_eating_grass(&self) -> bool {
        flag(&self.0, "horse_grazing")
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.setEatingGrass, org/bukkit/entity/AbstractHorse.setEatingHaystack
    pub fn set_eating_grass(&self, value: bool) -> bool {
        put(&self.0, "horse_grazing", json!(value))
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.isRearing
    pub fn is_rearing(&self) -> bool {
        flag(&self.0, "horse_rearing")
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.setRearing
    pub fn set_rearing(&self, value: bool) -> bool {
        put(&self.0, "horse_rearing", json!(value))
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.getMaxDomestication
    pub fn get_max_domestication(&self) -> i32 {
        plugin_get(&self.0, "max_domestication").as_f64().map_or_else(|| number(&self.0, "horse_max_temper"), |max| max as i32)
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.setMaxDomestication
    pub fn set_max_domestication(&self, level: i32) -> bool {
        level > 0 && plugin_set(&self.0, "max_domestication", json!(level))
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.isEating
    pub fn is_eating(&self) -> bool {
        plugin_get(&self.0, "mouth_open").as_f64() == Some(1.0)
    }
    /// bukkit: org/bukkit/entity/AbstractHorse.setEating
    pub fn set_eating(&self, eating: bool) -> bool {
        plugin_set(&self.0, "mouth_open", json!(eating))
    }
}

impl Bee {
    /// bukkit: org/bukkit/entity/Bee.getAnger
    pub fn get_anger(&self) -> i32 {
        number(&self.0, "bee_anger")
    }
    /// bukkit: org/bukkit/entity/Bee.setAnger
    pub fn set_anger(&self, value: i32) -> bool {
        put(&self.0, "bee_anger", json!(value))
    }
    /// bukkit: org/bukkit/entity/Bee.isRolling
    pub fn is_rolling(&self) -> bool {
        flag(&self.0, "bee_rolling")
    }
    /// bukkit: org/bukkit/entity/Bee.getTimeSinceSting
    pub fn get_time_since_sting(&self) -> i32 {
        number(&self.0, "bee_time_since_sting")
    }
    /// bukkit: org/bukkit/entity/Bee.setTimeSinceSting
    pub fn set_time_since_sting(&self, value: i32) -> bool {
        put(&self.0, "bee_time_since_sting", json!(value))
    }
}

impl Enderman {
    /// bukkit: org/bukkit/entity/Enderman.isScreaming
    pub fn is_screaming(&self) -> bool {
        flag(&self.0, "screaming")
    }
    /// bukkit: org/bukkit/entity/Enderman.setScreaming
    pub fn set_screaming(&self, value: bool) -> bool {
        put(&self.0, "screaming", json!(value))
    }
}

impl Ghast {
    /// bukkit: org/bukkit/entity/Ghast.isCharging
    pub fn is_charging(&self) -> bool {
        flag(&self.0, "charging")
    }
    /// bukkit: org/bukkit/entity/Ghast.setCharging
    pub fn set_charging(&self, value: bool) -> bool {
        put(&self.0, "charging", json!(value))
    }
}

impl Vex {
    /// bukkit: org/bukkit/entity/Vex.isCharging
    pub fn is_charging(&self) -> bool {
        flag(&self.0, "charging")
    }
    /// bukkit: org/bukkit/entity/Vex.setCharging
    pub fn set_charging(&self, value: bool) -> bool {
        put(&self.0, "charging", json!(value))
    }
}

impl Piglin {
    /// bukkit: org/bukkit/entity/Piglin.isDancing
    pub fn is_dancing(&self) -> bool {
        flag(&self.0, "dancing")
    }
}

impl Parrot {
    /// bukkit: org/bukkit/entity/Parrot.isDancing
    pub fn is_dancing(&self) -> bool {
        flag(&self.0, "dancing")
    }
}

impl Allay {
    /// bukkit: org/bukkit/entity/Allay.isDancing
    pub fn is_dancing(&self) -> bool {
        flag(&self.0, "dancing")
    }
}

impl Strider {
    /// bukkit: org/bukkit/entity/Strider.isShivering
    pub fn is_shivering(&self) -> bool {
        flag(&self.0, "shivering")
    }
    /// bukkit: org/bukkit/entity/Strider.setShivering
    pub fn set_shivering(&self, value: bool) -> bool {
        put(&self.0, "shivering", json!(value))
    }
}

impl Camel {
    /// bukkit: org/bukkit/entity/Camel.isDashing
    pub fn is_dashing(&self) -> bool {
        flag(&self.0, "dashing")
    }
    /// bukkit: org/bukkit/entity/Camel.setDashing
    pub fn set_dashing(&self, value: bool) -> bool {
        put(&self.0, "dashing", json!(value))
    }
}

impl Axolotl {
    /// bukkit: org/bukkit/entity/Axolotl.isPlayingDead
    pub fn is_playing_dead(&self) -> bool {
        flag(&self.0, "playing_dead")
    }
    /// bukkit: org/bukkit/entity/Axolotl.setPlayingDead
    pub fn set_playing_dead(&self, value: bool) -> bool {
        put(&self.0, "playing_dead", json!(value))
    }
}

impl PolarBear {
    /// bukkit: org/bukkit/entity/PolarBear.isStanding
    pub fn is_standing(&self) -> bool {
        flag(&self.0, "standing")
    }
    /// bukkit: org/bukkit/entity/PolarBear.setStanding
    pub fn set_standing(&self, value: bool) -> bool {
        put(&self.0, "standing", json!(value))
    }
}

impl Fox {
    /// bukkit: org/bukkit/entity/Fox.isInterested
    pub fn is_interested(&self) -> bool {
        flag(&self.0, "fox_interested")
    }
    /// bukkit: org/bukkit/entity/Fox.setInterested
    pub fn set_interested(&self, value: bool) -> bool {
        put(&self.0, "fox_interested", json!(value))
    }
    /// bukkit: org/bukkit/entity/Fox.isLeaping
    pub fn is_leaping(&self) -> bool {
        flag(&self.0, "fox_leaping")
    }
    /// bukkit: org/bukkit/entity/Fox.setLeaping
    pub fn set_leaping(&self, value: bool) -> bool {
        put(&self.0, "fox_leaping", json!(value))
    }
    /// bukkit: org/bukkit/entity/Fox.isFaceplanted
    pub fn is_faceplanted(&self) -> bool {
        flag(&self.0, "fox_faceplanted")
    }
    /// bukkit: org/bukkit/entity/Fox.setFaceplanted
    pub fn set_faceplanted(&self, value: bool) -> bool {
        put(&self.0, "fox_faceplanted", json!(value))
    }
    /// bukkit: org/bukkit/entity/Fox.isDefending
    pub fn is_defending(&self) -> bool {
        flag(&self.0, "fox_defending")
    }
    /// bukkit: org/bukkit/entity/Fox.setDefending
    pub fn set_defending(&self, value: bool) -> bool {
        put(&self.0, "fox_defending", json!(value))
    }
}

impl Cat {
    /// bukkit: org/bukkit/entity/Cat.isLyingDown
    pub fn is_lying_down(&self) -> bool {
        flag(&self.0, "cat_lying")
    }
    /// bukkit: org/bukkit/entity/Cat.setLyingDown
    pub fn set_lying_down(&self, value: bool) -> bool {
        put(&self.0, "cat_lying", json!(value))
    }
    /// bukkit: org/bukkit/entity/Cat.isHeadUp
    pub fn is_head_up(&self) -> bool {
        flag(&self.0, "cat_head_up")
    }
    /// bukkit: org/bukkit/entity/Cat.setHeadUp
    pub fn set_head_up(&self, value: bool) -> bool {
        put(&self.0, "cat_head_up", json!(value))
    }
}

impl Wolf {
    /// bukkit: org/bukkit/entity/Wolf.isInterested
    pub fn is_interested(&self) -> bool {
        flag(&self.0, "wolf_interested")
    }
    /// bukkit: org/bukkit/entity/Wolf.setInterested
    pub fn set_interested(&self, value: bool) -> bool {
        put(&self.0, "wolf_interested", json!(value))
    }
    /// bukkit: org/bukkit/entity/Wolf.isWet
    pub fn is_wet(&self) -> bool {
        flag(&self.0, "wolf_wet")
    }
}

impl Guardian {
    /// bukkit: org/bukkit/entity/Guardian.isMoving
    pub fn is_moving(&self) -> bool {
        flag(&self.0, "guardian_moving")
    }
    /// bukkit: org/bukkit/entity/Guardian.hasLaser
    pub fn has_laser(&self) -> bool {
        flag(&self.0, "guardian_laser")
    }
    /// bukkit: org/bukkit/entity/Guardian.getLaserTicks
    pub fn get_laser_ticks(&self) -> i32 {
        number(&self.0, "guardian_laser_ticks")
    }
    /// bukkit: org/bukkit/entity/Guardian.setLaserTicks
    pub fn set_laser_ticks(&self, value: i32) -> bool {
        put(&self.0, "guardian_laser_ticks", json!(value))
    }
}

impl Turtle {
    /// bukkit: org/bukkit/entity/Turtle.isLayingEgg
    pub fn is_laying_egg(&self) -> bool {
        flag(&self.0, "turtle_laying")
    }
    /// bukkit: org/bukkit/entity/Turtle.isDigging
    pub fn is_digging(&self) -> bool {
        flag(&self.0, "turtle_digging")
    }
    /// bukkit: org/bukkit/entity/Turtle.isGoingHome
    pub fn is_going_home(&self) -> bool {
        flag(&self.0, "turtle_home")
    }
}

impl Skeleton {
    /// bukkit: org/bukkit/entity/Skeleton.getConversionTime
    pub fn get_conversion_time(&self) -> i32 {
        number(&self.0, "skeleton_conversion")
    }
    /// bukkit: org/bukkit/entity/Skeleton.setConversionTime
    pub fn set_conversion_time(&self, value: i32) -> bool {
        put(&self.0, "skeleton_conversion", json!(value))
    }
    /// bukkit: org/bukkit/entity/Skeleton.inPowderedSnowTime
    pub fn in_powdered_snow_time(&self) -> i32 {
        number(&self.0, "skeleton_snow_time")
    }
}

impl Zombie {
    /// bukkit: org/bukkit/entity/Zombie.isDrowning
    pub fn is_drowning(&self) -> bool {
        flag(&self.0, "zombie_drowning")
    }
    /// bukkit: org/bukkit/entity/Zombie.isArmsRaised
    pub fn is_arms_raised(&self) -> bool {
        flag(&self.0, "arms_raised")
    }
    /// bukkit: org/bukkit/entity/Zombie.setArmsRaised
    pub fn set_arms_raised(&self, value: bool) -> bool {
        put(&self.0, "arms_raised", json!(value))
    }
}

impl Creeper {
    /// bukkit: org/bukkit/entity/Creeper.getFuseTicks
    pub fn get_fuse_ticks(&self) -> i32 {
        number(&self.0, "creeper_fuse")
    }
    /// bukkit: org/bukkit/entity/Creeper.setFuseTicks
    pub fn set_fuse_ticks(&self, value: i32) -> bool {
        put(&self.0, "creeper_fuse", json!(value))
    }
}

impl Witch {
    /// bukkit: org/bukkit/entity/Witch.isDrinkingPotion
    pub fn is_drinking_potion(&self) -> bool {
        flag(&self.0, "witch_drinking")
    }
    /// bukkit: org/bukkit/entity/Witch.getPotionUseTimeLeft
    pub fn get_potion_use_time_left(&self) -> i32 {
        number(&self.0, "witch_drink_ticks")
    }
    /// bukkit: org/bukkit/entity/Witch.setPotionUseTimeLeft
    pub fn set_potion_use_time_left(&self, value: i32) -> bool {
        put(&self.0, "witch_drink_ticks", json!(value))
    }
}

impl Raider {
    /// bukkit: org/bukkit/entity/Raider.isCelebrating
    pub fn is_celebrating(&self) -> bool {
        flag(&self.0, "raider_celebrating")
    }
    /// bukkit: org/bukkit/entity/Raider.setCelebrating
    pub fn set_celebrating(&self, value: bool) -> bool {
        put(&self.0, "raider_celebrating", json!(value))
    }
}

impl Animals {
    /// bukkit: org/bukkit/entity/Animals.isLoveMode
    pub fn is_love_mode(&self) -> bool {
        flag(&self.0, "love")
    }
}

impl Steerable {
    /// bukkit: org/bukkit/entity/Steerable.getBoostTicks
    pub fn get_boost_ticks(&self) -> i32 {
        number(&self.0, "boost_ticks")
    }
    /// bukkit: org/bukkit/entity/Steerable.setBoostTicks
    pub fn set_boost_ticks(&self, value: i32) -> bool {
        put(&self.0, "boost_ticks", json!(value))
    }
    /// bukkit: org/bukkit/entity/Steerable.getCurrentBoostTicks
    pub fn get_current_boost_ticks(&self) -> i32 {
        number(&self.0, "boost_current")
    }
    /// bukkit: org/bukkit/entity/Steerable.setCurrentBoostTicks
    pub fn set_current_boost_ticks(&self, value: i32) -> bool {
        put(&self.0, "boost_current", json!(value))
    }
}

impl Wither {
    /// bukkit: org/bukkit/entity/Wither.isCharged
    pub fn is_charged(&self) -> bool {
        flag(&self.0, "wither_powered")
    }
}

impl Creaking {
    /// bukkit: org/bukkit/entity/Creaking.isActive
    pub fn is_active(&self) -> bool {
        flag(&self.0, "creaking_active")
    }
}

impl PigZombie {
    /// bukkit: org/bukkit/entity/PigZombie.getAnger
    pub fn get_anger(&self) -> i32 {
        number(&self.0, "anger")
    }
    /// bukkit: org/bukkit/entity/PigZombie.setAnger
    pub fn set_anger(&self, value: i32) -> bool {
        put(&self.0, "anger", json!(value))
    }
}

// ---- Mobs: more than a field ----

impl Panda {
    /// bukkit: org/bukkit/entity/Panda.getCombinedGene
    pub fn get_combined_gene(&self) -> &'static str {
        crate::enums::panda_gene::values().get(number(&self.0, "panda_gene") as usize).copied().unwrap_or("NORMAL")
    }
}

/// bukkit: org/bukkit/entity/Panda$Gene.isRecessive
pub fn panda_gene_is_recessive(gene: &str) -> bool {
    matches!(gene, "BROWN" | "WEAK")
}

impl AbstractHorse {
    /// bukkit: org/bukkit/entity/AbstractHorse.getVariant
    pub fn get_variant(&self) -> &'static str {
        match self.get_type().trim_start_matches("minecraft:") {
            "donkey" => "DONKEY",
            "mule" => "MULE",
            "skeleton_horse" => "SKELETON_HORSE",
            "zombie_horse" => "UNDEAD_HORSE",
            "llama" => "LLAMA",
            "trader_llama" => "LLAMA",
            "camel" | "camel_husk" => "CAMEL",
            _ => "HORSE",
        }
    }
}

impl Allay {
    /// bukkit: org/bukkit/entity/Allay.resetDuplicationCooldown
    pub fn reset_duplication_cooldown(&self) -> bool {
        self.set_duplication_cooldown(6000)
    }
    /// bukkit: org/bukkit/entity/Allay.getJukebox
    pub fn get_jukebox(&self) -> Option<Location> {
        place(&self.0, &act(&self.0, "allay_jukebox", json!({})))
    }
    /// bukkit: org/bukkit/entity/Allay.duplicateAllay
    pub fn duplicate_allay(&self) -> Option<Allay> {
        let id = act(&self.0, "allay_duplicate", json!({})).as_u64()?;
        Some(Allay(Entity::of(id, &self.get_world())))
    }
}

impl Enderman {
    /// bukkit: org/bukkit/entity/Enderman.teleportRandomly, org/bukkit/entity/Enderman.teleport
    pub fn teleport_randomly(&self) -> bool {
        done(&self.0, "enderman_teleport", json!({}))
    }
    /// bukkit: org/bukkit/entity/Enderman.teleportTowards
    pub fn teleport_towards(&self, entity: &Entity) -> bool {
        entity.get_location().is_some_and(|at| done(&self.0, "enderman_teleport_to", json!({ "at": [at.x, at.y, at.z] })))
    }
}

impl TropicalFish {
    /// bukkit: org/bukkit/entity/TropicalFish.getPattern
    pub fn get_pattern(&self) -> &'static str {
        crate::enums::tropical_fish_pattern::values().get(number(&self.0, "fish_pattern") as usize).copied().unwrap_or("KOB")
    }
    /// bukkit: org/bukkit/entity/TropicalFish.setPattern
    pub fn set_pattern(&self, pattern: &str) -> bool {
        crate::enums::tropical_fish_pattern::values().iter().position(|name| *name == pattern).is_some_and(|index| put(&self.0, "fish_pattern", json!(index)))
    }
}

impl Spellcaster {
    /// bukkit: org/bukkit/entity/Spellcaster.getSpell
    pub fn get_spell(&self) -> &'static str {
        crate::enums::spellcaster_spell::values().get(number(&self.0, "spell") as usize).copied().unwrap_or("NONE")
    }
    /// bukkit: org/bukkit/entity/Spellcaster.setSpell
    pub fn set_spell(&self, spell: &str) -> bool {
        crate::enums::spellcaster_spell::values().iter().position(|name| *name == spell).is_some_and(|index| put(&self.0, "spell", json!(index)))
    }
}

impl Evoker {
    /// bukkit: org/bukkit/entity/Evoker.getCurrentSpell
    pub fn get_current_spell(&self) -> &'static str {
        crate::enums::evoker_spell::values().get(number(&self.0, "spell") as usize).copied().unwrap_or("NONE")
    }
    /// bukkit: org/bukkit/entity/Evoker.setCurrentSpell
    pub fn set_current_spell(&self, spell: Option<&str>) -> bool {
        let index = spell.and_then(|spell| crate::enums::evoker_spell::values().iter().position(|name| *name == spell)).unwrap_or(0);
        put(&self.0, "spell", json!(index))
    }
}

impl Warden {
    /// bukkit: org/bukkit/entity/Warden.getAnger
    pub fn get_anger(&self) -> i32 {
        act(&self.0, "warden_get", json!({})).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Warden.getAnger
    pub fn get_anger_at(&self, entity: &Entity) -> i32 {
        act(&self.0, "warden_get", target(entity)).as_i64().unwrap_or(0) as i32
    }
    /// bukkit~: org/bukkit/entity/Warden.getAnger | at a player: the one player the mob's `Who::Player` stands for
    pub fn get_anger_at_player(&self, player: &Player) -> i32 {
        act(&self.0, "warden_get", player_target(player)).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Warden.getHighestAnger
    pub fn get_highest_anger(&self) -> i32 {
        act(&self.0, "warden_highest", json!({})).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Warden.getAngerLevel
    pub fn get_anger_level(&self) -> &'static str {
        match self.get_anger() {
            0..=39 => "CALM",
            40..=79 => "AGITATED",
            _ => "ANGRY",
        }
    }
    /// bukkit: org/bukkit/entity/Warden.getEntityAngryAt
    pub fn get_entity_angry_at(&self) -> Option<Actor> {
        actor(&self.0, &act(&self.0, "warden_target", json!({})))
    }
    /// bukkit: org/bukkit/entity/Warden.increaseAnger
    pub fn increase_anger(&self, entity: &Entity, increase: i32) -> bool {
        let mut fields = target(entity);
        fields["value"] = json!(increase);
        act(&self.0, "warden_increase", fields).is_number()
    }
    /// bukkit: org/bukkit/entity/Warden.setAnger
    pub fn set_anger(&self, entity: &Entity, anger: i32) -> bool {
        let mut fields = target(entity);
        fields["value"] = json!(anger);
        act(&self.0, "warden_set", fields).is_number()
    }
    /// bukkit: org/bukkit/entity/Warden.clearAnger
    pub fn clear_anger(&self, entity: &Entity) -> bool {
        done(&self.0, "warden_clear", target(entity))
    }
}

/// `org.bukkit.entity.memory.MemoryKey`: `memory_key` holds the constants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryKey(pub String);

impl MemoryKey {
    /// bukkit: org/bukkit/entity/memory/MemoryKey.getKey
    pub fn get_key(&self) -> String {
        if self.0 == "LIKED_NOTEBLOCK_POSITION" {
            return "minecraft:liked_noteblock".to_owned();
        }
        format!("minecraft:{}", self.0.to_ascii_lowercase())
    }
    /// bukkit: org/bukkit/entity/memory/MemoryKey.getByKey
    pub fn get_by_key(key: &str) -> Option<Self> {
        let name = key.trim_start_matches("minecraft:").to_ascii_uppercase();
        let name = if name == "LIKED_NOTEBLOCK" { "LIKED_NOTEBLOCK_POSITION".to_owned() } else { name };
        crate::enums::memory_key::value_of(&name).map(|name| Self(name.to_owned()))
    }
}

impl Skeleton {
    /// bukkit: org/bukkit/entity/Skeleton.isConverting
    pub fn is_converting(&self) -> bool {
        self.get_conversion_time() >= 0
    }
}

impl Zombie {
    /// bukkit: org/bukkit/entity/Zombie.startDrowning
    pub fn start_drowning(&self, ticks: i32) -> bool {
        put(&self.0, "zombie_conversion", json!(ticks.max(1)))
    }
    /// bukkit: org/bukkit/entity/Zombie.stopDrowning
    pub fn stop_drowning(&self) -> bool {
        put(&self.0, "zombie_conversion", json!(-1))
    }
    /// bukkit: org/bukkit/entity/Zombie.shouldBurnInDay
    pub fn should_burn_in_day(&self) -> bool {
        flag(&self.0, "burns_in_day")
    }
    /// bukkit: org/bukkit/entity/Zombie.setShouldBurnInDay
    pub fn set_should_burn_in_day(&self, burn: bool) -> bool {
        plugin_set(&self.0, "burns_in_day", json!(burn)) && put(&self.0, "burns_in_day", json!(burn))
    }
    /// bukkit: org/bukkit/entity/Zombie.isVillager
    pub fn is_villager(&self) -> bool {
        is_type(&self.0, &["minecraft:zombie_villager"])
    }
    /// bukkit: org/bukkit/entity/Zombie.supportsBreakingDoors
    pub fn supports_breaking_doors(&self) -> bool {
        is_type(&self.0, &["minecraft:zombie", "minecraft:husk", "minecraft:drowned", "minecraft:zombie_villager"])
    }
    /// bukkit: org/bukkit/entity/Zombie.getVillagerProfession
    pub fn get_villager_profession(&self) -> Option<String> {
        self.data_at("VillagerData.profession").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Zombie.setVillagerProfession
    pub fn set_villager_profession(&self, profession: &str) -> bool {
        self.is_villager() && self.merge(&format!("{{VillagerData:{{profession:{}}}}}", crate::data::quoted(profession)))
    }
}

impl ZombieVillager {
    /// bukkit: org/bukkit/entity/ZombieVillager.isConverting
    pub fn is_converting(&self) -> bool {
        number(&self.0, "cure_time") >= 0
    }
}

impl AbstractSkeleton {
    /// bukkit: org/bukkit/entity/AbstractSkeleton.shouldBurnInDay
    pub fn should_burn_in_day(&self) -> bool {
        flag(&self.0, "burns_in_day")
    }
    /// bukkit: org/bukkit/entity/AbstractSkeleton.setShouldBurnInDay
    pub fn set_should_burn_in_day(&self, burn: bool) -> bool {
        plugin_set(&self.0, "burns_in_day", json!(burn)) && put(&self.0, "burns_in_day", json!(burn))
    }
    /// bukkit: org/bukkit/entity/AbstractSkeleton.getSkeletonType
    pub fn get_skeleton_type(&self) -> &'static str {
        match self.get_type().trim_start_matches("minecraft:") {
            "wither_skeleton" => "WITHER",
            "stray" => "STRAY",
            "bogged" => "BOGGED",
            "parched" => "PARCHED",
            _ => "NORMAL",
        }
    }
}

impl Phantom {
    /// bukkit: org/bukkit/entity/Phantom.shouldBurnInDay
    pub fn should_burn_in_day(&self) -> bool {
        flag(&self.0, "burns_in_day")
    }
    /// bukkit: org/bukkit/entity/Phantom.setShouldBurnInDay
    pub fn set_should_burn_in_day(&self, burn: bool) -> bool {
        plugin_set(&self.0, "burns_in_day", json!(burn)) && put(&self.0, "burns_in_day", json!(burn))
    }
}

/// `TimeInOverworld` as Bukkit's conversion counter, shared by hoglins and piglins.
fn is_nether(entity: &Entity) -> bool {
    entity.get_world().dimension().ends_with("the_nether")
}

impl Hoglin {
    /// bukkit: org/bukkit/entity/Hoglin.isConverting
    pub fn is_converting(&self) -> bool {
        !is_nether(&self.0) && !self.is_immune_to_zombification() && self.has_ai()
    }
    /// bukkit: org/bukkit/entity/Hoglin.getConversionTime
    pub fn get_conversion_time(&self) -> i32 {
        300 - number(&self.0, "piglin_conversion")
    }
    /// bukkit: org/bukkit/entity/Hoglin.setConversionTime
    pub fn set_conversion_time(&self, time: i32) -> bool {
        put(&self.0, "piglin_conversion", json!(300 - time))
    }
}

impl PiglinAbstract {
    /// bukkit: org/bukkit/entity/PiglinAbstract.isConverting
    pub fn is_converting(&self) -> bool {
        !is_nether(&self.0) && !self.is_immune_to_zombification() && self.has_ai()
    }
    /// bukkit: org/bukkit/entity/PiglinAbstract.getConversionTime
    pub fn get_conversion_time(&self) -> i32 {
        300 - number(&self.0, "piglin_conversion")
    }
    /// bukkit: org/bukkit/entity/PiglinAbstract.setConversionTime
    pub fn set_conversion_time(&self, time: i32) -> bool {
        put(&self.0, "piglin_conversion", json!(300 - time))
    }
}

impl Piglin {
    /// bukkit: org/bukkit/entity/Piglin.setDancing
    pub fn set_dancing(&self, dancing: bool) -> bool {
        put(&self.0, "dancing", json!(dancing))
    }
}

impl PigZombie {
    /// bukkit: org/bukkit/entity/PigZombie.isAngry
    pub fn is_angry(&self) -> bool {
        self.get_anger() > 0
    }
    /// bukkit: org/bukkit/entity/PigZombie.setAngry
    pub fn set_angry(&self, angry: bool) -> bool {
        self.set_anger(if angry { 400 } else { 0 })
    }
    /// bukkit: org/bukkit/entity/PigZombie.isConverting
    pub fn is_converting(&self) -> bool {
        false
    }
}

impl Sniffer {
    /// bukkit: org/bukkit/entity/Sniffer.getState
    pub fn get_state(&self) -> &'static str {
        crate::enums::sniffer_state::values().get(number(&self.0, "sniffer_state") as usize).copied().unwrap_or("IDLING")
    }
    /// bukkit: org/bukkit/entity/Sniffer.setState
    pub fn set_state(&self, state: &str) -> bool {
        crate::enums::sniffer_state::values().iter().position(|name| *name == state).is_some_and(|index| put(&self.0, "sniffer_state", json!(index)))
    }
    /// bukkit: org/bukkit/entity/Sniffer.getExploredLocations
    pub fn get_explored_locations(&self) -> Vec<Location> {
        get(&self.0, "sniffer_explored").as_array().into_iter().flatten().filter_map(|at| place(&self.0, at)).collect()
    }
    /// bukkit: org/bukkit/entity/Sniffer.addExploredLocation
    pub fn add_explored_location(&self, location: &Location) -> bool {
        let mut list = get(&self.0, "sniffer_explored").as_array().cloned().unwrap_or_default();
        let at = block_of(location);
        if !list.contains(&at) {
            list.insert(0, at);
        }
        put(&self.0, "sniffer_explored", json!(list))
    }
    /// bukkit: org/bukkit/entity/Sniffer.removeExploredLocation
    pub fn remove_explored_location(&self, location: &Location) -> bool {
        let mut list = get(&self.0, "sniffer_explored").as_array().cloned().unwrap_or_default();
        let at = block_of(location);
        list.retain(|known| *known != at);
        put(&self.0, "sniffer_explored", json!(list))
    }
}

impl Armadillo {
    /// bukkit: org/bukkit/entity/Armadillo.rollUp
    pub fn roll_up(&self) -> bool {
        done(&self.0, "armadillo_roll", json!({ "value": true }))
    }
    /// bukkit: org/bukkit/entity/Armadillo.rollOut
    pub fn roll_out(&self) -> bool {
        done(&self.0, "armadillo_roll", json!({ "value": false }))
    }
}

impl Creaking {
    /// bukkit: org/bukkit/entity/Creaking.activate
    pub fn activate(&self, player: &Player) -> bool {
        done(&self.0, "creaking_activate", player_target(player))
    }
    /// bukkit: org/bukkit/entity/Creaking.deactivate
    pub fn deactivate(&self) -> bool {
        done(&self.0, "creaking_deactivate", json!({}))
    }
}

impl SulfurCube {
    /// bukkit: org/bukkit/entity/SulfurCube.canExplode
    pub fn can_explode(&self) -> bool {
        done(&self.0, "sulfur_can_explode", json!({}))
    }
    /// bukkit: org/bukkit/entity/SulfurCube.getFuseTicks
    pub fn get_fuse_ticks(&self) -> i32 {
        act(&self.0, "sulfur_fuse", json!({})).as_i64().unwrap_or(-1) as i32
    }
    /// bukkit: org/bukkit/entity/SulfurCube.ignite
    pub fn ignite(&self, imminent: bool) -> bool {
        done(&self.0, "sulfur_ignite", json!({ "value": imminent }))
    }
    /// bukkit: org/bukkit/entity/SulfurCube.getEquipped
    pub fn get_equipped(&self) -> Option<ItemStack> {
        self.get_equipment().get_item("BODY").or_else(|| ItemStack::read(&self.data_at("equipment.body")))
    }
    /// bukkit: org/bukkit/entity/SulfurCube.setEquipped
    pub fn set_equipped(&self, stack: &ItemStack) -> bool {
        self.merge(&format!("{{equipment:{{body:{}}}}}", stack.to_snbt()))
    }
}

impl Wither {
    /// bukkit: org/bukkit/entity/Wither.enterInvulnerabilityPhase
    pub fn enter_invulnerability_phase(&self) -> bool {
        done(&self.0, "wither_invulnerable", json!({}))
    }
}

impl Creeper {
    /// bukkit: org/bukkit/entity/Creeper.ignite
    pub fn ignite(&self) -> bool {
        done(&self.0, "ignite", json!({}))
    }
    /// bukkit: org/bukkit/entity/Creeper.explode
    pub fn explode(&self) -> bool {
        done(&self.0, "explode", json!({}))
    }
}

impl CopperGolem {
    /// bukkit: org/bukkit/entity/CopperGolem.getWeatheringState
    pub fn get_weathering_state(&self) -> &'static str {
        crate::enums::weathering_copper_state::values().get(number(&self.0, "golem_weather") as usize).copied().unwrap_or("UNAFFECTED")
    }
    /// bukkit: org/bukkit/entity/CopperGolem.setWeatheringState
    pub fn set_weathering_state(&self, state: &str) -> bool {
        crate::enums::weathering_copper_state::values().iter().position(|name| *name == state).is_some_and(|index| put(&self.0, "golem_weather", json!(index)))
    }
    /// bukkit: org/bukkit/entity/CopperGolem.getGolemState
    pub fn get_golem_state(&self) -> &'static str {
        crate::enums::copper_golem_state::values().get(number(&self.0, "golem_state") as usize).copied().unwrap_or("IDLE")
    }
    /// bukkit: org/bukkit/entity/CopperGolem.setGolemState
    pub fn set_golem_state(&self, state: &str) -> bool {
        crate::enums::copper_golem_state::values().iter().position(|name| *name == state).is_some_and(|index| put(&self.0, "golem_state", json!(index)))
    }
    /// bukkit: org/bukkit/entity/CopperGolem.getOxidizing
    pub fn get_oxidizing(&self) -> Oxidizing {
        match get(&self.0, "golem_oxidizing").as_i64().unwrap_or(-1) {
            -2 => Oxidizing::Waxed,
            -1 => Oxidizing::Unset,
            time => Oxidizing::AtTime(time),
        }
    }
    /// bukkit: org/bukkit/entity/CopperGolem.setOxidizing
    pub fn set_oxidizing(&self, oxidizing: Oxidizing) -> bool {
        put(&self.0, "golem_oxidizing", json!(match oxidizing {
            Oxidizing::Waxed => -2,
            Oxidizing::Unset => -1,
            Oxidizing::AtTime(time) => time,
        }))
    }
}

/// `org.bukkit.entity.CopperGolem.Oxidizing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oxidizing {
    /// It won't oxidize any further.
    Waxed,
    /// Not waxed, and its next weathering isn't drawn yet.
    Unset,
    /// The absolute game time of its next weathering.
    AtTime(i64),
}

impl Oxidizing {
    /// bukkit: org/bukkit/entity/CopperGolem$Oxidizing.waxed
    pub fn waxed() -> Self {
        Self::Waxed
    }
    /// bukkit: org/bukkit/entity/CopperGolem$Oxidizing.unset
    pub fn unset() -> Self {
        Self::Unset
    }
    /// bukkit: org/bukkit/entity/CopperGolem$Oxidizing.atTime
    pub fn at_time(time: i64) -> Self {
        Self::AtTime(time)
    }
    /// bukkit: org/bukkit/entity/CopperGolem$Oxidizing$AtTime.time
    pub fn time(&self) -> Option<i64> {
        match self {
            Self::AtTime(time) => Some(*time),
            _ => None,
        }
    }
}

impl Steerable {
    /// bukkit: org/bukkit/entity/Steerable.setSaddle
    pub fn set_saddle(&self, saddled: bool) -> bool {
        put(&self.0, "saddle", json!(saddled))
    }
    /// bukkit: org/bukkit/entity/Steerable.getSteerMaterial
    pub fn get_steer_material(&self) -> &'static str {
        if is_type(&self.0, &["minecraft:strider"]) { "WARPED_FUNGUS_ON_A_STICK" } else { "CARROT_ON_A_STICK" }
    }
}

impl Bat {
    /// bukkit: org/bukkit/entity/Bat.getTargetLocation
    pub fn get_target_location(&self) -> Option<Location> {
        place(&self.0, &get(&self.0, "bat_target"))
    }
    /// bukkit: org/bukkit/entity/Bat.setTargetLocation
    pub fn set_target_location(&self, location: Option<&Location>) -> bool {
        put(&self.0, "bat_target", location.map_or(Value::Null, block_of))
    }
}

impl Dolphin {
    /// bukkit: org/bukkit/entity/Dolphin.getTreasureLocation
    pub fn get_treasure_location(&self) -> Option<Location> {
        place(&self.0, &get(&self.0, "dolphin_treasure"))
    }
    /// bukkit: org/bukkit/entity/Dolphin.setTreasureLocation
    pub fn set_treasure_location(&self, location: Option<&Location>) -> bool {
        put(&self.0, "dolphin_treasure", location.map_or(Value::Null, block_of))
    }
}

impl SchoolableFish {
    /// bukkit: io/papermc/paper/entity/SchoolableFish.getSchoolSize
    pub fn get_school_size(&self) -> i32 {
        number(&self.0, "fish_followers") + 1
    }
    /// bukkit: io/papermc/paper/entity/SchoolableFish.getSchoolLeader
    pub fn get_school_leader(&self) -> Option<SchoolableFish> {
        let id = get(&self.0, "fish_leader").as_u64()?;
        Some(SchoolableFish(Entity::of(id, &self.get_world())))
    }
    /// bukkit: io/papermc/paper/entity/SchoolableFish.startFollowing
    pub fn start_following(&self, leader: &SchoolableFish) -> bool {
        done(&self.0, "fish_follow", json!({ "leader": leader.get_entity_id() }))
    }
    /// bukkit: io/papermc/paper/entity/SchoolableFish.stopFollowing
    pub fn stop_following(&self) -> bool {
        done(&self.0, "fish_follow", json!({}))
    }
}

impl Leashable {
    /// bukkit: io/papermc/paper/entity/Leashable.isLeashed
    pub fn is_leashed(&self) -> bool {
        flag(&self.0, "leashed")
    }
    /// bukkit: io/papermc/paper/entity/Leashable.getLeashHolder, org/bukkit/entity/LivingEntity.getLeashHolder
    pub fn get_leash_holder(&self) -> Option<Actor> {
        actor(&self.0, &get(&self.0, "leash_holder"))
    }
    /// What holds the lead: `{ "player": true, "uuid": ... }`, `{ "entity": id }` or `{ "knot": [x, y, z] }`.
    pub fn get_leash_holder_data(&self) -> Value {
        get(&self.0, "leash_holder")
    }
    /// bukkit: io/papermc/paper/entity/Leashable.setLeashHolder, org/bukkit/entity/LivingEntity.setLeashHolder
    ///
    /// A player, a mob, a boat or a fence knot, or nobody. False for a dead holder, as on Paper.
    pub fn set_leash_holder(&self, holder: Option<&Actor>) -> bool {
        match holder {
            Some(Actor::Entity(entity)) if entity.is_dead() => false,
            Some(holder) => done(&self.0, "leash_to", who(holder)),
            None => done(&self.0, "leash_off", json!({})),
        }
    }
    /// A player takes the lead.
    pub fn leash_to_player(&self, player: &Player) -> bool {
        done(&self.0, "leash_to", player_target(player))
    }
}

impl RangedEntity {
    /// bukkit: com/destroystokyo/paper/entity/RangedEntity.isChargingAttack
    pub fn is_charging_attack(&self) -> bool {
        flag(&self.0, "aggressive")
    }
    /// bukkit: com/destroystokyo/paper/entity/RangedEntity.setChargingAttack
    pub fn set_charging_attack(&self, raise_hands: bool) -> bool {
        put(&self.0, "aggressive", json!(raise_hands))
    }
}

// ---- Pathfinder ----

/// `com.destroystokyo.paper.entity.Pathfinder`.
#[derive(Clone, Debug, PartialEq)]
pub struct Pathfinder(pub Entity);

/// `com.destroystokyo.paper.entity.Pathfinder.PathResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct PathResult {
    world: World,
    points: Vec<[i32; 3]>,
    next: usize,
    target: [i32; 3],
    reached: bool,
}

impl Mob {
    /// bukkit: org/bukkit/entity/Mob.getPathfinder
    pub fn get_pathfinder(&self) -> Pathfinder {
        Pathfinder(self.0.clone())
    }
}

fn path_result(entity: &Entity, value: &Value) -> Option<PathResult> {
    let point = |at: &Value| Some([at.get(0)?.as_i64()? as i32, at.get(1)?.as_i64()? as i32, at.get(2)?.as_i64()? as i32]);
    Some(PathResult {
        world: entity.get_world(),
        points: value["points"].as_array()?.iter().filter_map(point).collect(),
        next: value["next"].as_u64().unwrap_or(0) as usize,
        target: point(&value["target"])?,
        reached: value["reached"].as_bool().unwrap_or(false),
    })
}

impl Pathfinder {
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.getEntity
    pub fn get_entity(&self) -> Mob {
        Mob(self.0.clone())
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.moveTo
    pub fn move_to(&self, at: &Location, speed: f64) -> bool {
        done(&self.0, "path_to", json!({ "at": [at.x, at.y, at.z], "speed": speed }))
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.stopPathfinding
    pub fn stop_pathfinding(&self) -> bool {
        done(&self.0, "path_stop", json!({}))
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.hasPath
    pub fn has_path(&self) -> bool {
        self.get_current_path().is_some_and(|path| path.next < path.points.len())
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.getCurrentPath
    pub fn get_current_path(&self) -> Option<PathResult> {
        path_result(&self.0, &act(&self.0, "path", json!({})))
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.findPath
    pub fn find_path(&self, at: &Location) -> Option<PathResult> {
        path_result(&self.0, &act(&self.0, "path_find", json!({ "at": block_of(at) })))
    }
    fn nav(&self, what: &str) -> bool {
        act(&self.0, "path_flags", json!({}))[what].as_bool().unwrap_or(false)
    }
    fn set_nav(&self, what: &str, value: bool) -> bool {
        done(&self.0, "path_set", json!({ "what": what, "value": value }))
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.canOpenDoors
    pub fn can_open_doors(&self) -> bool {
        self.nav("open_doors")
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.setCanOpenDoors
    pub fn set_can_open_doors(&self, value: bool) -> bool {
        self.set_nav("open_doors", value)
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.canPassDoors
    pub fn can_pass_doors(&self) -> bool {
        self.nav("pass_doors")
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.setCanPassDoors
    pub fn set_can_pass_doors(&self, value: bool) -> bool {
        self.set_nav("pass_doors", value)
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.canFloat
    pub fn can_float(&self) -> bool {
        self.nav("float")
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder.setCanFloat
    pub fn set_can_float(&self, value: bool) -> bool {
        self.set_nav("float", value)
    }
}

impl PathResult {
    fn location(&self, point: [i32; 3]) -> Location {
        Location::new(self.world.clone(), f64::from(point[0]), f64::from(point[1]), f64::from(point[2]), 0.0, 0.0)
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder$PathResult.getPoints
    pub fn get_points(&self) -> Vec<Location> {
        self.points.iter().map(|point| self.location(*point)).collect()
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder$PathResult.getNextPointIndex
    pub fn get_next_point_index(&self) -> usize {
        self.next
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder$PathResult.getNextPoint
    pub fn get_next_point(&self) -> Option<Location> {
        self.points.get(self.next).map(|point| self.location(*point))
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder$PathResult.getFinalPoint
    pub fn get_final_point(&self) -> Option<Location> {
        self.points.last().map(|point| self.location(*point))
    }
    /// bukkit: com/destroystokyo/paper/entity/Pathfinder$PathResult.canReachFinalPoint
    pub fn can_reach_final_point(&self) -> bool {
        self.reached && self.points.last().is_some_and(|point| *point == self.target)
    }
}

// ---- Villagers ----

/// `com.destroystokyo.paper.entity.villager.Reputation`: a player's standing with one villager, by type.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reputation(std::collections::BTreeMap<String, i32>);

const GOSSIP: [(&str, &str); 5] = [("MAJOR_NEGATIVE", "major_negative"), ("MINOR_NEGATIVE", "minor_negative"), ("MINOR_POSITIVE", "minor_positive"), ("MAJOR_POSITIVE", "major_positive"), ("TRADING", "trading")];

fn gossip_id(java: &str) -> Option<&'static str> {
    GOSSIP.iter().find(|(name, _)| *name == java).map(|(_, id)| *id)
}

fn gossip_name(id: &str) -> Option<&'static str> {
    GOSSIP.iter().find(|(_, known)| *known == id).map(|(name, _)| *name)
}

impl Reputation {
    /// bukkit: com/destroystokyo/paper/entity/villager/Reputation.getReputation
    pub fn get_reputation(&self, kind: &str) -> i32 {
        self.0.get(kind).copied().unwrap_or(0)
    }
    /// bukkit: com/destroystokyo/paper/entity/villager/Reputation.setReputation
    pub fn set_reputation(&mut self, kind: &str, value: i32) {
        self.0.insert(kind.to_owned(), value);
    }
    /// bukkit: com/destroystokyo/paper/entity/villager/Reputation.hasReputationSet
    pub fn has_reputation_set(&self, kind: &str) -> bool {
        self.0.get(kind).is_some_and(|value| *value != 0)
    }
}

impl Villager {
    /// bukkit: org/bukkit/entity/Villager.sleep
    pub fn sleep(&self, at: &Location) -> bool {
        done(&self.0, "villager_sleep", json!({ "at": block_of(at) }))
    }
    /// bukkit: org/bukkit/entity/Villager.wakeup
    pub fn wakeup(&self) -> bool {
        done(&self.0, "villager_wake", json!({}))
    }
    /// bukkit: org/bukkit/entity/Villager.shakeHead
    pub fn shake_head(&self) -> bool {
        done(&self.0, "villager_shake", json!({}))
    }
    /// bukkit: org/bukkit/entity/Villager.getReputation
    pub fn get_reputation(&self, player: &str) -> Reputation {
        let mut reputation = Reputation::default();
        for row in act(&self.0, "reputation", json!({ "uuid": words(player) })).as_array().into_iter().flatten() {
            if let (Some(name), Some(value)) = (row[0].as_str().and_then(gossip_name), row[1].as_i64()) {
                reputation.set_reputation(name, value as i32);
            }
        }
        reputation
    }
    /// bukkit: org/bukkit/entity/Villager.getReputations
    pub fn get_reputations(&self) -> std::collections::BTreeMap<String, Reputation> {
        let mut all = std::collections::BTreeMap::<String, Reputation>::new();
        for row in act(&self.0, "reputations", json!({})).as_array().into_iter().flatten() {
            if let (Some(player), Some(name), Some(value)) = (uuid_text(&row[0]), row[1].as_str().and_then(gossip_name), row[2].as_i64()) {
                all.entry(player).or_default().set_reputation(name, value as i32);
            }
        }
        all
    }
    /// bukkit: org/bukkit/entity/Villager.setReputation
    pub fn set_reputation(&self, player: &str, reputation: &Reputation) -> bool {
        GOSSIP.iter().all(|(name, id)| done(&self.0, "reputation_set", json!({ "uuid": words(player), "kind": id, "value": reputation.get_reputation(name) })))
    }
    /// bukkit: org/bukkit/entity/Villager.setReputations
    pub fn set_reputations(&self, reputations: &std::collections::BTreeMap<String, Reputation>) -> bool {
        reputations.iter().all(|(player, reputation)| self.set_reputation(player, reputation))
    }
    /// bukkit: org/bukkit/entity/Villager.clearReputations
    pub fn clear_reputations(&self) -> bool {
        done(&self.0, "reputations_clear", json!({}))
    }
    /// bukkit: org/bukkit/entity/Villager.restock
    pub fn restock(&self) -> bool {
        done(&self.0, "trader_restock", json!({}))
    }
    /// bukkit: org/bukkit/entity/Villager.updateDemand
    pub fn update_demand(&self) -> bool {
        done(&self.0, "trader_demand", json!({}))
    }
}

impl AbstractVillager {
    /// bukkit: org/bukkit/entity/AbstractVillager.resetOffers
    pub fn reset_offers(&self) -> bool {
        done(&self.0, "trader_reset", json!({}))
    }
}

/// bukkit: org/bukkit/entity/Villager$Profession.translationKey
pub fn villager_profession_translation_key(profession: &str) -> String {
    let key = profession.trim_start_matches("minecraft:").to_ascii_lowercase();
    format!("entity.minecraft.villager.{key}")
}

// ---- What isn't a mob: vehicles, projectiles, area clouds ----

fn object(entity: &Entity, key: &str, fields: Value) -> Value {
    let mut request = fields;
    request["op"] = json!("entity_object");
    request["id"] = json!(entity.get_entity_id());
    request["dimension"] = json!(entity.get_world().dimension());
    request["key"] = json!(key);
    query(request)
}

fn object_done(entity: &Entity, key: &str, fields: Value) -> bool {
    object(entity, key, fields).as_bool().unwrap_or(false)
}

fn vector_of(value: &Value) -> Vector {
    let at = |i: usize| value[i].as_f64().unwrap_or(0.0);
    Vector::new(at(0), at(1), at(2))
}

fn vector_json(vector: &Vector) -> Value {
    json!([vector.get_x(), vector.get_y(), vector.get_z()])
}

impl Minecart {
    /// bukkit: org/bukkit/entity/Minecart.getDamage
    pub fn get_damage(&self) -> f64 {
        object(&self.0, "cart", json!({}))["damage"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/entity/Minecart.setDamage
    pub fn set_damage(&self, damage: f64) -> bool {
        object_done(&self.0, "cart_damage", json!({ "value": damage }))
    }
    /// bukkit: org/bukkit/entity/Minecart.getMinecartMaterial
    pub fn get_minecart_material(&self) -> String {
        self.get_type().trim_start_matches("minecraft:").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/entity/Minecart.getDisplayBlockData
    pub fn get_display_block_data(&self) -> Option<String> {
        // A cart with no block of its own shows what its kind carries.
        let kind = self.get_type();
        let carried = match kind.as_str() {
            "minecraft:tnt_minecart" => "minecraft:tnt",
            "minecraft:chest_minecart" => "minecraft:chest",
            "minecraft:furnace_minecart" => "minecraft:furnace",
            "minecraft:hopper_minecart" => "minecraft:hopper",
            "minecraft:spawner_minecart" => "minecraft:spawner",
            "minecraft:command_block_minecart" => "minecraft:command_block",
            _ => "minecraft:air",
        };
        Some(self.data_at("DisplayState.Name").as_str().unwrap_or(carried).to_owned())
    }
}

impl ExplosiveMinecart {
    /// bukkit: org/bukkit/entity/minecart/ExplosiveMinecart.ignite
    pub fn ignite(&self) -> bool {
        object_done(&self.0, "cart_ignite", json!({}))
    }
    /// bukkit: org/bukkit/entity/minecart/ExplosiveMinecart.isIgnited
    pub fn is_ignited(&self) -> bool {
        self.data_at("fuse").as_i64().unwrap_or(-1) >= 0
    }
    /// bukkit: org/bukkit/entity/minecart/ExplosiveMinecart.explode
    pub fn explode(&self) -> bool {
        object_done(&self.0, "cart_explode", json!({}))
    }
}

impl PoweredMinecart {
    /// bukkit: org/bukkit/entity/minecart/PoweredMinecart.getPushX
    pub fn get_push_x(&self) -> f64 {
        self.data_at("PushX").as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/entity/minecart/PoweredMinecart.getPushZ
    pub fn get_push_z(&self) -> f64 {
        self.data_at("PushZ").as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/entity/minecart/PoweredMinecart.setPushX
    pub fn set_push_x(&self, push: f64) -> bool {
        self.merge(&format!("{{PushX:{push:?}d}}"))
    }
    /// bukkit: org/bukkit/entity/minecart/PoweredMinecart.setPushZ
    pub fn set_push_z(&self, push: f64) -> bool {
        self.merge(&format!("{{PushZ:{push:?}d}}"))
    }
}

impl CommandMinecart {
    /// bukkit: org/bukkit/entity/minecart/CommandMinecart.setName
    pub fn set_name(&self, name: &str) -> bool {
        self.set_custom_name(name)
    }
}

fn wood(entity: &Entity) -> String {
    entity.get_type().trim_start_matches("minecraft:").trim_end_matches("_chest_boat").trim_end_matches("_boat").trim_end_matches("_chest_raft").trim_end_matches("_raft").to_owned()
}

/// What Bukkit stores on a boat and Paper's physics never read.
#[derive(Clone, Copy, Debug, PartialEq)]
struct BoatSettings {
    max_speed: f64,
    occupied: f64,
    unoccupied: f64,
    on_land: bool,
}

static BOATS: std::sync::Mutex<std::collections::BTreeMap<u64, BoatSettings>> = std::sync::Mutex::new(std::collections::BTreeMap::new());

impl Boat {
    fn settings<R>(&self, f: impl FnOnce(&mut BoatSettings) -> R) -> R {
        let mut all = BOATS.lock().unwrap_or_else(|e| e.into_inner());
        f(all.entry(self.get_entity_id()).or_insert(BoatSettings { max_speed: 0.4, occupied: 0.2, unoccupied: -1.0, on_land: false }))
    }
    /// bukkit: org/bukkit/entity/Boat.getBoatType
    pub fn get_boat_type(&self) -> String {
        wood(&self.0).to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/entity/Boat.getBoatMaterial
    pub fn get_boat_material(&self) -> String {
        self.get_type().trim_start_matches("minecraft:").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/entity/Boat.getWoodType
    pub fn get_wood_type(&self) -> &'static str {
        match wood(&self.0).as_str() {
            "spruce" => "REDWOOD",
            "birch" => "BIRCH",
            "jungle" => "JUNGLE",
            "acacia" => "ACACIA",
            "dark_oak" => "DARK_OAK",
            _ => "GENERIC",
        }
    }
    /// bukkit: org/bukkit/entity/Boat.getStatus
    pub fn get_status(&self) -> &'static str {
        let state = object(&self.0, "boat", json!({}));
        if state["under_water"].as_bool().unwrap_or(false) {
            let top = self.get_location().map(|at| Location::new(self.get_world(), at.x, at.y + 0.57, at.z, 0.0, 0.0).get_block());
            let flowing = top.is_some_and(|block| block.get_property("level").is_some_and(|level| level != "0"));
            if flowing { "UNDER_FLOWING_WATER" } else { "UNDER_WATER" }
        } else if state["in_liquid"].as_bool().unwrap_or(false) {
            "IN_WATER"
        } else if state["on_ground"].as_bool().unwrap_or(false) {
            "ON_LAND"
        } else if state.is_null() {
            "NOT_IN_WORLD"
        } else {
            "IN_AIR"
        }
    }
    /// bukkit: org/bukkit/entity/Boat.getMaxSpeed
    ///
    /// As on Paper: the boat doesn't read it, it's only kept for the plugin.
    pub fn get_max_speed(&self) -> f64 {
        self.settings(|boat| boat.max_speed)
    }
    /// bukkit: org/bukkit/entity/Boat.setMaxSpeed
    ///
    /// As on Paper: the boat doesn't read it, it's only kept for the plugin.
    pub fn set_max_speed(&self, speed: f64) {
        self.settings(|boat| boat.max_speed = speed);
    }
    /// bukkit: org/bukkit/entity/Boat.getOccupiedDeceleration
    ///
    /// As on Paper: the boat doesn't read it, it's only kept for the plugin.
    pub fn get_occupied_deceleration(&self) -> f64 {
        self.settings(|boat| boat.occupied)
    }
    /// bukkit: org/bukkit/entity/Boat.setOccupiedDeceleration
    ///
    /// As on Paper: the boat doesn't read it, it's only kept for the plugin.
    pub fn set_occupied_deceleration(&self, rate: f64) {
        self.settings(|boat| boat.occupied = rate);
    }
    /// bukkit: org/bukkit/entity/Boat.getUnoccupiedDeceleration
    ///
    /// As on Paper: the boat doesn't read it, it's only kept for the plugin.
    pub fn get_unoccupied_deceleration(&self) -> f64 {
        self.settings(|boat| boat.unoccupied)
    }
    /// bukkit: org/bukkit/entity/Boat.setUnoccupiedDeceleration
    ///
    /// As on Paper: the boat doesn't read it, it's only kept for the plugin.
    pub fn set_unoccupied_deceleration(&self, rate: f64) {
        self.settings(|boat| boat.unoccupied = rate);
    }
    /// bukkit: org/bukkit/entity/Boat.getWorkOnLand
    ///
    /// As on Paper: the boat doesn't read it, it's only kept for the plugin.
    pub fn get_work_on_land(&self) -> bool {
        self.settings(|boat| boat.on_land)
    }
    /// bukkit: org/bukkit/entity/Boat.setWorkOnLand
    ///
    /// As on Paper: the boat doesn't read it, it's only kept for the plugin.
    pub fn set_work_on_land(&self, work: bool) {
        self.settings(|boat| boat.on_land = work);
    }
}

/// bukkit: org/bukkit/entity/Boat$Type.getMaterial
pub fn boat_type_material(kind: &str) -> String {
    if kind == "BAMBOO" { "BAMBOO_RAFT".to_owned() } else { format!("{kind}_BOAT") }
}

impl Fireball {
    fn flight(&self) -> Value {
        object(&self.0, "projectile", json!({}))
    }
    /// bukkit: org/bukkit/entity/Fireball.getAcceleration, org/bukkit/entity/Fireball.getPower, org/bukkit/entity/Fireball.getDirection
    pub fn get_acceleration(&self) -> Vector {
        let flight = self.flight();
        let mut heading = vector_of(&flight["velocity"]);
        if heading.length_squared() > 0.0 {
            heading.normalize();
            heading.multiply(flight["acceleration"].as_f64().unwrap_or(0.0));
        }
        heading
    }
    /// bukkit: org/bukkit/entity/Fireball.setAcceleration, org/bukkit/entity/Fireball.setPower
    pub fn set_acceleration(&self, acceleration: &Vector) -> bool {
        object_done(&self.0, "fireball_acceleration", json!({ "value": vector_json(acceleration) }))
    }
    /// bukkit: org/bukkit/entity/Fireball.setDirection
    pub fn set_direction(&self, direction: &Vector) -> bool {
        object_done(&self.0, "fireball_direction", json!({ "value": vector_json(direction) }))
    }
}

impl AbstractArrow {
    /// bukkit: org/bukkit/entity/AbstractArrow.getKnockbackStrength
    pub fn get_knockback_strength(&self) -> i32 {
        object(&self.0, "projectile", json!({}))["punch"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setKnockbackStrength
    pub fn set_knockback_strength(&self, strength: i32) -> bool {
        object_done(&self.0, "arrow_punch", json!({ "value": strength }))
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getAttachedBlock
    pub fn get_attached_block(&self) -> Option<crate::Block> {
        self.get_attached_blocks().into_iter().next()
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getAttachedBlocks
    pub fn get_attached_blocks(&self) -> Vec<crate::Block> {
        let Some(at) = self.get_location().filter(|_| self.is_in_block()) else {
            return Vec::new();
        };
        // The arrow's position, grown by 0.06.
        let span = |center: f64| (center - 0.06).floor() as i32..=(center + 0.06).floor() as i32;
        let mut blocks = Vec::new();
        for x in span(at.x) {
            for y in span(at.y) {
                for z in span(at.z) {
                    let block = at.get_world().get_block_at(x, y, z);
                    if block.get_type().is_solid() {
                        blocks.push(block);
                    }
                }
            }
        }
        blocks
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.getPickupRule
    pub fn get_pickup_rule(&self) -> &'static str {
        crate::enums::abstract_arrow_pickup_rule::values().get(self.data_at("pickup").as_u64().unwrap_or(0) as usize).copied().unwrap_or("DISALLOWED")
    }
    /// bukkit: org/bukkit/entity/AbstractArrow.setPickupRule
    pub fn set_pickup_rule(&self, rule: &str) -> bool {
        crate::enums::abstract_arrow_pickup_rule::values().iter().position(|name| *name == rule).is_some_and(|index| self.merge(&format!("{{pickup:{index}b}}")))
    }
}

impl ShulkerBullet {
    fn state(&self) -> Value {
        object(&self.0, "projectile", json!({}))["shulker"].take()
    }
    /// bukkit: org/bukkit/entity/ShulkerBullet.getTargetDelta
    pub fn get_target_delta(&self) -> Vector {
        vector_of(&self.state()["delta"])
    }
    /// bukkit: org/bukkit/entity/ShulkerBullet.setTargetDelta
    pub fn set_target_delta(&self, delta: &Vector) -> bool {
        object_done(&self.0, "shulker_delta", json!({ "value": vector_json(delta) }))
    }
    /// bukkit: org/bukkit/entity/ShulkerBullet.getFlightSteps
    pub fn get_flight_steps(&self) -> i32 {
        self.state()["steps"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/ShulkerBullet.setFlightSteps
    pub fn set_flight_steps(&self, steps: i32) -> bool {
        object_done(&self.0, "shulker_steps", json!({ "value": steps }))
    }
    /// bukkit: org/bukkit/entity/ShulkerBullet.getCurrentMovementDirection
    pub fn get_current_movement_direction(&self) -> Option<&'static str> {
        self.state()["dir"].as_u64().and_then(|dir| ["DOWN", "UP", "NORTH", "SOUTH", "WEST", "EAST"].get(dir as usize).copied())
    }
    /// bukkit: org/bukkit/entity/ShulkerBullet.setCurrentMovementDirection
    pub fn set_current_movement_direction(&self, face: Option<&str>) -> bool {
        let dir = face.and_then(|face| ["DOWN", "UP", "NORTH", "SOUTH", "WEST", "EAST"].iter().position(|name| *name == face));
        object_done(&self.0, "shulker_dir", json!({ "value": dir }))
    }
}

impl ThrownPotion {
    /// bukkit: org/bukkit/entity/ThrownPotion.getEffects
    pub fn get_effects(&self) -> Vec<crate::PotionEffect> {
        let potion = object(&self.0, "projectile", json!({}))["potion"].take();
        let Some(id) = potion["effect"].as_str() else {
            return self.get_item().map_or_else(Vec::new, |stack| potion_effects(&stack));
        };
        vec![crate::PotionEffect::new(id, potion["ticks"].as_i64().unwrap_or(0) as i32, potion["amplifier"].as_i64().unwrap_or(0) as i32, false, true)]
    }
}

fn potion_effects(stack: &ItemStack) -> Vec<crate::PotionEffect> {
    let contents = stack.get_component("minecraft:potion_contents").cloned().unwrap_or_default();
    contents["custom_effects"].as_array().into_iter().flatten().filter_map(crate::PotionEffect::from_saved).collect()
}

impl Trident {
    /// bukkit: org/bukkit/entity/Trident.getDamage
    pub fn get_damage(&self) -> f64 {
        plugin_get(&self.0, "damage").as_f64().unwrap_or(8.0)
    }
    /// bukkit: org/bukkit/entity/Trident.setDamage
    pub fn set_damage(&self, damage: f64) -> bool {
        plugin_set(&self.0, "damage", json!(damage))
    }
    /// bukkit: org/bukkit/entity/Trident.setGlint
    pub fn set_glint(&self, glint: bool) -> bool {
        plugin_set(&self.0, "glint", json!(glint))
    }
    /// bukkit: org/bukkit/entity/Trident.getLoyaltyLevel
    pub fn get_loyalty_level(&self) -> i32 {
        object(&self.0, "projectile", json!({}))["loyalty"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Trident.hasGlint
    pub fn has_glint(&self) -> bool {
        plugin_get(&self.0, "glint").as_f64().map_or_else(|| object(&self.0, "projectile", json!({}))["enchanted"].as_bool().unwrap_or(false), |on| on != 0.0)
    }
}

impl Projectile {
    /// bukkit: org/bukkit/entity/Projectile.doesBounce
    ///
    /// Always false: the method does nothing on Paper either.
    pub fn does_bounce(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/entity/Projectile.setBounce
    ///
    /// Does nothing: the method does nothing on Paper either.
    pub fn set_bounce(&self, _bounce: bool) {}
}

/// `potion_contents` as a thing saves it, for arrows and area effect clouds, by where it is in the saved tree.
struct Contents<'a> {
    entity: &'a Entity,
    /// The saved tree's path to the contents.
    path: &'a str,
}

impl Contents<'_> {
    fn read(&self) -> Value {
        self.entity.data_at(self.path)
    }
    fn write(&self, contents: &Value) -> bool {
        // The whole of it, so a list that shrinks shrinks: a merge keeps what it doesn't name.
        self.entity.merge(&crate::data::patch(self.path, &crate::data::snbt(contents)))
    }
    fn change(&self, f: impl FnOnce(&mut Value)) -> bool {
        let mut contents = self.read();
        if !contents.is_object() {
            contents = json!({});
        }
        f(&mut contents);
        self.write(&contents)
    }
    fn potion(&self) -> Option<String> {
        self.read()["potion"].as_str().filter(|potion| !potion.is_empty()).map(str::to_owned)
    }
    fn effects(&self) -> Vec<crate::PotionEffect> {
        self.read()["custom_effects"].as_array().into_iter().flatten().filter_map(crate::PotionEffect::from_saved).collect()
    }
    fn write_effects(&self, effects: &[crate::PotionEffect]) -> bool {
        self.change(|contents| contents["custom_effects"] = effects.iter().map(crate::PotionEffect::to_saved).collect())
    }
    fn has_effect(&self, effect: &str) -> bool {
        self.effects().iter().any(|known| known.is(effect))
    }
    fn add(&self, effect: &crate::PotionEffect, overwrite: bool) -> bool {
        let mut effects = self.effects();
        match effects.iter().position(|known| known.is(effect.get_type())) {
            Some(_) if !overwrite => return false,
            Some(index) => effects[index] = effect.clone(),
            None => effects.push(effect.clone()),
        }
        self.write_effects(&effects)
    }
    fn remove(&self, effect: &str) -> bool {
        let mut effects = self.effects();
        let before = effects.len();
        effects.retain(|known| !known.is(effect));
        effects.len() != before && self.write_effects(&effects)
    }
    fn color(&self) -> Option<crate::Color> {
        let rgb = self.read()["custom_color"].as_i64().filter(|rgb| *rgb >= 0)?;
        Some(crate::Color::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8))
    }
    fn set_color(&self, color: Option<&crate::Color>) -> bool {
        self.change(|contents| {
            if let Some(object) = contents.as_object_mut() {
                match color {
                    Some(color) => object.insert("custom_color".to_owned(), json!(color.as_rgb() as i64)),
                    None => object.remove("custom_color"),
                };
            }
        })
    }
    fn set_potion(&self, kind: &str) -> bool {
        let id = if kind.contains(':') { kind.to_owned() } else { format!("minecraft:{}", kind.to_ascii_lowercase()) };
        self.change(|contents| contents["potion"] = json!(id))
    }
}

impl Arrow {
    fn contents(&self) -> Contents<'_> {
        Contents { entity: &self.0, path: "item.components.minecraft:potion_contents" }
    }
    /// bukkit: org/bukkit/entity/Arrow.getBasePotionType
    pub fn get_base_potion_type(&self) -> Option<String> {
        self.contents().potion()
    }
    /// bukkit: org/bukkit/entity/Arrow.setBasePotionType
    pub fn set_base_potion_type(&self, kind: &str) -> bool {
        self.contents().set_potion(kind)
    }
    /// bukkit: org/bukkit/entity/Arrow.getBasePotionData
    pub fn get_base_potion_data(&self) -> Option<crate::core3::PotionData> {
        self.contents().potion().map(|id| crate::core3::PotionData::from_id(&id))
    }
    /// bukkit: org/bukkit/entity/Arrow.setBasePotionData
    pub fn set_base_potion_data(&self, data: &crate::core3::PotionData) -> bool {
        self.contents().set_potion(&data_id(data))
    }
    /// bukkit: org/bukkit/entity/Arrow.getCustomEffects
    pub fn get_custom_effects(&self) -> Vec<crate::PotionEffect> {
        self.contents().effects()
    }
    /// bukkit: org/bukkit/entity/Arrow.hasCustomEffects
    pub fn has_custom_effects(&self) -> bool {
        !self.contents().effects().is_empty()
    }
    /// bukkit: org/bukkit/entity/Arrow.hasCustomEffect
    pub fn has_custom_effect(&self, effect: &str) -> bool {
        self.contents().has_effect(effect)
    }
    /// bukkit: org/bukkit/entity/Arrow.addCustomEffect
    pub fn add_custom_effect(&self, effect: &crate::PotionEffect, overwrite: bool) -> bool {
        self.contents().add(effect, overwrite)
    }
    /// bukkit: org/bukkit/entity/Arrow.removeCustomEffect
    pub fn remove_custom_effect(&self, effect: &str) -> bool {
        self.contents().remove(effect)
    }
    /// bukkit: org/bukkit/entity/Arrow.clearCustomEffects
    pub fn clear_custom_effects(&self) -> bool {
        self.contents().write_effects(&[])
    }
    /// bukkit: org/bukkit/entity/Arrow.getColor
    pub fn get_color(&self) -> Option<crate::Color> {
        self.contents().color()
    }
    /// bukkit: org/bukkit/entity/Arrow.setColor
    pub fn set_color(&self, color: Option<&crate::Color>) -> bool {
        self.contents().set_color(color)
    }
}

fn data_id(data: &crate::core3::PotionData) -> String {
    let kind = data.get_type().to_ascii_lowercase();
    if data.is_extended() {
        format!("minecraft:long_{kind}")
    } else if data.is_upgraded() {
        format!("minecraft:strong_{kind}")
    } else {
        format!("minecraft:{kind}")
    }
}

impl AreaEffectCloud {
    fn contents(&self) -> Contents<'_> {
        Contents { entity: &self.0, path: "potion_contents" }
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setSource
    ///
    /// What lives owns it; a block's source or anything else leaves it with no owner, as on Paper.
    pub fn set_source(&self, source: Option<&Actor>) -> bool {
        let owner = match source {
            Some(Actor::Player(player)) => Some(player.get_unique_id()),
            Some(Actor::Entity(entity)) if living(entity) => Some(entity.get_unique_id()),
            _ => None,
        };
        self.merge(&crate::data::patch("Owner", &crate::data::uuid_snbt(owner.as_deref().unwrap_or("00000000-0000-0000-0000-000000000000")).unwrap_or_default()))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setBasePotionType
    pub fn set_base_potion_type(&self, potion: Option<&str>) -> bool {
        // A merge can't take a tag away: an empty id is no potion.
        self.contents().change(|contents| contents["potion"] = json!(potion.map_or(String::new(), entity_key)))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setBasePotionData
    pub fn set_base_potion_data(&self, data: Option<&crate::core3::PotionData>) -> bool {
        let id = data.map(|data| format!("{}{}", if data.is_extended() { "long_" } else if data.is_upgraded() { "strong_" } else { "" }, data.get_type().trim_start_matches("minecraft:")));
        self.set_base_potion_type(id.as_deref())
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.addCustomEffect
    pub fn add_custom_effect(&self, effect: &crate::PotionEffect, overwrite: bool) -> bool {
        self.contents().add(effect, overwrite)
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.removeCustomEffect
    pub fn remove_custom_effect(&self, effect: &str) -> bool {
        let mut effects = self.contents().effects();
        let before = effects.len();
        effects.retain(|known| !known.is(effect));
        effects.len() != before && self.contents().write_effects(&effects)
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.clearCustomEffects
    pub fn clear_custom_effects(&self) -> bool {
        self.contents().write_effects(&[])
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setColor
    pub fn set_color(&self, color: Option<&crate::Color>) -> bool {
        // A merge can't take a tag away: -1 is no colour of its own.
        self.contents().change(|contents| contents["custom_color"] = json!(color.map_or(-1, |color| color.as_rgb() as i64)))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.setParticle
    ///
    /// `data` is what `ParticleBuilder.data` takes for that particle, for the overload with one.
    pub fn set_particle(&self, particle: &str, data: Option<&Value>) -> bool {
        let done = self.merge(&format!("{{custom_particle:{{type:{}}}}}", crate::data::quoted(&entity_key(particle))));
        done && plugin_set(&self.0, "particle_data", data.map_or(Value::Null, |data| json!(data.to_string())))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getSource
    ///
    /// `None` when the owner isn't loaded, as on Paper.
    pub fn get_source(&self) -> Option<Actor> {
        actor(&self.0, &json!({ "uuid": self.get_owner_unique_id()? }))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getBasePotionType
    pub fn get_base_potion_type(&self) -> Option<String> {
        self.contents().potion()
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getBasePotionData
    pub fn get_base_potion_data(&self) -> Option<crate::core3::PotionData> {
        self.contents().potion().map(|id| crate::core3::PotionData::from_id(&id))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getCustomEffects
    pub fn get_custom_effects(&self) -> Vec<crate::PotionEffect> {
        self.contents().effects()
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.hasCustomEffects
    pub fn has_custom_effects(&self) -> bool {
        !self.contents().effects().is_empty()
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.hasCustomEffect
    pub fn has_custom_effect(&self, effect: &str) -> bool {
        self.contents().has_effect(effect)
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getColor
    ///
    /// A cloud with no colour of its own has `PotionContents.BASE_POTION_COLOR`.
    pub fn get_color(&self) -> Option<crate::Color> {
        self.contents().color().or_else(|| Some(crate::Color::from_rgb(0x38, 0x5D, 0xC6)))
    }
    /// bukkit: org/bukkit/entity/AreaEffectCloud.getParticle
    pub fn get_particle(&self) -> Option<String> {
        self.data_at("custom_particle.type").as_str().map(str::to_owned)
    }
}

impl Display {
    /// bukkit: org/bukkit/entity/Display.getGlowColorOverride
    pub fn get_glow_color_override(&self) -> Option<crate::Color> {
        let rgb = self.data_at("glow_color_override").as_i64().filter(|rgb| *rgb != -1)?;
        Some(crate::Color::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8))
    }
    /// bukkit: org/bukkit/entity/Display.setGlowColorOverride
    pub fn set_glow_color_override(&self, color: Option<&crate::Color>) -> bool {
        self.merge(&format!("{{glow_color_override:{}}}", color.map_or(-1, |color| color.as_rgb() as i64)))
    }
    /// bukkit: org/bukkit/entity/Display.getBrightness
    pub fn get_brightness(&self) -> Option<Brightness> {
        let saved = self.data_at("brightness");
        Some(Brightness { block: saved["block"].as_i64()? as i32, sky: saved["sky"].as_i64()? as i32 })
    }
    /// bukkit: org/bukkit/entity/Display.setBrightness
    pub fn set_brightness(&self, brightness: Option<Brightness>) -> bool {
        match brightness {
            Some(light) => self.merge(&format!("{{brightness:{{block:{},sky:{}}}}}", light.block, light.sky)),
            None => self.merge("{brightness:{block:-1,sky:-1}}"),
        }
    }
    /// bukkit: org/bukkit/entity/Display.setTransformationMatrix
    pub fn set_transformation_matrix(&self, matrix: &[f32; 16]) -> bool {
        let list: Vec<String> = matrix.iter().map(|value| format!("{value:?}f")).collect();
        self.merge(&format!("{{transformation:[{}]}}", list.join(",")))
    }
}

/// `org.bukkit.entity.Display.Brightness`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Brightness {
    block: i32,
    sky: i32,
}

impl Brightness {
    /// bukkit: org/bukkit/entity/Display$Brightness.<init>
    pub fn new(block: i32, sky: i32) -> Self {
        Self { block, sky }
    }
    /// bukkit: org/bukkit/entity/Display$Brightness.getBlockLight
    pub fn get_block_light(&self) -> i32 {
        self.block
    }
    /// bukkit: org/bukkit/entity/Display$Brightness.getSkyLight
    pub fn get_sky_light(&self) -> i32 {
        self.sky
    }
}

impl Hanging {
    /// bukkit: org/bukkit/entity/Hanging.setFacingDirection
    pub fn set_facing_direction(&self, face: &str, force: bool) -> bool {
        if !force {
            let Some(at) = self.get_location() else {
                return false;
            };
            let (x, y, z) = match face {
                "NORTH" => (0.0, 0.0, 1.0),
                "SOUTH" => (0.0, 0.0, -1.0),
                "WEST" => (1.0, 0.0, 0.0),
                "EAST" => (-1.0, 0.0, 0.0),
                "UP" => (0.0, -1.0, 0.0),
                "DOWN" => (0.0, 1.0, 0.0),
                _ => return false,
            };
            let wall = Location::new(self.get_world(), at.x + x, at.y + y, at.z + z, 0.0, 0.0).get_block();
            if !wall.get_type().is_solid() {
                return false;
            }
        }
        if is_type(&self.0, &["minecraft:painting"]) {
            let Some(facing) = ["SOUTH", "WEST", "NORTH", "EAST"].iter().position(|name| *name == face) else {
                return false;
            };
            return self.merge(&format!("{{facing:{facing}b}}"));
        }
        let Some(facing) = ["DOWN", "UP", "NORTH", "SOUTH", "WEST", "EAST"].iter().position(|name| *name == face) else {
            return false;
        };
        self.merge(&format!("{{Facing:{facing}b}}"))
    }
}

// ---- Boss fights ----

/// `org.bukkit.entity.EnderDragon`.
#[derive(Clone, Debug, PartialEq)]
pub struct EnderDragon(pub Entity);

impl std::ops::Deref for EnderDragon {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for EnderDragon {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl EnderDragon {
    /// bukkit: org/bukkit/entity/EnderDragon.getPhase
    pub fn get_phase(&self) -> &'static str {
        crate::enums::ender_dragon_phase::values().get(self.data_at("DragonPhase").as_u64().unwrap_or(0) as usize).copied().unwrap_or("CIRCLING")
    }
    /// bukkit: org/bukkit/entity/EnderDragon.setPhase
    pub fn set_phase(&self, phase: &str) -> bool {
        crate::enums::ender_dragon_phase::values().iter().position(|name| *name == phase).is_some_and(|index| self.merge(&format!("{{DragonPhase:{index}}}")))
    }
    /// bukkit: org/bukkit/entity/EnderDragon.getDeathAnimationTicks
    pub fn get_death_animation_ticks(&self) -> i32 {
        self.data_at("DragonDeathTime").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/ComplexLivingEntity.getParts
    pub fn get_parts(&self) -> Vec<EnderDragonPart> {
        (1..=8).map(|offset| EnderDragonPart { id: self.get_entity_id() + offset, parent: self.clone() }).collect()
    }
}

/// `org.bukkit.entity.EnderDragonPart`: a dragon's body parts have ids and nothing else of their own.
#[derive(Clone, Debug, PartialEq)]
pub struct EnderDragonPart {
    id: u64,
    parent: EnderDragon,
}

impl EnderDragonPart {
    /// The part's own id.
    pub fn get_entity_id(&self) -> u64 {
        self.id
    }
    /// bukkit: org/bukkit/entity/EnderDragonPart.getParent, org/bukkit/entity/ComplexEntityPart.getParent
    pub fn get_parent(&self) -> &EnderDragon {
        &self.parent
    }
}

// ---- Players ----

/// What Bukkit remembers of a player and the game doesn't.
#[derive(Clone, Debug, Default)]
struct Noted {
    time: Option<(i64, bool)>,
    weather: Option<String>,
    list_name: Option<Component>,
    list_order: i32,
    unlisted: Vec<String>,
    pack_hash: Option<String>,
    border: Option<crate::world::WorldBorder>,
    waypoint_style: Option<String>,
    waypoint_color: Option<crate::Color>,
}

static NOTED: std::sync::Mutex<std::collections::BTreeMap<String, Noted>> = std::sync::Mutex::new(std::collections::BTreeMap::new());

fn noted<R>(uuid: &str, f: impl FnOnce(&mut Noted) -> R) -> R {
    f(NOTED.lock().unwrap_or_else(|e| e.into_inner()).entry(uuid.to_owned()).or_default())
}

/// `Player::send` tells this what a plugin's packet changed, for the getters that have no other source.
pub(crate) fn note_send(player: &Player, packet: &str, fields: &Value) {
    match packet {
        "player_time" => {
            let (time, relative) = (fields["time"].as_i64().unwrap_or(0), fields["relative"].as_bool().unwrap_or(true));
            let offset = if relative { time - player.get_world().get_time() } else { time };
            noted(player.uuid(), |note| note.time = (offset != 0 || !relative).then_some((offset, relative)));
            SKY_RUN.get_or_init(|| {
                crate::scheduler::run_task_timer(1, 1, keep_sky);
            });
        }
        "game_event" if matches!(fields["event"].as_i64(), Some(1 | 2)) => {
            let weather = if fields["event"].as_i64() == Some(2) { "DOWNFALL" } else { "CLEAR" };
            noted(player.uuid(), |note| note.weather = Some(weather.to_owned()));
            SKY_RUN.get_or_init(|| {
                crate::scheduler::run_task_timer(1, 1, keep_sky);
            });
        }
        "resource_pack_push" => noted(player.uuid(), |note| note.pack_hash = fields["hash"].as_str().map(str::to_owned)),
        "list_name" => noted(player.uuid(), |note| note.list_name = fields.get("name").filter(|name| !name.is_null()).cloned().map(Component)),
        "list_order" => noted(player.uuid(), |note| note.list_order = fields["order"].as_i64().unwrap_or(0) as i32),
        "listed" => {
            let target = fields["target"].as_str().unwrap_or_default().to_owned();
            let listed = fields["listed"].as_bool().unwrap_or(true);
            noted(player.uuid(), |note| {
                note.unlisted.retain(|known| *known != target);
                if !listed {
                    note.unlisted.push(target);
                }
            });
        }
        _ => {}
    }
}

/// `reset_player_weather` tells this: the weather it sends then isn't this player's own.
pub(crate) fn note_weather_reset(player: &Player) {
    noted(player.uuid(), |note| note.weather = None);
}

fn ask_player(player: &Player, key: &str) -> Value {
    query(json!({ "op": "entity_player_get", "uuid": player.uuid(), "key": key }))
}

fn set_player(player: &Player, key: &str, value: Value) -> bool {
    query(json!({ "op": "entity_player_set", "uuid": player.uuid(), "key": key, "value": value })).as_bool().unwrap_or(false)
}

fn send_player(player: &Player, kind: &str, fields: Value) -> bool {
    let mut request = fields;
    request["op"] = json!("entity_player_send");
    request["uuid"] = json!(player.uuid());
    request["kind"] = json!(kind);
    query(request).as_bool().unwrap_or(false)
}

/// The stack as the engine reads it back: what `save::item_from_nbt` takes.
fn stack_snbt(stack: Option<&ItemStack>) -> Value {
    stack.map_or(Value::Null, |stack| json!(stack.to_snbt()))
}

const SLOTS: [&str; 8] = ["HAND", "OFF_HAND", "FEET", "LEGS", "CHEST", "HEAD", "BODY", "SADDLE"];

impl Player {
    /// bukkit: org/bukkit/entity/Player.getScoreboard
    pub fn get_scoreboard(&self) -> ShownScoreboard {
        self.get_own_scoreboard().map_or_else(|| ShownScoreboard::Game(crate::scoreboard::get_main_scoreboard()), ShownScoreboard::Plugin)
    }
    /// bukkit: org/bukkit/entity/Player.openBook
    pub fn open_book(&self, book: &ItemStack) -> bool {
        send_player(self, "open_book", json!({ "stack": book.to_snbt() }))
    }
    /// bukkit: org/bukkit/entity/Player.sendEquipmentChange
    pub fn send_equipment_change(&self, entity_id: u64, slot: &str, stack: Option<&ItemStack>) -> bool {
        let Some(slot) = SLOTS.iter().position(|name| *name == slot) else {
            return false;
        };
        send_player(self, "equipment", json!({ "entity": entity_id, "slot": slot, "stack": stack_snbt(stack) }))
    }
    /// bukkit: org/bukkit/entity/Player.sendPotionEffectChange
    pub fn send_potion_effect_change(&self, entity_id: u64, effect: &crate::PotionEffect) -> bool {
        send_player(self, "potion", json!({ "entity": entity_id, "effect": effect.get_type(), "amplifier": effect.get_amplifier(), "duration": effect.get_duration(), "ambient": effect.is_ambient(), "particles": effect.has_particles() }))
    }
    /// bukkit: org/bukkit/entity/Player.sendPotionEffectChangeRemove
    pub fn send_potion_effect_change_remove(&self, entity_id: u64, effect: &str) -> bool {
        send_player(self, "potion_remove", json!({ "entity": entity_id, "effect": effect }))
    }
    /// bukkit: org/bukkit/entity/Player.sendSignChange
    pub fn send_sign_change(&self, at: &Location, lines: &[Component]) -> bool {
        let mut messages: Vec<Value> = lines.iter().map(|line| line.0.clone()).collect();
        messages.resize(4, json!(""));
        let text = crate::data::snbt(&json!({ "messages": messages, "has_glowing_text": false }));
        send_player(self, "block_entity", json!({ "pos": block_of(at), "snbt": format!("{{front_text:{text},back_text:{text}}}") }))
    }
    /// bukkit: org/bukkit/entity/Player.sendBlockUpdate
    pub fn send_block_update(&self, at: &Location, data: &Value) -> bool {
        send_player(self, "block_entity", json!({ "pos": block_of(at), "snbt": crate::data::snbt(data) }))
    }
    /// bukkit: org/bukkit/entity/Player.openVirtualSign
    pub fn open_virtual_sign(&self, at: &Location, front: bool) {
        self.open_sign(at, front);
    }
    /// bukkit: org/bukkit/entity/Player.getPlayerTimeOffset
    pub fn get_player_time_offset(&self) -> i64 {
        noted(self.uuid(), |note| note.time.map_or(0, |(offset, _)| offset))
    }
    /// bukkit: org/bukkit/entity/Player.isPlayerTimeRelative
    pub fn is_player_time_relative(&self) -> bool {
        noted(self.uuid(), |note| note.time.is_none_or(|(_, relative)| relative))
    }
    /// bukkit: org/bukkit/entity/Player.getPlayerTime
    pub fn get_player_time(&self) -> i64 {
        let offset = self.get_player_time_offset();
        if self.is_player_time_relative() { self.get_world().get_time() + offset } else { offset }
    }
    /// bukkit: org/bukkit/entity/Player.getPlayerWeather
    pub fn get_player_weather(&self) -> Option<String> {
        noted(self.uuid(), |note| note.weather.clone())
    }
    /// bukkit: org/bukkit/entity/Player.getPlayerListName
    pub fn get_player_list_name(&self) -> String {
        noted(self.uuid(), |note| note.list_name.as_ref().map(Component::to_legacy)).unwrap_or_else(|| self.get_name().to_owned())
    }
    /// bukkit: org/bukkit/entity/Player.getPlayerListOrder
    pub fn get_player_list_order(&self) -> i32 {
        noted(self.uuid(), |note| note.list_order)
    }
    /// bukkit: org/bukkit/entity/Player.isListed
    pub fn is_listed(&self, other: &Self) -> bool {
        noted(self.uuid(), |note| !note.unlisted.iter().any(|known| known == other.uuid()))
    }
    /// bukkit: org/bukkit/entity/Player.getSpectatorTarget
    pub fn get_spectator_target(&self) -> Option<Entity> {
        Some(Entity::of(ask_player(self, "spectator_target").as_u64()?, &self.get_world()))
    }
    /// bukkit: org/bukkit/entity/Player.setSprinting
    pub fn set_sprinting(&self, sprinting: bool) -> bool {
        set_player(self, "sprinting", json!(sprinting))
    }
    /// bukkit: org/bukkit/entity/Player.setSneaking
    pub fn set_sneaking(&self, sneaking: bool) -> bool {
        set_player(self, "sneaking", json!(sneaking))
    }
    /// bukkit: org/bukkit/entity/Player.getSimulationDistance
    pub fn get_simulation_distance(&self) -> i32 {
        ask_player(self, "simulation_distance").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.setViewDistance, org/bukkit/entity/Player.setSendViewDistance
    pub fn set_view_distance(&self, distance: i32) -> bool {
        set_player(self, "view_distance", json!(distance))
    }
    /// bukkit: org/bukkit/entity/Player.getIdleDuration
    pub fn get_idle_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(ask_player(self, "idle_ticks").as_u64().unwrap_or(0) * 50)
    }
    /// bukkit: org/bukkit/entity/Player.resetIdleDuration
    pub fn reset_idle_duration(&self) -> bool {
        set_player(self, "idle", Value::Null)
    }
    /// bukkit: org/bukkit/entity/Player.hasSeenWinScreen
    pub fn has_seen_win_screen(&self) -> bool {
        ask_player(self, "seen_credits").as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Player.setHasSeenWinScreen
    pub fn set_has_seen_win_screen(&self, seen: bool) -> bool {
        set_player(self, "seen_credits", json!(seen))
    }
    /// bukkit: org/bukkit/entity/Player.resetFlyingTicks
    pub fn reset_flying_ticks(&self) -> bool {
        set_player(self, "above_ground", Value::Null)
    }
    /// bukkit: org/bukkit/entity/Player.getSentChunkKeys
    pub fn get_sent_chunk_keys(&self) -> Vec<i64> {
        self.sent_chunks().into_iter().map(|[x, z]| (i64::from(x) & 0xFFFF_FFFF) | (i64::from(z) << 32)).collect()
    }
    fn sent_chunks(&self) -> Vec<[i32; 2]> {
        let pair = |at: &Value| Some([at.get(0)?.as_i64()? as i32, at.get(1)?.as_i64()? as i32]);
        ask_player(self, "sent_chunks").as_array().into_iter().flatten().filter_map(pair).collect()
    }
    /// bukkit: org/bukkit/entity/Player.getSentChunks
    pub fn get_sent_chunks(&self) -> Vec<(i32, i32)> {
        self.sent_chunks().into_iter().map(|[x, z]| (x, z)).collect()
    }
    /// bukkit: org/bukkit/entity/Player.isChunkSent
    pub fn is_chunk_sent(&self, x: i32, z: i32) -> bool {
        self.sent_chunks().contains(&[x, z])
    }
    /// bukkit: org/bukkit/entity/Player.getSocketAddress, org/bukkit/entity/Player.getAddress, org/bukkit/entity/Player$Spigot.getRawAddress
    pub fn get_socket_address(&self) -> Option<std::net::SocketAddr> {
        ask_player(self, "address").as_str()?.parse().ok()
    }
    /// bukkit: org/bukkit/entity/Player$Spigot.respawn
    pub fn respawn(&self) -> bool {
        query(json!({ "op": "ev_player_respawn_plugin", "uuid": self.uuid() }));
        send_player(self, "respawn", json!({}))
    }
    /// bukkit: org/bukkit/entity/Player.playNote
    pub fn play_note(&self, at: &Location, instrument: &str, note: u8) {
        let pitch = 2f32.powf((f32::from(note) - 12.0) / 12.0);
        self.play_sound(at, &format!("minecraft:block.note_block.{}", instrument.to_ascii_lowercase()), 3.0, pitch);
    }
    /// bukkit: org/bukkit/entity/Player.updateTitle
    pub fn update_title(&self, title: &Component, subtitle: &Component, fade_in: u32, stay: u32, fade_out: u32) {
        self.show_title(title, subtitle, fade_in, stay, fade_out);
    }
    /// bukkit: org/bukkit/entity/Player.applyMending
    pub fn apply_mending(&self, amount: i32) -> i32 {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "mending", "value": amount })).as_i64().unwrap_or(i64::from(amount)) as i32
    }
    /// bukkit: org/bukkit/entity/Player.getCooledAttackStrength, org/bukkit/entity/HumanEntity.getAttackCooldown
    pub fn get_cooled_attack_strength(&self) -> f32 {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "attack_strength" })).as_f64().unwrap_or(1.0) as f32
    }
    /// bukkit: org/bukkit/entity/Player.getCooldownPeriod
    pub fn get_cooldown_period(&self) -> f32 {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "attack_delay" })).as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/Player.getEnderPearls
    pub fn get_ender_pearls(&self) -> Vec<Entity> {
        let world = self.get_world();
        ask_player(self, "ender_pearls").as_array().into_iter().flatten().filter_map(Value::as_u64).map(|id| Entity::of(id, &world)).collect()
    }
    /// bukkit: org/bukkit/entity/Player.postEffects
    pub fn post_effects(&self) -> PlayerPostEffects {
        PlayerPostEffects(self.clone())
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getEquipment
    pub fn get_equipment(&self) -> crate::PlayerInventory {
        self.get_inventory()
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setItemOnCursor
    pub fn set_item_on_cursor(&self, stack: Option<&ItemStack>) -> bool {
        send_player(self, "cursor", json!({ "stack": stack_snbt(stack) }))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getEnchantmentSeed
    pub fn get_enchantment_seed(&self) -> i32 {
        ask_player(self, "xp_seed").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setEnchantmentSeed
    pub fn set_enchantment_seed(&self, seed: i32) -> bool {
        set_player(self, "xp_seed", json!(seed))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.isDeeplySleeping
    pub fn is_deeply_sleeping(&self) -> bool {
        self.is_sleeping() && self.get_sleep_ticks() >= 100
    }
    /// bukkit: org/bukkit/entity/HumanEntity.wakeup
    pub fn wakeup(&self) -> bool {
        send_player(self, "wake", json!({}))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setLastDeathLocation
    pub fn set_last_death_location(&self, at: Option<&Location>) -> bool {
        set_player(self, "last_death", at.map_or(Value::Null, |at| json!({ "dimension": at.world.dimension(), "pos": block_of(at) })))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.dropItem
    pub fn drop_item(&self, all: bool) -> bool {
        send_player(self, "drop", json!({ "all": all }))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setWindowProperty
    pub fn set_window_property(&self, property: i32, value: i32) -> bool {
        send_player(self, "window_property", json!({ "property": property, "value": value }))
    }
    fn open_block(&self, at: Option<&Location>, name: &str, force: bool) -> bool {
        let spot = at.cloned().unwrap_or_else(|| self.get_location());
        if !force && !spot.get_block().get_type().key().ends_with(name.trim_start_matches("minecraft:")) {
            return false;
        }
        send_player(self, "open_station", json!({ "pos": block_of(&spot), "block": name, "force": force }))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openWorkbench
    pub fn open_workbench(&self, at: Option<&Location>, force: bool) -> bool {
        self.open_block(at, "minecraft:crafting_table", force)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openEnchanting
    pub fn open_enchanting(&self, at: Option<&Location>, force: bool) -> bool {
        self.open_block(at, "minecraft:enchanting_table", force)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openAnvil
    pub fn open_anvil(&self, at: Option<&Location>, force: bool) -> bool {
        self.open_block(at, "minecraft:anvil", force)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openCartographyTable
    pub fn open_cartography_table(&self, at: Option<&Location>, force: bool) -> bool {
        self.open_block(at, "minecraft:cartography_table", force)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openGrindstone
    pub fn open_grindstone(&self, at: Option<&Location>, force: bool) -> bool {
        self.open_block(at, "minecraft:grindstone", force)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openLoom
    pub fn open_loom(&self, at: Option<&Location>, force: bool) -> bool {
        self.open_block(at, "minecraft:loom", force)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openSmithingTable
    pub fn open_smithing_table(&self, at: Option<&Location>, force: bool) -> bool {
        self.open_block(at, "minecraft:smithing_table", force)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openStonecutter
    pub fn open_stonecutter(&self, at: Option<&Location>, force: bool) -> bool {
        self.open_block(at, "minecraft:stonecutter", force)
    }
    // ---- The health bar's scale ----
    /// bukkit: org/bukkit/entity/Player.isHealthScaled
    pub fn is_health_scaled(&self) -> bool {
        self.player_plugin("health_scaled").as_f64() == Some(1.0)
    }
    /// bukkit: org/bukkit/entity/Player.getHealthScale
    pub fn get_health_scale(&self) -> f64 {
        self.player_plugin("health_scale").as_f64().unwrap_or(20.0)
    }
    /// bukkit: org/bukkit/entity/Player.setHealthScaled
    pub fn set_health_scaled(&self, scaled: bool) {
        self.player_set("plugin:health_scaled", json!(if scaled { 1.0 } else { 0.0 }));
        send_player(self, "health_scale", json!({ "scale": scaled.then(|| self.get_health_scale()) }));
    }
    /// bukkit: org/bukkit/entity/Player.setHealthScale
    pub fn set_health_scale(&self, scale: f64) {
        self.player_set("plugin:health_scale", json!(scale));
        self.set_health_scaled(true);
    }
    /// bukkit: org/bukkit/entity/Player.boostElytra, org/bukkit/entity/HumanEntity.fireworkBoost
    pub fn boost_elytra(&self, firework: &ItemStack) -> Option<Entity> {
        let id = query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "boost", "stack": firework.to_snbt() })).as_u64()?;
        Some(Entity::of(id, &self.get_world()))
    }
}

/// `io.papermc.paper.entity.PlayerPostEffects`: the post effects a player's client runs.
#[derive(Clone, Debug)]
pub struct PlayerPostEffects(Player);

impl PlayerPostEffects {
    /// bukkit: io/papermc/paper/entity/PlayerPostEffects.values
    pub fn values(&self) -> Vec<String> {
        ask_player(&self.0, "post_effects").as_array().into_iter().flatten().filter_map(|key| key.as_str().map(str::to_owned)).collect()
    }
    fn write(&self, effects: &[String]) -> bool {
        set_player(&self.0, "post_effects", json!(effects))
    }
    /// bukkit: io/papermc/paper/entity/PlayerPostEffects.set
    pub fn set(&self, effects: &[String]) -> bool {
        self.values() != effects && self.write(effects)
    }
    /// bukkit: io/papermc/paper/entity/PlayerPostEffects.add
    pub fn add(&self, effect: &str) -> bool {
        let mut effects = self.values();
        if effects.iter().any(|known| known == effect) {
            return false;
        }
        effects.push(effect.to_owned());
        self.write(&effects)
    }
    /// bukkit: io/papermc/paper/entity/PlayerPostEffects.remove
    pub fn remove(&self, effect: &str) -> bool {
        let mut effects = self.values();
        let before = effects.len();
        effects.retain(|known| known != effect);
        effects.len() != before && self.write(&effects)
    }
    /// bukkit: io/papermc/paper/entity/PlayerPostEffects.clear
    pub fn clear(&self) -> bool {
        !self.values().is_empty() && self.write(&[])
    }
    /// bukkit: io/papermc/paper/entity/PlayerPostEffects.update
    pub fn update(&self) -> bool {
        set_player(&self.0, "post_effects_send", Value::Null)
    }
}

// ---- The rest of the mob classes ----

impl Shearable {
    /// bukkit: org/bukkit/entity/Shearable.isSheared
    pub fn is_sheared(&self) -> bool {
        flag(&self.0, "sheared")
    }
    /// bukkit: org/bukkit/entity/Shearable.setSheared
    pub fn set_sheared(&self, sheared: bool) -> bool {
        put(&self.0, "sheared", json!(sheared))
    }
    /// bukkit: io/papermc/paper/entity/Shearable.readyToBeSheared
    pub fn ready_to_be_sheared(&self) -> bool {
        !self.is_sheared() && !self.is_dead() && self.data_at("Age").as_i64().unwrap_or(0) >= 0
    }
}

impl Bucketable {
    /// bukkit: io/papermc/paper/entity/Bucketable.getBaseBucketItem
    pub fn get_base_bucket_item(&self) -> Option<ItemStack> {
        let name = self.get_type();
        let bucket = format!("{name}_bucket");
        Some(ItemStack::new(crate::Material::of(&bucket), 1))
    }
    /// bukkit: io/papermc/paper/entity/Bucketable.getPickupSound
    pub fn get_pickup_sound(&self) -> &'static str {
        match self.get_type().as_str() {
            "minecraft:axolotl" => "minecraft:item.bucket.fill_axolotl",
            "minecraft:tadpole" => "minecraft:item.bucket.fill_tadpole",
            _ => "minecraft:item.bucket.fill_fish",
        }
    }
}

/// bukkit: io/papermc/paper/entity/RemovalReason.shouldDestroy
pub fn removal_reason_should_destroy(reason: &str) -> bool {
    matches!(reason, "KILLED" | "DISCARDED")
}

/// bukkit: io/papermc/paper/entity/RemovalReason.shouldSave
pub fn removal_reason_should_save(reason: &str) -> bool {
    reason == "UNLOADED_TO_CHUNK"
}

impl Husk {
    /// bukkit: org/bukkit/entity/Husk.isConverting
    pub fn is_converting(&self) -> bool {
        flag(&self.0, "zombie_drowning")
    }
}

impl Guardian {
    /// bukkit: org/bukkit/entity/Guardian.isElder
    pub fn is_elder(&self) -> bool {
        is_type(&self.0, &["minecraft:elder_guardian"])
    }
    /// bukkit: org/bukkit/entity/Guardian.getLaserDuration
    pub fn get_laser_duration(&self) -> i32 {
        if self.is_elder() { 60 } else { 80 }
    }
    /// bukkit: org/bukkit/entity/Guardian.setLaser
    pub fn set_laser(&self, activated: bool) -> bool {
        done(&self.0, "guardian_laser", json!({ "value": activated }))
    }
}

impl Frog {
    /// bukkit: org/bukkit/entity/Frog.getTongueTarget
    pub fn get_tongue_target(&self) -> Option<Entity> {
        Some(Entity::of(get(&self.0, "frog_tongue").as_u64()?, &self.get_world()))
    }
    /// bukkit: org/bukkit/entity/Frog.setTongueTarget
    pub fn set_tongue_target(&self, target: Option<&Entity>) -> bool {
        put(&self.0, "frog_tongue", target.map_or(Value::Null, |entity| json!(entity.get_entity_id())))
    }
}

impl SkeletonHorse {
    /// bukkit: org/bukkit/entity/SkeletonHorse.isTrap
    pub fn is_trap(&self) -> bool {
        flag(&self.0, "trap")
    }
    /// bukkit: org/bukkit/entity/SkeletonHorse.setTrap
    pub fn set_trap(&self, trap: bool) -> bool {
        put(&self.0, "trap", json!(trap))
    }
}

impl Vex {
    /// bukkit: org/bukkit/entity/Vex.getOwner, org/bukkit/entity/Vex.getSummoner
    pub fn get_owner(&self) -> Option<Mob> {
        let id = get(&self.0, "vex_owner").as_u64().filter(|id| *id != 0)?;
        Some(Mob(Entity::of(id, &self.get_world())))
    }
    /// bukkit: org/bukkit/entity/Vex.setOwner, org/bukkit/entity/Vex.setSummoner
    pub fn set_owner(&self, owner: Option<&Entity>) -> bool {
        put(&self.0, "vex_owner", json!(owner.map_or(0, Entity::get_entity_id)))
    }
    /// bukkit: org/bukkit/entity/Vex.hasLimitedLifetime, org/bukkit/entity/Vex.hasLimitedLife
    pub fn has_limited_lifetime(&self) -> bool {
        number(&self.0, "vex_life") > 0
    }
    /// bukkit: org/bukkit/entity/Vex.setLimitedLifetime
    pub fn set_limited_lifetime(&self, limited: bool) -> bool {
        put(&self.0, "vex_life", json!(if limited { 400 } else { -1 }))
    }
}

impl Llama {
    /// bukkit: org/bukkit/entity/Llama.inCaravan
    pub fn in_caravan(&self) -> bool {
        number(&self.0, "caravan_head") != 0
    }
    /// bukkit: org/bukkit/entity/Llama.getCaravanHead
    pub fn get_caravan_head(&self) -> Option<Llama> {
        let id = get(&self.0, "caravan_head").as_u64().filter(|id| *id != 0)?;
        Some(Llama(Entity::of(id - 1, &self.get_world())))
    }
    /// bukkit: org/bukkit/entity/Llama.joinCaravan
    pub fn join_caravan(&self, leader: &Llama) -> bool {
        put(&self.0, "caravan_head", json!(leader.get_entity_id() + 1))
    }
    /// bukkit: org/bukkit/entity/Llama.leaveCaravan
    pub fn leave_caravan(&self) -> bool {
        put(&self.0, "caravan_head", json!(0))
    }
    /// bukkit: org/bukkit/entity/Llama.getCaravanTail
    pub fn get_caravan_tail(&self) -> Option<Llama> {
        Some(Llama(Entity::of(act(&self.0, "caravan_tail", json!({})).as_u64()?, &self.get_world())))
    }
    /// bukkit: org/bukkit/entity/Llama.hasCaravanTail
    pub fn has_caravan_tail(&self) -> bool {
        self.get_caravan_tail().is_some()
    }
}

impl Wolf {
    /// bukkit: org/bukkit/entity/Wolf.getTailAngle
    pub fn get_tail_angle(&self) -> f32 {
        if self.is_angry() {
            return 1.539_380_4;
        }
        if !self.data_at("Owner").is_null() {
            let lost = self.get_max_health() - self.get_health();
            return (0.55 - lost as f32 * 0.02) * std::f32::consts::PI;
        }
        std::f32::consts::PI / 5.0
    }
}

impl Ocelot {
    /// bukkit: org/bukkit/entity/Ocelot.getCatType
    pub fn get_cat_type(&self) -> &'static str {
        "WILD_OCELOT"
    }
}

/// bukkit: org/bukkit/entity/Ocelot$Type.getId
pub fn ocelot_type_id(kind: &str) -> Option<i32> {
    crate::enums::ocelot_type::values().iter().position(|name| *name == kind).map(|index| index as i32)
}

impl Raider {
    /// bukkit: org/bukkit/entity/Raider.getCelebrationSound
    pub fn get_celebration_sound(&self) -> String {
        format!("minecraft:entity.{}.celebrate", self.get_type().trim_start_matches("minecraft:"))
    }
}

impl Witch {
    /// bukkit: org/bukkit/entity/Witch.getDrinkingPotion
    pub fn get_drinking_potion(&self) -> Option<ItemStack> {
        let kind = get(&self.0, "witch_potion");
        if !kind["stack"].is_null() {
            return ItemStack::read(&kind["stack"]);
        }
        let mut stack = ItemStack::new(crate::Material::of("minecraft:potion"), 1);
        stack.set_component("minecraft:potion_contents", json!({ "potion": kind.as_str()? }));
        Some(stack)
    }
    /// bukkit: org/bukkit/entity/Witch.setDrinkingPotion
    pub fn set_drinking_potion(&self, potion: Option<&ItemStack>) -> bool {
        let contents = potion.and_then(|stack| stack.get_component("minecraft:potion_contents"));
        let kind = potion.map(|_| contents.and_then(|contents| contents["potion"].as_str()).unwrap_or("minecraft:water").to_owned());
        done(&self.0, "witch_drink", json!({ "value": kind, "stack": potion.map(ItemStack::to_snbt) }))
    }
}

impl Warden {
    /// bukkit: org/bukkit/entity/Warden.setDisturbanceLocation
    pub fn set_disturbance_location(&self, at: &Location) -> bool {
        done(&self.0, "warden_disturbance", json!({ "at": block_of(at) }))
    }
}

/// `org.bukkit.entity.LightningStrike`, which the game tracks but never saves.
#[derive(Clone, Debug, PartialEq)]
pub struct LightningStrike(pub Entity);

impl std::ops::Deref for LightningStrike {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for LightningStrike {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl LightningStrike {
    /// bukkit: org/bukkit/entity/LightningStrike.getCausingPlayer, org/bukkit/entity/LightningStrike.getCausingEntity
    pub fn get_causing_player(&self) -> Option<Player> {
        let uuid = object(&self.0, "bolt", json!({}))["cause"].as_str()?.to_owned();
        crate::get_online_players().into_iter().find(|player| player.uuid() == uuid)
    }
    /// bukkit: org/bukkit/entity/LightningStrike.setCausingPlayer
    pub fn set_causing_player(&self, player: Option<&Player>) -> bool {
        object_done(&self.0, "bolt_cause", json!({ "uuid": player.map(Player::uuid) }))
    }
}

impl Firework {
    fn state(&self) -> Value {
        object(&self.0, "rocket", json!({}))
    }
    /// bukkit: org/bukkit/entity/Firework.isDetonated
    pub fn is_detonated(&self) -> bool {
        self.state().is_null()
    }
    /// bukkit: org/bukkit/entity/Firework.getAttachedTo, org/bukkit/entity/Firework.getBoostedEntity
    pub fn get_attached_to(&self) -> Option<Actor> {
        actor_id(&self.0, self.state()["attached_to"].as_u64()?)
    }
    /// bukkit: org/bukkit/entity/Firework.setAttachedTo
    pub fn set_attached_to(&self, entity: Option<&Actor>) -> bool {
        object_done(&self.0, "rocket_attach", entity.map_or(json!({ "entity": null }), who))
    }
    /// bukkit: org/bukkit/entity/Firework.getSpawningEntity
    pub fn get_spawning_entity(&self) -> Option<String> {
        let state = self.state();
        if let Some(owner) = state["owner"].as_str() {
            return Some(crate::dashed(owner));
        }
        match actor_id(&self.0, state["owner_entity"].as_u64()?)? {
            Actor::Player(player) => Some(player.get_unique_id()),
            Actor::Entity(entity) => Some(entity.get_unique_id()),
        }
    }
}

impl FallingBlock {
    /// bukkit: org/bukkit/entity/FallingBlock.getMaterial
    pub fn get_material(&self) -> Option<crate::Material> {
        Some(crate::Material::of(object(&self.0, "falling", json!({}))["block"].as_str()?))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.getSourceLoc
    pub fn get_source_loc(&self) -> Option<Location> {
        place(&self.0, &object(&self.0, "falling", json!({}))["start"])
    }
}

/// `io.papermc.paper.entity.PlayerGiveResult`: what `Player::give` couldn't fit, and the items it dropped for it.
#[derive(Clone, Debug, Default)]
pub struct PlayerGiveResult {
    leftovers: Vec<ItemStack>,
    drops: Vec<Entity>,
}

impl PlayerGiveResult {
    /// bukkit: io/papermc/paper/entity/PlayerGiveResult.leftovers
    pub fn leftovers(&self) -> &[ItemStack] {
        &self.leftovers
    }
    /// bukkit: io/papermc/paper/entity/PlayerGiveResult.drops
    pub fn drops(&self) -> &[Entity] {
        &self.drops
    }
}

/// `com.destroystokyo.paper.entity.TargetEntityInfo`.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetEntityInfo {
    entity: Entity,
    hit: Vector,
}

impl TargetEntityInfo {
    /// bukkit: com/destroystokyo/paper/entity/TargetEntityInfo.<init>
    pub fn new(entity: Entity, hit: Vector) -> Self {
        Self { entity, hit }
    }
    /// bukkit: com/destroystokyo/paper/entity/TargetEntityInfo.getEntity
    pub fn get_entity(&self) -> &Entity {
        &self.entity
    }
    /// bukkit: com/destroystokyo/paper/entity/TargetEntityInfo.getHitVector
    pub fn get_hit_vector(&self) -> &Vector {
        &self.hit
    }
}

// ---- Calls Paper refuses ----

impl AbstractHorse {
    /// bukkit: org/bukkit/entity/AbstractHorse.setVariant
    ///
    /// False where Java throws: spawn the other entity.
    pub fn set_variant(&self, _variant: &str) -> bool {
        false
    }
}

impl AbstractSkeleton {
    /// bukkit: org/bukkit/entity/AbstractSkeleton.setSkeletonType
    ///
    /// False where Java throws: spawn the other entity.
    pub fn set_skeleton_type(&self, _kind: &str) -> bool {
        false
    }
}

impl Guardian {
    /// bukkit: org/bukkit/entity/Guardian.setElder
    ///
    /// False where Java throws: spawn the other entity.
    pub fn set_elder(&self, _elder: bool) -> bool {
        false
    }
}

impl Zombie {
    /// bukkit: org/bukkit/entity/Zombie.setVillager
    ///
    /// False where Java throws: spawn the other entity.
    pub fn set_villager(&self, _villager: bool) -> bool {
        false
    }
}

impl Ocelot {
    /// bukkit: org/bukkit/entity/Ocelot.setCatType
    ///
    /// False where Java throws: cats are their own entity.
    pub fn set_cat_type(&self, _kind: &str) -> bool {
        false
    }
}

impl Boat {
    /// bukkit: org/bukkit/entity/Boat.setBoatType, org/bukkit/entity/Boat.setWoodType
    ///
    /// False where Java throws: each wood is its own entity.
    pub fn set_boat_type(&self, _kind: &str) -> bool {
        false
    }
}

impl Horse {
    /// bukkit: org/bukkit/entity/Horse.isCarryingChest
    pub fn is_carrying_chest(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/entity/Horse.setCarryingChest
    ///
    /// False where Java throws: a horse has no chest, a donkey does.
    pub fn set_carrying_chest(&self, _chest: bool) -> bool {
        false
    }
}

impl PigZombie {
    /// bukkit: org/bukkit/entity/PigZombie.getConversionTime
    ///
    /// `None` where Java throws: a zombified piglin does not convert.
    pub fn get_conversion_time(&self) -> Option<i32> {
        None
    }
    /// bukkit: org/bukkit/entity/PigZombie.setConversionTime
    ///
    /// False where Java throws: a zombified piglin does not convert.
    pub fn set_conversion_time(&self, _time: i32) -> bool {
        false
    }
}

// ---- Armor stands ----

const POSES: [(&str, &str); 6] = [("head", "Head"), ("body", "Body"), ("left_arm", "LeftArm"), ("right_arm", "RightArm"), ("left_leg", "LeftLeg"), ("right_leg", "RightLeg")];

/// `EquipmentSlot`'s filter bit, in the order the game keeps them.
fn slot_bit(slot: &str) -> Option<u32> {
    Some(match slot {
        "HAND" => 0,
        "FEET" => 1,
        "LEGS" => 2,
        "CHEST" => 3,
        "HEAD" => 4,
        "OFF_HAND" => 5,
        _ => return None,
    })
}

impl ArmorStand {
    fn rotations(&self, part: &str) -> crate::pure::Rotations {
        let saved = POSES.iter().find(|(name, _)| *name == part).map_or(Value::Null, |(_, key)| self.data_at(&format!("Pose.{key}")));
        let at = |i: usize| saved[i].as_f64().unwrap_or(0.0);
        crate::pure::Rotations::of_degrees(at(0), at(1), at(2))
    }
    fn set_rotations(&self, part: &str, rotations: &crate::pure::Rotations) -> bool {
        let Some((_, key)) = POSES.iter().find(|(name, _)| *name == part) else {
            return false;
        };
        self.merge(&format!("{{Pose:{{{key}:[{:?}f,{:?}f,{:?}f]}}}}", rotations.x() as f32, rotations.y() as f32, rotations.z() as f32))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getHeadRotations
    pub fn get_head_rotations(&self) -> crate::pure::Rotations {
        self.rotations("head")
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setHeadRotations
    pub fn set_head_rotations(&self, rotations: &crate::pure::Rotations) -> bool {
        self.set_rotations("head", rotations)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getBodyRotations
    pub fn get_body_rotations(&self) -> crate::pure::Rotations {
        self.rotations("body")
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setBodyRotations
    pub fn set_body_rotations(&self, rotations: &crate::pure::Rotations) -> bool {
        self.set_rotations("body", rotations)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getLeftArmRotations
    pub fn get_left_arm_rotations(&self) -> crate::pure::Rotations {
        self.rotations("left_arm")
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setLeftArmRotations
    pub fn set_left_arm_rotations(&self, rotations: &crate::pure::Rotations) -> bool {
        self.set_rotations("left_arm", rotations)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getRightArmRotations
    pub fn get_right_arm_rotations(&self) -> crate::pure::Rotations {
        self.rotations("right_arm")
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setRightArmRotations
    pub fn set_right_arm_rotations(&self, rotations: &crate::pure::Rotations) -> bool {
        self.set_rotations("right_arm", rotations)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getLeftLegRotations
    pub fn get_left_leg_rotations(&self) -> crate::pure::Rotations {
        self.rotations("left_leg")
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setLeftLegRotations
    pub fn set_left_leg_rotations(&self, rotations: &crate::pure::Rotations) -> bool {
        self.set_rotations("left_leg", rotations)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getRightLegRotations
    pub fn get_right_leg_rotations(&self) -> crate::pure::Rotations {
        self.rotations("right_leg")
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setRightLegRotations
    pub fn set_right_leg_rotations(&self, rotations: &crate::pure::Rotations) -> bool {
        self.set_rotations("right_leg", rotations)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getItem
    pub fn get_item(&self, slot: &str) -> Option<ItemStack> {
        self.get_equipment().get_item(slot)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setItem
    pub fn set_item(&self, slot: &str, stack: &ItemStack) -> bool {
        self.get_equipment().set_item(slot, stack)
    }
    fn disabled(&self) -> i64 {
        self.data_at("DisabledSlots").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.getDisabledSlots
    pub fn get_disabled_slots(&self) -> Vec<&'static str> {
        ["HAND", "FEET", "LEGS", "CHEST", "HEAD", "OFF_HAND"].into_iter().filter(|slot| slot_bit(slot).is_some_and(|bit| self.disabled() >> bit & 1 != 0)).collect()
    }
    fn lock_bit(slot: &str, lock: &str) -> Option<u32> {
        let kind = crate::enums::armor_stand_lock_type::values().iter().position(|name| *name == lock)? as u32;
        Some(slot_bit(slot)? + kind * 8)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.hasEquipmentLock
    pub fn has_equipment_lock(&self, slot: &str, lock: &str) -> bool {
        Self::lock_bit(slot, lock).is_some_and(|bit| self.disabled() >> bit & 1 != 0)
    }
    fn set_disabled(&self, value: i64) -> bool {
        self.merge(&format!("{{DisabledSlots:{value}}}"))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.addEquipmentLock
    pub fn add_equipment_lock(&self, slot: &str, lock: &str) -> bool {
        Self::lock_bit(slot, lock).is_some_and(|bit| self.set_disabled(self.disabled() | 1 << bit))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.removeEquipmentLock
    pub fn remove_equipment_lock(&self, slot: &str, lock: &str) -> bool {
        Self::lock_bit(slot, lock).is_some_and(|bit| self.set_disabled(self.disabled() & !(1 << bit)))
    }
    fn all_locks(slots: &[&str]) -> i64 {
        slots.iter().filter_map(|slot| slot_bit(slot)).fold(0, |bits, bit| bits | 0x010101 << bit)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setDisabledSlots
    pub fn set_disabled_slots(&self, slots: &[&str]) -> bool {
        self.set_disabled(Self::all_locks(slots))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.addDisabledSlots
    pub fn add_disabled_slots(&self, slots: &[&str]) -> bool {
        self.set_disabled(self.disabled() | Self::all_locks(slots))
    }
    /// bukkit: org/bukkit/entity/ArmorStand.removeDisabledSlots
    pub fn remove_disabled_slots(&self, slots: &[&str]) -> bool {
        self.set_disabled(self.disabled() & !Self::all_locks(slots))
    }
}

// ---- Goals ----

/// `com.destroystokyo.paper.entity.ai.GoalKey`: a goal's name, and the entity it's for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalKey {
    class: String,
    key: crate::persistence::NamespacedKey,
}

impl GoalKey {
    /// bukkit: com/destroystokyo/paper/entity/ai/GoalKey.of
    pub fn of(class: &str, key: crate::persistence::NamespacedKey) -> Self {
        Self { class: class.to_owned(), key }
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/GoalKey.getNamespacedKey
    pub fn get_namespaced_key(&self) -> &crate::persistence::NamespacedKey {
        &self.key
    }
    /// bukkit~: com/destroystokyo/paper/entity/ai/GoalKey.getEntityClass | the class's name
    pub fn get_entity_class(&self) -> &str {
        &self.class
    }
}

/// `com.destroystokyo.paper.entity.ai.Goal`, written by a plugin.
pub trait Goal: Send {
    /// bukkit: com/destroystokyo/paper/entity/ai/Goal.shouldActivate
    fn should_activate(&mut self) -> bool;
    /// bukkit: com/destroystokyo/paper/entity/ai/Goal.shouldStayActive
    fn should_stay_active(&mut self) -> bool {
        self.should_activate()
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/Goal.start
    fn start(&mut self) {}
    /// bukkit: com/destroystokyo/paper/entity/ai/Goal.stop
    fn stop(&mut self) {}
    /// bukkit: com/destroystokyo/paper/entity/ai/Goal.tick
    fn tick(&mut self) {}
    /// bukkit: com/destroystokyo/paper/entity/ai/Goal.getKey
    fn get_key(&self) -> GoalKey;
    /// bukkit: com/destroystokyo/paper/entity/ai/Goal.getTypes
    fn get_types(&self) -> Vec<&'static str>;
}

type SharedGoal = std::sync::Arc<std::sync::Mutex<Box<dyn Goal>>>;

struct Kept {
    /// What the game's selector calls it: one key can be on several goals.
    name: String,
    key: GoalKey,
    goal: SharedGoal,
    running: bool,
}

static GOALS: std::sync::Mutex<std::collections::BTreeMap<u64, (Entity, Vec<Kept>)>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
static GOALS_RUN: std::sync::OnceLock<()> = std::sync::OnceLock::new();
static GOAL_NAMES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Goals taken off while one of theirs was running: they stop once it's finished.
static GOALS_STOPPING: std::sync::Mutex<Vec<SharedGoal>> = std::sync::Mutex::new(Vec::new());

/// Stops a goal that's off its mob: it may be the very one that asked, whose lock is held further up.
fn stop_goal(goal: &SharedGoal) {
    match goal.try_lock() {
        Ok(mut goal) => goal.stop(),
        Err(_) => GOALS_STOPPING.lock().unwrap_or_else(|e| e.into_inner()).push(goal.clone()),
    }
}

/// The game's selector has decided who runs; now each goal's code. Nothing of a plugin's runs under `GOALS`: a goal may
/// add or remove goals and ask the game anything.
fn tick_goals() {
    let rows: Vec<(Entity, Vec<(String, SharedGoal, bool)>)> = GOALS.lock().unwrap_or_else(|e| e.into_inner()).values().map(|(mob, goals)| (mob.clone(), goals.iter().map(|kept| (kept.name.clone(), kept.goal.clone(), kept.running)).collect())).collect();
    for (mob, goals) in rows {
        if !mob.is_valid() {
            GOALS.lock().unwrap_or_else(|e| e.into_inner()).remove(&mob.get_entity_id());
            continue;
        }
        let listed = act(&mob, "goals_all", json!({}));
        let running = |name: &str| listed.as_array().into_iter().flatten().any(|row| row["name"] == name && row["running"] == true);
        for (name, goal, was) in goals {
            let now = running(&name);
            let wants = {
                let mut goal = goal.lock().unwrap_or_else(|e| e.into_inner());
                match (was, now) {
                    (false, true) => goal.start(),
                    (true, false) => goal.stop(),
                    (true, true) => goal.tick(),
                    (false, false) => {}
                }
                if now { goal.should_stay_active() } else { goal.should_activate() }
            };
            done(&mob, "goal_wants", json!({ "goal": name, "value": wants }));
            if was != now
                && let Some((_, kept)) = GOALS.lock().unwrap_or_else(|e| e.into_inner()).get_mut(&mob.get_entity_id())
                && let Some(kept) = kept.iter_mut().find(|kept| kept.name == name)
            {
                kept.running = now;
            }
        }
    }
    let stopping = std::mem::take(&mut *GOALS_STOPPING.lock().unwrap_or_else(|e| e.into_inner()));
    for goal in stopping {
        goal.lock().unwrap_or_else(|e| e.into_inner()).stop();
    }
}

fn goal_flags(types: &[&str]) -> u8 {
    types.iter().fold(0, |flags, kind| flags | match *kind {
        "MOVE" => 1,
        "LOOK" => 2,
        "JUMP" => 4,
        "TARGET" => 8,
        _ => 0,
    })
}

fn goal_types(flags: u8) -> Vec<&'static str> {
    if flags == 0 {
        return vec!["UNKNOWN_BEHAVIOR"];
    }
    [(1, "MOVE"), (2, "LOOK"), (4, "JUMP"), (8, "TARGET")].into_iter().filter(|(bit, _)| flags & bit != 0).map(|(_, kind)| kind).collect()
}

/// `RandomStrollGoal` as a key: `random_stroll`.
fn goal_word(name: &str) -> String {
    let name = name.strip_suffix("Goal").unwrap_or(name);
    let mut word = String::new();
    for (i, letter) in name.chars().enumerate() {
        if letter.is_uppercase() && i > 0 {
            word.push('_');
        }
        word.extend(letter.to_lowercase());
    }
    word
}

/// One goal a mob has, the game's or a plugin's, as `MobGoals` hands them out.
#[derive(Clone)]
pub struct MobGoal {
    mob: Entity,
    name: String,
    key: GoalKey,
    types: Vec<&'static str>,
    plugin: Option<SharedGoal>,
    running: bool,
}

impl MobGoal {
    fn call(&self, call: &str) -> bool {
        done(&self.mob, "goal_call", json!({ "goal": self.name, "call": call }))
    }
    /// Whether the selector has it running.
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Goal for MobGoal {
    fn should_activate(&mut self) -> bool {
        match &self.plugin {
            Some(goal) => goal.try_lock().is_ok_and(|mut goal| goal.should_activate()),
            None => self.call("use"),
        }
    }
    fn should_stay_active(&mut self) -> bool {
        match &self.plugin {
            Some(goal) => goal.try_lock().is_ok_and(|mut goal| goal.should_stay_active()),
            None => self.call("continue"),
        }
    }
    fn start(&mut self) {
        match &self.plugin {
            Some(goal) => {
                if let Ok(mut goal) = goal.try_lock() {
                    goal.start();
                }
            }
            None => {
                self.call("start");
            }
        }
    }
    fn stop(&mut self) {
        match &self.plugin {
            Some(goal) => {
                if let Ok(mut goal) = goal.try_lock() {
                    goal.stop();
                }
            }
            None => {
                self.call("stop");
            }
        }
    }
    fn tick(&mut self) {
        match &self.plugin {
            Some(goal) => {
                if let Ok(mut goal) = goal.try_lock() {
                    goal.tick();
                }
            }
            None => {
                self.call("tick");
            }
        }
    }
    fn get_key(&self) -> GoalKey {
        self.key.clone()
    }
    fn get_types(&self) -> Vec<&'static str> {
        self.types.clone()
    }
}

/// `com.destroystokyo.paper.entity.ai.MobGoals`.
///
/// A plugin's goal sits in the game's selector, with the game's own by priority and flags. Its code runs between mob
/// ticks, so what it says takes effect in the mob's next tick, and `start` and `stop` come one tick after the selector
/// chose. A goal with `TARGET` goes in the target selector, as on Paper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MobGoals;

/// bukkit: org/bukkit/Bukkit.getMobGoals, org/bukkit/Server.getMobGoals
pub fn get_mob_goals() -> MobGoals {
    MobGoals
}

impl MobGoals {
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.addGoal
    pub fn add_goal(&self, mob: &Mob, priority: i32, goal: Box<dyn Goal>) -> bool {
        GOALS_RUN.get_or_init(|| {
            crate::scheduler::run_task_timer(1, 1, tick_goals);
        });
        let key = goal.get_key();
        let flags = goal_flags(&goal.get_types());
        let name = format!("{}#{}", key.key.as_str(), GOAL_NAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        if !done(&mob.0, "goal_add", json!({ "goal": name, "flags": flags, "priority": priority })) {
            return false;
        }
        let kept = Kept { name, key, goal: std::sync::Arc::new(std::sync::Mutex::new(goal)), running: false };
        GOALS.lock().unwrap_or_else(|e| e.into_inner()).entry(mob.get_entity_id()).or_insert_with(|| (mob.0.clone(), Vec::new())).1.push(kept);
        true
    }
    /// The goals of a mob, with what the selector says of each.
    fn all(&self, mob: &Mob) -> Vec<MobGoal> {
        let listed = act(&mob.0, "goals_all", json!({}));
        let kept: Vec<(String, GoalKey, SharedGoal)> = GOALS.lock().unwrap_or_else(|e| e.into_inner()).get(&mob.get_entity_id()).map_or_else(Vec::new, |(_, goals)| goals.iter().map(|kept| (kept.name.clone(), kept.key.clone(), kept.goal.clone())).collect());
        let row = |row: &Value| {
            let name = row["name"].as_str()?.to_owned();
            let types = goal_types(row["flags"].as_u64()? as u8);
            let running = row["running"].as_bool()?;
            let own = kept.iter().find(|(known, ..)| *known == name);
            let key = own.map_or_else(|| GoalKey::of("Mob", crate::persistence::NamespacedKey::minecraft(&goal_word(&name))), |(_, key, _)| key.clone());
            Some(MobGoal { mob: mob.0.clone(), name, key, types, plugin: own.map(|(.., goal)| goal.clone()), running })
        };
        listed.as_array().into_iter().flatten().filter_map(row).collect()
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.removeGoal
    pub fn remove_goal(&self, mob: &Mob, key: &GoalKey) {
        for goal in self.get_goals(mob, key) {
            if let Some(own) = &goal.plugin
                && goal.running
            {
                stop_goal(own);
            }
            done(&mob.0, "goal_remove", json!({ "goal": goal.name }));
        }
        if let Some((_, goals)) = GOALS.lock().unwrap_or_else(|e| e.into_inner()).get_mut(&mob.get_entity_id()) {
            goals.retain(|kept| kept.key != *key);
        }
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.removeAllGoals
    pub fn remove_all_goals(&self, mob: &Mob) -> bool {
        let gone = GOALS.lock().unwrap_or_else(|e| e.into_inner()).remove(&mob.get_entity_id());
        for kept in gone.map_or_else(Vec::new, |(_, goals)| goals) {
            if kept.running {
                stop_goal(&kept.goal);
            }
        }
        done(&mob.0, "goals_clear", json!({}))
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.removeAllGoals
    pub fn remove_all_goals_of_type(&self, mob: &Mob, kind: &str) -> bool {
        let matches = |goal: &MobGoal| goal.types.contains(&kind);
        for goal in self.all(mob).iter().filter(|goal| matches(goal)) {
            if let Some(own) = &goal.plugin
                && goal.running
            {
                stop_goal(own);
            }
            done(&mob.0, "goal_remove", json!({ "goal": goal.name }));
            if let Some((_, goals)) = GOALS.lock().unwrap_or_else(|e| e.into_inner()).get_mut(&mob.get_entity_id()) {
                goals.retain(|kept| kept.name != goal.name);
            }
        }
        true
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.hasGoal
    pub fn has_goal(&self, mob: &Mob, key: &GoalKey) -> bool {
        !self.get_goals(mob, key).is_empty()
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.getGoal
    pub fn get_goal(&self, mob: &Mob, key: &GoalKey) -> Option<MobGoal> {
        self.get_goals(mob, key).into_iter().next()
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.getGoals
    pub fn get_goals(&self, mob: &Mob, key: &GoalKey) -> Vec<MobGoal> {
        self.all(mob).into_iter().filter(|goal| goal.key.key == key.key).collect()
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.getAllGoals
    pub fn get_all_goals(&self, mob: &Mob, kind: Option<&str>) -> Vec<MobGoal> {
        self.all(mob).into_iter().filter(|goal| kind.is_none_or(|kind| goal.types.contains(&kind))).collect()
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.getAllGoalsWithout
    pub fn get_all_goals_without(&self, mob: &Mob, kind: &str) -> Vec<MobGoal> {
        self.all(mob).into_iter().filter(|goal| !goal.types.contains(&kind)).collect()
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.getRunningGoals
    pub fn get_running_goals(&self, mob: &Mob, kind: Option<&str>) -> Vec<MobGoal> {
        self.get_all_goals(mob, kind).into_iter().filter(|goal| goal.running).collect()
    }
    /// bukkit: com/destroystokyo/paper/entity/ai/MobGoals.getRunningGoalsWithout
    pub fn get_running_goals_without(&self, mob: &Mob, kind: &str) -> Vec<MobGoal> {
        self.get_all_goals_without(mob, kind).into_iter().filter(|goal| goal.running).collect()
    }
}

// ---- Entity types ----

/// Each constant of `EntityType`: its name, class, old numeric id and whether a plugin can spawn it.
const ENTITY_TYPES: &[(&str, &str, i32, bool)] = &[
    ("ACACIA_BOAT", "AcaciaBoat", -1, true),
    ("ACACIA_CHEST_BOAT", "AcaciaChestBoat", -1, true),
    ("ALLAY", "Allay", -1, true),
    ("AREA_EFFECT_CLOUD", "AreaEffectCloud", 3, true),
    ("ARMADILLO", "Armadillo", -1, true),
    ("ARMOR_STAND", "ArmorStand", 30, true),
    ("ARROW", "Arrow", 10, true),
    ("AXOLOTL", "Axolotl", -1, true),
    ("BAMBOO_CHEST_RAFT", "BambooChestRaft", -1, true),
    ("BAMBOO_RAFT", "BambooRaft", -1, true),
    ("BAT", "Bat", 65, true),
    ("BEE", "Bee", -1, true),
    ("BIRCH_BOAT", "BirchBoat", -1, true),
    ("BIRCH_CHEST_BOAT", "BirchChestBoat", -1, true),
    ("BLAZE", "Blaze", 61, true),
    ("BLOCK_DISPLAY", "BlockDisplay", -1, true),
    ("BOGGED", "Bogged", -1, true),
    ("BREEZE", "Breeze", -1, true),
    ("BREEZE_WIND_CHARGE", "BreezeWindCharge", -1, true),
    ("CAMEL", "Camel", -1, true),
    ("CAMEL_HUSK", "CamelHusk", -1, true),
    ("CAT", "Cat", -1, true),
    ("CAVE_SPIDER", "CaveSpider", 59, true),
    ("CHERRY_BOAT", "CherryBoat", -1, true),
    ("CHERRY_CHEST_BOAT", "CherryChestBoat", -1, true),
    ("CHEST_MINECART", "StorageMinecart", 43, true),
    ("CHICKEN", "Chicken", 93, true),
    ("COD", "Cod", -1, true),
    ("COMMAND_BLOCK_MINECART", "CommandMinecart", 40, true),
    ("COPPER_GOLEM", "CopperGolem", -1, true),
    ("COW", "Cow", 92, true),
    ("CREAKING", "Creaking", -1, true),
    ("CREEPER", "Creeper", 50, true),
    ("CUSHION", "Cushion", -1, true),
    ("DARK_OAK_BOAT", "DarkOakBoat", -1, true),
    ("DARK_OAK_CHEST_BOAT", "DarkOakChestBoat", -1, true),
    ("DOLPHIN", "Dolphin", -1, true),
    ("DONKEY", "Donkey", 31, true),
    ("DRAGON_FIREBALL", "DragonFireball", 26, true),
    ("DROWNED", "Drowned", -1, true),
    ("EGG", "Egg", 7, true),
    ("ELDER_GUARDIAN", "ElderGuardian", 4, true),
    ("END_CRYSTAL", "EnderCrystal", 200, true),
    ("ENDER_DRAGON", "EnderDragon", 63, true),
    ("ENDER_PEARL", "EnderPearl", 14, true),
    ("ENDERMAN", "Enderman", 58, true),
    ("ENDERMITE", "Endermite", 67, true),
    ("EVOKER", "Evoker", 34, true),
    ("EVOKER_FANGS", "EvokerFangs", 33, true),
    ("EXPERIENCE_BOTTLE", "ThrownExpBottle", 17, true),
    ("EXPERIENCE_ORB", "ExperienceOrb", 2, true),
    ("EYE_OF_ENDER", "EnderSignal", 15, true),
    ("FALLING_BLOCK", "FallingBlock", 21, true),
    ("FIREBALL", "LargeFireball", 12, true),
    ("FIREWORK_ROCKET", "Firework", 22, true),
    ("FISHING_BOBBER", "FishHook", -1, false),
    ("FOX", "Fox", -1, true),
    ("FROG", "Frog", -1, true),
    ("FURNACE_MINECART", "PoweredMinecart", 44, true),
    ("GHAST", "Ghast", 56, true),
    ("GIANT", "Giant", 53, true),
    ("GLOW_ITEM_FRAME", "GlowItemFrame", -1, true),
    ("GLOW_SQUID", "GlowSquid", -1, true),
    ("GOAT", "Goat", -1, true),
    ("GUARDIAN", "Guardian", 68, true),
    ("HAPPY_GHAST", "HappyGhast", -1, true),
    ("HOGLIN", "Hoglin", -1, true),
    ("HOPPER_MINECART", "HopperMinecart", 46, true),
    ("HORSE", "Horse", 100, true),
    ("HUSK", "Husk", 23, true),
    ("ILLUSIONER", "Illusioner", 37, true),
    ("INTERACTION", "Interaction", -1, true),
    ("IRON_GOLEM", "IronGolem", 99, true),
    ("ITEM", "Item", 1, true),
    ("ITEM_DISPLAY", "ItemDisplay", -1, true),
    ("ITEM_FRAME", "ItemFrame", 18, true),
    ("JUNGLE_BOAT", "JungleBoat", -1, true),
    ("JUNGLE_CHEST_BOAT", "JungleChestBoat", -1, true),
    ("LEASH_KNOT", "LeashHitch", 8, true),
    ("LIGHTNING_BOLT", "LightningStrike", -1, true),
    ("LINGERING_POTION", "LingeringPotion", -1, true),
    ("LLAMA", "Llama", 103, true),
    ("LLAMA_SPIT", "LlamaSpit", 104, true),
    ("MAGMA_CUBE", "MagmaCube", 62, true),
    ("MANGROVE_BOAT", "MangroveBoat", -1, true),
    ("MANGROVE_CHEST_BOAT", "MangroveChestBoat", -1, true),
    ("MANNEQUIN", "Mannequin", -1, true),
    ("MARKER", "Marker", -1, true),
    ("MINECART", "RideableMinecart", 42, true),
    ("MOOSHROOM", "MushroomCow", 96, true),
    ("MULE", "Mule", 32, true),
    ("NAUTILUS", "Nautilus", -1, true),
    ("OAK_BOAT", "OakBoat", -1, true),
    ("OAK_CHEST_BOAT", "OakChestBoat", -1, true),
    ("OCELOT", "Ocelot", 98, true),
    ("OMINOUS_ITEM_SPAWNER", "OminousItemSpawner", -1, true),
    ("PAINTING", "Painting", 9, true),
    ("PALE_OAK_BOAT", "PaleOakBoat", -1, true),
    ("PALE_OAK_CHEST_BOAT", "PaleOakChestBoat", -1, true),
    ("PANDA", "Panda", -1, true),
    ("PARCHED", "Parched", -1, true),
    ("PARROT", "Parrot", 105, true),
    ("PHANTOM", "Phantom", -1, true),
    ("PIG", "Pig", 90, true),
    ("PIGLIN", "Piglin", -1, true),
    ("PIGLIN_BRUTE", "PiglinBrute", -1, true),
    ("PILLAGER", "Pillager", -1, true),
    ("PLAYER", "Player", -1, false),
    ("POLAR_BEAR", "PolarBear", 102, true),
    ("POPLAR_BOAT", "PoplarBoat", -1, true),
    ("POPLAR_CHEST_BOAT", "PoplarChestBoat", -1, true),
    ("PUFFERFISH", "PufferFish", -1, true),
    ("RABBIT", "Rabbit", 101, true),
    ("RAVAGER", "Ravager", -1, true),
    ("SALMON", "Salmon", -1, true),
    ("SHEEP", "Sheep", 91, true),
    ("SHULKER", "Shulker", 69, true),
    ("SHULKER_BULLET", "ShulkerBullet", 25, true),
    ("SILVERFISH", "Silverfish", 60, true),
    ("SKELETON", "Skeleton", 51, true),
    ("SKELETON_HORSE", "SkeletonHorse", 28, true),
    ("SLIME", "Slime", 55, true),
    ("SMALL_FIREBALL", "SmallFireball", 13, true),
    ("SNIFFER", "Sniffer", -1, true),
    ("SNOW_GOLEM", "Snowman", 97, true),
    ("SNOWBALL", "Snowball", 11, true),
    ("SPAWNER_MINECART", "SpawnerMinecart", 47, true),
    ("SPECTRAL_ARROW", "SpectralArrow", 24, true),
    ("SPIDER", "Spider", 52, true),
    ("SPLASH_POTION", "SplashPotion", 16, true),
    ("SPRUCE_BOAT", "SpruceBoat", -1, true),
    ("SPRUCE_CHEST_BOAT", "SpruceChestBoat", -1, true),
    ("SQUID", "Squid", 94, true),
    ("STRAY", "Stray", 6, true),
    ("STRIDER", "Strider", -1, true),
    ("SULFUR_CUBE", "SulfurCube", -1, true),
    ("TADPOLE", "Tadpole", -1, true),
    ("TEXT_DISPLAY", "TextDisplay", -1, true),
    ("TNT", "TNTPrimed", 20, true),
    ("TNT_MINECART", "ExplosiveMinecart", 45, true),
    ("TRADER_LLAMA", "TraderLlama", -1, true),
    ("TRIDENT", "Trident", -1, true),
    ("TROPICAL_FISH", "TropicalFish", -1, true),
    ("TURTLE", "Turtle", -1, true),
    ("VEX", "Vex", 35, true),
    ("VILLAGER", "Villager", 120, true),
    ("VINDICATOR", "Vindicator", 36, true),
    ("WANDERING_TRADER", "WanderingTrader", -1, true),
    ("WARDEN", "Warden", -1, true),
    ("WIND_CHARGE", "WindCharge", -1, true),
    ("WITCH", "Witch", 66, true),
    ("WITHER", "Wither", 64, true),
    ("WITHER_SKELETON", "WitherSkeleton", 5, true),
    ("WITHER_SKULL", "WitherSkull", 19, true),
    ("WOLF", "Wolf", 95, true),
    ("ZOGLIN", "Zoglin", -1, true),
    ("ZOMBIE", "Zombie", 54, true),
    ("ZOMBIE_HORSE", "ZombieHorse", 29, true),
    ("ZOMBIE_NAUTILUS", "ZombieNautilus", -1, true),
    ("ZOMBIE_VILLAGER", "ZombieVillager", 27, true),
    ("ZOMBIFIED_PIGLIN", "PigZombie", 57, true),
];

fn type_row(kind: &str) -> Option<&'static (&'static str, &'static str, i32, bool)> {
    let name = kind.trim_start_matches("minecraft:").to_ascii_uppercase();
    ENTITY_TYPES.iter().find(|row| row.0 == name)
}

/// bukkit: org/bukkit/entity/EntityType.getTypeId
pub fn entity_type_get_type_id(kind: &str) -> i32 {
    type_row(kind).map_or(-1, |row| row.2)
}

/// bukkit: org/bukkit/entity/EntityType.fromId
pub fn entity_type_from_id(id: i32) -> Option<&'static str> {
    if id < 0 {
        return None;
    }
    ENTITY_TYPES.iter().find(|row| row.2 == id).map(|row| row.0)
}

/// bukkit: org/bukkit/entity/EntityType.isSpawnable
pub fn entity_type_is_spawnable(kind: &str) -> bool {
    type_row(kind).is_some_and(|row| row.3)
}

/// bukkit~: org/bukkit/entity/EntityType.getEntityClass | the class's name
pub fn entity_type_get_entity_class(kind: &str) -> Option<&'static str> {
    type_row(kind).map(|row| row.1)
}

/// bukkit: org/bukkit/entity/EntityType.hasDefaultAttributes
pub fn entity_type_has_default_attributes(kind: &str) -> bool {
    let id = format!("minecraft:{}", kind.trim_start_matches("minecraft:").to_ascii_lowercase());
    crate::entities3::entity_type_is_alive(&id)
}

// ---- A player's odds and ends ----

impl Player {
    /// bukkit: org/bukkit/entity/Player.give
    pub fn give(&self, stacks: &[ItemStack], drop_if_full: bool) -> PlayerGiveResult {
        let rows: Vec<String> = stacks.iter().map(ItemStack::to_snbt).collect();
        let answer = query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "give", "stacks": rows, "drop": drop_if_full }));
        let world = self.get_world();
        PlayerGiveResult {
            leftovers: answer["left"].as_array().into_iter().flatten().filter_map(ItemStack::read).collect(),
            drops: answer["drops"].as_array().into_iter().flatten().filter_map(Value::as_u64).map(|id| Entity::of(id, &world)).collect(),
        }
    }
    /// bukkit: org/bukkit/entity/Player.ban, org/bukkit/entity/Player.banPlayerFull
    ///
    /// `expires` in milliseconds since the epoch, `None` for good: for a `Duration`, add it to the time now.
    pub fn ban(&self, reason: &str, expires: Option<i64>, source: &str, kick: bool) -> Option<crate::core2::BanEntry> {
        let list = crate::extra::get_ban_list("NAME");
        list.add_ban_until(self.get_name(), reason, expires, source);
        if kick {
            self.kick_player(reason);
        }
        list.get_ban_entry(self.get_name())
    }
    /// bukkit: org/bukkit/entity/Player.banIp, org/bukkit/entity/Player.banPlayerIP
    ///
    /// As `ban`, for the address they're on.
    pub fn ban_ip(&self, reason: &str, expires: Option<i64>, source: &str, kick: bool) -> Option<crate::core2::BanEntry> {
        let address = self.get_address()?;
        let ip = address.parse::<std::net::SocketAddr>().map_or(address, |at| at.ip().to_string());
        let list = crate::extra::get_ban_list("IP");
        list.add_ban_until(&ip, reason, expires, source);
        if kick {
            self.kick_player(reason);
        }
        list.get_ban_entry(&ip)
    }
    /// bukkit: org/bukkit/entity/Player.asHoverEvent
    pub fn as_hover_event(&self) -> Value {
        json!({ "action": "show_entity", "id": "minecraft:player", "uuid": crate::dashed(self.uuid()), "name": self.get_name() })
    }
}

// ---- Second pass ----

impl Panda {
    /// bukkit: org/bukkit/entity/Panda.setIsSitting
    pub fn set_is_sitting(&self, sitting: bool) -> bool {
        put(&self.0, "panda_sitting", json!(sitting))
    }
}

impl SchoolableFish {
    /// bukkit: io/papermc/paper/entity/SchoolableFish.getMaxSchoolSize
    pub fn get_max_school_size(&self) -> i32 {
        number(&self.0, "fish_max")
    }
}

impl Bucketable {
    /// bukkit: io/papermc/paper/entity/Bucketable.isFromBucket
    pub fn is_from_bucket(&self) -> bool {
        crate::data::flag(&self.data_at("FromBucket"))
    }
    /// bukkit: io/papermc/paper/entity/Bucketable.setFromBucket
    pub fn set_from_bucket(&self, from_bucket: bool) -> bool {
        self.merge(&format!("{{FromBucket:{from_bucket},PersistenceRequired:{from_bucket}}}"))
    }
}

/// A wolf or a cat, `io.papermc.paper.entity.CollarColorable`.
pub trait CollarColorable: std::ops::Deref<Target = Entity> {
    /// bukkit: io/papermc/paper/entity/CollarColorable.getCollarColor
    fn collar_color(&self) -> Option<&'static str> {
        crate::enums::dye_color::values().get(self.data_at("CollarColor").as_u64()? as usize).copied()
    }
    /// bukkit: io/papermc/paper/entity/CollarColorable.setCollarColor
    fn set_collar(&self, color: &str) -> bool {
        crate::enums::dye_color::values().iter().position(|name| *name == color).is_some_and(|index| self.merge(&format!("{{CollarColor:{index}b}}")))
    }
}

impl CollarColorable for Wolf {}
impl CollarColorable for Cat {}

impl AbstractArrow {
    /// bukkit: org/bukkit/entity/AbstractArrow.setShooter
    pub fn set_shooter_and_pickup(&self, shooter: &Player, reset_pickup: bool) -> bool {
        let owner = crate::data::uuid_snbt(&crate::dashed(shooter.uuid())).unwrap_or_default();
        if reset_pickup { self.merge(&format!("{{Owner:{owner},pickup:1b}}")) } else { self.merge(&format!("{{Owner:{owner}}}")) }
    }
}

impl Animals {
    /// bukkit: org/bukkit/entity/Animals.isBreedItem
    pub fn is_breed_item(&self, stack: &ItemStack) -> bool {
        let kind = self.get_type();
        let tag = match kind.trim_start_matches("minecraft:") {
            "mooshroom" => "cow".to_owned(),
            "donkey" | "mule" => "horse".to_owned(),
            "trader_llama" => "llama".to_owned(),
            other => other.to_owned(),
        };
        let members = query(json!({ "op": "tag", "kind": "item", "name": format!("minecraft:{tag}_food") }));
        members.as_array().is_some_and(|items| items.iter().any(|item| item.as_str() == Some(stack.get_type().key())))
    }
}

/// `AbstractHorseInventory`, `HorseInventory`, `LlamaInventory` and the rest: a mount's saddle and armor.
#[derive(Clone, Debug, PartialEq)]
pub struct MountInventory(Entity);

impl MountInventory {
    /// The saddle.
    pub fn get_saddle(&self) -> Option<ItemStack> {
        ItemStack::read(&self.0.data_at("equipment.saddle"))
    }
    /// A new saddle.
    pub fn set_saddle(&self, stack: &ItemStack) -> bool {
        self.0.merge(&format!("{{equipment:{{saddle:{}}}}}", stack.to_snbt()))
    }
    /// The armor or carpet.
    pub fn get_armor(&self) -> Option<ItemStack> {
        ItemStack::read(&self.0.data_at("equipment.body"))
    }
    /// New armor or a new carpet.
    pub fn set_armor(&self, stack: &ItemStack) -> bool {
        self.0.merge(&format!("{{equipment:{{body:{}}}}}", stack.to_snbt()))
    }
}

/// `AbstractVillager.getInventory`: what it carries, live.
#[derive(Clone, Debug, PartialEq)]
pub struct VillagerInventory(Entity);

impl VillagerInventory {
    /// bukkit: org/bukkit/inventory/Inventory.getSize
    pub fn get_size(&self) -> usize {
        8
    }
    /// bukkit: org/bukkit/inventory/Inventory.getContents
    pub fn get_contents(&self) -> Vec<Option<ItemStack>> {
        let mut list: Vec<Option<ItemStack>> = get(&self.0, "villager_items").as_array().into_iter().flatten().map(ItemStack::read).collect();
        list.resize(8, None);
        list
    }
    /// bukkit: org/bukkit/inventory/Inventory.getItem
    pub fn get_item(&self, slot: usize) -> Option<ItemStack> {
        self.get_contents().get(slot).cloned().flatten()
    }
    /// bukkit: org/bukkit/inventory/Inventory.setItem
    pub fn set_item(&self, slot: usize, stack: Option<&ItemStack>) -> bool {
        done(&self.0, "villager_item", json!({ "slot": slot, "stack": stack.map(ItemStack::to_snbt) }))
    }
    /// bukkit: org/bukkit/inventory/Inventory.clear
    pub fn clear(&self) {
        while self.set_item(0, None) {}
    }
}

impl AbstractHorse {
    /// bukkit: org/bukkit/entity/AbstractHorse.getInventory
    pub fn get_inventory(&self) -> MountInventory {
        MountInventory(self.0.clone())
    }
}

impl Horse {
    /// bukkit: org/bukkit/entity/Horse.getInventory
    pub fn get_inventory(&self) -> MountInventory {
        MountInventory(self.0.clone())
    }
}

impl Llama {
    /// bukkit: org/bukkit/entity/Llama.getInventory
    pub fn get_inventory(&self) -> MountInventory {
        MountInventory(self.0.clone())
    }
}

/// `org.bukkit.entity.AbstractNautilus`.
#[derive(Clone, Debug, PartialEq)]
pub struct AbstractNautilus(pub Entity);

impl std::ops::Deref for AbstractNautilus {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for AbstractNautilus {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl AbstractNautilus {
    /// bukkit: org/bukkit/entity/AbstractNautilus.getInventory
    pub fn get_inventory(&self) -> MountInventory {
        MountInventory(self.0.clone())
    }
}

impl AbstractVillager {
    /// bukkit~: org/bukkit/entity/AbstractVillager.getInventory | live, but the game keeps a villager's list packed: a stack that runs out moves the later ones down a slot
    pub fn get_inventory(&self) -> VillagerInventory {
        VillagerInventory(self.0.clone())
    }
}

impl Enderman {
    /// bukkit: org/bukkit/entity/Enderman.getCarriedMaterial
    pub fn get_carried_material(&self) -> MaterialData {
        MaterialData::new(crate::Material::of(self.data_at("carriedBlockState.Name").as_str().unwrap_or("minecraft:air")))
    }
    /// bukkit: org/bukkit/entity/Enderman.setCarriedMaterial
    pub fn set_carried_material(&self, material: &MaterialData) -> bool {
        self.merge(&format!("{{carriedBlockState:{{Name:{}}}}}", crate::data::quoted(material.get_item_type().key())))
    }
}

impl Minecart {
    /// bukkit: org/bukkit/entity/Minecart.getDisplayBlock
    pub fn get_display_block(&self) -> MaterialData {
        MaterialData::new(crate::Material::of(self.get_display_block_data().as_deref().unwrap_or("minecraft:air")))
    }
}

impl RangedEntity {
    /// bukkit: com/destroystokyo/paper/entity/RangedEntity.rangedAttack
    pub fn ranged_attack(&self, target_entity: &Actor, charge: f32) -> bool {
        let at = match target_entity {
            Actor::Player(player) => Some(player.get_eye_location()),
            Actor::Entity(entity) => entity.get_location(),
        };
        let mut fields = who(target_entity);
        fields["value"] = json!(charge);
        fields["at"] = at.map_or(Value::Null, |at| json!([at.x, at.y, at.z]));
        done(&self.0, "ranged_attack", fields)
    }
}

impl Shearable {
    /// bukkit: io/papermc/paper/entity/Shearable.shear
    pub fn shear(&self) -> bool {
        done(&self.0, "shear", json!({}))
    }
}

impl SulfurCube {
    /// bukkit: org/bukkit/entity/SulfurCube.setFuseTicks
    pub fn set_fuse_ticks(&self, ticks: i32) -> bool {
        done(&self.0, "sulfur_fuse_set", json!({ "value": ticks }))
    }
    /// bukkit: org/bukkit/entity/SulfurCube.swallow
    pub fn swallow(&self, stack: &ItemStack) -> bool {
        done(&self.0, "sulfur_swallowable", json!({ "item": stack.get_type().key() })) && self.get_equipped().is_none() && self.set_equipped(stack)
    }
}

impl Fox {
    fn trusted(&self) -> Vec<String> {
        self.data_at("Trusted").as_array().into_iter().flatten().filter_map(crate::data::uuid).collect()
    }
    /// bukkit: org/bukkit/entity/Fox.getFirstTrustedPlayer
    pub fn get_first_trusted_player(&self) -> Option<OfflinePlayer> {
        self.trusted().first().map(|uuid| offline_by_uuid(uuid))
    }
    /// bukkit: org/bukkit/entity/Fox.getSecondTrustedPlayer
    pub fn get_second_trusted_player(&self) -> Option<OfflinePlayer> {
        self.trusted().get(1).map(|uuid| offline_by_uuid(uuid))
    }
    fn set_trusted(&self, first: Option<&str>, second: Option<&str>) -> bool {
        put(&self.0, "trusted", json!([first.map(words), second.map(words)]))
    }
    /// bukkit: org/bukkit/entity/Fox.setFirstTrustedPlayer
    ///
    /// False where Java throws: the second has to go first.
    pub fn set_first_trusted_player(&self, player: Option<&dyn AnimalTamer>) -> bool {
        let trusted = self.trusted();
        let (first, second) = (player.and_then(AnimalTamer::tamer_unique_id), trusted.get(1));
        (first.is_some() || second.is_none()) && self.set_trusted(first.as_deref(), second.map(String::as_str))
    }
    /// bukkit: org/bukkit/entity/Fox.setSecondTrustedPlayer
    ///
    /// False where Java throws: there has to be a first.
    pub fn set_second_trusted_player(&self, player: Option<&dyn AnimalTamer>) -> bool {
        let trusted = self.trusted();
        let (first, second) = (trusted.first(), player.and_then(AnimalTamer::tamer_unique_id));
        (second.is_none() || first.is_some()) && self.set_trusted(first.map(String::as_str), second.as_deref())
    }
}


impl Creeper {
    /// bukkit: org/bukkit/entity/Creeper.ignite
    pub fn ignite_by(&self, igniter: &Actor) -> bool {
        hold(&self.0, "igniter", who(igniter));
        self.ignite()
    }
    /// bukkit: org/bukkit/entity/Creeper.getIgniter
    pub fn get_igniter(&self) -> Option<Actor> {
        actor(&self.0, &hold(&self.0, "igniter_get", Value::Null))
    }
}

/// `org.bukkit.entity.FishHook`, of a player's cast: the game keeps one hook per player.
#[derive(Clone, Debug, PartialEq)]
pub struct FishHook(Player);

impl FishHook {
    fn state(&self) -> Value {
        query(json!({ "op": "entity_player_turn", "uuid": self.0.uuid(), "key": "hook" }))
    }
    /// bukkit: org/bukkit/entity/FishHook.getState
    pub fn get_state(&self) -> &'static str {
        match self.state()["state"].as_str().unwrap_or_default() {
            "Bobbing" => "BOBBING",
            state if state.starts_with("Hooked") => "HOOKED_ENTITY",
            _ => "UNHOOKED",
        }
    }
    /// bukkit: org/bukkit/entity/FishHook.getHookedEntity
    pub fn get_hooked_entity(&self) -> Option<Actor> {
        let state = self.state();
        actor_in(&self.0.get_world(), state["state"].as_str()?.strip_prefix("Hooked(")?.strip_suffix(')')?.parse().ok()?)
    }
    /// bukkit: org/bukkit/entity/FishHook.isInOpenWater
    pub fn is_in_open_water(&self) -> bool {
        self.state()["open_water"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/FishHook.getWaitTime
    pub fn get_wait_time(&self) -> i32 {
        self.state()["lured_in"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/FishHook.getTimeUntilBite
    pub fn get_time_until_bite(&self) -> i32 {
        self.state()["hooked_in"].as_i64().unwrap_or(0) as i32
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/HumanEntity.getFishHook
    pub fn get_fish_hook(&self) -> Option<FishHook> {
        let hook = FishHook(self.clone());
        (!hook.state().is_null()).then_some(hook)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getEnderChest
    pub fn get_ender_chest(&self) -> crate::fam_items::EnderChest {
        self.ender_chest()
    }
    /// One slot of the ender chest, at once.
    pub fn set_ender_chest_item(&self, slot: usize, stack: Option<&ItemStack>) -> bool {
        send_player(self, "ender_item", json!({ "slot": slot, "stack": stack_snbt(stack) }))
    }
    /// bukkit: org/bukkit/entity/Player.getNoTickViewDistance
    pub fn get_no_tick_view_distance(&self) -> i32 {
        self.get_view_distance()
    }
    /// bukkit: org/bukkit/entity/Player.setNoTickViewDistance
    pub fn set_no_tick_view_distance(&self, distance: i32) -> bool {
        self.set_view_distance(distance)
    }
    /// bukkit: org/bukkit/entity/Player.identity
    pub fn identity(&self) -> Identity {
        Identity(crate::dashed(self.uuid()))
    }
    /// bukkit: org/bukkit/entity/Player.sendLinks
    pub fn send_links(&self, links: &[(Component, String)]) -> bool {
        let rows: Vec<Value> = links.iter().map(|(label, url)| json!({ "label": label.0, "url": url })).collect();
        send_player(self, "links", json!({ "links": rows }))
    }
    /// bukkit: org/bukkit/entity/Player.setPlayerProfile
    pub fn set_player_profile(&self, name: &str, properties: &[(String, String, Option<String>)]) -> bool {
        let rows: Vec<Value> = properties.iter().map(|(name, value, signature)| json!({ "name": name, "value": value, "signature": signature })).collect();
        send_player(self, "profile", json!({ "name": name, "properties": rows }))
    }
}

// ---- Mannequins ----

/// `org.bukkit.entity.Mannequin`.
#[derive(Clone, Debug, PartialEq)]
pub struct Mannequin(pub Entity);

impl std::ops::Deref for Mannequin {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl EntityClass for Mannequin {
    fn of(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Mannequin {
    /// bukkit: org/bukkit/entity/Mannequin.validPoses
    pub fn valid_poses() -> &'static [&'static str] {
        &["STANDING", "SNEAKING", "SWIMMING", "FALL_FLYING", "SLEEPING"]
    }
    /// bukkit: org/bukkit/entity/Mannequin.defaultDescription
    pub fn default_description() -> Component {
        Component(json!({ "translate": "entity.minecraft.mannequin.label" }))
    }
    /// bukkit: org/bukkit/entity/Mannequin.getProfile
    pub fn get_profile(&self) -> crate::components::ResolvableProfile {
        crate::components::ResolvableProfile(self.data_at("profile"))
    }
    /// bukkit: org/bukkit/entity/Mannequin.setProfile
    pub fn set_profile(&self, profile: &crate::components::ResolvableProfile) -> bool {
        self.merge(&format!("{{profile:{}}}", crate::data::snbt(&profile.0)))
    }
    /// bukkit: org/bukkit/entity/Mannequin.getDescription
    pub fn get_description(&self) -> Option<Component> {
        let saved = self.data_at("description");
        (!saved.is_null()).then_some(Component(saved))
    }
    /// bukkit: org/bukkit/entity/Mannequin.setDescription
    pub fn set_description(&self, description: Option<&Component>) -> bool {
        put(&self.0, "description", description.map_or(Value::Null, |text| json!(crate::data::snbt(&text.0))))
    }
    /// bukkit: org/bukkit/entity/Mannequin.isImmovable
    pub fn is_immovable(&self) -> bool {
        crate::data::flag(&self.data_at("immovable"))
    }
    /// bukkit: org/bukkit/entity/Mannequin.setImmovable
    pub fn set_immovable(&self, immovable: bool) -> bool {
        put(&self.0, "immovable", json!(immovable))
    }
    /// bukkit: org/bukkit/entity/Mannequin.getMainHand
    pub fn get_main_hand(&self) -> &'static str {
        if self.is_left_handed() { "LEFT" } else { "RIGHT" }
    }
    /// bukkit: org/bukkit/entity/Mannequin.setMainHand
    pub fn set_main_hand(&self, hand: &str) -> bool {
        self.set_left_handed(hand == "LEFT")
    }
}

impl Player {
}

impl AbstractWindCharge {
    /// bukkit: org/bukkit/entity/AbstractWindCharge.explode
    pub fn explode(&self) -> bool {
        object_done(&self.0, "wind_explode", json!({}))
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/Player.getDeathScreenScore
    pub fn get_death_screen_score(&self) -> i32 {
        ask_player(self, "death_score").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.setDeathScreenScore
    pub fn set_death_screen_score(&self, score: i32) -> bool {
        send_player(self, "death_score", json!({ "value": score }))
    }
}

// ---- What every entity answers: `Entity`, `LivingEntity`, `Mob` ----

fn common(entity: &Entity, key: &str, fields: Value) -> Value {
    let mut request = fields;
    request["op"] = json!("entity_common");
    request["id"] = json!(entity.get_entity_id());
    request["dimension"] = json!(entity.get_world().dimension());
    request["key"] = json!(key);
    query(request)
}

/// `net.kyori.adventure.util.TriState`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TriState {
    #[default]
    NotSet,
    True,
    False,
}

/// `EntityDamageEvent` as `getLastDamageCause` keeps it: why and how much.
#[derive(Clone, Debug, PartialEq)]
pub struct LastDamage {
    cause: String,
    damage: f64,
    /// Who hit: the game's `{ "id": mob }` or `{ "uuid": player }`, or null.
    attacker: Value,
    /// Where the hit one is, to find a mob attacker in.
    world: Option<World>,
}

impl LastDamage {
    /// A record of a hit, for `set_last_damage_cause`.
    pub fn new(cause: &str, damage: f64) -> Self {
        Self { cause: cause.to_owned(), damage, attacker: Value::Null, world: None }
    }
    /// Whether something hit it: Bukkit's last damage cause is then an `EntityDamageByEntityEvent`.
    pub fn has_damager(&self) -> bool {
        !self.attacker.is_null()
    }
    /// bukkit: org/bukkit/event/entity/EntityDamageByEntityEvent.getDamager
    ///
    /// The player or mob that hit it last (CoreProtect logs a kill by it).
    pub fn get_damager(&self) -> Option<crate::fam_ev_entity::Actor> {
        if let Some(uuid) = self.attacker["uuid"].as_str() {
            return Player::read(&query(json!({ "op": "player", "uuid": uuid }))).map(crate::fam_ev_entity::Actor::Player);
        }
        let id = self.attacker["id"].as_u64()?;
        Some(crate::fam_ev_entity::Actor::Entity(Entity::of(id, self.world.as_ref()?)))
    }
    /// `DamageCause`'s name.
    pub fn get_cause(&self) -> &str {
        &self.cause
    }
    /// The amount.
    pub fn get_damage(&self) -> f64 {
        self.damage
    }
}

static LAST_DAMAGE: std::sync::Mutex<std::collections::BTreeMap<u64, LastDamage>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
/// Who locked their freeze ticks, at what value.
/// `Entity.setVisibleByDefault(false)`, and who has been shown each one.
static UNSEEN: std::sync::Mutex<std::collections::BTreeMap<u64, Vec<String>>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
static UNSEEN_RUN: std::sync::OnceLock<()> = std::sync::OnceLock::new();
/// What `Entity.addAdditionalAlliedRule` takes.
pub type AlliedRule = std::sync::Arc<dyn Fn(&Entity) -> bool + Send + Sync>;
static ALLIES: std::sync::Mutex<std::collections::BTreeMap<u64, Vec<(String, AlliedRule)>>> = std::sync::Mutex::new(std::collections::BTreeMap::new());

/// The damage kinds the game names, as `EntityDamageEvent.DamageCause`.
fn damage_cause(kind: &str) -> &'static str {
    match kind {
        "Fall" | "Stalagmite" => "FALL",
        "OnFire" => "FIRE_TICK",
        "InFire" | "Campfire" | "SulfurCubeHot" => "FIRE",
        "Lava" => "LAVA",
        "HotFloor" => "HOT_FLOOR",
        "Drown" => "DROWNING",
        "Starve" => "STARVATION",
        "InWall" => "SUFFOCATION",
        "Cramming" => "CRAMMING",
        "Cactus" | "SweetBerryBush" => "CONTACT",
        "Magic" | "IndirectMagic" => "MAGIC",
        "Wither" => "WITHER",
        "Freeze" => "FREEZE",
        "DryOut" => "DRYOUT",
        "LightningBolt" => "LIGHTNING",
        "OutOfWorld" | "GenericKill" => "VOID",
        "OutsideBorder" => "WORLD_BORDER",
        "FallingBlock" | "FallingAnvil" => "FALLING_BLOCK",
        "Explosion" | "PlayerExplosion" => "BLOCK_EXPLOSION",
        "Arrow" | "Thrown" | "Fireball" | "WindCharge" | "Spit" => "PROJECTILE",
        "SonicBoom" => "SONIC_BOOM",
        "Thorns" => "THORNS",
        "MobAttack" | "PlayerAttack" | "MaceSmash" | "Spear" => "ENTITY_ATTACK",
        _ => "CUSTOM",
    }
}

const EFFECTS: &[(&str, i8)] = &[
    ("ARROW_PARTICLES", 0),
    ("RABBIT_JUMP", 1),
    ("RESET_SPAWNER_MINECART_DELAY", 1),
    ("HIT", 2),
    ("HURT", 2),
    ("DEATH", 3),
    ("EGG_BREAK", 3),
    ("SNOWBALL_BREAK", 3),
    ("PROJECTILE_CRACK", 3),
    ("ENTITY_DEATH", 3),
    ("FANG_ATTACK", 4),
    ("HOGLIN_ATTACK", 4),
    ("IRON_GOLEN_ATTACK", 4),
    ("RAVAGER_ATTACK", 4),
    ("WARDEN_ATTACK", 4),
    ("ZOGLIN_ATTACK", 4),
    ("ENTITY_ATTACK", 4),
    ("WOLF_SMOKE", 6),
    ("WOLF_HEARTS", 7),
    ("TAMING_FAILED", 6),
    ("TAMING_SUCCEEDED", 7),
    ("WOLF_SHAKE", 8),
    ("SHEEP_EAT", 10),
    ("SHEEP_EAT_GRASS", 10),
    ("IRON_GOLEM_ROSE", 11),
    ("VILLAGER_HEART", 12),
    ("VILLAGER_ANGRY", 13),
    ("VILLAGER_HAPPY", 14),
    ("WITCH_MAGIC", 15),
    ("ZOMBIE_TRANSFORM", 16),
    ("FIREWORK_EXPLODE", 17),
    ("LOVE_HEARTS", 18),
    ("SQUID_ROTATE", 19),
    ("ENTITY_POOF", 20),
    ("GUARDIAN_TARGET", 21),
    ("SHIELD_BLOCK", 29),
    ("SHIELD_BREAK", 30),
    ("ARMOR_STAND_HIT", 32),
    ("THORNS_HURT", 33),
    ("IRON_GOLEM_SHEATH", 34),
    ("TOTEM_RESURRECT", 35),
    ("PROTECTED_FROM_DEATH", 35),
    ("HURT_DROWN", 36),
    ("HURT_EXPLOSION", 37),
    ("DOLPHIN_FED", 38),
    ("RAVAGER_STUNNED", 39),
    ("CAT_TAME_FAIL", 40),
    ("CAT_TAME_SUCCESS", 41),
    ("TRUSTING_FAILED", 40),
    ("TRUSTING_SUCCEEDED", 41),
    ("VILLAGER_SPLASH", 42),
    ("PLAYER_BAD_OMEN_RAID", 43),
    ("HURT_BERRY_BUSH", 44),
    ("FOX_CHEW", 45),
    ("TELEPORT_ENDER", 46),
    ("BREAK_EQUIPMENT_MAIN_HAND", 47),
    ("BREAK_EQUIPMENT_OFF_HAND", 48),
    ("BREAK_EQUIPMENT_HELMET", 49),
    ("BREAK_EQUIPMENT_CHESTPLATE", 50),
    ("BREAK_EQUIPMENT_LEGGINGS", 51),
    ("BREAK_EQUIPMENT_BOOTS", 52),
    ("HONEY_BLOCK_SLIDE_PARTICLES", 53),
    ("HONEY_BLOCK_FALL_PARTICLES", 54),
    ("SWAP_HAND_ITEMS", 55),
    ("WOLF_SHAKE_STOP", 56),
    ("GOAT_LOWER_HEAD", 58),
    ("GOAT_RAISE_HEAD", 59),
    ("SPAWN_DEATH_SMOKE", 60),
    ("WARDEN_TENDRIL_SHAKE", 61),
    ("WARDEN_SONIC_ATTACK", 62),
    ("SNIFFER_DIG", 63),
    ("ARMADILLO_PEEK", 64),
    ("BODY_BREAK", 65),
    ("BREAK_EQUIPMENT_BODY", 65),
    ("SHAKE", 66),
    ("DROWN_PARTICLES", 67),
    ("BREAK_EQUIPMENT_SADDLE", 68),
    ("RAVAGER_ROARED", 69),
    ("TNT_MINECART_IGNITE", 70),
    ("BOAT_SINK", 71),
    ("BOAT_LAUNCH", 72),
];

impl Entity {
    /// bukkit: org/bukkit/entity/Entity.getTicksLived
    pub fn get_ticks_lived(&self) -> i32 {
        common(self, "ticks", json!({})).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Entity.setTicksLived
    pub fn set_ticks_lived(&self, ticks: i32) -> bool {
        common(self, "set_ticks", json!({ "value": ticks })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Entity.getLastDamageCause
    pub fn get_last_damage_cause(&self) -> Option<LastDamage> {
        if let Some(set) = LAST_DAMAGE.lock().unwrap_or_else(|e| e.into_inner()).get(&self.get_entity_id()) {
            return Some(set.clone());
        }
        let last = get(self, "last_damage");
        Some(LastDamage { cause: damage_cause(last["kind"].as_str()?).to_owned(), damage: last["amount"].as_f64().unwrap_or(0.0), attacker: last["attacker"].clone(), world: Some(self.get_world()) })
    }
    /// bukkit: org/bukkit/entity/Entity.setLastDamageCause
    pub fn set_last_damage_cause(&self, cause: Option<LastDamage>) {
        let mut all = LAST_DAMAGE.lock().unwrap_or_else(|e| e.into_inner());
        match cause {
            Some(cause) => all.insert(self.get_entity_id(), cause),
            None => all.remove(&self.get_entity_id()),
        };
    }
    /// bukkit: org/bukkit/entity/Entity.getVehicle
    pub fn get_vehicle(&self) -> Option<Entity> {
        Some(Entity::of(get(self, "vehicle").as_u64()?, &self.get_world()))
    }
    /// bukkit: org/bukkit/entity/Entity.getScoreboardEntryName
    pub fn get_scoreboard_entry_name(&self) -> String {
        self.get_unique_id()
    }
    /// bukkit: org/bukkit/entity/Entity.isVisibleByDefault
    pub fn is_visible_by_default(&self) -> bool {
        !UNSEEN.lock().unwrap_or_else(|e| e.into_inner()).contains_key(&self.get_entity_id())
    }
    /// bukkit: org/bukkit/entity/Entity.setVisibleByDefault
    pub fn set_visible_by_default(&self, visible: bool) {
        {
            let mut all = UNSEEN.lock().unwrap_or_else(|e| e.into_inner());
            if visible {
                all.remove(&self.get_entity_id());
            } else {
                all.entry(self.get_entity_id()).or_default();
            }
        }
        UNSEEN_RUN.get_or_init(|| {
            crate::scheduler::run_task_timer(20, 20, sync_unseen);
        });
        sync_unseen();
    }
    /// bukkit: org/bukkit/entity/Entity.copy
    pub fn copy(&self) -> Option<Entity> {
        let at = self.get_location()?;
        self.copy_to(&at)
    }
    /// bukkit: org/bukkit/entity/Entity.copy
    pub fn copy_to(&self, at: &Location) -> Option<Entity> {
        Entity::read(&common(self, "copy", json!({ "at": [at.x, at.y, at.z], "to": at.world.dimension() })))
    }
    /// bukkit: org/bukkit/entity/Entity.getAsString
    pub fn get_as_string(&self) -> Option<String> {
        common(self, "as_string", json!({})).as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/entity/Entity.createSnapshot
    pub fn create_snapshot(&self) -> Option<crate::entities3::EntitySnapshot> {
        self.get_as_string().map(crate::entities3::EntitySnapshot)
    }
    /// bukkit: org/bukkit/entity/Entity.setSneaking
    pub fn set_sneaking(&self, sneaking: bool) -> bool {
        plugin_set(self, "sneaking", json!(sneaking));
        common(self, "shared_flag", json!({ "bit": 2, "value": sneaking })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Entity.getPose
    pub fn get_pose(&self) -> &'static str {
        crate::enums::pose::values().get(number(self, "pose") as usize).copied().unwrap_or("STANDING")
    }
    /// bukkit: org/bukkit/entity/Entity.setPose
    pub fn set_pose(&self, pose: &str) -> bool {
        self.set_fixed_pose(pose, false)
    }
    /// bukkit: org/bukkit/entity/Entity.setPose
    ///
    /// Java's `setPose(pose, fixed)`.
    pub fn set_fixed_pose(&self, pose: &str, fixed: bool) -> bool {
        let Some(index) = crate::enums::pose::values().iter().position(|name| *name == pose) else {
            return false;
        };
        let _ = put(self, "pose", json!(index));
        plugin_set(self, "pose", json!(index)) && plugin_set(self, "pose_fixed", json!(fixed))
    }
    /// bukkit: org/bukkit/entity/Entity.hasFixedPose
    pub fn has_fixed_pose(&self) -> bool {
        plugin_get(self, "pose_fixed").as_f64() == Some(1.0)
    }
    /// bukkit: org/bukkit/entity/Entity.hasNoPhysics
    pub fn has_no_physics(&self) -> bool {
        common(self, "no_physics", json!({})).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Entity.setNoPhysics
    pub fn set_no_physics(&self, value: bool) -> bool {
        common(self, "set_no_physics", json!({ "value": value })).as_bool().unwrap_or(false) || plugin_set(self, "no_physics", json!(value))
    }
    /// bukkit: org/bukkit/entity/Entity.isInvisible
    pub fn is_invisible(&self) -> bool {
        if is_type(self, &["minecraft:armor_stand"]) {
            return crate::data::flag(&self.data_at("Invisible"));
        }
        flag(self, "invisible")
    }
    /// bukkit: org/bukkit/entity/Entity.setInvisible
    pub fn set_invisible(&self, invisible: bool) -> bool {
        if is_type(self, &["minecraft:armor_stand"]) {
            return self.merge(&format!("{{Invisible:{invisible}}}"));
        }
        if put(self, "invisible", json!(invisible)) {
            return true;
        }
        plugin_set(self, "invisible", json!(invisible));
        common(self, "shared_flag", json!({ "bit": 32, "value": invisible })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Entity.playEffect
    pub fn play_effect(&self, effect: &str) -> bool {
        EFFECTS.iter().find(|(name, _)| *name == effect).is_some_and(|(_, event)| common(self, "event", json!({ "event": event })).as_bool().unwrap_or(false))
    }
    /// bukkit: org/bukkit/entity/Entity.lookAt
    pub fn look_at_point(&self, x: f64, y: f64, z: f64, anchor: &str) -> bool {
        common(self, "look", json!({ "at": [x, y, z], "eyes": anchor == "EYES" })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Entity.lockFreezeTicks
    pub fn lock_freeze_ticks(&self, locked: bool) {
        let id = self.get_entity_id();
        match crate::get_online_players().into_iter().find(|player| player.get_entity_id() as u64 == id) {
            Some(player) => {
                player.player_set("plugin:freeze_locked", json!(if locked { 1.0 } else { 0.0 }));
            }
            None => {
                plugin_set(self, "freeze_locked", json!(if locked { 1.0 } else { 0.0 }));
            }
        }
    }
    /// bukkit: org/bukkit/entity/Entity.isFreezeTickingLocked
    pub fn is_freeze_ticking_locked(&self) -> bool {
        let id = self.get_entity_id();
        match crate::get_online_players().into_iter().find(|player| player.get_entity_id() as u64 == id) {
            Some(player) => player.player_plugin("freeze_locked").as_f64() == Some(1.0),
            None => plugin_get(self, "freeze_locked").as_f64() == Some(1.0),
        }
    }
    /// bukkit: org/bukkit/entity/Entity.broadcastHurtAnimation
    pub fn broadcast_hurt_animation(&self, players: &[Player]) {
        let yaw = self.get_location().map_or(0.0, |at| at.get_yaw());
        for player in players {
            player.send("hurt_animation", json!({ "id": self.get_entity_id(), "yaw": yaw }));
        }
    }
    /// bukkit: org/bukkit/entity/Entity.getMaxFireTicks
    ///
    /// `Entity.getFireImmuneTicks`: nothing overrides it but a player.
    pub fn get_max_fire_ticks(&self) -> i32 {
        let id = self.get_entity_id();
        if crate::get_online_players().iter().any(|player| player.get_entity_id() as u64 == id) { PLAYER_FIRE_TICKS } else { 0 }
    }
    /// bukkit: org/bukkit/entity/Entity.getYaw
    pub fn get_yaw(&self) -> f32 {
        self.get_location().map_or(0.0, |at| at.get_yaw())
    }
    /// bukkit: org/bukkit/entity/Entity.getPitch
    pub fn get_pitch(&self) -> f32 {
        self.get_location().map_or(0.0, |at| at.get_pitch())
    }
    /// bukkit: org/bukkit/entity/Entity.getGravity
    pub fn get_gravity(&self) -> f64 {
        if self.has_gravity() { self.get_default_gravity() } else { 0.0 }
    }
    /// bukkit: org/bukkit/entity/Entity.getDefaultGravity
    pub fn get_default_gravity(&self) -> f64 {
        if let Some(gravity) = get(self, "gravity").as_f64() {
            return gravity;
        }
        let id = self.get_entity_id();
        if let Some(player) = crate::get_online_players().into_iter().find(|player| player.get_entity_id() as u64 == id) {
            return player.get_attribute("minecraft:gravity").get_value();
        }
        match self.get_type().trim_start_matches("minecraft:") {
            "arrow" | "spectral_arrow" | "trident" => 0.05,
            "snowball" | "egg" | "ender_pearl" | "experience_orb" | "splash_potion" | "lingering_potion" | "experience_bottle" => 0.03,
            "fireball" | "small_fireball" | "dragon_fireball" | "wither_skull" | "wind_charge" | "breeze_wind_charge" | "shulker_bullet" => 0.0,
            "armor_stand" => 0.08,
            _ => 0.04,
        }
    }
    /// bukkit: org/bukkit/entity/Entity.wouldCollideUsing
    pub fn would_collide_using(&self, area: &crate::util::BoundingBox) -> bool {
        common(self, "collides", json!({ "min": [area.get_min_x(), area.get_min_y(), area.get_min_z()], "max": [area.get_max_x(), area.get_max_y(), area.get_max_z()] })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Entity.collidesAt
    pub fn collides_at(&self, at: &Location) -> bool {
        let half = self.get_width() / 2.0;
        self.would_collide_using(&crate::util::BoundingBox::new(at.x - half, at.y, at.z - half, at.x + half, at.y + self.get_height(), at.z + half))
    }
    fn fluid(&self, key: &str, block: &str) -> bool {
        match get(self, key).as_bool() {
            Some(answer) => answer,
            None => self.get_location().is_some_and(|at| at.get_block().get_type().key() == block),
        }
    }
    /// bukkit: org/bukkit/entity/Entity.isUnderWater
    pub fn is_under_water(&self) -> bool {
        self.fluid("eye_in_water", "minecraft:water")
    }
    /// bukkit: org/bukkit/entity/Entity.isInLava
    pub fn is_in_lava(&self) -> bool {
        self.fluid("in_lava", "minecraft:lava")
    }
    /// bukkit: org/bukkit/entity/Entity.isInBubbleColumn
    pub fn is_in_bubble_column(&self) -> bool {
        self.get_location().is_some_and(|at| at.get_block().get_type().key() == "minecraft:bubble_column")
    }
    /// bukkit: org/bukkit/entity/Entity.isInPowderedSnow
    pub fn is_in_powdered_snow(&self) -> bool {
        self.get_location().is_some_and(|at| at.get_block().get_type().key() == "minecraft:powder_snow")
    }
    /// bukkit: org/bukkit/entity/Entity.isInRain
    pub fn is_in_rain(&self) -> bool {
        common(self, "rain", json!({})).as_bool().unwrap_or(false)
    }
    fn in_water_now(&self) -> bool {
        self.fluid("in_water", "minecraft:water")
    }
    /// bukkit: org/bukkit/entity/Entity.isInWaterOrRain
    pub fn is_in_water_or_rain(&self) -> bool {
        self.in_water_now() || self.is_in_rain()
    }
    /// bukkit: org/bukkit/entity/Entity.isInWaterOrBubbleColumn
    pub fn is_in_water_or_bubble_column(&self) -> bool {
        self.in_water_now() || self.is_in_bubble_column()
    }
    /// bukkit: org/bukkit/entity/Entity.isInWaterOrRainOrBubbleColumn
    pub fn is_in_water_or_rain_or_bubble_column(&self) -> bool {
        self.is_in_water_or_rain() || self.is_in_bubble_column()
    }
    /// bukkit: org/bukkit/entity/Entity.getTrackedPlayers, org/bukkit/entity/Entity.getTrackedBy
    pub fn get_tracked_players(&self) -> Vec<Player> {
        let ids = common(self, "tracked_by", json!({}));
        let uuids: Vec<&str> = ids.as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        crate::get_online_players().into_iter().filter(|player| uuids.contains(&player.uuid())).collect()
    }
    /// bukkit: org/bukkit/entity/Entity.isTrackedBy
    pub fn is_tracked_by(&self, player: &Player) -> bool {
        self.get_tracked_players().iter().any(|known| known == player)
    }
    /// bukkit: org/bukkit/entity/Entity.isTicking
    pub fn is_ticking(&self) -> bool {
        common(self, "ticking", json!({})).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Entity.isInWorld
    pub fn is_in_world(&self) -> bool {
        self.is_valid()
    }
    /// bukkit: org/bukkit/entity/Entity.isMarkedInvulnerable
    pub fn is_marked_invulnerable(&self) -> bool {
        self.is_invulnerable()
    }
    /// bukkit: org/bukkit/entity/Entity.isInvulnerableTo
    ///
    /// `Entity.isInvulnerableToBase`: gone, the flag unless the damage passes it or a creative player deals it, fire on
    /// what doesn't burn, a fall on what the `fall_damage_immune` tag holds.
    pub fn is_invulnerable_to(&self, source: &crate::core2::DamageSource) -> bool {
        let kind = source.get_damage_type().trim_start_matches("minecraft:");
        let creative = source.get_causing_entity().and_then(|entity| online(&entity.get_unique_id())).is_some_and(|player| player.get_game_mode() == "CREATIVE");
        let fire = matches!(kind, "in_fire" | "campfire" | "on_fire" | "lava" | "hot_floor" | "sulfur_cube_hot" | "unattributed_fireball" | "fireball");
        self.is_dead()
            || (self.is_invulnerable() && !matches!(kind, "out_of_world" | "generic_kill") && !creative)
            || (fire && (flag(self, "fire_immune") || is_type(self, FIRE_IMMUNE)))
            || (matches!(kind, "fall" | "ender_pearl" | "stalagmite") && is_type(self, FALL_IMMUNE))
    }
    /// bukkit~: org/bukkit/entity/Entity.isAlliedTo | a shared team or a rule of `add_additional_allied_rule`: the game's own targeting asks only the team
    pub fn is_allied_to(&self, other: &Entity) -> bool {
        let board = crate::scoreboard::get_main_scoreboard();
        if let (Some(mine), Some(theirs)) = (board.get_entry_team(&self.get_unique_id()), board.get_entry_team(&other.get_unique_id()))
            && mine == theirs
        {
            return true;
        }
        self.get_additional_allied_rules().values().any(|rule| rule(other))
    }
    /// bukkit~: org/bukkit/entity/Entity.addAdditionalAlliedRule | `is_allied_to` honours it: the game's own targeting doesn't ask
    pub fn add_additional_allied_rule(&self, key: &str, rule: impl Fn(&Entity) -> bool + Send + Sync + 'static) {
        let mut all = ALLIES.lock().unwrap_or_else(|e| e.into_inner());
        let rules = all.entry(self.get_entity_id()).or_default();
        rules.retain(|(name, _)| name != key);
        rules.push((key.to_owned(), std::sync::Arc::new(rule)));
    }
    /// bukkit: org/bukkit/entity/Entity.removeAdditionalAlliedRule
    pub fn remove_additional_allied_rule(&self, key: &str) {
        if let Some(rules) = ALLIES.lock().unwrap_or_else(|e| e.into_inner()).get_mut(&self.get_entity_id()) {
            rules.retain(|(name, _)| name != key);
        }
    }
    /// bukkit: org/bukkit/entity/Entity.getAdditionalAlliedRules
    pub fn get_additional_allied_rules(&self) -> std::collections::BTreeMap<String, AlliedRule> {
        ALLIES.lock().unwrap_or_else(|e| e.into_inner()).get(&self.get_entity_id()).map_or_else(Default::default, |rules| rules.iter().cloned().collect())
    }
    /// bukkit: org/bukkit/entity/Entity.getVisualFire
    pub fn get_visual_fire(&self) -> TriState {
        if self.is_visual_fire() { TriState::True } else { TriState::NotSet }
    }
    /// bukkit: org/bukkit/entity/Entity.getSpawnCategory
    pub fn get_spawn_category(&self) -> String {
        crate::entities3::entity_type_spawn_category(&self.get_type()).unwrap_or_else(|| "MISC".to_owned())
    }
    /// bukkit: org/bukkit/entity/Entity.getSoundCategory
    pub fn get_sound_category(&self) -> &'static str {
        match self.get_spawn_category().as_str() {
            "MONSTER" => "HOSTILE",
            "AMBIENT" => "AMBIENT",
            _ => "NEUTRAL",
        }
    }
    /// bukkit: org/bukkit/entity/Entity.getSwimSound
    pub fn get_swim_sound(&self) -> Option<String> {
        get(self, "swim_sound").as_str().map(|sound| format!("minecraft:{sound}"))
    }
    /// bukkit: org/bukkit/entity/Entity.getSwimSplashSound, org/bukkit/entity/Entity.getSwimHighSpeedSplashSound
    pub fn get_swim_splash_sound(&self) -> &'static str {
        match self.get_type().as_str() {
            "minecraft:dolphin" => "minecraft:entity.dolphin.splash",
            "minecraft:axolotl" => "minecraft:entity.axolotl.splash",
            _ if self.get_spawn_category() == "MONSTER" => "minecraft:entity.hostile.splash",
            _ => "minecraft:entity.generic.splash",
        }
    }
    /// bukkit: org/bukkit/entity/Entity.getPistonMoveReaction
    pub fn get_piston_move_reaction(&self) -> &'static str {
        let ignored = ["minecraft:area_effect_cloud", "minecraft:marker", "minecraft:interaction", "minecraft:painting", "minecraft:item_frame", "minecraft:glow_item_frame", "minecraft:leash_knot", "minecraft:lightning_bolt"];
        let marker = is_type(self, &["minecraft:armor_stand"]) && crate::data::flag(&self.data_at("Marker"));
        if marker || is_type(self, &ignored) { "IGNORE" } else { "MOVE" }
    }
    /// bukkit: org/bukkit/entity/Entity.getWeapon
    pub fn get_weapon_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.data_at("weapon")).or_else(|| ItemStack::read(&self.data_at("equipment.mainhand")))
    }
    /// bukkit: org/bukkit/entity/Entity.getPickItemStack
    pub fn get_pick_item_stack(&self) -> Option<ItemStack> {
        ItemStack::read(&common(self, "pick", json!({})))
    }
    /// bukkit: org/bukkit/entity/Entity.asHoverEvent
    pub fn as_hover_event(&self) -> Value {
        json!({ "action": "show_entity", "id": self.get_type(), "uuid": self.get_unique_id(), "name": self.custom_name().map(|name| name.0) })
    }
    /// bukkit: org/bukkit/entity/Entity.teamDisplayName
    pub fn team_display_name(&self) -> Component {
        let name = self.custom_name().unwrap_or_else(|| Component(json!({ "translate": format!("entity.{}", self.get_type().replace(':', ".")) })));
        match crate::scoreboard::get_main_scoreboard().get_entry_team(&self.get_unique_id()) {
            Some(team) => Component(json!({ "text": "", "extra": [team.prefix().0, name.0, team.suffix().0] })),
            None => name,
        }
    }
    /// bukkit: org/bukkit/entity/Entity.getChunk
    pub fn get_chunk(&self) -> Option<crate::entity::Chunk> {
        self.get_location().map(|at| at.get_chunk())
    }
    /// bukkit: org/bukkit/entity/Entity.spigot
    pub fn spigot(&self) -> &Self {
        self
    }
}

/// bukkit: org/bukkit/entity/Entity.getServer
///
/// In Rust the server's calls are free functions: there's no object to fetch.
pub const ENTITY_SERVER: () = ();

/// What the tracker must keep from each viewer: every hidden entity, less who was shown it.
fn sync_unseen() {
    let all = UNSEEN.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let mut pairs = Vec::new();
    for player in crate::get_online_players() {
        for id in crate::entities3::hidden_of(&player) {
            pairs.push(json!([player.uuid(), id]));
        }
        for (id, shown) in &all {
            if !shown.iter().any(|uuid| uuid == player.uuid()) {
                pairs.push(json!([player.uuid(), id]));
            }
        }
    }
    query(json!({ "op": "entity_unseen", "pairs": pairs }));
}

// ---- Living things ----

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getKiller
    pub fn get_killer(&self) -> Option<Player> {
        let uuid = get_uuid(self, "killer")?;
        crate::get_online_players().into_iter().find(|player| player.uuid() == uuid)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setKiller
    pub fn set_killer(&self, killer: Option<&Player>) -> bool {
        put(self, "killer", killer.map_or(Value::Null, |player| json!(player.uuid())))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setNoDamageTicks
    pub fn set_no_damage_ticks(&self, ticks: i32) -> bool {
        put(self, "invulnerable_ticks", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getMaximumNoDamageTicks
    pub fn get_maximum_no_damage_ticks(&self) -> i32 {
        plugin_get(self, "no_damage").as_f64().map_or(20, |ticks| ticks as i32)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setLastDamage
    pub fn set_last_damage(&self, amount: f64) -> bool {
        put(self, "last_hurt", json!(amount))
    }
    fn swing(&self, off_hand: bool) -> bool {
        common(self, "swing", json!({ "off": off_hand })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.swingMainHand
    pub fn swing_main_hand(&self) -> bool {
        self.swing(false)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.swingOffHand
    pub fn swing_off_hand(&self) -> bool {
        self.swing(true)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.swingHand
    pub fn swing_hand(&self, slot: &str) -> bool {
        self.swing(slot == "OFF_HAND")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.startUsingItem
    pub fn start_using_item(&self, slot: &str) -> bool {
        put(self, "use_hand", json!(slot == "OFF_HAND")) && put(self, "using_item", json!(1))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.clearActiveItem
    pub fn clear_active_item(&self) -> bool {
        put(self, "using_item", json!(0))
    }
    /// bukkit~: org/bukkit/entity/LivingEntity.completeUsingActiveItem | the use ends: what eating or drinking gives is the game's own to a mob
    pub fn complete_using_active_item(&self) -> bool {
        self.clear_active_item()
    }
    /// bukkit: org/bukkit/entity/LivingEntity.hasActiveItem, org/bukkit/entity/LivingEntity.isHandRaised
    pub fn has_active_item(&self) -> bool {
        number(self, "using_item") > 0
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getItemInUseTicks, org/bukkit/entity/LivingEntity.getActiveItemRemainingTime, org/bukkit/entity/LivingEntity.getItemUseRemainingTime
    pub fn get_item_in_use_ticks(&self) -> i32 {
        number(self, "use_left")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setItemInUseTicks, org/bukkit/entity/LivingEntity.setActiveItemRemainingTime
    pub fn set_item_in_use_ticks(&self, ticks: i32) -> bool {
        put(self, "use_left", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getHandRaised, org/bukkit/entity/LivingEntity.getActiveItemHand
    pub fn get_hand_raised(&self) -> &'static str {
        if flag(self, "use_hand") { "OFF_HAND" } else { "HAND" }
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getActiveItem, org/bukkit/entity/LivingEntity.getItemInUse
    pub fn get_active_item(&self) -> Option<ItemStack> {
        if !self.has_active_item() {
            return None;
        }
        self.get_equipment().get_item(self.get_hand_raised())
    }
    /// bukkit: org/bukkit/entity/LivingEntity.isSwimming
    ///
    /// Only a drowned keeps it: on anything else `updateSwimming` takes it back next tick, on Paper too.
    pub fn is_swimming(&self) -> bool {
        flag(self, "swimming")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setSwimming
    ///
    /// Only a drowned keeps it: on anything else `updateSwimming` takes it back next tick, on Paper too.
    pub fn set_swimming(&self, swimming: bool) -> bool {
        put(self, "swimming", json!(swimming))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.isJumping
    pub fn is_jumping(&self) -> bool {
        flag(self, "jumping")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setJumping
    pub fn set_jumping(&self, jumping: bool) -> bool {
        put(self, "jumping", json!(jumping))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getNoActionTicks
    pub fn get_no_action_ticks(&self) -> i32 {
        number(self, "no_action")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setNoActionTicks
    pub fn set_no_action_ticks(&self, ticks: i32) -> bool {
        put(self, "no_action", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getBodyYaw
    pub fn get_body_yaw(&self) -> f32 {
        get(self, "body_yaw").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setBodyYaw
    pub fn set_body_yaw(&self, yaw: f32) -> bool {
        put(self, "body_yaw", json!(yaw))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getForwardsMovement
    pub fn get_forwards_movement(&self) -> f32 {
        get(self, "forwards").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getSidewaysMovement
    pub fn get_sideways_movement(&self) -> f32 {
        get(self, "sideways").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getUpwardsMovement
    pub fn get_upwards_movement(&self) -> f32 {
        get(self, "upwards").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.isClimbing
    pub fn is_climbing(&self) -> bool {
        flag(self, "climbing")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.knockback
    pub fn knockback(&self, strength: f64, x: f64, z: f64) -> bool {
        done(self, "knockback", json!({ "value": [strength, x, z] }))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.attack
    pub fn attack(&self, target_entity: &Entity) -> bool {
        done(self, "attack", target(target_entity))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.hasLineOfSight
    pub fn has_line_of_sight(&self, other: &Entity) -> bool {
        done(self, "sees", target(other))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.playHurtAnimation
    pub fn play_hurt_animation(&self, yaw: f32) -> bool {
        common(self, "hurt_animation", json!({ "yaw": yaw })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.playPickupItemAnimation
    pub fn play_pickup_item_animation(&self, item: &Entity, amount: i32) -> bool {
        common(self, "take_item", json!({ "item": item.get_entity_id(), "amount": amount })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.broadcastSlotBreak
    pub fn broadcast_slot_break(&self, slot: &str) -> bool {
        let event = match slot {
            "HAND" => 47,
            "OFF_HAND" => 48,
            "HEAD" => 49,
            "CHEST" => 50,
            "LEGS" => 51,
            "FEET" => 52,
            _ => return false,
        };
        common(self, "event", json!({ "event": event })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getSoundVolume
    pub fn get_sound_volume(&self) -> f32 {
        get(self, "sound_volume").as_f64().unwrap_or(1.0) as f32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getHurtSound
    pub fn get_hurt_sound(&self) -> Option<String> {
        get(self, "voice")[1].as_str().map(|sound| format!("minecraft:{sound}"))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getDeathSound
    pub fn get_death_sound(&self) -> Option<String> {
        get(self, "voice")[2].as_str().map(|sound| format!("minecraft:{sound}"))
    }
    fn fall_sound(&self, size: &str) -> String {
        let kind = if self.get_spawn_category() == "MONSTER" { "hostile" } else { "generic" };
        format!("minecraft:entity.{kind}.{size}_fall")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getFallDamageSoundSmall
    pub fn get_fall_damage_sound_small(&self) -> String {
        self.fall_sound("small")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getFallDamageSoundBig
    pub fn get_fall_damage_sound_big(&self) -> String {
        self.fall_sound("big")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getFallDamageSound
    pub fn get_fall_damage_sound(&self, height: i32) -> String {
        self.fall_sound(if height > 4 { "big" } else { "small" })
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getEatingSound
    pub fn get_eating_sound(&self, stack: &ItemStack) -> String {
        let Some(food) = stack.get_data("minecraft:consumable") else {
            return "minecraft:entity.generic.drink".to_owned();
        };
        if is_type(self, &["minecraft:wandering_trader"]) {
            let milk = stack.get_type().key() == "minecraft:milk_bucket";
            return if milk { "minecraft:entity.wandering_trader.drink_milk" } else { "minecraft:entity.wandering_trader.drink_potion" }.to_owned();
        }
        food["sound"].as_str().or_else(|| food["sound"]["sound_id"].as_str()).unwrap_or("minecraft:entity.generic.eat").to_owned()
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getDrinkingSound
    pub fn get_drinking_sound(&self, stack: &ItemStack) -> String {
        self.get_eating_sound(stack)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.canBreatheUnderwater
    pub fn can_breathe_underwater(&self) -> bool {
        let tagged = query(json!({ "op": "tag", "kind": "entity_type", "name": "minecraft:can_breathe_under_water" }));
        tagged.as_array().is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(self.get_type().as_str()))) || self.has_potion_effect("minecraft:water_breathing")
    }
    fn in_tag(&self, tag: &str) -> bool {
        query(json!({ "op": "tag", "kind": "entity_type", "name": tag })).as_array().is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(self.get_type().as_str())))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getCategory
    pub fn get_category(&self) -> &'static str {
        if self.in_tag("minecraft:undead") {
            "UNDEAD"
        } else if self.in_tag("minecraft:arthropod") {
            "ARTHROPOD"
        } else if self.in_tag("minecraft:illager") {
            "ILLAGER"
        } else if self.in_tag("minecraft:aquatic") {
            "WATER"
        } else {
            "NONE"
        }
    }
    /// bukkit: org/bukkit/entity/LivingEntity.canUseEquipmentSlot
    pub fn can_use_equipment_slot(&self, slot: &str) -> bool {
        match slot {
            "BODY" => is_type(self, &["minecraft:wolf", "minecraft:horse", "minecraft:llama", "minecraft:trader_llama", "minecraft:happy_ghast", "minecraft:nautilus", "minecraft:zombie_nautilus", "minecraft:sulfur_cube"]),
            "SADDLE" => is_type(self, &["minecraft:horse", "minecraft:donkey", "minecraft:mule", "minecraft:skeleton_horse", "minecraft:zombie_horse", "minecraft:camel", "minecraft:camel_husk", "minecraft:pig", "minecraft:strider", "minecraft:nautilus", "minecraft:zombie_nautilus"]),
            _ => !is_type(self, &["minecraft:armor_stand"]) || !matches!(slot, "BODY" | "SADDLE"),
        }
    }
    /// bukkit: org/bukkit/entity/LivingEntity.damageItemStack
    pub fn damage_item_stack(&self, stack: &ItemStack, amount: i32) -> Option<ItemStack> {
        let mut out = stack.clone();
        let most = stack.get_type().get_max_durability();
        if most <= 0 {
            return Some(out);
        }
        let damage = out.get_component("minecraft:damage").and_then(Value::as_i64).unwrap_or(0) + i64::from(amount);
        if damage >= i64::from(most) {
            return None;
        }
        out.set_component("minecraft:damage", json!(damage));
        Some(out)
    }
    fn eyes(&self) -> Option<(Location, Vector)> {
        let at = self.get_location()?;
        let (yaw, pitch) = (f64::from(at.get_yaw()).to_radians(), f64::from(at.get_pitch()).to_radians());
        let eyes = Location::new(self.get_world(), at.x, at.y + self.get_eye_height(), at.z, 0.0, 0.0);
        Some((eyes, Vector::new(-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos())))
    }
    /// The blocks along its look, nearest first.
    fn blocks_in_view(&self, range: i32) -> Vec<crate::Block> {
        let Some((eyes, way)) = self.eyes() else {
            return Vec::new();
        };
        let mut seen: Vec<crate::Block> = Vec::new();
        for step in 0..=(range * 4) {
            let along = f64::from(step) * 0.25;
            let at = Location::new(self.get_world(), eyes.x + way.get_x() * along, eyes.y + way.get_y() * along, eyes.z + way.get_z() * along, 0.0, 0.0);
            let block = at.get_block();
            if seen.last() != Some(&block) {
                seen.push(block);
            }
        }
        seen
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getLineOfSight
    pub fn get_line_of_sight(&self, transparent: &[crate::Material], range: i32) -> Vec<crate::Block> {
        let air = |block: &crate::Block| block.get_type().is_air() || transparent.contains(&block.get_type());
        let mut line = Vec::new();
        for block in self.blocks_in_view(range) {
            let stop = !air(&block);
            line.push(block);
            if stop {
                break;
            }
        }
        line
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getLastTwoTargetBlocks
    pub fn get_last_two_target_blocks(&self, transparent: &[crate::Material], range: i32) -> Vec<crate::Block> {
        let line = self.get_line_of_sight(transparent, range);
        let from = line.len().saturating_sub(2);
        line[from..].to_vec()
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getTargetBlockInfo, org/bukkit/entity/LivingEntity.getTargetBlockFace
    pub fn get_target_block_info(&self, range: i32) -> Option<crate::fam_paper::TargetBlockInfo> {
        let line = self.get_line_of_sight(&[], range);
        let hit = line.last().filter(|block| !block.get_type().is_air())?;
        let before = line.len().checked_sub(2).and_then(|index| line.get(index));
        let face = before.map_or("SELF", |from| {
            let (a, b) = (from.get_location(), hit.get_location());
            match (a.x - b.x, a.y - b.y, a.z - b.z) {
                (x, _, _) if x > 0.0 => "EAST",
                (x, _, _) if x < 0.0 => "WEST",
                (_, y, _) if y > 0.0 => "UP",
                (_, y, _) if y < 0.0 => "DOWN",
                (_, _, z) if z > 0.0 => "SOUTH",
                (_, _, z) if z < 0.0 => "NORTH",
                _ => "SELF",
            }
        });
        Some(crate::fam_paper::TargetBlockInfo::new(hit, face))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getTargetEntity, org/bukkit/entity/LivingEntity.getTargetEntityInfo
    pub fn get_target_entity(&self, range: i32) -> Option<TargetEntityInfo> {
        let (eyes, way) = self.eyes()?;
        let wall = self.get_line_of_sight(&[], range).last().filter(|block| !block.get_type().is_air()).map(|block| block.get_location());
        let reach = wall.map_or(f64::from(range), |block| ((block.x + 0.5 - eyes.x).powi(2) + (block.y + 0.5 - eyes.y).powi(2) + (block.z + 0.5 - eyes.z).powi(2)).sqrt());
        let mut best: Option<(f64, Entity, Vector)> = None;
        for other in self.get_world().get_entities() {
            if other.get_entity_id() == self.get_entity_id() {
                continue;
            }
            let (Some(at), half, tall) = (other.get_location(), other.get_width() / 2.0, other.get_height()) else {
                continue;
            };
            for step in 0..=((reach * 8.0) as i32) {
                let along = f64::from(step) * 0.125;
                let (x, y, z) = (eyes.x + way.get_x() * along, eyes.y + way.get_y() * along, eyes.z + way.get_z() * along);
                if (x - at.x).abs() <= half && (z - at.z).abs() <= half && y >= at.y && y <= at.y + tall {
                    if best.as_ref().is_none_or(|(known, _, _)| along < *known) {
                        best = Some((along, other.clone(), Vector::new(x, y, z)));
                    }
                    break;
                }
            }
        }
        best.map(|(_, entity, hit)| TargetEntityInfo::new(entity, hit))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.rayTraceEntities
    pub fn ray_trace_entities(&self, range: i32) -> Option<TargetEntityInfo> {
        self.get_target_entity(range)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getMemory
    pub fn get_memory(&self, key: &MemoryKey) -> Option<MemoryValue> {
        let saved = self.data_at("Brain.memories").get(key.get_key())?["value"].clone();
        MemoryValue::read(&saved, &key.get_key(), &self.get_world())
    }
}

fn get_uuid(entity: &Entity, key: &str) -> Option<String> {
    get(entity, key).as_str().map(str::to_owned)
}

impl Entity {
    /// bukkit: org/bukkit/entity/Damageable.setMaxHealth
    pub fn set_max_health(&self, health: f64) -> bool {
        put(self, "max_health", json!(health))
    }
    /// bukkit: org/bukkit/entity/Damageable.resetMaxHealth
    pub fn reset_max_health(&self) -> bool {
        put(self, "max_health", Value::Null)
    }
}

impl Mob {
    /// bukkit: org/bukkit/entity/Mob.isAggressive
    pub fn is_aggressive(&self) -> bool {
        flag(&self.0, "aggressive")
    }
    /// bukkit: org/bukkit/entity/Mob.setAggressive
    pub fn set_aggressive(&self, aggressive: bool) -> bool {
        put(&self.0, "aggressive", json!(aggressive))
    }
    /// bukkit: org/bukkit/entity/Mob.lookAt
    pub fn look_at(&self, at: &Location) -> bool {
        done(&self.0, "look_control", json!({ "at": [at.x, at.y, at.z] }))
    }
    /// bukkit: org/bukkit/entity/Mob.getMaxHeadPitch
    pub fn get_max_head_pitch(&self) -> i32 {
        number(&self.0, "max_head_pitch")
    }
    /// bukkit: org/bukkit/entity/Mob.getHeadRotationSpeed
    pub fn get_head_rotation_speed(&self) -> i32 {
        number(&self.0, "head_rotation")
    }
    /// bukkit: org/bukkit/entity/Mob.getAmbientSound
    pub fn get_ambient_sound(&self) -> Option<String> {
        get(&self.0, "voice")[0].as_str().map(|sound| format!("minecraft:{sound}"))
    }
    /// bukkit: org/bukkit/entity/Mob.isInDaylight
    pub fn is_in_daylight(&self) -> bool {
        done(&self.0, "daylight", json!({}))
    }
    /// bukkit: org/bukkit/entity/Mob.getPossibleExperienceReward
    pub fn get_possible_experience_reward(&self) -> i32 {
        act(&self.0, "experience", json!({})).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Mob.shouldDespawnInPeaceful
    pub fn should_despawn_in_peaceful(&self) -> bool {
        plugin_get(&self.0, "peaceful").as_f64().map_or_else(|| self.get_spawn_category() == "MONSTER", |on| on != 0.0)
    }
    /// bukkit: org/bukkit/entity/Mob.setDespawnInPeacefulOverride
    pub fn set_despawn_in_peaceful_override(&self, state: TriState) -> bool {
        plugin_set(&self.0, "peaceful", state.json())
    }
    /// bukkit: org/bukkit/entity/Mob.getDespawnInPeacefulOverride
    pub fn get_despawn_in_peaceful_override(&self) -> TriState {
        TriState::read(&plugin_get(&self.0, "peaceful"))
    }
}

// ---- Third pass ----

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getNoDamageTicks
    pub fn get_no_damage_ticks(&self) -> i32 {
        number(self, "invulnerable_ticks")
    }
    /// The damage it took last, live: what `get_last_damage` reads.
    pub(crate) fn last_hurt(&self) -> f64 {
        get(self, "last_hurt").as_f64().unwrap_or(0.0)
    }
}

impl Zombie {
    /// bukkit: org/bukkit/entity/Zombie.getConversionTime
    pub fn get_conversion_time(&self) -> i32 {
        number(&self.0, "zombie_conversion")
    }
    /// bukkit: org/bukkit/entity/Zombie.setConversionTime
    pub fn set_conversion_time(&self, time: i32) -> bool {
        put(&self.0, "zombie_conversion", json!(time))
    }
}

impl Husk {
    /// bukkit: org/bukkit/entity/Husk.getConversionTime
    pub fn get_conversion_time(&self) -> i32 {
        number(&self.0, "zombie_conversion")
    }
    /// bukkit: org/bukkit/entity/Husk.setConversionTime
    pub fn set_conversion_time(&self, time: i32) -> bool {
        put(&self.0, "zombie_conversion", json!(time))
    }
}

/// Whether a zombie's drowning has begun, for `Zombie.isConverting`.
pub(crate) fn zombie_converting(zombie: &Entity) -> bool {
    flag(zombie, "zombie_drowning")
}

impl Minecart {
    /// bukkit: org/bukkit/entity/Minecart.getMaxSpeed
    pub fn get_max_speed(&self) -> f64 {
        plugin_get(&self.0, "max_speed").as_f64().unwrap_or(0.4)
    }
    /// bukkit: org/bukkit/entity/Minecart.setMaxSpeed
    pub fn set_max_speed(&self, speed: f64) -> bool {
        speed >= 0.0 && plugin_set(&self.0, "max_speed", json!(speed))
    }
    /// bukkit: org/bukkit/entity/Minecart.isSlowWhenEmpty
    pub fn is_slow_when_empty(&self) -> bool {
        plugin_get(&self.0, "slow_when_empty").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/Minecart.setSlowWhenEmpty
    pub fn set_slow_when_empty(&self, slow: bool) -> bool {
        plugin_set(&self.0, "slow_when_empty", json!(slow))
    }
    fn velocity_mod(&self, key: &str, default: f64) -> Vector {
        let saved = plugin_get(&self.0, key);
        let at = |i: usize| saved[i].as_f64().unwrap_or(default);
        Vector::new(at(0), at(1), at(2))
    }
    /// bukkit: org/bukkit/entity/Minecart.getFlyingVelocityMod
    pub fn get_flying_velocity_mod(&self) -> Vector {
        self.velocity_mod("flying_mod", 0.95)
    }
    /// bukkit: org/bukkit/entity/Minecart.setFlyingVelocityMod
    pub fn set_flying_velocity_mod(&self, flying: &Vector) -> bool {
        plugin_set(&self.0, "flying_mod", vector_json(flying))
    }
    /// bukkit: org/bukkit/entity/Minecart.getDerailedVelocityMod
    pub fn get_derailed_velocity_mod(&self) -> Vector {
        self.velocity_mod("derailed_mod", 0.5)
    }
    /// bukkit: org/bukkit/entity/Minecart.setDerailedVelocityMod
    pub fn set_derailed_velocity_mod(&self, derailed: &Vector) -> bool {
        plugin_set(&self.0, "derailed_mod", vector_json(derailed))
    }
    /// bukkit: org/bukkit/entity/Minecart.setDisplayBlockData, org/bukkit/entity/Minecart.setDisplayBlock
    pub fn set_display_block_data(&self, data: Option<&crate::block_data::BlockData>) -> bool {
        match data {
            Some(data) => self.merge(&format!("{{DisplayState:{}}}", crate::entities2::block_state_snbt(data))),
            None => self.merge("{DisplayState:{Name:\"minecraft:air\"}}"),
        }
    }
}

impl FishHook {
    /// bukkit: org/bukkit/entity/FishHook.getMinWaitTime
    pub fn get_min_wait_time(&self) -> i32 {
        self.rules()["min_wait"].as_i64().unwrap_or(100) as i32
    }
    /// bukkit: org/bukkit/entity/FishHook.getMaxWaitTime
    pub fn get_max_wait_time(&self) -> i32 {
        self.rules()["max_wait"].as_i64().unwrap_or(600) as i32
    }
    /// bukkit: org/bukkit/entity/FishHook.getMinLureTime
    pub fn get_min_lure_time(&self) -> i32 {
        self.rules()["min_lure"].as_i64().unwrap_or(20) as i32
    }
    /// bukkit: org/bukkit/entity/FishHook.getMaxLureTime
    pub fn get_max_lure_time(&self) -> i32 {
        self.rules()["max_lure"].as_i64().unwrap_or(80) as i32
    }
    /// bukkit: org/bukkit/entity/FishHook.getMinLureAngle
    pub fn get_min_lure_angle(&self) -> f32 {
        self.rules()["min_angle"].as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/FishHook.getMaxLureAngle
    pub fn get_max_lure_angle(&self) -> f32 {
        self.rules()["max_angle"].as_f64().unwrap_or(360.0) as f32
    }
    /// bukkit: org/bukkit/entity/FishHook.isSkyInfluenced
    pub fn is_sky_influenced(&self) -> bool {
        self.rules()["sky"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/entity/FishHook.isRainInfluenced
    pub fn is_rain_influenced(&self) -> bool {
        self.rules()["rain"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/entity/FishHook.getApplyLure
    pub fn get_apply_lure(&self) -> bool {
        self.rules()["apply_lure"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/entity/FishHook.retrieve
    ///
    /// `hand` is `HAND` or `OFF_HAND`. What it comes to is the durability the rod would lose: nothing is taken here.
    pub fn retrieve(&self, hand: &str) -> i32 {
        query(json!({ "op": "entity_player_turn", "uuid": self.0.uuid(), "key": "reel", "hand": hand })).as_i64().unwrap_or(0) as i32
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/Player.getResourcePackHash
    pub fn get_resource_pack_hash(&self) -> Option<String> {
        noted(self.uuid(), |note| note.pack_hash.clone())
    }
    /// bukkit~: org/bukkit/entity/Player.getHAProxyAddress | `None`, always: Horizon's server takes no HAProxy header
    pub fn get_ha_proxy_address(&self) -> Option<std::net::SocketAddr> {
        None
    }
    /// bukkit: org/bukkit/entity/Player.sendMap
    pub fn send_map(&self, map: &crate::fam_core::MapView) -> bool {
        send_player(self, "map", json!({ "id": map.get_id() }))
    }
    /// bukkit: org/bukkit/entity/Player.setWorldBorder
    ///
    /// A change of level sends that level's border first, as `PlayerList.sendLevelInfo` does.
    pub fn set_world_border(&self, border: Option<&crate::world::WorldBorder>) -> bool {
        let world = self.get_world().get_world_border();
        let shown = border.unwrap_or(&world);
        let center = shown.get_center();
        noted(self.uuid(), |note| note.border = border.cloned());
        self.player_set("plugin:own_border", json!(if border.is_some() { 1.0 } else { 0.0 }));
        send_player(self, "border", json!({ "x": center.x, "z": center.z, "size": shown.get_size(), "warning_time": shown.get_warning_time(), "warning_blocks": shown.get_warning_distance() }))
    }
    /// bukkit: org/bukkit/entity/Player.getWorldBorder
    pub fn get_world_border(&self) -> Option<crate::world::WorldBorder> {
        noted(self.uuid(), |note| note.border.clone())
    }
    /// bukkit: org/bukkit/entity/Player.saveData
    pub fn save_data(&self) -> bool {
        send_player(self, "save", json!({}))
    }
    fn stuck(&self) -> Value {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "stuck" }))
    }
    fn set_stuck(&self, what: &str, value: i32) -> bool {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "set_stuck", "what": what, "value": value })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getArrowsInBody, org/bukkit/entity/LivingEntity.getArrowsStuck
    pub fn get_arrows_in_body(&self) -> i32 {
        self.stuck()["arrows"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setArrowsInBody, org/bukkit/entity/LivingEntity.setArrowsStuck
    pub fn set_arrows_in_body(&self, count: i32) -> bool {
        self.set_stuck("arrows", count)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getBeeStingersInBody
    pub fn get_bee_stingers_in_body(&self) -> i32 {
        self.stuck()["stingers"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setBeeStingersInBody
    pub fn set_bee_stingers_in_body(&self, count: i32) -> bool {
        self.set_stuck("stingers", count)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.isRiptiding
    pub fn is_riptiding(&self) -> bool {
        self.player_state()["living_flags"].as_i64().unwrap_or(0) & 4 != 0
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setRiptiding
    pub fn set_riptiding(&self, riptiding: bool) -> bool {
        set_player(self, "riptide", json!(riptiding))
    }
    fn warden(&self) -> Value {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "warden" }))
    }
    fn set_warden(&self, what: &str, value: i32) -> bool {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "set_warden", "what": what, "value": value })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Player.getWardenWarningLevel
    pub fn get_warden_warning_level(&self) -> i32 {
        self.warden()["level"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.setWardenWarningLevel
    pub fn set_warden_warning_level(&self, level: i32) -> bool {
        self.set_warden("level", level)
    }
    /// bukkit: org/bukkit/entity/Player.increaseWardenWarningLevel
    pub fn increase_warden_warning_level(&self) -> bool {
        self.set_warden("level", self.get_warden_warning_level() + 1)
    }
    /// bukkit: org/bukkit/entity/Player.getWardenWarningCooldown
    pub fn get_warden_warning_cooldown(&self) -> i32 {
        self.warden()["cooldown"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.setWardenWarningCooldown
    pub fn set_warden_warning_cooldown(&self, ticks: i32) -> bool {
        self.set_warden("cooldown", ticks)
    }
    /// bukkit: org/bukkit/entity/Player.getWardenTimeSinceLastWarning
    pub fn get_warden_time_since_last_warning(&self) -> i32 {
        self.warden()["since"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.setWardenTimeSinceLastWarning
    pub fn set_warden_time_since_last_warning(&self, ticks: i32) -> bool {
        self.set_warden("since", ticks)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.sleep
    ///
    /// False where there's no bed. `force` lies down past the night, the distance, what's above and any monsters.
    pub fn sleep(&self, at: &Location, force: bool) -> bool {
        if !at.get_block().get_type().key().ends_with("_bed") {
            return false;
        }
        if force {
            return query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "sleep_force", "pos": block_of(at) })).as_bool().unwrap_or(false);
        }
        send_player(self, "sleep", json!({ "pos": block_of(at) }))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getWaypointStyle
    pub fn get_waypoint_style(&self) -> Option<String> {
        noted(self.uuid(), |note| note.waypoint_style.clone())
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setWaypointStyle
    pub fn set_waypoint_style(&self, style: Option<&str>) {
        noted(self.uuid(), |note| note.waypoint_style = style.map(str::to_owned));
        let change = style.map_or("style reset".to_owned(), |style| format!("style set {style}"));
        horizon_mod::run_command(&format!("waypoint modify {} {change}", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getWaypointColor
    pub fn get_waypoint_color(&self) -> Option<crate::Color> {
        noted(self.uuid(), |note| note.waypoint_color)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setWaypointColor
    pub fn set_waypoint_color(&self, color: Option<crate::Color>) {
        noted(self.uuid(), |note| note.waypoint_color = color);
        let change = color.map_or("color reset".to_owned(), |color| format!("color hex {:06X}", color.as_rgb()));
        horizon_mod::run_command(&format!("waypoint modify {} {change}", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/Player.activeBossBars
    pub fn active_boss_bars(&self) -> Vec<crate::BossBar> {
        crate::fam_core::get_boss_bars().into_iter().filter(|bar| bar.get_players().iter().any(|player| player == self)).collect()
    }
}

impl LightningStrike {
    /// bukkit: org/bukkit/entity/LightningStrike.spigot
    pub fn spigot(&self) -> &Self {
        self
    }
}

impl Mannequin {
    /// bukkit: org/bukkit/entity/Mannequin.defaultProfile
    pub fn default_profile() -> crate::components::ResolvableProfile {
        crate::components::ResolvableProfile(json!({}))
    }
    /// bukkit: org/bukkit/entity/Mannequin.getSkinParts
    pub fn get_skin_parts(&self) -> crate::fam_paper::SkinParts {
        crate::fam_paper::SkinParts(!number(&self.0, "hidden_parts") & 0x7F)
    }
    /// bukkit: org/bukkit/entity/Mannequin.setSkinParts
    pub fn set_skin_parts(&self, parts: crate::fam_paper::SkinParts) -> bool {
        put(&self.0, "hidden_parts", json!(!parts.0 & 0x7F))
    }
}

impl Trident {
    /// bukkit: org/bukkit/entity/Trident.setLoyaltyLevel
    pub fn set_loyalty_level(&self, level: i32) -> bool {
        (0..=127).contains(&level) && object_done(&self.0, "trident_loyalty", json!({ "value": level }))
    }
}

impl Projectile {
    /// bukkit: org/bukkit/entity/Projectile.hitEntity
    ///
    /// The hit lands at the entity's own position, as `new EntityHitResult(entity)` has it.
    pub fn hit_entity(&self, target_entity: &Actor) -> bool {
        let at = match target_entity {
            Actor::Player(player) => Some(player.get_location()),
            Actor::Entity(entity) => entity.get_location(),
        };
        let mut fields = who(target_entity);
        fields["from"] = at.map_or(Value::Null, |at| json!([at.x, at.y, at.z]));
        object_done(&self.0, "hit", fields)
    }
    /// bukkit: org/bukkit/entity/Projectile.hitEntity
    pub fn hit_entity_from(&self, target_entity: &Actor, from: &Vector) -> bool {
        let mut fields = who(target_entity);
        fields["from"] = json!([from.x, from.y, from.z]);
        object_done(&self.0, "hit", fields)
    }
    /// bukkit: org/bukkit/entity/Projectile.canHitEntity
    pub fn can_hit_entity(&self, other: &Actor) -> bool {
        let owner = self.get_owner_unique_id();
        let shooter = owner.as_deref().and_then(online);
        let world = self.get_world();
        // As an `Entity`, for what asks a vehicle or the physics.
        let body = match other {
            Actor::Player(player) => Entity::of(player.get_entity_id() as u64, &world),
            Actor::Entity(entity) => entity.clone(),
        };
        let (uuid, kind, alive) = match other {
            Actor::Player(player) => (player.get_unique_id(), "minecraft:player".to_owned(), !player.is_dead() && player.get_game_mode() != "SPECTATOR"),
            Actor::Entity(entity) => (entity.get_unique_id(), entity.get_type(), entity.is_valid()),
        };
        let name = kind.trim_start_matches("minecraft:");
        // `Entity.canBeHitByProjectile`: alive and `isPickable`, class by class.
        let pickable = match other {
            Actor::Player(player) => player.is_collidable(),
            Actor::Entity(entity) if living(entity) => name != "ender_dragon" && entity.is_collidable() && !crate::data::flag(&entity.data_at("Marker")),
            Actor::Entity(_) => {
                matches!(name, "end_crystal" | "falling_block" | "tnt" | "shulker_bullet" | "item_frame" | "glow_item_frame" | "painting" | "leash_knot" | "fireball" | "wind_charge" | "breeze_wind_charge")
                    || name.ends_with("minecart")
                    || name.ends_with("_boat")
                    || name.ends_with("_raft")
            }
        };
        // `Projectile.canHitEntity`.
        let seen = shooter.as_ref().is_none_or(|shooter| match other {
            Actor::Player(player) => shooter.can_see(player),
            Actor::Entity(entity) => sees(shooter, entity.get_entity_id()),
        });
        let root = |entity: Entity| {
            let mut top = entity;
            while let Some(vehicle) = top.get_vehicle() {
                top = vehicle;
            }
            top.get_entity_id()
        };
        let owner_body = owner.as_deref().and_then(|uuid| match online(uuid) {
            Some(player) => Some(Entity::of(player.get_entity_id() as u64, &world)),
            None => world.get_entity(uuid),
        });
        let beside_owner = !self.has_left_shooter() && (owner.as_deref() == Some(uuid.as_str()) || owner_body.is_some_and(|owner| root(owner) == root(body.clone())));
        let base = alive && pickable && seen && !beside_owner;
        // Each kind's own `canHitEntity` on top.
        match self.get_type().trim_start_matches("minecraft:") {
            "fishing_bobber" => base || (name == "item" && alive),
            "shulker_bullet" | "fireball" | "small_fireball" | "dragon_fireball" | "wither_skull" => base && !body.has_no_physics(),
            "wind_charge" | "breeze_wind_charge" => base && !body.has_no_physics() && !matches!(name, "wind_charge" | "breeze_wind_charge" | "end_crystal"),
            "arrow" | "spectral_arrow" | "trident" => {
                let friends = match (&shooter, other) {
                    (Some(shooter), Actor::Player(player)) => {
                        let board = crate::scoreboard::get_main_scoreboard();
                        board.get_entry_team(shooter.get_name()).is_some_and(|mine| board.get_entry_team(player.get_name()).is_some_and(|theirs| mine == theirs && !mine.allow_friendly_fire()))
                    }
                    _ => false,
                };
                let pierced = object(&self.0, "projectile", json!({}))["pierced"].as_array().is_some_and(|list| list.iter().any(|id| id.as_u64() == Some(body.get_entity_id())));
                base && !friends && !pierced
            }
            _ => base,
        }
    }
}

impl ThrownPotion {
    /// bukkit: org/bukkit/entity/ThrownPotion.splash
    pub fn splash(&self) -> bool {
        object_done(&self.0, "splash", json!({}))
    }
}

impl Raider {
    /// bukkit: org/bukkit/entity/Raider.getRaid
    pub fn get_raid(&self) -> Option<crate::fam_core::Raid> {
        self.get_world().get_raid(get(&self.0, "raider_raid").as_i64()? as i32)
    }
}

impl EnderDragon {
    /// bukkit: org/bukkit/entity/EnderDragon.getDragonBattle
    pub fn get_dragon_battle(&self) -> Option<crate::fam_core::DragonBattle> {
        self.get_world().get_ender_dragon_battle()
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/HumanEntity.canCriticalAttack
    pub fn can_critical_attack(&self, other: &Entity) -> bool {
        const CLIMBED: [&str; 9] = ["ladder", "vine", "scaffolding", "weeping_vines", "weeping_vines_plant", "twisting_vines", "twisting_vines_plant", "cave_vines", "cave_vines_plant"];
        let feet = self.get_location().get_block();
        let name = feet.get_type().key().trim_start_matches("minecraft:").to_owned();
        let trapdoor = name.ends_with("_trapdoor") && feet.get_block_data().contains("open=true") && feet.get_relative(0, -1, 0).get_type().key() == "minecraft:ladder";
        !CLIMBED.contains(&name.as_str()) && !trapdoor && living(other) && query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "crit" })).as_bool().unwrap_or(false)
    }
}

/// `Firework.detonate`: the fuse runs out at its next tick.
pub(crate) fn firework_detonate(rocket: &Entity) -> bool {
    object_done(rocket, "rocket_detonate", json!({}))
}

// ---- Held past the game's own tick, and what the hooks note ----

fn hold(entity: &Entity, key: &str, value: Value) -> Value {
    query(json!({ "op": "entity_hold", "key": key, "id": entity.get_entity_id(), "dimension": entity.get_world().dimension(), "value": value }))
}

impl Allay {
    /// bukkit: org/bukkit/entity/Allay.startDancing
    pub fn start_dancing(&self) -> bool {
        hold(&self.0, "dance", json!(true)).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Allay.stopDancing
    pub fn stop_dancing(&self) -> bool {
        hold(&self.0, "dance", json!(false)).as_bool().unwrap_or(false)
    }
}

impl Bee {
    /// bukkit: org/bukkit/entity/Bee.getRollingOverride
    pub fn get_rolling_override(&self) -> TriState {
        match hold(&self.0, "roll_get", Value::Null).as_bool() {
            Some(true) => TriState::True,
            Some(false) => TriState::False,
            None => TriState::NotSet,
        }
    }
    /// bukkit: org/bukkit/entity/Bee.setRollingOverride
    pub fn set_rolling_override(&self, rolling: TriState) -> bool {
        let value = match rolling {
            TriState::True => json!(true),
            TriState::False => json!(false),
            TriState::NotSet => Value::Null,
        };
        hold(&self.0, "roll", value).as_bool().unwrap_or(false)
    }
}

impl Creeper {
    /// bukkit: org/bukkit/entity/Creeper.setMaxFuseTicks
    pub fn set_max_fuse_ticks(&self, ticks: i32) -> bool {
        self.merge(&format!("{{Fuse:{ticks}s}}")) && hold(&self.0, "fuse", json!(ticks)).as_bool().unwrap_or(false)
    }
}

impl ArmorStand {
    /// bukkit: org/bukkit/entity/ArmorStand.canMove
    pub fn can_move(&self) -> bool {
        hold(&self.0, "stand_get", Value::Null).as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setCanMove
    pub fn set_can_move(&self, can: bool) -> bool {
        hold(&self.0, "stand", json!(can)).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.canTick
    ///
    /// An armor stand's only tick in Horizon is its motion.
    pub fn can_tick(&self) -> bool {
        plugin_get(&self.0, "can_tick").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/ArmorStand.setCanTick
    pub fn set_can_tick(&self, can: bool) -> bool {
        plugin_set(&self.0, "can_tick", json!(if can { 1.0 } else { 0.0 })) && hold(&self.0, "stand_tick", json!(can)).as_bool().unwrap_or(false)
    }
}

/// How an entity came and how it went, as far as the hooks saw.
#[derive(Clone, Debug)]
struct Birth {
    reason: &'static str,
    at: [f64; 3],
    level: String,
}

static BIRTHS: std::sync::Mutex<std::collections::BTreeMap<u64, Birth>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
static DEATHS: std::sync::Mutex<std::collections::BTreeSet<u64>> = std::sync::Mutex::new(std::collections::BTreeSet::new());
static LAST_REASON: std::sync::Mutex<&'static str> = std::sync::Mutex::new("DEFAULT");
/// Born since the last tick's end, for the game to save with them.
static NEW_BIRTHS: std::sync::Mutex<Vec<Value>> = std::sync::Mutex::new(Vec::new());
/// Those that went to another level.
static LEFT: std::sync::Mutex<std::collections::BTreeSet<u64>> = std::sync::Mutex::new(std::collections::BTreeSet::new());
static NO_PORTALS: std::sync::Mutex<std::collections::BTreeSet<u64>> = std::sync::Mutex::new(std::collections::BTreeSet::new());

/// Past this many remembered entities the oldest ids go: an entity's id only grows.
const REMEMBERED: usize = 100_000;

pub(crate) fn install(host: horizon_mod::Host) {
    use horizon_mod::Answer;
    host.on("entity_after_mobs", |_| {
        let births = std::mem::take(&mut *NEW_BIRTHS.lock().unwrap_or_else(|e| e.into_inner()));
        if !births.is_empty() {
            query(json!({ "op": "entity_births", "rows": births }));
        }
        Answer::Pass
    });
    host.on("entity_change_level", |value| {
        if let Some(id) = value["entity"].as_u64() {
            let mut gone = LEFT.lock().unwrap_or_else(|e| e.into_inner());
            gone.insert(id);
            while gone.len() > REMEMBERED {
                gone.pop_first();
            }
        }
        Answer::Pass
    });
    for (hook, reason) in [("why_natural", "NATURAL"), ("why_egg", "SPAWNER_EGG"), ("why_breed", "BREEDING"), ("why_spawner", "SPAWNER"), ("why_golem", "VILLAGE_DEFENSE"), ("why_raid", "RAID"), ("why_phantom", "NATURAL"), ("why_convert", "CONVERSION"), ("why_trader", "NATURAL")] {
        host.on(hook, move |_| {
            *LAST_REASON.lock().unwrap_or_else(|e| e.into_inner()) = reason;
            Answer::Pass
        });
    }
    host.on("entity_load", |value| {
        if value["from_disk"].as_bool() == Some(false)
            && let Some(id) = value["entity"].as_u64()
        {
            let at = [0, 1, 2].map(|i| value["pos"][i].as_f64().unwrap_or(0.0));
            let reason = *LAST_REASON.lock().unwrap_or_else(|e| e.into_inner());
            let mut all = BIRTHS.lock().unwrap_or_else(|e| e.into_inner());
            let level = value["level"].as_str().unwrap_or_default().to_owned();
            all.insert(id, Birth { reason, at, level: level.clone() });
            while all.len() > REMEMBERED {
                all.pop_first();
            }
            NEW_BIRTHS.lock().unwrap_or_else(|e| e.into_inner()).push(json!([id, level, at, reason]));
        }
        Answer::Pass
    });
    host.on("mob_death", |value| {
        if let Some(id) = value["entity"].as_u64() {
            let mut all = DEATHS.lock().unwrap_or_else(|e| e.into_inner());
            all.insert(id);
            while all.len() > REMEMBERED {
                all.pop_first();
            }
        }
        Answer::Pass
    });
    host.on("pl_game_mode", |value| {
        if let Some(uuid) = value["uuid"].as_str()
            && let Some(player) = crate::get_online_players().into_iter().find(|player| player.uuid() == uuid)
        {
            let mode = ["SURVIVAL", "CREATIVE", "ADVENTURE", "SPECTATOR"].iter().position(|mode| *mode == player.get_game_mode());
            player.player_set("plugin:previous_mode", json!(mode));
        }
        Answer::Pass
    });
    host.on("entity_tnt", |_| Answer::Pass);
    host.on("entity_fangs", |_| Answer::Pass);
    host.on("packet_pack", |value| {
        if let (Some(uuid), Some(action)) = (value["uuid"].as_str(), value["action"].as_u64()) {
            PACK_STATUS.lock().unwrap_or_else(|e| e.into_inner()).insert(uuid.to_owned(), action);
        }
        Answer::Pass
    });
    host.on("entity_portal_gate", |value| {
        if value["entity"].as_u64().is_some_and(|id| NO_PORTALS.lock().unwrap_or_else(|e| e.into_inner()).contains(&id)) { Answer::Fail } else { Answer::Pass }
    });
}

impl Entity {
    /// bukkit: org/bukkit/entity/Entity.getOrigin
    pub fn get_origin(&self) -> Option<Location> {
        // Born this tick, it isn't in the game's hands yet.
        let born = BIRTHS.lock().unwrap_or_else(|e| e.into_inner()).get(&self.get_entity_id()).cloned();
        if let Some(birth) = born {
            let world = if birth.level.is_empty() { self.get_world() } else { World::of(&birth.level) };
            return Some(Location::new(world, birth.at[0], birth.at[1], birth.at[2], 0.0, 0.0));
        }
        let saved = plugin_get(self, "origin");
        let at = |i: usize| saved[i].as_f64();
        let world = saved[3].as_str().map_or_else(|| self.get_world(), World::of);
        Some(Location::new(world, at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/entity/Entity.getEntitySpawnReason
    pub fn get_entity_spawn_reason(&self) -> String {
        let born = BIRTHS.lock().unwrap_or_else(|e| e.into_inner()).get(&self.get_entity_id()).map(|birth| birth.reason);
        plugin_get(self, "spawn_reason").as_str().map(str::to_owned).or_else(|| born.map(str::to_owned)).unwrap_or_else(|| "DEFAULT".to_owned())
    }
    /// bukkit: org/bukkit/entity/Entity.fromMobSpawner
    pub fn from_mob_spawner(&self) -> bool {
        self.get_entity_spawn_reason() == "SPAWNER"
    }
    /// bukkit: org/bukkit/entity/Entity.getRemovalReason
    pub fn get_removal_reason(&self) -> Option<&'static str> {
        if self.is_valid() {
            return None;
        }
        let id = self.get_entity_id();
        Some(if DEATHS.lock().unwrap_or_else(|e| e.into_inner()).contains(&id) {
            "KILLED"
        } else if LEFT.lock().unwrap_or_else(|e| e.into_inner()).contains(&id) {
            "CHANGED_DIMENSION"
        } else if hold(self, "unloaded", Value::Null) == json!(true) {
            "UNLOADED_TO_CHUNK"
        } else {
            "DISCARDED"
        })
    }
    /// bukkit~: org/bukkit/entity/Entity.getRemoveEventCause | death, a chunk unloading, another level, else `DISCARD`: the game's despawns, merges, pickups and hits don't say which they are where they remove
    pub fn get_remove_event_cause(&self) -> Option<&'static str> {
        self.get_removal_reason().map(|reason| match reason {
            "KILLED" => "DEATH",
            "UNLOADED_TO_CHUNK" => "UNLOAD",
            "CHANGED_DIMENSION" => "CHANGED_DIMENSION",
            _ => "DISCARD",
        })
    }
}

impl Wither {
    /// bukkit: org/bukkit/entity/Wither.canTravelThroughPortals
    pub fn can_travel_through_portals(&self) -> bool {
        !NO_PORTALS.lock().unwrap_or_else(|e| e.into_inner()).contains(&self.get_entity_id())
    }
    /// bukkit: org/bukkit/entity/Wither.setCanTravelThroughPortals
    pub fn set_can_travel_through_portals(&self, can: bool) {
        let mut all = NO_PORTALS.lock().unwrap_or_else(|e| e.into_inner());
        if can {
            all.remove(&self.get_entity_id());
        } else {
            all.insert(self.get_entity_id());
        }
    }
}

/// The entity's live box, for `get_height` and the like: a mob's, else the usual size of its type.
pub(crate) fn live_size(entity: &Entity) -> Option<(f64, f64, f64)> {
    let size = get(entity, "size");
    if let (Some(width), Some(height), Some(eyes)) = (size[0].as_f64(), size[1].as_f64(), size[2].as_f64()) {
        return Some((width, height, eyes));
    }
    let (width, height) = match entity.get_type().trim_start_matches("minecraft:") {
        "item" | "snowball" | "egg" | "ender_pearl" | "experience_bottle" | "splash_potion" | "lingering_potion" | "eye_of_ender" | "llama_spit" | "item_frame" | "glow_item_frame" => (0.25, 0.25),
        "experience_orb" | "arrow" | "spectral_arrow" | "trident" | "evoker_fangs" => (0.5, 0.5),
        "tnt" | "falling_block" => (0.98, 0.98),
        "minecart" | "chest_minecart" | "hopper_minecart" | "furnace_minecart" | "tnt_minecart" | "command_block_minecart" | "spawner_minecart" => (0.98, 0.7),
        "armor_stand" => (0.5, 1.975),
        "end_crystal" => (2.0, 2.0),
        "fireball" => (1.0, 1.0),
        "small_fireball" | "wind_charge" | "breeze_wind_charge" | "shulker_bullet" | "wither_skull" => (0.3125, 0.3125),
        "firework_rocket" => (0.25, 0.25),
        "lightning_bolt" | "marker" | "area_effect_cloud" => (0.0, 0.0),
        kind if kind.ends_with("_boat") || kind.ends_with("_raft") => (1.375, 0.5625),
        _ => return None,
    };
    Some((width, height, height * 0.85))
}

static SKY_RUN: std::sync::OnceLock<()> = std::sync::OnceLock::new();

/// A player's own clock, against the server's own updates, each tick.
fn keep_sky() {
    for player in crate::get_online_players() {
        let time = noted(player.uuid(), |note| note.time);
        if let Some((offset, relative)) = time {
            let world = player.get_world();
            let shown = if relative { world.get_time() + offset } else { offset };
            query(json!({ "op": "player_send", "uuid": player.uuid(), "packet": "player_time", "game_time": world.get_full_time(), "time": shown, "relative": relative }));
        }
    }
}

/// Everyone's view of what's hidden changed.
pub(crate) fn hidden_changed() {
    sync_unseen();
}

// ---- Fourth pass ----

impl Player {
    /// bukkit: org/bukkit/entity/Player.spigot
    pub fn spigot(&self) -> &Self {
        self
    }
    /// bukkit: org/bukkit/entity/Player$Spigot.sendMessage
    pub fn spigot_send_message(&self, text: &str) {
        self.send_message(text);
    }
    /// bukkit: org/bukkit/entity/Player.getPreviousGameMode
    pub fn get_previous_game_mode(&self) -> Option<&'static str> {
        const MODES: [&str; 4] = ["SURVIVAL", "CREATIVE", "ADVENTURE", "SPECTATOR"];
        let mode = self.player_plugin("previous_mode").as_f64().map(|mode| mode as usize);
        let saved = || query(json!({ "op": "entity_player_get", "uuid": self.uuid(), "key": "previous_mode" })).as_u64().map(|mode| mode as usize);
        MODES.get(mode.or_else(saved)?).copied()
    }
    /// bukkit: org/bukkit/entity/Player.getResourcePackStatus
    pub fn get_resource_pack_status(&self) -> Option<&'static str> {
        let action = PACK_STATUS.lock().unwrap_or_else(|e| e.into_inner()).get(self.uuid()).copied()?;
        crate::enums::player_resource_pack_status_event_status::values().get(action as usize).copied()
    }
    /// bukkit: org/bukkit/entity/Damageable.getAbsorptionAmount
    pub fn get_absorption_amount(&self) -> f64 {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "absorption" })).as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/entity/Damageable.setAbsorptionAmount
    pub fn set_absorption_amount(&self, amount: f64) -> bool {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "set_absorption", "value": amount })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getSaturatedRegenRate
    pub fn get_saturated_regen_rate(&self) -> i32 {
        self.food_rate(0, 10)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setSaturatedRegenRate
    pub fn set_saturated_regen_rate(&self, ticks: i32) -> bool {
        self.set_food_rate(0, ticks)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getUnsaturatedRegenRate
    pub fn get_unsaturated_regen_rate(&self) -> i32 {
        self.food_rate(1, 80)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setUnsaturatedRegenRate
    pub fn set_unsaturated_regen_rate(&self, ticks: i32) -> bool {
        self.set_food_rate(1, ticks)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getStarvationRate
    pub fn get_starvation_rate(&self) -> i32 {
        self.food_rate(2, 80)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setStarvationRate
    pub fn set_starvation_rate(&self, ticks: i32) -> bool {
        self.set_food_rate(2, ticks)
    }
    fn food_rate(&self, which: usize, default: i32) -> i32 {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "food_rates" }))[which].as_i64().map_or(default, |ticks| ticks as i32)
    }
    fn set_food_rate(&self, which: usize, ticks: i32) -> bool {
        let mut rates: Vec<i32> = [(0, 10), (1, 80), (2, 80)].into_iter().map(|(index, default)| self.food_rate(index, default)).collect();
        rates[which] = ticks;
        self.player_set("plugin:food_rates", json!(rates))
    }
    fn shoulder(&self, side: &str, entity: &Entity) -> bool {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "shoulder_set", "side": side, "entity": entity.get_entity_id() })).as_bool().unwrap_or(false)
    }
    fn release(&self, side: &str) -> Option<Entity> {
        let id = query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "shoulder_release", "side": side })).as_u64()?;
        Some(Entity::of(id, &self.get_world()))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setShoulderEntityLeft
    pub fn set_shoulder_entity_left(&self, entity: &Entity) -> bool {
        self.shoulder("ShoulderEntityLeft", entity)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setShoulderEntityRight
    pub fn set_shoulder_entity_right(&self, entity: &Entity) -> bool {
        self.shoulder("ShoulderEntityRight", entity)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.releaseLeftShoulderEntity
    pub fn release_left_shoulder_entity(&self) -> Option<Entity> {
        self.release("ShoulderEntityLeft")
    }
    /// bukkit: org/bukkit/entity/HumanEntity.releaseRightShoulderEntity
    pub fn release_right_shoulder_entity(&self) -> Option<Entity> {
        self.release("ShoulderEntityRight")
    }
    /// bukkit: org/bukkit/entity/HumanEntity.openMerchant
    ///
    /// Anyone already trading with it refuses this, unless `force`, which closes them out as Paper does.
    pub fn open_merchant(&self, villager: &Entity, force: bool) -> bool {
        let trader = query(json!({ "op": "items_trader", "mob": villager.get_entity_id(), "dimension": villager.get_world().dimension() }));
        if force && let Some(trader) = trader.as_str().and_then(online) {
            trader.close_inventory();
        }
        self.close_inventory();
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "merchant", "entity": villager.get_entity_id(), "force": force })).as_bool().unwrap_or(false)
    }
}


static PACK_STATUS: std::sync::Mutex<std::collections::BTreeMap<String, u64>> = std::sync::Mutex::new(std::collections::BTreeMap::new());

/// bukkit: org/bukkit/entity/Ocelot$Type.getType
pub fn ocelot_type_by_id(id: i32) -> Option<&'static str> {
    crate::enums::ocelot_type::values().get(usize::try_from(id).ok()?).copied()
}

impl LightningStrike {
    /// bukkit: org/bukkit/entity/LightningStrike$Spigot.isSilent
    pub fn is_silent_strike(&self) -> bool {
        false
    }
}

impl Mannequin {
    /// bukkit: org/bukkit/entity/Mannequin.getEquipment
    pub fn equipment(&self) -> crate::inventory::EntityEquipment {
        self.get_equipment()
    }
}

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getSoundPitch
    pub fn get_sound_pitch(&self) -> f32 {
        let base = if crate::data::flag(&self.data_at("IsBaby")) || self.data_at("Age").as_i64().unwrap_or(0) < 0 { 1.5 } else { 1.0 };
        let (a, b) = (fastrand_unit(), fastrand_unit());
        (a - b) * 0.2 + base
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setMaximumNoDamageTicks
    pub fn set_maximum_no_damage_ticks(&self, ticks: i32) -> bool {
        plugin_set(self, "no_damage", json!(ticks))
    }
}

/// A draw in `0.0..1.0`, for a voice's pitch: vanilla's is a fresh draw each time too.
fn fastrand_unit() -> f32 {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x9E37_79B9);
    let mut bits = NEXT.fetch_add(0x9E37_79B9, std::sync::atomic::Ordering::Relaxed);
    bits ^= bits << 13;
    bits ^= bits >> 17;
    bits ^= bits << 5;
    (bits >> 8) as f32 / 16_777_216.0
}

impl ShulkerBullet {
    /// bukkit: org/bukkit/entity/ShulkerBullet.getTarget
    pub fn get_target(&self) -> Option<Actor> {
        actor(&self.0, &self.state()["target"])
    }
    /// Whether the bullet is after a player.
    pub fn targets_player(&self) -> bool {
        self.state()["target"]["player"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/ShulkerBullet.setTarget
    pub fn set_target(&self, entity: Option<&Actor>) -> bool {
        // By id for a player too: a bullet's `Who::Player` is only the shulker's own.
        let id = entity.and_then(|entity| who(entity)["entity"].as_u64());
        object_done(&self.0, "shulker_target", id.map_or(json!({}), |id| json!({ "entity": id })))
    }
}

impl Wither {
    /// bukkit: org/bukkit/entity/Wither.getTarget
    pub fn get_target(&self, head: &str) -> Option<Actor> {
        actor(&self.0, &act(&self.0, "wither_target", json!({ "head": head })))
    }
    /// bukkit: org/bukkit/entity/Wither.setTarget
    pub fn set_target(&self, head: &str, entity: Option<&Actor>) -> bool {
        let mut fields = entity.map_or(json!({}), who);
        fields["head"] = json!(head);
        done(&self.0, "wither_target_set", fields)
    }
}

impl TNTPrimed {
    /// bukkit: org/bukkit/entity/TNTPrimed.getSource
    pub fn get_source(&self) -> Option<Actor> {
        actor(&self.0, &object(&self.0, "tnt", json!({}))["source"])
    }
    /// bukkit: org/bukkit/entity/TNTPrimed.setSource
    ///
    /// Only what lives can be one: anything else clears it, as on Paper.
    pub fn set_source(&self, source: Option<&Actor>) -> bool {
        object_done(&self.0, "tnt_source", source.map_or(json!({}), who))
    }
    /// bukkit: org/bukkit/entity/TNTPrimed.getSourceLoc
    pub fn get_source_loc(&self) -> Option<Location> {
        place(&self.0, &object(&self.0, "tnt", json!({}))["from"])
    }
}

impl EvokerFangs {
    /// bukkit: org/bukkit/entity/EvokerFangs.getOwner
    pub fn get_owner(&self) -> Option<Entity> {
        Some(Entity::of(object(&self.0, "fangs", json!({}))["owner"].as_u64().filter(|id| *id != 0)?, &self.get_world()))
    }
    /// bukkit: org/bukkit/entity/EvokerFangs.setOwner
    pub fn set_owner(&self, owner: Option<&Entity>) -> bool {
        object_done(&self.0, "fangs_set", json!({ "owner": owner.map_or(0, Entity::get_entity_id) }))
    }
    /// bukkit: org/bukkit/entity/EvokerFangs.getAttackDelay
    pub fn get_attack_delay(&self) -> i32 {
        object(&self.0, "fangs", json!({}))["delay"].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/EvokerFangs.setAttackDelay
    pub fn set_attack_delay(&self, delay: i32) -> bool {
        object_done(&self.0, "fangs_set", json!({ "delay": delay }))
    }
}

impl FishHook {
    fn rules(&self) -> Value {
        query(json!({ "op": "entity_player_turn", "uuid": self.0.uuid(), "key": "hook_rules" }))
    }
    fn set(&self, what: &str, value: Value) -> bool {
        query(json!({ "op": "entity_player_turn", "uuid": self.0.uuid(), "key": "hook_set", "what": what, "value": value })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/FishHook.setMinWaitTime
    pub fn set_min_wait_time(&self, ticks: i32) -> bool {
        self.set("min_wait", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/FishHook.setMaxWaitTime
    pub fn set_max_wait_time(&self, ticks: i32) -> bool {
        self.set("max_wait", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/FishHook.setWaitTime
    ///
    /// Both ends of the wait; `set_time_until_lured` is the one-number overload.
    pub fn set_wait_time(&self, min: i32, max: i32) -> bool {
        self.set("wait", json!([min, max]))
    }
    /// bukkit: org/bukkit/entity/FishHook.setWaitTime
    pub fn set_time_until_lured(&self, ticks: i32) -> bool {
        self.set("wait_time", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/FishHook.setMinLureTime
    pub fn set_min_lure_time(&self, ticks: i32) -> bool {
        self.set("min_lure", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/FishHook.setMaxLureTime
    pub fn set_max_lure_time(&self, ticks: i32) -> bool {
        self.set("max_lure", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/FishHook.setLureTime
    pub fn set_lure_time(&self, min: i32, max: i32) -> bool {
        self.set("lure", json!([min, max]))
    }
    /// bukkit: org/bukkit/entity/FishHook.setMinLureAngle
    pub fn set_min_lure_angle(&self, angle: f32) -> bool {
        self.set("min_angle", json!(angle))
    }
    /// bukkit: org/bukkit/entity/FishHook.setMaxLureAngle
    pub fn set_max_lure_angle(&self, angle: f32) -> bool {
        self.set("max_angle", json!(angle))
    }
    /// bukkit: org/bukkit/entity/FishHook.setLureAngle
    pub fn set_lure_angle(&self, min: f32, max: f32) -> bool {
        self.set("angle", json!([min, max]))
    }
    /// bukkit: org/bukkit/entity/FishHook.setApplyLure
    pub fn set_apply_lure(&self, apply: bool) -> bool {
        self.set("apply_lure", json!(apply))
    }
    /// bukkit: org/bukkit/entity/FishHook.setSkyInfluenced
    pub fn set_sky_influenced(&self, on: bool) -> bool {
        self.set("sky", json!(on))
    }
    /// bukkit: org/bukkit/entity/FishHook.setRainInfluenced
    pub fn set_rain_influenced(&self, on: bool) -> bool {
        self.set("rain", json!(on))
    }
    /// bukkit: org/bukkit/entity/FishHook.getBiteChance
    ///
    /// Paper's says it has no effect, and it has none here: the set value, else 1/300 in the rain and 1/500 out of it.
    pub fn get_bite_chance(&self) -> f64 {
        self.rules()["bite_chance"].as_f64().unwrap_or(1.0 / 500.0)
    }
    /// bukkit: org/bukkit/entity/FishHook.setBiteChance
    pub fn set_bite_chance(&self, chance: f64) -> bool {
        self.set("bite_chance", json!(chance))
    }
    /// bukkit: org/bukkit/entity/FishHook.setTimeUntilBite
    pub fn set_time_until_bite(&self, ticks: i32) -> bool {
        self.set("bite_time", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/FishHook.resetFishingState
    pub fn reset_fishing_state(&self) -> bool {
        self.set("reset", Value::Null)
    }
    /// bukkit: org/bukkit/entity/FishHook.setHookedEntity
    pub fn set_hooked_entity(&self, entity: Option<&Actor>) -> bool {
        self.set("hooked", entity.map_or(Value::Null, |entity| who(entity)["entity"].clone()))
    }
    /// bukkit: org/bukkit/entity/FishHook.pullHookedEntity
    pub fn pull_hooked_entity(&self) -> bool {
        query(json!({ "op": "entity_player_turn", "uuid": self.0.uuid(), "key": "hook_pull" })).as_bool().unwrap_or(false)
    }
}

impl Firework {
    fn tag(&self, key: &str) -> i32 {
        self.0.data_at(key).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Firework.getTicksFlown, org/bukkit/entity/Firework.getLife
    pub fn get_ticks_flown(&self) -> i32 {
        self.tag("Life")
    }
    /// bukkit: org/bukkit/entity/Firework.setTicksFlown, org/bukkit/entity/Firework.setLife
    ///
    /// False once it has gone: what Java's `setLife` answers.
    pub fn set_ticks_flown(&self, ticks: i32) -> bool {
        self.0.merge(&format!("{{Life:{ticks}}}"))
    }
    /// bukkit: org/bukkit/entity/Firework.getTicksToDetonate, org/bukkit/entity/Firework.getMaxLife
    pub fn get_ticks_to_detonate(&self) -> i32 {
        self.tag("LifeTime")
    }
    /// bukkit: org/bukkit/entity/Firework.setTicksToDetonate, org/bukkit/entity/Firework.setMaxLife
    pub fn set_ticks_to_detonate(&self, ticks: i32) -> bool {
        self.0.merge(&format!("{{LifeTime:{ticks}}}"))
    }
}

impl LightningStrike {
    /// bukkit: org/bukkit/entity/LightningStrike.getLifeTicks
    pub fn get_life_ticks(&self) -> i32 {
        self.0.data_at("Life").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/LightningStrike.setLifeTicks
    pub fn set_life_ticks(&self, ticks: i32) -> bool {
        self.0.merge(&format!("{{Life:{ticks}}}"))
    }
    /// bukkit: org/bukkit/entity/LightningStrike.getFlashCount, org/bukkit/entity/LightningStrike.getFlashes
    pub fn get_flash_count(&self) -> i32 {
        self.0.data_at("Flashes").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/LightningStrike.setFlashCount, org/bukkit/entity/LightningStrike.setFlashes
    pub fn set_flash_count(&self, flashes: i32) -> bool {
        flashes >= 0 && self.0.merge(&format!("{{Flashes:{flashes}}}"))
    }
    /// bukkit: org/bukkit/entity/LightningStrike.isEffect
    pub fn is_effect(&self) -> bool {
        self.0.data_at("Effect").as_bool().unwrap_or(false)
    }
}

impl EnderSignal {
    /// bukkit: org/bukkit/entity/EnderSignal.getTargetLocation
    pub fn get_target_location(&self) -> Option<Location> {
        self.0.location_of(&self.0.data_at("Target"))
    }
    /// bukkit: org/bukkit/entity/EnderSignal.setTargetLocation
    ///
    /// With `update`, what Java's one-argument form does: it signals to the place, as a thrown eye does, which
    /// rolls `get_drop_item` again and sets the despawn timer to 0.
    pub fn set_target_location(&self, at: &Location, update: bool) -> bool {
        object_done(&self.0, "eye_target", json!({ "value": [at.x, at.y, at.z], "update": update }))
    }
    /// bukkit: org/bukkit/entity/EnderSignal.getDropItem
    pub fn get_drop_item(&self) -> bool {
        self.0.data_at("Survive").as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/EnderSignal.setDropItem
    pub fn set_drop_item(&self, drop: bool) -> bool {
        object_done(&self.0, "eye_drop", json!({ "value": drop }))
    }
    /// bukkit: org/bukkit/entity/EnderSignal.getItem
    pub fn get_item(&self) -> ItemStack {
        ItemStack::read(&self.0.data_at("Item")).unwrap_or_else(|| ItemStack::new(crate::Material::of("minecraft:ender_eye"), 1))
    }
    /// bukkit: org/bukkit/entity/EnderSignal.setItem
    pub fn set_item(&self, item: Option<&ItemStack>) -> bool {
        object_done(&self.0, "eye_item", json!({ "value": item.map(ItemStack::to_snbt) }))
    }
    /// bukkit: org/bukkit/entity/EnderSignal.getDespawnTimer
    pub fn get_despawn_timer(&self) -> i32 {
        self.0.data_at("Life").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/EnderSignal.setDespawnTimer
    pub fn set_despawn_timer(&self, ticks: i32) -> bool {
        object_done(&self.0, "eye_life", json!({ "value": ticks.max(0) }))
    }
}

impl Evoker {
    /// bukkit: org/bukkit/entity/Evoker.getWololoTarget
    pub fn get_wololo_target(&self) -> Option<Sheep> {
        let id = get(&self.0, "wololo").as_u64().filter(|id| *id != 0)?;
        Some(Entity::of(id, &self.0.get_world()).cast())
    }
    /// bukkit: org/bukkit/entity/Evoker.setWololoTarget
    pub fn set_wololo_target(&self, sheep: Option<&Sheep>) -> bool {
        put(&self.0, "wololo", json!(sheep.map_or(0, |sheep| sheep.0.get_entity_id())))
    }
}

impl Goat {
    /// bukkit: org/bukkit/entity/Goat.ram
    ///
    /// `at`: where the entity stands (`get_location`), which is what Java's takes from the one it's given.
    pub fn ram(&self, at: &Location) -> bool {
        done(&self.0, "ram", json!({ "at": [at.x, at.y, at.z] }))
    }
}

impl Sniffer {
    /// bukkit: org/bukkit/entity/Sniffer.canDig
    pub fn can_dig(&self) -> bool {
        act(&self.0, "can_dig", json!({})).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Sniffer.findPossibleDigLocation
    pub fn find_possible_dig_location(&self) -> Option<Location> {
        place(&self.0, &act(&self.0, "dig_spot", json!({})))
    }
}

impl Villager {
    /// bukkit: org/bukkit/entity/Villager.increaseLevel
    pub fn increase_level(&self, amount: i32) -> bool {
        done(&self.0, "level_up", json!({ "amount": amount }))
    }
    /// bukkit: org/bukkit/entity/Villager.addTrades
    pub fn add_trades(&self, amount: i32) -> bool {
        done(&self.0, "add_trades", json!({ "amount": amount }))
    }
}

impl Player {
    fn stuck_time(&self, key: &str) -> i32 {
        self.stuck()[key].as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getArrowCooldown, org/bukkit/entity/LivingEntity.getNextArrowRemoval
    pub fn get_arrow_cooldown(&self) -> i32 {
        self.stuck_time("arrow_time")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setArrowCooldown, org/bukkit/entity/LivingEntity.setNextArrowRemoval
    pub fn set_arrow_cooldown(&self, ticks: i32) -> bool {
        self.set_stuck("arrow_time", ticks)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getBeeStingerCooldown, org/bukkit/entity/LivingEntity.getNextBeeStingerRemoval
    pub fn get_bee_stinger_cooldown(&self) -> i32 {
        self.stuck_time("stinger_time")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setBeeStingerCooldown, org/bukkit/entity/LivingEntity.setNextBeeStingerRemoval
    pub fn set_bee_stinger_cooldown(&self, ticks: i32) -> bool {
        self.set_stuck("stinger_time", ticks)
    }
    /// bukkit: org/bukkit/entity/Player.getExpCooldown
    pub fn get_exp_cooldown(&self) -> i32 {
        query(json!({ "op": "entity_player_get", "uuid": self.uuid(), "key": "xp_delay" })).as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/entity/Player.setExpCooldown
    pub fn set_exp_cooldown(&self, ticks: i32) -> bool {
        let ticks = crate::fam_ev_player::exp_cooldown_set(self, ticks);
        query(json!({ "op": "entity_player_set", "uuid": self.uuid(), "key": "xp_delay", "value": ticks })).as_bool().unwrap_or(false)
    }
}

// ---- What a plugin sets past the game's fields: `plugin_set` in paper_fam_entity.rs ----

fn plugin_set(entity: &Entity, key: &str, value: Value) -> bool {
    query(json!({ "op": "entity_plugin_set", "id": entity.get_entity_id(), "dimension": entity.get_world().dimension(), "key": key, "value": value })).as_bool().unwrap_or(false)
}

fn plugin_get(entity: &Entity, key: &str) -> Value {
    query(json!({ "op": "entity_plugin_get", "id": entity.get_entity_id(), "dimension": entity.get_world().dimension(), "key": key }))
}

impl TriState {
    fn json(self) -> Value {
        match self {
            TriState::True => json!(true),
            TriState::False => json!(false),
            TriState::NotSet => Value::Null,
        }
    }
    fn read(value: &Value) -> Self {
        match value.as_f64() {
            Some(0.0) => TriState::False,
            Some(_) => TriState::True,
            None => TriState::NotSet,
        }
    }
}

impl Creeper {
    /// bukkit: org/bukkit/entity/Creeper.setExplosionRadius
    pub fn set_explosion_radius(&self, radius: i32) -> bool {
        radius >= 0 && put(&self.0, "explosion_radius", json!(radius))
    }
}

impl Enderman {
    /// bukkit: org/bukkit/entity/Enderman.hasBeenStaredAt
    pub fn has_been_stared_at(&self) -> bool {
        flag(&self.0, "stared_at")
    }
    /// bukkit: org/bukkit/entity/Enderman.setHasBeenStaredAt
    pub fn set_has_been_stared_at(&self, stared_at: bool) -> bool {
        put(&self.0, "stared_at", json!(stared_at))
    }
}

impl AbstractCubeMob {
    /// bukkit: org/bukkit/entity/AbstractCubeMob.canWander
    pub fn can_wander(&self) -> bool {
        plugin_get(&self.0, "wander").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/AbstractCubeMob.setWander
    pub fn set_wander(&self, wander: bool) -> bool {
        plugin_set(&self.0, "wander", json!(wander))
    }
}

impl WanderingTrader {
    /// bukkit: org/bukkit/entity/WanderingTrader.canDrinkPotion
    pub fn can_drink_potion(&self) -> bool {
        plugin_get(&self.0, "drink_potion").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/WanderingTrader.setCanDrinkPotion
    pub fn set_can_drink_potion(&self, drink: bool) -> bool {
        plugin_set(&self.0, "drink_potion", json!(drink))
    }
    /// bukkit: org/bukkit/entity/WanderingTrader.canDrinkMilk
    pub fn can_drink_milk(&self) -> bool {
        plugin_get(&self.0, "drink_milk").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/WanderingTrader.setCanDrinkMilk
    pub fn set_can_drink_milk(&self, drink: bool) -> bool {
        plugin_set(&self.0, "drink_milk", json!(drink))
    }
}

impl Endermite {
    /// bukkit: org/bukkit/entity/Endermite.isPlayerSpawned
    ///
    /// Always false: the game dropped this and Paper's says so (`this functionality no longer exists`).
    pub fn is_player_spawned(&self) -> bool {
        false
    }
    /// bukkit: org/bukkit/entity/Endermite.setPlayerSpawned
    ///
    /// Nothing happens, as on Paper: see `is_player_spawned`.
    pub fn set_player_spawned(&self, _spawned: bool) {}
}

impl HopperMinecart {
    /// bukkit: org/bukkit/entity/minecart/HopperMinecart.getPickupCooldown
    ///
    /// `None`: Java's throws, hopper minecarts have had no cooldown since 1.19.4.
    pub fn get_pickup_cooldown(&self) -> Option<i32> {
        None
    }
    /// bukkit: org/bukkit/entity/minecart/HopperMinecart.setPickupCooldown
    ///
    /// False: Java's throws, as `get_pickup_cooldown`'s does.
    pub fn set_pickup_cooldown(&self, _cooldown: i32) -> bool {
        false
    }
}

impl ExplosiveMinecart {
    /// bukkit: org/bukkit/entity/minecart/ExplosiveMinecart.getExplosionSpeedFactor
    pub fn get_explosion_speed_factor(&self) -> f32 {
        self.0.data_at("explosion_speed_factor").as_f64().unwrap_or(1.5) as f32
    }
    /// bukkit: org/bukkit/entity/minecart/ExplosiveMinecart.setExplosionSpeedFactor
    pub fn set_explosion_speed_factor(&self, factor: f32) -> bool {
        self.0.merge(&format!("{{explosion_speed_factor:{factor:?}f}}"))
    }
}

impl Item {
    /// bukkit: org/bukkit/entity/Item.canMobPickup
    pub fn can_mob_pickup(&self) -> bool {
        plugin_get(&self.0, "mob_pickup").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/Item.setCanMobPickup
    pub fn set_can_mob_pickup(&self, can: bool) -> bool {
        plugin_set(&self.0, "mob_pickup", json!(can))
    }
}

impl Entity {
    /// bukkit: io/papermc/paper/entity/Frictional.getFrictionState
    ///
    /// On a mob, an item or a minecart: the three kinds that are `Frictional`.
    pub fn get_friction_state(&self) -> TriState {
        TriState::read(&plugin_get(self, "friction"))
    }
    /// bukkit: io/papermc/paper/entity/Frictional.setFrictionState
    pub fn set_friction_state(&self, state: TriState) -> bool {
        plugin_set(self, "friction", state.json())
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setMaximumAir
    pub fn set_maximum_air(&self, ticks: i32) -> bool {
        plugin_set(self, "max_air", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getMaximumAir
    pub fn get_maximum_air(&self) -> i32 {
        number(self, "max_air")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.isCollidable
    pub fn is_collidable(&self) -> bool {
        plugin_get(self, "collidable").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setCollidable
    pub fn set_collidable(&self, collidable: bool) -> bool {
        plugin_set(self, "collidable", json!(collidable))
    }
}

impl MushroomCow {
    fn stew(&self) -> Vec<(String, i32)> {
        get(&self.0, "stew").as_array().into_iter().flatten().filter_map(|row| Some((row[0].as_str()?.to_owned(), row[1].as_i64()? as i32))).collect()
    }
    fn set_stew(&self, effects: Option<Vec<(String, i32)>>) -> bool {
        put(&self.0, "stew", effects.map_or(Value::Null, |effects| json!(effects)))
    }
    /// bukkit: org/bukkit/entity/MushroomCow.getStewEffects
    pub fn get_stew_effects(&self) -> Vec<SuspiciousEffectEntry> {
        self.stew().into_iter().map(|(effect, duration)| SuspiciousEffectEntry { effect, duration }).collect()
    }
    /// bukkit: org/bukkit/entity/MushroomCow.setStewEffects
    pub fn set_stew_effects(&self, effects: &[SuspiciousEffectEntry]) -> bool {
        self.set_stew(Some(effects.iter().map(|entry| (entry.effect.clone(), entry.duration)).collect()))
    }
    /// bukkit: org/bukkit/entity/MushroomCow.hasEffectsForNextStew
    pub fn has_effects_for_next_stew(&self) -> bool {
        !self.stew().is_empty()
    }
    /// bukkit: org/bukkit/entity/MushroomCow.getEffectsForNextStew
    pub fn get_effects_for_next_stew(&self) -> Vec<crate::PotionEffect> {
        self.stew().into_iter().map(|(effect, duration)| crate::PotionEffect::new(&effect, duration, 0, false, true)).collect()
    }
    /// bukkit: org/bukkit/entity/MushroomCow.addEffectToNextStew
    ///
    /// Java's overloads take a `PotionEffect` or an entry: an effect's id and its ticks are all a stew keeps.
    pub fn add_effect_to_next_stew(&self, effect: &str, duration: i32, overwrite: bool) -> bool {
        let id = entity_key(effect);
        let mut stew = self.stew();
        match stew.iter().position(|row| row.0 == id) {
            Some(_) if !overwrite => return false,
            Some(index) => stew[index].1 = duration,
            None => stew.push((id, duration)),
        }
        self.set_stew(Some(stew))
    }
    /// bukkit: org/bukkit/entity/MushroomCow.removeEffectFromNextStew
    pub fn remove_effect_from_next_stew(&self, effect: &str) -> bool {
        let id = entity_key(effect);
        let mut stew = self.stew();
        let before = stew.len();
        stew.retain(|row| row.0 != id);
        stew.len() != before && self.set_stew(Some(stew))
    }
    /// bukkit: org/bukkit/entity/MushroomCow.hasEffectForNextStew
    pub fn has_effect_for_next_stew(&self, effect: &str) -> bool {
        let id = entity_key(effect);
        self.stew().iter().any(|row| row.0 == id)
    }
    /// bukkit: org/bukkit/entity/MushroomCow.clearEffectsForNextStew
    pub fn clear_effects_for_next_stew(&self) -> bool {
        self.set_stew(None)
    }
    /// bukkit: org/bukkit/entity/MushroomCow.getStewEffectType
    ///
    /// `None`: Java's throws, a mushroom cow holds a list now (`get_stew_effects`).
    pub fn get_stew_effect_type(&self) -> Option<String> {
        None
    }
    /// bukkit: org/bukkit/entity/MushroomCow.setStewEffect
    ///
    /// False: Java's throws, as `get_stew_effect_type`'s does.
    pub fn set_stew_effect(&self, _effect: Option<&str>) -> bool {
        false
    }
    /// bukkit: org/bukkit/entity/MushroomCow.getStewEffectDuration
    ///
    /// `None`: Java's throws, as `get_stew_effect_type`'s does.
    pub fn get_stew_effect_duration(&self) -> Option<i32> {
        None
    }
    /// bukkit: org/bukkit/entity/MushroomCow.setStewEffectDuration
    ///
    /// False: Java's throws, as `get_stew_effect_type`'s does.
    pub fn set_stew_effect_duration(&self, _duration: i32) -> bool {
        false
    }
}

/// `io.papermc.paper.potion.SuspiciousEffectEntry`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuspiciousEffectEntry {
    pub effect: String,
    pub duration: i32,
}

fn entity_key(id: &str) -> String {
    if id.contains(':') { id.to_owned() } else { format!("minecraft:{id}") }
}

impl Piglin {
    fn material_list(&self, key: &str) -> Vec<String> {
        plugin_get(&self.0, key).as_array().into_iter().flatten().filter_map(|row| row.as_str().map(str::to_owned)).collect()
    }
    fn list_add(&self, key: &str, material: &crate::Material) -> bool {
        let mut list = self.material_list(key);
        let id = material.key().to_owned();
        if list.contains(&id) {
            return false;
        }
        list.push(id);
        plugin_set(&self.0, key, json!(list))
    }
    fn list_remove(&self, key: &str, material: &crate::Material) -> bool {
        let mut list = self.material_list(key);
        let id = material.key().to_owned();
        let before = list.len();
        list.retain(|known| *known != id);
        list.len() != before && plugin_set(&self.0, key, json!(list))
    }
    /// bukkit: org/bukkit/entity/Piglin.addBarterMaterial
    pub fn add_barter_material(&self, material: &crate::Material) -> bool {
        self.list_add("barter", material)
    }
    /// bukkit: org/bukkit/entity/Piglin.removeBarterMaterial
    pub fn remove_barter_material(&self, material: &crate::Material) -> bool {
        self.list_remove("barter", material)
    }
    /// bukkit: org/bukkit/entity/Piglin.getBarterList
    pub fn get_barter_list(&self) -> Vec<crate::Material> {
        self.material_list("barter").iter().map(|id| crate::Material::of(id)).collect()
    }
    /// bukkit: org/bukkit/entity/Piglin.addMaterialOfInterest
    pub fn add_material_of_interest(&self, material: &crate::Material) -> bool {
        self.list_add("interest", material)
    }
    /// bukkit: org/bukkit/entity/Piglin.removeMaterialOfInterest
    pub fn remove_material_of_interest(&self, material: &crate::Material) -> bool {
        self.list_remove("interest", material)
    }
    /// bukkit: org/bukkit/entity/Piglin.getInterestList
    pub fn get_interest_list(&self) -> Vec<crate::Material> {
        self.material_list("interest").iter().map(|id| crate::Material::of(id)).collect()
    }
    /// bukkit: org/bukkit/entity/Piglin.isChargingCrossbow
    pub fn is_charging_crossbow(&self) -> bool {
        plugin_get(&self.0, "charging").as_f64() == Some(1.0)
    }
    /// bukkit: org/bukkit/entity/Piglin.setChargingCrossbow
    pub fn set_charging_crossbow(&self, charging: bool) -> bool {
        plugin_set(&self.0, "charging", json!(charging))
    }
}

impl Player {
    fn player_set(&self, key: &str, value: Value) -> bool {
        query(json!({ "op": "entity_player_set", "uuid": self.uuid(), "key": key, "value": value })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Player.isTransferred
    pub fn is_transferred(&self) -> bool {
        self.get_connection().is_transferred()
    }
    /// bukkit: org/bukkit/entity/Player.retrieveCookie
    ///
    /// Java's future is `callback`: `None` for a cookie the client doesn't have.
    pub fn retrieve_cookie(&self, plugin: &crate::Plugin, key: &crate::persistence::NamespacedKey, callback: impl FnOnce(Option<Vec<u8>>) + Send + 'static) {
        self.get_connection().retrieve_cookie(plugin, key, callback);
    }
    fn player_plugin(&self, key: &str) -> Value {
        query(json!({ "op": "entity_player_get", "uuid": self.uuid(), "key": format!("plugin:{key}") }))
    }
    /// bukkit: org/bukkit/entity/Player.setFlyingFallDamage
    pub fn set_flying_fall_damage(&self, state: TriState) -> bool {
        self.player_set("plugin:flying_fall_damage", state.json())
    }
    /// bukkit: org/bukkit/entity/Player.hasFlyingFallDamage
    pub fn has_flying_fall_damage(&self) -> TriState {
        TriState::read(&self.player_plugin("flying_fall_damage"))
    }
    /// bukkit: org/bukkit/entity/Player.setAffectsSpawning
    pub fn set_affects_spawning(&self, affects: bool) -> bool {
        self.player_set("plugin:affects_spawning", json!(affects))
    }
    /// bukkit: org/bukkit/entity/Player.getAffectsSpawning
    pub fn get_affects_spawning(&self) -> bool {
        self.player_plugin("affects_spawning").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/Player.setSimulationDistance
    pub fn set_simulation_distance(&self, distance: i32) -> bool {
        self.player_set("plugin:simulation_distance", json!(distance))
    }
    /// bukkit: org/bukkit/entity/Player.unsetFixedPose
    pub fn unset_fixed_pose(&self) -> bool {
        self.player_set("fixed_pose", Value::Null)
    }
    /// bukkit: org/bukkit/entity/Entity.setPose
    ///
    /// A player's fixed pose, held until `unset_fixed_pose`: others see it and the box follows. The player's own
    /// client is told but works its own pose out again each tick (`Player.updatePlayerPose`), on Paper too.
    pub fn set_fixed_pose(&self, pose: &str) -> bool {
        crate::enums::pose::values().iter().position(|name| *name == pose).is_some_and(|index| self.player_set("fixed_pose", json!(index)))
    }
    /// bukkit: org/bukkit/entity/Entity.hasFixedPose
    pub fn has_fixed_pose(&self) -> bool {
        !query(json!({ "op": "entity_player_get", "uuid": self.uuid(), "key": "fixed_pose" })).is_null()
    }
    /// bukkit: org/bukkit/entity/Player.loadData
    pub fn load_data(&self) -> bool {
        self.player_set("load_data", Value::Null)
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getHurtDirection
    pub fn get_hurt_direction(&self) -> f32 {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "hurt_dir" })).as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setHurtDirection, org/bukkit/entity/LivingEntity.setHurtDirection
    pub fn set_hurt_direction(&self, direction: f32) -> bool {
        query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "set_hurt_dir", "value": direction })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.startRiptideAttack
    pub fn start_riptide_attack(&self, duration: i32, strength: f32, item: Option<&ItemStack>) -> bool {
        let done = query(json!({ "op": "entity_player_turn", "uuid": self.uuid(), "key": "riptide", "duration": duration, "strength": strength, "item": item.map(ItemStack::to_snbt) })).as_bool().unwrap_or(false);
        done && self.player_set("riptide", json!(true))
    }
}

impl Entity {
    /// bukkit: org/bukkit/entity/Entity.getAirDrag
    pub fn get_air_drag(&self) -> f32 {
        if let Some(drag) = get(self, "air_drag").as_f64() {
            return drag as f32;
        }
        let kind = self.get_type();
        match kind.trim_start_matches("minecraft:") {
            "arrow" | "spectral_arrow" | "trident" | "llama_spit" | "snowball" | "egg" | "ender_pearl" | "splash_potion" | "lingering_potion" | "experience_bottle" => 0.99,
            name if name.ends_with("minecart") => 0.95,
            name if name.ends_with("_boat") || name.ends_with("_raft") => 1.0,
            _ => 0.98,
        }
    }
    /// bukkit: org/bukkit/entity/Entity.spawnAt
    ///
    /// Always false: an `Entity` here is spawned or gone, and Java's refuses both. One that isn't yet is an
    /// `EntitySnapshot`, which `create_entity` spawns.
    pub fn spawn_at(&self, _at: &Location) -> bool {
        false
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getActiveItemUsedTime, org/bukkit/entity/LivingEntity.getHandRaisedTime
    pub fn get_active_item_used_time(&self) -> i32 {
        number(self, "use_used")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getCollidableExemptions
    pub fn get_collidable_exemptions(&self) -> CollidableExemptions {
        CollidableExemptions(self.clone())
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setMemory
    ///
    /// `None` forgets it. A memory this mob's brain has no use for stays in its save and does nothing, on Paper too.
    pub fn set_memory(&self, key: &MemoryKey, value: Option<&MemoryValue>) -> bool {
        query(json!({ "op": "entity_memory", "id": self.get_entity_id(), "dimension": self.get_world().dimension(), "key": key.get_key(), "snbt": value.map(MemoryValue::snbt) })).as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Mob.setLootTable
    pub fn set_death_loot_table(&self, table: Option<&str>, seed: i64) -> bool {
        put(self, "loot_table", json!({ "table": table.map(entity_key), "seed": seed }))
    }
}

impl LivingEntityShield {
    /// bukkit: org/bukkit/entity/LivingEntity.getShieldBlockingDelay
    ///
    /// 5, as Paper's: replaced by the shield's `blocks_attacks` component.
    pub const fn get_shield_blocking_delay() -> i32 {
        5
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setShieldBlockingDelay
    ///
    /// Nothing happens, as on Paper: see `get_shield_blocking_delay`.
    pub const fn set_shield_blocking_delay(_delay: i32) {}
}

/// Paper's two shield methods, deprecated to a constant and to nothing: they read no entity.
pub struct LivingEntityShield;

impl Entity {
    fn boss_bar(&self) -> Option<crate::BossBar> {
        let id = query(json!({ "op": "entity_boss_bar", "id": self.get_entity_id(), "dimension": self.get_world().dimension() }));
        Some(crate::BossBar { id: id.as_str()?.to_owned() })
    }
}

impl Wither {
    /// bukkit: org/bukkit/entity/Boss.getBossBar
    ///
    /// A bar like any other from here on: the game goes on setting its progress and who sees it, as on Paper.
    pub fn get_boss_bar(&self) -> Option<crate::BossBar> {
        self.0.boss_bar()
    }
}

impl EnderDragon {
    /// bukkit: org/bukkit/entity/Boss.getBossBar
    ///
    /// A bar like any other from here on: the game goes on setting its progress and who sees it, as on Paper.
    /// The battle's, as `CraftEnderDragon` gives: the dragon has no other.
    pub fn get_boss_bar(&self) -> Option<crate::BossBar> {
        match self.0.get_world().get_ender_dragon_battle() {
            Some(battle) => battle.get_boss_bar(),
            None => self.0.boss_bar(),
        }
    }
    /// bukkit: org/bukkit/entity/EnderDragon.getPodium
    pub fn get_podium(&self) -> Option<Location> {
        place(&self.0, &object(&self.0, "podium", json!({})))
    }
    /// bukkit: org/bukkit/entity/EnderDragon.setPodium
    pub fn set_podium(&self, at: Option<&Location>) -> bool {
        plugin_set(&self.0, "podium", at.map_or(Value::Null, |at| json!([at.x, at.y, at.z])))
    }
}

/// `org.bukkit.entity.Interaction$PreviousInteraction`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviousInteraction {
    uuid: String,
    timestamp: i64,
}

impl PreviousInteraction {
    /// bukkit: org/bukkit/entity/Interaction$PreviousInteraction.getPlayer
    pub fn get_player(&self) -> OfflinePlayer {
        offline_by_uuid(&self.uuid)
    }
    /// bukkit: org/bukkit/entity/Interaction$PreviousInteraction.getTimestamp
    ///
    /// The game time, whatever Javadoc says: that's what the game records and Paper hands on.
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl Interaction {
    fn previous(&self, key: &str) -> Option<PreviousInteraction> {
        let saved = self.0.data_at(key);
        Some(PreviousInteraction { uuid: uuid_text(&saved["player"])?, timestamp: saved["timestamp"].as_i64()? })
    }
    /// bukkit: org/bukkit/entity/Interaction.getLastAttack
    pub fn get_last_attack(&self) -> Option<PreviousInteraction> {
        self.previous("attack")
    }
    /// bukkit: org/bukkit/entity/Interaction.getLastInteraction
    pub fn get_last_interaction(&self) -> Option<PreviousInteraction> {
        self.previous("interaction")
    }
}

impl TNTPrimed {
    /// bukkit: org/bukkit/entity/TNTPrimed.getBlockData
    pub fn get_block_data(&self) -> Option<crate::block_data::BlockData> {
        crate::entities2::block_state(&self.0.data_at("block_state")).or_else(|| Some(crate::block_data::BlockData::parse("minecraft:tnt")))
    }
    /// bukkit: org/bukkit/entity/TNTPrimed.setBlockData
    pub fn set_block_data(&self, data: &crate::block_data::BlockData) -> bool {
        self.0.merge(&format!("{{block_state:{}}}", crate::entities2::block_state_snbt(data)))
    }
}

impl FallingBlock {
    /// bukkit: org/bukkit/entity/FallingBlock.doesAutoExpire
    pub fn does_auto_expire(&self) -> bool {
        plugin_get(&self.0, "auto_expire").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/FallingBlock.shouldAutoExpire
    pub fn should_auto_expire(&self, expires: bool) -> bool {
        plugin_set(&self.0, "auto_expire", json!(expires))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.getBlockState
    ///
    /// The block, and its block entity's saved form as a `Value`, if it has one.
    pub fn get_block_state(&self) -> Option<(crate::block_data::BlockData, Value)> {
        Some((crate::entities2::block_state(&self.0.data_at("BlockState"))?, self.0.data_at("TileEntityData")))
    }
    /// bukkit: org/bukkit/entity/FallingBlock.setBlockState
    ///
    /// `tile` goes on the block entity of where it lands, as `TileState`s save it.
    pub fn set_block_state(&self, data: &crate::block_data::BlockData, tile: Option<&Value>) -> bool {
        let tile = tile.map_or(String::new(), |tile| format!(",TileEntityData:{}", crate::data::snbt(tile)));
        self.0.merge(&format!("{{BlockState:{}{tile}}}", crate::entities2::block_state_snbt(data)))
    }
}

impl ThrownPotion {
    /// bukkit: org/bukkit/entity/ThrownPotion.getPotionMeta
    pub fn get_potion_meta(&self) -> Option<crate::meta::PotionMeta> {
        Some(self.get_item()?.meta())
    }
    /// bukkit: org/bukkit/entity/ThrownPotion.setPotionMeta
    pub fn set_potion_meta(&self, meta: &crate::meta::PotionMeta) -> bool {
        let Some(mut item) = self.get_item() else {
            return false;
        };
        match meta.0.get_component("minecraft:potion_contents") {
            Some(contents) => item.set_component("minecraft:potion_contents", contents.clone()),
            None => {
                item.remove_component("minecraft:potion_contents");
            }
        }
        self.set_item(&item)
    }
}

impl ExperienceOrb {
    /// bukkit: org/bukkit/entity/ExperienceOrb.getSpawnReason
    pub fn get_spawn_reason(&self) -> String {
        plugin_get(&self.0, "spawn_reason").as_str().unwrap_or("UNKNOWN").to_owned()
    }
    /// bukkit: org/bukkit/entity/ExperienceOrb.isFromBottle
    pub fn is_from_bottle(&self) -> bool {
        self.get_spawn_reason() == "EXP_BOTTLE"
    }
    /// bukkit: org/bukkit/entity/ExperienceOrb.getTriggerEntityId
    pub fn get_trigger_entity_id(&self) -> Option<String> {
        uuid_text(&plugin_get(&self.0, "trigger"))
    }
    /// bukkit: org/bukkit/entity/ExperienceOrb.getSourceEntityId
    pub fn get_source_entity_id(&self) -> Option<String> {
        uuid_text(&plugin_get(&self.0, "source"))
    }
}

impl Raider {
    /// bukkit: org/bukkit/entity/Raider.setRaid
    pub fn set_raid(&self, raid: Option<&crate::fam_core::Raid>) -> bool {
        put(&self.0, "raid", raid.map_or(Value::Null, |raid| json!(raid.get_id())))
    }
}

/// `org.bukkit.entity.EntityType.getDefaultAttributes`: the bases a new mob of the kind starts with.
#[derive(Clone, Debug, PartialEq)]
pub struct DefaultAttributes(Value);

impl DefaultAttributes {
    /// bukkit: org/bukkit/entity/EntityType.getDefaultAttributes
    ///
    /// `kind` is the game's id: `None` for what isn't a mob, which Java's refuses.
    pub fn of(kind: &str) -> Option<Self> {
        Some(query(json!({ "op": "entity_default_attributes", "kind": entity_key(kind) }))).filter(|rows| !rows.is_null()).map(Self)
    }
    pub fn get_attribute(&self, attribute: &str) -> Option<f64> {
        self.0[entity_key(&attribute.to_ascii_lowercase())].as_f64()
    }
}

/// `io.papermc.paper.entity.poi.PoiType`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoiType(pub String);

impl PoiType {
    fn ask(&self, state: Option<String>) -> Value {
        query(json!({ "op": "entity_poi", "kind": entity_key(&self.0), "state": state }))
    }
    /// bukkit: io/papermc/paper/entity/poi/PoiType.is
    pub fn is(&self, data: &crate::block_data::BlockData) -> bool {
        self.ask(Some(crate::entities2::block_state_snbt(data)))["matches"].as_bool().unwrap_or(false)
    }
    /// bukkit: io/papermc/paper/entity/poi/PoiType.hasOccupants
    pub fn has_occupants(&self) -> bool {
        self.ask(None)["occupants"].as_bool().unwrap_or(false)
    }
}

impl crate::fam_core::PoiSearchResult {
    /// bukkit: io/papermc/paper/entity/poi/PoiSearchResult.poiType
    pub fn poi_type(&self) -> PoiType {
        PoiType(self.get_poi_type().to_owned())
    }
    /// bukkit: io/papermc/paper/entity/poi/PoiSearchResult.location
    pub fn location(&self) -> &Location {
        self.get_location()
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/LivingEntity.setMaximumAir
    pub fn set_maximum_air(&self, ticks: i32) -> bool {
        self.player_set("max_air", json!(ticks))
    }
}

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getArrowsInBody, org/bukkit/entity/LivingEntity.getArrowsStuck
    pub fn get_arrows_in_body(&self) -> i32 {
        number(self, "arrows")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setArrowsInBody, org/bukkit/entity/LivingEntity.setArrowsStuck
    pub fn set_arrows_in_body(&self, count: i32) -> bool {
        put(self, "arrows", json!(count))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getBeeStingersInBody
    pub fn get_bee_stingers_in_body(&self) -> i32 {
        number(self, "stingers")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setBeeStingersInBody
    pub fn set_bee_stingers_in_body(&self, count: i32) -> bool {
        put(self, "stingers", json!(count))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getArrowCooldown, org/bukkit/entity/LivingEntity.getNextArrowRemoval
    pub fn get_arrow_cooldown(&self) -> i32 {
        number(self, "arrow_time")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setArrowCooldown, org/bukkit/entity/LivingEntity.setNextArrowRemoval
    pub fn set_arrow_cooldown(&self, ticks: i32) -> bool {
        put(self, "arrow_time", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getBeeStingerCooldown, org/bukkit/entity/LivingEntity.getNextBeeStingerRemoval
    pub fn get_bee_stinger_cooldown(&self) -> i32 {
        number(self, "stinger_time")
    }
    /// bukkit: org/bukkit/entity/LivingEntity.setBeeStingerCooldown, org/bukkit/entity/LivingEntity.setNextBeeStingerRemoval
    pub fn set_bee_stinger_cooldown(&self, ticks: i32) -> bool {
        put(self, "stinger_time", json!(ticks))
    }
}

/// What `LivingEntity.setMemory` takes: `MemoryKey.getMemoryClass` of each key is one of these.
#[derive(Clone, Debug, PartialEq)]
pub enum MemoryValue {
    Flag(bool),
    Number(i32),
    Time(i64),
    Uuid(String),
    Place(Location),
    Places(Vec<Location>),
}

impl Entity {
    /// bukkit: org/bukkit/entity/LivingEntity.getHurtDirection
    pub fn get_hurt_direction(&self) -> f32 {
        get(self, "hurt_dir").as_f64().unwrap_or(0.0) as f32
    }
}

impl MemoryValue {
    /// A memory as the mob saves it. Whole numbers are ticks or game times by the key's name, as `MemoryKey` types them.
    fn read(saved: &Value, key: &str, world: &World) -> Option<Self> {
        let place = |row: &Value| {
            let at = row["pos"].as_array().filter(|at| at.len() == 3)?;
            let spot = |i: usize| at[i].as_f64();
            let level = row["dimension"].as_str().map_or_else(|| world.clone(), World::of);
            Some(Location::new(level, spot(0)?, spot(1)?, spot(2)?, 0.0, 0.0))
        };
        match saved {
            Value::Bool(on) => Some(Self::Flag(*on)),
            Value::Number(number) if key.contains("last_") || key.ends_with("_time") => number.as_i64().map(Self::Time),
            Value::Number(number) if key.contains("detected") || key.contains("cooling") || key.contains("hunted") || key.contains("admiring") || key.contains("is_") || key.contains("has_") || key.contains("universal") => Some(Self::Flag(number.as_i64() != Some(0))),
            Value::Number(number) => number.as_i64().map(|number| Self::Number(number as i32)),
            Value::Array(rows) if rows.len() == 4 && rows.iter().all(Value::is_number) => uuid_text(saved).map(Self::Uuid),
            Value::Array(rows) => Some(Self::Places(rows.iter().filter_map(place).collect())),
            Value::Object(_) => place(saved).map(Self::Place),
            _ => None,
        }
    }
    fn snbt(&self) -> String {
        let place = |at: &Location| format!("{{dimension:{},pos:[I;{},{},{}]}}", crate::data::quoted(at.get_world().dimension()), at.get_block_x(), at.get_block_y(), at.get_block_z());
        match self {
            Self::Flag(on) => format!("{}b", i32::from(*on)),
            Self::Number(number) => number.to_string(),
            Self::Time(time) => format!("{time}L"),
            Self::Uuid(uuid) => crate::data::uuid_snbt(uuid).unwrap_or_default(),
            Self::Place(at) => place(at),
            Self::Places(all) => format!("[{}]", all.iter().map(place).collect::<Vec<_>>().join(",")),
        }
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/LivingEntity.setCollidable
    ///
    /// Mobs stop pushing this player and being pushed by it. Its own client still predicts the collision, as Paper's
    /// Javadoc warns: a team's `COLLISION_RULE` is what reaches that.
    pub fn set_collidable(&self, collidable: bool) -> bool {
        self.player_set("plugin:collidable", json!(collidable))
    }
    /// bukkit: org/bukkit/entity/LivingEntity.isCollidable
    pub fn is_collidable(&self) -> bool {
        self.player_plugin("collidable").as_f64() != Some(0.0)
    }
    /// bukkit: org/bukkit/entity/Entity.getMaxFireTicks
    pub fn get_max_fire_ticks(&self) -> i32 {
        PLAYER_FIRE_TICKS
    }
    /// bukkit~: org/bukkit/entity/Entity.setInvulnerable | kept with the player's plugin data and read back; the game doesn't yet spare the player's damage for it
    pub fn set_invulnerable(&self, invulnerable: bool) -> bool {
        self.player_set("plugin:invulnerable", json!(invulnerable))
    }
    /// bukkit: org/bukkit/entity/Entity.isInvulnerable
    pub fn is_invulnerable(&self) -> bool {
        let flag = self.player_plugin("invulnerable");
        flag.as_bool().unwrap_or_else(|| flag.as_f64().is_some_and(|n| n != 0.0))
    }
}

/// `World.spawn` of an orb: `ExperienceOrb.SpawnReason.CUSTOM`.
pub(crate) fn orb_spawned_by_plugin(orb: &Entity) {
    plugin_set(orb, "spawn_reason", json!("CUSTOM"));
}
