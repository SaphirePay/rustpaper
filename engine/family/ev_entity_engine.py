# Family ev_entity. Yours alone. Applied by fabric-port/tools/dim_patch.py, after the mixins: each text must be in its file once.
# A mob's tick has no `Game` to lend, and `horizon_client::mobs` can't reach the binary: `mobs::ee` is the way out of it.
GAME = 'horizon-client/src/game/'
MOBS = 'horizon-client/src/mobs/'
LIB = 'horizon-client/src/'

EE = r'''/// What a plugin hears from inside a mob's tick, where no `Game` can be lent. `install` hands this
/// crate what it needs of the game's side, `ask` fires at once with what's known about who's
/// involved, and `later` keeps a change back until the mobs are back in the game.
pub mod ee {
    use super::{Mob, Who};
    use crate::items::ItemStack;
    use serde_json::{Value, json};
    use std::cell::{Cell, RefCell};
    use std::sync::{Mutex, OnceLock};

    pub struct Bridge {
        /// Fires an event: its code, and the payload as its listeners left it.
        pub fire: fn(&str, &Value) -> (i32, Value),
        pub hears: fn(&str) -> bool,
        /// Who an id was as this tick began: `Null` for nobody.
        pub known: fn(i32) -> Value,
        /// A player by their id as the game writes it.
        pub player: fn(&str) -> Value,
        pub stack: fn(&ItemStack) -> Value,
        /// The tree a mob saves, under its uuid.
        pub tree: fn(&Mob, &str) -> Value,
    }

    /// A change a listener has a say in, with the game back in hand.
    pub struct Later {
        pub mob: u32,
        pub hook: &'static str,
        pub payload: Value,
        pub apply: Box<dyn FnOnce(&mut Mob, &Value)>,
    }

    static BRIDGE: OnceLock<Bridge> = OnceLock::new();
    /// The key `known` has the host's own player under.
    pub const HOST: i32 = i32::MIN;
    /// A death's sound, held back for `EntityDeathEvent`: (mob, sound, volume, pitch).
    static VOICES: Mutex<Vec<(u32, &'static str, f32, f32)>> = Mutex::new(Vec::new());

    thread_local! {
        static GUEST: Cell<i32> = const { Cell::new(HOST) };
        static BOLT: Cell<Option<u32>> = const { Cell::new(None) };
        static HIT_BY: RefCell<Value> = const { RefCell::new(Value::Null) };
        /// Why a mob is coming into the level: `""` for what no site has said.
        static WHY: Cell<&'static str> = const { Cell::new("") };
        /// A mob's draw at its last shot.
        static FORCES: RefCell<Vec<(u32, f32)>> = const { RefCell::new(Vec::new()) };
        /// Who the effect now being given comes from.
        static EFFECT_FROM: RefCell<Value> = const { RefCell::new(Value::Null) };
        static ROWS: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
        /// The mobs an event is about, as they stand: what a listener reads of them.
        static SUBJECTS: RefCell<Vec<std::rc::Rc<Mob>>> = const { RefCell::new(Vec::new()) };
        /// The mob whose events are being carried out.
        static ACTING: RefCell<Option<std::rc::Rc<Mob>>> = const { RefCell::new(None) };
        /// A listener of `PreCreatureSpawnEvent` wants the whole pack off.
        static ABORT: Cell<bool> = const { Cell::new(false) };
        static LATER: RefCell<Vec<Later>> = const { RefCell::new(Vec::new()) };
        static REFUSED: RefCell<Vec<[i32; 3]>> = const { RefCell::new(Vec::new()) };
        static HEAL_AS: Cell<&'static str> = const { Cell::new("MAGIC") };
        static TO_ENTITY: Cell<bool> = const { Cell::new(false) };
        static ACTOR: Cell<Option<u32>> = const { Cell::new(None) };
        static EXHAUST_AS: Cell<&'static str> = const { Cell::new("UNKNOWN") };
        static HERE: RefCell<String> = const { RefCell::new(String::new()) };
        static TAME_HEARD: Cell<bool> = const { Cell::new(false) };
        static UNLEASH_AS: Cell<&'static str> = const { Cell::new("UNKNOWN") };
        static EFFECT_CAUSE: Cell<&'static str> = const { Cell::new("UNKNOWN") };
    }

    pub fn install(bridge: Bridge) {
        let _ = BRIDGE.set(bridge);
    }
    pub fn hears(hook: &str) -> bool {
        BRIDGE.get().is_some_and(|bridge| (bridge.hears)(hook))
    }
    /// Who `Who::Player` is for the mob about to tick: the guest it's bound to, else the host.
    pub fn bind(guest: Option<i32>) {
        GUEST.set(guest.unwrap_or(HOST));
        REFUSED.with(|refused| refused.borrow_mut().clear());
    }
    fn known(id: i32) -> Value {
        BRIDGE.get().map_or(Value::Null, |bridge| (bridge.known)(id))
    }
    /// A mob as it stands now, kept to hand for whoever hears of it.
    pub fn row(m: &Mob) -> Value {
        let row = fresh(m);
        if row.is_object() {
            SUBJECTS.with(|held| {
                let mut held = held.borrow_mut();
                held.retain(|kept| kept.id != m.id);
                if held.len() >= 4 {
                    held.remove(0);
                }
                held.push(std::rc::Rc::new(m.clone()));
            });
        }
        row
    }
    pub fn let_go() {
        SUBJECTS.with(|held| held.borrow_mut().clear());
    }
    pub fn hold_actor(m: Option<&Mob>) {
        let held = m.map(|m| std::rc::Rc::new(m.clone()));
        ACTING.with(|acting| *acting.borrow_mut() = held);
    }
    /// A shared hold of the mob, so that nothing runs with a borrow of the list.
    fn subject(id: u64) -> Option<std::rc::Rc<Mob>> {
        let held = SUBJECTS.with(|held| held.borrow().iter().find(|m| u64::from(m.id) == id).cloned());
        held.or_else(|| ACTING.with(|acting| acting.borrow().as_ref().filter(|m| u64::from(m.id) == id).cloned()))
    }
    pub fn subject_row(id: u64) -> Option<Value> {
        subject(id).map(|m| fresh(&m)).filter(Value::is_object)
    }
    fn fresh(m: &Mob) -> Value {
        let mut row = known(m.id as i32);
        if row.is_object() {
            let at = [m.position.x, m.position.y, m.position.z];
            row["at"] = json!(at);
            row["pos"] = json!(at);
            row["yaw"] = json!(m.yaw);
            row["pitch"] = json!(m.head_pitch);
            row["health"] = json!(m.health);
            row["max_health"] = json!(m.attrs.max_health);
            row["data"] = json!({});
        }
        row
    }
    pub fn who(who: Who) -> Value {
        match who {
            Who::Mob(id) => known(id as i32),
            Who::Player => known(GUEST.get()),
        }
    }
    pub fn stack(item: &ItemStack) -> Value {
        BRIDGE.get().map_or(Value::Null, |bridge| (bridge.stack)(item))
    }
    fn rows_in(payload: &Value) -> Vec<Value> {
        let mut rows = Vec::new();
        for value in payload.as_object().into_iter().flat_map(|map| map.values()) {
            for row in value.as_array().map_or(std::slice::from_ref(value), Vec::as_slice) {
                if row["dimension"].is_string() {
                    rows.push(row.clone());
                }
            }
        }
        rows
    }
    /// Fires now: whether a listener refused, and the payload as they left it.
    pub fn ask(hook: &str, payload: Value) -> (bool, Value) {
        let Some(bridge) = BRIDGE.get() else {
            return (false, payload);
        };
        ROWS.with(|rows| *rows.borrow_mut() = rows_in(&payload));
        let (code, after) = (bridge.fire)(hook, &payload);
        ROWS.with(|rows| rows.borrow_mut().clear());
        let_go();
        (code == 2, after)
    }
    /// What a listener running in `ask` may read of an entity.
    pub fn row_of(id: u64, tree: bool) -> Option<Value> {
        let held = ROWS.with(|rows| rows.borrow().iter().find(|row| row["id"].as_u64() == Some(id)).cloned());
        let mut row = held.or_else(|| subject_row(id)).or_else(|| Some(known(id as i32)).filter(Value::is_object))?;
        if tree && let Some(bridge) = BRIDGE.get() {
            let uuid = row["uuid"].as_str().unwrap_or_default().to_owned();
            if let Some(m) = subject(id) {
                row["data"] = (bridge.tree)(&m, &uuid);
            }
        }
        Some(row)
    }
    /// A block an event names, for a listener running in `ask`.
    fn block_row(entity: &Value, pos: [i32; 3], name: &str) -> Value {
        json!({ "dimension": entity["dimension"], "pos": pos, "name": name })
    }
    /// What a listener running in `ask` may read of a player.
    pub fn player_of(uuid: &str) -> Option<Value> {
        ROWS.with(|rows| rows.borrow().iter().find(|row| row["uuid"] == uuid).cloned())
    }
    /// What a listener running in `ask` may read of a block.
    pub fn block_of(pos: &Value) -> Option<Value> {
        ROWS.with(|rows| rows.borrow().iter().find(|row| row["pos"] == *pos && row["name"].is_string()).cloned())
    }
    /// A change that waits for the mobs to be back: the listeners then have the whole game.
    pub fn later(mob: u32, hook: &'static str, payload: Value, apply: impl FnOnce(&mut Mob, &Value) + 'static) {
        LATER.with(|later| later.borrow_mut().push(Later { mob, hook, payload, apply: Box::new(apply) }));
    }
    pub fn take_later() -> Vec<Later> {
        LATER.with(|later| std::mem::take(&mut *later.borrow_mut()))
    }

    /// `Entity.checkFallDamage`: whether the landing does anything; a listener may change the fall.
    pub fn landed(m: &mut Mob, on: [i32; 3], name: &str) -> bool {
        if !hears("ee_land") {
            return true;
        }
        let entity = row(m);
        let block = block_row(&entity, on, name);
        let (refused, after) = ask("ee_land", json!({ "entity": entity, "block": block, "fall": m.fall_distance }));
        m.fall_distance = after["fall"].as_f64().unwrap_or(m.fall_distance);
        !refused
    }
    /// `Entity.push`: true if a listener keeps them from pushing.
    pub fn collide(m: &Mob, other: Who) -> bool {
        hears("ee_collide") && ask("ee_collide", json!({ "entities": [row(m), who(other)] })).0
    }
    /// The blocks `entityInside` does something in, on the mobs Horizon has.
    pub fn acts_on_entities(name: &str) -> bool {
        // The blocks whose `entityInside` Paper asks in, every one.
        let name = name.trim_start_matches("minecraft:");
        name.ends_with("_pressure_plate")
            || name.ends_with("_button")
            || matches!(
                name,
                "cactus" | "cobweb" | "fire" | "soul_fire" | "water_cauldron" | "lava_cauldron" | "powder_snow" | "powder_snow_cauldron"
                    | "sweet_berry_bush" | "wither_rose" | "open_eyeblossom" | "closed_eyeblossom" | "campfire" | "soul_campfire" | "honey_block"
                    | "nether_portal" | "end_portal" | "end_gateway" | "bubble_column" | "tripwire" | "detector_rail" | "big_dripleaf" | "frogspawn"
                    | "hopper" | "lily_pad" | "wheat" | "carrots" | "potatoes" | "beetroots" | "torchflower_crop" | "pitcher_crop"
            )
    }
    /// Each block's `entityInside`: true if a listener stopped it acting on this mob.
    pub fn inside(m: &Mob, name: &str, pos: [i32; 3]) -> bool {
        if !hears("ee_inside") || !acts_on_entities(name) {
            return false;
        }
        if REFUSED.with(|refused| refused.borrow().contains(&pos)) {
            return true;
        }
        let entity = row(m);
        let block = block_row(&entity, pos, name);
        let refused = ask("ee_inside", json!({ "entity": entity, "block": block })).0;
        if refused {
            REFUSED.with(|list| list.borrow_mut().push(pos));
        }
        refused
    }
    /// `TransportItemsBetweenContainers.isWantedBlock`, a block at a time.
    pub fn target_allowed(m: &Mob, pos: [i32; 3], name: &str) -> bool {
        if !hears("ee_transport_target") {
            return true;
        }
        let entity = row(m);
        let block = block_row(&entity, pos, name);
        let (_, after) = ask("ee_transport_target", json!({ "entity": entity, "block": block, "allowed": true }));
        after["allowed"].as_bool().unwrap_or(true)
    }
    /// Whether what a mob carries belongs in the container at `pos`: `own` is the game's answer.
    pub fn sorted(m: &Mob, carried: Option<&ItemStack>, pos: [i32; 3], items: &[Option<ItemStack>], own: bool) -> bool {
        if !hears("ee_transport_sort") {
            return own;
        }
        let contents: Vec<Value> = items.iter().map(|item| item.as_ref().map_or(Value::Null, stack)).collect();
        let held = carried.map_or(Value::Null, stack);
        let (_, after) = ask("ee_transport_sort", json!({ "entity": row(m), "pos": pos, "stack": held, "items": contents, "result": "DEFAULT" }));
        match after["result"].as_str() {
            Some("ALLOW") => true,
            Some("DENY") => false,
            _ => own,
        }
    }
    /// `ItemStack.hurtAndBreak` on what a mob wears: the durability it really loses.
    pub fn wear(m: &Mob, item: &ItemStack, loss: i32) -> i32 {
        if !hears("ee_item_wear") {
            return loss;
        }
        let (refused, after) = ask("ee_item_wear", json!({ "entity": row(m), "item": stack(item), "damage": loss }));
        if refused { 0 } else { after["damage"].as_i64().map_or(loss, |damage| damage as i32) }
    }
    /// `Entity.setSwimming`: true if a listener keeps it as it is.
    pub fn swim_refused(swimming: bool) -> bool {
        hears("ee_swim") && ask("ee_swim", json!({ "entity": who(Who::Player), "swimming": swimming })).0
    }
    /// `DyeItem.interactLivingEntity` and a collar's dye: the colour it takes, or `None` if refused.
    pub fn dye(m: &Mob, color: u8, player: [i32; 4]) -> Option<u8> {
        if !hears("ee_dye") {
            return Some(color);
        }
        let by = BRIDGE.get().map_or(Value::Null, |bridge| (bridge.player)(&format!("{:032x}", crate::lan::playerdata::ints_uuid(player))));
        let (refused, after) = ask("ee_dye", json!({ "entity": row(m), "kind": m.kind.id, "player": by, "color": color }));
        (!refused).then(|| after["color"].as_u64().map_or(color, |color| color as u8))
    }
    /// `LivingEntity.heal` with its reason: what it's healed by, or `None` if refused.
    pub fn regain_row(entity: Value, amount: f32, reason: &str, fast: bool) -> Option<f32> {
        if !hears("ee_regain") {
            return Some(amount);
        }
        let (refused, after) = ask("ee_regain", json!({ "entity": entity, "amount": amount, "reason": reason, "fast": fast }));
        (!refused).then(|| after["amount"].as_f64().map_or(amount, |amount| amount as f32))
    }
    pub fn regain(m: &Mob, amount: f32, reason: &str) -> Option<f32> {
        if hears("ee_regain") { regain_row(row(m), amount, reason, false) } else { Some(amount) }
    }
    pub fn heal(m: &mut Mob, amount: f32, reason: &str) {
        if let Some(amount) = regain(m, amount, reason) {
            m.health = (m.health + amount).min(m.attrs.max_health);
        }
    }
    /// The dragon isn't one of the mobs: it's known by its id.
    pub fn dragon_regain(id: u32, amount: f32) -> Option<f32> {
        if !hears("ee_regain") {
            return Some(amount);
        }
        let entity = json!({ "id": id, "dimension": "minecraft:the_end", "uuid": "", "kind": "minecraft:ender_dragon", "data": {} });
        regain_row(entity, amount, "ENDER_CRYSTAL", false)
    }
    /// The player in hand: the guest whose turn it is, else the host.
    pub fn player_regain(amount: f32, reason: &str, fast: bool) -> Option<f32> {
        if hears("ee_regain") { regain_row(who(Who::Player), amount, reason, fast) } else { Some(amount) }
    }
    /// Why the next heal in `apply_effect_action` happens: magic, unless told.
    pub fn heal_as(reason: &'static str) {
        HEAL_AS.set(reason);
    }
    pub fn take_heal_reason() -> &'static str {
        HEAL_AS.replace("MAGIC")
    }
    /// `Mob.setTarget(target, reason)`: a listener hears it with the mobs back in the game, and only
    /// then does the target change. `CLOSEST` is `CLOSEST_PLAYER` or `CLOSEST_ENTITY` by who it is.
    pub fn target(m: &mut Mob, target: Option<Who>, reason: &'static str, apply: fn(&mut Mob, Option<Who>)) {
        if !hears("ee_target") {
            return apply(m, target);
        }
        let reason = match (reason, target) {
            ("CLOSEST", Some(Who::Player)) => "CLOSEST_PLAYER",
            ("CLOSEST", _) => "CLOSEST_ENTITY",
            _ => reason,
        };
        let payload = json!({ "entity": row(m), "target": target.map_or(Value::Null, who), "reason": reason });
        later(m.id, "ee_target", payload, move |m, after| {
            let set = &after["target"];
            let set = match set["id"].as_u64() {
                _ if set.is_null() => None,
                Some(id) if set["player"] != true => Some(Who::Mob(id as u32)),
                _ => Some(Who::Player),
            };
            apply(m, set);
        });
    }
    fn with(m: &Mob, mut more: Value) -> Value {
        more["entity"] = row(m);
        more
    }
    /// Asks about a mob: true if a listener refused.
    pub fn stop(hook: &str, m: &Mob, more: Value) -> bool {
        hears(hook) && ask(hook, with(m, more)).0
    }
    /// Asks about a mob: what the listeners left, or `None` if one refused.
    pub fn say(hook: &str, m: &Mob, more: Value) -> Option<Value> {
        if !hears(hook) {
            return Some(more);
        }
        let (refused, after) = ask(hook, with(m, more));
        (!refused).then_some(after)
    }
    /// A potion a witch is about to lift or to drink, as the listeners left it.
    pub fn witch_potion(hook: &str, m: &Mob, potion: Option<&str>) -> Option<String> {
        let potion = potion?;
        effect_cause("ATTACK");
        let after = say(hook, m, json!({ "potion": potion }))?;
        after["potion"].as_str().map(str::to_owned)
    }
    /// What a witch drinks that the game has no case of: it's a potion like any other.
    pub fn drink(m: &mut Mob, potion: &str) {
        for (effect, ticks, amplifier) in crate::brewing::potion(potion).map_or(&[][..], |def| def.effects) {
            if !crate::brewing::effect(effect).is_some_and(|e| e.instant) {
                m.add_effect(effect, *ticks, *amplifier);
            }
        }
    }
    /// The path about to be found is to an entity: the mob's target.
    pub fn path_to_entity() {
        TO_ENTITY.set(true);
    }
    pub fn take_path_to_entity() -> bool {
        TO_ENTITY.replace(false)
    }
    /// `Entity.randomTeleport` and the like: where it lands, or `None` if refused.
    pub fn teleport(m: &Mob, to: [f64; 3]) -> Option<[f64; 3]> {
        if !hears("ee_teleport") {
            return Some(to);
        }
        let from = [m.position.x, m.position.y, m.position.z];
        let after = say("ee_teleport", m, json!({ "from": from, "to": to }))?;
        let at = |i: usize| after["to"][i].as_f64();
        Some([at(0)?, at(1)?, at(2)?])
    }
    /// A mob catching fire: for how many ticks, or `None` if refused. `by` is a row, with the
    /// block or the entity that lit it.
    pub fn combust(m: &Mob, ticks: i32, by: Value) -> Option<i32> {
        if !hears("ee_combust") {
            return Some(ticks);
        }
        let seconds = ticks as f32 / 20.0;
        let after = say("ee_combust", m, json!({ "duration": seconds, "by": by }))?;
        Some(after["duration"].as_f64().map_or(ticks, |seconds| (seconds * 20.0) as i32))
    }
    /// A block as `combust` names one.
    pub fn block_by(m: &Mob, name: &str, pos: [i32; 3]) -> Value {
        json!({ "block": name, "dimension": known(m.id as i32)["dimension"], "pos": pos })
    }
    pub fn entity_by(who_is: Who) -> Value {
        who(who_is)
    }
    /// `Animal.setInLove`: for how many ticks, or `None` if refused.
    pub fn love(m: &Mob, ticks: i32, cause: Option<[i32; 4]>) -> Option<i32> {
        if !hears("ee_love") {
            return Some(ticks);
        }
        let by = cause.map_or(Value::Null, |cause| BRIDGE.get().map_or(Value::Null, |bridge| (bridge.player)(&format!("{:032x}", crate::lan::playerdata::ints_uuid(cause)))));
        let after = say("ee_love", m, json!({ "ticks": ticks, "human": by }))?;
        Some(after["ticks"].as_i64().map_or(ticks, |ticks| ticks as i32))
    }
    /// `Villager.setVillagerData`: the profession it takes, or `None` if refused.
    pub fn career(m: &Mob, profession: u8) -> Option<u8> {
        let after = say("ee_career", m, json!({ "profession": profession }))?;
        Some(after["profession"].as_u64().map_or(profession, |profession| profession as u8))
    }
    /// `WardenAngerChangeEvent`: what the anger becomes, or `None` if refused.
    pub fn warden_anger(m: &Mob, at: Who, old: i32, new: i32) -> Option<i32> {
        let after = say("ee_warden_anger", m, json!({ "target": who(at), "old": old, "new": new }))?;
        Some(after["new"].as_i64().map_or(new, |new| new as i32))
    }
    /// `CreeperIgniteEvent`: whether it's lit after asking.
    pub fn creeper_ignite(m: &mut Mob, lit: bool) {
        if let Some(after) = say("ee_creeper_ignite", m, json!({ "ignited": lit, "fuse": fuse(m) })) {
            m.extra.flag = after["ignited"].as_bool().unwrap_or(lit);
            if let Some(fuse) = after["fuse"].as_i64() {
                m.extra.set_plugin_number("ee_fuse", fuse as f64);
            }
        }
    }
    /// A creeper's `maxSwell`.
    pub fn fuse(m: &Mob) -> i32 {
        m.extra.plugin_number("ee_fuse").map_or(30, |fuse| fuse as i32)
    }
    /// Who the events being handled are by.
    pub fn set_actor(mob: u32) {
        ACTOR.set(Some(mob));
    }
    pub fn actor_by() -> Value {
        ACTOR.get().map_or(Value::Null, |mob| known(mob as i32))
    }
    /// An entity the mobs don't include, in the mob's own level.
    pub fn other_by(m: &Mob, id: u32, kind: &str) -> Value {
        json!({ "id": id, "dimension": known(m.id as i32)["dimension"], "uuid": "", "kind": kind })
    }
    /// The level a thing with no row of its own is in.
    pub fn set_here(level: &str) {
        HERE.with(|here| *here.borrow_mut() = level.to_owned());
    }
    fn here() -> String {
        HERE.with(|here| here.borrow().clone())
    }
    /// `Entity.setPose`, for the player in hand: it can't be refused.
    pub fn pose(name: String) {
        if hears("ee_pose") {
            ask("ee_pose", json!({ "entity": who(Who::Player), "pose": name }));
        }
    }
    /// `LivingEntity.setSharedFlag` for gliding, on the player in hand: true if refused.
    pub fn glide_refused(gliding: bool) -> bool {
        hears("ee_glide") && ask("ee_glide", json!({ "entity": who(Who::Player), "gliding": gliding })).0
    }
    /// `FoodLevelChangeEvent`: the food it comes to, or `None` if refused.
    pub fn food(level: i32, item: Option<&ItemStack>) -> Option<i32> {
        if !hears("ee_food") {
            return Some(level);
        }
        let (refused, after) = ask("ee_food", json!({ "entity": who(Who::Player), "level": level, "item": item.map_or(Value::Null, stack) }));
        (!refused).then(|| after["level"].as_i64().map_or(level, |level| level as i32))
    }
    /// Why the next exhaustion happens: nothing's said, unless told.
    pub fn exhaust_as(reason: &'static str) {
        EXHAUST_AS.set(reason);
    }
    /// `MovementStat`'s name, for what walking costs.
    pub fn exhaust_for(stat: &str) {
        EXHAUST_AS.set(match stat {
            "minecraft:swim_one_cm" | "swim_one_cm" => "SWIM",
            "minecraft:walk_under_water_one_cm" | "walk_under_water_one_cm" => "WALK_UNDERWATER",
            "minecraft:walk_on_water_one_cm" | "walk_on_water_one_cm" => "WALK_ON_WATER",
            "minecraft:sprint_one_cm" | "sprint_one_cm" => "SPRINT",
            "minecraft:crouch_one_cm" | "crouch_one_cm" => "CROUCH",
            "minecraft:walk_one_cm" | "walk_one_cm" => "WALK",
            _ => "UNKNOWN",
        });
    }
    /// `EntityExhaustionEvent`: what it really costs, or `None` if refused.
    pub fn exhaustion(amount: f32) -> Option<f32> {
        let reason = EXHAUST_AS.replace("UNKNOWN");
        if !hears("ee_exhaustion") {
            return Some(amount);
        }
        let (refused, after) = ask("ee_exhaustion", json!({ "entity": who(Who::Player), "amount": amount, "reason": reason }));
        (!refused).then(|| after["amount"].as_f64().map_or(amount, |amount| amount as f32))
    }
    /// `EntityAirChangeEvent`, as a mob's air changed: what it's left with.
    pub fn air(m: &mut Mob, before: i32) {
        if m.air != before && hears("ee_air") {
            let (refused, after) = ask("ee_air", with(m, json!({ "amount": m.air })));
            m.air = if refused { before } else { after["amount"].as_i64().map_or(m.air, |air| air as i32) };
        }
    }
    /// `PigZombieAngerEvent`: for how long, or `None` if refused.
    pub fn pig_anger(m: &Mob, anger: i32) -> Option<i32> {
        let by = m.ai.target.map_or(Value::Null, who);
        let after = say("ee_pig_anger", m, json!({ "anger": anger, "target": by }))?;
        Some(after["anger"].as_i64().map_or(anger, |anger| anger as i32))
    }
    /// `EnderDragonChangePhaseEvent`: the phase it goes to, or `None` if refused.
    pub fn dragon_phase(id: u32, old: i64, new: i64) -> Option<i64> {
        if !hears("ee_dragon_phase") {
            return Some(new);
        }
        let entity = json!({ "id": id, "dimension": here(), "uuid": "", "kind": "minecraft:ender_dragon", "data": {} });
        let (refused, after) = ask("ee_dragon_phase", json!({ "entity": entity, "old": old, "new": new }));
        (!refused).then(|| after["new"].as_i64().unwrap_or(new))
    }
    /// An item on the ground, as an event's row.
    fn item_row(item: &crate::entities::ItemEntity) -> Value {
        json!({ "id": item.id, "dimension": here(), "uuid": "", "kind": "minecraft:item", "at": [item.position.x, item.position.y, item.position.z], "pos": [item.position.x, item.position.y, item.position.z], "stack": stack(&item.stack), "data": {} })
    }
    /// `ItemMergeEvent`: true if a listener keeps the two apart.
    pub fn item_merge(from: &crate::entities::ItemEntity, to: &crate::entities::ItemEntity) -> bool {
        hears("ee_item_merge") && ask("ee_item_merge", json!({ "entity": item_row(from), "target": item_row(to) })).0
    }
    fn orb_row(orb: &crate::entities::ExperienceOrb) -> Value {
        json!({ "id": orb.id, "dimension": here(), "uuid": "", "kind": "minecraft:experience_orb", "pos": [orb.position.x, orb.position.y, orb.position.z], "data": {} })
    }
    /// `ExperienceOrbMergeEvent`: true if a listener keeps the two apart.
    pub fn orb_merge(target: &crate::entities::ExperienceOrb, source: &crate::entities::ExperienceOrb) -> bool {
        hears("ee_orb_merge") && ask("ee_orb_merge", json!({ "entity": orb_row(target), "source": orb_row(source) })).0
    }
    /// `ArrowBodyCountChangeEvent`: what the count comes to.
    pub fn arrows(old: i32, new: i32) -> i32 {
        if old == new || !hears("ee_arrows") {
            return new;
        }
        let (refused, after) = ask("ee_arrows", json!({ "entity": who(Who::Player), "old": old, "new": new, "reset": false }));
        if refused { old } else { after["new"].as_i64().map_or(new, |new| new as i32) }
    }
    fn seat_row(id: u32) -> Value {
        let row = known(id as i32);
        if row.is_null() { json!({ "id": id, "dimension": here(), "uuid": "", "data": {} }) } else { row }
    }
    /// `Entity.startRiding`, by a mob: true if a listener keeps it off.
    pub fn mob_boards(rider: Value, vehicle: u32) -> bool {
        hears("ee_mount") && ask("ee_mount", json!({ "rider": rider, "mount": seat_row(vehicle) })).0
    }
    /// `Entity.removeVehicle`, by a mob: true if a listener keeps it on.
    pub fn mob_leaves(rider: Value, vehicle: u32) -> bool {
        hears("ee_dismount") && ask("ee_dismount", json!({ "rider": rider, "mount": seat_row(vehicle) })).0
    }
    /// `HorseJumpEvent`: the power it jumps with, 0 if refused.
    pub fn horse_jump(m: &Mob, power: f32) -> f32 {
        match say("ee_horse_jump", m, json!({ "power": power })) {
            Some(after) => after["power"].as_f64().map_or(power, |power| power as f32),
            None => 0.0,
        }
    }
    /// `Villager.restock`, an offer at a time: the uses go when the listeners have said.
    pub fn restock(m: &mut Mob, game_time: i64) {
        let Some(trade) = m.extra.trade.as_mut() else {
            return;
        };
        if !hears("ee_replenish") {
            return trade.restock(game_time);
        }
        let uses: Vec<i32> = trade.offers.iter().map(|offer| offer.uses).collect();
        trade.restock(game_time);
        for (offer, uses) in trade.offers.iter_mut().zip(&uses) {
            offer.uses = *uses;
        }
        let entity = row(m);
        for index in 0..uses.len() {
            later(m.id, "ee_replenish", json!({ "entity": entity, "index": index }), move |m, _| {
                if let Some(offer) = m.extra.trade.as_mut().and_then(|trade| trade.offers.get_mut(index)) {
                    offer.uses = 0;
                }
            });
        }
    }
    /// `Entity.checkFallDamage` for the player in hand: the fall that counts, 0 if refused.
    pub fn fall(fall: f64, on: [i32; 3], name: &str) -> f64 {
        if !hears("ee_land") {
            return fall;
        }
        let entity = who(Who::Player);
        let block = block_row(&entity, on, name);
        let (refused, after) = ask("ee_land", json!({ "entity": entity, "block": block, "fall": fall }));
        if refused { 0.0 } else { after["fall"].as_f64().unwrap_or(fall) }
    }
    /// A block's `entityInside`, on the player in hand: true if a listener stopped it.
    pub fn player_inside(name: &str, pos: [i32; 3]) -> bool {
        if !hears("ee_inside") || !acts_on_entities(name) {
            return false;
        }
        let entity = who(Who::Player);
        let block = block_row(&entity, pos, name);
        ask("ee_inside", json!({ "entity": entity, "block": block })).0
    }
    /// `EntityAirChangeEvent` on the player in hand: the air they're left with.
    pub fn player_air(before: i32, air: i32) -> i32 {
        if before == air || !hears("ee_air") {
            return air;
        }
        let (refused, after) = ask("ee_air", json!({ "entity": who(Who::Player), "amount": air }));
        if refused { before } else { after["amount"].as_i64().map_or(air, |air| air as i32) }
    }
    /// `HorseJumpEvent` from a guest's client, which moves the horse itself.
    pub fn guest_horse_jump(vehicle: u32, data: i32) {
        if hears("ee_horse_jump") {
            ask("ee_horse_jump", json!({ "entity": seat_row(vehicle), "power": data as f32 / 100.0 }));
        }
    }
    /// `EntityPotionEffectEvent` for a mob: true if refused.
    pub fn effect_refused(m: &Mob, effect: &str, ticks: i32, amplifier: i32) -> bool {
        if !hears("ee_effect") {
            return false;
        }
        let old = m.extra.effects.iter().find(|e| e.0 == effect).map_or(Value::Null, |e| json!({ "amplifier": e.2, "duration": e.1 }));
        let source = take_effect_from();
        ask("ee_effect", with(m, json!({ "effect": effect, "amplifier": amplifier, "duration": ticks, "old": old, "cause": EFFECT_CAUSE.get(), "source": source }))).0
    }
    /// What's giving effects just now.
    pub fn effect_cause(cause: &'static str) {
        EFFECT_CAUSE.set(cause);
    }
    /// Why the next lead breaks: nobody said, unless told.
    pub fn unleash_as(reason: &'static str) {
        UNLEASH_AS.set(reason);
    }
    pub fn take_unleash_reason() -> &'static str {
        UNLEASH_AS.replace("UNKNOWN")
    }
    /// Rows for what a listener outside `ask` may read: a hit, which fires from the game's own event.
    pub fn hold_rows(rows: Vec<Value>) {
        ROWS.with(|held| *held.borrow_mut() = rows);
    }
    /// `EntityToggleSitEvent`: it sits or stands unless a listener refuses.
    pub fn sit(m: &mut Mob, sitting: bool) {
        if sitting == m.extra.sitting || !stop("ee_sit", m, json!({ "sitting": sitting })) {
            m.extra.sitting = sitting;
        }
    }
    /// `PufferFishStateChangeEvent`: the puff it takes, unless a listener refuses.
    pub fn puff(m: &mut Mob, state: i32) {
        if !stop("ee_puff", m, json!({ "state": state, "old": m.extra.size })) {
            m.extra.size = state;
        }
    }
    /// `CraftEventFactory.callEntityTameEvent`, with the roll: true if refused.
    pub fn tame_refused(m: &Mob, owner: [i32; 4]) -> bool {
        if !hears("ee_tame") {
            return false;
        }
        TAME_HEARD.set(true);
        let uuid = format!("{:032x}", crate::lan::playerdata::ints_uuid(owner));
        let by = BRIDGE.get().map_or(Value::Null, |bridge| (bridge.player)(&uuid));
        ask("ee_tame", with(m, json!({ "kind": "tame", "owner": uuid, "by": by }))).0
    }
    /// Whether the taming the game now credits has been asked about already.
    pub fn take_tame_heard() -> bool {
        TAME_HEARD.replace(false)
    }
    /// A player catching fire: for how many ticks, or `None` if refused.
    pub fn player_combust(ticks: i32) -> Option<i32> {
        if !hears("ee_combust") {
            return Some(ticks);
        }
        let (refused, after) = ask("ee_combust", json!({ "entity": who(Who::Player), "duration": ticks as f32 / 20.0, "by": Value::Null }));
        (!refused).then(|| after["duration"].as_f64().map_or(ticks, |seconds| (seconds * 20.0) as i32))
    }
    /// `LivingEntity.knockback`: how hard and which way, or `None` if refused.
    pub fn mob_knock(m: &Mob, power: f64, x: f64, z: f64) -> Option<(f64, f64, f64)> {
        let after = say("ee_mob_knock", m, json!({ "power": power, "x": x, "z": z }))?;
        let number = |key: &str, own: f64| after[key].as_f64().unwrap_or(own);
        Some((number("power", power), number("x", x), number("z", z)))
    }
    /// `EntityPotionEffectEvent` as a mob's effects run out: a refused one has a tick left.
    pub fn effects_ending(m: &Mob, effects: &mut [(&'static str, i32, i32)]) {
        if !hears("ee_effect") {
            return;
        }
        for effect in effects.iter_mut().filter(|effect| effect.1 <= 0 && effect.1 != -1) {
            let old = json!({ "amplifier": effect.2, "duration": effect.1 });
            let payload = json!({ "effect": effect.0, "amplifier": effect.2, "duration": 0, "old": old, "cause": "EXPIRATION", "action": "REMOVED", "ended": true });
            if ask("ee_effect", with(m, payload)).0 {
                effect.1 = 1;
            }
        }
    }
    /// All of a mob's effects going at once: those a listener keeps stay.
    pub fn effects_clear(m: &mut Mob, cause: &str) {
        if !hears("ee_effect") {
            return m.extra.effects.clear();
        }
        let list = std::mem::take(&mut m.extra.effects);
        let mut kept = Vec::new();
        for effect in list {
            let old = json!({ "amplifier": effect.2, "duration": effect.1 });
            let payload = json!({ "effect": effect.0, "amplifier": effect.2, "duration": 0, "old": old, "cause": cause, "action": "CLEARED", "ended": true });
            if ask("ee_effect", with(m, payload)).0 {
                kept.push(effect);
            }
        }
        m.extra.effects = kept;
    }
    /// `sync::pose_id` before a mob's tick, if anyone wants to know.
    pub fn pose_of(m: &Mob) -> Option<u8> {
        hears("ee_pose").then(|| super::sync::pose_id(m))
    }
    /// `EntityPoseChangeEvent`, from what the tick did to it.
    pub fn pose_after(m: &Mob, before: Option<u8>) {
        if let Some(before) = before {
            let now = super::sync::pose_id(m);
            if now != before {
                ask("ee_pose", with(m, json!({ "pose_id": now })));
            }
        }
    }
    /// `LivingEntity.checkTotemDeathProtection` for a mob at no health.
    pub fn totem(m: &mut Mob, bypass: bool) {
        const TOTEM: &str = "minecraft:totem_of_undying";
        if bypass {
            return;
        }
        let hand = if m.extra.main_hand.as_deref() == Some(TOTEM) {
            Some("HAND")
        } else if m.extra.off_hand.as_deref() == Some(TOTEM) {
            Some("OFF_HAND")
        } else {
            None
        };
        // With no totem Paper asks too, already refused: a listener may bring it back.
        let heard = hears("ee_mob_totem");
        if hand.is_none() && !heard {
            return;
        }
        if heard && ask("ee_mob_totem", with(m, json!({ "hand": hand }))).0 {
            return;
        }
        match hand {
            Some("HAND") => m.extra.set_main_hand(None),
            Some(_) => m.extra.off_hand = None,
            None => {}
        }
        m.health = 1.0;
        effects_clear(m, "TOTEM");
        effect_cause("TOTEM");
        m.add_effect("minecraft:regeneration", 900, 1);
        m.add_effect("minecraft:absorption", 100, 1);
        m.add_effect("minecraft:fire_resistance", 800, 0);
        let at = [m.position.x, m.position.y, m.position.z];
        m.outbox.push(super::MobEvent::EntityEvent { mob: m.id, event: 35, at });
    }
    /// What the game does to a mob's hit once it has been heard, worked out once, for
    /// the hit and for `EntityDamageEvent` both. `effectiveness` is breach: it adds to the
    /// armour's fraction, so what it adds holds at whatever the hit comes to.
    pub fn mob_steps(
        m: &Mob,
        amount: f32,
        source: &super::Damage,
        effectiveness: Option<&mut dyn FnMut(f32) -> f32>,
    ) -> crate::health::HitSteps {
        use super::{DamageKind, Species};
        let (worn, worn_toughness) = m.worn_armor();
        let covered = if m.species() == Species::Shulker && m.extra.counter == 0 { 20.0 } else { 0.0 };
        let armor = m.kind.armor + worn + covered;
        let armor = (!source.bypasses_armor() && armor > 0.0).then_some((armor, m.kind.armor_toughness + worn_toughness));
        let mut steps = crate::health::HitSteps {
            armor,
            freezing: source.kind == DamageKind::Freeze && matches!(m.species(), Species::Strider | Species::Blaze | Species::MagmaCube),
            helmet: matches!(source.kind, DamageKind::FallingBlock | DamageKind::FallingAnvil | DamageKind::FallingStalactite)
                && !super::species::body_slot(m.species())
                && m.extra.head.is_some(),
            frames: if m.invulnerable as f32 > 10.0 { m.last_hurt } else { 0.0 },
            // `minecraft:bypasses_enchantments` holds only the sonic boom.
            protection: if source.kind == DamageKind::SonicBoom { 0.0 } else { super::gear::protection(m, source.kind) },
            ..Default::default()
        };
        if let (Some((armor, toughness)), Some(modify)) = (armor, effectiveness) {
            let fraction = crate::health::armor_fraction(armor, toughness, amount);
            steps.breach = Some(modify(fraction) - fraction);
        }
        if !matches!(source.kind, DamageKind::Starve | DamageKind::GenericKill | DamageKind::OutOfWorld) {
            steps.resistance = m.extra.effects.iter().find(|effect| effect.0 == "minecraft:resistance").map(|effect| effect.2);
        }
        if m.species() == Species::Witch {
            if source.attacker == Some(Who::Mob(m.id)) {
                steps.magic = Some(0.0);
            } else if matches!(source.kind, DamageKind::Magic | DamageKind::IndirectMagic | DamageKind::SonicBoom | DamageKind::Thorns) {
                steps.magic = Some(0.15);
            }
        }
        steps
    }
    /// `EntityDamageEvent` asks what the game's steps make of a hit: this hit is on
    /// the stack, whoever holds the game.
    pub fn hit_answer(request: &str) -> Option<String> {
        if !request.contains("ee_hit_") {
            return None;
        }
        let request: Value = serde_json::from_str(request).ok()?;
        match request["op"].as_str()? {
            "ee_hit_steps" => Some(crate::health::hit_rows(request["amount"].as_f64().map(|amount| amount as f32)).to_string()),
            "ee_hit_abort" => {
                ABORT.set(true);
                Some("true".to_owned())
            }
            "ee_hit_why" => Some(json!(WHY.get()).to_string()),
            "ee_hit_bolt" => Some(BOLT.get().map_or_else(|| "null".to_owned(), |id| id.to_string())),
            "ee_hit_final" => {
                crate::health::hit_said([request["damage"].as_f64()? as f32, request["absorbed"].as_f64()? as f32]);
                Some("true".to_owned())
            }
            _ => None,
        }
    }
    /// `Entity.removeVehicle` for a mob: true if a listener keeps it on, which nobody can
    /// when `forced`, its vehicle having gone from under it.
    pub fn mob_off(m: &Mob, vehicle: u32, forced: bool) -> bool {
        hears("ee_dismount") && ask("ee_dismount", json!({ "rider": row(m), "mount": seat_row(vehicle), "forced": forced })).0 && !forced
    }
    /// The bolt now striking, for what it strikes.
    pub fn set_bolt(id: Option<u32>) {
        BOLT.set(id);
    }
    /// `BowItem.getPowerForTime`, for `EntityShootBowEvent.getForce` once the arrow is out.
    pub fn set_force(mob: u32, force: f32) {
        FORCES.with(|forces| {
            let mut forces = forces.borrow_mut();
            forces.retain(|row| row.0 != mob);
            forces.push((mob, force));
        });
    }
    pub fn force_of(mob: u32) -> Option<f32> {
        FORCES.with(|forces| {
            let mut forces = forces.borrow_mut();
            let at = forces.iter().position(|row| row.0 == mob)?;
            Some(forces.remove(at).1)
        })
    }
    /// The next effect comes from this one.
    pub fn effect_from(who_by: Option<Who>) {
        let row = who_by.map_or(Value::Null, who);
        EFFECT_FROM.with(|from| *from.borrow_mut() = row);
    }
    pub fn take_effect_from() -> Value {
        EFFECT_FROM.with(|from| from.borrow_mut().take())
    }
    /// `CreatureSpawnEvent.SpawnReason` of the spawn that comes next.
    pub fn why(reason: &'static str) {
        WHY.set(reason);
    }
    pub fn why_now() -> &'static str {
        WHY.get()
    }
    pub fn take_abort() -> bool {
        ABORT.replace(false)
    }
    /// `NaturalSpawner.isValidSpawnPostitionForType`'s event, for each try: 0 goes on, 1 skips this
    /// one and 2 gives the pack up.
    pub fn natural_attempt(kind: &str, at: [f64; 3]) -> u8 {
        if !hears("ee_pre_spawn") {
            return 0;
        }
        why("NATURAL");
        let refused = ask("ee_pre_spawn", json!({ "level": here(), "kind": kind, "at": at, "reason": "NATURAL" })).0;
        if take_abort() {
            2
        } else {
            u8::from(refused)
        }
    }
    /// Who strikes the hit being heard.
    pub fn hit_by(who_by: Option<Who>) {
        let row = who_by.map_or(Value::Null, who);
        HIT_BY.with(|by| *by.borrow_mut() = row);
    }
    pub fn hit_by_row() -> Value {
        HIT_BY.with(|by| by.borrow().clone())
    }
    /// Who a mob's events are by, for what the game does with them after the tick.
    pub fn mark_actor(events: &mut Vec<super::MobEvent>, from: usize, mob: u32) {
        if events.len() > from && (hears("ee_mob") || hears("ee_effect")) {
            events.insert(from, super::MobEvent::Actor(mob));
        }
    }
    /// Holds a death's sound back for `EntityDeathEvent`: true if it's held.
    pub fn hold_voice(m: &Mob, sound: &'static str, volume: f32, pitch: f32) -> bool {
        let held = hears("ee_death");
        if held {
            VOICES.lock().unwrap_or_else(|e| e.into_inner()).push((m.id, sound, volume, pitch));
        }
        held
    }
    pub fn take_voice(mob: u32) -> Option<(&'static str, f32, f32)> {
        let mut voices = VOICES.lock().unwrap_or_else(|e| e.into_inner());
        let at = voices.iter().position(|voice| voice.0 == mob)?;
        let voice = voices.remove(at);
        Some((voice.1, voice.2, voice.3))
    }
}

'''

