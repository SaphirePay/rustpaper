//! The rest of `org.bukkit.entity.Player` and `HumanEntity`: what goes to one player's connection, and
//! what a player's saved data or client settings answer.
use crate::data::{self, at, flag};
use crate::{Component, Location, Material, Player, adventure, get_online_players};
use horizon_mod::{Value, query, run_command};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// What Bukkit keeps per player and the game doesn't: headers, footers and display names.
#[derive(Clone, Default)]
struct Kept {
    display_name: Option<Value>,
    list_header: Option<Value>,
    list_footer: Option<Value>,
    compass: Option<Location>,
    cooldowns: BTreeMap<String, (u64, i32)>,
    sleeping_ignored: bool,
    packs: Vec<String>,
}

static KEPT: Mutex<BTreeMap<String, Kept>> = Mutex::new(BTreeMap::new());

impl Player {
    fn kept<R>(&self, f: impl FnOnce(&mut Kept) -> R) -> R {
        f(KEPT.lock().unwrap_or_else(|e| e.into_inner()).entry(self.uuid().to_owned()).or_default())
    }
    pub(crate) fn send(&self, packet: &str, fields: Value) -> bool {
        crate::fam_entity::note_send(self, packet, &fields);
        let mut request = fields;
        request["op"] = json!("player_send");
        request["uuid"] = json!(self.uuid());
        request["packet"] = json!(packet);
        query(request).as_bool().unwrap_or(false)
    }
    fn set(&self, field: &str, value: Value) -> bool {
        query(json!({ "op": "player_set", "uuid": self.uuid(), "field": field, "value": value })).as_bool().unwrap_or(false)
    }
    fn number(&self, key: &str) -> i64 {
        self.player_state()[key].as_i64().unwrap_or(0)
    }
    /// The tree the game saves for this player.
    pub fn get_data(&self) -> Value {
        query(json!({ "op": "entity", "uuid": self.uuid(), "data": true }))["data"].take()
    }
    /// One path of `get_data`.
    pub fn data_at(&self, path: &str) -> Value {
        at(&self.get_data(), path).clone()
    }

    // ---- The list and the name ----

    /// bukkit: org/bukkit/entity/Player.displayName
    pub fn display_name(&self) -> Component {
        self.kept(|kept| kept.display_name.clone()).map_or_else(|| Component::text(self.get_name()), Component)
    }
    /// bukkit: org/bukkit/entity/Player.setDisplayName
    pub fn set_display_name(&self, name: &str) {
        self.kept(|kept| kept.display_name = Some(adventure::legacy(name).0));
    }
    /// bukkit: org/bukkit/entity/Player.displayName
    pub fn set_display_name_component(&self, name: &Component) {
        self.kept(|kept| kept.display_name = Some(name.0.clone()));
    }
    fn send_list_text(&self) {
        let (header, footer) = self.kept(|kept| (kept.list_header.clone(), kept.list_footer.clone()));
        self.send("tab_list", json!({ "header": header.unwrap_or(json!("")), "footer": footer.unwrap_or(json!("")) }));
    }
    /// bukkit: net/kyori/adventure/audience/Audience.sendPlayerListHeaderAndFooter
    pub fn send_player_list_header_and_footer(&self, header: &Component, footer: &Component) {
        self.kept(|kept| {
            kept.list_header = Some(header.0.clone());
            kept.list_footer = Some(footer.0.clone());
        });
        self.send_list_text();
    }
    /// bukkit: org/bukkit/entity/Player.setPlayerListHeaderFooter
    pub fn set_player_list_header_footer(&self, header: &str, footer: &str) {
        self.kept(|kept| {
            kept.list_header = Some(adventure::legacy(header).0);
            kept.list_footer = Some(adventure::legacy(footer).0);
        });
        self.send_list_text();
    }
    /// bukkit: net/kyori/adventure/audience/Audience.sendPlayerListHeader
    pub fn send_player_list_header(&self, header: &Component) {
        self.kept(|kept| kept.list_header = Some(header.0.clone()));
        self.send_list_text();
    }
    /// bukkit: net/kyori/adventure/audience/Audience.sendPlayerListFooter
    pub fn send_player_list_footer(&self, footer: &Component) {
        self.kept(|kept| kept.list_footer = Some(footer.0.clone()));
        self.send_list_text();
    }
    /// bukkit: org/bukkit/entity/Player.setPlayerListHeader
    pub fn set_player_list_header(&self, header: &str) {
        self.kept(|kept| kept.list_header = Some(adventure::legacy(header).0));
        self.send_list_text();
    }
    /// bukkit: org/bukkit/entity/Player.setPlayerListFooter
    pub fn set_player_list_footer(&self, footer: &str) {
        self.kept(|kept| kept.list_footer = Some(adventure::legacy(footer).0));
        self.send_list_text();
    }
    /// bukkit: org/bukkit/entity/Player.playerListHeader
    pub fn player_list_header(&self) -> Option<Component> {
        self.kept(|kept| kept.list_header.clone()).map(Component)
    }
    /// bukkit: org/bukkit/entity/Player.playerListFooter
    pub fn player_list_footer(&self) -> Option<Component> {
        self.kept(|kept| kept.list_footer.clone()).map(Component)
    }
    /// bukkit: org/bukkit/entity/Player.getPlayerListHeader
    pub fn get_player_list_header(&self) -> Option<String> {
        self.player_list_header().map(|text| text.to_legacy())
    }
    /// bukkit: org/bukkit/entity/Player.getPlayerListFooter
    pub fn get_player_list_footer(&self) -> Option<String> {
        self.player_list_footer().map(|text| text.to_legacy())
    }

