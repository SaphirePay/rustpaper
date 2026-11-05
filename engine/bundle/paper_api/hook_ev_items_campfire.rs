// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_campfire(game: &mut crate::game::Game<'_>, pos: Pos) {
    if crate::game::paper_bridge::hears("items_campfire") {
        let payload = serde_json::json!({ "uuid": "", "held": (crate::game::paper_fam_items::campfire_hold(game, pos)) });
        crate::game::paper_bridge::on_trigger(game, "items_campfire", payload);
    }
}
