// Family entity: answers to the queries its SDK file makes. Most of a mob's live state isn't saved, so a
// plugin's `Panda.isRolling` reads the fields here. A key names one fact; the SDK says which class it serves.
use crate::game::Game;
use crate::game::commands::Target;
use horizon_client::data_cmd::merged;
use horizon_client::lan::data::component_nbt;
use horizon_client::items::ItemStack;
use horizon_client::mobs::{self, Mob, Species, Who, goals::Cx};
use horizon_core::geometry::Vec3;
use horizon_core::random::{LegacyRandom, RandomSource};
use horizon_protocol::packets::play::{self, Clientbound as C};
use horizon_world::nbt::{Tag, parse_snbt};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

fn id_of(query: &Value) -> u32 {
    query["id"].as_u64().unwrap_or(0) as u32
}

fn dimension(query: &Value) -> String {
    query["dimension"].as_str().unwrap_or("minecraft:overworld").to_owned()
}

fn block(value: &Value) -> Option<[i32; 3]> {
    let at = |i: usize| value.get(i)?.as_i64().map(|v| v as i32);
    Some([at(0)?, at(1)?, at(2)?])
}

fn vector(value: &Value) -> Option<[f64; 3]> {
    let at = |i: usize| value.get(i)?.as_f64();
    Some([at(0)?, at(1)?, at(2)?])
}

fn spot(p: Option<[i32; 3]>) -> Value {
    p.map_or(Value::Null, |p| json!(p))
}

fn who(value: &Value) -> Option<Who> {
    if value["player"].as_bool() == Some(true) {
        return Some(Who::Player);
    }
    value["entity"].as_u64().map(|id| Who::Mob(id as u32))
}

fn who_json(who: Option<Who>) -> Value {
    match who {
        Some(Who::Player) => json!({ "player": true }),
        Some(Who::Mob(id)) => json!({ "entity": id }),
        None => Value::Null,
    }
}

/// The player a mob's `Who::Player` stands for: the guest bound to it, else the nearest to `at`, else the only one.
fn bound_player(game: &Game<'_>, mob: Option<u32>, at: Option<Vec3>) -> Option<String> {
    let host = game.lan.as_ref()?;
    let bound = mob.and_then(|id| host.mob_players.get(&id)).and_then(|conn| host.hub.guest(*conn));
    let near = |at: Vec3| {
        let away = |guest: &horizon_client::lan::guest::Guest| (0..3).map(|i| (guest.position[i] - [at.x, at.y, at.z][i]).powi(2)).sum::<f64>();
        host.hub.guests.iter().min_by(|a, b| away(a).total_cmp(&away(b)))
    };
    let guest = bound.or_else(|| at.and_then(near)).or_else(|| (host.hub.guests.len() == 1).then(|| &host.hub.guests[0]))?;
    Some(format!("{:032x}", guest.uuid))
}

/// `who_json`, with the player named: a plugin gets the `Player`, not only that there is one.
fn who_named(game: &Game<'_>, mob: Option<u32>, at: Option<Vec3>, who: Option<Who>) -> Value {
    match who {
        Some(Who::Player) => json!({ "player": true, "uuid": bound_player(game, mob, at) }),
        other => who_json(other),
    }
}

/// `Who::Player` means this player, for this mob, from here on.
fn bind_player(game: &mut Game<'_>, mob: u32, query: &Value) {
    let Some(words) = query["uuid"].as_array() else {
        return;
    };
    let words: Vec<i32> = words.iter().map(int).collect();
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    if let Some(conn) = host.hub.guests.iter().find(|guest| horizon_client::lan::playerdata::uuid_ints(guest.uuid).as_slice() == words.as_slice()).map(|guest| guest.conn) {
        host.mob_players.insert(mob, conn);
    }
}

/// Who lit a fuse: a player by UUID, or anything living by its id.
#[derive(Clone, Copy, PartialEq)]
enum Source {
    Player(u128),
    Mob(u32),
}

impl Source {
    fn read(value: &Value) -> Option<Self> {
        if let Some(words) = value["uuid"].as_array().filter(|words| words.len() == 4) {
            return Some(Self::Player(horizon_client::lan::playerdata::ints_uuid([int(&words[0]), int(&words[1]), int(&words[2]), int(&words[3])])));
        }
        if let Some(uuid) = value["uuid"].as_str() {
            return u128::from_str_radix(uuid, 16).ok().map(Self::Player);
        }
        value["entity"].as_u64().map(|id| Self::Mob(id as u32))
    }
    fn json(self) -> Value {
        match self {
            Self::Player(uuid) => json!({ "uuid": format!("{uuid:032x}") }),
            Self::Mob(id) => json!({ "entity": id }),
        }
    }
}

fn int(value: &Value) -> i32 {
    value.as_i64().unwrap_or(0) as i32
}

fn truth(value: &Value) -> bool {
    value.as_bool().unwrap_or(false)
}

fn in_mob_level<R>(game: &mut Game<'_>, query: &Value, f: impl FnOnce(&mut Game<'_>, usize) -> R) -> Option<R> {
    let id = id_of(query);
    game.native_in_level(&dimension(query), |game| {
        let index = game.mobs.iter().position(|m| m.id == id)?;
        Some(f(game, index))
    })
    .flatten()
}

/// `f` with the mob out of the list, so it can use the world: what it does to the world comes back as events.
fn with_world<R>(game: &mut Game<'_>, index: usize, f: impl FnOnce(&mut Mob, &mut Cx<'_, '_>) -> R) -> R {
    let mut mob = game.mobs[index].clone();
    let mut events = Vec::new();
    let mut random = LegacyRandom::new(game.world.game_time);
    let answer = {
        let world = game.mob_world_base(&[]);
        let mut cx = Cx { w: &world, lr: &mut random, events: &mut events };
        f(&mut mob, &mut cx)
    };
    game.mobs[index] = mob;
    if let Err(error) = game.handle_mob_events(events) {
        log::error!("a plugin's mob action: {error}");
    }
    answer
}

fn panda(m: &mut Mob) -> &mut mobs::panda::State {
    m.extra.panda.get_or_insert_with(Default::default)
}

fn panda_flag(m: &Mob, bit: u8) -> bool {
    mobs::panda::get(m).is_some_and(|state| state.flags & bit != 0)
}

fn set_panda_flag(m: &mut Mob, bit: u8, on: bool) {
    let state = panda(m);
    state.flags = if on { state.flags | bit } else { state.flags & !bit };
}

fn fox_flag(m: &mut Mob, bit: i32, on: bool) {
    m.extra.ints[0] = if on { m.extra.ints[0] | bit } else { m.extra.ints[0] & !bit };
}

fn bee(m: &mut Mob) -> Option<&mut mobs::bee::Bee> {
    m.extra.bee.as_deref_mut()
}

/// The ticks of anger left, for a neutral mob or a bee.
fn anger_ticks(m: &Mob, now: i64) -> i32 {
    match mobs::bee::get(m) {
        Some(bee) => mobs::anger_left(bee.anger_end, now),
        None => mobs::anger_left(m.extra.anger_end, now).max(m.extra.anger),
    }
}

fn set_anger_ticks(m: &mut Mob, now: i64, ticks: i32) {
    let end = if ticks > 0 { now + i64::from(ticks) } else { -1 };
    if let Some(bee) = bee(m) {
        bee.anger_end = end;
        return;
    }
    m.extra.anger_end = end;
    m.extra.anger = ticks.max(0);
    if ticks <= 0 {
        m.extra.angry_at = None;
    }
}

/// `true` for a fact a mob's fields hold, and the facts are read at once.
fn read(m: &Mob, key: &str, now: i64) -> Option<Value> {
    use mobs::panda::{FLAG_ON_BACK, FLAG_ROLL, FLAG_SNEEZE};
    let raider = mobs::raider::state(m);
    Some(match key {
        "panda_sneezing" => json!(panda_flag(m, FLAG_SNEEZE)),
        "panda_rolling" => json!(panda_flag(m, FLAG_ROLL)),
        "panda_on_back" => json!(panda_flag(m, FLAG_ON_BACK)),
        "panda_eating" => json!(mobs::panda::get(m).is_some_and(|state| state.eat > 0)),
        "panda_scared" => json!(mobs::panda::scared(m)),
        "panda_unhappy" => json!(mobs::panda::get(m).map_or(0, |state| state.unhappy)),
        "panda_sneeze_ticks" => json!(mobs::panda::get(m).map_or(0, |state| state.sneeze)),
        "panda_eat_ticks" => json!(mobs::panda::get(m).map_or(0, |state| state.eat)),
        "panda_gene" => json!(mobs::panda::variant(m)),
        "horse_grazing" => json!(mobs::equine::grazes(m.species()) && m.extra.flag),
        "horse_rearing" => json!(mobs::equine::grazes(m.species()) && m.extra.counter > 0),
        "horse_max_temper" => json!(mobs::equine::max_temper(m.species())),
        "bee_anger" | "anger" => json!(anger_ticks(m, now)),
        "bee_rolling" => json!(mobs::bee::rolling(m)),
        "bee_time_since_sting" => json!(mobs::bee::get(m).map_or(0, |bee| bee.time_since_sting)),
        "dancing" => json!(match m.species() {
            Species::Allay => m.extra.flag,
            Species::Parrot => m.extra.party_jukebox.is_some(),
            Species::Piglin => m.extra.scratch[0] > 0,
            _ => false,
        }),
        "charging" => json!(match m.species() {
            Species::Ghast | Species::Vex | Species::Pillager => m.extra.flag,
            _ => false,
        }),
        "screaming" => json!(m.species() == Species::Enderman && m.extra.flag),
        "standing" => json!(m.species() == Species::PolarBear && m.extra.flag),
        "shivering" => json!(mobs::strider::suffocating(m)),
        "dashing" => json!(mobs::camel::is_camel(m.species()) && m.extra.flag2),
        "playing_dead" => json!(mobs::axolotl::is_playing_dead(m)),
        "fish_pattern" => json!(m.extra.variant.min(11)),
        "spell" => json!(m.extra.variant2),
        "aggressive" => json!(m.ai.aggressive),
        "fox_interested" => json!(mobs::fox::flag(m, mobs::fox::INTERESTED)),
        "fox_leaping" => json!(mobs::fox::flag(m, mobs::fox::POUNCING)),
        "fox_faceplanted" => json!(mobs::fox::flag(m, mobs::fox::FACEPLANTED)),
        "fox_defending" => json!(mobs::fox::flag(m, mobs::fox::DEFENDING)),
        "cat_lying" => json!(m.species() == Species::Cat && m.extra.flag),
        "cat_head_up" => json!(m.species() == Species::Cat && m.extra.relax_one),
        "wolf_interested" => json!(m.species() == Species::Wolf && m.extra.flag),
        "wolf_wet" => json!(m.species() == Species::Wolf && m.extra.flag2),
        "guardian_moving" => json!(m.extra.flag2),
        "guardian_laser" => json!(m.extra.flag),
        "guardian_laser_ticks" => json!(m.extra.counter2),
        "turtle_laying" => json!(m.species() == Species::Turtle && m.extra.flag2),
        "turtle_digging" => json!(m.species() == Species::Turtle && m.extra.flag2),
        "turtle_home" => json!(m.species() == Species::Turtle && m.ai.goals.is_running("TurtleGoHomeGoal")),
        "skeleton_conversion" => json!(if m.extra.counter2 > 0 { m.extra.counter2 - 1 } else { -1 }),
        "skeleton_snow_time" => json!(m.extra.counter.max(0)),
        "zombie_drowning" => json!(matches!(m.species(), Species::Zombie | Species::Husk) && m.extra.counter2 > 0),
        "zombie_conversion" => json!(if m.extra.counter2 > 0 { m.extra.counter2 } else { -1 }),
        "zombie_water_time" => json!(m.extra.counter),
        "piglin_conversion" => json!(m.extra.counter2),
        "cure_time" => json!(mobs::zombie_villager::cure(m).map_or(-1, |cure| cure.time)),
        "cure_player" => mobs::zombie_villager::cure(m).and_then(|cure| cure.starter).map_or(Value::Null, |uuid| json!(uuid)),
        "burns_in_day" => json!(m.extra.plugin_flag("burns_in_day").unwrap_or(m.kind.flags.burns_in_day)),
        "arms_raised" => json!(m.ai.aggressive),
        "creeper_fuse" => json!(m.swell),
        "sniffer_state" => json!(mobs::sniffer::state(m)),
        "sniffer_explored" => json!(m.extra.positions),
        "armadillo_state" => json!(mobs::armadillo::state(m)),
        "creaking_active" => json!(mobs::creaking::is_active(m)),
        "golem_weather" => json!(m.extra.variant.min(3)),
        "golem_state" => json!(m.extra.variant3),
        "golem_oxidizing" => json!(m.extra.vector[0] as i64),
        "wither_powered" => json!(mobs::wither::powered(m)),
        "raider_celebrating" => json!(raider.is_some_and(|state| state.celebrating)),
        "raider_raid" => raider.and_then(|state| state.raid_id).map_or(Value::Null, |id| json!(id)),
        "witch_drinking" => json!(m.species() == Species::Witch && m.extra.flag),
        "witch_drink_ticks" => json!(if m.extra.flag { m.extra.using_item } else { 0 }),
        "villager_items" => json!(mobs::villager::ext(m).map(|ext| ext.inventory.iter().map(|item| super::commands::paper_entities::tag_json(&horizon_client::save::item_to_nbt(item))).collect::<Vec<_>>()).unwrap_or_default()),
        "witch_potion" if held().witches.contains_key(&m.id) => json!({ "stack": super::commands::paper_entities::tag_json(&horizon_client::save::item_to_nbt(&held().witches[&m.id])) }),
        "witch_potion" => m.extra.main_hand.as_ref().filter(|_| m.extra.flag).map_or(Value::Null, |item| json!(&**item)),
        "love" => json!(m.extra.in_love > 0),
        "bat_target" | "fish_leader" | "fish_followers" | "fish_max" => match key {
            "bat_target" => spot(m.extra.positions.first().copied()),
            "fish_leader" => m.extra.leader.map_or(Value::Null, |id| json!(id)),
            "fish_followers" => json!(m.extra.followers),
            _ => json!(if m.species() == Species::Salmon { 5 } else { 8 }),
        },
        "dolphin_treasure" => spot(m.extra.home),
        "boost_ticks" => json!(m.extra.counter2),
        "boost_current" => json!(m.extra.counter),
        "shulker_peek" => json!(m.extra.counter),
        "is_mob" => json!(true),
        "caravan_head" => json!(mobs::caravan::head_of(m)),
        "sheared" => json!(m.extra.sheared),
        "vex_owner" => json!(m.extra.counter2),
        "vex_life" => json!(m.extra.egg_time),
        "trap" => json!(m.extra.trap),
        "hidden_parts" => json!(m.extra.hidden_model_parts),
        "frog_tongue" => m.extra.frog.tongue_target.map_or(Value::Null, |id| json!(id)),
        _ => return None,
    })
}

/// What a plugin sets on a mob: `None` for a key that's no fact of mobs.
fn write(m: &mut Mob, key: &str, value: &Value, now: i64) -> Option<bool> {
    use mobs::panda::{FLAG_ON_BACK, FLAG_ROLL, FLAG_SNEEZE};
    let on = truth(value);
    let number = int(value);
    match key {
        "panda_sneezing" => {
            set_panda_flag(m, FLAG_SNEEZE, on);
            if !on {
                panda(m).sneeze = 0;
            }
        }
        "panda_rolling" => set_panda_flag(m, FLAG_ROLL, on),
        "hidden_parts" => m.extra.hidden_model_parts = number as u8,
        "panda_sitting" => set_panda_flag(m, mobs::panda::FLAG_SIT, on),
        "panda_on_back" => set_panda_flag(m, FLAG_ON_BACK, on),
        "panda_eating" => panda(m).eat = i32::from(on),
        "panda_unhappy" => panda(m).unhappy = number,
        "panda_sneeze_ticks" => panda(m).sneeze = number,
        "panda_eat_ticks" => panda(m).eat = number,
        "horse_grazing" => m.extra.flag = on && mobs::equine::grazes(m.species()),
        "horse_rearing" => {
            if on {
                mobs::equine::stand_if_possible(m);
            } else {
                m.extra.counter = 0;
            }
        }
        "bee_anger" | "anger" => set_anger_ticks(m, now, number),
        "bee_time_since_sting" => {
            if let Some(bee) = bee(m) {
                bee.time_since_sting = number;
            }
        }
        "dancing" => match m.species() {
            Species::Piglin => {
                m.extra.scratch[0] = i32::from(on);
                m.extra.scratch[1] = if on { number.max(300) } else { 0 };
            }
            _ => return Some(false),
        },
        "charging" => match m.species() {
            Species::Ghast | Species::Vex | Species::Pillager => m.extra.flag = on,
            _ => return Some(false),
        },
        "screaming" => m.extra.flag = on && m.species() == Species::Enderman,
        "standing" => mobs::polar_bear::set_standing(m, on),
        "shivering" => m.extra.flag = on && m.species() == Species::Strider,
        "dashing" => {
            if !mobs::camel::is_camel(m.species()) {
                return Some(false);
            }
            mobs::camel::set_dashing(m, on);
        }
        "playing_dead" => m.extra.axolotl.playing_dead = on,
        "fish_pattern" => m.extra.variant = number.clamp(0, 11) as u8,
        "spell" => m.extra.variant2 = number.clamp(0, 5) as u8,
        "aggressive" | "arms_raised" => m.ai.aggressive = on,
        "fox_interested" => fox_flag(m, mobs::fox::INTERESTED, on),
        "fox_leaping" => fox_flag(m, mobs::fox::POUNCING, on),
        "fox_faceplanted" => fox_flag(m, mobs::fox::FACEPLANTED, on),
        "fox_defending" => fox_flag(m, mobs::fox::DEFENDING, on),
        "cat_lying" => m.extra.flag = on && m.species() == Species::Cat,
        "cat_head_up" => m.extra.relax_one = on && m.species() == Species::Cat,
        "wolf_interested" => m.extra.flag = on && m.species() == Species::Wolf,
        "guardian_laser_ticks" => m.extra.counter2 = number,
        "skeleton_conversion" => m.extra.counter2 = if number < 0 { 0 } else { number + 1 },
        "zombie_conversion" => {
            m.extra.counter2 = number.max(0);
            if number < 0 {
                m.extra.counter = -1;
            }
        }
        "zombie_water_time" => m.extra.counter = number,
        "piglin_conversion" => m.extra.counter2 = number.max(0),
        "cure_time" => {
            if number < 0 {
                mobs::villager::ext_mut(m).cure = None;
            } else {
                let starter = mobs::zombie_villager::cure(m).and_then(|cure| cure.starter);
                mobs::villager::ext_mut(m).cure = Some(mobs::zombie_villager::Cure { time: number, starter });
            }
        }
        "cure_player" => {
            let starter = value.as_array().filter(|words| words.len() == 4).map(|words| [int(&words[0]), int(&words[1]), int(&words[2]), int(&words[3])]);
            let time = mobs::zombie_villager::cure(m).map_or(0, |cure| cure.time);
            mobs::villager::ext_mut(m).cure = Some(mobs::zombie_villager::Cure { time, starter });
        }
        "burns_in_day" => {
            let mut kind = (*m.kind).clone();
            kind.flags.burns_in_day = on;
            m.kind = Arc::new(kind);
        }
        "creeper_fuse" => m.swell = number.clamp(0, 30),
        "sniffer_state" => mobs::sniffer::set_state(m, number.clamp(0, 6) as u8),
        "sniffer_explored" => m.extra.positions = value.as_array().into_iter().flatten().filter_map(block).collect(),
        "golem_weather" => m.extra.variant = number.clamp(0, 3) as u8,
        "golem_state" => m.extra.variant3 = number.clamp(0, 4) as u8,
        "golem_oxidizing" => m.extra.vector[0] = value.as_i64().unwrap_or(-1) as f64,
        "raider_celebrating" => {
            if !mobs::raider::is_raider(m.species()) {
                return Some(false);
            }
            mobs::raider::state_mut(m).celebrating = on;
        }
        "witch_drink_ticks" => m.extra.using_item = number,
        "love" => m.extra.in_love = number,
        "bat_target" => {
            m.extra.positions.clear();
            m.extra.positions.extend(block(value));
        }
        "dolphin_treasure" => m.extra.home = block(value),
        "boost_ticks" => {
            m.extra.counter2 = number;
            m.extra.counter = 0;
        }
        "boost_current" => m.extra.counter = number,
        "shulker_peek" => m.extra.counter = number.clamp(0, 100),
        "sheared" => m.extra.sheared = on,
        "vex_owner" => m.extra.counter2 = number,
        "vex_life" => m.extra.egg_time = number,
        "trap" => m.extra.trap = on,
        "frog_tongue" => m.extra.frog.tongue_target = value.as_u64().map(|id| id as u32),
        "caravan_head" => m.extra.ints[1] = number,
        "saddle" => {
            let saddle = on.then(|| horizon_client::items::ItemStack::new("minecraft:saddle", 1));
            m.set_saddle(saddle);
        }
        _ => return None,
    }
    m.needs_sync = true;
    Some(true)
}

