// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_unleash(game: &mut crate::game::Game<'_>, id: u32, drop: bool, sound: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_unleash") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::unleash(game, id, drop) });
        if crate::game::paper_bridge::on_refusable(game, "ee_unleash", payload) {
            return ::core::ops::ControlFlow::Break(if let Some(drop) = crate::game::paper_fam_ev_entity::lead_drop(drop) { game.break_leash(id, drop, sound) });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
