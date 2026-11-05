// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_stand_use(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_stand_use") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id(), "use": crate::game::paper_fam_ev_player::stand_use(game, |game, held| game.equipment_slot(held)) }); if crate::game::paper_fam_ev_player::fire(game, "pl_stand_use", payload) { return ::core::ops::ControlFlow::Break(true); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_stand_use", payload) {
            return ::core::ops::ControlFlow::Break(true);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
