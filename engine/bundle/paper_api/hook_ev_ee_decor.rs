// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_decor(game: &mut crate::game::Game<'_>, blast: [f64; 3], diameter: f64, interaction: super::explosion::BlockInteraction) {
    if crate::game::paper_bridge::hears("ee_decor") {
        let payload = serde_json::json!({ "ee_why": !game.blast_spares_decorations(interaction) && crate::game::paper_fam_ev_entity::decorations(game, blast, diameter) });
        crate::game::paper_bridge::on_trigger(game, "ee_decor", payload);
    }
}
