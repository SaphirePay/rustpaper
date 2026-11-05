// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_smash(game: &mut crate::game::Game<'_>, victim: &str) -> ::core::ops::ControlFlow<f32> {
    if crate::game::paper_bridge::hears("ee_smash") {
        let payload = serde_json::json!({ "smash": game.smash_attack(), "victim": victim, "weapon": game.held().map(crate::game::paper_query::stack_json) });
        if crate::game::paper_bridge::on_refusable(game, "ee_smash", payload) {
            return ::core::ops::ControlFlow::Break(0.0);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
