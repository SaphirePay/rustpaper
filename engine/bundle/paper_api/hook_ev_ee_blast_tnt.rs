// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_blast_tnt(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ee_blast_tnt") {
        let payload = serde_json::json!({ "ee_why": crate::game::paper_fam_ev_entity::note_tnt(game.tnts.iter().map(|tnt| (tnt.id, [tnt.position.x, tnt.position.y, tnt.position.z])).collect()) });
        crate::game::paper_bridge::on_trigger(game, "ee_blast_tnt", payload);
    }
}
