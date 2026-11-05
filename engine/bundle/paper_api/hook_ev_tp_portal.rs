// Written by tools/gen_bundle.py.
fn paper_hook_ev_tp_portal(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("tp_portal") {
        let payload = serde_json::json!({ "reason": "NETHER_PORTAL" });
        crate::game::paper_bridge::on_trigger(game, "tp_portal", payload);
    }
}
