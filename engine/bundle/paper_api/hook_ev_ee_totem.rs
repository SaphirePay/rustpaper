// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_totem(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Option<Body>> {
    if crate::game::paper_bridge::hears("ee_totem") {
        let payload = serde_json::json!({ "used": game.player.as_ref().is_some_and(|p| p.health.totem_used), "hand": if game.held().is_some_and(|stack| &*stack.item == TOTEM) { "HAND" } else { "OFF_HAND" } });
        if crate::game::paper_bridge::on_refusable(game, "ee_totem", payload) {
            return ::core::ops::ControlFlow::Break({ crate::game::paper_fam_ev_entity::no_totem(game); None });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
