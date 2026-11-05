// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_idle(game: &mut crate::game::Game<'_>, minutes: i32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("core_idle") {
        let payload = serde_json::json!({ "skip": crate::game::paper_fam_core::idle_overridden(), "minutes": minutes });
        if crate::game::paper_bridge::on_refusable(game, "core_idle", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
