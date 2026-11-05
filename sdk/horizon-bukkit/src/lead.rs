//! What took engine work: scoreboards of a player's own, and the server settings Paper lets a plugin
//! change while it runs.
#![allow(missing_docs)]
use crate::adventure::Component;
use crate::Player;
use horizon_mod::query;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

/// `org.bukkit.scoreboard.Team` on a board a plugin made.
#[derive(Clone, Debug)]
struct BoardTeam {
    display: Component,
    prefix: Component,
    suffix: Component,
    color: Option<i32>,
    friendly_fire: bool,
    see_invisibles: bool,
    /// `Team.OptionStatus` by option: name tags, death messages, collision.
    options: BTreeMap<String, String>,
    entries: Vec<String>,
}

const TEAM_COLORS: [&str; 16] = ["BLACK", "DARK_BLUE", "DARK_GREEN", "DARK_AQUA", "DARK_RED", "DARK_PURPLE", "GOLD", "GRAY", "DARK_GRAY", "BLUE", "GREEN", "AQUA", "RED", "LIGHT_PURPLE", "YELLOW", "WHITE"];

impl BoardTeam {
    fn new(name: &str) -> Self {
        let blank = Component(json!(""));
        Self { display: Component(json!(name)), prefix: blank.clone(), suffix: blank, color: None, friendly_fire: true, see_invisibles: true, options: BTreeMap::new(), entries: Vec::new() }
    }
    /// `Team.Visibility` and `Team.CollisionRule`, which number these alike.
    fn status(&self, option: &str) -> i32 {
        match self.options.get(option).map(String::as_str) {
            Some("NEVER") => 1,
            Some("FOR_OTHER_TEAMS") => 2,
            Some("FOR_OWN_TEAM") => 3,
            _ => 0,
        }
    }
    fn send(&self, viewer: &Player, name: &str, method: i32, players: &[String]) {
        let options = i32::from(self.friendly_fire) | (i32::from(self.see_invisibles) << 1);
        query(json!({
            "op": "core_board_team", "uuid": viewer.uuid(), "name": name, "method": method, "players": players, "display": self.display.0, "prefix": self.prefix.0,
            "suffix": self.suffix.0, "visibility": self.status("NAME_TAG_VISIBILITY"), "collision": self.status("COLLISION_RULE"), "color": self.color, "options": options,
        }));
    }
}

#[derive(Default)]
struct Board {
    /// A counted criterion's total when last read, by objective and owner.
    counted: BTreeMap<(String, String), i64>,
    /// What each criterion read for each owner last: `ServerPlayer`'s `lastRecorded...`.
    recorded: BTreeMap<(String, String), i32>,
    /// Objective to (shown name, criteria, scores).
    objectives: BTreeMap<String, (Component, String, BTreeMap<String, i32>)>,
    /// Objective to `RenderType`: 1 for hearts.
    render: BTreeMap<String, i32>,
    /// Slot number to objective.
    display: BTreeMap<i32, String>,
    teams: BTreeMap<String, BoardTeam>,
    /// Players looking at this board.
    viewers: Vec<Player>,
}

/// `org.bukkit.scoreboard.Scoreboard` from `getNewScoreboard`: it exists only in the clients of the
/// players shown it, so one player's sidebar is never another's.
#[derive(Clone, Default)]
pub struct PlayerScoreboard(Arc<Mutex<Board>>);

