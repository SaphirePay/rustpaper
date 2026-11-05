//! Recipes, metadata, persistent data on what carries it, ban lists, profiles and the smaller classes.
use crate::entity::Entity;
use crate::persistence::NamespacedKey;
use crate::player::PlayerProfile;
use crate::server::server_folder;
use crate::{Block, ItemStack, Material, Player, World};
use horizon_mod::{Value, run_command};
use serde_json::{Map, json};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

// ---- Recipes: a data pack of the plugin's, as a server's own recipes are ----

/// `org.bukkit.inventory.ShapedRecipe`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapedRecipe {
    pub(crate) key: NamespacedKey,
    pub(crate) result: ItemStack,
    shape: Vec<String>,
    ingredients: BTreeMap<char, String>,
    group: String,
    pub(crate) extra: Map<String, Value>,
}

impl ShapedRecipe {
    /// bukkit: org/bukkit/inventory/ShapedRecipe.<init>
    pub fn new(key: NamespacedKey, result: ItemStack) -> Self {
        Self { key, result, shape: Vec::new(), ingredients: BTreeMap::new(), group: String::new(), extra: Map::new() }
    }
    /// bukkit: org/bukkit/inventory/ShapedRecipe.shape
    pub fn shape(mut self, rows: &[&str]) -> Self {
        self.shape = rows.iter().map(|row| (*row).to_owned()).collect();
        self
    }
    /// bukkit: org/bukkit/inventory/ShapedRecipe.setIngredient
    pub fn set_ingredient(mut self, key: char, material: &Material) -> Self {
        self.ingredients.insert(key, material.key().to_owned());
        self
    }
    /// bukkit: org/bukkit/inventory/ShapedRecipe.getShape
    pub fn get_shape(&self) -> &[String] {
        &self.shape
    }
    /// bukkit: org/bukkit/inventory/ShapedRecipe.getIngredientMap, org/bukkit/inventory/ShapedRecipe.getChoiceMap
    pub fn get_ingredient_map(&self) -> &BTreeMap<char, String> {
        &self.ingredients
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.getGroup
    pub fn get_group(&self) -> &str {
        &self.group
    }
    /// bukkit: org/bukkit/inventory/CraftingRecipe.setGroup
    pub fn set_group(&mut self, group: &str) {
        group.clone_into(&mut self.group);
    }
    fn saved(&self) -> Value {
        let key: Map<String, Value> = self.ingredients.iter().map(|(letter, item)| (letter.to_string(), json!(item))).collect();
        json!({ "type": "minecraft:crafting_shaped", "group": self.group, "pattern": self.shape, "key": key, "result": result_json(&self.result) })
    }
}

/// `org.bukkit.inventory.ShapelessRecipe`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapelessRecipe {
    pub(crate) key: NamespacedKey,
    pub(crate) result: ItemStack,
    ingredients: Vec<String>,
    pub(crate) extra: Map<String, Value>,
}

impl ShapelessRecipe {
    /// bukkit: org/bukkit/inventory/ShapelessRecipe.<init>
    pub fn new(key: NamespacedKey, result: ItemStack) -> Self {
        Self { key, result, ingredients: Vec::new(), extra: Map::new() }
    }
    /// bukkit: org/bukkit/inventory/ShapelessRecipe.addIngredient
    pub fn add_ingredient(mut self, material: &Material) -> Self {
        self.ingredients.push(material.key().to_owned());
        self
    }
    /// bukkit: org/bukkit/inventory/ShapelessRecipe.addIngredient
    ///
    /// The form with a count: that many of the material.
    pub fn add_ingredients(mut self, count: i32, material: &Material) -> Self {
        for _ in 0..count.max(0) {
            self.ingredients.push(material.key().to_owned());
        }
        self
    }
    /// bukkit: org/bukkit/inventory/ShapelessRecipe.removeIngredient
    pub fn remove_ingredient(mut self, material: &Material) -> Self {
        if let Some(index) = self.ingredients.iter().position(|item| item == material.key()) {
            self.ingredients.remove(index);
        }
        self
    }
    /// bukkit: org/bukkit/inventory/ShapelessRecipe.getIngredientList, org/bukkit/inventory/ShapelessRecipe.getChoiceList
    pub fn get_ingredient_list(&self) -> &[String] {
        &self.ingredients
    }
    fn saved(&self) -> Value {
        json!({ "type": "minecraft:crafting_shapeless", "ingredients": self.ingredients, "result": result_json(&self.result) })
    }
}

