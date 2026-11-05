// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_no_wear(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_no_wear") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::skip_wear() });
        if crate::game::paper_bridge::on_refusable(game, "ee_no_wear", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