fn mob_action(game: &mut Game<'_>, index: usize, action: &str, query: &Value) -> Option<Value> {
    let now = game.world.game_time;
    Some(match action {
        "enderman_teleport" => json!(with_world(game, index, |m, cx| mobs::enderman::teleport_random(m, cx.w, cx.events))),
        "enderman_teleport_to" => {
            let at = vector(&query["at"])?;
            json!(with_world(game, index, |m, cx| {
                let d = Vec3::new(m.position.x - at[0], m.position.y - at[1], m.position.z - at[2]);
                let (x, y, z) = (m.random.next_double(), m.random.next_double(), m.random.next_double());
                let length = d.length().max(1.0e-4);
                let (x, y, z) = (m.position.x + (x - 0.5) * 8.0 - d.x / length * 16.0, m.position.y + (y - 0.5) * 8.0 - d.y / length * 16.0, m.position.z + (z - 0.5) * 8.0 - d.z / length * 16.0);
                mobs::enderman::teleport_to(m, cx.w, x, y, z, cx.events)
            }))
        }
        "path_to" => {
            let at = vector(&query["at"])?;
            let speed = query["speed"].as_f64().unwrap_or(1.0);
            json!(with_world(game, index, |m, cx| m.nav_move_to(cx.w, at, speed)))
        }
        "path_find" => {
            let at = block(&query["at"])?;
            with_world(game, index, |m, cx| m.create_path_to(cx.w, at, 0).map_or(Value::Null, |path| path_json(&path)))
        }
        "path_stop" => {
            game.mobs[index].nav_stop();
            json!(true)
        }
        "path" => game.mobs[index].ai.nav.path.as_ref().map_or(Value::Null, path_json),
        "path_flags" => {
            let nav = &game.mobs[index].ai.nav;
            json!({ "float": nav.can_float, "open_doors": nav.can_open_doors, "pass_doors": nav.can_pass_doors })
        }
        "path_set" => {
            let nav = &mut game.mobs[index].ai.nav;
            match query["what"].as_str()? {
                "float" => nav.can_float = truth(&query["value"]),
                "open_doors" => nav.can_open_doors = truth(&query["value"]),
                "pass_doors" => nav.can_pass_doors = truth(&query["value"]),
                _ => return None,
            }
            json!(true)
        }
        "goals" => {
            let m = &game.mobs[index];
            json!({ "running": m.ai.goals.running_names(), "targets": m.ai.targets.running_names() })
        }
        "goals_all" => {
            let m = &game.mobs[index];
            let row = |(name, flags, running, priority): (String, u8, bool, i32)| json!({ "name": name, "flags": flags, "running": running, "priority": priority });
            json!(m.ai.goals.listed().into_iter().chain(m.ai.targets.listed()).map(row).collect::<Vec<_>>())
        }
        "goal_add" => {
            let name = query["goal"].as_str()?.to_owned();
            let flags = query["flags"].as_u64()? as u8;
            let priority = int(&query["priority"]);
            let m = &mut game.mobs[index];
            let goal = mobs::goals::PluginGoal::new(name, flags);
            // Paper's split: a goal that targets goes in the target selector.
            if flags & mobs::goals::TARGET != 0 {
                m.ai.targets.add(priority, goal);
            } else {
                m.ai.goals.add(priority, goal);
            }
            json!(true)
        }
        "goal_wants" => {
            let name = query["goal"].as_str()?;
            let wants = truth(&query["value"]);
            let m = &game.mobs[index];
            json!(m.ai.goals.set_wanted(name, wants) || m.ai.targets.set_wanted(name, wants))
        }
        "goal_remove" => {
            let name = query["goal"].as_str().map(str::to_owned);
            let flags = query["flags"].as_u64().map(|flags| flags as u8);
            json!(with_world(game, index, |m, cx| {
                let wanted = |goal: &dyn mobs::goals::Goal| {
                    name.as_ref().is_none_or(|name| goal.plugin_key().unwrap_or(goal.name()) == name)
                        && flags.is_none_or(|flags| if flags == 0 { goal.flags() == 0 } else { goal.flags() & flags != 0 })
                };
                let (mut goals, mut targets) = (std::mem::take(&mut m.ai.goals), std::mem::take(&mut m.ai.targets));
                let found = goals.remove_where(m, cx, wanted);
                let found = targets.remove_where(m, cx, wanted) || found;
                m.ai.goals = goals;
                m.ai.targets = targets;
                found
            }))
        }
        "goal_call" => {
            let name = query["goal"].as_str()?.to_owned();
            let call = query["call"].as_str()?.to_owned();
            json!(with_world(game, index, |m, cx| {
                let (mut goals, mut targets) = (std::mem::take(&mut m.ai.goals), std::mem::take(&mut m.ai.targets));
                let answer = goals.ask(&name, &call, m, cx).or_else(|| targets.ask(&name, &call, m, cx));
                m.ai.goals = goals;
                m.ai.targets = targets;
                answer.unwrap_or(false)
            }))
        }
        "goals_clear" => {
            let m = &mut game.mobs[index];
            m.ai.goals.clear();
            m.ai.targets.clear();
            json!(true)
        }
        "explode" => {
            let m = &mut game.mobs[index];
            if m.species() != Species::Creeper {
                return Some(json!(false));
            }
            m.extra.flag = true;
            m.swell = 29;
            json!(true)
        }
        "ignite" => {
            let m = &mut game.mobs[index];
            if m.species() != Species::Creeper {
                return Some(json!(false));
            }
            m.extra.flag = true;
            json!(true)
        }
        "wither_invulnerable" => {
            mobs::wither::make_invulnerable(&mut game.mobs[index]);
            json!(true)
        }
        "armadillo_roll" => {
            let m = &mut game.mobs[index];
            mobs::armadillo::set_state(m, if truth(&query["value"]) { mobs::armadillo::ROLLING } else { mobs::armadillo::UNROLLING });
            json!(true)
        }
        "creaking_activate" => {
            let m = &mut game.mobs[index];
            m.extra.flag = true;
            m.ai.target = who(query);
            json!(true)
        }
        "creaking_deactivate" => {
            let m = &mut game.mobs[index];
            m.extra.flag = false;
            m.ai.target = None;
            json!(true)
        }
        "sulfur_ignite" => json!(mobs::sulfur_cube::prime(&mut game.mobs[index], truth(&query["value"]))),
        "sulfur_fuse" => json!(mobs::sulfur_cube::fuse(&game.mobs[index])),
        "sulfur_can_explode" => json!(mobs::sulfur_cube::archetype(&game.mobs[index]).is_some()),
        "warden_get" => {
            let m = &game.mobs[index];
            let state = m.extra.warden.as_ref()?;
            match who(query) {
                Some(who) => json!(state.anger.anger_of(who)),
                None => json!(state.anger.active_anger(state.target())),
            }
        }
        "warden_highest" => {
            let state = game.mobs[index].extra.warden.as_ref()?;
            json!(state.anger.active_anger(None))
        }
        "warden_level" => {
            let m = &game.mobs[index];
            let anger = m.extra.warden.as_ref().map_or(0, |state| state.anger.active_anger(state.target()));
            json!(anger)
        }
        "warden_target" => {
            let m = &game.mobs[index];
            let target = m.extra.warden.as_ref()?.anger.suspects().first().copied();
            who_named(game, Some(m.id), Some(m.position), target)
        }
        "warden_increase" => {
            let target = who(query)?;
            let amount = int(&query["value"]);
            let state = game.mobs[index].extra.warden.as_mut()?;
            json!(state.anger.increase(target, amount))
        }
        "warden_set" => {
            let target = who(query)?;
            let amount = int(&query["value"]);
            let state = game.mobs[index].extra.warden.as_mut()?;
            state.anger.clear(target);
            json!(state.anger.increase(target, amount))
        }
        "warden_clear" => {
            let target = who(query)?;
            game.mobs[index].extra.warden.as_mut()?.anger.clear(target);
            json!(true)
        }
        "villager_sleep" => {
            let at = block(&query["at"])?;
            json!(with_world(game, index, |m, cx| mobs::villager::start_sleeping(m, at, cx.w, cx.events)))
        }
        "villager_wake" => {
            let id = game.mobs[index].id;
            game.wake_villager(id);
            json!(true)
        }
        "villager_shake" => {
            let m = &mut game.mobs[index];
            let mut events = Vec::new();
            mobs::villager::set_unhappy(m, &mut events);
            if let Err(error) = game.handle_mob_events(events) {
                log::error!("a plugin's villager: {error}");
            }
            json!(true)
        }
        "reputation" => {
            let words = query["uuid"].as_array().map(|words| [int(&words[0]), int(&words[1]), int(&words[2]), int(&words[3])])?;
            let ext = mobs::villager::ext(&game.mobs[index]);
            let rows: Vec<Value> = ext.map_or(Vec::new(), |ext| ext.gossips.entries().filter(|(target, _, _)| *target == words).map(|(_, kind, value)| json!([kind.id(), value])).collect());
            json!(rows)
        }
        "reputations" => {
            let ext = mobs::villager::ext(&game.mobs[index]);
            let rows: Vec<Value> = ext.map_or(Vec::new(), |ext| ext.gossips.entries().map(|(target, kind, value)| json!([target, kind.id(), value])).collect());
            json!(rows)
        }
        "reputation_set" => {
            use horizon_client::gossip::GossipType;
            let words = query["uuid"].as_array().map(|words| [int(&words[0]), int(&words[1]), int(&words[2]), int(&words[3])])?;
            let kind = GossipType::from_id(query["kind"].as_str()?)?;
            let ext = mobs::villager::ext_mut(&mut game.mobs[index]);
            ext.gossips.remove(words, kind);
            let value = int(&query["value"]);
            if value != 0 {
                ext.gossips.add(words, kind, value);
            }
            json!(true)
        }
        "reputations_clear" => {
            mobs::villager::ext_mut(&mut game.mobs[index]).gossips.clear();
            json!(true)
        }
        "trader_restock" => {
            let trader = game.mobs[index].extra.trade.as_deref_mut()?;
            trader.restock(now);
            json!(true)
        }
        "trader_demand" => {
            let trader = game.mobs[index].extra.trade.as_deref_mut()?;
            for offer in &mut trader.offers {
                offer.update_demand();
            }
            json!(true)
        }
        "trader_reset" => {
            let trader = game.mobs[index].extra.trade.as_deref_mut()?;
            trader.offers.clear();
            trader.unset = true;
            json!(true)
        }
        "allay_jukebox" => {
            let m = &game.mobs[index];
            let at = m.position;
            let near = |k: &(i32, i32, i32)| (f64::from(k.0) + 0.5 - at.x).powi(2) + (f64::from(k.1) + 0.5 - at.y).powi(2) + (f64::from(k.2) + 0.5 - at.z).powi(2) <= 100.0;
            game.world
                .block_entities
                .iter()
                .find(|(key, entity)| matches!(entity, horizon_client::containers::BlockEntity::Jukebox(jukebox) if jukebox.playing) && near(key))
                .map_or(Value::Null, |(key, _)| json!([key.0, key.1, key.2]))
        }
        "allay_duplicate" => {
            let at = {
                let m = &mut game.mobs[index];
                m.extra.egg_time = 6000;
                [m.position.x, m.position.y, m.position.z]
            };
            match game.spawn_mob("minecraft:allay", at, true) {
                Some(id) => {
                    if let Some(copy) = game.mobs.iter_mut().find(|m| m.id == id) {
                        copy.extra.egg_time = 6000;
                    }
                    json!(id)
                }
                None => Value::Null,
            }
        }
        "guardian_laser" => {
            let m = &mut game.mobs[index];
            let on = truth(&query["value"]);
            if on && m.ai.target.is_none() {
                return Some(json!(false));
            }
            m.extra.flag = on;
            m.extra.counter2 = 0;
            json!(true)
        }
        "witch_drink" => {
            let m = &mut game.mobs[index];
            match query["value"].as_str() {
                Some(kind) => {
                    let id = if kind.contains(':') { kind.to_owned() } else { format!("minecraft:{kind}") };
                    let mob = m.id;
                    match stack(&query["stack"]) {
                        Some(stack) => held().witches.insert(mob, stack),
                        None => held().witches.remove(&mob),
                    };
                    m.extra.set_main_hand(Some(id.into()));
                    m.extra.using_item = 32;
                    m.extra.flag = true;
                }
                None => {
                    m.extra.set_main_hand(None);
                    m.extra.using_item = 0;
                    m.extra.flag = false;
                }
            }
            json!(true)
        }
        "warden_disturbance" => {
            let at = block(&query["at"])?;
            with_world(game, index, |m, cx| mobs::warden_brain::set_disturbance(m, cx.w, at));
            json!(true)
        }
        "caravan_tail" => {
            let id = game.mobs[index].id;
            game.mobs.iter().find(|m| mobs::caravan::head_of(m) == id + 1).map_or(Value::Null, |m| json!(m.id))
        }
        "ranged_attack" => {
            let target = who(query)?;
            let power = query["value"].as_f64().unwrap_or(1.0) as f32;
            // `RangedAttackMob.performRangedAttack`: each kind's own shot.
            let at = vector(&query["at"]);
            with_world(game, index, |m, cx| match m.species() {
                Species::SnowGolem => mobs::golem::snowball_attack(m, target, power, cx),
                Species::Llama | Species::TraderLlama => mobs::llama::spit(m, target, power, cx),
                Species::Drowned => mobs::undead::drowned_throw(m, target, power, cx),
                Species::Witch => mobs::witch::witch_attack(m, target, power, cx),
                Species::Pillager | Species::Piglin => mobs::illager::shoot_crossbow(m, target, cx),
                Species::Wither => {
                    if let Some(at) = at {
                        mobs::wither::shoot(m, 0, at, false, cx.events);
                    }
                }
                _ => mobs::undead::perform_ranged_attack(m, target, cx, power),
            });
            json!(true)
        }
        "sulfur_fuse_set" => {
            game.mobs[index].extra.counter2 = int(&query["value"]) + 1;
            json!(true)
        }
        "sulfur_swallowable" => json!(mobs::sulfur_cube::swallowable(&game.mobs[index], query["item"].as_str()?)),
        "fish_follow" => {
            let leader = query["leader"].as_u64().map(|id| id as u32);
            game.mobs[index].extra.leader = leader;
            json!(true)
        }
        _ => return None,
    })
}

fn path_json(path: &mobs::path::Path) -> Value {
    json!({ "points": path.nodes.iter().map(|node| [node.x, node.y, node.z]).collect::<Vec<_>>(), "next": path.next, "target": path.target, "reached": path.reached })
}

/// `{ "op": "entity_merge_kept" }`: `/data merge` refuses what isn't a mob, item, stand or frame; a plugin may
/// set any of them. The rest of a saved compound is kept in `persist.raw`, as for those.
fn merge_kept(game: &mut Game<'_>, query: &Value) -> bool {
    let id = id_of(query);
    let Some(Ok((patch, _))) = query["snbt"].as_str().map(parse_snbt) else {
        return false;
    };
    game.native_in_level(&dimension(query), |game| {
        let Some(target) = i32::try_from(id).ok().and_then(|id| game.level_entity(id)) else {
            return kept_merge(game, id, &patch);
        };
        let Some(old) = game.entity_nbt(target) else {
            return false;
        };
        let new = merged(&old, &patch);
        let number = |key: &str| match new.get(key) {
            Some(Tag::Byte(v)) => Some(i64::from(*v)),
            Some(Tag::Short(v)) => Some(i64::from(*v)),
            Some(Tag::Int(v)) => Some(i64::from(*v)),
            Some(Tag::Long(v)) => Some(*v),
            _ => None,
        };
        match target {
            Target::Other(id) => {
                if let Some(other) = game.others.iter_mut().find(|other| other.id == id) {
                    if let Some(at) = horizon_client::entity_nbt::position(&new) {
                        other.position = at;
                    }
                    [other.yaw, other.pitch] = horizon_client::entity_nbt::rotation(&new);
                }
                game.persist.raw.insert(id, new);
                true
            }
            Target::Boat(id) => {
                let Some((position, velocity, yaw)) = horizon_client::entity_nbt::boat_from_nbt(&new) else {
                    return false;
                };
                if let Some(boat) = game.boats.iter_mut().find(|boat| boat.id == id) {
                    boat.position = position;
                    boat.velocity = velocity;
                    boat.yaw = yaw;
                }
                game.persist.raw.insert(id, new);
                true
            }
            Target::Cart(id) => {
                let Some(kind) = game.carts.iter().find(|cart| cart.id == id).map(|cart| cart.kind) else {
                    return false;
                };
                let Some(fresh) = super::minecarts::Minecart::from_nbt(&new, id, kind) else {
                    return false;
                };
                if let Some(cart) = game.carts.iter_mut().find(|cart| cart.id == id) {
                    let (previous, previous_yaw, ticks) = (cart.position, cart.yaw, cart.tick_count);
                    *cart = fresh;
                    cart.previous = previous;
                    cart.previous_yaw = previous_yaw;
                    cart.tick_count = ticks;
                }
                game.persist.raw.insert(id, new);
                true
            }
            Target::Orb(id) => {
                let Some(orb) = game.orbs.iter_mut().find(|orb| orb.id == id) else {
                    return false;
                };
                if let Some(value) = number("Value") {
                    orb.value = value.max(0) as u32;
                }
                if let Some(count) = number("Count") {
                    orb.count = count.max(1) as u32;
                }
                if let Some(age) = number("Age") {
                    orb.age = age.max(0) as u32;
                }
                true
            }
            Target::Projectile(id) => {
                let Some(p) = game.creatures.projectiles.iter_mut().find(|p| p.id == id) else {
                    return false;
                };
                p.velocity = horizon_client::entity_nbt::motion(&new);
                if let Some(a) = p.arrow.as_mut() {
                    if let Some(value) = number("PierceLevel") {
                        a.pierce = value as u8;
                    }
                    if let Some(value) = number("life") {
                        a.life = value as i32;
                    }
                    if let Some(value) = number("pickup") {
                        a.pickup = horizon_client::arrows::Pickup::from_byte(value);
                    }
                    if let Some(item) = new.get("item").and_then(horizon_client::save::item_from_nbt) {
                        if let horizon_client::projectiles::ProjectileKind::Arrow { effect, .. } = &mut p.kind {
                            *effect = super::dispenser::tipped_effect(&item);
                        }
                        a.pickup_item = item;
                    }
                    a.fired_from_weapon = new.get("weapon").and_then(horizon_client::save::item_from_nbt);
                    if let Some(Tag::Byte(value)) = new.get("DealtDamage") {
                        a.dealt_damage = *value != 0;
                    }
                }
                if let horizon_client::projectiles::ProjectileKind::Arrow { base_damage, critical, .. } = &mut p.kind {
                    if let Some(Tag::Double(bits)) = new.get("damage") {
                        *base_damage = f64::from_bits(*bits);
                    }
                    if let Some(value) = number("crit") {
                        *critical = value != 0;
                    }
                }
                p.needs_sync = true;
                true
            }
            Target::Painting(id) => {
                let Some((pos, facing, variant)) = horizon_client::entity_nbt::painting_from_nbt(&new) else {
                    return false;
                };
                let kind = game.painting_variants.get(&variant).cloned();
                let Some(painting) = game.paintings.iter_mut().find(|painting| painting.id == id) else {
                    return false;
                };
                painting.pos = pos;
                painting.facing = facing;
                if let Some(kind) = kind {
                    painting.variant = kind.id;
                    painting.width = kind.width;
                    painting.height = kind.height;
                }
                game.persist.raw.insert(id, new);
                true
            }
            Target::Crystal(id) => {
                let Some(mut fresh) = super::dragon::Crystal::from_nbt(&new, 0) else {
                    return false;
                };
                let Some(crystal) = game.crystals.iter_mut().find(|crystal| crystal.id == id) else {
                    return false;
                };
                fresh.id = id;
                fresh.time = crystal.time;
                *crystal = fresh;
                true
            }
            Target::Dragon(id) => {
                let seed = game.world.random.next_long();
                let Some(mut fresh) = super::dragon::Dragon::from_nbt(&new, seed) else {
                    return false;
                };
                fresh.id = id;
                game.dragon = Some(fresh);
                true
            }
            _ => false,
        }
    })
    .unwrap_or(false)
}

/// `None` for an op that isn't this family's.
/// A station a plugin opened during a player's turn, now it's over.
pub(in crate::game) fn open_station_now(game: &mut Game<'_>, request: &Value) {
    let _ = player_send(game, "open_station", request);
}

pub(in crate::game) fn query(game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    if !op.starts_with("entity_") {
        return None;
    }
    let now = game.world.game_time;
    let key = query["key"].as_str().unwrap_or_default();
    Some(match op {
        "entity_get" if key == "swimming" && held().swimming.contains(&id_of(query)) => json!(true),
        "entity_get" if key == "leash_holder" => leash_holder(game, query).unwrap_or(Value::Null),
        "entity_get" if key == "leashed" => json!(leash_holder(game, query).is_some()),
        "entity_act" if key == "leash_to" || key == "leash_off" => json!(leash_set(game, key, query).unwrap_or(false)),
        "entity_get" if key == "killer" => in_mob_level(game, query, |game, index| killer(game, index)).flatten().map_or(Value::Null, |uuid| json!(uuid)),
        // `getLastDamageCause()` with who did it: Bukkit's is an EntityDamageByEntityEvent when something hit it, whose
        // `getDamager()` CoreProtect logs kills by (a mob as `{id}`, a player as `{uuid}`, from the same record `killer` reads).
        "entity_get" if key == "last_damage" => in_mob_level(game, query, |game, index| {
            let (damage, _) = game.mobs[index].last_damage?;
            let attacker = match damage.attacker {
                Some(Who::Mob(id)) => json!({ "id": id }),
                Some(Who::Player) => killer(game, index).map_or(Value::Null, |uuid| json!({ "uuid": uuid })),
                None => Value::Null,
            };
            Some(json!({ "kind": format!("{:?}", damage.kind), "amount": game.mobs[index].last_hurt, "attacker": attacker }))
        })
        .flatten()
        .unwrap_or(Value::Null),
        "entity_get" => in_mob_level(game, query, |game, index| read(&game.mobs[index], key, now).or_else(|| read_more(&game.mobs[index], key))).flatten().unwrap_or(Value::Null),
        "entity_set" => json!(in_mob_level(game, query, |game, index| write(&mut game.mobs[index], key, &query["value"], now).or_else(|| write_more(game, index, key, &query["value"]))).flatten().unwrap_or(false)),
        "entity_act" => in_mob_level(game, query, |game, index| mob_action(game, index, key, query).or_else(|| mob_action_more(game, index, key, query))).flatten().unwrap_or(Value::Null),
        "entity_common" => common(game, key, query).or_else(|| born_ticks(game, key, query)).unwrap_or(Value::Null),
        "entity_unseen" => json!(unseen(game, query)),
        "entity_merge_kept" => json!(merge_kept(game, query)),
        "entity_kept" => kept(game, query).unwrap_or(Value::Null),
        "entity_births" => {
            for row in query["rows"].as_array().map_or(&[][..], Vec::as_slice) {
                let (Some(id), Some(at)) = (row[0].as_u64(), row[2].as_array()) else {
                    continue;
                };
                let mut origin = at.clone();
                origin.push(row[1].clone());
                let Tag::Compound(saved) = game.persist.raw.entry(id as u32).or_insert_with(|| Tag::Compound(Default::default())) else {
                    continue;
                };
                if let Tag::Compound(values) = saved.entry(PLUGIN.into()).or_insert_with(|| Tag::Compound(Default::default())) {
                    if let Some(tag) = plugin_tag(&json!(origin)) {
                        values.insert("origin".into(), tag);
                    }
                    if !values.contains_key(&"spawn_reason".into())
                        && let Some(tag) = plugin_tag(&row[3])
                    {
                        values.insert("spawn_reason".into(), tag);
                    }
                }
            }
            json!(true)
        }
        "entity_hold" => hold(game, key, query).unwrap_or(Value::Null),
        "entity_kept_list" => kept_list(game, query),
        "entity_poi" => poi(game, query),
        "entity_boss_bar" => boss_bar(game, query).unwrap_or(Value::Null),
        "entity_memory" => json!(memory_set(game, query).unwrap_or(false)),
        "entity_default_attributes" => default_attributes(game, query),
        "entity_plugin_set" => json!(plugin_set(game, key, query).unwrap_or(false)),
        "entity_plugin_get" => game.native_in_level(&dimension(query), |game| game.persist_raw(id_of(query)).and_then(|tag| tag.get(PLUGIN)).and_then(|tag| tag.get(key)).map(plugin_json)).flatten().unwrap_or(Value::Null),
        _ => return player_query(game, op, query),
    })
}

