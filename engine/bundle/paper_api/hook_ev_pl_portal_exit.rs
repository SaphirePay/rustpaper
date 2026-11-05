// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_portal_exit(game: &mut crate::game::Game<'_>, plan: &Plan) -> ::core::ops::ControlFlow<Option<FoundRectangle>> {
    if crate::game::paper_bridge::hears("pl_portal_exit") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::portal_gate([plan.approximate.x, plan.approximate.y, plan.approximate.z], game, |g, radius| { let to_nether = plan.dim == Dim::Nether; let border = g.border_bounds(); let portals = g.portal_blocks(plan.approximate, radius.unwrap_or(if to_nether { 16 } else { 128 })); let (low, high) = if to_nether { (0, 15) } else { (-4, 19) }; closest_portal(&portals, plan.approximate, to_nether, &border, low, high).is_some() }, None) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_portal_exit", payload) {
            return ::core::ops::ControlFlow::Break(None);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
