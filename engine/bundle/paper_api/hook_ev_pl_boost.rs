// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_boost(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_boost") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::boost_gate(game, stack, |game, stack| game.boost_firework(stack)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_boost", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
