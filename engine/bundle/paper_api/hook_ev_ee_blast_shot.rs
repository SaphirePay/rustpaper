// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_blast_shot(game: &mut crate::game::Game<'_>, p: &Projectile) {
    if crate::game::paper_bridge::hears("ee_blast_shot") {
        let payload = serde_json::json!({ "ee_why": crate::game::paper_fam_ev_entity::blast_by_shot(p) });
        crate::game::paper_bridge::on_trigger(game, "ee_blast_shot", payload);
    }
}
