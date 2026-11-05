// Written by tools/gen_bundle.py.
fn paper_hook_world_lifecycle(game: &mut crate::game::Game<'_>, phase: crate::game::native_lifecycle::WorldPhase) {
    crate::game::paper_events::on_world_lifecycle(game, phase);
}
