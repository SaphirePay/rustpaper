// Family items: answers to the queries its SDK file makes.
use super::render_control::parse_state;
use crate::game::Game;
use super::lan::LanHost;
use horizon_client::{
    block_entity_anim::{BellEntity, ConduitServer},
    containers::{Beacon, BlockEntity},
    effects::EffectInstance,
    items::ItemStack,
    lan::{
        hub::Hub,
        menu::{Slots, VEHICLE_KEY_Y},
    },
    menu::{Kind, Menu},
    merchant::MerchantOffer,
};
use horizon_core::random::{LegacyRandom, RandomSource};
use horizon_client_core::potions;
use horizon_protocol::packets::play::{self, Clientbound as C};
use horizon_core::{
    geometry::{Direction, Vec3},
    position::BlockPos,
};
use horizon_world::{
    level::{
        data::{INSTRUMENTS, PushReaction, state_flags},
        hit_slot,
    },
    compression::Compression,
    nbt::{Compound, Limits, NamedTag, Tag, parse_snbt, read_named, write_named},
    registry::BlockStateId,
    worldgen::structure::{Mirror, Rotation},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};

fn position(value: &Value) -> Option<[i32; 3]> {
    let list = value.as_array()?;
    Some([list.first()?.as_i64()? as i32, list.get(1)?.as_i64()? as i32, list.get(2)?.as_i64()? as i32])
}

fn dimension(query: &Value) -> &str {
    query["dimension"].as_str().unwrap_or("minecraft:overworld")
}

fn uuid(value: &Value) -> Option<u128> {
    u128::from_str_radix(value.as_str()?, 16).ok()
}

/// A stack a plugin built: its item, count and components, each as SNBT.
fn stack_from(value: &Value) -> Option<ItemStack> {
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

fn stack_value(stack: &ItemStack) -> Value {
    super::paper_query::stack_json(stack)
}

fn stacks_value(stacks: &[ItemStack]) -> Value {
    Value::Array(stacks.iter().map(stack_value).collect())
}

fn state_text(game: &Game<'_>, state: BlockStateId) -> String {
    game.pack.assets.registry.state(state).map_or_else(
        || "minecraft:air".to_owned(),
        |state| {
            if state.properties.is_empty() {
                return state.name.clone();
            }
            let properties: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
            format!("{}[{}]", state.name, properties.join(","))
        },
    )
}

/// The state a request names: `block` as the game writes it, or what stands at `pos`.
fn state_of(game: &mut Game<'_>, query: &Value) -> Option<BlockStateId> {
    if let Some(block) = query["block"].as_str() {
        return parse_state(game.pack.assets.registry, block).ok();
    }
    let pos = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| game.world.block(pos[0], pos[1], pos[2]))
}

/// Everything the game knows of a block state, wherever it stands: `BlockData` and `BlockType`.
fn block_facts(game: &Game<'_>, state: BlockStateId) -> Value {
    let facts = game.res.facts().get(state);
    let behaviours = game.res.behaviours();
    let cached = behaviours.facts(state);
    let data = behaviours.data(state);
    let sounds = &facts.sounds;
    let push = match cached.push {
        PushReaction::PushPull => "MOVE",
        PushReaction::Push => "PUSH_ONLY",
        PushReaction::Popped => "BREAK",
        PushReaction::Immoveable => "BLOCK",
        PushReaction::IgnoreEntity => "IGNORE",
    };
    json!({
        "state": state_text(game, state),
        "name": &*facts.name,
        "item": &*facts.item,
        "emission": facts.emission,
        "hardness": facts.hardness,
        "explosion_resistance": facts.explosion_resistance,
        "friction": facts.friction,
        "requires_tool": facts.requires_tool,
        "replaceable": facts.replaceable,
        "random_ticks": facts.random_ticks,
        "block_entity": facts.block_entity,
        "full_collision": facts.full_collision,
        "solid": facts.legacy_solid,
        "liquid": facts.fluid != horizon_client::blocks::Fluid::None,
        "occluding": facts.can_occlude,
        "map_color": horizon_client::blocks::MAP_COLORS.get(usize::from(facts.map_color)).copied().unwrap_or(0),
        "shape": &*facts.outline,
        "classes": facts.classes.iter().map(|class| &**class).collect::<Vec<_>>(),
        "push": push,
        "instrument": INSTRUMENTS.get(usize::from(cached.instrument)).copied().unwrap_or("harp"),
        "ignite_odds": data.ignite_odds,
        "ignited_by_lava": cached.flags & state_flags::IGNITED_BY_LAVA != 0,
        "fluid_level": facts.fluid_level,
        "fluid_falling": facts.fluid_falling,
        "fluid": match facts.fluid {
            horizon_client::blocks::Fluid::None => "none",
            horizon_client::blocks::Fluid::Water => "water",
            horizon_client::blocks::Fluid::Lava => "lava",
        },
        "collides": behaviours.collision_shape(state).is_none_or(|shape| !shape.is_empty()),
        "burn_odds": data.burn_odds,
        "volume": sounds.volume,
        "pitch": sounds.pitch,
        "break_sound": &*sounds.break_sound,
        "step_sound": &*sounds.step,
        "place_sound": &*sounds.place,
        "hit_sound": &*sounds.hit,
        "fall_sound": &*sounds.fall,
    })
}

fn direction(name: &str) -> Option<Direction> {
    Some(match name {
        "DOWN" => Direction::Down,
        "UP" => Direction::Up,
        "NORTH" => Direction::North,
        "SOUTH" => Direction::South,
        "WEST" => Direction::West,
        "EAST" => Direction::East,
        _ => return None,
    })
}

/// `{ "op": "items_block_sturdy", "block", "face", "support" }`: `BlockState.isFaceSturdy`, level-free.
fn sturdy(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let state = state_of(game, query)?;
    let face = direction(query["face"].as_str()?)?;
    let behaviours = game.res.behaviours();
    Some(match query["support"].as_str().unwrap_or("FULL") {
        "CENTER" => behaviours.is_face_sturdy_center(state, face),
        "RIGID" => behaviours.is_face_sturdy_rigid(state, face),
        _ => behaviours.is_face_sturdy(state, face),
    })
}

/// `Block.getDrops` of `block` or what stands at `pos`, for `tool`.
fn drops(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"]).unwrap_or([0, 0, 0]);
    let state = state_of(game, query)?;
    let tool = query.get("tool").filter(|tool| !tool.is_null()).and_then(stack_from);
    let facts = game.res.facts().get(state);
    // CraftBlock.getDrops: a tool that can't harvest the block gets nothing; no tool gets the table.
    if let Some(tool) = &tool
        && facts.requires_tool
        && !game.res.items().correct_for_drops(Some(tool), &facts.name)
    {
        return Some(json!([]));
    }
    // `LootContextParams.THIS_ENTITY` only when the plugin named an entity.
    horizon_items::items::LOOT_WITHOUT_ENTITY.store(!query["entity"].as_bool().unwrap_or(false), std::sync::atomic::Ordering::Relaxed);
    let stacks = game.native_in_level(dimension(query), |game| game.block_loot_at(pos, state, tool.as_ref()));
    horizon_items::items::LOOT_WITHOUT_ENTITY.store(false, std::sync::atomic::Ordering::Relaxed);
    Some(stacks_value(&stacks?))
}

/// `{ "op": "items_block_tool", "block", "tool" }`: what a stack does to a block.
fn tool_facts(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let state = state_of(game, query)?;
    let tool = stack_from(&query["tool"]);
    let facts = game.res.facts().get(state);
    let items = game.res.items();
    let correct = items.correct_for_drops(tool.as_ref(), &facts.name);
    Some(json!({
        "speed": items.mining_speed(tool.as_ref(), &facts.name),
        "correct": correct,
        "harvests": !facts.requires_tool || correct,
        "efficiency": tool.as_ref().map_or(0, |tool| tool.enchantment_level("minecraft:efficiency")),
    }))
}

/// `Block.getBreakSpeed`: `BlockBehaviour.getDestroyProgress` for that player, on their own turn.
fn break_speed(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    let player = uuid(&query["uuid"])?;
    let dimension = dimension(query).to_owned();
    if game.native_acting_id() == player {
        return game.native_in_level(&dimension, |game| json!(game.destroy_progress(pos)));
    }
    let mut host = game.lan.take()?;
    let conn = host.hub.guests.iter().find(|g| g.uuid == player).map(|g| g.conn);
    let answer = conn.and_then(|conn| game.as_guest(&mut host, conn, |game| game.destroy_progress(pos)));
    game.lan = Some(host);
    answer.map(|speed| json!(speed))
}

/// `Block.randomTick` and `fluidTick`.
fn tick_block(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    let random = query["kind"].as_str() == Some("random");
    game.native_in_level(dimension(query), |game| {
        let pos = BlockPos::new(x, y, z);
        game.with_level(|behaviours, level, ticks| {
            if random {
                behaviours.random_tick(level, ticks, pos);
            } else {
                behaviours.fluid_tick_at(level, ticks, pos);
            }
            true
        })
    })
}

/// `{ "op": "items_block_survives", ... }`: `BlockState.canSurvive` of `block` at `pos`.
fn survives(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    let state = parse_state(game.pack.assets.registry, query["block"].as_str()?).ok()?;
    game.native_in_level(dimension(query), |game| game.with_level(|behaviours, level, ticks| behaviours.can_survive(level, ticks, BlockPos::new(x, y, z), state)))
}

/// `BlockData.rotate` and `mirror`.
fn transform(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let state = parse_state(game.pack.assets.registry, query["block"].as_str()?).ok()?;
    let transforms = game.state_transforms()?;
    let changed = if let Some(name) = query["rotation"].as_str() {
        transforms.rotate(state, Rotation::from_name(name)?)
    } else {
        let mirror = match query["mirror"].as_str()? {
            "LEFT_RIGHT" => Mirror::LeftRight,
            "FRONT_BACK" => Mirror::FrontBack,
            _ => Mirror::None,
        };
        transforms.mirror(state, mirror)
    };
    Some(json!(state_text(game, changed)))
}

/// The default components of every item, as `items.json` lists them.
fn item_defaults(game: &Game<'_>) -> &'static Value {
    static DEFAULTS: OnceLock<Value> = OnceLock::new();
    DEFAULTS.get_or_init(|| serde_json::from_slice(&std::fs::read(game.options.resources.join("items.json")).unwrap_or_default()).unwrap_or(Value::Null))
}

/// `{ "op": "items_item", "item" }`: an item's default components, and what they come to.
fn item_info(game: &Game<'_>, item: &str) -> Value {
    let Some(info) = game.res.items().get(item) else {
        return Value::Null;
    };
    let fuel = info.fuel.as_ref().map(|(burn, _)| burn.clone());
    json!({
        "components": item_defaults(game)[item].clone(),
        "max_stack": info.max_stack,
        "max_damage": info.max_damage,
        "block": info.block.map(|state| state_text(game, state)),
        "crafting_remainder": info.crafting_remainder,
        "fuel": fuel,
        "compostable": info.compostable,
    })
}

/// `ItemType.getBurnDuration`, in a plain furnace.
fn burn_time(game: &Game<'_>, item: &str) -> Value {
    let stack = ItemStack { item: item.into(), count: 1, damage: 0, extra: None };
    json!(game.res.providers().burn_time(game.res.items(), &stack, false))
}

/// `BlockType.createBlockDataStates`: every state of a block.
fn block_states(game: &Game<'_>, block: &str) -> Value {
    let name = if block.contains(':') { block.to_owned() } else { format!("minecraft:{block}") };
    let registry = game.pack.assets.registry;
    let states = (0..registry.len()).map(|id| BlockStateId(id as _)).filter(|state| registry.state(*state).is_some_and(|known| known.name == name));
    Value::Array(states.map(|state| json!(state_text(game, state))).collect())
}

/// `{ "op": "items_potion", "id" }`: a potion's effects, and which stronger and longer forms it has.
fn potion_info(id: &str) -> Value {
    let name = id.trim_start_matches("minecraft:");
    let Some(def) = potions::potion(name) else {
        return Value::Null;
    };
    let base = name.trim_start_matches("long_").trim_start_matches("strong_");
    let effects: Vec<Value> = def.effects.iter().map(|(effect, duration, amplifier)| json!({ "id": potions::namespaced(potions::effect(effect).map_or("speed", |known| known.id)), "duration": duration, "amplifier": amplifier })).collect();
    json!({
        "effects": effects,
        "upgradeable": potions::potion(&format!("strong_{base}")).is_some(),
        "extendable": potions::potion(&format!("long_{base}")).is_some(),
    })
}

fn effect_info(id: &str) -> Value {
    let name = id.trim_start_matches("minecraft:");
    let Some(def) = potions::effect(name) else {
        return Value::Null;
    };
    let category = match def.category {
        potions::Category::Beneficial => "BENEFICIAL",
        potions::Category::Harmful => "HARMFUL",
        potions::Category::Neutral => "NEUTRAL",
    };
    let number = potions::all_effects().position(|known| known.id == def.id).map(|index| index + 1);
    json!({ "color": def.color, "category": category, "instant": def.instant, "number": number })
}

/// `PotionContents.getColor` over effects as items save them.
fn potion_color(effects: &Value) -> Value {
    let (mut r, mut g, mut b, mut total) = (0_u32, 0_u32, 0_u32, 0_u32);
    for row in effects.as_array().into_iter().flatten() {
        if row.get("show_particles").is_some_and(|show| show.as_bool() == Some(false)) {
            continue;
        }
        let Some(effect) = row["id"].as_str().and_then(potions::effect) else {
            continue;
        };
        let weight = row["amplifier"].as_u64().unwrap_or(0) as u32 + 1;
        r += weight * ((effect.color >> 16) & 255);
        g += weight * ((effect.color >> 8) & 255);
        b += weight * (effect.color & 255);
        total += weight;
    }
    if total == 0 {
        return json!(potions::BASE_POTION_COLOR);
    }
    json!(((r / total) << 16) | ((g / total) << 8) | (b / total))
}

/// `{ "op": "items_block_entity" ... }`: bells, which live with the block animations.
fn bell(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    let dimension = dimension(query).to_owned();
    game.native_in_level(&dimension, |game| {
        // Where nothing draws the level the bell's tick doesn't run: its ring's age is the same count.
        let now = game.world.game_time;
        let bell = game.be_anims.bells.get(&pos);
        let age = bell.map_or(i64::MAX, |bell| now - bell.last_ring_timestamp);
        let shaking = bell.is_some_and(|bell| bell.shaking) && age < 50;
        json!({ "ticks": if shaking { age } else { 0 }, "shaking": shaking, "resonating": bell.is_some_and(|bell| bell.resonating), "resonation_ticks": bell.map_or(0, |bell| bell.resonation_ticks) })
    })
}

fn jukebox(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let [x, y, z] = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| match game.world.block_entities.get(&(x, y, z)) {
        Some(BlockEntity::Jukebox(jukebox)) => json!({ "playing": jukebox.playing, "ticks": jukebox.ticks }),
        _ => Value::Null,
    })
}

fn can_smelt(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let stack = stack_from(&query["stack"])?;
    let kind = match query["block"].as_str()?.trim_start_matches("minecraft:") {
        "blast_furnace" => "minecraft:blasting",
        "smoker" => "minecraft:smoking",
        _ => "minecraft:smelting",
    };
    Some(game.recipes().cook(kind, &stack).is_some())
}

