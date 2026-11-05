// Written by tools/gen_bundle.py.
fn paper_hook_ev_raid_wave(game: &mut crate::game::Game<'_>, raid_id: i32, pos: [i32; 3], wave: i32) {
    if crate::game::paper_bridge::hears("raid_wave") {
        let payload = serde_json::json!({ "raid": raid_id, "pos": pos, "wave": wave });
        crate::game::paper_bridge::on_trigger(game, "raid_wave", payload);
    }
}