/// Who holds a mob's or a boat's lead: a player by name, a fence knot by its entity.
fn leash_holder(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    use mobs::leash::Holder;
    let id = id_of(query);
    game.native_in_level(&dimension(query), |game| {
        let (holder, player) = match game.mobs.iter().find(|m| m.id == id) {
            Some(m) => (m.extra.leash.clone(), m.extra.leash_player),
            None => game.boats.iter().find(|boat| boat.id == id).map(|boat| (boat.leash.clone(), boat.leash_player))?,
        };
        Some(match holder? {
            Holder::Player => match player {
                Some(words) => json!({ "player": true, "uuid": words }),
                None => who_named(game, Some(id), None, Some(Who::Player)),
            },
            Holder::Mob(id) | Holder::Boat(id) => json!({ "entity": id }),
            Holder::Knot(at) => json!({ "knot": at, "entity": game.knot_ids.get(&at) }),
        })
    })
    .flatten()
}

/// `Leashable.setLeashHolder`: a player, a mob, a boat or a fence knot, or nobody.
fn leash_set(game: &mut Game<'_>, key: &str, query: &Value) -> Option<bool> {
    use mobs::leash::Holder;
    let id = id_of(query);
    let words = query["uuid"].as_array().filter(|words| words.len() == 4).map(|words| [int(&words[0]), int(&words[1]), int(&words[2]), int(&words[3])]);
    game.native_in_level(&dimension(query), |game| {
        let holder = match (key, who(query)) {
            ("leash_off", _) => None,
            (_, Some(Who::Player)) => Some(Holder::Player),
            (_, Some(Who::Mob(other))) => Some(match game.knot_ids.iter().find(|(_, knot)| **knot == other) {
                Some((at, _)) => Holder::Knot(*at),
                None if game.boats.iter().any(|boat| boat.id == other) => Holder::Boat(other),
                None => Holder::Mob(other),
            }),
            _ => return None,
        };
        if let Some(index) = game.mobs.iter().position(|m| m.id == id) {
            match holder {
                Some(holder) => {
                    let m = &mut game.mobs[index];
                    m.extra.leash = Some(holder);
                    m.extra.leash_player = words;
                }
                None => game.break_leash(id, false, false),
            }
            return Some(true);
        }
        let index = game.boats.iter().position(|boat| boat.id == id)?;
        let mut boat = game.boats.remove(index);
        match holder {
            Some(holder) => {
                boat.leash = Some(holder);
                boat.leash_player = words;
            }
            None => game.release_boat_leash(&mut boat, false),
        }
        game.boats.insert(index, boat);
        Some(true)
    })
    .flatten()
}

/// What isn't a mob: carts, boats and projectiles, which no `/data` writes.
fn object(game: &mut Game<'_>, key: &str, query: &Value) -> Option<Value> {
    use horizon_client::projectiles::ProjectileKind as Kind;
    let id = id_of(query);
    let value = &query["value"];
    game.native_in_level(&dimension(query), |game| {
        Some(match key {
            "cart" => {
                let cart = game.carts.iter().find(|cart| cart.id == id)?;
                json!({ "damage": cart.damage, "fuse": cart.fuse })
            }
            "cart_damage" => {
                game.carts.iter_mut().find(|cart| cart.id == id)?.damage = value.as_f64()? as f32;
                json!(true)
            }
            "cart_ignite" => {
                let cart = game.carts.iter_mut().find(|cart| cart.id == id)?;
                if cart.fuse < 0 {
                    cart.fuse = 80;
                }
                json!(true)
            }
            "cart_explode" => {
                game.carts.iter_mut().find(|cart| cart.id == id)?.fuse = 0;
                json!(true)
            }
            "bolt" => {
                let bolt = game.bolts.iter().find(|bolt| bolt.id == id)?;
                let conn = bolt.cause.flatten();
                let cause = conn.and_then(|conn| game.lan.as_ref()?.hub.guest(conn)).map(|guest| format!("{:032x}", guest.uuid));
                json!({ "cause": cause })
            }
            "bolt_cause" => {
                let uuid = query["uuid"].as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
                let conn = uuid.and_then(|uuid| game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == uuid).map(|guest| guest.conn));
                game.bolts.iter_mut().find(|bolt| bolt.id == id)?.cause = conn.map(Some);
                json!(true)
            }
            "rocket" => {
                let rocket = game.rockets.iter().find(|rocket| rocket.id == id)?;
                let owner = rocket.guest.and_then(|conn| game.lan.as_ref()?.hub.guest(conn)).map(|guest| format!("{:032x}", guest.uuid));
                let mob = match rocket.owner {
                    Some(Who::Mob(id)) => Some(id),
                    _ => None,
                };
                json!({ "attached_to": rocket.attached_to, "owner": owner, "owner_entity": mob })
            }
            "rocket_detonate" => {
                let index = game.rockets.iter().position(|rocket| rocket.id == id)?;
                let old = &game.rockets[index];
                let fresh = horizon_client::entity_nbt::RocketNbt { position: old.position, motion: old.velocity, life: 1, lifetime: 0, item: Some(old.item.clone()), angled: old.angled, owner: old.owner, left_owner: true };
                let mut fresh = super::fireworks::Rocket::from_nbt(fresh);
                fresh.id = id;
                fresh.previous = old.previous;
                fresh.attached = old.attached;
                fresh.guest = old.guest;
                fresh.attached_to = old.attached_to;
                game.rockets[index] = fresh;
                json!(true)
            }
            "rocket_attach" => {
                let rocket = game.rockets.iter_mut().find(|rocket| rocket.id == id)?;
                rocket.attached_to = query["entity"].as_u64().map(|id| id as u32);
                json!(true)
            }
            "falling" => {
                let falling = game.falling.iter().find(|falling| falling.id == id)?;
                let start = falling.start_pos;
                let block = game.pack.assets.registry.state(falling.state).map(|state| state.name.to_string());
                json!({ "block": block, "start": [start.x, start.y, start.z] })
            }
            "tnt" => {
                let entry = held().tnt.get(&id).copied();
                let saved = || {
                    let Some(Tag::IntArray(owner)) = game.persist_raw(id)?.get("owner") else {
                        return None;
                    };
                    let words: [i32; 4] = owner.as_slice().try_into().ok()?;
                    Some(Source::Player(horizon_client::lan::playerdata::ints_uuid(words)).json())
                };
                let source = entry.and_then(|entry| entry.1).map(Source::json).or_else(saved);
                json!({ "from": entry.map(|entry| entry.0), "source": source })
            }
            "tnt_source" => {
                // Only what lives can own it, as on Paper: anything else clears it.
                let source = Source::read(query).filter(|source| match source {
                    Source::Mob(id) => game.mobs.iter().any(|m| m.id == *id),
                    Source::Player(_) => true,
                });
                let at = game.tnts.iter().find(|tnt| tnt.id == id)?.position;
                let mut held = held();
                let entry = held.tnt.entry(id).or_insert(([at.x, at.y, at.z], None));
                entry.1 = source;
                json!(true)
            }
            "fangs" => {
                let (owner, rises) = held().fangs.get(&id).copied()?;
                json!({ "owner": owner, "delay": (rises - game.world.game_time).max(0) })
            }
            "fangs_set" => {
                let now = game.world.game_time;
                let index = game.fangs.list.iter().position(|fangs| fangs.id == id)?;
                let (mut owner, rises) = held().fangs.get(&id).copied()?;
                let mut delay = (rises - now).max(0) as i32;
                if let Some(wanted) = query["owner"].as_u64() {
                    owner = wanted as u32;
                }
                if let Some(wanted) = query["delay"].as_i64() {
                    delay = wanted as i32;
                }
                let old = game.fangs.list.remove(index);
                game.insert_fangs(id, old.pos, old.yaw, delay, owner);
                held().fangs.insert(id, (owner, now + i64::from(delay)));
                json!(true)
            }
            "boat" => {
                let boat = game.boats.iter().find(|boat| boat.id == id)?;
                json!({ "under_water": boat.under_water(), "in_liquid": boat.in_liquid(), "on_ground": boat.on_ground })
            }
            "projectile" if game.tridents.iter().any(|trident| trident.id == id) => {
                let trident = game.tridents.iter().find(|trident| trident.id == id)?;
                let v = trident.velocity;
                json!({ "kind": "minecraft:trident", "velocity": [v.x, v.y, v.z], "stuck": trident.in_ground, "loyalty": trident.loyalty, "enchanted": !trident.stack.enchantments().is_empty() })
            }
            "projectile" => {
                let p = game.creatures.projectiles.iter().find(|p| p.id == id)?;
                let mut state = json!({
                    "kind": p.kind.id(), "velocity": [p.velocity.x, p.velocity.y, p.velocity.z], "acceleration": p.acceleration,
                    "punch": p.bonus.punch, "stuck": p.stuck,
                });
                if let Kind::Potion { effect, ticks, amplifier, color } = &p.kind {
                    state["potion"] = json!({ "effect": effect, "ticks": ticks, "amplifier": amplifier, "color": color });
                }
                if let Kind::ShulkerBullet { dir, steps, delta, target, .. } = &p.kind {
                    let mob = match p.owner {
                        Some(Who::Mob(id)) => Some(id),
                        _ => None,
                    };
                    state["shulker"] = json!({ "dir": dir, "steps": steps, "delta": delta, "target": who_named(game, mob, Some(p.position), Some(*target)) });
                }
                if let Some(arrow) = p.arrow.as_ref() {
                    state["loyalty"] = json!(arrow.pickup_item.enchantment_level("minecraft:loyalty"));
                    state["pierced"] = json!(arrow.pierced.iter().filter_map(|who| if let Who::Mob(id) = who { Some(*id) } else { None }).collect::<Vec<_>>());
                    state["enchanted"] = json!(!arrow.pickup_item.enchantments().is_empty());
                }
                state
            }
            "wind_explode" => {
                let index = game.creatures.projectiles.iter().position(|p| p.id == id)?;
                let p = game.creatures.projectiles.remove(index);
                let at = p.position;
                let block = [at.x.floor() as i32, at.y.floor() as i32, at.z.floor() as i32];
                let hit = horizon_client::projectiles::Hit::Block { pos: block, at, face: [0, 0, 0] };
                json!(game.wind_charge_hit(&p, hit).is_ok())
            }
            "hit" => {
                let index = game.creatures.projectiles.iter().position(|p| p.id == id)?;
                let p = game.creatures.projectiles.remove(index);
                let at = vector(&query["from"]).map_or(p.position, |from| Vec3::new(from[0], from[1], from[2]));
                json!(game.projectile_hit(&p, horizon_client::projectiles::Hit::Entity { who: who(query)?, at }).is_ok())
            }
            "splash" => {
                let index = game.creatures.projectiles.iter().position(|p| p.id == id)?;
                let p = game.creatures.projectiles.remove(index);
                let at = p.position;
                let block = [at.x.floor() as i32, at.y.floor() as i32, at.z.floor() as i32];
                json!(game.projectile_hit(&p, horizon_client::projectiles::Hit::Block { pos: block, at, face: [0, 1, 0] }).is_ok())
            }
            "trident_loyalty" if game.tridents.iter().any(|trident| trident.id == id) => {
                game.tridents.iter_mut().find(|trident| trident.id == id)?.loyalty = value.as_u64()? as u32;
                json!(true)
            }
            "trident_loyalty" => {
                let p = game.creatures.projectiles.iter_mut().find(|p| p.id == id)?;
                let arrow = p.arrow.as_mut()?;
                let level = value.as_i64()? as i32;
                let data = arrow.pickup_item.data_mut();
                data.enchantments.retain(|(name, _)| &**name != "minecraft:loyalty");
                if level > 0 {
                    data.enchantments.push(("minecraft:loyalty".into(), level as _));
                }
                json!(true)
            }
            "arrow_punch" => {
                game.creatures.projectiles.iter_mut().find(|p| p.id == id)?.bonus.punch = value.as_u64()? as u32;
                json!(true)
            }
            "fireball_acceleration" | "fireball_direction" => {
                let [x, y, z] = vector(value)?;
                let p = game.creatures.projectiles.iter_mut().find(|p| p.id == id)?;
                let wanted = Vec3::new(x, y, z);
                let length = wanted.length();
                if key == "fireball_acceleration" {
                    p.acceleration = length;
                }
                if length > 0.0 {
                    p.velocity = wanted.scale(p.velocity.length().max(1.0e-3) / length);
                } else {
                    p.velocity = Vec3::ZERO;
                }
                p.needs_sync = true;
                json!(true)
            }
            "shulker_target" => {
                // Nobody: an id nothing has, which the bullet looks for and doesn't find.
                let wanted = who(query).unwrap_or(Who::Mob(0));
                let p = game.creatures.projectiles.iter_mut().find(|p| p.id == id)?;
                let Kind::ShulkerBullet { target, .. } = &mut p.kind else {
                    return Some(json!(false));
                };
                *target = wanted;
                json!(true)
            }
            "shulker_delta" | "shulker_steps" | "shulker_dir" => {
                let p = game.creatures.projectiles.iter_mut().find(|p| p.id == id)?;
                let Kind::ShulkerBullet { dir, steps, delta, .. } = &mut p.kind else {
                    return Some(json!(false));
                };
                match key {
                    "shulker_delta" => *delta = vector(value)?,
                    "shulker_steps" => *steps = value.as_i64()? as i32,
                    _ => *dir = value.as_u64().map(|dir| dir as u8),
                }
                json!(true)
            }
            _ => return None,
        })
    })
    .flatten()
}

fn guest_of(game: &Game<'_>, query: &Value) -> Option<usize> {
    let uuid = u128::from_str_radix(query["uuid"].as_str()?, 16).ok()?;
    game.lan.as_ref()?.hub.guests.iter().position(|guest| guest.uuid == uuid)
}

fn stack(value: &Value) -> Option<ItemStack> {
    let (tag, _) = parse_snbt(value.as_str()?).ok()?;
    horizon_client::save::item_from_nbt(&tag)
}

