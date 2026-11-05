// Written by tools/gen_bundle.py.
fn paper_hook_adv_trade(game: &mut crate::game::Game<'_>, villager: &EntityView, stack: &ItemStack) {
    if crate::game::paper_bridge::hears("adv_trade") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(stack), "entity_kind": villager.kind, "at": villager.position });
        crate::game::paper_bridge::on_trigger(game, "adv_trade", payload);
    }
}
