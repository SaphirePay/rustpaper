// Written by tools/gen_bundle.py.
fn paper_hook_ev_potion_splash(game: &mut crate::game::Game<'_>, stack: &ItemStack, at: Vec3) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("potion_splash") {
        let payload = serde_json::json!({ "stack": crate::game::paper_query::stack_json(stack), "at": [at.x, at.y, at.z] });
        if crate::game::paper_bridge::on_refusable(game, "potion_splash", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
