// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_launch(game: &mut crate::game::Game<'_>, p: &mut horizon_client::projectiles::Projectile) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_launch") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::launch(game, p) });
        if crate::game::paper_bridge::on_refusable(game, "ee_launch", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
