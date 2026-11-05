// Written by tools/gen_bundle.py.
fn paper_hook_ev_tnt_spawn(game: &mut crate::game::Game<'_>, pos: BlockPos) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("tnt_spawn") {
        let payload = serde_json::json!({ "pos": [pos.x, pos.y, pos.z] });
        if crate::game::paper_bridge::on_refusable(game, "tnt_spawn", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
