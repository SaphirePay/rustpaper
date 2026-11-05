// Written by tools/gen_bundle.py.
fn paper_hook_ev_use_block(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, packet: &S) {
    if crate::game::paper_bridge::hears("use_block") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::note_click(host, conn, packet) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "use_block", payload);
    }
}
