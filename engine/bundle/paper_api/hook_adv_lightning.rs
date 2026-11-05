// Written by tools/gen_bundle.py.
fn paper_hook_adv_lightning(game: &mut crate::game::Game<'_>, bolt: &EntityView) {
    if crate::game::paper_bridge::hears("adv_lightning") {
        let payload = serde_json::json!({ "at": bolt.position });
        crate::game::paper_bridge::on_trigger(game, "adv_lightning", payload);
    }
}
