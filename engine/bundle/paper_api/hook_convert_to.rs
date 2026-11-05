// Written by tools/gen_bundle.py.
fn paper_hook_convert_to(game: &mut crate::game::Game<'_>, from: usize) {
    crate::game::paper_events::on_converted(game, from);
}
