// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_frame(game: &mut crate::game::Game<'_>, target: Option<InteractTarget>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_frame") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::frame_gate(game, false, target.map(|target| target.id), |game| game.interact_frame(target)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_frame", payload) {
            return ::core::ops::ControlFlow::Break(true);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
