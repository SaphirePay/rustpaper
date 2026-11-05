// Written by tools/gen_bundle.py.
fn paper_hook_ev_open_try(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, pos: (i32, i32, i32)) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("open_try") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "pos": [pos.0, pos.1, pos.2] });
        if crate::game::paper_bridge::on_refusable(game, "open_try", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
