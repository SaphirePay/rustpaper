// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_mend(game: &mut crate::game::Game<'_>, amount: u32) -> ::core::ops::ControlFlow<u32> {
    if crate::game::paper_bridge::hears("pl_mend") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::mend(game, amount) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_mend", payload) {
            return ::core::ops::ControlFlow::Break(amount);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
