// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_item(game: &mut crate::game::Game<'_>, stack: &ItemStack, position: [f64; 3], velocity: [f64; 3], pickup_delay: u32) -> ::core::ops::ControlFlow<()> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_item") && crate::game::paper_fam_ev_entity::item_spawn(game, stack, position, velocity, pickup_delay) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
