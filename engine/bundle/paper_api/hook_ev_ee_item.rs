// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_item(game: &mut crate::game::Game<'_>, stack: &ItemStack, position: [f64; 3], velocity: [f64; 3], pickup_delay: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_item") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::item_spawn(game, stack, position, velocity, pickup_delay) });
        if crate::game::paper_bridge::on_refusable(game, "ee_item", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
