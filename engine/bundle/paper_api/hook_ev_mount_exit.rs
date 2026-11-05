// Written by tools/gen_bundle.py.
fn paper_hook_ev_mount_exit(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("mount_exit") {
        let payload = serde_json::json!({  });
        if crate::game::paper_bridge::on_refusable(game, "mount_exit", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
