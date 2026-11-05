// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_shear(game: &mut crate::game::Game<'_>, index: usize, item: &str) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_shear") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::shear_golem(game, index, item, |g| { let m = &g.mobs[index]; if !m.alive() || m.extra.sheared { return None; } let table = match m.species() { Species::SnowGolem => "minecraft:shearing/snow_golem", Species::Bogged => "minecraft:shearing/bogged", _ => return None }; let tool = g.held().cloned(); Some(g.sequence_loot(table, table, tool.as_ref())) }) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_shear", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
