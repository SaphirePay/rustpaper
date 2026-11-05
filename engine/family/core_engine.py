# Family core. Yours alone. Applied by fabric-port/tools/dim_patch.py, after the mixins: each text must be in its file once.
CLIENT = 'horizon-client/src/'
GAME = CLIENT + 'game/'

EDITS = [
    # ---- `World.setVoidDamage*`: the level being ticked's rule, set just before by `core_void_mobs` and `core_void_guest`.
    (CLIENT + 'motion.rs', 'pub(crate) mod environment;', '''pub(crate) mod environment;

/// Whether the void hurts, how much a tick and how far from the level's floor it starts, for the
/// level being ticked. `Entity.checkBelowWorld` and `LivingEntity.onBelowWorld` have 64 under and 4.
pub static VOID_RULE: std::sync::Mutex<(bool, f32, f64)> = std::sync::Mutex::new((true, 4.0, -64.0));

fn void_rule() -> (bool, f32, f64) {
    *VOID_RULE.lock().unwrap_or_else(|e| e.into_inner())
}

/// What the void does to something at `y`, in a level whose floor is `bottom`: none, or how much.
pub(crate) fn void_hurt(y: f64, bottom: i32) -> Option<f32> {
    let (on, amount, offset) = void_rule();
    (on && y < f64::from(bottom) + offset).then_some(amount)
}'''),
    (CLIENT + 'motion/environment.rs', '''        if self.player.position.y < f64::from(self.scene.bottom_y() - 64) {
            let id = self
                .resources
                .health
                .damage
                .id("minecraft:out_of_world")
                .ok_or("missing environment damage type")?;
            self.player
                .hurt(&self.resources.health, self.scene.health_rules, id, 4.0)?;
        }''', '''        if let Some(amount) = crate::motion::void_hurt(self.player.position.y, self.scene.bottom_y()) {
            let id = self
                .resources
                .health
                .damage
                .id("minecraft:out_of_world")
                .ok_or("missing environment damage type")?;
            self.player
                .hurt(&self.resources.health, self.scene.health_rules, id, amount)?;
        }'''),
    (CLIENT + 'mobs/ai.rs', '''    if m.position.y < f64::from(w.world.bottom_y() - 64) {
        m.damage(4.0, Damage::of(DamageKind::OutOfWorld));
    }''', '''    if let Some(amount) = crate::motion::void_hurt(m.position.y, w.world.bottom_y()) {
        m.damage(amount, Damage::of(DamageKind::OutOfWorld));
    }'''),
    # ---- `SignChangeEvent`: a listener's lines go on as the listener wrote them, formatting and all.
    (GAME + 'sign_edit.rs', '''        for (line, new) in side.lines.iter_mut().zip(&p.lines) {
            *line = horizon_client::clipboard::strip_formatting(new);
        }
''', '''        for (line, new) in side.lines.iter_mut().zip(&p.lines) {
            *line = horizon_client::clipboard::strip_formatting(new);
        }
        if let Some(lines) = crate::game::paper_fam_core::take_sign_lines(p.pos, p.front) {
            for (line, new) in side.lines.iter_mut().zip(lines) {
                *line = new;
            }
        }
'''),
]

WORLD = 'horizon-world/src/'

