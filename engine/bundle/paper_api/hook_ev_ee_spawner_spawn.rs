// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_spawner_spawn(game: &mut crate::game::Game<'_>, key: Pos, ids: &HashMap<u32, u32>) {
    if crate::game::paper_bridge::hears("ee_spawner_spawn") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "spawner": [key.0, key.1, key.2], "ids": ids.values().collect::<Vec<_>>() });
        crate::game::paper_bridge::on_trigger(game, "ee_spawner_spawn", payload);
    }
}
