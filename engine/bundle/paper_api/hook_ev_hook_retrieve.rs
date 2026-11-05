// Written by tools/gen_bundle.py.
fn paper_hook_ev_hook_retrieve(game: &mut crate::game::Game<'_>, rod: &ItemStack) -> ::core::ops::ControlFlow<u32> {
    if crate::game::paper_bridge::hears("hook_retrieve") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(rod) });
        if crate::game::paper_bridge::on_refusable(game, "hook_retrieve", payload) {
            return ::core::ops::ControlFlow::Break(0);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
