// Written by tools/gen_bundle.py.
fn paper_hook_ev_rocket_boost(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("rocket_boost") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(stack) });
        if crate::game::paper_bridge::on_refusable(game, "rocket_boost", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
