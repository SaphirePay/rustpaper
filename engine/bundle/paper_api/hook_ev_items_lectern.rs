// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_lectern(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32) {
    if crate::game::paper_bridge::hears("items_lectern") {
        let payload = serde_json::json!({ "uuid": "", "open": ({ let who = host.hub.guest(conn).map(|g| g.uuid); match (who, host.play.get(&conn).and_then(|p| p.lectern.as_ref())) { (Some(who), Some(gl)) => crate::game::paper_fam_items::lectern_sent(who, gl.window, gl.key), _ => serde_json::Value::Null } }) });
        crate::game::paper_bridge::on_trigger(game, "items_lectern", payload);
    }
}
