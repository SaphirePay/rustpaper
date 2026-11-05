// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_spin(game: &mut crate::game::Game<'_>, hit: Option<u32>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_spin") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "target": hit });
        if crate::game::paper_bridge::on_refusable(game, "ee_spin", payload) {
            return ::core::ops::ControlFlow::Break(game.end_spin_attack(None));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
