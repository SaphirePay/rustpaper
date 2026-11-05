// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_tick(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ee_tick") {
        let payload = serde_json::json!({ "ee_tick": crate::game::paper_fam_ev_entity::tick_note(game) });
        crate::game::paper_bridge::on_trigger(game, "ee_tick", payload);
    }
}