thread_local! {
    /// `openMerchant`: it isn't a click, so none of a click's refusals.
    static TRADER_FORCED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(in crate::game) fn trader_forced() -> bool {
    TRADER_FORCED.get()
}

fn player_read(game: &mut Game<'_>, key: &str, query: &Value) -> Option<Value> {
    let index = guest_of(game, query)?;
    let conn = game.lan.as_ref()?.hub.guests[index].conn;
    if key == "death_score" {
        let saved = game.lan.as_ref()?.hub.guests[index].saved.as_ref();
        return Some(json!(saved.and_then(|saved| saved.get("Score")).and_then(tag_number).unwrap_or(0.0) as i32));
    }
    if key == "previous_mode" {
        let saved = game.lan.as_ref()?.hub.guests[index].saved.as_ref()?;
        return Some(saved.get("previousPlayerGameType").and_then(tag_number).map_or(Value::Null, |mode| json!(mode as i64)));
    }
    if key == "spectator_target" {
        return Some(game.cmds.guest_cameras.get(&conn).map_or(Value::Null, |camera| json!(camera.id)));
    }
    if key == "ender_pearls" {
        use horizon_client::projectiles::ProjectileKind as Kind;
        let pearls = game.creatures.projectiles.iter().filter(|p| matches!(p.kind, Kind::EnderPearl) && p.guest == Some(conn)).map(|p| p.id);
        return Some(json!(pearls.collect::<Vec<_>>()));
    }
    let hub = &game.lan.as_ref()?.hub;
    let guest = &hub.guests[index];
    if let Some(name) = key.strip_prefix("plugin:") {
        return Some(guest.saved.as_ref().and_then(|saved| saved.get(PLUGIN)).and_then(|plugin| plugin.get(name)).map_or(Value::Null, plugin_json));
    }
    Some(match key {
        "simulation_distance" => json!(horizon_client::paper_hub::simulation_of(guest.entity_id).unwrap_or(hub.config.simulation_distance)),
        "fixed_pose" => horizon_client::paper_hub::fixed_pose(guest.entity_id).map_or(Value::Null, |pose| json!(pose)),
        "max_air" => json!(horizon_client::paper_hub::max_air(guest.entity_id)),
        "idle_ticks" => json!(hub.ticks.saturating_sub(guest.last_action_tick)),
        "seen_credits" => json!(guest.seen_credits),
        "sent_chunks" => json!(guest.sent_chunks.iter().map(|(x, z)| [*x, *z]).collect::<Vec<_>>()),
        "address" => json!(guest.address.to_string()),
        "xp_seed" => json!(guest.xp_seed),
        "post_effects" => json!(guest.post_effects),
        "xp_delay" => json!(game.lan.as_ref()?.play.get(&conn)?.xp_delay),
        _ => return None,
    })
}

fn player_write(game: &mut Game<'_>, key: &str, query: &Value) -> Option<bool> {
    let index = guest_of(game, query)?;
    let value = &query["value"];
    if let Some(name) = key.strip_prefix("plugin:") {
        let distance = (name == "simulation_distance").then(|| int(value).clamp(2, 32));
        let hub = &mut game.lan.as_mut()?.hub;
        let attribute = hub.registry_id("minecraft:attribute", "minecraft:max_health");
        let guest = &mut hub.guests[index];
        guest_plugin_set(guest, name, match distance {
            Some(distance) => plugin_tag(&json!(distance)),
            None => plugin_tag(value),
        });
        guest_plugin_apply(guest, attribute);
        if let Some(distance) = distance {
            guest.send(&C::SetSimulationDistance(play::Distance { distance }));
        }
        return Some(true);
    }
    if key == "load_data" {
        let root = game.world_root.clone();
        let hub = &mut game.lan.as_mut()?.hub;
        return Some(horizon_client::lan::playerdata::load(&mut hub.guests[index], &root));
    }
    if key == "fixed_pose" {
        let guest = &game.lan.as_ref()?.hub.guests[index];
        horizon_client::paper_hub::set_fixed_pose(guest.entity_id, value.as_i64().map(|pose| pose as i32));
        let pose = guest.pose() as i32;
        guest.send(&C::SetEntityData(play::SetEntityData { id: guest.entity_id, values: vec![(6, horizon_protocol::entity_data::serializer::POSE, horizon_protocol::entity_data::Value::Id(pose))] }));
        return Some(true);
    }
    if key == "xp_delay" {
        let host = game.lan.as_mut()?;
        let conn = host.hub.guests[index].conn;
        host.play.get_mut(&conn)?.xp_delay = int(value).max(0) as u32;
        return Some(true);
    }
    let hub = &mut game.lan.as_mut()?.hub;
    let ticks = hub.ticks;
    let guest = &mut hub.guests[index];
    match key {
        "sprinting" => guest.sprinting = truth(value),
        "riptide" => guest.living_flags = if truth(value) { guest.living_flags | 4 } else { guest.living_flags & !4 },
        "sneaking" => guest.sneaking = truth(value),
        "seen_credits" => guest.seen_credits = truth(value),
        "xp_seed" => guest.xp_seed = int(value),
        "idle" => guest.last_action_tick = ticks,
        "above_ground" => guest.above_ground_ticks = 0,
        "view_distance" => {
            guest.view_distance = int(value).clamp(2, 32);
            let distance = guest.view_distance;
            guest.send(&C::SetChunkCacheRadius(play::Distance { distance }));
        }
        "last_death" => guest.last_death = value["dimension"].as_str().zip(block(&value["pos"])).map(|(dimension, pos)| (dimension.to_owned(), pos)),
        "post_effects" => {
            guest.post_effects = value.as_array().into_iter().flatten().filter_map(|key| key.as_str().map(str::to_owned)).collect();
            hub.send_post_effects(index);
        }
        "post_effects_send" => hub.send_post_effects(index),
        _ => return None,
    }
    Some(true)
}

fn player_send(game: &mut Game<'_>, kind: &str, query: &Value) -> Option<bool> {
    // In a player's turn `leave_guest` puts their old play state back over what this would write,
    // leaving the client a window with nothing behind it: it opens when their turn ends.
    if kind == "open_station" && game.lan_acting.is_some() {
        super::paper_bridge::push_action(json!({ "op": "open_station", "request": query }).to_string());
        return Some(true);
    }
    let index = guest_of(game, query)?;
    if kind == "save" {
        let mut host = game.lan.take()?;
        let conn = host.hub.guests[index].conn;
        if let Some(guest) = host.hub.guest(conn) {
            game.save_guest(&host, guest);
        }
        game.lan = Some(host);
        return Some(true);
    }
    if kind == "map" {
        let map_id = query["id"].as_i64()? as i32;
        let map = game.map(map_id)?;
        let packet = play::MapItemData { map_id, scale: map.scale as i8, locked: map.locked, decorations: None, patch: Some(play::MapPatch { start_x: 0, start_y: 0, width: 128, height: 128, colors: map.colors.clone() }) };
        game.lan.as_ref()?.hub.guests[index].send(&C::MapItemData(packet));
        return Some(true);
    }
    if kind == "open_station" {
        let pos = block(&query["pos"])?;
        let name = query["block"].as_str()?;
        let (conn, player) = game.lan.as_ref().map(|host| (host.hub.guests[index].conn, host.hub.guests[index].uuid))?;
        if query["force"].as_bool().unwrap_or(false) {
            horizon_client::paper_hub::FORCED_MENUS.lock().unwrap_or_else(|e| e.into_inner()).retain(|menu| menu.0 != player);
            horizon_client::paper_hub::FORCED_MENUS.lock().unwrap_or_else(|e| e.into_inner()).push((player, pos));
        }
        let mut host = game.lan.take()?;
        let opened = if name == "minecraft:crafting_table" {
            game.lan_open_crafting(&mut host, conn, pos);
            true
        } else {
            super::stations::Kind::from_block(name).is_some_and(|kind| game.lan_open_station(&mut host, conn, &super::stations::Station::new_kind(kind, pos)))
        };
        game.lan = Some(host);
        return Some(opened);
    }
    let hub = &mut game.lan.as_mut()?.hub;
    match kind {
        "open_book" => {
            let book = stack(&query["stack"]);
            let selected = hub.guests[index].selected;
            let held = hub.guests[index].inventory.get(selected).cloned().flatten();
            let (held, book) = (hub.stack(&held), hub.stack(&book));
            let guest = &hub.guests[index];
            guest.send(&C::SetPlayerInventory(play::SetPlayerInventory { slot: selected as i32, item: book }));
            guest.send(&C::OpenBook(play::OpenBook { hand: 0 }));
            guest.send(&C::SetPlayerInventory(play::SetPlayerInventory { slot: selected as i32, item: held }));
        }
        "equipment" => {
            let item = stack(&query["stack"]);
            let item = hub.stack(&item);
            let id = query["entity"].as_i64()? as i32;
            let slot = query["slot"].as_u64()? as u8;
            hub.guests[index].send(&C::SetEquipment(play::SetEquipment { id, slots: vec![(slot, item)] }));
        }
        "potion" | "potion_remove" => {
            let name = query["effect"].as_str()?;
            let name = if name.contains(':') { name.to_owned() } else { format!("minecraft:{}", name.to_ascii_lowercase()) };
            let effect = hub.registry_id("minecraft:mob_effect", &name)?;
            let id = query["entity"].as_i64()? as i32;
            if kind == "potion_remove" {
                hub.guests[index].send(&C::RemoveMobEffect(play::RemoveMobEffect { id, effect }));
            } else {
                let flags = i8::from(truth(&query["ambient"])) | if truth(&query["particles"]) { 2 } else { 0 };
                hub.guests[index].send(&C::UpdateMobEffect(play::UpdateMobEffect { id, effect, amplifier: int(&query["amplifier"]), duration: int(&query["duration"]), flags }));
            }
        }
        "block_entity" => {
            let pos = block(&query["pos"])?;
            let (tag, _) = parse_snbt(query["snbt"].as_str()?).ok()?;
            let state = horizon_world::registry::BlockStateId(query["state"].as_u64().unwrap_or(0) as u32);
            let kind = match query["kind_of"].as_i64() {
                Some(kind) => kind as i32,
                None => hub.encoder.entity_type(state.0)?,
            };
            hub.guests[index].send(&C::BlockEntityData(play::BlockEntityData { pos, kind, tag }));
        }
        "cursor" => {
            let item = stack(&query["stack"]);
            let net = hub.stack(&item);
            let guest = &mut hub.guests[index];
            guest.carried = item;
            guest.send(&C::SetCursorItem(play::SetCursorItem { item: net }));
        }
        "wake" => hub.wake(index),
        "sleep" => {
            let pos = block(&query["pos"])?;
            let click = play::UseItemOn { hand: 0, pos, face: 1, cursor: [0.5, 0.5, 0.5], inside: false, world_border: false, sequence: 0 };
            hub.guests[index].inbox.push_back((play::Serverbound::UseItemOn(click), 0));
        }
        "border" => {
            let packet = play::InitializeBorder {
                center_x: query["x"].as_f64()?,
                center_z: query["z"].as_f64()?,
                old_size: query["size"].as_f64()?,
                new_size: query["size"].as_f64()?,
                lerp_time: 0,
                absolute_max_size: 29_999_984,
                warning_blocks: int(&query["warning_blocks"]),
                warning_time: int(&query["warning_time"]),
            };
            hub.guests[index].send(&C::InitializeBorder(packet));
        }
        "death_score" => {
            if let Tag::Compound(saved) = hub.guests[index].saved.get_or_insert_with(|| Tag::Compound(Default::default())) {
                saved.insert("Score".into(), Tag::Int(int(&query["value"])));
            }
            let guest = &hub.guests[index];
            let values = vec![(18, horizon_protocol::entity_data::serializer::INT, horizon_protocol::entity_data::Value::Int(int(&query["value"])))];
            guest.send(&C::SetEntityData(play::SetEntityData { id: guest.entity_id, values }));
        }
        "ender_item" => {
            let item = stack(&query["stack"]);
            let slot = query["slot"].as_u64()? as usize;
            *hub.guests[index].ender_items.get_mut(slot)? = item;
        }
        "links" => {
            let links = query["links"].as_array()?.iter().map(|row| {
                let label = component_nbt(&horizon_client_render::packs::component(&row["label"]).component());
                horizon_protocol::packets::configuration::ServerLink { kind: horizon_protocol::packets::configuration::ServerLinkKind::Custom(label), link: row["url"].as_str().unwrap_or_default().to_owned() }
            });
            hub.guests[index].send(&C::ServerLinks(horizon_protocol::packets::configuration::ServerLinks { links: links.collect() }));
        }
        "profile" => {
            let name = query["name"].as_str()?.to_owned();
            let properties: Vec<horizon_protocol::packets::login::Property> = query["properties"].as_array()?.iter().filter_map(|row| Some(horizon_protocol::packets::login::Property { name: row["name"].as_str()?.to_owned(), value: row["value"].as_str()?.to_owned(), signature: row["signature"].as_str().map(str::to_owned) })).collect();
            let guest = &mut hub.guests[index];
            guest.properties = properties.clone();
            let (uuid, id, mode) = (guest.uuid, guest.entity_id, guest.game_mode);
            hub.broadcast(&C::PlayerInfoRemove(play::PlayerInfoRemove { uuids: vec![uuid] }));
            let entry = play::PlayerInfo { uuid, name: Some(name), properties, chat_session: None, game_mode: Some(mode), listed: Some(true), latency: None, display_name: None, list_order: None, show_hat: None };
            hub.broadcast(&C::PlayerInfoUpdate(play::PlayerInfoUpdate { actions: 1 | 4 | 8, entries: vec![entry] }));
            hub.respawned(id);
            // Their own client too, at the hub's tick, where it has the level to tell them of.
            horizon_client::paper_hub::ask_refresh(id);
        }
        "respawn" => hub.guests[index].inbox.push_back((play::Serverbound::ClientCommand(play::ClientCommand { action: 0 }), 0)),
        "drop" => {
            let action = if truth(&query["all"]) { 3 } else { 4 };
            hub.guests[index].inbox.push_back((play::Serverbound::PlayerAction(play::PlayerAction { action, pos: [0, 0, 0], direction: 0, sequence: 0 }), 0));
        }
        "window_property" => {
            let guest = &hub.guests[index];
            let container = guest.menu.as_ref()?.window;
            guest.send(&C::ContainerSetData(play::ContainerSetData { container, id: int(&query["property"]) as i16, value: int(&query["value"]) as i16 }));
        }
        "health_scale" => {
            let attribute = hub.registry_id("minecraft:attribute", "minecraft:max_health")?;
            let guest = &mut hub.guests[index];
            let id = guest.entity_id;
            horizon_client::paper_hub::set_health_scale(id, query["scale"].as_f64());
            let Some(scale) = query["scale"].as_f64() else {
                // Back to the real bar: its health at the next tick, and the real attributes now.
                guest.sent_health = None;
                if let Some(motion) = guest.motion.as_ref() {
                    let values = horizon_client::lan::attributes::syncable(&motion.attributes);
                    guest.send(&C::UpdateAttributes(play::UpdateAttributes { id, values }));
                }
                return Some(true);
            };
            // The hub sends the health itself from here on, scaled.
            guest.sent_health = None;
            guest.send(&C::UpdateAttributes(play::UpdateAttributes { id, values: vec![play::AttributeSnapshot { attribute, base: scale, modifiers: Vec::new() }] }));
        }
        _ => return None,
    }
    Some(true)
}

/// What needs the player's own turn: their state swapped into the game.
fn player_turn(game: &mut Game<'_>, key: &str, query: &Value) -> Option<Value> {
    let uuid = u128::from_str_radix(query["uuid"].as_str()?, 16).ok()?;
    let target = game.target_by_uuid(uuid)?;
    game.for_player(target, |game| {
        Some(match key {
            "mending" => json!(game.mending(query["value"].as_u64()? as u32)),
            "attack_strength" => json!(game.attack_strength_scale()),
            "give" => {
                let mut left = Vec::new();
                let mut drops = Vec::new();
                for row in query["stacks"].as_array()? {
                    let Some(mut stack) = stack(row) else {
                        continue;
                    };
                    let most = game.res.items().get(&stack.item).map_or(64, |item| item.max_stack as u32);
                    for slot in game.inventory.iter_mut().flatten() {
                        if slot.item == stack.item && slot.extra == stack.extra && slot.damage == stack.damage && slot.count < most {
                            let moved = (most - slot.count).min(stack.count);
                            slot.count += moved;
                            stack.count -= moved;
                        }
                    }
                    for slot in game.inventory.iter_mut().take(36) {
                        if stack.count > 0 && slot.is_none() {
                            let moved = most.min(stack.count);
                            let mut here = stack.clone();
                            here.count = moved;
                            *slot = Some(here);
                            stack.count -= moved;
                        }
                    }
                    if stack.count > 0 {
                        left.push(super::commands::paper_entities::tag_json(&horizon_client::save::item_to_nbt(&stack)));
                        if truth(&query["drop"]) {
                            let at = game.player.as_ref()?.position;
                            game.spawn_item(stack, [at.x, at.y, at.z], [0.0, 0.0, 0.0], 0);
                            drops.push(game.next_entity);
                        }
                    }
                }
                json!({ "left": left, "drops": drops })
            }
            "attack_delay" => json!(horizon_client::attack_strength::delay(game.main_hand_attack_speed())),
            "hook" => game.hook_status().unwrap_or(Value::Null),
            "absorption" => json!(game.player.as_ref()?.health.absorption()),
            "set_absorption" => {
                let amount = query["value"].as_f64()? as f32;
                json!(game.player.as_mut()?.internal_set_absorption(amount).is_ok())
            }
            "shoulder_set" => {
                let id = query["entity"].as_u64()? as u32;
                let side = query["side"].as_str()?;
                let tag = game.entity_nbt(Target::Mob(id))?;
                let Some(Tag::Compound(map)) = game.level.player_extra.as_mut() else {
                    return Some(json!(false));
                };
                map.insert(side.into(), tag);
                game.mobs.iter_mut().find(|m| m.id == id)?.removed = true;
                json!(true)
            }
            "shoulder_release" => {
                let side = query["side"].as_str()?;
                let at = game.player.as_ref()?.position;
                let Some(Tag::Compound(map)) = game.level.player_extra.as_mut() else {
                    return None;
                };
                let mut tag = map.remove(&side.into())?;
                if let Tag::Compound(saved) = &mut tag {
                    saved.remove(&"UUID".into());
                    saved.insert("Pos".into(), Tag::List([at.x, at.y + 0.7, at.z].iter().map(|v| Tag::Double(v.to_bits())).collect()));
                }
                game.load_mob(tag, None).map_or(Value::Null, |id| json!(id))
            }
            "merchant" => {
                let id = query["entity"].as_u64()? as u32;
                let index = game.mobs.iter().position(|m| m.id == id)?;
                if game.mobs[index].extra.flag2 && !truth(&query["force"]) {
                    return Some(json!(false));
                }
                TRADER_FORCED.set(true);
                let opened = game.interact_trader(index, "").unwrap_or(false);
                TRADER_FORCED.set(false);
                json!(opened)
            }
            "crit" => json!(game.critical_attack(1.0)),
            "reel" => {
                let rod = if query["hand"] == "OFF_HAND" { game.stack_in(super::shield::OFFHAND_SLOT).cloned() } else { game.held().cloned() };
                // Java refuses with anything else in that hand.
                let rod = rod.filter(|rod| &*rod.item == "minecraft:fishing_rod")?;
                json!(game.retrieve_hook(&rod))
            }
            "sleep_force" => {
                let pos = block(&query["pos"])?;
                if game.sleeping.is_some() || game.player.as_ref()?.health.dead_or_dying() {
                    return Some(json!(false));
                }
                let state = game.world.block(pos[0], pos[1], pos[2]);
                let props = game.pack.assets.registry.state(state)?.properties.clone();
                let d: [i32; 2] = match props.get("facing").map_or("north", String::as_str) {
                    "south" => [0, 1],
                    "west" => [-1, 0],
                    "east" => [1, 0],
                    _ => [0, -1],
                };
                let head = if props.get("part").map(String::as_str) == Some("foot") { [pos[0] + d[0], pos[1], pos[2] + d[1]] } else { pos };
                let yaw = game.player.as_ref()?.yaw;
                if game.bed_sets_spawn(state) {
                    game.respawn_config = Some(super::respawn_anchor::RespawnConfig::wrapped(game.dim, head, yaw, false));
                }
                game.stop_mining();
                game.left = false;
                game.right = false;
                game.using = None;
                game.sleeping = Some(head);
                game.sleep_timer = 0;
                let at = Vec3::new(f64::from(head[0]) + 0.5, f64::from(head[1]) + 0.5625 + 0.125, f64::from(head[2]) + 0.5);
                if let Some(p) = game.player.as_mut() {
                    p.position = at;
                    p.velocity = Vec3::ZERO;
                    p.pitch = 0.0;
                    p.teleported = true;
                }
                game.pitch = 0.0;
                if game.lan_acting.is_none() {
                    game.previous_position = at;
                    game.snap_player_model_pitch();
                }
                game.occupy_bed(head);
                game.stats.custom("sleep_in_bed", 1);
                game.trigger_player("minecraft:slept_in_bed");
                game.stats.reset("custom", "minecraft:time_since_rest");
                json!(true)
            }
            "stuck" => {
                let stuck = &game.combat.stuck;
                json!({ "arrows": stuck.arrows, "stingers": stuck.stingers, "arrow_time": stuck.remove_arrow_time, "stinger_time": stuck.remove_stinger_time })
            }
            "hook_rules" => {
                let hook = game.hook.as_ref()?;
                let (rules, at) = (hook.rules, hook.position);
                let above = horizon_core::position::BlockPos::new(at.x.floor() as i32, at.y.floor() as i32 + 1, at.z.floor() as i32);
                let raining = game.with_level(|b, level, _| b.is_raining_at(level, above));
                json!({
                    "min_wait": rules.wait.0,
                    "max_wait": rules.wait.1,
                    "min_lure": rules.lure_time.0,
                    "max_lure": rules.lure_time.1,
                    "min_angle": rules.lure_angle.0,
                    "max_angle": rules.lure_angle.1,
                    "apply_lure": rules.apply_lure,
                    "sky": rules.sky,
                    "rain": rules.rain,
                    "bite_chance": rules.bite_chance.unwrap_or(if raining { 1.0 / 300.0 } else { 1.0 / 500.0 }),
                })
            }
            "hook_set" => hook_set(game, query)?,
            "food_rates" => json!(game.player.as_ref()?.health.food.rates),
            "orb_killer" => Value::Null,
            "hurt_dir" => json!(game.player.as_ref()?.health.hurt_dir),
            "set_hurt_dir" => {
                game.player.as_mut()?.health.hurt_dir = query["value"].as_f64()? as f32;
                json!(true)
            }
            "riptide" => {
                let player = game.player.as_mut()?;
                player.auto_spin_attack_ticks = int(&query["duration"]);
                player.auto_spin_attack_dmg = query["strength"].as_f64()? as f32;
                let entity = player.entity_id;
                match stack(&query["item"]) {
                    Some(item) => {
                        held().spin_items.insert(entity, item);
                    }
                    None => {
                        held().spin_items.remove(&entity);
                    }
                }
                json!(true)
            }
            "hook_pull" => json!(game.pull_hooked()),
            "set_stuck" => {
                let value = query["value"].as_i64()?.max(0) as i32;
                match query["what"].as_str()? {
                    "arrows" => game.combat.stuck.arrows = value,
                    "arrow_time" => game.combat.stuck.remove_arrow_time = value,
                    "stinger_time" => game.combat.stuck.remove_stinger_time = value,
                    _ => game.combat.stuck.stingers = value,
                }
                json!(true)
            }
            "warden" => {
                let tracker = &game.sculk.tracker;
                json!({ "level": tracker.warning_level, "cooldown": tracker.cooldown_ticks, "since": tracker.ticks_since_last_warning })
            }
            "set_warden" => {
                let value = query["value"].as_i64()? as i32;
                let tracker = &mut game.sculk.tracker;
                match query["what"].as_str()? {
                    "level" => tracker.set_warning_level(value),
                    "cooldown" => tracker.cooldown_ticks = value,
                    _ => tracker.ticks_since_last_warning = value,
                }
                json!(true)
            }
            "boost" => {
                let firework = stack(&query["stack"])?;
                let gliding = game.player.as_ref()?.fall_flying;
                if &*firework.item != "minecraft:firework_rocket" || !gliding {
                    return Some(Value::Null);
                }
                let at = game.player.as_ref()?.position;
                let mut rocket = game.new_rocket(at, &firework, true);
                rocket.owner = Some(Who::Player);
                rocket.guest = game.lan_acting;
                game.rockets.push(rocket);
                game.rockets.last().map_or(Value::Null, |rocket| json!(rocket.id))
            }
            _ => return None,
        })
    })
    .flatten()
}

fn player_query(game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    let key = query["key"].as_str().unwrap_or_default();
    Some(match op {
        "entity_object" => object(game, key, query).or_else(|| object_more(game, key, query)).unwrap_or(Value::Null),
        "entity_player_get" => player_read(game, key, query).unwrap_or(Value::Null),
        "entity_player_set" => json!(player_write(game, key, query).unwrap_or(false)),
        "entity_player_send" => json!(player_send(game, query["kind"].as_str().unwrap_or_default(), query).unwrap_or(false)),
        "entity_player_turn" => player_turn(game, key, query).unwrap_or(Value::Null),
        _ => return None,
    })
}

/// More of a mob's facts, past `read`.
/// `ItemStack.getUseDuration` of what a mob can be seen using: a witch's potion, a crossbow's charge, else a bow's
/// or trident's hold.
fn use_duration(m: &Mob) -> i32 {
    if m.species() == Species::Witch {
        32
    } else if m.extra.main_hand.as_deref() == Some("minecraft:crossbow") {
        28
    } else {
        72_000
    }
}

fn read_more(m: &Mob, key: &str) -> Option<Value> {
    Some(match key {
        "size" => json!([m.width(), m.height(), m.eye_height()]),
        "last_damage" => m.last_damage.map_or(Value::Null, |(damage, _)| json!({ "kind": format!("{:?}", damage.kind), "amount": m.last_hurt })),
        "vehicle" => m.extra.mount.map_or(Value::Null, |id| json!(id)),
        "pose" => json!(mobs::sync::pose_id(m)),
        "invisible" => json!(m.invisible),
        "gravity" => json!(m.gravity()),
        "hurt_dir" => json!(m.extra.hurt_dir),
        "fire_immune" => json!(m.kind.flags.fire_immune),
        "use_left" | "use_used" => {
            let full = use_duration(m);
            let left = match m.species() {
                Species::Witch => m.extra.using_item,
                _ if m.extra.using_item > 0 => (full - m.extra.using_item).max(0),
                _ => 0,
            };
            json!(if key == "use_left" {
                left
            } else if m.extra.using_item > 0 {
                full - left
            } else {
                0
            })
        }
        "eye_in_water" => json!(m.fluids.eye_in_water),
        "in_water" => json!(m.fluids.in_water),
        "in_lava" => json!(m.fluids.in_lava),
        "swim_sound" => json!(mobs::ai::swim_sound(m)),
        "invulnerable_ticks" => json!(m.invulnerable),
        "last_hurt" => json!(m.last_hurt),
        "using_item" => json!(m.extra.using_item),
        "use_hand" => json!(m.extra.using_off_hand),
        "swimming" => json!(m.species() == Species::Drowned && m.extra.flag),
        "jumping" => json!(m.ai.jumping),
        "no_action" => json!(m.ai.no_action_time),
        "body_yaw" => json!(m.body_yaw),
        "forwards" => json!(m.ai.zza),
        "sideways" => json!(m.ai.xxa),
        "upwards" => json!(m.ai.yya),
        "climbing" => json!(m.climbing),
        "sound_volume" => json!(mobs::ai::sound_volume(m)),
        "voice" => json!(mobs::ai::voice_sounds(m)),
        "max_head_pitch" => json!(m.max_head_x_rot()),
        "wololo" => json!(m.extra.counter2),
        "air_drag" => json!(m.air_drag()),
        "max_air" => json!(match m.species() {
            Species::Axolotl => mobs::axolotl::MAX_AIR,
            Species::Dolphin => 4800,
            _ => m.extra.max_air(),
        }),
        "boss_progress" => json!(if m.extra.counter > 0 { 1.0 - m.extra.counter as f32 / mobs::wither::INVULNERABLE as f32 } else { (m.health / m.attrs.max_health).clamp(0.0, 1.0) }),
        "stared_at" => json!(m.extra.stared_at),
        "arrows" => json!(m.extra.arrow_count),
        "stingers" => json!(m.extra.stinger_count),
        "arrow_time" => json!(m.extra.arrow_time),
        "stinger_time" => json!(m.extra.stinger_time),
        "stew" => m.extra.stew_effects.as_ref().map_or(Value::Null, |effects| json!(effects.iter().map(|(id, ticks)| json!([&**id, ticks])).collect::<Vec<_>>())),
        "head_rotation" => json!(m.kind.head_rot_speed),
        _ => return None,
    })
}

/// The player a mob's last hit came from: the guest it's bound to, else the only one there is.
fn killer(game: &Game<'_>, index: usize) -> Option<String> {
    let m = &game.mobs[index];
    if m.ai.last_hurt_by_player <= 0 {
        return None;
    }
    let host = game.lan.as_ref()?;
    let conn = game.last_hurt_by_player.get(&m.id).copied().flatten().or_else(|| host.mob_players.get(&m.id).copied());
    let guest = conn.and_then(|conn| host.hub.guest(conn));
    let guest = guest.or_else(|| (host.hub.guests.len() == 1).then(|| &host.hub.guests[0]))?;
    Some(format!("{:032x}", guest.uuid))
}

fn write_more(game: &mut Game<'_>, index: usize, key: &str, value: &Value) -> Option<bool> {
    let on = truth(value);
    let number = int(value);
    if key == "loot_table" {
        let id = game.mobs[index].id;
        game.mobs[index].extra.loot_table = value["table"].as_str().map(str::to_owned);
        game.mobs[index].extra.loot_seed = value["seed"].as_i64().unwrap_or(0);
        if let Tag::Compound(saved) = game.persist.raw.entry(id).or_insert_with(|| Tag::Compound(Default::default())) {
            match value["table"].as_str() {
                Some(table) => {
                    saved.insert("DeathLootTable".into(), Tag::String(table.into()));
                    saved.insert("DeathLootTableSeed".into(), Tag::Long(value["seed"].as_i64().unwrap_or(0)));
                }
                None => {
                    saved.remove(&"DeathLootTable".into());
                    saved.remove(&"DeathLootTableSeed".into());
                }
            }
        }
        return Some(true);
    }
    if key == "explosion_radius" {
        let id = game.mobs[index].id;
        game.mobs[index].extra.explosion_radius = Some(number);
        if let Tag::Compound(saved) = game.persist.raw.entry(id).or_insert_with(|| Tag::Compound(Default::default())) {
            saved.insert("ExplosionRadius".into(), Tag::Byte(number as i8));
        }
        return Some(true);
    }
    if key == "killer" {
        let uuid = value.as_str().and_then(|uuid| u128::from_str_radix(uuid, 16).ok());
        let m = &mut game.mobs[index];
        match uuid {
            Some(_) => {
                m.ai.last_hurt_by = Some(Who::Player);
                m.ai.last_hurt_by_player = 100;
            }
            None => m.ai.last_hurt_by_player = 0,
        }
        return Some(true);
    }
    let m = &mut game.mobs[index];
    match key {
        "pose" => {
            if !matches!(m.species(), Species::Frog | Species::Cat | Species::Ocelot | Species::Goat) {
                return Some(false);
            }
            m.extra.pose = number.clamp(0, 17) as u8;
        }
        "invisible" => m.invisible = on,
        "wololo" => m.extra.counter2 = number,
        "raid" => {
            let state = m.extra.raider.get_or_insert_default();
            state.raid_id = value.as_i64().map(|id| id as i32);
        }
        "stared_at" => m.extra.stared_at = on,
        "arrows" => m.extra.arrow_count = number.max(0),
        "stingers" => m.extra.stinger_count = number.max(0),
        "arrow_time" => m.extra.arrow_time = number.max(0),
        "stinger_time" => m.extra.stinger_time = number.max(0),
        "immovable" => m.extra.immovable = on,
        "description" => {
            m.extra.description = match value.as_str() {
                Some(snbt) => mobs::MannequinDescription::Custom(parse_snbt(snbt).ok()?.0),
                None => mobs::MannequinDescription::Hidden,
            };
        }
        "trusted" => {
            let words = |row: &Value| Some([int(row.get(0)?), int(row.get(1)?), int(row.get(2)?), int(row.get(3)?)]);
            m.extra.trusted = [words(&value[0]), words(&value[1])];
        }
        "stew" => {
            m.extra.stew_effects = value.as_array().map(|rows| rows.iter().filter_map(|row| Some((std::sync::Arc::from(row[0].as_str()?), int(&row[1])))).collect());
        }
        "invulnerable_ticks" => m.invulnerable = number.max(0),
        "last_hurt" => m.last_hurt = value.as_f64().unwrap_or(0.0) as f32,
        "using_item" => m.extra.using_item = number.max(0),
        "use_hand" => m.extra.using_off_hand = on,
        "swimming" => {
            // `Entity.updateSwimming` takes it back next tick from whatever isn't sprinting in water, and no mob is.
            if m.species() != Species::Drowned {
                if on {
                    held().swimming.insert(m.id);
                } else {
                    held().swimming.remove(&m.id);
                }
                return Some(true);
            }
            m.extra.flag = on;
        }
        "use_left" => {
            let full = use_duration(m);
            m.extra.using_item = if m.species() == Species::Witch || number <= 0 { number.max(0) } else { (full - number).max(1) };
        }
        "jumping" => m.ai.jumping = on,
        "no_action" => m.ai.no_action_time = number,
        "body_yaw" => {
            m.body_yaw = value.as_f64().unwrap_or(0.0) as f32;
            m.previous_body_yaw = m.body_yaw;
        }
        "max_health" => {
            m.attrs.max_health = match value.as_f64() {
                Some(health) => health as f32,
                None => m.kind.max_health,
            };
            m.health = m.health.min(m.attrs.max_health);
        }
        _ => return None,
    }
    m.needs_sync = true;
    Some(true)
}

fn mob_action_more(game: &mut Game<'_>, index: usize, action: &str, query: &Value) -> Option<Value> {
    Some(match action {
        "knockback" => {
            let [power, x, z] = vector(&query["value"])?;
            let m = &mut game.mobs[index];
            m.knockback(power, x, z);
            m.needs_sync = true;
            json!(true)
        }
        "attack" => {
            let target = who(query)?;
            with_world(game, index, |m, cx| mobs::ai::do_hurt_target(m, target, cx));
            json!(true)
        }
        "sees" => {
            let target = who(query)?;
            json!(with_world(game, index, |m, cx| m.sees(cx.w, target)))
        }
        "look_control" => {
            let at = vector(&query["at"])?;
            game.mobs[index].look_at(at);
            json!(true)
        }
        "daylight" => {
            let (m, others) = (game.mobs[index].clone(), Vec::new());
            let world = game.mob_world_base(&others);
            let top = (m.position.x.floor() as i32, (m.position.y + f64::from(m.eye_height() as f32)).floor() as i32, m.position.z.floor() as i32);
            json!(world.daylight && mobs::hostile::mob_brightness(&m, &world) > 0.5 && world.can_see_sky(top.0, top.1, top.2))
        }
        "ram" => {
            let [x, y, z] = vector(&query["at"])?;
            mobs::goat::force_ram(&mut game.mobs[index], Vec3::new(x, y, z));
            json!(true)
        }
        "can_dig" => json!(with_world(game, index, |m, cx| mobs::animal_brain::can_dig(m, cx.w))),
        "dig_spot" => with_world(game, index, |m, cx| mobs::animal_brain::dig_position(m, cx.w)).map_or(Value::Null, |p| json!(p)),
        "level_up" => {
            let amount = int(&query["amount"]);
            let level = i32::from(game.mobs[index].extra.variant3.max(1));
            if amount <= 0 || level + amount > 5 {
                return Some(json!(false));
            }
            game.ensure_offers(index);
            for next in level + 1..=level + amount {
                let more = game.offers_at_level(index, next);
                let m = &mut game.mobs[index];
                m.extra.variant3 = next as u8;
                m.extra.trade.get_or_insert_default().offers.extend(more);
            }
            game.mobs[index].needs_sync = true;
            json!(true)
        }
        "add_trades" => {
            let amount = int(&query["amount"]).max(0) as usize;
            game.ensure_offers(index);
            let level = i32::from(game.mobs[index].extra.variant3.max(1));
            let mut more = game.offers_at_level(index, level);
            more.truncate(amount);
            let added = !more.is_empty();
            game.mobs[index].extra.trade.get_or_insert_default().offers.extend(more);
            json!(added)
        }
        "villager_item" => {
            let slot = query["slot"].as_u64()? as usize;
            let item = stack(&query["stack"]);
            let list = &mut mobs::villager::ext_mut(&mut game.mobs[index]).inventory;
            match item {
                Some(item) if slot < list.len() => list[slot] = item,
                Some(item) if list.len() < 8 => list.push(item),
                None if slot < list.len() => {
                    list.remove(slot);
                }
                _ => return Some(json!(false)),
            }
            json!(true)
        }
        "shear" => shear(game, index),
        "wither_target" => {
            let m = &game.mobs[index];
            match query["head"].as_str()? {
                "LEFT" => json!({ "entity": m.extra.ints[0] }),
                "RIGHT" => json!({ "entity": m.extra.ints[1] }),
                _ => who_named(game, Some(m.id), Some(m.position), m.ai.target),
            }
        }
        "wither_target_set" => {
            let entity = query["entity"].as_u64().map(|id| id as u32);
            let m = &mut game.mobs[index];
            match query["head"].as_str()? {
                "LEFT" => m.extra.ints[0] = entity.map_or(0, |id| id as i32),
                "RIGHT" => m.extra.ints[1] = entity.map_or(0, |id| id as i32),
                _ => {
                    let id = m.id;
                    m.ai.target = who(query);
                    bind_player(game, id, query);
                }
            }
            json!(true)
        }
        "experience" => {
            let mut copy = game.mobs[index].clone();
            json!(mobs::drops::base_experience(&mut copy))
        }
        _ => return None,
    })
}

/// What any entity answers, whatever holds it.
fn common(game: &mut Game<'_>, key: &str, query: &Value) -> Option<Value> {
    let id = id_of(query);
    game.native_in_level(&dimension(query), |game| {
        let target = game.level_entity(i32::try_from(id).ok()?);
        Some(match key {
            "ticks" => match target? {
                Target::Mob(_) => json!(game.mobs.iter().find(|m| m.id == id)?.age),
                Target::Item(_) => json!(game.dropped.iter().find(|e| e.id == id)?.age),
                Target::Orb(_) => json!(game.orbs.iter().find(|o| o.id == id)?.ticks),
                Target::Projectile(_) => json!(game.creatures.projectiles.iter().find(|p| p.id == id)?.age),
                Target::Boat(_) => json!(game.boats.iter().find(|b| b.id == id)?.age),
                Target::Cart(_) => json!(game.carts.iter().find(|c| c.id == id)?.tick_count),
                _ => return None,
            },
            "set_ticks" => {
                let ticks = query["value"].as_i64()?.max(0);
                match target? {
                    Target::Mob(_) => game.mobs.iter_mut().find(|m| m.id == id)?.age = ticks as u32,
                    Target::Item(_) => game.dropped.iter_mut().find(|e| e.id == id)?.age = ticks as _,
                    Target::Orb(_) => game.orbs.iter_mut().find(|o| o.id == id)?.ticks = ticks as u32,
                    Target::Projectile(_) => game.creatures.projectiles.iter_mut().find(|p| p.id == id)?.age = ticks as u32,
                    Target::Boat(_) => game.boats.iter_mut().find(|b| b.id == id)?.age = ticks as i32,
                    Target::Cart(_) => game.carts.iter_mut().find(|c| c.id == id)?.tick_count = ticks as i32,
                    _ => return Some(json!(false)),
                }
                json!(true)
            }
            "copy" => {
                let mut tag = game.entity_nbt(target?)?;
                if let Tag::Compound(map) = &mut tag {
                    map.remove(&"UUID".into());
                    map.remove(&"Passengers".into());
                    let at = vector(&query["at"])?.map(|v| Tag::Double(v.to_bits()));
                    map.insert("Pos".into(), Tag::List(at.to_vec()));
                    map.insert("id".into(), Tag::String(game.target_type(target?).as_str().into()));
                }
                game.native_in_level(query["to"].as_str()?, |game| {
                    let copy = game.spawn_entity_from_nbt(tag)?;
                    Some(game.native_entity(&json!({ "id": target_number(copy)?, "data": false })))
                })
                .flatten()
                .unwrap_or(Value::Null)
            }
            "as_string" => {
                let tag = game.entity_nbt(target?)?;
                json!(horizon_world::snbt_text::to_snbt(&tag))
            }
            "ticking" => {
                let at = game.target_position(target?)?;
                let ticking = game.entity_ticking_area();
                json!(ticking(at[0].floor() as i32, at[2].floor() as i32))
            }
            "no_physics" => {
                if let Some(p) = game.creatures.projectiles.iter().find(|p| p.id == id) {
                    return Some(json!(p.arrow.as_ref().is_some_and(|arrow| arrow.no_physics)));
                }
                if let Some(trident) = game.tridents.iter().find(|trident| trident.id == id) {
                    return Some(json!(trident.no_physics));
                }
                json!(game.mobs.iter().any(|m| m.id == id && (m.species() == Species::Vex || m.extra.plugin_flag("no_physics") == Some(true))))
            }
            "set_no_physics" => {
                let on = truth(&query["value"]);
                if let Some(arrow) = game.creatures.projectiles.iter_mut().find(|p| p.id == id).and_then(|p| p.arrow.as_mut()) {
                    arrow.no_physics = on;
                    return Some(json!(true));
                }
                if let Some(trident) = game.tridents.iter_mut().find(|trident| trident.id == id) {
                    trident.no_physics = on;
                    return Some(json!(true));
                }
                json!(false)
            }
            "shared_flag" => {
                horizon_client::paper_hub::set_shared_flag(id as i32, query["bit"].as_i64()? as i8, truth(&query["value"]));
                json!(true)
            }
            "event" => {
                let at = game.target_position(target?)?;
                game.lan_entity_event(id as i32, query["event"].as_i64()? as i8, at);
                json!(true)
            }
            "look" => {
                let [x, y, z] = vector(&query["at"])?;
                let eyes = truth(&query["eyes"]);
                let m = game.mobs.iter_mut().find(|m| m.id == id)?;
                let from_y = if eyes { m.eye_y() } else { m.position.y };
                let (dx, dy, dz) = (x - m.position.x, y - from_y, z - m.position.z);
                let yaw = (dz.atan2(dx).to_degrees() - 90.0) as f32;
                let pitch = (-dy.atan2(dx.hypot(dz)).to_degrees()) as f32;
                m.yaw = yaw;
                m.head_yaw = yaw;
                m.body_yaw = yaw;
                m.head_pitch = pitch;
                m.needs_sync = true;
                json!(true)
            }
            "collides" => {
                let (min, max) = (vector(&query["min"])?, vector(&query["max"])?);
                let area = horizon_core::geometry::Aabb::new(Vec3::new(min[0], min[1], min[2]), Vec3::new(max[0], max[1], max[2]));
                let others = Vec::new();
                // `Level.noCollision`: the blocks, and what `Entity.canBeCollidedWith` says stops a box.
                let solid = game.boats.iter().filter(|boat| boat.id != id).any(|boat| boat.bounds().intersects(area))
                    || game.mobs.iter().any(|m| m.id != id && m.alive() && (m.species() == Species::Shulker || (m.species() == Species::HappyGhast && !m.is_baby())) && m.bounds().intersects(area));
                json!(solid || !game.mob_world_base(&others).no_collision(area))
            }
            "rain" => {
                let at = game.target_position(target?)?;
                let others = Vec::new();
                let world = game.mob_world_base(&others);
                json!(world.raining_at(at[0].floor() as i32, at[1].floor() as i32, at[2].floor() as i32))
            }
            "tracked_by" => {
                let hub = &game.lan.as_ref()?.hub;
                json!(hub.guests.iter().filter(|guest| guest.tracker.is_tracking(id as i32)).map(|guest| format!("{:032x}", guest.uuid)).collect::<Vec<_>>())
            }
            "pick" => {
                let stack = game.entity_pick_result(id as i32);
                stack.map_or(Value::Null, |stack| super::commands::paper_entities::tag_json(&horizon_client::save::item_to_nbt(&stack)))
            }
            "swing" => {
                let off = truth(&query["off"]);
                let m = game.mobs.iter_mut().find(|m| m.id == id)?;
                m.swing_hand(off, 6);
                let at = [m.position.x, m.position.y, m.position.z];
                game.lan.as_ref()?.hub.broadcast_near(at, 128.0, &C::Animate(play::Animate { id: id as i32, action: if off { 3 } else { 0 } }), None);
                json!(true)
            }
            "hurt_animation" => {
                let at = game.target_position(target?)?;
                let yaw = query["yaw"].as_f64()? as f32;
                game.lan.as_ref()?.hub.broadcast_near(at, 128.0, &C::HurtAnimation(play::HurtAnimation { id: id as i32, yaw }), None);
                json!(true)
            }
            "take_item" => {
                let at = game.target_position(target?)?;
                let packet = play::TakeItemEntity { item: query["item"].as_i64()? as i32, player: id as i32, amount: int(&query["amount"]) };
                game.lan.as_ref()?.hub.broadcast_near(at, 128.0, &C::TakeItemEntity(packet), None);
                json!(true)
            }
            _ => return None,
        })
    })
    .flatten()
}

/// `{ "op": "entity_unseen", "pairs": [[viewer, entity id]...] }`: who the tracker keeps what from.
fn unseen(game: &mut Game<'_>, query: &Value) -> bool {
    let Some(host) = game.lan.as_mut() else {
        return false;
    };
    let pair = |row: &Value| {
        let uuid = u128::from_str_radix(row[0].as_str()?, 16).ok()?;
        Some((host.hub.guests.iter().find(|guest| guest.uuid == uuid)?.entity_id, row[1].as_i64()? as i32))
    };
    let pairs: Vec<(i32, i32)> = query["pairs"].as_array().into_iter().flatten().filter_map(pair).collect();
    horizon_client::paper_hub::set_unseen_ids(pairs);
    for guest in &mut host.hub.guests {
        guest.moved = true;
    }
    true
}

fn target_number(target: Target) -> Option<u32> {
    match target {
        Target::Player | Target::Guest(_) => None,
        Target::Mob(id) | Target::Item(id) | Target::Orb(id) | Target::Projectile(id) | Target::Stand(id) | Target::Frame(id) | Target::Painting(id) | Target::Boat(id) | Target::Cart(id) | Target::Other(id) | Target::Crystal(id) | Target::Dragon(id) => Some(id),
    }
}

/// What the game keeps outside `level_entity`: a primed TNT, a falling block, a rocket, a bolt, fangs.
/// A plugin reads them by the same shape as the `entity` op.
fn kept(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let id = id_of(query);
    let wants_data = query["data"].as_bool().unwrap_or(true);
    game.native_in_level(&dimension(query), |game| {
        let place = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
        let answer = |kind: &str, at: [f64; 3], yaw: f32, data: Value| json!({ "kind": kind, "dimension": place, "pos": at, "yaw": yaw, "pitch": 0.0, "id": id, "uuid": "", "data": data });
        if let Some(tnt) = game.tnts.iter().find(|tnt| tnt.id == id) {
            let (at, v) = (tnt.position, tnt.velocity);
            let block = game.persist_raw(id).and_then(|tag| tag.get("block_state")).map(super::commands::paper_entities::tag_json);
            return Some(answer("minecraft:tnt", [at.x, at.y, at.z], 0.0, json!({ "fuse": tnt.fuse, "OnGround": tnt.on_ground, "Motion": [v.x, v.y, v.z], "block_state": block })));
        }
        if let Some(index) = game.falling.iter().position(|falling| falling.id == id) {
            let falling = &game.falling[index];
            let (at, velocity, time, drop_item, damage, fall, state) = (falling.position, falling.velocity, falling.age, falling.drop_item, falling.damage, falling.fall_distance, falling.state);
            let data = if wants_data {
                let block = game.state_to_nbt(state);
                let mut tag = horizon_client::entity_nbt::falling_block_to_nbt(block, at, velocity, time, drop_item, damage, fall, [0; 4]);
                if let (Tag::Compound(saved), Some(tile)) = (&mut tag, falling_tile_tag(game, id, game.falling[index].saved.as_deref().map(|saved| &saved.1))) {
                    saved.insert("TileEntityData".into(), tile);
                }
                super::commands::paper_entities::tag_json(&tag)
            } else {
                Value::Null
            };
            return Some(answer("minecraft:falling_block", [at.x, at.y, at.z], 0.0, data));
        }
        if let Some(rocket) = game.rockets.iter().find(|rocket| rocket.id == id) {
            let (at, v) = (rocket.position, rocket.velocity);
            let item = horizon_client::save::item_to_nbt(&rocket.item);
            return Some(answer("minecraft:firework_rocket", [at.x, at.y, at.z], 0.0, json!({ "ShotAtAngle": rocket.angled, "Motion": [v.x, v.y, v.z], "FireworksItem": super::commands::paper_entities::tag_json(&item), "Life": rocket.life, "LifeTime": rocket.lifetime })));
        }
        if let Some(bolt) = game.bolts.iter().find(|bolt| bolt.id == id) {
            return Some(answer("minecraft:lightning_bolt", bolt.at, 0.0, json!({ "Life": bolt.life, "Flashes": bolt.flashes, "Effect": bolt.visual_only })));
        }
        if let Some(fangs) = game.fangs.list.iter().find(|fangs| fangs.id == id) {
            return Some(answer("minecraft:evoker_fangs", fangs.pos, fangs.yaw, json!({})));
        }
        if let Some(cloud) = game.breath_clouds.iter().find(|cloud| cloud.id == id) {
            return Some(answer("minecraft:area_effect_cloud", cloud.at, 0.0, super::commands::paper_entities::tag_json(&cloud_tag(cloud))));
        }
        if let Some(potion) = game.potions.thrown.iter().find(|potion| potion.id == id) {
            let kind = if &*potion.stack.item == "minecraft:lingering_potion" { "minecraft:lingering_potion" } else { "minecraft:splash_potion" };
            let item = super::commands::paper_entities::tag_json(&horizon_client::save::item_to_nbt(&potion.stack));
            return Some(answer(kind, [0.0; 3], 0.0, json!({ "Item": item })));
        }
        if let Some((at, _)) = game.knot_ids.iter().find(|(_, knot)| **knot == id) {
            return Some(answer("minecraft:leash_knot", at.map(|c| f64::from(c) + 0.5), 0.0, json!({})));
        }
        if let Some(eye) = game.end.eyes.iter().find(|eye| eye.id == id) {
            let item = eye.item.as_ref().map(|stack| super::commands::paper_entities::tag_json(&horizon_client::save::item_to_nbt(stack)));
            return Some(answer("minecraft:eye_of_ender", eye.position.to_array(), 0.0, json!({ "Item": item, "Survive": eye.survive, "Life": eye.life, "Target": eye.target.to_array() })));
        }
        None
    })
    .flatten()
}

/// `merge` for those: the tags each one's fields stand for.
fn kept_merge(game: &mut Game<'_>, id: u32, patch: &Tag) -> bool {
    use horizon_client::entity_nbt::motion;
    if let Some(index) = game.breath_clouds.iter().position(|cloud| cloud.id == id) {
        let cloud = game.breath_clouds.remove(index);
        let tag = merged(&cloud_tag(&cloud), patch);
        let [x, y, z] = cloud.at;
        game.others.push(super::commands::OtherEntity { id, kind: "minecraft:area_effect_cloud".to_owned(), position: Vec3::new(x, y, z), yaw: 0.0, pitch: 0.0 });
        game.persist.raw.insert(id, tag);
        return true;
    }
    let number = |key: &str| match patch.get(key) {
        Some(Tag::Byte(v)) => Some(i64::from(*v)),
        Some(Tag::Short(v)) => Some(i64::from(*v)),
        Some(Tag::Int(v)) => Some(i64::from(*v)),
        Some(Tag::Long(v)) => Some(*v),
        _ => None,
    };
    if let Some(tnt) = game.tnts.iter_mut().find(|tnt| tnt.id == id) {
        if let Some(fuse) = number("fuse") {
            tnt.fuse = fuse as i32;
        }
        if patch.get("Motion").is_some() {
            tnt.velocity = motion(patch);
        }
        if let Some(block) = patch.get("block_state")
            && let Tag::Compound(saved) = game.persist.raw.entry(id).or_insert_with(|| Tag::Compound(Default::default()))
        {
            saved.insert("block_state".into(), block.clone());
        }
        return true;
    }
    if let Some(potion) = game.potions.thrown.iter_mut().find(|potion| potion.id == id) {
        if let Some(item) = patch.get("Item").and_then(horizon_client::save::item_from_nbt) {
            potion.stack = item;
        }
        return true;
    }
    if let Some(index) = game.falling.iter().position(|falling| falling.id == id) {
        let anvil = game.res.facts().get(game.falling[index].state).name.ends_with("anvil");
        let falling = &mut game.falling[index];
        if let Some(drop_item) = number("DropItem") {
            falling.drop_item = drop_item != 0;
        }
        if patch.get("HurtEntities").is_some() || patch.get("FallHurtAmount").is_some() || patch.get("FallHurtMax").is_some() || patch.get("CancelDrop").is_some() {
            let mut whole = Tag::compound(Vec::<(&str, Tag)>::new());
            falling.damage.save(&mut whole);
            let whole = merged(&whole, patch);
            falling.damage = horizon_client::falling_block::Damage::from_nbt(&whole, anvil);
        }
        if patch.get("Motion").is_some() {
            falling.velocity = motion(patch);
        }
        if let Some(block) = patch.get("BlockState").and_then(|state| game.state_from_nbt(state)) {
            game.falling[index].state = block;
        }
        if let Some(data) = patch.get("TileEntityData")
            && let Tag::Compound(saved) = game.persist.raw.entry(id).or_insert_with(|| Tag::Compound(Default::default()))
        {
            saved.insert("TileEntityData".into(), data.clone());
        }
        return true;
    }
    if let Some(rocket) = game.rockets.iter_mut().find(|rocket| rocket.id == id) {
        if let Some(angled) = number("ShotAtAngle") {
            rocket.angled = angled != 0;
        }
        if let Some(life) = number("Life") {
            rocket.life = life as i32;
        }
        if let Some(lifetime) = number("LifeTime") {
            rocket.lifetime = lifetime as i32;
        }
        if let Some(item) = patch.get("FireworksItem").and_then(horizon_client::save::item_from_nbt) {
            let (_, explosions) = super::fireworks::fireworks_of(&item);
            rocket.explosions = explosions;
            rocket.item = item;
        }
        if patch.get("Motion").is_some() {
            rocket.velocity = motion(patch);
        }
        return true;
    }
    if let Some(bolt) = game.bolts.iter_mut().find(|bolt| bolt.id == id) {
        if let Some(life) = number("Life") {
            bolt.life = life as i32;
        }
        if let Some(flashes) = number("Flashes") {
            bolt.flashes = flashes as i32;
        }
        return true;
    }
    false
}

/// The entities outside `level_entity`, as `entities` lists the rest.
fn kept_list(game: &mut Game<'_>, query: &Value) -> Value {
    let ids = game.native_in_level(&dimension(query), |game| {
        let mut ids: Vec<u32> = game.tnts.iter().map(|tnt| tnt.id).collect();
        ids.extend(game.falling.iter().map(|falling| falling.id));
        ids.extend(game.rockets.iter().map(|rocket| rocket.id));
        ids.extend(game.bolts.iter().map(|bolt| bolt.id));
        ids.extend(game.fangs.list.iter().map(|fangs| fangs.id));
        ids.extend(game.end.eyes.iter().map(|eye| eye.id));
        ids.extend(game.potions.thrown.iter().map(|potion| potion.id));
        ids.extend(game.breath_clouds.iter().map(|cloud| cloud.id));
        ids.retain(|id| *id != 0);
        ids
    });
    ids.unwrap_or_default().into_iter().filter_map(|id| {
        let mut asked = query.clone();
        asked["id"] = json!(id);
        asked["data"] = json!(false);
        kept(game, &asked)
    }).collect()
}

/// What a plugin holds a mob to, past the game's own tick. `after_mobs` runs once the mobs have moved
/// and before anything is sent, so what it sets is what the clients see.
#[derive(Default)]
struct Held {
    /// An allay made to dance or to stop.
    dancers: HashMap<u32, bool>,
    /// A bee's rolling override.
    rollers: HashMap<u32, bool>,
    /// A creeper's own fuse: its length and how far it has burnt.
    fuses: HashMap<u32, (i32, i32)>,
    /// An armor stand that doesn't move, and where.
    stands: HashMap<u32, Vec3>,
    /// The bars plugins asked a boss for: its id, its level, and who it was showing to.
    boss_bars: HashMap<String, (u32, String, Vec<[i32; 4]>)>,
    boss_swept: bool,
    /// The game time each entity with no clock of its own was first seen at.
    born: HashMap<u32, i64>,
    /// `startRiptideAttack`'s item, by the player's entity id, while the spin lasts.
    spin_items: HashMap<i32, ItemStack>,
    /// What's been applied from each player's `horizon:plugin`, by entity id.
    applied: HashMap<i32, Tag>,
    /// What a mob's `getActiveItemUsedTime` has counted.
    used: HashMap<u32, i32>,
    /// `LivingEntity.getCollidableExemptions`: the mob, and the UUIDs in hex.
    exempt: HashMap<u32, Vec<String>>,
    /// The potion a plugin gave a witch, whole, while she drinks it.
    witches: HashMap<u32, ItemStack>,
    /// What a chunk took with it when it unloaded.
    unloaded: std::collections::HashSet<u32>,
    /// Mobs told to swim, until their next tick.
    swimming: std::collections::HashSet<u32>,
    /// Armor stands `setCanTick(false)` stopped, and where.
    stopped_stands: HashMap<u32, Vec3>,
    /// Who a plugin's area effect cloud has affected, and until when.
    clouds: HashMap<u32, Vec<(super::potions::Victim, i32)>>,
    /// A TNT block primed this tick, until it has its entity: where, and who.
    tnt_pending: Vec<([f64; 3], Option<Source>)>,
    /// A primed TNT's start and who lit it.
    tnt: HashMap<u32, ([f64; 3], Option<Source>)>,
    /// Who what's going off now counts as, for the TNT it lights.
    blast: Option<Source>,
    /// `Creeper.getIgniter`.
    igniters: HashMap<u32, Source>,
    /// Fangs placed this tick, until they're found in the list: where, how long and whose.
    fangs_pending: Vec<([f64; 3], i32, u32)>,
    /// Fangs: their owner, and the game time they rise at.
    fangs: HashMap<u32, (u32, i64)>,
}

static HELD: LazyLock<Mutex<Held>> = LazyLock::new(Default::default);

fn held() -> std::sync::MutexGuard<'static, Held> {
    HELD.lock().unwrap_or_else(|e| e.into_inner())
}

