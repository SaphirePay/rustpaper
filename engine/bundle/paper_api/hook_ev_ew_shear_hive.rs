// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_shear_hive(game: &mut crate::game::Game<'_>, pos: [i32; 3], held: Option<&ItemStack>) -> ::core::ops::ControlFlow<Result<bool>> {
    if crate::game::paper_bridge::hears("ew_shear_hive") {
        let payload = serde_json::json!({ "pos": pos, "stack": held.map(crate::game::paper_query::stack_json), "hive": crate::game::paper_fam_ev_world::full_hive(game, pos) });
        if crate::game::paper_bridge::on_refusable(game, "ew_shear_hive", payload) {
            return ::core::ops::ControlFlow::Break(Ok(false));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
