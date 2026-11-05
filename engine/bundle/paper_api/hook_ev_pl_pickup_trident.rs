// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_pickup_trident(game: &mut crate::game::Game<'_>, t: &ThrownTrident, by: &Catcher) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_pickup_trident") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::trident_gate(game, by.guest, t.id, &t.stack, t.creative_only) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_pickup_trident", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
