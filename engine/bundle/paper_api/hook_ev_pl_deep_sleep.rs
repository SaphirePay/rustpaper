// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_deep_sleep(game: &mut crate::game::Game<'_>, host: &LanHost, gi: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_deep_sleep") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::deep_sleep(game, host, gi) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_deep_sleep", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
