// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_effect(game: &mut crate::game::Game<'_>, effect: &EffectInstance) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("ee_effect") {
        let payload = serde_json::json!({ "effect": effect.id, "amplifier": effect.amplifier, "duration": effect.duration, "cause": crate::game::paper_fam_ev_entity::effect_cause(game), "source": crate::game::paper_fam_ev_entity::effect_source(game), "old": game.potions.effects.get(effect.id).map(|old| serde_json::json!({ "amplifier": old.amplifier, "duration": old.duration })) });
        if crate::game::paper_bridge::on_refusable(game, "ee_effect", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
