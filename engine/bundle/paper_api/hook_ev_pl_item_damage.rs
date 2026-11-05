// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_item_damage(game: &mut crate::game::Game<'_>, stack: &ItemStack, amount: u32) -> ::core::ops::ControlFlow<u32> {
    if crate::game::paper_bridge::hears("pl_item_damage") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::wear_gate(game, stack, amount) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_item_damage", payload) {
            return ::core::ops::ControlFlow::Break(0);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
