// Written by tools/gen_bundle.py.
fn paper_hook_ev_hopper_add(game: &mut crate::game::Game<'_>, from_hopper: Option<Pos>, stack: &ItemStack) -> ::core::ops::ControlFlow<Option<ItemStack>> {
    if crate::game::paper_bridge::hears("hopper_add") {
        let payload = serde_json::json!({ "kind": if from_hopper.is_some() { "hopper" } else { "item" }, "pos": from_hopper.map(|pos| [pos.0, pos.1, pos.2]), "stack": crate::game::paper_query::stack_json(stack) });
        if crate::game::paper_bridge::on_refusable(game, "hopper_add", payload) {
            return ::core::ops::ControlFlow::Break(Some(stack.clone()));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
