// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_compost(game: &mut crate::game::Game<'_>, mob: usize, pos: [i32; 3]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_compost") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "entity": game.mobs[mob].id, "pos": pos });
        if crate::game::paper_bridge::on_refusable(game, "ee_compost", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
