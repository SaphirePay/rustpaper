// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_sign_click(game: &mut crate::game::Game<'_>, front: bool) {
    if crate::game::paper_bridge::hears("pl_sign_click") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::sign_side(front) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_sign_click", payload);
    }
}
