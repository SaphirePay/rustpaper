// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_hopper_out(game: &mut crate::game::Game<'_>, pos: Pos, hopper: &View) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_hopper_out") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::hopper_gate({ let state = game.world.block(pos.0, pos.1, pos.2); game.containers().prop(state, "facing").and_then(dir_from_name).map(|facing| offset(pos, facing)) }, game, pos, false, |g: &mut Game<'_>, at: Pos| { let c = g.containers(); c.view(at, false).map(|v| (0..v.len()).map(|slot| c.get(&v, slot).cloned()).collect::<Vec<_>>()) }, |g, at| { let state = g.world.block(pos.0, pos.1, pos.2); let face = g.containers().prop(state, "facing").and_then(dir_from_name); match (at, face) { (Some(at), Some(face)) if at == offset(pos, face) => g.hopper_eject(pos, hopper), (Some(at), Some(face)) => match g.containers().view(at, false) { Some(target) => g.eject_into(pos, hopper, &target, face.opposite()), None => false }, _ => false } }) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_hopper_out", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
