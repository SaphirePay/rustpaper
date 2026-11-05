// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_explode(game: &mut crate::game::Game<'_>, radius: f32, interaction: BlockInteraction) {
    if crate::game::paper_bridge::hears("ew_explode") {
        let payload = serde_json::json!({ "uuid": null, "radius": radius, "interaction": format!("{interaction:?}"), "begun": crate::game::paper_fam_ev_world::blast_begins() });
        crate::game::paper_bridge::on_trigger(game, "ew_explode", payload);
    }
}
