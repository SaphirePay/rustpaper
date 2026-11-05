// Written by tools/gen_bundle.py.
fn paper_hook_ev_painting_break(game: &mut crate::game::Game<'_>, index: usize, by_player: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("painting_break") {
        let payload = serde_json::json!({ "index": index, "by_player": by_player });
        if crate::game::paper_bridge::on_refusable(game, "painting_break", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
