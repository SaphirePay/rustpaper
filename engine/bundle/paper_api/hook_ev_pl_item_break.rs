// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_item_break(game: &mut crate::game::Game<'_>, stack: &ItemStack) {
    if crate::game::paper_bridge::hears("pl_item_break") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id(), "stack": crate::game::paper_query::stack_json(stack), "broken": game.res.items().get(&stack.item).is_some_and(|item| item.max_damage > 0 && stack.damage >= item.max_damage) }); crate::game::paper_fam_ev_player::fire(game, "pl_item_break", payload); return } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_item_break", payload);
    }
}
