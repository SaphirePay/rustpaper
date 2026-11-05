# Family items. Yours alone. Loaded by event_hooks.py, sdk_events.py and sdk_watch.py.
# Names available here: B P D PLAYER (sdk_events) and STACK GUEST KEY POS STACK_PARAM HOST UNIT (event_hooks).
#
# A workstation's `Menu` is private to its `lan_*` module, and a hook's helper lives in the module it
# hooks, so these rows lend it to paper_fam_items as the game sends it. Each runs only if heard:
# `fam_items::install` listens to them all, for every plugin. Their payload does the work: the events
# themselves carry nothing a listener reads.
HOST_MUT = ('host', '&mut LanHost', '&mut *host')
HOST_REF = ('host', '&LanHost', '&*host')
CONN = ('conn', 'u32')
WINDOW = ('window', 'Option<i32>')
WHO = 'let who = host.hub.guest(conn).map(|g| g.uuid);'
CLOSED = '"uuid": crate::game::paper_bridge::uuid_of(host, conn), "closed": (crate::game::paper_fam_items::menu_closed(host, conn, window))'
TICK = '"uuid": "", "sent": (for conn in crate::game::paper_fam_items::pending_conns(&*host) {{ game.{send}(host, conn); }})'
REFUSED = '"uuid": "", "refused": (crate::game::paper_fam_items::{call})'

HOOKS = [
    ('items_station', 'lan_stations::lan_send_station', [HOST_MUT, CONN],
     '"uuid": "", "open": ({ ' + WHO + ' let hub = &host.hub; match (who, host.play.get_mut(&conn).and_then(|p| p.station.as_mut())) { (Some(who), Some(gs)) => crate::game::paper_fam_items::menu_sent(game, hub, who, gs.window, gs.pos, &mut gs.menu), _ => serde_json::Value::Null } })', None),
    ('items_stations_tick', 'lan_stations::lan_stations_tick', [HOST_MUT], TICK.format(send='lan_send_station') + ', "upkeep": (crate::game::paper_fam_items::tick(game, host))', None),
    ('items_station_closed', 'lan_stations::lan_station_closed', [HOST_REF, CONN, WINDOW], CLOSED, None),
    ('items_lectern', 'lan_stations::lan_send_lectern', [HOST_MUT, CONN],
     '"uuid": "", "open": ({ ' + WHO + ' match (who, host.play.get(&conn).and_then(|p| p.lectern.as_ref())) { (Some(who), Some(gl)) => crate::game::paper_fam_items::lectern_sent(who, gl.window, gl.key), _ => serde_json::Value::Null } })', None),
    ('items_table', 'lan_crafting::lan_send_table_content', [HOST_MUT, CONN],
     '"uuid": "", "open": ({ ' + WHO + ' let hub = &host.hub; match (who, host.play.get_mut(&conn).and_then(|p| p.crafting.table.as_mut())) { (Some(who), Some(table)) => crate::game::paper_fam_items::menu_sent(game, hub, who, table.0, table.3, &mut table.2), _ => serde_json::Value::Null } })', None),
    ('items_tables_tick', 'lan_crafting::lan_tables_tick', [HOST_MUT], TICK.format(send='lan_send_table_content'), None),
    ('items_table_closed', 'lan_crafting::lan_crafting_closed', [HOST_REF, CONN, WINDOW], CLOSED, None),
    ('items_merchant', 'lan_trading::lan_send_merchant_content', [HOST_REF, CONN, ('gm', '&mut GuestMerchant', '&mut *gm')],
     '"uuid": "", "open": ({ ' + WHO + ' match who { Some(who) => crate::game::paper_fam_items::merchant_sent(game, who, gm.window, gm.merchant.mob, &gm.slots, gm.merchant.session.active, &mut gm.merchant.session.offers), None => serde_json::Value::Null } })', None),
    ('items_merchant_closed', 'lan_trading::lan_merchant_closed', [HOST_REF, CONN, WINDOW], CLOSED, None),
    # What a listener answers: the SDK's `install` refuses when `refused` is true.
    ('items_lock', 'lan_actions::lan_open_container', [HOST_REF, CONN, ('pos', '(i32, i32, i32)')], REFUSED.format(call='lock_refuses(game, host, conn, pos)'), ('bool', 'true')),
    ('items_lock_station', 'lan_stations::lan_open_station', [HOST_REF, CONN, ('station', '&Station')], REFUSED.format(call='lock_refuses(game, host, conn, (station.pos[0], station.pos[1], station.pos[2]))'), ('bool', 'true')),
    ('items_hive_enter', 'beehives::bee_enter_hive', [('mob', 'u32'), ('at', '[i32; 3]')], REFUSED.format(call='hive_enter(game, mob, at)'), UNIT),
    ('items_beacon', 'beacon::beacon_effects', [('p', '[i32; 3]'), ('b', '&Beacon')], REFUSED.format(call='beacon_range(game, p, b)'), UNIT),
    ('items_campfire', 'campfire::tick_campfire', [('pos', 'Pos')], '"uuid": "", "held": (crate::game::paper_fam_items::campfire_hold(game, pos))', None),
    ('items_sculk', 'sculk::tick_sculk_entities', [], '"uuid": "", "shrieks": (for key in crate::game::paper_fam_items::take_shrieks() { let _ = game.try_shriek(key, true); }), "blooms": (for key in crate::game::paper_fam_items::take_blooms() { game.catalyst_bloom(key); })', None),
]
# A sensor's own range: the block entity has no field for it, so one a sensor doesn't use carries it
# (`warning_level`, saved only for shriekers), set from the extras file by `paper_fam_items::tick`.
EXTRA = [
    # The clicks in plugins' menus are answered before the game's loop: a cancel moves nothing, anything else moves the stacks.
    {'id': 'items_gui_clicks', 'target': "horizon_client::game::native_menus::Game<'_>::run_native_menu_events", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self')],
     'code': "fn paper_hook_items_gui_clicks(game: &mut crate::game::Game<'_>) {\n    crate::game::paper_fam_items::gui_clicks(game);\n}\n"},
    {'id': 'items_sensor_range', 'target': 'horizon_client_render::sculk::SculkEntity::listener_radius', 'kind': 'HEAD', 'cancellable': True, 'params': [('entity', '&SculkEntity', '&*self')],
     'code': 'fn paper_hook_items_sensor_range(entity: &SculkEntity) -> ::core::ops::ControlFlow<i32> {\n    if entity.warning_level > 0 && matches!(entity.kind, SculkKind::Sensor | SculkKind::Calibrated) {\n        ::core::ops::ControlFlow::Break(entity.warning_level)\n    } else {\n        ::core::ops::ControlFlow::Continue(())\n    }\n}\n'},
]
EVENTS = []   # rows as in plugin-tools/tools/sdk_events.py EVENTS
WATCH = []    # rows as in plugin-tools/tools/sdk_watch.py ROWS
