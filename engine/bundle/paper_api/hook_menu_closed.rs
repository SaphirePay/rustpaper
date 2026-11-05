// Written by tools/gen_bundle.py.
fn paper_hook_menu_closed(game: &mut crate::game::Game<'_>, uuid: u128, window: i32) {
    crate::game::paper_bridge::on_menu_closed(game, uuid, window);
}
