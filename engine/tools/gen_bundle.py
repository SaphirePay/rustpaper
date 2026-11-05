#!/usr/bin/env python3
"""Write the paper_events bundle's wiring: horizon-mixin.toml and one helper per hook.

Usage: gen_bundle.py HOOKS_JSON HORIZON_COMMIT

Every hook is checked against the pinned source's native-hooks.json first, so a hook that
doesn't exist stops here and not forty minutes into a build.
"""
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BUNDLE = ROOT / 'bundle' / 'paper_api'

COMMIT = sys.argv[2] if len(sys.argv) > 2 else ''
GAME = ('game', "&mut crate::game::Game<'_>", '&mut *self')
PHASE_T = 'crate::game::native_lifecycle::ServerTickPhase'
WORLD_T = 'crate::game::native_lifecycle::WorldPhase'
STATE_T = 'horizon_world::registry::BlockStateId'
STACK_T = 'horizon_client::items::ItemStack'
PLAYER_T = 'crate::game::native_server::NativeServerPlayer'
COMMAND_T = 'crate::game::native_server::NativeCommand'
DROPS_T = "crate::game::native_hooks::NativeLootDrops<'_>"
MUT_BOOL = '&mut bool'
POS_T = 'horizon_core::position::BlockPos'


def by_value(name, ty):
    return (name, ty, name)


def by_mut(name, ty):
    return (name, f'&mut {ty}', f'&mut *{name}')


