// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_open_left(game: &mut crate::game::Game<'_>, host: &mut LanHost) {
    if crate::game::paper_bridge::hears("pl_open_left") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::open_mark(host) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_open_left", payload);
    }
}
