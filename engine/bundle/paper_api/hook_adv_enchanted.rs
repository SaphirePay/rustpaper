// Written by tools/gen_bundle.py.
fn paper_hook_adv_enchanted(game: &mut crate::game::Game<'_>, stack: &ItemStack, levels: i32) {
    if crate::game::paper_bridge::hears("adv_enchanted") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(stack), "levels": levels });
        crate::game::paper_bridge::on_trigger(game, "adv_enchanted", payload);
    }
}
