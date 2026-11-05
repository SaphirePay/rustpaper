// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_blast_end(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ee_blast_end") {
        let payload = serde_json::json!({ "ee_why": crate::game::paper_fam_ev_entity::blast_over(game) });
        crate::game::paper_bridge::on_trigger(game, "ee_blast_end", payload);
    }
}
