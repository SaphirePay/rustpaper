// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_egg(game: &mut crate::game::Game<'_>, p: &Projectile, hit: &Hit) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("pl_egg") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::egg_hit(game, p.id, matches!(p.kind, ProjectileKind::Egg), [p.position.x, p.position.y, p.position.z], p.guest, |game| game.projectile_hit(p, hit.clone())) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_egg", payload) {
            return ::core::ops::ControlFlow::Break(Ok(()));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
