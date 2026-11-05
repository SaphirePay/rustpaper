// Written by tools/gen_bundle.py.
fn paper_hook_ev_sculk_bloom(game: &mut crate::game::Game<'_>, key: (i32, i32, i32)) {
    if crate::game::paper_bridge::hears("sculk_bloom") {
        let payload = serde_json::json!({ "pos": [key.0, key.1, key.2] });
        crate::game::paper_bridge::on_trigger(game, "sculk_bloom", payload);
    }
}
