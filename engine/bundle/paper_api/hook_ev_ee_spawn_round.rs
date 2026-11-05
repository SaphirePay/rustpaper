// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_spawn_round(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_spawn_round") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::spawn_round(game) });
        if crate::game::paper_bridge::on_refusable(game, "ee_spawn_round", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
