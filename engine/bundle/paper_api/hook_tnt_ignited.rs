// Written by tools/gen_bundle.py.
fn paper_hook_tnt_ignited(game: &mut crate::game::Game<'_>, stack: &ItemStack) {
    crate::game::paper_bridge::on_player_item(game, "tnt_ignite", stack);
}
