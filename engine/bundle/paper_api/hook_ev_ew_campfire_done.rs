// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_campfire_done(game: &mut crate::game::Game<'_>, pos: Pos, stack: &ItemStack) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_campfire_done") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": [pos.0, pos.1, pos.2], "stack": crate::game::paper_query::stack_json(stack), "source": crate::game::paper_fam_ev_world::cooked().0, "recipe": crate::game::paper_fam_ev_world::cooked().1 });
        if crate::game::paper_bridge::on_refusable(game, "ew_campfire_done", payload) {
            return ::core::ops::ControlFlow::Break({ if let Some(stack) = crate::game::paper_fam_ev_world::take_result() { game.campfire_drop(pos, stack); } });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
