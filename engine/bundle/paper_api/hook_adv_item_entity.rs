// Written by tools/gen_bundle.py.
fn paper_hook_adv_item_entity(game: &mut crate::game::Game<'_>, name: &str, stack: &ItemStack, target: &EntityView) {
    if crate::game::paper_bridge::hears("adv_item_entity") {
        let payload = serde_json::json!({ "kind": name, "stack": crate::game::paper_query::stack_json(stack), "entity_kind": target.kind, "at": target.position });
        crate::game::paper_bridge::on_trigger(game, "adv_item_entity", payload);
    }
}
