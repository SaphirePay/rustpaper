// Written by tools/gen_bundle.py.
fn paper_hook_ev_boat_enter(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("boat_enter") {
        let payload = serde_json::json!({  });
        if crate::game::paper_bridge::on_refusable(game, "boat_enter", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
