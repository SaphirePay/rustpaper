// Written by tools/gen_bundle.py.
fn paper_hook_ev_frame_use(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("frame_use") {
        let payload = serde_json::json!({  });
        if crate::game::paper_bridge::on_refusable(game, "frame_use", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
