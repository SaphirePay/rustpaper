"""The hooks behind Bukkit's events, one row each: gen_bundle.py makes the injection from these, and
paper_bridge.rs fires the row's id with its JSON.

A row is (id, target, params, payload, cancel). `target` is `module::fn` of the game's `Game`, or
`module::Type::fn`. Params are (name, type) or (name, type, source). `payload` is the inside of a
`json!({ })`. `cancel` is what the function returns when a listener refuses, as Rust text with its
type (`('()', '()')` or `('bool', 'false')`), or None for a hook that only tells.
"""
STACK = 'crate::game::paper_query::stack_json(stack)'
GUEST = '"uuid": crate::game::paper_bridge::uuid_of(host, conn)'
KEY = '"pos": [key.0, key.1, key.2]'
POS = '"pos": [pos.0, pos.1, pos.2]'
STACK_PARAM = ('stack', '&ItemStack')
HOST = [('host', '&LanHost', '&*host'), ('conn', 'u32')]
UNIT = ('()', '()')

ROWS = [
    ('boat_place', 'boats::place_boat', [STACK_PARAM], f'"stack": {STACK}', ('bool', 'false')),
    ('cart_place', 'minecarts::place_minecart', [STACK_PARAM], f'"stack": {STACK}', ('bool', 'false')),
    ('cart_hurt', 'minecarts::hurt_cart', [('i', 'usize'), ('damage', 'f32')], '"index": i, "damage": damage', UNIT),
    ('boat_hurt', 'boats::hurt_boat', [('i', 'usize'), ('damage', 'f32')], '"index": i, "damage": damage', ('bool', 'false')),
    ('cart_enter', 'minecarts::interact_cart', [], '', ('bool', 'false')),
    ('boat_enter', 'boats::interact_boat', [], '', ('bool', 'false')),
    ('cart_exit', 'minecarts::dismount_cart', [('i', 'usize')], '"index": i', UNIT),
    ('boat_exit', 'boats::dismount', [('i', 'usize')], '"index": i', UNIT),
    ('mount_exit', 'mounts::dismount_mount', [], '', UNIT),
    ('painting_place', 'paintings::place_painting', [STACK_PARAM], f'"stack": {STACK}', ('bool', 'false')),
    ('frame_place', 'frames::place_item_frame', [STACK_PARAM], f'"stack": {STACK}', ('bool', 'false')),
    ('painting_break', 'paintings::break_painting', [('index', 'usize'), ('by_player', 'bool')], '"index": index, "by_player": by_player', UNIT),
    ('frame_use', 'frames::interact_frame', [], '', ('bool', 'false')),
    ('frame_attack', 'frames::attack_frame', [], '', ('bool', 'false')),
    ('book_edit', 'books::lan_edit_book', [('host', '&LanHost', '&*host'), ('conn', 'u32'), ('p', '&play::EditBook', '&p')], f'{GUEST}, "pages": p.pages, "title": p.title, "slot": p.slot', UNIT),
    ('lectern_take', 'books::take_lectern_book', [('key', '(i32, i32, i32)')], KEY, UNIT),
    ('lectern_page', 'books::turn_page', [('forward', 'bool')], '"forward": forward', UNIT),
    ('crafter_craft', 'machines::crafter::crafter_eject', [('pos', 'Pos'), ('stack', '&ItemStack', '&stack'), ('recipe', '&str')], f'{POS}, "stack": {STACK}, "recipe": recipe', UNIT),
    ('dispense', 'dispenser::dispense_item', [('pos', 'Pos'), ('stack', '&ItemStack', '&stack')], f'{POS}, "stack": {STACK}', None),
    ('enchant_offers', 'lan_stations::lan_enchant_offers', [('host', '&LanHost', '&*host'), ('gi', 'usize')], '"uuid": format!("{:032x}", host.hub.guests[gi].uuid)', None),
    ('trade_select', 'lan_trading::lan_select_trade', [*HOST, ('index', 'i32')], f'{GUEST}, "index": index', UNIT),
    ('station_rename', 'lan_stations::lan_station_rename', [*HOST, ('name', '&str')], f'{GUEST}, "name": name', UNIT),
    ('station_button', 'lan_stations::lan_station_button', [*HOST, ('id', 'i32')], f'{GUEST}, "button": id', UNIT),
    ('leash_use', 'leads::interact_leash', [('index', 'usize')], '"index": index', ('bool', 'false')),
    ('leash_break', 'leads::break_leash', [('id', 'u32')], '"entity": id', UNIT),
    ('effect_add', 'potions::add_player_effect', [('effect', '&EffectInstance', '&effect')], '"effect": effect.id, "amplifier": effect.amplifier, "duration": effect.duration', ('bool', 'false')),
    ('rocket_explode', 'fireworks::explode_rocket', [('r', '&Rocket')], '"entity": r.id', UNIT),
    ('rocket_boost', 'fireworks::boost_firework', [STACK_PARAM], f'"stack": {STACK}', ('bool', 'false')),
    ('crystal_kill', 'dragon::kill_crystal', [('id', 'u32')], '"entity": id', UNIT),
    ('entity_portal', 'nether_portal::send_through_portal', [('id', 'u32'), ('entry', 'BlockPos')], '"entity": id, "pos": [entry.x, entry.y, entry.z]', UNIT),
    ('difficulty_set', 'commands::set_difficulty_forced', [('i', 'u8')], '"difficulty": i', UNIT),
    ('sculk_bloom', 'sculk::catalyst_bloom', [('key', '(i32, i32, i32)')], KEY, None),
    ('sensor', 'sculk::activate_sensor', [('key', '(i32, i32, i32)'), ('power', 'i32')], f'{KEY}, "power": power', None),
    ('totem', 'totem::apply_totem', [], '', None),
    ('name_mob', 'names::name_mob', [('index', 'usize')], '"index": index', ('bool', 'false')),
    ('trident_use', 'trident::use_trident', [STACK_PARAM], f'"stack": {STACK}', ('bool', 'false')),
    ('trident_release', 'trident::release_trident', [('ticks', 'u32')], '"ticks": ticks', UNIT),
    ('mending', 'enchant_effects::mending', [('amount', 'u32')], '"amount": amount', None),
    ('piglin_barter', 'piglins::finish_admiring', [('mob', 'u32')], '"entity": mob', UNIT),
    ('sulfur_split', 'sulfur_cubes::split_sulfur_cube', [('m', '&Mob')], '"entity": m.id', UNIT),
    ('villager_traded', 'villagers::villager_traded', [('id', 'u32')], '"entity": id', UNIT),
    ('exp_points', 'commands::give_points', [('amount', 'i32')], '"amount": amount', None),
    ('advancement_done', 'advancements::award', [('index', 'usize')], '"index": index', None),
    ('dig_abort', 'lan_actions::guest_forget_digging', HOST, GUEST, None),
    ('tnt_spawn', 'tnt::spawn_primed_tnt', [('pos', 'BlockPos')], '"pos": [pos.x, pos.y, pos.z]', UNIT),
    ('dispense_try', 'machines::dispense_from', [('pos', 'BlockPos')], '"pos": [pos.x, pos.y, pos.z]', None),
    ('map_fill', 'maps::use_empty_map', [STACK_PARAM], f'"stack": {STACK}', ('bool', 'false')),
    ('crafter_slot', 'machines::crafter::tick_crafter', [('pos', 'Pos')], POS, None),
    ('sign_open', 'sign_edit::lan_open_sign_editor', HOST, GUEST, UNIT),
    ('ender_eye', 'end::throw_ender_eye', [], '', UNIT),
    ('mob_mount', 'passengers::mount_mob', [('rider', 'u32'), ('vehicle', 'u32')], '"rider": rider, "vehicle": vehicle', UNIT),
    ('egg_use', 'creatures::use_spawn_egg', [STACK_PARAM], f'"stack": {STACK}', None),
    ('bolt', 'lightning::spawn_bolt', [('at', '[f64; 3]'), ('visual_only', 'bool')], '"at": at, "visual_only": visual_only', UNIT),
    ('compost', 'composter::use_composter', [('pos', '[i32; 3]'), ('held', 'Option<&ItemStack>')], '"pos": pos, "stack": held.map(crate::game::paper_query::stack_json)', ('bool', 'false')),
    ('raid_trigger', 'raids::create_or_extend_raid', [('at', '[i32; 3]'), ('amplifier', 'Option<i32>')], '"pos": at, "amplifier": amplifier', ('Option<i32>', 'None')),
    ('raid_wave', 'raids::spawn_wave', [('raid_id', 'i32'), ('pos', '[i32; 3]'), ('wave', 'i32')], '"raid": raid_id, "pos": pos, "wave": wave', None),
    ('potion_splash', 'potions::splash', [STACK_PARAM, ('at', 'Vec3')], f'"stack": {STACK}, "at": [at.x, at.y, at.z]', UNIT),
    ('hook_retrieve', 'fishing::retrieve_hook', [('rod', '&ItemStack')], '"stack": crate::game::paper_query::stack_json(rod)', ('u32', '0')),
    ('bell_hit', 'creatures::bell_projectile_hit', [('pos', '[i32; 3]')], '"pos": pos', UNIT),
    ('hopper_add', 'machines::add_item', [('from_hopper', 'Option<Pos>'), ('stack', '&ItemStack', '&stack')], f'"kind": if from_hopper.is_some() {{ "hopper" }} else {{ "item" }}, "pos": from_hopper.map(|pos| [pos.0, pos.1, pos.2]), "stack": {STACK}', ('Option<ItemStack>', 'Some(stack.clone())')),
    ('sheared', 'dispenser::shear_entity', [('target', 'Pos')], '"pos": [target.0, target.1, target.2]', None),
    ('bowl_pickup', 'campfire::campfire_drop', [('pos', 'Pos'), ('stack', '&ItemStack', '&stack')], f'{POS}, "stack": {STACK}', None),
    ('anchor_use', 'respawn_anchor::use_respawn_anchor', [('pos', '[i32; 3]')], '"pos": pos', None),
    ('mob_egg', 'creatures::spawn_egg_offspring', [('index', 'usize')], '"index": index', None),
    ('guest_attack', 'lan_actions::guest_attack', [*HOST, ('id', 'i32')], f'{GUEST}, "entity": id', UNIT),
    ('frame_drop', 'frames::drop_frame_items', [('by_player', 'bool')], '"by_player": by_player', None),
    ('mount_jump', 'mounts::charge_mount_jump', [('i', 'usize')], '"index": i', None),
    ('mount_feed', 'mounts::feed_mount', [('index', 'usize'), ('item', '&str')], '"index": index, "item": item', ('bool', 'false')),
    ('panda_use', 'pandas::interact_panda', [('index', 'usize'), ('item', '&str')], '"index": index, "item": item', ('bool', 'false')),
    ('golem_use', 'copper_golems::interact_copper_golem', [('index', 'usize'), ('item', '&str')], '"index": index, "item": item', ('bool', 'false')),
    ('armadillo_use', 'creatures::interact_armadillo', [('index', 'usize'), ('item', '&str')], '"index": index, "item": item', ('bool', 'false')),
    ('shear_mob', 'creatures::interact_shear_mob', [('index', 'usize'), ('item', '&str')], '"index": index, "item": item', ('bool', 'false')),
    ('enchant_done', 'lan_stations::lan_enchant', [('host', '&LanHost', '&*host'), ('gi', 'usize'), ('button', 'usize')], '"uuid": format!("{:032x}", host.hub.guests[gi].uuid), "button": button', UNIT),
    ('guest_join', 'lan_actions::enter_guest', [('host', '&LanHost', '&*host'), ('conn', 'u32')], GUEST, None),
    ('open_try', 'lan_actions::lan_open_container', [*HOST, ('pos', '(i32, i32, i32)')], f'{GUEST}, "pos": [pos.0, pos.1, pos.2]', ('bool', 'false')),
    ('why_natural', 'natural_spawning::spawn_natural', [], '"reason": "NATURAL"', None),
    ('why_egg', 'creatures::use_spawn_egg', [], '"reason": "SPAWNER_EGG"', None),
    ('why_breed', 'creatures::breed', [], '"reason": "BREEDING"', None),
    ('why_spawner', 'spawner::tick_spawner', [], '"reason": "SPAWNER"', None),
    ('why_golem', 'villagers::try_spawn_iron_golem', [], '"reason": "VILLAGE_DEFENSE"', None),
    ('why_raid', 'raids::spawn_wave', [], '"reason": "RAID"', None),
    ('why_phantom', 'stats::spawn_phantoms', [], '"reason": "NATURAL"', None),
    ('why_convert', 'creatures::convert_to', [], '"reason": "CONVERSION"', None),
    ('why_trader', 'wandering_trader::spawn_trader', [], '"reason": "NATURAL"', None),
    ('tp_command', 'commands::teleport_to', [], '"reason": "COMMAND"', None),
    ('tp_gateway', 'end::enter_gateway', [], '"reason": "END_GATEWAY"', None),
    ('tp_portal', 'nether_portal::player_through_nether_portal', [], '"reason": "NETHER_PORTAL"', None),
    ('knockback', 'creatures::knock_back_player', [('power', 'f64'), ('xd', 'f64'), ('zd', 'f64')], '"power": power, "x": xd, "z": zd', UNIT),
    ('potion_use', 'potions::use_potion_item', [STACK_PARAM], f'"stack": {STACK}', None),
]




