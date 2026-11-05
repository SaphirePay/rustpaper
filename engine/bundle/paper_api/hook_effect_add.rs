// Written by tools/gen_bundle.py.
fn paper_hook_effect_add(game: &mut crate::game::Game<'_>, effect: &EffectInstance) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_events::on_effect_add(game, effect.id, effect.amplifier) {
        ::core::ops::ControlFlow::Continue(())
    } else {
        ::core::ops::ControlFlow::Break(false)
    }
}