# hook, bindings, the body of its helper.
# Injections with no named hook: into the game's own code, pinned to the commit.
GAME_T = "&mut crate::game::Game<'_>"
RAW = [
    {'id': 'launch', 'target': 'horizon_client::game::run', 'kind': 'HEAD', 'params': [],
     'code': 'fn paper_hook_launch() {\n    crate::game::paper_events::at_launch();\n    crate::game::paper_plugins::launch();\n}\n'},
    {'id': 'chat', 'target': 'horizon_client::lan::hub::chat::Hub::handle_chat', 'kind': 'BEFORE_CALL', 'selector': 'self.broadcast_chat', 'require': 1, 'ordinal': 0, 'cancellable': True,
     'params': [('hub', '&Hub', '&*self'), ('i', 'usize', 'i'), ('message', '&ChatMessage', '$local_ref:message:0')],
     'code': 'fn paper_hook_chat(hub: &Hub, i: usize, message: &ChatMessage) -> ::core::ops::ControlFlow<()> {\n    let g = &hub.guests[i];\n    if crate::paper_hub::allow_chat(g.uuid, &g.name, &message.content) {\n        ::core::ops::ControlFlow::Continue(())\n    } else {\n        ::core::ops::ControlFlow::Break(())\n    }\n}\n'},
    {'id': 'respawn', 'target': 'horizon_client::lan::hub::Hub::respawn', 'kind': 'HEAD',
     'params': [('hub', '&Hub', '&*self'), ('i', 'usize', 'i')],
     'code': 'fn paper_hook_respawn(hub: &Hub, i: usize) {\n    let g = &hub.guests[i];\n    crate::paper_hub::push(crate::paper_hub::Pending::Respawned { player: g.uuid, name: g.name.clone() });\n}\n'},
    {'id': 'change_level', 'target': 'horizon_client::lan::hub::Hub::change_level', 'kind': 'HEAD',
     'params': [('hub', '&Hub', '&*self'), ('i', 'usize', 'i')],
     'code': 'fn paper_hook_change_level(hub: &Hub, i: usize) {\n    let g = &hub.guests[i];\n    crate::paper_hub::push(crate::paper_hub::Pending::ChangedLevel { player: g.uuid, name: g.name.clone(), from: g.dimension.to_string(), to: hub.config.dimension.to_string() });\n}\n'},
    {'id': 'game_message', 'target': 'horizon_client::lan::hub::Hub::system_message', 'kind': 'HEAD', 'cancellable': True,
     'params': [('hub', '&Hub', '&*self'), ('content', '&Tag', '&content')],
     'code': 'fn paper_hook_game_message(hub: &Hub, content: &Tag) -> ::core::ops::ControlFlow<()> {\n    if let Some(line) = crate::paper_hub::take_death_line() {\n        if let Some(line) = line {\n            hub.system_message(line);\n        }\n        return ::core::ops::ControlFlow::Break(());\n    }\n    if crate::paper_hub::hold_line(&hub.english(content), content) {\n        return ::core::ops::ControlFlow::Break(());\n    }\n    if crate::paper_hub::allow_game(&hub.english(content)) {\n        ::core::ops::ControlFlow::Continue(())\n    } else {\n        ::core::ops::ControlFlow::Break(())\n    }\n}\n'},
    {'id': 'command_chat', 'target': 'horizon_client::lan::hub::chat::Hub::broadcast_command_chat', 'kind': 'HEAD', 'cancellable': True,
     'params': [('hub', '&Hub', '&*self'), ('i', 'usize', 'i'), ('chat_type', '&str', 'chat_type'), ('text', '&str', 'text')],
     'code': 'fn paper_hook_command_chat(hub: &Hub, i: usize, chat_type: &str, text: &str) -> ::core::ops::ControlFlow<()> {\n    let guest = &hub.guests[i];\n    if crate::paper_hub::allow_command(chat_type, text, Some(guest.uuid), &guest.name) {\n        ::core::ops::ControlFlow::Continue(())\n    } else {\n        ::core::ops::ControlFlow::Break(())\n    }\n}\n'},
    {'id': 'command_message', 'target': "horizon_client::game::commands::Game<'_>::cmd_say", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('id', '&str', 'id'), ('ctx', '&Context', 'ctx')],
     'code': 'fn paper_hook_command_message(game: &mut crate::game::Game<\'_>, id: &str, ctx: &Context) -> ::core::ops::ControlFlow<R<i32>> {\n    let message = match ctx.get(if id == "me" { "action" } else { "message" }) {\n        Some(Arg::Str(text)) => text.as_str(),\n        _ => "",\n    };\n    let (sender, name) = game.message_source();\n    let kind = if id == "say" { "say_command" } else { "emote_command" };\n    if id == "msg" || horizon_client::paper_hub::allow_command(kind, message, sender, &name) {\n        ::core::ops::ControlFlow::Continue(())\n    } else {\n        ::core::ops::ControlFlow::Break(Ok(1))\n    }\n}\n'},
    {'id': 'save', 'target': "horizon_client::game::Game<'_>::save_game_with", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', GAME_T, '&mut *self'), ('wait', 'bool', 'wait')],
     'code': "fn paper_hook_save(game: &mut crate::game::Game<'_>, wait: bool) -> ::core::ops::ControlFlow<Result<()>> {\n    if !crate::game::paper_events::save_begins(game, wait) {\n        return ::core::ops::ControlFlow::Continue(());\n    }\n    let done = game.save_game_with(wait);\n    crate::game::paper_events::save_ended(game);\n    ::core::ops::ControlFlow::Break(done)\n}\n"},
    {'id': 'save_closing', 'target': "horizon_client::game::saving::Game<'_>::begin_closing_save", 'kind': 'HEAD',
     'params': [('game', GAME_T, '&mut *self')],
     'code': "fn paper_hook_save_closing(game: &mut crate::game::Game<'_>) {\n    crate::game::paper_events::closing_save_begins(game, true);\n}\n"},
    {'id': 'sleep_allow', 'target': "horizon_client::game::Game<'_>::try_sleep", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', GAME_T, '&mut *self'), ('pos', '[i32; 3]', 'pos')],
     'code': "fn paper_hook_sleep_allow(game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> ::core::ops::ControlFlow<()> {\n    if crate::game::paper_events::on_try_sleep(game, pos) {\n        ::core::ops::ControlFlow::Continue(())\n    } else {\n        ::core::ops::ControlFlow::Break(())\n    }\n}\n"},
    {'id': 'effect_add', 'target': "horizon_client::game::potions::Game<'_>::add_player_effect", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', GAME_T, '&mut *self'), ('effect', '&EffectInstance', '&effect')],
     'code': "fn paper_hook_effect_add(game: &mut crate::game::Game<'_>, effect: &EffectInstance) -> ::core::ops::ControlFlow<bool> {\n    if crate::game::paper_events::on_effect_add(game, effect.id, effect.amplifier) {\n        ::core::ops::ControlFlow::Continue(())\n    } else {\n        ::core::ops::ControlFlow::Break(false)\n    }\n}\n"},
    {'id': 'sleep_monsters', 'target': "horizon_client::game::Game<'_>::try_sleep", 'kind': 'MODIFY_VARIABLE', 'variable': '{ name = "monsters", ordinal = 0, at = "LOAD" }', 'require': 1, 'ordinal': 0,
     'params': [('value', 'bool', '$value'), ('game', GAME_T, '&mut *self'), ('pos', '[i32; 3]', 'pos')],
     'code': "fn paper_hook_sleep_monsters(value: bool, game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> bool {\n    crate::game::paper_events::on_nearby_monsters(game, pos, value)\n}\n"},
    {'id': 'convert_from', 'target': "horizon_client::game::creatures::Game<'_>::convert_to", 'kind': 'HEAD',
     'params': [('game', "&crate::game::Game<'_>", '&*self'), ('from', '&Mob', 'from'), ('kind', '&str', 'kind'), ('params', 'horizon_client::mobs::conversion::Params', 'params')],
     'code': "fn paper_hook_convert_from(game: &crate::game::Game<'_>, from: &Mob, kind: &str, params: horizon_client::mobs::conversion::Params) {\n    if game.res.mob_types().get(kind).is_some() {\n        crate::game::paper_events::on_converting(from.id, from.kind.id.as_ref(), params.keep_equipment);\n    }\n}\n"},
    {'id': 'convert_to', 'target': "horizon_client::game::native_hooks::Game<'_>::native_mobs_joined", 'kind': 'HEAD',
     'params': [('game', GAME_T, '&mut *self'), ('from', 'usize', 'from')],
     'code': "fn paper_hook_convert_to(game: &mut crate::game::Game<'_>, from: usize) {\n    crate::game::paper_events::on_converted(game, from);\n}\n"},
    {'id': 'wire_flush', 'target': 'horizon_protocol::connection::Connection::recv', 'kind': 'HEAD',
     'params': [('conn', '&mut Connection', '&mut *self')],
     'code': 'fn paper_hook_wire_flush(conn: &mut Connection) {\n    for body in horizon_core::paper_wire::wire::take_out() {\n        let _ = conn.send(&body);\n    }\n}\n'},
    {'id': 'wire_finish', 'target': 'horizon_protocol::connection::Connection::send', 'kind': 'HEAD', 'cancellable': True,
     'params': [('conn', '&mut Connection', '&mut *self'), ('body', '&[u8]', 'body')],
     'code': 'fn paper_hook_wire_finish(conn: &mut Connection, body: &[u8]) -> ::core::ops::ControlFlow<Result<()>> {\n    use crate::packets::{Empty, configuration::Clientbound};\n    use horizon_core::paper_wire::wire;\n    if !wire::tasks_pending() || body != Clientbound::FinishConfiguration(Empty).encode() {\n        return ::core::ops::ControlFlow::Continue(());\n    }\n    while wire::tasks_pending() {\n        match conn.recv() {\n            Ok((id, payload)) => wire::config_packet(id, &payload),\n            Err(e) => return ::core::ops::ControlFlow::Break(Err(e)),\n        }\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n'},
    {'id': 'chat_decorate', 'target': 'horizon_client::lan::hub::chat::Hub::handle_chat', 'kind': 'WRAP_OPERATION', 'selector': 'self.broadcast_chat', 'require': 1, 'ordinal': 0,
     'params': [('hub', '&mut Hub', '$receiver'), ('message', '&ChatMessage', '$arg0'), ('bound', '&play::ChatTypeBound', '$arg1'), ('line', 'String', '$arg2'), ('events', '&mut Vec<HostEvent>', '$arg3'),
                ('original', 'impl FnOnce(&mut Hub, &ChatMessage, &play::ChatTypeBound, String, &mut Vec<HostEvent>) -> ()', '$operation')],
     'code': 'fn paper_hook_chat_decorate(hub: &mut Hub, message: &ChatMessage, bound: &play::ChatTypeBound, line: String, events: &mut Vec<HostEvent>, original: impl FnOnce(&mut Hub, &ChatMessage, &play::ChatTypeBound, String, &mut Vec<HostEvent>) -> ()) {\n    match crate::paper_hub::decorate(message.sender, &message.content) {\n        Some(decorated) => {\n            let mut message = message.clone();\n            message.unsigned_content = Some(decorated);\n            original(hub, &message, bound, line, events);\n        }\n        None => original(hub, message, bound, line, events),\n    }\n}\n'},
    {'id': 'menu_load', 'target': 'horizon_client::game::lan_crafting::load_player', 'kind': 'HEAD',
     'params': [('g', '&Guest', 'g')],
     'code': 'fn paper_hook_menu_load(g: &Guest) {\n    horizon_client::paper_hub::set_menu_player(Some(g.uuid));\n}\n'},
    {'id': 'menu_store', 'target': 'horizon_client::game::lan_crafting::store_player', 'kind': 'HEAD',
     'params': [],
     'code': 'fn paper_hook_menu_store() {\n    horizon_client::paper_hub::set_menu_player(None);\n}\n'},
    {'id': 'reload_start', 'target': "horizon_client::game::Game<'_>::reload_world_tables", 'kind': 'HEAD',
     'params': [('game', GAME_T, '&mut *self')],
     'code': "fn paper_hook_reload_start(game: &mut crate::game::Game<'_>) {\n    crate::game::paper_events::on_reload(game);\n}\n"},
]


