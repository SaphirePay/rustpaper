// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_bone_meal(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> ::core::ops::ControlFlow<Result<bool>> {
    if crate::game::paper_bridge::hears("ew_bone_meal") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "stack": crate::game::paper_query::stack_json(stack), "pos": game.pick().map(|hit| hit.pos) });
        if crate::game::paper_bridge::on_refusable(game, "ew_bone_meal", payload) {
            return ::core::ops::ControlFlow::Break(Ok(false));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
