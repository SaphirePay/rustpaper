// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_version_line(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, command: &str) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("core_version_line") {
        let payload = serde_json::json!({ "skip": crate::game::paper_fam_core::version_line(host, conn, command) });
        if crate::game::paper_bridge::on_refusable(game, "core_version_line", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
