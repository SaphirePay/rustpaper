// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_detonate(game: &mut crate::game::Game<'_>, center: [f64; 3], blocks: &mut Vec<[i32; 3]>) {
    if crate::game::paper_bridge::hears("ee_detonate") {
        let payload = serde_json::json!({ "ee_done": crate::game::paper_fam_ev_entity::detonate(game, center, blocks) });
        crate::game::paper_bridge::on_trigger(game, "ee_detonate", payload);
    }
}