RAW += [
    {'id': 'block_placed', 'target': "horizon_client::game::native_hooks::Game<'_>::native_place_block", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack'), ('pos', '[i32; 3]', 'pos')],
     'code': "fn paper_hook_block_placed(game: &mut crate::game::Game<'_>, stack: &ItemStack, pos: [i32; 3]) {\n    crate::game::paper_bridge::on_block_placed(game, stack, pos);\n}\n"},
    {'id': 'flight_flags', 'target': 'horizon_client::lan::guest::Guest::ability_flags', 'kind': 'RETURN',
     'params': [('value', 'i8', '$value'), ('g', '&Guest', '&*self')],
     'code': 'fn paper_hook_flight_flags(value: i8, g: &Guest) -> i8 {\n    crate::paper_hub::ability_flags(g.uuid, value)\n}\n'},
    {'id': 'flight_wanted', 'target': 'horizon_client::lan::hub::Hub::process_inbox', 'kind': 'HEAD',
     'params': [('hub', '&mut Hub', '&mut *self'), ('i', 'usize', 'i')],
     'code': 'fn paper_hook_flight_wanted(hub: &mut Hub, i: usize) {\n    let g = &mut hub.guests[i];\n    if let Some(flying) = crate::paper_hub::flight_answer(g.uuid) {\n        g.flying = flying;\n    }\n    for (packet, _) in &g.inbox {\n        if let S::PlayerAbilities(p) = packet {\n            crate::paper_hub::flight_wanted(g.uuid, p.flags & 2 != 0);\n        }\n        crate::paper_hub::note_packet(g.uuid, packet);\n    }\n}\n'},
    {'id': 'item_consumed', 'target': "horizon_client::game::consume::Game<'_>::finish_consuming", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack')],
     'code': "fn paper_hook_item_consumed(game: &mut crate::game::Game<'_>, stack: &ItemStack) {\n    crate::game::paper_bridge::on_item_consumed(game, stack);\n}\n"},
    {'id': 'projectile_hit', 'target': "horizon_client::game::creatures::Game<'_>::projectile_hit", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('p', '&Projectile', 'p')],
     'code': "fn paper_hook_projectile_hit(game: &mut crate::game::Game<'_>, p: &Projectile) {\n    crate::game::paper_bridge::on_projectile_hit(game, p.id, p.kind.id(), [p.position.x, p.position.y, p.position.z], p.guest);\n}\n"},
    {'id': 'mob_bred', 'target': "horizon_client::game::creatures::Game<'_>::breed", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('parent', 'u32', 'parent'), ('partner', 'u32', 'partner')],
     'code': "fn paper_hook_mob_bred(game: &mut crate::game::Game<'_>, parent: u32, partner: u32) {\n    crate::game::paper_bridge::on_mob_bred(game, parent, partner);\n}\n"},
    {'id': 'rod_used', 'target': "horizon_client::game::fishing::Game<'_>::use_fishing_rod", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self')],
     'code': "fn paper_hook_rod_used(game: &mut crate::game::Game<'_>) {\n    crate::game::paper_bridge::on_rod_used(game);\n}\n"},
    {'id': 'container_opened', 'target': "horizon_client::game::lan_actions::Game<'_>::lan_open_container", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('conn', 'u32', 'conn'), ('pos', '(i32, i32, i32)', 'pos')],
     'code': "fn paper_hook_container_opened(game: &mut crate::game::Game<'_>, conn: u32, pos: (i32, i32, i32)) {\n    crate::game::paper_bridge::on_container_opened(game, conn, pos);\n}\n"},
    {'id': 'plugin_suggestions', 'target': 'horizon_client::lan::commands::CommandTree::packet_with', 'kind': 'RETURN',
     'params': [('value', 'Commands', '$value')],
     'code': 'fn paper_hook_plugin_suggestions(value: Commands) -> Commands {\n    crate::paper_hub::ask_server_for_plugins(value)\n}\n'},
    {'id': 'guest_suggestions', 'target': "horizon_client::game::lan_actions::Game<'_>::guest_suggestions", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('host', '&mut LanHost', '&mut *host'), ('conn', 'u32', 'conn'), ('payload', '&[u8]', 'payload')],
     'code': "fn paper_hook_guest_suggestions(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, payload: &[u8]) -> ::core::ops::ControlFlow<()> {\n    if crate::game::paper_bridge::on_suggestions(game, host, conn, payload) {\n        ::core::ops::ControlFlow::Break(())\n    } else {\n        ::core::ops::ControlFlow::Continue(())\n    }\n}\n"},
    {'id': 'sign_update', 'target': "horizon_client::game::sign_edit::Game<'_>::lan_sign_update", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('host', '&mut LanHost', '&mut *host'), ('conn', 'u32', 'conn'), ('p', '&play::SignUpdate', '&p')],
     'code': "fn paper_hook_sign_update(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, p: &play::SignUpdate) -> ::core::ops::ControlFlow<()> {\n    if crate::game::paper_bridge::on_sign_update(game, host, conn, p.pos, &p.lines, p.front) {\n        ::core::ops::ControlFlow::Break(())\n    } else {\n        ::core::ops::ControlFlow::Continue(())\n    }\n}\n"},
    {'id': 'tnt_ignited', 'target': "horizon_client::game::tnt::Game<'_>::ignite_tnt", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack')],
     'code': "fn paper_hook_tnt_ignited(game: &mut crate::game::Game<'_>, stack: &ItemStack) {\n    crate::game::paper_bridge::on_player_item(game, \"tnt_ignite\", stack);\n}\n"},
    {'id': 'bone_meal_used', 'target': "horizon_client::game::growth::Game<'_>::use_bone_meal", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack')],
     'code': "fn paper_hook_bone_meal_used(game: &mut crate::game::Game<'_>, stack: &ItemStack) {\n    crate::game::paper_bridge::on_player_item(game, \"bone_meal\", stack);\n}\n"},
    {'id': 'item_thrown', 'target': "horizon_client::game::creatures::Game<'_>::throw_item", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack')],
     'code': "fn paper_hook_item_thrown(game: &mut crate::game::Game<'_>, stack: &ItemStack) {\n    crate::game::paper_bridge::on_player_item(game, \"item_thrown\", stack);\n}\n"},
    {'id': 'crossbow_shot', 'target': "horizon_client::game::crossbow::Game<'_>::shoot_crossbow", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self')],
     'code': "fn paper_hook_crossbow_shot(game: &mut crate::game::Game<'_>) {\n    crate::game::paper_bridge::on_rod_like(game, \"bow_shot\");\n}\n"},
    {'id': 'adv_durability', 'target': "horizon_client::game::advancements::Game<'_>::trigger_item_durability", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack'), ('new_damage', 'u32', 'new_damage')],
     'code': "fn paper_hook_adv_durability(game: &mut crate::game::Game<'_>, stack: &ItemStack, new_damage: u32) {\n    if crate::game::paper_bridge::hears(\"adv_durability\") {\n        let payload = serde_json::json!({ \"stack\": crate::game::paper_query::stack_json(stack), \"damage\": new_damage });\n        crate::game::paper_bridge::on_trigger(game, \"adv_durability\", payload);\n    }\n}\n"},
    {'id': 'adv_enchanted', 'target': "horizon_client::game::advancements::Game<'_>::trigger_enchanted_item", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack'), ('levels', 'i32', 'levels')],
     'code': "fn paper_hook_adv_enchanted(game: &mut crate::game::Game<'_>, stack: &ItemStack, levels: i32) {\n    if crate::game::paper_bridge::hears(\"adv_enchanted\") {\n        let payload = serde_json::json!({ \"stack\": crate::game::paper_query::stack_json(stack), \"levels\": levels });\n        crate::game::paper_bridge::on_trigger(game, \"adv_enchanted\", payload);\n    }\n}\n"},
    {'id': 'adv_brewed', 'target': "horizon_client::game::advancements::Game<'_>::trigger_brewed_potion", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack')],
     'code': "fn paper_hook_adv_brewed(game: &mut crate::game::Game<'_>, stack: &ItemStack) {\n    if crate::game::paper_bridge::hears(\"adv_brewed\") {\n        let payload = serde_json::json!({ \"stack\": crate::game::paper_query::stack_json(stack) });\n        crate::game::paper_bridge::on_trigger(game, \"adv_brewed\", payload);\n    }\n}\n"},
    {'id': 'adv_trade', 'target': "horizon_client::game::advancements::Game<'_>::trigger_trade", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('villager', '&EntityView', 'villager'), ('stack', '&ItemStack', 'stack')],
     'code': "fn paper_hook_adv_trade(game: &mut crate::game::Game<'_>, villager: &EntityView, stack: &ItemStack) {\n    if crate::game::paper_bridge::hears(\"adv_trade\") {\n        let payload = serde_json::json!({ \"stack\": crate::game::paper_query::stack_json(stack), \"entity_kind\": villager.kind, \"at\": villager.position });\n        crate::game::paper_bridge::on_trigger(game, \"adv_trade\", payload);\n    }\n}\n"},
    {'id': 'adv_beacon', 'target': "horizon_client::game::advancements::Game<'_>::trigger_construct_beacon", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('levels', 'i32', 'levels')],
     'code': "fn paper_hook_adv_beacon(game: &mut crate::game::Game<'_>, levels: i32) {\n    if crate::game::paper_bridge::hears(\"adv_beacon\") {\n        let payload = serde_json::json!({ \"levels\": levels });\n        crate::game::paper_bridge::on_trigger(game, \"adv_beacon\", payload);\n    }\n}\n"},
    {'id': 'adv_target', 'target': "horizon_client::game::advancements::Game<'_>::trigger_target_hit", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('projectile', '&EntityView', 'projectile'), ('strength', 'i32', 'strength')],
     'code': "fn paper_hook_adv_target(game: &mut crate::game::Game<'_>, projectile: &EntityView, strength: i32) {\n    if crate::game::paper_bridge::hears(\"adv_target\") {\n        let payload = serde_json::json!({ \"strength\": strength, \"at\": projectile.position });\n        crate::game::paper_bridge::on_trigger(game, \"adv_target\", payload);\n    }\n}\n"},
    {'id': 'adv_lightning', 'target': "horizon_client::game::advancements::Game<'_>::trigger_lightning_strike", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('bolt', '&EntityView', 'bolt')],
     'code': "fn paper_hook_adv_lightning(game: &mut crate::game::Game<'_>, bolt: &EntityView) {\n    if crate::game::paper_bridge::hears(\"adv_lightning\") {\n        let payload = serde_json::json!({ \"at\": bolt.position });\n        crate::game::paper_bridge::on_trigger(game, \"adv_lightning\", payload);\n    }\n}\n"},
    {'id': 'adv_entity', 'target': "horizon_client::game::advancements::Game<'_>::trigger_entity", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('name', '&str', 'name'), ('target', '&EntityView', 'target')],
     'code': "fn paper_hook_adv_entity(game: &mut crate::game::Game<'_>, name: &str, target: &EntityView) {\n    if crate::game::paper_bridge::hears(\"adv_entity\") {\n        let payload = serde_json::json!({ \"kind\": name, \"entity_kind\": target.kind, \"at\": target.position });\n        crate::game::paper_bridge::on_trigger(game, \"adv_entity\", payload);\n    }\n}\n"},
    {'id': 'adv_player', 'target': "horizon_client::game::advancements::Game<'_>::trigger_player", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('name', '&str', 'name')],
     'code': "fn paper_hook_adv_player(game: &mut crate::game::Game<'_>, name: &str) {\n    if crate::game::paper_bridge::hears(\"adv_player\") {\n        let payload = serde_json::json!({ \"kind\": name });\n        crate::game::paper_bridge::on_trigger(game, \"adv_player\", payload);\n    }\n}\n"},
    {'id': 'adv_item_entity', 'target': "horizon_client::game::advancements::Game<'_>::trigger_item_entity", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('name', '&str', 'name'), ('stack', '&ItemStack', 'stack'), ('target', '&EntityView', 'target')],
     'code': "fn paper_hook_adv_item_entity(game: &mut crate::game::Game<'_>, name: &str, stack: &ItemStack, target: &EntityView) {\n    if crate::game::paper_bridge::hears(\"adv_item_entity\") {\n        let payload = serde_json::json!({ \"kind\": name, \"stack\": crate::game::paper_query::stack_json(stack), \"entity_kind\": target.kind, \"at\": target.position });\n        crate::game::paper_bridge::on_trigger(game, \"adv_item_entity\", payload);\n    }\n}\n"},
    {'id': 'guest_action', 'target': "horizon_client::game::lan_actions::Game<'_>::lan_guest_packet", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('host', '&mut LanHost', '&mut *host'), ('conn', 'u32', 'conn'), ('packet', '&S', '&packet')],
     'code': "fn paper_hook_guest_action(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, packet: &S) -> ::core::ops::ControlFlow<()> {\n    if let S::PlayerAction(a) = packet\n        && matches!(a.action, 3 | 4 | 6)\n        && crate::game::paper_bridge::on_player_action(game, host, conn, a.action)\n    {\n        return ::core::ops::ControlFlow::Break(());\n    }\n    ::core::ops::ControlFlow::Continue(())\n}\n"},
    {'id': 'item_dropped', 'target': "horizon_client::game::native_hooks::Game<'_>::native_item_dropped", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('player', 'u128', 'player'), ('item', '&str', 'item'), ('count', 'u32', 'count')],
     'code': "fn paper_hook_item_dropped(game: &mut crate::game::Game<'_>, player: u128, item: &str, count: u32) {\n    crate::game::paper_bridge::on_item_dropped(game, player, item, count);\n}\n"},
    {'id': 'item_picked_up', 'target': "horizon_client::game::native_hooks::Game<'_>::native_item_picked_up", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('player', 'u128', 'player'), ('item', '&str', 'item'), ('count', 'u32', 'count'), ('canceled', '&mut bool', '&mut *canceled')],
     'code': "fn paper_hook_item_picked_up(game: &mut crate::game::Game<'_>, player: u128, item: &str, count: u32, canceled: &mut bool) {\n    crate::game::paper_bridge::on_item_picked_up(game, player, item, count, canceled);\n}\n"},
    {'id': 'fill_bucket', 'target': "horizon_client::game::native_hooks::Game<'_>::native_fill_bucket", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('stack', '&ItemStack', 'stack'), ('pos', '[i32; 3]', 'pos'), ('canceled', '&mut bool', '&mut *canceled')],
     'code': "fn paper_hook_fill_bucket(game: &mut crate::game::Game<'_>, stack: &ItemStack, pos: [i32; 3], canceled: &mut bool) {\n    crate::game::paper_bridge::on_fill_bucket(game, stack, pos, canceled);\n}\n"},
    {'id': 'explosion_detonate', 'target': "horizon_client::game::native_hooks::Game<'_>::native_explosion_detonate", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('dimension', '&str', 'dimension'), ('center', '[f64; 3]', 'center'), ('blocks', '&mut Vec<[i32; 3]>', '&mut *blocks')],
     'code': "fn paper_hook_explosion_detonate(game: &mut crate::game::Game<'_>, dimension: &str, center: [f64; 3], blocks: &mut Vec<[i32; 3]>) {\n    crate::game::paper_bridge::on_explosion(game, dimension, center, blocks);\n}\n"},
    {'id': 'piston_move', 'target': "horizon_client::game::native_hooks::GameLevel<'_>::native_piston_move", 'kind': 'HEAD',
     'params': [('piston', 'horizon_core::position::BlockPos', 'piston'), ('direction', '[i32; 3]', 'direction'), ('push', '&[horizon_core::position::BlockPos]', 'push'), ('canceled', '&mut bool', '&mut *canceled')],
     'code': "fn paper_hook_piston_move(piston: horizon_core::position::BlockPos, direction: [i32; 3], push: &[horizon_core::position::BlockPos], canceled: &mut bool) {\n    crate::game::paper_bridge::on_piston(piston, direction, push, canceled);\n}\n"},
    {'id': 'menu_clicked', 'target': "horizon_client::game::native_hooks::Game<'_>::native_menu_clicked", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('uuid', 'u128', 'uuid'), ('window', 'i32', 'window'), ('slot', 'i32', 'slot'), ('button', 'i32', 'button'), ('mode', 'i32', 'mode')],
     'code': "fn paper_hook_menu_clicked(game: &mut crate::game::Game<'_>, uuid: u128, window: i32, slot: i32, button: i32, mode: i32) {\n    crate::game::paper_bridge::on_menu_clicked(game, uuid, window, slot, button, mode);\n}\n"},
    {'id': 'menu_closed', 'target': "horizon_client::game::native_hooks::Game<'_>::native_menu_closed", 'kind': 'HEAD',
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('uuid', 'u128', 'uuid'), ('window', 'i32', 'window')],
     'code': "fn paper_hook_menu_closed(game: &mut crate::game::Game<'_>, uuid: u128, window: i32) {\n    crate::game::paper_bridge::on_menu_closed(game, uuid, window);\n}\n"},
    {'id': 'guest_command', 'target': "horizon_client::game::lan_actions::Game<'_>::guest_command", 'kind': 'HEAD', 'cancellable': True,
     'params': [('game', "&mut crate::game::Game<'_>", '&mut *self'), ('host', '&mut LanHost', '&mut *host'), ('conn', 'u32', 'conn'), ('command', '&str', 'command')],
     'code': "fn paper_hook_guest_command(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, command: &str) -> ::core::ops::ControlFlow<()> {\n    let who = host.hub.guests.iter().find(|g| g.conn == conn).map(|g| (g.uuid, g.name.clone()));\n    match who {\n        Some((uuid, name)) if crate::game::paper_bridge::on_command_preprocess(game, uuid, &name, command) => ::core::ops::ControlFlow::Break(()),\n        _ => ::core::ops::ControlFlow::Continue(()),\n    }\n}\n"},
    {'id': 'mob_died', 'target': "horizon_client::game::creatures::Game<'_>::mob_died", 'kind': 'RETURN',
     'params': [('value', '()', '$value'), ('game', GAME_T, '&mut *self'), ('m', '&Mob', 'm')],
     'code': "fn paper_hook_mob_died(value: (), game: &mut crate::game::Game<'_>, m: &Mob) -> () {\n    crate::game::paper_events::on_mob_died(game, m.id, m.kind.id.as_ref());\n    value\n}\n"},
    {'id': 'effects_tick_end', 'target': "horizon_client::game::potions::Game<'_>::tick_mob_effects", 'kind': 'RETURN',
     'params': [('value', '()', '$value'), ('game', GAME_T, '&mut *self')],
     'code': "fn paper_hook_effects_tick_end(value: (), game: &mut crate::game::Game<'_>) -> () {\n    crate::game::paper_events::on_effects_ticked(game);\n    value\n}\n"},
]

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import event_hooks  # noqa: E402

