// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_packet(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, packet: &S) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_packet") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::hold_packet(game, host, conn, packet, |play| &mut play.attack_ticks, |play| play.using) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_packet", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
