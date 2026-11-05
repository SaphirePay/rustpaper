// Written by tools/gen_bundle.py.
fn paper_hook_use_item_on(game: &mut crate::game::Game<'_>, stack: &horizon_client::items::ItemStack, pos: [i32; 3], face: [i32; 3], claimed: &mut bool) {
    crate::game::paper_events::on_use_item_on(game, stack, pos, face, claimed);
}
