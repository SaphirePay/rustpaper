# Family items. Yours alone. Applied by fabric-port/tools/dim_patch.py, after the mixins: each text must be in its file once.
CLIENT = 'horizon-client/src/'
GAME = CLIENT + 'game/'
WORLD = 'horizon-world/src/'
ITEMS = 'horizon-items/src/'
RENDER = 'horizon-client-render/src/'

EDITS = [
    # ---- `Block.getData` and `BlockState.setRawData`: the flattening table, as `CraftLegacy` builds its own.
    (WORLD + 'datafix/mod.rs', 'mod block_state_data;', 'pub mod block_state_data;'),

    # ---- `Block.fluidTick` now, and the signals by face.
    (WORLD + 'level/mod.rs', '''    /// `Level.removeBlock(pos, movedByPiston)`: replace with the fluid's block.
''', '''    /// `FluidState.tick` at once, for `Block.fluidTick`.
    pub fn fluid_tick_at<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos) {
        self.sim(level, ticks).fluid_tick(pos);
    }
    /// `SignalGetter.getSignal(pos, direction)`.
    pub fn signal_at<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos, direction: Direction) -> i32 {
        self.sim(level, ticks).signal(pos, direction)
    }
    /// `SignalGetter.getDirectSignalTo(pos)`.
    pub fn direct_signal_to_at<L: LevelAccess + ?Sized>(&self, level: &mut L, ticks: &mut Ticks, pos: BlockPos) -> i32 {
        self.sim(level, ticks).direct_signal_to(pos)
    }
    /// `Level.removeBlock(pos, movedByPiston)`: replace with the fluid's block.
'''),

    # ---- A trim with a direct material or pattern: kept as saved, and sent as a direct holder.
    (ITEMS + 'item_data.rs', '''            if !MODELLED.contains(&key.as_str()) && !ominous_part {
''', '''            if (!MODELLED.contains(&key.as_str()) || (key == "minecraft:trim" && data.trim.is_none())) && !ominous_part {
'''),
    (CLIENT + 'lan/guest.rs', '''        for (name, value) in &d.other {
            let (removed, kind) = match name.strip_prefix('!') {
                Some(kind) => (true, kind),
                None => (false, name.as_str()),
            };
            if horizon_protocol::item::custom::id(kind).is_some() {
''', '''        // `ArmorTrim.STREAM_CODEC`, a holder each: the id plus one, or 0 and the definition.
        if let Some(trim) = d.other("minecraft:trim") {
            let mut w = horizon_protocol::buf::Writer::new();
            let mut holder = |w: &mut horizon_protocol::buf::Writer, key: &str, registry: &str, asset: &str| {
                let Some(tag) = trim.get(key) else {
                    return false;
                };
                if let Some(name) = crate::save::text(Some(tag)) {
                    let name = if name.contains(':') { name } else { format!("minecraft:{name}") };
                    let Some(id) = synced_id(registry, &name) else {
                        return false;
                    };
                    w.varint(id + 1);
                    return true;
                }
                let Some(id) = crate::save::text(tag.get(asset)) else {
                    return false;
                };
                w.varint(0);
                w.string(&id);
                w.nbt(tag.get("description").unwrap_or(&horizon_world::nbt::Tag::String("".into())));
                if key == "pattern" {
                    w.bool(matches!(tag.get("decal"), Some(horizon_world::nbt::Tag::Byte(1))));
                }
                true
            };
            if holder(&mut w, "material", "minecraft:trim_material", "palette_id") && holder(&mut w, "pattern", "minecraft:trim_pattern", "asset_id") {
                stack.added.push((horizon_protocol::item::component::TRIM, horizon_protocol::item::Component::Raw(w.data)));
            }
        }
        for (name, value) in &d.other {
            if name == "minecraft:trim" {
                continue;
            }
            let (removed, kind) = match name.strip_prefix('!') {
                Some(kind) => (true, kind),
                None => (false, name.as_str()),
            };
            if removed || horizon_protocol::item::custom::id(kind).is_some() {
'''),

    # ---- Brewing recipes a plugin adds at run time: `RecipeManager.addRecipe`, after the data's, with no reload.
    (RENDER + 'brewing.rs', '''impl Brewing {
    pub fn load(root: &Path, items: &Items, overlay: &PackOverlay) -> Self {
''', '''/// What a plugin's mix takes on one side: items, or stacks with the same components.
#[derive(Clone, Debug, Default)]
pub struct MixSide {
    pub items: HashSet<String>,
    pub stacks: Vec<ItemStack>,
}
impl MixSide {
    fn test(&self, stack: &ItemStack) -> bool {
        if self.stacks.is_empty() {
            self.items.contains(&*stack.item)
        } else {
            self.stacks.iter().any(|known| known.same_item_same_components(stack))
        }
    }
}
/// A `BrewingRecipe` added at run time. Its `recipe` lives as long as the game, as `find` hands it out.
pub struct CustomMix {
    pub input: MixSide,
    pub reagent: MixSide,
    pub recipe: &'static BrewingRecipe,
}
pub static CUSTOM_MIXES: std::sync::Mutex<Vec<CustomMix>> = std::sync::Mutex::new(Vec::new());
/// The data's recipes taken out by `RecipeManager.removeRecipe`.
pub static REMOVED_MIXES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

impl Brewing {
    pub fn load(root: &Path, items: &Items, overlay: &PackOverlay) -> Self {
'''),
    (RENDER + 'brewing.rs', '''        self.recipes
            .iter()
            .find(|r| r.input.test(input) && r.reagent.test(reagent))
    }
    pub fn is_reagent(&self, stack: &ItemStack) -> bool {
        self.reagents.contains(&*stack.item)
    }
    pub fn is_bottle(&self, stack: &ItemStack) -> bool {
        self.inputs.contains(&*stack.item)
    }
''', '''        let removed = REMOVED_MIXES.lock().unwrap_or_else(|e| e.into_inner());
        let known = self
            .recipes
            .iter()
            .find(|r| r.input.test(input) && r.reagent.test(reagent) && (removed.is_empty() || !removed.contains(&r.id)));
        drop(removed);
        if known.is_some() {
            return known;
        }
        let mixes = CUSTOM_MIXES.lock().unwrap_or_else(|e| e.into_inner());
        mixes.iter().find(|mix| mix.input.test(input) && mix.reagent.test(reagent)).map(|mix| mix.recipe)
    }
    pub fn is_reagent(&self, stack: &ItemStack) -> bool {
        self.reagents.contains(&*stack.item) || CUSTOM_MIXES.lock().unwrap_or_else(|e| e.into_inner()).iter().any(|mix| mix.reagent.test(stack))
    }
    pub fn is_bottle(&self, stack: &ItemStack) -> bool {
        self.inputs.contains(&*stack.item) || CUSTOM_MIXES.lock().unwrap_or_else(|e| e.into_inner()).iter().any(|mix| mix.input.test(stack))
    }
'''),

    # ---- `AnvilMenu.maximumRepairCost`: 40 unless a plugin set it, the creative rule apart.
    (CLIENT + 'stations.rs', '''/// `AnvilMenu.createResult`. `name` is the rename box text (None = untouched).
''', '''/// `AnvilMenu.maximumRepairCost` for the sum being worked out: a plugin's, set around the call.
pub static ANVIL_LIMIT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(40);

/// `AnvilMenu.createResult`. `name` is the rename box text (None = untouched).
'''),
    (CLIENT + 'stations.rs', '''        if cost >= 40 {
            cost = 39;
        }
''', '''        let limit = i64::from(ANVIL_LIMIT.load(std::sync::atomic::Ordering::Relaxed)) as _;
        if cost >= limit {
            cost = limit - 1;
        }
'''),
    (CLIENT + 'stations.rs', '''    if cost >= 40 && !creative {
''', '''    if cost >= i64::from(ANVIL_LIMIT.load(std::sync::atomic::Ordering::Relaxed)) as _ && !creative {
'''),

    # ---- `EnchantmentMenu.enchantmentSeed`: the menu's, which a plugin can set, and the player's only at the start.
    (GAME + 'lan_stations.rs', '''        let seed = host.hub.guests[gi].xp_seed;
''', '''        let seed = gs.menu.data[3];
'''),
    (GAME + 'lan_stations.rs', '''        let (seed, creative, conn) = (g.xp_seed, g.creative(), g.conn);
''', '''        let (seed, creative, conn) = (gs.menu.data[3], g.creative(), g.conn);
'''),
    (GAME + 'lan_stations.rs', '''        host.hub.guests[gi].xp_seed = self.world.random.next_int();
        let env = self.menu_env();
''', '''        host.hub.guests[gi].xp_seed = self.world.random.next_int();
        gs.menu.data[3] = host.hub.guests[gi].xp_seed;
        let env = self.menu_env();
'''),

    # ---- `RecipeChoice.ExactChoice`: an ingredient of stacks, for a plugin's recipe.
    (CLIENT + 'crafting.rs', '''pub struct Ingredient(pub HashSet<String>, pub Vec<String>);
''', '''pub struct Ingredient(pub HashSet<String>, pub Vec<String>, pub Option<Arc<Vec<ItemStack>>>);
'''),
    (CLIENT + 'crafting.rs', '''        Self(items.iter().cloned().collect(), items)
''', '''        Self(items.iter().cloned().collect(), items, None)
'''),
    (CLIENT + 'crafting.rs', '''        let kept = match kind {
            // `CompoundIngredient`: any of its children matching the stack
''', '''        // A plugin's `ExactChoice`: the stacks, by item and components.
        if kind == "horizon:exact" {
            let stacks: Vec<ItemStack> = o.get("stacks")?.as_array()?.iter().filter_map(recipe_result).collect();
            let mut names: Vec<String> = Vec::new();
            for stack in &stacks {
                if !names.iter().any(|name| **name == *stack.item) {
                    names.push(stack.item.to_string());
                }
            }
            let mut made = Self::of(names);
            made.2 = Some(Arc::new(stacks));
            return (!made.1.is_empty()).then_some(made);
        }
        let kept = match kind {
            // `CompoundIngredient`: any of its children matching the stack
'''),
    (CLIENT + 'crafting.rs', '''        self.0.contains(&*stack.item)
    }
}
/// `PlacementInfo`: the ingredients to find and, per recipe slot, the index
''', '''        self.0.contains(&*stack.item) && self.2.as_ref().is_none_or(|exact| exact.iter().any(|known| known.same_item_same_components(stack)))
    }
}
/// `PlacementInfo`: the ingredients to find and, per recipe slot, the index
'''),
    (CLIENT + 'crafting.rs', '''        let i = *index.get(kind)?.get(&*input.item)?;
        self.cooking.get(i)
    }
''', '''        let i = *index.get(kind)?.get(&*input.item)?;
        let found = self.cooking.get(i)?;
        if found.ingredient.test(input) {
            return Some(found);
        }
        // The first recipe for the item wants other components: an `ExactChoice`'s.
        self.cooking.iter().find(|c| c.kind == kind && c.ingredient.test(input))
    }
'''),

    # ---- `BlockState.getCollisionShape(level, pos)` of any state, not only the one standing there.
    (CLIENT + 'mobs/living.rs', '''    /// `Level.findSupportingBlock`: of the blocks whose collision shape
''', '''    /// `state.getCollisionShape(level, pos)`, with the world around `pos`, whatever stands there.
    pub fn state_collision_boxes(&self, x: i32, y: i32, z: i32, state: horizon_world::registry::BlockStateId) -> Vec<Aabb> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        let at = horizon_core::position::BlockPos::new(x, y, z);
        let context = horizon_world::collision_context::CollisionContext::empty(false);
        let offset = Vec3::new(f64::from(x), f64::from(y), f64::from(z));
        self.collision
            .collision_shape_with_context(&mut scene, state, at, &context)
            .map(|shape| shape.boxes().iter().map(|b| b.move_by(offset)).collect())
            .unwrap_or_default()
    }
    /// `Level.findSupportingBlock`: of the blocks whose collision shape
'''),

    # ---- `Block.getDrops` with no entity: `LootContextParams.THIS_ENTITY` absent.
    (ITEMS + 'items.rs', '''        self.block_drops_exploded(registry, state, held, dynamic, random, None, None, true)
''', '''        let breaker = !LOOT_WITHOUT_ENTITY.load(std::sync::atomic::Ordering::Relaxed);
        self.block_drops_exploded(registry, state, held, dynamic, random, None, None, breaker)
'''),
    (ITEMS + 'items.rs', '''struct LootContext<''', '''/// Set around a plugin's `Block.getDrops` with no entity.
pub static LOOT_WITHOUT_ENTITY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

struct LootContext<'''),

    # ---- What Paper keeps on a block entity and the game doesn't: it goes when the block entity goes, and comes from a placed item.
    (CLIENT + 'game.rs', '''        self.remove_entity(pos, old, side_effects)
    }
    fn load_block_entity(
''', '''        crate::game::paper_fam_items::block_gone([pos.x, pos.y, pos.z]);
        self.remove_entity(pos, old, side_effects)
    }
    fn load_block_entity(
'''),
    (CLIENT + 'game.rs', '''        let result = f(self.res.behaviours(), &mut level, &mut self.block_ticks);
        self.flush_spill();
''', '''        let result = f(self.res.behaviours(), &mut level, &mut self.block_ticks);
        crate::game::paper_fam_items::take_gone(self);
        self.flush_spill();
'''),
    (GAME + 'block_items.rs', '''        self.pending.banner.remove(&pos);
        let Some(saved) = self.saved_block_entity(key) else {
''', '''        self.pending.banner.remove(&pos);
        super::paper_fam_items::placed_extras(self, pos, stack);
        let Some(saved) = self.saved_block_entity(key) else {
'''),

    # ---- `ItemStack.computeTooltipLines`: what the game shows, for any tooltip flags.
    (GAME + 'stations.rs', '''    fn stack_tooltip_lines(
        &self,
''', '''    pub(super) fn stack_tooltip_lines(
        &self,
'''),
    (GAME + 'stations.rs', '''        let advanced = !nested && self.options_advanced_tooltips();
''', '''        let advanced = !nested && (super::paper_fam_items::TOOLTIP_ADVANCED.load(std::sync::atomic::Ordering::Relaxed) || self.options_advanced_tooltips());
'''),
]
