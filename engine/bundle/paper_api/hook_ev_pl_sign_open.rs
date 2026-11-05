// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_sign_open(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, edit: &SignEditor) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_sign_open") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "name": host.hub.guest(conn).map(|g| g.name.clone()), "pos": [edit.pos.0, edit.pos.1, edit.pos.2], "front": edit.front, "level": game.dim.id(), "cause": crate::game::paper_fam_ev_player::sign_cause(game) }); if crate::game::paper_fam_ev_player::fire(game, "pl_sign_open", payload) { return ::core::ops::ControlFlow::Break(()); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_sign_open", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
