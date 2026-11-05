// Written by tools/gen_bundle.py.
fn paper_hook_item_thrown(game: &mut crate::game::Game<'_>, stack: &ItemStack) {
    crate::game::paper_bridge::on_player_item(game, "item_thrown", stack);
}
