// Written by tools/gen_bundle.py.
fn paper_hook_server_about_to_start(game: &mut crate::game::Game<'_>, dedicated: bool) {
    crate::game::paper_events::on_server_starting(game, dedicated);
}
