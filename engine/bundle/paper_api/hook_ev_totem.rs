// Written by tools/gen_bundle.py.
fn paper_hook_ev_totem(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("totem") {
        let payload = serde_json::json!({  });
        crate::game::paper_bridge::on_trigger(game, "totem", payload);
    }
}
