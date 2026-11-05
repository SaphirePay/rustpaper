// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_bolt_trident(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ew_bolt_trident") {
        let payload = serde_json::json!({ "uuid": null, "why": "TRIDENT" });
        crate::game::paper_bridge::on_trigger(game, "ew_bolt_trident", payload);
    }
}