pub(in crate::game) fn after_mobs(game: &mut Game<'_>) -> bool {
    tick_clouds(game);
    tick_boss_bars(game);
    if let Some(host) = game.lan.as_mut() {
        let attribute = host.hub.registry_id("minecraft:attribute", "minecraft:max_health");
        horizon_client::paper_hub::note_max_health_attribute(attribute);
        for guest in &mut host.hub.guests {
            guest_plugin_apply(guest, attribute);
        }
    }
    witches_finish(game);
    let mut held = held();
    if !held.exempt.is_empty() {
        let players: Vec<(u32, [i32; 4])> = held.exempt.iter().flat_map(|(mob, uuids)| uuids.iter().filter_map(|uuid| Some((*mob, uuid_words(uuid)?)))).collect();
        for m in &mut game.mobs {
            m.extra.exempt_players.clear();
            m.extra.exempt_players.extend(players.iter().filter(|(mob, _)| *mob == m.id).map(|(_, uuid)| *uuid));
        }
        let pairs: Vec<(u32, u32)> = held.exempt.iter().flat_map(|(mob, uuids)| {
            let persist = &game.persist;
            uuids.iter().filter_map(|uuid| {
                let ints = uuid_words(uuid)?;
                persist.uuids.iter().find(|(_, known)| **known == ints).map(|(other, _)| (*mob, *other))
            })
        }).collect();
        for m in &mut game.mobs {
            m.extra.collide_exempt.clear();
            m.extra.collide_exempt.extend(pairs.iter().filter_map(|(a, b)| if *a == m.id { Some(*b) } else if *b == m.id { Some(*a) } else { None }));
        }
    }
    for m in &mut game.mobs {
        if m.extra.using_item > 0 {
            *held.used.entry(m.id).or_default() += 1;
        } else if !held.used.is_empty() {
            held.used.remove(&m.id);
        }
        if let Some(dancing) = held.dancers.get(&m.id).copied() {
            if !dancing && !m.extra.flag {
                held.dancers.remove(&m.id);
            }
            m.extra.flag = dancing;
        }
        if let Some(rolling) = held.rollers.get(&m.id).copied()
            && let Some(bee) = m.extra.bee.as_deref_mut()
        {
            bee.flags = if rolling { bee.flags | 2 } else { bee.flags & !2 };
        }
        if let Some((length, burnt)) = held.fuses.get_mut(&m.id) {
            if m.swell > 0 || m.extra.flag {
                *burnt += 1;
                m.swell = (*burnt * 30 / (*length).max(1)).min(29);
            } else {
                *burnt = 0;
            }
        }
    }
    held.swimming.clear();
    for m in &mut game.mobs {
        if m.hurt_time == 10
            && let Some(ticks) = m.extra.plugin_number("no_damage")
        {
            m.invulnerable = ticks as i32;
        }
    }
    let now = game.world.game_time;
    note_born(game, &mut held, now);
    let back: Vec<(u32, Tag)> = game.falling.iter()
        .filter(|falling| falling.id != 0 && !game.persist.raw.contains_key(&falling.id))
        .filter_map(|falling| Some((falling.id, falling.saved.as_ref()?.1.get(PLUGIN)?.clone())))
        .collect();
    for (id, plugin) in back {
        game.persist.raw.insert(id, Tag::compound([(PLUGIN, plugin)]));
    }
    if !held.igniters.is_empty() {
        let mobs = &game.mobs;
        held.igniters.retain(|id, _| mobs.iter().any(|m| m.id == *id));
    }
    for (at, source) in std::mem::take(&mut held.tnt_pending) {
        let found = game.tnts.iter().position(|tnt| (tnt.position.x - at[0]).abs() < 1.5 && (tnt.position.y - at[1]).abs() < 1.5 && (tnt.position.z - at[2]).abs() < 1.5 && !held.tnt.contains_key(&tnt.id));
        if let Some(index) = found {
            if game.tnts[index].id == 0 {
                game.next_entity += 1;
                game.tnts[index].id = game.next_entity;
            }
            held.tnt.insert(game.tnts[index].id, (at, source));
        }
    }
    for (at, delay, owner) in std::mem::take(&mut held.fangs_pending) {
        if let Some(fangs) = game.fangs.list.iter().find(|fangs| fangs.pos == at && !held.fangs.contains_key(&fangs.id)) {
            held.fangs.insert(fangs.id, (owner, now + i64::from(delay)));
        }
    }
    let (tnts, list) = (&game.tnts, &game.fangs.list);
    held.tnt.retain(|id, _| tnts.iter().any(|tnt| tnt.id == *id));
    held.fangs.retain(|id, _| list.iter().any(|fangs| fangs.id == *id));
    for stand in &mut game.armor_stands {
        if let Some(at) = held.stands.get(&stand.id).or_else(|| held.stopped_stands.get(&stand.id)) {
            stand.position = *at;
            stand.velocity = Vec3::ZERO;
        }
    }
    true
}

