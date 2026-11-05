// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_campfire(game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> ::core::ops::ControlFlow<Result<Option<bool>>> {
    if crate::game::paper_bridge::hears("ew_campfire") {
        let payload = serde_json::json!({ "pos": pos, "stack": game.held().map(crate::game::paper_query::stack_json), "time": game.held().cloned().and_then(|held| game.recipes().cook(KIND, &held).map(|recipe| recipe.time as i32)), "recipe": game.held().cloned().and_then(|held| game.recipes().cook(KIND, &held).map(|recipe| format!("minecraft:{}", recipe.id))) });
        if crate::game::paper_bridge::on_refusable(game, "ew_campfire", payload) {
            return ::core::ops::ControlFlow::Break(Ok(Some(false)));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
