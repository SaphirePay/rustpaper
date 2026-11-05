// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_stations(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_stations") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::stations_are(|host, conn| host.play.get_mut(&conn)?.station.as_mut().map(|station| (station.window, station.pos, &mut *station.menu))) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_stations", payload);
    }
}
