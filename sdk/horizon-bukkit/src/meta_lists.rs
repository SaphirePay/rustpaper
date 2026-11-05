//! The `ItemMeta` calls that are lists and maps, over the same components as `meta.rs`, and what ties
//! the meta classes to a stack.
use crate::meta::{
    BannerMeta, BookMeta, BundleMeta, CrossbowMeta, CustomModelDataComponent, EnchantmentStorageMeta, FireworkEffectMeta, FireworkMeta, KnowledgeBookMeta, MetaClass, PotionMeta, SuspiciousStewMeta, WritableBookMeta,
};
use crate::{Color, ItemStack, PotionEffect};
use horizon_mod::Value;
use serde_json::{Map, json};
use std::collections::BTreeMap;

/// What each `ItemFlag` hides, as the game's `tooltip_display` lists it.
fn hidden_by(flag: &str) -> &'static [&'static str] {
    match flag {
        "HIDE_ENCHANTS" => &["minecraft:enchantments"],
        "HIDE_ATTRIBUTES" => &["minecraft:attribute_modifiers"],
        "HIDE_UNBREAKABLE" => &["minecraft:unbreakable"],
        "HIDE_DESTROYS" => &["minecraft:can_break"],
        "HIDE_PLACED_ON" => &["minecraft:can_place_on"],
        "HIDE_DYE" => &["minecraft:dyed_color"],
        "HIDE_ARMOR_TRIM" => &["minecraft:trim"],
        "HIDE_STORED_ENCHANTS" => &["minecraft:stored_enchantments"],
        "HIDE_ADDITIONAL_TOOLTIP" => &["minecraft:potion_contents", "minecraft:written_book_content", "minecraft:fireworks", "minecraft:firework_explosion", "minecraft:bundle_contents", "minecraft:map_id", "minecraft:jukebox_playable", "minecraft:instrument"],
        _ => &[],
    }
}

