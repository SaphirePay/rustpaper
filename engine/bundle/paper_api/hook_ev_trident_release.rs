// Written by tools/gen_bundle.py.
fn paper_hook_ev_trident_release(game: &mut crate::game::Game<'_>, ticks: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("trident_release") {
        let payload = serde_json::json!({ "ticks": ticks });
        if crate::game::paper_bridge::on_refusable(game, "trident_release", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
