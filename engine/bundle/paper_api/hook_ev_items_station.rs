// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_station(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32) {
    if crate::game::paper_bridge::hears("items_station") {
        let payload = serde_json::json!({ "uuid": "", "open": ({ let who = host.hub.guest(conn).map(|g| g.uuid); let hub = &host.hub; match (who, host.play.get_mut(&conn).and_then(|p| p.station.as_mut())) { (Some(who), Some(gs)) => crate::game::paper_fam_items::menu_sent(game, hub, who, gs.window, gs.pos, &mut gs.menu), _ => serde_json::Value::Null } }) });
        crate::game::paper_bridge::on_trigger(game, "items_station", payload);
    }
}