def family_tables(names):
    """The per-family tables in this repository's family/, each run with these names."""
    import pathlib
    out = {'HOOKS': [], 'EVENTS': [], 'WATCH': [], 'EXTRA': []}
    for file in sorted((pathlib.Path(__file__).resolve().parent.parent / 'family').glob('*.py')):
        scope = dict(names)
        exec(compile(file.read_text(encoding='utf-8'), str(file), 'exec'), scope)
        for key in out:
            out[key] += scope.get(key, [])
    return out


FAMILIES = family_tables({'STACK': STACK, 'GUEST': GUEST, 'KEY': KEY, 'POS': POS, 'STACK_PARAM': STACK_PARAM, 'HOST': HOST, 'UNIT': UNIT, 'B': '', 'P': '', 'D': '', 'PLAYER': None})
ROWS += FAMILIES['HOOKS']

# Injections that aren't one event each.
EXTRA = [
    {'id': 'entity_sees', 'target': 'horizon_client::lan::entity::sees', 'kind': 'HEAD', 'cancellable': True, 'params': [('v', '&Viewer', 'v'), ('p', '&Probe', 'p')],
     'code': 'fn paper_hook_entity_sees(v: &Viewer, p: &Probe) -> ::core::ops::ControlFlow<bool> {\n    if crate::paper_hub::hidden_from(v.entity_id, p.id) {\n        return ::core::ops::ControlFlow::Break(false);\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n'},
    {'id': 'table_valid', 'target': "horizon_client::game::lan_crafting::Game<'_>::lan_table_valid", 'kind': 'HEAD', 'cancellable': True, 'params': [('g', '&horizon_client::lan::guest::Guest', 'g'), ('pos', '[i32; 3]', 'pos')],
     'code': 'fn paper_hook_table_valid(g: &horizon_client::lan::guest::Guest, pos: [i32; 3]) -> ::core::ops::ControlFlow<bool> {\n    if horizon_client::paper_hub::forced_menu(g.uuid, pos) {\n        return ::core::ops::ControlFlow::Break(true);\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n'},
    {'id': 'station_valid', 'target': "horizon_client::game::lan_stations::Game<'_>::lan_station_valid", 'kind': 'HEAD', 'cancellable': True, 'params': [('g', '&horizon_client::lan::guest::Guest', 'g'), ('gs', '&GuestStation', 'gs')],
     'code': 'fn paper_hook_station_valid(g: &horizon_client::lan::guest::Guest, gs: &GuestStation) -> ::core::ops::ControlFlow<bool> {\n    if horizon_client::paper_hub::forced_menu(g.uuid, gs.pos) {\n        return ::core::ops::ControlFlow::Break(true);\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n'},
    {'id': 'spawn_limit', 'target': 'horizon_client::mobs::spawn_selection::Category::max', 'kind': 'RETURN', 'params': [('category', 'Category', 'self'), ('value', 'i32', '$value')],
     'code': 'fn paper_hook_spawn_limit(category: Category, value: i32) -> i32 {\n    crate::paper_hub::spawn_limit(category, value)\n}\n'},
    {'id': 'mob_tick_begin', 'target': "horizon_client::game::Game<'_>::tick_mobs", 'kind': 'HEAD', 'params': [],
     'code': 'fn paper_hook_mob_tick_begin() {\n    crate::game::paper_bridge::on_mob_tick_begin();\n}\n'},
    {'id': 'mob_tick_end', 'target': "horizon_client::game::Game<'_>::tick_experience", 'kind': 'HEAD', 'params': [('game', "&mut crate::game::Game<'_>", '&mut *self')],
     'code': "fn paper_hook_mob_tick_end(game: &mut crate::game::Game<'_>) {\n    crate::game::paper_bridge::on_block_tick_end(game);\n}\n"},
    {'id': 'join_line', 'target': 'horizon_client::lan::hub::Hub::broadcast_except', 'kind': 'HEAD', 'cancellable': True, 'params': [('hub', '&Hub', '&*self'), ('packet', '&C', 'packet')],
     'code': 'fn paper_hook_join_line(hub: &Hub, packet: &C) -> ::core::ops::ControlFlow<()> {\n    if let C::SystemChat(chat) = packet && (crate::paper_hub::hold_line(&hub.english(&chat.content), &chat.content) || !crate::paper_hub::allow_game(&hub.english(&chat.content))) {\n        return ::core::ops::ControlFlow::Break(());\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n'},
    {'id': 'game_message_to', 'target': 'horizon_client::lan::hub::Hub::system_message_to', 'kind': 'HEAD', 'cancellable': True, 'params': [('hub', '&Hub', '&*self'), ('content', '&Tag', '&content')],
     'code': 'fn paper_hook_game_message_to(hub: &Hub, content: &Tag) -> ::core::ops::ControlFlow<()> {\n    if let Some(line) = crate::paper_hub::take_death_line() {\n        if let Some(line) = line {\n            hub.system_message(line);\n        }\n        return ::core::ops::ControlFlow::Break(());\n    }\n    if crate::paper_hub::allow_game(&hub.english(content)) {\n        ::core::ops::ControlFlow::Continue(())\n    } else {\n        ::core::ops::ControlFlow::Break(())\n    }\n}\n'},
    {'id': 'block_tick_begin', 'target': "horizon_client::game::Game<'_>::tick_blocks", 'kind': 'HEAD', 'params': [],
     'code': 'fn paper_hook_block_tick_begin() {\n    crate::game::paper_bridge::on_block_tick_begin();\n}\n'},
    # (`tick_blocks` is full of closures, where nothing can be injected at its end: paintings tick next.)
    {'id': 'block_tick_end', 'target': "horizon_client::game::paintings::Game<'_>::tick_paintings", 'kind': 'HEAD', 'params': [('game', "&mut crate::game::Game<'_>", '&mut *self')],
     'code': "fn paper_hook_block_tick_end(game: &mut crate::game::Game<'_>) {\n    crate::game::paper_bridge::on_block_tick_end(game);\n}\n"},
    {'id': 'status_ping', 'target': 'horizon_client::lan::Shared::status_json', 'kind': 'HEAD', 'cancellable': True, 'params': [('shared', '&Shared', '&*self')],
     'code': 'fn paper_hook_status_ping(shared: &Shared) -> ::core::ops::ControlFlow<String> {\n    match crate::paper_hub::status_ping(|| shared.status_json()) {\n        Some(json) => ::core::ops::ControlFlow::Break(json),\n        None => ::core::ops::ControlFlow::Continue(()),\n    }\n}\n'},
]

