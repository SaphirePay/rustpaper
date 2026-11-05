# What a mixin can't do for family paper: rows of (file under crates/, text, replacement). See `family_edits` in
# fabric-port/tools/dim_patch.py. These come after ev_player_engine.py's, which wrote `hold_configuration`.
LAN = 'horizon-client/src/lan/'


def build_time():
    """The moment a build's sources were first applied: the same sources keep it, so the stage stays as it was and
    cargo has nothing to do, and new ones rebuild `game.rs` anyway."""
    import hashlib
    import json
    import pathlib
    import sys
    import time
    # The tables are also read by the generators, which have no tree to hash.
    if len(sys.argv) < 2:
        return 0
    fresh = pathlib.Path(sys.argv[1])
    digest = hashlib.sha256()
    for path in sorted((fresh / 'crates').rglob('*.rs')):
        digest.update(path.read_bytes())
    note = fresh.parent / 'paper_build_time.json'
    try:
        kept = json.loads(note.read_text(encoding='utf-8'))
    except (OSError, ValueError):
        kept = {}
    if kept.get('sources') != digest.hexdigest():
        kept = {'sources': digest.hexdigest(), 'time': int(time.time() * 1000)}
        note.write_text(json.dumps(kept), encoding='utf-8')
    return kept['time']


