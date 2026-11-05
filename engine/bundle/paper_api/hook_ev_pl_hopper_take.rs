// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_hopper_take(game: &mut crate::game::Game<'_>, slot: usize) -> ::core::ops::ControlFlow<Option<ItemStack>> {
    if crate::game::paper_bridge::hears("pl_hopper_take") {
        let payload = serde_json::json!({ "x": if true { if crate::game::paper_fam_ev_player::hopper_slot(slot) { return ::core::ops::ControlFlow::Break(None); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_hopper_take", payload) {
            return ::core::ops::ControlFlow::Break(None);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
