// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_blast_crystal(game: &mut crate::game::Game<'_>, id: u32) {
    if crate::game::paper_bridge::hears("ee_blast_crystal") {
        let payload = serde_json::json!({ "ee_why": crate::game::paper_fam_ev_entity::blast_by_crystal(id) });
        crate::game::paper_bridge::on_trigger(game, "ee_blast_crystal", payload);
    }
}
