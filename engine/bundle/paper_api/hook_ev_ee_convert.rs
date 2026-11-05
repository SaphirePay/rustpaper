// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_convert(game: &mut crate::game::Game<'_>, from: &Mob) {
    if crate::game::paper_bridge::hears("ee_convert") {
        let payload = serde_json::json!({ "ee_from": crate::game::paper_fam_ev_entity::note_convert(from) });
        crate::game::paper_bridge::on_trigger(game, "ee_convert", payload);
    }
}
