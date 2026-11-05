// Written by tools/gen_bundle.py.
fn paper_hook_ev_cart_enter(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("cart_enter") {
        let payload = serde_json::json!({  });
        if crate::game::paper_bridge::on_refusable(game, "cart_enter", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
