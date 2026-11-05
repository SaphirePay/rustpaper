// Written by tools/gen_bundle.py.
fn paper_hook_ev_tp_gateway(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("tp_gateway") {
        let payload = serde_json::json!({ "reason": "END_GATEWAY" });
        crate::game::paper_bridge::on_trigger(game, "tp_gateway", payload);
    }
}
