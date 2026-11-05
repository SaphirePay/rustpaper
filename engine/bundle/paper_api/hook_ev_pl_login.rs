// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_login(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_login") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::login_gate(game, host, conn) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_login", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