fn dispense(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| {
        let state = game.world.block(x, y, z);
        game.dispense_from(BlockPos::new(x, y, z), state);
        true
    })
}

fn hive_sedated(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| {
        (1..=5).any(|down| {
            let state = game.world.block(x, y - down, z);
            let facts = game.res.facts().get(state);
            !facts.fluid.eq(&horizon_client::blocks::Fluid::None) || game.pack.assets.registry.state(state).is_some_and(|known| known.name.ends_with("campfire") && known.properties.get("lit").is_some_and(|lit| lit == "true"))
        })
    })
}

// ---- Open menus: the hooks in family/items.py lend each one here as the game sends it ----

/// What a plugin asked of a player's open menu, done the next time the game sends it: within the tick.
enum MenuWrite {
    Slot(usize, Option<ItemStack>),
    Data(usize, i32),
    RepairCount(u32),
    /// Nothing to change: the menu is only to be sent again.
    Resend,
    /// An enchanting table's offer: its index, enchantment, level and cost.
    Offer(usize, Option<String>, i32, i32),
}

/// Each player's open menu as last sent, by UUID: read while the host is out of the game.
static OPEN: Mutex<BTreeMap<u128, Value>> = Mutex::new(BTreeMap::new());
static PENDING: Mutex<BTreeMap<u128, Vec<MenuWrite>>> = Mutex::new(BTreeMap::new());
/// A player's anvil under a plugin's rule: its maximum cost, and whether levels may pass the enchantment's own.
static ANVIL_RULES: Mutex<BTreeMap<u128, (i32, bool)>> = Mutex::new(BTreeMap::new());

fn open_menus() -> std::sync::MutexGuard<'static, BTreeMap<u128, Value>> {
    OPEN.lock().unwrap_or_else(|e| e.into_inner())
}

fn pending_writes() -> std::sync::MutexGuard<'static, BTreeMap<u128, Vec<MenuWrite>>> {
    PENDING.lock().unwrap_or_else(|e| e.into_inner())
}

/// A workstation or crafting table as the game sends it: `Menu` is lent by the hook's module.
pub(in crate::game) fn menu_sent(game: &Game<'_>, hub: &Hub, player: u128, window: i32, pos: [i32; 3], menu: &mut Menu) -> Value {
    let env = game.menu_env();
    let mut resent = false;
    if let Some(writes) = pending_writes().remove(&player) {
        for write in writes {
            match write {
                MenuWrite::Slot(slot, stack) => menu.set_slot(&env, slot, stack),
                MenuWrite::Data(index, value) => {
                    if index < menu.data.len() {
                        menu.set_data(index, value);
                    }
                }
                MenuWrite::RepairCount(count) => menu.anvil.repair_count = count,
                MenuWrite::Resend => resent = true,
                MenuWrite::Offer(index, key, level, cost) => {
                    if menu.kind() == Kind::Enchanting && index < 3 {
                        menu.data[index] = cost;
                        menu.data[4 + index] = key.and_then(|key| hub.registry_id("minecraft:enchantment", &key)).unwrap_or(-1);
                        menu.data[7 + index] = level;
                    }
                }
            }
        }
    }
    let _ = resent;
    apply_anvil_rule(player, &env, menu);
    let own = menu.slot_count().saturating_sub(36);
    let slots: Vec<Value> = (0..own).map(|slot| menu.slot(slot).map_or(Value::Null, stack_value)).collect();
    let name = |registry: &str, id: i32| hub.registry_name(registry, id).map(str::to_owned);
    // The enchanting table's clues and the beacon's powers are registry numbers in the data slots.
    let names: Value = match menu.kind() {
        Kind::Enchanting => json!((4..7).map(|index| menu.data.get(index).and_then(|id| name("minecraft:enchantment", *id))).collect::<Vec<_>>()),
        Kind::Beacon => json!((1..3).map(|index| menu.data.get(index).and_then(|id| name("minecraft:mob_effect", *id - 1))).collect::<Vec<_>>()),
        _ => Value::Null,
    };
    open_menus().insert(player, json!({
        "kind": format!("{:?}", menu.kind()),
        "window": window,
        "pos": pos,
        "slots": slots,
        "data": menu.data,
        "names": names,
        "rename": menu.anvil.name,
        "repair_count": menu.anvil.repair_count,
        "patterns": menu.loom_patterns.iter().map(|pattern| &**pattern).collect::<Vec<_>>(),
        "recipes": menu.stone.recipes.iter().filter_map(|index| game.stations().recipes.stonecutting.get(*index)).map(|recipe| recipe.id.as_str()).collect::<Vec<_>>(),
    }));
    Value::Null
}

/// A lectern as the game sends it.
pub(in crate::game) fn lectern_sent(player: u128, window: i32, key: (i32, i32, i32)) -> Value {
    open_menus().insert(player, json!({ "kind": "Lectern", "window": window, "pos": [key.0, key.1, key.2], "slots": [], "data": [] }));
    Value::Null
}

/// A merchant's screen as the game sends it. `offers` are as `Offers.Recipes` saves them.
pub(in crate::game) fn merchant_sent(game: &mut Game<'_>, player: u128, window: i32, mob: Option<u32>, slots: &[Option<ItemStack>], selected: Option<usize>, offers: &mut [MerchantOffer]) -> Value {
    if let Some(mob) = mob {
        let words = game.uuid_for(mob);
        let flags = extra_at(game, &format!("entity|{words:?}"), "ignore_discounts");
        for (index, offer) in offers.iter_mut().enumerate() {
            if flags[index].as_bool() == Some(true) {
                offer.special_price_diff = 0;
            }
        }
    }
    let stack = |stack: &Option<ItemStack>| stack.as_ref().map_or(Value::Null, stack_value);
    let offers: Vec<Value> = offers.iter().map(|offer| json!({
        "buy": stack_value(&offer.base_cost_a.stack),
        "buyB": offer.cost_b.as_ref().map(|cost| stack_value(&cost.stack)),
        "sell": stack_value(&offer.result),
        "uses": offer.uses,
        "maxUses": offer.max_uses,
        "rewardExp": offer.reward_exp,
        "specialPrice": offer.special_price_diff,
        "demand": offer.demand,
        "priceMultiplier": offer.price_multiplier,
        "xp": offer.xp,
    })).collect();
    open_menus().insert(player, json!({ "kind": "Merchant", "window": window, "mob": mob, "slots": slots.iter().map(stack).collect::<Vec<_>>(), "selected": selected, "offers": offers }));
    Value::Null
}

/// The connections with writes waiting for their menu to be sent.
pub(in crate::game) fn pending_conns(host: &LanHost) -> Vec<u32> {
    let pending = pending_writes();
    host.hub.guests.iter().filter(|guest| pending.contains_key(&guest.uuid)).map(|guest| guest.conn).collect()
}

/// A menu closed: `None` closes whichever is open.
pub(in crate::game) fn menu_closed(host: &LanHost, conn: u32, window: Option<i32>) -> Value {
    let Some(player) = host.hub.guest(conn).map(|guest| guest.uuid) else {
        return Value::Null;
    };
    let mut open = open_menus();
    if window.is_none_or(|window| open.get(&player).is_some_and(|menu| menu["window"].as_i64() == Some(i64::from(window)))) {
        open.remove(&player);
        pending_writes().remove(&player);
        ANVIL_RULES.lock().unwrap_or_else(|e| e.into_inner()).remove(&player);
    }
    Value::Null
}

/// The container a player has open that isn't a workstation: a chest, a furnace, a hopper.
fn guest_menu(game: &Game<'_>, player: u128) -> Option<Value> {
    let guest = game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == player)?;
    let menu = guest.menu.as_ref()?;
    Some(json!({ "kind": "Block", "window": menu.window, "pos": menu.pos.map(|pos| [pos.0, pos.1, pos.2]), "size": menu.size, "menu": menu.kind }))
}

/// The inventory's own 2x2 grid, when nothing else is open: result, then the grid.
fn own_grid(game: &Game<'_>, player: u128) -> Option<Value> {
    let guest = game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == player)?;
    let stack = |stack: &Option<ItemStack>| stack.as_ref().map_or(Value::Null, stack_value);
    let mut slots = vec![stack(&guest.craft_result)];
    slots.extend(guest.craft_grid.iter().map(stack));
    Some(json!({ "kind": "Inventory", "window": 0, "slots": slots, "data": [] }))
}

/// `{ "op": "items_menu", "uuid" }`: what a player has open.
fn menu_state(game: &Game<'_>, query: &Value) -> Value {
    let Some(player) = uuid(&query["uuid"]) else {
        return Value::Null;
    };
    if let Some(open) = open_menus().get(&player) {
        return open.clone();
    }
    guest_menu(game, player).or_else(|| own_grid(game, player)).unwrap_or(Value::Null)
}

/// `{ "op": "items_menu_set", "uuid", "slot", "stack" }` and `"data"`: on the menu at its next send,
/// and at once on what later reads in this event see.
fn menu_set(game: &mut Game<'_>, query: &Value) -> bool {
    let Some(player) = uuid(&query["uuid"]) else {
        return false;
    };
    let mut open = open_menus();
    let Some(state) = open.get_mut(&player) else {
        drop(open);
        return own_grid_set(game, player, query);
    };
    let write = if let Some(slot) = query["slot"].as_u64() {
        let slot = slot as usize;
        let stack = stack_from(&query["stack"]);
        if let Some(place) = state["slots"].get_mut(slot) {
            *place = stack.as_ref().map_or(Value::Null, stack_value);
        }
        MenuWrite::Slot(slot, stack)
    } else if let Some(index) = query["offer"].as_u64() {
        let index = index as usize;
        let (level, cost) = (query["level"].as_i64().unwrap_or(-1) as i32, query["cost"].as_i64().unwrap_or(0) as i32);
        let key = query["key"].as_str().map(str::to_owned);
        if let Some(place) = state["data"].get_mut(index) {
            *place = json!(cost);
        }
        if let Some(place) = state["data"].get_mut(7 + index) {
            *place = json!(level);
        }
        if let Some(place) = state["names"].get_mut(index) {
            *place = json!(key);
        }
        MenuWrite::Offer(index, key, level, cost)
    } else if let Some(count) = query["repair_count"].as_u64() {
        state["repair_count"] = json!(count);
        MenuWrite::RepairCount(count as u32)
    } else {
        let (index, value) = (query["data"].as_u64().unwrap_or(0) as usize, query["value"].as_i64().unwrap_or(0) as i32);
        if let Some(place) = state["data"].get_mut(index) {
            *place = json!(value);
        }
        MenuWrite::Data(index, value)
    };
    pending_writes().entry(player).or_default().push(write);
    true
}

fn own_grid_set(game: &mut Game<'_>, player: u128, query: &Value) -> bool {
    let (Some(slot), Some(host)) = (query["slot"].as_u64(), game.lan.as_mut()) else {
        return false;
    };
    let Some(guest) = host.hub.guests.iter_mut().find(|guest| guest.uuid == player) else {
        return false;
    };
    let stack = stack_from(&query["stack"]);
    match slot {
        0 => guest.craft_result = stack,
        1..=4 => guest.craft_grid[slot as usize - 1] = stack,
        _ => return false,
    }
    true
}

/// `{ "op": "items_cursor", "uuid", "stack" }`: `InventoryView.setCursor`.
fn set_cursor(game: &mut Game<'_>, query: &Value) -> bool {
    let (Some(player), Some(host)) = (uuid(&query["uuid"]), game.lan.as_mut()) else {
        return false;
    };
    let Some(index) = host.hub.guests.iter().position(|guest| guest.uuid == player) else {
        return false;
    };
    let stack = stack_from(&query["stack"]);
    host.hub.guests[index].carried = stack.clone();
    let item = host.hub.stack(&stack);
    host.hub.guests[index].send(&C::SetCursorItem(play::SetCursorItem { item }));
    true
}

/// `{ "op": "items_menu_title", "uuid", "title" }`: `InventoryView.setTitle`, as Java does it: the same
/// window opened again under the new title, and every slot sent again.
fn set_title(game: &mut Game<'_>, query: &Value) -> bool {
    let (Some(player), Some(host)) = (uuid(&query["uuid"]), game.lan.as_mut()) else {
        return false;
    };
    let title = horizon_client::lan::data::component_nbt(&horizon_client_render::packs::component(&query["title"]).component());
    let open = open_menus().get(&player).and_then(|menu| Some((menu["window"].as_i64()? as i32, menu_name(menu["kind"].as_str()?)?)));
    if let Some((window, name)) = open {
        let Some(kind) = host.hub.registry_id("minecraft:menu", name) else {
            return false;
        };
        let Some(guest) = host.hub.guests.iter().find(|guest| guest.uuid == player) else {
            return false;
        };
        guest.send(&C::OpenScreen(play::OpenScreen { container: window, menu: kind, title }));
        pending_writes().entry(player).or_default().push(MenuWrite::Resend);
        return true;
    }
    let Some(guest) = host.hub.guests.iter_mut().find(|guest| guest.uuid == player) else {
        return false;
    };
    let Some((window, kind)) = guest.menu.as_ref().map(|menu| (menu.window, menu.kind)) else {
        return false;
    };
    guest.send(&C::OpenScreen(play::OpenScreen { container: window, menu: kind, title }));
    if let Some(menu) = guest.menu.as_mut() {
        menu.sent.clear();
    }
    true
}

/// `{ "op": "items_menu_property", "uuid", "id", "value" }`: `InventoryView.setProperty`: a data slot,
/// told to the client as the game tells it.
fn set_property(game: &mut Game<'_>, query: &Value) -> bool {
    let Some(player) = uuid(&query["uuid"]) else {
        return false;
    };
    let window = open_menus().get(&player).and_then(|menu| menu["window"].as_i64()).or_else(|| guest_menu(game, player).and_then(|menu| menu["window"].as_i64()));
    let (Some(window), Some(host)) = (window, game.lan.as_ref()) else {
        return false;
    };
    let Some(guest) = host.hub.guests.iter().find(|guest| guest.uuid == player) else {
        return false;
    };
    guest.send(&C::ContainerSetData(play::ContainerSetData { container: window as i32, id: query["id"].as_i64().unwrap_or(0) as i16, value: query["value"].as_i64().unwrap_or(0) as i16 }));
    true
}

/// Who trades with the merchant `mob`.
fn trader_of(mob: u64) -> Value {
    open_menus().iter().find(|(_, menu)| menu["mob"].as_u64() == Some(mob)).map_or(Value::Null, |(player, _)| json!(format!("{player:032x}")))
}

/// A pot's saved tag: `/data` doesn't reach it.
fn saved_tag(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| super::spill::saved_entity(&game.world, pos).map(|tag| super::commands::paper_entities::tag_json(&tag)))?
}

fn saved_merge(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let (patch, _) = parse_snbt(query["snbt"].as_str()?).ok()?;
    game.native_in_level(dimension(query), |game| {
        if &*game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name != "minecraft:decorated_pot" {
            return false;
        }
        let tag = super::spill::saved_entity(&game.world, pos).unwrap_or_else(|| Tag::compound([("id", Tag::String("minecraft:decorated_pot".into())), ("x", Tag::Int(pos[0])), ("y", Tag::Int(pos[1])), ("z", Tag::Int(pos[2]))]));
        super::spill::put_saved_entity(&mut game.world, pos, horizon_client::data_cmd::merged(&tag, &patch));
        game.mark_block_entity_updated((pos[0], pos[1], pos[2]));
        true
    })
}

