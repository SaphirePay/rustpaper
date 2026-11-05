// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_block_drops(game: &mut crate::game::Game<'_>, pos: [i32; 3], state: BlockStateId, player: Option<u128>, explosion: Option<f32>, drops: &mut Vec<ItemStack>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_block_drops") {
        let payload = serde_json::json!({ "uuid": player.map(|player| format!("{player:032x}")), "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": pos, "state": crate::game::paper_fam_ev_world::state_text(game, state), "explosion": explosion, "items": drops.iter().map(crate::game::paper_query::stack_json).collect::<Vec<_>>() });
        if crate::game::paper_bridge::on_refusable(game, "ew_block_drops", payload) {
            return ::core::ops::ControlFlow::Break(crate::game::paper_fam_ev_world::put_drops(drops));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
