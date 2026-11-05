// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_shear_entity(game: &mut crate::game::Game<'_>, pos: Pos, facing: Direction, stack: &ItemStack) -> ::core::ops::ControlFlow<Result<Dispensed>> {
    if crate::game::paper_bridge::hears("ew_shear_entity") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": [pos.0, pos.1, pos.2], "stack": crate::game::paper_query::stack_json(stack), "target": game.targets_at(offset(pos, facing)).iter().find_map(|target| match target { Target::Mob(i) => crate::game::paper_fam_ev_world::shearable(&game.mobs[*i]).map(|_| game.mobs[*i].id), _ => None }), "parent": game.targets_at(offset(pos, facing)).iter().find_map(|target| match target { Target::Mob(i) => crate::game::paper_fam_ev_world::shearable(&game.mobs[*i]).map(|tables| tables.0), _ => None }), "table": game.targets_at(offset(pos, facing)).iter().find_map(|target| match target { Target::Mob(i) => crate::game::paper_fam_ev_world::shearable(&game.mobs[*i]).map(|tables| tables.1), _ => None }) });
        if crate::game::paper_bridge::on_refusable(game, "ew_shear_entity", payload) {
            return ::core::ops::ControlFlow::Break(Ok((Some(stack.clone()), FAIL)));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