EDITS = [
    # Anyone with something to say to a connection in configuration says it from their own thread.
    ('horizon-client/src/game.rs', '''        if let Some(answer) = super::paper_fam_ev_world::free_query(request) {
''', '''        if let Some(answer) = super::paper_fam_paper::free_query(request) {
            return answer;
        }
        if let Some(answer) = super::paper_fam_ev_world::free_query(request) {
'''),
    # `PaperConfigurationTask`: after the packs, before the world, for as long as the listeners take.
    (LAN + 'conn.rs', '''    if reconfiguring {
        hold_configuration(c, uuid, &mut info)?;
        return Ok(Some(Parts { uuid, name: name.to_owned(), properties, info, brand }));
    }
    send(c, cfg::Clientbound::FinishConfiguration(Empty))?;
''', '''    if reconfiguring || crate::paper_hub::configuration_held() {
        hold_configuration(c, uuid, &mut info, reconfiguring)?;
        return Ok(Some(Parts { uuid, name: name.to_owned(), properties, info, brand }));
    }
    crate::paper_hub::configuring_ends();
    send(c, cfg::Clientbound::FinishConfiguration(Empty))?;
'''),
    (LAN + 'conn.rs', '''/// A reconfiguring player stays here until they're let go: `PlayerConnectionReconfigureEvent`, the
/// configure listeners' packets, then keep-alives until `completeReconfiguration`, then the
/// finish, and the client's answer to it.
fn hold_configuration(
    c: &mut Connection,
    uuid: u128,
    info: &mut ClientInformation,
) -> Result<(), String> {
    let e = |e: horizon_protocol::Error| e.to_string();
    crate::paper_hub::reconfigure_event(uuid);
    match crate::paper_hub::reconfigure_packets() {
        Ok(packets) => {
            for body in packets {
                c.send(&body).map_err(e)?;
            }
        }
        Err(packets) => {
            for body in packets {
                c.send(&body).map_err(e)?;
            }
            return Err("Kicked while reconfiguring".to_owned());
        }
    }
    std::thread::scope(|scope| {
        let (stop, stopped) = channel::<()>();
        let writer = &mut c.writer;
        scope.spawn(move || {
            let start = Instant::now();
            let mut next = start + KEEP_ALIVE_EVERY;
            while stopped.recv_timeout(Duration::from_millis(100)) == Err(RecvTimeoutError::Timeout) {
                if crate::paper_hub::reconfiguration_released(uuid) {
                    let _ = writer.send(&cfg::Clientbound::FinishConfiguration(Empty).encode());
                    return;
                }
                if Instant::now() >= next {
''', '''/// Configuration that waits on plugins: on Paper a reconfiguring player stays until
/// `completeReconfiguration`, and a first one until `AsyncPlayerConnectionConfigureEvent`'s listeners
/// return. This thread reads what the client says, another sends keep-alives and whatever a plugin
/// sends, from any thread, then the finish or the kick, and this one goes on to the client's answer.
fn hold_configuration(
    c: &mut Connection,
    uuid: u128,
    info: &mut ClientInformation,
    reconfiguring: bool,
) -> Result<(), String> {
    let id = crate::paper_hub::held_id();
    let closer = c.reader.closer().map_err(|e| e.to_string())?;
    let kicked = std::sync::atomic::AtomicBool::new(false);
    if reconfiguring {
        crate::paper_hub::reconfigure_event(uuid);
    } else {
        crate::paper_hub::configure_event();
    }
    let held = std::thread::scope(|scope| {
        let (stop, stopped) = channel::<()>();
        let writer = &mut c.writer;
        let kicked = &kicked;
        scope.spawn(move || {
            let start = Instant::now();
            let mut next = start + KEEP_ALIVE_EVERY;
            while stopped.recv_timeout(Duration::from_millis(20)) == Err(RecvTimeoutError::Timeout) {
                let (packets, kick, listening) = crate::paper_hub::held_take(id);
                for body in packets {
                    if writer.send(&body).is_err() {
                        return;
                    }
                }
                if let Some(body) = kick {
                    kicked.store(true, Ordering::Relaxed);
                    let _ = writer.send(&body);
                    closer.close();
                    return;
                }
                if !listening && (!reconfiguring || crate::paper_hub::reconfiguration_released(uuid)) {
                    let _ = writer.send(&cfg::Clientbound::FinishConfiguration(Empty).encode());
                    return;
                }
                if Instant::now() >= next {
'''),
    (LAN + 'conn.rs', '''                Ok(_) => {}
                Err(err) => break Err(err),
            }
        };
        let _ = stop.send(());
        result
    })
}
/// ServerboundResourcePackPacket.Action.DECLINED.
''', '''                Ok(_) => {}
                Err(err) => break Err(err),
            }
        };
        let _ = stop.send(());
        result
    });
    crate::paper_hub::configuring_ends();
    if kicked.load(Ordering::Relaxed) {
        return Err("Kicked while configuring".to_owned());
    }
    held
}
/// ServerboundResourcePackPacket.Action.DECLINED.
'''),
    # A plugin's pack is found where the plugin has it, under an id of its own.
    ('horizon-client-render/src/datapack.rs', '''/// A pack on disk.
pub struct DataPack {
    /// `file/<name>` in `/datapack` and level.dat.
''', '''/// A pack a plugin found, under `<plugin>/<id>`: `PaperDatapackRegistrar.discoverPack`.
pub struct PluginPack {
    pub id: String,
    pub path: PathBuf,
    pub title: serde_json::Value,
    /// `PackSelectionConfig`: `autoEnableOnServerStart`, then where it goes and whether it stays there.
    pub required: bool,
    pub top: bool,
    pub fixed: bool,
}
/// `PaperDatapackRegistrar.discoveredPacks`, for the packs plugins added.
pub static PLUGIN_PACKS: std::sync::Mutex<Vec<PluginPack>> = std::sync::Mutex::new(Vec::new());
/// What `removeDiscoveredPack` took out of the repository, for as long as the server runs.
pub static HIDDEN_PACKS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
fn plugin_pack<R>(id: &str, read: impl FnOnce(&PluginPack) -> R) -> Option<R> {
    PLUGIN_PACKS.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|pack| pack.id == id).map(read)
}
/// `Pack.Position.insert`: at its end of the order, inside the packs fixed there.
fn place(selected: &[String], top: bool) -> usize {
    let fixed = |id: &String| plugin_pack(id, |pack| pack.fixed && pack.top == top) == Some(true);
    if top {
        selected.iter().rposition(|id| !fixed(id)).map_or(0, |at| at + 1)
    } else {
        selected.iter().position(|id| !fixed(id)).unwrap_or(selected.len())
    }
}
/// Where `/datapack enable` puts `id`, told no more: `Pack.getDefaultPosition`, top but for a plugin's say.
pub fn default_place(selected: &[String], id: &str) -> usize {
    place(selected, plugin_pack(id, |pack| pack.top).unwrap_or(true))
}
/// A pack on disk.
pub struct DataPack {
    /// `file/<name>` in `/datapack` and level.dat.
'''),
    ('horizon-client-render/src/datapack.rs', '''    /// The `description` of `pack.mcmeta`, as text.
    pub fn description(&self) -> String {
''', '''    /// `PackDetector.detectPackResources`: a folder with `pack.mcmeta`, or a zip.
    pub fn at(id: &str, path: &Path) -> Option<Self> {
        if path.join("pack.mcmeta").is_file() {
            return Some(Self { id: id.to_owned(), source: Source::Dir(path.to_path_buf()) });
        }
        let zip = path.is_file().then(|| Zip::open(path).ok()).flatten().filter(|_| path.to_string_lossy().ends_with(".zip"))?;
        zip.contains("pack.mcmeta").then(|| Self { id: id.to_owned(), source: Source::Zip(Box::new(zip)) })
    }
    /// The `description` of `pack.mcmeta`, as text.
    pub fn description(&self) -> String {
'''),
    ('horizon-client-render/src/datapack.rs', '''    let mut packs = discover(world_root);
    packs.extend(feature_packs(resources));
    packs
}
''', '''    let mut packs = discover(world_root);
    packs.extend(feature_packs(resources));
    let plugins = PLUGIN_PACKS.lock().unwrap_or_else(|e| e.into_inner()).iter().filter_map(|pack| DataPack::at(&pack.id, &pack.path)).collect::<Vec<_>>();
    if !plugins.is_empty() {
        packs.extend(plugins);
        packs.sort_by(|a, b| a.id.cmp(&b.id));
    }
    let hidden = HIDDEN_PACKS.lock().unwrap_or_else(|e| e.into_inner());
    packs.retain(|pack| !hidden.contains(&pack.id));
    packs
}
'''),
    ('horizon-client-render/src/datapack.rs', '''            && p.id.starts_with("file/")
            && features_enabled(p, enabled_features)
        {
            enabled.push(p.id.clone());
        }
    }
    for p in packs {
        let feature_pack = !p.id.starts_with("file/") && p.id != "vanilla";
''', '''            && (p.id.starts_with("file/") || plugin_pack(&p.id, |_| ()).is_some())
            && features_enabled(p, enabled_features)
        {
            enabled.push(p.id.clone());
        }
    }
    // `PackRepository.rebuildSelected`: what's required comes back, at its end of the order.
    for p in packs {
        if plugin_pack(&p.id, |pack| pack.required) == Some(true) && !enabled.contains(&p.id) {
            let at = default_place(&enabled, &p.id);
            enabled.insert(at, p.id.clone());
        }
    }
    for p in packs {
        let feature_pack = !p.id.starts_with("file/") && p.id != "vanilla" && plugin_pack(&p.id, |_| ()).is_none();
'''),
    ('horizon-client/src/game/functions.rs', '''                    } else {
                        self.fns.enabled.len()
                    };
                    self.fns.enabled.insert(position, name.clone());
''', '''                    } else {
                        datapack::default_place(&self.fns.enabled, &name)
                    };
                    self.fns.enabled.insert(position, name.clone());
'''),
    # The properties of whoever's being let in or refused.
    (LAN + 'conn.rs', '''    if let Some(reason) = shared.try_admit(address, uuid, &name) {
''', '''    crate::paper_hub::admitting(&properties);
    if let Some(reason) = shared.try_admit(address, uuid, &name) {
'''),
    # `ServerBuildInfo.buildTime`. A build's sources decide it, so nothing changes when they don't.
    ('horizon-client/src/game.rs', '''    const BUILD_TIME_MS: u64 = 0;
''', f'''    const BUILD_TIME_MS: u64 = {build_time()};
'''),
    # `CombatTracker.combatStartTime` and `combatEndTime`, of mobs and of players.
    ('horizon-client/src/mobs/combat.rs', '''    last_damage_time: u32,
    pub in_combat: bool,
    taking_damage: bool,
}
''', '''    last_damage_time: u32,
    pub in_combat: bool,
    taking_damage: bool,
    started: u32,
    ended: u32,
}
'''),
    ('horizon-client/src/mobs/combat.rs', '''        if !self.in_combat && living_cause {
            self.in_combat = true;
        }
    }
''', '''        if !self.in_combat && living_cause {
            self.in_combat = true;
            self.started = tick;
            self.ended = tick;
        }
    }
    /// `CombatTracker.lastDamageTime`.
    pub fn last_damage_time(&self) -> u32 {
        self.last_damage_time
    }
    /// `CombatTracker.getCombatDuration`.
    pub fn duration(&self, tick: u32) -> u32 {
        if self.in_combat { tick.wrapping_sub(self.started) } else { self.ended.wrapping_sub(self.started) }
    }
    /// `CombatTracker.resetCombatState`.
    pub fn reset(&mut self, tick: u32) {
        self.taking_damage = false;
        self.in_combat = false;
        self.ended = tick;
        self.entries.clear();
    }
'''),
    ('horizon-client/src/mobs/combat.rs', '''            self.taking_damage = false;
            self.in_combat = false;
            self.entries.clear();
        }
    }
}
''', '''            self.reset(tick);
        }
    }
}
'''),
    ('horizon-client/src/game/combat.rs', '''    last_damage: i64,
    in_combat: bool,
    /// Attackers of damage dealt this tick, claimed by the matching
''', '''    last_damage: i64,
    in_combat: bool,
    started: i64,
    ended: i64,
    /// Attackers of damage dealt this tick, claimed by the matching
'''),
    ('horizon-client/src/game/combat.rs', '''                if !self.combat.in_combat {
                    self.combat.in_combat = true;
                }
            }
            self.stats.custom("damage_taken", (amount * 10.0).round() as i32);
''', '''                if !self.combat.in_combat {
                    self.combat.in_combat = true;
                    self.combat.started = now;
                    self.combat.ended = now;
                }
            }
            self.stats.custom("damage_taken", (amount * 10.0).round() as i32);
'''),
    ('horizon-client/src/game/combat.rs', '''            self.combat.entries.clear();
            self.combat.in_combat = false;
        }
    }
    fn damage_in_tag(&self, name: &str, tag: &str) -> bool {
''', '''            self.combat.entries.clear();
            self.combat.in_combat = false;
            self.combat.ended = self.world.game_time;
        }
    }
    fn damage_in_tag(&self, name: &str, tag: &str) -> bool {
'''),
    ('horizon-client/src/game/lan_actions.rs', '''        let g = &mut host.hub.guests[i];
        g.dead = false;
        g.load_timeout = horizon_client::lan::guest::CLIENT_LOADED_TIMEOUT;
''', '''        let g = &mut host.hub.guests[i];
        crate::game::paper_fam_paper::player_respawned(g.conn);
        g.dead = false;
        g.load_timeout = horizon_client::lan::guest::CLIENT_LOADED_TIMEOUT;
'''),
    ('horizon-client/src/game/combat.rs', '''        let entries = std::mem::take(&mut self.combat.entries);
        self.combat.in_combat = false;
        let Some(killing) = entries.last() else {
''', '''        let entries = std::mem::take(&mut self.combat.entries);
        self.combat.in_combat = false;
        self.combat.ended = self.world.game_time;
        let Some(killing) = entries.last() else {
'''),
    # A plugin's `minecraft:message` argument is signed as the game's own are.
    ('horizon-client/src/game/lan_chat.rs', '''        self.parse_as(permission, command).signable_arguments(vanilla::game_dispatcher())
''', '''        let arguments = self.parse_as(permission, command).signable_arguments(vanilla::game_dispatcher());
        if arguments.is_empty() { crate::game::paper_fam_paper::plugin_signable(self, command) } else { arguments }
'''),
    ('horizon-client/src/game/lan_actions.rs', '''        signed: Option<Vec<(String, horizon_client::lan::hub::chat::ChatMessage)>>,
    ) {
''', '''        signed: Option<Vec<(String, horizon_client::lan::hub::chat::ChatMessage)>>,
    ) {
        crate::game::paper_fam_paper::note_signed_arguments(signed.as_deref());
'''),
]