RAW += event_hooks.raws()
# `//set`: a command word with its own slash (WorldEdit), after the preprocess hook has seen it.
RAW += [
    {'id': 'slash_usage', 'target': 'horizon_client::game::native_server::usage_nodes', 'kind': 'HEAD', 'cancellable': True, 'params': [('usage', '&str', 'usage')], 'code': "fn paper_hook_slash_usage(usage: &str) -> ::core::ops::ControlFlow<Result<Vec<(String, Value, bool)>, String>> {\n    // A command word with a slash of its own (`/set`, WorldEdit's `//set` as typed): the rest read as any usage,\n    // the slash put back on the word, as Paper's command tree has it.\n    let Some(rest) = usage.strip_prefix('/') else { return ::core::ops::ControlFlow::Continue(()) };\n    if rest.is_empty() || rest.starts_with(['/', ' ']) {\n        return ::core::ops::ControlFlow::Continue(());\n    }\n    ::core::ops::ControlFlow::Break(usage_nodes(rest).map(|mut nodes| {\n        if let Some(first) = nodes.first_mut() {\n            first.0.insert(0, '/');\n        }\n        nodes\n    }))\n}\n"},
    {'id': 'slash_command', 'target': "horizon_client::game::lan_actions::Game<'_>::guest_command", 'kind': 'HEAD', 'cancellable': True, 'params': [('host', '&mut LanHost', '&mut *host'), ('conn', 'u32', 'conn'), ('command', '&str', 'command')], 'code': 'fn paper_hook_slash_command(host: &mut LanHost, conn: u32, command: &str) -> ::core::ops::ControlFlow<()> {\n    // `//set` arrives as `/set`: the mod\'s that took `/set` (WorldEdit\'s), which trimming every slash would\n    // turn into `/set`\'s.\n    if !command.starts_with(\'/\') || command.starts_with("//") {\n        return ::core::ops::ControlFlow::Continue(());\n    }\n    let Some(level) = Game::native_command_level(host, command) else { return ::core::ops::ControlFlow::Continue(()) };\n    let Some(permission) = host.hub.guest(conn).map(|g| g.permission) else { return ::core::ops::ControlFlow::Continue(()) };\n    if permission < level {\n        return ::core::ops::ControlFlow::Continue(());\n    }\n    let integrated = host.hub.config.integrated;\n    host.commands.push(GuestCommand { conn, command: command.to_owned(), permission, integrated, quiet: false });\n    ::core::ops::ControlFlow::Break(())\n}\n'},
]

