// Written by tools/gen_bundle.py.
fn paper_hook_ev_sign_open(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("sign_open") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn) });
        if crate::game::paper_bridge::on_refusable(game, "sign_open", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
