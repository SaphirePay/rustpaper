// Written by tools/gen_bundle.py.
fn paper_hook_ev_trident_use(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("trident_use") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(stack) });
        if crate::game::paper_bridge::on_refusable(game, "trident_use", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
