// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_death_mark(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ee_death_mark") {
        let payload = serde_json::json!({ "ee_mark": crate::game::paper_fam_ev_entity::mark_death(game) });
        crate::game::paper_bridge::on_trigger(game, "ee_death_mark", payload);
    }
}
