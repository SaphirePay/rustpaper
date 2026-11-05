// Written by tools/gen_bundle.py.
fn paper_hook_ev_dig_abort(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32) {
    if crate::game::paper_bridge::hears("dig_abort") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn) });
        crate::game::paper_bridge::on_trigger(game, "dig_abort", payload);
    }
}
