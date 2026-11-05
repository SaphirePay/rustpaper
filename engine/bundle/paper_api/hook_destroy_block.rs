// Written by tools/gen_bundle.py.
fn paper_hook_destroy_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], state: horizon_world::registry::BlockStateId, canceled: &mut bool) {
    crate::game::paper_events::on_destroy_block(game, pos, state, canceled);
}
