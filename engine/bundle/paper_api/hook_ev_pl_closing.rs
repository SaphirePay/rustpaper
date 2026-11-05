// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_closing(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32) {
    if crate::game::paper_bridge::hears("pl_closing") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::close_gate(game, host, conn, None) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_closing", payload);
    }
}
