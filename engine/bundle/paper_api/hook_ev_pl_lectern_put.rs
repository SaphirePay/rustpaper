// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_lectern_put(game: &mut crate::game::Game<'_>, pos: [i32; 3], held: Option<&ItemStack>) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_lectern_put") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::book_gate(game, pos, held, |game, pos, held| game.use_lectern(pos, held)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_lectern_put", payload) {
            return ::core::ops::ControlFlow::Break(true);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
