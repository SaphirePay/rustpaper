// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_hanging(game: &mut crate::game::Game<'_>, index: usize, by_player: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_hanging") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "entity": game.paintings[index].id, "kind": if by_player { "entity" } else { "other" }, "cause": if by_player { "ENTITY" } else { "PHYSICS" } });
        if crate::game::paper_bridge::on_refusable(game, "ee_hanging", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
