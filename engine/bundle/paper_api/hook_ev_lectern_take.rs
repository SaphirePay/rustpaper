// Written by tools/gen_bundle.py.
fn paper_hook_ev_lectern_take(game: &mut crate::game::Game<'_>, key: (i32, i32, i32)) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("lectern_take") {
        let payload = serde_json::json!({ "pos": [key.0, key.1, key.2] });
        if crate::game::paper_bridge::on_refusable(game, "lectern_take", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
