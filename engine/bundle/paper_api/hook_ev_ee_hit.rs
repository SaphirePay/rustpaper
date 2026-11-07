// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_hit(game: &mut crate::game::Game<'_>, p: &Projectile, hit: Hit) -> ::core::ops::ControlFlow<Result<()>> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_hit") && crate::game::paper_fam_ev_entity::projectile_hit(game, p, hit) {
        return ::core::ops::ControlFlow::Break({ crate::game::paper_fam_ev_entity::fly_on(game, p); Ok(()) });
    }
    ::core::ops::ControlFlow::Continue(())
}
