//! `org.bukkit.scoreboard`, attributes and boss bars, over the game's own commands and what it answers.
use crate::entity::Entity;
use crate::{BossBar, Component, Player, adventure};
use horizon_mod::{Value, query, run_command};
use serde_json::json;

fn board() -> Value {
    query(json!({ "op": "scoreboard" }))
}

/// `org.bukkit.scoreboard.Scoreboard`: the one the game keeps. `get_new_scoreboard` is this one too.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scoreboard;

/// bukkit: org/bukkit/Bukkit.getScoreboardManager, org/bukkit/Server.getScoreboardManager, org/bukkit/scoreboard/ScoreboardManager.getMainScoreboard
pub fn get_main_scoreboard() -> Scoreboard {
    Scoreboard
}

impl Scoreboard {
    /// bukkit: org/bukkit/scoreboard/Scoreboard.registerNewObjective
    pub fn register_new_objective(&self, name: &str, criteria: &str, display_name: &Component) -> Objective {
        run_command(&format!("scoreboard objectives add {name} {criteria} {}", display_name.0));
        Objective { name: name.to_owned() }
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getObjective
    pub fn get_objective(&self, name: &str) -> Option<Objective> {
        board()["objectives"].as_array()?.iter().any(|objective| objective["name"] == name).then(|| Objective { name: name.to_owned() })
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getObjectives
    pub fn get_objectives(&self) -> Vec<Objective> {
        board()["objectives"].as_array().into_iter().flatten().filter_map(|objective| Some(Objective { name: objective["name"].as_str()?.to_owned() })).collect()
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getObjectivesByCriteria
    pub fn get_objectives_by_criteria(&self, criteria: &str) -> Vec<Objective> {
        board()["objectives"].as_array().into_iter().flatten().filter(|objective| objective["criteria"] == criteria).filter_map(|objective| Some(Objective { name: objective["name"].as_str()?.to_owned() })).collect()
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.resetScores
    pub fn reset_scores(&self, entry: &str) {
        run_command(&format!("scoreboard players reset {entry}"));
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getEntries
    pub fn get_entries(&self) -> Vec<String> {
        board()["scores"].as_object().into_iter().flatten().map(|(entry, _)| entry.clone()).collect()
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getTeam
    pub fn get_team(&self, name: &str) -> Option<Team> {
        board()["teams"].as_array()?.iter().any(|team| team["name"] == name).then(|| Team { name: name.to_owned() })
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getTeams
    pub fn get_teams(&self) -> Vec<Team> {
        board()["teams"].as_array().into_iter().flatten().filter_map(|team| Some(Team { name: team["name"].as_str()?.to_owned() })).collect()
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.registerNewTeam
    pub fn register_new_team(&self, name: &str) -> Team {
        run_command(&format!("team add {name}"));
        Team { name: name.to_owned() }
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getEntryTeam, org/bukkit/scoreboard/Scoreboard.getPlayerTeam
    pub fn get_entry_team(&self, entry: &str) -> Option<Team> {
        board()["teams"].as_array()?.iter().find(|team| team["players"].as_array().is_some_and(|players| players.iter().any(|player| player == entry))).and_then(|team| Some(Team { name: team["name"].as_str()?.to_owned() }))
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.clearSlot
    pub fn clear_slot(&self, slot: &str) {
        run_command(&format!("scoreboard objectives setdisplay {}", display_slot(slot)));
    }
}

fn display_slot(slot: &str) -> String {
    match slot {
        "SIDEBAR" => "sidebar".to_owned(),
        "PLAYER_LIST" => "list".to_owned(),
        "BELOW_NAME" => "below_name".to_owned(),
        other => other.to_ascii_lowercase().replace("sidebar_team_", "sidebar.team."),
    }
}

/// `org.bukkit.scoreboard.Objective`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objective {
    name: String,
}

impl Objective {
    fn state(&self) -> Value {
        board()["objectives"].as_array().into_iter().flatten().find(|objective| objective["name"] == self.name.as_str()).cloned().unwrap_or_default()
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/scoreboard/Objective.displayName
    pub fn display_name(&self) -> Component {
        Component(self.state()["display_name"].take())
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getDisplayName
    pub fn get_display_name(&self) -> String {
        self.display_name().to_legacy()
    }
    /// bukkit: org/bukkit/scoreboard/Objective.setDisplayName
    pub fn set_display_name(&self, name: &str) {
        run_command(&format!("scoreboard objectives modify {} displayname {}", self.name, adventure::legacy(name).0));
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getCriteria, org/bukkit/scoreboard/Objective.getTrackedCriteria, org/bukkit/scoreboard/Criteria.getName
    pub fn get_criteria(&self) -> String {
        self.state()["criteria"].as_str().unwrap_or_default().to_owned()
    }
    /// bukkit: org/bukkit/scoreboard/Objective.unregister
    pub fn unregister(&self) {
        run_command(&format!("scoreboard objectives remove {}", self.name));
    }
    /// bukkit: org/bukkit/scoreboard/Objective.setDisplaySlot
    pub fn set_display_slot(&self, slot: &str) {
        run_command(&format!("scoreboard objectives setdisplay {} {}", display_slot(slot), self.name));
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getDisplaySlot
    pub fn get_display_slot(&self) -> Option<String> {
        board()["display"].as_object()?.iter().find(|(_, objective)| *objective == self.name.as_str()).map(|(slot, _)| slot.to_ascii_uppercase())
    }
    /// bukkit: org/bukkit/scoreboard/Objective.setRenderType
    pub fn set_render_type(&self, render: &str) {
        run_command(&format!("scoreboard objectives modify {} rendertype {}", self.name, render.to_ascii_lowercase()));
    }
    /// bukkit: org/bukkit/scoreboard/Objective.setAutoUpdateDisplay
    pub fn set_auto_update_display(&self, on: bool) {
        run_command(&format!("scoreboard objectives modify {} displayautoupdate {on}", self.name));
    }
    /// bukkit: org/bukkit/scoreboard/Objective.willAutoUpdateDisplay
    pub fn will_auto_update_display(&self) -> bool {
        self.state()["display_auto_update"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getScore, org/bukkit/scoreboard/Objective.getScoreFor
    pub fn get_score(&self, entry: &str) -> Score {
        Score { objective: self.name.clone(), entry: entry.to_owned() }
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getScoreboard
    pub fn get_scoreboard(&self) -> Scoreboard {
        Scoreboard
    }
}

/// `org.bukkit.scoreboard.Score`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Score {
    objective: String,
    entry: String,
}

impl Score {
    fn value(&self) -> Option<i64> {
        board()["scores"][&self.entry][&self.objective]["value"].as_i64()
    }
    /// bukkit: org/bukkit/scoreboard/Score.getEntry
    pub fn get_entry(&self) -> &str {
        &self.entry
    }
    /// bukkit: org/bukkit/scoreboard/Score.getObjective
    pub fn get_objective(&self) -> Objective {
        Objective { name: self.objective.clone() }
    }
    /// bukkit: org/bukkit/scoreboard/Score.getScore
    pub fn get_score(&self) -> i32 {
        self.value().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/scoreboard/Score.setScore
    pub fn set_score(&self, score: i32) {
        run_command(&format!("scoreboard players set {} {} {score}", self.entry, self.objective));
    }
    /// bukkit: org/bukkit/scoreboard/Score.isScoreSet
    pub fn is_score_set(&self) -> bool {
        self.value().is_some()
    }
    /// bukkit: org/bukkit/scoreboard/Score.resetScore
    pub fn reset_score(&self) {
        run_command(&format!("scoreboard players reset {} {}", self.entry, self.objective));
    }
    /// bukkit: org/bukkit/scoreboard/Score.getScoreboard
    pub fn get_scoreboard(&self) -> Scoreboard {
        Scoreboard
    }
}

/// `org.bukkit.scoreboard.Team`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Team {
    name: String,
}

impl Team {
    fn state(&self) -> Value {
        board()["teams"].as_array().into_iter().flatten().find(|team| team["name"] == self.name.as_str()).cloned().unwrap_or_default()
    }
    fn modify(&self, option: &str, value: &str) {
        run_command(&format!("team modify {} {option} {value}", self.name));
    }
    /// bukkit: org/bukkit/scoreboard/Team.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/scoreboard/Team.displayName
    pub fn display_name(&self) -> Component {
        Component(self.state()["display_name"].take())
    }
    /// bukkit: org/bukkit/scoreboard/Team.getDisplayName
    pub fn get_display_name(&self) -> String {
        self.display_name().to_legacy()
    }
    /// bukkit: org/bukkit/scoreboard/Team.setDisplayName
    pub fn set_display_name(&self, name: &str) {
        self.modify("displayName", &adventure::legacy(name).0.to_string());
    }
    /// bukkit: org/bukkit/scoreboard/Team.prefix
    pub fn prefix(&self) -> Component {
        Component(self.state()["prefix"].take())
    }
    /// bukkit: org/bukkit/scoreboard/Team.getPrefix
    pub fn get_prefix(&self) -> String {
        self.prefix().to_legacy()
    }
    /// bukkit: org/bukkit/scoreboard/Team.setPrefix
    pub fn set_prefix(&self, prefix: &str) {
        self.modify("prefix", &adventure::legacy(prefix).0.to_string());
    }
    /// bukkit: org/bukkit/scoreboard/Team.suffix
    pub fn suffix(&self) -> Component {
        Component(self.state()["suffix"].take())
    }
    /// bukkit: org/bukkit/scoreboard/Team.getSuffix
    pub fn get_suffix(&self) -> String {
        self.suffix().to_legacy()
    }
    /// bukkit: org/bukkit/scoreboard/Team.setSuffix
    pub fn set_suffix(&self, suffix: &str) {
        self.modify("suffix", &adventure::legacy(suffix).0.to_string());
    }
    /// bukkit: org/bukkit/scoreboard/Team.hasColor
    pub fn has_color(&self) -> bool {
        self.state()["color"].is_string()
    }
    /// bukkit: org/bukkit/scoreboard/Team.getColor
    pub fn get_color(&self) -> Option<String> {
        self.state()["color"].as_str().map(|color| color.to_ascii_uppercase())
    }
    /// bukkit: org/bukkit/scoreboard/Team.setColor
    pub fn set_color(&self, color: &str) {
        self.modify("color", &color.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/scoreboard/Team.allowFriendlyFire
    pub fn allow_friendly_fire(&self) -> bool {
        self.state()["allow_friendly_fire"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/scoreboard/Team.setAllowFriendlyFire
    pub fn set_allow_friendly_fire(&self, on: bool) {
        self.modify("friendlyFire", if on { "true" } else { "false" });
    }
    /// bukkit: org/bukkit/scoreboard/Team.canSeeFriendlyInvisibles
    pub fn can_see_friendly_invisibles(&self) -> bool {
        self.state()["see_friendly_invisibles"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/scoreboard/Team.setCanSeeFriendlyInvisibles
    pub fn set_can_see_friendly_invisibles(&self, on: bool) {
        self.modify("seeFriendlyInvisibles", if on { "true" } else { "false" });
    }
    /// bukkit: org/bukkit/scoreboard/Team.getEntries
    pub fn get_entries(&self) -> Vec<String> {
        self.state()["players"].as_array().into_iter().flatten().filter_map(|entry| entry.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/scoreboard/Team.getSize
    pub fn get_size(&self) -> usize {
        self.get_entries().len()
    }
    /// bukkit: org/bukkit/scoreboard/Team.addEntry, org/bukkit/scoreboard/Team.addPlayer
    pub fn add_entry(&self, entry: &str) {
        run_command(&format!("team join {} {entry}", self.name));
    }
    /// bukkit: org/bukkit/scoreboard/Team.addEntries
    pub fn add_entries(&self, entries: &[&str]) {
        entries.iter().for_each(|entry| self.add_entry(entry));
    }
    /// bukkit: org/bukkit/scoreboard/Team.addEntity
    pub fn add_entity(&self, entity: &Entity) {
        self.add_entry(&entity.get_unique_id());
    }
    /// bukkit: org/bukkit/scoreboard/Team.removeEntry, org/bukkit/scoreboard/Team.removePlayer
    pub fn remove_entry(&self, entry: &str) -> bool {
        let had = self.has_entry(entry);
        run_command(&format!("team leave {entry}"));
        had
    }
    /// bukkit: org/bukkit/scoreboard/Team.removeEntries
    pub fn remove_entries(&self, entries: &[&str]) -> bool {
        entries.iter().fold(false, |any, entry| self.remove_entry(entry) || any)
    }
    /// bukkit: org/bukkit/scoreboard/Team.removeEntity
    pub fn remove_entity(&self, entity: &Entity) -> bool {
        self.remove_entry(&entity.get_unique_id())
    }
    /// bukkit: org/bukkit/scoreboard/Team.hasEntry, org/bukkit/scoreboard/Team.hasPlayer
    pub fn has_entry(&self, entry: &str) -> bool {
        self.get_entries().iter().any(|known| known == entry)
    }
    /// bukkit: org/bukkit/scoreboard/Team.hasEntity
    pub fn has_entity(&self, entity: &Entity) -> bool {
        self.has_entry(&entity.get_unique_id())
    }
    /// bukkit: org/bukkit/scoreboard/Team.unregister
    pub fn unregister(&self) {
        run_command(&format!("team remove {}", self.name));
    }
    /// bukkit: org/bukkit/scoreboard/Team.getScoreboard
    pub fn get_scoreboard(&self) -> Scoreboard {
        Scoreboard
    }
    /// bukkit: org/bukkit/scoreboard/Team.setOption
    ///
    /// `NAME_TAG_VISIBILITY`, `DEATH_MESSAGE_VISIBILITY` or `COLLISION_RULE`, with `ALWAYS`, `NEVER`, `FOR_OTHER_TEAMS` or `FOR_OWN_TEAM`.
    pub fn set_option(&self, option: &str, status: &str) {
        let name = match option {
            "NAME_TAG_VISIBILITY" => "nametagVisibility",
            "DEATH_MESSAGE_VISIBILITY" => "deathMessageVisibility",
            _ => "collisionRule",
        };
        let value = match (option, status) {
            (_, "ALWAYS") => "always",
            (_, "NEVER") => "never",
            ("COLLISION_RULE", "FOR_OTHER_TEAMS") => "pushOtherTeams",
            ("COLLISION_RULE", _) => "pushOwnTeam",
            (_, "FOR_OTHER_TEAMS") => "hideForOwnTeam",
            _ => "hideForOtherTeams",
        };
        self.modify(name, value);
    }
}

/// bukkit: org/bukkit/attribute/Attributable, org/bukkit/attribute/AttributeInstance
///
/// One attribute of an entity or player: `target` is what `/attribute` selects it by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeInstance {
    target: String,
    attribute: String,
    saved: Value,
    /// A player's id: their attributes aren't what `/attribute` reaches.
    player: Option<String>,
}

/// `org.bukkit.attribute.AttributeModifier`.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeModifier {
    pub key: String,
    pub amount: f64,
    pub operation: String,
    /// The game's `EquipmentSlotGroup`: `any` unless an item said.
    pub slot: String,
}

impl AttributeModifier {
    /// bukkit: org/bukkit/attribute/AttributeModifier.<init>
    pub fn new(key: &str, amount: f64, operation: &str) -> Self {
        Self { key: key.to_owned(), amount, operation: operation.to_owned(), slot: "any".to_owned() }
    }
    /// bukkit: org/bukkit/attribute/AttributeModifier.getKey
    pub fn get_key(&self) -> &str {
        &self.key
    }
    /// bukkit: org/bukkit/attribute/AttributeModifier.getName
    pub fn get_name(&self) -> &str {
        &self.key
    }
    /// bukkit: org/bukkit/attribute/AttributeModifier.getAmount
    pub fn get_amount(&self) -> f64 {
        self.amount
    }
    /// bukkit: org/bukkit/attribute/AttributeModifier.getOperation
    pub fn get_operation(&self) -> &str {
        &self.operation
    }
    /// The command's and the saved form's name for the operation.
    fn command_operation(&self) -> &'static str {
        match self.operation.as_str() {
            "ADD_SCALAR" | "add_multiplied_base" => "add_multiplied_base",
            "MULTIPLY_SCALAR_1" | "add_multiplied_total" => "add_multiplied_total",
            _ => "add_value",
        }
    }
}

fn saved_attribute(data: &Value, id: &str) -> Value {
    data["attributes"].as_array().into_iter().flatten().find(|attribute| attribute["id"] == id).cloned().unwrap_or_default()
}

fn attribute_key(attribute: &str) -> String {
    if attribute.contains(':') { attribute.to_owned() } else { format!("minecraft:{}", attribute.to_ascii_lowercase()) }
}

impl Entity {
    /// bukkit: org/bukkit/attribute/Attributable.getAttribute
    pub fn get_attribute(&self, attribute: &str) -> AttributeInstance {
        let id = attribute_key(attribute);
        AttributeInstance { saved: saved_attribute(&self.get_data(), &id), target: self.get_unique_id(), attribute: id, player: None }
    }
}

impl Player {
    /// bukkit: org/bukkit/attribute/Attributable.getAttribute
    ///
    /// The live values: a player's saved data is only written when the game saves.
    pub fn get_attribute(&self, attribute: &str) -> AttributeInstance {
        let id = attribute_key(attribute);
        let mut saved = saved_attribute(&self.get_data(), &id);
        let live = query(json!({ "op": "attribute", "uuid": self.uuid(), "id": id }));
        if let Some(base) = live["base"].as_f64() {
            saved["base"] = json!(base);
            saved["value"] = live["value"].clone();
        }
        AttributeInstance { saved, target: self.get_name().to_owned(), attribute: id, player: Some(self.uuid().to_owned()) }
    }
}

impl AttributeInstance {
    /// bukkit: org/bukkit/attribute/AttributeInstance.getAttribute
    pub fn get_attribute(&self) -> &str {
        &self.attribute
    }
    /// bukkit: org/bukkit/attribute/AttributeInstance.getBaseValue
    pub fn get_base_value(&self) -> f64 {
        self.saved["base"].as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/attribute/AttributeInstance.setBaseValue
    pub fn set_base_value(&self, value: f64) {
        match &self.player {
            Some(uuid) => {
                query(json!({ "op": "attribute_set", "uuid": uuid, "id": self.attribute, "base": value }));
            }
            None => run_command(&format!("attribute {} {} base set {value}", self.target, self.attribute)),
        }
    }
    /// bukkit: org/bukkit/attribute/AttributeInstance.getModifiers
    pub fn get_modifiers(&self) -> Vec<AttributeModifier> {
        self.saved["modifiers"].as_array().into_iter().flatten().filter_map(|modifier| Some(AttributeModifier { key: modifier["id"].as_str()?.to_owned(), amount: modifier["amount"].as_f64()?, operation: modifier["operation"].as_str()?.to_owned(), slot: "any".to_owned() })).collect()
    }
    /// bukkit: org/bukkit/attribute/AttributeInstance.getModifier
    pub fn get_modifier(&self, key: &str) -> Option<AttributeModifier> {
        self.get_modifiers().into_iter().find(|modifier| modifier.key == key)
    }
    /// bukkit: org/bukkit/attribute/AttributeInstance.addModifier, org/bukkit/attribute/AttributeInstance.addTransientModifier
    pub fn add_modifier(&self, modifier: &AttributeModifier) {
        match &self.player {
            Some(uuid) => {
                query(json!({ "op": "attribute_set", "uuid": uuid, "id": self.attribute, "modifier": modifier.key, "amount": modifier.amount, "operation": modifier.command_operation(), "add": true }));
            }
            None => run_command(&format!("attribute {} {} modifier add {} {} {}", self.target, self.attribute, modifier.key, modifier.amount, modifier.command_operation())),
        }
    }
    /// bukkit: org/bukkit/attribute/AttributeInstance.removeModifier
    pub fn remove_modifier(&self, key: &str) {
        match &self.player {
            Some(uuid) => {
                query(json!({ "op": "attribute_set", "uuid": uuid, "id": self.attribute, "modifier": key, "add": false }));
            }
            None => run_command(&format!("attribute {} {} modifier remove {key}", self.target, self.attribute)),
        }
    }
    /// bukkit: org/bukkit/attribute/AttributeInstance.getValue
    ///
    /// As `AttributeInstance.calculateValue`: added values, then base multiples, then total multiples.
    pub fn get_value(&self) -> f64 {
        if let Some(value) = self.saved["value"].as_f64() {
            return value;
        }
        let modifiers = self.get_modifiers();
        let amounts = |operation: &str| -> Vec<f64> { modifiers.iter().filter(|modifier| modifier.command_operation() == operation).map(|modifier| modifier.amount).collect() };
        let base = self.get_base_value() + amounts("add_value").iter().sum::<f64>();
        let scaled = base + base * amounts("add_multiplied_base").iter().sum::<f64>();
        amounts("add_multiplied_total").iter().fold(scaled, |value, amount| value * (1.0 + amount))
    }
}

impl BossBar {
    fn query(&self) -> Value {
        query(json!({ "op": "boss_bar", "id": self.id() }))
    }
    /// bukkit: org/bukkit/boss/BossBar.getTitle
    pub fn get_title(&self) -> String {
        Component(self.query()["name"].take()).to_legacy()
    }
    /// bukkit: org/bukkit/boss/BossBar.getColor
    pub fn get_color(&self) -> String {
        self.query()["color"].as_str().unwrap_or("white").to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/boss/BossBar.setColor
    pub fn set_color(&self, color: &str) {
        run_command(&format!("bossbar set {} color {}", self.id(), color.to_ascii_lowercase()));
    }
    /// bukkit: org/bukkit/boss/BossBar.getStyle
    pub fn get_style(&self) -> String {
        self.query()["style"].as_str().unwrap_or("SOLID").to_owned()
    }
    /// bukkit: org/bukkit/boss/BossBar.setStyle
    pub fn set_style(&self, style: &str) {
        let name = match style {
            "SEGMENTED_6" => "notched_6",
            "SEGMENTED_10" => "notched_10",
            "SEGMENTED_12" => "notched_12",
            "SEGMENTED_20" => "notched_20",
            _ => "progress",
        };
        run_command(&format!("bossbar set {} style {name}", self.id()));
    }
    /// bukkit: org/bukkit/boss/BossBar.getProgress
    pub fn get_progress(&self) -> f64 {
        let state = self.query();
        state["value"].as_f64().unwrap_or(0.0) / state["max"].as_f64().unwrap_or(1.0).max(1.0)
    }
    /// bukkit: org/bukkit/boss/BossBar.isVisible
    pub fn is_visible(&self) -> bool {
        self.query()["visible"].as_bool().unwrap_or(true)
    }
    /// bukkit: org/bukkit/boss/BossBar.getPlayers
    pub fn get_players(&self) -> Vec<Player> {
        self.query()["players"].as_array().into_iter().flatten().filter_map(|uuid| Player::read(&query(json!({ "op": "player", "uuid": uuid })))).collect()
    }
    /// bukkit: org/bukkit/boss/BossBar.removePlayer
    pub fn remove_player(&self, player: &Player) {
        query(json!({ "op": "core_bossbar_player", "id": self.id(), "uuid": player.uuid(), "add": false }));
    }
}
