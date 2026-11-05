// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_void_mobs(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("core_void_mobs") {
        let payload = serde_json::json!({ "done": crate::game::paper_fam_core::void_rule_here(game.dim.id()) });
        crate::game::paper_bridge::on_trigger(game, "core_void_mobs", payload);
    }
}
