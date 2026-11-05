// Written by tools/gen_bundle.py.
fn paper_hook_item_picked_up(game: &mut crate::game::Game<'_>, player: u128, item: &str, count: u32, canceled: &mut bool) {
    crate::game::paper_bridge::on_item_picked_up(game, player, item, count, canceled);
}
