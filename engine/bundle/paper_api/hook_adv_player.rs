// Written by tools/gen_bundle.py.
fn paper_hook_adv_player(game: &mut crate::game::Game<'_>, name: &str) {
    if crate::game::paper_bridge::hears("adv_player") {
        let payload = serde_json::json!({ "kind": name });
        crate::game::paper_bridge::on_trigger(game, "adv_player", payload);
    }
}