/// `DecoratedPot.startWobble`: `DecoratedPotBlockEntity.wobble`'s block event.
fn pot_wobble(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let style = if query["style"].as_str() == Some("POSITIVE") { 0 } else { 1 };
    game.native_in_level(dimension(query), |game| {
        game.block_entity_event(pos, 1, style);
        game.lan_block_event(pos, 1, style);
        true
    })
}

/// `ChiseledBookshelf.getSlot`: the game's own sum, for a point on the block's front.
fn bookshelf_slot(game: &mut Game<'_>, query: &Value) -> Option<i32> {
    let [x, y, z] = position(&query["pos"])?;
    let at = |index: usize| query["at"][index].as_f64();
    let hit = Vec3::new(f64::from(x) + at(0)?, f64::from(y) + at(1)?, f64::from(z) + at(2)?);
    let state = state_of(game, query)?;
    let facing = game.pack.assets.registry.state(state)?.properties.get("facing").and_then(|name| direction(&name.to_ascii_uppercase()))?;
    Some(hit_slot(facing, facing, BlockPos::new(x, y, z), hit, 2, 3).map_or(-1, |slot| slot as i32))
}

/// `Lidded.open` and `close`, each once, whatever players do: `ContainerOpenersCounter` sees one opener more.
fn lid(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    static HELD: Mutex<Vec<(String, [i32; 3])>> = Mutex::new(Vec::new());
    let pos = position(&query["pos"])?;
    let open = query["open"].as_bool()?;
    let key = (dimension(query).to_owned(), pos);
    {
        let mut held = HELD.lock().unwrap_or_else(|e| e.into_inner());
        let known = held.iter().position(|row| *row == key);
        match (open, known) {
            (true, None) => held.push(key.clone()),
            (false, Some(index)) => {
                held.remove(index);
            }
            _ => return Some(false),
        }
    }
    game.native_in_level(&key.0, |game| {
        game.container_openers_changed((pos[0], pos[1], pos[2]), open);
        true
    })
}

fn lid_open(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let state = state_of(game, query)?;
    if game.pack.assets.registry.state(state).is_some_and(|known| known.properties.get("open").is_some_and(|open| open == "true")) {
        return Some(true);
    }
    game.native_in_level(dimension(query), |game| game.be_anims.chests.get(&pos).is_some_and(|lid| lid.should_be_open) || game.be_anims.shulkers.get(&pos).is_some_and(|box_| box_.open_count > 0))
}

/// `EntityBlockStorage.releaseEntities`, as a hive lets its bees out: the ids of the mobs that came.
fn hive_release(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| {
        let Some((_, facing)) = game.hive_state(pos) else {
            return json!([]);
        };
        let before: Vec<u32> = game.mobs.iter().map(|mob| mob.id).collect();
        game.empty_hive(pos, facing, horizon_client::beehive::Release::BeeReleased);
        Value::Array(game.mobs.iter().filter(|mob| !before.contains(&mob.id)).map(|mob| json!(mob.id)).collect())
    })
}

/// `EntityBlockStorage.addEntity`: a bee going in, as `BeeEnterHiveGoal` sends it.
fn hive_add(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let mob = query["id"].as_u64()? as u32;
    game.native_in_level(dimension(query), |game| {
        game.bee_enter_hive(mob, pos);
        true
    })
}

/// `{ "op": "items_conduit" }`: the frame as `ConduitBlockEntity.updateShape` finds it now.
fn conduit(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| {
        let (world, facts, registry) = (&game.world, game.res.facts(), game.pack.assets.registry);
        let frames = game.res.items().block_tags.get("minecraft:conduit_effect_block");
        let is_water = |at: [i32; 3]| facts.get(world.block(at[0], at[1], at[2])).fluid == horizon_client::blocks::Fluid::Water;
        let is_frame = |at: [i32; 3]| registry.state(world.block(at[0], at[1], at[2])).is_some_and(|state| frames.is_some_and(|tag| tag.contains(&state.name)));
        let check = ConduitServer::default().check(pos, 0, is_water, is_frame);
        let count = check.and_then(|check| check.frames);
        let mut blocks = Vec::new();
        for x in -2_i32..=2 {
            for y in -2_i32..=2 {
                for z in -2_i32..=2 {
                    let (ax, ay, az) = (x.abs(), y.abs(), z.abs());
                    let ring = (ax > 1 || ay > 1 || az > 1) && (x == 0 && (ay == 2 || az == 2) || y == 0 && (ax == 2 || az == 2) || z == 0 && (ax == 2 || ay == 2));
                    if count.is_some() && ring && is_frame([pos[0] + x, pos[1] + y, pos[2] + z]) {
                        blocks.push([pos[0] + x, pos[1] + y, pos[2] + z]);
                    }
                }
            }
        }
        json!({ "active": count.is_some(), "frames": count.unwrap_or(0), "blocks": blocks, "target": game.be_anims.conduit_servers.get(&pos).and_then(|known| known.target) })
    })
}

fn conduit_target(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let target = query["id"].as_u64().map(|id| id as u32);
    game.native_in_level(dimension(query), |game| {
        game.be_anims.conduit_servers.entry(pos).or_default().target = target;
        true
    })
}

/// `{ "op": "items_enchant", "stack", "levels", "treasure" }`: `EnchantmentHelper.enchantItem`.
fn enchant(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let stack = stack_from(&query["stack"])?;
    let levels = query["levels"].as_i64()? as i32;
    let items = game.res.items();
    let enchantments = std::sync::Arc::clone(&items.enchantments);
    let mut pool = enchantments.tag("minecraft:in_enchanting_table").to_vec();
    if query["treasure"].as_bool().unwrap_or(false) {
        pool.extend(enchantments.tag("minecraft:treasure").iter().cloned());
    }
    let list = enchantments.select(items, &mut game.world.random, &stack.item, levels, &pool);
    let mut out = if &*stack.item == "minecraft:book" { stack.transmute("minecraft:enchanted_book", stack.count) } else { stack };
    for (id, level) in list {
        out.enchant(&id, level);
    }
    Some(stack_value(&out))
}

/// The crafting recipe a grid makes, by id.
fn grid_recipe(game: &Game<'_>, query: &Value) -> Value {
    let grid: Vec<Option<ItemStack>> = query["grid"].as_array().into_iter().flatten().map(stack_from).collect();
    let size = if grid.len() > 4 { 3 } else { 2 };
    game.recipes().find(&grid, size).map_or(Value::Null, |(id, _)| json!(id))
}

/// `Jukebox.eject`: as a player's empty hand does it.
fn jukebox_eject(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| game.use_jukebox(pos, None))
}

/// `ItemStack.computeTooltipLines`, in plain text.
fn tooltip(game: &Game<'_>, query: &Value) -> Option<Value> {
    let stack = stack_from(&query["stack"])?;
    Some(json!(horizon_client::item_tooltip::lines(&stack, game.res.items().get(&stack.item), &|key| game.pack.names.display_get(key))))
}

// ---- What Paper keeps on a block entity and the game's own don't: one file beside the level ----

static EXTRAS: Mutex<Option<BTreeMap<String, Value>>> = Mutex::new(None);

fn extras_file(game: &Game<'_>) -> std::path::PathBuf {
    game.world_root.join("horizon-block-extras.json")
}

fn extras<R>(game: &Game<'_>, f: impl FnOnce(&mut BTreeMap<String, Value>) -> R) -> R {
    let mut guard = EXTRAS.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(|| serde_json::from_slice(&std::fs::read(extras_file(game)).unwrap_or_default()).unwrap_or_default()))
}

fn save_extras(game: &Game<'_>) {
    let text = extras(game, |all| serde_json::to_vec(all).unwrap_or_default());
    let _ = std::fs::write(extras_file(game), text);
}

fn block_key(dimension: &str, pos: [i32; 3]) -> String {
    format!("{}|{}|{}|{}", dimension.trim_start_matches("minecraft:"), pos[0], pos[1], pos[2])
}

/// The row a request names: a block by `pos`, or anything else by `key`.
fn extra_row(query: &Value) -> Option<String> {
    if let Some(key) = query["key"].as_str() {
        return Some(key.to_owned());
    }
    Some(block_key(dimension(query), position(&query["pos"])?))
}

fn extra_at(game: &Game<'_>, key: &str, field: &str) -> Value {
    extras(game, |all| all.get(key).map_or(Value::Null, |row| row[field].clone()))
}

fn extra_of_block(game: &Game<'_>, pos: [i32; 3], field: &str) -> Value {
    extra_at(game, &block_key(game.dim.id(), pos), field)
}

fn set_extra(game: &mut Game<'_>, key: &str, field: &str, value: Value) {
    let name = key.split('|').nth(1).and_then(|x| x.parse::<i32>().ok()).map(|x| {
        let mut parts = key.split('|').skip(2).filter_map(|n| n.parse::<i32>().ok());
        (x, parts.next().unwrap_or(0), parts.next().unwrap_or(0))
    });
    let block = name.map(|(x, y, z)| game.res.facts().get(game.world.block(x, y, z)).name.to_string());
    extras(game, |all| {
        let row = all.entry(key.to_owned()).or_insert_with(|| json!({}));
        match row.as_object_mut() {
            Some(fields) if value.is_null() => {
                fields.remove(field);
            }
            Some(fields) => {
                fields.insert(field.to_owned(), value);
                if let Some(block) = block {
                    fields.insert("block".to_owned(), json!(block));
                }
            }
            None => {}
        }
        if row.as_object().is_some_and(|fields| fields.keys().all(|key| key == "block")) {
            all.remove(key);
        }
    });
    save_extras(game);
}

/// Rows of blocks that have gone: a block entity's data goes with it, as a saved tag would.
fn prune_extras(game: &mut Game<'_>) {
    let here = format!("{}|", game.dim.id().trim_start_matches("minecraft:"));
    let rows: Vec<(String, [i32; 3], String)> = extras(game, |all| {
        all.iter()
            .filter(|(key, _)| key.starts_with(&here))
            .filter_map(|(key, row)| {
                let mut parts = key.split('|').skip(1).filter_map(|n| n.parse::<i32>().ok());
                Some((key.clone(), [parts.next()?, parts.next()?, parts.next()?], row["block"].as_str()?.to_owned()))
            })
            .collect()
    });
    let gone: Vec<String> = rows
        .into_iter()
        .filter(|(_, pos, block)| game.world.loaded(pos[0], pos[2]) && *game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name != **block)
        .map(|(key, _, _)| key)
        .collect();
    if !gone.is_empty() {
        extras(game, |all| gone.iter().for_each(|key| {
            all.remove(key);
        }));
        save_extras(game);
    }
}

/// Sensors with a range of their own: the game's entity has one number a sensor doesn't use.
fn apply_sensor_ranges(game: &mut Game<'_>) {
    let here = format!("{}|", game.dim.id().trim_start_matches("minecraft:"));
    let rows: Vec<([i32; 3], i32)> = extras(game, |all| {
        all.iter()
            .filter(|(key, _)| key.starts_with(&here))
            .filter_map(|(key, row)| {
                let range = row["listener_range"].as_i64()? as i32;
                let mut parts = key.split('|').skip(1).filter_map(|n| n.parse::<i32>().ok());
                Some(([parts.next()?, parts.next()?, parts.next()?], range))
            })
            .collect()
    });
    for (pos, range) in rows {
        if let Some(BlockEntity::Sculk(entity)) = game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2])) {
            entity.warning_level = range;
        }
    }
}

/// `Lockable`: true keeps the player out, with the game's message and sound.
pub(in crate::game) fn lock_refuses(game: &mut Game<'_>, host: &LanHost, conn: u32, pos: (i32, i32, i32)) -> bool {
    let at = [pos.0, pos.1, pos.2];
    let lock = extra_of_block(game, at, "lock");
    if lock.is_null() {
        return false;
    }
    let Some(guest) = host.hub.guest(conn) else {
        return false;
    };
    if guest.spectator() {
        return false;
    }
    let held = if game.lan_acting == Some(conn) { game.inventory.get(game.selected).cloned().flatten() } else { guest.inventory.get(guest.selected).cloned().flatten() };
    let opens = held.as_ref().is_some_and(|held| {
        if let Some(name) = lock["name"].as_str() {
            return held.custom_name().as_deref() == Some(name);
        }
        let seen = stack_value(held);
        seen["item"] == lock["item"] && lock["components"].as_object().into_iter().flatten().all(|(id, value)| seen["components"][id] == *value)
    });
    if opens {
        return false;
    }
    let block = game.res.facts().get(game.world.block(at[0], at[1], at[2])).name.trim_start_matches("minecraft:").to_owned();
    let content = Tag::compound([("translate", Tag::String("container.isLocked".into())), ("with", Tag::List(vec![Tag::compound([("translate", Tag::String(format!("block.minecraft.{block}").as_str().into()))])]))]);
    guest.send(&C::SystemChat(play::SystemChat { content, overlay: true }));
    game.audio.play("minecraft:block.chest.locked", Some(at.map(|value| f64::from(value) + 0.5)), 1.0, 1.0);
    true
}

/// A hive with a limit of its own. True when this took the bee's way in out of the game's hands.
pub(in crate::game) fn hive_enter(game: &mut Game<'_>, mob: u32, at: [i32; 3]) -> bool {
    static INSIDE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if INSIDE.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }
    let Some(limit) = extra_of_block(game, at, "max_bees").as_u64() else {
        return false;
    };
    let key = (at[0], at[1], at[2]);
    let Some(BlockEntity::Beehive(hive)) = game.world.block_entities.get_mut(&key) else {
        return false;
    };
    let count = hive.bees.len() as u64;
    if count >= limit {
        return true;
    }
    if count < 3 {
        return false;
    }
    // Past the game's three: it sees an empty hive for the moment the bee goes in.
    let waiting = std::mem::take(&mut hive.bees);
    INSIDE.store(true, std::sync::atomic::Ordering::Relaxed);
    game.bee_enter_hive(mob, at);
    INSIDE.store(false, std::sync::atomic::Ordering::Relaxed);
    if let Some(BlockEntity::Beehive(hive)) = game.world.block_entities.get_mut(&key) {
        let went_in = std::mem::replace(&mut hive.bees, waiting);
        hive.bees.extend(went_in);
    }
    true
}

/// `Campfire.stopCooking`: a slot held still, by taking back the tick the game is about to give it.
pub(in crate::game) fn campfire_hold(game: &mut Game<'_>, pos: (i32, i32, i32)) -> Value {
    let held = extra_of_block(game, [pos.0, pos.1, pos.2], "stop_cooking");
    if held.is_null() {
        return Value::Null;
    }
    let state = game.world.block(pos.0, pos.1, pos.2);
    let lit = game.pack.assets.registry.state(state).and_then(|state| state.properties.get("lit")).is_some_and(|lit| lit == "true");
    if lit && let Some(BlockEntity::Campfire(campfire)) = game.world.block_entities.get_mut(&pos) {
        for slot in 0..4 {
            if held[slot].as_bool() == Some(true) && campfire.items[slot].is_some() {
                campfire.progress[slot] -= 1;
            }
        }
    }
    Value::Null
}

