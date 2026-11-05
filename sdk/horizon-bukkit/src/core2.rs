//! More of Bukkit's core classes: what's plain data, what the game's data files say, and what the
//! server's own files keep.
#![allow(missing_docs)]
use crate::entity::Entity;
use crate::persistence::NamespacedKey;
use crate::server::server_folder;
use crate::{ItemStack, Location, Material, OfflinePlayer, Player, World};
use horizon_mod::{Value, query, run_command};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

// ---- Note ----

/// `org.bukkit.Note`: a note block's id, 0 to 24.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Note(pub u8);

/// `org.bukkit.Note$Tone`, as Bukkit numbers them: G is 0.
pub const TONES: [(&str, u8, bool); 7] = [("G", 0, true), ("A", 2, true), ("B", 4, false), ("C", 5, true), ("D", 7, true), ("E", 9, false), ("F", 10, true)];

impl Note {
    fn from_tone(octave: u8, tone: &str, shift: i8) -> Option<Self> {
        let (_, id, _) = TONES.iter().find(|(name, _, _)| *name == tone)?;
        let base = i16::from(*id) + i16::from(shift) + 12 * i16::from(octave);
        u8::try_from(base).ok().filter(|id| *id <= 24).map(Self)
    }
    /// bukkit: org/bukkit/Note.natural
    pub fn natural(octave: u8, tone: &str) -> Option<Self> {
        Self::from_tone(octave, tone, 0)
    }
    /// bukkit: org/bukkit/Note.sharp
    pub fn sharp(octave: u8, tone: &str) -> Option<Self> {
        Self::from_tone(octave, tone, 1)
    }
    /// bukkit: org/bukkit/Note.flat
    pub fn flat(octave: u8, tone: &str) -> Option<Self> {
        Self::from_tone(octave, tone, -1)
    }
    /// bukkit: org/bukkit/Note.sharped
    pub fn sharped(self) -> Option<Self> {
        (self.0 < 24).then_some(Self(self.0 + 1))
    }
    /// bukkit: org/bukkit/Note.flattened
    pub fn flattened(self) -> Option<Self> {
        self.0.checked_sub(1).map(Self)
    }
    /// bukkit: org/bukkit/Note.getId
    pub fn get_id(self) -> u8 {
        self.0
    }
    /// bukkit: org/bukkit/Note.getOctave
    pub fn get_octave(self) -> u8 {
        self.0 / 12
    }
    fn place(self) -> (u8, bool) {
        let within = self.0 % 12;
        match TONES.iter().find(|(_, id, _)| *id == within) {
            Some((_, id, _)) => (*id, false),
            None => (within.wrapping_sub(1) % 12, true),
        }
    }
    /// bukkit: org/bukkit/Note.getTone
    pub fn get_tone(self) -> &'static str {
        let (id, _) = self.place();
        TONES.iter().find(|(_, tone, _)| *tone == id).map_or("G", |(name, _, _)| name)
    }
    /// bukkit: org/bukkit/Note.isSharped
    pub fn is_sharped(self) -> bool {
        self.place().1
    }
    /// bukkit: org/bukkit/Note.getPitch
    pub fn get_pitch(self) -> f32 {
        2f32.powf((f32::from(self.0) - 12.0) / 12.0)
    }
}

// ---- Merchant recipes ----

/// `org.bukkit.inventory.MerchantRecipe`: an entry of a merchant's `Offers`.
#[derive(Clone, Debug, PartialEq)]
pub struct MerchantRecipe(pub Value);

