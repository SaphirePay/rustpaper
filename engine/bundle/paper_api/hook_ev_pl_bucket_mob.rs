// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_bucket_mob(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<bool>> {
    if crate::game::paper_bridge::hears("pl_bucket_mob") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id(), "scoop": crate::game::paper_fam_ev_player::scooped(game, |mob| bucketable(mob.species()).map(|found| found.0)), "stack": game.held().map(crate::game::paper_fam_ev_player::stack_value), "hand": crate::game::paper_fam_ev_player::hand(game) }); if crate::game::paper_fam_ev_player::fire(game, "pl_bucket_mob", payload) { return ::core::ops::ControlFlow::Break(Ok(true)); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_bucket_mob", payload) {
            return ::core::ops::ControlFlow::Break(Ok(true));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