/// `Beacon.setEffectRange`: true when the range here was used in the game's place.
pub(in crate::game) fn beacon_range(game: &mut Game<'_>, at: [i32; 3], beacon: &Beacon) -> bool {
    let Some(range) = extra_of_block(game, at, "effect_range").as_f64() else {
        return false;
    };
    let effect = |name: &Option<String>| name.as_deref().and_then(|name| potions::effect(name.trim_start_matches("minecraft:"))).map(|effect| effect.id);
    let Some(primary) = effect(&beacon.primary) else {
        return true;
    };
    let secondary = effect(&beacon.secondary);
    let amplifier = i32::from(beacon.levels >= 4 && secondary == Some(primary));
    let duration = (9 + beacon.levels * 2) * 20;
    let levels = beacon.levels;
    game.for_each_player(|game| {
        let Some(spot) = game.player.as_ref().map(|player| player.position) else {
            return;
        };
        let inside = spot.x >= f64::from(at[0]) - range && spot.x <= f64::from(at[0] + 1) + range && spot.z >= f64::from(at[2]) - range && spot.z <= f64::from(at[2] + 1) + range && spot.y >= f64::from(at[1]) - range;
        if !inside {
            return;
        }
        let ambient = |id, level| EffectInstance { ambient: true, ..EffectInstance::new(id, duration, level) };
        game.potions.effects.add(ambient(primary, amplifier));
        if let Some(other) = secondary.filter(|other| levels >= 4 && *other != primary) {
            game.potions.effects.add(ambient(other, 0));
        }
    });
    true
}

/// `ChiseledBookshelf`-style live view of what isn't a block: a player's inventory or ender chest, in
/// a menu over a block entity kept away from the world, as a chest minecart's is.
struct LiveView {
    viewer: u128,
    target: u128,
    ender: bool,
    key: (i32, i32, i32),
    dimension: String,
    last: Vec<Option<ItemStack>>,
}

static LIVE: Mutex<Vec<LiveView>> = Mutex::new(Vec::new());
static LIVE_NEXT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);

fn live_items(host: &LanHost, target: u128, ender: bool) -> Option<Vec<Option<ItemStack>>> {
    let guest = host.hub.guests.iter().find(|guest| guest.uuid == target)?;
    if ender {
        let mut items = guest.ender_items.clone();
        items.resize(27, None);
        return Some(items);
    }
    let mut items = guest.inventory.clone();
    items.resize(36, None);
    items.extend([guest.armor[3].clone(), guest.armor[2].clone(), guest.armor[1].clone(), guest.armor[0].clone(), guest.offhand.clone(), None, None, None, None]);
    Some(items)
}

fn live_write(host: &mut LanHost, target: u128, ender: bool, slot: usize, stack: Option<ItemStack>) {
    let Some(guest) = host.hub.guests.iter_mut().find(|guest| guest.uuid == target) else {
        return;
    };
    match (ender, slot) {
        (true, slot) => {
            if let Some(place) = guest.ender_items.get_mut(slot) {
                *place = stack;
            }
        }
        (false, 0..=35) => guest.inventory[slot] = stack,
        (false, 36..=39) => guest.armor[39 - slot] = stack,
        (false, 40) => guest.offhand = stack,
        _ => {}
    }
    if !ender && let Some(index) = host.hub.guests.iter().position(|guest| guest.uuid == target) {
        host.hub.send_inventory(index);
    }
}

fn open_live(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let viewer = uuid(&query["uuid"])?;
    let target = uuid(&query["target"])?;
    let ender = query["ender"].as_bool()?;
    let title = horizon_client::lan::data::component_nbt(&horizon_client_render::packs::component(&query["title"]).component());
    let mut host = game.lan.take()?;
    let answer = (|| {
        let (conn, dimension) = host.hub.guests.iter().find(|guest| guest.uuid == viewer).map(|guest| (guest.conn, guest.dimension.clone()))?;
        let items = live_items(&host, target, ender)?;
        game.lan_station_closed(&mut host, conn, None);
        game.lan_crafting_closed(&mut host, conn, None);
        game.guest_container_closed(&mut host, conn);
        let number = LIVE_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let key = (i32::MAX - 14, VEHICLE_KEY_Y, number);
        game.native_in_level(&dimension, |game| game.world.block_entities.insert(key, BlockEntity::Chest(items.clone())));
        let menu = if ender { "minecraft:generic_9x3" } else { "minecraft:generic_9x5" };
        // Not the block key as the menu's `pos`: a chest minecart's key closes the menu when no cart has that id.
        host.hub.open_container(conn, menu, title, (i32::MAX - 14, VEHICLE_KEY_Y + 1, number), Slots::Block(key), items.clone());
        if !ender {
            host.hub.hide_menu_slots(conn, vec![41, 42, 43, 44]);
        }
        LIVE.lock().unwrap_or_else(|e| e.into_inner()).push(LiveView { viewer, target, ender, key, dimension, last: items });
        Some(true)
    })();
    game.lan = Some(host);
    answer
}

fn sync_live(game: &mut Game<'_>, host: &mut LanHost) {
    let mut live = LIVE.lock().unwrap_or_else(|e| e.into_inner());
    let mut index = 0;
    while index < live.len() {
        let view = &mut live[index];
        let (key, dimension) = (view.key, view.dimension.clone());
        let viewer = host.hub.guests.iter().find(|guest| guest.uuid == view.viewer);
        let open = viewer.is_some_and(|guest| guest.menu.as_ref().is_some_and(|menu| menu.slots == Slots::Block(key)));
        let now = live_items(host, view.target, view.ender);
        let (true, Some(now)) = (open, now) else {
            if open && let Some(conn) = viewer.map(|guest| guest.conn) {
                host.hub.close_container(conn);
            }
            game.native_in_level(&dimension, |game| game.world.block_entities.remove(&key));
            live.remove(index);
            continue;
        };
        let shown: Vec<Option<ItemStack>> = game.native_in_level(&dimension, |game| game.world.block_entities.get(&key).map(|entity| entity.slots().to_vec()).unwrap_or_default()).unwrap_or_default();
        for slot in 0..shown.len().min(view.last.len()).min(now.len()) {
            if shown[slot] != view.last[slot] {
                // The viewer moved it: the owner's slot follows.
                view.last[slot].clone_from(&shown[slot]);
                live_write(host, view.target, view.ender, slot, shown[slot].clone());
            } else if now[slot] != view.last[slot] {
                // The owner moved it: the view follows, and the tick's sync tells the viewer.
                view.last[slot].clone_from(&now[slot]);
                let stack = now[slot].clone();
                game.native_in_level(&dimension, |game| {
                    if let Some(place) = game.world.block_entities.get_mut(&key).and_then(|entity| entity.slots_mut().get_mut(slot)) {
                        *place = stack;
                    }
                });
            }
        }
        index += 1;
    }
}

/// The bell's tick where nothing draws the level: the game runs it only with a renderer.
fn tick_bells(game: &mut Game<'_>) {
    if game.renderer.is_some() || game.remote.is_some() {
        return;
    }
    let rung: Vec<[i32; 3]> = game.be_anims.bells.iter().filter(|(_, bell)| bell.shaking || bell.resonating).map(|(pos, _)| *pos).collect();
    if rung.is_empty() {
        return;
    }
    let raiders = game.res.stations().enchantments.entity_tags.get("minecraft:raiders").cloned().unwrap_or_default();
    for pos in rung {
        let mobs = &game.mobs;
        let entity = |handle: u64| {
            let mob = mobs.iter().find(|mob| u64::from(mob.id) == handle)?;
            Some(BellEntity { position: [mob.position.x, mob.position.y, mob.position.z], alive: mob.alive() && !mob.removed, raider: raiders.contains(&mob.kind.id) })
        };
        let tick = game.be_anims.bells.entry(pos).or_default().tick(pos, entity);
        if tick.resonate {
            game.audio.play("minecraft:block.bell.resonate", Some(pos.map(|value| f64::from(value) + 0.5)), 1.0, 1.0);
        }
        if let Some((_, glow)) = tick.ended {
            for mob in game.mobs.iter_mut().filter(|mob| glow.contains(&u64::from(mob.id))) {
                mob.add_effect("minecraft:glowing", 60, 0);
            }
        }
    }
}

/// Once a tick, from the stations' tick: all this family's upkeep.
pub(in crate::game) fn tick(game: &mut Game<'_>, host: &mut LanHost) -> Value {
    static TICKS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    if TICKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 20 == 0 {
        prune_extras(game);
        apply_sensor_ranges(game);
    }
    tick_bells(game);
    sync_live(game, host);
    Value::Null
}

// ---- Calls the game keeps private to a module: a row in family/items.py runs them there ----

static SHRIEKS: Mutex<Vec<(i32, i32, i32)>> = Mutex::new(Vec::new());
static BLOOMS: Mutex<Vec<(i32, i32, i32)>> = Mutex::new(Vec::new());

/// Shriekers a plugin set off, for the row on `tick_sculk_entities`.
pub(in crate::game) fn take_shrieks() -> Vec<(i32, i32, i32)> {
    std::mem::take(&mut *SHRIEKS.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Catalysts a plugin made bloom.
pub(in crate::game) fn take_blooms() -> Vec<(i32, i32, i32)> {
    std::mem::take(&mut *BLOOMS.lock().unwrap_or_else(|e| e.into_inner()))
}

/// `SculkCatalyst.bloom`: the spreader takes the charge now, the block blooms when the row runs.
fn catalyst_bloom(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    let [ax, ay, az] = position(&query["at"])?;
    let charge = query["charge"].as_i64()? as i32;
    game.native_in_level(dimension(query), |game| {
        let Some(BlockEntity::Sculk(entity)) = game.world.block_entities.get_mut(&(x, y, z)) else {
            return false;
        };
        entity.spreader.add_cursors(BlockPos::new(ax, ay, az), charge);
        BLOOMS.lock().unwrap_or_else(|e| e.into_inner()).push((x, y, z));
        true
    })
}

fn try_shriek(query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    SHRIEKS.lock().unwrap_or_else(|e| e.into_inner()).push((x, y, z));
    Some(true)
}

// ---- A jukebox: the game's own tick starts and ends a song ----

fn jukebox_set(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    let start = query["start"].as_bool()?;
    game.native_in_level(dimension(query), |game| {
        let Some(BlockEntity::Jukebox(jukebox)) = game.world.block_entities.get_mut(&(x, y, z)) else {
            return false;
        };
        if start {
            if jukebox.record().is_none() || jukebox.playing {
                return false;
            }
            jukebox.playing = true;
            jukebox.ticks = 0;
        } else {
            // A song past its length ends at the next tick, with the stop the game tells everyone.
            jukebox.ticks = i64::MAX / 2;
        }
        true
    })
}

/// `Bell.ring`: as `BellBlock.attemptToRing`, from `face`, or the block's own facing.
fn bell_ring(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let face = query["face"].as_str().and_then(direction);
    let entity = query["entity"].is_u64();
    game.native_in_level(dimension(query), |game| {
        let state = game.world.block(pos[0], pos[1], pos[2]);
        if &*game.res.facts().get(state).name != "minecraft:bell" {
            return false;
        }
        let facing = game.pack.assets.registry.state(state).and_then(|state| state.properties.get("facing")).and_then(|name| direction(&name.to_ascii_uppercase()));
        let Some(side) = face.or(facing) else {
            return false;
        };
        let data = side as i32;
        game.block_entity_event(pos, 1, data);
        game.lan_block_event(pos, 1, data);
        game.audio.play("minecraft:block.bell.use", Some(pos.map(|value| f64::from(value) + 0.5)), 2.0, 1.0);
        game.post_block_game_event("minecraft:block_change", pos, entity.then_some(horizon_client::mobs::Who::Player), None);
        true
    })
}

// ---- A creaking heart ----

fn heart_creaking(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let [x, y, z] = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| match game.world.block_entities.get(&(x, y, z)) {
        Some(BlockEntity::CreakingHeart(heart)) => json!(heart.creaking.and_then(|(id, _)| id)),
        _ => Value::Null,
    })
}

fn heart_set(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    let id = query["id"].as_u64().map(|id| id as u32);
    game.native_in_level(dimension(query), |game| {
        let creaking = id.map(|id| (Some(id), game.uuid_for(id)));
        let Some(BlockEntity::CreakingHeart(heart)) = game.world.block_entities.get_mut(&(x, y, z)) else {
            return false;
        };
        heart.creaking = creaking;
        true
    })
}

const FACE_NAMES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
const FACE_STEPS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

fn shuffled_faces(game: &mut Game<'_>) -> [usize; 6] {
    let mut faces = [0, 1, 2, 3, 4, 5];
    for index in (1..6).rev() {
        let other = game.world.random.next_int_bound(index as i32 + 1).unwrap_or(0) as usize;
        faces.swap(index, other);
    }
    faces
}

/// `CreakingHeartBlockEntity.spreadResin`.
fn spread_resin(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    let dimension = dimension(query).to_owned();
    game.native_in_level(&dimension, |game| {
        let is_log = |game: &Game<'_>, at: [i32; 3]| {
            let name = game.res.facts().get(game.world.block(at[0], at[1], at[2])).name.clone();
            game.res.items().block_tags.get("minecraft:pale_oak_logs").is_some_and(|tag| tag.contains(&*name))
        };
        let mut seen: Vec<[i32; 3]> = Vec::new();
        let mut queue = std::collections::VecDeque::from([(pos, 0)]);
        while let Some((current, depth)) = queue.pop_front() {
            if seen.contains(&current) {
                continue;
            }
            seen.push(current);
            if is_log(game, current) {
                for face in shuffled_faces(game) {
                    let step = FACE_STEPS[face];
                    let at = [current[0] + step[0], current[1] + step[1], current[2] + step[2]];
                    let state = game.world.block(at[0], at[1], at[2]);
                    let facts = game.res.facts().get(state);
                    let side = FACE_NAMES[face ^ 1];
                    let text = if facts.name.as_ref() == "minecraft:air" || facts.name.as_ref() == "minecraft:cave_air" || facts.name.as_ref() == "minecraft:void_air" {
                        format!("minecraft:resin_clump[{side}=true]")
                    } else if facts.name.as_ref() == "minecraft:water" && facts.fluid_level == 8 && !facts.fluid_falling {
                        format!("minecraft:resin_clump[{side}=true,waterlogged=true]")
                    } else if facts.name.as_ref() == "minecraft:resin_clump" && game.pack.assets.registry.state(state).and_then(|known| known.properties.get(side)).is_some_and(|value| value != "true") {
                        let now = state_text(game, state);
                        now.replace(&format!("{side}=false"), &format!("{side}=true"))
                    } else {
                        continue;
                    };
                    if let Ok(placed) = parse_state(game.pack.assets.registry, &text)
                        && game.level_set(at, placed).is_ok()
                    {
                        return json!(at);
                    }
                }
            }
            if seen.len() >= 64 {
                return Value::Null;
            }
            if depth < 2 {
                for face in shuffled_faces(game) {
                    let step = FACE_STEPS[face];
                    let at = [current[0] + step[0], current[1] + step[1], current[2] + step[2]];
                    if is_log(game, at) {
                        queue.push_back((at, depth + 1));
                    }
                }
            }
        }
        Value::Null
    })
}

