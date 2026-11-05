// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_moves(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_moves") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::moves_check(game) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_moves", payload);
    }
}
