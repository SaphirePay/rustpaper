// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_fish_xp(game: &mut crate::game::Game<'_>, at: [f64; 3]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_fish_xp") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::fish_orbs(game, at, |game, at, value| game.spawn_orbs(at, value)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_fish_xp", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