/// `CreakingHeartBlockEntity.spawnProtector`, then `setCreakingInfo`: five tries in the game's own box.
fn spawn_creaking(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    let dimension = dimension(query).to_owned();
    let where_ = dimension.clone();
    game.native_in_level(&where_, |game| {
        let (width, height) = game.res.mob_types().get("minecraft:creaking").map_or((0.9, 2.7), |kind| (f64::from(kind.width as f32), f64::from(kind.height as f32)));
        for _ in 0..5 {
            let x = pos[0] + game.world.random.next_int_bound(33).unwrap_or(16) - 16;
            let z = pos[2] + game.world.random.next_int_bound(33).unwrap_or(16) - 16;
            if !game.world.loaded(x, z) {
                continue;
            }
            let mut y = pos[1] + 8;
            let mut above = game.world.block(x, y, z);
            let mut found = None;
            for _ in 0..17 {
                y -= 1;
                let here = game.world.block(x, y, z);
                let facts = game.res.facts().get(here);
                let leaves = game.res.items().block_tags.get("minecraft:leaves").is_some_and(|tag| tag.contains(&*facts.name));
                let free = game.res.behaviours().collision_shape(above).is_some_and(|shape| shape.is_empty());
                if free && !leaves && facts.sturdy_full & 2 != 0 {
                    found = Some(y + 1);
                    break;
                }
                above = here;
            }
            let Some(y) = found else {
                continue;
            };
            let cells = (height.ceil() as i32).max(1);
            let reach = (width / 2.0).ceil() as i32;
            let mut clear = true;
            for dx in -reach..reach {
                for dz in -reach..reach {
                    for dy in 0..cells {
                        let state = game.world.block(x + dx, y + dy, z + dz);
                        clear &= game.res.behaviours().collision_shape(state).is_some_and(|shape| shape.is_empty()) && game.res.facts().get(state).fluid == horizon_client::blocks::Fluid::None;
                    }
                }
            }
            if !clear {
                continue;
            }
            let snbt = format!("{{id:\"minecraft:creaking\",Pos:[{:?}d,{:?}d,{:?}d],home_pos:[I;{},{},{}]}}", f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5, pos[0], pos[1], pos[2]);
            let made = game.native_spawn(&dimension, &snbt);
            let Some(id) = made["id"].as_u64() else {
                continue;
            };
            let uuid = game.uuid_for(id as u32);
            if let Some(BlockEntity::CreakingHeart(heart)) = game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2])) {
                heart.creaking = Some((Some(id as u32), uuid));
            }
            game.audio.play("minecraft:block.creaking_heart.spawn", Some(pos.map(|value| f64::from(value) + 0.5)), 1.0, 1.0);
            return json!(id);
        }
        Value::Null
    })
}

// ---- Stacks as bytes, as Java writes them: compressed NBT with the data version ----

fn stack_bytes(query: &Value) -> Option<Value> {
    let mut tag = horizon_client::save::item_to_nbt(&stack_from(&query["stack"])?);
    if let Tag::Compound(fields) = &mut tag {
        fields.insert("DataVersion".into(), Tag::Int(horizon_world::DATA_VERSION));
    }
    let raw = write_named(&NamedTag { name: "".into(), tag }, Limits::default()).ok()?;
    let packed = Compression::Gzip.encode(&raw).ok()?;
    Some(json!(packed))
}

fn stack_from_bytes(query: &Value) -> Option<Value> {
    let bytes: Vec<u8> = query["bytes"].as_array()?.iter().filter_map(|byte| byte.as_u64().map(|byte| byte as u8)).collect();
    let raw = Compression::Gzip.decode(&bytes, 64 << 20).ok()?;
    let (named, _) = read_named(&raw, Limits::default()).ok()?;
    let mut tag = named.tag;
    let version = match tag.get("DataVersion") {
        Some(Tag::Int(version)) => *version,
        _ => 0,
    };
    horizon_world::datafix::update(horizon_world::datafix::schema::Ty::ItemStack, &mut tag, version, horizon_world::DATA_VERSION);
    Some(horizon_client::save::item_from_nbt(&tag).map_or(Value::Null, |stack| stack_value(&stack)))
}

// ---- Menus a plugin opens ----

/// A plugin's GUI of any menu the game has: `Bukkit.createInventory(holder, InventoryType...)`.
fn open_gui(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let player = uuid(&query["uuid"])?;
    let menu = query["menu"].as_str()?;
    let items: Vec<Option<ItemStack>> = query["items"].as_array()?.iter().map(stack_from).collect();
    let title = horizon_client::lan::data::component_nbt(&horizon_client_render::packs::component(&query["title"]).component());
    let host = game.lan.as_mut()?;
    let conn = host.hub.guests.iter().find(|guest| guest.placed && guest.uuid == player)?.conn;
    let number = LIVE_NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    host.hub.open_container(conn, menu, title, (i32::MAX - 14, VEHICLE_KEY_Y + 1, number), Slots::Native, items.clone());
    let guest = host.hub.guests.iter_mut().find(|guest| guest.uuid == player)?;
    guest.native_menu = items;
    let window = guest.menu.as_ref()?.window;
    held_guis().push((player, window));
    Some(true)
}

/// A merchant's screen for a plugin: a trader in the world, or offers of the plugin's own.
fn open_merchant(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let player = uuid(&query["uuid"])?;
    let mut host = game.lan.take()?;
    let answer = (|| {
        let conn = host.hub.guests.iter().find(|guest| guest.placed && guest.uuid == player)?.conn;
        if let Some(mob) = query["mob"].as_u64() {
            let index = game.mobs.iter().position(|known| u64::from(known.id) == mob)?;
            game.as_guest(&mut host, conn, |game| game.interact_trader(index, "minecraft:air"))?.ok()?;
            let station = game.station.take()?;
            game.lan_station_opened(&mut host, conn, station);
            return Some(true);
        }
        let (tag, _) = parse_snbt(query["offers"].as_str()?).ok()?;
        let offers = horizon_client::entity_nbt::offers_from_nbt(&tag);
        let state = super::trading::MerchantState { session: Box::new(horizon_client::merchant::Session::new(offers, false)), scroll: 0, dragging: false, focused: None, mob: None, name: query["title"].as_str().map(str::to_owned) };
        game.lan_open_merchant(&mut host, conn, state);
        Some(true)
    })();
    game.lan = Some(host);
    answer
}

/// The menus' registry names by what the engine calls them.
fn menu_name(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "Anvil" => "minecraft:anvil",
        "Crafting" => "minecraft:crafting",
        "Smithing" => "minecraft:smithing",
        "Grindstone" => "minecraft:grindstone",
        "Stonecutter" => "minecraft:stonecutter",
        "Enchanting" => "minecraft:enchantment",
        "Beacon" => "minecraft:beacon",
        "Cartography" => "minecraft:cartography_table",
        "Loom" => "minecraft:loom",
        "Lectern" => "minecraft:lectern",
        "Merchant" => "minecraft:merchant",
        _ => return None,
    })
}

/// The name of the menu a player has open, as the game's registry has it.
fn menu_type(game: &Game<'_>, player: u128) -> Value {
    let open = open_menus().get(&player).and_then(|menu| menu["kind"].as_str().map(str::to_owned));
    if let Some(name) = open.as_deref().and_then(menu_name) {
        return json!(name);
    }
    let host = game.lan.as_ref();
    let guest = host.and_then(|host| host.hub.guests.iter().find(|guest| guest.uuid == player));
    guest.and_then(|guest| guest.menu.as_ref()).and_then(|menu| host?.hub.registry_name("minecraft:menu", menu.kind)).map_or(Value::Null, |name| json!(name))
}

/// `AnvilView.setMaximumRepairCost` and `bypassEnchantmentLevelRestriction`: the menu's own, as Java's are.
fn anvil_rule(query: &Value) -> Option<bool> {
    let player = uuid(&query["uuid"])?;
    let mut rules = ANVIL_RULES.lock().unwrap_or_else(|e| e.into_inner());
    let rule = rules.entry(player).or_insert((40, false));
    if let Some(limit) = query["max"].as_i64() {
        rule.0 = limit as i32;
    }
    if let Some(bypass) = query["bypass"].as_bool() {
        rule.1 = bypass;
    }
    drop(rules);
    pending_writes().entry(player).or_default().push(MenuWrite::Resend);
    Some(true)
}

/// `AnvilMenu.createResult` under a rule of the plugin's, over the game's sum.
fn apply_anvil_rule(player: u128, env: &horizon_client::menu::Env<'_>, menu: &mut Menu) {
    let Some((limit, bypass)) = ANVIL_RULES.lock().unwrap_or_else(|e| e.into_inner()).get(&player).copied() else {
        return;
    };
    if menu.kind() != Kind::Anvil || menu.slot(0).is_none() {
        return;
    }
    let hover = menu.slot(0).map(|stack| horizon_client::stations::hover_name(&|id| env.names.display_item(id), stack)).unwrap_or_default();
    horizon_client::stations::ANVIL_LIMIT.store(limit, std::sync::atomic::Ordering::Relaxed);
    let mut out = horizon_client::stations::anvil_result(env.enchantments, env.items, menu.slot(0), menu.slot(1), menu.anvil.name.as_deref(), &hover, menu.creative);
    horizon_client::stations::ANVIL_LIMIT.store(40, std::sync::atomic::Ordering::Relaxed);
    if bypass && let (Some(input), Some(addition), Some(result)) = (menu.slot(0), menu.slot(1), out.result.as_mut()) {
        let book = !addition.stored_enchantments().is_empty();
        for (id, right) in addition.crafting_enchantments() {
            let left = input.enchantment_level(id);
            let level = if left == *right { right + 1 } else { (*right).max(left) };
            let Some(max) = env.enchantments.get(id).map(|def| (def.max_level, def.anvil_cost)) else {
                continue;
            };
            if level > max.0 && result.enchantment_level(id) == max.0 {
                result.enchant(id, level);
                let fee = if book { (i64::from(max.1) / 2).max(1) } else { i64::from(max.1) };
                out.cost += (fee * i64::from(level - max.0)) as i32;
            }
        }
    }
    if out.only_renaming && out.cost >= limit {
        out.cost = limit - 1;
    }
    if out.cost >= limit && !menu.creative {
        out.result = None;
    }
    menu.anvil.only_renaming = out.only_renaming;
    menu.anvil.repair_count = out.repair_item_count_cost;
    menu.data[0] = out.cost;
    menu.set_slot(env, 2, out.result);
}

/// The player allowed to edit a sign: `SignBlockEntity.playerWhoMayEdit`.
fn sign_editor(game: &Game<'_>, query: &Value) -> Value {
    let (Some(pos), Some(host)) = (position(&query["pos"]), game.lan.as_ref()) else {
        return Value::Null;
    };
    let conn = host.play.iter().find(|(_, play)| play.sign_editing == Some(pos)).map(|(conn, _)| *conn);
    conn.and_then(|conn| host.hub.guest(conn)).map_or(Value::Null, |guest| json!(format!("{:032x}", guest.uuid)))
}

fn set_sign_editor(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let host = game.lan.as_mut()?;
    for play in host.play.values_mut() {
        if play.sign_editing == Some(pos) {
            play.sign_editing = None;
        }
    }
    if let Some(player) = uuid(&query["uuid"]) {
        let conn = host.hub.guests.iter().find(|guest| guest.uuid == player)?.conn;
        host.play.entry(conn).or_default().sign_editing = Some(pos);
    }
    Some(true)
}

/// A block entity's whole saved field in place of what it holds: `/data merge` only ever adds keys.
fn block_put(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let field = query["field"].as_str()?;
    let (value, _) = parse_snbt(query["snbt"].as_str()?).ok()?;
    game.native_in_level(dimension(query), |game| {
        let key = (pos[0], pos[1], pos[2]);
        let name = game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name.clone();
        let mut tag = horizon_client::save::encode_block_entity(key, game.world.block_entities.get(&key)?, &name);
        if let Tag::Compound(fields) = &mut tag {
            fields.insert(field.into(), value);
        }
        let (_, entity) = horizon_client::save::decode_block_entity(&tag)?;
        game.world.block_entities.insert(key, entity);
        game.mark_block_entity_updated(key);
        Some(true)
    })?
}

/// The collision boxes of a state: at `pos` when it's the world's, as a block's shape can hang on its neighbours.
fn collision_boxes(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let state = state_of(game, query)?;
    if let Some(shape) = game.res.behaviours().collision_shape(state) {
        let boxes: Vec<[f64; 6]> = shape.boxes().iter().map(|shape| {
            let (low, high) = (shape.min_position(), shape.max_position());
            [low.x, low.y, low.z, high.x, high.y, high.z]
        }).collect();
        return Some(json!(boxes));
    }
    let pos = position(&query["pos"])?;
    game.native_in_level(dimension(query), |game| {
        let boxes: Vec<[f64; 6]> = game.mob_world_base(&[]).state_collision_boxes(pos[0], pos[1], pos[2], state).iter().map(|shape| {
            let (low, high) = (shape.min_position(), shape.max_position());
            [low.x - f64::from(pos[0]), low.y - f64::from(pos[1]), low.z - f64::from(pos[2]), high.x - f64::from(pos[0]), high.y - f64::from(pos[1]), high.z - f64::from(pos[2])]
        }).collect();
        json!(boxes)
    })
}

/// `ItemStack.hurtAndBreak(amount, entity, null, true)`, on a copy: Unbreaking and the entity's events, then the wear.
fn stack_damage(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let mut stack = stack_from(&query["stack"])?;
    let amount = i32::try_from(query["amount"].as_i64()?).ok()?;
    let max = u32::try_from(query["max"].as_i64()?).ok()?;
    let entity = query["entity"].as_u64().map(|id| id as u32);
    let player = uuid(&query["uuid"]);
    if max == 0 || amount == 0 {
        return Some(stack_value(&stack));
    }
    let loss = if amount < 0 {
        amount
    } else {
        let amount = amount as u32;
        let worn = match (player, entity) {
            (Some(player), _) if game.lan_acting_uuid == Some(player) => game.wear(&stack, amount),
            (Some(player), _) => {
                let mut host = game.lan.take()?;
                let conn = host.hub.guests.iter().find(|guest| guest.uuid == player).map(|guest| guest.conn);
                let worn = conn.and_then(|conn| game.as_guest(&mut host, conn, |game| game.wear(&stack, amount)));
                game.lan = Some(host);
                worn?
            }
            (None, Some(id)) => {
                let loss = game.durability_loss(&stack, amount);
                super::paper_fam_ev_entity::item_wear(game, id, &stack, loss)
            }
            (None, None) => game.durability_loss(&stack, amount),
        };
        i32::try_from(worn).ok()?
    };
    if loss == 0 {
        return Some(stack_value(&stack));
    }
    // `setDamageValue` clamps, and a stack at its limit breaks.
    stack.damage = (i64::from(stack.damage) + i64::from(loss)).clamp(0, i64::from(max)) as u32;
    if stack.damage >= max {
        if stack.count == 1 && let Some(player) = player {
            let payload = json!({ "uuid": format!("{player:032x}"), "stack": stack_value(&stack), "broken": true });
            super::paper_bridge::on_refusable(game, "pl_item_break", payload);
        }
        stack.count -= 1;
        if stack.count == 0 {
            return Some(Value::Null);
        }
    }
    Some(stack_value(&stack))
}

