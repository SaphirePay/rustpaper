// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_collide_boat(game: &mut crate::game::Game<'_>, i: usize, j: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_collide_boat") {
        let payload = serde_json::json!({ "entities": crate::game::paper_fam_ev_entity::boat_rows(game, i, j) });
        if crate::game::paper_bridge::on_refusable(game, "ee_collide_boat", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