EDITS = [
    # ---- The way out of a mob's tick.
    (LIB + 'mobs.rs', '''/// `Entity.push` on other mobs acts at once in vanilla: a mob ticking later''', EE + '''/// `Entity.push` on other mobs acts at once in vanilla: a mob ticking later'''),
    (GAME + 'creatures.rs', '''                horizon_client::mobs::apply_mob_pushes(m, &mut pushes);
                horizon_client::mobs::blocks_changed(m, &path_changes);''', '''                horizon_client::mobs::ee::bind(seen_guest.and_then(|conn| guest_ids.iter().find(|g| g.0 == conn).map(|g| g.1 as i32)));
                horizon_client::mobs::apply_mob_pushes(m, &mut pushes);
                horizon_client::mobs::blocks_changed(m, &path_changes);'''),
    (GAME + 'creatures.rs', '''        self.mobs.append(&mut mobs);
        self.died = died;
        for (id, at, width, height) in gone {''', '''        self.mobs.append(&mut mobs);
        self.died = died;
        super::paper_fam_ev_entity::after_mobs(self);
        for (id, at, width, height) in gone {'''),
    (LIB + 'game.rs', '''            .unwrap_or_else(|| {
                r#"{"error":"a query needs an event that has the game"}"#.to_owned()''', '''            .or_else(|| crate::game::paper_fam_ev_entity::free_answer(request))
            .unwrap_or_else(|| {
                r#"{"error":"a query needs an event that has the game"}"#.to_owned()'''),

    # ---- EntityLandEvent: before the fall does anything.
    (MOBS + 'living.rs', '''            if self.fall_distance > 0.0 {
                self.trample_farmland(w, events);
                self.cause_fall_damage(w, events);
            }
            self.fall_distance = 0.0;
        }
    }
    /// `FarmlandBlock.fallOn`''', '''            if self.fall_distance > 0.0 {
                let on = self.block_on_legacy(w);
                if super::ee::landed(self, on, w.block_name(on[0], on[1], on[2])) {
                    self.trample_farmland(w, events);
                    self.cause_fall_damage(w, events);
                }
            }
            self.fall_distance = 0.0;
        }
    }
    /// `FarmlandBlock.fallOn`'''),

    # ---- EntityCollideWithEntityEvent.
    (MOBS + 'ai.rs', '''        if same {
            continue;
        }
        // Entity.push(Entity) with `this` the other entity.''', '''        if same {
            continue;
        }
        if super::ee::collide(m, who) {
            continue;
        }
        // Entity.push(Entity) with `this` the other entity.'''),

    # ---- EntityInsideBlockEvent: each block's `entityInside`.
    (MOBS + 'ai.rs', '''                    if visited.contains(&p) {
                        continue;
                    }
                    crate::block_contact::emit(crate::block_contact::Event::Inside(''', '''                    if visited.contains(&p) {
                        continue;
                    }
                    if super::ee::inside(m, w.block_name(x, y, z), p) {
                        visited.push(p);
                        continue;
                    }
                    crate::block_contact::emit(crate::block_contact::Event::Inside('''),
    (MOBS + 'ai.rs', '''                    if visited.contains(&p) {
                        continue;
                    }
                    match w.block_name(x, y, z) {
                        "minecraft:wither_rose" => {''', '''                    if visited.contains(&p) {
                        continue;
                    }
                    if super::ee::inside(m, w.block_name(x, y, z), p) {
                        visited.push(p);
                        continue;
                    }
                    match w.block_name(x, y, z) {
                        "minecraft:wither_rose" => {'''),
    (MOBS + 'ai.rs', '''    if fire > 0.0 && !m.kind.flags.fire_immune {
        let multiplier = m.burning_time();''', '''    let feet = [m.position.x.floor() as i32, m.position.y.floor() as i32, m.position.z.floor() as i32];
    let fire = if fire > 0.0 && super::ee::inside(m, "minecraft:fire", feet) { 0.0 } else { fire };
    if fire > 0.0 && !m.kind.flags.fire_immune {
        let multiplier = m.burning_time();'''),
    (MOBS + 'ai.rs', '''    if m.fluids.in_lava && !m.kind.flags.fire_immune {
        m.ignite_for_ticks(300);''', '''    if m.fluids.in_lava && !m.kind.flags.fire_immune && !super::ee::inside(m, "minecraft:lava", feet) {
        m.ignite_for_ticks(300);'''),

    # ---- ItemTransportingEntityValidateTargetEvent, ItemTransportingEntitySortEvent.
    (MOBS + 'transport_items.rs', '''        if !(wanted(w, m, &t.name) && unchanged) || blocked(w, t.pos) {''', '''        if !(wanted(w, m, &t.name) && unchanged) || blocked(w, t.pos) || !super::ee::target_allowed(m, t.pos, &t.name) {'''),
    (MOBS + 'transport_items.rs', '''                    if wanted(w, m, &name) && !seen {''', '''                    if wanted(w, m, &name) && !seen && super::ee::target_allowed(m, p, &name) {'''),
    (MOBS + 'transport_items.rs', '''            } else if leaving_ok(m, &items) {
                self.put_down(m, cx, &t.halves, items);''', '''            } else if sort_ok(m, t.pos, &items) {
                self.put_down(m, cx, &t.halves, items);'''),
    (MOBS + 'transport_items.rs', '''    } else if leaving_ok(m, &items) {
        Interaction::PlaceItem''', '''    } else if sort_ok(m, halves[0], &items) {
        Interaction::PlaceItem'''),
    (MOBS + 'transport_items.rs', '''fn leaving_ok(m: &Mob, items: &[Option<ItemStack>]) -> bool {''', '''fn sort_ok(m: &Mob, at: [i32; 3], items: &[Option<ItemStack>]) -> bool {
    super::ee::sorted(m, held(m), at, items, leaving_ok(m, items))
}
fn leaving_ok(m: &Mob, items: &[Option<ItemStack>]) -> bool {'''),

    # ---- EntityDeathEvent's sound: `playDeathSound` runs after the event.
    (LIB + 'mobs.rs', '''                if !self.silent {
                    let at = [self.position.x, self.position.y, self.position.z];
                    self.outbox
                        .push(MobEvent::Sound(sound, at, ai::sound_volume(self), pitch));
                }''', '''                if !self.silent {
                    let at = [self.position.x, self.position.y, self.position.z];
                    if !(dead && ee::hold_voice(self, sound, ai::sound_volume(self), pitch)) {
                        self.outbox
                            .push(MobEvent::Sound(sound, at, ai::sound_volume(self), pitch));
                    }
                }'''),

    # ---- EntityDamageItemEvent: what a mob wears.
    (GAME + 'creatures.rs', '''            let loss = self.durability_loss(&stack, amount);
            if loss == 0 {
                continue;
            }''', '''            let loss = self.durability_loss(&stack, amount);
            let loss = super::paper_fam_ev_entity::item_wear(self, id, &stack, loss);
            if loss == 0 {
                continue;
            }'''),
    (GAME + 'creatures.rs', '''                let loss = self.durability_loss(&body, amount);
                if loss > 0 {''', '''                let loss = self.durability_loss(&body, amount);
                let loss = super::paper_fam_ev_entity::item_wear(self, id, &body, loss);
                if loss > 0 {'''),
    (GAME + 'creatures.rs', '''        let loss = self.durability_loss(&stack, amount);
        if loss == 0 {
            return;
        }''', '''        let loss = self.durability_loss(&stack, amount);
        let loss = super::paper_fam_ev_entity::item_wear(self, id, &stack, loss);
        if loss == 0 {
            return;
        }'''),
    (LIB + 'mobs.rs', '''        let before = self.extra.body_damage;
        let after = before + damage.ceil() as i32;''', '''        let before = self.extra.body_damage;
        let wear = match self.item_by_slot(horizon_world::player_inventory::EquipmentSlot::Body) {
            Some(item) => ee::wear(self, &item, damage.ceil() as i32),
            None => damage.ceil() as i32,
        };
        let after = before + wear;'''),

    # ---- EntityLungeEvent's power.
    (GAME + 'enchant_effects.rs', '''                effects.push((*level, entry["effect"].clone()));''', '''                effects.push((super::paper_fam_ev_entity::lunge_power(&id.to_string(), *level), entry["effect"].clone()));'''),

    # ---- PlayerNaturallySpawnCreaturesEvent: a radius for each player, and none for one refused.
    (GAME + 'natural_spawning.rs', '''            for dx in -8..=8 {
                for dz in -8..=8 {
                    let key = (pcx + dx, pcz + dz);''', '''            let reach = super::paper_fam_ev_entity::spawn_reach(p.id);
            for dx in -reach..=reach {
                for dz in -reach..=reach {
                    let key = (pcx + dx, pcz + dz);'''),

    # ---- EntityToggleSwimEvent.
    (LIB + 'motion/fluids.rs', '''        self.player.swimming = !self.player.flying
            && !self.player.is_passenger''', '''        let swimming = !self.player.flying
            && !self.player.is_passenger'''),
    (LIB + 'motion/fluids.rs', '''                        WATER,
                    )
            };
        self.tick_remaining_fire()?;''', '''                        WATER,
                    )
            };
        if swimming == self.player.swimming || !crate::mobs::ee::swim_refused(swimming) {
            self.player.swimming = swimming;
        }
        self.tick_remaining_fire()?;'''),

    # ---- EntityDyeEvent, SheepDyeWoolEvent: the dye's own moment.
    (GAME + 'creatures.rs', '''                    m.extra.color = c as u8;
                    consume = true;
                    sound = Some("item.dye.use");
                } else {
                    handled = false;
                }''', '''                    match horizon_client::mobs::ee::dye(m, c as u8, acting) {
                        Some(color) => {
                            m.extra.color = color;
                            consume = true;
                            sound = Some("item.dye.use");
                        }
                        None => handled = false,
                    }
                } else {
                    handled = false;
                }'''),
    (GAME + 'creatures.rs', '''                    m.extra.color = c as u8;
                }
                consume = true;
            }
            Species::Wolf if m.extra.tame && m.extra.owner == Some(acting)''', '''                    match horizon_client::mobs::ee::dye(m, c as u8, acting) {
                        Some(color) => {
                            m.extra.color = color;
                            consume = true;
                        }
                        None => handled = false,
                    }
                }
            }
            Species::Wolf if m.extra.tame && m.extra.owner == Some(acting)'''),
    (GAME + 'creatures.rs', '''                    m.extra.color = c as u8;
                }
                m.persistent = true;
                consume = true;
            }''', '''                    match horizon_client::mobs::ee::dye(m, c as u8, acting) {
                        Some(color) => {
                            m.extra.color = color;
                            m.persistent = true;
                            consume = true;
                        }
                        None => handled = false,
                    }
                }
            }'''),

    # ---- The portal's `Plan` takes what a plugin set.
    (GAME + 'nether_portal.rs', '''    /// Through an End portal: `approximate` is the End's spawn point, or the
    /// Overworld's shared spawn.
    pub(super) end: bool,
}
impl Plan {''', '''    /// Through an End portal: `approximate` is the End's spawn point, or the
    /// Overworld's shared spawn.
    pub(super) end: bool,
    pub(super) rules: PortalRules,
}
/// What a listener of `EntityPortalEvent` set on a traveller: the game's own until one does.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct PortalRules {
    pub(super) search: Option<i32>,
    pub(super) creation: Option<i32>,
    pub(super) no_portal: bool,
    /// `approximate` is where it lands, as it is.
    pub(super) exact: bool,
    /// Where it left from, should its way turn out to be closed.
    pub(super) back: Option<(Dim, BlockPos)>,
}
impl Plan {'''),
    (GAME + 'nether_portal.rs', '''            offset: Vec3::ZERO,
            width,
            height,
            end: true,
        }''', '''            offset: Vec3::ZERO,
            width,
            height,
            end: true,
            rules: PortalRules::default(),
        }'''),
    (GAME + 'nether_portal.rs', '''            end: int("end").unwrap_or(0) != 0,
        },''', '''            end: int("end").unwrap_or(0) != 0,
            rules: PortalRules::default(),
        },'''),
    (GAME + 'nether_portal.rs', '''            offset,
            width,
            height,
            end: false,
        }''', '''            offset,
            width,
            height,
            end: false,
            rules: PortalRules::default(),
        }'''),
    (GAME + 'nether_portal.rs', '''    fn entity_plan(
        &self,
        entry: BlockPos,
        at: Vec3,
        width: f32,
        height: f32,
        reset_forward: bool,
    ) -> Plan {
        if !self.is_end_portal''', '''    fn entity_plan(
        &self,
        entry: BlockPos,
        at: Vec3,
        width: f32,
        height: f32,
        reset_forward: bool,
    ) -> Plan {
        let mut plan = self.entity_plan_own(entry, at, width, height, reset_forward);
        super::paper_fam_ev_entity::portal_answer(&mut plan);
        plan
    }
    fn entity_plan_own(
        &self,
        entry: BlockPos,
        at: Vec3,
        width: f32,
        height: f32,
        reset_forward: bool,
    ) -> Plan {
        if !self.is_end_portal'''),
    (GAME + 'nether_portal.rs', '''        let radius = if to_nether { 16 } else { 128 };
        let border = self.border_bounds();''', '''        let radius = plan.rules.search.unwrap_or(if to_nether { 16 } else { 128 });
        let border = self.border_bounds();'''),
    (GAME + 'nether_portal.rs', '''        let logical = if to_nether { 128 } else { 384 };
        let (approximate, x_axis) = (plan.approximate, plan.entry_x_axis);
        let created = self''', '''        if plan.rules.no_portal {
            return None;
        }
        let logical = if to_nether { 128 } else { 384 };
        let (approximate, x_axis) = (plan.approximate, plan.entry_x_axis);
        if let Some(radius) = plan.rules.creation {
            horizon_world::level::portal_travel::CREATION_RADIUS.store(radius, std::sync::atomic::Ordering::Relaxed);
        }
        let created = self'''),
    (GAME + 'nether_portal.rs', '''        let _ = self.process_level_events();
        if let Some(exit) = created {
            let (at, _) = self.arrival(plan, exit);''', '''        if plan.rules.creation.is_some() {
            horizon_world::level::portal_travel::CREATION_RADIUS.store(16, std::sync::atomic::Ordering::Relaxed);
        }
        let _ = self.process_level_events();
        if let Some(exit) = created {
            let (at, _) = self.arrival(plan, exit);'''),
    (GAME + 'nether_portal.rs', '''            let (at, motion, [yaw, pitch]) = if plan.end {
                let at = self.end_portal_exit(&plan);''', '''            let (at, motion, [yaw, pitch]) = if plan.rules.exact {
                let p = plan.approximate;
                (Vec3::new(f64::from(p.x) + 0.5, f64::from(p.y), f64::from(p.z) + 0.5), motion, [yaw, pitch])
            } else if plan.end {
                let at = self.end_portal_exit(&plan);'''),
    (GAME + 'nether_portal.rs', '''                let Some(exit) = self.nether_exit_portal(&plan) else {
                    continue;
                };
                let (at, rotation) = self.arrival(&plan, exit);
                (at, rotate_y(motion, -rotation), [yaw + rotation, pitch])
            };
            if let Tag::Compound(c) = &mut tag {''', '''                let Some(exit) = self.nether_exit_portal(&plan) else {
                    // `setCanCreatePortal(false)` with no portal found: it never left.
                    if let Some((dim, approximate)) = plan.rules.back {
                        let rules = PortalRules { exact: true, ..PortalRules::default() };
                        self.portals.arrivals.push(Arrival { plan: Plan { dim, approximate, rules, ..plan }, tag });
                    }
                    continue;
                };
                let (at, rotation) = self.arrival(&plan, exit);
                (at, rotate_y(motion, -rotation), [yaw + rotation, pitch])
            };
            let (at, motion) = super::paper_fam_ev_entity::portal_exit(self, &tag, plan.rules, at, motion);
            if let Tag::Compound(c) = &mut tag {'''),

    # ---- A player's heal: their food, their peace, and what they drank.
    (LIB + 'hunger.rs', '''            if health < maximum {
                let h = &mut self.health;
                let mut observer = Updates(&mut self.events);''', '''            if health < maximum && let Some(amount) = crate::mobs::ee::player_regain(1.0, "REGEN", false) {
                let h = &mut self.health;
                let mut observer = Updates(&mut self.events);'''),
    (LIB + 'hunger.rs', '''                .heal(1.0)?;''', '''                .heal(amount)?;'''),
    (LIB + 'hunger.rs', '''    fn heal(&mut self, _: &mut FoodData, amount: f32) -> Result<()> {
        let mut observer = Updates(self.events);''', '''    fn heal(&mut self, food: &mut FoodData, amount: f32) -> Result<()> {
        // `FoodData.tick`'s first branch is the fast one: full, with saturation left.
        let Some(amount) = crate::mobs::ee::player_regain(amount, "SATIATED", food.food >= 20 && food.saturation > 0.0) else {
            return Ok(());
        };
        let mut observer = Updates(self.events);'''),
    (GAME + 'potions.rs', '''                let h = p.health.health();
                if h > 0.0 {
                    let _ = p.set_health((h + amount).min(max), false);
                }''', '''                let h = p.health.health();
                let reason = horizon_client::mobs::ee::take_heal_reason();
                if h > 0.0 && let Some(amount) = horizon_client::mobs::ee::player_regain(amount, reason, false) {
                    let _ = p.set_health((h + amount).min(max), false);
                }'''),
    (GAME + 'potions.rs', '''                                m.health = (m.health + (4 << amp.min(30)) as f32)
                                    .min(m.attrs.max_health);''', '''                                horizon_client::mobs::ee::heal(m, (4 << amp.min(30)) as f32, "MAGIC");'''),
    (GAME + 'wither.rs', '''                self.apply_effect_action(
                    horizon_client::effects::EffectAction::Heal(5.0),
                );''', '''                horizon_client::mobs::ee::heal_as("WITHER");
                self.apply_effect_action(
                    horizon_client::effects::EffectAction::Heal(5.0),
                );'''),
    (GAME + 'lan_actions.rs', '''        let swap = self.enter_guest(host, conn)?;
''', '''        let body = host.hub.guest(conn).map(|g| g.entity_id);
        let swap = self.enter_guest(host, conn)?;
        horizon_client::mobs::ee::bind(body);
'''),
    (GAME + 'lan_actions.rs', '''        self.leave_guest(host, swap);
''', '''        horizon_client::mobs::ee::bind(None);
        self.leave_guest(host, swap);
'''),

    # ---- Who a mob's events are by.
    (LIB + 'mobs.rs', '''    SetTarget { mob: u32, target: Who },''', '''    SetTarget { mob: u32, target: Who },
    /// The events up to the next of these are this mob's.
    Actor(u32),'''),
    (GAME + 'creatures.rs', '''                horizon_client::mobs::apply_brain_events(&mut mobs, &mut events, from);
            }
            let mut flags = keep.into_iter();''', '''                horizon_client::mobs::apply_brain_events(&mut mobs, &mut events, from);
                horizon_client::mobs::ee::mark_actor(&mut events, from, mobs[i].id);
            }
            let mut flags = keep.into_iter();'''),
    (GAME + 'creatures.rs', '''                MobEvent::SetBlock { pos, state } => self.level_set(pos, state)?,
                MobEvent::CropRaided { pos, state, old, mob } => {''', '''                MobEvent::Actor(mob) => horizon_client::mobs::ee::set_actor(mob),
                MobEvent::SetBlock { pos, state } => self.level_set(pos, state)?,
                MobEvent::CropRaided { pos, state, old, mob } => {'''),

    # ---- EntityRegainHealthEvent: every heal, before it, with its reason.
    ('horizon-client/src/mobs/ai.rs', '                m.health = (m.health + 1.0).min(m.attrs.max_health);', '                super::ee::heal(m, 1.0, "MAGIC_REGEN");'),
    ('horizon-client/src/mobs/allay.rs', '        m.health = (m.health + 1.0).min(m.attrs.max_health);', '        super::ee::heal(m, 1.0, "REGEN");'),
    ('horizon-client/src/mobs/equine.rs', '        m.health = (m.health + 1.0).min(m.attrs.max_health);', '        super::ee::heal(m, 1.0, "REGEN");'),
    ('horizon-client/src/mobs/happy_ghast.rs', '            m.health = (m.health + 1.0).min(m.attrs.max_health);', '            super::ee::heal(m, 1.0, "REGEN");'),
    ('horizon-client/src/mobs/llama.rs', '        m.health = (m.health + 1.0).min(m.attrs.max_health);', '        super::ee::heal(m, 1.0, "REGEN");'),
    ('horizon-client/src/mobs/witch.rs', '                    "minecraft:healing" => m.health = (m.health + 4.0).min(m.attrs.max_health),', '                    "minecraft:healing" => super::ee::heal(m, 4.0, "MAGIC"),'),
    ('horizon-client/src/mobs/wither.rs', '            m.health = (m.health + 10.0).min(m.attrs.max_health);', '            super::ee::heal(m, 10.0, "WITHER_SPAWN");'),
    ('horizon-client/src/mobs/wither.rs', '        m.health = (m.health + 1.0).min(m.attrs.max_health);', '        super::ee::heal(m, 1.0, "REGEN");'),
    ('horizon-client/src/game/creatures.rs', '                            m.health = (m.health + heal as f32).min(m.attrs.max_health);', '                            horizon_client::mobs::ee::heal(m, heal as f32, "MAGIC");'),
    ('horizon-client/src/game/creatures.rs', '                    .map_or(1.0, |f| f.0 as f32);\n                m.health = (m.health + heal).min(m.attrs.max_health);', '                    .map_or(1.0, |f| f.0 as f32);\n                horizon_client::mobs::ee::heal(m, heal, "EATING");'),
    ('horizon-client/src/game/creatures.rs', '                m.health = (m.health + heal).min(m.attrs.max_health);', '                horizon_client::mobs::ee::heal(m, heal, "EATING");'),
    ('horizon-client/src/game/creatures.rs', '                m.health = (m.health + 25.0).min(m.attrs.max_health);', '                horizon_client::mobs::ee::heal(m, 25.0, "CUSTOM");'),
    ('horizon-client/src/game/llamas.rs', '            m.health = (m.health + heal).min(m.attrs.max_health);', '            horizon_client::mobs::ee::heal(m, heal, "EATING");'),
    ('horizon-client/src/game/mounts.rs', '            m.health = (m.health + heal).min(m.attrs.max_health);', '            horizon_client::mobs::ee::heal(m, heal, "EATING");'),
    ('horizon-client/src/game/nautili.rs', '                m.health = (m.health + 2.0).min(m.attrs.max_health);', '                horizon_client::mobs::ee::heal(m, 2.0, "EATING");'),
    ('horizon-client/src/game/wither.rs', '                    w.health = (w.health + 5.0).min(w.attrs.max_health);', '                    horizon_client::mobs::ee::heal(w, 5.0, "WITHER");'),
    ('horizon-client/src/game/potions.rs', '                                m.health = (m.health + amount(4)).min(m.kind.max_health);', '                                if let Some(amount) = horizon_client::mobs::ee::regain(m, amount(4), "MAGIC") {\n                                    m.health = (m.health + amount).min(m.kind.max_health);\n                                }'),
    ('horizon-client/src/game/dragon.rs', '            d.health = (d.health + 1.0).min(MAX_HEALTH);', '            if let Some(amount) = horizon_client::mobs::ee::dragon_regain(d.id, 1.0) {\n                d.health = (d.health + amount).min(MAX_HEALTH);\n            }'),

    # ---- The one way a target changes.
    (LIB + 'mobs.rs', '''    /// `Entity.extinguishFire`: a burning mob first plays the extinguish sound''', '''    /// `Mob.setTarget`: nothing happens for the target it has.
    pub fn set_ai_target(&mut self, target: Option<Who>, reason: &'static str) {
        if self.ai.target != target {
            ee::target(self, target, reason, |m, target| m.ai.target = target);
        }
    }
    /// `Entity.extinguishFire`: a burning mob first plays the extinguish sound'''),
    (MOBS + 'monster_brain.rs', '''    mem(m).attack_target.set_opt(who);
    m.ai.target = who;
}''', '''    if m.ai.target != who {
        super::ee::target(m, who, "CLOSEST", |m, who| {
            mem(m).attack_target.set_opt(who);
            m.ai.target = who;
        });
    }
}'''),
    (MOBS + 'animal_brain.rs', '''            mem(m).attack_target.set(target);
            mem(m).cant_reach_walk_target_since.clear();
            m.ai.target = Some(target);
            true''', '''            mem(m).cant_reach_walk_target_since.clear();
            super::ee::target(m, Some(target), "CLOSEST", |m, who| {
                mem(m).attack_target.set_opt(who);
                m.ai.target = who;
            });
            true'''),
    (MOBS + 'animal_brain.rs', '''                mem(m).attack_target.clear();
                m.ai.target = None;
            }
            true''', '''                super::ee::target(m, None, "TARGET_DIED", |m, who| {
                    mem(m).attack_target.set_opt(who);
                    m.ai.target = who;
                });
            }
            true'''),
    (MOBS + 'warden_brain.rs', '''pub(super) fn set_attack_target(m: &mut Mob, target: Who) {
    let mm = mem(m);
    mm.roar_target.clear();
    mm.attack_target.set(target);
    mm.cant_reach_walk_target_since.clear();
    mm.sonic_boom_cooldown.set_ttl((), 200);
}''', '''pub(super) fn set_attack_target(m: &mut Mob, target: Who) {
    super::ee::target(m, Some(target), "CLOSEST", |m, target| {
        let mm = mem(m);
        mm.roar_target.clear();
        mm.attack_target.set_opt(target);
        mm.cant_reach_walk_target_since.clear();
        mm.sonic_boom_cooldown.set_ttl((), 200);
    });
}'''),

    # ---- EntityTargetEvent: every target a mob takes or drops goes through `Mob::set_ai_target`, with its reason.
    ('horizon-client/src/game/beehives.rs', '                    } else {\n                        m.ai.target = Some(Who::Player);', '                    } else {\n                        m.set_ai_target(Some(Who::Player), "CLOSEST_PLAYER");'),
    ('horizon-client/src/game/beehives.rs', '                        m.ai.target = Some(Who::Player);', '                        m.set_ai_target(Some(Who::Player), "CLOSEST_PLAYER");'),
    ('horizon-client/src/game/beehives.rs', '            self.mobs[i].ai.target = Some(Who::Player);', '            self.mobs[i].set_ai_target(Some(Who::Player), "CLOSEST_PLAYER");'),
    ('horizon-client/src/game/creatures.rs', '                    if let Some(m) = self.mobs.iter_mut().find(|m| m.id == mob) {\n                        m.ai.target = Some(target);\n                        m.ai.aggressive |= aggressive;', '                    if let Some(m) = self.mobs.iter_mut().find(|m| m.id == mob) {\n                        m.set_ai_target(Some(target), "CLOSEST");\n                        m.ai.aggressive |= aggressive;'),
    ('horizon-client/src/game/creatures.rs', '                        m.ai.target = Some(target);', '                        m.set_ai_target(Some(target), "TARGET_ATTACKED_NEARBY_ENTITY");'),
    ('horizon-client/src/game/creatures.rs', '                            m.ai.target = Some(Who::Mob(target));', '                            m.set_ai_target(Some(Who::Mob(target)), "CLOSEST_ENTITY");'),
    ('horizon-client/src/game/creatures.rs', '                    m.ai.target = None;', '                    m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/game/creatures.rs', '                m.ai.target = None;', '                m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/game/nautili.rs', '                    m.ai.target = None;', '                    m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/game/piglins.rs', '            m.ai.target = Some(Who::Player);', '            m.set_ai_target(Some(Who::Player), "CLOSEST_PLAYER");'),
    ('horizon-client/src/game/zombie_reinforcements.rs', '            reinforcement.ai.target = Some(target);', '            reinforcement.set_ai_target(Some(target), "REINFORCEMENT_TARGET");'),
    ('horizon-client/src/mobs/ai.rs', '                m.extra.anger = 0;\n                m.ai.target = None;', '                m.extra.anger = 0;\n                m.set_ai_target(None, "TARGET_DIED");'),
    ('horizon-client/src/mobs/ai.rs', '                m.ai.target = None;', '                m.set_ai_target(None, "TARGET_DIED");'),
    ('horizon-client/src/mobs/bee.rs', '    m.ai.target = None;', '    m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/caravan.rs', '        m.ai.target = self.attacker;', '        m.set_ai_target(self.attacker, "TARGET_ATTACKED_OWNER");'),
    ('horizon-client/src/mobs/caravan.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/enderman.rs', '        self.pending = false;\n        m.ai.target = None;', '        self.pending = false;\n        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/enderman.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/enderman.rs', '                m.ai.target = Some(Who::Player);', '                m.set_ai_target(Some(Who::Player), "CLOSEST_PLAYER");'),
    ('horizon-client/src/mobs/fox.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/goals.rs', '        }\n    }\n    fn stop(&mut self, m: &mut Mob, _: &mut Cx) {\n        m.ai.target = None;\n    }\n}', '        }\n    }\n    fn stop(&mut self, m: &mut Mob, _: &mut Cx) {\n        m.set_ai_target(None, "FORGOT_TARGET");\n    }\n}'),
    ('horizon-client/src/mobs/goals.rs', '        if cub {\n            m.ai.target = None;', '        if cub {\n            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/goals.rs', '    fn stop(&mut self, m: &mut Mob, _: &mut Cx) {\n        m.ai.target = None;', '    fn stop(&mut self, m: &mut Mob, _: &mut Cx) {\n        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/goals.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/goals.rs', '        m.ai.target = self.found;', '        m.set_ai_target(self.found, "CLOSEST");'),
    ('horizon-client/src/mobs/goals.rs', '        m.ai.target = m.ai.last_hurt_by;', '        m.set_ai_target(m.ai.last_hurt_by, "TARGET_ATTACKED_ENTITY");'),
    ('horizon-client/src/mobs/golem.rs', '        m.ai.target = Some(Who::Player);', '        m.set_ai_target(Some(Who::Player), "DEFEND_VILLAGE");'),
    ('horizon-client/src/mobs/golem.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/golem.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/guardian.rs', '            super::ai::do_hurt_target(m, who, cx);\n            m.ai.target = None;', '            super::ai::do_hurt_target(m, who, cx);\n            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/guardian.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/guardian.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/hostile.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/illager.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/nautilus.rs', '        m.ai.target = find_target(m, w);', '        let found = find_target(m, w);\n        m.set_ai_target(found, "CLOSEST");'),
    ('horizon-client/src/mobs/nautilus.rs', '        }\n        m.ai.target = None;', '        }\n        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/nautilus.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/panda.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/phantom.rs', '        if near && can_attack_seen(m, cx.w) {\n            m.ai.target = Some(Who::Player);', '        if near && can_attack_seen(m, cx.w) {\n            m.set_ai_target(Some(Who::Player), "CLOSEST_PLAYER");'),
    ('horizon-client/src/mobs/phantom.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/piglin.rs', '        if m.extra.anger == 0 && m.ai.target == Some(Who::Player) && !hostile_to_player(m, w) {\n            m.ai.target = None;', '        if m.extra.anger == 0 && m.ai.target == Some(Who::Player) && !hostile_to_player(m, w) {\n            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/piglin.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/piglin.rs', '        m.extra.ints[0] = 200;\n        m.ai.target = None;', '        m.extra.ints[0] = 200;\n        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/piglin.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/piglin.rs', '    m.ai.target = None;', '    m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/piglin.rs', '        m.ai.target = Some(Who::Mob(hoglin));', '        m.set_ai_target(Some(Who::Mob(hoglin)), "CLOSEST_ENTITY");'),
    ('horizon-client/src/mobs/polar_bear.rs', '    m.ai.target = None;', '    m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/shulker.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/shulker.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/tame.rs', '        if let Some((who, at)) = owner_view(m, cx.w).and_then(|p| p.hurt_by) {\n            m.ai.target = Some(who);', '        if let Some((who, at)) = owner_view(m, cx.w).and_then(|p| p.hurt_by) {\n            m.set_ai_target(Some(who), "TARGET_ATTACKED_OWNER");'),
    ('horizon-client/src/mobs/tame.rs', '            m.ai.target = Some(who);', '            m.set_ai_target(Some(who), "OWNER_ATTACKED_TARGET");'),
    ('horizon-client/src/mobs/tame.rs', '    m.ai.last_hurt_by_time = m.age as i32;\n    m.ai.target = None;', '    m.ai.last_hurt_by_time = m.age as i32;\n    m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/tame.rs', '        }\n    }\n    fn stop(&mut self, m: &mut Mob, _: &mut Cx) {\n        m.ai.target = None;\n    }\n}\n/// `Wolf.wantsToAttack`: never creepers, ghasts, tame cats and parrots, or a', '        }\n    }\n    fn stop(&mut self, m: &mut Mob, _: &mut Cx) {\n        m.set_ai_target(None, "FORGOT_TARGET");\n    }\n}\n/// `Wolf.wantsToAttack`: never creepers, ghasts, tame cats and parrots, or a'),
    ('horizon-client/src/mobs/tame.rs', '    fn stop(&mut self, m: &mut Mob, _: &mut Cx) {\n        m.ai.target = None;', '    fn stop(&mut self, m: &mut Mob, _: &mut Cx) {\n        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/tame.rs', '    fn tick(&mut self, m: &mut Mob, cx: &mut Cx) {\n        m.ai.target = None;', '    fn tick(&mut self, m: &mut Mob, cx: &mut Cx) {\n        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/tame.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/undead.rs', '            m.ai.target = None;', '            m.set_ai_target(None, "FORGOT_TARGET");'),
    ('horizon-client/src/mobs/witch.rs', '        m.ai.target = None;', '        m.set_ai_target(None, "FORGOT_TARGET");'),

    # ---- What a mob does in its tick, before it does it.
    (MOBS + 'living.rs', '''        // LivingEntity.jumpFromGround (and the rabbit's and cubes' own,
        // which call it or do the same): needsSync.
        self.needs_sync = true;''', '''        if super::ee::stop("ee_jump", self, serde_json::json!({})) {
            self.ai.jumping = false;
            return;
        }
        // LivingEntity.jumpFromGround (and the rabbit's and cubes' own,
        // which call it or do the same): needsSync.
        self.needs_sync = true;'''),
    (MOBS + 'turtle.rs', '''    fn start(&mut self, m: &mut Mob, _cx: &mut Cx) {
        set_going_home(m, true);
        self.stuck = false;''', '''    fn start(&mut self, m: &mut Mob, _cx: &mut Cx) {
        if super::ee::stop("ee_turtle_home", m, serde_json::json!({})) {
            self.stuck = true;
            return;
        }
        set_going_home(m, true);
        self.stuck = false;'''),
    (MOBS + 'turtle.rs', '''        if m.extra.counter < 1 {
            set_laying_egg(m, true);
        } else if m.extra.counter > 200 {''', '''        if m.extra.counter < 1 {
            let at = self.0.block;
            let dig = !super::ee::stop("ee_turtle_dig", m, serde_json::json!({ "at": [at[0], at[1] + 1, at[2]] }));
            set_laying_egg(m, dig);
        } else if m.extra.counter > 200 {'''),
    (MOBS + 'strider.rs', '''    m.extra.flag = !in_warm && !on_warm_strider;
}''', '''    let cold = !in_warm && !on_warm_strider;
    if cold == m.extra.flag || !super::ee::stop("ee_strider", m, serde_json::json!({ "shivering": cold })) {
        m.extra.flag = cold;
    }
}'''),
    (MOBS + 'cube.rs', '''    fn can_use(&mut self, m: &mut Mob, _: &mut Cx) -> bool {
        m.is_in_liquid()
    }''', '''    fn can_use(&mut self, m: &mut Mob, _: &mut Cx) -> bool {
        m.is_in_liquid() && !super::ee::stop("ee_slime_swim", m, serde_json::json!({}))
    }'''),
    (MOBS + 'cube.rs', '''            self.degrees = m.random.next_int_bound(360).unwrap_or(0) as f32;
        }
        set_direction(m, self.degrees, false);''', '''            let yaw = m.random.next_int_bound(360).unwrap_or(0) as f32;
            let Some(after) = super::ee::say("ee_slime_turn", m, serde_json::json!({ "yaw": yaw })) else {
                return;
            };
            self.degrees = after["yaw"].as_f64().map_or(yaw, |yaw| yaw as f32);
        }
        set_direction(m, self.degrees, false);'''),
    (MOBS + 'cube.rs', '''    fn can_use(&mut self, m: &mut Mob, _: &mut Cx) -> bool {
        m.extra.plugin_flag("wander") != Some(false)
    }
    fn tick(&mut self, m: &mut Mob, _: &mut Cx) {
        set_wanted(m, 1.0);''', '''    fn can_use(&mut self, m: &mut Mob, _: &mut Cx) -> bool {
        m.extra.plugin_flag("wander") != Some(false) && !super::ee::stop("ee_slime_wander", m, serde_json::json!({}))
    }
    fn tick(&mut self, m: &mut Mob, _: &mut Cx) {
        set_wanted(m, 1.0);'''),
    (MOBS + 'witch.rs', '''            if let Some(potion) = m.extra.take_main_hand() {
                match potion.as_ref() {''', '''            let held = m.extra.take_main_hand();
            if let Some(potion) = super::ee::witch_potion("ee_witch_drink", m, held.as_deref()) {
                match potion.as_str() {'''),
    (MOBS + 'witch.rs', '''                    "minecraft:swiftness" => m.add_effect("minecraft:speed", 3600, 0),
                    _ => {}''', '''                    "minecraft:swiftness" => m.add_effect("minecraft:speed", 3600, 0),
                    other => super::ee::drink(m, other),'''),
    (MOBS + 'witch.rs', '''    if let Some(p) = potion {
        m.extra.set_main_hand(Some(p.into()));''', '''    if let Some(p) = super::ee::witch_potion("ee_witch_ready", m, potion) {
        m.extra.set_main_hand(Some(p.as_str().into()));'''),
    (MOBS + 'illager.rs', '''        self.warmup -= 1;
        if self.warmup != 0 {
            return;
        }
        match self.spell {''', '''        self.warmup -= 1;
        if self.warmup != 0 {
            return;
        }
        if super::ee::stop("ee_spell", m, serde_json::json!({ "spell": self.spell as u8 })) {
            return;
        }
        match self.spell {'''),
    (MOBS + 'nav.rs', '''        if targets.is_empty()
            || self.position.y < f64::from(w.world.bottom_y())
            || !self.can_update_path()
        {
            return None;
        }''', '''        if targets.is_empty()
            || self.position.y < f64::from(w.world.bottom_y())
            || !self.can_update_path()
        {
            return None;
        }
        let heading = self.ai.target.filter(|_| super::ee::take_path_to_entity()).map_or(serde_json::Value::Null, super::ee::who);
        if super::ee::stop("ee_pathfind", self, serde_json::json!({ "at": targets[0], "target": heading })) {
            return None;
        }'''),
    (MOBS + 'nav.rs', '''        let p = [
            at[0].floor() as i32,
            at[1].floor() as i32,
            at[2].floor() as i32,
        ];
        self.create_path_to(w, p, reach)''', '''        let p = [
            at[0].floor() as i32,
            at[1].floor() as i32,
            at[2].floor() as i32,
        ];
        super::ee::path_to_entity();
        self.create_path_to(w, p, reach)'''),

    # ---- EndermanEscapeEvent, EntityTeleportEvent.
    (MOBS + 'enderman.rs', '''        {
            m.set_ai_target(None, "FORGOT_TARGET");
            teleport_random(m, cx.w, cx.events);
        }
    }
    // Being angry (target set)''', '''        {
            if !super::ee::stop("ee_enderman_escape", m, serde_json::json!({ "reason": "RUNAWAY" })) {
                m.set_ai_target(None, "FORGOT_TARGET");
                teleport_random(m, cx.w, cx.events);
            }
        }
    }
    // Being angry (target set)'''),
    (MOBS + 'enderman.rs', '''        let tries = m.extra.counter2;
        m.extra.counter2 = 0;
        for _ in 0..tries {
            if teleport_random(m, w, events) {
                break;
            }
        }
    }''', '''        let tries = m.extra.counter2;
        m.extra.counter2 = 0;
        let reason = if tries >= 64 {
            "INDIRECT"
        } else if m.fluids.in_water || super::ai::in_rain(m, w) {
            "DROWN"
        } else {
            "CRITICAL_HIT"
        };
        if !super::ee::stop("ee_enderman_escape", m, serde_json::json!({ "reason": reason })) {
            for _ in 0..tries {
                if teleport_random(m, w, events) {
                    break;
                }
            }
        }
    }'''),
    (MOBS + 'enderman.rs', '''                if m.distance_sqr(p.position) < 16.0 {
                    teleport_random(m, cx.w, cx.events);
                }''', '''                if m.distance_sqr(p.position) < 16.0
                    && !super::ee::stop("ee_enderman_escape", m, serde_json::json!({ "reason": "STARE" }))
                {
                    teleport_random(m, cx.w, cx.events);
                }'''),
    (MOBS + 'tame.rs', '''        if can_teleport_to(m, w, p) {
            m.position = Vec3::new(
                f64::from(p[0]) + 0.5,
                f64::from(p[1]),
                f64::from(p[2]) + 0.5,
            );
            m.previous = m.position;''', '''        if can_teleport_to(m, w, p) {
            let Some(to) = super::ee::teleport(m, [f64::from(p[0]) + 0.5, f64::from(p[1]), f64::from(p[2]) + 0.5]) else {
                return;
            };
            m.position = Vec3::new(to[0], to[1], to[2]);
            m.previous = m.position;'''),

    # ---- EntityCombustEvent and its two kinds.
    (MOBS + 'ai.rs', '''    if fire > 0.0 && !m.kind.flags.fire_immune {
        let multiplier = m.burning_time();
        m.fire_ticks = crate::motion::environment::ignite_contact(
            m.fire_ticks,
            false,
            &mut m.random,
            multiplier,
        );
        m.damage(fire, Damage::of(DamageKind::InFire));
    }''', '''    if fire > 0.0 && !m.kind.flags.fire_immune {
        let multiplier = m.burning_time();
        if m.fire_ticks > 0 || super::ee::combust(m, 160, super::ee::block_by(m, "minecraft:fire", feet)).is_some() {
            m.fire_ticks = crate::motion::environment::ignite_contact(
                m.fire_ticks,
                false,
                &mut m.random,
                multiplier,
            );
        }
        m.damage(fire, Damage::of(DamageKind::InFire));
    }'''),
    (MOBS + 'ai.rs', '''    if m.fluids.in_lava && !m.kind.flags.fire_immune && !super::ee::inside(m, "minecraft:lava", feet) {
        m.ignite_for_ticks(300);''', '''    if m.fluids.in_lava && !m.kind.flags.fire_immune && !super::ee::inside(m, "minecraft:lava", feet) {
        if let Some(ticks) = super::ee::combust(m, 300, super::ee::block_by(m, "minecraft:lava", feet)) {
            m.ignite_for_ticks(ticks);
        }'''),
    (MOBS + 'ai.rs', '''            m.ignite_for_ticks(160);''', '''            if let Some(ticks) = super::ee::combust(m, 160, serde_json::Value::Null) {
                m.ignite_for_ticks(ticks);
            }'''),

    # ---- EntityEnterLoveModeEvent.
    (MOBS + 'ai.rs', '''    if m.extra.age == 0 {
        m.extra.in_love = 600;
        m.extra.love_player = true;
        // Animal.setInLove: the player who fed it, for a fox baby to trust.
        m.extra.love_cause = love_cause;
    } else {''', '''    if m.extra.age == 0 {
        if let Some(ticks) = super::ee::love(m, 600, love_cause) {
            m.extra.in_love = ticks;
            m.extra.love_player = true;
            // Animal.setInLove: the player who fed it, for a fox baby to trust.
            m.extra.love_cause = love_cause;
        }
    } else {'''),

    # ---- VillagerCareerChangeEvent.
    (MOBS + 'villager.rs', '''pub fn set_profession(m: &mut Mob, profession: u8) {
    if m.extra.variant2 != profession {
        m.extra.variant2 = profession;''', '''pub fn set_profession(m: &mut Mob, profession: u8) {
    if m.extra.variant2 != profession {
        let Some(profession) = super::ee::career(m, profession) else {
            return;
        };
        m.extra.variant2 = profession;'''),

    # ---- WardenAngerChangeEvent.
    (MOBS + 'warden.rs', '''    let maybe_switch = st.target() != Some(Who::Player);
    let new = st.anger.increase(who, amount);''', '''    let maybe_switch = st.target() != Some(Who::Player);
    let old = st.anger.of(who);
    let wanted = (old + amount).min(MAX_ANGER);
    let Some(wanted) = super::ee::warden_anger(m, who, old, wanted) else {
        return;
    };
    let Some(st) = m.extra.warden.as_mut() else {
        return;
    };
    let new = st.anger.increase(who, wanted - old);'''),
    (MOBS + 'warden.rs', '''    /// `increaseAnger`: the entity's new anger (at most 150).
    pub fn increase(&mut self, who: Who, amount: i32) -> i32 {''', '''    pub fn of(&self, who: Who) -> i32 {
        self.anger.get(&who).copied().unwrap_or(0)
    }
    /// `increaseAnger`: the entity's new anger (at most 150).
    pub fn increase(&mut self, who: Who, amount: i32) -> i32 {'''),

    # ---- CreeperIgniteEvent, its fuse, and CreeperPowerEvent.
    (GAME + 'creatures.rs', '''crate::game::paper_fam_entity::creeper_lit(m.id, self.lan_acting_uuid);
                m.extra.flag = true;''', '''crate::game::paper_fam_entity::creeper_lit(m.id, self.lan_acting_uuid);
                horizon_client::mobs::ee::creeper_ignite(m, true);'''),
    (MOBS + 'hostile.rs', '''    m.swell = (m.swell + m.extra.counter).clamp(0, 30);''', '''    m.swell = (m.swell + m.extra.counter).clamp(0, super::ee::fuse(m));'''),
    (MOBS + 'hostile.rs', '''    m.swell = (m.swell + dir).max(0);
    if m.swell >= 30 {
        m.swell = 30;''', '''    m.swell = (m.swell + dir).max(0);
    let fuse = super::ee::fuse(m);
    if m.swell >= fuse {
        m.swell = fuse;'''),
    (GAME + 'lightning.rs', '''                Species::Creeper => m.extra.charged = true,''', '''                Species::Creeper => {
                    if !horizon_client::mobs::ee::stop("ee_creeper_power", m, serde_json::json!({ "cause": "LIGHTNING" })) {
                        m.extra.charged = true;
                    }
                }'''),

    # ---- The rest of EntityCombustEvent, and the moves EntityTeleportEvent hears.
    (GAME + 'creatures.rs', '''                            if let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) {
                                m.ignite_for_ticks(seconds * 20);
                            }''', '''                            if let Some(m) = self.mobs.iter_mut().find(|m| m.id == id)
                                && let Some(ticks) = horizon_client::mobs::ee::combust(m, seconds * 20, horizon_client::mobs::ee::actor_by())
                            {
                                m.ignite_for_ticks(ticks);
                            }'''),
    (GAME + 'creatures.rs', '''                                        if !m.kind.flags.fire_immune {
                                            m.ignite_for_ticks(100);
                                        }''', '''                                        if !m.kind.flags.fire_immune
                                            && let Some(ticks) = horizon_client::mobs::ee::combust(m, 100, horizon_client::mobs::ee::other_by(m, p.id, "minecraft:arrow"))
                                        {
                                            m.ignite_for_ticks(ticks);
                                        }'''),
    (GAME + 'enchant_effects.rs', '''        if ignite > 0.0 && let Some(m) = self.mobs.get_mut(mob)
            && !m.kind.flags.fire_immune
        {
            m.ignite_for_ticks((ignite * 20.0) as i32);
        }''', '''        if ignite > 0.0 && let Some(m) = self.mobs.get_mut(mob)
            && !m.kind.flags.fire_immune
            && let Some(ticks) = horizon_client::mobs::ee::combust(m, (ignite * 20.0) as i32, horizon_client::mobs::ee::entity_by(Who::Player))
        {
            m.ignite_for_ticks(ticks);
        }'''),
    (GAME + 'lightning.rs', '''            if m.fire_ticks == 0 {
                m.ignite_for_ticks(160);
            }
            m.damage(5.0, damage);''', '''            if m.fire_ticks == 0
                && let Some(ticks) = horizon_client::mobs::ee::combust(m, 160, horizon_client::mobs::ee::other_by(m, 0, "minecraft:lightning_bolt"))
            {
                m.ignite_for_ticks(ticks);
            }
            m.damage(5.0, damage);'''),
    (MOBS + 'enderman.rs', '''    m.position = Vec3::new(x, ny, z);
    m.previous = m.position;
    m.velocity = Vec3::ZERO;
    m.ai.nav.stop();''', '''    let Some(to) = super::ee::teleport(m, [x, ny, z]) else {
        return false;
    };
    m.position = Vec3::new(to[0], to[1], to[2]);
    m.previous = m.position;
    m.velocity = Vec3::ZERO;
    m.ai.nav.stop();'''),
    (MOBS + 'shulker.rs', '''            continue;
        }
        m.extra.variant3 = face;
        events.push(MobEvent::Sound(
            "entity.shulker.teleport",''', '''            continue;
        }
        let Some(to) = super::ee::teleport(m, [f64::from(t[0]) + 0.5, f64::from(t[1]), f64::from(t[2]) + 0.5]) else {
            return false;
        };
        let t = [to[0].floor() as i32, to[1].floor() as i32, to[2].floor() as i32];
        m.extra.variant3 = face;
        events.push(MobEvent::Sound(
            "entity.shulker.teleport",'''),
    (LIB + 'mobs.rs', '''    pub fn plugin_flag(&self, key: &str) -> Option<bool> {''', '''    /// A number kept on the mob's saved plugin data.
    pub fn set_plugin_number(&mut self, key: &str, value: f64) {
        use horizon_world::nbt::Tag;
        let tag = self.plugin.get_or_insert_with(|| Tag::Compound(Default::default()).into());
        if let Tag::Compound(values) = &mut **tag {
            values.insert(key.into(), Tag::Double(value.to_bits()));
        }
    }
    pub fn plugin_flag(&self, key: &str) -> Option<bool> {'''),

    # ---- EntityPoseChangeEvent, EntityToggleGlideEvent.
    (LIB + 'pose.rs', '''        if self.pose != pose {
            self.pose = pose;
            self.refresh_dimensions()?;
        }''', '''        if self.pose != pose {
            crate::mobs::ee::pose(format!("{pose:?}"));
            self.pose = pose;
            self.refresh_dimensions()?;
        }'''),
    (LIB + 'motion.rs', '''        if std::mem::take(&mut self.player.glide_ending) {
            self.player.fall_flying = false;
        }''', '''        if std::mem::take(&mut self.player.glide_ending) && !crate::mobs::ee::glide_refused(false) {
            self.player.fall_flying = false;
        }'''),
    (LIB + 'motion.rs', '''            && !self.player.fall_flying
            && can_glide
        {
            self.player.fall_flying = true;
        }''', '''            && !self.player.fall_flying
            && can_glide
            && !crate::mobs::ee::glide_refused(true)
        {
            self.player.fall_flying = true;
        }'''),
    (LIB + 'motion.rs', '''                self.travel_in_air(input)?;
                self.player.fall_flying = false;
            } else {
                self.travel_fall_flying()?;''', '''                self.travel_in_air(input)?;
                if !crate::mobs::ee::glide_refused(false) {
                    self.player.fall_flying = false;
                }
            } else {
                self.travel_fall_flying()?;'''),
    (GAME + 'lan_actions.rs', '''        if let Some(g) = host.hub.guest_mut(conn) {
            g.fall_flying = start && !in_liquid;
        }''', '''        if let Some(g) = host.hub.guest_mut(conn) {
            let gliding = start && !in_liquid;
            horizon_client::mobs::ee::bind(Some(g.entity_id));
            if gliding == g.fall_flying || !horizon_client::mobs::ee::glide_refused(gliding) {
                g.fall_flying = gliding;
            }
            horizon_client::mobs::ee::bind(None);
        }'''),

    # ---- FoodLevelChangeEvent.
    ('horizon-world/src/food.rs', '''    pub fn tick<W: FoodWorld>(&mut self, world: &mut W) -> Result<(), W::Error> {''', '''    pub fn tick<W: FoodWorld>(&mut self, world: &mut W) -> Result<(), W::Error> {
        let before = (self.food, self.exhaustion);
        self.tick_own(world)?;
        world.food_ticked(self, before.0, before.1)
    }
    fn tick_own<W: FoodWorld>(&mut self, world: &mut W) -> Result<(), W::Error> {'''),
    (LIB + 'hunger.rs', '''        if self.health.tick_count % 10 == 0 && self.health.food.needs_food() {
            self.health.food.food = self.health.food.food.wrapping_add(1);
        }''', '''        if self.health.tick_count % 10 == 0 && self.health.food.needs_food()
            && let Some(level) = crate::mobs::ee::food(self.health.food.food.wrapping_add(1), None)
        {
            self.health.food.food = level;
        }'''),
    (LIB + 'hunger.rs', '''impl FoodWorld for TickWorld<'_> {''', '''impl FoodWorld for TickWorld<'_> {
    fn food_ticked(&mut self, food: &mut FoodData, before: i32, exhaustion: f32) -> Result<()> {
        if food.food != before {
            food.food = crate::mobs::ee::food(food.food, None).unwrap_or(before);
        }
        // The first thing `FoodData.tick` does is pay off 4: what's above that now is what healing cost.
        let left = if exhaustion > 4.0 { exhaustion - 4.0 } else { exhaustion };
        if food.exhaustion > left {
            crate::mobs::ee::exhaust_as("REGEN");
            food.exhaustion = left + crate::mobs::ee::exhaustion(food.exhaustion - left).unwrap_or(0.0);
        }
        Ok(())
    }'''),
    (GAME + 'consume.rs', '''                let food = &mut p.health.food;
                food.food = (food.food + nutrition as i32).clamp(0, 20);
                food.saturation = (food.saturation + saturation)
                    .clamp(0.0, food.food as f32);''', '''                let food = &mut p.health.food;
                if let Some(level) = horizon_client::mobs::ee::food((food.food + nutrition as i32).clamp(0, 20), Some(stack)) {
                    food.food = level;
                    food.saturation = (food.saturation + saturation)
                        .clamp(0.0, food.food as f32);
                }'''),
    (GAME + 'potions.rs', '''                let food = &mut p.health.food;
                food.food = (food.food + nutrition).clamp(0, 20);
                food.saturation = (food.saturation + nutrition as f32 * saturation * 2.0)
                    .clamp(0.0, food.food as f32);''', '''                let food = &mut p.health.food;
                if let Some(level) = horizon_client::mobs::ee::food((food.food + nutrition).clamp(0, 20), None) {
                    food.food = level;
                    food.saturation = (food.saturation + nutrition as f32 * saturation * 2.0)
                        .clamp(0.0, food.food as f32);
                }'''),

    # ---- EntityExhaustionEvent.
    (LIB + 'hunger.rs', '''        if !self.invulnerable && !self.spectator {
            self.health.food.add_exhaustion(amount);
        }''', '''        if !self.invulnerable && !self.spectator
            && let Some(amount) = crate::mobs::ee::exhaustion(amount)
        {
            self.health.food.add_exhaustion(amount);
        }'''),
    (LIB + 'motion.rs', '''        self.player
            .cause_food_exhaustion(if self.player.sprinting { 0.2 } else { 0.05 });
        Ok(())''', '''        crate::mobs::ee::exhaust_as(if self.player.sprinting { "JUMP_SPRINT" } else { "JUMP" });
        self.player
            .cause_food_exhaustion(if self.player.sprinting { 0.2 } else { 0.05 });
        Ok(())'''),
    (LIB + 'motion.rs', '''    fn award(&mut self, stat: MovementStat, amount: i32) -> Result<()> {
        self.player
            .events''', '''    fn award(&mut self, stat: MovementStat, amount: i32) -> Result<()> {
        crate::mobs::ee::exhaust_for(stat.name());
        self.player
            .events'''),
    (GAME + 'lan_pvp.rs', '''            p.cause_food_exhaustion(0.1);''', '''            horizon_client::mobs::ee::exhaust_as("ATTACK");
            p.cause_food_exhaustion(0.1);'''),
    (GAME + 'spear.rs', '''            p.cause_food_exhaustion(0.1);''', '''            horizon_client::mobs::ee::exhaust_as("ATTACK");
            p.cause_food_exhaustion(0.1);'''),
    (GAME + 'potions.rs', '''                p.cause_food_exhaustion(amount);''', '''                horizon_client::mobs::ee::exhaust_as("HUNGER_EFFECT");
                p.cause_food_exhaustion(amount);'''),

    # ---- EntityAirChangeEvent, for mobs.
    (MOBS + 'ai.rs', '''        // Air supply and drowning (WaterAnimal.handleAirSupply for aquatic mobs).
        if m.species() == Species::Axolotl {''', '''        // Air supply and drowning (WaterAnimal.handleAirSupply for aquatic mobs).
        let air_before = m.air;
        if m.species() == Species::Axolotl {'''),
    (MOBS + 'ai.rs', '''        } else if m.air < m.extra.max_air() {
            m.air = (m.air + 4).min(m.extra.max_air());
        }
''', '''        } else if m.air < m.extra.max_air() {
            m.air = (m.air + 4).min(m.extra.max_air());
        }
        super::ee::air(m, air_before);
'''),

    # ---- PigZombieAngerEvent.
    (MOBS + 'undead.rs', '''        m.extra.flag2 = true;
        m.extra.anger = 400 + m.random.next_int_bound(380).unwrap_or(0);
        m.extra.counter = m.random.next_int_bound(21).unwrap_or(0).max(1);
        m.extra.counter2 = 80 + m.random.next_int_bound(41).unwrap_or(0);''', '''        let anger = 400 + m.random.next_int_bound(380).unwrap_or(0);
        match super::ee::pig_anger(m, anger) {
            Some(anger) => {
                m.extra.flag2 = true;
                m.extra.anger = anger;
                m.extra.counter = m.random.next_int_bound(21).unwrap_or(0).max(1);
                m.extra.counter2 = 80 + m.random.next_int_bound(41).unwrap_or(0);
            }
            None => m.set_ai_target(None, "FORGOT_TARGET"),
        }'''),

    # ---- EnderDragonChangePhaseEvent.
    (GAME + 'dragon.rs', '''        if self.phase.id() == phase.id() && phase.id() != 1 {
            return;
        }
        if let Phase::Flaming { .. } = phase {''', '''        if self.phase.id() == phase.id() && phase.id() != 1 {
            return;
        }
        let Some(new) = horizon_client::mobs::ee::dragon_phase(self.id, i64::from(self.phase.id()), i64::from(phase.id())) else {
            return;
        };
        let phase = if new == i64::from(phase.id()) { phase } else { Phase::from_id(new as i32) };
        if let Phase::Flaming { .. } = phase {'''),

    # ---- ItemMergeEvent, ExperienceOrbMergeEvent.
    (LIB + 'entities.rs', '''        let (to, from) = if other.count < this.count {
            (i, j)
        } else {
            (j, i)
        };
        // ItemEntity.merge(toStack, fromStack, 64).''', '''        let (to, from) = if other.count < this.count {
            (i, j)
        } else {
            (j, i)
        };
        if crate::mobs::ee::item_merge(&items[from], &items[to]) {
            continue;
        }
        // ItemEntity.merge(toStack, fromStack, 64).'''),
    (LIB + 'entities.rs', '''                && orbs[j].bounds().intersects(area)
            {
                orbs[i].count += orbs[j].count;''', '''                && orbs[j].bounds().intersects(area)
                && !crate::mobs::ee::orb_merge(&orbs[i], &orbs[j])
            {
                orbs[i].count += orbs[j].count;'''),

    # ---- ArrowBodyCountChangeEvent.
    (GAME + 'combat.rs', '''    pub(super) fn add_arrow(&mut self) {
        self.arrows += 1;
    }''', '''    pub(super) fn add_arrow(&mut self) {
        self.arrows = horizon_client::mobs::ee::arrows(self.arrows, self.arrows + 1);
    }'''),
    (GAME + 'combat.rs', '''        Self::decay(&mut self.arrows, &mut self.remove_arrow_time);
        Self::decay(&mut self.stingers, &mut self.remove_stinger_time);''', '''        Self::decay(&mut self.arrows, &mut self.remove_arrow_time);
        self.arrows = horizon_client::mobs::ee::arrows(arrows, self.arrows);
        Self::decay(&mut self.stingers, &mut self.remove_stinger_time);'''),
    (GAME + 'lan_actions.rs', '''                    if !paused {
                        combat.stuck.tick();
                    }
                    (combat.stuck.arrows, combat.stuck.stingers)''', '''                    if !paused {
                        horizon_client::mobs::ee::bind(Some(g.entity_id));
                        combat.stuck.tick();
                        horizon_client::mobs::ee::bind(None);
                    }
                    (combat.stuck.arrows, combat.stuck.stingers)'''),

    # ---- EntityIgniteEvent: a TNT minecart.
    (GAME + 'minecarts.rs', '''        c.fuse = 80;
        let at = [c.position.x, c.position.y, c.position.z];
        self.audio.play("minecraft:entity.tnt.primed", Some(at), 1.0, 1.0);''', '''        let Some(fuse) = super::paper_fam_ev_entity::cart_ignite(self, c.id, 80) else {
            return;
        };
        c.fuse = fuse;
        let at = [c.position.x, c.position.y, c.position.z];
        self.audio.play("minecraft:entity.tnt.primed", Some(at), 1.0, 1.0);'''),
    ('horizon-world/src/food.rs', '''pub trait FoodWorld {''', '''pub trait FoodWorld {
    /// What the tick did to the food, for whoever has a say in it.
    fn food_ticked(&mut self, _food: &mut FoodData, _before: i32, _exhaustion: f32) -> Result<(), Self::Error> {
        Ok(())
    }'''),

    # ---- EntityMountEvent, EntityDismountEvent.
    (MOBS + 'riders.rs', '''    if r.extra.boarding_cooldown > 0 || v.extra.rider.is_some() || v.rider.is_some() {
        return false;
    }
    link(mobs, rider, vehicle);
    true''', '''    if r.extra.boarding_cooldown > 0 || v.extra.rider.is_some() || v.rider.is_some() {
        return false;
    }
    if super::ee::mob_boards(super::ee::row(r), vehicle) {
        return false;
    }
    link(mobs, rider, vehicle);
    true'''),
    (MOBS + 'riders.rs', '''pub fn stop_riding(m: &mut Mob, at: Option<Vec3>) {
    if m.extra.mount.take().is_none() {
        return;
    }''', '''pub fn stop_riding(m: &mut Mob, at: Option<Vec3>) {
    if m.extra.mount.is_some_and(|vehicle| super::ee::mob_leaves(super::ee::row(m), vehicle)) {
        return;
    }
    if m.extra.mount.take().is_none() {
        return;
    }'''),
    (GAME + 'boat_passengers.rs', '''                        let m = &mut self.mobs[j];
                        m.extra.mount = Some(id);
                        m.extra.passenger = true;
                        m.ai.nav.stop();
                        let mob = m.id;''', '''                        let m = &mut self.mobs[j];
                        if horizon_client::mobs::ee::mob_boards(horizon_client::mobs::ee::row(m), id) {
                            continue;
                        }
                        m.extra.mount = Some(id);
                        m.extra.passenger = true;
                        m.ai.nav.stop();
                        let mob = m.id;'''),
    (GAME + 'minecarts.rs', '''                if can_ride_cart(species, cooldown) {
                    let mob = self.mobs[j].id;
                    {''', '''                if can_ride_cart(species, cooldown) {
                    let mob = self.mobs[j].id;
                    if horizon_client::mobs::ee::mob_boards(horizon_client::mobs::ee::row(&self.mobs[j]), id) {
                        continue;
                    }
                    {'''),
    (GAME + 'boats.rs', '''        self.riding = Some(self.boats[i].id);
        self.yaw = self.boats[i].yaw;
        self.trigger_player("minecraft:started_riding");''', '''        if super::paper_fam_ev_entity::mount_refused(self, self.boats[i].id) {
            return false;
        }
        self.riding = Some(self.boats[i].id);
        self.yaw = self.boats[i].yaw;
        self.trigger_player("minecraft:started_riding");'''),
    (GAME + 'cushions.rs', '''        self.stop_riding();
        self.riding = Some(id);''', '''        if super::paper_fam_ev_entity::mount_refused(self, id) {
            return false;
        }
        self.stop_riding();
        self.riding = Some(id);'''),
    (GAME + 'ghasts.rs', '''            // doPlayerRide.
            let id = self.mobs[index].id;
            self.break_leash(id, true, false);
            let was_empty = self.mobs[index].rider.is_none();''', '''            // doPlayerRide.
            let id = self.mobs[index].id;
            if super::paper_fam_ev_entity::mount_refused(self, id) {
                return false;
            }
            self.break_leash(id, true, false);
            let was_empty = self.mobs[index].rider.is_none();'''),
    (GAME + 'llamas.rs', '''        // doPlayerRide; mounting drops a lead.
        let id = self.mobs[index].id;
        self.break_leash(id, true, false);''', '''        // doPlayerRide; mounting drops a lead.
        let id = self.mobs[index].id;
        if super::paper_fam_ev_entity::mount_refused(self, id) {
            return false;
        }
        self.break_leash(id, true, false);'''),
    (GAME + 'minecarts.rs', '''                    return false;
                }
                self.riding = Some(self.carts[i].id);
            }
            CartKind::Chest | CartKind::Hopper => self.open_cart_container(i),''', '''                    return false;
                }
                if super::paper_fam_ev_entity::mount_refused(self, id) {
                    return false;
                }
                self.riding = Some(self.carts[i].id);
            }
            CartKind::Chest | CartKind::Hopper => self.open_cart_container(i),'''),
    (GAME + 'mounts.rs', '''            return false;
        }
        self.riding = Some(m.id);
        self.trigger_player("minecraft:started_riding");
        true
    }
    /// `FoodOnAStickItem.use` while riding: boost the mount, wear the stick.''', '''            return false;
        }
        let id = m.id;
        if super::paper_fam_ev_entity::mount_refused(self, id) {
            return false;
        }
        self.riding = Some(id);
        self.trigger_player("minecraft:started_riding");
        true
    }
    /// `FoodOnAStickItem.use` while riding: boost the mount, wear the stick.'''),
    (GAME + 'mounts.rs', '''        let m = &mut self.mobs[index];
        m.extra.flag = false;
        m.extra.counter = 0;
        self.yaw = m.yaw;
        self.riding = Some(m.id);
        self.mount.jump_ticks = 0;''', '''        let id = self.mobs[index].id;
        if super::paper_fam_ev_entity::mount_refused(self, id) {
            return false;
        }
        let m = &mut self.mobs[index];
        m.extra.flag = false;
        m.extra.counter = 0;
        self.yaw = m.yaw;
        self.riding = Some(m.id);
        self.mount.jump_ticks = 0;'''),
    (GAME + 'mounts.rs', '''        let Some(id) = self.riding.take() else {
            return;
        };
        let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) else {
            return;
        };
        m.rider = None;''', '''        if let Some(id) = self.riding
            && super::paper_fam_ev_entity::dismount_refused(self, id)
        {
            return;
        }
        let Some(id) = self.riding.take() else {
            return;
        };
        let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) else {
            return;
        };
        m.rider = None;'''),

    # ---- HorseJumpEvent.
    (GAME + 'mounts.rs', '''        let jump = self.charge_mount_jump(i, keys);
        let m = &mut self.mobs[i];
        if jump > 0.0 {
            if equine::is_equine(m.species()) {
                equine::on_player_jump(m);
            }''', '''        let jump = self.charge_mount_jump(i, keys);
        let m = &mut self.mobs[i];
        let jump = if jump > 0.0 && equine::is_equine(m.species()) { horizon_client::mobs::ee::horse_jump(m, jump) } else { jump };
        if jump > 0.0 {
            if equine::is_equine(m.species()) {
                equine::on_player_jump(m);
            }'''),

    # ---- EnderDragonFlameEvent, EnderDragonFireballHitEvent.
    (GAME + 'dragon.rs', '''                    self.breath_clouds
                        .push(Breath {
                            id: 0,
                            at: [x, y.floor() + 1.0, z],
                            radius: 5.0,
                            age: 0,
                            duration: 200,
                            from_dragon: true,
                            potion: None,
                        });''', '''                    self.breath_clouds
                        .push(Breath {
                            id: 0,
                            at: [x, y.floor() + 1.0, z],
                            radius: 5.0,
                            age: 0,
                            duration: 200,
                            from_dragon: true,
                            potion: None,
                        });
                    super::paper_fam_ev_entity::dragon_cloud(self, "ee_dragon_flame");'''),
    (GAME + 'dragon.rs', '''                    self.breath_clouds
                        .push(Breath {
                            id: 0,
                            at: [next.x, next.y, next.z],
                            radius: 3.0,
                            age: 0,
                            duration: 600,
                            from_dragon: false,
                            potion: None,
                        });''', '''                    self.breath_clouds
                        .push(Breath {
                            id: 0,
                            at: [next.x, next.y, next.z],
                            radius: 3.0,
                            age: 0,
                            duration: 600,
                            from_dragon: false,
                            potion: None,
                        });
                    super::paper_fam_ev_entity::ball_dying(b.id, [next.x, next.y, next.z], [b.velocity.x, b.velocity.y, b.velocity.z]);
                    super::paper_fam_ev_entity::dragon_cloud(self, "ee_dragon_fireball");
                    super::paper_fam_ev_entity::dying_over();'''),

    # ---- TrialSpawnerSpawnEvent.
    (GAME + 'trial_spawner.rs', '''            if let Some(id) = ids.get(&provisional) {
                self.persist.uuids.insert(*id, uuid);
                by_uuid.insert(uuid, *id);
            }''', '''            if let Some(id) = ids.get(&provisional) {
                self.persist.uuids.insert(*id, uuid);
                if !super::paper_fam_ev_entity::trial_spawned(self, *id, pos) {
                    by_uuid.insert(uuid, *id);
                }
            }'''),

    # ---- VillagerReplenishTradeEvent, VillagerAcquireTradeEvent.
    (MOBS + 'villager_brain.rs', '''    if let Some(t) = m.extra.trade.as_mut()
        && t.should_restock(w.game_time, day)
    {
        t.restock(w.game_time);
    }''', '''    if m.extra.trade.as_mut().is_some_and(|t| t.should_restock(w.game_time, day)) {
        super::ee::restock(m, w.game_time);
    }'''),
    (GAME + 'trading.rs', '''            let trade = self.mobs[index].extra.trade.get_or_insert_default();
            trade.offers = offers;
            trade.unset = false;
        }''', '''            let trade = self.mobs[index].extra.trade.get_or_insert_default();
            trade.offers = offers;
            trade.unset = false;
            super::paper_fam_ev_entity::offers_made(self, index, 0);
        }'''),

    # ---- Each of these was a take-over at the head of the game's own function: now it's a call at the moment.
    (GAME + 'creatures.rs', '''                    let n = 2 + self.world.random.next_int_bound(3).unwrap_or(0);
                    for k in 0..n {
                        let off = |kk: i32| {''', '''                    let n = 2 + self.world.random.next_int_bound(3).unwrap_or(0);
                    let n = super::paper_fam_ev_entity::slime_count(self, m, n);
                    for k in 0..n {
                        let off = |kk: i32| {'''),
    (GAME + 'creatures.rs', '''                if self.world.random.next_int_bound(8).unwrap_or(1) == 0 {
                    let n = if self.world.random.next_int_bound(32).unwrap_or(1) == 0 {
                        4
                    } else {
                        1
                    };
                    for _ in 0..n {
                        if let Some(id) = self.spawn_mob("minecraft:chicken", atp, true)
                            && let Some(c) = self.mobs.iter_mut().find(|m| m.id == id)
                        {
                            c.extra.age = -24000;
                        }
                    }
                }
                self.thrown_break_event(p.id, atp);
                self.item_burst(atp, "minecraft:egg");''', '''                let hatching = self.world.random.next_int_bound(8).unwrap_or(1) == 0;
                let n = if !hatching {
                    0
                } else if self.world.random.next_int_bound(32).unwrap_or(1) == 0 {
                    4
                } else {
                    1
                };
                let (n, kind) = super::paper_fam_ev_entity::egg_hatch(self, p.id, hatching, n);
                for _ in 0..n {
                    if let Some(id) = self.spawn_mob(&kind, atp, true)
                        && let Some(c) = self.mobs.iter_mut().find(|m| m.id == id)
                    {
                        c.extra.age = -24000;
                    }
                }
                self.thrown_break_event(p.id, atp);
                self.item_burst(atp, "minecraft:egg");'''),
    (GAME + 'piglins.rs', '''        let at = m.position;
        let host = self
            .player
            .as_ref()
            .filter(|p| !p.health.dead_or_dying() && !self.spectator())
            .map(|p| [p.position.x, p.position.y, p.position.z]);
        let guests = self''', '''        let at = m.position;
        let items = if self.mobs[i].is_baby() {
            items
        } else {
            match super::paper_fam_ev_entity::barter_outcome(self, mob, &item, items) {
                Some(items) => items,
                None => return,
            }
        };
        let host = self
            .player
            .as_ref()
            .filter(|p| !p.health.dead_or_dying() && !self.spectator())
            .map(|p| [p.position.x, p.position.y, p.position.z]);
        let guests = self'''),
    (GAME + 'potions.rs', '''        if &*stack.item == "minecraft:experience_bottle" {
            self.level_event_at(2002, block, 0xFF38_5DC6u32 as i32);
            self.level_event_at(1053, block, 0);
            let r = &mut self.world.random;
            let value = 3 + r.next_int_bound(5).unwrap_or(0)
                + r.next_int_bound(5).unwrap_or(0);
''', '''        if &*stack.item == "minecraft:experience_bottle" {
            let r = &mut self.world.random;
            let value = 3 + r.next_int_bound(5).unwrap_or(0)
                + r.next_int_bound(5).unwrap_or(0);
            let (value, shows) = super::paper_fam_ev_entity::bottle(self, stack, at, value);
            if shows {
                self.level_event_at(2002, block, 0xFF38_5DC6u32 as i32);
                self.level_event_at(1053, block, 0);
            }
'''),
    (GAME + 'potions.rs', '''        self.affect_entities_around(stack, at, owner.is_some());
        let lingering = &*stack.item == "minecraft:lingering_potion";''', '''        self.affect_entities_around(stack, at, owner.is_some());
        super::paper_fam_ev_entity::wet_after(self);
        let lingering = &*stack.item == "minecraft:lingering_potion";'''),
    (GAME + 'potions.rs', '''        for (i, m) in self.mobs.iter().enumerate() {
            if m.alive() && let Some(d2) = splash_distance_sqr(at, age, m.bounds()) {
                let inverted = undead
                    .get("minecraft:inverted_healing_and_harm")
                    .is_some_and(|t| t.contains(&m.kind.id));
                hits.push((i, 1.0 - d2.sqrt() / 4.0, inverted));
            }
        }''', '''        for (i, m) in self.mobs.iter().enumerate() {
            let own = splash_distance_sqr(at, age, m.bounds()).map(|d2| 1.0 - d2.sqrt() / 4.0);
            if m.alive() && let Some(intensity) = super::paper_fam_ev_entity::mob_strength(m.id, own) {
                let inverted = undead
                    .get("minecraft:inverted_healing_and_harm")
                    .is_some_and(|t| t.contains(&m.kind.id));
                hits.push((i, intensity, inverted));
            }
        }'''),
    (GAME + 'potions.rs', '''            if let Some(d2) = splash_distance_sqr(at, age, bounds) {
                let intensity = 1.0 - d2.sqrt() / 4.0;
                for (id, duration, amp) in effects {''', '''            let own = splash_distance_sqr(at, age, bounds).map(|d2| 1.0 - d2.sqrt() / 4.0);
            if let Some(intensity) = super::paper_fam_ev_entity::player_strength(self, own) {
                for (id, duration, amp) in effects {'''),

    # ---- A player's landing, what they stand in, and their air: their server-side copy's own move.
    (LIB + 'motion.rs', '''        // LivingEntity.checkFallDamage: a landing runs onChangedBlock.
        if grounded && self.player.fall_distance > 0.0 {
            self.player.location_effects.landed = true;
        }''', '''        if grounded && self.player.fall_distance > 0.0 {
            let name = self.assets.registry.state(state).map_or("minecraft:air", |state| state.name.as_str());
            self.player.fall_distance = crate::mobs::ee::fall(self.player.fall_distance, [position.x, position.y, position.z], name);
        }
        // LivingEntity.checkFallDamage: a landing runs onChangedBlock.
        if grounded && self.player.fall_distance > 0.0 {
            self.player.location_effects.landed = true;
        }'''),
    (LIB + 'motion.rs', '''            if inside && visited.insert(p.pack()) {
                crate::block_contact::emit(crate::block_contact::Event::Inside(''', '''            if inside && visited.insert(p.pack()) && !crate::mobs::ee::player_inside(name, [p.x, p.y, p.z]) {
                crate::block_contact::emit(crate::block_contact::Event::Inside('''),
    (LIB + 'motion/environment.rs', '''        if submerged
            && !self.player.status.water_breathing
            && !self.player.status.breath_of_the_nautilus''', '''        let air_before = self.player.air;
        if submerged
            && !self.player.status.water_breathing
            && !self.player.status.breath_of_the_nautilus'''),
    (LIB + 'motion/environment.rs', '''        if self.player.is_in_lava() {
            self.player.ignite_for_ticks(300)?;
            self.environment_hurt("minecraft:lava", 4.0)?;
        }
        Ok(())''', '''        self.player.air = crate::mobs::ee::player_air(air_before, self.player.air);
        if self.player.is_in_lava() {
            self.player.ignite_for_ticks(300)?;
            self.environment_hurt("minecraft:lava", 4.0)?;
        }
        Ok(())'''),
    (LIB + 'lan/hub.rs', '''                        1 | 2 => self.set_sprinting(i, p.action == 1),
                        0 => self.wake(i),
                        _ => {}''', '''                        1 | 2 => self.set_sprinting(i, p.action == 1),
                        0 => self.wake(i),
                        3 => {
                            if let Some(vehicle) = self.guests[i].vehicle {
                                crate::mobs::ee::guest_horse_jump(vehicle, p.data);
                            }
                        }
                        _ => {}'''),

    # ---- A mob's effects, EntityPortalEnterEvent each tick.
    (LIB + 'mobs.rs', '''    pub fn add_effect(&mut self, effect: &'static str, ticks: i32, amplifier: i32) {
        let effect = horizon_client_core::potions::namespaced(effect);''', '''    pub fn add_effect(&mut self, effect: &'static str, ticks: i32, amplifier: i32) {
        let effect = horizon_client_core::potions::namespaced(effect);
        if ee::effect_refused(self, effect, ticks, amplifier) {
            return;
        }'''),
    (GAME + 'nether_portal.rs', '''        for (id, pos) in inside {
            self.portals.entities.entry(id).or_default().set_inside(pos, ENTITY_DELAY);
        }''', '''        for (id, pos) in inside {
            if !super::paper_fam_ev_entity::portal_enter(self, id, pos) {
                self.portals.entities.entry(id).or_default().set_inside(pos, ENTITY_DELAY);
            }
        }'''),

    # ---- EntityUnleashEvent's reason.
    (GAME + 'creatures.rs', '''                    self.break_leash(mob, drop, sound)''', '''                    horizon_client::mobs::ee::unleash_as(if sound { "DISTANCE" } else { "HOLDER_GONE" });
                    self.break_leash(mob, drop, sound)'''),
    (GAME + 'leads.rs', '''            let creative = self.creative;
            self.break_leash(id, !creative, false);''', '''            let creative = self.creative;
            horizon_client::mobs::ee::unleash_as("PLAYER_UNLEASH");
            self.break_leash(id, !creative, false);'''),

    # ---- EntityPotentSulfurLaunchEvent.
    (GAME + 'potent_sulfur.rs', '''                && mob.velocity.y < limit
            {
                mob.velocity.y += push;
            }''', '''                && mob.velocity.y < limit
                && !horizon_client::mobs::ee::stop("ee_sulfur", mob, serde_json::json!({ "pos": [pos.0, pos.1, pos.2], "source": source }))
            {
                mob.velocity.y += push;
            }'''),

    # ---- The last of what was read from saved data a tick late.
    (MOBS + 'bat.rs', '''pub fn set_resting(m: &mut Mob, value: bool) {
    m.extra.flag = value;''', '''pub fn set_resting(m: &mut Mob, value: bool) {
    if value != m.extra.flag && super::ee::stop("ee_bat", m, serde_json::json!({ "awake": !value })) {
        return;
    }
    m.extra.flag = value;'''),
    (MOBS + 'ai.rs', '''    if m.species() == Species::Sheep {
        m.extra.sheared = false;
        // Sheep.ate: only a baby `canAgeUp` (no age lock holds it) grows.''', '''    if m.species() == Species::Sheep {
        if !super::ee::stop("ee_sheep_regrow", m, serde_json::json!({})) {
            m.extra.sheared = false;
        }
        // Sheep.ate: only a baby `canAgeUp` (no age lock holds it) grows.'''),
    (MOBS + 'aquatic.rs', '''            m.sound("entity.puffer_fish.blow_up", events);
            m.extra.size = 1;
        } else if m.extra.counter > 40 && puff == 1 {
            m.sound("entity.puffer_fish.blow_up", events);
            m.extra.size = 2;
        }''', '''            m.sound("entity.puffer_fish.blow_up", events);
            super::ee::puff(m, 1);
        } else if m.extra.counter > 40 && puff == 1 {
            m.sound("entity.puffer_fish.blow_up", events);
            super::ee::puff(m, 2);
        }'''),
    (MOBS + 'aquatic.rs', '''            m.sound("entity.puffer_fish.blow_out", events);
            m.extra.size = 1;
        } else if m.extra.counter2 > 100 && puff == 1 {
            m.sound("entity.puffer_fish.blow_out", events);
            m.extra.size = 0;
        }''', '''            m.sound("entity.puffer_fish.blow_out", events);
            super::ee::puff(m, 1);
        } else if m.extra.counter2 > 100 && puff == 1 {
            m.sound("entity.puffer_fish.blow_out", events);
            super::ee::puff(m, 0);
        }'''),

    # ---- EntityToggleSitEvent: every sit a mob takes or leaves.
    ('horizon-client/src/mobs/tame.rs', '    m.extra.sitting = false;', '    super::ee::sit(m, false);'),
    ('horizon-client/src/game/creatures.rs', '                && m.on_ground && !parrot_poisonous => {\n                m.extra.sitting = !m.extra.sitting;', '                && m.on_ground && !parrot_poisonous => {\n                let sit = !m.extra.sitting;\n                horizon_client::mobs::ee::sit(m, sit);'),
    ('horizon-client/src/game/creatures.rs', '                && !(is_food && takes_food(m)) => {\n                m.extra.sitting = !m.extra.sitting;\n                self.world', '                && !(is_food && takes_food(m)) => {\n                let sit = !m.extra.sitting;\n                horizon_client::mobs::ee::sit(m, sit);\n                self.world'),
    ('horizon-client/src/game/creatures.rs', '                m.extra.sitting = !m.extra.sitting;', '                let sit = !m.extra.sitting;\n                horizon_client::mobs::ee::sit(m, sit);'),
    ('horizon-client/src/game/creatures.rs', '                    m.extra.owner = Some(acting);\n                    m.extra.sitting = true;', '                    m.extra.owner = Some(acting);\n                    horizon_client::mobs::ee::sit(m, true);'),
    ('horizon-client/src/game/creatures.rs', '                    m.extra.sitting = true;', '                    horizon_client::mobs::ee::sit(m, true);'),

    # ---- EntityLoadCrossbowEvent, for a pillager.
    (MOBS + 'illager.rs', '''        stack.data_mut().charged = vec![ItemStack::new("minecraft:arrow", 1)];
    } else if stack.data().is_none_or(|d| d.charged.is_empty()) {''', '''        if super::ee::stop("ee_mob_crossbow", m, serde_json::json!({ "stack": super::ee::stack(&stack), "hand": "HAND" })) {
            return;
        }
        stack.data_mut().charged = vec![ItemStack::new("minecraft:arrow", 1)];
    } else if stack.data().is_none_or(|d| d.charged.is_empty()) {'''),

    # ---- EntityTameEvent: with the roll, as `TamableAnimal`s do.
    ('horizon-client/src/game/creatures.rs', '                m.persistent = true;\n                let tamed = m.random.next_int_bound(3).unwrap_or(1) == 0;', '                m.persistent = true;\n                let tamed = m.random.next_int_bound(3).unwrap_or(1) == 0 && !horizon_client::mobs::ee::tame_refused(m, acting);'),
    ('horizon-client/src/game/creatures.rs', '                let tamed = m.random.next_int_bound(3).unwrap_or(1) == 0;', '                let tamed = m.random.next_int_bound(3).unwrap_or(1) == 0 && !horizon_client::mobs::ee::tame_refused(m, acting);'),
    ('horizon-client/src/game/creatures.rs', '                let tamed = m.random.next_int_bound(10).unwrap_or(1) == 0;', '                let tamed = m.random.next_int_bound(10).unwrap_or(1) == 0 && !horizon_client::mobs::ee::tame_refused(m, acting);'),
    ('horizon-client/src/game/nautili.rs', '                let tamed = m.random.next_int_bound(3).unwrap_or(1) == 0;', '                let tamed = m.random.next_int_bound(3).unwrap_or(1) == 0\n                    && !horizon_client::mobs::ee::tame_refused(m, self.level.player_uuid.unwrap_or([0, 0, 0, 1]));'),

    # ---- EntityEnterLoveModeEvent for what the game feeds by hand, and EntityCombustEvent for a player.
    ('horizon-client/src/game/llamas.rs', '            m.extra.in_love = 600;', '            if let Some(ticks) = horizon_client::mobs::ee::love(m, 600, None) {\n                m.extra.in_love = ticks;\n            }'),
    ('horizon-client/src/game/mounts.rs', '            m.extra.in_love = 600;', '            if let Some(ticks) = horizon_client::mobs::ee::love(m, 600, None) {\n                m.extra.in_love = ticks;\n            }'),
    ('horizon-client/src/game/nautili.rs', '            m.extra.in_love = 600;', '            if let Some(ticks) = horizon_client::mobs::ee::love(m, 600, None) {\n                m.extra.in_love = ticks;\n            }'),
    ('horizon-client/src/game/pandas.rs', '            m.extra.in_love = 600;', '            if let Some(ticks) = horizon_client::mobs::ee::love(m, 600, None) {\n                m.extra.in_love = ticks;\n            }'),
    ('horizon-client/src/motion.rs', '        let scaled = (f64::from(ticks) * self.attribute("minecraft:burning_time")?).ceil() as i32;', '        let Some(ticks) = crate::mobs::ee::player_combust(ticks) else {\n            return Ok(());\n        };\n        let scaled = (f64::from(ticks) * self.attribute("minecraft:burning_time")?).ceil() as i32;'),

    # ---- EntityPortalEnterEvent for a player.
    (GAME + 'nether_portal.rs', '''    pub(super) fn player_inside_nether_portal(&mut self, pos: BlockPos) {
        let alive = self.player.as_ref().is_some_and(|p| !p.health.dead_or_dying());''', '''    pub(super) fn player_inside_nether_portal(&mut self, pos: BlockPos) {
        if super::paper_fam_ev_entity::player_portal_enter(self, pos) {
            return;
        }
        let alive = self.player.as_ref().is_some_and(|p| !p.health.dead_or_dying());'''),

    # ---- PlayerLeashEntityEvent, when the lead really goes on.
    (GAME + 'leads.rs', '''            let center = self.player_center();
            if self.can_attach_lead(index, center) {
                if self.mobs[index].extra.leash.is_some() {''', '''            let center = self.player_center();
            if self.can_attach_lead(index, center) && !super::paper_fam_ev_entity::leash_refused(self, index, item) {
                if self.mobs[index].extra.leash.is_some() {'''),

    # ---- EntityKnockbackEvent and its kin, for a mob.
    (LIB + 'mobs.rs', '''        if power <= 0.0 {
            return;
        }
        self.needs_sync = true;
        while xd * xd + zd * zd < f64::from(1.0e-5_f32) {''', '''        if power <= 0.0 {
            return;
        }
        let Some((power, x, z)) = ee::mob_knock(self, power, xd, zd) else {
            return;
        };
        (xd, zd) = (x, z);
        self.needs_sync = true;
        while xd * xd + zd * zd < f64::from(1.0e-5_f32) {'''),

    # ---- EntityPickupItemEvent for mobs.
    (GAME + 'mob_looting.rs', '''                // `Mob.pickUpItem`: nothing leaves the ground what it cannot use.
                let Some(taken) = self.equip_item_if_possible(i, &stack) else {''', '''                let id = self.mobs[i].id;
                if super::paper_fam_ev_entity::mob_pickup(self, id, j) {
                    j += 1;
                    continue;
                }
                // `Mob.pickUpItem`: nothing leaves the ground what it cannot use.
                let Some(taken) = self.equip_item_if_possible(i, &stack) else {'''),
    (GAME + 'villagers.rs', '''                let before = self.dropped[j].clone();
                let rest = villager::ext_mut(&mut self.mobs[i]).add_item(stack, max);''', '''                let id = self.mobs[i].id;
                if super::paper_fam_ev_entity::mob_pickup(self, id, j) {
                    j += 1;
                    continue;
                }
                let before = self.dropped[j].clone();
                let rest = villager::ext_mut(&mut self.mobs[i]).add_item(stack, max);'''),
    (GAME + 'piglins.rs', '''            let (taken_item, id) = (e.clone(), self.mobs[i].id);
            let item = e.stack.item.clone();
            self.add_pickup(''', '''            let (taken_item, id) = (e.clone(), self.mobs[i].id);
            let item = e.stack.item.clone();
            if super::paper_fam_ev_entity::mob_pickup(self, id, j) {
                continue;
            }
            self.add_pickup('''),
    (GAME + 'pandas.rs', '''                if let Some(j) = taken {
                    let e = self.remove_dropped(j);''', '''                if let Some(j) = taken
                    && !super::paper_fam_ev_entity::mob_pickup(self, id, j)
                {
                    let e = self.remove_dropped(j);'''),
    (GAME + 'allays.rs', '''                let (item, id) = (e.clone(), self.mobs[i].id);
                // It carries the fetched stack, potion and all, not just its id.''', '''                let (item, id) = (e.clone(), self.mobs[i].id);
                if super::paper_fam_ev_entity::mob_pickup(self, id, j) {
                    continue;
                }
                let e = &item;
                // It carries the fetched stack, potion and all, not just its id.'''),

    # ---- EntityPotionEffectEvent as effects end.
    (MOBS + 'ai.rs', '''    effects.retain(|e| e.1 > 0 || e.1 == -1);
    m.extra
        .hidden_effects''', '''    super::ee::effects_ending(m, &mut effects);
    effects.retain(|e| e.1 > 0 || e.1 == -1);
    m.extra
        .hidden_effects'''),
    (MOBS + 'trader.rs', '''                m.extra.effects.clear();
                cx.events.push(MobEvent::Sound(
                    "entity.wandering_trader.drink_milk",''', '''                super::ee::effects_clear(m, "MILK");
                cx.events.push(MobEvent::Sound(
                    "entity.wandering_trader.drink_milk",'''),
    (LIB + 'game.rs', '''            ConsumeEffect::ClearAllEffects => {
                self.potions.effects.clear();''', '''            ConsumeEffect::ClearAllEffects => {
                paper_fam_ev_entity::effects_clear(self, "MILK");'''),
    (LIB + 'game.rs', '''                    let id = target.trim_start_matches("minecraft:");
                    self.potions.effects.list.retain(|e| e.id != id);''', '''                    let id = target.trim_start_matches("minecraft:");
                    if !paper_fam_ev_entity::effect_gone(self, id, "MILK", "REMOVED") {
                        self.potions.effects.list.retain(|e| e.id != id);
                    }'''),
    (GAME + 'potions.rs', '''                .map(|e| e.id)
                .collect();
            self.potions.effects.clear();
            if !ids.is_empty() {''', '''                .map(|e| e.id)
                .collect();
            super::paper_fam_ev_entity::effects_clear(self, "DEATH");
            if !ids.is_empty() {'''),
    (GAME + 'potions.rs', '''        let mut effects = std::mem::take(&mut self.potions.effects);
        let (actions, raid_omen_added) = effects''', '''        let mut effects = std::mem::take(&mut self.potions.effects);
        let had = super::paper_fam_ev_entity::effects_held(&effects);
        let (actions, raid_omen_added) = effects'''),
    (GAME + 'potions.rs', '''        self.potions.effects = effects;
        if raid_omen_added {''', '''        self.potions.effects = effects;
        super::paper_fam_ev_entity::effects_expired(self, had);
        if raid_omen_added {'''),
    (GAME + 'totem.rs', '''        self.post_game_event("minecraft:item_interact_finish", body.at, ctx);
        self.potions.effects.clear();''', '''        self.post_game_event("minecraft:item_interact_finish", body.at, ctx);
        super::paper_fam_ev_entity::effects_clear(self, "TOTEM");'''),
    (GAME + 'commands/effects.rs', '''                                let any = !g.potions.effects.list.is_empty();
                                g.potions.effects.clear();''', '''                                let any = !g.potions.effects.list.is_empty();
                                super::super::paper_fam_ev_entity::effects_clear(g, "COMMAND");'''),
    (GAME + 'commands/effects.rs', '''                                let before = g.potions.effects.list.len();
                                g.potions.effects.list.retain(|x| x.id != e.id);''', '''                                let before = g.potions.effects.list.len();
                                if !super::super::paper_fam_ev_entity::effect_gone(g, e.id, "COMMAND", "REMOVED") {
                                    g.potions.effects.list.retain(|x| x.id != e.id);
                                }'''),

    # ---- EntityPoseChangeEvent for mobs.
    (GAME + 'creatures.rs', '''                let kept = m.tick(&w, &mut scratch, &mut events);
                if !kept && !m.alive() && !m.removed && !m.extra.effects.is_empty() {''', '''                let pose_was = horizon_client::mobs::ee::pose_of(m);
                let kept = m.tick(&w, &mut scratch, &mut events);
                horizon_client::mobs::ee::pose_after(m, pose_was);
                if !kept && !m.alive() && !m.removed && !m.extra.effects.is_empty() {'''),

    # ---- A mob's totem: `LivingEntity.checkTotemDeathProtection`.
    (LIB + 'mobs.rs', '''        ai::after_hurt(self, &source);
        if self.health <= 0.0 {
            self.health = 0.0;
        }''', '''        ai::after_hurt(self, &source);
        if self.health <= 0.0 {
            self.health = 0.0;
            ee::totem(self, source.kind.bypasses_invulnerability());
        }'''),

    # ---- The steps of a hit, for `EntityDamageEvent`'s modifiers.
    (LIB + 'health.rs', '''pub struct HealthResources {
    pub damage: Catalog,''', '''/// What `hurtServer` does to a hit after its event, step by step: what
/// `EntityDamageEvent`'s modifiers name, worked out before the hit lands.
#[derive(Clone, Debug, Default)]
pub struct HitSteps {
    /// The raised blocking item, if it can stop this hit.
    pub blocking: Option<Blocking>,
    pub freezing: bool,
    pub helmet: bool,
    /// What the hit before this one took, inside its invulnerability frames.
    pub frames: f32,
    /// A player's frames come off before armour, a mob's after everything.
    pub frames_first: bool,
    /// Armour and toughness.
    pub armor: Option<(f32, f32)>,
    /// What the weapon adds to the armour's fraction.
    pub breach: Option<f32>,
    pub resistance: Option<i32>,
    pub protection: f32,
    /// A mob's own share of magic, on top of `protection`.
    pub magic: Option<f32>,
    pub absorption: f32,
}
/// The hit as its steps leave it.
pub struct HitWork {
    /// By `EntityDamageEvent.DamageModifier`, what each one adds.
    pub rows: [(&'static str, f32); 8],
    /// What meets the health and absorption, a mob's frames still to come.
    pub damage: f32,
}
/// `LivingEntity.getDamageAfterArmorAbsorb`'s fraction, before any weapon.
pub fn armor_fraction(armor: f32, toughness: f32, damage: f32) -> f32 {
    (armor - damage / (2.0 + toughness / 4.0)).max(armor * 0.2).min(20.0) / 25.0
}
impl HitSteps {
    pub fn work(&self, base: f32) -> HitWork {
        fn step(now: &mut f32, next: impl FnOnce(f32) -> f32) -> f32 {
            let was = *now;
            *now = next(was);
            *now - was
        }
        let mut now = base.max(0.0);
        let blocked = match &self.blocking {
            Some(b) if now > 0.0 && !b.bypassed && !b.pierce => b.rules.resolve(&mut MatchedSets, now, b.angle).unwrap_or(0.0),
            _ => 0.0,
        };
        let blocking = step(&mut now, |now| now - blocked);
        let freezing = if self.freezing { step(&mut now, |now| now * 5.0) } else { 0.0 };
        let hard_hat = if self.helmet { step(&mut now, |now| now * 0.75) } else { 0.0 };
        if !now.is_finite() {
            now = f32::MAX;
        }
        let frames = if self.frames_first && self.frames > 0.0 { step(&mut now, |now| now - self.frames) } else { 0.0 };
        let armor = match self.armor {
            Some((armor, toughness)) => {
                let mut fraction = armor_fraction(armor, toughness, now);
                if let Some(breach) = self.breach {
                    fraction = (fraction + breach).clamp(0.0, 1.0);
                }
                step(&mut now, |now| now * (1.0 - fraction))
            }
            None => 0.0,
        };
        let resistance = match self.resistance {
            Some(amplifier) => {
                let left = 25i32.wrapping_sub(amplifier.wrapping_add(1).wrapping_mul(5));
                step(&mut now, |now| ((now * left as f32) / 25.0).max(0.0))
            }
            None => 0.0,
        };
        let protection = self.protection.clamp(0.0, 20.0);
        let mut magic = 0.0;
        if now > 0.0 && protection > 0.0 {
            magic += step(&mut now, |now| now * (1.0 - protection / 25.0));
        }
        if let Some(share) = self.magic {
            magic += step(&mut now, |now| now * share);
        }
        let last = if self.frames_first { 0.0 } else { self.frames.min(now) };
        let absorbed = (now - last).min(self.absorption).max(0.0);
        HitWork {
            rows: [
                ("INVULNERABILITY_REDUCTION", frames - last),
                ("FREEZING", freezing),
                ("HARD_HAT", hard_hat),
                ("BLOCKING", blocking),
                ("ARMOR", armor),
                ("RESISTANCE", resistance),
                ("MAGIC", magic),
                ("ABSORPTION", -absorbed),
            ],
            damage: now,
        }
    }
}
/// The hits now being heard, innermost last: their steps, what came in and what a
/// listener's modifiers made of it, as the damage and what absorption takes of it.
static HITS: std::sync::Mutex<Vec<(HitSteps, f32, Option<[f32; 2]>)>> = std::sync::Mutex::new(Vec::new());
pub fn hit_begins(steps: HitSteps, base: f32) {
    HITS.lock().unwrap_or_else(|e| e.into_inner()).push((steps, base, None));
}
pub fn hit_ends() -> Option<[f32; 2]> {
    HITS.lock().unwrap_or_else(|e| e.into_inner()).pop().and_then(|hit| hit.2)
}
pub fn hit_said(said: [f32; 2]) {
    if let Some(hit) = HITS.lock().unwrap_or_else(|e| e.into_inner()).last_mut() {
        hit.2 = Some(said);
    }
}
/// The modifiers of the hit being heard, of `base` or of what came in; null with none.
pub fn hit_rows(base: Option<f32>) -> serde_json::Value {
    let hits = HITS.lock().unwrap_or_else(|e| e.into_inner());
    let Some((steps, came, _)) = hits.last() else {
        return serde_json::Value::Null;
    };
    let base = base.unwrap_or(*came);
    let mut rows = serde_json::Map::new();
    rows.insert("BASE".to_owned(), base.into());
    for (name, change) in steps.work(base).rows {
        rows.insert(name.to_owned(), change.into());
    }
    rows.into()
}
/// The steps a player's hit will take.
fn player_steps(
    source: &DamageSource<'_, ()>,
    armor: f32,
    toughness: f32,
    absorption: f32,
    status: &crate::effects::StatusFlags,
    blocking: Option<&Blocking>,
) -> HitSteps {
    let effects = !source.bypasses(Bypass::Effects);
    HitSteps {
        blocking: blocking.cloned(),
        frames_first: true,
        armor: (!source.bypasses(Bypass::Armor)).then_some((armor.floor(), toughness)),
        resistance: status.resistance.filter(|_| effects && !source.bypasses(Bypass::Resistance)),
        protection: if effects && !source.bypasses(Bypass::Enchantments) { status.protection(&source.definition().name) } else { 0.0 },
        absorption,
        ..Default::default()
    }
}
pub struct HealthResources {
    pub damage: Catalog,'''),
    (LIB + 'health.rs', '''        let before = self.health.health() + self.health.absorption();
        let fire_ticks = self.fire_ticks;
        let source_name = source.definition().name.clone();''', '''        let before = self.health.health() + self.health.absorption();
        let steps = crate::mobs::ee::hears("allow_damage").then(|| {
            player_steps(&source, armor, toughness, self.health.absorption(), &self.status, blocking.as_ref())
        });
        let fire_ticks = self.fire_ticks;
        let source_name = source.definition().name.clone();'''),
    (LIB + 'health.rs', '''            totem_fired: false,
            canceled: false,
        };
        let hurt = ServerPlayerCombat::new(''', '''            totem_fired: false,
            canceled: false,
            steps,
            said: None,
        };
        let hurt = ServerPlayerCombat::new('''),
    (LIB + 'health.rs', '''    /// A native mod canceled the hit (`living_incoming_damage`).
    pub(crate) canceled: bool,
}''', '''    /// A native mod canceled the hit (`living_incoming_damage`).
    pub(crate) canceled: bool,
    /// The hit's steps, while someone listens to it.
    pub(crate) steps: Option<HitSteps>,
    /// What a listener's modifiers made of the hit.
    pub(crate) said: Option<[f32; 2]>,
}'''),
    (LIB + 'health.rs', '''    fn hurt_start(&mut self, _: &mut State, damage: &mut f32) -> io::Result<bool> {
        let source = HurtSource {''', '''    fn hurt_start(&mut self, state: &mut State, damage: &mut f32) -> io::Result<bool> {
        let cooling = state.cooldown as f32 > 10.0 && !self.source.hurt_tag(HurtTag::BypassesCooldown);
        let heard = self.steps.take().map(|mut steps| {
            steps.frames = if cooling { state.last_hurt } else { 0.0 };
            hit_begins(steps, *damage);
        }).is_some();
        let source = HurtSource {'''),
    (LIB + 'health.rs', '''            damage,
        );
        self.canceled |= canceled;
        Ok(canceled)''', '''            damage,
        );
        if heard {
            self.said = hit_ends();
        }
        self.canceled |= canceled;
        Ok(canceled)'''),
    (LIB + 'health.rs', '''    fn damage_event(&mut self) -> io::Result<()> {
        self.phase("entity-damage")
    }
}
impl CombatEnvironment<Attributes> for Environment<'_> {''', '''    fn damage_event(&mut self) -> io::Result<()> {
        self.phase("entity-damage")
    }
    fn damage_said(&self) -> Option<[f32; 2]> {
        self.said
    }
}
impl CombatEnvironment<Attributes> for Environment<'_> {'''),
    ('horizon-world/src/server_player_damage.rs', '''    fn damage_event(&mut self) -> Result<(), EnvError<Self>>;
}
pub struct ServerPlayerDamage''', '''    fn damage_event(&mut self) -> Result<(), EnvError<Self>>;
    /// The damage a listener's modifiers came to, and what absorption takes of it.
    fn damage_said(&self) -> Option<[f32; 2]> {
        None
    }
}
pub struct ServerPlayerDamage'''),
    ('horizon-world/src/server_player_damage.rs', '''    fn after_magic(&mut self, v: f32) -> Result<f32, Self::Error> {
        damage_reduction::after_magic(self, v)
    }
    fn absorption(&mut self) -> Result<f32, Self::Error> {
        self.health_binding().absorption().map_err(health_error)
    }
    fn set_absorption(&mut self, v: f32) -> Result<(), Self::Error> {
        self.health_binding()
            .set_absorption(v)
            .map_err(health_error)
    }''', '''    fn after_magic(&mut self, v: f32) -> Result<f32, Self::Error> {
        let damage = damage_reduction::after_magic(self, v)?;
        Ok(self
            .environment
            .damage_said()
            .map_or(damage, |said| said[0] + said[1]))
    }
    fn absorption(&mut self) -> Result<f32, Self::Error> {
        if let Some(said) = self.environment.damage_said() {
            return Ok(said[1]);
        }
        self.health_binding().absorption().map_err(health_error)
    }
    fn set_absorption(&mut self, v: f32) -> Result<(), Self::Error> {
        let v = match self.environment.damage_said() {
            Some(said) => {
                let held = self.health_binding().absorption().map_err(health_error)?;
                (held - said[1]).max(0.0) + v
            }
            None => v,
        };
        self.health_binding()
            .set_absorption(v)
            .map_err(health_error)
    }'''),
    (LIB + 'mobs.rs', '''        if crate::health::hurt_server_canceled(
            crate::health::Hurting::Mob(self, &source),
            &mut amount,
        ) {
            return reflected;
        }
        if amount < 0.0 {
            amount = 0.0;
        }
        if matches!(
            source.kind, DamageKind::FallingBlock | DamageKind::FallingAnvil |
            DamageKind::FallingStalactite
        ) && !species::body_slot(self.species()) && self.extra.head.is_some()
        {
            amount *= 0.75;
        }
        if source.kind == DamageKind::Freeze
            && matches!(
                self.species(), Species::Strider | Species::Blaze | Species::MagmaCube
            )
        {
            amount *= 5.0;
        }
        if !amount.is_finite() {
            amount = f32::MAX;
        }
        let (worn, worn_toughness) = self.worn_armor();
        let covered = if self.species() == Species::Shulker && self.extra.counter == 0 {
            20.0
        } else {
            0.0
        };
        if !source.bypasses_armor() && self.kind.armor + worn + covered > 0.0 {
            let armor = self.kind.armor + worn + covered;
            let toughness = 2.0 + (self.kind.armor_toughness + worn_toughness) / 4.0;
            let effective = (armor - amount / toughness).clamp(armor * 0.2, 20.0);
            let mut fraction = effective / 25.0;
            if let Some(modify) = armor_effectiveness {
                fraction = modify(fraction).clamp(0.0, 1.0);
            }
            amount *= 1.0 - fraction;
        }
        let epf = gear::protection(self, source.kind);
        if epf > 0.0 {
            amount *= 1.0 - epf / 25.0;
        }
        if self.species() == Species::Witch {
            if source.attacker == Some(Who::Mob(self.id)) {
                amount = 0.0;
            } else if matches!(
                source.kind, DamageKind::Magic | DamageKind::IndirectMagic |
                DamageKind::SonicBoom | DamageKind::Thorns
            ) {
                amount *= 0.15;
            }
        }
''', '''        let steps = ee::mob_steps(self, amount, &source, armor_effectiveness);
        let heard = ee::hears("allow_damage");
        if heard {
            crate::health::hit_begins(steps.clone(), amount);
            ee::hit_by(source.attacker);
            ee::row(self);
        }
        let canceled = crate::health::hurt_server_canceled(
            crate::health::Hurting::Mob(self, &source),
            &mut amount,
        );
        let said = heard.then(crate::health::hit_ends).flatten();
        if heard {
            ee::hit_by(None);
            ee::let_go();
        }
        if canceled {
            return reflected;
        }
        amount = match said {
            Some(said) => (said[0] + steps.frames).max(0.0),
            None => steps.work(amount).damage,
        };
'''),

    (LIB + 'game.rs', '''        runtime::borrowed(|game: &mut Game<'static>| crate::game::paper_query::answer(
                game,
                request,
            ))
            .or_else(|| crate::game::paper_fam_ev_entity::free_answer(request))''', '''        if let Some(answer) = horizon_client::mobs::ee::hit_answer(request) {
            return answer;
        }
        runtime::borrowed(|game: &mut Game<'static>| crate::game::paper_query::answer(
                game,
                request,
            ))
            .or_else(|| crate::game::paper_fam_ev_entity::free_answer(request))'''),

    # ---- What a listener can read of the mob whose events are running.
    (GAME + 'creatures.rs', '''                MobEvent::Actor(mob) => horizon_client::mobs::ee::set_actor(mob),''', '''                MobEvent::Actor(mob) => {
                    horizon_client::mobs::ee::set_actor(mob);
                    horizon_client::mobs::ee::hold_actor(self.mobs.iter().find(|m| m.id == mob));
                }'''),

    (LIB + 'hunger.rs', '''            totem_fired: false,
            canceled: false,
        };''', '''            totem_fired: false,
            canceled: false,
            steps: None,
            said: None,
        };'''),

    # ---- What's outside `level_entity`, by its id.
    (GAME + 'potions.rs', '''    previous: Vec3,
    position: Vec3,
    velocity: Vec3,
    age: u32,
    /// `Projectile.getOwner`\'s display name''', '''    previous: Vec3,
    pub(super) position: Vec3,
    pub(super) velocity: Vec3,
    age: u32,
    /// `Projectile.getOwner`\'s display name'''),
    (GAME + 'potions.rs', '''                    landed
                        .push((t.stack.clone(), p, mob, block, t.age, t.owner.clone()));''', '''                    landed
                        .push((t.stack.clone(), p, mob, block, t.age, t.owner.clone(), t.id, t.velocity));'''),
    (GAME + 'potions.rs', '''        for (stack, at, mob, block, age, owner) in landed {
            self.splash(&stack, at, mob, block, age, owner);
        }''', '''        for (stack, at, mob, block, age, owner, id, velocity) in landed {
            super::paper_fam_ev_entity::potion_dying(id, &stack, at, velocity);
            self.splash(&stack, at, mob, block, age, owner);
            super::paper_fam_ev_entity::dying_over();
        }'''),
    (GAME + 'potions.rs', '''    pub(super) fn launch_thrown(
        &mut self,
        stack: ItemStack,
        position: Vec3,
        velocity: Vec3,
        owner: Option<Text>,
    ) {''', '''    pub(super) fn launch_thrown(
        &mut self,
        stack: ItemStack,
        position: Vec3,
        velocity: Vec3,
        owner: Option<Text>,
    ) {
        self.launch_thrown_now(stack, position, velocity, owner);
        super::paper_fam_ev_entity::potion_launched(self);
    }
    fn launch_thrown_now(
        &mut self,
        stack: ItemStack,
        position: Vec3,
        velocity: Vec3,
        owner: Option<Text>,
    ) {'''),
    (GAME + 'potions.rs', '''        if lingering {
            if !def.effects.is_empty() {
                let at = mob.unwrap_or(at);
                self.breath_clouds''', '''        if lingering {
            if !def.effects.is_empty() && !super::paper_fam_ev_entity::cloud_out() {
                let at = mob.unwrap_or(at);
                self.breath_clouds'''),
    (GAME + 'potions.rs', '''    /// `AreaEffectCloud.tick` for the lingering potions\' clouds: the client
    /// tick\'s particles, then the server tick.''', '''    /// The cloud a lingering potion will leave, for its event to hold.
    pub(super) fn lingering_cloud(stack: &ItemStack, at: Vec3, owner: Option<Text>) -> Option<Breath> {
        let def = stack.potion().and_then(brewing::potion)?;
        (!def.effects.is_empty()).then(|| Breath {
            id: 0,
            at: [at.x, at.y, at.z],
            radius: 3.0,
            age: 0,
            duration: 600,
            from_dragon: false,
            potion: Some(PotionCloud {
                effects: def.effects.to_vec(),
                hidden: Vec::new(),
                ambient: Vec::new(),
                radius_per_tick: -3.0 / 600.0,
                color: brewing::potion_color(stack.potion()),
                victims: Vec::new(),
                source: owner.unwrap_or_else(|| Text::translate("entity.minecraft.area_effect_cloud", &[])),
            }),
        })
    }
    /// `AreaEffectCloud.tick` for the lingering potions\' clouds: the client
    /// tick\'s particles, then the server tick.'''),
    (GAME + 'dragon.rs', '''        self.audio
            .play(
                "minecraft:entity.ender_dragon.shoot",
                Some([at.x, at.y, at.z]),
                10.0,
                1.0,
            );
        let velocity = Vec3::new(v.x * 0.1, v.y * 0.1, v.z * 0.1);
        self.dragon_fireballs
            .push(DragonFireball {
                id: 0,
                position: at,
                previous: at,
                velocity,
                age: 0,
            });''', '''        let velocity = Vec3::new(v.x * 0.1, v.y * 0.1, v.z * 0.1);
        self.dragon_fireballs
            .push(DragonFireball {
                id: 0,
                position: at,
                previous: at,
                velocity,
                age: 0,
            });
        if super::paper_fam_ev_entity::fireball_shot(self) {
            return;
        }
        self.audio
            .play(
                "minecraft:entity.ender_dragon.shoot",
                Some([at.x, at.y, at.z]),
                10.0,
                1.0,
            );'''),
    (GAME + 'lightning.rs', '''                    if !b.visual_only {
                        let struck = self.thunder_hit(b);''', '''                    if !b.visual_only {
                        super::paper_fam_ev_entity::bolt_strikes(self, b);
                        let struck = self.thunder_hit(b);
                        super::paper_fam_ev_entity::dying_over();'''),
    (GAME + 'fishing.rs', '''    /// Its entity id, which a server\'s players track it under.
    id: u32,
    pub(super) position: Vec3,
    pub(super) previous: Vec3,
    velocity: Vec3,''', '''    /// Its entity id, which a server\'s players track it under.
    pub(super) id: u32,
    pub(super) position: Vec3,
    pub(super) previous: Vec3,
    pub(super) velocity: Vec3,'''),
    (LIB + 'game.rs', '''                        let kind = if &*potion.stack.item == "minecraft:lingering_potion"
                        {
                            "minecraft:lingering_potion"
                        } else {
                            "minecraft:splash_potion"
                        };''', '''                        let kind = if &*potion.stack.item == "minecraft:lingering_potion"
                        {
                            "minecraft:lingering_potion"
                        } else if &*potion.stack.item == "minecraft:experience_bottle" {
                            "minecraft:experience_bottle"
                        } else {
                            "minecraft:splash_potion"
                        };'''),
    (LIB + 'game.rs', '''answer(kind, [0.0; 3], 0.0, json!({ "Item" : item })),''', '''answer(kind, [potion.position.x, potion.position.y, potion.position.z], 0.0, json!({ "Item" : item, "Motion" : [potion.velocity.x, potion.velocity.y, potion.velocity.z] })),'''),
    (LIB + 'game.rs', '''                    if let Some((at, _)) = game
                        .knot_ids
                        .iter()
                        .find(|(_, knot)| **knot == id)''', '''                    if let Some((kind, at, data)) = super::paper_fam_ev_entity::kept_row(game, id) {
                        return Some(answer(&kind, at, 0.0, data));
                    }
                    if let Some((at, _)) = game
                        .knot_ids
                        .iter()
                        .find(|(_, knot)| **knot == id)'''),
    (LIB + 'game.rs', '''        if let Some(bolt) = game.bolts.iter_mut().find(|bolt| bolt.id == id) {
            if let Some(life) = number("Life") {''', '''        if let Some(done) = super::paper_fam_ev_entity::kept_merge(game, id, patch) {
            return done;
        }
        if let Some(bolt) = game.bolts.iter_mut().find(|bolt| bolt.id == id) {
            if let Some(life) = number("Life") {'''),
    (LIB + 'game.rs', '''                    ids.extend(game.breath_clouds.iter().map(|cloud| cloud.id));
                    ids.retain(|id| *id != 0);''', '''                    ids.extend(game.breath_clouds.iter().map(|cloud| cloud.id));
                    ids.extend(super::paper_fam_ev_entity::kept_ids(game));
                    ids.retain(|id| *id != 0);'''),
    (LIB + 'game.rs', '''                let id = query["id"].as_u64().unwrap_or(0) as u32;
                let vector = |value: &Value| Some([
                    value.get(0)?.as_f64()?,
                    value.get(1)?.as_f64()?,
                    value.get(2)?.as_f64()?,
                ]);
                let done = match (''', '''                let id = query["id"].as_u64().unwrap_or(0) as u32;
                if super::paper_fam_ev_entity::kept_action(game, &query, id) {
                    return "true".to_owned();
                }
                let vector = |value: &Value| Some([
                    value.get(0)?.as_f64()?,
                    value.get(1)?.as_f64()?,
                    value.get(2)?.as_f64()?,
                ]);
                let done = match ('''),

    # ---- Everyone on and off a vehicle, at the moment.
    (GAME + 'boats.rs', '''            if self.tick_cart_riding(keys) || self.tick_mount_riding(keys) {
                return Ok(());
            }
            self.riding = None;''', '''            if self.tick_cart_riding(keys) || self.tick_mount_riding(keys) {
                return Ok(());
            }
            super::paper_fam_ev_entity::vehicle_gone(self);
            self.riding = None;'''),
    (GAME + 'boats.rs', '''        if self.boat_lava_tick(i) {
            self.riding = None;
        }''', '''        if self.boat_lava_tick(i) {
            super::paper_fam_ev_entity::vehicle_gone(self);
            self.riding = None;
        }'''),
    (GAME + 'cushions.rs', '''        self.riding = None;
        let border = self.border_bounds();''', '''        super::paper_fam_ev_entity::vehicle_gone(self);
        self.riding = None;
        let border = self.border_bounds();'''),
    (GAME + 'game_mode.rs', '''        } else {
            self.riding = None;
        }
    }
}

#[cfg(test)]''', '''        } else if !super::paper_fam_ev_entity::dismount_refused(self, id) {
            self.riding = None;
        }
    }
}

#[cfg(test)]'''),
    (GAME + 'minecarts.rs', '''                if self.riding == Some(c.id) {
                    self.riding = None;
                }
                let y = c.position.y;''', '''                if self.riding == Some(c.id) && !super::paper_fam_ev_entity::dismount_refused(self, c.id) {
                    self.riding = None;
                }
                let y = c.position.y;'''),
    (GAME + 'minecarts.rs', '''        let Some(i) = self.carts.iter().position(|x| x.id == c.id) else {
            self.riding = None;
            return true;
        };''', '''        let Some(i) = self.carts.iter().position(|x| x.id == c.id) else {
            super::paper_fam_ev_entity::vehicle_gone(self);
            self.riding = None;
            return true;
        };'''),
    (GAME + 'minecarts.rs', '''            self.eject_cart_mobs(i);
            self.release_vehicle_container(c.id);
            self.carts.remove(i);
            self.riding = None;
        }
        if alive && self.riding.is_none() {''', '''            super::paper_fam_ev_entity::vehicle_gone(self);
            self.eject_cart_mobs(i);
            self.release_vehicle_container(c.id);
            self.carts.remove(i);
            self.riding = None;
        }
        if alive && self.riding.is_none() {'''),
    (GAME + 'screens.rs', '''            // new one is not put back into the vehicle.
            self.riding = None;''', '''            // new one is not put back into the vehicle.
            super::paper_fam_ev_entity::vehicle_gone(self);
            self.riding = None;'''),
    (GAME + 'commands/world_cmds.rs', '''                } else {
                    self.riding = Some(id);
                    self.yaw = self.boats[i].yaw;''', '''                } else {
                    if crate::game::paper_fam_ev_entity::mount_refused(self, id) {
                        return Err(generic());
                    }
                    self.riding = Some(id);
                    self.yaw = self.boats[i].yaw;'''),
    (GAME + 'commands/world_cmds.rs', '''                if target != Target::Player || self.ridden_by_guest(id) {
                    return Err(generic());
                }
                self.riding = Some(id);''', '''                if target != Target::Player || self.ridden_by_guest(id) || crate::game::paper_fam_ev_entity::mount_refused(self, id) {
                    return Err(generic());
                }
                self.riding = Some(id);'''),
    (GAME + 'commands/world_cmds.rs', '''        if let Some(m) = self.mobs.iter_mut().find(|m| m.id == mob_id) {
            m.extra.flag = false;
            m.extra.counter = 0;
            self.yaw = m.yaw;
        }
        self.riding = Some(mob_id);''', '''        if crate::game::paper_fam_ev_entity::mount_refused(self, mob_id) {
            return Err(generic());
        }
        if let Some(m) = self.mobs.iter_mut().find(|m| m.id == mob_id) {
            m.extra.flag = false;
            m.extra.counter = 0;
            self.yaw = m.yaw;
        }
        self.riding = Some(mob_id);'''),
    (GAME + 'commands/world_cmds.rs', '''        let Some(i) = self.mobs.iter().position(|m| m.id == rider) else {
            return;
        };
        let Some(v) = self.mobs[i].extra.mount.take() else {
            return;
        };
        let (width, yaw) = (f64::from(self.mobs[i].width()), self.mobs[i].yaw);''', '''        let Some(i) = self.mobs.iter().position(|m| m.id == rider) else {
            return;
        };
        if self.mobs[i].extra.mount.is_some_and(|v| horizon_client::mobs::ee::mob_off(&self.mobs[i], v, false)) {
            return;
        }
        let Some(v) = self.mobs[i].extra.mount.take() else {
            return;
        };
        let (width, yaw) = (f64::from(self.mobs[i].width()), self.mobs[i].yaw);'''),
    (GAME + 'nautili.rs', '''            // doPlayerRide.
            let id = self.mobs[index].id;
            self.break_leash(id, true, false);''', '''            // doPlayerRide.
            let id = self.mobs[index].id;
            if super::paper_fam_ev_entity::mount_refused(self, id) {
                return true;
            }
            self.break_leash(id, true, false);'''),
    (GAME + 'passengers.rs', '''            } else {
                let m = &mut self.mobs[i];
                m.extra.mount = None;
                m.extra.passenger = false;
            }
        }
        for m in &mut self.mobs {''', '''            } else {
                horizon_client::mobs::ee::mob_off(&self.mobs[i], v, true);
                let m = &mut self.mobs[i];
                m.extra.mount = None;
                m.extra.passenger = false;
            }
        }
        for m in &mut self.mobs {'''),
    (GAME + 'boat_passengers.rs', '''            let m = &mut self.mobs[j];
            m.extra.mount = None;
            m.extra.passenger = false;
            m.extra.boarding_cooldown = BOARDING_COOLDOWN;
            m.position = at;''', '''            if horizon_client::mobs::ee::mob_off(&self.mobs[j], id, broken) {
                continue;
            }
            let m = &mut self.mobs[j];
            m.extra.mount = None;
            m.extra.passenger = false;
            m.extra.boarding_cooldown = BOARDING_COOLDOWN;
            m.position = at;'''),
    (GAME + 'beehives.rs', '''        self.mobs[i].extra.mount = None;
        let tag = self.mob_tree_nbt(i);''', '''        if let Some(v) = self.mobs[i].extra.mount {
            horizon_client::mobs::ee::mob_off(&self.mobs[i], v, true);
        }
        self.mobs[i].extra.mount = None;
        let tag = self.mob_tree_nbt(i);'''),
    (GAME + 'native_mobs.rs', '''        m.ai.nav.stop();
        m.velocity.y = 0.0;
        m.extra.mount = None;''', '''        m.ai.nav.stop();
        m.velocity.y = 0.0;
        if let Some(v) = m.extra.mount {
            horizon_client::mobs::ee::mob_off(m, v, true);
        }
        m.extra.mount = None;'''),

    # ---- /tp of what isn't a player: `EntityTeleportEvent`.
    (GAME + 'commands.rs', '''    fn teleport_to(&mut self, t: Target, pos: [f64; 3], rotation: Option<[f32; 2]>) {
''', '''    fn teleport_to(&mut self, t: Target, mut pos: [f64; 3], rotation: Option<[f32; 2]>) {
        match crate::game::paper_fam_ev_entity::command_teleport(self, t, pos) {
            Some(to) => pos = to,
            None => return,
        }
'''),
    # ---- The bow's draw, for `EntityShootBowEvent.getForce`.
    (LIB + 'game.rs', '''                on_ground,
            );
        self.add_projectile(arrow);
        self.player_projectile_shot();
        let pitch = 1.0 / (self.world.random.next_float() * 0.4 + 1.2) + power * 0.5;''', '''                on_ground,
            );
        paper_fam_ev_entity::note_force(power);
        self.add_projectile(arrow);
        self.player_projectile_shot();
        let pitch = 1.0 / (self.world.random.next_float() * 0.4 + 1.2) + power * 0.5;'''),

    # ---- What Paper has in hand as its event is heard.
    (GAME + 'creatures.rs', '''        if let Some(id) = self
            .spawn_mob(&kind, [p.position.x, p.position.y, p.position.z], true)
            && let Some(child) = self.mobs.iter_mut().find(|m| m.id == id)''', '''        let born = match super::paper_fam_ev_entity::take_child(self) {
            Some(child) => child,
            None => self.spawn_mob(&kind, [p.position.x, p.position.y, p.position.z], true),
        };
        if let Some(id) = born
            && let Some(child) = self.mobs.iter_mut().find(|m| m.id == id)'''),
    (GAME + 'creatures.rs', '''                    self.mobs.push(*m);
                    self.native_mobs_joined(self.mobs.len() - 1, false);
                }
                MobEvent::SpawnKind { kind, at, setup } => {
                    if let Some(id) = self.spawn_mob(&kind, at, setup.persistent) {
                        self.apply_setup(id, &setup);
                    }
                }''', '''                    self.mobs.push(*m);
                    if super::paper_fam_ev_entity::shulker_born(self) {
                        self.mobs.pop();
                    } else {
                        self.native_mobs_joined(self.mobs.len() - 1, false);
                    }
                }
                MobEvent::SpawnKind { kind, at, setup } => {
                    let zap = super::paper_fam_ev_entity::zap_ahead(&kind);
                    if let Some(id) = self.spawn_mob(&kind, at, setup.persistent) {
                        self.apply_setup(id, &setup);
                        super::paper_fam_ev_entity::zapped(self, zap, id, at, &kind);
                    }
                }'''),
    (GAME + 'creatures.rs', '''                MobEvent::DropItem { item, count, at } => {
                    self.spawn_item(''', '''                MobEvent::DropItem { item, count, at } => {
                    super::paper_fam_ev_entity::dropping();
                    self.spawn_item('''),
    (LIB + 'game.rs', '''        if !self.native_entity_joins(self.next_entity, "minecraft:item", position, false)
        {''', '''        if paper_fam_ev_entity::drop_refused(self)
            || !self.native_entity_joins(self.next_entity, "minecraft:item", position, false)
        {'''),
    (GAME + 'native_hooks.rs', '''        for i in from..self.mobs.len() {
            let m = &self.mobs[i];
            let (kind, pos) = (mob_kind(m), [m.position.x, m.position.y, m.position.z]);
            self.native_mob_spawned(self.dim.id(), &kind, pos, natural);
        }''', '''        let mut i = from;
        while i < self.mobs.len() {
            let m = &self.mobs[i];
            let (kind, pos) = (mob_kind(m), [m.position.x, m.position.y, m.position.z]);
            self.native_mob_spawned(self.dim.id(), &kind, pos, natural);
            if super::paper_fam_ev_entity::mob_joins(self, i) {
                i += 1;
            } else {
                self.mobs.remove(i);
            }
        }'''),
    (GAME + 'dragon.rs', '''                    let amount = if c.from_dragon { 6.0 } else { 12.0 };
                    for p in &players {
                        let pp = p.position;
                        if (pp.x - c.at[0]).hypot(pp.z - c.at[2]) <= f64::from(r)
                            && (pp.y - c.at[1]).abs() < 1.0
                        {
                            harmed.push((p.who, amount));
                        }
                    }''', '''                    let amount = if c.from_dragon { 6.0 } else { 12.0 };
                    let mut near = Vec::new();
                    for p in &players {
                        let pp = p.position;
                        if (pp.x - c.at[0]).hypot(pp.z - c.at[2]) <= f64::from(r)
                            && (pp.y - c.at[1]).abs() < 1.0
                        {
                            near.push(p.who);
                        }
                    }
                    for who in super::paper_fam_ev_entity::breath_apply(self, c, near) {
                        harmed.push((who, amount));
                    }'''),
    (MOBS + 'undead.rs', '''                    perform_ranged_attack(m, who, cx, bow_power(pull));''', '''                    super::ee::set_force(m.id, bow_power(pull));
                    perform_ranged_attack(m, who, cx, bow_power(pull));'''),
    (GAME + 'creatures.rs', '''                MobEvent::EffectPlayer { effect, ticks, amplifier } => {
                    self.give_player_effect(effect, ticks, amplifier);''', '''                MobEvent::EffectPlayer { effect, ticks, amplifier } => {
                    super::paper_fam_ev_entity::mob_gives(self, effect);
                    self.give_player_effect(effect, ticks, amplifier);'''),
    (GAME + 'creatures.rs', '''                            self.give_player_effect("minecraft:levitation", 200, 0);''', '''                            horizon_client::mobs::ee::effect_from(p.owner);
                            self.give_player_effect("minecraft:levitation", 200, 0);'''),
    (GAME + 'creatures.rs', '''                            m.add_effect("minecraft:levitation", 200, 0);''', '''                            horizon_client::mobs::ee::effect_cause("ATTACK");
                            horizon_client::mobs::ee::effect_from(p.owner);
                            m.add_effect("minecraft:levitation", 200, 0);'''),
    (GAME + 'potions.rs', '''    pub(super) fn arrow_post_hurt_effects(&mut self, p: &Projectile, who: Who) {
''', '''    pub(super) fn arrow_post_hurt_effects(&mut self, p: &Projectile, who: Who) {
        super::paper_fam_ev_entity::note_cause(self, "ARROW");
'''),

    # ---- EntityPotionEffectEvent.getCause: each site says, by Paper's own table of causes.
    ('horizon-client/src/game/beacon.rs', '        self.potions.effects.add(ambient(primary, amplifier));', '        super::paper_fam_ev_entity::note_cause(self, "BEACON");\n        self.add_player_effect(ambient(primary, amplifier));'),
    ('horizon-client/src/game/beacon.rs', '            self.potions.effects.add(ambient(s, 0));', '            self.add_player_effect(ambient(s, 0));'),
    ('horizon-client/src/game/block_entity_anims.rs', '            g.potions.effects.add(power);', '            super::paper_fam_ev_entity::note_cause(g, "CONDUIT");\n            g.add_player_effect(power);'),
    ('horizon-client/src/game/block_entity_anims.rs', '                                m.add_effect("minecraft:glowing", 60, 0);', '                                horizon_client::mobs::ee::effect_cause("UNKNOWN");\n                                m.add_effect("minecraft:glowing", 60, 0);'),
    ('horizon-client/src/game/consume.rs', '            for (id, duration) in &data.stew_effects {', '            super::paper_fam_ev_entity::note_cause(self, "FOOD");\n            for (id, duration) in &data.stew_effects {'),
    ('horizon-client/src/game/creatures.rs', '                        m.add_effect("minecraft:nausea", 200, 0);', '                        horizon_client::mobs::ee::effect_cause("UNKNOWN");\n                        m.add_effect("minecraft:nausea", 200, 0);'),
    ('horizon-client/src/game/creatures.rs', '        if setup.nausea {\n            m.add_effect("minecraft:nausea", 200, 0);', '        if setup.nausea {\n            horizon_client::mobs::ee::effect_cause("UNKNOWN");\n            m.add_effect("minecraft:nausea", 200, 0);'),
    ('horizon-client/src/game/creatures.rs', '        self.give_player_effect("minecraft:wither", 40, 0);', '        super::paper_fam_ev_entity::cause_next(self, "WITHER_ROSE");\n        self.give_player_effect("minecraft:wither", 40, 0);'),
    ('horizon-client/src/game/creatures.rs', '                            e if t > 20 => self.give_player_effect(e, t, *amplifier),', '                            e if t > 20 => {\n                                super::paper_fam_ev_entity::cause_next(self, "POTION_SPLASH");\n                                self.give_player_effect(e, t, *amplifier)\n                            }'),
    ('horizon-client/src/game/creatures.rs', '                                m.add_effect(effect, t, *amplifier);', '                                horizon_client::mobs::ee::effect_cause("POTION_SPLASH");\n                                m.add_effect(effect, t, *amplifier);'),
    ('horizon-client/src/game/creatures.rs', '                m.add_effect("minecraft:poison", 900, 0);', '                horizon_client::mobs::ee::effect_cause("FOOD");\n                m.add_effect("minecraft:poison", 900, 0);'),
    ('horizon-client/src/game/creatures.rs', '                    m.add_effect("minecraft:wither", 40, 0);', '                    horizon_client::mobs::ee::effect_cause("FOOD");\n                    m.add_effect("minecraft:wither", 40, 0);'),
    ('horizon-client/src/game/creatures.rs', '                    m.add_effect("minecraft:poison", 25, 0);', '                    horizon_client::mobs::ee::effect_cause("FOOD");\n                    m.add_effect("minecraft:poison", 25, 0);'),
    ('horizon-client/src/game/potions.rs', '            self.add_player_effect(e);', '            super::paper_fam_ev_entity::note_cause(self, "POTION_DRINK");\n            self.add_player_effect(e);'),
    ('horizon-client/src/game/potions.rs', '            self.potions.effects.add(effect);', '            super::paper_fam_ev_entity::note_cause(self, "TURTLE_HELMET");\n            self.add_player_effect(effect);'),
    ('horizon-client/src/game/enchant_effects.rs', '            m.add_effect(id, ticks, amplifier);', '            horizon_client::mobs::ee::effect_cause("ATTACK");\n            m.add_effect(id, ticks, amplifier);'),
    ('horizon-client/src/game/potent_sulfur.rs', '        mob.add_effect(nausea, EFFECT_DURATION_IN_TICKS, 0);', '        horizon_client::mobs::ee::effect_cause("UNKNOWN");\n        mob.add_effect(nausea, EFFECT_DURATION_IN_TICKS, 0);'),
    ('horizon-client/src/game/potent_sulfur.rs', '            self.add_player_effect(nausea);', '            super::paper_fam_ev_entity::note_cause(self, "UNKNOWN");\n            self.add_player_effect(nausea);'),
    ('horizon-client/src/game/raids.rs', '                m.add_effect("glowing", 1000, 1);', '                horizon_client::mobs::ee::effect_cause("UNKNOWN");\n                m.add_effect("glowing", 1000, 1);'),
    ('horizon-client/src/game/raids.rs', '                        g.potions.effects.add(effect);', '                        super::paper_fam_ev_entity::note_cause(g, "UNKNOWN");\n                        g.add_player_effect(effect);'),
    ('horizon-client/src/game/raids.rs', '        self.potions.effects.add(effect);', '        super::paper_fam_ev_entity::note_cause(self, "UNKNOWN");\n        self.add_player_effect(effect);'),
    ('horizon-client/src/game/sculk.rs', '        self.potions.effects.add(effect);', '        super::paper_fam_ev_entity::note_cause(self, "WARDEN");\n        self.add_player_effect(effect);'),
    ('horizon-client/src/game/trading.rs', '                        m.add_effect("minecraft:regeneration", 200, 0);', '                        horizon_client::mobs::ee::effect_cause("VILLAGER_TRADE");\n                        m.add_effect("minecraft:regeneration", 200, 0);'),
    ('horizon-client/src/game/wither.rs', '                            m.add_effect("minecraft:wither", wither_ticks, 1);', '                            horizon_client::mobs::ee::effect_cause("ATTACK");\n                            m.add_effect("minecraft:wither", wither_ticks, 1);'),
    ('horizon-client/src/game/zombie_villagers.rs', '    to.add_effect("minecraft:nausea", 200, 0);', '    horizon_client::mobs::ee::effect_cause("CONVERSION");\n    to.add_effect("minecraft:nausea", 200, 0);'),
    ('horizon-client/src/mobs/animal_brain.rs', '                m.add_effect("minecraft:regeneration", 200, 0);', '                super::ee::effect_cause("AXOLOTL");\n                m.add_effect("minecraft:regeneration", 200, 0);'),
    ('horizon-client/src/mobs/finalize.rs', '            m.add_effect(effect, -1, 0);', '            super::ee::effect_cause("SPIDER_SPAWN");\n            m.add_effect(effect, -1, 0);'),
    ('horizon-client/src/mobs/illusioner.rs', '            m.add_effect(INVISIBILITY, 1200, 0);', '            super::ee::effect_cause("ILLUSION");\n            m.add_effect(INVISIBILITY, 1200, 0);'),
    ('horizon-client/src/mobs/zombie_villager.rs', '    m.add_effect("minecraft:strength", time, 0);', '    super::ee::effect_cause("CONVERSION");\n    m.add_effect("minecraft:strength", time, 0);'),
    ('horizon-client/src/mobs/ai.rs', '                                m.add_effect("minecraft:wither", 40, 0);', '                                super::ee::effect_cause("WITHER_ROSE");\n                                m.add_effect("minecraft:wither", 40, 0);'),
    ('horizon-client/src/mobs/ai.rs', '                            m.add_effect("minecraft:poison", 25, 0);', '                            super::ee::effect_cause("UNKNOWN");\n                            m.add_effect("minecraft:poison", 25, 0);'),

    # ---- Paper's order at a death: the loot, the event, and then the kill's credit.
    (GAME + 'creatures.rs', '''        if by_player {
            let guest = killer.flatten().filter(|_| self.lan_acting.is_none());
            let credited = guest
                .is_some_and(|conn| {
                    let Some(mut host) = self.lan.take() else {
                        return false;
                    };
                    let done = self
                        .as_guest(&mut host, conn, |g| g.credit_kill(m))
                        .is_some();
                    self.lan = Some(host);
                    done
                });
            if !credited {
                self.credit_kill(m);
            }
        }
''', ''),
    (GAME + 'creatures.rs', '''        self.create_wither_rose(m);
        if m.species() == Species::CopperGolem {''', '''        self.create_wither_rose(m);
        if !super::paper_fam_ev_entity::was_revived(m.id) {
            if by_player {
                let guest = killer.flatten().filter(|_| self.lan_acting.is_none());
                let credited = guest
                    .is_some_and(|conn| {
                        let Some(mut host) = self.lan.take() else {
                            return false;
                        };
                        let done = self
                            .as_guest(&mut host, conn, |g| g.credit_kill(m))
                            .is_some();
                        self.lan = Some(host);
                        done
                    });
                if !credited {
                    self.credit_kill(m);
                }
            }
        }
        if m.species() == Species::CopperGolem {'''),
    (LIB + 'game.rs', '''        if paper_fam_ev_entity::drop_refused(self)
            || !self.native_entity_joins(self.next_entity, "minecraft:item", position, false)
        {''', '''        if paper_fam_ev_entity::drop_refused(self)
            || (!paper_fam_ev_entity::death_window()
                && !self.native_entity_joins(self.next_entity, "minecraft:item", position, false))
        {'''),
    # ---- After the checks, as a predicate in `getPlayers` and past the odds.
    (GAME + 'creatures.rs', '''        let at = [p.position.x, p.position.y, p.position.z];
        self.give_player_effect("minecraft:mining_fatigue", 6000, 2);''', '''        let elder_at = at;
        let at = [p.position.x, p.position.y, p.position.z];
        if super::paper_fam_ev_entity::elder_refused(self, elder_at) {
            return false;
        }
        self.give_player_effect("minecraft:mining_fatigue", 6000, 2);'''),
    (GAME + 'stats.rs', '''        let group = 1 + self.world.random.next_int_bound(3).unwrap_or(0);
        for _ in 0..group {''', '''        let group = 1 + self.world.random.next_int_bound(3).unwrap_or(0);
        if super::paper_fam_ev_entity::phantom_refused(self) {
            return;
        }
        for _ in 0..group {'''),

    # ---- Reasons that come from what's being carried out.
    (GAME + 'creatures.rs', '''                    let zap = super::paper_fam_ev_entity::zap_ahead(&kind);''', '''                    horizon_client::mobs::ee::why(super::paper_fam_ev_entity::spawn_kind_why(&kind));
                    let zap = super::paper_fam_ev_entity::zap_ahead(&kind);'''),
    (GAME + 'creatures.rs', '''                    self.mobs.push(*m);
                    if super::paper_fam_ev_entity::shulker_born(self) {''', '''                    self.mobs.push(*m);
                    horizon_client::mobs::ee::why("BREEDING");
                    if super::paper_fam_ev_entity::shulker_born(self) {'''),
    (GAME + 'creatures.rs', '''        let born = match super::paper_fam_ev_entity::take_child(self) {''', '''        horizon_client::mobs::ee::why("BREEDING");
        let born = match super::paper_fam_ev_entity::take_child(self) {'''),
    (GAME + 'native_hooks.rs', '''        let mut i = from;
        while i < self.mobs.len() {
            let m = &self.mobs[i];''', '''        if natural {
            horizon_client::mobs::ee::why("NATURAL");
        }
        let mut i = from;
        while i < self.mobs.len() {
            let m = &self.mobs[i];'''),
    (GAME + 'golems.rs', '''        kind: &str,
        at: [f64; 3],
        yaw: f32,
    ) -> Option<u32> {
''', '''        kind: &str,
        at: [f64; 3],
        yaw: f32,
    ) -> Option<u32> {
        horizon_client::mobs::ee::why(super::paper_fam_ev_entity::built_why(kind));
'''),
    # ---- `EntityBreakEvent` for a frame.
    (GAME + 'frames.rs', '''        let f = if with_frame {
            self.item_frames.remove(i)''', '''        if with_frame && super::paper_fam_ev_entity::frame_refused(self, i, "ENTITY") {
            return true;
        }
        let f = if with_frame {
            self.item_frames.remove(i)'''),
    (GAME + 'frames.rs', '''                if !self.frame_survives(&f) {
                    self.item_frames.remove(i);''', '''                if !self.frame_survives(&f) && !super::paper_fam_ev_entity::frame_refused(self, i, "PHYSICS") {
                    self.item_frames.remove(i);'''),
    # ---- Why a mob comes into the level, by Paper's own table of spawn reasons.
    ('horizon-client/src/game/allays.rs', '            if let Some(id) = self.spawn_mob("minecraft:allay", pos, true)', '            horizon_client::mobs::ee::why("DUPLICATION");\n            if let Some(id) = self.spawn_mob("minecraft:allay", pos, true)'),
    ('horizon-client/src/game/cat_spawner.rs', '        let Some(id) = self.spawn_mob("minecraft:cat", at, persistent) else {', '        horizon_client::mobs::ee::why("NATURAL");\n        let Some(id) = self.spawn_mob("minecraft:cat", at, persistent) else {'),
    ('horizon-client/src/game/commands/raid.rs', '                match self.spawn_mob_for(kind, [player.x, player.y, player.z], true, true) {', '                horizon_client::mobs::ee::why("DEFAULT");\n                match self.spawn_mob_for(kind, [player.x, player.y, player.z], true, true) {'),
    ('horizon-client/src/game/commands.rs', '            self.spawn_mob_spawned(&short, pos);', '            horizon_client::mobs::ee::why("COMMAND");\n            self.spawn_mob_spawned(&short, pos);'),
    ('horizon-client/src/game/copper_golems.rs', '        let Some(id) = self.spawn_mob("minecraft:copper_golem", at, true) else {', '        horizon_client::mobs::ee::why("REANIMATE");\n        let Some(id) = self.spawn_mob("minecraft:copper_golem", at, true) else {'),
    ('horizon-client/src/game/creatures.rs', '                        if let Some(id) = self.spawn_mob(&m.kind.id, at, m.persistent)', '                        horizon_client::mobs::ee::why("SLIME_SPLIT");\n                        if let Some(id) = self.spawn_mob(&m.kind.id, at, m.persistent)'),
    ('horizon-client/src/game/creatures.rs', '            if let Some(id) = self.spawn_mob("minecraft:turtle", at, true)', '            horizon_client::mobs::ee::why("EGG");\n            if let Some(id) = self.spawn_mob("minecraft:turtle", at, true)'),
    ('horizon-client/src/game/creatures.rs', '                    if let Some(id) = self.spawn_mob(&kind, atp, true)', '                    horizon_client::mobs::ee::why("EGG");\n                    if let Some(id) = self.spawn_mob(&kind, atp, true)'),
    ('horizon-client/src/game/creatures.rs', '        let target = self.mob_entity(&self.mobs[index]);\n        let Some(id) = self.spawn_mob_spawned(&kind, at) else {', '        let target = self.mob_entity(&self.mobs[index]);\n        horizon_client::mobs::ee::why("SPAWNER_EGG");\n        let Some(id) = self.spawn_mob_spawned(&kind, at) else {'),
    ('horizon-client/src/game/creatures.rs', '        let at = [f64::from(p[0]) + 0.5, f64::from(p[1]), f64::from(p[2]) + 0.5];\n        let Some(id) = self.spawn_mob_spawned(&kind, at) else {', '        let at = [f64::from(p[0]) + 0.5, f64::from(p[1]), f64::from(p[2]) + 0.5];\n        horizon_client::mobs::ee::why("SPAWNER_EGG");\n        let Some(id) = self.spawn_mob_spawned(&kind, at) else {'),
    ('horizon-client/src/game/creatures.rs', '        if let Some(id) = self.spawn_mob("minecraft:zombified_piglin", at, false) {', '        horizon_client::mobs::ee::why("NETHER_PORTAL");\n        if let Some(id) = self.spawn_mob("minecraft:zombified_piglin", at, false) {'),
    ('horizon-client/src/game/dispenser.rs', '        let Some(id) = self.spawn_mob_at(&kind, front, facing != Direction::Up) else {', '        horizon_client::mobs::ee::why("DISPENSE_EGG");\n        let Some(id) = self.spawn_mob_at(&kind, front, facing != Direction::Up) else {'),
    ('horizon-client/src/game/dispenser.rs', '            if let Some(id) = self.spawn_mob_at(kind, target, true) {', '            horizon_client::mobs::ee::why("BUCKET");\n            if let Some(id) = self.spawn_mob_at(kind, target, true) {'),
    ('horizon-client/src/game/ghasts.rs', '        if let Some(id) = self.spawn_mob("minecraft:happy_ghast", at, true)', '        horizon_client::mobs::ee::why("REHYDRATION");\n        if let Some(id) = self.spawn_mob("minecraft:happy_ghast", at, true)'),
    ('horizon-client/src/game/native_mobs.rs', '        let id = self.spawn_mob(kind, pos, true)?;', '        horizon_client::mobs::ee::why("CUSTOM");\n        let id = self.spawn_mob(kind, pos, true)?;'),
    ('horizon-client/src/game/passengers.rs', '                if let Some(skeleton) = self.spawn_mob("minecraft:skeleton", at, false) {', '                horizon_client::mobs::ee::why("JOCKEY");\n                if let Some(skeleton) = self.spawn_mob("minecraft:skeleton", at, false) {'),
    ('horizon-client/src/game/passengers.rs', '                if let Some(zombie) = self.spawn_mob("minecraft:zombie", at, false) {', '                horizon_client::mobs::ee::why("JOCKEY");\n                if let Some(zombie) = self.spawn_mob("minecraft:zombie", at, false) {'),
    ('horizon-client/src/game/passengers.rs', '        let Some(camel) = self.spawn_mob("minecraft:camel_husk", at, false) else {', '        horizon_client::mobs::ee::why("JOCKEY");\n        let Some(camel) = self.spawn_mob("minecraft:camel_husk", at, false) else {'),
    ('horizon-client/src/game/passengers.rs', '        if let Some(n) = self.spawn_mob("minecraft:zombie_nautilus", at, false) {', '        horizon_client::mobs::ee::why("JOCKEY");\n        if let Some(n) = self.spawn_mob("minecraft:zombie_nautilus", at, false) {'),
    ('horizon-client/src/game/passengers.rs', '            if let Some(piglin) = self.spawn_mob("minecraft:zombified_piglin", at, false)', '            horizon_client::mobs::ee::why("JOCKEY");\n            if let Some(piglin) = self.spawn_mob("minecraft:zombified_piglin", at, false)'),
    ('horizon-client/src/game/passengers.rs', '            let Some(other) = self.spawn_mob("minecraft:skeleton_horse", at, true) else {', '            horizon_client::mobs::ee::why("TRAP");\n            let Some(other) = self.spawn_mob("minecraft:skeleton_horse", at, true) else {'),
    ('horizon-client/src/game/passengers.rs', '        let id = self.spawn_mob("minecraft:skeleton", at, true)?;', '        horizon_client::mobs::ee::why("TRAP");\n        let id = self.spawn_mob("minecraft:skeleton", at, true)?;'),
    ('horizon-client/src/game/potions.rs', '            if let Some(id) = self.spawn_mob_for("minecraft:silverfish", at, false, true)', '            horizon_client::mobs::ee::why("POTION_EFFECT");\n            if let Some(id) = self.spawn_mob_for("minecraft:silverfish", at, false, true)'),
    ('horizon-client/src/game/sniffers.rs', '        if let Some(id) = self.spawn_mob("minecraft:sniffer", at, true)', '        horizon_client::mobs::ee::why("EGG");\n        if let Some(id) = self.spawn_mob("minecraft:sniffer", at, true)'),
    ('horizon-client/src/game/sulfur_cubes.rs', '            if let Some(id) = self.spawn_mob(&m.kind.id, at, m.persistent)', '            horizon_client::mobs::ee::why("SLIME_SPLIT");\n            if let Some(id) = self.spawn_mob(&m.kind.id, at, m.persistent)'),
    ('horizon-client/src/game/wandering_trader.rs', '        let Some(id) = self.spawn_mob("minecraft:wandering_trader", at, true) else {', '        horizon_client::mobs::ee::why("NATURAL");\n        let Some(id) = self.spawn_mob("minecraft:wandering_trader", at, true) else {'),
    ('horizon-client/src/game/wandering_trader.rs', '        if let Some(id) = self.spawn_mob("minecraft:trader_llama", spot, false)', '        horizon_client::mobs::ee::why("NATURAL");\n        if let Some(id) = self.spawn_mob("minecraft:trader_llama", spot, false)'),
    ('horizon-client/src/game/zombie_reinforcements.rs', '        let Some(id) = self.spawn_mob(&kind.id, spot.map(f64::from), false) else {', '        horizon_client::mobs::ee::why("REINFORCEMENTS");\n        let Some(id) = self.spawn_mob(&kind.id, spot.map(f64::from), false) else {'),
    ('horizon-client/src/game/zombie_villagers.rs', '        let Some(id) = self.spawn_mob("minecraft:zombie_villager", at, old.persistent) else {', '        horizon_client::mobs::ee::why("INFECTION");\n        let Some(id) = self.spawn_mob("minecraft:zombie_villager", at, old.persistent) else {'),
    ('horizon-client/src/game/zombie_villagers.rs', '        let Some(vid) = self.spawn_mob("minecraft:villager", at, old.persistent) else {', '        horizon_client::mobs::ee::why("CURED");\n        let Some(vid) = self.spawn_mob("minecraft:villager", at, old.persistent) else {'),
    ('horizon-client/src/game.rs', '                        let _ = self.spawn_mob("minecraft:tadpole", spawn, true);', '                        horizon_client::mobs::ee::why("EGG");\n                        let _ = self.spawn_mob("minecraft:tadpole", spawn, true);'),
    ('horizon-client/src/game.rs', '                    if let Some(id) = self.spawn_mob("minecraft:silverfish", at, false) {', '                    horizon_client::mobs::ee::why("SILVERFISH_BLOCK");\n                    if let Some(id) = self.spawn_mob("minecraft:silverfish", at, false) {'),

    # ---- `AbstractMinecart`'s side of `EntityCollideWithEntityEvent`.
    (GAME + 'minecarts.rs', '''            if j == i || !hitbox.intersects(self.carts[j].bounds()) {
                continue;
            }
            let (this, other) = (&self.carts[j], &self.carts[i]);''', '''            if j == i || !hitbox.intersects(self.carts[j].bounds()) {
                continue;
            }
            if super::paper_fam_ev_entity::carts_collide(self, i, j) {
                continue;
            }
            let (this, other) = (&self.carts[j], &self.carts[i]);'''),

    # ---- `PreCreatureSpawnEvent` at each natural try, and the spawners by name.
    (MOBS + 'spawn_selection.rs', '''            let listed = *picked_at == [x, y, z];
''', '''            match super::ee::natural_attempt(&entry.entity, [xx, f64::from(y), zz]) {
                1 => continue,
                2 => return,
                _ => {}
            }
            let listed = *picked_at == [x, y, z];
'''),
    (GAME + 'spawner.rs', '''        let ids = self.add_planned(planned);
''', '''        horizon_client::mobs::ee::why("SPAWNER");
        let ids = self.add_planned(planned);
'''),
    (GAME + 'trial_spawner.rs', '''        let ids = self.add_planned(planned);
''', '''        horizon_client::mobs::ee::why("TRIAL_SPAWNER");
        let ids = self.add_planned(planned);
'''),

    # ---- Heard once the thing stands, and only for what it's about.
    (GAME + 'boats.rs', '''        let Some(boat) = self.boat_to_place(stack) else {
            return false;
        };
''', '''        let Some(boat) = self.boat_to_place(stack) else {
            return false;
        };
        // It stands for its event, and goes back to be added the game's own way.
        let id = boat.id;
        self.boats.push(boat);
        let hit = self.pick().map(|hit| (hit.pos, hit.face));
        let refused = super::paper_fam_ev_entity::placed(self, id, stack, hit);
        let at = self.boats.iter().position(|boat| boat.id == id);
        let Some(boat) = at.map(|at| self.boats.remove(at)).filter(|_| !refused) else {
            return false;
        };
'''),
    (GAME + 'minecarts.rs', '''        let cart = Minecart::new(self.next_entity, kind, at);
''', '''        let cart = Minecart::new(self.next_entity, kind, at);
        let id = cart.id;
        self.carts.push(cart);
        let refused = super::paper_fam_ev_entity::placed(self, id, stack, Some((hit.pos, hit.face)));
        let at = self.carts.iter().position(|cart| cart.id == id);
        let Some(cart) = at.map(|at| self.carts.remove(at)).filter(|_| !refused) else {
            return false;
        };
'''),    (GAME + 'armor_stands.rs', '''        self.armor_stands.push(stand);
        if !self.creative {
            self.consume_held();
        }''', '''        let id = stand.id;
        self.armor_stands.push(stand);
        if super::paper_fam_ev_entity::placed(self, id, stack, Some((hit.pos, hit.face))) {
            self.armor_stands.retain(|stand| stand.id != id);
            return false;
        }
        if !self.creative {
            self.consume_held();
        }'''),
    (GAME + 'dragon.rs', '''                invulnerable: false,
            });
        self.world.modified.insert((above[0] >> 4, above[2] >> 4));''', '''                invulnerable: false,
            });
        let id = self.next_entity;
        if super::paper_fam_ev_entity::placed(self, id, stack, Some((hit.pos, hit.face))) {
            self.crystals.retain(|crystal| crystal.id != id);
            return false;
        }
        self.world.modified.insert((above[0] >> 4, above[2] >> 4));'''),
    (GAME + 'combat.rs', '''        if !self.ui.rules.bool("show_death_messages") {
            return;
        }
        let host = self.player.is_some() && self.options.dedicated.is_none()''', '''        if !self.ui.rules.bool("show_death_messages") {
            return;
        }
        if super::paper_fam_ev_entity::pet_message_refused(self, m) {
            return;
        }
        let host = self.player.is_some() && self.options.dedicated.is_none()'''),
    (GAME + 'potions.rs', '''        super::paper_fam_ev_entity::note_cause(self, "ARROW");
''', '''        super::paper_fam_ev_entity::note_cause(self, "ARROW");
        super::paper_fam_ev_entity::arrow_stuck(self, p, who);
'''),

    # ---- `FallingBlockEntity`'s two moments of `EntityChangeBlockEvent`.
    (LIB + 'game.rs', '''                            damage,
                            saved: None,
                        });
                }
                LevelEvent::Player { pos, effect, .. } => {''', '''                            damage,
                            saved: None,
                        });
                    paper_fam_ev_entity::falling_starts(self, [pos.x, pos.y, pos.z], state);
                }
                LevelEvent::Player { pos, effect, .. } => {'''),
    (LIB + 'game.rs', '''                Ok(FallStep::Landed(pos)) => {
''', '''                Ok(FallStep::Landed(pos)) => {
                    if paper_fam_ev_entity::falling_lands(self, &mut f.id, f.state, pos) {
                        continue;
                    }
'''),
]
