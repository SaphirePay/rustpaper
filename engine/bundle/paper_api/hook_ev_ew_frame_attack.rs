// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_frame_attack(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("ew_frame_attack") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "by_player": true, "entity": game.frame_hit().filter(|hit| game.item_frames[hit.0].item.is_none() && !game.item_frames[hit.0].fixed).map(|hit| game.item_frames[hit.0].id) });
        if crate::game::paper_bridge::on_refusable(game, "ew_frame_attack", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
