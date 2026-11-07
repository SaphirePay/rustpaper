// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_unleash(game: &mut crate::game::Game<'_>, id: u32, drop: bool, sound: bool) -> ::core::ops::ControlFlow<()> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_unleash") && crate::game::paper_fam_ev_entity::unleash(game, id, drop) {
        return ::core::ops::ControlFlow::Break(if let Some(drop) = crate::game::paper_fam_ev_entity::lead_drop(drop) { game.break_leash(id, drop, sound) });
    }
    ::core::ops::ControlFlow::Continue(())
}
