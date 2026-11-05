// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_mob(game: &mut crate::game::Game<'_>, events: &Vec<MobEvent>) -> ::core::ops::ControlFlow<Result<()>> {
    if crate::game::paper_bridge::hears("ee_mob") {
        let payload = serde_json::json!({ "ee_refused": crate::game::paper_fam_ev_entity::mob_events(game, events) });
        if crate::game::paper_bridge::on_refusable(game, "ee_mob", payload) {
            return ::core::ops::ControlFlow::Break(game.handle_mob_events(crate::game::paper_fam_ev_entity::kept_events(events)));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
