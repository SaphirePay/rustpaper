// Written by tools/gen_bundle.py.
fn paper_hook_ev_entity_tnt(game: &mut crate::game::Game<'_>, pos: BlockPos) {
    if crate::game::paper_bridge::hears("entity_tnt") {
        let payload = serde_json::json!({ "done": crate::game::paper_fam_entity::tnt_primed(game, pos) });
        crate::game::paper_bridge::on_trigger(game, "entity_tnt", payload);
    }
}
