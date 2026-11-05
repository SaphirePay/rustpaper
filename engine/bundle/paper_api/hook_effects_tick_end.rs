// Written by tools/gen_bundle.py.
fn paper_hook_effects_tick_end(value: (), game: &mut crate::game::Game<'_>) -> () {
    crate::game::paper_events::on_effects_ticked(game);
    value
}
