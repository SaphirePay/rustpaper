// Written by tools/gen_bundle.py.
fn paper_hook_ev_compost(game: &mut crate::game::Game<'_>, pos: [i32; 3], held: Option<&ItemStack>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("compost") {
        let payload = serde_json::json!({ "pos": pos, "stack": held.map(crate::game::paper_query::stack_json) });
        if crate::game::paper_bridge::on_refusable(game, "compost", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
