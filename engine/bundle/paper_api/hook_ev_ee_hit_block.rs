// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_hit_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], stacks: &mut Vec<([i32; 3], ItemStack)>) -> ::core::ops::ControlFlow<Result<()>> {
    // EntityExplodeEvent's say for each block a blast takes is whether a listener set the yield, which the game holds: asking every plugin for each block (hundreds a TNT) made a TNT field take a whole tick per blast.
    if crate::game::paper_bridge::hears("ee_hit_block") && crate::game::paper_fam_ev_entity::yield_set() {
        return ::core::ops::ControlFlow::Break(game.explosion_hit_block(pos, crate::game::paper_fam_ev_entity::yield_radius(), BlockInteraction::DestroyWithDecay, stacks));
    }
    ::core::ops::ControlFlow::Continue(())
}
