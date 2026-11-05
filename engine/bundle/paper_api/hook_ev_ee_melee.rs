// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_melee(game: &mut crate::game::Game<'_>, living: bool) {
    if crate::game::paper_bridge::hears("ee_melee") {
        let payload = serde_json::json!({ "ee_crit": living && game.critical_attack(game.attack_strength_scale()) });
        crate::game::paper_bridge::on_trigger(game, "ee_melee", payload);
    }
}
