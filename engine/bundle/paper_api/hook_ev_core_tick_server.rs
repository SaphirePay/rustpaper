// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_tick_server(game: &mut crate::game::Game<'_>, paused: bool) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("core_tick_server") {
        let payload = serde_json::json!({ "skip": crate::game::paper_fam_core::server_pass(game, paused) });
        if crate::game::paper_bridge::on_refusable(game, "core_tick_server", payload) {
            return ::core::ops::ControlFlow::Break(Ok(()));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
