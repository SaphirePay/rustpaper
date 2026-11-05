// Written by tools/gen_bundle.py.
fn paper_hook_ev_panda_use(game: &mut crate::game::Game<'_>, index: usize, item: &str) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("panda_use") {
        let payload = serde_json::json!({ "index": index, "item": item });
        if crate::game::paper_bridge::on_refusable(game, "panda_use", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
