// Written by tools/gen_bundle.py.
fn paper_hook_ev_anchor_use(game: &mut crate::game::Game<'_>, pos: [i32; 3]) {
    if crate::game::paper_bridge::hears("anchor_use") {
        let payload = serde_json::json!({ "pos": pos });
        crate::game::paper_bridge::on_trigger(game, "anchor_use", payload);
    }
}
