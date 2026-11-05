// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_sculk(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("items_sculk") {
        let payload = serde_json::json!({ "uuid": "", "shrieks": (for key in crate::game::paper_fam_items::take_shrieks() { let _ = game.try_shriek(key, true); }), "blooms": (for key in crate::game::paper_fam_items::take_blooms() { game.catalyst_bloom(key); }) });
        crate::game::paper_bridge::on_trigger(game, "items_sculk", payload);
    }
}
