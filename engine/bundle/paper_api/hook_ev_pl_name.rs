// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_name(game: &mut crate::game::Game<'_>, index: usize) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_name") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::name_gate(game, index, |game, index| game.name_mob(index)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_name", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
