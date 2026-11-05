//! `org.bukkit.Bukkit` and `org.bukkit.Server`: answered by the game, by the server's own files
//! (`server.properties`, `ops.json`, `whitelist.json`, `banned-players.json`), or by what a command does.
use crate::{Component, OfflinePlayer, Player, World, adventure, get_online_players, get_world};
use horizon_mod::{Value, query, run_command};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The folder the server runs in: above `plugins/<name>`.
pub fn server_folder() -> PathBuf {
    std::env::current_dir().unwrap_or_default()
}

fn properties() -> BTreeMap<String, String> {
    let text = std::fs::read_to_string(server_folder().join("server.properties")).unwrap_or_default();
    text.lines().filter(|line| !line.starts_with('#')).filter_map(|line| line.split_once('=')).map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned())).collect()
}

pub(crate) fn property(key: &str) -> Option<String> {
    crate::fam_core::live_property(key).or_else(|| properties().remove(key))
}

fn number(key: &str, default: i32) -> i32 {
    property(key).and_then(|value| value.parse().ok()).unwrap_or(default)
}

fn flag(key: &str, default: bool) -> bool {
    property(key).map_or(default, |value| value == "true")
}

macro_rules! properties {
    ($($(#[$doc:meta])* $name:ident, $kind:tt, $key:literal, $default:expr;)*) => {
        $($(#[$doc])* pub fn $name() -> properties!(@type $kind) { properties!(@read $kind, $key, $default) })*
    };
    (@type int) => { i32 };
    (@type flag) => { bool };
    (@type text) => { String };
    (@read int, $key:literal, $default:expr) => { number($key, $default) };
    (@read flag, $key:literal, $default:expr) => { flag($key, $default) };
    (@read text, $key:literal, $default:expr) => { property($key).unwrap_or_else(|| $default.to_owned()) };
}

properties! {
    /// bukkit: org/bukkit/Bukkit.getMaxPlayers, org/bukkit/Server.getMaxPlayers
    get_max_players, int, "max-players", 20;
    /// bukkit: org/bukkit/Bukkit.getPort, org/bukkit/Server.getPort
    get_port, int, "server-port", 25565;
    /// bukkit: org/bukkit/Bukkit.getViewDistance, org/bukkit/Server.getViewDistance
    get_view_distance, int, "view-distance", 10;
    /// bukkit: org/bukkit/Bukkit.getSimulationDistance, org/bukkit/Server.getSimulationDistance
    get_simulation_distance, int, "simulation-distance", 10;
    /// bukkit: org/bukkit/Bukkit.getIp, org/bukkit/Server.getIp
    get_ip, text, "server-ip", "";
    /// bukkit: org/bukkit/Bukkit.getWorldType, org/bukkit/Server.getWorldType
    get_world_type, text, "level-type", "minecraft:normal";
    /// bukkit: org/bukkit/Bukkit.getGenerateStructures, org/bukkit/Server.getGenerateStructures
    get_generate_structures, flag, "generate-structures", true;
    /// bukkit: org/bukkit/Bukkit.getMaxWorldSize, org/bukkit/Server.getMaxWorldSize
    get_max_world_size, int, "max-world-size", 29_999_984;
    /// bukkit: org/bukkit/Bukkit.getOnlineMode, org/bukkit/Server.getOnlineMode
    get_online_mode, flag, "online-mode", true;
    /// bukkit: org/bukkit/Bukkit.getAllowFlight, org/bukkit/Server.getAllowFlight
    get_allow_flight, flag, "allow-flight", false;
    /// bukkit: org/bukkit/Bukkit.isHardcore, org/bukkit/Server.isHardcore
    is_hardcore, flag, "hardcore", false;
    /// bukkit: org/bukkit/Bukkit.hasWhitelist, org/bukkit/Server.hasWhitelist
    has_whitelist, flag, "white-list", false;
    /// bukkit: org/bukkit/Bukkit.isWhitelistEnforced, org/bukkit/Server.isWhitelistEnforced
    is_whitelist_enforced, flag, "enforce-whitelist", false;
    /// bukkit: org/bukkit/Bukkit.getMotd, org/bukkit/Server.getMotd
    get_motd, text, "motd", "A Minecraft Server";
    /// bukkit: org/bukkit/Bukkit.getResourcePack, org/bukkit/Server.getResourcePack
    get_resource_pack, text, "resource-pack", "";
    /// bukkit: org/bukkit/Bukkit.getResourcePackHash, org/bukkit/Server.getResourcePackHash
    get_resource_pack_hash, text, "resource-pack-sha1", "";
    /// bukkit: org/bukkit/Bukkit.getResourcePackPrompt, org/bukkit/Server.getResourcePackPrompt
    get_resource_pack_prompt, text, "resource-pack-prompt", "";
    /// bukkit: org/bukkit/Bukkit.isResourcePackRequired, org/bukkit/Server.isResourcePackRequired
    is_resource_pack_required, flag, "require-resource-pack", false;
    /// bukkit: org/bukkit/Bukkit.isLoggingIPs, org/bukkit/Server.isLoggingIPs
    is_logging_ips, flag, "log-ips", true;
    /// bukkit: org/bukkit/Bukkit.getSpawnRadius, org/bukkit/Server.getSpawnRadius
    get_spawn_radius, int, "spawn-protection", 16;
    /// bukkit: org/bukkit/Bukkit.isEnforcingSecureProfiles, org/bukkit/Server.isEnforcingSecureProfiles
    is_enforcing_secure_profiles, flag, "enforce-secure-profile", true;
    /// bukkit: org/bukkit/Bukkit.isAcceptingTransfers, org/bukkit/Server.isAcceptingTransfers
    is_accepting_transfers, flag, "accepts-transfers", false;
    /// bukkit: org/bukkit/Bukkit.getHideOnlinePlayers, org/bukkit/Server.getHideOnlinePlayers
    get_hide_online_players, flag, "hide-online-players", false;
    /// bukkit: org/bukkit/Bukkit.getIdleTimeout, org/bukkit/Server.getIdleTimeout
    get_idle_timeout, int, "player-idle-timeout", 0;
    /// bukkit: org/bukkit/Bukkit.getMaxChainedNeighborUpdates, org/bukkit/Server.getMaxChainedNeighborUpdates
    get_max_chained_neighbor_updates, int, "max-chained-neighbor-updates", 1_000_000;
    /// bukkit: org/bukkit/Bukkit.getPauseWhenEmptyTime, org/bukkit/Server.getPauseWhenEmptyTime
    get_pause_when_empty_time, int, "pause-when-empty-seconds", 60;
}

/// bukkit: org/bukkit/Bukkit.motd, org/bukkit/Server.motd
pub fn motd() -> Component {
    adventure::legacy(&get_motd())
}

/// bukkit: org/bukkit/Bukkit.getMinecraftVersion, org/bukkit/Server.getMinecraftVersion
pub fn get_minecraft_version() -> String {
    query(json!({ "op": "version" }))["minecraft"].as_str().unwrap_or("26.3").to_owned()
}

/// bukkit: org/bukkit/Bukkit.getPluginsFolder, org/bukkit/Server.getPluginsFolder
pub fn get_plugins_folder() -> PathBuf {
    server_folder().join("plugins")
}

/// bukkit: org/bukkit/Bukkit.getWorldContainer, org/bukkit/Server.getWorldContainer
pub fn get_world_container() -> PathBuf {
    server_folder()
}

/// bukkit: org/bukkit/Bukkit.getUpdateFolder, org/bukkit/Server.getUpdateFolder
pub fn get_update_folder() -> &'static str {
    "update"
}

/// bukkit: org/bukkit/Bukkit.getUpdateFolderFile, org/bukkit/Server.getUpdateFolderFile
pub fn get_update_folder_file() -> PathBuf {
    get_plugins_folder().join(get_update_folder())
}

fn profiles(file: &str) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(server_folder().join(file)).unwrap_or_default();
    serde_json::from_str::<Value>(&text).unwrap_or_default().as_array().into_iter().flatten().filter_map(|entry| Some((entry["name"].as_str()?.to_owned(), entry["uuid"].as_str()?.to_owned()))).collect()
}

fn offline(names: Vec<(String, String)>) -> Vec<OfflinePlayer> {
    names.iter().map(|(name, _)| crate::get_offline_player(name)).collect()
}

/// bukkit: org/bukkit/Bukkit.getOperators, org/bukkit/Server.getOperators
pub fn get_operators() -> Vec<OfflinePlayer> {
    offline(profiles("ops.json"))
}

/// bukkit: org/bukkit/Bukkit.getWhitelistedPlayers, org/bukkit/Server.getWhitelistedPlayers
pub fn get_whitelisted_players() -> Vec<OfflinePlayer> {
    offline(profiles("whitelist.json"))
}

/// bukkit: org/bukkit/Bukkit.getBannedPlayers, org/bukkit/Server.getBannedPlayers
pub fn get_banned_players() -> Vec<OfflinePlayer> {
    offline(profiles("banned-players.json"))
}

/// bukkit: org/bukkit/Bukkit.getIPBans, org/bukkit/Server.getIPBans
pub fn get_ip_bans() -> Vec<String> {
    let text = std::fs::read_to_string(server_folder().join("banned-ips.json")).unwrap_or_default();
    serde_json::from_str::<Value>(&text).unwrap_or_default().as_array().into_iter().flatten().filter_map(|entry| entry["ip"].as_str().map(str::to_owned)).collect()
}

/// bukkit: org/bukkit/Bukkit.banIP, org/bukkit/Server.banIP
pub fn ban_ip(address: &str) {
    run_command(&format!("ban-ip {address}"));
}

/// bukkit: org/bukkit/Bukkit.unbanIP, org/bukkit/Server.unbanIP
pub fn unban_ip(address: &str) {
    run_command(&format!("pardon-ip {address}"));
}

/// bukkit: org/bukkit/Bukkit.setWhitelist, org/bukkit/Server.setWhitelist
pub fn set_whitelist(on: bool) {
    run_command(if on { "whitelist on" } else { "whitelist off" });
}

/// bukkit: org/bukkit/Bukkit.reloadWhitelist, org/bukkit/Server.reloadWhitelist
pub fn reload_whitelist() {
    run_command("whitelist reload");
}

/// bukkit: org/bukkit/Bukkit.savePlayers, org/bukkit/Server.savePlayers
pub fn save_players() {
    run_command("save-all");
}

/// bukkit: org/bukkit/Bukkit.shutdown, org/bukkit/Server.shutdown
pub fn shutdown() {
    run_command("stop");
}

/// bukkit: org/bukkit/Bukkit.reloadData, org/bukkit/Server.reloadData
pub fn reload_data() {
    horizon_mod::query(serde_json::json!({ "op": "ev_world_free_reload_plugin" }));
    run_command("reload");
}

/// bukkit: org/bukkit/Bukkit.getDefaultGameMode, org/bukkit/Server.getDefaultGameMode
pub fn get_default_game_mode() -> String {
    property("gamemode").unwrap_or_else(|| "survival".to_owned()).to_ascii_uppercase()
}

/// bukkit: org/bukkit/Bukkit.setDefaultGameMode, org/bukkit/Server.setDefaultGameMode
pub fn set_default_game_mode(mode: &str) {
    run_command(&format!("defaultgamemode {}", mode.to_ascii_lowercase()));
}

/// bukkit: org/bukkit/Bukkit.getCurrentTick
pub fn get_current_tick() -> u64 {
    query(json!({ "op": "tick_times" }))["ticks"].as_u64().unwrap_or(0)
}

/// bukkit: org/bukkit/Bukkit.getAverageTickTime, org/bukkit/Server.getAverageTickTime
pub fn get_average_tick_time() -> f64 {
    // Paper's: the mean of the last hundred ticks' times, in milliseconds. The client's smoothed chart
    // is only for when the server keeps no log (it reads 0 on a dedicated server).
    let times = crate::fam_core::get_tick_times();
    if !times.is_empty() {
        return times.iter().sum::<i64>() as f64 / times.len() as f64 / 1_000_000.0;
    }
    query(json!({ "op": "tick_times" }))["average"].as_f64().unwrap_or(0.0)
}

/// bukkit: org/bukkit/Bukkit.getTPS, org/bukkit/Server.getTPS
pub fn get_tps() -> [f64; 3] {
    crate::fam_core::tick_rates()
}

/// bukkit: org/bukkit/Bukkit.getRespawnWorld, org/bukkit/Server.getRespawnWorld
pub fn get_respawn_world() -> Option<World> {
    query(json!({ "op": "core_respawn_world" })).as_str().map(World::of).or_else(|| get_world("world"))
}

/// bukkit: org/bukkit/Bukkit.isGlobalTickThread, org/bukkit/Server.isGlobalTickThread, org/bukkit/Bukkit.isTickingWorlds, org/bukkit/Server.isTickingWorlds
pub fn is_global_tick_thread() -> bool {
    crate::is_primary_thread()
}

/// bukkit: org/bukkit/Bukkit.isOwnedByCurrentRegion, org/bukkit/Server.isOwnedByCurrentRegion
///
/// One thread runs every region, on Paper too.
pub fn is_owned_by_current_region() -> bool {
    crate::is_primary_thread()
}

/// bukkit: org/bukkit/Bukkit.getPlayerUniqueId, org/bukkit/Server.getPlayerUniqueId
pub fn get_player_unique_id(name: &str) -> Option<String> {
    crate::get_player(name).map(|player| player.get_unique_id()).or_else(|| crate::get_offline_player(name).get_unique_id().map(str::to_owned))
}

/// bukkit: org/bukkit/Bukkit.getOfflinePlayers, org/bukkit/Server.getOfflinePlayers
pub fn get_offline_players() -> Vec<OfflinePlayer> {
    let text = std::fs::read_to_string(server_folder().join("usercache.json")).unwrap_or_default();
    serde_json::from_str::<Value>(&text).unwrap_or_default().as_array().into_iter().flatten().filter_map(|entry| Some(crate::get_offline_player(entry["name"].as_str()?))).collect()
}

/// bukkit: org/bukkit/Bukkit.broadcast, org/bukkit/Server.broadcast, net/kyori/adventure/audience/Audience.sendMessage
pub fn broadcast(text: &Component) {
    if crate::fam_ev_world::broadcast_goes_out(&text.plain()) {
        run_command(&format!("tellraw @a {}", text.0));
    }
}

/// bukkit: org/bukkit/Server.sendPlainMessage
pub fn send_plain_message(text: &str) {
    broadcast(&Component::text(text));
}

/// bukkit: org/bukkit/Server.sendRichMessage, org/bukkit/command/CommandSender.sendRichMessage
pub fn send_rich_message(text: &str) {
    broadcast(&adventure::mini_message(text));
}

/// bukkit: org/bukkit/Bukkit.selectEntities, org/bukkit/Server.selectEntities
///
/// Only `@a` and `@p`-free selectors of players go by name here; a plain name is a player.
pub fn select_players(selector: &str) -> Vec<Player> {
    match selector {
        "@a" => get_online_players(),
        name => crate::get_player(name).into_iter().collect(),
    }
}

impl OfflinePlayer {
    fn stored(&self, list: &str) -> bool {
        profiles(list).iter().any(|(name, _)| name.eq_ignore_ascii_case(self.get_name()))
    }
    /// bukkit: org/bukkit/OfflinePlayer.isBanned
    pub fn is_banned(&self) -> bool {
        self.stored("banned-players.json")
    }
    /// bukkit: org/bukkit/OfflinePlayer.isWhitelisted
    pub fn is_whitelisted(&self) -> bool {
        self.stored("whitelist.json")
    }
    /// bukkit: org/bukkit/OfflinePlayer.setWhitelisted
    pub fn set_whitelisted(&self, on: bool) {
        run_command(&format!("whitelist {} {}", if on { "add" } else { "remove" }, self.get_name()));
    }
    /// bukkit: org/bukkit/OfflinePlayer.isOp
    pub fn is_op(&self) -> bool {
        self.stored("ops.json")
    }
    /// bukkit: org/bukkit/OfflinePlayer.setOp, org/bukkit/permissions/ServerOperator.setOp
    pub fn set_op(&self, on: bool) {
        run_command(&format!("{} {}", if on { "op" } else { "deop" }, self.get_name()));
    }
    /// bukkit: org/bukkit/OfflinePlayer.isConnected
    pub fn is_connected(&self) -> bool {
        self.is_online()
    }
}

/// bukkit: org/bukkit/plugin/PluginDescriptionFile.getName, org/bukkit/plugin/PluginDescriptionFile.getFullName
pub fn describe(plugin: &crate::Plugin) -> String {
    format!("{} v{}", plugin.get_name(), plugin.get_version())
}
