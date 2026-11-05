// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_open_boat(game: &mut crate::game::Game<'_>, i: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_open_boat") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::vehicle_later(game, game.boats[i].id, game.boats[i].items.clone(), |g, id| if let Some(i) = g.boats.iter().position(|b| b.id == id) { g.open_chest_boat_container(i) }) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_open_boat", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
