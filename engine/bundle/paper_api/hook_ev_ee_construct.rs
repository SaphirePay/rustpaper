// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_construct(game: &mut crate::game::Game<'_>, kind: &str, at: [f64; 3]) -> ::core::ops::ControlFlow<Option<u32>> {
    if crate::game::paper_bridge::hears("ee_construct") {
        let payload = serde_json::json!({ "kind": "construct", "level": crate::game::paper_fam_ev_entity::here(game), "mob": crate::game::paper_fam_ev_entity::mob_id(kind), "at": at, "blocks": crate::game::paper_fam_ev_entity::pattern() });
        if crate::game::paper_bridge::on_refusable(game, "ee_construct", payload) {
            return ::core::ops::ControlFlow::Break({ crate::game::paper_fam_ev_entity::restore_pattern(game); None });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