EDITS += [
    # ---- `Server.clearRecipes`: the game's recipe files stop being read, and the packs' of that moment with them.
    (WORLD + 'loot.rs', '''impl PackOverlay<'_> {
    /// The overlay's files under `<kind>/`, with the prefix stripped.
    pub fn kind(&self, kind: &str) -> impl Iterator<Item = (&str, &Vec<Vec<u8>>)> {
        let prefix = format!("{kind}/");
        self.files
            .iter()
            .filter_map(move |(key, files)| Some((key.strip_prefix(&prefix)?, files)))
    }
''', '''/// `Server.clearRecipes`: `Some` from then until `resetRecipes`, holding the packs' recipes that went
/// with the game's. A recipe a plugin adds after that has an id not in it, and is read as usual.
pub static CLEARED_RECIPES: std::sync::Mutex<Option<std::collections::BTreeSet<String>>> = std::sync::Mutex::new(None);

pub fn recipes_cleared() -> bool {
    CLEARED_RECIPES.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}

impl PackOverlay<'_> {
    /// The overlay's files under `<kind>/`, with the prefix stripped.
    pub fn kind(&self, kind: &str) -> impl Iterator<Item = (&str, &Vec<Vec<u8>>)> {
        let prefix = format!("{kind}/");
        let gone = if kind == "recipe" { CLEARED_RECIPES.lock().unwrap_or_else(|e| e.into_inner()).clone() } else { None };
        self.files
            .iter()
            .filter_map(move |(key, files)| Some((key.strip_prefix(&prefix)?, files)))
            .filter(move |(id, _)| !gone.as_ref().is_some_and(|gone| gone.contains(*id)))
    }
'''),
    (CLIENT + 'crafting.rs', '''pub(crate) fn recipe_files(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
''', '''pub(crate) fn recipe_files(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    if horizon_world::loot::recipes_cleared() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
'''),
    (CLIENT + 'game.rs', '''        if files.is_empty() {
            return None;
        }
        let overlay = horizon_world::loot::PackOverlay {
            files: &files,
        };
        let root = &self.options.resources;
''', '''        if files.is_empty() && !horizon_world::loot::recipes_cleared() {
            return None;
        }
        let overlay = horizon_world::loot::PackOverlay {
            files: &files,
        };
        let root = &self.options.resources;
'''),
    # ---- `Server.setRespawnWorld`: `level.dat`'s `spawn` has a dimension, which the game read as always the Overworld.
    (CLIENT + 'save.rs', '''#[derive(Clone, Debug, Default)]
pub struct Level {
    pub name: String,''', '''/// `level.dat`'s `spawn.dimension`: where a player with no bed respawns.
static SPAWN_DIMENSION: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

pub fn spawn_dimension() -> String {
    let name = SPAWN_DIMENSION.lock().unwrap_or_else(|e| e.into_inner());
    if name.is_empty() { "minecraft:overworld".to_owned() } else { name.clone() }
}

pub fn set_spawn_dimension(name: &str) {
    name.clone_into(&mut SPAWN_DIMENSION.lock().unwrap_or_else(|e| e.into_inner()));
}

#[derive(Clone, Debug, Default)]
pub struct Level {
    pub name: String,'''),
    (CLIENT + 'save.rs', '''        level.world_spawn = match data.get("spawn").and_then(|s| s.get("pos")) {
''', '''        if let Some(Tag::String(name)) = data.get("spawn").and_then(|s| s.get("dimension")) {
            set_spawn_dimension(&name.to_string().unwrap_or_default());
        }
        level.world_spawn = match data.get("spawn").and_then(|s| s.get("pos")) {
'''),
    (CLIENT + 'save.rs', '''                    ("pos", Tag::IntArray(spawn.to_vec())),
                    ("dimension", string("minecraft:overworld")),
''', '''                    ("pos", Tag::IntArray(spawn.to_vec())),
                    ("dimension", string(&spawn_dimension())),
'''),
    (GAME + 'lan_actions.rs', '''        let level = elsewhere.map_or(super::Dim::Overworld, |c| c.dim);
''', '''        let level = elsewhere.map_or_else(|| super::Dim::from_id(&horizon_client::save::spawn_dimension()), |c| c.dim);
'''),
    (GAME + 'server_travel.rs', '''                dim: Dim::Overworld,
                source: self.dim,
                target: Vec3::new(f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5),
''', '''                dim: Dim::from_id(&horizon_client::save::spawn_dimension()),
                source: self.dim,
                target: Vec3::new(f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5),
'''),
    (GAME + 'server_travel.rs', '''                dim: Dim::Overworld,
                at,
                rotation: None,
            });
        self.run_level_teleports(host);
        self.world_spawn_trip(conn);
''', '''                dim: Dim::from_id(&horizon_client::save::spawn_dimension()),
                at,
                rotation: None,
            });
        self.run_level_teleports(host);
        self.world_spawn_trip(conn);
'''),
    # ---- `ProxiedCommandSender`: `/execute as ... run <a plugin's word>`, each source in turn.
    (GAME + 'commands/execute.rs', '''                        match self.with_source(s, |g| g.run_chain(command, last)) {
''', '''                        match self.with_source(s, |g| match crate::game::paper_fam_core::proxied_run(g) {
                            Some(done) => Ok(done),
                            None => g.run_chain(command, last),
                        }) {
'''),
    # ---- `BlockTransformer`: a template's blocks asked about, then placed with the answers.
    (WORLD + 'worldgen/structure/processor.rs', '''pub fn place_in_world<W: FeatureWorld + PlacementContext>(
''', '''/// A template placement a plugin takes part in: first to see what would go where, with nothing
/// placed, then with what it wants instead.
#[derive(Default)]
pub enum TemplatePlan {
    #[default]
    Off,
    Asking(Vec<([i32; 3], crate::registry::BlockStateId)>),
    Answered(std::collections::HashMap<[i32; 3], Option<crate::registry::BlockStateId>>),
}

thread_local! {
    pub static TEMPLATE_PLAN: std::cell::RefCell<TemplatePlan> = std::cell::RefCell::new(TemplatePlan::Off);
}

/// The one call each block of a template goes through: what to place there, if anything.
fn planned_state(pos: [i32; 3], state: crate::registry::BlockStateId) -> Option<crate::registry::BlockStateId> {
    TEMPLATE_PLAN.with(|plan| match &mut *plan.borrow_mut() {
        TemplatePlan::Off => Some(state),
        TemplatePlan::Asking(blocks) => {
            blocks.push((pos, state));
            None
        }
        TemplatePlan::Answered(answers) => answers.get(&pos).copied().unwrap_or(Some(state)),
    })
}

pub fn place_in_world<W: FeatureWorld + PlacementContext>(
'''),
    (WORLD + 'worldgen/structure/processor.rs', '''            settings.rotation,
        );
        if info.nbt.is_some() {
            world.set_block(info.pos, barrier, 820).map_err(world_err)?;
        }
''', '''            settings.rotation,
        );
        let Some(state) = planned_state(info.pos, state) else {
            continue;
        };
        if info.nbt.is_some() {
            world.set_block(info.pos, barrier, 820).map_err(world_err)?;
        }
'''),
    (GAME + 'structure_blocks.rs', '''        for entity in entities {
            self.spawn_entity_from_nbt(Tag::Compound(entity));
        }
        // The block entities the template loaded reach the clients.
''', '''        for entity in crate::game::paper_fam_core::planned_entities(entities) {
            self.spawn_entity_from_nbt(Tag::Compound(entity));
        }
        // The block entities the template loaded reach the clients.
'''),
]

