// Written by tools/gen_bundle.py.
fn paper_hook_perform_command(game: &mut crate::game::Game<'_>, player: &crate::game::native_server::NativeServerPlayer, command: &str, ran: &mut bool) {
    crate::game::paper_events::on_perform_command(game, player, command, ran);
}
