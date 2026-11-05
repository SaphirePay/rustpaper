// Written by tools/gen_bundle.py.
fn paper_hook_ev_potion_use(game: &mut crate::game::Game<'_>, stack: &ItemStack) {
    if crate::game::paper_bridge::hears("potion_use") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(stack) });
        crate::game::paper_bridge::on_trigger(game, "potion_use", payload);
    }
}
