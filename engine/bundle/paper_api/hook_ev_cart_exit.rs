// Written by tools/gen_bundle.py.
fn paper_hook_ev_cart_exit(game: &mut crate::game::Game<'_>, i: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("cart_exit") {
        let payload = serde_json::json!({ "index": i });
        if crate::game::paper_bridge::on_refusable(game, "cart_exit", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
