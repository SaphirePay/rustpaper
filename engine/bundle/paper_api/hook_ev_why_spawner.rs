// Written by tools/gen_bundle.py.
fn paper_hook_ev_why_spawner(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("why_spawner") {
        let payload = serde_json::json!({ "reason": "SPAWNER" });
        crate::game::paper_bridge::on_trigger(game, "why_spawner", payload);
    }
}
