// Written by tools/gen_bundle.py.
fn paper_hook_ev_advancement_done(game: &mut crate::game::Game<'_>, index: usize) {
    if crate::game::paper_bridge::hears("advancement_done") {
        let payload = serde_json::json!({ "index": index });
        crate::game::paper_bridge::on_trigger(game, "advancement_done", payload);
    }
}
