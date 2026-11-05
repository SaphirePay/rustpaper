// Written by tools/gen_bundle.py.
fn paper_hook_ev_dispense_try(game: &mut crate::game::Game<'_>, pos: BlockPos) {
    if crate::game::paper_bridge::hears("dispense_try") {
        let payload = serde_json::json!({ "pos": [pos.x, pos.y, pos.z] });
        crate::game::paper_bridge::on_trigger(game, "dispense_try", payload);
    }
}