EDITS += [
    # ---- Commands after the start: the bridge's own way of taking them, from the tick that wants one.
    (CLIENT + 'game.rs', '''    fn take_commands() {
        for action in runtime::take_actions() {
''', '''    pub(in crate::game) fn take_commands() {
        for action in runtime::take_actions() {
'''),
    # ---- `findLightningTarget` is the game's own search.
    (GAME + 'lightning.rs', '''    fn lightning_target(&mut self, x: i32, z: i32) -> Option<[i32; 3]> {
''', '''    pub(super) fn lightning_target(&mut self, x: i32, z: i32) -> Option<[i32; 3]> {
'''),
    # ---- `World.Spigot.strikeLightning(location, true)`: a bolt with no thunder.
    (GAME + 'lightning.rs', '''                if b.life == 2 {
                    let pitch = 0.8 + self.world.random.next_float() * 0.2;
''', '''                if b.life == 2 && !crate::game::paper_fam_core::silent_bolt(b.at) {
                    let pitch = 0.8 + self.world.random.next_float() * 0.2;
'''),
]

EDITS += [
    # ---- RCON: who sent a line, which `rcon.rs` knows and never told the game.
    ('horizon-server/src/rcon.rs', '''use horizon_server_files::properties::{ServerProperties, SystemEntropy};
''', '''use horizon_server_files::properties::{ServerProperties, SystemEntropy};

thread_local! {
    /// Who this connection's thread is serving.
    static PEER: std::cell::Cell<Option<SocketAddr>> = const { std::cell::Cell::new(None) };
}

/// The client of the line being run, as one word: `RemoteConsoleCommandSender.getAddress`.
fn peer() -> String {
    PEER.get().map_or_else(|| "-".to_owned(), |address| address.to_string())
}
'''),
    ('horizon-server/src/rcon.rs', '''                    let from = stream
                        .peer_addr()
''', '''                    PEER.set(stream.peer_addr().ok());
                    let from = stream
                        .peer_addr()
'''),
    # ---- A console line for a word a plugin took: the game's tree hasn't got it.
    (GAME + 'dedicated.rs', '''        let rest: Vec<&str> = words.collect();
        let lit = |s: &str| Text::literal(s.to_owned());
        match command {
            "stop" => {
''', '''        if crate::game::paper_fam_core::console_word(self, line) {
            return;
        }
        let rest: Vec<&str> = words.collect();
        let lit = |s: &str| Text::literal(s.to_owned());
        match command {
            "stop" => {
'''),
    (GAME + 'dedicated.rs', '''    fn console_reply(''', '''    pub(in crate::game) fn console_reply('''),
]

