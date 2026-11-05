// Written by tools/gen_bundle.py.
fn paper_hook_sleep_monsters(value: bool, game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> bool {
    crate::game::paper_events::on_nearby_monsters(game, pos, value)
}
