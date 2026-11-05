// Written by tools/gen_bundle.py.
fn paper_hook_ev_why_golem(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("why_golem") {
        let payload = serde_json::json!({ "reason": "VILLAGE_DEFENSE" });
        crate::game::paper_bridge::on_trigger(game, "why_golem", payload);
    }
}
