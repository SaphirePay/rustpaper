// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_lock_station(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, station: &Station) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("items_lock_station") {
        let payload = serde_json::json!({ "uuid": "", "refused": (crate::game::paper_fam_items::lock_refuses(game, host, conn, (station.pos[0], station.pos[1], station.pos[2]))) });
        if crate::game::paper_bridge::on_refusable(game, "items_lock_station", payload) {
            return ::core::ops::ControlFlow::Break(true);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
