// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_crafter(game: &mut crate::game::Game<'_>, pos: Pos, facing: Direction, stack: &ItemStack, recipe: &str) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_crafter") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": [pos.0, pos.1, pos.2], "stack": crate::game::paper_query::stack_json(stack), "recipe": recipe });
        if crate::game::paper_bridge::on_refusable(game, "ew_crafter", payload) {
            return ::core::ops::ControlFlow::Break({ if let Some(stack) = crate::game::paper_fam_ev_world::take_result() { game.crafter_eject(pos, facing, stack, recipe); } });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