impl PartialEq for PlayerScoreboard {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// Every board a plugin made: `CraftScoreboardManager` keeps these up, viewed or not.
static BOARDS: Mutex<Vec<Weak<Mutex<Board>>>> = Mutex::new(Vec::new());
/// Which board a player looks at, while it isn't the game's.
static LOOKING: Mutex<BTreeMap<String, PlayerScoreboard>> = Mutex::new(BTreeMap::new());
/// Boards waiting for the game's to leave a client: the hub was out of reach.
static TO_SHOW: Mutex<Vec<(Player, PlayerScoreboard)>> = Mutex::new(Vec::new());
/// How far `core_board_kills` has been read.
static KILLS_READ: AtomicU64 = AtomicU64::new(0);

/// bukkit: org/bukkit/scoreboard/ScoreboardManager.getNewScoreboard
pub fn get_new_scoreboard() -> PlayerScoreboard {
    let board = PlayerScoreboard::default();
    BOARDS.lock().unwrap_or_else(|e| e.into_inner()).push(Arc::downgrade(&board.0));
    board
}

/// What a criterion comes to for a player: a value as it stands, or a running total that scores
/// count up from when they were first seen on it.
enum Reading {
    Now(i32),
    Total(i64),
}

fn statistic(player: &Player, kind: &str, key: &str) -> i64 {
    query(json!({ "op": "core_stat_get", "uuid": player.uuid().replace('-', ""), "kind": kind, "key": key })).as_i64().unwrap_or(0)
}

/// `ObjectiveCriteria`, as `ServerPlayer.doTick` and `awardStat` keep them. Nothing for `dummy`,
/// `trigger` and the team ones, which come as they fall.
fn read_criteria(criteria: &str, player: &Player) -> Option<Reading> {
    let custom = "minecraft:custom";
    Some(match criteria {
        "health" => Reading::Now((player.get_health() + player.get_absorption_amount()).ceil() as i32),
        "food" => Reading::Now(player.get_food_level()),
        "air" => Reading::Now(player.get_remaining_air()),
        "armor" => Reading::Now(player.get_attribute("armor").get_value() as i32),
        "xp" => Reading::Now(player.get_total_experience()),
        "level" => Reading::Now(player.get_level()),
        "deathCount" => Reading::Total(statistic(player, custom, "minecraft:deaths")),
        "playerKillCount" => Reading::Total(statistic(player, custom, "minecraft:player_kills")),
        "totalKillCount" => Reading::Total(statistic(player, custom, "minecraft:player_kills") + statistic(player, custom, "minecraft:mob_kills")),
        _ => {
            let (kind, key) = criteria.split_once(':')?;
            kind.starts_with("minecraft.").then(|| Reading::Total(statistic(player, &kind.replace('.', ":"), &key.replace('.', ":"))))?
        }
    })
}

fn is_team_kill(criteria: &str) -> bool {
    criteria.starts_with("teamkill.") || criteria.starts_with("killedByTeam.")
}

/// Each tick: what the game keeps up on its own board, on the boards plugins made.
pub(crate) fn keep_up() {
    let boards: Vec<PlayerScoreboard> = {
        let mut all = BOARDS.lock().unwrap_or_else(|e| e.into_inner());
        all.retain(|board| board.strong_count() > 0);
        all.iter().filter_map(Weak::upgrade).map(PlayerScoreboard).collect()
    };
    if boards.is_empty() {
        return;
    }
    let online = crate::get_online_players();
    let waiting = std::mem::take(&mut *TO_SHOW.lock().unwrap_or_else(|e| e.into_inner()));
    for (player, board) in waiting {
        player.set_scoreboard(&board);
    }
    let names: Vec<&str> = online.iter().map(Player::get_name).collect();
    LOOKING.lock().unwrap_or_else(|e| e.into_inner()).retain(|uuid, _| online.iter().any(|player| player.uuid() == uuid));
    let mut wanted: Vec<String> = Vec::new();
    for board in &boards {
        let mut held = board.board();
        held.viewers.retain(|viewer| online.contains(viewer));
        held.recorded.retain(|(_, owner), _| names.contains(&owner.as_str()));
        wanted.extend(held.objectives.values().map(|(_, criteria, _)| criteria.clone()));
    }
    wanted.sort_unstable();
    wanted.dedup();
    if wanted.iter().any(|criteria| is_team_kill(criteria)) {
        let log = query(json!({ "op": "core_board_kills", "from": KILLS_READ.load(Ordering::Relaxed) }));
        KILLS_READ.store(log["next"].as_u64().unwrap_or(0), Ordering::Relaxed);
        for row in log["rows"].as_array().into_iter().flatten() {
            if let (Some(criteria), Some(owner)) = (row[0].as_str(), row[1].as_str()) {
                for board in &boards {
                    board.add_for(criteria, owner);
                }
            }
        }
    }
    wanted.retain(|criteria| !matches!(criteria.as_str(), "dummy" | "trigger") && !is_team_kill(criteria));
    if wanted.is_empty() {
        return;
    }
    for player in &online {
        let readings: BTreeMap<&str, Reading> = wanted.iter().filter_map(|criteria| Some((criteria.as_str(), read_criteria(criteria, player)?))).collect();
        for board in &boards {
            board.record(player.get_name(), &readings);
        }
    }
}

fn slot_number(slot: &str) -> i32 {
    match slot {
        "PLAYER_LIST" => 0,
        "SIDEBAR" => 1,
        "BELOW_NAME" => 2,
        _ => 1,
    }
}

fn slot_name(slot: i32) -> &'static str {
    match slot {
        0 => "PLAYER_LIST",
        2 => "BELOW_NAME",
        _ => "SIDEBAR",
    }
}