fn hold(game: &mut Game<'_>, key: &str, query: &Value) -> Option<Value> {
    let id = id_of(query);
    let value = &query["value"];
    // Read before the lock: going into a level can run anything.
    let stand_at = (key == "stand_tick" || (key == "stand" && !truth(value))).then(|| game.native_in_level(&dimension(query), |game| game.armor_stands.iter().find(|stand| stand.id == id).map(|stand| stand.position)).flatten());
    let mut held = held();
    Some(match key {
        "dance" => {
            held.dancers.insert(id, truth(value));
            json!(true)
        }
        "roll" => {
            match value.as_bool() {
                Some(rolling) => held.rollers.insert(id, rolling),
                None => held.rollers.remove(&id),
            };
            json!(true)
        }
        "roll_get" => held.rollers.get(&id).map_or(Value::Null, |rolling| json!(rolling)),
        "fuse" => {
            held.fuses.insert(id, (int(value).max(1), 0));
            json!(true)
        }
        "stand" => {
            if truth(value) {
                held.stands.remove(&id);
            } else {
                held.stands.insert(id, stand_at.flatten()?);
            }
            json!(true)
        }
        "stand_get" => json!(!held.stands.contains_key(&id)),
        "stand_tick" => {
            if truth(value) {
                held.stopped_stands.remove(&id);
            } else {
                held.stopped_stands.insert(id, stand_at.flatten()?);
            }
            json!(true)
        }
        "used" => json!(held.used.get(&id).copied().unwrap_or(0)),
        "exempt" => {
            held.exempt.insert(id, value.as_array()?.iter().filter_map(|uuid| uuid.as_str().map(str::to_owned)).collect());
            json!(true)
        }
        "exempt_get" => json!(held.exempt.get(&id).cloned().unwrap_or_default()),
        "igniter" => {
            match Source::read(value) {
                Some(source) => held.igniters.insert(id, source),
                None => held.igniters.remove(&id),
            };
            json!(true)
        }
        "igniter_get" => held.igniters.get(&id).copied().map_or(Value::Null, Source::json),
        "unloaded" => json!(held.unloaded.contains(&id)),
        _ => return None,
    })
}

pub(in crate::game) fn tnt_primed(game: &mut Game<'_>, pos: horizon_core::position::BlockPos) -> bool {
    let mut held = held();
    let source = held.blast.or(game.combat.explosion_source.map(Source::Mob)).or(game.lan_acting_uuid.map(Source::Player));
    held.tnt_pending.push(([f64::from(pos.x) + 0.5, f64::from(pos.y), f64::from(pos.z) + 0.5], source));
    true
}

/// A primed TNT goes off: what it lights is that one's source's.
pub(in crate::game) fn tnt_exploding(id: u32) {
    let mut held = held();
    held.blast = held.tnt.get(&id).and_then(|entry| entry.1);
}

pub(in crate::game) fn tnt_exploded() {
    held().blast = None;
}

/// A burning arrow lands: TNT it lights is its shooter's.
pub(in crate::game) fn arrow_landing(game: &Game<'_>, owner: Option<Who>, guest: Option<u32>, at: Vec3) {
    let source = match (guest, owner) {
        (Some(conn), _) => game.lan.as_ref().and_then(|host| host.hub.guest(conn)).map(|guest| Source::Player(guest.uuid)),
        (None, Some(Who::Mob(id))) => Some(Source::Mob(id)),
        (None, Some(Who::Player)) => bound_player(game, None, Some(at)).and_then(|uuid| u128::from_str_radix(&uuid, 16).ok()).map(Source::Player),
        (None, None) => None,
    };
    held().blast = source;
}

/// A flint and steel lights a creeper.
pub(in crate::game) fn creeper_lit(id: u32, acting: Option<u128>) {
    if let Some(uuid) = acting {
        held().igniters.insert(id, Source::Player(uuid));
    }
}

pub(in crate::game) fn fangs_placed(_game: &mut Game<'_>, at: [f64; 3], delay: i32, owner: u32) -> bool {
    held().fangs_pending.push((at, delay, owner));
    true
}

/// `FishHook`'s setters, under a player's turn: `game.hook` is that player's.
fn hook_set(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    use super::fishing::State;
    let what = query["what"].as_str()?;
    let value = &query["value"];
    let hook = game.hook.as_mut()?;
    let whole = |key: usize| value.get(key).and_then(Value::as_i64).map(|v| v as i32);
    let single = || value.as_i64().map(|v| v as i32);
    let angle = |key: usize| value.get(key).and_then(Value::as_f64).map(|v| v as f32);
    let rules = &mut hook.rules;
    match what {
        "min_wait" => {
            let v = single()?;
            if v < 0 || v > rules.wait.1 {
                return Some(json!(false));
            }
            rules.wait.0 = v;
        }
        "max_wait" => {
            let v = single()?;
            if v < 0 || v < rules.wait.0 {
                return Some(json!(false));
            }
            rules.wait.1 = v;
        }
        "wait" => {
            let (min, max) = (whole(0)?, whole(1)?);
            if min < 0 || max < min {
                return Some(json!(false));
            }
            rules.wait = (min, max);
        }
        "min_lure" => {
            let v = single()?;
            if v < 0 || v > rules.lure_time.1 {
                return Some(json!(false));
            }
            rules.lure_time.0 = v;
        }
        "max_lure" => {
            let v = single()?;
            if v < 0 || v < rules.lure_time.0 {
                return Some(json!(false));
            }
            rules.lure_time.1 = v;
        }
        "lure" => {
            let (min, max) = (whole(0)?, whole(1)?);
            if min < 0 || max < min {
                return Some(json!(false));
            }
            rules.lure_time = (min, max);
        }
        "min_angle" => {
            let v = value.as_f64()? as f32;
            if v > rules.lure_angle.1 {
                return Some(json!(false));
            }
            rules.lure_angle.0 = v;
        }
        "max_angle" => {
            let v = value.as_f64()? as f32;
            if v < rules.lure_angle.0 {
                return Some(json!(false));
            }
            rules.lure_angle.1 = v;
        }
        "angle" => {
            let (min, max) = (angle(0)?, angle(1)?);
            if min > max {
                return Some(json!(false));
            }
            rules.lure_angle = (min, max);
        }
        "apply_lure" => rules.apply_lure = truth(value),
        "sky" => rules.sky = truth(value),
        "rain" => rules.rain = truth(value),
        "bite_chance" => {
            let chance = value.as_f64()?;
            if !(0.0..=1.0).contains(&chance) {
                return Some(json!(false));
            }
            rules.bite_chance = Some(chance);
        }
        "wait_time" => hook.time_until_lured = single()?,
        "bite_time" => {
            let ticks = single()?;
            if ticks < 1 {
                return Some(json!(false));
            }
            hook.time_until_lured = 0;
            hook.time_until_hooked = ticks;
        }
        "reset" => {
            hook.time_until_lured = hook.rules.wait_ticks(&mut game.world.random, hook.lure);
            hook.time_until_hooked = 0;
        }
        "hooked" => hook.state = value.as_u64().map_or(State::Flying, |id| State::Hooked(id as u32)),
        _ => return None,
    }
    Some(json!(true))
}

/// More of what isn't a mob, past `object`.
fn object_more(game: &mut Game<'_>, key: &str, query: &Value) -> Option<Value> {
    let id = id_of(query);
    let value = &query["value"];
    game.native_in_level(&dimension(query), |game| {
        Some(match key {
            // `EyeOfEnder.signalTo`, from where it is now; without `update`, only the point it's heading for.
            "eye_target" => {
                let at = glam::DVec3::from_array(vector(value)?);
                let survive = game.world.random.next_int_bound(5).unwrap_or(0) > 0;
                let eye = game.end.eyes.iter_mut().find(|eye| eye.id == id)?;
                if truth(&query["update"]) {
                    let delta = at - eye.position;
                    let horizontal = delta.x.hypot(delta.z);
                    eye.target = if horizontal > 12.0 { eye.position + glam::DVec3::new(delta.x / horizontal * 12.0, 8.0, delta.z / horizontal * 12.0) } else { at };
                    eye.life = 0;
                    eye.survive = survive;
                } else {
                    eye.target = at;
                }
                json!(true)
            }
            "podium" => {
                let at = game.podium_top();
                json!([at.x, at.y, at.z])
            }
            "eye_drop" => {
                game.end.eyes.iter_mut().find(|eye| eye.id == id)?.survive = truth(value);
                json!(true)
            }
            "eye_life" => {
                game.end.eyes.iter_mut().find(|eye| eye.id == id)?.life = value.as_u64()? as u32;
                json!(true)
            }
            "eye_item" => {
                game.end.eyes.iter_mut().find(|eye| eye.id == id)?.item = stack(value);
                json!(true)
            }
            _ => return None,
        })
    })
    .flatten()
}

/// What a plugin set on an entity, past the game's own fields, lives in its saved compound under this tag: so it
/// goes with the entity through a save and a change of level. A mob has a copy the library reads.
const PLUGIN: &str = "horizon:plugin";

/// A number or a flag of `tag`, as `plugin_number` reads them, or a saved number of any width.
pub(in crate::game) fn tag_number(tag: &Tag) -> Option<f64> {
    Some(match tag {
        Tag::Byte(v) => f64::from(*v),
        Tag::Short(v) => f64::from(*v),
        Tag::Int(v) => f64::from(*v),
        Tag::Long(v) => *v as f64,
        Tag::Float(bits) => f64::from(f32::from_bits(*bits)),
        Tag::Double(bits) => f64::from_bits(*bits),
        _ => return None,
    })
}

/// `key` of what a plugin set, from an entity's compound.
pub(in crate::game) fn plugin_in(raw: Option<&Tag>, key: &str) -> Option<f64> {
    tag_number(raw?.get(PLUGIN)?.get(key)?)
}

pub(in crate::game) fn plugin_number(game: &Game<'_>, id: u32, key: &str) -> Option<f64> {
    plugin_in(game.persist_raw(id), key)
}

/// `Minecart.setDerailedVelocityMod` and `setFlyingVelocityMod`, if a plugin set either: the game's 0.5 and 0.95 otherwise.
pub(in crate::game) fn cart_mods(game: &Game<'_>, id: u32) -> Option<([f64; 3], [f64; 3])> {
    let plugin = game.persist_raw(id)?.get(PLUGIN)?;
    let mods = |key: &str, default: f64| -> Option<[f64; 3]> {
        let Some(Tag::List(rows)) = plugin.get(key) else {
            return Some([default; 3]);
        };
        Some([tag_number(rows.first()?)?, tag_number(rows.get(1)?)?, tag_number(rows.get(2)?)?])
    };
    (plugin.get("derailed_mod").is_some() || plugin.get("flying_mod").is_some()).then(|| Some((mods("derailed_mod", 0.5)?, mods("flying_mod", 0.95)?))).flatten()
}

