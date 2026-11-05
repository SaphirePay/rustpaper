# Family ev_player. Yours alone. Loaded by event_hooks.py, sdk_events.py and sdk_watch.py.
# Names available here: B P D PLAYER (sdk_events) and STACK GUEST KEY POS STACK_PARAM HOST UNIT (event_hooks).
FAM = 'crate::game::paper_fam_ev_player'
# The player whose turn it is, for what fires inside `as_guest`: `game.lan` is empty then, so a
# listener can't ask for a name, and it comes in the event.
ME = '"uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id()'
MUT_HOST = [('host', '&mut LanHost', '&mut *host'), ('conn', 'u32')]


def row(hook, target, params, payload, cancel=None):
    """A hook that fires through `paper_fam_ev_player::fire`: no `flush`, which would lose what a
    listener said to a player while the hub is out of `game.lan`. It leaves that for the tick's end."""
    made = f'let payload = serde_json::json!({{ {payload} }});'
    if cancel:
        act = f'{made} if {FAM}::fire(game, "{hook}", payload) {{ return ::core::ops::ControlFlow::Break({cancel[1]}); }} return ::core::ops::ControlFlow::Continue(())'
    else:
        act = f'{made} {FAM}::fire(game, "{hook}", payload); return'
    return hook, target, params, f'"x": if true {{ {act} }} else {{ 0 }}', cancel


def direct(hook, target, params, call, cancel=None):
    """A hook that is one call of `paper_fam_ev_player`, which fires what it wants itself."""
    return hook, target, params, f'"x": if true {{ return {FAM}::{call} }} else {{ 0 }}', cancel


def row_if(hook, target, params, test, cancel):
    """A hook whose one call says whether what follows is refused."""
    refused = cancel[1] if cancel else ''
    return hook, target, params, f'"x": if true {{ if {FAM}::{test} {{ return ::core::ops::ControlFlow::Break({refused}); }} return ::core::ops::ControlFlow::Continue(()) }} else {{ 0 }}', cancel


HOPPER_READ = "|g: &mut Game<'_>, at: Pos| { let c = g.containers(); c.view(at, false).map(|v| (0..v.len()).map(|slot| c.get(&v, slot).cloned()).collect::<Vec<_>>()) }"


