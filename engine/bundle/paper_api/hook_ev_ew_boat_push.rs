// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_boat_push(game: &mut crate::game::Game<'_>, i: usize, j: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_boat_push") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "vehicle": game.boats[i].id, "entity": game.mobs[j].id });
        if crate::game::paper_bridge::on_refusable(game, "ew_boat_push", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
