// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_move_turn(game: &mut crate::game::Game<'_>, host: &mut LanHost) {
    if crate::game::paper_bridge::hears("pl_move_turn") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::moves_turn(host) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_move_turn", payload);
    }
}
