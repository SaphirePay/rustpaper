// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_portal(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, target: Dim, plan: &Option<Plan>) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_portal") {
        let payload = serde_json::json!({ "x": if true { let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "name": host.hub.guest(conn).map(|g| g.name.clone()), "level": target.id(), "from": game.dim.id(), "back": host.hub.guest(conn).map(|g| g.position), "to": plan.as_ref().map(|plan| [plan.approximate.x, plan.approximate.y, plan.approximate.z]).or(if target == Dim::End { Some(super::end::END_SPAWN) } else { game.level.world_spawn }), "cause": if plan.is_some() { "NETHER_PORTAL" } else { "END_PORTAL" } }); if crate::game::paper_fam_ev_player::fire(game, "pl_portal", payload) { return ::core::ops::ControlFlow::Break(()); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_portal", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
