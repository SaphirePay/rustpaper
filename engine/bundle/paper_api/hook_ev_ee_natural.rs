// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_natural(game: &mut crate::game::Game<'_>, pending: &Vec<Mob>, persistent: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_natural") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::natural(game, pending) });
        if crate::game::paper_bridge::on_refusable(game, "ee_natural", payload) {
            return ::core::ops::ControlFlow::Break(game.add_spawned(crate::game::paper_fam_ev_entity::kept(pending), persistent));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
