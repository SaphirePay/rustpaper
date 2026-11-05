// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_vehicle_exit(game: &mut crate::game::Game<'_>, vehicle_y: f64) {
    if crate::game::paper_bridge::hears("ew_vehicle_exit") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "vehicle": game.riding, "at": vehicle_y, "cancellable": false });
        crate::game::paper_bridge::on_trigger(game, "ew_vehicle_exit", payload);
    }
}
