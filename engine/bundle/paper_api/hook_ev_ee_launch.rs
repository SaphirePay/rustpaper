// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_launch(game: &mut crate::game::Game<'_>, p: &mut horizon_client::projectiles::Projectile) -> ::core::ops::ControlFlow<()> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_launch") && crate::game::paper_fam_ev_entity::launch(game, p) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
