// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_exp(game: &mut crate::game::Game<'_>, gained: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_exp") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::exp_change(game, gained, |game, amount| game.give_orb_points(amount)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_exp", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