impl MerchantRecipe {
    /// bukkit: org/bukkit/inventory/MerchantRecipe.<init>
    pub fn new(result: &ItemStack, max_uses: i32) -> Self {
        Self(json!({ "sell": result.to_json(), "maxUses": max_uses, "uses": 0, "rewardExp": true, "xp": 0, "priceMultiplier": 0.0, "demand": 0, "specialPrice": 0 }))
    }
    fn int(&self, key: &str) -> i64 {
        self.0[key].as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        ItemStack::read(&self.0["sell"])
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getIngredients
    pub fn get_ingredients(&self) -> Vec<ItemStack> {
        ["buy", "buyB"].iter().filter_map(|key| ItemStack::read(&self.0[*key])).collect()
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.addIngredient
    pub fn add_ingredient(&mut self, stack: &ItemStack) {
        let key = if self.0["buy"].is_null() { "buy" } else { "buyB" };
        self.0[key] = stack.to_json();
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.removeIngredient
    pub fn remove_ingredient(&mut self, index: usize) {
        if let Some(fields) = self.0.as_object_mut() {
            let second = fields.remove("buyB");
            if index == 0 {
                fields.remove("buy");
                if let Some(second) = second {
                    fields.insert("buy".to_owned(), second);
                }
            }
        }
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setIngredients
    pub fn set_ingredients(&mut self, stacks: &[ItemStack]) {
        if let Some(fields) = self.0.as_object_mut() {
            fields.remove("buy");
            fields.remove("buyB");
        }
        stacks.iter().take(2).for_each(|stack| self.add_ingredient(stack));
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getDemand
    pub fn get_demand(&self) -> i64 {
        self.int("demand")
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setDemand
    pub fn set_demand(&mut self, demand: i32) {
        self.0["demand"] = json!(demand);
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getSpecialPrice
    pub fn get_special_price(&self) -> i64 {
        self.int("specialPrice")
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setSpecialPrice
    pub fn set_special_price(&mut self, price: i32) {
        self.0["specialPrice"] = json!(price);
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getUses
    pub fn get_uses(&self) -> i64 {
        self.int("uses")
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setUses
    pub fn set_uses(&mut self, uses: i32) {
        self.0["uses"] = json!(uses);
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getMaxUses
    pub fn get_max_uses(&self) -> i64 {
        self.int("maxUses")
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setMaxUses
    pub fn set_max_uses(&mut self, uses: i32) {
        self.0["maxUses"] = json!(uses);
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.hasExperienceReward
    pub fn has_experience_reward(&self) -> bool {
        self.0["rewardExp"].as_bool().unwrap_or(self.int("rewardExp") != 0)
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setExperienceReward
    pub fn set_experience_reward(&mut self, reward: bool) {
        self.0["rewardExp"] = json!(reward);
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getVillagerExperience
    pub fn get_villager_experience(&self) -> i64 {
        self.int("xp")
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setVillagerExperience
    pub fn set_villager_experience(&mut self, experience: i32) {
        self.0["xp"] = json!(experience);
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.getPriceMultiplier
    pub fn get_price_multiplier(&self) -> f64 {
        self.0["priceMultiplier"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/inventory/MerchantRecipe.setPriceMultiplier
    pub fn set_price_multiplier(&mut self, multiplier: f32) {
        self.0["priceMultiplier"] = json!(multiplier);
    }
}

// ---- Damage sources ----

/// `org.bukkit.damage.DamageSource`, as a plugin builds one to hand to `damage`.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageSource {
    damage_type: String,
    causing: Option<Entity>,
    direct: Option<Entity>,
    location: Option<Location>,
    pub(crate) context: std::collections::BTreeMap<String, Value>,
}

impl DamageSource {
    /// bukkit: org/bukkit/damage/DamageSource.builder, org/bukkit/damage/DamageSource$Builder.build
    pub fn builder(damage_type: &str) -> Self {
        Self { damage_type: damage_type.to_owned(), causing: None, direct: None, location: None, context: std::collections::BTreeMap::new() }
    }
    /// bukkit: org/bukkit/damage/DamageSource$Builder.withCausingEntity
    pub fn with_causing_entity(mut self, entity: &Entity) -> Self {
        self.causing = Some(entity.clone());
        self
    }
    /// bukkit: org/bukkit/damage/DamageSource$Builder.withDirectEntity
    pub fn with_direct_entity(mut self, entity: &Entity) -> Self {
        self.direct = Some(entity.clone());
        self
    }
    /// bukkit: org/bukkit/damage/DamageSource$Builder.withDamageLocation
    pub fn with_damage_location(mut self, location: &Location) -> Self {
        self.location = Some(location.clone());
        self
    }
    /// bukkit: org/bukkit/damage/DamageSource.getDamageType
    pub fn get_damage_type(&self) -> &str {
        &self.damage_type
    }
    /// bukkit: org/bukkit/damage/DamageSource.getCausingEntity
    pub fn get_causing_entity(&self) -> Option<&Entity> {
        self.causing.as_ref()
    }
    /// bukkit: org/bukkit/damage/DamageSource.getDirectEntity
    pub fn get_direct_entity(&self) -> Option<&Entity> {
        self.direct.as_ref()
    }
    /// bukkit: org/bukkit/damage/DamageSource.getDamageLocation
    pub fn get_damage_location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
    /// bukkit: org/bukkit/damage/DamageSource.isIndirect
    pub fn is_indirect(&self) -> bool {
        self.causing != self.direct
    }
    fn data(&self) -> Value {
        let name = self.damage_type.trim_start_matches("minecraft:");
        query(json!({ "op": "data_file", "path": format!("damage_type/{name}") }))
    }
    /// bukkit: org/bukkit/damage/DamageSource.getFoodExhaustion, org/bukkit/damage/DamageType.getExhaustion
    pub fn get_food_exhaustion(&self) -> f64 {
        self.data()["exhaustion"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/damage/DamageSource.scalesWithDifficulty
    pub fn scales_with_difficulty(&self) -> bool {
        self.data()["scaling"].as_str() != Some("never")
    }
}

// ---- Block shapes ----

/// `org.bukkit.util.VoxelShape`: boxes, each `[x1, y1, z1, x2, y2, z2]` within the block.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxelShape(pub Vec<[f64; 6]>);

impl VoxelShape {
    /// bukkit: org/bukkit/util/VoxelShape.getBoundingBoxes
    pub fn get_bounding_boxes(&self) -> &[[f64; 6]] {
        &self.0
    }
    /// bukkit: org/bukkit/util/VoxelShape.overlaps
    pub fn overlaps(&self, other: &[f64; 6]) -> bool {
        self.0.iter().any(|shape| (0..3).all(|axis| shape[axis] < other[axis + 3] && other[axis] < shape[axis + 3]))
    }
}

// ---- Advancements ----

/// `org.bukkit.advancement.Advancement`: its file in the game's data.
#[derive(Clone, Debug, PartialEq)]
pub struct Advancement {
    pub id: String,
    data: Value,
}

fn advancement_data(id: &str) -> Value {
    query(json!({ "op": "data_file", "path": format!("advancement/{}", id.trim_start_matches("minecraft:")) }))
}

/// bukkit: org/bukkit/Bukkit.getAdvancement, org/bukkit/Server.getAdvancement
pub fn get_advancement(id: &str) -> Option<Advancement> {
    let data = advancement_data(id);
    (!data.is_null()).then(|| Advancement { id: id.to_owned(), data })
}

impl Advancement {
    /// bukkit: org/bukkit/advancement/Advancement.getCriteria
    pub fn get_criteria(&self) -> Vec<String> {
        self.data["criteria"].as_object().map_or_else(Vec::new, |criteria| criteria.keys().cloned().collect())
    }
    /// bukkit: org/bukkit/advancement/Advancement.getRequirements, org/bukkit/advancement/AdvancementRequirements.getRequirements, org/bukkit/advancement/AdvancementRequirement.getRequiredCriteria
    pub fn get_requirements(&self) -> Vec<Vec<String>> {
        let rows = self.data["requirements"].as_array().cloned().unwrap_or_default();
        rows.iter().map(|row| row.as_array().into_iter().flatten().filter_map(|name| name.as_str().map(str::to_owned)).collect()).collect()
    }
    /// bukkit: org/bukkit/advancement/Advancement.getParent
    pub fn get_parent(&self) -> Option<Advancement> {
        get_advancement(self.data["parent"].as_str()?)
    }
    /// bukkit: org/bukkit/advancement/Advancement.getRoot
    pub fn get_root(&self) -> Advancement {
        let mut at = self.clone();
        while let Some(parent) = at.get_parent() {
            at = parent;
        }
        at
    }
    /// bukkit: org/bukkit/advancement/Advancement.getDisplay
    pub fn get_display(&self) -> Option<AdvancementDisplay> {
        let display = &self.data["display"];
        (!display.is_null()).then(|| {
            let mut shown = display.clone();
            shown["__key"] = json!(self.id);
            AdvancementDisplay(shown)
        })
    }
}

/// `org.bukkit.advancement.AdvancementDisplay`.
#[derive(Clone, Debug, PartialEq)]
pub struct AdvancementDisplay(pub Value);

impl AdvancementDisplay {
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.getTitle
    pub fn get_title(&self) -> &Value {
        &self.0["title"]
    }
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.getDescription
    pub fn get_description(&self) -> &Value {
        &self.0["description"]
    }
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.getIcon
    pub fn get_icon(&self) -> Option<ItemStack> {
        ItemStack::read(&self.0["icon"])
    }
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.shouldShowToast
    pub fn should_show_toast(&self) -> bool {
        self.0["show_toast"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.shouldAnnounceChat
    pub fn should_announce_chat(&self) -> bool {
        self.0["announce_to_chat"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.isHidden
    pub fn is_hidden(&self) -> bool {
        self.0["hidden"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/advancement/AdvancementDisplay.getType
    pub fn get_type(&self) -> &str {
        self.0["frame"].as_str().unwrap_or("task")
    }
}

/// `org.bukkit.advancement.AdvancementProgress`: the player's file, and `/advancement` to change it.
#[derive(Clone, Debug, PartialEq)]
pub struct AdvancementProgress {
    pub advancement: Advancement,
    player: String,
    uuid: String,
}

fn dashed(uuid: &str) -> String {
    if uuid.len() != 32 {
        return uuid.to_owned();
    }
    format!("{}-{}-{}-{}-{}", &uuid[..8], &uuid[8..12], &uuid[12..16], &uuid[16..20], &uuid[20..])
}

impl Player {
    /// bukkit: org/bukkit/entity/Player.getAdvancementProgress
    pub fn get_advancement_progress(&self, advancement: &Advancement) -> AdvancementProgress {
        AdvancementProgress { advancement: advancement.clone(), player: self.get_name().to_owned(), uuid: self.uuid().to_owned() }
    }
}

impl AdvancementProgress {
    /// What the game has now, while they're on; their file, while they aren't.
    fn saved(&self) -> Value {
        let live = query(json!({ "op": "core_adv_live", "uuid": self.uuid }));
        if !live.is_null() {
            return live[&self.advancement.id].clone();
        }
        let file = crate::fam_core::world_folder().join(format!("players/advancements/{}.json", dashed(&self.uuid)));
        let all = serde_json::from_str::<Value>(&std::fs::read_to_string(file).unwrap_or_default()).unwrap_or_default();
        all[&self.advancement.id].clone()
    }
    /// bukkit: org/bukkit/advancement/AdvancementProgress.getAdvancement
    pub fn get_advancement(&self) -> &Advancement {
        &self.advancement
    }
    /// bukkit: org/bukkit/advancement/AdvancementProgress.isDone
    pub fn is_done(&self) -> bool {
        self.saved()["done"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/advancement/AdvancementProgress.awardCriteria
    pub fn award_criteria(&self, criteria: &str) {
        run_command(&format!("advancement grant {} only {} {criteria}", self.player, self.advancement.id));
    }
    /// bukkit: org/bukkit/advancement/AdvancementProgress.revokeCriteria
    pub fn revoke_criteria(&self, criteria: &str) {
        run_command(&format!("advancement revoke {} only {} {criteria}", self.player, self.advancement.id));
    }
    /// bukkit: org/bukkit/advancement/AdvancementProgress.getAwardedCriteria
    pub fn get_awarded_criteria(&self) -> Vec<String> {
        self.saved()["criteria"].as_object().map_or_else(Vec::new, |criteria| criteria.keys().cloned().collect())
    }
    /// bukkit: org/bukkit/advancement/AdvancementProgress.getRemainingCriteria
    pub fn get_remaining_criteria(&self) -> Vec<String> {
        let awarded = self.get_awarded_criteria();
        self.advancement.get_criteria().into_iter().filter(|name| !awarded.contains(name)).collect()
    }
    /// bukkit: org/bukkit/advancement/AdvancementProgress.getDateAwarded
    pub fn get_date_awarded(&self, criteria: &str) -> Option<String> {
        self.saved()["criteria"][criteria].as_str().map(str::to_owned)
    }
}

// ---- Recipes the game has ----

/// A recipe as the game's data has it.
#[derive(Clone, Debug, PartialEq)]
pub struct GameRecipe {
    pub key: String,
    pub data: Value,
}

impl GameRecipe {
    /// bukkit: org/bukkit/inventory/Recipe.getResult
    pub fn get_result(&self) -> Option<ItemStack> {
        let result = &self.data["result"];
        let id = result["id"].as_str().or(result.as_str())?;
        Some(ItemStack::new(Material::of(id), result["count"].as_i64().unwrap_or(1) as i32))
    }
}

/// bukkit: org/bukkit/Bukkit.getRecipe, org/bukkit/Server.getRecipe
pub fn get_recipe(key: &NamespacedKey) -> Option<GameRecipe> {
    let data = query(json!({ "op": "data_file", "path": format!("recipe/{}", key.get_key()) }));
    (!data.is_null()).then(|| GameRecipe { key: format!("{}:{}", key.get_namespace(), key.get_key()), data })
}

/// bukkit: org/bukkit/Bukkit.recipeIterator, org/bukkit/Server.recipeIterator
pub fn recipe_iterator() -> Vec<GameRecipe> {
    let names = query(json!({ "op": "data_list", "path": "recipe" }));
    names.as_array().into_iter().flatten().filter_map(|name| name.as_str()).filter_map(NamespacedKey::from_string).filter_map(|key| get_recipe(&key)).collect()
}

/// bukkit: org/bukkit/Bukkit.getRecipesFor, org/bukkit/Server.getRecipesFor
pub fn get_recipes_for(result: &ItemStack) -> Vec<GameRecipe> {
    recipe_iterator().into_iter().filter(|recipe| recipe.get_result().is_some_and(|made| made.get_type() == result.get_type())).collect()
}

// ---- Bans ----

/// `org.bukkit.BanEntry`: a row of `banned-players.json` or `banned-ips.json`.
#[derive(Clone, Debug, PartialEq)]
pub struct BanEntry {
    pub(crate) row: Value,
    pub(crate) addresses: bool,
}

/// bukkit: org/bukkit/BanList.getEntries, org/bukkit/BanList.getBanEntries
pub fn get_ban_entries(addresses: bool) -> Vec<BanEntry> {
    let file = if addresses { "banned-ips.json" } else { "banned-players.json" };
    let rows = serde_json::from_str::<Value>(&std::fs::read_to_string(server_folder().join(file)).unwrap_or_default()).unwrap_or_default();
    rows.as_array().into_iter().flatten().map(|row| BanEntry { row: row.clone(), addresses }).collect()
}

impl BanEntry {
    fn text(&self, key: &str) -> Option<&str> {
        self.row[key].as_str()
    }
    /// bukkit: org/bukkit/BanEntry.getTarget, org/bukkit/BanEntry.getBanTarget
    pub fn get_target(&self) -> &str {
        self.text("name").or(self.text("ip")).unwrap_or_default()
    }
    /// bukkit: org/bukkit/BanEntry.getCreated
    pub fn get_created(&self) -> Option<&str> {
        self.text("created")
    }
    /// bukkit: org/bukkit/BanEntry.getSource
    pub fn get_source(&self) -> Option<&str> {
        self.text("source")
    }
    /// bukkit: org/bukkit/BanEntry.getExpiration
    pub fn get_expiration(&self) -> Option<&str> {
        self.text("expires").filter(|when| *when != "forever")
    }
    /// bukkit: org/bukkit/BanEntry.getReason
    pub fn get_reason(&self) -> Option<&str> {
        self.text("reason")
    }
    /// bukkit: org/bukkit/BanEntry.remove
    pub fn remove(&self) {
        run_command(&format!("{} {}", if self.addresses { "pardon-ip" } else { "pardon" }, self.get_target()));
    }
}

// ---- Offline players ----

fn player_file(uuid: &str) -> std::path::PathBuf {
    crate::fam_core::world_folder().join(format!("players/data/{}.dat", dashed(uuid)))
}

fn millis(time: std::io::Result<std::time::SystemTime>) -> i64 {
    time.ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok()).map_or(0, |since| since.as_millis() as i64)
}

impl OfflinePlayer {
    /// bukkit: org/bukkit/OfflinePlayer.getFirstPlayed
    pub fn get_first_played(&self) -> i64 {
        std::fs::metadata(player_file(self.get_unique_id().unwrap_or_default())).map_or(0, |file| millis(file.created()))
    }
    /// bukkit: org/bukkit/OfflinePlayer.getLastPlayed, org/bukkit/OfflinePlayer.getLastSeen, org/bukkit/OfflinePlayer.getLastLogin
    pub fn get_last_played(&self) -> i64 {
        std::fs::metadata(player_file(self.get_unique_id().unwrap_or_default())).map_or(0, |file| millis(file.modified()))
    }
    /// bukkit: org/bukkit/OfflinePlayer.getStatistic
    ///
    /// A saved statistic, `stat_type` and `name` as the game's stats file has them.
    pub fn get_statistic(&self, stat_type: &str, name: &str) -> i64 {
        if let Some(value) = self.online_statistic(stat_type, name) {
            return value;
        }
        let file = crate::fam_core::world_folder().join(format!("players/stats/{}.json", dashed(self.get_unique_id().unwrap_or_default())));
        let stats = serde_json::from_str::<Value>(&std::fs::read_to_string(file).unwrap_or_default()).unwrap_or_default();
        stats["stats"][stat_type][name].as_i64().unwrap_or(0)
    }
}

impl crate::Player {
    /// bukkit: org/bukkit/entity/Player.hasPlayedBefore
    ///
    /// Whether the game has saved them before: false through a first visit until the world saves.
    pub fn has_played_before(&self) -> bool {
        player_file(self.uuid()).is_file()
    }
    /// bukkit: org/bukkit/entity/Player.getNearbyEntities, org/bukkit/entity/Entity.getNearbyEntities
    ///
    /// In a box `x y z` each way around them, themselves left out, as Bukkit's.
    pub fn get_nearby_entities_around(&self, x: f64, y: f64, z: f64) -> Vec<crate::entity::Entity> {
        let me = self.get_unique_id();
        self.get_location().get_nearby_entities(x, y, z).into_iter().filter(|other| other.get_unique_id() != me).collect()
    }
}

// ---- Help ----

/// `org.bukkit.help.HelpTopic`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpTopic {
    name: String,
    short_text: String,
    full_text: String,
}

impl HelpTopic {
    pub fn new(name: &str, short_text: &str, full_text: &str) -> Self {
        Self { name: name.to_owned(), short_text: short_text.to_owned(), full_text: full_text.to_owned() }
    }
    /// bukkit: org/bukkit/help/HelpTopic.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/help/HelpTopic.getShortText
    pub fn get_short_text(&self) -> &str {
        &self.short_text
    }
    /// bukkit: org/bukkit/help/HelpTopic.getFullText
    pub fn get_full_text(&self) -> &str {
        &self.full_text
    }
    /// bukkit: org/bukkit/help/HelpTopic.amendTopic
    pub fn amend_topic(&mut self, short_text: Option<&str>, full_text: Option<&str>) {
        if let Some(text) = short_text {
            text.clone_into(&mut self.short_text);
        }
        if let Some(text) = full_text {
            self.full_text = text.replacen("<text>", &self.full_text, 1);
        }
    }
}

static HELP: Mutex<BTreeMap<String, HelpTopic>> = Mutex::new(BTreeMap::new());

/// bukkit: org/bukkit/Bukkit.getHelpMap, org/bukkit/Server.getHelpMap, org/bukkit/help/HelpMap.addTopic
pub fn add_help_topic(topic: HelpTopic) {
    HELP.lock().unwrap_or_else(|e| e.into_inner()).insert(topic.name.to_ascii_lowercase(), topic);
}

/// bukkit: org/bukkit/help/HelpMap.getHelpTopic
pub fn get_help_topic(name: &str) -> Option<HelpTopic> {
    HELP.lock().unwrap_or_else(|e| e.into_inner()).get(&name.to_ascii_lowercase()).cloned()
}

/// bukkit: org/bukkit/help/HelpMap.getHelpTopics
pub fn get_help_topics() -> Vec<HelpTopic> {
    HELP.lock().unwrap_or_else(|e| e.into_inner()).values().cloned().collect()
}

/// bukkit: org/bukkit/help/HelpMap.clear
pub fn clear_help() {
    HELP.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

// ---- Paintings ----

/// `org.bukkit.Art`: a file of the game's `painting_variant`.
#[derive(Clone, Debug, PartialEq)]
pub struct Art {
    pub id: String,
    data: Value,
}

/// bukkit: org/bukkit/Art.getByName
pub fn art_by_name(name: &str) -> Option<Art> {
    let id = name.trim_start_matches("minecraft:").to_ascii_lowercase();
    let data = query(json!({ "op": "data_file", "path": format!("painting_variant/{id}") }));
    (!data.is_null()).then(|| Art { id: format!("minecraft:{id}"), data })
}

impl Art {
    /// bukkit: org/bukkit/Art.getBlockWidth
    pub fn get_block_width(&self) -> i64 {
        self.data["width"].as_i64().unwrap_or(1)
    }
    /// bukkit: org/bukkit/Art.getBlockHeight
    pub fn get_block_height(&self) -> i64 {
        self.data["height"].as_i64().unwrap_or(1)
    }
    /// bukkit: org/bukkit/Art.getKey, org/bukkit/Art.key
    pub fn get_key(&self) -> &str {
        &self.id
    }
    /// bukkit: org/bukkit/Art.title
    pub fn title(&self) -> &Value {
        &self.data["title"]
    }
    /// bukkit: org/bukkit/Art.author
    pub fn author(&self) -> &Value {
        &self.data["author"]
    }
    /// bukkit: org/bukkit/Art.assetId
    pub fn asset_id(&self) -> &str {
        self.data["asset_id"].as_str().unwrap_or_default()
    }
}

// ---- Chunk snapshots ----

/// `org.bukkit.ChunkSnapshot`: the blocks of a chunk as they were when it was taken.
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkSnapshot {
    x: i32,
    z: i32,
    world: String,
    bottom: i32,
    palette: Vec<String>,
    blocks: Vec<u32>,
    height: i32,
    time: i64,
}

impl crate::entity::Chunk {
    /// bukkit: org/bukkit/Chunk.getChunkSnapshot, org/bukkit/World.getEmptyChunkSnapshot
    pub fn get_chunk_snapshot(&self) -> Option<ChunkSnapshot> {
        let (x, z) = (self.get_x(), self.get_z());
        let data = query(json!({ "op": "chunk_snapshot", "dimension": self.get_world().dimension(), "chunk": [x, z] }));
        let palette: Vec<String> = data["palette"].as_array()?.iter().filter_map(|name| name.as_str().map(str::to_owned)).collect();
        let blocks: Vec<u32> = data["blocks"].as_array()?.iter().filter_map(|id| id.as_u64().map(|id| id as u32)).collect();
        Some(ChunkSnapshot {
            x,
            z,
            world: self.get_world().get_name().to_owned(),
            bottom: data["bottom"].as_i64()? as i32,
            height: data["height"].as_i64()? as i32,
            time: data["time"].as_i64().unwrap_or(0),
            palette,
            blocks,
        })
    }
}

impl ChunkSnapshot {
    fn state(&self, x: i32, y: i32, z: i32) -> &str {
        let index = ((y - self.bottom) * 256 + (z & 15) * 16 + (x & 15)) as usize;
        self.blocks.get(index).and_then(|id| self.palette.get(*id as usize)).map_or("minecraft:air", String::as_str)
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getX
    pub fn get_x(&self) -> i32 {
        self.x
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getZ
    pub fn get_z(&self) -> i32 {
        self.z
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getWorldName, org/bukkit/ChunkSnapshot.getWorldKey
    pub fn get_world_name(&self) -> &str {
        &self.world
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getBlockType
    pub fn get_block_type(&self, x: i32, y: i32, z: i32) -> Material {
        Material::of(self.state(x, y, z).split('[').next().unwrap_or("minecraft:air"))
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getBlockData
    pub fn get_block_data(&self, x: i32, y: i32, z: i32) -> &str {
        self.state(x, y, z)
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getHighestBlockYAt
    pub fn get_highest_block_y_at(&self, x: i32, z: i32) -> i32 {
        let top = self.bottom + self.height - 1;
        (self.bottom..=top).rev().find(|y| !self.state(x, *y, z).ends_with("air")).unwrap_or(self.bottom)
    }
    /// bukkit: org/bukkit/ChunkSnapshot.isSectionEmpty
    pub fn is_section_empty(&self, section: i32) -> bool {
        let start = self.bottom + section * 16;
        (start..start + 16).all(|y| (0..16).all(|z| (0..16).all(|x| self.state(x, y, z).ends_with("air"))))
    }
    /// bukkit: org/bukkit/ChunkSnapshot.contains
    pub fn contains(&self, state: &str) -> bool {
        self.palette.iter().any(|name| name == state)
    }
    /// bukkit: org/bukkit/ChunkSnapshot.getCaptureFullTime
    pub fn get_capture_full_time(&self) -> i64 {
        self.time
    }
}

// ---- The tick manager ----

/// `org.bukkit.ServerTickManager`: `/tick`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ServerTickManager;

/// bukkit: org/bukkit/Bukkit.getServerTickManager, org/bukkit/Server.getServerTickManager
pub fn get_server_tick_manager() -> ServerTickManager {
    ServerTickManager
}

impl ServerTickManager {
    fn state(&self) -> Value {
        query(json!({ "op": "tick_state" }))
    }
    /// bukkit: org/bukkit/ServerTickManager.getTickRate
    pub fn get_tick_rate(&self) -> f64 {
        self.state()["rate"].as_f64().unwrap_or(20.0)
    }
    /// bukkit: org/bukkit/ServerTickManager.isSprinting
    pub fn is_sprinting(&self) -> bool {
        self.state()["sprinting"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/ServerTickManager.isFrozen
    pub fn is_frozen(&self) -> bool {
        self.state()["frozen"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/ServerTickManager.isStepping
    pub fn is_stepping(&self) -> bool {
        self.state()["stepping"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/ServerTickManager.isRunningNormally
    pub fn is_running_normally(&self) -> bool {
        !self.is_frozen() && !self.is_sprinting() && !self.is_stepping()
    }
    /// bukkit: org/bukkit/ServerTickManager.setTickRate
    pub fn set_tick_rate(&self, rate: f32) {
        run_command(&format!("tick rate {rate}"));
    }
    /// bukkit: org/bukkit/ServerTickManager.setFrozen
    pub fn set_frozen(&self, frozen: bool) {
        run_command(if frozen { "tick freeze" } else { "tick unfreeze" });
    }
    /// bukkit: org/bukkit/ServerTickManager.stepGameIfFrozen
    pub fn step_game_if_frozen(&self, ticks: i32) -> bool {
        let frozen = self.is_frozen();
        if frozen {
            run_command(&format!("tick step {ticks}"));
        }
        frozen
    }
    /// bukkit: org/bukkit/ServerTickManager.stopStepping
    pub fn stop_stepping(&self) -> bool {
        let stepping = self.is_stepping();
        if stepping {
            run_command("tick step stop");
        }
        stepping
    }
    /// bukkit: org/bukkit/ServerTickManager.requestGameToSprint
    pub fn request_game_to_sprint(&self, ticks: i32) -> bool {
        run_command(&format!("tick sprint {ticks}"));
        true
    }
    /// bukkit: org/bukkit/ServerTickManager.stopSprinting
    pub fn stop_sprinting(&self) -> bool {
        let sprinting = self.is_sprinting();
        if sprinting {
            run_command("tick sprint stop");
        }
        sprinting
    }
}

// ---- What the server's files and version say ----

/// `org.bukkit.UnsafeValues`, the part of it that isn't Java.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnsafeValues;

/// bukkit: org/bukkit/Bukkit.getUnsafe, org/bukkit/Server.getUnsafe
pub fn get_unsafe() -> UnsafeValues {
    UnsafeValues
}

impl UnsafeValues {
    /// bukkit: org/bukkit/UnsafeValues.getDataVersion
    pub fn get_data_version(&self) -> i64 {
        // A game too old to answer: 26.3's own number (horizon-core's `version::DATA_VERSION`), never 0,
        // which plugins read as a server older than any feature they check for.
        query(json!({ "op": "version" }))["data_version"].as_i64().filter(|n| *n > 0).unwrap_or(5023)
    }
    /// bukkit: org/bukkit/UnsafeValues.getProtocolVersion
    pub fn get_protocol_version(&self) -> i64 {
        query(json!({ "op": "version" }))["protocol_version"].as_i64().filter(|n| *n > 0).unwrap_or(777)
    }
}

/// bukkit: org/bukkit/Server$Spigot.getConfig, org/bukkit/Server$Spigot.getSpigotConfig, org/bukkit/Server$Spigot.getBukkitConfig, org/bukkit/Server$Spigot.getPaperConfig, org/bukkit/Server.spigot, org/bukkit/Bukkit.spigot, org/bukkit/Server.getServerConfig
///
/// A settings file of the server by name: `spigot.yml`, `bukkit.yml` or `config/paper-global.yml`.
/// Horizon reads none of them, so each holds only what its owner wrote.
pub fn get_server_config(file: &str) -> crate::config::YamlConfiguration {
    crate::config::YamlConfiguration::load_configuration(&server_folder().join(file))
}

/// bukkit: org/bukkit/Bukkit.getLevelDirectory, org/bukkit/Server.getLevelDirectory
pub fn get_level_directory() -> std::path::PathBuf {
    crate::fam_core::world_folder()
}

/// bukkit: org/bukkit/Bukkit.restart, org/bukkit/Server.restart, org/bukkit/Server$Spigot.restart
pub fn restart() {
    run_command("stop");
}

/// bukkit: org/bukkit/event/block/Action.isRightClick
pub fn action_is_right_click(action: &str) -> bool {
    action.starts_with("RIGHT_CLICK")
}

/// bukkit: org/bukkit/event/block/Action.isLeftClick
pub fn action_is_left_click(action: &str) -> bool {
    action.starts_with("LEFT_CLICK")
}

impl World {
    /// bukkit: org/bukkit/World.isChunkGenerated
    pub fn is_chunk_generated(&self, x: i32, z: i32) -> bool {
        let region = self.get_world_folder().join(format!("region/r.{}.{}.mca", x >> 5, z >> 5));
        self.get_chunk_at(x, z).is_loaded() || region.is_file()
    }
}
