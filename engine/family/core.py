# Family core. Yours alone. Loaded by event_hooks.py, sdk_events.py and sdk_watch.py.
# Names available here: B P D PLAYER (sdk_events) and STACK GUEST KEY POS STACK_PARAM HOST UNIT (event_hooks).
# Rows as in fabric-port/tools/event_hooks.py ROWS. None of these is an event a plugin hears: the payload
# is the work, in the module of the function hooked, which is what lets it touch that module's private
# parts. A row runs only if something listens, which `fam_core::install` does.
DRAGON = '''({
    if game.dim == Dim::End {
        game.load_fight();
        for order in crate::game::paper_fam_core::take_dragon_orders() {
            match order.as_str() {
                "respawn" => game.try_respawn(),
                "reset" => game.reset_spike_crystals(),
                "gateway" => {
                    if let Some(g) = game.fight.gateways.pop() {
                        let a = 2.0 * (-std::f64::consts::PI + std::f64::consts::PI / 20.0 * f64::from(g));
                        let pos = [(96.0 * a.cos()).floor() as i32, 75, (96.0 * a.sin()).floor() as i32];
                        game.level_event_at(3000, pos, 0);
                        let _ = game.place_gateway_at(pos);
                        game.fight.dirty = true;
                    }
                }
                "end" => {
                    if game.fight.respawn.is_some() {
                        game.finish_respawn();
                    }
                }
                "none" => {
                    if game.fight.respawn.is_some() {
                        game.abort_respawn();
                    }
                }
                phase => {
                    if game.fight.respawn.is_some() && let Some(stage) = Respawn::from_name(phase) {
                        game.set_respawn(Some(stage));
                    }
                }
            }
        }
        let crystals = game.fight.respawn_crystals.iter().map(|at| [at.x, at.y, at.z]).collect();
        crate::game::paper_fam_core::note_dragon(game.fight.gateways.len(), crystals);
    }
    0
})'''
PORTAL = '''({
    if game.dim == Dim::End && let Some(active) = crate::game::paper_fam_core::take_portal_order() {
        let origin = game.exit_portal().unwrap_or([0, game.podium_level().unwrap_or(61), 0]);
        let _ = game.place_podium(origin, active);
    }
    0
})'''
RAIDS = '''({
    for (id, waves) in crate::game::paper_fam_core::take_raid_waves() {
        if let Some(raid) = game.raiding.raids.map.get_mut(&id) {
            raid.num_groups = waves;
            game.raiding.raids.dirty = true;
        }
    }
    0
})'''
MAPS = '''({
    for id in crate::game::paper_fam_core::take_touched_maps() {
        if let Some(map) = game.map(id) {
            map.dirty = true;
            map.texture_dirty = true;
            for holder in &mut map.holders {
                holder.dirty = Some([0, 0, 127, 127]);
            }
        }
    }
    0
})'''
HOOKS = [
    ('core_void_mobs', 'root::tick_mobs', [], '"done": crate::game::paper_fam_core::void_rule_here(game.dim.id())', None),
    ('core_void_guest', 'lan_actions::guest_server_tick', [('host', '&LanHost', '&*host'), ('conn', 'u32')], '"done": crate::game::paper_fam_core::void_rule_here(host.hub.guest(conn).map_or("minecraft:overworld", |guest| guest.dimension.as_str()))', None),
    ('core_version_line', 'lan_actions::guest_command', [('host', '&mut LanHost', '&mut *host'), ('conn', 'u32'), ('command', '&str')], '"skip": crate::game::paper_fam_core::version_line(host, conn, command)', UNIT),
    ('core_map_tick', 'maps::tick_maps', [], f'"done": {MAPS}', None),
    ('core_tick_server', 'levels::tick_server', [('paused', 'bool')], '"skip": crate::game::paper_fam_core::server_pass(game, paused)', ('Result<()>', 'Ok(())')),
    ('core_idle', 'dedicated::kick_idle', [('minutes', 'i32')], '"skip": crate::game::paper_fam_core::idle_overridden(), "minutes": minutes', UNIT),
    ('core_spawn_pass', 'natural_spawning::spawn_natural', [], '"done": crate::game::paper_fam_core::spawn_pass(game)', None),
    ('core_raid_tick', 'raids::tick_raids', [], f'"done": {RAIDS}', None),
    ('core_dragon_tick', 'dragon::tick_fight', [], f'"done": {DRAGON}', None),
    ('core_end_tick', 'end::tick_end', [], f'"done": {PORTAL}', None),
    ('core_tick_command', 'commands::misc::cmd_tick', [('id', '&str'), ('ctx', '&Context')], '"done": crate::game::paper_fam_core::note_step(id, match ctx.get("time") { Some(Arg::Int(v)) => *v, _ => 1 })', None),
    ('core_dispense', 'dispenser::dispense_item', [('pos', 'Pos')], '"done": crate::game::paper_fam_core::note_dispense(game, [pos.0, pos.1, pos.2])', None),
]
EXTRA = [
    # What a client says it listens on, for `getListeningPluginChannels`: read as it passes, nothing held.
    {'id': 'core_channels', 'target': "horizon_client::game::lan_actions::Game<'_>::lan_guest_packet", 'kind': 'HEAD',
     'params': [('host', '&LanHost', '&*host'), ('conn', 'u32', 'conn'), ('packet', '&S', '&packet')],
     'code': "fn paper_hook_core_channels(host: &LanHost, conn: u32, packet: &S) {\n    crate::game::paper_fam_core::note_channels(host, conn, packet);\n}\n"},
    # A real `/reload` reads the game's recipes again; the SDK's own leave a clear alone.
    {'id': 'core_reload', 'target': "horizon_client::game::functions::Game<'_>::cmd_reload", 'kind': 'HEAD', 'params': [],
     'code': "fn paper_hook_core_reload() {\n    crate::game::paper_fam_core::real_reload();\n}\n"},
    # Whichever word a plugin owns, `/execute ... run` can't reach it through the game's own tree.
    {'id': 'core_proxied', 'target': "horizon_client::game::commands::Game<'_>::run_command", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('command', '&str', 'command')],
     'code': "fn paper_hook_core_proxied(game: &mut crate::game::Game<'_>, command: &str) -> ::core::ops::ControlFlow<bool> {\n    match crate::game::paper_fam_core::proxied_line(game, command) {\n        Some(ran) => ::core::ops::ControlFlow::Break(ran),\n        None => ::core::ops::ControlFlow::Continue(()),\n    }\n}\n"},
    # A paused (empty) server runs only its connections, but a login waits on answers only a tick
    # gives (ServerPlayConnectionEvents.INIT, its spawn, the listeners): paused, INIT ran out its
    # 5 s timeout every join. Run the pass as a real tick while a login waits.
    {'id': 'core_login_wake', 'target': "horizon_client::game::levels::Game<'_>::tick_server", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('paused', 'bool', 'paused')],
     'code': "fn paper_hook_core_login_wake(game: &mut crate::game::Game<'_>, paused: bool) -> ::core::ops::ControlFlow<Result<()>> {\n    if paused && horizon_client::paper_hub::logins_waiting() {\n        return ::core::ops::ControlFlow::Break(game.tick_server(false));\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n"},
    # Mojang's session service answers 503 to a share of its calls at random (11-03: from this PC and from
    # the VPS alike): the server asked once and refused the player ("Authentication servers are down").
    # Asked again a few times within about 5 s first, on the connection's own thread.
    {'id': 'core_verify_retry', 'target': "horizon_client_render::auth::has_joined", 'kind': 'HEAD', 'cancellable': True,
     'params': [('name', '&str', 'name'), ('server_hash', '&str', 'server_hash'), ('ip', 'Option<std::net::IpAddr>', 'ip')],
     'code': "fn paper_hook_core_verify_retry(name: &str, server_hash: &str, ip: Option<std::net::IpAddr>) -> ::core::ops::ControlFlow<Result<Option<JoinedProfile>, JoinError>> {\n    thread_local! {\n        static INSIDE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };\n    }\n    if INSIDE.with(std::cell::Cell::get) {\n        return ::core::ops::ControlFlow::Continue(());\n    }\n    INSIDE.with(|inside| inside.set(true));\n    let mut result = has_joined(name, server_hash, ip);\n    for pause in [250u64, 250, 500, 500, 750, 1000, 1000, 1500, 1500, 2000, 2000, 2000, 2000, 2000] {\n        if !matches!(result, Err(JoinError::Unavailable)) {\n            break;\n        }\n        log::warn!(\"Mojang's session service didn't answer {name}'s check; asking again\");\n        std::thread::sleep(std::time::Duration::from_millis(pause));\n        result = has_joined(name, server_hash, ip);\n    }\n    INSIDE.with(|inside| inside.set(false));\n    ::core::ops::ControlFlow::Break(result)\n}\n"},]
EVENTS = []   # rows as in plugin-tools/tools/sdk_events.py EVENTS
WATCH = []    # rows as in plugin-tools/tools/sdk_watch.py ROWS
