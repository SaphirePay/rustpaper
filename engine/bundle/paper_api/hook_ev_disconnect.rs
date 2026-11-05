// Written by tools/gen_bundle.py.
fn paper_hook_ev_disconnect(game: &mut crate::game::Game<'_>, g: &horizon_client::lan::guest::Guest) {
    if crate::game::paper_bridge::hears("disconnect") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::note_left(g) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "disconnect", payload);
    }
}
