# Family ev_player. Yours alone. Applied by fabric-port/tools/dim_patch.py, after the mixins: each text must be in its file once.
GAME = 'horizon-client/src/game/'
LAN = 'horizon-client/src/lan/'
FAM = 'crate::game::paper_fam_ev_player'

EDITS = [
    # ---- A connection's thread, where Paper's netty thread runs these.
    (LAN + 'conn.rs', '''            serve_login(c, address, &shared)
''', '''            let (failed, address) = crate::paper_hub::player_handshake(&intention.host, intention.port, intention.protocol, intention.intent, address);
            if let Some(reason) = failed {
                login_disconnect(&mut c, &reason);
                return Ok(());
            }
            serve_login(c, address, &shared)
'''),
    (LAN + 'conn.rs', '''        (None, None) => (offline_uuid(&hello.name), Vec::new()),
''', '''        (None, None) => crate::paper_hub::handshake_profile(offline_uuid(&hello.name)),
'''),
    (LAN + 'conn.rs', '''    if let Some(text) = super::code_of_conduct::pick(
        &shared.config.code_of_conducts,
        &info.language,
    ) {
        accept_code_of_conduct(c, text, &mut info)?;
    }
''', '''    if let Some(text) = crate::paper_hub::code_of_conduct(super::code_of_conduct::pick(
        &shared.config.code_of_conducts,
        &info.language,
    )) {
        accept_code_of_conduct(c, &text, &mut info)?;
    }
'''),
    (LAN + 'conn.rs', '''            Err(e) => return Ended::Closed(format!("Invalid packet {id}: {e}")),
''', '''            Err(e) => {
                if let Some((reason, told)) = crate::paper_hub::illegal_packet(id, &e.to_string()) {
                    return Ended::Kicked(reason, told);
                }
            }
'''),
    (LAN + 'conn.rs', '''    /// The average packets a second passed rate-limit (the average).
    RateExceeded(f32),
}
''', '''    /// The average packets a second passed rate-limit (the average).
    RateExceeded(f32),
    /// A packet that couldn't be read: the reason for the log, and what the client is told.
    Kicked(String, Option<horizon_world::nbt::Tag>),
}
'''),
    (LAN + 'conn.rs', '''            Ended::Closed(reason) => reason,
            Ended::PackDeclined(rejected) => {
''', '''            Ended::Closed(reason) => reason,
            Ended::Kicked(reason, told) => {
                if let Some(told) = told {
                    let _ = tx.send(Out::Disconnect(told));
                }
                reason
            }
            Ended::PackDeclined(rejected) => {
'''),

    # ---- A spawn point being set: the event comes first, so it can change what follows.
    ('horizon-client/src/game.rs', '''            self.respawn_config = Some(config);
            if moved {
                self.tell_sleeper(
                    "block.minecraft.set_spawn",
                    "Respawn point set",
                    false,
                );
            }
            self.spawn = if self.lan_acting.is_some() {
                None
            } else {
                self.level
                    .world_spawn
                    .map(|[x, y, z]| {
                        Vec3::new(f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5)
                    })
                    .or(self.spawn)
            };
        }
''', f'''            let answer = {FAM}::spawn_set(self, config.pos, moved, "BED");
            if !answer.refused {{
                self.respawn_config = Some(answer.applied(config));
                match &answer.line {{
                    Some(line) => {FAM}::spawn_line(self, line),
                    None if answer.notify => self.tell_sleeper(
                        "block.minecraft.set_spawn",
                        "Respawn point set",
                        false,
                    ),
                    None => {{}}
                }}
                self.spawn = if self.lan_acting.is_some() {{
                    None
                }} else {{
                    self.level
                        .world_spawn
                        .map(|[x, y, z]| {{
                            Vec3::new(f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5)
                        }})
                        .or(self.spawn)
                }};
            }}
        }}
'''),
    (GAME + 'respawn_anchor.rs', '''        if !same {
            self.system_message(Text::translate("block.minecraft.set_spawn", &[]));
            self.respawn_config = Some(config);
''', f'''        let answer = if same {{ None }} else {{ Some({FAM}::spawn_set(self, pos, true, "RESPAWN_ANCHOR")) }};
        if let Some(answer) = answer.filter(|answer| !answer.refused) {{
            match &answer.line {{
                Some(line) => {FAM}::spawn_line(self, line),
                None if answer.notify => self.system_message(Text::translate("block.minecraft.set_spawn", &[])),
                None => {{}}
            }}
            self.respawn_config = Some(answer.applied(config));
'''),
    (GAME + 'commands.rs', '''                |g| {
                    if level.kind() == Dim::Overworld {
                        g.spawn = Some(
                            Vec3::new(
                                f64::from(pos[0]) + 0.5,
''', f'''                |g| {{
                    let answer = {FAM}::spawn_set(g, pos, false, "COMMAND");
                    if answer.refused {{
                        return;
                    }}
                    let pos = answer.at.unwrap_or(pos);
                    let level = answer.dim.unwrap_or(level);
                    if level.kind() == Dim::Overworld {{
                        g.spawn = Some(
                            Vec3::new(
                                f64::from(pos[0]) + 0.5,
'''),
    (GAME + 'commands.rs', '''                            dim: level,
                            pos,
                            yaw,
                            forced: true,
                        });
                    }
                },
''', '''                            dim: level,
                            pos,
                            yaw,
                            forced: answer.forced.unwrap_or(true),
                        });
                    }
                },
'''),

    # ---- `PlayerInventorySlotChangeEvent.shouldTriggerAdvancements`.
    (GAME + 'advancements.rs', '''    pub(super) fn check_inventory_trigger(&mut self) {
        use std::hash::{Hash, Hasher};
''', f'''    pub(super) fn check_inventory_trigger(&mut self) {{
        let trigger = {FAM}::slots_changed(self);
        use std::hash::{{Hash, Hasher}};
'''),
    (GAME + 'advancements.rs', '''        self.adv.last_inventory = key;
        let stacks: Vec<ItemView> = self
''', '''        self.adv.last_inventory = key;
        if !trigger {
            return;
        }
        let stacks: Vec<ItemView> = self
'''),

    # ---- `PlayerShearEntityEvent.getDrops`: rolled before the event, and spawned as it left them.
    (GAME + 'creatures.rs', '''        let mut drops: Vec<(String, u32)> = Vec::new();
''', '''        let mut drops: Vec<(String, u32)> = Vec::new();
        let mut shorn: Vec<ItemStack> = Vec::new();
'''),
    (GAME + 'creatures.rs', '''                drops.push((mushroom.into(), 5));
''', f'''                match {FAM}::shear_drops() {{
                    Some(stacks) => shorn = stacks,
                    None => drops.push((mushroom.into(), 5)),
                }}
'''),
    (GAME + 'creatures.rs', '''                let stacks = world_items!(self)
                    .loot_table(
                        &format!("minecraft:shearing/sheep/{color}"),
                        false,
                        None,
                        &mut self.world.random,
                    );
                drops.extend(stacks.into_iter().map(|s| (s.item.to_string(), s.count)));
''', f'''                match {FAM}::shear_drops() {{
                    Some(stacks) => shorn = stacks,
                    None => {{
                        let stacks = world_items!(self)
                            .loot_table(
                                &format!("minecraft:shearing/sheep/{{color}}"),
                                false,
                                None,
                                &mut self.world.random,
                            );
                        drops.extend(stacks.into_iter().map(|s| (s.item.to_string(), s.count)));
                    }}
                }}
'''),
    (GAME + 'creatures.rs', '''        for (item, count) in drops {
            for _ in 0..count {
''', '''        for stack in shorn {
            let v = dispenser::new_item_motion();
            self.spawn_item(stack, [feet[0], feet[1] + 1.0, feet[2]], v, 10);
        }
        for (item, count) in drops {
            for _ in 0..count {
'''),
    (GAME + 'creatures.rs', '''        for stack in self.sequence_loot(table, table, tool.as_ref()) {
            self.spawn_at_location(at, height, stack);
        }
''', f'''        let stacks = match {FAM}::shear_drops() {{
            Some(stacks) => stacks,
            None => self.sequence_loot(table, table, tool.as_ref()),
        }};
        for stack in stacks {{
            self.spawn_at_location(at, height, stack);
        }}
'''),

    # ---- `PlayerHarvestBlockEvent.getItemsHarvested`.
    (GAME + 'spill.rs', '''        let drops = self.items.interact_loot(
            table,
            self.registry,
            state,
            &mut self.world.random,
            self.rand_sequences,
            self.seed,
        );
''', f'''        let drops = match {FAM}::harvest_drops([pos.x, pos.y, pos.z]) {{
            Some(drops) => drops,
            None => self.items.interact_loot(
                table,
                self.registry,
                state,
                &mut self.world.random,
                self.rand_sequences,
                self.seed,
            ),
        }};
'''),

    # ---- `setCreationRadius`, here and for ev_entity's `EntityPortalEvent`.
    ('horizon-world/src/level/portal_travel.rs', '''        for column in spiral_around(origin, 16, Direction::East, Direction::South) {
''', '''        for column in spiral_around(origin, CREATION_RADIUS.load(std::sync::atomic::Ordering::Relaxed), Direction::East, Direction::South) {
'''),
    ('horizon-world/src/level/portal_travel.rs', '''use std::cmp::Ordering;

/// `BlockUtil.FoundRectangle`: the low corner and the sizes along the two axes.
''', '''use std::cmp::Ordering;

/// How far from the exit a portal may be built: 16, unless a plugin says otherwise for this trip.
pub static CREATION_RADIUS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(16);

/// `BlockUtil.FoundRectangle`: the low corner and the sizes along the two axes.
'''),

    # ---- `PlayerServerFullCheckEvent`, where the game decides.
    (LAN + 'mod.rs', '''            if taken >= self.max_players() && !bypass {
                return Some(translatable("multiplayer.disconnect.server_full", &[]));
            }
''', '''            if let Some(reason) = crate::paper_hub::server_full(uuid, name, taken >= self.max_players() && !bypass) {
                return Some(reason);
            }
'''),
    # ---- `PlayerLinksSendEvent`, for this connection's own list.
    (LAN + 'conn.rs', '''    if !shared.config.server_links.is_empty() {
        send(
            c,
            cfg::Clientbound::ServerLinks(cfg::ServerLinks {
                links: shared.config.server_links.clone(),
            }),
        )?;
    }
''', '''    let links = crate::paper_hub::server_links(&shared.config.server_links);
    if !links.is_empty() {
        send(c, cfg::Clientbound::ServerLinks(cfg::ServerLinks { links }))?;
    }
'''),
    # ---- `PlayerConnectionCloseEvent` for a connection that never got to play.
    (LAN + 'conn.rs', '''        Ok(None) => {
            shared.release(uuid);
            Ok(())
        }
        Err(err) => {
            shared.release(uuid);
            let profile = format!("{name} ({})", uuid_string(uuid));
''', '''        Ok(None) => {
            shared.release(uuid);
            crate::paper_hub::connection_closed(uuid, &name, address);
            Ok(())
        }
        Err(err) => {
            shared.release(uuid);
            crate::paper_hub::connection_closed(uuid, &name, address);
            let profile = format!("{name} ({})", uuid_string(uuid));
'''),
    # ---- A thrown potion has its id from the start, for `PlayerLaunchProjectileEvent`.
    (GAME + 'potions.rs', '''            .push(ThrownPotion {
                id: 0,
                stack,
                previous: position,
                position,
                velocity,
                age: 0,
                owner,
            });
    }
}
''', f'''            .push(ThrownPotion {{
                id: {FAM}::thrown_id(),
                stack,
                previous: position,
                position,
                velocity,
                age: 0,
                owner,
            }});
    }}
}}
'''),
    # ---- `PlayerCommandPreprocessEvent.setMessage`: the command that runs is the one it left.
    (GAME + 'lan_actions.rs', '''        if self.guest_message_command(host, conn, command, signed.as_ref()) {
            return;
        }
        let words: Vec<&str> = command.trim_start_matches('/').split(' ').collect();
''', f'''        let rewritten = {FAM}::command_to(self, command);
        let command = rewritten.as_deref().unwrap_or(command);
        let other = {FAM}::command_as(self, host, conn);
        let signed = if rewritten.is_some() || other != conn {{ None }} else {{ signed }};
        let conn = other;
        // The game answers these before its dispatcher: without this, a plugin's /msg never runs.
        if !{FAM}::plugin_word(command.trim_start_matches('/').split(' ').next().unwrap_or_default()) && self.guest_message_command(host, conn, command, signed.as_ref()) {{
            return;
        }}
        let words: Vec<&str> = command.trim_start_matches('/').split(' ').collect();
'''),
    # ---- A plugin's /list comes before the game's, as in Bukkit's command map.
    (GAME + 'lan_actions.rs', '''        if words[0] == "list" && (words.len() == 1 || words[1..] == ["uuids"]) {
''', f'''        if !{FAM}::plugin_word(words[0]) && words[0] == "list" && (words.len() == 1 || words[1..] == ["uuids"]) {{
'''),
    # ---- `Player.setHealth(0)`: nobody's doing, so "died" and not "was killed".
    (GAME + 'commands.rs', '''                self.ui.death_cause = Some("death.attack.genericKill".into());
''', '''                self.ui.death_cause = Some(crate::game::paper_fam_ev_player::kill_cause().into());
'''),
    # ---- `/minecraft:say` by a player: the hub has told everyone, them too, so it isn't their reply as well.
    (GAME + 'commands.rs', '''        if let Some(capture) = self.chat_capture.as_mut() {
            capture.push(line.clone());
        }
        let focused = self.screen == Screen::Chat;
''', '''        if let Some(capture) = self.chat_capture.as_mut().filter(|_| id == "msg" || self.lan_acting.is_none()) {
            capture.push(line.clone());
        }
        let focused = self.screen == Screen::Chat;
'''),
    # ---- `PlayerBedEnterEvent.setUseBed(ALLOW)`: past the distance and the time of day.
    ('horizon-client/src/game.rs', '''        if !reachable(head) && !reachable(foot) {
            self.tell_sleeper(
''', f'''        if !reachable(head) && !reachable(foot) && !{FAM}::bed_forced() {{
            self.tell_sleeper(
'''),
    ('horizon-client/src/game.rs', '''        if !(self.dim.kind() == Dim::Overworld && self.sky_darken() >= 4) {
            self.tell_sleeper(
                "block.minecraft.bed.no_sleep",
''', f'''        if !(self.dim.kind() == Dim::Overworld && self.sky_darken() >= 4) && !{FAM}::bed_forced() {{
            self.tell_sleeper(
                "block.minecraft.bed.no_sleep",
'''),

    # ---- `PlayerFishEvent`'s moments, where they happen.
    (GAME + 'fishing.rs', '''            for stack in items {
                if self
                    .res
                    .items()
                    .item_tags
                    .get("minecraft:fishes")
                    .is_some_and(|t| t.contains(stack.item.as_str()))
                {
                    self.stats.custom("fish_caught", 1);
                }
                let mut item = ItemStack {
''', '''            for stack in items {
                let fish = self
                    .res
                    .items()
                    .item_tags
                    .get("minecraft:fishes")
                    .is_some_and(|t| t.contains(stack.item.as_str()));
                let mut item = ItemStack {
'''),
    (GAME + 'fishing.rs', '''                    [hook.position.x, hook.position.y, hook.position.z],
                    v,
                    0,
                );
                let xp = self.world.random.next_int_bound(6).unwrap_or(0) as u32 + 1;
''', '''                    [hook.position.x, hook.position.y, hook.position.z],
                    v,
                    0,
                );
                if @F::fish_caught(self, hook.id) {
                    self.hook = Some(hook);
                    return 0;
                }
                if fish {
                    self.stats.custom("fish_caught", 1);
                }
                let xp = self.world.random.next_int_bound(6).unwrap_or(0) as u32 + 1;
'''),
    (GAME + 'fishing.rs', '''            if h.nibble <= 0 {
                h.time_until_lured = 0;
                h.time_until_hooked = 0;
            }
''', '''            if h.nibble <= 0 {
                h.time_until_lured = 0;
                h.time_until_hooked = 0;
                @F::fish_state(self, h.id, "FAILED_ATTEMPT");
            }
'''),
    (GAME + 'fishing.rs', '''            } else {
                let pitch = 1.0
                    + (self.world.random.next_float() - self.world.random.next_float())
                        * 0.4;
                self.audio
                    .play(
                        "minecraft:entity.fishing_bobber.splash",
''', '''            } else {
                if @F::fish_state(self, h.id, "BITE") {
                    return;
                }
                let pitch = 1.0
                    + (self.world.random.next_float() - self.world.random.next_float())
                        * 0.4;
                self.audio
                    .play(
                        "minecraft:entity.fishing_bobber.splash",
'''),
    (GAME + 'fishing.rs', '''            if h.time_until_lured <= 0 {
                h.fish_angle = if''', '''            if h.time_until_lured <= 0 {
                if @F::fish_state(self, h.id, "LURED") {
                    h.time_until_lured = h.rules.wait_ticks(&mut self.world.random, h.lure);
                    return;
                }
                h.fish_angle = if'''),

    # ---- `PlayerReadyArrowEvent`, each time the game looks for ammunition: a refused one goes on to the next.
    (GAME + 'crossbow.rs', '''    pub(super) fn projectile_slot(&self, weapon: &str) -> Option<usize> {
''', '''    /// Every slot `projectile_slot` could give, in the order it looks.
    pub(super) fn projectile_slots(&self, weapon: &str) -> Vec<usize> {
        let held = |s: &ItemStack| is_arrow(s) || weapon == CROSSBOW && &*s.item == FIREWORK_ROCKET;
        let mut slots: Vec<usize> = [OFFHAND_SLOT, self.selected]
            .into_iter()
            .find(|slot| self.stack_in(*slot).is_some_and(held))
            .into_iter()
            .collect();
        for (slot, stack) in self.inventory.iter().enumerate() {
            if stack.as_ref().is_some_and(is_arrow) && !slots.contains(&slot) {
                slots.push(slot);
            }
        }
        slots
    }
    pub(super) fn projectile_slot(&self, weapon: &str) -> Option<usize> {
'''),
    (GAME + 'crossbow.rs', '''        } else if (self.projectile_slot(CROSSBOW).is_some() || self.creative)
            && self.using.is_none()
        {
''', '''        } else if self.using.is_none()
            && (@F::ready_slot(self, CROSSBOW).is_some() || self.creative)
        {
'''),
    (GAME + 'crossbow.rs', '''            let slot_ammo = self.projectile_slot(CROSSBOW);
''', '''            let slot_ammo = @F::ready_slot(self, CROSSBOW);
'''),
    ('horizon-client/src/game.rs', '''            if self.using.is_none()
                && (self.creative || self.projectile_slot("minecraft:bow").is_some())
            {
''', '''            if self.using.is_none()
                && (@F::ready_slot(self, "minecraft:bow").is_some() || self.creative)
            {
'''),
    ('horizon-client/src/game.rs', '''        let arrow_slot = self.projectile_slot("minecraft:bow");
''', '''        let arrow_slot = @F::ready_slot(self, "minecraft:bow");
'''),

    # ---- `PlayerStatisticIncrementEvent`: what a listener could stop waits for it.
    (GAME + 'stats.rs', '''    /// Statistic changes (type, key, amount; `None` = reset) for the
''', '''    /// Waiting for `PlayerStatisticIncrementEvent`: type, key, amount.
    pub(super) asked: Vec<(String, String, i32)>,
    /// Statistic changes (type, key, amount; `None` = reset) for the
'''),
    (GAME + 'stats.rs', '''            |ty| {
                let v = entry(entry(&mut self.values, ty), key);
''', '''            |ty| {
                if @F::stat_asked(n, key) {
                    self.asked.push((ty.to_owned(), key.to_owned(), n));
                    return;
                }
                let v = entry(entry(&mut self.values, ty), key);
'''),

    # ---- `PlayerAdvancementCriterionGrantEvent`: asked before it goes in.
    (GAME + 'advancements.rs', '''            self.adv.progress.grant(&a.id, &a.criteria[ci].name, &date);
            self.progress_changed(ai);
''', '''            if @F::criterion_refused(self, &a.id, &a.criteria[ci].name) {
                continue;
            }
            self.adv.progress.grant(&a.id, &a.criteria[ci].name, &date);
            self.progress_changed(ai);
'''),
    (GAME + 'advancements.rs', '''            let was = self.adv.progress.done(a);
            if self.adv.progress.grant(&a.id, c, &date) {
''', '''            let was = self.adv.progress.done(a);
            if !self.adv.progress.has(&a.id, c) && @F::criterion_refused(self, &a.id, c) {
                continue;
            }
            if self.adv.progress.grant(&a.id, c, &date) {
'''),

    # ---- A merchant's screen waits for `InventoryOpenEvent` like the rest.
    (GAME + 'trading.rs', '''    fn open_merchant_state(&mut self, state: MerchantState) {
''', '''    fn open_merchant_state(&mut self, state: MerchantState) {
        let run = |g: &mut Game<'_>, mut state: MerchantState, refused: bool, title: Option<String>| {
            if refused {
                g.merchant_closed(&mut state);
                return;
            }
            if title.is_some() {
                state.name = title;
            }
            g.open_merchant_state(state);
        };
        let Some(state) = @F::merchant_later(self, state, run) else {
            return;
        };
'''),

    # ---- A portal exit a plugin said not to build: back the way they came.
    (GAME + 'server_travel.rs', '''                        let (at, rotation) = self.arrival(&plan, exit);
                        (at, yaw + rotation)
                    }
                    _ => (trip.target, yaw),
''', '''                        let (at, rotation) = self.arrival(&plan, exit);
                        (at, yaw + rotation)
                    }
                    _ => {
                        if let Some((dim, at)) = @F::portal_back() {
                            self.level_teleports.push(LevelTeleport { conn: trip.conn, dim, at, rotation: None });
                        }
                        (trip.target, yaw)
                    }
'''),

    # ---- Play back to configuration (`ServerGamePacketListenerImpl.switchToConfig`).
    (LAN + 'mod.rs', '''    Disconnect(horizon_world::nbt::Tag),
    Close,
}
''', '''    Disconnect(horizon_world::nbt::Tag),
    /// The client has gone to configuration: the reader takes the connection back.
    Hand,
    Close,
}
'''),
    (LAN + 'conn.rs', '''    /// A network connection's keep-alive clock (none for a test's channel).
    watch: Option<Arc<KeepAliveWatch>>,
}
impl PlayerSender {
''', '''    /// A network connection's keep-alive clock (none for a test's channel).
    watch: Option<Arc<KeepAliveWatch>>,
    /// Told to go to configuration: its client no longer reads play packets.
    pub paused: Arc<std::sync::atomic::AtomicBool>,
}
impl PlayerSender {
'''),
    (LAN + 'conn.rs', '''            posted: Arc::default(),
            watch: None,
        }
    }
''', '''            posted: Arc::default(),
            watch: None,
            paused: Arc::default(),
        }
    }
'''),
    (LAN + 'conn.rs', '''    pub fn send(&self, body: Vec<u8>) {
        self.pending.fetch_add(body.len(), Ordering::Relaxed);
''', '''    pub fn send(&self, body: Vec<u8>) {
        if self.paused.load(Ordering::Relaxed) {
            return;
        }
        self.pending.fetch_add(body.len(), Ordering::Relaxed);
'''),
    (LAN + 'conn.rs', '''    pub fn send_many(&self, bodies: impl IntoIterator<Item = Vec<u8>>) {
        if !HOLDING.get() {
''', '''    pub fn send_many(&self, bodies: impl IntoIterator<Item = Vec<u8>>) {
        if self.paused.load(Ordering::Relaxed) {
            return;
        }
        if !HOLDING.get() {
'''),
    (LAN + 'conn.rs', '''    pending: Arc<AtomicUsize>,
) {
    let mut sent_at: Option<(i64, Instant)> = None;
''', '''    pending: Arc<AtomicUsize>,
    paused: Arc<std::sync::atomic::AtomicBool>,
) -> Option<ConnWriter> {
    let mut sent_at: Option<(i64, Instant)> = None;
'''),
    (LAN + 'conn.rs', '''            Ok(Out::Close) | Err(RecvTimeoutError::Disconnected) => break,
''', '''            Ok(Out::Hand) => return Some(writer),
            Ok(Out::Close) | Err(RecvTimeoutError::Disconnected) => break,
'''),
    (LAN + 'conn.rs', '''        if sent_at.is_none() && now >= next {
            let id = i64::try_from(start.elapsed().as_millis()).unwrap_or(0);
''', '''        if sent_at.is_none() && now >= next && !paused.load(Ordering::Relaxed) {
            let id = i64::try_from(start.elapsed().as_millis()).unwrap_or(0);
'''),
    (LAN + 'conn.rs', '''    writer.shutdown();
}
#[cfg(test)]
''', '''    writer.shutdown();
    None
}
#[cfg(test)]
'''),
    (LAN + 'conn.rs', '''fn play(
    c: Connection,
    shared: &Arc<Shared>,
    parts: Parts,
    address: std::net::SocketAddr,
) {
''', '''/// A connection in play, through each time it goes back to configuration: it leaves the game as
/// the configuration begins and joins it again as a new player when that ends, as vanilla's does.
fn play(
    c: Connection,
    shared: &Arc<Shared>,
    parts: Parts,
    address: std::net::SocketAddr,
) {
    let (mut c, mut parts) = (c, parts);
    loop {
        let Some((mut back, uuid, name, properties, seen)) = play_once(c, shared, parts, address) else {
            return;
        };
        crate::paper_hub::reconfiguring(true);
        let again = configure(&mut back, shared, uuid, &name, properties, Some(seen));
        crate::paper_hub::reconfiguring(false);
        match again {
            Ok(Some(next)) => {
                c = back;
                parts = next;
            }
            _ => {
                shared.release(uuid);
                crate::paper_hub::connection_closed(uuid, &name, address);
                return;
            }
        }
    }
}
/// `play` until the connection closes, or goes back to configuration: then what that needs.
fn play_once(
    c: Connection,
    shared: &Arc<Shared>,
    parts: Parts,
    address: std::net::SocketAddr,
) -> Option<(Connection, u128, String, Vec<login::Property>, (ClientInformation, String))> {
'''),
    (LAN + 'conn.rs', '''    let writer_thread = {
        let (alive, latency, pending) = (
''', '''    let paused = sender.paused.clone();
    let writer_thread = {
        let paused = paused.clone();
        let (alive, latency, pending) = (
'''),
    (LAN + 'conn.rs', '''                write_loop(writer, rx, start, alive, latency, pending)
''', '''                write_loop(writer, rx, start, alive, latency, pending, paused)
'''),
    (LAN + 'conn.rs', '''    let name = parts.name.clone();
    let joined = Joined {
''', '''    let name = parts.name.clone();
    let again = (parts.uuid, parts.name.clone(), parts.properties.clone(), (parts.info.clone(), parts.brand.clone()));
    let joined = Joined {
'''),
    (LAN + 'conn.rs', '''            required_pack,
            shared.config.rate_limit,
        ) {
            Ended::Closed(reason) => reason,
''', '''            required_pack,
            shared.config.rate_limit,
            &paused,
        ) {
            Ended::Reconfigure => {
                let _ = tx.send(Out::Hand);
                let writer = writer_thread.and_then(|thread| thread.join().ok()).flatten();
                let _ = events.send(NetEvent::Left(conn_id, "Reconfiguring".to_owned()));
                let Some(writer) = writer else {
                    shared.release(parts.uuid);
                    return None;
                };
                return Some((Connection { reader, writer }, again.0, again.1, again.2, again.3));
            }
            Ended::Closed(reason) => reason,
'''),
    (LAN + 'conn.rs', '''    shared.release(parts.uuid);
}
/// RateKickingConnection: the packets a second a connection averages, as
''', '''    shared.release(parts.uuid);
    None
}
/// RateKickingConnection: the packets a second a connection averages, as
'''),
    (LAN + 'conn.rs', '''    Kicked(String, Option<horizon_world::nbt::Tag>),
}
''', '''    Kicked(String, Option<horizon_world::nbt::Tag>),
    /// The client acknowledged going back to configuration.
    Reconfigure,
}
'''),
    (LAN + 'conn.rs', '''    required_pack: bool,
    rate_limit: i32,
) -> Ended {
''', '''    required_pack: bool,
    rate_limit: i32,
    paused: &std::sync::atomic::AtomicBool,
) -> Ended {
'''),
    (LAN + 'conn.rs', '''            Ok(play::Serverbound::KeepAlive(k)) => alive.store(k.id, Ordering::Relaxed),
''', '''            Ok(play::Serverbound::ConfigurationAcknowledged(_)) if paused.load(Ordering::Relaxed) => {
                return Ended::Reconfigure;
            }
            Ok(play::Serverbound::KeepAlive(k)) => alive.store(k.id, Ordering::Relaxed),
'''),
    (LAN + 'conn.rs', '''    let sync_data = shared.sync();
    let sync: &SyncData = &sync_data;
    let send = |c: &mut Connection, p: cfg::Clientbound| c.send(&p.encode()).map_err(e);
''', '''    configure(c, shared, uuid, name, properties, None)
}
/// The configuration phase. `again` is a player back from play, with what the client said the
/// first time, which it doesn't say twice: it stays here until `completeReconfiguration`.
fn configure(
    c: &mut Connection,
    shared: &Arc<Shared>,
    uuid: u128,
    name: &str,
    properties: Vec<login::Property>,
    again: Option<(ClientInformation, String)>,
) -> Result<Option<Parts>, String> {
    let e = |e: horizon_protocol::Error| e.to_string();
    let reconfiguring = again.is_some();
    let sync_data = shared.sync();
    let sync: &SyncData = &sync_data;
    let send = |c: &mut Connection, p: cfg::Clientbound| c.send(&p.encode()).map_err(e);
'''),
    (LAN + 'conn.rs', '''    let mut info = ClientInformation::new(10);
    let mut brand = String::from("vanilla");
    let known_pack = loop {
''', '''    let (mut info, mut brand) = again.unwrap_or_else(|| (ClientInformation::new(10), String::from("vanilla")));
    let known_pack = loop {
'''),
    (LAN + 'conn.rs', '''    send(c, cfg::Clientbound::FinishConfiguration(Empty))?;
    loop {
        let (id, payload) = c.recv().map_err(e)?;
        match decode_config(id, &payload).map_err(e)? {
            cfg::Serverbound::FinishConfiguration(_) => break,
''', '''    if reconfiguring {
        hold_configuration(c, uuid, &mut info)?;
        return Ok(Some(Parts { uuid, name: name.to_owned(), properties, info, brand }));
    }
    send(c, cfg::Clientbound::FinishConfiguration(Empty))?;
    loop {
        let (id, payload) = c.recv().map_err(e)?;
        match decode_config(id, &payload).map_err(e)? {
            cfg::Serverbound::FinishConfiguration(_) => break,
'''),
    (LAN + 'conn.rs', '''/// ServerboundResourcePackPacket.Action.DECLINED.
const PACK_DECLINED: i32 = 1;
''', '''/// A reconfiguring player stays here until they're let go: `PlayerConnectionReconfigureEvent`, the
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
                    next += KEEP_ALIVE_EVERY;
                    let id = i64::try_from(start.elapsed().as_millis()).unwrap_or(0);
                    if writer.send(&cfg::Clientbound::KeepAlive(cfg::KeepAlive { id }).encode()).is_err() {
                        return;
                    }
                }
            }
        });
        let result = loop {
            let packet = match c.reader.recv() {
                Ok((id, payload)) => decode_config(id, &payload).map_err(|e| e.to_string()),
                Err(err) => break Err(err.to_string()),
            };
            match packet {
                Ok(cfg::Serverbound::FinishConfiguration(_)) => break Ok(()),
                Ok(cfg::Serverbound::ClientInformation(i)) => *info = i,
                Ok(_) => {}
                Err(err) => break Err(err),
            }
        };
        let _ = stop.send(());
        result
    })
}
/// ServerboundResourcePackPacket.Action.DECLINED.
const PACK_DECLINED: i32 = 1;
'''),

    # ---- `AsyncPlayerPreLoginEvent`, with the profile still the game's to change.
    (LAN + 'conn.rs', '''    if let Some(reason) = shared.try_admit(address, uuid, &name) {
        let text = refuse(&mut c, shared, "Server thread", &user, &reason);
''', '''    let (uuid, properties) = match crate::paper_hub::pre_login(uuid, &mut name, properties) {
        Ok(profile) => profile,
        Err(reason) => {
            let text = refuse(&mut c, shared, "Server thread", &user, &reason);
            lost(shared, &user, &text);
            return Ok(());
        }
    };
    if let Some(reason) = shared.try_admit(address, uuid, &name) {
        let text = refuse(&mut c, shared, "Server thread", &user, &reason);
'''),
    # ---- `PlayerTrackEntityEvent`, before the entity goes out.
    (LAN + 'entity.rs', '''fn sees(v: &Viewer, p: &Probe, chunks: &[(i32, i32)], held: &mut [u8]) -> bool {
''', '''fn sees(v: &Viewer, p: &Probe, chunks: &[(i32, i32)], held: &mut [u8]) -> bool {
    sees_own(v, p, chunks, held) && crate::paper_hub::track_allowed(v.entity_id, p.id)
}
fn sees_own(v: &Viewer, p: &Probe, chunks: &[(i32, i32)], held: &mut [u8]) -> bool {
'''),
    # PlayerUnleashEntityEvent: shears on a mob, shears on a knot, and the holder's click.
    (GAME + 'leads.rs', '''            if self.mobs[index].extra.leash.is_some() {
                self.break_leash(id, true, false);
                dropped = true;
            }''', '''            if self.mobs[index].extra.leash.is_some()
                && let Some(drop) = @F::unleash(self, id, true)
            {
                self.break_leash(id, drop, false);
                dropped = true;
            }'''),
    (GAME + 'leads.rs', '''            for o in others {
                self.break_leash(o, true, false);
                dropped = true;
            }''', '''            for o in others {
                if let Some(drop) = @F::unleash(self, o, true) {
                    self.break_leash(o, drop, false);
                    dropped = true;
                }
            }'''),
    (GAME + 'leads.rs', '''            for i in existing {
                let id = self.mobs[i].id;
                self.break_leash(id, true, false);
            }''', '''            for i in existing {
                let id = self.mobs[i].id;
                if let Some(drop) = @F::unleash(self, id, true) {
                    self.break_leash(id, drop, false);
                }
            }'''),
    (GAME + 'leads.rs', '''            let creative = self.creative;
            horizon_client::mobs::ee::unleash_as("PLAYER_UNLEASH");
            self.break_leash(id, !creative, false);
''', '''            let creative = self.creative;
            let Some(drop) = @F::unleash(self, id, !creative) else {
                return false;
            };
            self.break_leash(id, drop, false);
'''),
    # PlayerBucketEvent.setItemStack: the stack goes where the filled bucket would have.
    ('horizon-client/src/game.rs', '''            let filled = ItemStack::new(filled, 1);
            self.filled_result(filled.clone());
            self.trigger_item("minecraft:filled_bucket", &filled);
            return Ok(UseResult::Success { swing: true });''', '''            let filled = ItemStack::new(filled, 1);
            self.filled_result(@F::bucket_filled(filled.clone()));
            self.trigger_item("minecraft:filled_bucket", &filled);
            return Ok(UseResult::Success { swing: true });'''),
    (GAME + 'powder_snow.rs', '''        self.filled_result(filled.clone());''', '''        self.filled_result(@F::bucket_filled(filled.clone()));'''),
    ('horizon-client/src/game.rs', '''            if !self.creative {
                *self.held_mut() = Some(ItemStack::new("minecraft:bucket", 1));
            }
            return Ok(UseResult::Success { swing: true });''', '''            match @F::bucket_left() {
                Some(left) => *self.held_mut() = left,
                None if !self.creative => {
                    *self.held_mut() = Some(ItemStack::new("minecraft:bucket", 1));
                }
                None => {}
            }
            return Ok(UseResult::Success { swing: true });'''),
    # The three toggle packets are heard before the hub sets its flag. A move behind one waits for it.
    (LAN + 'hub.rs', '''                S::PlayerInput(p) => {
                    let g = &mut self.guests[i];
                    g.input = p.flags;
                    if g.client_loaded() {
                        g.sneaking = p.flags & 32 != 0;
                    }
                }
''', '''                S::PlayerInput(p) if crate::paper_hub::toggle_held(self.guests[i].client_loaded() && self.guests[i].sneaking != (p.flags & 32 != 0)) => {
                    events.push(HostEvent::Packet(conn, Box::new(S::PlayerInput(p))));
                    if self.move_queued(i) {
                        break;
                    }
                }
                S::PlayerInput(p) => {
                    let g = &mut self.guests[i];
                    g.input = p.flags;
                    if g.client_loaded() {
                        g.sneaking = p.flags & 32 != 0;
                    }
                }
'''),
    (LAN + 'hub.rs', '''                S::PlayerCommand(p) => {
                    match p.action {
                        1 | 2 => self.set_sprinting(i, p.action == 1),
''', '''                S::PlayerCommand(p) if matches!(p.action, 1 | 2) && crate::paper_hub::toggle_held(self.guests[i].sprinting != (p.action == 1)) => {
                    events.push(HostEvent::Packet(conn, Box::new(S::PlayerCommand(p))));
                    if self.move_queued(i) {
                        break;
                    }
                }
                S::PlayerCommand(p) => {
                    match p.action {
                        1 | 2 => self.set_sprinting(i, p.action == 1),
'''),
    (LAN + 'hub.rs', '''                S::PlayerAbilities(p) => {
                    let g = &mut self.guests[i];
                    g.flying = p.flags & 2 != 0 && (g.creative() || g.spectator());
                }
''', '''                S::PlayerAbilities(p) if crate::paper_hub::toggle_held(self.may_fly(i) && self.guests[i].flying != (p.flags & 2 != 0)) => {
                    events.push(HostEvent::Packet(conn, Box::new(S::PlayerAbilities(p))));
                    if self.move_queued(i) {
                        break;
                    }
                }
                S::PlayerAbilities(p) => {
                    let may_fly = self.may_fly(i);
                    self.guests[i].flying = p.flags & 2 != 0 && may_fly;
                }
'''),
    (LAN + 'hub.rs', '''                    if let S::PlayerAbilities(p) = packet {
                        crate::paper_hub::flight_wanted(g.uuid, p.flags & 2 != 0);
                    }
''', '''                    if let S::PlayerAbilities(p) = packet
                        && !crate::paper_hub::toggle_held(true)
                    {
                        crate::paper_hub::flight_wanted(g.uuid, p.flags & 2 != 0);
                    }
'''),
    (LAN + 'hub.rs', '''    fn process_inbox(
        &mut self,
        i: usize,
''', '''    /// True if this guest may fly: by their mode, or a plugin's word.
    fn may_fly(&self, i: usize) -> bool {
        let g = &self.guests[i];
        g.creative() || g.spectator() || crate::paper_hub::flight_allowed(g.uuid)
    }
    /// What a toggle packet does, once the plugins have heard it.
    pub fn toggle_packet(&mut self, conn: u32, packet: S) {
        let Some(i) = self.guests.iter().position(|g| g.conn == conn) else {
            return;
        };
        let may_fly = self.may_fly(i);
        match packet {
            S::PlayerInput(p) => {
                let g = &mut self.guests[i];
                g.input = p.flags;
                if g.client_loaded() {
                    g.sneaking = p.flags & 32 != 0;
                }
            }
            S::PlayerCommand(p) => self.set_sprinting(i, p.action == 1),
            S::PlayerAbilities(p) => self.guests[i].flying = p.flags & 2 != 0 && may_fly,
            _ => {}
        }
    }
    fn process_inbox(
        &mut self,
        i: usize,
'''),
    # PlayerInteractEvent's two results: what the click may use.
    ('horizon-client/src/game.rs', '''            if matches!(self.guest_use, Some(GuestUse::OnBlock(_)))
                && let Some(hit) = self.pick()
            {
                self.spectator_open_menu(hit.pos);
            }
''', '''            if matches!(self.guest_use, Some(GuestUse::OnBlock(_)))
                && !@F::block_denied()
                && let Some(hit) = self.pick()
            {
                self.spectator_open_menu(hit.pos);
            }
'''),
    ('horizon-client/src/game.rs', '''        if !on_block && self.riding.is_some()
            && let Some(item) = self.held().map(|s| s.item.clone())
''', '''        if !on_block && !@F::item_denied() && self.riding.is_some()
            && let Some(item) = self.held().map(|s| s.item.clone())
'''),
    ('horizon-client/src/game.rs', '''            if on_block && self.with_hand(hand, Game::interact_knot) {
''', '''            if on_block && !@F::block_denied() && self.with_hand(hand, Game::interact_knot) {
'''),
    ('horizon-client/src/game.rs', '''        if !(sneaking && hands_full) && let Some(hit) = block_hit {
            let held = self.held().cloned();
            if self.use_lectern(hit.pos, held.as_ref()) {
''', '''        if !(sneaking && hands_full) && !@F::block_denied() && let Some(hit) = block_hit {
            let held = self.held().cloned();
            if self.use_lectern(hit.pos, held.as_ref()) {
'''),
    ('horizon-client/src/game.rs', '''        let Some(stack) = self.held().cloned() else {
            return Ok(false);
        };
        let Some(info) = self.items().get(&stack.item).cloned() else {
            return Ok(false);
        };
        if self
            .cooldowns
''', '''        if @F::item_denied()
            || @F::block_denied() && matches!(self.guest_use, Some(GuestUse::OnBlock(_)))
        {
            return Ok(false);
        }
        let Some(stack) = self.held().cloned() else {
            return Ok(false);
        };
        let Some(info) = self.items().get(&stack.item).cloned() else {
            return Ok(false);
        };
        if self
            .cooldowns
'''),
    (GAME + 'lan_actions.rs', '''                                    let state = g.world.block(a.pos[0], a.pos[1], a.pos[2]);
                                    let mut canceled = false;
                                    g.native_left_click_block(
''', '''                                    let state = g.world.block(a.pos[0], a.pos[1], a.pos[2]);
                                    let mut canceled = @F::block_denied() || @F::item_denied();
                                    g.native_left_click_block(
'''),
    (GAME + 'lan_actions.rs', '''    fn under_spawn_protection(&self, host: &LanHost, gi: usize, pos: [i32; 3]) -> bool {''', '''    pub(super) fn under_spawn_protection(&self, host: &LanHost, gi: usize, pos: [i32; 3]) -> bool {'''),
    (GAME + 'lan_actions.rs', '''    fn inside_border(&self, target: &Aabb) -> bool {''', '''    pub(super) fn inside_border(&self, target: &Aabb) -> bool {'''),
    # PlayerGameModeChangeEvent for what no command does.
    (GAME + 'commands.rs', '''            if changed != Some(true) {
                continue;
            }
            count += 1;
            if own {
                self.feedback(
                    Text::translate("commands.gamemode.success.self", &[mode_name]),
''', '''            if let Some(message) = @F::mode_message() {
                self.feedback_unbroadcast(horizon_client_render::packs::component(&message));
            }
            if changed != Some(true) {
                continue;
            }
            count += 1;
            if own {
                self.feedback(
                    Text::translate("commands.gamemode.success.self", &[mode_name]),
'''),
    (GAME + 'lan.rs', '''    for conn in stale {
        host.hub.set_game_mode(conn, mode, world, Some(collision));
    }
''', '''    for conn in stale {
        if !@F::mode_held(conn, mode, "DEFAULT_GAMEMODE") {
            host.hub.set_game_mode(conn, mode, world, Some(collision));
        }
    }
'''),
    (GAME + 'lan_actions.rs', '''        if self.level.hardcore {
            host.hub.set_game_mode(conn, SPECTATOR, &self.world, Some(self.collision));
        }
''', '''        if self.level.hardcore && !@F::mode_held(conn, SPECTATOR, "HARDCORE_DEATH") {
            host.hub.set_game_mode(conn, SPECTATOR, &self.world, Some(self.collision));
        }
'''),
    (GAME + 'integrated.rs', '''            host.hub
                .set_game_mode(conn, mode, &self.world, Some(self.collision));
''', '''            if !@F::mode_held(conn, mode, "UNKNOWN") {
                host.hub.set_game_mode(conn, mode, &self.world, Some(self.collision));
            }
'''),
    # PlayerFailMoveEvent: the hub holds a move it would turn down, and the turn answers.
    (LAN + 'hub.rs', '''        let d = [0, 1, 2].map(|k| to[k] - g.first_good[k]);
        let moved = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if g.sleeping {
''', '''        let let_through = crate::paper_hub::move_let_through(g.uuid);
        let d = [0, 1, 2].map(|k| to[k] - g.first_good[k]);
        let moved = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if g.sleeping {
'''),
    (LAN + 'hub.rs', '''                if moved - expected > limit * f64::from(packets) {
                    events
                        .push(
                            HostEvent::Warn(
                                format!(
                                    "{} moved too quickly! {},{},{}", g.name, crate
                                    ::java_text::double_to_string(d[0]), crate
                                    ::java_text::double_to_string(d[1]), crate
                                    ::java_text::double_to_string(d[2])
                                ),
                            ),
                        );
                    self.resend_teleport(i, None);
                    return;
                }
''', '''                if moved - expected > limit * f64::from(packets) && !let_through {
                    let line = format!(
                        "{} moved too quickly! {},{},{}", g.name, crate
                        ::java_text::double_to_string(d[0]), crate
                        ::java_text::double_to_string(d[1]), crate
                        ::java_text::double_to_string(d[2])
                    );
                    let row = crate::paper_hub::FailedMove {
                        uuid: g.uuid,
                        reason: "MOVED_TOO_QUICKLY",
                        from: g.position,
                        to,
                        rot: (yaw, pitch),
                        flags,
                        line: line.clone(),
                        refused: true,
                        back: None,
                    };
                    if crate::paper_hub::move_failed(row) {
                        return;
                    }
                    events.push(HostEvent::Warn(line));
                    self.resend_teleport(i, None);
                    return;
                }
'''),
    (LAN + 'hub.rs', '''        let (wrongly, refused, stands) = self.collide(i, to, ctx);
        if wrongly {
            let line = format!("{} moved wrongly!", self.guests[i].name);
            events.push(HostEvent::Warn(line));
        }
        if refused {
            let g = &mut self.guests[i];
            (g.yaw, g.pitch) = (yaw, pitch);
            self.resend_teleport(i, Some(start));
            return;
        }
''', '''        let (wrongly, refused, stands) = self.collide(i, to, ctx);
        let (wrongly, refused) = (wrongly && !let_through, refused && !let_through);
        let line = format!("{} moved wrongly!", self.guests[i].name);
        let held = (wrongly || refused) && crate::paper_hub::move_failed(crate::paper_hub::FailedMove {
            uuid: self.guests[i].uuid,
            reason: if wrongly { "MOVED_WRONGLY" } else { "CLIPPED_INTO_BLOCK" },
            from: start,
            to,
            rot: (yaw, pitch),
            flags,
            line: line.clone(),
            refused,
            back: Some(start),
        });
        if wrongly && !held {
            events.push(HostEvent::Warn(line));
        }
        if refused {
            let g = &mut self.guests[i];
            (g.yaw, g.pitch) = (yaw, pitch);
            if !held {
                self.resend_teleport(i, Some(start));
            }
            return;
        }
'''),
    (LAN + 'hub.rs', '''    /// True if this guest may fly: by their mode, or a plugin's word.
''', '''    /// `PlayerFailMoveEvent`'s answer to a move held back: sent back, or taken with no checks.
    pub fn move_answered(&mut self, conn: u32, allowed: bool, back: Option<[f64; 3]>, to: [f64; 3], rot: (f32, f32), flags: u8) {
        let Some(i) = self.guests.iter().position(|g| g.conn == conn) else {
            return;
        };
        if !allowed {
            self.resend_teleport(i, back);
            return;
        }
        let g = &mut self.guests[i];
        crate::paper_hub::let_move_through(g.uuid);
        let packet = S::MovePlayerPosRot(play::MovePlayerPosRot { x: to[0], y: to[1], z: to[2], yaw: rot.0, pitch: rot.1, flags });
        g.inbox.push_front((packet, 0));
    }
    /// True if this guest may fly: by their mode, or a plugin's word.
'''),
    (LAN + 'hub.rs', '''            S::PlayerAbilities(p) => self.guests[i].flying = p.flags & 2 != 0 && may_fly,
            _ => {}
''', '''            S::PlayerAbilities(p) => self.guests[i].flying = p.flags & 2 != 0 && may_fly,
            S::RecipeBookChangeSettings(p) => {
                if let Some(book) = crate::recipe_book::BookType::ALL.get(p.book_type as usize) {
                    let settings = &mut self.guests[i].recipe_book;
                    settings.open[book.index()] = p.open;
                    settings.filtering[book.index()] = p.filtering;
                }
            }
            _ => {}
'''),
    (LAN + 'hub.rs', '''                S::RecipeBookChangeSettings(p) => {
                    if let Some(book) = crate::recipe_book::BookType::ALL
''', '''                S::RecipeBookChangeSettings(p) if crate::paper_hub::toggle_held(true) => {
                    events.push(HostEvent::Packet(conn, Box::new(S::RecipeBookChangeSettings(p))));
                }
                S::RecipeBookChangeSettings(p) => {
                    if let Some(book) = crate::recipe_book::BookType::ALL
'''),
    # setCanCreatePortal(false): looked for before they go. portal_blocks reads the level from disk.
    (GAME + 'nether_portal.rs', '''    /// The exit portal of `plan` in the loaded dimension: the closest one
''', '''    /// True if the loaded dimension has a portal for `plan` to come out of.
    pub(super) fn exit_stands(&self, plan: &Plan, radius: Option<i32>) -> bool {
        let to_nether = plan.dim.kind() == Dim::Nether;
        let border = self.border_bounds();
        let portals = self.portal_blocks(plan.approximate, radius.unwrap_or(if to_nether { 16 } else { 128 }));
        let (low, high) = if to_nether { (0, 15) } else { (-4, 19) };
        closest_portal(&portals, plan.approximate, to_nether, &border, low, high).is_some()
    }
    /// The exit portal of `plan` in the loaded dimension: the closest one
'''),
    (GAME + 'server_travel.rs', '''            log::error!("could not open {}: {e}", target.id());
            return;
        }
        let destination = match plan {
            Some(plan) => self.arrival_centre(&plan),
''', '''            log::error!("could not open {}: {e}", target.id());
            return;
        }
        if plan.as_ref().is_some_and(|plan| {
            let at = [plan.approximate.x, plan.approximate.y, plan.approximate.z];
            @F::no_exit(self, at, |game, radius| game.exit_stands(plan, radius))
        }) {
            if let Err(e) = self.enter_level(Some(host), source) {
                log::error!("could not reopen {}: {e}", source.id());
            }
            return;
        }
        let destination = match plan {
            Some(plan) => self.arrival_centre(&plan),
'''),
    (GAME + 'server_travel.rs', '''                match self.resolve_gateway(job, trip.target) {
                    Ok(at) => (at, yaw),
''', '''                match self.resolve_gateway(job, trip.target) {
                    Ok(at) => match @F::gateway_arrived(self, host, trip.conn, at) {
                        Some(at) => (at, yaw),
                        None => continue,
                    },
'''),
    # PlayerTeleportEndGatewayEvent, once the way is worked out.
    (GAME + 'server_travel.rs', '''        let (job, destination) = self.gateway_job(gateway);
        let at = [destination.x, destination.y, destination.z];
        host.hub.teleport_keeping_motion(i, at);
        if let GatewayJob::Known { exact: true, .. } = job {
            return;
        }
''', '''        let (job, destination) = self.gateway_job(gateway);
        if let GatewayJob::Known { exact: true, .. } = job {
            if let Some((destination, _)) = @F::gateway_asked(self, host, conn, gateway, destination) {
                host.hub.teleport_keeping_motion(i, [destination.x, destination.y, destination.z]);
            }
            return;
        }
        if self.server_trips.iter().any(|t| t.is(conn)) {
            return;
        }
        // The exit is exact once the chunks there are in: they stay where they are till then.
        @F::gateway_pending(conn, gateway);
        let area = ((destination.x.floor() as i32) >> 4, (destination.z.floor() as i32) >> 4);
        if !self.end.areas.contains(&area) {
            self.end.areas.push(area);
            self.streaming.extra.push((area, 1));
        }
'''),
    # PlayerDropItemEvent: every drop of a player's goes through here.
    ('horizon-client/src/game.rs', '''            if let Some(item) = self.just_spawned_item() {
                item.thrower = Some(uuid);
            }
        }
    }
    /// The UUID of the player acting: the guest whose turn it is, else the host.
''', '''            if let Some(item) = self.just_spawned_item() {
                item.thrower = Some(uuid);
            }
        }
        @F::dropped(self, thrown_from_hand);
    }
    /// The UUID of the player acting: the guest whose turn it is, else the host.
'''),
    # Chat: the hub hands a checked line to ChatProcessor's thread, and sends what comes back.
    (LAN + 'hub/chat.rs', '''        let line = format!("<{name}> {}", message.content);
''', '''        let line = format!("<{name}> {}", message.content);
        if crate::paper_hub::chat_listened() {
            let online: Vec<serde_json::Value> = self.guests.iter().filter(|g| g.placed).map(|g| serde_json::json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name })).collect();
            let payload = serde_json::json!({
                "uuid": format!("{uuid:032x}"),
                "name": name,
                "level": self.guests[i].dimension,
                "message": message.content,
                "timestamp": message.timestamp,
                "salt": message.salt,
                "signature": message.signature.map(|signature| signature.to_vec()),
                "online": online,
            });
            crate::paper_hub::chat_heard(uuid, payload, Box::new((message, bound, line)));
            return;
        }
'''),
    (LAN + 'hub/chat.rs', '''    /// handleChatSessionUpdate: a new profile key, validated against the
''', '''    /// `Player.chat`: a line of no packet, unsigned, for its events. `None` if they aren't here.
    pub fn chat_compelled(&self, uuid: u128, text: &str) -> Option<(serde_json::Value, Box<dyn std::any::Any + Send>)> {
        let i = self.guests.iter().position(|g| g.uuid == uuid)?;
        let name = self.guests[i].name.clone();
        let message = ChatMessage {
            sender: uuid,
            index: 0,
            signature: None,
            content: text.to_owned(),
            timestamp: now_millis(),
            salt: 0,
            last_seen: Vec::new(),
            unsigned_content: None,
        };
        let bound = play::ChatTypeBound {
            chat_type: self.chat_type,
            name: self.player_display_name(uuid, &name),
            target: None,
        };
        let online: Vec<serde_json::Value> = self.guests.iter().filter(|g| g.placed).map(|g| serde_json::json!({ "uuid": format!("{:032x}", g.uuid), "name": g.name })).collect();
        let payload = serde_json::json!({ "uuid": format!("{uuid:032x}"), "name": name, "level": self.guests[i].dimension, "message": text, "timestamp": message.timestamp, "salt": 0, "signature": null, "online": online });
        let line = format!("<{name}> {text}");
        Some((payload, Box::new((message, bound, line))))
    }
    /// `ChatProcessor.complete`: what the listeners made of this guest's lines goes out here.
    pub(super) fn chat_answered(&mut self, i: usize, events: &mut Vec<HostEvent>) {
        let (uuid, name) = (self.guests[i].uuid, self.guests[i].name.clone());
        for (said, held) in crate::paper_hub::chat_answers(uuid) {
            let Ok(held) = held.downcast::<(ChatMessage, play::ChatTypeBound, String)>() else {
                continue;
            };
            let (mut message, mut bound, mut line) = *held;
            if let Some(shown) = said["name"].as_str().filter(|shown| *shown != name) {
                bound.name = horizon_protocol::text::literal(shown);
            }
            if said["chat_cancelled"] == true
                || !crate::paper_hub::allow_chat(uuid, &name, &message.content)
            {
                continue;
            }
            let text = message.content.clone();
            if let Some(shown) = said["chat_line"].as_str() {
                line = shown.to_owned();
            }
            if !said["chat_unsigned"].is_null() {
                message.unsigned_content = Some(crate::paper_hub::chat_tag(&said["chat_unsigned"]));
            }
            let lines = said["chat_lines"].as_object();
            let viewers: Option<Vec<&str>> = said["chat_recipients"].as_array().map(|all| all.iter().filter_map(serde_json::Value::as_str).collect());
            let secure = message.trusted(now_millis());
            if said["chat_console"] != false {
                events.push(HostEvent::Chat { line, secure });
            }
            for k in 0..self.guests.len() {
                let who = format!("{:032x}", self.guests[k].uuid);
                if !self.guests[k].placed || viewers.as_ref().is_some_and(|all| !all.contains(&who.as_str())) {
                    continue;
                }
                // The game has no chat type that shows a line as it stands: it would put the name in front
                // again. A line a plugin formatted goes out as the server's own, which is what Paper's is.
                if let Some(shown) = lines.and_then(|lines| lines.get(&who).or(lines.get("*"))) {
                    self.guests[k].send(&C::SystemChat(play::SystemChat { content: crate::paper_hub::chat_tag(shown), overlay: false }));
                    continue;
                }
                self.send_chat(k, &message, &bound);
            }
            events.push(HostEvent::PlayerChat { sender: uuid, text });
        }
    }
    /// handleChatSessionUpdate: a new profile key, validated against the
'''),
    (LAN + 'hub.rs', '''        let mut budget = PACKETS_PER_TICK;
        while budget > 0 {
''', '''        self.chat_answered(i, events);
        let mut budget = PACKETS_PER_TICK;
        while budget > 0 {
'''),
    # A chat thread's questions go to the game's turn.
    ('horizon-client/src/game.rs', '''        if let Some(answer) = super::paper_fam_ev_world::free_query(request) {
            return answer;
        }
''', '''        if let Some(answer) = super::paper_fam_ev_world::free_query(request) {
            return answer;
        }
        if !request.contains(r#""op":"event_patch""#)
            && let Some(answer) = horizon_client::paper_hub::chat_ask(request)
        {
            return answer;
        }
'''),
    # AsyncPlayerSpawnLocationEvent, while configuration holds; its place goes on the guest as it's loaded.
    (LAN + 'conn.rs', '''    if reconfiguring {
        hold_configuration(c, uuid, &mut info)?;
''', '''    crate::paper_hub::spawn_event(uuid);
    if reconfiguring {
        hold_configuration(c, uuid, &mut info)?;
'''),
    (GAME + 'lan.rs', '''        let back = self.dim;
        let level = super::Dim::from_id(&g.dimension);
        let elsewhere = self.options.dedicated.is_some() && level != back;
''', '''        @F::spawn_chosen(g);
        let back = self.dim;
        let level = super::Dim::from_id(&g.dimension);
        let elsewhere = self.options.dedicated.is_some() && level != back;
'''),
    (LAN + 'playerdata.rs', '''pub fn load(guest: &mut Guest, world: &Path) -> bool {
''', '''/// Where a saved player stands, with no guest to load them into.
pub fn saved_place(world: &Path, uuid: u128) -> Option<(String, [f64; 3], f32, f32)> {
    let file = path(world, uuid);
    let tag = read_dat(&file).or_else(|| read_dat(&file.with_extension("dat_old")))?;
    let mut level = Level::default();
    level.read_player(&tag);
    Some((level.dimension.clone(), level.position?, level.rotation[0], level.rotation[1]))
}
pub fn load(guest: &mut Guest, world: &Path) -> bool {
'''),
    # PlayerRespawnEvent's location, and the look at a level's saved chunks for it.
    (GAME + 'lan_actions.rs', '''        let back = self.dim;
        let away = server && host.hub.guests[i].dimension != level.id();
''', '''        let chosen = @F::respawn_chosen(&mut host.hub.guests[i]);
        let picked = chosen.is_some();
        let (elsewhere, level, spawn) = match chosen {
            Some((dim, at)) => (None, dim, at),
            None => (elsewhere, level, spawn),
        };
        let back = self.dim;
        let away = server && host.hub.guests[i].dimension != level.id();
'''),
    (GAME + 'lan_actions.rs', '''        } else if in_frame_bed.is_none() && host.hub.guests[i].spawn.is_none() {
            self.world_spawn_trip(conn);
        }
''', '''        } else if !picked && in_frame_bed.is_none() && host.hub.guests[i].spawn.is_none() {
            self.world_spawn_trip(conn);
        }
'''),
    (GAME + 'nether_portal.rs', '''    /// True if the loaded dimension has a portal for `plan` to come out of.
''', '''    /// Runs `look` with the saved chunks around `at` in the world, and takes them out after.
    pub(super) fn with_saved_chunks<R>(&mut self, at: [i32; 3], look: impl FnOnce(&mut Self) -> R) -> R {
        let (cx, cz) = (at[0] >> 4, at[2] >> 4);
        let mut added = Vec::new();
        for x in cx - 1..=cx + 1 {
            for z in cz - 1..=cz + 1 {
                if self.world.chunks.contains_key(&(x, z)) {
                    continue;
                }
                if let Some(chunk) = self.save.load(x, z, self.pack.assets.registry, world_biomes!(self)) {
                    self.world.chunks.insert((x, z), chunk.into());
                    added.push((x, z));
                }
            }
        }
        let result = look(self);
        for chunk in added {
            self.world.chunks.remove(&chunk);
        }
        result
    }
    /// True if the loaded dimension has a portal for `plan` to come out of.
'''),
    # PHYSICAL, before the change.
    (GAME + 'lan_mobs.rs', '''            .map(|g| {
                let b = g.bounds();
                let (lo, hi) = (b.min_position(), b.max_position());
                ([lo.x, lo.y, lo.z], [hi.x, hi.y, hi.z])
            })
''', '''            .filter(|g| !@F::step_uncounted(g.uuid))
            .map(|g| {
                let b = g.bounds();
                let (lo, hi) = (b.min_position(), b.max_position());
                ([lo.x, lo.y, lo.z], [hi.x, hi.y, hi.z])
            })
'''),
    ('horizon-world/src/block_landing.rs', '''    fn under_spawn_protection(&mut self, position: BlockPos) -> Result<bool, Self::Error>;
''', '''    fn under_spawn_protection(&mut self, position: BlockPos) -> Result<bool, Self::Error>;
    /// True if a player's step that would turn this block into `next` waits to be asked about.
    fn step_held(&mut self, _position: BlockPos, _next: Option<BlockStateId>) -> Result<bool, Self::Error> {
        Ok(false)
    }
'''),
    ('horizon-world/src/block_landing.rs', '''                        if width * width * height > 0.512f32 {
                            let next = push_entities_up(world, state, base, position)
''', '''                        if width * width * height > 0.512f32 {
                            if entity == LandingEntityKind::Player && world.step_held(position, Some(base)).map_err(LandingError::World)? {
                                return Ok(());
                            }
                            let next = push_entities_up(world, state, base, position)
'''),
    ('horizon-world/src/block_landing.rs', '''        if allowed && world.random_int(randomness).map_err(LandingError::World)? == 0 {
            let sound = world.known_sound(LandingSound::TurtleEggBreak);
''', '''        if allowed && world.random_int(randomness).map_err(LandingError::World)? == 0 {
            if entity == LandingEntityKind::Player && world.step_held(position, remaining).map_err(LandingError::World)? {
                return Ok(());
            }
            let sound = world.known_sound(LandingSound::TurtleEggBreak);
'''),
    ('horizon-client/src/landing.rs', '''    fn under_spawn_protection(&mut self, _: BlockPos) -> Result<bool> {
        Ok(false)
    }
''', '''    fn under_spawn_protection(&mut self, _: BlockPos) -> Result<bool> {
        Ok(false)
    }
    fn step_held(&mut self, position: BlockPos, next: Option<BlockStateId>) -> Result<bool> {
        Ok(crate::paper_hub::step_held([position.x, position.y, position.z], next))
    }
'''),
    (GAME + 'sculk.rs', '''            let _ = self.attack_block(pos);
        }
    }
    /// `RedStoneOreBlock.useItemOn`: a right-click lights the ore whatever the
''', '''            if burst && @F::ore_refused(self, pos) {
                return;
            }
            let _ = self.attack_block(pos);
        }
    }
    /// `RedStoneOreBlock.useItemOn`: a right-click lights the ore whatever the
'''),
    # A teleport to another level is heard too.
    (GAME + 'server_travel.rs', '''            let Some(i) = host.hub.guests.iter().position(|g| g.conn == t.conn) else {
                continue;
            };
            let source = self.dim;
''', '''            let Some(i) = host.hub.guests.iter().position(|g| g.conn == t.conn) else {
                continue;
            };
            let Some(t) = @F::level_tp_gate(self, host, t) else {
                continue;
            };
            let source = self.dim;
'''),
]
EDITS = [(name, old, new.replace('@F', FAM)) for name, old, new in EDITS]