EXTRA += FAMILIES['EXTRA']


def raws():
    """The rows as gen_bundle.py's injections."""
    out = list(EXTRA)
    for hook, target, params, payload, cancel in ROWS:
        parts = target.split('::')
        path = f"horizon_client::game::Game<'_>::{parts[-1]}" if parts[0] == 'root' else f"horizon_client::game::{'::'.join(parts[:-1])}::Game<'_>::{parts[-1]}"
        names = [(row[0], row[1], row[2] if len(row) > 2 else row[0]) for row in params]
        head = ''.join(f', {name}: {kind}' for name, kind, _ in names)
        result = f' -> ::core::ops::ControlFlow<{cancel[0]}>' if cancel else ''
        if cancel:
            body = f'        if crate::game::paper_bridge::on_refusable(game, "{hook}", payload) {{\n            return ::core::ops::ControlFlow::Break({cancel[1]});\n        }}\n'
            tail = '    ::core::ops::ControlFlow::Continue(())\n'
        else:
            body = f'        crate::game::paper_bridge::on_trigger(game, "{hook}", payload);\n'
            tail = ''
        code = f"fn paper_hook_ev_{hook}(game: &mut crate::game::Game<'_>{head}){result} {{\n    if crate::game::paper_bridge::hears(\"{hook}\") {{\n        let payload = serde_json::json!({{ {payload} }});\n{body}    }}\n{tail}}}\n"
        row = {'id': f'ev_{hook}', 'target': path, 'kind': 'HEAD', 'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), *names], 'code': code}
        if cancel:
            row['cancellable'] = True
        out.append(row)
    return out
