// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_orb(game: &mut crate::game::Game<'_>, host: &mut LanHost, gi: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_orb") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::orb_gate(game, host, gi, |play| &mut play.xp_delay, |game, host, gi| game.guest_orbs(host, gi)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_orb", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
