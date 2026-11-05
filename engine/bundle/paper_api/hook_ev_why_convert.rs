// Written by tools/gen_bundle.py.
fn paper_hook_ev_why_convert(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("why_convert") {
        let payload = serde_json::json!({ "reason": "CONVERSION" });
        crate::game::paper_bridge::on_trigger(game, "why_convert", payload);
    }
}
