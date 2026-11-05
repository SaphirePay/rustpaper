// Written by tools/gen_bundle.py.
fn paper_hook_left_click_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    crate::game::paper_events::on_left_click_block(game, pos, face, canceled);
}
