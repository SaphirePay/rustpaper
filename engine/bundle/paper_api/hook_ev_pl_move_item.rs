// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_move_item(game: &mut crate::game::Game<'_>, from_hopper: Option<Pos>, view: &View, stack: &ItemStack, face: Option<Direction>) -> ::core::ops::ControlFlow<Option<ItemStack>> {
    if crate::game::paper_bridge::hears("pl_move_item") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": null, "again": crate::game::paper_fam_ev_player::moved_again(), "from": from_hopper.map(|pos| [pos.0, pos.1, pos.2]).or(crate::game::paper_fam_ev_player::pulled()), "pull": crate::game::paper_fam_ev_player::pulled().is_some(), "to": view.parts.first().map(|part| [part.0.0, part.0.1, part.0.2]), "stack": crate::game::paper_query::stack_json(stack), "level": game.dim.id(), "item": crate::game::paper_fam_ev_player::sucked_item(game, from_hopper.is_none() && crate::game::paper_fam_ev_player::pulled().is_none(), view.parts.first().map(|part| part.0), stack) }); if crate::game::paper_fam_ev_player::fire(game, "pl_move_item", payload) { return ::core::ops::ControlFlow::Break(match crate::game::paper_fam_ev_player::take_moved() { Some(new) => game.add_item(from_hopper, view, new, face), None => Some(stack.clone()) }); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_move_item", payload) {
            return ::core::ops::ControlFlow::Break(match crate::game::paper_fam_ev_player::take_moved() { Some(new) => game.add_item(from_hopper, view, new, face), None => Some(stack.clone()) });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
