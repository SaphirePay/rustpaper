// Written by tools/gen_bundle.py.
fn paper_hook_block_placed(game: &mut crate::game::Game<'_>, stack: &ItemStack, pos: [i32; 3]) {
    crate::game::paper_bridge::on_block_placed(game, stack, pos);
}