/// An enchanting table's offers in the game's own random, or one seeded as a Java `Random` is.
fn enchant_seeded(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let stack = stack_from(&query["stack"])?;
    let levels = query["levels"].as_i64()? as i32;
    let items = game.res.items();
    let enchantments = std::sync::Arc::clone(&items.enchantments);
    let pool: Vec<std::sync::Arc<str>> = if query["treasure"].as_bool().unwrap_or(false) { enchantments.list.iter().map(|known| known.id.clone()).collect() } else { enchantments.tag("minecraft:in_enchanting_table").to_vec() };
    // CraftItemFactory: what the stack had goes first.
    let mut clean = stack.clone();
    if let Some(data) = clean.extra.as_mut() {
        data.enchantments.clear();
    }
    let list = match query["seed"].as_i64() {
        Some(seed) => enchantments.select(items, &mut LegacyRandom::new(seed), &clean.item, levels, &pool),
        None => enchantments.select(items, &mut game.world.random, &clean.item, levels, &pool),
    };
    let mut out = if &*clean.item == "minecraft:book" { clean.transmute("minecraft:enchanted_book", clean.count) } else { clean };
    for (id, level) in list {
        out.enchant(&id, level);
    }
    Some(stack_value(&out))
}

/// The smithing recipe three inputs make, by id.
fn smithing_recipe(game: &Game<'_>, query: &Value) -> Value {
    let slot = |index: usize| stack_from(&query["slots"][index]);
    let (template, base, addition) = (slot(0), slot(1), slot(2));
    let fits = |ingredient: &horizon_client::crafting::Ingredient, stack: &Option<ItemStack>| stack.as_ref().is_some_and(|stack| ingredient.test(stack));
    for recipe in &game.stations().recipes.smithing {
        match recipe {
            horizon_client::stations::Smithing::Transform { id, template: a, base: b, addition: c, .. } | horizon_client::stations::Smithing::Trim { id, template: a, base: b, addition: c, .. } => {
                if fits(a, &template) && fits(b, &base) && fits(c, &addition) {
                    return json!(id);
                }
            }
        }
    }
    Value::Null
}

/// What an effect does to an attribute: `MobEffect.addAttributeModifier`.
fn effect_attribute(id: &str) -> Value {
    if id.trim_start_matches("minecraft:") == "invisibility" {
        return json!({ "attribute": "minecraft:waypoint_transmit_range", "key": "minecraft:effect.waypoint_transmit_range_hide", "amount": -1.0, "operation": "add_multiplied_total" });
    }
    horizon_client::effects::effect_modifier(id.trim_start_matches("minecraft:")).map_or(Value::Null, |(attribute, key, amount, operation)| json!({ "attribute": attribute, "key": key, "amount": amount, "operation": operation }))
}

/// A tooltip's lines in full, as JSON text components, name first.
fn tooltip_components(game: &Game<'_>, query: &Value) -> Option<Value> {
    let stack = stack_from(&query["stack"])?;
    let lines = horizon_client::item_tooltip::lines(&stack, game.res.items().get(&stack.item), &|key| game.pack.names.display_get(key));
    let mut rows: Vec<Value> = lines.iter().map(|line| json!(line)).collect();
    let styled = horizon_client::item_tooltip::lore_texts(&stack);
    for (offset, line) in styled.iter().enumerate() {
        if let Some(place) = rows.get_mut(offset + 1) {
            *place = line.component();
        }
    }
    Some(Value::Array(rows))
}

/// A player's ender chest where the game has it now: in the game itself for whoever is acting.
fn ender_items(game: &Game<'_>, query: &Value) -> Option<Value> {
    let player = uuid(&query["uuid"])?;
    let guest = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| guest.uuid == player));
    let items = match guest {
        Some(guest) if game.lan_acting != Some(guest.conn) => &guest.ender_items,
        Some(_) => &game.ender_items,
        None if game.lan_acting_uuid == Some(player) => &game.ender_items,
        None => return None,
    };
    Some(items.iter().map(|stack| stack.as_ref().map_or(Value::Null, stack_value)).collect())
}

fn ender_set(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let player = uuid(&query["uuid"])?;
    let slot = usize::try_from(query["slot"].as_u64()?).ok().filter(|slot| *slot < 27)?;
    let stack = stack_from(&query["stack"]);
    let acting = game.lan_acting;
    let answer = |place: Option<&mut Option<ItemStack>>| {
        place.map(|place| *place = stack.clone()).is_some()
    };
    let conn = game.lan.as_ref().and_then(|host| host.hub.guests.iter().find(|guest| guest.uuid == player)).map(|guest| guest.conn);
    match conn {
        Some(conn) if acting != Some(conn) => {
            let guest = game.lan.as_mut()?.hub.guests.iter_mut().find(|guest| guest.conn == conn)?;
            if guest.ender_items.len() < 27 {
                guest.ender_items.resize(27, None);
            }
            Some(answer(guest.ender_items.get_mut(slot)))
        }
        Some(_) => Some(answer(game.ender_items.get_mut(slot))),
        None if game.lan_acting_uuid == Some(player) => Some(answer(game.ender_items.get_mut(slot))),
        None => None,
    }
}

// ---- A plugin's inventory holds what's put in it, as a Bukkit inventory does ----

/// The menus of `Bukkit.createInventory`, by player and window. A native mod's menu isn't here: what
/// it shows no click moves.
static HELD_GUIS: Mutex<Vec<(u128, i32)>> = Mutex::new(Vec::new());

fn held_guis() -> std::sync::MutexGuard<'static, Vec<(u128, i32)>> {
    HELD_GUIS.lock().unwrap_or_else(|e| e.into_inner())
}

/// `lan_actions::ItemRules`, for the same `menu::click`.
struct GuiRules<'a>(&'a horizon_client::items::Items);

impl horizon_client::lan::menu::Rules for GuiRules<'_> {
    fn items(&self) -> &horizon_client::items::Items {
        self.0
    }
    fn max_stack(&self, item: &str) -> u32 {
        self.0.get(item).map_or(64, |info| info.max_stack)
    }
    fn armor_index(&self, item: &str) -> Option<usize> {
        match self.0.get(item)?.equip_slot.as_deref()? {
            "head" => Some(0),
            "chest" => Some(1),
            "legs" => Some(2),
            "feet" => Some(3),
            _ => None,
        }
    }
}

type GuiClick = (u128, i32, i32, i32, i32);

/// The head of `run_native_menu_events`: the clicks in plugins' menus are answered here, the others
/// (native mods') stay for the game.
pub(in crate::game) fn gui_clicks(game: &mut Game<'_>) {
    let held = held_guis().clone();
    if held.is_empty() {
        return;
    }
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    let mut clicks: Vec<GuiClick> = Vec::new();
    host.hub.native_menu_events.retain(|event| match *event {
        horizon_client::lan::menu::NativeMenuEvent::Click { uuid, window, slot, button, mode } if held.contains(&(uuid, window)) => {
            clicks.push((uuid, window, slot, button, mode));
            false
        }
        _ => true,
    });
    for click in clicks {
        gui_click(game, click);
    }
    let Some(mut host) = game.lan.take() else {
        return;
    };
    // A menu that's gone hands back what its player had on the cursor.
    let gone: Vec<(u128, i32)> = held_guis().iter().copied().filter(|(uuid, window)| {
        !host.hub.guests.iter().any(|guest| guest.uuid == *uuid && guest.menu.as_ref().is_some_and(|menu| menu.window == *window && menu.slots == Slots::Native))
    }).collect();
    held_guis().retain(|row| !gone.contains(row));
    for (uuid, _) in gone {
        if let Some(index) = host.hub.guests.iter().position(|guest| guest.uuid == uuid) {
            game.guest_carried_back(&mut host, index);
        }
    }
    game.lan = Some(host);
}

/// `menu::click` on a plugin's menu: the slots are the guest's copy, which the SDK then follows.
fn gui_apply(host: &mut LanHost, items: &horizon_client::items::Items, index: usize, slot: i32, button: i32, mode: i32) -> Vec<ItemStack> {
    let guest = &mut host.hub.guests[index];
    let mut drag = guest.menu.as_mut().and_then(|menu| menu.drag.take());
    let hidden = guest.menu.as_ref().map_or_else(Vec::new, |menu| menu.hidden.clone());
    let mut shown = guest.native_menu.clone();
    let (drops, _) = {
        let mut view = horizon_client::lan::menu::View { guest: &mut *guest, container: Some(&mut shown), shulker: false, hidden: &hidden };
        horizon_client::lan::menu::click(&mut view, &mut drag, slot, button, mode, &GuiRules(items))
    };
    guest.native_menu = shown;
    if let Some(menu) = guest.menu.as_mut() {
        menu.drag = drag;
    }
    drops
}

/// `InventoryDragEvent`'s facts of the drag that ends now, tried on the guest and put back as it was.
fn gui_drag(host: &mut LanHost, items: &horizon_client::items::Items, index: usize, window: i32, button: i32, mode: i32) -> Option<Value> {
    let guest = &host.hub.guests[index];
    let drag = guest.menu.as_ref()?.drag.clone();
    let slots = drag.as_ref()?.1.clone();
    if slots.len() < 2 {
        return None;
    }
    let size = guest.menu.as_ref()?.size;
    let before = (guest.native_menu.clone(), guest.inventory.clone(), guest.carried.clone());
    let shown = |stack: &Option<ItemStack>| stack.as_ref().map_or(Value::Null, stack_value);
    let mut payload = json!({
        "kind": "drag", "uuid": format!("{:032x}", guest.uuid), "name": guest.name, "window": window, "type": "CHEST", "size": size,
        "items": before.0.iter().map(shown).collect::<Vec<_>>(), "cursor": shown(&before.2), "level": guest.dimension, "drag": button >> 2, "slots": slots,
    });
    gui_apply(host, items, index, -999, button, mode);
    let guest = &mut host.hub.guests[index];
    let placed: Vec<Value> = slots.iter().map(|raw| {
        let stack = if *raw < size {
            guest.native_menu.get(*raw).cloned().flatten()
        } else {
            let own = raw - size;
            guest.inventory.get(if own < 27 { 9 + own } else { own - 27 }).cloned().flatten()
        };
        json!([raw, shown(&stack)])
    }).collect();
    payload["placed"] = json!(placed);
    payload["cursor_after"] = shown(&guest.carried);
    (guest.native_menu, guest.inventory, guest.carried) = before;
    if let Some(menu) = guest.menu.as_mut() {
        menu.drag = drag;
    }
    Some(payload)
}

fn gui_click(game: &mut Game<'_>, (uuid, window, slot, button, mode): GuiClick) {
    let on_menu = |game: &Game<'_>| {
        let guest = game.lan.as_ref()?.hub.guests.iter().position(|guest| guest.uuid == uuid)?;
        let open = game.lan.as_ref()?.hub.guests[guest].menu.as_ref().filter(|menu| menu.window == window && menu.slots == Slots::Native)?;
        Some((guest, open.size))
    };
    let Some((_, size)) = on_menu(game) else {
        return;
    };
    if slot >= (size + 36) as i32 && slot != -999 {
        return;
    }
    let dragging = mode == 5;
    let ending = dragging && button & 3 == 2;
    let question = if dragging {
        let Some(mut host) = game.lan.take() else {
            return;
        };
        let items = game.res.items();
        let index = host.hub.guests.iter().position(|guest| guest.uuid == uuid);
        let payload = index.filter(|_| ending).and_then(|index| gui_drag(&mut host, items, index, window, button, mode));
        game.lan = Some(host);
        payload.map(|payload| ("pl_held", payload))
    } else {
        Some(("menu_click", json!({ "uuid": format!("{uuid:032x}"), "window": window, "slot": slot, "button": button, "mode": mode })))
    };
    if let Some((event, payload)) = question
        && super::paper_bridge::on_refusable(game, event, payload)
    {
        // Cancelled: the slots are as they were, and the hub already told the client so.
        if dragging
            && let Some(host) = game.lan.as_mut()
            && let Some(menu) = host.hub.guests.iter_mut().find(|guest| guest.uuid == uuid).and_then(|guest| guest.menu.as_mut())
        {
            menu.drag = None;
        }
        return;
    }
    // A listener may have closed it, or opened another.
    let Some((index, _)) = on_menu(game) else {
        return;
    };
    let Some(mut host) = game.lan.take() else {
        return;
    };
    let drops = gui_apply(&mut host, game.res.items(), index, slot, button, mode);
    if !dragging || ending {
        game.drop_guest_items(&host, index, drops);
        let conn = host.hub.guests[index].conn;
        host.hub.resend_menu(conn, window, &game.world);
    }
    let items: Vec<Value> = host.hub.guests[index].native_menu.iter().map(|stack| stack.as_ref().map_or(Value::Null, stack_value)).collect();
    game.lan = Some(host);
    if !dragging || ending {
        super::paper_bridge::on_refusable(game, "items_gui_changed", json!({ "uuid": format!("{uuid:032x}"), "items": items }));
    }
}

/// `LivingEntity.addEffect` in ticks, as a plugin asks it: `/effect` counts whole seconds.
fn effect_add(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let entity = query["entity"].as_u64()? as i32;
    let name = query["id"].as_str()?;
    let id: &'static str = horizon_client::brewing::effect(&format!("minecraft:{}", name.trim_start_matches("minecraft:")))?.id;
    let (duration, amplifier) = (query["duration"].as_i64()? as i32, query["amplifier"].as_i64()? as i32);
    let mut effect = EffectInstance::new(id, duration, amplifier);
    effect.ambient = query["ambient"].as_bool().unwrap_or(false);
    effect.visible = query["particles"].as_bool().unwrap_or(true);
    // A guest's own turn: they are the game's player now.
    if game.lan_acting_uuid.is_some() && game.lan_acting_uuid == uuid(&query["uuid"]) {
        return Some(game.add_player_effect(effect));
    }
    if let Some(mut host) = game.lan.take() {
        let conn = host.hub.guests.iter().find(|guest| guest.entity_id == entity).map(|guest| guest.conn);
        let done = conn.and_then(|conn| game.as_guest(&mut host, conn, |game| game.add_player_effect(effect.clone())));
        game.lan = Some(host);
        if done.is_some() {
            return done;
        }
    }
    game.native_in_level(dimension(query), |game| {
        let mob = game.mobs.iter_mut().find(|mob| mob.id as i32 == entity)?;
        let name = potions::namespaced(id);
        mob.add_effect(name, duration, amplifier);
        mob.extra.hidden_effects.retain(|hidden| *hidden != name);
        if !effect.visible {
            mob.extra.hidden_effects.push(name);
        }
        Some(true)
    }).flatten()
}

/// `InstrumentItem.getUseDuration`.
fn instrument_ticks(game: &Game<'_>, query: &Value) -> Option<i32> {
    let stack = stack_from(&query["stack"])?;
    Some(game.res.items().instrument(&stack)?.use_ticks())
}

/// Whose slots these are now: the game's for the player acting, the guest's otherwise. The hub is out of
/// the game's hands in a guest's turn, and only that guest is at hand then.
fn player_slot(game: &mut Game<'_>, player: u128, slot: usize, stack: Option<ItemStack>) -> Option<bool> {
    let acting = game.lan_acting_uuid == Some(player);
    if acting {
        match slot {
            0..=35 => game.inventory[slot] = stack,
            36..=39 => game.armor[39 - slot] = stack,
            40 => game.offhand = stack,
            _ => return None,
        }
        return Some(true);
    }
    let host = game.lan.as_mut()?;
    let index = host.hub.guests.iter().position(|guest| guest.uuid == player)?;
    let guest = &mut host.hub.guests[index];
    match slot {
        0..=35 => *guest.inventory.get_mut(slot)? = stack,
        36..=39 => guest.armor[39 - slot] = stack,
        40 => guest.offhand = stack,
        _ => return None,
    }
    host.hub.send_inventory(index);
    Some(true)
}

