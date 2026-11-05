//! Written by plugin-tools/tools/sdk_registry.py: do not edit.
#![allow(missing_docs, clippy::wrong_self_convention, clippy::too_many_arguments)]
use crate::persistence::NamespacedKey;
use horizon_mod::{Value, run_command};
use serde_json::{Map, json};

/// Into the plugins' data pack, which the game reads as it starts.
fn write_entry(folder: &str, key: &NamespacedKey, entry: &Value) -> bool {
    let root = crate::fam_core::world_folder().join("datapacks/plugin-registry");
    let file = root.join("data").join(key.get_namespace()).join(folder).join(format!("{}.json", key.get_key()));
    let Some(parent) = file.parent() else {
        return false;
    };
    let _ = std::fs::create_dir_all(parent);
    let _ = std::fs::write(root.join("pack.mcmeta"), json!({ "pack": { "pack_format": 94, "description": "Registry entries of plugins" } }).to_string());
    let written = std::fs::write(file, entry.to_string()).is_ok();
    if written {
        run_command("datapack enable \"file/plugin-registry\"");
    }
    written
}

/// A key, for what wants one by its name alone. `fam_paper::RegistryKey::typed_key` knows its registry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypedKey(pub NamespacedKey);

impl TypedKey {
    pub fn create(key: NamespacedKey) -> Self {
        Self(key)
    }
    /// bukkit: io/papermc/paper/registry/TypedKey.key
    pub fn key(&self) -> &NamespacedKey {
        &self.0
    }
}

fn push(fields: &mut Map<String, Value>, key: &str, value: Value) {
    let list = fields.entry(key.to_owned()).or_insert_with(|| json!([]));
    if let Some(list) = list.as_array_mut() {
        list.push(value);
    }
}

fn nested(fields: &mut Map<String, Value>, group: &str, key: &str, value: Value) {
    let group = fields.entry(group.to_owned()).or_insert_with(|| json!({}));
    group[key] = value;
}

/// A Java constant as the file writes it.
fn lower(value: Value) -> Value {
    value.as_str().map_or(value.clone(), |name| json!(name.to_ascii_lowercase()))
}

/// The file's word as Java's constant, or `default` where it writes nothing.
fn upper(value: &Value, default: &str) -> Value {
    json!(value.as_str().map_or_else(|| default.to_owned(), str::to_ascii_uppercase))
}

/// `io.papermc.paper.registry.data.client.ClientTextureAsset`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientTextureAsset {
    identifier: NamespacedKey,
    texture_path: NamespacedKey,
}

impl ClientTextureAsset {
    /// bukkit: io/papermc/paper/registry/data/client/ClientTextureAsset.clientTextureAsset
    ///
    /// The path is the identifier's, under `textures/`.
    pub fn client_texture_asset(identifier: &NamespacedKey) -> Option<Self> {
        let path = NamespacedKey::from_string(&format!("{}:textures/{}.png", identifier.get_namespace(), identifier.get_key()))?;
        Some(Self { identifier: identifier.clone(), texture_path: path })
    }
    /// bukkit: io/papermc/paper/registry/data/client/ClientTextureAsset.clientTextureAsset
    pub fn with_path(identifier: &NamespacedKey, texture_path: &NamespacedKey) -> Self {
        Self { identifier: identifier.clone(), texture_path: texture_path.clone() }
    }
    /// bukkit: io/papermc/paper/registry/data/client/ClientTextureAsset.identifier
    pub fn identifier(&self) -> &NamespacedKey {
        &self.identifier
    }
    /// bukkit: io/papermc/paper/registry/data/client/ClientTextureAsset.texturePath
    pub fn texture_path(&self) -> &NamespacedKey {
        &self.texture_path
    }
}

/// Data files name only the identifier.
impl From<ClientTextureAsset> for Value {
    fn from(asset: ClientTextureAsset) -> Self {
        json!(asset.identifier.as_str())
    }
}

/// `io.papermc.paper.registry.data.SulfurCubeArchetypeRegistryEntry$AttributeEntry`: one row of `attribute_modifiers`.
#[derive(Clone, Debug, PartialEq)]
pub struct SulfurCubeArchetypeRegistryEntryAttributeEntry(pub Value);

