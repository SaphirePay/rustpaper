// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_stations_tick(game: &mut crate::game::Game<'_>, host: &mut LanHost) {
    if crate::game::paper_bridge::hears("items_stations_tick") {
        let payload = serde_json::json!({ "uuid": "", "sent": (for conn in crate::game::paper_fam_items::pending_conns(&*host) { game.lan_send_station(host, conn); }), "upkeep": (crate::game::paper_fam_items::tick(game, host)) });
        crate::game::paper_bridge::on_trigger(game, "items_stations_tick", payload);
    }
}
