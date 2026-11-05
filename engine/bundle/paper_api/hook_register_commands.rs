// Written by tools/gen_bundle.py.
fn paper_hook_register_commands(game: &mut crate::game::Game<'_>, commands: &mut Vec<crate::game::native_server::NativeCommand>) {
    crate::game::paper_events::on_register_commands(game, commands);
}
