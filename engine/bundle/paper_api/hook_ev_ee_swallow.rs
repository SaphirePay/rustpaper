// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_swallow(game: &mut crate::game::Game<'_>, index: usize, item: &str) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("ee_swallow") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "entity": game.mobs[index].id, "item": item, "old": game.mobs[index].extra.head.as_deref(), "takes": sulfur_cube::swallowable(&game.mobs[index], item) && !game.mobs[index].is_baby() && !sulfur_cube::primed(&game.mobs[index]) });
        if crate::game::paper_bridge::on_refusable(game, "ee_swallow", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
