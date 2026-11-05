// Written by tools/gen_bundle.py.
fn paper_hook_ev_boat_exit(game: &mut crate::game::Game<'_>, i: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("boat_exit") {
        let payload = serde_json::json!({ "index": i });
        if crate::game::paper_bridge::on_refusable(game, "boat_exit", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
