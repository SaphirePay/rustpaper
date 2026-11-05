// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_fish_reel(game: &mut crate::game::Game<'_>, rod: &ItemStack) -> ::core::ops::ControlFlow<u32> {
    if crate::game::paper_bridge::hears("pl_fish_reel") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id(), "state": game.hook.as_ref().and_then(|h| if matches!(h.state, State::Hooked(_)) { Some("CAUGHT_ENTITY") } else if h.nibble > 0 { None } else if h.on_ground { Some("IN_GROUND") } else { Some("REEL_IN") }), "caught": game.hook.as_ref().and_then(|h| if let State::Hooked(id) = h.state { Some(id) } else { None }), "hook": game.hook.as_ref().map(|h| h.id), "hand": crate::game::paper_fam_ev_player::hand(game) }); if crate::game::paper_fam_ev_player::fire(game, "pl_fish_reel", payload) { return ::core::ops::ControlFlow::Break(0); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_fish_reel", payload) {
            return ::core::ops::ControlFlow::Break(0);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
