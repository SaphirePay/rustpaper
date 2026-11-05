// Written by tools/gen_bundle.py.
fn paper_hook_ev_bolt(game: &mut crate::game::Game<'_>, at: [f64; 3], visual_only: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("bolt") {
        let payload = serde_json::json!({ "at": at, "visual_only": visual_only });
        if crate::game::paper_bridge::on_refusable(game, "bolt", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
