// Written by tools/gen_bundle.py.
fn paper_hook_adv_entity(game: &mut crate::game::Game<'_>, name: &str, target: &EntityView) {
    if crate::game::paper_bridge::hears("adv_entity") {
        let payload = serde_json::json!({ "kind": name, "entity_kind": target.kind, "at": target.position });
        crate::game::paper_bridge::on_trigger(game, "adv_entity", payload);
    }
}
