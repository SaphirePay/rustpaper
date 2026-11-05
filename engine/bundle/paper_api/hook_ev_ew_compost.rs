// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_compost(game: &mut crate::game::Game<'_>, pos: [i32; 3], held: Option<&ItemStack>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("ew_compost") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": pos, "stack": held.map(crate::game::paper_query::stack_json), "levels": held.and_then(|stack| game.compostable(stack)).map(|layers| { let saved = game.world.random.clone(); let levels = game.provider_int(&layers, &ComputeContext { block: Some(pos), target: None }, 0).unwrap_or(0); game.world.random = saved; levels }) });
        if crate::game::paper_bridge::on_refusable(game, "ew_compost", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
