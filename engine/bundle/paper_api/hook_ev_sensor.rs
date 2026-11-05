// Written by tools/gen_bundle.py.
fn paper_hook_ev_sensor(game: &mut crate::game::Game<'_>, key: (i32, i32, i32), power: i32) {
    if crate::game::paper_bridge::hears("sensor") {
        let payload = serde_json::json!({ "pos": [key.0, key.1, key.2], "power": power });
        crate::game::paper_bridge::on_trigger(game, "sensor", payload);
    }
}
