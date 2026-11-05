# Server speed: each row's gain is measured with tools/perf_bench.py against a build without the rows.
LIB = 'horizon-client/src/'
EDITS = [
    # One write a tick for each player, not one a packet: with TCP_NODELAY each packet was a segment of its own.
    ('horizon-protocol/src/connection.rs', """pub struct ConnWriter {
    stream: Stream,
""", """pub struct ConnWriter {
    corked: bool,
    held: Vec<u8>,
    stream: Stream,
"""),
    ('horizon-protocol/src/connection.rs', """            writer: ConnWriter {
                stream: write,
""", """            writer: ConnWriter {
                corked: false,
                held: Vec::new(),
                stream: write,
"""),
    ('horizon-protocol/src/connection.rs', """    /// Send a frame whose contents [`frame::contents`] made with this
    /// writer's threshold.
    pub fn send_contents(&mut self, contents: &[u8]) -> Result<()> {
        let mut framed = frame::prefixed(contents);
        if let Some(c) = &mut self.encrypt {
            c.encrypt(&mut framed);
        }
        self.stream.write_all(&framed)?;
""", """    /// Hold what's sent until `uncork`: a tick's packets go out as one write, not one each.
    pub fn cork(&mut self) {
        self.corked = true;
    }
    /// Sends what `cork` held.
    pub fn uncork(&mut self) -> Result<()> {
        self.corked = false;
        self.flush_corked()
    }
    fn flush_corked(&mut self) -> Result<()> {
        if !self.held.is_empty() {
            self.stream.write_all(&self.held)?;
            self.held.clear();
        }
        Ok(())
    }
    /// Send a frame whose contents [`frame::contents`] made with this
    /// writer's threshold.
    pub fn send_contents(&mut self, contents: &[u8]) -> Result<()> {
        let mut framed = frame::prefixed(contents);
        if let Some(c) = &mut self.encrypt {
            c.encrypt(&mut framed);
        }
        if self.corked {
            self.held.extend_from_slice(&framed);
            if self.held.len() >= 1 << 18 {
                self.flush_corked()?;
            }
        } else {
            self.stream.write_all(&framed)?;
        }
"""),
    (LIB + 'lan/conn.rs', """        let mut write = |body: Vec<u8>| {
            pending.fetch_sub(body.len(), Ordering::Relaxed);
            send_body(&mut writer, &body).is_ok()
                && pending.load(Ordering::Relaxed) <= MAX_PENDING
        };
""", """        let write = |writer: &mut ConnWriter, body: Vec<u8>| {
            pending.fetch_sub(body.len(), Ordering::Relaxed);
            send_body(writer, &body).is_ok()
                && pending.load(Ordering::Relaxed) <= MAX_PENDING
        };
"""),
    (LIB + 'lan/conn.rs', """            Ok(Out::Packet(body)) => {
                if !write(body) {
                    break;
                }
            }
            Ok(Out::Packets(bodies)) => {
                if !bodies.into_iter().all(&mut write) {
                    break;
                }
            }
""", """            Ok(Out::Packet(body)) => {
                if !write(&mut writer, body) {
                    break;
                }
            }
            Ok(Out::Packets(bodies)) => {
                writer.cork();
                let sent = bodies.into_iter().all(|body| write(&mut writer, body));
                if !sent || writer.uncork().is_err() {
                    break;
                }
            }
"""),
    # A connection that fails while it logs in says why, for whoever reads the server's log.
    (LIB + 'lan/conn.rs', """            let profile = format!("{name} ({})", uuid_string(uuid));
            lost(shared, &profile, "Disconnected");
            Err(err)
""", """            let profile = format!("{name} ({})", uuid_string(uuid));
            if let Some(log) = &shared.config.log {
                (log.write)("Server thread", "WARN", &format!("{profile} failed to log in: {err}"));
            }
            lost(shared, &profile, "Disconnected");
            Err(err)
"""),
    # Spawning asks "who is within 128 blocks" of every chunk: by a grid of players, not a scan of them all.
    (LIB + 'mobs/spawn_selection.rs', """/// `LocalMobCapCalculator`.
#[derive(Clone, Debug)]
pub struct SpawnState {
""", """/// Players by 256-block cell, for what only asks about those within 128 blocks of a point: all
/// of them are in the nine cells around it.
#[derive(Clone, Debug, Default)]
pub struct PlayerCells {
    cells: FxHashMap<(i32, i32), Vec<u32>>,
}
impl PlayerCells {
    pub fn new(players: impl Iterator<Item = (f64, f64)>) -> Self {
        let mut cells: FxHashMap<(i32, i32), Vec<u32>> = FxHashMap::default();
        for (i, (x, z)) in players.enumerate() {
            let key = ((x.floor() as i32) >> 8, (z.floor() as i32) >> 8);
            cells.entry(key).or_default().push(i as u32);
        }
        Self { cells }
    }
    /// Whether `test` holds for a player of the cells around the point.
    pub fn any_near(&self, x: f64, z: f64, mut test: impl FnMut(usize) -> bool) -> bool {
        let (cx, cz) = ((x.floor() as i32) >> 8, (z.floor() as i32) >> 8);
        (cx - 1..=cx + 1).any(|a| {
            (cz - 1..=cz + 1).any(|b| {
                self.cells
                    .get(&(a, b))
                    .is_some_and(|list| list.iter().any(|&i| test(i as usize)))
            })
        })
    }
}

/// `LocalMobCapCalculator`.
#[derive(Clone, Debug)]
pub struct SpawnState {
    cells: PlayerCells,
"""),
    (LIB + 'mobs/spawn_selection.rs', """        let mut state = Self {
            spawnable_chunks,
            counts: [0; 8],
""", """        let mut state = Self {
            cells: PlayerCells::new(players.iter().map(|p| (p.position[0], p.position[2]))),
            spawnable_chunks,
            counts: [0; 8],
"""),
    # A player within 128 blocks of a chunk's centre is within 8 chunks of it, so `tracked` was implied.
    (LIB + 'mobs/spawn_selection.rs', """        let (list, players) = (&mut self.near_list, &self.players);
        let (start, len) = *self
            .players_near
            .entry((cx, cz))
            .or_insert_with(|| {
                let start = list.len() as u32;
                let tracked = players
                    .iter()
                    .any(|p| {
                        let (px, pz) = (
                            (p.position[0].floor() as i32) >> 4,
                            (p.position[2].floor() as i32) >> 4,
                        );
                        (px - cx).abs().max((pz - cz).abs()) <= 8
                    });
                if tracked {
                    let (x, z) = (f64::from(cx * 16 + 8), f64::from(cz * 16 + 8));
                    list.extend(
                        players
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| {
                                let (dx, dz) = (x - p.position[0], z - p.position[2]);
                                !p.spectator && dx * dx + dz * dz < 16384.0
                            })
                            .map(|(i, _)| i),
                    );
                }
                (start, list.len() as u32 - start)
            });
""", """        let (list, players, cells) = (&mut self.near_list, &self.players, &self.cells);
        let (start, len) = *self
            .players_near
            .entry((cx, cz))
            .or_insert_with(|| {
                let start = list.len() as u32;
                let (x, z) = (f64::from(cx * 16 + 8), f64::from(cz * 16 + 8));
                cells.any_near(x, z, |i| {
                    let p = &players[i];
                    let (dx, dz) = (x - p.position[0], z - p.position[2]);
                    if !p.spectator && dx * dx + dz * dz < 16384.0 {
                        list.push(i);
                    }
                    false
                });
                (start, list.len() as u32 - start)
            });
"""),
    (LIB + 'game/natural_spawning.rs', """        let levels = self.tickets.levels();
        candidates
            .retain(|&(cx, cz)| {
                levels.loaded_ticking(cx, cz)
                    && super::ticking_chunk(&self.world, cx, cz)
                    && players.iter().any(|p| close_enough_for_spawning(p, (cx, cz)))
            });
""", """        let levels = self.tickets.levels();
        let cells = spawn_selection::PlayerCells::new(players.iter().map(|p| (p.position.x, p.position.z)));
        candidates
            .retain(|&(cx, cz)| {
                levels.loaded_ticking(cx, cz)
                    && super::ticking_chunk(&self.world, cx, cz)
                    && cells.any_near(f64::from(cx * 16 + 8), f64::from(cz * 16 + 8), |i| {
                        close_enough_for_spawning(&players[i], (cx, cz))
                    })
            });
"""),
    # Sending a chunk looked at nine columns' sections, under a lock, for each column of the batch.
    (LIB + 'lan/chunks.rs', """    /// The non-air sections of the column and its neighbours, as bits. A
""", """    /// [`Self::body_with`] for a batch of columns: each column's sections are looked at once, not
    /// again for every neighbour of it in the batch.
    pub fn body_of_batch(
        &self,
        chunk: &Chunk,
        world: &crate::world::World,
        protos: &dyn Fn(i32, i32) -> Option<SectionMask>,
        seen: &mut FxHashMap<(i32, i32), SectionMask>,
    ) -> Vec<u8> {
        let mut around = SectionMask::EMPTY;
        for dz in -1..=1 {
            for dx in -1..=1 {
                let (x, z) = (chunk.x + dx, chunk.z + dz);
                if let Some(c) = world.chunks.get(&(x, z)) {
                    around |= *seen.entry((x, z)).or_insert_with(|| self.occupied(c));
                } else if let Some(sections) = protos(x, z) {
                    for i in sections.iter() {
                        if self.sections.contains(&i) {
                            around.insert(i - self.sections.start);
                        }
                    }
                }
            }
        }
        self.body(chunk, &world.block_entities, around)
    }

    /// The non-air sections of the column and its neighbours, as bits. A
"""),
    (LIB + 'lan/hub.rs', """        let mut encoded: Vec<Vec<u8>> = columns
            .chunks
            .iter()
            .map(|c| {
                self.encoder
                    .body_with(
                        c,
                        world,
                        &|x, z| { ctx.streaming.and_then(|s| s.proto_sections(x, z)) },
                    )
            })
            .collect();
""", """        let mut seen = crate::fast_hash::FxHashMap::default();
        let mut encoded: Vec<Vec<u8>> = columns
            .chunks
            .iter()
            .map(|c| {
                self.encoder
                    .body_of_batch(
                        c,
                        world,
                        &|x, z| { ctx.streaming.and_then(|s| s.proto_sections(x, z)) },
                        &mut seen,
                    )
            })
            .collect();
"""),
    # Spigot's entity activation range, off unless HORIZON_ACTIVATION names a range.
    (LIB + 'game/creatures.rs', """    /// The entity phase of `tick_creatures`: every mob's tick, deaths, and the
""", """    /// Whether a mob only checks its despawning this tick: with `HORIZON_ACTIVATION` set to a range in
    /// blocks, one past it from every player, standing on the ground with nothing happening to it,
    /// ticks one tick in eight (Spigot's entity activation range).
    fn at_rest(m: &Mob, nearest: Option<horizon_core::geometry::Vec3>, game_time: i64) -> bool {
        static RANGE: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
        let range = *RANGE.get_or_init(|| {
            std::env::var("HORIZON_ACTIVATION").ok().and_then(|text| text.parse().ok()).unwrap_or(0.0)
        });
        range > 0.0
            && (game_time as u64).wrapping_add(u64::from(m.id)) % 8 != 0
            && m.on_ground
            && m.hurt_time == 0
            && m.fire_ticks <= 0
            && m.alive()
            && !m.removed
            && !m.no_ai
            && m.rider.is_none()
            && m.extra.leash.is_none()
            && nearest.is_none_or(|p| {
                let (x, y, z) = (p.x - m.position.x, p.y - m.position.y, p.z - m.position.z);
                x * x + y * y + z * z > range * range
            })
    }
    /// The entity phase of `tick_creatures`: every mob's tick, deaths, and the
"""),
    (LIB + 'game/creatures.rs', """                if !ticking(m.position.x.floor() as i32, m.position.z.floor() as i32) {
""", """                if !ticking(m.position.x.floor() as i32, m.position.z.floor() as i32)
                    || Self::at_rest(m, w.nearest_player, self.world.game_time)
                {
"""),
]