HOOKS = [    # rows as in fabric-port/tools/event_hooks.py ROWS
    # Menu packets, held until the hub is back: see paper_fam_ev_player.rs.
    direct('pl_packet', 'lan_actions::lan_guest_packet', [*MUT_HOST, ('packet', '&S', '&packet')], 'hold_packet(game, host, conn, packet, |play| &mut play.attack_ticks, |play| play.using)', UNIT),
    direct('pl_held', 'lan_actions::run_guest_commands', [], 'run_held(game)'),
    direct('pl_tables', 'lan_crafting::lan_open_crafting', [], 'tables_are(|host, conn| host.play.get_mut(&conn)?.crafting.table.as_mut().map(|table| (table.0, table.3, &mut *table.2)))'),
    direct('pl_stations', 'lan_stations::lan_open_station', [], 'stations_are(|host, conn| host.play.get_mut(&conn)?.station.as_mut().map(|station| (station.window, station.pos, &mut *station.menu)))'),
    direct('pl_trades', 'lan_trading::lan_open_merchant', [], 'trades_are(|host, conn| host.play.get_mut(&conn)?.merchant.as_mut().map(|trade| (trade.window, &mut trade.slots)))'),

    row('pl_fish', 'fishing::use_fishing_rod', [STACK_PARAM], f'{ME}, "state": (&*stack.item == "minecraft:fishing_rod" && game.hook.is_none()).then_some("FISHING"), "hand": {FAM}::hand(game)', ('bool', 'false')),
    row('pl_fish_reel', 'fishing::retrieve_hook', [('rod', '&ItemStack')],
        f'{ME}, "state": game.hook.as_ref().and_then(|h| if matches!(h.state, State::Hooked(_)) {{ Some("CAUGHT_ENTITY") }} else if h.nibble > 0 {{ None }} else if h.on_ground {{ Some("IN_GROUND") }} else {{ Some("REEL_IN") }}), '
        f'"caught": game.hook.as_ref().and_then(|h| if let State::Hooked(id) = h.state {{ Some(id) }} else {{ None }}), "hook": game.hook.as_ref().map(|h| h.id), "hand": {FAM}::hand(game)', ('u32', '0')),
    direct('pl_fish_xp', 'root::spawn_orbs', [('at', '[f64; 3]')], 'fish_orbs(game, at, |game, at, value| game.spawn_orbs(at, value))', UNIT),
    direct('pl_exp', 'root::give_orb_points', [('gained', 'u32')], 'exp_change(game, gained, |game, amount| game.give_orb_points(amount))', UNIT),

    direct('pl_consume', 'consume::finish_consuming', [STACK_PARAM], 'consume_gate(game, stack, |game, eaten| game.res.items().get(&eaten.item).and_then(|info| info.consumable.clone().map(|consumable| (info.clone(), consumable))), |game, info, consumable, eaten| game.finish_consuming(info, consumable, eaten))', UNIT),
    direct('pl_item_damage', 'enchant_effects::wear', [STACK_PARAM, ('amount', 'u32')], 'wear_gate(game, stack, amount)', ('u32', '0')),
    row('pl_item_break', 'item_particles::eat_particles', [STACK_PARAM], f'{ME}, "stack": {STACK}, "broken": game.res.items().get(&stack.item).is_some_and(|item| item.max_damage > 0 && stack.damage >= item.max_damage)'),
    row('pl_stand_use', 'armor_stands::interact_armor_stand', [], f'{ME}, "use": {FAM}::stand_use(game, |game, held| game.equipment_slot(held))', ('bool', 'true')),
    direct('pl_shear', 'creatures::interact_shear_mob', [('index', 'usize'), ('item', '&str')], "shear_golem(game, index, item, |g| { let m = &g.mobs[index]; if !m.alive() || m.extra.sheared { return None; } let table = match m.species() { Species::SnowGolem => \"minecraft:shearing/snow_golem\", Species::Bogged => \"minecraft:shearing/bogged\", _ => return None }; let tool = g.held().cloned(); Some(g.sequence_loot(table, table, tool.as_ref())) })", ('bool', 'false')),
    row('pl_game_mode', 'commands::set_game_mode', [('mode', 'GameMode')], f'{ME}, "mode": mode.name(), "changes": game.mode != mode, "cause": {FAM}::mode_cause(game)', ('bool', 'false')),
    direct('pl_mend', 'enchant_effects::mending', [('amount', 'u32')], 'mend(game, amount)', ('u32', 'amount')),
    direct('pl_orb', 'lan_actions::guest_orbs', [('host', '&mut LanHost', '&mut *host'), ('gi', 'usize')], 'orb_gate(game, host, gi, |play| &mut play.xp_delay, |game, host, gi| game.guest_orbs(host, gi))', UNIT),
    direct('pl_cooldown', 'root::add_cooldown', [('group', '&Arc<str>', '&group'), ('ticks', 'i32')], 'cooldown_gate(game, group, ticks, |game, group, ticks| game.add_cooldown(group, ticks))', UNIT),
    direct('pl_shield_hit', 'shield::after_block', [('attacker', 'Option<u32>')], 'shield_attacker(attacker)'),
    direct('pl_shield', 'shield::blocked_by_attacker', [('component', '&horizon_client::items::BlocksAttacks', 'component'), ('attacker', '[f64; 2]'), ('fully', 'bool'), ('disable', 'f32'), ('at', 'Option<[f64; 3]>')],
           'shield_gate(game, disable, component.disable_scale, |game, disable| game.blocked_by_attacker(component, attacker, fully, disable, at))', UNIT),
    direct('pl_throw', 'creatures::throw_item', [STACK_PARAM], 'throw_gate(game, stack, |game, stack| game.throw_item(stack))', ('Result<bool>', 'Ok(false)')),
    direct('pl_throw_bottle', 'potions::use_potion_item', [STACK_PARAM], 'throw_gate(game, stack, |game, stack| game.use_potion_item(stack))', ('horizon_client::Result<bool>', 'Ok(false)')),
    direct('pl_launch', 'creatures::add_projectile', [('p', '&mut horizon_client::projectiles::Projectile', '&mut p')], 'launch(game, p)', UNIT),
    direct('pl_egg', 'creatures::projectile_hit', [('p', '&Projectile'), ('hit', '&Hit', '&hit')],
           'egg_hit(game, p.id, matches!(p.kind, ProjectileKind::Egg), [p.position.x, p.position.y, p.position.z], p.guest, |game| game.projectile_hit(p, hit.clone()))', ('Result<()>', 'Ok(())')),
    direct('pl_camera', 'lan_camera::guest_set_camera', [('host', '&LanHost', '&*host'), ('gi', 'usize'), ('camera', 'Option<i32>')], 'camera_gate(game, host, gi, game.cmds.guest_cameras.get(&host.hub.guests[gi].conn).map(|c| c.id), camera)', UNIT),
    row('pl_age_lock', 'creatures::set_age_locked', [('index', 'usize')], f'{ME}, "entity": game.mobs.get(index).map(|mob| mob.id), "locked": game.mobs.get(index).is_some_and(|mob| !mob.extra.age_locked), "stack": game.held().map({FAM}::stack_value), "hand": {FAM}::hand(game)', UNIT),
    row('pl_bucket_mob', 'creatures::interact_mob', [], f'{ME}, "scoop": {FAM}::scooped(game, |mob| bucketable(mob.species()).map(|found| found.0)), "stack": game.held().map({FAM}::stack_value), "hand": {FAM}::hand(game)', ('Result<bool>', 'Ok(true)')),
    direct('pl_bucket', 'native_hooks::native_fill_bucket', [STACK_PARAM, ('pos', '[i32; 3]'), ('canceled', '&mut bool', '&mut *canceled')], 'bucket_gate(game, stack, pos, canceled)'),
    row('pl_portal', 'server_travel::start_server_trip', [('host', '&LanHost', '&*host'), ('conn', 'u32'), ('target', 'Dim'), ('plan', '&Option<Plan>', '&plan')],
        '"uuid": crate::game::paper_bridge::uuid_of(host, conn), "name": host.hub.guest(conn).map(|g| g.name.clone()), "level": target.id(), "from": game.dim.id(), "back": host.hub.guest(conn).map(|g| g.position), "to": plan.as_ref().map(|plan| [plan.approximate.x, plan.approximate.y, plan.approximate.z]).or(if target == Dim::End { Some(super::end::END_SPAWN) } else { game.level.world_spawn }), "cause": if plan.is_some() { "NETHER_PORTAL" } else { "END_PORTAL" }', UNIT),
    direct('pl_sign_used', 'sign_edit::use_sign_empty_hand', [], 'sign_used(game)'),
    direct('pl_sign_click', 'sign_edit::execute_sign_click_commands', [('front', 'bool')], 'sign_side(front)'),
    direct('pl_lecterns', 'lan_stations::lan_open_lectern', [], 'lecterns_are(|host, conn| host.play.get(&conn)?.lectern.as_ref().map(|lectern| (lectern.window, lectern.key)))'),
    direct('pl_mounts', 'lan_mount_menu::lan_open_mount', [], 'mounts_are(|host, conn| host.play.get(&conn)?.mount_window.as_ref().map(|mount| (mount.window, mount.mob, mount.columns)), |game, index, columns| game.mount_engine_menu(index, columns), |game, index, columns, menu| game.lan_mount_store(index, columns, menu))'),
    row('pl_move_item', 'machines::add_item', [('from_hopper', 'Option<Pos>'), ('view', '&View', 'view'), ('stack', '&ItemStack', '&stack'), ('face', 'Option<Direction>')],
        f'"uuid": null, "again": {FAM}::moved_again(), "from": from_hopper.map(|pos| [pos.0, pos.1, pos.2]).or({FAM}::pulled()), "pull": {FAM}::pulled().is_some(), "to": view.parts.first().map(|part| [part.0.0, part.0.1, part.0.2]), "stack": {STACK}, "level": game.dim.id(), "item": {FAM}::sucked_item(game, from_hopper.is_none() && {FAM}::pulled().is_none(), view.parts.first().map(|part| part.0), stack)',
        ('Option<ItemStack>', f'match {FAM}::take_moved() {{ Some(new) => game.add_item(from_hopper, view, new, face), None => Some(stack.clone()) }}')),
    row('pl_sign_open', 'sign_edit::lan_open_sign_editor', [('host', '&LanHost', '&*host'), ('conn', 'u32'), ('edit', '&SignEditor', '&edit')],
        '"uuid": crate::game::paper_bridge::uuid_of(host, conn), "name": host.hub.guest(conn).map(|g| g.name.clone()), "pos": [edit.pos.0, edit.pos.1, edit.pos.2], "front": edit.front, "level": game.dim.id(), "cause": ' + FAM + '::sign_cause(game)', UNIT),
    direct('pl_advancement', 'advancements::award', [('index', 'usize')], 'award_gate(game, index, |game, index| game.award(index))', UNIT),
    direct('pl_name', 'names::name_mob', [('index', 'usize')], 'name_gate(game, index, |game, index| game.name_mob(index))', ('bool', 'false')),
    direct('pl_map_fill', 'maps::use_empty_map', [STACK_PARAM], 'map_gate(game, stack, |game, stack| game.use_empty_map(stack))', ('bool', 'false')),
    row_if('pl_riptide', 'trident::release_trident', [('ticks', 'u32')], 'riptide(game.in_water_or_rain(), view_vector(game.yaw, game.pitch), game, ticks)', UNIT),
    direct('respawn', 'respawn_anchor::find_respawn_and_use', [('c', 'RespawnConfig'), ('consume', 'bool')], 'respawn_gate(game, c.pos, consume, |found: &Result<Option<(Vec3, f32)>>| matches!(found, Ok(None)), |g| g.find_respawn_and_use(c, consume))', ('Result<Option<(Vec3, f32)>>', 'Ok(None)')),
    direct('pl_velocity', 'creatures::knock_back_player', [('power', 'f64'), ('xd', 'f64'), ('zd', 'f64')], 'velocity_gate(game, |game| game.knock_back_player(power, xd, zd))', UNIT),
    direct('pl_boost', 'fireworks::boost_firework', [STACK_PARAM], 'boost_gate(game, stack, |game, stack| game.boost_firework(stack))', ('bool', 'false')),
    # Named as a bridge event, so these run when a plugin hears that one.
    direct('use_block', 'lan_actions::lan_guest_packet', [('host', '&LanHost', '&*host'), ('conn', 'u32'), ('packet', '&S', '&packet')], 'note_click(host, conn, packet)'),
    direct('use_entity', 'lan_actions::lan_guest_packet', [('host', '&LanHost', '&*host'), ('conn', 'u32'), ('packet', '&S', '&packet')], 'note_click(host, conn, packet)'),
    direct('pl_frame', 'frames::interact_frame', [('target', 'Option<InteractTarget>')], 'frame_gate(game, false, target.map(|target| target.id), |game| game.interact_frame(target))', ('bool', 'true')),
    direct('pl_frame_hit', 'frames::attack_frame', [], 'frame_gate(game, true, None, |game| game.attack_frame())', ('bool', 'true')),
    direct('pl_furnace', 'block_entity_ticks::tick_furnace', [('pos', 'Pos')], 'furnace_tick(game, pos, |game, pos| game.tick_furnace(pos))', ('Result<()>', 'Ok(())')),
    direct('pl_brew', 'stations::tick_brewing_stand', [('pos', '(i32, i32, i32)')], 'brew_tick(game, pos, |game, pos| game.tick_brewing_stand(pos))', ('horizon_client::Result<()>', 'Ok(())')),
    row('pl_sign_command', 'sign_edit::run_sign_command', [('pos', '[i32; 3]'), ('command', '&str')], f'{ME}, "pos": pos, "command": command, "front": {FAM}::sign_front()', UNIT),
    row('pl_armor', 'player_equip::player_on_equip', [('slot', '&str'), ('old', 'Option<&ItemStack>'), ('new', 'Option<&ItemStack>')], f'{ME}, "slot": slot, "old": old.map({FAM}::stack_value), "new": new.map({FAM}::stack_value)'),
    direct('pl_pulling', 'machines::suck_from_slots', [('source', '&View', 'source')], 'pulling(source.parts.first().map(|part| part.0))'),
    direct('pl_not_pulling', 'machines::suck_item_entities', [], 'pulling(None)'),
    direct('pl_hopper_turn', 'machines::hopper_try_move_items', [], 'pulling(None)'),
    direct('pl_closing', 'lan_actions::guest_container_closed', [*MUT_HOST], 'close_gate(game, host, conn, None)'),
    direct('pl_closing_table', 'lan_crafting::lan_crafting_closed', [*MUT_HOST, ('window', 'Option<i32>')], 'close_gate(game, host, conn, Some(window))'),
    direct('pl_closing_station', 'lan_stations::lan_station_closed', [*MUT_HOST, ('window', 'Option<i32>')], 'close_gate(game, host, conn, Some(window))'),
    direct('pl_move_turn', 'native_server::register_native_commands', [('host', '&mut LanHost', '&mut *host')], 'moves_turn(host)'),
    direct('pl_moves', 'native_menus::run_native_menu_events', [], 'moves_check(game)'),
    direct('pl_tp_command', 'commands::cmd_teleport', [], 'tp_cause(game)'),
    direct('pl_teleport', 'commands::teleport_to_in_level', [('t', 'Target'), ('pos', '[f64; 3]'), ('rotation', 'Option<[f32; 2]>')], 'teleport_gate(game, t == Target::Player, pos, rotation, |game, pos, rotation| game.teleport_to_in_level(Target::Player, pos, rotation))', UNIT),
    direct('pl_pearl', 'creatures::projectile_hit', [('p', '&Projectile'), ('hit', '&Hit', '&hit')],
           'pearl_hit(game, matches!(p.kind, ProjectileKind::EnderPearl) && p.owner == Some(Who::Player), [p.previous.x, p.previous.y, p.previous.z], |game| game.projectile_hit(p, hit.clone()))', ('Result<()>', 'Ok(())')),
    direct('pl_chorus', 'consume::teleport_randomly', [('teleport', '&TeleportConsume', 'teleport')], 'chorus(game, |game| game.teleport_randomly(teleport))', UNIT),
    direct('pl_spectate', 'lan_actions::guest_spectate_teleport', [('host', '&mut LanHost', '&mut *host'), ('gi', 'usize'), ('uuid', 'u128')], 'spectate_gate(game, host, gi, uuid)', UNIT),
    direct('pl_recipes', 'advancements::recipe_unlocked', [('unlocked', '&[String]')], 'recipe_gate(game, unlocked, |g, kept| g.recipe_unlocked(kept))', UNIT),
    direct('pl_shear_mob', 'creatures::interact_mob', [('target', 'Option<InteractTarget>')], "shear_gate(game, target.map(|target| target.id), |g, index| { let m = &g.mobs[index]; if !m.alive() || m.is_baby() { return None; } match m.species() { Species::Mooshroom => Some(vec![ItemStack::new(if m.extra.variant == 1 { \"minecraft:brown_mushroom\" } else { \"minecraft:red_mushroom\" }, 5)]), Species::Sheep if !m.extra.sheared => { let color = horizon_client::mobs::render::DYE_NAMES[m.extra.color as usize & 15]; Some(world_items!(g).loot_table(&format!(\"minecraft:shearing/sheep/{color}\"), false, None, &mut g.world.random)) } _ => None } }, Ok(true))", ('Result<bool>', 'Ok(true)')),
    direct('pl_sync', 'lan::tick_lan', [], 'sync_is(|| SYNC.lock().ok().and_then(|sync| sync.as_ref()?.1.as_ref().ok().cloned()))'),
    direct('pl_portal_exit', 'nether_portal::nether_exit_portal', [('plan', '&Plan')], 'portal_gate([plan.approximate.x, plan.approximate.y, plan.approximate.z], game, |g, radius| { let to_nether = plan.dim == Dim::Nether; let border = g.border_bounds(); let portals = g.portal_blocks(plan.approximate, radius.unwrap_or(if to_nether { 16 } else { 128 })); let (low, high) = if to_nether { (0, 15) } else { (-4, 19) }; closest_portal(&portals, plan.approximate, to_nether, &border, low, high).is_some() }, None)', ('Option<FoundRectangle>', 'None')),
    direct('pl_bed_blast', 'respawn_anchor::destroy_bed_on_use', [('head', '[i32; 3]')], 'bed_blast(game, head, Ok(()))', ('Result<()>', 'Ok(())')),
    direct('pl_say', 'lan_chat::guest_message_command', [*MUT_HOST, ('command', '&str')], 'say_gate(game, host, conn, command, |g, h, line| g.guest_message_command(h, conn, line, None))', ('bool', 'false')),
    direct('pl_pickup_trident', 'trident::trident_taken', [('t', '&ThrownTrident'), ('by', '&Catcher')], 'trident_gate(game, by.guest, t.id, &t.stack, t.creative_only)', ('bool', 'false')),
    direct('pl_pickup_arrow', 'lan_actions::guest_arrows', [('host', '&mut LanHost', '&mut *host'), ('gi', 'usize')], 'arrow_gate(game, host, gi, |g, h| g.guest_arrows(h, gi))', UNIT),
    direct('pl_open_native', 'native_menus::native_open_menu', [('uuid', 'u128'), ('rows', 'u8'), ('title', '&Text', '&title'), ('items', '&Vec<Option<ItemStack>>', '&items')], 'open_native(game, uuid, rows, items, |g, retitled| g.native_open_menu(uuid, rows, retitled.map_or_else(|| title.clone(), Text::literal), items.clone()))', ('Option<i32>', 'None')),
    direct('pl_hopper_out', 'machines::hopper_eject', [('pos', 'Pos'), ('hopper', '&View')], f'hopper_gate({{ let state = game.world.block(pos.0, pos.1, pos.2); game.containers().prop(state, "facing").and_then(dir_from_name).map(|facing| offset(pos, facing)) }}, game, pos, false, {HOPPER_READ}, |g, at| {{ let state = g.world.block(pos.0, pos.1, pos.2); let face = g.containers().prop(state, "facing").and_then(dir_from_name); match (at, face) {{ (Some(at), Some(face)) if at == offset(pos, face) => g.hopper_eject(pos, hopper), (Some(at), Some(face)) => match g.containers().view(at, false) {{ Some(target) => g.eject_into(pos, hopper, &target, face.opposite()), None => false }}, _ => false }} }})', ('bool', 'false')),
    direct('pl_hopper_in', 'machines::hopper_suck', [('pos', 'Pos'), ('hopper', '&View')], f'hopper_gate(Some((pos.0, pos.1 + 1, pos.2)), game, pos, true, {HOPPER_READ}, |g, at| match at {{ Some(at) if at == (pos.0, pos.1 + 1, pos.2) => g.hopper_suck(pos, hopper), Some(at) => match g.containers().view(at, false) {{ Some(source) => g.suck_from_slots(None, &source, hopper), None => false }}, None => {{ let (min, max) = ([f64::from(pos.0), f64::from(pos.1) + 11.0 / 16.0, f64::from(pos.2)], [f64::from(pos.0) + 1.0, f64::from(pos.1) + 2.0, f64::from(pos.2) + 1.0]); g.suck_item_entities(hopper, min, max) }} }})', ('bool', 'false')),
    row_if('pl_hopper_take', 'machines::take_one', [('slot', 'usize')], 'hopper_slot(slot)', ('Option<ItemStack>', 'None')),
    direct('pl_kick_note', 'commands::extra::cmd_kick', [], 'kick_note(game)'),
    direct('pl_login', 'lan::lan_guest_joined', [*MUT_HOST], 'login_gate(game, host, conn)', UNIT),
    direct('pl_stats', 'lan_actions::leave_guest', [], 'stat_events(game)'),
    direct('pl_stats_on', 'lan_actions::enter_guest', [], 'stats_on()'),
    direct('pl_open', 'root::open_menu', [('pos', '[i32; 3]')], 'menu_later(game, pos)', UNIT),
    direct('pl_open_mount', 'mount_menu::open_mount_menu', [('index', 'usize')], 'mount_later(game, index)', UNIT),
    direct('pl_open_boat', 'boats::open_chest_boat_container', [('i', 'usize')], 'vehicle_later(game, game.boats[i].id, game.boats[i].items.clone(), |g, id| if let Some(i) = g.boats.iter().position(|b| b.id == id) { g.open_chest_boat_container(i) })', UNIT),
    direct('pl_open_cart', 'minecarts::open_cart_container', [('i', 'usize')], 'vehicle_later(game, game.carts[i].id, game.carts[i].items.clone(), |g, id| if let Some(i) = g.carts.iter().position(|c| c.id == id) { g.open_cart_container(i) })', UNIT),
    direct('pl_open_left', 'lan_actions::leave_guest', [('host', '&mut LanHost', '&mut *host')], 'open_mark(host)'),
    row('pl_equip', 'armor_stands::equip_from_hand', [STACK_PARAM], f'{ME}, "stack": {STACK}, "slot": game.res.items().get(&stack.item).and_then(|item| item.equip_slot.clone()), "worn": {FAM}::worn_in(game, stack), "hand": {FAM}::hand(game)', ('bool', 'false')),
    direct('pl_lectern_put', 'books::use_lectern', [('pos', '[i32; 3]'), ('held', 'Option<&ItemStack>')], 'book_gate(game, pos, held, |game, pos, held| game.use_lectern(pos, held))', ('bool', 'true')),
    direct('disconnect', 'lan_crafting::lan_crafting_left', [('g', '&horizon_client::lan::guest::Guest', '&*g')], 'note_left(g)'),
    direct('pl_deep_sleep', 'lan_actions::guest_sleep_tick', [('host', '&LanHost', '&*host'), ('gi', 'usize')], 'deep_sleep(game, host, gi)', UNIT),
]
EVENTS = []   # rows as in plugin-tools/tools/sdk_events.py EVENTS
WATCH = []    # rows as in plugin-tools/tools/sdk_watch.py ROWS

