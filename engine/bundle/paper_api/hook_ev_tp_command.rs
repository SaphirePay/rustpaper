// Written by tools/gen_bundle.py.
fn paper_hook_ev_tp_command(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("tp_command") {
        let payload = serde_json::json!({ "reason": "COMMAND" });
        crate::game::paper_bridge::on_trigger(game, "tp_command", payload);
    }
}