impl ItemStack {
    /// bukkit: org/bukkit/inventory/ItemStack.getItemMeta
    ///
    /// A class of meta: `stack.meta::<LeatherArmorMeta>()`, Java's cast of `getItemMeta()`.
    pub fn meta<M: MetaClass>(&self) -> M {
        M::of(self.clone())
    }
    /// bukkit: org/bukkit/inventory/ItemStack.setItemMeta
    pub fn set_meta<M: MetaClass>(&mut self, meta: &M) {
        *self = meta.stack().clone();
    }
    /// One path in a component; an empty path is the component.
    pub fn component_at(&self, id: &str, path: &str) -> Option<&Value> {
        let value = self.get_component(id)?;
        if path.is_empty() {
            return Some(value);
        }
        let found = crate::data::at(value, path);
        (!found.is_null()).then_some(found)
    }
    /// Writes one path in a component, making what isn't there.
    pub fn set_component_at(&mut self, id: &str, path: &str, value: Value) {
        if path.is_empty() {
            self.set_component(id, value);
            return;
        }
        let mut root = self.get_component(id).cloned().unwrap_or_else(|| json!({}));
        let mut place = &mut root;
        for key in path.split('.') {
            if !place.is_object() {
                *place = json!({});
            }
            place = place.as_object_mut().expect("just made an object").entry(key).or_insert(Value::Null);
        }
        *place = value;
        self.set_component(id, root);
    }
    fn hidden(&self) -> Vec<String> {
        self.component_at("minecraft:tooltip_display", "hidden_components").and_then(Value::as_array).into_iter().flatten().filter_map(|id| id.as_str().map(str::to_owned)).collect()
    }
    fn write_hidden(&mut self, hidden: Vec<String>) {
        self.set_component_at("minecraft:tooltip_display", "hidden_components", json!(hidden));
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.addItemFlags, org/bukkit/inventory/ItemStack.addItemFlags
    pub fn add_item_flags(&mut self, flags: &[&str]) {
        let mut hidden = self.hidden();
        for id in flags.iter().flat_map(|flag| hidden_by(flag)) {
            if !hidden.iter().any(|known| known == id) {
                hidden.push((*id).to_owned());
            }
        }
        self.write_hidden(hidden);
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.removeItemFlags, org/bukkit/inventory/ItemStack.removeItemFlags
    pub fn remove_item_flags(&mut self, flags: &[&str]) {
        let mut hidden = self.hidden();
        hidden.retain(|known| !flags.iter().flat_map(|flag| hidden_by(flag)).any(|id| id == known));
        self.write_hidden(hidden);
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasItemFlag, org/bukkit/inventory/ItemStack.hasItemFlag
    pub fn has_item_flag(&self, flag: &str) -> bool {
        let hidden = self.hidden();
        let ids = hidden_by(flag);
        !ids.is_empty() && ids.iter().all(|id| hidden.iter().any(|known| known == id))
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getItemFlags, org/bukkit/inventory/ItemStack.getItemFlags
    pub fn get_item_flags(&self) -> Vec<&'static str> {
        crate::enums::item_flag::values().iter().copied().filter(|flag| self.has_item_flag(flag)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasCustomName
    pub fn has_custom_name(&self) -> bool {
        self.has_display_name()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.customName
    pub fn custom_name(&self) -> Option<crate::Component> {
        self.display_name()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getItemName
    pub fn get_item_name(&self) -> Option<String> {
        self.item_name().map(|name| name.to_legacy())
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setItemName
    pub fn set_item_name(&mut self, name: &str) {
        self.set_item_name_component(&crate::adventure::legacy(name));
    }
    /// bukkit: org/bukkit/inventory/meta/Damageable.hasDamageValue
    pub fn has_damage_value(&self) -> bool {
        self.get_component("minecraft:damage").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/Damageable.resetDamage
    pub fn reset_damage(&mut self) {
        self.remove_component("minecraft:damage");
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getCustomModelDataComponent
    pub fn get_custom_model_data_component(&self) -> CustomModelDataComponent {
        self.meta()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasCustomModelDataComponent
    pub fn has_custom_model_data_component(&self) -> bool {
        self.get_component("minecraft:custom_model_data").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.setCustomModelDataComponent
    pub fn set_custom_model_data_component(&mut self, component: &CustomModelDataComponent) {
        self.set_meta(component);
    }
}

impl CustomModelDataComponent {
    fn list(&self, key: &str) -> Vec<Value> {
        self.0.component_at("minecraft:custom_model_data", key).and_then(Value::as_array).cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent.getFloats
    pub fn get_floats(&self) -> Vec<f32> {
        self.list("floats").iter().filter_map(Value::as_f64).map(|value| value as f32).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent.setFloats
    pub fn set_floats(&mut self, values: &[f32]) {
        self.0.set_component_at("minecraft:custom_model_data", "floats", json!(values));
    }
    /// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent.getFlags
    pub fn get_flags(&self) -> Vec<bool> {
        self.list("flags").iter().map(crate::data::flag).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent.setFlags
    pub fn set_flags(&mut self, values: &[bool]) {
        self.0.set_component_at("minecraft:custom_model_data", "flags", json!(values));
    }
    /// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent.getStrings
    pub fn get_strings(&self) -> Vec<String> {
        self.list("strings").iter().filter_map(|value| value.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent.setStrings
    pub fn set_strings(&mut self, values: &[&str]) {
        self.0.set_component_at("minecraft:custom_model_data", "strings", json!(values));
    }
    /// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent.getColors
    pub fn get_colors(&self) -> Vec<Color> {
        self.list("colors").iter().filter_map(Value::as_i64).map(|rgb| Color::from_rgb(((rgb >> 16) & 255) as u8, ((rgb >> 8) & 255) as u8, (rgb & 255) as u8)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/components/CustomModelDataComponent.setColors
    pub fn set_colors(&mut self, values: &[Color]) {
        self.0.set_component_at("minecraft:custom_model_data", "colors", json!(values.iter().map(|color| color.as_rgb()).collect::<Vec<_>>()));
    }
}

fn stacks_of(stack: &ItemStack, id: &str) -> Vec<ItemStack> {
    stack.get_component(id).and_then(Value::as_array).into_iter().flatten().filter_map(ItemStack::read).collect()
}

fn stacks_value(stacks: &[ItemStack]) -> Value {
    Value::Array(
        stacks
            .iter()
            .map(|stack| {
                let components: Map<String, Value> = stack.all_components().map(|(id, value)| (id.clone(), value.clone())).collect();
                json!({ "id": stack.get_type().key(), "count": stack.get_amount(), "components": components })
            })
            .collect(),
    )
}

impl BundleMeta {
    /// bukkit: org/bukkit/inventory/meta/BundleMeta.getItems
    pub fn get_items(&self) -> Vec<ItemStack> {
        stacks_of(&self.0, "minecraft:bundle_contents")
    }
    /// bukkit: org/bukkit/inventory/meta/BundleMeta.hasItems
    pub fn has_items(&self) -> bool {
        !self.get_items().is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/BundleMeta.setItems
    pub fn set_items(&mut self, stacks: &[ItemStack]) {
        self.0.set_component("minecraft:bundle_contents", stacks_value(stacks));
    }
    /// bukkit: org/bukkit/inventory/meta/BundleMeta.addItem
    pub fn add_item(&mut self, stack: &ItemStack) {
        let mut stacks = self.get_items();
        stacks.push(stack.clone());
        self.set_items(&stacks);
    }
}

impl CrossbowMeta {
    /// bukkit: org/bukkit/inventory/meta/CrossbowMeta.getChargedProjectiles
    pub fn get_charged_projectiles(&self) -> Vec<ItemStack> {
        stacks_of(&self.0, "minecraft:charged_projectiles")
    }
    /// bukkit: org/bukkit/inventory/meta/CrossbowMeta.hasChargedProjectiles
    pub fn has_charged_projectiles(&self) -> bool {
        !self.get_charged_projectiles().is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/CrossbowMeta.setChargedProjectiles
    pub fn set_charged_projectiles(&mut self, stacks: &[ItemStack]) {
        self.0.set_component("minecraft:charged_projectiles", stacks_value(stacks));
    }
    /// bukkit: org/bukkit/inventory/meta/CrossbowMeta.addChargedProjectile
    pub fn add_charged_projectile(&mut self, stack: &ItemStack) {
        let mut stacks = self.get_charged_projectiles();
        stacks.push(stack.clone());
        self.set_charged_projectiles(&stacks);
    }
}

impl EnchantmentStorageMeta {
    /// bukkit: org/bukkit/inventory/meta/EnchantmentStorageMeta.getStoredEnchants
    pub fn get_stored_enchants(&self) -> BTreeMap<String, u32> {
        self.0.get_component("minecraft:stored_enchantments").and_then(Value::as_object).into_iter().flatten().map(|(key, level)| (key.clone(), level.as_u64().unwrap_or(0) as u32)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/EnchantmentStorageMeta.hasStoredEnchants
    pub fn has_stored_enchants(&self) -> bool {
        !self.get_stored_enchants().is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/EnchantmentStorageMeta.hasStoredEnchant
    pub fn has_stored_enchant(&self, enchantment: &str) -> bool {
        self.get_stored_enchants().contains_key(enchantment)
    }
    /// bukkit: org/bukkit/inventory/meta/EnchantmentStorageMeta.getStoredEnchantLevel
    pub fn get_stored_enchant_level(&self, enchantment: &str) -> u32 {
        self.get_stored_enchants().get(enchantment).copied().unwrap_or(0)
    }
    /// bukkit: org/bukkit/inventory/meta/EnchantmentStorageMeta.addStoredEnchant
    pub fn add_stored_enchant(&mut self, enchantment: &str, level: u32) -> bool {
        let mut levels = self.get_stored_enchants();
        let changed = levels.insert(enchantment.to_owned(), level) != Some(level);
        self.0.set_component("minecraft:stored_enchantments", json!(levels));
        changed
    }
    /// bukkit: org/bukkit/inventory/meta/EnchantmentStorageMeta.removeStoredEnchant
    pub fn remove_stored_enchant(&mut self, enchantment: &str) -> bool {
        let mut levels = self.get_stored_enchants();
        let had = levels.remove(enchantment).is_some();
        self.0.set_component("minecraft:stored_enchantments", json!(levels));
        had
    }
}

fn effects_at(value: Option<&Value>) -> Vec<PotionEffect> {
    value.and_then(Value::as_array).into_iter().flatten().filter_map(PotionEffect::from_saved).collect()
}

impl PotionMeta {
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.getCustomEffects
    pub fn get_custom_effects(&self) -> Vec<PotionEffect> {
        effects_at(self.0.component_at("minecraft:potion_contents", "custom_effects"))
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.addCustomEffect
    pub fn add_custom_effect(&mut self, effect: &PotionEffect) -> bool {
        let mut effects = self.0.component_at("minecraft:potion_contents", "custom_effects").and_then(Value::as_array).cloned().unwrap_or_default();
        effects.push(effect.to_saved());
        self.0.set_component_at("minecraft:potion_contents", "custom_effects", Value::Array(effects));
        true
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.hasCustomEffect
    pub fn has_custom_effect(&self, effect: &str) -> bool {
        self.get_custom_effects().iter().any(|known| known.is(effect))
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.clearCustomEffects
    pub fn clear_custom_effects(&mut self) -> bool {
        let had = !self.get_custom_effects().is_empty();
        self.0.set_component_at("minecraft:potion_contents", "custom_effects", json!([]));
        had
    }
    /// bukkit: org/bukkit/inventory/meta/PotionMeta.removeCustomEffect
    pub fn remove_custom_effect(&mut self, effect: &str) -> bool {
        let kept: Vec<Value> = self.get_custom_effects().iter().filter(|known| !known.is(effect)).map(PotionEffect::to_saved).collect();
        let had = kept.len() != self.get_custom_effects().len();
        self.0.set_component_at("minecraft:potion_contents", "custom_effects", Value::Array(kept));
        had
    }
}

impl SuspiciousStewMeta {
    /// bukkit: org/bukkit/inventory/meta/SuspiciousStewMeta.getCustomEffects
    pub fn get_custom_effects(&self) -> Vec<PotionEffect> {
        effects_at(self.0.get_component("minecraft:suspicious_stew_effects"))
    }
    /// bukkit: org/bukkit/inventory/meta/SuspiciousStewMeta.hasCustomEffects
    pub fn has_custom_effects(&self) -> bool {
        !self.get_custom_effects().is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/SuspiciousStewMeta.hasCustomEffect
    pub fn has_custom_effect(&self, effect: &str) -> bool {
        self.get_custom_effects().iter().any(|known| known.is(effect))
    }
    /// bukkit: org/bukkit/inventory/meta/SuspiciousStewMeta.addCustomEffect
    pub fn add_custom_effect(&mut self, effect: &PotionEffect) -> bool {
        let mut effects: Vec<Value> = self.get_custom_effects().iter().map(PotionEffect::to_saved).collect();
        effects.push(effect.to_saved());
        self.0.set_component("minecraft:suspicious_stew_effects", Value::Array(effects));
        true
    }
    /// bukkit: org/bukkit/inventory/meta/SuspiciousStewMeta.clearCustomEffects
    pub fn clear_custom_effects(&mut self) -> bool {
        self.0.remove_component("minecraft:suspicious_stew_effects").is_some()
    }
    /// bukkit: org/bukkit/inventory/meta/SuspiciousStewMeta.removeCustomEffect
    pub fn remove_custom_effect(&mut self, effect: &str) -> bool {
        let kept: Vec<Value> = self.get_custom_effects().iter().filter(|known| !known.is(effect)).map(PotionEffect::to_saved).collect();
        let had = kept.len() != self.get_custom_effects().len();
        self.0.set_component("minecraft:suspicious_stew_effects", Value::Array(kept));
        had
    }
}

impl BookMeta {
    /// bukkit: org/bukkit/inventory/meta/BookMeta.pages
    pub fn pages(&self) -> Vec<crate::Component> {
        self.0.component_at("minecraft:written_book_content", "pages").and_then(Value::as_array).into_iter().flatten().map(|page| crate::Component(page.get("raw").unwrap_or(page).clone())).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.getPages
    pub fn get_pages(&self) -> Vec<String> {
        self.pages().iter().map(crate::Component::to_legacy).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.getPage
    pub fn get_page(&self, page: usize) -> Option<String> {
        self.get_pages().get(page.checked_sub(1)?).cloned()
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.setPages
    pub fn set_pages(&mut self, pages: &[&str]) {
        let pages: Vec<Value> = pages.iter().map(|page| json!({ "raw": crate::adventure::legacy(page).0 })).collect();
        self.0.set_component_at("minecraft:written_book_content", "pages", Value::Array(pages));
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.addPage, org/bukkit/inventory/meta/BookMeta.addPages
    pub fn add_page(&mut self, page: &str) {
        let mut pages = self.get_pages();
        pages.push(page.to_owned());
        self.set_pages(&pages.iter().map(String::as_str).collect::<Vec<_>>());
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.setPage
    pub fn set_page(&mut self, page: usize, text: &str) {
        let mut pages = self.get_pages();
        if let Some(place) = page.checked_sub(1).and_then(|index| pages.get_mut(index)) {
            text.clone_into(place);
        }
        self.set_pages(&pages.iter().map(String::as_str).collect::<Vec<_>>());
    }
    /// bukkit: org/bukkit/inventory/meta/BookMeta.getPageCount
    pub fn get_page_count(&self) -> usize {
        self.pages().len()
    }
}

impl WritableBookMeta {
    /// bukkit: org/bukkit/inventory/meta/WritableBookMeta.getPages
    pub fn get_pages(&self) -> Vec<String> {
        self.0.component_at("minecraft:writable_book_content", "pages").and_then(Value::as_array).into_iter().flatten().filter_map(|page| page.get("raw").unwrap_or(page).as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/WritableBookMeta.hasPages
    pub fn has_pages(&self) -> bool {
        !self.get_pages().is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/WritableBookMeta.getPageCount
    pub fn get_page_count(&self) -> usize {
        self.get_pages().len()
    }
    /// bukkit: org/bukkit/inventory/meta/WritableBookMeta.getPage
    pub fn get_page(&self, page: usize) -> Option<String> {
        self.get_pages().get(page.checked_sub(1)?).cloned()
    }
    /// bukkit: org/bukkit/inventory/meta/WritableBookMeta.setPages
    pub fn set_pages(&mut self, pages: &[&str]) {
        self.0.set_component_at("minecraft:writable_book_content", "pages", json!(pages.iter().map(|page| json!({ "raw": page })).collect::<Vec<_>>()));
    }
    /// bukkit: org/bukkit/inventory/meta/WritableBookMeta.addPage
    pub fn add_page(&mut self, page: &str) {
        let mut pages = self.get_pages();
        pages.push(page.to_owned());
        self.set_pages(&pages.iter().map(String::as_str).collect::<Vec<_>>());
    }
    /// bukkit: org/bukkit/inventory/meta/WritableBookMeta.setPage
    pub fn set_page(&mut self, page: usize, text: &str) {
        let mut pages = self.get_pages();
        if let Some(place) = page.checked_sub(1).and_then(|index| pages.get_mut(index)) {
            text.clone_into(place);
        }
        self.set_pages(&pages.iter().map(String::as_str).collect::<Vec<_>>());
    }
}

impl KnowledgeBookMeta {
    /// bukkit: org/bukkit/inventory/meta/KnowledgeBookMeta.getRecipes
    pub fn get_recipes(&self) -> Vec<String> {
        self.0.get_component("minecraft:recipes").and_then(Value::as_array).into_iter().flatten().filter_map(|key| key.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/KnowledgeBookMeta.hasRecipes
    pub fn has_recipes(&self) -> bool {
        !self.get_recipes().is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/KnowledgeBookMeta.setRecipes
    pub fn set_recipes(&mut self, keys: &[&str]) {
        self.0.set_component("minecraft:recipes", json!(keys));
    }
    /// bukkit: org/bukkit/inventory/meta/KnowledgeBookMeta.addRecipe
    pub fn add_recipe(&mut self, key: &str) {
        let mut keys = self.get_recipes();
        keys.push(key.to_owned());
        self.0.set_component("minecraft:recipes", json!(keys));
    }
}

/// `org.bukkit.block.banner.Pattern`: a kind and a color.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub pattern: String,
    pub color: String,
}

impl BannerMeta {
    /// bukkit: org/bukkit/inventory/meta/BannerMeta.getPatterns
    pub fn get_patterns(&self) -> Vec<Pattern> {
        self.0.get_component("minecraft:banner_patterns").and_then(Value::as_array).into_iter().flatten().filter_map(|layer| Some(Pattern { pattern: layer["pattern"].as_str()?.to_owned(), color: layer["color"].as_str()?.to_owned() })).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/BannerMeta.setPatterns
    pub fn set_patterns(&mut self, patterns: &[Pattern]) {
        self.0.set_component("minecraft:banner_patterns", Value::Array(patterns.iter().map(|layer| json!({ "pattern": layer.pattern, "color": layer.color })).collect()));
    }
    /// bukkit: org/bukkit/inventory/meta/BannerMeta.addPattern
    pub fn add_pattern(&mut self, pattern: Pattern) {
        let mut patterns = self.get_patterns();
        patterns.push(pattern);
        self.set_patterns(&patterns);
    }
    /// bukkit: org/bukkit/inventory/meta/BannerMeta.getPattern
    pub fn get_pattern(&self, index: usize) -> Option<Pattern> {
        self.get_patterns().get(index).cloned()
    }
    /// bukkit: org/bukkit/inventory/meta/BannerMeta.setPattern
    pub fn set_pattern(&mut self, index: usize, pattern: Pattern) {
        let mut patterns = self.get_patterns();
        if let Some(place) = patterns.get_mut(index) {
            *place = pattern;
        }
        self.set_patterns(&patterns);
    }
    /// bukkit: org/bukkit/inventory/meta/BannerMeta.removePattern
    pub fn remove_pattern(&mut self, index: usize) -> Option<Pattern> {
        let mut patterns = self.get_patterns();
        let removed = (index < patterns.len()).then(|| patterns.remove(index));
        self.set_patterns(&patterns);
        removed
    }
    /// bukkit: org/bukkit/inventory/meta/BannerMeta.numberOfPatterns
    pub fn number_of_patterns(&self) -> usize {
        self.get_patterns().len()
    }
}

/// `org.bukkit.FireworkEffect`: as the game saves an explosion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FireworkEffect {
    saved: Value,
}

impl FireworkEffect {
    pub(crate) fn saved_value(&self) -> &Value {
        &self.saved
    }
    /// bukkit: org/bukkit/FireworkEffect.builder, org/bukkit/FireworkEffect$Builder.build
    pub fn new(shape: &str, colors: &[Color], fades: &[Color], flicker: bool, trail: bool) -> Self {
        let rgb = |colors: &[Color]| json!(colors.iter().map(|color| color.as_rgb()).collect::<Vec<_>>());
        Self { saved: json!({ "shape": shape.to_ascii_lowercase(), "colors": rgb(colors), "fade_colors": rgb(fades), "has_twinkle": flicker, "has_trail": trail }) }
    }
    /// bukkit: org/bukkit/FireworkEffect.hasFlicker
    pub fn has_flicker(&self) -> bool {
        crate::data::flag(&self.saved["has_twinkle"])
    }
    /// bukkit: org/bukkit/FireworkEffect.hasTrail
    pub fn has_trail(&self) -> bool {
        crate::data::flag(&self.saved["has_trail"])
    }
    /// bukkit: org/bukkit/FireworkEffect.getType
    pub fn get_type(&self) -> String {
        self.saved["shape"].as_str().unwrap_or("small_ball").to_ascii_uppercase()
    }
}

impl FireworkMeta {
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.getEffects
    pub fn get_effects(&self) -> Vec<FireworkEffect> {
        self.0.component_at("minecraft:fireworks", "explosions").and_then(Value::as_array).into_iter().flatten().map(|saved| FireworkEffect { saved: saved.clone() }).collect()
    }
    fn write(&mut self, effects: &[FireworkEffect]) {
        self.0.set_component_at("minecraft:fireworks", "explosions", Value::Array(effects.iter().map(|effect| effect.saved.clone()).collect()));
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.addEffect
    pub fn add_effect(&mut self, effect: &FireworkEffect) {
        let mut effects = self.get_effects();
        effects.push(effect.clone());
        self.write(&effects);
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.addEffects
    pub fn add_effects(&mut self, added: &[FireworkEffect]) {
        let mut effects = self.get_effects();
        effects.extend_from_slice(added);
        self.write(&effects);
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.getEffectsSize
    pub fn get_effects_size(&self) -> usize {
        self.get_effects().len()
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.hasEffects
    pub fn has_effects(&self) -> bool {
        !self.get_effects().is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.removeEffect
    pub fn remove_effect(&mut self, index: usize) {
        let mut effects = self.get_effects();
        if index < effects.len() {
            effects.remove(index);
        }
        self.write(&effects);
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkMeta.clearEffects
    pub fn clear_effects(&mut self) {
        self.write(&[]);
    }
}

impl FireworkEffectMeta {
    /// bukkit: org/bukkit/inventory/meta/FireworkEffectMeta.getEffect
    pub fn get_effect(&self) -> Option<FireworkEffect> {
        self.0.get_component("minecraft:firework_explosion").map(|saved| FireworkEffect { saved: saved.clone() })
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkEffectMeta.setEffect
    pub fn set_effect(&mut self, effect: &FireworkEffect) {
        self.0.set_component("minecraft:firework_explosion", effect.saved.clone());
    }
    /// bukkit: org/bukkit/inventory/meta/FireworkEffectMeta.hasEffect
    pub fn has_effect(&self) -> bool {
        self.0.get_component("minecraft:firework_explosion").is_some()
    }
}
