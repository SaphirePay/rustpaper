// Written by tools/gen_bundle.py.
fn paper_hook_ev_station_rename(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, name: &str) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("station_rename") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "name": name });
        if crate::game::paper_bridge::on_refusable(game, "station_rename", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
