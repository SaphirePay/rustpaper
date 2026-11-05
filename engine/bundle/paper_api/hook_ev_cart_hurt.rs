// Written by tools/gen_bundle.py.
fn paper_hook_ev_cart_hurt(game: &mut crate::game::Game<'_>, i: usize, damage: f32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("cart_hurt") {
        let payload = serde_json::json!({ "index": i, "damage": damage });
        if crate::game::paper_bridge::on_refusable(game, "cart_hurt", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
