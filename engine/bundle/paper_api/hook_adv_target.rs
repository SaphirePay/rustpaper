// Written by tools/gen_bundle.py.
fn paper_hook_adv_target(game: &mut crate::game::Game<'_>, projectile: &EntityView, strength: i32) {
    if crate::game::paper_bridge::hears("adv_target") {
        let payload = serde_json::json!({ "strength": strength, "at": projectile.position });
        crate::game::paper_bridge::on_trigger(game, "adv_target", payload);
    }
}
