// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_frame_hit(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_frame_hit") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::frame_gate(game, true, None, |game| game.attack_frame()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_frame_hit", payload) {
            return ::core::ops::ControlFlow::Break(true);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
