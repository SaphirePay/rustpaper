// Written by tools/gen_bundle.py.
fn paper_hook_ev_entity_after_mobs(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("entity_after_mobs") {
        let payload = serde_json::json!({ "done": crate::game::paper_fam_entity::after_mobs(game) });
        crate::game::paper_bridge::on_trigger(game, "entity_after_mobs", payload);
    }
}