EXTRA = [
    # Who a plugin sent back to configuration while the hub was out of `game.lan`.
    {'id': 'pl_reconfigure_turn', 'target': "horizon_client::game::native_menus::Game<'_>::run_native_menu_events", 'kind': 'HEAD', 'params': [('game', "&mut crate::game::Game<'_>", '&mut *self')],
     'code': "fn paper_hook_pl_reconfigure_turn(game: &mut crate::game::Game<'_>) {\n    crate::game::paper_fam_ev_player::reconfigure_turn(game);\n}\n"},
    # `Hub::wake` has no game to ask: a sleeper waits for the tick, if a listener wants to be asked.
    {'id': 'pl_wake', 'target': 'horizon_client::lan::hub::Hub::wake', 'kind': 'HEAD', 'cancellable': True, 'params': [('hub', '&Hub', '&*self'), ('i', 'usize', 'i')],
     'code': 'fn paper_hook_pl_wake(hub: &Hub, i: usize) -> ::core::ops::ControlFlow<()> {\n    let g = &hub.guests[i];\n    if crate::paper_hub::wake_held(g.uuid, g.bed.is_some() || g.sleeping) {\n        return ::core::ops::ControlFlow::Break(());\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n'},
    # Whose commands went out, for `PlayerCommandSendEvent` at the next turn.
    {'id': 'pl_commands_sent', 'target': 'horizon_client::lan::hub::Hub::send_commands', 'kind': 'HEAD', 'cancellable': True, 'params': [('hub', '&Hub', '&*self'), ('i', 'usize', 'i')],
     'code': 'fn paper_hook_pl_commands_sent(hub: &Hub, i: usize) -> ::core::ops::ControlFlow<()> {\n    if crate::paper_hub::commands_sent(hub.guests[i].uuid) {\n        return ::core::ops::ControlFlow::Break(());\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n'},
    # `Hub::kick` has no game to ask: a kick waits for the turn, if a listener wants to be asked.
    {'id': 'pl_kick', 'target': 'horizon_client::lan::hub::Hub::kick', 'kind': 'HEAD', 'cancellable': True, 'params': [('conn', 'u32', 'conn'), ('reason', '&Tag', '&reason')],
     'code': 'fn paper_hook_pl_kick(conn: u32, reason: &Tag) -> ::core::ops::ControlFlow<()> {\n    if crate::paper_hub::kick_held(conn, reason) {\n        return ::core::ops::ControlFlow::Break(());\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n'},
    # `portal_blocks` takes `&self`, which a row can't.
    {'id': 'pl_portal_radius', 'target': "horizon_client::game::nether_portal::Game<'_>::portal_blocks", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', "&crate::game::Game<'_>", '&*self'), ('center', 'BlockPos', 'center')],
     'code': "fn paper_hook_pl_portal_radius(game: &crate::game::Game<'_>, center: BlockPos) -> ::core::ops::ControlFlow<Vec<BlockPos>> {\n    match crate::game::paper_fam_ev_player::portal_radius() {\n        Some(radius) => ::core::ops::ControlFlow::Break(game.portal_blocks(center, radius)),\n        None => ::core::ops::ControlFlow::Continue(()),\n    }\n}\n"},
    # The tree the hub gets is made here, once: the game keeps the list, for `PlayerCommandSendEvent`.
    {'id': 'pl_mod_commands', 'target': 'horizon_client::game::native_server::command_tree', 'kind': 'HEAD', 'params': [('commands', '&[NativeCommand]', 'commands')],
     'code': 'fn paper_hook_pl_mod_commands(commands: &[NativeCommand]) {\n    crate::game::paper_fam_ev_player::mod_commands_are(commands);\n}\n'},
]
