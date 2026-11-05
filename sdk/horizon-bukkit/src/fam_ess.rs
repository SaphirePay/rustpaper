//! What the port of EssentialsX found missing. Each of these was a call the Java makes and nothing
//! here answered.
use crate::server::get_world_container;
use crate::{Block, Material, Player, World, get_online_players};
use horizon_mod::{query, run_command};
use serde_json::{Value, json};

impl World {
    /// bukkit: org/bukkit/World.getUID
    ///
    /// `uid.dat` in the world's folder, two big-endian longs, made at first use as CraftBukkit makes it.
    pub fn get_uid(&self) -> String {
        let file = get_world_container().join(self.get_name()).join("uid.dat");
        let bytes = std::fs::read(&file).ok().filter(|bytes| bytes.len() >= 16).unwrap_or_else(|| {
            let bytes = random_uuid();
            if let Some(parent) = file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&file, bytes);
            bytes.to_vec()
        });
        let hex: String = bytes[..16].iter().map(|byte| format!("{byte:02x}")).collect();
        format!("{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..])
    }
}

impl crate::blocks::Sign {
    fn side(front: bool) -> &'static str {
        if front { "front_text" } else { "back_text" }
    }
    /// bukkit: org/bukkit/block/Sign.getSide
    ///
    /// The four lines of one side, as section sign text.
    pub fn get_side_lines(&self, front: bool) -> Vec<String> {
        let saved = self.0.saved(&format!("{}.messages", Self::side(front)));
        let mut lines: Vec<String> = saved.as_array().into_iter().flatten().map(|line| crate::Component(line.clone()).to_legacy()).collect();
        lines.resize(4, String::new());
        lines
    }
    /// bukkit: org/bukkit/block/sign/SignSide.setLine
    ///
    /// All four lines of one side, for `update`.
    pub fn set_side_lines(&mut self, front: bool, lines: &[String]) {
        let saved: Vec<String> = lines.iter().map(|line| crate::data::snbt(&crate::adventure::legacy(line).0)).collect();
        self.0.change(&format!("{}.messages", Self::side(front)), format!("[{}]", saved.join(",")));
    }
}

impl crate::block_data::TileState {
    /// bukkit: org/bukkit/block/CreatureSpawner.setSpawnedType, org/bukkit/block/CreatureSpawner.setDelay
    ///
    /// `entity` with its namespace. A `delay` above zero also sets the spawner's minimum and maximum.
    pub fn set_spawned_entity(&mut self, entity: &str, delay: i32) {
        self.change("SpawnData.entity.id", format!("\"{entity}\""));
        self.change("SpawnPotentials", "[]".to_owned());
        self.change("Delay", format!("{delay}s"));
        if delay > 0 {
            self.change("MinSpawnDelay", format!("{delay}s"));
            self.change("MaxSpawnDelay", format!("{delay}s"));
        }
    }
}

/// `Date` in the form `banned-players.json` keeps it, `yyyy-MM-dd HH:mm:ss Z`, in UTC.
fn ban_date(millis: i64) -> String {
    let seconds = millis.div_euclid(1000);
    let days = seconds.div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let of_era = days.rem_euclid(146_097);
    let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted = (5 * of_year + 2) / 153;
    let day = of_year - (153 * shifted + 2) / 5 + 1;
    let month = if shifted < 10 { shifted + 3 } else { shifted - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    let left = seconds.rem_euclid(86_400);
    format!("{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} +0000", left / 3600, left / 60 % 60, left % 60)
}

impl crate::extra::BanList {
    /// bukkit: org/bukkit/BanList.addBan
    ///
    /// With its expiry (milliseconds, `None` for good) and source, and a reason of any shape, which
    /// `add_ban` through `/ban` can't carry. It doesn't kick: `Bukkit`'s doesn't either.
    pub fn add_ban_until(&self, target: &str, reason: &str, expires: Option<i64>, source: &str) -> bool {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_millis() as i64);
        let mut row = json!({
            "created": ban_date(now),
            "source": source,
            "expires": expires.map_or_else(|| "forever".to_owned(), ban_date),
            "reason": reason,
        });
        if self.addresses {
            row["ip"] = json!(target);
        } else {
            let offline = crate::get_offline_player(target);
            let id = offline.get_unique_id().unwrap_or_default().replace('-', "");
            if id.len() != 32 {
                return false;
            }
            row["name"] = json!(target);
            row["uuid"] = json!(format!("{}-{}-{}-{}-{}", &id[..8], &id[8..12], &id[12..16], &id[16..20], &id[20..]));
        }
        query(json!({ "op": "core_ban_save", "addresses": self.addresses, "row": row })).as_bool().unwrap_or(false)
    }
}

/// `UUID.randomUUID`: version 4, from the system's hasher keys, which are random per process.
fn random_uuid() -> [u8; 16] {
    use std::hash::{BuildHasher, Hasher};
    let mut out = [0; 16];
    for (index, chunk) in out.chunks_mut(8).enumerate() {
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_usize(index);
        chunk.copy_from_slice(&hasher.finish().to_be_bytes());
    }
    out[6] = (out[6] & 0x0f) | 0x40;
    out[8] = (out[8] & 0x3f) | 0x80;
    out
}

/// bukkit: org/bukkit/Bukkit.getPlayer, org/bukkit/Server.getPlayer
///
/// By id: `Bukkit.getPlayer(UUID)`, dashed or not.
pub fn get_player_by_id(uuid: &str) -> Option<Player> {
    let plain = uuid.replace('-', "");
    Player::read(&query(json!({ "op": "player", "uuid": plain }))).or_else(|| get_online_players().into_iter().find(|player| player.uuid() == plain))
}

