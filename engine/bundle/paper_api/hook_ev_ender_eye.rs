// Written by tools/gen_bundle.py.
fn paper_hook_ev_ender_eye(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ender_eye") {
        let payload = serde_json::json!({  });
        if crate::game::paper_bridge::on_refusable(game, "ender_eye", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
