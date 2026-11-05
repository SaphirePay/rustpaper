// Written by tools/gen_bundle.py.
fn paper_hook_ev_exp_points(game: &mut crate::game::Game<'_>, amount: i32) {
    if crate::game::paper_bridge::hears("exp_points") {
        let payload = serde_json::json!({ "amount": amount });
        crate::game::paper_bridge::on_trigger(game, "exp_points", payload);
    }
}
