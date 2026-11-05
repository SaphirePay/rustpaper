// Written by tools/gen_bundle.py.
fn paper_hook_adv_durability(game: &mut crate::game::Game<'_>, stack: &ItemStack, new_damage: u32) {
    if crate::game::paper_bridge::hears("adv_durability") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(stack), "damage": new_damage });
        crate::game::paper_bridge::on_trigger(game, "adv_durability", payload);
    }
}