HOOKS = [
    ('server_about_to_start', [GAME, by_value('dedicated', 'bool')], 'on_server_starting(game, dedicated)'),
    ('server_stopping', [GAME], 'on_server_stopping(game)'),
    ('world_lifecycle', [GAME, by_value('phase', WORLD_T)], 'on_world_lifecycle(game, phase)'),
    ('server_tick', [GAME, by_value('phase', PHASE_T)], 'on_server_tick(game, phase)'),
    ('level_tick_pre', [GAME, by_value('dimension', '&str')], 'on_level_tick(game, dimension, false)'),
    ('level_tick_post', [GAME, by_value('dimension', '&str')], 'on_level_tick(game, dimension, true)'),
    ('player_logged_in', [GAME, by_value('player', 'u128'), by_value('name', '&str')], 'on_joined(game, player, name)'),
    ('player_logged_out', [GAME, by_value('player', 'u128'), by_value('name', '&str')], 'on_left(game, player, name)'),
    ('destroy_block', [GAME, by_value('pos', '[i32; 3]'), by_value('state', STATE_T), by_mut('canceled', 'bool')],
     'on_destroy_block(game, pos, state, canceled)'),
    ('left_click_block', [GAME, by_value('pos', '[i32; 3]'), by_value('face', '[i32; 3]'), by_mut('canceled', 'bool')],
     'on_left_click_block(game, pos, face, canceled)'),
    ('right_click_block', [GAME, by_value('pos', '[i32; 3]'), by_value('face', '[i32; 3]'), by_mut('canceled', 'bool')],
     'on_right_click_block(game, pos, face, canceled)'),
    ('use_item', [GAME, by_value('stack', f'&{STACK_T}'), by_mut('claimed', 'bool')], 'on_use_item(game, stack, claimed)'),
    ('use_item_on', [GAME, by_value('stack', f'&{STACK_T}'), by_value('pos', '[i32; 3]'), by_value('face', '[i32; 3]'), by_mut('claimed', 'bool')],
     'on_use_item_on(game, stack, pos, face, claimed)'),
    ('block_changed', [by_value('dimension', 'crate::game::LevelDimension'), by_value('pos', POS_T), by_value('old', STATE_T), by_value('new', STATE_T)],
     'on_block_changed(dimension, pos, old, new)'),
    ('block_spread',
     [by_value('dimension', 'crate::game::LevelDimension'), by_value('kind', '&str'), by_value('from', POS_T), by_value('to', POS_T), by_mut('canceled', 'bool')],
     'on_block_spread(dimension, kind, [from.x, from.y, from.z], [to.x, to.y, to.z], canceled)'),
    ('interact_entity', [GAME, by_value('entity', 'i32'), by_value('kind', '&str'), by_value('stack', f'Option<&{STACK_T}>'), by_mut('claimed', 'bool')],
     'on_interact_entity(game, entity, kind, stack, claimed)'),
    ('attack_entity', [GAME, by_value('entity', 'i32'), by_value('kind', '&str'), by_value('pos', '[f64; 3]'), by_mut('canceled', 'bool')],
     'on_attack_entity(game, entity, kind, pos, canceled)'),
    ('living_incoming_damage',
     [by_value('entity', 'i32'), by_value('kind', '&str'), by_value('damage_type', '&str'), by_value('causing', 'i32'), by_value('direct', 'i32'),
      by_mut('amount', 'f32'), by_mut('canceled', 'bool')],
     'on_incoming_damage(entity, kind, damage_type, causing, direct, amount, canceled)'),
    ('player_died', [GAME, by_value('player', 'u128'), by_value('cause', 'Option<&str>')], 'on_player_died(game, player, cause)'),
    ('mob_killed', [GAME, by_value('player', 'u128'), by_value('entity', 'u32'), by_value('kind', '&str')], 'on_mob_killed(game, player, entity, kind)'),
    ('entity_join_level',
     [GAME, by_value('dimension', '&str'), by_value('entity', 'u32'), by_value('kind', '&str'), by_value('pos', '[f64; 3]'), by_value('loaded_from_disk', 'bool'),
      by_mut('canceled', 'bool')],
     'on_entity_join(game, dimension, entity, kind, pos, loaded_from_disk, canceled)'),
    ('entity_leave_level', [GAME, by_value('dimension', '&str'), by_value('entity', 'u32'), by_value('kind', '&str')],
     'on_entity_leave(game, dimension, entity, kind)'),
    ('chunk_load', [GAME, by_value('dimension', '&str'), by_value('chunk', '[i32; 2]'), by_value('new', 'bool')], 'on_chunk_load(game, dimension, chunk, new)'),
    ('chunk_unload', [GAME, by_value('dimension', '&str'), by_value('chunk', '[i32; 2]')], 'on_chunk_unload(game, dimension, chunk)'),
    ('loot_drops', [GAME, by_value('context', f'&{DROPS_T}'), by_mut('drops', f'Vec<{STACK_T}>')], 'on_loot_drops(game, context, drops)'),
    ('on_equip', [GAME, by_value('slot', '&str'), by_value('new', f'Option<&{STACK_T}>')], 'on_equip(game, slot, new)'),
    ('register_commands', [GAME, by_mut('commands', f'Vec<{COMMAND_T}>')], 'on_register_commands(game, commands)'),
    ('perform_command', [GAME, by_value('player', f'&{PLAYER_T}'), by_value('command', '&str'), by_mut('ran', 'bool')],
     'on_perform_command(game, player, command, ran)'),
    ('net_from_client', [GAME, by_value('player', 'u128'), by_value('channel', '&str'), by_value('data', '&[u8]')],
     'on_net_from_client(game, player, channel, data)'),
]


