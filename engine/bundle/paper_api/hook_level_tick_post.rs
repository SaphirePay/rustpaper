// Written by tools/gen_bundle.py.
fn paper_hook_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    crate::game::paper_events::on_level_tick(game, dimension, true);
}
