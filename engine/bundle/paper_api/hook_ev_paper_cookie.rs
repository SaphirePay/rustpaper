// Written by tools/gen_bundle.py.
fn paper_hook_ev_paper_cookie(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, packet: &S) {
    if crate::game::paper_bridge::hears("paper_cookie") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "cookie": (if let S::CookieResponse(p) = packet { serde_json::json!({ "key": p.key, "payload": p.payload }) } else { serde_json::Value::Null }) });
        crate::game::paper_bridge::on_trigger(game, "paper_cookie", payload);
    }
}
