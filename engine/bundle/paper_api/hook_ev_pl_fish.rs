// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_fish(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_fish") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id(), "state": (&*stack.item == "minecraft:fishing_rod" && game.hook.is_none()).then_some("FISHING"), "hand": crate::game::paper_fam_ev_player::hand(game) }); if crate::game::paper_fam_ev_player::fire(game, "pl_fish", payload) { return ::core::ops::ControlFlow::Break(false); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_fish", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
