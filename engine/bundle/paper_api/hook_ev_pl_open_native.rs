// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_open_native(game: &mut crate::game::Game<'_>, uuid: u128, rows: u8, title: &Text, items: &Vec<Option<ItemStack>>) -> ::core::ops::ControlFlow<Option<i32>> {
    if crate::game::paper_bridge::hears("pl_open_native") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::open_native(game, uuid, rows, items, |g, retitled| g.native_open_menu(uuid, rows, retitled.map_or_else(|| title.clone(), Text::literal), items.clone())) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_open_native", payload) {
            return ::core::ops::ControlFlow::Break(None);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
