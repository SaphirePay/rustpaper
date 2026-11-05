// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_death(game: &mut crate::game::Game<'_>, m: &horizon_client::mobs::Mob) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_death") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::death(game, m) });
        if crate::game::paper_bridge::on_refusable(game, "ee_death", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
