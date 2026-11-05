// Written by tools/gen_bundle.py.
fn paper_hook_adv_beacon(game: &mut crate::game::Game<'_>, levels: i32) {
    if crate::game::paper_bridge::hears("adv_beacon") {
        let payload = serde_json::json!({ "levels": levels });
        crate::game::paper_bridge::on_trigger(game, "adv_beacon", payload);
    }
}
