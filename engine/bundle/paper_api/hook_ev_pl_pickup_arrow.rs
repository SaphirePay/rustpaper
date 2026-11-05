// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_pickup_arrow(game: &mut crate::game::Game<'_>, host: &mut LanHost, gi: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_pickup_arrow") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::arrow_gate(game, host, gi, |g, h| g.guest_arrows(h, gi)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_pickup_arrow", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
