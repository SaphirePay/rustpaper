// Written by tools/gen_bundle.py.
fn paper_hook_ev_why_trader(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("why_trader") {
        let payload = serde_json::json!({ "reason": "NATURAL" });
        crate::game::paper_bridge::on_trigger(game, "why_trader", payload);
    }
}
