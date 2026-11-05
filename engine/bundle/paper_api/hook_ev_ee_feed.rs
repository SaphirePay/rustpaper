// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_feed(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ee_feed") {
        let payload = serde_json::json!({ "ee_fed": crate::game::paper_fam_ev_entity::note_feed(game) });
        crate::game::paper_bridge::on_trigger(game, "ee_feed", payload);
    }
}
