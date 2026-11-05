// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_hit(game: &mut crate::game::Game<'_>, p: &Projectile, hit: Hit) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("ee_hit") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::projectile_hit(game, p, hit) });
        if crate::game::paper_bridge::on_refusable(game, "ee_hit", payload) {
            return ::core::ops::ControlFlow::Break({ crate::game::paper_fam_ev_entity::fly_on(game, p); Ok(()) });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
