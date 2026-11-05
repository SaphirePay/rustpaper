// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_open_mount(game: &mut crate::game::Game<'_>, index: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_open_mount") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::mount_later(game, index) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_open_mount", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