    // ---- Messages and commands ----

    /// bukkit: org/bukkit/entity/Player.sendRawMessage
    pub fn send_raw_message(&self, text: &str) {
        self.send_message(text);
    }
    /// bukkit: org/bukkit/entity/Player.chat
    pub fn chat(&self, line: &str) {
        match line.strip_prefix('/') {
            Some(command) => {
                self.perform_command(command);
            }
            None if query(json!({ "op": "ev_player_chat", "uuid": self.uuid(), "line": line })) == true => {}
            None => run_command(&format!("tellraw @a {}", json!({ "translate": "chat.type.text", "with": [self.get_name(), line] }))),
        }
    }
    /// bukkit: org/bukkit/entity/Player.updateCommands
    pub fn update_commands(&self) {
        query(json!({ "op": "resend_commands" }));
    }
    /// bukkit: org/bukkit/entity/Player.addCustomChatCompletions, org/bukkit/entity/Player.addAdditionalChatCompletions
    pub fn add_custom_chat_completions(&self, entries: &[&str]) {
        self.send("chat_completions", json!({ "action": 0, "entries": entries }));
    }
    /// bukkit: org/bukkit/entity/Player.removeCustomChatCompletions, org/bukkit/entity/Player.removeAdditionalChatCompletions
    pub fn remove_custom_chat_completions(&self, entries: &[&str]) {
        self.send("chat_completions", json!({ "action": 1, "entries": entries }));
    }
    /// bukkit: org/bukkit/entity/Player.setCustomChatCompletions
    pub fn set_custom_chat_completions(&self, entries: &[&str]) {
        self.send("chat_completions", json!({ "action": 2, "entries": entries }));
    }
    /// bukkit: org/bukkit/entity/Player.setTitleTimes
    pub fn set_title_times(&self, fade_in: u32, stay: u32, fade_out: u32) {
        run_command(&format!("title {} times {fade_in} {stay} {fade_out}", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/Player.setSubtitle
    pub fn set_subtitle(&self, text: &Component) {
        run_command(&format!("title {} subtitle {}", self.get_name(), text.0));
    }
    /// bukkit: org/bukkit/entity/Player.hideTitle
    pub fn hide_title(&self) {
        run_command(&format!("title {} clear", self.get_name()));
    }

    // ---- What only this player sees ----

    /// bukkit: org/bukkit/entity/Player.stopSound
    pub fn stop_sound(&self, sound: &str) {
        self.send("stop_sound", json!({ "sound": sound }));
    }
    /// bukkit: org/bukkit/entity/Player.stopAllSounds
    pub fn stop_all_sounds(&self) {
        self.send("stop_sound", json!({}));
    }
    /// bukkit: org/bukkit/entity/Player.playEffect
    pub fn play_effect(&self, at: &Location, effect: i32, data: i32) {
        self.send("level_event", json!({ "kind": effect, "pos": [at.get_block_x(), at.get_block_y(), at.get_block_z()], "data": data }));
    }
    /// bukkit: org/bukkit/entity/Player.sendBlockChange
    pub fn send_block_change(&self, at: &Location, block: &str) {
        self.send("block_update", json!({ "pos": [at.get_block_x(), at.get_block_y(), at.get_block_z()], "block": block }));
    }
    /// bukkit: org/bukkit/entity/Player.sendBlockChanges, org/bukkit/entity/Player.sendMultiBlockChange
    pub fn send_block_changes(&self, changes: &[(Location, String)]) {
        for (at, block) in changes {
            self.send_block_change(at, block);
        }
    }
    /// bukkit: org/bukkit/entity/Player.sendBlockDamage
    ///
    /// `progress` from 0 to 1, as Bukkit takes it; `source` tells cracks at one place apart.
    pub fn send_block_damage(&self, at: &Location, progress: f32, source: i32) {
        let stage = if progress <= 0.0 { 255 } else { (progress.min(1.0) * 9.0) as i32 };
        self.send("block_destruction", json!({ "id": source, "pos": [at.get_block_x(), at.get_block_y(), at.get_block_z()], "progress": stage }));
    }
    /// bukkit: org/bukkit/entity/Player.sendHurtAnimation
    pub fn send_hurt_animation(&self, yaw: f32) {
        self.send("hurt_animation", json!({ "id": self.get_entity_id(), "yaw": yaw }));
    }
    /// bukkit: org/bukkit/entity/Player.sendEntityEffect
    pub fn send_entity_effect(&self, effect: i8, entity: i64) {
        self.send("entity_event", json!({ "id": entity, "event": effect }));
    }
    /// bukkit: org/bukkit/entity/Player.sendOpLevel
    pub fn send_op_level(&self, level: i8) {
        self.send("entity_event", json!({ "id": self.get_entity_id(), "event": 24 + level.clamp(0, 4) }));
    }
    /// bukkit: org/bukkit/entity/Player.showWinScreen
    pub fn show_win_screen(&self) {
        self.send("game_event", json!({ "event": 4, "param": 1.0 }));
    }
    /// bukkit: org/bukkit/entity/Player.showDemoScreen
    pub fn show_demo_screen(&self) {
        self.send("game_event", json!({ "event": 5, "param": 0.0 }));
    }
    /// bukkit: org/bukkit/entity/Player.showElderGuardian
    pub fn show_elder_guardian(&self, silent: bool) {
        self.send("game_event", json!({ "event": 10, "param": if silent { 0.0 } else { 1.0 } }));
    }
    /// bukkit: org/bukkit/entity/Player.sendExperienceChange
    pub fn send_experience_change(&self, progress: f32, level: i32) {
        self.send("experience", json!({ "progress": progress, "level": level, "total": self.get_total_experience() }));
    }
    /// bukkit: org/bukkit/entity/Player.sendHealthUpdate
    pub fn send_health_update(&self, health: f64, food: i32, saturation: f32) {
        self.send("health", json!({ "health": health, "food": food, "saturation": saturation }));
    }
    /// bukkit: org/bukkit/entity/Player.openSign, org/bukkit/entity/HumanEntity.openSign
    pub fn open_sign(&self, at: &Location, front: bool) {
        self.send("open_sign", json!({ "pos": [at.get_block_x(), at.get_block_y(), at.get_block_z()], "front": front }));
    }

    // ---- The connection ----

    /// bukkit: org/bukkit/entity/Player.transfer
    pub fn transfer(&self, host: &str, port: i32) {
        self.send("transfer", json!({ "host": host, "port": port }));
    }
    /// bukkit: org/bukkit/entity/Player.storeCookie
    pub fn store_cookie(&self, key: &str, data: &[u8]) {
        self.send("store_cookie", json!({ "key": key, "data": data }));
    }
    /// bukkit: org/bukkit/entity/Player.getPing, org/bukkit/entity/Player$Spigot.getPing
    pub fn get_ping(&self) -> i32 {
        self.number("ping") as i32
    }
    /// bukkit: org/bukkit/entity/Player.getLocale, org/bukkit/entity/Player.locale
    pub fn get_locale(&self) -> String {
        self.player_state()["locale"].as_str().unwrap_or("en_us").to_owned()
    }
    /// bukkit: org/bukkit/entity/Player.getViewDistance, org/bukkit/entity/Player.getSendViewDistance
    pub fn get_view_distance(&self) -> i32 {
        self.number("server_view_distance") as i32
    }
    /// bukkit: org/bukkit/entity/Player.isAllowingServerListings
    pub fn is_allowing_server_listings(&self) -> bool {
        self.player_state()["allows_listing"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getMainHand
    pub fn get_main_hand(&self) -> &'static str {
        if self.number("main_hand") == 0 { "LEFT" } else { "RIGHT" }
    }
    /// bukkit: org/bukkit/entity/Player.getClientOption
    ///
    /// By its name in `ClientOption`: `LOCALE`, `VIEW_DISTANCE`, `CHAT_COLORS_ENABLED`, `MAIN_HAND`, `TEXT_FILTERING_ENABLED`, `ALLOW_SERVER_LISTINGS`.
    pub fn get_client_option(&self, option: &str) -> Value {
        let state = self.player_state();
        match option {
            "LOCALE" => state["locale"].clone(),
            "VIEW_DISTANCE" => state["view_distance"].clone(),
            "CHAT_COLORS_ENABLED" => state["chat_colors"].clone(),
            "CHAT_VISIBILITY" => state["chat_visibility"].clone(),
            "MAIN_HAND" => json!(self.get_main_hand()),
            "TEXT_FILTERING_ENABLED" => state["text_filtering"].clone(),
            "ALLOW_SERVER_LISTINGS" => state["allows_listing"].clone(),
            "SKIN_PARTS" => state["skin_parts"].clone(),
            "PARTICLE_VISIBILITY" => state["particle_status"].clone(),
            _ => Value::Null,
        }
    }

    // ---- The player list ----

    fn tell_everyone(&self, packet: &str, fields: serde_json::Value) {
        for viewer in crate::get_online_players() {
            let mut request = fields.clone();
            request["target"] = json!(self.uuid());
            viewer.send(packet, request);
        }
    }
    /// bukkit: org/bukkit/entity/Player.playerListName
    pub fn set_player_list_name_component(&self, name: Option<&Component>) {
        self.tell_everyone("list_name", json!({ "name": name.map(|name| &name.0) }));
    }
    /// bukkit: org/bukkit/entity/Player.setPlayerListName
    pub fn set_player_list_name(&self, name: Option<&str>) {
        self.set_player_list_name_component(name.map(adventure::legacy).as_ref());
    }
    /// bukkit: org/bukkit/entity/Player.setPlayerListOrder
    pub fn set_player_list_order(&self, order: i32) {
        self.tell_everyone("list_order", json!({ "order": order }));
    }
    /// bukkit: org/bukkit/entity/Player.unlistPlayer
    pub fn unlist_player(&self, other: &Self) -> bool {
        self.send("listed", json!({ "target": other.uuid(), "listed": false }))
    }
    /// bukkit: org/bukkit/entity/Player.listPlayer
    pub fn list_player(&self, other: &Self) -> bool {
        self.send("listed", json!({ "target": other.uuid(), "listed": true }))
    }

    // ---- This player's sky ----

    /// bukkit: org/bukkit/entity/Player.setPlayerTime
    pub fn set_player_time(&self, time: i64, relative: bool) {
        let world = self.get_world();
        self.send("player_time", json!({ "game_time": world.get_full_time(), "time": if relative { world.get_time() + time } else { time }, "relative": relative }));
    }
    /// bukkit: org/bukkit/entity/Player.resetPlayerTime
    pub fn reset_player_time(&self) {
        self.set_player_time(0, true);
    }
    /// bukkit: org/bukkit/entity/Player.setPlayerWeather
    ///
    /// The level's own stops reaching them, and the rain eases as `ServerPlayer.tickWeather` has it.
    pub fn set_player_weather(&self, weather: &str) {
        query(json!({ "op": "core_weather_own", "uuid": self.uuid(), "downfall": weather == "DOWNFALL" }));
        self.show_weather(weather);
    }
    fn show_weather(&self, weather: &str) {
        self.send("game_event", json!({ "event": if weather == "DOWNFALL" { 2 } else { 1 }, "param": 0.0 }));
    }
    /// bukkit: org/bukkit/entity/Player.resetPlayerWeather
    pub fn reset_player_weather(&self) {
        query(json!({ "op": "core_weather_own", "uuid": self.uuid() }));
        self.show_weather(if self.get_world().has_storm() { "DOWNFALL" } else { "CLEAR" });
        crate::fam_entity::note_weather_reset(self);
    }

    // ---- Flight ----

    /// bukkit: org/bukkit/entity/Player.getAllowFlight
    pub fn get_allow_flight(&self) -> bool {
        self.player_state()["allow_flight"].as_bool().unwrap_or(false)
    }
    /// bukkit: org/bukkit/entity/Player.setAllowFlight
    pub fn set_allow_flight(&self, allowed: bool) {
        self.set("allow_flight", json!(allowed));
    }
    /// bukkit: org/bukkit/entity/Player.setFlying
    pub fn set_flying(&self, flying: bool) {
        self.set("flying", json!(flying));
    }
    /// bukkit: org/bukkit/entity/Player.getFlySpeed
    pub fn get_fly_speed(&self) -> f32 {
        self.player_state()["fly_speed"].as_f64().unwrap_or(0.05) as f32 * 2.0
    }
    /// bukkit: org/bukkit/entity/Player.setFlySpeed
    ///
    /// From -1 to 1, as Java takes it: the client's own unit is half that.
    pub fn set_fly_speed(&self, speed: f32) {
        self.set("fly_speed", json!(speed / 2.0));
    }
    /// bukkit: org/bukkit/entity/Player.getWalkSpeed
    pub fn get_walk_speed(&self) -> f32 {
        self.player_state()["walk_speed"].as_f64().unwrap_or(0.1) as f32 * 2.0
    }
    /// bukkit: org/bukkit/entity/Player.setWalkSpeed
    pub fn set_walk_speed(&self, speed: f32) {
        self.set("walk_speed", json!(speed / 2.0));
    }

    // ---- Resource packs ----

    /// bukkit: org/bukkit/entity/Player.addResourcePack, net/kyori/adventure/audience/Audience.sendResourcePacks
    pub fn add_resource_pack(&self, id: &str, url: &str, hash: &str, prompt: Option<&Component>, required: bool) {
        let id = id.replace('-', "");
        self.kept(|kept| kept.packs.push(id.clone()));
        self.send("resource_pack_push", json!({ "id": id, "url": url, "hash": hash, "required": required, "prompt": prompt.map(|text| &text.0) }));
    }
    /// bukkit: org/bukkit/entity/Player.addResourcePack
    ///
    /// Bukkit's own shape: the SHA-1 as bytes and the prompt as legacy text (`§` codes).
    pub fn add_resource_pack_bytes(&self, id: &str, url: &str, hash: &[u8], prompt: Option<&str>, required: bool) {
        let hash: String = hash.iter().map(|byte| format!("{byte:02x}")).collect();
        let prompt = prompt.map(adventure::legacy);
        self.add_resource_pack(id, url, &hash, prompt.as_ref(), required);
    }
    /// bukkit: org/bukkit/entity/Player.setResourcePack, org/bukkit/entity/Player.setTexturePack
    pub fn set_resource_pack(&self, url: &str, hash: &str, required: bool) {
        self.remove_resource_packs();
        let id = format!("{:032x}", url.bytes().fold(0xcbf2_9ce4_8422_2325_u128, |all, byte| (all ^ u128::from(byte)).wrapping_mul(0x0100_0000_01b3)));
        self.add_resource_pack(&id, url, hash, None, required);
    }
    /// bukkit: org/bukkit/entity/Player.removeResourcePack, net/kyori/adventure/audience/Audience.removeResourcePacks
    pub fn remove_resource_pack(&self, id: &str) {
        let id = id.replace('-', "");
        self.kept(|kept| kept.packs.retain(|known| *known != id));
        self.send("resource_pack_pop", json!({ "id": id }));
    }
    /// bukkit: org/bukkit/entity/Player.removeResourcePacks, net/kyori/adventure/audience/Audience.clearResourcePacks
    pub fn remove_resource_packs(&self) {
        self.kept(|kept| kept.packs.clear());
        self.send("resource_pack_pop", json!({}));
    }
    /// bukkit: org/bukkit/entity/Player.hasResourcePack
    pub fn has_resource_pack(&self) -> bool {
        self.kept(|kept| !kept.packs.is_empty())
    }

    // ---- Experience ----

    /// bukkit: org/bukkit/entity/Player.giveExp
    pub fn give_exp(&self, amount: i32) {
        run_command(&format!("xp add {} {amount} points", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/Player.giveExpLevels
    pub fn give_exp_levels(&self, amount: i32) {
        run_command(&format!("xp add {} {amount} levels", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/Player.setLevel
    pub fn set_level(&self, level: i32) {
        run_command(&format!("xp set {} {level} levels", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/Player.setExp
    pub fn set_exp(&self, progress: f32) {
        self.set("exp_progress", json!(progress));
    }
    /// bukkit: org/bukkit/entity/Player.setTotalExperience
    pub fn set_total_experience(&self, total: i32) {
        self.set("exp_total", json!(total));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getExpToLevel, org/bukkit/entity/Player.getExperiencePointsNeededForNextLevel
    pub fn get_exp_to_level(&self) -> i32 {
        match self.get_level() {
            level if level >= 30 => 112 + (level - 30) * 9,
            level if level >= 15 => 37 + (level - 15) * 5,
            level => 7 + level * 2,
        }
    }
    /// bukkit: org/bukkit/entity/Player.calculateTotalExperiencePoints
    pub fn calculate_total_experience_points(&self) -> i32 {
        let level = self.get_level();
        let whole = match level {
            0..=16 => level * level + 6 * level,
            17..=31 => (2.5 * f64::from(level * level) - 40.5 * f64::from(level) + 360.0) as i32,
            _ => (4.5 * f64::from(level * level) - 162.5 * f64::from(level) + 2220.0) as i32,
        };
        whole + (self.get_exp() * self.get_exp_to_level() as f32).round() as i32
    }

    // ---- The body ----

    /// bukkit: org/bukkit/entity/HumanEntity.setFoodLevel, org/bukkit/entity/Player.setFoodLevel
    pub fn set_food_level(&self, food: i32) {
        self.set("food", json!(food));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setSaturation, org/bukkit/entity/Player.setSaturation
    pub fn set_saturation(&self, saturation: f32) {
        self.set("saturation", json!(saturation));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setExhaustion
    pub fn set_exhaustion(&self, exhaustion: f32) {
        self.set("exhaustion", json!(exhaustion));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getExhaustion
    pub fn get_exhaustion(&self) -> f32 {
        self.data_at("foodExhaustionLevel").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/entity/HumanEntity.isBlocking, org/bukkit/entity/HumanEntity.isHandRaised, org/bukkit/entity/LivingEntity.isHandRaised
    pub fn is_hand_raised(&self) -> bool {
        self.number("living_flags") & 1 != 0
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getSleepTicks
    pub fn get_sleep_ticks(&self) -> i32 {
        self.number("sleep_timer") as i32
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getPotentialBedLocation, org/bukkit/entity/HumanEntity.getBedLocation
    pub fn get_bed_location(&self) -> Option<Location> {
        let state = self.player_state();
        let at = |i: usize| state["bed"][i].as_f64();
        Some(Location::new(self.get_world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/entity/Player.getRespawnLocation, org/bukkit/entity/HumanEntity.getPotentialRespawnLocation
    pub fn get_respawn_location(&self) -> Option<Location> {
        let state = self.player_state();
        // A bed or anchor in another world keeps its own place; `spawn` is for the overworld alone.
        if let Some(level) = state["respawn_level"].as_str() {
            let at = |i: usize| state["respawn_pos"][i].as_f64();
            return Some(Location::new(crate::World::of(level), at(0)?, at(1)?, at(2)?, 0.0, 0.0));
        }
        let at = |i: usize| state["spawn"][i].as_f64();
        Some(Location::new(self.get_world(), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/entity/Player.setRespawnLocation, org/bukkit/entity/Player.setBedSpawnLocation
    pub fn set_respawn_location(&self, to: &Location) {
        run_command(&format!("execute in {} run spawnpoint {} {} {} {}", to.world.dimension(), self.get_name(), to.get_block_x(), to.get_block_y(), to.get_block_z()));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getLastDeathLocation
    pub fn get_last_death_location(&self) -> Option<Location> {
        let death = self.data_at("LastDeathLocation");
        let pos = &death["pos"];
        let at = |i: usize| pos[i].as_f64();
        Some(Location::new(crate::World::of(death["dimension"].as_str()?), at(0)?, at(1)?, at(2)?, 0.0, 0.0))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getShoulderEntityLeft
    pub fn get_shoulder_entity_left(&self) -> Option<Value> {
        Some(self.data_at("ShoulderEntityLeft")).filter(|entity| !entity.is_null())
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getShoulderEntityRight
    pub fn get_shoulder_entity_right(&self) -> Option<Value> {
        Some(self.data_at("ShoulderEntityRight")).filter(|entity| !entity.is_null())
    }
    /// bukkit: org/bukkit/entity/HumanEntity.discoverRecipe
    pub fn discover_recipe(&self, key: &str) {
        run_command(&format!("recipe give {} {key}", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.discoverRecipes
    pub fn discover_recipes(&self, keys: &[&str]) {
        keys.iter().for_each(|key| self.discover_recipe(key));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.undiscoverRecipe
    pub fn undiscover_recipe(&self, key: &str) {
        run_command(&format!("recipe take {} {key}", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.undiscoverRecipes
    pub fn undiscover_recipes(&self, keys: &[&str]) {
        keys.iter().for_each(|key| self.undiscover_recipe(key));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getDiscoveredRecipes
    pub fn get_discovered_recipes(&self) -> Vec<String> {
        self.data_at("recipeBook.recipes").as_array().into_iter().flatten().filter_map(|key| key.as_str().map(str::to_owned)).collect()
    }
    /// bukkit: org/bukkit/entity/HumanEntity.hasDiscoveredRecipe
    pub fn has_discovered_recipe(&self, key: &str) -> bool {
        self.get_discovered_recipes().iter().any(|known| known == key)
    }
    /// bukkit: org/bukkit/entity/HumanEntity.setCooldown
    pub fn set_cooldown(&self, material: &Material, ticks: i32) {
        let now = crate::scheduler::now();
        self.kept(|kept| kept.cooldowns.insert(material.key().to_owned(), (now, ticks)));
        self.send("cooldown", json!({ "group": material.key(), "ticks": ticks }));
    }
    /// bukkit: org/bukkit/entity/HumanEntity.getCooldown
    pub fn get_cooldown(&self, material: &Material) -> i32 {
        let now = crate::scheduler::now();
        self.kept(|kept| kept.cooldowns.get(material.key()).map_or(0, |(from, ticks)| (*ticks - (now - from) as i32).max(0)))
    }
    /// bukkit: org/bukkit/entity/HumanEntity.hasCooldown
    pub fn has_cooldown(&self, material: &Material) -> bool {
        self.get_cooldown(material) > 0
    }
    /// bukkit: org/bukkit/entity/Player.setSleepingIgnored
    pub fn set_sleeping_ignored(&self, ignored: bool) {
        self.kept(|kept| kept.sleeping_ignored = ignored);
    }
    /// bukkit: org/bukkit/entity/Player.isSleepingIgnored
    pub fn is_sleeping_ignored(&self) -> bool {
        self.kept(|kept| kept.sleeping_ignored)
    }
    /// bukkit: org/bukkit/entity/Player.setCompassTarget
    pub fn set_compass_target(&self, to: &Location) {
        self.kept(|kept| kept.compass = Some(to.clone()));
        self.send("spawn_position", json!({ "dimension": to.world.dimension(), "pos": [to.get_block_x(), to.get_block_y(), to.get_block_z()] }));
    }
    /// bukkit: org/bukkit/entity/Player.getCompassTarget
    pub fn get_compass_target(&self) -> Option<Location> {
        self.kept(|kept| kept.compass.clone()).or_else(|| Some(self.get_world().get_spawn_location()))
    }
    /// bukkit: org/bukkit/entity/Player.setRotation, org/bukkit/entity/Entity.setRotation
    pub fn set_rotation(&self, yaw: f32, pitch: f32) {
        run_command(&format!("tp {} ~ ~ ~ {yaw} {pitch}", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/Player.lookAt
    pub fn look_at(&self, x: f64, y: f64, z: f64) {
        run_command(&format!("execute as {0} at {0} run tp @s ~ ~ ~ facing {x} {y} {z}", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/Player.setSpectatorTarget
    pub fn set_spectator_target(&self, target: Option<&crate::entity::Entity>) {
        match target {
            Some(entity) => run_command(&format!("spectate {} {}", entity.get_unique_id(), self.get_name())),
            None => run_command(&format!("spectate @s {}", self.get_name())),
        }
    }
    /// bukkit: org/bukkit/entity/Player.resetCooldown
    pub fn reset_attack_cooldown(&self) {
        self.send("entity_event", json!({ "id": self.get_entity_id(), "event": 0 }));
    }
}

/// The ops, as `Bukkit.getOperators` gives them.
pub fn online_ops() -> Vec<Player> {
    get_online_players().into_iter().filter(Player::is_op).collect()
}

/// A flag of a saved compound, for callers of `get_data`.
pub fn saved_flag(data: &Value, path: &str) -> bool {
    flag(at(data, path))
}

/// bukkit: org/bukkit/entity/Player.getPlayerProfile, org/bukkit/profile/PlayerProfile, com/destroystokyo/paper/profile/PlayerProfile
///
/// A name, an id and the signed properties, `textures` among them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerProfile {
    pub name: String,
    pub id: String,
    pub properties: Vec<(String, String, Option<String>)>,
}

impl Player {
    /// bukkit: org/bukkit/entity/Player.getPlayerProfile, org/bukkit/OfflinePlayer.getPlayerProfile
    pub fn get_player_profile(&self) -> PlayerProfile {
        let state = self.player_state();
        let properties = state["profile"].as_array().into_iter().flatten().map(|property| (property["name"].as_str().unwrap_or_default().to_owned(), property["value"].as_str().unwrap_or_default().to_owned(), property["signature"].as_str().map(str::to_owned))).collect();
        PlayerProfile { name: self.get_name().to_owned(), id: self.get_unique_id(), properties }
    }
}

impl PlayerProfile {
    /// bukkit: org/bukkit/profile/PlayerProfile.getName, com/destroystokyo/paper/profile/PlayerProfile.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: org/bukkit/profile/PlayerProfile.getUniqueId, com/destroystokyo/paper/profile/PlayerProfile.getId
    pub fn get_unique_id(&self) -> &str {
        &self.id
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.getProperties
    pub fn get_properties(&self) -> &[(String, String, Option<String>)] {
        &self.properties
    }
    /// bukkit: com/destroystokyo/paper/profile/PlayerProfile.hasTextures
    pub fn has_textures(&self) -> bool {
        self.properties.iter().any(|(name, ..)| name == "textures")
    }
}

/// Keeps the import of `data` used by a doc link: `data::snbt` writes what these calls send.
pub use data::snbt as saved_text;
