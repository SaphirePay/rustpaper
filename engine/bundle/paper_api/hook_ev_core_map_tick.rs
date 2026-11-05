// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_map_tick(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("core_map_tick") {
        let payload = serde_json::json!({ "done": ({
    for id in crate::game::paper_fam_core::take_touched_maps() {
        if let Some(map) = game.map(id) {
            map.dirty = true;
            map.texture_dirty = true;
            for holder in &mut map.holders {
                holder.dirty = Some([0, 0, 127, 127]);
            }
        }
    }
    0
}) });
        crate::game::paper_bridge::on_trigger(game, "core_map_tick", payload);
    }
}
