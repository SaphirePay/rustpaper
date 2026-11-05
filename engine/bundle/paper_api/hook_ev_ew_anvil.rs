// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_anvil(game: &mut crate::game::Game<'_>, pos: [i32; 3], creative: bool) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("ew_anvil") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": pos, "creative": creative, "roll": game.world.random.clone().next_float() });
        if crate::game::paper_bridge::on_refusable(game, "ew_anvil", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
