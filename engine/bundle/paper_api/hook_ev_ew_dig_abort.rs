// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_dig_abort(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32) {
    if crate::game::paper_bridge::hears("ew_dig_abort") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "pos": host.play.get(&conn).and_then(|play| play.digging.map(|digging| digging.0)) });
        crate::game::paper_bridge::on_trigger(game, "ew_dig_abort", payload);
    }
}
