// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_breed(game: &mut crate::game::Game<'_>, parent: u32, partner: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_breed") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::breed(game, parent, partner) });
        if crate::game::paper_bridge::on_refusable(game, "ee_breed", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
