// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_lunge(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_lunge") {
        let payload = serde_json::json!({ "power": game.held().map_or(0, |stack| stack.enchantment_level("minecraft:lunge")) });
        if crate::game::paper_bridge::on_refusable(game, "ee_lunge", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