impl Board {
    fn show_objective(&self, viewer: &Player, name: &str, shown: &Component, method: i32) {
        viewer.send("sb_objective", json!({ "name": name, "method": method, "display": shown.0, "render": self.render.get(name).copied().unwrap_or(0) }));
    }
    fn show_all(&self, viewer: &Player) {
        for (name, team) in &self.teams {
            team.send(viewer, name, 0, &team.entries);
        }
        for (name, (shown, _, scores)) in &self.objectives {
            self.show_objective(viewer, name, shown, 0);
            for (owner, score) in scores {
                viewer.send("sb_score", json!({ "owner": owner, "objective": name, "score": score }));
            }
        }
        for (slot, objective) in &self.display {
            viewer.send("sb_display", json!({ "slot": slot, "objective": objective }));
        }
    }
    fn hide_all(&self, viewer: &Player) {
        for slot in self.display.keys() {
            viewer.send("sb_display", json!({ "slot": slot, "objective": "" }));
        }
        for name in self.objectives.keys() {
            viewer.send("sb_objective", json!({ "name": name, "method": 1 }));
        }
        for (name, team) in &self.teams {
            team.send(viewer, name, 1, &[]);
        }
    }
    fn team_changed(&self, name: &str) {
        if let Some(team) = self.teams.get(name) {
            for viewer in &self.viewers {
                team.send(viewer, name, 2, &[]);
            }
        }
    }
}