fn player_set(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let player = uuid(&query["uuid"])?;
    let slot = usize::try_from(query["slot"].as_u64()?).ok()?;
    player_slot(game, player, slot, stack_from(&query["stack"]))
}

/// `CraftInventory.addItem` over the 36 slots: what's similar first, then the empty ones. What's left.
fn player_add(game: &mut Game<'_>, query: &Value) -> Option<u32> {
    let player = uuid(&query["uuid"])?;
    let stack = stack_from(&query["stack"])?;
    let limit = game.res.items().get(&stack.item).map_or(64, |item| item.max_stack);
    let slots: Vec<Option<ItemStack>> = if game.lan_acting_uuid == Some(player) {
        game.inventory.iter().take(36).cloned().collect()
    } else {
        game.lan.as_ref()?.hub.guests.iter().find(|guest| guest.uuid == player)?.inventory.iter().take(36).cloned().collect()
    };
    let mut left = stack.count;
    let mut writes = Vec::new();
    for (slot, held) in slots.iter().enumerate() {
        if let Some(held) = held.as_ref().filter(|held| held.same_item_same_components(&stack) && held.count < limit) {
            let moved = (limit - held.count).min(left);
            left -= moved;
            writes.push((slot, Some(ItemStack { count: held.count + moved, ..held.clone() })));
            if left == 0 {
                break;
            }
        }
    }
    for (slot, held) in slots.iter().enumerate() {
        if left > 0 && held.is_none() {
            let moved = left.min(limit.max(99));
            left -= moved;
            writes.push((slot, Some(ItemStack { count: moved, ..stack.clone() })));
        }
    }
    for (slot, value) in writes {
        player_slot(game, player, slot, value)?;
    }
    Some(left)
}

/// `CraftRegionAccessor.setBiome`: the cell of a loaded chunk, marked for saving. No packet goes out.
fn set_biome(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let [x, y, z] = position(&query["pos"])?;
    let wanted = query["biome"].as_str()?;
    let wanted = if wanted.contains(':') { wanted.to_owned() } else { format!("minecraft:{}", wanted.to_ascii_lowercase()) };
    let target = game.res.biomes().list.iter().position(|biome| biome.name == wanted)?;
    game.native_in_level(dimension(query), |game| {
        let key = (x >> 4, z >> 4);
        let cell = ((y - game.world.bottom_y()) >> 2).clamp(0, game.world.layout().height() / 4 - 1);
        let chunk = game.world.chunks.get_mut(&key)?;
        std::sync::Arc::make_mut(chunk).set_biome_cell((x & 15) >> 2, cell, (z & 15) >> 2, target as u16);
        game.world.modified.insert(key);
        Some(true)
    }).flatten()
}

/// `CraftBlock.breakNaturally`: the tool's drops, the block's fluid left behind, and what ice and turtle
/// eggs do after.
fn block_break(game: &mut Game<'_>, query: &Value) -> Option<bool> {
    let pos = position(&query["pos"])?;
    let tool = query.get("tool").filter(|tool| !tool.is_null()).and_then(stack_from);
    let flag_of = |key: &str| query[key].as_bool().unwrap_or(false);
    let (effect, experience, force) = (flag_of("effect"), flag_of("experience"), flag_of("force"));
    game.native_in_level(dimension(query), |game| {
        let state = game.world.block(pos[0], pos[1], pos[2]);
        let (name, air, requires) = {
            let facts = game.res.facts().get(state);
            (facts.name.to_string(), &*facts.name == "minecraft:air" || &*facts.name == "minecraft:cave_air" || &*facts.name == "minecraft:void_air", facts.requires_tool)
        };
        let dropped = !air && (tool.is_none() || !requires || game.res.items().correct_for_drops(tool.as_ref(), &name));
        if dropped {
            // `Block.dropResources(..., null, tool, false)`: no entity in the loot context.
            horizon_items::items::LOOT_WITHOUT_ENTITY.store(true, std::sync::atomic::Ordering::Relaxed);
            let stacks = game.block_loot_at(pos, state, tool.as_ref());
            horizon_items::items::LOOT_WITHOUT_ENTITY.store(false, std::sync::atomic::Ordering::Relaxed);
            for stack in stacks {
                game.pop_resource(pos, stack);
            }
            if experience {
                let value = game.block_experience(&name);
                game.spawn_orbs(pos.map(|value| f64::from(value) + 0.5), value);
            }
        }
        if (dropped && effect) || (force && !air) {
            let fire = name == "minecraft:fire" || name == "minecraft:soul_fire";
            if fire {
                game.level_event_at(1009, pos, 0);
            } else {
                game.level_event_at(2014, pos, state.0 as i32);
            }
        }
        let at = BlockPos::new(pos[0], pos[1], pos[2]);
        let destroyed = game.with_level(|behaviours, level, ticks| behaviours.remove_block(level, ticks, at, false));
        if dropped {
            let held = tool.clone();
            game.with_level(|behaviours, level, ticks| {
                let prevents = held.as_ref().is_some_and(|tool| tool.enchantment_level("minecraft:silk_touch") > 0);
                behaviours.ice_destroyed_by_player(level, ticks, at, state, prevents);
                behaviours.turtle_egg_destroyed_by_player(level, ticks, at, state);
            });
        }
        destroyed && dropped
    })
}

// ---- Redstone at a block, as `CraftBlock` asks it ----

const SIGNAL_ORDER: [(Direction, [i32; 3]); 6] = [
    (Direction::Down, [0, -1, 0]),
    (Direction::Up, [0, 1, 0]),
    (Direction::North, [0, 0, -1]),
    (Direction::South, [0, 0, 1]),
    (Direction::West, [-1, 0, 0]),
    (Direction::East, [1, 0, 0]),
];

fn signal_at(game: &mut Game<'_>, pos: [i32; 3], side: Direction) -> i32 {
    let at = BlockPos::new(pos[0], pos[1], pos[2]);
    game.with_level(|behaviours, level, ticks| behaviours.signal_at(level, ticks, at, side))
}

/// `RedstoneWireBlock.POWER` of the state at `pos`, for those that have it.
fn power_property(game: &Game<'_>, pos: [i32; 3]) -> Option<i32> {
    let state = game.world.block(pos[0], pos[1], pos[2]);
    game.pack.assets.registry.state(state)?.properties.get("power")?.parse().ok()
}

fn indirectly_powered(game: &mut Game<'_>, pos: [i32; 3]) -> bool {
    let at = BlockPos::new(pos[0], pos[1], pos[2]);
    game.with_level(|behaviours, level, ticks| behaviours.has_neighbor_signal(level, ticks, at))
}

fn face_indirectly_powered(game: &mut Game<'_>, pos: [i32; 3], side: Direction, offset: [i32; 3]) -> bool {
    if signal_at(game, pos, side) > 0 {
        return true;
    }
    power_property(game, [pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]]).is_some_and(|power| power > 0)
}

/// `{ "op": "items_power", "kind": "direct" | "indirect" | "face" | "face_indirect" | "level", "face" }`:
/// `isBlockPowered`, `isBlockIndirectlyPowered`, `isBlockFacePowered`, `isBlockFaceIndirectlyPowered` and `getBlockPower`.
fn power(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let pos = position(&query["pos"])?;
    let kind = query["kind"].as_str()?.to_owned();
    let face = query["face"].as_str().and_then(direction);
    game.native_in_level(dimension(query), |game| {
        let at = BlockPos::new(pos[0], pos[1], pos[2]);
        let offset = face.and_then(|face| SIGNAL_ORDER.iter().find(|(side, _)| *side == face).map(|(_, offset)| *offset));
        match kind.as_str() {
            "direct" => Some(json!(game.with_level(|behaviours, level, ticks| behaviours.direct_signal_to_at(level, ticks, at)) > 0)),
            "indirect" => Some(json!(indirectly_powered(game, pos))),
            "face" => Some(json!(signal_at(game, pos, face?) > 0)),
            "face_indirect" => Some(json!(face_indirectly_powered(game, pos, face?, offset?))),
            "level" => {
                let mut power = 0;
                for (side, step) in SIGNAL_ORDER {
                    if face.is_none() || face == Some(side) {
                        let neighbour = [pos[0] + step[0], pos[1] + step[1], pos[2] + step[2]];
                        if signal_at(game, neighbour, side) > 0 && let Some(level) = power_property(game, neighbour) {
                            power = power.max(level);
                            if power == 15 {
                                return Some(json!(15));
                            }
                        }
                    }
                }
                if power != 0 {
                    return Some(json!(power));
                }
                let lit = match (face, offset) {
                    (Some(face), Some(offset)) => face_indirectly_powered(game, pos, face, offset),
                    _ => indirectly_powered(game, pos),
                };
                Some(json!(if lit { 15 } else { 0 }))
            }
            _ => None,
        }
    }).flatten()
}

// ---- The 1.12 numbers, built as `CraftLegacy` builds its maps ----

struct Legacy {
    by_state: std::collections::HashMap<horizon_world::registry::BlockStateId, (i32, u8)>,
    by_block: std::collections::HashMap<String, (i32, u8)>,
    to_state: std::collections::HashMap<(i32, u8), horizon_world::registry::BlockStateId>,
    to_block: std::collections::HashMap<(i32, u8), String>,
}

static LEGACY: std::sync::OnceLock<Legacy> = std::sync::OnceLock::new();

/// `CraftLegacy.whitelistedStates`: properties of the old blocks that no longer exist.
const LEGACY_DROPPED: [&str; 4] = ["explode", "check_decay", "decayable", "facing"];

