// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_game_mode(game: &mut crate::game::Game<'_>, mode: GameMode) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_game_mode") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id(), "mode": mode.name(), "changes": game.mode != mode, "cause": crate::game::paper_fam_ev_player::mode_cause(game) }); if crate::game::paper_fam_ev_player::fire(game, "pl_game_mode", payload) { return ::core::ops::ControlFlow::Break(false); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_game_mode", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
