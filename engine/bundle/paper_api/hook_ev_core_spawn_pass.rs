// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_spawn_pass(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("core_spawn_pass") {
        let payload = serde_json::json!({ "done": crate::game::paper_fam_core::spawn_pass(game) });
        crate::game::paper_bridge::on_trigger(game, "core_spawn_pass", payload);
    }
}
