# Family entity. Yours alone. Applied by fabric-port/tools/dim_patch.py, after the mixins: each text must be in its file once.
# `pub(super)` in `game::x` reaches `game::paper_fam_entity`: that's all most of these do.
GAME = 'horizon-client/src/game/'
MOBS = 'horizon-client/src/mobs/'
LIB = 'horizon-client/src/'

EDITS = [
    # ---- Fish hooks: `FishHook`'s wait, lure and angle, in place of the game's numbers.
    (GAME + 'fishing.rs', '''#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Flying,''', '''#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum State {
    Flying,'''),
    (GAME + 'fishing.rs', '''    velocity: Vec3,
    state: State,
    life: i32,
    on_ground: bool,
    nibble: i32,
    time_until_lured: i32,
    time_until_hooked: i32,
    fish_angle: f32,
    out_of_water: i32,
    open_water: bool,
    luck: i32,
    lure: i32,
    /// The position a server's update put it heading for.
    lerp: HookLerp,
}
''', '''    velocity: Vec3,
    pub(super) state: State,
    life: i32,
    on_ground: bool,
    pub(super) nibble: i32,
    pub(super) time_until_lured: i32,
    pub(super) time_until_hooked: i32,
    fish_angle: f32,
    out_of_water: i32,
    open_water: bool,
    luck: i32,
    pub(super) lure: i32,
    /// The position a server's update put it heading for.
    lerp: HookLerp,
    /// What a plugin set on this hook.
    pub(super) rules: HookRules,
}
'''),
    (GAME + 'fishing.rs', '''const SIZE: f64 = 0.25;
fn bounds(p: Vec3) -> Aabb {''', '''/// A hook's own numbers (`FishHook`): the game's, until a plugin sets them.
#[derive(Clone, Copy, Debug)]
pub(super) struct HookRules {
    pub(super) wait: (i32, i32),
    pub(super) lure_time: (i32, i32),
    pub(super) lure_angle: (f32, f32),
    pub(super) apply_lure: bool,
    pub(super) sky: bool,
    pub(super) rain: bool,
    pub(super) bite_chance: Option<f64>,
}
impl Default for HookRules {
    fn default() -> Self {
        Self { wait: (100, 600), lure_time: (20, 80), lure_angle: (0.0, 360.0), apply_lure: true, sky: true, rain: true, bite_chance: None }
    }
}
/// `Mth.nextInt`.
fn roll_between(r: &mut impl RandomSource, (min, max): (i32, i32)) -> i32 {
    if min >= max {
        min
    } else {
        min + r.next_int_bound(max - min + 1).unwrap_or(0)
    }
}
impl HookRules {
    /// The wait for a fish to appear, less the lure. A lure past the longest wait leaves one tick.
    pub(super) fn wait_ticks(&self, r: &mut impl RandomSource, lure: i32) -> i32 {
        let wait = roll_between(r, self.wait);
        match self.apply_lure {
            false => wait,
            true if lure >= self.wait.1 => 1,
            true => wait - lure,
        }
    }
}
const SIZE: f64 = 0.25;
fn bounds(p: Vec3) -> Aabb {'''),
    (GAME + 'fishing.rs', '                lure,\n', '                lure,\n                rules: HookRules::default(),\n'),
    (GAME + 'fishing.rs', '            lure: 0,\n', '            lure: 0,\n            rules: HookRules::default(),\n'),
    (GAME + 'fishing.rs', 'if self.world.random.next_float() < 0.25', 'if h.rules.rain && self.world.random.next_float() < 0.25'),
    (GAME + 'fishing.rs', 'if r.next_float() < 0.5 && !sky {', 'if h.rules.sky && r.next_float() < 0.5 && !sky {'),
    # These keep the game's `20 +` and `- h.lure` around them, which the mixins' formatting may wrap: a short text survives that.
    (GAME + 'fishing.rs', 'h.fish_angle = self.world.random.next_float() * 360.0;', 'h.fish_angle = if h.rules.lure_angle.0 >= h.rules.lure_angle.1 { h.rules.lure_angle.0 } else { self.world.random.next_float() * (h.rules.lure_angle.1 - h.rules.lure_angle.0) + h.rules.lure_angle.0 };'),
    (GAME + 'fishing.rs', 'self.world.random.next_int_bound(61).unwrap_or(0)', '(roll_between(&mut self.world.random, h.rules.lure_time) - 20)'),
    (GAME + 'fishing.rs', '100 + self.world.random.next_int_bound(501).unwrap_or(0)', '(h.rules.wait_ticks(&mut self.world.random, h.lure) + h.lure)'),
    (GAME + 'fishing.rs', '''    /// The hooks drawn here, with their owners' ids (0 for this player's own).''', '''    /// `FishingHook.pullEntity` of what the hook holds, the hook staying out.
    pub(super) fn pull_hooked(&mut self) -> bool {
        let (Some(hook), Some(p)) = (self.hook.as_ref(), self.player.as_ref()) else {
            return false;
        };
        let State::Hooked(id) = hook.state else {
            return false;
        };
        let d = Vec3::new(p.position.x - hook.position.x, p.position.y - hook.position.y, p.position.z - hook.position.z);
        let pulled = |v: Vec3| Vec3::new(v.x + d.x * 0.1, v.y + d.y * 0.1, v.z + d.z * 0.1);
        if let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) {
            m.velocity = pulled(m.velocity);
        } else if let Some(item) = self.dropped.iter_mut().find(|i| i.id == id) {
            item.velocity = pulled(item.velocity);
        } else if self.creatures.hook_targets.iter().any(|t| t.id == id) {
            self.creatures.hook_pulls.push((id, Vec3::new(d.x * 0.1, d.y * 0.1, d.z * 0.1)));
        } else {
            return false;
        }
        true
    }
    /// The hooks drawn here, with their owners' ids (0 for this player's own).'''),
    # ---- Fireworks, lightning, eyes of ender.
    (GAME + 'fireworks.rs', '''    life: i32,
    lifetime: i32,
    pub(super) explosions: Vec<FireworkExplosion>,''', '''    pub(super) life: i32,
    pub(super) lifetime: i32,
    pub(super) explosions: Vec<FireworkExplosion>,'''),
    (GAME + 'lightning.rs', '''    pub at: [f64; 3],
    life: i32,
    flashes: i32,
    seed: i64,
    visual_only: bool,''', '''    pub at: [f64; 3],
    pub(super) life: i32,
    pub(super) flashes: i32,
    seed: i64,
    pub(super) visual_only: bool,'''),
    (GAME + 'end.rs', '''    /// Thrown eyes of ender in flight.
    eyes: Vec<EyeOfEnder>,''', '''    /// Thrown eyes of ender in flight.
    pub(super) eyes: Vec<EyeOfEnder>,'''),
    (GAME + 'end.rs', '''/// The EyeOfEnder entity.
struct EyeOfEnder {
    /// Its entity id once a server's clients track it (0 until then).
    id: u32,
    position: glam::DVec3,
    previous: glam::DVec3,
    velocity: glam::DVec3,
    target: glam::DVec3,
    life: u32,
    survive: bool,
}''', '''/// The EyeOfEnder entity.
pub(super) struct EyeOfEnder {
    /// Its entity id once a server's clients track it (0 until then).
    pub(super) id: u32,
    pub(super) position: glam::DVec3,
    previous: glam::DVec3,
    pub(super) velocity: glam::DVec3,
    pub(super) target: glam::DVec3,
    pub(super) life: u32,
    pub(super) survive: bool,
    /// `DATA_ITEM_STACK` when a plugin set it: shown in the air, and what it drops.
    pub(super) item: Option<ItemStack>,
}'''),
    (GAME + 'end.rs', 'survive: false,', 'survive: false,\n item: None,'),
    (GAME + 'end.rs', 'survive,', 'survive,\n item: None,'),
    (GAME + 'end.rs', 'self.spawn_item(stack, at, [vx, 0.2, vz], 0);', 'self.spawn_item(eye.item.clone().unwrap_or(stack), at, [vx, 0.2, vz], 0);'),
    (GAME + 'lan_entities.rs', 'let mut e = state(id, "minecraft:eye_of_ender", position, 0.0, false);', '''let mut e = state(id, "minecraft:eye_of_ender", position, 0.0, false);
            if let Some(stack) = self.end.eyes.iter().find(|eye| eye.id == id).and_then(|eye| eye.item.as_ref()) {
                e.meta.push((8, serializer::ITEM_STACK, Value::Item(hub.stack(&Some(stack.clone())))));
            }'''),
    # ---- A player's cooldown and what is stuck in a body.
    (GAME + 'lan_actions.rs', '    xp_delay: u32,\n', '    pub(super) xp_delay: u32,\n'),
    (GAME + 'combat.rs', '''    pub(super) stingers: i32,
    remove_arrow_time: i32,
    remove_stinger_time: i32,
}''', '''    pub(super) stingers: i32,
    pub(super) remove_arrow_time: i32,
    pub(super) remove_stinger_time: i32,
}'''),
    # ---- The goat's ram, the sniffer's digging.
    (MOBS + 'goat.rs', '''    /// `LivingEntity.discardFriction`, set through the long jump.
    pub discard_friction: bool,
}
''', '''    /// `LivingEntity.discardFriction`, set through the long jump.
    pub discard_friction: bool,
    /// `Goat.ram`: the goal charges at `ram_target` when it next runs, cooldown or not.
    forced: bool,
}

/// `Goat.ram`: the ram activity, at once, at where the target stands.
pub fn force_ram(m: &mut Mob, at: Vec3) {
    let g = &mut m.extra.goat;
    g.ram_target = Some(at);
    g.candidate = None;
    if g.phase == Phase::Done {
        g.forced = true;
    } else {
        g.phase = Phase::RamStart;
        g.time = 0;
    }
}
'''),
    (MOBS + 'goat.rs', 'ram_cooldown(m) <= 0', 'ram_cooldown(m) <= 0 || m.extra.goat.forced'),
    (MOBS + 'goat.rs', '        m.extra.goat.phase = Phase::Prepare;', '''        if std::mem::take(&mut m.extra.goat.forced) {
            m.extra.goat.phase = Phase::RamStart;
            m.extra.goat.time = 0;
            lower_head(m, cx.events, true);
            return;
        }
        m.extra.goat.phase = Phase::Prepare;'''),
    (MOBS + 'animal_brain.rs', 'fn can_dig(m: &mut Mob, w: &MobWorld) -> bool {', 'pub fn can_dig(m: &mut Mob, w: &MobWorld) -> bool {'),
    (MOBS + 'animal_brain.rs', 'fn dig_position(m: &mut Mob, w: &MobWorld) -> Option<[i32; 3]> {', 'pub fn dig_position(m: &mut Mob, w: &MobWorld) -> Option<[i32; 3]> {'),
    # ---- A villager's trades.
    (GAME + 'trading.rs', '    fn offers_from_set(', '    pub(super) fn offers_from_set('),
    (GAME + 'trading.rs', '    /// `AbstractVillager.getOffers`: generated on first use.', '''    /// The offers of a villager's trade set at `level`, rolled as a level-up rolls them.
    pub(super) fn offers_at_level(&mut self, index: usize, level: i32) -> Vec<MerchantOffer> {
        let m = &self.mobs[index];
        let variant = format!("minecraft:{}", VILLAGER_TYPES[m.extra.variant as usize % VILLAGER_TYPES.len()]);
        let origin = [m.position.x, m.position.y, m.position.z];
        let set = merchant::Trades::set_for(PROFESSIONS[m.extra.variant2 as usize % PROFESSIONS.len()], level);
        if self.stations().trades.sets.contains_key(&set) {
            self.offers_from_set(&set, &variant, origin)
        } else {
            Vec::new()
        }
    }
    /// `AbstractVillager.getOffers`: generated on first use.'''),
    # ---- What a plugin set on a mob: `horizon:plugin` in its saved compound, with a copy here that the library reads.
    (LIB + 'mobs.rs', '''    pub piglin: piglin::Memories,
}
impl Extra {''', '''    pub piglin: piglin::Memories,
    /// What a plugin set past the game's own fields. Saved in the mob's compound as `horizon:plugin`.
    pub plugin: Option<Box<horizon_world::nbt::Tag>>,
    /// `Creeper.explosionRadius`, when it isn't the 3 the game hard-codes (`ExplosionRadius`).
    pub explosion_radius: Option<i32>,
    /// `AbstractHorse.temper`: `setMaxDomestication` takes it past a byte.
    pub temper: i32,
    /// `Mob.lootTableSeed`: 0 rolls from the world's sequence.
    pub loot_seed: i64,
    /// `LivingEntity.removeArrowTime` and `removeStingerTime`.
    pub arrow_time: i32,
    pub stinger_time: i32,
    /// `EnderMan.DATA_STARED_AT`.
    pub stared_at: bool,
    /// `LivingEntity.collidableExemptions`, as ids: both ways round, rebuilt each tick.
    pub collide_exempt: Vec<u32>,
    pub exempt_players: Vec<[i32; 4]>,
}
impl Extra {
    /// A number a plugin set: a flag is 0 or 1.
    pub fn plugin_number(&self, key: &str) -> Option<f64> {
        match self.plugin.as_deref()?.get(key)? {
            horizon_world::nbt::Tag::Double(bits) => Some(f64::from_bits(*bits)),
            _ => None,
        }
    }
    pub fn plugin_flag(&self, key: &str) -> Option<bool> {
        self.plugin_number(key).map(|value| value != 0.0)
    }
    /// Whether a list a plugin set holds `item`.
    pub fn plugin_lists(&self, key: &str, item: &str) -> bool {
        matches!(self.plugin.as_deref().and_then(|tag| tag.get(key)), Some(horizon_world::nbt::Tag::List(rows)) if rows.iter().any(|row| matches!(row, horizon_world::nbt::Tag::String(text) if text.0 == item.encode_utf16().collect::<Vec<_>>())))
    }
    /// The server's part of `LivingEntity.tick`: one in a count's worth of `20 * (30 - count)` ticks falls out.
    pub fn tick_stuck(&mut self) {
        fn decay(count: &mut i32, time: &mut i32) {
            if *count > 0 {
                if *time <= 0 {
                    *time = 20 * (30 - *count);
                }
                *time -= 1;
                if *time <= 0 {
                    *count -= 1;
                }
            }
        }
        decay(&mut self.arrow_count, &mut self.arrow_time);
        decay(&mut self.stinger_count, &mut self.stinger_time);
    }
    /// `Entity.getMaxAirSupply`, for the mobs that don't have their own.
    pub fn max_air(&self) -> i32 {
        self.plugin_number("max_air").map_or(300, |ticks| ticks as i32)
    }'''),
    (LIB + 'entity_nbt.rs', '''pub fn mob_from_nbt(
    tag: &Tag,
    types: &HashMap<String, Arc<MobType>>,
    id: u32,
    registry: Option<&BlockRegistry>,
) -> Option<Mob> {
''', '''pub fn mob_from_nbt(
    tag: &Tag,
    types: &HashMap<String, Arc<MobType>>,
    id: u32,
    registry: Option<&BlockRegistry>,
) -> Option<Mob> {
    let mut mob = mob_from_saved(tag, types, id, registry)?;
    mob.extra.plugin = tag.get("horizon:plugin").cloned().map(Box::new);
    mob.extra.explosion_radius = number(tag.get("ExplosionRadius")).map(|radius| radius as i32);
    Some(mob)
}
fn mob_from_saved(
    tag: &Tag,
    types: &HashMap<String, Arc<MobType>>,
    id: u32,
    registry: Option<&BlockRegistry>,
) -> Option<Mob> {
'''),
    (MOBS + 'hostile.rs', 'let radius = if m.extra.charged { 6.0 } else { 3.0 };', 'let radius = m.extra.explosion_radius.unwrap_or(3) as f32 * if m.extra.charged { 2.0 } else { 1.0 };'),
    (MOBS + 'equine.rs', 'let max = max_temper(m.species());', 'let max = m.extra.plugin_number("max_domestication").map_or(max_temper(m.species()), |max| max as i32);'),
    (GAME + 'mounts.rs', 'if temper > 0 && (used || !m.extra.tame) && m.extra.variant3 < 100 {', 'let limit = m.extra.plugin_number("max_domestication").map_or(100, |max| max as i32);\n        if temper > 0 && (used || !m.extra.tame) && m.extra.temper < limit {'),
    (GAME + 'mounts.rs', 'm.extra.variant3 = (m.extra.variant3 + temper).min(100);', 'm.extra.temper = (m.extra.temper + i32::from(temper)).min(limit);'),
    (GAME + 'llamas.rs', 'if (used || !m.extra.tame) && i32::from(m.extra.variant3) < llama::MAX_TEMPER {', 'let limit = m.extra.plugin_number("max_domestication").map_or(llama::MAX_TEMPER, |max| max as i32);\n        if (used || !m.extra.tame) && m.extra.temper < limit {'),
    (GAME + 'llamas.rs', 'm.extra.variant3 = (i32::from(m.extra.variant3) + temper).min(llama::MAX_TEMPER) as u8;', 'm.extra.temper = (m.extra.temper + temper).min(limit);'),
    # Synched data the game doesn't send: a horse's open mouth, an enderman stared at, a piglin charging.
    (GAME + 'lan.rs', '| if grazes && e.counter > 0 { 32 } else { 0 };', '| if grazes && e.counter > 0 { 32 } else { 0 }\n                | if e.plugin_flag("mouth_open") == Some(true) { 64 } else { 0 };'),
    (GAME + 'lan.rs', 'defaults.push((16, serializer::OPTIONAL_BLOCK_STATE, Value::Id(0)));', '''defaults.push((16, serializer::OPTIONAL_BLOCK_STATE, Value::Id(0)));
            if e.stared_at {
                meta.push((18, serializer::BOOLEAN, Value::Bool(true)));
            }
            defaults.push((18, serializer::BOOLEAN, Value::Bool(false)));'''),
    (GAME + 'lan.rs', '''        "minecraft:piglin" => {
            if e.flag2 {''', '''        "minecraft:piglin" => {
            if e.plugin_flag("charging") == Some(true) {
                meta.push((18, serializer::BOOLEAN, Value::Bool(true)));
            }
            defaults.push((18, serializer::BOOLEAN, Value::Bool(false)));
            if e.flag2 {'''),
    (MOBS + 'enderman.rs', '        m.extra.counter = m.age as i32;\n', '        m.extra.counter = m.age as i32;\n        m.extra.stared_at = true;\n'),
    (MOBS + 'enderman.rs', '        self.pending = false;\n        m.ai.target = None;\n', '        self.pending = false;\n        m.ai.target = None;\n        m.extra.stared_at = false;\n'),
    (MOBS + 'cube.rs', '&& (m.on_ground || m.is_in_liquid() || m.has_effect("minecraft:levitation"))', '&& (m.on_ground || m.is_in_liquid() || m.has_effect("minecraft:levitation"))\n            && m.extra.plugin_flag("wander") != Some(false)'),
    (MOBS + 'cube.rs', '''    fn can_use(&mut self, _: &mut Mob, _: &mut Cx) -> bool {
        true
    }
    fn tick(&mut self, m: &mut Mob, _: &mut Cx) {
        set_wanted(m, 1.0);''', '''    fn can_use(&mut self, m: &mut Mob, _: &mut Cx) -> bool {
        m.extra.plugin_flag("wander") != Some(false)
    }
    fn tick(&mut self, m: &mut Mob, _: &mut Cx) {
        set_wanted(m, 1.0);'''),
    (MOBS + 'ai.rs', '                m.air = 300;\n', '                m.air = m.extra.max_air();\n'),
    (MOBS + 'ai.rs', '} else if m.air < 300 {', '} else if m.air < m.extra.max_air() {'),
    (MOBS + 'ai.rs', 'm.air = (m.air + 4).min(300);', 'm.air = (m.air + 4).min(m.extra.max_air());'),
    (MOBS + 'ai.rs', '&& (self.species() != Species::Creaking || super::creaking::can_move(self))', '&& (self.species() != Species::Creaking || super::creaking::can_move(self))\n            && self.extra.plugin_flag("collidable") != Some(false)'),
    (MOBS + 'ai.rs', "    // EntitySelector.pushableBy(this): the teams' collision rules.\n", "    targets.retain(|(who, ..)| !matches!(who, Who::Mob(id) if m.extra.collide_exempt.contains(id)));\n    if w.player.as_ref().is_some_and(|p| crate::paper_hub::no_collide(p.uuid) || m.extra.exempt_players.contains(&p.uuid)) {\n        targets.retain(|(who, ..)| *who != Who::Player);\n    }\n    // EntitySelector.pushableBy(this): the teams' collision rules.\n"),
    (MOBS + 'ai.rs', 'if w.difficulty.effective == 0.0 && !super::spawn_rules::allowed_in_peaceful(m.species()) {', 'if w.difficulty.effective == 0.0 && m.extra.plugin_flag("peaceful").unwrap_or(!super::spawn_rules::allowed_in_peaceful(m.species())) {'),
    (MOBS + 'trader.rs', '!cx.w.daylight && !invisible(m)', 'm.extra.plugin_flag("drink_potion") != Some(false) && !cx.w.daylight && !invisible(m)'),
    (MOBS + 'trader.rs', 'cx.w.daylight && invisible(m)', 'm.extra.plugin_flag("drink_milk") != Some(false) && cx.w.daylight && invisible(m)'),
    # `Frictional`: TRUE keeps the friction a long jump drops, FALSE drops it.
    (MOBS + 'living.rs', 'if super::breeze::discards_friction(self)', 'if self.extra.plugin_flag("friction") == Some(false)\n            || (self.extra.plugin_flag("friction") != Some(true) && super::breeze::discards_friction(self))'),
    (MOBS + 'living.rs', '|| self.extra.frog.discard_friction', '|| (self.extra.plugin_flag("friction") != Some(true) && self.extra.frog.discard_friction)'),
    (MOBS + 'living.rs', '|| self.extra.goat.discard_friction', '|| (self.extra.plugin_flag("friction") != Some(true) && self.extra.goat.discard_friction)'),
    # ---- Minecarts.
    (GAME + 'minecarts.rs', 'let base = if self.cart_in_water(c) { 0.2 } else { 0.4 };', '''let limit = crate::game::paper_fam_entity::plugin_number(self, c.id, "max_speed").unwrap_or(0.4);
        let base = if self.cart_in_water(c) { limit / 2.0 } else { limit };'''),
    (GAME + 'minecarts.rs', 'fn max_speed_new(&self, c: &Minecart) -> f64 {', '''fn max_speed_new(&self, c: &Minecart) -> f64 {
        if let Some(limit) = crate::game::paper_fam_entity::plugin_number(self, c.id, "max_speed") {
            return if self.cart_in_water(c) { limit / 2.0 } else { limit };
        }'''),
    (GAME + 'minecarts.rs', 'fn natural_slowdown(&self, c: &mut Minecart, ridden: bool, factor: f64) -> Vec3 {', '''fn natural_slowdown(&self, c: &mut Minecart, ridden: bool, factor: f64) -> Vec3 {
        if crate::game::paper_fam_entity::plugin_number(self, c.id, "friction") == Some(0.0) {
            return Vec3::new(c.velocity.x, 0.0, c.velocity.z);
        }'''),
    (GAME + 'minecarts.rs', 'let f = if ridden { 0.997 } else { factor };', 'let f = if ridden || crate::game::paper_fam_entity::plugin_number(self, c.id, "slow_when_empty") == Some(0.0) { 0.997 } else { factor };'),
    (GAME + 'minecarts.rs', '''    fn come_off_track(&self, c: &mut Minecart) {
''', '''    fn come_off_track(&self, c: &mut Minecart) {
        // `Minecart.setDerailedVelocityMod` and `setFlyingVelocityMod`: the same, with a plugin's factors.
        if let Some(([dx, dy, dz], [fx, fy, fz])) = crate::game::paper_fam_entity::cart_mods(self, c.id) {
            let max = self.max_speed(c);
            c.velocity.x = c.velocity.x.clamp(-max, max);
            c.velocity.z = c.velocity.z.clamp(-max, max);
            if c.on_ground {
                c.velocity = Vec3::new(c.velocity.x * dx, c.velocity.y * dy, c.velocity.z * dz);
            }
            let v = c.velocity;
            let fall = c.fall_distance;
            self.move_cart(c, v);
            if !c.on_ground {
                c.velocity = Vec3::new(c.velocity.x * fx, c.velocity.y * fy, c.velocity.z * fz);
            } else if c.kind == CartKind::Tnt && fall >= 3.0 {
                c.fuse = 0;
                c.velocity = Vec3::new(c.velocity.x, fall / 10.0, c.velocity.z);
            }
            return;
        }
'''),
    (GAME + 'minecarts.rs', 'let power = (4.0 + r * 1.5 * speed) as f32;', 'let factor = self.persist_raw(c.id).and_then(|tag| tag.get("explosion_speed_factor")).and_then(crate::game::paper_fam_entity::tag_number).unwrap_or(1.5);\n        let power = (4.0 + r * factor * speed) as f32;'),
    (GAME + 'lan_entities.rs', 'e.meta = vehicle_meta(c.hurt_time, c.hurt_dir, c.damage);', '''e.meta = vehicle_meta(c.hurt_time, c.hurt_dir, c.damage);
            // AbstractMinecart.DATA_ID_CUSTOM_DISPLAY_BLOCK and DATA_ID_DISPLAY_OFFSET: saved, and a plugin's to set.
            if let Some(tag) = self.persist.raw.get(&c.id) {
                if let Some(state) = tag.get("DisplayState").and_then(|state| self.state_from_nbt(state)) {
                    e.meta.push((11, serializer::OPTIONAL_BLOCK_STATE, Value::Id(state.0 as i32)));
                }
                if let Some(offset) = tag.get("DisplayOffset").and_then(crate::game::paper_fam_entity::tag_number) {
                    e.meta.push((12, serializer::INT, Value::Int(offset as i32)));
                }
            }
            e.defaults.push((11, serializer::OPTIONAL_BLOCK_STATE, Value::Id(0)));
            e.defaults.push((12, serializer::INT, Value::Int(6)));'''),
    # ---- Items: which mobs may take them, and their friction.
    (LIB + 'entities.rs', '    pub const WIDTH: f64 = 0.25;\n', '''    pub const WIDTH: f64 = 0.25;
    /// `Item.canMobPickup`.
    pub fn mob_can_take(&self) -> bool {
        !crate::paper_hub::plugin_marked(0, self.id)
    }
'''),
    (LIB + 'entities.rs', '            let mut friction = 0.98;\n', '            let free = crate::paper_hub::plugin_marked(1, self.id);\n            let mut friction = if free { 1.0 } else { 0.98 };\n'),
    (LIB + 'entities.rs', 'friction = f64::from(facts.get(below).friction) * 0.98;', 'friction = if free { 1.0 } else { f64::from(facts.get(below).friction) * 0.98 };'),
    (LIB + 'entity_nbt.rs', 'pub fn item_entity_from_nbt(tag: &Tag, id: u32, bob_offset: f32) -> Option<ItemEntity> {', '''pub fn item_entity_from_nbt(tag: &Tag, id: u32, bob_offset: f32) -> Option<ItemEntity> {
    if let Some(plugin) = tag.get("horizon:plugin") {
        crate::paper_hub::plugin_mark(0, id, matches!(plugin.get("mob_pickup"), Some(Tag::Double(bits)) if f64::from_bits(*bits) == 0.0));
        crate::paper_hub::plugin_mark(1, id, matches!(plugin.get("friction"), Some(Tag::Double(bits)) if f64::from_bits(*bits) == 0.0));
    }
    item_entity_from_saved(tag, id, bob_offset)
}
fn item_entity_from_saved(tag: &Tag, id: u32, bob_offset: f32) -> Option<ItemEntity> {'''),
    (GAME + 'allays.rs', 'e.pickup_delay == 0\n', 'e.pickup_delay == 0 && e.mob_can_take()\n'),
    (GAME + 'allays.rs', 'e.pickup_delay == 0 && allay::considers_item_equal(&held, &e.stack)', 'e.pickup_delay == 0 && e.mob_can_take() && allay::considers_item_equal(&held, &e.stack)'),
    (GAME + 'mob_looting.rs', 'if e.pickup_delay > 0 || !reach.intersects(e.bounds()) {', 'if e.pickup_delay > 0 || !e.mob_can_take() || !reach.intersects(e.bounds()) {'),
    (GAME + 'pandas.rs', 'e.pickup_delay == 0 && g.panda_eats_from_ground(&e.stack.item)', 'e.pickup_delay == 0 && e.mob_can_take() && g.panda_eats_from_ground(&e.stack.item)'),
    (GAME + 'piglins.rs', 'if !close || e.pickup_delay > 0 {', 'if !close || e.pickup_delay > 0 || !e.mob_can_take() {'),
    (GAME + 'raids.rs', '&& e.pickup_delay == 0', '&& e.pickup_delay == 0\n                    && e.mob_can_take()'),
    (GAME + 'sulfur_cubes.rs', 'e.pickup_delay == 0 && reach.contains(e.position)', 'e.pickup_delay == 0 && e.mob_can_take() && reach.contains(e.position)'),
    (GAME + 'villagers.rs', 'if !near || e.pickup_delay > 0 || !wants {', 'if !near || e.pickup_delay > 0 || !e.mob_can_take() || !wants {'),
    # ---- Tridents.
    (GAME + 'trident.rs', '                            8.0,\n', '                            crate::game::paper_fam_entity::plugin_in(self.persist.raw.get(&t.id), "damage").map_or(8.0, |damage| damage as f32),\n'),
    (GAME + 'lan_entities.rs', 'if t.stack.has_any_enchantments() {', 'if crate::game::paper_fam_entity::plugin_in(self.persist.raw.get(&t.id), "glint").map_or(t.stack.has_any_enchantments(), |on| on != 0.0) {'),
    # ---- `HumanEntity`'s regeneration and starvation rates.
    ('horizon-world/src/food.rs', '    pub timer: i32,\n}\nimpl Default for FoodData {', '    pub timer: i32,\n    /// `saturatedRegenRate`, `unsaturatedRegenRate` and `starvationRate`.\n    pub rates: [i32; 3],\n}\nimpl Default for FoodData {'),
    ('horizon-world/src/food.rs', '            timer: 0,\n        }\n', '            timer: 0,\n            rates: [10, 80, 80],\n        }\n'),
    ('horizon-world/src/food.rs', 'if self.timer >= 10 {', 'if self.timer >= self.rates[0] {'),
    ('horizon-world/src/food.rs', '            if self.timer >= 80 {\n                world.heal(self, 1.0)?;', '            if self.timer >= self.rates[1] {\n                world.heal(self, 1.0)?;'),
    ('horizon-world/src/food.rs', '            if self.timer >= 80 {\n                if world.health(self)? > 10.0', '            if self.timer >= self.rates[2] {\n                if world.health(self)? > 10.0'),
    # ---- A piglin's own barter and interest lists.
    (GAME + 'piglins.rs', '.is_some_and(|o| self.piglin_loved(o));', '.is_some_and(|o| self.piglin_loved(o) || m.extra.plugin_lists("barter", o) || m.extra.plugin_lists("interest", o));'),
    (GAME + 'piglins.rs', 'if item == "minecraft:gold_ingot" {', 'if item == "minecraft:gold_ingot" || m.extra.plugin_lists("barter", item) {'),
    (GAME + 'piglins.rs', '} else if !self.piglin_loved(item) {', '} else if !self.piglin_loved(item) && !m.extra.plugin_lists("interest", item) {'),
    (GAME + 'piglins.rs', '.is_some_and(|(j, _)| self.piglin_loved(&self.dropped[j].stack.item));', '.is_some_and(|(j, _)| self.piglin_loved(&self.dropped[j].stack.item) || m.extra.plugin_lists("barter", &self.dropped[j].stack.item) || m.extra.plugin_lists("interest", &self.dropped[j].stack.item));'),
    (GAME + 'piglins.rs', 'if self.piglin_loved(&item) {', 'if self.piglin_loved(&item) || self.mobs[i].extra.plugin_lists("barter", &item) || self.mobs[i].extra.plugin_lists("interest", &item) {'),
    (GAME + 'piglins.rs', '|| item != "minecraft:gold_ingot"', '|| (item != "minecraft:gold_ingot" && !m.extra.plugin_lists("barter", item))'),
    (GAME + 'piglins.rs', 'if &*item != "minecraft:gold_ingot" {', 'if &*item != "minecraft:gold_ingot" && !self.mobs[i].extra.plugin_lists("barter", &item) {'),
    # ---- A player's flags: `plugin_mark` 2 is fall damage while allowed to fly, 3 is no part in spawning.
    (LIB + 'landing.rs', 'if self.player.may_fly {', 'if self.player.may_fly && !crate::paper_hub::plugin_marked(2, self.player.entity_id as u32) {'),
    (GAME + 'natural_spawning.rs', '''pub(super) struct LevelPlayer {
    pub(super) position: Vec3,''', '''pub(super) struct LevelPlayer {
    /// A guest's entity id; 0 for the player at this window.
    pub(super) id: i32,
    pub(super) position: Vec3,'''),
    (GAME + 'natural_spawning.rs', '.map(|p| LevelPlayer {', '.map(|p| LevelPlayer {\n                id: 0,'),
    (GAME + 'natural_spawning.rs', '.map(|g| LevelPlayer {', '.map(|g| LevelPlayer {\n                        id: g.entity_id,'),
    (GAME + 'natural_spawning.rs', '        let players = self.level_players();\n        let nearest = players', '        let players = self.spawn_players();\n        let nearest = players'),
    (GAME + 'natural_spawning.rs', '        let players = self.level_players();\n        if players.is_empty() {', '        let players = self.spawn_players();\n        if players.is_empty() {'),
    (GAME + 'natural_spawning.rs', 'self.level_players_iter().any(|p| p.alive_near(x, y, z, range))', 'self.level_players_iter().filter(|p| !horizon_client::paper_hub::plugin_marked(3, p.id as u32)).any(|p| p.alive_near(x, y, z, range))'),
    (GAME + 'natural_spawning.rs', '    pub(super) fn level_players(&self) -> Vec<LevelPlayer> {', '''    /// `EntitySelector.PLAYER_AFFECTS_SPAWNING`: the level's players but those a plugin took out of it.
    fn spawn_players(&self) -> Vec<LevelPlayer> {
        self.level_players_iter().filter(|p| !horizon_client::paper_hub::plugin_marked(3, p.id as u32)).collect()
    }
    pub(super) fn level_players(&self) -> Vec<LevelPlayer> {'''),
    # ---- Fixed poses.
    (MOBS + 'sync.rs', 'pub fn pose_id(m: &Mob) -> u8 {\n', 'pub fn pose_id(m: &Mob) -> u8 {\n    if let Some(pose) = m.extra.plugin_number("pose") {\n        return pose as u8;\n    }\n'),
    (LIB + 'lan/guest.rs', '    pub fn pose(&self) -> Pose {\n', '    pub fn pose(&self) -> Pose {\n        if let Some(pose) = crate::paper_hub::fixed_pose(self.entity_id) {\n            return Pose::from_id(pose);\n        }\n'),
    # ---- A mob's own loot table.
    (LIB + 'mobs.rs', '    /// `EnderMan.DATA_STARED_AT`.\n', '    /// `Mob.lootTable`, in place of the type\'s (`DeathLootTable`).\n    pub loot_table: Option<String>,\n    /// `EnderMan.DATA_STARED_AT`.\n'),
    (LIB + 'entity_nbt.rs', '    mob.extra.explosion_radius = number(tag.get("ExplosionRadius")).map(|radius| radius as i32);\n', '    mob.extra.explosion_radius = number(tag.get("ExplosionRadius")).map(|radius| radius as i32);\n    mob.extra.loot_table = crate::save::text(tag.get("DeathLootTable")).map(|table| table.to_string());\n    mob.extra.loot_seed = number(tag.get("DeathLootTableSeed")).unwrap_or(0);\n'),
    (MOBS + 'drops.rs', 'pub fn death_loot_table(m: &Mob, mob_drops: bool) -> Option<String> {\n', 'pub fn death_loot_table(m: &Mob, mob_drops: bool) -> Option<String> {\n    if let Some(table) = &m.extra.loot_table {\n        return mob_drops.then(|| table.clone());\n    }\n'),
    # ---- Experience orbs: why each one is there.
    (LIB + 'game.rs', '        let _yaw = r.next_float();\n        let p = Vec3::new(at[0], at[1], at[2]);\n        self.next_entity += 1;\n        self.orbs\n', '        let _yaw = r.next_float();\n        let p = Vec3::new(at[0], at[1], at[2]);\n        crate::game::paper_fam_entity::orb_born(self, self.next_entity + 1);\n        self.next_entity += 1;\n        self.orbs\n'),
    (GAME + 'creatures.rs', 'self.spawn_orbs(pos, value as u32)', '{\n                        crate::game::paper_fam_entity::orb_cause(self, "ENTITY_DEATH", Some(m.id), crate::game::paper_fam_entity::killer_uuid(self, m));\n                        self.spawn_orbs(pos, value as u32);\n                        crate::game::paper_fam_entity::orb_cause_over();\n                    }'),
    (GAME + 'creatures.rs', '            self.spawn_orbs(at, amount);\n', '            crate::game::paper_fam_entity::orb_breed(self);\n            self.spawn_orbs(at, amount);\n            crate::game::paper_fam_entity::orb_cause_over();\n'),
    (GAME + 'fishing.rs', 'self.spawn_orbs([owner.x, owner.y + 0.5, owner.z + 0.5], xp);', 'crate::game::paper_fam_entity::orb_cause(self, "FISHING", Some(hook.id), self.lan_acting_uuid);\n                self.spawn_orbs([owner.x, owner.y + 0.5, owner.z + 0.5], xp);\n                crate::game::paper_fam_entity::orb_cause_over();'),
    (GAME + 'lan_actions.rs', '            self.spawn_orbs(at, reward);\n', '            let killer = self.as_guest(host, conn, |game| game.player_hurt_by()).flatten();\n            crate::game::paper_fam_entity::orb_cause_death(host, conn, killer);\n            self.spawn_orbs(at, reward);\n            crate::game::paper_fam_entity::orb_cause_over();\n'),
    (GAME + 'potions.rs', 'self.spawn_orbs([at.x, at.y, at.z], value as u32);', 'crate::game::paper_fam_entity::orb_cause(self, "EXP_BOTTLE", super::paper_fam_ev_entity::dying_id(), crate::game::paper_fam_entity::named_player(self, owner.as_ref()));\n            self.spawn_orbs([at.x, at.y, at.z], value as u32);\n            crate::game::paper_fam_entity::orb_cause_over();'),
    (GAME + 'stations.rs', 'self.spawn_orbs(pos.map(|v| f64::from(v) + 0.5), xp);', 'crate::game::paper_fam_entity::orb_cause(self, "GRINDSTONE", None, self.lan_acting_uuid);\n            self.spawn_orbs(pos.map(|v| f64::from(v) + 0.5), xp);\n            crate::game::paper_fam_entity::orb_cause_over();'),
    (GAME + 'trading.rs', 'self.add_orb([origin[0], origin[1] + 0.5, origin[2]], v as u32)', '{\n                        crate::game::paper_fam_entity::orb_cause(self, "VILLAGER_TRADE", mob, self.lan_acting_uuid);\n                        self.add_orb([origin[0], origin[1] + 0.5, origin[2]], v as u32);\n                        crate::game::paper_fam_entity::orb_cause_over();\n                    }'),
    # ---- Falling blocks.
    (LIB + 'game.rs', 'if !on_ground && (f.age > 100 && outside || f.age > 600) {', 'if !on_ground && (f.age > 100 && outside || f.age > 600) && crate::game::paper_fam_entity::plugin_number(self, f.id, "auto_expire") != Some(0.0) {'),
    # ---- A primed TNT's look.
    (GAME + 'lan_entities.rs', '            e.precise = t.on_ground;\n', '''            if let Some(state) = self.persist.raw.get(&t.id).and_then(|tag| tag.get("block_state")).and_then(|state| self.state_from_nbt(state)) {
                e.meta.push((9, serializer::BLOCK_STATE, Value::Id(state.0 as i32)));
            }
            e.precise = t.on_ground;
'''),
    # ---- Thrown potions and clouds.
    (GAME + 'potions.rs', '''pub(super) struct ThrownPotion {
    /// Its entity id once a server's clients track it (0 until then).
    id: u32,
    stack: ItemStack,''', '''pub(super) struct ThrownPotion {
    /// Its entity id once a server's clients track it (0 until then).
    pub(super) id: u32,
    pub(super) stack: ItemStack,'''),
    (GAME + 'potions.rs', '    thrown: Vec<ThrownPotion>,\n', '    pub(super) thrown: Vec<ThrownPotion>,\n'),
    (GAME + 'potions.rs', '''#[derive(Clone, Copy, PartialEq)]
enum Victim {''', '''#[derive(Clone, Copy, PartialEq)]
pub(super) enum Victim {'''),
    (GAME + 'potions.rs', '    fn cloud_affect(\n', '    pub(super) fn cloud_affect(\n'),
    (GAME + 'lan_entities.rs', '                self.lan_display_data(tag, hub, &mut e);\n', '''                self.lan_display_data(tag, hub, &mut e);
                // AreaEffectCloud.DATA_RADIUS, DATA_WAITING and DATA_PARTICLE, from a plugin's cloud: `paper_fam_entity::tick_clouds`.
                if e.kind == "minecraft:area_effect_cloud" {
                    let (radius, waiting, options) = crate::game::paper_fam_entity::cloud_look(tag);
                    e.meta.push((8, serializer::FLOAT, Value::Float(radius)));
                    if waiting {
                        e.meta.push((9, serializer::BOOLEAN, Value::Bool(true)));
                    }
                    e.defaults.push((9, serializer::BOOLEAN, Value::Bool(false)));
                    if let Some(p) = Self::hub_particle(hub, &options, None) {
                        e.meta.push((10, serializer::PARTICLE, Value::Particle(p)));
                    }
                }
'''),
    # ---- The dragon's podium, and who touched an interaction.
    (GAME + 'dragon.rs', '    fn podium_top(&self) -> Vec3 {\n', '''    pub(super) fn podium_top(&self) -> Vec3 {
        if let Some(at) = crate::game::paper_fam_entity::podium(self) {
            return at;
        }
'''),
    (GAME + 'lan_actions.rs', '            S::Attack(p) => self.guest_attack(host, conn, p.id),\n', '''            S::Attack(p) => {
                crate::game::paper_fam_entity::interaction_touched(self, host, conn, p.id, "attack");
                self.guest_attack(host, conn, p.id)
            }
'''),
    (GAME + 'lan_actions.rs', '            S::Interact(p) => {\n', '''            S::Interact(p) => {
                crate::game::paper_fam_entity::interaction_touched(self, host, conn, p.id, "interaction");
'''),
    (MOBS + 'living.rs', '    /// `LivingEntity.getAirDrag`: what vertical air drag a bounce applies.\n    fn air_drag(&self) -> f32 {', '    /// `LivingEntity.getAirDrag`: what vertical air drag a bounce applies.\n    pub fn air_drag(&self) -> f32 {'),
    # ---- Saved with the mob now, or sent to who comes later.
    (MOBS + 'ai.rs', 'if m.kind.flags.burns_in_day && m.alive() && sun_burn_tick(m, w) {', 'if m.extra.plugin_flag("burns_in_day").unwrap_or(m.kind.flags.burns_in_day) && m.alive() && sun_burn_tick(m, w) {'),
    (GAME + 'lan.rs', '| i8::from(m.extra.sprinting) << 3', '| i8::from(m.extra.plugin_flag("sneaking") == Some(true)) << 1\n        | i8::from(m.extra.sprinting) << 3'),
    (MOBS + 'warden_brain.rs', 'pub(super) fn set_disturbance(m: &mut Mob, w: &MobWorld, position: [i32; 3]) {', 'pub fn set_disturbance(m: &mut Mob, w: &MobWorld, position: [i32; 3]) {'),
    # ---- A player's maximum air: `paper_hub::max_air`.
    (LIB + 'motion/environment.rs', 'self.player.air = (self.player.air + 4).min(300);', 'self.player.air = (self.player.air + 4).min(crate::paper_hub::max_air(self.player.entity_id));'),
    (GAME + 'spill.rs', 'self.award_orb(at, value, merge_id);', 'crate::game::paper_fam_entity::orb_cause(self, "BLOCK_BREAK", None, self.lan_acting_uuid);\n            self.award_orb(at, value, merge_id);\n            crate::game::paper_fam_entity::orb_cause_over();'),
    (LIB + 'game.rs', '            self.spawn_orbs(at, xp);\n', '            crate::game::paper_fam_entity::orb_cause(self, "FURNACE", None, self.lan_acting_uuid);\n            self.spawn_orbs(at, xp);\n            crate::game::paper_fam_entity::orb_cause_over();\n'),
    (MOBS + 'equine.rs', 'let temper = i32::from(m.extra.variant3);', 'let temper = m.extra.temper;'),
    (MOBS + 'equine.rs', 'm.extra.variant3 = (temper + 5).min(max) as u8;', 'm.extra.temper = (temper + 5).min(max);'),
    (LIB + 'entity_nbt.rs', 'map.insert("Temper".into(), Tag::Int(i32::from(m.extra.variant3)));', 'map.insert("Temper".into(), Tag::Int(m.extra.temper));'),
    (LIB + 'entity_nbt.rs', 'm.extra.variant3 = number(tag.get("Temper")).unwrap_or(0).clamp(0, 100) as u8;', 'm.extra.temper = number(tag.get("Temper")).unwrap_or(0) as i32;'),
    (MOBS + 'llama.rs', '    m.extra.variant3 = i32::from(m.extra.variant3).min(MAX_TEMPER) as u8;\n', ''),
    (GAME + 'creatures.rs', '"temper" : m.extra.variant3,', '"temper" : m.extra.temper,'),
    # ---- `Mob.setLootTable` with a seed: `LootTable.getRandomItems(params, seed)`.
    (MOBS + 'drops.rs', 'let items = roll(d, &table, &params, sequences);', '''let items = match m.extra.loot_seed {
            0 => roll(d, &table, &params, sequences),
            seed => d.loot.roll_entity(&table, &mut horizon_core::random::LegacyRandom::new(seed), &params),
        };'''),
    # ---- Arrows and stingers stuck in a mob.
    (MOBS + 'ai.rs', '        // Air supply and drowning (WaterAnimal.handleAirSupply for aquatic mobs).\n', '        m.extra.tick_stuck();\n        // Air supply and drowning (WaterAnimal.handleAirSupply for aquatic mobs).\n'),
    (GAME + 'creatures.rs', '                        dodged = !hurt\n', '''                        if hurt
                            && p.arrow.as_ref().is_some_and(|a| a.pierce == 0)
                            && let Some(m) = self.mobs.iter_mut().find(|m| m.id == id)
                        {
                            m.extra.arrow_count += 1;
                        }
                        dodged = !hurt
'''),
    (GAME + 'creatures.rs', '                    self.mob_hits_mob(target, amount, source);\n', '''                    let sting = source.kind == horizon_client::mobs::DamageKind::Sting;
                    if self.mob_hits_mob(target, amount, source)
                        && sting
                        && let Some(m) = self.mobs.iter_mut().find(|m| m.id == target)
                    {
                        m.extra.stinger_count += 1;
                    }
'''),
    (GAME + 'lan.rs', '    defaults.push((9, serializer::FLOAT, Value::Float(1.0)));\n', '''    defaults.push((9, serializer::FLOAT, Value::Float(1.0)));
    // LivingEntity.DATA_ARROW_COUNT_ID and DATA_STINGER_COUNT_ID.
    if m.extra.arrow_count != 0 {
        meta.push((12, serializer::INT, Value::Int(m.extra.arrow_count)));
    }
    defaults.push((12, serializer::INT, Value::Int(0)));
    if m.extra.stinger_count != 0 {
        meta.push((13, serializer::INT, Value::Int(m.extra.stinger_count)));
    }
    defaults.push((13, serializer::INT, Value::Int(0)));
'''),
    # ---- `setAffectsSpawning`: the despawn check.
    (GAME + 'creatures.rs', '''                w.nearest_player = super::natural_spawning::nearest_player(
                        &players,
''', '''                w.nearest_player = super::natural_spawning::nearest_player(
                        &spawn_players,
'''),
    (GAME + 'creatures.rs', '        let players = self.level_players();\n        let', '        let players = self.level_players();\n        let spawn_players = self.spawn_players();\n        let'),
    (GAME + 'natural_spawning.rs', '    fn spawn_players(&self) -> Vec<LevelPlayer> {', '    pub(super) fn spawn_players(&self) -> Vec<LevelPlayer> {'),
    # ---- A player's own simulation distance: their tickets, and nobody else's.
    (LIB + 'ticket_levels.rs', '    pub view_distance: i32,\n    pub simulation_distance: i32,\n}', '    pub view_distance: i32,\n    pub simulation_distance: i32,\n    /// The players with a simulation distance of their own (`Player.setSimulationDistance`), who aren\'t in `players`.\n    pub own: Vec<((i32, i32), i32)>,\n}'),
    (LIB + 'ticket_levels.rs', '            s.players.dedup();\n', '            s.players.dedup();\n            s.own.sort_unstable();\n            s.own.dedup();\n'),
    (LIB + 'ticket_levels.rs', '        .map(|&p| (p, sim))\n', '        .map(|&p| (p, sim))\n        .chain(s.own.iter().map(|&(p, own)| (p, own.clamp(0, 32))))\n'),
    (LIB + 'ticket_levels.rs', '''        .map(|&p| (p, reach))
        .chain(s.forced.iter().map(|&f| (f, 1)))''', '''        .map(|&p| (p, reach))
        .chain(s.own.iter().map(|&(p, own)| (p, reach_of(s, own).2)))
        .chain(s.forced.iter().map(|&f| (f, 1)))'''),
    (LIB + 'ticket_levels.rs', '''    for &(px, pz) in &s.players {
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let d = dx.abs().max(dz.abs());
                lower(
                    px + dx,
                    pz + dz,
                    i32::from(ENTITY_TICKING) + (d - view).max(0),
                    player_sim + d,
                );
            }
        }
    }
''', '''    let everyone = s.players.iter().map(|&p| (p, player_sim, reach)).chain(s.own.iter().map(|&(p, own)| {
        let (_, sim, reach) = reach_of(s, own);
        (p, sim, reach)
    }));
    for ((px, pz), player_sim, reach) in everyone {
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let d = dx.abs().max(dz.abs());
                lower(
                    px + dx,
                    pz + dz,
                    i32::from(ENTITY_TICKING) + (d - view).max(0),
                    player_sim + d,
                );
            }
        }
    }
'''),
    (LIB + 'ticket_levels.rs', '''    for &(px, pz) in &s.players {
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let d = dx.abs().max(dz.abs());
                lower(
                    (px + dx, pz + dz),
                    i32::from(ENTITY_TICKING) + (d - view).max(0),
                    player_sim + d,
                );
            }
        }
    }
''', '''    let everyone = s.players.iter().map(|&p| (p, player_sim, reach)).chain(s.own.iter().map(|&(p, own)| {
        let (_, sim, reach) = reach_of(s, own);
        (p, sim, reach)
    }));
    for ((px, pz), player_sim, reach) in everyone {
        for dz in -reach..=reach {
            for dx in -reach..=reach {
                let d = dx.abs().max(dz.abs());
                lower(
                    (px + dx, pz + dz),
                    i32::from(ENTITY_TICKING) + (d - view).max(0),
                    player_sim + d,
                );
            }
        }
    }
'''),
    (LIB + 'ticket_levels.rs', '''fn reach(s: &Sources) -> (i32, i32, i32) {
    let view = s.view_distance.clamp(0, 32);
    let simulation = s.simulation_distance.clamp(0, 32);
''', '''fn reach(s: &Sources) -> (i32, i32, i32) {
    reach_of(s, s.simulation_distance)
}

fn reach_of(s: &Sources, simulation: i32) -> (i32, i32, i32) {
    let view = s.view_distance.clamp(0, 32);
    let simulation = simulation.clamp(0, 32);
'''),
    (LIB + 'game.rs', '.map(|g| (g.chunk(), g.spectator()))', '.filter(|g| horizon_client::paper_hub::simulation_of(g.entity_id).is_none())\n                        .map(|g| (g.chunk(), g.spectator()))'),
    (LIB + 'game.rs', '                    portal: Vec::new(),\n', '                    portal: Vec::new(),\n                    own: crate::game::paper_fam_entity::own_simulation(self, rule),\n'),
    # ---- Per-player warden trackers.
    (GAME + 'sculk.rs', '''    last_health: Option<f32>,
    mount: Option<(u32, [f64; 3])>,
}
''', '''    last_health: Option<f32>,
    mount: Option<(u32, [f64; 3])>,
    /// This player's `WardenSpawnTracker`, and whether it came from their file yet.
    pub(super) tracker: WardenSpawnTracker,
    tracker_loaded: bool,
}
'''),
    (GAME + 'sculk.rs', '    pub(super) fn swap_player(&mut self, other: &mut PlayerEvents) {\n', '    pub(super) fn swap_player(&mut self, other: &mut PlayerEvents) {\n        std::mem::swap(&mut self.tracker, &mut other.tracker);\n        std::mem::swap(&mut self.tracker_loaded, &mut other.tracker_loaded);\n'),
    (GAME + 'sculk.rs', '    /// `ServerPlayer`\'s `warden_spawn_tracker` from the player data.\n', '''    /// `ServerPlayer.tick`'s `wardenSpawnTracker.tick` on a guest's turn, with their file in place.
    pub(super) fn guest_warden_tick(&mut self) {
        self.load_warden_tracker();
        self.sculk.tracker.tick();
        self.save_warden_tracker();
    }
    /// `ServerPlayer`'s `warden_spawn_tracker` from the player data.
'''),
    (GAME + 'sculk.rs', '''        let mut trackers = [self.sculk.tracker];
        let level = WardenSpawnTracker::try_warn(&mut trackers, warden_nearby);
        self.sculk.tracker = trackers[0];
''', '''        // `WardenSpawnTracker.getNearbyPlayers`: everyone within 16 blocks shares the warning.
        let near: Vec<u32> = self.lan.iter().flat_map(|host| {
            host.hub.guests.iter().filter(|g| host.hub.here(g) && (0..3).map(|i| (g.position[i] - c[i]).powi(2)).sum::<f64>() < 16.0 * 16.0).map(|g| g.conn)
        }).collect();
        let mut trackers = vec![self.sculk.tracker];
        if let Some(host) = self.lan.as_ref() {
            trackers.extend(near.iter().map(|conn| host.play.get(conn).map_or_else(Default::default, |play| play.sculk.tracker)));
        }
        let level = WardenSpawnTracker::try_warn(&mut trackers, warden_nearby);
        self.sculk.tracker = trackers[0];
        if let Some(host) = self.lan.as_mut() {
            for (conn, tracker) in near.iter().zip(&trackers[1..]) {
                if let Some(play) = host.play.get_mut(conn) {
                    play.sculk.tracker = *tracker;
                }
            }
        }
'''),
    (GAME + 'lan_actions.rs', '    sculk: super::sculk::PlayerEvents,\n', '    pub(super) sculk: super::sculk::PlayerEvents,\n'),
    (GAME + 'lan_actions.rs', '                game.tick_player_sculk_events();\n', '                game.guest_warden_tick();\n                game.tick_player_sculk_events();\n'),
    # ---- Every orb says why.
    (GAME + 'creaking.rs', '            let xp = 20 + self.world.random.next_int_bound(5).unwrap_or(0) as u32;\n', '            let xp = 20 + self.world.random.next_int_bound(5).unwrap_or(0) as u32;\n            crate::game::paper_fam_entity::orb_cause(self, "BLOCK_BREAK", None, self.lan_acting_uuid);\n'),
    (GAME + 'creaking.rs', '                xp,\n            );\n', '                xp,\n            );\n            crate::game::paper_fam_entity::orb_cause_over();\n'),
    (GAME + 'dragon.rs', 'self.spawn_orbs([p.x, p.y, p.z], (xp as f32 * 0.08) as u32);', 'crate::game::paper_fam_entity::orb_cause(self, "ENTITY_DEATH", self.dragon.as_ref().map(|d| d.id), None);\n            self.spawn_orbs([p.x, p.y, p.z], (xp as f32 * 0.08) as u32);\n            crate::game::paper_fam_entity::orb_cause_over();'),
    (GAME + 'dragon.rs', 'self.spawn_orbs([p.x, p.y, p.z], (xp as f32 * 0.2) as u32);', 'crate::game::paper_fam_entity::orb_cause(self, "ENTITY_DEATH", self.dragon.as_ref().map(|d| d.id), None);\n                self.spawn_orbs([p.x, p.y, p.z], (xp as f32 * 0.2) as u32);\n                crate::game::paper_fam_entity::orb_cause_over();'),
    (GAME + 'creatures.rs', '        let q = self.mobs.iter().find(|m| m.id == partner).cloned();\n', '        crate::game::paper_fam_entity::breeding_is(&p);\n        let q = self.mobs.iter().find(|m| m.id == partner).cloned();\n'),
    (GAME + 'creatures.rs', 'self.add_orb([p.position.x, p.position.y, p.position.z], xp);', 'crate::game::paper_fam_entity::orb_cause(self, "BREED", Some(parent), None);\n                self.add_orb([p.position.x, p.position.y, p.position.z], xp);\n                crate::game::paper_fam_entity::orb_cause_over();'),
    (LIB + 'game.rs', '''                if xp > 0 {
                    self.spawn_orbs(
                        [
                            f64::from(pos[0]) + 0.5,
                            f64::from(pos[1]) + 0.5,
                            f64::from(pos[2]) + 0.5,
                        ],
                        xp,
                    );
                }
''', '''                if xp > 0 {
                    crate::game::paper_fam_entity::orb_cause(self, "BLOCK_BREAK", None, self.lan_acting_uuid);
                    self.spawn_orbs(
                        [
                            f64::from(pos[0]) + 0.5,
                            f64::from(pos[1]) + 0.5,
                            f64::from(pos[2]) + 0.5,
                        ],
                        xp,
                    );
                    crate::game::paper_fam_entity::orb_cause_over();
                }
'''),
    (LIB + 'game.rs', '            self.spawn_orbs(feet, reward);\n', '            crate::game::paper_fam_entity::orb_cause_host_death(self.acting_uuid());\n            self.spawn_orbs(feet, reward);\n            crate::game::paper_fam_entity::orb_cause_over();\n'),
    # ---- `startRiptideAttack`'s item: the weapon of the spin's hit (`Player.getWeaponItem`).
    (GAME + 'trident.rs', '''            self.attack_mob(index);
        }
        if let Some(p) = self.player.as_mut() && p.auto_spin_attack_ticks <= 0 {
''', '''            let spin_item = crate::game::paper_fam_entity::spin_item(self);
            if let Some(item) = spin_item {
                let own = std::mem::replace(self.held_mut(), Some(item));
                self.attack_mob(index);
                let used = std::mem::replace(self.held_mut(), own);
                crate::game::paper_fam_entity::spin_item_back(self, used);
            } else {
                self.attack_mob(index);
            }
        }
        if let Some(p) = self.player.as_mut() && p.auto_spin_attack_ticks <= 0 {
            crate::game::paper_fam_entity::spin_over(p.entity_id);
'''),
    # ---- `persist.raw` of what was forgotten at the first save: everything `plugin_set` can be asked about.
    (GAME + 'persist.rs', '            .chain(self.others.iter().map(|e| e.id))\n            .collect();', '''            .chain(self.others.iter().map(|e| e.id))
            .chain(self.orbs.iter().map(|e| e.id))
            .chain(self.tnts.iter().map(|e| e.id))
            .chain(self.falling.iter().map(|e| e.id))
            .chain(self.tridents.iter().map(|e| e.id))
            .chain(self.rockets.iter().map(|e| e.id))
            .chain(self.creatures.projectiles.iter().map(|e| e.id))
            .chain(self.potions.thrown.iter().map(|e| e.id))
            .chain(self.dragon.iter().map(|e| e.id))
            .collect();'''),
    # ---- Thrown tridents: saved with their chunk, which the game didn't do at all.
    (GAME + 'trident.rs', 'impl ThrownTrident {\n', '''impl ThrownTrident {
    /// `ThrownTrident.addAdditionalSaveData`, over what it was loaded with.
    pub(super) fn to_nbt(&self, uuid: [i32; 4], owner: Option<[i32; 4]>, raw: Option<&horizon_world::nbt::Tag>) -> horizon_world::nbt::Tag {
        use horizon_world::nbt::Tag;
        let mut saved = match raw {
            Some(Tag::Compound(saved)) => saved.clone(),
            _ => Default::default(),
        };
        let list = |v: Vec3| Tag::List([v.x, v.y, v.z].iter().map(|d| Tag::Double(d.to_bits())).collect());
        for (key, value) in [
            ("id", Tag::String("minecraft:trident".into())),
            ("UUID", Tag::IntArray(uuid.to_vec())),
            ("Pos", list(self.position)),
            ("Motion", list(self.velocity)),
            ("Rotation", Tag::List(vec![Tag::Float(self.yaw.to_bits()), Tag::Float(self.pitch.to_bits())])),
            ("item", horizon_client::save::item_to_nbt(&self.stack)),
            ("inGround", Tag::Byte(i8::from(self.in_ground))),
            ("shake", Tag::Byte(self.shake as i8)),
            ("DealtDamage", Tag::Byte(i8::from(self.dealt_damage))),
            ("pickup", Tag::Byte(if self.creative_only { 2 } else { 1 })),
        ] {
            saved.insert(key.into(), value);
        }
        if let Some(owner) = owner {
            saved.insert("Owner".into(), Tag::IntArray(owner.to_vec()));
        }
        Tag::Compound(saved)
    }
    /// `ThrownTrident.readAdditionalSaveData`; `guest` is its owner, if they're on.
    pub(super) fn from_nbt(tag: &horizon_world::nbt::Tag, id: u32, guest: Option<u32>) -> Option<Self> {
        use horizon_world::nbt::Tag;
        let stack = tag.get("item").and_then(horizon_client::save::item_from_nbt)?;
        let position = horizon_client::entity_nbt::position(tag)?;
        let [yaw, pitch] = horizon_client::entity_nbt::rotation(tag);
        let byte = |key: &str| if let Some(Tag::Byte(value)) = tag.get(key) { *value } else { 0 };
        Some(Self {
            id,
            loyalty: stack.enchantment_level("minecraft:loyalty"),
            stack,
            position,
            previous: position,
            velocity: horizon_client::entity_nbt::motion(tag),
            yaw,
            pitch,
            in_ground: byte("inGround") != 0,
            in_ground_time: 0,
            shake: byte("shake").max(0) as u32,
            dealt_damage: byte("DealtDamage") != 0,
            no_physics: false,
            returning: 0,
            creative_only: byte("pickup") == 2,
            owner: 0,
            guest,
        })
    }
'''),
    (GAME + 'persist.rs', '        for f in &self.falling {\n            if keys.contains(&chunk_of(f.position)) {\n', '''        for t in &self.tridents {
            if keys.contains(&chunk_of(t.position)) {
                out.entry(chunk_of(t.position)).or_default().push(crate::game::paper_fam_entity::trident_tag(self, t));
            }
        }
        for f in &self.falling {
            if keys.contains(&chunk_of(f.position)) {
'''),
    (GAME + 'persist.rs', '            self.falling.retain(|f| !gone(f.position));\n', '            self.falling.retain(|f| !gone(f.position));\n            self.tridents.retain(|t| !gone(t.position));\n'),
    (GAME + 'persist.rs', '                    "minecraft:firework_rocket" => {\n', '''                    "minecraft:trident" => {
                        self.next_entity += 1;
                        let id = self.next_entity;
                        let guest = crate::game::paper_fam_entity::trident_owner(self, &tag);
                        if let Some(trident) = super::trident::ThrownTrident::from_nbt(&tag, id, guest) {
                            self.remember(id, uuid, tag);
                            self.tridents.push(trident);
                        }
                    }
                    "minecraft:firework_rocket" => {
'''),
    # ---- A pose a plugin set: the box too (`LivingEntity.getDimensions`).
    (MOBS + 'living.rs', '        if super::villager::sleeping(self) {\n', '        if super::villager::sleeping(self) || self.extra.plugin_number("pose") == Some(2.0) {\n'),
    # ---- A real bar for a boss: `Boss.getBossBar`.
    (GAME + 'bossbars.rs', '.filter(|m| m.species() == Species::Wither && m.alive() && !m.removed)', '.filter(|m| m.species() == Species::Wither && m.alive() && !m.removed && !crate::game::paper_fam_entity::boss_bar_owned(m.id))'),
    (GAME + 'bossbars.rs', '            && let Some(progress) = self.dragon_boss_progress(viewer.at, viewer.alive)\n', '            && !self.dragon.as_ref().is_some_and(|dragon| crate::game::paper_fam_entity::boss_bar_owned(dragon.id))\n            && let Some(progress) = self.dragon_boss_progress(viewer.at, viewer.alive)\n'),
    (GAME + 'bossbars.rs', '    fn boss_viewers(&self) -> Vec<Viewer> {', '    pub(super) fn boss_viewers(&self) -> Vec<Viewer> {'),
    (GAME + 'bossbars.rs', '''struct Viewer {
    uuid: u128,
    ints: [i32; 4],
    at: [f64; 3],
    alive: bool,
    here: bool,
''', '''pub(super) struct Viewer {
    uuid: u128,
    pub(super) ints: [i32; 4],
    pub(super) at: [f64; 3],
    pub(super) alive: bool,
    pub(super) here: bool,
'''),
    (GAME + 'bossbars.rs', '    conn: Option<u32>,\n    view_chunks: i32,\n}', '    conn: Option<u32>,\n    pub(super) view_chunks: i32,\n}'),
    # ---- Every cloud is a plugin's to change: `kept_merge` turns the game's into a compound `tick_clouds` runs.
    (GAME + 'potions.rs', '''pub(super) struct PotionCloud {
    effects: Vec<(&'static str, i32, i32)>,''', '''pub(super) struct PotionCloud {
    pub(super) effects: Vec<(&'static str, i32, i32)>,'''),
    (GAME + 'potions.rs', '''    radius_per_tick: f32,
    /// The potion's colour (`PotionContents.getColor`).
    color: u32,''', '''    pub(super) radius_per_tick: f32,
    /// The potion's colour (`PotionContents.getColor`).
    pub(super) color: u32,'''),
    (GAME + 'dragon.rs', '    pub(super) fn podium_top(&self) -> Vec3 {\n', '''    /// `DragonSittingFlamingPhase`: its breath stays while this lasts.
    pub(super) fn dragon_flaming(&self) -> bool {
        self.dragon.as_ref().is_some_and(|d| matches!(d.phase, Phase::Flaming { .. }))
    }
    pub(super) fn podium_top(&self) -> Vec3 {
'''),
    (GAME + 'lan_entities.rs', '                    let (radius, waiting, options) = crate::game::paper_fam_entity::cloud_look(tag);\n', '''                    let (radius, waiting, options) = crate::game::paper_fam_entity::cloud_look(tag);
                    let own_particle = crate::game::paper_fam_entity::cloud_particle(o.id);
'''),
    (GAME + 'lan_entities.rs', '                    if let Some(p) = Self::hub_particle(hub, &options, None) {\n                        e.meta.push((10, serializer::PARTICLE, Value::Particle(p)));\n                    }\n                }\n', '                    if let Some(p) = own_particle.or_else(|| Self::hub_particle(hub, &options, None)) {\n                        e.meta.push((10, serializer::PARTICLE, Value::Particle(p)));\n                    }\n                }\n'),
    (LIB + 'game.rs', "    fn particle_data(\n        game: &mut Game<'_>,\n", "    pub(in crate::game) fn particle_data(\n        game: &mut Game<'_>,\n"),
    # ---- A falling block's block entity.
    (LIB + 'game.rs', 'Ok(FallStep::Landed(pos)) => self.finish_falling_block(f, pos),', '''Ok(FallStep::Landed(pos)) => {
                    let tile = crate::game::paper_fam_entity::falling_tile(self, f.id, f.saved.as_deref().map(|saved| &saved.1));
                    self.finish_falling_block(f, pos);
                    if let Some(tile) = tile {
                        self.load_block_entity_nbt(pos, &tile);
                    }
                }'''),
    # ---- `LivingEntity.setMemory` goes through `/data`'s own reference.
    (GAME + 'commands.rs', '\nmod data;\n', '\npub(in crate::game) mod data;\n'),
    # ---- Why an orb is there goes with it through a save.
    (GAME + 'persist.rs', '.push(with_raw(entity_nbt::orb_to_nbt(o, uuid), raw.as_ref()));', '.push(crate::game::paper_fam_entity::keep_plugin(with_raw(entity_nbt::orb_to_nbt(o, uuid), raw.as_ref()), self.persist.raw.get(&o.id)));'),
    (GAME + 'persist.rs', '                            orb.id = self.next_entity;\n', '                            orb.id = self.next_entity;\n                            crate::game::paper_fam_entity::plugin_loaded(self, orb.id, &tag);\n'),
    # The tests' own literals.
    (LIB + 'ticket_levels.rs', 'simulation_distance: sim,\n', 'simulation_distance: sim,\n                own: Vec::new(),\n'),
    (LIB + 'ticket_levels.rs', 'simulation_distance: 2,\n', 'simulation_distance: 2,\n                own: Vec::new(),\n'),
    (LIB + 'ticket_levels.rs', 'simulation_distance: 2 + next(12).abs(),\n', 'simulation_distance: 2 + next(12).abs(),\n                own: Vec::new(),\n'),
    (LIB + 'ticket_levels.rs', 'simulation_distance: 12,\n', 'simulation_distance: 12,\n                own: Vec::new(),\n'),
    (LIB + 'ticket_levels.rs', 'simulation_distance: 10,\n', 'simulation_distance: 10,\n                own: Vec::new(),\n'),
    (LIB + 'game.rs', '    fn dying_id() -> Option<u32> {', '    pub(in crate::game) fn dying_id() -> Option<u32> {'),
    # ---- Saved, not session-only: a falling block's, a primed TNT's and the dragon's plugin values.
    (GAME + 'persist.rs', '.push(with_raw(tag, raw.as_ref()));', '.push(crate::game::paper_fam_entity::keep_plugin(with_raw(tag, raw.as_ref()), self.persist.raw.get(&f.id)));'),
    (GAME + 'persist.rs', '        for f in &self.falling {\n            if keys.contains(&chunk_of(f.position)) {\n                let (uuid, raw) = match &f.saved {', '''        for t in &self.tnts {
            if keys.contains(&chunk_of(t.position)) {
                out.entry(chunk_of(t.position)).or_default().push(crate::game::paper_fam_entity::tnt_tag(self, t));
            }
        }
        for f in &self.falling {
            if keys.contains(&chunk_of(f.position)) {
                let (uuid, raw) = match &f.saved {'''),
    (GAME + 'persist.rs', '            self.falling.retain(|f| !gone(f.position));\n', '            self.falling.retain(|f| !gone(f.position));\n            self.tnts.retain(|t| !gone(t.position));\n'),
    (GAME + 'persist.rs', '                            self.tnts.push(super::tnt::PrimedTnt {\n                                id: 0,\n', '''                            self.next_entity += 1;
                            self.remember(self.next_entity, uuid, tag.clone());
                            self.tnts.push(super::tnt::PrimedTnt {
                                id: self.next_entity,
'''),
    (GAME + 'persist.rs', '.push(d.to_nbt(uuid, raw.as_ref()));', '.push(crate::game::paper_fam_entity::keep_plugin(d.to_nbt(uuid, raw.as_ref()), self.persist.raw.get(&d.id)));'),
    (GAME + 'persist.rs', '                            self.dragon = Some(d);\n', '                            crate::game::paper_fam_entity::plugin_loaded(self, d.id, &tag);\n                            self.dragon = Some(d);\n'),
    # ---- Who a thing is: real objects, and what the leads and fuses need for it.
    (GAME + 'boat_leads.rs', '    fn release_boat_leash(', '    pub(super) fn release_boat_leash('),
    (GAME + 'tnt.rs', '                self.tnts.remove(i);', '                crate::game::paper_fam_entity::tnt_exploding(self.tnts[i].id);\n                self.tnts.remove(i);'),
    (GAME + 'tnt.rs', '                self.combat.explosion_in_water = false;', '                self.combat.explosion_in_water = false;\n                crate::game::paper_fam_entity::tnt_exploded();'),
    (GAME + 'creatures.rs', '                self.arrow_event(arrow.position, event);', '                crate::game::paper_fam_entity::arrow_landing(self, arrow.owner, arrow.guest, arrow.position);\n                self.arrow_event(arrow.position, event);\n                crate::game::paper_fam_entity::tnt_exploded();'),
    (GAME + 'creatures.rs', '            Species::Creeper if creeper_igniter => {', '            Species::Creeper if creeper_igniter => {\n                crate::game::paper_fam_entity::creeper_lit(m.id, self.lan_acting_uuid);'),
    # ---- Done as Paper does them, not as a click: traders, the halfway hit, a hook pulled in.
    (GAME + 'trading.rs', '            || m.extra.flag2\n', '            || (m.extra.flag2 && !crate::game::paper_fam_entity::trader_forced())\n'),
    (GAME + 'trading.rs', '            || (species == Species::Villager && villager::sleeping(m))\n', '            || (species == Species::Villager && villager::sleeping(m) && !crate::game::paper_fam_entity::trader_forced())\n'),
    (GAME + 'trading.rs', '        if m.is_baby() {\n            // A baby villag', '        if m.is_baby() && !crate::game::paper_fam_entity::trader_forced() {\n            // A baby villag'),
    (GAME + 'trading.rs', '        self.stats.custom("talked_to_villager", 1);', '        if !crate::game::paper_fam_entity::trader_forced() {\n            self.stats.custom("talked_to_villager", 1);\n        }'),
    (GAME + 'trading.rs', '        if trade.offers.is_empty() {\n            if species == Species::Villager {', '        if trade.offers.is_empty() && !crate::game::paper_fam_entity::trader_forced() {\n            if species == Species::Villager {'),
    (GAME + 'fishing.rs', '    fn retrieve_hook(', '    pub(super) fn retrieve_hook('),
    (LIB + 'mobs.rs', 'let dealt = if self.invulnerable as f32 > 10.0 {', 'let dealt = if self.invulnerable as f32 > self.extra.plugin_number("no_damage").map_or(10.0, |ticks| ticks as f32 / 2.0) {'),
    # ---- Goals in the selector, the bar scaled where it's sent, flags where states end.
    (MOBS + 'goals.rs', 'pub trait Goal: std::fmt::Debug {', '''pub trait Goal: std::fmt::Debug {
    /// A plugin's goal: its key.
    fn plugin_key(&self) -> Option<&str> {
        None
    }
    /// A plugin's goal: whether its code wants to run.
    fn set_wanted(&self, _wanted: bool) {}'''),
    (MOBS + 'goals.rs', '    pub fn running_names(&self) -> Vec<&\'static str> {', '''    /// Each goal: its name or a plugin's key, flags, whether it runs, and priority.
    pub fn listed(&self) -> Vec<(String, u8, bool, i32)> {
        self.goals.iter().map(|g| (g.goal.plugin_key().unwrap_or(g.goal.name()).to_owned(), g.goal.flags(), g.running, g.priority)).collect()
    }
    /// Tells a plugin's goal what its code said.
    pub fn set_wanted(&self, key: &str, wanted: bool) -> bool {
        let goal = self.goals.iter().find(|g| g.goal.plugin_key() == Some(key));
        goal.map(|g| g.goal.set_wanted(wanted)).is_some()
    }
    /// `GoalSelector.removeGoal`: stopped first, and nothing locked moves.
    pub fn remove_where(&mut self, m: &mut Mob, cx: &mut Cx, wanted: impl Fn(&dyn Goal) -> bool) -> bool {
        let mut found = false;
        let mut i = 0;
        while i < self.goals.len() {
            if !wanted(&*self.goals[i].goal) {
                i += 1;
                continue;
            }
            self.stop(i, m, cx);
            self.goals.remove(i);
            for slot in &mut self.locked {
                *slot = match *slot {
                    Some(j) if j == i => None,
                    Some(j) if j > i => Some(j - 1),
                    other => other,
                };
            }
            found = true;
        }
        found
    }
    /// One goal called from outside the tick, as a plugin may call any.
    pub fn ask(&mut self, name: &str, call: &str, m: &mut Mob, cx: &mut Cx) -> Option<bool> {
        let i = self.goals.iter().position(|g| g.goal.name() == name)?;
        let goal = &mut self.goals[i].goal;
        Some(match call {
            "use" => goal.can_use(m, cx),
            "continue" => goal.can_continue(m, cx),
            "start" => {
                goal.start(m, cx);
                true
            }
            "stop" => {
                goal.stop(m, cx);
                true
            }
            "tick" => {
                goal.tick(m, cx);
                true
            }
            _ => return None,
        })
    }
    pub fn running_names(&self) -> Vec<&\'static str> {'''),
    (MOBS + 'goals.rs', '// --- random positions (DefaultRandomPos / LandRandomPos / RandomPos) -------', '''/// A plugin's goal (`MobGoals.addGoal`). Its code can't run in a mob's tick, where the game holds what it would ask
/// for, so it says what it wants between ticks and the selector grants it here, by priority and flags as any goal.
#[derive(Clone, Debug)]
pub struct PluginGoal {
    key: String,
    flags: u8,
    wanted: Arc<std::sync::atomic::AtomicBool>,
}
impl PluginGoal {
    pub fn new(key: String, flags: u8) -> Self {
        Self { key, flags, wanted: Arc::default() }
    }
}
impl Goal for PluginGoal {
    fn flags(&self) -> u8 {
        self.flags
    }
    fn plugin_key(&self) -> Option<&str> {
        Some(&self.key)
    }
    fn set_wanted(&self, wanted: bool) {
        self.wanted.store(wanted, std::sync::atomic::Ordering::Relaxed);
    }
    fn can_use(&mut self, _m: &mut Mob, _cx: &mut Cx) -> bool {
        self.wanted.load(std::sync::atomic::Ordering::Relaxed)
    }
    goal_clone!();
}

// --- random positions (DefaultRandomPos / LandRandomPos / RandomPos) -------'''),
    (LIB + 'lan/hub.rs', '        let health = (g.health, g.food, g.saturation == 0.0);', '''        let most = g.motion.as_mut().and_then(|m| m.attribute("minecraft:max_health").ok()).unwrap_or(20.0);
        let health = (crate::paper_hub::shown_health(g.entity_id, g.health, most), g.food, g.saturation == 0.0);'''),
    (LIB + 'lan/hub.rs', '                    health: g.health,\n                    food: g.food,\n                    saturation: g.saturation,\n                }),\n            );\n            g.sent_health = Some(health);', '                    health: health.0,\n                    food: g.food,\n                    saturation: g.saturation,\n                }),\n            );\n            g.sent_health = Some(health);'),
    (LIB + 'lan/hub.rs', '            let values = super::attributes::take_dirty(&mut m.attributes);', '''            let mut values = super::attributes::take_dirty(&mut m.attributes);
            // `CraftPlayer.collectHealthScaleAttribute`: the bar's top stays what the plugin said.
            if let Some(scale) = crate::paper_hub::health_scale(g.entity_id) {
                let id = crate::paper_hub::max_health_attribute();
                for row in values.iter_mut().filter(|row| Some(row.attribute) == id) {
                    row.base = scale;
                    row.modifiers.clear();
                }
            }'''),
    (LIB + 'mobs/ai.rs', '    if in_snow && can_freeze {\n        m.ticks_frozen =', '    if m.extra.plugin_flag("freeze_locked") == Some(true) {\n    } else if in_snow && can_freeze {\n        m.ticks_frozen ='),
    (GAME + 'lan.rs', '        self.lan_object_states(host_entity, hub, out);', '        self.lan_object_states(host_entity, hub, out);\n        crate::game::paper_fam_entity::shared_flags_tail(out);'),
    (GAME + 'dispenser.rs', '    pub(super) fn mob_placed(&mut self, id: u32) -> Result<()> {', '    pub(super) fn mob_placed(&mut self, id: u32) -> Result<()> {\n        crate::game::paper_fam_entity::from_bucket(self, id);'),
    (GAME + 'dispenser.rs', '    fn shear_entity(&mut self, target: Pos) -> Result<bool> {', '    pub(super) fn shear_entity(&mut self, target: Pos) -> Result<bool> {'),
    (GAME + 'dispenser.rs', '            if self.shear_off_leashes(t) {', '            if crate::game::paper_fam_entity::shear_only().is_none() && self.shear_off_leashes(t) {'),
    (GAME + 'dispenser.rs', '            let Target::Mob(i) = t else { continue };\n            let m = &self.mobs[i];\n            if !m.alive() {', '            let Target::Mob(i) = t else { continue };\n            let m = &self.mobs[i];\n            if crate::game::paper_fam_entity::shear_only().is_some_and(|id| id != m.id) {\n                continue;\n            }\n            if !m.alive() {'),
    (GAME + 'dispenser.rs', '                Species::Sheep if !m.extra.sheared && !m.is_baby() => {', '                Species::Sheep if !m.extra.sheared && (!m.is_baby() || crate::game::paper_fam_entity::shear_only().is_some()) => {'),
    (GAME + 'commands/terrain_cmds.rs', '                host.hub.broadcast(&packet);', '''                if horizon_client::paper_hub::plugin_marks_empty(4) {
                    host.hub.broadcast(&packet);
                } else {
                    // `Player.setWorldBorder`: who has their own hears nothing of this one.
                    for guest in host.hub.guests.iter().filter(|guest| host.hub.here(guest) && !horizon_client::paper_hub::plugin_marked(4, guest.entity_id as u32)) {
                        guest.send(&packet);
                    }
                }'''),
    (LIB + 'mobs.rs', '    pub plugin: Option<Box<horizon_world::nbt::Tag>>,', '    pub plugin: Option<Box<horizon_world::nbt::Tag>>,\n    /// `LivingEntity.hurtDir`, in degrees from the way it faces.\n    pub hurt_dir: f32,'),
    (MOBS + 'living.rs', '        let mut wanted = self.velocity;\n        // Entity.move: a block the mob is stuck in', '        if self.extra.plugin_flag("no_physics") == Some(true) {\n            let v = self.velocity;\n            self.position = Vec3::new(self.position.x + v.x, self.position.y + v.y, self.position.z + v.z);\n            return;\n        }\n        let mut wanted = self.velocity;\n        // Entity.move: a block the mob is stuck in'),
    (LIB + 'motion.rs', '    pub ticks_frozen: i32,', '    pub ticks_frozen: i32,\n    /// `Entity.lockFreezeTicks`.\n    pub freeze_locked: bool,'),
    (LIB + 'motion.rs', '            ticks_frozen: 0,', '            ticks_frozen: 0,\n            freeze_locked: false,'),
    (LIB + 'motion.rs', '        }\n        // LivingEntity.aiStep "freezing" (server).\n        if !self.player.in_powder_snow || !self.player.can_freeze() {', '        }\n        // LivingEntity.aiStep "freezing" (server).\n        if !self.player.freeze_locked && (!self.player.in_powder_snow || !self.player.can_freeze()) {'),
    (LIB + 'motion.rs', '        self.effects_from_blocks(true)?;\n        // LivingEntity.aiStep "freezing" (server).\n        if !self.player.in_powder_snow || !self.player.can_freeze() {', '        self.effects_from_blocks(true)?;\n        // LivingEntity.aiStep "freezing" (server).\n        if !self.player.freeze_locked && (!self.player.in_powder_snow || !self.player.can_freeze()) {'),
    (LIB + 'motion.rs', '            if self.player.can_freeze() {\n                self.player.ticks_frozen =', '            if self.player.can_freeze() && !self.player.freeze_locked {\n                self.player.ticks_frozen ='),
    (LIB + 'mobs.rs', '    pub fn knockback(&mut self, power: f64, mut xd: f64, mut zd: f64) {', '''    pub fn knockback(&mut self, power: f64, mut xd: f64, mut zd: f64) {
        // `LivingEntity.indicateDamage`, for `getHurtDirection`.
        self.extra.hurt_dir = (zd.atan2(xd).to_degrees() - f64::from(self.yaw)) as f32;'''),
    (GAME + 'lan_actions.rs', '        self.guest_drop_carried(host, i);\n        self.guest_container_closed(host, conn);', '        crate::game::paper_fam_entity::score_reset(self.ui.rules.bool("keep_inventory"), &mut host.hub.guests[i]);\n        self.guest_drop_carried(host, i);\n        self.guest_container_closed(host, conn);'),
    (LIB + 'game.rs', '        let at = self.eye_position();\n        if let Some(level) = self.experience.give(gained as i32)', '        crate::game::paper_fam_entity::score_up(self, gained);\n        let at = self.eye_position();\n        if let Some(level) = self.experience.give(gained as i32)'),
    # ---- Each kind's shot by itself, a plugin's spawn as CUSTOM, a chunk's going noted.
    (MOBS + 'golem.rs', 'fn snowball_attack(m: &mut Mob, who: Who, _power: f32, cx: &mut Cx) {', 'pub fn snowball_attack(m: &mut Mob, who: Who, _power: f32, cx: &mut Cx) {'),
    (MOBS + 'illager.rs', 'fn shoot_crossbow(m: &mut Mob, who: Who, cx: &mut Cx) {', 'pub fn shoot_crossbow(m: &mut Mob, who: Who, cx: &mut Cx) {'),
    (MOBS + 'llama.rs', 'fn spit(m: &mut Mob, who: Who, _power: f32, cx: &mut Cx) {', 'pub fn spit(m: &mut Mob, who: Who, _power: f32, cx: &mut Cx) {'),
    (MOBS + 'undead.rs', 'fn drowned_throw(m: &mut Mob, who: Who, _power: f32, cx: &mut Cx) {', 'pub fn drowned_throw(m: &mut Mob, who: Who, _power: f32, cx: &mut Cx) {'),
    (MOBS + 'witch.rs', 'fn witch_attack(m: &mut Mob, who: Who, _power: f32, cx: &mut Cx) {', 'pub fn witch_attack(m: &mut Mob, who: Who, _power: f32, cx: &mut Cx) {'),
    (MOBS + 'wither.rs', 'fn shoot(m: &mut Mob, head: usize, at: [f64; 3], dangerous: bool, events: &mut Vec<MobEvent>) {', 'pub fn shoot(m: &mut Mob, head: usize, at: [f64; 3], dangerous: bool, events: &mut Vec<MobEvent>) {'),
    (GAME + 'persist.rs', '            let gone = |p: Vec3| keys.contains(&chunk_of(p));\n            let mut removed: Vec<u32> = Vec::new();', '            let gone = |p: Vec3| keys.contains(&chunk_of(p));\n            crate::game::paper_fam_entity::note_unloaded(self, &gone);\n            let mut removed: Vec<u32> = Vec::new();'),
    (GAME + 'commands.rs', '        pub(in crate::game) fn native_spawn(\n            &mut self,\n            dimension: &str,\n            snbt: &str,\n        ) -> Value {', '''        pub(in crate::game) fn native_spawn(&mut self, dimension: &str, snbt: &str) -> Value {
            let answer = self.native_spawn_inner(dimension, snbt);
            crate::game::paper_fam_entity::spawned_by_plugin(self, dimension, answer)
        }
        fn native_spawn_inner(
            &mut self,
            dimension: &str,
            snbt: &str,
        ) -> Value {'''),
    # ---- Player.setPlayerProfile: the player's own client, as `CraftPlayer.refreshPlayer` does it.
    (LIB + 'lan/hub.rs', '    /// A respawn point whose block is gone or blocked: the respawning player', '''    /// `CraftPlayer.refreshPlayer`: a Respawn that keeps all, then all a client forgets with it.
    fn refresh_own(&mut self, i: usize, ctx: &TickCtx) {
        let info = self.spawn_info(i, 0);
        let g = &mut self.guests[i];
        g.send(&C::Respawn(play::Respawn { spawn: info, keep: 3 }));
        g.send(&C::PlayerAbilities(play::PlayerAbilities { flags: g.ability_flags(), flying_speed: 0.05, walking_speed: 0.1 }));
        g.sent_effects.clear();
        g.sent_health = None;
        g.sent_xp = None;
        self.resend_teleport(i, None);
        self.send_permission_level(i);
        self.send_level_info(i, ctx, true);
        self.send_inventory(i);
    }
    /// A respawn point whose block is gone or blocked: the respawning player'''),
    (LIB + 'lan/hub.rs', '                self.sync_player_state(i);', '                if crate::paper_hub::take_refresh(self.guests[i].entity_id) {\n                    self.refresh_own(i, ctx);\n                }\n                self.sync_player_state(i);'),
]
