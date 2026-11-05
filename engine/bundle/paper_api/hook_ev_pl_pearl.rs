// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_pearl(game: &mut crate::game::Game<'_>, p: &Projectile, hit: &Hit) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("pl_pearl") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::pearl_hit(game, matches!(p.kind, ProjectileKind::EnderPearl) && p.owner == Some(Who::Player), [p.previous.x, p.previous.y, p.previous.z], |game| game.projectile_hit(p, hit.clone())) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_pearl", payload) {
            return ::core::ops::ControlFlow::Break(Ok(()));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
