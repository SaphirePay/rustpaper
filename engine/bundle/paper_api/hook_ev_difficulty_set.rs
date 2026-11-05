// Written by tools/gen_bundle.py.
fn paper_hook_ev_difficulty_set(game: &mut crate::game::Game<'_>, i: u8) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("difficulty_set") {
        let payload = serde_json::json!({ "difficulty": i });
        if crate::game::paper_bridge::on_refusable(game, "difficulty_set", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
