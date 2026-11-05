// Written by tools/gen_bundle.py.
fn paper_hook_use_item(game: &mut crate::game::Game<'_>, stack: &horizon_client::items::ItemStack, claimed: &mut bool) {
    crate::game::paper_events::on_use_item(game, stack, claimed);
}