/// `org.bukkit.inventory.FurnaceRecipe` and its sisters: `kind` is the game's recipe type.
#[derive(Clone, Debug, PartialEq)]
pub struct CookingRecipe {
    key: NamespacedKey,
    kind: &'static str,
    result: ItemStack,
    input: Material,
    experience: f32,
    cooking_time: i32,
    pub(crate) extra: Map<String, Value>,
}

impl CookingRecipe {
    /// bukkit: org/bukkit/inventory/FurnaceRecipe.<init>, org/bukkit/inventory/BlastingRecipe.<init>, org/bukkit/inventory/SmokingRecipe.<init>, org/bukkit/inventory/CampfireRecipe.<init>
    pub fn new(key: NamespacedKey, kind: &'static str, result: ItemStack, input: Material, experience: f32, cooking_time: i32) -> Self {
        Self { key, kind, result, input, experience, cooking_time, extra: Map::new() }
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.getInput
    pub fn get_input(&self) -> &Material {
        &self.input
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.setInput
    pub fn set_input(&mut self, input: Material) {
        self.input = input;
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.getResult
    pub fn get_result(&self) -> &ItemStack {
        &self.result
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.getExperience
    pub fn get_experience(&self) -> f32 {
        self.experience
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.setExperience
    pub fn set_experience(&mut self, experience: f32) {
        self.experience = experience;
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.getCookingTime
    pub fn get_cooking_time(&self) -> i32 {
        self.cooking_time
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.setCookingTime
    pub fn set_cooking_time(&mut self, ticks: i32) {
        self.cooking_time = ticks;
    }
    /// bukkit: org/bukkit/inventory/CookingRecipe.getKey
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
    fn saved(&self) -> Value {
        json!({ "type": self.kind, "ingredient": self.input.key(), "result": result_json(&self.result), "experience": self.experience, "cookingtime": self.cooking_time })
    }
}

/// `org.bukkit.inventory.StonecuttingRecipe`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StonecuttingRecipe {
    key: NamespacedKey,
    result: ItemStack,
    pub(crate) input: Material,
    pub(crate) extra: Map<String, Value>,
}

impl StonecuttingRecipe {
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.<init>
    pub fn new(key: NamespacedKey, result: ItemStack, input: Material) -> Self {
        Self { key, result, input, extra: Map::new() }
    }
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.getInput
    pub fn get_input(&self) -> &Material {
        &self.input
    }
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.getResult
    pub fn get_result(&self) -> &ItemStack {
        &self.result
    }
    /// bukkit: org/bukkit/inventory/StonecuttingRecipe.getKey
    pub fn get_key(&self) -> &NamespacedKey {
        &self.key
    }
    fn saved(&self) -> Value {
        json!({ "type": "minecraft:stonecutting", "ingredient": self.input.key(), "result": result_json(&self.result) })
    }
}

fn result_json(stack: &ItemStack) -> Value {
    let components: Map<String, Value> = stack.all_components().map(|(id, value)| (id.clone(), value.clone())).collect();
    json!({ "id": stack.get_type().key(), "count": stack.get_amount(), "components": components })
}

/// `org.bukkit.inventory.Recipe`.
pub enum Recipe {
    Shaped(ShapedRecipe),
    Shapeless(ShapelessRecipe),
    Cooking(CookingRecipe),
    Stonecutting(StonecuttingRecipe),
}

impl Recipe {
    fn key(&self) -> &NamespacedKey {
        match self {
            Self::Shaped(recipe) => &recipe.key,
            Self::Shapeless(recipe) => &recipe.key,
            Self::Cooking(recipe) => &recipe.key,
            Self::Stonecutting(recipe) => &recipe.key,
        }
    }
    fn saved(&self) -> Value {
        let (mut saved, extra) = match self {
            Self::Shaped(recipe) => (recipe.saved(), &recipe.extra),
            Self::Shapeless(recipe) => (recipe.saved(), &recipe.extra),
            Self::Cooking(recipe) => (recipe.saved(), &recipe.extra),
            Self::Stonecutting(recipe) => (recipe.saved(), &recipe.extra),
        };
        for (field, value) in extra {
            saved[field] = value.clone();
        }
        saved
    }
    /// bukkit: org/bukkit/inventory/Recipe.getResult
    pub fn get_result(&self) -> &ItemStack {
        match self {
            Self::Shaped(recipe) => &recipe.result,
            Self::Shapeless(recipe) => &recipe.result,
            Self::Cooking(recipe) => &recipe.result,
            Self::Stonecutting(recipe) => &recipe.result,
        }
    }
}

fn recipe_folder(key: &NamespacedKey) -> PathBuf {
    crate::fam_core::world_folder().join("datapacks/plugin-recipes/data").join(key.get_namespace()).join("recipe")
}

fn write_pack_meta() {
    let root = crate::fam_core::world_folder().join("datapacks/plugin-recipes");
    let _ = std::fs::create_dir_all(&root);
    let _ = std::fs::write(root.join("pack.mcmeta"), json!({ "pack": { "pack_format": 94, "description": "Recipes added by plugins" } }).to_string());
}

/// bukkit: org/bukkit/Bukkit.addRecipe, org/bukkit/Server.addRecipe, org/bukkit/Bukkit.addRecipes, org/bukkit/Server.addRecipes
///
/// Into a data pack, and the game reloads it at the end of the tick.
pub fn add_recipe(recipe: &Recipe) -> bool {
    write_pack_meta();
    let folder = recipe_folder(recipe.key());
    let _ = std::fs::create_dir_all(&folder);
    let written = std::fs::write(folder.join(format!("{}.json", recipe.key().get_key())), recipe.saved().to_string()).is_ok();
    if written {
        run_command("datapack enable \"file/plugin-recipes\"");
        run_command("reload");
    }
    written
}

/// bukkit: org/bukkit/Bukkit.removeRecipe, org/bukkit/Server.removeRecipe
pub fn remove_recipe(key: &NamespacedKey) -> bool {
    let removed = std::fs::remove_file(recipe_folder(key).join(format!("{}.json", key.get_key()))).is_ok();
    if removed {
        run_command("reload");
    }
    removed
}

/// bukkit: org/bukkit/Bukkit.hasRecipe, org/bukkit/Server.hasRecipe
pub fn has_recipe(key: &NamespacedKey) -> bool {
    recipe_folder(key).join(format!("{}.json", key.get_key())).is_file()
}

// ---- Metadata ----

/// `org.bukkit.metadata.MetadataValue`: `FixedMetadataValue` with the owner's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataValue {
    pub value: Value,
    pub plugin: String,
}

impl MetadataValue {
    /// bukkit: org/bukkit/metadata/FixedMetadataValue.<init>
    pub fn new(plugin: &crate::Plugin, value: Value) -> Self {
        Self { value, plugin: plugin.get_name().to_owned() }
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.value, org/bukkit/metadata/FixedMetadataValue.value
    pub fn value(&self) -> &Value {
        &self.value
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.asInt
    pub fn as_int(&self) -> i32 {
        self.value.as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.asLong
    pub fn as_long(&self) -> i64 {
        self.value.as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.asDouble
    pub fn as_double(&self) -> f64 {
        self.value.as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.asFloat
    pub fn as_float(&self) -> f32 {
        self.as_double() as f32
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.asShort
    pub fn as_short(&self) -> i16 {
        self.as_long() as i16
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.asByte
    pub fn as_byte(&self) -> i8 {
        self.as_long() as i8
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.asBoolean
    pub fn as_boolean(&self) -> bool {
        self.value.as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.asString
    pub fn as_string(&self) -> String {
        self.value.as_str().map_or_else(|| self.value.to_string(), str::to_owned)
    }
    /// bukkit: org/bukkit/metadata/MetadataValue.getOwningPlugin
    pub fn get_owning_plugin(&self) -> &str {
        &self.plugin
    }
}

static METADATA: Mutex<BTreeMap<(String, String), Vec<MetadataValue>>> = Mutex::new(BTreeMap::new());

fn metadata() -> std::sync::MutexGuard<'static, BTreeMap<(String, String), Vec<MetadataValue>>> {
    METADATA.lock().unwrap_or_else(|e| e.into_inner())
}

/// bukkit: org/bukkit/metadata/Metadatable.setMetadata
///
/// What Bukkit keeps on a player, entity, block or world: `holder` is its id.
pub fn set_metadata(holder: &str, key: &str, value: MetadataValue) {
    let mut all = metadata();
    let values = all.entry((holder.to_owned(), key.to_owned())).or_default();
    values.retain(|known| known.plugin != value.plugin);
    values.push(value);
}

/// bukkit: org/bukkit/metadata/Metadatable.getMetadata
pub fn get_metadata(holder: &str, key: &str) -> Vec<MetadataValue> {
    metadata().get(&(holder.to_owned(), key.to_owned())).cloned().unwrap_or_default()
}

/// bukkit: org/bukkit/metadata/Metadatable.hasMetadata
pub fn has_metadata(holder: &str, key: &str) -> bool {
    metadata().get(&(holder.to_owned(), key.to_owned())).is_some_and(|values| !values.is_empty())
}

/// bukkit: org/bukkit/metadata/Metadatable.removeMetadata
pub fn remove_metadata(holder: &str, key: &str, plugin: &crate::Plugin) {
    if let Some(values) = metadata().get_mut(&(holder.to_owned(), key.to_owned())) {
        values.retain(|known| known.plugin != plugin.get_name());
    }
}

// ---- Persistent data on what can hold it ----

const ITEM_ROOT: &str = "PublicBukkitValues";

impl ItemStack {
    fn pdc_root(&self) -> Map<String, Value> {
        self.component_at("minecraft:custom_data", ITEM_ROOT).and_then(Value::as_object).cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.get, org/bukkit/persistence/PersistentDataHolder.getPersistentDataContainer
    pub fn pdc_get(&self, key: &NamespacedKey) -> Option<Value> {
        self.pdc_root().get(key.as_str()).cloned()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.has, org/bukkit/persistence/PersistentDataContainerView.has
    pub fn pdc_has(&self, key: &NamespacedKey) -> bool {
        self.pdc_get(key).is_some()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.set
    pub fn pdc_set(&mut self, key: &NamespacedKey, value: Value) {
        let mut root = self.pdc_root();
        root.insert(key.as_str().to_owned(), value);
        self.set_component_at("minecraft:custom_data", ITEM_ROOT, Value::Object(root));
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.remove
    pub fn pdc_remove(&mut self, key: &NamespacedKey) {
        let mut root = self.pdc_root();
        root.remove(key.as_str());
        self.set_component_at("minecraft:custom_data", ITEM_ROOT, Value::Object(root));
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainerView.getKeys
    pub fn pdc_keys(&self) -> Vec<String> {
        self.pdc_root().keys().cloned().collect()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainerView.isEmpty
    pub fn pdc_is_empty(&self) -> bool {
        self.pdc_root().is_empty()
    }
}

impl Entity {
    fn pdc_root(&self) -> Map<String, Value> {
        self.data_at("BukkitValues").as_object().cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.get
    pub fn pdc_get(&self, key: &NamespacedKey) -> Option<Value> {
        self.pdc_root().get(key.as_str()).cloned()
    }
    /// bukkit: org/bukkit/persistence/PersistentDataContainer.set
    pub fn pdc_set(&self, key: &NamespacedKey, value: &Value) -> bool {
        self.merge(&format!("{{BukkitValues:{{{}:{}}}}}", crate::data::quoted(key.as_str()), crate::data::snbt(value)))
    }
}

// ---- Bans ----

/// `org.bukkit.BanList`: of names or addresses, over the server's own files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BanList {
    pub(crate) addresses: bool,
}

/// bukkit: org/bukkit/Bukkit.getBanList, org/bukkit/Server.getBanList
pub fn get_ban_list(kind: &str) -> BanList {
    BanList { addresses: kind == "IP" }
}

impl BanList {
    fn entries(&self) -> Vec<Value> {
        let file = if self.addresses { "banned-ips.json" } else { "banned-players.json" };
        serde_json::from_str::<Value>(&std::fs::read_to_string(server_folder().join(file)).unwrap_or_default()).ok().and_then(|value| value.as_array().cloned()).unwrap_or_default()
    }
    /// bukkit: org/bukkit/BanList.isBanned
    pub fn is_banned(&self, target: &str) -> bool {
        let key = if self.addresses { "ip" } else { "name" };
        self.entries().iter().any(|entry| entry[key].as_str().is_some_and(|known| known.eq_ignore_ascii_case(target)))
    }
    /// bukkit: org/bukkit/BanList.addBan
    pub fn add_ban(&self, target: &str, reason: &str) {
        run_command(&format!("{} {target} {reason}", if self.addresses { "ban-ip" } else { "ban" }));
    }
    /// bukkit: org/bukkit/BanList.pardon
    pub fn pardon(&self, target: &str) {
        run_command(&format!("{} {target}", if self.addresses { "pardon-ip" } else { "pardon" }));
    }
    /// bukkit: org/bukkit/BanList.getEntries
    pub fn get_entries(&self) -> Vec<Value> {
        self.entries()
    }
}

// ---- Profiles ----

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let word = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        for index in 0..4 {
            if index <= chunk.len() {
                out.push(char::from(BASE64[((word >> (18 - index * 6)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn unbase64(text: &str) -> Vec<u8> {
    let digits: Vec<u32> = text.bytes().filter_map(|byte| BASE64.iter().position(|known| *known == byte).map(|at| at as u32)).collect();
    let mut out = Vec::new();
    for chunk in digits.chunks(4) {
        let word = chunk.iter().enumerate().fold(0, |all, (index, digit)| all | (digit << (18 - index * 6)));
        out.extend_from_slice(&word.to_be_bytes()[1..chunk.len()]);
    }
    out
}

/// bukkit: org/bukkit/Bukkit.createPlayerProfile, org/bukkit/Server.createPlayerProfile, org/bukkit/Bukkit.createProfile, org/bukkit/Server.createProfile, org/bukkit/Bukkit.createProfileExact, org/bukkit/Server.createProfileExact
pub fn create_player_profile(id: &str, name: &str) -> PlayerProfile {
    PlayerProfile { name: name.to_owned(), id: id.to_owned(), properties: Vec::new() }
}

/// bukkit: org/bukkit/Bukkit.createProfile, org/bukkit/Server.createProfile, org/bukkit/Bukkit.createPlayerProfile, org/bukkit/Server.createPlayerProfile
///
/// One argument, as Paper's `createProfile(UUID)` or `createProfile(String name)`: an id by its shape, else a
/// name, and the other part left unknown.
pub fn create_profile_of(id_or_name: &str) -> PlayerProfile {
    let is_id = id_or_name.len() == 36 && id_or_name.chars().enumerate().all(|(at, c)| if matches!(at, 8 | 13 | 18 | 23) { c == '-' } else { c.is_ascii_hexdigit() });
    if is_id { create_player_profile(id_or_name, "") } else { create_player_profile("", id_or_name) }
}

impl PlayerProfile {
    pub(crate) fn textures(&self) -> Value {
        self.properties.iter().find(|(name, ..)| name == "textures").and_then(|(_, value, _)| serde_json::from_slice(&unbase64(value)).ok()).unwrap_or_default()
    }
    /// bukkit: org/bukkit/profile/PlayerTextures.getSkin
    pub fn get_skin(&self) -> Option<String> {
        self.textures()["textures"]["SKIN"]["url"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/profile/PlayerTextures.getCape
    pub fn get_cape(&self) -> Option<String> {
        self.textures()["textures"]["CAPE"]["url"].as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/profile/PlayerTextures.isEmpty
    pub fn is_empty(&self) -> bool {
        self.get_skin().is_none() && self.get_cape().is_none()
    }
    /// bukkit: org/bukkit/profile/PlayerTextures.setSkin
    pub fn set_skin(&mut self, url: &str) {
        let textures = json!({ "timestamp": 0, "profileId": self.id.replace('-', ""), "profileName": self.name, "textures": { "SKIN": { "url": url } } });
        self.properties.retain(|(name, ..)| name != "textures");
        self.properties.push(("textures".to_owned(), base64(textures.to_string().as_bytes()), None));
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.setProperty
    pub fn set_property(&mut self, name: &str, value: &str, signature: Option<&str>) {
        self.properties.retain(|(known, ..)| known != name);
        self.properties.push((name.to_owned(), value.to_owned(), signature.map(str::to_owned)));
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.removeProperty
    pub fn remove_property(&mut self, name: &str) -> bool {
        let before = self.properties.len();
        self.properties.retain(|(known, ..)| known != name);
        before != self.properties.len()
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.hasProperty
    pub fn has_property(&self, name: &str) -> bool {
        self.properties.iter().any(|(known, ..)| known == name)
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.clearProperties
    pub fn clear_properties(&mut self) {
        self.properties.clear();
    }
}

// ---- Small things ----

/// bukkit: org/bukkit/util/StringUtil.startsWithIgnoreCase
pub fn starts_with_ignore_case(text: &str, prefix: &str) -> bool {
    text.len() >= prefix.len() && text.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
}

/// bukkit: org/bukkit/util/StringUtil.copyPartialMatches
pub fn copy_partial_matches(token: &str, originals: &[&str]) -> Vec<String> {
    originals.iter().filter(|text| starts_with_ignore_case(text, token)).map(|text| (*text).to_owned()).collect()
}

/// bukkit: org/bukkit/inventory/ItemFactory.createItemStack
///
/// From what commands take: `minecraft:stone[minecraft:damage=3]` without its components.
pub fn create_item_stack(text: &str) -> ItemStack {
    ItemStack::new(Material::of(text.split('[').next().unwrap_or(text)), 1)
}

/// bukkit: org/bukkit/inventory/ItemFactory.getSpawnEgg
pub fn get_spawn_egg(entity: &str) -> Material {
    Material::of(&format!("{}_spawn_egg", entity.trim_start_matches("minecraft:").to_ascii_lowercase()))
}

/// bukkit: org/bukkit/inventory/ItemFactory.getDefaultLeatherColor
pub fn get_default_leather_color() -> crate::Color {
    crate::Color::from_rgb(160, 101, 64)
}

/// bukkit: org/bukkit/inventory/ItemFactory.getI18NDisplayName
pub fn translation_key_of(stack: &ItemStack) -> String {
    format!("item.minecraft.{}", stack.get_type().key().trim_start_matches("minecraft:"))
}

impl World {
    /// bukkit: org/bukkit/World.getWorldFolder, org/bukkit/World.getWorldPath
    pub fn get_world_folder(&self) -> PathBuf {
        let (space, path) = self.dimension().split_once(':').unwrap_or(("minecraft", self.dimension()));
        crate::fam_core::world_folder().join("dimensions").join(space).join(path)
    }
    /// bukkit: org/bukkit/World.playEffect
    pub fn play_effect(&self, player: &Player, at: &crate::Location, effect: i32, data: i32) {
        player.play_effect(at, effect, data);
    }
    /// bukkit: org/bukkit/World.getChunkAtAsync, org/bukkit/World.getChunkAtAsyncUrgently
    ///
    /// Chunks are read on the server thread here, so the callback is called at once.
    pub fn get_chunk_at_async(&self, x: i32, z: i32, callback: impl FnOnce(crate::entity::Chunk)) {
        callback(self.get_chunk_at(x, z));
    }
}

impl Block {
    /// bukkit: org/bukkit/block/Block.getBlockKeyX
    pub fn get_block_key_x(key: i64) -> i32 {
        ((key << 37) >> 37) as i32
    }
    /// bukkit: org/bukkit/block/Block.getBlockKeyZ
    pub fn get_block_key_z(key: i64) -> i32 {
        ((key << 10) >> 37) as i32
    }
    /// bukkit: org/bukkit/block/Block.getBlockKeyY
    pub fn get_block_key_y(key: i64) -> i32 {
        (key >> 54) as i32
    }
}

impl ItemStack {
    fn modifiers(&self) -> Vec<Value> {
        self.get_component("minecraft:attribute_modifiers").map(|value| value.get("modifiers").unwrap_or(value)).and_then(Value::as_array).cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.hasAttributeModifiers
    pub fn has_attribute_modifiers(&self) -> bool {
        !self.modifiers().is_empty()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.getAttributeModifiers
    pub fn get_attribute_modifiers(&self) -> Vec<(String, crate::scoreboard::AttributeModifier)> {
        self.modifiers().iter().filter_map(|entry| Some((entry["type"].as_str()?.to_owned(), crate::fam_core::slotted(entry["slot"].as_str(), crate::scoreboard::AttributeModifier::new(entry["id"].as_str()?, entry["amount"].as_f64()?, entry["operation"].as_str()?))))).collect()
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.addAttributeModifier
    ///
    /// Java's form: the slot is the modifier's own.
    pub fn add_attribute_modifier(&mut self, attribute: &str, modifier: &crate::scoreboard::AttributeModifier) -> bool {
        let slot = modifier.slot.clone();
        self.add_attribute_modifier_in(attribute, modifier, &slot)
    }
    /// `slot` is the game's group: `any`, `mainhand`, `armor`.
    pub fn add_attribute_modifier_in(&mut self, attribute: &str, modifier: &crate::scoreboard::AttributeModifier, slot: &str) -> bool {
        // Bukkit's names, old (`GENERIC_ATTACK_SPEED`) and new (`ATTACK_SPEED`), as the game's id.
        let attribute = if attribute.contains(':') {
            attribute.to_owned()
        } else {
            let bare = ["GENERIC_", "PLAYER_", "ZOMBIE_", "HORSE_"].iter().find_map(|p| attribute.strip_prefix(p)).unwrap_or(attribute);
            format!("minecraft:{}", bare.to_ascii_lowercase())
        };
        let attribute = attribute.as_str();
        let mut all = self.modifiers();
        all.push(json!({ "type": attribute, "id": modifier.key, "amount": modifier.amount, "operation": match modifier.operation.as_str() { "ADD_SCALAR" => "add_multiplied_base", "MULTIPLY_SCALAR_1" => "add_multiplied_total", _ => "add_value" }, "slot": slot }));
        self.set_component("minecraft:attribute_modifiers", Value::Array(all));
        true
    }
    /// bukkit: org/bukkit/inventory/meta/ItemMeta.removeAttributeModifier
    pub fn remove_attribute_modifier(&mut self, attribute: &str) -> bool {
        let before = self.modifiers();
        let kept: Vec<Value> = before.iter().filter(|entry| entry["type"] != attribute).cloned().collect();
        let removed = kept.len() != before.len();
        self.set_component("minecraft:attribute_modifiers", Value::Array(kept));
        removed
    }
}

impl crate::entities::Firework {
    /// bukkit: org/bukkit/entity/Firework.getFireworkMeta
    pub fn get_firework_meta(&self) -> Option<crate::meta::FireworkMeta> {
        self.get_item().map(|stack| stack.meta())
    }
    /// bukkit: org/bukkit/entity/Firework.setFireworkMeta
    pub fn set_firework_meta(&self, meta: &crate::meta::FireworkMeta) -> bool {
        self.set_item(&meta.0)
    }
    /// bukkit: org/bukkit/entity/Firework.detonate
    pub fn detonate(&self) -> bool {
        crate::fam_entity::firework_detonate(&self.0)
    }
}

impl crate::entities::Projectile {
    /// bukkit: org/bukkit/entity/Projectile.getShooter
    ///
    /// The player who shot it, by the owner saved on it.
    pub fn get_shooter(&self) -> Option<Player> {
        let owner = self.get_owner_unique_id()?;
        crate::get_online_players().into_iter().find(|player| player.get_unique_id() == owner)
    }
    /// bukkit: org/bukkit/entity/Projectile.setShooter
    pub fn set_shooter(&self, shooter: &Player) -> bool {
        self.merge(&format!("{{Owner:{}}}", crate::data::uuid_snbt(&shooter.get_unique_id()).unwrap_or_default()))
    }
}

/// bukkit: org/bukkit/Bukkit.getMessenger, org/bukkit/Server.getMessenger, org/bukkit/Bukkit.getItemFactory, org/bukkit/Server.getItemFactory, org/bukkit/Bukkit.getPluginCommand, org/bukkit/Server.getPluginCommand, org/bukkit/Bukkit.getOfflinePlayer, org/bukkit/Server.getOfflinePlayer
///
/// In Rust these are the functions and methods of what they'd hand back: there's no object to fetch.
pub const SERVER_OBJECTS: () = ();
