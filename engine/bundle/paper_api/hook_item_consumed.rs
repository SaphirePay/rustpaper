// Written by tools/gen_bundle.py.
fn paper_hook_item_consumed(game: &mut crate::game::Game<'_>, stack: &ItemStack) {
    crate::game::paper_bridge::on_item_consumed(game, stack);
}
