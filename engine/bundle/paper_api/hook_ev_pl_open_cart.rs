// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_open_cart(game: &mut crate::game::Game<'_>, i: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_open_cart") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::vehicle_later(game, game.carts[i].id, game.carts[i].items.clone(), |g, id| if let Some(i) = g.carts.iter().position(|c| c.id == id) { g.open_cart_container(i) }) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_open_cart", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
