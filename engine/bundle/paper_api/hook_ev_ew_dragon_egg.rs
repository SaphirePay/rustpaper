// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_dragon_egg(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("ew_dragon_egg") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "egg": !game.fight.previously_killed, "pos": [0, (0..256).rev().find(|&y| { let facts = game.res.facts().get(game.world.block(0, y, 0)); facts.full_collision || facts.legacy_solid }).map_or(64, |y| y + 1), 0] });
        crate::game::paper_bridge::on_trigger(game, "ew_dragon_egg", payload);
    }
}
