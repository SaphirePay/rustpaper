// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_why_beacon(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ee_why_beacon") {
        let payload = serde_json::json!({ "ee_why": crate::game::paper_fam_ev_entity::note_cause(game, "BEACON") });
        crate::game::paper_bridge::on_trigger(game, "ee_why_beacon", payload);
    }
}
