// Written by tools/gen_bundle.py.
fn paper_hook_ev_piglin_barter(game: &mut crate::game::Game<'_>, mob: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("piglin_barter") {
        let payload = serde_json::json!({ "entity": mob });
        if crate::game::paper_bridge::on_refusable(game, "piglin_barter", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
