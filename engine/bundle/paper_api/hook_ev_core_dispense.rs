// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_dispense(game: &mut crate::game::Game<'_>, pos: Pos) {
    if crate::game::paper_bridge::hears("core_dispense") {
        let payload = serde_json::json!({ "done": crate::game::paper_fam_core::note_dispense(game, [pos.0, pos.1, pos.2]) });
        crate::game::paper_bridge::on_trigger(game, "core_dispense", payload);
    }
}
