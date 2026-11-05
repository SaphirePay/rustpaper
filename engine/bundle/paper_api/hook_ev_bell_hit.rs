// Written by tools/gen_bundle.py.
fn paper_hook_ev_bell_hit(game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("bell_hit") {
        let payload = serde_json::json!({ "pos": pos });
        if crate::game::paper_bridge::on_refusable(game, "bell_hit", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
