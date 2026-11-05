// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_raid_tick(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("core_raid_tick") {
        let payload = serde_json::json!({ "done": ({
    for (id, waves) in crate::game::paper_fam_core::take_raid_waves() {
        if let Some(raid) = game.raiding.raids.map.get_mut(&id) {
            raid.num_groups = waves;
            game.raiding.raids.dirty = true;
        }
    }
    0
}) });
        crate::game::paper_bridge::on_trigger(game, "core_raid_tick", payload);
    }
}
