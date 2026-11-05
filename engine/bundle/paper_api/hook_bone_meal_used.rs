// Written by tools/gen_bundle.py.
fn paper_hook_bone_meal_used(game: &mut crate::game::Game<'_>, stack: &ItemStack) {
    crate::game::paper_bridge::on_player_item(game, "bone_meal", stack);
}