fn plugin_tag(value: &Value) -> Option<Tag> {
    Some(match value {
        Value::Bool(on) => Tag::Double(f64::from(u8::from(*on)).to_bits()),
        Value::Number(number) => Tag::Double(number.as_f64()?.to_bits()),
        Value::String(text) => Tag::String(text.as_str().into()),
        Value::Array(rows) => Tag::List(rows.iter().filter_map(plugin_tag).collect()),
        _ => return None,
    })
}

fn plugin_json(tag: &Tag) -> Value {
    match tag {
        Tag::IntArray(rows) => json!(rows),
        Tag::List(rows) => Value::Array(rows.iter().map(plugin_json).collect()),
        Tag::String(text) => json!(String::from_utf16_lossy(&text.0)),
        tag => tag_number(tag).map_or(Value::Null, |number| json!(number)),
    }
}

/// `{ "op": "entity_plugin_set", "key": ..., "value": ... }`: `null` takes it away.
fn plugin_set(game: &mut Game<'_>, key: &str, query: &Value) -> Option<bool> {
    let id = id_of(query);
    let value = plugin_tag(&query["value"]);
    game.native_in_level(&dimension(query), |game| {
        let Tag::Compound(saved) = game.persist.raw.entry(id).or_insert_with(|| Tag::Compound(Default::default())) else {
            return false;
        };
        if let Tag::Compound(values) = saved.entry(PLUGIN.into()).or_insert_with(|| Tag::Compound(Default::default())) {
            match value {
                Some(value) => {
                    values.insert(key.into(), value);
                }
                None => {
                    values.remove(&key.into());
                }
            }
        }
        let copy = saved.get(&PLUGIN.into()).cloned();
        if let Some(mob) = game.mobs.iter_mut().find(|m| m.id == id) {
            mob.extra.plugin = copy.map(Box::new);
        }
        if game.dropped.iter().any(|item| item.id == id) {
            let flag = |key: &str| plugin_number(game, id, key) == Some(0.0);
            horizon_client::paper_hub::plugin_mark(0, id, flag("mob_pickup"));
            horizon_client::paper_hub::plugin_mark(1, id, flag("friction"));
        }
        true
    })
}

fn uuid_words(hex: &str) -> Option<[i32; 4]> {
    let word = |i: usize| u32::from_str_radix(hex.get(i * 8..i * 8 + 8)?, 16).ok().map(|word| word as i32);
    Some([word(0)?, word(1)?, word(2)?, word(3)?])
}