impl PlayerScoreboard {
    fn board(&self) -> std::sync::MutexGuard<'_, Board> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
    /// One player's readings on this board.
    fn record(&self, owner: &str, readings: &BTreeMap<&str, Reading>) {
        let mut guard = self.board();
        let Board { objectives, viewers, counted, recorded, .. } = &mut *guard;
        let mut changed: Vec<(&str, i32)> = Vec::new();
        for (criteria, reading) in readings {
            if let Reading::Now(value) = reading
                && recorded.insert(((*criteria).to_owned(), owner.to_owned()), *value) != Some(*value)
            {
                changed.push((criteria, *value));
            }
        }
        for (name, (_, criteria, scores)) in objectives.iter_mut() {
            let score = match readings.get(criteria.as_str()) {
                Some(Reading::Now(_)) => match changed.iter().find(|(known, _)| *known == criteria.as_str()) {
                    Some((_, value)) => *value,
                    None => continue,
                },
                Some(Reading::Total(total)) => {
                    let last = counted.entry((name.clone(), owner.to_owned())).or_insert(*total);
                    let moved = i32::try_from(*total - *last).unwrap_or(0);
                    *last = *total;
                    if moved == 0 {
                        continue;
                    }
                    scores.get(owner).copied().unwrap_or(0).saturating_add(moved)
                }
                None => continue,
            };
            scores.insert(owner.to_owned(), score);
            for viewer in viewers.iter() {
                viewer.send("sb_score", json!({ "owner": owner, "objective": name, "score": score }));
            }
        }
    }
    /// One more for each objective of `criteria`.
    fn add_for(&self, criteria: &str, owner: &str) {
        let mut guard = self.board();
        let Board { objectives, viewers, .. } = &mut *guard;
        for (name, (_, kept, scores)) in objectives.iter_mut().filter(|(_, (_, kept, _))| kept == criteria) {
            let _ = kept;
            let score = scores.get(owner).copied().unwrap_or(0).saturating_add(1);
            scores.insert(owner.to_owned(), score);
            for viewer in viewers.iter() {
                viewer.send("sb_score", json!({ "owner": owner, "objective": name, "score": score }));
            }
        }
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.registerNewObjective
    ///
    /// Who's on now gets a score when a value next changes, as they've been recorded already.
    pub fn register_new_objective(&self, name: &str, criteria: &str, shown: &Component) -> bool {
        let mut board = self.board();
        if board.objectives.contains_key(name) {
            return false;
        }
        for player in crate::get_online_players() {
            if let Some(Reading::Now(value)) = read_criteria(criteria, &player) {
                board.recorded.entry((criteria.to_owned(), player.get_name().to_owned())).or_insert(value);
            }
        }
        if is_team_kill(criteria) {
            KILLS_READ.store(query(json!({ "op": "core_board_kills" }))["next"].as_u64().unwrap_or(0), Ordering::Relaxed);
        }
        board.objectives.insert(name.to_owned(), (shown.clone(), criteria.to_owned(), BTreeMap::new()));
        for viewer in &board.viewers {
            board.show_objective(viewer, name, shown, 0);
        }
        true
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getObjectives
    pub fn get_objectives(&self) -> Vec<String> {
        self.board().objectives.keys().cloned().collect()
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getObjective
    pub fn get_objective(&self, name: &str) -> Option<String> {
        self.board().objectives.contains_key(name).then(|| name.to_owned())
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getObjectivesByCriteria
    pub fn get_objectives_by_criteria(&self, criteria: &str) -> Vec<String> {
        self.board().objectives.iter().filter(|(_, (_, kept, _))| kept == criteria).map(|(name, _)| name.clone()).collect()
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getDisplayName, org/bukkit/scoreboard/Objective.displayName
    pub fn get_objective_display_name(&self, objective: &str) -> Option<Component> {
        self.board().objectives.get(objective).map(|(shown, _, _)| shown.clone())
    }
    /// bukkit: org/bukkit/scoreboard/Objective.setDisplayName
    pub fn set_objective_display_name(&self, objective: &str, shown: &Component) {
        let mut board = self.board();
        if let Some(row) = board.objectives.get_mut(objective) {
            row.0 = shown.clone();
            for viewer in &board.viewers {
                board.show_objective(viewer, objective, shown, 2);
            }
        }
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getCriteria
    pub fn get_objective_criteria(&self, objective: &str) -> Option<String> {
        self.board().objectives.get(objective).map(|(_, criteria, _)| criteria.clone())
    }
    /// bukkit: org/bukkit/scoreboard/Objective.setRenderType
    pub fn set_render_type(&self, objective: &str, render: &str) {
        let mut board = self.board();
        board.render.insert(objective.to_owned(), i32::from(render == "HEARTS"));
        if let Some((shown, _, _)) = board.objectives.get(objective) {
            for viewer in &board.viewers {
                board.show_objective(viewer, objective, shown, 2);
            }
        }
    }
    /// bukkit: org/bukkit/scoreboard/Objective.setDisplaySlot
    pub fn set_display_slot(&self, objective: &str, slot: &str) {
        let mut board = self.board();
        let number = slot_number(slot);
        board.display.insert(number, objective.to_owned());
        for viewer in &board.viewers {
            viewer.send("sb_display", json!({ "slot": number, "objective": objective }));
        }
    }
    /// bukkit: org/bukkit/scoreboard/Objective.getDisplaySlot
    pub fn get_display_slot(&self, objective: &str) -> Option<&'static str> {
        self.board().display.iter().find(|(_, shown)| *shown == objective).map(|(slot, _)| slot_name(*slot))
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.clearSlot
    pub fn clear_slot(&self, slot: &str) {
        let mut board = self.board();
        let number = slot_number(slot);
        board.display.remove(&number);
        for viewer in &board.viewers {
            viewer.send("sb_display", json!({ "slot": number, "objective": "" }));
        }
    }
    /// bukkit: org/bukkit/scoreboard/Score.setScore
    pub fn set_score(&self, objective: &str, owner: &str, score: i32) {
        let mut board = self.board();
        let Some((_, _, scores)) = board.objectives.get_mut(objective) else {
            return;
        };
        scores.insert(owner.to_owned(), score);
        for viewer in &board.viewers {
            viewer.send("sb_score", json!({ "owner": owner, "objective": objective, "score": score }));
        }
    }
    /// bukkit: org/bukkit/scoreboard/Score.getScore
    pub fn get_score(&self, objective: &str, owner: &str) -> Option<i32> {
        self.board().objectives.get(objective)?.2.get(owner).copied()
    }
    /// bukkit: org/bukkit/scoreboard/Score.isScoreSet
    pub fn is_score_set(&self, objective: &str, owner: &str) -> bool {
        self.get_score(objective, owner).is_some()
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getEntries
    pub fn get_entries(&self) -> Vec<String> {
        let mut entries: Vec<String> = self.board().objectives.values().flat_map(|(_, _, scores)| scores.keys().cloned().collect::<Vec<_>>()).collect();
        entries.sort_unstable();
        entries.dedup();
        entries
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.resetScores
    pub fn reset_scores(&self, owner: &str) {
        let mut board = self.board();
        for (_, _, scores) in board.objectives.values_mut() {
            scores.remove(owner);
        }
        for viewer in &board.viewers {
            viewer.send("sb_reset", json!({ "owner": owner }));
        }
    }
    /// bukkit: org/bukkit/scoreboard/Objective.unregister
    pub fn unregister_objective(&self, name: &str) {
        let mut board = self.board();
        board.objectives.remove(name);
        board.render.remove(name);
        board.display.retain(|_, objective| objective != name);
        for viewer in &board.viewers {
            viewer.send("sb_objective", json!({ "name": name, "method": 1 }));
        }
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.registerNewTeam
    ///
    /// False if the name is taken, where Java throws.
    pub fn register_new_team(&self, name: &str) -> bool {
        let mut board = self.board();
        if board.teams.contains_key(name) {
            return false;
        }
        let team = BoardTeam::new(name);
        for viewer in &board.viewers {
            team.send(viewer, name, 0, &[]);
        }
        board.teams.insert(name.to_owned(), team);
        true
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getTeams
    pub fn get_teams(&self) -> Vec<String> {
        self.board().teams.keys().cloned().collect()
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getTeam
    pub fn get_team(&self, name: &str) -> Option<String> {
        self.board().teams.contains_key(name).then(|| name.to_owned())
    }
    /// bukkit: org/bukkit/scoreboard/Scoreboard.getEntryTeam
    pub fn get_entry_team(&self, entry: &str) -> Option<String> {
        self.board().teams.iter().find(|(_, team)| team.entries.iter().any(|known| known == entry)).map(|(name, _)| name.clone())
    }
    /// bukkit: org/bukkit/scoreboard/Team.unregister
    pub fn unregister_team(&self, name: &str) {
        let mut board = self.board();
        if let Some(team) = board.teams.remove(name) {
            for viewer in &board.viewers {
                team.send(viewer, name, 1, &[]);
            }
        }
    }
    /// bukkit: org/bukkit/scoreboard/Team.addEntry
    ///
    /// Out of any team it was in, as Java's.
    pub fn add_team_entry(&self, name: &str, entry: &str) {
        if let Some(old) = self.get_entry_team(entry) {
            self.remove_team_entry(&old, entry);
        }
        let mut board = self.board();
        if let Some(team) = board.teams.get_mut(name) {
            team.entries.push(entry.to_owned());
            for viewer in &board.viewers {
                board.teams[name].send(viewer, name, 3, &[entry.to_owned()]);
            }
        }
    }
    /// bukkit: org/bukkit/scoreboard/Team.removeEntry
    pub fn remove_team_entry(&self, name: &str, entry: &str) -> bool {
        let mut board = self.board();
        let Some(team) = board.teams.get_mut(name) else {
            return false;
        };
        let before = team.entries.len();
        team.entries.retain(|known| known != entry);
        let removed = team.entries.len() != before;
        if removed {
            for viewer in &board.viewers {
                board.teams[name].send(viewer, name, 4, &[entry.to_owned()]);
            }
        }
        removed
    }
    /// bukkit: org/bukkit/scoreboard/Team.getEntries
    pub fn get_team_entries(&self, name: &str) -> Vec<String> {
        self.board().teams.get(name).map_or_else(Vec::new, |team| team.entries.clone())
    }
    fn change_team(&self, name: &str, change: impl FnOnce(&mut BoardTeam)) {
        let mut board = self.board();
        if let Some(team) = board.teams.get_mut(name) {
            change(team);
        }
        board.team_changed(name);
    }
    /// bukkit: org/bukkit/scoreboard/Team.displayName
    pub fn set_team_display_name(&self, name: &str, shown: &Component) {
        self.change_team(name, |team| team.display = shown.clone());
    }
    /// bukkit: org/bukkit/scoreboard/Team.prefix
    pub fn set_team_prefix(&self, name: &str, prefix: &Component) {
        self.change_team(name, |team| team.prefix = prefix.clone());
    }
    /// bukkit: org/bukkit/scoreboard/Team.suffix
    pub fn set_team_suffix(&self, name: &str, suffix: &Component) {
        self.change_team(name, |team| team.suffix = suffix.clone());
    }
    /// bukkit: org/bukkit/scoreboard/Team.setColor
    pub fn set_team_color(&self, name: &str, color: &str) {
        let index = TEAM_COLORS.iter().position(|known| known.eq_ignore_ascii_case(color)).map(|index| index as i32);
        self.change_team(name, |team| team.color = index);
    }
    /// bukkit: org/bukkit/scoreboard/Team.setAllowFriendlyFire
    pub fn set_team_friendly_fire(&self, name: &str, on: bool) {
        self.change_team(name, |team| team.friendly_fire = on);
    }
    /// bukkit: org/bukkit/scoreboard/Team.setCanSeeFriendlyInvisibles
    pub fn set_team_see_invisibles(&self, name: &str, on: bool) {
        self.change_team(name, |team| team.see_invisibles = on);
    }
    /// bukkit: org/bukkit/scoreboard/Team.setOption
    pub fn set_team_option(&self, name: &str, option: &str, status: &str) {
        self.change_team(name, |team| {
            team.options.insert(option.to_owned(), status.to_owned());
        });
    }
    fn show_to(&self, player: &Player) {
        let mut board = self.board();
        if !board.viewers.contains(player) {
            board.viewers.push(player.clone());
        }
        board.show_all(player);
    }
    fn take_off(&self, player: &Player) {
        let mut board = self.board();
        board.viewers.retain(|viewer| viewer != player);
        board.hide_all(player);
    }
}

impl Player {
    /// bukkit: org/bukkit/entity/Player.setScoreboard
    ///
    /// A board from `get_new_scoreboard`. `set_main_scoreboard` is the game's: Rust has one type for each.
    pub fn set_scoreboard(&self, board: &PlayerScoreboard) {
        let old = LOOKING.lock().unwrap_or_else(|e| e.into_inner()).remove(self.uuid());
        if let Some(old) = old {
            old.take_off(self);
        }
        if query(json!({ "op": "core_board_own", "uuid": self.uuid(), "own": true })).as_bool().unwrap_or(false) {
            LOOKING.lock().unwrap_or_else(|e| e.into_inner()).insert(self.uuid().to_owned(), board.clone());
            board.show_to(self);
        } else {
            TO_SHOW.lock().unwrap_or_else(|e| e.into_inner()).push((self.clone(), board.clone()));
        }
    }
    /// bukkit: org/bukkit/entity/Player.setScoreboard
    ///
    /// Back to the game's.
    pub fn set_main_scoreboard(&self) {
        let old = LOOKING.lock().unwrap_or_else(|e| e.into_inner()).remove(self.uuid());
        if let Some(old) = old {
            old.take_off(self);
        }
        query(json!({ "op": "core_board_own", "uuid": self.uuid(), "own": false }));
    }
    /// bukkit: org/bukkit/entity/Player.getScoreboard
    ///
    /// A plugin's board, if they're on one: `None` is the game's.
    pub fn get_own_scoreboard(&self) -> Option<PlayerScoreboard> {
        LOOKING.lock().unwrap_or_else(|e| e.into_inner()).get(self.uuid()).cloned()
    }
}

/// bukkit: org/bukkit/Bukkit.setMotd, org/bukkit/Server.setMotd
pub fn set_motd(motd: &str) -> bool {
    query(json!({ "op": "lead_motd", "motd": motd })).as_bool().unwrap_or(false)
}

/// bukkit: org/bukkit/Bukkit.setMaxPlayers, org/bukkit/Server.setMaxPlayers
pub fn set_max_players(count: u32) -> bool {
    query(json!({ "op": "lead_max_players", "count": count })).as_bool().unwrap_or(false)
}

fn spawn_key(category: &str) -> &'static str {
    match category {
        "MONSTER" => "Monster",
        "ANIMAL" => "Creature",
        "WATER_ANIMAL" => "WaterCreature",
        "WATER_AMBIENT" => "WaterAmbient",
        "WATER_UNDERGROUND_CREATURE" => "UndergroundWaterCreature",
        "AXOLOTL" => "Axolotls",
        "AMBIENT" => "Ambient",
        _ => "Misc",
    }
}

/// The limit for the whole server: `World::set_spawn_limit` is a level's own.
pub fn set_spawn_limit(category: &str, limit: i32) -> bool {
    query(json!({ "op": "lead_spawn_limit", "category": spawn_key(category), "limit": limit })).as_bool().unwrap_or(false)
}
