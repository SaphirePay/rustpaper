// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_compost_add(game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("ew_compost_add") {
        let payload = serde_json::json!({ "uuid": null, "adding": true, "pos": pos });
        if crate::game::paper_bridge::on_refusable(game, "ew_compost_add", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
