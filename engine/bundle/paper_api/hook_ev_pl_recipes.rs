// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_recipes(game: &mut crate::game::Game<'_>, unlocked: &[String]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_recipes") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::recipe_gate(game, unlocked, |g, kept| g.recipe_unlocked(kept)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_recipes", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
