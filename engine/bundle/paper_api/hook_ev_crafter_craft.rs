// Written by tools/gen_bundle.py.
fn paper_hook_ev_crafter_craft(game: &mut crate::game::Game<'_>, pos: Pos, stack: &ItemStack, recipe: &str) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("crafter_craft") {
        let payload = serde_json::json!({ "pos": [pos.0, pos.1, pos.2], "stack": crate::game::paper_query::stack_json(stack), "recipe": recipe });
        if crate::game::paper_bridge::on_refusable(game, "crafter_craft", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
