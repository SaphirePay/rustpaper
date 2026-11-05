// Written by tools/gen_bundle.py.
fn paper_hook_ev_why_breed(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("why_breed") {
        let payload = serde_json::json!({ "reason": "BREEDING" });
        crate::game::paper_bridge::on_trigger(game, "why_breed", payload);
    }
}
