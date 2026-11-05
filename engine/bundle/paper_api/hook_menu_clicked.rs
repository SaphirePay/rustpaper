// Written by tools/gen_bundle.py.
fn paper_hook_menu_clicked(game: &mut crate::game::Game<'_>, uuid: u128, window: i32, slot: i32, button: i32, mode: i32) {
    crate::game::paper_bridge::on_menu_clicked(game, uuid, window, slot, button, mode);
}
