// Written by tools/gen_bundle.py.
fn paper_hook_ev_crystal_kill(game: &mut crate::game::Game<'_>, id: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("crystal_kill") {
        let payload = serde_json::json!({ "entity": id });
        if crate::game::paper_bridge::on_refusable(game, "crystal_kill", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
