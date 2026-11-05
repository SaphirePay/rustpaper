// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_sign_command(game: &mut crate::game::Game<'_>, pos: [i32; 3], command: &str) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_sign_command") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": format!("{:032x}", game.native_acting_id()), "name": game.acting_player_name(), "level": game.dim.id(), "pos": pos, "command": command, "front": crate::game::paper_fam_ev_player::sign_front() }); if crate::game::paper_fam_ev_player::fire(game, "pl_sign_command", payload) { return ::core::ops::ControlFlow::Break(()); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_sign_command", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
