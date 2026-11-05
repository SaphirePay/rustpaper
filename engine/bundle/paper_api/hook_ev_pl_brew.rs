// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_brew(game: &mut crate::game::Game<'_>, pos: (i32, i32, i32)) -> ::core::ops::ControlFlow<horizon_client::Result<()>> {
    if crate::game::paper_bridge::hears("pl_brew") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::brew_tick(game, pos, |game, pos| game.tick_brewing_stand(pos)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_brew", payload) {
            return ::core::ops::ControlFlow::Break(Ok(()));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
