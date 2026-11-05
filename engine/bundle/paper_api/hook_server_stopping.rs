// Written by tools/gen_bundle.py.
fn paper_hook_server_stopping(game: &mut crate::game::Game<'_>) {
    crate::game::paper_events::on_server_stopping(game);
}