def toml_string(value):
    return json.dumps(value)


def main():
    hooks_json, commit = sys.argv[1:3]
    catalog = {h['name']: h for h in json.load(open(hooks_json, encoding='utf-8'))['hooks']}
    missing = [h[0] for h in HOOKS if h[0] not in catalog]
    if missing:
        sys.exit(f'hooks the pinned source lacks: {missing}')

    for old in BUNDLE.glob('hook_*.rs'):
        old.unlink()
    lines = ['# Written by tools/gen_bundle.py: change the table there, not this file.', '[mod]', 'id = "paper_api"', 'version = "0.1.0"',
             'priority = 100', '', '[compatibility]', f'horizon_revision = "{commit}"', '']
    for name, source, file, visibility in (
        ('paper_wire', 'paper_wire.rs', 'crates/horizon-core/src/lib.rs', 'pub'),
        ('paper_plugins', 'paper_plugins.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('video_recorder', 'video_recorder.rs', 'crates/horizon-client-render/src/lib.rs', 'pub'),
        ('paper_hub', 'paper_hub.rs', 'crates/horizon-client/src/lib.rs', 'pub'),
        ('paper_net', 'paper_net.rs', 'crates/horizon-client/src/lib.rs', 'pub'),
        ('paper_events', 'paper_events.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_bridge', 'paper_bridge.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_query', 'paper_query.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_players', 'paper_players.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_levels', 'paper_levels.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_lan', 'paper_lan.rs', 'crates/horizon-client/src/lan/mod.rs', 'pub'),
        ('paper_fam_ev_entity', 'paper_fam_ev_entity.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_fam_ev_player', 'paper_fam_ev_player.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_fam_ev_world', 'paper_fam_ev_world.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_fam_entity', 'paper_fam_entity.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_fam_core', 'paper_fam_core.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_fam_items', 'paper_fam_items.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_fam_paper', 'paper_fam_paper.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_fam_ess', 'paper_fam_ess.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_entity_events', 'paper_entity_events.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_worlds', 'paper_worlds.rs', 'crates/horizon-client/src/game.rs', 'pub(crate)'),
        ('paper_lan_hub', 'paper_lan_hub.rs', 'crates/horizon-client/src/lan/hub.rs', 'pub'),
        ('paper_entities', 'paper_entities.rs', 'crates/horizon-client/src/game/commands.rs', 'pub(crate)'),
    ):
        lines += ['[[module]]', f'id = "{name}"', f'file = "{file}"', f'name = "{name}"', f'source = "{source}"', f'visibility = "{visibility}"', '']
    lines += ['[[crate]]', 'id = "runtime"', 'path = "runtime"', '']
    for hook, bindings, body in HOOKS:
        params = ', '.join(f'{name}: {ty}' for name, ty, _ in bindings)
        call = f'crate::game::paper_events::{body}'
        helper = f'// Written by tools/gen_bundle.py.\nfn paper_hook_{hook}({params}) {{\n    {call};\n}}\n'
        (BUNDLE / f'hook_{hook}.rs').write_text(helper, encoding='utf-8', newline='\n')
        lines += ['[[injection]]', f'id = "{hook}"', f'hook = "{hook}"', f"hook_version = {catalog[hook]['version']}", 'kind = "HEAD"', f'helper = "hook_{hook}.rs"', 'bindings = [']
        lines += [f'  {{ name = {toml_string(n)}, ty = {toml_string(t)}, value = {toml_string(v)} }},' for n, t, v in bindings]
        lines += [']', '']
    for raw in RAW:
        (BUNDLE / f"hook_{raw['id']}.rs").write_text('// Written by tools/gen_bundle.py.\n' + raw['code'], encoding='utf-8', newline='\n')
        lines += ['[[injection]]', f"id = \"{raw['id']}\"", f"target = {toml_string(raw['target'])}", f"kind = \"{raw['kind']}\"", f"helper = \"hook_{raw['id']}.rs\""]
        for key in ('selector',):
            if key in raw:
                lines.append(f'{key} = {toml_string(raw[key])}')
        if 'variable' in raw:
            lines.append(f"variable = {raw['variable']}")
        for key in ('require', 'ordinal'):
            if key in raw:
                lines.append(f'{key} = {raw[key]}')
        if raw.get('cancellable'):
            lines.append('cancellable = true')
        lines.append('bindings = [')
        lines += [f'  {{ name = {toml_string(n)}, ty = {toml_string(t)}, value = {toml_string(v)} }},' for n, t, v in raw['params']]
        lines += [']', '']
    (BUNDLE / 'horizon-mixin.toml').write_text('\n'.join(lines), encoding='utf-8', newline='\n')
    print(f'{len(HOOKS)} hooks and {len(RAW)} direct injections wired, pinned to {commit[:9]}')


if __name__ == '__main__':
    main()