/// What the orbs about to spawn are for: `ExperienceOrb.getSpawnReason`, the entity they came from, and the player.
static ORB_CAUSE: Mutex<Option<(&'static str, Option<[i32; 4]>, Option<u128>)>> = Mutex::new(None);

/// `source` is the entity they came from; an entity with no saved UUID has the one its trackers see.
pub(in crate::game) fn orb_cause(game: &Game<'_>, reason: &'static str, source: Option<u32>, trigger: Option<u128>) {
    let source = source.map(|id| game.persist.uuids.get(&id).copied().unwrap_or_else(|| horizon_client::lan::playerdata::uuid_ints(super::lan::entity_uuid(id))));
    *ORB_CAUSE.lock().unwrap_or_else(|e| e.into_inner()) = Some((reason, source, trigger));
}

/// The parent of the baby about to come, for its orb: who fed it, if a player did.
static BREEDER: Mutex<Option<Option<[i32; 4]>>> = Mutex::new(None);

pub(in crate::game) fn breeding_is(parent: &Mob) {
    *BREEDER.lock().unwrap_or_else(|e| e.into_inner()) = Some(parent.extra.love_cause);
}

/// `Animal.finalizeSpawnChildFromBreeding`: the baby just made is the source, and the player who fed the parent the trigger.
pub(in crate::game) fn orb_breed(game: &Game<'_>) {
    let breeder = BREEDER.lock().unwrap_or_else(|e| e.into_inner()).take();
    let baby = breeder.map(|_| game.next_entity);
    orb_cause(game, "BREED", baby, breeder.flatten().map(horizon_client::lan::playerdata::ints_uuid));
}

/// The player a dying mob's last hit came from.
pub(in crate::game) fn killer_uuid(game: &Game<'_>, m: &Mob) -> Option<u128> {
    let index = game.mobs.iter().position(|known| known.id == m.id)?;
    u128::from_str_radix(&killer(game, index)?, 16).ok()
}

pub(in crate::game) fn spin_item(game: &Game<'_>) -> Option<ItemStack> {
    let entity = game.player.as_ref()?.entity_id;
    held().spin_items.get(&entity).cloned()
}

/// The item as the hit left it, for the next hit of the spin.
pub(in crate::game) fn spin_item_back(game: &Game<'_>, item: Option<ItemStack>) {
    let Some(entity) = game.player.as_ref().map(|player| player.entity_id) else {
        return;
    };
    let mut held = held();
    match item {
        Some(item) => {
            held.spin_items.insert(entity, item);
        }
        None => {
            held.spin_items.remove(&entity);
        }
    }
}

pub(in crate::game) fn spin_over(entity: i32) {
    held().spin_items.remove(&entity);
}

/// The players with a simulation distance of their own, as tickets want them.
pub(in crate::game) fn own_simulation(game: &Game<'_>, spectators_load: bool) -> Vec<((i32, i32), i32)> {
    let Some(host) = game.lan.as_ref() else {
        return Vec::new();
    };
    host.hub.guests.iter().filter(|guest| host.hub.here(guest) && super::game_mode::loads_chunks(spectators_load, guest.spectator())).filter_map(|guest| Some((guest.chunk(), horizon_client::paper_hub::simulation_of(guest.entity_id)?))).collect()
}

fn guest_plugin_set(guest: &mut horizon_client::lan::guest::Guest, key: &str, value: Option<Tag>) {
    let Tag::Compound(saved) = guest.saved.get_or_insert_with(|| Tag::Compound(Default::default())) else {
        return;
    };
    if let Tag::Compound(values) = saved.entry(PLUGIN.into()).or_insert_with(|| Tag::Compound(Default::default())) {
        match value {
            Some(value) => {
                values.insert(key.into(), value);
            }
            None => {
                values.remove(&key.into());
            }
        }
    }
}

/// What a player's file says a plugin set: into the places the game reads it, once a change. The food rates live
/// on the body, which a respawn replaces, so they go each time.
fn guest_plugin_apply(guest: &mut horizon_client::lan::guest::Guest, max_health: Option<i32>) {
    use horizon_client::paper_hub as hub;
    let Some(plugin) = guest.saved.as_ref().and_then(|saved| saved.get(PLUGIN)).cloned() else {
        return;
    };
    let number = |key: &str| plugin.get(key).and_then(tag_number);
    if let Some(Tag::List(rates)) = plugin.get("food_rates")
        && let Some(motion) = guest.motion.as_mut()
    {
        for (rate, row) in motion.health.food.rates.iter_mut().zip(rates) {
            *rate = tag_number(row).unwrap_or(f64::from(*rate)) as i32;
        }
    }
    let entity = guest.entity_id;
    if let Some(motion) = guest.motion.as_mut() {
        motion.freeze_locked = number("freeze_locked") == Some(1.0);
    }
    let mut held = held();
    if held.applied.get(&entity) == Some(&plugin) {
        return;
    }
    let scale = (number("health_scaled") == Some(1.0)).then(|| number("health_scale").unwrap_or(20.0));
    hub::set_health_scale(entity, scale);
    if let (Some(scale), Some(attribute)) = (scale, max_health) {
        guest.sent_health = None;
        guest.send(&C::UpdateAttributes(play::UpdateAttributes { id: entity, values: vec![play::AttributeSnapshot { attribute, base: scale, modifiers: Vec::new() }] }));
    }
    hub::plugin_mark(2, entity as u32, number("flying_fall_damage") == Some(1.0));
    hub::plugin_mark(3, entity as u32, number("affects_spawning") == Some(0.0));
    hub::plugin_mark(5 - 1, entity as u32, number("own_border") == Some(1.0));
    hub::set_no_collide(horizon_client::lan::playerdata::uuid_ints(guest.uuid), number("collidable") == Some(0.0));
    if let Some(ticks) = number("max_air") {
        hub::set_max_air(entity, ticks as i32);
    }
    if let Some(distance) = number("simulation_distance") {
        hub::set_simulation(entity, distance as i32);
        guest.send(&C::SetSimulationDistance(play::Distance { distance: distance as i32 }));
    }
    held.applied.insert(entity, plugin);
}

pub(in crate::game) fn orb_cause_over() {
    *ORB_CAUSE.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

pub(in crate::game) fn orb_born(game: &mut Game<'_>, id: u32) {
    let Some((reason, source, trigger)) = *ORB_CAUSE.lock().unwrap_or_else(|e| e.into_inner()) else {
        return;
    };
    let mut values = vec![("spawn_reason", Tag::String(reason.into()))];
    if let Some(source) = source {
        values.push(("source", Tag::IntArray(source.to_vec())));
    }
    if let Some(trigger) = trigger {
        values.push(("trigger", Tag::IntArray(horizon_client::lan::playerdata::uuid_ints(trigger).to_vec())));
    }
    let Tag::Compound(saved) = game.persist.raw.entry(id).or_insert_with(|| Tag::Compound(Default::default())) else {
        return;
    };
    saved.insert(PLUGIN.into(), Tag::compound(values));
}

/// `Interaction.skipAttackInteraction` and `interact`: who touched it last, and at what game time.
pub(in crate::game) fn interaction_touched(game: &mut Game<'_>, host: &crate::game::lan::LanHost, conn: u32, id: i32, key: &str) {
    let id = id as u32;
    if !game.others.iter().any(|other| other.id == id && other.kind == "minecraft:interaction") {
        return;
    }
    let Some(uuid) = host.hub.guest(conn).map(|guest| guest.uuid) else {
        return;
    };
    let now = game.world.game_time;
    let Tag::Compound(saved) = game.persist.raw.entry(id).or_insert_with(|| Tag::Compound(Default::default())) else {
        return;
    };
    saved.insert(key.into(), Tag::compound([("player", Tag::IntArray(horizon_client::lan::playerdata::uuid_ints(uuid).to_vec())), ("timestamp", Tag::Long(now))]));
}

/// A server-side `Interaction` (kept in `others`, so the game's own target lookup doesn't see it): its box
/// from the saved `width` and `height`, as `Interaction.getDimensions` (1 x 1 by default), feet-centred.
pub(in crate::game) fn interaction_box(game: &Game<'_>, id: i32) -> Option<horizon_core::geometry::Aabb> {
    let id = u32::try_from(id).ok()?;
    let other = game.others.iter().find(|other| other.id == id && other.kind == "minecraft:interaction")?;
    let saved = game.persist_raw(id);
    let size = |key: &str| saved.and_then(|tag| tag.get(key)).and_then(tag_number).unwrap_or(1.0);
    let (half, height, at) = (size("width") / 2.0, size("height"), other.position);
    Some(horizon_core::geometry::Aabb::new(Vec3::new(at.x - half, at.y, at.z - half), Vec3::new(at.x + half, at.y + height, at.z + half)))
}

pub(in crate::game) fn podium(game: &Game<'_>) -> Option<Vec3> {
    let id = game.dragon.as_ref()?.id;
    let Some(Tag::List(at)) = game.persist_raw(id)?.get(PLUGIN)?.get("podium") else {
        return None;
    };
    Some(Vec3::new(tag_number(at.first()?)?, tag_number(at.get(1)?)?, tag_number(at.get(2)?)?))
}

fn cloud_number(tag: &Tag, key: &str, default: f64) -> f64 {
    tag.get(key).and_then(tag_number).unwrap_or(default)
}

/// The radius, whether it's still waiting, and the colour of a plugin's cloud, for `lan_object_states`.
pub(in crate::game) fn cloud_look(tag: &Tag) -> (f32, bool, horizon_client::particles::ParticleOptions) {
    use horizon_client::particles::{ParticleOptions, ParticleType};
    let text = |tag: Option<&Tag>| if let Some(Tag::String(id)) = tag { Some(String::from_utf16_lossy(&id.0)) } else { None };
    let own = text(tag.get("custom_particle").and_then(|particle| particle.get("type"))).and_then(|name| ParticleType::from_name(name.trim_start_matches("minecraft:")).or_else(|| ParticleType::from_name(&name)));
    let options = own.map(ParticleOptions::simple).unwrap_or_else(|| {
        let potion = tag.get("potion_contents");
        let color = potion.and_then(|contents| contents.get("custom_color")).and_then(tag_number).filter(|color| *color >= 0.0).map_or_else(|| horizon_client::brewing::potion_color(text(potion.and_then(|contents| contents.get("potion"))).as_deref()), |color| color as i64 as u32);
        let c = |shift: u32| (color >> shift & 255) as f32 / 255.0;
        ParticleOptions::colored(ParticleType::EntityEffect, [c(16), c(8), c(0), 1.0])
    });
    (cloud_number(tag, "Radius", 3.0) as f32, cloud_number(tag, "Age", 0.0) < cloud_number(tag, "WaitTime", 20.0), options)
}

/// What a plugin's cloud puts on those in it: the potion's effects and its own.
fn cloud_effects(tag: &Tag) -> Vec<(&'static str, i32, i32)> {
    let Some(contents) = tag.get("potion_contents") else {
        return Vec::new();
    };
    let text = |tag: Option<&Tag>| if let Some(Tag::String(id)) = tag { Some(String::from_utf16_lossy(&id.0)) } else { None };
    let mut effects: Vec<(&'static str, i32, i32)> = text(contents.get("potion")).and_then(|id| horizon_client::brewing::potion(&id)).map_or_else(Vec::new, |potion| potion.effects.to_vec());
    if let Some(Tag::List(rows)) = contents.get("custom_effects") {
        for row in rows {
            let Some(id) = text(row.get("id")).and_then(|id| super::potions::mob_effect_id(&id)) else {
                continue;
            };
            effects.push((id, cloud_number(row, "duration", 1.0) as i32, cloud_number(row, "amplifier", 0.0) as i32));
        }
    }
    effects
}

/// `AreaEffectCloud.serverTick` for a cloud a plugin spawned: the game simulates those of its lingering potions and
/// its dragon, and keeps any other as a compound. Here that compound is what the cloud is, so a setter's change counts.
fn tick_clouds(game: &mut Game<'_>) {
    use super::potions::Victim;
    let ids: Vec<u32> = game.others.iter().filter(|other| other.kind == "minecraft:area_effect_cloud").map(|other| other.id).collect();
    for id in ids {
        let Some(tag) = game.persist.raw.get(&id).cloned() else {
            continue;
        };
        let from_dragon = plugin_in(Some(&tag), "from_dragon") == Some(1.0);
        cloud_options(game, id, &tag);
        let age = cloud_number(&tag, "Age", 0.0) as i32 + 1;
        let wait = cloud_number(&tag, "WaitTime", 20.0) as i32;
        let duration = cloud_number(&tag, "Duration", 600.0) as i32;
        let mut radius = cloud_number(&tag, "Radius", 3.0) as f32;
        let mut new_duration = duration;
        let mut gone = age - wait >= duration || (from_dragon && !game.dragon_flaming());
        if !gone && age >= wait {
            radius += cloud_number(&tag, "RadiusPerTick", 0.0) as f32;
            gone = radius < 0.5;
        }
        if !gone && age >= wait && age % 5 == 0 {
            let effects: Vec<_> = cloud_effects(&tag);
            // `potion_duration_scale`, with `cloud_affect`'s own quarter taken out.
            let scale = cloud_number(&tag, "potion_duration_scale", 1.0) * 4.0;
            let effects: Vec<_> = effects.into_iter().map(|(effect, ticks, amplifier)| (effect, (f64::from(ticks) * scale) as i32, amplifier)).collect();
            let at = game.others.iter().find(|other| other.id == id).map_or([0.0; 3], |other| [other.position.x, other.position.y, other.position.z]);
            let r = f64::from(radius);
            let area = horizon_core::geometry::Aabb::new(Vec3::new(at[0] - r, at[1], at[2] - r), Vec3::new(at[0] + r, at[1] + 0.5, at[2] + r));
            let mut victims = held().clouds.remove(&id).unwrap_or_default();
            victims.retain(|(_, until)| age < *until);
            let mut inside: Vec<(Victim, f64, f64)> = game.fight_players().into_iter().filter(|p| p.alive && p.bounds.intersects(area)).map(|p| (Victim::Player(p.who), p.position.x, p.position.z)).collect();
            let tags = &game.stations().enchantments.entity_tags;
            inside.extend(game.mobs.iter().filter(|m| m.alive() && m.bounds().intersects(area) && effects.iter().any(|(effect, ..)| super::potions::mob_can_be_affected(tags, &m.kind.id, effect))).map(|m| (Victim::Mob(m.id), m.position.x, m.position.z)));
            let delay = cloud_number(&tag, "ReapplicationDelay", 20.0) as i32;
            let on_use = cloud_number(&tag, "RadiusOnUse", 0.0) as f32;
            let duration_on_use = cloud_number(&tag, "DurationOnUse", 0.0) as i32;
            let source = horizon_client::text::Text::translate("entity.minecraft.area_effect_cloud", &[]);
            for (victim, x, z) in inside {
                let (dx, dz) = (x - at[0], z - at[2]);
                if effects.is_empty() || victims.iter().any(|(known, _)| *known == victim) || dx * dx + dz * dz > r * r {
                    continue;
                }
                victims.push((victim, age + delay));
                game.cloud_affect(victim, &effects, (&[], &[]), at, &source);
                radius += on_use;
                new_duration += duration_on_use;
                if radius < 0.5 || new_duration <= 0 {
                    gone = true;
                    break;
                }
            }
            held().clouds.insert(id, victims);
        }
        if gone {
            game.others.retain(|other| other.id != id);
            game.persist.raw.remove(&id);
            held().clouds.remove(&id);
            continue;
        }
        if let Some(Tag::Compound(saved)) = game.persist.raw.get_mut(&id) {
            saved.insert("Age".into(), Tag::Int(age));
            saved.insert("Radius".into(), Tag::Float(radius.to_bits()));
            saved.insert("Duration".into(), Tag::Int(new_duration));
        }
    }
}

/// `io.papermc.paper.entity.poi.PoiType`: whether a block state is that kind of point, and whether it can be occupied.
fn poi(game: &Game<'_>, query: &Value) -> Value {
    let kind = query["kind"].as_str().unwrap_or_default();
    let registry = game.pack.assets.registry;
    let mut tickets = None;
    let mut matches = false;
    let wanted = query["state"].as_str().and_then(|snbt| state_of_snbt(game, snbt));
    for state in 0..registry.len() as u32 {
        let id = horizon_world::registry::BlockStateId(state);
        if let Some((name, max)) = registry.state(id).and_then(horizon_client::save::poi_type)
            && name == kind
        {
            tickets = Some(max);
            matches |= wanted == Some(state);
        }
    }
    json!({ "known": tickets.is_some(), "occupants": tickets.is_some_and(|max| max > 0), "matches": matches })
}

/// `EntityType.getDefaultAttributes`: the bases a new mob of the kind starts with.
fn default_attributes(game: &Game<'_>, query: &Value) -> Value {
    let Some(kind) = query["kind"].as_str().and_then(|kind| game.res.mob_types().get(kind)) else {
        return Value::Null;
    };
    let mut rows: serde_json::Map<String, Value> = kind.attributes.iter().map(|(id, value)| (id.clone(), json!(value))).collect();
    for (id, value) in [
        ("minecraft:max_health", f64::from(kind.max_health)),
        ("minecraft:movement_speed", kind.speed),
        ("minecraft:flying_speed", kind.flying_speed),
        ("minecraft:attack_damage", f64::from(kind.attack_damage)),
        ("minecraft:attack_knockback", kind.attack_knockback),
        ("minecraft:follow_range", kind.follow_range),
        ("minecraft:step_height", kind.step_height),
        ("minecraft:armor", f64::from(kind.armor)),
        ("minecraft:armor_toughness", f64::from(kind.armor_toughness)),
        ("minecraft:knockback_resistance", kind.knockback_resistance),
        ("minecraft:jump_strength", kind.jump_strength),
        ("minecraft:gravity", kind.gravity),
        ("minecraft:safe_fall_distance", kind.safe_fall_distance),
        ("minecraft:fall_damage_multiplier", kind.fall_damage_multiplier),
        ("minecraft:water_movement_efficiency", kind.water_efficiency),
    ] {
        rows.entry(id.to_owned()).or_insert(json!(value));
    }
    Value::Object(rows)
}

/// `PoiType.is` by the block's saved form.
fn state_of_snbt(game: &Game<'_>, snbt: &str) -> Option<u32> {
    let (tag, _) = parse_snbt(snbt).ok()?;
    game.state_from_nbt(&tag).map(|state| state.0)
}

/// `horizon:plugin`'s `particle_data` made into what clouds send, once for each change.
static CLOUD_PARTICLES: LazyLock<Mutex<HashMap<u32, (String, horizon_protocol::entity_data::Particle)>>> = LazyLock::new(Default::default);

fn cloud_options(game: &mut Game<'_>, id: u32, tag: &Tag) {
    let Some(Tag::String(data)) = tag.get(PLUGIN).and_then(|plugin| plugin.get("particle_data")) else {
        return;
    };
    let Some(Tag::String(name)) = tag.get("custom_particle").and_then(|particle| particle.get("type")) else {
        return;
    };
    let (data, name) = (String::from_utf16_lossy(&data.0), String::from_utf16_lossy(&name.0));
    let wanted = format!("{name} {data}");
    if CLOUD_PARTICLES.lock().unwrap_or_else(|e| e.into_inner()).get(&id).is_some_and(|known| known.0 == wanted) {
        return;
    }
    let Some(options) = serde_json::from_str::<Value>(&data).ok().and_then(|data| super::paper_fam_paper::particle_data(game, &name, &data)) else {
        return;
    };
    let Some(number) = game.lan.as_ref().and_then(|host| host.hub.registry_id("minecraft:particle_type", &name)) else {
        return;
    };
    CLOUD_PARTICLES.lock().unwrap_or_else(|e| e.into_inner()).insert(id, (wanted, horizon_protocol::entity_data::Particle { id: number, data: options }));
}

pub(in crate::game) fn cloud_particle(id: u32) -> Option<horizon_protocol::entity_data::Particle> {
    CLOUD_PARTICLES.lock().unwrap_or_else(|e| e.into_inner()).get(&id).map(|known| known.1.clone())
}

/// A cloud of the game's as the compound a plugin's would be.
fn cloud_tag(cloud: &super::dragon::Breath) -> Tag {
    let number = |value: f32| Tag::Float(value.to_bits());
    let (wait, per_tick, on_use, scale, effects, color) = match &cloud.potion {
        Some(potion) => (super::potions::LINGERING_WAIT, potion.radius_per_tick, -0.5, 0.25, potion.effects.clone(), Some(potion.color)),
        // Instant damage at the cloud's half strength: the game's I is 6 and II 12 before it's halved here.
        None => (20, if cloud.from_dragon { 0.0 } else { 4.0 / cloud.duration as f32 }, 0.0, 1.0, vec![("minecraft:instant_damage", 1, if cloud.from_dragon { 1 } else { 2 })], None),
    };
    let effect = |(id, ticks, amplifier): &(&'static str, i32, i32)| Tag::compound([("id", Tag::String((*id).into())), ("duration", Tag::Int(*ticks)), ("amplifier", Tag::Byte(*amplifier as i8))]);
    let mut contents = vec![("custom_effects", Tag::List(effects.iter().map(effect).collect()))];
    if let Some(color) = color {
        contents.push(("custom_color", Tag::Int(color as i32)));
    }
    let uuid = horizon_client::lan::playerdata::uuid_ints(super::lan::entity_uuid(cloud.id));
    let mut tag = vec![
        ("id", Tag::String("minecraft:area_effect_cloud".into())),
        ("UUID", Tag::IntArray(uuid.to_vec())),
        ("Pos", Tag::List(cloud.at.iter().map(|value| Tag::Double(value.to_bits())).collect())),
        ("Age", Tag::Int(cloud.age)),
        ("Duration", Tag::Int(cloud.duration)),
        ("WaitTime", Tag::Int(wait)),
        ("ReapplicationDelay", Tag::Int(20)),
        ("Radius", number(cloud.radius)),
        ("RadiusPerTick", number(per_tick)),
        ("RadiusOnUse", number(on_use)),
        ("DurationOnUse", number(0.0)),
        ("potion_duration_scale", number(scale)),
        ("potion_contents", Tag::compound(contents)),
    ];
    if cloud.potion.is_none() {
        tag.push(("custom_particle", Tag::compound([("type", Tag::String("minecraft:dragon_breath".into()))])));
        if cloud.from_dragon {
            tag.push((PLUGIN, Tag::compound([("from_dragon", Tag::Double(1.0f64.to_bits()))])));
        }
    }
    Tag::compound(tag)
}

fn falling_tile_tag(game: &Game<'_>, id: u32, saved: Option<&Tag>) -> Option<Tag> {
    game.persist_raw(id).and_then(|raw| raw.get("TileEntityData")).or_else(|| saved?.get("TileEntityData")).cloned()
}

/// `FallingBlockEntity.blockData`, for the block entity of where it lands.
pub(in crate::game) fn falling_tile(game: &Game<'_>, id: u32, saved: Option<&Tag>) -> Option<String> {
    Some(horizon_world::snbt_text::to_snbt(&falling_tile_tag(game, id, saved)?))
}

/// What the game saves of a thrown trident, with a plugin's values.
pub(in crate::game) fn trident_tag(game: &Game<'_>, trident: &super::trident::ThrownTrident) -> Tag {
    let uuid = game.persist.uuids.get(&trident.id).copied().unwrap_or_else(|| {
        let at = trident.position;
        horizon_client::lan::playerdata::uuid_ints(super::lan::entity_uuid(trident.id ^ (at.x.to_bits() ^ at.z.to_bits()) as u32))
    });
    let owner = trident.guest.and_then(|conn| game.lan.as_ref()?.hub.guest(conn)).map(|guest| horizon_client::lan::playerdata::uuid_ints(guest.uuid));
    trident.to_nbt(uuid, owner, game.persist_raw(trident.id))
}

/// The guest a saved trident goes back to, if they're on.
pub(in crate::game) fn trident_owner(game: &Game<'_>, tag: &Tag) -> Option<u32> {
    let Some(Tag::IntArray(owner)) = tag.get("Owner") else {
        return None;
    };
    let host = game.lan.as_ref()?;
    host.hub.guests.iter().find(|guest| horizon_client::lan::playerdata::uuid_ints(guest.uuid).as_slice() == owner.as_slice()).map(|guest| guest.conn)
}

/// `LivingEntity.setMemory`: the value goes in as the mob saves it, and the mob is read back, so what the game keeps
/// live has it too. `snbt` of nothing erases it.
fn memory_set(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    use super::commands::data::DataRef;
    let id = id_of(query);
    let key = query["key"].as_str()?.to_owned();
    let short = key.trim_start_matches("minecraft:").to_owned();
    let value = query["snbt"].as_str().map(parse_snbt);
    let reference = DataRef::Entity(Target::Mob(id));
    game.native_in_level(&dimension(query), |game| {
        game.mobs.iter_mut().find(|m| m.id == id)?.extra.memories.retain(|memory| memory.0 != short);
        let mut whole = game.data_of(&reference).ok()?;
        let Tag::Compound(saved) = &mut whole else {
            return None;
        };
        let Tag::Compound(brain) = saved.entry("Brain".into()).or_insert_with(|| Tag::Compound(Default::default())) else {
            return None;
        };
        let Tag::Compound(memories) = brain.entry("memories".into()).or_insert_with(|| Tag::Compound(Default::default())) else {
            return None;
        };
        match value {
            Some(Ok((value, _))) => {
                memories.insert(key.as_str().into(), Tag::compound([("value", value)]));
            }
            Some(Err(_)) => return None,
            None => {
                memories.remove(&key.as_str().into());
            }
        }
        Some(game.set_data_of(&reference, whole).is_ok())
    })
    .flatten()
}

/// The bar of a wither or the dragon, made a bar of the game's the first time a plugin asks for it. From then on
/// `tick_boss_bars` keeps its progress and who sees it, as the boss's own `ServerBossEvent`; the rest is the plugin's.
fn boss_bar(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let id = id_of(query);
    let place = dimension(query);
    let bar = format!("horizon_bukkit:boss_{id}");
    game.native_in_level(&place, |game| {
        let dragon = game.dragon.as_ref().is_some_and(|dragon| dragon.id == id);
        let wither = game.mobs.iter().find(|m| m.id == id && m.species() == Species::Wither);
        if !dragon && wither.is_none() {
            return None;
        }
        if game.bosses.bars.get(&bar).is_none() {
            let name = match wither.and_then(|m| m.custom_name.clone()) {
                Some(name) => json!({ "text": name }),
                None if dragon => json!({ "translate": "entity.minecraft.ender_dragon" }),
                None => json!({ "translate": "entity.minecraft.wither" }),
            };
            game.bosses.bars.add(&bar, name);
            let made = game.bosses.bars.get_mut(&bar)?;
            made.color = if dragon { "pink" } else { "purple" }.to_owned();
            made.style = "progress".to_owned();
            made.max = 1_000_000;
            made.darken = !dragon;
            made.music = dragon;
            made.fog = dragon;
        }
        held().boss_bars.insert(bar.clone(), (id, place.clone(), Vec::new()));
        Some(json!(bar))
    })
    .flatten()
}

pub(in crate::game) fn boss_bar_owned(id: u32) -> bool {
    held().boss_bars.values().any(|bar| bar.0 == id)
}

fn tick_boss_bars(game: &mut Game<'_>) {
    let bars: Vec<(String, u32, String, Vec<[i32; 4]>)> = {
        let mut held = held();
        if !std::mem::replace(&mut held.boss_swept, true) {
            // The file keeps every bar; these only mean anything to the run that made them.
            let stale: Vec<String> = game.bosses.bars.ids().into_iter().filter(|id| id.starts_with("horizon_bukkit:boss_") && !held.boss_bars.contains_key(id)).collect();
            for id in stale {
                game.bosses.bars.remove(&id);
            }
        }
        held.boss_bars.iter().map(|(bar, (id, place, seen))| (bar.clone(), *id, place.clone(), seen.clone())).collect()
    };
    if bars.is_empty() {
        return;
    }
    let viewers = game.boss_viewers();
    let here = format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:"));
    for (bar, id, place, seen) in bars {
        if place != here {
            continue;
        }
        let wither = game.mobs.iter().find(|m| m.id == id && m.alive() && !m.removed);
        let (progress, near): (Option<f32>, Vec<[i32; 4]>) = if let Some(m) = wither {
            let progress = if m.extra.counter > 0 { 1.0 - m.extra.counter as f32 / mobs::wither::INVULNERABLE as f32 } else { (m.health / m.attrs.max_health).clamp(0.0, 1.0) };
            let reach = |viewer: &super::bossbars::Viewer| {
                let range = f64::from(10.min(viewer.view_chunks.max(0)) * 16);
                let (dx, dz) = (m.position.x - viewer.at[0], m.position.z - viewer.at[2]);
                dx * dx + dz * dz <= range * range
            };
            (Some(progress), viewers.iter().filter(|viewer| viewer.here && reach(viewer)).map(|viewer| viewer.ints).collect())
        } else if game.dragon.as_ref().is_some_and(|dragon| dragon.id == id) {
            let near: Vec<_> = viewers.iter().filter(|viewer| viewer.here && game.dragon_boss_progress(viewer.at, viewer.alive).is_some()).collect();
            let progress = game.dragon.as_ref().map(|dragon| (dragon.health / 200.0).clamp(0.0, 1.0));
            (progress, near.iter().map(|viewer| viewer.ints).collect())
        } else {
            (None, Vec::new())
        };
        let Some(progress) = progress else {
            game.bosses.bars.remove(&bar);
            held().boss_bars.remove(&bar);
            continue;
        };
        let name = wither.and_then(|m| m.custom_name.clone());
        if let Some(made) = game.bosses.bars.get_mut(&bar) {
            made.value = (progress * made.max as f32) as i32;
            if let Some(name) = name {
                made.name = json!({ "text": name });
            }
            // Who came near since last tick, and who left: `startSeenByPlayer`, `stopSeenByPlayer`. A player the plugin
            // added or took off itself stays as it left them.
            for viewer in near.iter().filter(|viewer| !seen.contains(viewer)) {
                if !made.players.contains(viewer) {
                    made.players.push(*viewer);
                }
            }
            for gone in seen.iter().filter(|viewer| !near.contains(viewer)) {
                made.players.retain(|player| player != gone);
            }
        }
        if let Some(held_bar) = held().boss_bars.get_mut(&bar) {
            held_bar.2 = near;
        }
    }
}

/// `horizon:plugin` onto a compound the game makes afresh.
pub(in crate::game) fn keep_plugin(mut tag: Tag, raw: Option<&Tag>) -> Tag {
    if let Tag::Compound(saved) = &mut tag {
        if let Some(plugin) = raw.and_then(|raw| raw.get(PLUGIN)) {
            saved.insert(PLUGIN.into(), plugin.clone());
        }
        if let Some(tile) = raw.and_then(|raw| raw.get("TileEntityData")) {
            saved.insert("TileEntityData".into(), tile.clone());
        }
    }
    tag
}

/// `horizon:plugin` of a compound that has just loaded, for an entity with no `persist.raw` of its own.
pub(in crate::game) fn plugin_loaded(game: &mut Game<'_>, id: u32, tag: &Tag) {
    if let Some(plugin) = tag.get(PLUGIN) {
        for (key, bit) in [("sneaking", 2), ("invisible", 32)] {
            if plugin.get(key).and_then(tag_number).is_some_and(|on| on != 0.0) {
                horizon_client::paper_hub::set_shared_flag(id as i32, bit, true);
            }
        }
    }
    if let Some(plugin) = tag.get(PLUGIN) {
        game.persist.raw.insert(id, Tag::compound([(PLUGIN, plugin.clone())]));
    }
}

/// A guest by the display name a thrown potion keeps of who threw it.
pub(in crate::game) fn named_player(game: &Game<'_>, name: Option<&horizon_client::text::Text>) -> Option<u128> {
    let name = name?.plain();
    game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.name == name).map(|guest| guest.uuid)
}

/// `ServerPlayer.die`'s orbs: the dead player, and the player who killed them.
pub(in crate::game) fn orb_cause_death(host: &crate::game::lan::LanHost, conn: u32, killer: Option<(Who, i64)>) {
    let dead = host.hub.guest(conn).map(|guest| horizon_client::lan::playerdata::uuid_ints(guest.uuid));
    let killer = killer.and_then(|(who, _)| match who {
        Who::Mob(id) => host.hub.guests.iter().find(|guest| guest.entity_id as u32 == id).map(|guest| guest.uuid),
        Who::Player => None,
    });
    *ORB_CAUSE.lock().unwrap_or_else(|e| e.into_inner()) = Some(("PLAYER_DEATH", dead, killer));
}

pub(in crate::game) fn orb_cause_host_death(dead: [i32; 4]) {
    *ORB_CAUSE.lock().unwrap_or_else(|e| e.into_inner()) = Some(("PLAYER_DEATH", Some(dead), None));
}

/// `Entity.tickCount` of the kinds with no clock: counted from the tick after they joined, as vanilla counts it, and
/// not saved, as vanilla doesn't.
fn note_born(game: &Game<'_>, held: &mut Held, now: i64) {
    let ids = game.armor_stands.iter().map(|e| e.id)
        .chain(game.item_frames.iter().map(|e| e.id))
        .chain(game.paintings.iter().map(|e| e.id))
        .chain(game.others.iter().map(|e| e.id))
        .chain(game.crystals.iter().map(|e| e.id))
        .chain(game.tnts.iter().map(|e| e.id))
        .chain(game.falling.iter().map(|e| e.id))
        .chain(game.rockets.iter().map(|e| e.id))
        .chain(game.bolts.iter().map(|e| e.id))
        .chain(game.fangs.list.iter().map(|e| e.id))
        .chain(game.end.eyes.iter().map(|e| e.id))
        .chain(game.potions.thrown.iter().map(|e| e.id))
        .chain(game.tridents.iter().map(|e| e.id))
        .chain(game.breath_clouds.iter().map(|e| e.id))
        .chain(game.dragon.iter().map(|e| e.id));
    let mut seen = Vec::new();
    for id in ids.filter(|id| *id != 0) {
        held.born.entry(id).or_insert(now);
        seen.push(id);
    }
    if now % 1200 == 0 {
        held.born.retain(|id, _| seen.contains(id));
    }
}

fn born_ticks(game: &mut Game<'_>, key: &str, query: &Value) -> Option<Value> {
    let id = id_of(query);
    let now = game.world.game_time;
    match key {
        "ticks" => Some(json!(held().born.get(&id).map_or(0, |born| now - born))),
        "set_ticks" => {
            let mut held = held();
            let born = held.born.get_mut(&id)?;
            *born = now - query["value"].as_i64()?.max(0);
            Some(json!(true))
        }
        _ => None,
    }
}

/// A primed TNT as the game would save it, had it saved them.
pub(in crate::game) fn tnt_tag(game: &Game<'_>, tnt: &super::tnt::PrimedTnt) -> Tag {
    let raw = game.persist_raw(tnt.id);
    let mut saved = match raw {
        Some(Tag::Compound(saved)) => saved.clone(),
        _ => Default::default(),
    };
    let list = |v: Vec3| Tag::List([v.x, v.y, v.z].iter().map(|d| Tag::Double(d.to_bits())).collect());
    let uuid = game.persist.uuids.get(&tnt.id).copied().unwrap_or_else(|| horizon_client::lan::playerdata::uuid_ints(super::lan::entity_uuid(tnt.id ^ (tnt.position.x.to_bits() ^ tnt.position.z.to_bits()) as u32)));
    let owner = held().tnt.get(&tnt.id).and_then(|entry| entry.1);
    let owner = match owner {
        Some(Source::Player(uuid)) => Some(horizon_client::lan::playerdata::uuid_ints(uuid)),
        Some(Source::Mob(id)) => game.persist.uuids.get(&id).copied(),
        None => None,
    };
    if let Some(owner) = owner {
        saved.insert("owner".into(), Tag::IntArray(owner.to_vec()));
    }
    for (key, value) in [
        ("id", Tag::String("minecraft:tnt".into())),
        ("UUID", Tag::IntArray(uuid.to_vec())),
        ("Pos", list(tnt.position)),
        ("Motion", list(tnt.velocity)),
        ("OnGround", Tag::Byte(i8::from(tnt.on_ground))),
        ("fuse", Tag::Short(tnt.fuse.clamp(0, i32::from(i16::MAX)) as i16)),
    ] {
        saved.insert(key.into(), value);
    }
    Tag::Compound(saved)
}

/// `Player.increaseScore`: what the death screen shows counts the experience taken in.
pub(in crate::game) fn score_up(game: &mut Game<'_>, gained: u32) {
    let conn = game.lan_acting;
    let Tag::Compound(saved) = game.level.player_extra.get_or_insert_with(|| Tag::Compound(Default::default())) else {
        return;
    };
    let score = saved.get(&"Score".into()).and_then(tag_number).unwrap_or(0.0) as i32 + gained as i32;
    saved.insert("Score".into(), Tag::Int(score));
    let Some(guest) = conn.and_then(|conn| game.lan.as_ref()?.hub.guest(conn)) else {
        return;
    };
    let values = vec![(18, horizon_protocol::entity_data::serializer::INT, horizon_protocol::entity_data::Value::Int(score))];
    guest.send(&C::SetEntityData(play::SetEntityData { id: guest.entity_id, values }));
}

/// `ServerPlayer.restoreFrom`: a death that keeps nothing keeps no score.
pub(in crate::game) fn score_reset(kept: bool, guest: &mut horizon_client::lan::guest::Guest) {
    if !kept
        && let Some(Tag::Compound(saved)) = guest.saved.as_mut()
    {
        saved.remove(&"Score".into());
    }
}

/// `setSneaking` and `setInvisible` on anything, for those who see it.
pub(in crate::game) fn shared_flags_tail(out: &mut horizon_client::lan::entity::States) {
    use horizon_protocol::entity_data::{Value as Data, serializer};
    for state in out.as_mut_slice() {
        let Some(flags) = horizon_client::paper_hub::shared_flags(state.id) else {
            continue;
        };
        match state.meta.iter_mut().find(|row| row.0 == 0) {
            Some((_, _, Data::Byte(byte))) => *byte |= flags,
            _ if flags != 0 => state.meta.push((0, serializer::BYTE, Data::Byte(flags))),
            _ => {}
        }
        if !state.defaults.iter().any(|row| row.0 == 0) {
            state.defaults.push((0, serializer::BYTE, Data::Byte(0)));
        }
    }
}

/// A bucket put this mob down: `Bucketable.setFromBucket`.
pub(in crate::game) fn from_bucket(game: &mut Game<'_>, id: u32) {
    if let Tag::Compound(saved) = game.persist.raw.entry(id).or_insert_with(|| Tag::Compound(Default::default())) {
        saved.insert("FromBucket".into(), Tag::Byte(1));
    }
}

thread_local! {
    /// `Shearable.shear` goes through the dispenser's own path, for this mob and past what makes a dispenser pass it by.
    static SHEAR_ONLY: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
}

pub(in crate::game) fn shear_only() -> Option<u32> {
    SHEAR_ONLY.get()
}

/// `Entity.hurt`'s `indicateDamage`, for `LivingEntity.getHurtDirection`.
fn shear(game: &mut Game<'_>, index: usize) -> Value {
    let m = &mut game.mobs[index];
    let (id, at) = (m.id, m.position);
    m.extra.sheared = false;
    let pos = (at.x.floor() as i32, at.y.floor() as i32, at.z.floor() as i32);
    SHEAR_ONLY.set(Some(id));
    let done = game.shear_entity(pos).unwrap_or(false);
    SHEAR_ONLY.set(None);
    json!(done)
}

/// A chunk goes: what it takes is `UNLOADED_TO_CHUNK`, not discarded.
pub(in crate::game) fn note_unloaded(game: &Game<'_>, gone: &dyn Fn(Vec3) -> bool) {
    let mut held = held();
    if held.unloaded.len() > 100_000 {
        held.unloaded.clear();
    }
    let ids = game.mobs.iter().filter(|e| gone(e.position)).map(|e| e.id)
        .chain(game.dropped.iter().filter(|e| gone(e.position)).map(|e| e.id))
        .chain(game.boats.iter().filter(|e| gone(e.position)).map(|e| e.id))
        .chain(game.carts.iter().filter(|e| gone(e.position)).map(|e| e.id))
        .chain(game.orbs.iter().filter(|e| gone(e.position)).map(|e| e.id))
        .chain(game.armor_stands.iter().filter(|e| gone(e.position)).map(|e| e.id))
        .chain(game.creatures.projectiles.iter().filter(|e| gone(e.position)).map(|e| e.id))
        .chain(game.tridents.iter().filter(|e| gone(e.position)).map(|e| e.id))
        .chain(game.tnts.iter().filter(|e| gone(e.position)).map(|e| e.id))
        .chain(game.falling.iter().filter(|e| gone(e.position)).map(|e| e.id));
    held.unloaded.extend(ids);
}

/// `World.spawn` and the rest: `CreatureSpawnEvent.SpawnReason.CUSTOM`, whatever the game did last.
pub(in crate::game) fn spawned_by_plugin(game: &mut Game<'_>, dimension_id: &str, answer: Value) -> Value {
    if let Some(id) = answer["id"].as_u64() {
        let _ = plugin_set(game, "spawn_reason", &json!({ "id": id, "dimension": dimension_id, "value": "CUSTOM" }));
    }
    answer
}

/// A witch has drunk what a plugin gave her: its own effects, past the kind she knows. Not under `held`: a new effect
/// can reach a listener, and a listener can reach `held`.
fn witches_finish(game: &mut Game<'_>) {
    let finished: Vec<(u32, ItemStack)> = {
        let mut held = held();
        let done: Vec<u32> = held.witches.keys().copied().filter(|id| !game.mobs.iter().any(|m| m.id == *id && m.extra.flag)).collect();
        done.into_iter().filter_map(|id| Some((id, held.witches.remove(&id)?))).collect()
    };
    for (id, stack) in finished {
        let Some(m) = game.mobs.iter_mut().find(|m| m.id == id) else {
            continue;
        };
        let tag = horizon_client::save::item_to_nbt(&stack);
        let contents = tag.get("components").and_then(|components| components.get("minecraft:potion_contents")).cloned();
        for (effect, ticks, amplifier) in contents.map(|contents| cloud_effects(&Tag::compound([("potion_contents", contents)]))).unwrap_or_default() {
            m.add_effect(effect, ticks, amplifier);
        }
    }
}