fn legacy_tables(game: &Game<'_>) -> &'static Legacy {
    LEGACY.get_or_init(|| {
        let registry = game.pack.assets.registry;
        let mut tables = Legacy { by_state: Default::default(), by_block: Default::default(), to_state: Default::default(), to_block: Default::default() };
        for id in (0..256).filter(|id| !matches!(id, 253 | 254)) {
            for data in 0..16u8 {
                let mut tag = horizon_world::datafix::block_state_data::get_tag(id << 4 | i32::from(data));
                horizon_world::datafix::update(horizon_world::datafix::schema::Ty::BlockState, &mut tag, 100, horizon_world::DATA_VERSION);
                let Some(name) = horizon_client::save::text(tag.get("id")) else {
                    continue;
                };
                if name.contains("%%FILTER_ME%%") || name == "minecraft:air" {
                    continue;
                }
                let mut rows: Vec<(String, String)> = Vec::new();
                if let Some(Tag::Compound(properties)) = tag.get("properties") {
                    for (key, value) in properties {
                        if let Some(value) = horizon_client::save::text(Some(value)) {
                            rows.push((key.to_string_lossy().to_string(), value));
                        }
                    }
                }
                // A property the block no longer has is passed over; `facing` only when it has to be.
                let parsed = [0, 3, 4].iter().find_map(|left_out: &usize| {
                    let kept: Vec<String> = rows.iter().filter(|(key, _)| !LEGACY_DROPPED[..*left_out].contains(&key.as_str())).map(|(key, value)| format!("{key}={value}")).collect();
                    let text = if kept.is_empty() { name.clone() } else { format!("{name}[{}]", kept.join(",")) };
                    super::render_control::parse_state(registry, &text).ok()
                });
                let Some(state) = parsed else {
                    continue;
                };
                tables.to_state.insert((id, data), state);
                tables.by_state.entry(state).or_insert((id, data));
                tables.to_block.insert((id, data), name.clone());
                tables.by_block.entry(name).or_insert((id, data));
            }
        }
        tables
    })
}

static LEGACY_REGISTRY: std::sync::OnceLock<&'static horizon_world::registry::BlockRegistry> = std::sync::OnceLock::new();

/// The tables built while a game is at hand, for a plugin's generator on a chunk worker.
pub(in crate::game) fn legacy_ready(game: &Game<'_>) {
    legacy_tables(game);
    let _ = LEGACY_REGISTRY.set(game.pack.assets.registry);
}

/// `legacy_data` with no game: a block's old data value, from text.
pub(in crate::game) fn legacy_byte(text: &str) -> Option<u8> {
    let state = super::render_control::parse_state(LEGACY_REGISTRY.get()?, text).ok()?;
    let tables = LEGACY.get()?;
    tables.by_state.get(&state).or_else(|| tables.by_block.get(text.split('[').next()?)).map(|row| row.1)
}

/// `CraftLegacy.toLegacy(state)`: `[id, data]`, air's for a block the old game didn't have.
fn legacy_data(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let state = state_of(game, query)?;
    let name = game.res.facts().get(state).name.to_string();
    let tables = legacy_tables(game);
    let (id, data) = tables.by_state.get(&state).or_else(|| tables.by_block.get(&name)).copied().unwrap_or((0, 0));
    Some(json!([id, data]))
}

/// `CraftMagicNumbers.getBlock(material, data)`: the state of the old number the block's type goes by.
fn legacy_state(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let name = query["block"].as_str()?.split('[').next()?.to_owned();
    let default = super::render_control::parse_state(game.pack.assets.registry, &name).ok()?;
    let data = u8::try_from(query["data"].as_i64()?).ok()?;
    let tables = legacy_tables(game);
    let (id, _) = tables.by_state.get(&default).or_else(|| tables.by_block.get(&name)).copied().unwrap_or((0, 0));
    let state = tables.to_state.get(&(id, data)).copied().or_else(|| tables.to_block.get(&(id, data)).and_then(|block| super::render_control::parse_state(game.pack.assets.registry, block).ok()));
    Some(state.map(|state| state_text(game, state)).map_or_else(|| json!("minecraft:air"), |text| json!(text)))
}

// ---- Brewing recipes added at run time ----

fn mix_side(value: &Value) -> horizon_client::brewing::MixSide {
    let names = value["items"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect();
    let stacks = value["stacks"].as_array().into_iter().flatten().filter_map(stack_from).collect();
    horizon_client::brewing::MixSide { items: names, stacks }
}

/// `RecipeManager.addRecipe`: false for an id that's taken, where Java throws.
fn mix_add(game: &Game<'_>, query: &Value) -> Option<bool> {
    let id = query["key"].as_str()?.to_owned();
    let output = stack_from(&query["output"])?;
    let brewing = &game.stations().brewing;
    let mut mixes = horizon_client::brewing::CUSTOM_MIXES.lock().unwrap_or_else(|e| e.into_inner());
    let removed = horizon_client::brewing::REMOVED_MIXES.lock().unwrap_or_else(|e| e.into_inner());
    if mixes.iter().any(|mix| mix.recipe.id == id) || (brewing.recipes.iter().any(|recipe| recipe.id == id) && !removed.contains(&id)) {
        return Some(false);
    }
    let nothing = || horizon_client::brewing::PotionIngredient { items: Default::default(), potions: None };
    let recipe: &'static horizon_client::brewing::BrewingRecipe = Box::leak(Box::new(horizon_client::brewing::BrewingRecipe { id, input: nothing(), reagent: nothing(), output }));
    mixes.push(horizon_client::brewing::CustomMix { input: mix_side(&query["input"]), reagent: mix_side(&query["reagent"]), recipe });
    Some(true)
}

fn mix_remove(game: &Game<'_>, query: &Value) -> Option<bool> {
    let id = query["key"].as_str()?;
    let mut mixes = horizon_client::brewing::CUSTOM_MIXES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(index) = mixes.iter().position(|mix| mix.recipe.id == id) {
        mixes.remove(index);
        return Some(true);
    }
    let mut removed = horizon_client::brewing::REMOVED_MIXES.lock().unwrap_or_else(|e| e.into_inner());
    if game.stations().brewing.recipes.iter().any(|recipe| recipe.id == id) && !removed.iter().any(|known| known == id) {
        removed.push(id.to_owned());
        return Some(true);
    }
    Some(false)
}

// ---- `ItemStack.getTooltipLines` ----

pub(in crate::game) static TOOLTIP_ADVANCED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn tooltip_styled(game: &mut Game<'_>, query: &Value) -> Option<Value> {
    let stack = stack_from(&query["stack"])?;
    let creative = query["creative"].as_bool().unwrap_or(false);
    if !horizon_client::item_tooltip::TooltipDisplay::for_stack(&stack).name_visible(creative) {
        return Some(json!([]));
    }
    TOOLTIP_ADVANCED.store(query["advanced"].as_bool().unwrap_or(false), std::sync::atomic::Ordering::Relaxed);
    let lines = game.stack_tooltip_lines(&stack, false, creative);
    TOOLTIP_ADVANCED.store(false, std::sync::atomic::Ordering::Relaxed);
    let rows: Vec<Value> = lines.iter().map(|(text, color)| {
        let mut row = text.component();
        if !row.is_object() {
            row = json!({ "text": row });
        }
        if *color != u32::MAX && row.get("color").is_none() {
            // `hud::rgba`: red in the low byte.
            row["color"] = json!(format!("#{:02x}{:02x}{:02x}", color & 255, (color >> 8) & 255, (color >> 16) & 255));
        }
        row
    }).collect();
    Some(Value::Array(rows))
}

// ---- The extras of a block's life ----

static GONE: Mutex<Vec<[i32; 3]>> = Mutex::new(Vec::new());
static ANY_GONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A block entity went: `take_gone` drops its row, the level's name being at hand there.
pub(in crate::game) fn block_gone(pos: [i32; 3]) {
    GONE.lock().unwrap_or_else(|e| e.into_inner()).push(pos);
    ANY_GONE.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub(in crate::game) fn take_gone(game: &Game<'_>) {
    if !ANY_GONE.swap(false, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let gone = std::mem::take(&mut *GONE.lock().unwrap_or_else(|e| e.into_inner()));
    let dropped = extras(game, |all| gone.iter().fold(false, |any, pos| all.remove(&block_key(game.dim.id(), *pos)).is_some() || any));
    if dropped {
        save_extras(game);
    }
}

/// `BlockItem.updateCustomBlockEntityTag` for what the game's block entity doesn't keep.
pub(in crate::game) fn placed_extras(game: &Game<'_>, pos: [i32; 3], stack: &ItemStack) {
    let Some(row) = stack.data().and_then(|data| data.other("minecraft:block_entity_data")).and_then(|data| data.get("horizon:extras")) else {
        return;
    };
    let row = super::commands::paper_entities::tag_json(row);
    extras(game, |all| all.insert(block_key(game.dim.id(), pos), row));
    save_extras(game);
}

/// The id of the block entity a block makes, as a stack's `block_entity_data` has to name it.
fn be_id(game: &mut Game<'_>, query: &Value) -> Option<String> {
    let name = query["block"].as_str()?.split('[').next()?.to_owned();
    let entity = game.new_block_entity(&name)?;
    horizon_client::save::text(horizon_client::save::encode_block_entity((0, 0, 0), &entity, &name).get("id"))
}

/// `None` for an op that isn't this family's.
pub(in crate::game) fn query(game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    Some(match op {
        "items_block" => state_of(game, query).map_or(Value::Null, |state| block_facts(game, state)),
        "items_block_sturdy" => json!(sturdy(game, query).unwrap_or(false)),
        "items_block_drops" => drops(game, query).unwrap_or_else(|| json!([])),
        "items_block_tool" => tool_facts(game, query).unwrap_or(Value::Null),
        "items_break_speed" => break_speed(game, query).unwrap_or(Value::Null),
        "items_block_tick" => json!(tick_block(game, query).unwrap_or(false)),
        "items_block_survives" => json!(survives(game, query).unwrap_or(false)),
        "items_block_transform" => transform(game, query).unwrap_or(Value::Null),
        "items_item" => item_info(game, query["item"].as_str().unwrap_or_default()),
        "items_burn_time" => burn_time(game, query["item"].as_str().unwrap_or_default()),
        "items_block_states" => block_states(game, query["block"].as_str().unwrap_or_default()),
        "items_potion" => potion_info(query["id"].as_str().unwrap_or_default()),
        "items_effect" => effect_info(query["id"].as_str().unwrap_or_default()),
        "items_effect_by_number" => usize::try_from(query["id"].as_i64().unwrap_or(0) - 1).ok().and_then(|index| potions::all_effects().nth(index)).map_or(Value::Null, |effect| json!(potions::namespaced(effect.id))),
        "items_potion_by_effect" => {
            let wanted = query["id"].as_str().unwrap_or_default().trim_start_matches("minecraft:");
            let found = ["swiftness", "slowness", "turtle_master", "strength", "healing", "harming", "leaping", "regeneration", "fire_resistance", "water_breathing", "invisibility", "night_vision", "weakness", "poison", "slow_falling", "luck"].into_iter().find(|name| potions::potion(name).is_some_and(|def| def.effects.iter().any(|(effect, _, _)| *effect == wanted)));
            found.map_or(Value::Null, |name| json!(format!("minecraft:{name}")))
        }
        "items_potion_color" => potion_color(&query["effects"]),
        "items_bell" => bell(game, query).unwrap_or(Value::Null),
        "items_bell_ring" => json!(bell_ring(game, query).unwrap_or(false)),
        "items_jukebox" => jukebox(game, query).unwrap_or(Value::Null),
        "items_can_smelt" => json!(can_smelt(game, query).unwrap_or(false)),
        "items_dispense" => json!(dispense(game, query).unwrap_or(false)),
        "items_hive_sedated" => json!(hive_sedated(game, query).unwrap_or(false)),
        "items_snbt" => query["snbt"].as_str().and_then(|snbt| parse_snbt(snbt).ok()).map_or(Value::Null, |(tag, _)| super::commands::paper_entities::tag_json(&tag)),
        "items_saved" => saved_tag(game, query).unwrap_or(Value::Null),
        "items_saved_merge" => json!(saved_merge(game, query).unwrap_or(false)),
        "items_pot_wobble" => json!(pot_wobble(game, query).unwrap_or(false)),
        "items_bookshelf_slot" => json!(bookshelf_slot(game, query).unwrap_or(-1)),
        "items_lid" => json!(lid(game, query).unwrap_or(false)),
        "items_lid_open" => json!(lid_open(game, query).unwrap_or(false)),
        "items_hive_release" => hive_release(game, query).unwrap_or(Value::Null),
        "items_hive_add" => json!(hive_add(game, query).unwrap_or(false)),
        "items_conduit" => conduit(game, query).unwrap_or(Value::Null),
        "items_conduit_target" => json!(conduit_target(game, query).unwrap_or(false)),
        "items_enchant" => enchant(game, query).unwrap_or(Value::Null),
        "items_grid_recipe" => grid_recipe(game, query),
        "items_jukebox_eject" => json!(jukebox_eject(game, query).unwrap_or(false)),
        "items_tooltip" => tooltip(game, query).unwrap_or(Value::Null),
        "items_catalyst_bloom" => json!(catalyst_bloom(game, query).unwrap_or(false)),
        "items_extra" => extra_row(query).map_or(Value::Null, |key| extras(game, |all| all.get(&key).cloned().unwrap_or(Value::Null))),
        "items_extra_set" => match (extra_row(query), query["field"].as_str()) {
            (Some(key), Some(field)) => {
                set_extra(game, &key, field, query["value"].clone());
                json!(true)
            }
            _ => json!(false),
        },
        "items_craft_rest" => {
            let grid: Vec<Option<ItemStack>> = query["grid"].as_array().into_iter().flatten().map(stack_from).collect();
            let size = if grid.len() == 9 { 3 } else { 2 };
            let recipes = game.res.recipes();
            let rests = recipes.find(&grid, size).map(|(key, _)| recipes.remainders(key, &grid, size)).unwrap_or_default();
            rests.iter().map(|stack| stack.as_ref().map_or(Value::Null, stack_value)).collect()
        }
        "items_entity_key" => query["mob"].as_u64().map_or(Value::Null, |mob| {
            let words = game.uuid_for(mob as u32);
            json!(format!("entity|{words:?}"))
        }),
        "items_gui_hold" => match (uuid(&query["uuid"]), query["window"].as_i64()) {
            (Some(player), Some(window)) => {
                held_guis().push((player, window as i32));
                json!(true)
            }
            _ => json!(false),
        },
        "items_data_version" => json!(horizon_world::DATA_VERSION),
        "items_effect_add" => json!(effect_add(game, query).unwrap_or(false)),
        "items_instrument_ticks" => instrument_ticks(game, query).map_or(Value::Null, |ticks| json!(ticks)),
        "items_player_set" => json!(player_set(game, query).unwrap_or(false)),
        "items_player_add" => player_add(game, query).map_or(Value::Null, |left| json!(left)),
        "items_set_biome" => json!(set_biome(game, query).unwrap_or(false)),
        "items_block_break" => json!(block_break(game, query).unwrap_or(false)),
        "items_power" => power(game, query).unwrap_or(Value::Null),
        "items_legacy_data" => legacy_data(game, query).unwrap_or(Value::Null),
        "items_legacy_state" => legacy_state(game, query).unwrap_or(Value::Null),
        "items_mix_add" => json!(mix_add(game, query).unwrap_or(false)),
        "items_mix_remove" => json!(mix_remove(game, query).unwrap_or(false)),
        "items_mix_reset" => {
            horizon_client::brewing::CUSTOM_MIXES.lock().unwrap_or_else(|e| e.into_inner()).clear();
            horizon_client::brewing::REMOVED_MIXES.lock().unwrap_or_else(|e| e.into_inner()).clear();
            json!(true)
        }
        "items_tooltip_styled" => tooltip_styled(game, query).unwrap_or(Value::Null),
        "items_be_id" => be_id(game, query).map_or(Value::Null, |id| json!(id)),
        "items_open_live" => json!(open_live(game, query).unwrap_or(false)),
        "items_ender" => ender_items(game, query).unwrap_or(Value::Null),
        "items_ender_set" => json!(ender_set(game, query).unwrap_or(false)),
        "items_anvil_state" => uuid(&query["uuid"]).map_or(Value::Null, |player| {
            let (max, bypass) = ANVIL_RULES.lock().unwrap_or_else(|e| e.into_inner()).get(&player).copied().unwrap_or((40, false));
            json!({ "max": max, "bypass": bypass })
        }),
        "items_open_gui" => json!(open_gui(game, query).unwrap_or(false)),
        "items_open_merchant" => json!(open_merchant(game, query).unwrap_or(false)),
        "items_menu_type" => uuid(&query["uuid"]).map_or(Value::Null, |player| menu_type(game, player)),
        "items_anvil_rule" => json!(anvil_rule(query).unwrap_or(false)),
        "items_jukebox_set" => json!(jukebox_set(game, query).unwrap_or(false)),
        "items_heart_creaking" => heart_creaking(game, query).unwrap_or(Value::Null),
        "items_heart_set" => json!(heart_set(game, query).unwrap_or(false)),
        "items_heart_resin" => spread_resin(game, query).unwrap_or(Value::Null),
        "items_heart_spawn" => spawn_creaking(game, query).unwrap_or(Value::Null),
        "items_stack_bytes" => stack_bytes(query).unwrap_or(Value::Null),
        "items_stack_from_bytes" => stack_from_bytes(query).unwrap_or(Value::Null),
        "items_try_shriek" => json!(try_shriek(query).unwrap_or(false)),
        "items_sign_editor" => sign_editor(game, query),
        "items_sign_editor_set" => json!(set_sign_editor(game, query).unwrap_or(false)),
        "items_block_put" => json!(block_put(game, query).unwrap_or(false)),
        "items_collision" => collision_boxes(game, query).unwrap_or(Value::Null),
        "items_stack_damage" => stack_damage(game, query).unwrap_or(Value::Null),
        "items_enchant_seeded" => enchant_seeded(game, query).unwrap_or(Value::Null),
        "items_smithing_recipe" => smithing_recipe(game, query),
        "items_effect_attribute" => effect_attribute(query["id"].as_str().unwrap_or_default()),
        "items_tooltip_components" => tooltip_components(game, query).unwrap_or(Value::Null),
        "items_menu" => menu_state(game, query),
        "items_menu_set" => json!(menu_set(game, query)),
        "items_cursor" => json!(set_cursor(game, query)),
        "items_menu_title" => json!(set_title(game, query)),
        "items_menu_property" => json!(set_property(game, query)),
        "items_trader" => trader_of(query["mob"].as_u64().unwrap_or(0)),
        "items_potion" => potion_info(query["id"].as_str().unwrap_or_default()),
        "items_effect" => effect_info(query["id"].as_str().unwrap_or_default()),
        "items_effect_by_number" => usize::try_from(query["id"].as_i64().unwrap_or(0) - 1).ok().and_then(|index| potions::all_effects().nth(index)).map_or(Value::Null, |effect| json!(potions::namespaced(effect.id))),
        "items_potion_by_effect" => {
            let wanted = query["id"].as_str().unwrap_or_default().trim_start_matches("minecraft:");
            let found = ["swiftness", "slowness", "turtle_master", "strength", "healing", "harming", "leaping", "regeneration", "fire_resistance", "water_breathing", "invisibility", "night_vision", "weakness", "poison", "slow_falling", "luck"].into_iter().find(|name| potions::potion(name).is_some_and(|def| def.effects.iter().any(|(effect, _, _)| *effect == wanted)));
            found.map_or(Value::Null, |name| json!(format!("minecraft:{name}")))
        }
        "items_potion_color" => potion_color(&query["effects"]),
        "items_bell" => bell(game, query).unwrap_or(Value::Null),
        "items_bell_ring" => json!(bell_ring(game, query).unwrap_or(false)),
        "items_jukebox" => jukebox(game, query).unwrap_or(Value::Null),
        "items_can_smelt" => json!(can_smelt(game, query).unwrap_or(false)),
        "items_dispense" => json!(dispense(game, query).unwrap_or(false)),
        "items_hive_sedated" => json!(hive_sedated(game, query).unwrap_or(false)),
        "items_menu" => menu_state(game, query),
        "items_menu_set" => json!(menu_set(game, query)),
        "items_cursor" => json!(set_cursor(game, query)),
        "items_menu_title" => json!(set_title(game, query)),
        "items_menu_property" => json!(set_property(game, query)),
        "items_trader" => trader_of(query["mob"].as_u64().unwrap_or(0)),
        _ => return None,
    })
}
