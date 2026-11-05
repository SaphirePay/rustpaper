// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_effect_tick(game: &mut crate::game::Game<'_>, action: &EffectAction) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_effect_tick") {
        let payload = serde_json::json!({ "effect": crate::game::paper_fam_ev_entity::effect_of(action), "amplifier": game.potions.effects.get(crate::game::paper_fam_ev_entity::effect_of(action)).map(|effect| effect.amplifier) });
        if crate::game::paper_bridge::on_refusable(game, "ee_effect_tick", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