impl SulfurCubeArchetypeRegistryEntryAttributeEntry {
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$AttributeEntry.of
    pub fn of(attribute: &NamespacedKey, modifier: &crate::scoreboard::AttributeModifier) -> Self {
        Self(json!({ "attribute": attribute.as_str(), "id": modifier.key, "amount": modifier.amount, "operation": modifier.operation.to_ascii_lowercase() }))
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$AttributeEntry.attribute
    pub fn attribute(&self) -> &Value {
        &self.0["attribute"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$AttributeEntry.modifier
    pub fn modifier(&self) -> crate::scoreboard::AttributeModifier {
        let row = &self.0;
        crate::scoreboard::AttributeModifier::new(row["id"].as_str().unwrap_or_default(), row["amount"].as_f64().unwrap_or(0.0), &row["operation"].as_str().unwrap_or_default().to_ascii_uppercase())
    }
}

impl From<SulfurCubeArchetypeRegistryEntryAttributeEntry> for Value {
    fn from(entry: SulfurCubeArchetypeRegistryEntryAttributeEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.BannerPatternRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct BannerPatternRegistryEntry(pub Value);

impl BannerPatternRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/BannerPatternRegistryEntry.assetId
    pub fn asset_id(&self) -> &Value {
        &self.0["asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/BannerPatternRegistryEntry.translationKey
    pub fn translation_key(&self) -> &Value {
        &self.0["translation_key"]
    }
}

impl From<BannerPatternRegistryEntry> for Value {
    fn from(entry: BannerPatternRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.BannerPatternRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BannerPatternRegistryEntryBuilder(pub Map<String, Value>);

impl BannerPatternRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/BannerPatternRegistryEntry$Builder.assetId
    pub fn asset_id(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/BannerPatternRegistryEntry$Builder.translationKey
    pub fn translation_key(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("translation_key".to_owned(), value.into());
        self
    }
    pub fn build(self) -> BannerPatternRegistryEntry {
        BannerPatternRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("banner_pattern", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.CatTypeRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct CatTypeRegistryEntry(pub Value);

impl CatTypeRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/CatTypeRegistryEntry.clientTextureAsset
    pub fn client_texture_asset(&self) -> &Value {
        &self.0["asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/CatTypeRegistryEntry.babyClientTextureAsset
    pub fn baby_client_texture_asset(&self) -> &Value {
        &self.0["baby_asset_id"]
    }
}

impl From<CatTypeRegistryEntry> for Value {
    fn from(entry: CatTypeRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.CatTypeRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CatTypeRegistryEntryBuilder(pub Map<String, Value>);

impl CatTypeRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/CatTypeRegistryEntry$Builder.clientTextureAsset
    pub fn client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/CatTypeRegistryEntry$Builder.babyClientTextureAsset
    pub fn baby_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("baby_asset_id".to_owned(), value.into());
        self
    }
    pub fn build(self) -> CatTypeRegistryEntry {
        CatTypeRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("cat_variant", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.ChickenVariantRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChickenVariantRegistryEntry(pub Value);

impl ChickenVariantRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/ChickenVariantRegistryEntry.clientTextureAsset
    pub fn client_texture_asset(&self) -> &Value {
        &self.0["asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/ChickenVariantRegistryEntry.babyClientTextureAsset
    pub fn baby_client_texture_asset(&self) -> &Value {
        &self.0["baby_asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/ChickenVariantRegistryEntry.model
    pub fn model(&self) -> Value {
        upper(&self.0["model"], "NORMAL")
    }
}

impl From<ChickenVariantRegistryEntry> for Value {
    fn from(entry: ChickenVariantRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.ChickenVariantRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChickenVariantRegistryEntryBuilder(pub Map<String, Value>);

impl ChickenVariantRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/ChickenVariantRegistryEntry$Builder.clientTextureAsset
    pub fn client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/ChickenVariantRegistryEntry$Builder.babyClientTextureAsset
    pub fn baby_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("baby_asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/ChickenVariantRegistryEntry$Builder.model
    pub fn model(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("model".to_owned(), lower(value.into()));
        self
    }
    pub fn build(self) -> ChickenVariantRegistryEntry {
        ChickenVariantRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("chicken_variant", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.ChickenVariantRegistryEntry$Model`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChickenVariantRegistryEntryModel(pub Value);

impl ChickenVariantRegistryEntryModel {
}

impl From<ChickenVariantRegistryEntryModel> for Value {
    fn from(entry: ChickenVariantRegistryEntryModel) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.CowVariantRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct CowVariantRegistryEntry(pub Value);

impl CowVariantRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/CowVariantRegistryEntry.clientTextureAsset
    pub fn client_texture_asset(&self) -> &Value {
        &self.0["asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/CowVariantRegistryEntry.babyClientTextureAsset
    pub fn baby_client_texture_asset(&self) -> &Value {
        &self.0["baby_asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/CowVariantRegistryEntry.model
    pub fn model(&self) -> Value {
        upper(&self.0["model"], "NORMAL")
    }
}

impl From<CowVariantRegistryEntry> for Value {
    fn from(entry: CowVariantRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.CowVariantRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CowVariantRegistryEntryBuilder(pub Map<String, Value>);

impl CowVariantRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/CowVariantRegistryEntry$Builder.clientTextureAsset
    pub fn client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/CowVariantRegistryEntry$Builder.babyClientTextureAsset
    pub fn baby_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("baby_asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/CowVariantRegistryEntry$Builder.model
    pub fn model(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("model".to_owned(), lower(value.into()));
        self
    }
    pub fn build(self) -> CowVariantRegistryEntry {
        CowVariantRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("cow_variant", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.CowVariantRegistryEntry$Model`.
#[derive(Clone, Debug, PartialEq)]
pub struct CowVariantRegistryEntryModel(pub Value);

impl CowVariantRegistryEntryModel {
}

impl From<CowVariantRegistryEntryModel> for Value {
    fn from(entry: CowVariantRegistryEntryModel) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.DamageTypeRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageTypeRegistryEntry(pub Value);

impl DamageTypeRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry.messageId
    pub fn message_id(&self) -> &Value {
        &self.0["message_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry.damageScaling
    pub fn damage_scaling(&self) -> Value {
        upper(&self.0["scaling"], "")
    }
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry.exhaustion
    pub fn exhaustion(&self) -> &Value {
        &self.0["exhaustion"]
    }
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry.damageEffect
    pub fn damage_effect(&self) -> Value {
        upper(&self.0["effects"], "HURT")
    }
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry.deathMessageType
    pub fn death_message_type(&self) -> Value {
        upper(&self.0["death_message_type"], "DEFAULT")
    }
}

impl From<DamageTypeRegistryEntry> for Value {
    fn from(entry: DamageTypeRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.DamageTypeRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageTypeRegistryEntryBuilder(pub Map<String, Value>);

impl DamageTypeRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry$Builder.messageId
    pub fn message_id(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("message_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry$Builder.exhaustion
    pub fn exhaustion(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("exhaustion".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry$Builder.damageScaling
    pub fn damage_scaling(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("scaling".to_owned(), lower(value.into()));
        self
    }
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry$Builder.damageEffect
    pub fn damage_effect(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("effects".to_owned(), lower(value.into()));
        self
    }
    /// bukkit: io/papermc/paper/registry/data/DamageTypeRegistryEntry$Builder.deathMessageType
    pub fn death_message_type(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("death_message_type".to_owned(), lower(value.into()));
        self
    }
    pub fn build(self) -> DamageTypeRegistryEntry {
        DamageTypeRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("damage_type", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.DecoratedPotPatternRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct DecoratedPotPatternRegistryEntry(pub Value);

impl DecoratedPotPatternRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/DecoratedPotPatternRegistryEntry.assetId
    pub fn asset_id(&self) -> &Value {
        &self.0["asset_id"]
    }
}

impl From<DecoratedPotPatternRegistryEntry> for Value {
    fn from(entry: DecoratedPotPatternRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.DecoratedPotPatternRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecoratedPotPatternRegistryEntryBuilder(pub Map<String, Value>);

impl DecoratedPotPatternRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/DecoratedPotPatternRegistryEntry$Builder.assetId
    pub fn asset_id(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    pub fn build(self) -> DecoratedPotPatternRegistryEntry {
        DecoratedPotPatternRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("decorated_pot_pattern", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.EnchantmentRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct EnchantmentRegistryEntry(pub Value);

impl EnchantmentRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.description
    pub fn description(&self) -> &Value {
        &self.0["description"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.supportedItems
    pub fn supported_items(&self) -> &Value {
        &self.0["supported_items"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.primaryItems
    pub fn primary_items(&self) -> &Value {
        &self.0["primary_items"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.weight
    pub fn weight(&self) -> &Value {
        &self.0["weight"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.maxLevel
    pub fn max_level(&self) -> &Value {
        &self.0["max_level"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.minimumCost
    pub fn minimum_cost(&self) -> &Value {
        &self.0["min_cost"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.maximumCost
    pub fn maximum_cost(&self) -> &Value {
        &self.0["max_cost"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.anvilCost
    pub fn anvil_cost(&self) -> &Value {
        &self.0["anvil_cost"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.activeSlots
    pub fn active_slots(&self) -> &Value {
        &self.0["slots"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry.exclusiveWith
    pub fn exclusive_with(&self) -> &Value {
        &self.0["exclusive_set"]
    }
}

impl From<EnchantmentRegistryEntry> for Value {
    fn from(entry: EnchantmentRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.EnchantmentRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnchantmentRegistryEntryBuilder(pub Map<String, Value>);

impl EnchantmentRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.description
    pub fn description(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("description".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.supportedItems
    pub fn supported_items(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("supported_items".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.primaryItems
    pub fn primary_items(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("primary_items".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.weight
    pub fn weight(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("weight".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.maxLevel
    pub fn max_level(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("max_level".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.minimumCost
    pub fn minimum_cost(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("min_cost".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.maximumCost
    pub fn maximum_cost(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("max_cost".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.anvilCost
    pub fn anvil_cost(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("anvil_cost".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.activeSlots
    pub fn active_slots(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("slots".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$Builder.exclusiveWith
    pub fn exclusive_with(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("exclusive_set".to_owned(), value.into());
        self
    }
    pub fn build(self) -> EnchantmentRegistryEntry {
        EnchantmentRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("enchantment", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.EnchantmentRegistryEntry$EnchantmentCost`.
#[derive(Clone, Debug, PartialEq)]
pub struct EnchantmentRegistryEntryEnchantmentCost(pub Value);

impl EnchantmentRegistryEntryEnchantmentCost {
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$EnchantmentCost.baseCost
    pub fn base_cost(&self) -> &Value {
        &self.0["base"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$EnchantmentCost.additionalPerLevelCost
    pub fn additional_per_level_cost(&self) -> &Value {
        &self.0["per_level_above_first"]
    }
    /// bukkit: io/papermc/paper/registry/data/EnchantmentRegistryEntry$EnchantmentCost.of
    pub fn of(base_cost: impl Into<Value>, additional_per_level_cost: impl Into<Value>) -> Self {
        Self(json!({ "base": base_cost.into(), "per_level_above_first": additional_per_level_cost.into() }))
    }
}

impl From<EnchantmentRegistryEntryEnchantmentCost> for Value {
    fn from(entry: EnchantmentRegistryEntryEnchantmentCost) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.FrogVariantRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct FrogVariantRegistryEntry(pub Value);

impl FrogVariantRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/FrogVariantRegistryEntry.clientTextureAsset
    pub fn client_texture_asset(&self) -> &Value {
        &self.0["asset_id"]
    }
}

impl From<FrogVariantRegistryEntry> for Value {
    fn from(entry: FrogVariantRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.FrogVariantRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrogVariantRegistryEntryBuilder(pub Map<String, Value>);

impl FrogVariantRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/FrogVariantRegistryEntry$Builder.clientTextureAsset
    pub fn client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    pub fn build(self) -> FrogVariantRegistryEntry {
        FrogVariantRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("frog_variant", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.GameEventRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct GameEventRegistryEntry(pub Value);

impl GameEventRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/GameEventRegistryEntry.range
    pub fn range(&self) -> &Value {
        &self.0["range"]
    }
}

impl From<GameEventRegistryEntry> for Value {
    fn from(entry: GameEventRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.GameEventRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameEventRegistryEntryBuilder(pub Map<String, Value>);

impl GameEventRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/GameEventRegistryEntry$Builder.range
    pub fn range(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("range".to_owned(), value.into());
        self
    }
    pub fn build(self) -> GameEventRegistryEntry {
        GameEventRegistryEntry(Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.InstrumentRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct InstrumentRegistryEntry(pub Value);

impl InstrumentRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry.soundEvent
    pub fn sound_event(&self) -> &Value {
        &self.0["sound_event"]
    }
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry.duration
    pub fn duration(&self) -> &Value {
        &self.0["use_duration"]
    }
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry.range
    pub fn range(&self) -> &Value {
        &self.0["range"]
    }
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry.durabilityDamage
    pub fn durability_damage(&self) -> Value {
        if self.0["durability_damage"].is_null() { json!(0) } else { self.0["durability_damage"].clone() }
    }
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry.description
    pub fn description(&self) -> &Value {
        &self.0["description"]
    }
}

impl From<InstrumentRegistryEntry> for Value {
    fn from(entry: InstrumentRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.InstrumentRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InstrumentRegistryEntryBuilder(pub Map<String, Value>);

impl InstrumentRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry$Builder.soundEvent
    pub fn sound_event(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("sound_event".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry$Builder.duration
    pub fn duration(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("use_duration".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry$Builder.range
    pub fn range(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("range".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry$Builder.durabilityDamage
    pub fn durability_damage(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("durability_damage".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/InstrumentRegistryEntry$Builder.description
    pub fn description(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("description".to_owned(), value.into());
        self
    }
    pub fn build(self) -> InstrumentRegistryEntry {
        InstrumentRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("instrument", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.JukeboxSongRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct JukeboxSongRegistryEntry(pub Value);

impl JukeboxSongRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/JukeboxSongRegistryEntry.soundEvent
    pub fn sound_event(&self) -> &Value {
        &self.0["sound_event"]
    }
    /// bukkit: io/papermc/paper/registry/data/JukeboxSongRegistryEntry.description
    pub fn description(&self) -> &Value {
        &self.0["description"]
    }
    /// bukkit: io/papermc/paper/registry/data/JukeboxSongRegistryEntry.lengthInSeconds
    pub fn length_in_seconds(&self) -> &Value {
        &self.0["length_in_seconds"]
    }
    /// bukkit: io/papermc/paper/registry/data/JukeboxSongRegistryEntry.comparatorOutput
    pub fn comparator_output(&self) -> &Value {
        &self.0["comparator_output"]
    }
}

impl From<JukeboxSongRegistryEntry> for Value {
    fn from(entry: JukeboxSongRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.JukeboxSongRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JukeboxSongRegistryEntryBuilder(pub Map<String, Value>);

impl JukeboxSongRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/JukeboxSongRegistryEntry$Builder.soundEvent
    pub fn sound_event(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("sound_event".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/JukeboxSongRegistryEntry$Builder.description
    pub fn description(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("description".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/JukeboxSongRegistryEntry$Builder.lengthInSeconds
    pub fn length_in_seconds(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("length_in_seconds".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/JukeboxSongRegistryEntry$Builder.comparatorOutput
    pub fn comparator_output(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("comparator_output".to_owned(), value.into());
        self
    }
    pub fn build(self) -> JukeboxSongRegistryEntry {
        JukeboxSongRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("jukebox_song", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.PaintingVariantRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct PaintingVariantRegistryEntry(pub Value);

impl PaintingVariantRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry.width
    pub fn width(&self) -> &Value {
        &self.0["width"]
    }
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry.height
    pub fn height(&self) -> &Value {
        &self.0["height"]
    }
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry.title
    pub fn title(&self) -> &Value {
        &self.0["title"]
    }
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry.author
    pub fn author(&self) -> &Value {
        &self.0["author"]
    }
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry.assetId
    pub fn asset_id(&self) -> &Value {
        &self.0["asset_id"]
    }
}

impl From<PaintingVariantRegistryEntry> for Value {
    fn from(entry: PaintingVariantRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.PaintingVariantRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintingVariantRegistryEntryBuilder(pub Map<String, Value>);

impl PaintingVariantRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry$Builder.width
    pub fn width(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("width".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry$Builder.height
    pub fn height(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("height".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry$Builder.title
    pub fn title(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("title".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry$Builder.author
    pub fn author(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("author".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/PaintingVariantRegistryEntry$Builder.assetId
    pub fn asset_id(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    pub fn build(self) -> PaintingVariantRegistryEntry {
        PaintingVariantRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("painting_variant", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.PigVariantRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct PigVariantRegistryEntry(pub Value);

impl PigVariantRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/PigVariantRegistryEntry.clientTextureAsset
    pub fn client_texture_asset(&self) -> &Value {
        &self.0["asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/PigVariantRegistryEntry.babyClientTextureAsset
    pub fn baby_client_texture_asset(&self) -> &Value {
        &self.0["baby_asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/PigVariantRegistryEntry.model
    pub fn model(&self) -> Value {
        upper(&self.0["model"], "NORMAL")
    }
}

impl From<PigVariantRegistryEntry> for Value {
    fn from(entry: PigVariantRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.PigVariantRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PigVariantRegistryEntryBuilder(pub Map<String, Value>);

impl PigVariantRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/PigVariantRegistryEntry$Builder.clientTextureAsset
    pub fn client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/PigVariantRegistryEntry$Builder.babyClientTextureAsset
    pub fn baby_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("baby_asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/PigVariantRegistryEntry$Builder.model
    pub fn model(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("model".to_owned(), lower(value.into()));
        self
    }
    pub fn build(self) -> PigVariantRegistryEntry {
        PigVariantRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("pig_variant", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.PigVariantRegistryEntry$Model`.
#[derive(Clone, Debug, PartialEq)]
pub struct PigVariantRegistryEntryModel(pub Value);

impl PigVariantRegistryEntryModel {
}

impl From<PigVariantRegistryEntryModel> for Value {
    fn from(entry: PigVariantRegistryEntryModel) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.SoundEventRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundEventRegistryEntry(pub Value);

impl SoundEventRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/SoundEventRegistryEntry.location
    pub fn location(&self) -> &Value {
        &self.0["sound_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/SoundEventRegistryEntry.fixedRange
    pub fn fixed_range(&self) -> &Value {
        &self.0["range"]
    }
}

impl From<SoundEventRegistryEntry> for Value {
    fn from(entry: SoundEventRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.SoundEventRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SoundEventRegistryEntryBuilder(pub Map<String, Value>);

impl SoundEventRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/SoundEventRegistryEntry$Builder.location
    pub fn location(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("sound_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/SoundEventRegistryEntry$Builder.fixedRange
    pub fn fixed_range(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("range".to_owned(), value.into());
        self
    }
    pub fn build(self) -> SoundEventRegistryEntry {
        SoundEventRegistryEntry(Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.SulfurCubeArchetypeRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct SulfurCubeArchetypeRegistryEntry(pub Value);

impl SulfurCubeArchetypeRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry.items
    pub fn items(&self) -> &Value {
        &self.0["items"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry.attributeModifiers
    pub fn attribute_modifiers(&self) -> &Value {
        &self.0["attribute_modifiers"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry.buoyant
    pub fn buoyant(&self) -> &Value {
        &self.0["buoyant"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry.explosion
    pub fn explosion(&self) -> &Value {
        &self.0["explosion"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry.contactDamage
    pub fn contact_damage(&self) -> &Value {
        &self.0["contact_damage"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry.knockbackModifiers
    pub fn knockback_modifiers(&self) -> &Value {
        &self.0["knockback_modifiers"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry.soundSettings
    pub fn sound_settings(&self) -> &Value {
        &self.0["sound_settings"]
    }
}

impl From<SulfurCubeArchetypeRegistryEntry> for Value {
    fn from(entry: SulfurCubeArchetypeRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.SulfurCubeArchetypeRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SulfurCubeArchetypeRegistryEntryBuilder(pub Map<String, Value>);

impl SulfurCubeArchetypeRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$Builder.items
    pub fn items(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("items".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$Builder.attributeModifiers
    pub fn attribute_modifiers(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("attribute_modifiers".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$Builder.buoyant
    pub fn buoyant(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("buoyant".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$Builder.explosion
    pub fn explosion(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("explosion".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$Builder.contactDamage
    pub fn contact_damage(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("contact_damage".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$Builder.knockbackModifiers
    pub fn knockback_modifiers(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("knockback_modifiers".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$Builder.soundSettings
    pub fn sound_settings(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("sound_settings".to_owned(), value.into());
        self
    }
    pub fn build(self) -> SulfurCubeArchetypeRegistryEntry {
        SulfurCubeArchetypeRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("sulfur_cube_archetype", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.SulfurCubeArchetypeRegistryEntry$ContactDamage`.
#[derive(Clone, Debug, PartialEq)]
pub struct SulfurCubeArchetypeRegistryEntryContactDamage(pub Value);

impl SulfurCubeArchetypeRegistryEntryContactDamage {
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$ContactDamage.damageType
    pub fn damage_type(&self) -> &Value {
        &self.0["damage_type"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$ContactDamage.amount
    pub fn amount(&self) -> &Value {
        &self.0["amount"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$ContactDamage.attributeToSource
    pub fn attribute_to_source(&self) -> &Value {
        &self.0["attribute_to_source"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$ContactDamage.of
    pub fn of(damage_type: impl Into<Value>, amount: impl Into<Value>, attribute_to_source: impl Into<Value>) -> Self {
        Self(json!({ "damage_type": damage_type.into(), "amount": amount.into(), "attribute_to_source": attribute_to_source.into() }))
    }
}

impl From<SulfurCubeArchetypeRegistryEntryContactDamage> for Value {
    fn from(entry: SulfurCubeArchetypeRegistryEntryContactDamage) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.SulfurCubeArchetypeRegistryEntry$ExplosionSettings`.
#[derive(Clone, Debug, PartialEq)]
pub struct SulfurCubeArchetypeRegistryEntryExplosionSettings(pub Value);

impl SulfurCubeArchetypeRegistryEntryExplosionSettings {
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$ExplosionSettings.power
    pub fn power(&self) -> &Value {
        &self.0["power"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$ExplosionSettings.incendiary
    pub fn incendiary(&self) -> &Value {
        &self.0["causes_fire"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$ExplosionSettings.fuseTicks
    pub fn fuse_ticks(&self) -> &Value {
        &self.0["fuse"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$ExplosionSettings.of
    pub fn of(power: impl Into<Value>, incendiary: impl Into<Value>, fuse_ticks: impl Into<Value>) -> Self {
        Self(json!({ "power": power.into(), "causes_fire": incendiary.into(), "fuse": fuse_ticks.into() }))
    }
}

impl From<SulfurCubeArchetypeRegistryEntryExplosionSettings> for Value {
    fn from(entry: SulfurCubeArchetypeRegistryEntryExplosionSettings) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.SulfurCubeArchetypeRegistryEntry$KnockbackModifiers`.
#[derive(Clone, Debug, PartialEq)]
pub struct SulfurCubeArchetypeRegistryEntryKnockbackModifiers(pub Value);

impl SulfurCubeArchetypeRegistryEntryKnockbackModifiers {
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$KnockbackModifiers.horizontalPower
    pub fn horizontal_power(&self) -> &Value {
        &self.0["horizontal_power"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$KnockbackModifiers.verticalPower
    pub fn vertical_power(&self) -> &Value {
        &self.0["vertical_power"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$KnockbackModifiers.of
    pub fn of(horizontal_power: impl Into<Value>, vertical_power: impl Into<Value>) -> Self {
        Self(json!({ "horizontal_power": horizontal_power.into(), "vertical_power": vertical_power.into() }))
    }
}

impl From<SulfurCubeArchetypeRegistryEntryKnockbackModifiers> for Value {
    fn from(entry: SulfurCubeArchetypeRegistryEntryKnockbackModifiers) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.SulfurCubeArchetypeRegistryEntry$SoundSettings`.
#[derive(Clone, Debug, PartialEq)]
pub struct SulfurCubeArchetypeRegistryEntrySoundSettings(pub Value);

impl SulfurCubeArchetypeRegistryEntrySoundSettings {
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$SoundSettings.hitSound
    pub fn hit_sound(&self) -> &Value {
        &self.0["hit_sound"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$SoundSettings.pushSound
    pub fn push_sound(&self) -> &Value {
        &self.0["push_sound"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$SoundSettings.pushSoundImpulseThreshold
    pub fn push_sound_impulse_threshold(&self) -> &Value {
        &self.0["push_sound_impulse_threshold"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$SoundSettings.pushSoundCooldown
    pub fn push_sound_cooldown(&self) -> &Value {
        &self.0["push_sound_cooldown"]
    }
    /// bukkit: io/papermc/paper/registry/data/SulfurCubeArchetypeRegistryEntry$SoundSettings.of
    pub fn of(hit_sound: impl Into<Value>, push_sound: impl Into<Value>, push_sound_impulse_threshold: impl Into<Value>, push_sound_cooldown: impl Into<Value>) -> Self {
        Self(json!({ "hit_sound": hit_sound.into(), "push_sound": push_sound.into(), "push_sound_impulse_threshold": push_sound_impulse_threshold.into(), "push_sound_cooldown": push_sound_cooldown.into() }))
    }
}

impl From<SulfurCubeArchetypeRegistryEntrySoundSettings> for Value {
    fn from(entry: SulfurCubeArchetypeRegistryEntrySoundSettings) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.TrimMaterialRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct TrimMaterialRegistryEntry(pub Value);

impl TrimMaterialRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/TrimMaterialRegistryEntry.paletteId
    pub fn palette_id(&self) -> &Value {
        &self.0["palette_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/TrimMaterialRegistryEntry.description
    pub fn description(&self) -> &Value {
        &self.0["description"]
    }
}

impl From<TrimMaterialRegistryEntry> for Value {
    fn from(entry: TrimMaterialRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.TrimMaterialRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrimMaterialRegistryEntryBuilder(pub Map<String, Value>);

impl TrimMaterialRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/TrimMaterialRegistryEntry$Builder.paletteId
    pub fn palette_id(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("palette_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/TrimMaterialRegistryEntry$Builder.description
    pub fn description(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("description".to_owned(), value.into());
        self
    }
    pub fn build(self) -> TrimMaterialRegistryEntry {
        TrimMaterialRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("trim_material", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.TrimPatternRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct TrimPatternRegistryEntry(pub Value);

impl TrimPatternRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/TrimPatternRegistryEntry.assetId
    pub fn asset_id(&self) -> &Value {
        &self.0["asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/TrimPatternRegistryEntry.description
    pub fn description(&self) -> &Value {
        &self.0["description"]
    }
    /// bukkit: io/papermc/paper/registry/data/TrimPatternRegistryEntry.decal
    pub fn decal(&self) -> &Value {
        &self.0["decal"]
    }
}

impl From<TrimPatternRegistryEntry> for Value {
    fn from(entry: TrimPatternRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.TrimPatternRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrimPatternRegistryEntryBuilder(pub Map<String, Value>);

impl TrimPatternRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/TrimPatternRegistryEntry$Builder.assetId
    pub fn asset_id(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/TrimPatternRegistryEntry$Builder.description
    pub fn description(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("description".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/TrimPatternRegistryEntry$Builder.decal
    pub fn decal(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("decal".to_owned(), value.into());
        self
    }
    pub fn build(self) -> TrimPatternRegistryEntry {
        TrimPatternRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("trim_pattern", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.WolfVariantRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct WolfVariantRegistryEntry(pub Value);

impl WolfVariantRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry.angryClientTextureAsset
    pub fn angry_client_texture_asset(&self) -> &Value {
        &self.0["assets"]["angry"]
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry.wildClientTextureAsset
    pub fn wild_client_texture_asset(&self) -> &Value {
        &self.0["assets"]["wild"]
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry.tameClientTextureAsset
    pub fn tame_client_texture_asset(&self) -> &Value {
        &self.0["assets"]["tame"]
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry.babyAngryClientTextureAsset
    pub fn baby_angry_client_texture_asset(&self) -> &Value {
        &self.0["baby_assets"]["angry"]
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry.babyWildClientTextureAsset
    pub fn baby_wild_client_texture_asset(&self) -> &Value {
        &self.0["baby_assets"]["wild"]
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry.babyTameClientTextureAsset
    pub fn baby_tame_client_texture_asset(&self) -> &Value {
        &self.0["baby_assets"]["tame"]
    }
}

impl From<WolfVariantRegistryEntry> for Value {
    fn from(entry: WolfVariantRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.WolfVariantRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WolfVariantRegistryEntryBuilder(pub Map<String, Value>);

impl WolfVariantRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry$Builder.angryClientTextureAsset
    pub fn angry_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        nested(&mut self.0, "assets", "angry", value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry$Builder.wildClientTextureAsset
    pub fn wild_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        nested(&mut self.0, "assets", "wild", value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry$Builder.tameClientTextureAsset
    pub fn tame_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        nested(&mut self.0, "assets", "tame", value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry$Builder.babyAngryClientTextureAsset
    pub fn baby_angry_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        nested(&mut self.0, "baby_assets", "angry", value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry$Builder.babyWildClientTextureAsset
    pub fn baby_wild_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        nested(&mut self.0, "baby_assets", "wild", value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/WolfVariantRegistryEntry$Builder.babyTameClientTextureAsset
    pub fn baby_tame_client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        nested(&mut self.0, "baby_assets", "tame", value.into());
        self
    }
    pub fn build(self) -> WolfVariantRegistryEntry {
        WolfVariantRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("wolf_variant", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.ZombieNautilusVariantRegistryEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct ZombieNautilusVariantRegistryEntry(pub Value);

impl ZombieNautilusVariantRegistryEntry {
    /// bukkit: io/papermc/paper/registry/data/ZombieNautilusVariantRegistryEntry.clientTextureAsset
    pub fn client_texture_asset(&self) -> &Value {
        &self.0["asset_id"]
    }
    /// bukkit: io/papermc/paper/registry/data/ZombieNautilusVariantRegistryEntry.model
    pub fn model(&self) -> Value {
        upper(&self.0["model"], "NORMAL")
    }
}

impl From<ZombieNautilusVariantRegistryEntry> for Value {
    fn from(entry: ZombieNautilusVariantRegistryEntry) -> Self {
        entry.0
    }
}

/// `io.papermc.paper.registry.data.ZombieNautilusVariantRegistryEntry$Builder`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZombieNautilusVariantRegistryEntryBuilder(pub Map<String, Value>);

impl ZombieNautilusVariantRegistryEntryBuilder {
    /// bukkit: io/papermc/paper/registry/data/ZombieNautilusVariantRegistryEntry$Builder.clientTextureAsset
    pub fn client_texture_asset(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("asset_id".to_owned(), value.into());
        self
    }
    /// bukkit: io/papermc/paper/registry/data/ZombieNautilusVariantRegistryEntry$Builder.model
    pub fn model(mut self, value: impl Into<Value>) -> Self {
        self.0.insert("model".to_owned(), lower(value.into()));
        self
    }
    pub fn build(self) -> ZombieNautilusVariantRegistryEntry {
        ZombieNautilusVariantRegistryEntry(Value::Object(self.0))
    }
    /// Into the plugins' data pack, for the next start.
    pub fn register(self, key: &NamespacedKey) -> bool {
        write_entry("zombie_nautilus_variant", key, &Value::Object(self.0))
    }
}

/// `io.papermc.paper.registry.data.ZombieNautilusVariantRegistryEntry$Model`.
#[derive(Clone, Debug, PartialEq)]
pub struct ZombieNautilusVariantRegistryEntryModel(pub Value);

impl ZombieNautilusVariantRegistryEntryModel {
}

impl From<ZombieNautilusVariantRegistryEntryModel> for Value {
    fn from(entry: ZombieNautilusVariantRegistryEntryModel) -> Self {
        entry.0
    }
}
