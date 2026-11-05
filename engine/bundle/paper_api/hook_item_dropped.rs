// Written by tools/gen_bundle.py.
fn paper_hook_item_dropped(game: &mut crate::game::Game<'_>, player: u128, item: &str, count: u32) {
    crate::game::paper_bridge::on_item_dropped(game, player, item, count);
}
