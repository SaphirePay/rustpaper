// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_camera(game: &mut crate::game::Game<'_>, host: &LanHost, gi: usize, camera: Option<i32>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_camera") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::camera_gate(game, host, gi, game.cmds.guest_cameras.get(&host.hub.guests[gi].conn).map(|c| c.id), camera) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_camera", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
