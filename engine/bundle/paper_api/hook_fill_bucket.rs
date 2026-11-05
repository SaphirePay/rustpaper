// Written by tools/gen_bundle.py.
fn paper_hook_fill_bucket(game: &mut crate::game::Game<'_>, stack: &ItemStack, pos: [i32; 3], canceled: &mut bool) {
    crate::game::paper_bridge::on_fill_bucket(game, stack, pos, canceled);
}
