// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_trap(game: &mut crate::game::Game<'_>, id: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_trap") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "entity": id });
        if crate::game::paper_bridge::on_refusable(game, "ee_trap", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