impl Player {
    /// bukkit: org/bukkit/entity/LivingEntity.getMaxHealth, org/bukkit/entity/Damageable.getMaxHealth
    pub fn get_max_health(&self) -> f64 {
        self.get_attribute("minecraft:max_health").get_value()
    }
    /// bukkit: org/bukkit/entity/Entity.isInWater
    pub fn is_in_water(&self) -> bool {
        self.get_location().get_block().get_type().key() == "minecraft:water"
    }
    /// bukkit: org/bukkit/entity/Entity.eject
    pub fn eject(&self) -> bool {
        let riders = self.data_at("Passengers").as_array().cloned().unwrap_or_default();
        for rider in &riders {
            if let Some(uuid) = rider["UUID"].as_array() {
                let words: Vec<String> = uuid.iter().filter_map(|word| word.as_i64().map(|word| (word as i32).to_string())).collect();
                run_command(&format!("execute as @e[nbt={{UUID:[I;{}]}}] run ride @s dismount", words.join(",")));
            }
        }
        !riders.is_empty()
    }
    /// bukkit: org/bukkit/entity/Entity.isEmpty, org/bukkit/entity/Entity.getPassengers
    ///
    /// Whether anything rides this player: what `isEmpty` answers, the other way round.
    pub fn has_passengers(&self) -> bool {
        self.data_at("Passengers").as_array().is_some_and(|riders| !riders.is_empty())
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getTargetBlockExact
    pub fn get_target_block_exact(&self, distance: i32, fluids: &str) -> Option<Block> {
        let eyes = self.get_eye_location();
        let world = eyes.get_world().clone();
        let hit = world.ray_trace(&eyes, &eyes.get_direction(), f64::from(distance), fluids, false, 0.0, |_| false)?;
        hit.get_hit_block().cloned()
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getTargetBlock
    pub fn get_target_block(&self, distance: i32) -> Option<Block> {
        self.get_target_block_exact(distance, "NEVER")
    }
}

impl Material {
    /// bukkit: org/bukkit/Material.isTransparent
    pub fn is_transparent(&self) -> bool {
        const NAMES: &str = include_str!("fam_ess_transparent.txt");
        NAMES.lines().any(|name| name == self.key().trim_start_matches("minecraft:"))
    }
}

impl Player {
    fn body(&self, field: &str) -> Value {
        self.state()[field].clone()
    }
    fn set_body(&self, field: &str, value: Value) -> bool {
        query(json!({ "op": "player_set", "uuid": self.uuid(), "field": field, "value": value })).as_bool().unwrap_or(false)
    }
    /// `Entity.getFireTicks`, of a player, which the `Entity` here isn't.
    pub fn get_fire_ticks(&self) -> i32 {
        self.body("fire_ticks").as_i64().unwrap_or(0) as i32
    }
    /// `Entity.setFireTicks`.
    pub fn set_fire_ticks(&self, ticks: i32) -> bool {
        self.set_body("fire_ticks", json!(ticks))
    }
    /// `LivingEntity.getRemainingAir`.
    pub fn get_remaining_air(&self) -> i32 {
        self.body("air").as_i64().unwrap_or(300) as i32
    }
    /// bukkit: org/bukkit/entity/LivingEntity.getMaximumAir
    pub fn get_maximum_air(&self) -> i32 {
        query(json!({ "op": "entity_player_get", "uuid": self.uuid(), "key": "max_air" })).as_i64().unwrap_or(300) as i32
    }
    /// `LivingEntity.setRemainingAir`.
    pub fn set_remaining_air(&self, ticks: i32) -> bool {
        self.set_body("air", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/Entity.setFreezeTicks
    pub fn set_freeze_ticks(&self, ticks: i32) -> bool {
        self.set_body("freeze_ticks", json!(ticks))
    }
    /// bukkit: org/bukkit/entity/Entity.getMaxFreezeTicks
    pub fn get_max_freeze_ticks(&self) -> i32 {
        140
    }
    /// bukkit: org/bukkit/entity/Player.setStatistic
    ///
    /// `TIME_SINCE_REST`, online.
    pub fn set_time_since_rest(&self, ticks: i32) -> bool {
        self.set_body("time_since_rest", json!(ticks))
    }
    /// bukkit~: org/bukkit/entity/Damageable.damage | by a damage type's name: a command, at the end of the tick inside a player's event
    pub fn damage(&self, amount: f64, kind: &str) {
        crate::run_command(&format!("damage {} {amount} {kind}", self.get_name()));
    }
    /// bukkit: org/bukkit/entity/Entity.addPassenger
    ///
    /// This player onto `vehicle`, which a `Player` here can't be passed as: a command, at the end of the tick.
    pub fn ride(&self, vehicle: &crate::entity::Entity) {
        crate::run_command(&format!("ride {} mount {}", self.get_name(), crate::dashed(&vehicle.get_unique_id())));
    }
    /// bukkit: org/bukkit/inventory/InventoryView.setItem
    ///
    /// A slot of whatever menu the player has open.
    pub fn set_menu_slot(&self, slot: usize, stack: Option<&crate::ItemStack>) {
        query(json!({ "op": "items_menu_set", "uuid": self.uuid(), "slot": slot, "stack": stack.map(crate::ItemStack::to_json) }));
    }
    /// `Entity.getFreezeTicks`.
    pub fn get_freeze_ticks(&self) -> i32 {
        self.body("freeze_ticks").as_i64().unwrap_or(0) as i32
    }
    /// `Entity.getFallDistance`.
    pub fn get_fall_distance(&self) -> f64 {
        self.body("fall_distance").as_f64().unwrap_or(0.0)
    }
    /// `Entity.setFallDistance`.
    pub fn set_fall_distance(&self, distance: f64) -> bool {
        self.set_body("fall_distance", json!(distance))
    }
}
