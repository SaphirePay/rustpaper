// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_why_mob(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ee_why_mob") {
        let payload = serde_json::json!({ "ee_why": crate::game::paper_fam_ev_entity::note_attack(game) });
        crate::game::paper_bridge::on_trigger(game, "ee_why_mob", payload);
    }
}
