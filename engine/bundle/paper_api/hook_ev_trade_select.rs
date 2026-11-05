// Written by tools/gen_bundle.py.
fn paper_hook_ev_trade_select(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, index: i32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("trade_select") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "index": index });
        if crate::game::paper_bridge::on_refusable(game, "trade_select", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
