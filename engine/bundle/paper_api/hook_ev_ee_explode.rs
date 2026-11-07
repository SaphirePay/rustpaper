// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_explode(game: &mut crate::game::Game<'_>, center: [f64; 3], radius: f32, interaction: BlockInteraction, fire: bool, damage_type: &str, resists: Option<([i32; 3], f32)>) -> ::core::ops::ControlFlow<Result<()>> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_explode") && crate::game::paper_fam_ev_entity::explode(game, center, radius, fire, interaction) {
        return ::core::ops::ControlFlow::Break(crate::game::paper_fam_ev_entity::explosion_again(game, center, interaction, damage_type, resists));
    }
    ::core::ops::ControlFlow::Continue(())
}
