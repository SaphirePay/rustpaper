// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_consume(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_consume") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::consume_gate(game, stack, |game, eaten| game.res.items().get(&eaten.item).and_then(|info| info.consumable.clone().map(|consumable| (info.clone(), consumable))), |game, info, consumable, eaten| game.finish_consuming(info, consumable, eaten)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_consume", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
