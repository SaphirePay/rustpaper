// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_loot(game: &mut crate::game::Game<'_>, context: &NativeLootDrops<'_>, drops: &mut Vec<ItemStack>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_loot") {
        let payload = serde_json::json!({ "uuid": context.player.map(|player| format!("{player:032x}")), "level": format!("minecraft:{}", context.dimension.trim_start_matches("minecraft:")), "table": context.table, "origin": context.origin, "entity": context.entity.map(|entity| entity.0), "items": drops.iter().map(crate::game::paper_query::stack_json).collect::<Vec<_>>() });
        if crate::game::paper_bridge::on_refusable(game, "ew_loot", payload) {
            return ::core::ops::ControlFlow::Break(crate::game::paper_fam_ev_world::put_drops(drops));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
