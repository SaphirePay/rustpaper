// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_crossbow(game: &mut crate::game::Game<'_>, ticks: u32, stack: &ItemStack) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_crossbow") {
        let payload = serde_json::json!({ "ready": (ticks + 1) as f32 / (Game::charge_ticks(stack).max(1) as f32) >= 1.0 && !stack.data().is_some_and(|data| !data.charged.is_empty()), "stack": crate::game::paper_query::stack_json(stack), "creative": game.creative, "hand": crate::game::paper_fam_ev_entity::hand_of(game, stack), "ammo": game.projectile_slot(CROSSBOW).and_then(|i| game.stack_in(i)).map(crate::game::paper_query::stack_json) });
        if crate::game::paper_bridge::on_refusable(game, "ee_crossbow", payload) {
            return ::core::ops::ControlFlow::Break(game.using = None);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
