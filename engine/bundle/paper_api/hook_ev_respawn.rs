// Written by tools/gen_bundle.py.
fn paper_hook_ev_respawn(game: &mut crate::game::Game<'_>, c: RespawnConfig, consume: bool) -> ::core::ops::ControlFlow<Result<Option<(Vec3, f32)>>> {
    if crate::game::paper_bridge::hears("respawn") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::respawn_gate(game, c.pos, consume, |found: &Result<Option<(Vec3, f32)>>| matches!(found, Ok(None)), |g| g.find_respawn_and_use(c, consume)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "respawn", payload) {
            return ::core::ops::ControlFlow::Break(Ok(None));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
