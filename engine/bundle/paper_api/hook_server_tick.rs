// Written by tools/gen_bundle.py.
fn paper_hook_server_tick(game: &mut crate::game::Game<'_>, phase: crate::game::native_lifecycle::ServerTickPhase) {
    crate::game::paper_events::on_server_tick(game, phase);
}
