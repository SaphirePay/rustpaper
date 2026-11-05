// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_cart_exit(game: &mut crate::game::Game<'_>, i: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_cart_exit") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "vehicle": game.carts[i].id });
        if crate::game::paper_bridge::on_refusable(game, "ew_cart_exit", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
