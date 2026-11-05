// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_advancement(game: &mut crate::game::Game<'_>, index: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_advancement") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::award_gate(game, index, |game, index| game.award(index)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_advancement", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
