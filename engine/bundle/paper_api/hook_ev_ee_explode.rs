// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_explode(game: &mut crate::game::Game<'_>, center: [f64; 3], radius: f32, interaction: BlockInteraction, fire: bool, damage_type: &str, resists: Option<([i32; 3], f32)>) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("ee_explode") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::explode(game, center, radius, fire, interaction) });
        if crate::game::paper_bridge::on_refusable(game, "ee_explode", payload) {
            return ::core::ops::ControlFlow::Break(crate::game::paper_fam_ev_entity::explosion_again(game, center, interaction, damage_type, resists));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
