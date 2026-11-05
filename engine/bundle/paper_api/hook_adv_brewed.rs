// Written by tools/gen_bundle.py.
fn paper_hook_adv_brewed(game: &mut crate::game::Game<'_>, stack: &ItemStack) {
    if crate::game::paper_bridge::hears("adv_brewed") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(stack) });
        crate::game::paper_bridge::on_trigger(game, "adv_brewed", payload);
    }
}