EDITS += [
    # ---- `getConnectionThrottle`: `bukkit.yml`'s number, doing what it does on Paper.
    (CLIENT + 'lib.rs', '''    pub fn player_handshake(
        host: &str,
        port: u16,
        protocol: i32,
        intent: i32,
        address: std::net::SocketAddr,
    ) -> (Option<Tag>, std::net::SocketAddr) {
        if !horizon_mod_runtime::listens("pl_handshake") {
''', '''    /// `settings.connection-throttle` of `bukkit.yml`, in milliseconds: 0 or less is none.
    fn throttle_millis() -> i64 {
        static MILLIS: std::sync::OnceLock<i64> = std::sync::OnceLock::new();
        *MILLIS.get_or_init(|| {
            let text = std::fs::read_to_string("bukkit.yml").unwrap_or_default();
            let found = text.lines().find_map(|line| line.trim().strip_prefix("connection-throttle:")?.trim().parse().ok());
            found.unwrap_or(4000)
        })
    }
    /// `ServerHandshakePacketListenerImpl`: someone logging in again too soon. Never this machine.
    fn throttled(address: std::net::IpAddr) -> bool {
        static LAST: std::sync::Mutex<Option<std::collections::HashMap<std::net::IpAddr, std::time::Instant>>> = std::sync::Mutex::new(None);
        let limit = throttle_millis();
        if limit <= 0 || address.is_loopback() {
            return false;
        }
        let now = std::time::Instant::now();
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        let last = last.get_or_insert_default();
        let hurried = last.get(&address).is_some_and(|at| now.duration_since(*at).as_millis() < limit as u128);
        last.insert(address, now);
        last.retain(|_, at| now.duration_since(*at).as_millis() < limit as u128 * 2);
        hurried
    }
    pub fn player_handshake(
        host: &str,
        port: u16,
        protocol: i32,
        intent: i32,
        address: std::net::SocketAddr,
    ) -> (Option<Tag>, std::net::SocketAddr) {
        if intent != 1 && throttled(address.ip()) {
            return (Some(reason_tag(&serde_json::json!("Connection throttled! Please wait before reconnecting."))), address);
        }
        if !horizon_mod_runtime::listens("pl_handshake") {
'''),
    # ---- `LootTable.populateLoot`: what the game's own drops go through, with a context.
    (GAME + 'creatures.rs', '''fn from_loot(stack: &horizon_world::loot::LootStack) -> ItemStack {
''', '''pub(in crate::game) fn from_loot(stack: &horizon_world::loot::LootStack) -> ItemStack {
'''),
    (GAME + 'creatures.rs', '''fn to_loot(stack: &ItemStack) -> horizon_world::loot::LootStack {
''', '''pub(in crate::game) fn to_loot(stack: &ItemStack) -> horizon_world::loot::LootStack {
'''),
    # ---- `Structure.place` with transformers: what they've answered shows while the next is asked.
    ('horizon-client-render/src/world.rs', '''    /// Resident storage write; marks the touched sections (and neighbours on
''', '''    /// A block with nothing told: no client, block entity, light or mesh hears of it. For a look
    /// at what a write would give, undone the same way before anything else reads the world.
    pub fn set_quietly(&mut self, x: i32, y: i32, z: i32, state: BlockStateId) -> Option<BlockStateId> {
        let chunk = self.chunks.get_mut(&(x >> 4, z >> 4))?;
        chunk.section_index(y)?;
        let previous = Arc::make_mut(chunk).set(x & 15, y, z & 15, state.0);
        self.revision = self.revision.wrapping_add(1);
        Some(BlockStateId(previous))
    }
    /// Resident storage write; marks the touched sections (and neighbours on
'''),
    # ---- `LootTable.fillInventory`: the second half of `LootTable.fill`, for what was rolled.
    ('horizon-world/src/loot.rs', '''/// `MobEffect.isInstantaneous`.
''', '''/// `LootTable.fill`, for what's been rolled: over `slots` free slots, each answer by its position
/// among them, as a chest's are.
pub fn spread_loot<R: RandomSource>(mut items: Vec<LootStack>, slots: usize, random: &mut R) -> Vec<Option<LootStack>> {
    let mut out = vec![None; slots];
    let mut available: Vec<usize> = (0..slots).collect();
    shuffle(&mut available, random);
    shuffle_and_split(&mut items, available.len(), random);
    for stack in items {
        let Some(slot) = available.pop() else { break };
        out[slot] = (!stack.is_empty()).then_some(stack);
    }
    out
}
/// `MobEffect.isInstantaneous`.
'''),
    # ---- `World.setViewDistance` and its like: each level's own, in its tickets and its streaming.
    (CLIENT + 'game.rs', '''                    view_distance: self.settings.render_distance.clamp(2, 32),
''', '''                    view_distance: crate::game::paper_fam_core::level_distance(self.dim.id(), "view", self.settings.render_distance).clamp(2, 32),
'''),
    (CLIENT + 'game.rs', '''                    simulation_distance: self.settings.simulation_distance.max(2),
''', '''                    simulation_distance: crate::game::paper_fam_core::level_distance(self.dim.id(), "simulation", self.settings.simulation_distance).max(2),
'''),
    (GAME + 'lan.rs', '''        let radius = host.hub.config.view_distance + 1;
''', '''        let radius = crate::game::paper_fam_core::level_distance(self.dim.id(), "view", host.hub.config.view_distance) + 1;
'''),
    (CLIENT + 'lan/hub.rs', '''    fn chunk_radius(&self) -> i32 {
        let view = self.config.view_distance;
        self.far.as_ref().map_or(view, |far| far.max.max(view))
    }
''', '''    fn chunk_radius(&self) -> i32 {
        self.radius_for(self.config.view_distance)
    }
    /// `chunk_radius`, for a view of its own: a level's.
    pub fn radius_for(&self, view: i32) -> i32 {
        self.far.as_ref().map_or(view, |far| far.max.max(view))
    }
'''),
    # ---- `Chunk.addPluginChunkTicket`: held as forced chunks are, without being any of `/forceload`'s.
    (CLIENT + 'game.rs', '''                    forced: self.streaming.forced.iter().copied().collect(),
''', '''                    forced: self.streaming.forced.iter().copied().chain(crate::game::paper_fam_core::ticket_chunks(self.dim.id())).collect(),
'''),
    (GAME + 'lan.rs', '''            .chain(self.arrival_chunks(self.dim).into_iter().map(|chunk| (chunk, 1)))
''', '''            .chain(self.arrival_chunks(self.dim).into_iter().map(|chunk| (chunk, 1)))
            .chain(crate::game::paper_fam_core::ticket_chunks(self.dim.id()).into_iter().map(|chunk| (chunk, 1)))
'''),
    (GAME + 'levels.rs', '''                let forced = if *d == self.dim {
''', '''                let forced = !crate::game::paper_fam_core::ticket_chunks(d.id()).is_empty() || if *d == self.dim {
'''),
    # ---- `Player.getListeningPluginChannels`: what a client says before it's in the game.
    (CLIENT + 'lan/conn.rs', '''/// ServerboundResourcePackPacket.Action.DECLINED.
''', '''/// What clients said they listen on while configuring, by id: `true` adds a channel, `false`
/// takes one away. The game side reads these when it's asked.
pub static CONFIG_CHANNELS: std::sync::Mutex<std::collections::BTreeMap<u128, Vec<(bool, String)>>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
thread_local! {
    /// Who this connection's thread is configuring.
    static CONFIGURING: std::cell::Cell<u128> = const { std::cell::Cell::new(0) };
}
/// `minecraft:register` and `minecraft:unregister`, as `ServerCommonPacketListenerImpl` takes them.
fn config_channels(uuid: u128, payload: &cfg::CustomPayload) {
    let adding = match payload.channel.as_str() {
        "minecraft:register" => true,
        "minecraft:unregister" => false,
        _ => return,
    };
    let names = payload.data.split(|byte| *byte == 0).filter(|name| !name.is_empty()).map(|name| (adding, String::from_utf8_lossy(name).into_owned()));
    CONFIG_CHANNELS.lock().unwrap_or_else(|e| e.into_inner()).entry(uuid).or_default().extend(names);
}
/// ServerboundResourcePackPacket.Action.DECLINED.
'''),
    (CLIENT + 'lan/conn.rs', '''    let result = login_and_configure(&mut c, shared, uuid, &name, properties);
''', '''    CONFIGURING.set(uuid);
    let result = login_and_configure(&mut c, shared, uuid, &name, properties);
'''),
    (CLIENT + 'lan/conn.rs', '''fn decode_config(id: i32, payload: &[u8]) -> horizon_protocol::Result<cfg::Serverbound> {
''', '''fn decode_config(id: i32, payload: &[u8]) -> horizon_protocol::Result<cfg::Serverbound> {
    let packet = decode_config_packet(id, payload)?;
    if let cfg::Serverbound::CustomPayload(payload) = &packet {
        config_channels(CONFIGURING.get(), payload);
    }
    Ok(packet)
}
fn decode_config_packet(id: i32, payload: &[u8]) -> horizon_protocol::Result<cfg::Serverbound> {
'''),
    # ---- `Player.getPersistentDataContainer`: `BukkitValues` in the player's file, as Java's.
    (CLIENT + 'lan/playerdata.rs', '''/// Put a guest's state in `level`'s player fields; `vehicle` is the
''', '''/// What plugins have written since these players' files were read: `BukkitValues`, by player.
pub static BUKKIT_VALUES: std::sync::Mutex<std::collections::BTreeMap<u128, Tag>> = std::sync::Mutex::new(std::collections::BTreeMap::new());

/// Put a guest's state in `level`'s player fields; `vehicle` is the
'''),
    (CLIENT + 'lan/playerdata.rs', '''    level.player_extra = guest.saved.clone();
''', '''    level.player_extra = guest.saved.clone();
    if let Some(values) = BUKKIT_VALUES.lock().unwrap_or_else(|e| e.into_inner()).get(&guest.uuid)
        && let Some(Tag::Compound(map)) = &mut level.player_extra
    {
        map.insert(horizon_world::nbt::JavaString::from("BukkitValues"), values.clone());
    }
'''),
    # ---- `Raid.getBossBar`: once a plugin has the bar, it's a bar like any other.
    (GAME + 'bossbars.rs', '''            && let Some((id, name, progress)) = self.raid_boss_at(viewer.at, viewer.alive)
''', '''            && let Some((id, name, progress)) = self.raid_boss_at(viewer.at, viewer.alive)
            && !crate::game::paper_fam_core::raid_bar_owned(id)
'''),
    (GAME + 'raids.rs', '''    /// The raids of the dimension the player is in, and that dimension.
    raids: Raids,
''', '''    /// The raids of the dimension the player is in, and that dimension.
    pub(super) raids: Raids,
'''),
    # ---- At once: a player's turn lends the hub, and a console line can run in it.
    (GAME + 'lan_actions.rs', '''        let result = horizon_client::block_contact::in_level(dimension, || f(self));
''', '''        let result = horizon_mod_runtime::lend_movable(&mut *host, || horizon_client::block_contact::in_level(dimension, || f(self)));
'''),
    (GAME + 'native_server.rs', '''    pub(super) fn native_run_command(&mut self, line: &str) -> bool {
        if self.lan_acting.is_some() || self.lan.is_none() {
            return false;
        }
''', '''    pub(super) fn native_run_command(&mut self, line: &str) -> bool {
        if self.lan_acting.is_some() || self.lan.is_none() {
            return false;
        }
        self.console_run(line)
    }
    /// `native_run_command` with the hub in `self.lan`, whoever's turn this is: a player's own
    /// commands run in that state too.
    pub(super) fn console_run(&mut self, line: &str) -> bool {
'''),
    (GAME + 'lan.rs', '''                    self.lan_guest_packet(&mut host, conn, *packet);
''', '''                    self.lan_guest_packet(&mut host, conn, *packet);
                    if crate::game::paper_fam_core::lines_waiting() {
                        self.lan = Some(host);
                        crate::game::paper_fam_core::run_waiting_lines(self);
                        host = self.lan.take()?;
                    }
'''),
    # ---- `World.createExplosion` says whether a listener let it.
    (GAME + 'explosion.rs', '''        if let Some(p) = &self.player {
            let d = [
                p.position.x - center[0],
''', '''        crate::game::paper_fam_core::explosion_went();
        if let Some(p) = &self.player {
            let d = [
                p.position.x - center[0],
'''),
    # ---- `DragonBattle.getBossBar`: the battle's, dragon or no dragon.
    (GAME + 'bossbars.rs', '''            && let Some(progress) = self.dragon_boss_progress(viewer.at, viewer.alive)
''', '''            && let Some(progress) = self.dragon_boss_progress(viewer.at, viewer.alive)
            && !crate::game::paper_fam_core::dragon_bar_owned()
'''),
    # ---- `CommandMap.clearCommands`: the game's own go from the tree too.
    (CLIENT + 'lan/commands.rs', '''        for root in [&self.root, extra] {
''', '''        let own = if GAME_WORDS_OFF.load(std::sync::atomic::Ordering::Relaxed) { &Value::Null } else { &self.root };
        for root in [own, extra] {
'''),
    (CLIENT + 'lan/commands.rs', '''impl CommandTree {
    /// commands.json next to''', '''/// `CommandMap.clearCommands` took the game's own commands away.
pub static GAME_WORDS_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
impl CommandTree {
    /// commands.json next to'''),
    # ---- Boards plugins make: a player looking at one is off the game's, and team kills reach them.
    (GAME + 'scoreboard.rs', '''        for g in host.hub.guests.iter_mut().filter(|g| g.placed) {
            if g.scoreboard != view {
                g.send_all(&g.scoreboard.changes(&view));
                g.scoreboard = view.clone();
            }
        }
''', '''        let nothing = horizon_client::lan::scoreboard::View::default();
        for g in host.hub.guests.iter_mut().filter(|g| g.placed) {
            let view = if crate::game::paper_fam_core::on_own_board(g.uuid) { &nothing } else { &view };
            if g.scoreboard != *view {
                g.send_all(&g.scoreboard.changes(view));
                g.scoreboard = view.clone();
            }
        }
'''),
    (GAME + 'scoreboard.rs', '''                let criteria = format!("teamkill.{color}");
''', '''                let criteria = format!("teamkill.{color}");
                crate::game::paper_fam_core::team_kill_score(&criteria, &me);
'''),
    (GAME + 'scoreboard.rs', '''                let criteria = format!("killedByTeam.{color}");
''', '''                let criteria = format!("killedByTeam.{color}");
                crate::game::paper_fam_core::team_kill_score(&criteria, &victim_name);
'''),
    # ---- `Player.setPlayerWeather`: the level's weather isn't this player's, as `ServerPlayer.updateWeather`.
    (CLIENT + 'lan/hub.rs', '''pub struct Hub {
    /// Clicks on and closings of native mods' menus, for the game.
''', '''/// Players with a plugin's weather: whether it's downfall, and `pluginRainPosition`.
pub static PERSONAL_WEATHER: std::sync::Mutex<std::collections::BTreeMap<u128, (bool, f32)>> = std::sync::Mutex::new(std::collections::BTreeMap::new());

pub struct Hub {
    /// Clicks on and closings of native mods' menus, for the game.
'''),
    (CLIENT + 'lan/hub.rs', '''        let w = &world.weather;
        let (raining, rain, thunder) = self.weather;
        if w.raining != raining {
            self.broadcast_level(
''', '''        self.personal_weather();
        let w = &world.weather;
        let (raining, rain, thunder) = self.weather;
        if w.raining != raining {
            self.broadcast_weather(
'''),
    (CLIENT + 'lan/hub.rs', '''        if (w.rain_level.1 - rain).abs() > f32::EPSILON {
            self.broadcast_level(
''', '''        if (w.rain_level.1 - rain).abs() > f32::EPSILON {
            self.broadcast_weather(
'''),
    (CLIENT + 'lan/hub.rs', '''        if (w.thunder_level.1 - thunder).abs() > f32::EPSILON {
            self.broadcast_level(
                &C::GameEvent(play::GameEvent {
                    event: game_event::THUNDER_LEVEL,
                    param: w.thunder_level.1,
                }),
            );
        }
''', '''        if (w.thunder_level.1 - thunder).abs() > f32::EPSILON {
            let personal = PERSONAL_WEATHER.lock().unwrap_or_else(|e| e.into_inner());
            for g in self.guests.iter().filter(|g| self.here(g)) {
                let clear = personal.get(&g.uuid).is_some_and(|(downfall, _)| !downfall);
                g.send(&C::GameEvent(play::GameEvent { event: game_event::THUNDER_LEVEL, param: if clear { 0.0 } else { w.thunder_level.1 } }));
            }
        }
'''),
    (CLIENT + 'lan/hub.rs', '''    /// `DedicatedServer.sendLowDiskSpaceWarning`: the players with
''', '''    /// `broadcast_level`, past the players with a plugin's weather.
    fn broadcast_weather(&self, packet: &C) {
        let personal = PERSONAL_WEATHER.lock().unwrap_or_else(|e| e.into_inner());
        let body = packet.encode();
        for g in self.guests.iter().filter(|g| self.here(g) && !personal.contains_key(&g.uuid)) {
            g.sender.send(body.clone());
        }
    }
    /// `ServerPlayer.tickWeather` and the plugin's half of `updateWeather`.
    fn personal_weather(&self) {
        let mut personal = PERSONAL_WEATHER.lock().unwrap_or_else(|e| e.into_inner());
        for g in &self.guests {
            if let Some((downfall, position)) = personal.get_mut(&g.uuid) {
                let next = (*position + if *downfall { 0.01 } else { -0.01 }).clamp(0.0, 1.0);
                if (next - *position).abs() > f32::EPSILON {
                    *position = next;
                    g.send(&C::GameEvent(play::GameEvent { event: game_event::RAIN_LEVEL, param: next }));
                }
            }
        }
    }
    /// `DedicatedServer.sendLowDiskSpaceWarning`: the players with
'''),
    # ---- `settings.allow-end` and `misc.enable-nether`: a level that's off has no way in.
    (GAME + 'end.rs', '''        (portal, gateway)
    }
    /// EndPortalBlock.getPortalDestination for the pending enter_dimension.
''', '''        (portal && crate::game::paper_fam_core::allow_end(), gateway)
    }
    /// EndPortalBlock.getPortalDestination for the pending enter_dimension.
'''),
    (GAME + 'nether_portal.rs', '''        target.kind() != Dim::Nether
            || self.ui.rules.bool("allow_entering_nether_using_portals")
''', '''        target.kind() != Dim::Nether
            || (crate::game::paper_fam_core::allow_nether() && self.ui.rules.bool("allow_entering_nether_using_portals"))
'''),
    (GAME + 'commands/execute.rs', '''                if !Dim::known(dim) {
''', '''                if !Dim::known(dim) || crate::game::paper_fam_core::level_off(dim) {
'''),
    # ---- `World.setAutoSave`: `ServerLevel.noSave`, which a save at the server's stop goes past.
    (CLIENT + 'game.rs', '''    fn save_level(&mut self, wait: bool) -> Result<()> {
''', '''    fn save_level(&mut self, wait: bool) -> Result<()> {
        if !wait && crate::game::paper_fam_core::no_save(self.dim.id()) {
            return Ok(());
        }
'''),
    (GAME + 'dedicated.rs', '''                    state.autosave = false;
''', '''                    state.autosave = false;
                    crate::game::paper_fam_core::no_save_anywhere(true);
'''),
    (GAME + 'dedicated.rs', '''                    state.autosave = true;
''', '''                    state.autosave = true;
                    crate::game::paper_fam_core::no_save_anywhere(false);
'''),
]

