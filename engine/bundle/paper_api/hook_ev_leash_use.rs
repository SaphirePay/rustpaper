// Written by tools/gen_bundle.py.
fn paper_hook_ev_leash_use(game: &mut crate::game::Game<'_>, index: usize) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("leash_use") {
        let payload = serde_json::json!({ "index": index });
        if crate::game::paper_bridge::on_refusable(game, "leash_use", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
