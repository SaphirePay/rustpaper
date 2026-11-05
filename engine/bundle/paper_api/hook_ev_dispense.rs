// Written by tools/gen_bundle.py.
fn paper_hook_ev_dispense(game: &mut crate::game::Game<'_>, pos: Pos, stack: &ItemStack) {
    if crate::game::paper_bridge::hears("dispense") {
        let payload = serde_json::json!({ "pos": [pos.0, pos.1, pos.2], "stack": crate::game::paper_query::stack_json(stack) });
        crate::game::paper_bridge::on_trigger(game, "dispense", payload);
    }
}
