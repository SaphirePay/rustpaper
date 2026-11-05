// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_enter_block(game: &mut crate::game::Game<'_>, mob: u32, at: [i32; 3]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_enter_block") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "entity": mob, "pos": at });
        if crate::game::paper_bridge::on_refusable(game, "ee_enter_block", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