EDITS += [
    # ---- The watchdog's crash report says where the server thread is stuck, as Paper's prints the
    # thread's Java stack: the plugin and listener it is in, and a converted plugin's Java calls.
    ('horizon-client-render/src/crash.rs', '''pub fn report_watchdog(home: &Path, side: &'static str, message: String) -> Option<PathBuf> {
    deliver(
        home,
        Report {
            time: Local::now(),
            description: "Watching Server".into(),
            error: format!("{message} (Watching Server)"),
            stack: String::new(),
''', '''pub fn report_watchdog(home: &Path, side: &'static str, message: String) -> Option<PathBuf> {
    report_watchdog_with(home, side, message, String::new())
}

/// `report_watchdog`, with what the stuck thread was doing as the report's stack.
pub fn report_watchdog_with(home: &Path, side: &'static str, message: String, stack: String) -> Option<PathBuf> {
    deliver(
        home,
        Report {
            time: Local::now(),
            description: "Watching Server".into(),
            error: format!("{message} (Watching Server)"),
            stack,
'''),
    (GAME + 'dedicated.rs', '''                let message = format!(
                    "ServerWatchdog detected that a single server tick took {took} seconds (should be max {should})"
                );
                match horizon_client::crash::report_watchdog(
                    std::path::Path::new("."),
                    "server",
                    message,
                ) {''', '''                let message = format!(
                    "ServerWatchdog detected that a single server tick took {took} seconds (should be max {should})"
                );
                let stuck = horizon_mod_runtime::watch::report(Duration::from_secs(2));
                for line in stuck.lines() {
                    log_from("Server Watchdog", "ERROR", line);
                }
                match horizon_client::crash::report_watchdog_with(
                    std::path::Path::new("."),
                    "server",
                    message,
                    stuck,
                ) {'''),
]
