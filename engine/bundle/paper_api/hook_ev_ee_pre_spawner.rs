// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_pre_spawner(game: &mut crate::game::Game<'_>, planned: &Vec<Planned>) -> ::core::ops::ControlFlow<HashMap<u32, u32>> {
    if crate::game::paper_bridge::hears("ee_pre_spawner") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "reason": horizon_client::mobs::ee::why_now(), "kind": planned.iter().find_map(|plan| match plan { Planned::Finalized(mob, _) => Some(mob.kind.id.clone()), Planned::Loaded(..) => None }), "at": planned.iter().find_map(|plan| match plan { Planned::Finalized(mob, _) => Some([mob.position.x, mob.position.y, mob.position.z]), Planned::Loaded(..) => None }) });
        if crate::game::paper_bridge::on_refusable(game, "ee_pre_spawner", payload) {
            return ::core::ops::ControlFlow::Break(HashMap::new());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
