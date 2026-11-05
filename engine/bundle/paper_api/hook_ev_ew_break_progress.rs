// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_break_progress(game: &mut crate::game::Game<'_>, breaker: i32, pos: [i32; 3], progress: i32) {
    if crate::game::paper_bridge::hears("ew_break_progress") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "breaker": breaker, "pos": pos, "progress": progress });
        crate::game::paper_bridge::on_trigger(game, "ew_break_progress", payload);
    }
}
