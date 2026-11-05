// Written by tools/gen_bundle.py.
fn paper_hook_ev_lectern_page(game: &mut crate::game::Game<'_>, forward: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("lectern_page") {
        let payload = serde_json::json!({ "forward": forward });
        if crate::game::paper_bridge::on_refusable(game, "lectern_page", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
