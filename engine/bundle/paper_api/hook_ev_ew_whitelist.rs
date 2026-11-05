// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_whitelist(game: &mut crate::game::Game<'_>, on: bool) {
    if crate::game::paper_bridge::hears("ew_whitelist") {
        let payload = serde_json::json!({ "uuid": null, "on": on });
        crate::game::paper_bridge::on_trigger(game, "ew_whitelist", payload);
    }
}
