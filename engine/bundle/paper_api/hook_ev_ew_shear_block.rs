// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_shear_block(game: &mut crate::game::Game<'_>, pos: [i32; 3], held: Option<&ItemStack>) -> ::core::ops::ControlFlow<Result<bool>> {
    if crate::game::paper_bridge::hears("ew_shear_block") {
        let payload = serde_json::json!({ "pos": pos, "stack": held.map(crate::game::paper_query::stack_json), "pumpkin": &*game.res.facts().get(game.world.block(pos[0], pos[1], pos[2])).name == "minecraft:pumpkin" });
        if crate::game::paper_bridge::on_refusable(game, "ew_shear_block", payload) {
            return ::core::ops::ControlFlow::Break(Ok(false));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
