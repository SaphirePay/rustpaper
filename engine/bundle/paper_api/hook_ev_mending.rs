// Written by tools/gen_bundle.py.
fn paper_hook_ev_mending(game: &mut crate::game::Game<'_>, amount: u32) {
    if crate::game::paper_bridge::hears("mending") {
        let payload = serde_json::json!({ "amount": amount });
        crate::game::paper_bridge::on_trigger(game, "mending", payload);
    }
}
