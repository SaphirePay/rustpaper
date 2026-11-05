// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_spectate(game: &mut crate::game::Game<'_>, host: &mut LanHost, gi: usize, uuid: u128) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_spectate") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::spectate_gate(game, host, gi, uuid) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_spectate", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
