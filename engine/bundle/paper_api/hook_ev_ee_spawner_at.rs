// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_spawner_at(game: &mut crate::game::Game<'_>, key: Pos) {
    if crate::game::paper_bridge::hears("ee_spawner_at") {
        let payload = serde_json::json!({ "spawner": [key.0, key.1, key.2] });
        crate::game::paper_bridge::on_trigger(game, "ee_spawner_at", payload);
    }
}
