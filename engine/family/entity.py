# Family entity. Yours alone. Loaded by event_hooks.py, sdk_events.py and sdk_watch.py.
# Names available here: B P D PLAYER (sdk_events) and STACK GUEST KEY POS STACK_PARAM HOST UNIT (event_hooks).
HERE = 'crate::game::paper_fam_entity'

HOOKS = [    # rows as in fabric-port/tools/event_hooks.py ROWS
    ('entity_after_mobs', 'root::tick_experience', [], f'"done": {HERE}::after_mobs(game)', None),
    ('entity_tnt', 'tnt::spawn_primed_tnt', [('pos', 'BlockPos')], f'"done": {HERE}::tnt_primed(game, pos)', None),
    ('entity_fangs', 'fangs::add_fangs', [('at', '[f64; 3]'), ('yaw', 'f32'), ('delay', 'i32'), ('owner', 'u32')], f'"done": {HERE}::fangs_placed(game, at, delay, owner)', None),
    ('entity_portal_gate', 'nether_portal::send_through_portal', [('id', 'u32'), ('entry', 'BlockPos')], '"entity": id', UNIT),
]
EVENTS = []   # rows as in plugin-tools/tools/sdk_events.py EVENTS
WATCH = []    # rows as in plugin-tools/tools/sdk_watch.py ROWS
