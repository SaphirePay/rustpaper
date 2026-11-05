// Written by tools/gen_bundle.py.
fn paper_hook_ev_crafter_slot(game: &mut crate::game::Game<'_>, pos: Pos) {
    if crate::game::paper_bridge::hears("crafter_slot") {
        let payload = serde_json::json!({ "pos": [pos.0, pos.1, pos.2] });
        crate::game::paper_bridge::on_trigger(game, "crafter_slot", payload);
    }
}
