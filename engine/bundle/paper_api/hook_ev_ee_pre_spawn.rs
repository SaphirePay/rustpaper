// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_pre_spawn(game: &mut crate::game::Game<'_>, kind: &str, at: [f64; 3]) -> ::core::ops::ControlFlow<Option<u32>> {
    if crate::game::paper_bridge::hears("ee_pre_spawn") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "kind": crate::game::paper_fam_ev_entity::mob_id(kind), "at": at, "reason": horizon_client::mobs::ee::why_now() });
        if crate::game::paper_bridge::on_refusable(game, "ee_pre_spawn", payload) {
            return ::core::ops::ControlFlow::Break(None);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
