// Written by tools/gen_bundle.py.
fn paper_hook_ev_boat_hurt(game: &mut crate::game::Game<'_>, i: usize, damage: f32) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("boat_hurt") {
        let payload = serde_json::json!({ "index": i, "damage": damage });
        if crate::game::paper_bridge::on_refusable(game, "boat_hurt", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
