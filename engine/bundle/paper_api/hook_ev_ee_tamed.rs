// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_tamed(game: &mut crate::game::Game<'_>, name: &str, target: &EntityView) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_tamed") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::tamed(game, name, &target.kind, target.position) });
        if crate::game::paper_bridge::on_refusable(game, "ee_tamed", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
