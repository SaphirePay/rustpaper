# The connection's part of the plugin layer: plugin channels, chat that plugins decorate, and reconfiguration.
CLIENT = 'horizon-client/src/'
GAME = CLIENT + 'game/'

EDITS = [
    # ---- Play channels in the game's own form: `minecraft:register`, and a mod's channel as itself
    # for a client that isn't Horizon, as `horizon:mod_net` is only Horizon's.
    (GAME + 'lan_actions.rs', '''            S::CustomPayload(p) if p.channel == horizon_protocol::mod_net::CHANNEL => {
                self.guest_native_net(host, conn, &p.data)
            }
''', '''            S::CustomPayload(p) if p.channel == horizon_protocol::mod_net::CHANNEL => {
                self.guest_native_net(host, conn, &p.data)
            }
            S::CustomPayload(p) if !p.channel.starts_with("horizon:") && p.channel != "minecraft:brand" => {
                self.guest_raw_payload(host, conn, p.channel, p.data)
            }
'''),
    (GAME + 'native_net.rs', '''    /// Whether messages from players wait for `run_native_net`.
''', '''    /// A channel of a client that isn't Horizon's, or `minecraft:register`: a message like any other.
    pub(super) fn guest_raw_payload(&mut self, host: &LanHost, conn: u32, channel: String, data: Vec<u8>) {
        let Some(player) = host.hub.guest(conn).map(|g| g.uuid) else {
            return;
        };
        let count = self.native_net.from_counts.entry(player).or_default();
        if *count >= PER_TICK {
            return;
        }
        *count += 1;
        self.native_net.from_players.push((player, Message { channel, data }));
    }

    /// Whether messages from players wait for `run_native_net`.
'''),
    (GAME + 'native_net.rs', '''            if let Some(g) = host
                .hub
                .guests
                .iter()
                .find(|g| g.placed && g.uuid == player && g.brand == "horizon")
            {
                host.hub.send_to(
                    g.conn,
                    &C::CustomPayload(CustomPayload {
                        channel: mod_net::CHANNEL.into(),
                        data,
                    }),
                );
            }
''', '''            if let Some(g) = host.hub.guests.iter().find(|g| g.placed && g.uuid == player) {
                let packet = if g.brand == "horizon" {
                    Some(CustomPayload { channel: mod_net::CHANNEL.into(), data })
                } else {
                    mod_net::decode(&data).ok().map(|m| CustomPayload { channel: m.channel, data: m.data })
                };
                if let Some(packet) = packet {
                    host.hub.send_to(g.conn, &C::CustomPayload(packet));
                }
            }
'''),
    (GAME + 'native_net.rs', '''        if !remote.mod_net {
            return Err("the server carries no mod messages".into());
        }
        let payload = payload(channel, data)?;
''', '''        if !remote.mod_net {
            self.send(S::CustomPayload(CustomPayload { channel: channel.to_owned(), data: data.to_vec() }));
            return Ok(());
        }
        let payload = payload(channel, data)?;
'''),
    # ---- `ServerMessageEvents`: game messages carry the component and the overlay flag; a line for
    # a team isn't one, as `broadcastSystemToTeam` doesn't go through `broadcastSystemMessage`.
    (CLIENT + 'lan/hub.rs', '''                if crate::paper_hub::hold_line(&hub.english(content), content) {
                    return ::core::ops::ControlFlow::Break(());
                }
                if crate::paper_hub::allow_game(&hub.english(content)) {
''', '''                if crate::paper_hub::hold_line(&hub.english(content), content) {
                    return ::core::ops::ControlFlow::Break(());
                }
                if crate::paper_hub::allow_game(&hub.english(content), content, false) {
'''),
    (CLIENT + 'lan/hub.rs', '''                    return ::core::ops::ControlFlow::Break(());
                }
                if crate::paper_hub::allow_game(&hub.english(content)) {
                    ::core::ops::ControlFlow::Continue(())
                } else {
                    ::core::ops::ControlFlow::Break(())
                }
            }
            paper_hook_game_message_to(&*self, &content)
''', '''                    return ::core::ops::ControlFlow::Break(());
                }
                ::core::ops::ControlFlow::Continue(())
            }
            paper_hook_game_message_to(&*self, &content)
'''),
    (CLIENT + 'lan/hub.rs', '''                    ) || !crate::paper_hub::allow_game(&hub.english(&chat.content)))
''', '''                    ) || !crate::paper_hub::allow_game(&hub.english(&chat.content), &chat.content, chat.overlay))
'''),
    # ---- `/say` and `/me` from anywhere reach everyone, decorated, as `MessageArgument` has it.
    (CLIENT + 'lan/hub.rs', '''    pub fn say(&self, source: &str, text: &str) {
        let chat_type = self
            .sync
            .id("minecraft:chat_type", "minecraft:say_command")
            .unwrap_or(self.chat_type);
''', '''    pub fn say(&self, source: &str, text: &str) {
        self.command_line("say_command", source, text);
    }
    /// A message command's line from what isn't a player's own typing: `kind` is its chat type.
    pub fn command_line(&self, kind: &str, source: &str, text: &str) {
        let chat_type = self
            .sync
            .id("minecraft:chat_type", &format!("minecraft:{kind}"))
            .unwrap_or(self.chat_type);
        let message = crate::paper_hub::take_decorated().unwrap_or_else(|| literal(text));
'''),
    (CLIENT + 'lan/hub.rs', '''            &C::DisguisedChat(play::DisguisedChat {
                message: literal(text),
                chat_type: play::ChatTypeBound {
                    chat_type,
                    name: literal(source),
''', '''            &C::DisguisedChat(play::DisguisedChat {
                message,
                chat_type: play::ChatTypeBound {
                    chat_type,
                    name: literal(source),
'''),
    (CLIENT + 'lan/hub/chat.rs', '''        match message {
            Some(message) => self.broadcast_chat(&message, &bound, line, events),
''', '''        let decorated = crate::paper_hub::take_decorated();
        match message {
            Some(mut message) => {
                if decorated.is_some() {
                    message.unsigned_content = decorated;
                }
                self.broadcast_chat(&message, &bound, line, events)
            }
'''),
    (CLIENT + 'lan/hub/chat.rs', '''                        self.send_disguised(k, literal(text), bound.clone());
''', '''                        self.send_disguised(k, decorated.clone().unwrap_or_else(|| literal(text)), bound.clone());
'''),
    (CLIENT + 'lan/hub/chat.rs', '''        let (outgoing, incoming) = (
            self.chat_type_id("msg_command_outgoing"),
''', '''        let decorated = crate::paper_hub::decorate(uuid, text);
        let message = message.map(|mut message| {
            if decorated.is_some() {
                message.unsigned_content = decorated.clone();
            }
            message
        });
        let (outgoing, incoming) = (
            self.chat_type_id("msg_command_outgoing"),
'''),
    (CLIENT + 'lan/hub/chat.rs', '''                    self.send_disguised(i, literal(text), out);
                    self.send_disguised(t, literal(text), inc);
''', '''                    let shown = decorated.clone().unwrap_or_else(|| literal(text));
                    self.send_disguised(i, shown.clone(), out);
                    self.send_disguised(t, shown, inc);
'''),
    (GAME + 'commands.rs', '''    fn cmd_say(&mut self, id: &str, ctx: &Context) -> R<i32> {
''', '''    /// Who a message command runs as: `ChatMessage`'s sender, if a player, and the name shown.
    pub(in crate::game) fn message_source(&self) -> (Option<u128>, String) {
        match self.command_source.map(|s| s.entity) {
            Some(Some(Target::Guest(conn))) => {
                let guest = self.lan.as_ref().and_then(|host| host.hub.guest(conn));
                (guest.map(|g| g.uuid), guest.map_or_else(String::new, |g| g.name.clone()))
            }
            Some(Some(Target::Player)) | None => (self.lan_acting_uuid, self.player_display_name().plain()),
            Some(Some(target)) => (None, self.target_name(target).plain()),
            Some(None) => (None, "Server".to_owned()),
        }
    }
    fn cmd_say(&mut self, id: &str, ctx: &Context) -> R<i32> {
'''),
    (GAME + 'commands.rs', '''        let line = match id {
            "say" => Text::translate("chat.type.announcement", &[name, message]),
''', '''        if id != "msg" && let Some(host) = &self.lan {
            host.hub.command_line(if id == "say" { "say_command" } else { "emote_command" }, &name.plain(), &message.plain());
        }
        let line = match id {
            "say" => Text::translate("chat.type.announcement", &[name, message]),
'''),
    (GAME + 'dedicated.rs', '''                let message = rest.join(" ");
                log("INFO", &format!("[Not Secure] [Server] {message}"));
                if let Some(h) = &self.lan {
                    h.hub.say("Server", &message);
                }
''', '''                let message = rest.join(" ");
                if horizon_client::paper_hub::allow_command("say_command", &message, None, "Server") {
                    log("INFO", &format!("[Not Secure] [Server] {message}"));
                    if let Some(h) = &self.lan {
                        h.hub.say("Server", &message);
                    }
                }
'''),
    # ---- A player back to configuration keeps its connection, so play's DISCONNECT isn't theirs.
    (CLIENT + 'lan/conn.rs', '''                let _ = events.send(NetEvent::Left(conn_id, "Reconfiguring".to_owned()));
''', '''                crate::paper_net::leaves_for_configuration(parts.uuid);
                let _ = events.send(NetEvent::Left(conn_id, "Reconfiguring".to_owned()));
'''),
    (GAME + 'locator_bar.rs', '''    fn own_uuid(&self) -> u128 {
''', '''    pub(super) fn own_uuid(&self) -> u128 {
'''),
    # ---- A reconfiguration asked for where the hub couldn't be reached goes out at the next tick.
    (GAME + 'lan.rs', '''        self.register_native_commands(&mut host);
        if !self.level_teleports.is_empty() {
''', '''        self.register_native_commands(&mut host);
        crate::game::paper_fam_ev_player::send_switches(&host);
        if !self.level_teleports.is_empty() {
'''),
    # ---- `/teammsg` is a `MessageArgument` too: what a decorator made of it is what the team hears.
    (GAME + 'commands/misc.rs', '''        let message = self.expand_selectors(&raw);
        let hover = text::Style {
            hover: Some(Box::new(Text::translate("chat.type.team.hover", &[]))),
''', '''        let message = self.expand_selectors(&raw);
        let message = horizon_client::paper_hub::decorate_as(self.lan_acting_uuid, &raw)
            .map_or(message, |line| crate::game::remote::remote_text(&line));
        let hover = text::Style {
            hover: Some(Box::new(Text::translate("chat.type.team.hover", &[]))),
'''),
]
