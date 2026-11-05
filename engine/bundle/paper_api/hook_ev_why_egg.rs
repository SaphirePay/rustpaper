// Written by tools/gen_bundle.py.
fn paper_hook_ev_why_egg(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("why_egg") {
        let payload = serde_json::json!({ "reason": "SPAWNER_EGG" });
        crate::game::paper_bridge::on_trigger(game, "why_egg", payload);
    }
}
