// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_armor(game: &mut crate::game::Game<'_>, slot: &str, old: Option<&ItemStack>, new: Option<&ItemStack>) {
    if crate::game::paper_bridge::hears("pl_armor") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id(), "slot": slot, "old": old.map(crate::game::paper_fam_ev_player::stack_value), "new": new.map(crate::game::paper_fam_ev_player::stack_value) }); crate::game::paper_fam_ev_player::fire(game, "pl_armor", payload); return } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_armor", payload);
    }
}
