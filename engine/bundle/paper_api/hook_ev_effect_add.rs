// Written by tools/gen_bundle.py.
fn paper_hook_ev_effect_add(game: &mut crate::game::Game<'_>, effect: &EffectInstance) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("effect_add") {
        let payload = serde_json::json!({ "effect": effect.id, "amplifier": effect.amplifier, "duration": effect.duration });
        if crate::game::paper_bridge::on_refusable(game, "effect_add", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
