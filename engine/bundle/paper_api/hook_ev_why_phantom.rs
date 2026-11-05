// Written by tools/gen_bundle.py.
fn paper_hook_ev_why_phantom(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("why_phantom") {
        let payload = serde_json::json!({ "reason": "NATURAL" });
        crate::game::paper_bridge::on_trigger(game, "why_phantom", payload);
    }
}
