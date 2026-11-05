// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_launch(game: &mut crate::game::Game<'_>, p: &mut horizon_client::projectiles::Projectile) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_launch") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::launch(game, p) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_launch", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
