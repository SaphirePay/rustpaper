// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_throw_bottle(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> ::core::ops::ControlFlow<horizon_client::Result<bool>> {
    if crate::game::paper_bridge::hears("pl_throw_bottle") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::throw_gate(game, stack, |game, stack| game.use_potion_item(stack)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_throw_bottle", payload) {
            return ::core::ops::ControlFlow::Break(Ok(false));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
