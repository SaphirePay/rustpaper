// Written by tools/gen_bundle.py.
fn paper_hook_ev_entity_fangs(game: &mut crate::game::Game<'_>, at: [f64; 3], yaw: f32, delay: i32, owner: u32) {
    if crate::game::paper_bridge::hears("entity_fangs") {
        let payload = serde_json::json!({ "done": crate::game::paper_fam_entity::fangs_placed(game, at, delay, owner) });
        crate::game::paper_bridge::on_trigger(game, "entity_fangs", payload);
    }
}
