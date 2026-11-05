// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_furnace(game: &mut crate::game::Game<'_>, pos: Pos) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("pl_furnace") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::furnace_tick(game, pos, |game, pos| game.tick_furnace(pos)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_furnace", payload) {
            return ::core::ops::ControlFlow::Break(Ok(()));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
