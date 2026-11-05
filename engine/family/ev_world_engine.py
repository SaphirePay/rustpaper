# Family ev_world. Yours alone. Applied by fabric-port/tools/dim_patch.py, after the mixins: each text must be in its file once.
GAME = 'horizon-client/src/game/'
LEVEL = 'horizon-world/src/level/'

EDITS = [
    # ---- What a campfire cooked, for `BlockCookEvent`.
    (GAME + 'campfire.rs', '''            self.campfire_drop(pos, result);
            self.mark_block_entity_updated(pos);''', '''            super::paper_fam_ev_world::note_cooked(self, &item);
            self.campfire_drop(pos, result);
            self.mark_block_entity_updated(pos);'''),
    # ---- Loot a listener has already seen and changed: what shears give.
    (GAME + 'dispenser.rs', '''    ) -> Vec<ItemStack> {
        match world_items!(self).loot_sequence(parent) {''', '''    ) -> Vec<ItemStack> {
        if let Some(loot) = super::paper_fam_ev_world::loot_next(table) {
            return loot;
        }
        match world_items!(self).loot_sequence(parent) {'''),
    # ---- A dispenser's velocity.
    (GAME + 'dispenser.rs', '''            r.triangle(f64::from(s[2]) * pow, spread),
        ];
        self.spawn_item(stack, at, v, 0);''', '''            r.triangle(f64::from(s[2]) * pow, spread),
        ];
        let v = super::paper_fam_ev_world::dispense_velocity().unwrap_or(v);
        self.spawn_item(stack, at, v, 0);'''),
    (GAME + 'dispenser.rs', '''        self.shoot_projectile_item(at, dir, &stack, power, uncertainty);''', '''        let dir = super::paper_fam_ev_world::dispense_velocity().unwrap_or(dir);
        self.shoot_projectile_item(at, dir, &stack, power, uncertainty);'''),
    # ---- `SculkBloomEvent`: the charge a catalyst is about to take.
    (GAME + 'sculk.rs', '''        let xp = experience(self);
        if xp > 0 {''', '''        let xp = experience(self);
        let xp = super::paper_fam_ev_world::bloom_charge(self, key, xp);
        if xp > 0 {'''),
    # ---- `GenericGameEvent.setRadius`.
    (GAME + 'sculk.rs', '''        let radius = sculk::notification_radius(event).min(16);''', '''        let radius = super::paper_fam_ev_world::take_radius().unwrap_or_else(|| sculk::notification_radius(event).min(16));'''),
    # ---- `StructuresLocateEvent.setRadius`.
    # ---- `BellResonateEvent` and `BellRevealRaiderEvent`: who glows.
    (GAME + 'block_entity_anims.rs', '''                        for m in self.mobs.iter_mut().filter(|_| self.remote.is_none()) {
                            if glow.contains(&u64::from(m.id)) {''', '''                        let glow = super::paper_fam_ev_world::bell_glow(self, pos, glow);
                        for m in self.mobs.iter_mut().filter(|_| self.remote.is_none()) {
                            if glow.contains(&u64::from(m.id)) {'''),
    # ---- `BlockPistonEvent.isSticky`: the piston's kind, for the hook that follows.
    (LEVEL + 'pistons.rs', '''use super::{
    Behaviours, LevelAccess, LevelEvent, Sim, Ticks,
    data::PushReaction,''', '''/// Whether the piston asking to move is a sticky one.
pub static PISTON_STICKY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

use super::{
    Behaviours, LevelAccess, LevelEvent, Sim, Ticks,
    data::PushReaction,'''),
    (LEVEL + 'pistons.rs', '''        if !self
            .level
            .allow_piston_move(piston_pos, push_direction, &to_push, &to_destroy)''', '''        PISTON_STICKY.store(sticky, std::sync::atomic::Ordering::Relaxed);
        if !self
            .level
            .allow_piston_move(piston_pos, push_direction, &to_push, &to_destroy)'''),
    # ---- A chunk a listener doesn't want saved.
    (GAME + '../game.rs', '''            .filter(|c| self.world.modified.remove(&(c.x, c.z)))
            .cloned()
            .collect();
        let modified = horizon_client::spans::time(''', '''            .filter(|c| self.world.modified.remove(&(c.x, c.z)) && paper_fam_ev_world::chunk_saved((c.x, c.z)))
            .cloned()
            .collect();
        let modified = horizon_client::spans::time('''),
    # ---- A vehicle running into a block.
    (GAME + 'minecarts.rs', '''            c.horizontal_collision = (moved.x - delta.x).abs() > 1.0e-7
                || (moved.z - delta.z).abs() > 1.0e-7;''', '''            c.horizontal_collision = (moved.x - delta.x).abs() > 1.0e-7
                || (moved.z - delta.z).abs() > 1.0e-7;
            if c.horizontal_collision {
                super::paper_fam_ev_world::note_vehicle_hit(c.id, [c.position.x, c.position.y, c.position.z], [delta.x, delta.y, delta.z], [moved.x, moved.z]);
            }'''),
    (GAME + 'boats.rs', '''            b.on_ground = b.velocity.y < 0.0 && moved.y > b.velocity.y;
            if (moved.x - b.velocity.x).abs() > 1.0e-7 {''', '''            b.on_ground = b.velocity.y < 0.0 && moved.y > b.velocity.y;
            if (moved.x - b.velocity.x).abs() > 1.0e-7 || (moved.z - b.velocity.z).abs() > 1.0e-7 {
                super::paper_fam_ev_world::note_vehicle_hit(b.id, [b.position.x, b.position.y, b.position.z], [b.velocity.x, b.velocity.y, b.velocity.z], [moved.x, moved.z]);
            }
            if (moved.x - b.velocity.x).abs() > 1.0e-7 {'''),
    # ---- A mob a boat is about to take on.
    (GAME + 'boat_passengers.rs', '''                    if room && m.extra.boarding_cooldown <= 0 {
                        let m = &mut self.mobs[j];''', '''                    if room && m.extra.boarding_cooldown <= 0 && !super::paper_fam_ev_world::boarding_refused(id, m.id) {
                        let m = &mut self.mobs[j];'''),
    # ---- Who runs a console line: RCON says `remote`.
    (GAME + 'dedicated/status.rs', '''                Some(("console", line)) => {''', '''                Some((kind @ ("console" | "remote"), line)) => {
                    let line = crate::game::paper_fam_core::remote_line(kind == "remote", line);
                    crate::game::paper_fam_ev_world::note_remote(kind == "remote");'''),
    # `remote <address> <line>`: the address is one word (`rcon.rs`'s `peer`), for `RemoteConsoleCommandSender.getAddress`.
    ('horizon-server/src/rcon.rs', '''game.ask(&format!("console {}", line.trim_start_matches('/')))''', '''game.ask(&format!("remote {} {}", peer(), line.trim_start_matches('/')))'''),
    ('horizon-server/src/rcon.rs', '''"console say hi" =>''', '''"remote - say hi" =>'''),
    # ---- The query port, once the server has its listener.
    (GAME + 'dedicated/status.rs', '''    pub(super) fn poll_server_control(&mut self, state: &mut ServerState) {
''', '''    pub(super) fn poll_server_control(&mut self, state: &mut ServerState) {
        crate::game::paper_fam_ev_world::start_query(self);
'''),
    ('horizon-client/src/lan/mod.rs', '''    pub fn max_players(&self) -> usize {
        self.max_players.load(Ordering::Relaxed)
    }
''', '''    pub fn max_players(&self) -> usize {
        self.max_players.load(Ordering::Relaxed)
    }
    /// The game's version, for the query port.
    pub fn game_version(&self) -> &'static str {
        GAME_VERSION
    }
    /// Everyone online by name, for the query port.
    pub fn online_names(&self) -> Vec<String> {
        self.online.lock().unwrap().iter().map(|(_, name)| name.clone()).collect()
    }
'''),
    # ---- `ProfileWhitelistVerifyEvent`: the whitelist's say, put to the listeners outside the lists' lock.
    (GAME + 'dedicated.rs', '''            gate_lists.lock().unwrap().can_login(address, &user)
''', '''            let (enabled, op, listed) = {
                let lists = gate_lists.lock().unwrap();
                (lists.use_whitelist, lists.ops.contains(&user), lists.whitelisted(&user))
            };
            let asked = horizon_client::paper_hub::whitelist_ask(&user.name, user.id, enabled, op, listed);
            let refused = gate_lists.lock().unwrap().can_login(address, &user);
            let not_listed = translatable("multiplayer.disconnect.not_whitelisted", &[]);
            match asked {
                Some((true, _)) if refused.as_ref() == Some(&not_listed) => None,
                Some((false, kick)) if refused.is_none() => Some(kick.map_or(not_listed, |kick| Tag::String(JavaString::from(kick.as_str())))),
                _ => refused,
            }
'''),
    # ---- `BlockDestroyEvent`: the level's own destroying, with no game to lend.
    (LEVEL + 'update.rs', '''        // Block.spawnDestroyParticles: fire spawns none, straw beds their own.
        let data = self.b.data(state);
        if data.is(Class::StrawBed) {
            self.level_event(2014, pos, state.0 as i32);
        } else if !(data.is(Class::BaseFire) || data.is(Class::Fire) || data.is(Class::SoulFire)) {
            self.level_event(2001, pos, state.0 as i32);
        }
        if drop {''', '''        if self.asked(super::Asked::Destroy { at: pos, state, drop, limit }) {
            return true;
        }
        self.destroy_told(pos, state, drop, Some(state), limit)
    }
    /// The rest of `Level.destroyBlock`, once its listeners have spoken.
    pub(crate) fn destroy_told(&mut self, pos: BlockPos, state: crate::registry::BlockStateId, drop: bool, effect: Option<crate::registry::BlockStateId>, limit: i32) -> bool {
        // Block.spawnDestroyParticles: fire spawns none, straw beds their own.
        let data = self.b.data(state);
        if let Some(effect) = effect {
            if data.is(Class::StrawBed) {
                self.level_event(2014, pos, effect.0 as i32);
            } else if !(data.is(Class::BaseFire) || data.is(Class::Fire) || data.is(Class::SoulFire)) {
                self.level_event(2001, pos, effect.0 as i32);
            }
        }
        if drop {'''),
    (LEVEL + 'mod.rs', '''pub mod pistons;
''', '''pub mod pistons;

'''),
    # ---- `SpongeAbsorbEvent`: what the water was, for what a listener refuses.
    (LEVEL + 'hooks.rs', '''    pub(crate) fn sponge_try_absorb(&mut self, start: BlockPos) {
        if self.sponge_remove_water(start) {
            let b = self.b;
''', '''    pub(crate) fn sponge_try_absorb(&mut self, start: BlockPos) {
        let mut planned = Vec::new();
        if self.sponge_remove_water(start, &mut planned, None) && self.asked(super::Asked::Sponge { at: start, blocks: planned }) {
            return;
        }
        self.sponge_absorbs(start, None);
    }
    /// The water going, all of it or what the listeners left, and the sponge wet.
    pub(crate) fn sponge_absorbs(&mut self, start: BlockPos, kept: Option<&[BlockPos]>) {
        let mut taken = Vec::new();
        if self.sponge_remove_water(start, &mut taken, Some(kept)) {
            let b = self.b;
'''),
    (LEVEL + 'hooks.rs', '''    fn sponge_remove_water(&mut self, start: BlockPos) -> bool {''', '''    fn sponge_remove_water(&mut self, start: BlockPos, taken: &mut Vec<(BlockPos, BlockStateId)>, real: Option<Option<&[BlockPos]>>) -> bool {'''),
    (LEVEL + 'hooks.rs', '''            if !visited.insert(pos) {
                continue;
            }
            let accept = if pos == start {''', '''            if !visited.insert(pos) {
                continue;
            }
            let before = self.state(pos);
            let accept = if pos == start {'''),
    (LEVEL + 'hooks.rs', '''            if !accept {
                continue;
            }
            count += 1;
            if count >= 65 {
                return count > 1;
            }''', '''            if !accept {
                continue;
            }
            if pos != start {
                let after = if b.table.has(before, b.props.waterlogged) { b.table.set_bool(before, b.props.waterlogged, false) } else { b.air() };
                taken.push((pos, after));
            }
            count += 1;
            if count >= 65 {
                return count > 1;
            }'''),
    (LEVEL + 'hooks.rs', '''                let state = self.state(pos);
                if !b.is_water_fluid(self.fluid_at(pos).kind()) {
                    false
                } else if self.pickup_block(pos, state, false) {
                    true
                } else {
                    if b.instance_of(state, Class::Liquid) {
                        self.set_block_and_update(pos, b.air());
                    } else {
''', '''                let state = self.state(pos);
                let leave = real.is_none_or(|kept| kept.is_some_and(|kept| !kept.contains(&pos)));
                let gives_up = (b.implements(state, super::classes::Interface::SimpleWaterlogged) && b.table.has(state, b.props.waterlogged) && !b.instance_of(state, Class::Barrier) && b.is_waterlogged(state))
                    || b.is_block(state, b.named.bubble_column)
                    || (b.instance_of(state, Class::Liquid) && b.table.int(state, b.props.level) == 0);
                if !b.is_water_fluid(self.fluid_at(pos).kind()) {
                    false
                } else if gives_up {
                    if !leave {
                        self.pickup_block(pos, state, false);
                    }
                    true
                } else {
                    if b.instance_of(state, Class::Liquid) {
                        if !leave {
                            self.set_block_and_update(pos, b.air());
                        }
                    } else {
'''),
    (LEVEL + 'hooks.rs', '''                        self.drop_resources(state, pos);
                        self.set_block_and_update(pos, b.air());
                    }
                    true
                }
            };
            if !accept {
''', '''                        if !leave {
                            self.drop_resources(state, pos);
                            self.set_block_and_update(pos, b.air());
                        }
                    }
                    true
                }
            };
            if !accept {
'''),
    # ---- `BlockBreakBlockEvent`: what breaks what, for the drops that follow.
    (LEVEL + 'pistons.rs', '''pub static PISTON_STICKY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
''', '''pub static PISTON_STICKY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// The blocks a piston or water is breaking, and what breaks them, until their drops are worked out.
pub static BROKEN_BY: std::sync::Mutex<Vec<([i32; 3], [i32; 3])>> = std::sync::Mutex::new(Vec::new());
'''),
    (LEVEL + 'pistons.rs', '''        PISTON_STICKY.store(sticky, std::sync::atomic::Ordering::Relaxed);
''', '''        PISTON_STICKY.store(sticky, std::sync::atomic::Ordering::Relaxed);
        BROKEN_BY.lock().unwrap_or_else(|e| e.into_inner()).extend(to_destroy.iter().map(|pos| ([pos.x, pos.y, pos.z], [piston_pos.x, piston_pos.y, piston_pos.z])));
'''),
    (LEVEL + 'fluids.rs', '''                if !self.can_survive(state, pos) {
                    self.destroy_block(pos, true, 512);
                }
                return true;''', '''                if !self.can_survive(state, pos) {
                    super::pistons::BROKEN_BY.lock().unwrap_or_else(|e| e.into_inner()).push(([pos.x, pos.y, pos.z], [pos.x, pos.y, pos.z]));
                    self.destroy_block(pos, true, 512);
                }
                return true;'''),
    (GAME + 'spill.rs', '''        for stack in drops {
            if &*stack.item != "minecraft:air" && stack.count > 0 {
                self.pop_resource(at, stack);
            }
        }
        self.pending.clear(at);''', '''        let drops = super::paper_fam_ev_world::broke_block(self.dimension, at, drops);
        for stack in drops {
            if &*stack.item != "minecraft:air" && stack.count > 0 {
                self.pop_resource(at, stack);
            }
        }
        self.pending.clear(at);'''),
    # ---- `AsyncStructureGenerateEvent`: a structure about to go into a chunk, and what a transformer does to it.
    ('horizon-world/src/worldgen/structure/start.rs', '''/// StructureStart.placeInChunk for pieces intersecting the chunk's writable box.
''', '''use crate::registry::BlockStateId;

/// Asked before a structure is placed in a chunk: the level, the structure's name, its box, the chunk and
/// why. 0 skips it, 1 places it, 2 places it and hands `STRUCTURE_WRITES_HOOK` what it wrote.
pub static STRUCTURE_HOOK: std::sync::OnceLock<fn(&str, &str, [i32; 3], [i32; 3], [i32; 2], &str) -> u8> = std::sync::OnceLock::new();

/// What `STRUCTURE_WRITES_HOOK` can reach: the blocks around the structure.
pub trait StructureRegion {
    fn read(&mut self, position: [i32; 3]) -> Option<BlockStateId>;
    fn write(&mut self, position: [i32; 3], state: BlockStateId) -> bool;
}

/// What's to be done with an entity a structure brought.
pub enum EntityVerdict {
    Keep,
    Drop,
    Replace(crate::nbt::Compound),
}

/// A place a structure wrote: what was there and what the structure put.
pub type StructureWrite = ([i32; 3], BlockStateId, BlockStateId);
/// The blocks to change and each entity's verdict.
pub type StructureChanges = (Vec<([i32; 3], BlockStateId)>, Vec<EntityVerdict>);

/// Given what a structure wrote to a chunk and the entities it brought.
pub static STRUCTURE_WRITES_HOOK: std::sync::OnceLock<fn(&mut dyn StructureRegion, Vec<StructureWrite>, Vec<crate::nbt::Compound>) -> StructureChanges> = std::sync::OnceLock::new();

thread_local! {
    static STRUCTURE_WRITES: std::cell::RefCell<Option<Vec<StructureWrite>>> = const { std::cell::RefCell::new(None) };
    /// The level this thread is placing structures in.
    pub static STRUCTURE_LEVEL: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// Whether what's written now is wanted.
#[inline]
pub fn structure_writes_wanted() -> bool {
    STRUCTURE_WRITES.with_borrow(Option::is_some)
}

/// What was written since `start_structure_writes`.
pub fn take_structure_writes() -> Vec<StructureWrite> {
    STRUCTURE_WRITES.take().unwrap_or_default()
}

/// What's being kept, for `writes` in its place.
pub fn swap_structure_writes(writes: Option<Vec<StructureWrite>>) -> Option<Vec<StructureWrite>> {
    STRUCTURE_WRITES.replace(writes)
}

/// From here to `finish_structure_writes`, this thread's writes are kept.
pub fn start_structure_writes() {
    STRUCTURE_WRITES.set(Some(Vec::new()));
}

#[inline]
pub fn note_structure_write(position: [i32; 3], old: BlockStateId, new: BlockStateId) {
    STRUCTURE_WRITES.with_borrow_mut(|writes| {
        if let Some(writes) = writes {
            writes.push((position, old, new));
        }
    });
}

/// `STRUCTURE_WRITES_HOOK` on what was written, one row a place, and the entities that stay.
pub fn finish_structure_writes(region: &mut dyn StructureRegion, entities: Vec<crate::nbt::Compound>) -> Vec<crate::nbt::Compound> {
    let writes = STRUCTURE_WRITES.take().unwrap_or_default();
    let Some(hook) = STRUCTURE_WRITES_HOOK.get() else {
        return entities;
    };
    let mut places: Vec<StructureWrite> = Vec::new();
    let mut seen = std::collections::HashMap::new();
    for (position, old, new) in writes {
        match seen.get(&position) {
            Some(&row) => {
                let kept: &mut StructureWrite = &mut places[row];
                kept.2 = new;
            }
            None => {
                seen.insert(position, places.len());
                places.push((position, old, new));
            }
        }
    }
    let (changes, verdicts) = hook(region, places, entities.clone());
    for (position, state) in changes {
        region.write(position, state);
    }
    let mut verdicts = verdicts.into_iter();
    entities
        .into_iter()
        .filter_map(|entity| match verdicts.next() {
            Some(EntityVerdict::Drop) => None,
            Some(EntityVerdict::Replace(changed)) => Some(changed),
            _ => Some(entity),
        })
        .collect()
}

/// StructureStart.placeInChunk for pieces intersecting the chunk's writable box.
'''),
    ('horizon-world/src/worldgen/generator_decoration.rs', '''                    for start in references.iter().filter(|s| s.structure == structure) {
                        let mut place_feature =''', '''                    for start in references.iter().filter(|s| s.structure == structure) {
                        let level = crate::worldgen::structure::start::STRUCTURE_LEVEL.with_borrow(Clone::clone);
                        let want = crate::worldgen::structure::start::STRUCTURE_HOOK.get().map_or(1, |hook| {
                            hook(&level, &start.name, start.bounding_box.min, start.bounding_box.max, [chunk_box.min[0] >> 4, chunk_box.min[2] >> 4], "WORLD_GENERATION")
                        });
                        if want == 0 {
                            continue;
                        }
                        let first_entity = world.entities.len();
                        if want == 2 {
                            crate::worldgen::structure::start::start_structure_writes();
                        }
                        let mut place_feature ='''),
    ('horizon-world/src/worldgen/generator_decoration.rs', '''                        .is_err()
                        {
                            structure_errors += 1;
                        }
''', '''                        .is_err()
                        {
                            structure_errors += 1;
                        }
                        if want == 2 {
                            let brought = world.entities.split_off(first_entity);
                            let kept = crate::worldgen::structure::start::finish_structure_writes(world, brought);
                            world.entities.extend(kept);
                        }
'''),
    ('horizon-world/src/worldgen/generator_decoration.rs', '''        let old = self.region.block(position)?;
        self.region.set_block_worldgen(position, state)?;
        // ProtoChunk keeps''', '''        let old = self.region.block(position)?;
        crate::worldgen::structure::start::note_structure_write(position, old, state);
        self.region.set_block_worldgen(position, state)?;
        // ProtoChunk keeps'''),
    ('horizon-world/src/worldgen/generator_decoration.rs', '''impl FeatureWorld for World<'_> {
    type Error = RegionError;
''', '''impl crate::worldgen::structure::start::StructureRegion for World<'_> {
    fn read(&mut self, position: [i32; 3]) -> Option<BlockStateId> {
        self.region.block(position).ok()
    }
    fn write(&mut self, position: [i32; 3], state: BlockStateId) -> bool {
        self.set_block(position, state, 3).unwrap_or(false)
    }
}
impl FeatureWorld for World<'_> {
    type Error = RegionError;
'''),
    ('horizon-world/src/level/feature_world.rs', '''        Ok(self.sim.set_block(pos(position), state, flags))
    }
}

impl<L: LevelAccess + ?Sized> PlacementContext for LevelFeatureWorld<'_, '_, L> {''', '''        Ok(self.sim.set_block(pos(position), state, flags))
    }
}

impl<L: LevelAccess + ?Sized> crate::worldgen::structure::start::StructureRegion for LevelFeatureWorld<'_, '_, L> {
    fn read(&mut self, position: [i32; 3]) -> Option<BlockStateId> {
        Some(self.sim.state(pos(position)))
    }
    fn write(&mut self, position: [i32; 3], state: BlockStateId) -> bool {
        self.sim.set_block(pos(position), state, 3)
    }
}

impl<L: LevelAccess + ?Sized> PlacementContext for LevelFeatureWorld<'_, '_, L> {'''),
    (LEVEL + 'growth.rs', '''                        [cx * 16 + 15, high_y, cz * 16 + 15],
                    );
                    if crate::worldgen::structure::start::place_in_chunk(
                        start,
                        &mut world,
                        resources,
                        &blocks,
                        &mut random,
                        chunk_box,
                        &mut place_feature,
                    )
                    .is_err()
                    {
                        placed = false;
                    }
''', '''                        [cx * 16 + 15, high_y, cz * 16 + 15],
                    );
                    let level_name = crate::worldgen::structure::start::STRUCTURE_LEVEL.with_borrow(Clone::clone);
                    let want = crate::worldgen::structure::start::STRUCTURE_HOOK.get().map_or(1, |hook| {
                        hook(&level_name, &start.name, min, max, [cx, cz], "COMMAND")
                    });
                    if want == 0 {
                        continue;
                    }
                    let first_entity = world.entities.len();
                    if want == 2 {
                        crate::worldgen::structure::start::start_structure_writes();
                    }
                    if crate::worldgen::structure::start::place_in_chunk(
                        start,
                        &mut world,
                        resources,
                        &blocks,
                        &mut random,
                        chunk_box,
                        &mut place_feature,
                    )
                    .is_err()
                    {
                        placed = false;
                    }
                    if want == 2 {
                        let brought = world.entities.split_off(first_entity);
                        let kept = crate::worldgen::structure::start::finish_structure_writes(&mut world, brought);
                        world.entities.extend(kept);
                    }
'''),
    (GAME + 'commands/place.rs', '''                let (placed, entities) =
                    self.with_level(|b, level, ticks| b.place_structure(level, ticks, &start));
''', '''                let level_name = format!("minecraft:{}", self.dim.id().trim_start_matches("minecraft:"));
                horizon_world::worldgen::structure::start::STRUCTURE_LEVEL.set(level_name);
                let (placed, entities) =
                    self.with_level(|b, level, ticks| b.place_structure(level, ticks, &start));
'''),
    ('horizon-client/src/vanillagen.rs', '''            if let Err(e) = worker.decorate(&mut array) {
                log::warn!("Horizon: decorating chunk failed: {e}");''', '''            horizon_world::worldgen::structure::start::STRUCTURE_LEVEL.with_borrow_mut(|level| level.clone_from(&self.dimension));
            if let Err(e) = worker.decorate(&mut array) {
                log::warn!("Horizon: decorating chunk failed: {e}");'''),
    # ---- `CommandRegisteredEvent`: the tree as it's built.
    (GAME + 'native_server.rs', '''        self.native_register_commands(&mut commands);
''', '''        self.native_register_commands(&mut commands);
        super::paper_fam_ev_world::sync_commands(self, &mut commands);
'''),
    # ---- A plugin's data packs go in before the first read of them.
    ('horizon-client-render/src/datapack.rs', '''pub fn discover_with_features(world_root: &Path, resources: &Path) -> Vec<DataPack> {
    let mut packs = discover(world_root);
''', '''pub fn discover_with_features(world_root: &Path, resources: &Path) -> Vec<DataPack> {
    if let Some(hook) = BEFORE_DISCOVERY.get() {
        hook(resources);
    }
    let mut packs = discover(world_root);
'''),
    ('horizon-client-render/src/datapack.rs', '''pub fn discover(world: &Path) -> Vec<DataPack> {
''', '''/// Called before a world's packs are looked for, with the game's own data.
pub static BEFORE_DISCOVERY: std::sync::OnceLock<fn(&Path)> = std::sync::OnceLock::new();

pub fn discover(world: &Path) -> Vec<DataPack> {
'''),
    # ---- `LightningStrikeEvent.getLightning`: with a listener, the bolt is in the level as the event fires.
    (GAME + 'lightning.rs', '''    /// `ClientLevel.addEntity` for a server's lightning bolt: on a client it
''', '''    /// The bolt `LightningStrikeEvent` names: its id, for a listener to find it by.
    pub(super) fn bolt_ahead(&mut self, at: [f64; 3], visual_only: bool) -> u32 {
        let r = &mut self.world.random;
        let seed = r.next_long();
        let flashes = r.next_int_bound(3).unwrap_or(0) + 1;
        self.next_entity += 1;
        let id = self.next_entity;
        self.bolts.push(Bolt {
            id,
            at,
            life: 2,
            flashes,
            seed,
            visual_only,
            converted: Vec::new(),
            hit: Vec::new(),
            hit_items: Vec::new(),
            spared: Vec::new(),
            fires: 0,
            cause: None,
        });
        super::paper_fam_ev_world::bolt_ahead(id);
        id
    }
    /// A cancelled `LightningStrikeEvent`.
    pub(super) fn drop_bolt_ahead(&mut self) {
        if let Some(id) = super::paper_fam_ev_world::take_bolt_ahead() {
            self.bolts.retain(|bolt| bolt.id != id);
        }
    }
    /// `ClientLevel.addEntity` for a server's lightning bolt: on a client it
'''),
    (GAME + 'lightning.rs', '''            self.post_game_event("minecraft:lightning_strike", at, ctx);
        }
        let r = &mut self.world.random;
''', '''            self.post_game_event("minecraft:lightning_strike", at, ctx);
        }
        if super::paper_fam_ev_world::take_bolt_ahead().is_some() {
            self.lightning_seen[0] += 1;
            return;
        }
        let r = &mut self.world.random;
'''),
    # ---- `EntitiesLoadEvent` and `EntitiesUnloadEvent`: a chunk's entities, together.
    (GAME + 'persist.rs', '''    pub(super) fn spawn_loaded_entities(&mut self) {
        let loaded = std::mem::take(&mut self.world.loaded_entities);
''', '''    pub(super) fn spawn_loaded_entities(&mut self) {
        let chunks: Vec<(i32, i32)> = self.world.loaded_entities.iter().filter(|(_, tags)| !tags.is_empty()).map(|(key, _)| *key).collect();
        self.spawn_entities_that_loaded();
        super::paper_fam_ev_world::entities_moved(self, "ew_entities_load", &chunks);
    }
    fn spawn_entities_that_loaded(&mut self) {
        let loaded = std::mem::take(&mut self.world.loaded_entities);
'''),
    (GAME + 'persist.rs', '''        let keys: HashSet<(i32, i32)> = unloaded.iter().map(|c| (c.x, c.z)).collect();
        let chunks = self.entity_chunks(&keys, true);
        if let Err(e) = self.save.save_entities_later(chunks) {''', '''        let keys: HashSet<(i32, i32)> = unloaded.iter().map(|c| (c.x, c.z)).collect();
        let leaving: Vec<(i32, i32)> = keys.iter().copied().collect();
        super::paper_fam_ev_world::entities_moved(self, "ew_entities_unload", &leaving);
        let chunks = self.entity_chunks(&keys, true);
        if let Err(e) = self.save.save_entities_later(chunks) {'''),
    # ---- `WeatherChangeEvent` and `ThunderChangeEvent`, at each of the three changes.
    ('horizon-client/src/game.rs', '''                self.world.weather.tick_timers(&mut self.world.random);
''', '''                let sky = (self.world.weather.raining, self.world.weather.thundering);
                self.world.weather.tick_timers(&mut self.world.random);
                paper_fam_ev_world::weather_moved(self, sky, "NATURAL");
'''),
    ('horizon-client/src/game.rs', '''                self.world.weather.raining = false;
                self.world.weather.thundering = false;
                self.world.weather.rain_time = 0;
                self.world.weather.thunder_time = 0;
''', '''                let sky = (self.world.weather.raining, self.world.weather.thundering);
                self.world.weather.raining = false;
                self.world.weather.thundering = false;
                self.world.weather.rain_time = 0;
                self.world.weather.thunder_time = 0;
                paper_fam_ev_world::weather_moved(self, sky, "SLEEP");
'''),
    (GAME + 'commands.rs', '''        w.raining = raining;
        w.thundering = thundering;
        self.feedback(Text::translate(key, &[]));''', '''        let sky = (w.raining, w.thundering);
        w.raining = raining;
        w.thundering = thundering;
        super::paper_fam_ev_world::weather_moved(self, sky, "COMMAND");
        self.feedback(Text::translate(key, &[]));'''),
    # ---- `LootGenerateEvent`: a container's first opener.
    (GAME + 'commands.rs', '''        if still_rolled {
            self.trigger_loot_table(&rolled.table);
        }
''', '''        if still_rolled {
            super::paper_fam_ev_world::chest_loot(self, key, &rolled.table);
            self.trigger_loot_table(&rolled.table);
        }
'''),
    # ---- `PortalCreateEvent`: any fire in a frame, and the portal made for a traveller.
    (LEVEL + 'hooks.rs', '''        ) && let Some(shape) = self.find_empty_portal_shape(pos, true)
        {
            self.create_portal_blocks(&shape);
            return;
        }
''', '''        ) && let Some(shape) = self.find_empty_portal_shape(pos, true)
        {
            if !self.portal_asked(&shape) {
                self.create_portal_blocks(&shape);
            }
            return;
        }
'''),
    (LEVEL + 'portal.rs', '''    /// `PortalShape.createPortalBlocks`: fill the frame (flags 18).
''', '''    /// Whether this frame's filling is left for `PortalCreateEvent`'s listeners.
    pub(super) fn portal_asked(&mut self, shape: &PortalShape) -> bool {
        let b = self.b;
        let base = b.default_of(b.named.nether_portal);
        let state = b
            .table
            .with_text(base, b.props.axis, if shape.x_axis { "x" } else { "z" })
            .unwrap_or(base);
        let mut rows = Vec::new();
        for h in 0..shape.height {
            for w in 0..shape.width {
                rows.push((step(step(shape.bottom_left, Direction::Up, h), shape.right, w), state, 18));
            }
        }
        self.asked(super::Asked::Portal { reason: "FIRE", rows })
    }
    /// `PortalShape.createPortalBlocks`: fill the frame (flags 18).
'''),
    (LEVEL + 'portal_travel.rs', '''    /// `PortalForcer.createPortal`: a 2 by 3 portal in an obsidian frame at the
''', '''    /// `PortalForcer.createPortal`: a 2 by 3 portal in an obsidian frame at the
'''),
    (LEVEL + 'portal_travel.rs', '''impl Behaviours {
    /// `PortalForcer.createPortal` on `level` (see `Sim::create_portal`).
''', '''thread_local! {
    /// While set, `create_portal` only says what it would build, in `PORTAL_PLAN`.
    pub static PORTAL_DRY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub static PORTAL_PLAN: std::cell::RefCell<Option<Vec<([i32; 3], BlockStateId)>>> = const { std::cell::RefCell::new(None) };
}

impl Behaviours {
    /// `PortalForcer.createPortal` on `level` (see `Sim::create_portal`).
'''),
    (LEVEL + 'portal_travel.rs', '''        let obsidian = b.default_of(b.named.obsidian);
        let spot = match full.or(partial) {''', '''        let obsidian = b.default_of(b.named.obsidian);
        let mut planned = Vec::new();
        let spot = match full.or(partial) {'''),
    (LEVEL + 'portal_travel.rs', '''                            self.set_block(at, if height < 0 { obsidian } else { air }, 3);
''', '''                            planned.push((at, if height < 0 { obsidian } else { air }, 3));
'''),
    (LEVEL + 'portal_travel.rs', '''                    self.set_block(
                        step(spot, direction, width).offset(0, height, 0),
                        obsidian,
                        3,
                    );
''', '''                    planned.push((step(spot, direction, width).offset(0, height, 0), obsidian, 3));
'''),
    (LEVEL + 'portal_travel.rs', '''                self.set_block(
                    step(spot, direction, width).offset(0, height, 0),
                    portal,
                    18,
                );
            }
        }
        Some(FoundRectangle {''', '''                planned.push((step(spot, direction, width).offset(0, height, 0), portal, 18));
            }
        }
        if PORTAL_DRY.get() {
            PORTAL_PLAN.set(Some(planned.iter().map(|row| ([row.0.x, row.0.y, row.0.z], row.1)).collect()));
            return None;
        }
        for (at, state, flags) in planned {
            self.set_block(at, state, flags);
        }
        Some(FoundRectangle {'''),
    (GAME + 'nether_portal.rs', '''        let created = self
            .with_level(|b, level, ticks| {
                b.create_portal(level, ticks, approximate, x_axis, &border, logical)
            });
''', '''        let acting = self.native_acting_id();
        super::paper_fam_ev_world::portal_traveller_or(acting);
        horizon_world::level::portal_travel::PORTAL_DRY.set(true);
        let _ = self.with_level(|b, level, ticks| b.create_portal(level, ticks, approximate, x_axis, &border, logical));
        horizon_world::level::portal_travel::PORTAL_DRY.set(false);
        if let Some(rows) = horizon_world::level::portal_travel::PORTAL_PLAN.take()
            && !super::paper_fam_ev_world::portal_planned(self, rows)
        {
            super::paper_fam_ev_world::portal_traveller(None);
            return None;
        }
        let created = self
            .with_level(|b, level, ticks| {
                b.create_portal(level, ticks, approximate, x_axis, &border, logical)
            });
        super::paper_fam_ev_world::portal_traveller(None);
'''),
    (GAME + 'server_travel.rs', '''                let exit = trip
                    .plan
                    .as_ref()
                    .and_then(|plan| self.nether_exit_portal(plan));
''', '''                super::paper_fam_ev_world::portal_traveller(Some(host.hub.guests[i].uuid));
                let exit = trip
                    .plan
                    .as_ref()
                    .and_then(|plan| self.nether_exit_portal(plan));
                super::paper_fam_ev_world::portal_traveller(None);
'''),
    # ---- `PortalCreateEvent`: the platform in the End.
    (GAME + 'end.rs', '''            let origin = [END_SPAWN[0], END_SPAWN[1] - 1, END_SPAWN[2]];
            for dz in -2..=2 {
                for dx in -2..=2 {
                    for dy in -1..3 {
                        let p = [origin[0] + dx, origin[1] + dy, origin[2] + dz];
                        let name = if dy == -1 {
                            "minecraft:obsidian"
                        } else {
                            "minecraft:air"
                        };
                        if &*self.block_name(p) != name {
                            self.set_state_named(p, name, &[])?;
                        }
                    }
                }
            }
''', '''            let origin = [END_SPAWN[0], END_SPAWN[1] - 1, END_SPAWN[2]];
            let mut planned = Vec::new();
            for dz in -2..=2 {
                for dx in -2..=2 {
                    for dy in -1..3 {
                        let p = [origin[0] + dx, origin[1] + dy, origin[2] + dz];
                        let name = if dy == -1 {
                            "minecraft:obsidian"
                        } else {
                            "minecraft:air"
                        };
                        if &*self.block_name(p) != name {
                            planned.push((p, name));
                        }
                    }
                }
            }
            if !planned.is_empty() && super::paper_fam_ev_world::end_platform(self, &planned) {
                for (p, name) in planned {
                    self.set_state_named(p, name, &[])?;
                }
            }
'''),
    # ---- `StructuresLocateEvent`: whoever is searching.
    ('horizon-client/src/vanillagen.rs', '''pub struct VanillaTerrain {
''', '''/// `StructuresLocateEvent`: the level, the structures, where from, the radius and whether only unexplored
/// ones will do. An answer ends the search with it.
pub static LOCATE_HOOK: std::sync::OnceLock<fn(&str, &mut Vec<String>, [i32; 3], &mut i32, &mut bool) -> Option<Option<(String, [i32; 3])>>> = std::sync::OnceLock::new();

pub struct VanillaTerrain {
'''),
    ('horizon-client/src/vanillagen.rs', '''        self.generator.worker().locate(names, from, radius)
''', '''        self.locate_for_map(names, from, radius, false)
'''),
    ('horizon-client/src/vanillagen.rs', '''        if !skip_known {
            return self.locate(names, from, radius);
        }
''', '''        let (mut names, mut radius, mut skip_known) = (names.to_vec(), radius, skip_known);
        if let Some(hook) = LOCATE_HOOK.get()
            && let Some(answer) = hook(&self.dimension, &mut names, from, &mut radius, &mut skip_known)
        {
            return answer;
        }
        let names = &names[..];
        if !skip_known {
            return self.generator.worker().locate(names, from, radius);
        }
'''),
    # ---- `NotePlayEvent`: as the note is about to sound.
    (LEVEL + 'mod.rs', '''mod place;
''', '''/// What notes are to sound as, where a listener changed one.
pub(crate) static NOTES_CHANGED: std::sync::Mutex<Vec<(horizon_core::position::BlockPos, crate::registry::BlockStateId)>> = std::sync::Mutex::new(Vec::new());
mod place;
'''),
    (LEVEL + 'redstone.rs', '''        if mob_head || b.is_air(self.above_state(pos)) {
            self.block_event(pos, b.block(state), 0, 0);
            self.game_event(pos, "minecraft:note_block_play");
        }
''', '''        if mob_head || b.is_air(self.above_state(pos)) {
            if self.asked(super::Asked::Note { at: pos, state }) {
                return;
            }
            self.block_event(pos, b.block(state), 0, 0);
            self.game_event(pos, "minecraft:note_block_play");
        }
'''),
    (LEVEL + 'redstone.rs', '''    pub(crate) fn note_trigger_event(&mut self, pos: BlockPos, state: BlockStateId) {
        let b = self.b;
''', '''    pub(crate) fn note_trigger_event(&mut self, pos: BlockPos, state: BlockStateId) {
        let b = self.b;
        let state = {
            let mut changed = super::NOTES_CHANGED.lock().unwrap_or_else(|e| e.into_inner());
            match changed.iter().position(|(at, _)| *at == pos) {
                Some(row) => changed.remove(row).1,
                None => state,
            }
        };
'''),
    # ---- `BlockLockCheckEvent`: what a listener said counts in the game's check.
    (GAME + 'advancements.rs', '''        let Some(lock) = self.world.block_lock(key) else {
            return false;
        };
        let lock = horizon_client::item_render::tag_json(lock);
        let hand = hand.map(item_view);
''', '''        let Some(lock) = self.world.block_lock(key) else {
            return false;
        };
        let (allowed, given) = super::paper_fam_ev_world::lock_said(key);
        if allowed {
            return false;
        }
        let lock = horizon_client::item_render::tag_json(lock);
        let hand = given.as_ref().or(hand).map(item_view);
'''),
    # ---- `HangingPlaceEvent`: the entity is up as it fires.
    (GAME + 'frames.rs', '''        self.item_frames.push(frame);
        if !self.creative {
            self.consume_held();
        }
''', '''        let id = frame.id;
        self.item_frames.push(frame);
        if !super::paper_fam_ev_world::hanging_placed(self, hit.pos, hit.face, stack, id) {
            self.item_frames.retain(|frame| frame.id != id);
            return false;
        }
        if !self.creative {
            self.consume_held();
        }
'''),
    (GAME + 'paintings.rs', '''        self.paintings.push(candidate);
        self.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
        if !self.creative {
            self.consume_held();
        }
''', '''        let id = candidate.id;
        self.paintings.push(candidate);
        if !super::paper_fam_ev_world::hanging_placed(self, hit.pos, hit.face, stack, id) {
            self.paintings.retain(|painting| painting.id != id);
            return false;
        }
        self.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
        if !self.creative {
            self.consume_held();
        }
'''),
    # ---- `StructureGrowEvent`: what a tree or a mushroom wrote.
    (LEVEL + 'growth.rs', '''    /// `TreeGrower.growTree`.
''', '''    /// `place_configured_feature` for a plant growing, with `StructureGrowEvent`'s say: `stood` is what's put
    /// back with the rest if it's cancelled.
    pub(crate) fn grow_feature(&mut self, name: &str, pos: BlockPos, stood: &[(BlockPos, BlockStateId)]) -> bool {
        use crate::worldgen::structure::start::{swap_structure_writes, take_structure_writes};
        if super::ASK_HOOK.get().is_none() {
            return self.place_configured_feature(name, pos);
        }
        let outer = swap_structure_writes(Some(Vec::new()));
        let placed = self.place_configured_feature(name, pos);
        let writes = take_structure_writes();
        // Bone meal is asking too: what it grew is part of what it wrote.
        if let Some(mut outer) = outer {
            outer.extend_from_slice(&writes);
            swap_structure_writes(Some(outer));
        }
        if placed {
            self.asked(super::Asked::Grow { at: pos, feature: name.to_owned(), writes, stood: stood.to_vec() });
        }
        placed
    }
    /// `TreeGrower.growTree`.
'''),
    (LEVEL + 'growth.rs', '''            if self.place_configured_feature(mega, pos.offset(dx, 0, dz)) {
''', '''            let stood: Vec<_> = around.iter().map(|&(s, p)| (p, s)).collect();
            if self.grow_feature(mega, pos.offset(dx, 0, dz), &stood) {
'''),
    (LEVEL + 'growth.rs', '''        if self.place_configured_feature(feature, pos) {
            return true;
        }
        self.set_block(pos, state, flags::UPDATE_NONE);
''', '''        if self.grow_feature(feature, pos, &[(pos, state)]) {
            return true;
        }
        self.set_block(pos, state, flags::UPDATE_NONE);
'''),
    (LEVEL + 'growth.rs', '''        if self.place_configured_feature(&feature, pos) {
            return true;
        }
        self.set_block_and_update(pos, state);
''', '''        if self.grow_feature(&feature, pos, &[(pos, state)]) {
            return true;
        }
        self.set_block_and_update(pos, state);
'''),
    (LEVEL + 'bonemeal.rs', '''                if let Some(feature) = b.data(state).feature.clone() {
                    self.place_configured_feature(&feature, pos);
                }
            }
            Kind::Azalea => {
''', '''                if let Some(feature) = b.data(state).feature.clone() {
                    self.grow_feature(&feature, pos, &[]);
                }
            }
            Kind::Azalea => {
'''),
    # ---- `BlockIgniteEvent` and `BlockBurnEvent`, each where the game lights a fire.
    (LEVEL + 'mod.rs', '''mod place;
''', '''
/// Something the level's own run is about to do that a listener has a say in. It holds what a game would
/// lend, so it stops short of the thing and the game does it after asking.
#[derive(Clone, Debug)]
pub enum Asked {
    Ignite { cause: &'static str, at: BlockPos, source: BlockPos, state: crate::registry::BlockStateId },
    Burn { at: BlockPos, source: BlockPos, was: crate::registry::BlockStateId, to: Option<crate::registry::BlockStateId>, tnt: bool },
    Tnt { cause: &'static str, at: BlockPos, remove: bool },
    Cauldron { at: BlockPos, old: crate::registry::BlockStateId, new: crate::registry::BlockStateId, event: Option<i32> },
    Note { at: BlockPos, state: crate::registry::BlockStateId },
    Bell { at: BlockPos, state: crate::registry::BlockStateId, direction: Direction },
    Portal { reason: &'static str, rows: Vec<(BlockPos, crate::registry::BlockStateId, i32)> },
    Grow { at: BlockPos, feature: String, writes: Vec<crate::worldgen::structure::start::StructureWrite>, stood: Vec<(BlockPos, crate::registry::BlockStateId)> },
    /// A change a block event has a say in, by the event's kind, and what's done if it's let through.
    Change { kind: &'static str, at: BlockPos, source: Option<BlockPos>, state: crate::registry::BlockStateId, doing: Vec<Then> },
    /// `Level.destroyBlock`.
    Destroy { at: BlockPos, state: crate::registry::BlockStateId, drop: bool, limit: i32 },
    /// A sponge about to take this water, and what each place becomes.
    Sponge { at: BlockPos, blocks: Vec<(BlockPos, crate::registry::BlockStateId)> },
    /// A piston about to move, with what's to go with it.
    Piston { kind: &'static str, at: BlockPos, state: crate::registry::BlockStateId, a: i32, param_b: i32, direction: Direction, sticky: bool, blocks: Vec<BlockPos> },
    /// A fluid about to move on to `to`, and what its tick goes on to do, if that's stopped with it.
    Flow { kind: crate::fluid::FluidKind, to: BlockPos, state: crate::registry::BlockStateId, direction: Direction, target: crate::fluid::FluidState, rest: Option<(BlockPos, crate::fluid::FluidState, crate::registry::BlockStateId)> },
}
/// One step of what a let-through change does.
#[derive(Clone, Copy, Debug)]
pub enum Then {
    Set(BlockPos, crate::registry::BlockStateId, i32),
    Remove(BlockPos),
    Drop(crate::registry::BlockStateId, BlockPos),
    Destroy(BlockPos, bool, i32),
    LevelEvent(i32, BlockPos),
    GameEvent(BlockPos, &'static str),
    GameEventOf(BlockPos, &'static str, crate::registry::BlockStateId),
    Neighbour(crate::registry::BlockStateId, BlockPos, states::BlockId),
    NeighbourAt(BlockPos, states::BlockId),
    ToBase(BlockPos, crate::registry::BlockStateId),
    Emit(BlockPos, crate::registry::BlockStateId, &'static str),
    Lever(BlockPos, crate::registry::BlockStateId),
    Press(BlockPos, crate::registry::BlockStateId),
    Release(BlockPos, crate::registry::BlockStateId, bool),
    FluidOn(crate::fluid::FluidKind, BlockPos, crate::registry::BlockStateId, crate::fluid::FluidState, Option<i32>),
}
/// True if a listener wants a say in it: the level leaves it.
pub static ASK_HOOK: std::sync::OnceLock<fn(Dimension, &Asked) -> bool> = std::sync::OnceLock::new();
/// Set once something's been left: whoever's running the level comes out as soon as it can, to answer.
pub static ASK_WAITING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl<L: LevelAccess + ?Sized> Sim<'_, L> {
    /// Whether this is left for the game, after its listeners.
    pub(crate) fn asked(&self, asked: Asked) -> bool {
        self.asking(&asked)
    }
    fn asking(&self, asked: &Asked) -> bool {
        let left = ASK_HOOK.get().is_some_and(|hook| hook(self.level.dimension(), asked));
        if left {
            ASK_WAITING.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        left
    }
    /// What `kind`'s event has a say in: done now, or once it's been let through. Bone meal writes down what
    /// it does for its own event, and doesn't stop for this one.
    pub(crate) fn change(&mut self, kind: &'static str, at: BlockPos, source: Option<BlockPos>, state: crate::registry::BlockStateId, doing: Vec<Then>) {
        let asked = Asked::Change { kind, at, source, state, doing };
        if !crate::worldgen::structure::start::structure_writes_wanted() && self.asking(&asked) {
            return;
        }
        if let Asked::Change { doing, .. } = asked {
            self.run_then(&doing);
        }
    }
    /// `change` for a block set, as most are.
    pub(crate) fn change_to(&mut self, kind: &'static str, at: BlockPos, source: Option<BlockPos>, state: crate::registry::BlockStateId, flags: i32) -> bool {
        self.change(kind, at, source, state, vec![Then::Set(at, state, flags)]);
        true
    }
    pub(crate) fn run_then(&mut self, doing: &[Then]) {
        for then in doing {
            match *then {
                Then::Set(at, state, flags) => {
                    self.set_block(at, state, flags);
                }
                Then::Remove(at) => {
                    self.remove_block(at, false);
                }
                Then::Drop(state, at) => self.drop_resources(state, at),
                Then::Destroy(at, drop, limit) => {
                    self.destroy_block(at, drop, limit);
                }
                Then::LevelEvent(id, at) => self.level_event(id, at, 0),
                Then::GameEvent(at, event) => self.game_event(at, event),
                Then::GameEventOf(at, event, state) => self.game_event_state(at, event, state),
                Then::Neighbour(state, at, block) => self.neighbor_changed_full(state, at, block, false),
                Then::NeighbourAt(at, block) => self.neighbor_changed_at(at, block),
                Then::ToBase(at, state) => self.turn_to_base_block(at, state),
                Then::Emit(pos, state, kind) => self.emit(LevelEvent::BlockEvent { pos, state, kind }),
                Then::Lever(at, state) => self.lever_pulled(at, state),
                Then::Press(at, state) => self.button_pressed(at, state),
                Then::Release(at, state, pressed) => self.button_turned(at, state, pressed),
                Then::FluidOn(kind, at, state, fluid, delay) => {
                    if let Some(delay) = delay {
                        self.schedule_fluid_tick(at, fluid.kind(), delay);
                    }
                    self.spread(kind, at, state, fluid);
                }
            }
        }
    }
}

impl Behaviours {
    /// The note a listener let through, sounding as `sounds` if it changed it.
    pub fn note_sounds<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos, state: crate::registry::BlockStateId, sounds: Option<crate::registry::BlockStateId>) {
        let mut sim = self.sim(level, ticks);
        if let Some(sounds) = sounds {
            NOTES_CHANGED.lock().unwrap_or_else(|e| e.into_inner()).push((pos, sounds));
        }
        sim.block_event(pos, self.block(state), 0, 0);
        sim.game_event(pos, "minecraft:note_block_play");
    }
    /// The ring a listener let through.
    pub fn bell_rings<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos, state: crate::registry::BlockStateId, direction: Direction) {
        self.sim(level, ticks).bell_sounds(pos, state, direction);
    }
    /// What a projectile at `location` makes a target give.
    pub fn target_strength<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos, location: [f64; 3], face: [i32; 3]) -> Option<i32> {
        self.sim(level, ticks).target_strength(pos, location, face)
    }
    /// A target giving what its listeners left.
    pub fn target_powers<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos, strength: i32, arrow: bool) {
        self.sim(level, ticks).target_powers(pos, strength, arrow);
    }
    /// A sponge's taking let through.
    pub fn sponge_absorbs<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos, kept: Option<&[BlockPos]>) {
        self.sim(level, ticks).sponge_absorbs(pos, kept);
    }
    /// A change let through.
    pub fn run_then<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, doing: &[Then]) {
        self.sim(level, ticks).run_then(doing);
    }
    /// A destruction let through, as it was left.
    pub fn destroy_told<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos, state: crate::registry::BlockStateId, drop: bool, effect: Option<crate::registry::BlockStateId>, limit: i32) -> bool {
        self.sim(level, ticks).destroy_told(pos, state, drop, effect, limit)
    }
    /// A fluid's move let through, its drops already seen to if `dropped`.
    #[allow(clippy::too_many_arguments)]
    pub fn fluid_flows<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, kind: crate::fluid::FluidKind, to: BlockPos, state: crate::registry::BlockStateId, direction: Direction, target: crate::fluid::FluidState, rest: Option<(BlockPos, crate::fluid::FluidState, crate::registry::BlockStateId)>, dropped: bool) {
        let mut sim = self.sim(level, ticks);
        sim.flow_on(kind, to, state, direction, target, rest, dropped);
    }
}
mod place;
'''),
    (LEVEL + 'tick.rs', '''                                let next = self.fire_state_with_age(test, spread_age);
                                self.set_block_and_update(test, next);
''', '''                                let next = self.fire_state_with_age(test, spread_age);
                                if !self.asked(super::Asked::Ignite { cause: "SPREAD", at: test, source: pos, state: next }) {
                                    self.set_block_and_update(test, next);
                                }
'''),
    (LEVEL + 'tick.rs', '''            if self.rand_int(age + 10) < 5 && !self.level.is_raining_at(pos) {
                let new_age = (age + self.rand_int(5) / 4).min(15);
                if allowed {
                    let next = self.fire_state_with_age(pos, new_age);
                    self.set_block_and_update(pos, next);
                }
            } else if allowed {
                self.remove_block(pos, false);
            }
            if allowed && b.instance_of(old, Class::Tnt) {
''', '''            let to = if self.rand_int(age + 10) < 5 && !self.level.is_raining_at(pos) {
                let new_age = (age + self.rand_int(5) / 4).min(15);
                Some(self.fire_state_with_age(pos, new_age))
            } else {
                None
            };
            if allowed && self.asked(super::Asked::Burn { at: pos, source: fire, was: old, to, tnt: b.instance_of(old, Class::Tnt) }) {
                return;
            }
            if allowed {
                match to {
                    Some(next) => {
                        self.set_block_and_update(pos, next);
                    }
                    None => {
                        self.remove_block(pos, false);
                    }
                }
            }
            if allowed && b.instance_of(old, Class::Tnt) {
'''),
    (LEVEL + 'fluids.rs', '''                        let fire = self.fire_state_for(test);
                        self.set_block_and_update(test, fire);
                        return;
''', '''                        let fire = self.fire_state_for(test);
                        if !self.asked(super::Asked::Ignite { cause: "LAVA", at: test, source: pos, state: fire }) {
                            self.set_block_and_update(test, fire);
                        }
                        return;
'''),
    (LEVEL + 'fluids.rs', '''                    let fire = self.fire_state_for(test);
                    self.set_block_and_update(above(test), fire);
''', '''                    let fire = self.fire_state_for(test);
                    if !self.asked(super::Asked::Ignite { cause: "LAVA", at: above(test), source: pos, state: fire }) {
                        self.set_block_and_update(above(test), fire);
                    }
'''),
    ('horizon-client/src/game.rs', '''            if self.can_light(clicked, &name) {
                let registry = self.pack.assets.registry;
''', '''            if self.can_light(clicked, &name) {
                if !paper_fam_ev_world::ignited_by_item(self, hit.pos, item == "fire_charge") {
                    if item == "fire_charge" {
                        self.consume_held();
                    } else {
                        self.damage_held(1);
                    }
                    return Ok(UseResult::Pass);
                }
                let registry = self.pack.assets.registry;
'''),
    ('horizon-client/src/game.rs', '''            if self.can_place_used_fire(target) {
                if !self.try_light_portal(target)? {
''', '''            if self.can_place_used_fire(target) {
                if !paper_fam_ev_world::ignited_by_item(self, target, item == "fire_charge") {
                    if item == "fire_charge" {
                        self.consume_held();
                    } else {
                        self.damage_held(1);
                    }
                    return Ok(UseResult::Pass);
                }
                if !self.try_light_portal(target)? {
'''),
    (GAME + 'dispenser.rs', '''            self.level_set([target.0, target.1, target.2], fire)?;
            self.post_block_game_event(
                "minecraft:block_place",
                [target.0, target.1, target.2],
                None,
                None,
            );
''', '''            if super::paper_fam_ev_world::ignited_by_block(self, [target.0, target.1, target.2], "FLINT_AND_STEEL", [pos.0, pos.1, pos.2]) {
                self.level_set([target.0, target.1, target.2], fire)?;
                self.post_block_game_event(
                    "minecraft:block_place",
                    [target.0, target.1, target.2],
                    None,
                    None,
                );
            }
'''),
    (GAME + 'explosion.rs', '''                    self.level_set(pos, fire)?;
                }
            }
        }
        let pitch = (1.0 + (client_float() - client_float()) * 0.2) * 0.7;
''', '''                    if super::paper_fam_ev_world::ignited_by_entity(self, pos, "EXPLOSION", super::paper_fam_ev_entity::exploder_id()) {
                        self.level_set(pos, fire)?;
                    }
                }
            }
        }
        let pitch = (1.0 + (client_float() - client_float()) * 0.2) * 0.7;
'''),
    ('horizon-client/src/game.rs', '''    fn exploder(id: u32, kind: &str) {
''', '''    /// The entity whose blast is under way.
    pub(in crate::game) fn exploder_id() -> Option<u32> {
        EXPLODER.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|(id, _)| *id)
    }
    fn exploder(id: u32, kind: &str) {
'''),
    (GAME + 'lightning.rs', '''    fn bolt_fire(&mut self, at: [f64; 3], extra: i32) -> i32 {
''', '''    fn bolt_fire(&mut self, bolt: u32, at: [f64; 3], extra: i32) -> i32 {
'''),
    (GAME + 'lightning.rs', '''            if let Some(fire) = fire {
                let _ = self.level_set(p, fire);
                set += 1;
            }
''', '''            if let Some(fire) = fire
                && super::paper_fam_ev_world::ignited_by_entity(self, p, "LIGHTNING", Some(bolt))
            {
                let _ = self.level_set(p, fire);
                set += 1;
            }
'''),
    (GAME + 'lightning.rs', '''self.bolt_fire(b.at, 4)''', '''self.bolt_fire(b.id, b.at, 4)'''),
    (GAME + 'lightning.rs', '''self.bolt_fire(b.at, 0)''', '''self.bolt_fire(b.id, b.at, 0)'''),
    (GAME + 'creatures.rs', '''                            self.level_set(fire, state)?;
                        }
                    }
                    Hit::None => {}
''', '''                            if super::paper_fam_ev_world::ignited_by_entity(self, fire, "FIREBALL", Some(p.id)) {
                                self.level_set(fire, state)?;
                            }
                        }
                    }
                    Hit::None => {}
'''),
    (GAME + 'creatures.rs', '''                    BlockReaction::Light(lit) => {
                        let _ = self.level_set([pos.x, pos.y, pos.z], lit);
''', '''                    BlockReaction::Light(lit) => {
                        let arrow = super::paper_fam_ev_world::arrow_at(self, [at.x, at.y, at.z]);
                        if !super::paper_fam_ev_world::ignited_by_entity(self, [pos.x, pos.y, pos.z], "ARROW", arrow) {
                            return;
                        }
                        let _ = self.level_set([pos.x, pos.y, pos.z], lit);
'''),
    (GAME + 'dragon.rs', '''                && self.world.loaded(b[0], b[2])
            {
                let _ = self.level_set(b, fire);
''', '''                && self.world.loaded(b[0], b[2])
                && super::paper_fam_ev_world::ignited_by_entity(self, b, "ENDER_CRYSTAL", Some(self.crystals[i].id))
            {
                let _ = self.level_set(b, fire);
'''),
    # ---- `CauldronLevelChangeEvent`: before each change.
    ('horizon-client/src/game.rs', '''        let result = cauldron::interact(
            &state,
            under_water,
            removes_dye,
            &mut Hand(self),
        );
''', '''        let told = paper_fam_ev_world::cauldron_asked(self, pos, &state, under_water, removes_dye);
        if told == Some(None) {
            return Ok(true);
        }
        let result = cauldron::interact(
            &state,
            under_water,
            removes_dye,
            &mut Hand(self),
        );
'''),
    ('horizon-client/src/game.rs', '''        if let Some(block) = result.block && let Ok(new) = registry.resolve(&block) {
            self.level_set(pos, new)?;
        }
''', '''        if let Some(Some(new)) = told {
            self.level_set(pos, new)?;
        } else if let Some(block) = result.block && let Ok(new) = registry.resolve(&block) {
            self.level_set(pos, new)?;
        }
'''),
    ('horizon-client/src/game.rs', '''        let Ok(new) = registry.resolve(&horizon_client::cauldron::extinguished(&old))
        else {
            return;
        };
''', '''        let Ok(new) = registry.resolve(&horizon_client::cauldron::extinguished(&old))
        else {
            return;
        };
        let Some(new) = paper_fam_ev_world::cauldron_extinguishes(self, at, &old, new) else {
            return;
        };
'''),
    (LEVEL + 'speleothem.rs', '''            self.set_block_and_update(pos, b.default_of(next));
            self.game_event(pos, "minecraft:block_change");
            self.level_event(event, pos, 0);
''', '''            let next = b.default_of(next);
            if self.asked(super::Asked::Cauldron { at: pos, old: state, new: next, event: Some(event) }) {
                return;
            }
            self.set_block_and_update(pos, next);
            self.game_event(pos, "minecraft:block_change");
            self.level_event(event, pos, 0);
'''),
    (LEVEL + 'speleothem.rs', '''                self.set_block_and_update(pos, b.table.set_int(state, b.props.level, level + 1));
                self.game_event(pos, "minecraft:block_change");
                self.level_event(1047, pos, 0);
''', '''                let next = b.table.set_int(state, b.props.level, level + 1);
                if self.asked(super::Asked::Cauldron { at: pos, old: state, new: next, event: Some(1047) }) {
                    return;
                }
                self.set_block_and_update(pos, next);
                self.game_event(pos, "minecraft:block_change");
                self.level_event(1047, pos, 0);
'''),
    (LEVEL + 'tick.rs', '''                    self.set_block_and_update(below_pos, b.default_of(next));
                    self.game_event(below_pos, "minecraft:block_change");
''', '''                    let next = b.default_of(next);
                    if !self.asked(super::Asked::Cauldron { at: below_pos, old: below_state, new: next, event: None }) {
                        self.set_block_and_update(below_pos, next);
                        self.game_event(below_pos, "minecraft:block_change");
                    }
'''),
    (LEVEL + 'tick.rs', '''                    self.set_block_and_update(below_pos, b.table.cycle(below_state, p.level));
                    self.game_event(below_pos, "minecraft:block_change");
''', '''                    let next = b.table.cycle(below_state, p.level);
                    if !self.asked(super::Asked::Cauldron { at: below_pos, old: below_state, new: next, event: None }) {
                        self.set_block_and_update(below_pos, next);
                        self.game_event(below_pos, "minecraft:block_change");
                    }
'''),
    # ---- `BellRingEvent`: whatever rings it.
    (LEVEL + 'redstone.rs', '''    pub(crate) fn bell_ring(&mut self, pos: BlockPos, state: BlockStateId, direction: Direction) {
''', '''    pub(crate) fn bell_ring(&mut self, pos: BlockPos, state: BlockStateId, direction: Direction) {
        if !self.asked(super::Asked::Bell { at: pos, state, direction }) {
            self.bell_sounds(pos, state, direction);
        }
    }
    pub(crate) fn bell_sounds(&mut self, pos: BlockPos, state: BlockStateId, direction: Direction) {
'''),
    ('horizon-client/src/mobs.rs', '''    RingBell { bell: [i32; 3] },
''', '''    RingBell { mob: u32, bell: [i32; 3] },
'''),
    ('horizon-client/src/mobs/villager_brain.rs', '''cx.events.push(MobEvent::RingBell { bell });''', '''cx.events.push(MobEvent::RingBell { mob: m.id, bell });'''),
    (GAME + 'creatures.rs', '''                MobEvent::RingBell { bell } => {
                    let pos = BlockPos::new(bell[0], bell[1], bell[2]);
''', '''                MobEvent::RingBell { mob, bell } => {
                    super::paper_fam_ev_world::bell_by_mob(mob);
                    let pos = BlockPos::new(bell[0], bell[1], bell[2]);
'''),
    # ---- `ServerCommandEvent`: a command block's line.
    (GAME + 'command_blocks.rs', '''        let line = command.strip_prefix('/').unwrap_or(command).to_owned();
        let outer = self.cmds.results.replace(Vec::new());
''', '''        let line = command.strip_prefix('/').unwrap_or(command).to_owned();
        let Some(line) = super::paper_fam_ev_world::server_command(self, line, source) else {
            return 0;
        };
        let outer = self.cmds.results.replace(Vec::new());
'''),
    # ---- `BlockCookEvent`: a furnace's.
    (GAME + 'block_entity_ticks.rs', '''        let Some(BlockEntity::Furnace(f)) = self.world.block_entities.get_mut(&pos) else {
            return Ok(());
        };
        let Some(lit) = f
            .tick(world_recipes!(self), world_items!(self), self.res.providers()) else {
            return Ok(());
        };
''', '''        let smelt = super::paper_fam_ev_world::furnace_smelt(self, pos);
        let Some(BlockEntity::Furnace(f)) = self.world.block_entities.get_mut(&pos) else {
            return Ok(());
        };
        let lit = f.tick(world_recipes!(self), world_items!(self), self.res.providers());
        super::paper_fam_ev_world::furnace_smelted(self, pos, smelt);
        let Some(lit) = lit else {
            return Ok(());
        };
'''),
    # ---- `BrewingStartEvent`: the tick it starts.
    (GAME + 'stations.rs', '''        let out = b
            .tick(
                &world_stations!(self).brewing,
                world_items!(self),
                self.res.providers(),
            );
''', '''        let idle = b.brew_time == 0;
        let out = b
            .tick(
                &world_stations!(self).brewing,
                world_items!(self),
                self.res.providers(),
            );
        let started = idle && b.brew_time > 0;
'''),
    (GAME + 'stations.rs', '''        Ok(())
    }
}
/// The anvil's rename box''', '''        if started {
            super::paper_fam_ev_world::brewing_started(self, pos);
        }
        Ok(())
    }
}
/// The anvil's rename box'''),
    # ---- `BeaconActivatedEvent` and `BeaconDeactivatedEvent`: as the sounds play, once it's back in the level.
    (GAME + 'beacon.rs', '''                    self.beacon_sound([x, y, z], "minecraft:block.beacon.activate");
''', '''                    self.beacon_sound([x, y, z], "minecraft:block.beacon.activate");
                    super::paper_fam_ev_world::beacon_turned(true);
'''),
    (GAME + 'beacon.rs', '''                self.beacon_sound([x, y, z], "minecraft:block.beacon.deactivate");
''', '''                self.beacon_sound([x, y, z], "minecraft:block.beacon.deactivate");
                super::paper_fam_ev_world::beacon_turned(false);
'''),
    (GAME + 'beacon.rs', '''        self.tick_beacon(key, &mut b, false);
        self.world.block_entities.put(key, BlockEntity::Beacon(b));
''', '''        self.tick_beacon(key, &mut b, false);
        self.world.block_entities.put(key, BlockEntity::Beacon(b));
        super::paper_fam_ev_world::beacon_ticked(self, key);
'''),
    # ---- `BlockDispenseLootEvent` and `VaultDisplayItemEvent`.
    ('horizon-client-render/src/vault.rs', '''        let items = roll(&self.config.loot_table.clone(), random, true);
        if items.is_empty() {
            return 0;
        }
        // unlock
''', '''        let given = VAULT_GIVEN.take();
        let granted = given.is_some();
        let items = match given {
            Some(items) => items,
            None => roll(&self.config.loot_table.clone(), random, true),
        };
        if items.is_empty() && !granted {
            return 0;
        }
        if VAULT_ASKING.get() {
            VAULT_ROLLED.set(Some(items));
            return 0;
        }
        // unlock
'''),
    ('horizon-client-render/src/vault.rs', '''#[derive(Clone, Debug, PartialEq)]
pub enum VaultEvent {
''', '''thread_local! {
    /// While set, a key's roll stops at `VAULT_ROLLED`, for listeners to see before anything is taken.
    pub static VAULT_ASKING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub static VAULT_ROLLED: std::cell::RefCell<Option<Vec<ItemStack>>> = const { std::cell::RefCell::new(None) };
    /// What the listeners left, in the place of a roll.
    pub static VAULT_GIVEN: std::cell::RefCell<Option<Vec<ItemStack>>> = const { std::cell::RefCell::new(None) };
}

#[derive(Clone, Debug, PartialEq)]
pub enum VaultEvent {
'''),
    (GAME + 'vault.rs', '''        let player = self.acting_uuid();
        let used = self
''', '''        let player = self.acting_uuid();
        if super::paper_fam_ev_world::vault_listened() {
            horizon_client::vault::VAULT_ASKING.set(true);
            let asked = self
                .with_vault(
                    (pos[0], pos[1], pos[2]),
                    |v, l, random, state, ominous, roll, events| {
                        v.use_item_on(l, random, pos, state, ominous, player, Some(&stack), roll, events)
                    },
                );
            horizon_client::vault::VAULT_ASKING.set(false);
            asked.transpose()?;
            if let Some(rolled) = horizon_client::vault::VAULT_ROLLED.take() {
                let Some(items) = super::paper_fam_ev_world::vault_loot(self, pos, rolled) else {
                    return Ok(true);
                };
                horizon_client::vault::VAULT_GIVEN.set(Some(items));
            }
        }
        let used = self
'''),
    (GAME + 'vault.rs', '''        let pos = [key.0, key.1, key.2];
        let ticked = self
''', '''        let pos = [key.0, key.1, key.2];
        let shown = super::paper_fam_ev_world::vault_shown(self, key);
        let ticked = self
'''),
    (GAME + 'vault.rs', '''        if let Some(result) = ticked {
            result?;
        }
''', '''        if let Some(result) = ticked {
            result?;
        }
        super::paper_fam_ev_world::vault_display(self, key, shown);
'''),
    (GAME + 'trial_spawner.rs', '''            // TrialSpawner.ejectReward: the table's items, each thrown up out
''', '''            let reward_table = &mut reward_table;
            // TrialSpawner.ejectReward: the table's items, each thrown up out
'''),
    (GAME + 'trial_spawner.rs', '''            let mut eject = |table: &str, random: &mut LegacyRandom| {
''', '''            let mut eject = |table: &str, random: &mut LegacyRandom| {
                *reward_table = Some(table.to_owned());
'''),
    (GAME + 'trial_spawner.rs', '''        let mut ejected: Vec<(ItemStack, [f64; 3])> = Vec::new();
''', '''        let mut ejected: Vec<(ItemStack, [f64; 3])> = Vec::new();
        let mut reward_table: Option<String> = None;
'''),
    (GAME + 'trial_spawner.rs', '''        for (stack, v) in ejected {
            self.spawn_item(stack, above, v, 0);
        }
''', '''        for (stack, v) in super::paper_fam_ev_world::trial_loot(self, pos, reward_table, ejected) {
            self.spawn_item(stack, above, v, 0);
        }
'''),
    # ---- `RaidStopEvent`, with why, and `RaidSpawnWaveEvent` with its raiders.
    ('horizon-client/src/raid.rs', '''pub enum Action {
''', '''pub enum Action {
    /// The raid stopped by itself, and why: `RaidStopEvent.Reason`.
    Stopped {
        reason: &'static str,
    },
'''),
    ('horizon-client/src/raid.rs', '''            if level.difficulty() == 0 {
                self.stop();
''', '''            if level.difficulty() == 0 {
                actions.push(Action::Stopped { reason: "PEACE" });
                self.stop();
'''),
    ('horizon-client/src/raid.rs', '''                } else {
                    self.stop();
                }
            }
            self.ticks_active += 1;
''', '''                } else {
                    actions.push(Action::Stopped { reason: "NOT_IN_VILLAGE" });
                    self.stop();
                }
            }
            self.ticks_active += 1;
'''),
    ('horizon-client/src/raid.rs', '''            if self.ticks_active >= TIMEOUT {
                self.stop();
''', '''            if self.ticks_active >= TIMEOUT {
                actions.push(Action::Stopped { reason: "TIMEOUT" });
                self.stop();
'''),
    ('horizon-client/src/raid.rs', '''                if attempt > 5 {
                    self.stop();
''', '''                if attempt > 5 {
                    actions.push(Action::Stopped { reason: "UNSPAWNABLE" });
                    self.stop();
'''),
    ('horizon-client/src/raid.rs', '''            if self.celebration_ticks >= CELEBRATION_TICKS {
                self.stop();
''', '''            if self.celebration_ticks >= CELEBRATION_TICKS {
                actions.push(Action::Stopped { reason: "FINISHED" });
                self.stop();
'''),
    (GAME + 'raids.rs', '''            Action::Horn { pos, .. } => self.raid_horn(id, pos),
''', '''            Action::Stopped { reason } => super::paper_fam_ev_world::raid_stopped(self, id, reason),
            Action::Horn { pos, .. } => self.raid_horn(id, pos),
'''),
    (GAME + 'raids.rs', '''                self.mount_mob(rider, id);
            }
        }
    }
    /// `Raid.joinRaid` for a new raider: placed at the block's centre one
''', '''                self.mount_mob(rider, id);
            }
        }
        super::paper_fam_ev_world::raid_wave_done(self, raid_id, pos, wave);
    }
    /// `Raid.joinRaid` for a new raider: placed at the block's centre one
'''),
    # ---- `AsyncServerDataFixerRemoveBlockEntityEvent`: a bed's old block entity.
    ('horizon-world/src/datafix/fixes_c.rs', '''        if let Some(Tag::List(bes)) = c.get_mut("block_entities") {
            bes.retain(|be| !is_bed(be));
        }
''', '''        let dimension = c.get("__context").and_then(|ctx| as_str(ctx.get("dimension"))).unwrap_or_default();
        if let Some(Tag::List(bes)) = c.get_mut("block_entities") {
            bes.retain(|be| {
                if !is_bed(be) {
                    return true;
                }
                if let Some(hook) = super::BLOCK_ENTITY_DROPPED.get() {
                    let at = |key: &str| match be.get(key) {
                        Some(Tag::Int(value)) => *value,
                        _ => 0,
                    };
                    hook(&dimension, "minecraft:bed", [at("x"), at("y"), at("z")], be.get("PublicBukkitValues"));
                }
                false
            });
        }
'''),
    ('horizon-world/src/datafix/mod.rs', '''pub fn update_chunk(tag: &mut Tag, dimension: &str, generator: Option<&str>) {
''', '''/// A block entity leaving a chunk because its type is no more: the level, the type, where and its plugin data.
pub static BLOCK_ENTITY_DROPPED: std::sync::OnceLock<fn(&str, &str, [i32; 3], Option<&Tag>)> = std::sync::OnceLock::new();

pub fn update_chunk(tag: &mut Tag, dimension: &str, generator: Option<&str>) {
'''),
    # ---- `VehicleEnterEvent` and `VehicleExitEvent`, for mobs and with the game's own checks passed.
    (GAME + 'boats.rs', '''        if super::paper_fam_ev_entity::mount_refused(self, self.boats[i].id) {
            return false;
        }
''', '''        if !super::paper_fam_ev_world::vehicle_enters(self, self.boats[i].id, None) {
            return false;
        }
        if super::paper_fam_ev_entity::mount_refused(self, self.boats[i].id) {
            return false;
        }
'''),
    (GAME + 'minecarts.rs', '''                if super::paper_fam_ev_entity::mount_refused(self, id) {
                    return false;
                }
''', '''                if !super::paper_fam_ev_world::vehicle_enters(self, id, None) {
                    return false;
                }
                if super::paper_fam_ev_entity::mount_refused(self, id) {
                    return false;
                }
'''),
    (GAME + 'boat_passengers.rs', '''                        let m = &mut self.mobs[j];
                        if horizon_client::mobs::ee::mob_boards(''', '''                        let mob = self.mobs[j].id;
                        if !super::paper_fam_ev_world::vehicle_enters(self, id, Some(mob)) {
                            continue;
                        }
                        let m = &mut self.mobs[j];
                        if horizon_client::mobs::ee::mob_boards('''),
    (GAME + 'minecarts.rs', '''                    let mob = self.mobs[j].id;
                    if horizon_client::mobs::ee::mob_boards(''', '''                    let mob = self.mobs[j].id;
                    if !super::paper_fam_ev_world::vehicle_enters(self, id, Some(mob)) {
                        continue;
                    }
                    if horizon_client::mobs::ee::mob_boards('''),
    (GAME + 'boat_passengers.rs', '''            if horizon_client::mobs::ee::mob_off(&self.mobs[j], id, broken) {
                continue;
            }
''', '''            let mob = self.mobs[j].id;
            if !super::paper_fam_ev_world::mob_exits(self, id, mob, !broken) {
                continue;
            }
            if horizon_client::mobs::ee::mob_off(&self.mobs[j], id, broken) {
                continue;
            }
'''),
    (GAME + 'minecarts.rs', '''                let m = &self.mobs[j];
                Vec3::new(m.position.x, m.position.y.max(y), m.position.z)
            };
            riders::stop_riding(&mut self.mobs[j], Some(at));
''', '''                let m = &self.mobs[j];
                Vec3::new(m.position.x, m.position.y.max(y), m.position.z)
            };
            let mob = self.mobs[j].id;
            super::paper_fam_ev_world::mob_exits(self, id, mob, false);
            riders::stop_riding(&mut self.mobs[j], Some(at));
'''),
    # ---- `TNTPrimeEvent`: before the fuse, for each cause.
    (LEVEL + 'hooks.rs', '''        if !creative && data.is(Class::Tnt) && b.table.bool(state, p.unstable) {
            self.prime_tnt(pos);
        }
''', '''        if !creative && data.is(Class::Tnt) && b.table.bool(state, p.unstable)
            && !self.asked(super::Asked::Tnt { cause: "BLOCK_BREAK", at: pos, remove: false })
        {
            self.prime_tnt(pos);
        }
'''),
    (LEVEL + 'redstone.rs', '''        if !self.level.is_client_side() && self.has_neighbor_signal(pos) {
            self.prime_tnt(pos);
            self.remove_block(pos, false);
        }
''', '''        if !self.level.is_client_side()
            && self.has_neighbor_signal(pos)
            && !self.asked(super::Asked::Tnt { cause: "REDSTONE", at: pos, remove: true })
        {
            self.prime_tnt(pos);
            self.remove_block(pos, false);
        }
'''),
    (GAME + 'tnt.rs', '''        let pos = BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]);
        self.spawn_primed_tnt(pos, false);
''', '''        let pos = BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]);
        if !super::paper_fam_ev_world::tnt_primes(self, hit.pos, "PLAYER", true, None, None) {
            return Ok(false);
        }
        self.spawn_primed_tnt(pos, false);
'''),
    (GAME + 'creatures.rs', '''                    BlockReaction::PrimeTnt => {
                        self.spawn_primed_tnt(pos, false);
''', '''                    BlockReaction::PrimeTnt => {
                        let arrow = super::paper_fam_ev_world::arrow_at(self, [at.x, at.y, at.z]);
                        if !super::paper_fam_ev_world::tnt_primes(self, [pos.x, pos.y, pos.z], "PROJECTILE", false, arrow, None) {
                            return;
                        }
                        self.spawn_primed_tnt(pos, false);
'''),
    (GAME + 'dispenser.rs', '''            if self.ui.rules.bool("tnt_explodes") {
                self.spawn_primed_tnt(target_pos, false);
''', '''            if !self.ui.rules.bool("tnt_explodes") {
                success = false;
            } else if super::paper_fam_ev_world::tnt_primes(self, [target.0, target.1, target.2], "DISPENSER", false, None, Some([pos.0, pos.1, pos.2])) {
                self.spawn_primed_tnt(target_pos, false);
'''),
    (GAME + 'dispenser.rs', '''                self.process_level_events()?;
            } else {
                success = false;
            }
        }
        let event = if success { CLICK } else { FAIL };
''', '''                self.process_level_events()?;
            }
        }
        let event = if success { CLICK } else { FAIL };
'''),
    (GAME + 'explosion.rs', '''        if is_tnt && self.ui.rules.bool("tnt_explodes") {
''', '''        if is_tnt
            && self.ui.rules.bool("tnt_explodes")
            && super::paper_fam_ev_world::tnt_primes(self, pos, "EXPLOSION", false, super::paper_fam_ev_entity::exploder_id(), None)
        {
'''),
    # ---- `BlockExplodeEvent`: the yield and the block.
    (GAME + 'respawn_anchor.rs', '''    fn explode_anchor(&mut self, pos: [i32; 3], _state: BlockStateId) -> Result<()> {
''', '''    fn explode_anchor(&mut self, pos: [i32; 3], _state: BlockStateId) -> Result<()> {
        super::paper_fam_ev_world::block_blows(self, pos);
'''),
    (GAME + 'respawn_anchor.rs', '''        self.level_set(head, self.world.air)?;
        let foot = [head[0] + back[0], head[1], head[2] + back[2]];
''', '''        super::paper_fam_ev_world::block_blows(self, head);
        self.level_set(head, self.world.air)?;
        let foot = [head[0] + back[0], head[1], head[2] + back[2]];
'''),
    (GAME + 'explosion.rs', '''            let decay = (interaction == BlockInteraction::DestroyWithDecay)
                .then_some(radius);
''', '''            let decay = super::paper_fam_ev_world::blast_decay(interaction == BlockInteraction::DestroyWithDecay, radius);
'''),
    # ---- `SpawnChangeEvent`: once the spawn has moved.
    (GAME + 'commands.rs', '''                    f64::from(pos[2]) + 0.5,
                ),
            );
        }
        let args = [
            Text::literal(pos[0].to_string()),
            Text::literal(pos[1].to_string()),
            Text::literal(pos[2].to_string()),
            Text::literal(java_float(f64::from(rot[0]))),
''', '''                    f64::from(pos[2]) + 0.5,
                ),
            );
        }
        super::paper_fam_ev_world::spawn_changed(self, old);
        let args = [
            Text::literal(pos[0].to_string()),
            Text::literal(pos[1].to_string()),
            Text::literal(pos[2].to_string()),
            Text::literal(java_float(f64::from(rot[0]))),
'''),
    # ---- What the level writes, for whoever's keeping it.
    (LEVEL + 'update.rs', '''    pub fn set_block(&mut self, pos: BlockPos, state: BlockStateId, flags: i32) -> bool {
        self.set_block_limit(pos, state, flags, flags::UPDATE_LIMIT)
''', '''    pub fn set_block(&mut self, pos: BlockPos, state: BlockStateId, flags: i32) -> bool {
        if crate::worldgen::structure::start::structure_writes_wanted() {
            crate::worldgen::structure::start::note_structure_write([pos.x, pos.y, pos.z], self.state(pos), state);
        }
        self.set_block_limit(pos, state, flags, flags::UPDATE_LIMIT)
'''),
    # ---- `VehicleCreateEvent`: each place a vehicle comes in.
    (GAME + 'boats.rs', '''        self.boats.push(boat);
        if !self.creative {
            self.consume_held();
        }
''', '''        let id = boat.id;
        self.boats.push(boat);
        if !super::paper_fam_ev_world::vehicle_created(self, id) {
            self.boats.retain(|boat| boat.id != id);
            return false;
        }
        if !self.creative {
            self.consume_held();
        }
'''),
    (GAME + 'minecarts.rs', '''        self.carts.push(cart);
        if !self.creative {
            self.consume_held();
        }
''', '''        let id = cart.id;
        self.carts.push(cart);
        if !super::paper_fam_ev_world::vehicle_created(self, id) {
            self.carts.retain(|cart| cart.id != id);
            return false;
        }
        if !self.creative {
            self.consume_held();
        }
'''),
    (GAME + 'dispenser.rs', '''        self.boats.push(boat);
        (shrink(stack), CLICK)
''', '''        let id = boat.id;
        self.boats.push(boat);
        if !super::paper_fam_ev_world::vehicle_created(self, id) {
            self.boats.retain(|boat| boat.id != id);
        }
        (shrink(stack), CLICK)
'''),
    (GAME + 'dispenser.rs', '''        self.carts.push(cart);
        (shrink(stack), CLICK)
''', '''        let id = cart.id;
        self.carts.push(cart);
        if !super::paper_fam_ev_world::vehicle_created(self, id) {
            self.carts.retain(|cart| cart.id != id);
        }
        (shrink(stack), CLICK)
'''),
    (GAME + 'commands.rs', '''                    self.carts.push(cart);
                    name = Text::translate(&format!("entity.minecraft.{k}"), &[]);
''', '''                    let id = cart.id;
                    self.carts.push(cart);
                    if !super::paper_fam_ev_world::vehicle_created(self, id) {
                        self.carts.retain(|cart| cart.id != id);
                    }
                    name = Text::translate(&format!("entity.minecraft.{k}"), &[]);
'''),
    # ---- `BlockCanBuildEvent`: the game's own answer, for a listener to change.
    (LEVEL + 'mod.rs', '''pub use place::{PlaceContext, PlaceOutcome, PlacementContext};
''', '''pub use place::{PLACE_PROBE, PLACE_SAID, PlaceContext, PlaceOutcome, PlacementContext};
'''),
    (LEVEL + 'place.rs', '''/// A player's use-item-on-block action (`UseOnContext` inputs).
''', '''thread_local! {
    /// While set, `placement` keeps a state that doesn't fit and says so here.
    pub static PLACE_PROBE: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    /// What a listener said of the next placing.
    pub static PLACE_SAID: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// A placing's answer, through the probe and whatever a listener said.
fn may_place(fits: bool) -> bool {
    if PLACE_PROBE.get().is_some() {
        PLACE_PROBE.set(Some(fits));
        return true;
    }
    PLACE_SAID.take().unwrap_or(fits)
}

/// A player's use-item-on-block action (`UseOnContext` inputs).
'''),
    (LEVEL + 'place.rs', '''                self.state_for_placement(binding.block, &ctx)
                    .filter(|state| {
                        (!must_survive || self.can_survive(*state, ctx.pos))
                            && self.level.is_unobstructed(ctx.pos, *state)
                    })
            }
        };
''', '''                self.state_for_placement(binding.block, &ctx)
                    .filter(|state| {
                        may_place(
                            (!must_survive || self.can_survive(*state, ctx.pos))
                                && self.level.is_unobstructed(ctx.pos, *state),
                        )
                    })
            }
        };
'''),
    (LEVEL + 'place.rs', '''                chosen.filter(|state| self.level.is_unobstructed(ctx.pos, *state))
            }
''', '''                chosen.filter(|state| may_place(self.level.is_unobstructed(ctx.pos, *state)))
            }
'''),
    ('horizon-client/src/game.rs', '''        let outcome = self
            .with_level(|b, level, ticks| b.place_block(level, ticks, &ctx));
        let PlaceOutcome::Placed { pos, state } = outcome else {
''', '''        {
            use horizon_world::level::{PLACE_PROBE, PLACE_SAID};
            PLACE_PROBE.set(Some(false));
            let probed = self.with_level(|b, level, ticks| b.placement_state(level, ticks, &ctx));
            let fits = PLACE_PROBE.take().unwrap_or(false);
            if let Some((_, state)) = probed {
                let said = paper_fam_ev_world::can_build(self, hit.pos, state, fits);
                PLACE_SAID.set(Some(said));
            }
        }
        let outcome = self
            .with_level(|b, level, ticks| b.place_block(level, ticks, &ctx));
        let PlaceOutcome::Placed { pos, state } = outcome else {
'''),
    # ---- What the level left for its listeners, as soon as there's a game to lend.
    ('horizon-client/src/game.rs', '''    fn process_level_events(&mut self) -> Result<()> {
        let events = std::mem::take(&mut self.level_events);
''', '''    fn process_level_events(&mut self) -> Result<()> {
        paper_fam_ev_world::run_asked(self);
        let events = std::mem::take(&mut self.level_events);
'''),
    # ---- `/item` reaches every container's slots, a furnace's too.
    (GAME + 'commands/items.rs', '''                let size = match self.world.block_entities.get(&(p[0], p[1], p[2])) {
                    Some(horizon_client::containers::BlockEntity::Chest(items)) => items.len(),
                    _ => 0,
                };
''', '''                let size = self.world.block_entities.get(&(p[0], p[1], p[2])).map_or(0, |entity| entity.slots().len());
'''),
    (GAME + 'commands/items.rs', '''            Slot::Block(p, i) => match self.world.block_entities.get(&(p[0], p[1], p[2])) {
                Some(horizon_client::containers::BlockEntity::Chest(items)) => {
                    items.get(*i).cloned().flatten()
                }
                _ => None,
            },
''', '''            Slot::Block(p, i) => self.world.block_entities.get(&(p[0], p[1], p[2])).and_then(|entity| entity.slots().get(*i).cloned().flatten()),
'''),
    (GAME + 'commands/items.rs', '''            Slot::Block(p, i) => match self.world.block_entities.get_mut(&(p[0], p[1], p[2])) {
                Some(horizon_client::containers::BlockEntity::Chest(items)) => {
                    match items.get_mut(*i) {
                        Some(slot) => {
                            *slot = stack;
                            true
                        }
                        None => false,
                    }
                }
                _ => false,
            },
''', '''            Slot::Block(p, i) => match self.world.block_entities.get_mut(&(p[0], p[1], p[2])).and_then(|entity| entity.slots_mut().get_mut(*i)) {
                Some(slot) => {
                    *slot = stack;
                    true
                }
                None => false,
            },
'''),
    # ---- `BlockDropItemEvent`: a creative break is told too, with nothing in it.
    ('horizon-client/src/game.rs', '''        if removed && !creative {
            let held = self.held().cloned();
            self.mine_block_with_held(&facts.name, facts.hardness);
''', '''        if removed && creative {
            paper_fam_ev_world::creative_drops(self, pos, state);
        }
        if removed && !creative {
            let held = self.held().cloned();
            self.mine_block_with_held(&facts.name, facts.hardness);
'''),
    # ---- `PrepareItemCraftEvent`: `/clear` touches each player's grid.
    (GAME + 'commands.rs', '''                .for_player(t, |g| g.clear_matching(&predicate, max))
                .unwrap_or(0);
            removed += n;
''', '''                .for_player(t, |g| g.clear_matching(&predicate, max))
                .unwrap_or(0);
            removed += n;
            let conn = match t {
                Target::Guest(conn) => Some(conn),
                Target::Player => self.lan_acting,
                _ => None,
            };
            if let Some(conn) = conn {
                super::paper_fam_ev_player::grid_touched(self, conn);
            }
'''),
    # ---- The level's run stops for what it's left, as soon as it can.
    ('horizon-client/src/game.rs', '''                            if !more || !level.events.is_empty()
                                || !level.spill.is_empty()
                            {
                                break more;
                            }
''', '''                            if !more || !level.events.is_empty()
                                || !level.spill.is_empty()
                                || horizon_world::level::ASK_WAITING.load(std::sync::atomic::Ordering::Relaxed)
                            {
                                break more;
                            }
'''),
    ('horizon-client/src/game.rs', '''                if !self.level_events.is_empty() {
                    self.process_level_events()?;
                }
                if !more {
                    break;
                }
''', '''                paper_fam_ev_world::run_asked(self);
                if !self.level_events.is_empty() {
                    self.process_level_events()?;
                }
                if !more {
                    break;
                }
'''),
    ('horizon-client/src/game.rs', '''        self.with_level(|b, level, ticks| {
            let mut sections = Vec::new();
            horizon_client::spans::time(
''', '''        let mut ticking = ticking.into_iter().filter(|_| !debug);
        loop {
        let mut asked = false;
        self.with_level(|b, level, ticks| {
            let mut sections = Vec::new();
            horizon_client::spans::time(
'''),
    ('horizon-client/src/game.rs', '''                    for (x, z) in ticking.into_iter().filter(|_| !debug) {
''', '''                    for (x, z) in ticking.by_ref() {
'''),
    ('horizon-client/src/game.rs', '''                                    .map(|(y, mask)| (*y, Some(mask.clone())))
                            },
                        );
                    }
                },
            );
            b.run_block_events(level, ticks);
            b.tick_block_entities(level, ticks);
        });
''', '''                                    .map(|(y, mask)| (*y, Some(mask.clone())))
                            },
                        );
                        if horizon_world::level::ASK_WAITING.load(std::sync::atomic::Ordering::Relaxed) {
                            asked = true;
                            break;
                        }
                    }
                },
            );
        });
        if !asked {
            break;
        }
        paper_fam_ev_world::run_asked(self);
        }
        loop {
            self.with_level(|b, level, ticks| b.run_block_events(level, ticks));
            if !horizon_world::level::ASK_WAITING.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            paper_fam_ev_world::run_asked(self);
        }
        self.with_level(|b, level, ticks| b.tick_block_entities(level, ticks));
'''),
    # ---- What water breaks, and what a level's event is left to say.
    (LEVEL + 'tick.rs', '                        if let Some(dead) = b.data(state).link {\n                            self.set_block(pos, b.default_of(dead), 2);\n                        }\n', '                        if let Some(dead) = b.data(state).link {\n                            self.change_to("fade", pos, None, b.default_of(dead), 2);\n                        }\n'),
    (LEVEL + 'tick.rs', '                            self.set_block(pos, next, 2);\n                        }\n                    }\n                }\n                Class::Chest | Class::EnderChest => {\n', '                            self.change_to("fade", pos, None, next, 2);\n                        }\n                    }\n                }\n                Class::Chest | Class::EnderChest => {\n'),
    (LEVEL + 'tick.rs', '                Class::Farmland => {\n                    if !self.can_survive(state, pos) {\n                        self.turn_to_base_block(pos, state);\n                    }\n                }\n', '                Class::Farmland => {\n                    if !self.can_survive(state, pos) {\n                        let base = b.data(state).link.unwrap_or(b.named.dirt);\n                        self.change("fade", pos, None, b.default_of(base), vec![super::Then::ToBase(pos, state)]);\n                    }\n                }\n'),
    (LEVEL + 'tick.rs', '                Class::Frogspawn => {\n                    if !self.can_survive(state, pos) {\n                        self.destroy_block(pos, false, 512);\n                    } else {\n                        self.emit(LevelEvent::BlockEvent {\n                            pos,\n                            state,\n                            kind: "hatch",\n                        });\n                        self.destroy_block(pos, false, 512);\n                    }\n                }\n', '                Class::Frogspawn => {\n                    if !self.can_survive(state, pos) {\n                        self.change("fade", pos, None, b.air(), vec![super::Then::Destroy(pos, false, 512)]);\n                    } else {\n                        self.change("fade", pos, None, b.air(), vec![super::Then::Emit(pos, state, "hatch"), super::Then::Destroy(pos, false, 512)]);\n                    }\n                }\n'),
    (LEVEL + 'tick.rs', '                            self.set_block(pos, b.table.set_int(state, p.age, age + 1), 2);\n                        }\n                    }\n                }\n                Class::Farmland => self.farmland_random_tick(state, pos),\n', '                            self.change_to("grow", pos, None, b.table.set_int(state, p.age, age + 1), 2);\n                        }\n                    }\n                }\n                Class::Farmland => self.farmland_random_tick(state, pos),\n'),
    (LEVEL + 'tick.rs', '                            let next = self.growing_plant_grow_into_state(state);\n                            self.set_block_and_update(growth, next);\n', '                            let next = self.growing_plant_grow_into_state(state);\n                            self.change_to("spread", growth, Some(pos), next, 3);\n'),
    (LEVEL + 'tick.rs', '                    if !b.table.bool(state, p.persistent) && b.table.int(state, p.distance) == 7 {\n                        self.drop_resources(state, pos);\n                        self.remove_block(pos, false);\n                    }\n', '                    if !b.table.bool(state, p.persistent) && b.table.int(state, p.distance) == 7 {\n                        self.change("decay", pos, None, b.air(), vec![super::Then::Drop(state, pos), super::Then::Remove(pos)]);\n                    }\n'),
    (LEVEL + 'tick.rs', '                    if age < 3 && self.rand_int(10) == 0 {\n                        self.set_block(pos, b.table.set_int(state, p.age, age + 1), 2);\n                    }\n', '                    if age < 3 && self.rand_int(10) == 0 {\n                        self.change_to("grow", pos, None, b.table.set_int(state, p.age, age + 1), 2);\n                    }\n'),
    (LEVEL + 'tick.rs', '                    if !self.can_be_nylium(state, pos) {\n                        self.set_block_and_update(pos, b.default_of(b.named.netherrack));\n                    }\n', '                    if !self.can_be_nylium(state, pos) {\n                        self.change_to("fade", pos, None, b.default_of(b.named.netherrack), 3);\n                    }\n'),
    (LEVEL + 'tick.rs', '                Class::RedStoneOre => {\n                    if b.table.bool(state, p.lit) {\n                        self.set_block_and_update(pos, b.table.set_bool(state, p.lit, false));\n                    }\n                }\n                Class::SnowLayer => {\n                    if self.level.block_light(pos) > 11 {\n                        self.drop_resources(state, pos);\n                        self.remove_block(pos, false);\n                        self.game_event(pos, "minecraft:block_destroy");\n                    }\n                }\n', '                Class::RedStoneOre => {\n                    if b.table.bool(state, p.lit) {\n                        self.change_to("fade", pos, None, b.table.set_bool(state, p.lit, false), 3);\n                    }\n                }\n                Class::SnowLayer => {\n                    if self.level.block_light(pos) > 11 {\n                        self.change("fade", pos, None, b.air(), vec![super::Then::Drop(state, pos), super::Then::Remove(pos), super::Then::GameEvent(pos, "minecraft:block_destroy")]);\n                    }\n                }\n'),
    (LEVEL + 'tick.rs', '                            if age == 15 {\n                                self.set_block_and_update(above(pos), b.default_of(block));\n                                self.set_block(pos, b.table.set_int(state, p.age, 0), 260);\n                            } else {\n', '                            if age == 15 {\n                                let top = above(pos);\n                                self.change("grow", top, None, b.default_of(block), vec![super::Then::Set(top, b.default_of(block), 3), super::Then::Set(pos, b.table.set_int(state, p.age, 0), 260)]);\n                            } else {\n'),
    (LEVEL + 'tick.rs', '                        let next = b.table.set_int(state, p.age, age + 1);\n                        self.set_block(pos, next, 2);\n                        self.game_event(pos, "minecraft:block_change");\n                    }\n                }\n                Class::Vine => self.vine_random_tick(state, pos),\n', '                        let next = b.table.set_int(state, p.age, age + 1);\n                        self.change("grow", pos, None, next, vec![super::Then::Set(pos, next, 2), super::Then::GameEvent(pos, "minecraft:block_change")]);\n                    }\n                }\n                Class::Vine => self.vine_random_tick(state, pos),\n'),
    (LEVEL + 'tick.rs', '        if !self.can_survive(state, pos) {\n            self.remove_block(pos, false);\n        }\n        let below_state = self.below_state(pos);\n        let infiniburn = match self.level.dimension() {\n', '        if !self.can_survive(state, pos) {\n            self.change("fade", pos, None, b.air(), vec![super::Then::Remove(pos)]);\n        }\n        let below_state = self.below_state(pos);\n        let infiniburn = match self.level.dimension() {\n'),
    (LEVEL + 'tick.rs', '            && self.random().next_float() < 0.2 + age as f32 * 0.03\n        {\n            self.remove_block(pos, false);\n            return;\n        }\n', '            && self.random().next_float() < 0.2 + age as f32 * 0.03\n        {\n            self.change("fade", pos, None, b.air(), vec![super::Then::Remove(pos)]);\n            return;\n        }\n'),
    (LEVEL + 'tick.rs', '                if !self.sturdy(below(pos), Direction::Up) || age > 3 {\n                    self.remove_block(pos, false);\n                }\n                return;\n            }\n            if age == 15 && self.rand_int(4) == 0 && !self.can_burn(self.below_state(pos)) {\n                self.remove_block(pos, false);\n                return;\n            }\n', '                if !self.sturdy(below(pos), Direction::Up) || age > 3 {\n                    self.change("fade", pos, None, b.air(), vec![super::Then::Remove(pos)]);\n                }\n                return;\n            }\n            if age == 15 && self.rand_int(4) == 0 && !self.can_burn(self.below_state(pos)) {\n                self.change("fade", pos, None, b.air(), vec![super::Then::Remove(pos)]);\n                return;\n            }\n'),
    (LEVEL + 'tick.rs', '                let next = self.crop_state_for_age(state, age + 1);\n                self.set_block(pos, next, 2);\n', '                let next = self.crop_state_for_age(state, age + 1);\n                self.change_to("grow", pos, None, next, 2);\n'),
    (LEVEL + 'tick.rs', '        if age < 7 {\n            self.set_block(pos, b.table.set_int(state, b.props.age, age + 1), 2);\n            return;\n        }\n', '        if age < 7 {\n            self.change_to("grow", pos, None, b.table.set_int(state, b.props.age, age + 1), 2);\n            return;\n        }\n'),
    (LEVEL + 'tick.rs', '            if let (Some(fruit), Some(stem)) = (data.link, data.link2) {\n                self.set_block_and_update(relative, b.default_of(fruit));\n                self.set_block_and_update(\n                    pos,\n                    b.table\n                        .set_dir(b.default_of(stem), b.props.facing, direction),\n                );\n            }\n', '            if let (Some(fruit), Some(stem)) = (data.link, data.link2) {\n                let attached = b.table.set_dir(b.default_of(stem), b.props.facing, direction);\n                self.change("grow", relative, None, b.default_of(fruit), vec![super::Then::Set(relative, b.default_of(fruit), 3), super::Then::Set(pos, attached, 3)]);\n            }\n'),
    (LEVEL + 'tick.rs', '        let grown = b.table.set_int(lower, p.age, new_age);\n        self.set_block(lower_pos, grown, 2);\n        if new_age >= 3 {\n', '        let grown = b.table.set_int(lower, p.age, new_age);\n        let mut doing = vec![super::Then::Set(lower_pos, grown, 2)];\n        if new_age >= 3 {\n'),
    (LEVEL + 'tick.rs', '                .with_bool(upper, p.waterlogged, self.is_water_at(upper_pos))\n                .unwrap_or(upper);\n            self.set_block(upper_pos, upper, 3);\n        }\n    }\n', '                .with_bool(upper, p.waterlogged, self.is_water_at(upper_pos))\n                .unwrap_or(upper);\n            doing.push(super::Then::Set(upper_pos, upper, 3));\n        }\n        self.change("grow", lower_pos, None, grown, doing);\n    }\n'),
    (LEVEL + 'tick.rs', '            if self.random().next_double() <= chance {\n                self.set_block_and_update(above_pos, b.default_of(b.named.cactus_flower));\n            }\n        } else if age == 15 && height < 3 {\n            self.set_block_and_update(above_pos, b.default_of(block));\n            let reset = b.table.set_int(state, b.props.age, 0);\n            self.set_block(pos, reset, 260);\n            self.neighbor_changed_full(reset, above_pos, block, false);\n        }\n', '            if self.random().next_double() <= chance {\n                self.change_to("grow", above_pos, None, b.default_of(b.named.cactus_flower), 3);\n            }\n        } else if age == 15 && height < 3 {\n            let reset = b.table.set_int(state, b.props.age, 0);\n            self.change("grow", above_pos, None, b.default_of(block), vec![super::Then::Set(above_pos, b.default_of(block), 3), super::Then::Set(pos, reset, 260), super::Then::Neighbour(reset, above_pos, block)]);\n        }\n'),
    (LEVEL + 'tick.rs', '            if moisture > 0 {\n                self.set_block(\n                    pos,\n                    b.table.set_int(state, b.props.moisture, moisture - 1),\n                    2,\n                );\n            } else if !b.in_set(self.above_state(pos), &b.tags.maintains_farmland) {\n                self.turn_to_base_block(pos, state);\n            }\n        } else if moisture < 7 {\n            self.set_block(pos, b.table.set_int(state, b.props.moisture, 7), 2);\n        }\n', '            if moisture > 0 {\n                self.change_to("moisture", pos, None, b.table.set_int(state, b.props.moisture, moisture - 1), 2);\n            } else if !b.in_set(self.above_state(pos), &b.tags.maintains_farmland) {\n                let base = b.data(state).link.unwrap_or(b.named.dirt);\n                self.change("fade", pos, None, b.default_of(base), vec![super::Then::ToBase(pos, state)]);\n            }\n        } else if moisture < 7 {\n            self.change_to("moisture", pos, None, b.table.set_int(state, b.props.moisture, 7), 2);\n        }\n'),
    (LEVEL + 'tick.rs', '        let mut offset = self.mushroom_offset(pos);\n        for _ in 0..4 {\n', '        let source = pos;\n        let mut offset = self.mushroom_offset(pos);\n        for _ in 0..4 {\n'),
    (LEVEL + 'tick.rs', '        if self.is_empty_block(offset) && self.can_survive(state, offset) {\n            self.set_block(offset, state, 2);\n        }\n    }\n    fn mushroom_offset(', '        if self.is_empty_block(offset) && self.can_survive(state, offset) {\n            self.change_to("spread", offset, Some(source), state, 2);\n        }\n    }\n    fn mushroom_offset('),
    (LEVEL + 'tick.rs', '        if !self.snowy_can_stay_alive(state, pos) {\n            self.set_block_and_update(pos, b.default_of(base));\n            return;\n        }\n', '        if !self.snowy_can_stay_alive(state, pos) {\n            self.change_to("fade", pos, None, b.default_of(base), 3);\n            return;\n        }\n'),
    (LEVEL + 'tick.rs', '                let snowy = b.in_set(self.above_state(test), &b.tags.snow);\n                self.set_block_and_update(test, b.table.set_bool(default, b.props.snowy, snowy));\n', '                let snowy = b.in_set(self.above_state(test), &b.tags.snow);\n                self.change_to("spread", test, Some(pos), b.table.set_bool(default, b.props.snowy, snowy), 3);\n'),
    (LEVEL + 'tick.rs', '                fluid == crate::fluid::FluidState::WATER,\n            );\n            self.set_block_and_update(grow, state);\n        }\n    }\n    pub(crate) fn bamboo_sapling_grow(', '                fluid == crate::fluid::FluidState::WATER,\n            );\n            if can_cluster_grow {\n                self.change_to("spread", grow, Some(pos), state, 3);\n            } else {\n                self.change_to("grow", grow, None, state, 3);\n            }\n        }\n    }\n    pub(crate) fn bamboo_sapling_grow('),
    (LEVEL + 'tick.rs', '        let state = b.table.set_text(bamboo, b.props.leaves, "small");\n        self.set_block(above(pos), state, 3);\n    }\n', '        let state = b.table.set_text(bamboo, b.props.leaves, "small");\n        self.change_to("spread", above(pos), Some(pos), state, 3);\n    }\n'),
    (LEVEL + 'tick.rs', '        let below_two_state = self.state(below_two);\n        let mut leaves = "none";\n', '        let below_two_state = self.state(below_two);\n        let mut doing = Vec::new();\n        let mut leaves = "none";\n'),
    (LEVEL + 'tick.rs', '                if b.is_block(below_two_state, b.named.bamboo) {\n                    self.set_block(\n                        below(pos),\n                        b.table.set_text(below_state, p.leaves, "small"),\n                        3,\n                    );\n                    self.set_block(\n                        below_two,\n                        b.table.set_text(below_two_state, p.leaves, "none"),\n                        3,\n                    );\n                }\n', '                if b.is_block(below_two_state, b.named.bamboo) {\n                    doing.push(super::Then::Set(below(pos), b.table.set_text(below_state, p.leaves, "small"), 3));\n                    doing.push(super::Then::Set(below_two, b.table.set_text(below_two_state, p.leaves, "none"), 3));\n                }\n'),
    (LEVEL + 'tick.rs', '        let _ = self.bamboo_height_above(pos);\n        self.set_block(above(pos), next, 3);\n    }\n', '        let _ = self.bamboo_height_above(pos);\n        doing.push(super::Then::Set(above(pos), next, 3));\n        self.change("spread", above(pos), Some(pos), next, doing);\n    }\n'),
    (LEVEL + 'tick.rs', '                if cw_face && b.multiface_can_attach(self.state(cw_test), cw) {\n                    self.set_block(\n                        test,\n                        b.table.set_bool(default, self.direction_prop(cw), true),\n                        2,\n                    );\n                } else if ccw_face && b.multiface_can_attach(self.state(ccw_test), ccw) {\n                    self.set_block(\n                        test,\n                        b.table.set_bool(default, self.direction_prop(ccw), true),\n                        2,\n                    );\n                } else {\n', '                if cw_face && b.multiface_can_attach(self.state(cw_test), cw) {\n                    let vine = b.table.set_bool(default, self.direction_prop(cw), true);\n                    self.change_to("spread", test, Some(pos), vine, 2);\n                } else if ccw_face && b.multiface_can_attach(self.state(ccw_test), ccw) {\n                    let vine = b.table.set_bool(default, self.direction_prop(ccw), true);\n                    self.change_to("spread", test, Some(pos), vine, 2);\n                } else {\n'),
    (LEVEL + 'tick.rs', '                        self.set_block(\n                            cw_test,\n                            b.table\n                                .set_bool(default, self.direction_prop(opposite), true),\n                            2,\n                        );\n', '                        let vine = b.table.set_bool(default, self.direction_prop(opposite), true);\n                        self.change_to("spread", cw_test, Some(pos), vine, 2);\n'),
    (LEVEL + 'tick.rs', '                        self.set_block(\n                            ccw_test,\n                            b.table\n                                .set_bool(default, self.direction_prop(opposite), true),\n                            2,\n                        );\n', '                        let vine = b.table.set_bool(default, self.direction_prop(opposite), true);\n                        self.change_to("spread", ccw_test, Some(pos), vine, 2);\n'),
    (LEVEL + 'tick.rs', '                        self.set_block(test, b.table.set_bool(default, p.up, true), 2);\n', '                        self.change_to("spread", test, Some(pos), b.table.set_bool(default, p.up, true), 2);\n'),
    (LEVEL + 'tick.rs', '            } else if b.multiface_can_attach(edge, direction) {\n                self.set_block(\n                    pos,\n                    b.table\n                        .set_bool(state, self.direction_prop(direction), true),\n                    2,\n                );\n            }\n', '            } else if b.multiface_can_attach(edge, direction) {\n                let vine = b.table.set_bool(state, self.direction_prop(direction), true);\n                self.change_to("grow", pos, None, vine, 2);\n            }\n'),
    (LEVEL + 'tick.rs', '                self.set_block(pos, b.table.set_bool(state, p.up, true), 2);\n                return;\n', '                self.change_to("grow", pos, None, b.table.set_bool(state, p.up, true), 2);\n                return;\n'),
    (LEVEL + 'tick.rs', '                    self.set_block(above_pos, above_state, 2);\n                }\n                return;\n', '                    self.change_to("spread", above_pos, Some(pos), above_state, 2);\n                }\n                return;\n'),
    (LEVEL + 'tick.rs', '                    self.set_block(below_pos, after, 2);\n                }\n            }\n        }\n    }\n    fn vine_can_spread(', '                    if b.is_air(below_state) {\n                        self.change_to("spread", below_pos, Some(pos), after, 2);\n                    } else {\n                        self.change_to("grow", below_pos, None, after, 2);\n                    }\n                }\n            }\n        }\n    }\n    fn vine_can_spread('),
    (LEVEL + 'tick.rs', '            let connected = self.chorus_plant_with_connections(pos, plant_state);\n            self.set_block(pos, connected, 2);\n            self.set_block(\n                above_pos,\n                b.table.set_int(b.default_of(block), p.age, age),\n                2,\n            );\n            self.level_event(1033, above_pos, 0);\n        } else if age < 4 {\n', '            let connected = self.chorus_plant_with_connections(pos, plant_state);\n            let flower = b.table.set_int(b.default_of(block), p.age, age);\n            self.change("spread", above_pos, Some(pos), flower, vec![super::Then::Set(pos, connected, 2), super::Then::Set(above_pos, flower, 2), super::Then::LevelEvent(1033, above_pos)]);\n        } else if age < 4 {\n'),
    (LEVEL + 'tick.rs', '                    self.set_block(\n                        target,\n                        b.table.set_int(b.default_of(block), p.age, age + 1),\n                        2,\n                    );\n                    self.level_event(1033, target, 0);\n                    branched = true;\n', '                    let flower = b.table.set_int(b.default_of(block), p.age, age + 1);\n                    self.change("spread", target, Some(pos), flower, vec![super::Then::Set(target, flower, 2), super::Then::LevelEvent(1033, target)]);\n                    branched = true;\n'),
    (LEVEL + 'tick.rs', '            } else {\n                self.set_block(pos, b.table.set_int(state, p.age, 5), 2);\n                self.level_event(1034, pos, 0);\n            }\n        } else {\n            self.set_block(pos, b.table.set_int(state, p.age, 5), 2);\n            self.level_event(1034, pos, 0);\n        }\n', '            } else {\n                let dead = b.table.set_int(state, p.age, 5);\n                self.change("grow", pos, None, dead, vec![super::Then::Set(pos, dead, 2), super::Then::LevelEvent(1034, pos)]);\n            }\n        } else {\n            let dead = b.table.set_int(state, p.age, 5);\n            self.change("grow", pos, None, dead, vec![super::Then::Set(pos, dead, 2), super::Then::LevelEvent(1034, pos)]);\n        }\n'),
    (LEVEL + 'tick.rs', '        if info.freezes_below {\n            self.set_block_and_update(below_pos, b.default_of(b.named.ice));\n        }\n', '        if info.freezes_below {\n            self.change_to("form", below_pos, None, b.default_of(b.named.ice), 3);\n        }\n'),
    (LEVEL + 'tick.rs', '                if layers < max_height.min(8) {\n                    self.set_block_and_update(top, b.table.set_int(state, p.layers, layers + 1));\n                }\n            } else {\n                self.set_block_and_update(top, b.default_of(b.named.snow));\n            }\n', '                if layers < max_height.min(8) {\n                    self.change_to("form", top, None, b.table.set_int(state, p.layers, layers + 1), 3);\n                }\n            } else {\n                self.change_to("form", top, None, b.default_of(b.named.snow), 3);\n            }\n'),
    (LEVEL + 'tick.rs', '            if let Some(next) = self.copper_next(state) {\n                self.set_block_and_update(pos, next);\n            }\n', '            if let Some(next) = self.copper_next(state) {\n                self.change_to("form", pos, None, next, 3);\n            }\n'),
    (LEVEL + 'hooks.rs', '        if self.level.dimension() == super::Dimension::Nether {\n            self.remove_block(pos, false);\n        } else {\n            let water = b.default_of(b.named.water);\n            self.set_block_and_update(pos, water);\n            self.neighbor_changed_at(pos, b.named.water);\n            self.game_event_state(pos, "minecraft:block_destroy", state);\n        }\n', '        if self.level.dimension() == super::Dimension::Nether {\n            self.change("fade", pos, None, b.air(), vec![super::Then::Remove(pos)]);\n        } else {\n            let water = b.default_of(b.named.water);\n            self.change("fade", pos, None, water, vec![super::Then::Set(pos, water, 3), super::Then::NeighbourAt(pos, b.named.water), super::Then::GameEventOf(pos, "minecraft:block_destroy", state)]);\n        }\n'),
    (LEVEL + 'timeline.rs', '            self.play(pos, "minecraft:entity.turtle.egg_crack", 0.7, pitch);\n            self.set_block(pos, b.table.set_int(state, b.props.hatch, hatch + 1), 2);\n            self.game_event(pos, "minecraft:block_change");\n        } else {\n            self.play(pos, "minecraft:entity.turtle.egg_hatch", 0.7, pitch);\n            self.remove_block(pos, false);\n            self.game_event(pos, "minecraft:block_destroy");\n            for _ in 0..b.table.int(state, b.props.eggs) {\n                self.level_event(2001, pos, state.0 as i32);\n            }\n            // Baby turtles are entities the caller spawns (one per egg).\n            self.emit(LevelEvent::BlockEvent {\n                pos,\n                state,\n                kind: "hatch_turtles",\n            });\n        }\n', '            self.play(pos, "minecraft:entity.turtle.egg_crack", 0.7, pitch);\n            let next = b.table.set_int(state, b.props.hatch, hatch + 1);\n            self.change("grow", pos, None, next, vec![super::Then::Set(pos, next, 2), super::Then::GameEvent(pos, "minecraft:block_change")]);\n        } else {\n            self.play(pos, "minecraft:entity.turtle.egg_hatch", 0.7, pitch);\n            // Baby turtles are entities the caller spawns (one per egg).\n            self.change("fade", pos, None, b.air(), vec![super::Then::Remove(pos), super::Then::GameEvent(pos, "minecraft:block_destroy"), super::Then::Emit(pos, state, "hatch_turtles")]);\n            for _ in 0..b.table.int(state, b.props.eggs) {\n                self.level_event(2001, pos, state.0 as i32);\n            }\n        }\n'),
    (LEVEL + 'timeline.rs', '            self.play(pos, "minecraft:block.sniffer_egg.crack", 0.7, pitch);\n            self.set_block(pos, b.table.set_int(state, b.props.hatch, hatch + 1), 2);\n        } else {\n            self.play(pos, "minecraft:block.sniffer_egg.hatch", 0.7, pitch);\n            self.destroy_block(pos, false, 512);\n            // The snifflet is an entity the caller spawns.\n            self.emit(LevelEvent::BlockEvent {\n                pos,\n                state,\n                kind: "hatch_sniffer",\n            });\n        }\n', '            self.play(pos, "minecraft:block.sniffer_egg.crack", 0.7, pitch);\n            self.change_to("grow", pos, None, b.table.set_int(state, b.props.hatch, hatch + 1), 2);\n        } else {\n            self.play(pos, "minecraft:block.sniffer_egg.hatch", 0.7, pitch);\n            // The snifflet is an entity the caller spawns.\n            self.change("fade", pos, None, b.air(), vec![super::Then::Destroy(pos, false, 512), super::Then::Emit(pos, state, "hatch_sniffer")]);\n        }\n'),
    (LEVEL + 'timeline.rs', '                self.play(pos, "minecraft:block.dried_ghast.transition", 1.0, 1.0);\n                self.set_block(pos, b.table.set_int(state, key, hydration + 1), 2);\n                self.game_event(pos, "minecraft:block_change");\n            } else {\n                self.remove_block(pos, false);\n                // The happy ghastling is an entity the caller spawns.\n                self.emit(LevelEvent::BlockEvent {\n                    pos,\n                    state,\n                    kind: "spawn_ghastling",\n                });\n            }\n        } else if hydration > 0 {\n            self.set_block(pos, b.table.set_int(state, key, hydration - 1), 2);\n            self.game_event(pos, "minecraft:block_change");\n        }\n', '                self.play(pos, "minecraft:block.dried_ghast.transition", 1.0, 1.0);\n                let next = b.table.set_int(state, key, hydration + 1);\n                self.change("grow", pos, None, next, vec![super::Then::Set(pos, next, 2), super::Then::GameEvent(pos, "minecraft:block_change")]);\n            } else {\n                // The happy ghastling is an entity the caller spawns.\n                self.change("fade", pos, None, b.air(), vec![super::Then::Remove(pos), super::Then::Emit(pos, state, "spawn_ghastling")]);\n            }\n        } else if hydration > 0 {\n            let next = b.table.set_int(state, key, hydration - 1);\n            self.change("fade", pos, None, next, vec![super::Then::Set(pos, next, 2), super::Then::GameEvent(pos, "minecraft:block_change")]);\n        }\n'),
    (LEVEL + 'speleothem.rs', '        self.set_block_and_update(pos, b.table.set_bool(state, b.props.waterlogged, water));\n    }\n    fn speleothem_grow_to(', '        self.change_to("spread", pos, Some(rel(pos, direction.opposite())), b.table.set_bool(state, b.props.waterlogged, water), 3);\n    }\n    fn speleothem_grow_to('),
    (LEVEL + 'speleothem.rs', '            self.set_block_and_update(source_pos, b.default_of(b.named.clay));\n            self.game_event(source_pos, "minecraft:block_change");\n            self.level_event(1504, tip, 0);\n', '            let clay = b.default_of(b.named.clay);\n            self.change("form", source_pos, None, clay, vec![super::Then::Set(source_pos, clay, 3), super::Then::GameEvent(source_pos, "minecraft:block_change"), super::Then::LevelEvent(1504, tip)]);\n'),
    (LEVEL + 'shapes.rs', '                if self.concrete_touches_liquid(pos) {\n                    b.data(state)\n                        .link\n                        .map_or(state, |concrete| b.default_of(concrete))\n                } else {\n', '                if self.concrete_touches_liquid(pos) {\n                    let concrete = b.data(state).link.map_or(state, |concrete| b.default_of(concrete));\n                    if self.asked(super::Asked::Change { kind: "form", at: pos, source: None, state: concrete, doing: vec![super::Then::Set(pos, concrete, 3)] }) { state } else { concrete }\n                } else {\n'),
    (LEVEL + 'falling.rs', '                if let Some(solid) = self.data(state).link {\n                    sim.set_block_and_update(pos, self.default_of(solid));\n                }\n', '                if let Some(solid) = self.data(state).link {\n                    sim.change_to("form", pos, None, self.default_of(solid), 3);\n                }\n'),
    (LEVEL + 'sculk.rs', '                if self.set_block(target, new, 2) {\n                    count += 1;\n                }\n', '                if self.change_to("spread", target, Some(pos), new, 2) {\n                    count += 1;\n                }\n'),
    (LEVEL + 'sculk.rs', '                        self.set_block_and_update(growth_pos, growth);\n', '                        self.change_to("spread", growth_pos, Some(pos), growth, 3);\n'),
    (LEVEL + 'sculk.rs', '            self.set_block_and_update(support_pos, default_sculk);\n', '            self.change_to("spread", support_pos, Some(pos), default_sculk, 3);\n'),
    (LEVEL + 'fluids.rs', '            if !b.is_air(state) {\n                if is_water(kind) {\n                    self.drop_resources(state, pos);\n                } else {\n', '            if !b.is_air(state) {\n                if is_water(kind) {\n                    if !FLOW_DROPPED.get() {\n                        self.drop_resources(state, pos);\n                    }\n                } else {\n'),
    (LEVEL + 'fluids.rs', '            if b.instance_of(state, Class::Liquid) {\n                self.set_block_and_update(pos, b.default_of(b.named.stone));\n            }\n            self.level_event(1501, pos, 0);\n            return;\n        }\n', '            let stone = b.default_of(b.named.stone);\n            if b.instance_of(state, Class::Liquid) {\n                self.change("form", pos, None, stone, vec![super::Then::Set(pos, stone, 3), super::Then::LevelEvent(1501, pos)]);\n            } else {\n                self.level_event(1501, pos, 0);\n            }\n            return;\n        }\n'),
    (LEVEL + 'fluids.rs', "    /// `FlowingFluid.spreadTo` (with LavaFluid's water-to-stone override).\n    fn spread_to(\n", "    /// The rest of a fluid's move, once its listeners have let it through.\n    #[allow(clippy::too_many_arguments)]\n    pub(super) fn flow_on(&mut self, kind: FluidKind, to: BlockPos, state: BlockStateId, direction: Direction, target: FluidState, rest: Option<(BlockPos, FluidState, BlockStateId)>, dropped: bool) {\n        FLOW_DROPPED.set(dropped);\n        self.spread_to(kind, to, state, direction, target);\n        FLOW_DROPPED.set(false);\n        if let Some((pos, fluid, state)) = rest\n            && self.source_neighbour_count(kind, pos) >= 3\n        {\n            self.spread_to_sides(kind, pos, fluid, state);\n        }\n    }\n    /// `FlowingFluid.spreadTo` (with LavaFluid's water-to-stone override).\n    fn spread_to(\n"),
    (LEVEL + 'fluids.rs', '                let neighbour_state = self.state(neighbour_pos);\n                self.spread_to(kind, neighbour_pos, neighbour_state, direction, new_fluid);\n', '                let neighbour_state = self.state(neighbour_pos);\n                if !self.asked(super::Asked::Flow { kind, to: neighbour_pos, state: neighbour_state, direction, target: new_fluid, rest: None }) {\n                    self.spread_to(kind, neighbour_pos, neighbour_state, direction, new_fluid);\n                }\n'),
    (LEVEL + 'fluids.rs', '                self.spread_to(kind, below_pos, below_state, Direction::Down, new_below);\n                if self.source_neighbour_count(kind, pos) >= 3 {\n', '                if self.asked(super::Asked::Flow { kind, to: below_pos, state: below_state, direction: Direction::Down, target: new_below, rest: Some((pos, fluid, state)) }) {\n                    return;\n                }\n                self.spread_to(kind, below_pos, below_state, Direction::Down, new_below);\n                if self.source_neighbour_count(kind, pos) >= 3 {\n'),
    (LEVEL + 'fluids.rs', '    /// `FlowingFluid.spread`.\n    fn spread(&mut self,', '    /// `FlowingFluid.spread`.\n    pub(super) fn spread(&mut self,'),
    (LEVEL + 'fluids.rs', '                fluid = new_fluid;\n                state = self.b.air();\n                self.set_block_and_update(pos, state);\n            } else if new_fluid != fluid {\n                fluid = new_fluid;\n                state = self.b.legacy_block(fluid);\n                self.set_block_and_update(pos, state);\n                self.schedule_fluid_tick(pos, fluid.kind(), delay);\n            }\n', '                fluid = new_fluid;\n                state = self.b.air();\n                if self.asked(super::Asked::Change { kind: "fluid", at: pos, source: None, state, doing: vec![super::Then::Set(pos, state, 3), super::Then::FluidOn(kind, pos, state, fluid, None)] }) {\n                    return;\n                }\n                self.set_block_and_update(pos, state);\n            } else if new_fluid != fluid {\n                fluid = new_fluid;\n                state = self.b.legacy_block(fluid);\n                if self.asked(super::Asked::Change { kind: "fluid", at: pos, source: None, state, doing: vec![super::Then::Set(pos, state, 3), super::Then::FluidOn(kind, pos, state, fluid, Some(delay))] }) {\n                    return;\n                }\n                self.set_block_and_update(pos, state);\n                self.schedule_fluid_tick(pos, fluid.kind(), delay);\n            }\n'),
    (LEVEL + 'fluids.rs', '                self.set_block_and_update(pos, b.default_of(block));\n                self.level_event(1501, pos, 0);\n                return false;\n            }\n            if over_soul_soil && b.is_block(self.state(neighbour), b.named.blue_ice) {\n                self.set_block_and_update(pos, b.default_of(b.named.basalt));\n                self.level_event(1501, pos, 0);\n                return false;\n            }\n', '                self.change("form", pos, None, b.default_of(block), vec![super::Then::Set(pos, b.default_of(block), 3), super::Then::LevelEvent(1501, pos)]);\n                return false;\n            }\n            if over_soul_soil && b.is_block(self.state(neighbour), b.named.blue_ice) {\n                let basalt = b.default_of(b.named.basalt);\n                self.change("form", pos, None, basalt, vec![super::Then::Set(pos, basalt, 3), super::Then::LevelEvent(1501, pos)]);\n                return false;\n            }\n'),
    (LEVEL + 'fluids.rs', 'fn is_water(kind: FluidKind) -> bool {\n', 'thread_local! {\n    /// While set, what water takes the place of has had its drops seen to.\n    static FLOW_DROPPED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };\n}\n\nfn is_water(kind: FluidKind) -> bool {\n'),
    # ---- `BlockPistonExtendEvent` and `BlockPistonRetractEvent`: asked before the piston does anything.
    (LEVEL + 'pistons.rs', '''pub static BROKEN_BY: std::sync::Mutex<Vec<([i32; 3], [i32; 3])>> = std::sync::Mutex::new(Vec::new());
''', '''pub static BROKEN_BY: std::sync::Mutex<Vec<([i32; 3], [i32; 3])>> = std::sync::Mutex::new(Vec::new());
thread_local! {
    /// Set while a piston its listeners let through moves.
    static PISTON_TOLD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
'''),
    (LEVEL + 'pistons.rs', '''        if a == 0 {
            if self.piston_move_blocks(pos, direction, true, sticky) != Some(true) {
                return false;
            }
''', '''        if !self.level.is_client_side()
            && !PISTON_TOLD.get()
            && let Some((kind, moves, blocks)) = self.piston_question(pos, direction, a, sticky)
            && self.asked(super::Asked::Piston { kind, at: pos, state, a, param_b, direction: moves, sticky, blocks })
        {
            return false;
        }
        if a == 0 {
            if self.piston_move_blocks(pos, direction, true, sticky) != Some(true) {
                return false;
            }
'''),
    (LEVEL + 'pistons.rs', '''    fn piston_trigger_event(
        &mut self,
''', '''    /// Which event this block event is, the way it says the piston goes and what moves: none if it
    /// can't move, or retracts without a word.
    fn piston_question(&self, pos: BlockPos, direction: Direction, a: i32, sticky: bool) -> Option<(&'static str, Direction, Vec<BlockPos>)> {
        let b = self.b;
        if a == 0 {
            let (push, destroy, _) = self.piston_resolve(pos, direction, true)?;
            return Some(("extend", direction, push.into_iter().chain(destroy).collect()));
        }
        if a != 1 && a != 2 {
            return None;
        }
        if !sticky {
            return Some(("retract", direction, Vec::new()));
        }
        let two = rel_n(pos, direction, 2);
        let moving_state = self.state(two);
        if b.is_block(moving_state, b.named.moving_piston) && self.moving_piston(two).is_some_and(|entity| entity.direction == direction && entity.extending) {
            return None;
        }
        let is_piston = b.is_block(moving_state, b.named.piston) || b.is_block(moving_state, b.named.sticky_piston);
        let held = a != 1
            || b.is_air(moving_state)
            || !self.is_pushable(moving_state, two, direction.opposite(), false, direction)
            || (b.facts(moving_state).push != PushReaction::PushPull && !is_piston);
        if held {
            return (a == 1 && b.is_air(moving_state)).then(|| ("retract", direction, Vec::new()));
        }
        let (push, destroy, _) = self.piston_resolve(pos, direction, false)?;
        Some(("retract", direction.opposite(), push.into_iter().chain(destroy).collect()))
    }
    fn piston_trigger_event(
        &mut self,
'''),
    (LEVEL + 'pistons.rs', '''    pub fn run_block_events<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks) {
        let mut reschedule = Vec::new();
        while !ticks.block_events.is_empty() {
''', '''    /// A piston its listeners let through, with the block event it would have been.
    pub fn piston_told<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos, state: BlockStateId, a: i32, param_b: i32) {
        let mut sim = self.sim(level, ticks);
        if sim.state(pos) != state {
            return;
        }
        PISTON_TOLD.set(true);
        let told = sim.trigger_event(state, pos, a, param_b);
        PISTON_TOLD.set(false);
        if told {
            sim.emit(LevelEvent::BlockEntityEvent { pos, state, a, b: param_b });
        }
    }
    pub fn run_block_events<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks) {
        let mut reschedule = Vec::new();
        while !ticks.block_events.is_empty() && !super::ASK_WAITING.load(std::sync::atomic::Ordering::Relaxed) {
'''),
    # ---- `AsyncStructureSpawnEvent`: as a chunk's start is made, once a start per structure.
    ('horizon-world/src/worldgen/structure/start.rs', '''                set.structures[0].0,
                chunk,
                false,
            ) {
                starts.push(start);
            }
            continue;
''', '''                set.structures[0].0,
                chunk,
                false,
            ) && structure_spawns(&start)
            {
                starts.push(start);
            }
            continue;
'''),
    ('horizon-world/src/worldgen/structure/start.rs', '''                registry, resources, bindings, context, selected.0, chunk, false,
            ) {
                starts.push(start);
                break;
            }
''', '''                registry, resources, bindings, context, selected.0, chunk, false,
            ) && structure_spawns(&start)
            {
                starts.push(start);
                break;
            }
'''),
    ('horizon-world/src/worldgen/structure/start.rs', '''/// `Structure.generate` of one structure for one chunk: what the chunk's
''', '''/// Asked as a structure's start is made: the level, its name, its box and the chunk. False stops it.
pub static STRUCTURE_SPAWN_HOOK: std::sync::OnceLock<fn(&str, &str, [i32; 3], [i32; 3], [i32; 2]) -> bool> = std::sync::OnceLock::new();

thread_local! {
    /// Set while a chunk is really being made: a start worked out again for one that's saved isn't a new one.
    static STRUCTURES_NEW: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// While it lives, the starts this thread works out are new ones, of this level.
pub struct NewStarts;

impl NewStarts {
    pub fn begin(level: &str) -> Self {
        STRUCTURE_LEVEL.with_borrow_mut(|known| level.clone_into(known));
        STRUCTURES_NEW.set(true);
        Self
    }
}

impl Drop for NewStarts {
    fn drop(&mut self) {
        STRUCTURES_NEW.set(false);
    }
}

fn structure_spawns(start: &StructureStart) -> bool {
    !STRUCTURES_NEW.get()
        || STRUCTURE_SPAWN_HOOK.get().is_none_or(|hook| {
            let level = STRUCTURE_LEVEL.with_borrow(Clone::clone);
            hook(&level, &start.name, start.bounding_box.min, start.bounding_box.max, [start.chunk.x, start.chunk.z])
        })
}

/// `Structure.generate` of one structure for one chunk: what the chunk's
'''),
    ('horizon-client/src/vanillagen.rs', '''    fn proto(&self, worker: &mut GeneratorWorker, x: i32, z: i32) -> Proto {
''', '''    fn proto(&self, worker: &mut GeneratorWorker, x: i32, z: i32) -> Proto {
        let _new = horizon_world::worldgen::structure::start::NewStarts::begin(&self.dimension);
'''),
    # ---- `BlockDispenseArmorEvent`: of who'll wear it.
    (GAME + 'dispenser.rs', '''        let Some(target) = target else {
            return false;
        };
        stack.count -= 1;
        let (at, sound) = match target {
''', '''        let Some(target) = target else {
            return false;
        };
        let (entity, player) = match target {
            Target::Mob(i) => (Some(self.mobs[i].id as i64), false),
            Target::Stand(i) => (Some(self.armor_stands[i].id as i64), false),
            Target::Player => (None, true),
        };
        if super::paper_fam_ev_world::armor_refused(self, pos, stack, entity, player) {
            return true;
        }
        stack.count -= 1;
        let (at, sound) = match target {
'''),
    # ---- `LightningStrikeEvent.getCause`: a skeleton trap's own.
    (GAME + 'passengers.rs', '''        let at = [horse.position.x, horse.position.y, horse.position.z];
        self.spawn_bolt(at, true);
''', '''        let at = [horse.position.x, horse.position.y, horse.position.z];
        super::paper_fam_ev_world::set_bolt_cause("TRAP");
        self.spawn_bolt(at, true);
'''),
    # ---- `BlockExpEvent`: what a grindstone gives.
    (GAME + 'stations.rs', '''        let xp = stations::grindstone_xp_roll(amount, &mut self.world.random);
        if xp > 0 {
''', '''        let xp = stations::grindstone_xp_roll(amount, &mut self.world.random);
        let xp = super::paper_fam_ev_world::grind_xp(self, pos, xp as i32) as _;
        if xp > 0 {
'''),
    # ---- `VehicleUpdateEvent` and `VehicleMoveEvent`: every vehicle, every tick.
    ('horizon-client/src/game.rs', '''            horizon_client::spans::time("tick_carts", || self.tick_carts());
''', '''            horizon_client::spans::time("tick_carts", || self.tick_carts());
            paper_fam_ev_world::vehicles_ticked(self);
'''),
    # ---- `WhitelistStateUpdateEvent`: each name in turn, only if it'd change anything.
    (GAME + 'commands/profiles.rs', '''                for target in &targets {
                    let done = {
                        let mut l = lists.lock().unwrap();
                        if add {
''', '''                for target in &targets {
                    let changes = {
                        let l = lists.lock().unwrap();
                        l.white.contains(target) != add
                    };
                    if changes && crate::game::paper_fam_ev_world::whitelist_refused(self, add, &target.name, target.id) {
                        continue;
                    }
                    let done = {
                        let mut l = lists.lock().unwrap();
                        if add {
'''),
    # ---- `MapInitializeEvent`: once the map's there.
    (GAME + 'maps.rs', '''        let id = self.next_map_id();
        self.maps.data.insert(id, data);
        id
    }
''', '''        let id = self.next_map_id();
        self.maps.data.insert(id, data);
        super::paper_fam_ev_world::map_made(self, id);
        id
    }
'''),
    (GAME + 'maps.rs', '''        let data = MapData::fresh(p.x, p.z, 0, self.dimension_id());
        self.maps.data.insert(id, data);
        let mut filled = ItemStack::new("minecraft:filled_map", 1);
''', '''        let data = MapData::fresh(p.x, p.z, 0, self.dimension_id());
        self.maps.data.insert(id, data);
        super::paper_fam_ev_world::map_made(self, id);
        let mut filled = ItemStack::new("minecraft:filled_map", 1);
'''),
    # ---- `BlockReceiveGameEvent`: whatever a listener makes of it, as it arrives.
    (GAME + 'sculk.rs', '''        if !can_receive {
            return;
        }
        let dest = center(key);
''', '''        if super::paper_fam_ev_world::vibration_refused(self, key, event, ctx.source, !can_receive) {
            return;
        }
        let dest = center(key);
'''),
    # ---- `TargetHitEvent`: before the block lights, with what it's to give.
    (LEVEL + 'redstone.rs', '''        arrow: bool,
    ) -> Option<i32> {
        let b = self.b;
        let state = self.state(pos);
        if !b.data(state).is(Class::Target) {
            return None;
        }
''', '''        arrow: bool,
    ) -> Option<i32> {
        let strength = self.target_strength(pos, location, face)?;
        self.target_powers(pos, strength, arrow);
        Some(strength)
    }
    /// `TargetBlock.getRedstoneStrength`, if there's a target there.
    pub(crate) fn target_strength(&self, pos: BlockPos, location: [f64; 3], face: [i32; 3]) -> Option<i32> {
        let b = self.b;
        let state = self.state(pos);
        if !b.data(state).is(Class::Target) {
            return None;
        }
'''),
    (LEVEL + 'redstone.rs', '''        let strength = ((15.0 * ((0.5 - distance) / 0.5).clamp(0.0, 1.0)).ceil() as i32).max(1);
        let block = b.block(state);
''', '''        Some(((15.0 * ((0.5 - distance) / 0.5).clamp(0.0, 1.0)).ceil() as i32).max(1))
    }
    /// `TargetBlock.setOutputPower`, unless it's giving out already.
    pub(crate) fn target_powers(&mut self, pos: BlockPos, strength: i32, arrow: bool) {
        let b = self.b;
        let state = self.state(pos);
        let block = b.block(state);
'''),
    (LEVEL + 'redstone.rs', '''            self.schedule_tick(pos, block, if arrow { 20 } else { 8 });
        }
        Some(strength)
    }
''', '''            self.schedule_tick(pos, block, if arrow { 20 } else { 8 });
        }
    }
'''),
    (GAME + 'creatures.rs', '''        let hit = self
            .with_level(|b, level, ticks| {
                b.target_projectile_hit(level, ticks, block, location, face, arrow)
            });
        if let Some(strength) = hit {
''', '''        let hit = self
            .with_level(|b, level, ticks| b.target_strength(level, ticks, block, location, face))
            .map(|strength| match super::paper_fam_ev_world::target_said(self, p.id, pos, face, strength) {
                Some(strength) => {
                    self.with_level(|b, level, ticks| b.target_powers(level, ticks, block, strength, arrow));
                    strength
                }
                None => 0,
            });
        if let Some(strength) = hit {
'''),
    # ---- `BlockRedstoneEvent`: levers and buttons, asked before they move.
    (LEVEL + 'redstone.rs', '''    pub(crate) fn button_press(&mut self, pos: BlockPos, state: BlockStateId) {
        let b = self.b;
        self.set_block_and_update(pos, b.table.set_bool(state, b.props.powered, true));
''', '''    pub(crate) fn button_press(&mut self, pos: BlockPos, state: BlockStateId) {
        let next = self.b.table.set_bool(state, self.b.props.powered, true);
        self.change("redstone", pos, None, next, vec![super::Then::Press(pos, state)]);
    }
    pub(crate) fn button_pressed(&mut self, pos: BlockPos, state: BlockStateId) {
        let b = self.b;
        self.set_block_and_update(pos, b.table.set_bool(state, b.props.powered, true));
'''),
    (LEVEL + 'redstone.rs', '''        if should_be_pressed != was_pressed {
            self.set_block_and_update(
                pos,
                b.table.set_bool(state, b.props.powered, should_be_pressed),
            );
            self.attached_update_neighbours(pos, state);
            if let Some(set) = data.set_type.clone() {
                let sound = if should_be_pressed {
''', '''        if should_be_pressed != was_pressed {
            let next = b.table.set_bool(state, b.props.powered, should_be_pressed);
            self.change("redstone", pos, None, next, vec![super::Then::Release(pos, state, should_be_pressed)]);
        }
        if should_be_pressed {
            self.schedule_tick(pos, b.block(state), data.ticks_to_stay_pressed);
        }
    }
    pub(crate) fn button_turned(&mut self, pos: BlockPos, state: BlockStateId, should_be_pressed: bool) {
        let b = self.b;
        let data = b.data(state);
        {
            self.set_block_and_update(
                pos,
                b.table.set_bool(state, b.props.powered, should_be_pressed),
            );
            self.attached_update_neighbours(pos, state);
            if let Some(set) = data.set_type.clone() {
                let sound = if should_be_pressed {
'''),
    (LEVEL + 'redstone.rs', '''                    "minecraft:block_deactivate"
                },
            );
        }
        if should_be_pressed {
            self.schedule_tick(pos, b.block(state), data.ticks_to_stay_pressed);
        }
    }
''', '''                    "minecraft:block_deactivate"
                },
            );
        }
    }
'''),
    (LEVEL + 'redstone.rs', '''    pub(crate) fn lever_pull(&mut self, pos: BlockPos, state: BlockStateId) {
        let b = self.b;
        let state = b.table.cycle(state, b.props.powered);
''', '''    pub(crate) fn lever_pull(&mut self, pos: BlockPos, state: BlockStateId) {
        let next = self.b.table.cycle(state, self.b.props.powered);
        self.change("redstone", pos, None, next, vec![super::Then::Lever(pos, state)]);
    }
    pub(crate) fn lever_pulled(&mut self, pos: BlockPos, state: BlockStateId) {
        let b = self.b;
        let state = b.table.cycle(state, b.props.powered);
'''),
    # ---- `BlockDropItemEvent`: what fell, as the entities it is.
    ('horizon-client/src/game.rs', '''                    .is_some_and(|h| h.enchantment_level("minecraft:silk_touch") > 0);
                for d in drops {
                    if &*d.item != "minecraft:air" {
                        self.pop_resource(pos, d);
                    }
                }
''', '''                    .is_some_and(|h| h.enchantment_level("minecraft:silk_touch") > 0);
                let first_drop = self.dropped.len();
                for d in drops {
                    if &*d.item != "minecraft:air" {
                        self.pop_resource(pos, d);
                    }
                }
                paper_fam_ev_world::drops_fell(self, pos, state, first_drop);
'''),
    # ---- `AsyncPlayerSendSuggestionsEvent`: the game's own, before they go.
    (GAME + 'lan_actions.rs', '''        suggestions.truncate(1000);
        let utf16 = |byte: usize| {''', '''        suggestions.truncate(1000);
        if let Some(player) = host.hub.guest(conn).map(|g| g.uuid)
            && let Some(left) = super::paper_fam_ev_world::sent_suggestions(self, player, &text, suggestions.iter().map(|s| s.text.clone()).collect())
        {
            if left.is_empty() {
                return;
            }
            let like = suggestions.first().cloned();
            suggestions.retain(|s| left.contains(&s.text));
            for word in left {
                if !suggestions.iter().any(|s| s.text == word)
                    && let Some(like) = &like
                {
                    suggestions.push(horizon_command::Suggestion { text: word, tooltip: None, int_value: None, ..like.clone() });
                }
            }
        }
        let utf16 = |byte: usize| {'''),
    # ---- A dispenser's slot, and whether it had anything.
    (GAME + 'machines.rs', '''        let Some(stack) = self.containers().get(&view, slot).cloned() else {''', '''        super::paper_fam_ev_world::note_slot(slot);
        let Some(stack) = self.containers().get(&view, slot).cloned() else {'''),
    (GAME + 'machines.rs', '''            self.dispenser_event(1001, key, 0);''', '''            super::paper_fam_ev_world::note_empty();
            self.dispenser_event(1001, key, 0);'''),
    # ---- The night a bed skips.
    (GAME + '../game.rs', '''                let t = self.world.day_time;
                self.world.day_time = t + 24000 - t.rem_euclid(24000);''', '''                let t = self.world.day_time;
                if let Some(skip) = paper_fam_ev_world::sleep_skip(self, 24000 - t.rem_euclid(24000)) {
                    self.world.day_time = t + skip;
                }'''),
    # ---- What bone meal grows is a change of the world's too.
    (GAME + 'growth.rs', '''        let used = self
            .with_level(|b, level, ticks| b.use_bone_meal(level, ticks, pos, face));''', '''        super::paper_bridge::on_bone_meal_begin();
        let used = self
            .with_level(|b, level, ticks| b.use_bone_meal(level, ticks, pos, face));
        super::paper_bridge::on_block_tick_pause();'''),
    # ---- `BlockFertilizeEvent`: what the bone meal wrote.
    (GAME + 'growth.rs', '''        let pos = BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]);
        super::paper_bridge::on_bone_meal_begin();
        let used = self
            .with_level(|b, level, ticks| b.use_bone_meal(level, ticks, pos, face));
        super::paper_bridge::on_block_tick_pause();
''', '''        let pos = BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]);
        horizon_world::worldgen::structure::start::start_structure_writes();
        super::paper_bridge::on_bone_meal_begin();
        let used = self
            .with_level(|b, level, ticks| b.use_bone_meal(level, ticks, pos, face));
        super::paper_bridge::on_block_tick_pause();
        let writes = horizon_world::worldgen::structure::start::take_structure_writes();
        super::paper_fam_ev_world::run_asked(self);
        if used && !super::paper_fam_ev_world::fertilized(self, hit.pos, writes) {
            return Ok(false);
        }
'''),
]
