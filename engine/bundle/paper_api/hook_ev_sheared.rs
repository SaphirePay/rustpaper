// Written by tools/gen_bundle.py.
fn paper_hook_ev_sheared(game: &mut crate::game::Game<'_>, target: Pos) {
    if crate::game::paper_bridge::hears("sheared") {
        let payload = serde_json::json!({ "pos": [target.0, target.1, target.2] });
        crate::game::paper_bridge::on_trigger(game, "sheared", payload);
    }
}
