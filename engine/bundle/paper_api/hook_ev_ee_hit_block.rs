// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_hit_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], stacks: &mut Vec<([i32; 3], ItemStack)>) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("ee_hit_block") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::yield_set() });
        if crate::game::paper_bridge::on_refusable(game, "ee_hit_block", payload) {
            return ::core::ops::ControlFlow::Break(game.explosion_hit_block(pos, crate::game::paper_fam_ev_entity::yield_radius(), BlockInteraction::DestroyWithDecay, stacks));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
