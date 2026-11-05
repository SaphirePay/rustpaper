// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_map_fill(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_map_fill") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::map_gate(game, stack, |game, stack| game.use_empty_map(stack)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_map_fill", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
