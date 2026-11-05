// Written by tools/gen_bundle.py.
fn paper_hook_mob_tick_end(game: &mut crate::game::Game<'_>) {
    crate::game::paper_bridge::on_block_tick_end(game);
}
