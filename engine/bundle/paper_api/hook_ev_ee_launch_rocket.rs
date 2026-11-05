// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_launch_rocket(game: &mut crate::game::Game<'_>, at: Vec3, stack: &ItemStack) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_launch_rocket") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "kind": "minecraft:firework_rocket", "stack": crate::game::paper_query::stack_json(stack), "at": [at.x, at.y, at.z] });
        if crate::game::paper_bridge::on_refusable(game, "ee_launch_rocket", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
