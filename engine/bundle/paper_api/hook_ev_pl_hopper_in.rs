// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_hopper_in(game: &mut crate::game::Game<'_>, pos: Pos, hopper: &View) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_hopper_in") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::hopper_gate(Some((pos.0, pos.1 + 1, pos.2)), game, pos, true, |g: &mut Game<'_>, at: Pos| { let c = g.containers(); c.view(at, false).map(|v| (0..v.len()).map(|slot| c.get(&v, slot).cloned()).collect::<Vec<_>>()) }, |g, at| match at { Some(at) if at == (pos.0, pos.1 + 1, pos.2) => g.hopper_suck(pos, hopper), Some(at) => match g.containers().view(at, false) { Some(source) => g.suck_from_slots(None, &source, hopper), None => false }, None => { let (min, max) = ([f64::from(pos.0), f64::from(pos.1) + 11.0 / 16.0, f64::from(pos.2)], [f64::from(pos.0) + 1.0, f64::from(pos.1) + 2.0, f64::from(pos.2) + 1.0]); g.suck_item_entities(hopper, min, max) } }) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_hopper_in", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
